//! Phases 3 and 4: the inverse BWT. The T-vector (lbzip2's packing: a row's byte in bits
//! 0-7, the next row above) is built by a counting sort, then walked from the primary
//! index. The walk is a chain of dependent loads through 3.6 MB, so it is bound by the
//! load's latency, not by bandwidth (docs/journal/2026-10-27-compute-frontier.md, section
//! 2). `walk_segments` walks it as K chains at once (memory-level parallelism within one
//! block) and joins them: exactly the plain walk's output, in about half its time.
//!
//! Interleaving several traversals of one inversion on one thread, so that out-of-order
//! execution keeps their loads in flight at once, is not new: Kärkkäinen, Kempa and
//! Puglisi, "Slashing the Time for BWT Inversion", DCC 2012, pp. 99-108, do it, and add
//! a super-alphabet (an LF^2 table: two characters a dependent lookup) and "Precopy" for
//! repetitive data. What is particular here is only how the traversals are found in an
//! existing bzip2 file, which carries no metadata for it: pseudo-random rows as segment
//! starts, a bitmap of them to stop each segment, and the segments joined in order.
//!
//! The kernels sit behind `InverseBwt` (a block's last column in, its RLE1-coded text
//! out), so another (an LF^2 walk: a pair table, with a flag for an intermediate row that
//! is a segment start) can be put in without touching the stream, CRC or thread code.

pub use crate::cells::{Cells, CellsKernel, Counts as CellsCounts, ForcedCells};

/// An inverse-BWT kernel: a block's RLE1-coded text from its last column `ll` (whose
/// bytes `counts` counts) and its primary index (below `ll.len()`: the caller checks),
/// into `out` (cleared). Its buffers are its own, kept from block to block.
pub trait InverseBwt: Default + Send {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>);

    /// Whether this kernel reads the MTF stage's count checkpoints (`mtf::mtf_checkpoints`,
    /// `mtf::CHECKPOINT` rows apart); if so the decoder calls `invert_checkpointed`
    fn wants_checkpoints(&self) -> bool {
        false
    }

    /// `invert`, given `checkpoints[i]`, the counts of `ll[..i * mtf::CHECKPOINT]`
    fn invert_checkpointed(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        orig_ptr: u32,
        out: &mut Vec<u8>,
    ) {
        let _ = checkpoints;
        self.invert(ll, counts, orig_ptr, out)
    }

    /// Its buffers' sizes, for the footprint ledger (src/trace.rs): tests and the `trace`
    /// feature only
    #[cfg(any(test, feature = "trace"))]
    fn buffers(&self) -> Vec<crate::trace::Buffer> {
        Vec::new()
    }

    /// `invert` with `ll` as both its input and its output, for a kernel that reads `ll`
    /// only while building its table: the block's bytes are then written over `ll`, so the
    /// decoder needs no separate output buffer (about one block a worker).
    ///
    /// The preconditions are `invert`'s: `counts` counts `ll`, and `orig_ptr` is below
    /// `ll.len()` (the caller checks).
    ///
    /// **On success** it returns true, and then:
    /// - `ll` holds exactly the bytes `invert` would have put in its `out` for the same
    ///   `ll`, `counts` and `orig_ptr`, so its length is the original `ll.len()`;
    /// - it doesn't grow `ll`: the output is no longer than the input, so `ll`'s buffer
    ///   and capacity are unchanged;
    /// - the kernel's own buffers are as `invert` leaves them, ready for the next block.
    ///
    /// **On fallback** it returns false and has changed nothing: not `ll` (contents,
    /// length or buffer), not the kernel. A kernel that doesn't support it returns false,
    /// and the decoder then calls `invert` with a buffer of its own. A kernel returns true
    /// for every block or false for every block, never by the block.
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], orig_ptr: u32) -> bool {
        let _ = (ll, counts, orig_ptr);
        false
    }

    /// Whether this kernel builds its table from a block's run records (`invert_runs`), so
    /// that the decoder may take a block of few symbols by the records' path
    /// (`runs::takes_runs`) instead of the column's. A kernel answers the same for every
    /// block.
    fn reads_runs(&self) -> bool {
        false
    }

    /// `invert` for the block of `n` rows whose run records `runs::front` left in `r`: the
    /// LF table `runs::fill` builds from them, walked backward, its bytes into `out`
    /// (cleared): exactly what `invert` gives for the block's column. `orig_ptr` is below
    /// n (the caller checks). Returns false if the records don't tile the block (a bug in
    /// the front: the caller then takes the column's path, which clears `out` itself).
    /// Called only when `reads_runs` is true.
    fn invert_runs(
        &mut self,
        r: &crate::runs::Runs,
        n: usize,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        k: crate::Kernels,
    ) -> bool {
        let _ = (r, n, orig_ptr, out, k);
        unreachable!("invert_runs on a kernel that doesn't read runs");
    }
}

/// The plain walk: one chain of dependent loads (lbzip2's and libbzip2's)
#[derive(Default)]
pub struct Plain {
    tt: TVector,
}

impl InverseBwt for Plain {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        build(ll, counts, &mut self.tt);
        walk(&self.tt, orig_ptr, out);
    }

    #[cfg(any(test, feature = "trace"))]
    fn buffers(&self) -> Vec<crate::trace::Buffer> {
        vec![crate::trace::Buffer::of("tt", &self.tt.0)]
    }
}

/// The segmented walk in passes (`walk_segments_room`), the plain one for a short block:
/// the default
#[derive(Default)]
pub struct Segmented {
    tt: TVector,
    seg: SegScratch,
}

impl InverseBwt for Segmented {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        build(ll, counts, &mut self.tt);
        walk_segments_room(&mut self.tt, orig_ptr, out, &mut self.seg);
    }

    // The build is `ll`'s last read: it copies every byte and link the walk needs into the
    // table. The walk (segmented, or plain for a short block) then reads only the table and
    // the segment scratch, so its output can go over `ll`. Both writers (`walk`,
    // `SegScratch::assemble`) clear their output and write exactly the table's n rows, and
    // n = ll.len() fits the capacity `ll` already has. So the success contract holds, and
    // Segmented never falls back.
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], orig_ptr: u32) -> bool {
        build(ll, counts, &mut self.tt);
        walk_segments_room(&mut self.tt, orig_ptr, ll, &mut self.seg);
        true
    }

    fn reads_runs(&self) -> bool {
        true
    }

    // `runs::fill` sets the table only once the records tile the block, and then every link
    // in it is below its length, as the walks require
    fn invert_runs(
        &mut self,
        r: &crate::runs::Runs,
        n: usize,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        k: crate::Kernels,
    ) -> bool {
        if !crate::runs::fill(r, n, &mut self.tt, k) {
            return false;
        }
        walk_back_room(&mut self.tt, orig_ptr, out, &mut self.seg);
        true
    }

    #[cfg(any(test, feature = "trace"))]
    fn buffers(&self) -> Vec<crate::trace::Buffer> {
        use crate::trace::Buffer;
        let s = &self.seg;
        let mut v = vec![Buffer::of("tt", &self.tt.0)];
        if s.bufs.capacity() > 0 {
            v.push(Buffer {
                name: "seg.bufs",
                len: s.bufs.iter().map(Vec::len).sum(),
                cap: s.bufs.iter().map(Vec::capacity).sum(),
            });
            v.push(Buffer::of("seg.bufs.outer", &s.bufs));
            v.push(Buffer::of("seg.starts", &s.starts));
            v.push(Buffer::of("seg.next", &s.next));
        }
        if s.bitmap.capacity() > 0 {
            v.push(Buffer::of("seg.bitmap", &s.bitmap));
        }
        v
    }
}

/// The cooperative inverse BWT: a block's table built and walked by K cores that share it
/// (`coop::build_fused`, then `coop::walk_room`, its threads kept for the kernel's life:
/// the coop journal, docs/journal/2026-10-28-cooperative-inversion.md). N workers on this
/// kernel keep N tables live on N × K cores, where production's keep N × K. Exactly
/// `walk`'s output.
///
/// It runs the one-core kernel (`Segmented`) where cooperation can't help or can't run: a
/// block below `MIN_COOP` rows, a call without checkpoints, or a pool whose threads
/// couldn't be spawned. If the cooperative build refuses the checkpoints, a bug in their
/// maker, the block is built on one core instead, so its output is still right (and a
/// debug build stops at the assertion).
///
/// A block of few symbols comes as its run records (`invert_runs`), and the team then fills
/// their LF table by shares and walks it backward (`coop::build_runs`,
/// `coop::walk_room_back`), with the same fallbacks to the one-core kernel.
pub struct Cooperative<const K: usize> {
    /// None until the first cooperative block; an Err if its threads couldn't be spawned
    pool: Option<std::io::Result<crate::coop::Pool>>,
    tt: TVector,
    seg: SegScratch,
    /// a block's records' cuts, for the team's fill (`invert_runs`)
    cuts: Vec<crate::runs::Cut>,
}

/// The rows below which `Cooperative` runs the one-core kernel: 16 checkpoints, 4 ×
/// `MIN_SEGMENTED`, the ranges' rounding then at most 1/32 of a range (to be tuned by the
/// decoder A/B)
pub const MIN_COOP: usize = 16 * crate::mtf::CHECKPOINT;

impl<const K: usize> Cooperative<K> {
    /// Whether a block has been inverted on the team (its threads spawned)
    #[cfg(test)]
    pub(crate) fn engaged(&self) -> bool {
        matches!(self.pool, Some(Ok(_)))
    }
}

impl<const K: usize> Default for Cooperative<K> {
    fn default() -> Self {
        Cooperative {
            pool: None,
            tt: TVector::new(),
            seg: SegScratch::default(),
            cuts: Vec::new(),
        }
    }
}

