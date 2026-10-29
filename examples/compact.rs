//! Research only, not the decoder: the inverse BWT with a compact ψ table, 20 bits a row
//! (2.5 B) where the T-vector takes 32 (lmmx/mibz2#39, #65). The knee is the live
//! T-vectors outgrowing the L3: on the 10-core i9 it sits at 4 × 3.6 MB ≈ 0.7 × 20 MB
//! (docs/journal/2026-10-28-workstation-scaling.md). At 2.5 B a row, six blocks' tables
//! take 13.5 MB, about what four T-vectors take now.
//!
//! ```text
//! compact selftest                         every table and walk against the production walk,
//!                                          on libbzip2-compressed edge cases
//! compact grid FILE.bz2 [ROUNDS] [ARM,...] N blocks at once, one core each: ARM is pN
//!                                          (production), tN (this walk, a 32-bit ψ),
//!                                          cN (packed 20-bit ψ) or bN (libbzip2's
//!                                          split ll16 + ll4)
//! ```
//!
//! **The F formulation** (as #42's move structure used it): the walk emits F[row] and moves
//! to ψ(row), starting at `orig_ptr`. That is `bwt::walk`'s output, since L[ψ(k)] = F[k].
//! F[row] needs no row of storage. It is the byte whose bucket of the first column holds the
//! row: a 2^5-row window's first byte from a small table (`FTable`, n/32 bytes), then a step
//! past any bucket boundary inside the window. So a table holds only ψ, the T-vector's link
//! without its byte. bzip2 blocks are at most 900,000 rows, under 2^20, so ψ fits 20 bits.
//!
//! **The tables, all filled by the same build** (production's counting sort, with its
//! 16-row run stores: row i's ψ entry goes to its byte's next slot in F order):
//! - `t` (`Psi32`): a u32 a row, 4 B. The control. The same walk and byte recovery as the
//!   compact tables, so the representation's own cost is t against c.
//! - `c` (`Packed20`): 20 bits a row, packed. One unaligned u32 load at byte 5r/2, shifted
//!   by 0 or 4, gives a row's ψ: one random access a step, as the T-vector's. 2.5 B a row.
//! - `b` (`Split20`): libbzip2's small-memory layout (`ll16`, `ll4`, `GET_LL`): 16 low bits a
//!   row in one array, 4 high bits a row packed in another. Two random accesses a step.
//!   2.5 B a row.
//!
//! **The walk** is the cooperative pilot's lane walk on one core: 32 lanes, 512 segments
//! from the room walk's xorshift starts plus `orig_ptr`, each segment's bytes into its own
//! buffer through the lane's own pointer, then the segments joined in the output's order.
//! A segment start's entry is replaced, for the walk, by n + its segment number (codes from
//! n to 2^20 are free in 20 bits), so a lane sees a start in the entry it loads anyway. The
//! start's own ψ is kept beside, where the lane taking it reads it. The production room walk
//! (`p`) also has its passes, which this walk doesn't. So t against p is this walk's own
//! cost, and c against t the representation's.
//!
//! `grid` times, as the cooperative pilot's grid does, the T-vector phase only (build +
//! walk; the scanner and MTF done first, outside the timing). N block workers each take
//! the next block from one counter. With COMPACT_CPUS (bench/compact.sh: one CPU of each
//! physical core), worker w is pinned to the w-th CPU and where it ran is checked. Round -1
//! checks every arm's every block against production's output by hash, untimed. The timed
//! rounds run in an order rotated and reversed by the round. Each is paired with the first
//! arm, with a 95% interval, and appended raw to COMPACT_JSONL when it is set.

#![allow(clippy::needless_range_loop)] // (a research kernel, indexed as written)

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use mibz2::bwt::{build, walk, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

const LANES: usize = 32;
const SEGMENTS: usize = 512;
/// a first-column window's rows, log2 (`FTable`)
const FS: u32 = 5;
/// the largest bzip2 block, in rows: 9 × 100,000
const MAX_ROWS: usize = 900_000;

/// The calling thread pinned to the CPU `cpu`: false if refused, or not Linux
fn pin(cpu: usize) -> bool {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
        }
        let mut mask = [0u64; 16]; // a cpu_set_t: 1024 CPUs
        mask[cpu / 64] |= 1 << (cpu % 64);
        unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) == 0 }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = cpu;
        false
    }
}

