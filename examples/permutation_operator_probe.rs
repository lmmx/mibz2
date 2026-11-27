#![forbid(unsafe_code)]
//! Two operators on the inverse BWT's permutation, priced against the walk they would replace.
//!
//! Smooth-zone diagnostic for claim `computational-permutation-structure`
//! (`docs/journal/2026-11-24-algorithm-level/computational-permutation-structure/`, frozen in
//! `registration.md` before data). Zero edits to `src/`, zero `unsafe`.
//!
//! - O1, `census`: labelled translation pieces of the powers `p^k` of raw LF, and of `T^k`, `T` the
//!   first return of LF to Open's retained rows (`LIMIT = 16`).
//! - O2, `bench`: word-capped labelled doubling of `T` by independent gathers, against the
//!   Open-format forward walk of `T` at one and four lanes.
//!
//! Usage: `permutation_operator_probe census|bench|all FILE.bz2 [--repeats N] [--train-repeats N]`

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const TRAIN: usize = 117;
const CAP: usize = 12;
const NEXT_BITS: u32 = 20;
const NEXT_MASK: u32 = (1 << NEXT_BITS) - 1;
const LEN_LONG: u32 = 31;
const I9_E_FWD4_MS: f64 = 325.2; // #992 warm baseline, projected i9 E of the 4-lane forward control
const I9_GATE_MS: f64 = 39.18;

// ---- admission (Open's rule, as in the #992/#998 harnesses) ----

const LAZY_DENSITY: (usize, usize) = (3, 20);
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u64 = 256_000;
const DENSE_STRIPS: usize = 16_384;

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs: Vec<u16> = Vec::with_capacity(l.len() / 4 + 16);
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

/// Maximal runs of `l`: (byte, start, width)
fn runs(l: &[u8]) -> Vec<(u8, u32, u32)> {
    let mut v = Vec::new();
    let mut i = 0;
    while i < l.len() {
        let c = l[i];
        let s = i;
        while i < l.len() && l[i] == c {
            i += 1;
        }
        v.push((c, s as u32, (i - s) as u32));
    }
    v
}

/// Open's retained rows (`split` in `src/open.rs`): thin maximal runs, and the primary alone
/// when it sits in a thick one. Returns (retained flag per row, thick pieces)
fn retained(l: &[u8], p: u32) -> (Vec<bool>, usize) {
    let mut keep = vec![false; l.len()];
    let mut strips = 0;
    for (_, pos, w) in runs(l) {
        let end = pos + w;
        if w > LIMIT && (pos..end).contains(&p) {
            keep[p as usize] = true;
            strips += usize::from(pos < p) + usize::from(p + 1 < end);
        } else if w <= LIMIT {
            keep[pos as usize..end as usize]
                .iter_mut()
                .for_each(|k| *k = true);
        } else {
            strips += 1;
        }
    }
    (keep, strips)
}

fn admitted(l: &[u8], p: u32, keep: &[bool], strips: usize) -> bool {
    let n = l.len();
    let records = records_of(l).len();
    if n == 0 || n > 900_000 || p as usize >= n {
        return false;
    }
    if records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n {
        return true;
    }
    if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
        let m = keep.iter().filter(|&&k| k).count() as u64;
        return m <= DENSE_M.min(n as u64 * 3 / 10) && strips <= DENSE_STRIPS;
    }
    false
}

fn raw_lf(l: &[u8]) -> Vec<u32> {
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut c = [0u32; 256];
    let mut t = 0;
    for a in 0..256 {
        c[a] = t;
        t += counts[a];
    }
    l.iter()
        .map(|&b| {
            let v = c[b as usize];
            c[b as usize] += 1;
            v
        })
        .collect()
}

// ---- the reference return map T, in Open's format ----

