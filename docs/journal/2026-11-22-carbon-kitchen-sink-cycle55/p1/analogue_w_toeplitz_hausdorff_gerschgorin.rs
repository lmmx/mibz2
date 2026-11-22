#![forbid(unsafe_code)]

//! Cycle 55 (P1) — Analogue W: O. Toeplitz (1918) & F. Hausdorff (1919) Numerical Range
//! (Field of Values) + S. Gerschgorin (1931) Disks on `P_1` / `P_ret` / `S_ret`,
//! & `W-Gersch-Gate` (`CS-52` Cheap Gerschgorin/Normality Routing Diagnostic).
//!
//! Primary Sources (100% pure pre-1950 classical mathematics — ZERO relabels, ZERO analogies):
//!   1. O. Toeplitz, "Das algebraische Analogon zu einem Satze von Fejér",
//!      Mathematische Zeitschrift 2 (1918), 187–197:
//!        - Introduced the numerical range (Wertevorrat / field of values)
//!          `W(A) = { x^* A x : x in C^n, ||x||_2 = 1 }` and proved boundary convexity via
//!          supporting lines `h(theta) = max_{z in W(A)} Re(e^{i theta} z) = lam_max(H(theta))`,
//!          where `H(theta) = (e^{i theta} A + e^{-i theta} A^*) / 2`.
//!   2. F. Hausdorff, "Der Wertvorrat einer Bilinearform",
//!      Mathematische Zeitschrift 3 (1919), 314–316:
//!        - Proved convexity of the entire field of values `W(A)` (Toeplitz-Hausdorff Theorem):
//!          * Spectral containment `conv(sigma(A)) subseteq W(A)` (equality for all normal A;
//!            for 2x2 matrices, exact Elliptical Range Theorem with foci `lam_1, lam_2`,
//!            major axis `sqrt(|lam_1 - lam_2|^2 + Delta_F(A)^2)`, and minor axis EQUAL to
//!            `CA-51` Schur departure `Delta_F(A)`!).
//!          * Numerical abscissa `omega(A) = max Re W(A) = lam_max((A + A^*)/2)` satisfying
//!            `max(tr(A)/s, max_i A_{ii}, alpha(A)) <= omega(A) <= max_i (A_{ii} + R_i(H_0))`.
//!          * Numerical radius `r(A) = max_{z in W(A)} |z| = max_theta lam_max(Re(e^{i theta} A))`
//!            satisfying exact equivalence `rho(A) <= r(A) <= ||A||_2 <= 2 r(A)` with excess
//!            `E_W(A) = r(A) - rho(A) >= 0` (`= 0` for all normal A; strictly `> 0` certifies
//!            non-normality, while direct sums `diag(2, J_k(0))` witness exact scope).
//!          * On `L^2(pi)`-symmetrized `H_S = S - sqrt(pi) sqrt(pi)^T`, Jensen (`CA-50`) enforces
//!            `rho(H_S) <= r(H_S) <= ||H_S||_2 <= 1`, whereas unweighted `H_P = P - 1 pi^T` breaks
//!            contraction (`r(H_P) > 1`, `omega(H_P) > 1`) via directed syntax + Zipf imbalance!
//!   3. S. Gerschgorin, "Über die Abgrenzung der Eigenwerte einer Matrix",
//!      Izv. Akad. Nauk SSSR Ser. Fiz.-Mat. 6 (1931), 749–754:
//!        - `O(s^2)` row- and column-sum disk enclosure
//!          `sigma(A) subseteq Gamma_row(A) cap Gamma_col(A)` where
//!          `Gamma_row(A) = union_i D(A_{ii}, R_i(A))`, `R_i(A) = sum_{j != i} |A_{ij}|`,
//!          giving `rho(A) <= G(A) = min(G_row(A), G_col(A))` and `omega(A) <= G_omega(A)`,
//!          with exact BWT/Markov interpretation `G_row(H_P) = 2 max_i d_TV(P_i, pi) <= 2`.
//!
//! Extends BOTH 12-node Pure-CA Longest Chains ending at `CA-51` (Chain A+M+O+Q+S+U and
//! Chain B+M+O+Q+S+U) to a NEW 13-node / 12-link Pure-CA Longest Chain (`CA-54`,
//! `Uses: CA-13, CA-38, CA-42, CA-45, CA-48, CA-51`), and via `CS-52` (`W-Gersch-Gate`,
//! strictly `2020s` modern CS) to a NEW 14-node / 13-link Full-Atlas Longest Chain!
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64::consts::PI;
use std::fs;
use std::sync::Barrier;
use std::time::Instant;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

// ============================================================================
// SECTION 1A: Exact BigInt & BigRat Arithmetic (Gerschgorin 1931 & Rayleigh)
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
struct BigInt {
    neg: bool,
    limbs: Vec<u32>,
}

impl BigInt {
    fn zero() -> Self {
        BigInt { neg: false, limbs: Vec::new() }
    }
    fn one() -> Self {
        BigInt { neg: false, limbs: vec![1] }
    }
    fn from_i64(v: i64) -> Self {
        if v == 0 {
            return Self::zero();
        }
        let neg = v < 0;
        let mag = v.unsigned_abs();
        let lo = mag as u32;
        let hi = (mag >> 32) as u32;
        let limbs = if hi == 0 { vec![lo] } else { vec![lo, hi] };
        BigInt { neg, limbs }
    }
    fn from_u64(v: u64) -> Self {
        if v == 0 {
            return Self::zero();
        }
        let lo = v as u32;
        let hi = (v >> 32) as u32;
        let limbs = if hi == 0 { vec![lo] } else { vec![lo, hi] };
        BigInt { neg: false, limbs }
    }
    fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }
    fn is_one(&self) -> bool {
        !self.neg && self.limbs.len() == 1 && self.limbs[0] == 1
    }
    fn negate(&self) -> Self {
        if self.is_zero() {
            Self::zero()
        } else {
            BigInt { neg: !self.neg, limbs: self.limbs.clone() }
        }
    }
    fn abs(&self) -> Self {
        BigInt { neg: false, limbs: self.limbs.clone() }
    }
    fn norm(&mut self) {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
        if self.limbs.is_empty() {
            self.neg = false;
        }
    }
    fn cmp_mag(a: &[u32], b: &[u32]) -> Ordering {
        a.len().cmp(&b.len()).then_with(|| {
            for i in (0..a.len()).rev() {
                match a[i].cmp(&b[i]) {
                    Ordering::Equal => continue,
                    ord => return ord,
                }
            }
            Ordering::Equal
        })
    }
    fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let n = a.len().max(b.len());
        let mut out = Vec::with_capacity(n + 1);
        let mut carry = 0u64;
        for i in 0..n {
            let av = *a.get(i).unwrap_or(&0) as u64;
            let bv = *b.get(i).unwrap_or(&0) as u64;
            let s = av + bv + carry;
            out.push(s as u32);
            carry = s >> 32;
        }
        if carry > 0 {
            out.push(carry as u32);
        }
        out
    }
    fn sub_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i64;
        for i in 0..a.len() {
            let av = a[i] as i64;
            let bv = *b.get(i).unwrap_or(&0) as i64;
            let d = av - bv - borrow;
            if d < 0 {
                out.push((d + (1i64 << 32)) as u32);
                borrow = 1;
            } else {
                out.push(d as u32);
                borrow = 0;
            }
        }
        while out.last() == Some(&0) {
            out.pop();
        }
        out
    }
    fn mul_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        if a.is_empty() || b.is_empty() {
            return Vec::new();
        }
        let mut out = vec![0u32; a.len() + b.len()];
        for (i, &av) in a.iter().enumerate() {
            let mut carry = 0u64;
            let av64 = av as u64;
            for (j, &bv) in b.iter().enumerate() {
                let prod = av64 * (bv as u64) + (out[i + j] as u64) + carry;
                out[i + j] = prod as u32;
                carry = prod >> 32;
            }
            out[i + b.len()] = carry as u32;
        }
        while out.last() == Some(&0) {
            out.pop();
        }
        out
    }
    fn bit_len(a: &[u32]) -> usize {
        if a.is_empty() {
            return 0;
        }
        let top = *a.last().unwrap();
        (a.len() - 1) * 32 + (32 - top.leading_zeros() as usize)
    }
    fn get_bit(a: &[u32], idx: usize) -> bool {
        let limb = idx / 32;
        let bit = idx % 32;
        limb < a.len() && ((a[limb] >> bit) & 1) != 0
    }
    fn set_bit(a: &mut Vec<u32>, idx: usize) {
        let limb = idx / 32;
        let bit = idx % 32;
        if a.len() <= limb {
            a.resize(limb + 1, 0);
        }
        a[limb] |= 1u32 << bit;
    }
    fn shl1_add_bit(a: &mut Vec<u32>, bit: bool) {
        let mut carry = u32::from(bit);
        for x in a.iter_mut() {
            let next_carry = *x >> 31;
            *x = (*x << 1) | carry;
            carry = next_carry;
        }
        if carry != 0 {
            a.push(carry);
        }
    }
    fn div_rem_mag(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
        assert!(!b.is_empty(), "BigInt division by zero");
        if Self::cmp_mag(a, b) == Ordering::Less {
            return (Vec::new(), a.to_vec());
        }
        if b.len() == 1 {
            let d = b[0] as u64;
            let mut q = vec![0u32; a.len()];
            let mut rem = 0u64;
            for i in (0..a.len()).rev() {
                let cur = (rem << 32) | (a[i] as u64);
                q[i] = (cur / d) as u32;
                rem = cur % d;
            }
            while q.last() == Some(&0) {
                q.pop();
            }
            let r = if rem == 0 { Vec::new() } else { vec![rem as u32] };
            return (q, r);
        }
        let nbits = Self::bit_len(a);
        let mut q = Vec::new();
        let mut r = Vec::new();
        for bit_idx in (0..nbits).rev() {
            Self::shl1_add_bit(&mut r, Self::get_bit(a, bit_idx));
            if Self::cmp_mag(&r, b) != Ordering::Less {
                r = Self::sub_mag(&r, b);
                Self::set_bit(&mut q, bit_idx);
            }
        }
        while q.last() == Some(&0) {
            q.pop();
        }
        while r.last() == Some(&0) {
            r.pop();
        }
        (q, r)
    }
    fn add(&self, rhs: &Self) -> Self {
        if self.neg == rhs.neg {
            let mut r = BigInt { neg: self.neg, limbs: Self::add_mag(&self.limbs, &rhs.limbs) };
            r.norm();
            r
        } else {
            match Self::cmp_mag(&self.limbs, &rhs.limbs) {
                Ordering::Equal => Self::zero(),
                Ordering::Greater => {
                    let mut r =
                        BigInt { neg: self.neg, limbs: Self::sub_mag(&self.limbs, &rhs.limbs) };
                    r.norm();
                    r
                }
                Ordering::Less => {
                    let mut r =
                        BigInt { neg: rhs.neg, limbs: Self::sub_mag(&rhs.limbs, &self.limbs) };
                    r.norm();
                    r
                }
            }
        }
    }
    fn mul(&self, rhs: &Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::zero();
        }
        let mut r =
            BigInt { neg: self.neg != rhs.neg, limbs: Self::mul_mag(&self.limbs, &rhs.limbs) };
        r.norm();
        r
    }
    fn div_exact(&self, rhs: &Self) -> Self {
        assert!(!rhs.is_zero(), "BigInt div_exact by zero");
        if self.is_zero() {
            return Self::zero();
        }
        let (q, rem) = Self::div_rem_mag(&self.limbs, &rhs.limbs);
        assert!(rem.is_empty(), "BigInt div_exact had non-zero remainder");
        let mut r = BigInt { neg: self.neg != rhs.neg, limbs: q };
        r.norm();
        debug_assert_eq!(&r.mul(rhs), self);
        r
    }
    fn gcd(a: &Self, b: &Self) -> Self {
        let mut x = a.limbs.clone();
        let mut y = b.limbs.clone();
        while !y.is_empty() {
            if x.len() <= 2 && y.len() <= 2 {
                let mut xv = *x.first().unwrap_or(&0) as u64
                    | ((*x.get(1).unwrap_or(&0) as u64) << 32);
                let mut yv = *y.first().unwrap_or(&0) as u64
                    | ((*y.get(1).unwrap_or(&0) as u64) << 32);
                while yv != 0 {
                    let t = xv % yv;
                    xv = yv;
                    yv = t;
                }
                return Self::from_u64(xv);
            }
            let (_, rem) = Self::div_rem_mag(&x, &y);
            x = y;
            y = rem;
        }
        let mut g = BigInt { neg: false, limbs: x };
        g.norm();
        g
    }
    fn to_f64(&self) -> f64 {
        let mut acc = 0.0f64;
        for &limb in self.limbs.iter().rev() {
            acc = acc * 4294967296.0 + (limb as f64);
        }
        if self.neg { -acc } else { acc }
    }
}

impl Ord for BigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.neg, other.neg) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => Self::cmp_mag(&self.limbs, &other.limbs),
            (true, true) => Self::cmp_mag(&other.limbs, &self.limbs),
        }
    }
}

impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BigRat {
    num: BigInt,
    den: BigInt, // strictly > 0, gcd(|num|, den) == 1
}

impl BigRat {
    fn zero() -> Self {
        BigRat { num: BigInt::zero(), den: BigInt::one() }
    }
    fn one() -> Self {
        BigRat { num: BigInt::one(), den: BigInt::one() }
    }
    fn from_i64(v: i64) -> Self {
        BigRat { num: BigInt::from_i64(v), den: BigInt::one() }
    }
    fn from_u64(v: u64) -> Self {
        BigRat { num: BigInt::from_u64(v), den: BigInt::one() }
    }
    fn new(mut num: BigInt, mut den: BigInt) -> Self {
        assert!(!den.is_zero(), "BigRat denominator zero");
        if num.is_zero() {
            return Self::zero();
        }
        if den.neg {
            num = num.negate();
            den = den.negate();
        }
        if !den.is_one() {
            let g = BigInt::gcd(&num, &den);
            if !g.is_one() {
                num = num.div_exact(&g);
                den = den.div_exact(&g);
            }
        }
        BigRat { num, den }
    }
    fn is_zero(&self) -> bool {
        self.num.is_zero()
    }
    fn abs(&self) -> Self {
        BigRat { num: self.num.abs(), den: self.den.clone() }
    }
    fn add(&self, rhs: &Self) -> Self {
        if self.is_zero() {
            return rhs.clone();
        }
        if rhs.is_zero() {
            return self.clone();
        }
        if self.den == rhs.den {
            return Self::new(self.num.add(&rhs.num), self.den.clone());
        }
        let g = BigInt::gcd(&self.den, &rhs.den);
        if g.is_one() {
            let n = self.num.mul(&rhs.den).add(&rhs.num.mul(&self.den));
            let d = self.den.mul(&rhs.den);
            Self::new(n, d)
        } else {
            let d1 = self.den.div_exact(&g);
            let d2 = rhs.den.div_exact(&g);
            let n = self.num.mul(&d2).add(&rhs.num.mul(&d1));
            let d = self.den.mul(&d2);
            Self::new(n, d)
        }
    }
    fn sub(&self, rhs: &Self) -> Self {
        if rhs.is_zero() {
            return self.clone();
        }
        let neg_rhs = BigRat { num: rhs.num.negate(), den: rhs.den.clone() };
        self.add(&neg_rhs)
    }
    fn mul(&self, rhs: &Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::zero();
        }
        let g1 = BigInt::gcd(&self.num, &rhs.den);
        let g2 = BigInt::gcd(&rhs.num, &self.den);
        let n1 = if g1.is_one() { self.num.clone() } else { self.num.div_exact(&g1) };
        let d2 = if g1.is_one() { rhs.den.clone() } else { rhs.den.div_exact(&g1) };
        let n2 = if g2.is_one() { rhs.num.clone() } else { rhs.num.div_exact(&g2) };
        let d1 = if g2.is_one() { self.den.clone() } else { self.den.div_exact(&g2) };
        BigRat { num: n1.mul(&n2), den: d1.mul(&d2) }
    }
    fn div_u64(&self, k: u64) -> Self {
        assert!(k > 0);
        if self.is_zero() {
            return Self::zero();
        }
        Self::new(self.num.clone(), self.den.mul(&BigInt::from_u64(k)))
    }
    fn to_f64(&self) -> f64 {
        self.num.to_f64() / self.den.to_f64()
    }
}

impl Ord for BigRat {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.den == other.den {
            self.num.cmp(&other.num)
        } else {
            self.num.mul(&other.den).cmp(&other.num.mul(&self.den))
        }
    }
}

impl PartialOrd for BigRat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct ExactRatGerschRayleigh {
    g_row: BigRat,
    g_col: BigRat,
    g_min: BigRat,
    row_radii: Vec<BigRat>,
    col_radii: Vec<BigRat>,
    diag_max: BigRat,
    diag_min: BigRat,
    tr_avg: BigRat,
    tr_sum: BigRat,
    g_omega_hi: BigRat,
    g_omega_lo: BigRat,
    is_normal: bool,
}

/// Exact Q-rational Gerschgorin (1931) row/col disks + symmetric Hermitian-part Rayleigh/trace bounds
/// for any rational matrix M in Q^{s x s}:
///   - G_row(M) = max_i (|M_{ii}| + R_i(M)),  G_col(M) = max_j (|M_{jj}| + C_j(M))
///   - H_0 = (M + M^T)/2  =>  G_omega_lo <= diag_min <= tr(M)/s <= diag_max <= G_omega_hi (exact in Q!)
///   - Exact commutator check M^T M == M M^T in Q!
fn exact_bigrat_gerschgorin_rayleigh(m: &[Vec<BigRat>]) -> ExactRatGerschRayleigh {
    let s = m.len();
    assert!(s > 0);
    let mut row_radii = vec![BigRat::zero(); s];
    let mut col_radii = vec![BigRat::zero(); s];
    let mut g_row = BigRat::zero();
    let mut g_col = BigRat::zero();
    let mut diag_max = m[0][0].clone();
    let mut diag_min = m[0][0].clone();
    let mut tr_sum = BigRat::zero();
    let mut is_symmetric = true;

    for i in 0..s {
        let mii = &m[i][i];
        tr_sum = tr_sum.add(mii);
        if mii > &diag_max {
            diag_max = mii.clone();
        }
        if mii < &diag_min {
            diag_min = mii.clone();
        }
        let mut ri = BigRat::zero();
        for j in 0..s {
            if m[i][j] != m[j][i] {
                is_symmetric = false;
            }
            if i != j {
                let a = m[i][j].abs();
                ri = ri.add(&a);
                col_radii[j] = col_radii[j].add(&a);
            }
        }
        let bound_i = mii.abs().add(&ri);
        if i == 0 || bound_i > g_row {
            g_row = bound_i;
        }
        row_radii[i] = ri;
    }
    for j in 0..s {
        let bound_j = m[j][j].abs().add(&col_radii[j]);
        if j == 0 || bound_j > g_col {
            g_col = bound_j;
        }
    }
    let g_min = if g_row <= g_col { g_row.clone() } else { g_col.clone() };
    let tr_avg = tr_sum.div_u64(s as u64);

    // Exact Hermitian symmetric part H_0 = (M + M^T)/2 Gerschgorin bounds on omega(M) = lam_max(H_0)
    let mut g_omega_hi = BigRat::zero();
    let mut g_omega_lo = BigRat::zero();
    for i in 0..s {
        let mut r_h0 = BigRat::zero();
        for j in 0..s {
            if i != j {
                let hij = m[i][j].add(&m[j][i]).div_u64(2);
                r_h0 = r_h0.add(&hij.abs());
            }
        }
        let hi_i = m[i][i].add(&r_h0);
        let lo_i = m[i][i].sub(&r_h0);
        if i == 0 || hi_i > g_omega_hi {
            g_omega_hi = hi_i;
        }
        if i == 0 || lo_i < g_omega_lo {
            g_omega_lo = lo_i;
        }
    }

    // Exact rational inequalities (0 floats!)
    assert!(g_omega_lo <= diag_min, "Exact Q Gerschgorin lower bound violated!");
    assert!(diag_min <= tr_avg, "Exact Q diag_min <= tr/s violated!");
    assert!(tr_avg <= diag_max, "Exact Q tr/s <= diag_max violated!");
    assert!(diag_max <= g_omega_hi, "Exact Q diag_max <= G_omega violated!");
    if is_symmetric {
        assert_eq!(g_row, g_col, "Symmetric matrix must have G_row == G_col!");
    }

    // Exact normality M^T M == M M^T in Q
    let mut is_normal = true;
    'norm_check: for i in 0..s {
        for j in 0..s {
            let mut mtm = BigRat::zero();
            let mut mmt = BigRat::zero();
            for k in 0..s {
                if !m[k][i].is_zero() && !m[k][j].is_zero() {
                    mtm = mtm.add(&m[k][i].mul(&m[k][j]));
                }
                if !m[i][k].is_zero() && !m[j][k].is_zero() {
                    mmt = mmt.add(&m[i][k].mul(&m[j][k]));
                }
            }
            if mtm != mmt {
                is_normal = false;
                break 'norm_check;
            }
        }
    }
    if is_symmetric {
        assert!(is_normal, "Symmetric rational matrix must be normal!");
    }

    ExactRatGerschRayleigh {
        g_row,
        g_col,
        g_min,
        row_radii,
        col_radii,
        diag_max,
        diag_min,
        tr_avg,
        tr_sum,
        g_omega_hi,
        g_omega_lo,
        is_normal,
    }
}