/// The CPU the calling thread is running on (-1 where unknown)
fn cpu_now() -> i64 {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn sched_getcpu() -> i32;
        }
        unsafe { sched_getcpu() as i64 }
    }
    #[cfg(not(target_os = "linux"))]
    -1
}

#[inline(always)]
fn prefetch(p: *const u8) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: a prefetch never faults
    unsafe {
        core::arch::x86_64::_mm_prefetch::<{ core::arch::x86_64::_MM_HINT_T0 }>(p as *const i8)
    }
    #[cfg(not(target_arch = "x86_64"))]
    let _ = p;
}

/// A ψ table: a row's link, 20 bits at least, written by the build and read by the walk
trait Psi: Default + Send {
    const NAME: &'static str;
    /// bytes a row
    const BYTES_A_ROW: f64;
    fn resize(&mut self, n: usize);
    fn set(&mut self, r: usize, v: u32);
    fn get(&self, r: usize) -> u32;
    /// a hint that row r is read soon
    fn prefetch(&self, r: usize);
    /// rows k..k + len set to v0, v0 + 1, ...: the build's run stores
    fn set_run(&mut self, k: usize, v0: u32, len: usize) {
        for j in 0..len {
            self.set(k + j, v0 + j as u32);
        }
    }
}

#[derive(Default)]
struct Psi32(Vec<u32>);

impl Psi for Psi32 {
    const NAME: &'static str = "32-bit ψ";
    const BYTES_A_ROW: f64 = 4.0;
    fn resize(&mut self, n: usize) {
        self.0.resize(n, 0);
    }
    #[inline(always)]
    fn set(&mut self, r: usize, v: u32) {
        self.0[r] = v;
    }
    #[inline(always)]
    fn get(&self, r: usize) -> u32 {
        // SAFETY: rows read are below n (the build's links and the starts)
        unsafe { *self.0.get_unchecked(r) }
    }
    #[inline(always)]
    fn prefetch(&self, r: usize) {
        prefetch(self.0.as_ptr().wrapping_add(r) as *const u8);
    }
}

/// 20 bits a row, packed: row r's at bit 20r, read as one unaligned u32 (3 bytes padding)
#[derive(Default)]
struct Packed20(Vec<u8>);

impl Psi for Packed20 {
    const NAME: &'static str = "packed 20-bit ψ";
    const BYTES_A_ROW: f64 = 2.5;
    fn resize(&mut self, n: usize) {
        self.0.resize((5 * n).div_ceil(2) + 4, 0);
    }
    /// byte stores, the shared nibble read-modify-written alone (an unaligned u32 store
    /// overlapping the next row's would stall the next row's load on its forwarding)
    #[inline(always)]
    fn set(&mut self, r: usize, v: u32) {
        let at = (5 * r) >> 1;
        let b = &mut self.0[at..at + 3];
        if r & 1 == 0 {
            b[0] = v as u8;
            b[1] = (v >> 8) as u8;
            b[2] = (b[2] & 0xf0) | ((v >> 16) as u8 & 0xf);
        } else {
            b[0] = (b[0] & 0x0f) | ((v as u8 & 0xf) << 4);
            b[1] = (v >> 4) as u8;
            b[2] = (v >> 12) as u8;
        }
    }
    #[inline(always)]
    fn get(&self, r: usize) -> u32 {
        let (at, sh) = ((5 * r) >> 1, ((r & 1) * 4) as u32);
        // SAFETY: rows read are below n, and the buffer is padded by 4
        let w = unsafe { (self.0.as_ptr().add(at) as *const u32).read_unaligned() };
        (w >> sh) & 0xf_ffff
    }
    #[inline(always)]
    fn prefetch(&self, r: usize) {
        prefetch(self.0.as_ptr().wrapping_add((5 * r) >> 1));
    }
    /// 16 rows packed in a local span (the bits from the first row's nibble on), then stored
    /// whole, its partial first and last bytes merged
    fn set_run(&mut self, k: usize, v0: u32, len: usize) {
        if len != 16 {
            return (0..len).for_each(|j| self.set(k + j, v0 + j as u32));
        }
        let o = (k & 1) * 4;
        let mut span = [0u8; 41];
        let (mut acc, mut bits, mut at) = (0u64, o, 0);
        for j in 0..16 {
            acc |= ((v0 + j as u32) as u64) << bits;
            bits += 20;
            while bits >= 8 {
                span[at] = acc as u8;
                acc >>= 8;
                bits -= 8;
                at += 1;
            }
        }
        if bits > 0 {
            span[at] = acc as u8;
            at += 1;
        }
        let base = (5 * k) >> 1;
        let dst = &mut self.0[base..base + at];
        let last = at - 1;
        let (first, end) = (dst[0], dst[last]);
        dst.copy_from_slice(&span[..at]);
        if o == 4 {
            dst[0] = (first & 0x0f) | span[0];
            // (o = 4: 4 + 320 bits end on a nibble, the last byte's high half another row's)
            dst[last] = (end & 0xf0) | (span[last] & 0x0f);
        }
    }
}

