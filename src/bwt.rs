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
}

/// The segmented walk (`walk_segments`), the plain one for a short block: the default
#[derive(Default)]
pub struct Segmented {
    tt: TVector,
    seg: SegScratch,
}

impl InverseBwt for Segmented {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        build(ll, counts, &mut self.tt);
        walk_segments_flagged(&mut self.tt, orig_ptr, out, &mut self.seg);
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
    for (i, &c) in ll.iter().enumerate() {
        let k = ftab[c as usize] as usize;
        // (k < len, as the counts are ll's own: indexed checked all the same, and the
        // link assigned, not added, so each link is some i < len whatever happened)
        tt[k] = (tt[k] & 0xff) | (i as u32) << 8;
        ftab[c as usize] += 1;
    }
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
const FLAGGED_SEGMENTS: usize = 512;
const PREFETCH: bool = true;
/// Blocks shorter than this are walked plainly (the segments' bookkeeping would cost more
/// than the latency they hide; such blocks are only a stream's last)
const MIN_SEGMENTED: usize = 1 << 16;

/// The segment walk's buffers, kept from block to block
#[derive(Default)]
pub struct SegScratch {
    starts: Vec<u32>,
    bitmap: Vec<u64>,
    bufs: Vec<Vec<u8>>,
    next: Vec<usize>,
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
    walk_segments_flagged_with::<FLAGGED_LANES, PREFETCH>(tt, orig_ptr, out, s, FLAGGED_SEGMENTS)
}

/// `walk_segments_flagged` with `L` lanes and `segments` segments, each lane's next entry
/// prefetched as soon as its row is known if `PF`: each configuration exactly `walk`'s
/// output, so they can be measured side by side
pub fn walk_segments_flagged_with<const L: usize, const PF: bool>(
    tt: &mut TVector,
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
    segments: usize,
) {
    let n = tt.0.len();
    out.clear();
    if n < MIN_SEGMENTED {
        return walk(tt, orig_ptr, out);
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
        if PF {
            prefetch(t, lanes[active].row);
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
                    if PF {
                        prefetch(t, l.row);
                    }
                    queued += 1;
                    continue;
                }
                l.row = q >> 8;
                if PF {
                    prefetch(t, l.row);
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
            if PF {
                prefetch(t, l.row);
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

/// The walk with Precopy (#9; Kärkkäinen, Kempa and Puglisi 2012): the flagged segmented
/// walk, where a lane that reaches row r just after row r - 1 was walked copies the text
/// that followed r - 1 instead of walking, for as long as the two walks stay one row apart.
///
/// Within a bucket of the first column, rows c and c + 1 step to rows LF(c) and
/// LF(c) + 1 exactly when the last column holds equal bytes at LF(c) and LF(c) + 1. So
/// from a source walk's rows s_0, s_1, ... (recorded as it goes) the walk from s_0 + 1
/// is s_t + 1 for as long as s_t and s_t + 1 share a bucket and the last column's bytes
/// at s_t and s_t + 1 are equal: two bitmaps of the block's rows, read sequentially along
/// the source, with no load from the T-vector. Each walked row's entry is overwritten
/// with where its byte went (its segment and offset), as no walk reads it again: the
/// entry beside a lane's next is usually on the same cache line, so finding a source is
/// all but free. Copied rows are left unmarked, so the copy's last row can still be
/// stepped from. `tt` is consumed (the next `build` rebuilds it). Exactly `walk`'s output.
#[derive(Default)]
pub struct Precopy {
    tt: TVector,
    seg: SegScratch,
    pc: PrecopyScratch,
}

impl InverseBwt for Precopy {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        build(ll, counts, &mut self.tt);
        walk_precopy(
            &mut self.tt,
            ll,
            counts,
            orig_ptr,
            out,
            &mut self.seg,
            &mut self.pc,
        );
    }
}

/// Precopy's buffers, kept from block to block: each segment's rows (beside its bytes in
/// SegScratch), and the bitmaps of the block's rows
#[derive(Default)]
pub struct PrecopyScratch {
    rows: Vec<Vec<u32>>,
    same_l: Vec<u64>,
    same_f: Vec<u64>,
    start: Vec<u64>,
    /// bytes copied rather than walked, over every block so far
    pub copied: u64,
}

/// A walked row's entry: VISITED, its segment (bits 20-29) and offset (bits 0-19); a
/// start's keeps START. A segment's rows end in SENT, past the last one written
const VISITED: u32 = 1 << 30;
const SENT: u32 = u32::MAX;

/// One lane in `walk_precopy`: its next row, its segment, where its next byte and row go
#[derive(Clone, Copy)]
struct PLane {
    row: u32,
    seg: usize,
    base: *mut u8,
    p: *mut u8,
    end: *mut u8,
    rp: *mut u32,
}

#[inline(always)]
fn bit(bm: &[u64], i: u32) -> bool {
    // SAFETY: callers' rows are below n, and the bitmaps have n.div_ceil(64) words
    unsafe { *bm.get_unchecked((i >> 6) as usize) >> (i & 63) & 1 != 0 }
}

pub fn walk_precopy(
    tt: &mut TVector,
    ll: &[u8],
    counts: &[u32; 256],
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
    pc: &mut PrecopyScratch,
) {
    const L: usize = FLAGGED_LANES;
    let n = tt.0.len();
    out.clear();
    if n < MIN_SEGMENTED {
        return walk(tt, orig_ptr, out);
    }
    let r0 = tt.0[orig_ptr as usize] >> 8;
    random_rows(n, FLAGGED_SEGMENTS - 1, &mut s.starts);
    s.starts.push(r0);
    s.starts.sort_unstable();
    s.starts.dedup();
    let ns = s.starts.len();
    let words = n.div_ceil(64);
    pc.start.clear();
    pc.start.resize(words, 0);
    for &r in &s.starts {
        tt.0[r as usize] |= START;
        pc.start[(r >> 6) as usize] |= 1 << (r & 63);
    }
    // same_l: row k's last-column byte is row k + 1's; same_f: rows k and k + 1 share a
    // bucket of the first column (k + 1 is no bucket's first row)
    pc.same_l.clear();
    pc.same_l.resize(words, 0);
    for (k, w) in ll.windows(2).enumerate() {
        pc.same_l[k >> 6] |= ((w[0] == w[1]) as u64) << (k & 63);
    }
    pc.same_f.clear();
    pc.same_f.resize(words, !0);
    let mut cum = 0usize;
    for &c in counts.iter() {
        cum += c as usize;
        if cum > 0 && cum <= n {
            let k = cum - 1;
            pc.same_f[k >> 6] &= !(1 << (k & 63));
        }
    }
    let cap = 2 * n / ns + 64;
    s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
    pc.rows.resize_with(ns.max(pc.rows.len()), Vec::new);
    for (b, r) in s.bufs[..ns].iter_mut().zip(&mut pc.rows[..ns]) {
        b.clear();
        b.reserve(cap);
        r.clear();
        r.reserve(b.capacity() + 1);
    }
    s.next.clear();
    s.next.resize(ns, usize::MAX);
    let starts = &s.starts;
    let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let t = tt.0.as_mut_ptr();
    let bufs = &mut s.bufs;
    let rows = &mut pc.rows;
    let next = &mut s.next;
    let (same_l, same_f, start_bm) = (&pc.same_l[..], &pc.same_f[..], &pc.start[..]);
    let mut copied = 0u64;

    // A lane's buffers are full (within one byte, and the rows' sentinel): both grow
    fn grow(l: &mut PLane, bufs: &mut [Vec<u8>], rows: &mut [Vec<u32>]) {
        let (b, r) = (&mut bufs[l.seg], &mut rows[l.seg]);
        // SAFETY: p and rp are within their buffers, every element below them written
        let len = unsafe { l.p.offset_from(l.base) } as usize;
        unsafe {
            b.set_len(len);
            r.set_len(len);
        }
        b.reserve(b.capacity());
        r.reserve(b.capacity() + 1 - len);
        l.base = b.as_mut_ptr();
        l.p = unsafe { l.base.add(len) };
        l.end = unsafe { l.base.add(b.capacity() - 1) };
        l.rp = unsafe { r.as_mut_ptr().add(len) };
        unsafe { *l.rp = SENT };
    }
    // A lane takes segment `seg`: its start's own step taken here
    let take = |seg: usize, bufs: &mut [Vec<u8>], rows: &mut [Vec<u32>]| -> PLane {
        let (b, r) = (&mut bufs[seg], &mut rows[seg]);
        let base = b.as_mut_ptr();
        let rp = r.as_mut_ptr();
        let row = starts[seg];
        // SAFETY: the start is a row below n; the buffers have cap > 2 spare elements
        unsafe {
            let q = *t.add(row as usize) & LINK;
            *base = q as u8;
            *rp = row;
            *rp.add(1) = SENT;
            *t.add(row as usize) = START | VISITED | (seg as u32) << 20;
            PLane {
                row: q >> 8,
                seg,
                base,
                p: base.add(1),
                end: base.add(b.capacity() - 1),
                rp: rp.add(1),
            }
        }
    };
    let finish = |l: &PLane, bufs: &mut [Vec<u8>], next: &mut [usize]| {
        let b = &mut bufs[l.seg];
        // SAFETY: p is within b's allocation, every byte below it written
        unsafe { b.set_len(l.p.offset_from(l.base) as usize) };
        next[l.seg] = seg_of(l.row);
    };
    // One step of a lane: walked, or a run copied from an earlier walk. True when its
    // segment has ended (at its row, a start)
    let mut advance = |l: &mut PLane, bufs: &mut [Vec<u8>], rows: &mut [Vec<u32>]| -> bool {
        // SAFETY: rows are below n (T-vector links; copied rows are s + 1 where same_l
        // holds at s, so s + 1 < n); a lane's p stays below end and rp within its rows
        unsafe {
            let c = l.row;
            let q = *t.add(c as usize);
            if q & START != 0 {
                return true;
            }
            let off = l.p.offset_from(l.base) as u32;
            *l.p = q as u8;
            *l.rp = c;
            *l.rp.add(1) = SENT;
            *t.add(c as usize) = VISITED | (l.seg as u32) << 20 | off;
            l.p = l.p.add(1);
            l.rp = l.rp.add(1);
            if l.p == l.end {
                grow(l, bufs, rows);
            }
            let nx = (q & LINK) >> 8;
            if c > 0 {
                let e = *t.add(c as usize - 1);
                if e & VISITED != 0 {
                    let sseg = ((e >> 20) & 0x3ff) as usize;
                    let mut j = (e & 0xf_ffff) as usize + 1;
                    let mut sr = rows[sseg].as_ptr();
                    let mut sv = *sr.add(j);
                    if sv != SENT && sv.wrapping_add(1) == nx {
                        // the copy: this walk is the source's, one row on
                        let mut sb = bufs[sseg].as_ptr();
                        loop {
                            let cr = sv + 1;
                            if bit(start_bm, cr) {
                                l.row = cr;
                                return true;
                            }
                            if !bit(same_l, sv) {
                                l.row = cr;
                                prefetch(t, cr);
                                return false;
                            }
                            *l.p = *sb.add(j);
                            *l.rp = cr;
                            *l.rp.add(1) = SENT;
                            l.p = l.p.add(1);
                            l.rp = l.rp.add(1);
                            copied += 1;
                            if l.p == l.end {
                                grow(l, bufs, rows);
                                sb = bufs[sseg].as_ptr();
                                sr = rows[sseg].as_ptr();
                            }
                            let sn = *sr.add(j + 1);
                            if sn == SENT || !bit(same_f, sv) || !bit(same_l, sn) {
                                // cr's entry is whole (copied rows are not marked)
                                l.row = (*t.add(cr as usize) & LINK) >> 8;
                                prefetch(t, l.row);
                                return false;
                            }
                            sv = sn;
                            j += 1;
                        }
                    }
                }
            }
            l.row = nx;
            prefetch(t, nx);
            false
        }
    };

    let mut queued = 0;
    let mut live: Vec<PLane> = Vec::with_capacity(L);
    while live.len() < L && queued < ns {
        let l = take(queued, bufs, rows);
        prefetch(t, l.row);
        live.push(l);
        queued += 1;
    }
    let mut lanes: [PLane; L] = [PLane {
        row: 0,
        seg: usize::MAX,
        base: std::ptr::null_mut(),
        p: std::ptr::null_mut(),
        end: std::ptr::null_mut(),
        rp: std::ptr::null_mut(),
    }; L];
    if live.len() == L {
        lanes.copy_from_slice(&live);
        live.clear();
        'full: loop {
            for k in 0..L {
                if advance(&mut lanes[k], bufs, rows) {
                    finish(&lanes[k], bufs, next);
                    if queued == ns {
                        live.extend(
                            lanes
                                .iter()
                                .enumerate()
                                .filter(|&(i, _)| i != k)
                                .map(|(_, l)| *l),
                        );
                        break 'full;
                    }
                    lanes[k] = take(queued, bufs, rows);
                    prefetch(t, lanes[k].row);
                    queued += 1;
                }
            }
        }
    }
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            if advance(&mut live[i], bufs, rows) {
                finish(&live[i], bufs, next);
                live.swap_remove(i);
                continue;
            }
            i += 1;
        }
    }
    pc.copied += copied;
    out.reserve(n);
    let mut seg = seg_of(r0);
    while out.len() < n {
        let b = &bufs[seg];
        out.extend_from_slice(&b[..b.len().min(n - out.len())]);
        seg = next[seg];
    }
}

/// A hint that the T-vector entry at `row` (below its length) is read soon: a software
/// prefetch into L1 where the target has one (`prefetcht0`, `prfm pldl1keep`), else
/// nothing. A hint only: it changes no value read.
#[inline(always)]
fn prefetch(t: *const u32, row: u32) {
    let p = t.wrapping_add(row as usize);
    #[cfg(target_arch = "x86_64")]
    // SAFETY: SSE is in x86_64's baseline; a prefetch never faults
    unsafe {
        core::arch::x86_64::_mm_prefetch::<{ core::arch::x86_64::_MM_HINT_T0 }>(p as *const i8)
    };
    #[cfg(target_arch = "aarch64")]
    // SAFETY: a prefetch never faults and writes nothing
    unsafe {
        core::arch::asm!("prfm pldl1keep, [{p}]", p = in(reg) p, options(nostack, preserves_flags, readonly))
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

    fn check(ll: &[u8]) {
        let mut counts = [0u32; 256];
        for &c in ll {
            counts[c as usize] += 1;
        }
        let mut tt = TVector::new();
        build(ll, &counts, &mut tt);
        let (mut plain, mut flagged, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
        let mut pc = PrecopyScratch::default();
        for orig_ptr in [0, ll.len() as u32 / 3, ll.len() as u32 - 1] {
            walk(&tt, orig_ptr, &mut plain);
            walk_segments_flagged(&mut tt, orig_ptr, &mut flagged, &mut seg);
            assert!(plain == flagged, "orig_ptr {orig_ptr}");
            // Precopy consumes its T-vector: a fresh one
            let mut tp = TVector::new();
            build(ll, &counts, &mut tp);
            walk_precopy(
                &mut tp,
                ll,
                &counts,
                orig_ptr,
                &mut flagged,
                &mut seg,
                &mut pc,
            );
            assert!(plain == flagged, "precopy, orig_ptr {orig_ptr}");
            // the flags are taken off again: the T-vector is as build left it
            assert!(tt.0.iter().all(|&e| e & START == 0));
            // other lane and segment counts, with and without the prefetch
            for segments in [1, 256, 1024] {
                walk_segments_flagged_with::<12, false>(
                    &mut tt,
                    orig_ptr,
                    &mut flagged,
                    &mut seg,
                    segments,
                );
                assert!(plain == flagged, "12 lanes, {segments} segments");
                walk_segments_flagged_with::<7, true>(
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

    /// Text of a few phrases repeated in a pseudo-random order (long repeats, as in
    /// Wikidata's JSON), `n` bytes
    fn phrases(n: usize) -> Vec<u8> {
        let words: [&[u8]; 6] = [
            b"{\"type\":\"item\",",
            b"\"labels\":{",
            b"\"en\":",
            b"\"value\":\"",
            b"Q42\"},",
            b"\"claims\":[",
        ];
        let mut x = 0x2545_f491_4f6c_dd1du64;
        let mut t = Vec::with_capacity(n + 32);
        while t.len() < n {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            t.extend_from_slice(words[(x % 6) as usize]);
        }
        t.truncate(n);
        t
    }

    /// The last column of `t`'s rotations sorted (a naive sort: test sizes only)
    fn bwt_of(t: &[u8]) -> Vec<u8> {
        let n = t.len();
        let mut idx: Vec<usize> = (0..n).collect();
        let dbl: Vec<u8> = t.iter().chain(t.iter()).copied().collect();
        idx.sort_by(|&a, &b| dbl[a..a + n].cmp(&dbl[b..b + n]));
        idx.iter().map(|&i| t[(i + n - 1) % n]).collect()
    }

    /// The flagged walk on a random last column and on a periodic one (a list of several
    /// cycles), both past MIN_SEGMENTED so the segments are walked: small enough for Miri
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
        check(&bwt_of(&phrases(n)));
    }
}
