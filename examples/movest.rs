//! Scratch (#42), not for master: a length-capped move structure for psi against the
//! T-vector, on one thread, per block, best of REPEATS, summed.
//!
//! ```text
//! movest FILE.bz2 [REPEATS] [GHZ] [ORDER: tm|mt] [CAPS: 2,4,8]
//! (env MOVEST_BLOCKS=N: the first N blocks only)
//! ```
//!
//! The walk in the F formulation: from row orig_ptr, emit F[row], row = psi(row) (the same
//! bytes as `bwt::walk`, since L[psi(k)] = F[k]). F is constant on psi's input intervals
//! (each lies in one F bucket), so an interval carries its byte. An interval is one u64:
//! - bits 0-22: idx, the interval holding the interval's image q;
//! - bit 23: the interval is a segment start (a one-row interval);
//! - bits 24-31: the byte;
//! - bits 32-47: d = q - start(idx), below len(idx);
//! - bits 48-63: len, at most the cap.
//!
//! A lane's state is one u64, s = off << 32 | x: its row is start(x) + off. A step: r = R[x];
//! while off >= len(r): off -= len, x += 1 (the fast-forward, done lazily on the next read,
//! so the prefetch of R[idx] covers it); emit byte(r); s = (off + d) << 32 | idx, which is
//! (s & HI) + (r & D_IDX).

use std::time::Instant;

use mibz2::bwt::{build, walk, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, rle, Kernels};

const IDX_MASK: u64 = (1 << 23) - 1;
const FLAG: u64 = 1 << 23;
const D_IDX: u64 = 0x0000_ffff_0000_0000 | IDX_MASK;
const HI: u64 = 0xffff_ffff_0000_0000;

#[inline(always)]
fn r_byte(r: u64) -> u8 {
    (r >> 24) as u8
}
/// len << 32, to compare against a lane's state
#[inline(always)]
fn r_len_hi(r: u64) -> u64 {
    (r >> 48) << 32
}

#[inline(always)]
fn prefetch<T>(p: *const T) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_mm_prefetch::<{ core::arch::x86_64::_MM_HINT_T0 }>(p as *const i8)
    };
}

/// The runs of L: run j is [start[j], start[j+1]) of byte[j]; start has r + 1 entries
#[derive(Default)]
struct Runs {
    start: Vec<u32>,
    byte: Vec<u8>,
}

/// L's runs, 8 bytes at a time: a word against the word a byte back, its non-zero bytes
/// the run boundaries
fn scan_runs(ll: &[u8], runs: &mut Runs) {
    let n = ll.len();
    runs.start.clear();
    runs.byte.clear();
    runs.start.reserve(n + 1);
    runs.byte.reserve(n + 1);
    let (ps, pb) = (runs.start.as_mut_ptr(), runs.byte.as_mut_ptr());
    let mut k = 0usize;
    if n > 0 {
        unsafe {
            *ps = 0;
            *pb = ll[0];
        }
        k = 1;
        let mut i = 1;
        const LO: u64 = 0x7f7f_7f7f_7f7f_7f7f;
        while i + 8 <= n {
            let a = u64::from_le_bytes(ll[i..i + 8].try_into().unwrap());
            let b = u64::from_le_bytes(ll[i - 1..i + 7].try_into().unwrap());
            let x = a ^ b;
            if x != 0 {
                let mut m = (((x & LO).wrapping_add(LO)) | x) & !LO;
                while m != 0 {
                    let o = i + (m.trailing_zeros() / 8) as usize;
                    unsafe {
                        *ps.add(k) = o as u32;
                        *pb.add(k) = *ll.get_unchecked(o);
                    }
                    k += 1;
                    m &= m - 1;
                }
            }
            i += 8;
        }
        while i < n {
            if ll[i] != ll[i - 1] {
                unsafe {
                    *ps.add(k) = i as u32;
                    *pb.add(k) = ll[i];
                }
                k += 1;
            }
            i += 1;
        }
    }
    unsafe {
        *ps.add(k) = n as u32;
        runs.start.set_len(k + 1);
        runs.byte.set_len(k);
    }
}

