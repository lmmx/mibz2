// bz2ngrs's inverse BWT, src/bwt.rs at Softleif/bz2ngrs 6a9259078bd97e950020db2f3b5dc4c21613d670,
// copied for bench/alternatives' `kernels` comparison only (never published, never part of
// mibz2's package).
//
// Copyright the bz2ngrs authors; Apache-2.0 WITH LLVM-exception
// (https://github.com/Softleif/bz2ngrs/blob/6a9259078bd97e950020db2f3b5dc4c21613d670/LICENSE).
// Ported there from bz2-ng's dec_block.c (MIT, Copyright Tim Fennell and contributors).
//
// Changed from the original, and nothing else:
// - `crate::decode::zeroed_box!` (bz2ngrs's src/decode.rs) is defined here as it is there;
// - `inverse_timed` and `count_only` added at the end: `inverse`'s steps timed apart,
//   and `build_t`'s counting loop alone.
//! Inverse BWT (`dec_block.c`, WIDE layout): a table build in four
//! interleaved quarters, splitter rows that cut the text into segments, a
//! walk of several segments in lock step into a pool of 1 KiB chunks, and a
//! gather of the segments in text order.
//!
//! The table maps each row to its successor in text order, `T(LF(i)) = i`,
//! with the row's F symbol in the low byte. It is a scatter by the exact
//! prefix sums of the symbols in L, so it is a permutation of `[0, n)` for any
//! L, hostile included: every walk step stays in range and every segment ends
//! at a marked row.
//!
//! All arrays are fixed powers of two indexed with masks, so the hot loops
//! need no bounds checks; a valid index never exceeds 900,000 < 2^20, so the
//! masks change nothing.

#![allow(clippy::all, dead_code)]

macro_rules! zeroed_box {
    ($t:ty, $n:expr) => {
        <Box<[$t; $n]>>::try_from(vec![0 as $t; $n].into_boxed_slice())
            .unwrap_or_else(|_| Box::new([0; $n]))
    };
}

/// Rows in the largest block, rounded up to a power of two.
pub(crate) const ROWS: usize = 1 << 20;
pub(crate) const ROW_MASK: usize = ROWS - 1;
/// L, the pool and the text: one block plus slack for 16-byte stores.
pub(crate) const BYTES: usize = ROWS + 64;

const CHUNK: usize = 1024;
/// Pool chunks: the pool holds at most 900,000 bytes, and each of at most 24
/// lanes has one chunk that is not full, so at most 878 + 24 = 902 are used.
const CHUNKS: usize = 1024;
/// Target rows per segment. When the first lane runs out of segments the
/// others finish theirs with less overlap, so shorter segments shorten that
/// tail; bz2-ng's 2048 left a third of the walk there at 64 Ki rows. Above
/// `SEGS * 512` rows the cap below lengthens them again, which is what the
/// larger blocks prefer.
const SEG_ROWS: u32 = 512;
/// Segments, at most this many minus one (with `orig_ptr`) are used.
const SEGS: usize = 512;
const SPLIT: u32 = 0x8000_0000;

/// Per-thread arrays of the inverse BWT.
pub(crate) struct Bwt {
    tt: Box<[u32; ROWS]>,
    m: Meta,
}

impl Bwt {
    pub(crate) fn new() -> Bwt {
        Bwt {
            tt: zeroed_box!(u32, ROWS),
            m: Meta::ZERO,
        }
    }
}

/// The walk's bookkeeping. Kept apart from the table and the pool so that
/// the walk can take all three as separate arguments: LLVM then knows the
/// pool's byte stores do not change them.
struct Meta {
    chunk_next: [u32; CHUNKS],
    seg_row: [u32; SEGS],
    seg_start: [u32; SEGS],
    seg_len: [u32; SEGS],
    seg_next: [u32; SEGS],
    n_seg: u32,
    next_chunk: u32,
    next_split: u32,
}

