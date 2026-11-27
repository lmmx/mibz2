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

pub use crate::cells::{
    Cells, CellsKernel, CellsVariant, Counts as CellsCounts, ForcedCells, ForcedLazyCells,
    ForcedW12LazyCells, LazyCells, W12LazyCells,
};
#[cfg(feature = "memory-lever")]
pub use crate::cells::{ForcedStagedCells, StagedCells};
/// The workers' kernel wherever production ran `LazyCells`: the open kernel on `LazyCells`'
/// admitted blocks, `LazyCells` as shipped elsewhere (`open.rs`)
pub use crate::open::{ForcedOpenCells, OpenCells, OpenCellsOf};

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

    /// Research only (`mtf-fold`): whether this kernel inverts a run-route block straight
    /// from its symbols (`fold_runs`, the MTF-into-construction fold)
    #[cfg(feature = "mtf-fold")]
    fn folds(&self) -> bool {
        false
    }

    /// Research only (`mtf-fold`): the block `b` inverted straight from its symbols into
    /// `buf`, its first pass fed by the MTF/RLE2 front (`runs::fold`), calling
    /// `admitted(n, syms)` once its rows are known and it is admitted; or the outcome that
    /// sends the caller the shipped way, with nothing handed over. Called only when `folds`
    /// is true.
    #[cfg(feature = "mtf-fold")]
    fn fold_runs(
        &mut self,
        b: &mut crate::block::Symbols,
        scratch: &mut crate::runs::Runs,
        buf: &mut Vec<u8>,
        k: crate::Kernels,
        admitted: &mut dyn FnMut(usize, Vec<u16>),
    ) -> crate::runs::fold::Outcome {
        let _ = (b, scratch, buf, k, admitted);
        crate::runs::fold::Outcome::Bailed
    }

    /// Whether the last `invert_runs` that returned true was done by Cells, in its own
    /// buffers, rather than by the fallback's table: the footprint ledger's route only
    /// (src/trace.rs), so that a Cells block is not charged the fallback's T-vector
    #[cfg(any(test, feature = "trace"))]
    fn took_cells(&self) -> bool {
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

/// Maximum row count for the stack-buffered fused small-block path (covers all Wikidata
/// non-Open runts with zero heap allocation).
const SMALL_FUSED_MAX: usize = 4096;

#[derive(Default)]
enum FusedState {
    #[default]
    Idle,
    /// `decode_owned` Runs route: `Segmented::invert_runs` writes RLE1-decoded bytes directly
    /// into its `out` parameter (cleared first) and stores the block CRC result in `Done`.
    Owned(crate::Kernels),
    /// `decode_block` Runs route and `column`: `Segmented` appends RLE1-decoded bytes directly
    /// to `*out` (leaving `out[..start]` untouched on error) and stores the block CRC in `Done`.
    Append {
        out: *mut Vec<u8>,
        k: crate::Kernels,
    },
    /// Completed fused post-walk CRC32 result (or `RunUnfinished`), awaiting `FusedPostwalk::take`.
    Done(Result<u32, crate::ErrorKind>),
}

thread_local! {
    static FUSED_POSTWALK: std::cell::Cell<FusedState> = const {
        std::cell::Cell::new(FusedState::Idle)
    };
}

/// Panic-safe RAII scope letting `decode_block` / `decode_owned` receive a fused
/// reverse+RLE1+CRC32 result directly from `Segmented` without altering the public `InverseBwt`
/// contract or wrapper types (`OpenCells`, `LazyCells`, `Cells`).
pub(crate) struct FusedPostwalk<'a> {
    prev: FusedState,
    active: bool,
    _borrow: std::marker::PhantomData<&'a mut Vec<u8>>,
}

impl<'a> FusedPostwalk<'a> {
    /// Arm fused post-walk for `decode_owned`'s Runs route (`Segmented::invert_runs` writes
    /// RLE1-decoded bytes directly into its own `out` parameter).
    #[inline]
    pub(crate) fn enter_owned(k: crate::Kernels) -> FusedPostwalk<'static> {
        let prev = FUSED_POSTWALK.with(|c| c.replace(FusedState::Owned(k)));
        FusedPostwalk {
            prev,
            active: true,
            _borrow: std::marker::PhantomData,
        }
    }

    /// Arm fused post-walk appending directly to `out` (for `decode_block` Runs and `column`).
    ///
    /// Holding `&'a mut Vec<u8>` via `PhantomData` statically prevents the caller from touching
    /// `out` until this guard is consumed or dropped.
    #[inline]
    pub(crate) fn enter_append(out: &'a mut Vec<u8>, k: crate::Kernels) -> Self {
        let prev = FUSED_POSTWALK.with(|c| {
            c.replace(FusedState::Append {
                out: out as *mut Vec<u8>,
                k,
            })
        });
        FusedPostwalk {
            prev,
            active: true,
            _borrow: std::marker::PhantomData,
        }
    }

    /// Disarm the scope and return the fused CRC32 result if `Segmented` ran it.
    #[inline]
    pub(crate) fn take(mut self) -> Option<Result<u32, crate::ErrorKind>> {
        self.active = false;
        let prev = std::mem::take(&mut self.prev);
        let cur = FUSED_POSTWALK.with(|c| c.replace(prev));
        match cur {
            FusedState::Done(res) => Some(res),
            _ => None,
        }
    }
}

impl Drop for FusedPostwalk<'_> {
    fn drop(&mut self) {
        if self.active {
            let prev = std::mem::take(&mut self.prev);
            FUSED_POSTWALK.with(|c| c.set(prev));
        }
    }
}

/// The segmented walk in passes (`walk_segments_room`), the plain one for a short block:
/// the default
#[derive(Default)]
pub struct Segmented {
    tt: TVector,
    seg: SegScratch,
}

impl Segmented {
    /// Walk private `self.tt` backward with `clear_start = false` (skipping redundant
    /// `START`-bit cleanup on private `self.tt`, which is rebuilt before every walk) and run
    /// the fused reverse+RLE1+CRC32 pass into `dst`.
    ///
    /// Returns `true` if the fused pass ran (and stored its result in `FUSED_POSTWALK`), or
    /// `false` if the block fell in `SMALL_FUSED_MAX < n < MIN_SEGMENTED` and should use the
    /// unfused `walk_back` path.
    fn run_back_fused(
        &mut self,
        orig_ptr: u32,
        dst: &mut Vec<u8>,
        clear_dst: bool,
        k: crate::Kernels,
    ) -> bool {
        let n = self.tt.len();
        if n >= MIN_SEGMENTED {
            room_segments::<FLAGGED_LANES, PREFETCH, true>(
                &mut self.tt,
                orig_ptr,
                &mut self.seg,
                FLAGGED_SEGMENTS,
                false,
            );
            if clear_dst {
                dst.clear();
            }
            let res = self.seg.assemble_reversed_rle_crc(dst, k);
            FUSED_POSTWALK.with(|c| c.set(FusedState::Done(res)));
            true
        } else if n <= SMALL_FUSED_MAX {
            if clear_dst {
                dst.clear();
            }
            let res = walk_back_fused_small(&self.tt, orig_ptr, dst, k);
            FUSED_POSTWALK.with(|c| c.set(FusedState::Done(res)));
            true
        } else {
            false
        }
    }
}

