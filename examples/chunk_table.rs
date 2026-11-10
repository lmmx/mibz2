//! The chunked link table (docs/journal/2026-11-10-chunked-link.md): the inverse BWT's table
//! with each link's low byte kept a row and its upper bits kept once a stretch. A check, its
//! size, and its step in the same loop as the T-vector's. Nothing in the decoder calls it.
//!
//! ```text
//! chunk_table FILE.bz2 [EVERY] [ROUNDS] [ARM]    ARM: both (default), t, g
//! ```
//!
//! **The table.** Row y of the T-vector links to row Ψ(y). The rows of one first-column byte
//! (a bucket) have increasing links, so consecutive rows of a bucket share their links' upper
//! bits for as long as the links stay inside one chunk of 256 rows: a *stretch*. The table is
//! - `low[y]`: the link's low byte, one byte a row;
//! - `bits`: one bit a row, set where a stretch begins, with the count of set bits before
//!   each word of 64 (`base`);
//! - `rec[k]`: the k-th stretch's upper bits and its bucket's byte, `link >> 8 << 8 | byte`.
//!
//! A step from row y: k = base[y / 64] + (the set bits of its word up to y) - 1, then
//! Ψ(y) = (rec[k] >> 8) << 8 | low[y], and the byte is rec[k]'s. One rule for every row: no
//! overflow, no flag, no second kind of step.
//!
//! **What it checks,** on every counted block: every row's link and byte against the
//! T-vector's, and the walks' bytes against `bwt::walk`'s.
//!
//! **What it prints:** the stretches a row and the table's bytes a row, and the time a row of
//! four walks, two tables in each of two loops: one lane, and 32 lanes each prefetching its
//! next row's lines. The loops are the same for both tables, so the difference is the step's.
//! The timings are local and indicative (RULES C5).

use std::time::Instant;

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

const LANES: usize = 32;

struct Chunked {
    low: Vec<u8>,
    bits: Vec<u64>,
    base: Vec<u32>,
    rec: Vec<u32>,
}

impl Chunked {
    /// From the T-vector and the byte counts (the streaming fill is in the journal entry; this
    /// one is the definition)
    fn from_t(t: &[u32], counts: &[u32; 256]) -> Chunked {
        let n = t.len();
        let mut g = Chunked {
            low: Vec::with_capacity(n + 64),
            bits: vec![0; n / 64 + 2],
            base: vec![0; n / 64 + 2],
            rec: Vec::new(),
        };
        let mut y = 0usize;
        for (c, &k) in counts.iter().enumerate() {
            let mut prev = u32::MAX;
            for _ in 0..k {
                let link = t[y] >> 8;
                g.low.push(link as u8);
                if link >> 8 != prev {
                    prev = link >> 8;
                    g.bits[y / 64] |= 1 << (y % 64);
                    g.rec.push(prev << 8 | c as u32);
                }
                y += 1;
            }
        }
        let mut sum = 0u32;
        for w in 0..g.bits.len() {
            g.base[w] = sum;
            sum += g.bits[w].count_ones();
        }
        g.low.resize(n + 64, 0);
        g
    }

    /// Row y's link and byte
    #[inline(always)]
    fn step(&self, y: usize) -> (usize, u8) {
        // SAFETY: y is a row, below n; k is below the stretches (row 0 begins one)
        unsafe {
            let w = *self.bits.get_unchecked(y >> 6) & (!0u64 >> (63 - (y & 63)));
            let k = *self.base.get_unchecked(y >> 6) + w.count_ones() - 1;
            let r = *self.rec.get_unchecked(k as usize);
            (
                ((r >> 8) << 8) as usize | *self.low.get_unchecked(y) as usize,
                r as u8,
            )
        }
    }

    fn bytes(&self, n: usize) -> usize {
        n + n.div_ceil(64) * 12 + self.rec.len() * 4
    }
}

