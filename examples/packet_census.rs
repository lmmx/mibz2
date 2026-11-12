//! The certified-packet census (docs/journal/2026-11-09-packet-census.md): how much of the
//! inverse BWT's output a consumer could copy as certified packets, on a block's actual,
//! marker-capped trajectory. It counts; it times nothing, and nothing in the decoder calls it.
//!
//! ```text
//! packet_census census FILE.bz2 [LABEL]   one JSON line a block (stdout)
//! packet_census synthetic SCALE           the same on the bench's corpora (tests/common, -9)
//! packet_census validate                  every packet checked against a brute-force model
//! ```
//!
//! **The table.** Production's T-vector is T[q] = L[q] | Ψ(q) << 8 (`bwt::build`), and the
//! walk reads states x0 = Ψ(orig_ptr), x_{t+1} = Ψ(x_t), emitting L[x_t] (`bwt::walk`).
//!
//! **Two step models,** each a map σ on rows and the bytes one step emits:
//! - single: σ = Ψ, one byte L(q) a step (production's one-hop walk);
//! - pair: σ = Ψ², two bytes (L(q), L(Ψ(q))) = (L(q), F(q)) a step (the pair convention).
//!
//! **The cells.** A model's cells are the maximal row intervals on which σ(q) − q and the
//! step's bytes are constant. For the pair model they refine Ψ²'s translation cells (at most
//! 2r − 1) by L's boundaries: at most 3r − 2 (the Researcher's bound, 6731804749). The
//! single model's are at most 2r − 1 (Ψ's r translation intervals cut by L's boundaries).
//! Each count is checked against its bound on every block.
//!
//! **A packet** starting at state q: a path of k cells c_0..c_{k-1} (the cells the walk
//! visits from q), its domain H = ∩_j ([u_j, v_j) − s_j) with s_j the deltas' prefix sums,
//! its word W (the cells' bytes) and its displacement Δ = s_k. For q + iΔ in H, i < m, the
//! walk emits W^m and reaches q + mΔ: one descriptor for k·m steps. The copy is capped by
//! the bytes left before the next marker (a segment start the production walk flags, or the
//! block's end), and may stop inside a repetition (W^m's prefix). The census tiles each
//! segment greedily, taking at each packet start the longest packet a model allows.
//!
//! **Every packet counted is checked** against the trajectory it claims to replace: its
//! bytes (from the descriptors alone) against the walk's, and, when not cut by a marker, its
//! end state against the walk's state there. `validate` checks every packet at every
//! position, origin and marker set of every ternary column up to length 9 and the `abab`
//! and `abcabc` families against an independent model (Ψ from a stable sort, no cells).

// (the bounds are written as the derivation states them: ≤ 2r − 1, ≤ 3r − 2)
#![allow(clippy::int_plus_one)]

use std::collections::HashSet;

use mibz2::bwt::{build, walk, TVector, MIN_SEGMENTED};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, Kernels};

#[path = "../tests/common/mod.rs"]
mod common;

/// The room walk's segments (`bwt::FLAGGED_SEGMENTS`, pub(crate)): copied here
const SEGMENTS: usize = 512;

/// `bwt::random_rows` (pub(crate)), copied: the segment starts' generator
fn random_rows(n: usize, m: usize) -> Vec<u32> {
    let mut x = 0x2545_f491_4f6c_dd1du64 ^ n as u64;
    (0..m)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x % n as u64) as u32
        })
        .collect()
}

/// The production starts of a block: `segments` − 1 pseudo-random rows and r0, sorted,
/// deduplicated (`room_segments`), or none for a block the walk leaves unsegmented
fn production_starts(n: usize, r0: u32, segments: usize) -> Vec<u32> {
    let mut s = random_rows(n, segments - 1);
    s.push(r0);
    s.sort_unstable();
    s.dedup();
    s
}

/// A block's column: L, Ψ (production's links) and Ψ²
struct Column {
    l: Vec<u8>,
    psi: Vec<u32>,
    psi2: Vec<u32>,
    runs: usize,
}