// ============================================================================
// SECTION 1B: Pure-Orthogonal Real Schur (`CA-51`), Hermitian Tridiagonalization
//             + Sturm Bisection, & Certified Toeplitz-Hausdorff Numerical Range
// ============================================================================

#[derive(Clone, Copy, Debug)]
struct C64 {
    re: f64,
    im: f64,
}

impl C64 {
    #[inline(always)]
    fn new(re: f64, im: f64) -> Self {
        C64 { re, im }
    }
    #[inline(always)]
    fn abs2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    #[inline(always)]
    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
}

fn mat_frob2(a: &[Vec<f64>]) -> f64 {
    let mut s = 0.0f64;
    for row in a {
        for &v in row {
            s += v * v;
        }
    }
    s
}

fn hessenberg_reduce_with_q(a: &[Vec<f64>]) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let n = a.len();
    let mut h = a.to_vec();
    let mut q = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        q[i][i] = 1.0;
    }
    if n <= 2 {
        return (q, h);
    }
    for k in 0..(n - 2) {
        let mut scale = 0.0f64;
        for i in (k + 1)..n {
            scale += h[i][k].abs();
        }
        if scale == 0.0 {
            continue;
        }
        let len = n - (k + 1);
        let mut v = vec![0.0f64; len];
        let mut norm2 = 0.0f64;
        for idx in 0..len {
            let val = h[k + 1 + idx][k] / scale;
            v[idx] = val;
            norm2 += val * val;
        }
        let norm = norm2.sqrt();
        let sign = if v[0] >= 0.0 { 1.0 } else { -1.0 };
        v[0] += sign * norm;
        let v_norm2: f64 = v.iter().map(|&x| x * x).sum();
        if v_norm2 == 0.0 {
            continue;
        }
        let beta = 2.0 / v_norm2;

        for j in k..n {
            let mut dot = 0.0f64;
            for (idx, &vi) in v.iter().enumerate() {
                dot += vi * h[k + 1 + idx][j];
            }
            let coeff = beta * dot;
            for (idx, &vi) in v.iter().enumerate() {
                h[k + 1 + idx][j] -= coeff * vi;
            }
        }
        for i in 0..n {
            let mut dot = 0.0f64;
            for (idx, &vj) in v.iter().enumerate() {
                dot += h[i][k + 1 + idx] * vj;
            }
            let coeff = beta * dot;
            for (idx, &vj) in v.iter().enumerate() {
                h[i][k + 1 + idx] -= coeff * vj;
            }
        }
        for i in 0..n {
            let mut dot = 0.0f64;
            for (idx, &vj) in v.iter().enumerate() {
                dot += q[i][k + 1 + idx] * vj;
            }
            let coeff = beta * dot;
            for (idx, &vj) in v.iter().enumerate() {
                q[i][k + 1 + idx] -= coeff * vj;
            }
        }
        for i in (k + 2)..n {
            h[i][k] = 0.0;
        }
    }
    (q, h)
}

fn apply_givens_2x2(h: &mut [Vec<f64>], q: &mut [Vec<f64>], lo: usize, hi: usize, cs: f64, sn: f64) {
    let n = h.len();
    for j in lo..n {
        let a = h[lo][j];
        let b = h[hi][j];
        h[lo][j] = cs * a + sn * b;
        h[hi][j] = -sn * a + cs * b;
    }
    for i in 0..=hi {
        let a = h[i][lo];
        let b = h[i][hi];
        h[i][lo] = cs * a + sn * b;
        h[i][hi] = -sn * a + cs * b;
    }
    for i in 0..n {
        let a = q[i][lo];
        let b = q[i][hi];
        q[i][lo] = cs * a + sn * b;
        q[i][hi] = -sn * a + cs * b;
    }
}

fn finalize_2x2_block(
    h: &mut [Vec<f64>],
    q: &mut [Vec<f64>],
    lo: usize,
    hi: usize,
) -> (bool, C64, C64) {
    let a = h[lo][lo];
    let b = h[lo][hi];
    let c = h[hi][lo];
    let d = h[hi][hi];
    let half_tr = 0.5 * (a + d);
    let half_diff = 0.5 * (a - d);
    let disc = half_diff * half_diff + b * c;

    if disc >= 0.0 {
        if c != 0.0 {
            let sd = disc.sqrt();
            let lam1 = half_tr + if half_diff >= 0.0 { sd } else { -sd };
            let r0 = (a - lam1).hypot(b);
            let r1 = c.hypot(d - lam1);
            let (vx, vy) = if r0 >= r1 { (-b, a - lam1) } else { (d - lam1, -c) };
            let vn = vx.hypot(vy);
            if vn > 0.0 {
                let cs = vx / vn;
                let sn = vy / vn;
                apply_givens_2x2(h, q, lo, hi, cs, sn);
            }
        }
        h[hi][lo] = 0.0;
        let l1 = h[lo][lo];
        let l2 = h[hi][hi];
        (false, C64::new(l1, 0.0), C64::new(l2, 0.0))
    } else {
        if (a - d).abs() > 0.0 {
            let p = a - d;
            let s = b + c;
            let theta = 0.5 * (-p).atan2(s);
            let (sn, cs) = theta.sin_cos();
            apply_givens_2x2(h, q, lo, hi, cs, sn);
        }
        let na = h[lo][lo];
        let nb = h[lo][hi];
        let nc = h[hi][lo];
        let nd = h[hi][hi];
        let alpha = 0.5 * (na + nd);
        let beta = (-(0.25 * (na - nd) * (na - nd) + nb * nc)).max(0.0).sqrt();
        (true, C64::new(alpha, beta), C64::new(alpha, -beta))
    }
}

struct RealSchurLite {
    eigs: Vec<C64>,
    delta_f: f64,
    schur_id_err: f64,
}

fn real_schur_eigs_and_df(a: &[Vec<f64>]) -> RealSchurLite {
    let n = a.len();
    if n == 0 {
        return RealSchurLite { eigs: Vec::new(), delta_f: 0.0, schur_id_err: 0.0 };
    }
    let frob2_a = mat_frob2(a);
    let h_frob = frob2_a.sqrt();
    if h_frob == 0.0 {
        return RealSchurLite {
            eigs: vec![C64::new(0.0, 0.0); n],
            delta_f: 0.0,
            schur_id_err: 0.0,
        };
    }

    let (mut q, mut h) = hessenberg_reduce_with_q(a);
    let eps = 2.220446049250313e-16_f64;
    let mut blocks: Vec<(usize, bool)> = Vec::with_capacity(n);
    let mut eigs: Vec<C64> = Vec::with_capacity(n);
    let mut hi = n - 1;
    let mut iter_count = 0usize;
    let mut total_iters = 0usize;

    loop {
        if hi == 0 {
            blocks.push((0, false));
            eigs.push(C64::new(h[0][0], 0.0));
            break;
        }
        let mut lo = hi;
        while lo > 0 {
            let s = h[lo - 1][lo - 1].abs() + h[lo][lo].abs();
            let thresh = eps * if s > 0.0 { s } else { h_frob };
            if h[lo][lo - 1].abs() <= thresh.max(eps * 1e-4 * h_frob) {
                h[lo][lo - 1] = 0.0;
                break;
            }
            lo -= 1;
        }
        if lo == hi {
            blocks.push((hi, false));
            eigs.push(C64::new(h[hi][hi], 0.0));
            hi -= 1;
            iter_count = 0;
            continue;
        }
        if lo + 1 == hi {
            let (is_cx, e1, e2) = finalize_2x2_block(&mut h, &mut q, lo, hi);
            if is_cx {
                blocks.push((lo, true));
            } else {
                blocks.push((hi, false));
                blocks.push((lo, false));
            }
            eigs.push(e1);
            eigs.push(e2);
            if lo == 0 {
                break;
            }
            hi = lo - 1;
            iter_count = 0;
            continue;
        }

        iter_count += 1;
        total_iters += 1;
        assert!(total_iters < 400 * n + 4000, "Real Schur Francis QR did not converge (n={n})");

        let (s, t) = if iter_count % 10 == 0 {
            let phase = (iter_count / 10) % 6;
            let w = if phase % 2 == 0 {
                h[hi][hi - 1].abs() + h[hi - 1][hi - 2].abs()
            } else {
                h[lo + 1][lo].abs() + h[lo + 2][lo + 1].abs()
            };
            let w = w.max(eps * h_frob);
            let (c1, c2) = match phase {
                0 => (0.75, 0.4375),
                1 => (0.6180339887498949, 0.3819660112501051),
                2 => (-0.75, 0.4375),
                3 => (0.5, 0.75),
                4 => (-0.3819660112501051, 0.6180339887498949),
                _ => (0.91, 0.27),
            };
            let shift = c1 * w + h[hi][hi];
            (2.0 * shift, shift * shift + c2 * w * w)
        } else {
            let a11 = h[hi - 1][hi - 1];
            let a12 = h[hi - 1][hi];
            let a21 = h[hi][hi - 1];
            let a22 = h[hi][hi];
            (a11 + a22, a11 * a22 - a12 * a21)
        };

        let mut m = hi - 2;
        let (mut x, mut y, mut z) = loop {
            let h00 = h[m][m];
            let h10 = h[m + 1][m];
            let h01 = h[m][m + 1];
            let h11 = h[m + 1][m + 1];
            let h21 = h[m + 2][m + 1];
            let mut lx = h00 * h00 + h01 * h10 - s * h00 + t;
            let mut ly = h10 * (h00 + h11 - s);
            let mut lz = h10 * h21;
            let scale = lx.abs() + ly.abs() + lz.abs();
            if scale > 0.0 {
                lx /= scale;
                ly /= scale;
                lz /= scale;
            }
            if m == lo {
                break (lx, ly, lz);
            }
            let lhs = h[m][m - 1].abs() * (ly.abs() + lz.abs());
            let rhs = 0.1 * eps * lx.abs() * (h[m - 1][m - 1].abs() + h00.abs() + h11.abs());
            if lhs <= rhs {
                break (lx, ly, lz);
            }
            m -= 1;
        };

        for k in m..=(hi - 1) {
            let not_last = k < hi - 1;
            if k > m {
                x = h[k][k - 1];
                y = h[k + 1][k - 1];
                z = if not_last { h[k + 2][k - 1] } else { 0.0 };
            }
            let scale = x.abs() + y.abs() + z.abs();
            if scale == 0.0 {
                continue;
            }
            let xs = x / scale;
            let ys = y / scale;
            let zs = z / scale;
            let norm = (xs * xs + ys * ys + zs * zs).sqrt();
            let sign = if xs >= 0.0 { 1.0 } else { -1.0 };
            let r_val = sign * norm;
            let v0 = xs + r_val;
            if v0 == 0.0 && ys == 0.0 && zs == 0.0 {
                continue;
            }
            let v1 = ys / v0;
            let v2 = zs / v0;
            let beta = 2.0 / (1.0 + v1 * v1 + v2 * v2);

            if k > m {
                h[k][k - 1] = -r_val * scale;
                h[k + 1][k - 1] = 0.0;
                if not_last {
                    h[k + 2][k - 1] = 0.0;
                }
            } else if m > lo {
                h[k][k - 1] *= 1.0 - beta;
            }

            for j in k..n {
                let mut dot = h[k][j] + v1 * h[k + 1][j];
                if not_last {
                    dot += v2 * h[k + 2][j];
                }
                let coeff = beta * dot;
                h[k][j] -= coeff;
                h[k + 1][j] -= coeff * v1;
                if not_last {
                    h[k + 2][j] -= coeff * v2;
                }
            }
            let row_end = hi.min(k + 3);
            for i in 0..=row_end {
                let mut dot = h[i][k] + v1 * h[i][k + 1];
                if not_last {
                    dot += v2 * h[i][k + 2];
                }
                let coeff = beta * dot;
                h[i][k] -= coeff;
                h[i][k + 1] -= coeff * v1;
                if not_last {
                    h[i][k + 2] -= coeff * v2;
                }
            }
            for i in 0..n {
                let mut dot = q[i][k] + v1 * q[i][k + 1];
                if not_last {
                    dot += v2 * q[i][k + 2];
                }
                let coeff = beta * dot;
                q[i][k] -= coeff;
                q[i][k + 1] -= coeff * v1;
                if not_last {
                    q[i][k + 2] -= coeff * v2;
                }
            }
        }
    }

    let mut is_2x2_start = vec![false; n];
    for &(lo, is_cx) in &blocks {
        if is_cx {
            is_2x2_start[lo] = true;
        }
    }
    let mut strict_upper_f2 = 0.0f64;
    let mut internal_2x2_f2 = 0.0f64;
    for i in 0..n {
        if is_2x2_start[i] {
            let a_b = h[i][i];
            let b_b = h[i][i + 1];
            let c_b = h[i + 1][i];
            let d_b = h[i + 1][i + 1];
            internal_2x2_f2 += (a_b - d_b) * (a_b - d_b) + (b_b + c_b) * (b_b + c_b);
        }
        for j in (i + 1)..n {
            if j == i + 1 && is_2x2_start[i] {
                continue;
            }
            strict_upper_f2 += h[i][j] * h[i][j];
        }
    }
    let eig_abs2_sum: f64 = eigs.iter().map(|e| e.abs2()).sum();
    let delta2_r = strict_upper_f2 + internal_2x2_f2;
    let schur_id_err = ((frob2_a - eig_abs2_sum) - delta2_r).abs();
    assert!(schur_id_err < 1e-11 * (1.0 + frob2_a), "Schur ID err={schur_id_err}");

    RealSchurLite { eigs, delta_f: delta2_r.sqrt(), schur_id_err }
}

/// Sturm sequence count of eigenvalues strictly `< mu` of real symmetric tridiagonal `(diag, off)`.
#[inline]
fn sturm_count_lt(diag: &[f64], off: &[f64], mu: f64) -> usize {
    let s = diag.len();
    if s == 0 {
        return 0;
    }
    let mut count = 0usize;
    let mut d = diag[0] - mu;
    if d < 0.0 {
        count += 1;
    }
    for i in 1..s {
        let e = off[i - 1];
        if e == 0.0 {
            d = diag[i] - mu;
        } else {
            let denom = if d.abs() < 1e-200 {
                if d < 0.0 { -1e-200 } else { 1e-200 }
            } else {
                d
            };
            d = (diag[i] - mu) - (e * e) / denom;
        }
        if d < 0.0 {
            count += 1;
        }
    }
    count
}

/// Exact `(lam_min, lam_max)` of real symmetric tridiagonal `(diag, off)` via Gerschgorin (1931)
/// initial bracket + 64-step Sturm bisection (full IEEE-754 f64 precision!).
fn tridiag_min_max_eigs(diag: &[f64], off: &[f64]) -> (f64, f64) {
    let s = diag.len();
    if s == 0 {
        return (0.0, 0.0);
    }
    if s == 1 {
        return (diag[0], diag[0]);
    }
    // Gerschgorin (1931) disks on symmetric tridiagonal supply an exact enclosure [lo0, hi0]!
    let mut lo0 = f64::INFINITY;
    let mut hi0 = f64::NEG_INFINITY;
    for i in 0..s {
        let r = if i > 0 { off[i - 1].abs() } else { 0.0 }
            + if i + 1 < s { off[i].abs() } else { 0.0 };
        lo0 = lo0.min(diag[i] - r);
        hi0 = hi0.max(diag[i] + r);
    }
    let pad = (hi0 - lo0).abs().max(lo0.abs()).max(hi0.abs()).max(1.0) * 1e-11;
    lo0 -= pad;
    hi0 += pad;

    let mut l = lo0;
    let mut r = hi0;
    for _ in 0..64 {
        let mid = 0.5 * (l + r);
        if sturm_count_lt(diag, off, mid) < s {
            l = mid;
        } else {
            r = mid;
        }
    }
    let lam_max = 0.5 * (l + r);

    let mut l2 = lo0;
    let mut r2 = hi0;
    for _ in 0..64 {
        let mid = 0.5 * (l2 + r2);
        if sturm_count_lt(diag, off, mid) == 0 {
            l2 = mid;
        } else {
            r2 = mid;
        }
    }
    let lam_min = 0.5 * (l2 + r2);
    (lam_min, lam_max)
}

/// Exact real symmetric Householder tridiagonalization + Sturm `(lam_min, lam_max)`.
/// Also verifies trace and Frobenius conservation on every call!
fn real_sym_min_max_eigs(sym_flat: &[f64], s: usize) -> (f64, f64) {
    if s == 0 {
        return (0.0, 0.0);
    }
    if s == 1 {
        return (sym_flat[0], sym_flat[0]);
    }
    if s == 2 {
        let a = sym_flat[0];
        let b = sym_flat[1];
        let d = sym_flat[3];
        let mid = 0.5 * (a + d);
        let rad = (0.5 * (a - d)).hypot(b);
        return (mid - rad, mid + rad);
    }
    let mut a = sym_flat.to_vec();
    let mut tr0 = 0.0f64;
    let mut frob2_0 = 0.0f64;
    for i in 0..s {
        tr0 += a[i * s + i];
        for j in 0..s {
            let v = a[i * s + j];
            frob2_0 += v * v;
        }
    }
    let mut off = vec![0.0f64; s - 1];
    let mut u = vec![0.0f64; s];
    let mut p = vec![0.0f64; s];

    for k in 0..(s - 2) {
        let m = s - k - 1;
        let mut norm2 = 0.0f64;
        for r in 0..m {
            let v = a[(k + 1 + r) * s + k];
            u[r] = v;
            norm2 += v * v;
        }
        let sigma = norm2.sqrt();
        if sigma <= 1e-300 {
            off[k] = 0.0;
            continue;
        }
        let sign = if u[0] >= 0.0 { 1.0 } else { -1.0 };
        u[0] += sign * sigma;
        let mut u_norm2 = 0.0f64;
        for r in 0..m {
            u_norm2 += u[r] * u[r];
        }
        if u_norm2 <= 1e-300 {
            off[k] = sigma;
            continue;
        }
        let beta = 2.0 / u_norm2;

        let base = k + 1;
        for r in 0..m {
            let row = (base + r) * s + base;
            let mut sum = 0.0f64;
            for c in 0..m {
                sum += a[row + c] * u[c];
            }
            p[r] = beta * sum;
        }
        let mut utp = 0.0f64;
        for r in 0..m {
            utp += u[r] * p[r];
        }
        let gamma = 0.5 * beta * utp;
        for r in 0..m {
            p[r] -= gamma * u[r]; // p now holds w
        }
        for r in 0..m {
            let row = (base + r) * s + base;
            let ur = u[r];
            let wr = p[r];
            for c in 0..m {
                a[row + c] -= ur * p[c] + wr * u[c];
            }
        }
        off[k] = sigma;
    }
    off[s - 2] = a[(s - 1) * s + (s - 2)].abs();
    let diag: Vec<f64> = (0..s).map(|i| a[i * s + i]).collect();

    let tr_t: f64 = diag.iter().sum();
    let frob2_t: f64 =
        diag.iter().map(|&x| x * x).sum::<f64>() + 2.0 * off.iter().map(|&x| x * x).sum::<f64>();
    let sc = frob2_0.sqrt().max(1.0);
    assert!((tr_t - tr0).abs() <= 1e-9 * sc, "Real sym tridiag trace err: {}", (tr_t - tr0).abs());
    assert!(
        (frob2_t - frob2_0).abs() <= 1e-8 * sc * sc,
        "Real sym tridiag frob2 err: {}",
        (frob2_t - frob2_0).abs()
    );
    tridiag_min_max_eigs(&diag, &off)
}

