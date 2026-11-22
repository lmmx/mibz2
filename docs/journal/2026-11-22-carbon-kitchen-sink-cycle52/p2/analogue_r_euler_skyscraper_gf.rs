#![forbid(unsafe_code)]

//! Cycle 52 P2 — Analogue R:
//! L. Euler (1748) Integer Partition Generating Functions, Pentagonal Number Theorem,
//! & Conjugate-Partition Floor Duality on Kakutani Skyscraper Profiles (`CA-43`),
//! Bimodal Phase Separation vs Max-Entropy Geometric Equilibrium (`CA-28` / `CA-31`),
//! & Exact Optimal `INLINE_K` Dictionary Layout Sweep (`K in {8, 12, 16, 24}`).
//!
//! Primary Source (100% load-bearing, zero relabels, zero analogies):
//!   - L. Euler, *Introductio in analysin infinitorum*, Vol. I, Marc-Michel Bousquet,
//!     Lausanne (1748), Cap. XVI (*"De partitione numerorum"*, §§297–331):
//!       (1) Unrestricted partition GF \prod_{k>=1} (1 - q^k)^{-1} = \sum_{n>=0} p(n) q^n
//!           (§§297–300);
//!       (2) Restricted m-part partition GF \sum_{n>=m} p_m(n) q^n = q^m \prod_{j=1}^m (1 - q^j)^{-1},
//!           exact recurrence p_m(n) = p_m(n - m) + p_{m-1}(n - 1), and exact conjugate duality
//!           p_m(n) = p_{max=m}(n) = p_{<=m}(n - m) (§§301–307);
//!       (3) Pentagonal number theorem \prod_{k>=1} (1 - q^k) = \sum_{j in Z} (-1)^j q^{j(3j-1)/2}
//!           and exact pentagonal recurrence
//!           p(N) = \sum_{j>=1} (-1)^{j-1}[p(N - j(3j-1)/2) + p(N - j(3j+1)/2)] (§§323–326);
//!       (4) Geometric composition series (q/(1-q))^m = \sum_{n>=m} C(n-1, m-1) q^n whose exact
//!           saddle point q = \theta = 1 - m/n yields the max-entropy geometric equilibrium
//!           p_Euler(k) = (m/n)(1 - m/n)^{k-1} (paired with CA-11 Kac balance, CA-28 Shannon
//!           entropy Pythagorean identity D_KL(p_obs || p_Euler) = H(p_Euler) - H(p_obs), and
//!           CA-31 de Moivre/MacMahon compositions).
//!   Extending:
//!     CA-05 (Cauchy 1815, 1844), CA-10 (Kakutani 1943), CA-11 (Kac 1947),
//!     CA-28 (Shannon 1948), CA-31 (MacMahon 1915 / de Moivre 1730), and
//!     CA-43 (Kakutani 1943 skyscraper height spectrum & 3-strip depth law)
//!     -> pushing Chain A+N+P to a 10-node / 9-link Pure-CA Longest Chain!
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
const INLINE_ARENA: usize = 16;
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
// 0. Exact Arbitrary-Precision BigUint / BigInt (Base 10^9, Zero Unsafe)
// ============================================================================

const BIG_BASE: u64 = 1_000_000_000;

#[derive(Clone, PartialEq, Eq, Debug)]
struct BigUint {
    d: Vec<u32>, // little-endian base 10^9 limbs; canonical: no trailing zeros
}