#[inline(always)]
fn pf<T>(p: *const T) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: a prefetch reads nothing
    unsafe {
        std::arch::x86_64::_mm_prefetch(p as *const i8, std::arch::x86_64::_MM_HINT_T0)
    };
    #[cfg(not(target_arch = "x86_64"))]
    let _ = p;
}

#[inline(never)]
fn walk_t_one(t: &[u32], start: usize, out: &mut [u8]) {
    let mut y = start;
    for o in out.iter_mut() {
        // SAFETY: every link is below n
        let q = unsafe { *t.get_unchecked(y) };
        *o = q as u8;
        y = (q >> 8) as usize;
    }
}

#[inline(never)]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "popcnt,bmi1,bmi2"))]
unsafe fn walk_g_one(g: &Chunked, start: usize, out: &mut [u8]) {
    let mut y = start;
    for o in out.iter_mut() {
        let (next, b) = g.step(y);
        *o = b;
        y = next;
    }
}

/// 32 lanes in turn, lane j writing the j-th slice of `out` from row `starts[j]`. (The
/// chunked walks are compiled with popcnt: without it the step's count is a call's worth of
/// shifts and masks, 32 instructions a row against 15.)
#[inline(never)]
fn walk_t_lanes(t: &[u32], starts: &[usize; LANES], out: &mut [u8]) {
    let len = out.len() / LANES;
    let mut rows = *starts;
    let p = out.as_mut_ptr();
    for i in 0..len {
        for (j, y) in rows.iter_mut().enumerate() {
            // SAFETY: links are below n; lane j writes inside its slice
            unsafe {
                let q = *t.get_unchecked(*y);
                *p.add(j * len + i) = q as u8;
                *y = (q >> 8) as usize;
                pf(t.as_ptr().add(*y));
            }
        }
    }
}

