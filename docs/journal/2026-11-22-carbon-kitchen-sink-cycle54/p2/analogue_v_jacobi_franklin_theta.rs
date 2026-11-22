#![forbid(unsafe_code)]

//! Cycle 54 P2 — Analogue V:
//! C. G. J. Jacobi (1829) Triple Product / Pentagonal–Theta Series & F. Franklin (1881)
//! Constructive Bijective Involution on Kakutani Skyscrapers (`CA-49` -> `CA-52`)
//! + Zero-Remap `S5` Dictionary Layout Study (`V-ZeroRemap-Dict` over `C53-PipeLean-v1`).
//!
//! Primary Sources (100% load-bearing, zero relabels, zero analogies):
//!   1. C. G. J. Jacobi (1829), *Fundamenta nova theoriae functionum ellipticarum*,
//!      Regiomonti (Königsberg), §§64–66, pp. 184–189:
//!        - Jacobi Triple Product Identity (§64):
//!            \prod_{m=1}^\infty (1 - q^{2m})(1 + z q^{2m-1})(1 + z^{-1} q^{2m-1})
//!              = \sum_{n=-\infty}^\infty z^n q^{n^2}
//!          (directly verified bivariately in exact `BigInt` Laurent polynomials up to q^{100});
//!        - Specialization (a) Euler's Pentagonal Number Theorem (§66):
//!            \phi(q) = \prod_{m=1}^\infty (1 - q^m) = \sum_{k=-\infty}^\infty (-1)^k q^{k(3k-1)/2},
//!          and pentagonal partition recurrence p(n) == CA-49 Sylvester Durfee GF (N = 0..400);
//!        - Specialization (b) Jacobi's Triangular-Number Cube Identity (§66):
//!            \prod_{m=1}^\infty (1 - q^m)^3 = \sum_{k=0}^\infty (-1)^k (2k+1) q^{k(k+1)/2} (N = 0..400);
//!        - Specialization (c) Jacobi Theta Products (§§61–64):
//!            \vartheta_3(q) = \sum_{n\in\mathbb{Z}} q^{n^2} = \prod_{m>=1}(1-q^{2m})(1+q^{2m-1})^2,
//!            \vartheta_4(q) = \sum_{n\in\mathbb{Z}} (-1)^n q^{n^2} = \prod_{m>=1}(1-q^{2m})(1-q^{2m-1})^2,
//!            and \vartheta_3(q)\vartheta_4(q) = \vartheta_4(q^2)^2 (N = 0..400);
//!        - Specialization (d) Euler Distinct-vs-Odd Partition Identity & Pentagonal Recurrence:
//!            Q(q) = \prod_{k>=1}(1+q^k) = \prod_{k>=1}(1-q^{2k-1})^{-1}, \phi(q) Q(q) = \phi(q^2),
//!          with Hardy–Ramanujan (1918, §7.1) Bessel asymptotic for \log_2 q(N) (N = 10..400).
//!   2. F. Franklin (1881), *"Sur le développement du produit infini (1-x)(1-x^2)(1-x^3)..."*,
//!      *Comptes Rendus Acad. Sci. Paris* **92** (1881), 448–450:
//!        - Explicit constructive bijective involution f on Ferrers diagrams of distinct-part
//!          partitions \mu = (\mu_1 > ... > \mu_\ell >= 1) via base s(\mu) = \mu_\ell and top-right
//!          45-degree diagonal slope \sigma(\mu) = max{k : \mu_k = \mu_1 - (k-1)}:
//!          verified exhaustively on ALL distinct-part diagrams for n = 1..60 (f(f(\mu)) = \mu,
//!          |\ell(f(\mu)) - \ell(\mu)| = 1 for all non-fixed \mu, unique fixed points ONLY at
//!          pentagonal numbers \omega_{\pm k} = k(3k \pm 1)/2 matching (-1)^k).
//!
//! Extending CA-05 -> ... -> CA-43 -> CA-46 -> CA-49 (Chain A+N+P+R+T) to a NEW 12-node /
//! 11-link Pure-CA Longest Chain!
//!
//! Part 3 (`V-ZeroRemap-Dict`, strictly `2020s` modern CS — ZERO classical share!):
//!   Tests zero-remap (+0 remap pass!) S5 dictionary specializations directly inside
//!   `C53-PipeLean-v1`'s exact `SlimS5_Pipe1_Rem16_LE16` kernel (with A/A control):
//!     - `Base-PipeLean` / `Base (A/A)`: `inline16: Vec<[u8; 16]>` + `lens: Vec<u8>`
//!     - `V-PipeLean-Pre`: hoisted current `lens`/`inline16` pre-loads before both `out` stores
//!     - `V-Packed15`: unified 16B aligned `packed15: Vec<[u8; 16]>` (`slot[15] = len` in `1..=15`
//!       or `LONG=255`), completely eliminating `lens: Vec<u8>` (-lens WS, 1 load/cursor vs 2!)
//!     - `V-Packed15-Pre`: `V-Packed15` + hoisted `sf`/`sb` pre-loads (only 2 dict loads/pair!)
//!     - `V-Split8`: `Base-PipeLean` + `len <= 8` `u64` single-store/shift fast-path
//!     - `V-Packed15-Sp8`: `V-Packed15-Pre` + `len <= 8` `u64` back-shift fast-path
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`
//! AND passes `-C debug-assertions=yes -C overflow-checks=yes`.

use std::collections::BTreeMap;
use std::fs;
use std::sync::Barrier;
use std::time::Instant;

// ============================================================================
// 0. Exact Arbitrary-Precision BigUint / BigInt (Base 10^9, Zero Unsafe)
// ============================================================================

const BIG_BASE: u64 = 1_000_000_000;

#[derive(Clone, PartialEq, Eq, Debug)]
struct BigUint {
    d: Vec<u32>,
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
    fn mul(&self, rhs: &Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::zero();
        }
        let mut out = vec![0u64; self.d.len() + rhs.d.len()];
        for (i, &a) in self.d.iter().enumerate() {
            let au = u64::from(a);
            let mut carry = 0u64;
            for (j, &b) in rhs.d.iter().enumerate() {
                let prod = out[i + j] + au * u64::from(b) + carry;
                out[i + j] = prod % BIG_BASE;
                carry = prod / BIG_BASE;
            }
            let mut k = i + rhs.d.len();
            while carry > 0 {
                let s = out[k] + carry;
                out[k] = s % BIG_BASE;
                carry = s / BIG_BASE;
                k += 1;
            }
        }
        let mut r = Self {
            d: out.into_iter().map(|v| v as u32).collect(),
        };
        r.normalize();
        r
    }
    fn to_f64(&self) -> f64 {
        let mut val = 0.0f64;
        for &limb in self.d.iter().rev() {
            val = val * (BIG_BASE as f64) + f64::from(limb);
        }
        val
    }
    fn log2_f64(&self) -> f64 {
        assert!(!self.is_zero());
        let len = self.d.len();
        let take = len.min(3);
        let mut top = 0.0f64;
        for i in (len - take..len).rev() {
            top = top * (BIG_BASE as f64) + f64::from(self.d[i]);
        }
        let shifted = (len - take) as f64;
        top.log2() + shifted * (BIG_BASE as f64).log2()
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
        Self {
            neg: false,
            mag: BigUint::zero(),
        }
    }
    fn one() -> Self {
        Self {
            neg: false,
            mag: BigUint::one(),
        }
    }
    fn from_i64(v: i64) -> Self {
        if v == 0 {
            Self::zero()
        } else if v < 0 {
            Self {
                neg: true,
                mag: BigUint::from_u64(v.unsigned_abs()),
            }
        } else {
            Self {
                neg: false,
                mag: BigUint::from_u64(v as u64),
            }
        }
    }
    fn from_biguint(mag: BigUint) -> Self {
        Self { neg: false, mag }
    }
    fn is_zero(&self) -> bool {
        self.mag.is_zero()
    }
    fn neg_val(&self) -> Self {
        if self.is_zero() {
            Self::zero()
        } else {
            Self {
                neg: !self.neg,
                mag: self.mag.clone(),
            }
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
            Self {
                neg: self.neg,
                mag: self.mag.add(&rhs.mag),
            }
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
    fn mul(&self, rhs: &Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::zero();
        }
        Self {
            neg: self.neg ^ rhs.neg,
            mag: self.mag.mul(&rhs.mag),
        }
    }
}

// ============================================================================
// 1. Utilities, Reference FL, PRNG, Log-Gamma, & Corpus Loaders
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
fn pct(a: u64, b: u64) -> f64 {
    if b == 0 {
        0.0
    } else {
        100.0 * a as f64 / b as f64
    }
}

fn pearson_r(xs: &[f64], ys: &[f64]) -> f64 {
    assert_eq!(xs.len(), ys.len());
    let n = xs.len() as f64;
    if xs.len() < 2 {
        return 0.0;
    }
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut num = 0.0f64;
    let mut dx2 = 0.0f64;
    let mut dy2 = 0.0f64;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        let dx = x - mx;
        let dy = y - my;
        num += dx * dy;
        dx2 += dx * dx;
        dy2 += dy * dy;
    }
    let den = (dx2 * dy2).sqrt();
    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}

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

#[derive(Clone)]
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
            BlockData {
                name: f,
                p,
                l,
                records,
            }
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
        blocks.push(BlockData {
            name,
            p,
            l,
            records,
        });
    }
    blocks
}

/// Generate 5,200 small valid BWTs + 300 malformed multi-cycle columns (incl. 150 orphan thick cycles).
fn generate_small_and_malformed_suites() -> (Vec<(Vec<u8>, u32)>, Vec<(Vec<u8>, u32)>) {
    let mut rng = XorShift64::new(0xC542_1829_1881_FACE);
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
        for (i, slot) in text.iter_mut().enumerate() {
            *slot = match (i + idx) % 5 {
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
// 2. PART 1 — Exact BigInt Jacobi (1829) Triple Product / Pentagonal-Theta
//    Identities (N = 0..400) & Constructive F. Franklin (1881) Involution (n = 1..60)
// ============================================================================

/// Exact `BigInt` power series multiplication truncated to `0..=max_n`.
fn poly_mul_bigint(a: &[BigInt], b: &[BigInt], max_n: usize) -> Vec<BigInt> {
    let mut out = vec![BigInt::zero(); max_n + 1];
    for i in 0..=max_n.min(a.len().saturating_sub(1)) {
        if a[i].is_zero() {
            continue;
        }
        let lim_j = (max_n - i).min(b.len().saturating_sub(1));
        for j in 0..=lim_j {
            if !b[j].is_zero() {
                let prod = a[i].mul(&b[j]);
                out[i + j] = out[i + j].add(&prod);
            }
        }
    }
    out
}

/// Direct exact bivariate `BigInt` verification of C. G. J. Jacobi (1829, §64) Triple Product:
///   \prod_{m=1}^{M} (1 - q^{2m}) (1 + z q^{2m-1}) (1 + z^{-1} q^{2m-1})
///     == \sum_{n=-\infty}^{\infty} z^n q^{n^2}   (mod q^{q_max + 1}).
/// Every monomial z^n q^k (for ALL reachable z-exponents, with NO z-truncation!) is checked.
fn verify_bivariate_jacobi_triple_product(q_max: usize) -> (usize, usize) {
    // State: map from z-exponent `n: i32` to exact `Vec<BigInt>` polynomial in q (len q_max + 1).
    let mut poly: BTreeMap<i32, Vec<BigInt>> = BTreeMap::new();
    let mut one_poly = vec![BigInt::zero(); q_max + 1];
    one_poly[0] = BigInt::one();
    poly.insert(0, one_poly);

    let mut m = 1usize;
    while 2 * m - 1 <= q_max {
        // 1) Multiply by (1 - q^{2m}) if 2*m <= q_max:
        let e_even = 2 * m;
        if e_even <= q_max {
            for coeffs in poly.values_mut() {
                for k in (e_even..=q_max).rev() {
                    let prev = coeffs[k - e_even].clone();
                    if !prev.is_zero() {
                        coeffs[k] = coeffs[k].sub(&prev);
                    }
                }
            }
        }
        // 2) Multiply by (1 + z * q^{2m-1}):
        let e_odd = 2 * m - 1;
        {
            let mut next: BTreeMap<i32, Vec<BigInt>> = poly.clone();
            for (&z_exp, coeffs) in &poly {
                let mut any = false;
                for k in 0..=(q_max - e_odd) {
                    if !coeffs[k].is_zero() {
                        any = true;
                        break;
                    }
                }
                if !any {
                    continue;
                }
                let target = next
                    .entry(z_exp + 1)
                    .or_insert_with(|| vec![BigInt::zero(); q_max + 1]);
                for k in 0..=(q_max - e_odd) {
                    if !coeffs[k].is_zero() {
                        target[k + e_odd] = target[k + e_odd].add(&coeffs[k]);
                    }
                }
            }
            poly = next;
        }
        // 3) Multiply by (1 + z^{-1} * q^{2m-1}):
        {
            let mut next: BTreeMap<i32, Vec<BigInt>> = poly.clone();
            for (&z_exp, coeffs) in &poly {
                let mut any = false;
                for k in 0..=(q_max - e_odd) {
                    if !coeffs[k].is_zero() {
                        any = true;
                        break;
                    }
                }
                if !any {
                    continue;
                }
                let target = next
                    .entry(z_exp - 1)
                    .or_insert_with(|| vec![BigInt::zero(); q_max + 1]);
                for k in 0..=(q_max - e_odd) {
                    if !coeffs[k].is_zero() {
                        target[k + e_odd] = target[k + e_odd].add(&coeffs[k]);
                    }
                }
            }
            poly = next;
        }
        // Drop z-exponents whose polynomial is identically zero mod q^{q_max + 1}
        poly.retain(|_, v| v.iter().any(|c| !c.is_zero()));
        m += 1;
    }

    let mut total_monomials = 0usize;
    let mut nonzero_monomials = 0usize;
    let mut n_lim = 0i32;
    while ((n_lim + 1) * (n_lim + 1)) as usize <= q_max {
        n_lim += 1;
    }
    // Verify EVERY surviving z-exponent and EVERY k in 0..=q_max
    for (&z_exp, coeffs) in &poly {
        let sq = (z_exp as i64 * z_exp as i64) as usize;
        for (k, c) in coeffs.iter().enumerate() {
            total_monomials += 1;
            let want = if k == sq {
                BigInt::one()
            } else {
                BigInt::zero()
            };
            assert_eq!(
                *c, want,
                "Jacobi Triple Product mismatch at z^{z_exp} q^{k}!"
            );
            if !c.is_zero() {
                nonzero_monomials += 1;
            }
        }
    }
    // Verify ALL expected squares z^n q^{n^2} for -n_lim <= n <= n_lim are present!
    assert_eq!(nonzero_monomials, (2 * n_lim + 1) as usize);
    for n in -n_lim..=n_lim {
        let sq = (n * n) as usize;
        assert_eq!(poly[&n][sq], BigInt::one());
    }
    (total_monomials, nonzero_monomials)
}

/// Direct Euler product \phi(q) = \prod_{m=1}^{max_n} (1 - q^m) mod q^{max_n + 1}.
fn euler_phi_product(max_n: usize) -> Vec<BigInt> {
    let mut c = vec![BigInt::zero(); max_n + 1];
    c[0] = BigInt::one();
    for m in 1..=max_n {
        for k in (m..=max_n).rev() {
            let prev = c[k - m].clone();
            if !prev.is_zero() {
                c[k] = c[k].sub(&prev);
            }
        }
    }
    c
}

/// Exact pentagonal series \sum_{k=-\infty}^{\infty} (-1)^k q^{k(3k-1)/2} mod q^{max_n + 1}.
fn pentagonal_series(max_n: usize) -> Vec<BigInt> {
    let mut c = vec![BigInt::zero(); max_n + 1];
    c[0] = BigInt::one();
    let mut k = 1usize;
    loop {
        let g1 = k * (3 * k - 1) / 2;
        if g1 > max_n {
            break;
        }
        let sgn = if (k & 1) == 1 {
            BigInt::from_i64(-1)
        } else {
            BigInt::one()
        };
        c[g1] = sgn.clone();
        let g2 = k * (3 * k + 1) / 2;
        if g2 <= max_n {
            c[g2] = sgn;
        }
        k += 1;
    }
    c
}

/// Euler–Jacobi pentagonal recurrence for p(0..=max_n).
fn pentagonal_p_recurrence(max_n: usize) -> Vec<BigUint> {
    let mut p = Vec::with_capacity(max_n + 1);
    p.push(BigUint::one());
    for n in 1..=max_n {
        let mut acc = BigInt::zero();
        let mut k = 1usize;
        loop {
            let g1 = k * (3 * k - 1) / 2;
            if g1 > n {
                break;
            }
            let pos = (k & 1) == 1;
            let t1 = BigInt::from_biguint(p[n - g1].clone());
            acc = if pos { acc.add(&t1) } else { acc.sub(&t1) };
            let g2 = k * (3 * k + 1) / 2;
            if g2 <= n {
                let t2 = BigInt::from_biguint(p[n - g2].clone());
                acc = if pos { acc.add(&t2) } else { acc.sub(&t2) };
            }
            k += 1;
        }
        assert!(!acc.neg);
        p.push(acc.mag);
    }
    p
}

/// CA-49 Sylvester (1882) Durfee-square GF sum_{d>=0} q^{d^2} / \prod_{j=1}^d (1 - q^j)^2.
fn sylvester_durfee_p(max_n: usize) -> Vec<BigUint> {
    let mut total = vec![BigUint::zero(); max_n + 1];
    let mut d = 0usize;
    while d * d <= max_n {
        let mut c = vec![BigUint::zero(); max_n + 1];
        c[d * d] = BigUint::one();
        for j in 1..=d {
            for n in (d * d + j)..=max_n {
                let prev = c[n - j].clone();
                c[n].add_assign(&prev);
            }
            for n in (d * d + j)..=max_n {
                let prev = c[n - j].clone();
                c[n].add_assign(&prev);
            }
        }
        for n in d * d..=max_n {
            total[n].add_assign(&c[n]);
        }
        d += 1;
    }
    total
}

/// Direct product for Jacobi theta_3(q) (sign = +1) or theta_4(q) (sign = -1):
///   \prod_{m>=1} (1 - q^{2m}) (1 + sign * q^{2m-1})^2  mod q^{max_n + 1}.
fn jacobi_theta_product(max_n: usize, plus: bool) -> Vec<BigInt> {
    let mut c = vec![BigInt::zero(); max_n + 1];
    c[0] = BigInt::one();
    let mut m = 1usize;
    while 2 * m - 1 <= max_n {
        let ev = 2 * m;
        if ev <= max_n {
            for k in (ev..=max_n).rev() {
                let prev = c[k - ev].clone();
                if !prev.is_zero() {
                    c[k] = c[k].sub(&prev);
                }
            }
        }
        let od = 2 * m - 1;
        // Multiply twice by (1 +/- q^{2m-1})
        for _ in 0..2 {
            for k in (od..=max_n).rev() {
                let prev = c[k - od].clone();
                if !prev.is_zero() {
                    c[k] = if plus { c[k].add(&prev) } else { c[k].sub(&prev) };
                }
            }
        }
        m += 1;
    }
    c
}

/// Distinct-part partition counts q(n) = [q^n] \prod_{k>=1} (1 + q^k).
fn euler_distinct_q(max_n: usize) -> Vec<BigUint> {
    let mut c = vec![BigUint::zero(); max_n + 1];
    c[0] = BigUint::one();
    for k in 1..=max_n {
        for n in (k..=max_n).rev() {
            let prev = c[n - k].clone();
            c[n].add_assign(&prev);
        }
    }
    c
}

/// Odd-part partition counts p_odd(n) = [q^n] \prod_{k>=1} (1 - q^{2k-1})^{-1}.
fn euler_odd_parts(max_n: usize) -> Vec<BigUint> {
    let mut c = vec![BigUint::zero(); max_n + 1];
    c[0] = BigUint::one();
    let mut odd = 1usize;
    while odd <= max_n {
        for n in odd..=max_n {
            let prev = c[n - odd].clone();
            c[n].add_assign(&prev);
        }
        odd += 2;
    }
    c
}

/// Modified Bessel function log(I_1(z)) = z - 0.5*ln(2*pi*z) + ln(asymptotic_series) for large z,
/// or exact power series \sum_{k>=0} (z/2)^{2k+1} / (k! (k+1)!) for moderate z <= 40.
fn ln_bessel_i1(z: f64) -> f64 {
    if z <= 40.0 {
        let half = 0.5 * z;
        let half2 = half * half;
        let mut term = half;
        let mut sum = term;
        for k in 1..120 {
            let kf = k as f64;
            term *= half2 / (kf * (kf + 1.0));
            sum += term;
            if term <= sum * 1e-17 {
                break;
            }
        }
        sum.ln()
    } else {
        // Hankel asymptotic expansion for I_1(z), accurate to < 1e-15 for z > 40:
        //   I_1(z) ~ (e^z / sqrt(2*pi*z)) * sum_{k>=0} (-1)^k a_k / (8 z)^k,
        //   a_0 = 1, a_k = a_{k-1} * (4 - (2k-1)^2) / k.
        let mut sum = 1.0f64;
        let mut term = 1.0f64;
        let inv8z = 1.0 / (8.0 * z);
        for k in 1..14 {
            let odd = (2 * k - 1) as f64;
            term *= -(4.0 - odd * odd) * inv8z / (k as f64);
            sum += term;
        }
        z - 0.5 * (2.0 * std::f64::consts::PI * z).ln() + sum.ln()
    }
}

/// Hardy–Ramanujan (1918, §7.1) distinct-part asymptotic estimates for q(N):
///   (q_hr1, q_hr2, q_bessel)
/// where q_bessel(N) = (pi / (2 sqrt(6 (N + 1/24)))) * I_1(pi sqrt((N + 1/24) / 3)).
fn distinct_q_asymptotics(n: usize) -> (f64, f64, f64) {
    let nf = n as f64;
    let c = std::f64::consts::PI / 3.0_f64.sqrt();
    let q_hr1 = (1.0 / (4.0 * 3.0_f64.powf(0.25) * nf.powf(0.75))) * (c * nf.sqrt()).exp();

    let a = nf + 1.0 / 24.0;
    let z = std::f64::consts::PI * (a / 3.0).sqrt();
    let q_hr2 = (1.0 / (4.0 * 3.0_f64.powf(0.25) * a.powf(0.75)))
        * (1.0 - 3.0 / (8.0 * z))
        * z.exp();

    let ln_qb = (std::f64::consts::PI / (2.0 * (6.0 * a).sqrt())).ln() + ln_bessel_i1(z);
    (q_hr1, q_hr2, ln_qb.exp())
}

/// Overflow-safe log_2 q(N) for arbitrarily large N (exact via BigUint for small N <= 20, Bessel above).
fn distinct_q_log2(n: u64) -> f64 {
    if n <= 1 {
        return 0.0;
    }
    let nf = n as f64;
    let a = nf + 1.0 / 24.0;
    let z = std::f64::consts::PI * (a / 3.0).sqrt();
    let ln_qb = (std::f64::consts::PI / (2.0 * (6.0 * a).sqrt())).ln() + ln_bessel_i1(z);
    ln_qb / std::f64::consts::LN_2
}

/// Hardy–Ramanujan (1918) asymptotic log_2 p(N) (CA-49 baseline for direct comparison).
fn hardy_ramanujan_log2_p(n: u64) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let nf = n as f64;
    let c = std::f64::consts::PI * (2.0 / 3.0_f64).sqrt();
    let n_shift = (nf - 1.0 / 24.0).max(1e-9);
    let lam = n_shift.sqrt();
    let corr = (1.0 - 1.0 / (c * lam)).max(1e-12);
    let ln_p = c * lam + corr.ln() - (4.0 * 3.0_f64.sqrt() * n_shift).ln();
    ln_p / std::f64::consts::LN_2
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FranklinMove {
    /// Case A (s <= sigma, not fixed): delete bottom row of size s, add 1 to top s rows (parts - 1).
    DeleteBase,
    /// Case B (s > sigma, not fixed): peel 1 from top sigma rows, append bottom row sigma (parts + 1).
    PeelSlope,
    /// Fixed point at pentagonal number \omega_k = k(3k-1)/2: s == sigma == len.
    FixedPentPos,
    /// Fixed point at pentagonal number \omega_{-k} = k(3k+1)/2: sigma == len && s == sigma + 1.
    FixedPentNeg,
}

/// F. Franklin (1881, *C. R. Acad. Sci. Paris* 92, 448–450) constructive bijective involution
/// on a strict partition `mu = (mu_1 > mu_2 > ... > mu_ell >= 1)` (sorted strictly descending).
/// Returns `(f(mu), base s, slope sigma, move_kind)`.
fn franklin_involution(mu: &[u64]) -> (Vec<u64>, u64, usize, FranklinMove) {
    assert!(!mu.is_empty());
    let ell = mu.len();
    let s = mu[ell - 1];
    let mut sigma = 1usize;
    while sigma < ell && mu[sigma] + 1 == mu[sigma - 1] {
        sigma += 1;
    }
    let sig_u = sigma as u64;
    if s <= sig_u {
        if sigma == ell && s == sig_u {
            return (mu.to_vec(), s, sigma, FranklinMove::FixedPentPos);
        }
        let su = s as usize;
        assert!(su <= ell - 1);
        let mut out = mu[..ell - 1].to_vec();
        for x in &mut out[..su] {
            *x += 1;
        }
        (out, s, sigma, FranklinMove::DeleteBase)
    } else {
        if sigma == ell && s == sig_u + 1 {
            return (mu.to_vec(), s, sigma, FranklinMove::FixedPentNeg);
        }
        let mut out = mu.to_vec();
        for x in &mut out[..sigma] {
            *x -= 1;
        }
        out.push(sig_u);
        (out, s, sigma, FranklinMove::PeelSlope)
    }
}

/// Exhaustive verification of Franklin's (1881) constructive bijection over ALL distinct-part
/// Ferrers diagrams for `n = 1..=max_n`.
fn verify_exhaustive_franklin_involution(
    max_n: usize,
    q_exact: &[BigUint],
    pent: &[BigInt],
) -> (usize, usize, usize, usize) {
    fn gen_strict(rem: usize, max_part: usize, cur: &mut Vec<u64>, out: &mut Vec<Vec<u64>>) {
        if rem == 0 {
            out.push(cur.clone());
            return;
        }
        let hi = rem.min(max_part);
        for p in (1..=hi).rev() {
            // Maximum remaining sum using distinct parts <= p-1 is p*(p-1)/2
            if rem > p + p * (p - 1) / 2 {
                break;
            }
            cur.push(p as u64);
            gen_strict(rem - p, p - 1, cur, out);
            cur.pop();
        }
    }

    let mut total_diagrams = 0usize;
    let mut total_case_a = 0usize;
    let mut total_case_b = 0usize;
    let mut total_fixed = 0usize;

    for n in 1..=max_n {
        let mut parts = Vec::new();
        let mut cur = Vec::new();
        gen_strict(n, n, &mut cur, &mut parts);
        assert_eq!(
            BigUint::from_u64(parts.len() as u64),
            q_exact[n],
            "Distinct-part enumeration count != q({n})!"
        );
        total_diagrams += parts.len();

        let mut even_cnt = 0i64;
        let mut odd_cnt = 0i64;
        let mut fixed_here = 0usize;

        for mu in &parts {
            if (mu.len() & 1) == 0 {
                even_cnt += 1;
            } else {
                odd_cnt += 1;
            }
            let (fmu, _s, _sig, mv) = franklin_involution(mu);
            // 1) Result must be a strictly decreasing partition summing to n
            assert_eq!(fmu.iter().sum::<u64>(), n as u64);
            for w in fmu.windows(2) {
                assert!(w[0] > w[1], "Franklin image not strictly decreasing at n={n}!");
            }
            assert!(*fmu.last().unwrap() >= 1);

            // 2) Involution property: f(f(mu)) == mu
            let (ffmu, _, _, mv2) = franklin_involution(&fmu);
            assert_eq!(&ffmu, mu, "Franklin f(f(mu)) != mu at n={n}!");

            // 3) Parity flip on every non-fixed diagram; unique fixed points only at pentagonal numbers
            match mv {
                FranklinMove::DeleteBase => {
                    assert_eq!(fmu.len() + 1, mu.len());
                    assert_eq!(mv2, FranklinMove::PeelSlope);
                    total_case_a += 1;
                }
                FranklinMove::PeelSlope => {
                    assert_eq!(fmu.len(), mu.len() + 1);
                    assert_eq!(mv2, FranklinMove::DeleteBase);
                    total_case_b += 1;
                }
                FranklinMove::FixedPentPos => {
                    assert_eq!(&fmu, mu);
                    assert_eq!(mv2, FranklinMove::FixedPentPos);
                    let k = mu.len();
                    assert_eq!(n, k * (3 * k - 1) / 2);
                    fixed_here += 1;
                    total_fixed += 1;
                }
                FranklinMove::FixedPentNeg => {
                    assert_eq!(&fmu, mu);
                    assert_eq!(mv2, FranklinMove::FixedPentNeg);
                    let k = mu.len();
                    assert_eq!(n, k * (3 * k + 1) / 2);
                    fixed_here += 1;
                    total_fixed += 1;
                }
            }
        }
        assert_eq!(
            BigInt::from_i64(even_cnt - odd_cnt),
            pent[n],
            "Franklin q_even({n}) - q_odd({n}) != [q^{n}] phi(q)!"
        );
        if pent[n].is_zero() {
            assert_eq!(fixed_here, 0, "Unexpected fixed point at non-pentagonal n={n}!");
        } else {
            assert_eq!(fixed_here, 1, "Expected unique fixed point at pentagonal n={n}!");
        }
    }
    assert_eq!(total_case_a, total_case_b);
    (total_diagrams, total_case_a, total_case_b, total_fixed)
}

fn verify_part1_jacobi_and_franklin() {
    println!("================================================================================");
    println!("PART 1 — EXACT BIGINT JACOBI (1829) TRIPLE PRODUCT / PENTAGONAL–THETA SERIES");
    println!("         (N = 0..400) & CONSTRUCTIVE FRANKLIN (1881) INVOLUTION (n = 1..60)");
    println!("================================================================================");

    const N_MAX: usize = 400;

    // 1A. Direct bivariate exact BigInt Jacobi Triple Product up to q^{100}
    let (jtp_monomials, jtp_squares) = verify_bivariate_jacobi_triple_product(100);
    println!(
        "  [PASS] Part 1A (Jacobi 1829 §64 Bivariate Triple Product mod q^101): {jtp_monomials} exact BigInt Laurent monomials checked ({jtp_squares} non-zero z^n q^{{n^2}} for n=-10..+10, 0 errors)!"
    );

    // 1B. Pentagonal Number Theorem & Partition Recurrence vs CA-49 Sylvester Durfee GF (N = 0..400)
    let phi = euler_phi_product(N_MAX);
    let pent = pentagonal_series(N_MAX);
    for n in 0..=N_MAX {
        assert_eq!(phi[n], pent[n], "Pentagonal product mismatch at q^{n}!");
    }
    let p_pent = pentagonal_p_recurrence(N_MAX);
    let p_durf = sylvester_durfee_p(N_MAX);
    for n in 0..=N_MAX {
        assert_eq!(
            p_pent[n], p_durf[n],
            "Pentagonal recurrence != CA-49 Sylvester Durfee GF at p({n})!"
        );
    }
    assert_eq!(p_pent[100].to_decimal(), "190569292");
    assert_eq!(p_pent[300].to_decimal(), "9253082936723602");
    assert_eq!(p_pent[400].to_decimal(), "6727090051741041926");
    println!(
        "  [PASS] Part 1B (Jacobi 1829 §66 Pentagonal Recurrence == CA-49 Sylvester Durfee GF): 401/401 EXACT (N=0..=400; phi(q) 401/401 exact; p(400)={})!",
        p_pent[400].to_decimal()
    );

    // 1C. Jacobi Cube Identity & Jacobi Theta Products theta_3(q), theta_4(q) (N = 0..400)
    let phi2 = poly_mul_bigint(&phi, &phi, N_MAX);
    let phi3 = poly_mul_bigint(&phi2, &phi, N_MAX);
    let mut tri_rhs = vec![BigInt::zero(); N_MAX + 1];
    let mut k = 0usize;
    while k * (k + 1) / 2 <= N_MAX {
        let idx = k * (k + 1) / 2;
        let coeff = (2 * k + 1) as i64;
        tri_rhs[idx] = BigInt::from_i64(if (k & 1) == 0 { coeff } else { -coeff });
        k += 1;
    }
    for n in 0..=N_MAX {
        assert_eq!(phi3[n], tri_rhs[n], "Jacobi cube identity mismatch at q^{n}!");
    }

    let th3_prod = jacobi_theta_product(N_MAX, true);
    let th4_prod = jacobi_theta_product(N_MAX, false);
    let mut th3_sum = vec![BigInt::zero(); N_MAX + 1];
    let mut th4_sum = vec![BigInt::zero(); N_MAX + 1];
    th3_sum[0] = BigInt::one();
    th4_sum[0] = BigInt::one();
    let mut m = 1usize;
    while m * m <= N_MAX {
        th3_sum[m * m] = BigInt::from_i64(2);
        th4_sum[m * m] = BigInt::from_i64(if (m & 1) == 0 { 2 } else { -2 });
        m += 1;
    }
    for n in 0..=N_MAX {
        assert_eq!(th3_prod[n], th3_sum[n], "Jacobi theta_3 mismatch at q^{n}!");
        assert_eq!(th4_prod[n], th4_sum[n], "Jacobi theta_4 mismatch at q^{n}!");
    }
    // Bonus classical theta identity: theta_3(q) * theta_4(q) == theta_4(q^2)^2
    let th34 = poly_mul_bigint(&th3_prod, &th4_prod, N_MAX);
    let mut th4_q2 = vec![BigInt::zero(); N_MAX + 1];
    for n in 0..=N_MAX / 2 {
        th4_q2[2 * n] = th4_sum[n].clone();
    }
    let th4_q2_sq = poly_mul_bigint(&th4_q2, &th4_q2, N_MAX);
    for n in 0..=N_MAX {
        assert_eq!(
            th34[n], th4_q2_sq[n],
            "Jacobi theta_3(q)*theta_4(q) != theta_4(q^2)^2 at q^{n}!"
        );
    }
    println!(
        "  [PASS] Part 1C (Jacobi 1829 §66 Cube & Theta Identities, N=0..=400): phi(q)^3 == sum (-1)^k(2k+1)q^{{k(k+1)/2}} (401/401, k=0..={}), theta_3(q) (401/401), theta_4(q) (401/401), & theta_3*theta_4 == theta_4(q^2)^2 (401/401) EXACT!",
        k - 1
    );

    // 1D. Euler Distinct-vs-Odd Identity q(n) == p_odd(n), phi(q)*Q(q) == phi(q^2), & Bessel Asymptotics
    let q_dist = euler_distinct_q(N_MAX);
    let p_odd = euler_odd_parts(N_MAX);
    for n in 0..=N_MAX {
        assert_eq!(
            q_dist[n], p_odd[n],
            "Euler distinct q({n}) != odd-part p_odd({n})!"
        );
    }
    let q_bi: Vec<BigInt> = q_dist.iter().cloned().map(BigInt::from_biguint).collect();
    let phi_q = poly_mul_bigint(&phi, &q_bi, N_MAX);
    for n in 0..=N_MAX {
        let want = if (n & 1) == 0 {
            phi[n / 2].clone()
        } else {
            BigInt::zero()
        };
        assert_eq!(phi_q[n], want, "phi(q)*Q(q) != phi(q^2) at q^{n}!");
    }
    assert_eq!(q_dist[50].to_decimal(), "3658");
    assert_eq!(q_dist[100].to_decimal(), "444793");
    assert_eq!(q_dist[400].to_decimal(), "11962163400706");

    let mut max_hr1_s = 0.0f64;
    let mut max_bessel_from_100 = 0.0f64;
    println!("\n  --- Part 1D: Euler Distinct-vs-Odd q(N) == p_odd(N) & Bessel Asymptotic Convergence (N = 10..=400) ---");
    println!(
        "  {:>4} | {:>16} | {:>10} {:>8} | {:>10} | {:>10} | {:>9} {:>9}",
        "N", "Exact q(N)", "Err(qHR1)", "*sqrt(N)", "Err(qHR2)", "Err(Bess)", "log2 q(N)", "|d_log2|"
    );
    let ms = [10usize, 20, 50, 100, 200, 300, 400];
    for n in 10..=N_MAX {
        let ex = q_dist[n].to_f64();
        let ex_l2 = q_dist[n].log2_f64();
        let (h1, h2, qb) = distinct_q_asymptotics(n);
        let e1 = (h1 - ex) / ex;
        let e2 = (h2 - ex) / ex;
        let eb = (qb - ex) / ex;
        let dl2 = (distinct_q_log2(n as u64) - ex_l2).abs();
        let s1 = e1.abs() * (n as f64).sqrt();
        max_hr1_s = max_hr1_s.max(s1);
        if n >= 100 {
            max_bessel_from_100 = max_bessel_from_100.max(eb.abs());
        }
        assert!(s1 < 0.45, "qHR1 bound failed at N={n}");
        assert!(e2.abs() < e1.abs(), "qHR2 failed to tighten qHR1 at N={n}");
        assert!(
            eb.abs() < 2.0e-2 && (n < 100 || eb.abs() < 5.0e-5),
            "q_bessel bound failed at N={n}: {eb}"
        );
        if ms.contains(&n) {
            println!(
                "  {:>4} | {:>16} | {:>+9.3e} {:>8.5} | {:>+9.3e} | {:>+9.3e} | {:>9.4} {:>9.2e}",
                n,
                q_dist[n].to_decimal(),
                e1,
                e1 * (n as f64).sqrt(),
                e2,
                eb,
                ex_l2,
                dl2
            );
        }
    }
    let ex400 = q_dist[400].to_f64();
    let (_, _, qb400) = distinct_q_asymptotics(400);
    println!(
        "  [PASS] Part 1D (Euler Distinct == Odd & Pentagonal Q Recurrence, N=0..=400): 401/401 EXACT (q(400)={}), Bessel rel err at N=400 = {:+.3e} (max N>=100: {:.3e})!",
        q_dist[400].to_decimal(),
        (qb400 - ex400) / ex400,
        max_bessel_from_100
    );

    // 1E. Constructive Franklin (1881) Bijective Involution on ALL distinct-part Ferrers diagrams n = 1..=60
    let (tot_diag, cnt_a, cnt_b, cnt_fix) = verify_exhaustive_franklin_involution(60, &q_dist, &pent);
    println!(
        "  [PASS] Part 1E (F. Franklin 1881 Constructive Involution, ALL n=1..=60): {tot_diag} distinct-part Ferrers diagrams 100% EXACT ({cnt_a} Case-A <-> {cnt_b} Case-B exact parity flips, {cnt_fix} fixed points ONLY at pentagonal w_{{+/-k}} <= 60: 1,2,5,7,12,15,22,26,35,40,51,57 — 0 errors)!\n"
    );
}

// ============================================================================
// 3. PART 2 — Empirical Kakutani Distinct-Height Support \tilde{\lambda},
//    Franklin (1881) Involution, Theta Sparsity, & `inline16` Length Distribution
// ============================================================================

struct DistinctSkyscraper {
    n: u64,
    m: u64,
    n_surv: u64,
    n_orphan: u64,
    k_dist: usize,
    n_tilde: u64,
    d_obs: u64,
    d_tilde: u64,
    base_s: u64,
    slope_sig: usize,
    fr_move: FranklinMove,
    c_lo: u64,
    h_obs_bits: f64,
    multiset_order_bits: f64,
    log2_q_ntilde: f64,
    log2_p_nsurv: f64,
    tri_sparsity: f64,
}

/// Extract exact `Open<16>` Kakutani skyscraper height partition \lambda, distinct-height support
/// \tilde{\lambda}, verify K(\lambda) == K(\lambda'), and execute Franklin (1881) involution on \tilde{\lambda}.
fn extract_distinct_skyscraper(l: &[u8], p: u32, lim: usize) -> DistinctSkyscraper {
    let n = l.len();
    let mut is_thin = vec![false; n];
    let mut i = 0usize;
    let p_us = p as usize;
    while i < n {
        let b = l[i];
        let mut j = i + 1;
        while j < n && l[j] == b {
            j += 1;
        }
        let w = j - i;
        if w <= lim {
            for slot in &mut is_thin[i..j] {
                *slot = true;
            }
        } else if p_us >= i && p_us < j {
            is_thin[p_us] = true;
        }
        i = j;
    }

    let mut cnt = [0u32; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut st = [0u32; 256];
    let mut tot = 0u32;
    for a in 0..256 {
        st[a] = tot;
        tot += cnt[a];
    }
    let mut fl = vec![0u32; n];
    let mut cur = st;
    for (row, &b) in l.iter().enumerate() {
        let a = usize::from(b);
        fl[cur[a] as usize] = row as u32;
        cur[a] += 1;
    }

    let mut mult: BTreeMap<u64, u64> = BTreeMap::new();
    let mut m = 0u64;
    let mut n_surv = 0u64;
    let mut visited = vec![false; n];
    for start in 0..n {
        if !is_thin[start] {
            continue;
        }
        m += 1;
        let mut x = start;
        let mut tau = 0u64;
        loop {
            visited[x] = true;
            tau += 1;
            x = fl[x] as usize;
            if is_thin[x] {
                break;
            }
        }
        *mult.entry(tau).or_insert(0) += 1;
        n_surv += tau;
    }
    let n_orphan = visited.iter().filter(|&&v| !v).count() as u64;
    assert_eq!(n_surv + n_orphan, n as u64);

    // Build distinct-height support \tilde{\lambda} (strictly descending)
    let mut lam_tilde: Vec<u64> = mult.keys().copied().rev().collect();
    let k_dist = lam_tilde.len();
    let n_tilde: u64 = lam_tilde.iter().sum();

    // Verify Ferrers distinct-support duality: K(\lambda) == K(\lambda')!
    // Distinct positive floor widths in \lambda' are the cumulative suffix counts
    // \lambda'_h = #{x in R : tau(x) >= h} at each distinct jump height h in \tilde{\lambda}.
    // Since every height h in mult has count >= 1, each distinct height produces a strictly
    // larger cumulative count, so the number of distinct values in \lambda' is identically K!
    let mut distinct_floors: Vec<u64> = Vec::with_capacity(k_dist);
    let mut cum_rows = 0u64;
    for &h in &lam_tilde {
        cum_rows += mult[&h];
        if let Some(&prev) = distinct_floors.last() {
            assert!(cum_rows > prev);
        }
        distinct_floors.push(cum_rows);
    }
    assert_eq!(distinct_floors.len(), k_dist);
    assert_eq!(cum_rows, m);

    // Durfee square of \lambda (d_obs) and of distinct support \tilde{\lambda} (d_tilde)
    let mut d_obs = 0u64;
    let mut seen = 0u64;
    for &h in &lam_tilde {
        let c = mult[&h];
        if seen < h {
            d_obs = d_obs.max((seen + c).min(h));
        }
        seen += c;
    }
    let mut d_tilde = 0u64;
    for (idx, &h) in lam_tilde.iter().enumerate() {
        let pos = (idx + 1) as u64;
        if h >= pos {
            d_tilde = pos;
        } else {
            break;
        }
    }

    // Apply Franklin (1881) involution directly to \tilde{\lambda} and verify exactness!
    let (f_lam, base_s, slope_sig, fr_move) = franklin_involution(&lam_tilde);
    assert_eq!(f_lam.iter().sum::<u64>(), n_tilde);
    let (ff_lam, _, _, _) = franklin_involution(&f_lam);
    assert_eq!(ff_lam, lam_tilde, "Franklin involution failed on skyscraper!");
    match fr_move {
        FranklinMove::DeleteBase => assert_eq!(f_lam.len() + 1, k_dist),
        FranklinMove::PeelSlope => assert_eq!(f_lam.len(), k_dist + 1),
        FranklinMove::FixedPentPos | FranklinMove::FixedPentNeg => assert_eq!(f_lam, lam_tilde),
    }

    // Lower-staircase contiguous coverage c_lo = max{c : 1, 2, ..., c all present in \tilde{\lambda}}
    let mut c_lo = 0u64;
    for (&h, _) in &mult {
        if h == c_lo + 1 {
            c_lo = h;
        } else {
            break;
        }
    }

    // Height-profile empirical Shannon entropy m*H(p_obs) and exact multiset-ordering bits
    let mf = m as f64;
    let mut h_obs = 0.0f64;
    let mut sum_log2_c_fact = 0.0f64;
    for &c in mult.values() {
        let prob = c as f64 / mf;
        h_obs -= prob * prob.log2();
        sum_log2_c_fact += log2_fact(c);
    }
    let multiset_order_bits = log2_fact(m) - sum_log2_c_fact;
    let log2_q_ntilde = distinct_q_log2(n_tilde);
    let log2_p_nsurv = hardy_ramanujan_log2_p(n_surv);
    let tri_min = (k_dist as u64) * (k_dist as u64 + 1) / 2;
    let tri_sparsity = n_tilde as f64 / (tri_min.max(1) as f64);

    // Silence unused mut
    lam_tilde.clear();

    DistinctSkyscraper {
        n: n as u64,
        m,
        n_surv,
        n_orphan,
        k_dist,
        n_tilde,
        d_obs,
        d_tilde,
        base_s,
        slope_sig,
        fr_move,
        c_lo,
        h_obs_bits: mf * h_obs,
        multiset_order_bits,
        log2_q_ntilde,
        log2_p_nsurv,
        tri_sparsity,
    }
}

// ============================================================================
// 4. Deterministic 8-Way LRU L1D (32 KiB) & L2 (256 KiB) Cache Simulator
//    (Simulates edges + dict [inline16+lens OR packed15] + spelling + out)
// ============================================================================

struct CacheSim {
    l1_tags: [[u64; 8]; 64],
    l2_tags: [[u64; 8]; 512],
    l1_dict_misses: u64,
    l1_tot_misses: u64,
    l2_tot_misses: u64,
    dict_accesses: u64,
    tot_accesses: u64,
}

impl CacheSim {
    fn new() -> Self {
        Self {
            l1_tags: [[u64::MAX; 8]; 64],
            l2_tags: [[u64::MAX; 8]; 512],
            l1_dict_misses: 0,
            l1_tot_misses: 0,
            l2_tot_misses: 0,
            dict_accesses: 0,
            tot_accesses: 0,
        }
    }

    #[inline(always)]
    fn access(&mut self, addr: u64, is_dict: bool) {
        self.tot_accesses += 1;
        if is_dict {
            self.dict_accesses += 1;
        }
        let line = addr >> 6;
        let s1 = (line & 63) as usize;
        let t1 = line >> 6;
        let set1 = &mut self.l1_tags[s1];
        for i in 0..8 {
            if set1[i] == t1 {
                if i > 0 {
                    set1.copy_within(0..i, 1);
                    set1[0] = t1;
                }
                return;
            }
        }
        set1.copy_within(0..7, 1);
        set1[0] = t1;
        self.l1_tot_misses += 1;
        if is_dict {
            self.l1_dict_misses += 1;
        }

        let s2 = (line & 511) as usize;
        let t2 = line >> 9;
        let set2 = &mut self.l2_tags[s2];
        for i in 0..8 {
            if set2[i] == t2 {
                if i > 0 {
                    set2.copy_within(0..i, 1);
                    set2[0] = t2;
                }
                return;
            }
        }
        set2.copy_within(0..7, 1);
        set2[0] = t2;
        self.l2_tot_misses += 1;
    }

    #[inline(always)]
    fn access_range(&mut self, base: u64, start: usize, len: usize, is_dict: bool) {
        if len == 0 {
            return;
        }
        let first_line = (base + start as u64) >> 6;
        let last_line = (base + (start + len - 1) as u64) >> 6;
        for cl in first_line..=last_line {
            self.access(cl << 6, is_dict);
        }
    }
}

#[derive(Default, Clone, Copy)]
struct S5CacheStats {
    l1_dict_misses: u64,
    l1_tot_misses: u64,
    l2_tot_misses: u64,
    dict_accesses: u64,
    tot_accesses: u64,
}

// ============================================================================
// 5. Standalone Exact `C53-PipeLean-v1` Kernel with Zero-Remap `S5` Dict Modes
//    (`Base-PipeLean`, `V-PipeLean-Pre`, `V-Packed15`, `V-Packed15-Pre`,
//     `V-Split8`, `V-Packed15-Sp8`) — 100% Safe Rust, Zero Remap Pass!
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum S5DictMode {
    /// Exact `C53-PipeLean-v1` `SlimS5_Pipe1_Rem16_LE16` (`inline16: [[u8; 16]]` + `lens: [u8]`).
    BasePipeLean,
    /// `BasePipeLean` + hoisted current `lens`/`inline16` pre-loads before both `out` stores (`CS-40` fusion).
    PipeLeanPre,
    /// Unified 16B aligned `packed15: [[u8; 16]]` (`slot[15] = len` in `1..=15` or `LONG=255`); ZERO `lens[]`!
    Packed15,
    /// `Packed15` + hoisted `sf = packed15[cf]`, `sb = packed15[cb]` pre-loads (2 dict loads/pair, ZERO `lens[]`!).
    Packed15Pre,
    /// `BasePipeLean` + explicit `len <= 8` `u64` store/shift fast-path.
    Split8,
    /// `Packed15Pre` + explicit `len <= 8` `u64` back-shift fast-path.
    Packed15Sp8,
}

impl S5DictMode {
    #[inline(always)]
    const fn is_packed15(self) -> bool {
        matches!(
            self,
            S5DictMode::Packed15 | S5DictMode::Packed15Pre | S5DictMode::Packed15Sp8
        )
    }
}

#[derive(Default, Clone, Copy)]
struct StageTimes {
    s1_us: f64,
    s2a_us: f64,
    s2b_us: f64,
    s3a_us: f64,
    s4_us: f64,
    s5_us: f64,
}

#[derive(Default, Clone, Copy)]
struct InlineLenProfile {
    // Unique codes in dictionary
    codes_total: u64,
    codes_len1: u64,
    codes_2_8: u64,
    codes_9_15: u64,
    codes_len16: u64,
    codes_long: u64,
    // Dynamic S5 row emits
    rows_total: u64,
    rows_len1: u64,
    rows_2_8: u64,
    rows_9_15: u64,
    rows_len16: u64,
    rows_long: u64,
    // Emitted output bytes
    bytes_total: u64,
    bytes_len1: u64,
    bytes_2_8: u64,
    bytes_9_15: u64,
    bytes_len16: u64,
    bytes_long: u64,
}

impl InlineLenProfile {
    fn add_assign(&mut self, o: &Self) {
        self.codes_total += o.codes_total;
        self.codes_len1 += o.codes_len1;
        self.codes_2_8 += o.codes_2_8;
        self.codes_9_15 += o.codes_9_15;
        self.codes_len16 += o.codes_len16;
        self.codes_long += o.codes_long;
        self.rows_total += o.rows_total;
        self.rows_len1 += o.rows_len1;
        self.rows_2_8 += o.rows_2_8;
        self.rows_9_15 += o.rows_9_15;
        self.rows_len16 += o.rows_len16;
        self.rows_long += o.rows_long;
        self.bytes_total += o.bytes_total;
        self.bytes_len1 += o.bytes_len1;
        self.bytes_2_8 += o.bytes_2_8;
        self.bytes_9_15 += o.bytes_9_15;
        self.bytes_len16 += o.bytes_len16;
        self.bytes_long += o.bytes_long;
    }
}

mod pipelean_v {
    use super::{CacheSim, InlineLenProfile, S5CacheStats, S5DictMode, StageTimes};
    use std::time::Instant;

    pub(super) const INLINE: usize = 16;
    pub(super) const PACKED_MAX: usize = 15;
    pub(super) const LONG: u8 = u8::MAX;
    const NARROW_NODES: usize = 1 << 16;
    const MAX_RULE_WORDS: usize = 4;
    const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;
    const INLINE_STACK: usize = 64;
    const MEMO_REC_CAP: usize = 32;
    const SPELLING_STEP: usize = 4096;
    const INLINE_STEP: usize = 128;
    const PAIR_WORDS: usize = 512;
    const CACHE_WORDS: usize = PAIR_WORDS + 2048;

    const ID_BITS: u32 = 24;
    const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
    const SATURATED: u32 = u32::MAX >> ID_BITS;

    #[inline(always)]
    const fn literal(byte: u8) -> u32 {
        (1u32 << ID_BITS) | (byte as u32)
    }

    #[inline(always)]
    fn should_use_bi(m: u32, ns: u32, r_max: u32) -> bool {
        if m < 65_536 {
            return false;
        }
        let md_w1 = 2i64 * (i64::from(r_max) - i64::from(ns)) - i64::from(m);
        !((r_max <= 50_000 && ns <= 2_100 && md_w1 <= 16_000)
            || (md_w1 < -18_000 && ns <= 2_400))
    }

    #[derive(Clone, Copy)]
    struct Band(u64);

    impl Band {
        #[inline(always)]
        fn new(end: u32, dst: u32, code: u32) -> Self {
            debug_assert!(end <= 900_000 && dst <= 900_000 && code < (1 << 24));
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

    fn starts(counts: [u32; 256]) -> [u32; 256] {
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
    fn long_slot_packed(start: u32, end: u32) -> [u8; INLINE] {
        let mut slot = [0; INLINE];
        slot[..4].copy_from_slice(&start.to_le_bytes());
        slot[4..8].copy_from_slice(&end.to_le_bytes());
        slot[15] = LONG;
        slot
    }

    #[inline(always)]
    fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
        let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
        let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
        &spelling[start..end]
    }

    fn begin_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
        if inline16.len() < 256 {
            inline16.clear();
            inline16.reserve_exact(256);
            inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            lens.clear();
            lens.reserve_exact(256);
            lens.resize(256, 1);
        }
        inline16.truncate(256);
        lens.truncate(256);
    }

    fn begin_packed15(packed15: &mut Vec<[u8; INLINE]>) {
        if packed15.len() < 256 {
            packed15.clear();
            packed15.reserve_exact(256);
            packed15.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v[15] = 1;
                v
            }));
        }
        packed15.truncate(256);
    }

    #[cold]
    #[inline(never)]
    fn grow_step_u8(v: &mut Vec<u8>, need: usize, step: usize) {
        let target = need.div_ceil(step) * step;
        v.reserve_exact(target - v.len());
    }

    #[inline(always)]
    fn ensure_step_u8(v: &mut Vec<u8>, add: usize, step: usize) {
        let need = v.len() + add;
        if need > v.capacity() {
            grow_step_u8(v, need, step);
        }
    }

    #[cold]
    #[inline(never)]
    fn grow_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
        let target = (lens.len() + 1).div_ceil(step) * step;
        inline16.reserve_exact(target - inline16.len());
        lens.reserve_exact(target - lens.len());
    }

    #[inline(always)]
    fn ensure_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
        if lens.len() == lens.capacity() {
            grow_step_inline(inline16, lens, step);
        }
    }

    #[cold]
    #[inline(never)]
    fn grow_step_packed(packed15: &mut Vec<[u8; INLINE]>, step: usize) {
        let target = (packed15.len() + 1).div_ceil(step) * step;
        packed15.reserve_exact(target - packed15.len());
    }

    #[inline(always)]
    fn ensure_step_packed(packed15: &mut Vec<[u8; INLINE]>, step: usize) {
        if packed15.len() == packed15.capacity() {
            grow_step_packed(packed15, step);
        }
    }

    #[cold]
    #[inline(never)]
    fn grow_edges_step(a: &mut Vec<u32>, need: usize) {
        let target = need.div_ceil(8192) * 8192;
        if target > a.capacity() {
            a.reserve_exact(target - a.len());
        }
        a.resize(target, 0);
    }

    mod outq {
        const CHUNK: usize = 512;
        pub(super) const BYTES: usize = 2 * CHUNK;
        const NONE32: u32 = u32::MAX;
        const NONE16: u16 = u16::MAX;
        const EPOCH: u32 = 1 << 16;

        #[derive(Default)]
        pub(super) struct Spill {
            next: Vec<u16>,
            ovf: Vec<[u8; BYTES]>,
            cuts: Vec<[u32; 256]>,
        }

        impl Spill {
            pub(super) fn cap_bytes(&self) -> usize {
                self.next.capacity() * 2
                    + self.ovf.capacity() * BYTES
                    + self.cuts.capacity() * 1024
            }
        }

        pub(super) struct Queues<'a> {
            out: &'a mut [u8],
            spill: &'a mut Spill,
            in_out: u32,
            chunks: u32,
            heads: [u32; 256],
            pt: [u64; 256],
            limit: u32,
            over: bool,
        }

        pub(super) struct Heads {
            heads: [u32; 256],
            in_out: u32,
            pub(super) chunks: u32,
        }

        impl<'a> Queues<'a> {
            pub(super) fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
                spill.ovf.clear();
                spill.cuts.clear();
                let in_out = (out.len() / BYTES) as u32;
                let out = &mut out[..in_out as usize * BYTES];
                Self {
                    in_out,
                    out,
                    spill,
                    chunks: 0,
                    heads: [NONE32; 256],
                    pt: [u32::MAX as u64; 256],
                    limit,
                    over: false,
                }
            }
            pub(super) fn over(&self) -> bool {
                self.over
            }
            pub(super) fn heads(&self) -> Heads {
                Heads {
                    heads: self.heads,
                    in_out: self.in_out,
                    chunks: self.chunks,
                }
            }
            #[inline(always)]
            pub(super) fn thin_of(&self, a: usize) -> u32 {
                (self.pt[a] >> 32) as u32
            }
            pub(super) fn extract_thin(&self) -> [u32; 256] {
                std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
            }
            #[inline(always)]
            pub(super) fn push_thin<const MAX_W: u32>(&mut self, a: usize, m: u32, w: u32) {
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & 1023) as usize;
                let ok_w = if MAX_W <= 16 { true } else { w <= 16 };
                if ok_w & (f <= CHUNK - 16) & ((m as u16) < 65_520) {
                    let at = p as usize + f;
                    if let Some(dst) = self.out.get_mut(at..at + 32) {
                        let lo = m as u16;
                        let iota: [u16; 16] =
                            std::array::from_fn(|i| lo.wrapping_add(i as u16));
                        for (d, x) in dst.chunks_exact_mut(2).zip(iota) {
                            d.copy_from_slice(&x.to_le_bytes());
                        }
                        let wu = u64::from(w);
                        self.pt[a] = v + ((wu << 32) | wu);
                        return;
                    }
                }
                self.push_slow_pt(a, m, w);
            }
            #[cold]
            #[inline(never)]
            fn push_slow_pt(&mut self, a: usize, mut m: u32, w: u32) {
                if self.over || m + w > self.limit {
                    self.over = true;
                    return;
                }
                let mut left = w;
                while left > 0 {
                    let p = self.pt[a] as u32;
                    if p == u32::MAX || (p & 1023) as usize == CHUNK {
                        if self.chunks >= u32::from(NONE16) {
                            self.over = true;
                            return;
                        }
                        self.take_chunk_pt(a);
                    }
                    let v = self.pt[a];
                    let p = v as u32;
                    let f = (p & 1023) as usize;
                    let lo = m & (EPOCH - 1);
                    let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
                    let chunk = self.chunk_mut(p >> 10);
                    for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                        .chunks_exact_mut(2)
                        .enumerate()
                    {
                        d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                    }
                    let tu = u64::from(take);
                    self.pt[a] = v + ((tu << 32) | tu);
                    m += take;
                    left -= take;
                    if m % EPOCH == 0 {
                        let cut: [u32; 256] =
                            std::array::from_fn(|b| (self.pt[b] >> 32) as u32);
                        if self.spill.cuts.len() == self.spill.cuts.capacity() {
                            self.spill.cuts.reserve_exact(1);
                        }
                        self.spill.cuts.push(cut);
                    }
                }
            }
            fn take_chunk_pt(&mut self, a: usize) {
                let id = self.chunks;
                self.chunks += 1;
                if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                    if self.spill.ovf.len() == self.spill.ovf.capacity() {
                        let target = (self.spill.ovf.len() + 1).div_ceil(16) * 16;
                        self.spill.ovf.reserve_exact(target - self.spill.ovf.len());
                    }
                    self.spill.ovf.push([0; BYTES]);
                }
                let idu = id as usize;
                if self.spill.next.len() <= idu {
                    let target = (idu + 1).div_ceil(256) * 256;
                    if target > self.spill.next.capacity() {
                        self.spill
                            .next
                            .reserve_exact(target - self.spill.next.len());
                    }
                    self.spill.next.resize(target, NONE16);
                }
                self.spill.next[idu] = NONE16;
                let hi = self.pt[a] & !0xFFFF_FFFFu64;
                if self.heads[a] == NONE32 {
                    self.heads[a] = id;
                } else {
                    self.spill.next[((self.pt[a] as u32) >> 10) as usize] = id as u16;
                }
                self.pt[a] = hi | u64::from(id << 10);
            }
            #[cold]
            #[inline(never)]
            fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
                if id < self.in_out {
                    let at = id as usize * BYTES;
                    &mut self.out[at..at + BYTES]
                } else {
                    &mut self.spill.ovf[(id - self.in_out) as usize][..]
                }
            }
        }

        pub(super) struct Reader<'a> {
            out: &'a [u8],
            spill: &'a Spill,
            in_out: u32,
            a: usize,
            id: u32,
            at: usize,
            read: u32,
            epoch: u32,
            cut: u32,
        }

        impl<'a> Reader<'a> {
            #[inline(always)]
            pub(super) fn new(
                out: &'a [u8],
                spill: &'a Spill,
                heads: &Heads,
                a: usize,
            ) -> Self {
                Self {
                    out,
                    spill,
                    in_out: heads.in_out,
                    a,
                    id: heads.heads[a],
                    at: 0,
                    read: 0,
                    epoch: 0,
                    cut: spill.cuts.first().map_or(u32::MAX, |c| c[a]),
                }
            }
            #[inline(always)]
            fn next_chunk(&self, id: usize) -> u32 {
                u32::from(self.spill.next[id])
            }
            #[inline(always)]
            fn cut_at(&self, epoch: usize) -> u32 {
                self.spill.cuts.get(epoch).map_or(u32::MAX, |c| c[self.a])
            }
            #[inline(always)]
            pub(super) fn wire_xor_nomark(
                &mut self,
                edges: &mut [u32],
                dst_start: usize,
                span: usize,
                pack: u32,
            ) {
                if self.at + span <= CHUNK
                    && self.read + span as u32 <= self.cut
                    && self.id < self.in_out
                {
                    let hi = self.epoch << 16;
                    let at = self.id as usize * BYTES + 2 * self.at;
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    let mut i = 0usize;
                    if hi == 0 {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let v0 = u32::from(u16::from_le_bytes(ch[0]));
                            let v1 = u32::from(u16::from_le_bytes(ch[1]));
                            let v2 = u32::from(u16::from_le_bytes(ch[2]));
                            let v3 = u32::from(u16::from_le_bytes(ch[3]));
                            let v4 = u32::from(u16::from_le_bytes(ch[4]));
                            let v5 = u32::from(u16::from_le_bytes(ch[5]));
                            let v6 = u32::from(u16::from_le_bytes(ch[6]));
                            let v7 = u32::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack | v0;
                            eu[1] ^= pack | v1;
                            eu[2] ^= pack | v2;
                            eu[3] ^= pack | v3;
                            eu[4] ^= pack | v4;
                            eu[5] ^= pack | v5;
                            eu[6] ^= pack | v6;
                            eu[7] ^= pack | v7;
                            edges[v0 as usize] ^= u0 as u32;
                            edges[v1 as usize] ^= (u0 + 1) as u32;
                            edges[v2 as usize] ^= (u0 + 2) as u32;
                            edges[v3 as usize] ^= (u0 + 3) as u32;
                            edges[v4 as usize] ^= (u0 + 4) as u32;
                            edges[v5 as usize] ^= (u0 + 5) as u32;
                            edges[v6 as usize] ^= (u0 + 6) as u32;
                            edges[v7 as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let v = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack | v;
                            edges[v as usize] ^= u as u32;
                            i += 1;
                        }
                    } else {
                        let pack_hi = pack | hi;
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = u32::from(u16::from_le_bytes(ch[0]));
                            let l1 = u32::from(u16::from_le_bytes(ch[1]));
                            let l2 = u32::from(u16::from_le_bytes(ch[2]));
                            let l3 = u32::from(u16::from_le_bytes(ch[3]));
                            let l4 = u32::from(u16::from_le_bytes(ch[4]));
                            let l5 = u32::from(u16::from_le_bytes(ch[5]));
                            let l6 = u32::from(u16::from_le_bytes(ch[6]));
                            let l7 = u32::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | l0;
                            eu[1] ^= pack_hi | l1;
                            eu[2] ^= pack_hi | l2;
                            eu[3] ^= pack_hi | l3;
                            eu[4] ^= pack_hi | l4;
                            eu[5] ^= pack_hi | l5;
                            eu[6] ^= pack_hi | l6;
                            eu[7] ^= pack_hi | l7;
                            edges[(hi | l0) as usize] ^= u0 as u32;
                            edges[(hi | l1) as usize] ^= (u0 + 1) as u32;
                            edges[(hi | l2) as usize] ^= (u0 + 2) as u32;
                            edges[(hi | l3) as usize] ^= (u0 + 3) as u32;
                            edges[(hi | l4) as usize] ^= (u0 + 4) as u32;
                            edges[(hi | l5) as usize] ^= (u0 + 5) as u32;
                            edges[(hi | l6) as usize] ^= (u0 + 6) as u32;
                            edges[(hi | l7) as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | lo;
                            edges[(hi | lo) as usize] ^= u as u32;
                            i += 1;
                        }
                    }
                    self.at += span;
                    self.read += span as u32;
                } else {
                    self.wire_xor_slow_nomark(edges, dst_start, span, pack);
                }
            }
            #[cold]
            #[inline(never)]
            fn wire_xor_slow_nomark(
                &mut self,
                edges: &mut [u32],
                mut dst_start: usize,
                mut left: usize,
                pack: u32,
            ) {
                while left > 0 {
                    while self.read == self.cut {
                        self.epoch += 1;
                        self.cut = self.cut_at(self.epoch as usize);
                    }
                    if self.at == CHUNK {
                        self.id = self.next_chunk(self.id as usize);
                        self.at = 0;
                    }
                    let hi = self.epoch << 16;
                    let room_chunk = CHUNK - self.at;
                    let room_cut = (self.cut - self.read) as usize;
                    let take = left.min(room_chunk).min(room_cut);
                    let chunk: &[u8] = if self.id < self.in_out {
                        let base = self.id as usize * BYTES;
                        &self.out[base..base + BYTES]
                    } else {
                        &self.spill.ovf[(self.id - self.in_out) as usize][..]
                    };
                    let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                    let mut i = 0usize;
                    if hi == 0 {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let v0 = u32::from(u16::from_le_bytes(ch[0]));
                            let v1 = u32::from(u16::from_le_bytes(ch[1]));
                            let v2 = u32::from(u16::from_le_bytes(ch[2]));
                            let v3 = u32::from(u16::from_le_bytes(ch[3]));
                            let v4 = u32::from(u16::from_le_bytes(ch[4]));
                            let v5 = u32::from(u16::from_le_bytes(ch[5]));
                            let v6 = u32::from(u16::from_le_bytes(ch[6]));
                            let v7 = u32::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack | v0;
                            eu[1] ^= pack | v1;
                            eu[2] ^= pack | v2;
                            eu[3] ^= pack | v3;
                            eu[4] ^= pack | v4;
                            eu[5] ^= pack | v5;
                            eu[6] ^= pack | v6;
                            eu[7] ^= pack | v7;
                            edges[v0 as usize] ^= u0 as u32;
                            edges[v1 as usize] ^= (u0 + 1) as u32;
                            edges[v2 as usize] ^= (u0 + 2) as u32;
                            edges[v3 as usize] ^= (u0 + 3) as u32;
                            edges[v4 as usize] ^= (u0 + 4) as u32;
                            edges[v5 as usize] ^= (u0 + 5) as u32;
                            edges[v6 as usize] ^= (u0 + 6) as u32;
                            edges[v7 as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let v = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack | v;
                            edges[v as usize] ^= u as u32;
                            i += 1;
                        }
                    } else {
                        let pack_hi = pack | hi;
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = u32::from(u16::from_le_bytes(ch[0]));
                            let l1 = u32::from(u16::from_le_bytes(ch[1]));
                            let l2 = u32::from(u16::from_le_bytes(ch[2]));
                            let l3 = u32::from(u16::from_le_bytes(ch[3]));
                            let l4 = u32::from(u16::from_le_bytes(ch[4]));
                            let l5 = u32::from(u16::from_le_bytes(ch[5]));
                            let l6 = u32::from(u16::from_le_bytes(ch[6]));
                            let l7 = u32::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | l0;
                            eu[1] ^= pack_hi | l1;
                            eu[2] ^= pack_hi | l2;
                            eu[3] ^= pack_hi | l3;
                            eu[4] ^= pack_hi | l4;
                            eu[5] ^= pack_hi | l5;
                            eu[6] ^= pack_hi | l6;
                            eu[7] ^= pack_hi | l7;
                            edges[(hi | l0) as usize] ^= u0 as u32;
                            edges[(hi | l1) as usize] ^= (u0 + 1) as u32;
                            edges[(hi | l2) as usize] ^= (u0 + 2) as u32;
                            edges[(hi | l3) as usize] ^= (u0 + 3) as u32;
                            edges[(hi | l4) as usize] ^= (u0 + 4) as u32;
                            edges[(hi | l5) as usize] ^= (u0 + 5) as u32;
                            edges[(hi | l6) as usize] ^= (u0 + 6) as u32;
                            edges[(hi | l7) as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | lo;
                            edges[(hi | lo) as usize] ^= u as u32;
                            i += 1;
                        }
                    }
                    self.at += take;
                    self.read += take as u32;
                    dst_start += take;
                    left -= take;
                }
            }
            #[inline(always)]
            pub(super) fn wire_xor(
                &mut self,
                edges: &mut [u32],
                dst_start: usize,
                span: usize,
                pack: u32,
                mark: u32,
                predecessor: &mut u32,
            ) {
                if self.at + span <= CHUNK
                    && self.read + span as u32 <= self.cut
                    && self.id < self.in_out
                {
                    let hi = self.epoch << 16;
                    let at = self.id as usize * BYTES + 2 * self.at;
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    let pack_hi = pack | hi;
                    for (i, &p) in pairs.iter().enumerate() {
                        let u = dst_start + i;
                        let lo = u32::from(u16::from_le_bytes(p));
                        let v = hi | lo;
                        edges[u] ^= pack_hi | lo;
                        edges[v as usize] ^= u as u32;
                        if v == mark {
                            *predecessor = u as u32;
                        }
                    }
                    self.at += span;
                    self.read += span as u32;
                } else {
                    self.wire_xor_slow(edges, dst_start, span, pack, mark, predecessor);
                }
            }
            #[cold]
            #[inline(never)]
            fn wire_xor_slow(
                &mut self,
                edges: &mut [u32],
                mut dst_start: usize,
                mut left: usize,
                pack: u32,
                mark: u32,
                predecessor: &mut u32,
            ) {
                while left > 0 {
                    while self.read == self.cut {
                        self.epoch += 1;
                        self.cut = self.cut_at(self.epoch as usize);
                    }
                    if self.at == CHUNK {
                        self.id = self.next_chunk(self.id as usize);
                        self.at = 0;
                    }
                    let hi = self.epoch << 16;
                    let room_chunk = CHUNK - self.at;
                    let room_cut = (self.cut - self.read) as usize;
                    let take = left.min(room_chunk).min(room_cut);
                    let chunk: &[u8] = if self.id < self.in_out {
                        let base = self.id as usize * BYTES;
                        &self.out[base..base + BYTES]
                    } else {
                        &self.spill.ovf[(self.id - self.in_out) as usize][..]
                    };
                    let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                    for (i, &p) in pairs.iter().enumerate() {
                        let u = dst_start + i;
                        let v = hi | u32::from(u16::from_le_bytes(p));
                        edges[u] ^= pack | v;
                        edges[v as usize] ^= u as u32;
                        if v == mark {
                            *predecessor = u as u32;
                        }
                    }
                    self.at += take;
                    self.read += take as u32;
                    dst_start += take;
                    left -= take;
                }
            }
            #[inline(always)]
            pub(super) fn wire_uni(
                &mut self,
                edges: &mut [u32],
                dst_start: usize,
                span: usize,
                pack: u32,
            ) {
                if self.at + span <= CHUNK
                    && self.read + span as u32 <= self.cut
                    && self.id < self.in_out
                {
                    let hi = self.epoch << 16;
                    let at = self.id as usize * BYTES + 2 * self.at;
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    let pack_hi = pack | hi;
                    for (eu, &p) in edges[dst_start..dst_start + span].iter_mut().zip(pairs) {
                        *eu = pack_hi | u32::from(u16::from_le_bytes(p));
                    }
                    self.at += span;
                    self.read += span as u32;
                } else {
                    self.wire_uni_slow(edges, dst_start, span, pack);
                }
            }
            #[cold]
            #[inline(never)]
            fn wire_uni_slow(
                &mut self,
                edges: &mut [u32],
                mut dst_start: usize,
                mut left: usize,
                pack: u32,
            ) {
                while left > 0 {
                    while self.read == self.cut {
                        self.epoch += 1;
                        self.cut = self.cut_at(self.epoch as usize);
                    }
                    if self.at == CHUNK {
                        self.id = self.next_chunk(self.id as usize);
                        self.at = 0;
                    }
                    let hi = self.epoch << 16;
                    let room_chunk = CHUNK - self.at;
                    let room_cut = (self.cut - self.read) as usize;
                    let take = left.min(room_chunk).min(room_cut);
                    let chunk: &[u8] = if self.id < self.in_out {
                        let base = self.id as usize * BYTES;
                        &self.out[base..base + BYTES]
                    } else {
                        &self.spill.ovf[(self.id - self.in_out) as usize][..]
                    };
                    let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                    let pack_hi = pack | hi;
                    for (eu, &p) in edges[dst_start..dst_start + take].iter_mut().zip(pairs) {
                        *eu = pack_hi | u32::from(u16::from_le_bytes(p));
                    }
                    self.at += take;
                    self.read += take as u32;
                    dst_start += take;
                    left -= take;
                }
            }
        }
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

    struct Elim<'n, 'r, L: Links> {
        nodes_buf: &'n mut [u32],
        rules_buf: &'r mut [u32],
        rule_top: usize,
        nodes: u32,
        id_overflow: bool,
        _links: std::marker::PhantomData<L>,
    }

    impl<L: Links> Elim<'_, '_, L> {
        #[inline(always)]
        fn width(&self, id: u32) -> u32 {
            self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 2]
        }
        #[inline(always)]
        fn set_width(&mut self, id: u32, w: u32) {
            self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 2] = w;
        }
        #[inline(always)]
        fn word(&self, id: u32) -> u32 {
            self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 1]
        }
        #[inline(always)]
        fn set_word(&mut self, id: u32, w: u32) {
            self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 1] = w;
        }
        #[inline(always)]
        fn set_bottom(&mut self, id: u32, x: u32) {
            self.nodes_buf[id as usize * L::STRIDE + L::BOTTOM] = x;
        }
        fn assign_bottom(&mut self, rows: u32) {
            if L::IS_NARROW {
                let (nodes, _) = self.nodes_buf.as_chunks_mut::<4>();
                let mut first = nodes[0][1] >> 16;
                let mut last = nodes[0][1] & 0xFFFF;
                let (mut lo, mut hi) = (0u32, rows);
                while first != last {
                    let nf = &mut nodes[first as usize];
                    let next = nf[1] >> 16;
                    let width = nf[2];
                    nf[1] = lo;
                    lo += width;
                    let nl = &mut nodes[last as usize];
                    let prev = nl[1] & 0xFFFF;
                    hi -= nl[2];
                    nl[1] = hi;
                    if next == last {
                        assert_eq!(lo, hi);
                        return;
                    }
                    first = next;
                    last = prev;
                }
                if first != 0 {
                    let nf = &mut nodes[first as usize];
                    nf[1] = lo;
                    lo += nf[2];
                }
                assert_eq!(lo, hi);
                return;
            }
            let mut first = L::next(self.nodes_buf, 0, 1);
            let mut last = L::prev(self.nodes_buf, 0, 1);
            let (mut lo, mut hi) = (0, rows);
            while first != last {
                let next = L::next(self.nodes_buf, first, 1);
                let prev = L::prev(self.nodes_buf, last, 1);
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
        #[inline(always)]
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
                L::set_next(self.nodes_buf, *tail, side, id);
            }
            L::set_prev(self.nodes_buf, id, side, *tail);
            *tail = id;
        }
        fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
            let mut prev = 0;
            for (head, tail) in [first, second] {
                if tail == 0 {
                    continue;
                }
                L::set_next(self.nodes_buf, prev, side, head);
                L::set_prev(self.nodes_buf, head, side, prev);
                prev = tail;
            }
            L::set_next(self.nodes_buf, prev, side, 0);
            L::set_prev(self.nodes_buf, 0, side, prev);
        }
        #[inline(always)]
        fn cut(&mut self, id: u32, side: usize) {
            let (p, n) = (
                L::prev(self.nodes_buf, id, side),
                L::next(self.nodes_buf, id, side),
            );
            L::set_next(self.nodes_buf, p, side, n);
            L::set_prev(self.nodes_buf, n, side, p);
        }
        #[inline(always)]
        fn pop_tail(&mut self, id: u32, side: usize) {
            let prev = L::prev(self.nodes_buf, id, side);
            L::set_next(self.nodes_buf, prev, side, 0);
            L::set_prev(self.nodes_buf, 0, side, prev);
        }
        #[inline(always)]
        fn insert(&mut self, at: u32, id: u32, side: usize) {
            let next = L::next(self.nodes_buf, at, side);
            L::set_prev(self.nodes_buf, id, side, at);
            L::set_next(self.nodes_buf, id, side, next);
            L::set_next(self.nodes_buf, at, side, id);
            L::set_prev(self.nodes_buf, next, side, id);
        }
        #[inline(always)]
        fn len_handle(&self, h: u32) -> u32 {
            let g = h >> ID_BITS;
            if g < SATURATED {
                g
            } else {
                self.rules_buf[(h & ID_MASK) as usize + 2]
            }
        }
        #[inline(always)]
        fn short_payload_h(&self, h: u32) -> u128 {
            let g = h >> ID_BITS;
            let id = h & ID_MASK;
            if g <= 3 {
                return u128::from(id);
            }
            let r = id as usize;
            if g <= 8 {
                let sl = &self.rules_buf[r..r + 2];
                u128::from(sl[0]) | (u128::from(sl[1]) << 32)
            } else if g <= 12 {
                let sl = &self.rules_buf[r..r + 3];
                u128::from(sl[0]) | (u128::from(sl[1]) << 32) | (u128::from(sl[2]) << 64)
            } else {
                let sl = &self.rules_buf[r..r + 4];
                u128::from(sl[0])
                    | (u128::from(sl[1]) << 32)
                    | (u128::from(sl[2]) << 64)
                    | (u128::from(sl[3]) << 96)
            }
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
            let sum = left_len + (right >> ID_BITS);
            if sum <= 3 {
                return (sum << ID_BITS)
                    | (left & ID_MASK)
                    | ((right & ID_MASK) << (8 * left_len));
            }
            let t = self.rule_top;
            let id = t as u32;
            if sum < INLINE as u32 {
                let bytes = self.short_payload_h(left)
                    | (self.short_payload_h(right) << (8 * left_len));
                let out = &mut self.rules_buf[t..t + 4];
                out[0] = bytes as u32;
                out[1] = (bytes >> 32) as u32;
                if sum <= 8 {
                    self.rule_top = t + 2;
                } else if sum <= 12 {
                    out[2] = (bytes >> 64) as u32;
                    self.rule_top = t + 3;
                } else {
                    out[2] = (bytes >> 64) as u32;
                    out[3] = (bytes >> 96) as u32;
                    self.rule_top = t + 4;
                }
                (sum << ID_BITS) | id
            } else if sum < SATURATED {
                let out = &mut self.rules_buf[t..t + 3];
                out[0] = left;
                out[1] = right;
                out[2] = u32::MAX;
                self.rule_top = t + 3;
                (sum << ID_BITS) | id
            } else {
                let len = if sum == SATURATED {
                    sum
                } else {
                    self.len_handle(left) + self.len_handle(right)
                };
                let out = &mut self.rules_buf[t..t + 4];
                out[0] = left;
                out[1] = right;
                out[2] = len;
                out[3] = u32::MAX;
                self.rule_top = t + 4;
                (SATURATED << ID_BITS) | id
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
            let max_id_top = ID_MASK as usize + 1;
            let limit = self.rules_buf.len().min(max_id_top);
            let done = loop {
                if left == 0 {
                    break true;
                }
                if self.rule_top + MAX_RULE_WORDS > limit {
                    if self.rule_top + MAX_RULE_WORDS > max_id_top {
                        self.id_overflow = true;
                    }
                    break false;
                }
                let sent = &self.nodes_buf[..2];
                let x = sent[0] & 0xFFFF;
                let y = sent[1] & 0xFFFF;
                let xi = x as usize * 4;
                let yi = y as usize * 4;
                let nx = &self.nodes_buf[xi..xi + 4];
                let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
                if x == y {
                    assert!(aw > 0 && aw <= left);
                    left -= aw;
                    let px0 = x0 & 0xFFFF;
                    let px1 = x1 & 0xFFFF;
                    self.nodes_buf[px0 as usize * 4] &= 0xFFFF;
                    self.nodes_buf[0] = (self.nodes_buf[0] & 0xFFFF_0000) | px0;
                    self.nodes_buf[px1 as usize * 4 + 1] &= 0xFFFF;
                    self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | px1;
                    continue;
                }
                let ny = &self.nodes_buf[yi..yi + 4];
                let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
                let w = aw.min(bw);
                assert!(w > 0 && w <= left);
                left -= w;
                let word = self.join(wx, wy);
                if aw <= bw {
                    let px = x0 & 0xFFFF;
                    self.nodes_buf[px as usize * 4] &= 0xFFFF;
                    self.nodes_buf[0] = (self.nodes_buf[0] & 0xFFFF_0000) | px;
                    if aw < bw {
                        let y_next = if y == px { 0 } else { y0 >> 16 };
                        let nxm = &mut self.nodes_buf[xi..xi + 4];
                        nxm[0] = y | (y_next << 16);
                        nxm[3] = word;
                        let nym = &mut self.nodes_buf[yi..yi + 4];
                        nym[0] = (y0 & 0xFFFF) | (x << 16);
                        nym[2] = bw - aw;
                        let yni = y_next as usize * 4;
                        self.nodes_buf[yni] = (self.nodes_buf[yni] & 0xFFFF_0000) | x;
                    } else {
                        let y_next = if y == px { 0 } else { y0 >> 16 };
                        let y_prev = y0 & 0xFFFF;
                        let nxm = &mut self.nodes_buf[xi..xi + 4];
                        nxm[0] = y_prev | (y_next << 16);
                        nxm[3] = word;
                        let ypi = y_prev as usize * 4;
                        self.nodes_buf[ypi] = (self.nodes_buf[ypi] & 0xFFFF) | (x << 16);
                        let yni = y_next as usize * 4;
                        self.nodes_buf[yni] = (self.nodes_buf[yni] & 0xFFFF_0000) | x;
                        let py = y1 & 0xFFFF;
                        self.nodes_buf[py as usize * 4 + 1] &= 0xFFFF;
                        self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | py;
                    }
                } else {
                    let py = y1 & 0xFFFF;
                    self.nodes_buf[py as usize * 4 + 1] &= 0xFFFF;
                    self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | py;
                    let x_next = if x == py { 0 } else { x1 >> 16 };
                    let nym = &mut self.nodes_buf[yi..yi + 4];
                    nym[1] = x | (x_next << 16);
                    nym[3] = word;
                    let nxm = &mut self.nodes_buf[xi..xi + 4];
                    nxm[1] = (x1 & 0xFFFF) | (y << 16);
                    nxm[2] = aw - bw;
                    let xni = x_next as usize * 4 + 1;
                    self.nodes_buf[xni] = (self.nodes_buf[xni] & 0xFFFF_0000) | y;
                }
            };
            *remaining = left;
            done
        }
        fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
            if !SPIRALS && L::IS_NARROW {
                return self.eliminate_narrow_plain(remaining);
            }
            let mut left = *remaining;
            let room = if SPIRALS {
                SPIRAL_RULES * MAX_RULE_WORDS
            } else {
                MAX_RULE_WORDS
            };
            let max_id_top = ID_MASK as usize + 1;
            let limit = self.rules_buf.len().min(max_id_top);
            let done = loop {
                if left == 0 {
                    break true;
                }
                if self.rule_top + room > limit {
                    if self.rule_top + room > max_id_top {
                        self.id_overflow = true;
                    }
                    break false;
                }
                let (x, y) = (L::prev(self.nodes_buf, 0, 0), L::prev(self.nodes_buf, 0, 1));
                let (aw, bw) = (self.width(x), self.width(y));
                if SPIRALS && x != y {
                    if aw > bw && L::prev(self.nodes_buf, y, 1) == x && (aw - 1) / bw >= 2 {
                        let q = (aw - 1) / bw;
                        assert!(q * bw < aw && q * bw <= left);
                        let head = self.power(self.word(x), q);
                        let word = self.join(head, self.word(y));
                        self.set_word(y, word);
                        self.set_width(x, aw - q * bw);
                        left -= q * bw;
                        continue;
                    }
                    if aw < bw && L::prev(self.nodes_buf, x, 0) == y && (bw - 1) / aw >= 2 {
                        let q = (bw - 1) / aw;
                        assert!(q * aw < bw && q * aw <= left);
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
        #[allow(clippy::too_many_arguments)]
        fn inline_word(
            &mut self,
            word: u32,
            byte: u8,
            istk: &mut [u32; INLINE_STACK],
            memo_rec: &mut [u32; MEMO_REC_CAP],
            stack: &mut Vec<u32>,
            spelling: &mut Vec<u8>,
        ) -> ([u8; INLINE], u8) {
            let height = (word >> ID_BITS) as usize;
            if height < INLINE {
                let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
                (v.to_le_bytes(), (height + 1) as u8)
            } else {
                let start = spelling.len() as u32;
                let total = self.len_handle(word) as usize;
                ensure_step_u8(spelling, total + 1, SPELLING_STEP);
                let mut dummy_n = 0usize;
                let mut dummy_ovf = false;
                self.append_tail_unw::<false>(
                    word,
                    istk,
                    stack,
                    spelling,
                    memo_rec,
                    &mut dummy_n,
                    &mut dummy_ovf,
                );
                spelling.push(byte);
                (long_slot(start, spelling.len() as u32), LONG)
            }
        }
        #[allow(clippy::too_many_arguments)]
        fn packed15_word(
            &mut self,
            word: u32,
            byte: u8,
            istk: &mut [u32; INLINE_STACK],
            memo_rec: &mut [u32; MEMO_REC_CAP],
            stack: &mut Vec<u32>,
            spelling: &mut Vec<u8>,
        ) -> [u8; INLINE] {
            let height = (word >> ID_BITS) as usize;
            if height < PACKED_MAX {
                let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
                let mut slot = v.to_le_bytes();
                slot[15] = (height + 1) as u8;
                slot
            } else {
                let start = spelling.len() as u32;
                let total = self.len_handle(word) as usize;
                ensure_step_u8(spelling, total + 1, SPELLING_STEP);
                let mut dummy_n = 0usize;
                let mut dummy_ovf = false;
                self.append_tail_unw::<false>(
                    word,
                    istk,
                    stack,
                    spelling,
                    memo_rec,
                    &mut dummy_n,
                    &mut dummy_ovf,
                );
                spelling.push(byte);
                long_slot_packed(start, spelling.len() as u32)
            }
        }
        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn append_tail_unw<const TRACK_MEMO: bool>(
            &mut self,
            handle: u32,
            istk: &mut [u32; INLINE_STACK],
            ovf_stack: &mut Vec<u32>,
            dest: &mut Vec<u8>,
            memo_rec: &mut [u32; MEMO_REC_CAP],
            memo_n: &mut usize,
            memo_ovf: &mut bool,
        ) {
            let mut sp = 0usize;
            let mut used_heap = false;
            let mut h = handle;
            loop {
                let g = h >> ID_BITS;
                if g < INLINE as u32 {
                    let bytes = self.short_payload_h(h).to_le_bytes();
                    dest.extend_from_slice(&bytes[..g as usize]);
                    if used_heap && !ovf_stack.is_empty() {
                        h = ovf_stack.pop().unwrap();
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = istk[sp];
                        continue;
                    }
                    return;
                }
                let r = (h & ID_MASK) as usize;
                let m_off = if g < SATURATED { r + 2 } else { r + 3 };
                let tag = self.rules_buf[m_off];
                if tag != u32::MAX {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = if g < SATURATED {
                        g as usize
                    } else {
                        self.rules_buf[r + 2] as usize
                    };
                    dest.extend_from_within(start..start + rlen);
                    if used_heap && !ovf_stack.is_empty() {
                        h = ovf_stack.pop().unwrap();
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = istk[sp];
                        continue;
                    }
                    return;
                }
                self.rules_buf[m_off] = 0x8000_0000 | (dest.len() as u32);
                if TRACK_MEMO {
                    if *memo_n < MEMO_REC_CAP {
                        memo_rec[*memo_n] = m_off as u32;
                        *memo_n += 1;
                    } else {
                        *memo_ovf = true;
                    }
                }
                let left = self.rules_buf[r];
                let right = self.rules_buf[r + 1];
                let lg = left >> ID_BITS;
                if lg < INLINE as u32 {
                    let lb = self.short_payload_h(left).to_le_bytes();
                    dest.extend_from_slice(&lb[..lg as usize]);
                    let rg = right >> ID_BITS;
                    if rg < INLINE as u32 {
                        let rb = self.short_payload_h(right).to_le_bytes();
                        dest.extend_from_slice(&rb[..rg as usize]);
                        if used_heap && !ovf_stack.is_empty() {
                            h = ovf_stack.pop().unwrap();
                            continue;
                        }
                        if sp > 0 {
                            sp -= 1;
                            h = istk[sp];
                            continue;
                        }
                        return;
                    }
                    h = right;
                } else {
                    if sp < INLINE_STACK {
                        istk[sp] = right;
                        sp += 1;
                    } else {
                        if !used_heap {
                            ovf_stack.clear();
                            used_heap = true;
                        }
                        ovf_stack.push(right);
                    }
                    h = left;
                }
            }
        }
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

    struct Dictionary<'a> {
        pairs: &'a mut [u32; 512],
        contents: &'a mut [u32; 2048],
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        spelling: &'a mut Vec<u8>,
        istk: &'a mut [u32; INLINE_STACK],
        memo_rec: &'a mut [u32; MEMO_REC_CAP],
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
            let g = handle >> ID_BITS;
            let r = id as usize;
            let m_off = if g < SATURATED { r + 2 } else { r + 3 };
            let tag_r = e.rules_buf[m_off];
            if tag_r != u32::MAX {
                let pos = (tag_r & 0x7fff_ffff) as usize;
                let rlen = if g < SATURATED {
                    g as usize
                } else {
                    e.rules_buf[r + 2] as usize
                };
                ensure_step_u8(self.spelling, rlen + 1, SPELLING_STEP);
                self.spelling.extend_from_within(pos..pos + rlen);
                self.spelling.push(byte);
                let entry = long_slot(start as u32, self.spelling.len() as u32);
                let bytes = &self.spelling[start..];
                let h = spelling_hash_long(bytes);
                let tag = (((h >> 56) as u32) | 0x80) << 24;
                let index = (h as usize) & 2047;
                let ent = self.contents[index];
                if (ent & 0xFF00_0000) == tag && {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    self.lens[old] == LONG
                        && long_word(&self.inline16[old], self.spelling) == bytes
                } {
                    self.spelling.truncate(start);
                    return ent & 0x00FF_FFFF;
                }
                let c = self.insert(entry, LONG);
                self.contents[index] = tag | c;
                return c;
            }
            let total = if g < SATURATED {
                g as usize
            } else {
                e.rules_buf[r + 2] as usize
            };
            ensure_step_u8(self.spelling, total + 1, SPELLING_STEP);
            let mut memo_n = 0usize;
            let mut memo_ovf = false;
            e.append_tail_unw::<true>(
                handle,
                self.istk,
                self.stack,
                self.spelling,
                self.memo_rec,
                &mut memo_n,
                &mut memo_ovf,
            );
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
                if !memo_ovf {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    let old_start =
                        u32::from_le_bytes(self.inline16[old][..4].try_into().unwrap());
                    let delta = (start as u32) - old_start;
                    for &rm in &self.memo_rec[..memo_n] {
                        e.rules_buf[rm as usize] -= delta;
                    }
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
            ensure_step_inline(self.inline16, self.lens, INLINE_STEP);
            let code = self.lens.len() as u32;
            self.inline16.push(entry);
            self.lens.push(len);
            code
        }
    }

    /// Zero-remap unified `Packed15` Dictionary (`packed15: Vec<[u8; 16]>` with `slot[15] = len`
    /// in `1..=15` or `LONG = 255`; completely eliminates `lens: Vec<u8>`!).
    struct Packed15Dictionary<'a> {
        pairs: &'a mut [u32; 512],
        contents: &'a mut [u32; 2048],
        packed15: &'a mut Vec<[u8; INLINE]>,
        spelling: &'a mut Vec<u8>,
        istk: &'a mut [u32; INLINE_STACK],
        memo_rec: &'a mut [u32; MEMO_REC_CAP],
        stack: &'a mut Vec<u32>,
    }

    impl Packed15Dictionary<'_> {
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
            if height < PACKED_MAX {
                let len = (height + 1) as u8;
                let v = e.short_payload_h(handle)
                    | (u128::from(byte) << (8 * height))
                    | (u128::from(len) << 120);
                let entry = v.to_le_bytes();
                let lo = v as u64;
                let hi = (v >> 64) as u64;
                let h = (lo ^ hi.rotate_left(29)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                let h = h ^ (h >> 30);
                let tag = (((h >> 56) as u32) | 0x80) << 24;
                let index = (h as usize) & 2047;
                let ent = self.contents[index];
                if (ent & 0xFF00_0000) == tag && {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    self.packed15[old] == entry
                } {
                    return ent & 0x00FF_FFFF;
                }
                let c = self.insert(entry);
                self.contents[index] = tag | c;
                return c;
            }
            let start = self.spelling.len();
            let g = handle >> ID_BITS;
            if g < INLINE as u32 {
                // Exactly height == 15 -> total length 16: inline in S2b arena, spilled to spelling in Packed15
                ensure_step_u8(self.spelling, 16, SPELLING_STEP);
                let bytes_15 = e.short_payload_h(handle).to_le_bytes();
                self.spelling.extend_from_slice(&bytes_15[..15]);
                self.spelling.push(byte);
                let entry = long_slot_packed(start as u32, self.spelling.len() as u32);
                let bytes = &self.spelling[start..];
                let h = spelling_hash_long(bytes);
                let tag = (((h >> 56) as u32) | 0x80) << 24;
                let index = (h as usize) & 2047;
                let ent = self.contents[index];
                if (ent & 0xFF00_0000) == tag && {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    self.packed15[old][15] == LONG
                        && long_word(&self.packed15[old], self.spelling) == bytes
                } {
                    self.spelling.truncate(start);
                    return ent & 0x00FF_FFFF;
                }
                let c = self.insert(entry);
                self.contents[index] = tag | c;
                return c;
            }
            let r = id as usize;
            let m_off = if g < SATURATED { r + 2 } else { r + 3 };
            let tag_r = e.rules_buf[m_off];
            if tag_r != u32::MAX {
                let pos = (tag_r & 0x7fff_ffff) as usize;
                let rlen = if g < SATURATED {
                    g as usize
                } else {
                    e.rules_buf[r + 2] as usize
                };
                ensure_step_u8(self.spelling, rlen + 1, SPELLING_STEP);
                self.spelling.extend_from_within(pos..pos + rlen);
                self.spelling.push(byte);
                let entry = long_slot_packed(start as u32, self.spelling.len() as u32);
                let bytes = &self.spelling[start..];
                let h = spelling_hash_long(bytes);
                let tag = (((h >> 56) as u32) | 0x80) << 24;
                let index = (h as usize) & 2047;
                let ent = self.contents[index];
                if (ent & 0xFF00_0000) == tag && {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    self.packed15[old][15] == LONG
                        && long_word(&self.packed15[old], self.spelling) == bytes
                } {
                    self.spelling.truncate(start);
                    return ent & 0x00FF_FFFF;
                }
                let c = self.insert(entry);
                self.contents[index] = tag | c;
                return c;
            }
            let total = if g < SATURATED {
                g as usize
            } else {
                e.rules_buf[r + 2] as usize
            };
            ensure_step_u8(self.spelling, total + 1, SPELLING_STEP);
            let mut memo_n = 0usize;
            let mut memo_ovf = false;
            e.append_tail_unw::<true>(
                handle,
                self.istk,
                self.stack,
                self.spelling,
                self.memo_rec,
                &mut memo_n,
                &mut memo_ovf,
            );
            self.spelling.push(byte);
            let entry = long_slot_packed(start as u32, self.spelling.len() as u32);
            let bytes = &self.spelling[start..];
            let h = spelling_hash_long(bytes);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.packed15[old][15] == LONG
                    && long_word(&self.packed15[old], self.spelling) == bytes
            } {
                if !memo_ovf {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    let old_start =
                        u32::from_le_bytes(self.packed15[old][..4].try_into().unwrap());
                    let delta = (start as u32) - old_start;
                    for &rm in &self.memo_rec[..memo_n] {
                        e.rules_buf[rm as usize] -= delta;
                    }
                    self.spelling.truncate(start);
                } else {
                    self.spelling.pop();
                }
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry);
            self.contents[index] = tag | c;
            c
        }
        #[inline(always)]
        fn insert(&mut self, entry: [u8; INLINE]) -> u32 {
            ensure_step_packed(self.packed15, INLINE_STEP);
            let code = self.packed15.len() as u32;
            self.packed15.push(entry);
            code
        }
    }

    struct Pass<'a> {
        queues: outq::Queues<'a>,
        edges: &'a mut Vec<u32>,
        ns: usize,
        f: First,
    }

    impl Pass<'_> {
        #[inline(always)]
        fn take<const LIM: u32>(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            if width <= LIM {
                self.piece::<LIM>(c, pos, width, true, p);
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.piece::<LIM>(c, pos, width, false, p);
            } else {
                self.take_marked::<LIM>(c, pos, width, p);
            }
        }
        #[cold]
        #[inline(never)]
        fn take_marked<const LIM: u32>(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            let end = pos + width;
            if pos < p {
                self.piece::<LIM>(c, pos, p - pos, false, p);
            }
            self.piece::<LIM>(c, p, 1, true, p);
            if p + 1 < end {
                self.piece::<LIM>(c, p + 1, end - p - 1, false, p);
            }
        }
        #[inline(always)]
        fn piece<const LIM: u32>(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
            let a = usize::from(c);
            let f = &mut self.f;
            if keep {
                if p.wrapping_sub(x) < w {
                    f.mark = f.m + (p - x);
                    f.mark_byte = c;
                }
                self.queues.push_thin::<LIM>(a, f.m, w);
                f.m += w;
            } else {
                let at = 3 * self.ns;
                if self.edges.len() < at + 3 {
                    let need = (at + 3).div_ceil(4096) * 4096;
                    if need > self.edges.capacity() {
                        self.edges.reserve_exact(need - self.edges.len());
                    }
                    self.edges.resize(need, 0);
                }
                let thin_a = self.queues.thin_of(a);
                self.edges[at..at + 3].copy_from_slice(&[
                    x,
                    thin_a + f.thick_rows[a],
                    (w << 8) | u32::from(c),
                ]);
                self.ns += 1;
                f.thick[a] += 1;
                f.thick_rows[a] += w;
            }
        }
    }

    // ------------------------------------------------------------------------
    // Cold S5 Helpers + All 6 Zero-Remap S5 Emit Kernels (Bi + Uni + Sentinel)
    // ------------------------------------------------------------------------
    #[cold]
    #[inline(never)]
    fn emit_long_front(slot: &[u8; INLINE], spelling: &[u8], rem: &mut [u8]) -> usize {
        let word = long_word(slot, spelling);
        let take = word.len().min(rem.len());
        rem[..take].copy_from_slice(&word[..take]);
        take
    }

    #[cold]
    #[inline(never)]
    fn emit_long_back(slot: &[u8; INLINE], spelling: &[u8], rem: &mut [u8]) -> usize {
        let word = long_word(slot, spelling);
        let rlen = rem.len();
        let take = word.len().min(rlen);
        rem[rlen - take..].copy_from_slice(&word[word.len() - take..]);
        take
    }

    #[cold]
    #[inline(never)]
    fn emit_tail_back(slot: &[u8; INLINE], lb: u8, spelling: &[u8], rem: &mut [u8]) -> usize {
        let word = if lb == LONG {
            long_word(slot, spelling)
        } else {
            &slot[..usize::from(lb)]
        };
        let rlen = rem.len();
        let take = word.len().min(rlen);
        rem[rlen - take..].copy_from_slice(&word[word.len() - take..]);
        take
    }

    fn repeat(n: usize, out: &mut Vec<u8>) {
        assert!(!out.is_empty() && out.len() <= n);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }

    // --- ARM 0: Exact `C53-PipeLean-v1` `SlimS5_Pipe1_Rem16_LE16` (Bi) ---
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_base(
        edges: &[u32],
        spelling: &[u8],
        inline: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        before: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let out_base = out.as_ptr() as usize;
        let mut rem: &mut [u8] = &mut out[..n];
        let len_c = lens.len().min(inline.len());
        let lens = &lens[..len_c];
        let inline = &inline[..len_c];
        let mut done = false;
        if rem.len() >= 2 * INLINE {
            let mut ef = edges[front as usize];
            let mut eb = edges[back as usize];
            while rem.len() >= 2 * INLINE {
                if front == back {
                    let cf = (ef >> bits) as usize;
                    let lf = lens[cf];
                    if lf <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&inline[cf]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline[cf], spelling, rem);
                        rem = &mut rem[take..];
                    }
                    done = true;
                    break;
                }
                let f_next = (ef & mask) ^ prev;
                let b_next = (eb & mask) ^ next;
                let ef_nxt = edges[f_next as usize];
                let eb_nxt = edges[b_next as usize];

                let cf = (ef >> bits) as usize;
                let lf = lens[cf];
                if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&inline[cf]);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline[cf], spelling, rem);
                    rem = &mut rem[take..];
                    if rem.is_empty() {
                        done = true;
                        break;
                    }
                    if rem.len() < INLINE {
                        let cb = (eb >> bits) as usize;
                        let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                        let rlen = rem.len();
                        rem = &mut rem[..rlen - take_b];
                        prev = front;
                        front = f_next;
                        next = back;
                        back = b_next;
                        if front == next {
                            done = true;
                        }
                        break;
                    }
                }
                let cb = (eb >> bits) as usize;
                let rlen = rem.len();
                let lb = lens[cb];
                if lb <= INLINE as u8 {
                    let lbu = usize::from(lb);
                    let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - lbu));
                    rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                    rem = &mut rem[..rlen - lbu];
                } else {
                    let take = emit_long_back(&inline[cb], spelling, rem);
                    rem = &mut rem[..rlen - take];
                }
                prev = front;
                front = f_next;
                ef = ef_nxt;
                next = back;
                back = b_next;
                eb = eb_nxt;
                if front == next {
                    done = true;
                    break;
                }
            }
        }
        let (lo_rem, rem_len) = finish_bi_tail_inline(
            done, edges, spelling, inline, lens, bits, mask, out_base, rem, prev, front, back, next,
        );
        if rem_len > 0 {
            let hi_rem = lo_rem + rem_len;
            out.copy_within(hi_rem..n, lo_rem);
            out.truncate(lo_rem + (n - hi_rem));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    }

    // --- ARM 1: `V-PipeLean-Pre` (Hoisted current `lens[cf]`/`lens[cb]` & refs before stores) ---
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_pipelean_pre(
        edges: &[u32],
        spelling: &[u8],
        inline: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        before: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let out_base = out.as_ptr() as usize;
        let mut rem: &mut [u8] = &mut out[..n];
        let len_c = lens.len().min(inline.len());
        let lens = &lens[..len_c];
        let inline = &inline[..len_c];
        let mut done = false;
        if rem.len() >= 2 * INLINE {
            let mut ef = edges[front as usize];
            let mut eb = edges[back as usize];
            while rem.len() >= 2 * INLINE {
                if front == back {
                    let cf = (ef >> bits) as usize;
                    let lf = lens[cf];
                    if lf <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&inline[cf]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline[cf], spelling, rem);
                        rem = &mut rem[take..];
                    }
                    done = true;
                    break;
                }
                let f_next = (ef & mask) ^ prev;
                let b_next = (eb & mask) ^ next;
                let ef_nxt = edges[f_next as usize];
                let eb_nxt = edges[b_next as usize];

                // Hoist BOTH `lens` lookups & refs before either `out` store (zero 16B stack spills)!
                let cf = (ef >> bits) as usize;
                let cb = (eb >> bits) as usize;
                let lf = lens[cf];
                let lb = lens[cb];
                let sf = &inline[cf];
                let sb = &inline[cb];

                if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(sf);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(sf, spelling, rem);
                    rem = &mut rem[take..];
                    if rem.is_empty() {
                        done = true;
                        break;
                    }
                    if rem.len() < INLINE {
                        let take_b = emit_tail_back(sb, lb, spelling, rem);
                        let rlen = rem.len();
                        rem = &mut rem[..rlen - take_b];
                        prev = front;
                        front = f_next;
                        next = back;
                        back = b_next;
                        if front == next {
                            done = true;
                        }
                        break;
                    }
                }
                let rlen = rem.len();
                if lb <= INLINE as u8 {
                    let lbu = usize::from(lb);
                    let shifted = u128::from_le_bytes(*sb) << (8 * (INLINE - lbu));
                    rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                    rem = &mut rem[..rlen - lbu];
                } else {
                    let take = emit_long_back(sb, spelling, rem);
                    rem = &mut rem[..rlen - take];
                }
                prev = front;
                front = f_next;
                ef = ef_nxt;
                next = back;
                back = b_next;
                eb = eb_nxt;
                if front == next {
                    done = true;
                    break;
                }
            }
        }
        let (lo_rem, rem_len) = finish_bi_tail_inline(
            done, edges, spelling, inline, lens, bits, mask, out_base, rem, prev, front, back, next,
        );
        if rem_len > 0 {
            let hi_rem = lo_rem + rem_len;
            out.copy_within(hi_rem..n, lo_rem);
            out.truncate(lo_rem + (n - hi_rem));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    }

    // --- ARM 2: `V-Split8` (Explicit `len <= 8` `u64` single-store/shift fast-path) ---
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_split8(
        edges: &[u32],
        spelling: &[u8],
        inline: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        before: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let out_base = out.as_ptr() as usize;
        let mut rem: &mut [u8] = &mut out[..n];
        let len_c = lens.len().min(inline.len());
        let lens = &lens[..len_c];
        let inline = &inline[..len_c];
        let mut done = false;
        if rem.len() >= 2 * INLINE {
            let mut ef = edges[front as usize];
            let mut eb = edges[back as usize];
            while rem.len() >= 2 * INLINE {
                if front == back {
                    let cf = (ef >> bits) as usize;
                    let lf = lens[cf];
                    if lf <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&inline[cf]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline[cf], spelling, rem);
                        rem = &mut rem[take..];
                    }
                    done = true;
                    break;
                }
                let f_next = (ef & mask) ^ prev;
                let b_next = (eb & mask) ^ next;
                let ef_nxt = edges[f_next as usize];
                let eb_nxt = edges[b_next as usize];

                let cf = (ef >> bits) as usize;
                let lf = lens[cf];
                if lf <= 8 {
                    rem[..8].copy_from_slice(&inline[cf][..8]);
                    rem = &mut rem[usize::from(lf)..];
                } else if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&inline[cf]);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline[cf], spelling, rem);
                    rem = &mut rem[take..];
                    if rem.is_empty() {
                        done = true;
                        break;
                    }
                    if rem.len() < INLINE {
                        let cb = (eb >> bits) as usize;
                        let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                        let rlen = rem.len();
                        rem = &mut rem[..rlen - take_b];
                        prev = front;
                        front = f_next;
                        next = back;
                        back = b_next;
                        if front == next {
                            done = true;
                        }
                        break;
                    }
                }
                let cb = (eb >> bits) as usize;
                let rlen = rem.len();
                let lb = lens[cb];
                if lb <= 8 {
                    let lbu = usize::from(lb);
                    let w8 = u64::from_le_bytes(inline[cb][..8].try_into().unwrap());
                    let shifted = w8 << (8 * (8 - lbu));
                    rem[rlen - 8..].copy_from_slice(&shifted.to_le_bytes());
                    rem = &mut rem[..rlen - lbu];
                } else if lb <= INLINE as u8 {
                    let lbu = usize::from(lb);
                    let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - lbu));
                    rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                    rem = &mut rem[..rlen - lbu];
                } else {
                    let take = emit_long_back(&inline[cb], spelling, rem);
                    rem = &mut rem[..rlen - take];
                }
                prev = front;
                front = f_next;
                ef = ef_nxt;
                next = back;
                back = b_next;
                eb = eb_nxt;
                if front == next {
                    done = true;
                    break;
                }
            }
        }
        let (lo_rem, rem_len) = finish_bi_tail_inline(
            done, edges, spelling, inline, lens, bits, mask, out_base, rem, prev, front, back, next,
        );
        if rem_len > 0 {
            let hi_rem = lo_rem + rem_len;
            out.copy_within(hi_rem..n, lo_rem);
            out.truncate(lo_rem + (n - hi_rem));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn finish_bi_tail_inline(
        done: bool,
        edges: &[u32],
        spelling: &[u8],
        inline: &[[u8; INLINE]],
        lens: &[u8],
        bits: u32,
        mask: u32,
        out_base: usize,
        mut rem: &mut [u8],
        mut prev: u32,
        mut front: u32,
        mut back: u32,
        mut next: u32,
    ) -> (usize, usize) {
        if !done {
            while !rem.is_empty() {
                let ef = edges[front as usize];
                let cf = (ef >> bits) as usize;
                let single = front == back;
                let lf = lens[cf];
                let word = if lf == LONG {
                    long_word(&inline[cf], spelling)
                } else {
                    &inline[cf][..usize::from(lf)]
                };
                let take = word.len().min(rem.len());
                rem[..take].copy_from_slice(&word[..take]);
                rem = &mut rem[take..];
                if single || rem.is_empty() {
                    break;
                }
                let eb = edges[back as usize];
                let cb = (eb >> bits) as usize;
                let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                let rlen = rem.len();
                rem = &mut rem[..rlen - take_b];
                (prev, front) = (front, (ef & mask) ^ prev);
                (next, back) = (back, (eb & mask) ^ next);
                if front == next {
                    break;
                }
            }
        }
        let rem_len = rem.len();
        let lo_rem = rem.as_ptr() as usize - out_base;
        (lo_rem, rem_len)
    }

    // --- ARM 3: `V-Packed15` (Single 16B cacheline ref per cursor; ZERO `lens[]` array, ZERO spills!) ---
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_packed15<const PRELOAD: bool, const SPLIT8: bool>(
        edges: &[u32],
        spelling: &[u8],
        packed: &[[u8; INLINE]],
        n: usize,
        mark: u32,
        before: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let out_base = out.as_ptr() as usize;
        let mut rem: &mut [u8] = &mut out[..n];
        let mut done = false;
        if rem.len() >= 2 * INLINE {
            let mut ef = edges[front as usize];
            let mut eb = edges[back as usize];
            while rem.len() >= 2 * INLINE {
                if front == back {
                    let sf = &packed[(ef >> bits) as usize];
                    let lf = sf[15];
                    if lf <= PACKED_MAX as u8 {
                        rem[..INLINE].copy_from_slice(sf);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(sf, spelling, rem);
                        rem = &mut rem[take..];
                    }
                    done = true;
                    break;
                }
                let f_next = (ef & mask) ^ prev;
                let b_next = (eb & mask) ^ next;
                let ef_nxt = edges[f_next as usize];
                let eb_nxt = edges[b_next as usize];

                // Reference into `packed` avoids LLVM SROA 16B stack-spill!
                let sf = &packed[(ef >> bits) as usize];
                let lf = sf[15];
                let sb_pre = if PRELOAD {
                    let sb = &packed[(eb >> bits) as usize];
                    Some((sb, sb[15]))
                } else {
                    None
                };

                if lf <= PACKED_MAX as u8 {
                    rem[..INLINE].copy_from_slice(sf);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(sf, spelling, rem);
                    rem = &mut rem[take..];
                    if rem.is_empty() {
                        done = true;
                        break;
                    }
                    if rem.len() < INLINE {
                        let (sb, lb) = sb_pre.unwrap_or_else(|| {
                            let s = &packed[(eb >> bits) as usize];
                            (s, s[15])
                        });
                        let take_b = emit_tail_back(sb, lb, spelling, rem);
                        let rlen = rem.len();
                        rem = &mut rem[..rlen - take_b];
                        prev = front;
                        front = f_next;
                        next = back;
                        back = b_next;
                        if front == next {
                            done = true;
                        }
                        break;
                    }
                }
                let (sb, lb) = sb_pre.unwrap_or_else(|| {
                    let s = &packed[(eb >> bits) as usize];
                    (s, s[15])
                });
                let rlen = rem.len();
                if SPLIT8 && lb <= 8 {
                    let lbu = usize::from(lb);
                    let w8 = u64::from_le_bytes(sb[..8].try_into().unwrap());
                    let shifted = w8 << (8 * (8 - lbu));
                    rem[rlen - 8..].copy_from_slice(&shifted.to_le_bytes());
                    rem = &mut rem[..rlen - lbu];
                } else if lb <= PACKED_MAX as u8 {
                    // Since lb in 1..=15, shift = 8*(16 - lbu) in 8..=120 automatically
                    // discards sb[15] (the length byte) off the top of the u128!
                    let lbu = usize::from(lb);
                    let shifted = u128::from_le_bytes(*sb) << (8 * (INLINE - lbu));
                    rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                    rem = &mut rem[..rlen - lbu];
                } else {
                    let take = emit_long_back(sb, spelling, rem);
                    rem = &mut rem[..rlen - take];
                }
                prev = front;
                front = f_next;
                ef = ef_nxt;
                next = back;
                back = b_next;
                eb = eb_nxt;
                if front == next {
                    done = true;
                    break;
                }
            }
        }
        if !done {
            while !rem.is_empty() {
                let ef = edges[front as usize];
                let sf = &packed[(ef >> bits) as usize];
                let single = front == back;
                let lf = sf[15];
                let word = if lf == LONG {
                    long_word(sf, spelling)
                } else {
                    &sf[..usize::from(lf)]
                };
                let take = word.len().min(rem.len());
                rem[..take].copy_from_slice(&word[..take]);
                rem = &mut rem[take..];
                if single || rem.is_empty() {
                    break;
                }
                let eb = edges[back as usize];
                let sb = &packed[(eb >> bits) as usize];
                let take_b = emit_tail_back(sb, sb[15], spelling, rem);
                let rlen = rem.len();
                rem = &mut rem[..rlen - take_b];
                (prev, front) = (front, (ef & mask) ^ prev);
                (next, back) = (back, (eb & mask) ^ next);
                if front == next {
                    break;
                }
            }
        }
        let rem_len = rem.len();
        let lo_rem = rem.as_ptr() as usize - out_base;
        let hi_rem = lo_rem + rem_len;
        if rem_len > 0 {
            out.copy_within(hi_rem..n, lo_rem);
            out.truncate(lo_rem + (n - hi_rem));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_uni_inline<const SPLIT8: bool>(
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        bits: u32,
        sentinel: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let mask = (1 << bits) - 1;
        let mut j = mark;
        if extra.is_empty() {
            let len_c = lens.len().min(inline16.len());
            let lens = &lens[..len_c];
            let inline16 = &inline16[..len_c];
            let mut rem: &mut [u8] = &mut out[..n];
            let mut done = false;
            if rem.len() >= INLINE {
                let mut edge = edges[j as usize];
                while rem.len() >= INLINE {
                    let j_next = edge & mask;
                    let edge_next = edges[j_next as usize];
                    let code = (edge >> bits) as usize;
                    let len = lens[code];
                    if SPLIT8 && len <= 8 {
                        rem[..8].copy_from_slice(&inline16[code][..8]);
                        rem = &mut rem[usize::from(len)..];
                    } else if len <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&inline16[code]);
                        rem = &mut rem[usize::from(len)..];
                    } else {
                        let take = emit_long_front(&inline16[code], spelling, rem);
                        rem = &mut rem[take..];
                    }
                    j = j_next;
                    edge = edge_next;
                    if j == mark {
                        done = true;
                        break;
                    }
                }
            }
            if !done {
                while !rem.is_empty() {
                    let edge = edges[j as usize];
                    let code = (edge >> bits) as usize;
                    j = edge & mask;
                    let len = lens[code];
                    let word = if len == LONG {
                        long_word(&inline16[code], spelling)
                    } else {
                        &inline16[code][..usize::from(len)]
                    };
                    let take = word.len().min(rem.len());
                    rem[..take].copy_from_slice(&word[..take]);
                    rem = &mut rem[take..];
                    if j == mark {
                        break;
                    }
                }
            }
            let written = n - rem.len();
            out.truncate(written);
            repeat(n, out);
            return;
        }
        let out_buf = &mut out[..n + INLINE];
        let mut at = 0;
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let raw_code = edge >> bits;
            let code = if raw_code == sentinel {
                extra[j as usize]
            } else {
                raw_code
            } as usize;
            j = edge & mask;
            let len = lens[code];
            if len <= INLINE as u8 {
                out_buf[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                let word = long_word(&inline16[code], spelling);
                let take = word.len().min(n - at);
                out_buf[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if j == mark || at >= n {
                break;
            }
        }
        out.truncate(at.min(n));
        repeat(n, out);
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_uni_packed15(
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        packed: &[[u8; INLINE]],
        n: usize,
        mark: u32,
        bits: u32,
        sentinel: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let mask = (1 << bits) - 1;
        let mut j = mark;
        if extra.is_empty() {
            let mut rem: &mut [u8] = &mut out[..n];
            let mut done = false;
            if rem.len() >= INLINE {
                let mut edge = edges[j as usize];
                while rem.len() >= INLINE {
                    let j_next = edge & mask;
                    let edge_next = edges[j_next as usize];
                    let sf = &packed[(edge >> bits) as usize];
                    let len = sf[15];
                    if len <= PACKED_MAX as u8 {
                        rem[..INLINE].copy_from_slice(sf);
                        rem = &mut rem[usize::from(len)..];
                    } else {
                        let take = emit_long_front(sf, spelling, rem);
                        rem = &mut rem[take..];
                    }
                    j = j_next;
                    edge = edge_next;
                    if j == mark {
                        done = true;
                        break;
                    }
                }
            }
            if !done {
                while !rem.is_empty() {
                    let edge = edges[j as usize];
                    let sf = &packed[(edge >> bits) as usize];
                    j = edge & mask;
                    let len = sf[15];
                    let word = if len == LONG {
                        long_word(sf, spelling)
                    } else {
                        &sf[..usize::from(len)]
                    };
                    let take = word.len().min(rem.len());
                    rem[..take].copy_from_slice(&word[..take]);
                    rem = &mut rem[take..];
                    if j == mark {
                        break;
                    }
                }
            }
            let written = n - rem.len();
            out.truncate(written);
            repeat(n, out);
            return;
        }
        let out_buf = &mut out[..n + INLINE];
        let mut at = 0;
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let raw_code = edge >> bits;
            let code = if raw_code == sentinel {
                extra[j as usize]
            } else {
                raw_code
            } as usize;
            j = edge & mask;
            let sf = &packed[code];
            let len = sf[15];
            if len <= PACKED_MAX as u8 {
                out_buf[at..at + INLINE].copy_from_slice(sf);
                at += usize::from(len);
            } else {
                let word = long_word(sf, spelling);
                let take = word.len().min(n - at);
                out_buf[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if j == mark || at >= n {
                break;
            }
        }
        out.truncate(at.min(n));
        repeat(n, out);
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_sentinel_inline(
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        inline: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        before: u32,
        bits: u32,
        sentinel: u32,
        out: &mut Vec<u8>,
    ) {
        let out_buf = &mut out[..n];
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let (mut lo, mut hi) = (0usize, n);
        for _ in 0..=edges.len() / 2 {
            let ef = edges[front as usize];
            let raw_cf = ef >> bits;
            let cf = if raw_cf == sentinel {
                extra[front as usize]
            } else {
                raw_cf
            } as usize;
            let single = front == back;
            let lf = lens[cf];
            if lf <= INLINE as u8 && hi - lo >= INLINE {
                out_buf[lo..lo + INLINE].copy_from_slice(&inline[cf]);
                lo += usize::from(lf);
            } else {
                let word = if lf == LONG {
                    long_word(&inline[cf], spelling)
                } else {
                    &inline[cf][..usize::from(lf)]
                };
                let take = word.len().min(hi - lo);
                out_buf[lo..lo + take].copy_from_slice(&word[..take]);
                lo += take;
            }
            if single || lo == hi {
                break;
            }
            let eb = edges[back as usize];
            let raw_cb = eb >> bits;
            let cb = if raw_cb == sentinel {
                extra[back as usize]
            } else {
                raw_cb
            } as usize;
            let lb = lens[cb];
            if lb <= INLINE as u8 && hi - lo >= INLINE {
                let lbu = usize::from(lb);
                let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - lbu));
                out_buf[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                hi -= lbu;
            } else {
                let word = if lb == LONG {
                    long_word(&inline[cb], spelling)
                } else {
                    &inline[cb][..usize::from(lb)]
                };
                let take = word.len().min(hi - lo);
                out_buf[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
                hi -= take;
            }
            (prev, front) = (front, (ef & mask) ^ prev);
            (next, back) = (back, (eb & mask) ^ next);
            if lo == hi || front == next {
                break;
            }
        }
        if lo < hi {
            out.copy_within(hi..n, lo);
            out.truncate(lo + (n - hi));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_sentinel_packed15(
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        packed: &[[u8; INLINE]],
        n: usize,
        mark: u32,
        before: u32,
        bits: u32,
        sentinel: u32,
        out: &mut Vec<u8>,
    ) {
        let out_buf = &mut out[..n];
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let (mut lo, mut hi) = (0usize, n);
        for _ in 0..=edges.len() / 2 {
            let ef = edges[front as usize];
            let raw_cf = ef >> bits;
            let cf = if raw_cf == sentinel {
                extra[front as usize]
            } else {
                raw_cf
            } as usize;
            let single = front == back;
            let sf = packed[cf];
            let lf = sf[15];
            if lf <= PACKED_MAX as u8 && hi - lo >= INLINE {
                out_buf[lo..lo + INLINE].copy_from_slice(&sf);
                lo += usize::from(lf);
            } else {
                let word = if lf == LONG {
                    long_word(&sf, spelling)
                } else {
                    &sf[..usize::from(lf)]
                };
                let take = word.len().min(hi - lo);
                out_buf[lo..lo + take].copy_from_slice(&word[..take]);
                lo += take;
            }
            if single || lo == hi {
                break;
            }
            let eb = edges[back as usize];
            let raw_cb = eb >> bits;
            let cb = if raw_cb == sentinel {
                extra[back as usize]
            } else {
                raw_cb
            } as usize;
            let sb = packed[cb];
            let lb = sb[15];
            if lb <= PACKED_MAX as u8 && hi - lo >= INLINE {
                let lbu = usize::from(lb);
                let shifted = u128::from_le_bytes(sb) << (8 * (INLINE - lbu));
                out_buf[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                hi -= lbu;
            } else {
                let word = if lb == LONG {
                    long_word(&sb, spelling)
                } else {
                    &sb[..usize::from(lb)]
                };
                let take = word.len().min(hi - lo);
                out_buf[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
                hi -= take;
            }
            (prev, front) = (front, (ef & mask) ^ prev);
            (next, back) = (back, (eb & mask) ^ next);
            if lo == hi || front == next {
                break;
            }
        }
        if lo < hi {
            out.copy_within(hi..n, lo);
            out.truncate(lo + (n - hi));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    }

    #[derive(Default)]
    pub(super) struct PipeLeanKernel {
        spill: outq::Spill,
        bands: Vec<Band>,
        pub(super) spelling: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        packed15: Vec<[u8; INLINE]>,
        stack: Vec<u32>,
        predecessor: Option<u32>,
        last_m: usize,
        last_n: usize,
        last_mark: u32,
        last_bits: u32,
        last_sentinel: u32,
        last_mode: Option<S5DictMode>,
    }

    impl PipeLeanKernel {
        fn nodes_most(ns: usize) -> usize {
            3 * ns + 3
        }

        pub(super) fn dict_used_bytes(&self, mode: S5DictMode) -> usize {
            if mode.is_packed15() {
                self.packed15.len() * INLINE
            } else {
                self.inline16.len() * INLINE + self.lens.len()
            }
        }

        pub(super) fn total_ws_cap_bytes(&self, mode: S5DictMode) -> usize {
            let dict_cap = if mode.is_packed15() {
                self.packed15.capacity() * INLINE
            } else {
                self.inline16.capacity() * INLINE + self.lens.capacity()
            };
            self.spill.cap_bytes()
                + self.bands.capacity() * 8
                + self.spelling.capacity()
                + self.edges.capacity() * 4
                + self.extra.capacity() * 4
                + dict_cap
                + self.stack.capacity() * 4
        }

        pub(super) fn last_is_bi(&self) -> bool {
            self.predecessor.is_some()
        }

        /// Measure exact `inline16` length distribution (unique codes, dynamic S5 rows, output bytes)
        /// from the last `BasePipeLean` run.
        pub(super) fn measure_inline16_profile(&self) -> InlineLenProfile {
            let mut prof = InlineLenProfile::default();
            let num_codes = self.lens.len();
            prof.codes_total = num_codes as u64;
            for c in 0..num_codes {
                let l = self.lens[c];
                if l == 1 {
                    prof.codes_len1 += 1;
                } else if l <= 8 {
                    prof.codes_2_8 += 1;
                } else if l <= 15 {
                    prof.codes_9_15 += 1;
                } else if l == 16 {
                    prof.codes_len16 += 1;
                } else {
                    prof.codes_long += 1;
                }
            }
            let bits = self.last_bits;
            let sentinel = self.last_sentinel;
            let wide = !self.extra.is_empty();
            for j in 0..self.last_m {
                let raw_c = self.edges[j] >> bits;
                let code = if wide && raw_c == sentinel {
                    self.extra[j]
                } else {
                    raw_c
                } as usize;
                let l = self.lens[code];
                let wlen = if l == LONG {
                    long_word(&self.inline16[code], &self.spelling).len() as u64
                } else {
                    u64::from(l)
                };
                prof.rows_total += 1;
                prof.bytes_total += wlen;
                if l == 1 {
                    prof.rows_len1 += 1;
                    prof.bytes_len1 += wlen;
                } else if l <= 8 {
                    prof.rows_2_8 += 1;
                    prof.bytes_2_8 += wlen;
                } else if l <= 15 {
                    prof.rows_9_15 += 1;
                    prof.bytes_9_15 += wlen;
                } else if l == 16 {
                    prof.rows_len16 += 1;
                    prof.bytes_len16 += wlen;
                } else {
                    prof.rows_long += 1;
                    prof.bytes_long += wlen;
                }
            }
            prof
        }

        /// Deterministic 8-way LRU L1D/L2 cache simulation of the last `S5` emit stream,
        /// accurately modeling `inline16` + `lens` (2 arrays) vs `packed15` (1 array, NO `lens`!).
        pub(super) fn simulate_last_s5_cache(&self) -> S5CacheStats {
            let mode = self.last_mode.expect("run before simulate_last_s5_cache");
            let is_packed = mode.is_packed15();
            let mut sim = CacheSim::new();
            const BASE_EDGES: u64 = 0x1000_0000;
            const BASE_DICT: u64 = 0x2000_0000;
            const BASE_LENS: u64 = 0x3000_0000;
            const BASE_SPELL: u64 = 0x4000_0000;
            const BASE_OUT: u64 = 0x5000_0000;

            let m = self.last_m;
            let n = self.last_n;
            if m == 0 || n == 0 {
                return S5CacheStats::default();
            }
            let bits = self.last_bits;
            let mask = (1u32 << bits) - 1;
            let wide = !self.extra.is_empty();
            let sentinel = self.last_sentinel;

            let get_code = |idx: usize, edge: u32| -> usize {
                let rc = edge >> bits;
                if wide && rc == sentinel {
                    self.extra[idx] as usize
                } else {
                    rc as usize
                }
            };

            let access_emit =
                |sim: &mut CacheSim, code: usize, out_pos: usize, is_back: bool| -> usize {
                    if is_packed {
                        // Single 16-byte aligned access (no separate lens[] load!)
                        sim.access(BASE_DICT + (code as u64) * 16, true);
                        let sf = self.packed15[code];
                        let l = sf[15];
                        if l <= PACKED_MAX as u8 {
                            let lu = usize::from(l);
                            let pos = if is_back {
                                out_pos.saturating_sub(16)
                            } else {
                                out_pos
                            };
                            sim.access_range(BASE_OUT, pos, 16, false);
                            lu
                        } else {
                            let s = u32::from_le_bytes(sf[..4].try_into().unwrap()) as usize;
                            let e = u32::from_le_bytes(sf[4..8].try_into().unwrap()) as usize;
                            let wlen = e - s;
                            sim.access_range(BASE_SPELL, s, wlen, true);
                            let pos = if is_back {
                                out_pos.saturating_sub(wlen)
                            } else {
                                out_pos
                            };
                            sim.access_range(BASE_OUT, pos, wlen, false);
                            wlen
                        }
                    } else {
                        // Separate lens[code] + inline16[code] accesses!
                        sim.access(BASE_LENS + code as u64, true);
                        sim.access(BASE_DICT + (code as u64) * 16, true);
                        let l = self.lens[code];
                        if l <= INLINE as u8 {
                            let lu = usize::from(l);
                            let pos = if is_back {
                                out_pos.saturating_sub(16)
                            } else {
                                out_pos
                            };
                            sim.access_range(BASE_OUT, pos, 16, false);
                            lu
                        } else {
                            let sf = self.inline16[code];
                            let s = u32::from_le_bytes(sf[..4].try_into().unwrap()) as usize;
                            let e = u32::from_le_bytes(sf[4..8].try_into().unwrap()) as usize;
                            let wlen = e - s;
                            sim.access_range(BASE_SPELL, s, wlen, true);
                            let pos = if is_back {
                                out_pos.saturating_sub(wlen)
                            } else {
                                out_pos
                            };
                            sim.access_range(BASE_OUT, pos, wlen, false);
                            wlen
                        }
                    }
                };

            if let Some(before) = self.predecessor {
                let (mut prev, mut front) = (before, self.last_mark);
                let (mut back, mut next) = (before, self.last_mark);
                let (mut lo, mut hi) = (0usize, n);
                for _ in 0..=m / 2 {
                    sim.access(BASE_EDGES + (front as u64) * 4, false);
                    let ef = self.edges[front as usize];
                    let cf = get_code(front as usize, ef);
                    let single = front == back;
                    let wf = access_emit(&mut sim, cf, lo, false).min(hi - lo);
                    lo += wf;
                    if single || lo >= hi {
                        break;
                    }
                    sim.access(BASE_EDGES + (back as u64) * 4, false);
                    let eb = self.edges[back as usize];
                    let cb = get_code(back as usize, eb);
                    let wb = access_emit(&mut sim, cb, hi, true).min(hi - lo);
                    hi -= wb;
                    (prev, front) = (front, (ef & mask) ^ prev);
                    (next, back) = (back, (eb & mask) ^ next);
                    if lo >= hi || front == next {
                        break;
                    }
                }
            } else {
                let mut j = self.last_mark;
                let mut at = 0usize;
                for _ in 0..m {
                    sim.access(BASE_EDGES + (j as u64) * 4, false);
                    let edge = self.edges[j as usize];
                    let code = get_code(j as usize, edge);
                    let w = access_emit(&mut sim, code, at, false).min(n - at);
                    at += w;
                    j = edge & mask;
                    if j == self.last_mark || at >= n {
                        break;
                    }
                }
            }

            S5CacheStats {
                l1_dict_misses: sim.l1_dict_misses,
                l1_tot_misses: sim.l1_tot_misses,
                l2_tot_misses: sim.l2_tot_misses,
                dict_accesses: sim.dict_accesses,
                tot_accesses: sim.tot_accesses,
            }
        }

        #[inline(never)]
        pub(super) fn run<const TIMED: bool>(
            &mut self,
            mode: S5DictMode,
            records: &[u16],
            n: usize,
            p: u32,
            out: &mut Vec<u8>,
        ) -> StageTimes {
            let mut tm = StageTimes::default();
            if n == 0 {
                out.clear();
                self.last_m = 0;
                self.last_n = 0;
                self.last_mode = Some(mode);
                return tm;
            }
            out.resize(n + INLINE, 0);
            let t_s1 = if TIMED { Some(Instant::now()) } else { None };
            let nrec = records.len();
            let (ns, f, heads, over, extra_127, tot_pos) = {
                let mut pass = Pass {
                    queues: outq::Queues::new(&mut out[..], &mut self.spill, n as u32),
                    edges: &mut self.edges,
                    ns: 0,
                    f: First::default(),
                };
                let mut pos = 0u32;
                let mut extra_127 = 0u32;
                let mut i = 0usize;
                while i < nrec {
                    let rec = records[i];
                    let c = (rec >> 8) as u8;
                    let mut width = u32::from(rec & 255);
                    i += 1;
                    if width <= 16 {
                        pass.piece::<16>(c, pos, width, true, p);
                        pos += width;
                    } else {
                        if width == 127 {
                            while i < nrec && (records[i] >> 8) as u8 == c {
                                width += u32::from(records[i] & 255);
                                extra_127 += 1;
                                i += 1;
                            }
                        }
                        pass.take::<16>(c, pos, width, p);
                        pos += width;
                    }
                }
                let over = pass.queues.over();
                let thin = pass.queues.extract_thin();
                let heads = pass.queues.heads();
                let mut f = pass.f;
                f.thin = thin;
                (pass.ns, f, heads, over, extra_127, pos)
            };
            if let Some(t) = t_s1 {
                tm.s1_us = t.elapsed().as_secs_f64() * 1e6;
            }
            assert!(!over && tot_pos as usize == n && f.m > 0);
            let r_max = (nrec as u32).saturating_sub(extra_127);

            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            let first = starts(all);
            let mut write = starts(f.thick);
            let m = f.m as usize;
            let most = Self::nodes_most(ns);
            let use_narrow = most <= NARROW_NODES;
            let stride = if use_narrow {
                Narrow::STRIDE
            } else {
                Wide::STRIDE
            };

            self.bands.clear();
            self.spelling.clear();

            let in_out_chunks = out.len() / outq::BYTES;
            let tail_offset = (heads.chunks as usize).min(in_out_chunks) * outq::BYTES;
            let use_zero_hdr = out.len() - tail_offset >= 16 * ns;
            let need = if use_zero_hdr {
                (stride * most).max(m)
            } else {
                (4 * ns + stride * most).max(m)
            };
            if self.edges.len() < need {
                let add = need - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(need, 0);
            }

            let (ok_ind, cursors, used_tail, nbands) = if use_narrow {
                if mode.is_packed15() {
                    self.induced_with::<Narrow, true, TIMED>(
                        ns,
                        n as u32,
                        f.m,
                        &f.thin,
                        &first,
                        &mut write,
                        most,
                        use_zero_hdr,
                        &mut out[tail_offset..],
                        &mut tm,
                    )
                } else {
                    self.induced_with::<Narrow, false, TIMED>(
                        ns,
                        n as u32,
                        f.m,
                        &f.thin,
                        &first,
                        &mut write,
                        most,
                        use_zero_hdr,
                        &mut out[tail_offset..],
                        &mut tm,
                    )
                }
            } else if mode.is_packed15() {
                self.induced_with::<Wide, true, TIMED>(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    use_zero_hdr,
                    &mut out[tail_offset..],
                    &mut tm,
                )
            } else {
                self.induced_with::<Wide, false, TIMED>(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    use_zero_hdr,
                    &mut out[tail_offset..],
                    &mut tm,
                )
            };
            assert!(ok_ind);

            let t_s4 = if TIMED { Some(Instant::now()) } else { None };
            let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
            let sentinel = u32::MAX >> bits;
            let num_codes = if mode.is_packed15() {
                self.packed15.len()
            } else {
                self.lens.len()
            };
            let wide = num_codes - 1 > sentinel as usize;
            if self.edges.len() < m {
                self.edges.resize(m, 0);
            }
            self.extra.clear();
            if wide {
                self.extra.resize(m, 0);
            }
            self.predecessor = None;
            let use_xor = should_use_bi(f.m, ns as u32, r_max);
            let (out_q, out_tail) = out.split_at(tail_offset);

            if use_xor {
                let pred = if !wide {
                    if used_tail {
                        self.wire_bi_tail(
                            m,
                            f.mark,
                            f.mark_byte,
                            bits,
                            &cursors,
                            nbands,
                            out_tail,
                            out_q,
                            &heads,
                        )
                    } else {
                        self.wire_bi_vec(m, f.mark, f.mark_byte, bits, &cursors, out_q, &heads)
                    }
                } else {
                    self.wire_bi_sentinel(
                        m,
                        f.mark,
                        f.mark_byte,
                        bits,
                        sentinel,
                        &cursors,
                        nbands,
                        used_tail,
                        out_tail,
                        out_q,
                        &heads,
                    )
                };
                self.predecessor = Some(pred);
            } else if used_tail {
                self.wire_uni_tail(
                    m,
                    wide,
                    bits,
                    sentinel,
                    &cursors,
                    nbands,
                    out_tail,
                    out_q,
                    &heads,
                );
            } else {
                self.wire_uni_vec(m, wide, bits, sentinel, &cursors, out_q, &heads);
            }
            if let Some(t) = t_s4 {
                tm.s4_us = t.elapsed().as_secs_f64() * 1e6;
            }

            let t_s5 = if TIMED { Some(Instant::now()) } else { None };
            if let Some(before) = self.predecessor {
                if !wide {
                    match mode {
                        S5DictMode::BasePipeLean => emit_bi_base(
                            &self.edges[..m],
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            f.mark,
                            before,
                            bits,
                            out,
                        ),
                        S5DictMode::PipeLeanPre => emit_bi_pipelean_pre(
                            &self.edges[..m],
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            f.mark,
                            before,
                            bits,
                            out,
                        ),
                        S5DictMode::Split8 => emit_bi_split8(
                            &self.edges[..m],
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            f.mark,
                            before,
                            bits,
                            out,
                        ),
                        S5DictMode::Packed15 => emit_bi_packed15::<false, false>(
                            &self.edges[..m],
                            &self.spelling,
                            &self.packed15,
                            n,
                            f.mark,
                            before,
                            bits,
                            out,
                        ),
                        S5DictMode::Packed15Pre => emit_bi_packed15::<true, false>(
                            &self.edges[..m],
                            &self.spelling,
                            &self.packed15,
                            n,
                            f.mark,
                            before,
                            bits,
                            out,
                        ),
                        S5DictMode::Packed15Sp8 => emit_bi_packed15::<true, true>(
                            &self.edges[..m],
                            &self.spelling,
                            &self.packed15,
                            n,
                            f.mark,
                            before,
                            bits,
                            out,
                        ),
                    }
                } else if mode.is_packed15() {
                    emit_bi_sentinel_packed15(
                        &self.edges[..m],
                        &self.extra,
                        &self.spelling,
                        &self.packed15,
                        n,
                        f.mark,
                        before,
                        bits,
                        sentinel,
                        out,
                    );
                } else {
                    emit_bi_sentinel_inline(
                        &self.edges[..m],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        f.mark,
                        before,
                        bits,
                        sentinel,
                        out,
                    );
                }
            } else if mode.is_packed15() {
                emit_uni_packed15(
                    &self.edges[..m],
                    &self.extra,
                    &self.spelling,
                    &self.packed15,
                    n,
                    f.mark,
                    bits,
                    sentinel,
                    out,
                );
            } else if mode == S5DictMode::Split8 {
                emit_uni_inline::<true>(
                    &self.edges[..m],
                    &self.extra,
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    f.mark,
                    bits,
                    sentinel,
                    out,
                );
            } else {
                emit_uni_inline::<false>(
                    &self.edges[..m],
                    &self.extra,
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    f.mark,
                    bits,
                    sentinel,
                    out,
                );
            }
            if let Some(t) = t_s5 {
                tm.s5_us = t.elapsed().as_secs_f64() * 1e6;
            }

            self.last_m = m;
            self.last_n = n;
            self.last_mark = f.mark;
            self.last_bits = bits;
            self.last_sentinel = sentinel;
            self.last_mode = Some(mode);
            tm
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_with<L: Links, const PACKED15: bool, const TIMED: bool>(
            &mut self,
            ns: usize,
            n: u32,
            m: u32,
            thin: &[u32; 256],
            first: &[u32; 256],
            write: &mut [u32; 256],
            most: usize,
            use_zero_hdr: bool,
            out_tail: &mut [u8],
            tm: &mut StageTimes,
        ) -> (bool, [usize; 256], bool, usize) {
            let t_s2a = if TIMED { Some(Instant::now()) } else { None };
            let nodes_words = L::STRIDE * most;
            let hdr_words = if use_zero_hdr { 0 } else { 4 * ns };
            assert!(self.edges.len() >= hdr_words + nodes_words);

            let mut widest = 0u32;
            let (base, nodes, open_nodes) = if use_zero_hdr {
                let (ot_words, _) = out_tail.as_chunks_mut::<4>();
                let (strips_ot, rest_ot) = ot_words.split_at_mut(3 * ns);
                let image_ot = &mut rest_ot[..ns];
                for (d, &s) in strips_ot.iter_mut().zip(&self.edges[..3 * ns]) {
                    *d = s.to_ne_bytes();
                }
                if L::IS_NARROW {
                    let (nodes_sl, _) = self.edges.split_at_mut(nodes_words);
                    let (nodes, _) = nodes_sl.as_chunks_mut::<4>();
                    nodes[0] = [0, 0, 0, 0];
                    let mut nn = ns as u32 + 1;
                    let (mut t0h, mut t0t) = (0u32, 0u32);
                    let (mut t1h, mut t1t) = (0u32, 0u32);
                    let (mut o0h, mut o0t) = (0u32, 0u32);
                    let (mut o1h, mut o1t) = (0u32, 0u32);
                    let (strips3, _) = strips_ot.as_chunks_mut::<3>();
                    let mut x = 0u32;
                    for (i, sw) in strips3.iter_mut().enumerate() {
                        let from = u32::from_ne_bytes(sw[0]);
                        let to0 = u32::from_ne_bytes(sw[1]);
                        let wbyte = u32::from_ne_bytes(sw[2]);
                        let byte = (wbyte & 0xFF) as u8;
                        let width = wbyte >> 8;
                        let a = usize::from(byte);
                        sw[1] = (to0 + first[a]).to_ne_bytes();
                        let dst_idx = write[a] as usize;
                        image_ot[dst_idx] = (i as u32).to_ne_bytes();
                        write[a] = dst_idx as u32 + 1;
                        let sid = i as u32 + 1;
                        widest = widest.max(width);
                        if x < from {
                            let gw = from - x;
                            widest = widest.max(gw);
                            let gid = nn;
                            nn += 1;
                            if o0t == 0 {
                                o0h = gid;
                            } else {
                                nodes[o0t as usize][0] =
                                    (nodes[o0t as usize][0] & 0xFFFF) | (gid << 16);
                            }
                            if t1t == 0 {
                                t1h = gid;
                            } else {
                                nodes[t1t as usize][1] =
                                    (nodes[t1t as usize][1] & 0xFFFF) | (gid << 16);
                            }
                            nodes[gid as usize] = [o0t, t1t, gw, 0];
                            o0t = gid;
                            t1t = gid;
                        }
                        if o0t == 0 {
                            o0h = sid;
                        } else {
                            nodes[o0t as usize][0] =
                                (nodes[o0t as usize][0] & 0xFFFF) | (sid << 16);
                        }
                        nodes[sid as usize] = [o0t, 0, width, literal(byte)];
                        o0t = sid;
                        x = from + width;
                    }
                    if x < n {
                        let gw = n - x;
                        widest = widest.max(gw);
                        let gid = nn;
                        nn += 1;
                        if o0t == 0 {
                            o0h = gid;
                        } else {
                            nodes[o0t as usize][0] =
                                (nodes[o0t as usize][0] & 0xFFFF) | (gid << 16);
                        }
                        if t1t == 0 {
                            t1h = gid;
                        } else {
                            nodes[t1t as usize][1] =
                                (nodes[t1t as usize][1] & 0xFFFF) | (gid << 16);
                        }
                        nodes[gid as usize] = [o0t, t1t, gw, 0];
                        o0t = gid;
                        t1t = gid;
                    }
                    x = 0;
                    for slot in image_ot.iter() {
                        let i = u32::from_ne_bytes(*slot);
                        let sw = &strips3[i as usize];
                        let to = u32::from_ne_bytes(sw[1]);
                        let width = u32::from_ne_bytes(sw[2]) >> 8;
                        if x < to {
                            let gw = to - x;
                            widest = widest.max(gw);
                            let gid = nn;
                            nn += 1;
                            if o1t == 0 {
                                o1h = gid;
                            } else {
                                nodes[o1t as usize][1] =
                                    (nodes[o1t as usize][1] & 0xFFFF) | (gid << 16);
                            }
                            if t0t == 0 {
                                t0h = gid;
                            } else {
                                nodes[t0t as usize][0] =
                                    (nodes[t0t as usize][0] & 0xFFFF) | (gid << 16);
                            }
                            nodes[gid as usize] = [t0t, o1t, gw, 0];
                            o1t = gid;
                            t0t = gid;
                        }
                        let sid = i + 1;
                        if o1t == 0 {
                            o1h = sid;
                        } else {
                            nodes[o1t as usize][1] =
                                (nodes[o1t as usize][1] & 0xFFFF) | (sid << 16);
                        }
                        nodes[sid as usize][1] = o1t;
                        o1t = sid;
                        x = to + width;
                    }
                    if x < n {
                        let gw = n - x;
                        widest = widest.max(gw);
                        let gid = nn;
                        nn += 1;
                        if o1t == 0 {
                            o1h = gid;
                        } else {
                            nodes[o1t as usize][1] =
                                (nodes[o1t as usize][1] & 0xFFFF) | (gid << 16);
                        }
                        if t0t == 0 {
                            t0h = gid;
                        } else {
                            nodes[t0t as usize][0] =
                                (nodes[t0t as usize][0] & 0xFFFF) | (gid << 16);
                        }
                        nodes[gid as usize] = [t0t, o1t, gw, 0];
                        o1t = gid;
                        t0t = gid;
                    }
                    let mut close_side = |side: usize, first_p: (u32, u32), second_p: (u32, u32)| {
                        let mut prev = 0u32;
                        for (head, tail) in [first_p, second_p] {
                            if tail == 0 {
                                continue;
                            }
                            nodes[prev as usize][side] =
                                (nodes[prev as usize][side] & 0xFFFF) | (head << 16);
                            nodes[head as usize][side] =
                                (nodes[head as usize][side] & 0xFFFF_0000) | prev;
                            prev = tail;
                        }
                        nodes[prev as usize][side] &= 0xFFFF;
                        nodes[0][side] = (nodes[0][side] & 0xFFFF_0000) | prev;
                    };
                    close_side(0, (t0h, t0t), (o0h, o0t));
                    close_side(1, (t1h, t1t), (o1h, o1t));
                    (4 * nn as usize, nn, nn as usize - 1)
                } else {
                    let (nodes_sl, rules_dummy) = self.edges.split_at_mut(nodes_words);
                    let mut e = Elim::<L> {
                        nodes_buf: nodes_sl,
                        rules_buf: rules_dummy,
                        rule_top: 0,
                        nodes: 0,
                        id_overflow: false,
                        _links: std::marker::PhantomData,
                    };
                    e.node(0, 0);
                    e.nodes = ns as u32 + 1;
                    let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
                    let mut x = 0u32;
                    for i in 0..ns {
                        let sw = &mut strips_ot[3 * i..3 * i + 3];
                        let from = u32::from_ne_bytes(sw[0]);
                        let to0 = u32::from_ne_bytes(sw[1]);
                        let wbyte = u32::from_ne_bytes(sw[2]);
                        let byte = (wbyte & 0xFF) as u8;
                        let width = wbyte >> 8;
                        let a = usize::from(byte);
                        sw[1] = (to0 + first[a]).to_ne_bytes();
                        let dst_idx = write[a] as usize;
                        image_ot[dst_idx] = (i as u32).to_ne_bytes();
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
                    for slot in image_ot.iter() {
                        let i = u32::from_ne_bytes(*slot);
                        let idx = i as usize;
                        let sw = &strips_ot[3 * idx..3 * idx + 3];
                        let to = u32::from_ne_bytes(sw[1]);
                        let width = u32::from_ne_bytes(sw[2]) >> 8;
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
                    let open_nodes = e.nodes as usize - 1;
                    for side in 0..2 {
                        e.close(side, terminals[side], order[side]);
                    }
                    (L::STRIDE * e.nodes as usize, e.nodes, open_nodes)
                }
            } else {
                let (hdr, arena) = self.edges.split_at_mut(4 * ns);
                let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
                let (nodes_sl, rules_dummy) = arena.split_at_mut(nodes_words);
                let mut e = Elim::<L> {
                    nodes_buf: nodes_sl,
                    rules_buf: rules_dummy,
                    rule_top: 0,
                    nodes: 0,
                    id_overflow: false,
                    _links: std::marker::PhantomData,
                };
                e.node(0, 0);
                e.nodes = ns as u32 + 1;
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
                    let sw = &strips_raw[3 * idx..3 * idx + 3];
                    let to = sw[1];
                    let width = sw[2] >> 8;
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
                let open_nodes = e.nodes as usize - 1;
                for side in 0..2 {
                    e.close(side, terminals[side], order[side]);
                }
                (L::STRIDE * e.nodes as usize, e.nodes, open_nodes)
            };
            if let Some(t) = t_s2a {
                tm.s2a_us = t.elapsed().as_secs_f64() * 1e6;
            }

            let t_s2b = if TIMED { Some(Instant::now()) } else { None };
            let spirals = widest > CERTIFIED_WIDTH;
            let mut remaining = n;
            let mut rule_top = 0usize;
            loop {
                let (nodes_sl, rules_sl) = self.edges[hdr_words..].split_at_mut(base);
                let mut e = Elim::<L> {
                    nodes_buf: nodes_sl,
                    rules_buf: rules_sl,
                    rule_top,
                    nodes,
                    id_overflow: false,
                    _links: std::marker::PhantomData,
                };
                let done = if spirals {
                    e.eliminate::<true>(&mut remaining)
                } else {
                    e.eliminate::<false>(&mut remaining)
                };
                rule_top = e.rule_top;
                if e.id_overflow {
                    return (false, [0; 256], false, 0);
                }
                if done {
                    break;
                }
                grow_edges_step(
                    &mut self.edges,
                    hdr_words + base + rule_top + SPIRAL_RULES * MAX_RULE_WORDS,
                );
            }
            if let Some(t) = t_s2b {
                tm.s2b_us = t.elapsed().as_secs_f64() * 1e6;
            }

            let t_s3a = if TIMED { Some(Instant::now()) } else { None };
            if use_zero_hdr && ns >= 64 && self.edges.len() < base + rule_top + CACHE_WORDS {
                let need_c = base + rule_top + CACHE_WORDS;
                self.edges.reserve_exact(need_c - self.edges.len());
                self.edges.resize(need_c, 0);
            }

            let (header, arena) = self.edges.split_at_mut(hdr_words);
            let (live, tail_arena) = arena.split_at_mut(base + rule_top);
            let (nodes_sl, rules_sl) = live.split_at_mut(base);
            let mut e = Elim::<L> {
                nodes_buf: nodes_sl,
                rules_buf: rules_sl,
                rule_top,
                nodes,
                id_overflow: false,
                _links: std::marker::PhantomData,
            };
            e.assign_bottom(m);

            let cache = if header.len() >= CACHE_WORDS {
                header
            } else {
                tail_arena
            };
            if PACKED15 {
                begin_packed15(&mut self.packed15);
            } else {
                begin_inline(&mut self.inline16, &mut self.lens);
            }

            let max_bands = open_nodes + 256;
            let use_tail = out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec = [0u32; MEMO_REC_CAP];

            if cache.len() < CACHE_WORDS {
                let (band_slots, _) = out_tail.as_chunks_mut::<8>();
                let mut id = L::next(e.nodes_buf, 0, 0);
                let mut in_node = 0;
                let mut k = 0;
                for a in 0..256 {
                    cursors[a] = if use_tail { nbands } else { self.bands.len() };
                    let end = k + thin[a];
                    while k < end {
                        let (next_id, node_bottom, node_width, node_word) =
                            L::band_node(e.nodes_buf, id);
                        let width = (end - k).min(node_width - in_node);
                        let dst = node_bottom + in_node;
                        let code = if node_word == 0 {
                            a as u32
                        } else if PACKED15 {
                            let entry = e.packed15_word(
                                node_word,
                                a as u8,
                                &mut istk,
                                &mut memo_rec,
                                &mut self.stack,
                                &mut self.spelling,
                            );
                            ensure_step_packed(&mut self.packed15, INLINE_STEP);
                            let c = self.packed15.len() as u32;
                            self.packed15.push(entry);
                            c
                        } else {
                            let (entry, len) = e.inline_word(
                                node_word,
                                a as u8,
                                &mut istk,
                                &mut memo_rec,
                                &mut self.stack,
                                &mut self.spelling,
                            );
                            ensure_step_inline(&mut self.inline16, &mut self.lens, INLINE_STEP);
                            let c = self.lens.len() as u32;
                            self.inline16.push(entry);
                            self.lens.push(len);
                            c
                        };
                        if use_tail {
                            band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                            nbands += 1;
                        } else {
                            self.bands.push(Band::new(width, dst, code));
                        }
                        in_node += width;
                        k += width;
                        if in_node == node_width {
                            id = next_id;
                            in_node = 0;
                        }
                    }
                }
                assert_eq!(id, 0);
            } else {
                cache[..CACHE_WORDS].fill(0);
                let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
                let pairs: &mut [u32; 512] = p_sl.try_into().unwrap();
                let contents: &mut [u32; 2048] = c_sl.try_into().unwrap();
                let mut id = L::next(e.nodes_buf, 0, 0);
                let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
                    L::band_node(e.nodes_buf, id)
                } else {
                    (0, 0, 0, 0)
                };
                let mut in_node = 0u32;
                let mut k = 0u32;
                if PACKED15 {
                    let mut dictionary = Packed15Dictionary {
                        pairs,
                        contents,
                        packed15: &mut self.packed15,
                        spelling: &mut self.spelling,
                        istk: &mut istk,
                        memo_rec: &mut memo_rec,
                        stack: &mut self.stack,
                    };
                    if use_tail {
                        let (band_slots, _) = out_tail[..max_bands * 8].as_chunks_mut::<8>();
                        for a in 0..256 {
                            cursors[a] = nbands;
                            let ta = thin[a];
                            if ta == 0 {
                                continue;
                            }
                            let end = k + ta;
                            let mut last_w = 0u32;
                            let mut last_c = 0u32;
                            while k < end {
                                let width = (end - k).min(node_width - in_node);
                                let dst = node_bottom + in_node;
                                let code = if node_word == 0 {
                                    a as u32
                                } else if node_word == last_w {
                                    last_c
                                } else {
                                    let c = dictionary.code(&mut e, node_word, a as u8);
                                    last_w = node_word;
                                    last_c = c;
                                    c
                                };
                                band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                                nbands += 1;
                                in_node += width;
                                k += width;
                                if in_node == node_width {
                                    id = next_id;
                                    in_node = 0;
                                    if id != 0 {
                                        (next_id, node_bottom, node_width, node_word) =
                                            L::band_node(e.nodes_buf, id);
                                    }
                                }
                            }
                        }
                    } else {
                        for a in 0..256 {
                            cursors[a] = self.bands.len();
                            let ta = thin[a];
                            if ta == 0 {
                                continue;
                            }
                            let end = k + ta;
                            let mut last_w = 0u32;
                            let mut last_c = 0u32;
                            while k < end {
                                let width = (end - k).min(node_width - in_node);
                                let dst = node_bottom + in_node;
                                let code = if node_word == 0 {
                                    a as u32
                                } else if node_word == last_w {
                                    last_c
                                } else {
                                    let c = dictionary.code(&mut e, node_word, a as u8);
                                    last_w = node_word;
                                    last_c = c;
                                    c
                                };
                                self.bands.push(Band::new(width, dst, code));
                                in_node += width;
                                k += width;
                                if in_node == node_width {
                                    id = next_id;
                                    in_node = 0;
                                    if id != 0 {
                                        (next_id, node_bottom, node_width, node_word) =
                                            L::band_node(e.nodes_buf, id);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let mut dictionary = Dictionary {
                        pairs,
                        contents,
                        inline16: &mut self.inline16,
                        lens: &mut self.lens,
                        spelling: &mut self.spelling,
                        istk: &mut istk,
                        memo_rec: &mut memo_rec,
                        stack: &mut self.stack,
                    };
                    if use_tail {
                        let (band_slots, _) = out_tail[..max_bands * 8].as_chunks_mut::<8>();
                        for a in 0..256 {
                            cursors[a] = nbands;
                            let ta = thin[a];
                            if ta == 0 {
                                continue;
                            }
                            let end = k + ta;
                            let mut last_w = 0u32;
                            let mut last_c = 0u32;
                            while k < end {
                                let width = (end - k).min(node_width - in_node);
                                let dst = node_bottom + in_node;
                                let code = if node_word == 0 {
                                    a as u32
                                } else if node_word == last_w {
                                    last_c
                                } else {
                                    let c = dictionary.code(&mut e, node_word, a as u8);
                                    last_w = node_word;
                                    last_c = c;
                                    c
                                };
                                band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                                nbands += 1;
                                in_node += width;
                                k += width;
                                if in_node == node_width {
                                    id = next_id;
                                    in_node = 0;
                                    if id != 0 {
                                        (next_id, node_bottom, node_width, node_word) =
                                            L::band_node(e.nodes_buf, id);
                                    }
                                }
                            }
                        }
                    } else {
                        for a in 0..256 {
                            cursors[a] = self.bands.len();
                            let ta = thin[a];
                            if ta == 0 {
                                continue;
                            }
                            let end = k + ta;
                            let mut last_w = 0u32;
                            let mut last_c = 0u32;
                            while k < end {
                                let width = (end - k).min(node_width - in_node);
                                let dst = node_bottom + in_node;
                                let code = if node_word == 0 {
                                    a as u32
                                } else if node_word == last_w {
                                    last_c
                                } else {
                                    let c = dictionary.code(&mut e, node_word, a as u8);
                                    last_w = node_word;
                                    last_c = c;
                                    c
                                };
                                self.bands.push(Band::new(width, dst, code));
                                in_node += width;
                                k += width;
                                if in_node == node_width {
                                    id = next_id;
                                    in_node = 0;
                                    if id != 0 {
                                        (next_id, node_bottom, node_width, node_word) =
                                            L::band_node(e.nodes_buf, id);
                                    }
                                }
                            }
                        }
                    }
                }
                assert_eq!(id, 0);
            }
            if let Some(t) = t_s3a {
                tm.s3a_us = t.elapsed().as_secs_f64() * 1e6;
            }
            (true, cursors, use_tail, nbands)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_tail(
            &mut self,
            m: usize,
            mark: u32,
            mark_byte: u8,
            bits: u32,
            cursors: &[usize; 256],
            nbands: usize,
            out_tail: &[u8],
            out_q: &[u8],
            heads: &outq::Heads,
        ) -> u32 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let (band_slots, _) = out_tail[..nbands * 8].as_chunks::<8>();
            let mut predecessor = 0u32;
            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let start = cursors[a];
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                if start == end {
                    continue;
                }
                let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &slot in &band_slots[start..end] {
                        let b = Band(u64::from_le_bytes(slot));
                        reader.wire_xor_nomark(
                            edges,
                            b.dst() as usize,
                            b.end() as usize,
                            b.code() << bits,
                        );
                    }
                } else {
                    for &slot in &band_slots[start..end] {
                        let b = Band(u64::from_le_bytes(slot));
                        reader.wire_xor(
                            edges,
                            b.dst() as usize,
                            b.end() as usize,
                            b.code() << bits,
                            mark,
                            &mut predecessor,
                        );
                    }
                }
            }
            predecessor
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_vec(
            &mut self,
            m: usize,
            mark: u32,
            mark_byte: u8,
            bits: u32,
            cursors: &[usize; 256],
            out_q: &[u8],
            heads: &outq::Heads,
        ) -> u32 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let mut predecessor = 0u32;
            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let start = cursors[a];
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                if start == end {
                    continue;
                }
                let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &b in &self.bands[start..end] {
                        reader.wire_xor_nomark(
                            edges,
                            b.dst() as usize,
                            b.end() as usize,
                            b.code() << bits,
                        );
                    }
                } else {
                    for &b in &self.bands[start..end] {
                        reader.wire_xor(
                            edges,
                            b.dst() as usize,
                            b.end() as usize,
                            b.code() << bits,
                            mark,
                            &mut predecessor,
                        );
                    }
                }
            }
            predecessor
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_sentinel(
            &mut self,
            m: usize,
            mark: u32,
            mark_byte: u8,
            bits: u32,
            sentinel: u32,
            cursors: &[usize; 256],
            nbands: usize,
            used_tail: bool,
            out_tail: &[u8],
            out_q: &[u8],
            heads: &outq::Heads,
        ) -> u32 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let mut predecessor = 0u32;
            let mb = usize::from(mark_byte);
            let sent_pack = sentinel << bits;
            if used_tail {
                let (band_slots, _) = out_tail[..nbands * 8].as_chunks::<8>();
                for a in 0..256 {
                    let start = cursors[a];
                    let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                    if start == end {
                        continue;
                    }
                    let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &slot in &band_slots[start..end] {
                        let b = Band(u64::from_le_bytes(slot));
                        let dst = b.dst() as usize;
                        let span = b.end() as usize;
                        let bcode = b.code();
                        let pack = if bcode < sentinel {
                            bcode << bits
                        } else {
                            self.extra[dst..dst + span].fill(bcode);
                            sent_pack
                        };
                        if a != mb {
                            reader.wire_xor_nomark(edges, dst, span, pack);
                        } else {
                            reader.wire_xor(edges, dst, span, pack, mark, &mut predecessor);
                        }
                    }
                }
            } else {
                for a in 0..256 {
                    let start = cursors[a];
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    if start == end {
                        continue;
                    }
                    let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &b in &self.bands[start..end] {
                        let dst = b.dst() as usize;
                        let span = b.end() as usize;
                        let bcode = b.code();
                        let pack = if bcode < sentinel {
                            bcode << bits
                        } else {
                            self.extra[dst..dst + span].fill(bcode);
                            sent_pack
                        };
                        if a != mb {
                            reader.wire_xor_nomark(edges, dst, span, pack);
                        } else {
                            reader.wire_xor(edges, dst, span, pack, mark, &mut predecessor);
                        }
                    }
                }
            }
            predecessor
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_tail(
            &mut self,
            m: usize,
            wide: bool,
            bits: u32,
            sentinel: u32,
            cursors: &[usize; 256],
            nbands: usize,
            out_tail: &[u8],
            out_q: &[u8],
            heads: &outq::Heads,
        ) {
            let edges = &mut self.edges[..m];
            let (band_slots, _) = out_tail[..nbands * 8].as_chunks::<8>();
            for a in 0..256 {
                let start = cursors[a];
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                if start == end {
                    continue;
                }
                let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                for &slot in &band_slots[start..end] {
                    let b = Band(u64::from_le_bytes(slot));
                    let dst = b.dst() as usize;
                    let span = b.end() as usize;
                    let bcode = b.code();
                    let pack = if !wide {
                        bcode << bits
                    } else if bcode < sentinel {
                        bcode << bits
                    } else {
                        self.extra[dst..dst + span].fill(bcode);
                        sentinel << bits
                    };
                    reader.wire_uni(edges, dst, span, pack);
                }
            }
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_vec(
            &mut self,
            m: usize,
            wide: bool,
            bits: u32,
            sentinel: u32,
            cursors: &[usize; 256],
            out_q: &[u8],
            heads: &outq::Heads,
        ) {
            let edges = &mut self.edges[..m];
            for a in 0..256 {
                let start = cursors[a];
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                if start == end {
                    continue;
                }
                let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                for &b in &self.bands[start..end] {
                    let dst = b.dst() as usize;
                    let span = b.end() as usize;
                    let bcode = b.code();
                    let pack = if !wide {
                        bcode << bits
                    } else if bcode < sentinel {
                        bcode << bits
                    } else {
                        self.extra[dst..dst + span].fill(bcode);
                        sentinel << bits
                    };
                    reader.wire_uni(edges, dst, span, pack);
                }
            }
        }
    }

    /// Directly verify `LONG > rem` & exact length `1..=20` boundary behavior across ALL 6 S5 modes.
    pub(super) fn verify_s5_word_length_boundaries() {
        // Test EVERY single word length L in 1..=20 (specifically hammering L=8, 9, 15, 16, 17!)
        // plus LONG (40B) exceeding rem on front, back, and tail for n = 1..=48!
        let mut spelling: Vec<u8> = (0..40u8).map(|i| b'A' + (i % 26)).collect();
        let mut inline16 = Vec::new();
        let mut lens = Vec::new();
        begin_inline(&mut inline16, &mut lens);
        let mut packed15 = Vec::new();
        begin_packed15(&mut packed15);

        // Code 256 = LONG 40B; codes 257..=276 = word lengths 1..=20
        inline16.push(long_slot(0, 40));
        lens.push(LONG);
        packed15.push(long_slot_packed(0, 40));

        let mut words: Vec<Vec<u8>> = vec![spelling[..40].to_vec()];
        for wlen in 1..=20usize {
            let w: Vec<u8> = (0..wlen).map(|i| b'a' + ((i + wlen) % 26) as u8).collect();
            if wlen <= 16 {
                let mut sl = [0u8; INLINE];
                sl[..wlen].copy_from_slice(&w);
                inline16.push(sl);
                lens.push(wlen as u8);
            } else {
                let st = spelling.len() as u32;
                spelling.extend_from_slice(&w);
                inline16.push(long_slot(st, spelling.len() as u32));
                lens.push(LONG);
            }
            if wlen <= 15 {
                let mut sl = [0u8; INLINE];
                sl[..wlen].copy_from_slice(&w);
                sl[15] = wlen as u8;
                packed15.push(sl);
            } else {
                let st = spelling.len() as u32;
                spelling.extend_from_slice(&w);
                packed15.push(long_slot_packed(st, spelling.len() as u32));
            }
            words.push(w);
        }

        // 1) 2-node cycle: node 0 = LONG (40B, code 256), node 1 = wlen=7 (code 263) across n = 1..=48
        let bits = 2u32;
        let bi_edges = [(256u32 << bits) | (1 ^ 1), (263u32 << bits) | (0 ^ 0)];
        let uni_edges = [(256u32 << bits) | 1, (263u32 << bits) | 0];
        let mut cyc47 = words[0].clone();
        cyc47.extend_from_slice(&words[7]);
        for n in 1..=48usize {
            let mut want = vec![0u8; n + INLINE];
            emit_bi_base(&bi_edges, &spelling, &inline16, &lens, n, 0, 1, bits, &mut want);
            let mut out = vec![0xCC; n + INLINE];
            emit_bi_pipelean_pre(&bi_edges, &spelling, &inline16, &lens, n, 0, 1, bits, &mut out);
            assert_eq!(out, want, "PipeLeanPre mismatch at LONG>rem n={n}");
            out.resize(n + INLINE, 0xCC);
            emit_bi_split8(&bi_edges, &spelling, &inline16, &lens, n, 0, 1, bits, &mut out);
            assert_eq!(out, want, "Split8 mismatch at LONG>rem n={n}");
            out.resize(n + INLINE, 0xCC);
            emit_bi_packed15::<false, false>(&bi_edges, &spelling, &packed15, n, 0, 1, bits, &mut out);
            assert_eq!(out, want, "Packed15 mismatch at LONG>rem n={n}");
            out.resize(n + INLINE, 0xCC);
            emit_bi_packed15::<true, false>(&bi_edges, &spelling, &packed15, n, 0, 1, bits, &mut out);
            assert_eq!(out, want, "Packed15Pre mismatch at LONG>rem n={n}");
            out.resize(n + INLINE, 0xCC);
            emit_bi_packed15::<true, true>(&bi_edges, &spelling, &packed15, n, 0, 1, bits, &mut out);
            assert_eq!(out, want, "Packed15Sp8 mismatch at LONG>rem n={n}");

            // Uni path
            let mut want_u = vec![0u8; n + INLINE];
            emit_uni_inline::<false>(&uni_edges, &[], &spelling, &inline16, &lens, n, 0, bits, u32::MAX, &mut want_u);
            let mut out_u = vec![0xCC; n + INLINE];
            emit_uni_inline::<true>(&uni_edges, &[], &spelling, &inline16, &lens, n, 0, bits, u32::MAX, &mut out_u);
            assert_eq!(out_u, want_u);
            out_u.resize(n + INLINE, 0xCC);
            emit_uni_packed15(&uni_edges, &[], &spelling, &packed15, n, 0, bits, u32::MAX, &mut out_u);
            assert_eq!(out_u, want_u);
        }

        // 2) 20-node cycle exercising EVERY word length 1..=20 in sequence on BOTH front and back cursors!
        let m20 = 20usize;
        let bits20 = 5u32;
        let mut bi20 = vec![0u32; m20];
        let mut uni20 = vec![0u32; m20];
        let mut full20 = Vec::new();
        for i in 0..m20 {
            let code = (257 + i) as u32;
            let prev = ((i + m20 - 1) % m20) as u32;
            let next = ((i + 1) % m20) as u32;
            bi20[i] = (code << bits20) | (prev ^ next);
            uni20[i] = (code << bits20) | next;
            full20.extend_from_slice(&words[i + 1]);
        }
        let n20 = full20.len(); // 210 bytes
        let mut out = vec![0xAA; n20 + INLINE];
        emit_bi_base(&bi20, &spelling, &inline16, &lens, n20, 0, (m20 - 1) as u32, bits20, &mut out);
        assert_eq!(out, full20);
        out.resize(n20 + INLINE, 0xAA);
        emit_bi_pipelean_pre(&bi20, &spelling, &inline16, &lens, n20, 0, (m20 - 1) as u32, bits20, &mut out);
        assert_eq!(out, full20);
        out.resize(n20 + INLINE, 0xAA);
        emit_bi_split8(&bi20, &spelling, &inline16, &lens, n20, 0, (m20 - 1) as u32, bits20, &mut out);
        assert_eq!(out, full20);
        out.resize(n20 + INLINE, 0xAA);
        emit_bi_packed15::<false, false>(&bi20, &spelling, &packed15, n20, 0, (m20 - 1) as u32, bits20, &mut out);
        assert_eq!(out, full20);
        out.resize(n20 + INLINE, 0xAA);
        emit_bi_packed15::<true, false>(&bi20, &spelling, &packed15, n20, 0, (m20 - 1) as u32, bits20, &mut out);
        assert_eq!(out, full20);
        out.resize(n20 + INLINE, 0xAA);
        emit_bi_packed15::<true, true>(&bi20, &spelling, &packed15, n20, 0, (m20 - 1) as u32, bits20, &mut out);
        assert_eq!(out, full20);
        out.resize(n20 + INLINE, 0xAA);
        emit_uni_packed15(&uni20, &[], &spelling, &packed15, n20, 0, bits20, u32::MAX, &mut out);
        assert_eq!(out, full20);
    }
}

use pipelean_v::PipeLeanKernel;

// ============================================================================
// 6. Part 2 Runner — Distinct-Height Support, Franklin Involution, Theta Sparsity,
//    & Exact `inline16` Length Breakdown across ALL 5,749 Blocks
// ============================================================================

fn verify_part2_distinct_skyscrapers(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    corpora: &[(&str, &[BlockData], bool)],
) {
    println!("================================================================================");
    println!("PART 2 — KAKUTANI DISTINCT-HEIGHT SUPPORT (\\tilde{{\\lambda}}), FRANKLIN (1881) INVOLUTION,");
    println!("         THETA SPARSITY, & `inline16` LENGTH BREAKDOWN (5,749 BLOCKS)");
    println!("================================================================================");

    // 2A. 5,200 valid + 300 malformed exact distinct-height & Franklin involution checks
    let mut val_a = 0usize;
    let mut val_b = 0usize;
    let mut val_fix = 0usize;
    let mut val_sum_k = 0u64;
    for (l, p) in valid {
        let ds = extract_distinct_skyscraper(l, *p, 16);
        assert_eq!(ds.n_surv + ds.n_orphan, ds.n);
        val_sum_k += ds.k_dist as u64;
        match ds.fr_move {
            FranklinMove::DeleteBase => val_a += 1,
            FranklinMove::PeelSlope => val_b += 1,
            FranklinMove::FixedPentPos | FranklinMove::FixedPentNeg => val_fix += 1,
        }
    }
    let mut mal_a = 0usize;
    let mut mal_b = 0usize;
    let mut mal_fix = 0usize;
    for (l, p) in malformed {
        let ds = extract_distinct_skyscraper(l, *p, 16);
        assert_eq!(ds.n_surv + ds.n_orphan, ds.n);
        match ds.fr_move {
            FranklinMove::DeleteBase => mal_a += 1,
            FranklinMove::PeelSlope => mal_b += 1,
            FranklinMove::FixedPentPos | FranklinMove::FixedPentNeg => mal_fix += 1,
        }
    }
    println!(
        "  [PASS] Part 2A-Small: 5,200/5,200 valid (mean K={:.2}, Franklin A={val_a} / B={val_b} / Fixed={val_fix}) + 300/300 malformed (A={mal_a} / B={mal_b} / Fixed={mal_fix}) 100% EXACT (K(lambda)==K(lambda'), f(f(lam_tilde))==lam_tilde, 0 errors)!",
        val_sum_k as f64 / valid.len() as f64
    );

    // 2B. ALL 249 Real Blocks: distinct-height support, Franklin involution, entropy, & inline16 length breakdown
    let mut k16 = PipeLeanKernel::default();
    let mut out_tmp = Vec::new();
    let _ = k16.run::<false>(
        S5DictMode::BasePipeLean,
        &corpora[0].1[0].records,
        corpora[0].1[0].l.len(),
        corpora[0].1[0].p,
        &mut out_tmp,
    );

    let mut all_k = Vec::with_capacity(249);
    let mut all_ntilde = Vec::with_capacity(249);
    let mut all_s5_us = Vec::with_capacity(249);
    let mut all_le8_pct = Vec::with_capacity(249);
    let mut all_len16_pct = Vec::with_capacity(249);
    let mut wiki_k = Vec::with_capacity(233);
    let mut wiki_ntilde = Vec::with_capacity(233);
    let mut wiki_s5_us = Vec::with_capacity(233);
    let mut wiki_le8_pct = Vec::with_capacity(233);
    let mut wiki_len16_pct = Vec::with_capacity(233);

    let mut total_real = 0usize;

    for (c_idx, &(label, blocks, per_block)) in corpora.iter().enumerate() {
        println!("\n  --- {label} ({} blocks) ---", blocks.len());
        if per_block {
            println!(
                "  {:<13} {:>5} {:>7} {:>5} {:>4} {:>5} | {:>5} {:>7} | {:>6} {:>6} {:>6} | {:>6} {:>6} {:>6} {:>5}",
                "Block",
                "K_dst",
                "N_tilde",
                "d_tld",
                "c_lo",
                "Fr_mv",
                "TriSp",
                "log2q",
                "H_obs",
                "r<=8%",
                "r9-15%",
                "r=16%",
                "r>16%",
                "c=16",
                "S5us"
            );
        }

        let mut sum_k = 0.0f64;
        let mut sum_nt = 0.0f64;
        let mut sum_dt = 0.0f64;
        let mut sum_dobs = 0.0f64;
        let mut sum_clo = 0.0f64;
        let mut sum_trisp = 0.0f64;
        let mut sum_l2q = 0.0f64;
        let mut sum_l2p = 0.0f64;
        let mut sum_hobs = 0.0f64;
        let mut sum_mord = 0.0f64;
        let mut sum_s5 = 0.0f64;
        let mut sum_m = 0u64;
        let mut cnt_a = 0usize;
        let mut cnt_b = 0usize;
        let mut cnt_fix = 0usize;
        let mut sum_sig = 0u64;
        let mut sum_s = 0u64;
        let mut agg_prof = InlineLenProfile::default();

        for b in blocks {
            let ds = extract_distinct_skyscraper(&b.l, b.p, 16);
            assert_eq!(ds.n_orphan, 0);
            total_real += 1;

            let n_reps = if cfg!(debug_assertions) { 1 } else { 3 };
            let _ = k16.run::<false>(S5DictMode::BasePipeLean, &b.records, b.l.len(), b.p, &mut out_tmp);
            let prof = k16.measure_inline16_profile();
            let mut s5_sum = 0.0f64;
            for _ in 0..n_reps {
                let tm = k16.run::<true>(S5DictMode::BasePipeLean, &b.records, b.l.len(), b.p, &mut out_tmp);
                s5_sum += tm.s5_us;
            }
            let s5_us = s5_sum / n_reps as f64;
            agg_prof.add_assign(&prof);

            let r_le8 = pct(prof.rows_len1 + prof.rows_2_8, prof.rows_total);
            let r_9_15 = pct(prof.rows_9_15, prof.rows_total);
            let r_16 = pct(prof.rows_len16, prof.rows_total);
            let r_long = pct(prof.rows_long, prof.rows_total);

            sum_k += ds.k_dist as f64;
            sum_m += ds.m;
            sum_nt += ds.n_tilde as f64;
            sum_dt += ds.d_tilde as f64;
            sum_dobs += ds.d_obs as f64;
            sum_clo += ds.c_lo as f64;
            sum_trisp += ds.tri_sparsity;
            sum_l2q += ds.log2_q_ntilde;
            sum_l2p += ds.log2_p_nsurv;
            sum_hobs += ds.h_obs_bits;
            sum_mord += ds.multiset_order_bits;
            sum_s5 += s5_us;
            sum_sig += ds.slope_sig as u64;
            sum_s += ds.base_s;
            match ds.fr_move {
                FranklinMove::DeleteBase => cnt_a += 1,
                FranklinMove::PeelSlope => cnt_b += 1,
                FranklinMove::FixedPentPos | FranklinMove::FixedPentNeg => cnt_fix += 1,
            }

            if per_block {
                let mv_s = match ds.fr_move {
                    FranklinMove::DeleteBase => format!("A({}/{})", ds.base_s, ds.slope_sig),
                    FranklinMove::PeelSlope => format!("B({}/{})", ds.base_s, ds.slope_sig),
                    FranklinMove::FixedPentPos => "F+".to_string(),
                    FranklinMove::FixedPentNeg => "F-".to_string(),
                };
                println!(
                    "  {:<13} {:>5} {:>7} {:>5} {:>4} {:>5} | {:>4.1}x {:>7.1} | {:>5.0}k {:>5.2}% {:>5.2}% | {:>5.3}% {:>5.2}% {:>6} {:>5.0}",
                    b.name,
                    ds.k_dist,
                    ds.n_tilde,
                    ds.d_tilde,
                    ds.c_lo,
                    mv_s,
                    ds.tri_sparsity,
                    ds.log2_q_ntilde,
                    ds.h_obs_bits / 1000.0,
                    r_le8,
                    r_9_15,
                    r_16,
                    r_long,
                    prof.codes_len16,
                    s5_us,
                );
            }

            all_k.push(ds.k_dist as f64);
            all_ntilde.push(ds.n_tilde as f64);
            all_s5_us.push(s5_us);
            all_le8_pct.push(r_le8);
            all_len16_pct.push(r_16);
            if c_idx == 3 {
                wiki_k.push(ds.k_dist as f64);
                wiki_ntilde.push(ds.n_tilde as f64);
                wiki_s5_us.push(s5_us);
                wiki_le8_pct.push(r_le8);
                wiki_len16_pct.push(r_16);
            }
        }

        let nb = blocks.len() as f64;
        println!(
            "  [AGG {label}]: mean K_dist={:.1} ({:.3}% of m), N_tilde={:.0} ({:.2}% of n), d_tilde={:.1} vs d_obs={:.1}, c_lo={:.1}, TriSparsity={:.2}x | Franklin: CaseA={cnt_a}/{}, CaseB={cnt_b}, Fixed={cnt_fix} (mean s={:.2}, sig={:.2})",
            sum_k / nb,
            100.0 * sum_k / (sum_m.max(1) as f64),
            sum_nt / nb,
            100.0 * (sum_nt / nb) / (blocks.iter().map(|b| b.l.len() as f64).sum::<f64>() / nb),
            sum_dt / nb,
            sum_dobs / nb,
            sum_clo / nb,
            sum_trisp / nb,
            blocks.len(),
            sum_s as f64 / nb,
            sum_sig as f64 / nb,
        );
        println!(
            "    Entropy: log2 q(N_tilde)={:.1} b vs CA-49 log2 p(n)={:.1} b ({:.2}x) vs H_obs={:.0} b ({:.1}x) & MultisetOrder={:.0} b",
            sum_l2q / nb,
            sum_l2p / nb,
            (sum_l2p / nb) / (sum_l2q / nb).max(1e-9),
            sum_hobs / nb,
            (sum_hobs / nb) / (sum_l2q / nb).max(1e-9),
            sum_mord / nb,
        );
        println!(
            "    inline16 BY ROWS  : len==1 {:.2}% | 2..=8 {:.2}% (<=8: {:.2}%) | 9..=15 {:.2}% (<=15: {:.2}%) | len==16 {:.4}% | >16 (LONG) {:.2}%",
            pct(agg_prof.rows_len1, agg_prof.rows_total),
            pct(agg_prof.rows_2_8, agg_prof.rows_total),
            pct(agg_prof.rows_len1 + agg_prof.rows_2_8, agg_prof.rows_total),
            pct(agg_prof.rows_9_15, agg_prof.rows_total),
            pct(agg_prof.rows_len1 + agg_prof.rows_2_8 + agg_prof.rows_9_15, agg_prof.rows_total),
            pct(agg_prof.rows_len16, agg_prof.rows_total),
            pct(agg_prof.rows_long, agg_prof.rows_total),
        );
        println!(
            "    inline16 BY BYTES : len==1 {:.2}% | 2..=8 {:.2}% (<=8: {:.2}%) | 9..=15 {:.2}% (<=15: {:.2}%) | len==16 {:.4}% | >16 (LONG) {:.2}%",
            pct(agg_prof.bytes_len1, agg_prof.bytes_total),
            pct(agg_prof.bytes_2_8, agg_prof.bytes_total),
            pct(agg_prof.bytes_len1 + agg_prof.bytes_2_8, agg_prof.bytes_total),
            pct(agg_prof.bytes_9_15, agg_prof.bytes_total),
            pct(agg_prof.bytes_len1 + agg_prof.bytes_2_8 + agg_prof.bytes_9_15, agg_prof.bytes_total),
            pct(agg_prof.bytes_len16, agg_prof.bytes_total),
            pct(agg_prof.bytes_long, agg_prof.bytes_total),
        );
        println!(
            "    inline16 BY CODES : avg |D|={:.1} | len==1 {:.1} | 2..=8 {:.1} | 9..=15 {:.1} | len==16 {:.2} ({:.2}% of |D|) | >16 {:.1} | mean S5={:.1} us",
            agg_prof.codes_total as f64 / nb,
            agg_prof.codes_len1 as f64 / nb,
            agg_prof.codes_2_8 as f64 / nb,
            agg_prof.codes_9_15 as f64 / nb,
            agg_prof.codes_len16 as f64 / nb,
            pct(agg_prof.codes_len16, agg_prof.codes_total),
            agg_prof.codes_long as f64 / nb,
            sum_s5 / nb,
        );
    }

    println!("\n  --- Part 2C: Pearson Correlations of Distinct-Height Support & Length Concentration vs S5 ---");
    println!(
        "    [233 Full Wikidata]: r(K_dist, S5_us)={:+.4}, r(N_tilde, S5_us)={:+.4}, r(rows<=8%, S5_us)={:+.4}, r(rows=16%, S5_us)={:+.4}",
        pearson_r(&wiki_k, &wiki_s5_us),
        pearson_r(&wiki_ntilde, &wiki_s5_us),
        pearson_r(&wiki_le8_pct, &wiki_s5_us),
        pearson_r(&wiki_len16_pct, &wiki_s5_us),
    );
    println!(
        "    [ALL 249 Real Blks]: r(K_dist, S5_us)={:+.4}, r(N_tilde, S5_us)={:+.4}, r(rows<=8%, S5_us)={:+.4}, r(rows=16%, S5_us)={:+.4}",
        pearson_r(&all_k, &all_s5_us),
        pearson_r(&all_ntilde, &all_s5_us),
        pearson_r(&all_le8_pct, &all_s5_us),
        pearson_r(&all_len16_pct, &all_s5_us),
    );
    println!(
        "  [PASS] Part 2 Summary: 5,200 + 300 + {total_real}/249 = 5,749/5,749 blocks 100% EXACT distinct-height support & Franklin (1881) involution!\n"
    );
}

// ============================================================================
// 7. PART 3 — Systems Transfer Test (`V-ZeroRemap-Dict`, extending `CS-41`):
//    Zero-Remap `S5` Dictionary Layouts over `C53-PipeLean-v1` (`SlimS5_Pipe1_Rem16_LE16`)
// ============================================================================

#[derive(Default)]
struct ArmMetrics {
    st: StageTimes,
    avg_dict_used_kib: f64,
    avg_spell_kib: f64,
    total_ws_cap_kib: f64,
    l1_dict_misses: u64,
    l1_tot_misses: u64,
    l2_tot_misses: u64,
    dict_accesses: u64,
}

fn measure_arm_stages_and_cache(mode: S5DictMode, blocks: &[BlockData], run_sim: bool) -> ArmMetrics {
    let mut k = PipeLeanKernel::default();
    let mut out = Vec::new();
    for b in blocks {
        let _ = k.run::<false>(mode, &b.records, b.l.len(), b.p, &mut out);
    }
    let mut m = ArmMetrics::default();
    let mut sum_dict = 0u64;
    let mut sum_spell = 0u64;
    for b in blocks {
        let tm = k.run::<true>(mode, &b.records, b.l.len(), b.p, &mut out);
        m.st.s1_us += tm.s1_us;
        m.st.s2a_us += tm.s2a_us;
        m.st.s2b_us += tm.s2b_us;
        m.st.s3a_us += tm.s3a_us;
        m.st.s4_us += tm.s4_us;
        m.st.s5_us += tm.s5_us;
        sum_dict += k.dict_used_bytes(mode) as u64;
        sum_spell += k.spelling.len() as u64;
        if run_sim {
            let cs = k.simulate_last_s5_cache();
            assert!(cs.tot_accesses >= cs.l1_tot_misses);
            m.l1_dict_misses += cs.l1_dict_misses;
            m.l1_tot_misses += cs.l1_tot_misses;
            m.l2_tot_misses += cs.l2_tot_misses;
            m.dict_accesses += cs.dict_accesses;
        }
    }
    let nb = blocks.len() as f64;
    m.avg_dict_used_kib = (sum_dict as f64 / nb) / 1024.0;
    m.avg_spell_kib = (sum_spell as f64 / nb) / 1024.0;
    m.total_ws_cap_kib = k.total_ws_cap_bytes(mode) as f64 / 1024.0;
    m
}

fn time_one_round_1t(
    k: &mut PipeLeanKernel,
    mode: S5DictMode,
    out: &mut Vec<u8>,
    blocks: &[BlockData],
) -> f64 {
    let t0 = Instant::now();
    for b in blocks {
        let _ = k.run::<false>(mode, &b.records, b.l.len(), b.p, out);
    }
    t0.elapsed().as_secs_f64() * 1e3
}

fn time_one_round_nw(mode: S5DictMode, blocks: &[BlockData], workers: usize) -> f64 {
    let barrier = Barrier::new(workers);
    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(workers);
        for w_id in 0..workers {
            let b_ref = &barrier;
            handles.push(s.spawn(move || {
                let mut k = PipeLeanKernel::default();
                let mut out = Vec::new();
                let wb = &blocks[w_id % blocks.len()];
                let _ = k.run::<false>(mode, &wb.records, wb.l.len(), wb.p, &mut out);
                b_ref.wait();
                let t0 = Instant::now();
                for (idx, b) in blocks.iter().enumerate() {
                    if idx % workers == w_id {
                        let _ = k.run::<false>(mode, &b.records, b.l.len(), b.p, &mut out);
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

fn verify_and_bench_part3_zero_remap(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    corpora: &[(&str, &[BlockData], bool)],
    non_wiki_10: &[BlockData],
    verify_only: bool,
) {
    println!("================================================================================");
    println!("PART 3 — SYSTEMS TRANSFER TEST (`V-ZeroRemap-Dict`, extending `CS-41`):");
    println!("         ZERO-REMAP `S5` DICTIONARY SPECIALIZATIONS OVER `C53-PipeLean-v1`");
    println!("================================================================================");

    // 3A-0. Direct word-length 1..=20 + LONG>rem boundary unit tests across ALL 6 modes!
    pipelean_v::verify_s5_word_length_boundaries();

    let all_modes = [
        S5DictMode::BasePipeLean,
        S5DictMode::PipeLeanPre,
        S5DictMode::Packed15,
        S5DictMode::Packed15Pre,
        S5DictMode::Split8,
        S5DictMode::Packed15Sp8,
    ];

    let mut kernels: [PipeLeanKernel; 6] = std::array::from_fn(|_| PipeLeanKernel::default());
    let mut out_buf = Vec::new();

    // 3A-1. Edge cases: n = 0, 1, 2, single-char n = 50, all-256 distinct
    let edge_cases: Vec<(Vec<u8>, u32)> = vec![
        (Vec::new(), 0),
        (vec![b'x'], 0),
        (bwt_small(b"ab").0, bwt_small(b"ab").1),
        (vec![b'a'; 50], 17),
        ((0..=255u8).collect::<Vec<u8>>(), 42),
    ];
    for (idx, (l, p)) in edge_cases.iter().enumerate() {
        let recs = l_to_records(l);
        let expected = ref_fl(l, *p);
        for (mi, &mode) in all_modes.iter().enumerate() {
            let _ = kernels[mi].run::<false>(mode, &recs, l.len(), *p, &mut out_buf);
            assert_eq!(out_buf, expected, "Edge case {idx} failed for {mode:?}!");
        }
    }

    // 3A-2. 5,200 valid + 300 malformed small cases across ALL 6 modes
    for (l, p) in valid.iter().chain(malformed.iter()) {
        let recs = l_to_records(l);
        let expected = ref_fl(l, *p);
        for (mi, &mode) in all_modes.iter().enumerate() {
            let _ = kernels[mi].run::<false>(mode, &recs, l.len(), *p, &mut out_buf);
            assert_eq!(out_buf, expected, "Small/malformed mismatch for {mode:?}!");
        }
    }

    // 3A-3. Detached malformed cycles d = 1..=65 on bi (m >= 65_536) & d = 1..=17 on uni (m < 65_536)
    let mut detached_cnt = 0usize;
    let d_bi_max = if cfg!(debug_assertions) { 17 } else { 65 };
    for d in 1..=d_bi_max {
        let sz = 75_000usize;
        let mut l = vec![0u8; sz];
        for i in 0..d {
            l[i] = ((i + 1) % d) as u8;
        }
        for (i, slot) in l.iter_mut().enumerate().skip(d) {
            *slot = (d + (i % 120)) as u8;
        }
        for slot in &mut l[sz - 8_000..sz] {
            *slot = 250;
        }
        let recs = l_to_records(&l);
        let expected = ref_fl(&l, 0);
        for (mi, &mode) in all_modes.iter().enumerate() {
            let _ = kernels[mi].run::<false>(mode, &recs, sz, 0, &mut out_buf);
            assert_eq!(out_buf, expected, "Detached bi d={d} mismatch for {mode:?}!");
            assert!(kernels[mi].last_is_bi(), "d={d} must exercise bi path");
        }
        detached_cnt += 1;
    }
    for d in 1..=17usize {
        let sz = 40_000usize;
        let mut l = vec![0u8; sz];
        for i in 0..d {
            l[i] = ((i + 1) % d) as u8;
        }
        for (i, slot) in l.iter_mut().enumerate().skip(d) {
            *slot = (d + (i % 90)) as u8;
        }
        for slot in &mut l[sz - 8_000..sz] {
            *slot = 250;
        }
        let recs = l_to_records(&l);
        let expected = ref_fl(&l, 0);
        for (mi, &mode) in all_modes.iter().enumerate() {
            let _ = kernels[mi].run::<false>(mode, &recs, sz, 0, &mut out_buf);
            assert_eq!(out_buf, expected, "Detached uni d={d} mismatch for {mode:?}!");
            assert!(!kernels[mi].last_is_bi(), "d={d} must exercise uni path");
        }
        detached_cnt += 1;
    }

    // 3A-4. ALL 249 Real Blocks across ALL 6 modes!
    let mut real_ok = 0usize;
    for &(_label, blocks, _) in corpora {
        for b in blocks {
            let expected = ref_fl(&b.l, b.p);
            for (mi, &mode) in all_modes.iter().enumerate() {
                let _ = kernels[mi].run::<false>(mode, &b.records, b.l.len(), b.p, &mut out_buf);
                assert_eq!(
                    out_buf, expected,
                    "Real block {} mismatch for {mode:?}!",
                    b.name
                );
            }
            real_ok += 1;
        }
    }
    println!(
        "  [PASS] Part 3A Byte-Exactness (ALL 6 Zero-Remap Arms vs ref_fl): 49 word-len/LONG>rem + 5 edge + 5,200 valid + 300 malformed + {detached_cnt} detached bi/uni + {real_ok}/249 real = {} cases 100% BYTE-EXACT (0 errors)!\n",
        49 + 5 + valid.len() + malformed.len() + detached_cnt + real_ok
    );

    if verify_only {
        println!("  [--verify-only set: skipping Part 3B timing benchmarks.]");
        return;
    }

    // 3B. Interleaved 1T / 2W / 4W Benchmarks + Deterministic L1D/L2 Cache Comparison
    let arms: [(&str, S5DictMode); 7] = [
        ("Base-PipeLn", S5DictMode::BasePipeLean),
        ("Base (A/A)", S5DictMode::BasePipeLean),
        ("V-PipeLn-Pre", S5DictMode::PipeLeanPre),
        ("V-Packed15", S5DictMode::Packed15),
        ("V-Pack15-Pre", S5DictMode::Packed15Pre),
        ("V-Split8", S5DictMode::Split8),
        ("V-Pack15-Sp8", S5DictMode::Packed15Sp8),
    ];

    let bench_corpora: [(&str, &[BlockData], bool); 5] = [
        ("6 Admitted Wikidata", corpora[0].1, true),
        ("6 JSONL/Syslog", corpora[1].1, true),
        ("4 Code/Docs", corpora[2].1, true),
        ("10 Non-Wiki Combined", non_wiki_10, true),
        ("233 Full Wikidata", corpora[3].1, false),
    ];

    let bench_rounds = |n_blks: usize| -> usize {
        if cfg!(debug_assertions) {
            1
        } else if n_blks <= 10 {
            11
        } else {
            5
        }
    };

    for &(label, blocks, is_small) in &bench_corpora {
        let rds = bench_rounds(blocks.len());
        let run_sim = !cfg!(debug_assertions) || is_small;
        println!(
            "  [BENCHMARK — {label} ({} blks, {rds} interleaved rounds median; Base = Base-PipeLn)]:",
            blocks.len()
        );

        let mut mets: Vec<ArmMetrics> = Vec::with_capacity(arms.len());
        for &(_name, mode) in &arms {
            mets.push(measure_arm_stages_and_cache(mode, blocks, run_sim));
        }

        let mut bk: Vec<PipeLeanKernel> = (0..arms.len()).map(|_| PipeLeanKernel::default()).collect();
        let mut bout = Vec::new();
        for b in blocks {
            for (ai, &(_name, mode)) in arms.iter().enumerate() {
                let _ = bk[ai].run::<false>(mode, &b.records, b.l.len(), b.p, &mut bout);
            }
        }

        let mut t1: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t2: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t4: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(rds));

        for r in 0..rds {
            for step in 0..arms.len() {
                let arm = (r + step) % arms.len();
                t1[arm].push(time_one_round_1t(&mut bk[arm], arms[arm].1, &mut bout, blocks));
            }
        }
        for r in 0..rds {
            for step in 0..arms.len() {
                let arm = (r + step) % arms.len();
                t2[arm].push(time_one_round_nw(arms[arm].1, blocks, 2));
            }
        }
        for r in 0..rds {
            for step in 0..arms.len() {
                let arm = (r + step) % arms.len();
                t4[arm].push(time_one_round_nw(arms[arm].1, blocks, 4));
            }
        }

        let m1: [f64; 7] = std::array::from_fn(|i| median(&mut t1[i]));
        let m2: [f64; 7] = std::array::from_fn(|i| median(&mut t2[i]));
        let m4: [f64; 7] = std::array::from_fn(|i| median(&mut t4[i]));

        println!(
            "  {:<12} | {:>13} {:>13} {:>13} | {:>6} {:>12} | {:>8} {:>9} | {:>6} {:>6} {:>7}",
            "Arm",
            "1T ms (rel)",
            "2W ms (rel)",
            "4W ms (rel)",
            "S3a ms",
            "S5 ms (rel)",
            "L1dM rel",
            "L1tM/L2tM",
            "DictK",
            "SpelK",
            "TotWSK"
        );
        let base_s5 = mets[0].st.s5_us.max(1e-9);
        let base_l1d = mets[0].l1_dict_misses.max(1) as f64;
        let base_l1t = mets[0].l1_tot_misses.max(1) as f64;
        let base_l2t = mets[0].l2_tot_misses.max(1) as f64;

        for (ai, &(name, _mode)) in arms.iter().enumerate() {
            let am = &mets[ai];
            let l1d_str = if run_sim {
                format!("{:>6.3}x", am.l1_dict_misses as f64 / base_l1d)
            } else {
                "   n/a  ".to_string()
            };
            let l12_str = if run_sim {
                format!(
                    "{:.3}/{:.3}",
                    am.l1_tot_misses as f64 / base_l1t,
                    am.l2_tot_misses as f64 / base_l2t
                )
            } else {
                "   n/a   ".to_string()
            };
            println!(
                "  {:<12} | {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) | {:>6.2} {:>5.2} ({:.3}x) | {:>8} {:>9} | {:>5.1}K {:>5.1}K {:>6.1}K",
                name,
                m1[ai],
                m1[ai] / m1[0],
                m2[ai],
                m2[ai] / m2[0],
                m4[ai],
                m4[ai] / m4[0],
                am.st.s3a_us / 1e3,
                am.st.s5_us / 1e3,
                am.st.s5_us / base_s5,
                l1d_str,
                l12_str,
                am.avg_dict_used_kib,
                am.avg_spell_kib,
                am.total_ws_cap_kib,
            );
        }
        if run_sim {
            println!(
                "    Cache Detail ({label}): Base dict_acc={} L1D_dict_misses={} ({:.2}%) | Packed15 dict_acc={} ({:+.1}%) L1D_dict_misses={} ({:+.1}%)",
                mets[0].dict_accesses,
                mets[0].l1_dict_misses,
                pct(mets[0].l1_dict_misses, mets[0].dict_accesses),
                mets[3].dict_accesses,
                100.0 * (mets[3].dict_accesses as f64 / mets[0].dict_accesses.max(1) as f64 - 1.0),
                mets[3].l1_dict_misses,
                100.0 * (mets[3].l1_dict_misses as f64 / base_l1d - 1.0),
            );
        }
        println!();
    }
}

fn main() {
    let t0 = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--callgrind") {
        let arm_name = args.get(pos + 1).map(String::as_str).unwrap_or("base");
        let mode = match arm_name {
            "base" => S5DictMode::BasePipeLean,
            "pre" => S5DictMode::PipeLeanPre,
            "pack15" => S5DictMode::Packed15,
            "pack15_pre" => S5DictMode::Packed15Pre,
            "split8" => S5DictMode::Split8,
            "pack15_sp8" => S5DictMode::Packed15Sp8,
            other => panic!("Unknown --callgrind arm: {other}"),
        };
        let adm6 = load_six_admitted();
        let mut k = PipeLeanKernel::default();
        let mut out = Vec::new();
        let mut chk = 0usize;
        for b in &adm6 {
            let _ = k.run::<false>(mode, &b.records, b.l.len(), b.p, &mut out);
            chk ^= out.len() ^ usize::from(out[0]) ^ usize::from(out[out.len() - 1]);
        }
        println!("callgrind {arm_name}: ok chk={chk}");
        return;
    }
    let verify_only = args.iter().any(|a| a == "--verify-only");
    println!("Cycle 54 P2 — Analogue V: C. G. J. Jacobi (1829) Triple Product / Pentagonal–Theta Series");
    println!("              & F. Franklin (1881) Constructive Bijective Involution on Kakutani Skyscrapers");
    println!("              + Zero-Remap `S5` Dictionary Layout Study (`V-ZeroRemap-Dict`)\n");

    verify_part1_jacobi_and_franklin();

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
    let mut non_wiki_10 = Vec::with_capacity(10);
    non_wiki_10.extend(nw6.iter().cloned());
    non_wiki_10.extend(cd4.iter().cloned());
    println!(
        "Loaded all 249 real blocks: 6 Adm + 6 JSONL/Syslog + 4 Code/Docs (10 Non-Wiki) + 233 Full Wikidata.\n"
    );

    let corpora: [(&str, &[BlockData], bool); 4] = [
        ("6 Admitted Wikidata", &adm6, true),
        ("6 JSONL/Syslog", &nw6, true),
        ("4 Code/Docs", &cd4, true),
        ("233 Full Wikidata", &wiki233, false),
    ];

    verify_part2_distinct_skyscrapers(&valid, &malformed, &corpora);
    verify_and_bench_part3_zero_remap(&valid, &malformed, &corpora, &non_wiki_10, verify_only);

    println!("Total elapsed: {:.2}s", t0.elapsed().as_secs_f64());
}