impl<const K: usize> InverseBwt for Cooperative<K> {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        build(ll, counts, &mut self.tt);
        walk_segments_room(&mut self.tt, orig_ptr, out, &mut self.seg);
    }

    fn wants_checkpoints(&self) -> bool {
        K > 1
    }

    fn invert_checkpointed(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        orig_ptr: u32,
        out: &mut Vec<u8>,
    ) {
        if K < 2 || ll.len() < MIN_COOP {
            return self.invert(ll, counts, orig_ptr, out);
        }
        let pool = match self.pool.get_or_insert_with(|| crate::coop::Pool::new(K)) {
            Ok(p) => p,
            Err(_) => return self.invert(ll, counts, orig_ptr, out),
        };
        let built = crate::coop::build_fused(
            ll,
            counts,
            checkpoints,
            crate::mtf::CHECKPOINT,
            pool,
            &mut self.tt,
        );
        if let Err(e) = built {
            debug_assert!(
                false,
                "the cooperative build refused its checkpoints: {}",
                e.0
            );
            build(ll, counts, &mut self.tt);
        }
        crate::coop::walk_room::<FLAGGED_LANES>(
            &mut self.tt,
            orig_ptr,
            &mut self.seg,
            pool,
            FLAGGED_SEGMENTS * K,
        );
        self.seg.assemble(out);
    }

    fn reads_runs(&self) -> bool {
        true
    }

    // The team's fill and walk of the records' LF table (`coop::build_runs`, then
    // `coop::walk_room_back`), where `invert_checkpointed` runs the team's build and walk
    // of the column's; the one-core kernel's (`Segmented::invert_runs`) where it runs that.
    // The cuts are the tiling's check (`runs::cuts`): refused, the records are not the
    // block's, and the caller takes the column's path.
    fn invert_runs(
        &mut self,
        r: &crate::runs::Runs,
        n: usize,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        k: crate::Kernels,
    ) -> bool {
        let pool = if K < 2 || n < MIN_COOP {
            None
        } else {
            self.pool
                .get_or_insert_with(|| crate::coop::Pool::new(K))
                .as_ref()
                .ok()
        };
        let Some(pool) = pool else {
            if !crate::runs::fill(r, n, &mut self.tt, k) {
                return false;
            }
            walk_back_room(&mut self.tt, orig_ptr, out, &mut self.seg);
            return true;
        };
        if !crate::runs::cuts(r, n, &mut self.cuts) {
            return false;
        }
        if let Err(e) = crate::coop::build_runs(r, &self.cuts, n, k, pool, &mut self.tt) {
            debug_assert!(false, "the team's fill failed its check: {}", e.0);
            if !crate::runs::fill(r, n, &mut self.tt, k) {
                return false;
            }
        }
        crate::coop::walk_room_back::<FLAGGED_LANES>(
            &mut self.tt,
            orig_ptr,
            &mut self.seg,
            pool,
            FLAGGED_SEGMENTS * K,
        );
        self.seg.assemble_reversed(out);
        true
    }

    #[cfg(any(test, feature = "trace"))]
    fn buffers(&self) -> Vec<crate::trace::Buffer> {
        use crate::trace::Buffer;
        let s = &self.seg;
        let mut v = vec![Buffer::of("tt", &self.tt.0)];
        if s.bufs.capacity() > 0 {
            v.push(Buffer {
                name: "seg.bufs",
                len: s.bufs.iter().map(Vec::len).sum(),
                cap: s.bufs.iter().map(Vec::capacity).sum(),
            });
            v.push(Buffer::of("seg.bufs.outer", &s.bufs));
            v.push(Buffer::of("seg.starts", &s.starts));
            v.push(Buffer::of("seg.next", &s.next));
        }
        if self.cuts.capacity() > 0 {
            v.push(Buffer::of("cuts", &self.cuts));
        }
        v
    }
}

/// A block's T-vector. Built only by `build`, or by `coop::build_fused` once its end check
/// has passed, so every link in it is below its length, which the walks rely on to read
/// it unchecked.
#[derive(Default)]
pub struct TVector(pub(crate) Vec<u32>);

impl TVector {
    pub fn new() -> Self {
        TVector(Vec::new())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The T-vector of the last column `ll`, whose bytes `counts` counts.
///
/// Built in one pass (`build_one_pass`) when the counts are `ll`'s own, as the decoder's
/// always are; otherwise by `build_two_pass`, whose table is defined for any counts that
/// keep its writes in bounds. Both give the same table where both apply.
pub fn build(ll: &[u8], counts: &[u32; 256], tt: &mut TVector) {
    if !build_one_pass(ll, counts, &mut tt.0) {
        build_two_pass(ll, counts, &mut tt.0);
    }
}

/// `build` in one pass, as `coop::build_fused` builds at one core: each entry written whole,
/// `ll[s] | i << 8` at slot s, with no pass first copying the last column in and no
/// read-modify-write of each entry. Each write is checked against its bucket's end, and
/// after the scatter every bucket's cursor must have reached its end, which proves every
/// slot below n last written with its entry. Only then is the table's length set; otherwise it
/// is left empty and false returned (the counts are not `ll`'s own, or don't sum to n).
fn build_one_pass(ll: &[u8], counts: &[u32; 256], tt: &mut Vec<u32>) -> bool {
    let n = ll.len();
    tt.clear();
    if n >= 1 << 24 || counts.iter().map(|&c| c as usize).sum::<usize>() != n {
        return false;
    }
    let (mut ftab, mut lim) = ([0u32; 256], [0u32; 256]);
    let mut cum = 0u32;
    for b in 0..256 {
        ftab[b] = cum;
        cum += counts[b];
        lim[b] = cum;
    }
    tt.reserve(n);
    let t = tt.as_mut_ptr();
    // the one-row step: row i's link at its byte's next slot, inside the bucket
    let one = |i: usize, ftab: &mut [u32; 256]| -> bool {
        let c = row_byte(ll, i);
        let s = ftab[c];
        if s >= lim[c] {
            return false;
        }
        // SAFETY: s is below its bucket's end, at most n, within tt's reserved capacity
        unsafe { *t.add(s as usize) = ll[s as usize] as u32 | (i as u32) << 8 };
        ftab[c] = s + 1;
        true
    };
    // 16 rows at a time, a window cut into its runs of equal bytes. Equal bytes in a
    // window are consecutive rows of their byte's bucket (rows i..i + L take its next L
    // slots in order), so a run of L takes one cursor step of L, and its entries go in
    // with one 16-wide write (`run16`): libsais's RUN_LANES, made to start at any row and
    // to end before 16. A window of one run is that run of 16. Row by row, each row of a
    // run would reload the cursor its predecessor has just stored: a store-forward chain
    // as long as the run. A window of more than RUNS runs is mostly unequal neighbours and
    // takes the one-row steps, which then have no such chain: a window of 16 runs (most of
    // a high-entropy block's) is sent there by one compare, before the count of its runs.
    let mut i = 0;
    while i + 16 <= n {
        // bit j set: row i + j continues a run (j > 0, and its byte is row i + j - 1's)
        let joins = run_joins(&ll[i..i + 16]);
        if joins == ALL_JOINED {
            // one run of 16, most of Wikidata's windows
            let c = row_byte(ll, i);
            let s = ftab[c] as usize;
            if s + 16 > lim[c] as usize {
                return false;
            }
            // SAFETY: s..s + 16 is below its bucket's end, at most n: inside ll and inside
            // tt's reserved capacity
            unsafe { run16(ll.as_ptr().add(s), (i as u32) << 8, t.add(s)) };
            ftab[c] = (s + 16) as u32;
        } else if joins != 0 && few_runs(joins) {
            // bit j set: row i + j starts a run
            let mut heads = joins ^ 0xffff;
            while heads != 0 {
                let j = heads.trailing_zeros() as usize;
                heads &= heads - 1;
                let len = (heads | 1 << 16).trailing_zeros() as usize - j;
                let c = row_byte(ll, i + j);
                let s = ftab[c] as usize;
                if s + 16 <= lim[c] as usize {
                    // SAFETY: s..s + 16 is below its bucket's end, at most n: inside ll
                    // and inside tt's reserved capacity. The run's own len entries are
                    // final. The rest, s + len..s + 16, are slots of this bucket above its
                    // new cursor, so each is written again by the step that reaches it,
                    // before the end check can pass (the safety comment there)
                    unsafe { run16(ll.as_ptr().add(s), ((i + j) as u32) << 8, t.add(s)) };
                    ftab[c] = (s + len) as u32;
                } else {
                    // near its bucket's end: the run's rows one at a time
                    for i in i + j..i + j + len {
                        if !one(i, &mut ftab) {
                            return false;
                        }
                    }
                }
            }
        } else {
            for i in i..i + 16 {
                if !one(i, &mut ftab) {
                    return false;
                }
            }
        }
        i += 16;
    }
    for i in i..n {
        if !one(i, &mut ftab) {
            return false;
        }
    }
    if ftab != lim {
        return false;
    }
    // SAFETY: every slot below n was last written with its entry: each bucket's slots by
    // its own cursor, which rose from the bucket's start to its end (the check above), each
    // step writing the entries of the slots it passed. A 16-wide write also wrote slots
    // above its step's end, inside the bucket; the cursor then passed each of them in a
    // later step, which wrote its entry over it. The buckets tile [0, n)
    unsafe { tt.set_len(n) };
    true
}

/// The most runs a 16-row window may have for `build_one_pass` to take it run by run
/// (each run one cursor step and one 16-wide write); a window of more runs takes the
/// one-row steps
const RUNS: usize = 6;

/// Row i's byte, `ll[i]`, read from memory (a volatile read, bounds checked as `ll[i]`).
/// Its window's 16 bytes are already in a vector (`run_joins`), and with a plain read the
/// compiler took them from there: it stored the vector to the stack and copied out each
/// byte, about 30 instructions on every window of more than one run, high-entropy
/// blocks' above all. Read from `ll`, each is one load, as on master
#[inline(always)]
fn row_byte(ll: &[u8], i: usize) -> usize {
    // SAFETY: a reference to the byte, so valid and aligned for the read
    unsafe { core::ptr::read_volatile(&ll[i]) as usize }
}

/// `run_joins` of a window of one run
const ALL_JOINED: u32 = 0xfffe;

/// Whether a window with these `run_joins` has at most RUNS runs: its heads (`joins ^
/// 0xffff`) number at most RUNS, so its joins at least 16 - RUNS. Counted without POPCNT,
/// which baseline x86-64 lacks (#310's software popcount was about a dozen instructions on
/// every window of more than one run), from the counts of the two bytes: two loads from a
/// 256-byte table. `build_one_pass` asks only for windows of more than one run and fewer
/// than 16. Tested against `count_ones` on all 32,768 masks `run_joins` can return
#[inline(always)]
fn few_runs(joins: u32) -> bool {
    POP8[joins as u8 as usize] + POP8[(joins >> 8) as u8 as usize] >= (16 - RUNS) as u8
}

/// The number of bits set in each byte
static POP8: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        t[i] = (i as u32).count_ones() as u8;
        i += 1;
    }
    t
};

/// The rows of a 16-byte window that continue a run: bit j is set when j > 0 and `w[j]`
/// equals `w[j - 1]`. A window of one run has all 15 (`ALL_JOINED`), one of 16 runs none:
/// each one compare, with no count of its runs
#[inline(always)]
fn run_joins(w: &[u8]) -> u32 {
    let w: &[u8; 16] = w.try_into().unwrap();
    #[cfg(target_arch = "x86_64")]
    // SAFETY: SSE2 is in x86_64's baseline; w is 16 readable bytes
    unsafe {
        use core::arch::x86_64::*;
        let v = _mm_loadu_si128(w.as_ptr().cast());
        // byte j of `prev` is w[j - 1] (byte 0 is 0, and bit 0 is cleared below whatever)
        let prev = _mm_slli_si128::<1>(v);
        _mm_movemask_epi8(_mm_cmpeq_epi8(v, prev)) as u32 & ALL_JOINED
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let mut joins = 0;
        for j in 1..16 {
            joins |= ((w[j] == w[j - 1]) as u32) << j;
        }
        joins
    }
}