struct Block {
    n: usize,
    l: Vec<u8>,
    lf: Vec<u32>,
    /// T on retained ranks; `codes[j]` the dictionary code of `w(j)`
    next: Vec<u32>,
    codes: Vec<u32>,
    words: Vec<Vec<u8>>,
    mark: u32,
    single_cycle: bool,
    /// the backward stream reversed must be this (bwt::walk)
    walked: Vec<u8>,
    // Open format
    edges: Vec<u32>,
    bits: u32,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    spelling: Vec<u8>,
    long_at: Vec<(u32, u32)>,
    /// four lane starts: (rank, byte offset in the backward stream)
    lanes: [(u32, usize); 4],
}

fn make_block(l: Vec<u8>, p: u32, keep: &[bool], walked: Vec<u8>) -> Block {
    let n = l.len();
    let lf = raw_lf(&l);
    let mut rank = vec![u32::MAX; n];
    let mut m = 0u32;
    for (x, &k) in keep.iter().enumerate() {
        if k {
            rank[x] = m;
            m += 1;
        }
    }
    let mut next = vec![0u32; m as usize];
    let mut codes = vec![0u32; m as usize];
    let mut words: Vec<Vec<u8>> = (0..256u32).map(|b| vec![b as u8]).collect();
    let mut dict: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut w = Vec::new();
    for x in 0..n {
        if !keep[x] {
            continue;
        }
        w.clear();
        w.push(l[x]);
        let mut y = lf[x] as usize;
        while !keep[y] {
            w.push(l[y]);
            y = lf[y] as usize;
        }
        let j = rank[x] as usize;
        next[j] = rank[y];
        codes[j] = if w.len() == 1 {
            u32::from(w[0])
        } else if let Some(&c) = dict.get(&w) {
            c
        } else {
            let c = words.len() as u32;
            words.push(w.clone());
            dict.insert(w.clone(), c);
            c
        };
    }
    let mark = rank[p as usize];
    // cycle of the mark
    let (mut len, mut cur) = (0usize, mark);
    loop {
        len += words[codes[cur as usize] as usize].len();
        cur = next[cur as usize];
        if cur == mark {
            break;
        }
    }
    let single_cycle = len == n;
    let bits = (u32::BITS - (m.max(2) - 1).leading_zeros()).max(1);
    assert!(
        words.len() <= (u32::MAX >> bits) as usize + 1,
        "codes do not fit beside next"
    );
    let edges = (0..m as usize)
        .map(|j| next[j] | (codes[j] << bits))
        .collect();
    let mut inline16 = Vec::with_capacity(words.len());
    let mut lens = Vec::with_capacity(words.len());
    let mut spelling = Vec::new();
    let mut long_at = Vec::with_capacity(words.len());
    for wd in &words {
        let mut slot = [0u8; INLINE];
        if wd.len() <= INLINE {
            slot[..wd.len()].copy_from_slice(wd);
            lens.push(wd.len() as u8);
            long_at.push((0, 0));
        } else {
            lens.push(LONG);
            let s = spelling.len() as u32;
            spelling.extend_from_slice(wd);
            long_at.push((s, spelling.len() as u32));
        }
        inline16.push(slot);
    }
    // lane starts at quarter visits of the mark's cycle
    let mut lanes = [(mark, 0usize); 4];
    if single_cycle {
        let mut off = 0usize;
        let mut cur = mark;
        let quarter = [m as usize / 4, m as usize / 2, 3 * m as usize / 4];
        for v in 0..m as usize {
            for (q, &at) in quarter.iter().enumerate() {
                if v == at {
                    lanes[q + 1] = (cur, off);
                }
            }
            off += words[codes[cur as usize] as usize].len();
            cur = next[cur as usize];
        }
    }
    Block {
        n,
        l,
        lf,
        next,
        codes,
        words,
        mark,
        single_cycle,
        walked,
        edges,
        bits,
        inline16,
        lens,
        spelling,
        long_at,
        lanes,
    }
}

// ---- O1: labelled power pieces ----