/// libbzip2's small-memory layout: 16 low bits a row (ll16), 4 high bits a row (ll4, two a
/// byte), joined as its GET_LL does
#[derive(Default)]
struct Split20 {
    lo: Vec<u16>,
    hi: Vec<u8>,
}

impl Psi for Split20 {
    const NAME: &'static str = "split 16 + 4-bit ψ (libbzip2)";
    const BYTES_A_ROW: f64 = 2.5;
    fn resize(&mut self, n: usize) {
        self.lo.resize(n, 0);
        self.hi.resize(n.div_ceil(2), 0);
    }
    #[inline(always)]
    fn set(&mut self, r: usize, v: u32) {
        self.lo[r] = v as u16;
        let sh = (r & 1) * 4;
        let b = &mut self.hi[r >> 1];
        *b = (*b & !(0xf << sh)) | (((v >> 16) as u8 & 0xf) << sh);
    }
    #[inline(always)]
    fn get(&self, r: usize) -> u32 {
        // SAFETY: rows read are below n
        unsafe {
            let lo = *self.lo.get_unchecked(r) as u32;
            let hi = (*self.hi.get_unchecked(r >> 1) >> ((r & 1) * 4)) & 0xf;
            lo | (hi as u32) << 16
        }
    }
    #[inline(always)]
    fn prefetch(&self, r: usize) {
        prefetch(self.lo.as_ptr().wrapping_add(r) as *const u8);
        prefetch(self.hi.as_ptr().wrapping_add(r >> 1));
    }
    /// the low halves stored as a slice; the high nibbles a whole byte for each pair of rows,
    /// a nibble merged at an odd start or end
    fn set_run(&mut self, k: usize, v0: u32, len: usize) {
        for (j, x) in self.lo[k..k + len].iter_mut().enumerate() {
            *x = (v0 + j as u32) as u16;
        }
        let hi = |r: usize| ((v0 + (r - k) as u32) >> 16) as u8 & 0xf;
        let (mut r, end) = (k, k + len);
        if r & 1 == 1 {
            let b = &mut self.hi[r >> 1];
            *b = (*b & 0x0f) | (hi(r) << 4);
            r += 1;
        }
        while r + 2 <= end {
            self.hi[r >> 1] = hi(r) | (hi(r + 1) << 4);
            r += 2;
        }
        if r < end {
            let b = &mut self.hi[r >> 1];
            *b = (*b & 0xf0) | hi(r);
        }
    }
}

/// The first column: each bucket's first row, and each 2^FS-row window's first row's byte
#[derive(Default)]
struct FTable {
    cum: Vec<u32>, // 257: cum[c] the first row of byte c's bucket, cum[256] = n
    fast: Vec<u8>,
}

