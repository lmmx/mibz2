#![forbid(unsafe_code)]

//! Cycle 47 P3 — Track 2: Foundational `FQ2` ("Is the First Return the Right Object?")
//! Exact Iterated First-Return Cut Tower `L0 -> L1 -> L2 -> L3 -> L4 -> L5 -> ... -> 1`
//! with Classical Pre-1950 Algebra & Dynamics:
//!   - Smith Normal Form (`1861`, MSC `15A21`) with exact arbitrary-precision BigInt (zero overflow!)
//!   - Frobenius-Stickelberger Abelian Torsion / Critical Group (`1879`, MSC `20K01`)
//!   - Kirchhoff Matrix-Tree Laplacian (`1847`, MSC `05C50`)
//!   - Jacobi-Perron Multidimensional Continued Fractions (`1868/1907`, MSC `11J70`)
//!   - Poincaré-Birkhoff Iterated Section Return Tower (`1890/1931`, MSC `37A05`)
//!   - Schreier Coset Graphs & Transversal Rewriting (`1927`, MSC `20F05`)
//!
//! Compiles with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;

// ============================================================================
// 1. Minimal Exact Signed BigInt (Base 10^9) for Overflow-Free Smith Normal Form (1861)
// ============================================================================

const BASE: u64 = 1_000_000_000;

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
        let mut mag = v.unsigned_abs();
        let mut limbs = Vec::new();
        while mag > 0 {
            limbs.push((mag % BASE) as u32);
            mag /= BASE;
        }
        BigInt { neg, limbs }
    }
    fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }
    fn is_one(&self) -> bool {
        !self.neg && self.limbs.len() == 1 && self.limbs[0] == 1
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
            out.push((s % BASE) as u32);
            carry = s / BASE;
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
            let mut d = av - bv - borrow;
            if d < 0 {
                d += BASE as i64;
                borrow = 1;
            } else {
                borrow = 0;
            }
            out.push(d as u32);
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
        let mut out = vec![0u64; self.limbs.len() + rhs.limbs.len()];
        for (i, &a) in self.limbs.iter().enumerate() {
            let a64 = a as u64;
            let mut carry = 0u64;
            for (j, &b) in rhs.limbs.iter().enumerate() {
                let cur = out[i + j] + a64 * (b as u64) + carry;
                out[i + j] = cur % BASE;
                carry = cur / BASE;
            }
            out[i + rhs.limbs.len()] += carry;
        }
        let mut limbs: Vec<u32> = Vec::with_capacity(out.len());
        let mut carry = 0u64;
        for v in out {
            let s = v + carry;
            limbs.push((s % BASE) as u32);
            carry = s / BASE;
        }
        if carry > 0 {
            limbs.push(carry as u32);
        }
        let mut r = BigInt { neg: self.neg != rhs.neg, limbs };
        r.norm();
        r
    }
    fn mul_small(a: &[u32], m: u64) -> Vec<u32> {
        if m == 0 || a.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(a.len() + 1);
        let mut carry = 0u64;
        for &x in a {
            let cur = (x as u64) * m + carry;
            out.push((cur % BASE) as u32);
            carry = cur / BASE;
        }
        if carry > 0 {
            out.push(carry as u32);
        }
        out
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
        if rhs.limbs.len() == 1 {
            let d = rhs.limbs[0] as u64;
            let mut q_limbs = vec![0u32; self.limbs.len()];
            let mut rem = 0u64;
            for i in (0..self.limbs.len()).rev() {
                let cur = rem * BASE + self.limbs[i] as u64;
                q_limbs[i] = (cur / d) as u32;
                rem = cur % d;
            }
            let mut q = BigInt { neg: self.neg != rhs.neg, limbs: q_limbs };
            q.norm();
            let mut r = BigInt { neg: self.neg, limbs: if rem == 0 { Vec::new() } else { vec![rem as u32] } };
            r.norm();
            return (q, r);
        }
        let mut rem_limbs: Vec<u32> = Vec::new();
        let mut q_limbs = vec![0u32; self.limbs.len()];
        for i in (0..self.limbs.len()).rev() {
            if !(rem_limbs.is_empty() && self.limbs[i] == 0) {
                rem_limbs.insert(0, self.limbs[i]);
            }
            if Self::cmp_mag(&rem_limbs, &rhs.limbs) == Ordering::Less {
                q_limbs[i] = 0;
                continue;
            }
            let mut lo = 1u64;
            let mut hi = BASE - 1;
            let mut best = 1u64;
            while lo <= hi {
                let mid = lo + (hi - lo) / 2;
                let prod = Self::mul_small(&rhs.limbs, mid);
                match Self::cmp_mag(&prod, &rem_limbs) {
                    Ordering::Less | Ordering::Equal => {
                        best = mid;
                        lo = mid + 1;
                    }
                    Ordering::Greater => {
                        hi = mid - 1;
                    }
                }
            }
            let prod = Self::mul_small(&rhs.limbs, best);
            rem_limbs = Self::sub_mag(&rem_limbs, &prod);
            q_limbs[i] = best as u32;
        }
        let mut q = BigInt { neg: self.neg != rhs.neg, limbs: q_limbs };
        q.norm();
        let mut r = BigInt { neg: self.neg, limbs: rem_limbs };
        r.norm();
        (q, r)
    }
    fn log10_approx(&self) -> f64 {
        if self.is_zero() {
            return f64::NEG_INFINITY;
        }
        let n = self.limbs.len();
        if n == 1 {
            (self.limbs[0] as f64).log10()
        } else {
            let top = (self.limbs[n - 1] as f64) * (BASE as f64) + (self.limbs[n - 2] as f64);
            top.log10() + 9.0 * ((n - 2) as f64)
        }
    }
    fn short_str(&self) -> String {
        if self.is_zero() {
            return "0".to_string();
        }
        if self.limbs.len() == 1 {
            format!("{}{}", if self.neg { "-" } else { "" }, self.limbs[0])
        } else if self.limbs.len() == 2 {
            let v = (self.limbs[1] as u64) * BASE + self.limbs[0] as u64;
            format!("{}{v}", if self.neg { "-" } else { "" })
        } else {
            format!("~1e{:.1}", self.log10_approx())
        }
    }
}

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