/// Inverts the BWT `ll[0..n)` with origin row `orig_ptr < n` into
/// `text[0..n)`. `ll` is used as the walk's pool, so its contents are lost.
pub(crate) fn inverse(
    b: &mut Bwt,
    ll: &mut [u8; BYTES],
    n: u32,
    orig_ptr: u32,
    text: &mut [u8; BYTES],
) {
    build_t(&mut b.tt, ll, n);
    let lanes = mark_splitters(b, n, orig_ptr);
    let Bwt { tt, m } = b;
    let origin = m.seg_index(orig_ptr);
    m.next_chunk = 0;
    m.next_split = 0;
    walk(tt, ll, m, lanes as usize);
    gather(m, ll, origin, n, text);
}

/// Parts of L that the table build counts and scatters interleaved. L is
/// mostly runs of one byte, so within a part each counter and cursor update
/// waits for the previous one; the parts are independent chains.
const PARTS: usize = 8;

/// Builds the table from L, in `PARTS` parts with their own cursors.
fn build_t(tt: &mut [u32; ROWS], ll: &[u8; BYTES], n: u32) {
    let n = (n as usize).min(ROWS);
    let q = n / PARTS;
    // Byte `i` of part `k`; the last part also takes the `n % PARTS` tail.
    let at = |k: usize, i: usize| usize::from(ll[(k * q + i) & ROW_MASK]);

    let mut h = [[0u32; 256]; PARTS];
    for i in 0..q {
        for (k, h) in h.iter_mut().enumerate() {
            h[at(k, i)] += 1;
        }
    }
    for i in q..n - (PARTS - 1) * q {
        h[PARTS - 1][at(PARTS - 1, i)] += 1;
    }

    // Exclusive prefix sums, split into per-part cursors.
    let mut cf = [[0u32; 256]; PARTS];
    let mut sum = 0u32;
    for c in 0..256 {
        for k in 0..PARTS {
            cf[k][c] = sum;
            sum += h[k][c];
        }
    }

    // Four parts per pass: eight at once spill on x86-64 (16 registers) and
    // gain nothing on aarch64. All cursors are loaded before any entry is
    // stored, and each entry is stored before its cursor (the clang order of
    // dec_block.c).
    for g in (0..PARTS).step_by(4) {
        for i in 0..q {
            let c: [usize; 4] = core::array::from_fn(|k| at(g + k, i));
            let j: [u32; 4] = core::array::from_fn(|k| cf[g + k][c[k]]);
            for k in 0..4 {
                tt[j[k] as usize & ROW_MASK] = (((g + k) * q + i) as u32) << 8 | c[k] as u32;
                cf[g + k][c[k]] = j[k] + 1;
            }
        }
    }
    for i in q..n - (PARTS - 1) * q {
        let c = at(PARTS - 1, i);
        let j = cf[PARTS - 1][c];
        tt[j as usize & ROW_MASK] = (((PARTS - 1) * q + i) as u32) << 8 | c as u32;
        cf[PARTS - 1][c] = j + 1;
    }
}

/// Chooses splitter rows (evenly spaced, plus `orig_ptr`), marks them in the
/// table and returns the lane count.
///
/// The lane counts differ from bz2-ng's (24 from 64 Ki rows, 8 from 16 Ki):
/// measured on the M4 Pro, 24 lanes pay off from 4 Ki rows and 8 from 1 Ki
/// given at least three segments per lane.
fn mark_splitters(b: &mut Bwt, n: u32, op: u32) -> u32 {
    let (lanes, min_segs) = match n {
        4096.. => (24, 72),
        1024.. => (8, 24),
        _ => (2, 1),
    };
    let want = (n / SEG_ROWS).max(min_segs).clamp(1, SEGS as u32 - 2);
    let rows = &mut b.m.seg_row;
    let mut m = 0usize;
    let mut inserted = false;
    let mut push = |r: u32| {
        if m == 0 || rows[(m - 1) % SEGS] != r {
            rows[m % SEGS] = r;
            m += 1;
        }
    };
    for k in 0..want {
        let r = (u64::from(k) * u64::from(n) / u64::from(want)) as u32;
        if !inserted && op <= r {
            push(op);
            inserted = true;
        }
        push(r);
    }
    if !inserted {
        push(op);
    }
    b.m.n_seg = m as u32;
    for &r in &b.m.seg_row[..m.min(SEGS)] {
        b.tt[r as usize & ROW_MASK] |= SPLIT;
    }
    if m >= 2 { lanes } else { 1 }
}