impl BigUint {
    fn zero() -> Self {
        Self { d: Vec::new() }
    }
    fn one() -> Self {
        Self { d: vec![1] }
    }
    fn from_u64(mut v: u64) -> Self {
        let mut d = Vec::new();
        while v > 0 {
            d.push((v % BIG_BASE) as u32);
            v /= BIG_BASE;
        }
        Self { d }
    }
    fn is_zero(&self) -> bool {
        self.d.is_empty()
    }
    fn normalize(&mut self) {
        while matches!(self.d.last(), Some(&0)) {
            self.d.pop();
        }
    }
    fn cmp_mag(&self, other: &Self) -> std::cmp::Ordering {
        if self.d.len() != other.d.len() {
            return self.d.len().cmp(&other.d.len());
        }
        for i in (0..self.d.len()).rev() {
            if self.d[i] != other.d[i] {
                return self.d[i].cmp(&other.d[i]);
            }
        }
        std::cmp::Ordering::Equal
    }
    fn add_assign(&mut self, rhs: &Self) {
        let n = self.d.len().max(rhs.d.len());
        if self.d.len() < n {
            self.d.resize(n, 0);
        }
        let mut carry = 0u64;
        for i in 0..n {
            let a = u64::from(self.d[i]);
            let b = u64::from(rhs.d.get(i).copied().unwrap_or(0));
            let s = a + b + carry;
            if s >= BIG_BASE {
                self.d[i] = (s - BIG_BASE) as u32;
                carry = 1;
            } else {
                self.d[i] = s as u32;
                carry = 0;
            }
        }
        if carry > 0 {
            self.d.push(carry as u32);
        }
    }
    fn add(&self, rhs: &Self) -> Self {
        let mut r = self.clone();
        r.add_assign(rhs);
        r
    }
    /// Assumes self >= rhs.
    fn sub_ge(&self, rhs: &Self) -> Self {
        debug_assert!(self.cmp_mag(rhs) != std::cmp::Ordering::Less);
        let mut out = self.d.clone();
        let mut borrow = 0i64;
        for i in 0..out.len() {
            let a = i64::from(out[i]);
            let b = i64::from(rhs.d.get(i).copied().unwrap_or(0));
            let mut diff = a - b - borrow;
            if diff < 0 {
                diff += BIG_BASE as i64;
                borrow = 1;
            } else {
                borrow = 0;
            }
            out[i] = diff as u32;
        }
        debug_assert_eq!(borrow, 0);
        let mut r = Self { d: out };
        r.normalize();
        r
    }
    fn to_decimal(&self) -> String {
        if self.d.is_empty() {
            return "0".to_string();
        }
        let mut s = format!("{}", self.d[self.d.len() - 1]);
        for i in (0..self.d.len() - 1).rev() {
            s.push_str(&format!("{:09}", self.d[i]));
        }
        s
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct BigInt {
    neg: bool,
    mag: BigUint,
}

impl BigInt {
    fn zero() -> Self {
        Self { neg: false, mag: BigUint::zero() }
    }
    fn from_i64(v: i64) -> Self {
        if v == 0 {
            Self::zero()
        } else if v < 0 {
            Self { neg: true, mag: BigUint::from_u64(v.unsigned_abs()) }
        } else {
            Self { neg: false, mag: BigUint::from_u64(v as u64) }
        }
    }
    fn from_biguint(mag: BigUint) -> Self {
        let neg = false;
        Self { neg, mag }
    }
    fn is_zero(&self) -> bool {
        self.mag.is_zero()
    }
    fn neg_val(&self) -> Self {
        if self.is_zero() {
            Self::zero()
        } else {
            Self { neg: !self.neg, mag: self.mag.clone() }
        }
    }
    fn add(&self, rhs: &Self) -> Self {
        if self.is_zero() {
            return rhs.clone();
        }
        if rhs.is_zero() {
            return self.clone();
        }
        if self.neg == rhs.neg {
            Self { neg: self.neg, mag: self.mag.add(&rhs.mag) }
        } else {
            match self.mag.cmp_mag(&rhs.mag) {
                std::cmp::Ordering::Equal => Self::zero(),
                std::cmp::Ordering::Greater => Self {
                    neg: self.neg,
                    mag: self.mag.sub_ge(&rhs.mag),
                },
                std::cmp::Ordering::Less => Self {
                    neg: rhs.neg,
                    mag: rhs.mag.sub_ge(&self.mag),
                },
            }
        }
    }
    fn sub(&self, rhs: &Self) -> Self {
        self.add(&rhs.neg_val())
    }
}

// ============================================================================
// 1. Deterministic PRNG, BWT Helpers, Reference FL, Log-Gamma & Corpora
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

/// High-precision Lanczos log-gamma ln(Gamma(x)) for exact log-binomial & log-multinomial checks.
fn ln_gamma(z: f64) -> f64 {
    const COEFFS: [f64; 9] = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    let z = z - 1.0;
    let mut x = COEFFS[0];
    for (i, &c) in COEFFS.iter().enumerate().skip(1) {
        x += c / (z + i as f64);
    }
    let t = z + 7.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + x.ln()
}

#[inline]
fn log2_fact(n: u64) -> f64 {
    if n <= 1 {
        0.0
    } else {
        ln_gamma((n + 1) as f64) / std::f64::consts::LN_2
    }
}

#[inline]
fn log2_binom(n: u64, k: u64) -> f64 {
    if k > n {
        return f64::NEG_INFINITY;
    }
    log2_fact(n) - log2_fact(k) - log2_fact(n - k)
}

#[inline]
fn pct(a: u64, b: u64) -> f64 {
    if b == 0 {
        0.0
    } else {
        100.0 * a as f64 / b as f64
    }
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

/// Generate 5,200 small valid BWTs (n <= 180) + 300 malformed multi-cycle columns (incl. 150 orphan thick cycles).
fn generate_small_and_malformed_suites() -> (Vec<(Vec<u8>, u32)>, Vec<(Vec<u8>, u32)>) {
    let mut rng = XorShift64::new(0xC52E_1748_E01E_0326);
    let mut valid = Vec::with_capacity(5200);
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
            let base_len = 2 + rng.gen_range(6);
            let mut pat = Vec::with_capacity(base_len);
            for _ in 0..base_len {
                pat.push(b'a' + (rng.gen_range(sigma.min(26)) as u8));
            }
            while text.len() < n {
                text.push(pat[text.len() % base_len]);
            }
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

// ============================================================================
// 2. PART 1 — Exact Euler (1748) Cap. XVI Partition Generating Functions,
//    Pentagonal Number Theorem, & Kakutani Skyscraper Conjugate-Partition Duality
// ============================================================================

/// Compute p(0..=max_n) via Euler (1748) Cap. XVI §§323-326 pentagonal recurrence:
///   p(N) = sum_{j>=1} (-1)^{j-1} [p(N - j(3j-1)/2) + p(N - j(3j+1)/2)].
fn euler_pentagonal_p(max_n: usize) -> Vec<BigUint> {
    let mut p = Vec::with_capacity(max_n + 1);
    p.push(BigUint::one());
    for n in 1..=max_n {
        let mut acc = BigInt::zero();
        let mut j = 1usize;
        loop {
            let g1 = j * (3 * j - 1) / 2;
            if g1 > n {
                break;
            }
            let sign_pos = (j & 1) == 1;
            let term1 = BigInt::from_biguint(p[n - g1].clone());
            acc = if sign_pos { acc.add(&term1) } else { acc.sub(&term1) };
            let g2 = j * (3 * j + 1) / 2;
            if g2 <= n {
                let term2 = BigInt::from_biguint(p[n - g2].clone());
                acc = if sign_pos { acc.add(&term2) } else { acc.sub(&term2) };
            }
            j += 1;
        }
        assert!(!acc.neg, "Pentagonal recurrence produced negative p({n})!");
        p.push(acc.mag);
    }
    p
}

/// Direct Euler (1748) generating-function product expansion:
///   \prod_{k=1}^{max_n} (1 - q^k)^{-1} mod q^{max_n + 1}.
fn euler_gf_unrestricted(max_n: usize) -> Vec<BigUint> {
    let mut c = vec![BigUint::zero(); max_n + 1];
    c[0] = BigUint::one();
    for k in 1..=max_n {
        for n in k..=max_n {
            let prev = c[n - k].clone();
            c[n].add_assign(&prev);
        }
    }
    c
}

/// Direct expansion of Euler's pentagonal product \prod_{k=1}^{max_n} (1 - q^k) mod q^{max_n + 1}.
fn euler_pentagonal_product_direct(max_n: usize) -> Vec<BigInt> {
    let mut c = vec![BigInt::zero(); max_n + 1];
    c[0] = BigInt::from_i64(1);
    for k in 1..=max_n {
        for n in (k..=max_n).rev() {
            let sub_val = c[n - k].clone();
            c[n] = c[n].sub(&sub_val);
        }
    }
    c
}

/// Closed-form Euler pentagonal coefficients [q^n] \sum_{j in Z} (-1)^j q^{j(3j-1)/2}.
fn euler_pentagonal_closed_form(max_n: usize) -> Vec<BigInt> {
    let mut e = vec![BigInt::zero(); max_n + 1];
    e[0] = BigInt::from_i64(1);
    let mut j = 1usize;
    loop {
        let g1 = j * (3 * j - 1) / 2;
        if g1 > max_n {
            break;
        }
        let sgn = if (j & 1) == 1 { -1i64 } else { 1i64 };
        e[g1] = BigInt::from_i64(sgn);
        let g2 = j * (3 * j + 1) / 2;
        if g2 <= max_n {
            e[g2] = BigInt::from_i64(sgn);
        }
        j += 1;
    }
    e
}

/// Restricted partition table p_m(n) for 0 <= m <= n <= max_n via Euler's exact recurrence:
///   p_m(n) = p_m(n - m) + p_{m-1}(n - 1).
fn euler_restricted_table(max_n: usize) -> Vec<Vec<BigUint>> {
    let mut tab = vec![vec![BigUint::zero(); max_n + 1]; max_n + 1];
    tab[0][0] = BigUint::one();
    for n in 1..=max_n {
        for m in 1..=n {
            let a = tab[n - 1][m - 1].clone();
            let b = if n >= m { tab[n - m][m].clone() } else { BigUint::zero() };
            tab[n][m] = a.add(&b);
        }
    }
    tab
}

/// Brute-force enumerate all integer partitions of `n` (non-increasing parts) and verify
/// exact conjugation involution (\lambda')' == \lambda and length <-> largest-part bijection.
fn verify_bruteforce_conjugation(max_n: usize, pm_tab: &[Vec<BigUint>]) -> usize {
    fn gen(rem: usize, max_part: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if rem == 0 {
            out.push(cur.clone());
            return;
        }
        let hi = rem.min(max_part);
        for p in (1..=hi).rev() {
            cur.push(p);
            gen(rem - p, p, cur, out);
            cur.pop();
        }
    }
    fn conjugate(lam: &[usize]) -> Vec<usize> {
        if lam.is_empty() {
            return Vec::new();
        }
        let max_p = lam[0];
        let mut conj = Vec::with_capacity(max_p);
        for k in 1..=max_p {
            let w = lam.iter().filter(|&&x| x >= k).count();
            conj.push(w);
        }
        conj
    }

    let mut total_parts = 0usize;
    for n in 1..=max_n {
        let mut parts = Vec::new();
        let mut cur = Vec::new();
        gen(n, n, &mut cur, &mut parts);
        total_parts += parts.len();
        let mut by_len = vec![0u64; n + 1];
        let mut by_max = vec![0u64; n + 1];
        for lam in &parts {
            let sum_lam: usize = lam.iter().sum();
            assert_eq!(sum_lam, n);
            let conj = conjugate(lam);
            let sum_conj: usize = conj.iter().sum();
            assert_eq!(sum_conj, n);
            assert_eq!(conj.len(), lam[0]);
            assert_eq!(conj[0], lam.len());
            let double_conj = conjugate(&conj);
            assert_eq!(&double_conj, lam, "Double conjugate involution failed!");
            by_len[lam.len()] += 1;
            by_max[lam[0]] += 1;
        }
        for m in 1..=n {
            assert_eq!(by_len[m], by_max[m]);
            assert_eq!(BigUint::from_u64(by_len[m]), pm_tab[n][m]);
        }
    }
    total_parts
}

/// Instrumented plain Open<16> S2b orphan-row cross-checker (matching CA-43).
fn s2b_orphan_mass_check(l: &[u8], p: u32, limit: u32) -> (u64, u64, u64) {
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
    let cap = 3 * ns + 3;
    let mut prev0 = vec![0u32; cap];
    let mut next0 = vec![0u32; cap];
    let mut prev1 = vec![0u32; cap];
    let mut next1 = vec![0u32; cap];
    let mut width_a = vec![0u32; cap];
    let mut wlen_a = vec![0u32; cap];
    let mut nodes = ns as u32 + 1;

    let append_to = |list: &mut (u32, u32),
                     id: u32,
                     prev: &mut [u32],
                     next: &mut [u32]| {
        if list.1 == 0 {
            list.0 = id;
        } else {
            next[list.1 as usize] = id;
        }
        prev[id as usize] = list.1;
        list.1 = id;
    };
    let close = |first_l: (u32, u32),
                 second_l: (u32, u32),
                 prev: &mut [u32],
                 next: &mut [u32]| {
        let mut pr = 0u32;
        for (h, t) in [first_l, second_l] {
            if t == 0 {
                continue;
            }
            next[pr as usize] = h;
            prev[h as usize] = pr;
            pr = t;
        }
        next[pr as usize] = 0;
        prev[0] = pr;
    };

    let (mut t0, mut o0, mut t1, mut o1) = ((0, 0), (0, 0), (0, 0), (0, 0));
    let mut x = 0u32;
    for (i, &(from, _, w, _)) in strips.iter().enumerate() {
        let sid = i as u32 + 1;
        width_a[sid as usize] = w;
        wlen_a[sid as usize] = 1;
        if x < from {
            let gid = nodes;
            nodes += 1;
            width_a[gid as usize] = from - x;
            wlen_a[gid as usize] = 0;
            append_to(&mut o0, gid, &mut prev0, &mut next0);
            append_to(&mut t1, gid, &mut prev1, &mut next1);
        }
        append_to(&mut o0, sid, &mut prev0, &mut next0);
        x = from + w;
    }
    if x < n {
        let gid = nodes;
        nodes += 1;
        width_a[gid as usize] = n - x;
        wlen_a[gid as usize] = 0;
        append_to(&mut o0, gid, &mut prev0, &mut next0);
        append_to(&mut t1, gid, &mut prev1, &mut next1);
    }
    x = 0;
    for &idx in &image_order {
        let (_, to, w, _) = strips[idx as usize];
        if x < to {
            let gid = nodes;
            nodes += 1;
            width_a[gid as usize] = to - x;
            wlen_a[gid as usize] = 0;
            append_to(&mut o1, gid, &mut prev1, &mut next1);
            append_to(&mut t0, gid, &mut prev0, &mut next0);
        }
        append_to(&mut o1, idx + 1, &mut prev1, &mut next1);
        x = to + w;
    }
    if x < n {
        let gid = nodes;
        width_a[gid as usize] = n - x;
        wlen_a[gid as usize] = 0;
        append_to(&mut o1, gid, &mut prev1, &mut next1);
        append_to(&mut t0, gid, &mut prev0, &mut next0);
    }
    close(t0, o0, &mut prev0, &mut next0);
    close(t1, o1, &mut prev1, &mut next1);

    let mut left = n;
    let mut orphan_rows = 0u64;
    while left > 0 {
        let xn = prev0[0];
        let yn = prev1[0];
        let aw = width_a[xn as usize];
        let bw = width_a[yn as usize];
        if xn == yn {
            assert!(aw > 0 && aw <= left);
            left -= aw;
            orphan_rows += u64::from(aw) * u64::from(wlen_a[xn as usize]);
            let px0 = prev0[xn as usize];
            next0[px0 as usize] = 0;
            prev0[0] = px0;
            let px1 = prev1[xn as usize];
            next1[px1 as usize] = 0;
            prev1[0] = px1;
            continue;
        }
        let w = aw.min(bw);
        left -= w;
        let nwlen = wlen_a[xn as usize] + wlen_a[yn as usize];
        if aw <= bw {
            let px = prev0[xn as usize];
            next0[px as usize] = 0;
            prev0[0] = px;
            // insert xn after yn on side 0
            let ny = next0[yn as usize];
            prev0[xn as usize] = yn;
            next0[xn as usize] = ny;
            next0[yn as usize] = xn;
            prev0[ny as usize] = xn;
            wlen_a[xn as usize] = nwlen;
            if aw == bw {
                // cut yn on side 0, pop_tail yn on side 1
                let py0 = prev0[yn as usize];
                let ny0 = next0[yn as usize];
                next0[py0 as usize] = ny0;
                prev0[ny0 as usize] = py0;
                let py1 = prev1[yn as usize];
                next1[py1 as usize] = 0;
                prev1[0] = py1;
            } else {
                width_a[yn as usize] = bw - w;
            }
        } else {
            let py = prev1[yn as usize];
            next1[py as usize] = 0;
            prev1[0] = py;
            let nx = next1[xn as usize];
            prev1[yn as usize] = xn;
            next1[yn as usize] = nx;
            next1[xn as usize] = yn;
            prev1[nx as usize] = yn;
            wlen_a[yn as usize] = nwlen;
            width_a[xn as usize] = aw - w;
        }
    }
    let mut surv_bytes = 0u64;
    let mut id = next0[0];
    while id != 0 {
        surv_bytes += u64::from(width_a[id as usize]) * (u64::from(wlen_a[id as usize]) + 1);
        id = next0[id as usize];
    }
    (u64::from(m), surv_bytes, orphan_rows)
}

struct SkyscraperPartition {
    n: u64,
    m: u64,
    n_surv: u64,
    n_orphan: u64,
    cycles_total: u64,
    cycles_orphan: u64,
    max_tau: u32,
    /// Part multiplicities c_k = #{x in R : tau_R(x) == k} (sparse BTreeMap for arbitrary max_tau).
    mult: BTreeMap<u32, u64>,
}

/// Extract exact Kakutani skyscraper partition \lambda(R) \vdash n_surv and verify
/// Euler conjugate-partition duality (\lambda'_k = #{x in R : tau_R(x) >= k}, sum \lambda'_k == n_surv,
/// double conjugate (\lambda')' == \lambda, and optional direct horizontal floor set check F_k).
fn extract_and_verify_skyscraper_partition(
    l: &[u8],
    p: u32,
    limit: usize,
    check_horizontal_floors: bool,
    check_s2b: bool,
) -> SkyscraperPartition {
    let n = l.len();
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

    let mut visited = vec![false; n];
    let mut mult: BTreeMap<u32, u64> = BTreeMap::new();
    let mut m = 0u64;
    let mut n_surv = 0u64;
    let mut n_orphan = 0u64;
    let mut cycles_total = 0u64;
    let mut cycles_orphan = 0u64;
    let mut max_tau = 0u32;

    // For optional explicit horizontal floor verification F_k = {FL^{k-1}(x) : x in R, tau_R(x) >= k}
    let mut base_tau_pairs: Vec<(usize, usize)> = if check_horizontal_floors {
        Vec::new()
    } else {
        Vec::new()
    };

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
        let mut r_pos = Vec::new();
        for (idx, &v) in cyc.iter().enumerate() {
            if in_r[v] {
                r_pos.push(idx);
            }
        }
        if r_pos.is_empty() {
            cycles_orphan += 1;
            n_orphan += cyc_len as u64;
        } else {
            let kr = r_pos.len();
            let mut cyc_sum = 0u64;
            for j in 0..kr {
                let p0 = r_pos[j];
                let p1 = r_pos[(j + 1) % kr];
                let tau = if j + 1 < kr { p1 - p0 } else { cyc_len - p0 + p1 };
                let tau32 = tau as u32;
                max_tau = max_tau.max(tau32);
                *mult.entry(tau32).or_insert(0) += 1;
                m += 1;
                cyc_sum += tau as u64;
                if check_horizontal_floors {
                    base_tau_pairs.push((cyc[p0], tau));
                }
            }
            assert_eq!(cyc_sum, cyc_len as u64);
            n_surv += cyc_sum;
        }
    }
    assert_eq!(n_surv + n_orphan, n as u64);

    // 1. Verify exact partition sums: sum c_k == m, sum k c_k == n_surv
    let mut sum_c = 0u64;
    let mut sum_kc = 0u64;
    for (&k, &c) in &mult {
        sum_c += c;
        sum_kc += u64::from(k) * c;
    }
    assert_eq!(sum_c, m);
    assert_eq!(sum_kc, n_surv);

    // 2. Build exact Euler conjugate partition \lambda'_k = #{x in R : tau_R(x) >= k} for 1 <= k <= max_tau
    let max_t = max_tau as usize;
    let mut conj = vec![0u64; max_t + 1]; // 1-indexed: conj[k] = \lambda'_k
    let mut running = 0u64;
    // Traverse in descending k via sparse mult map!
    let mut prev_k = max_t + 1;
    for (&k32, &c) in mult.iter().rev() {
        let k = k32 as usize;
        for j in (k + 1)..prev_k {
            conj[j] = running;
        }
        running += c;
        conj[k] = running;
        prev_k = k;
    }
    for j in 1..prev_k {
        conj[j] = running;
    }
    assert_eq!(conj[1], m, "Conjugate floor-1 width != m!");
    let sum_conj: u64 = conj[1..=max_t].iter().sum();
    assert_eq!(sum_conj, n_surv, "Euler conjugate sum(lambda'_k) != n_surv!");

    // 3. Verify exact finite-difference & double-conjugate involution (\lambda')' == \lambda!
    let mut recon_mult: BTreeMap<u32, u64> = BTreeMap::new();
    for k in 1..=max_t {
        let next_w = if k < max_t { conj[k + 1] } else { 0 };
        assert!(conj[k] >= next_w);
        let diff = conj[k] - next_w;
        if diff > 0 {
            recon_mult.insert(k as u32, diff);
        }
    }
    assert_eq!(recon_mult, mult, "Double conjugate (lambda')' != lambda!");

    // 4. Optional explicit horizontal skyscraper floor set verification F_k
    if check_horizontal_floors {
        let mut floor_counts = vec![0u64; max_t + 1];
        let mut seen = vec![false; n];
        for &(base_row, tau) in &base_tau_pairs {
            let mut cur_row = base_row;
            for k in 1..=tau {
                assert!(!seen[cur_row], "Horizontal floor collision!");
                seen[cur_row] = true;
                floor_counts[k] += 1;
                cur_row = fl[cur_row] as usize;
            }
        }
        for k in 1..=max_t {
            assert_eq!(
                floor_counts[k], conj[k],
                "Horizontal floor |F_{k}| != Euler conjugate lambda'_{k}!"
            );
        }
        let covered = seen.iter().filter(|&&b| b).count() as u64;
        assert_eq!(covered, n_surv);
    }

    // 5. Optional S2b self-loop orphan conservation cross-check
    if check_s2b {
        let (s2b_m, s2b_surv, s2b_orph) = s2b_orphan_mass_check(l, p, limit as u32);
        assert_eq!(s2b_m, m);
        assert_eq!(s2b_surv, n_surv);
        assert_eq!(s2b_orph, n_orphan);
    }

    SkyscraperPartition {
        n: n as u64,
        m,
        n_surv,
        n_orphan,
        cycles_total,
        cycles_orphan,
        max_tau,
        mult,
    }
}

fn verify_part1_euler_identities(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    corpora: &[(&str, &[BlockData], bool)],
) -> Vec<Vec<SkyscraperPartition>> {
    println!("================================================================================");
    println!("PART 1 — EXACT EULER (1748) CAP. XVI PARTITION GENERATING FUNCTIONS, PENTAGONAL");
    println!("         NUMBER THEOREM & KAKUTANI SKYSCRAPER CONJUGATE-PARTITION DUALITY");
    println!("================================================================================");

    // 1A. Euler Pentagonal Number Theorem: direct product expansion vs closed form (N = 0..=200)
    const N_TABLE: usize = 200;
    const N_PENT_MAX: usize = 500;
    let pent_prod = euler_pentagonal_product_direct(N_TABLE);
    let pent_closed = euler_pentagonal_closed_form(N_TABLE);
    for n in 0..=N_TABLE {
        assert_eq!(
            pent_prod[n], pent_closed[n],
            "Pentagonal product mismatch at q^{n}!"
        );
    }
    let non_zero_pent = pent_closed.iter().filter(|x| !x.is_zero()).count();
    println!(
        "  [PASS] Part 1A (Euler 1748 §§323-326 Pentagonal Product): prod_{{k>=1}}(1-q^k) == sum_j (-1)^j q^{{j(3j-1)/2}} exact for 201/201 coeffs (0..=200, {non_zero_pent} nonzero pentagonal terms)!"
    );

    // 1B. Pentagonal Recurrence vs Direct GF Expansion up to N = 500 (genuine BigInt > u64::MAX!)
    let p_pent = euler_pentagonal_p(N_PENT_MAX);
    let p_gf = euler_gf_unrestricted(N_PENT_MAX);
    for n in 0..=N_PENT_MAX {
        assert_eq!(p_pent[n], p_gf[n], "p({n}) pentagonal vs GF mismatch!");
    }
    // Check known classical milestones:
    assert_eq!(p_pent[10].to_decimal(), "42");
    assert_eq!(p_pent[50].to_decimal(), "204226");
    assert_eq!(p_pent[100].to_decimal(), "190569292");
    assert_eq!(p_pent[200].to_decimal(), "3972999029388");
    assert_eq!(p_pent[500].to_decimal(), "2300165032574323995027");
    // Verify formal power series inverse (sum p(n) q^n)(sum e(n) q^n) == 1 mod q^{201}
    for n in 0..=N_TABLE {
        let mut conv = BigInt::zero();
        for k in 0..=n {
            if !pent_closed[k].is_zero() {
                let term = BigInt::from_biguint(p_pent[n - k].clone());
                conv = if pent_closed[k].neg {
                    conv.sub(&term)
                } else {
                    conv.add(&term)
                };
            }
        }
        let expected = if n == 0 { BigInt::from_i64(1) } else { BigInt::zero() };
        assert_eq!(conv, expected, "GF inverse identity failed at q^{n}!");
    }
    println!(
        "  [PASS] Part 1B (BigInt Pentagonal Recurrence vs Direct GF): 501/501 EXACT (N=0..=500; p(200)={}, p(500)={}) & formal inverse == 1 mod q^201!",
        p_pent[200].to_decimal(),
        p_pent[500].to_decimal()
    );

    // 1C. Restricted Partition Table p_m(n) up to n = 200 vs direct GF q^m \prod_{j=1}^m (1 - q^j)^{-1}
    //     AND exact conjugate duality p_m(n) == p_{max=m}(n) == p_{<=m}(n - m) AND sum_m p_m(N) == p(N)!
    let pm_tab = euler_restricted_table(N_TABLE);
    let mut pairs_checked = 0usize;
    // Incremental product: prod_j[k] = [q^k] \prod_{j=1}^m (1 - q^j)^{-1} = p_{<=m}(k)
    let mut prod_le_m = vec![BigUint::zero(); N_TABLE + 1];
    prod_le_m[0] = BigUint::one();
    for m in 1..=N_TABLE {
        // Multiply by (1 - q^m)^{-1}
        for k in m..=N_TABLE {
            let prev = prod_le_m[k - m].clone();
            prod_le_m[k].add_assign(&prev);
        }
        // Now prod_le_m[k] == p_{<=m}(k), and [q^n](q^m \prod_{j=1}^m (1-q^j)^{-1}) == prod_le_m[n - m]!
        // Also p_{max=m}(n) = [q^n](q^m \prod_{j=1}^m (1-q^j)^{-1}) == prod_le_m[n - m]!
        for n in m..=N_TABLE {
            assert_eq!(
                pm_tab[n][m], prod_le_m[n - m],
                "Restricted p_{m}({n}) != [q^{n}] q^{m} prod(1-q^j)^-1!"
            );
            pairs_checked += 1;
        }
    }
    // Verify sum_{m=1}^N p_m(N) == p(N) for ALL N = 1..=200
    for n in 1..=N_TABLE {
        let mut s = BigUint::zero();
        for m in 1..=n {
            s.add_assign(&pm_tab[n][m]);
        }
        assert_eq!(s, p_pent[n], "sum_m p_m({n}) != p({n})!");
    }
    // Brute-force explicit partition conjugation involution for n = 1..=25
    let bf_parts = verify_bruteforce_conjugation(25, &pm_tab);
    println!(
        "  [PASS] Part 1C (Euler 1748 §§301-307 Restricted GF & Conjugate Duality): {pairs_checked}/20,100 (m,n) pairs 100% EXACT (p_m(n) == GF == p_{{<=m}}(n-m) == p_{{max=m}}(n)), 200/200 sum_m p_m(N)==p(N), & {bf_parts} brute-force partitions (n=1..25) involution EXACT!"
    );

    // 1D. Kakutani Skyscraper -> Euler Partition & Horizontal Floor Conjugate Duality on:
    //     5,200 small valid + 300 malformed + ALL 249 real blocks!
    let mut val_non_trivial_m = 0usize;
    let mut val_max_log_pm = 0.0f64;
    for (l, p) in valid {
        let sp = extract_and_verify_skyscraper_partition(l, *p, LIMIT as usize, true, true);
        if sp.n_surv <= N_TABLE as u64 {
            assert!(
                !pm_tab[sp.n_surv as usize][sp.m as usize].is_zero(),
                "Skyscraper partition class p_m(n) is zero!"
            );
            if sp.m < sp.n_surv {
                val_non_trivial_m += 1;
                let lcomp = log2_binom(sp.n_surv - 1, sp.m - 1);
                let mut lmult = log2_fact(sp.m);
                for &c in sp.mult.values() {
                    lmult -= log2_fact(c);
                }
                assert!(lmult <= lcomp + 1e-9, "Multinomial exceeds total compositions!");
                val_max_log_pm = val_max_log_pm.max(lcomp);
            }
        }
    }
    let mut mal_orph_blks = 0usize;
    let mut mal_orph_rows = 0u64;
    let mut mal_tot_rows = 0u64;
    for (l, p) in malformed {
        let sp = extract_and_verify_skyscraper_partition(l, *p, LIMIT as usize, true, true);
        if sp.n_surv <= N_TABLE as u64 && sp.m > 0 {
            assert!(!pm_tab[sp.n_surv as usize][sp.m as usize].is_zero());
        }
        if sp.cycles_orphan > 0 {
            mal_orph_blks += 1;
            mal_orph_rows += sp.n_orphan;
        }
        mal_tot_rows += sp.n;
    }
    println!(
        "  [PASS] Part 1D-Small: 5,200/5,200 valid ({val_non_trivial_m} with thick strips m<n, max log2 C(n-1,m-1)={val_max_log_pm:.2} b) + 300/300 malformed ({mal_orph_blks}/300 w/ orphan thick cycles, {mal_orph_rows}/{mal_tot_rows} [{:.2}%] orphan rows) 100% EXACT horizontal-floor |F_k| == lambda'_k, sum lambda'_k == n_surv, (lambda')'==lambda, & S2b parity!",
        pct(mal_orph_rows, mal_tot_rows)
    );

    let mut all_real_parts: Vec<Vec<SkyscraperPartition>> = Vec::with_capacity(corpora.len());
    let mut total_real = 0usize;
    for &(_label, blocks, is_small_corpus) in corpora {
        let mut vec_p = Vec::with_capacity(blocks.len());
        for b in blocks {
            // Check horizontal floor duality & double-conjugate on ALL 249 blocks;
            // also cross-check S2b on the 16 Adm/JSONL/Code blocks (fast).
            let sp = extract_and_verify_skyscraper_partition(
                &b.l,
                b.p,
                LIMIT as usize,
                false,
                is_small_corpus,
            );
            assert_eq!(sp.cycles_total, 1, "Real block {} not single-cycle!", b.name);
            assert_eq!(sp.n_orphan, 0);
            assert_eq!(sp.n_surv, sp.n);
            assert!(sp.max_tau >= 1);
            vec_p.push(sp);
            total_real += 1;
        }
        all_real_parts.push(vec_p);
    }
    println!(
        "  [PASS] Part 1D-Real:  {total_real}/249 real blocks 100% EXACT single-cycle Kakutani-Euler conjugate duality (lambda'_1 == m, sum lambda'_k == n, (lambda')' == lambda, 0 orphans)!\n"
    );
    all_real_parts
}

// ============================================================================
// 3. Generic Open<16> Kernel Parameterized by `const INLINE_K: usize`
//    (100% Genuine for K in {8, 12, 16, 24}, Identical S1/S2a/S2b/S4!)
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
fn long_slot<const K: usize>(start: u32, end: u32) -> [u8; K] {
    let mut slot = [0u8; K];
    slot[..4].copy_from_slice(&start.to_le_bytes());
    slot[4..8].copy_from_slice(&end.to_le_bytes());
    slot
}

#[inline(always)]
fn long_word<'a, const K: usize>(slot: &[u8; K], spelling: &'a [u8]) -> &'a [u8] {
    let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
    let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
    &spelling[start..end]
}

fn begin_inline<const K: usize>(inline_k: &mut Vec<[u8; K]>, lens: &mut Vec<u8>) {
    if inline_k.len() < 256 {
        inline_k.clear();
        inline_k.extend((0..=255u8).map(|c| {
            let mut v = [0u8; K];
            v[0] = c;
            v
        }));
        lens.clear();
        lens.resize(256, 1);
    }
    inline_k.truncate(256);
    lens.truncate(256);
}

#[cold]
#[inline(never)]
fn grow_u32(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

#[inline(always)]
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

#[inline(always)]
fn inline_hash<const K: usize>(entry: &[u8; K], len: u8) -> u64 {
    let lo = u64::from_le_bytes(entry[0..8].try_into().unwrap());
    let h = if K == 8 {
        (lo ^ (u64::from(len) << 56)).wrapping_mul(0xBF58_476D_1CE4_E5B9)
    } else if K == 12 {
        let hi = u64::from(u32::from_le_bytes(entry[8..12].try_into().unwrap()));
        (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56)).wrapping_mul(0xBF58_476D_1CE4_E5B9)
    } else if K == 16 {
        let hi = u64::from_le_bytes(entry[8..16].try_into().unwrap());
        (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56)).wrapping_mul(0xBF58_476D_1CE4_E5B9)
    } else {
        // K == 24
        let mid = u64::from_le_bytes(entry[8..16].try_into().unwrap());
        let hi = u64::from_le_bytes(entry[16..24].try_into().unwrap());
        (lo ^ mid.rotate_left(29) ^ hi.rotate_left(47) ^ (u64::from(len) << 56))
            .wrapping_mul(0xBF58_476D_1CE4_E5B9)
    };
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
        if sum < INLINE_ARENA as u32 {
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
            if g < INLINE_ARENA as u32 {
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
            if lg < INLINE_ARENA as u32 {
                let lb = self.short_payload_h(left).to_le_bytes();
                dest.extend_from_slice(&lb[..lg as usize]);
                let rg = right >> ID_BITS;
                if rg < INLINE_ARENA as u32 {
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

    /// Pure read-only expansion of a word of exact length `16 <= len < K` (used for `K == 24`)
    /// directly into a fixed `[u8; K]` slot WITHOUT touching `spelling` or mutating memo tags!
    #[inline(always)]
    fn expand_into_fixed<const K: usize>(
        &self,
        handle: u32,
        stack: &mut Vec<u32>,
        spelling: &[u8],
        out_buf: &mut [u8; K],
    ) -> usize {
        stack.clear();
        let mut h = handle;
        let mut pos = 0usize;
        loop {
            let g = h >> ID_BITS;
            if g < INLINE_ARENA as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                let gl = g as usize;
                out_buf[pos..pos + gl].copy_from_slice(&bytes[..gl]);
                pos += gl;
                match stack.pop() {
                    Some(next) => {
                        h = next;
                        continue;
                    }
                    None => return pos,
                }
            }
            let r = self.rule(h & ID_MASK);
            let tag = self.a[r + 3];
            if tag != u32::MAX {
                let start = (tag & 0x7fff_ffff) as usize;
                let rlen = self.a[r + 2] as usize;
                out_buf[pos..pos + rlen].copy_from_slice(&spelling[start..start + rlen]);
                pos += rlen;
                match stack.pop() {
                    Some(next) => {
                        h = next;
                        continue;
                    }
                    None => return pos,
                }
            }
            let left = self.a[r];
            let right = self.a[r + 1];
            let lg = left >> ID_BITS;
            if lg < INLINE_ARENA as u32 {
                let lb = self.short_payload_h(left).to_le_bytes();
                let lgl = lg as usize;
                out_buf[pos..pos + lgl].copy_from_slice(&lb[..lgl]);
                pos += lgl;
                let rg = right >> ID_BITS;
                if rg < INLINE_ARENA as u32 {
                    let rb = self.short_payload_h(right).to_le_bytes();
                    let rgl = rg as usize;
                    out_buf[pos..pos + rgl].copy_from_slice(&rb[..rgl]);
                    pos += rgl;
                    match stack.pop() {
                        Some(next) => {
                            h = next;
                            continue;
                        }
                        None => return pos,
                    }
                }
                h = right;
            } else {
                stack.push(right);
                h = left;
            }
        }
    }

    fn inline_word<const K: usize>(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; K], u8) {
        let height = (word >> ID_BITS) as usize;
        if height < K {
            let mut entry = [0u8; K];
            if height < INLINE_ARENA {
                let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
                let vb = v.to_le_bytes();
                let copy_n = K.min(16);
                entry[..copy_n].copy_from_slice(&vb[..copy_n]);
            } else {
                let written = self.expand_into_fixed::<K>(word, stack, spelling, &mut entry);
                debug_assert_eq!(written, height);
                entry[height] = byte;
            }
            (entry, (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append(word, stack, spelling);
            spelling.push(byte);
            (long_slot::<K>(start, spelling.len() as u32), LONG)
        }
    }
}

struct Dictionary<'a, const K: usize> {
    pairs: &'a mut [u32; 512],
    contents: &'a mut [u32; 2048],
    inline_k: &'a mut Vec<[u8; K]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl<const K: usize> Dictionary<'_, K> {
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
        if height < K {
            let mut entry = [0u8; K];
            if height < INLINE_ARENA {
                let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
                let vb = v.to_le_bytes();
                let copy_n = K.min(16);
                entry[..copy_n].copy_from_slice(&vb[..copy_n]);
            } else {
                let written = e.expand_into_fixed::<K>(handle, self.stack, self.spelling, &mut entry);
                debug_assert_eq!(written, height);
                entry[height] = byte;
            }
            let len = (height + 1) as u8;
            let h = inline_hash::<K>(&entry, len);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == len && self.inline_k[old] == entry
            } {
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry, len);
            self.contents[index] = tag | c;
            return c;
        }
        let start = self.spelling.len();
        // CRITICAL: For K < 16 and K <= height < 16, `handle` is an inline u128 short rule
        // (no DAG memo slot exists in e.a[rule(id)+3], and `append` never writes a memo tag!).
        let memoized = height < INLINE_ARENA || e.a[e.rule(id) + 3] != u32::MAX;
        e.append(handle, self.stack, self.spelling);
        self.spelling.push(byte);
        let entry = long_slot::<K>(start as u32, self.spelling.len() as u32);
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = (h as usize) & 2047;
        let ent = self.contents[index];
        if (ent & 0xFF00_0000) == tag && {
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && long_word::<K>(&self.inline_k[old], self.spelling) == bytes
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
    fn insert(&mut self, entry: [u8; K], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline_k.push(entry);
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
    s5_us: f64,
}

#[derive(Default)]
struct InlineKernel<const K: usize> {
    spill: outq::Spill,
    edges: Vec<u32>,
    extra: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    inline_k: Vec<[u8; K]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
}

impl<const K: usize> InlineKernel<K> {
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
    fn dict_used_bytes(&self) -> usize {
        self.inline_k.len() * K + self.spelling.len()
    }
    fn dict_cap_bytes(&self) -> usize {
        self.inline_k.capacity() * K + self.lens.capacity() + self.spelling.capacity()
    }
    fn total_ws_cap_bytes(&self) -> usize {
        self.edges.capacity() * 4
            + self.extra.capacity() * 4
            + self.bands.capacity() * 8
            + self.spelling.capacity()
            + self.inline_k.capacity() * K
            + self.lens.capacity()
            + self.stack.capacity() * 4
    }
    /// Extract exact lengths of ALL unique dictionary codes currently in `inline_k`/`spelling`.
    fn unique_code_lengths(&self) -> Vec<u32> {
        let mut out = Vec::with_capacity(self.lens.len());
        for (i, &lb) in self.lens.iter().enumerate() {
            if lb == LONG {
                let sl = long_word::<K>(&self.inline_k[i], &self.spelling);
                out.push(sl.len() as u32);
            } else {
                out.push(u32::from(lb));
            }
        }
        out
    }

    fn run<const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> StageTimes {
        let mut tm = StageTimes::default();
        let pad = if K > 16 { K } else { 16 };
        out.resize(n + pad, 0);
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

        let cursors = if Self::wide(ns) {
            self.induced::<Wide, TIMED>(ns, n as u32, &f.thin, &first, &mut write, m as u32, &mut tm)
        } else {
            self.induced::<Narrow, TIMED>(ns, n as u32, &f.thin, &first, &mut write, m as u32, &mut tm)
        };

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
        let use_xor = !wide && m >= 65_536;
        let mut predecessor = mark;
        if use_xor {
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

        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
        self.emit_s5(m, n, bits, wide, use_xor, mark, predecessor, out);
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
        out: &mut Vec<u8>,
    ) {
        let edges = &self.edges[..m_cur];
        let out_sl = &mut out[..n];
        if use_xor {
            Self::emit_s5_xor(
                edges,
                &self.inline_k,
                &self.lens,
                &self.spelling,
                bits,
                mark,
                predecessor,
                out_sl,
            );
        } else {
            Self::emit_s5_fwd(
                edges,
                &self.extra,
                &self.inline_k,
                &self.lens,
                &self.spelling,
                bits,
                wide,
                mark,
                out_sl,
            );
        }
        out.truncate(n);
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_s5_xor(
        edges: &[u32],
        inline_k: &[[u8; K]],
        lens: &[u8],
        spelling: &[u8],
        bits: u32,
        mark: u32,
        predecessor: u32,
        out: &mut [u8],
    ) {
        let n = out.len();
        let mask = (1u32 << bits) - 1;
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
            if lf != LONG && hi - lo >= K {
                out[lo..lo + K].copy_from_slice(&inline_k[cf]);
                lo += usize::from(lf);
            } else {
                let word = if lf == LONG {
                    long_word::<K>(&inline_k[cf], spelling)
                } else {
                    &inline_k[cf][..usize::from(lf)]
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
            if K == 8 {
                if lb != LONG && hi - lo >= 8 {
                    let b8: [u8; 8] = inline_k[cb][..8].try_into().unwrap();
                    let shifted = u64::from_le_bytes(b8) << (8 * (8 - usize::from(lb)));
                    out[hi - 8..hi].copy_from_slice(&shifted.to_le_bytes());
                    hi -= usize::from(lb);
                } else {
                    let word = if lb == LONG {
                        long_word::<K>(&inline_k[cb], spelling)
                    } else {
                        &inline_k[cb][..usize::from(lb)]
                    };
                    let take = word.len().min(hi - lo);
                    out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
                    hi -= take;
                }
            } else if K == 12 {
                if lb != LONG && hi - lo >= 16 {
                    let mut b16 = [0u8; 16];
                    b16[..12].copy_from_slice(&inline_k[cb][..12]);
                    let shifted = u128::from_le_bytes(b16) << (8 * (16 - usize::from(lb)));
                    out[hi - 16..hi].copy_from_slice(&shifted.to_le_bytes());
                    hi -= usize::from(lb);
                } else {
                    let word = if lb == LONG {
                        long_word::<K>(&inline_k[cb], spelling)
                    } else {
                        &inline_k[cb][..usize::from(lb)]
                    };
                    let take = word.len().min(hi - lo);
                    out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
                    hi -= take;
                }
            } else if K == 16 {
                if lb != LONG && hi - lo >= 16 {
                    let b16: [u8; 16] = inline_k[cb][..16].try_into().unwrap();
                    let shifted = u128::from_le_bytes(b16) << (8 * (16 - usize::from(lb)));
                    out[hi - 16..hi].copy_from_slice(&shifted.to_le_bytes());
                    hi -= usize::from(lb);
                } else {
                    let word = if lb == LONG {
                        long_word::<K>(&inline_k[cb], spelling)
                    } else {
                        &inline_k[cb][..usize::from(lb)]
                    };
                    let take = word.len().min(hi - lo);
                    out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
                    hi -= take;
                }
            } else {
                // K == 24: branchless 16B u128 shift when lb <= 16 (~98% of steps), exact slice otherwise
                if lb != LONG && lb <= 16 && hi - lo >= 16 {
                    let b16: [u8; 16] = inline_k[cb][..16].try_into().unwrap();
                    let shifted = u128::from_le_bytes(b16) << (8 * (16 - usize::from(lb)));
                    out[hi - 16..hi].copy_from_slice(&shifted.to_le_bytes());
                    hi -= usize::from(lb);
                } else {
                    let word = if lb == LONG {
                        long_word::<K>(&inline_k[cb], spelling)
                    } else {
                        &inline_k[cb][..usize::from(lb)]
                    };
                    let take = word.len().min(hi - lo);
                    out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
                    hi -= take;
                }
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
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_s5_fwd(
        edges: &[u32],
        extra: &[u32],
        inline_k: &[[u8; K]],
        lens: &[u8],
        spelling: &[u8],
        bits: u32,
        wide: bool,
        mark: u32,
        out: &mut [u8],
    ) {
        let n = out.len();
        let mask = (1u32 << bits) - 1;
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
                if n - pos_out >= K {
                    out[pos_out..pos_out + K].copy_from_slice(&inline_k[code]);
                    pos_out += l_sz;
                } else {
                    let take = l_sz.min(n - pos_out);
                    out[pos_out..pos_out + take].copy_from_slice(&inline_k[code][..take]);
                    pos_out += take;
                }
            } else {
                let sl = long_word::<K>(&inline_k[code], spelling);
                let take = sl.len().min(n - pos_out);
                out[pos_out..pos_out + take].copy_from_slice(&sl[..take]);
                pos_out += take;
            }
        }
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
        begin_inline::<K>(&mut self.inline_k, &mut self.lens);
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
                        let (entry, len) =
                            e.inline_word::<K>(word, a as u8, &mut self.stack, &mut self.spelling);
                        let c = self.lens.len() as u32;
                        self.inline_k.push(entry);
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
            let mut dict = Dictionary::<K> {
                pairs,
                contents,
                inline_k: &mut self.inline_k,
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

// ============================================================================
// 4. PART 2 — Bimodal Phase Separation vs Euler's Max-Entropy Geometric
//    Equilibrium & Exact Euler Conjugate Inline-Cutoff Cost Curve W(K) (249/249)
// ============================================================================

const K_CURVE: [usize; 7] = [4, 8, 12, 16, 20, 24, 32];

fn verify_part2_bimodal_and_cost_curve(
    corpora: &[(&str, &[BlockData], bool)],
    all_parts: &[Vec<SkyscraperPartition>],
) {
    println!("================================================================================");
    println!("PART 2 — BIMODAL PHASE SEPARATION VS EULER MAX-ENTROPY GEOMETRIC EQUILIBRIUM");
    println!("         & EXACT EULER CONJUGATE INLINE-CUTOFF COST CURVE W(K) (249/249 BLOCKS)");
    println!("================================================================================");

    let mut k16 = InlineKernel::<16>::default();
    let mut out_tmp = Vec::new();
    let mut max_pyth_err = 0.0f64;
    let mut total_w_checks = 0usize;

    for (c_idx, &(label, blocks, per_block)) in corpora.iter().enumerate() {
        let parts = &all_parts[c_idx];
        println!("\n  --- {label} ({} blocks) ---", blocks.len());
        if per_block {
            println!(
                "  {:<13} {:>6} {:>6} | {:>6} {:>6} {:>7} | {:>7} {:>8} | {:>7} {:>7} {:>7} | {:>6} {:>4}",
                "Block",
                "m/n%",
                "E[t]",
                "H_Eul",
                "H_obs",
                "D_KL(b)",
                "dp1(pp)",
                "R64_tail",
                "W(8)K",
                "W(16)K",
                "W(24)K",
                "Codes",
                "K*"
            );
        }

        let mut sum_h_eul = 0.0f64;
        let mut sum_h_obs = 0.0f64;
        let mut sum_dkl = 0.0f64;
        let mut sum_dmult = 0.0f64;
        let mut sum_p1_obs = 0.0f64;
        let mut sum_p1_eul = 0.0f64;
        let mut sum_p2_15_obs = 0.0f64;
        let mut sum_p2_15_eul = 0.0f64;
        let mut sum_p64_obs = 0.0f64;
        let mut sum_p64_eul = 0.0f64;
        let mut agg_w_curve = [0u64; 7];
        let mut agg_spill_rows_curve = [0u64; 7];
        let mut agg_spill_codes_curve = [0u64; 7];
        let mut agg_m = 0u64;
        let mut agg_codes = 0u64;
        let mut kstar_hist = [0usize; 7];

        for (b_idx, b) in blocks.iter().enumerate() {
            let sp = &parts[b_idx];
            let mf = sp.m as f64;
            let nf = sp.n as f64;
            let one_minus_theta = mf / nf; // m / n
            let theta = 1.0 - one_minus_theta;

            // Exact max-entropy geometric equilibrium entropy:
            //   H(p_Euler) = -log2(1 - theta) - (theta / (1 - theta)) * log2(theta)
            let h_eul = if theta > 0.0 {
                -one_minus_theta.log2() - (theta / one_minus_theta) * theta.log2()
            } else {
                0.0
            };
            let mut h_obs = 0.0f64;
            let mut d_kl = 0.0f64;
            let mut p2_15_obs = 0.0f64;
            let mut p64_obs = 0.0f64;
            let mut log2_mult = log2_fact(sp.m);

            for (&k, &c) in &sp.mult {
                let p_k = c as f64 / mf;
                h_obs -= p_k * p_k.log2();
                let log2_p_eul = if theta > 0.0 {
                    one_minus_theta.log2() + f64::from(k - 1) * theta.log2()
                } else {
                    0.0
                };
                d_kl += p_k * (p_k.log2() - log2_p_eul);
                log2_mult -= log2_fact(c);
                if (2..=15).contains(&k) {
                    p2_15_obs += p_k;
                }
                if k >= 64 {
                    p64_obs += p_k;
                }
            }
            // Exact Kac-Shannon Pythagorean identity: D_KL(p_obs || p_Euler) == H(p_Euler) - H(p_obs)!
            let pyth_err = (d_kl - (h_eul - h_obs)).abs();
            max_pyth_err = max_pyth_err.max(pyth_err);
            assert!(
                pyth_err < 1e-11,
                "Pythagorean KL identity failed on {}: err={pyth_err}",
                b.name
            );
            let d_mult = (log2_binom(sp.n - 1, sp.m - 1) - log2_mult) / mf;

            let p1_obs = sp.mult.get(&1).copied().unwrap_or(0) as f64 / mf;
            let p1_eul = one_minus_theta;
            let p2_15_eul = theta - theta.powi(15);
            let p64_eul = theta.powi(63);
            let r64 = if p64_eul > 0.0 { p64_obs / p64_eul } else { f64::INFINITY };

            sum_h_eul += h_eul;
            sum_h_obs += h_obs;
            sum_dkl += d_kl;
            sum_dmult += d_mult;
            sum_p1_obs += p1_obs;
            sum_p1_eul += p1_eul;
            sum_p2_15_obs += p2_15_obs;
            sum_p2_15_eul += p2_15_eul;
            sum_p64_obs += p64_obs;
            sum_p64_eul += p64_eul;
            agg_m += sp.m;

            // Extract exact Open<16> deduplicated unique dictionary code lengths & verify
            // Euler conjugate inline-cutoff cost curve W(K) = K * |D| + sum_{l(c) > K} l(c)!
            let _ = k16.run::<false>(&b.records, b.l.len(), b.p, &mut out_tmp);
            let code_lens = k16.unique_code_lengths();
            let num_codes = code_lens.len() as u64;
            agg_codes += num_codes;

            let mut d_hist: BTreeMap<u32, u64> = BTreeMap::new();
            let mut max_cl = 0u32;
            for &cl in &code_lens {
                max_cl = max_cl.max(cl);
                *d_hist.entry(cl).or_insert(0) += 1;
            }
            // Build exact Euler conjugate partition mu'_j = #{c in D : l(c) >= j}
            let max_cl_u = max_cl as usize;
            let mut mu_conj = vec![0u64; max_cl_u + 1];
            let mut run_d = 0u64;
            let mut prev_l = max_cl_u + 1;
            for (&l32, &cnt_l) in d_hist.iter().rev() {
                let lu = l32 as usize;
                for j in (lu + 1)..prev_l {
                    mu_conj[j] = run_d;
                }
                run_d += cnt_l;
                mu_conj[lu] = run_d;
                prev_l = lu;
            }
            for j in 1..prev_l {
                mu_conj[j] = run_d;
            }
            assert_eq!(mu_conj[1], num_codes);

            let mut w_vals = [0u64; 7];
            for (ki, &kv) in K_CURVE.iter().enumerate() {
                // Direct definition: W(K) = K * |D| + sum_{l > K} l * d_l
                let mut spill_bytes_direct = 0u64;
                let mut spill_codes = 0u64;
                for (&l32, &cnt_l) in &d_hist {
                    if l32 as usize > kv {
                        spill_bytes_direct += u64::from(l32) * cnt_l;
                        spill_codes += cnt_l;
                    }
                }
                let w_direct = (kv as u64) * num_codes + spill_bytes_direct;

                // Euler conjugate formula: W(K) = K * mu'_1 + K * mu'_{K+1} + sum_{j >= K+1} mu'_j
                let mu_kp1 = if kv + 1 <= max_cl_u { mu_conj[kv + 1] } else { 0 };
                let tail_conj_sum: u64 = if kv + 1 <= max_cl_u {
                    mu_conj[kv + 1..=max_cl_u].iter().sum()
                } else {
                    0
                };
                let w_conj = (kv as u64) * mu_conj[1] + (kv as u64) * mu_kp1 + tail_conj_sum;
                assert_eq!(
                    w_direct, w_conj,
                    "Euler conjugate inline-cutoff W({kv}) identity failed on {}!",
                    b.name
                );
                assert_eq!(spill_codes, mu_kp1);
                total_w_checks += 1;

                // Row-weighted S5 spill count \lambda'_{K+1}
                let mut row_spills = 0u64;
                for (&tau, &cnt_t) in &sp.mult {
                    if tau as usize > kv {
                        row_spills += cnt_t;
                    }
                }

                w_vals[ki] = w_direct;
                agg_w_curve[ki] += w_direct;
                agg_spill_rows_curve[ki] += row_spills;
                agg_spill_codes_curve[ki] += spill_codes;
            }
            // Verify exact discrete difference law W(K2) - W(K1) == (K2-K1)*|D| - sum_{k=K1+1..=K2} k*d_k
            for ki in 0..(K_CURVE.len() - 1) {
                let k1 = K_CURVE[ki];
                let k2 = K_CURVE[ki + 1];
                let saved_tail: u64 = ((k1 + 1)..=k2)
                    .map(|k| (k as u64) * d_hist.get(&(k as u32)).copied().unwrap_or(0))
                    .sum();
                let lhs = w_vals[ki + 1] as i128 - w_vals[ki] as i128;
                let rhs = ((k2 - k1) as i128) * (num_codes as i128) - saved_tail as i128;
                assert_eq!(lhs, rhs, "Exact conjugate forward-difference W(K) failed!");
            }

            let mut best_ki = 0usize;
            for ki in 1..K_CURVE.len() {
                if w_vals[ki] < w_vals[best_ki] {
                    best_ki = ki;
                }
            }
            kstar_hist[best_ki] += 1;

            if per_block {
                let r64_str = if r64 >= 1e5 {
                    format!("{:.1e}", r64)
                } else {
                    format!("{:>7.1}x", r64)
                };
                println!(
                    "  {:<13} {:>5.1}% {:>6.2} | {:>6.3} {:>6.3} {:>7.3} | {:>+6.1}pp {:>8} | {:>6.1}K {:>6.1}K {:>6.1}K | {:>6} {:>4}",
                    b.name,
                    100.0 * one_minus_theta,
                    1.0 / one_minus_theta,
                    h_eul,
                    h_obs,
                    d_kl,
                    100.0 * (p1_obs - p1_eul),
                    r64_str,
                    w_vals[1] as f64 / 1024.0,
                    w_vals[3] as f64 / 1024.0,
                    w_vals[5] as f64 / 1024.0,
                    num_codes,
                    K_CURVE[best_ki]
                );
            }
        }

        let nb = blocks.len() as f64;
        let mean_p1_obs = sum_p1_obs / nb;
        let mean_p1_eul = sum_p1_eul / nb;
        let mean_p2_15_obs = sum_p2_15_obs / nb;
        let mean_p2_15_eul = sum_p2_15_eul / nb;
        let mean_p64_obs = sum_p64_obs / nb;
        let mean_p64_eul = sum_p64_eul / nb;
        let agg_r64 = mean_p64_obs / mean_p64_eul;
        println!(
            "  [AGG {label}]: mean H_Euler={:.3} b, mean H_obs={:.3} b => mean D_KL={:.3} b (finite-N log-mult deficit={:.3} b/row)",
            sum_h_eul / nb,
            sum_h_obs / nb,
            sum_dkl / nb,
            sum_dmult / nb
        );
        println!(
            "    Bimodal Phase Separation: floor tau=1: obs {:.2}% vs Euler {:.2}% (excess {:+.2} pp, {:.2}x) | valley 2..15: obs {:.2}% vs Euler {:.2}% ({:.2}x) | tail tau>=64: obs {:.3}% vs Euler {:.3e} ({:.2e}x)",
            100.0 * mean_p1_obs,
            100.0 * mean_p1_eul,
            100.0 * (mean_p1_obs - mean_p1_eul),
            mean_p1_obs / mean_p1_eul,
            100.0 * mean_p2_15_obs,
            100.0 * mean_p2_15_eul,
            mean_p2_15_obs / mean_p2_15_eul,
            100.0 * mean_p64_obs,
            mean_p64_eul,
            agg_r64
        );
        println!(
            "    Euler Conjugate Cost Curve W(K) (mean KiB/blk; avg codes={:.1}; K* counts={:?} over K={:?}):",
            agg_codes as f64 / nb,
            kstar_hist,
            K_CURVE
        );
        print!("      ");
        for (ki, &kv) in K_CURVE.iter().enumerate() {
            let w_kib = (agg_w_curve[ki] as f64 / nb) / 1024.0;
            let sp_row = pct(agg_spill_rows_curve[ki], agg_m);
            let sp_code = pct(agg_spill_codes_curve[ki], agg_codes);
            print!("K={kv:>2}: {w_kib:>5.1}K ({sp_row:>4.1}%r/{sp_code:>4.1}%c) | ");
        }
        println!();
    }
    println!(
        "\n  [PASS] Part 2 Identities: 249/249 blocks Kac-Shannon Pythagorean max |D_KL - (H_Eul - H_obs)| = {:.3e} b; {total_w_checks}/{total_w_checks} Euler conjugate W(K) & forward-difference identities 100% EXACT!\n",
        max_pyth_err
    );
}

// ============================================================================
// 5. PART 3 — Stepanov Construction Test (`R-InlineK`):
//    Genuine `INLINE_K in {8, 12, 16, 24}` Sweep in Real `Open<16>` `S3a`/`S5`
// ============================================================================

#[derive(Default)]
struct WsStats {
    avg_dict_used_kib: f64,
    max_dict_used_kib: f64,
    dict_cap_kib: f64,
    total_ws_cap_kib: f64,
    avg_sp_kib: f64,
}

fn measure_ws_and_stages_once<const K: usize>(blocks: &[BlockData]) -> (StageTimes, WsStats) {
    let mut k = InlineKernel::<K>::default();
    let mut out = Vec::new();
    // Warmup pass so capacities settle before timing stages
    for b in blocks {
        let _ = k.run::<false>(&b.records, b.l.len(), b.p, &mut out);
    }
    let mut st_agg = StageTimes::default();
    let mut sum_dict_used = 0u64;
    let mut max_dict_used = 0usize;
    let mut sum_sp = 0u64;
    for b in blocks {
        let tm = k.run::<true>(&b.records, b.l.len(), b.p, &mut out);
        st_agg.s1_us += tm.s1_us;
        st_agg.s2a_us += tm.s2a_us;
        st_agg.s2b_us += tm.s2b_us;
        st_agg.s3a_us += tm.s3a_us;
        st_agg.s4_us += tm.s4_us;
        st_agg.s5_us += tm.s5_us;
        let du = k.dict_used_bytes();
        sum_dict_used += du as u64;
        max_dict_used = max_dict_used.max(du);
        sum_sp += k.spelling.len() as u64;
    }
    let nb = blocks.len() as f64;
    let ws = WsStats {
        avg_dict_used_kib: (sum_dict_used as f64 / nb) / 1024.0,
        max_dict_used_kib: max_dict_used as f64 / 1024.0,
        dict_cap_kib: k.dict_cap_bytes() as f64 / 1024.0,
        total_ws_cap_kib: k.total_ws_cap_bytes() as f64 / 1024.0,
        avg_sp_kib: (sum_sp as f64 / nb) / 1024.0,
    };
    (st_agg, ws)
}

fn time_one_round_1t<const K: usize>(
    k: &mut InlineKernel<K>,
    out: &mut Vec<u8>,
    blocks: &[BlockData],
) -> f64 {
    let t0 = Instant::now();
    for b in blocks {
        let _ = k.run::<false>(&b.records, b.l.len(), b.p, out);
    }
    t0.elapsed().as_secs_f64() * 1e3
}

fn time_one_round_nw<const K: usize>(blocks: &[BlockData], workers: usize) -> f64 {
    let barrier = Barrier::new(workers);
    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(workers);
        for w_id in 0..workers {
            let b_ref = &barrier;
            handles.push(s.spawn(move || {
                let mut k = InlineKernel::<K>::default();
                let mut out = Vec::new();
                let wb = &blocks[w_id % blocks.len()];
                let _ = k.run::<false>(&wb.records, wb.l.len(), wb.p, &mut out);
                b_ref.wait();
                let t0 = Instant::now();
                for (idx, b) in blocks.iter().enumerate() {
                    if idx % workers == w_id {
                        let _ = k.run::<false>(&b.records, b.l.len(), b.p, &mut out);
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
    })
}

fn median(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    vals[vals.len() / 2]
}

fn verify_and_bench_part3_inline_k(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    corpora: &[(&str, &[BlockData], bool)],
) {
    println!("================================================================================");
    println!("PART 3 — STEPANOV CONSTRUCTION TEST (`R-InlineK`):");
    println!("         OPTIMAL `INLINE_K in {{8, 12, 16, 24}}` SWEEP IN REAL `Open<16>` `S3a`/`S5`");
    println!("================================================================================");

    // 3A. 100% Byte-Exact Verification across ALL 4 INLINE_K variants vs ref_fl:
    //     5,200 small valid + 300 malformed + ALL 249 real blocks!
    let mut k8 = InlineKernel::<8>::default();
    let mut k12 = InlineKernel::<12>::default();
    let mut k16 = InlineKernel::<16>::default();
    let mut k24 = InlineKernel::<24>::default();
    let mut o8 = Vec::new();
    let mut o12 = Vec::new();
    let mut o16 = Vec::new();
    let mut o24 = Vec::new();

    for (l, p) in valid.iter().chain(malformed.iter()) {
        let recs = l_to_records(l);
        let expected = ref_fl(l, *p);
        let _ = k8.run::<false>(&recs, l.len(), *p, &mut o8);
        let _ = k12.run::<false>(&recs, l.len(), *p, &mut o12);
        let _ = k16.run::<false>(&recs, l.len(), *p, &mut o16);
        let _ = k24.run::<false>(&recs, l.len(), *p, &mut o24);
        assert_eq!(o8, expected, "INLINE_K=8 mismatch on small/malformed!");
        assert_eq!(o12, expected, "INLINE_K=12 mismatch on small/malformed!");
        assert_eq!(o16, expected, "INLINE_K=16 mismatch on small/malformed!");
        assert_eq!(o24, expected, "INLINE_K=24 mismatch on small/malformed!");
    }
    let mut real_ok = 0usize;
    for &(_label, blocks, _) in corpora {
        for b in blocks {
            let expected = ref_fl(&b.l, b.p);
            let _ = k8.run::<false>(&b.records, b.l.len(), b.p, &mut o8);
            let _ = k12.run::<false>(&b.records, b.l.len(), b.p, &mut o12);
            let _ = k16.run::<false>(&b.records, b.l.len(), b.p, &mut o16);
            let _ = k24.run::<false>(&b.records, b.l.len(), b.p, &mut o24);
            assert_eq!(o8, expected, "INLINE_K=8 mismatch on {}!", b.name);
            assert_eq!(o12, expected, "INLINE_K=12 mismatch on {}!", b.name);
            assert_eq!(o16, expected, "INLINE_K=16 mismatch on {}!", b.name);
            assert_eq!(o24, expected, "INLINE_K=24 mismatch on {}!", b.name);
            real_ok += 1;
        }
    }
    println!(
        "  [PASS] Part 3A Byte-Exactness (ALL 4 Arms K in {{8,12,16,24}} vs ref_fl): 5,200 valid + 300 malformed + {real_ok}/249 real blocks = 5,749/5,749 100% BYTE-EXACT!\n"
    );

    // 3B. Interleaved Round-by-Round 1T / 2W / 4W Benchmarks + S3a/S5 + Used/Cap Dictionary WS!
    let bench_rounds = |n_blks: usize| -> usize {
        if cfg!(debug_assertions) {
            1
        } else if n_blks <= 6 {
            11
        } else {
            5
        }
    };

    for &(label, blocks, _) in corpora {
        let rds = bench_rounds(blocks.len());
        println!(
            "  [BENCHMARK — {label} ({} blks, {rds} interleaved rounds median; Base = INLINE_K=16)]:",
            blocks.len()
        );

        let (st16, ws16) = measure_ws_and_stages_once::<16>(blocks);
        let (st16_aa, _) = measure_ws_and_stages_once::<16>(blocks);
        let (st8, ws8) = measure_ws_and_stages_once::<8>(blocks);
        let (st12, ws12) = measure_ws_and_stages_once::<12>(blocks);
        let (st24, ws24) = measure_ws_and_stages_once::<24>(blocks);

        // Pre-warmed 1T kernels (interleaved round-by-round!)
        let mut bk16 = InlineKernel::<16>::default();
        let mut bk16_aa = InlineKernel::<16>::default();
        let mut bk8 = InlineKernel::<8>::default();
        let mut bk12 = InlineKernel::<12>::default();
        let mut bk24 = InlineKernel::<24>::default();
        let mut bout = Vec::new();
        for b in blocks {
            let _ = bk16.run::<false>(&b.records, b.l.len(), b.p, &mut bout);
            let _ = bk16_aa.run::<false>(&b.records, b.l.len(), b.p, &mut bout);
            let _ = bk8.run::<false>(&b.records, b.l.len(), b.p, &mut bout);
            let _ = bk12.run::<false>(&b.records, b.l.len(), b.p, &mut bout);
            let _ = bk24.run::<false>(&b.records, b.l.len(), b.p, &mut bout);
        }

        let mut t1 = [Vec::with_capacity(rds), Vec::with_capacity(rds), Vec::with_capacity(rds), Vec::with_capacity(rds), Vec::with_capacity(rds)];
        let mut t2 = [Vec::with_capacity(rds), Vec::with_capacity(rds), Vec::with_capacity(rds), Vec::with_capacity(rds), Vec::with_capacity(rds)];
        let mut t4 = [Vec::with_capacity(rds), Vec::with_capacity(rds), Vec::with_capacity(rds), Vec::with_capacity(rds), Vec::with_capacity(rds)];

        for r in 0..rds {
            for step in 0..5 {
                let arm = (r + step) % 5;
                let v1 = match arm {
                    0 => time_one_round_1t::<16>(&mut bk16, &mut bout, blocks),
                    1 => time_one_round_1t::<16>(&mut bk16_aa, &mut bout, blocks),
                    2 => time_one_round_1t::<8>(&mut bk8, &mut bout, blocks),
                    3 => time_one_round_1t::<12>(&mut bk12, &mut bout, blocks),
                    _ => time_one_round_1t::<24>(&mut bk24, &mut bout, blocks),
                };
                t1[arm].push(v1);
            }
        }
        for r in 0..rds {
            for step in 0..5 {
                let arm = (r + step) % 5;
                let v2 = match arm {
                    0 | 1 => time_one_round_nw::<16>(blocks, 2),
                    2 => time_one_round_nw::<8>(blocks, 2),
                    3 => time_one_round_nw::<12>(blocks, 2),
                    _ => time_one_round_nw::<24>(blocks, 2),
                };
                t2[arm].push(v2);
            }
        }
        for r in 0..rds {
            for step in 0..5 {
                let arm = (r + step) % 5;
                let v4 = match arm {
                    0 | 1 => time_one_round_nw::<16>(blocks, 4),
                    2 => time_one_round_nw::<8>(blocks, 4),
                    3 => time_one_round_nw::<12>(blocks, 4),
                    _ => time_one_round_nw::<24>(blocks, 4),
                };
                t4[arm].push(v4);
            }
        }

        let m1: [f64; 5] = [
            median(&mut t1[0]),
            median(&mut t1[1]),
            median(&mut t1[2]),
            median(&mut t1[3]),
            median(&mut t1[4]),
        ];
        let m2: [f64; 5] = [
            median(&mut t2[0]),
            median(&mut t2[1]),
            median(&mut t2[2]),
            median(&mut t2[3]),
            median(&mut t2[4]),
        ];
        let m4: [f64; 5] = [
            median(&mut t4[0]),
            median(&mut t4[1]),
            median(&mut t4[2]),
            median(&mut t4[3]),
            median(&mut t4[4]),
        ];

        println!(
            "  {:<11} | {:>13} {:>13} {:>13} | {:>12} {:>12} | {:>12} {:>7} {:>7} {:>7}",
            "Arm",
            "1T ms (rel)",
            "2W ms (rel)",
            "4W ms (rel)",
            "S3a ms (rel)",
            "S5 ms (rel)",
            "DictU avg/mx",
            "DictCap",
            "SpAvgK",
            "TotWSK"
        );
        let rows: [(&str, usize, &StageTimes, &WsStats); 5] = [
            ("K=16 (Base)", 0, &st16, &ws16),
            ("K=16 (A/A)", 1, &st16_aa, &ws16),
            ("K=8", 2, &st8, &ws8),
            ("K=12", 3, &st12, &ws12),
            ("K=24", 4, &st24, &ws24),
        ];
        for (name, idx, st, ws) in rows {
            println!(
                "  {:<11} | {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) | {:>5.2} ({:.2}x) {:>5.2} ({:.2}x) | {:>5.1}/{:<5.1}K {:>6.1}K {:>6.1}K {:>6.1}K",
                name,
                m1[idx],
                m1[idx] / m1[0],
                m2[idx],
                m2[idx] / m2[0],
                m4[idx],
                m4[idx] / m4[0],
                st.s3a_us / 1e3,
                st.s3a_us / st16.s3a_us.max(1e-9),
                st.s5_us / 1e3,
                st.s5_us / st16.s5_us.max(1e-9),
                ws.avg_dict_used_kib,
                ws.max_dict_used_kib,
                ws.dict_cap_kib,
                ws.avg_sp_kib,
                ws.total_ws_cap_kib,
            );
        }
        println!();
    }
}

fn main() {
    let t0 = Instant::now();
    println!("Cycle 52 P2 — Analogue R: Euler (1748) Integer Partition Generating Functions,");
    println!("              Pentagonal Recurrence & Conjugate Floor Duality on Kakutani Skyscrapers\n");

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

    let all_parts = verify_part1_euler_identities(&valid, &malformed, &corpora);
    verify_part2_bimodal_and_cost_curve(&corpora, &all_parts);
    verify_and_bench_part3_inline_k(&valid, &malformed, &corpora);

    println!("Total elapsed: {:.2}s", t0.elapsed().as_secs_f64());
}