fn column(l: &[u8]) -> Column {
    let n = l.len();
    let mut cnt = [0usize; 256];
    for &c in l {
        cnt[c as usize] += 1;
    }
    let mut next = [0usize; 256];
    let mut cum = 0;
    for c in 0..256 {
        next[c] = cum;
        cum += cnt[c];
    }
    let mut psi = vec![0u32; n];
    for (i, &c) in l.iter().enumerate() {
        psi[next[c as usize]] = i as u32;
        next[c as usize] += 1;
    }
    let psi2 = psi.iter().map(|&y| psi[y as usize]).collect();
    let runs = if n == 0 {
        0
    } else {
        1 + l.windows(2).filter(|w| w[0] != w[1]).count()
    };
    Column {
        l: l.to_vec(),
        psi,
        psi2,
        runs,
    }
}

/// A model's cells: each row's cell, and each cell's [u, v), delta and bytes
struct Cells {
    sb: usize,
    id: Vec<u32>,
    u: Vec<i64>,
    v: Vec<i64>,
    d: Vec<i64>,
    b: Vec<[u8; 2]>,
    /// the maximal intervals of constant delta alone (translation cells)
    translation: usize,
}

fn cells(col: &Column, pair: bool) -> Cells {
    let n = col.l.len();
    let sigma = if pair { &col.psi2 } else { &col.psi };
    let key = |q: usize| -> (i64, [u8; 2]) {
        let d = sigma[q] as i64 - q as i64;
        let b = if pair {
            [col.l[q], col.l[col.psi[q] as usize]]
        } else {
            [col.l[q], 0]
        };
        (d, b)
    };
    let mut c = Cells {
        sb: if pair { 2 } else { 1 },
        id: vec![0; n],
        u: Vec::new(),
        v: Vec::new(),
        d: Vec::new(),
        b: Vec::new(),
        translation: 0,
    };
    let mut prev: Option<(i64, [u8; 2])> = None;
    for q in 0..n {
        let k = key(q);
        if prev.map(|p| p.0) != Some(k.0) {
            c.translation += 1;
        }
        if prev != Some(k) {
            if let Some(last) = c.v.last_mut() {
                *last = q as i64;
            }
            c.u.push(q as i64);
            c.v.push(n as i64);
            c.d.push(k.0);
            c.b.push(k.1);
        }
        c.id[q] = (c.u.len() - 1) as u32;
        prev = Some(k);
    }
    c
}

/// A packet: its bytes copied, its path's cells, its repetitions, and its word's
/// displacement; `path[..k]` the cells' ids
#[derive(Clone, Copy, Default)]
struct Packet {
    len: usize,
    k: usize,
    m: u64,
    delta: i64,
    path: [u32; 16],
}

/// Each path packet from state q with `rem` bytes before the next marker, k = 1..=kmax
/// (until a path alone reaches the marker): `f` gets the packet and its uncapped
/// repetitions (u64::MAX for Δ = 0)
fn paths(c: &Cells, q: u32, rem: usize, kmax: usize, mut f: impl FnMut(Packet, u64)) {
    let qi = q as i64;
    let (mut lo, mut hi, mut s) = (i64::MIN, i64::MAX, 0i64);
    let mut y = qi;
    let mut path = [0u32; 16];
    for k in 1..=kmax {
        let id = c.id[y as usize] as usize;
        path[k - 1] = id as u32;
        lo = lo.max(c.u[id] - s);
        hi = hi.min(c.v[id] - s);
        s += c.d[id];
        y += c.d[id];
        debug_assert!(lo <= qi && qi < hi);
        let reps = if s > 0 {
            ((hi - 1 - qi) / s + 1) as u64
        } else if s < 0 {
            ((qi - lo) / -s + 1) as u64
        } else {
            u64::MAX
        };
        let per = k * c.sb;
        let m = reps.min(rem.div_ceil(per) as u64);
        let len = (per as u64 * m).min(rem as u64) as usize;
        f(
            Packet {
                len,
                k,
                m,
                delta: s,
                path,
            },
            reps,
        );
        if per >= rem {
            break;
        }
    }
}

/// The longest packet from state q with `rem` bytes before the next marker: paths of
/// k ≤ kmax cells; with `repeat`, a path of k ≥ 2 counts only if it repeats (m ≥ 2)
fn best_packet(c: &Cells, q: u32, rem: usize, kmax: usize, repeat: bool) -> Packet {
    let mut best = Packet::default();
    paths(c, q, rem, kmax, |p, reps| {
        if !(repeat && p.k >= 2 && reps < 2) && p.len > best.len {
            best = p;
        }
    });
    best
}

