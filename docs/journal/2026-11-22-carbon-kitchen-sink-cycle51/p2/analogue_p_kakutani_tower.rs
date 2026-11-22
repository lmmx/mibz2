#![forbid(unsafe_code)]

//! Cycle 51 P2 — Analogue P:
//! S. Kakutani (1943) / G. D. Birkhoff (1931) / E. Hopf (1937) Ergodic Skyscraper Towers,
//! Exact 3-Strip Interior-Primary Christoffel `S2b` Depth Law (Extending `CA-40` to a
//! 9-Node / 8-Link Pure-`CA` Longest Chain), Exact `Open<16>` Skyscraper Height Spectrum &
//! Birkhoff–Hopf Cycle Conservation, and 2-Stage Kakutani Induction (`R_2 ⊂ R_1`) Test.
//!
//! Primary Sources:
//!   - S. Kakutani, "Induced measure preserving transformations",
//!     Proc. Imp. Acad. Tokyo 19 (1943), 635–641
//!     (full skyscraper decomposition X = \bigsqcup_{k>=1} \bigsqcup_{j=0}^{k-1} T^j A_k,
//!      height-sum theorem \sum_{x in A} \tau_A(x) = |X|, and 2-stage composition
//!      (T_{R_1})_{R_2} = T_{R_2} with additive height \tau_{R_2}(x) = \sum \tau_{R_1});
//!   - G. D. Birkhoff, "Proof of the ergodic theorem",
//!     Proc. Nat. Acad. Sci. USA 17 (1931), 656–660;
//!   - E. Hopf, Ergodentheorie, Ergebnisse der Mathematik und ihrer Grenzgebiete,
//!     Band 5, Heft 2, Springer, Berlin (1937), Ch. I–II
//!     (ergodic cycle decomposition & conservative/recurrent vs orphan-cycle balance);
//!   Extending:
//!   - CA-05 (Cauchy 1815, 1844), CA-10 (Kakutani 1943), CA-11 (Kac 1947),
//!     CA-35 (Lucas 1891), and CA-40 (Markov 1879–80).
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`
//! AND passes `-C debug-assertions=yes -C overflow-checks=yes`.

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/outq.rs"]
mod outq;

use std::collections::BTreeMap;
use std::fs;
use std::sync::Barrier;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const ID_BITS: u32 = 24;
const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
const SATURATED: u32 = 255;

#[inline(always)]
const fn literal(byte: u8) -> u32 {
    (1u32 << ID_BITS) | (byte as u32)
}

// ============================================================================
// 0. Deterministic PRNG, Number-Theoretic Helpers, Reference FL & Corpora
// ============================================================================

struct XorShift64(u64);

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    #[inline]
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    #[inline]
    fn gen_range(&mut self, hi: usize) -> usize {
        (self.next_u64() as usize) % hi
    }
}

#[inline]
fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

fn continued_fraction(mut p: u64, mut q: u64) -> Vec<u64> {
    assert!(p >= 1 && q >= 1);
    let mut cf = Vec::new();
    while q != 0 {
        cf.push(p / q);
        let r = p % q;
        p = q;
        q = r;
    }
    cf
}

fn from_cf(cf: &[u64]) -> (usize, usize) {
    let mut num = 1u64;
    let mut den = 0u64;
    for &a in cf.iter().rev() {
        let next_num = a * num + den;
        den = num;
        num = next_num;
    }
    (num as usize, den as usize)
}

fn fib_u_seq(max_k: usize) -> Vec<u64> {
    let mut u = vec![0u64; max_k + 1];
    if max_k >= 1 {
        u[1] = 1;
    }
    for k in 2..=max_k {
        u[k] = u[k - 1] + u[k - 2];
    }
    u
}

fn generate_markov_slopes(limit: u128) -> Vec<(u128, usize, usize)> {
    let mut out = Vec::new();
    fn rec(
        xl: u128,
        pl: usize,
        ql: usize,
        yr: u128,
        pr: usize,
        qr: usize,
        z: u128,
        limit: u128,
        out: &mut Vec<(u128, usize, usize)>,
    ) {
        if z > limit {
            return;
        }
        let pm = pl + pr;
        let qm = ql + qr;
        out.push((z, pm, qm));
        if let Some(zl) = xl.checked_mul(3).and_then(|v| v.checked_mul(z)).and_then(|v| v.checked_sub(yr)) {
            if zl <= limit {
                rec(xl, pl, ql, z, pm, qm, zl, limit, out);
            }
        }
        if let Some(zr) = z.checked_mul(3).and_then(|v| v.checked_mul(yr)).and_then(|v| v.checked_sub(xl)) {
            if zr <= limit {
                rec(z, pm, qm, yr, pr, qr, zr, limit, out);
            }
        }
    }
    if limit >= 5 {
        rec(1, 1, 0, 2, 0, 1, 5, limit, &mut out);
    }
    out.sort_by_key(|t| (t.0, t.1, t.2));
    out
}

/// Exact forward-FL reference decoder (matching bzip2 / mibz2 semantics on valid AND malformed inputs).
fn ref_fl(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    if n == 0 {
        return Vec::new();
    }
    let mut cnt = [0u32; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for a in 0..256 {
        starts[a] = tot;
        tot += cnt[a];
    }
    let mut fl = vec![0u32; n];
    let mut f_byte = vec![0u8; n];
    let mut cur = starts;
    for (i, &b) in l.iter().enumerate() {
        let a = usize::from(b);
        let pos = cur[a] as usize;
        fl[pos] = i as u32;
        f_byte[pos] = b;
        cur[a] += 1;
    }
    let mut out = Vec::with_capacity(n);
    let mut x = p as usize;
    for _ in 0..n {
        out.push(f_byte[x]);
        x = fl[x] as usize;
    }
    out
}

/// Exact O(n^2 log n) BWT for small valid test strings (n <= 200).
fn bwt_small(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &rot) in sa.iter().enumerate() {
        if rot == 0 {
            p = i as u32;
        }
        l.push(text[(rot + n - 1) % n]);
    }
    (l, p)
}

/// Encode raw L column into canonical CAP=127 run-length records (mibz2 format).
fn l_to_records(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
    let mut i = 0usize;
    while i < l.len() {
        let b = l[i];
        let mut j = i + 1;
        while j < l.len() && l[j] == b {
            j += 1;
        }
        let mut rem = j - i;
        while rem > 0 {
            let take = rem.min(127);
            recs.push((u16::from(b) << 8) | (take as u16));
            rem -= take;
        }
        i = j;
    }
    recs
}

fn records_to_l(records: &[u16], n: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &rec in records {
        let b = (rec >> 8) as u8;
        let w = (rec & 255) as usize;
        for _ in 0..w {
            l.push(b);
        }
    }
    assert_eq!(l.len(), n);
    l
}

struct BlockData {
    name: String,
    p: u32,
    l: Vec<u8>,
    records: Vec<u16>,
}

fn resolve_corpus_path(candidates: &[&str]) -> String {
    for &p in candidates {
        if std::path::Path::new(p).exists() {
            return p.to_string();
        }
    }
    panic!("Could not locate corpus among {:?}", candidates);
}

fn load_six_admitted() -> Vec<BlockData> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = fs::read_to_string(format!("{dir}/manifest.json")).expect("read manifest.json");
    let mut specs = Vec::new();
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") {
            file = v.trim_end_matches("\",").to_string();
        }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            let p = v.trim_end_matches(',').parse::<u32>().expect("parse p");
            specs.push((file.clone(), p));
        }
    }
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = fs::read(format!("{dir}/{f}")).expect("read .L");
            let records = l_to_records(&l);
            BlockData { name: f, p, l, records }
        })
        .collect()
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool) -> Vec<BlockData> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::new();
    for b_idx in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let route_class = raw[off + 16];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        if only_full && route_class > 1 {
            continue;
        }
        let name = if let Some(ns) = names {
            if b_idx < ns.len() {
                ns[b_idx].to_string()
            } else {
                format!("blk-{idx}")
            }
        } else {
            format!("w431-{idx}")
        };
        let l = records_to_l(&records, n);
        blocks.push(BlockData { name, p, l, records });
    }
    blocks
}

#[inline]
fn pct(a: u64, b: u64) -> f64 {
    if b == 0 {
        0.0
    } else {
        100.0 * a as f64 / b as f64
    }
}

// ============================================================================
// 1. PART 1 — Exact Resolution of Cycle 50 Probe 2:
//    Kakutani (1943) 3-Strip Interior-Primary Cut-Position Depth Law on CA-40
// ============================================================================

#[inline]
fn join_depth(dl: Option<u64>, dr: Option<u64>) -> Option<u64> {
    match (dl, dr) {
        (None, r) => r,
        (l, None) => l,
        (Some(a), Some(b)) => Some(1 + a.max(b)),
    }
}

/// Exact O(log n) analytical Kakutani 3-interval skyscraper cut-position depth reduction
/// for any thick Christoffel BWT `L = b^q a^p` (`gcd(p, q) = 1`, `p, q > LIMIT`, `n = p + q`)
/// at ANY primary index `r in 0..n`.
///
/// Proves that `Open<16>` `S2b` preserves a strict 2-phase 5-node Kakutani 3-strip invariant:
///   - Phase `cut_in_B`: `S0 = G_im(1) | B1(b1) | G_dom(1) | B2(b2) | A(A)`,
///                       `S1 = G_dom(1) | A(A) | B1(b1) | G_im(1) | B2(b2)`,
///   - Phase `cut_in_A`: `S0 = G_im(1) | B(B) | A1(a1) | G_dom(1) | A2(a2)`,
///                       `S1 = G_dom(1) | A1(a1) | G_im(1) | A2(a2) | B(B)`.
/// Every Euclidean quotient `k` maps this template strictly into itself in O(1) integer
/// arithmetic, while each cut crossing flips `cut_in_B <-> cut_in_A`.
/// Crucially, all non-final partial quotients `a_k (k < ell)` are absorbed in full without
/// splitting depth (`k_2 + 1 + k_1 == a_k`), and ONLY the final partial quotient `a_ell`
/// (when `min(A, B) == 1`) splits into two parallel spirals of depths `~c_ell` and `~a_ell - c_ell`,
/// yielding the exact sharp universal envelope:
///   `sum(a_i) - floor(a_ell / 2) <= h_S2b(p, q, r) <= sum(a_i) + 1`,
/// and the Markov-Zone Universal Primary Rigidity Corollary (`max(a_i) <= 2 => |h_S2b - sum| <= 1`
/// for ALL `r in 0..n`).
fn kakutani_3strip_depth_exact(p: u64, q: u64, r: u64) -> u64 {
    assert!(p >= 1 && q >= 1 && r < p + q);
    assert_eq!(gcd_u64(p, q), 1);

    let mut cut_in_b: bool;
    let mut a_w: u64;
    let mut b_w: u64;
    let mut a1: u64;
    let mut a2: u64;
    let mut b1: u64;
    let mut b2: u64;
    let mut d_a: Option<u64>;
    let mut d_b: Option<u64>;
    let mut d_a1: Option<u64>;
    let mut d_a2: Option<u64>;
    let mut d_b1: Option<u64>;
    let mut d_b2: Option<u64>;
    let mut d_gdom: Option<u64> = None;
    let mut d_gim: Option<u64> = None;

    if r < q {
        cut_in_b = true;
        a_w = p;
        d_a = Some(0);
        b1 = r;
        b2 = q - 1 - r;
        d_b1 = if b1 > 0 { Some(0) } else { None };
        d_b2 = if b2 > 0 { Some(0) } else { None };
        b_w = q;
        d_b = None;
        a1 = 0;
        a2 = 0;
        d_a1 = None;
        d_a2 = None;
    } else {
        cut_in_b = false;
        b_w = q;
        d_b = Some(0);
        a1 = r - q;
        a2 = p - 1 - (r - q);
        d_a1 = if a1 > 0 { Some(0) } else { None };
        d_a2 = if a2 > 0 { Some(0) } else { None };
        a_w = p;
        d_a = None;
        b1 = 0;
        b2 = 0;
        d_b1 = None;
        d_b2 = None;
    }

    loop {
        if cut_in_b {
            b_w = b1 + 1 + b2;
            if a_w == 1 && b_w == 1 {
                let w1 = join_depth(d_a, d_gim);
                return join_depth(d_gdom, w1).unwrap_or(0);
            }
            if a_w > b_w {
                let k = (a_w - 1) / b_w;
                let da = d_a.unwrap();
                if b2 > 0 {
                    d_b2 = Some(da.max(d_b2.unwrap()) + k);
                }
                d_gim = match d_gim {
                    None => Some(da + (k - 1)),
                    Some(dg) => Some(da.max(dg) + k),
                };
                if b1 > 0 {
                    d_b1 = Some(da.max(d_b1.unwrap()) + k);
                }
                a_w -= k * b_w;
            } else if b2 >= a_w {
                let k = b2 / a_w;
                d_a = Some(d_a.unwrap().max(d_b2.unwrap()) + k);
                b2 -= k * a_w;
                if b2 == 0 {
                    d_b2 = None;
                }
            } else {
                if b2 > 0 {
                    d_b2 = join_depth(d_a, d_b2);
                    a_w -= b2;
                }
                if a_w == 1 {
                    d_gim = join_depth(d_a, d_gim);
                    cut_in_b = false;
                    b_w = b1;
                    d_b = d_b1;
                    a1 = 0;
                    d_a1 = None;
                    a2 = b2;
                    d_a2 = d_b2;
                } else {
                    d_gim = join_depth(d_a, d_gim);
                    a_w -= 1;
                    let new_da1 = join_depth(d_a, d_b1);
                    let new_b = b1 - a_w;
                    cut_in_b = false;
                    b_w = new_b;
                    d_b = d_b1;
                    a1 = a_w;
                    d_a1 = new_da1;
                    a2 = b2;
                    d_a2 = d_b2;
                }
            }
        } else {
            a_w = a1 + 1 + a2;
            if a_w == 1 && b_w == 1 {
                let w1 = join_depth(d_gdom, d_b);
                return join_depth(w1, d_gim).unwrap_or(0);
            }
            if b_w > a_w {
                let k = (b_w - 1) / a_w;
                let db = d_b.unwrap();
                if a2 > 0 {
                    d_a2 = Some(d_a2.unwrap().max(db) + k);
                }
                d_gdom = match d_gdom {
                    None => Some(db + (k - 1)),
                    Some(dg) => Some(dg.max(db) + k),
                };
                if a1 > 0 {
                    d_a1 = Some(d_a1.unwrap().max(db) + k);
                }
                b_w -= k * a_w;
            } else if a2 >= b_w {
                let k = a2 / b_w;
                d_b = Some(d_a2.unwrap().max(d_b.unwrap()) + k);
                a2 -= k * b_w;
                if a2 == 0 {
                    d_a2 = None;
                }
            } else {
                if a2 > 0 {
                    d_a2 = join_depth(d_a2, d_b);
                    b_w -= a2;
                }
                if b_w == 1 {
                    d_gdom = join_depth(d_gdom, d_b);
                    cut_in_b = true;
                    a_w = a1;
                    d_a = d_a1;
                    b1 = 0;
                    d_b1 = None;
                    b2 = a2;
                    d_b2 = d_a2;
                } else {
                    d_gdom = join_depth(d_gdom, d_b);
                    b_w -= 1;
                    let new_db1 = join_depth(d_a1, d_b);
                    let new_a = a1 - b_w;
                    cut_in_b = true;
                    a_w = new_a;
                    d_a = d_a1;
                    b1 = b_w;
                    d_b1 = new_db1;
                    b2 = a2;
                    d_b2 = d_a2;
                }
            }
        }
    }
}

/// Instrumented plain `Open<16>` `S2b` simulator returning:
/// `(max_depth, surviving_total_len, self_loop_orphan_rows, surviving_band_lens)`
#[derive(Clone, Copy)]
struct RuleMeta {
    len: u32,
    depth: u32,
}

struct InstS2b {
    prev0: Vec<u32>,
    next0: Vec<u32>,
    prev1: Vec<u32>,
    next1: Vec<u32>,
    width: Vec<u32>,
    word: Vec<u32>,
    nodes: u32,
    rules: Vec<RuleMeta>,
    orphan_rows: u64,
    self_loop_count: u64,
}

impl InstS2b {
    fn new(cap: usize) -> Self {
        let mut rules = Vec::with_capacity(cap * 2 + 257);
        rules.push(RuleMeta { len: 0, depth: 0 });
        for _ in 0..256 {
            rules.push(RuleMeta { len: 1, depth: 0 });
        }
        Self {
            prev0: vec![0; cap],
            next0: vec![0; cap],
            prev1: vec![0; cap],
            next1: vec![0; cap],
            width: vec![0; cap],
            word: vec![0; cap],
            nodes: 0,
            rules,
            orphan_rows: 0,
            self_loop_count: 0,
        }
    }

    fn node(&mut self, w: u32, wd: u32) -> u32 {
        let id = self.nodes;
        self.nodes += 1;
        self.width[id as usize] = w;
        self.word[id as usize] = wd;
        id
    }

    fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
        let (head, tail) = list;
        if side == 0 {
            if *tail == 0 {
                *head = id;
            } else {
                self.next0[*tail as usize] = id;
            }
            self.prev0[id as usize] = *tail;
        } else {
            if *tail == 0 {
                *head = id;
            } else {
                self.next1[*tail as usize] = id;
            }
            self.prev1[id as usize] = *tail;
        }
        *tail = id;
    }

    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0u32;
        for (head, tail) in [first, second] {
            if tail == 0 {
                continue;
            }
            if side == 0 {
                self.next0[prev as usize] = head;
                self.prev0[head as usize] = prev;
            } else {
                self.next1[prev as usize] = head;
                self.prev1[head as usize] = prev;
            }
            prev = tail;
        }
        if side == 0 {
            self.next0[prev as usize] = 0;
            self.prev0[0] = prev;
        } else {
            self.next1[prev as usize] = 0;
            self.prev1[0] = prev;
        }
    }

    fn pop_tail(&mut self, id: u32, side: usize) {
        if side == 0 {
            let p = self.prev0[id as usize];
            self.next0[p as usize] = 0;
            self.prev0[0] = p;
        } else {
            let p = self.prev1[id as usize];
            self.next1[p as usize] = 0;
            self.prev1[0] = p;
        }
    }

    fn cut(&mut self, id: u32, side: usize) {
        if side == 0 {
            let p = self.prev0[id as usize];
            let n = self.next0[id as usize];
            self.next0[p as usize] = n;
            self.prev0[n as usize] = p;
        } else {
            let p = self.prev1[id as usize];
            let n = self.next1[id as usize];
            self.next1[p as usize] = n;
            self.prev1[n as usize] = p;
        }
    }

    fn insert(&mut self, at: u32, id: u32, side: usize) {
        if side == 0 {
            let n = self.next0[at as usize];
            self.prev0[id as usize] = at;
            self.next0[id as usize] = n;
            self.next0[at as usize] = id;
            self.prev0[n as usize] = id;
        } else {
            let n = self.next1[at as usize];
            self.prev1[id as usize] = at;
            self.next1[id as usize] = n;
            self.next1[at as usize] = id;
            self.prev1[n as usize] = id;
        }
    }

    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let ml = self.rules[left as usize];
        let mr = self.rules[right as usize];
        let id = self.rules.len() as u32;
        self.rules.push(RuleMeta {
            len: ml.len + mr.len,
            depth: 1 + ml.depth.max(mr.depth),
        });
        id
    }

    fn eliminate(&mut self, n: u32) {
        let mut left = n;
        while left > 0 {
            let x = self.prev0[0];
            let y = self.prev1[0];
            let aw = self.width[x as usize];
            let bw = self.width[y as usize];
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let wlen = self.rules[self.word[x as usize] as usize].len;
                self.orphan_rows += u64::from(aw) * u64::from(wlen);
                self.self_loop_count += 1;
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            let wd = self.join(self.word[x as usize], self.word[y as usize]);
            if aw <= bw {
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.word[x as usize] = wd;
                if aw == bw {
                    self.cut(y, 0);
                    self.pop_tail(y, 1);
                } else {
                    self.width[y as usize] = bw - w;
                }
            } else {
                self.pop_tail(y, 1);
                self.insert(x, y, 1);
                self.word[y as usize] = wd;
                self.width[x as usize] = aw - w;
            }
        }
    }
}

fn run_inst_s2b(l: &[u8], p: u32, limit: u32) -> (u32, u64, u64, u64, u64) {
    let n = l.len() as u32;
    let mut thin = [0u32; 256];
    let mut thick = [0u32; 256];
    let mut thick_rows = [0u32; 256];
    let mut strips: Vec<(u32, u32, u32, u8)> = Vec::new();
    let mut m = 0u32;
    let mut add_piece = |c: u8, x: u32, w: u32, keep: bool| {
        let a = usize::from(c);
        if keep {
            thin[a] += w;
            m += w;
        } else {
            strips.push((x, thin[a] + thick_rows[a], w, c));
            thick[a] += 1;
            thick_rows[a] += w;
        }
    };
    let mut pos = 0usize;
    while pos < l.len() {
        let c = l[pos];
        let mut end = pos + 1;
        while end < l.len() && l[end] == c {
            end += 1;
        }
        let width = (end - pos) as u32;
        let pos_u = pos as u32;
        if width <= limit {
            add_piece(c, pos_u, width, true);
        } else if p.wrapping_sub(pos_u) >= width {
            add_piece(c, pos_u, width, false);
        } else {
            if pos_u < p {
                add_piece(c, pos_u, p - pos_u, false);
            }
            add_piece(c, p, 1, true);
            if p + 1 < pos_u + width {
                add_piece(c, p + 1, pos_u + width - p - 1, false);
            }
        }
        pos = end;
    }
    let mut first = [0u32; 256];
    let mut tot = 0u32;
    for a in 0..256 {
        first[a] = tot;
        tot += thin[a] + thick_rows[a];
    }
    for s in &mut strips {
        s.1 += first[usize::from(s.3)];
    }
    let ns = strips.len();
    let mut write = [0u32; 256];
    let mut tw = 0u32;
    for a in 0..256 {
        write[a] = tw;
        tw += thick[a];
    }
    let mut image_order = vec![0u32; ns];
    for (i, &(_, _, _, byte)) in strips.iter().enumerate() {
        let a = usize::from(byte);
        image_order[write[a] as usize] = i as u32;
        write[a] += 1;
    }

    let most = 3 * ns + 3;
    let mut e = InstS2b::new(most);
    e.node(0, 0);
    e.nodes = ns as u32 + 1;
    let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
    let mut x = 0u32;
    for (idx, &(from, _, width, byte)) in strips.iter().enumerate() {
        let sid = idx as u32 + 1;
        e.width[sid as usize] = width;
        e.word[sid as usize] = u32::from(byte) + 1;
        if x < from {
            let gid = e.node(from - x, 0);
            e.append_to(&mut order[0], gid, 0);
            e.append_to(&mut terminals[1], gid, 1);
        }
        e.append_to(&mut order[0], sid, 0);
        x = from + width;
    }
    if x < n {
        let gid = e.node(n - x, 0);
        e.append_to(&mut order[0], gid, 0);
        e.append_to(&mut terminals[1], gid, 1);
    }
    x = 0;
    for &idx in &image_order {
        let (_, to, width, _) = strips[idx as usize];
        if x < to {
            let gid = e.node(to - x, 0);
            e.append_to(&mut order[1], gid, 1);
            e.append_to(&mut terminals[0], gid, 0);
        }
        e.append_to(&mut order[1], idx + 1, 1);
        x = to + width;
    }
    if x < n {
        let gid = e.node(n - x, 0);
        e.append_to(&mut order[1], gid, 1);
        e.append_to(&mut terminals[0], gid, 0);
    }
    for side in 0..2 {
        e.close(side, terminals[side], order[side]);
    }
    e.eliminate(n);

    // Every surviving node on side 0 has width `w` retained rows, each emitting `len(word) + 1` bytes!
    let mut max_depth = 0u32;
    let mut surv_bytes = 0u64;
    let mut surv_rows = 0u64;
    let mut cur = e.next0[0];
    while cur != 0 {
        let w = u64::from(e.width[cur as usize]);
        let wd = e.word[cur as usize];
        let meta = e.rules[wd as usize];
        max_depth = max_depth.max(meta.depth);
        surv_rows += w;
        surv_bytes += w * u64::from(meta.len + 1);
        cur = e.next0[cur as usize];
    }
    assert_eq!(surv_rows, u64::from(m));
    (max_depth, surv_bytes, e.orphan_rows, e.self_loop_count, u64::from(m))
}

/// Exact FQ2 cut-tower depth verifier for Christoffel words (proving D_FQ2 == sum(a_i) for all primaries).
fn fq2_depth_christoffel(p: usize, q: usize, r: u32) -> u64 {
    let n = p + q;
    let mut pi: Vec<u32> = (0..n as u32).map(|x| (x + q as u32) % (n as u32)).collect();
    let mut fb: Vec<u8> = Vec::with_capacity(n);
    fb.extend(std::iter::repeat_n(b'a', p));
    fb.extend(std::iter::repeat_n(b'b', q));
    let mut mark = r as usize;
    let mut m = n;
    let mut depth = 0u64;
    while m > 1 {
        let mut bands = Vec::new();
        let mut i = 0usize;
        while i < m {
            let mut j = i + 1;
            while j < m && pi[j] == pi[j - 1] + 1 && fb[j] == fb[i] {
                j += 1;
            }
            bands.push((i as u32, pi[i], (j - i) as u32));
            i = j;
        }
        let b = bands.len();
        let mut cod: Vec<usize> = (0..b).collect();
        cod.sort_unstable_by_key(|&k| bands[k].1);
        let mut edges: Vec<(usize, usize, u32, u32, bool)> = Vec::new();
        let (mut ci, mut di) = (0usize, 0usize);
        let mut y = 0u32;
        while ci < b && di < b {
            let v = cod[ci];
            let u = di;
            let ce = bands[v].1 + bands[v].2;
            let de = bands[u].0 + bands[u].2;
            let y1 = ce.min(de);
            if y1 > y {
                edges.push((v, u, y, y1 - y, (mark as u32) >= y && (mark as u32) < y1));
            }
            y = y1;
            if ce == y1 {
                ci += 1;
            }
            if de == y1 {
                di += 1;
            }
        }
        let mut order: Vec<usize> = (0..edges.len()).collect();
        order.sort_unstable_by(|&a, &b_i| {
            edges[b_i]
                .3
                .cmp(&edges[a].3)
                .then_with(|| edges[a].4.cmp(&edges[b_i].4))
                .then_with(|| a.cmp(&b_i))
        });
        let mut par: Vec<usize> = (0..b).collect();
        let mut in_f = vec![false; edges.len()];
        for &ei in &order {
            let (mut u, mut v) = (edges[ei].0, edges[ei].1);
            while par[u] != u {
                u = par[u];
            }
            while par[v] != v {
                v = par[v];
            }
            if u != v {
                par[u] = v;
                in_f[ei] = true;
            }
        }
        let mut ret = vec![false; m];
        for (ei, &(_, _, y0, w, _)) in edges.iter().enumerate() {
            if !in_f[ei] {
                for row in (y0 as usize)..((y0 + w) as usize) {
                    ret[row] = true;
                }
            }
        }
        ret[mark] = true;
        let mut rank = vec![u32::MAX; m];
        let mut rows = Vec::new();
        for (x, &keep) in ret.iter().enumerate() {
            if keep {
                rank[x] = rows.len() as u32;
                rows.push(x);
            }
        }
        let m_next = rows.len();
        let mut pi_next = vec![0u32; m_next];
        let mut fb_next = vec![0u8; m_next];
        for (ni, &sx) in rows.iter().enumerate() {
            fb_next[ni] = fb[sx];
            let mut cur = pi[sx] as usize;
            while !ret[cur] {
                cur = pi[cur] as usize;
            }
            pi_next[ni] = rank[cur];
        }
        mark = rank[mark] as usize;
        pi = pi_next;
        fb = fb_next;
        m = m_next;
        depth += 1;
    }
    depth
}

fn verify_part1_3strip_law() {
    println!("================================================================================");
    println!("PART 1 — EXACT RESOLUTION OF CYCLE 50 PROBE 2: KAKUTANI (1943) 3-STRIP");
    println!("         INTERIOR-PRIMARY CHRISTOFFEL S2b DEPTH LAW (EXTENDING CA-40)");
    println!("================================================================================");

    let mut rng = XorShift64::new(0x51C5_1709_A0C1_1943);
    let mut grid_triples = 0usize;
    let mut grid_pairs = 0usize;
    let mut grid_exact_eq = 0usize;
    let mut grid_env_viol = 0usize;
    let mut grid_markov_triples = 0usize;
    let mut grid_markov_viol = 0usize;
    let mut max_drop = 0i64;
    let mut max_drop_case = (0usize, 0usize, 0u32, 0u64, 0u64, 0u64);
    let mut max_above = 0i64;
    let mut probe2_gt1 = 0usize;
    let mut probe2_total = 0usize;

    // 1A. Full Referee Probe 2 Grid (17 <= p, q <= 60, gcd(p, q) == 1) x 7 Probe-2 primaries
    //     + 2 extra deterministic + 2 random interior primaries = 11 primaries/pair (~12,500+ triples!)
    for p in 17..=60usize {
        for q in 17..=60usize {
            if gcd_u64(p as u64, q as u64) != 1 {
                continue;
            }
            grid_pairs += 1;
            let n = p + q;
            let cf = continued_fraction(p as u64, q as u64);
            let cf_sum: u64 = cf.iter().sum();
            let a_last = *cf.last().unwrap();
            let max_ai = *cf.iter().max().unwrap();

            let mut l = vec![b'b'; q];
            l.extend(std::iter::repeat_n(b'a', p));

            // Exact 7 Probe-2 primaries
            let probe7 = [0usize, 1, q / 2, q - 1, q, q + p / 2, n - 1];
            for &r in &probe7 {
                let (h_s2b, surv_b, orph_b, _, m_out) = run_inst_s2b(&l, r as u32, LIMIT);
                assert_eq!(m_out, 1);
                assert_eq!(surv_b, n as u64);
                assert_eq!(orph_b, 0);
                let h_pred = kakutani_3strip_depth_exact(p as u64, q as u64, r as u64);
                assert_eq!(
                    u64::from(h_s2b),
                    h_pred,
                    "Kakutani 3-strip law mismatch at ({p}, {q}, r={r})"
                );
                let diff = h_s2b as i64 - cf_sum as i64;
                probe2_total += 1;
                if diff.abs() > 1 {
                    probe2_gt1 += 1;
                }
                if diff < max_drop {
                    max_drop = diff;
                    max_drop_case = (p, q, r as u32, u64::from(h_s2b), cf_sum, a_last);
                }
                max_above = max_above.max(diff);
                if (u64::from(h_s2b)) < cf_sum - a_last / 2 || (u64::from(h_s2b)) > cf_sum + 1 {
                    grid_env_viol += 1;
                }
                if max_ai <= 2 {
                    grid_markov_triples += 1;
                    if diff.abs() > 1 {
                        grid_markov_viol += 1;
                    }
                }
                grid_exact_eq += 1;
                grid_triples += 1;
            }

            // Plus 4 extra interior primaries (2 structured + 2 random interior!)
            let extra = [
                (p * q) % n,
                n / 3,
                1 + rng.gen_range(q - 1),
                q + 1 + rng.gen_range(p - 1),
            ];
            for &r in &extra {
                let (h_s2b, surv_b, orph_b, _, _) = run_inst_s2b(&l, r as u32, LIMIT);
                assert_eq!(surv_b, n as u64);
                assert_eq!(orph_b, 0);
                let h_pred = kakutani_3strip_depth_exact(p as u64, q as u64, r as u64);
                assert_eq!(u64::from(h_s2b), h_pred);
                let diff = h_s2b as i64 - cf_sum as i64;
                if diff < max_drop {
                    max_drop = diff;
                    max_drop_case = (p, q, r as u32, u64::from(h_s2b), cf_sum, a_last);
                }
                max_above = max_above.max(diff);
                if (u64::from(h_s2b)) < cf_sum - a_last / 2 || (u64::from(h_s2b)) > cf_sum + 1 {
                    grid_env_viol += 1;
                }
                if max_ai <= 2 {
                    grid_markov_triples += 1;
                    if diff.abs() > 1 {
                        grid_markov_viol += 1;
                    }
                }
                grid_exact_eq += 1;
                grid_triples += 1;
            }
        }
    }

    // 1B. Exhaustive ALL r in 0..p+q across ALL 17 <= p, q <= 32 (7,990 triples!)
    let mut exh_triples = 0usize;
    let mut exh_fq2_checked = 0usize;
    for p in 17..=32usize {
        for q in 17..=32usize {
            if gcd_u64(p as u64, q as u64) != 1 {
                continue;
            }
            let n = p + q;
            let cf = continued_fraction(p as u64, q as u64);
            let cf_sum: u64 = cf.iter().sum();
            let a_last = *cf.last().unwrap();
            let mut l = vec![b'b'; q];
            l.extend(std::iter::repeat_n(b'a', p));
            for r in 0..n {
                let (h_s2b, _, _, _, _) = run_inst_s2b(&l, r as u32, LIMIT);
                let h_pred = kakutani_3strip_depth_exact(p as u64, q as u64, r as u64);
                assert_eq!(u64::from(h_s2b), h_pred);
                assert!(u64::from(h_s2b) >= cf_sum - a_last / 2 && u64::from(h_s2b) <= cf_sum + 1);
                exh_triples += 1;
            }
            // Verify FQ2 depth == sum(a_i) at interior primaries (q/2, q + p/2)
            for &r in &[0u32, (q / 2) as u32, (q + p / 2) as u32] {
                assert_eq!(fq2_depth_christoffel(p, q, r), cf_sum);
                exh_fq2_checked += 1;
            }
        }
    }

    // 1C. Thick Markov–Christoffel Words (M <= 10^35 with p,q > 16: 260 genuine thick pairs x 7 primaries!)
    let markov_nodes = generate_markov_slopes(100_000_000_000_000_000_000_000_000_000_000_000u128);
    let mut mk_triples = 0usize;
    let mut mk_pairs = 0usize;
    let mut mk_fq2 = 0usize;
    for &(_z, p, q) in &markov_nodes {
        if p <= LIMIT as usize || q <= LIMIT as usize {
            continue;
        }
        mk_pairs += 1;
        let n = p + q;
        let cf = continued_fraction(p as u64, q as u64);
        let cf_sum: u64 = cf.iter().sum();
        let a_last = *cf.last().unwrap();
        let mut l = vec![b'b'; q];
        l.extend(std::iter::repeat_n(b'a', p));
        for r in [0, 1, q / 2, q - 1, q, q + p / 2, n - 1] {
            let (h_s2b, _, _, _, _) = run_inst_s2b(&l, r as u32, LIMIT);
            let h_pred = kakutani_3strip_depth_exact(p as u64, q as u64, r as u64);
            assert_eq!(u64::from(h_s2b), h_pred);
            assert!(u64::from(h_s2b) >= cf_sum - a_last / 2 && u64::from(h_s2b) <= cf_sum + 1);
            mk_triples += 1;
        }
        if n <= 120 {
            assert_eq!(fq2_depth_christoffel(p, q, (q / 2) as u32), cf_sum);
            mk_fq2 += 1;
        }
    }

    let fib = fib_u_seq(30);
    let (pell_p, pell_q) = from_cf(&[2, 2, 2, 2, 2, 2, 2, 2]);
    let (mk5_p, mk5_q) = from_cf(&[1, 1, 2, 2, 1, 1, 2, 2, 1, 1, 2, 2]);
    let (hp1_p, hp1_q) = from_cf(&[1, 50, 20]);
    let (hp2_p, hp2_q) = from_cf(&[1, 50, 50]);
    let (hp3_p, hp3_q) = from_cf(&[1, 20, 4]);
    let (hp4_p, hp4_q) = from_cf(&[1, 15, 3]);

    let named: Vec<(&str, usize, usize)> = vec![
        ("Probe2 (59, 60) [0;1,59]", 59, 60),
        ("Probe2 [1; 50, 20]", hp1_p, hp1_q),
        ("High-a1,small-al [1;20,4]", hp3_p, hp3_q),
        ("High-a1,small-al [1;15,3]", hp4_p, hp4_q),
        ("Equal big [1; 50, 50]", hp2_p, hp2_q),
        ("Lucas Fib F_18 (a=1)", fib[17] as usize, fib[16] as usize),
        ("Lucas Fib F_22 (a=1)", fib[21] as usize, fib[20] as usize),
        ("Pell [2; 2^7] (a=2)", pell_p, pell_q),
        ("Markov M=5 (1122)^3", mk5_p, mk5_q),
    ];

    println!("  [Part 1A] Probe-2 Grid (17<=p,q<=60, {grid_pairs} coprime pairs) x 11 primaries = {grid_triples} triples:");
    println!("    * Exact Kakutani 3-strip O(log n) law:  {grid_exact_eq}/{grid_triples} (100.000% EXACT, 0 mismatches!)");
    println!("    * Probe-2 7-primary subset:             {probe2_gt1}/{probe2_total} ({:.2}%) have |h - sum| > 1", pct(probe2_gt1 as u64, probe2_total as u64));
    println!("    * Worst interior drop observed:         (p={}, q={}, r={}) -> h_S2b={} vs sum={} (diff={:+}, -floor(a_l/2)={:+})",
        max_drop_case.0, max_drop_case.1, max_drop_case.2, max_drop_case.3, max_drop_case.4, max_drop, -(max_drop_case.5 as i64 / 2));
    println!("    * Sharp universal envelope [sum - floor(a_l/2), sum + 1]: {grid_env_viol} violations (max diff = {max_above:+})");
    println!("    * Markov-Zone Rigidity Corollary (max(a_i)<=2 => |h-sum|<=1 ALL r): {grid_markov_viol}/{grid_markov_triples} violations!");
    println!("  [Part 1B] Exhaustive ALL r in 0..p+q (17<=p,q<=32): {exh_triples}/{exh_triples} (0 mismatches) & {exh_fq2_checked}/{exh_fq2_checked} FQ2==sum(a_i)!");
    println!("  [Part 1C] Thick Markov-Christoffel (M<=10^35, {mk_pairs} thick pairs x 7 primaries): {mk_triples}/{mk_triples} exact S2b & {mk_fq2}/{mk_fq2} FQ2==sum(a_i)!");

    println!("\n  [Named Family Contrast — Proving ONLY a_ell Splits While Earlier a_k Remain Invariant]:");
    println!(
        "  {:<26} {:>6} {:>5} {:>4} | {:>12} {:>12} | {:>10} {:>6}",
        "Christoffel Family", "n=p+q", "sum_a", "a_l", "h_S2b [min..max]", "diff [min..max]", "-fl(a_l/2)", "Err"
    );
    for (label, p, q) in named {
        let n = p + q;
        let cf = continued_fraction(p as u64, q as u64);
        let cf_sum: u64 = cf.iter().sum();
        let a_last = *cf.last().unwrap();
        let mut l = vec![b'b'; q];
        l.extend(std::iter::repeat_n(b'a', p));
        // Sample all r if n <= 200 else 64 dense+key primaries including 0..a_last+2 and q/2, q+p/2
        let mut rs: Vec<usize> = if n <= 200 {
            (0..n).collect()
        } else {
            let mut v: Vec<usize> = (0..=(a_last as usize + 2).min(n - 1)).collect();
            for k in 0..16 {
                v.push((n * k) / 16);
            }
            v.extend([q / 2, q - 1, q, q + p / 2, n - 1]);
            v.sort_unstable();
            v.dedup();
            v
        };
        let mut mn = u64::MAX;
        let mut mx = 0u64;
        let mut errs = 0usize;
        for &r in &rs {
            let (h_s2b, _, _, _, _) = run_inst_s2b(&l, r as u32, LIMIT);
            let h_pred = kakutani_3strip_depth_exact(p as u64, q as u64, r as u64);
            if u64::from(h_s2b) != h_pred {
                errs += 1;
            }
            mn = mn.min(u64::from(h_s2b));
            mx = mx.max(u64::from(h_s2b));
        }
        assert_eq!(errs, 0);
        // Also verify FQ2 depth == sum(a_i) at interior r = q/2 for n <= 2500
        if n <= 2500 {
            assert_eq!(fq2_depth_christoffel(p, q, (q / 2) as u32), cf_sum);
        }
        rs.clear();
        println!(
            "  {:<26} {:>6} {:>5} {:>4} | {:>5}..{:<6} {:>+5}..{:<+6} | {:>+10} {:>6}",
            label,
            n,
            cf_sum,
            a_last,
            mn,
            mx,
            mn as i64 - cf_sum as i64,
            mx as i64 - cf_sum as i64,
            -(a_last as i64 / 2),
            errs
        );
    }
    println!();
}

// ============================================================================
// 2. PART 2 — Exact Kakutani (1943) Skyscraper Height Spectrum &
//    Birkhoff (1931) / Hopf (1937) Cycle Partition on Open<16>
// ============================================================================

struct SkyscraperSummary {
    n: u64,
    m: u64,
    cycles_total: u64,
    cycles_sampled: u64,
    cycles_orphan: u64,
    orphan_rows: u64,
    sum_tau: u64,
    max_tau: u64,
    // Row counts by height stratum
    rows_tau1: u64,
    rows_tau2_15: u64,
    rows_tau16: u64,
    rows_tau17p: u64,
    rows_tau64p: u64,
    // Byte sums by height stratum
    bytes_tau1: u64,
    bytes_tau2_15: u64,
    bytes_tau16: u64,
    bytes_tau17p: u64,
    bytes_tau64p: u64,
    gini: f64,
    h_row_bits: f64,
    h_byte_bits: f64,
}

/// Exact direct FL Kakutani skyscraper + Birkhoff–Hopf ergodic cycle analyzer.
/// Returns `(summary, ret_mask, fl, tau_of_ret_idx, next_ret_idx_of_ret_idx, mark_ret_idx)`.
fn analyze_kakutani_birkhoff_hopf(
    l: &[u8],
    p: u32,
    limit: usize,
    verify_s2b_orphan: bool,
) -> (SkyscraperSummary, Vec<u32>, Vec<u32>, Vec<u32>, u32) {
    let n = l.len();
    // 1. Build exact FL permutation on [0, n)
    let mut cnt = [0u32; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for a in 0..256 {
        starts[a] = tot;
        tot += cnt[a];
    }
    let mut fl = vec![0u32; n];
    let mut cur = starts;
    for (i, &b) in l.iter().enumerate() {
        let a = usize::from(b);
        fl[cur[a] as usize] = i as u32;
        cur[a] += 1;
    }

    // 2. Build exact Open<16> retained base R (thin runs w <= limit + primary p)
    let mut in_r = vec![false; n];
    let mut pos = 0usize;
    while pos < n {
        let b = l[pos];
        let mut end = pos + 1;
        while end < n && l[end] == b {
            end += 1;
        }
        if end - pos <= limit {
            for r in pos..end {
                in_r[r] = true;
            }
        }
        pos = end;
    }
    in_r[p as usize] = true;

    let mut rank = vec![u32::MAX; n];
    let mut ret_rows = Vec::new();
    for (x, &keep) in in_r.iter().enumerate() {
        if keep {
            rank[x] = ret_rows.len() as u32;
            ret_rows.push(x as u32);
        }
    }
    let m = ret_rows.len();
    let mark_idx = rank[p as usize];

    // 3. Birkhoff–Hopf disjoint cycle decomposition + exact Kakutani skyscraper partition check!
    let mut visited = vec![false; n];
    let mut covered_by_tower = vec![false; n];
    let mut tau_by_idx = vec![0u32; m];
    let mut next_r1 = vec![0u32; m];
    let mut cycles_total = 0u64;
    let mut cycles_sampled = 0u64;
    let mut cycles_orphan = 0u64;
    let mut orphan_rows = 0u64;
    let mut sum_tau = 0u64;

    for start in 0..n {
        if visited[start] {
            continue;
        }
        cycles_total += 1;
        let mut cyc = Vec::new();
        let mut x = start;
        while !visited[x] {
            visited[x] = true;
            cyc.push(x);
            x = fl[x] as usize;
        }
        let cyc_len = cyc.len();
        // Find all base points in R \cap C_i
        let mut r_positions = Vec::new();
        for (pos_in_c, &v) in cyc.iter().enumerate() {
            if in_r[v] {
                r_positions.push(pos_in_c);
            }
        }
        if r_positions.is_empty() {
            cycles_orphan += 1;
            orphan_rows += cyc_len as u64;
        } else {
            cycles_sampled += 1;
            let k_r = r_positions.len();
            let mut cyc_tau_sum = 0u64;
            for j in 0..k_r {
                let p0 = r_positions[j];
                let p1 = r_positions[(j + 1) % k_r];
                let tau = if j + 1 < k_r { p1 - p0 } else { cyc_len - p0 + p1 };
                let u_row = cyc[p0];
                let v_row = cyc[p1];
                let u_idx = rank[u_row] as usize;
                let v_idx = rank[v_row];
                tau_by_idx[u_idx] = tau as u32;
                next_r1[u_idx] = v_idx;
                cyc_tau_sum += tau as u64;
                // Verify exact Kakutani skyscraper column disjointness: T^t(u) for t = 0..tau-1
                for step in 0..tau {
                    let w = cyc[(p0 + step) % cyc_len];
                    assert!(!covered_by_tower[w], "Kakutani skyscraper overlap!");
                    covered_by_tower[w] = true;
                }
            }
            // Exact Birkhoff–Hopf cycle-by-cycle Kakutani height-sum conservation!
            assert_eq!(
                cyc_tau_sum, cyc_len as u64,
                "Birkhoff-Hopf cycle conservation violated!"
            );
            sum_tau += cyc_tau_sum;
        }
    }
    // Global Kakutani–Birkhoff–Hopf conservation: sum_{x in R} tau_R(x) + orphan_rows == n
    assert_eq!(sum_tau + orphan_rows, n as u64);
    let cov_cnt = covered_by_tower.iter().filter(|&&b| b).count() as u64;
    assert_eq!(cov_cnt, sum_tau);

    // Optional exact cross-check against Open<16> S2b self-loop orphan conservation!
    if verify_s2b_orphan {
        let (_md, surv_b, s2b_orph, _sl, s2b_m) = run_inst_s2b(l, p, limit as u32);
        assert_eq!(s2b_m, m as u64);
        assert_eq!(surv_b, sum_tau, "S2b surviving bytes != sum(tau_R)!");
        assert_eq!(s2b_orph, orphan_rows, "S2b self-loop mass != Birkhoff-Hopf orphan cycles!");
        assert_eq!(surv_b + s2b_orph, n as u64);
    }

    // 4. Exact height histogram, strata, Gini coefficient, and Shannon entropy
    let mut hist: BTreeMap<u32, u64> = BTreeMap::new();
    let mut max_tau = 0u64;
    let mut rows_tau1 = 0u64;
    let mut rows_tau2_15 = 0u64;
    let mut rows_tau16 = 0u64;
    let mut rows_tau17p = 0u64;
    let mut rows_tau64p = 0u64;
    let mut bytes_tau1 = 0u64;
    let mut bytes_tau2_15 = 0u64;
    let mut bytes_tau16 = 0u64;
    let mut bytes_tau17p = 0u64;
    let mut bytes_tau64p = 0u64;

    for &t in &tau_by_idx {
        let tu = u64::from(t);
        max_tau = max_tau.max(tu);
        *hist.entry(t).or_insert(0) += 1;
        if t == 1 {
            rows_tau1 += 1;
            bytes_tau1 += tu;
        } else if t <= 15 {
            rows_tau2_15 += 1;
            bytes_tau2_15 += tu;
        } else if t == 16 {
            rows_tau16 += 1;
            bytes_tau16 += tu;
        } else {
            rows_tau17p += 1;
            bytes_tau17p += tu;
        }
        if t >= 64 {
            rows_tau64p += 1;
            bytes_tau64p += tu;
        }
    }

    // Exact Gini coefficient from sorted histogram:
    // G = (2 * sum_{i=1}^m i * tau_{(i)}) / (m * sum tau) - (m + 1) / m
    let mut cum_idx = 0u128;
    let mut weighted_sum = 0u128;
    let mut h_row_bits = 0.0f64;
    let mut h_byte_bits = 0.0f64;
    let mf = m as f64;
    let sf = sum_tau.max(1) as f64;

    for (&k, &c) in &hist {
        // Indices for these c equal items are (cum_idx + 1) ..= (cum_idx + c)
        let c128 = u128::from(c);
        let idx_sum = c128 * (2 * cum_idx + c128 + 1) / 2;
        weighted_sum += idx_sum * u128::from(k);
        cum_idx += c128;

        let pr = c as f64 / mf;
        if pr > 0.0 {
            h_row_bits -= pr * pr.log2();
        }
        let pb = (c * u64::from(k)) as f64 / sf;
        if pb > 0.0 {
            h_byte_bits -= pb * pb.log2();
        }
    }
    let gini = if m > 0 && sum_tau > 0 {
        (2.0 * weighted_sum as f64) / (mf * sf) - (mf + 1.0) / mf
    } else {
        0.0
    };

    let summary = SkyscraperSummary {
        n: n as u64,
        m: m as u64,
        cycles_total,
        cycles_sampled,
        cycles_orphan,
        orphan_rows,
        sum_tau,
        max_tau,
        rows_tau1,
        rows_tau2_15,
        rows_tau16,
        rows_tau17p,
        rows_tau64p,
        bytes_tau1,
        bytes_tau2_15,
        bytes_tau16,
        bytes_tau17p,
        bytes_tau64p,
        gini,
        h_row_bits,
        h_byte_bits,
    };
    (summary, fl, ret_rows, next_r1, mark_idx)
}

/// Generate 5,200 small valid BWTs + 300 malformed multi-cycle columns (incl. 150 deliberate orphan pure-thick cycles!).
fn generate_small_and_malformed_suites() -> (Vec<(Vec<u8>, u32)>, Vec<(Vec<u8>, u32)>) {
    let mut rng = XorShift64::new(0xC51B_1240_DEAF_1937);
    let mut valid = Vec::with_capacity(5200);
    // 5,000 random valid BWTs across diverse alphabets & lengths (including thick runs!)
    for idx in 0..5000usize {
        let sigma = match idx % 7 {
            0 => 2,
            1 => 3,
            2 => 4,
            3 => 6,
            4 => 12,
            5 => 26,
            _ => 64,
        };
        let n = 4 + rng.gen_range(96);
        let mut text = Vec::with_capacity(n);
        if idx % 5 == 0 {
            // Repetitive / run-heavy text to trigger thick strips even at small n!
            let base_len = 2 + rng.gen_range(6);
            let mut pat = Vec::with_capacity(base_len);
            for _ in 0..base_len {
                pat.push(b'a' + (rng.gen_range(sigma.min(26)) as u8));
            }
            while text.len() < n {
                text.push(pat[text.len() % base_len]);
            }
            // Add a small perturbation on half to make primitive
            if idx % 10 == 0 && n > 2 {
                text[0] = b'z';
            }
        } else {
            for _ in 0..n {
                text.push(b'!' + (rng.gen_range(sigma) as u8));
            }
        }
        valid.push(bwt_small(&text));
    }
    // 200 larger thick-run valid BWTs (n = 80..180)
    for idx in 0..200usize {
        let n = 80 + rng.gen_range(100);
        let mut text = vec![b'a'; n];
        for i in 0..n {
            text[i] = match (i + idx) % 5 {
                0 => b'a',
                1 => b'b',
                2 => b'a',
                3 => b'c',
                _ => b'a' + (rng.gen_range(4) as u8),
            };
        }
        valid.push(bwt_small(&text));
    }

    // 300 malformed multi-cycle columns:
    //   150 random raw columns (natural multi-cycle FL)
    //   150 deliberately crafted columns with ORPHAN PURE-THICK CYCLES (R \cap C_i = \emptyset!)
    let mut malformed = Vec::with_capacity(300);
    for _ in 0..150 {
        let n = 10 + rng.gen_range(120);
        let sigma = 2 + rng.gen_range(8);
        let mut l = Vec::with_capacity(n);
        for _ in 0..n {
            l.push(b'a' + (rng.gen_range(sigma) as u8));
        }
        let p = rng.gen_range(n) as u32;
        malformed.push((l, p));
    }
    for idx in 0..150usize {
        let mut l = Vec::new();
        match idx % 3 {
            0 => {
                // Fixed-point orphan pure-thick cycle: a^k at [0..k) where L[i] == F[i] == 'a' (k >= 17),
                // followed by mixed thin/thick tail containing primary p!
                let k = 17 + (idx % 25);
                l.extend(std::iter::repeat_n(b'a', k));
                let tail_n = 20 + (idx % 40);
                for _ in 0..tail_n {
                    l.push(b'b' + (rng.gen_range(5) as u8));
                }
                let p = (k + rng.gen_range(tail_n)) as u32;
                malformed.push((l, p));
            }
            1 => {
                // 2-run thick Christoffel orphan component: 'b'^q 'a'^p (p, q >= 17) as a disjoint prefix
                // over {'a', 'b'}, followed by {'c', 'd', 'e'} segment where primary sits!
                let pa = 17 + (idx % 12);
                let qb = 18 + (idx % 13);
                l.extend(std::iter::repeat_n(b'b', qb));
                l.extend(std::iter::repeat_n(b'a', pa));
                let tail_n = 15 + (idx % 30);
                for _ in 0..tail_n {
                    l.push(b'c' + (rng.gen_range(4) as u8));
                }
                let p = (pa + qb + rng.gen_range(tail_n)) as u32;
                malformed.push((l, p));
            }
            _ => {
                // Multiple disjoint orphan pure-thick blocks + thin middle!
                let k1 = 18 + (idx % 10);
                let k2 = 19 + (idx % 10);
                l.extend(std::iter::repeat_n(b'a', k1));
                let mid = 12 + (idx % 20);
                for _ in 0..mid {
                    l.push(b'b' + (rng.gen_range(4) as u8));
                }
                l.extend(std::iter::repeat_n(b'z', k2));
                let p = (k1 + rng.gen_range(mid)) as u32;
                malformed.push((l, p));
            }
        }
    }
    (valid, malformed)
}

fn verify_part2_skyscraper_and_birkhoff_hopf(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    corpora: &[(&str, &[BlockData], bool)],
) {
    println!("================================================================================");
    println!("PART 2 — EXACT KAKUTANI (1943) SKYSCRAPER HEIGHT SPECTRUM & BIRKHOFF (1931) /");
    println!("         HOPF (1937) CYCLE PARTITION ON Open<16>");
    println!("================================================================================");

    // 2A. 5,200 small valid BWTs
    let mut val_single_cyc = 0usize;
    let mut val_multi_cyc = 0usize;
    let mut val_orph_blks = 0usize;
    let mut val_sampled_cycles = 0u64;
    for (l, p) in valid {
        let (s, _, _, _, _) = analyze_kakutani_birkhoff_hopf(l, *p, LIMIT as usize, true);
        assert_eq!(s.cycles_sampled + s.cycles_orphan, s.cycles_total);
        val_sampled_cycles += s.cycles_sampled;
        if s.cycles_total == 1 {
            val_single_cyc += 1;
            assert_eq!(s.sum_tau, s.n);
            assert_eq!(s.orphan_rows, 0);
        } else {
            val_multi_cyc += 1;
        }
        if s.cycles_orphan > 0 {
            val_orph_blks += 1;
        }
    }
    println!(
        "  [PASS] Part 2A: {}/{} small valid BWTs 100% EXACT (single-cycle={val_single_cyc}, periodic multi-cycle={val_multi_cyc}, sampled-cycles={val_sampled_cycles}, orphan-blks={val_orph_blks})!",
        valid.len(),
        valid.len()
    );

    // 2B. 300 malformed multi-cycle columns (including deliberate orphan pure-thick cycles!)
    let mut mal_multi = 0usize;
    let mut mal_with_orphans = 0usize;
    let mut mal_sampled_cycles = 0u64;
    let mut mal_orphan_cycles = 0u64;
    let mut mal_orphan_rows = 0u64;
    let mut mal_total_rows = 0u64;
    for (l, p) in malformed {
        let (s, _, _, _, _) = analyze_kakutani_birkhoff_hopf(l, *p, LIMIT as usize, true);
        assert_eq!(s.cycles_sampled + s.cycles_orphan, s.cycles_total);
        mal_sampled_cycles += s.cycles_sampled;
        if s.cycles_total > 1 {
            mal_multi += 1;
        }
        if s.cycles_orphan > 0 {
            mal_with_orphans += 1;
            mal_orphan_cycles += s.cycles_orphan;
            mal_orphan_rows += s.orphan_rows;
        }
        mal_total_rows += s.n;
    }
    println!(
        "  [PASS] Part 2B: {}/{} malformed columns 100% EXACT Birkhoff-Hopf & S2b self-loop orphan conservation!",
        malformed.len(),
        malformed.len()
    );
    println!(
        "         multi-cycle={mal_multi}/300, sampled cycles={mal_sampled_cycles}, with orphan pure-thick cycles={mal_with_orphans}/300 ({mal_orphan_cycles} orphan cycles, {mal_orphan_rows}/{mal_total_rows} rows [{:.2}%])!",
        pct(mal_orphan_rows, mal_total_rows)
    );

    // 2C. ALL 249 Real Blocks — Exact Kakutani Skyscraper Height Spectrum!
    println!("\n  [Part 2C — Exact Kakutani Skyscraper Height Spectrum Across ALL 249 Real Blocks]:");
    for &(label, blocks, per_block) in corpora {
        println!("\n  --- {label} ({} blocks) ---", blocks.len());
        if per_block {
            println!(
                "  {:<13} {:>6} {:>5} {:>5} | {:>11} {:>11} {:>11} {:>11} | {:>5} {:>5} {:>5}",
                "Block",
                "m/n%",
                "E[t]",
                "max_t",
                "t=1 (r%/b%)",
                "2..15(r/b%)",
                "16+(r%/b%)",
                "64+(r%/b%)",
                "Gini",
                "H_r",
                "H_b"
            );
        }
        let mut agg_n = 0u64;
        let mut agg_m = 0u64;
        let mut agg_max_tau = 0u64;
        let mut agg_r1 = 0u64;
        let mut agg_r2_15 = 0u64;
        let mut agg_r16 = 0u64;
        let mut agg_r17p = 0u64;
        let mut agg_r64p = 0u64;
        let mut agg_b1 = 0u64;
        let mut agg_b2_15 = 0u64;
        let mut agg_b16 = 0u64;
        let mut agg_b17p = 0u64;
        let mut agg_b64p = 0u64;
        let mut sum_gini = 0.0f64;
        let mut sum_hr = 0.0f64;
        let mut sum_hb = 0.0f64;

        for b in blocks {
            // Verify direct FL Kakutani skyscraper conservation (and S2b parity on the 16 non-wiki/adm blocks)
            let (s, _, _, _, _) = analyze_kakutani_birkhoff_hopf(&b.l, b.p, LIMIT as usize, per_block);
            assert_eq!(s.cycles_total, 1, "Real block {} not single-cycle!", b.name);
            assert_eq!(s.cycles_orphan, 0);
            assert_eq!(s.sum_tau, s.n);

            agg_n += s.n;
            agg_m += s.m;
            agg_max_tau = agg_max_tau.max(s.max_tau);
            agg_r1 += s.rows_tau1;
            agg_r2_15 += s.rows_tau2_15;
            agg_r16 += s.rows_tau16;
            agg_r17p += s.rows_tau17p;
            agg_r64p += s.rows_tau64p;
            agg_b1 += s.bytes_tau1;
            agg_b2_15 += s.bytes_tau2_15;
            agg_b16 += s.bytes_tau16;
            agg_b17p += s.bytes_tau17p;
            agg_b64p += s.bytes_tau64p;
            sum_gini += s.gini;
            sum_hr += s.h_row_bits;
            sum_hb += s.h_byte_bits;

            if per_block {
                let r16p = s.rows_tau16 + s.rows_tau17p;
                let b16p = s.bytes_tau16 + s.bytes_tau17p;
                println!(
                    "  {:<13} {:>5.1}% {:>5.2} {:>5} | {:>4.1}/{:<5.1}% {:>4.1}/{:<5.1}% {:>4.1}/{:<5.1}% {:>4.2}/{:<5.1}% | {:>5.3} {:>5.2} {:>5.2}",
                    b.name,
                    pct(s.m, s.n),
                    s.n as f64 / s.m as f64,
                    s.max_tau,
                    pct(s.rows_tau1, s.m),
                    pct(s.bytes_tau1, s.n),
                    pct(s.rows_tau2_15, s.m),
                    pct(s.bytes_tau2_15, s.n),
                    pct(r16p, s.m),
                    pct(b16p, s.n),
                    pct(s.rows_tau64p, s.m),
                    pct(s.bytes_tau64p, s.n),
                    s.gini,
                    s.h_row_bits,
                    s.h_byte_bits
                );
            }
        }
        let nb = blocks.len() as f64;
        let agg_r16p = agg_r16 + agg_r17p;
        let agg_b16p = agg_b16 + agg_b17p;
        println!(
            "  [AGG {label}]: n={agg_n}, m={agg_m} ({:.2}%), E[tau]=n/m={:.3}, max_tau={agg_max_tau} | 100% single-cycle, 0 orphans!",
            pct(agg_m, agg_n),
            agg_n as f64 / agg_m as f64
        );
        println!(
            "    Strata (row% of m / byte% of n): tau=1: {:.2}% / {:.2}% | 2<=tau<=15: {:.2}% / {:.2}% | tau>=16: {:.2}% / {:.2}% (tau=16: {:.2}%/{:.2}%, tau>=17 spelling: {:.2}%/{:.2}%) | tau>=64: {:.3}% / {:.2}%",
            pct(agg_r1, agg_m),
            pct(agg_b1, agg_n),
            pct(agg_r2_15, agg_m),
            pct(agg_b2_15, agg_n),
            pct(agg_r16p, agg_m),
            pct(agg_b16p, agg_n),
            pct(agg_r16, agg_m),
            pct(agg_b16, agg_n),
            pct(agg_r17p, agg_m),
            pct(agg_b17p, agg_n),
            pct(agg_r64p, agg_m),
            pct(agg_b64p, agg_n),
        );
        println!(
            "    Concentration: mean Gini = {:.4}, mean H_row(tau) = {:.3} bits, mean H_byte(tau) = {:.3} bits",
            sum_gini / nb,
            sum_hr / nb,
            sum_hb / nb
        );
    }
    println!();
}

// ============================================================================
// 3. PART 3 — 2-Stage Kakutani Induction (R_2 ⊂ R_1 Skyscraper Composition):
//    Exact Mathematical Verification + Open<16> 1T/2W/4W Stepanov Construction Test
// ============================================================================

/// Verify Kakutani (1943) 2-Stage Composition Theorem:
///   (T_{R_1})_{R_2} == T_{R_2}  AND  tau_{R_2}(x) == sum_{j=0}^{tau_{R_2}^{(R_1)}(x) - 1} tau_{R_1}(T_{R_1}^j(x))
/// directly against 1-stage FL induction onto R_2 \subset [0, n) across ALL three subsets R_2!
fn verify_2stage_kakutani_identity(l: &[u8], p: u32, mode_name: &str, mode: u8) -> (usize, usize, usize, u64) {
    let n = l.len();
    let (s1, fl, ret_rows, next_r1, mark_idx) =
        analyze_kakutani_birkhoff_hopf(l, p, LIMIT as usize, false);
    let m1 = ret_rows.len();

    // Compute exact tau_R1[u] for every u in 0..m1 (by walking FL from ret_rows[u] to next_r1[u]!)
    let mut in_r1 = vec![false; n];
    for &row in &ret_rows {
        in_r1[row as usize] = true;
    }
    let mut tau_r1 = vec![0u32; m1];
    for (u, &row) in ret_rows.iter().enumerate() {
        let mut x = fl[row as usize] as usize;
        let mut steps = 1u32;
        while !in_r1[x] {
            x = fl[x] as usize;
            steps += 1;
        }
        tau_r1[u] = steps;
    }

    // Choose R_2 \subset R_1 (always containing mark_idx!):
    //   mode 1 (K2-TallSkip): R_2 = {mark} U {u in R_1 : tau_R1(u) == 1}
    //   mode 2 (K2-Indep2):   E_2 = {v != mark : (v & 1 == 1) && (next_r1[v] & 1 == 0 || next_r1[v] == mark)}, R_2 = R_1 \ E_2
    //   mode 3 (K2-Parity50): R_2 = {mark} U {u in R_1 : (u & 1) == 0}
    let mut in_r2_idx = vec![false; m1];
    match mode {
        1 => {
            for u in 0..m1 {
                if u as u32 == mark_idx || tau_r1[u] == 1 {
                    in_r2_idx[u] = true;
                }
            }
        }
        2 => {
            for u in 0..m1 {
                let is_skip = (u as u32 != mark_idx)
                    && ((u & 1) == 1)
                    && (((next_r1[u] & 1) == 0) || next_r1[u] == mark_idx);
                in_r2_idx[u] = !is_skip;
            }
        }
        3 => {
            for u in 0..m1 {
                if u as u32 == mark_idx || (u & 1) == 0 {
                    in_r2_idx[u] = true;
                }
            }
        }
        _ => unreachable!(),
    }
    assert!(in_r2_idx[mark_idx as usize]);

    // 1. Compute 2-stage induced (T_{R_1})_{R_2} and additive height tau_2stage(x)
    let mut in_r2_full = vec![false; n];
    let mut r2_rank_from_r1 = vec![u32::MAX; m1];
    let mut r2_rows_full = Vec::new();
    let mut r2_r1_indices = Vec::new();
    for u in 0..m1 {
        if in_r2_idx[u] {
            r2_rank_from_r1[u] = r2_rows_full.len() as u32;
            r2_rows_full.push(ret_rows[u]);
            r2_r1_indices.push(u);
            in_r2_full[ret_rows[u] as usize] = true;
        }
    }
    let m2 = r2_rows_full.len();
    let mut max_chain = 0u64;
    let mut sum_tau2 = 0u64;

    for (i2, &u1) in r2_r1_indices.iter().enumerate() {
        // Stage-2 induction inside R_1: follow next_r1 until hitting in_r2_idx
        let mut cur1 = next_r1[u1] as usize;
        let mut acc_tau = u64::from(tau_r1[u1]);
        let mut chain_len = 1u64;
        while !in_r2_idx[cur1] {
            acc_tau += u64::from(tau_r1[cur1]);
            cur1 = next_r1[cur1] as usize;
            chain_len += 1;
        }
        let target_2stage = r2_rank_from_r1[cur1];
        if mode == 2 {
            assert!(chain_len <= 2, "K2-Indep2 must have chain_len <= 2");
        }
        max_chain = max_chain.max(chain_len);
        sum_tau2 += acc_tau;

        // Direct 1-stage induction of FL onto R_2 \subset [0, n)
        let row0 = r2_rows_full[i2] as usize;
        let mut xf = fl[row0] as usize;
        let mut direct_tau = 1u64;
        while !in_r2_full[xf] {
            xf = fl[xf] as usize;
            direct_tau += 1;
        }
        let target_direct = r2_rows_full[target_2stage as usize] as usize;
        assert_eq!(
            xf, target_direct,
            "Kakutani (T_{{R_1}})_{{R_2}} != T_{{R_2}} on mode {mode_name}!"
        );
        assert_eq!(
            acc_tau, direct_tau,
            "Kakutani additive height law violated on mode {mode_name}!"
        );
    }

    // On single-cycle valid BWTs, sum_{x in R_2} tau_{R_2}(x) == n identically!
    if s1.cycles_total == 1 {
        assert_eq!(sum_tau2, n as u64);
    }
    (n, m1, m2, max_chain)
}

// ============================================================================
// 4. Full Open<16> Kernel + 3 Genuine 2-Stage Kakutani Induction Arms
// ============================================================================

#[derive(Clone, Copy)]
struct Band(u64);

impl Band {
    #[inline(always)]
    fn new(end: u32, dst: u32, code: u32) -> Self {
        Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
    }
    #[inline(always)]
    fn end(self) -> u32 {
        (self.0 & 0xF_FFFF) as u32
    }
    #[inline(always)]
    fn dst(self) -> u32 {
        ((self.0 >> 20) & 0xF_FFFF) as u32
    }
    #[inline(always)]
    fn code(self) -> u32 {
        (self.0 >> 40) as u32
    }
}

#[derive(Clone, Copy)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    mark_byte: u8,
}

impl Default for First {
    fn default() -> Self {
        Self {
            thick_rows: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            mark_byte: 0,
        }
    }
}

fn starts_arr(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

#[inline(always)]
fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
    let mut slot = [0; INLINE];
    slot[..4].copy_from_slice(&start.to_le_bytes());
    slot[4..8].copy_from_slice(&end.to_le_bytes());
    slot
}

#[inline(always)]
fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
    let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
    let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
    &spelling[start..end]
}

fn begin_inline(words: usize, inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
    if inline16.len() < 256 {
        inline16.clear();
        inline16.extend((0..=255u8).map(|c| {
            let mut v = [0; INLINE];
            v[0] = c;
            v
        }));
        lens.clear();
        lens.resize(256, 1);
    }
    inline16.truncate(256);
    lens.truncate(256);
    if words > 0 && inline16.capacity() < 256 + words {
        inline16.reserve_exact(256 + words - inline16.len());
        lens.reserve_exact(256 + words - lens.len());
    }
}

#[cold]
#[inline(never)]
fn grow_u32(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

fn spelling_hash_long(bytes: &[u8]) -> u64 {
    let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ part(bytes.len() - 8)
            .rotate_left(29)
            .wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

trait Links {
    const STRIDE: usize;
    const IS_NARROW: bool;
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32);
}

struct Narrow;
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
    const IS_NARROW: bool = true;
    const BOTTOM: usize = 1;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] & 0xFFFF
    }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] >> 16
    }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF_0000) | to;
    }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 4..id as usize * 4 + 4];
        (sl[0] >> 16, sl[1], sl[2], sl[3])
    }
}

impl Links for Wide {
    const STRIDE: usize = 6;
    const IS_NARROW: bool = false;
    const BOTTOM: usize = 2;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side]
    }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side + 1]
    }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side] = to;
    }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 6..id as usize * 6 + 6];
        (sl[1], sl[2], sl[4], sl[5])
    }
}

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links> Elim<'_, L> {
    #[inline(always)]
    fn width(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2]
    }
    #[inline(always)]
    fn set_width(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w;
    }
    #[inline(always)]
    fn word(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1]
    }
    #[inline(always)]
    fn set_word(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w;
    }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, x: u32) {
        self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
    }

    fn assign_bottom(&mut self, rows: u32) {
        let mut first = L::next(self.a, 0, 1);
        let mut last = L::prev(self.a, 0, 1);
        let (mut lo, mut hi) = (0, rows);
        while first != last {
            let next = L::next(self.a, first, 1);
            let prev = L::prev(self.a, last, 1);
            let width = self.width(first);
            hi -= self.width(last);
            self.set_bottom(first, lo);
            self.set_bottom(last, hi);
            lo += width;
            if next == last {
                assert_eq!(lo, hi);
                return;
            }
            first = next;
            last = prev;
        }
        if first != 0 {
            self.set_bottom(first, lo);
            lo += self.width(first);
        }
        assert_eq!(lo, hi);
    }

    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes;
        self.nodes += 1;
        self.set_width(id, width);
        self.set_word(id, word);
        id
    }

    #[inline(always)]
    fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
        let (head, tail) = list;
        if *tail == 0 {
            *head = id;
        } else {
            L::set_next(self.a, *tail, side, id);
        }
        L::set_prev(self.a, id, side, *tail);
        *tail = id;
    }

    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, second] {
            if tail == 0 {
                continue;
            }
            L::set_next(self.a, prev, side, head);
            L::set_prev(self.a, head, side, prev);
            prev = tail;
        }
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }

    #[inline(always)]
    fn cut(&mut self, id: u32, side: usize) {
        let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
        L::set_next(self.a, p, side, n);
        L::set_prev(self.a, n, side, p);
    }

    #[inline(always)]
    fn pop_tail(&mut self, id: u32, side: usize) {
        let prev = L::prev(self.a, id, side);
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }

    #[inline(always)]
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }

    #[inline(always)]
    fn rule(&self, id: u32) -> usize {
        self.base + id as usize * RULE
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.a[self.rule(h & ID_MASK) + 2]
        }
    }

    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 3 {
            return u128::from(id);
        }
        let r = self.rule(id);
        let sl = &self.a[r..r + 4];
        u128::from(sl[0])
            | (u128::from(sl[1]) << 32)
            | (u128::from(sl[2]) << 64)
            | (u128::from(sl[3] & 0x00ff_ffff) << 96)
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let left_len = left >> ID_BITS;
        let right_len = right >> ID_BITS;
        let sum = left_len + right_len;
        if sum < INLINE as u32 {
            if sum <= 3 {
                let lp = left & ID_MASK;
                let rp = right & ID_MASK;
                return (sum << ID_BITS) | lp | (rp << (8 * left_len));
            }
            let payload =
                self.short_payload_h(left) | (self.short_payload_h(right) << (8 * left_len));
            let id = ((self.top - self.base) / RULE) as u32;
            let bytes = payload | (u128::from(sum) << 120);
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = bytes as u32;
            out[1] = (bytes >> 32) as u32;
            out[2] = (bytes >> 64) as u32;
            out[3] = (bytes >> 96) as u32;
            self.top += RULE;
            (sum << ID_BITS) | id
        } else {
            let len = if sum <= SATURATED {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            let id = ((self.top - self.base) / RULE) as u32;
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.top += RULE;
            (len.min(SATURATED) << ID_BITS) | id
        }
    }

    fn power(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join(acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = self.join(base, base);
        }
    }

    #[inline(never)]
    fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let sent = &self.a[..2];
            let x = sent[0] & 0xFFFF;
            let y = sent[1] & 0xFFFF;
            let xi = x as usize * 4;
            let yi = y as usize * 4;
            let nx = &self.a[xi..xi + 4];
            let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                self.a[px0 as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px0;
                self.a[px1 as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | px1;
                continue;
            }
            let ny = &self.a[yi..yi + 4];
            let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            let word = self.join(wx, wy);
            if aw <= bw {
                let px = x0 & 0xFFFF;
                self.a[px as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px;
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y | (y_next << 16);
                    nxm[3] = word;
                    let nym = &mut self.a[yi..yi + 4];
                    nym[0] = (y0 & 0xFFFF) | (x << 16);
                    nym[2] = bw - aw;
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                } else {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y_prev | (y_next << 16);
                    nxm[3] = word;
                    let ypi = y_prev as usize * 4;
                    self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    let py = y1 & 0xFFFF;
                    self.a[py as usize * 4 + 1] &= 0xFFFF;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                }
            } else {
                let py = y1 & 0xFFFF;
                self.a[py as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                let x_next = if x == py { 0 } else { x1 >> 16 };
                let nym = &mut self.a[yi..yi + 4];
                nym[1] = x | (x_next << 16);
                nym[3] = word;
                let nxm = &mut self.a[xi..xi + 4];
                nxm[1] = (x1 & 0xFFFF) | (y << 16);
                nxm[2] = aw - bw;
                let xni = x_next as usize * 4 + 1;
                self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
            }
        };
        *remaining = left;
        done
    }

    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
        if L::IS_NARROW && !SPIRALS {
            return self.eliminate_narrow_plain(remaining);
        }
        let mut left = *remaining;
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if SPIRALS && x != y {
                if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            if x == y {
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.pop_tail(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.pop_tail(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        *remaining = left;
        done
    }

    #[inline(always)]
    fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        let mut h = handle;
        loop {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                dest.extend_from_slice(&bytes[..g as usize]);
                match stack.pop() {
                    Some(next) => {
                        h = next;
                        continue;
                    }
                    None => return,
                }
            }
            let r = self.rule(h & ID_MASK);
            let tag = self.a[r + 3];
            if tag != u32::MAX {
                let start = (tag & 0x7fff_ffff) as usize;
                let rlen = self.a[r + 2] as usize;
                dest.extend_from_within(start..start + rlen);
                match stack.pop() {
                    Some(next) => {
                        h = next;
                        continue;
                    }
                    None => return,
                }
            }
            self.a[r + 3] = 0x8000_0000 | (dest.len() as u32);
            let left = self.a[r];
            let right = self.a[r + 1];
            let lg = left >> ID_BITS;
            if lg < INLINE as u32 {
                let lb = self.short_payload_h(left).to_le_bytes();
                dest.extend_from_slice(&lb[..lg as usize]);
                let rg = right >> ID_BITS;
                if rg < INLINE as u32 {
                    let rb = self.short_payload_h(right).to_le_bytes();
                    dest.extend_from_slice(&rb[..rg as usize]);
                    match stack.pop() {
                        Some(next) => {
                            h = next;
                            continue;
                        }
                        None => return,
                    }
                }
                h = right;
            } else {
                stack.push(right);
                h = left;
            }
        }
    }

    fn inline_word(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
            (v.to_le_bytes(), (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append(word, stack, spelling);
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }
}

struct Dictionary<'a> {
    pairs: &'a mut [u32; 512],
    contents: &'a mut [u32; 2048],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl Dictionary<'_> {
    #[inline(always)]
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        let bucket = ((handle.wrapping_mul(0x9e37_79b9)
            ^ (u32::from(byte).wrapping_mul(0x85eb_ca6b)))
            >> 24) as usize;
        let slot = bucket * 2;
        let val = self.pairs[slot + 1];
        if self.pairs[slot] == handle && (val >> 24) == u32::from(byte) {
            return val & 0x00FF_FFFF;
        }
        let code = self.code_miss(e, handle, handle & ID_MASK, byte);
        self.pairs[slot] = handle;
        self.pairs[slot + 1] = (u32::from(byte) << 24) | code;
        code
    }

    fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
            let entry = v.to_le_bytes();
            let len = (height + 1) as u8;
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let h = h ^ (h >> 30);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == len && self.inline16[old] == entry
            } {
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry, len);
            self.contents[index] = tag | c;
            return c;
        }
        let start = self.spelling.len();
        let memoized = e.a[e.rule(id) + 3] != u32::MAX;
        e.append(handle, self.stack, self.spelling);
        self.spelling.push(byte);
        let entry = long_slot(start as u32, self.spelling.len() as u32);
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = (h as usize) & 2047;
        let ent = self.contents[index];
        if (ent & 0xFF00_0000) == tag && {
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes
        } {
            if memoized {
                self.spelling.truncate(start);
            } else {
                self.spelling.pop();
            }
            return ent & 0x00FF_FFFF;
        }
        let c = self.insert(entry, LONG);
        self.contents[index] = tag | c;
        c
    }

    #[inline(always)]
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}

#[inline(always)]
fn split_run(
    c: u8,
    pos: u32,
    width: u32,
    p: u32,
    limit: u32,
    mut visit: impl FnMut(u8, u32, u32, bool),
) {
    let end = pos + width;
    if width > limit && (pos..end).contains(&p) {
        if pos < p {
            visit(c, pos, p - pos, false);
        }
        visit(c, p, 1, true);
        if p + 1 < end {
            visit(c, p + 1, end - p - 1, false);
        }
    } else {
        visit(c, pos, width, width <= limit);
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct StageTimes {
    s1_us: f64,
    s2a_us: f64,
    s2b_us: f64,
    s3a_us: f64,
    s4_us: f64,
    s4b_us: f64, // Stage-2 Kakutani induction cost!
    s5_us: f64,
    m1: u64,
    m2: u64,
}

impl StageTimes {
    fn total_us(&self) -> f64 {
        self.s1_us + self.s2a_us + self.s2b_us + self.s3a_us + self.s4_us + self.s4b_us + self.s5_us
    }
}

/// Open<16> kernel supporting 4 modes:
///   MODE = 0: Base-O16 (Open<16> F-Inline3 + K-TailUnw + Bidirectional XOR S5)
///   MODE = 1: K2-TallSkip (2-Stage Kakutani Induction R_2 = {mark} U {u in R_1 : tau_1(u) == 1})
///   MODE = 2: K2-Indep2   (2-Stage Kakutani Induction on independent set E_2, tau^{(R_1)} in {1, 2})
///   MODE = 3: K2-Parity50 (2-Stage Kakutani Induction R_2 = {mark} U {u in R_1 : (u & 1) == 0})
#[derive(Default)]
struct KakutaniKernel {
    spill: outq::Spill,
    edges: Vec<u32>,
    extra: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    // Stage-2 Kakutani buffers (reused across blocks; strictly accounted in ws_bytes!)
    edges2: Vec<u32>,
    extra2: Vec<u32>,
    r2_map: Vec<u32>,
    s2_dedup: Vec<u32>,
    tmp_word: Vec<u8>,
}

impl KakutaniKernel {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }
    fn wide(ns: usize) -> bool {
        Self::nodes_most(ns) > NARROW_NODES
    }
    fn arena_words(ns: usize, m: usize) -> usize {
        let stride = if Self::wide(ns) { Wide::STRIDE } else { Narrow::STRIDE };
        (4 * ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
    }
    fn ws_bytes(&self) -> usize {
        self.edges.capacity() * 4
            + self.extra.capacity() * 4
            + self.bands.capacity() * 8
            + self.spelling.capacity()
            + self.inline16.capacity() * INLINE
            + self.lens.capacity()
            + self.stack.capacity() * 4
            + self.edges2.capacity() * 4
            + self.extra2.capacity() * 4
            + self.r2_map.capacity() * 4
            + self.s2_dedup.capacity() * 4
            + self.tmp_word.capacity()
    }

    fn run<const MODE: u8, const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> StageTimes {
        let mut tm = StageTimes::default();
        out.resize(n + INLINE, 0);
        if n == 0 {
            out.truncate(0);
            return tm;
        }
        let t_s1 = if TIMED { Some(Instant::now()) } else { None };
        let mut f = First::default();
        let mut ns = 0usize;
        let heads = {
            let mut queues = outq::Queues::new(out, &mut self.spill, u32::MAX);
            let mut pos = 0u32;
            let mut i = 0usize;
            while i < records.len() {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                while i < records.len() && (records[i] >> 8) as u8 == c {
                    width += u32::from(records[i] & 255);
                    i += 1;
                }
                if width == 0 {
                    continue;
                }
                if width <= LIMIT {
                    let a = usize::from(c);
                    if p.wrapping_sub(pos) < width {
                        f.mark = f.m + (p - pos);
                        f.mark_byte = c;
                    }
                    queues.push_thin(a, f.m, width);
                    f.m += width;
                } else if p.wrapping_sub(pos) >= width {
                    let a = usize::from(c);
                    let at = 3 * ns;
                    if self.edges.len() < at + 3 {
                        let need = (at + 3).max(2 * self.edges.len()).max(1024);
                        self.edges.reserve_exact(need - self.edges.len());
                        self.edges.resize(need, 0);
                    }
                    let thin_a = queues.thin_of(a);
                    self.edges[at..at + 3].copy_from_slice(&[
                        pos,
                        thin_a + f.thick_rows[a],
                        (width << 8) | u32::from(c),
                    ]);
                    ns += 1;
                    f.thick[a] += 1;
                    f.thick_rows[a] += width;
                } else {
                    split_run(c, pos, width, p, LIMIT, |ch, x, w, keep| {
                        let a = usize::from(ch);
                        if keep {
                            if p.wrapping_sub(x) < w {
                                f.mark = f.m + (p - x);
                                f.mark_byte = ch;
                            }
                            queues.push_thin(a, f.m, w);
                            f.m += w;
                        } else {
                            let at = 3 * ns;
                            if self.edges.len() < at + 3 {
                                let need = (at + 3).max(2 * self.edges.len()).max(1024);
                                self.edges.reserve_exact(need - self.edges.len());
                                self.edges.resize(need, 0);
                            }
                            let thin_a = queues.thin_of(a);
                            self.edges[at..at + 3].copy_from_slice(&[
                                x,
                                thin_a + f.thick_rows[a],
                                (w << 8) | u32::from(ch),
                            ]);
                            ns += 1;
                            f.thick[a] += 1;
                            f.thick_rows[a] += w;
                        }
                    });
                }
                pos += width;
            }
            assert_eq!(pos as usize, n);
            f.thin = queues.extract_thin();
            queues.heads()
        };
        if let Some(t) = t_s1 {
            tm.s1_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let m = f.m as usize;
        tm.m1 = m as u64;
        let mut counts = [0u32; 256];
        for a in 0..256 {
            counts[a] = f.thin[a] + f.thick_rows[a];
        }
        let first = starts_arr(counts);
        let mut write = starts_arr(f.thick);

        let need = Self::arena_words(ns, m);
        if self.edges.len() < need {
            self.edges.reserve_exact(need - self.edges.len());
            self.edges.resize(need, 0);
        }
        self.bands.clear();
        self.spelling.clear();
        if n >= 65_536 {
            if self.bands.capacity() < 8192 {
                self.bands.reserve_exact(8192 - self.bands.len());
            }
            if self.spelling.capacity() < 49_152 {
                self.spelling.reserve_exact(49_152 - self.spelling.len());
            }
        }

        let cursors = if Self::wide(ns) {
            self.induced::<Wide, TIMED>(ns, n as u32, &f.thin, &first, &mut write, m as u32, &mut tm)
        } else {
            self.induced::<Narrow, TIMED>(ns, n as u32, &f.thin, &first, &mut write, m as u32, &mut tm)
        };

        // S4: Wiring Stage-1 R_1 edges
        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m {
            self.edges.resize(m, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m, 0);
        }
        let mark = f.mark;
        let mb = usize::from(f.mark_byte);
        let use_xor_s1 = MODE == 0 && !wide && m >= 65_536;
        let mut predecessor = mark;
        if use_xor_s1 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, &heads, a);
                if a != mb {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                        );
                    }
                } else {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                            mark,
                            &mut predecessor,
                        );
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, &heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                }
            }
        }
        if let Some(t) = t_s4 {
            tm.s4_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let mask1 = (1u32 << bits) - 1;

        if MODE == 0 {
            tm.m2 = m as u64;
            let t_s5 = if TIMED { Some(Instant::now()) } else { None };
            self.emit_s5(
                m,
                n,
                bits,
                wide,
                use_xor_s1,
                mark,
                predecessor,
                true,
                out,
            );
            if let Some(t) = t_s5 {
                tm.s5_us = t.elapsed().as_secs_f64() * 1e6;
            }
            return tm;
        }

        // S4b: Genuine 2-Stage Kakutani Induction (R_2 \subset R_1, (T_{R_1})_{R_2} = T_{R_2})!
        let t_s4b = if TIMED { Some(Instant::now()) } else { None };
        if self.r2_map.len() < m {
            self.r2_map.resize(m, u32::MAX);
        }
        let r2_map = &mut self.r2_map[..m];
        let mut m2 = 0usize;
        match MODE {
            1 => {
                // K2-TallSkip: R_2 = {mark} U {u in R_1 : tau_1(u) == 1} (i.e. code < 256!)
                for u in 0..m {
                    let code = if wide {
                        self.extra[u]
                    } else {
                        self.edges[u] >> bits
                    };
                    if u as u32 == mark || self.lens[code as usize] == 1 {
                        r2_map[u] = m2 as u32;
                        m2 += 1;
                    } else {
                        r2_map[u] = u32::MAX;
                    }
                }
            }
            2 => {
                // K2-Indep2: E_2 = {v != mark : (v & 1 == 1) && (next(v) & 1 == 0 || next(v) == mark)}
                for u in 0..m {
                    let nxt = self.edges[u] & mask1;
                    let skip = (u as u32 != mark)
                        && ((u & 1) == 1)
                        && (((nxt & 1) == 0) || nxt == mark);
                    if !skip {
                        r2_map[u] = m2 as u32;
                        m2 += 1;
                    } else {
                        r2_map[u] = u32::MAX;
                    }
                }
            }
            3 => {
                // K2-Parity50: R_2 = {mark} U {u in R_1 : (u & 1) == 0}
                for u in 0..m {
                    if u as u32 == mark || (u & 1) == 0 {
                        r2_map[u] = m2 as u32;
                        m2 += 1;
                    } else {
                        r2_map[u] = u32::MAX;
                    }
                }
            }
            _ => unreachable!(),
        }
        let mark2 = r2_map[mark as usize];
        assert!(mark2 < m2 as u32);
        tm.m2 = m2 as u64;

        let bits2 = (u32::BITS - (m2 as u32 - 1).leading_zeros()).max(1);
        let mask2 = (1u32 << bits2) - 1;
        if self.edges2.len() < m2 {
            self.edges2.resize(m2, 0);
        }
        if self.extra2.len() < m2 {
            self.extra2.resize(m2, 0);
        }
        let mut max_code2 = self.lens.len() - 1;
        // 4096-slot direct-mapped Stage-2 word deduplication table (reused across blocks; zero orphan spelling bytes!)
        const S2_DEDUP_SLOTS: usize = 4096;
        if self.s2_dedup.len() < S2_DEDUP_SLOTS {
            self.s2_dedup.resize(S2_DEDUP_SLOTS, 0);
        }
        self.s2_dedup[..S2_DEDUP_SLOTS].fill(0);

        let dedup_inline = |entry: [u8; INLINE],
                            len: u8,
                            dedup: &mut [u32],
                            inline16: &mut Vec<[u8; INLINE]>,
                            lens: &mut Vec<u8>|
         -> u32 {
            let v = u128::from_le_bytes(entry);
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let h = h ^ (h >> 30);
            let idx = (h as usize) & (S2_DEDUP_SLOTS - 1);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let ent = dedup[idx];
            if (ent & 0xFF00_0000) == tag {
                let old = (ent & 0x00FF_FFFF) as usize;
                if old < lens.len() && lens[old] == len && inline16[old] == entry {
                    return old as u32;
                }
            }
            let cid = lens.len() as u32;
            inline16.push(entry);
            lens.push(len);
            if cid <= 0x00FF_FFFF {
                dedup[idx] = tag | cid;
            }
            cid
        };

        let dedup_long = |sp_start: usize,
                          dedup: &mut [u32],
                          inline16: &mut Vec<[u8; INLINE]>,
                          lens: &mut Vec<u8>,
                          spelling: &mut Vec<u8>|
         -> u32 {
            let bytes = &spelling[sp_start..];
            let h = spelling_hash_long(bytes);
            let idx = (h as usize) & (S2_DEDUP_SLOTS - 1);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let ent = dedup[idx];
            if (ent & 0xFF00_0000) == tag {
                let old = (ent & 0x00FF_FFFF) as usize;
                if old < lens.len() && lens[old] == LONG && long_word(&inline16[old], spelling) == bytes {
                    spelling.truncate(sp_start);
                    return old as u32;
                }
            }
            let cid = lens.len() as u32;
            inline16.push(long_slot(sp_start as u32, spelling.len() as u32));
            lens.push(LONG);
            if cid <= 0x00FF_FFFF {
                dedup[idx] = tag | cid;
            }
            cid
        };

        // Induce T_{R_2} and compose Kakutani words W_2(u) = W_1(u) . W_1(T_{R_1}(u)) ...
        for u in 0..m {
            let u2 = self.r2_map[u];
            if u2 == u32::MAX {
                continue;
            }
            let w0 = self.edges[u];
            let c0 = if wide { self.extra[u] } else { w0 >> bits };
            let mut cur = (w0 & mask1) as usize;
            let nxt_mapped = self.r2_map[cur];
            if nxt_mapped != u32::MAX {
                // tau_{R_2}^{(R_1)}(u) == 1: unchanged word & single hop!
                self.edges2[u2 as usize] = nxt_mapped;
                self.extra2[u2 as usize] = c0;
                continue;
            }
            // tau_{R_2}^{(R_1)}(u) >= 2: compose Stage-1 words along the skyscraper jump!
            let l0 = self.lens[c0 as usize];
            if MODE == 2 {
                // Fast path for K2-Indep2: chain length is GUARANTEED == 2!
                let w1 = self.edges[cur];
                let c1 = if wide { self.extra[cur] } else { w1 >> bits };
                let dst2 = self.r2_map[(w1 & mask1) as usize];
                debug_assert_ne!(dst2, u32::MAX);
                let l1 = self.lens[c1 as usize];
                let new_code = if l0 != LONG && l1 != LONG && usize::from(l0) + usize::from(l1) <= INLINE {
                    let mut entry = self.inline16[c0 as usize];
                    let s0 = usize::from(l0);
                    let s1 = usize::from(l1);
                    entry[s0..s0 + s1].copy_from_slice(&self.inline16[c1 as usize][..s1]);
                    dedup_inline(
                        entry,
                        (s0 + s1) as u8,
                        &mut self.s2_dedup,
                        &mut self.inline16,
                        &mut self.lens,
                    )
                } else {
                    let sp_start = self.spelling.len();
                    if l0 != LONG {
                        self.spelling
                            .extend_from_slice(&self.inline16[c0 as usize][..usize::from(l0)]);
                    } else {
                        let st = u32::from_le_bytes(self.inline16[c0 as usize][..4].try_into().unwrap()) as usize;
                        let en = u32::from_le_bytes(self.inline16[c0 as usize][4..8].try_into().unwrap()) as usize;
                        self.spelling.extend_from_within(st..en);
                    }
                    if l1 != LONG {
                        self.spelling
                            .extend_from_slice(&self.inline16[c1 as usize][..usize::from(l1)]);
                    } else {
                        let st = u32::from_le_bytes(self.inline16[c1 as usize][..4].try_into().unwrap()) as usize;
                        let en = u32::from_le_bytes(self.inline16[c1 as usize][4..8].try_into().unwrap()) as usize;
                        self.spelling.extend_from_within(st..en);
                    }
                    dedup_long(
                        sp_start,
                        &mut self.s2_dedup,
                        &mut self.inline16,
                        &mut self.lens,
                        &mut self.spelling,
                    )
                };
                max_code2 = max_code2.max(new_code as usize);
                self.edges2[u2 as usize] = dst2;
                self.extra2[u2 as usize] = new_code;
            } else {
                // General arbitrary chain length tau_{R_2}^{(R_1)}(u) >= 2 (K2-TallSkip / K2-Parity50)
                self.tmp_word.clear();
                if l0 != LONG {
                    self.tmp_word
                        .extend_from_slice(&self.inline16[c0 as usize][..usize::from(l0)]);
                } else {
                    let sl = long_word(&self.inline16[c0 as usize], &self.spelling);
                    self.tmp_word.extend_from_slice(sl);
                }
                while self.r2_map[cur] == u32::MAX {
                    let wc = self.edges[cur];
                    let cc = if wide { self.extra[cur] } else { wc >> bits };
                    let lc = self.lens[cc as usize];
                    if lc != LONG {
                        self.tmp_word
                            .extend_from_slice(&self.inline16[cc as usize][..usize::from(lc)]);
                    } else {
                        let sl = long_word(&self.inline16[cc as usize], &self.spelling);
                        self.tmp_word.extend_from_slice(sl);
                    }
                    cur = (wc & mask1) as usize;
                }
                let dst2 = self.r2_map[cur];
                let cid = if self.tmp_word.len() <= INLINE {
                    let mut entry = [0u8; INLINE];
                    entry[..self.tmp_word.len()].copy_from_slice(&self.tmp_word);
                    dedup_inline(
                        entry,
                        self.tmp_word.len() as u8,
                        &mut self.s2_dedup,
                        &mut self.inline16,
                        &mut self.lens,
                    )
                } else {
                    let sp_start = self.spelling.len();
                    self.spelling.extend_from_slice(&self.tmp_word);
                    dedup_long(
                        sp_start,
                        &mut self.s2_dedup,
                        &mut self.inline16,
                        &mut self.lens,
                        &mut self.spelling,
                    )
                };
                max_code2 = max_code2.max(cid as usize);
                self.edges2[u2 as usize] = dst2;
                self.extra2[u2 as usize] = cid;
            }
        }

        // Pack Stage-2 edges (with bidirectional XOR when narrow & m2 >= 65_536!)
        let wide2 = max_code2 > (u32::MAX >> bits2) as usize;
        let use_xor_s2 = !wide2 && m2 >= 65_536;
        let mut pred2 = mark2;
        if use_xor_s2 {
            // Convert (edges2[u] = next_u, extra2[u] = code_u) into packed XOR edges in `edges[..m2]`
            let xor_e = &mut self.edges[..m2];
            xor_e.fill(0);
            for u in 0..m2 {
                let nxt = self.edges2[u] & mask2;
                let code = self.extra2[u];
                xor_e[u] ^= nxt | (code << bits2);
                xor_e[nxt as usize] ^= u as u32;
                if nxt == mark2 {
                    pred2 = u as u32;
                }
            }
        } else if !wide2 {
            for u in 0..m2 {
                self.edges2[u] = (self.edges2[u] & mask2) | (self.extra2[u] << bits2);
            }
        }
        if let Some(t) = t_s4b {
            tm.s4b_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
        self.emit_s5(
            m2,
            n,
            bits2,
            wide2,
            use_xor_s2,
            mark2,
            pred2,
            use_xor_s2,
            out,
        );
        if let Some(t) = t_s5 {
            tm.s5_us = t.elapsed().as_secs_f64() * 1e6;
        }
        tm
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_s5(
        &self,
        m_cur: usize,
        n: usize,
        bits: u32,
        wide: bool,
        use_xor: bool,
        mark: u32,
        predecessor: u32,
        from_primary_edges: bool,
        out: &mut Vec<u8>,
    ) {
        let edges = if from_primary_edges {
            &self.edges[..m_cur]
        } else {
            &self.edges2[..m_cur]
        };
        let extra = if from_primary_edges {
            &self.extra
        } else {
            &self.extra2
        };
        let inline16 = &self.inline16;
        let lens = &self.lens;
        let spelling = &self.spelling;
        let mask = (1u32 << bits) - 1;

        if use_xor {
            let (mut prev, mut front) = (predecessor, mark);
            let (mut back, mut next) = (predecessor, mark);
            let (mut lo, mut hi) = (0usize, n);
            for pair in 0..=edges.len() / 2 {
                if lo == hi || (pair != 0 && front == next) {
                    break;
                }
                let ef = edges[front as usize];
                let cf = (ef >> bits) as usize;
                let single = front == back;
                let lf = lens[cf];
                if lf != LONG && hi - lo >= INLINE {
                    out[lo..lo + INLINE].copy_from_slice(&inline16[cf]);
                    lo += usize::from(lf);
                } else {
                    let word = if lf == LONG {
                        long_word(&inline16[cf], spelling)
                    } else {
                        &inline16[cf][..usize::from(lf)]
                    };
                    let take = word.len().min(hi - lo);
                    out[lo..lo + take].copy_from_slice(&word[..take]);
                    lo += take;
                }
                if single {
                    break;
                }
                let eb = edges[back as usize];
                let cb = (eb >> bits) as usize;
                let lb = lens[cb];
                if lb != LONG && hi - lo >= INLINE {
                    let shifted =
                        u128::from_le_bytes(inline16[cb]) << (8 * (INLINE - usize::from(lb)));
                    out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                    hi -= usize::from(lb);
                } else {
                    let word = if lb == LONG {
                        long_word(&inline16[cb], spelling)
                    } else {
                        &inline16[cb][..usize::from(lb)]
                    };
                    let take = word.len().min(hi - lo);
                    out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
                    hi -= take;
                }
                (prev, front) = (front, (ef & mask) ^ prev);
                (next, back) = (back, (eb & mask) ^ next);
            }
            if lo < hi {
                out.copy_within(hi..n, lo);
                let period = lo + (n - hi);
                let mut filled = period;
                while filled < n {
                    let take = filled.min(n - filled);
                    out.copy_within(0..take, filled);
                    filled += take;
                }
            }
        } else {
            let mut cur = mark;
            let mut pos_out = 0usize;
            while pos_out < n {
                let w = edges[cur as usize];
                let code = if wide {
                    extra[cur as usize] as usize
                } else {
                    (w >> bits) as usize
                };
                cur = w & mask;
                let len_b = lens[code];
                if len_b != LONG {
                    let l_sz = usize::from(len_b);
                    if n - pos_out >= INLINE {
                        out[pos_out..pos_out + INLINE].copy_from_slice(&inline16[code]);
                        pos_out += l_sz;
                    } else {
                        let take = l_sz.min(n - pos_out);
                        out[pos_out..pos_out + take].copy_from_slice(&inline16[code][..take]);
                        pos_out += take;
                    }
                } else {
                    let sl = long_word(&inline16[code], spelling);
                    let take = sl.len().min(n - pos_out);
                    out[pos_out..pos_out + take].copy_from_slice(&sl[..take]);
                    pos_out += take;
                }
            }
        }
        out.truncate(n);
    }

    #[allow(clippy::too_many_arguments)]
    fn induced<L: Links, const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        m: u32,
        tm: &mut StageTimes,
    ) -> [usize; 256] {
        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        let (hdr, arena) = self.edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        e.nodes = ns as u32 + 1;
        let mut widest = 0u32;
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0u32;
        for i in 0..ns {
            let sw = &mut strips_raw[3 * i..3 * i + 3];
            let from = sw[0];
            let wbyte = sw[2];
            let byte = (wbyte & 0xFF) as u8;
            let width = wbyte >> 8;
            let a = usize::from(byte);
            sw[1] += first[a];
            let dst_idx = write[a] as usize;
            image_order[dst_idx] = i as u32;
            write[a] = dst_idx as u32 + 1;
            let sid = i as u32 + 1;
            e.set_width(sid, width);
            e.set_word(sid, literal(byte));
            widest = widest.max(width);
            if x < from {
                let gw = from - x;
                widest = widest.max(gw);
                let gid = e.node(gw, 0);
                e.append_to(&mut order[0], gid, 0);
                e.append_to(&mut terminals[1], gid, 1);
            }
            e.append_to(&mut order[0], sid, 0);
            x = from + width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }
        x = 0;
        for &i in image_order.iter() {
            let idx = i as usize;
            let to = strips_raw[3 * idx + 1];
            let width = strips_raw[3 * idx + 2] >> 8;
            if x < to {
                widest = widest.max(to - x);
                let id = e.node(to - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i + 1, 1);
            x = to + width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[1], id, 1);
            e.append_to(&mut terminals[0], id, 0);
        }
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        if let Some(t) = t_s2a {
            tm.s2a_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let mut remaining = n;
        let spirals = widest > CERTIFIED_WIDTH;
        let grow_extra = if spirals { SPIRAL_RULES * RULE } else { RULE };
        while !if spirals {
            e.eliminate::<true>(&mut remaining)
        } else {
            e.eliminate::<false>(&mut remaining)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow_u32(&mut self.edges, 4 * ns + top + grow_extra);
            e = Elim {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        if let Some(t) = t_s2b {
            tm.s2b_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        e.assign_bottom(m);
        const PAIR_WORDS: usize = 512;
        const CACHE_WORDS: usize = PAIR_WORDS + 2048;
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS { header } else { tail };
        let mut e = Elim::<L> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, &mut self.inline16, &mut self.lens);
        let mut cursors = [0usize; 256];
        if cache.len() < CACHE_WORDS {
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0u32;
            for (a, &width) in thin.iter().enumerate() {
                cursors[a] = self.bands.len();
                let mut left = width;
                while left > 0 {
                    let (next, bottom, w, word) = L::band_node(e.a, id);
                    let take = left.min(w - in_node);
                    let code = if word == 0 {
                        a as u32
                    } else {
                        let (entry, len) = e.inline_word(word, a as u8, &mut self.stack, &mut self.spelling);
                        let c = self.lens.len() as u32;
                        self.inline16.push(entry);
                        self.lens.push(len);
                        c
                    };
                    self.bands.push(Band::new(take, bottom + in_node, code));
                    in_node += take;
                    left -= take;
                    if in_node == w {
                        id = next;
                        in_node = 0;
                    }
                }
            }
        } else {
            let (pairs_sl, rest) = cache.split_at_mut(PAIR_WORDS);
            let (contents_sl, _) = rest.split_at_mut(2048);
            pairs_sl.fill(0);
            contents_sl.fill(0);
            let pairs: &mut [u32; 512] = pairs_sl.try_into().unwrap();
            let contents: &mut [u32; 2048] = contents_sl.try_into().unwrap();
            let mut dict = Dictionary {
                pairs,
                contents,
                inline16: &mut self.inline16,
                lens: &mut self.lens,
                spelling: &mut self.spelling,
                stack: &mut self.stack,
            };
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0u32;
            for (a, &width) in thin.iter().enumerate() {
                cursors[a] = self.bands.len();
                let mut left = width;
                while left > 0 {
                    let (next, bottom, w, word) = L::band_node(e.a, id);
                    let take = left.min(w - in_node);
                    let code = if word == 0 {
                        a as u32
                    } else {
                        dict.code(&mut e, word, a as u8)
                    };
                    self.bands.push(Band::new(take, bottom + in_node, code));
                    in_node += take;
                    left -= take;
                    if in_node == w {
                        id = next;
                        in_node = 0;
                    }
                }
            }
        }
        if let Some(t) = t_s3a {
            tm.s3a_us = t.elapsed().as_secs_f64() * 1e6;
        }
        cursors
    }
}

fn run_corpus_1t<const MODE: u8>(blocks: &[BlockData], rounds: usize) -> (f64, StageTimes, usize, usize, usize) {
    let mut k = KakutaniKernel::default();
    let mut out = Vec::new();
    // Warmup + stage timing pass
    let mut st_agg = StageTimes::default();
    let mut max_sp = 0usize;
    let mut max_inl = 0usize;
    for b in blocks {
        let tm = k.run::<MODE, true>(&b.records, b.l.len(), b.p, &mut out);
        st_agg.s1_us += tm.s1_us;
        st_agg.s2a_us += tm.s2a_us;
        st_agg.s2b_us += tm.s2b_us;
        st_agg.s3a_us += tm.s3a_us;
        st_agg.s4_us += tm.s4_us;
        st_agg.s4b_us += tm.s4b_us;
        st_agg.s5_us += tm.s5_us;
        st_agg.m1 += tm.m1;
        st_agg.m2 += tm.m2;
        max_sp = max_sp.max(k.spelling.len());
        max_inl = max_inl.max(k.lens.len());
    }
    let ws = k.ws_bytes();
    let mut times = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let t0 = Instant::now();
        for b in blocks {
            let _ = k.run::<MODE, false>(&b.records, b.l.len(), b.p, &mut out);
        }
        times.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (times[rounds / 2], st_agg, ws, max_sp, max_inl)
}

fn run_corpus_nw<const MODE: u8>(blocks: &[BlockData], workers: usize, rounds: usize) -> f64 {
    let mut times = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let barrier = Barrier::new(workers);
        let t_ms = std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(workers);
            for w_id in 0..workers {
                let b_ref = &barrier;
                handles.push(s.spawn(move || {
                    let mut k = KakutaniKernel::default();
                    let mut out = Vec::new();
                    // Warmup 1 block
                    let wb = &blocks[w_id % blocks.len()];
                    let _ = k.run::<MODE, false>(&wb.records, wb.l.len(), wb.p, &mut out);
                    b_ref.wait();
                    let t0 = Instant::now();
                    for (idx, b) in blocks.iter().enumerate() {
                        if idx % workers == w_id {
                            let _ = k.run::<MODE, false>(&b.records, b.l.len(), b.p, &mut out);
                        }
                    }
                    t0.elapsed().as_secs_f64() * 1e3
                }));
            }
            let mut max_w = 0.0f64;
            for h in handles {
                max_w = max_w.max(h.join().unwrap());
            }
            max_w
        });
        times.push(t_ms);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[rounds / 2]
}

fn verify_and_bench_part3(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    adm6: &[BlockData],
    nonwiki10: &[BlockData],
    wiki233: &[BlockData],
) {
    println!("================================================================================");
    println!("PART 3 — 2-STAGE KAKUTANI INDUCTION (R_2 ⊂ R_1 SKYSCRAPER COMPOSITION):");
    println!("         EXACTNESS VERIFICATION & 1T/2W/4W STEPANOV CONSTRUCTION BENCHMARK");
    println!("================================================================================");

    // 3A. Direct Kakutani (1943) Composition Identity: (T_{R_1})_{R_2} == T_{R_2} & additive tau_{R_2}
    //     across 5,200 small valid + 300 malformed + 16 Adm/NonWiki real blocks (all 3 R_2 modes!)
    let modes = [(1u8, "K2-TallSkip"), (2u8, "K2-Indep2"), (3u8, "K2-Parity50")];
    for &(mode, mname) in &modes {
        let mut tot_m1 = 0usize;
        let mut tot_m2 = 0usize;
        let mut max_ch = 0u64;
        for (l, p) in valid {
            let (_n, m1, m2, ch) = verify_2stage_kakutani_identity(l, *p, mname, mode);
            tot_m1 += m1;
            tot_m2 += m2;
            max_ch = max_ch.max(ch);
        }
        for (l, p) in malformed {
            let (_n, m1, m2, ch) = verify_2stage_kakutani_identity(l, *p, mname, mode);
            tot_m1 += m1;
            tot_m2 += m2;
            max_ch = max_ch.max(ch);
        }
        for b in adm6.iter().chain(nonwiki10.iter()) {
            let (_n, m1, m2, ch) = verify_2stage_kakutani_identity(&b.l, b.p, mname, mode);
            tot_m1 += m1;
            tot_m2 += m2;
            max_ch = max_ch.max(ch);
        }
        println!(
            "  [PASS] Kakutani (T_{{R_1}})_{{R_2}} == T_{{R_2}} & additive tau ({:<11}): 5,200 valid + 300 malformed + 16 real = 5,516/5,516 EXACT! (m2/m1={:.2}%, max_chain={max_ch})",
            mname,
            pct(tot_m2 as u64, tot_m1 as u64)
        );
    }

    // 3B. End-to-End 100% Byte-Exact Kernel Verification (ALL 4 Arms vs ref_fl)
    //     across 5,200 valid + 300 malformed + ALL 249 real blocks!
    let mut k0 = KakutaniKernel::default();
    let mut k1 = KakutaniKernel::default();
    let mut k2 = KakutaniKernel::default();
    let mut k3 = KakutaniKernel::default();
    let mut o0 = Vec::new();
    let mut o1 = Vec::new();
    let mut o2 = Vec::new();
    let mut o3 = Vec::new();

    for (l, p) in valid.iter().chain(malformed.iter()) {
        let recs = l_to_records(l);
        let expected = ref_fl(l, *p);
        let _ = k0.run::<0, false>(&recs, l.len(), *p, &mut o0);
        let _ = k1.run::<1, false>(&recs, l.len(), *p, &mut o1);
        let _ = k2.run::<2, false>(&recs, l.len(), *p, &mut o2);
        let _ = k3.run::<3, false>(&recs, l.len(), *p, &mut o3);
        assert_eq!(o0, expected, "Base-O16 mismatch on small/malformed");
        assert_eq!(o1, expected, "K2-TallSkip mismatch on small/malformed");
        assert_eq!(o2, expected, "K2-Indep2 mismatch on small/malformed");
        assert_eq!(o3, expected, "K2-Parity50 mismatch on small/malformed");
    }
    let mut real_checked = 0usize;
    for b in adm6.iter().chain(nonwiki10.iter()).chain(wiki233.iter()) {
        let expected = ref_fl(&b.l, b.p);
        let _ = k0.run::<0, false>(&b.records, b.l.len(), b.p, &mut o0);
        let _ = k1.run::<1, false>(&b.records, b.l.len(), b.p, &mut o1);
        let _ = k2.run::<2, false>(&b.records, b.l.len(), b.p, &mut o2);
        let _ = k3.run::<3, false>(&b.records, b.l.len(), b.p, &mut o3);
        assert_eq!(o0, expected, "Base-O16 mismatch on {}", b.name);
        assert_eq!(o1, expected, "K2-TallSkip mismatch on {}", b.name);
        assert_eq!(o2, expected, "K2-Indep2 mismatch on {}", b.name);
        assert_eq!(o3, expected, "K2-Parity50 mismatch on {}", b.name);
        real_checked += 1;
    }
    println!(
        "  [PASS] End-to-End Byte-Exactness (ALL 4 Arms vs ref_fl): 5,200 valid + 300 malformed + {real_checked}/249 real blocks 100% BYTE-EXACT!\n"
    );

    // 3C. Paired 1T / 2W / 4W Benchmarks + Stage Breakdown + Working-Set Audit
    //     Use fewer rounds on debug/overflow-checked builds so both builds run cleanly.
    let rounds_adm = if cfg!(debug_assertions) { 1 } else { 11 };
    let rounds_nw = if cfg!(debug_assertions) { 1 } else { 7 };
    let rounds_wiki = if cfg!(debug_assertions) { 1 } else { 3 };

    let bench_corpora: [(&str, &[BlockData], usize); 3] = [
        ("6 Admitted Wikidata", adm6, rounds_adm),
        ("10 Non-Wiki (6 JSONL/Log + 4 Code)", nonwiki10, rounds_nw),
        ("233 Full Wikidata", wiki233, rounds_wiki),
    ];

    for (label, blks, rds) in bench_corpora {
        println!("  [BENCHMARK — {label} ({} blks, {rds} rounds median)]:", blks.len());
        let (t1_b0, st_b0, ws_b0, sp_b0, inl_b0) = run_corpus_1t::<0>(blks, rds);
        let (t1_aa, _st_aa, _, _, _) = run_corpus_1t::<0>(blks, rds);
        let (t1_k1, st_k1, ws_k1, sp_k1, inl_k1) = run_corpus_1t::<1>(blks, rds);
        let (t1_k2, st_k2, ws_k2, sp_k2, inl_k2) = run_corpus_1t::<2>(blks, rds);
        let (t1_k3, st_k3, ws_k3, sp_k3, inl_k3) = run_corpus_1t::<3>(blks, rds);

        let t2_b0 = run_corpus_nw::<0>(blks, 2, rds);
        let t2_aa = run_corpus_nw::<0>(blks, 2, rds);
        let t2_k1 = run_corpus_nw::<1>(blks, 2, rds);
        let t2_k2 = run_corpus_nw::<2>(blks, 2, rds);
        let t2_k3 = run_corpus_nw::<3>(blks, 2, rds);

        let t4_b0 = run_corpus_nw::<0>(blks, 4, rds);
        let t4_aa = run_corpus_nw::<0>(blks, 4, rds);
        let t4_k1 = run_corpus_nw::<1>(blks, 4, rds);
        let t4_k2 = run_corpus_nw::<2>(blks, 4, rds);
        let t4_k3 = run_corpus_nw::<3>(blks, 4, rds);

        println!(
            "  {:<13} | {:>13} {:>13} {:>13} | {:>6} | {:>6} {:>7} {:>7} {:>6} | {:>7} {:>6} {:>7}",
            "Arm", "1T ms (rel)", "2W ms (rel)", "4W ms (rel)", "m2/m1", "S4 ms", "S4b ms", "S5 ms", "S5rel", "WS KiB", "SpKiB", "Codes"
        );
        let rows = [
            ("Base-O16", t1_b0, t2_b0, t4_b0, st_b0, ws_b0, sp_b0, inl_b0),
            ("Base-O16-AA", t1_aa, t2_aa, t4_aa, st_b0, ws_b0, sp_b0, inl_b0),
            ("K2-TallSkip", t1_k1, t2_k1, t4_k1, st_k1, ws_k1, sp_k1, inl_k1),
            ("K2-Indep2", t1_k2, t2_k2, t4_k2, st_k2, ws_k2, sp_k2, inl_k2),
            ("K2-Parity50", t1_k3, t2_k3, t4_k3, st_k3, ws_k3, sp_k3, inl_k3),
        ];
        for (name, t1, t2, t4, st, ws, sp, inl) in rows {
            println!(
                "  {:<13} | {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) | {:>5.1}% | {:>6.2} {:>7.2} {:>7.2} ({:.2}x) | {:>7.1} {:>6.1} {:>7}",
                name,
                t1,
                t1 / t1_b0,
                t2,
                t2 / t2_b0,
                t4,
                t4 / t4_b0,
                pct(st.m2, st.m1),
                st.s4_us / 1e3,
                st.s4b_us / 1e3,
                st.s5_us / 1e3,
                st.s5_us / st_b0.s5_us.max(1e-9),
                ws as f64 / 1024.0,
                sp as f64 / 1024.0,
                inl,
            );
        }
        let _ = st_b0.total_us();
        println!();
    }
}

fn main() {
    let t0 = Instant::now();
    println!("Cycle 51 P2 — Analogue P: Kakutani (1943) / Birkhoff (1931) / Hopf (1937) Ergodic Skyscraper Towers\n");

    verify_part1_3strip_law();

    let (valid, malformed) = generate_small_and_malformed_suites();

    let adm6 = load_six_admitted();
    let nw_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/non_wikidata_blocks.bin",
    ]);
    let cd_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/code_docs_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/code_docs_blocks.bin",
    ]);
    let w_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/wikidata_431_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/wikidata_431_blocks.bin",
    ]);
    let nw_names = ["jsonl_0", "jsonl_1", "jsonl_2", "jsonl_3", "jsonl_4", "syslog_0"];
    let cd_names = ["code_docs_0", "code_docs_1", "code_docs_2", "code_docs_3"];

    let nw6 = load_bin_blocks(&nw_path, Some(&nw_names), false);
    let cd4 = load_bin_blocks(&cd_path, Some(&cd_names), false);
    let wiki233 = load_bin_blocks(&w_path, None, true);
    assert_eq!(adm6.len(), 6);
    assert_eq!(nw6.len(), 6);
    assert_eq!(cd4.len(), 4);
    assert_eq!(wiki233.len(), 233);
    println!("Loaded all 249 real blocks: 6 Adm + 6 JSONL/Syslog + 4 Code/Docs + 233 Full Wikidata.\n");

    let corpora: [(&str, &[BlockData], bool); 4] = [
        ("6 Admitted Wikidata", &adm6, true),
        ("6 JSONL/Syslog", &nw6, true),
        ("4 Code/Docs", &cd4, true),
        ("233 Full Wikidata", &wiki233, false),
    ];
    verify_part2_skyscraper_and_birkhoff_hopf(&valid, &malformed, &corpora);

    let mut nonwiki10: Vec<BlockData> = Vec::with_capacity(10);
    for b in nw6.into_iter().chain(cd4.into_iter()) {
        nonwiki10.push(b);
    }
    verify_and_bench_part3(&valid, &malformed, &adm6, &nonwiki10, &wiki233);

    println!("Total elapsed: {:.2}s", t0.elapsed().as_secs_f64());
}