#[derive(Default)]
struct Move {
    /// the intervals in F order, then a sentinel
    rec: Vec<u64>,
    /// interval starts (F order), then n
    pp: Vec<u32>,
    /// the interval of each segment start
    start_x: Vec<u32>,
    fpos: Vec<u32>,
    first: Vec<u32>,
    cnt: Vec<u32>,
    bitmap: Vec<u64>,
}

impl Move {
    fn intervals(&self) -> usize {
        self.rec.len() - 1
    }
    fn bytes(&self) -> usize {
        self.intervals() * 8
    }
}

/// The pieces of the F interval [p, p + len): cut at every `cap` rows from p, and around
/// every row of `starts` (sorted, all in the range) as a one-row piece. `f(start, len, si)`
/// for each, si the index into `starts` of a start's piece
#[inline(always)]
fn pieces(p: u32, len: u32, cap: u32, starts: &[u32], mut f: impl FnMut(u32, u32, Option<usize>)) {
    let end = p + len;
    let mut cur = p;
    let mut si = 0;
    while cur < end {
        let grid = (p + ((cur - p) / cap + 1) * cap).min(end);
        if si < starts.len() && starts[si] == cur {
            f(cur, 1, Some(si));
            si += 1;
            cur += 1;
            continue;
        }
        let stop = if si < starts.len() && starts[si] < grid {
            starts[si]
        } else {
            grid
        };
        f(cur, stop - cur, None);
        cur = stop;
    }
}

/// Whether the bitmap has a row of [p, end) set
#[inline(always)]
fn any_set(bm: &[u64], p: u32, end: u32) -> bool {
    let (a, b) = (p as usize >> 6, (end as usize - 1) >> 6);
    let lo = !0u64 << (p & 63);
    let hi = !0u64 >> (63 - ((end - 1) & 63));
    if a == b {
        return bm[a] & lo & hi != 0;
    }
    if bm[a] & lo != 0 || bm[b] & hi != 0 {
        return true;
    }
    bm[a + 1..b].iter().any(|&w| w != 0)
}

