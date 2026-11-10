//! The chunked link table (docs/journal/2026-11-10-chunked-link.md): the inverse BWT's table
//! with each link's low byte kept a row and its upper bits kept once a stretch. A check, its
//! size, and its step in the same loop as the T-vector's. Nothing in the decoder calls it.
//!
//! ```text
//! chunk_table FILE.bz2 [EVERY] [ROUNDS] [ARM]    ARM: both (default), t, g, fill
//! ```
//!
//! **The table.** Row y of the T-vector links to row Ψ(y). The rows of one first-column byte
//! (a bucket) have increasing links, so consecutive rows of a bucket share their links' upper
//! bits for as long as the links stay inside one chunk of 256 rows: a *stretch*. The table is
//! - `low[y]`: the link's low byte, one byte a row;
//! - `dir`: one bit a row, set where a stretch begins, with the count of set bits before
//!   each word of 64;
//! - `rec[k]`: the k-th stretch's upper bits and its bucket's byte, `link >> 8 << 8 | byte`.
//!
//! A step from row y: k = (the count before y's word) + (the set bits of its word up to y) - 1, then
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
    /// a 64 rows: the stretch-start bits, and the stretches before the word, less one
    dir: Vec<[u64; 2]>,
    rec: Vec<u32>,
}

impl Chunked {
    /// From the T-vector and the byte counts: the definition (`fill_rows` and `fill_runs` make
    /// the same table from the last column, and are checked against this one)
    fn from_t(t: &[u32], counts: &[u32; 256]) -> Chunked {
        let n = t.len();
        let mut g = Chunked {
            low: Vec::with_capacity(n + 64),
            dir: vec![[0; 2]; n / 64 + 2],
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
                    g.dir[y / 64][0] |= 1 << (y % 64);
                    g.rec.push(prev << 8 | c as u32);
                }
                y += 1;
            }
        }
        let mut sum = 0u64;
        for d in g.dir.iter_mut() {
            d[1] = sum.wrapping_sub(1);
            sum += d[0].count_ones() as u64;
        }
        g.low.resize(n + 64, 0);
        g
    }

    /// Row y's link and byte
    #[inline(always)]
    fn step(&self, y: usize) -> (usize, u8) {
        // SAFETY: y is a row, below n; k is below the stretches (row 0 begins one)
        unsafe {
            let d = self.dir.get_unchecked(y >> 6);
            // the set bits at or below y: the shift takes its count mod 64
            let k = d[1].wrapping_add((d[0] << (63 - (y & 63))).count_ones() as u64);
            let r = *self.rec.get_unchecked(k as usize);
            (
                ((r >> 8) << 8) as usize | *self.low.get_unchecked(y) as usize,
                r as u8,
            )
        }
    }

    fn bytes(&self, n: usize) -> usize {
        // the directory at 12 bytes a word (a u32 count suffices; it is 16 here for one load)
        n + n.div_ceil(64) * 12 + self.rec.len() * 4
    }
}

/// The fills' table and scratch, kept across blocks
#[derive(Default)]
struct Fill {
    low: Vec<u8>,
    dir: Vec<[u64; 2]>,
    /// the stretch records as they arrive, each bucket's in a span of its own
    span: Vec<u32>,
    /// the same, packed in bucket order: the table's `rec`
    rec: Vec<u32>,
}

/// Each bucket's first row and end, and where its records' span starts. A bucket has at most
/// one stretch a chunk and at most one a row, so its span is the lesser of the two.
fn layout(counts: &[u32; 256], n: usize, f: &mut Fill) -> ([u32; 256], [u32; 256], [u32; 256]) {
    let chunks = n.div_ceil(256) as u32;
    let (mut cur, mut lim, mut roff) = ([0u32; 256], [0u32; 256], [0u32; 256]);
    let (mut cum, mut rcum) = (0u32, 0u32);
    for c in 0..256 {
        cur[c] = cum;
        cum += counts[c];
        lim[c] = cum;
        roff[c] = rcum;
        rcum += counts[c].min(chunks);
    }
    if f.low.len() < n + 64 {
        f.low.resize(n + 64, 0);
    }
    if f.span.len() < rcum as usize {
        f.span.resize(rcum as usize, 0);
    }
    f.dir.clear();
    f.dir.resize(n / 64 + 2, [0; 2]);
    (cur, lim, roff)
}