impl FTable {
    fn build(&mut self, counts: &[u32; 256], n: usize) {
        self.cum.clear();
        let mut s = 0u32;
        for &c in counts {
            self.cum.push(s);
            s += c;
        }
        self.cum.push(s);
        self.fast.clear();
        let mut c = 0usize;
        for w in 0..(n >> FS) + 1 {
            let row = (w << FS) as u32;
            while c < 255 && self.cum[c + 1] <= row {
                c += 1;
            }
            self.fast.push(c as u8);
        }
    }

    /// F[r]: the byte whose bucket holds row r (below n)
    #[inline(always)]
    fn byte(&self, r: usize) -> u8 {
        // SAFETY: r < n, so r >> FS indexes fast, and c + 1 stays within cum (c < 256)
        unsafe {
            let mut c = *self.fast.get_unchecked(r >> FS) as usize;
            while *self.cum.get_unchecked(c + 1) as usize <= r {
                c += 1;
            }
            c as u8
        }
    }

    fn bytes(&self) -> usize {
        self.cum.len() * 4 + self.fast.len()
    }
}

/// ψ of the last column `ll` into `t`: row i's link goes to the next slot of its byte's
/// bucket, in F order, with production's 16-row run stores (16 equal bytes are 16
/// consecutive rows of their bucket)
fn build_psi<T: Psi>(ll: &[u8], counts: &[u32; 256], t: &mut T) {
    let n = ll.len();
    t.resize(n);
    let mut ftab = [0u32; 256];
    let mut cum = 0u32;
    for i in 0..256 {
        ftab[i] = cum;
        cum += counts[i];
    }
    let mut i = 0;
    while i + 16 <= n {
        let a = u64::from_ne_bytes(ll[i..i + 8].try_into().unwrap());
        let b = u64::from_ne_bytes(ll[i + 8..i + 16].try_into().unwrap());
        if a == b && a == a.rotate_left(8) {
            let c = ll[i] as usize;
            t.set_run(ftab[c] as usize, i as u32, 16);
            ftab[c] += 16;
        } else {
            for ii in i..i + 16 {
                let c = ll[ii] as usize;
                t.set(ftab[c] as usize, ii as u32);
                ftab[c] += 1;
            }
        }
        i += 16;
    }
    for ii in i..n {
        let c = ll[ii] as usize;
        t.set(ftab[c] as usize, ii as u32);
        ftab[c] += 1;
    }
}

/// The room walk's starts: `m` xorshift rows seeded by n
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

/// The walk's buffers, kept from block to block
#[derive(Default)]
struct Walk {
    starts: Vec<u32>,
    link: Vec<u32>,
    bufs: Vec<Vec<u8>>,
    next: Vec<usize>,
}

struct Lane {
    row: u32,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

/// The block's bytes into `out`: 32 lanes over 512 segments of ψ, F[row] emitted at each
/// row, the segments joined in the output's order (`walk`'s output)
fn walk_psi<T: Psi>(
    t: &mut T,
    f: &FTable,
    orig_ptr: u32,
    n: usize,
    w: &mut Walk,
    out: &mut Vec<u8>,
) {
    out.clear();
    if n == 0 {
        return;
    }
    random_rows(n, SEGMENTS - 1, &mut w.starts);
    w.starts.push(orig_ptr);
    w.starts.sort_unstable();
    w.starts.dedup();
    let ns = w.starts.len();
    assert!(n + ns < 1 << 20, "a block over 2^20 rows less its segments");
    // each start's own ψ kept beside, its entry a code n + segment for the walk
    w.link.clear();
    for (s, &r) in w.starts.iter().enumerate() {
        w.link.push(t.get(r as usize));
        t.set(r as usize, (n + s) as u32);
    }
    let cap = 2 * n / ns + 64;
    w.bufs.resize_with(ns.max(w.bufs.len()), Vec::new);
    for b in &mut w.bufs[..ns] {
        b.clear();
        b.reserve(cap);
    }
    w.next.clear();
    w.next.resize(ns, usize::MAX);
    let (starts, link, bufs, next) = (&w.starts, &w.link, &mut w.bufs, &mut w.next);
    let nu = n as u32;
    // SAFETY (the pointers): a segment is taken once and walked by one lane, which writes
    // only its buffer, growing it before it is full
    let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
        let b = &mut bufs[seg];
        let base = b.as_mut_ptr();
        unsafe { *base = f.byte(starts[seg] as usize) };
        let row = link[seg];
        t.prefetch(row as usize);
        Lane {
            row,
            seg,
            p: unsafe { base.add(1) },
            end: unsafe { base.add(b.capacity()) },
        }
    };
    let mut queued = 0;
    let mut live: Vec<Lane> = Vec::with_capacity(LANES);
    while live.len() < LANES && queued < ns {
        live.push(take(queued, bufs));
        queued += 1;
    }
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            let v = t.get(l.row as usize);
            if v >= nu {
                // the row is a start: the segment ends before it
                let b = &mut bufs[l.seg];
                unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
                next[l.seg] = (v - nu) as usize;
                if queued < ns {
                    *l = take(queued, bufs);
                    queued += 1;
                    i += 1;
                } else {
                    live.swap_remove(i);
                }
                continue;
            }
            if l.p == l.end {
                let b = &mut bufs[l.seg];
                let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
                unsafe { b.set_len(len) };
                b.reserve(b.capacity());
                l.p = unsafe { b.as_mut_ptr().add(len) };
                l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
            }
            unsafe {
                *l.p = f.byte(l.row as usize);
                l.p = l.p.add(1);
            }
            l.row = v;
            t.prefetch(v as usize);
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
}