/// The capped move structure of psi from L's runs, in three passes over the runs in L
/// order (no pass in F order, whose reads would be scattered over every run array):
/// - A: each run's F position (a counting sort's running sums) and its piece count;
/// - B: each run's pieces written at its bucket's next piece index (256 sequential write
///   streams);
/// - C: each piece's image, increasing in L order, merged against the interval starts
///   for idx and d.
fn build_move(n: usize, runs: &Runs, counts: &[u32; 256], cap: u32, starts: &[u32], m: &mut Move) {
    assert!(cap >= 1 && cap < 1 << 16);
    let t0 = Instant::now();
    let r = runs.byte.len();
    let mut ftab = [0u32; 256];
    let mut cum = 0;
    for c in 0..256 {
        ftab[c] = cum;
        cum += counts[c];
    }
    m.bitmap.clear();
    m.bitmap.resize(n.div_ceil(64), 0);
    for &s in starts {
        m.bitmap[s as usize >> 6] |= 1 << (s & 63);
    }
    let in_range = |p: u32, end: u32| {
        let a = starts.partition_point(|&s| s < p);
        let b = a + starts[a..].partition_point(|&s| s < end);
        (a, &starts[a..b])
    };
    // A
    m.fpos.clear();
    m.fpos.resize(r, 0);
    m.cnt.clear();
    m.cnt.resize(r, 0);
    let mut xc = [0u32; 256];
    for j in 0..r {
        let c = runs.byte[j] as usize;
        let len = runs.start[j + 1] - runs.start[j];
        let p = ftab[c];
        ftab[c] += len;
        m.fpos[j] = p;
        let pc = if any_set(&m.bitmap, p, p + len) {
            let mut k = 0;
            pieces(p, len, cap, in_range(p, p + len).1, |_, _, _| k += 1);
            k
        } else {
            len.div_ceil(cap)
        };
        m.cnt[j] = pc;
        xc[c] += pc;
    }
    let t1 = Instant::now();
    let mut xtab = [0u32; 256];
    let mut total = 0u32;
    for c in 0..256 {
        xtab[c] = total;
        total += xc[c];
    }
    let total = total as usize;
    assert!(total < 1 << 23);
    // B
    m.rec.clear();
    m.rec.resize(total + 1, 0);
    m.pp.clear();
    m.pp.resize(total + 1, 0);
    m.first.clear();
    m.first.resize(r, 0);
    m.start_x.clear();
    m.start_x.resize(starts.len(), 0);
    {
        let (rec, pp, start_x) = (&mut m.rec, &mut m.pp, &mut m.start_x);
        for j in 0..r {
            let c = runs.byte[j] as usize;
            let x0 = xtab[c] as usize;
            xtab[c] += m.cnt[j];
            m.first[j] = x0 as u32;
            let p = m.fpos[j];
            let len = runs.start[j + 1] - runs.start[j];
            let byte = (c as u64) << 24;
            if any_set(&m.bitmap, p, p + len) {
                let (a, sub) = in_range(p, p + len);
                let mut x = x0;
                pieces(p, len, cap, sub, |cur, l, si| {
                    pp[x] = cur;
                    let mut e = byte | (l as u64) << 48;
                    if let Some(si) = si {
                        start_x[a + si] = x as u32;
                        e |= FLAG;
                    }
                    rec[x] = e;
                    x += 1;
                });
            } else {
                let mut cur = p;
                let end = p + len;
                let mut x = x0;
                while cur < end {
                    let l = cap.min(end - cur);
                    pp[x] = cur;
                    rec[x] = byte | (l as u64) << 48;
                    cur += l;
                    x += 1;
                }
            }
        }
    }
    let t2 = Instant::now();
    m.pp[total] = n as u32;
    // the sentinel: R[x + 1] readable for every interval x, never moved onto
    m.rec[total] = 0xffffu64 << 48;
    // C
    let pp = &m.pp;
    let rec = &mut m.rec;
    let mut y = 0usize;
    for j in 0..r {
        let x0 = m.first[j] as usize;
        let q0 = runs.start[j];
        let p0 = pp[x0];
        for x in x0..x0 + m.cnt[j] as usize {
            let q = q0 + (pp[x] - p0);
            while pp[y + 1] <= q {
                y += 1;
            }
            rec[x] |= y as u64 | ((q - pp[y]) as u64) << 32;
        }
    }
    let t3 = Instant::now();
    unsafe {
        PH[0] += (t1 - t0).as_secs_f64();
        PH[1] += (t2 - t1).as_secs_f64();
        PH[2] += (t3 - t2).as_secs_f64();
    }
}
static mut PH: [f64; 3] = [0.0; 3];

/// The record a lane's state s reads, s moved on to the interval holding its row. With BL,
/// the first fast-forward is a select between R[x] and R[x + 1] (a sentinel ends `rec`),
/// the rest a loop; ff counts [fast-forwards, steps with >= 1, steps with >= 2]
#[inline(always)]
fn norm<const BL: bool, const COUNT: bool>(rr: *const u64, s: &mut u64, ff: &mut [u64; 3]) -> u64 {
    let x = *s as u32 as usize;
    let mut r = unsafe { *rr.add(x) };
    let mut k = 0u64;
    if BL {
        let r1 = unsafe { *rr.add(x + 1) };
        let lh = r_len_hi(r);
        let c = *s >= lh;
        *s -= std::hint::select_unpredictable(c, lh - 1, 0);
        r = std::hint::select_unpredictable(c, r1, r);
        k = c as u64;
    }
    while *s >= r_len_hi(r) {
        *s -= r_len_hi(r) - 1;
        r = unsafe { *rr.add(*s as u32 as usize) };
        k += 1;
    }
    if COUNT {
        ff[0] += k;
        ff[1] += (k >= 1) as u64;
        ff[2] += (k >= 2) as u64;
    }
    r
}

/// The state of row `row`
fn state_of(m: &Move, row: u32) -> u64 {
    let x = m.pp.partition_point(|&p| p <= row) - 1;
    ((row - m.pp[x]) as u64) << 32 | x as u64
}

