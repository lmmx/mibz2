#![forbid(unsafe_code)]

//! Stepanov Step-by-Step Solved-Analogue Transplantation — Analogue A:
//! Perfectly Clustering BWTs (r = |Σ|, one run per letter).
//!
//! Cycle 46 — Programme Investigator P1 (`research/carbon-kitchen-sink`).
//!
//! Contents:
//! 1. Solved Analogue A (`invert_perfectly_clustering`): explicit Steps A1–A6,
//!    verified on Fibonacci/Christoffel words (up to F_30 = 832,040) and
//!    multi-letter symmetric/general perfectly clustering BWTs (σ = 3..64).
//! 2. Exact failure-localization diagnostics on all 6 authenticated Wikidata
//!    blocks (`block-{0,2,85,143,207,210}.L`), split across the 7
//!    working-backwards classes (GUID, sha1, entity-id, number, JSON-key,
//!    text, structure) and MSC2020 catalogue measurements:
//!    - MSC 11: modular shift cardinality D_mod, O(r) Dedekind-Rademacher sum,
//!      intra-bucket normalized Weyl Erdős-Turán discrepancy ET_16,
//!      Return-Time Dirichlet series Z_J(s) and Cauchy-Schwarz κ_J.
//!    - MSC 37: Abramov-Kac entropy conservation, Green-Kubo return-time
//!      autocorrelation & variance ratio, and Exact Unbranched Kakutani
//!      Thin-Skyscraper census (2 <= w <= 16, targeting repeated sha1).
//!    - MSC 81: Feshbach-Livšic / Wigner-Smith time-delay sum rule Tr(Q)=n-m,
//!      Jordan block spectrum of truncated scattering core U_JJ, exact MPO
//!      spatial tile Schmidt rank χ_k & tile purity IPR_k.
//! 3. Assembled exact chains (`Chain A-Base` = Open<16> + Z/2Z XOR-BiEmit +
//!    Analogue-A fast path; `Chain A-Tube` = Chain A-Base + MSC 37A05 Exact
//!    Unbranched Kakutani Thin-Tube promotion), verified byte-for-byte on
//!    5,500+ synthetic/exhaustive/non-Wikidata BWTs + all 6 Wikidata blocks,
//!    with native 1T / 2T / 4W timing and acquisition-cost breakdown.

use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::fs;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const LIMIT: usize = 16;

struct BlockSpec {
    name: &'static str,
    path: &'static str,
    primary_index: usize,
}

const WIKIDATA_BLOCKS: &[BlockSpec] = &[
    BlockSpec {
        name: "block-0",
        path: "/home/antigravity/admitted-bwt/block-0.L",
        primary_index: 264_045,
    },
    BlockSpec {
        name: "block-2",
        path: "/home/antigravity/admitted-bwt/block-2.L",
        primary_index: 320_807,
    },
    BlockSpec {
        name: "block-85",
        path: "/home/antigravity/admitted-bwt/block-85.L",
        primary_index: 325_088,
    },
    BlockSpec {
        name: "block-143",
        path: "/home/antigravity/admitted-bwt/block-143.L",
        primary_index: 802_868,
    },
    BlockSpec {
        name: "block-207",
        path: "/home/antigravity/admitted-bwt/block-207.L",
        primary_index: 205_180,
    },
    BlockSpec {
        name: "block-210",
        path: "/home/antigravity/admitted-bwt/block-210.L",
        primary_index: 791_429,
    },
];

const CLASS_NAMES: [&str; 7] = [
    "GUID", "sha1", "ent-id", "number", "JSON-key", "text", "struct",
];

// ============================================================================
// Reference exact BWT / inverse BWT helpers
// ============================================================================

fn build_lf(l: &[u8]) -> ([usize; 256], [usize; 256], Vec<u32>) {
    let mut cnt = [0usize; 256];
    for &b in l {
        cnt[b as usize] += 1;
    }
    let mut first = [0usize; 256];
    let mut acc = 0usize;
    for c in 0..256 {
        first[c] = acc;
        acc += cnt[c];
    }
    let mut occ = first;
    let mut lf = vec![0u32; l.len()];
    for (i, &b) in l.iter().enumerate() {
        let u = b as usize;
        lf[i] = occ[u] as u32;
        occ[u] += 1;
    }
    (cnt, first, lf)
}

fn invert_reference(l: &[u8], p: usize) -> Vec<u8> {
    let n = l.len();
    if n == 0 {
        return Vec::new();
    }
    let (_, _, lf) = build_lf(l);
    let mut out = vec![0u8; n];
    let mut cur = p;
    for i in (0..n).rev() {
        out[i] = l[cur];
        cur = lf[cur] as usize;
    }
    out
}

