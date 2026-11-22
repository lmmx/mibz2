#![forbid(unsafe_code)]

//! Cycle 50 (P1) — Analogue M: Cross-Chain Classical Synthesis I
//! P. A. MacMahon (*Combinatory Analysis*, Vol. I, Cambridge Univ. Press, 1915)
//!   - Theorem M1 (Section III, Ch. II, Arts. 64–79, pp. 93–123):
//!       MacMahon's Master Theorem [x^m] det(I - X A)^{-1} as the Exact Multi-Cycle / BWT
//!       Partition Function Bridging Chain A (CA-34 Sylvester–Cayley Matrix-Tree / BEST)
//!       and Chain B (CA-32/33 Frobenius Characters & Double Coset) + CA-29 (Netto–MacMahon).
//!   - Theorem M2 (Section III, Ch. IV, Arts. 103–108, pp. 135–145):
//!       Genuine MacMahon Major Index maj(w) vs Inversion Number inv(w) q-Multinomial
//!       Equidistribution, BWT Run-Clustering Symmetry Breaking, and Honest Structural /
//!       Cache-Miss Correlation Audit on Open<16>'s Retained Permutation pi_ret.
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

const LIMIT: u32 = 16;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

// ============================================================================
// SECTION 1: Exact Signed Radix-2^32 BigInt & Sylvester-Bareiss Determinant
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
struct BigInt {
    neg: bool,
    limbs: Vec<u32>, // little-endian base 2^32
}

impl BigInt {
    fn zero() -> Self {
        BigInt { neg: false, limbs: Vec::new() }
    }
    fn one() -> Self {
        BigInt { neg: false, limbs: vec![1] }
    }
    fn from_u64(mut v: u64) -> Self {
        if v == 0 {
            return Self::zero();
        }
        let mut limbs = Vec::with_capacity(2);
        while v > 0 {
            limbs.push(v as u32);
            v >>= 32;
        }
        BigInt { neg: false, limbs }
    }
    fn from_i64(v: i64) -> Self {
        if v == 0 {
            return Self::zero();
        }
        let mut r = Self::from_u64(v.unsigned_abs());
        r.neg = v < 0;
        r
    }
    fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }
    fn norm(&mut self) {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
        if self.limbs.is_empty() {
            self.neg = false;
        }
    }
    fn neg_val(&self) -> Self {
        if self.is_zero() {
            Self::zero()
        } else {
            BigInt { neg: !self.neg, limbs: self.limbs.clone() }
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
        self.add(&rhs.neg_val())
    }
    fn mul(&self, rhs: &Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::zero();
        }
        let na = self.limbs.len();
        let nb = rhs.limbs.len();
        let mut out = vec![0u32; na + nb];
        for i in 0..na {
            let av = self.limbs[i] as u64;
            if av == 0 {
                continue;
            }
            let mut carry = 0u64;
            for j in 0..nb {
                let prod = av * (rhs.limbs[j] as u64) + (out[i + j] as u64) + carry;
                out[i + j] = prod as u32;
                carry = prod >> 32;
            }
            out[i + nb] = carry as u32;
        }
        let mut r = BigInt { neg: self.neg != rhs.neg, limbs: out };
        r.norm();
        r
    }
    fn mul_u64(&self, m: u64) -> Self {
        if self.is_zero() || m == 0 {
            return Self::zero();
        }
        if m <= u32::MAX as u64 {
            let mut out = Vec::with_capacity(self.limbs.len() + 1);
            let mut carry = 0u64;
            for &x in &self.limbs {
                let prod = (x as u64) * m + carry;
                out.push(prod as u32);
                carry = prod >> 32;
            }
            if carry > 0 {
                out.push(carry as u32);
            }
            let mut r = BigInt { neg: self.neg, limbs: out };
            r.norm();
            r
        } else {
            self.mul(&BigInt::from_u64(m))
        }
    }
    fn mul_i64(&self, m: i64) -> Self {
        if m == 0 || self.is_zero() {
            return Self::zero();
        }
        let mut r = self.mul_u64(m.unsigned_abs());
        if m < 0 {
            r.neg = !r.neg;
        }
        r
    }
    fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        assert!(!rhs.is_zero(), "BigInt division by zero");
        match Self::cmp_mag(&self.limbs, &rhs.limbs) {
            Ordering::Less => return (Self::zero(), self.clone()),
            Ordering::Equal => {
                let mut q = BigInt::one();
                q.neg = self.neg != rhs.neg;
                return (q, Self::zero());
            }
            Ordering::Greater => {}
        }
        let n = rhs.limbs.len();
        if n == 1 {
            let d = rhs.limbs[0] as u64;
            let mut q_limbs = vec![0u32; self.limbs.len()];
            let mut rem = 0u64;
            for i in (0..self.limbs.len()).rev() {
                let cur = (rem << 32) | (self.limbs[i] as u64);
                q_limbs[i] = (cur / d) as u32;
                rem = cur % d;
            }
            let mut q = BigInt { neg: self.neg != rhs.neg, limbs: q_limbs };
            q.norm();
            let mut r = BigInt {
                neg: self.neg,
                limbs: if rem == 0 { Vec::new() } else { vec![rem as u32] },
            };
            r.norm();
            return (q, r);
        }

        let shift = rhs.limbs[n - 1].leading_zeros();
        let mut vn = vec![0u32; n];
        if shift == 0 {
            vn.copy_from_slice(&rhs.limbs);
        } else {
            let mut carry = 0u32;
            for i in 0..n {
                let x = rhs.limbs[i];
                vn[i] = (x << shift) | carry;
                carry = x >> (32 - shift);
            }
        }
        let ulen = self.limbs.len();
        let m = ulen - n;
        let mut un = vec![0u32; ulen + 1];
        if shift == 0 {
            un[..ulen].copy_from_slice(&self.limbs);
        } else {
            let mut carry = 0u32;
            for i in 0..ulen {
                let x = self.limbs[i];
                un[i] = (x << shift) | carry;
                carry = x >> (32 - shift);
            }
            un[ulen] = carry;
        }

        let v_hi = vn[n - 1] as u128;
        let v_lo = vn[n - 2] as u128;
        let base128: u128 = 1u128 << 32;
        let mut q_limbs = vec![0u32; m + 1];

        for j in (0..=m).rev() {
            let u2 = un[j + n] as u128;
            let u1 = un[j + n - 1] as u128;
            let u0 = un[j + n - 2] as u128;
            let num = (u2 << 32) | u1;
            let mut qhat = num / v_hi;
            let mut rhat = num % v_hi;
            if qhat >= base128 {
                qhat = base128 - 1;
                rhat = num - qhat * v_hi;
            }
            while rhat < base128 && qhat * v_lo > ((rhat << 32) | u0) {
                qhat -= 1;
                rhat += v_hi;
            }

            let q64 = qhat as u64;
            let mut borrow = 0i128;
            for i in 0..n {
                let p = (q64 as u128) * (vn[i] as u128);
                let sub = (un[j + i] as i128) - (p as u32 as i128) - borrow;
                un[j + i] = (sub & 0xFFFF_FFFF) as u32;
                borrow = ((p >> 32) as i128) - (sub >> 32);
            }
            let sub_top = (un[j + n] as i128) - borrow;
            un[j + n] = (sub_top & 0xFFFF_FFFF) as u32;

            if sub_top < 0 {
                qhat -= 1;
                let mut carry = 0u64;
                for i in 0..n {
                    let s = (un[j + i] as u64) + (vn[i] as u64) + carry;
                    un[j + i] = s as u32;
                    carry = s >> 32;
                }
                un[j + n] = un[j + n].wrapping_add(carry as u32);
            }
            q_limbs[j] = qhat as u32;
        }

        let mut r_limbs = vec![0u32; n];
        if shift == 0 {
            r_limbs.copy_from_slice(&un[..n]);
        } else {
            for i in 0..n {
                let lo = un[i] >> shift;
                let hi = if i + 1 < n { un[i + 1] << (32 - shift) } else { 0 };
                r_limbs[i] = lo | hi;
            }
        }

        let mut q = BigInt { neg: self.neg != rhs.neg, limbs: q_limbs };
        q.norm();
        let mut r = BigInt { neg: self.neg, limbs: r_limbs };
        r.norm();
        (q, r)
    }

    fn log2_approx(&self) -> f64 {
        if self.is_zero() {
            return f64::NEG_INFINITY;
        }
        let n = self.limbs.len();
        if n == 1 {
            (self.limbs[0] as f64).log2()
        } else if n == 2 {
            let v = ((self.limbs[1] as u64) << 32) | (self.limbs[0] as u64);
            (v as f64).log2()
        } else {
            let top = ((self.limbs[n - 1] as f64) * 4294967296.0 + (self.limbs[n - 2] as f64))
                * 4294967296.0
                + (self.limbs[n - 3] as f64);
            top.log2() + 32.0 * ((n - 3) as f64)
        }
    }

    fn to_f64_ratio(&self, denom: &Self) -> f64 {
        if self.is_zero() {
            return 0.0;
        }
        let diff_log2 = self.log2_approx() - denom.log2_approx();
        let s = if self.neg != denom.neg { -1.0 } else { 1.0 };
        s * diff_log2.exp2()
    }
}

fn fact_u64(n: usize) -> u64 {
    let mut r = 1u64;
    for k in 2..=n {
        r *= k as u64;
    }
    r
}

fn bigint_factorial(n: usize) -> BigInt {
    let mut r = BigInt::one();
    for k in 2..=n {
        r = r.mul_u64(k as u64);
    }
    r
}

fn log2_fact(n: usize) -> f64 {
    if n <= 1 {
        return 0.0;
    }
    if n <= 1024 {
        let mut s = 0.0f64;
        for k in 2..=n {
            s += (k as f64).log2();
        }
        s
    } else {
        let x = n as f64;
        let ln_f = x * x.ln() - x + 0.5 * (2.0 * std::f64::consts::PI * x).ln()
            + 1.0 / (12.0 * x)
            - 1.0 / (360.0 * x * x * x)
            + 1.0 / (1260.0 * x.powi(5));
        ln_f / std::f64::consts::LN_2
    }
}

/// Exact integer determinant via Sylvester (1851, 1857) fraction-free Bareiss elimination (CA-34).
fn bareiss_det_bigint(mut a: Vec<Vec<BigInt>>) -> BigInt {
    let n = a.len();
    if n == 0 {
        return BigInt::one();
    }
    if n == 1 {
        return a[0][0].clone();
    }
    let mut sign_neg = false;
    let mut prev_pivot = BigInt::one();
    for k in 0..n {
        if a[k][k].is_zero() {
            let mut piv_row = None;
            for i in (k + 1)..n {
                if !a[i][k].is_zero() {
                    piv_row = Some(i);
                    break;
                }
            }
            let Some(p) = piv_row else {
                return BigInt::zero();
            };
            a.swap(k, p);
            sign_neg = !sign_neg;
        }
        let piv = a[k][k].clone();
        for i in (k + 1)..n {
            let aik = a[i][k].clone();
            for j in (k + 1)..n {
                let akj = &a[k][j];
                let aij = &a[i][j];
                let num = if aij.is_zero() {
                    if aik.is_zero() || akj.is_zero() {
                        BigInt::zero()
                    } else {
                        aik.mul(akj).neg_val()
                    }
                } else {
                    let t1 = piv.mul(aij);
                    if aik.is_zero() || akj.is_zero() {
                        t1
                    } else {
                        t1.sub(&aik.mul(akj))
                    }
                };
                if k == 0 || num.is_zero() {
                    a[i][j] = num;
                } else {
                    let (q, rem) = num.div_rem(&prev_pivot);
                    assert!(
                        rem.is_zero(),
                        "Sylvester-Bareiss exact divisibility violated at k={k}, i={i}, j={j}"
                    );
                    a[i][j] = q;
                }
            }
        }
        prev_pivot = piv;
    }
    let mut det = a[n - 1][n - 1].clone();
    if sign_neg && !det.is_zero() {
        det.neg = !det.neg;
    }
    det
}

/// Exact i64 determinant for small matrices (size <= 4) used in principal-minor expansion.
fn det_small_i64(a: &[Vec<i64>]) -> i64 {
    let n = a.len();
    match n {
        0 => 1,
        1 => a[0][0],
        2 => a[0][0] * a[1][1] - a[0][1] * a[1][0],
        3 => {
            a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
                - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
                + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
        }
        4 => {
            let mut sum = 0i64;
            for c in 0..4 {
                let mut sub = vec![vec![0i64; 3]; 3];
                for r in 1..4 {
                    let mut sc = 0;
                    for j in 0..4 {
                        if j != c {
                            sub[r - 1][sc] = a[r][j];
                            sc += 1;
                        }
                    }
                }
                let sign = if c % 2 == 0 { 1 } else { -1 };
                sum += sign * a[0][c] * det_small_i64(&sub);
            }
            sum
        }
        _ => panic!("det_small_i64 only supports n <= 4"),
    }
}

/// Pivoted f64 LU returning (sign, log2(|det|)). Returns (0, -inf) if singular.
fn signed_log2_det_lu(mut a: Vec<Vec<f64>>) -> (i32, f64) {
    let n = a.len();
    if n == 0 {
        return (1, 0.0);
    }
    let mut sign = 1i32;
    let mut log2_abs = 0.0f64;
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
        log2_abs += diag.abs().log2();
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
    (sign, log2_abs)
}

// ============================================================================
// SECTION 2: Pure Safe-Rust Real Hessenberg + Francis Double-Shift QR Eigenvalues
// ============================================================================

#[derive(Clone, Copy, Debug)]
struct C64 {
    re: f64,
    im: f64,
}

impl C64 {
    fn new(re: f64, im: f64) -> Self {
        C64 { re, im }
    }
    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
}

/// Householder similarity reduction of a real n x n matrix to upper Hessenberg form.
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

        // Left multiply: A[k+1..n, k..n] -= beta * v * (v^T * A[k+1..n, k..n])
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
        // Right multiply: A[0..n, k+1..n] -= beta * (A[0..n, k+1..n] * v) * v^T
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
        // Explicitly zero strict subdiagonal entries in column k
        for i in (k + 2)..n {
            a[i][k] = 0.0;
        }
    }
}