/// One chain over the whole cycle from orig_ptr, n steps
#[inline(never)]
fn walk_chain<const BL: bool, const COUNT: bool>(
    m: &Move,
    orig_ptr: u32,
    n: usize,
    out: &mut Vec<u8>,
) -> [u64; 3] {
    out.clear();
    out.reserve(n);
    let rr = m.rec.as_ptr();
    let mut s = state_of(m, orig_ptr);
    let o = out.as_mut_ptr();
    let mut ff = [0u64; 3];
    for i in 0..n {
        let r = norm::<BL, COUNT>(rr, &mut s, &mut ff);
        unsafe { *o.add(i) = r_byte(r) };
        s = (s & HI) + (r & D_IDX);
    }
    unsafe { out.set_len(n) };
    ff
}

#[derive(Clone, Copy)]
struct Lane {
    s: u64,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

#[derive(Default)]
struct MScratch {
    bufs: Vec<Vec<u8>>,
    next: Vec<usize>,
}

/// The room walk over the move structure: L lanes, segments from `starts` (sorted rows,
/// orig_ptr among them), passes of `room` rounds with a shared offset, as
/// `bwt::walk_segments_room_with`. Returns the fast-forwards (if COUNT)
#[inline(never)]
fn walk_room_move<const L: usize, const BL: bool, const COUNT: bool, const PF: bool>(
    m: &Move,
    starts: &[u32],
    orig_ptr: u32,
    n: usize,
    out: &mut Vec<u8>,
    s: &mut MScratch,
) -> [u64; 3] {
    out.clear();
    let ns = starts.len();
    let cap = 2 * n / ns + 64;
    s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
    for b in &mut s.bufs[..ns] {
        b.clear();
        b.reserve(cap);
    }
    s.next.clear();
    s.next.resize(ns, usize::MAX);
    let start_x = &m.start_x;
    let seg_of = |s: u64| start_x.binary_search(&(s as u32)).unwrap();
    let rr = m.rec.as_ptr();
    let bufs = &mut s.bufs;
    let next = &mut s.next;
    let mut ff = [0u64; 3];

    // A lane takes a segment: its first step, from its start's own one-row interval
    let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
        let b = &mut bufs[seg];
        let base = b.as_mut_ptr();
        let r = unsafe { *rr.add(start_x[seg] as usize) };
        unsafe { *base = r_byte(r) };
        Lane {
            s: r & D_IDX,
            seg,
            p: unsafe { base.add(1) },
            end: unsafe { base.add(b.capacity()) },
        }
    };
    let finish = |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
        let b = &mut bufs[l.seg];
        unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
        next[l.seg] = seg_of(l.s);
    };
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
        s: 0,
        seg: usize::MAX,
        p: std::ptr::null_mut(),
        end: std::ptr::null_mut(),
    });
    let mut active = 0;
    while active < L && queued < ns {
        lanes[active] = take(queued, bufs);
        if PF {
            prefetch(rr.wrapping_add(lanes[active].s as u32 as usize));
        }
        queued += 1;
        active += 1;
    }
    if active == L {
        'full: loop {
            let mut room = cap - 1;
            for l in lanes.iter_mut() {
                if l.p == l.end {
                    grow(l, bufs);
                }
                room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
            }
            let mut base: [*mut u8; L] = std::array::from_fn(|k| lanes[k].p);
            for o in 0..room {
                for k in 0..L {
                    let l = &mut lanes[k];
                    let mut st = l.s;
                    let r = norm::<BL, COUNT>(rr, &mut st, &mut ff);
                    if r & FLAG != 0 {
                        l.s = st;
                        l.p = base[k].wrapping_add(o);
                        finish(l, bufs, next);
                        if queued == ns {
                            for (j, lj) in lanes.iter_mut().enumerate() {
                                if j < k {
                                    lj.p = base[j].wrapping_add(o + 1);
                                } else if j > k {
                                    lj.p = base[j].wrapping_add(o);
                                }
                            }
                            break 'full;
                        }
                        *l = take(queued, bufs);
                        if PF {
                            prefetch(rr.wrapping_add(l.s as u32 as usize));
                        }
                        queued += 1;
                        base[k] = l.p.wrapping_sub(o + 1);
                        continue;
                    }
                    st = (st & HI) + (r & D_IDX);
                    l.s = st;
                    if PF {
                        prefetch(rr.wrapping_add(st as u32 as usize));
                    }
                    unsafe { *base[k].wrapping_add(o) = r_byte(r) };
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
        .filter(|l| l.seg != usize::MAX && next[l.seg] == usize::MAX)
        .collect();
    for l in live.iter_mut() {
        if l.p == l.end {
            grow(l, bufs);
        }
    }
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            let mut st = l.s;
            let r = norm::<BL, COUNT>(rr, &mut st, &mut ff);
            if r & FLAG != 0 {
                l.s = st;
                finish(l, bufs, next);
                live.swap_remove(i);
                continue;
            }
            st = (st & HI) + (r & D_IDX);
            l.s = st;
            if PF {
                prefetch(rr.wrapping_add(st as u32 as usize));
            }
            unsafe {
                *l.p = r_byte(r);
                l.p = l.p.add(1);
            }
            if l.p == l.end {
                grow(l, bufs);
            }
            i += 1;
        }
    }
    out.reserve(n);
    let mut seg = starts.binary_search(&orig_ptr).unwrap();
    while out.len() < n {
        let b = &bufs[seg];
        out.extend_from_slice(&b[..b.len().min(n - out.len())]);
        seg = next[seg];
    }
    ff
}