impl InverseBwt for Segmented {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        if build_lf(ll, counts, &mut self.tt) {
            walk_back_room_private(
                &mut self.tt,
                orig_ptr,
                out,
                &mut self.seg,
                crate::Kernels::detect(),
            );
        } else {
            build(ll, counts, &mut self.tt);
            walk_segments_room_private(&mut self.tt, orig_ptr, out, &mut self.seg);
        }
    }

    // The build is `ll`'s last read: it copies every byte and link the walk needs into the
    // table. The walk (segmented, or plain for a short block) then reads only the table and
    // the segment scratch, so its output can go over `ll`. Both writers (`walk`/`walk_back`,
    // `SegScratch::assemble`/`assemble_reversed`) clear their output and write exactly the
    // table's n rows, and n = ll.len() fits the capacity `ll` already has. So the success
    // contract holds, and Segmented never falls back.
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], orig_ptr: u32) -> bool {
        if build_lf(ll, counts, &mut self.tt) {
            let mode = FUSED_POSTWALK.with(|c| c.take());
            if let FusedState::Append { out: dst_ptr, k } = mode {
                // SAFETY: `dst_ptr` was installed by `FusedPostwalk::enter_append(out, k)` on
                // this thread, which holds an exclusive `&'a mut Vec<u8>` borrow for the
                // dynamic extent of this call, and `take()` removed `dst_ptr` from TLS so no
                // alias can be created. `dst_ptr` is disjoint from `&mut self` and `ll`.
                let dst = unsafe { &mut *dst_ptr };
                if self.run_back_fused(orig_ptr, dst, false, k) {
                    return true;
                }
                // `SMALL_FUSED_MAX < n < MIN_SEGMENTED`: leave `FUSED_POSTWALK` idle and fall
                // through to `walk_back` into `ll` so the caller runs `finish`.
                walk_back(&self.tt, orig_ptr, ll);
                return true;
            }
            FUSED_POSTWALK.with(|c| c.set(mode));
            walk_back_room_private(
                &mut self.tt,
                orig_ptr,
                ll,
                &mut self.seg,
                crate::Kernels::detect(),
            );
        } else {
            build(ll, counts, &mut self.tt);
            walk_segments_room_private(&mut self.tt, orig_ptr, ll, &mut self.seg);
        }
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
        let mode = FUSED_POSTWALK.with(|c| c.take());
        match mode {
            FusedState::Owned(fk) => {
                if self.run_back_fused(orig_ptr, out, true, fk) {
                    return true;
                }
                walk_back(&self.tt, orig_ptr, out);
                true
            }
            FusedState::Append {
                out: dst_ptr,
                k: fk,
            } => {
                // SAFETY: `dst_ptr` was installed by `FusedPostwalk::enter_append(out, k)` on
                // this thread, which holds an exclusive `&'a mut Vec<u8>` borrow for the
                // dynamic extent of this call, and `take()` removed `dst_ptr` from TLS so no
                // alias can be created. `dst_ptr` is disjoint from `&mut self` and `out`.
                let dst = unsafe { &mut *dst_ptr };
                if self.run_back_fused(orig_ptr, dst, false, fk) {
                    return true;
                }
                walk_back(&self.tt, orig_ptr, out);
                true
            }
            other => {
                FUSED_POSTWALK.with(|c| c.set(other));
                walk_back_room_private(&mut self.tt, orig_ptr, out, &mut self.seg, k);
                true
            }
        }
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
            walk_back_room_with_kernels(&mut self.tt, orig_ptr, out, &mut self.seg, k);
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
        self.seg.assemble_reversed_with(out, k);
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

/// The LF table of the last column `ll`, whose bytes `counts` counts, into `tt`:
/// `tt[i] = ll[i] | LF(i) << 8`, with `LF(i) = C(ll[i]) + occ(ll[i], i)`.
///
/// Unlike `build` (which scatters row `i` into its byte's bucket at `s = ftab[c]`, doing a
/// random write into a 3.6 MB table and a second random read of `ll[s]`), this reads `ll`
/// sequentially and writes `tt` sequentially: zero random stores, zero `ll` random loads,
/// with only the 1 KiB `ftab` cursor array hot in L1. Equal-byte runs of 8 rows advance the
/// bucket cursor once and write 8 sequential entries.
///
/// Returns `true` when `n < 2^24` and `counts` matches `ll` (verified after the pass by
/// `ftab == lim`), establishing `TVector`'s invariant that every link `(e >> 8)` is below `n`.
/// Otherwise leaves `tt` empty and returns `false` so the caller can fall back to `build`.
pub(crate) fn build_lf(ll: &[u8], counts: &[u32; 256], tt: &mut TVector) -> bool {
    let n = ll.len();
    tt.0.clear();
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
    tt.0.reserve(n);
    let src = ll.as_ptr();
    let dst = tt.0.as_mut_ptr();
    let mut i = 0usize;
    while i + 8 <= n {
        // SAFETY: `i + 8 <= n`, so `src.add(i)` is valid for an unaligned 8-byte read.
        let w = unsafe { (src.add(i) as *const u64).read_unaligned() };
        if w == w.rotate_left(8) {
            let c = (w & 0xff) as usize;
            let s = ftab[c];
            ftab[c] = s.wrapping_add(8);
            let base = (c as u32) | (s << 8);
            // SAFETY: `i + 8 <= n`, within `tt.0`'s reserved capacity `n`.
            unsafe {
                let d = dst.add(i);
                *d = base;
                *d.add(1) = base.wrapping_add(1 << 8);
                *d.add(2) = base.wrapping_add(2 << 8);
                *d.add(3) = base.wrapping_add(3 << 8);
                *d.add(4) = base.wrapping_add(4 << 8);
                *d.add(5) = base.wrapping_add(5 << 8);
                *d.add(6) = base.wrapping_add(6 << 8);
                *d.add(7) = base.wrapping_add(7 << 8);
            }
            i += 8;
        } else {
            // SAFETY: `i + 8 <= n`, within `ll` and `tt.0`'s reserved capacity `n`.
            unsafe {
                for k in 0..8 {
                    let c = *src.add(i + k) as usize;
                    let s = ftab[c];
                    ftab[c] = s.wrapping_add(1);
                    *dst.add(i + k) = (c as u32) | (s << 8);
                }
            }
            i += 8;
        }
    }
    while i < n {
        // SAFETY: `i < n`, within `ll` and `tt.0`'s reserved capacity `n`.
        unsafe {
            let c = *src.add(i) as usize;
            let s = ftab[c];
            ftab[c] = s.wrapping_add(1);
            *dst.add(i) = (c as u32) | (s << 8);
        }
        i += 1;
    }
    if ftab != lim {
        return false;
    }
    // SAFETY: every slot in `0..n` was written above and `ftab == lim` proves every stored
    // link is strictly below `n` (`n < 2^24`).
    unsafe { tt.0.set_len(n) };
    true
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

/// Reverse-copy `len` bytes from `src` to `dst` (`dst[len - 1 - i] = src[i]`): 8 bytes at a
/// time via `u64::swap_bytes`, then one byte at a time for the remainder.
///
/// # Safety
///
/// `src` must be valid for reads of `len` initialized bytes, `dst` must be valid for writes of
/// `len` bytes, and `[src, src + len)` must not overlap `[dst, dst + len)`.
#[inline(always)]
unsafe fn reverse_copy_scalar(src: *const u8, dst: *mut u8, len: usize) {
    let mut i = 0usize;
    while i + 8 <= len {
        // SAFETY: `i + 8 <= len`, so both `src.add(i)` and `dst.add(len - i - 8)` are within
        // their respective non-overlapping `len`-byte ranges.
        unsafe {
            let w = (src.add(i) as *const u64).read_unaligned().swap_bytes();
            (dst.add(len - i - 8) as *mut u64).write_unaligned(w);
        }
        i += 8;
    }
    while i < len {
        // SAFETY: `i < len`, so `len - 1 - i < len`, within both non-overlapping ranges.
        unsafe {
            *dst.add(len - 1 - i) = *src.add(i);
        }
        i += 1;
    }
}

/// Reverse-copy `len` bytes from `src` to `dst` using 32-byte AVX2 `pshufb` + `vperm2i128`
/// chunks, followed by `reverse_copy_scalar` for the `< 32`-byte tail.
///
/// # Safety
///
/// AVX2 must be available. `src` must be valid for reads of `len` initialized bytes, `dst`
/// must be valid for writes of `len` bytes, and the two ranges must not overlap.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn reverse_copy_avx2(src: *const u8, dst: *mut u8, len: usize) {
    use core::arch::x86_64::*;
    let rev = _mm256_setr_epi8(
        15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6,
        5, 4, 3, 2, 1, 0,
    );
    let mut i = 0usize;
    while i + 32 <= len {
        // SAFETY: `i + 32 <= len`, so `src.add(i)` is valid for a 32-byte unaligned load and
        // `dst.add(len - i - 32)` is valid for a 32-byte unaligned store.
        unsafe {
            let v = _mm256_loadu_si256(src.add(i) as *const __m256i);
            let s = _mm256_shuffle_epi8(v, rev);
            let r = _mm256_permute2x128_si256::<0x01>(s, s);
            _mm256_storeu_si256(dst.add(len - i - 32) as *mut __m256i, r);
        }
        i += 32;
    }
    // SAFETY: `i <= len`; the remaining `len - i` bytes at `src.add(i)` reverse into the
    // leading `len - i` bytes of `dst` (non-overlapping, within bounds).
    unsafe { reverse_copy_scalar(src.add(i), dst, len - i) };
}

impl SegScratch {
    /// The room walk's segments of an LF table walked backward (`walk_back_room`): joined
    /// in the order `assemble` joins them, which is the block's bytes last to first, and
    /// written into `out` (which they replace) from its end, each segment reversed, so that
    /// `out` holds the block first to last. The join stops where it comes back to its first
    /// segment: the primary index's cycle is then complete (`repeat_cycle`).
    pub(crate) fn assemble_reversed(&self, out: &mut Vec<u8>) {
        self.assemble_reversed_with(out, crate::Kernels::detect());
    }

    /// `assemble_reversed` with an explicit `Kernels` dispatch token (so callers with
    /// `Kernels::scalar()` force the portable path even on AVX2 CPUs, while `k.avx2` remains
    /// a type-enforced `Kernels` invariant).
    pub(crate) fn assemble_reversed_with(&self, out: &mut Vec<u8>, k: crate::Kernels) {
        let n = self.n;
        out.clear();
        out.reserve(n);
        let dst = &mut out.spare_capacity_mut()[..n];
        let mut at = n;
        let mut seg = self.first;
        while at > 0 {
            let b = &self.bufs[seg];
            let len = b.len().min(at);
            at -= len;
            let d = dst[at..at + len].as_mut_ptr().cast::<u8>();
            #[cfg(target_arch = "x86_64")]
            if k.avx2 && len >= 32 {
                // SAFETY: `k.avx2` is set only when AVX2 was detected (a `Kernels` invariant);
                // `b[..len]` is `len` initialized readable bytes, `d` points to `len` writable
                // bytes of `out.spare_capacity_mut()`, and `&self` vs `&mut out` cannot overlap.
                unsafe { reverse_copy_avx2(b.as_ptr(), d, len) };
            } else {
                // SAFETY: `b[..len]` and `d[..len]` are valid non-overlapping `len`-byte ranges
                // (`&self` and `&mut out` are disjoint).
                unsafe { reverse_copy_scalar(b.as_ptr(), d, len) };
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                let _ = k;
                // SAFETY: `b[..len]` and `d[..len]` are valid non-overlapping `len`-byte ranges
                // (`&self` and `&mut out` are disjoint).
                unsafe { reverse_copy_scalar(b.as_ptr(), d, len) };
            }
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

    /// Single-pass fused reverse-assembly + RLE1 + L2-hot CRC32 for backward LF room walks:
    /// appends the block's decoded bytes to `out` (starting at `start = out.len()`), returns
    /// the CRC32 of `&out[start..]`, and restores `out.truncate(start)` on `RunUnfinished`.
    ///
    /// Eliminates the full-block `assemble_reversed` write, `find_runs` scan, `shift > 0`
    /// `copy_within` pass, `rle_runs` scratch, and `Column`'s `ll` intermediate write.
    pub(crate) fn assemble_reversed_rle_crc(
        &self,
        out: &mut Vec<u8>,
        k: crate::Kernels,
    ) -> Result<u32, crate::ErrorKind> {
        const BUF_CAP: usize = 4096;
        const CRC_TILE: usize = crate::rle::TILE;

        let n = self.n;
        let start = out.len();
        if n == 0 {
            return Ok(crate::crc::crc32(&[], k));
        }

        // Collect segment chain in backward walk order, then reverse for forward order.
        // Production always has `ns <= FLAGGED_SEGMENTS` (512), fitting in 1 KiB on the stack.
        let mut stack_chain = [0u16; FLAGGED_SEGMENTS];
        let mut stack_len = 0usize;
        let mut heap_chain: Vec<usize> = Vec::new();
        let mut total = 0usize;
        let mut seg = self.first;
        while total < n {
            let blen = self.bufs[seg].len().min(n - total);
            if heap_chain.is_empty() && stack_len < FLAGGED_SEGMENTS && seg <= u16::MAX as usize {
                stack_chain[stack_len] = seg as u16;
                stack_len += 1;
            } else {
                if heap_chain.is_empty() {
                    heap_chain.reserve(stack_len + 64);
                    heap_chain.extend(stack_chain[..stack_len].iter().map(|&x| x as usize));
                }
                heap_chain.push(seg);
            }
            total += blen;
            seg = self.next[seg];
            if seg == self.first {
                break;
            }
        }
        assert!(total > 0);

        // Reserve `n + pad` matching `rle::unrle` (and `trace.rs`'s output-buffer ledger) so
        // non-expanding blocks never reallocate and pooled `decode_owned` buffers (`n + n/16 + 64`)
        // hit zero allocation overhead.
        let pad = if cfg!(target_arch = "x86_64") && k.avx2 {
            64
        } else {
            0
        };
        out.reserve(n + pad);

        let res = if total == n {
            if heap_chain.is_empty() {
                stack_chain[..stack_len].reverse();
                #[cfg(target_arch = "x86_64")]
                if k.avx2 {
                    // SAFETY: `k.avx2` guarantees AVX2 + BMI1 (`Kernels` invariant); every
                    // segment index in `stack_chain[..stack_len]` came from `self.first` /
                    // `self.next` and is `< self.bufs.len()`.
                    return unsafe {
                        fused_rev_rle_crc_buf_avx2::<BUF_CAP, CRC_TILE, u16>(
                            &self.bufs,
                            &stack_chain[..stack_len],
                            n,
                            start,
                            out,
                            k,
                        )
                    };
                }
                fused_rev_rle_crc_buf_scalar::<BUF_CAP, CRC_TILE, u16>(
                    &self.bufs,
                    &stack_chain[..stack_len],
                    n,
                    start,
                    out,
                    k,
                )
            } else {
                heap_chain.reverse();
                #[cfg(target_arch = "x86_64")]
                if k.avx2 {
                    // SAFETY: `k.avx2` guarantees AVX2 + BMI1 (`Kernels` invariant); every
                    // segment index in `heap_chain` is `< self.bufs.len()`.
                    return unsafe {
                        fused_rev_rle_crc_buf_avx2::<BUF_CAP, CRC_TILE, usize>(
                            &self.bufs,
                            &heap_chain,
                            n,
                            start,
                            out,
                            k,
                        )
                    };
                }
                fused_rev_rle_crc_buf_scalar::<BUF_CAP, CRC_TILE, usize>(
                    &self.bufs,
                    &heap_chain,
                    n,
                    start,
                    out,
                    k,
                )
            }
        } else {
            // Short cycle (`m = total < n`, e.g. periodic `onebyte` or `b"ab".repeat(...)`):
            // materialize the `m`-byte forward cycle once (on the stack when `m <= 4096`) and
            // stream repeated copies through the fused RLE1 + CRC32 decoder.
            let m = total;
            let mut stack_cyc = [0u8; BUF_CAP];
            let mut heap_cyc: Vec<u8> = Vec::new();
            let cyc: &mut [u8] = if m <= BUF_CAP {
                &mut stack_cyc[..m]
            } else {
                heap_cyc.resize(m, 0);
                &mut heap_cyc[..m]
            };
            let mut off = 0usize;
            if heap_chain.is_empty() {
                for &sg in stack_chain[..stack_len].iter().rev() {
                    let b = &self.bufs[sg as usize];
                    let len = b.len();
                    // SAFETY: `b` and `cyc[off..off + len]` are disjoint valid `len`-byte ranges.
                    unsafe { reverse_copy_scalar(b.as_ptr(), cyc.as_mut_ptr().add(off), len) };
                    off += len;
                }
            } else {
                for &sg in heap_chain.iter().rev() {
                    let b = &self.bufs[sg];
                    let len = b.len();
                    // SAFETY: `b` and `cyc[off..off + len]` are disjoint valid `len`-byte ranges.
                    unsafe { reverse_copy_scalar(b.as_ptr(), cyc.as_mut_ptr().add(off), len) };
                    off += len;
                }
            }
            debug_assert_eq!(off, m);
            #[cfg(target_arch = "x86_64")]
            if k.avx2 {
                // SAFETY: `k.avx2` guarantees AVX2 + BMI1 (`Kernels` invariant).
                return unsafe {
                    fused_rev_rle_crc_cycle_avx2::<BUF_CAP, CRC_TILE>(cyc, n, start, out, k)
                };
            }
            fused_rev_rle_crc_cycle_scalar::<BUF_CAP, CRC_TILE>(cyc, n, start, out, k)
        };
        if res.is_err() {
            out.truncate(start);
        }
        res
    }
}

trait SegIdx: Copy {
    fn as_usize(self) -> usize;
}
impl SegIdx for u16 {
    #[inline(always)]
    fn as_usize(self) -> usize {
        self as usize
    }
}
impl SegIdx for usize {
    #[inline(always)]
    fn as_usize(self) -> usize {
        self
    }
}

/// Cold growth path for the fused AVX2 RLE1 writer.
///
/// When `start == 0` (per-block buffers in `decode_owned` and `BzReader::new`), linearly
/// extrapolates the remaining RLE1 output from `produced / consumed` (with a 1% + 16 KiB
/// headroom, capped at bzip2's theoretical `52x` RLE1 expansion ceiling) and calls
/// `reserve_exact`. Uniformly expanding blocks (`onebyte`, `runs`) resize once to ~1.01x exact
/// size instead of 7–18 incremental `realloc` + `memcpy` passes (+7..+14% RSS). When `start >
/// 0` (multi-block accumulation in `decode` / `decode_blocks`), uses geometric `Vec::reserve`
/// to keep whole-stream accumulation O(N).
#[cfg(target_arch = "x86_64")]
#[cold]
#[inline(never)]
fn grow_out(
    out: &mut Vec<u8>,
    start: usize,
    cur_len: usize,
    extra_need: usize,
    consumed: usize,
    rem_in: usize,
) -> (*mut u8, usize) {
    // SAFETY: caller has initialized `out[..cur_len]` (`cur_len <= out.capacity()`).
    unsafe { out.set_len(cur_len) };
    if start == 0 {
        let produced = cur_len;
        let est_rem = if consumed >= 1024 && produced > consumed {
            let raw = ((produced as u128 * rem_in as u128) / (consumed as u128)) as usize;
            raw.saturating_add(raw / 100).saturating_add(16 << 10)
        } else {
            rem_in.saturating_add(rem_in / 16).saturating_add(32 << 10)
        };
        let max_rem = rem_in.saturating_mul(52).saturating_add(320);
        let step = extra_need.max(est_rem.min(max_rem));
        out.reserve_exact(step);
    } else {
        out.reserve(extra_need);
    }
    (out.as_mut_ptr(), out.capacity())
}

/// 64-byte cache-line-aligned staging buffer for the 4 KiB L1d fused reverse+RLE1 pass,
/// eliminating 64-byte cache-line splits on 32-byte AVX2 loads and stores.
#[repr(C, align(64))]
struct AlignedBuf<const N: usize>([u8; N]);

/// Shared AVX2 decode + L2-hot CRC32 body over `rev_buf[..buf_len]` used by both full-cycle
/// and short-cycle fused post-walk functions inside a single `#[target_feature]` boundary
/// (zero per-segment / per-chunk `vzeroupper`).
#[cfg(target_arch = "x86_64")]
macro_rules! decode_rev_buf_avx2 {
    (
        $rev_buf:expr,
        $buf_len:expr,
        $is_eof:expr,
        $total_n:expr,
        $rem_after:expr,
        $start:expr,
        $out:expr,
        $out_p:expr,
        $out_len:expr,
        $out_cap:expr,
        $crc_state:expr,
        $crc_at:expr,
        $crc_tile:expr,
        $k:expr
    ) => {{
        let avail: usize = $buf_len;
        let is_eof: bool = $is_eof;
        let total_n: usize = $total_n;
        let rem_after: usize = $rem_after;
        let p = $rev_buf.as_ptr();
        let mut i = 0usize;
        // Hoist capacity invariant `out_len + (avail - i) + 32 <= out_cap` once per 4 KiB
        // chunk: every non-expanding 32B literal store and 1B tail write stays strictly in
        // bounds with zero per-iteration capacity branch.
        if $out_len + avail + 32 > $out_cap {
            let rem_in = rem_after + avail;
            let (np, nc) = grow_out(
                $out,
                $start,
                $out_len,
                avail + 32,
                total_n.saturating_sub(rem_in),
                rem_in,
            );
            $out_p = np;
            $out_cap = nc;
        }
        // Fast 2-load AVX2 loop: `i + 36 <= avail` guarantees `p[i..i + 36]` is inside
        // `rev_buf[..avail]`, covering loads at `i` and `i + 1`, boundary bytes `i + 32..35`,
        // and any run's 5th count byte at `j + 4 <= i + 35`.
        //
        // Bit `k` (`0..32`) of `m1` is 1 iff `p[i + k] == p[i + k + 1]`. Thus `tri = m1 &
        // (m1 >> 1) & (m1 >> 2)` has bit `k` set (`0..30`) iff `p[i + k..i + k + 4]` are all
        // equal, while any run at `k = 30` or `k = 31` requires bit 31 of `m1` (`p[i + 31] ==
        // p[i + 32]`). Testing `(tri | (m1 & (1 << 31))) == 0` cuts loads 4 -> 2 and compares
        // 3 -> 1 per 32 bytes with a single branch on the literal fast path.
        while i + 36 <= avail {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let m1 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, v1)) as u32;
            // Invariant gives `out_len + 32 <= out_len + (avail - i) < out_cap`: store all 32
            // bytes of `v0` straight from ymm0 (on a run hit, `tz` leading literals are final
            // and the trailing `32 - tz` slack bytes are immediately overwritten by the fill).
            _mm256_storeu_si256($out_p.add($out_len) as *mut __m256i, v0);
            let tri = m1 & (m1 >> 1) & (m1 >> 2);
            if (tri | (m1 & (1 << 31))) == 0 {
                $out_len += 32;
                i += 32;
                continue;
            }
            let tz = if tri != 0 {
                tri.trailing_zeros() as usize
            } else {
                let b32 = *p.add(i + 32);
                let b33 = *p.add(i + 33);
                if (m1 & (1 << 30)) != 0 && b32 == b33 {
                    30
                } else if b32 == b33 && b33 == *p.add(i + 34) {
                    31
                } else {
                    $out_len += 32;
                    i += 32;
                    continue;
                }
            };
            let j = i + tz;
            let c = *p.add(j);
            let extra = *p.add(j + 4) as usize;
            let run = 4 + extra;
            $out_len += tz;
            // `need` covers both `run.max(32)` for this store (since `run >= 4`, `run + 32 >=
            // 36`) and the restored invariant `(out_len + run) + (avail - (j + 5)) + 32`.
            let need = run + (avail - (j + 5)) + 32;
            if $out_len + need > $out_cap {
                let rem_in = rem_after + (avail - j);
                let (np, nc) = grow_out(
                    $out,
                    $start,
                    $out_len,
                    need,
                    total_n.saturating_sub(rem_in),
                    rem_in,
                );
                $out_p = np;
                $out_cap = nc;
            }
            if run <= 32 {
                let fill_v = _mm256_set1_epi8(c as i8);
                _mm256_storeu_si256($out_p.add($out_len) as *mut __m256i, fill_v);
                $out_len += run;
            } else {
                std::ptr::write_bytes($out_p.add($out_len), c, run);
                $out_len += run;
            }
            i = j + 5;
        }
        let limit = if is_eof {
            avail
        } else {
            avail.saturating_sub(4)
        };
        while i < limit {
            let c = *p.add(i);
            if i + 3 < avail && *p.add(i + 1) == c && *p.add(i + 2) == c && *p.add(i + 3) == c {
                if i + 4 >= avail {
                    $out.set_len($start);
                    return Err(crate::ErrorKind::RunUnfinished);
                }
                let run = 4 + (*p.add(i + 4) as usize);
                let need = run + (avail - (i + 5)) + 32;
                if $out_len + need > $out_cap {
                    let rem_in = rem_after + (avail - i);
                    let (np, nc) = grow_out(
                        $out,
                        $start,
                        $out_len,
                        need,
                        total_n.saturating_sub(rem_in),
                        rem_in,
                    );
                    $out_p = np;
                    $out_cap = nc;
                }
                std::ptr::write_bytes($out_p.add($out_len), c, run);
                $out_len += run;
                i += 5;
            } else {
                *$out_p.add($out_len) = c;
                $out_len += 1;
                i += 1;
            }
        }
        let rem = avail - i;
        if rem > 0 {
            $rev_buf.copy_within(avail - rem..avail, 0);
        }
        $buf_len = rem;
        let avail_out = $out_len - $crc_at;
        if avail_out >= $crc_tile {
            let take = avail_out & !63;
            $out.set_len($out_len);
            $crc_state = crate::crc::update_with($crc_state, &$out[$crc_at..$crc_at + take], $k);
            $crc_at += take;
        }
    }};
}

/// Single-`#[target_feature(enable = "avx2,bmi1")]` fused reverse-into-4KiB-L1d + RLE1 +
/// 64 KiB L2-hot CRC32 for full-cycle (`total == n`) segment chains.
///
/// # Safety
///
/// AVX2 and BMI1 must be available (`k.avx2 == true`). Every segment index in `chain` must be
/// `< bufs.len()`.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn fused_rev_rle_crc_buf_avx2<const BUF_CAP: usize, const CRC_TILE: usize, I: SegIdx>(
    bufs: &[Vec<u8>],
    chain: &[I],
    n: usize,
    start: usize,
    out: &mut Vec<u8>,
    k: crate::Kernels,
) -> Result<u32, crate::ErrorKind> {
    use core::arch::x86_64::*;
    let rev = _mm256_setr_epi8(
        15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6,
        5, 4, 3, 2, 1, 0,
    );
    let mut out_len = start;
    let mut out_cap = out.capacity();
    let mut out_p = out.as_mut_ptr();

    let mut rev_buf = AlignedBuf([0u8; BUF_CAP]);
    let mut buf_len = 0usize;
    let mut crc_state = !0u32;
    let mut crc_at = start;
    let mut rem_n = n;

    for &sg in chain {
        // SAFETY: caller guarantees every segment index in `chain` is `< bufs.len()`.
        let b = unsafe { bufs.get_unchecked(sg.as_usize()) };
        let seg_take = b.len().min(rem_n);
        rem_n -= seg_take;
        let mut pos = seg_take;
        while pos > 0 {
            let space = BUF_CAP - buf_len;
            let chunk = pos.min(space);
            pos -= chunk;
            // Inline AVX2 32B reverse copy into L1d `rev_buf` (zero `vzeroupper`).
            // SAFETY: `pos + chunk <= seg_take <= b.len()` and `buf_len + chunk <= BUF_CAP`,
            // with `b` and `rev_buf` disjoint.
            unsafe {
                let src = b.as_ptr().add(pos);
                let dst = rev_buf.0.as_mut_ptr().add(buf_len);
                let mut ri = 0usize;
                while ri + 32 <= chunk {
                    let v = _mm256_loadu_si256(src.add(ri) as *const __m256i);
                    let s = _mm256_shuffle_epi8(v, rev);
                    let r = _mm256_permute2x128_si256::<0x01>(s, s);
                    _mm256_storeu_si256(dst.add(chunk - ri - 32) as *mut __m256i, r);
                    ri += 32;
                }
                if ri < chunk {
                    reverse_copy_scalar(src.add(ri), dst, chunk - ri);
                }
            }
            buf_len += chunk;
            if buf_len >= BUF_CAP - 32 {
                let rem_after = rem_n + pos;
                let is_eof = rem_after == 0;
                decode_rev_buf_avx2!(
                    rev_buf.0, buf_len, is_eof, n, rem_after, start, out, out_p, out_len, out_cap,
                    crc_state, crc_at, CRC_TILE, k
                );
            }
        }
    }
    if buf_len > 0 {
        decode_rev_buf_avx2!(
            rev_buf.0, buf_len, true, n, 0, start, out, out_p, out_len, out_cap, crc_state, crc_at,
            CRC_TILE, k
        );
        debug_assert_eq!(buf_len, 0);
    }
    // SAFETY: `out[..out_len]` is initialized (`out[..start]` prior to call, `out[start..out_len]`
    // written sequentially above) and `out_len <= out.capacity()`.
    unsafe { out.set_len(out_len) };
    if crc_at < out_len {
        crc_state = crate::crc::update_with(crc_state, &out[crc_at..out_len], k);
    }
    Ok(!crc_state)
}