impl Meta {
    const ZERO: Meta = Meta {
        chunk_next: [0; CHUNKS],
        seg_row: [0; SEGS],
        seg_start: [0; SEGS],
        seg_len: [0; SEGS],
        seg_next: [0; SEGS],
        n_seg: 0,
        next_chunk: 0,
        next_split: 0,
    };

    /// Index of `row` in the sorted splitter list (`row` is a splitter).
    fn seg_index(&self, row: u32) -> u32 {
        let n = (self.n_seg as usize).min(SEGS);
        self.seg_row[..n].partition_point(|&r| r < row) as u32
    }

    /// Gives a lane a fresh chunk and returns its start. With `link`, `out`
    /// is the end of the lane's full current chunk, which is linked to it.
    fn new_chunk(&mut self, cold: &mut Cold, out: u32, link: bool) -> u32 {
        let c = self.next_chunk;
        self.next_chunk += 1;
        if link {
            let prev = (out as usize / CHUNK).wrapping_sub(1);
            self.chunk_next[prev % CHUNKS] = c;
            cold.written += CHUNK as u32;
        }
        self.chunk_next[c as usize % CHUNKS] = 0;
        (c % CHUNKS as u32) * CHUNK as u32
    }

    /// Starts the next unwalked segment on a lane whose next write goes to
    /// `out`: emits the splitter row's byte and returns the lane's next row
    /// and write position, or `None` if no segment is left.
    fn seg_begin(
        &mut self,
        tt: &[u32; ROWS],
        pool: &mut [u8; BYTES],
        cold: &mut Cold,
        out: u32,
    ) -> Option<(u32, u32)> {
        if self.next_split >= self.n_seg {
            return None;
        }
        let seg = self.next_split;
        self.next_split += 1;
        let s = seg as usize % SEGS;
        self.seg_start[s] = out;
        cold.seg = seg;
        cold.mark = cold.written + out % CHUNK as u32;
        let e = tt[self.seg_row[s] as usize & ROW_MASK];
        pool[out as usize & ROW_MASK] = e as u8;
        let out = out + 1;
        let out = if (out as usize).is_multiple_of(CHUNK) {
            self.new_chunk(cold, out, true)
        } else {
            out
        };
        Some(((e >> 8) & ROW_MASK as u32, out))
    }
}

/// A lane's state that only the rare steps touch. The hot state, the current
/// row `p` and the next write position `out`, goes by value so the unrolled
/// lock-step loops keep it in registers. `out` always points at a writable
/// byte: a lane takes its next chunk as soon as the current one is full.
#[derive(Clone, Copy, Default)]
struct Cold {
    /// Bytes written before the current chunk.
    written: u32,
    /// Segment being walked, and the lane's byte count at its start.
    seg: u32,
    mark: u32,
}

/// One step of a lane: emits the current row's byte and follows the table.
/// A marked row (end of segment) and a full chunk go out of line. Returns
/// the new `p` and `out`, and false when the lane has nothing left to walk.
#[inline(always)]
fn step(
    tt: &[u32; ROWS],
    pool: &mut [u8; BYTES],
    m: &mut Meta,
    cold: &mut Cold,
    p: u32,
    out: u32,
) -> (u32, u32, bool) {
    let e = tt[p as usize & ROW_MASK];
    if e & SPLIT != 0 {
        return seg_done(tt, pool, m, cold, p, out);
    }
    pool[out as usize & ROW_MASK] = e as u8;
    let (p, out) = ((e >> 8) & ROW_MASK as u32, out + 1);
    if (out as usize).is_multiple_of(CHUNK) {
        return (p, chunk_full(m, cold, out), true);
    }
    (p, out, true)
}