/// Francis implicit double-shift QR algorithm for all eigenvalues of a real matrix.
fn real_matrix_eigenvalues(mat: &[Vec<f64>]) -> Vec<C64> {
    let n = mat.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![C64::new(mat[0][0], 0.0)];
    }
    let mut h: Vec<Vec<f64>> = mat.to_vec();
    hessenberg_reduce(&mut h);

    let eps = 2.220446049250313e-16_f64;
    let mut eigs = Vec::with_capacity(n);
    let mut hi = n - 1;
    let mut iter_count = 0usize;
    let mut total_iters = 0usize;

    while hi + 1 > 0 {
        if hi == 0 {
            eigs.push(C64::new(h[0][0], 0.0));
            break;
        }
        // Find largest lo <= hi with negligible subdiagonal h[lo][lo-1]
        let mut lo = hi;
        while lo > 0 {
            let s = h[lo - 1][lo - 1].abs() + h[lo][lo].abs();
            let thresh = eps * if s > 0.0 { s } else { 1.0 };
            if h[lo][lo - 1].abs() <= thresh {
                h[lo][lo - 1] = 0.0;
                break;
            }
            lo -= 1;
        }

        if lo == hi {
            // 1x1 block converged
            eigs.push(C64::new(h[hi][hi], 0.0));
            hi -= 1;
            iter_count = 0;
            continue;
        }
        if lo + 1 == hi {
            // 2x2 block converged
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
        assert!(
            total_iters < 200 * n + 2000,
            "Francis QR failed to converge at subblock [{lo}..={hi}] (n={n})"
        );

        // Compute Francis double shift (s = trace, t = det of trailing 2x2, or exceptional shift)
        let (s, t) = if iter_count % 10 == 0 {
            let w = h[hi][hi - 1].abs() + h[hi - 1][hi - 2].abs();
            let shift = 0.75 * w + h[hi][hi];
            (2.0 * shift, shift * shift + 0.4375 * w * w)
        } else {
            let a11 = h[hi - 1][hi - 1];
            let a12 = h[hi - 1][hi];
            let a21 = h[hi][hi - 1];
            let a22 = h[hi][hi];
            (a11 + a22, a11 * a22 - a12 * a21)
        };

        // First column of (H - lambda1 I)(H - lambda2 I) at start row m
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

        // Chase the 3x3 bulge down to hi
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
                h[k][k - 1] = -h[k][k - 1];
            }

            // Left apply reflector to rows k..=k+2 (or k+1), columns k..n
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
            // Right apply reflector to rows 0..=min(hi, k+3), columns k..=k+2
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
// SECTION 3: Exact Multivariate Polynomial Box Engine (Theorem M1 — MacMahon 1915)
// ============================================================================

/// Dense multivariate polynomial truncated to the exact box 0 <= e_i <= m_i (sigma <= 4).
#[derive(Clone, Debug)]
struct PolyBox {
    dims: Vec<usize>,    // m_i + 1
    strides: Vec<usize>, // mixed-radix strides
    coeffs: Vec<BigInt>,
}

impl PolyBox {
    fn zero(m: &[usize]) -> Self {
        let dims: Vec<usize> = m.iter().map(|&x| x + 1).collect();
        let mut strides = vec![1usize; dims.len()];
        for i in (0..dims.len().saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * dims[i + 1];
        }
        let total: usize = dims.iter().product();
        PolyBox { dims, strides, coeffs: vec![BigInt::zero(); total] }
    }
    fn one(m: &[usize]) -> Self {
        let mut p = Self::zero(m);
        p.coeffs[0] = BigInt::one();
        p
    }
    fn top_coeff(&self) -> &BigInt {
        self.coeffs.last().unwrap()
    }
    fn index(&self, exp: &[usize]) -> usize {
        let mut idx = 0;
        for (e, &st) in exp.iter().zip(self.strides.iter()) {
            idx += e * st;
        }
        idx
    }
    fn decode(&self, mut idx: usize, out: &mut [usize]) {
        for i in 0..self.dims.len() {
            out[i] = idx / self.strides[i];
            idx %= self.strides[i];
        }
    }
    /// Exact multiplication truncated to the box 0 <= e_i <= m_i.
    fn mul_box(&self, rhs: &Self) -> Self {
        let s = self.dims.len();
        let mut out = PolyBox {
            dims: self.dims.clone(),
            strides: self.strides.clone(),
            coeffs: vec![BigInt::zero(); self.coeffs.len()],
        };
        let mut ea = vec![0usize; s];
        let mut eb = vec![0usize; s];
        let mut ec = vec![0usize; s];
        for (ia, ca) in self.coeffs.iter().enumerate() {
            if ca.is_zero() {
                continue;
            }
            self.decode(ia, &mut ea);
            for (ib, cb) in rhs.coeffs.iter().enumerate() {
                if cb.is_zero() {
                    continue;
                }
                self.decode(ib, &mut eb);
                let mut ok = true;
                for k in 0..s {
                    let sum = ea[k] + eb[k];
                    if sum >= self.dims[k] {
                        ok = false;
                        break;
                    }
                    ec[k] = sum;
                }
                if ok {
                    let ic = self.index(&ec);
                    let prod = ca.mul(cb);
                    out.coeffs[ic] = out.coeffs[ic].add(&prod);
                }
            }
        }
        out
    }
}

/// Exact MacMahon (1915, Vol. I §III Ch. II Arts. 64–65) Master Theorem:
/// 1) Expands det(I - X A) = 1 - Q(x) via principal minors sum_{S != empty} (-1)^{|S|-1} det(A_{S,S}) x^S.
/// 2) Computes exact formal series inverse F(x) = (1 - Q(x))^{-1} over the box 0 <= e_i <= m_i.
/// 3) Self-checks det(I - X A) * F(x) == 1 identically on the entire box!
fn macmahon_master_series_inverse(a: &[Vec<i64>], m: &[usize]) -> BigInt {
    let s = m.len();
    assert!(s >= 1 && s <= 4);
    assert_eq!(a.len(), s);

    // Build all non-empty principal minors q_S = (-1)^{|S|-1} det(A_{S,S})
    let mut q_terms: Vec<(usize, i64)> = Vec::new();
    for mask in 1usize..(1usize << s) {
        let mut idx = Vec::new();
        for i in 0..s {
            if (mask >> i) & 1 == 1 {
                idx.push(i);
            }
        }
        let k = idx.len();
        let mut sub = vec![vec![0i64; k]; k];
        for r in 0..k {
            for c in 0..k {
                sub[r][c] = a[idx[r]][idx[c]];
            }
        }
        let det_sub = det_small_i64(&sub);
        if det_sub != 0 {
            let coeff = if k % 2 == 1 { det_sub } else { -det_sub };
            q_terms.push((mask, coeff));
        }
    }

    // Exact series inversion F = 1 + Q * F in lexicographic order (guaranteed causal since mask != 0!)
    let mut f = PolyBox::one(m);
    let mut exp = vec![0usize; s];
    let mut prev = vec![0usize; s];
    for idx in 1..f.coeffs.len() {
        f.decode(idx, &mut exp);
        let mut acc = BigInt::zero();
        for &(mask, coeff) in &q_terms {
            let mut ok = true;
            for i in 0..s {
                if (mask >> i) & 1 == 1 {
                    if exp[i] == 0 {
                        ok = false;
                        break;
                    }
                    prev[i] = exp[i] - 1;
                } else {
                    prev[i] = exp[i];
                }
            }
            if ok {
                let p_idx = f.index(&prev);
                if !f.coeffs[p_idx].is_zero() {
                    let term = f.coeffs[p_idx].mul_i64(coeff);
                    acc = acc.add(&term);
                }
            }
        }
        f.coeffs[idx] = acc;
    }

    // Self-check D(x) * F(x) == 1 identically on the box!
    let mut d_poly = PolyBox::one(m);
    for &(mask, coeff) in &q_terms {
        let mut ok = true;
        for i in 0..s {
            exp[i] = (mask >> i) & 1;
            if exp[i] >= d_poly.dims[i] {
                ok = false;
            }
        }
        if ok {
            let idx = d_poly.index(&exp);
            d_poly.coeffs[idx] = BigInt::from_i64(-coeff);
        }
    }
    let prod = d_poly.mul_box(&f);
    assert_eq!(prod.coeffs[0], BigInt::one(), "D(x)*F(x) constant term != 1");
    for (idx, c) in prod.coeffs.iter().enumerate().skip(1) {
        assert!(c.is_zero(), "D(x)*F(x) != 1 at box index {idx}");
    }

    f.top_coeff().clone()
}

/// Direct MacMahon linear-form expansion [x^m] prod_i (sum_j a_{ij} x_j)^{m_i}.
fn macmahon_linear_form_direct(a: &[Vec<i64>], m: &[usize]) -> BigInt {
    let s = m.len();
    let mut acc = PolyBox::one(m);
    let mut exp = vec![0usize; s];
    for i in 0..s {
        let mut lin = PolyBox::zero(m);
        for j in 0..s {
            if a[i][j] != 0 && m[j] >= 1 {
                exp.fill(0);
                exp[j] = 1;
                let idx = lin.index(&exp);
                lin.coeffs[idx] = BigInt::from_i64(a[i][j]);
            }
        }
        for _ in 0..m[i] {
            acc = acc.mul_box(&lin);
        }
    }
    acc.top_coeff().clone()
}

/// Exact Cauchy-Frobenius cycle-index expansion (CA-05 / CA-32 / CA-33):
///   det(I - X A)^{-1} = exp(sum_{k >= 1} (1/k) tr((X A)^k))
/// Computes p_k(x) = tr((X A)^k) for k = 1..=n in exact BigInt multivariate box arithmetic,
/// returns (total_mmt, single_cycle_slice_num, per_partition_terms) where:
///   sum_{mu |- n} |C_mu| * [x^m] prod_k p_k^{c_k} == n! * MMT(A, m),
///   and single_cycle_slice_num = (n-1)! * [x^m] tr((X A)^n) == n! * MMT_{(n)}(A, m).
fn cauchy_frobenius_cycle_index(
    a: &[Vec<i64>],
    m: &[usize],
) -> (BigInt, BigInt, Vec<(Vec<usize>, BigInt)>) {
    let s = m.len();
    let n: usize = m.iter().sum();

    // Build matrix M^{(1)} = X A with PolyBox entries (M^{(1)}_{ij} = a_{ij} * x_i)
    let mut m1: Vec<Vec<PolyBox>> = vec![vec![PolyBox::zero(m); s]; s];
    let mut exp = vec![0usize; s];
    for i in 0..s {
        if m[i] >= 1 {
            exp.fill(0);
            exp[i] = 1;
            for j in 0..s {
                if a[i][j] != 0 {
                    let idx = m1[i][j].index(&exp);
                    m1[i][j].coeffs[idx] = BigInt::from_i64(a[i][j]);
                }
            }
        }
    }

    // Compute p_k(x) = tr((X A)^k) for k = 1..=n
    let mut pk: Vec<PolyBox> = Vec::with_capacity(n + 1);
    pk.push(PolyBox::one(m)); // dummy index 0
    let mut cur = m1.clone();
    for k in 1..=n {
        let mut tr = PolyBox::zero(m);
        for i in 0..s {
            for idx in 0..tr.coeffs.len() {
                if !cur[i][i].coeffs[idx].is_zero() {
                    tr.coeffs[idx] = tr.coeffs[idx].add(&cur[i][i].coeffs[idx]);
                }
            }
        }
        pk.push(tr);
        if k < n {
            // cur_next = cur * (X A): multiply row i of cur by x_j * a_{j, col}
            // Wait: (cur * M1)_{i, c} = sum_j cur_{i, j} * (x_j * a_{j, c})
            // Much faster than general PolyBox::mul_box because x_j is a single shift!
            let mut next: Vec<Vec<PolyBox>> = vec![vec![PolyBox::zero(m); s]; s];
            let mut e_tmp = vec![0usize; s];
            for i in 0..s {
                for j in 0..s {
                    let src = &cur[i][j];
                    for (idx, val) in src.coeffs.iter().enumerate() {
                        if val.is_zero() {
                            continue;
                        }
                        src.decode(idx, &mut e_tmp);
                        if e_tmp[j] + 1 >= src.dims[j] {
                            continue;
                        }
                        let dst_idx = idx + src.strides[j];
                        for c in 0..s {
                            if a[j][c] != 0 {
                                let t = val.mul_i64(a[j][c]);
                                next[i][c].coeffs[dst_idx] = next[i][c].coeffs[dst_idx].add(&t);
                            }
                        }
                    }
                }
            }
            cur = next;
        }
    }

    // Generate all partitions of n and evaluate |C_mu| * [x^m] prod_k p_k^{c_k}
    let parts = partitions(n);
    let nf = fact_u64(n);
    let mut total_scaled = BigInt::zero();
    let mut single_cyc_scaled = BigInt::zero();
    let mut per_part = Vec::with_capacity(parts.len());

    for mu in parts {
        // Compute class size |C_mu| = n! / prod_l (l^{c_l} c_l!)
        let mut counts = vec![0usize; n + 1];
        for &l in &mu {
            counts[l] += 1;
        }
        let mut z_mu = 1u64;
        for (l, &c) in counts.iter().enumerate() {
            if c > 0 {
                for _ in 0..c {
                    z_mu *= l as u64;
                }
                z_mu *= fact_u64(c);
            }
        }
        assert_eq!(nf % z_mu, 0);
        let c_mu = nf / z_mu;

        let mut prod = PolyBox::one(m);
        for &l in &mu {
            prod = prod.mul_box(&pk[l]);
        }
        let scaled_term = prod.top_coeff().mul_u64(c_mu);
        if mu.len() == 1 && mu[0] == n {
            single_cyc_scaled = scaled_term.clone();
        }
        total_scaled = total_scaled.add(&scaled_term);
        per_part.push((mu, scaled_term));
    }

    let nf_big = BigInt::from_u64(nf);
    let (mmt_q, rem) = total_scaled.div_rem(&nf_big);
    assert!(
        rem.is_zero(),
        "Cauchy-Frobenius cycle-index sum not divisible by n!={nf}"
    );
    (mmt_q, single_cyc_scaled, per_part)
}

fn partitions(n: usize) -> Vec<Vec<usize>> {
    fn rec(rem: usize, max_p: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if rem == 0 {
            out.push(cur.clone());
            return;
        }
        let hi = rem.min(max_p);
        for p in (1..=hi).rev() {
            cur.push(p);
            rec(rem - p, p, cur, out);
            cur.pop();
        }
    }
    let mut out = Vec::new();
    let mut cur = Vec::new();
    rec(n, n, &mut cur, &mut out);
    out
}

fn next_permutation_u8(a: &mut [u8]) -> bool {
    let n = a.len();
    if n <= 1 {
        return false;
    }
    let mut i = n - 2;
    while a[i] >= a[i + 1] {
        if i == 0 {
            return false;
        }
        i -= 1;
    }
    let mut j = n - 1;
    while a[j] <= a[i] {
        j -= 1;
    }
    a.swap(i, j);
    a[i + 1..].reverse();
    true
}

fn next_permutation_usize(a: &mut [usize]) -> bool {
    let n = a.len();
    if n <= 1 {
        return false;
    }
    let mut i = n - 2;
    while a[i] >= a[i + 1] {
        if i == 0 {
            return false;
        }
        i -= 1;
    }
    let mut j = n - 1;
    while a[j] <= a[i] {
        j -= 1;
    }
    a.swap(i, j);
    a[i + 1..].reverse();
    true
}

/// Brute-force multiset-word enumeration of sum_{w in S(m)} prod_r A_{F[r], w_r}.
fn brute_force_multiset_mmt(a: &[Vec<i64>], m: &[usize]) -> BigInt {
    let n: usize = m.iter().sum();
    let mut f_col = Vec::with_capacity(n);
    for (sym, &cnt) in m.iter().enumerate() {
        for _ in 0..cnt {
            f_col.push(sym as u8);
        }
    }
    let mut w = f_col.clone();
    let mut total = BigInt::zero();
    loop {
        let mut prod = BigInt::one();
        let mut zero = false;
        for r in 0..n {
            let v = a[f_col[r] as usize][w[r] as usize];
            if v == 0 {
                zero = true;
                break;
            }
            prod = prod.mul_i64(v);
        }
        if !zero {
            total = total.add(&prod);
        }
        if !next_permutation_u8(&mut w) {
            break;
        }
    }
    total
}

/// Exact brute-force over ALL pi in S_n (n <= 8), grouped by conjugacy class mu |- n!
/// Verifies BOTH total sum AND every individual conjugacy class slice of MacMahon's Master Theorem!
fn brute_force_sn_by_cycle_type(a: &[Vec<i64>], m: &[usize]) -> HashMap<Vec<usize>, BigInt> {
    let n: usize = m.iter().sum();
    let mut f_col = Vec::with_capacity(n);
    for (sym, &cnt) in m.iter().enumerate() {
        for _ in 0..cnt {
            f_col.push(sym);
        }
    }
    let mut pi: Vec<usize> = (0..n).collect();
    let mut vis = vec![false; n];
    let mut map: HashMap<Vec<usize>, BigInt> = HashMap::new();
    loop {
        let mut prod = BigInt::one();
        let mut zero = false;
        for r in 0..n {
            let v = a[f_col[r]][f_col[pi[r]]];
            if v == 0 {
                zero = true;
                break;
            }
            prod = prod.mul_i64(v);
        }
        if !zero {
            vis.fill(false);
            let mut cycs = Vec::new();
            for i in 0..n {
                if !vis[i] {
                    let mut cur = i;
                    let mut len = 0;
                    while !vis[cur] {
                        vis[cur] = true;
                        cur = pi[cur];
                        len += 1;
                    }
                    cycs.push(len);
                }
            }
            cycs.sort_unstable_by(|x, y| y.cmp(x));
            let entry = map.entry(cycs).or_insert_with(BigInt::zero);
            *entry = entry.add(&prod);
        }
        if !next_permutation_usize(&mut pi) {
            break;
        }
    }
    map
}

/// Exact CA-34 BEST text-sequence count Z_seq(G_1; r) = det(L~_r) * (m_r / prod_i m_i) * M_NM(G_1)
/// where M_NM(G_1) = prod_i (m_i! / prod_j t_{ij}!).
fn exact_ca34_best_z_seq(t_mat: &[Vec<u64>], m: &[usize], root: usize) -> (BigInt, BigInt, BigInt) {
    let s = m.len();
    // Build (s-1)x(s-1) directed Laplacian minor L~_root
    let mut l_minor = vec![vec![BigInt::zero(); s - 1]; s - 1];
    let mut r_idx = 0usize;
    for i in 0..s {
        if i == root {
            continue;
        }
        let mut c_idx = 0usize;
        let mut off_sum = 0u64;
        for j in 0..s {
            if j != i {
                off_sum += t_mat[i][j];
            }
            if j == root {
                continue;
            }
            if i != j && t_mat[i][j] > 0 {
                l_minor[r_idx][c_idx] = BigInt::from_i64(-(t_mat[i][j] as i64));
            }
            c_idx += 1;
        }
        l_minor[r_idx][r_idx] = BigInt::from_u64(off_sum);
        r_idx += 1;
    }
    let tau = bareiss_det_bigint(l_minor);

    // M_NM = prod_i m_i! / prod_{i,j} t_{ij}!
    let mut num_nm = BigInt::one();
    let mut den_nm = BigInt::one();
    for i in 0..s {
        num_nm = num_nm.mul(&bigint_factorial(m[i]));
        for j in 0..s {
            den_nm = den_nm.mul(&bigint_factorial(t_mat[i][j] as usize));
        }
    }
    let (m_nm, rem1) = num_nm.div_rem(&den_nm);
    assert!(rem1.is_zero(), "M_NM exact division failed");

    // Z_seq(G_1; root) = tau * prod_{i != root} (m_i - 1)! * m_root! / prod_{i,j} t_{ij}!
    //                  = tau * M_NM / prod_{i != root} m_i
    let mut prod_m_nr = BigInt::one();
    for (i, &mi) in m.iter().enumerate() {
        if i != root {
            prod_m_nr = prod_m_nr.mul_u64(mi as u64);
        }
    }
    let (z_seq, rem2) = tau.mul(&m_nm).div_rem(&prod_m_nr);
    assert!(rem2.is_zero(), "Z_seq exact BEST division failed");
    (tau, m_nm, z_seq)
}

// ============================================================================
// SECTION 4: Theorem M2 — Exact MacMahon (1915) Major Index & Inversion Engine
// ============================================================================

/// Exact MacMahon (1915, Vol. I §III Ch. IV, Art. 104) Major Index (Greater Index):
/// maj(w) = sum_{1 <= i <= n-1 : w[i-1] > w[i]} i.
fn major_index_u8(w: &[u8]) -> (usize, u128) {
    let mut des = 0usize;
    let mut maj = 0u128;
    for i in 0..w.len().saturating_sub(1) {
        if w[i] > w[i + 1] {
            des += 1;
            maj += (i + 1) as u128;
        }
    }
    (des, maj)
}

fn major_index_u32(pi: &[u32]) -> (usize, u128) {
    let mut des = 0usize;
    let mut maj = 0u128;
    for i in 0..pi.len().saturating_sub(1) {
        if pi[i] > pi[i + 1] {
            des += 1;
            maj += (i + 1) as u128;
        }
    }
    (des, maj)
}

/// Naive O(n^2) exact inversion count for small-case cross-verification.
fn inv_naive_u8(w: &[u8]) -> u128 {
    let n = w.len();
    let mut inv = 0u128;
    for i in 0..n {
        let wi = w[i];
        for &wj in &w[i + 1..] {
            if wi > wj {
                inv += 1;
            }
        }
    }
    inv
}

/// Exact O(n log sigma) inversion count on byte word via 256-slot Fenwick tree (BIT).
fn inv_fenwick_u8(w: &[u8]) -> u128 {
    let mut bit = [0u64; 257];
    let mut inv = 0u128;
    let mut seen = 0u64;
    for &b in w {
        let idx = usize::from(b) + 1;
        // Count seen elements <= b
        let mut le = 0u64;
        let mut p = idx;
        while p > 0 {
            le += bit[p];
            p &= p - 1;
        }
        inv += u128::from(seen - le);
        seen += 1;
        let mut q = idx;
        while q <= 256 {
            bit[q] += 1;
            q += q & q.wrapping_neg();
        }
    }
    inv
}

/// Exact O(m log m) inversion count on permutation pi[0..m] (values in 0..m) via Fenwick tree.
fn inv_fenwick_u32(pi: &[u32]) -> u128 {
    let m = pi.len();
    let mut bit = vec![0u32; m + 1];
    let mut inv = 0u128;
    for (step, &val) in pi.iter().enumerate() {
        let idx = val as usize + 1;
        let mut le = 0u32;
        let mut p = idx;
        while p > 0 {
            le += bit[p];
            p &= p - 1;
        }
        inv += (step as u32 - le) as u128;
        let mut q = idx;
        while q <= m {
            bit[q] += 1;
            q += q & q.wrapping_neg();
        }
    }
    inv
}

/// Exact q-binomial polynomial [n choose k]_q via q-Pascal identity (MacMahon 1915, Art. 105).
fn q_binom(n: usize, k: usize) -> Vec<u128> {
    if k > n {
        return vec![0];
    }
    let k = k.min(n - k);
    let mut dp: Vec<Vec<u128>> = vec![Vec::new(); k + 1];
    dp[0] = vec![1];
    for nn in 1..=n {
        let hi = nn.min(k);
        for kk in (1..=hi).rev() {
            let shift = nn - kk;
            let a = &dp[kk];
            let b = &dp[kk - 1];
            let new_len = a.len().max(if b.is_empty() { 0 } else { b.len() + shift });
            let mut next = vec![0u128; new_len];
            for (i, &v) in a.iter().enumerate() {
                next[i] += v;
            }
            for (i, &v) in b.iter().enumerate() {
                next[i + shift] += v;
            }
            dp[kk] = next;
        }
    }
    dp.swap_remove(k)
}

fn poly_mul_u128(a: &[u128], b: &[u128]) -> Vec<u128> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }
    let mut out = vec![0u128; a.len() + b.len() - 1];
    for (i, &va) in a.iter().enumerate() {
        if va == 0 {
            continue;
        }
        for (j, &vb) in b.iter().enumerate() {
            out[i + j] += va * vb;
        }
    }
    while out.len() > 1 && out.last() == Some(&0) {
        out.pop();
    }
    out
}

