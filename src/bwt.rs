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

/// An inverse-BWT kernel: a block's RLE1-coded text from its last column `ll` (whose
/// bytes `counts` counts) and its primary index (below `ll.len()`: the caller checks),
/// into `out` (cleared). Its buffers are its own, kept from block to block.
pub trait InverseBwt: Default + Send {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>);

    /// Its buffers' sizes, for the footprint ledger (src/trace.rs): tests and the `trace`
    /// feature only
    #[cfg(any(test, feature = "trace"))]
    fn buffers(&self) -> Vec<crate::trace::Buffer> {
        Vec::new()
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

/// A block's T-vector. Built only by `build`, so every link in it is below its length,
/// which the walks rely on to read it unchecked.
#[derive(Default)]
pub struct TVector(Vec<u32>);

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

/// The T-vector of the last column `ll`, whose bytes `counts` counts
pub fn build(ll: &[u8], counts: &[u32; 256], tt: &mut TVector) {
    let tt = &mut tt.0;
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
fn read_u64(b: &[u8], i: usize) -> u64 {
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
const FLAGGED_LANES: usize = 32;
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
pub(crate) const MIN_SEGMENTED: usize = 1 << 16;

/// The segment walk's buffers, kept from block to block
#[derive(Default)]
pub struct SegScratch {
    starts: Vec<u32>,
    bitmap: Vec<u64>,
    bufs: Vec<Vec<u8>>,
    next: Vec<usize>,
    /// the room walk's last block: its length and the segment its output starts in
    n: usize,
    first: usize,
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
const START: u32 = 1 << 31;
const LINK: u32 = !START;

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
    walk_segments_flagged_with::<FLAGGED_LANES, PREFETCH>(tt, orig_ptr, out, s, FLAGGED_SEGMENTS)
}

/// `walk_segments_flagged` with `L` lanes and `segments` segments, each lane's next entry
/// prefetched as soon as its row is known if `PF`: each configuration exactly `walk`'s
/// output, so they can be measured side by side. It segments a block of any length (the
/// tests walk small ones; `walk_segments_flagged` leaves those below MIN_SEGMENTED to `walk`).
pub fn walk_segments_flagged_with<const L: usize, const PF: u8>(
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
    room_segments::<FLAGGED_LANES, PREFETCH>(tt, orig_ptr, s, FLAGGED_SEGMENTS);
    true
}

/// `walk_segments_room` with `L` lanes and `segments` segments, on a block of any length
/// (the tests walk small ones; `walk_segments_room` leaves those below MIN_SEGMENTED to
/// `walk`).
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
pub fn walk_segments_room_with<const L: usize, const PF: u8>(
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
    room_segments::<L, PF>(tt, orig_ptr, s, segments);
    s.assemble(out);
}

/// The room walk itself, on a block of one row or more: each segment's bytes into its
/// buffer in `s`, its successor in `s.next`, and the block's length and first segment
/// for `SegScratch::assemble`. Never inlined, so the decoder and the stage table's split
/// timing (`walk_segments_room_parts`) run the same machine code for the walk
#[inline(never)]
fn room_segments<const L: usize, const PF: u8>(
    tt: &mut TVector,
    orig_ptr: u32,
    s: &mut SegScratch,
    segments: usize,
) {
    let n = tt.0.len();
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

    let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
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
    let finish = |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
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
            let mut base: [*mut u8; L] = std::array::from_fn(|k| lanes[k].p);
            for off in 0..room {
                for k in 0..L {
                    let l = &mut lanes[k];
                    // SAFETY: rows are below n (TVector), with the flag masked off below
                    let q = unsafe { *t.add(l.row as usize) };
                    if q & START != 0 {
                        l.p = base[k].wrapping_add(off);
                        finish(l, bufs, next);
                        if queued == ns {
                            // the lanes before this one wrote at `off`, those after it not yet
                            for (j, lj) in lanes.iter_mut().enumerate() {
                                if j < k {
                                    lj.p = base[j].wrapping_add(off + 1);
                                } else if j > k {
                                    lj.p = base[j].wrapping_add(off);
                                }
                            }
                            transition!(FULL_PHASE_ENDS_MID_PASS);
                            break 'full;
                        }
                        transition!(MID_PASS_TAKES);
                        *l = take(queued, bufs);
                        if PF != PF_OFF {
                            prefetch::<PF>(t, l.row);
                        }
                        queued += 1;
                        // its next byte, at offset off + 1, goes at its p
                        base[k] = l.p.wrapping_sub(off + 1);
                        continue;
                    }
                    l.row = q >> 8;
                    if PF != PF_OFF {
                        prefetch::<PF>(t, l.row);
                    }
                    // SAFETY: base + off is below the lane's end: room is at most its spare
                    // at the pass's start (or, for a buffer taken mid-pass, its capacity
                    // after its first byte)
                    debug_assert!(base[k].wrapping_add(off) < l.end);
                    unsafe { *base[k].wrapping_add(off) = q as u8 };
                }
            }
            for (l, b) in lanes.iter_mut().zip(base) {
                l.p = b.wrapping_add(room);
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

/// A hint that the T-vector entry at `row` (below its length) is read soon: a software
/// prefetch at the level `H` names (`PF_L1`, `PF_L2`, `PF_NTA`) where the target has one,
/// else nothing. A hint only: it changes no value read.
#[inline(always)]
fn prefetch<const H: u8>(t: *const u32, row: u32) {
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
fn random_rows(n: usize, m: usize, rows: &mut Vec<u32>) {
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
mod tests {
    use super::*;

    /// The segments' reference, safe and slow: from each start, the walks' own steps (its
    /// first byte from its own entry, then a row's byte at a time until the next start),
    /// giving each segment's bytes and its successor. Every segment, on every cycle of the
    /// list, not only those the output joins: a walk writes them all.
    fn segment_oracle(tt: &TVector, starts: &[u32]) -> Vec<(Vec<u8>, usize)> {
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
                walk_segments_room_with::<32, PF_L1>(
                    &mut tt,
                    orig_ptr,
                    &mut flagged,
                    &mut seg,
                    segments,
                );
                assert!(plain == flagged, "room walk, {segments} segments");
                walk_segments_room_with::<5, PF_OFF>(
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
                walk_segments_flagged_with::<12, PF_OFF>(
                    &mut tt,
                    orig_ptr,
                    &mut flagged,
                    &mut seg,
                    segments,
                );
                assert!(plain == flagged, "12 lanes, {segments} segments");
                walk_segments_flagged_with::<7, PF_L1>(
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
                    walk_segments_room_with::<32, PF_L1>(
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
        for case in 0..200 {
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
                    walk_segments_room_with::<32, PF_L1>(
                        &mut tt, orig_ptr, &mut out, &mut seg, segments,
                    );
                    assert!(plain == out, "room walk, {what}");
                    check_segments(&tt, &seg, &what);
                    walk_segments_room_with::<5, PF_OFF>(
                        &mut tt, orig_ptr, &mut out, &mut seg, segments,
                    );
                    assert!(plain == out, "room walk, 5 lanes, {what}");
                    check_segments(&tt, &seg, &what);
                    walk_segments_flagged_with::<32, PF_L1>(
                        &mut tt, orig_ptr, &mut out, &mut seg, segments,
                    );
                    assert!(plain == out, "flagged walk, {what}");
                    check_segments(&tt, &seg, &what);
                    walk_segments_flagged_with::<7, PF_OFF>(
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
            walk_segments_room_with::<32, PF_L1>(&mut tt, orig_ptr, &mut out, &mut seg, 256);
            assert!(plain == out, "room walk, seed {seed}");
            check_segments(&tt, &seg, "room walk");
            walk_segments_flagged_with::<32, PF_L1>(&mut tt, orig_ptr, &mut out, &mut seg, 256);
            assert!(plain == out, "flagged walk, seed {seed}");
            check_segments(&tt, &seg, "flagged walk");
        }
        assert_reached();
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
                walk_segments_room_with::<32, PF_L1>(&mut tt, 1, &mut out, &mut seg, segments);
                assert!(plain == out, "room walk, {segments} segments");
                check_segments(&tt, &seg, "room walk");
                walk_segments_flagged_with::<32, PF_L1>(&mut tt, 1, &mut out, &mut seg, segments);
                assert!(plain == out, "flagged walk, {segments} segments");
                check_segments(&tt, &seg, "flagged walk");
            }
        }
    }
}