/// A packet checked against the trajectory `x` (states) and its bytes `out`, from t: its
/// word from the descriptors alone, repeated, equals the walk's bytes; uncut, its end state
/// is the walk's
fn check(c: &Cells, p: &Packet, x: &[u32], out: &[u8], t: usize) -> bool {
    let mut w = [0u8; 32];
    for j in 0..p.k {
        let b = c.b[p.path[j] as usize];
        w[j * c.sb] = b[0];
        if c.sb == 2 {
            w[j * 2 + 1] = b[1];
        }
    }
    let per = p.k * c.sb;
    if t + p.len > out.len() {
        return false;
    }
    for i in 0..p.len {
        if out[t + i] != w[i % per] {
            return false;
        }
    }
    let full = per as u64 * p.m;
    if p.len as u64 == full && t + p.len < x.len() {
        let end = x[t] as i64 + p.m as i64 * p.delta;
        if end != x[t + p.len] as i64 {
            return false;
        }
    }
    true
}

/// The models: a name, pair or single, k's bound, and whether k ≥ 2 must repeat
const MODELS: [(&str, bool, usize, bool); 11] = [
    ("single_k1", false, 1, true),
    ("single_k2r", false, 2, true),
    ("single_k4r", false, 4, true),
    ("single_k8r", false, 8, true),
    ("single_k16r", false, 16, true),
    ("pair_k1", true, 1, true),
    ("pair_k2r", true, 2, true),
    ("pair_k3r", true, 3, true),
    ("pair_k4r", true, 4, true),
    ("pair_k8r", true, 8, true),
    ("pair_k4path", true, 4, false),
];
const THRESH: [usize; 5] = [2, 4, 8, 16, 64];

#[derive(Default, Clone)]
struct Tally {
    packets: u64,
    bytes: u64,
    bytes_ge: [u64; 5],
    by_k: [u64; 17],
    /// Σ k: the cell lookups if no path is cached
    lookups: u64,
    /// the distinct descriptors (a cell, or a path of cells) the tiling used
    descriptors: u64,
}

/// One block's census: the trajectory from r0 for n steps (`walk`'s output), cut at the
/// production starts; each model's greedy tiling, every packet checked
fn census_block(l: &[u8], orig_ptr: u32, segmented: bool) -> String {
    let n = l.len();
    let col = column(l);
    let r0 = col.psi[orig_ptr as usize];
    let mut x = Vec::with_capacity(n);
    let mut y = r0;
    for _ in 0..n {
        x.push(y);
        y = col.psi[y as usize];
    }
    let out: Vec<u8> = x.iter().map(|&s| l[s as usize]).collect();
    // the cuts: segment starts the trajectory runs into (production's room walk)
    // (the plain walk, for a block below MIN_SEGMENTED, has no marker but the block's end)
    let starts = if segmented {
        production_starts(n, r0, SEGMENTS)
    } else {
        Vec::new()
    };
    let mut is_start = vec![false; n];
    for &s in &starts {
        is_start[s as usize] = true;
    }
    let mut bounds = vec![0usize];
    for (t, &s) in x.iter().enumerate().skip(1) {
        if is_start[s as usize] {
            bounds.push(t);
        }
    }
    bounds.push(n);
    let pair_walk_loads: usize = bounds.windows(2).map(|w| (w[1] - w[0]).div_ceil(2)).sum();

    let tandem = tandem_cover(&out, &bounds);
    let single = cells(&col, false);
    let pair = cells(&col, true);
    let r = col.runs;
    assert!(single.translation <= r, "Ψ's translation cells > r");
    assert!(single.u.len() <= 2 * r - 1, "single cells > 2r - 1");
    assert!(
        pair.translation <= 2 * r - 1,
        "Ψ² translation cells > 2r - 1"
    );
    assert!(pair.u.len() <= 3 * r - 2, "pair cells > 3r - 2");

    let mut js = format!(
        "\"n\":{n},\"r\":{r},\"psi_cells\":{},\"single_cells\":{},\"psi2_cells\":{},\"pair_cells\":{},\"segments\":{},\"cuts\":{},\"pair_walk_loads\":{pair_walk_loads},\"tandem_le\":{tandem:?}",
        single.translation,
        single.u.len(),
        pair.translation,
        pair.u.len(),
        starts.len(),
        bounds.len() - 1,
    );
    for &(name, is_pair, kmax, repeat) in &MODELS {
        let c = if is_pair { &pair } else { &single };
        let mut tl = Tally::default();
        let mut seen: HashSet<(usize, [u32; 16])> = HashSet::new();
        for w in bounds.windows(2) {
            let (mut t, end) = (w[0], w[1]);
            while t < end {
                let rem = end - t;
                let p = if is_pair && rem == 1 {
                    // a pair walk's odd end: one byte (PAIR_LF), one lookup
                    let id = c.id[x[t] as usize];
                    let mut path = [0u32; 16];
                    path[0] = id;
                    Packet {
                        len: 1,
                        k: 1,
                        m: 1,
                        delta: c.d[id as usize],
                        path,
                    }
                } else {
                    best_packet(c, x[t], rem, kmax, repeat)
                };
                assert!(p.len >= 1);
                assert!(check(c, &p, &x, &out, t), "{name}: a packet at {t} fails");
                tl.packets += 1;
                tl.bytes += p.len as u64;
                for (i, &th) in THRESH.iter().enumerate() {
                    if p.len >= th {
                        tl.bytes_ge[i] += p.len as u64;
                    }
                }
                tl.by_k[p.k] += 1;
                tl.lookups += p.k as u64;
                seen.insert((p.k, p.path));
                t += p.len;
            }
        }
        tl.descriptors = seen.len() as u64;
        assert_eq!(tl.bytes, n as u64);
        js += &format!(
            ",\"{name}\":{{\"packets\":{},\"lookups\":{},\"descriptors\":{},\"bytes_ge\":{:?},\"by_k\":{:?}}}",
            tl.packets, tl.lookups, tl.descriptors, tl.bytes_ge, tl.by_k
        );
    }
    js
}