#[cold]
#[inline(never)]
fn chunk_full(m: &mut Meta, cold: &mut Cold, out: u32) -> u32 {
    m.new_chunk(cold, out, true)
}

/// End of the lane's segment: records it and starts the next one.
#[cold]
#[inline(never)]
fn seg_done(
    tt: &[u32; ROWS],
    pool: &mut [u8; BYTES],
    m: &mut Meta,
    cold: &mut Cold,
    p: u32,
    out: u32,
) -> (u32, u32, bool) {
    let s = cold.seg as usize % SEGS;
    m.seg_len[s] = cold.written + out % CHUNK as u32 - cold.mark;
    m.seg_next[s] = m.seg_index(p);
    match m.seg_begin(tt, pool, cold, out) {
        Some((p, out)) => (p, out, true),
        None => (DONE, out, false),
    }
}

/// The row of a lane that has run out of segments.
const DONE: u32 = u32::MAX;

/// `K` lanes in lock step until any lane runs out of segments (its row is
/// then [`DONE`]), written out with one local per lane's hot state (an
/// array, even indexed by constants, stays in memory).
macro_rules! lockstep {
    ($name:ident, $k:literal, [$(($p:ident $o:ident $i:literal))*]) => {
        #[inline(never)]
        fn $name(
            tt: &[u32; ROWS],
            pool: &mut [u8; BYTES],
            m: &mut Meta,
            hot: &mut [(u32, u32); $k],
            cold: &mut [Cold; $k],
        ) {
            $(let (mut $p, mut $o) = hot[$i];)*
            let mut live = true;
            while live {
                $(
                    let (p, o, l) = step(tt, pool, m, &mut cold[$i], $p, $o);
                    $p = p;
                    $o = o;
                    live &= l;
                )*
            }
            $(hot[$i] = ($p, $o);)*
        }
    };
}
lockstep!(lockstep1, 1, [(p0 o0 0)]);
lockstep!(lockstep2, 2, [(p0 o0 0) (p1 o1 1)]);
lockstep!(lockstep8, 8, [(p0 o0 0) (p1 o1 1) (p2 o2 2) (p3 o3 3) (p4 o4 4) (p5 o5 5) (p6 o6 6) (p7 o7 7)]);

/// [`lockstep!`] for lane counts too large for registers, where unrolling
/// only adds spills: the lanes stay in their arrays.
#[inline(never)]
fn lockstep_mem<const K: usize>(
    tt: &[u32; ROWS],
    pool: &mut [u8; BYTES],
    m: &mut Meta,
    hot: &mut [(u32, u32); K],
    cold: &mut [Cold; K],
) {
    let mut dead = 0u32;
    while dead == 0 {
        for (k, (h, c)) in hot.iter_mut().zip(cold.iter_mut()).enumerate() {
            let (p, o, l) = step(tt, pool, m, c, h.0, h.1);
            *h = (p, o);
            dead |= u32::from(!l) << k;
        }
    }
}