/// `m` pseudo-random rows below `n` (bwt.rs's xorshift)
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

fn best(repeats: usize, mut f: impl FnMut()) -> f64 {
    (0..repeats)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64()
        })
        .fold(f64::INFINITY, f64::min)
}

#[derive(Default, Clone)]
struct CapStats {
    scan: f64,
    build_runs_seg: f64,
    build_seg: f64, // scan + build, with the segment starts
    build_runs_chain: f64,
    chain: [f64; 2], // [loop, select]
    room: [f64; 4],  // [32 lanes loop, 32 select, 24 select, 16 select]
    intervals_seg: u64,
    intervals_chain: u64,
    bytes_seg: u64,
    ff_chain: [u64; 3],
    ff_room: [u64; 3],
    max_block_bytes: u64,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .expect("usage: movest FILE.bz2 [REPEATS] [GHZ] [tm|mt] [CAPS]");
    let repeats: usize = args.get(2).map_or(3, |s| s.parse().unwrap());
    let ghz: f64 = args.get(3).map_or(2.8, |s| s.parse().unwrap());
    let move_first = args.get(4).is_some_and(|s| s == "mt");
    let caps: Vec<u64> = args
        .get(5)
        .map_or("2,4,8", |s| s.as_str())
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let data = std::fs::read(path).unwrap();
    let k = Kernels::detect();