/// The ceiling on any repeated packet, whatever its cells: a packet W^m with m ≥ 2 is a
/// tandem repeat of period |W| in the output, inside one segment. The bytes of `out` inside
/// some segment's tandem repeat (two whole copies or more) of period ≤ 4, 8, 16, 32 and 64
fn tandem_cover(out: &[u8], bounds: &[usize]) -> [u64; 5] {
    const PS: [usize; 5] = [4, 8, 16, 32, 64];
    let mut cov = vec![u8::MAX; out.len()];
    for w in bounds.windows(2) {
        let seg = &out[w[0]..w[1]];
        let c = &mut cov[w[0]..w[1]];
        for p in 1..=64usize.min(seg.len() / 2) {
            // maximal runs of seg[i] == seg[i + p]: a run of length ≥ p is two copies
            let (mut i0, mut run) = (0, 0);
            for i in 0..=seg.len() - p {
                if i < seg.len() - p && seg[i] == seg[i + p] {
                    if run == 0 {
                        i0 = i;
                    }
                    run += 1;
                } else {
                    if run >= p {
                        for e in &mut c[i0..i0 + run + p] {
                            *e = (*e).min(p as u8);
                        }
                    }
                    run = 0;
                }
            }
        }
    }
    PS.map(|p| cov.iter().filter(|&&e| (e as usize) <= p).count() as u64)
}

/// Each block of a .bz2: its last column decoded by the crate (MTF), its T-vector built and
/// walked by the crate, the walk's output checked against the census's trajectory and the
/// block's CRC; then the census
fn census_file(data: &[u8], label: &str) {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
    let (mut tt, mut walked, mut dec) = (TVector::new(), Vec::new(), Vec::new());
    let mut i = 0;
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        build(&ll, &counts, &mut tt);
        walk(&tt, b.orig_ptr, &mut walked);
        dec.clear();
        rle::unrle(&walked, &mut dec, k).unwrap();
        assert_eq!(crc::crc32(&dec, k), b.crc, "block CRC");
        // the census's own trajectory must be the crate's walk
        let col = column(&ll);
        let mut y = col.psi[b.orig_ptr as usize];
        for &w in &walked {
            assert_eq!(col.l[y as usize], w, "trajectory differs from walk");
            y = col.psi[y as usize];
        }
        let seg = ll.len() >= MIN_SEGMENTED;
        let js = census_block(&ll, b.orig_ptr, seg);
        println!(
            "{{\"corpus\":\"{label}\",\"block\":{i},\"decoded\":{},\"segmented\":{seg},{js}}}",
            dec.len()
        );
        i += 1;
        syms = b.syms;
    }
}