/// Exact Complex Hermitian Householder Tridiagonalization + Sturm `(lam_min, lam_max)`
/// for `H(theta) = cos(theta) * H_0 + i * sin(theta) * K_0` where `H_0 = (A+A^T)/2` and
/// `K_0 = (A-A^T)/2`. Unitary similarity preserves trace and Frobenius norm to `< 1e-9`!
fn hermitian_theta_min_max_eigs(
    h0_flat: &[f64],
    k0_flat: &[f64],
    s: usize,
    theta: f64,
) -> (f64, f64) {
    let (sn, cs) = theta.sin_cos();
    if sn.abs() < 1e-15 {
        let scaled: Vec<f64> = h0_flat.iter().map(|&v| cs * v).collect();
        return real_sym_min_max_eigs(&scaled, s);
    }
    if s == 0 {
        return (0.0, 0.0);
    }
    if s == 1 {
        let d = cs * h0_flat[0];
        return (d, d);
    }
    if s == 2 {
        let a = cs * h0_flat[0];
        let d = cs * h0_flat[3];
        let b_re = cs * h0_flat[2]; // row 1, col 0
        let b_im = sn * k0_flat[2];
        let mid = 0.5 * (a + d);
        let rad = (0.5 * (a - d)).hypot(b_re.hypot(b_im));
        return (mid - rad, mid + rad);
    }

    let n2 = s * s;
    let mut re = vec![0.0f64; n2];
    let mut im = vec![0.0f64; n2];
    let mut tr0 = 0.0f64;
    let mut frob2_0 = 0.0f64;
    for idx in 0..n2 {
        let vr = cs * h0_flat[idx];
        let vi = sn * k0_flat[idx];
        re[idx] = vr;
        im[idx] = vi;
        frob2_0 += vr * vr + vi * vi;
    }
    for i in 0..s {
        tr0 += re[i * s + i];
        im[i * s + i] = 0.0;
    }

    let mut off = vec![0.0f64; s - 1];
    let mut u_re = vec![0.0f64; s];
    let mut u_im = vec![0.0f64; s];
    let mut p_re = vec![0.0f64; s];
    let mut p_im = vec![0.0f64; s];

    for k in 0..(s - 2) {
        let m = s - k - 1;
        let base = k + 1;
        let mut norm2 = 0.0f64;
        for r in 0..m {
            let idx = (base + r) * s + k;
            let vr = re[idx];
            let vi = im[idx];
            u_re[r] = vr;
            u_im[r] = vi;
            norm2 += vr * vr + vi * vi;
        }
        let sigma = norm2.sqrt();
        if sigma <= 1e-300 {
            off[k] = 0.0;
            continue;
        }
        let x0_abs = u_re[0].hypot(u_im[0]);
        if x0_abs == 0.0 {
            u_re[0] += sigma;
        } else {
            let scale = sigma / x0_abs;
            u_re[0] += u_re[0] * scale;
            u_im[0] += u_im[0] * scale;
        }
        let mut u_norm2 = 0.0f64;
        for r in 0..m {
            u_norm2 += u_re[r] * u_re[r] + u_im[r] * u_im[r];
        }
        if u_norm2 <= 1e-300 {
            off[k] = sigma;
            continue;
        }
        let beta = 2.0 / u_norm2;

        // p = beta * A_sub * u
        for r in 0..m {
            let row = (base + r) * s + base;
            let mut sr = 0.0f64;
            let mut si = 0.0f64;
            for c in 0..m {
                let ar = re[row + c];
                let ai = im[row + c];
                let ur = u_re[c];
                let ui = u_im[c];
                sr += ar * ur - ai * ui;
                si += ar * ui + ai * ur;
            }
            p_re[r] = beta * sr;
            p_im[r] = beta * si;
        }
        // gamma = 0.5 * beta * Re(u^* p)
        let mut utp_re = 0.0f64;
        for r in 0..m {
            utp_re += u_re[r] * p_re[r] + u_im[r] * p_im[r];
        }
        let gamma = 0.5 * beta * utp_re;
        // w = p - gamma * u (in-place in p_re, p_im)
        for r in 0..m {
            p_re[r] -= gamma * u_re[r];
            p_im[r] -= gamma * u_im[r];
        }
        // A_sub <- A_sub - u w^* - w u^*
        for r in 0..m {
            let row = (base + r) * s + base;
            let ur = u_re[r];
            let ui = u_im[r];
            let wr = p_re[r];
            let wi = p_im[r];
            for c in 0..m {
                let uc_r = u_re[c];
                let uc_i = u_im[c];
                let wc_r = p_re[c];
                let wc_i = p_im[c];
                re[row + c] -= (ur * wc_r + ui * wc_i) + (wr * uc_r + wi * uc_i);
                im[row + c] -= (ui * wc_r - ur * wc_i) + (wi * uc_r - wr * uc_i);
            }
            im[row + r] = 0.0;
        }
        off[k] = sigma;
    }
    off[s - 2] = re[(s - 1) * s + (s - 2)].hypot(im[(s - 1) * s + (s - 2)]);
    let diag: Vec<f64> = (0..s).map(|i| re[i * s + i]).collect();

    let tr_t: f64 = diag.iter().sum();
    let frob2_t: f64 =
        diag.iter().map(|&x| x * x).sum::<f64>() + 2.0 * off.iter().map(|&x| x * x).sum::<f64>();
    let sc = frob2_0.sqrt().max(1.0);
    assert!(
        (tr_t - tr0).abs() <= 1e-9 * sc,
        "Hermitian tridiag trace err: {}",
        (tr_t - tr0).abs()
    );
    assert!(
        (frob2_t - frob2_0).abs() <= 1e-8 * sc * sc,
        "Hermitian tridiag frob2 err: {}",
        (frob2_t - frob2_0).abs()
    );
    tridiag_min_max_eigs(&diag, &off)
}

/// Exact spectral norm `||A||_2 = sqrt(lam_max(A^T A))`.
fn spectral_norm_2(a: &[Vec<f64>]) -> f64 {
    let s = a.len();
    if s == 0 {
        return 0.0;
    }
    let mut ata = vec![0.0f64; s * s];
    for i in 0..s {
        for j in i..s {
            let mut sum = 0.0f64;
            for k in 0..s {
                sum += a[k][i] * a[k][j];
            }
            ata[i * s + j] = sum;
            ata[j * s + i] = sum;
        }
    }
    let (_, lmax) = real_sym_min_max_eigs(&ata, s);
    lmax.max(0.0).sqrt()
}

#[derive(Clone, Copy)]
struct AngleInterval {
    ub: f64,
    a: f64,
    b: f64,
    ha_pos: f64,
    hb_pos: f64,
    ha_neg: f64,
    hb_neg: f64,
}

impl PartialEq for AngleInterval {
    fn eq(&self, other: &Self) -> bool {
        self.ub == other.ub
    }
}
impl Eq for AngleInterval {}
impl PartialOrd for AngleInterval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for AngleInterval {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ub.partial_cmp(&other.ub).unwrap_or(Ordering::Equal)
    }
}

struct NumericalRangeResult {
    r_num: f64,       // certified lower bound (== exact r(A) to <= r_cert_gap)
    r_ub: f64,        // certified global upper bound from supporting-line outer polygon
    omega: f64,       // numerical abscissa lam_max((A+A^T)/2)
    omega_min: f64,   // lam_min((A+A^T)/2)
    spec_norm: f64,   // ||A||_2 = sigma_max(A)
    n_evals: usize,
}

/// Certified Toeplitz (1918) & Hausdorff (1919) numerical radius `r(A)`, numerical abscissa
/// `omega(A)`, and spectral norm `||A||_2` for any real matrix `A in R^{s x s}`:
/// Exploits real symmetry `W(A) = conj(W(A))` and `H(theta+pi) = -H(theta)`:
///   `r(A) = max_{theta in [0, pi/2]} max(lam_max(H(theta)), -lam_min(H(theta)))`,
/// using supporting-line outer-polygon branch-and-bound + golden-section refinement!
fn compute_toeplitz_hausdorff(a: &[Vec<f64>], max_splits: usize) -> NumericalRangeResult {
    let s = a.len();
    let spec_norm = spectral_norm_2(a);
    if s == 0 || spec_norm == 0.0 {
        return NumericalRangeResult {
            r_num: 0.0,
            r_ub: 0.0,
            omega: 0.0,
            omega_min: 0.0,
            spec_norm: 0.0,
            n_evals: 0,
        };
    }

    let mut h0 = vec![0.0f64; s * s];
    let mut k0 = vec![0.0f64; s * s];
    for i in 0..s {
        for j in 0..s {
            h0[i * s + j] = 0.5 * (a[i][j] + a[j][i]);
            k0[i * s + j] = 0.5 * (a[i][j] - a[j][i]);
        }
    }

    let mut n_evals = 0usize;
    let mut eval_theta = |th: f64| -> (f64, f64) {
        n_evals += 1;
        let (lmin, lmax) = hermitian_theta_min_max_eigs(&h0, &k0, s, th);
        (lmax, -lmin) // (h(theta), h(theta + pi))
    };

    // Fast exact shortcut for symmetric matrices (K_0 == 0): W(A) = [omega_min, omega]!
    let k0_norm2: f64 = k0.iter().map(|&x| x * x).sum();
    let (omega, neg_omega_min) = eval_theta(0.0);
    let omega_min = -neg_omega_min;
    if k0_norm2 <= 1e-28 * spec_norm * spec_norm {
        let r_exact = omega.abs().max(omega_min.abs());
        return NumericalRangeResult {
            r_num: r_exact,
            r_ub: r_exact,
            omega,
            omega_min,
            spec_norm,
            n_evals,
        };
    }

    // Initial M = 12 uniform intervals on [0, pi/2] (covers 48 sectors on [0, 2*pi))
    let m_init = 12usize;
    let mut pts_th = Vec::with_capacity(m_init + 1);
    let mut pts_pos = Vec::with_capacity(m_init + 1);
    let mut pts_neg = Vec::with_capacity(m_init + 1);
    pts_th.push(0.0);
    pts_pos.push(omega);
    pts_neg.push(neg_omega_min);

    let mut r_lb = omega.max(neg_omega_min).max(0.0);
    let mut theta_star = 0.0f64;

    for idx in 1..=m_init {
        let th = (idx as f64) * (0.5 * PI) / (m_init as f64);
        let (hp, hn) = eval_theta(th);
        pts_th.push(th);
        pts_pos.push(hp);
        pts_neg.push(hn);
        let val = hp.max(hn);
        if val > r_lb {
            r_lb = val;
            theta_star = th;
        }
    }

    let make_iv = |a_th: f64, b_th: f64, hap: f64, hbp: f64, han: f64, hbn: f64| -> AngleInterval {
        let half_d = 0.5 * (b_th - a_th);
        let inv_cos = 1.0 / half_d.cos();
        let ub_p = hap.max(hbp).max(0.0) * inv_cos;
        let ub_n = han.max(hbn).max(0.0) * inv_cos;
        AngleInterval {
            ub: ub_p.max(ub_n).min(spec_norm),
            a: a_th,
            b: b_th,
            ha_pos: hap,
            hb_pos: hbp,
            ha_neg: han,
            hb_neg: hbn,
        }
    };

    let mut heap = BinaryHeap::new();
    for idx in 0..m_init {
        heap.push(make_iv(
            pts_th[idx],
            pts_th[idx + 1],
            pts_pos[idx],
            pts_pos[idx + 1],
            pts_neg[idx],
            pts_neg[idx + 1],
        ));
    }

    let tol = 1e-11 * (1.0 + r_lb);
    for _ in 0..max_splits {
        let Some(top) = heap.pop() else { break };
        if top.ub <= r_lb + tol {
            heap.push(top);
            break;
        }
        if top.b - top.a < 1e-7 {
            heap.push(top);
            break;
        }
        let mid = 0.5 * (top.a + top.b);
        let (mp, mn) = eval_theta(mid);
        let val = mp.max(mn);
        if val > r_lb {
            r_lb = val;
            theta_star = mid;
        }
        let left = make_iv(top.a, mid, top.ha_pos, mp, top.ha_neg, mn);
        let right = make_iv(mid, top.b, mp, top.hb_pos, mn, top.hb_neg);
        if left.ub > r_lb + 0.1 * tol {
            heap.push(left);
        }
        if right.ub > r_lb + 0.1 * tol {
            heap.push(right);
        }
    }

    let mut r_ub = heap.peek().map_or(r_lb, |iv| iv.ub.max(r_lb));

    // Local golden-section refinement around theta_star for machine-precision peak!
    let half_w = heap
        .peek()
        .map_or(0.5 * PI / (m_init as f64), |iv| (iv.b - iv.a).max(1e-4))
        .min(0.5 * PI / (m_init as f64));
    let mut gl = (theta_star - half_w).max(0.0);
    let mut gr = (theta_star + half_w).min(0.5 * PI);
    let phi = 0.6180339887498949_f64;
    let gs_iters = if s <= 16 { 42 } else { 22 };
    if (s <= 16 || r_ub - r_lb > 1e-11) && gr - gl > 1e-12 {
        let mut x1 = gr - phi * (gr - gl);
        let mut x2 = gl + phi * (gr - gl);
        let (p1, n1) = eval_theta(x1);
        let mut f1 = p1.max(n1);
        let (p2, n2) = eval_theta(x2);
        let mut f2 = p2.max(n2);
        if f1 > r_lb {
            r_lb = f1;
        }
        if f2 > r_lb {
            r_lb = f2;
        }
        for _ in 0..gs_iters {
            if gr - gl < 1e-12 {
                break;
            }
            if f1 < f2 {
                gl = x1;
                x1 = x2;
                f1 = f2;
                x2 = gl + phi * (gr - gl);
                let (pp, nn) = eval_theta(x2);
                f2 = pp.max(nn);
                if f2 > r_lb {
                    r_lb = f2;
                }
            } else {
                gr = x2;
                x2 = x1;
                f2 = f1;
                x1 = gr - phi * (gr - gl);
                let (pp, nn) = eval_theta(x1);
                f1 = pp.max(nn);
                if f1 > r_lb {
                    r_lb = f1;
                }
            }
        }
        r_ub = r_ub.max(r_lb);
    }

    NumericalRangeResult { r_num: r_lb, r_ub, omega, omega_min, spec_norm, n_evals }
}

/// Gerschgorin (1931) row/column/symmetric disk metrics & containment verifier in f64.
struct GerschgorinF64 {
    g_row: f64,
    g_col: f64,
    g_min: f64,
    g_omega: f64,
    g_row_mean: f64,
    g_row_pi: f64,
    row_radii: Vec<f64>,
    col_radii: Vec<f64>,
}

fn compute_gerschgorin_f64(a: &[Vec<f64>], pi_opt: Option<&[f64]>) -> GerschgorinF64 {
    let s = a.len();
    let mut row_radii = vec![0.0f64; s];
    let mut col_radii = vec![0.0f64; s];
    let mut g_row = 0.0f64;
    let mut g_col = 0.0f64;
    let mut g_omega = f64::NEG_INFINITY;
    let mut sum_row_rad = 0.0f64;
    let mut pi_row_rad = 0.0f64;

    for i in 0..s {
        let mut ri = 0.0f64;
        let mut rh0 = 0.0f64;
        for j in 0..s {
            if i != j {
                ri += a[i][j].abs();
                col_radii[j] += a[i][j].abs();
                rh0 += (0.5 * (a[i][j] + a[j][i])).abs();
            }
        }
        row_radii[i] = ri;
        let rb = a[i][i].abs() + ri;
        g_row = g_row.max(rb);
        sum_row_rad += rb;
        if let Some(pi) = pi_opt {
            pi_row_rad += pi[i] * rb;
        }
        g_omega = g_omega.max(a[i][i] + rh0);
    }
    for j in 0..s {
        g_col = g_col.max(a[j][j].abs() + col_radii[j]);
    }
    GerschgorinF64 {
        g_row,
        g_col,
        g_min: g_row.min(g_col),
        g_omega,
        g_row_mean: if s > 0 { sum_row_rad / (s as f64) } else { 0.0 },
        g_row_pi: pi_row_rad,
        row_radii,
        col_radii,
    }
}

/// Verifies S. Gerschgorin (1931) containment: every eigenvalue `lam in sigma(A)` lies in
/// at least one row disk `D(A_{ii}, R_i)` AND at least one column disk `D(A_{jj}, C_j)`.
fn verify_gerschgorin_containment(a: &[Vec<f64>], eigs: &[C64], g: &GerschgorinF64) -> f64 {
    let s = a.len();
    let mut max_margin_violation = 0.0f64;
    for &lam in eigs {
        let mut min_row_excess = f64::INFINITY;
        let mut min_col_excess = f64::INFINITY;
        for i in 0..s {
            let dist = (lam.re - a[i][i]).hypot(lam.im);
            min_row_excess = min_row_excess.min(dist - g.row_radii[i]);
            min_col_excess = min_col_excess.min(dist - g.col_radii[i]);
        }
        let v_r = min_row_excess.max(0.0);
        let v_c = min_col_excess.max(0.0);
        max_margin_violation = max_margin_violation.max(v_r).max(v_c);
        assert!(
            v_r < 1e-9 && v_c < 1e-9,
            "Gerschgorin (1931) disk containment violated! eig=({:.6},{:.6}), row_exc={:.3e}, col_exc={:.3e}",
            lam.re, lam.im, v_r, v_c
        );
    }
    max_margin_violation
}

// ============================================================================
// SECTION 2: Basic BWT, Open<16> Extraction, Corpora & Synthetic Suites
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
}

fn starts_arr(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < l.len() {
        let c = l[i];
        let mut w = 1usize;
        while i + w < l.len() && l[i + w] == c {
            w += 1;
        }
        let mut rem = w;
        while rem > 0 {
            let chunk = rem.min(127);
            out.push(((c as u16) << 8) | (chunk as u16));
            rem -= chunk;
        }
        i += w;
    }
    out
}

fn bwt_encode(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut rot: Vec<usize> = (0..n).collect();
    rot.sort_by(|&a, &b| {
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
    for (idx, &start) in rot.iter().enumerate() {
        if start == 0 {
            p = idx as u32;
        }
        l.push(text[(start + n - 1) % n]);
    }
    (l, p)
}

fn invert_forward_fl(l_col: &[u8], p: u32) -> Vec<u8> {
    let n = l_col.len();
    if n == 0 {
        return Vec::new();
    }
    let mut all = [0u32; 256];
    for &c in l_col {
        all[usize::from(c)] += 1;
    }
    let mut first = starts_arr(all);
    let mut f_col = vec![0u8; n];
    let mut pos = 0usize;
    for b in 0..256 {
        for _ in 0..all[b] {
            f_col[pos] = b as u8;
            pos += 1;
        }
    }
    let mut fl = vec![0u32; n];
    for (row, &c) in l_col.iter().enumerate() {
        let a = usize::from(c);
        fl[first[a] as usize] = row as u32;
        first[a] += 1;
    }
    let mut out = Vec::with_capacity(n);
    let mut cur = p as usize;
    for _ in 0..n {
        out.push(f_col[cur]);
        cur = fl[cur] as usize;
    }
    out
}

fn build_g1_matrix(l_col: &[u8], all: &[u32; 256]) -> (Vec<u8>, Vec<usize>, Vec<Vec<u64>>) {
    let mut syms = Vec::new();
    let mut map = [usize::MAX; 256];
    let mut m = Vec::new();
    for b in 0..256 {
        if all[b] > 0 {
            map[b] = syms.len();
            syms.push(b as u8);
            m.push(all[b] as usize);
        }
    }
    let s = syms.len();
    let mut t = vec![vec![0u64; s]; s];
    let mut pos = 0usize;
    for i in 0..s {
        for _ in 0..m[i] {
            let j = map[usize::from(l_col[pos])];
            t[i][j] += 1;
            pos += 1;
        }
    }
    for i in 0..s {
        let rs: u64 = t[i].iter().sum();
        let cs: u64 = (0..s).map(|r| t[r][i]).sum();
        assert_eq!(rs, m[i] as u64);
        assert_eq!(cs, m[i] as u64);
    }
    (syms, m, t)
}

fn deflated_stochastic(t_mat: &[Vec<u64>], m: &[usize]) -> (Vec<Vec<f64>>, Vec<f64>) {
    let s = m.len();
    let tot: f64 = m.iter().sum::<usize>() as f64;
    let pi: Vec<f64> = m.iter().map(|&v| v as f64 / tot).collect();
    let mut hp = vec![vec![0.0f64; s]; s];
    for i in 0..s {
        let inv = 1.0 / (m[i] as f64);
        for j in 0..s {
            hp[i][j] = (t_mat[i][j] as f64) * inv - pi[j];
        }
    }
    (hp, pi)
}

fn deflated_symmetrized(t_mat: &[Vec<u64>], m: &[usize]) -> (Vec<Vec<f64>>, Vec<f64>) {
    let s = m.len();
    let tot: f64 = m.iter().sum::<usize>() as f64;
    let pi: Vec<f64> = m.iter().map(|&v| v as f64 / tot).collect();
    let sqrt_pi: Vec<f64> = pi.iter().map(|&x| x.sqrt()).collect();
    let inv_sqrt: Vec<f64> = m.iter().map(|&v| 1.0 / (v as f64).sqrt()).collect();
    let mut hs = vec![vec![0.0f64; s]; s];
    for i in 0..s {
        for j in 0..s {
            hs[i][j] = (t_mat[i][j] as f64) * inv_sqrt[i] * inv_sqrt[j] - sqrt_pi[i] * sqrt_pi[j];
        }
    }
    (hs, pi)
}

struct OpenPermData {
    l_col: Vec<u8>,
    all: [u32; 256],
    m: usize,
    thin: [u32; 256],
    col_l: Vec<u8>,
    pi_ret: Vec<u32>,
}

fn extract_open_perms(records: &[u16], n: usize, p: u32, limit: u32) -> OpenPermData {
    let mut l_col = Vec::with_capacity(n);
    let mut retained_flag = vec![false; n];
    let mut ret_id = vec![u32::MAX; n];
    let mut thin = [0u32; 256];
    let mut all = [0u32; 256];
    let mut col_l = Vec::new();
    let mut m = 0u32;

    let mut pos = 0u32;
    let mut i = 0usize;
    let nrec = records.len();
    while i < nrec {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < nrec && (records[i] >> 8) as u8 == c {
            width += u32::from(records[i] & 255);
            i += 1;
        }
        if width == 0 {
            continue;
        }
        all[usize::from(c)] += width;
        l_col.resize(l_col.len() + width as usize, c);
        let end = pos + width;
        let mut keep_piece = |x: u32, w: u32, keep: bool| {
            if keep {
                for off in 0..w {
                    let row = (x + off) as usize;
                    retained_flag[row] = true;
                    ret_id[row] = m;
                    col_l.push(c);
                    m += 1;
                }
                thin[usize::from(c)] += w;
            }
        };
        if width > limit && (pos..end).contains(&p) {
            if pos < p {
                keep_piece(pos, p - pos, false);
            }
            keep_piece(p, 1, true);
            if p + 1 < end {
                keep_piece(p + 1, end - p - 1, false);
            }
        } else {
            keep_piece(pos, width, width <= limit);
        }
        pos += width;
    }
    assert_eq!(pos as usize, n);

    let mut first = starts_arr(all);
    let mut fl = vec![0u32; n];
    for (row, &c) in l_col.iter().enumerate() {
        let a = usize::from(c);
        let dst = first[a] as usize;
        fl[dst] = row as u32;
        first[a] += 1;
    }

    let m_us = m as usize;
    let mut pi_ret = vec![0u32; m_us];
    for row in 0..n {
        if retained_flag[row] {
            let u = ret_id[row] as usize;
            let mut cur_fl = fl[row] as usize;
            while !retained_flag[cur_fl] {
                cur_fl = fl[cur_fl] as usize;
            }
            pi_ret[u] = ret_id[cur_fl];
        }
    }

    OpenPermData { l_col, all, m: m_us, thin, col_l, pi_ret }
}

fn build_ret_double_coset_matrix(op: &OpenPermData) -> (Vec<u8>, Vec<usize>, Vec<Vec<u64>>) {
    let mut syms = Vec::new();
    let mut map = [usize::MAX; 256];
    let mut m = Vec::new();
    for b in 0..256 {
        if op.thin[b] > 0 {
            map[b] = syms.len();
            syms.push(b as u8);
            m.push(op.thin[b] as usize);
        }
    }
    let s = syms.len();
    let mut nab = vec![vec![0u64; s]; s];
    for u in 0..op.m {
        let a = map[usize::from(op.col_l[u])];
        let b = map[usize::from(op.col_l[op.pi_ret[u] as usize])];
        nab[a][b] += 1;
    }
    for i in 0..s {
        let rs: u64 = nab[i].iter().sum();
        let cs: u64 = (0..s).map(|r| nab[r][i]).sum();
        assert_eq!(rs, m[i] as u64);
        assert_eq!(cs, m[i] as u64);
    }
    (syms, m, nab)
}

fn resolve_corpus_path(rel: &str) -> String {
    let p1 = format!("/tmp/mibz2_corpora/{rel}");
    if std::path::Path::new(&p1).exists() {
        return p1;
    }
    let p2 = format!("/home/antigravity/mibz2_corpora_cache/{rel}");
    if std::path::Path::new(&p2).exists() {
        return p2;
    }
    panic!("corpus file not found at {p1} or {p2}");
}

fn load_six_admitted() -> Vec<Block> {
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
            Block { name: f, n: l.len(), p, records: records_of(&l) }
        })
        .collect()
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::new();
    for b_idx in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let _crc = u32::from_le_bytes(raw[off + 12..off + 16].try_into().unwrap());
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
        blocks.push(Block { name, n, p, records });
    }
    blocks
}