/// A block's last column, its byte counts and primary index
struct Block {
    ll: Vec<u8>,
    counts: [u32; 256],
    orig_ptr: u32,
}

fn blocks(data: &[u8]) -> Vec<Block> {
    let kern = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut v = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        mtf::mtf(&symbols, &mut ll, &mut counts, kern).unwrap();
        v.push(Block {
            ll,
            counts,
            orig_ptr: symbols.orig_ptr,
        });
    }
    v
}

fn fnv(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| {
        (h ^ x as u64).wrapping_mul(0x100_0000_01b3)
    })
}

/// One block worker's inversion and buffers
enum Inv {
    Prod(TVector, SegScratch),
    T(Psi32, FTable, Walk),
    C(Packed20, FTable, Walk),
    B(Split20, FTable, Walk),
}

impl Inv {
    fn new(kind: char) -> Inv {
        match kind {
            'p' => Inv::Prod(TVector::new(), SegScratch::default()),
            't' => Inv::T(Psi32::default(), FTable::default(), Walk::default()),
            'c' => Inv::C(Packed20::default(), FTable::default(), Walk::default()),
            'b' => Inv::B(Split20::default(), FTable::default(), Walk::default()),
            _ => panic!("arm kinds: p, t, c, b"),
        }
    }

    /// bytes a row of the table the walk reads at random
    fn bytes_a_row(kind: char) -> f64 {
        match kind {
            'p' | 't' => 4.0,
            'c' => Packed20::BYTES_A_ROW,
            _ => Split20::BYTES_A_ROW,
        }
    }

    /// The block's inversion into `out`: the build's and the walk's seconds
    fn invert(&mut self, b: &Block, out: &mut Vec<u8>) -> (f64, f64) {
        fn compact<T: Psi>(
            t: &mut T,
            f: &mut FTable,
            w: &mut Walk,
            b: &Block,
            out: &mut Vec<u8>,
        ) -> (f64, f64) {
            let s = Instant::now();
            build_psi(&b.ll, &b.counts, t);
            f.build(&b.counts, b.ll.len());
            let m = s.elapsed().as_secs_f64();
            walk_psi(t, f, b.orig_ptr, b.ll.len(), w, out);
            (m, s.elapsed().as_secs_f64() - m)
        }
        match self {
            Inv::Prod(tv, seg) => {
                let s = Instant::now();
                build(&b.ll, &b.counts, tv);
                let m = s.elapsed().as_secs_f64();
                walk_segments_room(tv, b.orig_ptr, out, seg);
                (m, s.elapsed().as_secs_f64() - m)
            }
            Inv::T(t, f, w) => compact(t, f, w, b, out),
            Inv::C(t, f, w) => compact(t, f, w, b, out),
            Inv::B(t, f, w) => compact(t, f, w, b, out),
        }
    }

