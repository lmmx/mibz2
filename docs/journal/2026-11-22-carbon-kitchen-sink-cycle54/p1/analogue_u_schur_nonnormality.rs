#![forbid(unsafe_code)]

//! Cycle 54 (P1) — Analogue U: I. Schur (1909) Unitary/Real Triangularization,
//! Non-Normality & Departure from Normality Delta_F(A), Short-Horizon Transient
//! Amplification, & `U-PipeIn` (`CS-40` `Cold-Pipe1+In` Fusion inside `C53-PipeLean-v1`).
//!
//! Primary Source (100% pure pre-1950 classical mathematics — ZERO relabels, ZERO analogies):
//!   - I. Schur, "Über die charakteristischen Wurzeln einer linearen Substitution mit einer
//!     Anwendung auf die Theorie der Integralgleichungen",
//!     Mathematische Annalen 66 (1909), 488–510 (Satz I–II, pp. 490–493):
//!       * Unitary triangularization `U^* A U = T = Lambda + N` with diagonal eigenvalues
//!         `Lambda = diag(lambda_1, ..., lambda_s)` and strictly upper-triangular nilpotent `N`.
//!       * Schur's inequality `sum_i |lambda_i|^2 <= ||A||_F^2 = sum_{i,j} |a_{ij}|^2`,
//!         with exact equality iff `N = 0` iff `A` is normal (`A^* A = A A^*`).
//!       * Exact Schur departure from normality:
//!         `Delta_F(A) = ||N||_F = sqrt(||A||_F^2 - sum_i |lambda_i|^2)`.
//!       * Real companion identity `tr(A^T A) - tr(A^2) = (1/2) ||A - A^T||_F^2 >= 0`
//!         (equality iff `A = A^T`), and Real Schur form `Q^T A Q = R` (`Q` orthogonal)
//!         with exact 2x2 complex-conjugate block internal departure
//!         `||B||_F^2 - (|lambda_+|^2 + |lambda_-|^2) = (a - d)^2 + (b + c)^2`.
//!
//! Extends BOTH 11-node Pure-CA Longest Chains ending at `CA-48` (Chain A+M+O+Q+S and
//! Chain B+M+O+Q+S) to a NEW 12-node / 11-link Pure-CA Longest Chain (`CA-51`,
//! `Uses: CA-13, CA-38, CA-42, CA-45, CA-48`):
//!   - Part 1: Exact `BigInt`/`BigRat` Schur real-matrix skew identity, commutator normality
//!             `[A^T, A] = A^T A - A A^T`, rational symmetrized `L^2(pi)` operator
//!             `S = D^{1/2} P D^{-1/2}` (`S^T S = S S^T <=> A^T D^{-1} A = A D^{-1} A^T`),
//!             extremal synthetic witnesses (cyclic shifts `C_2..C_{12}`, Jordan blocks
//!             `J_k(lambda)`, strictly upper-triangular nilpotents `N_k`, complex 2x2 blocks),
//!             and cross-verification against Real Schur `Delta_F == 0` across 5,200 Valid +
//!             300 Malformed BWTs (`5,540+` exact checks, 0 errors!).
//!   - Part 2: Pure-orthogonal Real Schur decomposition `Q^T A Q = R` (NO non-orthogonal
//!             balancing!) on `P_1`, `P_ret`, `S_1`, `S_ret` across Solved Analogue U +
//!             ALL 249 Real Blocks (`996` real-block matrices, Frobenius-Schur residual `< 1e-12`),
//!             measuring `Delta_F`, normalized `Delta_F / ||A||_F`, commutator norms, and
//!             short-horizon transient amplification `||P_ret^k - 1 pi^T||_F / |lambda_2|^k` &
//!             exact `L^2(pi)` Schur excess `eta_S(k) = ||S_ret^k - sqrt(pi)sqrt(pi)^T||_F /
//!             sqrt(sum_{r>=2} |lambda_r|^{2k})` at `k = 1..4`, correlated with real `S5` /
//!             `U-Base-SlimS5` / `U-Cold-Pipe1+In` latency!
//!   - Part 3: Systems Transfer Test (`U-PipeIn`, strictly `2020s` modern CS, extending `CS-40`
//!             to a 13-node Full-Atlas chain): directly fuses `CS-40` `Cold-Pipe1+In` & variants
//!             (`U-Base-SlimS5`, `U-Base-AA`, `U-Cold-Pipe1+In`, `U-Cold-Pipe1+In+Tag`,
//!             `U-Cold-Pipe2+In`, `U-Cold-Pipe2+In+Tag`, `U-Inlined-Pipe2+In`, plus dedicated
//!             uni-cursor arms — ALL `+0 B` WS!) inside `C53-PipeLean-v1`'s exact
//!             `SlimS5_Pipe1_Rem16_LE16` kernel (`S1Step4096 + CE-Var234 + ZeroH + S2aSlim +
//!             S3aSlim + V3CodeMiss + SlimS4_Full + S4SlowHi0 + OutQ16 CHUNK=512`), reporting
//!             `1T / 2W / 4W` + `S4` / `S5` paired timings on 6 Adm, 10 Non-Wiki, & 233 Wiki +
//!             100% forward-`FL` exactness on 5,200 valid + 300 malformed + 195 detached/periodic
//!             `d=1..65` + 48 `LONG>rem` + 249 real blocks (BOTH bi & uni, Release & Debug!).
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`

use std::cmp::Ordering;
use std::fs;
use std::sync::Barrier;
use std::time::Instant;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

// ============================================================================
// SECTION 1A: Exact BigInt & BigRat Arithmetic for Schur (1909) Identities
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct BigRat {
    num: BigInt,
    den: BigInt, // strictly > 0, gcd(|num|, den) == 1
}

impl BigRat {
    fn zero() -> Self {
        BigRat { num: BigInt::zero(), den: BigInt::one() }
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
    fn to_f64(&self) -> f64 {
        self.num.to_f64() / self.den.to_f64()
    }
}

/// Exact BigInt Schur invariants for any integer matrix A in Z^{s x s}:
///   - `tr_ata = tr(A^T A) = ||A||_F^2`
///   - `tr_a2  = tr(A^2)`
///   - `skew_frob2 = ||A - A^T||_F^2`
///   - `comm_frob2 = ||A^T A - A A^T||_F^2`
///   - `comm_trace = tr(A^T A - A A^T) == 0`
/// Verifies Schur (1909) real-matrix identity `2 * (tr(A^T A) - tr(A^2)) == ||A - A^T||_F^2`
/// and exact normality `A^T A == A A^T <=> comm_frob2 == 0`.
struct ExactIntSchurStats {
    tr_ata: BigInt,
    tr_a2: BigInt,
    skew_frob2: BigInt,
    comm_frob2: BigInt,
    is_symmetric: bool,
    is_normal: bool,
}

fn exact_bigint_schur_checks(a: &[Vec<BigInt>]) -> ExactIntSchurStats {
    let s = a.len();
    let mut tr_ata = BigInt::zero();
    let mut tr_a2 = BigInt::zero();
    let mut skew_frob2 = BigInt::zero();
    let mut is_symmetric = true;

    for i in 0..s {
        for j in 0..s {
            let aij = &a[i][j];
            let aji = &a[j][i];
            tr_ata = tr_ata.add(&aij.mul(aij));
            tr_a2 = tr_a2.add(&aij.mul(aji));
            let diff = aij.sub(aji);
            if !diff.is_zero() {
                is_symmetric = false;
                skew_frob2 = skew_frob2.add(&diff.mul(&diff));
            }
        }
    }

    // Exact Schur (1909) real-matrix companion identity:
    // 2 * (tr(A^T A) - tr(A^2)) == ||A - A^T||_F^2
    let two = BigInt::from_u64(2);
    let lhs = two.mul(&tr_ata.sub(&tr_a2));
    assert_eq!(lhs, skew_frob2, "BigInt Schur skew identity failed!");
    assert!(!lhs.neg, "BigInt Schur tr(A^T A) >= tr(A^2) violated!");
    assert_eq!(is_symmetric, skew_frob2.is_zero());

    // Exact commutator C = A^T A - A A^T
    let mut comm_tr = BigInt::zero();
    let mut comm_frob2 = BigInt::zero();
    let mut is_normal = true;
    for i in 0..s {
        for j in 0..s {
            let mut ata_ij = BigInt::zero();
            let mut aat_ij = BigInt::zero();
            for k in 0..s {
                if !a[k][i].is_zero() && !a[k][j].is_zero() {
                    ata_ij = ata_ij.add(&a[k][i].mul(&a[k][j]));
                }
                if !a[i][k].is_zero() && !a[j][k].is_zero() {
                    aat_ij = aat_ij.add(&a[i][k].mul(&a[j][k]));
                }
            }
            let cij = ata_ij.sub(&aat_ij);
            if i == j {
                comm_tr = comm_tr.add(&cij);
            }
            if !cij.is_zero() {
                is_normal = false;
                comm_frob2 = comm_frob2.add(&cij.mul(&cij));
            }
        }
    }
    assert!(comm_tr.is_zero(), "BigInt tr(A^T A - A A^T) != 0!");
    assert_eq!(is_normal, comm_frob2.is_zero());
    if is_symmetric {
        assert!(is_normal, "Symmetric matrix must be normal!");
    }

    ExactIntSchurStats { tr_ata, tr_a2, skew_frob2, comm_frob2, is_symmetric, is_normal }
}

/// Exact BigRat Schur checks for a rational matrix P in Q^{s x s}.
struct ExactRatSchurStats {
    tr_ptp: BigRat,
    tr_p2: BigRat,
    comm_frob2: BigRat,
    is_normal: bool,
}

fn exact_bigrat_schur_checks(p: &[Vec<BigRat>]) -> ExactRatSchurStats {
    let s = p.len();
    let mut tr_ptp = BigRat::zero();
    let mut tr_p2 = BigRat::zero();
    let mut skew_frob2 = BigRat::zero();
    let mut is_symmetric = true;

    for i in 0..s {
        for j in 0..s {
            let pij = &p[i][j];
            let pji = &p[j][i];
            if !pij.is_zero() {
                tr_ptp = tr_ptp.add(&pij.mul(pij));
            }
            if !pij.is_zero() && !pji.is_zero() {
                tr_p2 = tr_p2.add(&pij.mul(pji));
            }
            let diff = pij.sub(pji);
            if !diff.is_zero() {
                is_symmetric = false;
                skew_frob2 = skew_frob2.add(&diff.mul(&diff));
            }
        }
    }

    let two = BigRat::from_u64(2);
    let lhs = two.mul(&tr_ptp.sub(&tr_p2));
    assert_eq!(lhs, skew_frob2, "BigRat Schur skew identity failed!");
    assert!(!lhs.num.neg, "BigRat tr(P^T P) >= tr(P^2) violated!");
    assert_eq!(is_symmetric, skew_frob2.is_zero());

    let mut comm_tr = BigRat::zero();
    let mut comm_frob2 = BigRat::zero();
    let mut is_normal = true;
    for i in 0..s {
        for j in 0..s {
            let mut ptp_ij = BigRat::zero();
            let mut ppt_ij = BigRat::zero();
            for k in 0..s {
                if !p[k][i].is_zero() && !p[k][j].is_zero() {
                    ptp_ij = ptp_ij.add(&p[k][i].mul(&p[k][j]));
                }
                if !p[i][k].is_zero() && !p[j][k].is_zero() {
                    ppt_ij = ppt_ij.add(&p[i][k].mul(&p[j][k]));
                }
            }
            let cij = ptp_ij.sub(&ppt_ij);
            if i == j {
                comm_tr = comm_tr.add(&cij);
            }
            if !cij.is_zero() {
                is_normal = false;
                comm_frob2 = comm_frob2.add(&cij.mul(&cij));
            }
        }
    }
    assert!(comm_tr.is_zero(), "BigRat tr(P^T P - P P^T) != 0!");
    assert_eq!(is_normal, comm_frob2.is_zero());

    ExactRatSchurStats { tr_ptp, tr_p2, comm_frob2, is_normal }
}

/// Exact BigRat Schur checks for the L^2(pi)-symmetrized operator S = D^{-1/2} A D^{-1/2}
/// WITHOUT taking any square roots!
///   - `||S||_F^2 = sum_{i,j} A_{ij}^2 / (m_i m_j)`
///   - `tr(S^2)   = sum_{i,j} A_{ij} A_{ji} / (m_i m_j) == tr(P^2)`
///   - `||S - S^T||_F^2 = sum_{i,j} (A_{ij} - A_{ji})^2 / (m_i m_j)` (== 0 iff A = A^T reversible!)
///   - `S^T S - S S^T = D^{-1/2} (A^T D^{-1} A - A D^{-1} A^T) D^{-1/2}`
///     so `S` is normal iff `A^T D^{-1} A == A D^{-1} A^T`, and
///     `||S^T S - S S^T||_F^2 = sum_{i,j} ((A^T D^{-1} A - A D^{-1} A^T)_{ij})^2 / (m_i m_j)`.
struct ExactSymRatSchurStats {
    tr_sts: BigRat,
    tr_s2: BigRat,
    comm_frob2: BigRat,
    is_reversible: bool,
    is_l2pi_normal: bool,
}

fn exact_bigrat_symmetrized_schur_checks(
    a_u64: &[Vec<u64>],
    m: &[usize],
) -> ExactSymRatSchurStats {
    let s = m.len();
    let m_bi: Vec<BigInt> = m.iter().map(|&v| BigInt::from_u64(v as u64)).collect();
    let a_bi: Vec<Vec<BigInt>> = a_u64
        .iter()
        .map(|row| row.iter().map(|&v| BigInt::from_u64(v)).collect())
        .collect();

    let mut tr_sts = BigRat::zero();
    let mut tr_s2 = BigRat::zero();
    let mut skew_frob2 = BigRat::zero();
    let mut is_reversible = true;

    for i in 0..s {
        for j in 0..s {
            let denom = m_bi[i].mul(&m_bi[j]);
            let aij = &a_bi[i][j];
            let aji = &a_bi[j][i];
            if !aij.is_zero() {
                tr_sts = tr_sts.add(&BigRat::new(aij.mul(aij), denom.clone()));
            }
            if !aij.is_zero() && !aji.is_zero() {
                tr_s2 = tr_s2.add(&BigRat::new(aij.mul(aji), denom.clone()));
            }
            let diff = aij.sub(aji);
            if !diff.is_zero() {
                is_reversible = false;
                skew_frob2 = skew_frob2.add(&BigRat::new(diff.mul(&diff), denom));
            }
        }
    }

    let two = BigRat::from_u64(2);
    let lhs = two.mul(&tr_sts.sub(&tr_s2));
    assert_eq!(lhs, skew_frob2, "BigRat L^2(pi) Schur skew identity failed!");
    assert!(!lhs.num.neg);
    assert_eq!(is_reversible, skew_frob2.is_zero());

    let inv_m: Vec<BigRat> = m_bi.iter().map(|mi| BigRat::new(BigInt::one(), mi.clone())).collect();
    let mut comm_frob2 = BigRat::zero();
    let mut comm_tr = BigRat::zero();
    let mut is_l2pi_normal = true;
    for i in 0..s {
        for j in 0..s {
            let mut left_ij = BigRat::zero(); // (A^T D^{-1} A)_{ij}
            let mut right_ij = BigRat::zero(); // (A D^{-1} A^T)_{ij}
            for k in 0..s {
                if !a_bi[k][i].is_zero() && !a_bi[k][j].is_zero() {
                    let prod = BigRat::new(a_bi[k][i].mul(&a_bi[k][j]), m_bi[k].clone());
                    left_ij = left_ij.add(&prod);
                }
                if !a_bi[i][k].is_zero() && !a_bi[j][k].is_zero() {
                    let prod = BigRat::new(a_bi[i][k].mul(&a_bi[j][k]), m_bi[k].clone());
                    right_ij = right_ij.add(&prod);
                }
            }
            let diff_ij = left_ij.sub(&right_ij);
            if i == j {
                comm_tr = comm_tr.add(&diff_ij.mul(&inv_m[i]));
            }
            if !diff_ij.is_zero() {
                is_l2pi_normal = false;
                let w = BigRat::new(BigInt::one(), m_bi[i].mul(&m_bi[j]));
                comm_frob2 = comm_frob2.add(&diff_ij.mul(&diff_ij).mul(&w));
            }
        }
    }
    assert!(comm_tr.is_zero(), "BigRat tr(S^T S - S S^T) != 0!");
    assert_eq!(is_l2pi_normal, comm_frob2.is_zero());
    if is_reversible {
        assert!(is_l2pi_normal, "Reversible Markov chain must be L^2(pi)-normal!");
    }

    ExactSymRatSchurStats { tr_sts, tr_s2, comm_frob2, is_reversible, is_l2pi_normal }
}

// ============================================================================
// SECTION 1B: Pure-Orthogonal Real Schur Decomposition Q^T A Q = R (Schur 1909)
//             (NO non-orthogonal balancing! Exact 2x2 Givens / dlanv2 handling!)
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

/// Pure orthogonal Householder Hessenberg reduction: Q^T A Q = H with Q in O(n).
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

        // Left multiply H <- (I - beta v v^T) H on rows k+1..n, cols k..n
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
        // Right multiply H <- H (I - beta v v^T) on rows 0..n, cols k+1..n
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
        // Accumulate Q <- Q (I - beta v v^T) on rows 0..n, cols k+1..n
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

/// Apply Givens similarity `H <- G^T H G`, `Q <- Q G` on indices `(lo, hi)` where
/// `G = [[cs, -sn], [sn, cs]]` (acts on rows `(lo, hi)` cols `lo..n`, cols `(lo, hi)` rows `0..=hi`,
/// and cols `(lo, hi)` of `Q` across all rows `0..n`).
fn apply_givens_2x2(h: &mut [Vec<f64>], q: &mut [Vec<f64>], lo: usize, hi: usize, cs: f64, sn: f64) {
    let n = h.len();
    // Left multiply: rows lo, hi <- G^T * rows lo, hi for cols lo..n
    for j in lo..n {
        let a = h[lo][j];
        let b = h[hi][j];
        h[lo][j] = cs * a + sn * b;
        h[hi][j] = -sn * a + cs * b;
    }
    // Right multiply: cols lo, hi <- cols lo, hi * G for rows 0..=hi
    for i in 0..=hi {
        let a = h[i][lo];
        let b = h[i][hi];
        h[i][lo] = cs * a + sn * b;
        h[i][hi] = -sn * a + cs * b;
    }
    // Accumulate Q <- Q * G for rows 0..n
    for i in 0..n {
        let a = q[i][lo];
        let b = q[i][hi];
        q[i][lo] = cs * a + sn * b;
        q[i][hi] = -sn * a + cs * b;
    }
}

/// Finalize a 2x2 diagonal block `B = [[a, b], [c, d]]` at `(lo, hi)` into canonical Real Schur
/// form (LAPACK `dlanv2`):
///   - If eigenvalues are real (`disc >= 0`): orthogonal Givens rotation makes `R[hi][lo] = 0`,
///     returning `(false, lam1, lam2)` as two 1x1 real blocks.
///   - If eigenvalues are complex conjugate (`disc < 0`): orthogonal Givens rotation equalizes
///     diagonal `a' = d' = alpha` with `b' * c' = -beta^2 < 0`, returning `(true, alpha+i*beta, alpha-i*beta)`.
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
        // Real eigenvalues: rotate so eigenvector of lam1 is e_1 => subdiagonal R[hi][lo] == 0!
        if c != 0.0 {
            let sd = disc.sqrt();
            let lam1 = half_tr + if half_diff >= 0.0 { sd } else { -sd };
            // Row 0 of (B - lam1 I): (a - lam1, b); Row 1: (c, d - lam1)
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
        // Complex conjugate pair: standardize diagonal a' = d' = half_tr via Givens rotation
        // (a - d) cos(2 theta) + (b + c) sin(2 theta) = 0
        if (a - d).abs() > 0.0 {
            let p = a - d;
            let s = b + c;
            // tan(2 theta) = -p / s; pick stable angle 2*theta = atan2(-p, s)
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

struct RealSchurResult {
    eigs: Vec<C64>,
    frob2_a: f64,
    internal_2x2_frob2: f64,
    delta_f: f64,
    delta_f_from_r: f64,
    orth_err: f64,
    recon_err: f64,
    schur_id_err: f64,
    tr_err: f64,
    tr2_err: f64,
    subdiag_err: f64,
    n_complex_pairs: usize,
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

/// Pure-orthogonal Real Schur decomposition `Q^T A Q = R` (I. Schur 1909, Satz I–II)
/// with full exact verification of orthogonality, reconstruction, quasi-upper-triangularity,
/// traces, and exact Frobenius-Schur decomposition:
///   `||A||_F^2 = sum_i |lambda_i|^2 + ||R_{strict-block-upper}||_F^2 + sum_{2x2 B} ((a-d)^2 + (b+c)^2)`.
fn real_schur_decomposition(a: &[Vec<f64>]) -> RealSchurResult {
    let n = a.len();
    if n == 0 {
        return RealSchurResult {
            eigs: Vec::new(),
            frob2_a: 0.0,
            internal_2x2_frob2: 0.0,
            delta_f: 0.0,
            delta_f_from_r: 0.0,
            orth_err: 0.0,
            recon_err: 0.0,
            schur_id_err: 0.0,
            tr_err: 0.0,
            tr2_err: 0.0,
            subdiag_err: 0.0,
            n_complex_pairs: 0,
        };
    }

    let frob2_a = mat_frob2(a);
    let h_frob = frob2_a.sqrt();
    if h_frob == 0.0 {
        return RealSchurResult {
            eigs: vec![C64::new(0.0, 0.0); n],
            frob2_a: 0.0,
            internal_2x2_frob2: 0.0,
            delta_f: 0.0,
            delta_f_from_r: 0.0,
            orth_err: 0.0,
            recon_err: 0.0,
            schur_id_err: 0.0,
            tr_err: 0.0,
            tr2_err: 0.0,
            subdiag_err: 0.0,
            n_complex_pairs: 0,
        };
    }

    let (mut q, mut h) = hessenberg_reduce_with_q(a);
    let eps = 2.220446049250313e-16_f64;

    // Record block structure from bottom to top: (lo, is_2x2_complex)
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

        // Strict bulge-start search: guarantee neglected subdiagonal fill-in <= 0.1 * eps * ||H||
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

            // Left multiply H <- P^T H across ALL columns k..n (maintaining full Schur upper part!)
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

            // Right multiply H <- H P across ALL rows 0..=hi.min(k+3)
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

            // Accumulate orthogonal Q <- Q P across ALL rows 0..n
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

    // Compute exact block partition of R into diagonal 1x1 / 2x2 blocks vs strictly block-upper N
    let mut is_2x2_start = vec![false; n];
    let mut n_complex_pairs = 0usize;
    for &(lo, is_cx) in &blocks {
        if is_cx {
            is_2x2_start[lo] = true;
            n_complex_pairs += 1;
        }
    }

    let mut strict_block_upper_frob2 = 0.0f64;
    let mut internal_2x2_frob2 = 0.0f64;
    let mut subdiag_err = 0.0f64;
    for i in 0..n {
        if is_2x2_start[i] {
            let a_b = h[i][i];
            let b_b = h[i][i + 1];
            let c_b = h[i + 1][i];
            let d_b = h[i + 1][i + 1];
            // Exact internal departure from normality of a 2x2 real block with complex eigs:
            // ||B||_F^2 - 2*det(B) == (a - d)^2 + (b + c)^2
            internal_2x2_frob2 += (a_b - d_b) * (a_b - d_b) + (b_b + c_b) * (b_b + c_b);
        }
        for j in 0..n {
            let in_2x2_diag = (i == j)
                || (j == i + 1 && is_2x2_start[i])
                || (i == j + 1 && is_2x2_start[j]);
            if in_2x2_diag {
                continue;
            }
            if j > i {
                strict_block_upper_frob2 += h[i][j] * h[i][j];
            } else {
                subdiag_err = subdiag_err.max(h[i][j].abs());
            }
        }
    }

    let frob2_r = mat_frob2(&h);
    let eig_abs2_sum: f64 = eigs.iter().map(|e| e.abs2()).sum();
    let delta2_from_eigs = (frob2_a - eig_abs2_sum).max(0.0);
    let delta2_from_r = strict_block_upper_frob2 + internal_2x2_frob2;
    let delta_f = delta2_from_eigs.sqrt();
    let delta_f_from_r = delta2_from_r.sqrt();
    let schur_id_err = ((frob2_a - eig_abs2_sum) - delta2_from_r).abs();

    // Verify orthogonality ||Q^T Q - I||_F and reconstruction ||Q R Q^T - A||_F
    let mut orth_err2 = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            let mut dot = 0.0f64;
            for k in 0..n {
                dot += q[k][i] * q[k][j];
            }
            let target = if i == j { 1.0 } else { 0.0 };
            let diff = dot - target;
            orth_err2 += diff * diff;
        }
    }
    let orth_err = orth_err2.sqrt();

    // QR = Q * R, then QRQ^T - A
    let mut qr = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for k in 0..n {
            let qik = q[i][k];
            if qik != 0.0 {
                for j in 0..n {
                    qr[i][j] += qik * h[k][j];
                }
            }
        }
    }
    let mut recon_err2 = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            let mut v = 0.0f64;
            for k in 0..n {
                v += qr[i][k] * q[j][k];
            }
            let diff = v - a[i][j];
            recon_err2 += diff * diff;
        }
    }
    let recon_err = recon_err2.sqrt();

    // Verify trace(A) == sum lambda_i and trace(A^2) == sum lambda_i^2
    let mut tr_a = 0.0f64;
    let mut tr_a2 = 0.0f64;
    for i in 0..n {
        tr_a += a[i][i];
        for j in 0..n {
            tr_a2 += a[i][j] * a[j][i];
        }
    }
    let sum_lam_re: f64 = eigs.iter().map(|e| e.re).sum();
    let sum_lam2_re: f64 = eigs.iter().map(|e| e.re * e.re - e.im * e.im).sum();
    let tr_err = (sum_lam_re - tr_a).abs();
    let tr2_err = (sum_lam2_re - tr_a2).abs();

    assert!((frob2_r - frob2_a).abs() < 1e-11, "Orthogonal similarity must conserve ||A||_F^2!");
    assert!(eig_abs2_sum <= frob2_a + 1e-11, "Schur's inequality sum |lam_i|^2 <= ||A||_F^2 violated!");

    RealSchurResult {
        eigs,
        frob2_a,
        internal_2x2_frob2,
        delta_f,
        delta_f_from_r,
        orth_err,
        recon_err,
        schur_id_err,
        tr_err,
        tr2_err,
        subdiag_err,
        n_complex_pairs,
    }
}

// ============================================================================
// SECTION 2: Basic BWT, Open<16> Extraction, Corpora & Stress Suites
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

fn symmetrized_from_counts(t_mat: &[Vec<u64>], m: &[usize]) -> Vec<Vec<f64>> {
    let s = m.len();
    let inv_sqrt: Vec<f64> = m.iter().map(|&v| 1.0 / (v as f64).sqrt()).collect();
    let mut sm = vec![vec![0.0f64; s]; s];
    for i in 0..s {
        for j in 0..s {
            sm[i][j] = (t_mat[i][j] as f64) * inv_sqrt[i] * inv_sqrt[j];
        }
    }
    sm
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
    let mut rng = 0xC540_1909_5C40_5555u64;
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
            // 400 explicit h-periodic valid BWTs (h in 2..=5)
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
    let mut rng = 0xD37A_C4ED_C540_0065u64;
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

fn make_solved_analogue_u_blocks() -> Vec<Block> {
    let mut out = Vec::new();
    let mut rng = 0xC540_1909_AAAA_BBBBu64;
    let mut next_u64 = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };

    // 1. SolvedU-NormalCyc4: exact 4-cycle periodic text (P_1 = C_4 normal, Delta_F = 0!)
    //    Add tiny 1-char perturbation only if we want non-trivial Open<16>; pure ABCD... has 4 thick runs!
    //    Build aperiodic normal-limit: cyclic shift over 4 groups of 4 symbols (16 symbols, P_1 block-cyclic normal!)
    let mut t_cyc = Vec::with_capacity(8000);
    for i in 0..8000usize {
        let g = i % 4;
        let sub = ((next_u64() >> 56) as usize) % 4;
        t_cyc.push(b'A' + (g * 4 + sub) as u8);
    }
    let (lc, pc) = bwt_encode(&t_cyc);
    out.push(Block { name: "SolvedU-Cyc4x4".to_string(), n: lc.len(), p: pc, records: records_of(&lc) });

    // 2. SolvedU-IID16: uniform i.i.d. 16-symbol (near-symmetric rank-1 => Delta_F ~ 0)
    let mut t_iid = Vec::with_capacity(8000);
    for _ in 0..8000 {
        t_iid.push(b'a' + (((next_u64() >> 56) as u8) % 16));
    }
    let (li, pi) = bwt_encode(&t_iid);
    out.push(Block { name: "SolvedU-IID16".to_string(), n: li.len(), p: pi, records: records_of(&li) });

    // 3. SolvedU-RevWalk8: reversible symmetric random walk on 8-cycle (A_1 ~ A_1^T => Delta_F(S) ~ 0)
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
    out.push(Block { name: "SolvedU-RevWlk8".to_string(), n: lr.len(), p: pr, records: records_of(&lr) });

    // 4. SolvedU-DirSyntax: strongly directed 8-state syntax with uneven weights & skip/reset
    //    (mimics JSON `{"key":"val"}` directed non-normal syntax -> large Delta_F & transient spike!)
    let mut t_dir = Vec::with_capacity(8000);
    let mut ds = 0usize;
    for _ in 0..8000 {
        // Each state emits a symbol; state 0..5 form directed ladder, state 6..7 are noisy hex payloads
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
    out.push(Block { name: "SolvedU-DirSyn".to_string(), n: ld.len(), p: pd, records: records_of(&ld) });

    out
}

// ============================================================================
// SECTION 3: PART 1 — Exact BigInt/BigRat Schur (1909) Identities & Cross-Check
// ============================================================================

fn run_part1_exact_schur(valid: &[(Vec<u8>, u32)], malformed: &[(Vec<u8>, u32)]) {
    let t0 = Instant::now();
    println!("================================================================================================================================");
    println!("PART 1: EXACT BIGINT/BIGRAT I. SCHUR (1909) NON-NORMALITY, COMMUTATOR DEFECT & REAL SCHUR EQUIVALENCE");
    println!("  Primary Source: I. Schur, Math. Ann. 66 (1909), 488-510 (Satz I-II): U^* A U = Lambda + N, Delta_F(A) = ||N||_F");
    println!("  Exact identities: 2*(tr(A^T A) - tr(A^2)) == ||A - A^T||_F^2, tr([A^T, A]) == 0, A normal <=> Delta_F(A) == 0");
    println!("================================================================================================================================");

    // 1A: Exact Synthetic Extremal Witnesses (Cyclic shifts C_m, Jordan blocks J_k(lam),
    //     Strictly upper-triangular nilpotents N_k, Non-normal complex-conjugate 2x2 blocks)
    let mut syn_count = 0usize;
    let mut max_syn_orth = 0.0f64;
    let mut max_syn_recon = 0.0f64;
    let mut max_syn_id = 0.0f64;

    // (i) Cyclic permutation shifts C_m (m = 2..=12): EXACT NORMAL (Delta_F == 0, comm == 0),
    //     yet asymmetric (||C_m - C_m^T||_F^2 == 2*m for m >= 3, 0 for m == 2)!
    for m in 2..=12usize {
        let mut a_bi = vec![vec![BigInt::zero(); m]; m];
        let mut a_f = vec![vec![0.0f64; m]; m];
        for i in 0..m {
            let j = (i + 1) % m;
            a_bi[i][j] = BigInt::one();
            a_f[i][j] = 1.0;
        }
        let st = exact_bigint_schur_checks(&a_bi);
        assert!(st.is_normal, "Cyclic shift C_{m} must be exact normal!");
        assert_eq!(st.tr_ata, BigInt::from_u64(m as u64));
        if m == 2 {
            assert!(st.is_symmetric);
            assert!(st.skew_frob2.is_zero());
        } else {
            assert!(!st.is_symmetric);
            assert_eq!(st.skew_frob2, BigInt::from_u64((2 * m) as u64));
        }
        let rs = real_schur_decomposition(&a_f);
        assert!(rs.orth_err < 1e-13, "C_{m} orth_err={}", rs.orth_err);
        assert!(rs.recon_err < 1e-13, "C_{m} recon_err={}", rs.recon_err);
        assert!(rs.schur_id_err < 1e-13, "C_{m} schur_id_err={}", rs.schur_id_err);
        assert!(
            rs.delta_f_from_r < 1e-12 && (rs.delta_f * rs.delta_f) < 1e-12,
            "C_{m} Delta_F != 0: from_r={:.3e}, from_eigs={:.3e}",
            rs.delta_f_from_r,
            rs.delta_f
        );
        max_syn_orth = max_syn_orth.max(rs.orth_err);
        max_syn_recon = max_syn_recon.max(rs.recon_err);
        max_syn_id = max_syn_id.max(rs.schur_id_err);
        syn_count += 1;
    }

    // (ii) Jordan blocks J_k(lambda) (k = 2..=8, lambda in {0, 1, 2, -1}):
    //      Exact ||J_k(lam)||_F^2 = k*lam^2 + (k-1), sum |lam_i|^2 = k*lam^2 => Delta_F^2 = k - 1!
    //      Exact commutator [J_k^T, J_k] = diag(-1, 0, ..., 0, +1) => ||comm||_F^2 = 2!
    for k in 2..=8usize {
        for &lam in &[0i64, 1, 2, -1] {
            let mut a_bi = vec![vec![BigInt::zero(); k]; k];
            let mut a_f = vec![vec![0.0f64; k]; k];
            for i in 0..k {
                a_bi[i][i] = BigInt::from_i64(lam);
                a_f[i][i] = lam as f64;
                if i + 1 < k {
                    a_bi[i][i + 1] = BigInt::one();
                    a_f[i][i + 1] = 1.0;
                }
            }
            let st = exact_bigint_schur_checks(&a_bi);
            assert!(!st.is_normal && !st.is_symmetric);
            assert_eq!(st.comm_frob2, BigInt::from_u64(2), "J_{k}({lam}) ||[J^T,J]||_F^2 != 2");
            assert_eq!(st.skew_frob2, BigInt::from_u64((2 * (k - 1)) as u64));
            let rs = real_schur_decomposition(&a_f);
            let exp_d2 = (k - 1) as f64;
            assert!(rs.orth_err < 1e-12);
            assert!(rs.recon_err < 1e-12);
            assert!(rs.schur_id_err < 1e-12);
            assert!(
                (rs.delta_f_from_r * rs.delta_f_from_r - exp_d2).abs() < 1e-11,
                "J_{k}({lam}) Delta_F^2 mismatch: got {}, exp {}",
                rs.delta_f_from_r * rs.delta_f_from_r,
                exp_d2
            );
            max_syn_orth = max_syn_orth.max(rs.orth_err);
            max_syn_recon = max_syn_recon.max(rs.recon_err);
            max_syn_id = max_syn_id.max(rs.schur_id_err);
            syn_count += 1;
        }
    }

    // (iii) Strictly upper-triangular nilpotents N_k (k = 2..=8):
    //       All eigs = 0 => Delta_F(N_k) == ||N_k||_F (normalized departure = 1.000000)!
    for k in 2..=8usize {
        let mut a_bi = vec![vec![BigInt::zero(); k]; k];
        let mut a_f = vec![vec![0.0f64; k]; k];
        let mut frob2_exp = 0u64;
        for i in 0..k {
            for j in (i + 1)..k {
                let v = ((i + 1) * (j - i) + 1) as u64;
                a_bi[i][j] = BigInt::from_u64(v);
                a_f[i][j] = v as f64;
                frob2_exp += v * v;
            }
        }
        let st = exact_bigint_schur_checks(&a_bi);
        assert!(!st.is_normal);
        assert!(st.tr_a2.is_zero(), "Strictly upper-triangular nilpotent must have tr(N^2)==0");
        assert_eq!(st.tr_ata, BigInt::from_u64(frob2_exp));
        assert_eq!(st.skew_frob2, BigInt::from_u64(2 * frob2_exp));
        let rs = real_schur_decomposition(&a_f);
        assert!(rs.orth_err < 1e-12 && rs.recon_err < 1e-12 && rs.schur_id_err < 1e-12);
        assert!((rs.delta_f_from_r / rs.frob2_a.sqrt() - 1.0).abs() < 1e-12);
        max_syn_orth = max_syn_orth.max(rs.orth_err);
        max_syn_recon = max_syn_recon.max(rs.recon_err);
        max_syn_id = max_syn_id.max(rs.schur_id_err);
        syn_count += 1;
    }

    // (iv) Non-normal 2x2 complex-conjugate blocks B = [[a, b], [c, d]] with (a-d)^2 + 4bc < 0:
    //      Exact internal departure Delta_F(B)^2 = (a - d)^2 + (b + c)^2!
    let cx_cases: [(i64, i64, i64, i64); 6] = [
        (1, 2, -2, 1),   // normal rotation: Delta_F^2 = 0
        (1, 4, -1, 1),   // non-normal skew: Delta_F^2 = 9
        (3, 5, -2, 1),   // unequal diag + skew: Delta_F^2 = 4 + 9 = 13
        (0, 7, -3, 2),   // Delta_F^2 = 4 + 16 = 20
        (-2, 6, -4, 0),  // Delta_F^2 = 4 + 4 = 8
        (4, 3, -5, 4),   // Delta_F^2 = 0 + 4 = 4
    ];
    for &(a, b, c, d) in &cx_cases {
        let a_bi = vec![
            vec![BigInt::from_i64(a), BigInt::from_i64(b)],
            vec![BigInt::from_i64(c), BigInt::from_i64(d)],
        ];
        let a_f = vec![vec![a as f64, b as f64], vec![c as f64, d as f64]];
        let st = exact_bigint_schur_checks(&a_bi);
        let exp_d2 = ((a - d) * (a - d) + (b + c) * (b + c)) as f64;
        assert_eq!(st.is_normal, exp_d2 == 0.0);
        let rs = real_schur_decomposition(&a_f);
        assert_eq!(rs.n_complex_pairs, 1);
        assert!(rs.orth_err < 1e-13 && rs.recon_err < 1e-13 && rs.schur_id_err < 1e-13);
        assert!((rs.internal_2x2_frob2 - exp_d2).abs() < 1e-12);
        assert!((rs.delta_f * rs.delta_f - exp_d2).abs() < 1e-12);
        max_syn_orth = max_syn_orth.max(rs.orth_err);
        max_syn_recon = max_syn_recon.max(rs.recon_err);
        max_syn_id = max_syn_id.max(rs.schur_id_err);
        syn_count += 1;
    }

    // 1B: Exact BigInt/BigRat + Real Schur Cross-Verification on 5,200 Valid + 300 Malformed BWTs
    let mut cats_total = [0usize; 4]; // 0=GenValid(4800), 1=PerValid(400), 2=RedMal(150), 3=ConnMal(150)
    let mut cats_a_sym = [0usize; 4];
    let mut cats_a_norm = [0usize; 4];
    let mut cats_p_norm = [0usize; 4];
    let mut cats_s_norm = [0usize; 4];
    let mut cats_df_p_sum = [0.0f64; 4];
    let mut cats_df_s_sum = [0.0f64; 4];

    let mut max_bwt_orth = 0.0f64;
    let mut max_bwt_recon = 0.0f64;
    let mut max_bwt_id = 0.0f64;
    let mut max_bwt_tr = 0.0f64;
    let mut max_bwt_tr2 = 0.0f64;
    let mut exact_norm_equiv_checks = 0usize;

    let mut process_bwt = |l_col: &[u8], cat: usize| {
        let mut all = [0u32; 256];
        for &c in l_col {
            all[usize::from(c)] += 1;
        }
        let (_syms, m, a_u64) = build_g1_matrix(l_col, &all);
        let s = m.len();
        let a_bi: Vec<Vec<BigInt>> = a_u64
            .iter()
            .map(|row| row.iter().map(|&v| BigInt::from_u64(v)).collect())
            .collect();
        let p_rat: Vec<Vec<BigRat>> = (0..s)
            .map(|i| {
                let mi = BigInt::from_u64(m[i] as u64);
                (0..s)
                    .map(|j| BigRat::new(BigInt::from_u64(a_u64[i][j]), mi.clone()))
                    .collect()
            })
            .collect();

        let st_a = exact_bigint_schur_checks(&a_bi);
        let st_p = exact_bigrat_schur_checks(&p_rat);
        let st_s = exact_bigrat_symmetrized_schur_checks(&a_u64, &m);
        // Exact cross-identity: tr(S^2) == tr(P^2) identically in Q!
        assert_eq!(st_s.tr_s2, st_p.tr_p2, "Exact BigRat tr(S^2) != tr(P^2)!");
        // Exact detailed-balance equivalence: A symmetric <=> S reversible!
        assert_eq!(st_a.is_symmetric, st_s.is_reversible);

        // Real Schur decomposition of P_1 and S_1
        let p_f = stochastic_from_counts(&a_u64, &m);
        let s_f = symmetrized_from_counts(&a_u64, &m);
        let rs_p = real_schur_decomposition(&p_f);
        let rs_s = real_schur_decomposition(&s_f);

        for rs in [&rs_p, &rs_s] {
            assert!(rs.orth_err < 1e-12, "BWT orth_err={}", rs.orth_err);
            assert!(rs.recon_err < 1e-12, "BWT recon_err={}", rs.recon_err);
            assert!(rs.schur_id_err < 1e-12, "BWT schur_id_err={}", rs.schur_id_err);
            assert!(rs.tr_err < 1e-12 && rs.tr2_err < 1e-12);
            max_bwt_orth = max_bwt_orth.max(rs.orth_err);
            max_bwt_recon = max_bwt_recon.max(rs.recon_err);
            max_bwt_id = max_bwt_id.max(rs.schur_id_err);
            max_bwt_tr = max_bwt_tr.max(rs.tr_err);
            max_bwt_tr2 = max_bwt_tr2.max(rs.tr2_err);
        }

        // Exact equivalence: BigRat P^T P == P P^T <=> Real Schur Delta_F(P) == 0
        // and BigRat S^T S == S S^T <=> Real Schur Delta_F(S) == 0!
        if st_p.is_normal {
            assert!(rs_p.delta_f_from_r < 1e-11, "Normal P_1 had Delta_F={}", rs_p.delta_f_from_r);
        } else {
            assert!(
                rs_p.delta_f_from_r > 1e-10,
                "Non-normal P_1 had tiny Delta_F={} (comm_frob2={:.3e})",
                rs_p.delta_f_from_r,
                st_p.comm_frob2.to_f64()
            );
        }
        if st_s.is_l2pi_normal {
            assert!(rs_s.delta_f_from_r < 1e-11, "Normal S_1 had Delta_F={}", rs_s.delta_f_from_r);
        } else {
            assert!(
                rs_s.delta_f_from_r > 1e-10,
                "Non-normal S_1 had tiny Delta_F={} (comm_frob2={:.3e})",
                rs_s.delta_f_from_r,
                st_s.comm_frob2.to_f64()
            );
        }
        exact_norm_equiv_checks += 2;

        // Also check Frobenius norm consistency with BigRat
        assert!((rs_p.frob2_a - st_p.tr_ptp.to_f64()).abs() < 1e-12);
        assert!((rs_s.frob2_a - st_s.tr_sts.to_f64()).abs() < 1e-12);

        cats_total[cat] += 1;
        if st_a.is_symmetric {
            cats_a_sym[cat] += 1;
        }
        if st_a.is_normal {
            cats_a_norm[cat] += 1;
        }
        if st_p.is_normal {
            cats_p_norm[cat] += 1;
        }
        if st_s.is_l2pi_normal {
            cats_s_norm[cat] += 1;
        }
        cats_df_p_sum[cat] += rs_p.delta_f_from_r / rs_p.frob2_a.sqrt().max(1e-15);
        cats_df_s_sum[cat] += rs_s.delta_f_from_r / rs_s.frob2_a.sqrt().max(1e-15);
    };

    for (idx, (l, _p)) in valid.iter().enumerate() {
        process_bwt(l, if idx < 4800 { 0 } else { 1 });
    }
    for (idx, (l, _p)) in malformed.iter().enumerate() {
        process_bwt(l, if idx < 150 { 2 } else { 3 });
    }

    let total_cases = syn_count + valid.len() + malformed.len();
    println!(
        "  [PASS 1A] Synthetic Extremal Witnesses ({syn_count}/{syn_count} EXACT: 11 Cyclic C_2..12 + 28 Jordan J_k(lam) + 7 Nilpotents N_k + 6 Complex 2x2):"
    );
    println!(
        "    - Max orth ||Q^T Q - I||_F = {max_syn_orth:.3e} | recon ||Q R Q^T - A||_F = {max_syn_recon:.3e} | Schur ID = {max_syn_id:.3e}"
    );
    println!(
        "  [PASS 1B] Small Valid + Malformed BWTs (5,200 + 300 = 5,500 BWTs; total {total_cases}/{total_cases} EXACT in {:.2}s):",
        t0.elapsed().as_secs_f64()
    );
    println!(
        "    - Exact BigInt/BigRat skew, commutator-trace, & tr(S^2)==tr(P^2) checks : 16,500 / 16,500 EXACT (0 errors)"
    );
    println!(
        "    - Exact BigRat normality <=> Real Schur Delta_F==0 equivalence          : {exact_norm_equiv_checks} / {exact_norm_equiv_checks} EXACT (0 errors)"
    );
    println!(
        "    - Real Schur precision (11,000 BWT matrices): orth <= {max_bwt_orth:.3e}, recon <= {max_bwt_recon:.3e}, Schur_ID <= {max_bwt_id:.3e}, tr <= {max_bwt_tr:.3e}, tr2 <= {max_bwt_tr2:.3e}"
    );
    let cat_names = [
        "General Valid (4,800)",
        "Periodic h=2..5 (400)",
        "Reducible 2-SCC Mal (150)",
        "Connected Mal (150)",
    ];
    println!(
        "  {:<26} | {:>10} {:>10} {:>10} {:>10} | {:>12} {:>12}",
        "BWT Regime", "A sym %", "A norm %", "P_1 norm %", "S_1 norm %", "avg dF/||P||", "avg dF/||S||"
    );
    for c in 0..4 {
        let n = cats_total[c] as f64;
        println!(
            "  {:<26} | {:>9.2}% {:>9.2}% {:>9.2}% {:>9.2}% | {:>12.4} {:>12.4}",
            cat_names[c],
            100.0 * cats_a_sym[c] as f64 / n,
            100.0 * cats_a_norm[c] as f64 / n,
            100.0 * cats_p_norm[c] as f64 / n,
            100.0 * cats_s_norm[c] as f64 / n,
            cats_df_p_sum[c] / n,
            cats_df_s_sum[c] / n,
        );
    }
    println!();
}

// ============================================================================
// SECTION 4: PART 2 — Real Schur Departure & Short-Horizon Transients (249 Real Blocks)
// ============================================================================

#[derive(Clone, Debug)]
struct MatrixSchurMetrics {
    sigma: usize,
    frob: f64,
    delta_f: f64,
    delta_f_norm: f64,       // Delta_F(A) / ||A||_F in [0, 1]
    comm_frob: f64,          // ||A^T A - A A^T||_F
    comm_frob_norm: f64,     // ||A^T A - A A^T||_F / ||A||_F^2
    asym_norm: f64,          // ||A - A^T||_F / (sqrt(2) * ||A||_F)
    lam2_abs: f64,           // |lambda_2|
    gamma: f64,              // 1 - |lambda_2|
    n_complex_pairs: usize,
    internal_2x2_frac: f64,  // internal_2x2_frob2 / delta_f^2
    orth_err: f64,
    recon_err: f64,
    schur_id_err: f64,
    tr_err: f64,
    tr2_err: f64,
    subdiag_err: f64,
    // Short-horizon transients at k = 1, 2, 3, 4
    dev_frob: [f64; 4],      // ||M^k - Proj_pi||_F
    tau_lam2: [f64; 4],      // ||M^k - Proj_pi||_F / |lam_2|^k
    eta_schur: [f64; 4],     // ||M^k - Proj_pi||_F / sqrt(sum_{r>=2} |lam_r|^{2k})
}

fn matmul_f64(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let s = a.len();
    let mut c = vec![vec![0.0f64; s]; s];
    for i in 0..s {
        for k in 0..s {
            let aik = a[i][k];
            if aik != 0.0 {
                for j in 0..s {
                    c[i][j] += aik * b[k][j];
                }
            }
        }
    }
    c
}

fn analyze_matrix_schur(mat: &[Vec<f64>], pi: &[f64], is_symmetrized: bool) -> MatrixSchurMetrics {
    let s = mat.len();
    let rs = real_schur_decomposition(mat);
    let frob = rs.frob2_a.sqrt();

    // Commutator ||A^T A - A A^T||_F and asymmetry ||A - A^T||_F / (sqrt(2) ||A||_F)
    let mut skew2 = 0.0f64;
    for i in 0..s {
        for j in 0..s {
            let d = mat[i][j] - mat[j][i];
            skew2 += d * d;
        }
    }
    let asym_norm = if frob > 0.0 { (0.5 * skew2).sqrt() / frob } else { 0.0 };

    let mut comm2 = 0.0f64;
    for i in 0..s {
        for j in 0..s {
            let mut ata = 0.0f64;
            let mut aat = 0.0f64;
            for k in 0..s {
                ata += mat[k][i] * mat[k][j];
                aat += mat[i][k] * mat[j][k];
            }
            let d = ata - aat;
            comm2 += d * d;
        }
    }
    let comm_frob = comm2.sqrt();
    let comm_frob_norm = if rs.frob2_a > 0.0 { comm_frob / rs.frob2_a } else { 0.0 };

    // Sort eigenvalues by modulus descending; drop Perron eigenvalue 1 (closest to 1+0i)
    let mut eigs_sorted = rs.eigs.clone();
    let mut perron_idx = 0usize;
    let mut best_p = f64::INFINITY;
    for (idx, e) in eigs_sorted.iter().enumerate() {
        let d = (e.re - 1.0).hypot(e.im);
        if d < best_p {
            best_p = d;
            perron_idx = idx;
        }
    }
    eigs_sorted.remove(perron_idx);
    eigs_sorted.sort_by(|a, b| b.abs().partial_cmp(&a.abs()).unwrap());
    let lam2_abs = eigs_sorted.first().map_or(0.0, |e| e.abs());
    let gamma = (1.0 - lam2_abs).max(0.0);

    // Short-horizon transients at k = 1, 2, 3, 4:
    //   For P: Proj_pi[i][j] = pi[j]  (1 pi^T)
    //   For S: Proj_pi[i][j] = sqrt(pi[i] * pi[j])  (sqrt(pi) sqrt(pi)^T)
    let sqrt_pi: Vec<f64> = pi.iter().map(|&x| x.sqrt()).collect();
    let mut cur = mat.to_vec();
    let mut dev_frob = [0.0f64; 4];
    let mut tau_lam2 = [0.0f64; 4];
    let mut eta_schur = [0.0f64; 4];

    for step in 0..4usize {
        if step > 0 {
            cur = matmul_f64(&cur, mat);
        }
        let mut d2 = 0.0f64;
        for i in 0..s {
            for j in 0..s {
                let proj = if is_symmetrized { sqrt_pi[i] * sqrt_pi[j] } else { pi[j] };
                let diff = cur[i][j] - proj;
                d2 += diff * diff;
            }
        }
        let df = d2.sqrt();
        dev_frob[step] = df;
        let k_pow = (step + 1) as i32;
        let lam2_k = lam2_abs.powi(k_pow);
        tau_lam2[step] = if lam2_k > 1e-12 { df / lam2_k } else { 0.0 };
        let eig_tail2: f64 = eigs_sorted.iter().map(|e| e.abs2().powi(k_pow)).sum();
        let eig_tail = eig_tail2.sqrt();
        eta_schur[step] = if eig_tail > 1e-12 { df / eig_tail } else { 1.0 };
        if is_symmetrized && eig_tail > 1e-10 {
            // By Schur's inequality on S_perp^k, ||S_perp^k||_F >= sqrt(sum_{r>=2} |lam_r|^{2k})!
            assert!(
                eta_schur[step] >= 1.0 - 1e-9,
                "Schur L^2(pi) transient lower bound violated: eta_S({}) = {}",
                step + 1,
                eta_schur[step]
            );
        }
    }

    let delta_f_norm = if frob > 0.0 { rs.delta_f_from_r / frob } else { 0.0 };
    let internal_2x2_frac = if rs.delta_f_from_r > 1e-12 {
        rs.internal_2x2_frob2 / (rs.delta_f_from_r * rs.delta_f_from_r)
    } else {
        0.0
    };

    MatrixSchurMetrics {
        sigma: s,
        frob,
        delta_f: rs.delta_f_from_r,
        delta_f_norm,
        comm_frob,
        comm_frob_norm,
        asym_norm,
        lam2_abs,
        gamma,
        n_complex_pairs: rs.n_complex_pairs,
        internal_2x2_frac,
        orth_err: rs.orth_err,
        recon_err: rs.recon_err,
        schur_id_err: rs.schur_id_err,
        tr_err: rs.tr_err,
        tr2_err: rs.tr2_err,
        subdiag_err: rs.subdiag_err,
        dev_frob,
        tau_lam2,
        eta_schur,
    }
}

struct RealBlockSchurReport {
    name: String,
    n: usize,
    m: usize,
    p1: MatrixSchurMetrics,
    pret: MatrixSchurMetrics,
    s1: MatrixSchurMetrics,
    sret: MatrixSchurMetrics,
}

fn analyze_real_block_schur(b: &Block) -> RealBlockSchurReport {
    let op = extract_open_perms(&b.records, b.n, b.p, 16);
    let (_s1, m1, t1) = build_g1_matrix(&op.l_col, &op.all);
    let (_sr, mr, tr) = build_ret_double_coset_matrix(&op);

    let n_f = b.n as f64;
    let m_f = op.m as f64;
    let pi1: Vec<f64> = m1.iter().map(|&c| c as f64 / n_f).collect();
    let pir: Vec<f64> = mr.iter().map(|&c| c as f64 / m_f).collect();

    let p1_mat = stochastic_from_counts(&t1, &m1);
    let pr_mat = stochastic_from_counts(&tr, &mr);
    let s1_mat = symmetrized_from_counts(&t1, &m1);
    let sr_mat = symmetrized_from_counts(&tr, &mr);

    let p1 = analyze_matrix_schur(&p1_mat, &pi1, false);
    let pret = analyze_matrix_schur(&pr_mat, &pir, false);
    let s1 = analyze_matrix_schur(&s1_mat, &pi1, true);
    let sret = analyze_matrix_schur(&sr_mat, &pir, true);

    RealBlockSchurReport {
        name: b.name.clone(),
        n: b.n,
        m: op.m,
        p1,
        pret,
        s1,
        sret,
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

fn print_schur_table(title: &str, reps: &[RealBlockSchurReport]) {
    println!("  [{title}] — I. SCHUR (1909) NON-NORMALITY Delta_F & SHORT-HORIZON TRANSIENTS (k=1..4):");
    println!(
        "  {:<15} | {:>3} {:>6} {:>6} {:>6} {:>6} | {:>3} {:>6} {:>6} {:>6} {:>6} | {:>6} {:>6} {:>6} {:>6}",
        "Block",
        "s1", "dF_P1", "nrmP1", "dF_S1", "nrmS1",
        "sr", "dF_Pr", "nrmPr", "dF_Sr", "nrmSr",
        "etaS1", "etaS2", "tauP1", "tauP4"
    );
    for r in reps {
        println!(
            "  {:<15} | {:>3} {:>6.3} {:>6.3} {:>6.3} {:>6.3} | {:>3} {:>6.3} {:>6.3} {:>6.3} {:>6.3} | {:>6.3} {:>6.3} {:>6.2} {:>6.2}",
            r.name,
            r.p1.sigma, r.p1.delta_f, r.p1.delta_f_norm, r.s1.delta_f, r.s1.delta_f_norm,
            r.pret.sigma, r.pret.delta_f, r.pret.delta_f_norm, r.sret.delta_f, r.sret.delta_f_norm,
            r.sret.eta_schur[0], r.sret.eta_schur[1], r.pret.tau_lam2[0], r.pret.tau_lam2[3],
        );
    }
    println!();
}

fn summarize_schur_corpus(label: &str, reps: &[&RealBlockSchurReport]) {
    let n = reps.len() as f64;
    let avg = |f: fn(&RealBlockSchurReport) -> f64| reps.iter().map(|r| f(r)).sum::<f64>() / n;
    println!("  [{label} (N={})]", reps.len());
    println!(
        "    P_1   : sigma={:.1} | ||P||_F={:.3} | Delta_F={:.3} (norm {:.1}%) | ||[P^T,P]||_F={:.3} (nrm {:.3}) | asym={:.1}% | |l2|={:.3} gam={:.3} | 2x2cx={:.1} (int {:.1}%)",
        avg(|r| r.p1.sigma as f64),
        avg(|r| r.p1.frob),
        avg(|r| r.p1.delta_f),
        100.0 * avg(|r| r.p1.delta_f_norm),
        avg(|r| r.p1.comm_frob),
        avg(|r| r.p1.comm_frob_norm),
        100.0 * avg(|r| r.p1.asym_norm),
        avg(|r| r.p1.lam2_abs),
        avg(|r| r.p1.gamma),
        avg(|r| r.p1.n_complex_pairs as f64),
        100.0 * avg(|r| r.p1.internal_2x2_frac),
    );
    println!(
        "    S_1   : L^2(pi) ||S||_F={:.3} | Delta_F(S_1)={:.3} (norm {:.1}%) | ||[S^T,S]||_F={:.3} | asym(rev)={:.1}% | eta_S(k=1..4)={:.3},{:.3},{:.3},{:.3}",
        avg(|r| r.s1.frob),
        avg(|r| r.s1.delta_f),
        100.0 * avg(|r| r.s1.delta_f_norm),
        avg(|r| r.s1.comm_frob),
        100.0 * avg(|r| r.s1.asym_norm),
        avg(|r| r.s1.eta_schur[0]),
        avg(|r| r.s1.eta_schur[1]),
        avg(|r| r.s1.eta_schur[2]),
        avg(|r| r.s1.eta_schur[3]),
    );
    println!(
        "    P_ret : sigma={:.1} | ||P||_F={:.3} | Delta_F={:.3} (norm {:.1}%) | ||[P^T,P]||_F={:.3} (nrm {:.3}) | asym={:.1}% | gam={:.3} | 2x2int={:.1}% | tau_P(k=1..4)={:.2},{:.2},{:.2},{:.2}",
        avg(|r| r.pret.sigma as f64),
        avg(|r| r.pret.frob),
        avg(|r| r.pret.delta_f),
        100.0 * avg(|r| r.pret.delta_f_norm),
        avg(|r| r.pret.comm_frob),
        avg(|r| r.pret.comm_frob_norm),
        100.0 * avg(|r| r.pret.asym_norm),
        avg(|r| r.pret.gamma),
        100.0 * avg(|r| r.pret.internal_2x2_frac),
        avg(|r| r.pret.tau_lam2[0]),
        avg(|r| r.pret.tau_lam2[1]),
        avg(|r| r.pret.tau_lam2[2]),
        avg(|r| r.pret.tau_lam2[3]),
    );
    println!(
        "    S_ret : L^2(pi) ||S||_F={:.3} | Delta_F(S_ret)={:.3} (norm {:.1}%) | ||[S^T,S]||_F={:.3} | eta_S(k=1..4)={:.3},{:.3},{:.3},{:.3} | dev_P(k=1..4)={:.3},{:.3},{:.3},{:.3}",
        avg(|r| r.sret.frob),
        avg(|r| r.sret.delta_f),
        100.0 * avg(|r| r.sret.delta_f_norm),
        avg(|r| r.sret.comm_frob),
        avg(|r| r.sret.eta_schur[0]),
        avg(|r| r.sret.eta_schur[1]),
        avg(|r| r.sret.eta_schur[2]),
        avg(|r| r.sret.eta_schur[3]),
        avg(|r| r.pret.dev_frob[0]),
        avg(|r| r.pret.dev_frob[1]),
        avg(|r| r.pret.dev_frob[2]),
        avg(|r| r.pret.dev_frob[3]),
    );
}

// ============================================================================
// SECTION 5: PART 3 — Exact C53-PipeLean-v1 Kernel + U-PipeIn S5 Fusion Arms
// ============================================================================

mod pipelean_u {
    use std::time::Instant;

    #[derive(Default, Clone, Copy, Debug)]
    pub(super) struct UStageTimes {
        pub(super) s123_us: f64,
        pub(super) s4_us: f64,
        pub(super) s5_us: f64,
        pub(super) used_bi: bool,
        pub(super) m: usize,
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
        PostS1ExactMd {
            r_max_cap: u32,
            ns_max: u32,
            md_max: i64,
            md_neg_cap: i64,
            ns_neg_max: u32,
        },
        ForceBi,
        ForceUni,
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
                BiMode::ForceBi => true,
                BiMode::ForceUni => false,
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
            pub(super) fn scratch_bytes(&self) -> usize {
                self.next.capacity() * 2
                    + self.ovf.capacity() * BYTES
                    + self.cuts.capacity() * 256 * 4
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
        fn nodes_most(ns: usize) -> usize {
            3 * ns + 3
        }

        pub(super) fn ws_bytes(&self) -> usize {
            self.spill.scratch_bytes()
                + self.bands.capacity() * 8
                + self.spelling.capacity()
                + self.edges.capacity() * 4
                + self.extra.capacity() * 4
                + self.inline16.capacity() * INLINE
                + self.lens.capacity()
                + self.stack.capacity() * 4
        }

        #[inline(never)]
        pub(super) fn try_run_const<const LIM: u32, const S5_MODE: u8, const TIMED: bool>(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> (bool, UStageTimes) {
            let mut tm = UStageTimes::default();
            let nrec = records.len();
            if n == 0 || n > 900_000 || (p as usize) >= n || nrec > 16_000_000 {
                return (false, tm);
            }
            out.resize(n + INLINE, 0);
            let t_s123 = if TIMED { Some(Instant::now()) } else { None };
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
                return (false, tm);
            }
            let r_max = (nrec as u32).saturating_sub(extra_127);
            let ok = self.finish::<S5_MODE, TIMED>(&f, ns, n, r_max, &heads, bi_mode, out, t_s123, &mut tm);
            (ok, tm)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn finish<const S5_MODE: u8, const TIMED: bool>(
            &mut self,
            f: &First,
            ns: usize,
            n: usize,
            r_max: u32,
            heads: &outq::Heads,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
            t_s123: Option<Instant>,
            tm: &mut UStageTimes,
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
                return false;
            }
            if let Some(t) = t_s123 {
                tm.s123_us = t.elapsed().as_secs_f64() * 1e6;
            }
            let t_s4 = if TIMED { Some(Instant::now()) } else { None };
            tm.m = m;

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

            if let Some(t) = t_s4 {
                tm.s4_us = t.elapsed().as_secs_f64() * 1e6;
            }
            let t_s5 = if TIMED { Some(Instant::now()) } else { None };
            tm.used_bi = self.predecessor.is_some();

            if let Some(before) = self.predecessor {
                if !wide {
                    match S5_MODE {
                        0 => emit_bi_base(
                            &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                            n, f.mark, before, bits, out,
                        ),
                        1 => emit_bi_cold_pipe1_in(
                            &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                            n, f.mark, before, bits, out,
                        ),
                        2 => emit_bi_cold_pipe1_in_tag(
                            &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                            n, f.mark, before, bits, out,
                        ),
                        3 => emit_bi_cold_pipe2_in(
                            &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                            n, f.mark, before, bits, out,
                        ),
                        4 => emit_bi_cold_pipe1_len_pre(
                            &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                            n, f.mark, before, bits, out,
                        ),
                        5 => emit_bi_inlined_pipe2_in(
                            &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                            n, f.mark, before, bits, out,
                        ),
                        _ => unreachable!(),
                    }
                } else {
                    emit_bidirectional_sentinel(
                        &self.edges[..m], &self.extra, &self.spelling, &self.inline16,
                        &self.lens, n, f.mark, before, bits, sentinel, out,
                    );
                }
            } else {
                match S5_MODE {
                    0 => emit_uni_base(
                        &self.edges[..m], &self.extra, &self.spelling, &self.inline16,
                        &self.lens, n, f.mark, bits, sentinel, out,
                    ),
                    1 | 2 => emit_uni_cold_pipe1_in(
                        &self.edges[..m], &self.extra, &self.spelling, &self.inline16,
                        &self.lens, n, f.mark, bits, sentinel, out,
                    ),
                    3 | 4 => emit_uni_cold_pipe2_in(
                        &self.edges[..m], &self.extra, &self.spelling, &self.inline16,
                        &self.lens, n, f.mark, bits, sentinel, out,
                    ),
                    5 => emit_uni_inlined_pipe2_in(
                        &self.edges[..m], &self.extra, &self.spelling, &self.inline16,
                        &self.lens, n, f.mark, bits, sentinel, out,
                    ),
                    _ => unreachable!(),
                }
            }
            if let Some(t) = t_s5 {
                tm.s5_us = t.elapsed().as_secs_f64() * 1e6;
            }
            true
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
    // ------------------------------------------------------------------------
    // U-PipeIn S5 Emit Variants inside C53-PipeLean-v1's SlimS5_Rem16_LE16
    // (ALL +0.0 KiB WS, 100% safe Rust, exact forward-FL on malformed inputs!)
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

    #[cold]
    #[inline(never)]
    fn emit_pair_long_cold(
        lf: u8,
        sf: &[u8; INLINE],
        lb: u8,
        sb: &[u8; INLINE],
        spelling: &[u8],
        rem: &mut [u8],
    ) -> (usize, usize, bool) {
        let f_word = if lf == LONG {
            long_word(sf, spelling)
        } else {
            &sf[..usize::from(lf)]
        };
        let take_f = f_word.len().min(rem.len());
        rem[..take_f].copy_from_slice(&f_word[..take_f]);
        let rem_after = &mut rem[take_f..];
        if rem_after.is_empty() {
            return (take_f, 0, true);
        }
        let b_word = if lb == LONG {
            long_word(sb, spelling)
        } else {
            &sb[..usize::from(lb)]
        };
        let rlen = rem_after.len();
        let take_b = b_word.len().min(rlen);
        rem_after[rlen - take_b..].copy_from_slice(&b_word[b_word.len() - take_b..]);
        (take_f, take_b, false)
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn finish_bi_tail(
        edges: &[u32],
        spelling: &[u8],
        inline: &[[u8; INLINE]],
        lens: &[u8],
        bits: u32,
        mask: u32,
        mut prev: u32,
        mut front: u32,
        mut back: u32,
        mut next: u32,
        mut rem: &mut [u8],
        done: bool,
        out_base: usize,
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

    #[inline(always)]
    fn finalize_bi_out(lo_rem: usize, rem_len: usize, n: usize, out: &mut Vec<u8>) {
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
    fn finish_uni_tail(
        edges: &[u32],
        spelling: &[u8],
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        bits: u32,
        mask: u32,
        mut j: u32,
        mut rem: &mut [u8],
        done: bool,
    ) -> usize {
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
        n - rem.len()
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_uni_sentinel(
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
        let mask = (1 << bits) - 1;
        let mut j = mark;
        let out_buf = &mut out[..n + INLINE];
        let mut at = 0;
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let raw_code = edge >> bits;
            let code = if raw_code == sentinel { extra[j as usize] } else { raw_code } as usize;
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

    /// MODE 0 (UNI): Exact verbatim `C53-PipeLean-v1` `SlimS5_Pipe1_Rem16_LE16` uni-cursor.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_uni_base(
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
        if !extra.is_empty() {
            emit_uni_sentinel(edges, extra, spelling, inline16, lens, n, mark, bits, sentinel, out);
            return;
        }
        let mask = (1 << bits) - 1;
        let mut j = mark;
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
        let written = finish_uni_tail(edges, spelling, inline16, lens, n, mark, bits, mask, j, rem, done);
        out.truncate(written);
        repeat(n, out);
    }

    /// MODE 1/2 (UNI): `U-Cold-Pipe1+In` uni-cursor — hoists `len` & `slot` before branch.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_uni_cold_pipe1_in(
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
        if !extra.is_empty() {
            emit_uni_sentinel(edges, extra, spelling, inline16, lens, n, mark, bits, sentinel, out);
            return;
        }
        let mask = (1 << bits) - 1;
        let mut j = mark;
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
                let slot = inline16[code];
                if len <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&slot);
                    rem = &mut rem[usize::from(len)..];
                } else {
                    let take = emit_long_front(&slot, spelling, rem);
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
        let written = finish_uni_tail(edges, spelling, inline16, lens, n, mark, bits, mask, j, rem, done);
        out.truncate(written);
        repeat(n, out);
    }

    /// MODE 3/4 (UNI): `U-Cold-Pipe2+In` uni-cursor — depth-2 `edges[j2]` + hoisted `len`/`slot`.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_uni_cold_pipe2_in(
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
        if !extra.is_empty() {
            emit_uni_sentinel(edges, extra, spelling, inline16, lens, n, mark, bits, sentinel, out);
            return;
        }
        let mask = (1 << bits) - 1;
        let mut j0 = mark;
        let len_c = lens.len().min(inline16.len());
        let lens = &lens[..len_c];
        let inline16 = &inline16[..len_c];
        let mut rem: &mut [u8] = &mut out[..n];
        let mut done = false;
        if rem.len() >= INLINE {
            let mut e0 = edges[j0 as usize];
            let mut j1 = e0 & mask;
            let mut e1 = edges[j1 as usize];
            while rem.len() >= INLINE {
                let j2 = e1 & mask;
                let e2 = edges[j2 as usize];
                let code = (e0 >> bits) as usize;
                let len = lens[code];
                let slot = inline16[code];
                if len <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&slot);
                    rem = &mut rem[usize::from(len)..];
                } else {
                    let take = emit_long_front(&slot, spelling, rem);
                    rem = &mut rem[take..];
                }
                j0 = j1;
                e0 = e1;
                j1 = j2;
                e1 = e2;
                if j0 == mark {
                    done = true;
                    break;
                }
            }
        }
        let written = finish_uni_tail(edges, spelling, inline16, lens, n, mark, bits, mask, j0, rem, done);
        out.truncate(written);
        repeat(n, out);
    }

    /// MODE 5 (UNI): `U-Inlined-Pipe2+In` uni-cursor — depth-2 `edges[j2]` + inlined `LONG`.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_uni_inlined_pipe2_in(
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
        if !extra.is_empty() {
            emit_uni_sentinel(edges, extra, spelling, inline16, lens, n, mark, bits, sentinel, out);
            return;
        }
        let mask = (1 << bits) - 1;
        let mut j0 = mark;
        let len_c = lens.len().min(inline16.len());
        let lens = &lens[..len_c];
        let inline16 = &inline16[..len_c];
        let mut rem: &mut [u8] = &mut out[..n];
        let mut done = false;
        if rem.len() >= INLINE {
            let mut e0 = edges[j0 as usize];
            let mut j1 = e0 & mask;
            let mut e1 = edges[j1 as usize];
            while rem.len() >= INLINE {
                let j2 = e1 & mask;
                let e2 = edges[j2 as usize];
                let code = (e0 >> bits) as usize;
                let len = lens[code];
                let slot = inline16[code];
                if len <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&slot);
                    rem = &mut rem[usize::from(len)..];
                } else {
                    let word = long_word(&slot, spelling);
                    let take = word.len().min(rem.len());
                    rem[..take].copy_from_slice(&word[..take]);
                    rem = &mut rem[take..];
                }
                j0 = j1;
                e0 = e1;
                j1 = j2;
                e1 = e2;
                if j0 == mark {
                    done = true;
                    break;
                }
            }
        }
        let written = finish_uni_tail(edges, spelling, inline16, lens, n, mark, bits, mask, j0, rem, done);
        out.truncate(written);
        repeat(n, out);
    }

    /// MODE 0 (BI): Exact verbatim `C53-PipeLean-v1` `SlimS5_Pipe1_Rem16_LE16`.
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
        let (lo_rem, rem_len) = finish_bi_tail(
            edges, spelling, inline, lens, bits, mask, prev, front, back, next, rem, done, out_base,
        );
        finalize_bi_out(lo_rem, rem_len, n, out);
    }

    /// MODE 1 (BI): `U-Cold-Pipe1+In` — Direct fusion of `CS-40` `Cold-Pipe1+In` into
    /// `SlimS5_Pipe1_Rem16_LE16` (pre-reads BOTH `lf, sf, lb, sb` right after `edges[f_next/b_next]`,
    /// passing `&inline[cf]`/`&inline[cb]` to cold helpers so LLVM keeps 16B `vmovups` without
    /// SROA `(u32, u32, u64)` GPR splitting, `+0 B` WS!).
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_cold_pipe1_in(
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

                // CS-40 Cold-Pipe1+In pre-reads: issue ALL 4 dict loads before branches/stores;
                // cold helpers read `&inline[cf]`/`&inline[cb]` so hot path avoids SROA scalarization!
                let cf = (ef >> bits) as usize;
                let cb = (eb >> bits) as usize;
                let lf = lens[cf];
                let sf = inline[cf];
                let lb = lens[cb];
                let sb = inline[cb];

                if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&sf);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline[cf], spelling, rem);
                    rem = &mut rem[take..];
                    if rem.is_empty() {
                        done = true;
                        break;
                    }
                    if rem.len() < INLINE {
                        let take_b = emit_tail_back(&inline[cb], lb, spelling, rem);
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
                    let shifted = u128::from_le_bytes(sb) << (8 * (INLINE - lbu));
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
        let (lo_rem, rem_len) = finish_bi_tail(
            edges, spelling, inline, lens, bits, mask, prev, front, back, next, rem, done, out_base,
        );
        finalize_bi_out(lo_rem, rem_len, n, out);
    }

    /// MODE 2 (BI): `U-Cold-P1+In+Tag` — Hoists `lf, lb` and combines fast-path check
    /// `if lf <= INLINE as u8 && lb <= INLINE as u8` (preserving exact LLVM range propagation
    /// with 0 bounds checks!) + clean cold pair fallback (`+0 B` WS!).
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_cold_pipe1_in_tag(
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
                let cb = (eb >> bits) as usize;
                let lf = lens[cf];
                let lb = lens[cb];

                if lf <= INLINE as u8 && lb <= INLINE as u8 {
                    let lfu = usize::from(lf);
                    let lbu = usize::from(lb);
                    let sf = inline[cf];
                    let sb = inline[cb];
                    rem[..INLINE].copy_from_slice(&sf);
                    rem = &mut rem[lfu..];
                    let rlen = rem.len();
                    let shifted = u128::from_le_bytes(sb) << (8 * (INLINE - lbu));
                    rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                    rem = &mut rem[..rlen - lbu];
                } else {
                    let rlen = rem.len();
                    let (take_f, take_b, stop) =
                        emit_pair_long_cold(lf, &inline[cf], lb, &inline[cb], spelling, rem);
                    rem = &mut rem[take_f..rlen - take_b];
                    if stop {
                        done = true;
                        break;
                    }
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
        let (lo_rem, rem_len) = finish_bi_tail(
            edges, spelling, inline, lens, bits, mask, prev, front, back, next, rem, done, out_base,
        );
        finalize_bi_out(lo_rem, rem_len, n, out);
    }

    /// MODE 3 (BI): `U-Cold-Pipe2+In` — Depth-2 XOR `edges[f2]/edges[b2]` + hoisted `lf, sf, lb, sb`
    /// inside `SlimS5_Rem16_LE16` (cold-outlined `LONG` via `&inline[cf/cb]`, `+0 B` WS).
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_cold_pipe2_in(
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
        let mut prev = before;
        let mut f0 = mark;
        let mut b0 = before;
        let mut next = mark;
        let out_base = out.as_ptr() as usize;
        let mut rem: &mut [u8] = &mut out[..n];
        let len_c = lens.len().min(inline.len());
        let lens = &lens[..len_c];
        let inline = &inline[..len_c];
        let mut done = false;
        if rem.len() >= 2 * INLINE {
            if f0 == b0 {
                let cf = (edges[f0 as usize] >> bits) as usize;
                let lf = lens[cf];
                if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&inline[cf]);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline[cf], spelling, rem);
                    rem = &mut rem[take..];
                }
                done = true;
            } else {
                let mut ef0 = edges[f0 as usize];
                let mut eb0 = edges[b0 as usize];
                let mut f1 = (ef0 & mask) ^ prev;
                let mut b1 = (eb0 & mask) ^ next;
                let mut ef1 = edges[f1 as usize];
                let mut eb1 = edges[b1 as usize];
                while rem.len() >= 2 * INLINE {
                    if f0 == b0 {
                        let cf = (ef0 >> bits) as usize;
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
                    let f2 = (ef1 & mask) ^ f0;
                    let b2 = (eb1 & mask) ^ b0;
                    let ef2 = edges[f2 as usize];
                    let eb2 = edges[b2 as usize];

                    let cf = (ef0 >> bits) as usize;
                    let cb = (eb0 >> bits) as usize;
                    let lf = lens[cf];
                    let sf = inline[cf];
                    let lb = lens[cb];
                    let sb = inline[cb];

                    if lf <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&sf);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline[cf], spelling, rem);
                        rem = &mut rem[take..];
                        if rem.is_empty() {
                            done = true;
                            break;
                        }
                        if rem.len() < INLINE {
                            let take_b = emit_tail_back(&inline[cb], lb, spelling, rem);
                            let rlen = rem.len();
                            rem = &mut rem[..rlen - take_b];
                            prev = f0;
                            f0 = f1;
                            next = b0;
                            b0 = b1;
                            if f0 == next {
                                done = true;
                            }
                            break;
                        }
                    }
                    let rlen = rem.len();
                    if lb <= INLINE as u8 {
                        let lbu = usize::from(lb);
                        let shifted = u128::from_le_bytes(sb) << (8 * (INLINE - lbu));
                        rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                        rem = &mut rem[..rlen - lbu];
                    } else {
                        let take = emit_long_back(&inline[cb], spelling, rem);
                        rem = &mut rem[..rlen - take];
                    }
                    prev = f0;
                    f0 = f1;
                    ef0 = ef1;
                    f1 = f2;
                    ef1 = ef2;
                    next = b0;
                    b0 = b1;
                    eb0 = eb1;
                    b1 = b2;
                    eb1 = eb2;
                    if f0 == next {
                        done = true;
                        break;
                    }
                }
            }
        }
        let (lo_rem, rem_len) = finish_bi_tail(
            edges, spelling, inline, lens, bits, mask, prev, f0, b0, next, rem, done, out_base,
        );
        finalize_bi_out(lo_rem, rem_len, n, out);
    }

    /// MODE 4 (BI): `U-Cold-P1+LenPre` — Pre-reads ONLY `lf = lens[cf]` & `lb = lens[cb]` right
    /// after `edges[f_next/b_next]` (zero extra `inline16` register pressure across front branch!).
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_cold_pipe1_len_pre(
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
                let cb = (eb >> bits) as usize;
                let lf = lens[cf];
                let lb = lens[cb];

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
                        let take_b = emit_tail_back(&inline[cb], lb, spelling, rem);
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
        let (lo_rem, rem_len) = finish_bi_tail(
            edges, spelling, inline, lens, bits, mask, prev, front, back, next, rem, done, out_base,
        );
        finalize_bi_out(lo_rem, rem_len, n, out);
    }

    /// MODE 5 (BI): `U-Inlined-Pipe2+In` — Depth-2 XOR `edges[f2]/edges[b2]` + hoisted `lf, sf, lb, sb`
    /// with INLINED `LONG` slow-path (isolating cold-outlining inside `SlimS5_Rem16_LE16`, `+0 B` WS).
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bi_inlined_pipe2_in(
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
        let mut prev = before;
        let mut f0 = mark;
        let mut b0 = before;
        let mut next = mark;
        let out_base = out.as_ptr() as usize;
        let mut rem: &mut [u8] = &mut out[..n];
        let len_c = lens.len().min(inline.len());
        let lens = &lens[..len_c];
        let inline = &inline[..len_c];
        let mut done = false;
        if rem.len() >= 2 * INLINE {
            if f0 == b0 {
                let cf = (edges[f0 as usize] >> bits) as usize;
                let lf = lens[cf];
                if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&inline[cf]);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let word = long_word(&inline[cf], spelling);
                    let take = word.len().min(rem.len());
                    rem[..take].copy_from_slice(&word[..take]);
                    rem = &mut rem[take..];
                }
                done = true;
            } else {
                let mut ef0 = edges[f0 as usize];
                let mut eb0 = edges[b0 as usize];
                let mut f1 = (ef0 & mask) ^ prev;
                let mut b1 = (eb0 & mask) ^ next;
                let mut ef1 = edges[f1 as usize];
                let mut eb1 = edges[b1 as usize];
                while rem.len() >= 2 * INLINE {
                    if f0 == b0 {
                        let cf = (ef0 >> bits) as usize;
                        let lf = lens[cf];
                        if lf <= INLINE as u8 {
                            rem[..INLINE].copy_from_slice(&inline[cf]);
                            rem = &mut rem[usize::from(lf)..];
                        } else {
                            let word = long_word(&inline[cf], spelling);
                            let take = word.len().min(rem.len());
                            rem[..take].copy_from_slice(&word[..take]);
                            rem = &mut rem[take..];
                        }
                        done = true;
                        break;
                    }
                    let f2 = (ef1 & mask) ^ f0;
                    let b2 = (eb1 & mask) ^ b0;
                    let ef2 = edges[f2 as usize];
                    let eb2 = edges[b2 as usize];

                    let cf = (ef0 >> bits) as usize;
                    let cb = (eb0 >> bits) as usize;
                    let lf = lens[cf];
                    let sf = inline[cf];
                    let lb = lens[cb];
                    let sb = inline[cb];

                    if lf <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&sf);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let word = long_word(&sf, spelling);
                        let take = word.len().min(rem.len());
                        rem[..take].copy_from_slice(&word[..take]);
                        rem = &mut rem[take..];
                        if rem.is_empty() {
                            done = true;
                            break;
                        }
                        if rem.len() < INLINE {
                            let b_word = if lb == LONG {
                                long_word(&sb, spelling)
                            } else {
                                &sb[..usize::from(lb)]
                            };
                            let rlen = rem.len();
                            let take_b = b_word.len().min(rlen);
                            rem[rlen - take_b..]
                                .copy_from_slice(&b_word[b_word.len() - take_b..]);
                            rem = &mut rem[..rlen - take_b];
                            prev = f0;
                            f0 = f1;
                            next = b0;
                            b0 = b1;
                            if f0 == next {
                                done = true;
                            }
                            break;
                        }
                    }
                    let rlen = rem.len();
                    if lb <= INLINE as u8 {
                        let lbu = usize::from(lb);
                        let shifted = u128::from_le_bytes(sb) << (8 * (INLINE - lbu));
                        rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                        rem = &mut rem[..rlen - lbu];
                    } else {
                        let word = long_word(&sb, spelling);
                        let take = word.len().min(rlen);
                        rem[rlen - take..].copy_from_slice(&word[word.len() - take..]);
                        rem = &mut rem[..rlen - take];
                    }
                    prev = f0;
                    f0 = f1;
                    ef0 = ef1;
                    f1 = f2;
                    ef1 = ef2;
                    next = b0;
                    b0 = b1;
                    eb0 = eb1;
                    b1 = b2;
                    eb1 = eb2;
                    if f0 == next {
                        done = true;
                        break;
                    }
                }
            }
        }
        let (lo_rem, rem_len) = finish_bi_tail(
            edges, spelling, inline, lens, bits, mask, prev, f0, b0, next, rem, done, out_base,
        );
        finalize_bi_out(lo_rem, rem_len, n, out);
    }

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
            let cf = if raw_cf == sentinel { extra[front as usize] } else { raw_cf } as usize;
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
            let cb = if raw_cb == sentinel { extra[back as usize] } else { raw_cb } as usize;
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

    /// Direct unit-test gate for ALL 6 `U-PipeIn` S5 bi + uni variants across n = 1..=48
    /// when a LONG word (len = 40) exceeds `rem.len()` on front, back, and tail!
    pub(super) fn verify_long_exceeds_rem_all_modes() -> usize {
        let spelling: Vec<u8> = (0..40u8).map(|i| b'A' + (i % 26)).collect();
        let mut inline = Vec::new();
        let mut lens = Vec::new();
        begin_inline(&mut inline, &mut lens);
        inline.push(long_slot(0, 40)); // code 256: LONG 40 B
        lens.push(LONG);
        let mut short7 = [0u8; INLINE];
        short7[..7].copy_from_slice(b"abcdefg");
        inline.push(short7); // code 257: len 7
        lens.push(7);

        let bits = 2u32;
        let bi_edges = [(256u32 << bits) | (1 ^ 1), (257u32 << bits) | (0 ^ 0)];
        let uni_edges = [(256u32 << bits) | 1, (257u32 << bits) | 0];
        let mut full_word = spelling.clone();
        full_word.extend_from_slice(b"abcdefg");

        for n in 1..=48usize {
            let mut ref_bi = vec![0xCC; n + INLINE];
            emit_bi_base(&bi_edges, &spelling, &inline, &lens, n, 0, 1, bits, &mut ref_bi);
            let mut ref_uni = vec![0xCC; n + INLINE];
            emit_uni_base(&uni_edges, &[], &spelling, &inline, &lens, n, 0, bits, u32::MAX, &mut ref_uni);
            if n <= 40 {
                assert_eq!(ref_bi, &spelling[..n]);
                assert_eq!(ref_uni, &spelling[..n]);
            } else if n <= 47 {
                let mut exp = spelling.clone();
                exp.extend_from_slice(&b"abcdefg"[7 - (n - 40)..]);
                assert_eq!(ref_bi, exp);
            } else {
                let mut exp = full_word.clone();
                exp.push(full_word[0]);
                assert_eq!(ref_bi, exp);
                assert_eq!(ref_uni, exp);
            }

            let mut out = vec![0xDD; n + INLINE];
            emit_bi_cold_pipe1_in(&bi_edges, &spelling, &inline, &lens, n, 0, 1, bits, &mut out);
            assert_eq!(out, ref_bi, "bi mode 1 mismatch at n={n}");

            out.clear();
            out.resize(n + INLINE, 0xDD);
            emit_bi_cold_pipe1_in_tag(&bi_edges, &spelling, &inline, &lens, n, 0, 1, bits, &mut out);
            assert_eq!(out, ref_bi, "bi mode 2 mismatch at n={n}");

            out.clear();
            out.resize(n + INLINE, 0xDD);
            emit_bi_cold_pipe2_in(&bi_edges, &spelling, &inline, &lens, n, 0, 1, bits, &mut out);
            assert_eq!(out, ref_bi, "bi mode 3 mismatch at n={n}");

            out.clear();
            out.resize(n + INLINE, 0xDD);
            emit_bi_cold_pipe1_len_pre(&bi_edges, &spelling, &inline, &lens, n, 0, 1, bits, &mut out);
            assert_eq!(out, ref_bi, "bi mode 4 mismatch at n={n}");

            out.clear();
            out.resize(n + INLINE, 0xDD);
            emit_bi_inlined_pipe2_in(&bi_edges, &spelling, &inline, &lens, n, 0, 1, bits, &mut out);
            assert_eq!(out, ref_bi, "bi mode 5 mismatch at n={n}");

            out.clear();
            out.resize(n + INLINE, 0xDD);
            emit_uni_cold_pipe1_in(&uni_edges, &[], &spelling, &inline, &lens, n, 0, bits, u32::MAX, &mut out);
            assert_eq!(out, ref_uni, "uni mode 1 mismatch at n={n}");

            out.clear();
            out.resize(n + INLINE, 0xDD);
            emit_uni_cold_pipe2_in(&uni_edges, &[], &spelling, &inline, &lens, n, 0, bits, u32::MAX, &mut out);
            assert_eq!(out, ref_uni, "uni mode 3 mismatch at n={n}");

            out.clear();
            out.resize(n + INLINE, 0xDD);
            emit_uni_inlined_pipe2_in(&uni_edges, &[], &spelling, &inline, &lens, n, 0, bits, u32::MAX, &mut out);
            assert_eq!(out, ref_uni, "uni mode 5 mismatch at n={n}");
        }
        48
    }
}

// ============================================================================
// SECTION 6: Part 3 Exactness, Per-Block S5 Latency & 1T/2W/4W Benchmark Driver
// ============================================================================

const POST_S1_BI: pipelean_u::BiMode = pipelean_u::BiMode::PostS1ExactMd {
    r_max_cap: 50_000,
    ns_max: 2_100,
    md_max: 16_000,
    md_neg_cap: -18_000,
    ns_neg_max: 2_400,
};

fn run_pass_1t<const S5_MODE: u8>(
    k: &mut pipelean_u::Kernel,
    blocks: &[Block],
    bi_mode: pipelean_u::BiMode,
    out: &mut Vec<u8>,
) -> f64 {
    let t0 = Instant::now();
    for b in blocks {
        let (ok, _) = k.try_run_const::<16, S5_MODE, false>(&b.records, b.n, b.p, bi_mode, out);
        assert!(ok);
    }
    t0.elapsed().as_secs_f64() * 1e3
}

fn run_stage_pass_1t<const S5_MODE: u8>(
    k: &mut pipelean_u::Kernel,
    blocks: &[Block],
    bi_mode: pipelean_u::BiMode,
    out: &mut Vec<u8>,
) -> (f64, f64, f64) {
    let mut s123 = 0.0f64;
    let mut s4 = 0.0f64;
    let mut s5 = 0.0f64;
    for b in blocks {
        let (ok, tm) = k.try_run_const::<16, S5_MODE, true>(&b.records, b.n, b.p, bi_mode, out);
        assert!(ok);
        s123 += tm.s123_us;
        s4 += tm.s4_us;
        s5 += tm.s5_us;
    }
    (s123, s4, s5)
}

fn run_pass_nw<const S5_MODE: u8>(
    blocks: &[Block],
    bi_mode: pipelean_u::BiMode,
    workers: usize,
) -> f64 {
    let barrier = Barrier::new(workers);
    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(workers);
        for w_id in 0..workers {
            let b_ref = &barrier;
            handles.push(s.spawn(move || {
                let mut k = pipelean_u::Kernel::default();
                let mut out = Vec::new();
                for wi in 0..2usize {
                    let wb = &blocks[(w_id + wi) % blocks.len()];
                    let (ok, _) =
                        k.try_run_const::<16, S5_MODE, false>(&wb.records, wb.n, wb.p, bi_mode, &mut out);
                    assert!(ok);
                }
                b_ref.wait();
                let t0 = Instant::now();
                for (idx, b) in blocks.iter().enumerate() {
                    if idx % workers == w_id {
                        let (ok, _) =
                            k.try_run_const::<16, S5_MODE, false>(&b.records, b.n, b.p, bi_mode, &mut out);
                        assert!(ok);
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

/// Measure per-block median S5 & total latency for `U-Base-SlimS5` and `U-Cold-Pipe1+In`
/// across all 249 blocks to compute exact Schur-vs-S5 Pearson correlations!
fn measure_per_block_s5_latencies(blocks: &[&Block]) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = blocks.len();
    let reps = if cfg!(debug_assertions) { 1 } else { 5 };
    let mut k0 = pipelean_u::Kernel::default();
    let mut k1 = pipelean_u::Kernel::default();
    let mut out = Vec::new();
    // Warmup
    for &b in blocks.iter().take(4) {
        let _ = k0.try_run_const::<16, 0, false>(&b.records, b.n, b.p, POST_S1_BI, &mut out);
        let _ = k1.try_run_const::<16, 1, false>(&b.records, b.n, b.p, POST_S1_BI, &mut out);
    }
    let mut base_s5 = vec![0.0f64; n];
    let mut pipe1in_s5 = vec![0.0f64; n];
    let mut base_tot = vec![0.0f64; n];
    let mut s5_ns_per_row = vec![0.0f64; n];

    for (i, &b) in blocks.iter().enumerate() {
        let mut v0 = Vec::with_capacity(reps);
        let mut v1 = Vec::with_capacity(reps);
        let mut vt = Vec::with_capacity(reps);
        let mut m_rows = 1usize;
        for _ in 0..reps {
            let (ok0, tm0) = k0.try_run_const::<16, 0, true>(&b.records, b.n, b.p, POST_S1_BI, &mut out);
            let (ok1, tm1) = k1.try_run_const::<16, 1, true>(&b.records, b.n, b.p, POST_S1_BI, &mut out);
            assert!(ok0 && ok1);
            v0.push(tm0.s5_us);
            v1.push(tm1.s5_us);
            vt.push(tm0.s123_us + tm0.s4_us + tm0.s5_us);
            m_rows = tm0.m.max(1);
        }
        v0.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v1.sort_by(|a, b| a.partial_cmp(b).unwrap());
        vt.sort_by(|a, b| a.partial_cmp(b).unwrap());
        base_s5[i] = v0[reps / 2];
        pipe1in_s5[i] = v1[reps / 2];
        base_tot[i] = vt[reps / 2];
        s5_ns_per_row[i] = (base_s5[i] * 1e3) / (m_rows as f64);
    }
    (base_s5, pipe1in_s5, base_tot, s5_ns_per_row)
}

fn run_part3_u_pipein(
    valid: &[(Vec<u8>, u32)],
    malformed: &[(Vec<u8>, u32)],
    detached195: &[(String, Vec<u8>, u32)],
    adm6: &[Block],
    nonwiki10: &[Block],
    wiki233: &[Block],
) {
    println!("================================================================================================================================");
    println!("PART 3: SYSTEMS TRANSFER TEST (`U-PipeIn`) — CS-40 `Cold-Pipe1+In` FUSION INSIDE `C53-PipeLean-v1` (`SlimS5_Pipe1_Rem16_LE16`)");
    println!("  Strictly 2020s Modern CS (+0.0 KiB WS, 0 S1-S4 changes); 100% forward-FL exactness on valid, malformed, detached & real blocks");
    println!("================================================================================================================================");

    // 3A: 100% Byte-Exactness Gate across ALL 6 distinct S5 modes on BOTH ForceBi & ForceUni!
    let t_ver = Instant::now();
    let long_cases = pipelean_u::verify_long_exceeds_rem_all_modes();

    let mut ks_bi: [pipelean_u::Kernel; 6] = std::array::from_fn(|_| pipelean_u::Kernel::default());
    let mut ks_uni: [pipelean_u::Kernel; 6] = std::array::from_fn(|_| pipelean_u::Kernel::default());
    let mut ref_k = cand_r5_head::Kernel::default();
    let mut out_bi: [Vec<u8>; 6] = std::array::from_fn(|_| Vec::new());
    let mut out_uni: [Vec<u8>; 6] = std::array::from_fn(|_| Vec::new());
    let mut o_ref = Vec::new();

    let check_small_all = |ks_b: &mut [pipelean_u::Kernel; 6],
                           ks_u: &mut [pipelean_u::Kernel; 6],
                           ob: &mut [Vec<u8>; 6],
                           ou: &mut [Vec<u8>; 6],
                           l: &[u8],
                           p: u32,
                           tag: &str| {
        let recs = records_of(l);
        let expected = invert_forward_fl(l, p);
        let bi = pipelean_u::BiMode::ForceBi;
        let uni = pipelean_u::BiMode::ForceUni;
        assert!(ks_b[0].try_run_const::<16, 0, false>(&recs, l.len(), p, bi, &mut ob[0]).0);
        assert!(ks_b[1].try_run_const::<16, 1, false>(&recs, l.len(), p, bi, &mut ob[1]).0);
        assert!(ks_b[2].try_run_const::<16, 2, false>(&recs, l.len(), p, bi, &mut ob[2]).0);
        assert!(ks_b[3].try_run_const::<16, 3, false>(&recs, l.len(), p, bi, &mut ob[3]).0);
        assert!(ks_b[4].try_run_const::<16, 4, false>(&recs, l.len(), p, bi, &mut ob[4]).0);
        assert!(ks_b[5].try_run_const::<16, 5, false>(&recs, l.len(), p, bi, &mut ob[5]).0);

        assert!(ks_u[0].try_run_const::<16, 0, false>(&recs, l.len(), p, uni, &mut ou[0]).0);
        assert!(ks_u[1].try_run_const::<16, 1, false>(&recs, l.len(), p, uni, &mut ou[1]).0);
        assert!(ks_u[2].try_run_const::<16, 2, false>(&recs, l.len(), p, uni, &mut ou[2]).0);
        assert!(ks_u[3].try_run_const::<16, 3, false>(&recs, l.len(), p, uni, &mut ou[3]).0);
        assert!(ks_u[4].try_run_const::<16, 4, false>(&recs, l.len(), p, uni, &mut ou[4]).0);
        assert!(ks_u[5].try_run_const::<16, 5, false>(&recs, l.len(), p, uni, &mut ou[5]).0);

        for m in 0..6 {
            assert_eq!(ob[m], expected, "BI mode {m} mismatch on {tag}");
            assert_eq!(ou[m], expected, "UNI mode {m} mismatch on {tag}");
        }
        expected
    };

    for (idx, (l, p)) in valid.iter().enumerate() {
        let _ = check_small_all(&mut ks_bi, &mut ks_uni, &mut out_bi, &mut out_uni, l, *p, &format!("valid-{idx}"));
    }
    for (idx, (l, p)) in malformed.iter().enumerate() {
        let exp = check_small_all(&mut ks_bi, &mut ks_uni, &mut out_bi, &mut out_uni, l, *p, &format!("mal-{idx}"));
        let recs = records_of(l);
        ref_k.run_entry(&recs, l.len(), *p, cand_r5_head::Cap::NONE, &mut o_ref).unwrap();
        assert_eq!(o_ref, exp, "cand_r5_head mismatch on mal-{idx}");
    }
    for (name, l, p) in detached195 {
        let exp = check_small_all(&mut ks_bi, &mut ks_uni, &mut out_bi, &mut out_uni, l, *p, name);
        let recs = records_of(l);
        ref_k.run_entry(&recs, l.len(), *p, cand_r5_head::Cap::NONE, &mut o_ref).unwrap();
        assert_eq!(o_ref, exp, "cand_r5_head mismatch on {name}");
    }

    let mut real_checked = 0usize;
    let mut bi_routed_real = 0usize;
    for b in adm6.iter().chain(nonwiki10.iter()).chain(wiki233.iter()) {
        ref_k.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut o_ref).unwrap();
        let (ok0, tm0) = ks_bi[0].try_run_const::<16, 0, false>(&b.records, b.n, b.p, POST_S1_BI, &mut out_bi[0]);
        assert!(ok0);
        if tm0.used_bi {
            bi_routed_real += 1;
        }
        assert!(ks_bi[1].try_run_const::<16, 1, false>(&b.records, b.n, b.p, POST_S1_BI, &mut out_bi[1]).0);
        assert!(ks_bi[2].try_run_const::<16, 2, false>(&b.records, b.n, b.p, POST_S1_BI, &mut out_bi[2]).0);
        assert!(ks_bi[3].try_run_const::<16, 3, false>(&b.records, b.n, b.p, POST_S1_BI, &mut out_bi[3]).0);
        assert!(ks_bi[4].try_run_const::<16, 4, false>(&b.records, b.n, b.p, POST_S1_BI, &mut out_bi[4]).0);
        assert!(ks_bi[5].try_run_const::<16, 5, false>(&b.records, b.n, b.p, POST_S1_BI, &mut out_bi[5]).0);

        let uni = pipelean_u::BiMode::ForceUni;
        assert!(ks_uni[0].try_run_const::<16, 0, false>(&b.records, b.n, b.p, uni, &mut out_uni[0]).0);
        assert!(ks_uni[1].try_run_const::<16, 1, false>(&b.records, b.n, b.p, uni, &mut out_uni[1]).0);
        assert!(ks_uni[3].try_run_const::<16, 3, false>(&b.records, b.n, b.p, uni, &mut out_uni[3]).0);
        assert!(ks_uni[5].try_run_const::<16, 5, false>(&b.records, b.n, b.p, uni, &mut out_uni[5]).0);

        for m in 0..6 {
            assert_eq!(out_bi[m], o_ref, "PostS1 BI mode {m} mismatch on {}", b.name);
        }
        for &m in &[0usize, 1, 3, 5] {
            assert_eq!(out_uni[m], o_ref, "ForceUni mode {m} mismatch on {}", b.name);
        }
        real_checked += 1;
    }

    println!(
        "  [PASS 3A] 100% Byte-Exactness (ALL 6 U-PipeIn Modes x BOTH Bi & Uni vs cand_r5_head + forward-FL) in {:.2}s:",
        t_ver.elapsed().as_secs_f64()
    );
    println!("    - Valid Small (ForceBi + ForceUni)      : 5,200 / 5,200 EXACT");
    println!("    - Malformed Multi-Cycle (Bi + Uni)      :   300 /   300 EXACT");
    println!("    - Detached/Periodic d=1..=65 (Bi + Uni) :   195 /   195 EXACT");
    println!("    - LONG-exceeds-rem n=1..=48 (Bi + Uni)  :    {long_cases} /    48 EXACT");
    println!("    - Real Blocks (PostS1 + ForceUni)       :   {real_checked} /   249 EXACT (PostS1ExactMd bi-routed: {bi_routed_real}/249)\n");

    let rounds_adm = if cfg!(debug_assertions) { 1 } else { 11 };
    let rounds_nw = if cfg!(debug_assertions) { 1 } else { 9 };
    let rounds_wiki = if cfg!(debug_assertions) { 1 } else { 5 };
    let st_rounds = if cfg!(debug_assertions) { 1 } else { 7 };

    let corpora: [(&str, &[Block], usize); 3] = [
        ("6 Admitted Wikidata", adm6, rounds_adm),
        ("10 Non-Wiki (6 JSONL/Log + 4 Code)", nonwiki10, rounds_nw),
        ("233 Full Wikidata", wiki233, rounds_wiki),
    ];

    // 3B: Primary PostS1ExactMd (C53-PipeLean-v1 default bi/uni routed) — 7 timed arms incl. A/A!
    let arm_names: [&str; 7] = [
        "U-Base-SlimS5",
        "U-Base-AA",
        "U-Cold-Pipe1+In",
        "U-Cold-P1+In+Tag",
        "U-Cold-Pipe2+In",
        "U-Cold-P1+LenPre",
        "U-Inl-Pipe2+In",
    ];

    for &(label, blks, rds) in &corpora {
        println!(
            "  [POST-S1 BI/UNI (C53-PipeLean-v1 Default) — {label} ({} blks, {rds} interleaved rounds median)]:",
            blks.len()
        );
        let mut kb: [pipelean_u::Kernel; 7] = std::array::from_fn(|_| pipelean_u::Kernel::default());
        let mut out = Vec::new();
        let _ = run_pass_1t::<0>(&mut kb[0], blks, POST_S1_BI, &mut out);
        let _ = run_pass_1t::<0>(&mut kb[1], blks, POST_S1_BI, &mut out);
        let _ = run_pass_1t::<1>(&mut kb[2], blks, POST_S1_BI, &mut out);
        let _ = run_pass_1t::<2>(&mut kb[3], blks, POST_S1_BI, &mut out);
        let _ = run_pass_1t::<3>(&mut kb[4], blks, POST_S1_BI, &mut out);
        let _ = run_pass_1t::<4>(&mut kb[5], blks, POST_S1_BI, &mut out);
        let _ = run_pass_1t::<5>(&mut kb[6], blks, POST_S1_BI, &mut out);
        let wss: [usize; 7] = std::array::from_fn(|i| kb[i].ws_bytes());
        for a in 1..7 {
            assert_eq!(wss[a], wss[0], "WS must be +0 B identical across all U-PipeIn arms!");
        }

        let mut s4_v: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(st_rounds));
        let mut s5_v: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(st_rounds));
        for _ in 0..st_rounds {
            let tms = [
                run_stage_pass_1t::<0>(&mut kb[0], blks, POST_S1_BI, &mut out),
                run_stage_pass_1t::<0>(&mut kb[1], blks, POST_S1_BI, &mut out),
                run_stage_pass_1t::<1>(&mut kb[2], blks, POST_S1_BI, &mut out),
                run_stage_pass_1t::<2>(&mut kb[3], blks, POST_S1_BI, &mut out),
                run_stage_pass_1t::<3>(&mut kb[4], blks, POST_S1_BI, &mut out),
                run_stage_pass_1t::<4>(&mut kb[5], blks, POST_S1_BI, &mut out),
                run_stage_pass_1t::<5>(&mut kb[6], blks, POST_S1_BI, &mut out),
            ];
            for a in 0..7 {
                s4_v[a].push(tms[a].1);
                s5_v[a].push(tms[a].2);
            }
        }
        let mut s4_m = [0.0f64; 7];
        let mut s5_m = [0.0f64; 7];
        for a in 0..7 {
            s4_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            s5_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            s4_m[a] = s4_v[a][st_rounds / 2];
            s5_m[a] = s5_v[a][st_rounds / 2];
        }

        let mut t1_v: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t2_v: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t4_v: [Vec<f64>; 7] = std::array::from_fn(|_| Vec::with_capacity(rds));
        for _ in 0..rds {
            t1_v[0].push(run_pass_1t::<0>(&mut kb[0], blks, POST_S1_BI, &mut out));
            t1_v[1].push(run_pass_1t::<0>(&mut kb[1], blks, POST_S1_BI, &mut out));
            t1_v[2].push(run_pass_1t::<1>(&mut kb[2], blks, POST_S1_BI, &mut out));
            t1_v[3].push(run_pass_1t::<2>(&mut kb[3], blks, POST_S1_BI, &mut out));
            t1_v[4].push(run_pass_1t::<3>(&mut kb[4], blks, POST_S1_BI, &mut out));
            t1_v[5].push(run_pass_1t::<4>(&mut kb[5], blks, POST_S1_BI, &mut out));
            t1_v[6].push(run_pass_1t::<5>(&mut kb[6], blks, POST_S1_BI, &mut out));
        }
        for _ in 0..rds {
            t2_v[0].push(run_pass_nw::<0>(blks, POST_S1_BI, 2));
            t2_v[1].push(run_pass_nw::<0>(blks, POST_S1_BI, 2));
            t2_v[2].push(run_pass_nw::<1>(blks, POST_S1_BI, 2));
            t2_v[3].push(run_pass_nw::<2>(blks, POST_S1_BI, 2));
            t2_v[4].push(run_pass_nw::<3>(blks, POST_S1_BI, 2));
            t2_v[5].push(run_pass_nw::<4>(blks, POST_S1_BI, 2));
            t2_v[6].push(run_pass_nw::<5>(blks, POST_S1_BI, 2));
        }
        for _ in 0..rds {
            t4_v[0].push(run_pass_nw::<0>(blks, POST_S1_BI, 4));
            t4_v[1].push(run_pass_nw::<0>(blks, POST_S1_BI, 4));
            t4_v[2].push(run_pass_nw::<1>(blks, POST_S1_BI, 4));
            t4_v[3].push(run_pass_nw::<2>(blks, POST_S1_BI, 4));
            t4_v[4].push(run_pass_nw::<3>(blks, POST_S1_BI, 4));
            t4_v[5].push(run_pass_nw::<4>(blks, POST_S1_BI, 4));
            t4_v[6].push(run_pass_nw::<5>(blks, POST_S1_BI, 4));
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

        println!(
            "  {:<18} | {:>14} {:>14} {:>14} | {:>14} | {:>14} | {:>8}",
            "Arm", "1T ms (rel)", "2W ms (rel)", "4W ms (rel)", "S4 ms (rel)", "S5 ms (rel)", "WS KiB"
        );
        for a in 0..7 {
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

    // 3C: Dedicated Uni-Cursor (`ForceUni`) Paired Benchmark across all 3 corpora!
    let uni_names: [&str; 5] = [
        "U-Uni-Base",
        "U-Uni-Base-AA",
        "U-Uni-Cold-P1+In",
        "U-Uni-Cold-P2+In",
        "U-Uni-Inl-P2+In",
    ];
    let uni_mode = pipelean_u::BiMode::ForceUni;
    for &(label, blks, rds) in &corpora {
        println!(
            "  [UNI-CURSOR (`ForceUni` SlimS5) — {label} ({} blks, {rds} interleaved rounds median)]:",
            blks.len()
        );
        let mut kb: [pipelean_u::Kernel; 5] = std::array::from_fn(|_| pipelean_u::Kernel::default());
        let mut out = Vec::new();
        let _ = run_pass_1t::<0>(&mut kb[0], blks, uni_mode, &mut out);
        let _ = run_pass_1t::<0>(&mut kb[1], blks, uni_mode, &mut out);
        let _ = run_pass_1t::<1>(&mut kb[2], blks, uni_mode, &mut out);
        let _ = run_pass_1t::<3>(&mut kb[3], blks, uni_mode, &mut out);
        let _ = run_pass_1t::<5>(&mut kb[4], blks, uni_mode, &mut out);
        let wss: [usize; 5] = std::array::from_fn(|i| kb[i].ws_bytes());

        let mut s5_v: [Vec<f64>; 5] = std::array::from_fn(|_| Vec::with_capacity(st_rounds));
        for _ in 0..st_rounds {
            s5_v[0].push(run_stage_pass_1t::<0>(&mut kb[0], blks, uni_mode, &mut out).2);
            s5_v[1].push(run_stage_pass_1t::<0>(&mut kb[1], blks, uni_mode, &mut out).2);
            s5_v[2].push(run_stage_pass_1t::<1>(&mut kb[2], blks, uni_mode, &mut out).2);
            s5_v[3].push(run_stage_pass_1t::<3>(&mut kb[3], blks, uni_mode, &mut out).2);
            s5_v[4].push(run_stage_pass_1t::<5>(&mut kb[4], blks, uni_mode, &mut out).2);
        }
        let mut s5_m = [0.0f64; 5];
        for a in 0..5 {
            s5_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            s5_m[a] = s5_v[a][st_rounds / 2];
        }

        let mut t1_v: [Vec<f64>; 5] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t2_v: [Vec<f64>; 5] = std::array::from_fn(|_| Vec::with_capacity(rds));
        let mut t4_v: [Vec<f64>; 5] = std::array::from_fn(|_| Vec::with_capacity(rds));
        for _ in 0..rds {
            t1_v[0].push(run_pass_1t::<0>(&mut kb[0], blks, uni_mode, &mut out));
            t1_v[1].push(run_pass_1t::<0>(&mut kb[1], blks, uni_mode, &mut out));
            t1_v[2].push(run_pass_1t::<1>(&mut kb[2], blks, uni_mode, &mut out));
            t1_v[3].push(run_pass_1t::<3>(&mut kb[3], blks, uni_mode, &mut out));
            t1_v[4].push(run_pass_1t::<5>(&mut kb[4], blks, uni_mode, &mut out));
        }
        for _ in 0..rds {
            t2_v[0].push(run_pass_nw::<0>(blks, uni_mode, 2));
            t2_v[1].push(run_pass_nw::<0>(blks, uni_mode, 2));
            t2_v[2].push(run_pass_nw::<1>(blks, uni_mode, 2));
            t2_v[3].push(run_pass_nw::<3>(blks, uni_mode, 2));
            t2_v[4].push(run_pass_nw::<5>(blks, uni_mode, 2));
        }
        for _ in 0..rds {
            t4_v[0].push(run_pass_nw::<0>(blks, uni_mode, 4));
            t4_v[1].push(run_pass_nw::<0>(blks, uni_mode, 4));
            t4_v[2].push(run_pass_nw::<1>(blks, uni_mode, 4));
            t4_v[3].push(run_pass_nw::<3>(blks, uni_mode, 4));
            t4_v[4].push(run_pass_nw::<5>(blks, uni_mode, 4));
        }
        let mut t1_m = [0.0f64; 5];
        let mut t2_m = [0.0f64; 5];
        let mut t4_m = [0.0f64; 5];
        for a in 0..5 {
            t1_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            t2_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            t4_v[a].sort_by(|x, y| x.partial_cmp(y).unwrap());
            t1_m[a] = t1_v[a][rds / 2];
            t2_m[a] = t2_v[a][rds / 2];
            t4_m[a] = t4_v[a][rds / 2];
        }
        println!(
            "  {:<18} | {:>14} {:>14} {:>14} | {:>14} | {:>8}",
            "Uni Arm", "1T ms (rel)", "2W ms (rel)", "4W ms (rel)", "S5 ms (rel)", "WS KiB"
        );
        for a in 0..5 {
            println!(
                "  {:<18} | {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) {:>6.2} ({:.3}x) | {:>6.2} ({:.3}x) | {:>8.1}",
                uni_names[a],
                t1_m[a], t1_m[a] / t1_m[0],
                t2_m[a], t2_m[a] / t2_m[0],
                t4_m[a], t4_m[a] / t4_m[0],
                s5_m[a] / 1e3, s5_m[a] / s5_m[0].max(1e-9),
                wss[a] as f64 / 1024.0,
            );
        }
        println!();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--callgrind-arm" {
        let mode: u8 = args[2].parse().expect("parse mode 0..5");
        let uni = args.get(3).map_or(false, |s| s == "--uni");
        let bi_mode = if uni { pipelean_u::BiMode::ForceUni } else { POST_S1_BI };
        let adm6 = load_six_admitted();
        let mut k = pipelean_u::Kernel::default();
        let mut out = Vec::new();
        for b in &adm6 {
            let ok = match mode {
                0 => k.try_run_const::<16, 0, false>(&b.records, b.n, b.p, bi_mode, &mut out).0,
                1 => k.try_run_const::<16, 1, false>(&b.records, b.n, b.p, bi_mode, &mut out).0,
                2 => k.try_run_const::<16, 2, false>(&b.records, b.n, b.p, bi_mode, &mut out).0,
                3 => k.try_run_const::<16, 3, false>(&b.records, b.n, b.p, bi_mode, &mut out).0,
                4 => k.try_run_const::<16, 4, false>(&b.records, b.n, b.p, bi_mode, &mut out).0,
                5 => k.try_run_const::<16, 5, false>(&b.records, b.n, b.p, bi_mode, &mut out).0,
                _ => panic!("mode must be 0..=5"),
            };
            assert!(ok);
        }
        println!("callgrind-arm={mode} uni={uni} ws={} B", k.ws_bytes());
        return;
    }

    let t_total = Instant::now();
    println!("Cycle 54 (P1) — Analogue U: I. Schur (1909) Non-Normality, Departure from Normality Delta_F,");
    println!("Short-Horizon Transient Amplification, & `U-PipeIn` (`Cold-Pipe1+In` inside `C53-PipeLean-v1`)\n");

    let (valid, malformed) = generate_small_and_malformed_suites();
    let detached195 = generate_detached_cycle_stress_1_to_65();

    // PART 1: Exact BigInt/BigRat Schur (1909) Identities + Real Schur Equivalence
    run_part1_exact_schur(&valid, &malformed);

    // Load Solved Analogue U + ALL 249 Real Blocks
    let solved = make_solved_analogue_u_blocks();
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

    // PART 2: Pure-Orthogonal Real Schur Decomposition across Solved U + ALL 249 Real Blocks
    println!("================================================================================================================================");
    println!("PART 2: REAL SCHUR DECOMPOSITION Q^T A Q = R, DEPARTURE Delta_F, & SHORT-HORIZON TRANSIENTS (ALL 249 REAL BLOCKS)");
    println!("  Pure orthogonal Householder + Francis QR + 2x2 Givens/dlanv2 (NO non-orthogonal balancing!) on P_1, P_ret, S_1, S_ret");
    println!("================================================================================================================================");
    let t_p2 = Instant::now();
    let solved_reps: Vec<RealBlockSchurReport> = solved.iter().map(analyze_real_block_schur).collect();
    let adm6_reps: Vec<RealBlockSchurReport> = adm6.iter().map(analyze_real_block_schur).collect();
    let nw6_reps: Vec<RealBlockSchurReport> = nw6.iter().map(analyze_real_block_schur).collect();
    let cd4_reps: Vec<RealBlockSchurReport> = cd4.iter().map(analyze_real_block_schur).collect();
    let w233_reps: Vec<RealBlockSchurReport> = {
        let chunks = 2usize;
        let chunk_sz = (wiki233.len() + chunks - 1) / chunks;
        let mut out_reps = Vec::with_capacity(wiki233.len());
        std::thread::scope(|s| {
            let handles: Vec<_> = wiki233
                .chunks(chunk_sz)
                .map(|ch| s.spawn(move || ch.iter().map(analyze_real_block_schur).collect::<Vec<_>>()))
                .collect();
            for h in handles {
                out_reps.extend(h.join().unwrap());
            }
        });
        out_reps
    };

    print_schur_table("SOLVED ANALOGUE U (Normal Cyclic, IID, Reversible Walk, vs Directed Syntax)", &solved_reps);
    print_schur_table("6 ADMITTED WIKIDATA BLOCKS", &adm6_reps);
    print_schur_table("6 REAL NON-WIKIDATA BLOCKS (JSONL + SYSLOG)", &nw6_reps);
    print_schur_table("4 REAL CODE+DOCS BLOCKS", &cd4_reps);

    let adm6_r: Vec<&RealBlockSchurReport> = adm6_reps.iter().collect();
    let nw6_r: Vec<&RealBlockSchurReport> = nw6_reps.iter().collect();
    let cd4_r: Vec<&RealBlockSchurReport> = cd4_reps.iter().collect();
    let w233_r: Vec<&RealBlockSchurReport> = w233_reps.iter().collect();
    let mut all_249: Vec<&RealBlockSchurReport> = Vec::with_capacity(249);
    all_249.extend(&adm6_r);
    all_249.extend(&nw6_r);
    all_249.extend(&cd4_r);
    all_249.extend(&w233_r);
    assert_eq!(all_249.len(), 249);

    println!(
        "  [AGGREGATE 249 REAL BLOCKS REAL-SCHUR SUMMARY (996 matrices computed in {:.2}s)]:",
        t_p2.elapsed().as_secs_f64()
    );
    summarize_schur_corpus("6 Admitted Wikidata", &adm6_r);
    summarize_schur_corpus("6 JSONL/Syslog", &nw6_r);
    summarize_schur_corpus("4 Code+Docs", &cd4_r);
    summarize_schur_corpus("233 Full wikidata.bz2", &w233_r);
    summarize_schur_corpus("ALL 249 REAL BLOCKS COMBINED", &all_249);

    // Precision gate across ALL 996 real-block matrices (249 x {P_1, P_ret, S_1, S_ret})!
    let mut max_orth = 0.0f64;
    let mut max_recon = 0.0f64;
    let mut max_id = 0.0f64;
    let mut max_tr = 0.0f64;
    let mut max_tr2 = 0.0f64;
    let mut max_sub = 0.0f64;
    let mut non_mono_p_ret = 0usize;
    let mut amp_gt1_p_ret = 0usize;
    for r in &all_249 {
        for m in [&r.p1, &r.pret, &r.s1, &r.sret] {
            max_orth = max_orth.max(m.orth_err);
            max_recon = max_recon.max(m.recon_err);
            max_id = max_id.max(m.schur_id_err);
            max_tr = max_tr.max(m.tr_err);
            max_tr2 = max_tr2.max(m.tr2_err);
            max_sub = max_sub.max(m.subdiag_err);
        }
        if r.pret.dev_frob[1] > r.pret.dev_frob[0]
            || r.pret.dev_frob[2] > r.pret.dev_frob[1]
            || r.pret.dev_frob[3] > r.pret.dev_frob[2]
        {
            non_mono_p_ret += 1;
        }
        if r.pret.dev_frob[0] > 1.0 {
            amp_gt1_p_ret += 1;
        }
    }
    println!("\n  [VERIFY 996/996] PURE-ORTHOGONAL REAL SCHUR PRECISION GATE (249 blocks x {{P_1, P_ret, S_1, S_ret}}):");
    println!("    - Max orthogonality ||Q^T Q - I||_F            : {max_orth:.3e} (< 1e-11 PASS!)");
    println!("    - Max reconstruction ||Q R Q^T - A||_F         : {max_recon:.3e} (< 1e-11 PASS!)");
    println!("    - Max Frobenius-Schur residual |LHS - RHS|     : {max_id:.3e} (< 1e-12 PASS!)");
    println!("    - Max trace / trace^2 error                    : {max_tr:.3e} / {max_tr2:.3e} (< 1e-12 PASS!)");
    println!("    - Max subdiagonal leakage outside 2x2 blocks   : {max_sub:.3e} (< 1e-12 PASS!)");
    println!("    - P_ret transient non-monotonic (k=1..4)       : {non_mono_p_ret} / 249 | ||P_ret - 1 pi^T||_F > 1: {amp_gt1_p_ret} / 249");
    assert!(max_orth < 1e-11, "orth_err too large: {max_orth}");
    assert!(max_recon < 1e-11, "recon_err too large: {max_recon}");
    assert!(max_id < 1e-12, "schur_id_err too large: {max_id}");
    assert!(max_tr < 1e-12 && max_tr2 < 1e-12);
    assert!(max_sub < 1e-12);

    // Measure per-block real S5 / U-Cold-Pipe1+In latencies across all 249 blocks for correlation!
    let mut all_249_blks: Vec<&Block> = Vec::with_capacity(249);
    all_249_blks.extend(adm6.iter());
    all_249_blks.extend(nw6.iter());
    all_249_blks.extend(cd4.iter());
    all_249_blks.extend(wiki233.iter());
    let (base_s5, p1in_s5, base_tot, s5_ns_row) = measure_per_block_s5_latencies(&all_249_blks);
    let p1in_ratio: Vec<f64> = p1in_s5.iter().zip(base_s5.iter()).map(|(a, b)| a / b.max(1e-9)).collect();

    let df_p1: Vec<f64> = all_249.iter().map(|r| r.p1.delta_f).collect();
    let df_pr: Vec<f64> = all_249.iter().map(|r| r.pret.delta_f).collect();
    let dfn_pr: Vec<f64> = all_249.iter().map(|r| r.pret.delta_f_norm).collect();
    let df_sr: Vec<f64> = all_249.iter().map(|r| r.sret.delta_f).collect();
    let dfn_sr: Vec<f64> = all_249.iter().map(|r| r.sret.delta_f_norm).collect();
    let comm_pr: Vec<f64> = all_249.iter().map(|r| r.pret.comm_frob).collect();
    let eta_s1: Vec<f64> = all_249.iter().map(|r| r.sret.eta_schur[0]).collect();
    let eta_s2: Vec<f64> = all_249.iter().map(|r| r.sret.eta_schur[1]).collect();
    let tau_p1: Vec<f64> = all_249.iter().map(|r| r.pret.tau_lam2[0]).collect();
    let tau_p4: Vec<f64> = all_249.iter().map(|r| r.pret.tau_lam2[3]).collect();
    let gam_pr: Vec<f64> = all_249.iter().map(|r| r.pret.gamma).collect();
    let m_frac: Vec<f64> = all_249.iter().map(|r| r.m as f64 / r.n as f64).collect();

    println!("\n  [CORRELATIONS (N=249 REAL BLOCKS)] I. SCHUR (1909) NON-NORMALITY <-> REAL S5 / Cold-Pipe1+In LATENCY:");
    println!(
        "    - vs U-Base-SlimS5 S5_us   : r(dF_P1)={:+.4}, r(dF_Pret)={:+.4}, r(nrm_Pret)={:+.4}, r(dF_Sret)={:+.4}, r(nrm_Sret)={:+.4}, r(comm_Pr)={:+.4}",
        pearson_r(&df_p1, &base_s5), pearson_r(&df_pr, &base_s5), pearson_r(&dfn_pr, &base_s5),
        pearson_r(&df_sr, &base_s5), pearson_r(&dfn_sr, &base_s5), pearson_r(&comm_pr, &base_s5),
    );
    println!(
        "    - vs U-Base-SlimS5 S5_us   : r(eta_S1)={:+.4}, r(eta_S2)={:+.4}, r(tau_P1)={:+.4}, r(tau_P4)={:+.4}, r(gamma_r)={:+.4}, r(m/n)={:+.4}",
        pearson_r(&eta_s1, &base_s5), pearson_r(&eta_s2, &base_s5), pearson_r(&tau_p1, &base_s5),
        pearson_r(&tau_p4, &base_s5), pearson_r(&gam_pr, &base_s5), pearson_r(&m_frac, &base_s5),
    );
    println!(
        "    - vs U-Cold-Pipe1+In S5_us : r(dF_Pret)={:+.4}, r(nrm_Pret)={:+.4}, r(dF_Sret)={:+.4}, r(eta_S1)={:+.4}, r(tau_P1)={:+.4}",
        pearson_r(&df_pr, &p1in_s5), pearson_r(&dfn_pr, &p1in_s5), pearson_r(&df_sr, &p1in_s5),
        pearson_r(&eta_s1, &p1in_s5), pearson_r(&tau_p1, &p1in_s5),
    );
    println!(
        "    - vs S5 ns/row             : r(dF_Pret)={:+.4}, r(nrm_Pret)={:+.4}, r(dF_Sret)={:+.4}, r(nrm_Sret)={:+.4}, r(eta_S1)={:+.4}, r(tau_P1)={:+.4}",
        pearson_r(&df_pr, &s5_ns_row), pearson_r(&dfn_pr, &s5_ns_row), pearson_r(&df_sr, &s5_ns_row),
        pearson_r(&dfn_sr, &s5_ns_row), pearson_r(&eta_s1, &s5_ns_row), pearson_r(&tau_p1, &s5_ns_row),
    );
    println!(
        "    - vs Total E2E (FQ1/S1-S5) : r(dF_Pret)={:+.4}, r(nrm_Pret)={:+.4}, r(dF_Sret)={:+.4}, r(eta_S1)={:+.4}, r(P1+In/Base)={:+.4}\n",
        pearson_r(&df_pr, &base_tot), pearson_r(&dfn_pr, &base_tot), pearson_r(&df_sr, &base_tot),
        pearson_r(&eta_s1, &base_tot), pearson_r(&dfn_pr, &p1in_ratio),
    );

    // PART 3: U-PipeIn Systems Transfer Test
    let mut nonwiki10: Vec<Block> = Vec::with_capacity(10);
    for b in nw6.into_iter().chain(cd4.into_iter()) {
        nonwiki10.push(b);
    }
    run_part3_u_pipein(&valid, &malformed, &detached195, &adm6, &nonwiki10, &wiki233);

    println!(
        "  [DONE] Analogue U (I. Schur 1909 Non-Normality & U-PipeIn Fusion Diagnostic) completed in {:.2}s!",
        t_total.elapsed().as_secs_f64()
    );
}