/// The 16 entries `src[j] | (base + (j << 8))` for j < 16, at `dst`: on x86_64 as four
/// SSE2 vectors (the bytes widened by unpacking with zero; a byte and a link share no bit,
/// so the OR is an add), elsewhere one at a time. The links wrap in u32 past 2^24 rows,
/// which only an entry above its run's end can reach, and those are overwritten
///
/// # Safety
/// `src` must be readable for 16 bytes and `dst` writable for 16 entries.
#[inline(always)]
unsafe fn run16(src: *const u8, base: u32, dst: *mut u32) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: SSE2 is in x86_64's baseline; the caller guarantees both ranges
    unsafe {
        use core::arch::x86_64::*;
        let z = _mm_setzero_si128();
        let v = _mm_loadu_si128(src.cast());
        let lo = _mm_unpacklo_epi8(v, z);
        let hi = _mm_unpackhi_epi8(v, z);
        let b = _mm_set1_epi32(base as i32);
        let j0 = _mm_add_epi32(b, _mm_setr_epi32(0, 1 << 8, 2 << 8, 3 << 8));
        let j1 = _mm_add_epi32(b, _mm_setr_epi32(4 << 8, 5 << 8, 6 << 8, 7 << 8));
        let j2 = _mm_add_epi32(b, _mm_setr_epi32(8 << 8, 9 << 8, 10 << 8, 11 << 8));
        let j3 = _mm_add_epi32(b, _mm_setr_epi32(12 << 8, 13 << 8, 14 << 8, 15 << 8));
        let d = dst.cast::<__m128i>();
        _mm_storeu_si128(d, _mm_add_epi32(_mm_unpacklo_epi16(lo, z), j0));
        _mm_storeu_si128(d.add(1), _mm_add_epi32(_mm_unpackhi_epi16(lo, z), j1));
        _mm_storeu_si128(d.add(2), _mm_add_epi32(_mm_unpacklo_epi16(hi, z), j2));
        _mm_storeu_si128(d.add(3), _mm_add_epi32(_mm_unpackhi_epi16(hi, z), j3));
    }
    #[cfg(not(target_arch = "x86_64"))]
    for j in 0..16 {
        // SAFETY: the caller guarantees both ranges
        unsafe {
            *dst.add(j) = (*src.add(j) as u32).wrapping_add(base.wrapping_add((j as u32) << 8))
        };
    }
}

/// The two-pass build: the last column copied into the table, then each row's link added
/// to its byte's next slot. `build` takes it when the counts are not `ll`'s own, so that
/// its table stays what it always was for any counts that keep the writes in bounds
fn build_two_pass(ll: &[u8], counts: &[u32; 256], tt: &mut Vec<u32>) {
    tt.clear();
    tt.extend(ll.iter().map(|&c| c as u32));
    let mut ftab = [0u32; 256];
    let mut cum = 0u32;
    for i in 0..256 {
        ftab[i] = cum;
        cum += counts[i];
    }
    // 16 bytes at a time: 16 equal bytes are 16 consecutive rows of their byte's bucket
    // (rows i..i + 16 take its next 16 slots in order), so their links go in with one
    // 16-wide store and one counter step, libsais's RUN_LANES. After the BWT, most of
    // Wikidata's last column is in such runs; other windows take the one-byte steps.
    let mut i = 0;
    while i + 16 <= ll.len() {
        let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
        if a == b && a == a.rotate_left(8) {
            let c = ll[i] as usize;
            let k = ftab[c] as usize;
            let base = (i as u32) << 8;
            for (j, e) in tt[k..k + 16].iter_mut().enumerate() {
                *e = (*e & 0xff) | (base + ((j as u32) << 8));
            }
            ftab[c] += 16;
        } else {
            link(ll, tt, &mut ftab, i..i + 16);
        }
        i += 16;
    }
    link(ll, tt, &mut ftab, i..ll.len());
}

/// `build`'s one-byte steps, over the rows `rows`
#[inline(always)]
fn link(ll: &[u8], tt: &mut [u32], ftab: &mut [u32; 256], rows: std::ops::Range<usize>) {
    for i in rows {
        let c = ll[i];
        let k = ftab[c as usize] as usize;
        // (k < len, as the counts are ll's own: indexed checked all the same, and the
        // link assigned, not added, so each link is some i < len whatever happened)
        tt[k] = (tt[k] & 0xff) | (i as u32) << 8;
        ftab[c as usize] += 1;
    }
}

#[inline(always)]
pub(crate) fn read_u64(b: &[u8], i: usize) -> u64 {
    u64::from_ne_bytes(b[i..i + 8].try_into().unwrap())
}

/// The plain walk: the list followed from the primary index, the RLE1-coded block out
/// (`orig_ptr` below the block's length: checked by the caller)
pub fn walk(tt: &TVector, orig_ptr: u32, out: &mut Vec<u8>) {
    let t = &tt.0;
    let n = t.len();
    out.clear();
    out.reserve(n);
    let mut p = t[orig_ptr as usize];
    for _ in 0..n {
        // SAFETY: every link is below n (TVector)
        p = unsafe { *t.get_unchecked((p >> 8) as usize) };
        out.push(p as u8);
    }
}

/// The lanes walked at once, and the segments a block is cut into, in the bitmap walk
/// (the prototype's measured best on this machine class: kernels/bz2k)
const LANES: usize = 12;
const SEGMENTS: usize = 256;
/// The same in the flagged walk, which prefetches each lane's next entry as soon as its
/// row is known (#5). A prefetch retires without waiting for its line, so the lanes' misses
/// are no longer bounded by how many steps the reorder buffer holds, and more lanes keep
/// more of them in flight: 32 lanes and 512 segments were the best of 8-32 lanes and
/// 256-1024 segments on Wikidata's blocks, at one walk and at four at once
pub(crate) const FLAGGED_LANES: usize = 32;
pub(crate) const FLAGGED_SEGMENTS: usize = 512;
const PREFETCH: u8 = PF_L1;
/// The walks' prefetch (their `PF`): none, or a hint into L1 (`prefetcht0`, `prfm
/// pldl1keep`), into L2 (`prefetcht1`, `prfm pldl2keep`), or non-temporal (`prefetchnta`,
/// `prfm pldl1strm`): the same values read whichever, so they can be measured side by side
pub const PF_OFF: u8 = 0;
pub const PF_L1: u8 = 1;
pub const PF_L2: u8 = 2;
pub const PF_NTA: u8 = 3;
/// Blocks shorter than this are walked plainly (the segments' bookkeeping would cost more
/// than the latency they hide; such blocks are only a stream's last)
pub const MIN_SEGMENTED: usize = 1 << 16;

/// The segment walk's buffers, kept from block to block
#[derive(Default)]
pub struct SegScratch {
    pub(crate) starts: Vec<u32>,
    bitmap: Vec<u64>,
    pub(crate) bufs: Vec<Vec<u8>>,
    pub(crate) next: Vec<usize>,
    /// the room walk's last block: its length and the segment its output starts in
    pub(crate) n: usize,
    pub(crate) first: usize,
}

impl SegScratch {
    /// The room walk's segments (`walk_segments_room_parts`'s), joined in the output's
    /// order into `out`, which they replace: the walk's own last step, timed apart
    pub fn assemble(&self, out: &mut Vec<u8>) {
        let n = self.n;
        out.clear();
        out.reserve(n);
        let mut seg = self.first;
        while out.len() < n {
            let b = &self.bufs[seg];
            out.extend_from_slice(&b[..b.len().min(n - out.len())]);
            seg = self.next[seg];
        }
    }
}

/// The forward walk's bytes from the backward walk's: `out[n - m..]` holds the first m bytes
/// of the LF walk from the primary index, last to first, where m is the length of the
/// primary index's cycle, or n if that is longer. The forward walk (`walk`) gives n bytes of
/// Ψ from the same row: that cycle's bytes, first to last, repeated and cut at n. Those are
/// `out[n - m..]`'s, so they are moved to the front and repeated. (m = n for a block of
/// distinct rotations; a periodic text's cycle is its period. This makes the backward walk
/// exactly `walk` on any table, so a corrupt block's error is the same too.)
fn repeat_cycle<T: Copy>(out: &mut [T], m: usize) {
    let n = out.len();
    if m == 0 || m >= n {
        return;
    }
    out.copy_within(n - m.., 0);
    let mut have = m;
    while have < n {
        let take = have.min(n - have);
        out.copy_within(..take, have);
        have += take;
    }
}

impl SegScratch {
    /// The room walk's segments of an LF table walked backward (`walk_back_room`): joined
    /// in the order `assemble` joins them, which is the block's bytes last to first, and
    /// written into `out` (which they replace) from its end, each segment reversed, so that
    /// `out` holds the block first to last. The join stops where it comes back to its first
    /// segment: the primary index's cycle is then complete (`repeat_cycle`).
    pub(crate) fn assemble_reversed(&self, out: &mut Vec<u8>) {
        let n = self.n;
        out.clear();
        out.reserve(n);
        let dst = &mut out.spare_capacity_mut()[..n];
        let mut at = n;
        let mut seg = self.first;
        while at > 0 {
            let b = &self.bufs[seg];
            let len = b.len().min(at);
            for (d, &x) in dst[at - len..at].iter_mut().rev().zip(&b[..len]) {
                d.write(x);
            }
            at -= len;
            seg = self.next[seg];
            if seg == self.first {
                break;
            }
        }
        // (the first segment holds its start's byte at least, so at < n here)
        assert!(at < n || n == 0);
        repeat_cycle(dst, n - at);
        // SAFETY: the join wrote dst[at..n], and `repeat_cycle` copies those n - at bytes to
        // the front and repeats them to n (at > 0 only when the cycle closed, at m = n - at
        // < n), so every byte of dst[..n] is written
        unsafe { out.set_len(n) };
    }
}

/// The plain walk of an LF table (`runs::fill`'s): from `orig_ptr`, i ← LF(i), the block's
/// bytes last to first, written into `out` from its end; exactly `walk`'s output on the
/// block's T-vector (`repeat_cycle`)
pub fn walk_back(tt: &TVector, orig_ptr: u32, out: &mut Vec<u8>) {
    let t = &tt.0;
    let n = t.len();
    out.clear();
    out.resize(n, 0);
    let mut i = orig_ptr as usize;
    let mut m = n;
    for k in 0..n {
        let e = t[i];
        out[n - 1 - k] = e as u8;
        i = (e >> 8) as usize;
        if i == orig_ptr as usize {
            m = k + 1;
            break;
        }
    }
    repeat_cycle(out, m);
}

/// `walk_segments_room` on an LF table (`runs::fill`'s), walked backward from `orig_ptr`
/// (below its length: the caller checks): the room walk's own code with its first row the
/// primary index, its segments joined in reverse. Exactly `walk`'s output on the block's
/// T-vector; the plain backward walk for a block too short to segment
pub fn walk_back_room(tt: &mut TVector, orig_ptr: u32, out: &mut Vec<u8>, s: &mut SegScratch) {
    if tt.len() < MIN_SEGMENTED {
        return walk_back(tt, orig_ptr, out);
    }
    room_segments::<FLAGGED_LANES, PREFETCH, true>(tt, orig_ptr, s, FLAGGED_SEGMENTS);
    s.assemble_reversed(out);
}

/// `walk_back_room` in its two parts, as `walk_segments_room_parts`: the room walk of the LF
/// table, which leaves each segment's bytes in `s` and returns true, for
/// `SegScratch::assemble_reversed` to join into the output; or, for a block too short to
/// segment, `walk_back`'s output written to `out`, returning false
pub(crate) fn walk_back_room_parts(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
) -> bool {
    if tt.len() < MIN_SEGMENTED {
        walk_back(tt, orig_ptr, out);
        return false;
    }
    room_segments::<FLAGGED_LANES, PREFETCH, true>(tt, orig_ptr, s, FLAGGED_SEGMENTS);
    true
}

/// `walk_back_room` with `L` lanes and `segments` segments, on a block of any length (the
/// tests walk small ones)
#[cfg(test)]
pub(crate) fn walk_back_room_with<const L: usize>(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
    segments: usize,
) {
    out.clear();
    if tt.0.is_empty() {
        return;
    }
    room_segments::<L, PREFETCH, true>(tt, orig_ptr, s, segments);
    s.assemble_reversed(out);
}