    /// the first-column table's bytes (the compact arms' extra footprint)
    fn aux_bytes(&self) -> usize {
        match self {
            Inv::Prod(..) => 0,
            Inv::T(_, f, _) | Inv::C(_, f, _) | Inv::B(_, f, _) => f.bytes(),
        }
    }
}

/// `selftest`: every table's walk against the production walk on libbzip2-compressed edge
/// cases (block sizes 100 kB and 900 kB; one byte, two, runs, periodic texts whose ψ has
/// many cycles, random and text-like bytes, and a stream of many blocks)
fn selftest() {
    let mut cases: Vec<(String, Vec<u8>)> = vec![
        ("one byte".into(), vec![b'x']),
        ("two bytes".into(), b"ab".to_vec()),
        ("a run of 1 MB".into(), vec![0u8; 1 << 20]),
        ("period 3".into(), b"abc".repeat(300_000)),
        (
            "period 1000".into(),
            (0..1000u32)
                .map(|i| (i * 7 % 251) as u8)
                .collect::<Vec<_>>()
                .repeat(1000),
        ),
    ];
    let mut x = 0x9e37_79b9_7f4a_7c15u64;
    let mut rnd = |m: usize| -> Vec<u8> {
        (0..m)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x as u8
            })
            .collect()
    };
    cases.push(("random 2 MB".into(), rnd(2 << 20)));
    let words = [
        "the ",
        "bwt ",
        "block ",
        "row ",
        "of ",
        "wikidata ",
        "Q42 ",
        "\"label\": ",
    ];
    let text: Vec<u8> = (0..400_000)
        .flat_map(|i| words[(i * 2654435761usize >> 7) % words.len()].bytes())
        .collect();
    cases.push(("text-like 2.4 MB".into(), text));
    let (mut blocks_checked, mut rows) = (0usize, 0usize);
    let mut inv: Vec<(char, Inv)> = "tcb".chars().map(|k| (k, Inv::new(k))).collect();
    let mut tv = TVector::new();
    let (mut want, mut got) = (Vec::new(), Vec::new());
    for (name, data) in &cases {
        for level in [1, 9] {
            let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(level));
            std::io::Write::write_all(&mut e, data).unwrap();
            let z = e.finish().unwrap();
            for b in blocks(&z) {
                build(&b.ll, &b.counts, &mut tv);
                walk(&tv, b.orig_ptr, &mut want);
                for (k, i) in inv.iter_mut() {
                    i.invert(&b, &mut got);
                    assert!(
                        got == want,
                        "{k} differs from the plain walk: {name}, level {level}, {} rows",
                        b.ll.len()
                    );
                }
                blocks_checked += 1;
                rows += b.ll.len();
            }
        }
        println!("ok: {name} (levels 1 and 9)");
    }
    println!("selftest: {blocks_checked} blocks, {rows} rows, every table's walk (t, c, b) the plain walk's, byte for byte");
}

/// The 97.5% quantile of Student's t with `df` degrees of freedom (a 95% two-sided interval)
fn t975(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    match df {
        0 => f64::NAN,
        1..=30 => T[df - 1],
        _ => 1.96 + 2.4 / df as f64,
    }
}

/// The mean of `x` and its 95% half-width
fn interval(x: &[f64]) -> (f64, f64) {
    let n = x.len();
    let m = x.iter().sum::<f64>() / n as f64;
    if n < 2 {
        return (m, f64::NAN);
    }
    let v = x.iter().map(|y| (y - m) * (y - m)).sum::<f64>() / (n - 1) as f64;
    (m, t975(n - 1) * (v / n as f64).sqrt())
}