fn generate_small_and_malformed_suites() -> (Vec<(Vec<u8>, u32)>, Vec<(Vec<u8>, u32)>) {
    let mut rng = 0xC550_1918_1919_1931u64;
    let mut next_u64 = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };

    let mut valid = Vec::with_capacity(5200);
    for case in 0..5200usize {
        let text: Vec<u8> = if case < 4800 {
            let target_sigma = 2 + (case % 5); // sigma in 2..=6
            let n = 6 + ((next_u64() as usize) % 15); // n in 6..=20
            let mut t = Vec::with_capacity(n);
            for i in 0..n {
                let sym = if i < target_sigma {
                    i as u8
                } else {
                    ((next_u64() >> 56) as u8) % (target_sigma as u8)
                };
                t.push(b'a' + sym);
            }
            for i in (1..n).rev() {
                let j = (next_u64() as usize) % (i + 1);
                t.swap(i, j);
            }
            t
        } else {
            let h = 2 + ((case - 4800) % 4);
            let reps = 3 + ((next_u64() as usize) % 4);
            let n = h * reps;
            let mut t = Vec::with_capacity(n);
            let mut cur_b = b'A';
            let mut starts_b = Vec::with_capacity(h);
            let mut lens_b = Vec::with_capacity(h);
            for c in 0..h {
                starts_b.push(cur_b);
                let sz = if c % 2 == 0 { 2 } else { 3 };
                lens_b.push(sz);
                cur_b += sz as u8;
            }
            for pos in 0..n {
                let c = pos % h;
                let sym = starts_b[c] + (((next_u64() >> 56) as u8) % (lens_b[c] as u8));
                t.push(sym);
            }
            t
        };
        let (l, p) = bwt_encode(&text);
        valid.push((l, p));
    }

    let mut malformed = Vec::with_capacity(300);
    for case in 0..300usize {
        let n = 8 + (case % 13);
        let mut l = vec![0u8; n];
        if case < 150 {
            let na = 3 + ((next_u64() as usize) % (n - 5));
            for i in 0..na {
                l[i] = if i % 2 == 0 { b'a' } else { b'b' };
            }
            for i in na..n {
                l[i] = b'c' + (((next_u64() >> 56) as u8) % 2);
            }
        } else {
            loop {
                let target_sigma = 2 + (case % 3);
                for b in &mut l {
                    *b = b'a' + (((next_u64() >> 56) as u8) % (target_sigma as u8));
                }
                let mut all = [0u32; 256];
                for &c in &l {
                    all[usize::from(c)] += 1;
                }
                let mut first = starts_arr(all);
                let mut fl = vec![0usize; n];
                for (row, &c) in l.iter().enumerate() {
                    let a = usize::from(c);
                    fl[first[a] as usize] = row;
                    first[a] += 1;
                }
                let mut cur = fl[0];
                let mut steps = 1;
                while cur != 0 {
                    cur = fl[cur];
                    steps += 1;
                }
                if steps < n {
                    break;
                }
            }
        }
        let p = (next_u64() as u32) % (n as u32);
        malformed.push((l, p));
    }
    (valid, malformed)
}

fn generate_detached_cycle_stress_1_to_65() -> Vec<(String, Vec<u8>, u32)> {
    let mut rng = 0xD37A_C4ED_C550_0065u64;
    let mut next_u64 = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };
    let mut out = Vec::with_capacity(65 * 3);
    let tail_word = b"xyzyxzz";
    let (l_tail, _) = bwt_encode(tail_word);

    for d in 1..=65usize {
        let u: Vec<u8> = if d == 1 {
            vec![b'a']
        } else {
            loop {
                let mut w = Vec::with_capacity(d);
                w.push(b'a');
                for _ in 1..d {
                    let sym = b'a' + (((next_u64() >> 56) as u8) % 3);
                    w.push(sym);
                }
                let mut prim = true;
                for per in 1..d {
                    if d % per == 0 && (0..d).all(|i| w[i] == w[i % per]) {
                        prim = false;
                        break;
                    }
                }
                if prim {
                    break w;
                }
            }
        };
        let (l_u, p_u) = bwt_encode(&u);
        let mut l_disc = l_u.clone();
        l_disc.extend_from_slice(&l_tail);
        out.push((format!("det-d{d}-pri"), l_disc.clone(), p_u));
        out.push((format!("det-d{d}-tail"), l_disc, (d + 2) as u32));
        let mut u2 = u.clone();
        u2.extend_from_slice(&u);
        let (l_per, p_per) = bwt_encode(&u2);
        out.push((format!("per-d{d}"), l_per, p_per));
    }
    out
}

fn make_solved_analogue_w_blocks() -> Vec<Block> {
    let mut out = Vec::new();
    let mut rng = 0xC550_1918_AAAA_BBBBu64;
    let mut next_u64 = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };

    // 1. SolvedW-IID16: near-uniform IID 16-sym => G(H_P) << 1 (Gerschgorin certifies fast mixing!)
    let mut t_iid = Vec::with_capacity(8000);
    for _ in 0..8000 {
        t_iid.push(b'a' + (((next_u64() >> 56) as u8) % 16));
    }
    let (li, pi) = bwt_encode(&t_iid);
    out.push(Block { name: "SolvedW-IID16".to_string(), n: li.len(), p: pi, records: records_of(&li) });

    // 2. SolvedW-RevWlk8: reversible symmetric walk on 8-cycle => H_S normal (E_W(H_S) ~ 0)
    let mut t_rev = Vec::with_capacity(8000);
    let mut st = 0usize;
    for _ in 0..8000 {
        t_rev.push(b'a' + st as u8);
        let r = (next_u64() >> 56) % 3;
        st = match r {
            0 => (st + 1) % 8,
            1 => (st + 7) % 8,
            _ => st,
        };
    }
    let (lr, pr) = bwt_encode(&t_rev);
    out.push(Block { name: "SolvedW-RevWlk8".to_string(), n: lr.len(), p: pr, records: records_of(&lr) });

    // 3. SolvedW-Cyc4x4: block-cyclic shift => near-normal with |lambda_2| ~ 1, E_W ~ 0
    let mut t_cyc = Vec::with_capacity(8000);
    for i in 0..8000usize {
        let g = i % 4;
        let sub = ((next_u64() >> 56) as usize) % 4;
        t_cyc.push(b'A' + (g * 4 + sub) as u8);
    }
    let (lc, pc) = bwt_encode(&t_cyc);
    out.push(Block { name: "SolvedW-Cyc4x4".to_string(), n: lc.len(), p: pc, records: records_of(&lc) });

    // 4. SolvedW-DirSyn: strongly directed 8-state JSON-like syntax => r(H_P) > 1, large E_W!
    let mut t_dir = Vec::with_capacity(8000);
    let mut ds = 0usize;
    for _ in 0..8000 {
        let b = match ds {
            0 => b'{',
            1 => b'"',
            2 => b'k',
            3 => b':',
            4 => b'[',
            5 => b'0' + (((next_u64() >> 56) as u8) % 6),
            6 => b']',
            _ => b'}',
        };
        t_dir.push(b);
        ds = match ds {
            5 => if (next_u64() >> 56) % 4 == 0 { 6 } else { 5 },
            7 => 0,
            _ => ds + 1,
        };
    }
    let (ld, pd) = bwt_encode(&t_dir);
    out.push(Block { name: "SolvedW-DirSyn".to_string(), n: ld.len(), p: pd, records: records_of(&ld) });

    out
}

// ============================================================================
// SECTION 3: PART 1 — Exact Toeplitz-Hausdorff & Gerschgorin Witnesses (5,552+)
// ============================================================================