/// The walk, segmented where the block is long enough: exactly `walk`'s output
pub fn walk_fast(tt: &TVector, orig_ptr: u32, out: &mut Vec<u8>, s: &mut SegScratch) {
    if tt.len() < MIN_SEGMENTED {
        walk(tt, orig_ptr, out)
    } else {
        walk_segments(tt, orig_ptr, out, s)
    }
}

/// The walk split into segments walked LANES at a time, then joined.
///
/// SEGMENTS pseudo-random rows are taken as segment starts, the walk's own first row r0
/// among them (a random row is a random position in the text, so the segments are about
/// equal; evenly spaced rows clustered in the text). A segment is the walk from its start
/// until the next row is a start; the lanes walk segments in lockstep, each taking the
/// next unwalked one when its own ends, and each segment records the start it ran into.
/// The output is then the segments joined from r0's, each followed by the one it ran into,
/// until the block's length is reached: by construction the first n steps of the walk
/// from r0, which is what `walk` writes. That holds whatever the list's shape: where it is
/// several cycles (a block that is a string repeated), the starts off r0's cycle are
/// walked and never joined, and r0's cycle is joined round again as `walk` goes round it.
/// Every segment ends, since its own start is on its cycle; so the lanes take at most n
/// steps in all.
pub fn walk_segments(tt: &TVector, orig_ptr: u32, out: &mut Vec<u8>, s: &mut SegScratch) {
    let t = &tt.0;
    let n = t.len();
    out.clear();
    let r0 = t[orig_ptr as usize] >> 8;
    random_rows(n, SEGMENTS - 1, &mut s.starts);
    s.starts.push(r0);
    s.starts.sort_unstable();
    s.starts.dedup();
    let starts = &s.starts;
    let ns = starts.len();
    s.bitmap.clear();
    s.bitmap.resize(n.div_ceil(64), 0);
    for &r in starts {
        s.bitmap[(r >> 6) as usize] |= 1 << (r & 63);
    }
    let cap = 2 * n / ns + 64;
    s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
    for b in &mut s.bufs[..ns] {
        b.clear();
        b.reserve(cap);
    }
    s.next.clear();
    s.next.resize(ns, usize::MAX);
    let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let mut lane_seg = [usize::MAX; LANES];
    let mut lane_row = [0u32; LANES];
    let mut queued = 0;
    for k in 0..LANES {
        if queued < ns {
            lane_seg[k] = queued;
            lane_row[k] = starts[queued];
            queued += 1;
        }
    }
    let mut active = lane_seg.iter().filter(|&&x| x != usize::MAX).count();
    let bm = &s.bitmap;
    let bufs = &mut s.bufs;
    while active > 0 {
        for k in 0..LANES {
            let seg = lane_seg[k];
            if seg == usize::MAX {
                continue;
            }
            // SAFETY: every row is below n (TVector; the starts are rows below n)
            let q = unsafe { *t.get_unchecked(lane_row[k] as usize) };
            let row = q >> 8;
            bufs[seg].push(q as u8);
            lane_row[k] = row;
            // SAFETY: row < n, and the bitmap has n.div_ceil(64) words
            if unsafe { *bm.get_unchecked((row >> 6) as usize) } >> (row & 63) & 1 != 0 {
                s.next[seg] = seg_of(row);
                if queued < ns {
                    lane_seg[k] = queued;
                    lane_row[k] = starts[queued];
                    queued += 1;
                } else {
                    lane_seg[k] = usize::MAX;
                    active -= 1;
                }
            }
        }
    }
    out.reserve(n);
    let mut seg = seg_of(r0);
    while out.len() < n {
        let b = &bufs[seg];
        out.extend_from_slice(&b[..b.len().min(n - out.len())]);
        seg = s.next[seg];
    }
}

/// The flag marking a segment start in its own T-vector entry (links are below 2^20, so
/// bit 31 is free), and the mask that takes it off a link
pub(crate) const START: u32 = 1 << 31;
pub(crate) const LINK: u32 = !START;

/// One lane's state in `walk_segments_flagged`: the row it reads next, and where its
/// segment's bytes go (`p` up to `end`, the segment's buffer's spare capacity)
#[derive(Clone, Copy)]
struct Lane {
    row: u32,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

/// `walk_segments`, its bookkeeping out of the hot loop (#4): the starts flagged in their
/// own entries (no bitmap), a segment's first step taken when a lane takes it (so a
/// flagged entry read in the loop is always the next segment's start), the bytes written
/// through raw per-lane pointers (no `Vec` header per step), and a loop that does not test
/// for idle lanes until the last segments are taken. The flags are taken off again before
/// it returns, so `tt` is as `build` left it. Exactly `walk`'s output.
pub fn walk_segments_flagged(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
) {
    if tt.len() < MIN_SEGMENTED {
        return walk(tt, orig_ptr, out);
    }
    walk_segments_flagged_with_prefetch::<FLAGGED_LANES, PREFETCH>(
        tt,
        orig_ptr,
        out,
        s,
        FLAGGED_SEGMENTS,
    )
}

/// `walk_segments_flagged` with `L` lanes and `segments` segments, each lane's next entry
/// prefetched as soon as its row is known, at the level `PF` names (`PF_OFF` for none,
/// `PF_L1`, `PF_L2`, `PF_NTA`): each configuration exactly `walk`'s output, so they can be
/// measured side by side. It segments a block of any length (the tests walk small ones;
/// `walk_segments_flagged` leaves those below MIN_SEGMENTED to `walk`).
pub fn walk_segments_flagged_with_prefetch<const L: usize, const PF: u8>(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
    segments: usize,
) {
    let n = tt.0.len();
    out.clear();
    if n == 0 {
        return;
    }
    let r0 = tt.0[orig_ptr as usize] >> 8;
    random_rows(n, segments - 1, &mut s.starts);
    s.starts.push(r0);
    s.starts.sort_unstable();
    s.starts.dedup();
    let ns = s.starts.len();
    for &r in &s.starts {
        tt.0[r as usize] |= START;
    }
    let cap = 2 * n / ns + 64;
    s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
    for b in &mut s.bufs[..ns] {
        b.clear();
        b.reserve(cap);
    }
    s.next.clear();
    s.next.resize(ns, usize::MAX);
    let starts = &s.starts;
    let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let t = tt.0.as_ptr();
    let bufs = &mut s.bufs;
    let next = &mut s.next;

    // A lane takes segment `seg`: its first step (its start's own, flagged, entry) taken
    // here, so the loop's flag test only ever sees the next segment's start
    let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
        let b = &mut bufs[seg];
        let base = b.as_mut_ptr();
        // SAFETY: the starts are rows below n; the buffer has `cap` > 0 spare bytes
        let q = unsafe { *t.add(starts[seg] as usize) } & LINK;
        debug_assert!(b.capacity() > 1);
        unsafe { *base = q as u8 };
        Lane {
            row: q >> 8,
            seg,
            p: unsafe { base.add(1) },
            end: unsafe { base.add(b.capacity()) },
        }
    };
    // A lane's segment ends at `row`, a start: its length recorded and its successor
    let finish = |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
        let b = &mut bufs[l.seg];
        // SAFETY: p is within b's allocation, every byte below it written
        unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
        next[l.seg] = seg_of(l.row);
    };
    // A lane's buffer is full: it grows (its written bytes kept)
    let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
        let b = &mut bufs[l.seg];
        let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
        unsafe { b.set_len(len) };
        b.reserve(b.capacity());
        l.p = unsafe { b.as_mut_ptr().add(len) };
        l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
    };

    let mut queued = 0;
    let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
        row: 0,
        seg: usize::MAX,
        p: std::ptr::null_mut(),
        end: std::ptr::null_mut(),
    });
    let mut active = 0;
    while active < L && queued < ns {
        lanes[active] = take(queued, bufs);
        if PF != PF_OFF {
            prefetch::<PF>(t, lanes[active].row);
        }
        queued += 1;
        active += 1;
    }
    // Every lane busy while segments remain to be taken
    if active == L {
        'full: loop {
            for l in lanes.iter_mut() {
                // SAFETY: rows are below n (TVector), with the flag masked off below
                let q = unsafe { *t.add(l.row as usize) };
                if q & START != 0 {
                    finish(l, bufs, next);
                    if queued == ns {
                        break 'full;
                    }
                    *l = take(queued, bufs);
                    if PF != PF_OFF {
                        prefetch::<PF>(t, l.row);
                    }
                    queued += 1;
                    continue;
                }
                l.row = q >> 8;
                if PF != PF_OFF {
                    prefetch::<PF>(t, l.row);
                }
                // SAFETY: p < end, within the buffer's allocation
                unsafe {
                    *l.p = q as u8;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    grow(l, bufs);
                }
            }
        }
    }
    // The tail: the lanes still walking, each dropped as its segment ends. (A lane that
    // broke out of the full loop above has finished: its segment was recorded, and it is
    // known by p having been reset below.)
    let mut live: Vec<Lane> = lanes[..active]
        .iter()
        .copied()
        .filter(|l| next_unset(l, next))
        .collect();
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            let q = unsafe { *t.add(l.row as usize) };
            if q & START != 0 {
                finish(l, bufs, next);
                live.swap_remove(i);
                continue;
            }
            l.row = q >> 8;
            if PF != PF_OFF {
                prefetch::<PF>(t, l.row);
            }
            unsafe {
                *l.p = q as u8;
                l.p = l.p.add(1);
            }
            if l.p == l.end {
                grow(l, bufs);
            }
            i += 1;
        }
    }
    for &r in starts {
        tt.0[r as usize] &= LINK;
    }
    out.reserve(n);
    let mut seg = seg_of(r0);
    while out.len() < n {
        let b = &bufs[seg];
        out.extend_from_slice(&b[..b.len().min(n - out.len())]);
        seg = next[seg];
    }
}