fn pieces_raw(b: &Block, ks: &[usize]) -> Vec<usize> {
    let n = b.n;
    let base: Vec<bool> = (0..n).map(|j| j + 1 < n && b.l[j] == b.l[j + 1]).collect();
    let mut eq = base.clone();
    let mut out = Vec::new();
    let mut k = 1;
    for &want in ks {
        while k < want {
            // eq_{k+1}[j] = base[j] && eq_k[lf j]
            let new: Vec<bool> = (0..n).map(|j| base[j] && eq[b.lf[j] as usize]).collect();
            eq = new;
            k += 1;
        }
        out.push(n - eq.iter().filter(|&&e| e).count());
    }
    out
}

fn pieces_t(b: &Block, ks: &[usize]) -> Vec<usize> {
    let m = b.next.len();
    let base: Vec<bool> = (0..m)
        .map(|j| j + 1 < m && b.codes[j] == b.codes[j + 1] && b.next[j] + 1 == b.next[j + 1])
        .collect();
    let mut eq = base.clone();
    let mut out = Vec::new();
    let mut k = 1;
    for &want in ks {
        while k < want {
            let new: Vec<bool> = (0..m).map(|j| base[j] && eq[b.next[j] as usize]).collect();
            eq = new;
            k += 1;
        }
        out.push(m - eq.iter().filter(|&&e| e).count());
    }
    out
}

// ---- consumers ----

#[inline(always)]
fn step_ctrl<const TAIL: bool>(b: &Block, cur: u32, w: usize, out: &mut [u8]) -> (u32, usize) {
    let e = b.edges[cur as usize];
    let code = (e >> b.bits) as usize;
    let l = b.lens[code];
    if l != LONG {
        if TAIL {
            out[w..w + l as usize].copy_from_slice(&b.inline16[code][..l as usize]);
        } else {
            out[w..w + INLINE].copy_from_slice(&b.inline16[code]);
        }
        (e & ((1 << b.bits) - 1), w + l as usize)
    } else {
        let (s, t) = b.long_at[code];
        let len = (t - s) as usize;
        out[w..w + len].copy_from_slice(&b.spelling[s as usize..t as usize]);
        (e & ((1 << b.bits) - 1), w + len)
    }
}

fn ctrl1(b: &Block, out: &mut [u8]) {
    let (mut cur, mut w) = (b.mark, 0usize);
    while w < b.n {
        (cur, w) = step_ctrl::<false>(b, cur, w, out);
    }
}