// ---- validation: an independent model ----

/// Ψ by definition: F is L stably sorted; row s of F is L's row i, so Ψ(s) = i
fn oracle_psi(l: &[u8]) -> Vec<u32> {
    let mut idx: Vec<u32> = (0..l.len() as u32).collect();
    idx.sort_by_key(|&i| (l[i as usize], i));
    idx
}

#[derive(Default)]
struct Counts {
    columns: u64,
    states: u64,
    packets: u64,
    packets_k2plus: u64,
    repeated: u64,
    cell_rows: u64,
    walk_checks: u64,
}

fn validate_column(l: &[u8], v: &mut Counts) {
    let n = l.len();
    let col = column(l);
    let opsi = oracle_psi(l);
    assert_eq!(col.psi, opsi, "Ψ differs from the stable sort");
    let single = cells(&col, false);
    let pair = cells(&col, true);
    let r = col.runs;
    assert!(single.translation <= r && single.u.len() <= 2 * r - 1);
    assert!(pair.translation <= 2 * r - 1 && pair.u.len() <= 3 * r - 2);
    // every cell constant, by the oracle's Ψ (no cells): bytes and delta; and maximal, its
    // first row's key differing from the row before's
    for (c, is_pair) in [(&single, false), (&pair, true)] {
        let key = |q: usize| {
            let s1 = opsi[q] as usize;
            if is_pair {
                (opsi[s1] as i64 - q as i64, [l[q], l[s1]])
            } else {
                (s1 as i64 - q as i64, [l[q], 0])
            }
        };
        for id in 0..c.u.len() {
            assert!(c.u[id] < c.v[id] && (id + 1 == c.u.len() || c.v[id] == c.u[id + 1]));
            if id > 0 {
                assert_ne!(
                    key(c.u[id] as usize - 1),
                    key(c.u[id] as usize),
                    "a cell not maximal"
                );
            }
            for q in c.u[id]..c.v[id] {
                let q = q as usize;
                assert_eq!(c.id[q] as usize, id);
                assert_eq!(key(q), (c.d[id], c.b[id]));
                v.cell_rows += 1;
            }
        }
        assert_eq!((c.u[0], *c.v.last().unwrap()), (0, n as i64));
    }
    // the crate's T-vector walked from each origin equals the oracle's walk
    let mut counts = [0u32; 256];
    for &c in l {
        counts[c as usize] += 1;
    }
    let mut tt = TVector::new();
    build(l, &counts, &mut tt);
    let mut walked = Vec::new();
    for orig in 0..n as u32 {
        let r0 = opsi[orig as usize];
        let mut x = Vec::with_capacity(n);
        let mut y = r0;
        for _ in 0..n {
            x.push(y);
            y = opsi[y as usize];
        }
        let out: Vec<u8> = x.iter().map(|&s| l[s as usize]).collect();
        walk(&tt, orig, &mut walked);
        assert_eq!(walked, out, "the oracle's walk differs from bwt::walk");
        v.walk_checks += 1;
        // marker sets: none (the plain walk); r0 alone; production's rule at 2 and 4
        // segments; every row a start
        let mut sets: Vec<Vec<u32>> = vec![Vec::new(), vec![r0]];
        sets.push(production_starts(n, r0, 2));
        sets.push(production_starts(n, r0, 4));
        sets.push((0..n as u32).collect());
        for st in &sets {
            let mut is_start = vec![false; n];
            for &s in st {
                is_start[s as usize] = true;
            }
            // the bytes left in t's segment, by the oracle's trajectory
            let mut rem = vec![0usize; n];
            let mut e = n;
            for t in (0..n).rev() {
                rem[t] = e - t;
                if is_start[x[t] as usize] {
                    e = t;
                }
            }
            for t in 0..n {
                v.states += 1;
                for &(_, is_pair, kmax, repeat) in &MODELS {
                    let c = if is_pair { &pair } else { &single };
                    if is_pair && rem[t] == 1 {
                        continue;
                    }
                    let p = best_packet(c, x[t], rem[t], kmax, repeat);
                    // the packet's claim, against the oracle alone
                    assert!(p.len >= 1 && p.len <= rem[t]);
                    for i in 1..p.len {
                        assert!(!is_start[x[t + i] as usize], "a packet crosses a marker");
                    }
                    assert!(check(c, &p, &x, &out, t), "a packet's bytes or end differ");
                    // and every path from t (each k up to kmax): correct, and maximal: the
                    // oracle's own steps (its Ψ, the cells only as labels) repeat the path's
                    // cells exactly m whole times before the marker, no more
                    paths(c, x[t], rem[t], kmax, |pk, _| {
                        assert!(check(c, &pk, &x, &out, t), "a path's bytes or end differ");
                        let per = pk.k * c.sb;
                        let mut y = x[t];
                        let mut whole = 0usize;
                        'reps: while whole * per < rem[t] {
                            for &id in &pk.path[..pk.k] {
                                if c.id[y as usize] != id {
                                    break 'reps;
                                }
                                for _ in 0..c.sb {
                                    y = opsi[y as usize];
                                }
                            }
                            whole += 1;
                        }
                        assert_eq!(pk.len, (whole * per).min(rem[t]), "a path is not maximal");
                        v.packets += 1;
                        if pk.k >= 2 {
                            v.packets_k2plus += 1;
                        }
                        if pk.m >= 2 && pk.len > per {
                            v.repeated += 1;
                        }
                    });
                }
            }
        }
    }
    v.columns += 1;
}