/// `grid`: the T-vector phase with N blocks in flight, one core each
fn grid(bs: &[Block], rounds: usize, specs: &[String], cpus: Option<&[usize]>) {
    let mut inv = Inv::new('p');
    let mut out = Vec::new();
    let want: Vec<u64> = bs
        .iter()
        .map(|b| {
            inv.invert(b, &mut out);
            fnv(&out)
        })
        .collect();
    drop(inv);
    let bytes: usize = bs.iter().map(|b| b.ll.len()).sum();
    let max_rows = bs.iter().map(|b| b.ll.len()).max().unwrap_or(0);
    assert!(max_rows <= MAX_ROWS, "a block of {max_rows} rows");
    let arms: Vec<(char, usize)> = specs
        .iter()
        .map(|s| (s.as_bytes()[0] as char, s[1..].parse().unwrap()))
        .collect();
    if let Some(c) = cpus {
        for ((_, n), s) in arms.iter().zip(specs) {
            assert!(*n <= c.len(), "{s} needs {n} CPUs, {} given", c.len());
        }
    }
    let mut results: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    let mut phases: Vec<Vec<(f64, f64)>> = vec![Vec::new(); arms.len()];
    let mut aux = vec![0usize; arms.len()];
    let mut misplaced = vec![0usize; arms.len()];
    let mut raw = std::env::var("COMPACT_JSONL").ok().map(|p| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .unwrap()
    });
    for round in -1..rounds as i64 {
        let check = round < 0;
        let r = round.max(0) as usize;
        let mut order: Vec<usize> = (0..arms.len()).collect();
        order.rotate_left(r % arms.len());
        if (r / arms.len()) % 2 == 1 {
            order.reverse();
        }
        for &a in &order {
            let (kind, n) = arms[a];
            let mut workers: Vec<Inv> = (0..n).map(|_| Inv::new(kind)).collect();
            let go = std::sync::Barrier::new(n + 1);
            let next_block = AtomicUsize::new(0);
            let (wall, busy) = std::thread::scope(|sc| {
                let hs: Vec<_> = workers
                    .iter_mut()
                    .enumerate()
                    .map(|(w, inv)| {
                        let (go, want, next_block, cpu) =
                            (&go, &want, &next_block, cpus.map(|c| c[w]));
                        sc.spawn(move || {
                            if let Some(c) = cpu {
                                assert!(pin(c), "could not pin a worker to CPU {c}");
                            }
                            let mut out = Vec::new();
                            let (mut tb, mut tw) = (0.0, 0.0);
                            go.wait();
                            loop {
                                let i = next_block.fetch_add(1, Ordering::Relaxed);
                                if i >= bs.len() {
                                    break;
                                }
                                let (b, w) = inv.invert(&bs[i], &mut out);
                                (tb, tw) = (tb + b, tw + w);
                                if check {
                                    assert_eq!(
                                        fnv(&out),
                                        want[i],
                                        "{kind}: block {i} differs from production"
                                    );
                                }
                            }
                            (tb, tw, cpu.map_or(true, |c| cpu_now() == c as i64))
                        })
                    })
                    .collect();
                go.wait();
                let s = Instant::now();
                let ends: Vec<(f64, f64, bool)> =
                    hs.into_iter().map(|h| h.join().unwrap()).collect();
                (s.elapsed().as_secs_f64(), ends)
            });
            let placed = busy.iter().all(|b| b.2);
            if check {
                assert!(placed, "{}: a worker was seen off its CPU", specs[a]);
                aux[a] = workers.iter().map(|w| w.aux_bytes()).max().unwrap_or(0);
                eprintln!("checked {}: every block's output production's", specs[a]);
                continue;
            }
            misplaced[a] += !placed as usize;
            let tb = busy.iter().map(|b| b.0).sum::<f64>() / n as f64;
            let tw = busy.iter().map(|b| b.1).sum::<f64>() / n as f64;
            let bmax = busy.iter().map(|b| b.0 + b.1).fold(0.0, f64::max);
            let mbs = bytes as f64 / wall / 1e6;
            results[a].push(mbs);
            phases[a].push((tb, tw));
            if let Some(f) = raw.as_mut() {
                use std::io::Write;
                writeln!(
                    f,
                    r#"{{"round": {round}, "arm": "{}", "kind": "{kind}", "workers": {n}, "bytes_a_row": {}, "pinned": {}, "placement_ok": {placed}, "wall_s": {wall:.6}, "mb_s": {mbs:.3}, "build_s_a_worker": {tb:.6}, "walk_s_a_worker": {tw:.6}, "busy_max_s": {bmax:.6}, "unaccounted_s": {:.6}}}"#,
                    specs[a],
                    Inv::bytes_a_row(kind),
                    cpus.is_some(),
                    wall - bmax
                )
                .unwrap();
            }
            eprintln!("round {round} {}: {mbs:.0} MB/s", specs[a]);
        }
    }
    let med = |v: &[f64]| {
        let mut v = v.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    // each kind's own 4-worker arm, if run: the scaling against it
    let own4: Vec<Option<usize>> = arms
        .iter()
        .map(|&(k, _)| arms.iter().position(|&(k2, n2)| k2 == k && n2 == 4))
        .collect();
    let base = &results[0];
    println!(
        "{} blocks, {:.1} MB, {rounds} timed rounds; the T-vector phase only (build + walk), not the decoder; {}",
        bs.len(),
        bytes as f64 / 1e6,
        if cpus.is_some() { "each worker pinned to its own physical core, checked" } else { "workers not pinned" }
    );
    println!(
        "| arm | table | workers | live tables (MB, largest block) | MB/s, median | vs {0}, paired: mean ±95% (min-max) | vs its own 4 workers, paired | build MB/s | walk MB/s | rounds misplaced |",
        specs[0]
    );
    println!("|---|---|--:|--:|--:|--:|--:|--:|--:|--:|");
    for (a, r) in results.iter().enumerate() {
        let (kind, n) = arms[a];
        let ratios: Vec<f64> = r.iter().zip(base).map(|(x, y)| x / y).collect();
        let (mean, h) = interval(&ratios);
        let (lo, hi) = ratios
            .iter()
            .fold((f64::INFINITY, 0f64), |(l, h), &x| (l.min(x), h.max(x)));
        let own = own4[a].map_or("".into(), |b| {
            let x: Vec<f64> = r.iter().zip(&results[b]).map(|(x, y)| x / y).collect();
            let (m, h) = interval(&x);
            format!("{m:.3} ±{h:.3}")
        });
        let mbs = |t: f64| bytes as f64 / t / 1e6;
        let table = match kind {
            'p' => "T-vector (production)",
            't' => Psi32::NAME,
            'c' => Packed20::NAME,
            _ => Split20::NAME,
        };
        let live = n as f64 * (max_rows as f64 * Inv::bytes_a_row(kind) + aux[a] as f64) / 1e6;
        println!(
            "| {} | {table} | {n} | {live:.1} | {:.0} | {mean:.3} ±{h:.3} ({lo:.3}-{hi:.3}) | {own} | {:.0} | {:.0} | {} |",
            specs[a],
            med(r),
            med(&phases[a].iter().map(|p| mbs(p.0)).collect::<Vec<_>>()),
            med(&phases[a].iter().map(|p| mbs(p.1)).collect::<Vec<_>>()),
            misplaced[a]
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("selftest") => selftest(),
        Some("grid") => {
            let data = std::fs::read(args.get(2).expect("compact grid FILE.bz2 ..."))
                .expect("a .bz2 file");
            let rounds = args.get(3).map_or(4, |s| s.parse().unwrap());
            let specs: Vec<String> = args
                .get(4)
                .map_or("p4,p1,t1,c1,b1,t4,c4,b4", |s| s)
                .split(',')
                .map(String::from)
                .collect();
            // COMPACT_CPUS: the CPUs to pin to, one a worker (bench/compact.sh: one of each core)
            let cpus: Option<Vec<usize>> = std::env::var("COMPACT_CPUS")
                .ok()
                .filter(|s| !s.is_empty())
                .map(|s| s.split(',').map(|c| c.parse().unwrap()).collect());
            grid(&blocks(&data), rounds, &specs, cpus.as_deref());
        }
        _ => panic!("compact selftest | compact grid FILE.bz2 [ROUNDS] [ARMS]"),
    }
}