#[derive(Debug, Clone)]
struct SnfResult {
    dim: usize,
    rank: usize,
    non_unit_factors: Vec<BigInt>,
    log10_torsion_order: f64,
}

/// Exact overflow-free integer Smith Normal Form (`Smith 1861`, MSC `15A21`).
fn smith_normal_form(raw: Vec<Vec<i64>>) -> SnfResult {
    let d = raw.len();
    let mut a: Vec<Vec<BigInt>> = raw
        .into_iter()
        .map(|row| row.into_iter().map(BigInt::from_i64).collect())
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
        let Some((pi, pj)) = best else {
            break;
        };
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

    let mut diag = Vec::new();
    for i in 0..k {
        let v = a[i][i].abs();
        if !v.is_zero() {
            diag.push(v);
        }
    }
    for w in diag.windows(2) {
        let (_, rem) = w[1].div_rem(&w[0]);
        assert!(rem.is_zero(), "SNF divisibility invariant failed!");
    }
    let rank = diag.len();
    let non_unit_factors: Vec<BigInt> = diag.into_iter().filter(|v| !v.is_one()).collect();
    let log10_torsion_order: f64 = non_unit_factors.iter().map(|v| v.log10_approx()).sum();

    SnfResult {
        dim: d,
        rank,
        non_unit_factors,
        log10_torsion_order,
    }
}

// ============================================================================
// 2. Disjoint Set Union (Kruskal Max-Weight Spanning Forest = Schreier Transversal)
// ============================================================================