/// Walks every segment with `k` (1, 2, 8 or 24) lanes in lock step, then
/// finishes the stragglers.
#[inline(never)]
fn walk(tt: &[u32; ROWS], pool: &mut [u8; BYTES], m: &mut Meta, k: usize) {
    let mut hot = [(0u32, 0u32); 24];
    let mut cold = [Cold::default(); 24];
    let k = k.min(24);
    let mut active = 0;
    while active < k {
        let mut c = Cold::default();
        let out = m.new_chunk(&mut c, 0, false);
        let Some(h) = m.seg_begin(tt, pool, &mut c, out) else {
            break;
        };
        hot[active] = h;
        cold[active] = c;
        active += 1;
    }
    if active == k {
        macro_rules! run {
            ($f:ident, $n:literal) => {
                match (hot.first_chunk_mut::<$n>(), cold.first_chunk_mut::<$n>()) {
                    (Some(h), Some(c)) => $f(tt, pool, m, h, c),
                    _ => {}
                }
            };
        }
        match k {
            24 => run!(lockstep_mem, 24),
            8 => run!(lockstep8, 8),
            2 => run!(lockstep2, 2),
            _ => run!(lockstep1, 1),
        }
        let mut a = 0;
        for i in 0..k {
            if hot[i].0 != DONE {
                hot[a] = hot[i];
                cold[a] = cold[i];
                a += 1;
            }
        }
        active = a;
    }
    let mut i = 0;
    while active > 0 {
        if i >= active {
            i = 0;
        }
        let (p, o, live) = step(tt, pool, m, &mut cold[i], hot[i].0, hot[i].1);
        if live {
            hot[i] = (p, o);
            i += 1;
        } else {
            active -= 1;
            hot[i] = hot[active];
            cold[i] = cold[active];
        }
    }
}

/// Copies the segments into `text` in text order, starting with the one at
/// `orig_ptr` and wrapping round the cycle until `n` bytes are out.
fn gather(b: &Meta, pool: &[u8; BYTES], origin: u32, n: u32, text: &mut [u8; BYTES]) {
    let n = (n as usize).min(ROWS);
    let mut seg = origin as usize % SEGS;
    let mut t = 0usize;
    while t < n {
        let mut left = (b.seg_len[seg] as usize).min(n - t);
        if left == 0 {
            break; // unreachable: every segment holds its splitter's byte
        }
        let mut off = b.seg_start[seg] as usize & ROW_MASK;
        loop {
            let room = CHUNK - off % CHUNK;
            let k = room.min(left);
            if let (Some(dst), Some(src)) = (text.get_mut(t..t + k), pool.get(off..off + k)) {
                dst.copy_from_slice(src);
            }
            t += k;
            left -= k;
            if left == 0 {
                break;
            }
            off = (b.chunk_next[off / CHUNK % CHUNKS] as usize % CHUNKS) * CHUNK;
        }
        seg = b.seg_next[seg] as usize % SEGS;
    }
}

/// `inverse`, its build, walk (the splitters chosen and marked, and the lanes walked) and
/// gather timed apart, in seconds
pub(crate) fn inverse_timed(
    b: &mut Bwt,
    ll: &mut [u8; BYTES],
    n: u32,
    orig_ptr: u32,
    text: &mut [u8; BYTES],
) -> [f64; 3] {
    let t0 = std::time::Instant::now();
    build_t(&mut b.tt, ll, n);
    let t1 = std::time::Instant::now();
    let lanes = mark_splitters(b, n, orig_ptr);
    let Bwt { tt, m } = b;
    let origin = m.seg_index(orig_ptr);
    m.next_chunk = 0;
    m.next_split = 0;
    walk(tt, ll, m, lanes as usize);
    let t2 = std::time::Instant::now();
    gather(m, ll, origin, n, text);
    let t3 = std::time::Instant::now();
    [
        (t1 - t0).as_secs_f64(),
        (t2 - t1).as_secs_f64(),
        (t3 - t2).as_secs_f64(),
    ]
}

/// `build_t`'s counting half alone (its first loop, as it is there), to split the build
/// into its count and its scatter
pub(crate) fn count_only(ll: &[u8; BYTES], n: u32) -> u32 {
    const PARTS: usize = 8;
    let n = (n as usize).min(ROWS);
    let q = n / PARTS;
    let at = |k: usize, i: usize| usize::from(ll[(k * q + i) & ROW_MASK]);
    let mut h = [[0u32; 256]; PARTS];
    for i in 0..q {
        for (k, h) in h.iter_mut().enumerate() {
            h[at(k, i)] += 1;
        }
    }
    for i in q..n - (PARTS - 1) * q {
        h[PARTS - 1][at(PARTS - 1, i)] += 1;
    }
    h.iter().map(|h| h[0]).sum()
}
