#![forbid(unsafe_code)]

//! Cycle 49 P2 — Analogue K, Theorem K1:
//! Sylvester (1851, 1857), Cauchy–Binet (1812), Kirchhoff (1847), & Cayley (1889)
//! Directed Matrix-Tree / Arborescence Theorem & Exact BEST Factorization Across
//! Byte-Digram G_1, Open<16> Band-Transition Graphs, and the FQ2 Iterated-Cut Tower.
//!
//! Primary Sources:
//!   - A.-L. Cauchy (1812) & J. Binet (1812): Cauchy-Binet determinant formula
//!   - G. R. Kirchhoff (1847), Ann. Phys. Chem. 72, 497-508: Graph Laplacian & Matrix-Tree
//!   - J. J. Sylvester (1851, Phil. Mag.; 1857, Q. J. Pure Appl. Math. 1, 42-56):
//!     Sylvester's determinant identity (exact fraction-free Bareiss elimination)
//!   - A. Cayley (1889, Q. J. Pure Appl. Math. 23, 376-378): Spanning tree enumeration
//!   - C. A. B. Smith & W. T. Tutte (1941) / van Aardenne-Ehrenfest & de Bruijn (1951): BEST
//!   - E. Netto (1901) / P. A. MacMahon (1915): Local multiset-permutation factor (CA-29)
//!   - H. J. S. Smith (1861) & G. Frobenius / L. Stickelberger (1879): |Pic^0(G_k)| = tau_r(G_k)
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`

use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::time::Instant;

// ============================================================================
// 1. Exact Signed Radix-2^32 BigInt (Overflow-Free, Knuth Algorithm D Division)
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
    fn is_one(&self) -> bool {
        !self.neg && self.limbs.len() == 1 && self.limbs[0] == 1
    }
    fn to_u128(&self) -> Option<u128> {
        if self.neg || self.limbs.len() > 4 {
            return None;
        }
        let mut v = 0u128;
        for (i, &w) in self.limbs.iter().enumerate() {
            v |= (w as u128) << (32 * i);
        }
        Some(v)
    }
    fn norm(&mut self) {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
        if self.limbs.is_empty() {
            self.neg = false;
        }
    }
    fn abs(&self) -> Self {
        BigInt { neg: false, limbs: self.limbs.clone() }
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
                    let mut r = BigInt { neg: self.neg, limbs: Self::sub_mag(&self.limbs, &rhs.limbs) };
                    r.norm();
                    r
                }
                Ordering::Less => {
                    let mut r = BigInt { neg: rhs.neg, limbs: Self::sub_mag(&rhs.limbs, &self.limbs) };
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
    /// Knuth Algorithm D division in radix 2^32. Returns (quotient, remainder) with
    /// truncated-toward-zero semantics: self == q * rhs + r, |r| < |rhs|.
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

        // Normalize so top limb of divisor has MSB set
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

            // Multiply and subtract qhat * vn from un[j..=j+n]
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
                // Rare add-back step (Knuth D6)
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

        // Unnormalize remainder
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
            // Take top 3 limbs (~65..96 bits) for full f64 53-bit mantissa accuracy
            let top = ((self.limbs[n - 1] as f64) * 4294967296.0 + (self.limbs[n - 2] as f64))
                * 4294967296.0
                + (self.limbs[n - 3] as f64);
            top.log2() + 32.0 * ((n - 3) as f64)
        }
    }
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
    // Exact sum for n <= 1024, Stirling + exact series otherwise
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

// ============================================================================
// 2. Sylvester (1851, 1857) Fraction-Free Bareiss Determinant, SNF Cross-Check,
//    Pivoted f64 LU, & Exact Sylvester-Schur Degree-1 Contraction
// ============================================================================

/// Exact integer determinant via Sylvester's (1851, 1857) determinant identity
/// (fraction-free Bareiss elimination). Asserts zero remainder at every division!
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

/// Pivoted f64 LU log2(|det|) for fast evaluation and exact cross-check against BigInt Bareiss.
fn log2_det_lu_f64(mut a: Vec<Vec<f64>>) -> f64 {
    let n = a.len();
    if n == 0 {
        return 0.0;
    }
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
            return f64::NEG_INFINITY;
        }
        if piv_row != k {
            a.swap(k, piv_row);
        }
        let diag = a[k][k];
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
    log2_abs
}

/// Exact BigInt extended GCD & Smith Normal Form (CA-17/CA-18 cross-check: |Pic^0(G)| == tau_r(G)).
fn big_ext_gcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    let (mut old_r, mut r) = (a.clone(), b.clone());
    let (mut old_s, mut s) = (BigInt::one(), BigInt::zero());
    let (mut old_t, mut t) = (BigInt::zero(), BigInt::one());
    while !r.is_zero() {
        let (q, rem) = old_r.div_rem(&r);
        let ns = old_s.sub(&q.mul(&s));
        let nt = old_t.sub(&q.mul(&t));
        old_r = r;
        r = rem;
        old_s = s;
        s = ns;
        old_t = t;
        t = nt;
    }
    if old_r.neg {
        (old_r.neg_val(), old_s.neg_val(), old_t.neg_val())
    } else {
        (old_r, old_s, old_t)
    }
}

fn snf_torsion_order_bigint(raw: &[Vec<i64>]) -> (usize, BigInt) {
    let d = raw.len();
    let mut a: Vec<Vec<BigInt>> = raw
        .iter()
        .map(|row| row.iter().map(|&v| BigInt::from_i64(v)).collect())
        .collect();
    let mut k = 0usize;
    while k < d {
        let mut best: Option<(usize, usize)> = None;
        for i in k..d {
            for j in k..d {
                if !a[i][j].is_zero() {
                    let better = match best {
                        None => true,
                        Some((bi, bj)) => {
                            BigInt::cmp_mag(&a[i][j].limbs, &a[bi][bj].limbs) == Ordering::Less
                        }
                    };
                    if better {
                        best = Some((i, j));
                    }
                }
            }
        }
        let Some((pi, pj)) = best else { break };
        a.swap(k, pi);
        for row in &mut a {
            row.swap(k, pj);
        }
        if a[k][k].neg {
            for j in k..d {
                a[k][j] = a[k][j].neg_val();
            }
        }
        loop {
            let mut changed = true;
            while changed {
                changed = false;
                for i in (k + 1)..d {
                    if !a[i][k].is_zero() {
                        changed = true;
                        let p = a[k][k].clone();
                        let x = a[i][k].clone();
                        let (q, rem) = x.div_rem(&p);
                        if rem.is_zero() {
                            for j in k..d {
                                let sub = q.mul(&a[k][j]);
                                a[i][j] = a[i][j].sub(&sub);
                            }
                        } else {
                            let (g, s, t) = big_ext_gcd(&p, &x);
                            let (u_pos, _) = x.div_rem(&g);
                            let u = u_pos.neg_val();
                            let (v, _) = p.div_rem(&g);
                            for j in k..d {
                                let rk = a[k][j].clone();
                                let ri = a[i][j].clone();
                                a[k][j] = s.mul(&rk).add(&t.mul(&ri));
                                a[i][j] = u.mul(&rk).add(&v.mul(&ri));
                            }
                        }
                    }
                }
                for j in (k + 1)..d {
                    if !a[k][j].is_zero() {
                        changed = true;
                        let p = a[k][k].clone();
                        let x = a[k][j].clone();
                        let (q, rem) = x.div_rem(&p);
                        if rem.is_zero() {
                            for i in k..d {
                                let sub = q.mul(&a[i][k]);
                                a[i][j] = a[i][j].sub(&sub);
                            }
                        } else {
                            let (g, s, t) = big_ext_gcd(&p, &x);
                            let (u_pos, _) = x.div_rem(&g);
                            let u = u_pos.neg_val();
                            let (v, _) = p.div_rem(&g);
                            for i in k..d {
                                let ck = a[i][k].clone();
                                let cj = a[i][j].clone();
                                a[i][k] = s.mul(&ck).add(&t.mul(&cj));
                                a[i][j] = u.mul(&ck).add(&v.mul(&cj));
                            }
                        }
                    }
                }
            }
            if a[k][k].neg {
                a[k][k] = a[k][k].neg_val();
            }
            let p = a[k][k].clone();
            let mut bad_row = None;
            'chk: for i in (k + 1)..d {
                for j in (k + 1)..d {
                    let (_, rem) = a[i][j].div_rem(&p);
                    if !rem.is_zero() {
                        bad_row = Some(i);
                        break 'chk;
                    }
                }
            }
            if let Some(bi) = bad_row {
                for j in k..d {
                    let add = a[bi][j].clone();
                    a[k][j] = a[k][j].add(&add);
                }
            } else {
                break;
            }
        }
        k += 1;
    }
    let mut prod = BigInt::one();
    let mut rank = 0usize;
    for i in 0..k {
        let v = a[i][i].abs();
        if !v.is_zero() {
            rank += 1;
            prod = prod.mul(&v);
        }
    }
    (rank, prod)
}

// ============================================================================
// 3. Eulerian Directed Multigraph & Exact Sylvester-Schur Reduction
// ============================================================================

#[derive(Clone, Debug)]
struct EulerDigraph {
    n_verts: usize,
    /// Non-loop outgoing edges: out_adj[u] = sorted/deduped Vec<(v, weight)> with v != u, weight > 0
    out_adj: Vec<Vec<(usize, u64)>>,
    /// Self-loop counts a_{uu}
    loops: Vec<u64>,
    /// Total out-degree d^+(u) = loops[u] + sum_{v != u} a_{uv}
    out_deg: Vec<u64>,
}

impl EulerDigraph {
    fn from_edge_list(n_verts: usize, edges: &[(usize, usize, u64)]) -> Self {
        let mut out_adj = vec![Vec::<(usize, u64)>::new(); n_verts];
        let mut loops = vec![0u64; n_verts];
        let mut out_deg = vec![0u64; n_verts];
        let mut in_deg = vec![0u64; n_verts];
        for &(u, v, w) in edges {
            if w == 0 {
                continue;
            }
            out_deg[u] += w;
            in_deg[v] += w;
            if u == v {
                loops[u] += w;
            } else {
                out_adj[u].push((v, w));
            }
        }
        for u in 0..n_verts {
            assert_eq!(out_deg[u], in_deg[u], "Graph is not Euler-balanced at v={u}");
            if out_adj[u].len() > 1 {
                out_adj[u].sort_unstable_by_key(|&(v, _)| v);
                let mut write = 0usize;
                for read in 0..out_adj[u].len() {
                    let (v, w) = out_adj[u][read];
                    if write > 0 && out_adj[u][write - 1].0 == v {
                        out_adj[u][write - 1].1 += w;
                    } else {
                        out_adj[u][write] = (v, w);
                        write += 1;
                    }
                }
                out_adj[u].truncate(write);
            }
        }
        EulerDigraph { n_verts, out_adj, loops, out_deg }
    }

    /// Build exact unweighted support digraph (every non-loop edge weight -> 1, loops -> 0).
    /// Note: support digraph is generally NOT Eulerian, so cofactor depends slightly on root r;
    /// we root at max-degree vertex r_max for canonical comparison.
    fn support_minor_matrix_i64(&self, root: usize) -> Vec<Vec<i64>> {
        let n = self.n_verts;
        if n <= 1 {
            return Vec::new();
        }
        let mut idx_map = vec![usize::MAX; n];
        let mut verts = Vec::with_capacity(n - 1);
        for u in 0..n {
            if u != root {
                verts.push(u);
            }
        }
        verts.sort_unstable_by_key(|&u| self.out_adj[u].len());
        for (pos, &u) in verts.iter().enumerate() {
            idx_map[u] = pos;
        }
        let m = n - 1;
        let mut mat = vec![vec![0i64; m]; m];
        for (i, &u) in verts.iter().enumerate() {
            let mut diag = 0i64;
            for &(v, _) in &self.out_adj[u] {
                diag += 1;
                if v != root {
                    mat[i][idx_map[v]] -= 1;
                }
            }
            mat[i][i] = diag;
        }
        mat
    }

    /// Build full reduced directed Laplacian minor \tilde{L}_root(G) ordered by increasing degree.
    fn minor_matrix_i64(&self, root: usize) -> Vec<Vec<i64>> {
        let n = self.n_verts;
        if n <= 1 {
            return Vec::new();
        }
        let mut verts: Vec<usize> = (0..n).filter(|&u| u != root).collect();
        verts.sort_unstable_by_key(|&u| (self.out_adj[u].len(), self.out_deg[u]));
        let mut idx_map = vec![usize::MAX; n];
        for (pos, &u) in verts.iter().enumerate() {
            idx_map[u] = pos;
        }
        let m = n - 1;
        let mut mat = vec![vec![0i64; m]; m];
        for (i, &u) in verts.iter().enumerate() {
            let mut diag = 0i64;
            for &(v, w) in &self.out_adj[u] {
                diag += w as i64;
                if v != root {
                    mat[i][idx_map[v]] -= w as i64;
                }
            }
            mat[i][i] = diag;
        }
        mat
    }

    /// Exact local entropy terms:
    ///   - local_fact_bits = sum_v log2((d^+(v) - 1)!)
    ///   - netto_macmahon_bits = sum_v log2( d^+(v)! / prod_u a_{vu}! )
    ///   - sum_log2_deg_nonroot = sum_{v != root} log2(d^+(v))
    ///   - c47_relabel_bits = sum_v log2(out_distinct_total(v)!)  (Cycle 47 quarantined formula!)
    fn local_entropies(&self, root: usize) -> (f64, f64, f64, f64) {
        let mut local_fact_bits = 0.0f64;
        let mut nm_bits = 0.0f64;
        let mut sum_log2_d_nr = 0.0f64;
        let mut c47_relabel_bits = 0.0f64;
        for u in 0..self.n_verts {
            let d = self.out_deg[u] as usize;
            if d == 0 {
                continue;
            }
            local_fact_bits += log2_fact(d - 1);
            let mut denom = log2_fact(self.loops[u] as usize);
            for &(_, w) in &self.out_adj[u] {
                denom += log2_fact(w as usize);
            }
            nm_bits += log2_fact(d) - denom;
            if u != root {
                sum_log2_d_nr += (d as f64).log2();
            }
            let distinct_out = self.out_adj[u].len() + usize::from(self.loops[u] > 0);
            if distinct_out > 1 {
                c47_relabel_bits += log2_fact(distinct_out);
            }
        }
        (local_fact_bits, nm_bits, sum_log2_d_nr, c47_relabel_bits)
    }

    /// Exact Sylvester-Schur degree-1 contraction of an Eulerian directed multigraph.
    /// Repeatedly eliminates any non-root vertex v with unique non-loop out-neighbor OR
    /// unique non-loop in-neighbor, multiplying the exact determinant scale by its weight.
    /// Returns (disconnected, core_dim, Option<core_minor_i64>, scale_log2, optional_scale_bigint, contracted_count).
    /// If `core_dim > max_core`, skips allocating the dense matrix and returns `None`.
    fn schur_reduce_for_det(
        &self,
        root: usize,
        track_bigint: bool,
        max_core: usize,
    ) -> (bool, usize, Option<Vec<Vec<i64>>>, f64, Option<BigInt>, usize) {
        let n = self.n_verts;
        if n <= 1 {
            return (false, 0, Some(Vec::new()), 0.0, track_bigint.then(BigInt::one), 0);
        }
        let mut out_map: Vec<HashMap<usize, u64>> = vec![HashMap::new(); n];
        let mut in_map: Vec<HashMap<usize, u64>> = vec![HashMap::new(); n];
        for u in 0..n {
            for &(v, w) in &self.out_adj[u] {
                *out_map[u].entry(v).or_insert(0) += w;
                *in_map[v].entry(u).or_insert(0) += w;
            }
        }

        let mut active = vec![true; n];
        let mut in_queue = vec![false; n];
        let mut q = VecDeque::new();
        for u in 0..n {
            if u != root && (out_map[u].len() <= 1 || in_map[u].len() <= 1) {
                q.push_back(u);
                in_queue[u] = true;
            }
        }

        let mut scale_log2 = 0.0f64;
        let mut scale_big = track_bigint.then(BigInt::one);
        let mut contracted = 0usize;

        while let Some(v) = q.pop_front() {
            in_queue[v] = false;
            if !active[v] || v == root {
                continue;
            }
            let out_len = out_map[v].len();
            let in_len = in_map[v].len();
            if out_len == 0 || in_len == 0 {
                // Disconnected component -> det == 0
                return (
                    true,
                    0,
                    Some(Vec::new()),
                    f64::NEG_INFINITY,
                    track_bigint.then(BigInt::zero),
                    contracted,
                );
            }
            if out_len == 1 {
                let (&u, &d) = out_map[v].iter().next().unwrap();
                active[v] = false;
                contracted += 1;
                scale_log2 += (d as f64).log2();
                if let Some(ref mut sb) = scale_big {
                    *sb = sb.mul_u64(d);
                }
                // Remove v -> u
                out_map[v].clear();
                in_map[u].remove(&v);
                // Redirect every w -> v to w -> u (if w != u)
                let incoming: Vec<(usize, u64)> = in_map[v].drain().collect();
                for (w, c) in incoming {
                    out_map[w].remove(&v);
                    if w != u {
                        *out_map[w].entry(u).or_insert(0) += c;
                        *in_map[u].entry(w).or_insert(0) += c;
                    }
                    if active[w] && w != root && !in_queue[w]
                        && (out_map[w].len() <= 1 || in_map[w].len() <= 1)
                    {
                        q.push_back(w);
                        in_queue[w] = true;
                    }
                }
                if active[u] && u != root && !in_queue[u]
                    && (out_map[u].len() <= 1 || in_map[u].len() <= 1)
                {
                    q.push_back(u);
                    in_queue[u] = true;
                }
            } else if in_len == 1 {
                let (&w, &d) = in_map[v].iter().next().unwrap();
                active[v] = false;
                contracted += 1;
                scale_log2 += (d as f64).log2();
                if let Some(ref mut sb) = scale_big {
                    *sb = sb.mul_u64(d);
                }
                // Remove w -> v
                in_map[v].clear();
                out_map[w].remove(&v);
                // Redirect every v -> u to w -> u (if u != w)
                let outgoing: Vec<(usize, u64)> = out_map[v].drain().collect();
                for (u, c) in outgoing {
                    in_map[u].remove(&v);
                    if u != w {
                        *out_map[w].entry(u).or_insert(0) += c;
                        *in_map[u].entry(w).or_insert(0) += c;
                    }
                    if active[u] && u != root && !in_queue[u]
                        && (out_map[u].len() <= 1 || in_map[u].len() <= 1)
                    {
                        q.push_back(u);
                        in_queue[u] = true;
                    }
                }
                if active[w] && w != root && !in_queue[w]
                    && (out_map[w].len() <= 1 || in_map[w].len() <= 1)
                {
                    q.push_back(w);
                    in_queue[w] = true;
                }
            }
        }

        let mut core_verts: Vec<usize> = (0..n).filter(|&u| active[u] && u != root).collect();
        let m = core_verts.len();
        if m > max_core {
            return (false, m, None, scale_log2, scale_big, contracted);
        }
        core_verts.sort_unstable_by_key(|&u| out_map[u].len());
        let mut idx_map = vec![usize::MAX; n];
        for (pos, &u) in core_verts.iter().enumerate() {
            idx_map[u] = pos;
        }
        let mut mat = vec![vec![0i64; m]; m];
        for (i, &u) in core_verts.iter().enumerate() {
            let mut diag = 0i64;
            for (&v, &w) in &out_map[u] {
                diag += w as i64;
                if v != root {
                    mat[i][idx_map[v]] -= w as i64;
                }
            }
            mat[i][i] = diag;
        }
        (false, m, Some(mat), scale_log2, scale_big, contracted)
    }

    fn det_exact_via_schur_bareiss(&self, root: usize) -> BigInt {
        let (disc, _, core_opt, _, scale_big, _) = self.schur_reduce_for_det(root, true, usize::MAX);
        if disc {
            return BigInt::zero();
        }
        let core = core_opt.unwrap();
        let big_mat: Vec<Vec<BigInt>> = core
            .into_iter()
            .map(|row| row.into_iter().map(BigInt::from_i64).collect())
            .collect();
        let core_det = bareiss_det_bigint(big_mat);
        scale_big.unwrap().mul(&core_det)
    }

    fn log2_det_lu(&self, root: usize) -> (f64, usize) {
        let (disc, core_dim, core_opt, scale_log2, _, _) =
            self.schur_reduce_for_det(root, false, 4096);
        if disc {
            return (f64::NEG_INFINITY, 0);
        }
        if core_dim == 0 {
            return (scale_log2, 0);
        }
        let Some(core) = core_opt else {
            return (f64::NAN, core_dim);
        };
        let fmat: Vec<Vec<f64>> = core
            .into_iter()
            .map(|row| row.into_iter().map(|v| v as f64).collect())
            .collect();
        let core_log2 = log2_det_lu_f64(fmat);
        (scale_log2 + core_log2, core_dim)
    }
}

/// Build byte-digram Eulerian multigraph G_1 directly from a BWT column L (and F = sorted L).
/// Each row i in 0..n defines a directed transition L[i] -> F[i] in the cyclic text!
fn build_byte_digram_graph(l: &[u8]) -> (EulerDigraph, [usize; 256], Vec<u8>) {
    let n = l.len();
    let mut cnt = [0usize; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut byte_to_v = [usize::MAX; 256];
    let mut v_to_byte = Vec::new();
    for b in 0..256 {
        if cnt[b] > 0 {
            byte_to_v[b] = v_to_byte.len();
            v_to_byte.push(b as u8);
        }
    }
    let sigma = v_to_byte.len();
    let mut adj_counts = vec![0u64; sigma * sigma];
    let mut pos = 0usize;
    for f_byte in 0..256 {
        let c = cnt[f_byte];
        if c == 0 {
            continue;
        }
        let vf = byte_to_v[f_byte];
        for &lb in &l[pos..pos + c] {
            let vl = byte_to_v[usize::from(lb)];
            adj_counts[vl * sigma + vf] += 1;
        }
        pos += c;
    }
    assert_eq!(pos, n);
    let mut edges = Vec::new();
    for u in 0..sigma {
        for v in 0..sigma {
            let w = adj_counts[u * sigma + v];
            if w > 0 {
                edges.push((u, v, w));
            }
        }
    }
    (EulerDigraph::from_edge_list(sigma, &edges), byte_to_v, v_to_byte)
}

// ============================================================================
// 4. Brute-Force Eulerian Tour Enumeration & Solved Analogue K Decoder
// ============================================================================

/// Brute-force enumerate ALL distinct edge-labeled Eulerian circuits starting with fixed
/// labeled edge e0 = (root -> first_v), verifying BEST Z_edge = tau_r * prod((d^+(v)-1)!).
fn brute_force_edge_eulerian_tours(g: &EulerDigraph, root: usize) -> u64 {
    // Assign explicit edge IDs 0..m
    let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); g.n_verts];
    let mut edge_id = 0usize;
    for u in 0..g.n_verts {
        for _ in 0..g.loops[u] {
            adj[u].push((u, edge_id));
            edge_id += 1;
        }
        for &(v, w) in &g.out_adj[u] {
            for _ in 0..w {
                adj[u].push((v, edge_id));
                edge_id += 1;
            }
        }
    }
    let total_edges = edge_id;
    if total_edges == 0 {
        return 1;
    }
    // Fix first edge e0 = adj[root][0]
    let (first_v, e0) = adj[root][0];
    let mut used = vec![false; total_edges];
    used[e0] = true;

    fn dfs(
        u: usize,
        depth: usize,
        total_edges: usize,
        root: usize,
        adj: &[Vec<(usize, usize)>],
        used: &mut [bool],
    ) -> u64 {
        if depth == total_edges {
            return u64::from(u == root);
        }
        let mut count = 0u64;
        for &(v, eid) in &adj[u] {
            if !used[eid] {
                used[eid] = true;
                count += dfs(v, depth + 1, total_edges, root, adj, used);
                used[eid] = false;
            }
        }
        count
    }

    dfs(first_v, 1, total_edges, root, &adj, &mut used)
}

/// Brute-force enumerate ALL distinct rooted text sequences t_0..t_{n-1} with t_0 = root
/// realizing exact bigram multigraph G, verifying:
///   Z_seq(G; root) = d^+(root) * tau_r(G) * prod((d^+(v)-1)!) / prod(a_{uv}!).
fn brute_force_seq_eulerian_tours(g: &EulerDigraph, root: usize) -> u64 {
    let mut rem: Vec<Vec<(usize, u64)>> = vec![Vec::new(); g.n_verts];
    let mut total_edges = 0usize;
    for u in 0..g.n_verts {
        if g.loops[u] > 0 {
            rem[u].push((u, g.loops[u]));
            total_edges += g.loops[u] as usize;
        }
        for &(v, w) in &g.out_adj[u] {
            rem[u].push((v, w));
            total_edges += w as usize;
        }
    }
    if total_edges == 0 {
        return 1;
    }

    fn dfs(
        u: usize,
        depth: usize,
        total_edges: usize,
        root: usize,
        rem: &mut [Vec<(usize, u64)>],
    ) -> u64 {
        if depth == total_edges {
            return u64::from(u == root);
        }
        let mut count = 0u64;
        let deg_types = rem[u].len();
        for idx in 0..deg_types {
            if rem[u][idx].1 > 0 {
                let v = rem[u][idx].0;
                rem[u][idx].1 -= 1;
                count += dfs(v, depth + 1, total_edges, root, rem);
                rem[u][idx].1 += 1;
            }
        }
        count
    }

    dfs(root, 0, total_edges, root, &mut rem)
}

/// Solved Analogue K: Exact ZERO-ORBIT-WALKING BWT decoder when tau_r(G_1) == 1
/// and prod_v (d^+(v)-1)! == 1 (i.e., Z_edge(G_1) == 1: Eulerian cactus with max d^+ <= 2).
/// Reconstructs the exact text solely from the unique spanning in-arborescence T_r!
fn decode_analogue_k_unique_arborescence(l: &[u8], p: u32) -> Option<Vec<u8>> {
    let n = l.len();
    if n == 0 {
        return Some(Vec::new());
    }
    let (g, byte_to_v, v_to_byte) = build_byte_digram_graph(l);
    // Check Analogue K sufficient condition: max out-degree <= 2 AND tau_r == 1
    if g.out_deg.iter().any(|&d| d > 2) {
        return None;
    }
    // Starting character t_0 = F[p], and t_{n-1} = L[p].
    // Wait: in forward-FL (bzip2 convention), output[0] = F[p] = L[FL[p]]!
    // And output[n-1] = L[p]. The first edge from t_{n-1} to t_0 is L[p] -> F[p]!
    let mut cnt = [0usize; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut f_byte_at_p = 0u8;
    let mut acc = 0usize;
    for b in 0..256 {
        acc += cnt[b];
        if (p as usize) < acc {
            f_byte_at_p = b as u8;
            break;
        }
    }
    let root = byte_to_v[usize::from(l[p as usize])];
    let first_v = byte_to_v[usize::from(f_byte_at_p)];

    // Verify det(\tilde{L}_root) == 1 (unique spanning in-arborescence T_root)
    let tau = g.det_exact_via_schur_bareiss(root);
    if !tau.is_one() {
        return None;
    }

    // Build reverse adjacency to find the unique spanning in-arborescence T_root via reverse-BFS.
    // Note: since tau_root == 1, there is NO ambiguity in choosing each non-root vertex's
    // tree edge! Every vertex u != root has a unique simple directed path to root;
    // the edge out of u on that unique path is its unique arborescence edge.
    let nv = g.n_verts;
    let mut rev_adj = vec![Vec::<usize>::new(); nv];
    for u in 0..nv {
        for &(v, w) in &g.out_adj[u] {
            // Since tau == 1 and G is strongly connected, any bridge/tree edge has w == 1
            // (if a tree edge had w >= 2, tau >= 2!).
            assert_eq!(w, 1, "tau_r == 1 forbids multi-edges on tree paths");
            rev_adj[v].push(u);
        }
    }
    let mut parent = vec![usize::MAX; nv];
    let mut visited = vec![false; nv];
    visited[root] = true;
    let mut bfs = VecDeque::new();
    bfs.push_back(root);
    while let Some(v) = bfs.pop_front() {
        for &u in &rev_adj[v] {
            if !visited[u] {
                visited[u] = true;
                parent[u] = v;
                bfs.push_back(u);
            }
        }
    }
    if !visited.iter().all(|&x| x) {
        return None;
    }

    // Order each vertex's outgoing edges: non-tree edges FIRST, tree edge parent[u] LAST!
    // At root: first edge is `first_v`, remaining edge (if d^+(root) == 2) comes second.
    let mut ordered_out: Vec<Vec<usize>> = vec![Vec::new(); nv];
    for u in 0..nv {
        let mut edges_u = Vec::new();
        for _ in 0..g.loops[u] {
            edges_u.push(u);
        }
        for &(v, w) in &g.out_adj[u] {
            for _ in 0..w {
                edges_u.push(v);
            }
        }
        if u == root {
            if let Some(pos) = edges_u.iter().position(|&v| v == first_v) {
                edges_u.swap(0, pos);
            } else {
                return None;
            }
            ordered_out[u] = edges_u;
        } else {
            let p_u = parent[u];
            let mut non_tree = Vec::new();
            let mut saw_tree = false;
            for v in edges_u {
                if !saw_tree && v == p_u {
                    saw_tree = true;
                } else {
                    non_tree.push(v);
                }
            }
            assert!(saw_tree);
            non_tree.push(p_u);
            ordered_out[u] = non_tree;
        }
    }

    // Walk the unique Eulerian tour in O(n) graph hops without any FL table!
    let mut cursor = vec![0usize; nv];
    let mut out = Vec::with_capacity(n);
    let mut cur = root;
    for _ in 0..n {
        let next = ordered_out[cur][cursor[cur]];
        cursor[cur] += 1;
        out.push(v_to_byte[next]);
        cur = next;
    }
    assert_eq!(cur, root);
    Some(out)
}

// ============================================================================
// 5. BWT Helpers, Open<16> Band-Transition Multigraph & FQ2 Iterated-Cut Tower
// ============================================================================

fn invert_fl_ref(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut cnt = [0usize; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut starts = [0usize; 256];
    let mut sum = 0usize;
    for a in 0..256 {
        starts[a] = sum;
        sum += cnt[a];
    }
    let mut fl = vec![0usize; n];
    for (i, &b) in l.iter().enumerate() {
        let a = usize::from(b);
        fl[starts[a]] = i;
        starts[a] += 1;
    }
    let mut out = Vec::with_capacity(n);
    let mut x = p as usize;
    for _ in 0..n {
        x = fl[x];
        out.push(l[x]);
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

struct Dsu {
    parent: Vec<u32>,
    rank: Vec<u8>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Dsu { parent: (0..n as u32).collect(), rank: vec![0; n] }
    }
    fn find(&mut self, mut x: u32) -> u32 {
        while self.parent[x as usize] != x {
            let p = self.parent[x as usize];
            self.parent[x as usize] = self.parent[p as usize];
            x = p;
        }
        x
    }
    fn union(&mut self, a: u32, b: u32) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return false;
        }
        if self.rank[ra as usize] < self.rank[rb as usize] {
            self.parent[ra as usize] = rb;
        } else if self.rank[ra as usize] > self.rank[rb as usize] {
            self.parent[rb as usize] = ra;
        } else {
            self.parent[rb as usize] = ra;
            self.rank[ra as usize] += 1;
        }
        true
    }
}

#[derive(Clone, Copy, Debug)]
struct BandPiece {
    d_start: u32,
    c_start: u32,
    width: u32,
}

struct LevelState {
    m: usize,
    mark: usize,
    pi: Vec<u32>,
    first_byte: Vec<u8>,
}

impl LevelState {
    fn from_bwt(l: &[u8], p: u32) -> Self {
        let n = l.len();
        let mut cnt = [0u32; 256];
        for &b in l {
            cnt[usize::from(b)] += 1;
        }
        let mut starts = [0u32; 256];
        let mut sum = 0u32;
        for a in 0..256 {
            starts[a] = sum;
            sum += cnt[a];
        }
        let mut fl = vec![0u32; n];
        let mut f_byte = vec![0u8; n];
        let mut cur = starts;
        for (i, &b) in l.iter().enumerate() {
            let a = usize::from(b);
            let s = cur[a] as usize;
            fl[s] = i as u32;
            f_byte[s] = b;
            cur[a] += 1;
        }
        LevelState { m: n, mark: p as usize, pi: fl, first_byte: f_byte }
    }

    fn extract_affine_bands(&self) -> Vec<BandPiece> {
        let mut bands = Vec::new();
        let mut i = 0usize;
        while i < self.m {
            let mut j = i + 1;
            while j < self.m
                && self.pi[j] == self.pi[j - 1] + 1
                && self.first_byte[j] == self.first_byte[i]
            {
                j += 1;
            }
            bands.push(BandPiece {
                d_start: i as u32,
                c_start: self.pi[i],
                width: (j - i) as u32,
            });
            i = j;
        }
        bands
    }

    /// Build the exact first-byte Eulerian multigraph G_byte(L_k) on active first bytes (|V| <= 256).
    fn byte_transition_graph(&self) -> EulerDigraph {
        let mut cnt = [0usize; 256];
        for &b in &self.first_byte {
            cnt[usize::from(b)] += 1;
        }
        let mut b2v = [usize::MAX; 256];
        let mut sigma = 0usize;
        for b in 0..256 {
            if cnt[b] > 0 {
                b2v[b] = sigma;
                sigma += 1;
            }
        }
        let mut mat = vec![0u64; sigma * sigma];
        for x in 0..self.m {
            let u = b2v[usize::from(self.first_byte[x])];
            let v = b2v[usize::from(self.first_byte[self.pi[x] as usize])];
            mat[u * sigma + v] += 1;
        }
        let mut edges = Vec::new();
        for u in 0..sigma {
            for v in 0..sigma {
                let w = mat[u * sigma + v];
                if w > 0 {
                    edges.push((u, v, w));
                }
            }
        }
        EulerDigraph::from_edge_list(sigma, &edges)
    }

    /// Build exact band-overlap Eulerian multigraph G_band(L_k) where vertices are bands
    /// and edge b1 -> b2 has weight |C_{b1} \cap D_{b2}|.
    fn band_overlap_graph(bands: &[BandPiece], m: usize) -> EulerDigraph {
        let b = bands.len();
        let mut cod_order: Vec<u32> = (0..b as u32).collect();
        cod_order.sort_unstable_by_key(|&idx| bands[idx as usize].c_start);
        let mut edges = Vec::with_capacity(2 * b);
        let (mut ci, mut di) = (0usize, 0usize);
        let mut y = 0u32;
        while ci < b && di < b {
            let v = cod_order[ci] as usize;
            let u = di;
            let c_end = bands[v].c_start + bands[v].width;
            let d_end = bands[u].d_start + bands[u].width;
            let y1 = c_end.min(d_end);
            if y1 > y {
                edges.push((v, u, (y1 - y) as u64));
            }
            y = y1;
            if c_end == y1 {
                ci += 1;
            }
            if d_end == y1 {
                di += 1;
            }
        }
        assert_eq!(y as usize, m);
        EulerDigraph::from_edge_list(b, &edges)
    }

    fn forest_section_cut(&self, bands: &[BandPiece]) -> Vec<bool> {
        let b = bands.len();
        let mut cod_order: Vec<u32> = (0..b as u32).collect();
        cod_order.sort_unstable_by_key(|&idx| bands[idx as usize].c_start);
        struct OvEdge {
            v: u32,
            u: u32,
            y0: u32,
            w: u32,
            has_mark: bool,
        }
        let mark_u32 = self.mark as u32;
        let mut edges = Vec::with_capacity(2 * b);
        let (mut ci, mut di) = (0usize, 0usize);
        let mut y = 0u32;
        while ci < b && di < b {
            let v = cod_order[ci] as usize;
            let u = di;
            let c_end = bands[v].c_start + bands[v].width;
            let d_end = bands[u].d_start + bands[u].width;
            let y1 = c_end.min(d_end);
            if y1 > y {
                edges.push(OvEdge {
                    v: v as u32,
                    u: u as u32,
                    y0: y,
                    w: y1 - y,
                    has_mark: mark_u32 >= y && mark_u32 < y1,
                });
            }
            y = y1;
            if c_end == y1 {
                ci += 1;
            }
            if d_end == y1 {
                di += 1;
            }
        }
        let mut order: Vec<usize> = (0..edges.len()).collect();
        order.sort_unstable_by(|&a, &b| {
            edges[b]
                .w
                .cmp(&edges[a].w)
                .then_with(|| edges[a].has_mark.cmp(&edges[b].has_mark))
                .then_with(|| a.cmp(&b))
        });
        let mut dsu = Dsu::new(b);
        let mut in_forest = vec![false; edges.len()];
        for &ei in &order {
            let e = &edges[ei];
            if dsu.union(e.v, e.u) {
                in_forest[ei] = true;
            }
        }
        let mut retained = vec![false; self.m];
        for (ei, e) in edges.iter().enumerate() {
            if !in_forest[ei] {
                let s = e.y0 as usize;
                let end = s + e.w as usize;
                for r in s..end {
                    retained[r] = true;
                }
            }
        }
        retained[self.mark] = true;
        retained
    }

    fn induce_return(&self, retained: &[bool]) -> Self {
        let mut rank = vec![u32::MAX; self.m];
        let mut ret_rows = Vec::new();
        for (x, &keep) in retained.iter().enumerate() {
            if keep {
                rank[x] = ret_rows.len() as u32;
                ret_rows.push(x);
            }
        }
        let m_next = ret_rows.len();
        let mark_next = rank[self.mark] as usize;
        let mut pi_next = vec![0u32; m_next];
        let mut fb_next = vec![0u8; m_next];
        for (new_idx, &start_x) in ret_rows.iter().enumerate() {
            fb_next[new_idx] = self.first_byte[start_x];
            let mut cur = self.pi[start_x] as usize;
            while !retained[cur] {
                cur = self.pi[cur] as usize;
            }
            pi_next[new_idx] = rank[cur];
        }
        LevelState { m: m_next, mark: mark_next, pi: pi_next, first_byte: fb_next }
    }

    /// Build Open<16> L1 induced system AND its exact codomain-band partition (the exact S3a bands!).
    /// In Open<16>, thin rows are rows with L-run width <= 16 (plus `mark`).
    /// On the codomain side (LF-image of retained rows = thin domain in FL!), consecutive retained
    /// rows in the same byte bucket whose excursion word and FL-destination are contiguous form an
    /// Build exact Open<16> S3a codomain band-transition Eulerian multigraph G_band^{O16}.
    /// Vertices are the exact S3a bands (B_cod ~ 1.6k-6.8k on Wikidata/JSONL): contiguous runs
    /// in bucket-sorted thin order k in 0..m where induced LF destination `dst` advances by +1
    /// and the thick excursion word is constant. Every retained row u in 0..m yields one directed
    /// edge `band_in(u) -> band_out(u)`, so deg^+(b) == deg^-(b) == w_b identically!
    fn open16_s3a_band_graph(l: &[u8], p: u32, limit: usize) -> (usize, usize, EulerDigraph) {
        let n = l.len();
        let mut cnt = [0u32; 256];
        for &b in l {
            cnt[usize::from(b)] += 1;
        }
        let mut starts = [0u32; 256];
        let mut sum = 0u32;
        for a in 0..256 {
            starts[a] = sum;
            sum += cnt[a];
        }
        let mut lf = vec![0u32; n];
        let mut cur_s = starts;
        for (i, &b) in l.iter().enumerate() {
            let a = usize::from(b);
            lf[i] = cur_s[a];
            cur_s[a] += 1;
        }

        let mut retained = vec![false; n];
        let mut pos = 0usize;
        while pos < n {
            let c = l[pos];
            let mut end = pos + 1;
            while end < n && l[end] == c {
                end += 1;
            }
            if end - pos <= limit {
                for r in pos..end {
                    retained[r] = true;
                }
            } else if (pos..end).contains(&(p as usize)) {
                retained[p as usize] = true;
            }
            pos = end;
        }

        let mut rank = vec![u32::MAX; n];
        let mut ret_rows = Vec::new();
        for (x, &keep) in retained.iter().enumerate() {
            if keep {
                rank[x] = ret_rows.len() as u32;
                ret_rows.push(x);
            }
        }
        let m = ret_rows.len();
        let mut thin_cnt = [0usize; 256];
        for &x in &ret_rows {
            thin_cnt[usize::from(l[x])] += 1;
        }
        let mut thin_starts = [0usize; 256];
        let mut tsum = 0usize;
        for a in 0..256 {
            thin_starts[a] = tsum;
            tsum += thin_cnt[a];
        }
        let mut order_k = vec![0u32; m];
        let mut bucket_of_k = vec![0u8; m];
        let mut k_cur = thin_starts;
        for (u, &x) in ret_rows.iter().enumerate() {
            let b = l[x];
            let k = k_cur[usize::from(b)];
            k_cur[usize::from(b)] += 1;
            order_k[k] = u as u32;
            bucket_of_k[k] = b;
        }

        let mut dst_k = vec![0u32; m];
        let mut hash_k = vec![0u64; m];
        for k in 0..m {
            let x0 = ret_rows[order_k[k] as usize];
            let mut cur = lf[x0] as usize;
            let mut h = 0xcbf2_9ce4_8422_2325u64;
            let mut steps = 0u64;
            while !retained[cur] {
                h ^= u64::from(l[cur]);
                h = h.wrapping_mul(0x0000_0100_0000_01B3);
                steps += 1;
                cur = lf[cur] as usize;
            }
            dst_k[k] = rank[cur];
            hash_k[k] = h ^ (steps.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        }

        let mut band_in = vec![0u32; m];
        let mut band_out = vec![0u32; m];
        let mut num_bands = 0usize;
        let mut i = 0usize;
        while i < m {
            let b0 = bucket_of_k[i];
            let h0 = hash_k[i];
            let mut j = i + 1;
            while j < m
                && bucket_of_k[j] == b0
                && hash_k[j] == h0
                && dst_k[j] == dst_k[j - 1] + 1
            {
                j += 1;
            }
            let bid = num_bands as u32;
            num_bands += 1;
            let dst0 = dst_k[i] as usize;
            let w = j - i;
            for off in 0..w {
                band_in[dst0 + off] = bid;
                band_out[order_k[i + off] as usize] = bid;
            }
            i = j;
        }

        let mut edges = Vec::with_capacity(m);
        for u in 0..m {
            edges.push((band_in[u] as usize, band_out[u] as usize, 1u64));
        }
        (m, num_bands, EulerDigraph::from_edge_list(num_bands, &edges))
    }
}

// ============================================================================
// 6. Corpora Loaders (4 Corpora: 6 Admitted, 6 Non-Wiki, 4 Code/Docs, 233 Wiki)
// ============================================================================

struct BlockL {
    name: String,
    p: u32,
    l: Vec<u8>,
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

fn load_six_admitted() -> Vec<BlockL> {
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
            BlockL { name: f, p, l }
        })
        .collect()
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool) -> Vec<BlockL> {
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
        blocks.push(BlockL { name, p, l });
    }
    blocks
}

fn resolve_corpus_path(candidates: &[&str]) -> String {
    for &p in candidates {
        if std::path::Path::new(p).exists() {
            return p.to_string();
        }
    }
    panic!("Could not locate corpus among {:?}", candidates);
}

// ============================================================================
// 7. Part 1: Self-Tests, 6,000+ Small BWT Brute-Force BEST, & Solved Analogue K
// ============================================================================

struct Lcg(u64);
impl Lcg {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
}

fn run_part1_verification() {
    println!("================================================================================");
    println!("PART 1: EXACT SYLVESTER-BAREISS, CAUCHY-BINET, BEST BRUTE-FORCE & ANALOGUE K");
    println!("================================================================================");

    // 1A. BigInt Knuth Algorithm D stress test (2,000 random multi-limb divisions)
    {
        let mut rng = Lcg(0xC49_A1A1_0001);
        for _ in 0..2000 {
            let na = 2 + (rng.next_u64() as usize % 6);
            let nb = 1 + (rng.next_u64() as usize % na);
            let mut a = BigInt {
                neg: (rng.next_u64() & 1) == 1,
                limbs: (0..na).map(|_| rng.next_u64() as u32).collect(),
            };
            let mut b = BigInt {
                neg: (rng.next_u64() & 1) == 1,
                limbs: (0..nb).map(|_| rng.next_u64() as u32).collect(),
            };
            a.norm();
            b.norm();
            if b.is_zero() {
                b = BigInt::one();
            }
            let (q, r) = a.div_rem(&b);
            let recomposed = q.mul(&b).add(&r);
            assert_eq!(recomposed, a, "BigInt div_rem reconstruction failed!");
            assert_eq!(BigInt::cmp_mag(&r.limbs, &b.limbs), Ordering::Less);
        }
        // Known exact determinant test: 3x3 matrix from Smith 1861 test in fq2_iterated_cut_tower
        // det([[2,4,4],[-6,6,12],[10,-4,-16]]) = 2*(-96+48) - 4*(96-120) + 4*(24-60) = -96 + 96 - 144 = -144
        let m = vec![
            vec![BigInt::from_i64(2), BigInt::from_i64(4), BigInt::from_i64(4)],
            vec![BigInt::from_i64(-6), BigInt::from_i64(6), BigInt::from_i64(12)],
            vec![BigInt::from_i64(10), BigInt::from_i64(-4), BigInt::from_i64(-16)],
        ];
        assert_eq!(bareiss_det_bigint(m), BigInt::from_i64(-144));
        println!("  [PASS] BigInt radix-2^32 Knuth Alg-D (2,000 random multi-limb tests) + Bareiss unit check");
    }

    // 1B. Exhaustive small BWTs: binary n=1..=11 (4,094) + ternary n=1..=6 (1,092) + 1,000 random 4..8-letter (n=4..=12)
    let mut small_texts: Vec<Vec<u8>> = Vec::with_capacity(6200);
    for n in 1..=11usize {
        for mask in 0..(1u32 << n) {
            let mut t = Vec::with_capacity(n);
            for bit in 0..n {
                t.push(if ((mask >> bit) & 1) == 0 { b'a' } else { b'b' });
            }
            small_texts.push(t);
        }
    }
    for n in 1..=6usize {
        let total = 3usize.pow(n as u32);
        for mut code in 0..total {
            let mut t = Vec::with_capacity(n);
            for _ in 0..n {
                t.push(b'a' + (code % 3) as u8);
                code /= 3;
            }
            small_texts.push(t);
        }
    }
    {
        let mut rng = Lcg(0xC49_5EED_BEEF);
        for _ in 0..1000 {
            let n = 4 + (rng.next_u64() as usize % 9); // 4..=12
            let alpha = 4 + (rng.next_u64() as usize % 5); // 4..=8
            let mut t = Vec::with_capacity(n);
            for _ in 0..n {
                t.push(b'a' + (rng.next_u64() as usize % alpha) as u8);
            }
            small_texts.push(t);
        }
    }

    let mut total_checked = 0usize;
    let mut brute_edge_checked = 0usize;
    let mut brute_seq_checked = 0usize;
    let mut snf_checked = 0usize;
    let mut root_inv_checked = 0usize;
    let mut analogue_k_small_hits = 0usize;
    let mut analogue_k_nontrivial_cacti = 0usize;
    let mut max_lu_err = 0.0f64;

    for text in &small_texts {
        let (l, p) = bwt_small(text);
        let (g, _, _) = build_byte_digram_graph(&l);
        let root = 0usize;

        // 1. Exact uncontracted Sylvester-Bareiss determinant
        let mat_i64 = g.minor_matrix_i64(root);
        let mat_big: Vec<Vec<BigInt>> = mat_i64
            .iter()
            .map(|r| r.iter().map(|&v| BigInt::from_i64(v)).collect())
            .collect();
        let tau_raw = bareiss_det_bigint(mat_big);
        assert!(!tau_raw.neg && !tau_raw.is_zero(), "Valid BWT must have tau_r >= 1");

        // 2. Exact Sylvester-Schur contraction == uncontracted Bareiss
        let tau_schur = g.det_exact_via_schur_bareiss(root);
        assert_eq!(tau_raw, tau_schur, "Schur contraction mismatch vs Bareiss!");

        // 3. Root invariance check (Eulerian property: all cofactors identical!)
        if g.n_verts >= 2 {
            let tau_alt = g.det_exact_via_schur_bareiss(g.n_verts - 1);
            assert_eq!(tau_raw, tau_alt, "Eulerian cofactor root invariance failed!");
            root_inv_checked += 1;
        }

        // 4. CA-17/18 SNF critical group order |Pic^0(G)| == tau_r(G) check
        if total_checked < 1200 || (g.n_verts >= 4 && snf_checked < 1800) {
            let (rk, pic_order) = snf_torsion_order_bigint(&mat_i64);
            assert_eq!(rk, g.n_verts.saturating_sub(1));
            assert_eq!(pic_order, tau_raw, "CA-18 |Pic^0(G)| != tau_r(G)!");
            snf_checked += 1;
        }

        // 5. Pivoted f64 LU cross-check against BigInt Bareiss
        let (log2_lu, _) = g.log2_det_lu(root);
        let log2_big = tau_raw.log2_approx();
        let err = (log2_lu - log2_big).abs();
        if err > max_lu_err {
            max_lu_err = err;
        }
        assert!(err < 1e-10, "LU error {err} too large!");

        // 6. Exact BEST Z_edge = tau_r * prod((d^+(v)-1)!)
        let mut prod_deg_fact = BigInt::one();
        let mut prod_mult_fact = BigInt::one();
        for u in 0..g.n_verts {
            let d = g.out_deg[u] as usize;
            prod_deg_fact = prod_deg_fact.mul(&bigint_factorial(d - 1));
            prod_mult_fact = prod_mult_fact.mul(&bigint_factorial(g.loops[u] as usize));
            for &(_, w) in &g.out_adj[u] {
                prod_mult_fact = prod_mult_fact.mul(&bigint_factorial(w as usize));
            }
        }
        let z_edge = tau_raw.mul(&prod_deg_fact);
        let z_seq_num = z_edge.mul_u64(g.out_deg[root]);
        let (z_seq, z_seq_rem) = z_seq_num.div_rem(&prod_mult_fact);
        assert!(z_seq_rem.is_zero(), "BEST Z_seq must be an exact integer!");

        if let Some(ze_u128) = z_edge.to_u128() {
            if ze_u128 <= 20_000 {
                let enum_edge = brute_force_edge_eulerian_tours(&g, root);
                assert_eq!(
                    enum_edge as u128, ze_u128,
                    "Brute-force Z_edge mismatch! enum={enum_edge} vs BEST={ze_u128}"
                );
                brute_edge_checked += 1;
            }
        }
        if let Some(zs_u128) = z_seq.to_u128() {
            if zs_u128 <= 20_000 {
                let enum_seq = brute_force_seq_eulerian_tours(&g, root);
                assert_eq!(
                    enum_seq as u128, zs_u128,
                    "Brute-force Z_seq mismatch! enum={enum_seq} vs BEST={zs_u128}"
                );
                brute_seq_checked += 1;
            }
        }

        // 7. Solved Analogue K verification (Z_edge == 1 => zero-orbit-walking exact decode!)
        if z_edge.is_one() {
            let decoded = decode_analogue_k_unique_arborescence(&l, p)
                .expect("Analogue K decoder must succeed whenever Z_edge == 1");
            assert_eq!(decoded, *text, "Analogue K decoded text mismatch!");
            analogue_k_small_hits += 1;
            if g.out_deg.iter().any(|&d| d == 2) {
                analogue_k_nontrivial_cacti += 1;
            }
        }

        total_checked += 1;
    }

    println!(
        "  [PASS] {total_checked} small BWTs (4,094 bin + 1,092 ter + 1,000 rand): 100.00% exact!"
    );
    println!(
        "         Brute-force Z_edge == tau_r * prod((d+-1)!): {brute_edge_checked}/{brute_edge_checked} EXACT integer equality"
    );
    println!(
        "         Brute-force Z_seq  == P_arb * M_NM:          {brute_seq_checked}/{brute_seq_checked} EXACT integer equality"
    );
    println!(
        "         Sylvester-Schur contraction == Bareiss:      {total_checked}/{total_checked} EXACT"
    );
    println!(
        "         Eulerian cofactor root invariance:           {root_inv_checked}/{root_inv_checked} EXACT"
    );
    println!(
        "         CA-17/18 |Pic^0(G)| == det(L~_r) SNF check:  {snf_checked}/{snf_checked} EXACT"
    );
    println!(
        "         Pivoted f64 LU vs BigInt max |log2 err|:     {max_lu_err:.3e} (< 1e-10)"
    );
    println!(
        "         Solved Analogue K (Z_edge == 1) exact decode: {analogue_k_small_hits} hits ({analogue_k_nontrivial_cacti} non-trivial deg-2 cacti!)"
    );

    // 1C. Large Synthetic Solved Analogue K Cactus Word (256-letter degree-2 Eulerian cactus, n=510)
    // Construction: chain of 255 2-cycles (0<->1, 1<->2, ..., 254<->255):
    //   d^+(0) = 1, d^+(255) = 1, d^+(1..254) = 2 => prod((d^+-1)!) = 1, tau_r = 1 => Z_edge = 1!
    {
        let mut cactus_text = Vec::with_capacity(510);
        for b in 0..=255u8 {
            cactus_text.push(b);
        }
        for b in (1..=254u8).rev() {
            cactus_text.push(b);
        }
        assert_eq!(cactus_text.len(), 510);
        let (l, p) = bwt_small(&cactus_text);
        let (g, _, _) = build_byte_digram_graph(&l);
        let tau = g.det_exact_via_schur_bareiss(0);
        assert!(tau.is_one(), "256-vertex cactus must have tau_r == 1");
        let (lf_bits, _, _, _) = g.local_entropies(0);
        assert_eq!(lf_bits, 0.0, "256-vertex deg-2 cactus must have local_fact_bits == 0");
        let decoded = decode_analogue_k_unique_arborescence(&l, p).unwrap();
        assert_eq!(decoded, cactus_text);
        assert_eq!(decoded, invert_fl_ref(&l, p));
        println!("  [PASS] Solved Analogue K 256-letter degree-2 cactus (n=510, |V|=256, 254 deg-2 vertices): tau_r=1, Z_edge=1, 100% exact 0-orbit-walk decode!");
    }

    // 1D. Malformed multi-cycle disconnected digraph check (260 columns): det == 0 iff disconnected!
    {
        let mut rng = Lcg(0xDEAD_BEEF_C49);
        let mut mal_disc = 0usize;
        let mut mal_conn = 0usize;
        for case in 0..260usize {
            let n = 16 + (case % 64);
            let mut l = Vec::with_capacity(n);
            if case < 130 {
                // Guaranteed disconnected byte graph: first half uses bytes 0..4, second half 10..14
                // Sort blocks so L[0..h] has only 0..4 and L[h..n] has only 10..14 => F[0..h] in 0..4, F[h..n] in 10..14!
                let h = n / 2;
                for _ in 0..h {
                    l.push((rng.next_u64() % 4) as u8);
                }
                for _ in h..n {
                    l.push(10 + (rng.next_u64() % 4) as u8);
                }
            } else {
                for _ in 0..n {
                    l.push((rng.next_u64() % 8) as u8);
                }
            }
            let (g, _, _) = build_byte_digram_graph(&l);
            let tau = g.det_exact_via_schur_bareiss(0);
            // Check strong connectivity of G via BFS from 0
            let mut vis = vec![false; g.n_verts];
            vis[0] = true;
            let mut q = VecDeque::new();
            q.push_back(0);
            while let Some(u) = q.pop_front() {
                for &(v, _) in &g.out_adj[u] {
                    if !vis[v] {
                        vis[v] = true;
                        q.push_back(v);
                    }
                }
            }
            let connected = vis.iter().all(|&x| x);
            if connected {
                assert!(!tau.is_zero(), "Connected Eulerian graph must have tau_r > 0");
                mal_conn += 1;
            } else {
                assert!(tau.is_zero(), "Disconnected Eulerian graph must have tau_r == 0");
                mal_disc += 1;
            }
        }
        println!("  [PASS] 260 malformed BWT columns: det(L~_r)==0 <-> disconnected exact ({mal_disc} disconnected det=0, {mal_conn} connected det>0)");
    }
    println!();
}

// ============================================================================
// 8. Part 2 & Part 3: Full 4-Corpus Matrix-Tree / BEST / Open<16> / FQ2 Analysis
// ============================================================================

#[derive(Clone, Debug)]
struct DigraphMetrics {
    v: usize,
    edges_distinct: usize,
    core_v: usize,
    log2_tau: f64,
    log2_tau_supp: f64,
    log2_local_fact: f64,
    log2_z_edge: f64,
    log2_nm: f64,
    log2_parb_pen: f64,
    log2_z_seq: f64,
    c47_relabel_bits: f64,
    bigint_verified: bool,
    lu_err: f64,
}

fn analyze_digraph(g: &EulerDigraph, run_bigint: bool, run_supp: bool) -> DigraphMetrics {
    let v = g.n_verts;
    let edges_distinct: usize = g
        .out_adj
        .iter()
        .enumerate()
        .map(|(u, adj)| adj.len() + usize::from(g.loops[u] > 0))
        .sum();
    // Root at max-degree vertex for best conditioning
    let root = (0..v).max_by_key(|&u| g.out_deg[u]).unwrap_or(0);
    let (log2_local_fact, log2_nm, sum_log2_d_nr, c47_relabel_bits) = g.local_entropies(root);

    let (log2_lu, core_v) = g.log2_det_lu(root);
    let (log2_tau, bigint_verified, lu_err) = if run_bigint && core_v <= 255 {
        let tau_big = g.det_exact_via_schur_bareiss(root);
        let l2_big = tau_big.log2_approx();
        let err = if l2_big.is_finite() && log2_lu.is_finite() {
            (l2_big - log2_lu).abs()
        } else {
            0.0
        };
        assert!(err < 1e-7, "BigInt vs LU mismatch: big={l2_big} lu={log2_lu} err={err}");
        (l2_big, true, err)
    } else {
        (log2_lu, false, 0.0)
    };

    let log2_tau_supp = if run_supp && v >= 2 && v <= 512 {
        let smat = g.support_minor_matrix_i64(root);
        let fmat: Vec<Vec<f64>> = smat
            .into_iter()
            .map(|row| row.into_iter().map(|x| x as f64).collect())
            .collect();
        log2_det_lu_f64(fmat)
    } else if v <= 1 {
        0.0
    } else {
        f64::NAN
    };

    let log2_z_edge = log2_tau + log2_local_fact;
    // Exact Markov arborescence connectivity penalty:
    //   -log2(P_arb) = sum_{u != root} log2(d^+(u)) - log2(tau_r) >= 0
    let log2_parb_pen = (sum_log2_d_nr - log2_tau).max(0.0);
    let log2_z_seq = log2_nm - log2_parb_pen;

    DigraphMetrics {
        v,
        edges_distinct,
        core_v,
        log2_tau,
        log2_tau_supp,
        log2_local_fact,
        log2_z_edge,
        log2_nm,
        log2_parb_pen,
        log2_z_seq,
        c47_relabel_bits,
        bigint_verified,
        lu_err,
    }
}

fn summarize_g1_corpus(label: &str, blocks: &[BlockL], run_bigint_all: bool, per_block: bool) {
    let t0 = Instant::now();
    println!("================================================================================");
    println!("PART 2 — G_1 BYTE-DIGRAM MATRIX-TREE / BEST: {label} ({} blocks)", blocks.len());
    println!("================================================================================");
    let mut metrics = Vec::with_capacity(blocks.len());
    let mut bigint_cnt = 0usize;
    let mut max_err = 0.0f64;

    if per_block {
        println!(
            "{:<13} {:>3} {:>4} {:>7} {:>10} {:>11} {:>7} {:>10} {:>7} {:>7} {:>8} {:>4}",
            "Block", "|V|", "core", "lg2_tau", "lg2_tauSup", "lg2_locFac", "tau/Z%", "lg2_NM", "-lgP_a", "C47rel", "err(LU)", "Big?"
        );
    }

    for b in blocks {
        let (g1, _, _) = build_byte_digram_graph(&b.l);
        // Run exact BigInt Bareiss on all blocks when requested OR whenever |V| <= 160
        let do_big = run_bigint_all || g1.n_verts <= 160;
        let m = analyze_digraph(&g1, do_big, true);
        if m.bigint_verified {
            bigint_cnt += 1;
            if m.lu_err > max_err {
                max_err = m.lu_err;
            }
        }
        if per_block {
            let tau_pct = 100.0 * m.log2_tau / m.log2_z_edge.max(1e-9);
            println!(
                "{:<13} {:>3} {:>4} {:>7.1} {:>10.1} {:>11.1} {:>6.3}% {:>10.1} {:>7.2} {:>7.1} {:>8.1e} {:>4}",
                b.name,
                m.v,
                m.core_v,
                m.log2_tau,
                m.log2_tau_supp,
                m.log2_local_fact,
                tau_pct,
                m.log2_nm,
                m.log2_parb_pen,
                m.c47_relabel_bits,
                m.lu_err,
                if m.bigint_verified { "YES" } else { "LU" }
            );
        }
        metrics.push(m);
    }

    let k = metrics.len() as f64;
    let mean = |f: fn(&DigraphMetrics) -> f64| metrics.iter().map(f).sum::<f64>() / k;
    let m_v = mean(|m| m.v as f64);
    let m_ed = mean(|m| m.edges_distinct as f64);
    let m_tau = mean(|m| m.log2_tau);
    let m_tsup = mean(|m| m.log2_tau_supp);
    let m_lf = mean(|m| m.log2_local_fact);
    let m_ze = mean(|m| m.log2_z_edge);
    let m_nm = mean(|m| m.log2_nm);
    let m_pen = mean(|m| m.log2_parb_pen);
    let m_zs = mean(|m| m.log2_z_seq);
    let m_c47 = mean(|m| m.c47_relabel_bits);
    let m_n = blocks.iter().map(|b| b.l.len() as f64).sum::<f64>() / k;
    println!(
        "  MEAN ({} blks, {bigint_cnt}/{} BigInt exact, max LU err={max_err:.2e}, {:.2}s):",
        blocks.len(),
        blocks.len(),
        t0.elapsed().as_secs_f64()
    );
    println!(
        "    |V|={m_v:.1} |E_dist|={m_ed:.0} | Global Arborescence log2(tau_r)={m_tau:.1} bits (unweighted support={m_tsup:.1} bits, mult-wt={:.1} bits)",
        m_tau - m_tsup
    );
    println!(
        "    Local Factorial sum log2((d+-1)!)={m_lf:.1} bits | Edge-BEST Z_edge={m_ze:.1} bits | tau_r share={:.4}%",
        100.0 * m_tau / m_ze.max(1e-9)
    );
    println!(
        "    Netto-MacMahon H_NM={m_nm:.1} bits ({:.3} b/byte) | Markov Arborescence Penalty -log2(P_arb)={m_pen:.2} bits ({:.4}% of H_NM) | Exact Z_seq={m_zs:.1} bits ({:.3} b/byte)",
        m_nm / m_n,
        100.0 * m_pen / m_nm.max(1e-9),
        m_zs / m_n
    );
    println!(
        "    CONTRAST vs C47 quarantined relabel: C47 `kirchhoff_bits_distinct`={m_c47:.1} bits vs TRUE log2(tau_r)={m_tau:.1} bits (ratio C47/true = {:.3}x!)",
        m_c47 / m_tau.max(1e-9)
    );
    println!();
}

fn summarize_open16_and_fq2(label: &str, blocks: &[BlockL], per_block: bool, max_o16_core_lu: usize) {
    let t0 = Instant::now();
    println!("================================================================================");
    println!("PART 3 — OPEN<16> BAND-TRANSITION & FQ2 ITERATED-CUT TOWER: {label} ({} blocks)", blocks.len());
    println!("================================================================================");

    // 3A. Open<16> S3a band-transition multigraph G_band^{O16}
    let mut o16_computed = 0usize;
    let mut o16_big_cnt = 0usize;
    let mut sum_bcod = 0.0f64;
    let mut sum_core = 0.0f64;
    let mut sum_tau = 0.0f64;
    let mut sum_lf = 0.0f64;
    let mut sum_nm = 0.0f64;
    let mut sum_pen = 0.0f64;
    let mut sum_m = 0.0f64;

    if per_block {
        println!(
            "  [Open<16> S3a Band-Transition Multigraph G_band^{{O16}}]:"
        );
        println!(
            "  {:<13} {:>6} {:>5} {:>5} {:>6} {:>9} {:>10} {:>7} {:>9} {:>8} {:>5}",
            "Block", "m_O16", "B_cod", "core", "shr%", "lg2_tau", "lg2_locFac", "tau/Z%", "lg2_NM", "-lgP_a", "Mode"
        );
    }

    for b in blocks {
        let (m_o16, b_cod, g_band) = LevelState::open16_s3a_band_graph(&b.l, b.p, 16);
        let root = (0..g_band.n_verts)
            .max_by_key(|&u| g_band.out_deg[u])
            .unwrap_or(0);
        let (disc, core_v, core_opt, scale_log2, _, _) =
            g_band.schur_reduce_for_det(root, false, max_o16_core_lu);
        let (lf_bits, nm_bits, sum_log2_d_nr, _) = g_band.local_entropies(root);
        let (log2_tau, mode_str) = if disc {
            (f64::NEG_INFINITY, "DISC")
        } else if let Some(core) = core_opt {
            if core_v <= 100 {
                let big = g_band.det_exact_via_schur_bareiss(root);
                let l2b = big.log2_approx();
                let fmat: Vec<Vec<f64>> = core
                    .into_iter()
                    .map(|row| row.into_iter().map(|x| x as f64).collect())
                    .collect();
                let l2lu = scale_log2 + log2_det_lu_f64(fmat);
                assert!((l2b - l2lu).abs() < 1e-7);
                o16_big_cnt += 1;
                (l2b, "Big!")
            } else {
                let fmat: Vec<Vec<f64>> = core
                    .into_iter()
                    .map(|row| row.into_iter().map(|x| x as f64).collect())
                    .collect();
                (scale_log2 + log2_det_lu_f64(fmat), "LU")
            }
        } else {
            (f64::NAN, "SKIP")
        };

        if log2_tau.is_finite() {
            let z_edge = log2_tau + lf_bits;
            let pen = (sum_log2_d_nr - log2_tau).max(0.0);
            o16_computed += 1;
            sum_bcod += b_cod as f64;
            sum_core += core_v as f64;
            sum_tau += log2_tau;
            sum_lf += lf_bits;
            sum_nm += nm_bits;
            sum_pen += pen;
            sum_m += m_o16 as f64;
            if per_block {
                println!(
                    "  {:<13} {:>6} {:>5} {:>5} {:>5.1}% {:>9.1} {:>10.1} {:>6.2}% {:>9.1} {:>8.1} {:>5}",
                    b.name,
                    m_o16,
                    b_cod,
                    core_v,
                    100.0 * (1.0 - core_v as f64 / b_cod.max(1) as f64),
                    log2_tau,
                    lf_bits,
                    100.0 * log2_tau / z_edge.max(1e-9),
                    nm_bits,
                    pen,
                    mode_str
                );
            }
        } else if per_block {
            println!(
                "  {:<13} {:>6} {:>5} {:>5} {:>5.1}% {:>9} {:>10.1} {:>7} {:>9.1} {:>8} {:>5}",
                b.name,
                m_o16,
                b_cod,
                core_v,
                100.0 * (1.0 - core_v as f64 / b_cod.max(1) as f64),
                "-",
                lf_bits,
                "-",
                nm_bits,
                "-",
                mode_str
            );
        }
    }
    if o16_computed > 0 {
        let k = o16_computed as f64;
        let m_tau = sum_tau / k;
        let m_lf = sum_lf / k;
        let m_nm = sum_nm / k;
        let m_pen = sum_pen / k;
        let m_ze = m_tau + m_lf;
        println!(
            "  Open<16> Band-Graph MEAN ({o16_computed}/{} blks, {o16_big_cnt} BigInt): B_cod={:.0} -> Schur core={:.0} (-{:.1}%) | log2(tau_r)={m_tau:.1} bits ({:.2}% of Z_edge) | locFac={m_lf:.1} bits | H_NM={m_nm:.1} bits ({:.3} b/thin-row) | -log2(P_arb)={m_pen:.1} bits ({:.2}% of H_NM)",
            blocks.len(),
            sum_bcod / k,
            sum_core / k,
            100.0 * (1.0 - sum_core / sum_bcod.max(1.0)),
            100.0 * m_tau / m_ze.max(1e-9),
            m_nm / (sum_m / k).max(1.0),
            100.0 * m_pen / m_nm.max(1e-9)
        );
    }

    // 3B. FQ2 Iterated-Cut Tower Collapse (L0 -> L1 -> ... -> L_D = 1)
    // Aggregate level-by-level across corpus for full untruncated G_byte(L_k) AND G_band(L_k) (when tractable)!
    const MAX_LVL: usize = 8;
    #[derive(Default)]
    struct LvlAgg {
        cnt: usize,
        m_sum: f64,
        v_sum: f64,
        b_sum: f64,
        tau_byte_sum: f64,
        lf_byte_sum: f64,
        nm_byte_sum: f64,
        pen_byte_sum: f64,
        band_cnt: usize,
        band_tau_sum: f64,
        band_lf_sum: f64,
        band_zero_tau_cnt: usize,
    }
    let mut agg: Vec<LvlAgg> = (0..MAX_LVL).map(|_| LvlAgg::default()).collect();

    if per_block {
        println!("\n  [FQ2 Iterated-Cut Tower — Full Untruncated G_byte(L_k) & G_band(L_k) Collapse]:");
    }

    for b in blocks {
        let mut cur = LevelState::from_bwt(&b.l, b.p);
        if per_block {
            println!("  --- {} ---", b.name);
        }
        for depth in 0..MAX_LVL {
            let bands = cur.extract_affine_bands();
            let g_byte = cur.byte_transition_graph();
            // Run exact BigInt on per-block corpora (16 core blocks) or whenever |V| <= 120
            let do_big = per_block || g_byte.n_verts <= 120;
            let mb = analyze_digraph(&g_byte, do_big, false);

            // Also compute G_band(L_k) whenever band count or Schur core is small (<= 600)
            let mut band_tau_str = String::from("-");
            let mut band_tau_val = None;
            let mut band_lf_val = 0.0f64;
            if bands.len() == 1 {
                band_tau_val = Some(0.0);
                band_tau_str = "0.0 (m=1!)".to_string();
            } else if bands.len() <= 2000 {
                let gb = LevelState::band_overlap_graph(&bands, cur.m);
                let root_b = (0..gb.n_verts).max_by_key(|&u| gb.out_deg[u]).unwrap_or(0);
                let (disc, core_b, core_opt_b, scale_l2, _, _) =
                    gb.schur_reduce_for_det(root_b, false, 600);
                let (blf, _, _, _) = gb.local_entropies(root_b);
                band_lf_val = blf;
                if !disc {
                    if let Some(core) = core_opt_b {
                        let l2t = if core_b <= 80 {
                            gb.det_exact_via_schur_bareiss(root_b).log2_approx()
                        } else {
                            let fm: Vec<Vec<f64>> = core
                                .into_iter()
                                .map(|r| r.into_iter().map(|x| x as f64).collect())
                                .collect();
                            scale_l2 + log2_det_lu_f64(fm)
                        };
                        band_tau_val = Some(l2t);
                        band_tau_str = format!("{l2t:.1} (c={}, lf={blf:.1})", bands.len());
                    }
                }
            }

            let a = &mut agg[depth];
            a.cnt += 1;
            a.m_sum += cur.m as f64;
            a.v_sum += mb.v as f64;
            a.b_sum += bands.len() as f64;
            a.tau_byte_sum += mb.log2_tau;
            a.lf_byte_sum += mb.log2_local_fact;
            a.nm_byte_sum += mb.log2_nm;
            a.pen_byte_sum += mb.log2_parb_pen;
            if let Some(bt) = band_tau_val {
                a.band_cnt += 1;
                a.band_tau_sum += bt;
                a.band_lf_sum += band_lf_val;
                if bt < 1e-9 && band_lf_val < 1e-9 {
                    a.band_zero_tau_cnt += 1;
                }
            }

            if per_block {
                println!(
                    "    L{depth}: m={:>6} B={:>6} | G_byte(|V|={:>3}): lg2_tau={:>7.1} locFac={:>10.1} ({:>5.2}%) H_NM={:>9.1} -lgP_a={:>6.2} | G_band: {band_tau_str}",
                    cur.m,
                    bands.len(),
                    mb.v,
                    mb.log2_tau,
                    mb.log2_local_fact,
                    100.0 * mb.log2_tau / mb.log2_z_edge.max(1e-9),
                    mb.log2_nm,
                    mb.log2_parb_pen,
                );
            }

            if cur.m == 1 {
                break;
            }
            let retained = cur.forest_section_cut(&bands);
            let next = cur.induce_return(&retained);
            if next.m == cur.m {
                break;
            }
            cur = next;
        }
    }

    println!("\n  [FQ2 Level-by-Level Corpus Mean Collapse ({}, {:.2}s)]:", label, t0.elapsed().as_secs_f64());
    println!(
        "  {:>3} {:>4} {:>8} {:>7} {:>5} {:>9} {:>11} {:>7} {:>9} {:>7} | {:>16}",
        "Lvl", "blks", "mean_m", "mean_B", "|V|", "lg2_tau_b", "locFac_b", "tau/Z%", "H_NM_b", "-lgP_a", "G_band(tau / K=1)"
    );
    for (depth, a) in agg.iter().enumerate() {
        if a.cnt == 0 {
            continue;
        }
        let k = a.cnt as f64;
        let mt = a.tau_byte_sum / k;
        let ml = a.lf_byte_sum / k;
        let mze = mt + ml;
        let bstr = if a.band_cnt > 0 {
            format!(
                "{:.1} ({}/{} K1)",
                a.band_tau_sum / a.band_cnt as f64,
                a.band_zero_tau_cnt,
                a.band_cnt
            )
        } else {
            "-".to_string()
        };
        println!(
            "   L{depth} {:>4} {:>8.1} {:>7.1} {:>5.1} {:>9.1} {:>11.1} {:>6.3}% {:>9.1} {:>7.2} | {:>16}",
            a.cnt,
            a.m_sum / k,
            a.b_sum / k,
            a.v_sum / k,
            mt,
            ml,
            100.0 * mt / mze.max(1e-9),
            a.nm_byte_sum / k,
            a.pen_byte_sum / k,
            bstr
        );
    }
    println!();
}

fn main() {
    let t_start = Instant::now();
    run_part1_verification();

    let wiki6 = load_six_admitted();
    let nonwiki_names = ["jsonl_0", "jsonl_1", "jsonl_2", "jsonl_3", "jsonl_4", "syslog_0"];
    let nonwiki_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/non_wikidata_blocks.bin",
    ]);
    let code_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/code_docs_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/code_docs_blocks.bin",
    ]);
    let wiki431_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/wikidata_431_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/wikidata_431_blocks.bin",
    ]);

    let nonwiki6 = load_bin_blocks(&nonwiki_path, Some(&nonwiki_names), false);
    let code_names = ["code_docs_0", "code_docs_1", "code_docs_2", "code_docs_3"];
    let code4 = load_bin_blocks(&code_path, Some(&code_names), false);
    let wiki233 = load_bin_blocks(&wiki431_path, None, true);

    println!(
        "Loaded corpora: {} Admitted, {} JSONL/Syslog, {} Code/Docs, {} Full Wikidata (total = {} real blocks)\n",
        wiki6.len(),
        nonwiki6.len(),
        code4.len(),
        wiki233.len(),
        wiki6.len() + nonwiki6.len() + code4.len() + wiki233.len()
    );

    // Part 2: Exact G_1 byte-digram Matrix-Tree / BEST across ALL 4 corpora!
    summarize_g1_corpus("6 Admitted Wikidata", &wiki6, true, true);
    summarize_g1_corpus("6 Real Non-Wikidata (JSONL + Syslog)", &nonwiki6, true, true);
    summarize_g1_corpus("4 Real Code & Docs", &code4, true, true);
    summarize_g1_corpus("233 Full wikidata.bz2", &wiki233, false, false);

    // Part 3: Open<16> band-transition multigraph + FQ2 Iterated-Cut Tower collapse
    summarize_open16_and_fq2("6 Admitted Wikidata", &wiki6, true, 2600);
    summarize_open16_and_fq2("6 Real Non-Wikidata (JSONL + Syslog)", &nonwiki6, true, 2600);
    summarize_open16_and_fq2("4 Real Code & Docs", &code4, true, 2600);
    summarize_open16_and_fq2("233 Full wikidata.bz2", &wiki233, false, 1400);

    println!("Total runtime: {:.2}s", t_start.elapsed().as_secs_f64());
}