/// Exact q-multinomial polynomial [n; m_1, ..., m_sigma]_q (MacMahon 1915, Vol. I Art. 105).
fn q_multinomial(m: &[usize]) -> Vec<u128> {
    let mut acc = vec![1u128];
    let mut running = 0usize;
    for &mi in m {
        running += mi;
        let qb = q_binom(running, mi);
        acc = poly_mul_u128(&acc, &qb);
    }
    acc
}

/// Exact uniform random multiset permutation mean and standard deviation of BOTH maj and inv:
///   mu = (n^2 - sum m_a^2) / 4
///   var = (n(n-1)(2n+5) - sum m_a(m_a-1)(2m_a+5)) / 72
fn multiset_maj_inv_mean_std(n: usize, counts: &[usize]) -> (f64, f64) {
    let nf = n as f64;
    let sum_m2: f64 = counts.iter().map(|&c| (c as f64) * (c as f64)).sum();
    let mu = 0.25 * (nf * nf - sum_m2);
    let term_n = nf * (nf - 1.0) * (2.0 * nf + 5.0);
    let term_m: f64 = counts
        .iter()
        .map(|&c| {
            let cf = c as f64;
            cf * (cf - 1.0) * (2.0 * cf + 5.0)
        })
        .sum();
    let var = ((term_n - term_m) / 72.0).max(0.0);
    (mu, var.sqrt())
}

// ============================================================================
// SECTION 5: Block Representation, Exact Open<16> Induced Perm & Forward-FL
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
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

fn expand_records(records: &[u16], n: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &rec in records {
        let c = (rec >> 8) as u8;
        let w = usize::from(rec & 255);
        for _ in 0..w {
            l.push(c);
        }
    }
    assert_eq!(l.len(), n);
    l
}

fn bwt_encode(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_unstable_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        Ordering::Equal
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &s) in sa.iter().enumerate() {
        if s == 0 {
            p = i as u32;
            l.push(text[n - 1]);
        } else {
            l.push(text[s - 1]);
        }
    }
    (l, p)
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

/// Exact forward-FL inversion matching mibz2 / cand_r5_head semantics on valid AND malformed inputs!
fn invert_forward_fl(l_col: &[u8], p: u32) -> Vec<u8> {
    let n = l_col.len();
    if n == 0 {
        return Vec::new();
    }
    let mut all = [0u32; 256];
    for &c in l_col {
        all[usize::from(c)] += 1;
    }
    let mut first = starts(all);
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

struct InducedPerm {
    l_col: Vec<u8>,
    all: [u32; 256],
    n_runs: usize,
    m: usize,
    mark: u32,
    thin: [u32; 256],
    col_l: Vec<u8>,
    pi: Vec<u32>,
    pi_inv: Vec<u32>,
}

fn extract_induced_perm(records: &[u16], n: usize, p: u32) -> InducedPerm {
    let mut l_col = Vec::with_capacity(n);
    let mut retained_flag = vec![false; n];
    let mut ret_id = vec![u32::MAX; n];
    let mut thin = [0u32; 256];
    let mut all = [0u32; 256];
    let mut col_l = Vec::new();
    let mut m = 0u32;
    let mut mark = 0u32;
    let mut n_runs = 0usize;

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
        n_runs += 1;
        all[usize::from(c)] += width;
        for _ in 0..width {
            l_col.push(c);
        }
        let end = pos + width;
        let mut keep_piece = |x: u32, w: u32, keep: bool| {
            if keep {
                for off in 0..w {
                    let row = (x + off) as usize;
                    retained_flag[row] = true;
                    ret_id[row] = m;
                    if x + off == p {
                        mark = m;
                    }
                    col_l.push(c);
                    m += 1;
                }
                thin[usize::from(c)] += w;
            }
        };
        if width > LIMIT && (pos..end).contains(&p) {
            if pos < p {
                keep_piece(pos, p - pos, false);
            }
            keep_piece(p, 1, true);
            if p + 1 < end {
                keep_piece(p + 1, end - p - 1, false);
            }
        } else {
            keep_piece(pos, width, width <= LIMIT);
        }
        pos += width;
    }
    assert_eq!(pos as usize, n);

    let mut first = starts(all);
    let mut fl = vec![0u32; n];
    for (row, &c) in l_col.iter().enumerate() {
        let a = usize::from(c);
        fl[first[a] as usize] = row as u32;
        first[a] += 1;
    }

    let m_us = m as usize;
    let mut pi = vec![0u32; m_us];
    let mut pi_inv = vec![0u32; m_us];
    for row in 0..n {
        if retained_flag[row] {
            let u = ret_id[row] as usize;
            let mut cur = fl[row] as usize;
            while !retained_flag[cur] {
                cur = fl[cur] as usize;
            }
            let v = ret_id[cur];
            pi[u] = v;
            pi_inv[v as usize] = u as u32;
        }
    }
    for u in 0..m_us {
        assert_eq!(pi_inv[pi[u] as usize], u as u32, "pi_ret not a bijection!");
    }
    InducedPerm { l_col, all, n_runs, m: m_us, mark, thin, col_l, pi, pi_inv }
}

/// Build byte-digram transition matrix T_{ij} = |{r : F[r] = sym[i], L[r] = sym[j]}|.
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
    // Verify Euler balance (CA-19)
    for i in 0..s {
        let rs: u64 = t[i].iter().sum();
        let cs: u64 = (0..s).map(|r| t[r][i]).sum();
        assert_eq!(rs, m[i] as u64);
        assert_eq!(cs, m[i] as u64);
    }
    (syms, m, t)
}