struct Dsu {
    parent: Vec<u32>,
    rank: Vec<u8>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Dsu {
            parent: (0..n as u32).collect(),
            rank: vec![0; n],
        }
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

// ============================================================================
// 3. Exact Iterated First-Return Level Representation
// ============================================================================

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
    word_id: Vec<u32>,
    word_bytes: Vec<u8>,
    word_offsets: Vec<u32>,
    first_byte: Vec<u8>,
}

#[derive(Debug, Clone)]
struct LevelStats {
    level: usize,
    m: usize,
    sigma: usize,
    bands: usize,
    aff_pieces: usize,
    ov_edges: usize,
    components: usize,
    betti1: usize,
    max_word_len: usize,
    mean_word_len: f64,
    mean_band_w: f64,
    max_band_w: usize,
    w1_band_pct: f64,
    jp_m_ratio: f64,
    jp_b_ratio: f64,
    snf_byte_dim: usize,
    snf_byte_rank: usize,
    snf_byte_non_unit: usize,
    snf_byte_log10: f64,
    snf_byte_top: String,
    snf_band_info: String,
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
        let mut byte_to_id = [u32::MAX; 256];
        let mut word_bytes = Vec::new();
        let mut word_offsets = vec![0u32];
        for a in 0..256 {
            if cnt[a] > 0 {
                let id = (word_offsets.len() - 1) as u32;
                byte_to_id[a] = id;
                word_bytes.push(a as u8);
                word_offsets.push(word_bytes.len() as u32);
            }
        }
        let word_id: Vec<u32> = f_byte.iter().map(|&b| byte_to_id[usize::from(b)]).collect();
        LevelState {
            m: n,
            mark: p as usize,
            pi: fl,
            word_id,
            word_bytes,
            word_offsets,
            first_byte: f_byte,
        }
    }

    fn open16_level1(&self, l: &[u8], limit: usize) -> (Self, usize) {
        let n = l.len();
        assert_eq!(self.m, n);
        let mut retained = vec![false; n];
        let mut thick_strips = 0usize;
        let mut pos = 0usize;
        while pos < n {
            let c = l[pos];
            let mut end = pos + 1;
            while end < n && l[end] == c {
                end += 1;
            }
            let w = end - pos;
            if w <= limit {
                for r in pos..end {
                    retained[r] = true;
                }
            } else {
                thick_strips += 1;
                if (pos..end).contains(&self.mark) {
                    retained[self.mark] = true;
                }
            }
            pos = end;
        }
        let next = self.induce_return(&retained);
        (next, thick_strips)
    }

    fn word_slice(&self, id: u32) -> &[u8] {
        let s = self.word_offsets[id as usize] as usize;
        let e = self.word_offsets[id as usize + 1] as usize;
        &self.word_bytes[s..e]
    }

    fn extract_bands(&self) -> (Vec<BandPiece>, usize) {
        let mut bands = Vec::new();
        let mut aff_pieces = 0usize;
        let mut i = 0usize;
        while i < self.m {
            let mut j_aff = i + 1;
            while j_aff < self.m && self.pi[j_aff] == self.pi[j_aff - 1] + 1 {
                j_aff += 1;
            }
            aff_pieces += 1;
            let mut s = i;
            while s < j_aff {
                let wid = self.word_id[s];
                let mut e = s + 1;
                while e < j_aff && self.word_id[e] == wid {
                    e += 1;
                }
                bands.push(BandPiece {
                    d_start: s as u32,
                    c_start: self.pi[s],
                    width: (e - s) as u32,
                });
                s = e;
            }
            i = j_aff;
        }
        (bands, aff_pieces)
    }

    fn forest_section_cut(&self, bands: &[BandPiece]) -> (usize, usize, usize, Vec<bool>) {
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
        assert_eq!(y as usize, self.m);

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
        let mut forest_edges = 0usize;
        for &ei in &order {
            let e = &edges[ei];
            if dsu.union(e.v, e.u) {
                in_forest[ei] = true;
                forest_edges += 1;
            }
        }
        let components = b - forest_edges;
        let betti1 = edges.len() - forest_edges;

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
        (edges.len(), components, betti1, retained)
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
        let mut word_id_next = vec![0u32; m_next];
        let mut first_byte_next = vec![0u8; m_next];
        let mut word_bytes_next = Vec::new();
        let mut word_offsets_next = vec![0u32];
        let mut intern: HashMap<Vec<u8>, u32> = HashMap::new();
        let mut scratch = Vec::new();

        for (new_idx, &start_x) in ret_rows.iter().enumerate() {
            scratch.clear();
            let mut cur = start_x;
            loop {
                scratch.extend_from_slice(self.word_slice(self.word_id[cur]));
                cur = self.pi[cur] as usize;
                if retained[cur] {
                    break;
                }
            }
            pi_next[new_idx] = rank[cur];
            first_byte_next[new_idx] = scratch[0];
            let wid = if let Some(&id) = intern.get(&scratch) {
                id
            } else {
                let id = (word_offsets_next.len() - 1) as u32;
                word_bytes_next.extend_from_slice(&scratch);
                word_offsets_next.push(word_bytes_next.len() as u32);
                intern.insert(scratch.clone(), id);
                id
            };
            word_id_next[new_idx] = wid;
        }

        LevelState {
            m: m_next,
            mark: mark_next,
            pi: pi_next,
            word_id: word_id_next,
            word_bytes: word_bytes_next,
            word_offsets: word_offsets_next,
            first_byte: first_byte_next,
        }
    }

    fn compute_snf(&self, bands: &[BandPiece]) -> (SnfResult, String) {
        let mut bcnt = [0usize; 256];
        for &b in &self.first_byte {
            bcnt[usize::from(b)] += 1;
        }
        let mut active: Vec<(usize, usize)> = (0..256)
            .filter(|&a| bcnt[a] > 0)
            .map(|a| (bcnt[a], a))
            .collect();
        active.sort_unstable_by(|x, y| y.0.cmp(&x.0).then_with(|| x.1.cmp(&y.1)));

        let max_k = 7usize;
        let dim = if active.len() <= max_k {
            active.len()
        } else {
            max_k + 1
        };
        let mut map = [usize::MAX; 256];
        for (idx, &(_, a)) in active.iter().enumerate() {
            map[a] = if idx < max_k { idx } else { max_k };
        }
        let mut lap = vec![vec![0i64; dim]; dim];
        for x in 0..self.m {
            let u = map[usize::from(self.first_byte[x])];
            let v = map[usize::from(self.first_byte[self.pi[x] as usize])];
            if u != v {
                lap[u][u] += 1;
                lap[u][v] -= 1;
            }
        }
        let snf_byte = smith_normal_form(lap);

        let band_info = if bands.len() <= 20 && bands.len() >= 2 {
            let b = bands.len();
            let mut cod_order: Vec<u32> = (0..b as u32).collect();
            cod_order.sort_unstable_by_key(|&idx| bands[idx as usize].c_start);
            let mut blap = vec![vec![0i64; b]; b];
            let (mut ci, mut di) = (0usize, 0usize);
            let mut y = 0u32;
            while ci < b && di < b {
                let v = cod_order[ci] as usize;
                let u = di;
                let c_end = bands[v].c_start + bands[v].width;
                let d_end = bands[u].d_start + bands[u].width;
                let y1 = c_end.min(d_end);
                if y1 > y && v != u {
                    let w = (y1 - y) as i64;
                    blap[v][v] += w;
                    blap[v][u] -= w;
                }
                y = y1;
                if c_end == y1 {
                    ci += 1;
                }
                if d_end == y1 {
                    di += 1;
                }
            }
            let bs = smith_normal_form(blap);
            let top = bs
                .non_unit_factors
                .last()
                .map_or_else(|| "1".to_string(), |x| x.short_str());
            format!(
                "B={b} rk={} nu={} top={top} lg={:.2}",
                bs.rank,
                bs.non_unit_factors.len(),
                bs.log10_torsion_order
            )
        } else if bands.len() == 1 {
            "B=1 LIMIT(rk=0, T exact)".to_string()
        } else {
            format!("B={} TU inc rk={}", bands.len(), bands.len() - 1)
        };

        (snf_byte, band_info)
    }

    fn decode_text(&self, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(n);
        let mut x = self.mark;
        for _ in 0..self.m {
            out.extend_from_slice(self.word_slice(self.word_id[x]));
            x = self.pi[x] as usize;
            if x == self.mark || out.len() >= n {
                break;
            }
        }
        out.truncate(n);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
        out
    }

    fn total_word_len(&self) -> usize {
        self.word_id
            .iter()
            .map(|&id| {
                (self.word_offsets[id as usize + 1] - self.word_offsets[id as usize]) as usize
            })
            .sum()
    }
}

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