    let mut blocks = Vec::new();
    let mut sc = Scanner::new(SliceSource(&data));
    let limit: usize = std::env::var("MOVEST_BLOCKS").map_or(usize::MAX, |s| s.parse().unwrap());
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        blocks.push(symbols);
        if blocks.len() == limit {
            break;
        }
    }

    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let (mut tt, mut seg) = (TVector::new(), SegScratch::default());
    let (mut refo, mut mout, mut dec) = (Vec::new(), Vec::new(), Vec::new());
    let mut runs = Runs::default();
    let mut mv = Move::default();
    let mut ms = MScratch::default();
    let mut starts = Vec::new();
    let (mut t_build, mut t_walk, mut t_room) = (0.0, 0.0, 0.0);
    let mut cs = vec![CapStats::default(); caps.len()];
    let (mut rows, mut decoded, mut nblocks, mut nruns, mut skipped) = (0u64, 0u64, 0, 0u64, 0u64);
    let (mut multicycle, mut handoffs) = (0, 0u64);
    for b in &blocks {
        mtf::mtf(b, &mut ll, &mut counts, k).unwrap();
        let n = ll.len();
        if n < 1 << 16 {
            skipped += n as u64;
            continue;
        }
        nblocks += 1;
        rows += n as u64;
        // the reference, untimed
        build(&ll, &counts, &mut tt);
        walk(&tt, b.orig_ptr, &mut refo);
        macro_rules! today {
            () => {
                t_build += best(repeats, || build(&ll, &counts, &mut tt));
                t_walk += best(repeats, || walk(&tt, b.orig_ptr, &mut mout));
                assert!(mout == refo);
                t_room += best(repeats, || {
                    walk_segments_room(&mut tt, b.orig_ptr, &mut mout, &mut seg)
                });
                assert!(mout == refo, "the room walk differs");
            };
        }
        if !move_first {
            today!();
        }
        random_rows(n, 511, &mut starts);
        starts.push(b.orig_ptr);
        starts.sort_unstable();
        starts.dedup();
        handoffs += starts.len() as u64;
        for (ci, &c) in caps.iter().enumerate() {
            let st = &mut cs[ci];
            scan_runs(&ll, &mut runs);
            let r = runs.byte.len() as u64;
            if ci == 0 {
                nruns += r;
            }
            let cap = ((c * n as u64) / r).clamp(1, (1 << 16) - 1) as u32;
            st.scan += best(repeats, || scan_runs(&ll, &mut runs));
            // (1) the chain: no segment starts
            st.build_runs_chain +=
                best(repeats, || build_move(n, &runs, &counts, cap, &[], &mut mv));
            st.intervals_chain += mv.intervals() as u64;
            st.chain[0] += best(repeats, || {
                walk_chain::<false, false>(&mv, b.orig_ptr, n, &mut mout);
            });
            assert!(mout == refo, "move chain differs, block of {n}, c {c}");
            st.chain[1] += best(repeats, || {
                walk_chain::<true, false>(&mv, b.orig_ptr, n, &mut mout);
            });
            assert!(
                mout == refo,
                "move chain (select) differs, block of {n}, c {c}"
            );
            let f = walk_chain::<false, true>(&mv, b.orig_ptr, n, &mut mout);
            for i in 0..3 {
                st.ff_chain[i] += f[i];
            }
            if ci == 0 {
                // the cycle check: the chain from orig_ptr is back at it first after n steps
                let s0 = state_of(&mv, b.orig_ptr);
                let mut s = s0;
                let mut back = 0;
                let mut f = [0; 3];
                for _ in 0..n {
                    let r = norm::<false, false>(mv.rec.as_ptr(), &mut s, &mut f);
                    if s == s0 {
                        back += 1;
                    }
                    s = (s & HI) + (r & D_IDX);
                }
                if back != 1 {
                    multicycle += 1;
                }
            }
            // (2) the segmented walk: the starts are one-row intervals
            st.build_runs_seg += best(repeats, || {
                build_move(n, &runs, &counts, cap, &starts, &mut mv)
            });
            st.build_seg += best(repeats, || {
                scan_runs(&ll, &mut runs);
                build_move(n, &runs, &counts, cap, &starts, &mut mv)
            });
            st.intervals_seg += mv.intervals() as u64;
            st.bytes_seg += mv.bytes() as u64;
            if n >= 899_000 {
                st.max_block_bytes = st.max_block_bytes.max(mv.bytes() as u64);
            }
            macro_rules! room_arm {
                ($i:expr, $l:expr, $bl:expr) => {
                    st.room[$i] += best(repeats, || {
                        walk_room_move::<$l, $bl, false, true>(
                            &mv, &starts, b.orig_ptr, n, &mut mout, &mut ms,
                        );
                    });
                    assert!(
                        mout == refo,
                        "move room walk {} differs, block of {n}, c {c}",
                        $i
                    );
                };
            }
            room_arm!(0, 32, false);
            room_arm!(1, 32, true);
            room_arm!(2, 24, true);
            room_arm!(3, 16, true);
            let f = walk_room_move::<32, false, true, true>(
                &mv, &starts, b.orig_ptr, n, &mut mout, &mut ms,
            );
            for i in 0..3 {
                st.ff_room[i] += f[i];
            }
        }
        if move_first {
            today!();
        }
        dec.clear();
        rle::unrle(&refo, &mut dec, k).unwrap();
        assert_eq!(mibz2::crc::crc32(&dec, k), b.crc, "a block's CRC");
        decoded += dec.len() as u64;
    }

    let cb = |s: f64| s * ghz * 1e9 / decoded as f64;
    #[allow(static_mut_refs)]
    unsafe {
        eprintln!(
            "build phases (all calls): A {:.3} B {:.3} C {:.3} s",
            PH[0], PH[1], PH[2]
        );
    }
    let pct = |a: f64, b: f64| 100.0 * (a / b - 1.0);
    println!(
        "{path}: {nblocks} blocks of >= 65,536 rows ({skipped} rows in smaller blocks skipped), {:.1} M rows -> {:.1} MB decoded; r/n {:.4}; order {}; c/B of decoded output at {ghz} GHz, best of {repeats} a block, summed",
        rows as f64 / 1e6,
        decoded as f64 / 1e6,
        nruns as f64 / rows as f64,
        if move_first { "move first" } else { "T-vector first" }
    );
    println!(
        "  blocks whose chain from orig_ptr is not one cycle: {multicycle}; segment handoffs {handoffs} ({:.0} a block)",
        handoffs as f64 / nblocks as f64
    );
    let base = t_build + t_room;
    println!("  T-vector build                    {:6.3}", cb(t_build));
    println!("  T-vector walk, plain chain        {:6.3}", cb(t_walk));
    println!("  T-vector walk, room               {:6.3}", cb(t_room));
    println!("  T-vector build + room             {:6.3}", cb(base));
    for (ci, &c) in caps.iter().enumerate() {
        let st = &cs[ci];
        println!("  c = {c}:");
        println!(
            "    intervals/n {:.4} (chain), {:.4} (segmented: +{:.0} a block for the starts); {:.3} B/row; largest 900k-row block {:.0} kB",
            st.intervals_chain as f64 / rows as f64,
            st.intervals_seg as f64 / rows as f64,
            (st.intervals_seg - st.intervals_chain) as f64 / nblocks as f64,
            st.bytes_seg as f64 / rows as f64,
            st.max_block_bytes as f64 / 1e3
        );
        for (name, f) in [("chain", st.ff_chain), ("room ", st.ff_room)] {
            println!(
                "    fast-forwards a step, {name}: {:.4} (steps with >= 1: {:.4}, >= 2: {:.4})",
                f[0] as f64 / rows as f64,
                f[1] as f64 / rows as f64,
                f[2] as f64 / rows as f64
            );
        }
        let bestroom = st.room.iter().copied().fold(f64::INFINITY, f64::min);
        println!("    run scan of L                   {:6.3}", cb(st.scan));
        println!(
            "    build from runs, no starts      {:6.3}",
            cb(st.build_runs_chain)
        );
        println!(
            "    build from runs, 512 starts     {:6.3}",
            cb(st.build_runs_seg)
        );
        println!(
            "    move build (scan + runs)        {:6.3}  vs T-vector build {:+.1}%",
            cb(st.build_seg),
            pct(st.build_seg, t_build)
        );
        println!(
            "    (1) move chain, loop            {:6.3}  vs plain chain {:+.1}%",
            cb(st.chain[0]),
            pct(st.chain[0], t_walk)
        );
        println!(
            "    (1) move chain, select          {:6.3}  vs plain chain {:+.1}%",
            cb(st.chain[1]),
            pct(st.chain[1], t_walk)
        );
        for (i, name) in [
            "32 lanes, loop  ",
            "32 lanes, select",
            "24 lanes, select",
            "16 lanes, select",
        ]
        .iter()
        .enumerate()
        {
            println!(
                "    (2) move room, {name} {:6.3}  vs room {:+.1}%",
                cb(st.room[i]),
                pct(st.room[i], t_room)
            );
        }
        println!(
            "    (3) move build + best room      {:6.3}  vs build + room {:+.1}%   (criterion: <= -10%)",
            cb(st.build_seg + bestroom),
            pct(st.build_seg + bestroom, base)
        );
        println!(
            "    runs build + best room          {:6.3}  vs build + room {:+.1}%   (the run list given, as if from MTF)",
            cb(st.build_runs_seg + bestroom),
            pct(st.build_runs_seg + bestroom, base)
        );
    }
}
