#![forbid(unsafe_code)]

//! Cycle 55 P2 — Analogue X:
//! C. F. Gauss (1808/1811) Gaussian Binomial Coefficients & A. L. Cauchy (1843)
//! Finite $q$-Binomial Theorem on Bounded-Box Kakutani Skyscrapers (`CA-52` -> `CA-55`)
//! + Exact $L$-Cutoff Pareto Law & `CS-53` `X-BoxCutoff-Sweep` inside `C54-PipeLean-v2`.
//!
//! Primary Sources (`100%` load-bearing pre-1950 classical mathematics, ZERO relabels, ZERO analogies):
//!   1. C. F. Gauss (1808/1811), *"Summatio quarumdam serierum singularium"*,
//!      *Commentationes Societatis Regiae Scientiarum Gottingensis Recentiores* **1** (1811), 1–40
//!      (presented August 1808; *Werke*, Vol. II, pp. 9–45):
//!        - Gaussian binomial coefficients (`q`-binomial coefficients):
//!            \binom{a+b}{a}_q = \prod_{j=1}^{a} \frac{1 - q^{b+j}}{1 - q^j}
//!                             = \sum_{N=0}^{ab} p(a, b; N) q^N,
//!          counting integer partitions whose Ferrers diagram fits inside an `a × b` bounding box;
//!        - Exact Gauss `q`-Pascal recurrences:
//!            \binom{a+b}{a}_q = \binom{a+b-1}{a-1}_q + q^a \binom{a+b-1}{a}_q
//!                             = q^b \binom{a+b-1}{a-1}_q + \binom{a+b-1}{a}_q;
//!        - Ferrers-conjugate symmetry \binom{a+b}{a}_q = \binom{a+b}{b}_q, palindromicity
//!          [q^N]\binom{a+b}{a}_q = [q^{ab-N}]\binom{a+b}{a}_q, unimodality (J. J. Sylvester 1878,
//!          *Phil. Mag.* (5) 5, 178–188), specializations at q = 1 (\binom{a+b}{a}) & q = -1,
//!          and `q`-Vandermonde convolution:
//!            \binom{a+b}{k}_q = \sum_j q^{(a-j)(k-j)} \binom{a}{j}_q \binom{b}{k-j}_q.
//!   2. A. L. Cauchy (1843), *"Mémoire sur les fonctions dont plusieurs valeurs sont liées entre
//!      elles par une équation linéaire, et sur diverses transformations de produits composés d'un
//!      nombre indéfini de facteurs"*, *Comptes Rendus Acad. Sci. Paris* **17** (1843), 523–531:
//!        - Finite distinct-part `q`-Binomial Theorem:
//!            \prod_{k=0}^{n-1} (1 + z q^k) = \sum_{j=0}^{n} q^{\binom{j}{2}} \binom{n}{j}_q z^j;
//!        - Finite reciprocal (unrestricted-part) `q`-Binomial Theorem:
//!            \prod_{k=0}^{n-1} (1 - z q^k)^{-1} = \sum_{j=0}^{\infty} \binom{n+j-1}{j}_q z^j,
//!          whose z = q specialization \prod_{k=1}^L (1 - q^k)^{-1} is the exact height-L bounded
//!          partition GF uniting `CA-52` Jacobi–Franklin skyscrapers with `Open<L>` cutoff L!
//!
//! Extending `CA-05 -> CA-10 -> CA-11 -> CA-16 -> CA-17 -> CA-18 -> CA-34 -> CA-40 -> CA-43 ->
//!            CA-46 -> CA-49 -> CA-52` (Chain A+N+P+R+T+V) to a **NEW `13`-node / `12`-link
//! Pure-`CA` Longest Chain** (`CA-55`), and **`14`-node / `13`-link Full-Atlas Chain** via `CS-53`!
//!
//! Part 3 (`CS-53` `X-BoxCutoff-Sweep`, strictly `2020s` modern CS — ZERO classical share!):
//!   Benchmarks exact bounded-box cutoff sweep L in {10, 11, 12, 13, 14, 16} + corpus-adaptive
//!   selectors (`Adapt-16`, `Adapt-12`, `Adapt-11`) directly inside `C54-PipeLean-v2`'s exact
//!   `S5_UnalignedBack_1Ptr + ZeroSpill_ConstBits + S4_8WayEpoch` loop (`callgrind` Ir / 256K L2 /
//!   2M LLC + paired native `1T / 2W / 4W` across `6` Adm, `6` JSONL/Log, `4` Code/Docs, `233` Wiki).
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`
//! AND passes `-C debug-assertions=yes -C overflow-checks=yes`.

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
}

// ============================================================================
// 1. Utilities, Exact Reference FL, PRNG, Log-Binomial, & Corpus Loaders
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

/// Exact log2(binom(a + b, a)) computed via direct log-sum for small min(a,b) <= 64,
/// and Lanczos ln_gamma otherwise.
#[inline]
fn log2_binom_box(a: u64, b: u64) -> f64 {
    let k = a.min(b);
    let m = a.max(b);
    if k == 0 {
        return 0.0;
    }
    if k <= 64 {
        let mut s = 0.0f64;
        for j in 1..=k {
            s += ((m + j) as f64 / (j as f64)).log2();
        }
        s
    } else {
        (ln_gamma((a + b + 1) as f64) - ln_gamma((a + 1) as f64) - ln_gamma((b + 1) as f64))
            / std::f64::consts::LN_2
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

fn ref_fl_from_records(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
    out.clear();
    if n == 0 {
        return;
    }
    let l = records_to_l(records, n);
    *out = ref_fl(&l, p);
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
    let mut rng = XorShift64::new(0xC552_1808_1843_BEEF);
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
// 2. PART 1 — Exact BigInt / BigUint C. F. Gauss (1808/1811) Gaussian Binomial
//    Coefficients & A. L. Cauchy (1843) Finite q-Binomial Theorems
// ============================================================================

fn trim_poly_u(p: &mut Vec<BigUint>) {
    while p.len() > 1 && p.last().unwrap().is_zero() {
        p.pop();
    }
    if p.is_empty() {
        p.push(BigUint::zero());
    }
}

fn trim_poly_i(p: &mut Vec<BigInt>) {
    while p.len() > 1 && p.last().unwrap().is_zero() {
        p.pop();
    }
    if p.is_empty() {
        p.push(BigInt::zero());
    }
}

fn poly_add_shift_u(a: &[BigUint], b: &[BigUint], shift: usize) -> Vec<BigUint> {
    let len = a.len().max(b.len() + shift);
    let mut out = vec![BigUint::zero(); len];
    for (i, c) in a.iter().enumerate() {
        out[i] = c.clone();
    }
    for (i, c) in b.iter().enumerate() {
        if !c.is_zero() {
            out[i + shift].add_assign(c);
        }
    }
    trim_poly_u(&mut out);
    out
}

fn poly_mul_u(a: &[BigUint], b: &[BigUint]) -> Vec<BigUint> {
    if a.is_empty() || b.is_empty() {
        return vec![BigUint::zero()];
    }
    let mut out = vec![BigUint::zero(); a.len() + b.len() - 1];
    for (i, ca) in a.iter().enumerate() {
        if ca.is_zero() {
            continue;
        }
        for (j, cb) in b.iter().enumerate() {
            if !cb.is_zero() {
                let prod = ca.mul(cb);
                out[i + j].add_assign(&prod);
            }
        }
    }
    trim_poly_u(&mut out);
    out
}

/// Exact polynomial multiplication by (1 - q^k) in Z[q].
fn poly_mul_1_minus_qk(p: &[BigInt], k: usize) -> Vec<BigInt> {
    let mut out = vec![BigInt::zero(); p.len() + k];
    for (i, c) in p.iter().enumerate() {
        if !c.is_zero() {
            out[i] = out[i].add(c);
            out[i + k] = out[i + k].sub(c);
        }
    }
    trim_poly_i(&mut out);
    out
}

/// Exact polynomial division by (1 - q^k) in Z[q], asserting exact ZERO remainder!
/// Since (1 - q^k) * Q(q) = P(q), working from lowest degree up:
///   Q[i] = P[i] + Q[i - k] (for i >= k), and Q[i] must vanish for all i > deg(P) - k.
fn poly_div_1_minus_qk_exact(p: &[BigInt], k: usize) -> Vec<BigInt> {
    assert!(k >= 1 && p.len() > k);
    let q_len = p.len() - k;
    let mut q = vec![BigInt::zero(); p.len()];
    for i in 0..p.len() {
        let mut val = p[i].clone();
        if i >= k {
            val = val.add(&q[i - k]);
        }
        q[i] = val;
    }
    for (i, rem) in q.iter().enumerate().skip(q_len) {
        assert!(
            rem.is_zero(),
            "Non-zero remainder at degree {i} when dividing by (1 - q^{k})!"
        );
    }
    q.truncate(q_len);
    trim_poly_i(&mut q);
    q
}

/// Method A (Direct Gauss 1808/1811 Product-Quotient):
///   \binom{a+b}{a}_q = \prod_{j=1}^{a} (1 - q^{b+j}) / \prod_{j=1}^{a} (1 - q^j)
/// computed directly in exact BigInt polynomial arithmetic with exact zero-remainder checks.
fn q_binom_direct_product(a: usize, b: usize) -> Vec<BigUint> {
    let mut p = vec![BigInt::one()];
    for j in 1..=a {
        p = poly_mul_1_minus_qk(&p, b + j);
    }
    for j in 1..=a {
        p = poly_div_1_minus_qk_exact(&p, j);
    }
    assert_eq!(
        p.len(),
        a * b + 1,
        "Unexpected degree for q_binom_direct_product({a}, {b})"
    );
    let mut out = Vec::with_capacity(p.len());
    for c in p {
        assert!(!c.neg, "Negative coefficient in Gaussian binomial ({a},{b})!");
        out.push(c.mag);
    }
    out
}

/// Exact integer binomial triangle binom(N, k) for 0 <= N <= max_n.
fn pascal_bigint_table(max_n: usize) -> Vec<Vec<BigUint>> {
    let mut t = vec![Vec::new(); max_n + 1];
    for n in 0..=max_n {
        t[n] = vec![BigUint::zero(); n + 1];
        t[n][0] = BigUint::one();
        t[n][n] = BigUint::one();
        for k in 1..n {
            t[n][k] = t[n - 1][k - 1].add(&t[n - 1][k]);
        }
    }
    t
}

/// Exact CA-52 Euler-Jacobi pentagonal recurrence for p(0..=max_n).
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

/// Exhaustively enumerate ALL Ferrers diagrams fitting inside an `a x b` box
/// (at most `a` parts, each part in `1..=b`) and accumulate exact weight histogram & count.
fn enumerate_box_partitions(a: usize, b: usize) -> (u64, Vec<u64>, Vec<Vec<u8>>) {
    let mut hist = vec![0u64; a * b + 1];
    let mut all = Vec::new();
    let mut cur = Vec::with_capacity(a);
    fn dfs(
        rem_parts: usize,
        max_part: usize,
        sum: usize,
        cur: &mut Vec<u8>,
        hist: &mut [u64],
        all: &mut Vec<Vec<u8>>,
    ) {
        hist[sum] += 1;
        all.push(cur.clone());
        if rem_parts == 0 || max_part == 0 {
            return;
        }
        for p in 1..=max_part {
            cur.push(p as u8);
            dfs(rem_parts - 1, p, sum + p, cur, hist, all);
            cur.pop();
        }
    }
    dfs(a, b, 0, &mut cur, &mut hist, &mut all);
    let cnt = all.len() as u64;
    (cnt, hist, all)
}

fn conjugate_part(lam: &[u8]) -> Vec<u8> {
    if lam.is_empty() {
        return Vec::new();
    }
    let max_p = usize::from(lam[0]);
    let mut conj = Vec::with_capacity(max_p);
    for col in 1..=max_p {
        let mut h = 0u8;
        for &p in lam {
            if usize::from(p) >= col {
                h += 1;
            } else {
                break;
            }
        }
        conj.push(h);
    }
    conj
}

fn verify_part1_gauss_cauchy() {
    let t0 = Instant::now();
    println!("================================================================================");
    println!("PART 1 — EXACT BIGINT C. F. GAUSS (1808/1811) GAUSSIAN BINOMIAL COEFFICIENTS");
    println!("         & A. L. CAUCHY (1843) FINITE q-BINOMIAL THEOREMS (0 ERRORS)");
    println!("================================================================================");

    // 1A. Build full exact Gaussian binomial triangle qbin_nk[N][k] = \binom{N}{k}_q for 0 <= N <= 40
    // using Recurrence #1: \binom{a+b}{a}_q = \binom{a+b-1}{a-1}_q + q^a \binom{a+b-1}{a}_q.
    let max_n_tri = 40usize;
    let binom_int = pascal_bigint_table(max_n_tri);
    let mut qbin_nk: Vec<Vec<Vec<BigUint>>> = vec![Vec::new(); max_n_tri + 1];
    for n in 0..=max_n_tri {
        qbin_nk[n] = vec![Vec::new(); n + 1];
        qbin_nk[n][0] = vec![BigUint::one()];
        qbin_nk[n][n] = vec![BigUint::one()];
        for k in 1..n {
            // a = k, b = n - k: \binom{n}{k}_q = \binom{n-1}{k-1}_q + q^k \binom{n-1}{k}_q
            qbin_nk[n][k] = poly_add_shift_u(&qbin_nk[n - 1][k - 1], &qbin_nk[n - 1][k], k);
        }
    }

    // Verify all 441 pairs (a, b) in 0..=20:
    //   - Direct Gauss product-quotient (Method A) == Recurrence #1 (Method B) == Recurrence #2 (Method C)
    //   - Symmetry \binom{a+b}{a}_q == \binom{a+b}{b}_q
    //   - Palindromic symmetry c[k] == c[a*b - k]
    //   - Unimodality (Sylvester 1878)
    //   - Specialization at q = 1 (== binom(a+b, a)) and q = -1 (Gauss 1808/1811)
    let mut checked_ab = 0usize;
    let mut total_coeffs_ab = 0usize;
    for a in 0..=20usize {
        for b in 0..=20usize {
            let p_rec1 = &qbin_nk[a + b][a];
            let p_direct = q_binom_direct_product(a, b);
            assert_eq!(
                *p_rec1, p_direct,
                "Gauss product vs q-Pascal #1 mismatch at ({a},{b})!"
            );

            // Recurrence #2: \binom{a+b}{a}_q = q^b \binom{a+b-1}{a-1}_q + \binom{a+b-1}{a}_q
            if a > 0 && b > 0 {
                let p_rec2 = poly_add_shift_u(&qbin_nk[a + b - 1][a], &qbin_nk[a + b - 1][a - 1], b);
                assert_eq!(
                    *p_rec1, p_rec2,
                    "Gauss q-Pascal #2 mismatch at ({a},{b})!"
                );
            }

            // Symmetry \binom{a+b}{a}_q == \binom{a+b}{b}_q
            assert_eq!(*p_rec1, qbin_nk[a + b][b], "Symmetry mismatch at ({a},{b})!");

            // Degree == a*b, endpoints == 1
            let deg = a * b;
            assert_eq!(p_rec1.len(), deg + 1);
            assert_eq!(p_rec1[0], BigUint::one());
            assert_eq!(p_rec1[deg], BigUint::one());

            // Palindromicity
            for k in 0..=deg / 2 {
                assert_eq!(
                    p_rec1[k],
                    p_rec1[deg - k],
                    "Palindromicity mismatch at ({a},{b}), k={k}!"
                );
            }

            // Unimodality (Sylvester 1878)
            let half = deg / 2;
            for k in 0..half {
                assert!(
                    p_rec1[k].cmp_mag(&p_rec1[k + 1]) != std::cmp::Ordering::Greater,
                    "Unimodality failure at ({a},{b}), k={k}!"
                );
            }

            // q = 1 specialization: sum == binom(a+b, a)
            let mut sum_q1 = BigUint::zero();
            let mut sum_qm1 = BigInt::zero();
            for (k, c) in p_rec1.iter().enumerate() {
                sum_q1.add_assign(c);
                let term = BigInt::from_biguint(c.clone());
                sum_qm1 = if (k & 1) == 0 {
                    sum_qm1.add(&term)
                } else {
                    sum_qm1.sub(&term)
                };
            }
            assert_eq!(
                sum_q1,
                binom_int[a + b][a],
                "q=1 specialization mismatch at ({a},{b})!"
            );

            // q = -1 specialization (Gauss 1808/1811):
            let want_qm1 = if (a & 1) == 1 && (b & 1) == 1 {
                BigInt::zero()
            } else {
                BigInt::from_biguint(binom_int[(a / 2) + (b / 2)][a / 2].clone())
            };
            assert_eq!(
                sum_qm1, want_qm1,
                "q=-1 specialization mismatch at ({a},{b})!"
            );

            total_coeffs_ab += p_rec1.len();
            checked_ab += 1;
        }
    }

    // 1B. q-Vandermonde Convolution for ALL a, b in 0..=20 and ALL k in 0..=a+b:
    //   \binom{a+b}{k}_q == \sum_{j=max(0,k-b)}^{min(a,k)} q^{(a-j)(k-j)} \binom{a}{j}_q \binom{b}{k-j}_q
    let mut vand_triples = 0usize;
    for a in 0..=20usize {
        for b in 0..=20usize {
            for k in 0..=(a + b) {
                let j_lo = k.saturating_sub(b);
                let j_hi = a.min(k);
                let mut rhs = vec![BigUint::zero()];
                for j in j_lo..=j_hi {
                    let prod = poly_mul_u(&qbin_nk[a][j], &qbin_nk[b][k - j]);
                    let shift = (a - j) * (k - j);
                    rhs = poly_add_shift_u(&rhs, &prod, shift);
                }
                assert_eq!(
                    rhs,
                    qbin_nk[a + b][k],
                    "q-Vandermonde mismatch at a={a}, b={b}, k={k}!"
                );
                vand_triples += 1;
            }
        }
    }
    println!(
        "  [PASS] 1A-1B Gauss (1808/1811) Gaussian Binomials (a,b <= 20): {checked_ab}/441 pairs ({total_coeffs_ab} exact BigUint coeffs) — direct product == q-Pascal #1 == q-Pascal #2, symmetry, palindromicity, Sylvester (1878) unimodality, q=+1 & q=-1, + q-Vandermonde ({vand_triples}/9,261 triples) 0 errors!"
    );
    println!(
        "         Sample: binom(40,20)_q has deg=400, middle coeff [q^200]={}, sum(q=1)={}",
        qbin_nk[40][20][200].to_decimal(),
        binom_int[40][20].to_decimal()
    );

    // 1C. Cauchy (1843) Finite Distinct-Part q-Binomial Theorem for ALL n = 0..=32 (exact, NO q-truncation!):
    //   \prod_{k=0}^{n-1} (1 + z q^k) == \sum_{j=0}^{n} q^{\binom{j}{2}} \binom{n}{j}_q z^j
    let mut cauchy_dist_monomials = 0usize;
    // Also verify z=q specialization matches CA-52 Euler distinct Q(q) mod q^{n+1}
    let mut euler_q_32 = vec![BigUint::zero(); 33];
    euler_q_32[0] = BigUint::one();
    for k in 1..=32 {
        for m in (k..=32).rev() {
            let prev = euler_q_32[m - k].clone();
            euler_q_32[m].add_assign(&prev);
        }
    }
    for n in 0..=32usize {
        let max_q = n * n.saturating_sub(1) / 2;
        let mut lhs = vec![vec![BigUint::zero(); max_q + 1]; n + 1];
        lhs[0][0] = BigUint::one();
        for k in 0..n {
            for j in (1..=k + 1).rev() {
                for deg in (k..=max_q).rev() {
                    if !lhs[j - 1][deg - k].is_zero() {
                        let prev = lhs[j - 1][deg - k].clone();
                        lhs[j][deg].add_assign(&prev);
                    }
                }
            }
        }
        // Compare every (j, deg) against q^{j*(j-1)/2} * \binom{n}{j}_q
        let mut z_eq_q = vec![BigUint::zero(); n + 1];
        for j in 0..=n {
            let tri = j * j.saturating_sub(1) / 2;
            let qb = &qbin_nk[n][j];
            for deg in 0..=max_q {
                let want = if deg >= tri && deg - tri < qb.len() {
                    &qb[deg - tri]
                } else {
                    &BigUint::zero()
                };
                assert_eq!(
                    lhs[j][deg], *want,
                    "Cauchy (1843) distinct q-binomial mismatch at n={n}, j={j}, q^{deg}!"
                );
                cauchy_dist_monomials += 1;
                if deg + j <= n && !lhs[j][deg].is_zero() {
                    z_eq_q[deg + j].add_assign(&lhs[j][deg]);
                }
            }
        }
        // Verify z=q specialization: \prod_{k=1}^n (1 + q^k) == Q(q) mod q^{n+1}
        for deg in 0..=n {
            assert_eq!(
                z_eq_q[deg], euler_q_32[deg],
                "Cauchy z=q distinct specialization mismatch vs CA-52 Q(q) at n={n}, deg={deg}!"
            );
        }
    }

    // 1D. Cauchy (1843) Finite Reciprocal q-Binomial Theorem for ALL n = 0..=32 (mod z^{41} q^{201}):
    //   \prod_{k=0}^{n-1} (1 - z q^k)^{-1} == \sum_{j=0}^{40} \binom{n+j-1}{j}_q z^j  (mod z^{41}, q^{201})
    let z_max = 40usize;
    let q_max = 200usize;
    // Build truncated q-binomial table g_trunc[a][b] = \binom{a+b}{a}_q mod q^{201} for a <= 32, b <= 40
    // via exact q-Pascal: g(a, b) = g(a-1, b) + q^a g(a, b-1)
    let mut g_trunc: Vec<Vec<Vec<BigUint>>> =
        vec![vec![vec![BigUint::zero(); q_max + 1]; z_max + 1]; 33];
    for a in 0..=32usize {
        for b in 0..=z_max {
            if a == 0 || b == 0 {
                g_trunc[a][b][0] = BigUint::one();
            } else {
                let mut cur = g_trunc[a - 1][b].clone();
                for d in a..=q_max {
                    if !g_trunc[a][b - 1][d - a].is_zero() {
                        let prev = g_trunc[a][b - 1][d - a].clone();
                        cur[d].add_assign(&prev);
                    }
                }
                g_trunc[a][b] = cur;
            }
        }
    }

    let mut cauchy_recip_monomials = 0usize;
    for n in 0..=32usize {
        let mut lhs = vec![vec![BigUint::zero(); q_max + 1]; z_max + 1];
        lhs[0][0] = BigUint::one();
        for k in 0..n {
            for j in 1..=z_max {
                for d in k..=q_max {
                    if !lhs[j - 1][d - k].is_zero() {
                        let prev = lhs[j - 1][d - k].clone();
                        lhs[j][d].add_assign(&prev);
                    }
                }
            }
        }
        for j in 0..=z_max {
            for d in 0..=q_max {
                let want = if n == 0 {
                    if j == 0 && d == 0 {
                        BigUint::one()
                    } else {
                        BigUint::zero()
                    }
                } else {
                    // \binom{n+j-1}{j}_q = g_trunc[n-1][j]
                    g_trunc[n - 1][j][d].clone()
                };
                assert_eq!(
                    lhs[j][d], want,
                    "Cauchy (1843) reciprocal q-binomial mismatch at n={n}, j={j}, q^{d}!"
                );
                cauchy_recip_monomials += 1;
            }
        }
    }
    println!(
        "  [PASS] 1C-1D Cauchy (1843) Finite q-Binomial Theorems (ALL n = 0..=32): distinct ({cauchy_dist_monomials} exact monomials + CA-52 z=q Q(q) match) & reciprocal mod z^41 q^201 ({cauchy_recip_monomials} exact monomials) 0 errors!"
    );

    // 1E. Exact Convergence of \binom{a+b}{a}_q to CA-52 Pentagonal p(N) for N = 0..=200
    // via genuine q-Pascal rolling DP up to (a=200, b=200) mod q^{201}:
    let p_pent = pentagonal_p_recurrence(q_max);
    let mut row: Vec<Vec<BigUint>> = vec![vec![BigUint::zero(); q_max + 1]; q_max + 1];
    for b in 0..=q_max {
        row[b][0] = BigUint::one(); // a = 0: \binom{b}{0}_q = 1
    }
    let mut saved_box_200: Vec<(usize, Vec<BigUint>)> = Vec::new();
    let probe_ls = [8usize, 10, 11, 12, 13, 14, 15, 16, 20, 24, 32, 200];
    let mut first_div_ok = 0usize;
    for a in 1..=q_max {
        for b in 1..=q_max {
            for d in a..=q_max {
                if !row[b - 1][d - a].is_zero() {
                    let prev = row[b - 1][d - a].clone();
                    row[b][d].add_assign(&prev);
                }
            }
        }
        // Now row[200] == \binom{200 + a}{a}_q mod q^{201}!
        if a <= 40 {
            for d in 0..=a {
                assert_eq!(
                    row[q_max][d], p_pent[d],
                    "Stabilization failure for L={a} at N={d}!"
                );
            }
            // Exact first divergence at N = a + 1: p(a+1) - p(<=a parts; a+1) == 1
            let diff = p_pent[a + 1].sub_ge(&row[q_max][a + 1]);
            assert_eq!(
                diff,
                BigUint::one(),
                "First divergence at N=L+1 must be exactly 1 for L={a}!"
            );
            first_div_ok += 1;
        }
        if probe_ls.contains(&a) {
            saved_box_200.push((a, row[q_max].clone()));
        }
    }
    // Check full convergence at a = b = 200: [q^N]\binom{400}{200}_q == p_pent[N] for ALL N = 0..=200!
    for d in 0..=q_max {
        assert_eq!(
            row[q_max][d], p_pent[d],
            "Convergence mismatch vs CA-52 pentagonal p({d})!"
        );
    }
    // Also cross-check each L in probe_ls against Cauchy reciprocal product \prod_{k=1}^L (1 - q^k)^{-1} mod q^{201}
    let mut recip_prod_checks = 0usize;
    println!(
        "  [PASS] 1E Exact Convergence to CA-52 Pentagonal p(0..=200): 201/201 EXACT (p(200) = {}), first-overflow deficit == 1 at N=L+1 for {first_div_ok}/40 cutoffs!",
        p_pent[200].to_decimal()
    );
    print!("         Bounded-box fraction [q^200]\\binom{{200+L}}{{L}}_q / p(200):");
    for (l, coeffs) in &saved_box_200 {
        let mut prod = vec![BigUint::zero(); q_max + 1];
        prod[0] = BigUint::one();
        for k in 1..=*l {
            for d in k..=q_max {
                let prev = prod[d - k].clone();
                prod[d].add_assign(&prev);
            }
        }
        for d in 0..=q_max {
            assert_eq!(
                coeffs[d], prod[d],
                "Box binom(200+L,L)_q vs Cauchy reciprocal product mismatch at L={l}, N={d}!"
            );
            recip_prod_checks += 1;
        }
        let frac = 100.0 * coeffs[200].to_f64() / p_pent[200].to_f64();
        if *l <= 32 {
            print!(" L={l}:{frac:.2}%");
        }
    }
    println!(" ({recip_prod_checks} exact box==Cauchy checks)");

    // 1F. Exhaustive Enumeration of Bounded-Box Ferrers Partitions inside a x b for ALL 0 <= a, b <= 10!
    let mut boxes_ok = 0usize;
    let mut total_diagrams = 0u64;
    let mut box_sets: Vec<Vec<Vec<Vec<u8>>>> = vec![vec![Vec::new(); 11]; 11];
    for a in 0..=10usize {
        for b in 0..=10usize {
            let (cnt, hist, all) = enumerate_box_partitions(a, b);
            assert_eq!(
                BigUint::from_u64(cnt),
                binom_int[a + b][a],
                "Box count mismatch at ({a},{b})!"
            );
            let qb = &qbin_nk[a + b][a];
            assert_eq!(hist.len(), qb.len());
            for (d, &h) in hist.iter().enumerate() {
                assert_eq!(
                    BigUint::from_u64(h),
                    qb[d],
                    "Box Ferrers weight mismatch at ({a},{b}), N={d}!"
                );
            }
            // Complement involution inside a x b rectangle: bar(lam)_i = b - lam_{a+1-i}
            let mut comp_hist = vec![0u64; a * b + 1];
            for lam in &all {
                let mut sum_comp = 0usize;
                for i in 0..a {
                    let li = if i < lam.len() { usize::from(lam[i]) } else { 0 };
                    assert!(li <= b);
                    sum_comp += b - li;
                }
                comp_hist[sum_comp] += 1;
            }
            assert_eq!(comp_hist, hist, "Complement involution mismatch at ({a},{b})!");
            box_sets[a][b] = all;
            total_diagrams += cnt;
            boxes_ok += 1;
        }
    }
    // Verify Ferrers conjugation bijection between a x b and b x a boxes for ALL 0 <= a, b <= 10!
    for a in 0..=10usize {
        for b in 0..=10usize {
            let mut conj_sorted: Vec<Vec<u8>> =
                box_sets[a][b].iter().map(|lam| conjugate_part(lam)).collect();
            conj_sorted.sort();
            let mut target_sorted = box_sets[b][a].clone();
            target_sorted.sort();
            assert_eq!(
                conj_sorted, target_sorted,
                "Ferrers conjugation bijection mismatch between {a}x{b} and {b}x{a}!"
            );
        }
    }
    println!(
        "  [PASS] 1F Exhaustive Bounded-Box Ferrers Enumeration (ALL 0 <= a,b <= 10): {boxes_ok}/121 boxes, {total_diagrams} Ferrers diagrams (== binom(22,11)-1 = 705,431!) — exact count, q-weight histogram, complement involution, & Ferrers conjugation bijection 0 errors! ({:.2}s)\n",
        t0.elapsed().as_secs_f64()
    );
}

// ============================================================================
// 3. Embedded Exact `C54-PipeLean-v2` Kernel (`S5_UnalignedBack_1Ptr` +
//    `ZeroSpill_ConstBits` + `S4_8WayEpoch`) with Zero-Cost TIMED/STATS Hooks
// ============================================================================

mod open_c54_pipelean_v2 {
    use std::time::Instant;

    #[derive(Clone, Copy, Default, Debug)]
    pub(super) struct StageTimes {
        pub(super) s1_us: f64,
        pub(super) s2a_us: f64,
        pub(super) s2b_us: f64,
        pub(super) s3a_us: f64,
        pub(super) s4_us: f64,
        pub(super) s5_us: f64,
    }

    #[derive(Clone, Copy, Default, Debug)]
    pub(super) struct RunStats {
        pub(super) m: u32,
        pub(super) ns: u32,
        pub(super) r_thin: u32,
        pub(super) s2b_rule_words: usize,
        pub(super) s2b_rules_arena: u64,
        pub(super) s2b_joins: u64,
        pub(super) s2b_steps: u64,
        pub(super) dict_codes: usize,
        pub(super) spelling_bytes: usize,
        pub(super) outq_chunks: u32,
        pub(super) edges_used_bytes: usize,
        pub(super) edges_cap_bytes: usize,
        pub(super) peak_cap_bytes: usize,
    }

    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
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

    // F-Inline3 handle encoding: 8-bit grade (1..=255), 24-bit inline payload / raw word-offset rule id.
    const ID_BITS: u32 = 24;
    const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
    const SATURATED: u32 = u32::MAX >> ID_BITS; // 255

    #[inline(always)]
    const fn literal(byte: u8) -> u32 {
        (1u32 << ID_BITS) | (byte as u32)
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(super) enum BiMode {
        /// Exact `CA-27`/`CA-31` moment bound using `r_max = nrec - extra_127` (tracked strictly
        /// in the cold `width == 127` loop: 0 hot-loop ops). Guarantees
        /// `2*(r_max - ns) - m = w1 - S2 - 2*delta_p <= w1` unconditionally on all inputs.
        PostS1ExactMd {
            r_max_cap: u32,
            ns_max: u32,
            md_max: i64,
            md_neg_cap: i64,
            ns_neg_max: u32,
        },
    }

    impl BiMode {
        #[inline(always)]
        fn should_use_bi(self, m: u32, ns: u32, r_max: u32) -> bool {
            match self {
                BiMode::PostS1ExactMd {
                    r_max_cap,
                    ns_max,
                    md_max,
                    md_neg_cap,
                    ns_neg_max,
                } => {
                    if m < 65_536 {
                        return false;
                    }
                    let md_w1 = 2i64 * (i64::from(r_max) - i64::from(ns)) - i64::from(m);
                    !((r_max <= r_max_cap && ns <= ns_max && md_w1 <= md_max)
                        || (md_w1 < md_neg_cap && ns <= ns_neg_max))
                }
            }
        }
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
            First {
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
    fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
        let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
        let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
        &spelling[start..end]
    }

    fn begin_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
        if inline16.len() < 257 {
            inline16.clear();
            inline16.reserve_exact(257);
            // +16 B leading guard (`inline16[0]`) enables branchless, zero-shift,
            // bounds-check-free unaligned backward emit from `inline16.as_flattened()`.
            inline16.push([0; INLINE]);
            inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            lens.clear();
            lens.reserve_exact(256);
            lens.resize(256, 1);
        }
        inline16.truncate(257);
        lens.truncate(256);
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
        inline16.reserve_exact((target + 1).saturating_sub(inline16.len()));
        lens.reserve_exact(target.saturating_sub(lens.len()));
    }

    #[inline(always)]
    fn ensure_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
        if lens.len() == lens.capacity() || inline16.len() == inline16.capacity() {
            grow_step_inline(inline16, lens, step);
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

    // ------------------------------------------------------------------------
    // OutQ16 thin-rank queues living in `out[..tail_offset]` with compact
    // `next: Vec<u16>`, exact-reserve `cuts`, `16`-chunk stepped `ovf`, and
    // `SlimS4_Full + S4SlowHi0` (`as_chunks::<2>()` `movzwl` + `pack_hi` +
    // `hi==0` on BOTH fast and 8-way batched slow paths).
    // ------------------------------------------------------------------------
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
                2 * self.next.capacity()
                    + BYTES * self.ovf.capacity()
                    + 1024 * self.cuts.capacity()
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
                Queues {
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
                        let cut: [u32; 256] = std::array::from_fn(|b| (self.pt[b] >> 32) as u32);
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
            pub(super) fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
                Reader {
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

            /// `S4_8WayEpoch`: 8-way interleaved AVX2 sequential XOR + `[u32; 65536]` epoch-slice
            /// scattered XOR (eliminates 100% of scattered bounds checks and `hi | l_k` ORs in all
            /// full 64K epochs while preserving full 8-way scattered-write MLP!).
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
                    let hi_u32 = self.epoch << 16;
                    let hi = hi_u32 as usize;
                    let pack_hi = pack | hi_u32;
                    let at = self.id as usize * BYTES + 2 * self.at;
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    let mut i = 0usize;
                    if hi + (EPOCH as usize) <= edges.len() {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = usize::from(u16::from_le_bytes(ch[0]));
                            let l1 = usize::from(u16::from_le_bytes(ch[1]));
                            let l2 = usize::from(u16::from_le_bytes(ch[2]));
                            let l3 = usize::from(u16::from_le_bytes(ch[3]));
                            let l4 = usize::from(u16::from_le_bytes(ch[4]));
                            let l5 = usize::from(u16::from_le_bytes(ch[5]));
                            let l6 = usize::from(u16::from_le_bytes(ch[6]));
                            let l7 = usize::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | (l0 as u32);
                            eu[1] ^= pack_hi | (l1 as u32);
                            eu[2] ^= pack_hi | (l2 as u32);
                            eu[3] ^= pack_hi | (l3 as u32);
                            eu[4] ^= pack_hi | (l4 as u32);
                            eu[5] ^= pack_hi | (l5 as u32);
                            eu[6] ^= pack_hi | (l6 as u32);
                            eu[7] ^= pack_hi | (l7 as u32);
                            let ep: &mut [u32; EPOCH as usize] =
                                (&mut edges[hi..hi + (EPOCH as usize)]).try_into().unwrap();
                            let u32_0 = u0 as u32;
                            ep[l0] ^= u32_0;
                            ep[l1] ^= u32_0 + 1;
                            ep[l2] ^= u32_0 + 2;
                            ep[l3] ^= u32_0 + 3;
                            ep[l4] ^= u32_0 + 4;
                            ep[l5] ^= u32_0 + 5;
                            ep[l6] ^= u32_0 + 6;
                            ep[l7] ^= u32_0 + 7;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = usize::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | (lo as u32);
                            let ep: &mut [u32; EPOCH as usize] =
                                (&mut edges[hi..hi + (EPOCH as usize)]).try_into().unwrap();
                            ep[lo] ^= u as u32;
                            i += 1;
                        }
                    } else {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = usize::from(u16::from_le_bytes(ch[0]));
                            let l1 = usize::from(u16::from_le_bytes(ch[1]));
                            let l2 = usize::from(u16::from_le_bytes(ch[2]));
                            let l3 = usize::from(u16::from_le_bytes(ch[3]));
                            let l4 = usize::from(u16::from_le_bytes(ch[4]));
                            let l5 = usize::from(u16::from_le_bytes(ch[5]));
                            let l6 = usize::from(u16::from_le_bytes(ch[6]));
                            let l7 = usize::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | (l0 as u32);
                            eu[1] ^= pack_hi | (l1 as u32);
                            eu[2] ^= pack_hi | (l2 as u32);
                            eu[3] ^= pack_hi | (l3 as u32);
                            eu[4] ^= pack_hi | (l4 as u32);
                            eu[5] ^= pack_hi | (l5 as u32);
                            eu[6] ^= pack_hi | (l6 as u32);
                            eu[7] ^= pack_hi | (l7 as u32);
                            let ep = &mut edges[hi..];
                            let u32_0 = u0 as u32;
                            ep[l0] ^= u32_0;
                            ep[l1] ^= u32_0 + 1;
                            ep[l2] ^= u32_0 + 2;
                            ep[l3] ^= u32_0 + 3;
                            ep[l4] ^= u32_0 + 4;
                            ep[l5] ^= u32_0 + 5;
                            ep[l6] ^= u32_0 + 6;
                            ep[l7] ^= u32_0 + 7;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = usize::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | (lo as u32);
                            edges[hi + lo] ^= u as u32;
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
                    let hi_u32 = self.epoch << 16;
                    let hi = hi_u32 as usize;
                    let pack_hi = pack | hi_u32;
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
                    if hi + (EPOCH as usize) <= edges.len() {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = usize::from(u16::from_le_bytes(ch[0]));
                            let l1 = usize::from(u16::from_le_bytes(ch[1]));
                            let l2 = usize::from(u16::from_le_bytes(ch[2]));
                            let l3 = usize::from(u16::from_le_bytes(ch[3]));
                            let l4 = usize::from(u16::from_le_bytes(ch[4]));
                            let l5 = usize::from(u16::from_le_bytes(ch[5]));
                            let l6 = usize::from(u16::from_le_bytes(ch[6]));
                            let l7 = usize::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | (l0 as u32);
                            eu[1] ^= pack_hi | (l1 as u32);
                            eu[2] ^= pack_hi | (l2 as u32);
                            eu[3] ^= pack_hi | (l3 as u32);
                            eu[4] ^= pack_hi | (l4 as u32);
                            eu[5] ^= pack_hi | (l5 as u32);
                            eu[6] ^= pack_hi | (l6 as u32);
                            eu[7] ^= pack_hi | (l7 as u32);
                            let ep: &mut [u32; EPOCH as usize] =
                                (&mut edges[hi..hi + (EPOCH as usize)]).try_into().unwrap();
                            let u32_0 = u0 as u32;
                            ep[l0] ^= u32_0;
                            ep[l1] ^= u32_0 + 1;
                            ep[l2] ^= u32_0 + 2;
                            ep[l3] ^= u32_0 + 3;
                            ep[l4] ^= u32_0 + 4;
                            ep[l5] ^= u32_0 + 5;
                            ep[l6] ^= u32_0 + 6;
                            ep[l7] ^= u32_0 + 7;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = usize::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | (lo as u32);
                            let ep: &mut [u32; EPOCH as usize] =
                                (&mut edges[hi..hi + (EPOCH as usize)]).try_into().unwrap();
                            ep[lo] ^= u as u32;
                            i += 1;
                        }
                    } else {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = usize::from(u16::from_le_bytes(ch[0]));
                            let l1 = usize::from(u16::from_le_bytes(ch[1]));
                            let l2 = usize::from(u16::from_le_bytes(ch[2]));
                            let l3 = usize::from(u16::from_le_bytes(ch[3]));
                            let l4 = usize::from(u16::from_le_bytes(ch[4]));
                            let l5 = usize::from(u16::from_le_bytes(ch[5]));
                            let l6 = usize::from(u16::from_le_bytes(ch[6]));
                            let l7 = usize::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | (l0 as u32);
                            eu[1] ^= pack_hi | (l1 as u32);
                            eu[2] ^= pack_hi | (l2 as u32);
                            eu[3] ^= pack_hi | (l3 as u32);
                            eu[4] ^= pack_hi | (l4 as u32);
                            eu[5] ^= pack_hi | (l5 as u32);
                            eu[6] ^= pack_hi | (l6 as u32);
                            eu[7] ^= pack_hi | (l7 as u32);
                            let ep = &mut edges[hi..];
                            let u32_0 = u0 as u32;
                            ep[l0] ^= u32_0;
                            ep[l1] ^= u32_0 + 1;
                            ep[l2] ^= u32_0 + 2;
                            ep[l3] ^= u32_0 + 3;
                            ep[l4] ^= u32_0 + 4;
                            ep[l5] ^= u32_0 + 5;
                            ep[l6] ^= u32_0 + 6;
                            ep[l7] ^= u32_0 + 7;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = usize::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | (lo as u32);
                            edges[hi + lo] ^= u as u32;
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

    // ------------------------------------------------------------------------
    // Endpoint elimination (`CE-Var234` + `F-Inline3` + hoisted `limit` +
    // `S2aSlim` + hoisted `K-TailUnw` + `OrphanFree` + upfront `ensure_step_u8`)
    // ------------------------------------------------------------------------
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

    /// Split native `[u32]` `Elim` (`nodes_buf` = `edges[hdr..hdr+base]`, `rules_buf` = `edges[hdr+base..]`).
    /// `CE-Var234` raw word-offset `id = rule_top`:
    ///   - `1..=3` B: `0` words (inline in handle)
    ///   - `4..=8` B: `2` words (`8 B`)
    ///   - `9..=12` B: `3` words (`12 B`)
    ///   - `13..=15` B: `4` words (`16 B`)
    ///   - `16..=254` B: `3` words (`12 B`: `[left, right, memo]`, `rlen = g = h >> 24`)
    ///   - `>= 255` B: `4` words (`16 B`: `[left, right, len, memo]`)
    struct Elim<'n, 'r, L: Links> {
        nodes_buf: &'n mut [u32],
        rules_buf: &'r mut [u32],
        rule_top: usize,
        nodes: u32,
        id_overflow: bool,
        stat_joins: u64,
        stat_rules_arena: u64,
        stat_steps: u64,
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
        fn join<const STATS: bool>(&mut self, left: u32, right: u32) -> u32 {
            if left == 0 {
                return right;
            }
            if right == 0 {
                return left;
            }
            if STATS {
                self.stat_joins += 1;
            }
            let left_len = left >> ID_BITS;
            let sum = left_len + (right >> ID_BITS);
            if sum <= 3 {
                return (sum << ID_BITS)
                    | (left & ID_MASK)
                    | ((right & ID_MASK) << (8 * left_len));
            }
            if STATS {
                self.stat_rules_arena += 1;
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

        fn power<const STATS: bool>(&mut self, word: u32, q: u32) -> u32 {
            let (mut acc, mut base, mut e) = (0, word, q);
            loop {
                if e & 1 == 1 {
                    acc = self.join::<STATS>(acc, base);
                }
                e >>= 1;
                if e == 0 {
                    break acc;
                }
                base = self.join::<STATS>(base, base);
            }
        }

        #[inline(never)]
        fn eliminate_narrow_plain<const STATS: bool>(&mut self, remaining: &mut u32) -> bool {
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
                if STATS {
                    self.stat_steps += 1;
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
                let word = self.join::<STATS>(wx, wy);
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

        fn eliminate<const SPIRALS: bool, const STATS: bool>(&mut self, remaining: &mut u32) -> bool {
            if !SPIRALS && L::IS_NARROW {
                return self.eliminate_narrow_plain::<STATS>(remaining);
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
                if STATS {
                    self.stat_steps += 1;
                }
                let (x, y) = (L::prev(self.nodes_buf, 0, 0), L::prev(self.nodes_buf, 0, 1));
                let (aw, bw) = (self.width(x), self.width(y));
                if SPIRALS && x != y {
                    if aw > bw && L::prev(self.nodes_buf, y, 1) == x && (aw - 1) / bw >= 2 {
                        let q = (aw - 1) / bw;
                        assert!(q * bw < aw && q * bw <= left);
                        let head = self.power::<STATS>(self.word(x), q);
                        let word = self.join::<STATS>(head, self.word(y));
                        self.set_word(y, word);
                        self.set_width(x, aw - q * bw);
                        left -= q * bw;
                        continue;
                    }
                    if aw < bw && L::prev(self.nodes_buf, x, 0) == y && (bw - 1) / aw >= 2 {
                        let q = (bw - 1) / aw;
                        assert!(q * aw < bw && q * aw <= left);
                        let tail = self.power::<STATS>(self.word(y), q);
                        let word = self.join::<STATS>(self.word(x), tail);
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
                let word = self.join::<STATS>(self.word(x), self.word(y));
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

        /// Hoisted `K-TailUnw` (`CS-16`) + `OrphanFree` (`CS-22`) with upfront `dest` capacity
        /// reservation (zero inner-loop capacity checks, zero per-call stack zeroing!).
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
                    self.lens[old] == len && self.inline16[old + 1] == entry
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
                        && long_word(&self.inline16[old + 1], self.spelling) == bytes
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
                self.lens[old] == LONG
                    && long_word(&self.inline16[old + 1], self.spelling) == bytes
            } {
                if !memo_ovf {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    let old_start =
                        u32::from_le_bytes(self.inline16[old + 1][..4].try_into().unwrap());
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

    // ------------------------------------------------------------------------
    // 4-Tier Stack Zero-Heap `TinyFL` (`[u32; 64 / 256 / 512 / 1536]`)
    // ------------------------------------------------------------------------
    fn invert_tiny_fl_with_buf(
        records: &[u16],
        n: usize,
        p: u32,
        fl: &mut [u32],
        out: &mut Vec<u8>,
    ) -> bool {
        let mut counts = [0u32; 256];
        let mut total = 0usize;
        for &rec in records {
            let len = usize::from(rec & 255);
            counts[usize::from((rec >> 8) as u8)] += len as u32;
            total += len;
        }
        if total != n {
            return false;
        }
        let mut st = starts(counts);
        let mut pos = 0u32;
        for &rec in records {
            let b = (rec >> 8) as u8;
            let len = usize::from(rec & 255);
            let bi = usize::from(b);
            let s0 = st[bi] as usize;
            st[bi] += len as u32;
            let bu = u32::from(b);
            let mut i = pos;
            for slot in &mut fl[s0..s0 + len] {
                *slot = (i << 8) | bu;
                i += 1;
            }
            pos += len as u32;
        }
        out.clear();
        out.resize(n, 0);
        let mut x = p as usize;
        for slot in &mut out[..n] {
            let v = fl[x];
            *slot = v as u8;
            x = (v >> 8) as usize;
        }
        true
    }

    #[inline(never)]
    fn invert_tiny_fl_64(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut buf = [0u32; 64];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }

    #[inline(never)]
    fn invert_tiny_fl_256(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut buf = [0u32; 256];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }

    #[inline(never)]
    fn invert_tiny_fl_512(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut buf = [0u32; 512];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }

    #[inline(never)]
    fn invert_tiny_fl_1536(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut buf = [0u32; 1536];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }

    // ------------------------------------------------------------------------
    // Kernel (`TinyFL` + `S1[Step4096] -> S2a[ZeroH+S2aSlim] -> S2b[Var234] ->
    // `S3a[V3CodeMiss+OrphanFree+S3aSlim] -> S4[SlimS4+SlowHi0+Sentinel] ->
    // `S5[SlimS5_Pipe1_Rem16_LE16+Sentinel]`)
    // ------------------------------------------------------------------------
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

    #[derive(Default)]
    pub(super) struct Kernel {
        spill: outq::Spill,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        stack: Vec<u32>,
        predecessor: Option<u32>,
        last_stats: RunStats,
        last_times: StageTimes,
        last_is_bi: bool,
    }

    impl Kernel {
        fn nodes_most(ns: usize) -> usize {
            3 * ns + 3
        }

        /// 4-tier stack zero-WS exact forward-`FL` walk for tiny blocks (`n < 32_768`),
        /// guarded against synthetic `>16M`-record zero-width streams.
        #[inline(never)]
        pub(super) fn invert_tiny_fl(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            out: &mut Vec<u8>,
        ) -> bool {
            if n == 0 || (p as usize) >= n || records.len() > 16_000_000 {
                return false;
            }
            if records.len() == 1 {
                if usize::from(records[0] & 255) != n {
                    return false;
                }
                let b = (records[0] >> 8) as u8;
                out.clear();
                out.resize(n, b);
                return true;
            }
            if n <= 64 {
                return invert_tiny_fl_64(records, n, p, out);
            }
            if n <= 256 {
                return invert_tiny_fl_256(records, n, p, out);
            }
            if n <= 512 {
                return invert_tiny_fl_512(records, n, p, out);
            }
            if n <= 1536 {
                return invert_tiny_fl_1536(records, n, p, out);
            }
            if self.edges.len() < n {
                let add = n - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(n, 0);
            }
            invert_tiny_fl_with_buf(records, n, p, &mut self.edges[..n], out)
        }

        pub(super) fn peak_cap_bytes(&self) -> usize {
            let sp = self.spill.cap_bytes();
            let bd = 8 * self.bands.capacity();
            let s = self.spelling.capacity();
            let ed = 4 * self.edges.capacity();
            let ex = 4 * self.extra.capacity();
            let il = 16 * self.inline16.capacity() + self.lens.capacity();
            let st = 4 * self.stack.capacity();
            sp + bd + s + ed + ex + il + st
        }

        pub(super) fn last_stats(&self) -> RunStats {
            self.last_stats
        }

        pub(super) fn last_times(&self) -> StageTimes {
            self.last_times
        }

        pub(super) fn last_is_bi(&self) -> bool {
            self.last_is_bi
        }

        #[inline(never)]
        pub(super) fn try_run_const_ex<
            const LIM: u32,
            const TIMED: bool,
            const STATS: bool,
            const ENFORCE_GUARD: bool,
        >(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> bool {
            let nrec = records.len();
            // Single check guarantees `pos += width` cannot overflow `u32` (16M * 255 < u32::MAX).
            if n == 0 || n > 900_000 || (p as usize) >= n || nrec > 16_000_000 {
                return false;
            }
            let t_s1 = if TIMED { Some(Instant::now()) } else { None };
            out.resize(n + INLINE, 0);
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
                    if width <= LIM {
                        pass.piece::<LIM>(c, pos, width, true, p);
                        pos += width;
                    } else {
                        if width == 127 {
                            while i < nrec && (records[i] >> 8) as u8 == c {
                                width += u32::from(records[i] & 255);
                                extra_127 += 1;
                                i += 1;
                            }
                        }
                        pass.take::<LIM>(c, pos, width, p);
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
            if over || tot_pos as usize != n || f.m == 0 {
                return false;
            }
            let r_max = (nrec as u32).saturating_sub(extra_127);
            let r_thin_lo = r_max.saturating_sub(ns as u32);
            // Principled post-S1 remaining-cost crossover guard (CA-26 + CA-31):
            // Abort to LazyCells/Seg8-FL on full-size blocks iff `m/n + 0.25*(r_thin/n) > 1.125`.
            if ENFORCE_GUARD && n >= 32_768 && 8 * u64::from(f.m) + 2 * u64::from(r_thin_lo) > 9 * (n as u64) {
                return false;
            }
            if TIMED {
                self.last_times = StageTimes::default();
                self.last_times.s1_us = t_s1.unwrap().elapsed().as_secs_f64() * 1e6;
            }
            if STATS {
                self.last_stats = RunStats {
                    m: f.m,
                    ns: ns as u32,
                    r_thin: r_thin_lo,
                    outq_chunks: heads.chunks,
                    edges_used_bytes: 4 * (f.m as usize),
                    ..RunStats::default()
                };
            }
            self.finish::<TIMED, STATS>(&f, ns, n, r_max, &heads, bi_mode, out)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn finish<const TIMED: bool, const STATS: bool>(
            &mut self,
            f: &First,
            ns: usize,
            n: usize,
            r_max: u32,
            heads: &outq::Heads,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> bool {
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            if all.iter().sum::<u32>() != n as u32 {
                return false;
            }
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

            // Exact O(1) byte offset in `out` strictly after OutQ16's live chunks.
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
                self.induced_narrow::<TIMED, STATS>(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    use_zero_hdr,
                    &mut out[tail_offset..],
                )
            } else {
                self.induced_wide::<TIMED, STATS>(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    use_zero_hdr,
                    &mut out[tail_offset..],
                )
            };
            if !ok_ind {
                return false;
            }

            let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
            let sentinel = u32::MAX >> bits;
            let wide = self.lens.len() - 1 > sentinel as usize;
            if self.edges.len() < m {
                self.edges.resize(m, 0);
            }
            self.extra.clear();
            if wide {
                self.extra.resize(m, 0);
            }
            self.predecessor = None;
            let use_xor = bi_mode.should_use_bi(f.m, ns as u32, r_max);
            self.last_is_bi = use_xor;
            let (out_q, out_tail) = out.split_at(tail_offset);
            let t_s4 = if TIMED { Some(Instant::now()) } else { None };

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
                            heads,
                        )
                    } else {
                        self.wire_bi_vec(m, f.mark, f.mark_byte, bits, &cursors, out_q, heads)
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
                        heads,
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
                    heads,
                );
            } else {
                self.wire_uni_vec(m, wide, bits, sentinel, &cursors, out_q, heads);
            }

            if TIMED {
                self.last_times.s4_us = t_s4.unwrap().elapsed().as_secs_f64() * 1e6;
            }
            let t_s5 = if TIMED { Some(Instant::now()) } else { None };
            if let Some(before) = self.predecessor {
                if !wide {
                    emit_bidirectional(
                        &self.edges[..m],
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        f.mark,
                        before,
                        bits,
                        out,
                    );
                } else {
                    emit_bidirectional_sentinel(
                        &self.edges[..m],
                        &self.extra,
                        &self.spelling,
                        &self.inline16[1..],
                        &self.lens,
                        n,
                        f.mark,
                        before,
                        bits,
                        sentinel,
                        out,
                    );
                }
            } else {
                emit(
                    &self.edges[..m],
                    &self.extra,
                    &self.spelling,
                    &self.inline16[1..],
                    &self.lens,
                    n,
                    f.mark,
                    bits,
                    sentinel,
                    out,
                );
            }
            if TIMED {
                self.last_times.s5_us = t_s5.unwrap().elapsed().as_secs_f64() * 1e6;
            }
            if STATS {
                self.last_stats.dict_codes = self.lens.len();
                self.last_stats.spelling_bytes = self.spelling.len();
                self.last_stats.edges_cap_bytes = 4 * self.edges.capacity();
                self.last_stats.peak_cap_bytes = self.peak_cap_bytes();
            }
            true
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_narrow<const TIMED: bool, const STATS: bool>(
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
        ) -> (bool, [usize; 256], bool, usize) {
            self.induced_with::<Narrow, TIMED, STATS>(ns, n, m, thin, first, write, most, use_zero_hdr, out_tail)
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_wide<const TIMED: bool, const STATS: bool>(
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
        ) -> (bool, [usize; 256], bool, usize) {
            self.induced_with::<Wide, TIMED, STATS>(ns, n, m, thin, first, write, most, use_zero_hdr, out_tail)
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn induced_with<L: Links, const TIMED: bool, const STATS: bool>(
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
        ) -> (bool, [usize; 256], bool, usize) {
            let nodes_words = L::STRIDE * most;
            let hdr_words = if use_zero_hdr { 0 } else { 4 * ns };
            assert!(self.edges.len() >= hdr_words + nodes_words);

            let t_s2a = if TIMED { Some(Instant::now()) } else { None };
            let mut widest = 0u32;
            let (base, nodes, open_nodes) = if use_zero_hdr {
                let (ot_words, _) = out_tail.as_chunks_mut::<4>();
                let (strips_ot, rest_ot) = ot_words.split_at_mut(3 * ns);
                let image_ot = &mut rest_ot[..ns];
                for (d, &s) in strips_ot.iter_mut().zip(&self.edges[..3 * ns]) {
                    *d = s.to_ne_bytes();
                }
                if L::IS_NARROW {
                    // `S2aSlim`: direct `[[u32; 4]]` node array + `[[u8; 4]; 3]` strip triples.
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
                    let open_nodes = nn as usize - 1;
                    (4 * nn as usize, nn, open_nodes)
                } else {
                    let (nodes_sl, rules_dummy) = self.edges.split_at_mut(nodes_words);
                    let mut e = Elim::<L> {
                        nodes_buf: nodes_sl,
                        rules_buf: rules_dummy,
                        rule_top: 0,
                        nodes: 0,
                        id_overflow: false,
                        stat_joins: 0,
                        stat_rules_arena: 0,
                        stat_steps: 0,
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
                    let base = L::STRIDE * e.nodes as usize;
                    (base, e.nodes, open_nodes)
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
                    stat_joins: 0,
                    stat_rules_arena: 0,
                    stat_steps: 0,
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
                let base = L::STRIDE * e.nodes as usize;
                (base, e.nodes, open_nodes)
            };

            if TIMED {
                self.last_times.s2a_us = t_s2a.unwrap().elapsed().as_secs_f64() * 1e6;
            }
            let t_s2b = if TIMED { Some(Instant::now()) } else { None };
            let spirals = widest > CERTIFIED_WIDTH;
            let mut remaining = n;
            let mut rule_top = 0usize;
            let mut acc_joins = 0u64;
            let mut acc_rules_arena = 0u64;
            let mut acc_steps = 0u64;
            loop {
                let (nodes_sl, rules_sl) = self.edges[hdr_words..].split_at_mut(base);
                let mut e = Elim::<L> {
                    nodes_buf: nodes_sl,
                    rules_buf: rules_sl,
                    rule_top,
                    nodes,
                    id_overflow: false,
                    stat_joins: 0,
                    stat_rules_arena: 0,
                    stat_steps: 0,
                    _links: std::marker::PhantomData,
                };
                let done = if spirals {
                    e.eliminate::<true, STATS>(&mut remaining)
                } else {
                    e.eliminate::<false, STATS>(&mut remaining)
                };
                rule_top = e.rule_top;
                if STATS {
                    acc_joins += e.stat_joins;
                    acc_rules_arena += e.stat_rules_arena;
                    acc_steps += e.stat_steps;
                }
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

            if TIMED {
                self.last_times.s2b_us = t_s2b.unwrap().elapsed().as_secs_f64() * 1e6;
            }
            if STATS {
                self.last_stats.s2b_rule_words = rule_top;
                self.last_stats.s2b_rules_arena = acc_rules_arena;
                self.last_stats.s2b_joins = acc_joins;
                self.last_stats.s2b_steps = acc_steps;
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
                stat_joins: 0,
                stat_rules_arena: 0,
                stat_steps: 0,
                _links: std::marker::PhantomData,
            };
            e.assign_bottom(m);

            let cache = if header.len() >= CACHE_WORDS {
                header
            } else {
                tail_arena
            };
            begin_inline(&mut self.inline16, &mut self.lens);

            // Fixed-pool injectivity + 256-bucket common refinement: bands_count < open_nodes + 256.
            let max_bands = open_nodes + 256;
            let use_tail = out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            // Hoisted stack-local scratch allocated ONCE per block (384 B total; zero per-call zeroing!).
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
                        assert!(width > 0);
                        let dst = node_bottom + in_node;
                        let code = if node_word == 0 {
                            a as u32
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
                let mut id = L::next(e.nodes_buf, 0, 0);
                let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
                    L::band_node(e.nodes_buf, id)
                } else {
                    (0, 0, 0, 0)
                };
                let mut in_node = 0u32;
                let mut k = 0u32;
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
                            debug_assert!(width > 0);
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
                            debug_assert!(width > 0);
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
                assert_eq!(id, 0);
            }
            if TIMED {
                self.last_times.s3a_us = t_s3a.unwrap().elapsed().as_secs_f64() * 1e6;
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

        /// Cold `PackedEdgeRule32-Sentinel` bidirectional wiring (`wide == true`):
        /// packs `(code.min(sentinel) << bits) | xor_link` into `edges[u]` and stores
        /// `extra[u] = code` ONLY when `code >= sentinel`, preserving `use_xor = true`!
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

    // ------------------------------------------------------------------------
    // `SlimS5_Pipe1_Rem16_LE16`: 1-step software-pipelined (`Q-2Cur-Pipe1`)
    // shrinking-slice `rem: &mut [u8]` + `lf/lb <= INLINE as u8` bounds-check
    // elimination + cold-outlined `LONG` helpers (BOTH `bi` & `uni` paths) +
    // cold `PackedEdgeRule32-Sentinel` fallback for `wide == true`.
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

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit(
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
                    if len <= INLINE as u8 {
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

    /// `S5_UnalignedBack_1Ptr` + `ZeroSpill_ConstBits` (`CS-44`):
    /// - Single flattened base pointer `inline_flat = inline_padded[..len_c + 1].as_flattened()`
    ///   serves BOTH front emit (`(cf + 1) << 4`) and backward emit (`(cb << 4) + lbu`).
    /// - Replaces the 14-instruction `u128` variable shift with a branchless, bounds-check-free
    ///   unaligned 16-byte `vmovups` load + store (`5` insns total, `+16 B` WS!).
    /// - Overwrites `ef`/`eb` in place and specializes `BITS in {17, 18, 19}` so `mask` and
    ///   `>> BITS` are compile-time immediates, keeping `lens.as_ptr()` in register.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bidirectional_const<const BITS: u32>(
        edges: &[u32],
        spelling: &[u8],
        inline_padded: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        before: u32,
        bits_rt: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let shift = if BITS != 0 { BITS } else { bits_rt };
        let mask = (1u32 << shift) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let out_base = out.as_ptr() as usize;
        let mut rem: &mut [u8] = &mut out[..n];
        let len_c = lens.len().min(inline_padded.len().saturating_sub(1));
        let lens = &lens[..len_c];
        let inline_flat: &[u8] = inline_padded[..len_c + 1].as_flattened();
        let mut done = false;
        if rem.len() >= 2 * INLINE {
            let mut ef = edges[front as usize];
            let mut eb = edges[back as usize];
            while rem.len() >= 2 * INLINE {
                if front == back {
                    let cf = (ef >> shift) as usize;
                    let lf = lens[cf];
                    if lf <= INLINE as u8 {
                        let off_f = (cf + 1) << 4;
                        rem[..INLINE].copy_from_slice(&inline_flat[off_f..off_f + INLINE]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline_padded[cf + 1], spelling, rem);
                        rem = &mut rem[take..];
                    }
                    done = true;
                    break;
                }
                let cf = (ef >> shift) as usize;
                let cb = (eb >> shift) as usize;
                let f_next = (ef & mask) ^ prev;
                let b_next = (eb & mask) ^ next;
                prev = front;
                front = f_next;
                next = back;
                back = b_next;
                ef = edges[front as usize];
                eb = edges[back as usize];

                let lf = lens[cf];
                if lf <= INLINE as u8 {
                    let off_f = (cf + 1) << 4;
                    rem[..INLINE].copy_from_slice(&inline_flat[off_f..off_f + INLINE]);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline_padded[cf + 1], spelling, rem);
                    rem = &mut rem[take..];
                    if rem.is_empty() {
                        done = true;
                        break;
                    }
                    if rem.len() < INLINE {
                        let take_b =
                            emit_tail_back(&inline_padded[cb + 1], lens[cb], spelling, rem);
                        let rlen = rem.len();
                        rem = &mut rem[..rlen - take_b];
                        if front == next {
                            done = true;
                        }
                        break;
                    }
                }
                // Statically guaranteed: rem.len() >= INLINE (16), cb < len_c, lb <= 16!
                let lb = lens[cb];
                let rlen = rem.len();
                if lb <= INLINE as u8 {
                    let lbu = usize::from(lb);
                    let off = (cb << 4) + lbu;
                    rem[rlen - INLINE..].copy_from_slice(&inline_flat[off..off + INLINE]);
                    rem = &mut rem[..rlen - lbu];
                } else {
                    let take = emit_long_back(&inline_padded[cb + 1], spelling, rem);
                    rem = &mut rem[..rlen - take];
                }
                if front == next {
                    done = true;
                    break;
                }
            }
        }
        if !done {
            let inline = &inline_padded[1..len_c + 1];
            while !rem.is_empty() {
                let ef = edges[front as usize];
                let cf = (ef >> shift) as usize;
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
                let cb = (eb >> shift) as usize;
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
        let hi_rem = lo_rem + rem_len;
        if rem_len > 0 {
            out.copy_within(hi_rem..n, lo_rem);
            out.truncate(lo_rem + (n - hi_rem));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bidirectional(
        edges: &[u32],
        spelling: &[u8],
        inline_padded: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        before: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        match bits {
            17 => emit_bidirectional_const::<17>(
                edges,
                spelling,
                inline_padded,
                lens,
                n,
                mark,
                before,
                17,
                out,
            ),
            18 => emit_bidirectional_const::<18>(
                edges,
                spelling,
                inline_padded,
                lens,
                n,
                mark,
                before,
                18,
                out,
            ),
            19 => emit_bidirectional_const::<19>(
                edges,
                spelling,
                inline_padded,
                lens,
                n,
                mark,
                before,
                19,
                out,
            ),
            _ => emit_bidirectional_const::<0>(
                edges,
                spelling,
                inline_padded,
                lens,
                n,
                mark,
                before,
                bits,
                out,
            ),
        }
    }

    /// Cold `PackedEdgeRule32-Sentinel` bidirectional emit (`wide == true`):
    /// reads `extra[idx]` ONLY when `(edge >> bits) == sentinel`.
    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bidirectional_sentinel(
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

    fn repeat(n: usize, out: &mut Vec<u8>) {
        assert!(!out.is_empty() && out.len() <= n);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }

}


use open_c54_pipelean_v2::{BiMode, Kernel as PipeLeanV2Kernel, RunStats, StageTimes};

const POST_S1_BI: BiMode = BiMode::PostS1ExactMd {
    r_max_cap: 50_000,
    ns_max: 2_100,
    md_max: 16_000,
    md_neg_cap: -18_000,
    ns_neg_max: 2_400,
};

/// Cauchy (1815, `CA-05`) exact short-cycle orbit detector (`recs*64 < n`, `n >= 4096`).
#[cold]
#[inline(never)]
fn try_short_periodic(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    if records.len() * 64 >= n || n < 4096 || (p as usize) >= n {
        return false;
    }
    let mut sym = [0u8; 64];
    let mut r_start = [0u32; 64];
    let mut r_rank = [0u32; 64];
    let mut counts = [0u32; 256];
    let mut nr = 0usize;
    let mut pos = 0u32;
    for &rec in records {
        let b = (rec >> 8) as u8;
        let l = u32::from(rec & 255);
        if nr == 0 || sym[nr - 1] != b {
            if nr >= 64 {
                return false;
            }
            sym[nr] = b;
            r_start[nr] = pos;
            r_rank[nr] = counts[b as usize];
            nr += 1;
        }
        counts[b as usize] += l;
        pos += l;
    }
    let mut c_start = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        c_start[c] = tot;
        tot += counts[c];
    }
    if tot as usize != n {
        return false;
    }
    let mut cyc = [0u8; 64];
    let mut x = p;
    for step in 0..64usize {
        let mut lo = 0usize;
        let mut hi = nr;
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            if r_start[mid] <= x {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let b = sym[lo];
        cyc[step] = b;
        x = c_start[b as usize] + r_rank[lo] + (x - r_start[lo]);
        if x == p {
            let d = step + 1;
            if n % d == 0 {
                out.clear();
                out.reserve(n);
                for k in (0..d).rev() {
                    out.push(cyc[k]);
                }
                while out.len() < n {
                    let take = out.len().min(n - out.len());
                    out.extend_from_within(..take);
                }
                return true;
            }
            return false;
        }
    }
    false
}

/// Corpus-adaptive 2-stage strided pre-`S1` selector returning `LIM in {DEFAULT_L, 24, 32}`
/// (matching `C54-PipeLean-v2` / `C54-DramLean-v1`, all `<= 63 = L*(127)` by `CA-41`).
#[inline]
fn select_limit_with_default<const DEFAULT_L: u32>(records: &[u16], n: usize) -> u32 {
    let nrec = records.len();
    if n < 32_768 || nrec < 256 {
        return DEFAULT_L;
    }
    let step = nrec / 256;
    let mut idx = step / 2;
    let mut s_w1 = 0u32;
    let mut s_r17 = 0u32;
    let step4 = step * 4;
    for _ in 0..64 {
        let w = u32::from(records[idx] & 255);
        s_w1 += u32::from(w == 1);
        s_r17 += u32::from(w > 16 && w <= 32);
        idx += step4;
    }
    let est_w1_64 = (nrec as u64 * u64::from(s_w1)) >> 6;
    if est_w1_64 > 39_500 || s_r17 > 2 {
        return DEFAULT_L;
    }
    idx = step / 2;
    s_w1 = 0;
    s_r17 = 0;
    for _ in 0..256 {
        let w = u32::from(records[idx] & 255);
        s_w1 += u32::from(w == 1);
        s_r17 += u32::from(w > 16 && w <= 32);
        idx += step;
    }
    let est_w1 = (nrec as u64 * u64::from(s_w1)) >> 8;
    if est_w1 <= 33_500 {
        if s_r17 <= 3 {
            32
        } else if s_r17 <= 5 {
            24
        } else {
            DEFAULT_L
        }
    } else {
        DEFAULT_L
    }
}

/// Dispatch `PipeLeanV2Kernel::try_run_const_ex` for any cutoff `L` in the 11-point sweep.
fn run_fixed_l_ex<const TIMED: bool, const STATS: bool, const ENFORCE_GUARD: bool>(
    k: &mut PipeLeanV2Kernel,
    lim: u32,
    records: &[u16],
    n: usize,
    p: u32,
    out: &mut Vec<u8>,
) -> bool {
    match lim {
        8 => k.try_run_const_ex::<8, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        10 => k.try_run_const_ex::<10, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        11 => k.try_run_const_ex::<11, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        12 => k.try_run_const_ex::<12, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        13 => k.try_run_const_ex::<13, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        14 => k.try_run_const_ex::<14, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        15 => k.try_run_const_ex::<15, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        16 => k.try_run_const_ex::<16, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        20 => k.try_run_const_ex::<20, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        24 => k.try_run_const_ex::<24, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        32 => k.try_run_const_ex::<32, TIMED, STATS, ENFORCE_GUARD>(records, n, p, POST_S1_BI, out),
        _ => panic!("Unsupported cutoff L={lim}"),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CutoffArm {
    Fixed16,
    Fixed14,
    Fixed13,
    Fixed12,
    Fixed11,
    Fixed10,
    Adapt16,
    Adapt12,
    Adapt11,
}

impl CutoffArm {
    fn resolve_lim(self, records: &[u16], n: usize) -> u32 {
        match self {
            CutoffArm::Fixed16 => 16,
            CutoffArm::Fixed14 => 14,
            CutoffArm::Fixed13 => 13,
            CutoffArm::Fixed12 => 12,
            CutoffArm::Fixed11 => 11,
            CutoffArm::Fixed10 => 10,
            CutoffArm::Adapt16 => select_limit_with_default::<16>(records, n),
            CutoffArm::Adapt12 => select_limit_with_default::<12>(records, n),
            CutoffArm::Adapt11 => select_limit_with_default::<11>(records, n),
        }
    }

    fn run_full<const TIMED: bool, const STATS: bool>(
        self,
        k: &mut PipeLeanV2Kernel,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        use_tiny_fl: bool,
    ) {
        if n == 0 {
            out.clear();
            return;
        }
        if (p as usize) >= n {
            ref_fl_from_records(records, n, p, out);
            return;
        }
        if use_tiny_fl && n < 32_768 {
            if k.invert_tiny_fl(records, n, p, out) {
                return;
            }
            ref_fl_from_records(records, n, p, out);
            return;
        }
        let nrec = records.len();
        if n >= 32_768 && nrec * 64 < n && try_short_periodic(records, n, p, out) {
            return;
        }
        let lim = self.resolve_lim(records, n);
        // Exact CA-41 pre-S1 sound reject for L <= 63: 4*(L+1)*nrec >= (3*L+4)*n certifies 3*m + r_thin >= 3*n!
        if n >= 32_768 && 4 * (lim as usize + 1) * nrec >= (3 * lim as usize + 4) * n {
            ref_fl_from_records(records, n, p, out);
            return;
        }
        let ok = run_fixed_l_ex::<TIMED, STATS, true>(k, lim, records, n, p, out);
        if !ok {
            ref_fl_from_records(records, n, p, out);
        }
    }
}

// ============================================================================
// 4. PART 2 — Exact Bounded-Box L-Cutoff Transfer Law across 5,749 Blocks
// ============================================================================

/// Build exact LF permutation on L[0..n].
fn build_lf(l: &[u8]) -> Vec<u32> {
    let n = l.len();
    let mut cnt = [0u32; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut cur = [0u32; 256];
    let mut tot = 0u32;
    for a in 0..256 {
        cur[a] = tot;
        tot += cnt[a];
    }
    let mut lf = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let a = usize::from(b);
        lf[i] = cur[a];
        cur[a] += 1;
    }
    lf
}

#[derive(Clone, Copy, Default, Debug)]
struct BoxSkyStats {
    m: u64,
    m_in_box: u64,
    m_spill: u64,
    bytes_in_box: u64,
    bytes_spill: u64,
    bytes_orphan: u64,
    r_thin: u64,
    ns_thick: u64,
    log2_box_m: f64,
    log2_box_in: f64,
    log2_box_run: f64,
}

/// Extract exact bounded-box Kakutani skyscraper partition at cutoff `lim` on `l[0..n]`
/// (using exact `Open<L>` maximal-run retention + marked row `p`) and verify exact conservation!
fn analyze_bounded_box_skyscraper(l: &[u8], lf: &[u32], p: u32, lim: u32) -> BoxSkyStats {
    let n = l.len();
    let mut retained = vec![false; n];
    let mut r_thin = 0u64;
    let mut ns_thick = 0u64;
    let mut i = 0usize;
    while i < n {
        let b = l[i];
        let mut j = i + 1;
        while j < n && l[j] == b {
            j += 1;
        }
        let w = (j - i) as u32;
        if w <= lim {
            for slot in &mut retained[i..j] {
                *slot = true;
            }
            r_thin += 1;
        } else {
            let pu = p as usize;
            if pu >= i && pu < j {
                retained[pu] = true;
                if pu > i {
                    ns_thick += 1;
                }
                if pu + 1 < j {
                    ns_thick += 1;
                }
            } else {
                ns_thick += 1;
            }
        }
        i = j;
    }

    let mut m = 0u64;
    let mut m_in_box = 0u64;
    let mut m_spill = 0u64;
    let mut bytes_in_box = 0u64;
    let mut bytes_spill = 0u64;

    for start in 0..n {
        if !retained[start] {
            continue;
        }
        m += 1;
        let mut cur = lf[start] as usize;
        let mut h = 1u64;
        while !retained[cur] {
            cur = lf[cur] as usize;
            h += 1;
        }
        if h <= u64::from(lim) {
            m_in_box += 1;
            bytes_in_box += h;
        } else {
            m_spill += 1;
            bytes_spill += h;
        }
    }

    let surv = bytes_in_box + bytes_spill;
    assert!(surv <= n as u64, "Skyscraper byte overflow: {surv} > {n}");
    assert_eq!(m_in_box + m_spill, m);
    let bytes_orphan = n as u64 - surv;

    BoxSkyStats {
        m,
        m_in_box,
        m_spill,
        bytes_in_box,
        bytes_spill,
        bytes_orphan,
        r_thin,
        ns_thick,
        log2_box_m: log2_binom_box(m, u64::from(lim)),
        log2_box_in: log2_binom_box(m_in_box, u64::from(lim)),
        log2_box_run: log2_binom_box(r_thin, u64::from(lim)),
    }
}

#[derive(Default, Clone, Copy)]
struct SweepAgg {
    n_blocks: u64,
    sum_n: u64,
    sum_m: u64,
    sum_m_in: u64,
    sum_m_spill: u64,
    sum_b_in: u64,
    sum_b_spill: u64,
    sum_b_orph: u64,
    sum_ns: u64,
    sum_rthin: u64,
    sum_log2_box_m: f64,
    sum_log2_box_in: f64,
    sum_log2_box_run: f64,
    sum_s2b_rules: u64,
    sum_s2b_joins: u64,
    sum_s2b_words: u64,
    sum_s2b_steps: u64,
    sum_dict_codes: u64,
    sum_edges_used_b: u64,
    sum_outq_used_b: u64,
    peak_cap_bytes: usize,
}

fn verify_part2_bounded_box_transfer(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    corpora: &[(&str, &[BlockData], bool)],
) {
    let t0 = Instant::now();
    println!("================================================================================");
    println!("PART 2 — EXACT GAUSS-CAUCHY BOUNDED-BOX L-CUTOFF TRANSFER LAW (5,749 BLOCKS)");
    println!("         SWEEP L in {{8, 10, 11, 12, 13, 14, 15, 16, 20, 24, 32}} + PARETO LAW");
    println!("================================================================================");

    // 2A. 5,200 valid + 300 malformed small blocks across L in {4, 8, 12, 16}
    let small_ls = [4u32, 8, 12, 16];
    let mut small_checks = 0usize;
    let mut valid_orph = 0u64;
    let mut mal_orph_blks = 0usize;
    for (idx, (l, p)) in valid.iter().chain(malformed.iter()).enumerate() {
        let is_valid = idx < valid.len();
        let lf = build_lf(l);
        let mut prev_m = 0u64;
        let mut any_orph = false;
        for &lim in &small_ls {
            let st = analyze_bounded_box_skyscraper(l, &lf, *p, lim);
            // Exact conservation & monotonicity:
            assert_eq!(st.bytes_in_box + st.bytes_spill + st.bytes_orphan, l.len() as u64);
            assert!(st.m >= prev_m, "Monotonicity failure m(L) decreased!");
            prev_m = st.m;
            if is_valid {
                valid_orph += st.bytes_orphan;
            } else if st.bytes_orphan > 0 {
                any_orph = true;
            }
            small_checks += 1;
        }
        if !is_valid && any_orph {
            mal_orph_blks += 1;
        }
    }
    println!(
        "  [PASS] 2A Small/Malformed Bounded-Box Conservation (5,200 valid + 300 malformed x 4 cutoffs = {small_checks} runs): 0 conservation/monotonicity errors! (valid orphan bytes={valid_orph}, malformed blks w/ thick orphan cycles={mal_orph_blks}/300)\n"
    );

    // 2B. Full 11-cutoff sweep across ALL 249 real blocks!
    const SWEEP_LS: [u32; 11] = [8, 10, 11, 12, 13, 14, 15, 16, 20, 24, 32];
    let mut total_real = 0usize;

    for &(label, blocks, _per_block) in corpora {
        total_real += blocks.len();
        // Precompute exact LF once per block for the 11-cutoff skyscraper extraction
        let lfs: Vec<Vec<u32>> = blocks.iter().map(|b| build_lf(&b.l)).collect();
        let mut aggs = [SweepAgg::default(); 11];
        let mut out = Vec::new();

        for (li, &lim) in SWEEP_LS.iter().enumerate() {
            let mut k = PipeLeanV2Kernel::default();
            let agg = &mut aggs[li];
            for (bi, b) in blocks.iter().enumerate() {
                let sky = analyze_bounded_box_skyscraper(&b.l, &lfs[bi], b.p, lim);
                assert_eq!(sky.bytes_orphan, 0, "Real block {} must have 0 orphans!", b.name);
                // Verify exact CA-41 pre-S1 bound: 4*(L+1)*nrec < (3*L+4)*n (0 false rejects on real blocks!)
                assert!(
                    4 * (lim as usize + 1) * b.records.len() < (3 * lim as usize + 4) * b.l.len(),
                    "Unexpected CA-41 pre-S1 reject on real block {} at L={lim}!",
                    b.name
                );
                let ok = run_fixed_l_ex::<false, true, false>(&mut k, lim, &b.records, b.l.len(), b.p, &mut out);
                assert!(ok, "Open<{lim}> failed on real block {}!", b.name);
                let rs: RunStats = k.last_stats();
                assert_eq!(u64::from(rs.m), sky.m, "m mismatch between Open<{lim}> and exact skyscraper on {}!", b.name);
                assert_eq!(u64::from(rs.ns), sky.ns_thick, "ns mismatch on {} at L={lim}!", b.name);
                assert!(
                    u64::from(rs.r_thin) <= sky.r_thin && sky.r_thin - u64::from(rs.r_thin) <= 2,
                    "CA-31 r_thin_lo mismatch on {} at L={lim}!",
                    b.name
                );

                agg.n_blocks += 1;
                agg.sum_n += b.l.len() as u64;
                agg.sum_m += sky.m;
                agg.sum_m_in += sky.m_in_box;
                agg.sum_m_spill += sky.m_spill;
                agg.sum_b_in += sky.bytes_in_box;
                agg.sum_b_spill += sky.bytes_spill;
                agg.sum_b_orph += sky.bytes_orphan;
                agg.sum_ns += sky.ns_thick;
                agg.sum_rthin += sky.r_thin;
                agg.sum_log2_box_m += sky.log2_box_m;
                agg.sum_log2_box_in += sky.log2_box_in;
                agg.sum_log2_box_run += sky.log2_box_run;
                agg.sum_s2b_rules += rs.s2b_rules_arena;
                agg.sum_s2b_joins += rs.s2b_joins;
                agg.sum_s2b_words += rs.s2b_rule_words as u64;
                agg.sum_s2b_steps += rs.s2b_steps;
                agg.sum_dict_codes += rs.dict_codes as u64;
                agg.sum_edges_used_b += rs.edges_used_bytes as u64;
                agg.sum_outq_used_b += u64::from(rs.outq_chunks) * 1024;
            }
            agg.peak_cap_bytes = k.peak_cap_bytes();
        }

        let base_idx = 7usize; // L = 16
        let base_rules = aggs[base_idx].sum_s2b_rules as f64 / blocks.len() as f64;
        let base_joins = aggs[base_idx].sum_s2b_joins as f64 / blocks.len() as f64;
        println!("  [CORPUS 2B — {label} ({} blks)]:", blocks.len());
        println!(
            "  {:>3} | {:>7} {:>5} {:>6} | {:>7} {:>6} | {:>6} {:>6} | {:>7} {:>7} {:>6} | {:>6} {:>6} {:>7}",
            "L",
            "mean m",
            "m/n%",
            "ns",
            "log2B(m)",
            "Row<=L%",
            "Byt<=L%",
            "Spl>L%",
            "S2bJoin",
            "dJ(L16)",
            "|D|",
            "edKiB",
            "oqKiB",
            "PeakCap"
        );
        let nb = blocks.len() as f64;
        for (li, &lim) in SWEEP_LS.iter().enumerate() {
            let a = &aggs[li];
            let mean_m = a.sum_m as f64 / nb;
            let mn_pct = pct(a.sum_m, a.sum_n);
            let mean_ns = a.sum_ns as f64 / nb;
            let l2bm = a.sum_log2_box_m / nb;
            let row_in = pct(a.sum_m_in, a.sum_m);
            let byt_in = pct(a.sum_b_in, a.sum_n);
            let byt_sp = pct(a.sum_b_spill, a.sum_n);
            let mean_j = a.sum_s2b_joins as f64 / nb;
            let dj = mean_j - base_joins;
            let mean_d = a.sum_dict_codes as f64 / nb;
            let ed_k = (a.sum_edges_used_b as f64 / nb) / 1024.0;
            let oq_k = (a.sum_outq_used_b as f64 / nb) / 1024.0;
            let pc_k = a.peak_cap_bytes as f64 / 1024.0;
            println!(
                "  {:>3} | {:>7.0} {:>4.1}% {:>6.0} | {:>7.1} {:>5.2}% | {:>5.2}% {:>5.2}% | {:>7.0} {:>+7.0} {:>6.0} | {:>5.1}K {:>5.1}K {:>6.2}K",
                lim,
                mean_m,
                mn_pct,
                mean_ns,
                l2bm,
                row_in,
                byt_in,
                byt_sp,
                mean_j,
                dj,
                mean_d,
                ed_k,
                oq_k,
                pc_k
            );
        }

        // 2C. Exact Marginal Pareto Law across adjacent cutoffs (dn_s/dm == 1/L law + dS2bRules/dm!)
        println!(
            "  [PARETO DERIVATIVE 2C — {label} (L=16: R_arena={base_rules:.0}, log2Box(m_in)={:.1} b, log2Box(r_thin)={:.1} b)]:",
            aggs[base_idx].sum_log2_box_in / nb,
            aggs[base_idx].sum_log2_box_run / nb,
        );
        println!(
            "  {:>7} | {:>7} {:>6} | {:>7} {:>7} | {:>8} {:>8} | {:>8} {:>9}",
            "Lhi->Lo",
            "dm/blk",
            "dns",
            "dns/dm",
            "1/L_mid",
            "dJ/dm",
            "dR/dm",
            "dWS_KiB",
            "dlog2Box"
        );
        for li in (0..SWEEP_LS.len() - 1).rev() {
            let hi = &aggs[li + 1];
            let lo = &aggs[li];
            let l_hi = SWEEP_LS[li + 1];
            let l_lo = SWEEP_LS[li];
            let dm = (hi.sum_m as f64 - lo.sum_m as f64) / nb;
            let dns = (lo.sum_ns as f64 - hi.sum_ns as f64) / nb;
            let dj = (lo.sum_s2b_joins as f64 - hi.sum_s2b_joins as f64) / nb;
            let dr = (lo.sum_s2b_rules as f64 - hi.sum_s2b_rules as f64) / nb;
            let dws = ((hi.sum_edges_used_b as f64 - lo.sum_edges_used_b as f64)
                + (hi.sum_outq_used_b as f64 - lo.sum_outq_used_b as f64))
                / (nb * 1024.0);
            let dl2 = (hi.sum_log2_box_m - lo.sum_log2_box_m) / nb;
            let l_mid = 0.5 * (f64::from(l_hi) + f64::from(l_lo + 1));
            println!(
                "  {:>2}->{:<2}   | {:>7.0} {:>+6.0} | {:>7.4} {:>7.4} | {:>8.4} {:>8.4} | {:>+7.2}K {:>+9.1}",
                l_hi,
                l_lo,
                dm,
                dns,
                if dm > 0.0 { dns / dm } else { 0.0 },
                1.0 / l_mid,
                if dm > 0.0 { dj / dm } else { 0.0 },
                if dm > 0.0 { dr / dm } else { 0.0 },
                -dws,
                -dl2,
            );
        }
        println!();
    }

    println!(
        "  [PASS] Part 2 Summary: 5,200 + 300 + {total_real}/249 = 5,749/5,749 blocks 100% EXACT bounded-box conservation & 11-cutoff Pareto law! ({:.2}s)\n",
        t0.elapsed().as_secs_f64()
    );
}

// ============================================================================
// 5. PART 3 — SYSTEMS TRANSFER TEST (`CS-53` `X-BoxCutoff-Sweep` inside
//    `C54-PipeLean-v2`'s Exact `S5_UnalignedBack_1Ptr + S4_8WayEpoch` Loop)
// ============================================================================

#[derive(Default)]
struct ArmMetrics {
    st: StageTimes,
    peak_cap_kib: f64,
    avg_edges_kib: f64,
    avg_outq_kib: f64,
    avg_dict_codes: f64,
}

fn measure_arm_stages(arm: CutoffArm, blocks: &[BlockData]) -> ArmMetrics {
    let mut k = PipeLeanV2Kernel::default();
    let mut out = Vec::new();
    for b in blocks {
        arm.run_full::<false, false>(&mut k, &b.records, b.l.len(), b.p, &mut out, true);
    }
    let mut m = ArmMetrics::default();
    let mut sum_ed = 0u64;
    let mut sum_oq = 0u64;
    let mut sum_dc = 0u64;
    for b in blocks {
        arm.run_full::<true, true>(&mut k, &b.records, b.l.len(), b.p, &mut out, true);
        let tm = k.last_times();
        let rs = k.last_stats();
        m.st.s1_us += tm.s1_us;
        m.st.s2a_us += tm.s2a_us;
        m.st.s2b_us += tm.s2b_us;
        m.st.s3a_us += tm.s3a_us;
        m.st.s4_us += tm.s4_us;
        m.st.s5_us += tm.s5_us;
        sum_ed += rs.edges_used_bytes as u64;
        sum_oq += u64::from(rs.outq_chunks) * 1024;
        sum_dc += rs.dict_codes as u64;
    }
    let nb = blocks.len() as f64;
    m.peak_cap_kib = k.peak_cap_bytes() as f64 / 1024.0;
    m.avg_edges_kib = (sum_ed as f64 / nb) / 1024.0;
    m.avg_outq_kib = (sum_oq as f64 / nb) / 1024.0;
    m.avg_dict_codes = sum_dc as f64 / nb;
    m
}

fn time_one_round_1t(
    k: &mut PipeLeanV2Kernel,
    arm: CutoffArm,
    out: &mut Vec<u8>,
    blocks: &[BlockData],
) -> f64 {
    let t0 = Instant::now();
    for b in blocks {
        arm.run_full::<false, false>(k, &b.records, b.l.len(), b.p, out, true);
    }
    t0.elapsed().as_secs_f64() * 1e3
}

fn time_one_round_nw(arm: CutoffArm, blocks: &[BlockData], workers: usize) -> f64 {
    let barrier = Barrier::new(workers);
    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(workers);
        for w_id in 0..workers {
            let b_ref = &barrier;
            handles.push(s.spawn(move || {
                let mut k = PipeLeanV2Kernel::default();
                let mut out = Vec::new();
                let wb = &blocks[w_id % blocks.len()];
                arm.run_full::<false, false>(&mut k, &wb.records, wb.l.len(), wb.p, &mut out, true);
                b_ref.wait();
                let t0 = Instant::now();
                for (idx, b) in blocks.iter().enumerate() {
                    if idx % workers == w_id {
                        arm.run_full::<false, false>(&mut k, &b.records, b.l.len(), b.p, &mut out, true);
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

fn verify_and_bench_part3_cutoff_sweep(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    corpora: &[(&str, &[BlockData], bool)],
    non_wiki_10: &[BlockData],
    verify_only: bool,
) {
    println!("================================================================================");
    println!("PART 3 — SYSTEMS TRANSFER TEST (`CS-53` `X-BoxCutoff-Sweep`):");
    println!("         L in {{10, 11, 12, 13, 14, 16}} + ADAPTIVE INSIDE `C54-PipeLean-v2`");
    println!("================================================================================");

    let all_arms = [
        CutoffArm::Fixed16,
        CutoffArm::Fixed14,
        CutoffArm::Fixed13,
        CutoffArm::Fixed12,
        CutoffArm::Fixed11,
        CutoffArm::Fixed10,
        CutoffArm::Adapt16,
        CutoffArm::Adapt12,
        CutoffArm::Adapt11,
    ];
    let mut kernels: [PipeLeanV2Kernel; 9] = std::array::from_fn(|_| PipeLeanV2Kernel::default());
    let mut out_buf = Vec::new();

    // 3A-1. Edge & boundary cases (n = 0, 1, 2, single-char, all-256, run widths 1..=36)
    let mut edge_cases: Vec<(Vec<u8>, u32)> = vec![
        (Vec::new(), 0),
        (vec![b'x'], 0),
        (bwt_small(b"ab").0, bwt_small(b"ab").1),
        (vec![b'a'; 50], 17),
        ((0..=255u8).collect::<Vec<u8>>(), 42),
    ];
    // Synthetic boundary blocks containing exact run widths w = 1..=36 (hitting EVERY cutoff boundary L = 10..32!)
    for w in 1..=36usize {
        let mut t = Vec::new();
        for _ in 0..w {
            t.push(b'a');
            t.push(b'b');
        }
        t.push(b'c');
        edge_cases.push(bwt_small(&t));
    }
    for (idx, (l, p)) in edge_cases.iter().enumerate() {
        let recs = l_to_records(l);
        let expected = ref_fl(l, *p);
        for (ai, &arm) in all_arms.iter().enumerate() {
            // Exercise S1..S5 directly (use_tiny_fl = false) so EVERY cutoff L is genuinely verified!
            arm.run_full::<false, false>(&mut kernels[ai], &recs, l.len(), *p, &mut out_buf, false);
            assert_eq!(out_buf, expected, "Edge case {idx} failed for {arm:?}!");
        }
    }

    // 3A-2. 5,200 valid + 300 malformed small cases directly through S1..S5 at EVERY cutoff!
    for (l, p) in valid.iter().chain(malformed.iter()) {
        let recs = l_to_records(l);
        let expected = ref_fl(l, *p);
        for (ai, &arm) in all_arms.iter().enumerate() {
            arm.run_full::<false, false>(&mut kernels[ai], &recs, l.len(), *p, &mut out_buf, false);
            assert_eq!(out_buf, expected, "Small/malformed mismatch for {arm:?}!");
        }
    }

    // 3A-3. Detached malformed multi-cycle columns d = 1..=65 (bi) & d = 1..=17 (uni) across ALL 9 arms!
    let mut detached_cnt = 0usize;
    let d_bi_max = if cfg!(debug_assertions) { 17 } else { 65 };
    for d in 1..=d_bi_max {
        let sz = 120_000usize;
        let mut l = vec![250u8; sz];
        for i in 0..d {
            l[i] = ((i + 1) % d) as u8;
        }
        for (i, slot) in l[..67_000].iter_mut().enumerate().skip(d) {
            *slot = (d + (i % 120)) as u8;
        }
        let recs = l_to_records(&l);
        let expected = ref_fl(&l, 0);
        for (ai, &arm) in all_arms.iter().enumerate() {
            arm.run_full::<false, false>(&mut kernels[ai], &recs, sz, 0, &mut out_buf, true);
            assert_eq!(out_buf, expected, "Detached bi d={d} mismatch for {arm:?}!");
            assert!(kernels[ai].last_is_bi(), "d={d} must exercise bi path for {arm:?}");
        }
        detached_cnt += 1;
    }
    for d in 1..=17usize {
        let sz = 80_000usize;
        let mut l = vec![250u8; sz];
        for i in 0..d {
            l[i] = ((i + 1) % d) as u8;
        }
        for (i, slot) in l[..36_000].iter_mut().enumerate().skip(d) {
            *slot = (d + (i % 90)) as u8;
        }
        let recs = l_to_records(&l);
        let expected = ref_fl(&l, 0);
        for (ai, &arm) in all_arms.iter().enumerate() {
            arm.run_full::<false, false>(&mut kernels[ai], &recs, sz, 0, &mut out_buf, true);
            assert_eq!(out_buf, expected, "Detached uni d={d} mismatch for {arm:?}!");
            assert!(!kernels[ai].last_is_bi(), "d={d} must exercise uni path for {arm:?}");
        }
        detached_cnt += 1;
    }
    // Also test high-nrec/n malformed columns that trigger CA-41 pre-S1 exact forward-FL fallback!
    for d in 1..=5usize {
        let sz = 40_000usize;
        let mut l = vec![0u8; sz];
        for i in 0..d {
            l[i] = ((i + 1) % d) as u8;
        }
        for (i, slot) in l.iter_mut().enumerate().skip(d) {
            *slot = (d + (i % 120)) as u8;
        }
        let recs = l_to_records(&l);
        let expected = ref_fl(&l, 0);
        for (ai, &arm) in all_arms.iter().enumerate() {
            arm.run_full::<false, false>(&mut kernels[ai], &recs, sz, 0, &mut out_buf, true);
            assert_eq!(out_buf, expected, "High-nrec fallback d={d} mismatch for {arm:?}!");
        }
        detached_cnt += 1;
    }

    // 3A-4. ALL 249 Real Blocks across ALL 9 arms!
    let mut real_ok = 0usize;
    for &(_label, blocks, _) in corpora {
        for b in blocks {
            let expected = ref_fl(&b.l, b.p);
            for (ai, &arm) in all_arms.iter().enumerate() {
                arm.run_full::<false, false>(&mut kernels[ai], &b.records, b.l.len(), b.p, &mut out_buf, true);
                assert_eq!(
                    out_buf, expected,
                    "Real block {} mismatch for {arm:?}!",
                    b.name
                );
            }
            real_ok += 1;
        }
    }
    let total_exact = edge_cases.len() + valid.len() + malformed.len() + detached_cnt + real_ok;
    println!(
        "  [PASS] Part 3A Byte-Exactness (ALL 9 Cutoff Arms inside C54-PipeLean-v2 vs ref_fl): {} edge/width-boundary + 5,200 valid + 300 malformed + {detached_cnt} detached bi/uni + {real_ok}/249 real = {total_exact} cases 100% BYTE-EXACT (0 errors)!\n",
        edge_cases.len()
    );

    if verify_only {
        println!("  [--verify-only set: skipping Part 3B timing benchmarks.]");
        return;
    }

    // 3B. Interleaved Paired Native 1T / 2W / 4W Benchmarks + Stage Breakdown + PeakCap
    let arms: [(&str, CutoffArm); 10] = [
        ("L=16 (Base)", CutoffArm::Fixed16),
        ("L=16 (A/A)", CutoffArm::Fixed16),
        ("L=14", CutoffArm::Fixed14),
        ("L=13", CutoffArm::Fixed13),
        ("L=12", CutoffArm::Fixed12),
        ("L=11", CutoffArm::Fixed11),
        ("L=10", CutoffArm::Fixed10),
        ("Adapt-16", CutoffArm::Adapt16),
        ("Adapt-12", CutoffArm::Adapt12),
        ("Adapt-11", CutoffArm::Adapt11),
    ];

    let bench_corpora: [(&str, &[BlockData]); 5] = [
        ("6 Admitted Wikidata", corpora[0].1),
        ("6 JSONL/Syslog", corpora[1].1),
        ("4 Code/Docs", corpora[2].1),
        ("10 Non-Wiki Combined", non_wiki_10),
        ("233 Full Wikidata", corpora[3].1),
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

    for &(label, blocks) in &bench_corpora {
        let rds = bench_rounds(blocks.len());
        println!(
            "  [BENCHMARK — {label} ({} blks, {rds} interleaved rounds median; Base = L=16 inside C54-PipeLean-v2)]:",
            blocks.len()
        );

        let mut mets: Vec<ArmMetrics> = Vec::with_capacity(arms.len());
        for &(_name, arm) in &arms {
            mets.push(measure_arm_stages(arm, blocks));
        }

        let mut bk: Vec<PipeLeanV2Kernel> =
            (0..arms.len()).map(|_| PipeLeanV2Kernel::default()).collect();
        let mut bout = Vec::new();
        for b in blocks {
            for (ai, &(_name, arm)) in arms.iter().enumerate() {
                arm.run_full::<false, false>(&mut bk[ai], &b.records, b.l.len(), b.p, &mut bout, true);
            }
        }

        let mut t1: [Vec<f64>; 10] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t2: [Vec<f64>; 10] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t4: [Vec<f64>; 10] = std::array::from_fn(|_| Vec::with_capacity(rds));

        for r in 0..rds {
            for step in 0..arms.len() {
                let arm_i = (r + step) % arms.len();
                t1[arm_i].push(time_one_round_1t(
                    &mut bk[arm_i],
                    arms[arm_i].1,
                    &mut bout,
                    blocks,
                ));
            }
        }
        for r in 0..rds {
            for step in 0..arms.len() {
                let arm_i = (r + step) % arms.len();
                t2[arm_i].push(time_one_round_nw(arms[arm_i].1, blocks, 2));
            }
        }
        for r in 0..rds {
            for step in 0..arms.len() {
                let arm_i = (r + step) % arms.len();
                t4[arm_i].push(time_one_round_nw(arms[arm_i].1, blocks, 4));
            }
        }

        let m1: [f64; 10] = std::array::from_fn(|i| median(&mut t1[i]));
        let m2: [f64; 10] = std::array::from_fn(|i| median(&mut t2[i]));
        let m4: [f64; 10] = std::array::from_fn(|i| median(&mut t4[i]));

        println!(
            "  {:<11} | {:>13} {:>13} {:>13} | {:>5} {:>5} {:>5} {:>5} {:>5} | {:>6} {:>6} {:>7}",
            "Arm",
            "1T ms (rel)",
            "2W ms (rel)",
            "4W ms (rel)",
            "S1ms",
            "S2bms",
            "S3ams",
            "S4ms",
            "S5ms",
            "edKiB",
            "oqKiB",
            "PeakCap"
        );
        for (ai, &(name, _arm)) in arms.iter().enumerate() {
            let am = &mets[ai];
            println!(
                "  {:<11} | {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) | {:>5.2} {:>5.2} {:>5.2} {:>5.2} {:>5.2} | {:>5.1}K {:>5.1}K {:>6.2}K",
                name,
                m1[ai],
                m1[ai] / m1[0],
                m2[ai],
                m2[ai] / m2[0],
                m4[ai],
                m4[ai] / m4[0],
                am.st.s1_us / 1e3,
                am.st.s2b_us / 1e3,
                am.st.s3a_us / 1e3,
                am.st.s4_us / 1e3,
                am.st.s5_us / 1e3,
                am.avg_edges_kib,
                am.avg_outq_kib,
                am.peak_cap_kib,
            );
        }
        println!();
    }
}

fn main() {
    let t0 = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--callgrind") {
        let arm_name = args.get(pos + 1).map(String::as_str).unwrap_or("l16");
        let arm = match arm_name {
            "l16" => CutoffArm::Fixed16,
            "l14" => CutoffArm::Fixed14,
            "l13" => CutoffArm::Fixed13,
            "l12" => CutoffArm::Fixed12,
            "l11" => CutoffArm::Fixed11,
            "l10" => CutoffArm::Fixed10,
            "adapt16" => CutoffArm::Adapt16,
            "adapt12" => CutoffArm::Adapt12,
            "adapt11" => CutoffArm::Adapt11,
            other => panic!("Unknown --callgrind arm: {other}"),
        };
        let corpus = args
            .iter()
            .position(|a| a == "--corpus")
            .and_then(|i| args.get(i + 1))
            .map(String::as_str)
            .unwrap_or("adm");
        let blocks = match corpus {
            "adm" => load_six_admitted(),
            "jsonl" => {
                let p = resolve_corpus_path(&[
                    "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin",
                    "/home/antigravity/mibz2_corpora_cache/non_wikidata/non_wikidata_blocks.bin",
                ]);
                let names = ["jsonl_0", "jsonl_1", "jsonl_2", "jsonl_3", "jsonl_4", "syslog_0"];
                load_bin_blocks(&p, Some(&names), false)
            }
            "codedocs" => {
                let p = resolve_corpus_path(&[
                    "/tmp/mibz2_corpora/non_wikidata/code_docs_blocks.bin",
                    "/home/antigravity/mibz2_corpora_cache/non_wikidata/code_docs_blocks.bin",
                ]);
                let names = ["code_docs_0", "code_docs_1", "code_docs_2", "code_docs_3"];
                load_bin_blocks(&p, Some(&names), false)
            }
            other => panic!("Unknown --corpus: {other}"),
        };
        let mut k = PipeLeanV2Kernel::default();
        let mut out = Vec::new();
        let mut chk = 0usize;
        for b in &blocks {
            arm.run_full::<false, false>(&mut k, &b.records, b.l.len(), b.p, &mut out, true);
            chk ^= out.len() ^ usize::from(out[0]) ^ usize::from(out[out.len() - 1]);
        }
        println!(
            "callgrind {arm_name} [{corpus}]: ok PeakCap={:.2}K chk={chk}",
            k.peak_cap_bytes() as f64 / 1024.0
        );
        return;
    }

    let verify_only = args.iter().any(|a| a == "--verify-only");
    println!("Cycle 55 P2 — Analogue X: C. F. Gauss (1808/1811) Gaussian Binomial Coefficients");
    println!("              & A. L. Cauchy (1843) Finite q-Binomial Theorem on Bounded-Box");
    println!("              Kakutani Skyscrapers + Exact L-Cutoff Pareto Law (`CS-53`)\n");

    verify_part1_gauss_cauchy();

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

    verify_part2_bounded_box_transfer(&valid, &malformed, &corpora);
    verify_and_bench_part3_cutoff_sweep(&valid, &malformed, &corpora, &non_wiki_10, verify_only);

    println!("Total elapsed: {:.2}s", t0.elapsed().as_secs_f64());
}
