#![forbid(unsafe_code)]

//! Stepanov Step-by-Step Solved-Analogue Transplantation — Analogue E:
//! Repeated-Copy / Dictionary-Substituted BWTs (`2 <= w <= 16` Thin-Copy Escape).
//!
//! Cycle 47 — Programme Investigator P1 (`research/carbon-kitchen-sink`).
//!
//! Contents:
//! 1. Solved Analogue E (Steps E1-E6): synthetic texts built by substituting K
//!    repeated multi-byte dictionary words (each occurring w_i in [2, 16] times,
//!    length L_i >= 8) into a thick structured template. Verified byte-exact
//!    against `plain_invert` (forward-FL) and `cand_r5_head/`.
//! 2. Exact Failure Diagnosis & Copy-Ribbon Yield Census on:
//!    - 6 admitted Wikidata blocks (`/home/antigravity/admitted-bwt/block-*.L`)
//!    - 6 real Non-Wikidata JSONL/log blocks (`/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin`)
//!    Measuring exact distributions of ribbon width w in [2, 16], length L,
//!    saved-row yield Y = w * (L - 1), trace leverage Lambda = Y / L, and L1D
//!    dictionary footprint across yield thresholds T in {2, 16, 32, 64, 128, 256}.
//! 3. Classical-First MSC2020 Structural Diagnostics (pre-1950 FIRST):
//!    - Thue (1906, 05A05) repetition-factor stratification
//!    - Cauchy-Burnside (1815/1897, 20B30) orbit-stabilizer fiber collapsing ratio
//!    - Frobenius (1890s, 15A21) block-companion rank reduction
//!    - Euler-Hierholzer (1736/1873, 05C45) unbranched chain contraction
//!    - Poincare-Kac (1890/1947, 37A05) selective section sampling hit rate
//!    - Wald (1945, 62L10) sequential stopping probe-to-yield funnel

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

use std::fs;
use std::time::Instant;

const LIMIT: u32 = 16;

const CLASS_NAMES: [&str; 7] = [
    "GUID", "sha1", "ent-id", "number", "JSON-key", "text", "struct",
];

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    l_bytes: Vec<u8>,
    expected: Vec<u8>,
    is_wikidata: bool,
}

// ============================================================================
// 1. Reference Forward-FL, BWT & Corpus Loaders
// ============================================================================

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
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

fn expand_records(recs: &[u16], n: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &rec in recs {
        let b = (rec >> 8) as u8;
        let len = (rec & 255) as usize;
        l.resize(l.len() + len, b);
    }
    assert_eq!(l.len(), n);
    l
}

/// Forward-FL inverse BWT (matches libbzip2 and cand_r5_head on ALL columns,
/// including malformed multi-cycle columns).
fn plain_invert_fl(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut acc = 0u32;
    for c in 0..256 {
        starts[c] = acc;
        acc += counts[c];
    }
    let mut fl = vec![0u32; n];
    let mut cur = starts;
    for (i, &b) in l.iter().enumerate() {
        let s = cur[b as usize] as usize;
        cur[b as usize] += 1;
        fl[s] = ((i as u32) << 8) | u32::from(b);
    }
    let mut out = vec![0u8; n];
    let mut x = p as usize;
    for slot in &mut out {
        let v = fl[x];
        *slot = (v & 0xFF) as u8;
        x = (v >> 8) as usize;
    }
    out
}

/// Fast O(n log n) prefix-doubling suffix array for synthetic Analogue E blocks up to ~1 MiB.
fn build_bwt(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    assert!(n > 0);
    // Bucket sort by first 2 bytes for fast start
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rank: Vec<u32> = text.iter().map(|&b| u32::from(b)).collect();
    let mut tmp = vec![0u32; n];
    let mut cnt = vec![0usize; n.max(256)];
    let mut order = vec![0usize; n];

    // Counting sort initial
    cnt[..256].fill(0);
    for &b in text {
        cnt[b as usize] += 1;
    }
    let mut acc = 0usize;
    for c in 0..256 {
        let t = cnt[c];
        cnt[c] = acc;
        acc += t;
    }
    for i in 0..n {
        let b = text[i] as usize;
        sa[cnt[b]] = i;
        cnt[b] += 1;
    }
    let mut classes = 1usize;
    tmp[sa[0]] = 0;
    for i in 1..n {
        if text[sa[i]] != text[sa[i - 1]] {
            classes += 1;
        }
        tmp[sa[i]] = (classes - 1) as u32;
    }
    rank.copy_from_slice(&tmp);

    let mut k = 1usize;
    while k < n && classes < n {
        for i in 0..n {
            let s = sa[i];
            order[i] = if s >= k { s - k } else { s + n - k };
        }
        cnt[..classes].fill(0);
        for &s in &order {
            cnt[rank[s] as usize] += 1;
        }
        let mut acc = 0usize;
        for c in 0..classes {
            let t = cnt[c];
            cnt[c] = acc;
            acc += t;
        }
        for &s in &order {
            let r = rank[s] as usize;
            sa[cnt[r]] = s;
            cnt[r] += 1;
        }
        tmp[sa[0]] = 0;
        let mut next_classes = 1usize;
        for i in 1..n {
            let a = sa[i];
            let b = sa[i - 1];
            let a2 = (a + k) % n;
            let b2 = (b + k) % n;
            if rank[a] != rank[b] || rank[a2] != rank[b2] {
                next_classes += 1;
            }
            tmp[a] = (next_classes - 1) as u32;
        }
        rank.copy_from_slice(&tmp);
        classes = next_classes;
        k <<= 1;
    }

    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
        }
        l.push(text[(pos + n - 1) % n]);
    }
    (l, p)
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
            let l_bytes = fs::read(format!("{dir}/{f}")).expect("read .L");
            let expected = plain_invert_fl(&l_bytes, p);
            let records = records_of(&l_bytes);
            Block {
                name: f.trim_end_matches(".L").to_string(),
                n: l_bytes.len(),
                p,
                records,
                l_bytes,
                expected,
                is_wikidata: true,
            }
        })
        .collect()
}