// Counts of the walks' rarer transitions, in test builds only (per thread, so each test
// reads its own), so a test can show it reached them; nothing in other builds
#[cfg(test)]
thread_local! {
    static GROWS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static PASS_ENDS_FULL: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static MID_PASS_TAKES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FULL_PHASE_ENDS_MID_PASS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static TAIL_ENTERED_FULL: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    // the room walk's deferred handovers: several lanes marked at one offset; the queue
    // running out with a marked lane after this one still waiting; and running out after
    // an earlier marked lane at the same offset took a segment
    static MARKED_SEVERAL: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static ENDS_WITH_LANES_WAITING: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static ENDS_AFTER_A_TAKE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    // ... and, when it runs out, an unmarked later lane (stepped already) leaving with its
    // buffer exactly full, or with its next entry the end of its segment
    static LATER_STEPPED_FULL: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static LATER_STEPPED_ON_START: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    // the staged busy phase's chunk edges: a pass of more than one chunk; a take at a
    // chunk's last offset (its row staged from st = w, one past the chunk) or in a later
    // chunk; a segment taken and finished in one chunk; a lane marked at a later chunk's
    // first offset; and the queue running out at a later chunk's first offset, at a
    // chunk's last offset short of the pass's end, or at the pass's last offset
    static ROOM_OVER_TILE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static TAKE_AT_CHUNK_END: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static TAKE_IN_LATER_CHUNK: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DONE_IN_TAKING_CHUNK: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static MARK_AT_LATER_CHUNK_START: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static EXHAUSTED_AT_CHUNK_START: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static EXHAUSTED_AT_CHUNK_END: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static EXHAUSTED_AT_PASS_END: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
macro_rules! transition {
    ($c:ident) => {
        $c.with(|c| c.set(c.get() + 1))
    };
}
#[cfg(not(test))]
macro_rules! transition {
    ($c:ident) => {};
}

/// `walk_segments_flagged` in passes, as libsais's chain decode runs (`decode_chains`):
/// each pass is `room` rounds of every lane, `room` the fewest bytes any lane's buffer has
/// spare, so a lane writes its byte at its base plus the pass's offset, with no per-step
/// pointer update or end test. A lane whose segment ends mid-pass takes the next one and
/// is rebased so that base + offset is its new buffer. Exactly `walk`'s output.
pub fn walk_segments_room(tt: &mut TVector, orig_ptr: u32, out: &mut Vec<u8>, s: &mut SegScratch) {
    if walk_segments_room_parts(tt, orig_ptr, out, s) {
        s.assemble(out);
    }
}

/// `walk_segments_room` in its two parts, to time them apart (examples/phases.rs): the
/// walk, which leaves each segment's bytes in `s` and returns true, for
/// `SegScratch::assemble` to join into the output; or, for a block too short to segment,
/// `walk`'s output written to `out`, returning false
pub fn walk_segments_room_parts(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
) -> bool {
    if tt.len() < MIN_SEGMENTED {
        walk(tt, orig_ptr, out);
        return false;
    }
    room_segments::<FLAGGED_LANES, PREFETCH, false>(tt, orig_ptr, s, FLAGGED_SEGMENTS);
    true
}

/// `walk_segments_room` with `L` lanes and `segments` segments, each lane's next entry
/// prefetched at the level `PF` names (`PF_OFF` for none, `PF_L1`, `PF_L2`, `PF_NTA`), on a
/// block of any length (the tests walk small ones; `walk_segments_room` leaves those below
/// MIN_SEGMENTED to `walk`).
///
/// What its unchecked writes rely on (the walks' contract, which the tests check
/// against a safe reference of the segments, `tests::segment_oracle`):
/// - **Rows:** every row read is below n (links come from `build`; starts are rows).
/// - **Ownership:** a segment is taken once and walked by one lane, which writes only
///   its buffer; a lane stops at the next start, so no two lanes walk the same rows.
/// - **Spare:** at a pass's start every lane has at least `room` bytes spare, and a lane
///   taken mid-pass at offset o has at least `cap` - 1 after its first byte, which covers
///   the pass's `room` - o - 1 to come; so every write at base + offset is in bounds.
/// - **Handoffs:** a lane comes into a pass, and into the tail, with p < end (the tail
///   writes before it tests); a pass may end with p == end, so both entries grow first.
/// - **Initialised:** a lane's writes are consecutive from its buffer's start, so every
///   byte below p is written when `set_len` exposes it.
/// - **Lifetimes:** a buffer grows only at a pass's start or in the tail, and its lane's
///   p and end (and base, from p) are taken from the grown buffer; `bufs` itself is not
///   resized during the walk, so no other buffer moves.
pub fn walk_segments_room_with_prefetch<const L: usize, const PF: u8>(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
    segments: usize,
) {
    out.clear();
    if tt.0.is_empty() {
        return;
    }
    room_segments::<L, PF, false>(tt, orig_ptr, s, segments);
    s.assemble(out);
}

/// The room walk itself, on a block of one row or more: each segment's bytes into its
/// buffer in `s`, its successor in `s.next`, and the block's length and first segment
/// for `SegScratch::assemble`. Never inlined, so the decoder and the stage table's split
/// timing (`walk_segments_room_parts`) run the same machine code for the walk
#[inline(never)]
fn room_segments<const L: usize, const PF: u8, const BACK: bool>(
    tt: &mut TVector,
    orig_ptr: u32,
    s: &mut SegScratch,
    segments: usize,
) {
    // a pass marks its lanes in a u64
    const { assert!(L <= 64) };
    let n = tt.0.len();
    // A start is marked in bit 31 of its entry, which is bit 23 of the entry's link: a link
    // of 2^23 or more would read as a start (bzip2's blocks are at most 900,000 rows)
    assert!(
        n <= 1 << 23,
        "the room walk takes at most 2^23 rows, not {n}"
    );
    // the walk's first row: Ψ's from the primary index; or, on an LF table walked backward
    // (`BACK`), the primary index itself, whose byte is the block's last
    let r0 = if BACK {
        assert!((orig_ptr as usize) < n, "the primary index past the block");
        orig_ptr
    } else {
        tt.0[orig_ptr as usize] >> 8
    };
    random_rows(n, segments - 1, &mut s.starts);
    s.starts.push(r0);
    s.starts.sort_unstable();
    s.starts.dedup();
    let ns = s.starts.len();
    for &r in &s.starts {
        tt.0[r as usize] |= START;
    }
    let cap = 2 * n / ns + 64;
    s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
    for b in &mut s.bufs[..ns] {
        b.clear();
        b.reserve(cap);
    }
    s.next.clear();
    s.next.resize(ns, usize::MAX);
    let starts = &s.starts;
    let seg_of = move |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let t = tt.0.as_ptr();
    let bufs = &mut s.bufs;
    let next = &mut s.next;

    let take = move |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
        let b = &mut bufs[seg];
        let base = b.as_mut_ptr();
        // SAFETY: the starts are rows below n; the buffer has `cap` > 0 spare bytes
        let q = unsafe { *t.add(starts[seg] as usize) } & LINK;
        unsafe { *base = q as u8 };
        Lane {
            row: q >> 8,
            seg,
            p: unsafe { base.add(1) },
            end: unsafe { base.add(b.capacity()) },
        }
    };
    let finish = move |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
        let b = &mut bufs[l.seg];
        // SAFETY: p is within b's allocation, every byte below it written
        unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
        next[l.seg] = seg_of(l.row);
    };
    let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
        transition!(GROWS);
        let b = &mut bufs[l.seg];
        let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
        unsafe { b.set_len(len) };
        b.reserve(b.capacity());
        l.p = unsafe { b.as_mut_ptr().add(len) };
        l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
    };

    let mut queued = 0;
    let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
        row: 0,
        seg: usize::MAX,
        p: std::ptr::null_mut(),
        end: std::ptr::null_mut(),
    });
    let mut active = 0;
    while active < L && queued < ns {
        lanes[active] = take(queued, bufs);
        if PF != PF_OFF {
            prefetch::<PF>(t, lanes[active].row);
        }
        queued += 1;
        active += 1;
    }
    if active == L {
        // While every lane is busy the rows are held together in `rows`, and the bytes are
        // staged: offset j of a chunk of at most TILE offsets puts lane k's byte at
        // tile[k][j], one base for every lane, instead of at its buffer's base + off. Lane
        // k's current segment has its bytes from st[k] in its tile row; they are copied to
        // its buffer at its p when the chunk ends or the segment does, so every byte lands
        // where the unstaged walk wrote it, and the pass's room bounds the copies as it
        // bounded the writes.
        let mut rows = [0u32; L];
        for (k, l) in lanes.iter().enumerate() {
            rows[k] = l.row;
        }
        let mut tile = [[0u8; TILE]; L];
        // copy lane k's tile row from st[k] to end into its buffer at p
        let flush = |l: &mut Lane, row: &[u8; TILE], from: usize, to: usize| {
            let len = to - from;
            // SAFETY: from..to is inside the row; the lane's buffer has room for the
            // chunk's bytes (the pass's room, as for the unstaged writes), so p + len is
            // at most its end
            debug_assert!(from <= to && to <= TILE);
            debug_assert!(len <= unsafe { l.end.offset_from(l.p) } as usize);
            unsafe {
                std::ptr::copy_nonoverlapping(row.as_ptr().add(from), l.p, len);
                l.p = l.p.add(len);
            }
        };
        'full: loop {
            // The pass's room: what every lane has spare, and no more than a buffer taken
            // mid-pass has after its first byte (`cap` - 1)
            let mut room = cap - 1;
            for l in lanes.iter_mut() {
                if l.p == l.end {
                    grow(l, bufs);
                }
                room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
            }
            debug_assert!(room > 0);
            if lanes
                .iter()
                .any(|l| unsafe { l.end.offset_from(l.p) } as usize == room)
            {
                transition!(PASS_ENDS_FULL);
            }
            let mut done = 0;
            if room > TILE {
                transition!(ROOM_OVER_TILE);
            }
            while done < room {
                let w = (room - done).min(TILE);
                let mut st = [0usize; L];
                for j in 0..w {
                    // The offset: every lane whose entry doesn't start a segment steps; a
                    // lane whose entry does is only marked, and handed over below, in lane
                    // order, as master's room walk does
                    let rs = &mut rows;
                    let mut marked: u64 = 0;
                    let tp = tile.as_mut_ptr().cast::<u8>();
                    macro_rules! sstep {
                        ($k:expr) => {{
                            let k = $k;
                            // SAFETY: k < L; rows are below n (TVector); k * TILE + j is
                            // inside the tile (j < w <= TILE)
                            unsafe {
                                let r = rs.get_unchecked_mut(k);
                                let q = *t.add(*r as usize);
                                if q & START != 0 {
                                    marked |= 1 << k;
                                } else {
                                    *tp.add(k * TILE + j) = q as u8;
                                    *r = q >> 8;
                                    if PF != PF_OFF {
                                        prefetch::<PF>(t, q >> 8);
                                    }
                                }
                            }
                        }};
                    }
                    let mut c = 0;
                    while c + 8 <= L {
                        sstep!(c);
                        sstep!(c + 1);
                        sstep!(c + 2);
                        sstep!(c + 3);
                        sstep!(c + 4);
                        sstep!(c + 5);
                        sstep!(c + 6);
                        sstep!(c + 7);
                        c += 8;
                    }
                    while c < L {
                        sstep!(c);
                        c += 1;
                    }
                    if marked != 0 && j == 0 && done > 0 {
                        transition!(MARK_AT_LATER_CHUNK_START);
                    }
                    #[cfg(test)]
                    if marked.count_ones() > 1 {
                        transition!(MARKED_SEVERAL);
                    }
                    #[cfg(test)]
                    let mut took = false;
                    while marked != 0 {
                        let k = marked.trailing_zeros() as usize;
                        marked &= marked - 1;
                        let l = &mut lanes[k];
                        l.row = rs[k];
                        if st[k] > 0 {
                            transition!(DONE_IN_TAKING_CHUNK);
                        }
                        // its segment's bytes end before offset j
                        flush(l, &tile[k], st[k], j);
                        finish(l, bufs, next);
                        if queued == ns {
                            // Every lane but the marked ones after this one has its byte at
                            // j: an unmarked lane wrote it, and an earlier marked lane's new
                            // segment starts after it. A marked lane after this one still
                            // sits on its segment's end, without a byte at j, for the tail.
                            for (i, li) in lanes.iter_mut().enumerate() {
                                li.row = rs[i];
                                if i != k {
                                    let waiting = marked >> i & 1 != 0;
                                    flush(li, &tile[i], st[i], j + usize::from(!waiting));
                                    #[cfg(test)]
                                    if i > k && !waiting {
                                        if li.p == li.end {
                                            transition!(LATER_STEPPED_FULL);
                                        }
                                        // SAFETY: the lane's row is below n
                                        if unsafe { *t.add(li.row as usize) } & START != 0 {
                                            transition!(LATER_STEPPED_ON_START);
                                        }
                                    }
                                }
                            }
                            transition!(FULL_PHASE_ENDS_MID_PASS);
                            #[cfg(test)]
                            {
                                if j == 0 && done > 0 {
                                    transition!(EXHAUSTED_AT_CHUNK_START);
                                }
                                if j + 1 == w && done + w < room {
                                    transition!(EXHAUSTED_AT_CHUNK_END);
                                }
                                if done + j + 1 == room {
                                    transition!(EXHAUSTED_AT_PASS_END);
                                }
                                if marked != 0 {
                                    transition!(ENDS_WITH_LANES_WAITING);
                                }
                                if took {
                                    transition!(ENDS_AFTER_A_TAKE);
                                }
                            }
                            break 'full;
                        }
                        transition!(MID_PASS_TAKES);
                        *l = take(queued, bufs);
                        rs[k] = l.row;
                        if PF != PF_OFF {
                            prefetch::<PF>(t, l.row);
                        }
                        queued += 1;
                        // its first byte is in its buffer; its next, at offset j + 1, is
                        // staged
                        st[k] = j + 1;
                        #[cfg(test)]
                        {
                            took = true;
                            if j + 1 == w {
                                transition!(TAKE_AT_CHUNK_END);
                            }
                            if done > 0 {
                                transition!(TAKE_IN_LATER_CHUNK);
                            }
                        }
                    }
                }
                for (k, l) in lanes.iter_mut().enumerate() {
                    flush(l, &tile[k], st[k], w);
                }
                done += w;
            }
        }
    }
    let mut live: Vec<Lane> = lanes[..active]
        .iter()
        .copied()
        .filter(|l| next_unset(l, next))
        .collect();
    // A pass can end with a lane's buffer exactly full (its room was all its spare): the
    // tail writes before it tests, so each lane comes into it with a byte to spare
    for l in live.iter_mut() {
        if l.p == l.end {
            transition!(TAIL_ENTERED_FULL);
            grow(l, bufs);
        }
        debug_assert!(l.p < l.end);
    }
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            let q = unsafe { *t.add(l.row as usize) };
            if q & START != 0 {
                finish(l, bufs, next);
                live.swap_remove(i);
                continue;
            }
            l.row = q >> 8;
            if PF != PF_OFF {
                prefetch::<PF>(t, l.row);
            }
            unsafe {
                *l.p = q as u8;
                l.p = l.p.add(1);
            }
            if l.p == l.end {
                grow(l, bufs);
            }
            i += 1;
        }
    }
    for &r in starts {
        tt.0[r as usize] &= LINK;
    }
    s.first = seg_of(r0);
    s.n = n;
}