fn run_forest_tower(
    name: &str,
    l: &[u8],
    p: u32,
    expected: &[u8],
    max_depth: usize,
) -> Vec<LevelStats> {
    let n = l.len();
    let mut cur = LevelState::from_bwt(l, p);
    let mut results = Vec::new();
    let mut prev_m = 0usize;
    let mut prev_b = 0usize;

    for depth in 0..=max_depth {
        assert_eq!(cur.total_word_len(), n, "Kac conservation failed at {name} L{depth}");
        let (bands, aff_pieces) = cur.extract_bands();
        let (ov_edges, components, betti1, retained) = cur.forest_section_cut(&bands);
        let sigma = cur.word_offsets.len() - 1;
        let max_word_len = (0..sigma)
            .map(|id| (cur.word_offsets[id + 1] - cur.word_offsets[id]) as usize)
            .max()
            .unwrap_or(0);
        let max_band_w = bands.iter().map(|b| b.width as usize).max().unwrap_or(0);
        let w1_bands = bands.iter().filter(|b| b.width == 1).count();
        let (snf_byte, band_info) = cur.compute_snf(&bands);

        let jp_m_ratio = if depth == 0 { 1.0 } else { prev_m as f64 / cur.m as f64 };
        let jp_b_ratio = if depth == 0 { 1.0 } else { prev_b as f64 / bands.len().max(1) as f64 };
        let top_str = snf_byte
            .non_unit_factors
            .last()
            .map_or_else(|| "1".to_string(), |x| x.short_str());

        results.push(LevelStats {
            level: depth,
            m: cur.m,
            sigma,
            bands: bands.len(),
            aff_pieces,
            ov_edges,
            components,
            betti1,
            max_word_len,
            mean_word_len: n as f64 / cur.m as f64,
            mean_band_w: cur.m as f64 / bands.len().max(1) as f64,
            max_band_w,
            w1_band_pct: 100.0 * w1_bands as f64 / bands.len().max(1) as f64,
            jp_m_ratio,
            jp_b_ratio,
            snf_byte_dim: snf_byte.dim,
            snf_byte_rank: snf_byte.rank,
            snf_byte_non_unit: snf_byte.non_unit_factors.len(),
            snf_byte_log10: snf_byte.log10_torsion_order,
            snf_byte_top: top_str,
            snf_band_info: band_info,
        });

        if cur.m == 1 {
            let decoded = cur.decode_text(n);
            assert_eq!(decoded, expected, "Limit object L{depth} text mismatch on {name}");
            break;
        }
        prev_m = cur.m;
        prev_b = bands.len();
        let next = cur.induce_return(&retained);
        if next.m == cur.m {
            let decoded = next.decode_text(n);
            assert_eq!(decoded, expected, "Fixed-point L{depth} text mismatch on {name}");
            break;
        }
        cur = next;
    }
    let decoded = cur.decode_text(n);
    assert_eq!(decoded, expected, "Final tower level mismatch on {name}");
    results
}