fn ctrl4(b: &Block, out: &mut [u8]) {
    let ends = [b.lanes[1].1, b.lanes[2].1, b.lanes[3].1, b.n];
    let mut cur = [b.lanes[0].0, b.lanes[1].0, b.lanes[2].0, b.lanes[3].0];
    let mut w = [b.lanes[0].1, b.lanes[1].1, b.lanes[2].1, b.lanes[3].1];
    loop {
        let mut live = false;
        for i in 0..4 {
            if w[i] < ends[i] {
                (cur[i], w[i]) = if w[i] + INLINE < ends[i] {
                    step_ctrl::<false>(b, cur[i], w[i], out)
                } else {
                    step_ctrl::<true>(b, cur[i], w[i], out)
                };
                live = true;
            }
        }
        if !live {
            break;
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Rec {
    nl: u32,
    pay: [u8; CAP],
}

/// Pass 0: Open's edges and dictionary to 16-byte records (charged to C)
#[allow(clippy::needless_range_loop)]
fn pass0(b: &Block, rec: &mut Vec<Rec>) {
    let m = b.edges.len();
    rec.resize(m, Rec::default());
    let mask = (1u32 << b.bits) - 1;
    for j in 0..m {
        let e = b.edges[j];
        let code = (e >> b.bits) as usize;
        let l = b.lens[code];
        let next = e & mask;
        rec[j] = if l != LONG && l as usize <= CAP {
            let mut pay = [0u8; CAP];
            pay.copy_from_slice(&b.inline16[code][..CAP]);
            Rec {
                nl: next | (u32::from(l) << NEXT_BITS),
                pay,
            }
        } else {
            let mut pay = [0u8; CAP];
            pay[..4].copy_from_slice(&(code as u32).to_le_bytes());
            Rec {
                nl: next | (LEN_LONG << NEXT_BITS),
                pay,
            }
        };
    }
}

/// One doubling round: independent gathers, double-buffered
fn round(src: &[Rec], dst: &mut [Rec]) {
    for x in 0..src.len() {
        let a = src[x];
        let la = (a.nl >> NEXT_BITS) & 31;
        let mut r = a;
        if la != LEN_LONG {
            let bnode = src[(a.nl & NEXT_MASK) as usize];
            let lb = (bnode.nl >> NEXT_BITS) & 31;
            if lb != LEN_LONG && (la + lb) as usize <= CAP {
                // the words as integers: no per-row memcpy
                let wide = |p: &[u8; CAP]| {
                    let mut q = [0u8; 16];
                    q[..CAP].copy_from_slice(p);
                    u128::from_le_bytes(q)
                };
                let low = wide(&a.pay) & ((1u128 << (8 * la)) - 1);
                let joined = low | (wide(&bnode.pay) << (8 * la));
                r.pay.copy_from_slice(&joined.to_le_bytes()[..CAP]);
                r.nl = (bnode.nl & NEXT_MASK) | ((la + lb) << NEXT_BITS);
            }
        }
        dst[x] = r;
    }
}

fn construct(b: &Block, rounds: usize, a: &mut Vec<Rec>, t: &mut Vec<Rec>) {
    pass0(b, a);
    t.resize(a.len(), Rec::default());
    for _ in 0..rounds {
        round(a, t);
        std::mem::swap(a, t);
    }
}

#[inline(always)]
fn step_j<const TAIL: bool>(
    b: &Block,
    rec: &[Rec],
    cur: u32,
    w: usize,
    out: &mut [u8],
) -> (u32, usize) {
    let r = rec[cur as usize];
    let l = (r.nl >> NEXT_BITS) & 31;
    if l != LEN_LONG {
        if TAIL {
            out[w..w + l as usize].copy_from_slice(&r.pay[..l as usize]);
        } else {
            out[w..w + CAP].copy_from_slice(&r.pay);
        }
        (r.nl & NEXT_MASK, w + l as usize)
    } else {
        let code = u32::from_le_bytes([r.pay[0], r.pay[1], r.pay[2], r.pay[3]]) as usize;
        let lc = b.lens[code];
        if lc != LONG {
            if TAIL {
                out[w..w + lc as usize].copy_from_slice(&b.inline16[code][..lc as usize]);
            } else {
                out[w..w + INLINE].copy_from_slice(&b.inline16[code]);
            }
            (r.nl & NEXT_MASK, w + lc as usize)
        } else {
            let (s, t) = b.long_at[code];
            let len = (t - s) as usize;
            out[w..w + len].copy_from_slice(&b.spelling[s as usize..t as usize]);
            (r.nl & NEXT_MASK, w + len)
        }
    }
}

fn j1(b: &Block, rec: &[Rec], out: &mut [u8]) -> usize {
    let (mut cur, mut w, mut v) = (b.mark, 0usize, 0usize);
    while w < b.n {
        (cur, w) = step_j::<false>(b, rec, cur, w, out);
        v += 1;
    }
    v
}

fn j4(b: &Block, rec: &[Rec], out: &mut [u8]) {
    let ends = [b.lanes[1].1, b.lanes[2].1, b.lanes[3].1, b.n];
    let mut cur = [b.lanes[0].0, b.lanes[1].0, b.lanes[2].0, b.lanes[3].0];
    let mut w = [b.lanes[0].1, b.lanes[1].1, b.lanes[2].1, b.lanes[3].1];
    loop {
        let mut live = false;
        for i in 0..4 {
            if w[i] < ends[i] {
                (cur[i], w[i]) = if w[i] + INLINE < ends[i] {
                    step_j::<false>(b, rec, cur[i], w[i], out)
                } else {
                    step_j::<true>(b, rec, cur[i], w[i], out)
                };
                live = true;
            }
        }
        if !live {
            break;
        }
    }
}

fn exact(b: &Block, out: &[u8]) -> bool {
    let n = b.n;
    out[..n].iter().rev().eq(b.walked.iter())
}

// ---- loading ----

fn load(path: &str) -> Vec<Block> {
    let data = std::fs::read(path).expect("read bz2");
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut syms, mut ll, mut counts, mut tt) =
        (Vec::new(), Vec::new(), [0u32; 256], TVector::default());
    let mut blocks = Vec::new();
    let mut total = 0;
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        total += 1;
        let (keep, strips) = retained(&ll, b.orig_ptr);
        if admitted(&ll, b.orig_ptr, &keep, strips) {
            let mut walked = Vec::new();
            build(&ll, &counts, &mut tt);
            walk(&tt, b.orig_ptr, &mut walked);
            blocks.push(make_block(ll.clone(), b.orig_ptr, &keep, walked));
        }
        syms = b.syms;
    }
    eprintln!("{} blocks, {} admitted", total, blocks.len());
    blocks
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn pct(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() - 1) as f64 * q).round() as usize]
}