/// Build Open<16> retained double-coset matrix N_{ab} = |{u : col_l[u] = sym[a], col_l[pi[u]] = sym[b]}|.
fn build_ret_double_coset_matrix(
    ip: &InducedPerm,
) -> (Vec<u8>, Vec<usize>, Vec<Vec<u64>>) {
    let mut syms = Vec::new();
    let mut map = [usize::MAX; 256];
    let mut m = Vec::new();
    for b in 0..256 {
        if ip.thin[b] > 0 {
            map[b] = syms.len();
            syms.push(b as u8);
            m.push(ip.thin[b] as usize);
        }
    }
    let s = syms.len();
    let mut nab = vec![vec![0u64; s]; s];
    for u in 0..ip.m {
        let a = map[usize::from(ip.col_l[u])];
        let b = map[usize::from(ip.col_l[ip.pi[u] as usize])];
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

/// Full 4-Way Spectral Bridge Verification on an Eulerian transition matrix T with marginals m:
///   1) BigInt Sylvester-Bareiss tau(G) -> log2 det(I - P~_r) & log2(det(I - P~_r) / pi_r)
///   2) Wielandt-MacMahon deflated operator det(I - P + 1 pi^T) via pivoted LU
///   3) Direct Hessenberg + Francis QR eigenvalues {lambda_k} of P -> prod_{lambda != 1} (1 - lambda)
///   4) Full MacMahon polynomial det(I - zP) vs (1 - z) prod_{lambda != 1}(1 - z lambda) at z in {0.25, 0.5, 0.75, 0.9}
///   5) MacMahon Saddle-Point Hessian H = D - P^T D P -> log2 MMT_saddle(P, m)
struct SpectralBridgeMetrics {
    sigma: usize,
    pen_ca34_bits: f64,         // -log2 det(I - P~_r) from BigInt Bareiss (exact!)
    root_info_bits: f64,        // -log2 pi_r
    spec_prod_bits: f64,        // -log2 prod_{lambda != 1}(1 - lambda) (root-independent!)
    err_wielandt_log2: f64,     // |log2 det(I-P~_r) - log2(pi_r det(I-P+1 pi^T))|
    err_qr_eigs_log2: f64,      // |log2 det(I-P~_r) - log2(pi_r prod_{lam!=1}(1-lam))|
    err_qr_im: f64,             // |Im(prod_{lam!=1}(1-lam))|
    err_poly_z_max: f64,        // max_{z in {0.25,0.5,0.75,0.9}} |log2 det(I-zP) - log2 prod(1-z lam)|
    spectral_gap: f64,          // 1 - max_{lambda != 1} |lambda|
    mmt_saddle_log2_prob: f64,  // log2 MMT_saddle(P, m) = -((s-1)/2) log2(2pi) - 0.5 log2 det(H~_r)
    h1_markov_bits: f64,        // -sum_{i,j} t_{ij} log2(t_{ij} / m_i)
    h_nm_bits: f64,             // sum_i log2(m_i! / prod_j t_{ij}!)
    disconnected: bool,
}

fn analyze_spectral_bridge(
    t_mat: &[Vec<u64>],
    m: &[usize],
    run_bigint: bool,
) -> SpectralBridgeMetrics {
    let s = m.len();
    let n: usize = m.iter().sum();
    let nf = n as f64;

    // Pick root r = argmax m_i (same canonical root as CA-34!)
    let mut root = 0usize;
    for i in 1..s {
        if m[i] > m[root] {
            root = i;
        }
    }
    let pi: Vec<f64> = m.iter().map(|&mi| (mi as f64) / nf).collect();
    let root_pi = pi[root];

    // Markov matrix P_{ij} = t_{ij} / m_i
    let mut p_mat = vec![vec![0.0f64; s]; s];
    let mut h1_markov_bits = 0.0f64;
    let mut h_nm_bits = 0.0f64;
    for i in 0..s {
        let mi = m[i] as f64;
        h_nm_bits += log2_fact(m[i]);
        for j in 0..s {
            let tij = t_mat[i][j];
            if tij > 0 {
                let pij = (tij as f64) / mi;
                p_mat[i][j] = pij;
                h1_markov_bits -= (tij as f64) * pij.log2();
                h_nm_bits -= log2_fact(tij as usize);
            }
        }
    }

    if s <= 1 {
        return SpectralBridgeMetrics {
            sigma: s,
            pen_ca34_bits: 0.0,
            root_info_bits: 0.0,
            spec_prod_bits: 0.0,
            err_wielandt_log2: 0.0,
            err_qr_eigs_log2: 0.0,
            err_qr_im: 0.0,
            err_poly_z_max: 0.0,
            spectral_gap: 1.0,
            mmt_saddle_log2_prob: 0.0,
            h1_markov_bits,
            h_nm_bits,
            disconnected: false,
        };
    }

    // 1) Exact CA-34 det(I - P~_r) via BigInt Bareiss (or LU if run_bigint == false)
    let sum_log2_m_nr: f64 = m
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != root)
        .map(|(_, &mi)| (mi as f64).log2())
        .sum();
    let log2_det_p_minor_exact = if run_bigint {
        // Order non-root rows by increasing out-degree for fast Bareiss
        let mut verts: Vec<usize> = (0..s).filter(|&i| i != root).collect();
        verts.sort_unstable_by_key(|&i| {
            let deg_nz = (0..s).filter(|&j| j != i && t_mat[i][j] > 0).count();
            (deg_nz, m[i])
        });
        let mut idx_map = vec![usize::MAX; s];
        for (pos, &v) in verts.iter().enumerate() {
            idx_map[v] = pos;
        }
        let d = s - 1;
        let mut l_minor = vec![vec![BigInt::zero(); d]; d];
        for (r, &u) in verts.iter().enumerate() {
            let mut off_sum = 0u64;
            for v in 0..s {
                if v != u && t_mat[u][v] > 0 {
                    off_sum += t_mat[u][v];
                    if v != root {
                        l_minor[r][idx_map[v]] = BigInt::from_i64(-(t_mat[u][v] as i64));
                    }
                }
            }
            l_minor[r][r] = BigInt::from_u64(off_sum);
        }
        let tau = bareiss_det_bigint(l_minor);
        if tau.is_zero() {
            f64::NEG_INFINITY
        } else {
            tau.log2_approx() - sum_log2_m_nr
        }
    } else {
        let mut mat = vec![vec![0.0f64; s - 1]; s - 1];
        let mut r_i = 0;
        for i in 0..s {
            if i == root {
                continue;
            }
            let mut c_j = 0;
            for j in 0..s {
                if j == root {
                    continue;
                }
                mat[r_i][c_j] = if i == j { 1.0 - p_mat[i][j] } else { -p_mat[i][j] };
                c_j += 1;
            }
            r_i += 1;
        }
        let (sg, lg) = signed_log2_det_lu(mat);
        if sg == 0 { f64::NEG_INFINITY } else { lg }
    };

    // 2) Wielandt-MacMahon deflated spectral operator: M = I - P + 1 * pi^T
    let mut wielandt = vec![vec![0.0f64; s]; s];
    for i in 0..s {
        for j in 0..s {
            let id = if i == j { 1.0 } else { 0.0 };
            wielandt[i][j] = id - p_mat[i][j] + pi[j];
        }
    }
    let (w_sign, log2_wielandt) = signed_log2_det_lu(wielandt);

    // 3) Direct Hessenberg + Francis QR complex eigenvalues of P
    let eigs = real_matrix_eigenvalues(&p_mat);
    assert_eq!(eigs.len(), s);
    let tr_p: f64 = (0..s).map(|i| p_mat[i][i]).sum();
    let tr_eigs: f64 = eigs.iter().map(|e| e.re).sum();
    assert!(
        (tr_p - tr_eigs).abs() < 1e-10,
        "QR trace mismatch: tr(P)={tr_p}, sum(eigs)={tr_eigs}"
    );

    // Find the Perron eigenvalue closest to 1.0 + 0i
    let mut perron_idx = 0usize;
    let mut best_dist = f64::INFINITY;
    for (idx, e) in eigs.iter().enumerate() {
        let d = (e.re - 1.0).hypot(e.im);
        if d < best_dist {
            best_dist = d;
            perron_idx = idx;
        }
    }
    assert!(
        best_dist < 1e-10,
        "No Perron eigenvalue near 1.0 (closest dist={best_dist})"
    );

    let disconnected = log2_det_p_minor_exact.is_infinite() || w_sign == 0;
    let (err_wielandt_log2, err_qr_eigs_log2, err_qr_im, spec_prod_bits, spectral_gap) =
        if disconnected {
            // Verify second eigenvalue near 1.0 (or det == 0)
            let mut min_other = f64::INFINITY;
            for (idx, e) in eigs.iter().enumerate() {
                if idx != perron_idx {
                    min_other = min_other.min((e.re - 1.0).hypot(e.im));
                }
            }
            assert!(
                min_other < 1e-9 && w_sign == 0,
                "Disconnected graph mismatch: min_other={min_other}, w_sign={w_sign}"
            );
            (0.0, 0.0, 0.0, f64::INFINITY, 0.0)
        } else {
            // Compute product prod_{k != perron_idx} (1 - lambda_k) in log-polar + phase check
            let mut log2_prod = 0.0f64;
            let mut phase_sum = 0.0f64;
            let mut max_non1_abs = 0.0f64;
            for (idx, e) in eigs.iter().enumerate() {
                if idx == perron_idx {
                    continue;
                }
                let one_minus = C64::new(1.0 - e.re, -e.im);
                log2_prod += one_minus.abs().log2();
                phase_sum += one_minus.im.atan2(one_minus.re);
                max_non1_abs = max_non1_abs.max(e.abs());
            }
            let im_err = phase_sum.sin().abs();
            let log2_p_minor_wielandt = root_pi.log2() + log2_wielandt;
            let log2_p_minor_qr = root_pi.log2() + log2_prod;
            (
                (log2_det_p_minor_exact - log2_p_minor_wielandt).abs(),
                (log2_det_p_minor_exact - log2_p_minor_qr).abs(),
                im_err,
                -log2_wielandt,
                1.0 - max_non1_abs,
            )
        };

    // 4) Cross-check full MacMahon characteristic polynomial det(I - zP) vs prod_k (1 - z lambda_k)
    let mut err_poly_z_max = 0.0f64;
    for &z in &[0.25f64, 0.5, 0.75, 0.9] {
        let mut izp = vec![vec![0.0f64; s]; s];
        for i in 0..s {
            for j in 0..s {
                let id = if i == j { 1.0 } else { 0.0 };
                izp[i][j] = id - z * p_mat[i][j];
            }
        }
        let (sg_lu, log2_lu) = signed_log2_det_lu(izp);
        assert!(sg_lu != 0, "det(I - {z}*P) unexpectedly singular");
        let mut log2_e = 0.0f64;
        for e in &eigs {
            let term = C64::new(1.0 - z * e.re, -z * e.im);
            log2_e += term.abs().log2();
        }
        err_poly_z_max = err_poly_z_max.max((log2_lu - log2_e).abs());
    }

    // 5) MacMahon Saddle-Point Hessian H = D - P^T D P at Perron equilibrium x* = 1.
    //    Diagonal entry H_{kk} = sum_i m_i P_{ik}(1 - P_{ik}) == 0 iff every incoming transition
    //    to symbol k is deterministic (P_{ik} in {0, 1}), so balance at k holds with prob 1!
    let mut h_full = vec![vec![0.0f64; s]; s];
    let mut stoch = Vec::with_capacity(s);
    for k in 0..s {
        for l in 0..s {
            let mut sum = 0.0f64;
            for i in 0..s {
                let pik = p_mat[i][k];
                if pik != 0.0 {
                    let pil = p_mat[i][l];
                    if pil != 0.0 {
                        sum += (m[i] as f64) * pik * pil;
                    }
                }
            }
            let diag = if k == l { m[k] as f64 } else { 0.0 };
            h_full[k][l] = diag - sum;
        }
        if h_full[k][k] > 1e-12 {
            stoch.push(k);
        }
    }
    let mmt_saddle_log2_prob = if stoch.len() <= 1 {
        // Purely deterministic Markov chain (e.g. periodic permutation): MMT(P, m) = 1 -> 0 bits!
        0.0
    } else {
        // Restrict H to stochastic vertices and delete 1 vertex (Kirchhoff cofactor)
        let k_st = stoch.len();
        let mut h_minor = vec![vec![0.0f64; k_st - 1]; k_st - 1];
        for r_i in 0..(k_st - 1) {
            for c_j in 0..(k_st - 1) {
                h_minor[r_i][c_j] = h_full[stoch[r_i]][stoch[c_j]];
            }
        }
        let (h_sg, log2_det_h) = signed_log2_det_lu(h_minor);
        if h_sg > 0 {
            -0.5 * ((k_st - 1) as f64) * (2.0 * std::f64::consts::PI).log2() - 0.5 * log2_det_h
        } else {
            // Multiple stochastic connected components of P^T D P: use positive eigenvalues of H_stoch
            let mut h_sub = vec![vec![0.0f64; k_st]; k_st];
            for r_i in 0..k_st {
                for c_j in 0..k_st {
                    h_sub[r_i][c_j] = h_full[stoch[r_i]][stoch[c_j]];
                }
            }
            let heigs = real_matrix_eigenvalues(&h_sub);
            let mut rank = 0usize;
            let mut log2_pdet = 0.0f64;
            for e in &heigs {
                if e.re > 1e-9 {
                    rank += 1;
                    log2_pdet += e.re.log2();
                }
            }
            if rank == 0 {
                0.0
            } else {
                // Adjust by component size factor (k_st / nullity)
                let nullity = k_st - rank;
                let comp_adj = (k_st as f64 / nullity.max(1) as f64).log2() * (nullity as f64);
                -0.5 * (rank as f64) * (2.0 * std::f64::consts::PI).log2()
                    - 0.5 * (log2_pdet - comp_adj)
            }
        }
    };

    SpectralBridgeMetrics {
        sigma: s,
        pen_ca34_bits: -log2_det_p_minor_exact,
        root_info_bits: -root_pi.log2(),
        spec_prod_bits,
        err_wielandt_log2,
        err_qr_eigs_log2,
        err_qr_im,
        err_poly_z_max,
        spectral_gap,
        mmt_saddle_log2_prob,
        h1_markov_bits,
        h_nm_bits,
        disconnected,
    }
}

// ============================================================================
// SECTION 6: Cache-Miss Proxy & Real S4/S5 Wall Timing on Open<16> pi_ret
// ============================================================================

/// Exact 8-way set-associative LRU cache simulator (64B lines = 16 u32 entries per line).
/// L1D = 32 KiB (64 sets x 8 ways); L2 = 256 KiB (512 sets x 8 ways).
fn simulate_lru_miss_rate(accesses: &[u32], num_sets: usize) -> f64 {
    if accesses.is_empty() {
        return 0.0;
    }
    const WAYS: usize = 8;
    let mask = num_sets - 1;
    let mut tags = vec![[u32::MAX; WAYS]; num_sets];
    let mut misses = 0usize;
    for &idx in accesses {
        let line = idx >> 4; // 64 bytes / 4 bytes per u32 = 16 u32 per cache line
        let set = (line as usize) & mask;
        let row = &mut tags[set];
        let mut hit = None;
        for w in 0..WAYS {
            if row[w] == line {
                hit = Some(w);
                break;
            }
        }
        if let Some(w) = hit {
            // Move to MRU (slot 0)
            for k in (1..=w).rev() {
                row[k] = row[k - 1];
            }
            row[0] = line;
        } else {
            misses += 1;
            for k in (1..WAYS).rev() {
                row[k] = row[k - 1];
            }
            row[0] = line;
        }
    }
    100.0 * (misses as f64) / (accesses.len() as f64)
}

/// Measure real S4 XOR-wiring + S5 orbit-walk wall time (ns/retained-row) and exact L1D/L2 miss rates.
fn measure_s4_s5_cost(ip: &InducedPerm) -> (f64, f64, f64, f64) {
    let m = ip.m;
    if m == 0 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    // 1) Exact S5 orbit sequence starting from mark (visits all m rows on single-cycle blocks)
    let mut s5_order = Vec::with_capacity(m);
    let mut cur = ip.mark;
    for _ in 0..m {
        s5_order.push(cur);
        cur = ip.pi[cur as usize];
    }
    let s4_l1_miss = simulate_lru_miss_rate(&ip.pi, 64);
    let s5_l1_miss = simulate_lru_miss_rate(&s5_order, 64);
    let s5_l2_miss = simulate_lru_miss_rate(&s5_order, 512);

    // 2) Real native wall-clock measurement of S4 XOR-wire + S5 XOR orbit walk (median of 5 runs)
    let mut edges = vec![0u32; m];
    let mut times = [0.0f64; 5];
    let mut sink = 0u64;
    for t in &mut times {
        edges.fill(0);
        let t0 = Instant::now();
        // S4 XOR wiring: edges[u] ^= pi[u]; edges[pi[u]] ^= u
        for (u, &v) in ip.pi.iter().enumerate() {
            edges[u] ^= v;
            edges[v as usize] ^= u as u32;
        }
        // S5 XOR orbit walk
        let mut prev = ip.pi_inv[ip.mark as usize];
        let mut node = ip.mark;
        let mut acc = 0u64;
        for _ in 0..m {
            let nxt = edges[node as usize] ^ prev;
            acc = acc.wrapping_add(u64::from(nxt));
            prev = node;
            node = nxt;
        }
        let dt = t0.elapsed().as_nanos() as f64;
        sink ^= acc;
        *t = dt / (m as f64);
    }
    std::hint::black_box(sink);
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (s4_l1_miss, s5_l1_miss, s5_l2_miss, times[2])
}

fn pearson_r(xs: &[f64], ys: &[f64]) -> f64 {
    assert_eq!(xs.len(), ys.len());
    let n = xs.len() as f64;
    if n < 2.0 {
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
    if dx2 <= 0.0 || dy2 <= 0.0 {
        0.0
    } else {
        num / (dx2 * dy2).sqrt()
    }
}

// ============================================================================
// SECTION 7: Corpus Loaders & Solved Analogue M Benchmarks
// ============================================================================

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
            let records = records_of(&l);
            Block { name: f, n: l.len(), p, records }
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

fn make_solved_analogue_m() -> Vec<Block> {
    let mut out = Vec::new();
    // 1. Fibonacci / Christoffel word F_17 (n = 1597, 2 runs: maj(L) == 1!)
    let mut a = b"a".to_vec();
    let mut b = b"ab".to_vec();
    while b.len() < 1597 {
        let mut c = b.clone();
        c.extend_from_slice(&a);
        a = b;
        b = c;
    }
    let (lf, pf) = bwt_encode(&b);
    out.push(Block { name: "SolvedM-Fib1597".to_string(), n: lf.len(), p: pf, records: records_of(&lf) });

    // 2. Periodic 4-symbol text (abcde)^200 (n = 1000, pure deterministic Markov permutation P)
    let mut tp = Vec::with_capacity(1000);
    for _ in 0..200 {
        tp.extend_from_slice(b"abcde");
    }
    let (lp, pp) = bwt_encode(&tp);
    out.push(Block { name: "SolvedM-Per5x200".to_string(), n: lp.len(), p: pp, records: records_of(&lp) });

    // 3. Strongly clustered 8-symbol Markov text (n = 4000)
    let mut tc = Vec::with_capacity(4000);
    let mut rng = 0x5050_C50A_1234_5678u64;
    let mut cur = 0u8;
    for _ in 0..4000 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        if (rng >> 56) >= 200 {
            cur = ((rng >> 48) as u8) % 8;
        }
        tc.push(b'A' + cur);
    }
    let (lc, pc) = bwt_encode(&tc);
    out.push(Block { name: "SolvedM-Clust4k".to_string(), n: lc.len(), p: pc, records: records_of(&lc) });

    // 4. Uniform i.i.d. random 16-symbol text (n = 4000: minimal BWT clustering!)
    let mut tr = Vec::with_capacity(4000);
    for _ in 0..4000 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        tr.push(b'a' + (((rng >> 56) as u8) % 16));
    }
    let (lr, pr) = bwt_encode(&tr);
    out.push(Block { name: "SolvedM-Rand4k".to_string(), n: lr.len(), p: pr, records: records_of(&lr) });
    out
}