/// The busy phase's chunk: at most this many offsets' bytes are staged per lane (an 8 KiB
/// tile at 32 lanes, in L1) before they are copied to the lanes' buffers
const TILE: usize = 256;

/// `walk_segments_flagged_with_prefetch` with 0.1.5's switch: `PF` prefetches each lane's
/// next entry into L1 (`PF_L1`, 0.1.5's prefetch) or not at all (`PF_OFF`). Kept so that
/// 0.1.5's callers build unchanged.
#[inline]
pub fn walk_segments_flagged_with<const L: usize, const PF: bool>(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
    segments: usize,
) {
    if PF {
        walk_segments_flagged_with_prefetch::<L, PF_L1>(tt, orig_ptr, out, s, segments)
    } else {
        walk_segments_flagged_with_prefetch::<L, PF_OFF>(tt, orig_ptr, out, s, segments)
    }
}

/// `walk_segments_room_with_prefetch` with 0.1.5's switch: `PF` prefetches each lane's next
/// entry into L1 (`PF_L1`, 0.1.5's prefetch) or not at all (`PF_OFF`). Kept so that 0.1.5's
/// callers build unchanged.
#[inline]
pub fn walk_segments_room_with<const L: usize, const PF: bool>(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
    segments: usize,
) {
    if PF {
        walk_segments_room_with_prefetch::<L, PF_L1>(tt, orig_ptr, out, s, segments)
    } else {
        walk_segments_room_with_prefetch::<L, PF_OFF>(tt, orig_ptr, out, s, segments)
    }
}

/// A hint that the T-vector entry at `row` (below its length) is read soon: a software
/// prefetch at the level `H` names (`PF_L1`, `PF_L2`, `PF_NTA`) where the target has one,
/// else nothing. A hint only: it changes no value read.
#[inline(always)]
pub(crate) fn prefetch<const H: u8>(t: *const u32, row: u32) {
    let p = t.wrapping_add(row as usize);
    #[cfg(target_arch = "x86_64")]
    // SAFETY: SSE is in x86_64's baseline; a prefetch never faults
    unsafe {
        use core::arch::x86_64::{_mm_prefetch, _MM_HINT_NTA, _MM_HINT_T0, _MM_HINT_T1};
        match H {
            PF_L2 => _mm_prefetch::<_MM_HINT_T1>(p as *const i8),
            PF_NTA => _mm_prefetch::<_MM_HINT_NTA>(p as *const i8),
            _ => _mm_prefetch::<_MM_HINT_T0>(p as *const i8),
        }
    };
    #[cfg(target_arch = "aarch64")]
    // SAFETY: a prefetch never faults and writes nothing
    unsafe {
        match H {
            PF_L2 => {
                core::arch::asm!("prfm pldl2keep, [{p}]", p = in(reg) p, options(nostack, preserves_flags, readonly))
            }
            PF_NTA => {
                core::arch::asm!("prfm pldl1strm, [{p}]", p = in(reg) p, options(nostack, preserves_flags, readonly))
            }
            _ => {
                core::arch::asm!("prfm pldl1keep, [{p}]", p = in(reg) p, options(nostack, preserves_flags, readonly))
            }
        }
    };
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let _ = p;
}

/// A lane whose segment has not been recorded as finished
fn next_unset(l: &Lane, next: &[usize]) -> bool {
    l.seg != usize::MAX && next[l.seg] == usize::MAX
}