#[inline(never)]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "popcnt,bmi1,bmi2"))]
unsafe fn walk_g_lanes(g: &Chunked, starts: &[usize; LANES], out: &mut [u8]) {
    let len = out.len() / LANES;
    let mut rows = *starts;
    let p = out.as_mut_ptr();
    for i in 0..len {
        for (j, y) in rows.iter_mut().enumerate() {
            let (next, b) = g.step(*y);
            // SAFETY: lane j writes inside its slice; the prefetches read nothing
            unsafe {
                *p.add(j * len + i) = b;
                pf(g.low.as_ptr().add(next));
                pf(g.bits.as_ptr().add(next >> 6));
            }
            *y = next;
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1]).expect("read");
    let every: usize = args.get(2).map_or(1, |s| s.parse().unwrap());
    let rounds: usize = args.get(3).map_or(5, |s| s.parse().unwrap());
    let arm = args.get(4).map_or("both", |s| s.as_str());
    let k = Kernels::detect();
    #[cfg(target_arch = "x86_64")]
    assert!(
        is_x86_feature_detected!("popcnt") && is_x86_feature_detected!("bmi2"),
        "the chunked walks are compiled for popcnt and bmi"
    );
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut ll, mut counts, mut tt) = (Vec::new(), [0u32; 256], TVector::new());
    let (mut syms, mut want) = (Vec::new(), Vec::new());
    let (mut blocks, mut rows, mut stretches, mut bytes, mut seen) = (0usize, 0usize, 0usize, 0, 0);
    // ns, summed over blocks, of each walk's fastest round
    let mut ns = [0f64; 4];
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        let orig = b.orig_ptr as usize;
        syms = b.syms;
        let n = ll.len();
        seen += 1;
        if n < 100_000 || (seen - 1) % every != 0 {
            continue;
        }
        build(&ll, &counts, &mut tt);
        // the T-vector, by its definition (the decoder's own is private to it; `walk`'s
        // bytes below check this one): slot s holds its row's last-column byte and its link
        let t = &{
            let mut next = [0u32; 256];
            let mut cum = 0u32;
            for c in 0..256 {
                next[c] = cum;
                cum += counts[c];
            }
            let mut t = vec![0u32; n];
            for (i, &c) in ll.iter().enumerate() {
                t[next[c as usize] as usize] = (i as u32) << 8;
                next[c as usize] += 1;
            }
            for (e, &c) in t.iter_mut().zip(&ll) {
                *e |= c as u32;
            }
            t
        };
        let g = Chunked::from_t(t, &counts);
        // every row: the link, and the byte (the first column's, which is the last column's
        // at the row linked to)
        for y in 0..n {
            let (next, byte) = g.step(y);
            assert_eq!(next as u32, t[y] >> 8, "row {y}'s link");
            assert_eq!(byte, t[next] as u8, "row {y}'s byte");
        }
        walk(&tt, b.orig_ptr, &mut want);
        blocks += 1;
        rows += n;
        stretches += g.rec.len();
        bytes += g.bytes(n);

        // `walk` emits the last column's byte of each row after the origin; the chunked step
        // emits the first column's byte of the row it leaves: the same bytes from the origin
        let first = (t[orig] >> 8) as usize;
        let mut got = vec![0u8; n];
        // the lanes' starts: the rows at each slice's first byte
        let len = n / LANES;
        let (mut st_t, mut st_g) = ([0usize; LANES], [0usize; LANES]);
        {
            let (mut yt, mut yg) = (first, orig);
            for i in 0..len * LANES {
                if i % len == 0 {
                    st_t[i / len] = yt;
                    st_g[i / len] = yg;
                }
                yt = (t[yt] >> 8) as usize;
                yg = (t[yg] >> 8) as usize;
            }
        }
        let mut best = [f64::MAX; 4];
        for _ in 0..rounds {
            if arm != "g" {
                let s = Instant::now();
                walk_t_one(t, first, &mut got);
                best[0] = best[0].min(s.elapsed().as_nanos() as f64);
                assert!(got == want, "the T-vector's one lane");
                let s = Instant::now();
                walk_t_lanes(t, &st_t, &mut got[..len * LANES]);
                best[2] =
                    best[2].min(s.elapsed().as_nanos() as f64 * n as f64 / (len * LANES) as f64);
                assert!(
                    got[..len * LANES] == want[..len * LANES],
                    "the T-vector's lanes"
                );
            }
            if arm != "t" {
                let s = Instant::now();
                // SAFETY: main checked the CPU has popcnt and bmi
                unsafe { walk_g_one(&g, orig, &mut got) };
                best[1] = best[1].min(s.elapsed().as_nanos() as f64);
                assert!(got == want, "the chunked table's one lane");
                let s = Instant::now();
                unsafe { walk_g_lanes(&g, &st_g, &mut got[..len * LANES]) };
                best[3] =
                    best[3].min(s.elapsed().as_nanos() as f64 * n as f64 / (len * LANES) as f64);
                assert!(
                    got[..len * LANES] == want[..len * LANES],
                    "the chunked table's lanes"
                );
            }
        }
        for (a, b) in ns.iter_mut().zip(best) {
            if b != f64::MAX {
                *a += b;
            }
        }
    }
    let r = rows as f64;
    println!(
        "{blocks} blocks, {rows} rows: every link and byte checked; {:.4} stretches a row, {:.3} bytes a row ({:.2} MB a 900,000-row block; the T-vector 4, 3.60 MB)",
        stretches as f64 / r,
        bytes as f64 / r,
        bytes as f64 / r * 0.9
    );
    println!(
        "ns a row, the fastest of {rounds} a block: one lane T {:.3}, chunked {:.3} ({:.2}x); {LANES} lanes T {:.3}, chunked {:.3} ({:.2}x)",
        ns[0] / r,
        ns[1] / r,
        ns[1] / ns[0],
        ns[2] / r,
        ns[3] / r,
        ns[3] / ns[2]
    );
}