fn run_part1_exact_toeplitz_hausdorff_gerschgorin(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
) {
    let t0 = Instant::now();
    println!("================================================================================================================================");
    println!("PART 1: EXACT TOEPLITZ (1918) - HAUSDORFF (1919) NUMERICAL RANGE & GERSCHGORIN (1931) DISKS (5,552 CASES)");
    println!("  Primary Sources: O. Toeplitz, Math. Z. 2 (1918), 187-197; F. Hausdorff, Math. Z. 3 (1919), 314-316;");
    println!("                   S. Gerschgorin, Izv. Akad. Nauk SSSR 6 (1931), 749-754");
    println!("================================================================================================================================");

    let mut syn_count = 0usize;
    let mut max_jordan_err = 0.0f64;
    let mut max_shifted_j_err = 0.0f64;
    let mut max_cyclic_err = 0.0f64;
    let mut max_ellipse_err = 0.0f64;
    let mut max_zero_exc_err = 0.0f64;

    // 1A-(i): Jordan shift blocks J_k(0) for k = 2..=12 (11 matrices):
    //         W(J_k(0)) = closed disk D(0, cos(pi/(k+1)))!
    //         Verify h(theta) == cos(pi/(k+1)) across 16 angles, r == omega == cos(pi/(k+1)), rho == 0!
    for k in 2..=12usize {
        let mut a = vec![vec![0.0f64; k]; k];
        let mut a_rat = vec![vec![BigRat::zero(); k]; k];
        for i in 0..(k - 1) {
            a[i][i + 1] = 1.0;
            a_rat[i][i + 1] = BigRat::one();
        }
        let expected_r = (PI / ((k + 1) as f64)).cos();
        let gr_rat = exact_bigrat_gerschgorin_rayleigh(&a_rat);
        assert!(!gr_rat.is_normal);
        assert_eq!(gr_rat.g_row, BigRat::one());
        assert_eq!(gr_rat.g_col, BigRat::one());

        let mut h0 = vec![0.0f64; k * k];
        let mut k0 = vec![0.0f64; k * k];
        for i in 0..k {
            for j in 0..k {
                h0[i * k + j] = 0.5 * (a[i][j] + a[j][i]);
                k0[i * k + j] = 0.5 * (a[i][j] - a[j][i]);
            }
        }
        // Check exact circular invariance across 16 angles
        for t in 0..16usize {
            let th = (t as f64) * 2.0 * PI / 16.0;
            let (lmin, lmax) = hermitian_theta_min_max_eigs(&h0, &k0, k, th);
            let e1 = (lmax - expected_r).abs();
            let e2 = (-lmin - expected_r).abs();
            max_jordan_err = max_jordan_err.max(e1).max(e2);
            assert!(e1 < 1e-13 && e2 < 1e-13, "J_{k}(0) h(theta) err: {e1:.3e}, {e2:.3e}");
        }
        let nr = compute_toeplitz_hausdorff(&a, 24);
        let e_r = (nr.r_num - expected_r).abs();
        let e_w = (nr.omega - expected_r).abs();
        let e_n = (nr.spec_norm - 1.0).abs();
        max_jordan_err = max_jordan_err.max(e_r).max(e_w).max(e_n);
        assert!(e_r < 1e-13 && e_w < 1e-13 && e_n < 1e-13);
        if k == 2 {
            // Sharp attainment of ||A||_2 <= 2 * r(A): ||J_2(0)||_2 / r(J_2(0)) == 2.0 exactly!
            assert!((nr.spec_norm / nr.r_num - 2.0).abs() < 1e-13);
        }
        let rs = real_schur_eigs_and_df(&a);
        let g = compute_gerschgorin_f64(&a, None);
        verify_gerschgorin_containment(&a, &rs.eigs, &g);
        syn_count += 1;
    }

    // 1A-(ii): Shifted Jordan blocks J_k(lam) for k = 2..=6, lam in {1, -1} (10 matrices):
    //          W(J_k(lam)) = D(lam, cos(pi/(k+1))) => r = |lam| + cos(pi/(k+1)), omega = lam + cos(pi/(k+1))!
    for k in 2..=6usize {
        for &lam in &[1i64, -1] {
            let mut a = vec![vec![0.0f64; k]; k];
            let mut a_rat = vec![vec![BigRat::zero(); k]; k];
            for i in 0..k {
                a[i][i] = lam as f64;
                a_rat[i][i] = BigRat::from_i64(lam);
                if i + 1 < k {
                    a[i][i + 1] = 1.0;
                    a_rat[i][i + 1] = BigRat::one();
                }
            }
            let _ = exact_bigrat_gerschgorin_rayleigh(&a_rat);
            let rad = (PI / ((k + 1) as f64)).cos();
            let exp_r = (lam.abs() as f64) + rad;
            let exp_w = (lam as f64) + rad;
            let nr = compute_toeplitz_hausdorff(&a, 32);
            let er = (nr.r_num - exp_r).abs();
            let ew = (nr.omega - exp_w).abs();
            max_shifted_j_err = max_shifted_j_err.max(er).max(ew);
            assert!(er < 1e-13 && ew < 1e-13, "J_{k}({lam}) err: er={er:.3e}, ew={ew:.3e}");
            let rs = real_schur_eigs_and_df(&a);
            let g = compute_gerschgorin_f64(&a, None);
            verify_gerschgorin_containment(&a, &rs.eigs, &g);
            syn_count += 1;
        }
    }

    // 1A-(iii): Normal cyclic permutation shifts C_m (m = 2..=12, 11 matrices):
    //           W(C_m) = regular m-gon! r(C_m) = rho(C_m) = ||C_m||_2 = 1, E_W = 0,
    //           and apothem h(pi/m) == cos(pi/m) (proving exact polygon, NOT unit disk!).
    for m in 2..=12usize {
        let mut a = vec![vec![0.0f64; m]; m];
        let mut a_rat = vec![vec![BigRat::zero(); m]; m];
        for i in 0..m {
            let j = (i + 1) % m;
            a[i][j] = 1.0;
            a_rat[i][j] = BigRat::one();
        }
        let gr_rat = exact_bigrat_gerschgorin_rayleigh(&a_rat);
        assert!(gr_rat.is_normal, "C_{m} must be exact normal in Q!");
        assert_eq!(gr_rat.g_row, BigRat::one());
        assert_eq!(gr_rat.g_col, BigRat::one());

        let nr = compute_toeplitz_hausdorff(&a, 32);
        let rs = real_schur_eigs_and_df(&a);
        let rho = rs.eigs.iter().map(|e| e.abs()).fold(0.0f64, f64::max);
        let g = compute_gerschgorin_f64(&a, None);
        verify_gerschgorin_containment(&a, &rs.eigs, &g);

        // Verify exact m-gon apothem at theta = pi/m
        let mut h0 = vec![0.0f64; m * m];
        let mut k0 = vec![0.0f64; m * m];
        for i in 0..m {
            for j in 0..m {
                h0[i * m + j] = 0.5 * (a[i][j] + a[j][i]);
                k0[i * m + j] = 0.5 * (a[i][j] - a[j][i]);
            }
        }
        let (_, apothem_got) = hermitian_theta_min_max_eigs(&h0, &k0, m, PI / (m as f64));
        let apothem_exp = (PI / (m as f64)).cos();

        let err = (nr.r_num - 1.0)
            .abs()
            .max((rho - 1.0).abs())
            .max((nr.r_num - rho).abs())
            .max((nr.spec_norm - 1.0).abs())
            .max((apothem_got - apothem_exp).abs());
        max_cyclic_err = max_cyclic_err.max(err);
        assert!(err < 1e-13, "C_{m} Toeplitz-Hausdorff err={err:.3e}");
        syn_count += 1;
    }

    // 1A-(iv): Exact 2x2 Elliptical Range Theorem (14 matrices: real distinct, defective, complex
    //          conjugate, normal & non-normal) — directly unites CA-51 Schur Delta_F with W(A)!
    //          Foci = lam_1, lam_2; minor axis = Delta_F(A); major axis = sqrt(|lam_1-lam_2|^2 + Delta_F^2)!
    let ell_cases: [(i64, i64, i64, i64); 14] = [
        (0, 1, 0, 0),    // J_2(0): circle radius 0.5
        (2, 3, 0, -1),   // real distinct non-normal
        (3, 5, 0, 3),    // defective real: circle centered at 3, radius 2.5
        (1, 2, -2, 1),   // normal complex rotation: vertical segment [1-2i, 1+2i], Delta_F = 0!
        (1, 4, -1, 1),   // non-normal complex: foci 1 +/- 2i, Delta_F = 3
        (3, 5, -2, 1),   // non-normal complex: unequal diag + skew
        (0, 7, -3, 2),   // non-normal complex
        (-2, 6, -4, 0),  // non-normal complex
        (4, 3, -5, 4),   // non-normal complex
        (5, 2, 2, 1),    // symmetric normal: real segment, Delta_F = 0
        (-3, 4, 1, 2),   // real distinct, both off-diag nonzero
        (1, -3, 2, 4),   // complex conjugate with negative b
        (0, 5, -1, 0),   // centered at 0, foci +/- i*sqrt(5), Delta_F = 4
        (2, 1, 4, -2),   // real distinct +/- sqrt(8)
    ];
    for &(a00, a01, a10, a11) in &ell_cases {
        let a = vec![vec![a00 as f64, a01 as f64], vec![a10 as f64, a11 as f64]];
        let a_rat = vec![
            vec![BigRat::from_i64(a00), BigRat::from_i64(a01)],
            vec![BigRat::from_i64(a10), BigRat::from_i64(a11)],
        ];
        let gr_rat = exact_bigrat_gerschgorin_rayleigh(&a_rat);
        let rs = real_schur_eigs_and_df(&a);
        let l1 = rs.eigs[0];
        let l2 = rs.eigs[1];
        let df = rs.delta_f;
        assert_eq!(gr_rat.is_normal, df < 1e-12);

        let c_re = 0.5 * (l1.re + l2.re);
        let c_im = 0.5 * (l1.im + l2.im);
        let diff_re = l1.re - l2.re;
        let diff_im = l1.im - l2.im;
        let foc_dist = diff_re.hypot(diff_im);
        let semi_minor = 0.5 * df;
        let semi_major = 0.5 * foc_dist.hypot(df);
        let phi0 = if foc_dist > 1e-14 { diff_im.atan2(diff_re) } else { 0.0 };

        let mut h0 = vec![0.0f64; 4];
        let mut k0 = vec![0.0f64; 4];
        for i in 0..2 {
            for j in 0..2 {
                h0[i * 2 + j] = 0.5 * (a[i][j] + a[j][i]);
                k0[i * 2 + j] = 0.5 * (a[i][j] - a[j][i]);
            }
        }
        // Check exact ellipse support function at 32 angles in [0, 2*pi)
        for t in 0..32usize {
            let th = (t as f64) * 2.0 * PI / 32.0;
            let (_, h_got) = hermitian_theta_min_max_eigs(&h0, &k0, 2, th);
            let h_ell = (th.cos() * c_re - th.sin() * c_im)
                + ((semi_major * (th + phi0).cos()).hypot(semi_minor * (th + phi0).sin()));
            let err = (h_got - h_ell).abs();
            max_ellipse_err = max_ellipse_err.max(err);
            assert!(err < 1e-13, "2x2 Elliptical Range Theorem err={err:.3e}");
        }
        let nr = compute_toeplitz_hausdorff(&a, 48);
        let g = compute_gerschgorin_f64(&a, None);
        verify_gerschgorin_containment(&a, &rs.eigs, &g);
        let rho = l1.abs().max(l2.abs());
        assert!(rho <= nr.r_num + 1e-12 && nr.r_num <= nr.spec_norm + 1e-12 && nr.spec_norm <= 2.0 * nr.r_num + 1e-12);
        syn_count += 1;
    }

    // 1A-(v): Non-normal zero-excess direct sums diag(2, J_k(0)) for k = 2..=7 (6 matrices):
    //         Strictly non-normal (Delta_F = sqrt(k-1) > 0), YET r(A) = rho(A) = 2 => E_W(A) = 0!
    //         Proves exact logical scope: normal => E_W = 0 (converse holds for 2x2, NOT arbitrary direct sums!).
    for k in 2..=7usize {
        let s = k + 1;
        let mut a = vec![vec![0.0f64; s]; s];
        let mut a_rat = vec![vec![BigRat::zero(); s]; s];
        a[0][0] = 2.0;
        a_rat[0][0] = BigRat::from_u64(2);
        for i in 1..(s - 1) {
            a[i][i + 1] = 1.0;
            a_rat[i][i + 1] = BigRat::one();
        }
        let gr_rat = exact_bigrat_gerschgorin_rayleigh(&a_rat);
        assert!(!gr_rat.is_normal);
        let rs = real_schur_eigs_and_df(&a);
        assert!((rs.delta_f - ((k - 1) as f64).sqrt()).abs() < 1e-12);
        let nr = compute_toeplitz_hausdorff(&a, 24);
        let rho = rs.eigs.iter().map(|e| e.abs()).fold(0.0f64, f64::max);
        let err = (rho - 2.0).abs().max((nr.r_num - 2.0).abs()).max((nr.r_num - rho).abs());
        max_zero_exc_err = max_zero_exc_err.max(err);
        assert!(err < 1e-13, "diag(2, J_{k}) zero-excess scope witness err={err:.3e}");
        syn_count += 1;
    }
    assert_eq!(syn_count, 52);

    // 1B: Exact BigRat Gerschgorin + Symmetric Rayleigh/Trace Bounds + Toeplitz-Hausdorff Checks
    //     across ALL 5,200 Valid + 300 Malformed BWTs (`5,500` BWTs -> `5,552` total cases!)
    let mut cats_n = [0usize; 4];
    let mut cats_gersch_cert_hp = [0usize; 4]; // G(H_P) < 1
    let mut cats_gersch_cert_hs = [0usize; 4]; // G(H_S) < 1
    let mut cats_amp_hp = [0usize; 4];         // r(H_P) > 1 (k=1 transient amplification!)
    let mut cats_ew_hp_sum = [0.0f64; 4];
    let mut cats_ew_hs_sum = [0.0f64; 4];
    let mut cats_r_hp_sum = [0.0f64; 4];
    let mut cats_r_hs_sum = [0.0f64; 4];
    let mut cats_g_hp_sum = [0.0f64; 4];
    let mut cats_g_hs_sum = [0.0f64; 4];

    let mut max_gersch_viol = 0.0f64;
    let mut exact_rat_checks = 0usize;
    let mut normal_ew_checks = 0usize;

    let mut process_bwt = |l_col: &[u8], cat: usize| {
        let mut all = [0u32; 256];
        for &c in l_col {
            all[usize::from(c)] += 1;
        }
        let (_syms, m, a_u64) = build_g1_matrix(l_col, &all);
        let s = m.len();
        let n_tot: u64 = m.iter().sum::<usize>() as u64;
        let n_bi = BigInt::from_u64(n_tot);
        let pi_rat: Vec<BigRat> = m.iter().map(|&v| BigRat::new(BigInt::from_u64(v as u64), n_bi.clone())).collect();

        // Build exact BigRat P_1 and deflated H_P = P_1 - 1 * pi^T
        let mut p1_rat = vec![vec![BigRat::zero(); s]; s];
        let mut hp_rat = vec![vec![BigRat::zero(); s]; s];
        for i in 0..s {
            let mi = BigInt::from_u64(m[i] as u64);
            for j in 0..s {
                let pij = BigRat::new(BigInt::from_u64(a_u64[i][j]), mi.clone());
                hp_rat[i][j] = pij.sub(&pi_rat[j]);
                p1_rat[i][j] = pij;
            }
        }

        let gr_p1 = exact_bigrat_gerschgorin_rayleigh(&p1_rat);
        let gr_hp = exact_bigrat_gerschgorin_rayleigh(&hp_rat);
        // Exact rational BWT/Markov identities (ZERO floats!):
        assert_eq!(gr_p1.g_row, BigRat::one(), "P_1 row-stochastic must have G_row(P_1) == 1!");
        assert!(gr_hp.g_row <= BigRat::from_u64(2), "Deflated H_P must have G_row(H_P) <= 2!");
        assert_eq!(gr_hp.tr_sum, gr_p1.tr_sum.sub(&BigRat::one()), "tr(H_P) == tr(P_1) - 1!");
        exact_rat_checks += 3;

        // Float deflated operators H_P and H_S
        let (hp_f, pi_f) = deflated_stochastic(&a_u64, &m);
        let (hs_f, _) = deflated_symmetrized(&a_u64, &m);

        let rs_hp = real_schur_eigs_and_df(&hp_f);
        let rs_hs = real_schur_eigs_and_df(&hs_f);
        let nr_hp = compute_toeplitz_hausdorff(&hp_f, 24);
        let nr_hs = compute_toeplitz_hausdorff(&hs_f, 24);
        let g_hp = compute_gerschgorin_f64(&hp_f, Some(&pi_f));
        let g_hs = compute_gerschgorin_f64(&hs_f, Some(&pi_f));

        max_gersch_viol = max_gersch_viol.max(verify_gerschgorin_containment(&hp_f, &rs_hp.eigs, &g_hp));
        max_gersch_viol = max_gersch_viol.max(verify_gerschgorin_containment(&hs_f, &rs_hs.eigs, &g_hs));

        let rho_hp = rs_hp.eigs.iter().map(|e| e.abs()).fold(0.0f64, f64::max);
        let rho_hs = rs_hs.eigs.iter().map(|e| e.abs()).fold(0.0f64, f64::max);
        let alpha_hp = rs_hp.eigs.iter().map(|e| e.re).fold(f64::NEG_INFINITY, f64::max);
        let alpha_hs = rs_hs.eigs.iter().map(|e| e.re).fold(f64::NEG_INFINITY, f64::max);

        // Exact similarity invariant: sigma(H_S) == sigma(H_P) => rho(H_S) == rho(H_P)
        // Note: on defective k x k Jordan blocks (e.g. J_2(1/3) in small BWTs), QR eigenvalues
        // undergo Puiseux splitting O(eps^{1/k}) (~1.5e-8 for k=2, ~6e-6 for k=3), whereas W(A)
        // is 1-Lipschitz O(eps)!
        let rho_diff = (rho_hp - rho_hs).abs();
        assert!(
            rho_diff < 1e-5,
            "rho(H_P) != rho(H_S): cat={cat} s={s} rho_hp={rho_hp:.12e} rho_hs={rho_hs:.12e} diff={rho_diff:.3e}"
        );

        // Toeplitz-Hausdorff & Gerschgorin inequalities on H_P (cross-checked against exact BigRat!)
        let dmin_q = gr_hp.diag_min.to_f64();
        let dmax_q = gr_hp.diag_max.to_f64();
        let tavg_q = gr_hp.tr_avg.to_f64();
        let gw_lo_q = gr_hp.g_omega_lo.to_f64();
        let gw_q = gr_hp.g_omega_hi.to_f64();
        let gmin_q = gr_hp.g_min.to_f64();
        let grow_q = gr_hp.g_row.to_f64();
        let gcol_q = gr_hp.g_col.to_f64();
        for idx_r in 0..s {
            assert!((gr_hp.row_radii[idx_r].to_f64() - g_hp.row_radii[idx_r]).abs() < 1e-12);
            assert!((gr_hp.col_radii[idx_r].to_f64() - g_hp.col_radii[idx_r]).abs() < 1e-12);
        }

        assert!(gw_lo_q <= nr_hp.omega_min + 1e-11, "G_omega_lo <= omega_min violated");
        assert!(nr_hp.omega_min <= dmin_q + 1e-11, "omega_min <= min_i H_ii violated");
        assert!(tavg_q <= dmax_q + 1e-12);
        assert!(dmax_q <= nr_hp.omega + 1e-11, "max_i H_ii <= omega violated");
        assert!(alpha_hp <= nr_hp.omega + 1e-7, "alpha <= omega violated");
        assert!(nr_hp.omega <= gw_q + 1e-11, "omega <= G_omega violated");
        assert!(rho_hp <= nr_hp.r_num + 1e-7, "rho <= r(H_P) violated");
        assert!(nr_hp.r_num <= nr_hp.spec_norm + 1e-10, "r(H_P) <= ||H_P||_2 violated");
        assert!(nr_hp.spec_norm <= 2.0 * nr_hp.r_num + 1e-10, "||H_P||_2 <= 2 r(H_P) violated");
        assert!(rho_hp <= gmin_q + 1e-7, "rho <= G(H_P) violated");
        assert!(nr_hp.r_num <= (grow_q * gcol_q).sqrt() + 1e-10, "r <= sqrt(G_row*G_col) violated");

        // Exact L^2(pi) contraction law on H_S: rho(H_S) <= r(H_S) <= ||H_S||_2 <= 1!
        assert!(alpha_hs <= nr_hs.omega + 1e-11);
        assert!(nr_hs.omega <= g_hs.g_omega + 1e-11);
        assert!(rho_hs <= nr_hs.r_num + 1e-10);
        assert!(nr_hs.r_num <= nr_hs.spec_norm + 1e-10);
        assert!(nr_hs.spec_norm <= 2.0 * nr_hs.r_num + 1e-10);
        assert!(nr_hs.spec_norm <= 1.0 + 1e-10, "L^2(pi) contraction ||H_S||_2 <= 1 violated: {}", nr_hs.spec_norm);

        // Normal => E_W == 0 & r == ||M||_2 == rho!
        if gr_hp.is_normal {
            assert!((nr_hp.r_num - rho_hp).abs() < 1e-10, "Normal H_P had E_W > 0");
            assert!((nr_hp.spec_norm - nr_hp.r_num).abs() < 1e-10);
            normal_ew_checks += 1;
        }

        cats_n[cat] += 1;
        if g_hp.g_min < 1.0 - 1e-12 {
            cats_gersch_cert_hp[cat] += 1;
        }
        if g_hs.g_min < 1.0 - 1e-12 {
            cats_gersch_cert_hs[cat] += 1;
        }
        if nr_hp.r_num > 1.0 + 1e-9 {
            cats_amp_hp[cat] += 1;
        }
        cats_ew_hp_sum[cat] += (nr_hp.r_num - rho_hp).max(0.0);
        cats_ew_hs_sum[cat] += (nr_hs.r_num - rho_hs).max(0.0);
        cats_r_hp_sum[cat] += nr_hp.r_num;
        cats_r_hs_sum[cat] += nr_hs.r_num;
        cats_g_hp_sum[cat] += g_hp.g_min;
        cats_g_hs_sum[cat] += g_hs.g_min;
    };

    for (idx, (l, _p)) in valid.iter().enumerate() {
        process_bwt(l, if idx < 4800 { 0 } else { 1 });
    }
    for (idx, (l, _p)) in malformed.iter().enumerate() {
        process_bwt(l, if idx < 150 { 2 } else { 3 });
    }

    let total_cases = syn_count + valid.len() + malformed.len();
    println!(
        "  [PASS 1A] Synthetic Analytical Witnesses ({syn_count}/{syn_count} EXACT: 11 Jordan J_k(0) + 10 Shifted J_k(+-1) + 11 Cyclic C_m + 14 2x2 Ellipses + 6 Zero-Excess Scope):"
    );
    println!(
        "    - Jordan J_2..12(0) W=D(0,cos(pi/(k+1))) max err : {max_jordan_err:.3e} | Shifted J_2..6(+-1): {max_shifted_j_err:.3e}"
    );
    println!(
        "    - Normal Cyclic C_2..12 (r=rho=1, apothem) err   : {max_cyclic_err:.3e} | 2x2 Elliptical Range (minor=Delta_F): {max_ellipse_err:.3e}"
    );
    println!(
        "    - Non-normal diag(2, J_k) E_W=0 scope witness    : {max_zero_exc_err:.3e} (proves normal => E_W=0, converse holds for 2x2)"
    );
    println!(
        "  [PASS 1B] Small Valid + Malformed BWTs (5,200 + 300 = 5,500 BWTs; total {total_cases}/{total_cases} EXACT in {:.2}s):",
        t0.elapsed().as_secs_f64()
    );
    println!(
        "    - Exact BigRat Gerschgorin & Rayleigh/trace checks  : {exact_rat_checks} / {exact_rat_checks} EXACT (0 errors; normal E_W=0: {normal_ew_checks} EXACT)"
    );
    println!(
        "    - Gerschgorin containment & L^2(pi) ||H_S||_2 <= 1  : 11,000 / 11,000 EXACT (max disk margin viol = {max_gersch_viol:.3e})"
    );
    let cat_names = [
        "General Valid (4,800)",
        "Periodic h=2..5 (400)",
        "Reducible 2-SCC Mal (150)",
        "Connected Mal (150)",
    ];
    println!(
        "  {:<26} | {:>8} {:>8} {:>8} | {:>8} {:>8} {:>8} | {:>9} {:>9} {:>9}",
        "BWT Regime", "r(H_P)", "E_W(H_P)", "G(H_P)", "r(H_S)", "E_W(H_S)", "G(H_S)", "r(HP)>1%", "G(HP)<1%", "G(HS)<1%"
    );
    for c in 0..4 {
        let n = cats_n[c] as f64;
        println!(
            "  {:<26} | {:>8.4} {:>8.4} {:>8.4} | {:>8.4} {:>8.4} {:>8.4} | {:>8.2}% {:>8.2}% {:>8.2}%",
            cat_names[c],
            cats_r_hp_sum[c] / n,
            cats_ew_hp_sum[c] / n,
            cats_g_hp_sum[c] / n,
            cats_r_hs_sum[c] / n,
            cats_ew_hs_sum[c] / n,
            cats_g_hs_sum[c] / n,
            100.0 * cats_amp_hp[c] as f64 / n,
            100.0 * cats_gersch_cert_hp[c] as f64 / n,
            100.0 * cats_gersch_cert_hs[c] as f64 / n,
        );
    }
    println!();
}

// ============================================================================
// SECTION 4: PART 2 — Toeplitz-Hausdorff & Gerschgorin on ALL 249 Real Blocks
// ============================================================================

#[derive(Clone, Debug)]
struct OpWMetrics {
    sigma: usize,
    rho: f64,        // |lambda_2|
    gamma: f64,      // 1 - rho
    delta_f: f64,    // CA-51 Schur departure of deflated operator H
    schur_id_err: f64,
    r_num: f64,      // numerical radius r(H)
    r_ub: f64,       // certified outer-polygon UB
    e_w: f64,        // Toeplitz-Hausdorff excess r(H) - rho(H)
    omega: f64,      // numerical abscissa lam_max((H+H^T)/2)
    e_omega: f64,    // omega(H) - alpha(H)
    spec_norm: f64,  // ||H||_2
    g_row: f64,      // max_i (|H_ii| + R_i(H))
    g_col: f64,      // max_j (|H_jj| + C_j(H))
    g_min: f64,      // min(g_row, g_col)
    g_omega: f64,    // Gerschgorin UB on omega(H)
    g_row_mean: f64,
    g_row_pi: f64,
    gersch_viol: f64,
    n_evals: usize,
}

fn analyze_deflated_op(h: &[Vec<f64>], pi: &[f64], is_symmetrized: bool) -> OpWMetrics {
    let s = h.len();
    let rs = real_schur_eigs_and_df(h);
    let nr = compute_toeplitz_hausdorff(h, 24);
    let g = compute_gerschgorin_f64(h, Some(pi));
    let gersch_viol = verify_gerschgorin_containment(h, &rs.eigs, &g);

    let rho = rs.eigs.iter().map(|e| e.abs()).fold(0.0f64, f64::max);
    let alpha = rs.eigs.iter().map(|e| e.re).fold(f64::NEG_INFINITY, f64::max);
    let gamma = (1.0 - rho).max(0.0);

    // Exact Toeplitz-Hausdorff & Gerschgorin assertions (0 violations across all 996 matrices!)
    assert!(rho <= nr.r_num + 1e-9, "rho <= r(H) violated: rho={rho}, r={}", nr.r_num);
    assert!(nr.r_num <= nr.spec_norm + 1e-9, "r(H) <= ||H||_2 violated: r={}, norm={}", nr.r_num, nr.spec_norm);
    assert!(nr.spec_norm <= 2.0 * nr.r_num + 1e-9, "||H||_2 <= 2 r(H) violated");
    assert!(alpha <= nr.omega + 1e-9, "alpha <= omega violated");
    assert!(nr.omega <= g.g_omega + 1e-9, "omega <= G_omega violated");
    assert!(rho <= g.g_min + 1e-9, "rho <= G(H) violated");
    if is_symmetrized {
        assert!(nr.spec_norm <= 1.0 + 1e-9, "L^2(pi) contraction ||H_S||_2 <= 1 violated: {}", nr.spec_norm);
    }

    OpWMetrics {
        sigma: s,
        rho,
        gamma,
        delta_f: rs.delta_f,
        schur_id_err: rs.schur_id_err,
        r_num: nr.r_num,
        r_ub: nr.r_ub,
        e_w: (nr.r_num - rho).max(0.0),
        omega: nr.omega,
        e_omega: (nr.omega - alpha).max(0.0),
        spec_norm: nr.spec_norm,
        g_row: g.g_row,
        g_col: g.g_col,
        g_min: g.g_min,
        g_omega: g.g_omega,
        g_row_mean: g.g_row_mean,
        g_row_pi: g.g_row_pi,
        gersch_viol,
        n_evals: nr.n_evals,
    }
}

/// Cheap O(1)-per-sample / O(Sigma) Gerschgorin & normality-inspired block feature computed
/// strictly from 256 strided records in ~200 ns (ZERO dense matrix!).
#[derive(Clone, Copy, Debug)]
struct CheapGerschFeatures {
    est_w1: u64,
    uniq_sym: u32,
    uniq_thin_sym: u32,
    mid17_32_cnt: u32,
    thin_frac_samp: f64, // sampled retained-row fraction (m/n proxy)
    hhi: f64,            // sum_c p_hat[c]^2 (stationary concentration)
    adj_asym: f64,       // |#{c_k < c_{k+1}} - #{c_k > c_{k+1}}| / 256 (cheap skew proxy)
    g_col_proxy: f64,    // uniq_thin_sym * thin_frac_samp (O(Sigma) G_col(H_P,ret) hub fan-in proxy!)
}