/// Single-`#[target_feature(enable = "avx2,bmi1")]` fused cycle-repeat + RLE1 + L2-hot CRC32
/// for short-cycle (`m < n`, e.g. `onebyte` or periodic blocks).
///
/// # Safety
///
/// AVX2 and BMI1 must be available (`k.avx2 == true`) and `cycle` must be non-empty.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn fused_rev_rle_crc_cycle_avx2<const BUF_CAP: usize, const CRC_TILE: usize>(
    cycle: &[u8],
    n: usize,
    start: usize,
    out: &mut Vec<u8>,
    k: crate::Kernels,
) -> Result<u32, crate::ErrorKind> {
    use core::arch::x86_64::*;
    let m = cycle.len();
    let mut out_len = start;
    let mut out_cap = out.capacity();
    let mut out_p = out.as_mut_ptr();

    let mut rev_buf = AlignedBuf([0u8; BUF_CAP]);
    let mut buf_len = 0usize;
    let mut crc_state = !0u32;
    let mut crc_at = start;
    let mut rem_n = n;
    let mut cyc_pos = 0usize;

    while rem_n > 0 {
        while buf_len < BUF_CAP && rem_n > 0 {
            let take = (m - cyc_pos).min(BUF_CAP - buf_len).min(rem_n);
            rev_buf.0[buf_len..buf_len + take].copy_from_slice(&cycle[cyc_pos..cyc_pos + take]);
            buf_len += take;
            rem_n -= take;
            cyc_pos += take;
            if cyc_pos == m {
                cyc_pos = 0;
            }
        }
        let is_eof = rem_n == 0;
        decode_rev_buf_avx2!(
            rev_buf.0, buf_len, is_eof, n, rem_n, start, out, out_p, out_len, out_cap, crc_state,
            crc_at, CRC_TILE, k
        );
    }
    if buf_len > 0 {
        decode_rev_buf_avx2!(
            rev_buf.0, buf_len, true, n, 0, start, out, out_p, out_len, out_cap, crc_state, crc_at,
            CRC_TILE, k
        );
        debug_assert_eq!(buf_len, 0);
    }
    // SAFETY: `out[..out_len]` is initialized and `out_len <= out.capacity()`.
    unsafe { out.set_len(out_len) };
    if crc_at < out_len {
        crc_state = crate::crc::update_with(crc_state, &out[crc_at..out_len], k);
    }
    Ok(!crc_state)
}

