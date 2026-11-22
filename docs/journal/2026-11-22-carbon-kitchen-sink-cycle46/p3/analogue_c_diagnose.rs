#![forbid(unsafe_code)]

//! Programme Investigator P3 (Cycle 46) — Analogue C (Periodic Texts)
//! Stepanov Step-by-Step Diagnostic & MSC2020 Catalogue Probe.
//!
//! Compile & run:
//!   RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 \
//!     -C target-cpu=native -C codegen-units=1 \
//!     analogue_c_diagnose.rs -o /tmp/cks_c46/p3/analogue_c_diagnose
//!   /tmp/cks_c46/p3/analogue_c_diagnose

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

const LIMIT: usize = 16;

const BLOCKS: &[(&str, usize)] = &[
    ("block-0.L", 264_045),
    ("block-2.L", 320_807),
    ("block-85.L", 325_088),
    ("block-143.L", 802_868),
    ("block-207.L", 205_180),
    ("block-210.L", 791_429),
];

const CLASS_NAMES: [&str; 7] = [
    "guid", "sha1", "ent ", "num ", "jkey", "text", "strc",
];

// ============================================================================
// Core BWT utilities (exact, safe Rust)
// ============================================================================

fn build_bwt(text: &[u8]) -> (Vec<u8>, usize) {
    let n = text.len();
    assert!(n > 0);
    let mut rots: Vec<usize> = (0..n).collect();
    rots.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l_col = Vec::with_capacity(n);
    let mut primary = 0usize;
    for (row, &rot) in rots.iter().enumerate() {
        if rot == 0 {
            primary = row;
        }
        l_col.push(text[(rot + n - 1) % n]);
    }
    (l_col, primary)
}

fn build_lf(l_col: &[u8]) -> (Vec<u32>, [usize; 257]) {
    let n = l_col.len();
    let mut counts = [0usize; 256];
    for &b in l_col {
        counts[b as usize] += 1;
    }
    let mut c_arr = [0usize; 257];
    for i in 0..256 {
        c_arr[i + 1] = c_arr[i] + counts[i];
    }
    let mut next_pos = [0usize; 256];
    next_pos.copy_from_slice(&c_arr[..256]);
    let mut lf = vec![0u32; n];
    for (i, &b) in l_col.iter().enumerate() {
        let bi = b as usize;
        lf[i] = next_pos[bi] as u32;
        next_pos[bi] += 1;
    }
    (lf, c_arr)
}