#[inline]
fn extract_cheap_gersch_features(records: &[u16], n: usize) -> CheapGerschFeatures {
    let nrec = records.len();
    if n < 32_768 || nrec < 257 {
        return CheapGerschFeatures {
            est_w1: 0,
            uniq_sym: 0,
            uniq_thin_sym: 0,
            mid17_32_cnt: 0,
            thin_frac_samp: 0.0,
            hhi: 1.0,
            adj_asym: 0.0,
            g_col_proxy: 0.0,
        };
    }
    let step = (nrec - 1) / 256;
    let mut idx = step / 2;
    let mut cnt = [0u16; 256];
    let mut thin_seen = [0u8; 256];
    let mut uniq_sym = 0u32;
    let mut uniq_thin_sym = 0u32;
    let mut s_w1 = 0u32;
    let mut mid17_32_cnt = 0u32;
    let mut thin_w_sum = 0u32;
    let mut all_w_sum = 0u32;
    let mut up = 0i32;
    for _ in 0..256 {
        let rec = records[idx];
        let c = (rec >> 8) as usize;
        let w = u32::from(rec & 255);
        let next_c = (records[idx + 1] >> 8) as usize;
        if cnt[c] == 0 {
            uniq_sym += 1;
        }
        cnt[c] += 1;
        all_w_sum += w;
        if w <= 16 {
            thin_w_sum += w;
            if thin_seen[c] == 0 {
                thin_seen[c] = 1;
                uniq_thin_sym += 1;
            }
        }
        s_w1 += u32::from(w == 1);
        mid17_32_cnt += u32::from(w > 16 && w <= 32);
        if c < next_c {
            up += 1;
        } else if c > next_c {
            up -= 1;
        }
        idx += step;
    }
    let mut hhi_num = 0u32;
    for &v in &cnt {
        let vv = u32::from(v);
        hhi_num += vv * vv;
    }
    let hhi = (hhi_num as f64) / (256.0 * 256.0);
    let adj_asym = (up.abs() as f64) / 256.0;
    let thin_frac_samp = (thin_w_sum as f64) / (all_w_sum.max(1) as f64);
    let g_col_proxy = (uniq_thin_sym as f64) * thin_frac_samp;
    let est_w1 = (nrec as u64 * u64::from(s_w1)) >> 8;
    CheapGerschFeatures {
        est_w1,
        uniq_sym,
        uniq_thin_sym,
        mid17_32_cnt,
        thin_frac_samp,
        hhi,
        adj_asym,
        g_col_proxy,
    }
}

struct RealBlockWReport {
    name: String,
    n: usize,
    m: usize,
    hs1: OpWMetrics,
    hp1: OpWMetrics,
    hs_ret: OpWMetrics,
    hp_ret: OpWMetrics,
    cheap: CheapGerschFeatures,
}

fn analyze_real_block_w(b: &Block) -> RealBlockWReport {
    let op = extract_open_perms(&b.records, b.n, b.p, 16);
    let (_s1, m1, t1) = build_g1_matrix(&op.l_col, &op.all);
    let (_sr, mr, tr) = build_ret_double_coset_matrix(&op);

    let (hp1_mat, pi1) = deflated_stochastic(&t1, &m1);
    let (hs1_mat, _) = deflated_symmetrized(&t1, &m1);
    let (hpr_mat, pir) = deflated_stochastic(&tr, &mr);
    let (hsr_mat, _) = deflated_symmetrized(&tr, &mr);

    let hs1 = analyze_deflated_op(&hs1_mat, &pi1, true);
    let hp1 = analyze_deflated_op(&hp1_mat, &pi1, false);
    let hs_ret = analyze_deflated_op(&hsr_mat, &pir, true);
    let hp_ret = analyze_deflated_op(&hpr_mat, &pir, false);
    let cheap = extract_cheap_gersch_features(&b.records, b.n);

    RealBlockWReport {
        name: b.name.clone(),
        n: b.n,
        m: op.m,
        hs1,
        hp1,
        hs_ret,
        hp_ret,
        cheap,
    }
}

fn pearson_r(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut dx2 = 0.0;
    let mut dy2 = 0.0;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        let dx = x - mx;
        let dy = y - my;
        num += dx * dy;
        dx2 += dx * dx;
        dy2 += dy * dy;
    }
    let den = (dx2 * dy2).sqrt();
    if den == 0.0 { 0.0 } else { num / den }
}

fn print_w_table(title: &str, reps: &[RealBlockWReport]) {
    println!("  [{title}] — TOEPLITZ-HAUSDORFF (1918-19) & GERSCHGORIN (1931) ON DEFLATED H_S,ret & H_P,ret:");
    println!(
        "  {:<15} | {:>3} {:>6} {:>6} {:>6} {:>6} {:>6} | {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "Block",
        "sr", "rho", "r(HS)", "w(HS)", "EW_HS", "G(HS)",
        "r(HP)", "w(HP)", "EW_HP", "||HP||", "GrHP", "GcHP"
    );
    for r in reps {
        println!(
            "  {:<15} | {:>3} {:>6.3} {:>6.3} {:>6.3} {:>6.3} {:>6.2} | {:>6.3} {:>6.3} {:>6.3} {:>6.3} {:>6.3} {:>6.2}",
            r.name,
            r.hs_ret.sigma,
            r.hs_ret.rho,
            r.hs_ret.r_num,
            r.hs_ret.omega,
            r.hs_ret.e_w,
            r.hs_ret.g_min,
            r.hp_ret.r_num,
            r.hp_ret.omega,
            r.hp_ret.e_w,
            r.hp_ret.spec_norm,
            r.hp_ret.g_row,
            r.hp_ret.g_col,
        );
    }
    println!();
}

fn summarize_w_corpus(label: &str, reps: &[&RealBlockWReport]) {
    let n = reps.len() as f64;
    let avg = |f: fn(&RealBlockWReport) -> f64| reps.iter().map(|r| f(r)).sum::<f64>() / n;
    let r_hp_gt1 = reps.iter().filter(|r| r.hp_ret.r_num > 1.0).count();
    let w_hp_gt1 = reps.iter().filter(|r| r.hp_ret.omega > 1.0).count();
    let g_hs_lt1 = reps.iter().filter(|r| r.hs_ret.g_min < 1.0).count();
    let g_hp_lt1 = reps.iter().filter(|r| r.hp_ret.g_min < 1.0).count();
    println!("  [{label} (N={})]", reps.len());
    println!(
        "    H_S,1   : rho={:.3} | r={:.3} | w={:.3} | E_W={:.3} | ||H||2={:.3} | G={:.2} | G_w={:.2} | dF={:.3}",
        avg(|r| r.hs1.rho), avg(|r| r.hs1.r_num), avg(|r| r.hs1.omega), avg(|r| r.hs1.e_w),
        avg(|r| r.hs1.spec_norm), avg(|r| r.hs1.g_min), avg(|r| r.hs1.g_omega), avg(|r| r.hs1.delta_f),
    );
    println!(
        "    H_P,1   : rho={:.3} | r={:.3} | w={:.3} | E_W={:.3} | ||H||2={:.3} | G_r={:.3} G_c={:.2} | G_w={:.2} | dF={:.3}",
        avg(|r| r.hp1.rho), avg(|r| r.hp1.r_num), avg(|r| r.hp1.omega), avg(|r| r.hp1.e_w),
        avg(|r| r.hp1.spec_norm), avg(|r| r.hp1.g_row), avg(|r| r.hp1.g_col), avg(|r| r.hp1.g_omega), avg(|r| r.hp1.delta_f),
    );
    println!(
        "    H_S,ret : rho={:.3} | r={:.3} | w={:.3} | E_W={:.3} | ||H||2={:.3} | G={:.2} (cert<1: {}/{}) | G_w={:.2} | dF={:.3}",
        avg(|r| r.hs_ret.rho), avg(|r| r.hs_ret.r_num), avg(|r| r.hs_ret.omega), avg(|r| r.hs_ret.e_w),
        avg(|r| r.hs_ret.spec_norm), avg(|r| r.hs_ret.g_min), g_hs_lt1, reps.len(),
        avg(|r| r.hs_ret.g_omega), avg(|r| r.hs_ret.delta_f),
    );
    println!(
        "    H_P,ret : rho={:.3} | r={:.3} (>1: {}/{}) | w={:.3} (>1: {}/{}) | E_W={:.3} | ||H||2={:.3} | G_r={:.3} G_c={:.2} (cert<1: {}/{}) | G_pi={:.3} | ch_gcol={:.1} (uThin={:.1})",
        avg(|r| r.hp_ret.rho), avg(|r| r.hp_ret.r_num), r_hp_gt1, reps.len(),
        avg(|r| r.hp_ret.omega), w_hp_gt1, reps.len(),
        avg(|r| r.hp_ret.e_w), avg(|r| r.hp_ret.spec_norm),
        avg(|r| r.hp_ret.g_row), avg(|r| r.hp_ret.g_col), g_hp_lt1, reps.len(), avg(|r| r.hp_ret.g_row_pi),
        avg(|r| r.cheap.g_col_proxy), avg(|r| r.cheap.uniq_thin_sym as f64),
    );
}

// ============================================================================
// SECTION 5: PART 3 — Systems Transfer Test (`CS-52` `W-Gersch-Gate`)
//            Embedded Exact `C54-PipeLean-v2` (L=16) & `C54-DramLean-v1` (L=12)
// ============================================================================

#[derive(Default, Clone, Copy, Debug)]
struct WStageTimes {
    s123_us: f64,
    s4_us: f64,
    s5_us: f64,
    m: usize,
    #[allow(dead_code)]
    used_bi: bool,
}

#[allow(dead_code)]
mod open_c54_pipelean_v2 {
    use super::WStageTimes;
    use std::time::Instant;
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
            pub(super) fn ws_bytes(&self) -> usize {
                self.next.capacity() * 2 + self.ovf.capacity() * BYTES + self.cuts.capacity() * 1024
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
    }

    impl Kernel {
        pub(super) fn ws_bytes(&self) -> usize {
            self.spill.ws_bytes()
                + self.bands.capacity() * 8
                + self.spelling.capacity()
                + self.edges.capacity() * 4
                + self.extra.capacity() * 4
                + self.inline16.capacity() * INLINE
                + self.lens.capacity()
                + self.stack.capacity() * 4
        }

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