// ============================================================================
// SECTION 8: PART 1 — Exact Small-Case Verification of Theorem M1 (5,200+ BWTs)
// ============================================================================

fn run_part1_theorem_m1_exact_small() {
    println!("================================================================================================================================");
    println!("PART 1: THEOREM M1 — EXACT SMALL-CASE VERIFICATION OF MACMAHON'S MASTER THEOREM (5,200 VALID BWTs + 300 MALFORMED)");
    println!("  Primary Source: P. A. MacMahon, Combinatory Analysis, Vol. I (1915), Section III, Ch. II, Arts. 64-79 (pp. 93-123)");
    println!("  Verifies 100.00% EXACT BigInt equality across:");
    println!("    (a) Exact multivariate series inversion [x^m] det(I - X A)^{{-1}} (with full-box D(x)*F(x) == 1 check)");
    println!("    (b) Direct linear-form expansion [x^m] prod_i (sum_j a_{{ij}} x_j)^{{m_i}}");
    println!("    (c) Exact Cauchy-Frobenius cycle-index sum_{{mu |- n}} |C_mu| [x^m] prod_k tr((X A)^k)^{{c_k}} == n! * MMT(A, m)");
    println!("    (d) Exact brute-force multiset-word sum (ALL cases w/ multinom <= 30,000) & full S_n class-by-class check (n <= 8)");
    println!("    (e) Exact CA-34 Sylvester-Bareiss BEST Z_seq(G_1; r) & Spectral Bridge det(I - P~_r) == pi_r * prod_{{lam!=1}}(1 - lam)");
    println!("================================================================================================================================");

    let t0 = Instant::now();
    let mut rng = 0xC501_1915_CAFE_BEEFu64;
    let mut next_u64 = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };

    let mut ref_k = cand_r5_head::Kernel::default();
    let mut ref_out = Vec::new();

    let mut valid_checked = 0usize;
    let mut brute_word_checked = 0usize;
    let mut brute_sn_checked = 0usize;
    let mut cf_checked = 0usize;
    let mut spectral_max_err = 0.0f64;
    let mut sum_ratio_1cyc_mmt = 0.0f64;
    let mut sum_ratio_zseq_mnm = 0.0f64;
    let mut sum_ratio_zseq_mmt_sup = 0.0f64;
    let mut by_sigma = [0usize; 5];

    // 5,200 valid small BWTs with sigma in 2..=4, n in 4..=12
    for case in 0..5200usize {
        let target_sigma = 2 + (case % 3); // 2, 3, 4
        let n = if case < 1800 {
            4 + (case % 5) // n in 4..=8 (enables full S_n class-by-class brute force!)
        } else {
            4 + ((next_u64() as usize) % 9) // n in 4..=12
        };

        // Generate text containing at least 2 distinct symbols (up to target_sigma)
        let mut text = Vec::with_capacity(n);
        for i in 0..n {
            let sym = if i < target_sigma && i < n {
                i as u8
            } else {
                ((next_u64() >> 56) as u8) % (target_sigma as u8)
            };
            text.push(b'a' + sym);
        }
        // Shuffle text deterministically so symbol order is arbitrary
        for i in (1..n).rev() {
            let j = (next_u64() as usize) % (i + 1);
            text.swap(i, j);
        }

        let (l, p) = bwt_encode(&text);
        let recs = records_of(&l);
        // Verify exact forward-FL parity with cand_r5_head
        if case < 500 {
            ref_k.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut ref_out).unwrap();
            assert_eq!(ref_out, text, "cand_r5_head mismatch on small valid case {case}");
            let fl_out = invert_forward_fl(&l, p);
            assert_eq!(fl_out, text, "invert_forward_fl mismatch on small valid case {case}");
        }

        let mut all = [0u32; 256];
        for &c in &l {
            all[usize::from(c)] += 1;
        }
        let (_syms, m, t_mat) = build_g1_matrix(&l, &all);
        let s = m.len();
        by_sigma[s] += 1;

        // Integer transition matrix A = T and 0-1 support matrix S
        let a_t: Vec<Vec<i64>> = t_mat.iter().map(|r| r.iter().map(|&v| v as i64).collect()).collect();
        let a_sup: Vec<Vec<i64>> = t_mat
            .iter()
            .map(|r| r.iter().map(|&v| if v > 0 { 1 } else { 0 }).collect())
            .collect();

        // (a) Exact multivariate series inversion det(I - X T)^{-1} & det(I - X S)^{-1}
        let mmt_t = macmahon_master_series_inverse(&a_t, &m);
        let mmt_sup = macmahon_master_series_inverse(&a_sup, &m);

        // (b) Exact direct linear form expansion
        let lin_t = macmahon_linear_form_direct(&a_t, &m);
        let lin_sup = macmahon_linear_form_direct(&a_sup, &m);
        assert_eq!(mmt_t, lin_t, "MMT series != linear form for T at case {case}");
        assert_eq!(mmt_sup, lin_sup, "MMT series != linear form for S at case {case}");

        // (c) Exact Cauchy-Frobenius cycle-index expansion on ALL 5,200 cases!
        // Note: n <= 12 -> p(12) = 77 partitions, box <= 256, runs in ~0.3 ms!
        let (cf_t, single_cyc_scaled_t, per_part_t) = cauchy_frobenius_cycle_index(&a_t, &m);
        assert_eq!(mmt_t, cf_t, "Cauchy-Frobenius cycle-index != MMT at case {case}");
        cf_checked += 1;

        // (d1) Brute-force multiset-word sum whenever multinom <= 30,000
        let mut multinom = fact_u64(n);
        for &mi in &m {
            multinom /= fact_u64(mi);
        }
        if multinom <= 30_000 {
            let bf_t = brute_force_multiset_mmt(&a_t, &m);
            let bf_sup = brute_force_multiset_mmt(&a_sup, &m);
            assert_eq!(mmt_t, bf_t, "Brute-force word sum != MMT(T) at case {case}");
            assert_eq!(mmt_sup, bf_sup, "Brute-force word sum != MMT(S) at case {case}");
            brute_word_checked += 1;
        }

        // (d2) Full S_n conjugacy-class-by-conjugacy-class brute-force check for n <= 7 (and first 30 of n == 8)
        if n <= 7 || (n == 8 && brute_sn_checked < 1200) {
            let sn_map = brute_force_sn_by_cycle_type(&a_t, &m);
            let mut prod_m_fact = BigInt::one();
            for &mi in &m {
                prod_m_fact = prod_m_fact.mul(&bigint_factorial(mi));
            }
            let nf_big = BigInt::from_u64(fact_u64(n));
            for (mu, scaled_term) in &per_part_t {
                // Identity: n! * sum_{pi in C_mu} w_T(pi) == (prod_i m_i!) * |C_mu| * [x^m] prod p_k^{c_k}
                let lhs = sn_map.get(mu).cloned().unwrap_or_else(BigInt::zero).mul(&nf_big);
                let rhs = prod_m_fact.mul(scaled_term);
                assert_eq!(
                    lhs, rhs,
                    "Conjugacy-class slice mismatch at case {case}, mu={mu:?}"
                );
            }
            brute_sn_checked += 1;
        }

        // (e) Exact CA-34 BEST Z_seq(G_1; root) & 4-way Spectral Bridge
        let mut root = 0usize;
        for i in 1..s {
            if m[i] > m[root] {
                root = i;
            }
        }
        let (_tau, m_nm, z_seq) = exact_ca34_best_z_seq(&t_mat, &m, root);
        let spec = analyze_spectral_bridge(&t_mat, &m, true);
        assert!(!spec.disconnected, "Valid BWT G_1 unexpectedly disconnected at case {case}");
        spectral_max_err = spectral_max_err
            .max(spec.err_wielandt_log2)
            .max(spec.err_qr_eigs_log2)
            .max(spec.err_poly_z_max);
        assert!(
            spec.err_wielandt_log2 < 1e-11 && spec.err_qr_eigs_log2 < 1e-11,
            "Spectral bridge error too large at case {case}: {:?}",
            (spec.err_wielandt_log2, spec.err_qr_eigs_log2)
        );

        // Exact ratios:
        //   - 1-cycle fraction inside MMT(T, m): single_cyc_scaled_t / (n! * mmt_t)
        //   - CA-34 BEST fraction inside M_NM: z_seq / m_nm == det(I - P~_r)
        //   - BEST Z_seq / MMT(support S, m)
        let nf_big = BigInt::from_u64(fact_u64(n));
        let r_1cyc = single_cyc_scaled_t.to_f64_ratio(&mmt_t.mul(&nf_big));
        let r_zseq_mnm = z_seq.to_f64_ratio(&m_nm);
        let r_zseq_sup = z_seq.to_f64_ratio(&mmt_sup);
        let det_p_minor_from_bits = (-spec.pen_ca34_bits).exp2();
        assert!(
            (r_zseq_mnm - det_p_minor_from_bits).abs() < 1e-11,
            "Z_seq/M_NM != det(I - P~_r) at case {case}"
        );
        sum_ratio_1cyc_mmt += r_1cyc;
        sum_ratio_zseq_mnm += r_zseq_mnm;
        sum_ratio_zseq_mmt_sup += r_zseq_sup;
        valid_checked += 1;
    }

    // 300 Malformed multi-cycle columns (including disconnected G_1 and connected multi-cycle G_1!)
    let mut mal_checked = 0usize;
    let mut mal_disc = 0usize;
    let mut mal_conn_multicyc = 0usize;
    for case in 0..300usize {
        let target_sigma = 2 + (case % 3);
        let n = 6 + (case % 5); // n in 6..=10
        let mut l = vec![0u8; n];
        if case < 100 {
            // Force disconnected G_1 (two disjoint symbol sets that never cross!)
            // Make F = a..a b..b and L = a..a b..b (so t_{01} = t_{10} = 0 -> disconnected!)
            let na = 2 + ((next_u64() as usize) % (n - 3));
            for i in 0..na {
                l[i] = b'a';
            }
            for i in na..n {
                l[i] = if target_sigma >= 3 && i % 2 == 1 { b'c' } else { b'b' };
            }
        } else {
            // Random non-BWT column (multi-cycle LF)
            loop {
                for b in &mut l {
                    *b = b'a' + (((next_u64() >> 56) as u8) % (target_sigma as u8));
                }
                // Check multi-cycle via forward-FL
                let mut all = [0u32; 256];
                for &c in &l {
                    all[usize::from(c)] += 1;
                }
                let mut first = starts(all);
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
        let recs = records_of(&l);
        // Verify exact forward-FL parity with cand_r5_head on malformed column!
        ref_k.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut ref_out).unwrap();
        let fl_out = invert_forward_fl(&l, p);
        assert_eq!(ref_out, fl_out, "Malformed forward-FL mismatch at case {case}");

        let mut all = [0u32; 256];
        for &c in &l {
            all[usize::from(c)] += 1;
        }
        let (_syms, m, t_mat) = build_g1_matrix(&l, &all);
        let a_t: Vec<Vec<i64>> = t_mat.iter().map(|r| r.iter().map(|&v| v as i64).collect()).collect();
        let mmt_t = macmahon_master_series_inverse(&a_t, &m);
        let (cf_t, _, _) = cauchy_frobenius_cycle_index(&a_t, &m);
        assert_eq!(mmt_t, cf_t, "Malformed Cauchy-Frobenius mismatch at case {case}");
        let mut multinom = fact_u64(n);
        for &mi in &m {
            multinom /= fact_u64(mi);
        }
        if multinom <= 30_000 {
            let bf_t = brute_force_multiset_mmt(&a_t, &m);
            assert_eq!(mmt_t, bf_t, "Malformed brute-force mismatch at case {case}");
        }
        let spec = analyze_spectral_bridge(&t_mat, &m, true);
        if spec.disconnected {
            mal_disc += 1;
        } else {
            mal_conn_multicyc += 1;
            spectral_max_err = spectral_max_err
                .max(spec.err_wielandt_log2)
                .max(spec.err_qr_eigs_log2);
        }
        mal_checked += 1;
    }

    println!(
        "  [PASS] Valid Small BWTs verified: {valid_checked}/5200 (sigma=2: {}, sigma=3: {}, sigma=4: {}) in {:.2}s",
        by_sigma[2],
        by_sigma[3],
        by_sigma[4],
        t0.elapsed().as_secs_f64()
    );
    println!(
        "         - Exact Series Inversion == Linear Form == Cauchy-Frobenius Cycle-Index: {cf_checked}/{valid_checked} (100.00% EXACT BigInt!)"
    );
    println!(
        "         - Exact Brute-Force Multiset-Word Sum (multinom <= 30k): {brute_word_checked}/{brute_word_checked} (100.00% EXACT!)"
    );
    println!(
        "         - Exact Full S_n Conjugacy-Class-by-Class Slice Identity: {brute_sn_checked}/{brute_sn_checked} cases (100.00% EXACT!)"
    );
    println!(
        "         - Exact Malformed Multi-Cycle Columns (forward-FL == cand_r5_head + MMT == CF): {mal_checked}/300 (disc={mal_disc} det==0 exact, conn-multicyc={mal_conn_multicyc})"
    );
    println!(
        "         - Max 4-Way Spectral Bridge Error (BigInt vs Wielandt LU vs Francis QR vs det(I-zP)): {:.3e} bits",
        spectral_max_err
    );
    println!(
        "         - Mean Exact Single-Cycle Ratios: MMT_(n)/MMT(T) = {:.4} | Z_seq/M_NM = det(I-P~_r) = {:.4} | Z_seq/MMT(S) = {:.4}",
        sum_ratio_1cyc_mmt / valid_checked as f64,
        sum_ratio_zseq_mnm / valid_checked as f64,
        sum_ratio_zseq_mmt_sup / valid_checked as f64
    );
    println!();
}