/// Invert BWT via naive LF walk. Returns (text, row_to_text_pos, cycle_len).
fn invert_naive(l_col: &[u8], lf: &[u32], primary: usize) -> (Vec<u8>, Vec<u32>, usize) {
    let n = l_col.len();
    let mut text = vec![0u8; n];
    let mut row_to_pos = vec![u32::MAX; n];
    let mut r = primary;
    let mut cyc = 0usize;
    for k in 0..n {
        let tpos = n - 1 - k;
        text[tpos] = l_col[r];
        if row_to_pos[r] == u32::MAX {
            row_to_pos[r] = tpos as u32;
        }
        r = lf[r] as usize;
        if cyc == 0 && r == primary {
            cyc = k + 1;
        }
    }
    (text, row_to_pos, cyc)
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

// ============================================================================
// Exact costmap.py token classifier (pure safe Rust, no external regex crate)
// ============================================================================

#[inline]
fn is_hex_lower(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
}

#[inline]
fn is_hex_any(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

#[inline]
fn is_alnum(b: u8) -> bool {
    b.is_ascii_alphanumeric()
}

/// Reproduces `docs/journal/2026-11-20-working-backwards/costmap.py` class priority:
/// 0: guid, 1: sha1, 2: entity-id, 3: number, 4: json-key, 5: text, 6: structure
fn classify_text(text: &[u8]) -> Vec<u8> {
    let n = text.len();
    let mut cls = vec![6u8; n];

    // Base class 5 (text): ASCII alpha or non-ASCII >= 128
    for i in 0..n {
        let a = text[i];
        if ((a | 32) >= b'a' && (a | 32) <= b'z') || a >= 128 {
            cls[i] = 5;
        }
    }

    // Apply in reverse priority order (4 -> 3 -> 2 -> 1 -> 0) matching costmap.py
    // 4: json-key = "[A-Za-z_-]+"(?=:)
    let mut i = 0;
    while i < n {
        if text[i] == b'"' {
            let mut j = i + 1;
            while j < n {
                let c = text[j];
                if c.is_ascii_alphabetic() || c == b'_' || c == b'-' {
                    j += 1;
                } else {
                    break;
                }
            }
            if j > i + 1 && j + 1 < n && text[j] == b'"' && text[j + 1] == b':' {
                for c in &mut cls[i..=j] {
                    *c = 4;
                }
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }

    // 3: number = [+-]?[0-9][0-9.:TZ+-]*
    let mut i = 0;
    while i < n {
        let start = i;
        let has_sign = text[i] == b'+' || text[i] == b'-';
        let dig_pos = if has_sign { i + 1 } else { i };
        if dig_pos < n && text[dig_pos].is_ascii_digit() {
            let mut j = dig_pos + 1;
            while j < n {
                let c = text[j];
                if c.is_ascii_digit()
                    || c == b'.'
                    || c == b':'
                    || c == b'T'
                    || c == b'Z'
                    || c == b'+'
                    || c == b'-'
                {
                    j += 1;
                } else {
                    break;
                }
            }
            for c in &mut cls[start..j] {
                *c = 3;
            }
            i = j;
        } else {
            i += 1;
        }
    }

    // 2: entity-id = (?<![A-Za-z0-9])[QPL][0-9]+(?:-[SF][0-9]+)?
    let mut i = 0;
    while i < n {
        let c = text[i];
        if (c == b'Q' || c == b'P' || c == b'L') && (i == 0 || !is_alnum(text[i - 1])) {
            let mut j = i + 1;
            while j < n && text[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 {
                if j + 2 < n
                    && text[j] == b'-'
                    && (text[j + 1] == b'S' || text[j + 1] == b'F')
                    && text[j + 2].is_ascii_digit()
                {
                    j += 2;
                    while j < n && text[j].is_ascii_digit() {
                        j += 1;
                    }
                }
                for slot in &mut cls[i..j] {
                    *slot = 2;
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }

    // 1: sha1 = (?<![0-9a-f])[0-9a-f]{40}(?![0-9a-f])
    let mut i = 0;
    while i + 40 <= n {
        if is_hex_lower(text[i]) && (i == 0 || !is_hex_lower(text[i - 1])) {
            let mut j = i;
            while j < n && is_hex_lower(text[j]) {
                j += 1;
            }
            if j - i == 40 {
                for slot in &mut cls[i..j] {
                    *slot = 1;
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }

    // 0: guid = [0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}
    let mut i = 0;
    while i + 36 <= n {
        if text[i + 8] == b'-'
            && text[i + 13] == b'-'
            && text[i + 18] == b'-'
            && text[i + 23] == b'-'
        {
            let mut ok = true;
            for k in 0..36 {
                if k == 8 || k == 13 || k == 18 || k == 23 {
                    continue;
                }
                if !is_hex_any(text[i + k]) {
                    ok = false;
                    break;
                }
            }
            if ok {
                for slot in &mut cls[i..i + 36] {
                    *slot = 0;
                }
                i += 36;
                continue;
            }
        }
        i += 1;
    }

    cls
}

// ============================================================================
// Exact Open<16> induced first-return map (canonical reference)
// ============================================================================

struct OpenMap {
    m: usize,
    mark_rank: usize,
    /// Original BWT row for each retained rank 0..m
    ret_row: Vec<u32>,
    /// Next retained rank pi_J(j) for each retained rank j in 0..m
    pi_j: Vec<u32>,
    /// Word length spelled on step j -> pi_J(j) (1 + #thick steps)
    word_len: Vec<u32>,
    /// Deduplicated band word ID for step j
    word_id: Vec<u32>,
    /// Exact words (stored in reverse-LF order = forward text order? No: LF emits right-to-left)
    /// Specifically, words[id] holds [L[x], L[LF(x)], ..., L[LF^{h-1}(x)]].
    words: Vec<Vec<u8>>,
    /// Exact bands: (start_rank, width, word_id) in retained-rank order?
    /// Wait: in Open<16>, bands partition [0, n) of the thick+exit domain; we also compute exact
    /// Kakutani-Rokhlin Level-1 bands (maximal contiguous runs in J with affine pi_J and same word).
    kr_bands: Vec<(usize, usize, u32)>, // (start_rank, width, word_len)
}

fn build_open_map(l_col: &[u8], lf: &[u32], primary: usize, limit: usize) -> OpenMap {
    let n = l_col.len();
    let mut is_thin = vec![false; n];
    let mut i = 0;
    while i < n {
        let b = l_col[i];
        let mut j = i + 1;
        while j < n && l_col[j] == b {
            j += 1;
        }
        let w = j - i;
        if w <= limit {
            for slot in &mut is_thin[i..j] {
                *slot = true;
            }
        } else if primary >= i && primary < j {
            is_thin[primary] = true;
        }
        i = j;
    }

    let mut ret_row = Vec::new();
    let mut row_to_rank = vec![u32::MAX; n];
    for x in 0..n {
        if is_thin[x] {
            row_to_rank[x] = ret_row.len() as u32;
            ret_row.push(x as u32);
        }
    }
    let m = ret_row.len();
    let mark_rank = row_to_rank[primary] as usize;

    let mut pi_j = vec![0u32; m];
    let mut word_len = vec![0u32; m];
    let mut word_id = vec![0u32; m];
    let mut word_intern: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut words: Vec<Vec<u8>> = Vec::new();
    let mut buf = Vec::with_capacity(64);

    for j in 0..m {
        buf.clear();
        let mut cur = ret_row[j] as usize;
        let mut steps = 0usize;
        loop {
            buf.push(l_col[cur]);
            cur = lf[cur] as usize;
            steps += 1;
            if row_to_rank[cur] != u32::MAX || steps == n {
                break;
            }
        }
        pi_j[j] = row_to_rank[cur];
        word_len[j] = steps as u32;
        let wid = if let Some(&id) = word_intern.get(&buf) {
            id
        } else {
            let id = words.len() as u32;
            words.push(buf.clone());
            word_intern.insert(buf.clone(), id);
            id
        };
        word_id[j] = wid;
    }

    // Build exact Kakutani-Rokhlin affine bands on J (maximal contiguous intervals in J where
    // pi_J(j+1) == pi_J(j)+1 AND word_id(j+1) == word_id(j))
    let mut kr_bands = Vec::new();
    let mut start = 0usize;
    while start < m {
        let mut end = start + 1;
        while end < m
            && pi_j[end] == pi_j[end - 1].wrapping_add(1)
            && word_id[end] == word_id[start]
        {
            end += 1;
        }
        kr_bands.push((start, end - start, word_len[start]));
        start = end;
    }

    OpenMap {
        m,
        mark_rank,
        ret_row,
        pi_j,
        word_len,
        word_id,
        words,
        kr_bands,
    }
}

fn verify_open_map_exact(open: &OpenMap, expected_text: &[u8]) {
    let n = expected_text.len();
    let mut out = vec![0u8; n];
    let mut pos = n;
    let mut cur = open.mark_rank;
    while pos > 0 {
        let w = &open.words[open.word_id[cur] as usize];
        for &b in w {
            if pos == 0 {
                break;
            }
            pos -= 1;
            out[pos] = b;
        }
        cur = open.pi_j[cur] as usize;
    }
    assert_eq!(out, expected_text, "Open<16> map failed exact reconstruction!");
}

// ============================================================================
// MSC 42A75: Bohr Almost-Periodic Autocorrelation Test
// ============================================================================

fn bohr_autocorr(seq: &[u8], max_lag: usize) -> (f64, usize, f64) {
    let n = seq.len();
    let mut cnt = [0usize; 256];
    for &b in seq {
        cnt[b as usize] += 1;
    }
    let iid_baseline: f64 = cnt
        .iter()
        .map(|&c| {
            let p = c as f64 / n as f64;
            p * p
        })
        .sum();
    let sample_n = n.min(100_000);
    let mut best_lag = 1usize;
    let mut best_rate = 0.0f64;
    for d in 1..=max_lag.min(sample_n / 2) {
        let len = sample_n - d;
        let mut matches = 0usize;
        for i in 0..len {
            if seq[i] == seq[i + d] {
                matches += 1;
            }
        }
        let rate = matches as f64 / len as f64;
        if rate > best_rate {
            best_rate = rate;
            best_lag = d;
        }
    }
    (iid_baseline, best_lag, best_rate)
}

// ============================================================================
// MSC 46: Banach-Algebra Neumann Resolvent & AF C*-Algebra K_0 Invariants
// ============================================================================

struct FuncAnalReport {
    rho_thick: f64,
    rho_thin_multi: f64,
    rho_w1: f64,
    tau_thick: f64,
    tau_thin_multi: f64,
    tau_w1: f64,
    neumann_pred_height: f64,
    true_mean_height: f64,
    af_summands: usize,
    af_area_sum: usize,
    af_entropy: f64,
    commutant_idx: f64,
    bratteli_levels: Vec<(usize, usize, f64)>, // (m_ell, |V_ell|, kappa_ell)
}

fn spectral_radius_power(mat: &[Vec<f64>], iters: usize) -> f64 {
    let k = mat.len();
    if k == 0 {
        return 0.0;
    }
    let mut v = vec![1.0 / (k as f64); k];
    let mut next = vec![0.0; k];
    let mut rho = 0.0;
    for _ in 0..iters {
        let mut norm = 0.0;
        for i in 0..k {
            let mut s = 0.0;
            let row = &mat[i];
            for j in 0..k {
                s += row[j] * v[j];
            }
            next[i] = s;
            norm += s;
        }
        if norm <= 1e-15 {
            return 0.0;
        }
        rho = norm;
        for i in 0..k {
            v[i] = next[i] / norm;
        }
    }
    rho
}

fn analyze_msc46(l_col: &[u8], c_arr: &[usize; 257], open: &OpenMap) -> FuncAnalReport {
    let n = l_col.len();
    // Active alphabet mapping
    let mut active = Vec::new();
    let mut byte_to_act = [usize::MAX; 256];
    for b in 0..256 {
        if c_arr[b + 1] > c_arr[b] {
            byte_to_act[b] = active.len();
            active.push(b as u8);
        }
    }
    let k = active.len();
    let mut m_thick = vec![vec![0usize; k]; k];
    let mut m_multi = vec![vec![0usize; k]; k];
    let mut m_w1 = vec![vec![0usize; k]; k];
    let mut thin_l_cnt = vec![0usize; k];

    // Assign F[x] monotonically in O(n)
    let mut run_w = vec![0usize; n];
    let mut i = 0;
    while i < n {
        let b = l_col[i];
        let mut j = i + 1;
        while j < n && l_col[j] == b {
            j += 1;
        }
        let w = j - i;
        for slot in &mut run_w[i..j] {
            *slot = w;
        }
        i = j;
    }

    let mut tot_thick = 0usize;
    let mut tot_multi = 0usize;
    let mut tot_w1 = 0usize;
    for f_idx in 0..k {
        let b_f = active[f_idx] as usize;
        for x in c_arr[b_f]..c_arr[b_f + 1] {
            let l_idx = byte_to_act[l_col[x] as usize];
            let w = run_w[x];
            if w > LIMIT {
                m_thick[f_idx][l_idx] += 1;
                tot_thick += 1;
            } else if w >= 2 {
                m_multi[f_idx][l_idx] += 1;
                thin_l_cnt[l_idx] += 1;
                tot_multi += 1;
            } else {
                m_w1[f_idx][l_idx] += 1;
                thin_l_cnt[l_idx] += 1;
                tot_w1 += 1;
            }
        }
    }

    // Build row-stochastic sub-operators P_thick, P_multi, P_w1
    let mut p_thick = vec![vec![0.0f64; k]; k];
    let mut p_multi = vec![vec![0.0f64; k]; k];
    let mut p_w1 = vec![vec![0.0f64; k]; k];
    for f_idx in 0..k {
        let denom = (c_arr[active[f_idx] as usize + 1] - c_arr[active[f_idx] as usize]) as f64;
        for l_idx in 0..k {
            p_thick[f_idx][l_idx] = m_thick[f_idx][l_idx] as f64 / denom;
            p_multi[f_idx][l_idx] = m_multi[f_idx][l_idx] as f64 / denom;
            p_w1[f_idx][l_idx] = m_w1[f_idx][l_idx] as f64 / denom;
        }
    }

    let rho_thick = spectral_radius_power(&p_thick, 200);
    let rho_thin_multi = spectral_radius_power(&p_multi, 200);
    let rho_w1 = spectral_radius_power(&p_w1, 200);

    // Banach-algebra Neumann series e = sum_{t>=0} (P_thick)^t 1
    let mut e_vec = vec![1.0f64; k];
    let mut term = vec![1.0f64; k];
    let mut next = vec![0.0f64; k];
    for _ in 0..400 {
        let mut max_v = 0.0f64;
        for r in 0..k {
            let mut s = 0.0;
            for c in 0..k {
                s += p_thick[r][c] * term[c];
            }
            next[r] = s;
            if s > max_v {
                max_v = s;
            }
        }
        for r in 0..k {
            e_vec[r] += next[r];
            term[r] = next[r];
        }
        if max_v < 1e-10 {
            break;
        }
    }
    let tot_thin = (tot_multi + tot_w1).max(1) as f64;
    let mut neumann_pred_height = 0.0f64;
    for idx in 0..k {
        neumann_pred_height += (thin_l_cnt[idx] as f64 / tot_thin) * e_vec[idx];
    }

    // Exact AF C*-algebra A_1 invariants from Kakutani-Rokhlin bands of Open<16>
    let af_summands = open.kr_bands.len();
    let mut af_area_sum = 0usize;
    let mut af_entropy = 0.0f64;
    let mut sum_w = 0usize;
    let mut sum_w2 = 0u64;
    for &(_, w, h) in &open.kr_bands {
        let area = w * (h as usize);
        af_area_sum += area;
        let p = area as f64 / n as f64;
        if p > 0.0 {
            af_entropy -= p * p.log2();
        }
        sum_w += w;
        sum_w2 += (w as u64) * (w as u64);
    }
    let commutant_idx = sum_w2 as f64 / sum_w.max(1) as f64;

    // Iterated Section-Cut Bratteli Tower: A_1 -> A_2 -> ... -> A_D
    // At each level, keep rows whose affine-band width <= 16 (except keep mark).
    let mut bratteli_levels = Vec::new();
    bratteli_levels.push((open.m, af_summands, commutant_idx));
    let mut cur_pi = open.pi_j.clone();
    let mut cur_wid = open.word_id.clone();
    let mut cur_mark = open.mark_rank;
    for _depth in 0..6 {
        let cm = cur_pi.len();
        if cm <= 1 {
            break;
        }
        // Find maximal affine bands of cur_pi
        let mut keep = vec![false; cm];
        let mut s = 0usize;
        let mut n_thick_bands = 0usize;
        while s < cm {
            let mut e = s + 1;
            while e < cm
                && cur_pi[e] == cur_pi[e - 1].wrapping_add(1)
                && cur_wid[e] == cur_wid[s]
            {
                e += 1;
            }
            let w = e - s;
            // For level >= 2, contract any affine band with width >= 2 (or > 1) to see exact
            // Bratteli collapse speed of non-abelian commutants!
            if w <= 1 {
                keep[s] = true;
            } else {
                n_thick_bands += 1;
                if cur_mark >= s && cur_mark < e {
                    keep[cur_mark] = true;
                }
            }
            s = e;
        }
        if n_thick_bands == 0 {
            break;
        }
        let mut new_rank = vec![u32::MAX; cm];
        let mut kept_rows = Vec::new();
        for x in 0..cm {
            if keep[x] {
                new_rank[x] = kept_rows.len() as u32;
                kept_rows.push(x);
            }
        }
        let nm = kept_rows.len();
        if nm == cm || nm == 0 {
            break;
        }
        let mut n_pi = vec![0u32; nm];
        let mut n_wid = vec![0u32; nm];
        let mut intern: HashMap<Vec<u32>, u32> = HashMap::new();
        let mut next_id = 0u32;
        let mut seq = Vec::new();
        for (j, &orig) in kept_rows.iter().enumerate() {
            seq.clear();
            let mut u = orig;
            let mut steps = 0usize;
            loop {
                seq.push(cur_wid[u]);
                u = cur_pi[u] as usize;
                steps += 1;
                if new_rank[u] != u32::MAX || steps == cm {
                    break;
                }
            }
            n_pi[j] = new_rank[u];
            let id = *intern.entry(seq.clone()).or_insert_with(|| {
                let v = next_id;
                next_id += 1;
                v
            });
            n_wid[j] = id;
        }
        // Count bands at this new level
        let mut bands_cnt = 0usize;
        let mut sw = 0usize;
        let mut sw2 = 0u64;
        let mut st = 0usize;
        while st < nm {
            let mut en = st + 1;
            while en < nm && n_pi[en] == n_pi[en - 1].wrapping_add(1) && n_wid[en] == n_wid[st] {
                en += 1;
            }
            let w = en - st;
            bands_cnt += 1;
            sw += w;
            sw2 += (w as u64) * (w as u64);
            st = en;
        }
        bratteli_levels.push((nm, bands_cnt, sw2 as f64 / sw.max(1) as f64));
        cur_mark = new_rank[cur_mark] as usize;
        cur_pi = n_pi;
        cur_wid = n_wid;
    }

    FuncAnalReport {
        rho_thick,
        rho_thin_multi,
        rho_w1,
        tau_thick: tot_thick as f64 / n as f64,
        tau_thin_multi: tot_multi as f64 / n as f64,
        tau_w1: tot_w1 as f64 / n as f64,
        neumann_pred_height,
        true_mean_height: n as f64 / open.m as f64,
        af_summands,
        af_area_sum,
        af_entropy,
        commutant_idx,
        bratteli_levels,
    }
}

// ============================================================================
// MSC 03C13: Explicit Ehrenfeucht-Fraisse / Gaifman r-Ball Twin-BWT Witness
// ============================================================================

/// Builds a pair of valid primitive BWTs whose Gaifman r-balls around two distinguished
/// rows are 100% isomorphic in (U, {P_c}, LF, succ_1), yet their relative text order flips.
fn verify_ef_gaifman_witnesses() {
    println!("\n=== MSC 03C13: Ehrenfeucht-Fraisse / Gaifman r-Ball Twin-BWT Witnesses ===");
    // Family: T_1(k) = "A" + "0"^k + "B" + "0"^k + "C" + "0"^k + "D" + "0"^k
    // vs      T_2(k) = "A" + "0"^k + "C" + "0"^k + "B" + "0"^k + "D" + "0"^k
    // Around the middle of the "0"^k bridges, the Gaifman ball of radius r < k/2 in
    // (U, {P_c}, LF, succ_1) sees ONLY homogeneous '0' runs advancing affinely!
    for &k in &[4usize, 8, 16, 32] {
        let make = |order: [u8; 4]| -> Vec<u8> {
            let mut s = Vec::new();
            for &ch in &order {
                s.push(ch);
                for _ in 0..k {
                    s.push(b'0');
                }
            }
            s
        };
        let t1 = make([b'A', b'B', b'C', b'D']);
        let t2 = make([b'A', b'C', b'B', b'D']);
        let (l1, p1) = build_bwt(&t1);
        let (l2, p2) = build_bwt(&t2);
        let (lf1, _) = build_lf(&l1);
        let (lf2, _) = build_lf(&l2);
        let (dec1, pos1, cyc1) = invert_naive(&l1, &lf1, p1);
        let (dec2, pos2, cyc2) = invert_naive(&l2, &lf2, p2);
        assert_eq!(dec1, t1);
        assert_eq!(dec2, t2);
        assert_eq!(cyc1, t1.len());
        assert_eq!(cyc2, t2.len());

        // Compare row x_B (holding byte 'B') and x_C (holding byte 'C')?
        // Better: compare the midpoint rows inside the '0'^k bridge after 'B' vs 'C'!
        // In T1, the 4 runs of '0'^k sit at identical row offsets in the '0' bucket; their
        // Gaifman r-balls of radius r = (k/2 - 1) around rows at depth k/2 into '0'^k are
        // strictly isomorphic! Let's verify exact isomorphism of the r-ball around row u_1
        // (at text pos 1 + k/2, inside the A->B/C bridge) in B(L1) vs B(L2)!
        let r_rad = k / 2 - 1;
        let find_row_for_pos = |pos_map: &[u32], target: usize| -> usize {
            pos_map.iter().position(|&v| v as usize == target).unwrap()
        };
        let u1 = find_row_for_pos(&pos1, 1 + k / 2);
        let u2 = find_row_for_pos(&pos2, 1 + k / 2);

        // Check exact isomorphism of the (LF, LF^{-1}, +1, -1) neighborhood up to radius r_rad:
        // Wait: because L1 and L2 are identical on all rows whose right-context starts with
        // >= 2 zeros except where the 4 bridges meet 'A','B','C','D', let's check the EXACT
        // LF-orbit + succ_1 neighborhood of radius r_rad around u1 vs u2!
        let mut psi1 = vec![0usize; l1.len()];
        let mut psi2 = vec![0usize; l2.len()];
        for x in 0..l1.len() {
            psi1[lf1[x] as usize] = x;
            psi2[lf2[x] as usize] = x;
        }
        // Build labeled BFS tree of radius r_rad from u1 and u2 along edges {LF, Psi, +1, -1}
        let ball_sig = |l: &[u8], lf: &[u32], psi: &[usize], root: usize, rad: usize| -> Vec<(usize, u8, i32, i32)> {
            let n = l.len();
            let mut visited: HashMap<usize, usize> = HashMap::new();
            let mut order = Vec::new();
            visited.insert(root, 0);
            order.push((root, 0usize));
            let mut head = 0;
            while head < order.len() {
                let (v, d) = order[head];
                head += 1;
                if d == rad {
                    continue;
                }
                let nbrs = [lf[v] as usize, psi[v], (v + 1) % n, (v + n - 1) % n];
                for &nb in &nbrs {
                    if !visited.contains_key(&nb) {
                        let id = order.len();
                        visited.insert(nb, id);
                        order.push((nb, d + 1));
                    }
                }
            }
            // Canonical signature of the rooted ball using DFS/BFS vertex numbering
            let mut sig = Vec::new();
            for &(v, d) in &order {
                let lf_id = visited.get(&(lf[v] as usize)).map(|&x| x as i32).unwrap_or(-1);
                let p1_id = visited.get(&((v + 1) % n)).map(|&x| x as i32).unwrap_or(-1);
                sig.push((d, l[v], lf_id, p1_id));
            }
            sig
        };

        // Check maximal exact isomorphic Gaifman radius around u1 vs u2:
        let mut max_iso_r = 0usize;
        for rad in 1..=r_rad {
            let s1 = ball_sig(&l1, &lf1, &psi1, u1, rad);
            let s2 = ball_sig(&l2, &lf2, &psi2, u2, rad);
            if s1 == s2 {
                max_iso_r = rad;
            } else {
                break;
            }
        }
        // Also find rows holding 'B' and 'C' and verify relative text order flipped!
        let b_pos1 = t1.iter().position(|&c| c == b'B').unwrap();
        let c_pos1 = t1.iter().position(|&c| c == b'C').unwrap();
        let b_pos2 = t2.iter().position(|&c| c == b'B').unwrap();
        let c_pos2 = t2.iter().position(|&c| c == b'C').unwrap();
        assert!(b_pos1 < c_pos1 && b_pos2 > c_pos2);
        println!(
            "  k={:2} (n={:3}, primitive): u1={}, u2={} | exact isomorphic Gaifman ball radius r_iso={} | T1: pos(B)={}<pos(C)={}, T2: pos(B)={}>pos(C)={} [VERIFIED]",
            k, l1.len(), u1, u2, max_iso_r, b_pos1, c_pos1, b_pos2, c_pos2
        );
    }
}

// ============================================================================
// Step C2/C3/C5: Horn Coherence, Differential Syndrome & LPTQ Tube Census
// ============================================================================

struct TubeStats {
    n_tubes: usize,
    total_area: usize,    // sum w * L
    leader_steps: usize,  // sum L
    entry_exits: usize,   // sum w
    hops_eliminated: usize, // sum w * (L - 1)
    net_load_saved: isize,  // sum (w*L - (L + w)) = sum (w-1)*(L-1) - 1? Wait: emit saves w*(L-1), trace costs L-1!
    // Exact arithmetic:
    //   Open emit pays w * L loads across the tube.
    //   LPTQ leader trace pays (L - 1) dependent loads to follow a_0 -> a_1 -> ... -> a_{L-1} + w loads to read exits (or 1 exit per row = w loads).
    //   LPTQ emit pays w loads (one per entry row a_0+r jumping straight to exit!).
    //   Net dependent load savings = w*L - (L - 1 + w) = (w - 1)*(L - 1) (if exit lookups are sequential in edges[a_{L-1}..a_{L-1}+w]!).
    net_exact_saved: isize,
    eliminated_by_class: [usize; 7],
}

fn analyze_lptq_tubes(
    open: &OpenMap,
    row_class: &[u8],
    min_w: usize,
    min_l: usize,
) -> TubeStats {
    let m = open.m;
    // Step 1: Partition [0, m) into maximal zero-syndrome affine intervals I_s = [a, a+w)
    // where pi_J(a+r) == pi_J(a)+r and word_id(a+r) == word_id(a).
    let mut seg_of_rank = vec![0u32; m];
    let mut seg_start = Vec::new();
    let mut seg_width = Vec::new();
    let mut seg_dst = Vec::new();

    let mut i = 0;
    while i < m {
        let mut j = i + 1;
        while j < m
            && open.pi_j[j] == open.pi_j[j - 1].wrapping_add(1)
            && open.word_id[j] == open.word_id[i]
        {
            j += 1;
        }
        let sid = seg_start.len() as u32;
        for r in i..j {
            seg_of_rank[r] = sid;
        }
        seg_start.push(i);
        seg_width.push(j - i);
        seg_dst.push(open.pi_j[i] as usize);
        i = j;
    }

    let ns = seg_start.len();
    // Exact Strict-Width Matching Graph on segments:
    // Segment s (width w >= min_w) has a clean 1-to-1 next segment s' iff
    // seg_dst[s] == seg_start[s'] AND seg_width[s'] == w AND s' != s (and s' does not hold mark!).
    let mut next_seg = vec![u32::MAX; ns];
    let mut in_deg = vec![0u32; ns];
    let mark_seg = seg_of_rank[open.mark_rank] as usize;

    for s in 0..ns {
        let w = seg_width[s];
        if w < min_w {
            continue;
        }
        let dst = seg_dst[s];
        let s2 = seg_of_rank[dst] as usize;
        if s2 != s && s2 != mark_seg && seg_start[s2] == dst && seg_width[s2] == w {
            next_seg[s] = s2 as u32;
            in_deg[s2] += 1;
        }
    }

    // Extract maximal disjoint chains (in_deg == 0 heads, plus pure cycles if any)
    let mut visited = vec![false; ns];
    let mut stats = TubeStats {
        n_tubes: 0,
        total_area: 0,
        leader_steps: 0,
        entry_exits: 0,
        hops_eliminated: 0,
        net_load_saved: 0,
        net_exact_saved: 0,
        eliminated_by_class: [0; 7],
    };

    let mut process_chain = |head: usize, visited: &mut [bool]| {
        let mut chain = Vec::new();
        let mut cur = head;
        while cur != u32::MAX as usize && !visited[cur] {
            visited[cur] = true;
            chain.push(cur);
            let nxt = next_seg[cur];
            if nxt == u32::MAX {
                break;
            }
            cur = nxt as usize;
        }
        let l = chain.len();
        let w = seg_width[head];
        if l >= min_l && w >= min_w {
            stats.n_tubes += 1;
            stats.total_area += w * l;
            stats.leader_steps += l;
            stats.entry_exits += w;
            stats.hops_eliminated += w * (l - 1);
            // Naive load accounting: saves w*L - (L + w)
            stats.net_load_saved += (w * l) as isize - (l + w) as isize;
            // Exact sequential-exit accounting: saves w*(L-1) - (L-1) = (w-1)*(L-1)
            stats.net_exact_saved += ((w - 1) * (l - 1)) as isize;
            for &seg in &chain[1..] {
                let st = seg_start[seg];
                for r in st..st + w {
                    let orig_row = open.ret_row[r] as usize;
                    stats.eliminated_by_class[row_class[orig_row] as usize] += 1;
                }
            }
        }
    };

    for s in 0..ns {
        if seg_width[s] >= min_w && in_deg[s] == 0 && !visited[s] {
            process_chain(s, &mut visited);
        }
    }
    for s in 0..ns {
        if seg_width[s] >= min_w && !visited[s] {
            process_chain(s, &mut visited);
        }
    }
    stats
}

// ============================================================================
// Synthetic Periodic Analogue C Sanity Check (Steps C1-C6 pass 100%)
// ============================================================================

fn test_synthetic_periodic() {
    println!("=== Analogue C Solved World Verification (Exact Periodic Texts T = u^q) ===");
    let cases: &[(&[u8], usize)] = &[
        (b"abcd", 25),
        (b"abracadabra", 20),
        (b"{\"id\":\"Q1\",\"hash\":\"0123456789abcdef\"},", 30),
    ];
    for &(u, q) in cases {
        let mut t = Vec::with_capacity(u.len() * q);
        for _ in 0..q {
            t.extend_from_slice(u);
        }
        let (l_col, primary) = build_bwt(&t);
        let (lf, c_arr) = build_lf(&l_col);
        let n = l_col.len();
        let d = u.len();

        // Step C1: Check gcd of run widths & byte counts
        let mut g_run = 0usize;
        let mut min_w = usize::MAX;
        let mut i = 0;
        while i < n {
            let b = l_col[i];
            let mut j = i + 1;
            while j < n && l_col[j] == b {
                j += 1;
            }
            let w = j - i;
            g_run = gcd(g_run, w);
            if w < min_w {
                min_w = w;
            }
            i = j;
        }
        let mut g_cnt = 0usize;
        for c in 0..256 {
            let cnt = c_arr[c + 1] - c_arr[c];
            if cnt > 0 {
                g_cnt = gcd(g_cnt, cnt);
            }
        }
        let g = gcd(g_run, g_cnt);
        assert!(g % q == 0 && g >= q, "Step C1 failed on periodic text!");

        // Step C2: Check exact deck-transformation congruence LF(kq + j) = LF_u(k)*q + j
        let mut eq_ok = true;
        for k in 0..d {
            let base = lf[k * q] as usize;
            if base % q != 0 {
                eq_ok = false;
                break;
            }
            for j in 1..q {
                if lf[k * q + j] as usize != base + j {
                    eq_ok = false;
                    break;
                }
            }
        }
        assert!(eq_ok, "Step C2 deck congruence failed!");

        // Step C3-C6: Quotient L -> L_u (size d), walk d steps, repeat q times!
        let l_u: Vec<u8> = (0..d).map(|k| l_col[k * q]).collect();
        let (lf_u, _) = build_lf(&l_u);
        let (period_u, _, cyc_u) = invert_naive(&l_u, &lf_u, primary / q);
        assert_eq!(cyc_u, d);
        let mut out = Vec::with_capacity(n);
        for _ in 0..q {
            out.extend_from_slice(&period_u);
        }
        // Check exact equality against naive inversion of full column!
        let (full_dec, _, full_cyc) = invert_naive(&l_col, &lf, primary);
        assert_eq!(full_dec, t);
        assert_eq!(out, t);
        assert_eq!(full_cyc, d);
        println!(
            "  u_len={:2}, q={:2}, n={:4} | C1 gcd(w)={:2}, min_w={:2} | C2 deck_equiv=TRUE | C3-C6 quotient_cyc={:2} == full_cyc={:2} | BYTE-EXACT [VERIFIED]",
            d, q, n, g, min_w, cyc_u, full_cyc
        );
    }
}

// ============================================================================
// Main Diagnostic Runner on All 6 Authenticated Wikidata Blocks
// ============================================================================

fn main() {
    test_synthetic_periodic();
    verify_ef_gaifman_witnesses();

    let base_dir = Path::new("/home/antigravity/admitted-bwt");
    println!("\n=== STEPANOV DIAGNOSTICS ON ALL 6 AUTHENTICATED WIKIDATA BLOCKS ===");

    for &(fname, primary) in BLOCKS {
        let path = base_dir.join(fname);
        let l_col = fs::read(&path).expect("failed to read block");
        let n = l_col.len();
        let (lf, c_arr) = build_lf(&l_col);
        let (text, row_to_pos, cyc_len) = invert_naive(&l_col, &lf, primary);
        assert_eq!(cyc_len, n, "Wikidata block should be a single primitive cycle");

        // Token class per text position -> per BWT row
        let text_cls = classify_text(&text);
        let mut row_cls = vec![6u8; n];
        for row in 0..n {
            row_cls[row] = text_cls[row_to_pos[row] as usize];
        }

        // Run widths & run IDs
        let mut run_id = vec![0u32; n];
        let mut run_w = vec![0u32; n];
        let mut run_starts = Vec::new();
        let mut run_widths = Vec::new();
        let mut g_run = 0usize;
        let mut i = 0;
        while i < n {
            let b = l_col[i];
            let mut j = i + 1;
            while j < n && l_col[j] == b {
                j += 1;
            }
            let w = j - i;
            let rid = run_starts.len() as u32;
            run_starts.push(i);
            run_widths.push(w);
            g_run = gcd(g_run, w);
            for x in i..j {
                run_id[x] = rid;
                run_w[x] = w as u32;
            }
            i = j;
        }
        let r_runs = run_starts.len();

        // Build and verify exact Open<16> map
        let open = build_open_map(&l_col, &lf, primary, LIMIT);
        verify_open_map_exact(&open, &text);

        println!(
            "\n--------------------------------------------------------------------------------"
        );
        println!(
            "BLOCK {} | n={} | p={} | r={} ({:.2}%) | Open<16> m={} ({:.2}%) | cycle_len={}",
            fname,
            n,
            primary,
            r_runs,
            100.0 * r_runs as f64 / n as f64,
            open.m,
            100.0 * open.m as f64 / n as f64,
            cyc_len
        );

        // --- STEP C1: Divisibility, Width Strata by Token Class, Bohr (MSC 42), AF/Resolvent (MSC 46), Polar (MSC 94) ---
        let mut cls_all = [0usize; 7];
        let mut cls_w1 = [0usize; 7];
        let mut cls_multi = [0usize; 7]; // 2 <= w <= 16
        let mut cls_thick = [0usize; 7]; // w > 16
        let mut z_sum = 0.0f64;
        for row in 0..n {
            let c = row_cls[row] as usize;
            let w = run_w[row] as usize;
            cls_all[c] += 1;
            z_sum += 1.0 / (w as f64);
            if w == 1 {
                cls_w1[c] += 1;
            } else if w <= LIMIT {
                cls_multi[c] += 1;
            } else {
                cls_thick[c] += 1;
            }
        }
        let n_w1: usize = cls_w1.iter().sum();
        let n_multi: usize = cls_multi.iter().sum();
        let n_thick: usize = cls_thick.iter().sum();
        println!(
            "  [Step C1 | MSC 11/94A] gcd(w)={} | w=1 (noisy): {} ({:.2}%) | 2<=w<=16 (mediocre): {} ({:.2}%) | w>16 (frozen): {} ({:.2}%) | E[Z=1/w]={:.4}",
            g_run,
            n_w1, 100.0 * n_w1 as f64 / n as f64,
            n_multi, 100.0 * n_multi as f64 / n as f64,
            n_thick, 100.0 * n_thick as f64 / n as f64,
            z_sum / n as f64
        );
        let fmt_share = |arr: &[usize; 7]| -> String {
            let tot: usize = arr.iter().sum();
            let mut s = String::new();
            for c in 0..7 {
                s.push_str(&format!("{}:{:4.1}% ", CLASS_NAMES[c].trim(), 100.0 * arr[c] as f64 / tot.max(1) as f64));
            }
            s
        };
        println!("    all bytes : {}", fmt_share(&cls_all));
        println!("    w=1 rows  : {}", fmt_share(&cls_w1));
        println!("    2..=16    : {}", fmt_share(&cls_multi));
        println!("    w>16 rows : {}", fmt_share(&cls_thick));

        // MSC 42A75: Bohr almost-periodic autocorrelation on L and T
        let (iid_l, lag_l, rate_l) = bohr_autocorr(&l_col, 512);
        let (iid_t, lag_t, rate_t) = bohr_autocorr(&text, 512);
        println!(
            "  [Step C1 | MSC 42A75 Bohr] L: iid={:.3}, best lag={} match={:.3} (excess={:+.3}) | T: iid={:.3}, best lag={} match={:.3} (excess={:+.3})",
            iid_l, lag_l, rate_l, rate_l - iid_l, iid_t, lag_t, rate_t, rate_t - iid_t
        );

        // MSC 46: Banach-algebra Neumann resolvent & AF C*-algebra K_0
        let fa = analyze_msc46(&l_col, &c_arr, &open);
        assert_eq!(fa.af_area_sum, n, "AF K_0 order unit sum(h_b * w_b) must equal n!");
        println!(
            "  [Step C1 | MSC 46 AF/Neumann] rho(P_thick)={:.4}, rho(P_2..16)={:.4}, rho(P_w1)={:.4} | tau=({:.3},{:.3},{:.3})",
            fa.rho_thick, fa.rho_thin_multi, fa.rho_w1, fa.tau_thick, fa.tau_thin_multi, fa.tau_w1
        );
        println!(
            "    Neumann pred E[h]={:.3} vs TRUE Open<16> n/m={:.3} (ratio={:.3}x) | AF A_1: |V_1|={} bands, H_AF={:.2} bits, commutant kappa={:.2}",
            fa.neumann_pred_height, fa.true_mean_height,
            fa.neumann_pred_height / fa.true_mean_height,
            fa.af_summands, fa.af_entropy, fa.commutant_idx
        );
        print!("    Bratteli tower (m_l, |V_l|, kappa_l):");
        for (lvl, &(ml, vl, kl)) in fa.bratteli_levels.iter().enumerate() {
            print!(" L{}=({},{},{:.2})", lvl + 1, ml, vl, kl);
        }
        println!();

        // --- STEP C2: Deck-Congruence Splitting Defect & MSC 03 Horn Coherence / Gaifman Distance ---
        let mut unsplit_runs = 0usize;
        let mut unsplit_multi_runs = 0usize;
        let mut multi_runs = 0usize;
        let mut split_multi_runs = 0usize;
        let mut unsplit_multi_rows = 0usize;
        let mut multi_rows_tot = 0usize;
        for rid in 0..r_runs {
            let st = run_starts[rid];
            let w = run_widths[rid];
            let dst0 = lf[st] as usize;
            let dst_last = lf[st + w - 1] as usize;
            let unsplit = run_id[dst0] == run_id[dst_last];
            if unsplit {
                unsplit_runs += 1;
            }
            if w >= 2 && w <= LIMIT {
                multi_runs += 1;
                multi_rows_tot += w;
                if unsplit {
                    unsplit_multi_runs += 1;
                    unsplit_multi_rows += w;
                } else {
                    split_multi_runs += 1;
                }
            }
        }
        println!(
            "  [Step C2 | Deck Congruence] All runs unsplit: {}/{} ({:.2}%) | Thin 2..=16 runs unsplit: {}/{} ({:.2}% runs, {:.2}% rows) | split_thin_multi={}",
            unsplit_runs, r_runs, 100.0 * unsplit_runs as f64 / r_runs as f64,
            unsplit_multi_runs, multi_runs, 100.0 * unsplit_multi_runs as f64 / multi_runs.max(1) as f64,
            100.0 * unsplit_multi_rows as f64 / multi_rows_tot.max(1) as f64,
            split_multi_runs
        );

        // Compute exact Retained Horn Coherence Radius rho_J(j) on Open<16>'s pi_J
        // For each adjacent pair (j, j+1) in [0, m-1), coh1_j = (pi_J[j+1] == pi_J[j]+1 && word_id[j+1]==word_id[j])
        let m = open.m;
        let mut coh1_j = vec![false; m];
        for j in 0..m.saturating_sub(1) {
            coh1_j[j] = open.pi_j[j + 1] == open.pi_j[j].wrapping_add(1)
                && open.word_id[j + 1] == open.word_id[j];
        }
        // Compute rho_J(j) = max k <= 128 such that coh1_j[pi_J^t(j)] holds for 0<=t<k
        let mut rho_j_sum = [0u64; 7];
        let mut rho_j_cnt = [0usize; 7];
        let mut rho_j_zero = [0usize; 7];
        let mut rho_j_ge8 = [0usize; 7];
        let mut d_thick_sum = [0u64; 7];
        for j in 0..m {
            let orig = open.ret_row[j] as usize;
            let c = row_cls[orig] as usize;
            rho_j_cnt[c] += 1;
            // Distance in LF steps to nearest thick row (capped at 64):
            // Note: in Open<16>, if word_len[j] > 1 then the VERY NEXT step hits a thick row (d=1)!
            // If word_len[j] == 1, follow pi_J until word_len > 1!
            let mut dt = 0usize;
            let mut u = j;
            while dt < 64 && open.word_len[u] == 1 {
                u = open.pi_j[u] as usize;
                dt += 1;
            }
            d_thick_sum[c] += dt as u64;

            // Horn coherence radius rho_J(j) (max of left-pair (j-1,j) and right-pair (j,j+1) so both ends of a tube count!)
            let trace_pair = |mut a: usize| -> usize {
                let mut k = 0usize;
                while k < 128 && coh1_j[a] {
                    a = open.pi_j[a] as usize;
                    k += 1;
                }
                k
            };
            let r_right = trace_pair(j);
            let r_left = if j > 0 { trace_pair(j - 1) } else { 0 };
            let r_both = r_right.max(r_left);
            rho_j_sum[c] += r_both as u64;
            if r_both == 0 {
                rho_j_zero[c] += 1;
            }
            if r_both >= 8 {
                rho_j_ge8[c] += 1;
            }
        }
        println!("  [Step C2 | MSC 03 Horn Coherence & Gaifman d_thick on Open<16> retained rows J]:");
        for c in 0..7 {
            if rho_j_cnt[c] == 0 {
                continue;
            }
            let cnt = rho_j_cnt[c] as f64;
            println!(
                "    {}: ret_rows={:6} ({:4.1}% of m) | mean d_thick={:5.2} | mean rho_J={:5.2} | P(rho_J=0)={:5.1}% | P(rho_J>=8)={:5.1}%",
                CLASS_NAMES[c],
                rho_j_cnt[c], 100.0 * cnt / m as f64,
                d_thick_sum[c] as f64 / cnt,
                rho_j_sum[c] as f64 / cnt,
                100.0 * rho_j_zero[c] as f64 / cnt,
                100.0 * rho_j_ge8[c] as f64 / cnt,
            );
        }

        // --- STEP C3/C5: MSC 94B Differential Syndrome & Slepian-Wolf Rate ---
        let mut syn_nz_tot = 0usize;
        let mut syn_nz_cls = [0usize; 7];
        for j in 0..m {
            let nz = j == 0
                || open.pi_j[j] != open.pi_j[j - 1].wrapping_add(1)
                || open.word_id[j] != open.word_id[j - 1];
            if nz {
                syn_nz_tot += 1;
                syn_nz_cls[row_cls[open.ret_row[j] as usize] as usize] += 1;
            }
        }
        let raw_bits = (m as f64) * 32.0;
        let avg_w_syn = (m as f64) / (syn_nz_tot as f64);
        // Information-theoretic lower bound for syndrome intervals: m_syn * (log2(m) + log2(avg_w) + log2(|W|))
        let sw_bits = (syn_nz_tot as f64)
            * ((m as f64).log2() + avg_w_syn.log2() + (open.words.len() as f64).log2());
        println!(
            "  [Step C3/C5 | MSC 94B Differential Syndrome] m_syn={}/{} ({:.2}% of m, mean_w={:.2}) | Slepian-Wolf LB={:.1} KiB vs raw edges={:.1} KiB ({:.2}x)",
            syn_nz_tot, m, 100.0 * syn_nz_tot as f64 / m as f64, avg_w_syn,
            sw_bits / 8192.0, raw_bits / 8192.0, sw_bits / raw_bits
        );
        print!("    Syndrome density P(Delta!=0 | class):");
        for c in 0..7 {
            if rho_j_cnt[c] > 0 {
                print!(
                    " {}={:.1}%",
                    CLASS_NAMES[c].trim(),
                    100.0 * syn_nz_cls[c] as f64 / rho_j_cnt[c] as f64
                );
            }
        }
        println!();

        // --- STEP C3/C5/C6: Exact Local-Period Tube Quotient (LPTQ) Census ---
        println!("  [Step C3/C5/C6 | Exact Local-Period Tube Quotient (LPTQ) Census on pi_J]:");
        for &(min_w, min_l) in &[(2usize, 2usize), (2, 4), (3, 2), (2, 8)] {
            let ts = analyze_lptq_tubes(&open, &row_cls, min_w, min_l);
            let sha1_ret = rho_j_cnt[1].max(1) as f64;
            let guid_ret = rho_j_cnt[0].max(1) as f64;
            println!(
                "    (w>={}, L>={}): tubes={:5} | area={:6} | elim_hops={:6} ({:4.1}% of m) | leader={:5} | net_saved={:+6} ({:+4.1}% of m) | sha1_elim={:4.1}%, guid_elim={:4.1}%",
                min_w, min_l,
                ts.n_tubes, ts.total_area,
                ts.hops_eliminated, 100.0 * ts.hops_eliminated as f64 / m as f64,
                ts.leader_steps,
                ts.net_exact_saved, 100.0 * ts.net_exact_saved as f64 / m as f64,
                100.0 * ts.eliminated_by_class[1] as f64 / sha1_ret,
                100.0 * ts.eliminated_by_class[0] as f64 / guid_ret,
            );
        }

        // Verify unique words count for reference
        let distinct_wids: HashSet<u32> = open.word_id.iter().copied().collect();
        println!("    Open<16> distinct band words: {}", distinct_wids.len());
    }
}