fn census(blocks: &[Block]) {
    let train = &blocks[..TRAIN.min(blocks.len())];
    let ks = [1usize, 2, 4, 8, 16, 32];
    let kt = [1usize, 2, 4, 8];
    let mut raw: Vec<Vec<f64>> = vec![Vec::new(); ks.len()];
    let mut tab_ok: Vec<usize> = vec![0; ks.len()];
    let mut visit_ok: Vec<usize> = vec![0; ks.len()];
    let mut tt: Vec<Vec<f64>> = vec![Vec::new(); kt.len()];
    let (mut mn, mut lit) = (Vec::new(), 0f64);
    let mut visits = 0usize;
    for b in train {
        let m = b.next.len();
        let dict: usize = b.words.iter().skip(256).map(|w| w.len()).sum();
        let open_bytes = 4 * m + dict;
        mn.push(m as f64 / b.n as f64);
        lit += b.codes.iter().filter(|&&c| c < 256).count() as f64;
        visits += m;
        for (i, &pk) in pieces_raw(b, &ks).iter().enumerate() {
            raw[i].push(pk as f64 / b.n as f64);
            if pk * (ks[i] + 8) <= open_bytes {
                tab_ok[i] += 1;
            }
            if b.n / ks[i] <= m {
                visit_ok[i] += 1;
            }
        }
        for (i, &pk) in pieces_t(b, &kt).iter().enumerate() {
            tt[i].push(pk as f64 / m as f64);
        }
    }
    println!(
        "O1 census, train {} blocks: median m/n = {:.4}, literal share of visits = {:.3}",
        train.len(),
        median(&mut mn),
        lit / visits as f64
    );
    println!("  k   P_k(p)/n med [p10..p90]   n/k<=m blocks   P_k(k+8)<=Open bytes blocks");
    for (i, &k) in ks.iter().enumerate() {
        let (a, lo, hi) = (
            median(&mut raw[i]),
            pct(&mut raw[i], 0.1),
            pct(&mut raw[i], 0.9),
        );
        println!(
            "  {k:>2}  {a:.4} [{lo:.4}..{hi:.4}]   {:>3}   {:>3}",
            visit_ok[i], tab_ok[i]
        );
    }
    println!("  k   P_k(T)/m med [p10..p90]");
    for (i, &k) in kt.iter().enumerate() {
        let (a, lo, hi) = (
            median(&mut tt[i]),
            pct(&mut tt[i], 0.1),
            pct(&mut tt[i], 0.9),
        );
        println!("  {k:>2}  {a:.4} [{lo:.4}..{hi:.4}]");
    }
}