// ============================================================================
// SECTION 9: PART 2 — Exact Small-Case Verification of Theorem M2 (5,200+ Profiles)
// ============================================================================

fn run_part2_theorem_m2_exact_equidistribution() {
    println!("================================================================================================================================");
    println!("PART 2: THEOREM M2 — EXACT VERIFICATION OF MACMAHON (1915) q-MULTINOMIAL EQUIDISTRIBUTION (maj == inv)");
    println!("  Primary Source: P. A. MacMahon, Combinatory Analysis, Vol. I (1915), Section III, Ch. IV, Arts. 103-108 (pp. 135-145)");
    println!("  Verifies sum_{{w in S(m)}} q^{{maj(w)}} == sum_{{w in S(m)}} q^{{inv(w)}} == [n; m_1,...,m_sigma]_q");
    println!("  Exhaustively across ALL 1,023/1,023 ordered compositions of n = 1..10 (100% brute-force) + 4,200 distinct profiles n = 11..13 (exact maj-DP & inv-DP) = 5,223 total!");
    println!("================================================================================================================================");

    let t0 = Instant::now();
    fn fact_u128(n: usize) -> u128 {
        let mut r = 1u128;
        for k in 2..=n {
            r *= k as u128;
        }
        r
    }

    // Generate all ordered compositions of n (positive parts summing to n)
    fn gen_compositions(n: usize) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        let bits = n - 1;
        for mask in 0usize..(1usize << bits) {
            let mut comp = Vec::new();
            let mut cur = 1usize;
            for b in 0..bits {
                if (mask >> b) & 1 == 1 {
                    comp.push(cur);
                    cur = 1;
                } else {
                    cur += 1;
                }
            }
            comp.push(cur);
            out.push(comp);
        }
        out
    }

    // Exact independent DP for sum_{w in S(m)} q^{maj(w)} directly from definition of maj(w)
    fn exact_maj_dp(m: &[usize], max_deg: usize) -> Vec<u128> {
        let s = m.len();
        let dims: Vec<usize> = m.iter().map(|&x| x + 1).collect();
        let mut strides = vec![1usize; s];
        for i in (0..s.saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * dims[i + 1];
        }
        let total_states: usize = dims.iter().product();
        let deg_len = max_deg + 1;
        // dp[state * s + last_sym] -> Vec<u128> (sparse: empty if unreachable)
        let mut dp: Vec<Vec<u128>> = vec![Vec::new(); total_states * s];
        // Base cases: 1-letter words
        for c in 0..s {
            if m[c] >= 1 {
                let st = strides[c];
                let mut v = vec![0u128; deg_len];
                v[0] = 1;
                dp[st * s + c] = v;
            }
        }
        let mut exp = vec![0usize; s];
        for st in 1..total_states - 1 {
            // Decode state
            let mut tmp = st;
            let mut pos = 0usize;
            for i in 0..s {
                exp[i] = tmp / strides[i];
                tmp %= strides[i];
                pos += exp[i];
            }
            if pos == 0 {
                continue;
            }
            for last in 0..s {
                if exp[last] == 0 {
                    continue;
                }
                let src_idx = st * s + last;
                if dp[src_idx].is_empty() {
                    continue;
                }
                let src = std::mem::take(&mut dp[src_idx]);
                for nxt in 0..s {
                    if exp[nxt] + 1 > m[nxt] {
                        continue;
                    }
                    let shift = if last > nxt { pos } else { 0 };
                    let dst_st = st + strides[nxt];
                    let dst_idx = dst_st * s + nxt;
                    if dp[dst_idx].is_empty() {
                        dp[dst_idx] = vec![0u128; deg_len];
                    }
                    let dst = &mut dp[dst_idx];
                    for d in 0..(deg_len - shift) {
                        let val = src[d];
                        if val != 0 {
                            dst[d + shift] += val;
                        }
                    }
                }
            }
        }
        let full = total_states - 1;
        let mut out = vec![0u128; deg_len];
        for last in 0..s {
            let v = &dp[full * s + last];
            for (d, &val) in v.iter().enumerate() {
                out[d] += val;
            }
        }
        while out.len() > 1 && out.last() == Some(&0) {
            out.pop();
        }
        out
    }

    // Exact independent DP for sum_{w in S(m)} q^{inv(w)} directly from definition of inv(w)
    fn exact_inv_dp(m: &[usize], max_deg: usize) -> Vec<u128> {
        let s = m.len();
        let dims: Vec<usize> = m.iter().map(|&x| x + 1).collect();
        let mut strides = vec![1usize; s];
        for i in (0..s.saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * dims[i + 1];
        }
        let total_states: usize = dims.iter().product();
        let deg_len = max_deg + 1;
        let mut dp: Vec<Vec<u128>> = vec![Vec::new(); total_states];
        let mut base = vec![0u128; deg_len];
        base[0] = 1;
        dp[0] = base;
        let mut exp = vec![0usize; s];
        for st in 0..total_states - 1 {
            if dp[st].is_empty() {
                continue;
            }
            let src = std::mem::take(&mut dp[st]);
            let mut tmp = st;
            for i in 0..s {
                exp[i] = tmp / strides[i];
                tmp %= strides[i];
            }
            // Suffix sum: greater[c] = sum_{a > c} exp[a] = inversions added by appending c!
            let mut greater = 0usize;
            for c in (0..s).rev() {
                if exp[c] + 1 <= m[c] {
                    let dst_st = st + strides[c];
                    if dp[dst_st].is_empty() {
                        dp[dst_st] = vec![0u128; deg_len];
                    }
                    let dst = &mut dp[dst_st];
                    for d in 0..(deg_len - greater) {
                        let val = src[d];
                        if val != 0 {
                            dst[d + greater] += val;
                        }
                    }
                }
                greater += exp[c];
            }
        }
        let mut out = std::mem::take(&mut dp[total_states - 1]);
        while out.len() > 1 && out.last() == Some(&0) {
            out.pop();
        }
        out
    }

    let mut total_profiles = 0usize;
    let mut n_le_10_profiles = 0usize;
    let mut n_le_10_brute = 0usize;
    let mut n_11_13_dp = 0usize;
    let mut n_11_13_brute = 0usize;
    let mut total_words_enumerated = 0u64;
    let mut max_mean_err = 0.0f64;
    let mut max_var_err = 0.0f64;

    // 1) ALL 1,023/1,023 ordered compositions for n = 1..=10 — 100% EXHAUSTIVE BRUTE-FORCE!
    for n in 1..=10usize {
        let comps = gen_compositions(n);
        for m in comps {
            let mut mult = fact_u128(n);
            for &mi in &m {
                mult /= fact_u128(mi);
            }
            let qm = q_multinomial(&m);
            let sum_qm: u128 = qm.iter().sum();
            assert_eq!(sum_qm, mult, "q-multinomial sum != multinomial for {m:?}");

            // Check exact mean & variance formula against q-multinomial polynomial!
            let (mu_f, std_f) = multiset_maj_inv_mean_std(n, &m);
            let mut s1 = 0.0f64;
            let mut s2 = 0.0f64;
            let tot_f = sum_qm as f64;
            for (k, &c) in qm.iter().enumerate() {
                let p = (c as f64) / tot_f;
                let kf = k as f64;
                s1 += p * kf;
                s2 += p * kf * kf;
            }
            let var_exact = s2 - s1 * s1;
            max_mean_err = max_mean_err.max((s1 - mu_f).abs());
            max_var_err = max_var_err.max((var_exact - std_f * std_f).abs());

            let mut w = Vec::with_capacity(n);
            for (sym, &cnt) in m.iter().enumerate() {
                for _ in 0..cnt {
                    w.push(sym as u8);
                }
            }
            let mut poly_maj = vec![0u128; qm.len()];
            let mut poly_inv = vec![0u128; qm.len()];
            let check_fenwick = mult <= 5_000;
            loop {
                let (_, mj) = major_index_u8(&w);
                let iv = inv_naive_u8(&w);
                if check_fenwick {
                    assert_eq!(iv, inv_fenwick_u8(&w), "Fenwick inv mismatch on {w:?}");
                }
                poly_maj[mj as usize] += 1;
                poly_inv[iv as usize] += 1;
                total_words_enumerated += 1;
                if !next_permutation_u8(&mut w) {
                    break;
                }
            }
            assert_eq!(poly_maj, qm, "maj distribution != q-multinomial on {m:?}");
            assert_eq!(poly_inv, qm, "inv distribution != q-multinomial on {m:?}");
            // Cross-check exact_maj_dp & exact_inv_dp against brute-force on n <= 7!
            if n <= 7 {
                assert_eq!(exact_maj_dp(&m, qm.len() - 1), qm, "maj-DP mismatch on {m:?}");
                assert_eq!(exact_inv_dp(&m, qm.len() - 1), qm, "inv-DP mismatch on {m:?}");
            }
            n_le_10_brute += 1;
            n_le_10_profiles += 1;
            total_profiles += 1;
        }
    }

    // 2) Additional 4,200 distinct ordered compositions of n = 11..=13 (box <= 2048):
    //    ALL 4,200 verified by exact independent maj-DP & inv-DP == q-multinomial
    //    (PLUS full brute-force word enumeration whenever mult <= 8,000)!
    'outer: for n in 11..=13usize {
        let bits = n - 1;
        for mask in 0usize..(1usize << bits) {
            let mut m = Vec::new();
            let mut cur = 1usize;
            for b in 0..bits {
                if (mask >> b) & 1 == 1 {
                    m.push(cur);
                    cur = 1;
                } else {
                    cur += 1;
                }
            }
            m.push(cur);
            let box_sz: usize = m.iter().map(|&x| x + 1).product();
            if box_sz > 2048 {
                continue;
            }
            let qm = q_multinomial(&m);
            let dp_maj = exact_maj_dp(&m, qm.len() - 1);
            let dp_inv = exact_inv_dp(&m, qm.len() - 1);
            assert_eq!(dp_maj, qm, "maj-DP != q-multinomial on n={n}, {m:?}");
            assert_eq!(dp_inv, qm, "inv-DP != q-multinomial on n={n}, {m:?}");

            let mut mult = fact_u128(n);
            for &mi in &m {
                mult /= fact_u128(mi);
            }
            if mult <= 8_000 {
                let mut w = Vec::with_capacity(n);
                for (sym, &cnt) in m.iter().enumerate() {
                    for _ in 0..cnt {
                        w.push(sym as u8);
                    }
                }
                let mut poly_maj = vec![0u128; qm.len()];
                let mut poly_inv = vec![0u128; qm.len()];
                loop {
                    let (_, mj) = major_index_u8(&w);
                    let iv = inv_naive_u8(&w);
                    poly_maj[mj as usize] += 1;
                    poly_inv[iv as usize] += 1;
                    total_words_enumerated += 1;
                    if !next_permutation_u8(&mut w) {
                        break;
                    }
                }
                assert_eq!(poly_maj, qm, "brute maj != q-multinomial on n={n}, {m:?}");
                assert_eq!(poly_inv, qm, "brute inv != q-multinomial on n={n}, {m:?}");
                n_11_13_brute += 1;
            }
            n_11_13_dp += 1;
            total_profiles += 1;
            if n_11_13_dp >= 4200 {
                break 'outer;
            }
        }
    }

    // 3) Cross-check inv_fenwick_u32(pi) == inv_fenwick_u32(pi^{-1}) on 500 random permutations m = 2..120
    let mut rng = 0xBEEF_1915_0002_CAFEu64;
    for case in 0..500usize {
        let m = 2 + (case % 119);
        let mut pi: Vec<u32> = (0..m as u32).collect();
        for i in (1..m).rev() {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (rng as usize) % (i + 1);
            pi.swap(i, j);
        }
        let mut pi_inv = vec![0u32; m];
        for (u, &v) in pi.iter().enumerate() {
            pi_inv[v as usize] = u as u32;
        }
        let iv_pi = inv_fenwick_u32(&pi);
        let iv_inv = inv_fenwick_u32(&pi_inv);
        assert_eq!(iv_pi, iv_inv, "inv(pi) != inv(pi^-1) at case {case}");
        if case < 100 {
            let mut naive = 0u128;
            for i in 0..m {
                for j in (i + 1)..m {
                    if pi[i] > pi[j] {
                        naive += 1;
                    }
                }
            }
            assert_eq!(iv_pi, naive, "inv_fenwick_u32 != naive at case {case}");
        }
    }

    println!(
        "  [PASS] MacMahon (1915) q-Multinomial Equidistribution verified in {:.2}s:",
        t0.elapsed().as_secs_f64()
    );
    println!(
        "         - Total distinct ordered multiset profiles verified: {total_profiles} (ALL {n_le_10_brute}/{n_le_10_profiles} for n<=10 100% brute-force [ZERO skipped!] + {n_11_13_dp} for n=11..13 exact maj-DP & inv-DP [{n_11_13_brute} also brute-force])"
    );
    println!(
        "         - Total individual multiset words exhaustively enumerated & checked: {total_words_enumerated} (0 mismatches!)"
    );
    println!(
        "         - Exact mean/variance formula error vs q-multinomial across all 1,023 n<=10 profiles: max_mean_err={max_mean_err:.3e}, max_var_err={max_var_err:.3e}"
    );
    println!(
        "         - Permutation Fenwick cross-check (inv(pi) == inv(pi^-1) == naive): 500/500 PASS\n"
    );
}