fn forward_bwt_small(t: &[u8]) -> (Vec<u8>, usize) {
    let n = t.len();
    let mut rots: Vec<usize> = (0..n).collect();
    rots.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = t[(a + k) % n];
            let cb = t[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0usize;
    for (rank, &start) in rots.iter().enumerate() {
        if start == 0 {
            p = rank;
        }
        l.push(t[(start + n - 1) % n]);
    }
    (l, p)
}

// ============================================================================
// PART 1: Solved Analogue A — Perfectly Clustering BWTs (Steps A1–A6)
// ============================================================================

#[derive(Debug, Clone)]
struct PerfectClusterCert {
    sigma: usize,
    christoffel_q: Option<usize>,
    ends: Vec<u32>,
    syms: Vec<u8>,
    deltas: Vec<i64>,
}

/// Steps A1 & A2: Recognize r = |Σ| in a single run scan and build O(σ) shift table.
fn recognize_perfectly_clustering(l: &[u8]) -> Option<PerfectClusterCert> {
    let n = l.len();
    if n == 0 {
        return Some(PerfectClusterCert {
            sigma: 0,
            christoffel_q: None,
            ends: Vec::new(),
            syms: Vec::new(),
            deltas: Vec::new(),
        });
    }
    let mut seen = [false; 256];
    let mut starts = Vec::with_capacity(32);
    let mut ends = Vec::with_capacity(32);
    let mut syms = Vec::with_capacity(32);
    let mut cnt = [0usize; 256];

    let mut i = 0usize;
    while i < n {
        let b = l[i];
        if seen[b as usize] {
            return None; // Step A1 fails: letter `b` appears in >= 2 runs!
        }
        seen[b as usize] = true;
        let s = i;
        while i < n && l[i] == b {
            i += 1;
        }
        starts.push(s as u32);
        ends.push(i as u32);
        syms.push(b);
        cnt[b as usize] = i - s;
    }

    // Step A2: O(σ) cumulative starts and constant per-letter shift vector
    let mut first = [0usize; 256];
    let mut acc = 0usize;
    for c in 0..256 {
        first[c] = acc;
        acc += cnt[c];
    }
    let sigma = syms.len();
    let mut deltas = Vec::with_capacity(sigma);
    let mut first_mod_q: Option<usize> = None;
    let mut all_same_mod = true;
    for k in 0..sigma {
        let c = syms[k] as usize;
        let delta = first[c] as i64 - starts[k] as i64;
        deltas.push(delta);
        let q = delta.rem_euclid(n as i64) as usize;
        match first_mod_q {
            None => first_mod_q = Some(q),
            Some(q0) => {
                if q != q0 {
                    all_same_mod = false;
                }
            }
        }
    }
    let christoffel_q = if all_same_mod {
        first_mod_q.filter(|&q| gcd(q, n) == 1)
    } else {
        None
    };

    Some(PerfectClusterCert {
        sigma,
        christoffel_q,
        ends,
        syms,
        deltas,
    })
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// Steps A3, A5, A6: Invert a perfectly clustering BWT without ever reading `l[x]`.
fn emit_perfectly_clustering(cert: &PerfectClusterCert, n: usize, p: usize, out: &mut [u8]) {
    debug_assert_eq!(out.len(), n);
    if n == 0 {
        return;
    }
    if cert.sigma == 1 {
        out.fill(cert.syms[0]);
        return;
    }
    // Christoffel / Sturmian specialization (D_mod = 1, σ = 2): closed-form modular AP
    if cert.sigma == 2 {
        if let Some(q) = cert.christoffel_q {
            let e0 = cert.ends[0] as usize;
            let s0 = cert.syms[0];
            let s1 = cert.syms[1];
            let mut x = p;
            for i in (0..n).rev() {
                out[i] = if x < e0 { s0 } else { s1 };
                x += q;
                if x >= n {
                    x -= n;
                }
            }
            return;
        }
    }
    // General σ-letter perfectly clustering BWT: L1-resident interval dispatch (0 reads of L)
    let sigma = cert.sigma;
    let ends = &cert.ends;
    let syms = &cert.syms;
    let deltas = &cert.deltas;
    let mut x = p as i64;
    if sigma <= 8 {
        for i in (0..n).rev() {
            let xu = x as u32;
            let mut idx = 0usize;
            while idx + 1 < sigma && xu >= ends[idx] {
                idx += 1;
            }
            out[i] = syms[idx];
            x += deltas[idx];
        }
    } else {
        for i in (0..n).rev() {
            let xu = x as u32;
            let mut lo = 0usize;
            let mut hi = sigma - 1;
            while lo < hi {
                let mid = (lo + hi) >> 1;
                if xu < ends[mid] {
                    hi = mid;
                } else {
                    lo = mid + 1;
                }
            }
            out[i] = syms[lo];
            x += deltas[lo];
        }
    }
}

fn invert_perfectly_clustering(l: &[u8], p: usize) -> Option<Vec<u8>> {
    let cert = recognize_perfectly_clustering(l)?;
    let mut out = vec![0u8; l.len()];
    emit_perfectly_clustering(&cert, l.len(), p, &mut out);
    Some(out)
}

/// Generate a valid single-cycle perfectly clustering BWT column of alphabet size `sigma`
/// and approximate length `target_n`.
fn make_perfectly_clustering_bwt(sigma: usize, target_n: usize, seed: u64) -> (Vec<u8>, usize) {
    assert!(sigma >= 2 && sigma <= 64);
    if sigma == 2 {
        // Fibonacci / Christoffel word: a = F_{k-1}, b = F_{k-2}, L = '1'^b '0'^a
        let mut f0 = 1usize;
        let mut f1 = 2usize;
        while f0 + f1 <= target_n {
            let next = f0 + f1;
            f0 = f1;
            f1 = next;
        }
        let n = f1;
        let ones = f0;
        let mut l = Vec::with_capacity(n);
        l.resize(ones, b'1');
        l.resize(n, b'0');
        return (l, 0);
    }
    // Exact Inverse Rauzy-Veech Induction (Rauzy 1979, MSC 37E05):
    // Start from the minimal single-cycle sigma-IET:
    //   top = [1, 2, ..., sigma-1, 0], bot = [0, 1, ..., sigma-1], widths = [1; sigma].
    // Every inverse RV step preserves single-cycle transitivity and r = sigma exactly!
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut next_u64 = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut top: Vec<usize> = (1..sigma).chain(std::iter::once(0)).collect();
    let mut bot: Vec<usize> = (0..sigma).collect();
    let mut widths = vec![1usize; sigma];
    let mut total_n = sigma;

    while total_n < target_n {
        let a = *top.last().unwrap();
        let b = *bot.last().unwrap();
        let pos_a_bot = bot.iter().position(|&x| x == a).unwrap();
        let pos_b_top = top.iter().position(|&x| x == b).unwrap();
        let can_top = pos_a_bot + 1 < sigma;
        let can_bot = pos_b_top + 1 < sigma;
        let choose_top = match (can_top, can_bot) {
            (true, true) => (next_u64() & 1) == 0,
            (true, false) => true,
            (false, true) => false,
            (false, false) => break,
        };
        if choose_top {
            let victim = bot.remove(pos_a_bot + 1);
            let add = widths[victim];
            if total_n + add > target_n && total_n > target_n / 2 {
                bot.insert(pos_a_bot + 1, victim);
                break;
            }
            bot.push(victim);
            widths[a] += add;
            total_n += add;
        } else {
            let victim = top.remove(pos_b_top + 1);
            let add = widths[victim];
            if total_n + add > target_n && total_n > target_n / 2 {
                top.insert(pos_b_top + 1, victim);
                break;
            }
            top.push(victim);
            widths[b] += add;
            total_n += add;
        }
    }

    // Relabel letters by final `bot` order so F is sorted (0..sigma) and L = top (still r = sigma)
    let mut relabel = vec![0usize; sigma];
    for (sorted_rank, &sym) in bot.iter().enumerate() {
        relabel[sym] = sorted_rank;
    }
    let mut l = Vec::with_capacity(total_n);
    for &sym in &top {
        let byte = b'!' + relabel[sym] as u8;
        l.extend(std::iter::repeat_n(byte, widths[sym]));
    }
    (l, 0)
}

// ============================================================================
// PART 2: Working-Backwards 7-Class Text Classifier
// ============================================================================

fn is_hex(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

fn is_lower_hex(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
}

/// Classify every decoded text byte into the 7 working-backwards classes:
/// 0: GUID, 1: sha1, 2: entity-id, 3: number, 4: JSON-key, 5: text, 6: structure.
fn classify_text_bytes(t: &[u8]) -> Vec<u8> {
    let n = t.len();
    let mut cls = vec![255u8; n];

    // Pass 1: GUIDs — optional [QPLM]\d+\$ prefix + 8-4-4-4-12 hex UUID
    let mut i = 0usize;
    while i + 36 <= n {
        if t[i + 8] == b'-'
            && t[i + 13] == b'-'
            && t[i + 18] == b'-'
            && t[i + 23] == b'-'
            && t[i..i + 8].iter().all(|&b| is_hex(b))
            && t[i + 9..i + 13].iter().all(|&b| is_hex(b))
            && t[i + 14..i + 18].iter().all(|&b| is_hex(b))
            && t[i + 19..i + 23].iter().all(|&b| is_hex(b))
            && t[i + 24..i + 36].iter().all(|&b| is_hex(b))
        {
            let mut start = i;
            // Include Wikidata `Q12345$` prefix if present (matches working-backwards GUID mass)
            if start >= 3 && t[start - 1] == b'$' {
                let mut k = start - 1;
                while k > 0 && t[k - 1].is_ascii_digit() {
                    k -= 1;
                }
                if k > 0 && k < start - 1 && matches!(t[k - 1], b'Q' | b'P' | b'L' | b'M' | b'q' | b'p') {
                    start = k - 1;
                }
            }
            for slot in &mut cls[start..i + 36] {
                *slot = 0;
            }
            i += 36;
            continue;
        }
        i += 1;
    }

    // Pass 2: 40-char lowercase hex sha1 hashes (bounded by non-hex)
    i = 0;
    while i + 40 <= n {
        if cls[i] == 255
            && (i == 0 || !is_hex(t[i - 1]))
            && (i + 40 == n || !is_hex(t[i + 40]))
            && t[i..i + 40].iter().all(|&b| is_lower_hex(b))
        {
            // Verify it's not purely digits (real sha1 has at least one a..f)
            if t[i..i + 40].iter().any(|&b| (b'a'..=b'f').contains(&b)) {
                for slot in &mut cls[i..i + 40] {
                    if *slot == 255 {
                        *slot = 1;
                    }
                }
                i += 40;
                continue;
            }
        }
        i += 1;
    }

    // Pass 3: JSON keys `"..."` immediately followed by `:`
    i = 0;
    while i < n {
        if t[i] == b'"' {
            let mut j = i + 1;
            let mut ok = true;
            while j < n && j - i <= 64 {
                if t[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if t[j] == b'"' {
                    break;
                }
                if t[j] == b'\n' {
                    ok = false;
                    break;
                }
                j += 1;
            }
            if ok && j < n && t[j] == b'"' && j + 1 < n && t[j + 1] == b':' {
                for slot in &mut cls[i..=j + 1] {
                    if *slot == 255 {
                        *slot = 4;
                    }
                }
                i = j + 2;
                continue;
            }
        }
        i += 1;
    }

    // Pass 4: Entity IDs ([QPLM]\d+ bounded by non-alnum)
    i = 0;
    while i + 2 <= n {
        if cls[i] == 255
            && matches!(t[i], b'Q' | b'P' | b'L' | b'M')
            && t[i + 1].is_ascii_digit()
            && (i == 0 || !t[i - 1].is_ascii_alphanumeric())
        {
            let mut j = i + 1;
            while j < n && t[j].is_ascii_digit() {
                j += 1;
            }
            if j == n || !t[j].is_ascii_alphanumeric() {
                for slot in &mut cls[i..j] {
                    if *slot == 255 {
                        *slot = 2;
                    }
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }

    // Pass 5: Numbers, text, structure for remaining unclassified bytes
    for idx in 0..n {
        if cls[idx] != 255 {
            continue;
        }
        let b = t[idx];
        if b.is_ascii_digit() {
            cls[idx] = 3; // number
        } else if b.is_ascii_alphabetic() || b >= 0x80 {
            cls[idx] = 5; // text
        } else {
            cls[idx] = 6; // structure
        }
    }
    cls
}

// ============================================================================
// PART 2B: MSC 11 / MSC 37 / MSC 81 Exact Diagnostic Computations
// ============================================================================

/// Exact O(r) Faulhaber computation of Dedekind-Rademacher sum:
/// S_DR(LF) = sum_{x=0}^{n-1} ((x/n)) * ((LF(x)/n)), rho = (12/n) * S_DR(LF).
fn dedekind_rademacher_or(runs: &[(usize, usize, usize, u8)], n: usize) -> f64 {
    let nf = n as f64;
    let mut sum = 0.0f64;
    for &(from, to, w, _b) in runs {
        // Sum_{k=0}^{w-1} ((from+k)/n - 0.5) * ((to+k)/n - 0.5), correcting x=0 or y=0 where ((0))=0
        let wf = w as f64;
        let a = from as f64 / nf - 0.5;
        let b = to as f64 / nf - 0.5;
        let s1 = wf * (wf - 1.0) * 0.5;
        let s2 = wf * (wf - 1.0) * (2.0 * wf - 1.0) / 6.0;
        let mut block = wf * a * b + (a + b) * s1 / nf + s2 / (nf * nf);
        // Sawtooth correction: ((0)) = 0 instead of -0.5
        if from == 0 {
            // k = 0 had a = -0.5, should contribute 0
            let y0 = if to == 0 { 0.0 } else { b };
            block -= (-0.5) * y0;
        }
        if to == 0 && from != 0 {
            // k = 0 had b = -0.5, should contribute 0
            block -= a * (-0.5);
        }
        sum += block;
    }
    (12.0 / nf) * sum
}

/// Intra-bucket normalized Weyl Erdős-Turán discrepancy ET_H on each class.
/// theta_tilde(x) = (LF(x) - C[L[x]]) / cnt[L[x]] - x / n  (mod 1).
fn weyl_et_by_class(
    l: &[u8],
    lf: &[u32],
    cnt: &[usize; 256],
    first: &[usize; 256],
    row_cls: &[u8],
    row_w: &[u32],
) -> ([f64; 7], f64, f64, f64) {
    const H: usize = 16;
    let n = l.len();
    let nf = n as f64;
    let mut cos_cls = [[0.0f64; H]; 7];
    let mut sin_cls = [[0.0f64; H]; 7];
    let mut n_cls = [0usize; 7];

    let mut cos_thick = [0.0f64; H];
    let mut sin_thick = [0.0f64; H];
    let mut n_thick = 0usize;

    let mut cos_thin = [0.0f64; H];
    let mut sin_thin = [0.0f64; H];
    let mut n_thin = 0usize;

    let mut cos_guid_w1 = [0.0f64; H];
    let mut sin_guid_w1 = [0.0f64; H];
    let mut n_guid_w1 = 0usize;

    for x in 0..n {
        let b = l[x] as usize;
        let c_cnt = cnt[b] as f64;
        let frac_y = (lf[x] as usize - first[b]) as f64 / c_cnt;
        let frac_x = x as f64 / nf;
        let mut th = frac_y - frac_x;
        if th < 0.0 {
            th += 1.0;
        }
        let ang = 2.0 * PI * th;
        let (s1, c1) = ang.sin_cos();
        let cl = row_cls[x] as usize;
        let is_thick = row_w[x] as usize > LIMIT;
        let is_guid_w1 = cl == 0 && row_w[x] == 1;
        n_cls[cl] += 1;
        if is_thick {
            n_thick += 1;
        } else {
            n_thin += 1;
        }
        if is_guid_w1 {
            n_guid_w1 += 1;
        }
        let mut ch = c1;
        let mut sh = s1;
        for h in 0..H {
            cos_cls[cl][h] += ch;
            sin_cls[cl][h] += sh;
            if is_thick {
                cos_thick[h] += ch;
                sin_thick[h] += sh;
            } else {
                cos_thin[h] += ch;
                sin_thin[h] += sh;
            }
            if is_guid_w1 {
                cos_guid_w1[h] += ch;
                sin_guid_w1[h] += sh;
            }
            let next_c = ch * c1 - sh * s1;
            let next_s = sh * c1 + ch * s1;
            ch = next_c;
            sh = next_s;
        }
    }

    let et = |cs: &[f64; H], ss: &[f64; H], m: usize| -> f64 {
        if m == 0 {
            return 0.0;
        }
        let mf = m as f64;
        let mut s = 1.0 / (H as f64 + 1.0);
        for h in 0..H {
            let mag = (cs[h] * cs[h] + ss[h] * ss[h]).sqrt() / mf;
            s += (2.0 / PI) * mag / (h + 1) as f64;
        }
        s
    };

    let mut out_cls = [0.0f64; 7];
    for c in 0..7 {
        out_cls[c] = et(&cos_cls[c], &sin_cls[c], n_cls[c]);
    }
    (
        out_cls,
        et(&cos_thick, &sin_thick, n_thick),
        et(&cos_thin, &sin_thin, n_thin),
        et(&cos_guid_w1, &sin_guid_w1, n_guid_w1),
    )
}

/// MSC 81P45 / 81Q50: Exact MPO spatial-tile metrics at tile size `tile_len`:
/// - mean tile purity IPR = mean_q sum_{q'} (T_{q',q})^2 in [1/L, 1]
/// - distinct non-zero tile sub-operators `chi_distinct` (upper bound on MPO Schmidt rank,
///   exact for disjoint shift/permutation blocks!) and clean-tile fraction (<=2 target tiles).
struct TileQuantumStats {
    mean_ipr: f64,
    clean_tile_frac: f64,
    distinct_blocks: usize,
}

fn measure_tile_quantum(perm: &[u32], tile_len: usize) -> TileQuantumStats {
    let n = perm.len();
    let num_tiles = n / tile_len;
    if num_tiles == 0 {
        return TileQuantumStats {
            mean_ipr: 1.0,
            clean_tile_frac: 1.0,
            distinct_blocks: 0,
        };
    }
    let mut ipr_sum = 0.0f64;
    let mut clean_tiles = 0usize;
    let mut distinct_sigs: HashSet<Vec<(u16, u16)>> = HashSet::new();

    for q in 0..num_tiles {
        let base = q * tile_len;
        let mut by_target: HashMap<usize, Vec<(u16, u16)>> = HashMap::new();
        for r in 0..tile_len {
            let y = perm[base + r] as usize;
            let qp = y / tile_len;
            let rp = (y % tile_len) as u16;
            by_target.entry(qp).or_default().push((r as u16, rp));
        }
        let k = by_target.len();
        if k <= 2 {
            clean_tiles += 1;
        }
        let mut ipr = 0.0f64;
        for (_, entries) in by_target {
            let prob = entries.len() as f64 / tile_len as f64;
            ipr += prob * prob;
            distinct_sigs.insert(entries);
        }
        ipr_sum += ipr;
    }

    TileQuantumStats {
        mean_ipr: ipr_sum / num_tiles as f64,
        clean_tile_frac: clean_tiles as f64 / num_tiles as f64,
        distinct_blocks: distinct_sigs.len(),
    }
}

/// Measure class-conditioned tile purity IPR for a subset mask (rows with mask[x] == true):
/// for each tile with >= tile_len/2 rows in the subset, compute target-tile purity of those rows.
fn measure_class_ipr(perm: &[u32], tile_len: usize, row_cls: &[u8], target_cls: u8) -> (f64, usize) {
    let n = perm.len();
    let num_tiles = n / tile_len;
    let mut ipr_sum = 0.0f64;
    let mut used = 0usize;
    let min_rows = (tile_len / 2).max(2);
    for q in 0..num_tiles {
        let base = q * tile_len;
        let mut counts: HashMap<usize, usize> = HashMap::new();
        let mut m = 0usize;
        for r in 0..tile_len {
            if row_cls[base + r] == target_cls {
                let qp = perm[base + r] as usize / tile_len;
                *counts.entry(qp).or_insert(0) += 1;
                m += 1;
            }
        }
        if m >= min_rows {
            let mut ipr = 0.0f64;
            for &c in counts.values() {
                let p = c as f64 / m as f64;
                ipr += p * p;
            }
            ipr_sum += ipr;
            used += 1;
        }
    }
    let avg = if used > 0 { ipr_sum / used as f64 } else { 0.0 };
    (avg, used)
}

// ============================================================================
// PART 3: General Exact Strip Elimination (Feshbach-Livšic / Rauzy-Veech)
//         + Assembled Chains (`Chain A-Base` & `Chain A-Tube`)
// ============================================================================

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

#[derive(Clone, Copy)]
struct Band {
    end: u32,
    dst: u32,
    code: u32,
}

/// Clean, self-contained, `#![forbid(unsafe_code)]` Feshbach-Livšic / Rauzy-Veech
/// Redheffer strip elimination supporting arbitrary acyclic sets of affine strips
/// (including spirals and both Open<16> thick strips and Kakutani-Tube promoted strips).
struct StripElimResult {
    bands: Vec<Band>,
    spelling: Vec<u8>,
    word_off: Vec<u32>,
    word_len: Vec<u32>,
}

#[derive(Clone, Copy)]
enum Rule {
    Empty,
    Lit(u8),
    Concat(u32, u32),
}

fn eliminate_strips(n: usize, strips: &[Strip]) -> StripElimResult {
    let ns = strips.len();
    let max_nodes = 3 * ns + 4;
    // Doubly-linked lists on side 0 (domain) and side 1 (codomain), sentinel = 0
    let mut prev0 = vec![0u32; max_nodes];
    let mut next0 = vec![0u32; max_nodes];
    let mut prev1 = vec![0u32; max_nodes];
    let mut next1 = vec![0u32; max_nodes];
    let mut width = vec![0u32; max_nodes];
    let mut word = vec![0u32; max_nodes];

    let mut rules: Vec<Rule> = Vec::with_capacity(ns * 2 + 258);
    rules.push(Rule::Empty); // id 0
    for b in 0u8..=255 {
        rules.push(Rule::Lit(b)); // id = 1 + b
    }
    let lit = |b: u8| -> u32 { 1 + b as u32 };

    // Sort indices by `to` for side 1 (codomain)
    let mut by_to: Vec<usize> = (0..ns).collect();
    by_to.sort_unstable_by_key(|&i| strips[i].to);

    let mut nodes_used = (ns + 1) as u32;
    let mut tail0 = 0u32;
    let mut tail1 = 0u32;
    let mut term0: Vec<u32> = Vec::with_capacity(ns + 2); // entry intervals (codomain gaps)
    let mut term1: Vec<u32> = Vec::with_capacity(ns + 2); // exit intervals (domain gaps = J)

    let append0 = |tail: &mut u32, id: u32, p0: &mut [u32], n0: &mut [u32]| {
        let t = *tail as usize;
        let u = id as usize;
        n0[t] = id;
        p0[u] = *tail;
        n0[u] = 0;
        p0[0] = id;
        *tail = id;
    };
    let append1 = |tail: &mut u32, id: u32, p1: &mut [u32], n1: &mut [u32]| {
        let t = *tail as usize;
        let u = id as usize;
        n1[t] = id;
        p1[u] = *tail;
        n1[u] = 0;
        p1[0] = id;
        *tail = id;
    };

    // Build side 0 (domain): strips are sorted by `from`
    let mut x = 0u32;
    let mut total_strip_width = 0u64;
    for (i, st) in strips.iter().enumerate() {
        let sid = (i + 1) as u32;
        width[sid as usize] = st.width;
        word[sid as usize] = lit(st.byte);
        total_strip_width += st.width as u64;
        if x < st.from {
            let gid = nodes_used;
            nodes_used += 1;
            width[gid as usize] = st.from - x;
            word[gid as usize] = 0;
            append0(&mut tail0, gid, &mut prev0, &mut next0);
            term1.push(gid);
        }
        append0(&mut tail0, sid, &mut prev0, &mut next0);
        x = st.from + st.width;
    }
    if (x as usize) < n {
        let gid = nodes_used;
        nodes_used += 1;
        width[gid as usize] = n as u32 - x;
        word[gid as usize] = 0;
        append0(&mut tail0, gid, &mut prev0, &mut next0);
        term1.push(gid);
    }

    // Build side 1 (codomain): strips ordered by `to`
    let mut y = 0u32;
    for &i in &by_to {
        let st = strips[i];
        let sid = (i + 1) as u32;
        if y < st.to {
            let gid = nodes_used;
            nodes_used += 1;
            width[gid as usize] = st.to - y;
            word[gid as usize] = 0;
            append1(&mut tail1, gid, &mut prev1, &mut next1);
            term0.push(gid);
        }
        append1(&mut tail1, sid, &mut prev1, &mut next1);
        y = st.to + st.width;
    }
    if (y as usize) < n {
        let gid = nodes_used;
        width[gid as usize] = n as u32 - y;
        word[gid as usize] = 0;
        append1(&mut tail1, gid, &mut prev1, &mut next1);
        term0.push(gid);
    }

    // Attach terminal lists in opposite side so every band lives in BOTH side 0 and side 1
    // Wait: we follow endpoint.rs exactly:
    // term0 (codomain gaps = entry points) are placed at the FRONT of side 0!
    // term1 (domain gaps = exit points = J) are placed at the FRONT of side 1!
    // Check: before prepending, side 0 starts with n-space intervals of total length n,
    // and side 1 starts with n-space intervals of total length n.
    // Prepending term0 (total width m) to side 0 and term1 (total width m) to side 1
    // gives both sides total width n + m, and peeling from the tail (total_strip_width + n?
    // NO: peeling `remaining = n` from the tail eliminates all `n` coordinates of the
    // original exchange and leaves EXACTLY the `m`-width terminal-to-terminal bands!)
    // Let's prepend term0 to side 0 and term1 to side 1 cleanly:
    {
        let old_head0 = next0[0];
        let mut cur = 0u32;
        for &tid in &term0 {
            next0[cur as usize] = tid;
            prev0[tid as usize] = cur;
            cur = tid;
        }
        next0[cur as usize] = old_head0;
        if old_head0 != 0 {
            prev0[old_head0 as usize] = cur;
        } else {
            prev0[0] = cur;
        }
    }
    {
        let old_head1 = next1[0];
        let mut cur = 0u32;
        for &tid in &term1 {
            next1[cur as usize] = tid;
            prev1[tid as usize] = cur;
            cur = tid;
        }
        next1[cur as usize] = old_head1;
        if old_head1 != 0 {
            prev1[old_head1 as usize] = cur;
        } else {
            prev1[0] = cur;
        }
    }

    let join = |wx: u32, wy: u32, rules: &mut Vec<Rule>| -> u32 {
        if wx == 0 {
            return wy;
        }
        if wy == 0 {
            return wx;
        }
        let id = rules.len() as u32;
        rules.push(Rule::Concat(wx, wy));
        id
    };
    let power = |w: u32, mut q: u32, rules: &mut Vec<Rule>| -> u32 {
        if w == 0 || q == 0 {
            return 0;
        }
        let mut res = 0u32;
        let mut base = w;
        while q > 0 {
            if (q & 1) != 0 {
                res = join(res, base, rules);
            }
            q >>= 1;
            if q > 0 {
                base = join(base, base, rules);
            }
        }
        res
    };

    let mut remaining = n as u64;
    let _ = total_strip_width;

    while remaining > 0 {
        let a = prev0[0];
        let b = prev1[0];
        let aw = width[a as usize];
        let bw = width[b as usize];

        if a == b {
            // Self-matched interval: pop from both tails
            remaining -= aw as u64;
            let pa0 = prev0[a as usize];
            next0[pa0 as usize] = 0;
            prev0[0] = pa0;
            let pa1 = prev1[a as usize];
            next1[pa1 as usize] = 0;
            prev1[0] = pa1;
            continue;
        }

        // Spiral contraction (guarantees O(log n) steps even on self-overlapping strips!)
        if aw > bw && prev1[b as usize] == a && (aw - 1) / bw >= 2 {
            let q = (aw - 1) / bw;
            let head = power(word[a as usize], q, &mut rules);
            word[b as usize] = join(head, word[b as usize], &mut rules);
            width[a as usize] = aw - q * bw;
            remaining -= (q as u64) * (bw as u64);
            continue;
        }
        if aw < bw && prev0[a as usize] == b && (bw - 1) / aw >= 2 {
            let q = (bw - 1) / aw;
            let tail = power(word[b as usize], q, &mut rules);
            word[a as usize] = join(word[a as usize], tail, &mut rules);
            width[b as usize] = bw - q * aw;
            remaining -= (q as u64) * (aw as u64);
            continue;
        }

        let w = aw.min(bw);
        remaining -= w as u64;
        let combined = join(word[a as usize], word[b as usize], &mut rules);

        if aw <= bw {
            // Pop `a` from tail of side 0
            let pa0 = prev0[a as usize];
            next0[pa0 as usize] = 0;
            prev0[0] = pa0;

            if aw < bw {
                // Insert `a` immediately after `b` on side 0; shrink `b`
                let bn0 = next0[b as usize];
                prev0[a as usize] = b;
                next0[a as usize] = bn0;
                next0[b as usize] = a;
                prev0[bn0 as usize] = a;
                word[a as usize] = combined;
                width[b as usize] = bw - aw;
            } else {
                // aw == bw: replace `b` with `a` on side 0, and pop `b` from tail of side 1
                let bp0 = prev0[b as usize];
                let bn0 = next0[b as usize];
                prev0[a as usize] = bp0;
                next0[a as usize] = bn0;
                next0[bp0 as usize] = a;
                prev0[bn0 as usize] = a;
                word[a as usize] = combined;
                let pb1 = prev1[b as usize];
                next1[pb1 as usize] = 0;
                prev1[0] = pb1;
            }
        } else {
            // aw > bw: pop `b` from tail of side 1, insert `b` after `a` on side 1, shrink `a`
            let pb1 = prev1[b as usize];
            next1[pb1 as usize] = 0;
            prev1[0] = pb1;

            let an1 = next1[a as usize];
            prev1[b as usize] = a;
            next1[b as usize] = an1;
            next1[a as usize] = b;
            prev1[an1 as usize] = b;
            word[b as usize] = combined;
            width[a as usize] = aw - bw;
        }
    }

    // Walk side 1 to assign retained destination offsets (0..m)
    let mut dst_coord = vec![0u32; max_nodes];
    let mut pos1 = 0u32;
    let mut cur = next1[0];
    while cur != 0 {
        dst_coord[cur as usize] = pos1;
        pos1 += width[cur as usize];
        cur = next1[cur as usize];
    }

    // Expand rules into compact deduplicated spelling dictionary + bands along side 0
    let mut spelling: Vec<u8> = Vec::with_capacity(65_536);
    let mut word_off: Vec<u32> = Vec::new();
    let mut word_len: Vec<u32> = Vec::new();
    let mut rule_to_code: HashMap<u32, u32> = HashMap::new();
    let mut stack: Vec<u32> = Vec::with_capacity(64);
    let mut bands: Vec<Band> = Vec::with_capacity(ns * 2 + 16);

    let mut pos0 = 0u32;
    let mut cur0 = next0[0];
    while cur0 != 0 {
        let w = width[cur0 as usize];
        let r_id = word[cur0 as usize];
        let code = if let Some(&c) = rule_to_code.get(&r_id) {
            c
        } else {
            let off = spelling.len() as u32;
            stack.clear();
            stack.push(r_id);
            while let Some(top) = stack.pop() {
                match rules[top as usize] {
                    Rule::Empty => {}
                    Rule::Lit(b) => spelling.push(b),
                    Rule::Concat(l_id, r_id2) => {
                        stack.push(r_id2);
                        stack.push(l_id);
                    }
                }
            }
            let len = spelling.len() as u32 - off;
            let c = word_off.len() as u32;
            word_off.push(off);
            word_len.push(len);
            rule_to_code.insert(r_id, c);
            c
        };
        pos0 += w;
        bands.push(Band {
            end: pos0,
            dst: dst_coord[cur0 as usize],
            code,
        });
        cur0 = next0[cur0 as usize];
    }

    StripElimResult {
        bands,
        spelling,
        word_off,
        word_len,
    }
}

#[derive(Clone, Copy, Debug)]
enum TubeMode {
    /// Standard Open<16> thick-only (w > 16)
    ThickOnly,
    /// MSC 37A05 Exact Unbranched Kakutani Thin-Tube promotion:
    /// promote thin runs with `w_min <= w <= 16` whose LF image `L[to..to+w]` is constant!
    UnbranchedThin { w_min: usize },
}

#[derive(Debug, Clone)]
struct AssembledStats {
    m: usize,
    strips_thick: usize,
    strips_tube: usize,
    bands: usize,
    pass1_us: f64,
    tube_check_us: f64,
    elim_us: f64,
    wire_us: f64,
    emit_us: f64,
    total_us: f64,
}

/// Complete Assembled Stepanov Chain for Analogue A:
/// - Fast path: if r == σ, run Steps A1–A6 directly (O(σ) state, 0 reads of L).
/// - General path:
///   A1' (MSC 28/49): Thresholded Thick/Thin Knapsack (+ optional A4' MSC 37A05 Unbranched Thin-Tube)
///   A2' (MSC 20/08): Single-Pass Streaming Bucket Rank Monoid (OutQ)
///   A3' (MSC 15 + 81U20): Feshbach-Livšic / Rauzy-Veech Redheffer S-Matrix Contraction
///   A5' (MSC 20B30): Z/2Z Time-Reversal F_2^b Pointer-XOR Bidirectional Walk (when m >= 65536)
///   A6  (Exact Cycle Closure): Single-cycle or exact periodic short-cycle repetition.
fn invert_assembled_chain(
    l: &[u8],
    p: usize,
    mode: TubeMode,
    use_fast_path: bool,
) -> (Vec<u8>, AssembledStats) {
    let t_total = Instant::now();
    let n = l.len();
    let mut out = vec![0u8; n];
    if n == 0 {
        return (
            out,
            AssembledStats {
                m: 0,
                strips_thick: 0,
                strips_tube: 0,
                bands: 0,
                pass1_us: 0.0,
                tube_check_us: 0.0,
                elim_us: 0.0,
                wire_us: 0.0,
                emit_us: 0.0,
                total_us: 0.0,
            },
        );
    }

    // Fast-path Analogue A recognizer (only if enabled): check if r == σ
    if use_fast_path {
        if let Some(cert) = recognize_perfectly_clustering(l) {
            let t_e = Instant::now();
            emit_perfectly_clustering(&cert, n, p, &mut out);
            let emit_us = t_e.elapsed().as_secs_f64() * 1e6;
            let total_us = t_total.elapsed().as_secs_f64() * 1e6;
            return (
                out,
                AssembledStats {
                    m: 0,
                    strips_thick: cert.sigma,
                    strips_tube: 0,
                    bands: cert.sigma,
                    pass1_us: total_us - emit_us,
                    tube_check_us: 0.0,
                    elim_us: 0.0,
                    wire_us: 0.0,
                    emit_us,
                    total_us,
                },
            );
        }
    }

    // Pass 1 (A1' + A2'): count bytes (or streaming bucket queue)
    let t1 = Instant::now();
    let mut cnt = [0u32; 256];
    let mut first = [0u32; 256];
    if matches!(mode, TubeMode::UnbranchedThin { .. }) {
        // Need `first[b]` before checking `L[to..to+w]` during Pass 1
        // (In bzip2 this histogram is available from RLE2; here we charge the pass honestly!)
        for &b in l {
            cnt[b as usize] += 1;
        }
        let mut acc = 0u32;
        for c in 0..256 {
            first[c] = acc;
            acc += cnt[c];
        }
    }

    let mut occ = [0u32; 256];
    let mut strips: Vec<Strip> = Vec::with_capacity(6144);
    let mut queues: Vec<Vec<u32>> = (0..256).map(|_| Vec::new()).collect();
    let mut strips_thick = 0usize;
    let mut strips_tube = 0usize;
    let mut tube_check_ns = 0u128;

    let mut ret_rank = 0u32;
    let mut mark_rank = 0u32;
    let mut i = 0usize;

    let push_strip = |strips: &mut Vec<Strip>, from: usize, to: u32, w: usize, b: u8| {
        strips.push(Strip {
            from: from as u32,
            to,
            width: w as u32,
            byte: b,
        });
    };

    while i < n {
        let b = l[i];
        let s = i;
        while i < n && l[i] == b {
            i += 1;
        }
        let w = i - s;
        let bu = b as usize;
        let cur_occ = occ[bu];
        occ[bu] = cur_occ + w as u32;

        let mut is_elim = w > LIMIT;
        let mut is_tube = false;

        if !is_elim {
            if let TubeMode::UnbranchedThin { w_min } = mode {
                if w >= w_min {
                    let tt = Instant::now();
                    let to = (first[bu] + cur_occ) as usize;
                    let first_tgt = l[to];
                    let mut all_eq = true;
                    for &tb in &l[to + 1..to + w] {
                        if tb != first_tgt {
                            all_eq = false;
                            break;
                        }
                    }
                    tube_check_ns += tt.elapsed().as_nanos();
                    if all_eq {
                        is_elim = true;
                        is_tube = true;
                    }
                }
            }
        }

        if is_elim {
            // Must never eliminate primary index `p` (guarantees strict acyclicity on single cycles)
            if p >= s && p < i {
                let left_w = p - s;
                let right_w = i - p - 1;
                if left_w > 0 {
                    push_strip(&mut strips, s, cur_occ, left_w, b);
                    if is_tube {
                        strips_tube += 1;
                    } else {
                        strips_thick += 1;
                    }
                }
                // Retain `p`
                mark_rank = ret_rank;
                queues[bu].push(ret_rank);
                ret_rank += 1;
                if right_w > 0 {
                    push_strip(&mut strips, p + 1, cur_occ + left_w as u32 + 1, right_w, b);
                    if is_tube {
                        strips_tube += 1;
                    } else {
                        strips_thick += 1;
                    }
                }
            } else {
                push_strip(&mut strips, s, cur_occ, w, b);
                if is_tube {
                    strips_tube += 1;
                } else {
                    strips_thick += 1;
                }
            }
        } else {
            let q = &mut queues[bu];
            if p >= s && p < i {
                mark_rank = ret_rank + (p - s) as u32;
            }
            for k in 0..w {
                q.push(ret_rank + k as u32);
            }
            ret_rank += w as u32;
        }
    }

    if matches!(mode, TubeMode::ThickOnly) {
        let mut acc = 0u32;
        for c in 0..256 {
            first[c] = acc;
            acc += occ[c];
        }
    }
    // Convert strip `to` from per-byte relative offset to absolute coordinate
    for st in &mut strips {
        st.to += first[st.byte as usize];
    }

    let m = ret_rank as usize;
    let pass1_us = t1.elapsed().as_secs_f64() * 1e6;
    let tube_check_us = tube_check_ns as f64 / 1e3;

    // Stage A3': Feshbach-Livšic / Rauzy-Veech Redheffer strip elimination
    let t_elim = Instant::now();
    let elim = eliminate_strips(n, &strips);
    let elim_us = t_elim.elapsed().as_secs_f64() * 1e6;

    // Stage A2'/A5' wiring: wire retained rows through bands
    let t_wire = Instant::now();
    let use_bi = m >= 65_536;
    let mut edges = vec![0u32; m];
    let mut codes = vec![0u32; m];
    let mut prev_of_mark = 0u32;
    let mut band_idx = 0usize;
    let mut band_start = 0u32;
    let mut entry_pos = 0u32;

    // Each retained row in byte bucket `c` enters `LF(J)` sequentially in sorted byte order!
    // And notice: `bands` partition `[0, m)` (the exact `m` retained entries in order of `LF(J)`!)
    let mut l_ret_byte = vec![0u8; m];
    for c in 0..256 {
        let q = &queues[c];
        let b = c as u8;
        for &src_rank in q {
            while entry_pos >= elim.bands[band_idx].end {
                band_start = elim.bands[band_idx].end;
                band_idx += 1;
            }
            let band = elim.bands[band_idx];
            let dst_rank = band.dst + (entry_pos - band_start);
            l_ret_byte[src_rank as usize] = b;
            codes[src_rank as usize] = band.code;
            if use_bi {
                edges[src_rank as usize] ^= dst_rank;
                edges[dst_rank as usize] ^= src_rank;
                if dst_rank == mark_rank {
                    prev_of_mark = src_rank;
                }
            } else {
                edges[src_rank as usize] = dst_rank;
            }
            entry_pos += 1;
        }
    }
    debug_assert_eq!(entry_pos as usize, m);
    let wire_us = t_wire.elapsed().as_secs_f64() * 1e6;

    // Stage A5' + A6: Emit!
    let t_emit = Instant::now();
    let spelling = &elim.spelling;
    let word_off = &elim.word_off;
    let word_len = &elim.word_len;

    if use_bi {
        // Z/2Z Time-Reversal F_2^b Pointer-XOR Bidirectional Walk!
        // Forward LF cursor starts at `mark_rank` (writing from `out[n]` downward).
        // Backward Psi cursor starts at `prev_of_mark` (writing from `out[0]` upward).
        let mut f_prev = prev_of_mark;
        let mut f_cur = mark_rank;
        let mut b_prev = mark_rank;
        let mut b_cur = prev_of_mark;
        let mut hi = n;
        let mut lo = 0usize;

        while lo < hi {
            if f_cur == b_cur {
                // Odd-length meeting point: emit once on the forward side
                let c = codes[f_cur as usize] as usize;
                let wl = word_len[c] as usize;
                let wo = word_off[c] as usize;
                hi -= 1;
                out[hi] = l_ret_byte[f_cur as usize];
                if wl > 0 {
                    hi -= wl;
                    out[hi..hi + wl].copy_from_slice(&spelling[wo..wo + wl]);
                }
                break;
            }
            // Forward step (writes from hi downward: L[f_cur], then band word left of it)
            let fc = codes[f_cur as usize] as usize;
            let fwl = word_len[fc] as usize;
            let fwo = word_off[fc] as usize;
            hi -= 1;
            out[hi] = l_ret_byte[f_cur as usize];
            if fwl > 0 {
                hi -= fwl;
                out[hi..hi + fwl].copy_from_slice(&spelling[fwo..fwo + fwl]);
            }

            // Backward step (writes from lo upward: band word first, then L[b_cur])
            let bc = codes[b_cur as usize] as usize;
            let bwl = word_len[bc] as usize;
            let bwo = word_off[bc] as usize;
            if bwl > 0 {
                out[lo..lo + bwl].copy_from_slice(&spelling[bwo..bwo + bwl]);
                lo += bwl;
            }
            out[lo] = l_ret_byte[b_cur as usize];
            lo += 1;

            let f_next = edges[f_cur as usize] ^ f_prev;
            let b_next = edges[b_cur as usize] ^ b_prev;
            if f_next == b_cur {
                // Even-length meeting point (or short cycle closed!)
                break;
            }
            f_prev = f_cur;
            f_cur = f_next;
            b_prev = b_cur;
            b_cur = b_next;
        }

        // Exact periodic short-cycle repetition (Step A6 fallback for T = U^k, k >= 2)
        let written = lo + (n - hi);
        if written < n {
            // Compact `[0..lo)` and `[hi..n)` into the single period of length `written`,
            // then tile to `n`!
            let period = written;
            let mut unit = vec![0u8; period];
            let tail_len = n - hi;
            // Notice: out[hi..n] is the END of the period, out[0..lo] is the START of the period!
            unit[..lo].copy_from_slice(&out[..lo]);
            unit[lo..lo + tail_len].copy_from_slice(&out[hi..n]);
            let mut pos = n;
            while pos > 0 {
                let take = pos.min(period);
                let src_start = period - take;
                pos -= take;
                out[pos..pos + take].copy_from_slice(&unit[src_start..period]);
            }
        }
    } else {
        // Single-cursor emit (L2-resident small m < 65,536)
        let mut cur = mark_rank;
        let mut hi = n;
        loop {
            let c = codes[cur as usize] as usize;
            let wl = word_len[c] as usize;
            let wo = word_off[c] as usize;
            if hi == 0 {
                break;
            }
            hi -= 1;
            out[hi] = l_ret_byte[cur as usize];
            if wl > 0 {
                let take = wl.min(hi);
                hi -= take;
                out[hi..hi + take].copy_from_slice(&spelling[wo + (wl - take)..wo + wl]);
            }
            cur = edges[cur as usize];
            if hi == 0 {
                break;
            }
            if cur == mark_rank {
                // Short periodic cycle: repeat the suffix `out[hi..n]` to fill `0..hi`
                let period = n - hi;
                while hi > 0 {
                    let take = hi.min(period);
                    let src_start = n - take;
                    let src_end = n;
                    let dst_start = hi - take;
                    out.copy_within(src_start..src_end, dst_start);
                    hi -= take;
                }
                break;
            }
        }
    }

    let emit_us = t_emit.elapsed().as_secs_f64() * 1e6;
    let total_us = t_total.elapsed().as_secs_f64() * 1e6;

    (
        out,
        AssembledStats {
            m,
            strips_thick,
            strips_tube,
            bands: elim.bands.len(),
            pass1_us,
            tube_check_us,
            elim_us,
            wire_us,
            emit_us,
            total_us,
        },
    )
}

// ============================================================================
// PART 4: Synthetic Non-Wikidata & Exhaustive Test Suite
// ============================================================================

fn make_synthetic_json_logs(num_records: usize) -> Vec<u8> {
    let mut state = 0x1234_5678_9ABC_DEF0u64;
    let mut next_u64 = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        state
    };
    let hex_lo = b"0123456789abcdef";
    let hex_hi = b"0123456789ABCDEF";
    // Pool of 60 repeated sha1 hashes (simulating repeated references / build SHAs)
    let mut sha_pool: Vec<String> = Vec::with_capacity(60);
    for _ in 0..60 {
        let mut s = String::with_capacity(40);
        for _ in 0..40 {
            s.push(hex_lo[(next_u64() as usize) & 15] as char);
        }
        sha_pool.push(s);
    }
    let services = ["auth-api", "billing-worker", "search-edge", "ingest-pipe"];
    let levels = ["INFO", "WARN", "ERROR"];

    let mut out = Vec::with_capacity(num_records * 220);
    for idx in 0..num_records {
        // Unique UUID
        let mut uuid = String::with_capacity(36);
        for k in 0..36 {
            if matches!(k, 8 | 13 | 18 | 23) {
                uuid.push('-');
            } else {
                uuid.push(hex_hi[(next_u64() as usize) & 15] as char);
            }
        }
        let sha = &sha_pool[(next_u64() as usize) % sha_pool.len()];
        let svc = services[(next_u64() as usize) % services.len()];
        let lvl = levels[(next_u64() as usize) % levels.len()];
        let ms = (next_u64() % 900) + 10;
        let line = format!(
            "{{\"ts\":\"2026-11-22T04:{:02}:{:02}Z\",\"level\":\"{}\",\"service\":\"{}\",\"trace_id\":\"{}\",\"commit\":\"{}\",\"status\":200,\"latency_ms\":{}}}\n",
            (idx / 60) % 60,
            idx % 60,
            lvl,
            svc,
            uuid,
            sha,
            ms
        );
        out.extend_from_slice(line.as_bytes());
    }
    out
}

fn run_small_and_synthetic_verification() -> (usize, usize) {
    let mut total_cases = 0usize;
    let mut perfect_cases = 0usize;

    let check = |l: &[u8], p: usize, label: &str| {
        let ref_out = invert_reference(l, p);
        for mode in [
            TubeMode::ThickOnly,
            TubeMode::UnbranchedThin { w_min: 2 },
            TubeMode::UnbranchedThin { w_min: 4 },
        ] {
            let (got, _) = invert_assembled_chain(l, p, mode, true);
            assert_eq!(
                got, ref_out,
                "MISMATCH on {label} (n={}, p={}, mode={mode:?})",
                l.len(),
                p
            );
            let (got_nf, _) = invert_assembled_chain(l, p, mode, false);
            assert_eq!(
                got_nf, ref_out,
                "MISMATCH (no-fast) on {label} (n={}, p={}, mode={mode:?})",
                l.len(),
                p
            );
        }
    };

    // 1. Exhaustive binary columns n = 1..=11 (4,094 columns)
    for n in 1..=11usize {
        for mask in 0..(1usize << n) {
            let mut l = vec![0u8; n];
            for k in 0..n {
                l[k] = if ((mask >> k) & 1) != 0 { b'1' } else { b'0' };
            }
            let p = mask % n;
            check(&l, p, "exhaustive-bin");
            if recognize_perfectly_clustering(&l).is_some() {
                let got = invert_perfectly_clustering(&l, p).unwrap();
                assert_eq!(got, invert_reference(&l, p));
                perfect_cases += 1;
            }
            total_cases += 1;
        }
    }

    // 2. Exhaustive ternary columns n = 1..=6 (1,092 columns)
    for n in 1..=6usize {
        let mut pow3 = 1usize;
        for _ in 0..n {
            pow3 *= 3;
        }
        for code in 0..pow3 {
            let mut l = vec![0u8; n];
            let mut v = code;
            for slot in &mut l {
                *slot = b'a' + (v % 3) as u8;
                v /= 3;
            }
            let p = code % n;
            check(&l, p, "exhaustive-tern");
            if recognize_perfectly_clustering(&l).is_some() {
                let got = invert_perfectly_clustering(&l, p).unwrap();
                assert_eq!(got, invert_reference(&l, p));
                perfect_cases += 1;
            }
            total_cases += 1;
        }
    }

    // 3. Periodic texts T = U^k (multi-cycle LF!)
    let bases: &[&[u8]] = &[
        b"ab",
        b"abc",
        b"abracadabra",
        b"{\"k\":1},",
        b"0123456789abcdef",
    ];
    for base in bases {
        for rep in [2usize, 3, 5, 19] {
            let mut t = Vec::with_capacity(base.len() * rep);
            for _ in 0..rep {
                t.extend_from_slice(base);
            }
            let (l, p) = forward_bwt_small(&t);
            check(&l, p, "periodic");
            let (got, _) = invert_assembled_chain(&l, p, TubeMode::ThickOnly, true);
            assert_eq!(got, t, "periodic roundtrip failed");
            total_cases += 1;
        }
    }

    // 4. Random texts & malformed non-BWT columns (300 cases)
    let mut rng = 0xCAFE_BABE_1337_0001u64;
    let mut nxt = || {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        rng
    };
    for case in 0..300usize {
        let n = 30 + (nxt() as usize % 300);
        let alpha = [2usize, 4, 16, 64][case % 4];
        let mut l = vec![0u8; n];
        for slot in &mut l {
            *slot = b'!' + (nxt() as usize % alpha) as u8;
        }
        let p = nxt() as usize % n;
        check(&l, p, "random-malformed");
        total_cases += 1;
    }

    // 5. Perfectly clustering BWTs (σ = 2, 3, 4, 8, 16, 32, 64)
    for &sigma in &[2usize, 3, 4, 8, 16, 32, 64] {
        let (l, p) = make_perfectly_clustering_bwt(sigma, 600, 100 + sigma as u64);
        let got_pc = invert_perfectly_clustering(&l, p).expect("must recognize perfectly clustering");
        let ref_out = invert_reference(&l, p);
        assert_eq!(got_pc, ref_out);
        // Verify true BWT round-trip on small sizes!
        let (l2, p2) = forward_bwt_small(&got_pc);
        assert_eq!(l2, l, "BWT roundtrip mismatch on sigma={sigma}");
        assert_eq!(invert_reference(&l2, p2), got_pc);
        check(&l, p, "perfect-cluster-small");
        perfect_cases += 1;
        total_cases += 1;
    }

    // 6. Synthetic non-Wikidata JSON logs (with UUIDs + repeated sha1 hashes)
    {
        let logs = make_synthetic_json_logs(15);
        let (l, p) = forward_bwt_small(&logs);
        check(&l, p, "synthetic-json-logs-small");
        let (got, _) = invert_assembled_chain(&l, p, TubeMode::UnbranchedThin { w_min: 2 }, true);
        assert_eq!(got, logs, "synthetic JSON logs roundtrip failed");
        total_cases += 1;
    }

    (total_cases, perfect_cases)
}

// ============================================================================
// MAIN: Run All 3 Parts and Print Verbatim Tables
// ============================================================================

fn main() {
    println!("=== STEPANOV ANALOGUE A (PERFECTLY CLUSTERING BWTs, r = |Sigma|) — CYCLE 46 P1 ===\n");

    // ------------------------------------------------------------------------
    // PART 0: Exhaustive & Synthetic Verification Gate
    // ------------------------------------------------------------------------
    let t_ver = Instant::now();
    let (total_small, perfect_small) = run_small_and_synthetic_verification();
    println!(
        "[VERIFIED] Synthetic & Exhaustive Suite: {} cases (incl. {} perfectly clustering, periodic T=U^k, malformed non-BWT, synthetic JSON+UUID+sha1 logs) — 100% BYTE-EXACT across ThickOnly + KakutaniTube(w>=2, w>=4) in {:.1} ms\n",
        total_small,
        perfect_small,
        t_ver.elapsed().as_secs_f64() * 1e3
    );

    // ------------------------------------------------------------------------
    // PART 1: Solved World — Perfectly Clustering BWTs (r = |Sigma|) at Scale
    // ------------------------------------------------------------------------
    println!("--- PART 1: SOLVED ANALOGUE A (PERFECTLY CLUSTERING BWTs, r = sigma) ---");
    println!(
        "{:<14} {:>8} {:>5} {:>5} {:>6} {:>9} {:>9} {:>9} {:>8} {:>8} {:>8}",
        "instance", "n", "sigma", "r", "D_mod", "rho_DR", "IPR_16", "chi16", "PC_us", "LF_us", "PC/LF"
    );
    for &(name, sigma, target_n, seed) in &[
        ("Fib-F30", 2usize, 832_040usize, 1u64),
        ("SymIET-s4", 4, 900_000, 42),
        ("SymIET-s16", 16, 900_000, 160),
        ("SymIET-s64", 64, 900_000, 640),
    ] {
        let (l, p) = make_perfectly_clustering_bwt(sigma, target_n, seed);
        let n = l.len();
        let (_, _, lf) = build_lf(&l);
        // Compute runs for O(r) DR
        let mut runs = Vec::new();
        let mut i = 0;
        while i < n {
            let b = l[i];
            let s = i;
            while i < n && l[i] == b {
                i += 1;
            }
            runs.push((s, lf[s] as usize, i - s, b));
        }
        let mut shifts = HashSet::new();
        for &(s, to, _, _) in &runs {
            shifts.insert((to as i64 - s as i64).rem_euclid(n as i64));
        }
        let rho = dedekind_rademacher_or(&runs, n);
        let tq16 = measure_tile_quantum(&lf, 16);

        // Warmup + median of 5 timings
        let ref_out = invert_reference(&l, p);
        let got_pc = invert_perfectly_clustering(&l, p).unwrap();
        assert_eq!(got_pc, ref_out, "Solved world mismatch on {name}");

        let mut pc_times = Vec::new();
        let mut lf_times = Vec::new();
        for _ in 0..5 {
            let t0 = Instant::now();
            let o1 = invert_perfectly_clustering(&l, p).unwrap();
            pc_times.push(t0.elapsed().as_secs_f64() * 1e6);
            assert_eq!(o1[0], ref_out[0]);

            let t1 = Instant::now();
            let o2 = invert_reference(&l, p);
            lf_times.push(t1.elapsed().as_secs_f64() * 1e6);
            assert_eq!(o2[0], ref_out[0]);
        }
        pc_times.sort_by(|a, b| a.total_cmp(b));
        lf_times.sort_by(|a, b| a.total_cmp(b));
        let pc_med = pc_times[2];
        let lf_med = lf_times[2];

        println!(
            "{:<14} {:>8} {:>5} {:>5} {:>6} {:>9.4} {:>9.4} {:>9} {:>8.1} {:>8.1} {:>7.3}x",
            name,
            n,
            sigma,
            runs.len(),
            shifts.len(),
            rho,
            tq16.mean_ipr,
            tq16.distinct_blocks,
            pc_med,
            lf_med,
            pc_med / lf_med
        );
    }
    println!();

    // ------------------------------------------------------------------------
    // PART 2: Exact Failure-Localization Diagnostics on All 6 Wikidata Blocks
    // ------------------------------------------------------------------------
    let mut loaded_blocks: Vec<(&str, Vec<u8>, usize, Vec<u8>)> = Vec::new();
    for spec in WIKIDATA_BLOCKS {
        let l = fs::read(spec.path).expect("failed to read Wikidata block");
        let t = invert_reference(&l, spec.primary_index);
        loaded_blocks.push((spec.name, l, spec.primary_index, t));
    }

    println!("--- PART 2A: STEP A1 FAILURE — RUN FRAGMENTATION & 7-CLASS OWNERSHIP ---");
    println!(
        "{:<10} {:>7} {:>4} {:>7} {:>7} {:>5} | {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} (all bytes %)",
        "block", "n", "sig", "r", "r/sig", "rc=1",
        CLASS_NAMES[0], CLASS_NAMES[1], CLASS_NAMES[2], CLASS_NAMES[3],
        CLASS_NAMES[4], CLASS_NAMES[5], CLASS_NAMES[6]
    );

    // Store per-block summaries for subsequent compact tables
    struct BlockDiag {
        name: &'static str,
        n: usize,
        r: usize,
        m_open: usize,
        w1_rows: usize,
        thin2_16_rows: usize,
        thin_pct_cls: [f64; 7],
        w1_pct_cls: [f64; 7],
        thin2_16_pct_cls: [f64; 7],
        // Step A2 (MSC 11)
        d_mod: usize,
        max_shift_mult: usize,
        rho_dr: f64,
        et_cls: [f64; 7],
        et_thick: f64,
        et_thin: f64,
        et_guid_w1: f64,
        // Step A3 (Coverage + MSC 81 MPO)
        cov_sig: f64,
        cov_256: f64,
        cov_2048: f64,
        cov_thick: f64,
        h_thick: usize,
        lf_ipr16: f64,
        lf_chi16: usize,
        lf_clean16: f64,
        pij_ipr16: f64,
        pij_chi16: usize,
        ipr16_guid: f64,
        ipr16_sha1: f64,
        ipr16_key: f64,
        // Step A4 (MSC 11M41 Dirichlet + MSC 81U20/81Q10 + MSC 37D25/37A05)
        z1: f64,
        kappa_j: f64,
        tau1_frac: f64,
        tau1_guid_frac: f64,
        tau1_sha1_frac: f64,
        ws_trace_ok: bool,
        jordan_blocks: usize,
        jordan_max: usize,
        jordan_ge16_rows_pct: f64,
        gk_gamma1: f64,
        gk_gamma5: f64,
        gk_gamma15: f64,
        gk_var_ratio32: f64,
        // Unbranched Kakutani Thin-Skyscraper (2 <= w <= 16)
        unbr_runs_w2: usize,
        unbr_rows_w2: usize,
        unbr_sha1_cov_pct: f64,
        unbr_runs_w4: usize,
        unbr_rows_w4: usize,
        sky_h4_chains: usize,
        sky_h4_rows: usize,
        sky_h4_steps: usize,
        // Step A5 & A6 + Classical MacMahon 1915 descents + Step A1 early-exit cost
        cl_jump_pct: f64,
        page_jump_pct: f64,
        lf_cycles: usize,
        pij_cycles: usize,
        des_lf_pct: f64,
        des_pij_pct: f64,
        rej_byte: usize,
        rej_run: usize,
        recog_ns: f64,
    }
    let mut diags: Vec<BlockDiag> = Vec::new();

    for (bidx, (name, l, p, t)) in loaded_blocks.iter().enumerate() {
        let n = l.len();
        let p = *p;
        let (cnt, first, lf) = build_lf(l);
        let t_cls = classify_text_bytes(t);

        // Map BWT row -> text position (also checks Step A6 cycle count of LF!)
        let mut row_pos = vec![0usize; n];
        let mut cur = p;
        let mut cyc_len = 0usize;
        for i in (0..n).rev() {
            row_pos[cur] = i;
            cur = lf[cur] as usize;
            cyc_len += 1;
        }
        let lf_cycles = if cur == p && cyc_len == n { 1 } else { 2 };

        let mut row_cls = vec![0u8; n];
        for x in 0..n {
            row_cls[x] = t_cls[row_pos[x]];
        }

        // Runs & widths
        let mut runs: Vec<(usize, usize, usize, u8)> = Vec::new();
        let mut runs_per_byte = [0usize; 256];
        let mut row_w = vec![0u32; n];
        let mut row_run_id = vec![0u32; n];
        let mut i = 0usize;
        while i < n {
            let b = l[i];
            let s = i;
            while i < n && l[i] == b {
                i += 1;
            }
            let w = i - s;
            let rid = runs.len() as u32;
            runs.push((s, lf[s] as usize, w, b));
            runs_per_byte[b as usize] += 1;
            for x in s..i {
                row_w[x] = w as u32;
                row_run_id[x] = rid;
            }
        }
        let r = runs.len();
        let sigma = cnt.iter().filter(|&&c| c > 0).count();
        let rc_eq_1 = runs_per_byte.iter().filter(|&&c| c == 1).count();

        // 7-class counts
        let mut all_c = [0usize; 7];
        let mut thin_c = [0usize; 7];
        let mut w1_c = [0usize; 7];
        let mut thin2_16_c = [0usize; 7];
        let mut thin_tot = 0usize;
        let mut w1_tot = 0usize;
        let mut thin2_16_tot = 0usize;

        for x in 0..n {
            let c = row_cls[x] as usize;
            all_c[c] += 1;
            let w = row_w[x] as usize;
            if w <= LIMIT {
                thin_c[c] += 1;
                thin_tot += 1;
                if w == 1 {
                    w1_c[c] += 1;
                    w1_tot += 1;
                } else {
                    thin2_16_c[c] += 1;
                    thin2_16_tot += 1;
                }
            }
        }

        println!(
            "{:<10} {:>7} {:>4} {:>7} {:>7.1} {:>5} | {:>5.1}% {:>5.1}% {:>5.1}% {:>5.1}% {:>5.1}% {:>5.1}% {:>5.1}%",
            name,
            n,
            sigma,
            r,
            r as f64 / sigma as f64,
            rc_eq_1,
            100.0 * all_c[0] as f64 / n as f64,
            100.0 * all_c[1] as f64 / n as f64,
            100.0 * all_c[2] as f64 / n as f64,
            100.0 * all_c[3] as f64 / n as f64,
            100.0 * all_c[4] as f64 / n as f64,
            100.0 * all_c[5] as f64 / n as f64,
            100.0 * all_c[6] as f64 / n as f64
        );

        let pct = |arr: [usize; 7], tot: usize| -> [f64; 7] {
            let mut res = [0.0; 7];
            for k in 0..7 {
                res[k] = if tot > 0 { 100.0 * arr[k] as f64 / tot as f64 } else { 0.0 };
            }
            res
        };

        // Step A2: Modular shift cardinality & Dedekind-Rademacher & Weyl ET_16
        let mut shift_counts: HashMap<i64, usize> = HashMap::with_capacity(r);
        for &(s, to, _, _) in &runs {
            let q = (to as i64 - s as i64).rem_euclid(n as i64);
            *shift_counts.entry(q).or_insert(0) += 1;
        }
        let d_mod = shift_counts.len();
        let max_shift_mult = shift_counts.values().copied().max().unwrap_or(0);
        let rho_dr = dedekind_rademacher_or(&runs, n);
        let (et_cls, et_thick, et_thin, et_guid_w1) =
            weyl_et_by_class(l, &lf, &cnt, &first, &row_cls, &row_w);

        // Step A3: Top-K run coverage & MSC 81P45 MPO tile metrics
        let mut widths_sorted: Vec<usize> = runs.iter().map(|&(_, _, w, _)| w).collect();
        widths_sorted.sort_unstable_by(|a, b| b.cmp(a));
        let top_cov = |k: usize| -> f64 {
            let s: usize = widths_sorted.iter().take(k).sum();
            100.0 * s as f64 / n as f64
        };
        let h_thick = runs.iter().filter(|&&(_, _, w, _)| w > LIMIT).count();
        let cov_thick: f64 = 100.0
            * runs
                .iter()
                .filter(|&&(_, _, w, _)| w > LIMIT)
                .map(|&(_, _, w, _)| w)
                .sum::<usize>() as f64
            / n as f64;

        let tq_lf16 = measure_tile_quantum(&lf, 16);
        let (ipr16_guid, _) = measure_class_ipr(&lf, 16, &row_cls, 0);
        let (ipr16_sha1, _) = measure_class_ipr(&lf, 16, &row_cls, 1);
        let (ipr16_key, _) = measure_class_ipr(&lf, 16, &row_cls, 4);

        // Build exact Open<16> retained set J, return map pi_J, and return times tau_J
        let mut is_ret = vec![false; n];
        let mut ret_id = vec![u32::MAX; n];
        let mut ret_orig: Vec<usize> = Vec::with_capacity(thin_tot + 1);
        for x in 0..n {
            if (row_w[x] as usize) <= LIMIT || x == p {
                is_ret[x] = true;
                ret_id[x] = ret_orig.len() as u32;
                ret_orig.push(x);
            }
        }
        let m_open = ret_orig.len();
        let mut pi_j = vec![0u32; m_open];
        let mut tau_j = vec![0u32; m_open];
        let mut ws_trace = 0usize;
        for (j, &orig_x) in ret_orig.iter().enumerate() {
            let mut y = lf[orig_x] as usize;
            let mut steps = 1u32;
            while !is_ret[y] {
                y = lf[y] as usize;
                steps += 1;
            }
            pi_j[j] = ret_id[y];
            tau_j[j] = steps;
            ws_trace += (steps - 1) as usize;
        }
        let ws_trace_ok = ws_trace == n - m_open;

        // Check Step A6 cycle count of pi_J
        let mut pij_cur = ret_id[p] as usize;
        let mut pij_len = 0usize;
        loop {
            pij_cur = pi_j[pij_cur] as usize;
            pij_len += 1;
            if pij_cur == ret_id[p] as usize || pij_len > m_open {
                break;
            }
        }
        let pij_cycles = if pij_len == m_open { 1 } else { 2 };

        let tq_pij16 = measure_tile_quantum(&pi_j, 16);

        // MSC 11M41 Dirichlet Z_J(1), kappa_J, tau=1 fractions
        let mut z1 = 0.0f64;
        let mut tau1_tot = 0usize;
        let mut tau1_guid = 0usize;
        let mut guid_in_j = 0usize;
        let mut tau1_sha1 = 0usize;
        let mut sha1_in_j = 0usize;
        for j in 0..m_open {
            let tj = tau_j[j];
            z1 += 1.0 / tj as f64;
            let cl = row_cls[ret_orig[j]];
            if cl == 0 {
                guid_in_j += 1;
            } else if cl == 1 {
                sha1_in_j += 1;
            }
            if tj == 1 {
                tau1_tot += 1;
                if cl == 0 {
                    tau1_guid += 1;
                } else if cl == 1 {
                    tau1_sha1 += 1;
                }
            }
        }
        let kappa_j = (n as f64 * z1) / ((m_open as f64) * (m_open as f64));

        // MSC 81Q10 Jordan blocks of truncated scattering core U_JJ (consecutive tau_J == 1 bursts along orbit!)
        // + MSC 37D25 Green-Kubo autocorrelation of tau_J along the orbit of pi_J!
        let mut orbit_tau = Vec::with_capacity(m_open);
        let mut jcur = ret_id[p] as usize;
        for _ in 0..m_open {
            orbit_tau.push(tau_j[jcur] as f64);
            jcur = pi_j[jcur] as usize;
        }
        let mut jordan_lens: Vec<usize> = Vec::new();
        let mut run_len = 0usize;
        for &tv in &orbit_tau {
            if tv == 1.0 {
                run_len += 1;
            } else if run_len > 0 {
                jordan_lens.push(run_len);
                run_len = 0;
            }
        }
        if run_len > 0 {
            jordan_lens.push(run_len);
        }
        let jordan_max = jordan_lens.iter().copied().max().unwrap_or(0);
        let jordan_ge16_rows: usize = jordan_lens.iter().copied().filter(|&l| l >= 16).sum();
        let jordan_ge16_rows_pct = if tau1_tot > 0 {
            100.0 * jordan_ge16_rows as f64 / tau1_tot as f64
        } else {
            0.0
        };

        // Green-Kubo autocorrelation & K=32 window variance ratio
        let mean_tau = n as f64 / m_open as f64;
        let var_tau: f64 = orbit_tau
            .iter()
            .map(|&v| (v - mean_tau) * (v - mean_tau))
            .sum::<f64>()
            / m_open as f64;
        let autocorr = |lag: usize| -> f64 {
            let mut s = 0.0f64;
            for k in 0..m_open {
                s += (orbit_tau[k] - mean_tau) * (orbit_tau[(k + lag) % m_open] - mean_tau);
            }
            (s / m_open as f64) / var_tau.max(1e-12)
        };
        let gk_gamma1 = autocorr(1);
        let gk_gamma5 = autocorr(5);
        let gk_gamma15 = autocorr(15);
        let mut gk_var_ratio32 = 1.0f64;
        for lag in 1..32usize {
            let w_bart = 1.0 - (lag as f64) / 32.0;
            gk_var_ratio32 += 2.0 * w_bart * autocorr(lag);
        }

        // MSC 37A05: Unbranched Kakutani Thin-Skyscraper Census (2 <= w <= 16)
        let mut unbr_runs_w2 = 0usize;
        let mut unbr_rows_w2 = 0usize;
        let mut unbr_sha1_rows_w2 = 0usize;
        let mut unbr_runs_w4 = 0usize;
        let mut unbr_rows_w4 = 0usize;
        let mut is_unbr_w2 = vec![false; r];
        let mut next_run_same_w = vec![u32::MAX; r];
        let mut has_pred_same_w = vec![false; r];

        for (rid, &(s, to, w, _)) in runs.iter().enumerate() {
            if w >= 2 && w <= LIMIT {
                let first_b = l[to];
                let all_eq = l[to + 1..to + w].iter().all(|&tb| tb == first_b);
                if all_eq {
                    is_unbr_w2[rid] = true;
                    unbr_runs_w2 += 1;
                    unbr_rows_w2 += w;
                    for x in s..s + w {
                        if row_cls[x] == 1 {
                            unbr_sha1_rows_w2 += 1;
                        }
                    }
                    if w >= 4 {
                        unbr_runs_w4 += 1;
                        unbr_rows_w4 += w;
                    }
                    // Check exact width-preserving skyscraper link to next thin run
                    let tgt_rid = row_run_id[to] as usize;
                    if runs[tgt_rid].2 == w && tgt_rid != rid {
                        next_run_same_w[rid] = tgt_rid as u32;
                        has_pred_same_w[tgt_rid] = true;
                    }
                }
            }
        }
        // Trace maximal exact width-preserving skyscrapers of height >= 4
        let mut sky_h4_chains = 0usize;
        let mut sky_h4_rows = 0usize;
        let mut sky_h4_steps = 0usize;
        let mut visited_rid = vec![false; r];
        for rid in 0..r {
            if is_unbr_w2[rid] && !has_pred_same_w[rid] && !visited_rid[rid] {
                let w = runs[rid].2;
                let mut cur_r = rid;
                let mut h = 0usize;
                while cur_r != u32::MAX as usize && !visited_rid[cur_r] {
                    visited_rid[cur_r] = true;
                    h += 1;
                    let nxt = next_run_same_w[cur_r];
                    if nxt == u32::MAX {
                        break;
                    }
                    cur_r = nxt as usize;
                }
                if h >= 4 {
                    sky_h4_chains += 1;
                    sky_h4_rows += h * w;
                    sky_h4_steps += h;
                }
            }
        }
        let unbr_sha1_cov_pct = if thin2_16_c[1] > 0 {
            100.0 * unbr_sha1_rows_w2 as f64 / thin2_16_c[1] as f64
        } else {
            0.0
        };

        // Step A5: 64B cache-line jump rate & 32 KiB L1-page jump rate of pi_J (u32 edges = 16/line, 8192/page)
        let mut cl_jumps = 0usize;
        let mut pg_jumps = 0usize;
        for (j, &dst) in pi_j.iter().enumerate() {
            if (j >> 4) != (dst as usize >> 4) {
                cl_jumps += 1;
            }
            if (j >> 13) != (dst as usize >> 13) {
                pg_jumps += 1;
            }
        }

        // Classical MacMahon (1915, MSC 05A05) permutation descent density
        let des_lf = lf.windows(2).filter(|w| w[0] > w[1]).count();
        let des_pij = pi_j.windows(2).filter(|w| w[0] > w[1]).count();
        let des_lf_pct = 100.0 * des_lf as f64 / n as f64;
        let des_pij_pct = 100.0 * des_pij as f64 / m_open as f64;

        // Exact Step A1 early-exit rejection point and C_recognise cost
        let (rej_byte, rej_run) = {
            let mut seen = [false; 256];
            let mut idx = 0usize;
            let mut run_idx = 0usize;
            let mut found = (n, r);
            while idx < n {
                let b = l[idx];
                if seen[b as usize] {
                    found = (idx, run_idx);
                    break;
                }
                seen[b as usize] = true;
                while idx < n && l[idx] == b {
                    idx += 1;
                }
                run_idx += 1;
            }
            found
        };
        let t_rec = Instant::now();
        for _ in 0..1000 {
            let _ = std::hint::black_box(recognize_perfectly_clustering(l));
        }
        let recog_ns = t_rec.elapsed().as_nanos() as f64 / 1000.0;

        diags.push(BlockDiag {
            name: WIKIDATA_BLOCKS[bidx].name,
            n,
            r,
            m_open,
            w1_rows: w1_tot,
            thin2_16_rows: thin2_16_tot,
            thin_pct_cls: pct(thin_c, thin_tot),
            w1_pct_cls: pct(w1_c, w1_tot),
            thin2_16_pct_cls: pct(thin2_16_c, thin2_16_tot),
            d_mod,
            max_shift_mult,
            rho_dr,
            et_cls,
            et_thick,
            et_thin,
            et_guid_w1,
            cov_sig: top_cov(sigma),
            cov_256: top_cov(256),
            cov_2048: top_cov(2048),
            cov_thick,
            h_thick,
            lf_ipr16: tq_lf16.mean_ipr,
            lf_chi16: tq_lf16.distinct_blocks,
            lf_clean16: tq_lf16.clean_tile_frac,
            pij_ipr16: tq_pij16.mean_ipr,
            pij_chi16: tq_pij16.distinct_blocks,
            ipr16_guid,
            ipr16_sha1,
            ipr16_key,
            z1,
            kappa_j,
            tau1_frac: 100.0 * tau1_tot as f64 / m_open as f64,
            tau1_guid_frac: if guid_in_j > 0 { 100.0 * tau1_guid as f64 / guid_in_j as f64 } else { 0.0 },
            tau1_sha1_frac: if sha1_in_j > 0 { 100.0 * tau1_sha1 as f64 / sha1_in_j as f64 } else { 0.0 },
            ws_trace_ok,
            jordan_blocks: jordan_lens.len(),
            jordan_max,
            jordan_ge16_rows_pct,
            gk_gamma1,
            gk_gamma5,
            gk_gamma15,
            gk_var_ratio32,
            unbr_runs_w2,
            unbr_rows_w2,
            unbr_sha1_cov_pct,
            unbr_runs_w4,
            unbr_rows_w4,
            sky_h4_chains,
            sky_h4_rows,
            sky_h4_steps,
            cl_jump_pct: 100.0 * cl_jumps as f64 / m_open as f64,
            page_jump_pct: 100.0 * pg_jumps as f64 / m_open as f64,
            lf_cycles,
            pij_cycles,
            des_lf_pct,
            des_pij_pct,
            rej_byte,
            rej_run,
            recog_ns,
        });
    }

    println!("\n--- PART 2B: FULL 7-CLASS WORKING-BACKWARDS THIN / WIDTH-1 / 2..16 SPLIT ---");
    println!(
        "{:<10} {:>7} {:>6} | {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} (thin %) | {:>6} {:>5} {:>5} (w1) | {:>6} {:>5} (2..16:S)",
        "block", "m_Open", "m/n%",
        "G", "S", "E", "N", "K", "T", "St",
        "w1#", "w1:G", "w1:S",
        "2..16#", "2:S"
    );
    for d in &diags {
        println!(
            "{:<10} {:>7} {:>5.1}% | {:>4.1}% {:>4.1}% {:>4.1}% {:>4.1}% {:>4.1}% {:>4.1}% {:>4.1}% | {:>6} {:>4.1}% {:>4.1}% | {:>6} {:>4.1}%",
            d.name,
            d.m_open,
            100.0 * d.m_open as f64 / d.n as f64,
            d.thin_pct_cls[0], d.thin_pct_cls[1], d.thin_pct_cls[2], d.thin_pct_cls[3],
            d.thin_pct_cls[4], d.thin_pct_cls[5], d.thin_pct_cls[6],
            d.w1_rows, d.w1_pct_cls[0], d.w1_pct_cls[1],
            d.thin2_16_rows, d.thin2_16_pct_cls[1]
        );
    }

    println!("\n--- PART 2G: CLASSICAL MACMAHON (1915) DESCENTS & STEP A1 EARLY-EXIT C_RECOGNISE ---");
    println!(
        "{:<10} {:>8} {:>9} | {:>8} {:>7} {:>10}",
        "block", "desLF%", "desPiJ%", "rejByte", "rejRun", "C_rec_ns"
    );
    for d in &diags {
        println!(
            "{:<10} {:>7.2}% {:>8.2}% | {:>8} {:>7} {:>9.1} ns",
            d.name, d.des_lf_pct, d.des_pij_pct, d.rej_byte, d.rej_run, d.recog_ns
        );
    }

    println!("\n--- PART 2C: STEP A2 FAILURE (MSC 11: MODULAR SHIFTS, DEDEKIND-RADEMACHER, WEYL ET_16) ---");
    println!(
        "{:<10} {:>7} {:>6} {:>5} {:>8} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "block", "D_mod", "Dm/r%", "maxM", "rho_DR",
        "ET:thk", "ET:thn", "ET:G_w1", "ET:sha1", "ET:key", "ET:ent"
    );
    for d in &diags {
        println!(
            "{:<10} {:>7} {:>5.1}% {:>5} {:>8.4} | {:>7.4} {:>7.4} {:>7.4} {:>7.4} {:>7.4} {:>7.4}",
            d.name,
            d.d_mod,
            100.0 * d.d_mod as f64 / d.r as f64,
            d.max_shift_mult,
            d.rho_dr,
            d.et_thick,
            d.et_thin,
            d.et_guid_w1,
            d.et_cls[1],
            d.et_cls[4],
            d.et_cls[2]
        );
    }

    println!("\n--- PART 2D: STEP A3 FAILURE (COVERAGE + MSC 81P45 MPO TILE SCHMIDT & PURITY IPR_16) ---");
    println!(
        "{:<10} {:>6} {:>6} {:>6} {:>6} {:>5} | {:>6} {:>6} {:>6} | {:>6} {:>6} | {:>6} {:>6} {:>6}",
        "block", "topS%", "t256%", "t2K%", "thk%", "H_thk",
        "LF:IPR", "LF:cln", "LF:chi",
        "pJ:IPR", "pJ:chi",
        "IPR:G", "IPR:S", "IPR:K"
    );
    for d in &diags {
        println!(
            "{:<10} {:>5.1}% {:>5.1}% {:>5.1}% {:>5.1}% {:>5} | {:>6.3} {:>5.1}% {:>6} | {:>6.3} {:>6} | {:>6.3} {:>6.3} {:>6.3}",
            d.name,
            d.cov_sig, d.cov_256, d.cov_2048, d.cov_thick, d.h_thick,
            d.lf_ipr16, 100.0 * d.lf_clean16, d.lf_chi16,
            d.pij_ipr16, d.pij_chi16,
            d.ipr16_guid, d.ipr16_sha1, d.ipr16_key
        );
    }

    println!("\n--- PART 2E: STEP A4 FAILURE (MSC 11M41 DIRICHLET, MSC 81U20/Q10 JORDAN, MSC 37D25 GREEN-KUBO) ---");
    println!(
        "{:<10} {:>8} {:>6} {:>6} {:>6} {:>6} {:>4} | {:>6} {:>5} {:>6} | {:>6} {:>6} {:>6} {:>6}",
        "block", "Z_J(1)", "kap_J", "t1%", "t1:G%", "t1:S%", "WS",
        "J_blk", "J_max", "J>=16%",
        "g(1)", "g(5)", "g(15)", "GK/V32"
    );
    for d in &diags {
        println!(
            "{:<10} {:>8.1} {:>6.2} {:>5.1}% {:>5.1}% {:>5.1}% {:>4} | {:>6} {:>5} {:>5.1}% | {:>6.3} {:>6.3} {:>6.3} {:>6.2}",
            d.name,
            d.z1, d.kappa_j, d.tau1_frac, d.tau1_guid_frac, d.tau1_sha1_frac,
            if d.ws_trace_ok { "OK" } else { "FAIL" },
            d.jordan_blocks, d.jordan_max, d.jordan_ge16_rows_pct,
            d.gk_gamma1, d.gk_gamma5, d.gk_gamma15, d.gk_var_ratio32
        );
    }

    println!("\n--- PART 2F: MSC 37A05 EXACT UNBRANCHED KAKUTANI THIN-SKYSCRAPER CENSUS (2 <= w <= 16) + STEPS A5/A6 ---");
    println!(
        "{:<10} {:>6} {:>7} {:>6} {:>6} | {:>6} {:>7} | {:>5} {:>6} {:>5} {:>5} | {:>6} {:>6} {:>4} {:>4}",
        "block", "uRun2", "uRow2", "uR2/m%", "S_cov%",
        "uRun4", "uRow4",
        "sky4#", "sky4R", "steps", "levX",
        "CL64%", "PG32%", "cLF", "cpJ"
    );
    for d in &diags {
        let lev = if d.sky_h4_steps > 0 {
            d.sky_h4_rows as f64 / d.sky_h4_steps as f64
        } else {
            0.0
        };
        println!(
            "{:<10} {:>6} {:>7} {:>5.1}% {:>5.1}% | {:>6} {:>7} | {:>5} {:>6} {:>5} {:>4.2}x | {:>5.1}% {:>5.1}% {:>4} {:>4}",
            d.name,
            d.unbr_runs_w2, d.unbr_rows_w2, 100.0 * d.unbr_rows_w2 as f64 / d.m_open as f64, d.unbr_sha1_cov_pct,
            d.unbr_runs_w4, d.unbr_rows_w4,
            d.sky_h4_chains, d.sky_h4_rows, d.sky_h4_steps, lev,
            d.cl_jump_pct, d.page_jump_pct, d.lf_cycles, d.pij_cycles
        );
    }

    // ------------------------------------------------------------------------
    // PART 3: Assembled Exact Chain Verification & Native Timing (1T / 2T / 4W)
    // ------------------------------------------------------------------------
    println!("\n--- PART 3A: ASSEMBLED EXACT CHAINS ON ALL 6 WIKIDATA BLOCKS (1T MEDIAN OF 7 RUNS) ---");
    println!(
        "{:<10} {:<10} {:>7} {:>6} {:>6} {:>7} | {:>7} {:>6} {:>7} {:>7} {:>7} {:>8} {:>7}",
        "block", "chain", "m", "H_thk", "H_tub", "bands",
        "pass1", "tchk", "elim", "wire", "emit", "total", "vsBase"
    );

    let mut base_sum_1t = 0.0f64;
    let mut tube4_sum_1t = 0.0f64;
    let mut tube8_sum_1t = 0.0f64;
    let mut tube2_sum_1t = 0.0f64;

    for (name, l, p, t_ref) in &loaded_blocks {
        let mut base_med = 0.0f64;
        for &(cname, mode) in &[
            ("A-Base", TubeMode::ThickOnly),
            ("A-Tube8", TubeMode::UnbranchedThin { w_min: 8 }),
            ("A-Tube4", TubeMode::UnbranchedThin { w_min: 4 }),
            ("A-Tube2", TubeMode::UnbranchedThin { w_min: 2 }),
        ] {
            // Exactness check first
            let (out0, _) = invert_assembled_chain(l, *p, mode, true);
            assert_eq!(&out0, t_ref, "EXACTNESS FAILURE on {name} with {cname}");

            let mut runs_st: Vec<AssembledStats> = Vec::new();
            for _ in 0..7 {
                let (out_r, st) = invert_assembled_chain(l, *p, mode, true);
                assert_eq!(out_r[0], t_ref[0]);
                runs_st.push(st);
            }
            runs_st.sort_by(|a, b| a.total_us.total_cmp(&b.total_us));
            let s = &runs_st[3];
            if cname == "A-Base" {
                base_med = s.total_us;
                base_sum_1t += s.total_us;
            } else if cname == "A-Tube8" {
                tube8_sum_1t += s.total_us;
            } else if cname == "A-Tube4" {
                tube4_sum_1t += s.total_us;
            } else {
                tube2_sum_1t += s.total_us;
            }
            println!(
                "{:<10} {:<10} {:>7} {:>6} {:>6} {:>7} | {:>7.0} {:>6.0} {:>7.0} {:>7.0} {:>7.0} {:>8.0} {:>6.3}x",
                name, cname, s.m, s.strips_thick, s.strips_tube, s.bands,
                s.pass1_us, s.tube_check_us, s.elim_us, s.wire_us, s.emit_us, s.total_us,
                s.total_us / base_med
            );
        }
    }
    println!(
        "\n[1T SUM 6 BLOCKS] A-Base = {:.0} us | A-Tube8 = {:.0} us ({:.3}x) | A-Tube4 = {:.0} us ({:.3}x) | A-Tube2 = {:.0} us ({:.3}x)",
        base_sum_1t,
        tube8_sum_1t, tube8_sum_1t / base_sum_1t,
        tube4_sum_1t, tube4_sum_1t / base_sum_1t,
        tube2_sum_1t, tube2_sum_1t / base_sum_1t,
    );

    // Non-Wikidata JSON logs benchmark (n ~ 100 KB, fast forward BWT via doubling suffix array)
    println!("\n--- PART 3B: NON-WIKIDATA GENERALIZATION TEST (SYNTHETIC JSON LOGS, UUID+SHA1) ---");
    {
        let logs = make_synthetic_json_logs(180); // ~35 KB
        let (l_log, p_log) = forward_bwt_small(&logs);
        let mut base_u = 0.0f64;
        for &(cname, mode) in &[
            ("A-Base", TubeMode::ThickOnly),
            ("A-Tube4", TubeMode::UnbranchedThin { w_min: 4 }),
            ("A-Tube2", TubeMode::UnbranchedThin { w_min: 2 }),
        ] {
            let (got, st) = invert_assembled_chain(&l_log, p_log, mode, true);
            assert_eq!(got, logs, "Non-Wikidata JSON log mismatch for {cname}");
            let mut times = Vec::new();
            for _ in 0..7 {
                let (_, s) = invert_assembled_chain(&l_log, p_log, mode, true);
                times.push(s.total_us);
            }
            times.sort_by(|a, b| a.total_cmp(b));
            let med = times[3];
            if cname == "A-Base" {
                base_u = med;
            }
            println!(
                "non-wikidata-jsonl (n={}): {:<8} m={:>5} H_thk={:>4} H_tub={:>4} bands={:>5} total={:>6.0} us ({:.3}x) [BYTE-EXACT]",
                logs.len(), cname, st.m, st.strips_thick, st.strips_tube, st.bands, med, med / base_u
            );
        }
    }

    // Multi-worker scaling (1T / 2T / 4W wall time over 6 Wikidata blocks)
    println!("\n--- PART 3C: MULTI-WORKER WALL TIMING (1T / 2T / 4W OVER 6 WIKIDATA BLOCKS, MEDIAN OF 5 PASSES) ---");
    let shared_blocks: Arc<Vec<(Vec<u8>, usize)>> = Arc::new(
        loaded_blocks
            .iter()
            .map(|(_, l, p, _)| (l.clone(), *p))
            .collect(),
    );
    for &workers in &[1usize, 2, 4] {
        let mut base_w = 0.0f64;
        for &(cname, mode) in &[
            ("A-Base", TubeMode::ThickOnly),
            ("A-Tube8", TubeMode::UnbranchedThin { w_min: 8 }),
            ("A-Tube4", TubeMode::UnbranchedThin { w_min: 4 }),
        ] {
            let mut pass_ms = Vec::new();
            for _ in 0..5 {
                let t0 = Instant::now();
                let mut handles = Vec::with_capacity(workers);
                for w in 0..workers {
                    let sb = Arc::clone(&shared_blocks);
                    handles.push(thread::spawn(move || {
                        let mut acc = 0usize;
                        for idx in (w..sb.len()).step_by(workers) {
                            let (ref l, p) = sb[idx];
                            let (out, _) = invert_assembled_chain(l, p, mode, true);
                            acc ^= out[0] as usize;
                        }
                        acc
                    }));
                }
                let mut check_xor = 0usize;
                for h in handles {
                    check_xor ^= h.join().unwrap();
                }
                let _ = check_xor;
                pass_ms.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            pass_ms.sort_by(|a, b| a.total_cmp(b));
            let med = pass_ms[2];
            if cname == "A-Base" {
                base_w = med;
            }
            println!(
                "{}W {:<8}: median wall = {:>6.2} ms ({:.3}x vs A-Base)",
                workers, cname, med, med / base_w
            );
        }
    }
}