struct Scratch {
    out: Vec<u8>,
    a: Vec<Rec>,
    t: Vec<Rec>,
}

fn time<F: FnMut()>(mut f: F) -> f64 {
    let s = Instant::now();
    f();
    s.elapsed().as_secs_f64() * 1e3
}

/// One repeat of every arm over `blocks`: per-block warm-up then timed, summed (ms)
fn pass_arms(blocks: &[&Block], rounds: &[usize], s: &mut Scratch) -> Vec<f64> {
    let mut acc = vec![0f64; 2 + 3 * rounds.len()];
    for b in blocks {
        s.out.resize(b.n + INLINE + CAP, 0);
        ctrl1(b, &mut s.out);
        acc[0] += time(|| ctrl1(b, black_box(&mut s.out)));
        ctrl4(b, &mut s.out);
        acc[1] += time(|| ctrl4(b, black_box(&mut s.out)));
        for (i, &r) in rounds.iter().enumerate() {
            construct(b, r, &mut s.a, &mut s.t);
            acc[2 + 3 * i] += time(|| construct(b, r, black_box(&mut s.a), &mut s.t));
            j1(b, &s.a, &mut s.out);
            acc[3 + 3 * i] += time(|| {
                black_box(j1(b, &s.a, black_box(&mut s.out)));
            });
            j4(b, &s.a, &mut s.out);
            acc[4 + 3 * i] += time(|| j4(b, &s.a, black_box(&mut s.out)));
        }
    }
    acc
}