fn run_open16_rooted_tower(name: &str, l: &[u8], p: u32, expected: &[u8]) {
    let n = l.len();
    let l0 = LevelState::from_bwt(l, p);
    let (b0, _) = l0.extract_bands();
    let (l1, thick_strips) = l0.open16_level1(l, 16);
    assert_eq!(l1.total_word_len(), n);
    let (b1, aff1) = l1.extract_bands();
    let (e1, _, betti1, ret1) = l1.forest_section_cut(&b1);
    let l2 = l1.induce_return(&ret1);
    assert_eq!(l2.total_word_len(), n);
    let (b2, _) = l2.extract_bands();
    let (_, _, _, ret2) = l2.forest_section_cut(&b2);
    let l3 = l2.induce_return(&ret2);
    let (b3, _) = l3.extract_bands();
    let (_, _, _, ret3) = l3.forest_section_cut(&b3);
    let l4 = l3.induce_return(&ret3);
    let (b4, _) = l4.extract_bands();
    let (_, _, _, ret4) = l4.forest_section_cut(&b4);
    let l5 = l4.induce_return(&ret4);
    assert_eq!(l5.decode_text(n), expected, "Open16-rooted L5 mismatch on {name}");

    let max_w1 = b1.iter().map(|x| x.width).max().unwrap_or(0);
    let w1_frac1 = 100.0 * b1.iter().filter(|x| x.width == 1).count() as f64 / b1.len().max(1) as f64;
    let w1_frac2 = 100.0 * b2.iter().filter(|x| x.width == 1).count() as f64 / b2.len().max(1) as f64;
    let r_thin = b0.len() - thick_strips;
    println!(
        "{:<16} | L0:m={:>6} B={:>6} | O16-L1:m={:>6} B_ex={:>6} (aff={:>6} H={:>4} r_th={:>6} maxW={:>3} w1={:>4.1}% E={:>6} b1={:>5}) | L2:m={:>5} B={:>5} w1={:>4.1}% | L3:m={:>4} B={:>4} | L4:m={:>3} | L5:m={:>2}",
        name, n, b0.len(), l1.m, b1.len(), aff1, thick_strips, r_thin, max_w1, w1_frac1, e1, betti1,
        l2.m, b2.len(), w1_frac2, l3.m, b3.len(), l4.m, l5.m
    );
}