/// The spans packed in bucket order, and the directory's counts
#[inline(always)]
fn pack(f: &mut Fill, roff: &[u32; 256], rcur: &[u32; 256]) {
    f.rec.clear();
    for c in 0..256 {
        f.rec
            .extend_from_slice(&f.span[roff[c] as usize..rcur[c] as usize]);
    }
    let mut sum = 0u64;
    for d in f.dir.iter_mut() {
        d[1] = sum.wrapping_sub(1);
        sum += d[0].count_ones() as u64;
    }
}

/// The table from the last column, a row at a time: one byte written a row, and a bit and a
/// record where the row's chunk is new to its byte. False if the counts are not the column's.
#[inline(never)]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "popcnt,bmi1,bmi2"))]
unsafe fn fill_rows(ll: &[u8], counts: &[u32; 256], f: &mut Fill) -> bool {
    let n = ll.len();
    if counts.iter().map(|&c| c as usize).sum::<usize>() != n {
        return false;
    }
    let (mut cur, lim, roff) = layout(counts, n, f);
    let mut rcur = roff;
    let mut last = [u32::MAX; 256];
    for (i, &c) in ll.iter().enumerate() {
        let c = c as usize;
        let y = cur[c] as usize;
        if y >= lim[c] as usize {
            return false;
        }
        cur[c] += 1;
        let ch = (i >> 8) as u32;
        // SAFETY: y is below its bucket's end, which is at most n; a bucket's records are
        // at most one a chunk and one a row, the length of its span
        unsafe {
            *f.low.get_unchecked_mut(y) = i as u8;
            if last[c] != ch {
                last[c] = ch;
                f.dir.get_unchecked_mut(y >> 6)[0] |= 1 << (y & 63);
                *f.span.get_unchecked_mut(rcur[c] as usize) = ch << 8 | c as u32;
                rcur[c] += 1;
            }
        }
    }
    pack(f, &roff, &rcur);
    true
}