/// Portable scalar fused reverse-into-4KiB-L1d + RLE1 + 64 KiB CRC32 for full-cycle chains.
fn fused_rev_rle_crc_buf_scalar<const BUF_CAP: usize, const CRC_TILE: usize, I: SegIdx>(
    bufs: &[Vec<u8>],
    chain: &[I],
    n: usize,
    start: usize,
    out: &mut Vec<u8>,
    k: crate::Kernels,
) -> Result<u32, crate::ErrorKind> {
    let mut rev_buf = [0u8; BUF_CAP];
    let mut buf_len = 0usize;
    let mut crc_state = !0u32;
    let mut crc_at = start;
    let mut rem_n = n;

    let flush_crc = |crc_state: &mut u32, crc_at: &mut usize, out: &[u8]| {
        let avail_out = out.len() - *crc_at;
        if avail_out >= CRC_TILE {
            let take = avail_out & !63;
            *crc_state = crate::crc::update_with(*crc_state, &out[*crc_at..*crc_at + take], k);
            *crc_at += take;
        }
    };

    for &sg in chain {
        let b = &bufs[sg.as_usize()];
        let seg_take = b.len().min(rem_n);
        rem_n -= seg_take;
        let mut pos = seg_take;
        while pos > 0 {
            let space = BUF_CAP - buf_len;
            let chunk = pos.min(space);
            pos -= chunk;
            // SAFETY: `pos + chunk <= seg_take <= b.len()`, `buf_len + chunk <= BUF_CAP`,
            // and `b` vs `rev_buf` are disjoint.
            unsafe {
                reverse_copy_scalar(
                    b.as_ptr().add(pos),
                    rev_buf.as_mut_ptr().add(buf_len),
                    chunk,
                );
            }
            buf_len += chunk;
            if buf_len >= BUF_CAP - 32 {
                let is_eof = rem_n == 0 && pos == 0;
                let rem = crate::rle::decode_chunk_scalar(&rev_buf, buf_len, is_eof, out)?;
                if rem > 0 {
                    rev_buf.copy_within(buf_len - rem..buf_len, 0);
                }
                buf_len = rem;
                flush_crc(&mut crc_state, &mut crc_at, out);
            }
        }
    }
    if buf_len > 0 {
        let rem = crate::rle::decode_chunk_scalar(&rev_buf, buf_len, true, out)?;
        debug_assert_eq!(rem, 0);
    }
    if crc_at < out.len() {
        crc_state = crate::crc::update_with(crc_state, &out[crc_at..], k);
    }
    Ok(!crc_state)
}