fn run_small_and_analogue_tests() {
    println!("=== PART 1: SMALL EXACTNESS & SOLVED-ANALOGUE TOWER CHECKS ===");
    {
        let m = vec![
            vec![2, 4, 4],
            vec![-6, 6, 12],
            vec![10, -4, -16],
        ];
        let s = smith_normal_form(m);
        assert_eq!(s.rank, 3);
        assert_eq!(
            s.non_unit_factors,
            vec![BigInt::from_i64(2), BigInt::from_i64(6), BigInt::from_i64(12)]
        );
    }
    let mut checked = 0usize;
    let mut hit_one = 0usize;
    let mut max_depth_seen = 0usize;
    for n in 1..=11usize {
        for mask in 0..(1u32 << n) {
            let mut text = Vec::with_capacity(n);
            for bit in 0..n {
                text.push(if ((mask >> bit) & 1) == 0 { b'a' } else { b'b' });
            }
            let (l, p) = bwt_small(&text);
            let res = run_forest_tower("small_bin", &l, p, &text, 14);
            let last = res.last().unwrap();
            max_depth_seen = max_depth_seen.max(last.level);
            if last.m == 1 {
                hit_one += 1;
            }
            checked += 1;
        }
    }
    for n in 1..=6usize {
        let total = 3usize.pow(n as u32);
        for mut code in 0..total {
            let mut text = Vec::with_capacity(n);
            for _ in 0..n {
                text.push(b'a' + (code % 3) as u8);
                code /= 3;
            }
            let (l, p) = bwt_small(&text);
            let res = run_forest_tower("small_ter", &l, p, &text, 14);
            let last = res.last().unwrap();
            max_depth_seen = max_depth_seen.max(last.level);
            if last.m == 1 {
                hit_one += 1;
            }
            checked += 1;
        }
    }
    println!(
        "  [PASS] {checked} exhaustive small BWTs: Kac conservation + BigInt SNF + exact reconstruction ({hit_one}/{checked} hit m=1; max depth={max_depth_seen})"
    );

    {
        let n = 196_418usize;
        let q = 121_393usize;
        let mut l = vec![b'b'; q];
        l.resize(n, b'a');
        let exp = invert_fl_ref(&l, 0);
        let res = run_forest_tower("Fib-F27", &l, 0, &exp, 10);
        println!("  [Control] Fib-F27 (Sturmian, n={n}, Jacobi-Perron golden ratio phi=1.61803):");
        for st in &res {
            println!(
                "    L{}: m={:>7} |Sig|={:>3} B={:>3} |E|={:>3} b1={:>2} maxL={:>5} w_bar={:>8.2} JP_m={:>5.3}x JP_B={:>4.2}x SNF_byte(d={},rk={},nu={},lg={:.1})",
                st.level, st.m, st.sigma, st.bands, st.ov_edges, st.betti1, st.max_word_len, st.mean_band_w, st.jp_m_ratio, st.jp_b_ratio,
                st.snf_byte_dim, st.snf_byte_rank, st.snf_byte_non_unit, st.snf_byte_log10
            );
        }
    }
    println!();
}

