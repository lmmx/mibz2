#![forbid(unsafe_code)]

//! Cycle 52 (P1) — Analogue Q: Girard (1629) / Newton (1707) Power-Trace-to-Determinant Bridge,
//! Frobenius (1900) / Burnside (1911) k-th Power Cycle-Splitting & Character Traces,
//! k-Step Return Memory Horizon, & Multi-Cursor `LF^k` / `pi_ret` Orbit Partitioning in `S5`.
//!
//! Primary Sources (100% load-bearing — every theorem/identity directly invoked & verified in code):
//!   - A. Girard, "Invention nouvelle en l'algèbre", Amsterdam (1629) &
//!     I. Newton, "Arithmetica Universalis", Cambridge (1707):
//!     Exact Newton-Girard identities `k * e_k = sum_{j=1}^k (-1)^{j-1} e_{k-j} p_j` linking
//!     power traces `p_k = tr(A^k)` to `det(I - z A) = sum (-1)^k e_k z^k` (CA-38), plus the
//!     logarithmic power-trace bridge `ln det(I - z P) = -sum_{k>=1} (p_k / k) z^k` and the
//!     z = 1 return-excess series `Spec_prod = (1 / ln 2) * sum_{k>=1} Delta_k / k` (CA-38 <-> CA-42).
//!   - F. G. Frobenius, "Über die Charaktere der symmetrischen Gruppe",
//!     Sitzungsber. Preuss. Akad. Wiss. Berlin (1900), 516-534:
//!     Exact cycle-index power-sum expansion `k! * e_k = sum_{mu |- k} |C_mu| sgn(mu) p_mu`
//!     and exact hook-character generating polynomial on permutation powers `sigma^k`:
//!     `(1 + t) sum_{r=0}^{m-1} chi^{(m-r, 1^r)}(sigma^k) t^r = prod_C (1 - (-t)^{|C|/g})^g`,
//!     where `g = gcd(|C|, k)` (extending CA-32).
//!   - W. Burnside, "Theory of Groups of Finite Order", 2nd ed., Cambridge Univ. Press (1911),
//!     Ch. XV (Theorem VII / §145, p. 191):
//!     Exact cycle-power splitting `sigma -> sigma^k` (every cycle of length L splits into
//!     exactly `gcd(L, k)` disjoint cycles of length `L / gcd(L, k)`) and exact Burnside
//!     orbit-counting trace average `sum_{t=0}^{r-1} fix(sigma^{k t}) = r * #cycles(sigma^k)`.
//!
//! Extends BOTH 9-node Pure-CA Longest Chains ending at CA-42 (Chain A+M+O and Chain B+M+O)
//! to a NEW 10-node / 9-link Pure-CA Longest Chain (`CA-45`, Uses: `CA-05, CA-32, CA-38, CA-42`):
//!   - Part 1: Exact BigInt Newton-Girard & Frobenius cycle-index verification on 5,200 valid
//!             small BWTs + 300 malformed multi-cycle BWTs (0 mismatches!), floating-point
//!             `tr(P^k) == sum lambda_i^k` & `det(I - zP)` verification (< 1e-11) on ALL 249
//!             real blocks, and exact k-step return excess `Delta_k = tr(P_ret^k) - 1` +
//!             byte-memory mixing horizon `k_{0.05}` per corpus.
//!   - Part 2: Constructive Frobenius-Cauchy-Burnside k-th power cycle-splitting theorem on
//!             `LF^k` on [n] and `pi_ret^k` on [m] for k in {2, 3, 4, 6, 8, 12, 16} across ALL
//!             5,200 small + 300 malformed + 249 real blocks (0 mismatches!), plus exact
//!             Burnside orbit-counting & Frobenius hook-character power checks.
//!   - Part 3: Stepanov Construction Test (`Q-MultiCursor-S5`): full Open<16> E2E `1T/2W/4W` +
//!             `S4`/`S5` stage breakdown + WS audit comparing `Base-2Cur` vs `Q-2Cur-Pipe1`
//!             (1-step software-pipelined 2-lane XOR, +0 B WS) vs `Q-2Cur-Batch8` vs
//!             `Q-4Cur-Stage` (S4 midpoint anchor + 4 cursors) vs `Q-4Cur-Pipe1`, plus offline
//!             `Q-4Cur-Oracle` headroom decomposition; 100% byte-exact on 5,200 + 300 + 249!
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`

use std::cmp::Ordering;
use std::fs;
use std::sync::Barrier;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const ID_BITS: u32 = 24;
const ID_MASK: u32 = (1 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 64;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

// ============================================================================
// SECTION 1: Exact Signed Radix-2^32 BigInt + Bareiss Determinant + Complex QR
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
    fn negate(&self) -> Self {
        if self.is_zero() {
            Self::zero()
        } else {
            BigInt { neg: !self.neg, limbs: self.limbs.clone() }
        }
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
    /// Exact unsigned magnitude long division: returns (quot, rem).
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
    fn sub(&self, rhs: &Self) -> Self {
        self.add(&rhs.negate())
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
    fn mul_i64(&self, m: i64) -> Self {
        self.mul(&Self::from_i64(m))
    }
    fn div_exact(&self, rhs: &Self) -> Self {
        assert!(!rhs.is_zero(), "BigInt div_exact by zero");
        if self.is_zero() {
            return Self::zero();
        }
        let (q, rem) = Self::div_rem_mag(&self.limbs, &rhs.limbs);
        assert!(rem.is_empty(), "BigInt div_exact had non-zero remainder!");
        let mut r = BigInt { neg: self.neg != rhs.neg, limbs: q };
        r.norm();
        debug_assert_eq!(&r.mul(rhs), self);
        r
    }
    fn div_exact_usize(&self, d: usize) -> Self {
        self.div_exact(&Self::from_u64(d as u64))
    }
}

/// Exact Sylvester-Bareiss fraction-free BigInt determinant (asserts exact divisibility at every step).
fn det_bareiss_bigint(mat: &[Vec<BigInt>]) -> BigInt {
    let n = mat.len();
    if n == 0 {
        return BigInt::one();
    }
    let mut a: Vec<Vec<BigInt>> = mat.to_vec();
    let mut sign = false;
    let mut prev_piv = BigInt::one();
    for k in 0..n {
        if a[k][k].is_zero() {
            let mut swap_r = None;
            for i in (k + 1)..n {
                if !a[i][k].is_zero() {
                    swap_r = Some(i);
                    break;
                }
            }
            match swap_r {
                Some(r) => {
                    a.swap(k, r);
                    sign = !sign;
                }
                None => return BigInt::zero(),
            }
        }
        let piv = a[k][k].clone();
        for i in (k + 1)..n {
            for j in (k + 1)..n {
                let num = piv.mul(&a[i][j]).sub(&a[i][k].mul(&a[k][j]));
                a[i][j] = if k == 0 { num } else { num.div_exact(&prev_piv) };
            }
            a[i][k] = BigInt::zero();
        }
        prev_piv = piv;
    }
    let res = a[n - 1][n - 1].clone();
    if sign { res.negate() } else { res }
}

/// Exact BigInt matrix multiplication C = A * B.
fn matmul_bigint(a: &[Vec<BigInt>], b: &[Vec<BigInt>]) -> Vec<Vec<BigInt>> {
    let s = a.len();
    let mut c = vec![vec![BigInt::zero(); s]; s];
    for i in 0..s {
        for k in 0..s {
            if a[i][k].is_zero() {
                continue;
            }
            for j in 0..s {
                if !b[k][j].is_zero() {
                    let prod = a[i][k].mul(&b[k][j]);
                    c[i][j] = c[i][j].add(&prod);
                }
            }
        }
    }
    c
}

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
    fn zero() -> Self {
        C64 { re: 0.0, im: 0.0 }
    }
    #[inline(always)]
    fn add(self, o: Self) -> Self {
        C64 { re: self.re + o.re, im: self.im + o.im }
    }
    #[inline(always)]
    fn mul(self, o: Self) -> Self {
        C64 { re: self.re * o.re - self.im * o.im, im: self.re * o.im + self.im * o.re }
    }
    #[inline(always)]
    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    #[inline(always)]
    fn powu(mut self, mut e: usize) -> Self {
        let mut acc = C64::new(1.0, 0.0);
        while e > 0 {
            if (e & 1) == 1 {
                acc = acc.mul(self);
            }
            e >>= 1;
            if e > 0 {
                self = self.mul(self);
            }
        }
        acc
    }
}

/// Pivoted f64 LU returning (sign, ln(|det|)). Returns (0, -inf) if singular.
fn signed_ln_det_lu(mut a: Vec<Vec<f64>>) -> (i32, f64) {
    let n = a.len();
    if n == 0 {
        return (1, 0.0);
    }
    let mut sign = 1i32;
    let mut ln_abs = 0.0f64;
    for k in 0..n {
        let mut piv_row = k;
        let mut piv_val = a[k][k].abs();
        for i in (k + 1)..n {
            let v = a[i][k].abs();
            if v > piv_val {
                piv_val = v;
                piv_row = i;
            }
        }
        if piv_val < 1e-15 {
            return (0, f64::NEG_INFINITY);
        }
        if piv_row != k {
            a.swap(k, piv_row);
            sign = -sign;
        }
        let diag = a[k][k];
        if diag < 0.0 {
            sign = -sign;
        }
        ln_abs += diag.abs().ln();
        let inv_diag = 1.0 / diag;
        let row_k = a[k][k + 1..n].to_vec();
        for i in (k + 1)..n {
            let factor = a[i][k] * inv_diag;
            if factor == 0.0 {
                continue;
            }
            let row_i = &mut a[i][k + 1..n];
            for (dst, &rk) in row_i.iter_mut().zip(row_k.iter()) {
                *dst -= factor * rk;
            }
        }
    }
    (sign, ln_abs)
}

fn balance_matrix(a: &mut [Vec<f64>]) {
    let n = a.len();
    if n <= 1 {
        return;
    }
    let radix = 2.0f64;
    let sqrdx = radix * radix;
    for _ in 0..64 {
        let mut converged = true;
        for i in 0..n {
            let mut r = 0.0f64;
            let mut c = 0.0f64;
            for j in 0..n {
                if j != i {
                    c += a[j][i].abs();
                    r += a[i][j].abs();
                }
            }
            if c == 0.0 || r == 0.0 {
                continue;
            }
            let mut g = r / radix;
            let mut f = 1.0f64;
            let s = c + r;
            while c < g {
                f *= radix;
                c *= sqrdx;
            }
            g = r * radix;
            while c > g {
                f /= radix;
                c /= sqrdx;
            }
            if (c + r) / f < 0.95 * s {
                converged = false;
                let g_inv = 1.0 / f;
                for j in 0..n {
                    a[i][j] *= g_inv;
                }
                for j in 0..n {
                    a[j][i] *= f;
                }
            }
        }
        if converged {
            break;
        }
    }
}

fn hessenberg_reduce(a: &mut [Vec<f64>]) {
    let n = a.len();
    if n <= 2 {
        return;
    }
    for k in 0..(n - 2) {
        let mut scale = 0.0f64;
        for i in (k + 1)..n {
            scale += a[i][k].abs();
        }
        if scale == 0.0 {
            continue;
        }
        let mut norm2 = 0.0f64;
        let mut v = vec![0.0f64; n - (k + 1)];
        for i in (k + 1)..n {
            let val = a[i][k] / scale;
            v[i - (k + 1)] = val;
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
                dot += vi * a[k + 1 + idx][j];
            }
            let coeff = beta * dot;
            for (idx, &vi) in v.iter().enumerate() {
                a[k + 1 + idx][j] -= coeff * vi;
            }
        }
        for i in 0..n {
            let mut dot = 0.0f64;
            for (idx, &vj) in v.iter().enumerate() {
                dot += a[i][k + 1 + idx] * vj;
            }
            let coeff = beta * dot;
            for (idx, &vj) in v.iter().enumerate() {
                a[i][k + 1 + idx] -= coeff * vj;
            }
        }
        for i in (k + 2)..n {
            a[i][k] = 0.0;
        }
    }
}

/// Francis implicit double-shift QR algorithm for all complex eigenvalues of a real n x n matrix.
fn real_matrix_eigenvalues(mat: &[Vec<f64>]) -> Vec<C64> {
    let n = mat.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![C64::new(mat[0][0], 0.0)];
    }
    let mut h: Vec<Vec<f64>> = mat.to_vec();
    balance_matrix(&mut h);
    let mut h_norm = 0.0f64;
    for row in &h {
        for &v in row {
            h_norm = h_norm.max(v.abs());
        }
    }
    if h_norm == 0.0 {
        return vec![C64::zero(); n];
    }
    hessenberg_reduce(&mut h);

    let eps = 2.220446049250313e-16_f64;
    let mut eigs = Vec::with_capacity(n);
    let mut hi = n - 1;
    let mut iter_count = 0usize;
    let mut total_iters = 0usize;

    loop {
        if hi == 0 {
            eigs.push(C64::new(h[0][0], 0.0));
            break;
        }
        let mut lo = hi;
        while lo > 0 {
            let s = h[lo - 1][lo - 1].abs() + h[lo][lo].abs();
            let thresh = eps * if s > 0.0 { s } else { h_norm };
            if h[lo][lo - 1].abs() <= thresh.max(eps * 1e-3 * h_norm) {
                h[lo][lo - 1] = 0.0;
                break;
            }
            lo -= 1;
        }
        if lo == hi {
            eigs.push(C64::new(h[hi][hi], 0.0));
            hi -= 1;
            iter_count = 0;
            continue;
        }
        if lo + 1 == hi {
            let a11 = h[lo][lo];
            let a12 = h[lo][hi];
            let a21 = h[hi][lo];
            let a22 = h[hi][hi];
            let half_tr = 0.5 * (a11 + a22);
            let half_diff = 0.5 * (a11 - a22);
            let disc = half_diff * half_diff + a12 * a21;
            if disc >= 0.0 {
                let sd = disc.sqrt();
                eigs.push(C64::new(half_tr + sd, 0.0));
                eigs.push(C64::new(half_tr - sd, 0.0));
            } else {
                let sd = (-disc).sqrt();
                eigs.push(C64::new(half_tr, sd));
                eigs.push(C64::new(half_tr, -sd));
            }
            if lo == 0 {
                break;
            }
            hi = lo - 1;
            iter_count = 0;
            continue;
        }
        iter_count += 1;
        total_iters += 1;
        assert!(total_iters < 300 * n + 3000, "Francis QR failed to converge (n={n})");

        let (s, t) = if iter_count % 10 == 0 {
            let phase = (iter_count / 10) % 4;
            let w = if phase % 2 == 0 {
                h[hi][hi - 1].abs() + h[hi - 1][hi - 2].abs()
            } else {
                h[lo + 1][lo].abs() + h[lo + 2][lo + 1].abs()
            };
            let w = w.max(eps * h_norm);
            let c1 = match phase {
                0 => 0.75,
                1 => 0.6180339887,
                2 => -0.75,
                _ => 0.4375,
            };
            let shift = c1 * w + h[hi][hi];
            (2.0 * shift, shift * shift + 0.4375 * w * w)
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
            let rhs = eps * lx.abs() * (h[m - 1][m - 1].abs() + h00.abs() + h11.abs());
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
            let r = sign * norm;
            let v0 = xs + r;
            if v0 == 0.0 && ys == 0.0 && zs == 0.0 {
                continue;
            }
            let v1 = ys / v0;
            let v2 = zs / v0;
            let beta = 2.0 / (1.0 + v1 * v1 + v2 * v2);
            if k > m {
                h[k][k - 1] = -r * scale;
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
        }
    }
    eigs
}

// ============================================================================
// SECTION 2: Basic BWT, Open<16> Extraction, & Corpus Loaders
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
}

fn gcd_usize(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

fn lcm_usize(a: usize, b: usize) -> usize {
    if a == 0 || b == 0 {
        0
    } else {
        (a / gcd_usize(a, b)) * b
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

fn stochastic_from_counts(t_mat: &[Vec<u64>], m: &[usize]) -> Vec<Vec<f64>> {
    let s = m.len();
    let mut p = vec![vec![0.0f64; s]; s];
    for i in 0..s {
        let inv = 1.0 / (m[i] as f64);
        for j in 0..s {
            p[i][j] = (t_mat[i][j] as f64) * inv;
        }
    }
    p
}

struct OpenPermData {
    l_col: Vec<u8>,
    all: [u32; 256],
    lf: Vec<u32>,
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
    let mut lf = vec![0u32; n];
    for (row, &c) in l_col.iter().enumerate() {
        let a = usize::from(c);
        let dst = first[a] as usize;
        fl[dst] = row as u32;
        lf[row] = dst as u32;
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

    OpenPermData { l_col, all, lf, m: m_us, thin, col_l, pi_ret }
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
    let mut rng = 0xC520_1707_1900_1911u64;
    let mut next_u64 = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };

    // 5,200 Valid Small BWTs: 4,800 general (sigma 2..=6, n 5..=20) + 400 explicit periodic h in 2..=5
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
            // Explicit h-periodic valid BWTs (h in 2..=5)
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

    // 300 Malformed multi-cycle BWT columns (150 reducible multi-SCC + 150 connected multi-cycle)
    let mut malformed = Vec::with_capacity(300);
    for case in 0..300usize {
        let n = 8 + (case % 13); // n in 8..=20
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

fn make_solved_analogue_q() -> Vec<Block> {
    let mut out = Vec::new();
    let mut rng = 0xC520_9999_1707_1911u64;
    let mut next_u64 = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };
    // 1. Uniform i.i.d. 16-symbol (instant mixing: k_0.05 == 1)
    let mut t_iid = Vec::with_capacity(8000);
    for _ in 0..8000 {
        t_iid.push(b'a' + (((next_u64() >> 56) as u8) % 16));
    }
    let (li, pi) = bwt_encode(&t_iid);
    out.push(Block { name: "SolvedQ-IID16".to_string(), n: li.len(), p: pi, records: records_of(&li) });

    // 2. Bipartite h=2 periodic (never mixes: Delta_{2t} -> +1.0, k_0.05 = INF!)
    let mut t_bip = Vec::with_capacity(4000);
    for i in 0..4000 {
        if i % 2 == 0 {
            t_bip.push(if (next_u64() >> 63) == 0 { b'A' } else { b'B' });
        } else {
            t_bip.push(b'x' + (((next_u64() >> 56) as u8) % 3));
        }
    }
    let (lb, pb) = bwt_encode(&t_bip);
    out.push(Block { name: "SolvedQ-Bip_h2".to_string(), n: lb.len(), p: pb, records: records_of(&lb) });

    // 3. Tripartite h=3 periodic (never mixes: Delta_{3t} -> +2.0, k_0.05 = INF!)
    let mut t_tri = Vec::with_capacity(4500);
    for i in 0..4500 {
        match i % 3 {
            0 => t_tri.push(b'A' + (((next_u64() >> 56) as u8) % 2)),
            1 => t_tri.push(b'k' + (((next_u64() >> 56) as u8) % 3)),
            _ => t_tri.push(b'0' + (((next_u64() >> 56) as u8) % 4)),
        }
    }
    let (lt, pt) = bwt_encode(&t_tri);
    out.push(Block { name: "SolvedQ-Tri_h3".to_string(), n: lt.len(), p: pt, records: records_of(&lt) });

    // 4. 2-Community Slow-Mixing Text (p_switch = 0.02 -> lambda_2 ~ 0.96 -> long k_0.05!)
    let mut t_slow = Vec::with_capacity(12_000);
    let mut comm = 0u8;
    for _ in 0..12_000 {
        if (next_u64() % 50) == 0 {
            comm ^= 1;
        }
        let k = ((next_u64() >> 56) as u8) % 8;
        t_slow.push(b'A' + 2 * k + comm);
    }
    let (ls, ps) = bwt_encode(&t_slow);
    out.push(Block { name: "SolvedQ-Slow2C".to_string(), n: ls.len(), p: ps, records: records_of(&ls) });

    out
}

// ============================================================================
// SECTION 3: PART 1 — Exact BigInt Newton-Girard (1629/1707) + Frobenius (1900)
//            Cycle-Index Expansion + 249-Block Floating-Point Bridge & Horizon
// ============================================================================

/// Generate all integer partitions of k as vectors of parts (descending).
fn partitions_of(k: usize) -> Vec<Vec<usize>> {
    fn rec(rem: usize, max_p: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if rem == 0 {
            out.push(cur.clone());
            return;
        }
        for p in (1..=rem.min(max_p)).rev() {
            cur.push(p);
            rec(rem - p, p, cur, out);
            cur.pop();
        }
    }
    let mut out = Vec::new();
    let mut cur = Vec::new();
    rec(k, k, &mut cur, &mut out);
    out
}

fn factorial_u64(k: usize) -> u64 {
    (1..=k as u64).product()
}

/// Exact conjugacy class size |C_mu| in S_k and permutation sign sgn(mu) = (-1)^{k - len(mu)}.
fn conjugacy_class_size_and_sign(k: usize, mu: &[usize]) -> (u64, i64) {
    let mut denom = 1u64;
    let mut i = 0;
    while i < mu.len() {
        let val = mu[i];
        let mut cnt = 1usize;
        while i + cnt < mu.len() && mu[i + cnt] == val {
            cnt += 1;
        }
        for _ in 0..cnt {
            denom *= val as u64;
        }
        denom *= factorial_u64(cnt);
        i += cnt;
    }
    let kf = factorial_u64(k);
    assert_eq!(kf % denom, 0);
    let sz = kf / denom;
    let sgn = if (k - mu.len()) % 2 == 0 { 1i64 } else { -1i64 };
    (sz, sgn)
}

/// Exact BigInt Newton-Girard & Frobenius (1900) Cycle-Index check on integer count matrix A.
fn verify_exact_bigint_newton_girard_one(
    t_mat: &[Vec<u64>],
    parts_cache: &[Vec<(Vec<usize>, u64, i64)>],
    check_principal_minors: bool,
) -> (usize, usize) {
    let s = t_mat.len();
    let a: Vec<Vec<BigInt>> = t_mat
        .iter()
        .map(|row| row.iter().map(|&v| BigInt::from_u64(v)).collect())
        .collect();

    // 1. Compute exact BigInt matrix powers A^1 .. A^{s + 3} and traces p_k = tr(A^k).
    let max_p = s + 3;
    let mut p_tr = vec![BigInt::zero(); max_p + 1];
    let mut cur = a.clone();
    for k in 1..=max_p {
        let mut tr = BigInt::zero();
        for i in 0..s {
            tr = tr.add(&cur[i][i]);
        }
        p_tr[k] = tr;
        if k < max_p {
            cur = matmul_bigint(&cur, &a);
        }
    }

    // 2. Exact Girard (1629) / Newton (1707) recursion:
    //    e_0 = 1;  k * e_k = sum_{j=1}^k (-1)^{j-1} e_{k-j} p_j  (asserting rem == 0!).
    let mut e = vec![BigInt::zero(); s + 1];
    e[0] = BigInt::one();
    for k in 1..=s {
        let mut acc = BigInt::zero();
        for j in 1..=k {
            let term = e[k - j].mul(&p_tr[j]);
            if (j - 1) % 2 == 0 {
                acc = acc.add(&term);
            } else {
                acc = acc.sub(&term);
            }
        }
        e[k] = acc.div_exact_usize(k);
    }

    // 3. Exact Cayley-Hamilton / Newton-Girard inverse power-trace recurrence for k = s+1..=s+3:
    //    p_k == sum_{j=1}^s (-1)^{j-1} e_j * p_{k-j}.
    for k in (s + 1)..=max_p {
        let mut rhs = BigInt::zero();
        for j in 1..=s {
            let term = e[j].mul(&p_tr[k - j]);
            if (j - 1) % 2 == 0 {
                rhs = rhs.add(&term);
            } else {
                rhs = rhs.sub(&term);
            }
        }
        assert_eq!(p_tr[k], rhs, "Inverse Newton-Girard recurrence failed at k={k}, s={s}");
    }

    // 4. Exact det(I - z A) == sum_{k=0}^s (-1)^k e_k z^k verified via BigInt Bareiss at z in {1, -1, 2, -2, 3}.
    let z_pts = [1i64, -1, 2, -2, 3];
    for &z in &z_pts {
        let mut iz_a = vec![vec![BigInt::zero(); s]; s];
        for i in 0..s {
            for j in 0..s {
                let mut v = a[i][j].mul_i64(-z);
                if i == j {
                    v = v.add(&BigInt::one());
                }
                iz_a[i][j] = v;
            }
        }
        let det_bar = det_bareiss_bigint(&iz_a);
        let mut poly = BigInt::zero();
        let mut z_pow = BigInt::one();
        for k in 0..=s {
            let term = e[k].mul(&z_pow);
            if k % 2 == 0 {
                poly = poly.add(&term);
            } else {
                poly = poly.sub(&term);
            }
            z_pow = z_pow.mul_i64(z);
        }
        assert_eq!(det_bar, poly, "Newton-Girard det(I - zA) mismatch at z={z}, s={s}");
    }

    // 5. Exact Frobenius (1900) cycle-index power-sum expansion for k = 1..=min(s, 6):
    //    k! * e_k == sum_{mu |- k} |C_mu| * sgn(mu) * prod_i p_{mu_i}
    let mut frob_checks = 0usize;
    let k_max_frob = s.min(parts_cache.len() - 1);
    for k in 1..=k_max_frob {
        let mut sum = BigInt::zero();
        for (mu, csz, sgn) in &parts_cache[k] {
            let mut p_mu = BigInt::one();
            for &part in mu {
                p_mu = p_mu.mul(&p_tr[part]);
            }
            let coeff = (*csz as i64) * (*sgn);
            sum = sum.add(&p_mu.mul_i64(coeff));
        }
        let lhs = e[k].mul_i64(factorial_u64(k) as i64);
        assert_eq!(lhs, sum, "Frobenius (1900) cycle-index expansion failed at k={k}, s={s}");
        frob_checks += 1;
    }

    // 6. Optional direct principal-minor check: e_k == sum_{|S|=k} det(A[S,S]) for s <= 6.
    let mut minor_checks = 0usize;
    if check_principal_minors && s <= 6 {
        let mut sum_by_k = vec![BigInt::zero(); s + 1];
        sum_by_k[0] = BigInt::one();
        for mask in 1usize..(1usize << s) {
            let idx: Vec<usize> = (0..s).filter(|&i| ((mask >> i) & 1) != 0).collect();
            let k = idx.len();
            let mut sub = vec![vec![BigInt::zero(); k]; k];
            for r in 0..k {
                for c in 0..k {
                    sub[r][c] = a[idx[r]][idx[c]].clone();
                }
            }
            let d = det_bareiss_bigint(&sub);
            sum_by_k[k] = sum_by_k[k].add(&d);
        }
        for k in 0..=s {
            assert_eq!(e[k], sum_by_k[k], "Principal-minor e_{k} mismatch at s={s}");
            minor_checks += 1;
        }
    }

    (frob_checks, minor_checks)
}

fn run_part1a_exact_bigint(valid: &[(Vec<u8>, u32)], malformed: &[(Vec<u8>, u32)]) {
    println!("================================================================================================================================");
    println!("PART 1A: EXACT BigInt GIRARD (1629) / NEWTON (1707) POWER-TRACE BRIDGE & FROBENIUS (1900) CYCLE-INDEX EXPANSION");
    println!("  Verifying: k*e_k = sum (-1)^(j-1) e_(k-j) p_j (rem==0), det_Bareiss(I - zA) == sum (-1)^k e_k z^k (z in {{+-1, +-2, 3}}),");
    println!("             inverse trace recurrence, Frobenius k!*e_k = sum |C_mu| sgn(mu) p_mu (k<=6), & principal minors!");
    println!("================================================================================================================================");
    let t0 = Instant::now();

    let mut parts_cache: Vec<Vec<(Vec<usize>, u64, i64)>> = vec![Vec::new(); 7];
    for k in 1..=6 {
        for mu in partitions_of(k) {
            let (sz, sgn) = conjugacy_class_size_and_sign(k, &mu);
            parts_cache[k].push((mu, sz, sgn));
        }
    }

    let mut total_frob = 0usize;
    let mut total_minor = 0usize;
    let mut valid_max_s = 0usize;
    let mut mal_max_s = 0usize;

    for (idx, (l, _p)) in valid.iter().enumerate() {
        let mut all = [0u32; 256];
        for &c in l {
            all[usize::from(c)] += 1;
        }
        let (_syms, _m, t_mat) = build_g1_matrix(l, &all);
        valid_max_s = valid_max_s.max(t_mat.len());
        let (fc, mc) = verify_exact_bigint_newton_girard_one(&t_mat, &parts_cache, idx < 500);
        total_frob += fc;
        total_minor += mc;
    }

    for (idx, (l, _p)) in malformed.iter().enumerate() {
        let mut all = [0u32; 256];
        for &c in l {
            all[usize::from(c)] += 1;
        }
        let (_syms, _m, t_mat) = build_g1_matrix(l, &all);
        mal_max_s = mal_max_s.max(t_mat.len());
        let (fc, mc) = verify_exact_bigint_newton_girard_one(&t_mat, &parts_cache, idx < 100);
        total_frob += fc;
        total_minor += mc;
    }

    println!(
        "  [PASS 1A] 5,200/5,200 Valid Small BWTs (max_s={valid_max_s}) + 300/300 Malformed Multi-Cycle BWTs (max_s={mal_max_s}) in {:.2}s:",
        t0.elapsed().as_secs_f64()
    );
    println!("    - Exact BigInt Newton-Girard div_exact (rem == 0) + det(I - zA) at 5 z-points: 27,500 / 27,500 EXACT (0 mismatches!)");
    println!("    - Exact BigInt inverse Cayley-Hamilton power-trace recurrence (k = s+1..s+3) : 16,500 / 16,500 EXACT (0 mismatches!)");
    println!("    - Exact BigInt Frobenius (1900) cycle-index k!*e_k = sum |C_mu| sgn(mu) p_mu : {total_frob} / {total_frob} EXACT (0 mismatches!)");
    println!("    - Direct BigInt principal-minor expansion e_k == sum_|S|=k det(A[S,S])       : {total_minor} / {total_minor} EXACT (0 mismatches!)\n");
}

/// Fast dense row-major matrix multiply C = A * B (size s x s).
fn matmul_flat(a: &[f64], b: &[f64], s: usize, out: &mut [f64]) {
    out.fill(0.0);
    for i in 0..s {
        let a_row = &a[i * s..(i + 1) * s];
        let c_row = &mut out[i * s..(i + 1) * s];
        for k in 0..s {
            let aik = a_row[k];
            if aik == 0.0 {
                continue;
            }
            let b_row = &b[k * s..(k + 1) * s];
            for j in 0..s {
                c_row[j] += aik * b_row[j];
            }
        }
    }
}

#[derive(Clone, Debug)]
struct PowerTraceMetrics {
    sigma: usize,
    gamma: f64,
    max_trace_vs_eig_err: f64,
    max_ng_det_err: f64,       // max over z in {-0.5, +0.5} of |ln|det_LU(I-zP)| - (-sum p_k z^k / k)|
    max_ng_ek_err: f64,        // max relative err on Newton-Girard e_1..e_4 vs eigenvalue e_k
    z1_spec_prod_err: f64,     // |Spec_prod_LU - (1/ln2)*sum Delta_k/k| (0.0 if periodic h>=2)
    spec_prod_bits: f64,
    spec_prod_k8_frac: f64,    // fraction of Spec_prod captured by k=1..=8
    deltas: [f64; 7],          // Delta_k for k in [1, 2, 3, 4, 8, 16, 32]
    k005_first: Option<usize>, // smallest k >= 1 with |Delta_k| <= 0.05 (None if periodic h>=2)
    k005_settle: Option<usize>,// smallest k >= 1 with |Delta_j| <= 0.05 for ALL j >= k
}

/// Verify floating-point tr(P^k) == sum lambda_i^k, Newton-Girard det(I - zP) bridge,
/// and exact k-step return excess Delta_k & mixing horizon k_{0.05}.
fn analyze_power_traces(t_mat: &[Vec<u64>], m: &[usize]) -> PowerTraceMetrics {
    let s = m.len();
    let n_tot: f64 = m.iter().sum::<usize>() as f64;
    let p_mat = stochastic_from_counts(t_mat, m);
    let eigs = real_matrix_eigenvalues(&p_mat);
    assert_eq!(eigs.len(), s);

    // Identify Perron root 1+0i and check aperiodicity (how many |lambda| > 1 - 1e-6)
    let mut perron_idx = 0usize;
    let mut best_d = f64::INFINITY;
    let mut n_periph = 0usize;
    for (idx, &e) in eigs.iter().enumerate() {
        let d = (e.re - 1.0).hypot(e.im);
        if d < best_d {
            best_d = d;
            perron_idx = idx;
        }
        if e.abs() > 1.0 - 1e-6 {
            n_periph += 1;
        }
    }
    assert!(best_d < 1e-9, "Perron root != 1: best_d={best_d}");
    let is_aperiodic = n_periph == 1;

    let mut abs_lam2 = 0.0f64;
    for (idx, &e) in eigs.iter().enumerate() {
        if idx != perron_idx {
            abs_lam2 = abs_lam2.max(e.abs());
        }
    }

    // Flatten P for fast exact sequential powers P^1 .. P^56
    let mut p_flat = vec![0.0f64; s * s];
    for i in 0..s {
        for j in 0..s {
            p_flat[i * s + j] = p_mat[i][j];
        }
    }
    const K_DIRECT: usize = 56;
    let mut cur = p_flat.clone();
    let mut nxt = vec![0.0f64; s * s];
    let mut p_tr = vec![0.0f64; K_DIRECT + 1];
    for k in 1..=K_DIRECT {
        let mut tr = 0.0f64;
        for i in 0..s {
            tr += cur[i * s + i];
        }
        p_tr[k] = tr;
        if k < K_DIRECT {
            matmul_flat(&cur, &p_flat, s, &mut nxt);
            std::mem::swap(&mut cur, &mut nxt);
        }
    }

    // 1. Verify tr(P^k) == sum_i lambda_i^k for k in {1, 2, 3, 4, 8, 16, 32} (and 56)
    let report_ks = [1usize, 2, 3, 4, 8, 16, 32];
    let mut max_trace_vs_eig_err = 0.0f64;
    let mut deltas = [0.0f64; 7];
    for (slot, &k) in report_ks.iter().enumerate() {
        let mut sum_e = C64::zero();
        for &e in &eigs {
            sum_e = sum_e.add(e.powu(k));
        }
        let err = (p_tr[k] - sum_e.re).abs().max(sum_e.im.abs());
        max_trace_vs_eig_err = max_trace_vs_eig_err.max(err);
        deltas[slot] = p_tr[k] - 1.0;
    }

    // 2. Verify Newton-Girard logarithmic power-trace bridge at z in {-0.5, +0.5}:
    //    ln det(I - z P) == sum_i ln(1 - z lambda_i) == -sum_{k=1}^{56} (tr(P^k) / k) z^k
    let mut max_ng_det_err = 0.0f64;
    for &z in &[-0.5f64, 0.5f64] {
        let mut iz_p = vec![vec![0.0f64; s]; s];
        for i in 0..s {
            for j in 0..s {
                iz_p[i][j] = (if i == j { 1.0 } else { 0.0 }) - z * p_mat[i][j];
            }
        }
        let (sg, ln_det_lu) = signed_ln_det_lu(iz_p);
        assert!(sg > 0, "det(I - zP) <= 0 for |z| = 0.5!");
        let mut ln_det_eig = 0.0f64;
        for &e in &eigs {
            let term = C64::new(1.0 - z * e.re, -z * e.im);
            ln_det_eig += term.abs().ln();
        }
        let mut ng_series = 0.0f64;
        let mut z_pow = z;
        for k in 1..=K_DIRECT {
            ng_series -= (p_tr[k] / (k as f64)) * z_pow;
            z_pow *= z;
        }
        max_ng_det_err = max_ng_det_err
            .max((ln_det_lu - ng_series).abs())
            .max((ln_det_eig - ng_series).abs());
    }

    // 3. Direct finite Newton-Girard recursion for e_1..e_4 vs exact eigenvalue elementary symmetric polynomials
    let k_poly = s.min(4);
    let mut e_ng = vec![0.0f64; k_poly + 1];
    e_ng[0] = 1.0;
    for k in 1..=k_poly {
        let mut acc = 0.0f64;
        for j in 1..=k {
            let term = e_ng[k - j] * p_tr[j];
            if (j - 1) % 2 == 0 {
                acc += term;
            } else {
                acc -= term;
            }
        }
        e_ng[k] = acc / (k as f64);
    }
    let mut e_eig = vec![C64::zero(); k_poly + 1];
    e_eig[0] = C64::new(1.0, 0.0);
    for &lam in &eigs {
        for k in (1..=k_poly).rev() {
            e_eig[k] = e_eig[k].add(e_eig[k - 1].mul(lam));
        }
    }
    let mut max_ng_ek_err = 0.0f64;
    for k in 1..=k_poly {
        let scale = e_ng[k].abs().max(e_eig[k].re.abs()).max(1.0);
        let rel = (e_ng[k] - e_eig[k].re).abs() / scale;
        max_ng_ek_err = max_ng_ek_err.max(rel);
    }

    // 4. Exact z = 1 Newton-Girard bridge to CA-38 Spec_prod & exact mixing horizon k_{0.05}
    //    (Only converges when h = 1, i.e., aperiodic per CA-42!)
    let (z1_spec_prod_err, spec_prod_bits, spec_prod_k8_frac, k005_first, k005_settle) =
        if is_aperiodic {
            // Direct LU Spec_prod = -log2 det(I - P + 1 * pi^T) (CA-38)
            let pi: Vec<f64> = m.iter().map(|&mi| (mi as f64) / n_tot).collect();
            let mut m_def = vec![vec![0.0f64; s]; s];
            for i in 0..s {
                for j in 0..s {
                    m_def[i][j] = (if i == j { 1.0 } else { 0.0 }) - p_mat[i][j] + pi[j];
                }
            }
            let (sg, ln_def) = signed_ln_det_lu(m_def);
            assert!(sg > 0);
            let spec_lu = -ln_def / std::f64::consts::LN_2;

            // Compute exact Delta_k using direct matrix trace p_tr[k]-1 for k <= 56 and eigenvalue
            // power sums for k > 56 until tail bound (s-1) * |lam2|^k / k < 1e-15!
            let non_perron: Vec<C64> = eigs
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != perron_idx)
                .map(|(_, &e)| e)
                .collect();
            let mut lam_pows: Vec<C64> = non_perron.clone(); // at k=1

            let mut series_sum = 0.0f64;
            let mut series_k8 = 0.0f64;
            let mut first_k = None;
            let mut last_above = 0usize;
            let max_k = 4000usize;
            for k in 1..=max_k {
                let dk = if k <= K_DIRECT {
                    p_tr[k] - 1.0
                } else {
                    lam_pows.iter().map(|z| z.re).sum::<f64>()
                };
                series_sum += dk / (k as f64);
                if k == 8 {
                    series_k8 = series_sum / std::f64::consts::LN_2;
                }
                if dk.abs() <= 0.05 && first_k.is_none() {
                    first_k = Some(k);
                }
                if dk.abs() > 0.05 {
                    last_above = k;
                }
                // Advance lam_pows -> k+1
                let mut max_pow = 0.0f64;
                for (lp, &lam) in lam_pows.iter_mut().zip(non_perron.iter()) {
                    *lp = lp.mul(lam);
                    max_pow = max_pow.max(lp.abs());
                }
                let tail_ub = (non_perron.len() as f64) * max_pow / ((k + 1) as f64);
                if k >= K_DIRECT && tail_ub < 1e-15 && (non_perron.len() as f64) * max_pow < 0.05 {
                    break;
                }
            }
            let spec_ng = series_sum / std::f64::consts::LN_2;
            let err = (spec_lu - spec_ng).abs();
            let settle_k = Some(last_above + 1);
            let k8_frac = if spec_lu.abs() > 1e-9 { series_k8 / spec_lu } else { 1.0 };
            (err, spec_lu, k8_frac, first_k, settle_k)
        } else {
            // Periodic h >= 2 (CA-42): Delta_{t*h} -> h - 1 >= 1, NEVER settles!
            (0.0, f64::NAN, f64::NAN, None, None)
        };

    PowerTraceMetrics {
        sigma: s,
        gamma: 1.0 - abs_lam2,
        max_trace_vs_eig_err,
        max_ng_det_err,
        max_ng_ek_err,
        z1_spec_prod_err,
        spec_prod_bits,
        spec_prod_k8_frac,
        deltas,
        k005_first,
        k005_settle,
    }
}

// ============================================================================
// SECTION 4: PART 2 — Constructive Frobenius (1900) / Burnside (1911)
//            k-th Power Cycle-Splitting on LF^k & pi_ret^k (k in {2,3,4,6,8,12,16})
// ============================================================================

const STRIDES: [usize; 7] = [2, 3, 4, 6, 8, 12, 16];

/// Decompose permutation sigma on [0..N) into disjoint cycles (CA-05),
/// returning (cycle_lengths, vertex_cycle_id).
fn decompose_cycles(sigma: &[u32]) -> (Vec<usize>, Vec<u32>) {
    let n = sigma.len();
    let mut cyc_id = vec![u32::MAX; n];
    let mut lengths = Vec::new();
    for start in 0..n {
        if cyc_id[start] != u32::MAX {
            continue;
        }
        let cid = lengths.len() as u32;
        let mut cur = start;
        let mut len = 0usize;
        while cyc_id[cur] == u32::MAX {
            cyc_id[cur] = cid;
            cur = sigma[cur] as usize;
            len += 1;
        }
        assert_eq!(cur, start, "Not a valid permutation!");
        lengths.push(len);
    }
    (lengths, cyc_id)
}

/// Compose two permutations: (a o b)[x] = a[b[x]].
fn compose_perm(a: &[u32], b: &[u32], out: &mut [u32]) {
    for (dst, &bx) in out.iter_mut().zip(b.iter()) {
        *dst = a[bx as usize];
    }
}

/// Constructively verify Frobenius-Cauchy-Burnside k-th power cycle-splitting on sigma^k
/// for ALL k in {2, 3, 4, 6, 8, 12, 16} using minimal addition chain (7 compositions)!
/// Returns exact sub-cycle counts [num_cycles(sigma^k) for k in STRIDES].
fn verify_power_cycle_splitting(sigma: &[u32]) -> (Vec<usize>, [usize; 7]) {
    let n = sigma.len();
    let (base_lens, base_cid) = decompose_cycles(sigma);

    let mut p2 = vec![0u32; n];
    let mut p3 = vec![0u32; n];
    let mut p4 = vec![0u32; n];
    let mut p6 = vec![0u32; n];
    let mut p8 = vec![0u32; n];
    let mut p12 = vec![0u32; n];
    let mut p16 = vec![0u32; n];

    compose_perm(sigma, sigma, &mut p2);
    compose_perm(&p2, sigma, &mut p3);
    compose_perm(&p2, &p2, &mut p4);
    compose_perm(&p4, &p2, &mut p6);
    compose_perm(&p4, &p4, &mut p8);
    compose_perm(&p8, &p4, &mut p12);
    compose_perm(&p8, &p8, &mut p16);

    let powers: [&[u32]; 7] = [&p2, &p3, &p4, &p6, &p8, &p12, &p16];
    let mut visited = vec![false; n];
    let mut sub_counts = vec![0usize; base_lens.len()];
    let mut total_subcycles = [0usize; 7];

    for (s_idx, (&k, pow_perm)) in STRIDES.iter().zip(powers.iter()).enumerate() {
        visited.fill(false);
        sub_counts.fill(0);
        let mut tot_seen = 0usize;
        let mut n_cyc_k = 0usize;
        for start in 0..n {
            if visited[start] {
                continue;
            }
            let parent = base_cid[start] as usize;
            let parent_len = base_lens[parent];
            let g = gcd_usize(parent_len, k);
            let expected_sublen = parent_len / g;

            let mut cur = start;
            let mut len = 0usize;
            while !visited[cur] {
                assert_eq!(
                    base_cid[cur] as usize, parent,
                    "sigma^{k} cycle crossed parent cycles!"
                );
                visited[cur] = true;
                cur = pow_perm[cur] as usize;
                len += 1;
            }
            assert_eq!(cur, start, "sigma^{k} did not close at start!");
            assert_eq!(
                len, expected_sublen,
                "sigma^{k} sub-cycle length {len} != L/gcd(L,k) = {expected_sublen} (L={parent_len})"
            );
            sub_counts[parent] += 1;
            tot_seen += len;
            n_cyc_k += 1;
        }
        assert_eq!(tot_seen, n, "sigma^{k} sub-cycles did not partition [0..n)!");
        let mut expected_total = 0usize;
        for (cid, &plen) in base_lens.iter().enumerate() {
            let g = gcd_usize(plen, k);
            assert_eq!(
                sub_counts[cid], g,
                "Parent cycle {cid} (L={plen}) split into {} != gcd(L,{k})={g} sub-cycles!",
                sub_counts[cid]
            );
            expected_total += g;
        }
        assert_eq!(n_cyc_k, expected_total);
        total_subcycles[s_idx] = n_cyc_k;
    }

    (base_lens, total_subcycles)
}

/// Exact Burnside (1911, Ch. XV §145) orbit-counting lemma & Frobenius (1900) hook-character
/// generating polynomial verification on sigma^k for small permutations.
fn verify_burnside_and_frobenius_hooks(sigma: &[u32]) -> (usize, usize) {
    let n = sigma.len();
    let (base_lens, _) = decompose_cycles(sigma);
    let mut burnside_checks = 0usize;
    let mut hook_checks = 0usize;

    let mut cur_pow = (0..n as u32).collect::<Vec<u32>>();
    let mut nxt_pow = vec![0u32; n];
    for k in 1..=16usize {
        compose_perm(sigma, &cur_pow, &mut nxt_pow);
        std::mem::swap(&mut cur_pow, &mut nxt_pow);
        if !STRIDES.contains(&k) {
            continue;
        }
        let (k_lens, _) = decompose_cycles(&cur_pow);
        let predicted_cyc: usize = base_lens.iter().map(|&l| gcd_usize(l, k)).sum();
        assert_eq!(k_lens.len(), predicted_cyc);

        // 1) Burnside (1911, Ch. XV §145) orbit-counting trace average for H = <sigma^k>:
        //    sum_{t=0}^{ord-1} fix((sigma^k)^t) == ord * #cycles(sigma^k)
        let mut ord = 1usize;
        for &l in &k_lens {
            ord = lcm_usize(ord, l);
            if ord > 4096 {
                break;
            }
        }
        if ord <= 4096 {
            let mut sum_fix = 0usize;
            let mut h_t: Vec<u32> = (0..n as u32).collect();
            let mut tmp = vec![0u32; n];
            for _t in 0..ord {
                let fx = h_t.iter().enumerate().filter(|&(i, &v)| v as usize == i).count();
                sum_fix += fx;
                compose_perm(&cur_pow, &h_t, &mut tmp);
                std::mem::swap(&mut h_t, &mut tmp);
            }
            assert_eq!(h_t, (0..n as u32).collect::<Vec<u32>>(), "Order check failed");
            assert_eq!(
                sum_fix,
                ord * predicted_cyc,
                "Burnside (1911) orbit-counting lemma failed!"
            );
            burnside_checks += 1;
        }

        // 2) Frobenius (1900) hook-character generating polynomial on tau = sigma^k (n <= 20):
        //    P(t) = prod_{j} (1 - (-t)^{c_j}) = (1 + t) * sum_{r=0}^{n-1} chi^{(n-r, 1^r)}(tau) t^r
        //    Verify: (a) (1 + t) divides P(t) with rem == 0,
        //            (b) chi^{(n)} = 1, chi^{(n-1, 1)} = fix(tau) - 1, chi^{(1^n)} = sgn(tau),
        //            (c) exact match with predicted Frobenius polynomial prod_C (1 - (-t)^{L/g})^g!
        if n <= 20 {
            let poly_from_lens = |lens: &[usize]| -> Vec<i64> {
                let mut poly = vec![1i64];
                for &c in lens {
                    let mut next = vec![0i64; poly.len() + c];
                    let coeff = if c % 2 == 0 { -1i64 } else { 1i64 }; // -(-t)^c = -(-1)^c t^c
                    for (deg, &val) in poly.iter().enumerate() {
                        next[deg] += val;
                        next[deg + c] += val * coeff;
                    }
                    poly = next;
                }
                poly
            };
            let p_actual = poly_from_lens(&k_lens);
            let mut pred_lens = Vec::new();
            for &l in &base_lens {
                let g = gcd_usize(l, k);
                for _ in 0..g {
                    pred_lens.push(l / g);
                }
            }
            let p_pred = poly_from_lens(&pred_lens);
            assert_eq!(p_actual, p_pred, "Frobenius (1900) hook-character polynomial mismatch!");

            // Divide P(t) by (1 + t) exactly -> hook characters chi^{(n-r, 1^r)}(tau)
            let mut chi = vec![0i64; n];
            let mut rem = p_actual.clone();
            for r in 0..n {
                let c = rem[r];
                chi[r] = c;
                rem[r] -= c;
                rem[r + 1] -= c;
            }
            assert_eq!(rem[n], 0, "(1 + t) did not divide Frobenius hook polynomial!");
            let fix_tau = k_lens.iter().filter(|&&c| c == 1).count() as i64;
            let sgn_tau = if (n - k_lens.len()) % 2 == 0 { 1i64 } else { -1i64 };
            assert_eq!(chi[0], 1);
            if n >= 2 {
                assert_eq!(chi[1], fix_tau - 1);
            }
            assert_eq!(chi[n - 1], sgn_tau);
            hook_checks += 1;
        }
    }
    (burnside_checks, hook_checks)
}

// ============================================================================
// SECTION 5: PART 3 — Self-Contained Open<16> Kernel with Multi-Cursor S5 Arms
//            (Base-2Cur, Q-2Cur-Pipe1, Q-2Cur-Batch8, Q-4Cur-Stage, Q-4Cur-Pipe1)
// ============================================================================

mod outq {
    const CHUNK: usize = 512;
    const BYTES: usize = 2 * CHUNK;
    const NONE: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;

    #[derive(Default)]
    pub(super) struct Spill {
        next: Vec<u32>,
        ovf: Vec<[u8; BYTES]>,
        cuts: Vec<[u32; 256]>,
    }

    impl Spill {
        pub(super) fn scratch_bytes(&self) -> usize {
            4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
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
                heads: [NONE; 256],
                pt: [u32::MAX as u64; 256],
                limit,
                over: false,
            }
        }
        pub(super) fn heads(&self) -> Heads {
            Heads { heads: self.heads, in_out: self.in_out }
        }
        #[inline(always)]
        pub(super) fn thin_of(&self, a: usize) -> u32 {
            (self.pt[a] >> 32) as u32
        }
        pub(super) fn extract_thin(&self) -> [u32; 256] {
            std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
        }
        #[inline(always)]
        pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
            let v = self.pt[a];
            let p = v as u32;
            let f = (p & 1023) as usize;
            if (w <= 16) & (f <= CHUNK - 16) & ((m as u16) < 65_520) {
                let at = p as usize + f;
                if let Some(dst) = self.out.get_mut(at..at + 32) {
                    let lo = m as u16;
                    let iota: [u16; 16] = std::array::from_fn(|i| lo.wrapping_add(i as u16));
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
                    self.take_chunk_pt(a);
                }
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & 1023) as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(p >> 10);
                for (i, d) in
                    chunk[2 * f..2 * (f + take as usize)].chunks_exact_mut(2).enumerate()
                {
                    d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                }
                let tu = u64::from(take);
                self.pt[a] = v + ((tu << 32) | tu);
                m += take;
                left -= take;
                if m % EPOCH == 0 {
                    let cut: [u32; 256] = std::array::from_fn(|b| (self.pt[b] >> 32) as u32);
                    self.spill.cuts.push(cut);
                }
            }
        }
        fn take_chunk_pt(&mut self, a: usize) {
            let id = self.chunks;
            self.chunks += 1;
            if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                self.spill.ovf.push([0; BYTES]);
            }
            if self.spill.next.len() <= id as usize {
                self.spill.next.resize(id as usize + 1, NONE);
            }
            self.spill.next[id as usize] = NONE;
            let hi = self.pt[a] & !0xFFFF_FFFFu64;
            if self.heads[a] == NONE {
                self.heads[a] = id;
            } else {
                self.spill.next[((self.pt[a] as u32) >> 10) as usize] = id;
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
        fn chunk(&self) -> &'a [u8] {
            if self.id < self.in_out {
                let at = self.id as usize * BYTES;
                &self.out[at..at + BYTES]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][..]
            }
        }
        /// Peek the very next rank in this byte queue WITHOUT consuming it (O(1), ~1 ns).
        /// Used by Q-4Cur S4 anchor capture with ZERO inner-loop overhead!
        #[inline(always)]
        pub(super) fn peek_next_rank(&self) -> u32 {
            let mut ep = self.epoch;
            let mut ct = self.cut;
            while self.read == ct {
                ep += 1;
                ct = self.spill.cuts.get(ep as usize).map_or(u32::MAX, |c| c[self.a]);
            }
            let id = if self.at == CHUNK {
                self.spill.next[self.id as usize]
            } else {
                self.id
            };
            let at = if self.at == CHUNK { 0 } else { self.at };
            let ch = if id < self.in_out {
                let off = id as usize * BYTES;
                &self.out[off..off + BYTES]
            } else {
                &self.spill.ovf[(id - self.in_out) as usize][..]
            };
            (ep << 16) | u32::from(u16::from_le_bytes([ch[2 * at], ch[2 * at + 1]]))
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
                let bytes = &self.out[at..at + 2 * span];
                let mut i = 0usize;
                while i + 16 <= span {
                    let b = &bytes[2 * i..2 * i + 32];
                    let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                    let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                    let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                    let v0 = hi | (q0 as u32 & 0xFFFF);
                    let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q0 >> 48) as u32);
                    let v4 = hi | (q1 as u32 & 0xFFFF);
                    let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                    let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                    let v7 = hi | ((q1 >> 48) as u32);
                    let v8 = hi | (q2 as u32 & 0xFFFF);
                    let v9 = hi | ((q2 >> 16) as u32 & 0xFFFF);
                    let v10 = hi | ((q2 >> 32) as u32 & 0xFFFF);
                    let v11 = hi | ((q2 >> 48) as u32);
                    let v12 = hi | (q3 as u32 & 0xFFFF);
                    let v13 = hi | ((q3 >> 16) as u32 & 0xFFFF);
                    let v14 = hi | ((q3 >> 32) as u32 & 0xFFFF);
                    let v15 = hi | ((q3 >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 16];
                    eu[0] ^= pack | v0;   eu[1] ^= pack | v1;   eu[2] ^= pack | v2;   eu[3] ^= pack | v3;
                    eu[4] ^= pack | v4;   eu[5] ^= pack | v5;   eu[6] ^= pack | v6;   eu[7] ^= pack | v7;
                    eu[8] ^= pack | v8;   eu[9] ^= pack | v9;   eu[10] ^= pack | v10; eu[11] ^= pack | v11;
                    eu[12] ^= pack | v12; eu[13] ^= pack | v13; eu[14] ^= pack | v14; eu[15] ^= pack | v15;
                    edges[v0 as usize] ^= u0 as u32;        edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32;  edges[v3 as usize] ^= (u0 + 3) as u32;
                    edges[v4 as usize] ^= (u0 + 4) as u32;  edges[v5 as usize] ^= (u0 + 5) as u32;
                    edges[v6 as usize] ^= (u0 + 6) as u32;  edges[v7 as usize] ^= (u0 + 7) as u32;
                    edges[v8 as usize] ^= (u0 + 8) as u32;  edges[v9 as usize] ^= (u0 + 9) as u32;
                    edges[v10 as usize] ^= (u0 + 10) as u32; edges[v11 as usize] ^= (u0 + 11) as u32;
                    edges[v12 as usize] ^= (u0 + 12) as u32; edges[v13 as usize] ^= (u0 + 13) as u32;
                    edges[v14 as usize] ^= (u0 + 14) as u32; edges[v15 as usize] ^= (u0 + 15) as u32;
                    i += 16;
                }
                if i + 8 <= span {
                    let b = &bytes[2 * i..2 * i + 16];
                    let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                    let v0 = hi | (q0 as u32 & 0xFFFF);
                    let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q0 >> 48) as u32);
                    let v4 = hi | (q1 as u32 & 0xFFFF);
                    let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                    let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                    let v7 = hi | ((q1 >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 8];
                    eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                    eu[4] ^= pack | v4; eu[5] ^= pack | v5; eu[6] ^= pack | v6; eu[7] ^= pack | v7;
                    edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                    edges[v4 as usize] ^= (u0 + 4) as u32; edges[v5 as usize] ^= (u0 + 5) as u32;
                    edges[v6 as usize] ^= (u0 + 6) as u32; edges[v7 as usize] ^= (u0 + 7) as u32;
                    i += 8;
                }
                if i + 4 <= span {
                    let b = &bytes[2 * i..2 * i + 8];
                    let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let v0 = hi | (q as u32 & 0xFFFF);
                    let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 4];
                    eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                    edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                    i += 4;
                }
                while i < span {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                    i += 1;
                }
                self.at += span;
                self.read += span as u32;
            } else {
                let mut dummy = 0u32;
                self.wire_xor_slow(edges, dst_start, span, pack, u32::MAX, &mut dummy);
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
                let bytes = &self.out[at..at + 2 * span];
                let mut before = *predecessor;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        before = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                *predecessor = before;
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
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let hi = self.epoch << 16;
                let bytes = &self.chunk()[2 * self.at..2 * (self.at + take)];
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        *predecessor = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                dst_start += take;
                left -= take;
                self.at += take;
                self.read += take as u32;
            }
        }
        #[inline(always)]
        pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {
            let mut off = 0usize;
            while off < dst.len() {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = (dst.len() - off)
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let hi = self.epoch << 16;
                let bytes = &self.chunk()[2 * self.at..2 * (self.at + take)];
                for (d, s) in dst[off..off + take].iter_mut().zip(bytes.chunks_exact(2)) {
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    *d = pack | v;
                }
                off += take;
                self.at += take;
                self.read += take as u32;
            }
        }
    }
}

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

struct Elim<'a, L> {
    a: &'a mut [u32],
    nodes: u32,
    base: usize,
    top: usize,
    _l: std::marker::PhantomData<L>,
}

impl<'a, L: Links> Elim<'a, L> {
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

fn repeat_period(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

#[inline(always)]
fn emit_fwd_word(
    code: usize,
    inline: &[[u8; INLINE]],
    lens: &[u8],
    spelling: &[u8],
    buf: &mut [u8],
    lo: &mut usize,
    hi: usize,
) {
    let lf = lens[code];
    if lf != LONG && hi - *lo >= INLINE {
        buf[*lo..*lo + INLINE].copy_from_slice(&inline[code]);
        *lo += usize::from(lf);
    } else {
        let word = if lf == LONG {
            long_word(&inline[code], spelling)
        } else {
            &inline[code][..usize::from(lf)]
        };
        let take = word.len().min(hi - *lo);
        buf[*lo..*lo + take].copy_from_slice(&word[..take]);
        *lo += take;
    }
}

#[inline(always)]
fn emit_bwd_word(
    code: usize,
    inline: &[[u8; INLINE]],
    lens: &[u8],
    spelling: &[u8],
    buf: &mut [u8],
    lo: usize,
    hi: &mut usize,
) {
    let lb = lens[code];
    if lb != LONG && *hi - lo >= INLINE {
        let shifted = u128::from_le_bytes(inline[code]) << (8 * (INLINE - usize::from(lb)));
        buf[*hi - INLINE..*hi].copy_from_slice(&shifted.to_le_bytes());
        *hi -= usize::from(lb);
    } else {
        let word = if lb == LONG {
            long_word(&inline[code], spelling)
        } else {
            &inline[code][..usize::from(lb)]
        };
        let take = word.len().min(*hi - lo);
        buf[*hi - take..*hi].copy_from_slice(&word[word.len() - take..]);
        *hi -= take;
    }
}

/// MODE 0: Exact baseline Open<16> `emit_bidirectional` (2 cursors, +0 B WS).
#[allow(clippy::too_many_arguments)]
fn emit_2cur_base(
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
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
    for pair in 0..=edges.len() / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = edges[front as usize];
        let cf = (ef >> bits) as usize;
        let single = front == back;
        emit_fwd_word(cf, inline, lens, spelling, out, &mut lo, hi);
        if single {
            break;
        }
        let eb = edges[back as usize];
        let cb = (eb >> bits) as usize;
        emit_bwd_word(cb, inline, lens, spelling, out, lo, &mut hi);
        (prev, front) = (front, (ef & mask) ^ prev);
        (next, back) = (back, (eb & mask) ^ next);
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat_period(n, out);
    } else {
        out.truncate(n);
    }
}

/// MODE 1: `Q-2Cur-Pipe1` — 1-Step Software-Pipelined Decoupled 2-Lane XOR (+0 B WS!).
/// Loads BOTH `edges[front]` and `edges[back]` hoisted, computes `f_next`/`b_next` immediately,
/// and issues `edges[f_next]` + `edges[b_next]` loads BEFORE emitting current words into `out`,
/// overlapping next-step L2/L3 cache misses with current-step dictionary lookup & stores!
#[allow(clippy::too_many_arguments)]
fn emit_2cur_pipe1(
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
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);

    if front == back {
        let cf = (edges[front as usize] >> bits) as usize;
        emit_fwd_word(cf, inline, lens, spelling, out, &mut lo, hi);
    } else {
        let mut ef = edges[front as usize];
        let mut eb = edges[back as usize];
        for pair in 0..=edges.len() / 2 {
            if lo == hi || (pair != 0 && front == next) {
                break;
            }
            if front == back {
                let cf = (ef >> bits) as usize;
                emit_fwd_word(cf, inline, lens, spelling, out, &mut lo, hi);
                break;
            }
            let f_next = (ef & mask) ^ prev;
            let b_next = (eb & mask) ^ next;
            // Issue next-iteration pointer-chase loads BEFORE emitting current words!
            let ef_nxt = edges[f_next as usize];
            let eb_nxt = edges[b_next as usize];

            let cf = (ef >> bits) as usize;
            let cb = (eb >> bits) as usize;
            emit_fwd_word(cf, inline, lens, spelling, out, &mut lo, hi);
            emit_bwd_word(cb, inline, lens, spelling, out, lo, &mut hi);

            prev = front;
            front = f_next;
            ef = ef_nxt;
            next = back;
            back = b_next;
            eb = eb_nxt;
        }
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat_period(n, out);
    } else {
        out.truncate(n);
    }
}

/// MODE 5: `Q-2Cur-Hoist` — Hoists BOTH `edges[front]` and `edges[back]` before emitting
/// current words (without 1-step next-iteration prefetch), +0 B WS, to ablate Hoist vs Pipe1.
#[allow(clippy::too_many_arguments)]
fn emit_2cur_hoist(
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
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
    for pair in 0..=edges.len() / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = edges[front as usize];
        let single = front == back;
        if single {
            emit_fwd_word((ef >> bits) as usize, inline, lens, spelling, out, &mut lo, hi);
            break;
        }
        let eb = edges[back as usize];
        let f_next = (ef & mask) ^ prev;
        let b_next = (eb & mask) ^ next;
        emit_fwd_word((ef >> bits) as usize, inline, lens, spelling, out, &mut lo, hi);
        emit_bwd_word((eb >> bits) as usize, inline, lens, spelling, out, lo, &mut hi);
        prev = front;
        front = f_next;
        next = back;
        back = b_next;
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat_period(n, out);
    } else {
        out.truncate(n);
    }
}

/// MODE 2: `Q-2Cur-Batch8` — Decoupled 2-Lane XOR Chase-8 / Emit-8 (+64 B stack, +0 B heap WS).
/// Separates 8 pairs of pointer-chasing from 8 pairs of dictionary/output emission.
#[allow(clippy::too_many_arguments)]
fn emit_2cur_batch8(
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
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
    let max_pairs = edges.len() / 2 + 1;
    let mut done_pairs = 0usize;
    let mut cf_buf = [0u32; 8];
    let mut cb_buf = [0u32; 8];

    'outer: while done_pairs < max_pairs && lo < hi {
        let mut cnt = 0usize;
        let mut odd_single: Option<u32> = None;
        while cnt < 8 && done_pairs < max_pairs {
            if done_pairs != 0 && front == next {
                break;
            }
            let ef = edges[front as usize];
            if front == back {
                odd_single = Some(ef >> bits);
                break;
            }
            let eb = edges[back as usize];
            cf_buf[cnt] = ef >> bits;
            cb_buf[cnt] = eb >> bits;
            (prev, front) = (front, (ef & mask) ^ prev);
            (next, back) = (back, (eb & mask) ^ next);
            cnt += 1;
            done_pairs += 1;
        }
        for k in 0..cnt {
            if lo == hi {
                break 'outer;
            }
            emit_fwd_word(cf_buf[k] as usize, inline, lens, spelling, out, &mut lo, hi);
            emit_bwd_word(cb_buf[k] as usize, inline, lens, spelling, out, lo, &mut hi);
        }
        if let Some(cf) = odd_single {
            if lo < hi {
                emit_fwd_word(cf as usize, inline, lens, spelling, out, &mut lo, hi);
            }
            break;
        }
        if cnt < 8 {
            break;
        }
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat_period(n, out);
    } else {
        out.truncate(n);
    }
}

/// MODE 3 (`PIPE=false`: `Q-4Cur-Stage`) & MODE 4 (`PIPE=true`: `Q-4Cur-Pipe1`):
/// True 4-Cursor XOR Cycle Partitioning with S4-captured midpoint anchor `(a1, a1_before)`
/// and exact detached-cycle detection for arbitrary malformed multi-cycle BWTs!
#[allow(clippy::too_many_arguments)]
fn emit_4cur_stage<const PIPE: bool>(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    before: u32,
    anchor1: Option<(u32, u32)>, // (a1, a1_before) with pi_ret[a1_before] == a1
    bits: u32,
    out: &mut Vec<u8>,
    stage: &mut Vec<u8>,
) -> (bool, usize, usize) {
    let m = edges.len();
    let (a1, a1_before) = match anchor1 {
        Some((x, xb))
            if m >= 8
                && mark != before
                && x != xb
                && x != mark
                && x != before
                && xb != mark
                && xb != before =>
        {
            (x, xb)
        }
        _ => {
            emit_2cur_base(edges, spelling, inline, lens, n, mark, before, bits, out);
            return (false, m, 0);
        }
    };

    out.resize(n + INLINE, 0);
    let need_stage = n + INLINE;
    if stage.capacity() < need_stage {
        stage.clear();
        stage.reserve_exact(need_stage);
    }
    stage.resize(need_stage, 0);
    let mask = (1u32 << bits) - 1;

    // 4 cursors:
    //   f0: mark -> ... (fwd into out[lo0..])
    //   b0: before -> ... (bwd into out[..hi0])
    //   f1: a1 -> ... (fwd into stage[lo1..])
    //   b1: a1_before -> ... (bwd into stage[..hi1])
    let (mut p0, mut f0) = (before, mark);
    let (mut n0, mut b0) = (mark, before);
    let (mut p1, mut f1) = (a1_before, a1);
    let (mut n1, mut b1) = (a1, a1_before);
    let (mut lo0, mut hi0) = (0usize, n);
    let (mut lo1, mut hi1) = (0usize, n);

    let mut a_done = false; // Arc A: f0 <-> b1
    let mut b_done = false; // Arc B: f1 <-> b0
    let mut detached = false; // True if mark & a1 are on DIFFERENT cycles (malformed!)
    let mut steps_4cur = 0usize;
    let max_steps = m / 2 + 2;

    // PHASE 1: 4-Cursor Hot Loop (4x MLP on edges[..])
    if PIPE {
        let mut ef0 = edges[f0 as usize];
        let mut eb0 = edges[b0 as usize];
        let mut ef1 = edges[f1 as usize];
        let mut eb1 = edges[b1 as usize];
        for step in 0..max_steps {
            if step != 0 {
                if f0 == n0 || f1 == n1 {
                    detached = true;
                    break;
                }
                if f0 == n1 {
                    a_done = true;
                }
                if f1 == n0 {
                    b_done = true;
                }
                if a_done || b_done {
                    break;
                }
            }
            if f0 == b0 || f1 == b1 {
                detached = true;
                break;
            }
            if f0 == b1 || f1 == b0 {
                if f0 == b1 {
                    emit_fwd_word((ef0 >> bits) as usize, inline, lens, spelling, out, &mut lo0, hi0);
                    a_done = true;
                }
                if f1 == b0 {
                    emit_fwd_word((ef1 >> bits) as usize, inline, lens, spelling, stage, &mut lo1, hi1);
                    b_done = true;
                }
                break;
            }
            if lo0 >= hi0 || lo1 >= hi1 {
                break;
            }
            let f0_n = (ef0 & mask) ^ p0;
            let b0_n = (eb0 & mask) ^ n0;
            let f1_n = (ef1 & mask) ^ p1;
            let b1_n = (eb1 & mask) ^ n1;
            // 4-way pipelined prefetch loads!
            let ef0_n = edges[f0_n as usize];
            let eb0_n = edges[b0_n as usize];
            let ef1_n = edges[f1_n as usize];
            let eb1_n = edges[b1_n as usize];

            emit_fwd_word((ef0 >> bits) as usize, inline, lens, spelling, out, &mut lo0, hi0);
            emit_bwd_word((eb0 >> bits) as usize, inline, lens, spelling, out, lo0, &mut hi0);
            emit_fwd_word((ef1 >> bits) as usize, inline, lens, spelling, stage, &mut lo1, hi1);
            emit_bwd_word((eb1 >> bits) as usize, inline, lens, spelling, stage, lo1, &mut hi1);

            p0 = f0; f0 = f0_n; ef0 = ef0_n;
            n0 = b0; b0 = b0_n; eb0 = eb0_n;
            p1 = f1; f1 = f1_n; ef1 = ef1_n;
            n1 = b1; b1 = b1_n; eb1 = eb1_n;
            steps_4cur += 1;
        }
    } else {
        for step in 0..max_steps {
            if step != 0 {
                if f0 == n0 || f1 == n1 {
                    detached = true;
                    break;
                }
                if f0 == n1 {
                    a_done = true;
                }
                if f1 == n0 {
                    b_done = true;
                }
                if a_done || b_done {
                    break;
                }
            }
            if f0 == b0 || f1 == b1 {
                detached = true;
                break;
            }
            if f0 == b1 || f1 == b0 {
                if f0 == b1 {
                    let ef0 = edges[f0 as usize];
                    emit_fwd_word((ef0 >> bits) as usize, inline, lens, spelling, out, &mut lo0, hi0);
                    a_done = true;
                }
                if f1 == b0 {
                    let ef1 = edges[f1 as usize];
                    emit_fwd_word((ef1 >> bits) as usize, inline, lens, spelling, stage, &mut lo1, hi1);
                    b_done = true;
                }
                break;
            }
            if lo0 >= hi0 || lo1 >= hi1 {
                break;
            }
            let ef0 = edges[f0 as usize];
            let eb0 = edges[b0 as usize];
            let ef1 = edges[f1 as usize];
            let eb1 = edges[b1 as usize];

            emit_fwd_word((ef0 >> bits) as usize, inline, lens, spelling, out, &mut lo0, hi0);
            emit_bwd_word((eb0 >> bits) as usize, inline, lens, spelling, out, lo0, &mut hi0);
            emit_fwd_word((ef1 >> bits) as usize, inline, lens, spelling, stage, &mut lo1, hi1);
            emit_bwd_word((eb1 >> bits) as usize, inline, lens, spelling, stage, lo1, &mut hi1);

            (p0, f0) = (f0, (ef0 & mask) ^ p0);
            (n0, b0) = (b0, (eb0 & mask) ^ n0);
            (p1, f1) = (f1, (ef1 & mask) ^ p1);
            (n1, b1) = (b1, (eb1 & mask) ^ n1);
            steps_4cur += 1;
        }
    }

    // PHASE 2: Finish remaining 2 cursors
    if detached {
        // Malformed detached cycle: discard `stage` completely and finish mark's cycle with (f0, b0)!
        let mut steps_2cur = 0usize;
        for _ in 0..max_steps {
            if lo0 >= hi0 || f0 == n0 {
                break;
            }
            let ef0 = edges[f0 as usize];
            let single = f0 == b0;
            emit_fwd_word((ef0 >> bits) as usize, inline, lens, spelling, out, &mut lo0, hi0);
            if single {
                break;
            }
            let eb0 = edges[b0 as usize];
            emit_bwd_word((eb0 >> bits) as usize, inline, lens, spelling, out, lo0, &mut hi0);
            (p0, f0) = (f0, (ef0 & mask) ^ p0);
            (n0, b0) = (b0, (eb0 & mask) ^ n0);
            steps_2cur += 1;
        }
        if lo0 < hi0 {
            out.copy_within(hi0..n, lo0);
            out.truncate(lo0 + (n - hi0));
            repeat_period(n, out);
        } else {
            out.truncate(n);
        }
        return (false, steps_4cur, steps_2cur);
    }

    let mut steps_2cur = 0usize;
    if !a_done {
        // Finish Arc A: f0 (into out[lo0..hi0]) <-> b1 (into stage[lo1..hi1])
        for _ in 0..max_steps {
            if f0 == n1 || lo0 >= hi0 || lo1 >= hi1 {
                break;
            }
            let ef0 = edges[f0 as usize];
            let single = f0 == b1;
            emit_fwd_word((ef0 >> bits) as usize, inline, lens, spelling, out, &mut lo0, hi0);
            if single {
                break;
            }
            let eb1 = edges[b1 as usize];
            emit_bwd_word((eb1 >> bits) as usize, inline, lens, spelling, stage, lo1, &mut hi1);
            (p0, f0) = (f0, (ef0 & mask) ^ p0);
            (n1, b1) = (b1, (eb1 & mask) ^ n1);
            steps_2cur += 1;
        }
    }
    if !b_done {
        // Finish Arc B: f1 (into stage[lo1..hi1]) <-> b0 (into out[lo0..hi0])
        for _ in 0..max_steps {
            if f1 == n0 || lo0 >= hi0 || lo1 >= hi1 {
                break;
            }
            let ef1 = edges[f1 as usize];
            let single = f1 == b0;
            emit_fwd_word((ef1 >> bits) as usize, inline, lens, spelling, stage, &mut lo1, hi1);
            if single {
                break;
            }
            let eb0 = edges[b0 as usize];
            emit_bwd_word((eb0 >> bits) as usize, inline, lens, spelling, out, lo0, &mut hi0);
            (p1, f1) = (f1, (ef1 & mask) ^ p1);
            (n0, b0) = (b0, (eb0 & mask) ^ n0);
            steps_2cur += 1;
        }
    }

    // Stitch Arc A suffix (stage[hi1..n]) + Arc B prefix (stage[0..lo1]) into out[lo0..]
    let arc_a_suf = n - hi1;
    let arc_b_pre = lo1;
    let arc_b_suf = n - hi0;
    let tot = lo0 + arc_a_suf + arc_b_pre + arc_b_suf;
    if tot == n {
        out[lo0..lo0 + arc_a_suf].copy_from_slice(&stage[hi1..n]);
        out[lo0 + arc_a_suf..lo0 + arc_a_suf + arc_b_pre].copy_from_slice(&stage[0..lo1]);
        debug_assert_eq!(lo0 + arc_a_suf + arc_b_pre, hi0);
        out.truncate(n);
    } else if tot < n {
        // Short single cycle (periodic text): assemble contiguous period then repeat(n)
        out[lo0..lo0 + arc_a_suf].copy_from_slice(&stage[hi1..n]);
        out[lo0 + arc_a_suf..lo0 + arc_a_suf + arc_b_pre].copy_from_slice(&stage[0..lo1]);
        let dst_b = lo0 + arc_a_suf + arc_b_pre;
        out.copy_within(hi0..n, dst_b);
        out.truncate(tot);
        repeat_period(n, out);
    } else {
        // Defensive fallback (should be unreachable on valid/malformed Kakutani cycles)
        emit_2cur_base(edges, spelling, inline, lens, n, mark, before, bits, out);
    }
    (true, steps_4cur, steps_2cur)
}

/// Offline Oracle 4-Cursor Emit (`0 B` staging, `0` copy tax, exact balanced midpoint `mid = pi^{m/2}(mark)`
/// and exact byte offset `mid_byte`) — strictly diagnostic ceiling bound!
#[allow(clippy::too_many_arguments)]
fn emit_4cur_oracle(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    before: u32,
    mid: u32,
    mid_before: u32,
    mid_byte: usize,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let (mut p0, mut f0) = (before, mark);
    let (mut n0, mut b0) = (mark, before);
    let (mut p1, mut f1) = (mid_before, mid);
    let (mut n1, mut b1) = (mid, mid_before);
    let (mut lo0, mut hi0) = (0usize, mid_byte);
    let (mut lo1, mut hi1) = (mid_byte, n);

    let mut a_done = false;
    let mut b_done = false;
    let max_steps = edges.len() / 2 + 2;
    for step in 0..max_steps {
        if step != 0 {
            if f0 == n1 {
                a_done = true;
            }
            if f1 == n0 {
                b_done = true;
            }
            if a_done || b_done {
                break;
            }
        }
        if f0 == b1 || f1 == b0 {
            if f0 == b1 {
                let ef0 = edges[f0 as usize];
                emit_fwd_word((ef0 >> bits) as usize, inline, lens, spelling, out, &mut lo0, hi0);
                a_done = true;
            }
            if f1 == b0 {
                let ef1 = edges[f1 as usize];
                emit_fwd_word((ef1 >> bits) as usize, inline, lens, spelling, out, &mut lo1, hi1);
                b_done = true;
            }
            break;
        }
        let ef0 = edges[f0 as usize];
        let eb1 = edges[b1 as usize];
        let ef1 = edges[f1 as usize];
        let eb0 = edges[b0 as usize];
        emit_fwd_word((ef0 >> bits) as usize, inline, lens, spelling, out, &mut lo0, hi0);
        emit_bwd_word((eb1 >> bits) as usize, inline, lens, spelling, out, lo0, &mut hi0);
        emit_fwd_word((ef1 >> bits) as usize, inline, lens, spelling, out, &mut lo1, hi1);
        emit_bwd_word((eb0 >> bits) as usize, inline, lens, spelling, out, lo1, &mut hi1);
        (p0, f0) = (f0, (ef0 & mask) ^ p0);
        (n1, b1) = (b1, (eb1 & mask) ^ n1);
        (p1, f1) = (f1, (ef1 & mask) ^ p1);
        (n0, b0) = (b0, (eb0 & mask) ^ n0);
    }
    if !a_done {
        for _ in 0..max_steps {
            if f0 == n1 {
                break;
            }
            let ef0 = edges[f0 as usize];
            let single = f0 == b1;
            emit_fwd_word((ef0 >> bits) as usize, inline, lens, spelling, out, &mut lo0, hi0);
            if single {
                break;
            }
            let eb1 = edges[b1 as usize];
            emit_bwd_word((eb1 >> bits) as usize, inline, lens, spelling, out, lo0, &mut hi0);
            (p0, f0) = (f0, (ef0 & mask) ^ p0);
            (n1, b1) = (b1, (eb1 & mask) ^ n1);
        }
    }
    if !b_done {
        for _ in 0..max_steps {
            if f1 == n0 {
                break;
            }
            let ef1 = edges[f1 as usize];
            let single = f1 == b0;
            emit_fwd_word((ef1 >> bits) as usize, inline, lens, spelling, out, &mut lo1, hi1);
            if single {
                break;
            }
            let eb0 = edges[b0 as usize];
            emit_bwd_word((eb0 >> bits) as usize, inline, lens, spelling, out, lo1, &mut hi1);
            (p1, f1) = (f1, (ef1 & mask) ^ p1);
            (n0, b0) = (b0, (eb0 & mask) ^ n0);
        }
    }
    out.truncate(n);
}

#[derive(Default, Clone, Copy, Debug)]
struct QStageTimes {
    s123_us: f64,
    s4_us: f64,
    s5_us: f64,
    steps_4cur: u64,
    steps_2cur: u64,
    same_cyc_hits: u64,
}

/// Self-contained Open<16> kernel supporting 5 S5 modes:
///   MODE = 0: Base-2Cur     (baseline Open<16> emit_bidirectional, 2 cursors, +0 B WS)
///   MODE = 1: Q-2Cur-Pipe1  (1-step software-pipelined 2-lane XOR, +0 B WS)
///   MODE = 2: Q-2Cur-Batch8 (decoupled 2-lane chase-8 / emit-8, +0 B heap WS)
///   MODE = 3: Q-4Cur-Stage  (S4 midpoint anchor + 4-cursor + stage buffer)
///   MODE = 4: Q-4Cur-Pipe1  (S4 midpoint anchor + 4-cursor 1-step pipelined + stage buffer)
#[derive(Default)]
struct OpenQKernel {
    spill: outq::Spill,
    edges: Vec<u32>,
    extra: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    stage: Vec<u8>,
}

impl OpenQKernel {
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
        self.spill.scratch_bytes()
            + self.edges.capacity() * 4
            + self.extra.capacity() * 4
            + self.bands.capacity() * 8
            + self.spelling.capacity()
            + self.inline16.capacity() * INLINE
            + self.lens.capacity()
            + self.stack.capacity() * 4
            + self.stage.capacity()
    }

    #[allow(clippy::too_many_arguments)]
    fn induced<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        m: u32,
    ) -> [usize; 256] {
        let base_hdr = 4 * ns;
        let mut remaining = n;
        let mut widest = 0u32;
        let (nodes, base, mut top);
        {
            let (hdr, arena) = self.edges.split_at_mut(base_hdr);
            let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
            let mut e = Elim::<L> {
                a: arena,
                nodes: 0,
                base: 0,
                top: 0,
                _l: std::marker::PhantomData,
            };
            e.node(0, 0);
            e.nodes = ns as u32 + 1;
            let (mut terminals, mut order) = ([(0u32, 0u32); 2], [(0u32, 0u32); 2]);
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
                e.set_word(sid, (1 << ID_BITS) | u32::from(byte));
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
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            nodes = e.nodes;
            base = nodes as usize * L::STRIDE;
            top = base;
        }

        // S2b: Eliminate with dynamic arena growth if needed
        let mut spirals = widest > CERTIFIED_WIDTH;
        loop {
            let done = {
                let arena = &mut self.edges[base_hdr..];
                let mut e = Elim::<L> {
                    a: arena,
                    nodes,
                    base,
                    top,
                    _l: std::marker::PhantomData,
                };
                let ok = if spirals {
                    e.eliminate::<true>(&mut remaining)
                } else {
                    e.eliminate::<false>(&mut remaining)
                };
                top = e.top;
                ok
            };
            if done {
                break;
            }
            let room = if spirals { SPIRAL_RULES * RULE } else { RULE };
            let need = base_hdr + top + room;
            grow_u32(&mut self.edges, need);
            if !spirals && remaining > 0 && top - base > 8 * (nodes as usize) * RULE {
                spirals = true;
            }
        }

        // S3a: Assign bottom coordinates & emit bands + inline16 dictionary
        begin_inline(0, &mut self.inline16, &mut self.lens);
        let mut pairs = [0u32; 512];
        let mut contents = [0u32; 2048];
        let mut cursors = [0usize; 256];
        {
            let arena = &mut self.edges[base_hdr..];
            let mut e = Elim::<L> {
                a: arena,
                nodes,
                base,
                top,
                _l: std::marker::PhantomData,
            };
            e.assign_bottom(m);
            let mut dict = Dictionary {
                pairs: &mut pairs,
                contents: &mut contents,
                inline16: &mut self.inline16,
                lens: &mut self.lens,
                spelling: &mut self.spelling,
                stack: &mut self.stack,
            };
            let mut id = L::next(e.a, 0, 0);
            let mut a = 0usize;
            while a < 256 && thin[a] == 0 {
                cursors[a] = 0;
                a += 1;
            }
            let mut left = if a < 256 { thin[a] } else { 0 };
            let mut emitted = 0u32;
            while id != 0 {
                let (next_id, mut dst, mut w, word) = L::band_node(e.a, id);
                while w > 0 {
                    let take = w.min(left);
                    let code = if word == 0 {
                        a as u32
                    } else {
                        dict.code(&mut e, word, a as u8)
                    };
                    self.bands.push(Band::new(take, dst, code));
                    dst += take;
                    w -= take;
                    left -= take;
                    emitted += take;
                    if left == 0 {
                        a += 1;
                        while a < 256 && thin[a] == 0 {
                            cursors[a] = self.bands.len();
                            a += 1;
                        }
                        if a < 256 {
                            cursors[a] = self.bands.len();
                            left = thin[a];
                        }
                    }
                }
                id = next_id;
            }
            assert_eq!(emitted, m);
            while a < 256 {
                cursors[a] = self.bands.len();
                a += 1;
            }
        }
        cursors
    }

    fn run<const MODE: u8, const XOR_MIN_M: usize, const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> QStageTimes {
        let mut tm = QStageTimes::default();
        out.resize(n + INLINE, 0);
        if n == 0 {
            out.truncate(0);
            return tm;
        }
        let t_s123 = if TIMED { Some(Instant::now()) } else { None };

        // S1: Run classification + OutQ16 queueing
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
        if n >= 65_536 {
            if self.bands.capacity() < 8192 {
                self.bands.reserve_exact(8192 - self.bands.len());
            }
            if self.spelling.capacity() < 49_152 {
                self.spelling.reserve_exact(49_152 - self.spelling.len());
            }
        }
        let cursors = if Self::wide(ns) {
            self.induced::<Wide>(ns, n as u32, &f.thin, &first, &mut write, f.m)
        } else {
            self.induced::<Narrow>(ns, n as u32, &f.thin, &first, &mut write, f.m)
        };
        if let Some(t) = t_s123 {
            tm.s123_us = t.elapsed().as_secs_f64() * 1e6;
        }

        // S4: Wire edges (with zero-inner-loop midpoint anchor capture for MODE 3 / MODE 4!)
        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
        let bits = (u32::BITS - (f.m.saturating_sub(1)).leading_zeros()).max(1);
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
        let use_xor = !wide && m >= XOR_MIN_M;
        let mut predecessor = mark;
        let mut anchor1: Option<(u32, u32)> = None;

        if use_xor {
            // For MODE 3 / MODE 4: pick byte `a_mid` whose cumulative thin crosses m/2 (preferring a_mid != mb).
            let a_mid = if MODE == 3 || MODE == 4 {
                let half = f.m / 2;
                let mut acc = 0u32;
                let mut chosen = usize::MAX;
                let mut fallback = usize::MAX;
                for a in 0..256 {
                    if f.thin[a] > 0 {
                        if a != mb && fallback == usize::MAX {
                            fallback = a;
                        }
                        acc += f.thin[a];
                        if acc >= half && a != mb && chosen == usize::MAX {
                            chosen = a;
                            break;
                        }
                    }
                }
                if chosen != usize::MAX { chosen } else { fallback }
            } else {
                usize::MAX
            };

            let edges = &mut self.edges[..m];
            edges.fill(0);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                if cursors[a] == end {
                    continue;
                }
                let mut queue = outq::Reader::new(out, &self.spill, &heads, a);
                if (MODE == 3 || MODE == 4) && a == a_mid {
                    let first_band = self.bands[cursors[a]];
                    let u_a1 = first_band.dst();
                    let v_a1 = queue.peek_next_rank();
                    anchor1 = Some((v_a1, u_a1));
                }
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

        // S5: Emit
        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
        if use_xor {
            match MODE {
                0 => emit_2cur_base(
                    &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                    n, mark, predecessor, bits, out,
                ),
                1 => emit_2cur_pipe1(
                    &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                    n, mark, predecessor, bits, out,
                ),
                2 => emit_2cur_batch8(
                    &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                    n, mark, predecessor, bits, out,
                ),
                3 => {
                    let (same, s4c, s2c) = emit_4cur_stage::<false>(
                        &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                        n, mark, predecessor, anchor1, bits, out, &mut self.stage,
                    );
                    tm.same_cyc_hits = u64::from(same);
                    tm.steps_4cur = s4c as u64;
                    tm.steps_2cur = s2c as u64;
                }
                4 => {
                    let (same, s4c, s2c) = emit_4cur_stage::<true>(
                        &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                        n, mark, predecessor, anchor1, bits, out, &mut self.stage,
                    );
                    tm.same_cyc_hits = u64::from(same);
                    tm.steps_4cur = s4c as u64;
                    tm.steps_2cur = s2c as u64;
                }
                5 => emit_2cur_hoist(
                    &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                    n, mark, predecessor, bits, out,
                ),
                _ => unreachable!(),
            }
        } else {
            // Standard 1-cursor emit for small/wide blocks
            out.resize(n + INLINE, 0);
            let mut at = 0usize;
            let mask = (1u32 << bits) - 1;
            let mut j = mark;
            for _ in 0..m {
                let edge = self.edges[j as usize];
                let code = if wide { self.extra[j as usize] } else { edge >> bits } as usize;
                j = edge & mask;
                emit_fwd_word(code, &self.inline16, &self.lens, &self.spelling, out, &mut at, n);
                if j == mark || at >= n {
                    break;
                }
            }
            out.truncate(at.min(n));
            repeat_period(n, out);
        }
        if let Some(t) = t_s5 {
            tm.s5_us = t.elapsed().as_secs_f64() * 1e6;
        }
        tm
    }
}

// ============================================================================
// SECTION 6: Drivers for Part 1B, Part 2, and Part 3 (Exactness + 1T/2W/4W)
// ============================================================================

struct RealBlockReport {
    name: String,
    p1: PowerTraceMetrics,
    pret: PowerTraceMetrics,
    lf_subcycles: [usize; 7],
    ret_subcycles: [usize; 7],
}

fn analyze_real_block_p12(blk: &Block) -> RealBlockReport {
    let op = extract_open_perms(&blk.records, blk.n, blk.p, LIMIT);
    let (_s1, m1, t1) = build_g1_matrix(&op.l_col, &op.all);
    let p1 = analyze_power_traces(&t1, &m1);
    let (_sr, mr, tr) = build_ret_double_coset_matrix(&op);
    let pret = analyze_power_traces(&tr, &mr);

    // Part 2: Constructive k-th power cycle-splitting on BOTH LF^k ([n]) and pi_ret^k ([m])!
    let (lf_base, lf_subcycles) = verify_power_cycle_splitting(&op.lf);
    let (ret_base, ret_subcycles) = verify_power_cycle_splitting(&op.pi_ret);
    assert_eq!(lf_base.len(), 1, "Real block {} LF not single-cycle!", blk.name);
    assert_eq!(ret_base.len(), 1, "Real block {} pi_ret not single-cycle!", blk.name);

    RealBlockReport {
        name: blk.name.clone(),
        p1,
        pret,
        lf_subcycles,
        ret_subcycles,
    }
}

fn fmt_opt_k(k: Option<usize>) -> String {
    match k {
        Some(v) => format!("{v:>4}"),
        None => " INF".to_string(),
    }
}

fn print_p1b_table(title: &str, reps: &[RealBlockReport]) {
    println!("  [{title}] — P_ret k-STEP RETURN EXCESS Delta_k = tr(P_ret^k) - 1 & MIXING HORIZON k_{{0.05}}:");
    println!(
        "  {:<15} {:>3} {:>6} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} | {:>4} {:>4} | {:>9} {:>9} {:>9}",
        "Block", "sr", "gam_r", "D_1", "D_2", "D_3", "D_4", "D_8", "D_16", "D_32",
        "k1st", "kSet", "TrEigErr", "NGDetErr", "z1SpErr"
    );
    for r in reps {
        let pr = &r.pret;
        println!(
            "  {:<15} {:>3} {:>6.4} | {:>+7.3} {:>+7.3} {:>+7.3} {:>+7.3} {:>+7.3} {:>+7.3} {:>+7.3} | {} {} | {:>9.2e} {:>9.2e} {:>9.2e}",
            r.name, pr.sigma, pr.gamma,
            pr.deltas[0], pr.deltas[1], pr.deltas[2], pr.deltas[3],
            pr.deltas[4], pr.deltas[5], pr.deltas[6],
            fmt_opt_k(pr.k005_first), fmt_opt_k(pr.k005_settle),
            pr.max_trace_vs_eig_err, pr.max_ng_det_err, pr.z1_spec_prod_err
        );
    }
    println!();
}

fn summarize_p12_corpus(label: &str, reps: &[&RealBlockReport]) {
    let k = reps.len() as f64;
    let mut mean_d = [0.0f64; 7];
    let mut mean_d1 = [0.0f64; 7];
    for r in reps {
        for j in 0..7 {
            mean_d[j] += r.pret.deltas[j];
            mean_d1[j] += r.p1.deltas[j];
        }
    }
    for j in 0..7 {
        mean_d[j] /= k;
        mean_d1[j] /= k;
    }
    let mut k1_vals: Vec<usize> = reps.iter().filter_map(|r| r.pret.k005_first).collect();
    let mut ks_vals: Vec<usize> = reps.iter().filter_map(|r| r.pret.k005_settle).collect();
    let mut k1_p1: Vec<usize> = reps.iter().filter_map(|r| r.p1.k005_settle).collect();
    k1_vals.sort_unstable();
    ks_vals.sort_unstable();
    k1_p1.sort_unstable();
    let mean_k1 = k1_vals.iter().sum::<usize>() as f64 / k1_vals.len().max(1) as f64;
    let mean_ks = ks_vals.iter().sum::<usize>() as f64 / ks_vals.len().max(1) as f64;
    let med_ks = ks_vals.get(ks_vals.len() / 2).copied().unwrap_or(0);
    let max_ks = ks_vals.last().copied().unwrap_or(0);
    let mean_ks_p1 = k1_p1.iter().sum::<usize>() as f64 / k1_p1.len().max(1) as f64;
    let mean_k8_frac = reps.iter().map(|r| r.pret.spec_prod_k8_frac).sum::<f64>() / k;
    let mean_sp = reps.iter().map(|r| r.pret.spec_prod_bits).sum::<f64>() / k;

    // Part 2: mean gcd(n, k) and gcd(m, k)
    let mut mean_gcd_n = [0.0f64; 7];
    let mut mean_gcd_m = [0.0f64; 7];
    let mut split_m_gt1 = [0usize; 7];
    for r in reps {
        for j in 0..7 {
            mean_gcd_n[j] += r.lf_subcycles[j] as f64;
            mean_gcd_m[j] += r.ret_subcycles[j] as f64;
            if r.ret_subcycles[j] > 1 {
                split_m_gt1[j] += 1;
            }
        }
    }
    for j in 0..7 {
        mean_gcd_n[j] /= k;
        mean_gcd_m[j] /= k;
    }

    println!("  [{label} (N={})]", reps.len());
    println!(
        "    P_ret Delta_k (k=1,2,3,4,8,16,32): {:+.3}, {:+.3}, {:+.3}, {:+.3}, {:+.4}, {:+.4}, {:+.5}",
        mean_d[0], mean_d[1], mean_d[2], mean_d[3], mean_d[4], mean_d[5], mean_d[6]
    );
    println!(
        "    P_1   Delta_k (k=1,2,3,4,8,16,32): {:+.3}, {:+.3}, {:+.3}, {:+.3}, {:+.4}, {:+.4}, {:+.5}",
        mean_d1[0], mean_d1[1], mean_d1[2], mean_d1[3], mean_d1[4], mean_d1[5], mean_d1[6]
    );
    println!(
        "    Mixing Horizon k_{{0.05}} (P_ret): first={mean_k1:.2}, settle mean={mean_ks:.2} (med={med_ks}, max={max_ks}) | P_1 settle={mean_ks_p1:.2} | Spec_prod={mean_sp:.3} b (k<=8 captures {:.1}%)",
        mean_k8_frac * 100.0
    );
    println!(
        "    Cycle-Power Splits [k=2,3,4,6,8,12,16]: mean gcd(n,k)={:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2} | mean gcd(m,k)={:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2} (m>1: {}/{}/{}/{}/{}/{}/{})",
        mean_gcd_n[0], mean_gcd_n[1], mean_gcd_n[2], mean_gcd_n[3], mean_gcd_n[4], mean_gcd_n[5], mean_gcd_n[6],
        mean_gcd_m[0], mean_gcd_m[1], mean_gcd_m[2], mean_gcd_m[3], mean_gcd_m[4], mean_gcd_m[5], mean_gcd_m[6],
        split_m_gt1[0], split_m_gt1[1], split_m_gt1[2], split_m_gt1[3], split_m_gt1[4], split_m_gt1[5], split_m_gt1[6]
    );
}

fn run_one_pass_1t<const MODE: u8>(
    k: &mut OpenQKernel,
    blocks: &[Block],
    out: &mut Vec<u8>,
) -> f64 {
    let t0 = Instant::now();
    for b in blocks {
        let _ = k.run::<MODE, 65_536, false>(&b.records, b.n, b.p, out);
    }
    t0.elapsed().as_secs_f64() * 1e3
}

fn run_one_stage_pass_1t<const MODE: u8>(
    k: &mut OpenQKernel,
    blocks: &[Block],
    out: &mut Vec<u8>,
) -> QStageTimes {
    let mut agg = QStageTimes::default();
    for b in blocks {
        let tm = k.run::<MODE, 65_536, true>(&b.records, b.n, b.p, out);
        agg.s123_us += tm.s123_us;
        agg.s4_us += tm.s4_us;
        agg.s5_us += tm.s5_us;
        agg.steps_4cur += tm.steps_4cur;
        agg.steps_2cur += tm.steps_2cur;
        agg.same_cyc_hits += tm.same_cyc_hits;
    }
    agg
}

fn run_one_pass_nw<const MODE: u8>(blocks: &[Block], workers: usize) -> f64 {
    let barrier = Barrier::new(workers);
    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(workers);
        for w_id in 0..workers {
            let b_ref = &barrier;
            handles.push(s.spawn(move || {
                let mut k = OpenQKernel::default();
                let mut out = Vec::new();
                // Warmup 2 blocks per worker before barrier
                for wi in 0..2usize {
                    let wb = &blocks[(w_id + wi) % blocks.len()];
                    let _ = k.run::<MODE, 65_536, false>(&wb.records, wb.n, wb.p, &mut out);
                }
                b_ref.wait();
                let t0 = Instant::now();
                for (idx, b) in blocks.iter().enumerate() {
                    if idx % workers == w_id {
                        let _ = k.run::<MODE, 65_536, false>(&b.records, b.n, b.p, &mut out);
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

fn run_oracle_s5_headroom(label: &str, blocks: &[Block]) {
    let mut k = OpenQKernel::default();
    let mut out = Vec::new();
    let mut out_o = Vec::new();
    let mut stage = Vec::new();
    let mut t_base_us = 0.0f64;
    let mut t_pipe1_us = 0.0f64;
    let mut t_4cur_stage_us = 0.0f64;
    let mut t_4cur_oracle_us = 0.0f64;
    let mut tot_m = 0u64;
    let mut tot_4c_steps = 0u64;
    let mut tot_2c_steps = 0u64;

    let reps = if cfg!(debug_assertions) { 1 } else { 7 };

    for b in blocks {
        // Run baseline to populate edges, spelling, inline16, lens in k!
        let _ = k.run::<0, 65_536, false>(&b.records, b.n, b.p, &mut out);
        // Re-extract exact mark, before, mid, mid_before, mid_byte offline for diagnostic!
        let op = extract_open_perms(&b.records, b.n, b.p, LIMIT);
        let m = op.m;
        if m < 65_536 {
            continue;
        }
        let bits = (u32::BITS - (m as u32 - 1).leading_zeros()).max(1);
        let wide = k.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if wide {
            continue;
        }
        let mask = (1u32 << bits) - 1;
        // Recover mark and before from k.edges using op.pi_ret
        let mut mark = 0u32;
        let mut cur_row = 0usize;
        let mut rec_i = 0usize;
        let mut kept_cnt = 0u32;
        while rec_i < b.records.len() {
            let rec = b.records[rec_i];
            let c = (rec >> 8) as u8;
            let mut w = u32::from(rec & 255);
            rec_i += 1;
            while rec_i < b.records.len() && (b.records[rec_i] >> 8) as u8 == c {
                w += u32::from(b.records[rec_i] & 255);
                rec_i += 1;
            }
            split_run(c, cur_row as u32, w, b.p, LIMIT, |_ch, x, ww, keep| {
                if keep {
                    if b.p.wrapping_sub(x) < ww {
                        mark = kept_cnt + (b.p - x);
                    }
                    kept_cnt += ww;
                }
            });
            cur_row += w as usize;
        }
        let mut before = 0u32;
        for (u, &v) in op.pi_ret.iter().enumerate() {
            if v == mark {
                before = u as u32;
                break;
            }
        }
        // Walk half the cycle offline to get exact oracle midpoint (mid, mid_before, mid_byte)
        let half_steps = m / 2;
        let mut prev = before;
        let mut cur = mark;
        let mut mid_byte = 0usize;
        for _ in 0..half_steps {
            let e = k.edges[cur as usize];
            let code = (e >> bits) as usize;
            let ll = k.lens[code];
            mid_byte += if ll == LONG {
                long_word(&k.inline16[code], &k.spelling).len()
            } else {
                usize::from(ll)
            };
            let nxt = (e & mask) ^ prev;
            prev = cur;
            cur = nxt;
        }
        let mid = cur;
        let mid_before = prev;

        // Also pick the same real S4-style anchor1 (first retained row in middle bucket)
        let a1_real = (m / 2) as u32;
        let mut a1_before = 0u32;
        for (u, &v) in op.pi_ret.iter().enumerate() {
            if v == a1_real {
                a1_before = u as u32;
                break;
            }
        }

        // Verify oracle 4-cursor is 100% byte-exact!
        emit_4cur_oracle(
            &k.edges[..m], &k.spelling, &k.inline16, &k.lens, b.n,
            mark, before, mid, mid_before, mid_byte, bits, &mut out_o,
        );
        assert_eq!(out_o, out, "Q-4Cur-Oracle byte mismatch on {}", b.name);

        let mut tb = Vec::with_capacity(reps);
        let mut tp = Vec::with_capacity(reps);
        let mut ts = Vec::with_capacity(reps);
        let mut to = Vec::with_capacity(reps);
        let (mut s4c_last, mut s2c_last) = (0usize, 0usize);
        for _ in 0..reps {
            let t0 = Instant::now();
            emit_2cur_base(
                &k.edges[..m], &k.spelling, &k.inline16, &k.lens, b.n,
                mark, before, bits, &mut out_o,
            );
            tb.push(t0.elapsed().as_secs_f64() * 1e6);

            let t1 = Instant::now();
            emit_2cur_pipe1(
                &k.edges[..m], &k.spelling, &k.inline16, &k.lens, b.n,
                mark, before, bits, &mut out_o,
            );
            tp.push(t1.elapsed().as_secs_f64() * 1e6);

            let t2 = Instant::now();
            let (_, s4c, s2c) = emit_4cur_stage::<false>(
                &k.edges[..m], &k.spelling, &k.inline16, &k.lens, b.n,
                mark, before, Some((a1_real, a1_before)), bits, &mut out_o, &mut stage,
            );
            ts.push(t2.elapsed().as_secs_f64() * 1e6);
            s4c_last = s4c;
            s2c_last = s2c;

            let t3 = Instant::now();
            emit_4cur_oracle(
                &k.edges[..m], &k.spelling, &k.inline16, &k.lens, b.n,
                mark, before, mid, mid_before, mid_byte, bits, &mut out_o,
            );
            to.push(t3.elapsed().as_secs_f64() * 1e6);
        }
        tb.sort_by(|a, b| a.partial_cmp(b).unwrap());
        tp.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        to.sort_by(|a, b| a.partial_cmp(b).unwrap());
        t_base_us += tb[reps / 2];
        t_pipe1_us += tp[reps / 2];
        t_4cur_stage_us += ts[reps / 2];
        t_4cur_oracle_us += to[reps / 2];
        tot_m += m as u64;
        tot_4c_steps += s4c_last as u64;
        tot_2c_steps += s2c_last as u64;
    }

    let eff_cursors = if tot_4c_steps + tot_2c_steps > 0 {
        (tot_m as f64) / ((tot_4c_steps + tot_2c_steps) as f64)
    } else {
        0.0
    };
    println!(
        "  [S5 ISOLATED HEADROOM — {label}]: Base-2Cur={:.2}ms | Q-2Cur-Pipe1={:.2}ms ({:.3}x) | Q-4Cur-Stage={:.2}ms ({:.3}x, eff_cur={:.2}) | Q-4Cur-Oracle*={:.2}ms ({:.3}x, 0B stage)",
        t_base_us / 1e3,
        t_pipe1_us / 1e3, t_pipe1_us / t_base_us.max(1e-9),
        t_4cur_stage_us / 1e3, t_4cur_stage_us / t_base_us.max(1e-9), eff_cursors,
        t_4cur_oracle_us / 1e3, t_4cur_oracle_us / t_base_us.max(1e-9),
    );
}

fn run_part3_multicursor_s5(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    adm6: &[Block],
    nonwiki10: &[Block],
    wiki233: &[Block],
) {
    println!("================================================================================================================================");
    println!("PART 3: STEPANOV CONSTRUCTION TEST — Q-MultiCursor-S5 (k-WAY MULTI-CURSOR S5 EMIT & PIPELINED 2-LANE XOR)");
    println!("  Arms: Base-2Cur | Q-2Cur-Hoist (+0B) | Q-2Cur-Pipe1 (+0B) | Q-2Cur-Batch8 (+0B heap) | Q-4Cur-Stage (+879K) | Q-4Cur-Pipe1");
    println!("================================================================================================================================");

    // 3A: 100% Byte-Exactness Gate across 5,200 valid + 300 malformed (FORCED XOR_MIN_M=2!) + ALL 249 real blocks!
    let t_ver = Instant::now();
    let mut k0 = OpenQKernel::default();
    let mut k1 = OpenQKernel::default();
    let mut k2 = OpenQKernel::default();
    let mut k3 = OpenQKernel::default();
    let mut k4 = OpenQKernel::default();
    let mut k5 = OpenQKernel::default();
    let mut ref_k = cand_r5_head::Kernel::default();
    let (mut o0, mut o1, mut o2, mut o3, mut o4, mut o5, mut o_ref) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());

    let mut small_4cur_same = 0usize;
    let mut small_4cur_detached_or_fb = 0usize;
    let mut mal_4cur_same = 0usize;
    let mut mal_4cur_detached_or_fb = 0usize;

    for (idx, (l, p)) in valid.iter().enumerate() {
        let recs = records_of(l);
        let expected = invert_forward_fl(l, *p);
        let _ = k0.run::<0, 2, false>(&recs, l.len(), *p, &mut o0);
        let _ = k5.run::<5, 2, false>(&recs, l.len(), *p, &mut o5);
        let _ = k1.run::<1, 2, false>(&recs, l.len(), *p, &mut o1);
        let _ = k2.run::<2, 2, false>(&recs, l.len(), *p, &mut o2);
        let tm3 = k3.run::<3, 2, false>(&recs, l.len(), *p, &mut o3);
        let _ = k4.run::<4, 2, false>(&recs, l.len(), *p, &mut o4);
        assert_eq!(o0, expected, "Base-2Cur mismatch on valid case {idx}");
        assert_eq!(o5, expected, "Q-2Cur-Hoist mismatch on valid case {idx}");
        assert_eq!(o1, expected, "Q-2Cur-Pipe1 mismatch on valid case {idx}");
        assert_eq!(o2, expected, "Q-2Cur-Batch8 mismatch on valid case {idx}");
        assert_eq!(o3, expected, "Q-4Cur-Stage mismatch on valid case {idx}");
        assert_eq!(o4, expected, "Q-4Cur-Pipe1 mismatch on valid case {idx}");
        if tm3.same_cyc_hits > 0 {
            small_4cur_same += 1;
        } else {
            small_4cur_detached_or_fb += 1;
        }
    }

    for (idx, (l, p)) in malformed.iter().enumerate() {
        let recs = records_of(l);
        let expected = invert_forward_fl(l, *p);
        ref_k.run_entry(&recs, l.len(), *p, cand_r5_head::Cap::NONE, &mut o_ref).unwrap();
        assert_eq!(o_ref, expected, "cand_r5_head mismatch on malformed {idx}");
        let _ = k0.run::<0, 2, false>(&recs, l.len(), *p, &mut o0);
        let _ = k5.run::<5, 2, false>(&recs, l.len(), *p, &mut o5);
        let _ = k1.run::<1, 2, false>(&recs, l.len(), *p, &mut o1);
        let _ = k2.run::<2, 2, false>(&recs, l.len(), *p, &mut o2);
        let tm3 = k3.run::<3, 2, false>(&recs, l.len(), *p, &mut o3);
        let _ = k4.run::<4, 2, false>(&recs, l.len(), *p, &mut o4);
        assert_eq!(o0, expected, "Base-2Cur mismatch on malformed case {idx}");
        assert_eq!(o5, expected, "Q-2Cur-Hoist mismatch on malformed case {idx}");
        assert_eq!(o1, expected, "Q-2Cur-Pipe1 mismatch on malformed case {idx}");
        assert_eq!(o2, expected, "Q-2Cur-Batch8 mismatch on malformed case {idx}");
        assert_eq!(o3, expected, "Q-4Cur-Stage mismatch on malformed case {idx}");
        assert_eq!(o4, expected, "Q-4Cur-Pipe1 mismatch on malformed case {idx}");
        if tm3.same_cyc_hits > 0 {
            mal_4cur_same += 1;
        } else {
            mal_4cur_detached_or_fb += 1;
        }
    }

    let mut real_checked = 0usize;
    let mut real_4cur_same = 0usize;
    for b in adm6.iter().chain(nonwiki10.iter()).chain(wiki233.iter()) {
        ref_k.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut o_ref).unwrap();
        let _ = k0.run::<0, 65_536, false>(&b.records, b.n, b.p, &mut o0);
        let _ = k5.run::<5, 65_536, false>(&b.records, b.n, b.p, &mut o5);
        let _ = k1.run::<1, 65_536, false>(&b.records, b.n, b.p, &mut o1);
        let _ = k2.run::<2, 65_536, false>(&b.records, b.n, b.p, &mut o2);
        let tm3 = k3.run::<3, 65_536, false>(&b.records, b.n, b.p, &mut o3);
        let _ = k4.run::<4, 65_536, false>(&b.records, b.n, b.p, &mut o4);
        assert_eq!(o0, o_ref, "Base-2Cur mismatch on {}", b.name);
        assert_eq!(o5, o_ref, "Q-2Cur-Hoist mismatch on {}", b.name);
        assert_eq!(o1, o_ref, "Q-2Cur-Pipe1 mismatch on {}", b.name);
        assert_eq!(o2, o_ref, "Q-2Cur-Batch8 mismatch on {}", b.name);
        assert_eq!(o3, o_ref, "Q-4Cur-Stage mismatch on {}", b.name);
        assert_eq!(o4, o_ref, "Q-4Cur-Pipe1 mismatch on {}", b.name);
        if tm3.same_cyc_hits > 0 {
            real_4cur_same += 1;
        }
        real_checked += 1;
    }

    println!(
        "  [PASS 3A] 100% Byte-Exactness (ALL 6 Arms vs cand_r5_head + forward-FL) in {:.2}s:",
        t_ver.elapsed().as_secs_f64()
    );
    println!("    - Valid Small (XOR_MIN_M=2)     : 5,200/5,200 EXACT (Q-4Cur same-cycle={small_4cur_same}, fb/tiny={small_4cur_detached_or_fb})");
    println!("    - Malformed Multi-Cycle (XOR=2) :   300/300   EXACT (Q-4Cur same-cycle={mal_4cur_same}, detached-cycle/fb={mal_4cur_detached_or_fb})");
    println!("    - Real Blocks (XOR_MIN_M=65536) :   {real_checked}/249   EXACT (Q-4Cur 4-cursor active on {real_4cur_same}/249 real blocks!)\n");

    // 3B: Isolated S5 Headroom Decomposition (including Offline Q-4Cur-Oracle* 0B-staging ceiling!)
    run_oracle_s5_headroom("6 Admitted Wikidata", adm6);
    run_oracle_s5_headroom("10 Non-Wikidata (6 JSONL/Log + 4 Code)", nonwiki10);
    println!();

    // 3C: Warmed-Up Round-Robin Interleaved 1T / 2W / 4W Full E2E + S4/S5 Stage Attribution + WS Audit
    let rounds_adm = if cfg!(debug_assertions) { 1 } else { 9 };
    let rounds_nw = if cfg!(debug_assertions) { 1 } else { 7 };
    let rounds_wiki = if cfg!(debug_assertions) { 1 } else { 3 };
    let st_rounds = if cfg!(debug_assertions) { 1 } else { 3 };

    let corpora: [(&str, &[Block], usize); 3] = [
        ("6 Admitted Wikidata", adm6, rounds_adm),
        ("10 Non-Wiki (6 JSONL/Log + 4 Code)", nonwiki10, rounds_nw),
        ("233 Full Wikidata", wiki233, rounds_wiki),
    ];

    for (label, blks, rds) in corpora {
        println!(
            "  [E2E INTERLEAVED BENCHMARK — {label} ({} blks, {rds} interleaved rounds median)]:",
            blks.len()
        );
        // Fresh kernel instances per arm + full warmup pass BEFORE timing!
        let mut ks = [
            OpenQKernel::default(),
            OpenQKernel::default(),
            OpenQKernel::default(),
            OpenQKernel::default(),
            OpenQKernel::default(),
            OpenQKernel::default(),
            OpenQKernel::default(),
        ];
        let mut out = Vec::new();
        let _ = run_one_pass_1t::<0>(&mut ks[0], blks, &mut out);
        let _ = run_one_pass_1t::<0>(&mut ks[1], blks, &mut out);
        let _ = run_one_pass_1t::<5>(&mut ks[2], blks, &mut out);
        let _ = run_one_pass_1t::<1>(&mut ks[3], blks, &mut out);
        let _ = run_one_pass_1t::<2>(&mut ks[4], blks, &mut out);
        let _ = run_one_pass_1t::<3>(&mut ks[5], blks, &mut out);
        let _ = run_one_pass_1t::<4>(&mut ks[6], blks, &mut out);
        let wss = [
            ks[0].ws_bytes(),
            ks[1].ws_bytes(),
            ks[2].ws_bytes(),
            ks[3].ws_bytes(),
            ks[4].ws_bytes(),
            ks[5].ws_bytes(),
            ks[6].ws_bytes(),
        ];

        // Interleaved S4/S5 stage timing rounds (median of st_rounds)
        let mut s4_v: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(st_rounds));
        let mut s5_v: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(st_rounds));
        let mut st_q4 = QStageTimes::default();
        for _ in 0..st_rounds {
            let tms = [
                run_one_stage_pass_1t::<0>(&mut ks[0], blks, &mut out),
                run_one_stage_pass_1t::<0>(&mut ks[1], blks, &mut out),
                run_one_stage_pass_1t::<5>(&mut ks[2], blks, &mut out),
                run_one_stage_pass_1t::<1>(&mut ks[3], blks, &mut out),
                run_one_stage_pass_1t::<2>(&mut ks[4], blks, &mut out),
                run_one_stage_pass_1t::<3>(&mut ks[5], blks, &mut out),
                run_one_stage_pass_1t::<4>(&mut ks[6], blks, &mut out),
            ];
            for a in 0..7 {
                s4_v[a].push(tms[a].s4_us);
                s5_v[a].push(tms[a].s5_us);
            }
            st_q4 = tms[5];
        }
        let mut s4_med = [0.0f64; 7];
        let mut s5_med = [0.0f64; 7];
        for a in 0..7 {
            s4_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            s5_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            s4_med[a] = s4_v[a][st_rounds / 2];
            s5_med[a] = s5_v[a][st_rounds / 2];
        }

        // Interleaved 1T, 2W, 4W E2E rounds
        let mut t1_v: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t2_v: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t4_v: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(rds));
        for _ in 0..rds {
            t1_v[0].push(run_one_pass_1t::<0>(&mut ks[0], blks, &mut out));
            t1_v[1].push(run_one_pass_1t::<0>(&mut ks[1], blks, &mut out));
            t1_v[2].push(run_one_pass_1t::<5>(&mut ks[2], blks, &mut out));
            t1_v[3].push(run_one_pass_1t::<1>(&mut ks[3], blks, &mut out));
            t1_v[4].push(run_one_pass_1t::<2>(&mut ks[4], blks, &mut out));
            t1_v[5].push(run_one_pass_1t::<3>(&mut ks[5], blks, &mut out));
            t1_v[6].push(run_one_pass_1t::<4>(&mut ks[6], blks, &mut out));
        }
        for _ in 0..rds {
            t2_v[0].push(run_one_pass_nw::<0>(blks, 2));
            t2_v[1].push(run_one_pass_nw::<0>(blks, 2));
            t2_v[2].push(run_one_pass_nw::<5>(blks, 2));
            t2_v[3].push(run_one_pass_nw::<1>(blks, 2));
            t2_v[4].push(run_one_pass_nw::<2>(blks, 2));
            t2_v[5].push(run_one_pass_nw::<3>(blks, 2));
            t2_v[6].push(run_one_pass_nw::<4>(blks, 2));
        }
        for _ in 0..rds {
            t4_v[0].push(run_one_pass_nw::<0>(blks, 4));
            t4_v[1].push(run_one_pass_nw::<0>(blks, 4));
            t4_v[2].push(run_one_pass_nw::<5>(blks, 4));
            t4_v[3].push(run_one_pass_nw::<1>(blks, 4));
            t4_v[4].push(run_one_pass_nw::<2>(blks, 4));
            t4_v[5].push(run_one_pass_nw::<3>(blks, 4));
            t4_v[6].push(run_one_pass_nw::<4>(blks, 4));
        }

        let mut t1_m = [0.0f64; 7];
        let mut t2_m = [0.0f64; 7];
        let mut t4_m = [0.0f64; 7];
        for a in 0..7 {
            t1_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            t2_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            t4_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            t1_m[a] = t1_v[a][rds / 2];
            t2_m[a] = t2_v[a][rds / 2];
            t4_m[a] = t4_v[a][rds / 2];
        }

        let names = [
            "Base-2Cur",
            "Base-2Cur-AA",
            "Q-2Cur-Hoist",
            "Q-2Cur-Pipe1",
            "Q-2Cur-Batch8",
            "Q-4Cur-Stage",
            "Q-4Cur-Pipe1",
        ];
        println!(
            "  {:<14} | {:>14} {:>14} {:>14} | {:>7} {:>7} | {:>7} {:>7} | {:>8}",
            "Arm", "1T ms (rel)", "2W ms (rel)", "4W ms (rel)",
            "S4 ms", "S4 rel", "S5 ms", "S5 rel", "WS KiB"
        );
        for a in 0..7 {
            println!(
                "  {:<14} | {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) | {:>7.2} ({:.3}x) | {:>7.2} ({:.3}x) | {:>8.1}",
                names[a],
                t1_m[a], t1_m[a] / t1_m[0],
                t2_m[a], t2_m[a] / t2_m[0],
                t4_m[a], t4_m[a] / t4_m[0],
                s4_med[a] / 1e3, s4_med[a] / s4_med[0].max(1e-9),
                s5_med[a] / 1e3, s5_med[a] / s5_med[0].max(1e-9),
                wss[a] as f64 / 1024.0,
            );
        }
        let s4_steps_frac = if st_q4.steps_4cur + st_q4.steps_2cur > 0 {
            100.0 * (st_q4.steps_4cur as f64) / ((st_q4.steps_4cur + st_q4.steps_2cur) as f64)
        } else {
            0.0
        };
        let eff_cur = if st_q4.steps_4cur + st_q4.steps_2cur > 0 {
            (4.0 * st_q4.steps_4cur as f64 + 2.0 * st_q4.steps_2cur as f64)
                / ((st_q4.steps_4cur + st_q4.steps_2cur) as f64)
        } else {
            0.0
        };
        println!(
            "    Q-4Cur Whitworth-Stevens balance: 4-cur loop steps = {:.1}% of iterations (effective active cursors = {:.2} vs Whitworth-Stevens 3.00 theory!), WS delta = {:+.1} KiB\n",
            s4_steps_frac,
            eff_cur,
            (wss[5] as f64 - wss[0] as f64) / 1024.0
        );
    }
}

fn main() {
    let t_total = Instant::now();
    println!("Cycle 52 (P1) — Analogue Q: Girard (1629) / Newton (1707) Power-Trace Bridge,");
    println!("Frobenius (1900) / Burnside (1911) k-th Power Cycle Splitting, & Multi-Cursor S5\n");

    // Generate 5,200 Valid Small BWTs + 300 Malformed Multi-Cycle BWTs
    let (valid, malformed) = generate_small_and_malformed_suites();

    // PART 1A: Exact BigInt Newton-Girard & Frobenius (1900) Cycle-Index on 5,200 + 300
    run_part1a_exact_bigint(&valid, &malformed);

    // PART 2A: Constructive Frobenius-Cauchy-Burnside k-th Power Cycle-Splitting +
    //          Burnside (1911) Orbit-Counting + Frobenius (1900) Hook Characters on 5,200 + 300
    println!("================================================================================================================================");
    println!("PART 2A: FROBENIUS (1900) / BURNSIDE (1911) k-TH POWER CYCLE-SPLITTING ON LF^k & pi_ret^k (5,200 SMALL + 300 MALFORMED)");
    println!("  Strides k in {{2, 3, 4, 6, 8, 12, 16}}: every L-cycle splits into exact gcd(L, k) disjoint sub-cycles of length L / gcd(L, k)");
    println!("================================================================================================================================");
    let t_p2a = Instant::now();
    let mut burnside_tot = 0usize;
    let mut hook_tot = 0usize;
    let mut valid_lf_cyc = 0usize;
    let mut mal_lf_cyc = 0usize;
    let mut mal_ret_cyc = 0usize;

    for (l, p) in &valid {
        let recs = records_of(l);
        let op = extract_open_perms(&recs, l.len(), *p, 2); // LIMIT=2 on small n for nontrivial pi_ret!
        let (lf_lens, _) = verify_power_cycle_splitting(&op.lf);
        let (_ret_lens, _) = verify_power_cycle_splitting(&op.pi_ret);
        valid_lf_cyc += lf_lens.len();
        let (bc1, hc1) = verify_burnside_and_frobenius_hooks(&op.lf);
        let (bc2, hc2) = verify_burnside_and_frobenius_hooks(&op.pi_ret);
        burnside_tot += bc1 + bc2;
        hook_tot += hc1 + hc2;
    }
    for (l, p) in &malformed {
        let recs = records_of(l);
        let op = extract_open_perms(&recs, l.len(), *p, 2);
        let (lf_lens, _) = verify_power_cycle_splitting(&op.lf);
        let (ret_lens, _) = verify_power_cycle_splitting(&op.pi_ret);
        mal_lf_cyc += lf_lens.len();
        mal_ret_cyc += ret_lens.len();
        let (bc1, hc1) = verify_burnside_and_frobenius_hooks(&op.lf);
        let (bc2, hc2) = verify_burnside_and_frobenius_hooks(&op.pi_ret);
        burnside_tot += bc1 + bc2;
        hook_tot += hc1 + hc2;
    }
    println!(
        "  [PASS 2A] 5,200 Valid + 300 Malformed (BOTH LF^k & pi_ret^k across ALL 7 strides k in {{2,3,4,6,8,12,16}}) in {:.2}s:",
        t_p2a.elapsed().as_secs_f64()
    );
    println!("    - Constructive cycle-splitting checks: 77,000 / 77,000 EXACT (0 mismatches!; valid LF cycles={valid_lf_cyc}, mal LF={mal_lf_cyc}, mal pi_ret={mal_ret_cyc})");
    println!("    - Burnside (1911, Ch. XV §145) orbit-counting trace average: {burnside_tot} / {burnside_tot} EXACT (0 mismatches!)");
    println!("    - Frobenius (1900) hook-character generating polynomial    : {hook_tot} / {hook_tot} EXACT (0 mismatches!)\n");

    // Load Solved Analogue Q + ALL 249 Real Blocks
    let solved = make_solved_analogue_q();
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

    // PART 1B & PART 2B: Solved Analogue Q + ALL 249 Real Blocks
    println!("================================================================================================================================");
    println!("PART 1B & 2B: FLOATING-POINT NEWTON-GIRARD BRIDGE (< 1e-11), MIXING HORIZON k_{{0.05}}, & LF^k / pi_ret^k SPLITTING ON 249 REAL BLOCKS");
    println!("================================================================================================================================");
    let t_real = Instant::now();
    let solved_reps: Vec<RealBlockReport> = solved.iter().map(analyze_real_block_p12).collect();
    let adm6_reps: Vec<RealBlockReport> = adm6.iter().map(analyze_real_block_p12).collect();
    let nw6_reps: Vec<RealBlockReport> = nw6.iter().map(analyze_real_block_p12).collect();
    let cd4_reps: Vec<RealBlockReport> = cd4.iter().map(analyze_real_block_p12).collect();
    let w233_reps: Vec<RealBlockReport> = {
        let chunks = 4usize;
        let chunk_sz = (wiki233.len() + chunks - 1) / chunks;
        let mut out_reps = Vec::with_capacity(wiki233.len());
        std::thread::scope(|s| {
            let handles: Vec<_> = wiki233
                .chunks(chunk_sz)
                .map(|ch| s.spawn(move || ch.iter().map(analyze_real_block_p12).collect::<Vec<_>>()))
                .collect();
            for h in handles {
                out_reps.extend(h.join().unwrap());
            }
        });
        out_reps
    };

    print_p1b_table("SOLVED ANALOGUE Q (Aperiodic vs Periodic h=2,3 Contrast!)", &solved_reps);
    print_p1b_table("6 ADMITTED WIKIDATA BLOCKS", &adm6_reps);
    print_p1b_table("6 REAL NON-WIKIDATA BLOCKS (JSONL + SYSLOG)", &nw6_reps);
    print_p1b_table("4 REAL CODE+DOCS BLOCKS", &cd4_reps);

    let adm6_r: Vec<&RealBlockReport> = adm6_reps.iter().collect();
    let nw6_r: Vec<&RealBlockReport> = nw6_reps.iter().collect();
    let cd4_r: Vec<&RealBlockReport> = cd4_reps.iter().collect();
    let w233_r: Vec<&RealBlockReport> = w233_reps.iter().collect();
    let mut all_249: Vec<&RealBlockReport> = Vec::with_capacity(249);
    all_249.extend(&adm6_r);
    all_249.extend(&nw6_r);
    all_249.extend(&cd4_r);
    all_249.extend(&w233_r);
    assert_eq!(all_249.len(), 249);

    println!(
        "  [AGGREGATE 249 REAL BLOCKS SUMMARY (Parts 1B + 2B computed in {:.2}s)]:",
        t_real.elapsed().as_secs_f64()
    );
    summarize_p12_corpus("6 Admitted Wikidata", &adm6_r);
    summarize_p12_corpus("6 JSONL/Syslog", &nw6_r);
    summarize_p12_corpus("4 Code+Docs", &cd4_r);
    summarize_p12_corpus("233 Full wikidata.bz2", &w233_r);
    summarize_p12_corpus("ALL 249 REAL BLOCKS COMBINED", &all_249);

    let max_tr_p1 = all_249.iter().map(|r| r.p1.max_trace_vs_eig_err).fold(0.0f64, f64::max);
    let max_tr_pr = all_249.iter().map(|r| r.pret.max_trace_vs_eig_err).fold(0.0f64, f64::max);
    let max_ng_p1 = all_249.iter().map(|r| r.p1.max_ng_det_err).fold(0.0f64, f64::max);
    let max_ng_pr = all_249.iter().map(|r| r.pret.max_ng_det_err).fold(0.0f64, f64::max);
    let max_ek_p1 = all_249.iter().map(|r| r.p1.max_ng_ek_err).fold(0.0f64, f64::max);
    let max_ek_pr = all_249.iter().map(|r| r.pret.max_ng_ek_err).fold(0.0f64, f64::max);
    let max_z1_p1 = all_249.iter().map(|r| r.p1.z1_spec_prod_err).fold(0.0f64, f64::max);
    let max_z1_pr = all_249.iter().map(|r| r.pret.z1_spec_prod_err).fold(0.0f64, f64::max);

    println!("\n  [VERIFY 249/249] GLOBAL PRECISION & CYCLE-SPLITTING GATE:");
    println!("    - tr(P^k) vs sum(lambda_i^k) max err      : P_1 = {max_tr_p1:.3e}, P_ret = {max_tr_pr:.3e}  (< 1e-11 PASS!)");
    println!("    - Newton-Girard det(I - zP) (z=+-0.5) err : P_1 = {max_ng_p1:.3e}, P_ret = {max_ng_pr:.3e}  (< 1e-11 PASS!)");
    println!("    - Newton-Girard e_1..e_4 vs eig rel err   : P_1 = {max_ek_p1:.3e}, P_ret = {max_ek_pr:.3e}  (< 1e-11 PASS!)");
    println!("    - Newton-Girard z=1 Spec_prod series err  : P_1 = {max_z1_p1:.3e}, P_ret = {max_z1_pr:.3e}  (< 1e-11 PASS!)");
    println!("    - Constructive LF^k & pi_ret^k splitting  : 3,486 / 3,486 (249 x 2 x 7 strides) EXACT (0 mismatches!)\n");
    assert!(max_tr_p1 < 1e-11 && max_tr_pr < 1e-11);
    assert!(max_ng_p1 < 1e-11 && max_ng_pr < 1e-11);
    assert!(max_ek_p1 < 1e-11 && max_ek_pr < 1e-11);
    assert!(max_z1_p1 < 1e-11 && max_z1_pr < 1e-11);

    // PART 3: Stepanov Construction Test (Q-MultiCursor-S5)
    let mut nonwiki10: Vec<Block> = Vec::with_capacity(10);
    for b in nw6.into_iter().chain(cd4.into_iter()) {
        nonwiki10.push(b);
    }
    run_part3_multicursor_s5(&valid, &malformed, &adm6, &nonwiki10, &wiki233);

    println!(
        "  [DONE] Analogue Q (Girard 1629 / Newton 1707 / Frobenius 1900 / Burnside 1911) completed in {:.2}s!",
        t_total.elapsed().as_secs_f64()
    );
}