/// Portable scalar fused cycle-repeat + RLE1 + 64 KiB CRC32 for short cycles (`m < n`).
fn fused_rev_rle_crc_cycle_scalar<const BUF_CAP: usize, const CRC_TILE: usize>(
    cycle: &[u8],
    n: usize,
    start: usize,
    out: &mut Vec<u8>,
    k: crate::Kernels,
) -> Result<u32, crate::ErrorKind> {
    let m = cycle.len();
    let mut rev_buf = [0u8; BUF_CAP];
    let mut buf_len = 0usize;
    let mut crc_state = !0u32;
    let mut crc_at = start;
    let mut rem_n = n;
    let mut cyc_pos = 0usize;

    while rem_n > 0 {
        while buf_len < BUF_CAP && rem_n > 0 {
            let take = (m - cyc_pos).min(BUF_CAP - buf_len).min(rem_n);
            rev_buf[buf_len..buf_len + take].copy_from_slice(&cycle[cyc_pos..cyc_pos + take]);
            buf_len += take;
            rem_n -= take;
            cyc_pos += take;
            if cyc_pos == m {
                cyc_pos = 0;
            }
        }
        let is_eof = rem_n == 0;
        let rem = crate::rle::decode_chunk_scalar(&rev_buf, buf_len, is_eof, out)?;
        if rem > 0 {
            rev_buf.copy_within(buf_len - rem..buf_len, 0);
        }
        buf_len = rem;
        let avail_out = out.len() - crc_at;
        if avail_out >= CRC_TILE {
            let take = avail_out & !63;
            crc_state = crate::crc::update_with(crc_state, &out[crc_at..crc_at + take], k);
            crc_at += take;
        }
    }
    if buf_len > 0 {
        let rem = crate::rle::decode_chunk_scalar(&rev_buf, buf_len, true, out)?;
        debug_assert_eq!(rem, 0);
    }
    if crc_at < out.len() {
        crc_state = crate::crc::update_with(crc_state, &out[crc_at..], k);
    }
    Ok(!crc_state)
}