fn bench(blocks: &[Block], repeats: usize, train_repeats: usize) {
    let mut s = Scratch {
        out: Vec::new(),
        a: Vec::new(),
        t: Vec::new(),
    };
    let rounds = [0usize, 1, 2, 3];
    // exactness on every admitted block, every arm
    let mut ok = 0;
    let mut visits = vec![0usize; rounds.len()];
    let mut m_total = 0usize;
    let mut multi = 0;
    for b in blocks {
        s.out.clear();
        s.out.resize(b.n + INLINE + CAP, 0);
        ctrl1(b, &mut s.out);
        let mut good = exact(b, &s.out);
        if b.single_cycle {
            s.out.iter_mut().for_each(|x| *x = 0);
            ctrl4(b, &mut s.out);
            good &= exact(b, &s.out);
        } else {
            multi += 1;
        }
        for (i, &r) in rounds.iter().enumerate() {
            construct(b, r, &mut s.a, &mut s.t);
            s.out.iter_mut().for_each(|x| *x = 0);
            visits[i] += j1(b, &s.a, &mut s.out);
            good &= exact(b, &s.out);
            if b.single_cycle {
                s.out.iter_mut().for_each(|x| *x = 0);
                j4(b, &s.a, &mut s.out);
                good &= exact(b, &s.out);
            }
        }
        m_total += b.next.len();
        ok += usize::from(good);
    }
    println!("exactness: {ok}/{} blocks byte-identical to bwt::walk on every arm ({multi} multi-cycle: 4-lane arms skipped there)", blocks.len());
    println!(
        "visits: ctrl {m_total}; {}",
        rounds
            .iter()
            .zip(&visits)
            .map(|(r, v)| format!("J_{r} {v} ({:.3}x)", *v as f64 / m_total as f64))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let single: Vec<&Block> = blocks.iter().filter(|b| b.single_cycle).collect();
    let split = blocks.iter().take(TRAIN).filter(|b| b.single_cycle).count();
    let (train, held) = single.split_at(split);
    // train selection
    let mut tr: Vec<Vec<f64>> = vec![Vec::new(); 2 + 3 * rounds.len()];
    for _ in 0..train_repeats {
        for (i, v) in pass_arms(train, &rounds, &mut s).into_iter().enumerate() {
            tr[i].push(v);
        }
    }
    let trm: Vec<f64> = tr.iter_mut().map(|v| median(v)).collect();
    let pick = (0..rounds.len())
        .min_by(|&a, &b| {
            (trm[2 + 3 * a] + trm[4 + 3 * a])
                .partial_cmp(&(trm[2 + 3 * b] + trm[4 + 3 * b]))
                .unwrap()
        })
        .unwrap();
    println!(
        "train ({} blocks, {train_repeats} repeats, medians ms): ctrl1 {:.2}, ctrl4 {:.2}",
        train.len(),
        trm[0],
        trm[1]
    );
    for (i, r) in rounds.iter().enumerate() {
        println!(
            "  r={r}: C {:.2}  E1 {:.2}  E4 {:.2}  C+E4 {:.2}",
            trm[2 + 3 * i],
            trm[3 + 3 * i],
            trm[4 + 3 * i],
            trm[2 + 3 * i] + trm[4 + 3 * i]
        );
    }
    println!("  selected r = {}", rounds[pick]);
    // held-out
    let mut h: Vec<Vec<f64>> = vec![Vec::new(); 2 + 3 * rounds.len()];
    for _ in 0..repeats {
        for (i, v) in pass_arms(held, &rounds, &mut s).into_iter().enumerate() {
            h[i].push(v);
        }
    }
    let fmt = |v: &mut Vec<f64>| {
        let med = median(v);
        let (lo, hi) = (v[0], v[v.len() - 1]);
        (med, lo, hi)
    };
    let c1 = fmt(&mut h[0]);
    let c4 = fmt(&mut h[1]);
    println!(
        "held-out ({} blocks, {repeats} interleaved repeats, ms med (min..max)):",
        held.len()
    );
    println!(
        "  ctrl1 {:.2} ({:.2}..{:.2})   ctrl4 {:.2} ({:.2}..{:.2})",
        c1.0, c1.1, c1.2, c4.0, c4.1, c4.2
    );
    for (i, r) in rounds.iter().enumerate() {
        let c = fmt(&mut h[2 + 3 * i].clone());
        let e1 = fmt(&mut h[3 + 3 * i].clone());
        let e4 = fmt(&mut h[4 + 3 * i].clone());
        // paired per-repeat totals C+E4
        let mut tot: Vec<f64> = (0..repeats)
            .map(|k| h[2 + 3 * i][k] + h[4 + 3 * i][k])
            .collect();
        let t = fmt(&mut tot);
        let net = c4.0 - t.0;
        let proj = net / c4.0 * I9_E_FWD4_MS;
        let disjoint = t.2 < c4.1 || t.1 > c4.2;
        println!(
            "  r={r}{}: C {:.2} ({:.2}..{:.2})  E1 {:.2} ({:.2}..{:.2}) [{:.3}x ctrl1]  E4 {:.2} ({:.2}..{:.2}) [{:.3}x ctrl4]  C+E4 {:.2} ({:.2}..{:.2})  net vs ctrl4 {:+.2} ms ({:+.1}%) disjoint={} -> i9 proj {:+.1} ms (gate {})",
            if i == pick { "*" } else { "" },
            c.0, c.1, c.2, e1.0, e1.1, e1.2, e1.0 / c1.0, e4.0, e4.1, e4.2, e4.0 / c4.0, t.0, t.1, t.2, net, 100.0 * net / c4.0, disjoint, proj, I9_GATE_MS
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("all");
    let path = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("corpora/wikidata.bz2");
    let get = |flag: &str, d: usize| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(d)
    };
    let blocks = load(path);
    if mode == "census" || mode == "all" {
        census(&blocks);
    }
    if mode == "bench" || mode == "all" {
        bench(&blocks, get("--repeats", 7), get("--train-repeats", 3));
    }
}