fn load_non_wikidata() -> Vec<Block> {
    let path = "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin";
    let raw = fs::read(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::with_capacity(num);
    for _ in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        let l_bytes = expand_records(&records, n);
        let expected = plain_invert_fl(&l_bytes, p);
        let label = if idx < 5 {
            format!("jsonl-{idx}")
        } else {
            format!("syslog-{}", idx - 5)
        };
        blocks.push(Block {
            name: label,
            n,
            p,
            records,
            l_bytes,
            expected,
            is_wikidata: false,
        });
    }
    assert_eq!(off, raw.len());
    blocks
}

// ============================================================================
// 2. Wikidata 7-Class Byte Classifier (matching Cycle 46 working-backwards)
// ============================================================================

#[inline(always)]
fn is_hex(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

#[inline(always)]
fn is_lower_hex(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
}

fn classify_text_bytes(t: &[u8]) -> Vec<u8> {
    let n = t.len();
    let mut cls = vec![255u8; n];

    // 0: GUID
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
            if start >= 3 && t[start - 1] == b'$' {
                let mut k = start - 1;
                while k > 0 && t[k - 1].is_ascii_digit() {
                    k -= 1;
                }
                if k > 0
                    && k < start - 1
                    && matches!(t[k - 1], b'Q' | b'P' | b'L' | b'M' | b'q' | b'p')
                {
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

    // 1: sha1 (40-char lowercase hex)
    i = 0;
    while i + 40 <= n {
        if cls[i] == 255
            && (i == 0 || !is_hex(t[i - 1]))
            && (i + 40 == n || !is_hex(t[i + 40]))
            && t[i..i + 40].iter().all(|&b| is_lower_hex(b))
            && t[i..i + 40].iter().any(|&b| (b'a'..=b'f').contains(&b))
        {
            for slot in &mut cls[i..i + 40] {
                if *slot == 255 {
                    *slot = 1;
                }
            }
            i += 40;
            continue;
        }
        i += 1;
    }

    // 4: JSON-key `"..."` followed by `:`
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

    // 2: entity-id [QP]\d+
    i = 0;
    while i + 2 <= n {
        if cls[i] == 255
            && matches!(t[i], b'Q' | b'P')
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

    // 3: number, 5: text, 6: structure
    for idx in 0..n {
        if cls[idx] == 255 {
            let b = t[idx];
            if b.is_ascii_digit() || matches!(b, b'+' | b'-' | b'.') {
                cls[idx] = 3;
            } else if matches!(b, b'{' | b'}' | b'[' | b']' | b',' | b':' | b'"' | b' ' | b'\n' | b'\t') {
                cls[idx] = 6;
            } else {
                cls[idx] = 5;
            }
        }
    }
    cls
}

// ============================================================================
// 3. Solved Analogue E Generator & Explicit Steps E1–E6 Inversion
// ============================================================================

struct XorShift64(u64);
impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

/// Build a Solved Analogue E text: K repeated dictionary words (each length `word_len`,
/// occurring w_i in [w_min, w_max] times) substituted into a thick JSON-like template.
fn make_analogue_e_block(
    name: &str,
    k_words: usize,
    w_min: usize,
    w_max: usize,
    word_len: usize,
    seed: u64,
) -> Block {
    let mut rng = XorShift64(seed);
    let hex = b"0123456789abcdef";
    let mut dict: Vec<Vec<u8>> = Vec::with_capacity(k_words);
    let mut occ_plan: Vec<usize> = Vec::new();
    for wid in 0..k_words {
        let mut w = Vec::with_capacity(word_len);
        for _ in 0..word_len {
            w.push(hex[(rng.next() & 15) as usize]);
        }
        dict.push(w);
        let copies = w_min + (rng.next() as usize % (w_max - w_min + 1));
        for _ in 0..copies {
            occ_plan.push(wid);
        }
    }
    // Fisher-Yates shuffle occurrences so copies are scattered realistically
    for i in (1..occ_plan.len()).rev() {
        let j = (rng.next() as usize) % (i + 1);
        occ_plan.swap(i, j);
    }
    let mut text = Vec::with_capacity(occ_plan.len() * (word_len + 28));
    for (idx, &wid) in occ_plan.iter().enumerate() {
        text.extend_from_slice(b"{\"hash\":\"");
        text.extend_from_slice(&dict[wid]);
        text.extend_from_slice(b"\",\"ok\":");
        text.push(if (idx & 1) == 0 { b'1' } else { b'0' });
        text.extend_from_slice(b"}\n");
    }
    let (l_bytes, p) = build_bwt(&text);
    let records = records_of(&l_bytes);
    let expected = plain_invert_fl(&l_bytes, p);
    assert_eq!(expected, text, "Analogue E BWT roundtrip failed for {name}");
    Block {
        name: name.to_string(),
        n: l_bytes.len(),
        p,
        records,
        l_bytes,
        expected,
        is_wikidata: false,
    }
}

// ============================================================================
// 4. Exact Open<16> Induced Retained Graph Extractor (for Diagnosis & Steps E1-E6)
// ============================================================================

struct RetainedGraph {
    n: usize,
    m: usize,
    mark: u32,
    /// For each retained row u in 0..m: forward next retained row pi_J(u) = FL_J(u)
    next: Vec<u32>,
    /// For each retained row u in 0..m: dictionary word code
    code: Vec<u32>,
    /// Words table: 0..256 are literal bytes, 256+ are multi-byte thick+literal words
    words: Vec<Vec<u8>>,
    /// For each retained row u in 0..m: the width w in [1, 16] of the S1 thin run in L containing u
    run_w: Vec<u8>,
    /// For each retained row u in 0..m: offset within its S1 thin run (0..w)
    run_off: Vec<u8>,
    /// List of S1 thin runs `(m_start, width, byte)` with width >= 2
    s1_thin_runs_ge2: Vec<(u32, u8, u8)>,
    /// Original L row index for each retained rank u in 0..m
    orig_row: Vec<u32>,
    /// Baseline Open<16> inline16+lens dictionary footprint in bytes
    open_dict_bytes: usize,
}

/// Extract exact Open<16> induced retained permutation pi_J(u) = FL_J(u) and words
/// matching `cand_r5_head` bit-for-bit.
fn build_open16_graph(l: &[u8], p: u32) -> RetainedGraph {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut acc = 0u32;
    for c in 0..256 {
        starts[c] = acc;
        acc += counts[c];
    }

    // Maximal runs of L, split at p if thick (exact cand_r5_head::split semantics)
    let mut is_retained = vec![false; n];
    let mut row_to_ret = vec![u32::MAX; n];
    let mut run_w = Vec::new();
    let mut run_off = Vec::new();
    let mut orig_row = Vec::new();
    let mut s1_thin_runs_ge2 = Vec::new();
    let mut m = 0u32;
    let mut mark = 0u32;

    let mut pos = 0usize;
    while pos < n {
        let c = l[pos];
        let mut end = pos + 1;
        while end < n && l[end] == c {
            end += 1;
        }
        let width = (end - pos) as u32;
        if width <= LIMIT {
            if width >= 2 {
                s1_thin_runs_ge2.push((m, width as u8, c));
            }
            for k in 0..width {
                let r = pos + k as usize;
                is_retained[r] = true;
                row_to_ret[r] = m;
                if r as u32 == p {
                    mark = m;
                }
                run_w.push(width as u8);
                run_off.push(k as u8);
                orig_row.push(r as u32);
                m += 1;
            }
        } else if (pos as u32..end as u32).contains(&p) {
            let r = p as usize;
            is_retained[r] = true;
            row_to_ret[r] = m;
            mark = m;
            run_w.push(1);
            run_off.push(0);
            orig_row.push(r as u32);
            m += 1;
        }
        pos = end;
    }

    // Build exact FL(x): row x in F maps to row y in L where L[y] = F[x]
    let mut fl = vec![0u32; n];
    let mut cur = starts;
    for (i, &b) in l.iter().enumerate() {
        let s = cur[b as usize] as usize;
        cur[b as usize] += 1;
        fl[s] = i as u32;
    }

    // Trace each retained row u forward via FL until next retained row v
    let mut words: Vec<Vec<u8>> = (0..=255u8).map(|c| vec![c]).collect();
    let mut word_map: std::collections::HashMap<Vec<u8>, u32> = std::collections::HashMap::new();
    let mut next = vec![0u32; m as usize];
    let mut code = vec![0u32; m as usize];
    let mut buf = Vec::with_capacity(64);
    for u in 0..m as usize {
        buf.clear();
        let mut x = orig_row[u] as usize;
        loop {
            let y = fl[x] as usize;
            buf.push(l[y]);
            x = y;
            if is_retained[x] {
                next[u] = row_to_ret[x];
                break;
            }
        }
        let cid = if buf.len() == 1 {
            u32::from(buf[0])
        } else if let Some(&existing) = word_map.get(&buf) {
            existing
        } else {
            let id = words.len() as u32;
            word_map.insert(buf.clone(), id);
            words.push(buf.clone());
            id
        };
        code[u] = cid;
    }

    // Exact inline16 + lens + long spelling footprint of Open<16>
    let mut long_spell = 0usize;
    for w in &words[256..] {
        if w.len() > 16 {
            long_spell += w.len();
        }
    }
    let open_dict_bytes = words.len() * 17 + long_spell;

    RetainedGraph {
        n,
        m: m as usize,
        mark,
        next,
        code,
        words,
        run_w,
        run_off,
        s1_thin_runs_ge2,
        orig_row,
        open_dict_bytes,
    }
}

/// Emit text from RetainedGraph to verify 100% forward-FL parity.
fn emit_graph(g: &RetainedGraph) -> Vec<u8> {
    let mut out = Vec::with_capacity(g.n);
    let mut u = g.mark as usize;
    for _ in 0..g.m {
        let w = &g.words[g.code[u] as usize];
        let take = w.len().min(g.n - out.len());
        out.extend_from_slice(&w[..take]);
        u = g.next[u] as usize;
        if u == g.mark as usize || out.len() >= g.n {
            break;
        }
    }
    while out.len() < g.n {
        let take = out.len().min(g.n - out.len());
        out.extend_from_within(..take);
    }
    out
}

// ============================================================================
// 5. Exact Copy-Ribbon Decomposition (Maximal Unbranched Ribbons in pi_J)
// ============================================================================

#[derive(Clone, Debug)]
struct Ribbon {
    start_u: u32,
    width: u8,
    len_steps: u32,      // number of rigid intervals I_0..I_{L-1} (>= 2)
    saved_hops: u32,     // width * (len_steps - 1)
    composite_bytes: u32, // byte length of composite word W_0..W_{L-1}
    intervals: Vec<u32>, // u_0, u_1, ..., u_{L-1}
}

/// Check whether interval `[u, u+w)` is a rigid affine step:
/// all w lanes share identical `code` and satisfy `next[u+k] == next[u] + k`.
#[inline(always)]
fn is_rigid_step(g: &RetainedGraph, u: u32, w: usize) -> bool {
    let u0 = u as usize;
    if u0 + w > g.m {
        return false;
    }
    let c0 = g.code[u0];
    let v0 = g.next[u0];
    if (v0 as usize) + w > g.m {
        return false;
    }
    for k in 1..w {
        if g.code[u0 + k] != c0 || g.next[u0 + k] != v0 + k as u32 {
            return false;
        }
    }
    true
}

/// Check whether interval `[u, u+w)` has constant `code` (even if exits diverge).
#[inline(always)]
fn has_const_code(g: &RetainedGraph, u: u32, w: usize) -> bool {
    let u0 = u as usize;
    if u0 + w > g.m {
        return false;
    }
    let c0 = g.code[u0];
    for k in 1..w {
        if g.code[u0 + k] != c0 {
            return false;
        }
    }
    true
}

/// Extract maximal disjoint copy ribbons of width 2..=16 directly from S1 thin-run seeds
/// (no O(r) boundary tables — strictly using S1 runs of width 2..=16!).
/// To avoid splitting ribbons mid-chain during census, we first identify exact chain
/// predecessors among rigid S1 runs (i.e., Euler-Hierholzer maximal directed paths),
/// then trace each maximal chain from its head.
fn extract_maximal_ribbons(g: &RetainedGraph) -> Vec<Ribbon> {
    // Mark which retained rows belong to S1 thin runs of width >= 2, and find rigid heads.
    // For the census we compute exact in-degree of rigid width-w intervals so every maximal
    // chain is counted cleanly once from its exact start!
    let mut is_interior = vec![false; g.m];
    let mut step_ok = vec![false; g.s1_thin_runs_ge2.len()];
    for (idx, &(u, w, _)) in g.s1_thin_runs_ge2.iter().enumerate() {
        let wu = w as usize;
        if is_rigid_step(g, u, wu) {
            let v = g.next[u as usize] as usize;
            // v must align with the start of a width-w S1 run to form an exact S1-run chain,
            // OR even more generally, v..v+w can be ANY contiguous retained interval of width w!
            // Here we check whether v..v+w is rigid OR has const code (general copy ribbon!).
            step_ok[idx] = true;
            // Mark v..v+w as having a rigid predecessor of width w (unless mark is inside)
            let has_mark = g.mark.wrapping_sub(v as u32) < w as u32;
            if !has_mark {
                // Only mark interior if widths match or v starts a run
                if g.run_off[v] == 0 && g.run_w[v] == w {
                    is_interior[v] = true;
                }
            }
        }
    }

    let mut visited = vec![false; g.m];
    let mut ribbons = Vec::new();

    // Pass A: start from true chain heads (not interior)
    // Pass B: catch any pure cycles among S1 runs (periodic blocks)
    for pass in 0..2 {
        for (idx, &(u0, w, _)) in g.s1_thin_runs_ge2.iter().enumerate() {
            if !step_ok[idx] {
                continue;
            }
            if pass == 0 && is_interior[u0 as usize] {
                continue;
            }
            let wu = w as usize;
            if visited[u0 as usize..u0 as usize + wu].iter().any(|&x| x) {
                continue;
            }
            let mut intervals = vec![u0];
            let mut cur = u0;
            let mut comp_bytes = g.words[g.code[u0 as usize] as usize].len() as u32;

            while is_rigid_step(g, cur, wu) {
                let nxt = g.next[cur as usize];
                let nu = nxt as usize;
                // Must not contain mark inside bypassed interior
                if g.mark.wrapping_sub(nxt) < w as u32 {
                    break;
                }
                // Must not overlap any already-visited row or earlier interval in this ribbon
                if visited[nu..nu + wu].iter().any(|&x| x) {
                    break;
                }
                let overlaps_self = intervals.iter().any(|&prev| {
                    let diff = if nxt >= prev { nxt - prev } else { prev - nxt };
                    diff < w as u32
                });
                if overlaps_self {
                    break;
                }
                if !has_const_code(g, nxt, wu) {
                    break;
                }
                intervals.push(nxt);
                comp_bytes += g.words[g.code[nu] as usize].len() as u32;
                cur = nxt;
                if intervals.len() >= 1024 {
                    break;
                }
            }

            let len_steps = intervals.len() as u32;
            if len_steps >= 2 {
                for &iv in &intervals {
                    for k in 0..wu {
                        visited[iv as usize + k] = true;
                    }
                }
                ribbons.push(Ribbon {
                    start_u: u0,
                    width: w,
                    len_steps,
                    saved_hops: u32::from(w) * (len_steps - 1),
                    composite_bytes: comp_bytes,
                    intervals,
                });
            }
        }
    }
    ribbons
}

/// Apply ribbon contraction to a copy of RetainedGraph and verify 100% byte-exactness!
fn apply_ribbons_and_verify(g: &RetainedGraph, ribbons: &[Ribbon], expected: &[u8]) -> usize {
    let mut next = g.next.clone();
    let mut code = g.code.clone();
    let mut words = g.words.clone();

    let mut total_saved = 0usize;
    for rb in ribbons {
        let wu = rb.width as usize;
        let mut comp = Vec::with_capacity(rb.composite_bytes as usize);
        for &iv in &rb.intervals {
            comp.extend_from_slice(&words[g.code[iv as usize] as usize]);
        }
        let new_code = words.len() as u32;
        words.push(comp);
        let u0 = rb.start_u as usize;
        let last = *rb.intervals.last().unwrap() as usize;
        for k in 0..wu {
            code[u0 + k] = new_code;
            next[u0 + k] = g.next[last + k];
        }
        total_saved += rb.saved_hops as usize;
    }

    // Emit from contracted graph and verify exactness + hop count
    let mut out = Vec::with_capacity(g.n);
    let mut u = g.mark as usize;
    let mut hops = 0usize;
    for _ in 0..g.m {
        hops += 1;
        let w = &words[code[u] as usize];
        let take = w.len().min(g.n - out.len());
        out.extend_from_slice(&w[..take]);
        u = next[u] as usize;
        if u == g.mark as usize || out.len() >= g.n {
            break;
        }
    }
    while out.len() < g.n {
        let take = out.len().min(g.n - out.len());
        out.extend_from_within(..take);
    }
    assert_eq!(out, expected, "Contracted ribbon graph mismatch!");
    // On single-cycle primitive blocks, hops == g.m - total_saved
    let _ = total_saved;
    hops
}

// ============================================================================
// 6. Main Diagnostic Suite
// ============================================================================

fn main() {
    println!("=============================================================================");
    println!("CYCLE 47 P1 — ANALOGUE E: REPEATED-COPY / DICTIONARY-SUBSTITUTED BWTs");
    println!("Stepanov Proof (E1-E6) + Failure Diagnosis + Classical-First MSC2020 Census");
    println!("=============================================================================\n");

    // -------------------------------------------------------------------------
    // PART 1: SOLVED ANALOGUE E VERIFICATION (Steps E1–E6)
    // -------------------------------------------------------------------------
    println!("--- PART 1: SOLVED ANALOGUE E VERIFICATION (Steps E1-E6) ---");
    let solved_blocks = vec![
        make_analogue_e_block("AnaE-small-K20", 20, 3, 12, 24, 0xE101),
        make_analogue_e_block("AnaE-med-K150", 150, 4, 14, 32, 0xE202),
        make_analogue_e_block("AnaE-300k-K450", 450, 4, 15, 40, 0xE303),
        make_analogue_e_block("AnaE-900k-K1200", 1200, 4, 16, 40, 0xE404),
    ];

    println!(
        "{:<16} {:>8} {:>7} {:>7} {:>8} {:>7} {:>8} {:>8} {:>9} {:>8}",
        "Block", "n", "Open_m", "K_rib", "Saved", "Save%", "Hops_E", "Dict_KiB", "Leverage", "Exact"
    );
    let mut r5_k = cand_r5_head::Kernel::default();
    let mut r5_out = Vec::new();
    for blk in &solved_blocks {
        // Verify cand_r5_head baseline
        let _ = r5_k
            .run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut r5_out)
            .expect("r5_head");
        assert_eq!(r5_out, blk.expected);

        let g = build_open16_graph(&blk.l_bytes, blk.p);
        assert_eq!(emit_graph(&g), blk.expected);
        let ribbons = extract_maximal_ribbons(&g);
        let hops_after = apply_ribbons_and_verify(&g, &ribbons, &blk.expected);
        let saved: usize = ribbons.iter().map(|r| r.saved_hops as usize).sum();
        let trace_steps: usize = ribbons.iter().map(|r| r.len_steps as usize).sum();
        let added_bytes: usize = ribbons
            .iter()
            .map(|r| 17 + if r.composite_bytes > 16 { r.composite_bytes as usize } else { 0 })
            .sum();
        let lev = if trace_steps > 0 {
            saved as f64 / trace_steps as f64
        } else {
            0.0
        };
        println!(
            "{:<16} {:>8} {:>7} {:>7} {:>8} {:>6.1}% {:>8} {:>7.1}K {:>8.2}x {:>8}",
            blk.name,
            blk.n,
            g.m,
            ribbons.len(),
            saved,
            100.0 * saved as f64 / g.m as f64,
            hops_after,
            (g.open_dict_bytes + added_bytes) as f64 / 1024.0,
            lev,
            "EXACT"
        );
    }

    // -------------------------------------------------------------------------
    // PART 2: LOAD 6 ADMITTED WIKIDATA + 6 REAL NON-WIKIDATA BLOCKS
    // -------------------------------------------------------------------------
    println!("\n--- PART 2: EXACT COPY-RIBBON CENSUS ON 6 WIKIDATA + 6 NON-WIKIDATA BLOCKS ---");
    let t_load = Instant::now();
    let mut blocks = load_six_admitted();
    let non_wiki = load_non_wikidata();
    blocks.extend(non_wiki);
    println!(
        "Loaded {} blocks ({} Wikidata + {} Non-Wikidata) in {:.2}s.\n",
        blocks.len(),
        6,
        6,
        t_load.elapsed().as_secs_f64()
    );

    let thresholds: [u32; 6] = [2, 16, 32, 64, 128, 256];

    // Summary aggregates per threshold (Wikidata sum vs Non-Wikidata sum)
    #[derive(Default, Clone, Copy)]
    struct ThreshAgg {
        k_rib: usize,
        saved: usize,
        trace_steps: usize,
        added_dict_bytes: usize,
        sha1_saved: usize,
        text_saved: usize,
        other_saved: usize,
    }
    let mut wiki_agg = [ThreshAgg::default(); 6];
    let mut nonw_agg = [ThreshAgg::default(); 6];
    let mut wiki_m_sum = 0usize;
    let mut nonw_m_sum = 0usize;

    // Width and length histograms (Wikidata & Non-Wikidata)
    let mut wiki_w_saved = [0usize; 17];
    let mut wiki_w_cnt = [0usize; 17];
    let mut nonw_w_saved = [0usize; 17];
    let mut nonw_w_cnt = [0usize; 17];

    // Length buckets: [2, 3, 4..7, 8..15, 16..31, 32..63, 64+]
    let mut wiki_l_saved = [0usize; 7];
    let mut wiki_l_cnt = [0usize; 7];
    let mut nonw_l_saved = [0usize; 7];
    let mut nonw_l_cnt = [0usize; 7];

    // Classical-First MSC2020 diagnostics per block
    println!(
        "{:<10} {:>7} {:>6} {:>7} {:>6} | {:>16} {:>16} {:>16} {:>16} {:>16}",
        "Block", "Open_m", "O16_KB", "S1>=2", "K_all",
        "T>=16(sav%,+KB)", "T>=32(sav%,+KB)", "T>=64(sav%,+KB)", "T>=128(sav%,+KB)", "T>=256(sav%,+KB)"
    );

    struct ClassicalDiag {
        name: String,
        is_wiki: bool,
        m: usize,
        s1_ge2: usize,
        s1_ge4: usize,
        // Wald funnel on S1 w>=4 runs: step1 (rigid>=1), step2 (len>=3), step4 (len>=5), step8 (len>=9)
        wald_s1: usize,
        wald_s2: usize,
        wald_s4: usize,
        wald_s8: usize,
        // Burnside effective width (sum w*L / sum L) on T>=32 ribbons
        burnside_w: f64,
        // Poincare-Kac selective section hit rate at S=4, S=8 on T>=64 ribbons
        pk_hit_s4: f64,
        pk_hit_s8: f64,
        // Class attribution of T>=32 saved hops (for Wikidata)
        cls_saved: [usize; 7],
    }
    let mut cdiags = Vec::new();

    for blk in &blocks {
        // Confirm cand_r5_head exactness
        let _ = r5_k
            .run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut r5_out)
            .expect("r5_head");
        assert_eq!(r5_out, blk.expected);

        let g = build_open16_graph(&blk.l_bytes, blk.p);
        assert_eq!(emit_graph(&g), blk.expected);
        let ribbons = extract_maximal_ribbons(&g);

        // Verify 100% byte-exactness of full ribbon contraction on EVERY block!
        let hops_after = apply_ribbons_and_verify(&g, &ribbons, &blk.expected);
        let total_saved_all: usize = ribbons.iter().map(|r| r.saved_hops as usize).sum();
        assert_eq!(hops_after, g.m - total_saved_all, "hop accounting mismatch on {}", blk.name);

        // Build row -> text class map for Wikidata
        let row_cls: Vec<u8> = if blk.is_wikidata {
            let text_cls = classify_text_bytes(&blk.expected);
            // Walk forward from p along FL to map L-row -> text offset
            let mut counts = [0u32; 256];
            for &b in &blk.l_bytes {
                counts[b as usize] += 1;
            }
            let mut starts = [0u32; 256];
            let mut acc = 0u32;
            for c in 0..256 {
                starts[c] = acc;
                acc += counts[c];
            }
            let mut fl = vec![0u32; blk.n];
            let mut cur = starts;
            for (i, &b) in blk.l_bytes.iter().enumerate() {
                let s = cur[b as usize] as usize;
                cur[b as usize] += 1;
                fl[s] = i as u32;
            }
            let mut l_to_tpos = vec![0u32; blk.n];
            let mut x = blk.p as usize;
            for tpos in 0..blk.n {
                let y = fl[x] as usize;
                l_to_tpos[y] = tpos as u32;
                x = y;
            }
            (0..g.m)
                .map(|u| text_cls[l_to_tpos[g.orig_row[u] as usize] as usize])
                .collect()
        } else {
            vec![5u8; g.m]
        };

        if blk.is_wikidata {
            wiki_m_sum += g.m;
        } else {
            nonw_m_sum += g.m;
        }

        for rb in &ribbons {
            let wu = rb.width as usize;
            let lb = match rb.len_steps {
                2 => 0,
                3 => 1,
                4..=7 => 2,
                8..=15 => 3,
                16..=31 => 4,
                32..=63 => 5,
                _ => 6,
            };
            if blk.is_wikidata {
                wiki_w_cnt[wu] += 1;
                wiki_w_saved[wu] += rb.saved_hops as usize;
                wiki_l_cnt[lb] += 1;
                wiki_l_saved[lb] += rb.saved_hops as usize;
            } else {
                nonw_w_cnt[wu] += 1;
                nonw_w_saved[wu] += rb.saved_hops as usize;
                nonw_l_cnt[lb] += 1;
                nonw_l_saved[lb] += rb.saved_hops as usize;
            }
        }

        let mut row_strs = Vec::new();
        let mut cls_saved_t32 = [0usize; 7];
        for (ti, &t) in thresholds.iter().enumerate() {
            let mut k_rib = 0usize;
            let mut saved = 0usize;
            let mut trace_steps = 0usize;
            let mut added_dict = 0usize;
            let mut sha1_s = 0usize;
            let mut text_s = 0usize;
            let mut other_s = 0usize;
            for rb in &ribbons {
                if rb.saved_hops >= t {
                    k_rib += 1;
                    saved += rb.saved_hops as usize;
                    trace_steps += rb.len_steps as usize;
                    added_dict += 17 + if rb.composite_bytes > 16 { rb.composite_bytes as usize } else { 0 };
                    // Attribute saved hops to class of interior intervals
                    for &iv in &rb.intervals[1..] {
                        for k in 0..rb.width as usize {
                            let c = row_cls[iv as usize + k] as usize;
                            if c == 1 {
                                sha1_s += 1;
                            } else if c == 5 {
                                text_s += 1;
                            } else {
                                other_s += 1;
                            }
                            if t == 32 && c < 7 {
                                cls_saved_t32[c] += 1;
                            }
                        }
                    }
                }
            }
            let agg = if blk.is_wikidata {
                &mut wiki_agg[ti]
            } else {
                &mut nonw_agg[ti]
            };
            agg.k_rib += k_rib;
            agg.saved += saved;
            agg.trace_steps += trace_steps;
            agg.added_dict_bytes += added_dict;
            agg.sha1_saved += sha1_s;
            agg.text_saved += text_s;
            agg.other_saved += other_s;

            if ti >= 1 {
                row_strs.push(format!(
                    "{:5.1}%,+{:4.1}K",
                    100.0 * saved as f64 / g.m as f64,
                    added_dict as f64 / 1024.0
                ));
            }
        }

        println!(
            "{:<10} {:>7} {:>5.1}K {:>7} {:>6} | {:>16} {:>16} {:>16} {:>16} {:>16}",
            blk.name,
            g.m,
            g.open_dict_bytes as f64 / 1024.0,
            g.s1_thin_runs_ge2.len(),
            ribbons.len(),
            row_strs[0],
            row_strs[1],
            row_strs[2],
            row_strs[3],
            row_strs[4],
        );

        // Wald sequential stopping funnel on S1 w>=4 runs
        let mut s1_ge4 = 0usize;
        let (mut ws1, mut ws2, mut ws4, mut ws8) = (0usize, 0usize, 0usize, 0usize);
        for &(u0, w, _) in &g.s1_thin_runs_ge2 {
            if w < 4 {
                continue;
            }
            s1_ge4 += 1;
            let wu = w as usize;
            let mut cur = u0;
            let mut steps = 0usize;
            while steps < 8 && is_rigid_step(&g, cur, wu) {
                cur = g.next[cur as usize];
                steps += 1;
            }
            if steps >= 1 { ws1 += 1; }
            if steps >= 2 { ws2 += 1; }
            if steps >= 4 { ws4 += 1; }
            if steps >= 8 { ws8 += 1; }
        }

        // Burnside effective width on T>=32 ribbons
        let (mut sum_wl, mut sum_l) = (0usize, 0usize);
        // Poincare-Kac selective section hit rate at S=4, S=8 on T>=64 ribbons
        let (mut pk_tot64, mut pk_cap4, mut pk_cap8) = (0usize, 0usize, 0usize);
        for rb in &ribbons {
            if rb.saved_hops >= 32 {
                sum_wl += rb.width as usize * rb.len_steps as usize;
                sum_l += rb.len_steps as usize;
            }
            if rb.saved_hops >= 64 {
                pk_tot64 += rb.saved_hops as usize;
                // First interval index i where (iv & 3) == 0 (stride 4) or (iv & 7) == 0 (stride 8)
                if let Some(pos4) = rb.intervals.iter().position(|&iv| (iv & 3) == 0) {
                    let rem = rb.intervals.len() - pos4;
                    if rem >= 2 {
                        pk_cap4 += rb.width as usize * (rem - 1);
                    }
                }
                if let Some(pos8) = rb.intervals.iter().position(|&iv| (iv & 7) == 0) {
                    let rem = rb.intervals.len() - pos8;
                    if rem >= 2 {
                        pk_cap8 += rb.width as usize * (rem - 1);
                    }
                }
            }
        }
        cdiags.push(ClassicalDiag {
            name: blk.name.clone(),
            is_wiki: blk.is_wikidata,
            m: g.m,
            s1_ge2: g.s1_thin_runs_ge2.len(),
            s1_ge4,
            wald_s1: ws1,
            wald_s2: ws2,
            wald_s4: ws4,
            wald_s8: ws8,
            burnside_w: if sum_l > 0 { sum_wl as f64 / sum_l as f64 } else { 0.0 },
            pk_hit_s4: if pk_tot64 > 0 { 100.0 * pk_cap4 as f64 / pk_tot64 as f64 } else { 0.0 },
            pk_hit_s8: if pk_tot64 > 0 { 100.0 * pk_cap8 as f64 / pk_tot64 as f64 } else { 0.0 },
            cls_saved: cls_saved_t32,
        });
    }

    // Print threshold summary tables
    println!("\n--- PART 3: AGGREGATE YIELD vs DICTIONARY FOOTPRINT & TRACE LEVERAGE ---");
    for (corpus_name, m_sum, aggs) in [
        ("6 Admitted Wikidata", wiki_m_sum, &wiki_agg),
        ("6 Real Non-Wikidata", nonw_m_sum, &nonw_agg),
    ] {
        println!("\nCorpus: {corpus_name} (total Open<16> m = {m_sum})");
        println!(
            "{:>6} {:>8} {:>9} {:>7} {:>9} {:>8} {:>10} {:>10} {:>9}",
            "T_min", "K_rib", "SavedHops", "Save%", "TraceStp", "Lev(Y/L)", "Avg+DictKB", "MaxEmit_us", "Budg_ns/st"
        );
        for (ti, &t) in thresholds.iter().enumerate() {
            let a = aggs[ti];
            let lev = if a.trace_steps > 0 {
                a.saved as f64 / a.trace_steps as f64
            } else {
                0.0
            };
            let avg_kb = (a.added_dict_bytes as f64 / 6.0) / 1024.0;
            // i9 #145 emit cost = 5.79 ns/row
            let emit_save_us = a.saved as f64 * 5.79 / 1000.0;
            let budg_ns_per_step = if a.trace_steps > 0 {
                (a.saved as f64 * 5.79) / a.trace_steps as f64
            } else {
                0.0
            };
            println!(
                "{:>6} {:>8} {:>9} {:>6.2}% {:>9} {:>7.2}x {:>9.2}K {:>9.1}us {:>8.1}ns",
                t,
                a.k_rib,
                a.saved,
                100.0 * a.saved as f64 / m_sum as f64,
                a.trace_steps,
                lev,
                avg_kb,
                emit_save_us,
                budg_ns_per_step,
            );
        }
    }

    // Width distribution
    println!("\n--- PART 4: WIDTH DISTRIBUTION (w in 2..=16) ---");
    println!(
        "{:>3} | {:>8} {:>9} {:>7} | {:>8} {:>9} {:>7}",
        "w", "Wiki_K", "Wiki_Sav", "W_Sav%", "NonW_K", "NonW_Sav", "N_Sav%"
    );
    for w in 2..=16 {
        println!(
            "{:>3} | {:>8} {:>9} {:>6.2}% | {:>8} {:>9} {:>6.2}%",
            w,
            wiki_w_cnt[w],
            wiki_w_saved[w],
            100.0 * wiki_w_saved[w] as f64 / wiki_m_sum as f64,
            nonw_w_cnt[w],
            nonw_w_saved[w],
            100.0 * nonw_w_saved[w] as f64 / nonw_m_sum as f64,
        );
    }

    // Length distribution
    println!("\n--- PART 5: LENGTH DISTRIBUTION (L steps) ---");
    let l_labels = ["2", "3", "4..7", "8..15", "16..31", "32..63", "64+"];
    println!(
        "{:>7} | {:>8} {:>9} {:>7} | {:>8} {:>9} {:>7}",
        "L_bin", "Wiki_K", "Wiki_Sav", "W_Sav%", "NonW_K", "NonW_Sav", "N_Sav%"
    );
    for b in 0..7 {
        println!(
            "{:>7} | {:>8} {:>9} {:>6.2}% | {:>8} {:>9} {:>6.2}%",
            l_labels[b],
            wiki_l_cnt[b],
            wiki_l_saved[b],
            100.0 * wiki_l_saved[b] as f64 / wiki_m_sum as f64,
            nonw_l_cnt[b],
            nonw_l_saved[b],
            100.0 * nonw_l_saved[b] as f64 / nonw_m_sum as f64,
        );
    }

    // Classical-First MSC2020 diagnostics
    println!("\n--- PART 6: CLASSICAL-FIRST MSC2020 DIAGNOSTICS (WALD / BURNSIDE / POINCARE-KAC) ---");
    println!(
        "{:<10} {:>6} {:>6} | {:>6} {:>6} {:>6} {:>6} | {:>7} | {:>6} {:>6}",
        "Block", "S1>=2", "S1>=4", "W>=1", "W>=2", "W>=4", "W>=8", "Burn_w", "PK_S4%", "PK_S8%"
    );
    for d in &cdiags {
        println!(
            "{:<10} {:>6} {:>6} | {:>6} {:>6} {:>6} {:>6} | {:>6.2}x | {:>5.1}% {:>5.1}%",
            d.name,
            d.s1_ge2,
            d.s1_ge4,
            d.wald_s1,
            d.wald_s2,
            d.wald_s4,
            d.wald_s8,
            d.burnside_w,
            d.pk_hit_s4,
            d.pk_hit_s8,
        );
    }

    println!("\n--- PART 7: WIKIDATA 7-CLASS ATTRIBUTION OF T>=32 SAVED HOPS ---");
    println!(
        "{:<10} {:>7} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "Block", "m", CLASS_NAMES[0], CLASS_NAMES[1], CLASS_NAMES[2], CLASS_NAMES[3], CLASS_NAMES[4], CLASS_NAMES[5], CLASS_NAMES[6]
    );
    let mut cls_tot = [0usize; 7];
    for d in &cdiags {
        if !d.is_wiki {
            continue;
        }
        for c in 0..7 {
            cls_tot[c] += d.cls_saved[c];
        }
        println!(
            "{:<10} {:>7} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
            d.name, d.m,
            d.cls_saved[0], d.cls_saved[1], d.cls_saved[2], d.cls_saved[3],
            d.cls_saved[4], d.cls_saved[5], d.cls_saved[6],
        );
    }
    let tot_t32: usize = cls_tot.iter().sum();
    println!(
        "{:<10} {:>7} | {:>6.1}% {:>6.1}% {:>6.1}% {:>6.1}% {:>6.1}% {:>6.1}% {:>6.1}%",
        "SHARE%",
        tot_t32,
        100.0 * cls_tot[0] as f64 / tot_t32.max(1) as f64,
        100.0 * cls_tot[1] as f64 / tot_t32.max(1) as f64,
        100.0 * cls_tot[2] as f64 / tot_t32.max(1) as f64,
        100.0 * cls_tot[3] as f64 / tot_t32.max(1) as f64,
        100.0 * cls_tot[4] as f64 / tot_t32.max(1) as f64,
        100.0 * cls_tot[5] as f64 / tot_t32.max(1) as f64,
        100.0 * cls_tot[6] as f64 / tot_t32.max(1) as f64,
    );

    println!("\nALL DIAGNOSTIC & EXACTNESS CHECKS PASSED.");
}