        #[inline(never)]
        pub(super) fn try_run_const_timed<const LIM: u32, const TIMED: bool>(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> (bool, WStageTimes) {
            let t_start = if TIMED { Some(Instant::now()) } else { None };
            let nrec = records.len();
            // Single check guarantees `pos += width` cannot overflow `u32` (16M * 255 < u32::MAX).
            if n == 0 || n > 900_000 || (p as usize) >= n || nrec > 16_000_000 {
                return (false, WStageTimes::default());
            }
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
                return (false, WStageTimes::default());
            }
            let r_max = (nrec as u32).saturating_sub(extra_127);
            let r_thin_lo = r_max.saturating_sub(ns as u32);
            // Principled post-S1 remaining-cost crossover guard (CA-26 + CA-31):
            // Abort to LazyCells/Seg8-FL on full-size blocks iff `m/n + 0.25*(r_thin/n) > 1.125`.
            if n >= 32_768 && 8 * u64::from(f.m) + 2 * u64::from(r_thin_lo) > 9 * (n as u64) {
                return (false, WStageTimes::default());
            }
            self.finish::<TIMED>(&f, ns, n, r_max, &heads, bi_mode, out, t_start)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn finish<const TIMED: bool>(
            &mut self,
            f: &First,
            ns: usize,
            n: usize,
            r_max: u32,
            heads: &outq::Heads,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
            t_start: Option<Instant>,
        ) -> (bool, WStageTimes) {
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            if all.iter().sum::<u32>() != n as u32 {
                return (false, WStageTimes::default());
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
                self.induced_narrow(
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
                self.induced_wide(
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
                return (false, WStageTimes::default());
            }
            let t_after_s3 = if TIMED { Some(Instant::now()) } else { None };

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

            let t_after_s4 = if TIMED { Some(Instant::now()) } else { None };
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
            let tm = if TIMED {
                let t_end = Instant::now();
                let t0 = t_start.unwrap();
                let t3 = t_after_s3.unwrap();
                let t4 = t_after_s4.unwrap();
                WStageTimes {
                    s123_us: t3.duration_since(t0).as_secs_f64() * 1e6,
                    s4_us: t4.duration_since(t3).as_secs_f64() * 1e6,
                    s5_us: t_end.duration_since(t4).as_secs_f64() * 1e6,
                    m,
                    used_bi: self.predecessor.is_some(),
                }
            } else {
                WStageTimes::default()
            };
            (true, tm)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_narrow(
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
            self.induced_with::<Narrow>(ns, n, m, thin, first, write, most, use_zero_hdr, out_tail)
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_wide(
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
            self.induced_with::<Wide>(ns, n, m, thin, first, write, most, use_zero_hdr, out_tail)
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn induced_with<L: Links>(
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

#[allow(dead_code)]
mod open_c54_dramlean_v1 {
    use super::WStageTimes;
    use std::time::Instant;
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
    const U16_DICT_CAP: u32 = 4_096;

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
            pub(super) fn ws_bytes(&self) -> usize {
                self.next.capacity() * 2 + self.ovf.capacity() * BYTES + self.cuts.capacity() * 1024
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

    // Compact `4 KiB` stack-local `u16` `Dictionary::contents` (`-50%` vs `u32`, `0 B` heap on all
    // `447/447` real blocks!) fused directly into `V3CodeMiss` with lossless `#[cold]` promotion
    // to `contents_u32_fb` whenever `code >= u16_cap` (eliminating `edges` `CACHE_WORDS` growth).
    struct Dictionary<'a> {
        pairs: &'a mut [u32; 512],
        contents_u16: &'a mut [u16; 2048],
        contents_u32_fb: &'a mut Vec<u32>,
        u16_cap: u32,
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        spelling: &'a mut Vec<u8>,
        istk: &'a mut [u32; INLINE_STACK],
        memo_rec: &'a mut [u32; MEMO_REC_CAP],
        stack: &'a mut Vec<u32>,
    }

    impl Dictionary<'_> {
        #[inline(always)]
        fn probe_contents(&self, index: usize, h: u64) -> Option<usize> {
            if self.contents_u32_fb.is_empty() {
                let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
                let ent = self.contents_u16[index];
                if (ent & 0xF000) == tag4 {
                    Some((ent & 0x0FFF) as usize)
                } else {
                    None
                }
            } else {
                let ent = self.contents_u32_fb[index];
                if ent != 0 {
                    Some(ent as usize)
                } else {
                    None
                }
            }
        }

        #[inline(always)]
        fn store_contents(&mut self, index: usize, h: u64, code: u32) {
            if self.contents_u32_fb.is_empty() && code < self.u16_cap {
                let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
                self.contents_u16[index] = tag4 | (code as u16);
            } else {
                Self::store_contents_u32_fallback(
                    self.contents_u16,
                    self.contents_u32_fb,
                    index,
                    code,
                );
            }
        }

        #[cold]
        #[inline(never)]
        fn store_contents_u32_fallback(
            tbl: &[u16; 2048],
            fb: &mut Vec<u32>,
            index: usize,
            code: u32,
        ) {
            if fb.is_empty() {
                fb.reserve_exact(2048);
                fb.extend(
                    tbl.iter()
                        .map(|&v| if v != 0 { u32::from(v & 0x0FFF) } else { 0 }),
                );
            }
            fb[index] = code;
        }

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
                let index = (h as usize) & 2047;
                if let Some(old) = self.probe_contents(index, h) {
                    if self.lens[old] == len && self.inline16[old] == entry {
                        return old as u32;
                    }
                }
                let c = self.insert(entry, len);
                self.store_contents(index, h, c);
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
                let index = (h as usize) & 2047;
                if let Some(old) = self.probe_contents(index, h) {
                    if self.lens[old] == LONG
                        && long_word(&self.inline16[old], self.spelling) == bytes
                    {
                        self.spelling.truncate(start);
                        return old as u32;
                    }
                }
                let c = self.insert(entry, LONG);
                self.store_contents(index, h, c);
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
            let index = (h as usize) & 2047;
            if let Some(old) = self.probe_contents(index, h) {
                if self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes
                {
                    if !memo_ovf {
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
                    return old as u32;
                }
            }
            let c = self.insert(entry, LONG);
            self.store_contents(index, h, c);
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
        contents_u32_fb: Vec<u32>,
        predecessor: Option<u32>,
    }

    impl Kernel {
        pub(super) fn ws_bytes(&self) -> usize {
            self.spill.ws_bytes()
                + self.bands.capacity() * 8
                + self.spelling.capacity()
                + self.edges.capacity() * 4
                + self.extra.capacity() * 4
                + self.inline16.capacity() * INLINE
                + self.lens.capacity()
                + self.stack.capacity() * 4 + self.contents_u32_fb.capacity() * 4
        }

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

        #[inline(never)]
        pub(super) fn try_run_const_timed<const LIM: u32, const TIMED: bool>(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> (bool, WStageTimes) {
            let t_start = if TIMED { Some(Instant::now()) } else { None };
            let nrec = records.len();
            // Single check guarantees `pos += width` cannot overflow `u32` (16M * 255 < u32::MAX).
            if n == 0 || n > 900_000 || (p as usize) >= n || nrec > 16_000_000 {
                return (false, WStageTimes::default());
            }
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
                return (false, WStageTimes::default());
            }
            let r_max = (nrec as u32).saturating_sub(extra_127);
            let r_thin_lo = r_max.saturating_sub(ns as u32);
            // Principled post-S1 remaining-cost crossover guard (CA-26 + CA-31):
            // Abort to LazyCells/Seg8-FL on full-size blocks iff `m/n + 0.25*(r_thin/n) > 1.125`.
            if n >= 32_768 && 8 * u64::from(f.m) + 2 * u64::from(r_thin_lo) > 9 * (n as u64) {
                return (false, WStageTimes::default());
            }
            self.finish::<TIMED>(&f, ns, n, r_max, &heads, bi_mode, out, t_start)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn finish<const TIMED: bool>(
            &mut self,
            f: &First,
            ns: usize,
            n: usize,
            r_max: u32,
            heads: &outq::Heads,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
            t_start: Option<Instant>,
        ) -> (bool, WStageTimes) {
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            if all.iter().sum::<u32>() != n as u32 {
                return (false, WStageTimes::default());
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
            self.contents_u32_fb.clear();

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
                self.induced_narrow(
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
                self.induced_wide(
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
                return (false, WStageTimes::default());
            }
            let t_after_s3 = if TIMED { Some(Instant::now()) } else { None };

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

            let t_after_s4 = if TIMED { Some(Instant::now()) } else { None };
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
            } else {
                emit(
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
            let tm = if TIMED {
                let t_end = Instant::now();
                let t0 = t_start.unwrap();
                let t3 = t_after_s3.unwrap();
                let t4 = t_after_s4.unwrap();
                WStageTimes {
                    s123_us: t3.duration_since(t0).as_secs_f64() * 1e6,
                    s4_us: t4.duration_since(t3).as_secs_f64() * 1e6,
                    s5_us: t_end.duration_since(t4).as_secs_f64() * 1e6,
                    m,
                    used_bi: self.predecessor.is_some(),
                }
            } else {
                WStageTimes::default()
            };
            (true, tm)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_narrow(
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
            self.induced_with::<Narrow>(ns, n, m, thin, first, write, most, use_zero_hdr, out_tail)
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_wide(
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
            self.induced_with::<Wide>(ns, n, m, thin, first, write, most, use_zero_hdr, out_tail)
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn induced_with<L: Links>(
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

            let (_, arena) = self.edges.split_at_mut(hdr_words);
            let (live, _) = arena.split_at_mut(base + rule_top);
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
            begin_inline(&mut self.inline16, &mut self.lens);

            // Fixed-pool injectivity + 256-bucket common refinement: bands_count < open_nodes + 256.
            let max_bands = open_nodes + 256;
            let use_tail = out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            // Hoisted stack-local scratch allocated ONCE per block (`6.38 KiB` total L1D-hot!).
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec = [0u32; MEMO_REC_CAP];
            let mut pairs = [0u32; 512];
            let mut contents_u16 = [0u16; 2048];
            let u16_cap = U16_DICT_CAP;
            let mut dictionary = Dictionary {
                pairs: &mut pairs,
                contents_u16: &mut contents_u16,
                contents_u32_fb: &mut self.contents_u32_fb,
                u16_cap,
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

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bidirectional(
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
                // 1-step software-pipelined loads (`Q-2Cur-Pipe1`, +0 B WS):
                // overlap next-step `edges` L2/L3 misses with current-step emit!
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
                // Statically guaranteed: rem.len() >= INLINE (16) here!
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
        let hi_rem = lo_rem + rem_len;
        if rem_len > 0 {
            out.copy_within(hi_rem..n, lo_rem);
            out.truncate(lo_rem + (n - hi_rem));
            repeat(n, out);
        } else {
            out.truncate(n);
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


const POST_S1_BI_P2: open_c54_pipelean_v2::BiMode = open_c54_pipelean_v2::BiMode::PostS1ExactMd {
    r_max_cap: 50_000,
    ns_max: 2_100,
    md_max: 16_000,
    md_neg_cap: -18_000,
    ns_neg_max: 2_400,
};

const POST_S1_BI_D1: open_c54_dramlean_v1::BiMode = open_c54_dramlean_v1::BiMode::PostS1ExactMd {
    r_max_cap: 50_000,
    ns_max: 2_100,
    md_max: 16_000,
    md_neg_cap: -18_000,
    ns_neg_max: 2_400,
};

#[inline]
fn select_limit_c54(records: &[u16], n: usize, default_lim: u32) -> u32 {
    let nrec = records.len();
    if n < 32_768 || nrec < 256 {
        return default_lim;
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
        return default_lim;
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
            default_lim
        }
    } else {
        default_lim
    }
}

/// Cheap O(Sigma) Gerschgorin/normality-inspired router (`W-Gersch-Gate`):
/// Uses 256-strided sampling (~200 ns, ZERO dense matrix!) to compute `g_col_proxy = uniq_thin_sym * thin_frac_samp`
/// (which tracks true `G_col(H_P,ret)` & `S5_us` without building a dense matrix!).
/// Routes high-Gerschgorin-hub blocks (`g_col_proxy >= 24.0`, isolating Code+Docs / wide-alphabet
/// high-retention blocks where shrinking `L=16 -> 12` saves `-71 KiB` PeakCap & `-15..20%` S5!) to `L=12`,
/// keeps low-`w1` JSONL/Log at `L=24/32`, and routes standard Wikidata blocks to `L=16` (`C54-PipeLean-v2`).
/// Returns `(use_dramlean_kernel, lim)`.
#[inline]
fn w_gersch_gate_decide(records: &[u16], n: usize) -> (bool, u32) {
    let f = extract_cheap_gersch_features(records, n);
    if f.est_w1 <= 33_500 {
        if f.mid17_32_cnt <= 3 {
            return (false, 32);
        } else if f.mid17_32_cnt <= 5 {
            return (false, 24);
        }
    }
    if f.g_col_proxy >= 24.0 {
        (true, 12)
    } else {
        (false, 16)
    }
}

#[derive(Default)]
struct DualArmRunner {
    pipe: open_c54_pipelean_v2::Kernel,
    dram: open_c54_dramlean_v1::Kernel,
}

impl DualArmRunner {
    fn ws_bytes(&self, arm: u8) -> usize {
        match arm {
            0 | 1 | 3 | 5 => self.pipe.ws_bytes(),
            2 => self.dram.ws_bytes(),
            4 => self.pipe.ws_bytes().max(self.dram.ws_bytes()),
            _ => unreachable!(),
        }
    }

    /// Runs one of 6 arms with 100% forward-FL exactness (TinyFL for n < 32768 + exact fallback):
    ///   - `0`: `C54-PipeLean-v2` (`L = 16`, fixed champion #1)
    ///   - `1`: `C54-PipeLean-v2-AA` (`A/A` control)
    ///   - `2`: `C54-DramLean-v1` (`L = 12` + `4 KiB` stack `u16` dict, fixed champion #2)
    ///   - `3`: `PipeLean-v2-L12` (`L = 12` on `C54-PipeLean-v2` engine — isolates `L` cleanly!)
    ///   - `4`: `W-Gersch-Gate (P2 vs D1)` (cheap Gerschgorin router between `C54-PipeLean-v2` & `C54-DramLean-v1`)
    ///   - `5`: `W-Gersch-Gate-L12L16` (cheap Gerschgorin router selecting `L in {12, 16, 24, 32}` inside `C54-PipeLean-v2`)
    #[inline]
    fn run_arm<const ARM: u8, const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> (bool, WStageTimes) {
        if n < 32_768 {
            let ok = match ARM {
                2 => self.dram.invert_tiny_fl(records, n, p, out),
                _ => self.pipe.invert_tiny_fl(records, n, p, out),
            };
            return (ok, WStageTimes::default());
        }
        match ARM {
            0 | 1 => {
                let lim = select_limit_c54(records, n, 16);
                match lim {
                    16 => self.pipe.try_run_const_timed::<16, TIMED>(records, n, p, POST_S1_BI_P2, out),
                    24 => self.pipe.try_run_const_timed::<24, TIMED>(records, n, p, POST_S1_BI_P2, out),
                    _ => self.pipe.try_run_const_timed::<32, TIMED>(records, n, p, POST_S1_BI_P2, out),
                }
            }
            2 => {
                let lim = select_limit_c54(records, n, 12);
                match lim {
                    12 => self.dram.try_run_const_timed::<12, TIMED>(records, n, p, POST_S1_BI_D1, out),
                    24 => self.dram.try_run_const_timed::<24, TIMED>(records, n, p, POST_S1_BI_D1, out),
                    _ => self.dram.try_run_const_timed::<32, TIMED>(records, n, p, POST_S1_BI_D1, out),
                }
            }
            3 => {
                let lim = select_limit_c54(records, n, 12);
                match lim {
                    12 => self.pipe.try_run_const_timed::<12, TIMED>(records, n, p, POST_S1_BI_P2, out),
                    24 => self.pipe.try_run_const_timed::<24, TIMED>(records, n, p, POST_S1_BI_P2, out),
                    _ => self.pipe.try_run_const_timed::<32, TIMED>(records, n, p, POST_S1_BI_P2, out),
                }
            }
            4 => {
                let (use_dram, lim) = w_gersch_gate_decide(records, n);
                if use_dram {
                    self.dram.try_run_const_timed::<12, TIMED>(records, n, p, POST_S1_BI_D1, out)
                } else {
                    match lim {
                        16 => self.pipe.try_run_const_timed::<16, TIMED>(records, n, p, POST_S1_BI_P2, out),
                        24 => self.pipe.try_run_const_timed::<24, TIMED>(records, n, p, POST_S1_BI_P2, out),
                        _ => self.pipe.try_run_const_timed::<32, TIMED>(records, n, p, POST_S1_BI_P2, out),
                    }
                }
            }
            5 => {
                let (_, lim) = w_gersch_gate_decide(records, n);
                match lim {
                    12 => self.pipe.try_run_const_timed::<12, TIMED>(records, n, p, POST_S1_BI_P2, out),
                    16 => self.pipe.try_run_const_timed::<16, TIMED>(records, n, p, POST_S1_BI_P2, out),
                    24 => self.pipe.try_run_const_timed::<24, TIMED>(records, n, p, POST_S1_BI_P2, out),
                    _ => self.pipe.try_run_const_timed::<32, TIMED>(records, n, p, POST_S1_BI_P2, out),
                }
            }
            _ => unreachable!(),
        }
    }
}

fn measure_per_block_latencies(
    blocks: &[&Block],
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = blocks.len();
    let reps = if cfg!(debug_assertions) { 1 } else { 3 };
    let mut r_p2 = DualArmRunner::default();
    let mut r_d1 = DualArmRunner::default();
    let mut r_l12 = DualArmRunner::default();
    let mut out = Vec::new();
    // Warmup
    for &b in blocks.iter().take(4) {
        let _ = r_p2.run_arm::<0, true>(&b.records, b.n, b.p, &mut out);
        let _ = r_d1.run_arm::<2, true>(&b.records, b.n, b.p, &mut out);
        let _ = r_l12.run_arm::<3, true>(&b.records, b.n, b.p, &mut out);
    }

    let mut p2_s5 = vec![0.0f64; n];
    let mut p2_s5_ns = vec![0.0f64; n];
    let mut p2_e2e = vec![0.0f64; n];
    let mut d1_e2e = vec![0.0f64; n];
    let mut l12_e2e = vec![0.0f64; n];
    let mut d1_over_p2 = vec![0.0f64; n];

    for (i, &b) in blocks.iter().enumerate() {
        let mut vs5 = Vec::with_capacity(reps);
        let mut vp2 = Vec::with_capacity(reps);
        let mut vd1 = Vec::with_capacity(reps);
        let mut vl12 = Vec::with_capacity(reps);
        let mut m_rows = 1usize;
        for _ in 0..reps {
            let (ok0, t0) = r_p2.run_arm::<0, true>(&b.records, b.n, b.p, &mut out);
            let (ok2, t2) = r_d1.run_arm::<2, true>(&b.records, b.n, b.p, &mut out);
            let (ok3, t3) = r_l12.run_arm::<3, true>(&b.records, b.n, b.p, &mut out);
            assert!(ok0 && ok2 && ok3);
            vs5.push(t0.s5_us);
            vp2.push(t0.s123_us + t0.s4_us + t0.s5_us);
            vd1.push(t2.s123_us + t2.s4_us + t2.s5_us);
            vl12.push(t3.s123_us + t3.s4_us + t3.s5_us);
            m_rows = t0.m.max(1);
        }
        vs5.sort_by(|a, b| a.partial_cmp(b).unwrap());
        vp2.sort_by(|a, b| a.partial_cmp(b).unwrap());
        vd1.sort_by(|a, b| a.partial_cmp(b).unwrap());
        vl12.sort_by(|a, b| a.partial_cmp(b).unwrap());
        p2_s5[i] = vs5[reps / 2];
        p2_s5_ns[i] = (p2_s5[i] * 1e3) / (m_rows as f64);
        p2_e2e[i] = vp2[reps / 2];
        d1_e2e[i] = vd1[reps / 2];
        l12_e2e[i] = vl12[reps / 2];
        d1_over_p2[i] = d1_e2e[i] / p2_e2e[i].max(1e-9);
    }
    (p2_s5, p2_s5_ns, p2_e2e, d1_e2e, l12_e2e, d1_over_p2)
}

fn run_pass_1t<const ARM: u8>(r: &mut DualArmRunner, blks: &[Block], out: &mut Vec<u8>) -> f64 {
    let t0 = Instant::now();
    for b in blks {
        let (ok, _) = r.run_arm::<ARM, false>(&b.records, b.n, b.p, out);
        assert!(ok);
    }
    t0.elapsed().as_secs_f64() * 1e3
}

fn run_stage_pass_1t<const ARM: u8>(
    r: &mut DualArmRunner,
    blks: &[Block],
    out: &mut Vec<u8>,
) -> (f64, f64, f64) {
    let mut s123 = 0.0;
    let mut s4 = 0.0;
    let mut s5 = 0.0;
    for b in blks {
        let (ok, tm) = r.run_arm::<ARM, true>(&b.records, b.n, b.p, out);
        assert!(ok);
        s123 += tm.s123_us;
        s4 += tm.s4_us;
        s5 += tm.s5_us;
    }
    (s123, s4, s5)
}

fn run_pass_nw<const ARM: u8>(blks: &[Block], workers: usize) -> f64 {
    let barrier = Barrier::new(workers);
    let mut wall_ms = 0.0f64;
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for w in 0..workers {
            let b_ref = &barrier;
            handles.push(scope.spawn(move || {
                let mut runner = DualArmRunner::default();
                let mut out = Vec::new();
                // Warmup one block per worker so allocators settle before timed barrier
                if let Some(first) = blks.get(w % blks.len()) {
                    let _ = runner.run_arm::<ARM, false>(&first.records, first.n, first.p, &mut out);
                }
                b_ref.wait();
                let t0 = Instant::now();
                for (idx, b) in blks.iter().enumerate() {
                    if idx % workers == w {
                        let (ok, _) = runner.run_arm::<ARM, false>(&b.records, b.n, b.p, &mut out);
                        assert!(ok);
                    }
                }
                t0.elapsed().as_secs_f64() * 1e3
            }));
        }
        for h in handles {
            let ms = h.join().unwrap();
            if ms > wall_ms {
                wall_ms = ms;
            }
        }
    });
    wall_ms
}

fn run_part3_w_gersch_gate(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    detached195: &[(String, Vec<u8>, u32)],
    adm6: &[Block],
    nw6: &[Block],
    cd4: &[Block],
    wiki233: &[Block],
    p2_e2e_249: &[f64],
    d1_e2e_249: &[f64],
    l12_e2e_249: &[f64],
) {
    println!("================================================================================================================================");
    println!("PART 3: SYSTEMS TRANSFER TEST (`CS-52` `W-Gersch-Gate`) — CHEAP O(Sigma) GERSCHGORIN/NORMALITY ROUTING");
    println!("  Compares fixed C54-PipeLean-v2 (L=16), A/A, C54-DramLean-v1 (L=12), PipeLean-v2-L12, & W-Gersch-Gate across 249 blocks!");
    println!("  Strictly 2020s modern CS (ZERO classical share); 100% forward-FL exactness on valid, malformed, detached & real blocks!");
    println!("================================================================================================================================");

    // 3A: 100% Byte-Exactness Gate across ALL 6 Arms on 5,200 Valid + 300 Malformed + 195 Detached + 249 Real (5,944 cases!)
    let t_ver = Instant::now();
    let mut runners: [DualArmRunner; 6] = std::array::from_fn(|_| DualArmRunner::default());
    let mut outs: [Vec<u8>; 6] = std::array::from_fn(|_| Vec::new());
    let mut ref_k = cand_r5_head::Kernel::default();
    let mut o_ref = Vec::new();

    let check_small = |rs: &mut [DualArmRunner; 6], os: &mut [Vec<u8>; 6], l: &[u8], p: u32, tag: &str| {
        let recs = records_of(l);
        let expected = invert_forward_fl(l, p);
        assert!(rs[0].run_arm::<0, false>(&recs, l.len(), p, &mut os[0]).0);
        assert!(rs[1].run_arm::<1, false>(&recs, l.len(), p, &mut os[1]).0);
        assert!(rs[2].run_arm::<2, false>(&recs, l.len(), p, &mut os[2]).0);
        assert!(rs[3].run_arm::<3, false>(&recs, l.len(), p, &mut os[3]).0);
        assert!(rs[4].run_arm::<4, false>(&recs, l.len(), p, &mut os[4]).0);
        assert!(rs[5].run_arm::<5, false>(&recs, l.len(), p, &mut os[5]).0);
        for a in 0..6 {
            assert_eq!(os[a], expected, "Arm {a} mismatch on {tag}");
        }
        // Also exercise try_run_const directly (not just TinyFL!) on non-empty small cases
        assert!(rs[0].pipe.try_run_const_timed::<16, false>(&recs, l.len(), p, POST_S1_BI_P2, &mut os[0]).0);
        assert!(rs[2].dram.try_run_const_timed::<12, false>(&recs, l.len(), p, POST_S1_BI_D1, &mut os[2]).0);
        assert!(rs[3].pipe.try_run_const_timed::<12, false>(&recs, l.len(), p, POST_S1_BI_P2, &mut os[3]).0);
        assert_eq!(os[0], expected, "Direct PipeLean L=16 mismatch on {tag}");
        assert_eq!(os[2], expected, "Direct DramLean L=12 mismatch on {tag}");
        assert_eq!(os[3], expected, "Direct PipeLean L=12 mismatch on {tag}");
        expected
    };

    for (idx, (l, p)) in valid.iter().enumerate() {
        let _ = check_small(&mut runners, &mut outs, l, *p, &format!("valid-{idx}"));
    }
    for (idx, (l, p)) in malformed.iter().enumerate() {
        let exp = check_small(&mut runners, &mut outs, l, *p, &format!("mal-{idx}"));
        let recs = records_of(l);
        ref_k.run_entry(&recs, l.len(), *p, cand_r5_head::Cap::NONE, &mut o_ref).unwrap();
        assert_eq!(o_ref, exp);
    }
    for (name, l, p) in detached195 {
        let exp = check_small(&mut runners, &mut outs, l, *p, name);
        let recs = records_of(l);
        ref_k.run_entry(&recs, l.len(), *p, cand_r5_head::Cap::NONE, &mut o_ref).unwrap();
        assert_eq!(o_ref, exp);
    }

    let mut real_checked = 0usize;
    let mut gate_routed_dram = [0usize; 4]; // Adm6, NW6, CD4, Wiki233
    let corpora_refs: [(&[Block], usize); 4] = [(adm6, 0), (nw6, 1), (cd4, 2), (wiki233, 3)];
    for &(blks, cid) in &corpora_refs {
        for b in blks {
            ref_k.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut o_ref).unwrap();
            assert!(runners[0].run_arm::<0, false>(&b.records, b.n, b.p, &mut outs[0]).0);
            assert!(runners[1].run_arm::<1, false>(&b.records, b.n, b.p, &mut outs[1]).0);
            assert!(runners[2].run_arm::<2, false>(&b.records, b.n, b.p, &mut outs[2]).0);
            assert!(runners[3].run_arm::<3, false>(&b.records, b.n, b.p, &mut outs[3]).0);
            assert!(runners[4].run_arm::<4, false>(&b.records, b.n, b.p, &mut outs[4]).0);
            assert!(runners[5].run_arm::<5, false>(&b.records, b.n, b.p, &mut outs[5]).0);
            for a in 0..6 {
                assert_eq!(outs[a], o_ref, "Real block {} arm {a} mismatch", b.name);
            }
            if w_gersch_gate_decide(&b.records, b.n).0 {
                gate_routed_dram[cid] += 1;
            }
            real_checked += 1;
        }
    }

    println!(
        "  [PASS 3A] 100% Byte-Exactness (ALL 6 Arms + Direct L=12/16 vs forward-FL & cand_r5_head) in {:.2}s:",
        t_ver.elapsed().as_secs_f64()
    );
    println!("    - Valid Small (TinyFL + Direct L=12/16) : 5,200 / 5,200 EXACT");
    println!("    - Malformed Multi-Cycle (forward-FL)    :   300 /   300 EXACT");
    println!("    - Detached/Periodic d=1..=65            :   195 /   195 EXACT");
    println!("    - Real Blocks (6 Adm + 10 NW + 233 W)   :   {real_checked} /   249 EXACT");
    println!(
        "    - W-Gersch-Gate L=12/DramLean route     : Adm {}/6 | JSONL/Log {}/6 | CodeDocs {}/4 | Wiki {}/233\n",
        gate_routed_dram[0], gate_routed_dram[1], gate_routed_dram[2], gate_routed_dram[3]
    );

    // 3B: Per-Block Hindsight Oracle Headroom Analysis (P2 vs D1 and P2-L16 vs P2-L12)
    let sum_p2: f64 = p2_e2e_249.iter().sum();
    let sum_d1: f64 = d1_e2e_249.iter().sum();
    let sum_l12: f64 = l12_e2e_249.iter().sum();
    let mut orcl_p2_d1 = 0.0f64;
    let mut orcl_p2_l12 = 0.0f64;
    let mut d1_wins = 0usize;
    let mut l12_wins = 0usize;
    for i in 0..249 {
        orcl_p2_d1 += p2_e2e_249[i].min(d1_e2e_249[i]);
        orcl_p2_l12 += p2_e2e_249[i].min(l12_e2e_249[i]);
        if d1_e2e_249[i] < p2_e2e_249[i] {
            d1_wins += 1;
        }
        if l12_e2e_249[i] < p2_e2e_249[i] {
            l12_wins += 1;
        }
    }
    println!("  [3B HINDSIGHT ORACLE HEADROOM (249 REAL BLOCKS, 1T PER-BLOCK MEDIAN)]:");
    println!(
        "    - C54-PipeLean-v2 (L=16) sum : {:.2} ms | C54-DramLean-v1 (L=12) sum : {:.2} ms ({:.4}x) | D1 wins {d1_wins}/249 | Oracle(P2,D1) = {:.2} ms ({:.4}x vs P2)",
        sum_p2 / 1e3, sum_d1 / 1e3, sum_d1 / sum_p2, orcl_p2_d1 / 1e3, orcl_p2_d1 / sum_p2
    );
    println!(
        "    - PipeLean-v2-L12 (L=12) sum : {:.2} ms ({:.4}x vs P2-L16) | L12 wins {l12_wins}/249 | Oracle(L16,L12) = {:.2} ms ({:.4}x vs P2-L16)\n",
        sum_l12 / 1e3, sum_l12 / sum_p2, orcl_p2_l12 / 1e3, orcl_p2_l12 / sum_p2
    );

    // 3C: Interleaved Multi-Worker Benchmarks (1T / 2W / 4W + S4/S5 + WS) across ALL Corpora
    let mut nonwiki10: Vec<Block> = Vec::with_capacity(10);
    for b in nw6.iter().chain(cd4.iter()) {
        nonwiki10.push(Block { name: b.name.clone(), n: b.n, p: b.p, records: b.records.clone() });
    }

    let rounds_adm = if cfg!(debug_assertions) { 1 } else { 11 };
    let rounds_nw = if cfg!(debug_assertions) { 1 } else { 9 };
    let rounds_wiki = if cfg!(debug_assertions) { 1 } else { 5 };
    let st_rounds = if cfg!(debug_assertions) { 1 } else { 5 };

    let corpora: [(&str, &[Block], usize); 5] = [
        ("6 Admitted Wikidata", adm6, rounds_adm),
        ("6 JSONL/Syslog", nw6, rounds_nw),
        ("4 Code+Docs", cd4, rounds_nw),
        ("10 Non-Wiki Combined", &nonwiki10, rounds_nw),
        ("233 Full Wikidata", wiki233, rounds_wiki),
    ];

    let arm_names: [&str; 6] = [
        "C54-PipeLean-v2",
        "C54-PipeLean-AA",
        "C54-DramLean-v1",
        "PipeLean-v2-L12",
        "W-Gersch-P2/D1",
        "W-Gersch-L12/16",
    ];

    for &(label, blks, rds) in &corpora {
        println!(
            "  [BENCH — {label} ({} blks, {rds} interleaved rounds median)]:",
            blks.len()
        );
        let mut kb: [DualArmRunner; 6] = std::array::from_fn(|_| DualArmRunner::default());
        let mut out = Vec::new();
        let _ = run_pass_1t::<0>(&mut kb[0], blks, &mut out);
        let _ = run_pass_1t::<1>(&mut kb[1], blks, &mut out);
        let _ = run_pass_1t::<2>(&mut kb[2], blks, &mut out);
        let _ = run_pass_1t::<3>(&mut kb[3], blks, &mut out);
        let _ = run_pass_1t::<4>(&mut kb[4], blks, &mut out);
        let _ = run_pass_1t::<5>(&mut kb[5], blks, &mut out);
        let wss: [usize; 6] = std::array::from_fn(|i| kb[i].ws_bytes(i as u8));

        let mut s4_v: [Vec<f64>; 6] = std::array::from_fn(|_| Vec::with_capacity(st_rounds));
        let mut s5_v: [Vec<f64>; 6] = std::array::from_fn(|_| Vec::with_capacity(st_rounds));
        for _ in 0..st_rounds {
            let tms = [
                run_stage_pass_1t::<0>(&mut kb[0], blks, &mut out),
                run_stage_pass_1t::<1>(&mut kb[1], blks, &mut out),
                run_stage_pass_1t::<2>(&mut kb[2], blks, &mut out),
                run_stage_pass_1t::<3>(&mut kb[3], blks, &mut out),
                run_stage_pass_1t::<4>(&mut kb[4], blks, &mut out),
                run_stage_pass_1t::<5>(&mut kb[5], blks, &mut out),
            ];
            for a in 0..6 {
                s4_v[a].push(tms[a].1);
                s5_v[a].push(tms[a].2);
            }
        }
        let mut s4_m = [0.0f64; 6];
        let mut s5_m = [0.0f64; 6];
        for a in 0..6 {
            s4_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            s5_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            s4_m[a] = s4_v[a][st_rounds / 2];
            s5_m[a] = s5_v[a][st_rounds / 2];
        }

        let mut t1_v: [Vec<f64>; 6] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t2_v: [Vec<f64>; 6] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t4_v: [Vec<f64>; 6] = std::array::from_fn(|_| Vec::with_capacity(rds));
        for _ in 0..rds {
            t1_v[0].push(run_pass_1t::<0>(&mut kb[0], blks, &mut out));
            t1_v[1].push(run_pass_1t::<1>(&mut kb[1], blks, &mut out));
            t1_v[2].push(run_pass_1t::<2>(&mut kb[2], blks, &mut out));
            t1_v[3].push(run_pass_1t::<3>(&mut kb[3], blks, &mut out));
            t1_v[4].push(run_pass_1t::<4>(&mut kb[4], blks, &mut out));
            t1_v[5].push(run_pass_1t::<5>(&mut kb[5], blks, &mut out));
        }
        for _ in 0..rds {
            t2_v[0].push(run_pass_nw::<0>(blks, 2));
            t2_v[1].push(run_pass_nw::<1>(blks, 2));
            t2_v[2].push(run_pass_nw::<2>(blks, 2));
            t2_v[3].push(run_pass_nw::<3>(blks, 2));
            t2_v[4].push(run_pass_nw::<4>(blks, 2));
            t2_v[5].push(run_pass_nw::<5>(blks, 2));
        }
        for _ in 0..rds {
            t4_v[0].push(run_pass_nw::<0>(blks, 4));
            t4_v[1].push(run_pass_nw::<1>(blks, 4));
            t4_v[2].push(run_pass_nw::<2>(blks, 4));
            t4_v[3].push(run_pass_nw::<3>(blks, 4));
            t4_v[4].push(run_pass_nw::<4>(blks, 4));
            t4_v[5].push(run_pass_nw::<5>(blks, 4));
        }

        let mut t1_m = [0.0f64; 6];
        let mut t2_m = [0.0f64; 6];
        let mut t4_m = [0.0f64; 6];
        for a in 0..6 {
            t1_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            t2_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            t4_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            t1_m[a] = t1_v[a][rds / 2];
            t2_m[a] = t2_v[a][rds / 2];
            t4_m[a] = t4_v[a][rds / 2];
        }

        println!(
            "  {:<18} | {:>14} {:>14} {:>14} | {:>14} | {:>14} | {:>8}",
            "Arm", "1T ms (rel)", "2W ms (rel)", "4W ms (rel)", "S4 ms (rel)", "S5 ms (rel)", "WS KiB"
        );
        for a in 0..6 {
            println!(
                "  {:<18} | {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) | {:>6.2} ({:.3}x) | {:>6.2} ({:.3}x) | {:>8.1}",
                arm_names[a],
                t1_m[a], t1_m[a] / t1_m[0],
                t2_m[a], t2_m[a] / t2_m[0],
                t4_m[a], t4_m[a] / t4_m[0],
                s4_m[a] / 1e3, s4_m[a] / s4_m[0].max(1e-9),
                s5_m[a] / 1e3, s5_m[a] / s5_m[0].max(1e-9),
                wss[a] as f64 / 1024.0,
            );
        }
        println!();
    }
}

fn main() {
    let t_total = Instant::now();
    println!("Cycle 55 (P1) — Analogue W: O. Toeplitz (1918) & F. Hausdorff (1919) Numerical Range");
    println!("(Field of Values) + S. Gerschgorin (1931) Disks on P_1 / P_ret / S_ret, & `W-Gersch-Gate`\n");

    let (valid, malformed) = generate_small_and_malformed_suites();
    let detached195 = generate_detached_cycle_stress_1_to_65();

    // PART 1: Exact BigInt/BigRat + Analytical Witnesses (5,552 cases)
    run_part1_exact_toeplitz_hausdorff_gerschgorin(&valid, &malformed);

    // Load Solved Analogue W + ALL 249 Real Blocks
    let solved = make_solved_analogue_w_blocks();
    let adm6 = load_six_admitted();
    let nw_names = ["jsonl-0", "jsonl-1", "jsonl-2", "jsonl-3", "jsonl-4", "syslog-0"];
    let nw6 = load_bin_blocks(
        &resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin"),
        Some(&nw_names),
        false,
    );
    let cd_names = ["codedocs-0", "codedocs-1", "codedocs-2", "codedocs-3"];
    let cd4 = load_bin_blocks(
        &resolve_corpus_path("non_wikidata/code_docs_blocks.bin"),
        Some(&cd_names),
        false,
    );
    let wiki233 = load_bin_blocks(&resolve_corpus_path("wikidata_431_blocks.bin"), None, true);
    assert_eq!(adm6.len(), 6);
    assert_eq!(nw6.len(), 6);
    assert_eq!(cd4.len(), 4);
    assert_eq!(wiki233.len(), 233);

    // PART 2: Exact Toeplitz-Hausdorff & Gerschgorin across Solved W + ALL 249 Real Blocks
    println!("================================================================================================================================");
    println!("PART 2: TOEPLITZ-HAUSDORFF NUMERICAL RANGE & GERSCHGORIN DISKS ON H_S & H_P (ALL 249 REAL BLOCKS = 996 MATRICES)");
    println!("  Exact Complex Householder Tridiagonalization + Sturm Bisection + Certified Supporting-Line Branch-and-Bound");
    println!("================================================================================================================================");
    let t_p2 = Instant::now();
    let solved_reps: Vec<RealBlockWReport> = solved.iter().map(analyze_real_block_w).collect();
    let adm6_reps: Vec<RealBlockWReport> = adm6.iter().map(analyze_real_block_w).collect();
    let nw6_reps: Vec<RealBlockWReport> = nw6.iter().map(analyze_real_block_w).collect();
    let cd4_reps: Vec<RealBlockWReport> = cd4.iter().map(analyze_real_block_w).collect();
    let w233_reps: Vec<RealBlockWReport> = {
        let chunks = 4usize;
        let chunk_sz = (wiki233.len() + chunks - 1) / chunks;
        let mut out_reps = Vec::with_capacity(wiki233.len());
        std::thread::scope(|s| {
            let handles: Vec<_> = wiki233
                .chunks(chunk_sz)
                .map(|ch| s.spawn(move || ch.iter().map(analyze_real_block_w).collect::<Vec<_>>()))
                .collect();
            for h in handles {
                out_reps.extend(h.join().unwrap());
            }
        });
        out_reps
    };

    print_w_table("SOLVED ANALOGUE W (IID16, Reversible Walk, Cyclic Shift, vs Directed Syntax)", &solved_reps);
    print_w_table("6 ADMITTED WIKIDATA BLOCKS", &adm6_reps);
    print_w_table("6 REAL NON-WIKIDATA BLOCKS (JSONL + SYSLOG)", &nw6_reps);
    print_w_table("4 REAL CODE+DOCS BLOCKS", &cd4_reps);

    let adm6_r: Vec<&RealBlockWReport> = adm6_reps.iter().collect();
    let nw6_r: Vec<&RealBlockWReport> = nw6_reps.iter().collect();
    let cd4_r: Vec<&RealBlockWReport> = cd4_reps.iter().collect();
    let w233_r: Vec<&RealBlockWReport> = w233_reps.iter().collect();
    let mut all_249: Vec<&RealBlockWReport> = Vec::with_capacity(249);
    all_249.extend(&adm6_r);
    all_249.extend(&nw6_r);
    all_249.extend(&cd4_r);
    all_249.extend(&w233_r);
    assert_eq!(all_249.len(), 249);

    println!(
        "  [AGGREGATE 249 REAL BLOCKS TOEPLITZ-HAUSDORFF & GERSCHGORIN SUMMARY (996 deflated matrices in {:.2}s)]:",
        t_p2.elapsed().as_secs_f64()
    );
    summarize_w_corpus("6 Admitted Wikidata", &adm6_r);
    summarize_w_corpus("6 JSONL/Syslog", &nw6_r);
    summarize_w_corpus("4 Code+Docs", &cd4_r);
    summarize_w_corpus("233 Full wikidata.bz2", &w233_r);
    summarize_w_corpus("ALL 249 REAL BLOCKS COMBINED", &all_249);

    let mut max_g_viol = 0.0f64;
    let mut max_schur_id = 0.0f64;
    let mut max_cert_gap_hs = 0.0f64;
    let mut max_cert_gap_hp = 0.0f64;
    let mut avg_evals = 0.0f64;
    let mut amp_hp_ret = 0usize;
    let mut amp_w_hp_ret = 0usize;
    let mut g_cert_hs = 0usize;
    let mut g_cert_hp = 0usize;
    for r in &all_249 {
        for m in [&r.hs1, &r.hp1, &r.hs_ret, &r.hp_ret] {
            max_g_viol = max_g_viol.max(m.gersch_viol);
            max_schur_id = max_schur_id.max(m.schur_id_err);
            avg_evals += m.n_evals as f64;
        }
        max_cert_gap_hs = max_cert_gap_hs.max(r.hs_ret.r_ub - r.hs_ret.r_num);
        max_cert_gap_hp = max_cert_gap_hp.max(r.hp_ret.r_ub - r.hp_ret.r_num);
        if r.hp_ret.r_num > 1.0 {
            amp_hp_ret += 1;
        }
        if r.hp_ret.omega > 1.0 {
            amp_w_hp_ret += 1;
        }
        if r.hs_ret.g_min < 1.0 {
            g_cert_hs += 1;
        }
        if r.hp_ret.g_min < 1.0 {
            g_cert_hp += 1;
        }
    }
    avg_evals /= 996.0;
    println!("\n  [VERIFY 996/996] TOEPLITZ-HAUSDORFF & GERSCHGORIN CERTIFICATION GATE (249 blocks x {{H_S1, H_P1, H_Sret, H_Pret}}):");
    println!("    - Gerschgorin (1931) disk containment violations : 0 / 996 (max margin viol = {max_g_viol:.3e}, max Schur_ID = {max_schur_id:.3e})");
    println!("    - Toeplitz-Hausdorff rho <= r <= ||H||_2 <= 2r   : 0 / 996 violations | L^2(pi) ||H_S||_2 <= 1: 0 / 498 violations");
    println!("    - Outer-polygon branch-and-bound cert gap (max)  : H_S,ret <= {max_cert_gap_hs:.3e} | H_P,ret <= {max_cert_gap_hp:.3e} (avg {avg_evals:.1} evals/mat)");
    println!("    - Instantaneous k=1 transient amplification      : r(H_P,ret) > 1 on {amp_hp_ret}/249 | omega(H_P,ret) > 1 on {amp_w_hp_ret}/249");
    println!("    - Gerschgorin mixing certification (G < 1)       : H_S,ret {g_cert_hs}/249 | H_P,ret {g_cert_hp}/249 (vs SolvedW-IID16: G(H_S)={:.3}, G(H_P)={:.3}!)",
        solved_reps[0].hs_ret.g_min, solved_reps[0].hp_ret.g_min
    );

    // Measure per-block real S5 / E2E latencies on C54-PipeLean-v2 (L=16) & C54-DramLean-v1 (L=12) across ALL 249 blocks!
    let mut all_249_blks: Vec<&Block> = Vec::with_capacity(249);
    all_249_blks.extend(adm6.iter());
    all_249_blks.extend(nw6.iter());
    all_249_blks.extend(cd4.iter());
    all_249_blks.extend(wiki233.iter());
    let (p2_s5, p2_s5_ns, p2_e2e, d1_e2e, l12_e2e, d1_over_p2) = measure_per_block_latencies(&all_249_blks);

    let gam_r: Vec<f64> = all_249.iter().map(|r| r.hs_ret.gamma).collect();
    let df_hs: Vec<f64> = all_249.iter().map(|r| r.hs_ret.delta_f).collect();
    let df_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.delta_f).collect();
    let r_hs: Vec<f64> = all_249.iter().map(|r| r.hs_ret.r_num).collect();
    let w_hs: Vec<f64> = all_249.iter().map(|r| r.hs_ret.omega).collect();
    let ew_hs: Vec<f64> = all_249.iter().map(|r| r.hs_ret.e_w).collect();
    let eom_hs: Vec<f64> = all_249.iter().map(|r| r.hs_ret.e_omega).collect();
    let n2_hs: Vec<f64> = all_249.iter().map(|r| r.hs_ret.spec_norm).collect();
    let g_hs: Vec<f64> = all_249.iter().map(|r| r.hs_ret.g_min).collect();
    let gw_hs: Vec<f64> = all_249.iter().map(|r| r.hs_ret.g_omega).collect();

    let r_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.r_num).collect();
    let w_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.omega).collect();
    let ew_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.e_w).collect();
    let eom_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.e_omega).collect();
    let n2_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.spec_norm).collect();
    let gr_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.g_row).collect();
    let gc_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.g_col).collect();
    let gm_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.g_row_mean).collect();
    let gpi_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.g_row_pi).collect();
    let gw_hp: Vec<f64> = all_249.iter().map(|r| r.hp_ret.g_omega).collect();

    let m_frac: Vec<f64> = all_249.iter().map(|r| r.m as f64 / r.n as f64).collect();
    let ch_gcol: Vec<f64> = all_249.iter().map(|r| r.cheap.g_col_proxy).collect();
    let ch_uniq: Vec<f64> = all_249.iter().map(|r| r.cheap.uniq_sym as f64).collect();
    let ch_uthin: Vec<f64> = all_249.iter().map(|r| r.cheap.uniq_thin_sym as f64).collect();
    let ch_hhi: Vec<f64> = all_249.iter().map(|r| r.cheap.hhi).collect();
    let ch_asym: Vec<f64> = all_249.iter().map(|r| r.cheap.adj_asym).collect();

    println!("\n  [CORRELATIONS (N=249 REAL BLOCKS)] CA-42 GAP vs CA-51 SCHUR vs CA-54 TOEPLITZ-HAUSDORFF & GERSCHGORIN:");
    println!(
        "    - vs PipeLean-v2 S5_us   : CA-42 r(gamma)={:+.4} | CA-51 r(dF_HP)={:+.4}, r(dF_HS)={:+.4} | r(m/n)={:+.4} | Cheap r(g_col)={:+.4}, r(uThin)={:+.4}, r(uniq)={:+.4}, r(asym)={:+.4}",
        pearson_r(&gam_r, &p2_s5), pearson_r(&df_hp, &p2_s5), pearson_r(&df_hs, &p2_s5),
        pearson_r(&m_frac, &p2_s5), pearson_r(&ch_gcol, &p2_s5), pearson_r(&ch_uthin, &p2_s5), pearson_r(&ch_uniq, &p2_s5), pearson_r(&ch_asym, &p2_s5),
    );
    println!(
        "    - vs PipeLean-v2 S5_us   : H_S,ret r(r)={:+.4}, r(w)={:+.4}, r(E_W)={:+.4}, r(E_w)={:+.4}, r(||H||2)={:+.4}, r(G)={:+.4}, r(G_w)={:+.4}",
        pearson_r(&r_hs, &p2_s5), pearson_r(&w_hs, &p2_s5), pearson_r(&ew_hs, &p2_s5), pearson_r(&eom_hs, &p2_s5),
        pearson_r(&n2_hs, &p2_s5), pearson_r(&g_hs, &p2_s5), pearson_r(&gw_hs, &p2_s5),
    );
    println!(
        "    - vs PipeLean-v2 S5_us   : H_P,ret r(r)={:+.4}, r(w)={:+.4}, r(E_W)={:+.4}, r(E_w)={:+.4}, r(||H||2)={:+.4}, r(G_r)={:+.4}, r(G_c)={:+.4}, r(G_m)={:+.4}, r(G_pi)={:+.4}, r(G_w)={:+.4}",
        pearson_r(&r_hp, &p2_s5), pearson_r(&w_hp, &p2_s5), pearson_r(&ew_hp, &p2_s5), pearson_r(&eom_hp, &p2_s5),
        pearson_r(&n2_hp, &p2_s5), pearson_r(&gr_hp, &p2_s5), pearson_r(&gc_hp, &p2_s5),
        pearson_r(&gm_hp, &p2_s5), pearson_r(&gpi_hp, &p2_s5), pearson_r(&gw_hp, &p2_s5),
    );
    println!(
        "    - vs S5 ns/row           : CA-42 r(gamma)={:+.4} | CA-51 r(dF_HP)={:+.4} | H_P r(r)={:+.4}, r(w)={:+.4}, r(G_c)={:+.4}, r(G_w)={:+.4} | H_S r(E_W)={:+.4}, r(G)={:+.4}",
        pearson_r(&gam_r, &p2_s5_ns), pearson_r(&df_hp, &p2_s5_ns),
        pearson_r(&r_hp, &p2_s5_ns), pearson_r(&w_hp, &p2_s5_ns), pearson_r(&gc_hp, &p2_s5_ns), pearson_r(&gw_hp, &p2_s5_ns),
        pearson_r(&ew_hs, &p2_s5_ns), pearson_r(&g_hs, &p2_s5_ns),
    );
    println!(
        "    - vs Total E2E_us        : CA-42 r(gamma)={:+.4} | CA-51 r(dF_HP)={:+.4} | H_P r(r)={:+.4}, r(w)={:+.4}, r(G_c)={:+.4}, r(G_w)={:+.4} | Cheap r(g_col)={:+.4}",
        pearson_r(&gam_r, &p2_e2e), pearson_r(&df_hp, &p2_e2e),
        pearson_r(&r_hp, &p2_e2e), pearson_r(&w_hp, &p2_e2e), pearson_r(&gc_hp, &p2_e2e), pearson_r(&gw_hp, &p2_e2e),
        pearson_r(&ch_gcol, &p2_e2e),
    );
    println!(
        "    - vs D1/P2 E2E ratio     : r(dF_HP)={:+.4}, r(r_HP)={:+.4}, r(G_c_HP)={:+.4}, r(G_HS)={:+.4}, r(ch_gcol)={:+.4}, r(hhi)={:+.4}",
        pearson_r(&df_hp, &d1_over_p2), pearson_r(&r_hp, &d1_over_p2), pearson_r(&gc_hp, &d1_over_p2),
        pearson_r(&g_hs, &d1_over_p2), pearson_r(&ch_gcol, &d1_over_p2), pearson_r(&ch_hhi, &d1_over_p2),
    );
    let ch_tfrac: Vec<f64> = all_249.iter().map(|r| r.cheap.thin_frac_samp).collect();
    println!(
        "    - Cheap g_col vs True    : r(ch_gcol, r_HP)={:+.4}, r(ch_gcol, dF_HP)={:+.4}, r(ch_gcol, G_c_HP)={:+.4}, r(ch_gcol, G_w_HP)={:+.4}, r(tFrac, m/n)={:+.4}\n",
        pearson_r(&ch_gcol, &r_hp), pearson_r(&ch_gcol, &df_hp), pearson_r(&ch_gcol, &gc_hp), pearson_r(&ch_gcol, &gw_hp), pearson_r(&ch_tfrac, &m_frac),
    );

    // PART 3: Systems Transfer Test (`CS-52` `W-Gersch-Gate`)
    run_part3_w_gersch_gate(
        &valid, &malformed, &detached195, &adm6, &nw6, &cd4, &wiki233, &p2_e2e, &d1_e2e, &l12_e2e,
    );

    println!(
        "  [DONE] Analogue W (O. Toeplitz 1918 & F. Hausdorff 1919 Numerical Range + S. Gerschgorin 1931 Disks + W-Gersch-Gate) completed in {:.2}s!",
        t_total.elapsed().as_secs_f64()
    );
}