/// `m` pseudo-random rows below `n` (xorshift, seeded by n: the same rows every time)
pub(crate) fn random_rows(n: usize, m: usize, rows: &mut Vec<u32>) {
    rows.clear();
    let mut x = 0x2545_f491_4f6c_dd1du64 ^ n as u64;
    rows.extend((0..m).map(|_| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        (x % n as u64) as u32
    }));
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// The segments' reference, safe and slow: from each start, the walks' own steps (its
    /// first byte from its own entry, then a row's byte at a time until the next start),
    /// giving each segment's bytes and its successor. Every segment, on every cycle of the
    /// list, not only those the output joins: a walk writes them all.
    pub(crate) fn segment_oracle(tt: &TVector, starts: &[u32]) -> Vec<(Vec<u8>, usize)> {
        let t = &tt.0;
        let mut index = vec![usize::MAX; t.len()];
        for (i, &r) in starts.iter().enumerate() {
            index[r as usize] = i;
        }
        starts
            .iter()
            .map(|&r| {
                let mut bytes = vec![t[r as usize] as u8];
                let mut row = (t[r as usize] >> 8) as usize;
                while index[row] == usize::MAX {
                    bytes.push(t[row] as u8);
                    row = (t[row] >> 8) as usize;
                }
                (bytes, index[row])
            })
            .collect()
    }

    /// A walk's segments (each one's bytes and successor, in `s`) are the reference's
    fn check_segments(tt: &TVector, s: &SegScratch, what: &str) {
        let want = segment_oracle(tt, &s.starts);
        for (i, (bytes, next)) in want.iter().enumerate() {
            assert!(s.bufs[i] == *bytes, "{what}: segment {i}'s bytes");
            assert_eq!(s.next[i], *next, "{what}: segment {i}'s successor");
        }
    }

    fn count(c: &'static std::thread::LocalKey<std::cell::Cell<usize>>) -> usize {
        c.with(|c| c.get())
    }

    fn check(ll: &[u8]) {
        let mut counts = [0u32; 256];
        for &c in ll {
            counts[c as usize] += 1;
        }
        let mut tt = TVector::new();
        build(ll, &counts, &mut tt);
        let (mut plain, mut flagged, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
        for orig_ptr in [0, ll.len() as u32 / 3, ll.len() as u32 - 1] {
            walk(&tt, orig_ptr, &mut plain);
            walk_segments_flagged(&mut tt, orig_ptr, &mut flagged, &mut seg);
            assert!(plain == flagged, "orig_ptr {orig_ptr}");
            if ll.len() >= MIN_SEGMENTED {
                check_segments(&tt, &seg, "flagged walk");
            }
            walk_segments_room(&mut tt, orig_ptr, &mut flagged, &mut seg);
            assert!(plain == flagged, "room walk, orig_ptr {orig_ptr}");
            if ll.len() >= MIN_SEGMENTED {
                check_segments(&tt, &seg, "room walk");
            }
            assert!(tt.0.iter().all(|&e| e & START == 0));
            for segments in [2, 40, 512, 4096] {
                walk_segments_room_with_prefetch::<32, PF_L1>(
                    &mut tt,
                    orig_ptr,
                    &mut flagged,
                    &mut seg,
                    segments,
                );
                assert!(plain == flagged, "room walk, {segments} segments");
                walk_segments_room_with_prefetch::<5, PF_OFF>(
                    &mut tt,
                    orig_ptr,
                    &mut flagged,
                    &mut seg,
                    segments,
                );
                assert!(plain == flagged, "room walk, 5 lanes, {segments} segments");
            }
            // the flags are taken off again: the T-vector is as build left it
            assert!(tt.0.iter().all(|&e| e & START == 0));
            // other lane and segment counts, with and without the prefetch
            for segments in [1, 256, 1024] {
                walk_segments_flagged_with_prefetch::<12, PF_OFF>(
                    &mut tt,
                    orig_ptr,
                    &mut flagged,
                    &mut seg,
                    segments,
                );
                assert!(plain == flagged, "12 lanes, {segments} segments");
                walk_segments_flagged_with_prefetch::<7, PF_L1>(
                    &mut tt,
                    orig_ptr,
                    &mut flagged,
                    &mut seg,
                    segments,
                );
                assert!(plain == flagged, "7 lanes, prefetch, {segments} segments");
                assert!(tt.0.iter().all(|&e| e & START == 0));
            }
        }
    }

    /// `build` (16-byte run stores) gives the one-pass counting sort's T-vector exactly: runs
    /// of every length around 16 at every alignment, mixed with random bytes, a run to the
    /// end, one byte throughout, and lengths 0 to 40 for the tail
    #[test]
    fn run_stores_equal_one_pass() {
        fn one_pass(ll: &[u8], counts: &[u32; 256]) -> Vec<u32> {
            let mut ftab = [0u32; 256];
            let mut cum = 0;
            for c in 0..256 {
                ftab[c] = cum;
                cum += counts[c];
            }
            let mut tt: Vec<u32> = ll.iter().map(|&c| c as u32).collect();
            for (i, &c) in ll.iter().enumerate() {
                let k = ftab[c as usize] as usize;
                tt[k] |= (i as u32) << 8;
                ftab[c as usize] += 1;
            }
            tt
        }
        let mut x = 0x2545_f491_4f6c_dd1du64;
        let mut rnd = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let mut inputs: Vec<Vec<u8>> = (0..=40).map(|n| vec![7u8; n]).collect();
        for _ in 0..40 {
            let mut ll = Vec::new();
            while ll.len() < 3000 {
                let c = (rnd() % 5) as u8;
                let run = [1, 2, 15, 16, 17, 31, 32, 33, 48, 100][(rnd() % 10) as usize];
                ll.extend(std::iter::repeat_n(c, run));
            }
            ll.truncate(3000 - (rnd() % 16) as usize);
            inputs.push(ll);
        }
        inputs.push((0..5000).map(|_| rnd() as u8).collect());
        inputs.push(
            (0..5000)
                .map(|i| if i < 1003 { (i % 3) as u8 } else { 9 })
                .collect(),
        );
        for ll in &inputs {
            let mut counts = [0u32; 256];
            for &c in ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            build(ll, &counts, &mut tt);
            assert!(tt.0 == one_pass(ll, &counts), "length {}", ll.len());
        }
    }

    /// Columns for the builds' tests: lengths 0 to 40 of one byte, short runs and runs of
    /// every length around 16 at every alignment among other bytes (windows of one run to
    /// sixteen, and runs that end near their bucket's end), random bytes, and a run to the end
    fn build_inputs(rounds: usize, len: usize) -> Vec<Vec<u8>> {
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        let mut rnd = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let mut inputs: Vec<Vec<u8>> = (0..=40).map(|n| vec![3u8; n]).collect();
        for _ in 0..rounds {
            let mut ll = Vec::new();
            while ll.len() < len {
                let c = (rnd() % 5) as u8;
                let run = [1, 2, 3, 4, 5, 7, 15, 16, 17, 31, 32, 33][(rnd() % 12) as usize];
                ll.extend(std::iter::repeat_n(c, run));
            }
            ll.truncate(len - (rnd() % 16) as usize);
            inputs.push(ll);
        }
        inputs.push((0..len).map(|_| rnd() as u8).collect());
        inputs.push(
            (0..len)
                .map(|i| if i < len / 3 { (i % 3) as u8 } else { 9 })
                .collect(),
        );
        inputs
    }

    fn counts_of(ll: &[u8]) -> [u32; 256] {
        let mut counts = [0u32; 256];
        for &c in ll {
            counts[c as usize] += 1;
        }
        counts
    }

    /// On its own counts, a column's one-pass build is taken and is the two-pass table
    fn one_pass_matches(ll: &[u8]) {
        let counts = counts_of(ll);
        let (mut one, mut two) = (Vec::new(), Vec::new());
        assert!(
            build_one_pass(ll, &counts, &mut one),
            "length {}: refused",
            ll.len()
        );
        build_two_pass(ll, &counts, &mut two);
        assert!(one == two, "length {}", ll.len());
    }

    /// Counts that are not the column's own but keep the two-pass build's writes in bounds
    /// (one count moved from the first byte present to the last, so the first bucket
    /// overflows into the next; or one count short, so the counts sum to n - 1): the one-pass
    /// build refuses them, and `build` gives the two-pass table, as before the one-pass build
    fn wrong_counts_fall_back(ll: &[u8]) {
        let counts = counts_of(ll);
        let present: Vec<usize> = (0..256).filter(|&c| counts[c] > 0).collect();
        let (Some(&lo), Some(&hi)) = (present.first(), present.last()) else {
            return;
        };
        let mut moved = counts;
        let mut short = counts;
        short[hi] -= 1;
        let mut cases = vec![short];
        if lo != hi {
            moved[lo] -= 1;
            moved[hi] += 1;
            cases.push(moved);
        }
        for wrong in cases {
            let mut one = Vec::new();
            assert!(
                !build_one_pass(ll, &wrong, &mut one),
                "length {}: accepted",
                ll.len()
            );
            assert!(one.is_empty(), "a refused build leaves the table empty");
            let (mut tt, mut want) = (TVector::new(), Vec::new());
            build(ll, &wrong, &mut tt);
            build_two_pass(ll, &wrong, &mut want);
            assert!(tt.0 == want, "length {}: not the two-pass table", ll.len());
        }
    }

    #[test]
    fn one_pass_build_is_the_two_pass_build() {
        for ll in build_inputs(40, 3000) {
            one_pass_matches(&ll);
        }
    }

    #[test]
    fn wrong_counts_take_the_two_pass_build() {
        for ll in build_inputs(10, 3000).iter().filter(|ll| !ll.is_empty()) {
            wrong_counts_fall_back(ll);
        }
    }

    /// `few_runs` is #310's `heads.count_ones() <= RUNS` on every mask `run_joins` can
    /// return (bit 0 clear, any of bits 1 to 15, none above), with heads `joins ^ 0xffff`;
    /// and the build's classifier as a whole (`ALL_JOINED`, then 0, then `few_runs`) sends
    /// each window where #310's did: one run, at most RUNS runs, or one row at a time
    #[test]
    fn few_runs_counts_the_runs() {
        for high in 0..1u32 << 15 {
            let joins = high << 1;
            let heads = joins ^ 0xffff;
            assert_eq!(heads & 1, 1);
            assert_eq!(
                few_runs(joins),
                heads.count_ones() as usize <= RUNS,
                "joins {joins:#06x}"
            );
            let new = if joins == ALL_JOINED {
                0
            } else if joins != 0 && few_runs(joins) {
                1
            } else {
                2
            };
            let old = if heads == 1 {
                0
            } else if heads.count_ones() as usize <= RUNS {
                1
            } else {
                2
            };
            assert_eq!(new, old, "joins {joins:#06x}");
        }
    }

    /// Miri's columns reach each of the build's window paths: one run, at most RUNS runs,
    /// more than RUNS runs but fewer than 16 (the run count's table) and 16 runs (the
    /// first compare)
    #[test]
    fn miri_inputs_reach_every_window_path() {
        let mut seen = [false; 4];
        for ll in build_inputs(3, 80).iter().filter(|ll| ll.len() <= 80) {
            for w in ll.chunks_exact(16) {
                let runs = 16 - run_joins(w).count_ones() as usize;
                assert_eq!(runs, 1 + (1..16).filter(|&j| w[j] != w[j - 1]).count());
                seen[match runs {
                    1 => 0,
                    r if r <= RUNS => 1,
                    16 => 3,
                    _ => 2,
                }] = true;
            }
        }
        assert_eq!(seen, [true; 4], "one run, few runs, many runs, 16 runs");
    }

    /// The one-pass build's unchecked writes and its `set_len` under Miri, on small columns:
    /// accepted, refused before the scatter (counts not summing to n) and refused mid-scatter
    /// (an overflowing bucket). With counts summing to n and every write inside its bucket,
    /// no bucket can end short, so the end check is the safety argument's step, not a path
    #[test]
    fn miri_build_one_pass() {
        for ll in build_inputs(3, 80).iter().filter(|ll| ll.len() <= 80) {
            one_pass_matches(ll);
            if !ll.is_empty() {
                wrong_counts_fall_back(ll);
            }
        }
    }

    /// The room walk on degenerate columns (cycles of one and two rows: segments of one or
    /// two bytes, ending in every round of a pass), at every lane count's edge
    #[test]
    fn room_walk_short_cycles() {
        let n = MIN_SEGMENTED + 77;
        for ll in [
            vec![3u8; n],
            (0..n).map(|i| (i % 2) as u8).collect::<Vec<u8>>(),
        ] {
            let mut counts = [0u32; 256];
            for &c in &ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let (mut plain, mut room, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
            for orig_ptr in [0, 1, n as u32 / 2, n as u32 - 1] {
                walk(&tt, orig_ptr, &mut plain);
                for segments in [33, 64, 512, 4096] {
                    walk_segments_room_with_prefetch::<32, PF_L1>(
                        &mut tt, orig_ptr, &mut room, &mut seg, segments,
                    );
                    assert!(plain == room, "orig_ptr {orig_ptr}, {segments} segments");
                }
            }
        }
    }

    /// The room walk on the permutations no encoder writes (tests/decode.rs's crafted
    /// blocks): random columns over 1 to 256 symbols, periodic ones (lists of several
    /// cycles), at random primary indices. A pass that ended with a buffer exactly full and
    /// then handed its lanes to the tail once wrote a byte past it (found by those blocks).
    #[test]
    fn room_walk_crafted_permutations() {
        let mut x = 99u64;
        let mut rnd = move |m: u64| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x % m
        };
        for case in 0..20000 {
            let n = MIN_SEGMENTED + rnd(200_000) as usize;
            let alphabet = 1 + rnd(if case % 2 == 0 { 3 } else { 256 });
            let mut ll: Vec<u8> = (0..n).map(|_| (rnd(alphabet) * 7) as u8).collect();
            if case % 5 == 0 {
                let p = 1 + rnd(5) as usize;
                for i in p..n {
                    ll[i] = ll[i % p];
                }
            }
            let orig_ptr = rnd(n as u64) as u32;
            let mut counts = [0u32; 256];
            for &c in &ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let (mut plain, mut room, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
            walk(&tt, orig_ptr, &mut plain);
            walk_segments_room(&mut tt, orig_ptr, &mut room, &mut seg);
            assert!(plain == room, "case {case} n {n} alphabet {alphabet}");
            check_segments(&tt, &seg, "room walk");
            walk_segments_flagged(&mut tt, orig_ptr, &mut room, &mut seg);
            assert!(plain == room, "flagged walk, case {case}");
            check_segments(&tt, &seg, "flagged walk");
        }
        // the transitions the walk's handoffs guard were all reached (the one that
        // overran, a lane coming into the tail with its buffer full, among them)
        assert_reached();
    }

    /// Asserts that every transition the room walk's handoffs guard was reached on this
    /// thread
    fn assert_reached() {
        for (name, c) in [
            ("a buffer grown", &GROWS),
            ("a pass ending with a buffer full", &PASS_ENDS_FULL),
            ("a segment taken mid-pass", &MID_PASS_TAKES),
            ("the full phase ending mid-pass", &FULL_PHASE_ENDS_MID_PASS),
            ("a lane coming into the tail full", &TAIL_ENTERED_FULL),
        ] {
            assert!(count(c) > 0, "never reached: {name}");
        }
    }

    /// The room walk's deferred handovers, against `walk` on random columns small enough
    /// for many lanes to reach a segment's end at the same offset: every interleaving the
    /// pass can take (several lanes marked at one offset, the queue running out with marked
    /// lanes still waiting, or after an earlier marked lane took a segment) is reached, at
    /// 32 lanes and at 5, and every output is `walk`'s
    #[test]
    fn room_walk_deferred_handovers() {
        for c in [
            &MARKED_SEVERAL,
            &ENDS_WITH_LANES_WAITING,
            &ENDS_AFTER_A_TAKE,
            &LATER_STEPPED_FULL,
            &LATER_STEPPED_ON_START,
        ] {
            c.with(|c| c.set(0));
        }
        let mut x = 0x2545_f491_4f6c_dd1du64;
        let mut rnd = move |m: u64| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x % m
        };
        for case in 0..20000 {
            let n = 64 + rnd(if case % 3 == 0 { 600 } else { 3000 }) as usize;
            let alphabet = 1 + rnd(if case % 2 == 0 { 3 } else { 256 });
            let ll: Vec<u8> = (0..n).map(|_| (rnd(alphabet) * 7) as u8).collect();
            let orig_ptr = rnd(n as u64) as u32;
            let mut counts = [0u32; 256];
            for &c in &ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let (mut plain, mut out, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
            walk(&tt, orig_ptr, &mut plain);
            // a few segments more than lanes run the queue out early, while the first
            // passes' lanes are still the ones setting the room
            let segments = if case % 2 == 0 {
                33 + rnd(32) as usize
            } else {
                2 + rnd(n as u64 / 2) as usize
            };
            walk_segments_room_with_prefetch::<32, PF_L1>(
                &mut tt, orig_ptr, &mut out, &mut seg, segments,
            );
            assert!(plain == out, "32 lanes, case {case}");
            check_segments(&tt, &seg, "32 lanes");
            walk_segments_room_with_prefetch::<5, PF_OFF>(
                &mut tt, orig_ptr, &mut out, &mut seg, segments,
            );
            assert!(plain == out, "5 lanes, case {case}");
            check_segments(&tt, &seg, "5 lanes");
        }
        // A stepped later lane leaving exactly full needs the queue to run out at a pass's
        // last offset with that lane setting the pass's room: not reached in 60,000 random
        // columns. It is the tail's existing case of a lane coming in full (reached in
        // `miri_room_walk_handoffs`), so LATER_STEPPED_FULL is counted but not required.
        for (name, c) in [
            ("several lanes marked at one offset", &MARKED_SEVERAL),
            (
                "the queue running out with lanes waiting",
                &ENDS_WITH_LANES_WAITING,
            ),
            ("the queue running out after a take", &ENDS_AFTER_A_TAKE),
            (
                "a stepped later lane leaving on its segment's end",
                &LATER_STEPPED_ON_START,
            ),
        ] {
            assert!(count(c) > 0, "never reached: {name}");
        }
    }

    /// The staged busy phase's chunk edges, at 32 lanes and at 5: blocks long enough and
    /// segments few enough that a pass's room spans several tiles. Each state is required:
    /// a pass of more than one chunk; a take at a chunk's last offset (its row then staged
    /// from one past the chunk) and in a later chunk; a segment taken and finished in one
    /// chunk; a lane marked at a later chunk's first offset; and the queue running out at a
    /// later chunk's first offset, at a chunk's last offset short of the pass's end, and at
    /// the pass's last offset. Every output against `walk` and the segment oracle
    #[test]
    #[cfg_attr(miri, ignore)]
    fn room_walk_staged_chunk_edges() {
        let required = [
            ("a pass of more than one chunk", &ROOM_OVER_TILE),
            ("a take at a chunk's last offset", &TAKE_AT_CHUNK_END),
            ("a take in a later chunk", &TAKE_IN_LATER_CHUNK),
            (
                "a segment taken and finished in one chunk",
                &DONE_IN_TAKING_CHUNK,
            ),
            (
                "a lane marked at a later chunk's first offset",
                &MARK_AT_LATER_CHUNK_START,
            ),
            (
                "the queue running out at a later chunk's first offset",
                &EXHAUSTED_AT_CHUNK_START,
            ),
            (
                "the queue running out at a chunk's last offset",
                &EXHAUSTED_AT_CHUNK_END,
            ),
            (
                "the queue running out at a pass's last offset",
                &EXHAUSTED_AT_PASS_END,
            ),
            ("a lane taken mid-pass", &MID_PASS_TAKES),
        ];
        for (_, c) in required {
            c.with(|c| c.set(0));
        }
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        let mut rnd = move |m: u64| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x % m
        };
        let mut cases = 0;
        while cases < 20000 && required.iter().any(|(_, c)| count(c) == 0) {
            cases += 1;
            let n = 4000 + rnd(28000) as usize;
            let alphabet = 1 + rnd(if cases % 2 == 0 { 3 } else { 256 });
            let ll: Vec<u8> = (0..n).map(|_| (rnd(alphabet) * 7) as u8).collect();
            let orig_ptr = rnd(n as u64) as u32;
            let mut counts = [0u32; 256];
            for &c in &ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let (mut plain, mut out, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
            walk(&tt, orig_ptr, &mut plain);
            // a pass's room is about 2n / segments + 63: under n / 150 segments it is over
            // two tiles, and over 32 of them the busy phase runs at 32 lanes
            let segments = 33 + rnd((n / 150) as u64) as usize;
            walk_segments_room_with_prefetch::<32, PF_L1>(
                &mut tt, orig_ptr, &mut out, &mut seg, segments,
            );
            assert!(plain == out, "32 lanes, case {cases}");
            check_segments(&tt, &seg, "32 lanes");
            walk_segments_room_with_prefetch::<5, PF_OFF>(
                &mut tt, orig_ptr, &mut out, &mut seg, segments,
            );
            assert!(plain == out, "5 lanes, case {cases}");
            check_segments(&tt, &seg, "5 lanes");
        }
        for (name, c) in required {
            assert!(count(c) > 0, "never reached in {cases} cases: {name}");
        }
    }

    /// A block of more than 2^23 rows is refused rather than walked with its links misread
    #[test]
    #[cfg_attr(miri, ignore)]
    #[should_panic(expected = "at most 2^23 rows")]
    fn room_walk_refuses_more_than_2_23_rows() {
        let n = (1 << 23) + 1;
        let mut tt = TVector((0..n as u32).map(|r| r << 8).collect());
        let (mut out, mut seg) = (Vec::new(), SegScratch::default());
        walk_segments_room_with_prefetch::<32, PF_L1>(&mut tt, 0, &mut out, &mut seg, 64);
    }

    /// The flagged walk on a random last column and on a periodic one (a list of several
    /// cycles), both past MIN_SEGMENTED so the segments are walked
    #[test]
    fn flagged_walk_equals_plain() {
        let n = MIN_SEGMENTED + 1000;
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        let random: Vec<u8> = (0..n)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                (x % 7) as u8
            })
            .collect();
        check(&random);
        let periodic: Vec<u8> = (0..n).map(|i| b"abc"[i % 3]).collect();
        check(&periodic);
    }

    /// The segmented walks segment a block of any length (the wrappers leave those below
    /// MIN_SEGMENTED to `walk`): every length edge, through both, at lane and segment
    /// counts below, at and above the lanes
    #[test]
    fn segmented_walks_on_any_length() {
        let mut x = 7u64;
        for n in [
            1,
            2,
            3,
            31,
            32,
            33,
            100,
            1000,
            MIN_SEGMENTED - 1,
            MIN_SEGMENTED,
            MIN_SEGMENTED + 1,
        ] {
            let ll: Vec<u8> = (0..n)
                .map(|_| {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    (x % 3) as u8
                })
                .collect();
            let mut counts = [0u32; 256];
            for &c in &ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let (mut plain, mut out, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
            for orig_ptr in [0, n as u32 / 2, n as u32 - 1] {
                walk(&tt, orig_ptr, &mut plain);
                walk_segments_room(&mut tt, orig_ptr, &mut out, &mut seg);
                assert!(plain == out, "room walk, n {n}");
                walk_segments_flagged(&mut tt, orig_ptr, &mut out, &mut seg);
                assert!(plain == out, "flagged walk, n {n}");
                for segments in [1, 2, 33, 256] {
                    let what = format!("n {n}, {segments} segments");
                    walk_segments_room_with_prefetch::<32, PF_L1>(
                        &mut tt, orig_ptr, &mut out, &mut seg, segments,
                    );
                    assert!(plain == out, "room walk, {what}");
                    check_segments(&tt, &seg, &what);
                    walk_segments_room_with_prefetch::<5, PF_OFF>(
                        &mut tt, orig_ptr, &mut out, &mut seg, segments,
                    );
                    assert!(plain == out, "room walk, 5 lanes, {what}");
                    check_segments(&tt, &seg, &what);
                    walk_segments_flagged_with_prefetch::<32, PF_L1>(
                        &mut tt, orig_ptr, &mut out, &mut seg, segments,
                    );
                    assert!(plain == out, "flagged walk, {what}");
                    check_segments(&tt, &seg, &what);
                    walk_segments_flagged_with_prefetch::<7, PF_OFF>(
                        &mut tt, orig_ptr, &mut out, &mut seg, segments,
                    );
                    assert!(plain == out, "flagged walk, 7 lanes, {what}");
                    check_segments(&tt, &seg, &what);
                }
                assert!(tt.0.iter().all(|&e| e & START == 0));
            }
        }
    }

    /// For Miri (`cargo +nightly miri test --lib -- miri_`): the segmented walks' unsafe
    /// code on blocks small enough to interpret, each found to reach every transition the
    /// room walk's handoffs guard. The test asserts it reached them, so a change to the walk
    /// that stops these blocks reaching one fails here rather than passing having checked
    /// less. (Each is 32 lanes and 256 segments on a random column of about 3,000 rows: a
    /// block that small needs that many segments for a buffer to grow; the counts were
    /// searched for natively, and these are the smallest found.)
    #[test]
    fn miri_room_walk_handoffs() {
        for seed in [170_778u64, 8_824] {
            let mut x = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
            let mut rnd = move |m: u64| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x % m
            };
            let n = 200 + rnd(4000) as usize;
            let alphabet = 1 + rnd(if seed % 2 == 0 { 3 } else { 256 });
            let ll: Vec<u8> = (0..n).map(|_| (rnd(alphabet) * 7) as u8).collect();
            let orig_ptr = rnd(n as u64) as u32;
            let mut counts = [0u32; 256];
            for &c in &ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let (mut plain, mut out, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
            walk(&tt, orig_ptr, &mut plain);
            walk_segments_room_with_prefetch::<32, PF_L1>(
                &mut tt, orig_ptr, &mut out, &mut seg, 256,
            );
            assert!(plain == out, "room walk, seed {seed}");
            check_segments(&tt, &seg, "room walk");
            walk_segments_flagged_with_prefetch::<32, PF_L1>(
                &mut tt, orig_ptr, &mut out, &mut seg, 256,
            );
            assert!(plain == out, "flagged walk, seed {seed}");
            check_segments(&tt, &seg, "flagged walk");
        }
        assert_reached();
        // 0.1.5's signatures (a bool switch) still build and give walk's output
        {
            let ll: Vec<u8> = (0..5000u32).map(|i| (i * 7 % 13) as u8).collect();
            let mut counts = [0u32; 256];
            for &c in &ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let (mut plain, mut out, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
            walk(&tt, 17, &mut plain);
            walk_segments_room_with::<32, true>(&mut tt, 17, &mut out, &mut seg, 64);
            assert!(plain == out, "room walk, 0.1.5's signature, prefetch on");
            walk_segments_room_with::<5, false>(&mut tt, 17, &mut out, &mut seg, 64);
            assert!(plain == out, "room walk, 0.1.5's signature, prefetch off");
            walk_segments_flagged_with::<32, true>(&mut tt, 17, &mut out, &mut seg, 64);
            assert!(plain == out, "flagged walk, 0.1.5's signature, prefetch on");
            walk_segments_flagged_with::<7, false>(&mut tt, 17, &mut out, &mut seg, 64);
            assert!(
                plain == out,
                "flagged walk, 0.1.5's signature, prefetch off"
            );
        }
        // and the degenerate columns (segments of one and two bytes), on a smaller block
        let n = 600;
        for ll in [vec![3u8; n], (0..n).map(|i| (i % 2) as u8).collect()] {
            let mut counts = [0u32; 256];
            for &c in &ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let (mut plain, mut out, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
            walk(&tt, 1, &mut plain);
            for segments in [33, 256] {
                walk_segments_room_with_prefetch::<32, PF_L1>(
                    &mut tt, 1, &mut out, &mut seg, segments,
                );
                assert!(plain == out, "room walk, {segments} segments");
                check_segments(&tt, &seg, "room walk");
                walk_segments_flagged_with_prefetch::<32, PF_L1>(
                    &mut tt, 1, &mut out, &mut seg, segments,
                );
                assert!(plain == out, "flagged walk, {segments} segments");
                check_segments(&tt, &seg, "flagged walk");
            }
        }
    }
}