fn validate() {
    let mut v = Counts::default();
    for len in 1..=9u32 {
        for code in 0..3u32.pow(len) {
            let mut c = code;
            let l: Vec<u8> = (0..len)
                .map(|_| {
                    let b = b"abc"[(c % 3) as usize];
                    c /= 3;
                    b
                })
                .collect();
            validate_column(&l, &mut v);
        }
    }
    let ternary = v.columns;
    // the families: L of (ab)^j is b^j a^j; of (abc)^j, c^j a^j b^j; and with a tail
    for j in 1..=24usize {
        let rep = |b: u8, k: usize| std::iter::repeat(b).take(k);
        let abab: Vec<u8> = rep(b'b', j).chain(rep(b'a', j)).collect();
        let abc: Vec<u8> = rep(b'c', j)
            .chain(rep(b'a', j))
            .chain(rep(b'b', j))
            .collect();
        validate_column(&abab, &mut v);
        validate_column(&abc, &mut v);
        for text in [
            b"ab".repeat(j),
            b"abc".repeat(j),
            [b"abc".repeat(j), b"ab".to_vec()].concat(),
        ] {
            validate_column(&bwt_of(&text), &mut v);
        }
    }
    println!(
        "{{\"ternary_columns\":{ternary},\"columns\":{},\"walk_checks\":{},\"states\":{},\"packets_checked\":{},\"composed_k2plus\":{},\"repeated\":{},\"cell_rows\":{}}}",
        v.columns, v.walk_checks, v.states, v.packets, v.packets_k2plus, v.repeated, v.cell_rows
    );
    // the counterexamples, in numbers: one census row each (one chain from the text's row)
    for (name, text) in [
        ("abab_x60", b"ab".repeat(60)),
        ("abcabc_x60", b"abc".repeat(60)),
    ] {
        let l = bwt_of(&text);
        println!("{{\"family\":\"{name}\",{}}}", census_block(&l, 0, false));
    }
}

/// The last column of a text's sorted rotations (cyclic, as bzip2's BWT)
fn bwt_of(t: &[u8]) -> Vec<u8> {
    let n = t.len();
    let mut rot: Vec<usize> = (0..n).collect();
    rot.sort_by(|&a, &b| {
        (0..n)
            .map(|i| t[(a + i) % n])
            .cmp((0..n).map(|i| t[(b + i) % n]))
    });
    rot.iter().map(|&i| t[(i + n - 1) % n]).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("census") => {
            let data = std::fs::read(&args[1]).unwrap();
            let label = args.get(2).cloned().unwrap_or_else(|| args[1].clone());
            census_file(&data, &label);
        }
        Some("synthetic") => {
            let scale: usize = args.get(1).map_or(4, |s| s.parse().unwrap());
            for (name, data) in common::corpora(scale) {
                let z = common::compress(&data, 9);
                eprintln!("{name}: {} -> {} bytes", data.len(), z.len());
                census_file(&z, name);
            }
        }
        Some("validate") => validate(),
        _ => eprintln!("usage: packet_census census FILE.bz2 [LABEL] | synthetic SCALE | validate"),
    }
}