/// The same table a run at a time: a run of the column is consecutive rows of its bucket,
/// whose low bytes count up from the run's first row's, written 16 at a time; it begins a
/// stretch where its chunk is new to its byte, and one more at each chunk it crosses.
#[inline(never)]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "popcnt,bmi1,bmi2"))]
unsafe fn fill_runs(ll: &[u8], counts: &[u32; 256], f: &mut Fill) -> bool {
    let n = ll.len();
    if counts.iter().map(|&c| c as usize).sum::<usize>() != n {
        return false;
    }
    let (mut cur, lim, roff) = layout(counts, n, f);
    let mut rcur = roff;
    let mut last = [u32::MAX; 256];
    let mut i = 0;
    while i < n {
        let b = ll[i];
        // the run's end, a word at a time
        let pat = b as u64 * 0x0101_0101_0101_0101;
        let mut j = i + 1;
        loop {
            if j + 8 <= n {
                let x = u64::from_le_bytes(ll[j..j + 8].try_into().unwrap()) ^ pat;
                if x != 0 {
                    j += (x.trailing_zeros() >> 3) as usize;
                    break;
                }
                j += 8;
            } else {
                while j < n && ll[j] == b {
                    j += 1;
                }
                break;
            }
        }
        let (c, len) = (b as usize, j - i);
        let y = cur[c] as usize;
        if y + len > lim[c] as usize {
            return false;
        }
        cur[c] = (y + len) as u32;
        // SAFETY: rows y..y + len are in the bucket; a 16-byte store past the run stays in
        // the bucket (rows written later) where the test below holds, and otherwise the
        // bytes are written one at a time. The records are within the bucket's span.
        unsafe {
            let lowp = f.low.as_mut_ptr().add(y);
            #[cfg(target_arch = "x86_64")]
            let wide = y + len.next_multiple_of(16) <= lim[c] as usize;
            #[cfg(not(target_arch = "x86_64"))]
            let wide = false;
            if wide {
                #[cfg(target_arch = "x86_64")]
                {
                    use std::arch::x86_64::*;
                    let iota = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
                    let mut o = 0;
                    while o < len {
                        let v = _mm_add_epi8(iota, _mm_set1_epi8((i + o) as i8));
                        _mm_storeu_si128(lowp.add(o) as *mut __m128i, v);
                        o += 16;
                    }
                }
            } else {
                for o in 0..len {
                    *lowp.add(o) = (i + o) as u8;
                }
            }
            let mut mark = |row: usize, ch: u32| {
                f.dir.get_unchecked_mut(row >> 6)[0] |= 1 << (row & 63);
                *f.span.get_unchecked_mut(rcur[c] as usize) = ch << 8 | c as u32;
                rcur[c] += 1;
            };
            let ch = (i >> 8) as u32;
            if last[c] != ch {
                mark(y, ch);
            }
            let mut edge = (i | 255) + 1;
            while edge < j {
                mark(y + (edge - i), (edge >> 8) as u32);
                edge += 256;
            }
            last[c] = ((j - 1) >> 8) as u32;
        }
        i = j;
    }
    pack(f, &roff, &rcur);
    true
}

/// Production's build, in a function of its own for the count
#[inline(never)]
fn count_t_build(ll: &[u8], counts: &[u32; 256], tt: &mut TVector) {
    build(ll, counts, tt);
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
                pf(g.dir.as_ptr().add(next >> 6));
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
    let (mut fill, mut fill_ns) = (Fill::default(), [0f64; 3]);
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        let orig = b.orig_ptr as usize;
        syms = b.syms;
        let n = ll.len();
        seen += 1;
        if n < 100_000 || (seen - 1) % every != 0 {
            continue;
        }
        count_t_build(&ll, &counts, &mut tt);
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
        if arm == "fill" {
            // the two fills against the definition, and their time beside production's build
            let mut best = [f64::MAX; 3];
            for _ in 0..rounds {
                let s = Instant::now();
                count_t_build(&ll, &counts, &mut tt);
                best[0] = best[0].min(s.elapsed().as_nanos() as f64);
                for (which, slot) in [(0, 1), (1, 2)] {
                    let s = Instant::now();
                    // SAFETY: main checked the CPU has popcnt and bmi
                    let ok = unsafe {
                        if which == 0 {
                            fill_rows(&ll, &counts, &mut fill)
                        } else {
                            fill_runs(&ll, &counts, &mut fill)
                        }
                    };
                    best[slot] = best[slot].min(s.elapsed().as_nanos() as f64);
                    assert!(ok, "the fill refused the column's own counts");
                    assert!(
                        fill.low[..n] == g.low[..n] && fill.dir == g.dir && fill.rec == g.rec,
                        "fill {which} is not the table"
                    );
                }
            }
            for (a, b) in fill_ns.iter_mut().zip(best) {
                *a += b;
            }
            continue;
        }

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
    if arm == "fill" {
        println!(
            "the fills equal the table on every block. ns a row, the fastest of {rounds} a block: production's build {:.3}, the fill by rows {:.3} ({:.2}x), by runs {:.3} ({:.2}x)",
            fill_ns[0] / r,
            fill_ns[1] / r,
            fill_ns[1] / fill_ns[0],
            fill_ns[2] / r,
            fill_ns[2] / fill_ns[0]
        );
        return;
    }
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