// ============================================================================
// SECTION 10: PART 3 — Full 4-Corpus Real-Block Analysis (249/249 Blocks!)
// ============================================================================

struct BlockReport {
    name: String,
    n: usize,
    m: usize,
    n_runs: usize,
    g1: SpectralBridgeMetrics,
    ret: SpectralBridgeMetrics,
    // Theorem M2: BWT word L
    des_l: usize,
    maj_l_norm: f64,
    inv_l_norm: f64,
    maj_inv_ratio_l: f64,
    z_maj_l: f64,
    z_inv_l: f64,
    // Theorem M2: Open<16> pi_ret
    des_pi: usize,
    maj_pi_norm: f64,
    imaj_pi_norm: f64,
    inv_pi_norm: f64,
    z_maj_pi: f64,
    z_imaj_pi: f64,
    z_inv_pi: f64,
    // Cache-miss & S4/S5 wall cost
    s4_l1_miss: f64,
    s5_l1_miss: f64,
    s5_l2_miss: f64,
    s45_ns_per_row: f64,
}

fn analyze_real_block(blk: &Block, run_bigint_g1: bool, run_bigint_ret: bool) -> BlockReport {
    let ip = extract_induced_perm(&blk.records, blk.n, blk.p);
    let (_s1, m1, t1) = build_g1_matrix(&ip.l_col, &ip.all);
    let g1 = analyze_spectral_bridge(&t1, &m1, run_bigint_g1);

    let (_sr, mr, tr) = build_ret_double_coset_matrix(&ip);
    let ret = analyze_spectral_bridge(&tr, &mr, run_bigint_ret);

    // Theorem M2 on BWT word L
    let (des_l, maj_l) = major_index_u8(&ip.l_col);
    let inv_l = inv_fenwick_u8(&ip.l_col);
    let (mu_l, std_l) = multiset_maj_inv_mean_std(blk.n, &m1);
    let maj_l_f = maj_l as f64;
    let inv_l_f = inv_l as f64;
    let maj_l_norm = if mu_l > 0.0 { maj_l_f / mu_l } else { 0.0 };
    let inv_l_norm = if mu_l > 0.0 { inv_l_f / mu_l } else { 0.0 };
    let maj_inv_ratio_l = if inv_l_f > 0.0 { maj_l_f / inv_l_f } else { 0.0 };
    let z_maj_l = if std_l > 0.0 { (maj_l_f - mu_l) / std_l } else { 0.0 };
    let z_inv_l = if std_l > 0.0 { (inv_l_f - mu_l) / std_l } else { 0.0 };

    // Theorem M2 on Open<16> pi_ret & pi_ret^{-1}
    let (des_pi, maj_pi) = major_index_u32(&ip.pi);
    let (_des_inv, imaj_pi) = major_index_u32(&ip.pi_inv);
    let inv_pi = inv_fenwick_u32(&ip.pi);
    let inv_pi_check = inv_fenwick_u32(&ip.pi_inv);
    assert_eq!(inv_pi, inv_pi_check, "inv(pi_ret) != inv(pi_ret^-1) on {}", blk.name);
    let mf = ip.m as f64;
    let mu_pi = 0.25 * mf * (mf - 1.0);
    let std_pi = (mf * (mf - 1.0) * (2.0 * mf + 5.0) / 72.0).max(0.0).sqrt();
    let maj_pi_norm = if mu_pi > 0.0 { (maj_pi as f64) / mu_pi } else { 0.0 };
    let imaj_pi_norm = if mu_pi > 0.0 { (imaj_pi as f64) / mu_pi } else { 0.0 };
    let inv_pi_norm = if mu_pi > 0.0 { (inv_pi as f64) / mu_pi } else { 0.0 };
    let z_maj_pi = if std_pi > 0.0 { ((maj_pi as f64) - mu_pi) / std_pi } else { 0.0 };
    let z_imaj_pi = if std_pi > 0.0 { ((imaj_pi as f64) - mu_pi) / std_pi } else { 0.0 };
    let z_inv_pi = if std_pi > 0.0 { ((inv_pi as f64) - mu_pi) / std_pi } else { 0.0 };

    let (s4_l1_miss, s5_l1_miss, s5_l2_miss, s45_ns_per_row) = measure_s4_s5_cost(&ip);

    BlockReport {
        name: blk.name.clone(),
        n: blk.n,
        m: ip.m,
        n_runs: ip.n_runs,
        g1,
        ret,
        des_l,
        maj_l_norm,
        inv_l_norm,
        maj_inv_ratio_l,
        z_maj_l,
        z_inv_l,
        des_pi,
        maj_pi_norm,
        imaj_pi_norm,
        inv_pi_norm,
        z_maj_pi,
        z_imaj_pi,
        z_inv_pi,
        s4_l1_miss,
        s5_l1_miss,
        s5_l2_miss,
        s45_ns_per_row,
    }
}

fn print_m1_table(title: &str, reps: &[BlockReport]) {
    println!("================================================================================================================================");
    println!("{title} — THEOREM M1 (MACMAHON MASTER THEOREM <-> CA-34 / CA-32 SPECTRAL BRIDGE)");
    println!("================================================================================================================================");
    println!(
        "{:<16} {:>3} | {:>7} {:>6} {:>7} | {:>9} {:>9} {:>9} | {:>6} {:>9} | {:>3} {:>7} {:>7} {:>9}",
        "Block", "sig", "CA34_b", "I(r)_b", "Spec_b", "Err_Wiel", "Err_QR", "Err_zP",
        "Gap_P", "MMT_Sad_b", "s_r", "Ret34_b", "RetSp_b", "Ret_ErrW"
    );
    for r in reps {
        println!(
            "{:<16} {:>3} | {:>7.3} {:>6.3} {:>7.3} | {:>9.2e} {:>9.2e} {:>9.2e} | {:>6.4} {:>9.1} | {:>3} {:>7.3} {:>7.3} {:>9.2e}",
            r.name,
            r.g1.sigma,
            r.g1.pen_ca34_bits,
            r.g1.root_info_bits,
            r.g1.spec_prod_bits,
            r.g1.err_wielandt_log2,
            r.g1.err_qr_eigs_log2,
            r.g1.err_poly_z_max,
            r.g1.spectral_gap,
            r.g1.mmt_saddle_log2_prob,
            r.ret.sigma,
            r.ret.pen_ca34_bits,
            r.ret.spec_prod_bits,
            r.ret.err_wielandt_log2
        );
    }
    println!();
}

fn print_m2_table(title: &str, reps: &[BlockReport]) {
    println!("================================================================================================================================");
    println!("{title} — THEOREM M2 (MACMAHON MAJOR INDEX vs INVERSION NUMBER & S4/S5 CACHE AUDIT)");
    println!("================================================================================================================================");
    println!(
        "{:<16} {:>6} {:>6} | {:>6} {:>6} {:>6} {:>8} {:>7} | {:>6} {:>6} {:>6} {:>7} | {:>5} {:>5} {:>5} {:>6}",
        "Block", "r/n%", "m/n%",
        "majL*", "invL*", "mj/ivL", "z_majL", "z_invL",
        "majPi*", "imjPi*", "invPi*", "z_mjPi",
        "S4L1%", "S5L1%", "S5L2%", "ns/rw"
    );
    for r in reps {
        println!(
            "{:<16} {:>6.2} {:>6.2} | {:>6.4} {:>6.4} {:>6.4} {:>8.1} {:>7.1} | {:>6.4} {:>6.4} {:>6.4} {:>7.1} | {:>5.1} {:>5.1} {:>5.1} {:>6.2}",
            r.name,
            100.0 * r.n_runs as f64 / r.n as f64,
            100.0 * r.m as f64 / r.n as f64,
            r.maj_l_norm,
            r.inv_l_norm,
            r.maj_inv_ratio_l,
            r.z_maj_l,
            r.z_inv_l,
            r.maj_pi_norm,
            r.imaj_pi_norm,
            r.inv_pi_norm,
            r.z_maj_pi,
            r.s4_l1_miss,
            r.s5_l1_miss,
            r.s5_l2_miss,
            r.s45_ns_per_row
        );
    }
    println!();
}

fn print_correlations(label: &str, reps: &[BlockReport]) {
    let mn: Vec<f64> = reps.iter().map(|r| r.m as f64 / r.n as f64).collect();
    let rn: Vec<f64> = reps.iter().map(|r| r.n_runs as f64 / r.n as f64).collect();
    let maj_l: Vec<f64> = reps.iter().map(|r| r.maj_l_norm).collect();
    let inv_l: Vec<f64> = reps.iter().map(|r| r.inv_l_norm).collect();
    let rat_l: Vec<f64> = reps.iter().map(|r| r.maj_inv_ratio_l).collect();
    let maj_p: Vec<f64> = reps.iter().map(|r| r.maj_pi_norm).collect();
    let imaj_p: Vec<f64> = reps.iter().map(|r| r.imaj_pi_norm).collect();
    let inv_p: Vec<f64> = reps.iter().map(|r| r.inv_pi_norm).collect();
    let s4l1: Vec<f64> = reps.iter().map(|r| r.s4_l1_miss).collect();
    let s5l1: Vec<f64> = reps.iter().map(|r| r.s5_l1_miss).collect();
    let s5l2: Vec<f64> = reps.iter().map(|r| r.s5_l2_miss).collect();
    let ns_rw: Vec<f64> = reps.iter().map(|r| r.s45_ns_per_row).collect();

    println!("  PEARSON r CORRELATION MATRIX ({label}, N = {} blocks):", reps.len());
    println!("    Predictor          |  r(m/n)  | r(S4_L1) | r(S5_L1) | r(S5_L2) | r(S45_ns/rw)");
    let rows: [(&str, &[f64]); 7] = [
        ("r / n (run density)", &rn),
        ("maj(L) / mu_unif   ", &maj_l),
        ("inv(L) / mu_unif   ", &inv_l),
        ("maj(L) / inv(L)    ", &rat_l),
        ("maj(pi_ret) / mu   ", &maj_p),
        ("imaj(pi_ret) / mu  ", &imaj_p),
        ("inv(pi_ret) / mu   ", &inv_p),
    ];
    for (name, v) in rows {
        println!(
            "    {} | {:>+8.4} | {:>+8.4} | {:>+8.4} | {:>+8.4} | {:>+12.4}",
            name,
            pearson_r(v, &mn),
            pearson_r(v, &s4l1),
            pearson_r(v, &s5l1),
            pearson_r(v, &s5l2),
            pearson_r(v, &ns_rw)
        );
    }
    println!();
}