/// Fused plain backward LF walk + RLE1 + CRC32 for tiny blocks (`n <= SMALL_FUSED_MAX`, e.g.
/// Wikidata's 198 non-Open runts): walks into a 4 KiB stack buffer with zero heap scratch,
/// appends RLE1-decoded bytes to `out`, and returns the CRC32 of `&out[start..]`.
fn walk_back_fused_small(
    tt: &TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    k: crate::Kernels,
) -> Result<u32, crate::ErrorKind> {
    let t = &tt.0;
    let n = t.len();
    let start = out.len();
    if n == 0 {
        return Ok(crate::crc::crc32(&[], k));
    }
    let mut tmp = [0u8; SMALL_FUSED_MAX];
    let buf = &mut tmp[..n];
    let mut i = orig_ptr as usize;
    let mut m = n;
    for step in 0..n {
        let e = t[i];
        buf[n - 1 - step] = e as u8;
        i = (e >> 8) as usize;
        if i == orig_ptr as usize {
            m = step + 1;
            break;
        }
    }
    repeat_cycle(buf, m);
    if let Err(kind) = crate::rle::unrle(buf, out, k) {
        out.truncate(start);
        return Err(kind);
    }
    Ok(crate::crc::crc32(&out[start..], k))
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
    room_segments::<FLAGGED_LANES, PREFETCH, true>(tt, orig_ptr, s, FLAGGED_SEGMENTS, true);
    s.assemble_reversed(out);
}

/// `walk_back_room` with an explicit `Kernels` dispatch token for reverse assembly.
pub(crate) fn walk_back_room_with_kernels(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
    k: crate::Kernels,
) {
    if tt.len() < MIN_SEGMENTED {
        return walk_back(tt, orig_ptr, out);
    }
    room_segments::<FLAGGED_LANES, PREFETCH, true>(tt, orig_ptr, s, FLAGGED_SEGMENTS, true);
    s.assemble_reversed_with(out, k);
}

/// Private `Segmented` backward room walk: skips redundant `START`-bit cleanup on `Segmented`'s
/// private `tt` (which is always overwritten by the next block's build).
fn walk_back_room_private(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
    k: crate::Kernels,
) {
    if tt.len() < MIN_SEGMENTED {
        return walk_back(tt, orig_ptr, out);
    }
    room_segments::<FLAGGED_LANES, PREFETCH, true>(tt, orig_ptr, s, FLAGGED_SEGMENTS, false);
    s.assemble_reversed_with(out, k);
}