fn print_tower(name: &str, stats: &[LevelStats]) {
    println!("--- {name} ---");
    println!(
        "{:>3} {:>7} {:>7} {:>6} {:>6} {:>7} {:>3} {:>6} {:>7} {:>7} {:>6} {:>6} {:>6} {:>6} {:>25} {:>26}",
        "Lvl", "m_k", "|Sig_k|", "B_k", "B_aff", "|E_k|", "cp", "betti1", "maxWLen", "meanWL", "maxBw", "w1%", "JP_m", "JP_B", "SNF_8x8(d,rk,nu,lg,top)", "SNF_band / TU"
    );
    for s in stats {
        let snf_b = format!(
            "{},{},{},{:.1},{}",
            s.snf_byte_dim, s.snf_byte_rank, s.snf_byte_non_unit, s.snf_byte_log10, s.snf_byte_top
        );
        println!(
            " L{:<1} {:>7} {:>7} {:>6} {:>6} {:>7} {:>3} {:>6} {:>7} {:>7.1} {:>6} {:>5.1}% {:>5.2}x {:>5.2}x {:>25} {:>26} (w={:.2})",
            s.level,
            s.m,
            s.sigma,
            s.bands,
            s.aff_pieces,
            s.ov_edges,
            s.components,
            s.betti1,
            s.max_word_len,
            s.mean_word_len,
            s.max_band_w,
            s.w1_band_pct,
            s.jp_m_ratio,
            s.jp_b_ratio,
            snf_b,
            s.snf_band_info,
            s.mean_band_w
        );
    }
}