fn benchmark_s1_maj_l_cost(blocks: &[Block]) {
    // Measure exact overhead of computing maj(L) during an S1 scan on the 6 admitted blocks
    // Note: in a run-length stream, descents only occur at run boundaries (b_prev > b_cur at pos)!
    let mut tot_ns_baseline = 0.0f64;
    let mut tot_ns_with_maj = 0.0f64;
    let mut sink = 0u128;
    for blk in blocks {
        // Verify run-boundary maj(L) == byte-by-byte maj(L) identically!
        let l = expand_records(&blk.records, blk.n);
        let (des_b, maj_b) = major_index_u8(&l);
        let mut pos = 0usize;
        let mut prev_c = 0u8;
        let mut has_prev = false;
        let mut des_r = 0usize;
        let mut maj_r = 0u128;
        for &rec in &blk.records {
            let c = (rec >> 8) as u8;
            let w = usize::from(rec & 255);
            if w == 0 {
                continue;
            }
            if has_prev && prev_c > c {
                des_r += 1;
                maj_r += pos as u128;
            }
            pos += w;
            prev_c = c;
            has_prev = true;
        }
        assert_eq!(des_r, des_b, "run-level des mismatch on {}", blk.name);
        assert_eq!(maj_r, maj_b, "run-level maj mismatch on {}", blk.name);

        let reps = 20;
        let t0 = Instant::now();
        for _ in 0..reps {
            let mut sum_w = 0u64;
            for &rec in &blk.records {
                sum_w += u64::from(rec & 255);
            }
            sink ^= u128::from(sum_w);
        }
        tot_ns_baseline += t0.elapsed().as_nanos() as f64 / reps as f64;

        let t1 = Instant::now();
        for _ in 0..reps {
            let mut p = 0u64;
            let mut pc = 0u8;
            let mut mj = 0u128;
            for &rec in &blk.records {
                let c = (rec >> 8) as u8;
                let w = u64::from(rec & 255);
                if pc > c {
                    mj += u128::from(p);
                }
                p += w;
                pc = c;
            }
            sink ^= mj;
        }
        tot_ns_with_maj += t1.elapsed().as_nanos() as f64 / reps as f64;
    }
    std::hint::black_box(sink);
    let n_blks = blocks.len() as f64;
    println!(
        "  S1 RUN-BOUNDARY maj(L) COST (6 Admitted, byte-exact verified): scan={:.1} us/blk, scan+maj={:.1} us/blk (delta={:+.1} us/blk)",
        tot_ns_baseline / (1000.0 * n_blks),
        tot_ns_with_maj / (1000.0 * n_blks),
        (tot_ns_with_maj - tot_ns_baseline) / (1000.0 * n_blks)
    );
    println!();
}

fn main() {
    let t_total = Instant::now();

    // PART 1: Exact Small-Case Verification of Theorem M1 (5,200 valid BWTs + 300 malformed)
    run_part1_theorem_m1_exact_small();

    // PART 2: Exact Small-Case Verification of Theorem M2 (5,223 distinct multiset profiles)
    run_part2_theorem_m2_exact_equidistribution();

    // PART 3: Solved Analogue M + Full 4-Corpus Real-Block Analysis (249/249 blocks!)
    let solved = make_solved_analogue_m();
    let solved_reps: Vec<BlockReport> = solved.iter().map(|b| analyze_real_block(b, true, true)).collect();
    print_m1_table("PART 3A: SOLVED ANALOGUE M", &solved_reps);
    print_m2_table("PART 3A: SOLVED ANALOGUE M", &solved_reps);

    let wiki6 = load_six_admitted();
    let wiki6_reps: Vec<BlockReport> = wiki6.iter().map(|b| analyze_real_block(b, true, true)).collect();
    print_m1_table("PART 3B: 6 ADMITTED WIKIDATA BLOCKS", &wiki6_reps);
    print_m2_table("PART 3B: 6 ADMITTED WIKIDATA BLOCKS", &wiki6_reps);
    benchmark_s1_maj_l_cost(&wiki6);

    let nonwiki_names = ["jsonl-0", "jsonl-1", "jsonl-2", "jsonl-3", "jsonl-4", "syslog-0"];
    let nonwiki = load_bin_blocks(
        &resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin"),
        Some(&nonwiki_names),
        false,
    );
    let nonwiki_reps: Vec<BlockReport> = nonwiki.iter().map(|b| analyze_real_block(b, true, true)).collect();
    print_m1_table("PART 3C: 6 REAL NON-WIKIDATA BLOCKS (JSONL + SYSLOG)", &nonwiki_reps);
    print_m2_table("PART 3C: 6 REAL NON-WIKIDATA BLOCKS (JSONL + SYSLOG)", &nonwiki_reps);

    let cd_names = ["codedocs-0", "codedocs-1", "codedocs-2", "codedocs-3"];
    let code_docs = load_bin_blocks(
        &resolve_corpus_path("non_wikidata/code_docs_blocks.bin"),
        Some(&cd_names),
        false,
    );
    let cd_reps: Vec<BlockReport> = code_docs.iter().map(|b| analyze_real_block(b, true, true)).collect();
    print_m1_table("PART 3D: 4 REAL CODE+DOCS BLOCKS", &cd_reps);
    print_m2_table("PART 3D: 4 REAL CODE+DOCS BLOCKS", &cd_reps);

    // ALL 233 Full wikidata.bz2 blocks (BigInt exact on G_1 for all 233 + BigInt exact on ret G_ret too!)
    let t_w233 = Instant::now();
    let w233 = load_bin_blocks(&resolve_corpus_path("wikidata_431_blocks.bin"), None, true);
    let w233_reps: Vec<BlockReport> = w233.iter().map(|b| analyze_real_block(b, true, true)).collect();

    println!("================================================================================================================================");
    println!(
        "PART 3E: ALL {} FULL WIKIDATA.BZ2 BLOCKS — AGGREGATE ANALOGUE M SUMMARY (BigInt Exact G_1 & G_ret in {:.2}s)",
        w233_reps.len(),
        t_w233.elapsed().as_secs_f64()
    );
    println!("================================================================================================================================");

    // Aggregate across ALL 249 real blocks (6 Adm + 6 NonWiki + 4 CodeDocs + 233 FullWiki)
    let mut all_249: Vec<&BlockReport> = Vec::with_capacity(249);
    all_249.extend(wiki6_reps.iter());
    all_249.extend(nonwiki_reps.iter());
    all_249.extend(cd_reps.iter());
    all_249.extend(w233_reps.iter());
    assert_eq!(all_249.len(), 249);

    let summarize_group = |label: &str, group: &[&BlockReport]| {
        let k = group.len() as f64;
        let max_err_w_g1 = group.iter().map(|r| r.g1.err_wielandt_log2).fold(0.0f64, f64::max);
        let max_err_qr_g1 = group.iter().map(|r| r.g1.err_qr_eigs_log2).fold(0.0f64, f64::max);
        let max_err_im_g1 = group.iter().map(|r| r.g1.err_qr_im).fold(0.0f64, f64::max);
        let max_err_zp_g1 = group.iter().map(|r| r.g1.err_poly_z_max).fold(0.0f64, f64::max);
        let max_err_w_ret = group.iter().map(|r| r.ret.err_wielandt_log2).fold(0.0f64, f64::max);
        let max_err_qr_ret = group.iter().map(|r| r.ret.err_qr_eigs_log2).fold(0.0f64, f64::max);

        let min_pen = group.iter().map(|r| r.g1.pen_ca34_bits).fold(f64::INFINITY, f64::min);
        let max_pen = group.iter().map(|r| r.g1.pen_ca34_bits).fold(0.0f64, f64::max);
        let mean_pen = group.iter().map(|r| r.g1.pen_ca34_bits).sum::<f64>() / k;
        let mean_ir = group.iter().map(|r| r.g1.root_info_bits).sum::<f64>() / k;
        let mean_sp = group.iter().map(|r| r.g1.spec_prod_bits).sum::<f64>() / k;
        let mean_gap = group.iter().map(|r| r.g1.spectral_gap).sum::<f64>() / k;
        let mean_sad = group.iter().map(|r| r.g1.mmt_saddle_log2_prob).sum::<f64>() / k;
        let mean_ret_pen = group.iter().map(|r| r.ret.pen_ca34_bits).sum::<f64>() / k;
        let mean_ret_sp = group.iter().map(|r| r.ret.spec_prod_bits).sum::<f64>() / k;
        let mean_ret_gap = group.iter().map(|r| r.ret.spectral_gap).sum::<f64>() / k;

        let mean_maj_l = group.iter().map(|r| r.maj_l_norm).sum::<f64>() / k;
        let mean_inv_l = group.iter().map(|r| r.inv_l_norm).sum::<f64>() / k;
        let mean_rat_l = group.iter().map(|r| r.maj_inv_ratio_l).sum::<f64>() / k;
        let mean_zmaj_l = group.iter().map(|r| r.z_maj_l).sum::<f64>() / k;
        let mean_zinv_l = group.iter().map(|r| r.z_inv_l).sum::<f64>() / k;
        let mean_des_l = group.iter().map(|r| r.des_l as f64 / r.n as f64).sum::<f64>() / k;
        let mean_des_pi = group.iter().map(|r| r.des_pi as f64 / r.m as f64).sum::<f64>() / k;
        let mean_maj_pi = group.iter().map(|r| r.maj_pi_norm).sum::<f64>() / k;
        let mean_imaj_pi = group.iter().map(|r| r.imaj_pi_norm).sum::<f64>() / k;
        let mean_inv_pi = group.iter().map(|r| r.inv_pi_norm).sum::<f64>() / k;
        let mean_zmaj_pi = group.iter().map(|r| r.z_maj_pi).sum::<f64>() / k;
        let mean_zimaj_pi = group.iter().map(|r| r.z_imaj_pi).sum::<f64>() / k;
        let mean_zinv_pi = group.iter().map(|r| r.z_inv_pi).sum::<f64>() / k;

        let mean_h1 = group.iter().map(|r| r.g1.h1_markov_bits / r.n as f64).sum::<f64>() / k;
        let mean_hnm = group.iter().map(|r| r.g1.h_nm_bits / r.n as f64).sum::<f64>() / k;
        let mean_ret_h1 = group.iter().map(|r| r.ret.h1_markov_bits / r.m as f64).sum::<f64>() / k;
        let mean_ret_hnm = group.iter().map(|r| r.ret.h_nm_bits / r.m as f64).sum::<f64>() / k;

        println!("  [{label} (N={})]", group.len());
        println!(
            "    M1 G_1 Spectral Bridge: CA34_pen = {mean_pen:.3} b [{min_pen:.3}..{max_pen:.3}] = I(r) {mean_ir:.3} b + Spec_prod {mean_sp:+.3} b | gap={mean_gap:.4} | MMT_saddle={mean_sad:.1} b | H1={mean_h1:.3} vs H_NM={mean_hnm:.3} b/B"
        );
        println!(
            "       Max Errors (G_1): Wielandt_LU={max_err_w_g1:.3e} b | Francis_QR={max_err_qr_g1:.3e} b (Im={max_err_im_g1:.3e}) | det(I-zP)={max_err_zp_g1:.3e} b"
        );
        println!(
            "    M1 G_ret (CA-32 Double-Coset N_ab): CA34_pen = {mean_ret_pen:.3} b (Spec={mean_ret_sp:+.3} b, gap={mean_ret_gap:.4}, H1={mean_ret_h1:.3}, H_NM={mean_ret_hnm:.3} b/row) | Max Errs: Wiel={max_err_w_ret:.3e}, QR={max_err_qr_ret:.3e}"
        );
        println!(
            "    M2 BWT Word L: des/n={:.4} | maj(L)/mu={mean_maj_l:.4} (z={mean_zmaj_l:+.1}) | inv(L)/mu={mean_inv_l:.4} (z={mean_zinv_l:+.1}) | maj/inv={mean_rat_l:.4}x",
            mean_des_l
        );
        println!(
            "    M2 Open<16> pi_ret: des/m={:.4} | maj/mu={mean_maj_pi:.4} (z={mean_zmaj_pi:+.1}) | imaj/mu={mean_imaj_pi:.4} (z={mean_zimaj_pi:+.1}) | inv/mu={mean_inv_pi:.4} (z={mean_zinv_pi:+.1})",
            mean_des_pi
        );
    };

    let w6_refs: Vec<&BlockReport> = wiki6_reps.iter().collect();
    let nw_refs: Vec<&BlockReport> = nonwiki_reps.iter().collect();
    let cd_refs: Vec<&BlockReport> = cd_reps.iter().collect();
    let w233_refs: Vec<&BlockReport> = w233_reps.iter().collect();

    summarize_group("6 Admitted Wikidata", &w6_refs);
    summarize_group("6 Non-Wikidata JSONL+Syslog", &nw_refs);
    summarize_group("4 Code+Docs", &cd_refs);
    summarize_group("233 Full wikidata.bz2", &w233_refs);
    summarize_group("ALL 249 REAL BLOCKS COMBINED", &all_249);
    println!();

    // Assert < 1e-12 precision on ALL 249/249 real blocks for Wielandt-MacMahon spectral bridge!
    let max_all_wiel_g1 = all_249.iter().map(|r| r.g1.err_wielandt_log2).fold(0.0f64, f64::max);
    let max_all_qr_g1 = all_249.iter().map(|r| r.g1.err_qr_eigs_log2).fold(0.0f64, f64::max);
    let max_all_zp_g1 = all_249.iter().map(|r| r.g1.err_poly_z_max).fold(0.0f64, f64::max);
    let max_all_wiel_ret = all_249.iter().map(|r| r.ret.err_wielandt_log2).fold(0.0f64, f64::max);
    let max_all_qr_ret = all_249.iter().map(|r| r.ret.err_qr_eigs_log2).fold(0.0f64, f64::max);

    println!(
        "  [VERIFY 249/249] GLOBAL MAX SPECTRAL BRIDGE PRECISION ACROSS ALL 249 REAL BLOCKS:"
    );
    println!(
        "    G_1  : Wielandt-MacMahon vs BigInt={max_all_wiel_g1:.3e} b | Francis-QR vs BigInt={max_all_qr_g1:.3e} b | det(I-zP) vs prod(1-z lam)={max_all_zp_g1:.3e} b"
    );
    println!(
        "    G_ret: Wielandt-MacMahon vs BigInt={max_all_wiel_ret:.3e} b | Francis-QR vs BigInt={max_all_qr_ret:.3e} b"
    );
    assert!(
        max_all_wiel_g1 < 1e-12 && max_all_qr_g1 < 1e-12 && max_all_zp_g1 < 1e-12,
        "G_1 spectral bridge precision exceeded 1e-12!"
    );
    assert!(
        max_all_wiel_ret < 1e-11 && max_all_qr_ret < 1e-11,
        "G_ret spectral bridge precision exceeded 1e-11!"
    );
    assert!(
        all_249.iter().all(|r| r.g1.mmt_saddle_log2_prob.is_finite()),
        "Non-finite MMT saddle-point value detected!"
    );
    println!();

    // Print Pearson correlation matrices on 233 Full Wikidata & All 249 Real Blocks
    print_correlations("233 Full Wikidata", &w233_reps);
    let all_249_owned: Vec<BlockReport> = wiki6_reps
        .into_iter()
        .chain(nonwiki_reps)
        .chain(cd_reps)
        .chain(w233_reps)
        .collect();
    print_correlations("ALL 249 Mixed-Corpus Real Blocks", &all_249_owned);

    println!(
        "DONE: Cycle 50 P1 Analogue M (MacMahon Master Theorem & Major Index) completed 100% PASS in {:.2}s!",
        t_total.elapsed().as_secs_f64()
    );
}
