#![forbid(unsafe_code)]

//! Cycle 50 P2 — Analogue N, Theorem N1:
//! A. A. Markov (1879, 1880) Indefinite Binary Quadratic Forms, Markov Tree & Lagrange Spectrum,
//! Christoffel–Ostrowski–Lucas–Sylvester Depth Rigidity, and 2-Band Induced Spectrum on 249 Real Blocks.
//!
//! Primary Sources:
//!   - A. A. Markov, "Sur les formes quadratiques binaires indéfinies",
//!     Mathematische Annalen 15 (1879), 381–406;
//!   - A. A. Markov, "Sur les formes quadratiques binaires indéfinies (Second mémoire)",
//!     Mathematische Annalen 17 (1880), 379–399;
//!   Bridging directly to:
//!   - CA-01: Euclid (c. 300 BCE), Elements VII/X (continued fractions)
//!   - CA-02: E. B. Christoffel (1875), Ann. Mat. Pura Appl. (2) 6, 148–152 (balanced words)
//!   - CA-03: A. Ostrowski (1922), Abh. Math. Sem. Univ. Hamburg 1, 77–98 (numeration)
//!   - CA-34: J. J. Sylvester (1851, 1857) & A. Cayley (1889) (Matrix-Tree, Chain A leaf!)
//!   - CA-35: É. Lucas (1891), Théorie des nombres (Cassini & S2b subtraction reference depth)
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`

use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::Instant;

const LIMIT: u32 = 16;

// ============================================================================
// 1. SL_2(Z) Matrices, Continued Fractions, Christoffel Words & Markov Forms
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Mat2 {
    a: i128,
    b: i128,
    c: i128,
    d: i128,
}

impl Mat2 {
    const ID: Mat2 = Mat2 { a: 1, b: 0, c: 0, d: 1 };
    /// Markov generator A = [[1,1],[1,0]]^2 = [[2,1],[1,1]] (tr = 3*1, m12 = 1, CF period 1,1).
    const MARKOV_A: Mat2 = Mat2 { a: 2, b: 1, c: 1, d: 1 };
    /// Markov generator B = [[2,1],[1,0]]^2 = [[5,2],[2,1]] (tr = 3*2, m12 = 2, CF period 2,2).
    const MARKOV_B: Mat2 = Mat2 { a: 5, b: 2, c: 2, d: 1 };

    #[inline]
    fn mul(self, rhs: Mat2) -> Mat2 {
        Mat2 {
            a: self.a * rhs.a + self.b * rhs.c,
            b: self.a * rhs.b + self.b * rhs.d,
            c: self.c * rhs.a + self.d * rhs.c,
            d: self.c * rhs.b + self.d * rhs.d,
        }
    }

    #[inline]
    fn checked_mul(self, rhs: Mat2) -> Option<Mat2> {
        Some(Mat2 {
            a: self.a.checked_mul(rhs.a)?.checked_add(self.b.checked_mul(rhs.c)?)?,
            b: self.a.checked_mul(rhs.b)?.checked_add(self.b.checked_mul(rhs.d)?)?,
            c: self.c.checked_mul(rhs.a)?.checked_add(self.d.checked_mul(rhs.c)?)?,
            d: self.c.checked_mul(rhs.b)?.checked_add(self.d.checked_mul(rhs.d)?)?,
        })
    }

    #[inline]
    fn det(self) -> i128 {
        self.a * self.d - self.b * self.c
    }

    #[inline]
    fn trace(self) -> i128 {
        self.a + self.d
    }

    #[inline]
    fn inv_sl2(self) -> Mat2 {
        debug_assert_eq!(self.det(), 1);
        Mat2 { a: self.d, b: -self.b, c: -self.c, d: self.a }
    }

    #[inline]
    fn cf_step(q: i128) -> Mat2 {
        Mat2 { a: q, b: 1, c: 1, d: 0 }
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

/// Exact canonical continued fraction `[a_0; a_1, ..., a_ell]` for rational `p / q` (`p >= 1, q >= 1`).
/// Canonical finite CF convention: when `p != q`, last partial quotient `>= 2`;
/// for `p == q == 1`, returns `[1]`.
/// Notice `sum(a_i)` equals the exact number of subtractive Euclidean steps from `(p, q)` to `(1, 0)` or `(0, 1)`!
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

/// Exact sum of partial quotients of `p/q` (equals subtractive Euclid / FQ2 cut-tower depth!).
fn cf_sum_and_max(p: u64, q: u64) -> (Vec<u64>, u64, u64) {
    let cf = continued_fraction(p, q);
    let sum: u64 = cf.iter().sum();
    let mx: u64 = cf.iter().copied().max().unwrap_or(0);
    (cf, sum, mx)
}

/// Exact Ostrowski (CA-03) denominator sequence `q_k` and maximal Ostrowski digits for `p/q`.
fn ostrowski_denominators(cf: &[u64]) -> Vec<u64> {
    let mut qs = Vec::with_capacity(cf.len() + 1);
    let mut q_prev2 = 0u64;
    let mut q_prev1 = 1u64;
    qs.push(1);
    for &a in cf.iter().skip(1) {
        let q_next = a.saturating_mul(q_prev1).saturating_add(q_prev2);
        qs.push(q_next);
        q_prev2 = q_prev1;
        q_prev1 = q_next;
    }
    qs
}

/// Exact lower Christoffel word `W_{p,q}` of length `p + q` over `{b'a', b'b'}` (p copies of 'a', q copies of 'b'),
/// constructed directly from Christoffel's (1875) discrete lattice-path definition (CA-02):
/// step `k = 1..=p+q`: move East ('a') if `floor(k*q / (p+q)) == floor((k-1)*q / (p+q))`?
/// Standard lower Christoffel word for slope `q/p` (with `p` horizontal `'a'` steps and `q` vertical `'b'` steps):
/// starts with `'a'`, ends with `'b'` when `p >= 1, q >= 1`:
/// trace integer residues `r_0 = 0`; at each step if `r + q < p + q`? Specifically:
/// maintain `y = 0, x = 0`: take `'a'` (x += 1) if `q * (x + 1) <= p * (y + 1)`? Wait:
/// Let's use the exact integer arithmetic lower-path condition (Berstel–Lauve–Reutenauer–Saliola / Christoffel 1875):
/// Lattice points `(x, y)` from `(0, 0)` to `(p, q)` staying on or BELOW the line `q*x - p*y = 0`, i.e., `q*x >= p*y`:
/// - At `(0, 0)`: moving North to `(0, 1)` gives `0 < p`, illegal; must move East (`'a'`) to `(1, 0)` (`q >= 0`).
/// - At `(x, y)` (`x + y < p + q`): can move North (`'b'`) iff `y < q` AND `q*x >= p*(y + 1)`?
///   Wait: check `(p=2, q=1)`:
///   (0,0): `0 >= 2` false -> 'a' -> (1,0)
///   (1,0): `1 >= 2` false -> 'a' -> (2,0)
///   (2,0): `2 >= 2` true  -> 'b' -> (2,1). Word = `"aab"`!
///   Check `(p=1, q=2)`:
///   (0,0): `0 >= 1` false -> 'a' -> (1,0)
///   (1,0): `2 >= 1` true  -> 'b' -> (1,1)
///   (1,1): `2 >= 2` true  -> 'b' -> (1,2). Word = `"abb"`!
///   Check `(p=3, q=2)`:
///   (0,0)->'a'(1,0): `2>=3` F -> 'a'(2,0): `4>=3` T -> 'b'(2,1): `4>=6` F -> 'a'(3,1): `6>=6` T -> 'b'(3,2).
///   Word = `"aabab"`! Exact!
///   Check `(p=2, q=3)`:
///   (0,0)->'a'(1,0): `3>=2` T -> 'b'(1,1): `3>=4` F -> 'a'(2,1): `6>=4` T -> 'b'(2,2): `6>=6` T -> 'b'(2,3).
///   Word = `"ababb"`! Exact!
fn lower_christoffel_word(p: usize, q: usize) -> Vec<u8> {
    if p == 0 {
        return vec![b'b'; q];
    }
    if q == 0 {
        return vec![b'a'; p];
    }
    assert_eq!(gcd_u64(p as u64, q as u64), 1);
    let mut w = Vec::with_capacity(p + q);
    let (mut x, mut y) = (0u64, 0u64);
    let (pu, qu) = (p as u64, q as u64);
    while (x as usize) + (y as usize) < p + q {
        if y < qu && qu * x >= pu * (y + 1) {
            w.push(b'b');
            y += 1;
        } else {
            w.push(b'a');
            x += 1;
        }
    }
    assert_eq!(x, pu);
    assert_eq!(y, qu);
    w
}

/// Evaluate Cohn–Markov SL_2(Z) matrix `M(W)` directly letter-by-letter along word `W`.
fn cohn_markov_matrix(word: &[u8]) -> Option<Mat2> {
    let mut m = Mat2::ID;
    for &ch in word {
        let g = match ch {
            b'a' => Mat2::MARKOV_A,
            b'b' => Mat2::MARKOV_B,
            _ => unreachable!(),
        };
        m = m.checked_mul(g)?;
    }
    Some(m)
}

/// Direct O(p+q) BWT of a small word (used to certify `L(W_{p,q}) == b^q a^p`).
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

/// Compute the positive quadratic irrational root `[overline{seq}] > 1` of a purely periodic
/// continued fraction `seq = [c_0, c_1, ..., c_{k-1}]` (`c_i >= 1`) via its SL_2/GL_2 matrix product.
fn periodic_cf_value(seq: &[u64]) -> f64 {
    let mut m = Mat2::ID;
    for &q in seq {
        m = m.mul(Mat2::cf_step(i128::from(q)));
    }
    // Fixed point alpha > 0 satisfies m.c * alpha^2 + (m.d - m.a) * alpha - m.b = 0.
    let a = m.c as f64;
    let b = (m.d - m.a) as f64;
    let c = -(m.b as f64);
    let disc = b * b - 4.0 * a * c;
    (-b + disc.sqrt()) / (2.0 * a)
}

/// Directly compute Markov's (1879, 1880) Lagrange constant
/// `M(theta) = max_{0 <= j < L} ( [overline{c_j, c_{j+1}, ...}] + [0; overline{c_{j-1}, c_{j-2}, ...}] )`
/// from the periodic continued fraction `psi(W_{p,q})` (`a -> 1,1`, `b -> 2,2`).
fn markov_lagrange_from_periodic_cf(word: &[u8]) -> (f64, Vec<u64>) {
    let mut period = Vec::with_capacity(word.len() * 2);
    for &ch in word {
        match ch {
            b'a' => {
                period.push(1);
                period.push(1);
            }
            b'b' => {
                period.push(2);
                period.push(2);
            }
            _ => unreachable!(),
        }
    }
    let len = period.len();
    let mut best = 0.0f64;
    let mut fwd = vec![0u64; len];
    let mut rev = vec![0u64; len];
    for j in 0..len {
        for t in 0..len {
            fwd[t] = period[(j + t) % len];
            rev[t] = period[(j + len - 1 - t) % len];
        }
        let alpha = periodic_cf_value(&fwd);
        let beta = 1.0 / periodic_cf_value(&rev);
        let sum = alpha + beta;
        if sum > best {
            best = sum;
        }
    }
    (best, period)
}

/// Markov (1879–80) canonical binary quadratic form `f_z(x, y) = z*x^2 + (3z - 2u)*x*y + (v - 3u)*y^2`
/// where `u` solves `u^2 + 1 == 0 (mod z)` (`0 <= u < z`, derived from triple `(x, y, z)` via
/// `u * x ≡ ±y (mod z)`), and `v = (u^2 + 1) / z`.
fn markov_canonical_form(x: u128, y: u128, z: u128) -> (i128, i128, i128, u128, u128) {
    if z == 1 {
        // f_1(x,y) = x^2 - xy - y^2 (disc = 5, inf |f_1| = 1)
        return (1, -1, -1, 0, 1);
    }
    if z == 2 {
        // f_2(x,y) = 2x^2 + 4xy - 2y^2 -> primitive x^2 - 2xy - y^2 (with z-scaling 2x^2 - 4xy - 2y^2, disc = 32 = 9*4-4)
        return (2, 4, -2, 1, 1);
    }
    // Extended Euclidean inverse of x mod z to compute u = (y * x^{-1}) mod z
    let inv_x = mod_inv_u128(x % z, z).expect("Markov triple entries are pairwise coprime");
    let mut u = (y % z) * inv_x % z;
    if 2 * u > z {
        u = z - u;
    }
    assert_eq!((u * u + 1) % z, 0, "Markov congruence u^2 + 1 == 0 (mod z) failed");
    let v = (u * u + 1) / z;
    let a = z as i128;
    let b = (3 * z - 2 * u) as i128;
    let c = (v as i128) - 3 * (u as i128);
    let disc = b * b - 4 * a * c;
    assert_eq!(disc, (9 * z * z - 4) as i128, "Markov discriminant 9z^2 - 4 mismatch");
    (a, b, c, u, v)
}

fn mod_inv_u128(a: u128, m: u128) -> Option<u128> {
    let (mut old_r, mut r) = (a as i128, m as i128);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        let nr = old_r - q * r;
        let ns = old_s - q * s;
        old_r = r;
        r = nr;
        old_s = s;
        s = ns;
    }
    if old_r != 1 {
        return None;
    }
    let res = old_s.rem_euclid(m as i128);
    Some(res as u128)
}

#[derive(Clone, Debug)]
struct MarkovNode {
    x: u128,
    y: u128,
    z: u128,
    p: usize,
    q: usize,
    tree_depth: usize,
}

/// Generate ALL Markov triples `(x, y, z)` with `z <= limit` in exact bijection with
/// Stern–Brocot / Farey Christoffel slopes `(p, q)`.
fn generate_markov_tree(limit: u128) -> Vec<MarkovNode> {
    let mut out = Vec::new();
    // Singular roots: (1, 1, 1) at (1, 0) and (1, 1, 2) at (0, 1)
    if limit >= 1 {
        out.push(MarkovNode { x: 1, y: 1, z: 1, p: 1, q: 0, tree_depth: 0 });
    }
    if limit >= 2 {
        out.push(MarkovNode { x: 1, y: 1, z: 2, p: 0, q: 1, tree_depth: 0 });
    }
    fn rec(
        xl: u128,
        pl: usize,
        ql: usize,
        yr: u128,
        pr: usize,
        qr: usize,
        z: u128,
        depth: usize,
        limit: u128,
        out: &mut Vec<MarkovNode>,
    ) {
        if z > limit {
            return;
        }
        let pm = pl + pr;
        let qm = ql + qr;
        out.push(MarkovNode { x: xl, y: yr, z, p: pm, q: qm, tree_depth: depth });
        // Left child: replace yr by z' = 3*xl*z - yr, interval [(pl,ql), (pm,qm)]
        if let Some(zl) = xl.checked_mul(3).and_then(|v| v.checked_mul(z)).and_then(|v| v.checked_sub(yr)) {
            if zl <= limit {
                rec(xl, pl, ql, z, pm, qm, zl, depth + 1, limit, out);
            }
        }
        // Right child: replace xl by z' = 3*z*yr - xl, interval [(pm,qm), (pr,qr)]
        if let Some(zr) = z.checked_mul(3).and_then(|v| v.checked_mul(yr)).and_then(|v| v.checked_sub(xl)) {
            if zr <= limit {
                rec(z, pm, qm, yr, pr, qr, zr, depth + 1, limit, out);
            }
        }
    }
    if limit >= 5 {
        rec(1, 1, 0, 2, 0, 1, 5, 1, limit, &mut out);
    }
    out.sort_by_key(|n| (n.z, n.p, n.q));
    out
}

// ============================================================================
// 2. Exact CA-34 Sylvester–Bareiss Determinant, FQ2 Cut Tower & CA-35 S2b Depth
// ============================================================================

/// Fraction-free Sylvester (1851, 1857) / Bareiss exact integer determinant for small matrices
/// (exact `i128` with `rem == 0` asserted at every pivot step!).
fn sylvester_bareiss_det_i128(mut mat: Vec<Vec<i128>>) -> i128 {
    let n = mat.len();
    if n == 0 {
        return 1;
    }
    let mut prev_piv = 1i128;
    let mut sign = 1i128;
    for k in 0..n {
        let mut piv_row = k;
        while piv_row < n && mat[piv_row][k] == 0 {
            piv_row += 1;
        }
        if piv_row == n {
            return 0;
        }
        if piv_row != k {
            mat.swap(k, piv_row);
            sign = -sign;
        }
        let piv = mat[k][k];
        for i in (k + 1)..n {
            for j in (k + 1)..n {
                let num = mat[i][j] * piv - mat[i][k] * mat[k][j];
                assert_eq!(num % prev_piv, 0, "Sylvester-Bareiss exact divisibility violated");
                mat[i][j] = num / prev_piv;
            }
            mat[i][k] = 0;
        }
        prev_piv = piv;
    }
    sign * mat[n - 1][n - 1]
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

    /// Compute exact CA-34 Sylvester-Bareiss Matrix-Tree determinants of `G_band(L_k)`:
    /// returns `(tau_support, tau_weighted, z_edge_support)`.
    fn band_matrix_tree_dets(bands: &[BandPiece], m: usize) -> (i128, i128, i128) {
        let b = bands.len();
        if b <= 1 {
            return (1, 1, 1);
        }
        let mut cod_order: Vec<u32> = (0..b as u32).collect();
        cod_order.sort_unstable_by_key(|&idx| bands[idx as usize].c_start);
        let mut wt = vec![vec![0i128; b]; b];
        let (mut ci, mut di) = (0usize, 0usize);
        let mut y = 0u32;
        while ci < b && di < b {
            let v = cod_order[ci] as usize;
            let u = di;
            let c_end = bands[v].c_start + bands[v].width;
            let d_end = bands[u].d_start + bands[u].width;
            let y1 = c_end.min(d_end);
            if y1 > y {
                wt[v][u] += i128::from(y1 - y);
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
        // Build reduced (n-1)x(n-1) directed Laplacians deleting root = 0
        let dim = b - 1;
        let mut lap_sup = vec![vec![0i128; dim]; dim];
        let mut lap_wt = vec![vec![0i128; dim]; dim];
        let mut z_fac_sup = 1i128;
        for u in 0..b {
            let mut out_sup_deg = 0i128;
            for v in 0..b {
                if wt[u][v] > 0 {
                    out_sup_deg += 1;
                }
            }
            if out_sup_deg > 1 {
                for f in 2..out_sup_deg {
                    z_fac_sup *= f;
                }
            }
            if u == 0 {
                continue;
            }
            let r = u - 1;
            let mut diag_sup = 0i128;
            let mut diag_wt = 0i128;
            for v in 0..b {
                if v != u && wt[u][v] > 0 {
                    diag_sup += 1;
                    diag_wt += wt[u][v];
                    if v > 0 {
                        lap_sup[r][v - 1] -= 1;
                        lap_wt[r][v - 1] -= wt[u][v];
                    }
                }
            }
            lap_sup[r][r] = diag_sup;
            lap_wt[r][r] = diag_wt;
        }
        let tau_sup = sylvester_bareiss_det_i128(lap_sup);
        let tau_wt = sylvester_bareiss_det_i128(lap_wt);
        (tau_sup, tau_wt, tau_sup * z_fac_sup)
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
}

/// Exact Lucas Fibonacci sequence `U_k` and Lucas subtraction reference depth `h_Luc(ell)` (CA-35).
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

fn lucas_opt_depth(ell: u32, fib_u: &[u64]) -> u32 {
    let target = u64::from(ell.max(1));
    let mut h = 0usize;
    while h + 2 < fib_u.len() && fib_u[h + 2] < target {
        h += 1;
    }
    h as u32
}

/// Exact instrumented Open<16> S2b Depth + 2-Band Spiral Spectrum analyzer.
#[derive(Default, Clone, Debug)]
struct S2bSpectrumReport {
    n: u64,
    m: u64,
    ns: usize,
    thick_rows: u64,
    // Dynamic S2b elimination breakdown (plain subtractive vs spiral-contracted)
    plain_steps: u64,
    plain_joins: u64,
    plain_max_depth: u32,
    plain_max_len: u32,
    spiral_steps: u64,
    spiral_joins: u64,
    spiral_fires_q2: u64,
    spiral_fires_q3p: u64,
    spiral_steps_saved_q2: u64,
    spiral_steps_saved_q3p: u64,
    spiral_max_depth: u32,
    spiral_max_len: u32,
    // Row-peel mass decomposition during S2b (sums to n!)
    rows_self_loop: u64,       // x == y (1-band self-loop)
    rows_2band_hi_pq: u64,     // 2-band adjacent with q >= 3 (ABOVE Markov threshold!)
    rows_2band_markov: u64,    // 2-band adjacent with q in {1, 2} (INSIDE Markov threshold!)
    rows_multiband_mix: u64,   // x, y not 2-band adjacent (multi-band / GUID mixing)
    // Also count steps per class in spiral mode
    steps_self_loop: u64,
    steps_2band_hi_pq: u64,
    steps_2band_markov: u64,
    steps_multiband_mix: u64,
    max_spiral_q: u32,
    // Thick-strip overlap connected components in S2a
    thick_comps_total: usize,
    thick_comps_size1: usize,
    thick_comps_size2: usize,
    thick_comps_size2_rows: u64,
    thick_comps_2letter: usize,
    thick_comps_2letter_rows: u64,
}

#[derive(Clone, Copy)]
struct RuleMeta {
    len: u32,
    true_depth: u32,
}

struct InstElim {
    prev0: Vec<u32>,
    next0: Vec<u32>,
    prev1: Vec<u32>,
    next1: Vec<u32>,
    width: Vec<u32>,
    word: Vec<u32>,
    nodes: u32,
    rules: Vec<RuleMeta>,
    steps: u64,
    joins: u64,
    spiral_fires_q2: u64,
    spiral_fires_q3p: u64,
    spiral_steps_saved_q2: u64,
    spiral_steps_saved_q3p: u64,
    rows_self_loop: u64,
    rows_2band_hi_pq: u64,
    rows_2band_markov: u64,
    rows_multiband_mix: u64,
    steps_self_loop: u64,
    steps_2band_hi_pq: u64,
    steps_2band_markov: u64,
    steps_multiband_mix: u64,
    max_spiral_q: u32,
}

impl InstElim {
    fn new(cap_nodes: usize) -> Self {
        let mut rules = Vec::with_capacity(cap_nodes * 2 + 257);
        rules.push(RuleMeta { len: 0, true_depth: 0 });
        for _ in 0..256 {
            rules.push(RuleMeta { len: 1, true_depth: 0 });
        }
        Self {
            prev0: vec![0; cap_nodes],
            next0: vec![0; cap_nodes],
            prev1: vec![0; cap_nodes],
            next1: vec![0; cap_nodes],
            width: vec![0; cap_nodes],
            word: vec![0; cap_nodes],
            nodes: 0,
            rules,
            steps: 0,
            joins: 0,
            spiral_fires_q2: 0,
            spiral_fires_q3p: 0,
            spiral_steps_saved_q2: 0,
            spiral_steps_saved_q3p: 0,
            rows_self_loop: 0,
            rows_2band_hi_pq: 0,
            rows_2band_markov: 0,
            rows_multiband_mix: 0,
            steps_self_loop: 0,
            steps_2band_hi_pq: 0,
            steps_2band_markov: 0,
            steps_multiband_mix: 0,
            max_spiral_q: 0,
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
            true_depth: 1 + ml.true_depth.max(mr.true_depth),
        });
        self.joins += 1;
        id
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

    fn eliminate(&mut self, n: u32, spirals: bool) {
        let mut left = n;
        while left > 0 {
            let x = self.prev0[0];
            let y = self.prev1[0];
            let aw = self.width[x as usize];
            let bw = self.width[y as usize];
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                self.steps += 1;
                self.steps_self_loop += 1;
                self.rows_self_loop += u64::from(aw);
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            // Check 2-band adjacency (prev1[y] == x when aw > bw, or prev0[x] == y when bw > aw,
            // or mutual adjacency when aw == bw)
            let adj_x_dom = aw > bw && self.prev1[y as usize] == x;
            let adj_y_dom = aw < bw && self.prev0[x as usize] == y;
            let adj_eq = aw == bw && (self.prev1[y as usize] == x || self.prev0[x as usize] == y);

            if spirals && (adj_x_dom || adj_y_dom) {
                let (big, small) = if adj_x_dom { (aw, bw) } else { (bw, aw) };
                let q = (big - 1) / small;
                if q >= 2 {
                    self.max_spiral_q = self.max_spiral_q.max(q);
                    let rows = u64::from(q) * u64::from(small);
                    assert!(q * small < big && q * small <= left);
                    if q >= 3 {
                        self.spiral_fires_q3p += 1;
                        self.spiral_steps_saved_q3p += u64::from(q - 1);
                        self.steps_2band_hi_pq += 1;
                        self.rows_2band_hi_pq += rows;
                    } else {
                        self.spiral_fires_q2 += 1;
                        self.spiral_steps_saved_q2 += u64::from(q - 1);
                        self.steps_2band_markov += 1;
                        self.rows_2band_markov += rows;
                    }
                    if adj_x_dom {
                        let head = self.power(self.word[x as usize], q);
                        let wd = self.join(head, self.word[y as usize]);
                        self.word[y as usize] = wd;
                        self.width[x as usize] = aw - q * bw;
                    } else {
                        let tail = self.power(self.word[y as usize], q);
                        let wd = self.join(self.word[x as usize], tail);
                        self.word[x as usize] = wd;
                        self.width[y as usize] = bw - q * aw;
                    }
                    left -= q * small;
                    self.steps += 1;
                    continue;
                }
            }

            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            self.steps += 1;
            if adj_x_dom || adj_y_dom || adj_eq {
                self.steps_2band_markov += 1;
                self.rows_2band_markov += u64::from(w);
            } else {
                self.steps_multiband_mix += 1;
                self.rows_multiband_mix += u64::from(w);
            }

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

    fn max_surv_depth_and_len(&self) -> (u32, u32) {
        let mut max_d = 0u32;
        let mut max_l = 0u32;
        let mut cur = self.next0[0];
        while cur != 0 {
            let wd = self.word[cur as usize];
            if wd != 0 {
                let meta = self.rules[wd as usize];
                max_d = max_d.max(meta.true_depth);
                max_l = max_l.max(meta.len);
            }
            cur = self.next0[cur as usize];
        }
        (max_d, max_l)
    }
}

fn build_open16_strips(l: &[u8], p: u32, limit: u32) -> (u32, [u32; 256], [u32; 256], Vec<(u32, u32, u32, u8)>) {
    let n = l.len();
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
    while pos < n {
        let c = l[pos];
        let mut end = pos + 1;
        while end < n && l[end] == c {
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
    (m, first, thick, strips)
}

fn run_s2b_spectrum(l: &[u8], p: u32, limit: u32) -> S2bSpectrumReport {
    let n = l.len() as u32;
    let (m, _first, thick, strips) = build_open16_strips(l, p, limit);
    let ns = strips.len();
    let thick_rows: u64 = strips.iter().map(|s| u64::from(s.2)).sum();

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

    // Thick-strip overlap connected components in S2a (domain-image strip overlap)
    let mut dsu = Dsu::new(ns);
    {
        let (mut di, mut ci) = (0usize, 0usize);
        while di < ns && ci < ns {
            let s_dom = strips[di];
            let s_cod_idx = image_order[ci] as usize;
            let s_cod = strips[s_cod_idx];
            let d_lo = s_dom.0;
            let d_hi = d_lo + s_dom.2;
            let c_lo = s_cod.1;
            let c_hi = c_lo + s_cod.2;
            if d_lo < c_hi && c_lo < d_hi {
                dsu.union(di as u32, s_cod_idx as u32);
            }
            if d_hi <= c_hi {
                di += 1;
            } else {
                ci += 1;
            }
        }
    }
    let mut comp_sz: HashMap<u32, (usize, u64, HashSet<u8>)> = HashMap::new();
    for (i, &(_, _, w, ch)) in strips.iter().enumerate() {
        let r = dsu.find(i as u32);
        let ent = comp_sz.entry(r).or_insert_with(|| (0, 0, HashSet::new()));
        ent.0 += 1;
        ent.1 += u64::from(w);
        ent.2.insert(ch);
    }
    let mut comps_1 = 0usize;
    let mut comps_2 = 0usize;
    let mut comps_2_rows = 0u64;
    let mut comps_2let = 0usize;
    let mut comps_2let_rows = 0u64;
    for (_, (cnt, rows, letters)) in &comp_sz {
        if *cnt == 1 {
            comps_1 += 1;
        } else if *cnt == 2 {
            comps_2 += 1;
            comps_2_rows += *rows;
        }
        if letters.len() == 2 {
            comps_2let += 1;
            comps_2let_rows += *rows;
        }
    }

    let build_and_run = |spirals: bool| -> InstElim {
        let most = 3 * ns + 3;
        let mut e = InstElim::new(most);
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
        e.eliminate(n, spirals);
        e
    };

    let ep = build_and_run(false);
    let (plain_max_depth, plain_max_len) = ep.max_surv_depth_and_len();
    let es = build_and_run(true);
    let (spiral_max_depth, spiral_max_len) = es.max_surv_depth_and_len();
    assert_eq!(
        plain_max_len, spiral_max_len,
        "S2b spiral contraction must preserve exact max excursion length"
    );

    assert_eq!(
        es.rows_self_loop + es.rows_2band_hi_pq + es.rows_2band_markov + es.rows_multiband_mix,
        u64::from(n),
        "S2b row-peel partition must sum to n"
    );

    S2bSpectrumReport {
        n: u64::from(n),
        m: u64::from(m),
        ns,
        thick_rows,
        plain_steps: ep.steps,
        plain_joins: ep.joins,
        plain_max_depth,
        plain_max_len,
        spiral_steps: es.steps,
        spiral_joins: es.joins,
        spiral_fires_q2: es.spiral_fires_q2,
        spiral_fires_q3p: es.spiral_fires_q3p,
        spiral_steps_saved_q2: es.spiral_steps_saved_q2,
        spiral_steps_saved_q3p: es.spiral_steps_saved_q3p,
        spiral_max_depth,
        spiral_max_len,
        rows_self_loop: es.rows_self_loop,
        rows_2band_hi_pq: es.rows_2band_hi_pq,
        rows_2band_markov: es.rows_2band_markov,
        rows_multiband_mix: es.rows_multiband_mix,
        steps_self_loop: es.steps_self_loop,
        steps_2band_hi_pq: es.steps_2band_hi_pq,
        steps_2band_markov: es.steps_2band_markov,
        steps_multiband_mix: es.steps_multiband_mix,
        max_spiral_q: es.max_spiral_q,
        thick_comps_total: comp_sz.len(),
        thick_comps_size1: comps_1,
        thick_comps_size2: comps_2,
        thick_comps_size2_rows: comps_2_rows,
        thick_comps_2letter: comps_2let,
        thick_comps_2letter_rows: comps_2let_rows,
    }
}

// ============================================================================
// 3. Exact 1-Pass O(r * sigma) 2-Letter BWT Restriction & Thick-Mass Spectrum
// ============================================================================

#[derive(Default, Clone, Debug)]
struct PairSpectrumBlock {
    sigma: usize,
    total_pairs: u64,
    // Exact 2-letter restriction L|_{a,b} classes across all C(sigma, 2) pairs:
    pairs_r2_christoffel: u64,      // r_2 == 2 and starts with b ('b^q a^p')
    pairs_r2_chr_markov: u64,       // ... and slope p/q has max_ai <= 2 (Markov-zone!)
    pairs_r2_chr_hipq: u64,         // ... and slope p/q has max_ai > 2 (high-PQ!)
    pairs_r2_chr_primitive: u64,    // ... and gcd(c_a, c_b) == 1 (strictly primitive Christoffel!)
    pairs_r2_identity: u64,         // r_2 == 2 and starts with a ('a^p b^q', identity-degenerate)
    pairs_near_bal_3_4: u64,        // 3 <= r_2 <= 4
    pairs_chaotic_5p: u64,          // r_2 >= 5
    sum_defect: u64,                // sum(r_2 - 2)
    sum_cf_len_r2: u64,
    sum_max_ai_r2: u64,
    max_ai_r2_seen: u64,
    r2_markov_mu_sum: f64,
    r2_markov_mu_cnt: u64,
    // Exact Open<16> thick-row mass (N_thick = n - m) partitioned by bigram pair (L[i], F[i]):
    thick_rows_total: u64,
    thick_self_loop: u64,           // L[i] == F[i] (1-letter periodic run a^k, a_1 = inf >> 2)
    thick_r2_chr_hipq: u64,         // L[i] != F[i], r_2 == 2 ('b^q a^p'), max_ai > 2
    thick_r2_chr_markov: u64,       // L[i] != F[i], r_2 == 2 ('b^q a^p'), max_ai <= 2
    thick_r2_identity: u64,         // L[i] != F[i], r_2 == 2 ('a^p b^q')
    thick_near_bal_3_4: u64,        // L[i] != F[i], 3 <= r_2 <= 4
    thick_chaotic_5p: u64,          // L[i] != F[i], r_2 >= 5
    // Same partition on ALL n rows (including thin rows) for contrast:
    thin_rows_total: u64,
    thin_self_loop: u64,
    thin_r2_chr_hipq: u64,
    thin_r2_chr_markov: u64,
    thin_r2_identity: u64,
    thin_near_bal_3_4: u64,
    thin_chaotic_5p: u64,
}

/// Classify a 2-letter pair's reduced slope `(p, q)` (`p = c_a/g, q = c_b/g`) via Euclid CF & Markov constant.
/// Returns `(cf_len, max_ai, mu_opt)` where `mu = sqrt(9 - 4/M_{p,q}^2)` when `M_{p,q}` fits in `i128`
/// (and `3.0` to machine precision when `M_{p,q} > 10^9`, since `4 / M^2 < 4e-18 < eps_f64`!).
fn slope_markov_invariant(ca: u64, cb: u64) -> (u64, u64, f64) {
    let g = gcd_u64(ca, cb);
    let p = ca / g;
    let q = cb / g;
    let (cf, _sum, max_ai) = cf_sum_and_max(p, q);
    // For p + q <= 25, compute exact Markov number via Cohn-Markov SL_2(Z) matrix product;
    // for larger p + q, M_{p,q} >= F_{2(p+q)} > 10^9 so sqrt(9 - 4/M^2) = 3.0 to 1e-16!
    let mu = if p + q <= 25 {
        let w = lower_christoffel_word(p as usize, q as usize);
        if let Some(mat) = cohn_markov_matrix(&w) {
            let m_num = mat.b as f64;
            (9.0 - 4.0 / (m_num * m_num)).sqrt()
        } else {
            3.0
        }
    } else {
        3.0
    };
    (cf.len() as u64, max_ai, mu)
}

fn analyze_2letter_pair_spectrum(l: &[u8], p: u32, limit: usize) -> PairSpectrumBlock {
    let n = l.len();
    let mut cnt = [0u64; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut b2v = [usize::MAX; 256];
    let mut v2b = Vec::new();
    for b in 0..256 {
        if cnt[b] > 0 {
            b2v[b] = v2b.len();
            v2b.push(b as u8);
        }
    }
    let sigma = v2b.len();

    // 1-pass O(r * sigma) exact r_2(L|_{a,b}) and exact first-seen letter for EVERY pair {a, b}!
    let mut alt = vec![0u32; sigma * sigma]; // transitions between u and v in L|_{u,v}
    let mut first_seen = vec![u32::MAX; sigma];
    let mut last_seen = vec![0u32; sigma];
    let mut seen_list: Vec<usize> = Vec::with_capacity(sigma);

    let mut run_id = 0u32;
    let mut pos = 0usize;
    while pos < n {
        let b = l[pos];
        let mut end = pos + 1;
        while end < n && l[end] == b {
            end += 1;
        }
        run_id += 1;
        let u = b2v[usize::from(b)];
        let prev_u = last_seen[u];
        if prev_u == 0 {
            first_seen[u] = run_id;
            for &v in &seen_list {
                // v appeared before u -> first transition v -> u in L|_{u,v}
                let (lo, hi) = if u < v { (u, v) } else { (v, u) };
                alt[lo * sigma + hi] += 1;
            }
            seen_list.push(u);
        } else {
            for &v in &seen_list {
                if v != u && last_seen[v] > prev_u {
                    let (lo, hi) = if u < v { (u, v) } else { (v, u) };
                    alt[lo * sigma + hi] += 1;
                }
            }
        }
        last_seen[u] = run_id;
        pos = end;
    }

    // Classify every distinct pair (u < v):
    // pair_class[u*sigma + v]:
    //   1 = r_2==2 Christoffel ('b^q a^p') high-PQ (max_ai > 2)
    //   2 = r_2==2 Christoffel ('b^q a^p') Markov-zone (max_ai <= 2)
    //   3 = r_2==2 identity-degenerate ('a^p b^q')
    //   4 = near-balanced 3 <= r_2 <= 4
    //   5 = chaotic r_2 >= 5
    let mut pair_class = vec![0u8; sigma * sigma];
    let mut rep = PairSpectrumBlock { sigma, ..Default::default() };

    for u in 0..sigma {
        let cu = cnt[usize::from(v2b[u])];
        for v in (u + 1)..sigma {
            let cv = cnt[usize::from(v2b[v])];
            let r2 = u64::from(alt[u * sigma + v]) + 1;
            debug_assert!(r2 >= 2);
            rep.total_pairs += 1;
            rep.sum_defect += r2 - 2;
            let cls = if r2 == 2 {
                // Since u < v implies v2b[u] < v2b[v], exact Christoffel BWT is b^q a^p = v^{cv} u^{cu},
                // i.e., v (the larger byte) appears BEFORE u (the smaller byte)!
                if first_seen[v] < first_seen[u] {
                    rep.pairs_r2_christoffel += 1;
                    if gcd_u64(cu, cv) == 1 {
                        rep.pairs_r2_chr_primitive += 1;
                    }
                    let (cf_len, max_ai, mu) = slope_markov_invariant(cu, cv);
                    rep.sum_cf_len_r2 += cf_len;
                    rep.sum_max_ai_r2 += max_ai;
                    rep.max_ai_r2_seen = rep.max_ai_r2_seen.max(max_ai);
                    rep.r2_markov_mu_sum += mu;
                    rep.r2_markov_mu_cnt += 1;
                    if max_ai <= 2 {
                        rep.pairs_r2_chr_markov += 1;
                        2u8
                    } else {
                        rep.pairs_r2_chr_hipq += 1;
                        1u8
                    }
                } else {
                    rep.pairs_r2_identity += 1;
                    3u8
                }
            } else if r2 <= 4 {
                rep.pairs_near_bal_3_4 += 1;
                4u8
            } else {
                rep.pairs_chaotic_5p += 1;
                5u8
            };
            pair_class[u * sigma + v] = cls;
            pair_class[v * sigma + u] = cls;
        }
    }

    // Now partition ALL thick rows (width > limit, minus primary index p) AND thin rows
    // by their exact bigram pair (L[i], F[i])!
    let mut is_thick = vec![false; n];
    pos = 0;
    while pos < n {
        let c = l[pos];
        let mut end = pos + 1;
        while end < n && l[end] == c {
            end += 1;
        }
        if end - pos > limit {
            for r in pos..end {
                is_thick[r] = true;
            }
            if (pos..end).contains(&(p as usize)) {
                is_thick[p as usize] = false;
            }
        }
        pos = end;
    }

    let mut f_pos = 0usize;
    for b_f in 0..256 {
        let c = cnt[b_f] as usize;
        if c == 0 {
            continue;
        }
        let vf = b2v[b_f];
        for i in f_pos..(f_pos + c) {
            let vl = b2v[usize::from(l[i])];
            if is_thick[i] {
                rep.thick_rows_total += 1;
                if vl == vf {
                    rep.thick_self_loop += 1;
                } else {
                    match pair_class[vl * sigma + vf] {
                        1 => rep.thick_r2_chr_hipq += 1,
                        2 => rep.thick_r2_chr_markov += 1,
                        3 => rep.thick_r2_identity += 1,
                        4 => rep.thick_near_bal_3_4 += 1,
                        5 => rep.thick_chaotic_5p += 1,
                        _ => unreachable!(),
                    }
                }
            } else {
                rep.thin_rows_total += 1;
                if vl == vf {
                    rep.thin_self_loop += 1;
                } else {
                    match pair_class[vl * sigma + vf] {
                        1 => rep.thin_r2_chr_hipq += 1,
                        2 => rep.thin_r2_chr_markov += 1,
                        3 => rep.thin_r2_identity += 1,
                        4 => rep.thin_near_bal_3_4 += 1,
                        5 => rep.thin_chaotic_5p += 1,
                        _ => unreachable!(),
                    }
                }
            }
        }
        f_pos += c;
    }
    assert_eq!(f_pos, n);
    assert_eq!(rep.thick_rows_total + rep.thin_rows_total, n as u64);
    assert_eq!(
        rep.thick_self_loop
            + rep.thick_r2_chr_hipq
            + rep.thick_r2_chr_markov
            + rep.thick_r2_identity
            + rep.thick_near_bal_3_4
            + rep.thick_chaotic_5p,
        rep.thick_rows_total
    );
    rep
}

// ============================================================================
// 4. Corpora Loaders (All 249 Real Blocks: 6 Adm + 6 JSONL/Log + 4 Code + 233 Wiki)
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

fn resolve_corpus_path(candidates: &[&str]) -> String {
    for &p in candidates {
        if std::path::Path::new(p).exists() {
            return p.to_string();
        }
    }
    panic!("Could not locate corpus among {:?}", candidates);
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

fn pct(a: u64, b: u64) -> f64 {
    if b == 0 {
        0.0
    } else {
        100.0 * a as f64 / b as f64
    }
}

// ============================================================================
// 5. Part 1 & Part 2 Verifiers
// ============================================================================

fn verify_part1_markov_tree() -> Vec<MarkovNode> {
    println!("================================================================================");
    println!("PART 1 — MARKOV (1879, 1880) TREE (M <= 10^5 & M <= 10^9), SL_2(Z) COHN TRACE,");
    println!("         INDEFINITE BINARY QUADRATIC FORMS f_M(x,y) & EXACT LAGRANGE SPECTRUM");
    println!("================================================================================");

    let nodes_1e5 = generate_markov_tree(100_000);
    let nodes_1e9 = generate_markov_tree(1_000_000_000);
    let max_depth_1e5 = nodes_1e5.iter().map(|n| n.tree_depth).max().unwrap_or(0);
    let max_depth_1e9 = nodes_1e9.iter().map(|n| n.tree_depth).max().unwrap_or(0);
    println!(
        "  Generated {} Markov tree nodes with M <= 10^5 (max tree depth={max_depth_1e5}) and {} nodes with M <= 10^9 (max tree depth={max_depth_1e9}).",
        nodes_1e5.len(),
        nodes_1e9.len()
    );

    println!(
        "\n  {:>7} {:>17} {:>7} {:>3} | {:>12} {:>22} {:>12} {:>11}",
        "M=z", "Triple (x,y,z)", "p/q", "n", "tr(M)/3", "Form f_z=(a,b,c)", "mu=sqrt(9-4/M2)", "CF_err"
    );

    let mut max_cf_err = 0.0f64;
    let mut verified_1e5 = 0usize;
    let mut min_form_checked = 0usize;

    for nd in &nodes_1e5 {
        // 1. Exact Markov Diophantine equation x^2 + y^2 + z^2 == 3*x*y*z
        assert_eq!(
            nd.x * nd.x + nd.y * nd.y + nd.z * nd.z,
            3 * nd.x * nd.y * nd.z,
            "Markov equation failed for {:?}",
            nd
        );

        // 2. Construct exact lower Christoffel word W_{p,q} & certify BWT == b^q a^p
        let word = lower_christoffel_word(nd.p, nd.q);
        assert_eq!(word.len(), nd.p + nd.q);
        if nd.p >= 1 && nd.q >= 1 {
            let (bwt_l, _) = bwt_small(&word);
            let mut expected_bwt = vec![b'b'; nd.q];
            expected_bwt.extend(std::iter::repeat_n(b'a', nd.p));
            assert_eq!(bwt_l, expected_bwt, "Christoffel BWT identity failed for ({}, {})", nd.p, nd.q);
        }

        // 3. Exact SL_2(Z) Cohn-Markov matrix product M(W_{p,q})
        let mat = cohn_markov_matrix(&word).expect("i128 matrix fit for M <= 10^5");
        assert_eq!(mat.det(), 1, "det(M(W)) != 1");
        assert_eq!(mat.trace(), 3 * (nd.z as i128), "tr(M(W)) != 3*M");
        assert_eq!(mat.b, nd.z as i128, "M(W)_12 != M");

        // 4. Exact Markov canonical quadratic form f_z(u, v) = a*u^2 + b*uv + c*v^2
        let (fa, fb, fc, _u, _v) = markov_canonical_form(nd.x, nd.y, nd.z);
        assert_eq!(fb * fb - 4 * fa * fc, (9 * nd.z * nd.z - 4) as i128);

        // Verify inf_{(u,v) != (0,0)} |f_z(u, v)| == z on [-12..=12]^2 for all M <= 1000
        if nd.z <= 1000 {
            let mut min_abs = i128::MAX;
            for uu in -12i128..=12 {
                for vv in -12i128..=12 {
                    if uu == 0 && vv == 0 {
                        continue;
                    }
                    let val = (fa * uu * uu + fb * uu * vv + fc * vv * vv).abs();
                    if val < min_abs {
                        min_abs = val;
                    }
                }
            }
            let expected_min = if nd.z == 2 { 2 } else { nd.z as i128 };
            assert_eq!(min_abs, expected_min, "Markov form minimum failed at z={}", nd.z);
            min_form_checked += 1;
        }

        // 5. Direct Markov (1879-80) periodic continued fraction psi(W_{p,q}) -> {1, 2} ONLY
        //    and exact Lagrange constant max_j (theta_j + beta_j) == sqrt(9 - 4/z^2) < 3
        let (mu_cf, period) = markov_lagrange_from_periodic_cf(&word);
        assert!(period.iter().all(|&c| c == 1 || c == 2), "Markov CF must stay in {{1,2}}");
        let mu_exact = (9.0 - 4.0 / ((nd.z as f64) * (nd.z as f64))).sqrt();
        assert!(mu_exact < 3.0);
        let err = (mu_cf - mu_exact).abs();
        assert!(err < 1e-11, "Lagrange constant mismatch at z={}: cf={mu_cf} vs exact={mu_exact}", nd.z);
        max_cf_err = max_cf_err.max(err);
        verified_1e5 += 1;

        println!(
            "  {:>7} {:>17} {:>3}/{:<3} {:>3} | {:>12} ({:>5},{:>6},{:>6}) {:>15.12} {:>10.2e}",
            nd.z,
            format!("({},{},{})", nd.x, nd.y, nd.z),
            nd.p,
            nd.q,
            nd.p + nd.q,
            mat.trace() / 3,
            fa,
            fb,
            fc,
            mu_exact,
            err
        );
    }

    // Also verify ALL nodes up to M <= 10^9 (Diophantine + Cohn SL_2(Z) trace + m_12 + Fricke commutator = -2!)
    let mut verified_1e9 = 0usize;
    let mut fricke_checked = 0usize;
    let ab_comm = Mat2::MARKOV_A
        .mul(Mat2::MARKOV_B)
        .mul(Mat2::MARKOV_A.inv_sl2())
        .mul(Mat2::MARKOV_B.inv_sl2());
    assert_eq!(ab_comm.trace(), -2, "Fricke commutator tr([A,B]) != -2");

    for nd in &nodes_1e9 {
        assert_eq!(nd.x * nd.x + nd.y * nd.y + nd.z * nd.z, 3 * nd.x * nd.y * nd.z);
        let word = lower_christoffel_word(nd.p, nd.q);
        let mat = cohn_markov_matrix(&word).expect("i128 matrix fit for M <= 10^9");
        assert_eq!(mat.det(), 1);
        assert_eq!(mat.trace(), 3 * (nd.z as i128));
        assert_eq!(mat.b, nd.z as i128);
        let _ = markov_canonical_form(nd.x, nd.y, nd.z);
        verified_1e9 += 1;
        if nd.p >= 1 && nd.q >= 1 {
            fricke_checked += 1;
        }
    }

    println!(
        "\n  [PASS] Part 1: {verified_1e5}/{verified_1e5} Markov triples (M <= 10^5) & {verified_1e9}/{verified_1e9} (M <= 10^9) 100% EXACT!"
    );
    println!(
        "         x^2+y^2+z^2==3xyz, det(M)==1, tr(M(W))==3M, M_12==M, tr([A,B])==-2 ({fricke_checked} proper), form-min ({min_form_checked} blks), max CF-Lagrange err = {max_cf_err:.2e}\n"
    );
    nodes_1e5
}

fn verify_part2_fq2_sylvester_s2b_depth(nodes_1e5: &[MarkovNode]) {
    println!("================================================================================");
    println!("PART 2 — EXACT FQ2 CUT-TOWER DEPTH, CA-34 MATRIX-TREE tau_r(G_k), AND");
    println!("         CA-03 OSTROWSKI / CA-35 LUCAS S2b DEPTH RIGIDITY (5,500+ CHRISTOFFEL)");
    println!("================================================================================");

    let fib_u = fib_u_seq(45);

    // Build test set:
    // (A) All proper Markov-Christoffel words M <= 10^5 (p >= 1, q >= 1)
    // (B) ALL coprime (p, q) with 1 <= p <= 100, 1 <= q <= 100 (6,087 words!)
    let mut markov_checked = 0usize;
    let mut markov_levels_total = 0usize;
    for nd in nodes_1e5 {
        if nd.p == 0 || nd.q == 0 {
            continue;
        }
        let mut l = vec![b'b'; nd.q];
        l.extend(std::iter::repeat_n(b'a', nd.p));
        let (_cf, cf_sum, _max_ai) = cf_sum_and_max(nd.p as u64, nd.q as u64);

        // Verify for primary index p=0 (and also p = n/2)
        for &primary in &[0u32, ((nd.p + nd.q) / 2) as u32] {
            let mut cur = LevelState::from_bwt(&l, primary);
            let mut ep = nd.p as i128;
            let mut eq = nd.q as i128;
            let mut depth = 0u64;
            loop {
                let bands = cur.extract_affine_bands();
                let (tau_sup, tau_wt, z_sup) = LevelState::band_matrix_tree_dets(&bands, cur.m);
                assert_eq!(tau_sup, 1, "Unweighted support tau_r(G_k) must be 1 on Markov-Christoffel");
                assert_eq!(z_sup, 1, "Unweighted support Z_edge(G_k) must be 1 (Solved Analogue K!)");
                if cur.m == 1 {
                    assert_eq!(tau_wt, 1);
                    break;
                }
                assert_eq!(bands.len(), 2, "Every non-terminal FQ2 level of W_{{p,q}} must have 2 bands");
                assert_eq!(
                    tau_wt,
                    ep.min(eq),
                    "Weighted CA-34 Matrix-Tree tau_r(G_k^wt) must equal exact Euclidean remainder min(p_k, q_k)!"
                );
                let retained = cur.forest_section_cut(&bands);
                cur = cur.induce_return(&retained);
                if ep > eq {
                    ep -= eq;
                } else if eq > ep {
                    eq -= ep;
                } else {
                    // (1, 1) -> m=1
                    ep = 1;
                    eq = 0;
                }
                depth += 1;
                markov_levels_total += 1;
                assert!(depth <= 500);
            }
            assert_eq!(
                depth, cf_sum,
                "FQ2 depth must equal exact sum(a_i) for ({}, {}) primary={primary}",
                nd.p, nd.q
            );
        }
        markov_checked += 1;
    }

    // (B) ALL coprime (p, q) with 1 <= p <= 100, 1 <= q <= 100 (6,087 general rational Christoffel words!)
    let mut gen_checked = 0usize;
    let mut gen_levels_total = 0usize;
    let mut gen_markov_zone = 0usize;
    let mut gen_hipq = 0usize;
    let mut s2b_depth_checked = 0usize;
    let mut s2b_depth_exact_sum = 0usize;
    let mut s2b_max_diff_from_sum = 0i64;

    for p in 1..=100usize {
        for q in 1..=100usize {
            if gcd_u64(p as u64, q as u64) != 1 {
                continue;
            }
            let (cf, cf_sum, max_ai) = cf_sum_and_max(p as u64, q as u64);
            let ost_q = ostrowski_denominators(&cf);
            assert_eq!(*ost_q.last().unwrap(), q as u64, "Ostrowski last denominator q_ell == q");

            if max_ai <= 2 {
                gen_markov_zone += 1;
            } else {
                gen_hipq += 1;
            }

            let mut l = vec![b'b'; q];
            l.extend(std::iter::repeat_n(b'a', p));
            let mut cur = LevelState::from_bwt(&l, 0);
            let mut ep = p as i128;
            let mut eq = q as i128;
            let mut depth = 0u64;
            loop {
                let bands = cur.extract_affine_bands();
                let (tau_sup, tau_wt, z_sup) = LevelState::band_matrix_tree_dets(&bands, cur.m);
                assert_eq!(tau_sup, 1);
                assert_eq!(z_sup, 1);
                if cur.m == 1 {
                    assert_eq!(tau_wt, 1);
                    break;
                }
                assert_eq!(bands.len(), 2);
                assert_eq!(tau_wt, ep.min(eq));
                let retained = cur.forest_section_cut(&bands);
                cur = cur.induce_return(&retained);
                if ep > eq {
                    ep -= eq;
                } else if eq > ep {
                    eq -= ep;
                } else {
                    ep = 1;
                    eq = 0;
                }
                depth += 1;
                gen_levels_total += 1;
            }
            assert_eq!(depth, cf_sum);
            if p > LIMIT as usize && q > LIMIT as usize {
                let srep = run_s2b_spectrum(&l, 0, LIMIT);
                s2b_depth_checked += 1;
                let diff = srep.plain_max_depth as i64 - cf_sum as i64;
                if diff == 0 {
                    s2b_depth_exact_sum += 1;
                }
                s2b_max_diff_from_sum = s2b_max_diff_from_sum.max(diff.abs());
            }
            gen_checked += 1;
        }
    }

    println!(
        "  [PASS] Exact FQ2 + CA-34 Matrix-Tree Rigidity on {markov_checked} Markov-Christoffel (x2 primary, {markov_levels_total} levels)"
    );
    println!(
        "         AND {gen_checked} General Christoffel words (1<=p,q<=100; {gen_markov_zone} Markov-zone max_ai<=2, {gen_hipq} High-PQ max_ai>2, {gen_levels_total} levels):"
    );
    println!(
        "         * Unweighted support tau_r(G_k^sup) == 1 & Z_edge(G_k^sup) == 1 (Solved Analogue K) at 100.00% of ALL levels!"
    );
    println!(
        "         * Weighted multigraph tau_r(G_k^wt) == min(p_k, q_k) (exact Euclidean descent) at 100.00% of ALL levels!"
    );
    println!(
        "         * Exact FQ2 cut-tower depth D_FQ2(W_{{p,q}}) == sum(a_i) at 100.00% of ALL words!"
    );
    println!(
        "         * Open<16> plain S2b max_depth vs sum(a_i) on ALL {s2b_depth_checked} thick pairs (17<=p,q<=100): exact={s2b_depth_exact_sum}/{s2b_depth_checked}, max |h_S2b - sum(a_i)| = {s2b_max_diff_from_sum}!"
    );

    // Exact rational reconstruction from canonical CF [a_0; a_1, ..., a_ell]
    let from_cf = |cf: &[u64]| -> (usize, usize) {
        let mut num = 1u64;
        let mut den = 0u64;
        for &a in cf.iter().rev() {
            let next_num = a * num + den;
            den = num;
            num = next_num;
        }
        (num as usize, den as usize)
    };

    // (C) Large-n Open<16> S2b Depth Contrast: Lucas-Fibonacci (CA-35, a_i == 1) vs Markov-Zone (a_i in {1,2}) vs High-PQ (a_i >> 2)!
    println!("\n  [Open<16> S2b Depth Rigidity: Lucas (CA-35) vs Markov-Zone (a_i<=2) vs High-PQ (a_i>>2)]:");
    println!(
        "  {:<24} {:>7} {:>5} {:>5} {:>5} | {:>6} {:>6} {:>6} | {:>6} {:>6} {:>6} | {:>5} {:>6}",
        "Christoffel Family", "n=p+q", "CF_l", "sum_a", "max_a", "plStp", "plDep", "Def_pl", "spStp", "spDep", "Def_sp", "h_Luc", "spFir"
    );

    let (pell_p1, pell_q1) = from_cf(&[2, 2, 2, 2, 2, 2, 2, 2]);
    let (pell_p2, pell_q2) = from_cf(&[2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]);
    let (alt_p, alt_q) = from_cf(&[1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 2]);
    let (mk5_p, mk5_q) = from_cf(&[1, 1, 2, 2, 1, 1, 2, 2, 1, 1, 2, 2, 1, 1, 2, 2]);

    let cases: Vec<(&str, usize, usize)> = vec![
        // 1. Lucas-Fibonacci extreme (CA-35): a_i == 1, bottom of Markov spectrum (M=1, mu=sqrt(5))
        ("Lucas Fib F_14 (a=1)", fib_u[13] as usize, fib_u[12] as usize),
        ("Lucas Fib F_18 (a=1)", fib_u[17] as usize, fib_u[16] as usize),
        ("Lucas Fib F_22 (a=1)", fib_u[21] as usize, fib_u[20] as usize),
        ("Lucas Fib F_25 (a=1)", fib_u[24] as usize, fib_u[23] as usize),
        // 2. Pure Pell / Markov M=2 (a_i == 2 strictly, silver ratio mu=2*sqrt(2) < 3)
        ("Pell [2;2^7] (a=2)", pell_p1, pell_q1),
        ("Pell [2;2^10] (a=2)", pell_p2, pell_q2),
        // 3. Mixed Markov-zone a_i in {1, 2} strictly (below Markov < 3 threshold!)
        ("Markov alt {1,2}", alt_p, alt_q),
        ("Markov M=5 (1122)^4", mk5_p, mk5_q),
        // 4. High-PQ (a_i >> 2, FAR ABOVE Markov < 3 threshold -> near-periodic!)
        ("High-PQ [10; 20]", 201, 20),
        ("High-PQ [1; 50, 20]", 1021, 1001),
        ("High-PQ [1; 200, 50]", 10051, 10001),
        ("High-PQ [1; 500, 20]", 10021, 10001),
        ("High-PQ [20; 500]", 10001, 500),
    ];

    for (label, p, q) in cases {
        if gcd_u64(p as u64, q as u64) != 1 || p <= 16 || q <= 16 {
            continue;
        }
        let n = p + q;
        let (cf, cf_sum, max_ai) = cf_sum_and_max(p as u64, q as u64);
        let mut l = vec![b'b'; q];
        l.extend(std::iter::repeat_n(b'a', p));
        let rep = run_s2b_spectrum(&l, 0, LIMIT);
        assert_eq!(rep.plain_max_len, rep.spiral_max_len);
        let h_luc = lucas_opt_depth(rep.plain_max_len, &fib_u);
        let def_pl = rep.plain_max_depth as i32 - h_luc as i32;
        let def_sp = rep.spiral_max_depth as i32 - h_luc as i32;
        println!(
            "  {:<24} {:>7} {:>5} {:>5} {:>5} | {:>6} {:>6} {:>+6} | {:>6} {:>6} {:>+6} | {:>5} {:>6}",
            label,
            n,
            cf.len(),
            cf_sum,
            max_ai,
            rep.plain_steps,
            rep.plain_max_depth,
            def_pl,
            rep.spiral_steps,
            rep.spiral_max_depth,
            def_sp,
            h_luc,
            rep.spiral_fires_q2 + rep.spiral_fires_q3p
        );
    }
    println!();
}

// ============================================================================
// 6. Part 3: Full 249-Block Real-Corpus Markov–Lagrange & 2-Band Spectrum
// ============================================================================

fn analyze_real_corpus(label: &str, blocks: &[BlockL], per_block: bool) {
    println!("================================================================================");
    println!("PART 3 — 2-BAND INDUCED MARKOV–LAGRANGE SPECTRUM: {label} ({} blks)", blocks.len());
    println!("================================================================================");

    let mut p_agg = PairSpectrumBlock::default();
    let mut s_agg = S2bSpectrumReport::default();
    let mut total_blocks = 0usize;
    let mut sum_sigma = 0usize;
    let mut total_pairs_r2_chr_blks = 0usize;
    let mut total_blks_with_hi_pq_spiral = 0usize;

    if per_block {
        println!(
            "  {:<13} {:>4} {:>6} | {:>5} {:>5} {:>5} {:>6} | {:>5} {:>5} {:>5} {:>5} {:>5} | {:>5} {:>5} {:>5} {:>5} {:>5}",
            "Block", "sig", "Pairs", "r2Chr", "r2Mk", "r2Hi", "r2Id", "Thk%", "Self%", "2Hi%", "2Mk%", "Chao%", "S2bSl", "S2bHi", "S2bMk", "S2bMx", "Sav%"
        );
    }

    for b in blocks {
        let ps = analyze_2letter_pair_spectrum(&b.l, b.p, LIMIT as usize);
        let ss = run_s2b_spectrum(&b.l, b.p, LIMIT);
        total_blocks += 1;
        sum_sigma += ps.sigma;
        if ps.pairs_r2_christoffel > 0 {
            total_pairs_r2_chr_blks += 1;
        }
        if ss.spiral_fires_q3p > 0 {
            total_blks_with_hi_pq_spiral += 1;
        }

        // Accumulate pair stats
        p_agg.total_pairs += ps.total_pairs;
        p_agg.pairs_r2_christoffel += ps.pairs_r2_christoffel;
        p_agg.pairs_r2_chr_markov += ps.pairs_r2_chr_markov;
        p_agg.pairs_r2_chr_hipq += ps.pairs_r2_chr_hipq;
        p_agg.pairs_r2_chr_primitive += ps.pairs_r2_chr_primitive;
        p_agg.pairs_r2_identity += ps.pairs_r2_identity;
        p_agg.pairs_near_bal_3_4 += ps.pairs_near_bal_3_4;
        p_agg.pairs_chaotic_5p += ps.pairs_chaotic_5p;
        p_agg.sum_defect += ps.sum_defect;
        p_agg.sum_cf_len_r2 += ps.sum_cf_len_r2;
        p_agg.sum_max_ai_r2 += ps.sum_max_ai_r2;
        p_agg.max_ai_r2_seen = p_agg.max_ai_r2_seen.max(ps.max_ai_r2_seen);
        p_agg.r2_markov_mu_sum += ps.r2_markov_mu_sum;
        p_agg.r2_markov_mu_cnt += ps.r2_markov_mu_cnt;
        p_agg.thick_rows_total += ps.thick_rows_total;
        p_agg.thick_self_loop += ps.thick_self_loop;
        p_agg.thick_r2_chr_hipq += ps.thick_r2_chr_hipq;
        p_agg.thick_r2_chr_markov += ps.thick_r2_chr_markov;
        p_agg.thick_r2_identity += ps.thick_r2_identity;
        p_agg.thick_near_bal_3_4 += ps.thick_near_bal_3_4;
        p_agg.thick_chaotic_5p += ps.thick_chaotic_5p;
        p_agg.thin_rows_total += ps.thin_rows_total;
        p_agg.thin_self_loop += ps.thin_self_loop;
        p_agg.thin_r2_chr_hipq += ps.thin_r2_chr_hipq;
        p_agg.thin_r2_chr_markov += ps.thin_r2_chr_markov;
        p_agg.thin_r2_identity += ps.thin_r2_identity;
        p_agg.thin_near_bal_3_4 += ps.thin_near_bal_3_4;
        p_agg.thin_chaotic_5p += ps.thin_chaotic_5p;

        // Accumulate S2b dynamic stats
        s_agg.n += ss.n;
        s_agg.m += ss.m;
        s_agg.ns += ss.ns;
        s_agg.thick_rows += ss.thick_rows;
        s_agg.plain_steps += ss.plain_steps;
        s_agg.plain_joins += ss.plain_joins;
        s_agg.plain_max_depth = s_agg.plain_max_depth.max(ss.plain_max_depth);
        s_agg.spiral_steps += ss.spiral_steps;
        s_agg.spiral_joins += ss.spiral_joins;
        s_agg.spiral_fires_q2 += ss.spiral_fires_q2;
        s_agg.spiral_fires_q3p += ss.spiral_fires_q3p;
        s_agg.spiral_steps_saved_q2 += ss.spiral_steps_saved_q2;
        s_agg.spiral_steps_saved_q3p += ss.spiral_steps_saved_q3p;
        s_agg.spiral_max_depth = s_agg.spiral_max_depth.max(ss.spiral_max_depth);
        s_agg.rows_self_loop += ss.rows_self_loop;
        s_agg.rows_2band_hi_pq += ss.rows_2band_hi_pq;
        s_agg.rows_2band_markov += ss.rows_2band_markov;
        s_agg.rows_multiband_mix += ss.rows_multiband_mix;
        s_agg.steps_self_loop += ss.steps_self_loop;
        s_agg.steps_2band_hi_pq += ss.steps_2band_hi_pq;
        s_agg.steps_2band_markov += ss.steps_2band_markov;
        s_agg.steps_multiband_mix += ss.steps_multiband_mix;
        s_agg.max_spiral_q = s_agg.max_spiral_q.max(ss.max_spiral_q);
        s_agg.thick_comps_total += ss.thick_comps_total;
        s_agg.thick_comps_size1 += ss.thick_comps_size1;
        s_agg.thick_comps_size2 += ss.thick_comps_size2;
        s_agg.thick_comps_size2_rows += ss.thick_comps_size2_rows;
        s_agg.thick_comps_2letter += ss.thick_comps_2letter;
        s_agg.thick_comps_2letter_rows += ss.thick_comps_2letter_rows;

        if per_block {
            let sav = pct(ss.spiral_steps_saved_q2 + ss.spiral_steps_saved_q3p, ss.plain_steps);
            println!(
                "  {:<13} {:>4} {:>6} | {:>5} {:>5} {:>5} {:>6} | {:>4.1}% {:>4.1}% {:>4.1}% {:>4.1}% {:>4.1}% | {:>4.1}% {:>4.1}% {:>4.1}% {:>4.1}% {:>4.2}%",
                b.name,
                ps.sigma,
                ps.total_pairs,
                ps.pairs_r2_christoffel,
                ps.pairs_r2_chr_markov,
                ps.pairs_r2_chr_hipq,
                ps.pairs_r2_identity,
                pct(ps.thick_rows_total, b.l.len() as u64),
                pct(ps.thick_self_loop, ps.thick_rows_total),
                pct(ps.thick_r2_chr_hipq, ps.thick_rows_total),
                pct(ps.thick_r2_chr_markov, ps.thick_rows_total),
                pct(ps.thick_chaotic_5p + ps.thick_near_bal_3_4, ps.thick_rows_total),
                pct(ss.rows_self_loop, ss.n),
                pct(ss.rows_2band_hi_pq, ss.n),
                pct(ss.rows_2band_markov, ss.n),
                pct(ss.rows_multiband_mix, ss.n),
                sav
            );
        }
    }

    let mean_def = p_agg.sum_defect as f64 / p_agg.total_pairs.max(1) as f64;
    let mean_cf_r2 = p_agg.sum_cf_len_r2 as f64 / p_agg.pairs_r2_christoffel.max(1) as f64;
    let mean_ai_r2 = p_agg.sum_max_ai_r2 as f64 / p_agg.pairs_r2_christoffel.max(1) as f64;
    let mean_mu_r2 = if p_agg.r2_markov_mu_cnt > 0 {
        p_agg.r2_markov_mu_sum / p_agg.r2_markov_mu_cnt as f64
    } else {
        0.0
    };
    let tot_saved = s_agg.spiral_steps_saved_q2 + s_agg.spiral_steps_saved_q3p;

    println!("\n  [AGGREGATE 2-LETTER PAIR SPECTRUM — {label} ({total_blocks} blks, mean sigma={:.1})]:",
        sum_sigma as f64 / total_blocks.max(1) as f64);
    println!(
        "    Distinct pairs {{a,b}}: {} | r2==2 Christoffel ('b^q a^p')={} ({:.2}%, in {total_pairs_r2_chr_blks}/{total_blocks} blks: prim={}, Markov(max_ai<=2)={}, High-PQ(max_ai>2)={})",
        p_agg.total_pairs,
        p_agg.pairs_r2_christoffel,
        pct(p_agg.pairs_r2_christoffel, p_agg.total_pairs),
        p_agg.pairs_r2_chr_primitive,
        p_agg.pairs_r2_chr_markov,
        p_agg.pairs_r2_chr_hipq
    );
    println!(
        "    r2==2 Identity ('a^p b^q')={} ({:.2}%) | Near-bal (3<=r2<=4)={} ({:.2}%) | Chaotic (r2>=5)={} ({:.2}%) | Mean Sturmian defect (r2-2)={mean_def:.1}",
        p_agg.pairs_r2_identity,
        pct(p_agg.pairs_r2_identity, p_agg.total_pairs),
        p_agg.pairs_near_bal_3_4,
        pct(p_agg.pairs_near_bal_3_4, p_agg.total_pairs),
        p_agg.pairs_chaotic_5p,
        pct(p_agg.pairs_chaotic_5p, p_agg.total_pairs)
    );
    println!(
        "    On r2==2 Christoffel pairs: mean CF_len={mean_cf_r2:.2}, mean max(a_i)={mean_ai_r2:.1} (max={}), mean Markov mu={mean_mu_r2:.6}",
        p_agg.max_ai_r2_seen
    );
    println!(
        "  [STATIC BIGRAM-PAIR THICK-MASS DECOMPOSITION — Open<16> N_thick={} ({:.2}% of n)]:",
        p_agg.thick_rows_total,
        pct(p_agg.thick_rows_total, s_agg.n)
    );
    println!(
        "    1-letter Self-Loop (a==b, a_1=inf>>2): {:.3}% | r2==2 Chr High-PQ (a_i>2): {:.4}% | r2==2 Chr Markov (a_i<=2): {:.4}% | r2==2 Id: {:.4}% | Near-bal (3..4): {:.3}% | Chaotic (r2>=5): {:.3}%",
        pct(p_agg.thick_self_loop, p_agg.thick_rows_total),
        pct(p_agg.thick_r2_chr_hipq, p_agg.thick_rows_total),
        pct(p_agg.thick_r2_chr_markov, p_agg.thick_rows_total),
        pct(p_agg.thick_r2_identity, p_agg.thick_rows_total),
        pct(p_agg.thick_near_bal_3_4, p_agg.thick_rows_total),
        pct(p_agg.thick_chaotic_5p, p_agg.thick_rows_total)
    );
    println!(
        "    (Contrast Thin m={}: Self={:.2}%, r2ChrHi={:.4}%, r2ChrMk={:.4}%, r2Id={:.4}%, Near3..4={:.3}%, Chao5+={:.2}%)",
        p_agg.thin_rows_total,
        pct(p_agg.thin_self_loop, p_agg.thin_rows_total),
        pct(p_agg.thin_r2_chr_hipq, p_agg.thin_rows_total),
        pct(p_agg.thin_r2_chr_markov, p_agg.thin_rows_total),
        pct(p_agg.thin_r2_identity, p_agg.thin_rows_total),
        pct(p_agg.thin_near_bal_3_4, p_agg.thin_rows_total),
        pct(p_agg.thin_chaotic_5p, p_agg.thin_rows_total)
    );
    println!(
        "  [DYNAMIC Open<16> S2b 2-BAND & THICK-COMPONENT SPECTRUM]:"
    );
    println!(
        "    S2a Thick-Overlap Components: total={} (size-1={}, size-2={} [{:.3}% thick rows], 2-letter={} [{:.3}% thick rows])",
        s_agg.thick_comps_total,
        s_agg.thick_comps_size1,
        s_agg.thick_comps_size2,
        pct(s_agg.thick_comps_size2_rows, s_agg.thick_rows),
        s_agg.thick_comps_2letter,
        pct(s_agg.thick_comps_2letter_rows, s_agg.thick_rows)
    );
    println!(
        "    S2b Row-Peel Mass (100% of n={}): Self-Loop(x==y)={:.2}% | 2-Band High-PQ(q>=3)={:.3}% | 2-Band Markov(q in 1..2)={:.2}% | Multi-Band Mix={:.2}%",
        s_agg.n,
        pct(s_agg.rows_self_loop, s_agg.n),
        pct(s_agg.rows_2band_hi_pq, s_agg.n),
        pct(s_agg.rows_2band_markov, s_agg.n),
        pct(s_agg.rows_multiband_mix, s_agg.n)
    );
    println!(
        "    S2b Step Counts: plain={} -> spiral={} (saved={} [{:.3}%]: q=2 fires={} saved={}, q>=3 fires={} saved={} in {total_blks_with_hi_pq_spiral}/{total_blocks} blks, max_q={})",
        s_agg.plain_steps,
        s_agg.spiral_steps,
        tot_saved,
        pct(tot_saved, s_agg.plain_steps),
        s_agg.spiral_fires_q2,
        s_agg.spiral_steps_saved_q2,
        s_agg.spiral_fires_q3p,
        s_agg.spiral_steps_saved_q3p,
        s_agg.max_spiral_q
    );
    println!(
        "    S2b Joins & Depth: plain_joins={} -> spiral_joins={} ({:+.3}%) | max_depth: plain={} -> spiral={}\n",
        s_agg.plain_joins,
        s_agg.spiral_joins,
        100.0 * (s_agg.spiral_joins as f64 - s_agg.plain_joins as f64) / s_agg.plain_joins.max(1) as f64,
        s_agg.plain_max_depth,
        s_agg.spiral_max_depth
    );
}

fn main() {
    let t0 = Instant::now();
    println!("Cycle 50 P2 — Analogue N, Theorem N1: Markov (1879-80) Quadratic Forms & 2-Band Spectrum\n");

    let nodes_1e5 = verify_part1_markov_tree();
    verify_part2_fq2_sylvester_s2b_depth(&nodes_1e5);

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
    println!(
        "Loaded all 249 real blocks: 6 Admitted + 6 JSONL/Syslog + 4 Code/Docs + 233 Full Wikidata.\n"
    );

    analyze_real_corpus("6 Admitted Wikidata", &adm6, true);
    analyze_real_corpus("6 Non-Wiki (5 JSONL + 1 Syslog)", &nw6, true);
    analyze_real_corpus("4 Code/Docs", &cd4, true);
    analyze_real_corpus("233 Full Wikidata", &wiki233, false);

    println!("Total elapsed: {:.2}s", t0.elapsed().as_secs_f64());
}
