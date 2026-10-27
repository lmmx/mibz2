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

/// The pair walk (#6): ψ², two bytes a step, as libsais 2.12 inverts a cyclic BWT
/// (`build_pairs`, `walk_pairs_flagged`), the segmented walk for a short block
#[derive(Default)]
pub struct Pairs {
    pairs: PairTable,
    seg: SegScratch,
    lfs: Vec<(u32, u32)>,
    short: Segmented,
}

impl InverseBwt for Pairs {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        if ll.len() < MIN_SEGMENTED {
            return self.short.invert(ll, counts, orig_ptr, out);
        }
        walk_pairs_flagged::<FLAGGED_LANES, PREFETCH>(
            &mut self.pairs,
            ll,
            counts,
            orig_ptr,
            out,
            &mut self.seg,
            &mut self.lfs,
            FLAGGED_SEGMENTS,
        );
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

/// The pair table: row r's entry is its last-column byte L[r] | ψ²(r) << 8, and the byte
/// after L[r] in the text, L[ψ(r)], is F(r), row r's first byte, found from `first` and
/// the buckets' starts `f`. So a step from r gives two bytes and the row two on. Built
/// only by `build_pairs`, so every link is below its length; bits 30 and 31 are the walk's
/// flags (links are below 2^20).
pub struct PairTable {
    p: Vec<u32>,
    b2: Vec<u32>,
    first: Vec<u8>,
    f: [u32; 257],
    fshift: u32,
}

impl Default for PairTable {
    fn default() -> Self {
        PairTable {
            p: Vec::new(),
            b2: Vec::new(),
            first: Vec::new(),
            f: [0; 257],
            fshift: 0,
        }
    }
}

/// A segment's start (in its own entry), and with it, in the entry of the row one before
/// it in the text (LF of the start), the mark that a lane reaching that row ends there
/// after one byte: a step of two rows passes over every other row, so a start is seen
/// from whichever of the two the lane lands on (libsais's marks and their LF rows)
const PAIR_LF: u32 = 1 << 30;
const PAIR_LINK: u32 = !(START | PAIR_LF);

impl PairTable {
    /// Row r's first byte: its F bucket, from the slice of rows r is in
    #[inline(always)]
    fn first_byte(&self, r: u32) -> u8 {
        // SAFETY: r < n, so r >> fshift < first.len(), and the bucket ends rise to f[256] = n
        unsafe {
            let mut c = *self.first.get_unchecked((r >> self.fshift) as usize) as usize;
            while *self.f.get_unchecked(c + 1) <= r {
                c += 1;
            }
            c as u8
        }
    }
}

/// ψ² of the last column `ll` (whose bytes `counts` counts), as libsais builds it.
/// - A step i of the counting sort has p = LF(i) (its byte c's next slot), and i's row
///   two on from the row LF(p) = LF²(i), the next slot of the bigram (L[p], c): the rows
///   sorted by their first two bytes, in i's order within each.
/// - The bigram counts come from a byte histogram of each F bucket's slice of `ll` (row p
///   in bucket c is bigram (L[p], c)): sequential, not a pass of LF lookups.
/// - 16 equal bytes c at i whose LF rows' 16 bytes are equal too (the run lemma, twice)
///   go in with one 16-wide store, libsais's RUN_LANES, as in `build`.
///
/// `rows` (sorted) are the rows whose LF is wanted: each one's is pushed to `lf`, in order.
#[allow(clippy::needless_range_loop)]
pub fn build_pairs(
    ll: &[u8],
    counts: &[u32; 256],
    t: &mut PairTable,
    rows: &[u32],
    lf: &mut Vec<u32>,
) {
    let n = ll.len();
    t.p.clear();
    t.p.extend(ll.iter().map(|&c| c as u32));
    let mut cum = 0u32;
    for c in 0..256 {
        t.f[c] = cum;
        cum += counts[c];
    }
    t.f[256] = cum;
    t.fshift = 0;
    while (n >> t.fshift) >= 1 << 15 {
        t.fshift += 1;
    }
    t.first.clear();
    let mut c = 0;
    for j in 0..((n - 1) >> t.fshift) + 1 {
        let r = (j << t.fshift) as u32;
        while t.f[c + 1] <= r {
            c += 1;
        }
        t.first.push(c as u8);
    }
    // The bigram counts, then each bigram's first slot (first byte major)
    t.b2.clear();
    t.b2.resize(1 << 16, 0);
    let mut h = [[0u32; 256]; 4];
    for c in 0..256 {
        let slice = &ll[t.f[c] as usize..t.f[c + 1] as usize];
        if slice.is_empty() {
            continue;
        }
        let mut q = slice.chunks_exact(4);
        for x in &mut q {
            h[0][x[0] as usize] += 1;
            h[1][x[1] as usize] += 1;
            h[2][x[2] as usize] += 1;
            h[3][x[3] as usize] += 1;
        }
        for &d in q.remainder() {
            h[0][d as usize] += 1;
        }
        for d in 0..256 {
            t.b2[d << 8 | c] = h[0][d] + h[1][d] + h[2][d] + h[3][d];
            (h[0][d], h[1][d], h[2][d], h[3][d]) = (0, 0, 0, 0);
        }
    }
    let mut sum = 0u32;
    for e in t.b2.iter_mut() {
        let v = *e;
        *e = sum;
        sum += v;
    }
    // The scatter
    let (p, b2) = (&mut t.p, &mut t.b2);
    let mut b1 = [0u32; 256];
    b1.copy_from_slice(&t.f[..256]);
    lf.clear();
    let mut next = rows.first().map_or(usize::MAX, |&r| r as usize);
    let mut ri = 0;
    let mut i = 0;
    while i + 16 <= n {
        let c = ll[i] as usize;
        let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
        if a == b && a == a.rotate_left(8) {
            let q = b1[c] as usize;
            if q + 16 <= n {
                let (a, b) = (read_u64(ll, q), read_u64(ll, q + 8));
                if a == b && a == a.rotate_left(8) {
                    let w = (ll[q] as usize) << 8 | c;
                    let x = b2[w] as usize;
                    let base = (i as u32) << 8;
                    for (j, e) in p[x..x + 16].iter_mut().enumerate() {
                        *e = (*e & 0xff) | (base + ((j as u32) << 8));
                    }
                    while next < i + 16 {
                        lf.push((q + next - i) as u32);
                        ri += 1;
                        next = rows.get(ri).map_or(usize::MAX, |&r| r as usize);
                    }
                    b1[c] += 16;
                    b2[w] += 16;
                    i += 16;
                    continue;
                }
            }
        }
        let e = i + 16;
        if next >= e {
            pair_steps(ll, p, b2, &mut b1, i..e);
        } else {
            for i in i..e {
                if i == next {
                    lf.push(b1[ll[i] as usize]);
                    ri += 1;
                    next = rows.get(ri).map_or(usize::MAX, |&r| r as usize);
                }
                pair_steps(ll, p, b2, &mut b1, i..i + 1);
            }
        }
        i = e;
    }
    for i in i..n {
        if i == next {
            lf.push(b1[ll[i] as usize]);
            ri += 1;
            next = rows.get(ri).map_or(usize::MAX, |&r| r as usize);
        }
        pair_steps(ll, p, b2, &mut b1, i..i + 1);
    }
}

/// `build_pairs`'s one-row steps, over the rows `rows`
#[inline(always)]
fn pair_steps(
    ll: &[u8],
    p: &mut [u32],
    b2: &mut [u32],
    b1: &mut [u32; 256],
    rows: std::ops::Range<usize>,
) {
    for i in rows {
        // SAFETY: the counts are ll's own, so q < n, and the bigram slots x < n
        unsafe {
            let c = *ll.get_unchecked(i) as usize;
            let q = *b1.get_unchecked(c) as usize;
            *b1.get_unchecked_mut(c) += 1;
            let w = (*ll.get_unchecked(q) as usize) << 8 | c;
            let x = *b2.get_unchecked(w) as usize;
            *b2.get_unchecked_mut(w) += 1;
            let e = p.get_unchecked_mut(x);
            *e = (*e & 0xff) | (i as u32) << 8;
        }
    }
}

/// The row ψ(orig_ptr) the text starts at: the row whose LF is `orig_ptr`, the
/// (orig_ptr − f[c])-th c in `ll`, for c = F(orig_ptr)
fn pair_start(ll: &[u8], t: &PairTable, orig_ptr: u32) -> u32 {
    let c = t.first_byte(orig_ptr);
    let mut k = (orig_ptr - t.f[c as usize]) as usize;
    let mut base = 0;
    for chunk in ll.chunks(64) {
        let m = chunk.iter().filter(|&&b| b == c).count();
        if k < m {
            let j = chunk
                .iter()
                .enumerate()
                .filter(|&(_, &b)| b == c)
                .nth(k)
                .unwrap()
                .0;
            return (base + j) as u32;
        }
        k -= m;
        base += chunk.len();
    }
    unreachable!("the counts are ll's own")
}

/// The flagged segmented walk over the pair table: `walk_segments_flagged`'s lanes,
/// segments and joins, two bytes a step. The segments' starts are flagged in their own
/// entries, and their LF rows with `PAIR_LF`: a lane landing on one writes its byte and
/// ends, its successor the start one row on. A start whose LF row is another start is
/// dropped (the text's first row is kept). Exactly `walk`'s output.
#[allow(clippy::too_many_arguments)]
pub fn walk_pairs_flagged<const L: usize, const PF: bool>(
    t: &mut PairTable,
    ll: &[u8],
    counts: &[u32; 256],
    orig_ptr: u32,
    out: &mut Vec<u8>,
    s: &mut SegScratch,
    lfs: &mut Vec<(u32, u32)>,
    segments: usize,
) {
    let n = ll.len();
    out.clear();
    random_rows(n, segments - 1, &mut s.starts);
    s.starts.sort_unstable();
    s.starts.dedup();
    let mut lf_rows = Vec::with_capacity(s.starts.len());
    build_pairs(ll, counts, t, &s.starts, &mut lf_rows);
    let r0 = pair_start(ll, t, orig_ptr);
    // (row, its LF) for every start, the text's first row among them, sorted by row
    let mut st: Vec<(u32, u32)> = s
        .starts
        .iter()
        .copied()
        .zip(lf_rows.iter().copied())
        .collect();
    match st.binary_search_by_key(&r0, |e| e.0) {
        Ok(_) => {}
        Err(k) => st.insert(k, (r0, orig_ptr)),
    }
    // A start whose LF row is another start: one of the two dropped (never the first row)
    let mut drop = vec![false; st.len()];
    for k in 0..st.len() {
        // (a row that is its own LF, a cycle of one, needs no LF mark: its lane comes back
        // to it. ψ has several cycles when a block's text is periodic, as `pattern` is)
        if let Ok(j) = st.binary_search_by_key(&st[k].1, |e| e.0) {
            if j == k {
                continue;
            }
            if st[j].0 == r0 {
                drop[k] = true;
            } else {
                drop[j] = true;
            }
        }
    }
    let mut kept = drop.iter();
    st.retain(|_| !*kept.next().unwrap());
    s.starts.clear();
    s.starts.extend(st.iter().map(|e| e.0));
    lfs.clear();
    lfs.extend(st.iter().enumerate().map(|(k, e)| (e.1, k as u32)));
    lfs.sort_unstable();
    for &(r, l) in &st {
        if l != r {
            t.p[l as usize] |= START | PAIR_LF;
        }
        t.p[r as usize] |= START;
    }

    let ns = s.starts.len();
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
    let lfs = &*lfs;
    let seg_after = |row: u32| match lfs.binary_search_by_key(&row, |e| e.0) {
        Ok(k) => lfs[k].1 as usize,
        Err(_) => usize::MAX,
    };
    let tp = t.p.as_ptr();
    let tt = &*t;
    let bufs = &mut s.bufs;
    let next = &mut s.next;

    // A lane takes segment `seg`: its first step (its start's own, flagged, entry) here
    let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
        let b = &mut bufs[seg];
        let base = b.as_mut_ptr();
        let r = starts[seg];
        // SAFETY: the starts are rows below n; the buffer has `cap` > 2 spare bytes
        let q = unsafe { *tp.add(r as usize) } & PAIR_LINK;
        unsafe {
            *base = q as u8;
            *base.add(1) = tt.first_byte(r);
        }
        Lane {
            row: q >> 8,
            seg,
            p: unsafe { base.add(2) },
            end: unsafe { base.add(b.capacity()) },
        }
    };
    // A lane reads a flagged entry at its row: its segment ends there (after the row's
    // byte, at an LF row), its length and successor recorded
    let finish = |l: &mut Lane, q: u32, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
        if q & PAIR_LF != 0 {
            // SAFETY: a step leaves at least 2 spare bytes
            unsafe {
                *l.p = q as u8;
                l.p = l.p.add(1);
            }
            next[l.seg] = seg_after(l.row);
        } else {
            next[l.seg] = seg_of(l.row);
        }
        let b = &mut bufs[l.seg];
        // SAFETY: p is within b's allocation, every byte below it written
        unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
    };
    // Fewer than 2 spare bytes: the buffer grows (its written bytes kept)
    let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
        let b = &mut bufs[l.seg];
        let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
        unsafe { b.set_len(len) };
        b.reserve(b.capacity());
        l.p = unsafe { b.as_mut_ptr().add(len) };
        l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
    };
    // One step: two bytes, and the row two on
    let step = |l: &mut Lane, q: u32, bufs: &mut Vec<Vec<u8>>| {
        let r = l.row;
        l.row = q >> 8;
        if PF {
            prefetch(tp, l.row);
        }
        // SAFETY: at least 2 spare bytes, within the buffer's allocation
        unsafe {
            (l.p as *mut u16).write_unaligned(u16::from_le_bytes([q as u8, tt.first_byte(r)]));
            l.p = l.p.add(2);
        }
        if unsafe { l.end.offset_from(l.p) } < 2 {
            grow(l, bufs);
        }
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
            prefetch(tp, lanes[active].row);
        }
        queued += 1;
        active += 1;
    }
    if active == L {
        'full: loop {
            for l in lanes.iter_mut() {
                // SAFETY: rows are below n (PairTable)
                let q = unsafe { *tp.add(l.row as usize) };
                if q & START != 0 {
                    finish(l, q, bufs, next);
                    if queued == ns {
                        break 'full;
                    }
                    *l = take(queued, bufs);
                    if PF {
                        prefetch(tp, l.row);
                    }
                    queued += 1;
                    continue;
                }
                step(l, q, bufs);
            }
        }
    }
    let mut live: Vec<Lane> = lanes[..active]
        .iter()
        .copied()
        .filter(|l| next_unset(l, next))
        .collect();
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            let q = unsafe { *tp.add(l.row as usize) };
            if q & START != 0 {
                finish(l, q, bufs, next);
                live.swap_remove(i);
                continue;
            }
            step(l, q, bufs);
            i += 1;
        }
    }
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
        for orig_ptr in [0, ll.len() as u32 / 3, ll.len() as u32 - 1] {
            walk(&tt, orig_ptr, &mut plain);
            walk_segments_flagged(&mut tt, orig_ptr, &mut flagged, &mut seg);
            assert!(plain == flagged, "orig_ptr {orig_ptr}");
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

    /// The pair walk gives the plain walk's output: random columns (lists of many cycles,
    /// so lanes go round their own), periodic ones, ones in long runs (the 16-wide stores
    /// on both sides), at several primary indices and segment counts
    #[test]
    fn pair_walk_equals_plain() {
        let n = MIN_SEGMENTED + 1001;
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        let mut rnd = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let mut inputs = vec![
            (0..n).map(|_| (rnd() % 7) as u8).collect::<Vec<u8>>(),
            (0..n).map(|i| b"abc"[i % 3]).collect(),
            (0..n).map(|i| b"ab"[i % 2]).collect(),
        ];
        let mut runs = Vec::new();
        while runs.len() < n {
            let c = (rnd() % 4) as u8;
            runs.extend(std::iter::repeat_n(
                c,
                [1, 5, 16, 17, 40, 300][(rnd() % 6) as usize],
            ));
        }
        runs.truncate(n);
        inputs.push(runs);
        for ll in &inputs {
            let mut counts = [0u32; 256];
            for &c in ll {
                counts[c as usize] += 1;
            }
            let mut tt = TVector::new();
            build(ll, &counts, &mut tt);
            let (mut plain, mut pairs) = (Vec::new(), Vec::new());
            let (mut t, mut seg, mut lfs) =
                (PairTable::default(), SegScratch::default(), Vec::new());
            for orig_ptr in [0, ll.len() as u32 / 3, ll.len() as u32 - 1] {
                walk(&tt, orig_ptr, &mut plain);
                for segments in [2, 64, 512, 4096] {
                    walk_pairs_flagged::<7, true>(
                        &mut t, ll, &counts, orig_ptr, &mut pairs, &mut seg, &mut lfs, segments,
                    );
                    assert!(plain == pairs, "orig_ptr {orig_ptr}, {segments} segments");
                }
                let mut k = Pairs::default();
                k.invert(ll, &counts, orig_ptr, &mut pairs);
                assert!(plain == pairs, "Pairs, orig_ptr {orig_ptr}");
            }
        }
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
    }
}