fn main() {
    run_small_and_analogue_tests();

    let wiki_specs: [(&str, u32); 6] = [
        ("block-0.L", 264045),
        ("block-2.L", 320807),
        ("block-85.L", 325088),
        ("block-143.L", 802868),
        ("block-207.L", 205180),
        ("block-210.L", 791429),
    ];
    let non_wiki_specs: [(&str, u32); 6] = [
        ("jsonl_block_0.L", 897198),
        ("jsonl_block_1.L", 672135),
        ("jsonl_block_2.L", 334517),
        ("jsonl_block_3.L", 734310),
        ("jsonl_block_4.L", 522665),
        ("syslog_block_0.L", 165238),
    ];

    println!("=== PART 2: CANONICAL FOREST-CHORD ITERATED FIRST-RETURN TOWER (L0 -> ... -> 1) ===");
    let mut wiki_data = Vec::new();
    for &(file, p) in &wiki_specs {
        let path = format!("/home/antigravity/admitted-bwt/{file}");
        let l = fs::read(&path).unwrap();
        let exp = invert_fl_ref(&l, p);
        let stats = run_forest_tower(file, &l, p, &exp, 10);
        assert_eq!(stats.last().unwrap().m, 1, "{file} did not reach m=1 limit!");
        print_tower(file, &stats);
        wiki_data.push((file, l, p, exp));
    }
    println!();
    let mut non_wiki_data = Vec::new();
    for &(file, p) in &non_wiki_specs {
        let path = format!("/tmp/mibz2_corpora/non_wikidata/{file}");
        let l = fs::read(&path).unwrap();
        let exp = invert_fl_ref(&l, p);
        let stats = run_forest_tower(file, &l, p, &exp, 10);
        assert_eq!(stats.last().unwrap().m, 1, "{file} did not reach m=1 limit!");
        print_tower(file, &stats);
        non_wiki_data.push((file, l, p, exp));
    }
    println!();

    println!("=== PART 3: OPEN<16>-ROOTED ITERATED TOWER (L0 -> Open16 L1 -> Forest L2..L5) & L1->L2 OBSTRUCTION ===");
    for (file, l, p, exp) in &wiki_data {
        run_open16_rooted_tower(file, l, *p, exp);
    }
    for (file, l, p, exp) in &non_wiki_data {
        run_open16_rooted_tower(file, l, *p, exp);
    }
    println!();

    println!("=== PART 4: DIRECT L0 -> L2 SHORTCUT TEST (CHORD-OF-CHORD CORRELATION) ===");
    for (file, l, p, _) in wiki_data.iter().chain(non_wiki_data.iter()) {
        let l0 = LevelState::from_bwt(l, *p);
        let (b0, _) = l0.extract_bands();
        let (_, _, _, ret0) = l0.forest_section_cut(&b0);
        let l1 = l0.induce_return(&ret0);
        let (b1, _) = l1.extract_bands();
        let (_, _, _, ret1) = l1.forest_section_cut(&b1);

        let mut l1_to_l0 = Vec::with_capacity(l1.m);
        for (x, &keep) in ret0.iter().enumerate() {
            if keep {
                l1_to_l0.push(x);
            }
        }
        let mut l0_run_w = vec![0u32; l.len()];
        for band in &b0 {
            for r in band.d_start..(band.d_start + band.width) {
                l0_run_w[r as usize] = band.width;
            }
        }
        let mut cnt_w1_l1 = 0usize;
        let mut cnt_w1_l2 = 0usize;
        let mut cnt_le16_l2 = 0usize;
        let mut m2 = 0usize;
        for (idx1, &keep2) in ret1.iter().enumerate() {
            let w0 = l0_run_w[l1_to_l0[idx1]];
            if w0 == 1 {
                cnt_w1_l1 += 1;
            }
            if keep2 {
                m2 += 1;
                if w0 == 1 {
                    cnt_w1_l2 += 1;
                }
                if w0 <= 16 {
                    cnt_le16_l2 += 1;
                }
            }
        }
        let l1_chords = l1.m;
        let l1_w1_b = b1.iter().filter(|x| x.width == 1).count();
        println!(
            "{:<16} | m1={:>6} B1={:>6} (w1_B1={:>5.1}%) | m2={:>5} ({:>4.1}% of m1) | L2_from_L0_w1={:>4.1}% (vs L1 {:>4.1}%) L2_from_L0<=16={:>4.1}%",
            file,
            l1_chords,
            b1.len(),
            100.0 * l1_w1_b as f64 / b1.len().max(1) as f64,
            m2,
            100.0 * m2 as f64 / l1_chords.max(1) as f64,
            100.0 * cnt_w1_l2 as f64 / m2.max(1) as f64,
            100.0 * cnt_w1_l1 as f64 / l1_chords.max(1) as f64,
            100.0 * cnt_le16_l2 as f64 / m2.max(1) as f64
        );
    }
}