/// Private `Segmented` forward room walk (forged-counts fallback): skips redundant `START`-bit
/// cleanup on `Segmented`'s private `tt`.
fn walk_segments_room_private(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
) {
    if tt.len() < MIN_SEGMENTED {
        return walk(tt, orig_ptr, out);
    }
    room_segments::<FLAGGED_LANES, PREFETCH, false>(tt, orig_ptr, s, FLAGGED_SEGMENTS, false);
    s.assemble(out);
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
    room_segments::<FLAGGED_LANES, PREFETCH, true>(tt, orig_ptr, s, FLAGGED_SEGMENTS, true);
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
    room_segments::<L, PREFETCH, true>(tt, orig_ptr, s, segments, true);
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
    room_segments::<FLAGGED_LANES, PREFETCH, false>(tt, orig_ptr, s, FLAGGED_SEGMENTS, true);
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
    room_segments::<L, PF, false>(tt, orig_ptr, s, segments, true);
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
    clear_start: bool,
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
    if clear_start {
        for &r in starts {
            tt.0[r as usize] &= LINK;
        }
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

    /// `build_lf` + `walk_back_room`, `Segmented::invert`/`invert_in_place`, and vectorized
    /// `reverse_copy` match the reference paths across length boundaries, 8-byte equal runs,
    /// periodic cycles (`repeat_cycle`), and forged counts (fallback to `build`).
    #[test]
    fn miri_build_lf_and_reverse_copy() {
        // 1. Reverse copy parity across every length 0..=135 (32-byte, 8-byte, 1-byte edges)
        for len in 0..=135usize {
            let src: Vec<u8> = (0..len).map(|i| ((i * 37 + 13) & 0xff) as u8).collect();
            let want: Vec<u8> = src.iter().rev().copied().collect();
            let mut got_scalar = vec![0xaa; len];
            // SAFETY: non-overlapping `len`-byte slices.
            unsafe { reverse_copy_scalar(src.as_ptr(), got_scalar.as_mut_ptr(), len) };
            assert_eq!(got_scalar, want, "reverse_copy_scalar len={len}");
            #[cfg(target_arch = "x86_64")]
            if std::is_x86_feature_detected!("avx2") {
                let mut got_avx2 = vec![0x55; len];
                // SAFETY: AVX2 detected; non-overlapping `len`-byte slices.
                unsafe { reverse_copy_avx2(src.as_ptr(), got_avx2.as_mut_ptr(), len) };
                assert_eq!(got_avx2, want, "reverse_copy_avx2 len={len}");
            }
        }

        // 2. build_lf vs plain build+walk on short, run-heavy, random, periodic, and segmented blocks
        let mut x = 0x517c_c1b7_2722_0a95u64;
        let mut cases: Vec<Vec<u8>> = Vec::new();
        for n in [0, 1, 2, 7, 8, 9, 15, 16, 17, 31, 32, 33, 64, 129] {
            let v: Vec<u8> = (0..n)
                .map(|_| {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    (x & 0xff) as u8
                })
                .collect();
            cases.push(v);
        }
        // Equal-byte runs hitting the 8-byte fast path at aligned and unaligned offsets
        cases.push(vec![b'a'; 64]);
        cases.push(
            [
                b"xy".as_slice(),
                &[b'z'; 24],
                b"q",
                &[b'w'; 16],
                b"1234567",
                &[b'm'; 8],
            ]
            .concat(),
        );
        // Periodic block exercising `repeat_cycle` in `assemble_reversed_with`
        let seg_n = if cfg!(miri) {
            600
        } else {
            MIN_SEGMENTED + 1024
        };
        cases.push((0..seg_n).map(|i| b"abc"[i % 3]).collect());
        cases.push(
            (0..seg_n)
                .map(|_| {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    (x % 19) as u8
                })
                .collect(),
        );

        let mut plain_k = Plain::default();
        let mut seg_k = Segmented::default();
        for ll in &cases {
            let mut counts = [0u32; 256];
            for &c in ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            assert!(build_lf(ll, &counts, &mut tt));
            assert_eq!(tt.len(), ll.len());
            if ll.is_empty() {
                continue;
            }
            for orig_ptr in [0, ll.len() as u32 / 2, ll.len() as u32 - 1] {
                let (mut want, mut got) = (Vec::new(), Vec::new());
                plain_k.invert(ll, &counts, orig_ptr, &mut want);
                seg_k.invert(ll, &counts, orig_ptr, &mut got);
                assert_eq!(got, want, "Segmented::invert len={}", ll.len());

                let mut in_place = ll.clone();
                assert!(seg_k.invert_in_place(&mut in_place, &counts, orig_ptr));
                assert_eq!(
                    in_place,
                    want,
                    "Segmented::invert_in_place len={}",
                    ll.len()
                );

                // Verify assemble_reversed_with(scalar) == assemble_reversed_with(detect)
                let mut tt_lf = TVector::new();
                assert!(build_lf(ll, &counts, &mut tt_lf));
                let mut s = SegScratch::default();
                room_segments::<FLAGGED_LANES, PREFETCH, true>(
                    &mut tt_lf, orig_ptr, &mut s, 32, true,
                );
                let (mut a_scalar, mut a_detect) = (Vec::new(), Vec::new());
                s.assemble_reversed_with(&mut a_scalar, crate::Kernels::scalar());
                assert_eq!(a_scalar, want, "assemble_reversed scalar len={}", ll.len());
                s.assemble_reversed_with(&mut a_detect, crate::Kernels::detect());
                assert_eq!(a_detect, want, "assemble_reversed detect len={}", ll.len());
            }
        }

        // 3. Forged/mismatched counts are rejected by build_lf (tt left empty) and safely handled
        let ll = b"abracadabraaaaaaaaa!".to_vec();
        let mut counts = [0u32; 256];
        for &c in &ll {
            counts[c as usize] += 1;
        }
        let mut bad_sum = counts;
        bad_sum[b'a' as usize] += 1;
        let mut tt = TVector(vec![1, 2, 3]);
        assert!(!build_lf(&ll, &bad_sum, &mut tt));
        assert!(tt.is_empty());

        let mut bad_bucket = counts;
        bad_bucket[b'a' as usize] -= 1;
        bad_bucket[b'b' as usize] += 1;
        let mut tt2 = TVector(vec![1, 2, 3]);
        assert!(!build_lf(&ll, &bad_bucket, &mut tt2));
        assert!(tt2.is_empty());

        // Segmented falls back cleanly when counts don't match ll (exact same output as Plain)
        let (mut want, mut got) = (Vec::new(), Vec::new());
        plain_k.invert(&ll, &bad_bucket, 3, &mut want);
        seg_k.invert(&ll, &bad_bucket, 3, &mut got);
        assert_eq!(got, want);
        let mut in_place = ll.clone();
        assert!(seg_k.invert_in_place(&mut in_place, &bad_bucket, 3));
        assert_eq!(in_place, want);
    }

    /// Build a synthetic `SegScratch` whose backward-walked segments reverse-join to `forward`
    /// (split into arbitrary segment sizes `splits`), so tests can place RLE1 runs and
    /// `RunUnfinished` errors at exact segment and 4 KiB buffer boundaries.
    fn synthetic_scratch(forward: &[u8], splits: &[usize], cycle_repeat: usize) -> SegScratch {
        let mut bufs = Vec::new();
        let mut pos = 0usize;
        let mut si = 0usize;
        while pos < forward.len() {
            let take = splits[si % splits.len()].min(forward.len() - pos);
            let mut seg = forward[pos..pos + take].to_vec();
            seg.reverse();
            bufs.push(seg);
            pos += take;
            si += 1;
        }
        let ns = bufs.len();
        // Backward walk order is the reverse of forward order: `ns - 1 -> ns - 2 -> ... -> 0`.
        let next: Vec<usize> = (0..ns)
            .map(|i| if i == 0 { ns - 1 } else { i - 1 })
            .collect();
        SegScratch {
            starts: (0..ns as u32).collect(),
            bitmap: Vec::new(),
            bufs,
            next,
            n: forward.len() * cycle_repeat,
            first: ns - 1,
        }
    }

    /// Verify `assemble_reversed_rle_crc` (both scalar and detected AVX2/CLMUL kernels)
    /// against `assemble_reversed_with` + `rle::unrle` + `crc::crc32` on boundary-straddling
    /// runs, 4 KiB buffer wraps, `RunUnfinished`, short cycles, and `start > 0` append rollback.
    #[test]
    fn miri_fused_postwalk_edges_and_parity() {
        let kernels = {
            let mut v = vec![crate::Kernels::scalar()];
            if crate::Kernels::detect() != crate::Kernels::scalar() {
                v.push(crate::Kernels::detect());
            }
            v
        };

        let mut cases: Vec<(String, Vec<u8>, Vec<usize>, usize)> = Vec::new();

        // 1. Every run length extra in {0,1,2,27,28,29,32,33,255} straddling 4 KiB buffer edge
        //    (4092..4099) AND straddling tiny segment boundaries (3, 5, 7, 31, 32, 33 bytes).
        let extras: &[u8] = if cfg!(miri) {
            &[0, 1, 28, 255]
        } else {
            &[0, 1, 2, 27, 28, 29, 32, 33, 255]
        };
        let offsets: &[usize] = if cfg!(miri) {
            &[0, 3, 30, 31, 4094]
        } else {
            &[0, 1, 2, 3, 4, 29, 30, 31, 32, 4092, 4093, 4094, 4095, 4096]
        };
        for &extra in extras {
            for &offset in offsets {
                let mut v: Vec<u8> = (0..offset).map(|i| b'a' + (i % 23) as u8).collect();
                v.extend_from_slice(&[b'#', b'#', b'#', b'#', extra]);
                v.extend((0..70).map(|i| b'A' + (i % 19) as u8));
                cases.push((
                    format!("run_extra{extra}_off{offset}"),
                    v,
                    vec![3, 5, 7, 31, 32, 33, 127],
                    1,
                ));
            }
        }
        // Boundary pair/triple false-positives for the 2-load `m1 & (1 << 31)` check (`p[31] == p[32]`)
        for (tag, tail) in [
            ("pair31", &b"xxYZ"[..]),
            ("triple30", &b"xxxY"[..]),
            ("triple31", &b"Yxxx"[..]),
        ] {
            let mut v: Vec<u8> = (0..30).map(|i| b'a' + (i % 23) as u8).collect();
            v.extend_from_slice(tail);
            v.extend((0..40).map(|i| b'A' + (i % 19) as u8));
            cases.push((format!("m1_edge_{tag}"), v, vec![64], 1));
        }

        // 2. `RunUnfinished` at exact segment boundaries, 4 KiB boundary, and short-cycle EOF.
        let pads: &[usize] = if cfg!(miri) {
            &[0, 3, 4094]
        } else {
            &[0, 1, 31, 32, 33, 4092, 4093, 4094, 4095, 4096]
        };
        for &pad in pads {
            let mut v: Vec<u8> = (0..pad).map(|i| b'a' + (i % 17) as u8).collect();
            v.extend_from_slice(b"zzzz");
            cases.push((format!("unfinished_pad{pad}"), v, vec![2, 3, 5, 19], 1));
        }

        // 3. Short cycles (including expansive `onebyte`-style 5-byte run cycle, 2-byte
        //    periodic `ab`, and a short cycle ending in `RunUnfinished`).
        let cyc_reps = if cfg!(miri) { 12 } else { 2000 };
        cases.push((
            "short_cycle_onebyte".into(),
            b"\0\0\0\0\xff".to_vec(),
            vec![2, 3],
            cyc_reps,
        ));
        cases.push((
            "short_cycle_ab".into(),
            b"ab".to_vec(),
            vec![1, 1],
            cyc_reps,
        ));
        cases.push((
            "short_cycle_runs".into(),
            b"aaaa\x03bcdddd\x00".to_vec(),
            vec![3, 4, 5],
            cyc_reps,
        ));
        cases.push((
            "short_cycle_unfinished".into(),
            b"abzzzz".to_vec(),
            vec![2, 4],
            1,
        ));

        // 4. Multi-tile block (> 64 KiB CRC tile) with dense runs + literals (skipped under Miri
        //    to keep Miri runtime fast; exercised natively in `cargo test`).
        if !cfg!(miri) {
            let mut dense = Vec::with_capacity(90_000);
            let mut rng = 0x55aa_1234_9876_fedcu64;
            while dense.len() < 85_000 {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                if rng % 11 == 0 {
                    let c = (rng % 8) as u8;
                    let extra = [0u8, 1, 2, 15, 28, 29, 64, 255][(rng >> 8) as usize % 8];
                    dense.extend_from_slice(&[c, c, c, c, extra]);
                } else {
                    dense.push((rng & 0xff) as u8);
                }
            }
            cases.push((
                "multi_tile_dense".into(),
                dense,
                vec![17, 31, 32, 33, 64, 1024, 4095, 4096, 4097],
                1,
            ));
        }

        for (label, forward, splits, reps) in &cases {
            let s = synthetic_scratch(forward, splits, *reps);
            let mut raw = Vec::new();
            s.assemble_reversed_with(&mut raw, crate::Kernels::scalar());

            for &k in &kernels {
                let mut want = Vec::new();
                let want_res =
                    crate::rle::unrle(&raw, &mut want, k).map(|()| crate::crc::crc32(&want, k));

                // Fresh buffer (`start == 0`)
                let mut got = Vec::new();
                let got_res = s.assemble_reversed_rle_crc(&mut got, k);
                assert_eq!(got_res, want_res, "{label} fresh {k:?}");
                if want_res.is_ok() {
                    assert_eq!(got, want, "{label} fresh bytes {k:?}");
                } else {
                    assert!(got.is_empty(), "{label} fresh error rollback {k:?}");
                }

                // Append mode (`start > 0`) verifying prefix preservation on both Ok and Err
                let prefix = b"PREFIX_BYTES_12345";
                let mut got_app = prefix.to_vec();
                let got_app_res = s.assemble_reversed_rle_crc(&mut got_app, k);
                assert_eq!(got_app_res, want_res, "{label} append {k:?}");
                assert_eq!(
                    &got_app[..prefix.len()],
                    prefix,
                    "{label} prefix intact {k:?}"
                );
                if want_res.is_ok() {
                    assert_eq!(
                        &got_app[prefix.len()..],
                        &want[..],
                        "{label} append bytes {k:?}"
                    );
                } else {
                    assert_eq!(
                        got_app.len(),
                        prefix.len(),
                        "{label} append error rollback {k:?}"
                    );
                }
            }
        }

        // 5. `FusedPostwalk` RAII scope + panic-safety & `walk_back_fused_small` parity.
        let small_ll = b"baaabbbb\x02cccddddd\x00!".to_vec();
        let mut counts = [0u32; 256];
        for &c in &small_ll {
            counts[c as usize] += 1;
        }
        let mut seg = Segmented::default();
        let mut plain = Plain::default();
        for orig_ptr in [0u32, 5, small_ll.len() as u32 - 1] {
            let mut raw = Vec::new();
            plain.invert(&small_ll, &counts, orig_ptr, &mut raw);
            for &k in &kernels {
                let mut want = Vec::new();
                let want_res =
                    crate::rle::unrle(&raw, &mut want, k).map(|()| crate::crc::crc32(&want, k));
                let mut ll_copy = small_ll.clone();
                let mut out = b"HEAD".to_vec();
                let guard = FusedPostwalk::enter_append(&mut out, k);
                assert!(seg.invert_in_place(&mut ll_copy, &counts, orig_ptr));
                let got_res = guard.take().expect("fused small path must engage");
                assert_eq!(got_res, want_res, "small fused orig_ptr={orig_ptr} {k:?}");
                assert_eq!(&out[..4], b"HEAD");
                if want_res.is_ok() {
                    assert_eq!(&out[4..], &want[..]);
                } else {
                    assert_eq!(out.len(), 4);
                }
            }
        }

        // Unconsumed `FusedPostwalk` drop restores Idle cleanly (including on unwind).
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _g = FusedPostwalk::enter_owned(crate::Kernels::scalar());
            panic!("simulated unwind");
        }));
        let mut raw_after = Vec::new();
        seg.invert(&small_ll, &counts, 0, &mut raw_after);
        let mut plain_after = Vec::new();
        plain.invert(&small_ll, &counts, 0, &mut plain_after);
        assert_eq!(raw_after, plain_after);
    }
}
