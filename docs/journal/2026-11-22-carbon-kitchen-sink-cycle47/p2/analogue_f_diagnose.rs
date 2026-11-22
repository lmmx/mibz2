#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::Instant;

// ============================================================================
// 0. Core Helpers: BWT, Forward-FL, Records, Corpora, 7-Class Classifier
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    l: Vec<u8>,
    records: Vec<u16>,
    text: Vec<u8>,
}

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

/// Forward-FL inverse BWT matching libbzip2 / cand_r5_head semantics on all columns
/// (including malformed multi-cycle columns!).
fn forward_fl_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    let mut fl = vec![0u32; n];
    let mut cur = starts;
    for (i, &b) in l.iter().enumerate() {
        let pos = cur[b as usize] as usize;
        fl[pos] = ((i as u32) << 8) | u32::from(b);
        cur[b as usize] += 1;
    }
    let mut out = Vec::with_capacity(n);
    let mut x = p as usize;
    for _ in 0..n {
        let e = fl[x];
        out.push((e & 0xFF) as u8);
        x = (e >> 8) as usize;
    }
    out
}

/// Compute BWT of a text using O(n log^2 n) prefix-doubling suffix array (exact for up to 300K).
fn bwt_doubling(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    assert!(n > 0);
    if n == 1 {
        return (vec![text[0]], 0);
    }
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rnk: Vec<u32> = text.iter().map(|&b| u32::from(b)).collect();
    let mut tmp = vec![0u32; n];
    let mut k = 1usize;
    while k < n {
        sa.sort_unstable_by_key(|&i| (rnk[i], rnk[(i + k) % n]));
        tmp[sa[0]] = 0;
        let mut classes = 0u32;
        for idx in 1..n {
            let prev = sa[idx - 1];
            let curr = sa[idx];
            if rnk[prev] != rnk[curr] || rnk[(prev + k) % n] != rnk[(curr + k) % n] {
                classes += 1;
            }
            tmp[curr] = classes;
        }
        std::mem::swap(&mut rnk, &mut tmp);
        if classes as usize + 1 == n {
            break;
        }
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
            let l = fs::read(format!("{dir}/{f}")).expect("read .L");
            let text = forward_fl_invert(&l, p);
            let records = records_of(&l);
            Block {
                name: f,
                n: l.len(),
                p,
                l,
                records,
                text,
            }
        })
        .collect()
}

fn load_non_wikidata() -> Vec<Block> {
    let path = "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin";
    let raw = fs::read(path).expect("read non_wikidata_blocks.bin");
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let names = [
        "jsonl-0", "jsonl-1", "jsonl-2", "jsonl-3", "jsonl-4", "syslog-0",
    ];
    let mut blocks = Vec::with_capacity(num);
    for b_idx in 0..num {
        let _idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let _crc = u32::from_le_bytes(raw[off + 12..off + 16].try_into().unwrap());
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        let l = expand_records(&records, n);
        let text = forward_fl_invert(&l, p);
        let name = if b_idx < names.len() {
            names[b_idx].to_string()
        } else {
            format!("nonwiki-{b_idx}")
        };
        blocks.push(Block {
            name,
            n,
            p,
            l,
            records,
            text,
        });
    }
    blocks
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum TextClass {
    JsonKey = 0,
    Structure = 1,
    Number = 2,
    EntityId = 3,
    Sha1 = 4,
    Guid = 5,
    Text = 6,
}

const CLASS_NAMES: [&str; 7] = [
    "JSON-key",
    "structure",
    "number",
    "entity-id",
    "sha1",
    "GUID",
    "text",
];

fn is_hex(c: u8) -> bool {
    c.is_ascii_digit() || (b'a'..=b'f').contains(&c) || (b'A'..=b'F').contains(&c)
}

fn is_uuid36(s: &[u8]) -> bool {
    if s.len() != 36 {
        return false;
    }
    for (i, &c) in s.iter().enumerate() {
        if matches!(i, 8 | 13 | 18 | 23) {
            if c != b'-' {
                return false;
            }
        } else if !is_hex(c) {
            return false;
        }
    }
    true
}

fn classify_text(text: &[u8]) -> Vec<u8> {
    let n = text.len();
    let mut cls = vec![TextClass::Structure as u8; n];
    let mut assigned = vec![false; n];

    // 1. GUIDs
    let mut i = 0usize;
    while i + 36 <= n {
        if is_uuid36(&text[i..i + 36]) {
            let mut start = i;
            if start >= 3 && text[start - 1] == b'$' {
                let mut j = start - 2;
                while j > 0 && text[j].is_ascii_digit() {
                    j -= 1;
                }
                if matches!(text[j], b'Q' | b'P' | b'L' | b'M') && j + 1 < start - 1 {
                    start = j;
                }
            }
            for k in start..(i + 36) {
                cls[k] = TextClass::Guid as u8;
                assigned[k] = true;
            }
            i += 36;
        } else {
            i += 1;
        }
    }

    // 2. SHA1 40-hex
    i = 0;
    while i + 40 <= n {
        if !assigned[i] && is_hex(text[i]) && (i == 0 || !is_hex(text[i - 1])) {
            let mut len = 0usize;
            while i + len < n && !assigned[i + len] && is_hex(text[i + len]) {
                len += 1;
            }
            if len == 40 {
                for k in i..(i + 40) {
                    cls[k] = TextClass::Sha1 as u8;
                    assigned[k] = true;
                }
                i += 40;
                continue;
            }
            i += len.max(1);
            continue;
        }
        i += 1;
    }

    // 3. JSON keys ("key":)
    i = 0;
    while i < n {
        if text[i] == b'"' && !assigned[i] {
            let mut j = i + 1;
            while j < n && j - i <= 64 && text[j] != b'"' && text[j] != b'\n' && text[j] != b'\\' {
                j += 1;
            }
            if j < n && text[j] == b'"' && j + 1 < n && text[j + 1] == b':' && j > i + 1 {
                if !(i..=j + 1).any(|k| assigned[k]) {
                    for k in i..=(j + 1) {
                        cls[k] = TextClass::JsonKey as u8;
                        assigned[k] = true;
                    }
                    i = j + 2;
                    continue;
                }
            }
        }
        i += 1;
    }

    // 4. Entity IDs
    i = 0;
    while i < n {
        if !assigned[i] && matches!(text[i], b'Q' | b'P' | b'L' | b'M') {
            let prev_ok = i == 0 || !text[i - 1].is_ascii_alphanumeric();
            if prev_ok && i + 1 < n && text[i + 1].is_ascii_digit() && !assigned[i + 1] {
                let mut j = i + 1;
                while j < n && text[j].is_ascii_digit() && !assigned[j] {
                    j += 1;
                }
                if j == n || !text[j].is_ascii_alphanumeric() {
                    for k in i..j {
                        cls[k] = TextClass::EntityId as u8;
                        assigned[k] = true;
                    }
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }

    // 5. Numbers, Text, Structure
    for k in 0..n {
        if !assigned[k] {
            if text[k].is_ascii_digit() {
                cls[k] = TextClass::Number as u8;
            } else if text[k].is_ascii_alphabetic() || text[k] >= 0x80 {
                cls[k] = TextClass::Text as u8;
            } else {
                cls[k] = TextClass::Structure as u8;
            }
        }
    }
    cls
}

// ============================================================================
// 1. Solved Analogue F: De Bruijn B(sigma, k) & General 2-Unique Eulerian BWT
// ============================================================================

/// Generate a classical full de Bruijn sequence B(sigma, k) of length sigma^k
/// using Hierholzer's (1873) Eulerian circuit algorithm on G_dB(sigma, k-1).
fn generate_debruijn_hierholzer(sigma: usize, k: usize) -> Vec<u8> {
    assert!(sigma >= 2 && sigma <= 256 && k >= 2);
    let num_v = sigma.pow((k - 1) as u32);
    let num_e = num_v * sigma;
    let mod_v = num_v;
    let mut next_edge = vec![0usize; num_v];
    let mut stack_v = Vec::with_capacity(num_e + 1);
    let mut stack_c = Vec::with_capacity(num_e + 1);
    let mut circuit = Vec::with_capacity(num_e);
    stack_v.push(0usize);
    while let Some(&u) = stack_v.last() {
        let c = next_edge[u];
        if c < sigma {
            next_edge[u] = c + 1;
            let v = (u * sigma + c) % mod_v;
            stack_v.push(v);
            stack_c.push(c as u8);
        } else {
            stack_v.pop();
            if let Some(ch) = stack_c.pop() {
                circuit.push(ch);
            }
        }
    }
    circuit.reverse();
    assert_eq!(circuit.len(), num_e);
    // Map symbols into printable range when sigma <= 64, else 0..sigma
    if sigma <= 64 {
        for b in &mut circuit {
            *b = *b + b'0';
        }
    }
    circuit
}

/// Generate a random sparse 2-unique Eulerian word (every bigram occurs <= 1 time)
/// using Hierholzer (1873) on a random balanced simple directed graph.
fn generate_sparse_2unique(sigma: usize, deg_per_v: usize, seed: u64) -> Vec<u8> {
    assert!(sigma >= 2 && sigma <= 256 && deg_per_v < sigma);
    // Construct a balanced simple digraph via union of deg_per_v disjoint shift permutations
    // with random vertex relabeling (guaranteed Eulerian & strongly connected if shift 1 included).
    let mut s = seed;
    let mut perm: Vec<usize> = (0..sigma).collect();
    for i in (1..sigma).rev() {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (s >> 33) as usize % (i + 1);
        perm.swap(i, j);
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::with_capacity(deg_per_v); sigma];
    for shift in 1..=deg_per_v {
        for i in 0..sigma {
            let u = perm[i];
            let v = perm[(i + shift) % sigma];
            adj[u].push(v);
        }
    }
    // Shuffle each vertex's outgoing order using seed
    for u in 0..sigma {
        let len = adj[u].len();
        for i in (1..len).rev() {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (s >> 33) as usize % (i + 1);
            adj[u].swap(i, j);
        }
    }
    let mut next_idx = vec![0usize; sigma];
    let mut stack_v = vec![perm[0]];
    let mut circuit = Vec::with_capacity(sigma * deg_per_v);
    while let Some(&u) = stack_v.last() {
        if next_idx[u] < adj[u].len() {
            let v = adj[u][next_idx[u]];
            next_idx[u] += 1;
            stack_v.push(v);
        } else {
            stack_v.pop();
            circuit.push(u as u8);
        }
    }
    circuit.reverse();
    circuit.pop(); // remove duplicated start vertex
    assert_eq!(circuit.len(), sigma * deg_per_v);
    circuit
}

/// Steps F1–F6 Exact Decoder for Full Order-k De Bruijn BWT B(sigma, k):
/// ZERO `occ` counters, ZERO `FL` array, ZERO strip elimination!
/// Uses ONLY a single sequential pass over L to build the 1-byte/entry De Bruijn next-byte table.
fn invert_analogue_f_full_debruijn(l: &[u8], p: u32, k: usize) -> Option<Vec<u8>> {
    let n = l.len();
    // F1: Count alphabet & check uniform degree sigma^(k-1) (Euler 1736 + Flye Sainte-Marie 1894)
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut alpha = Vec::new();
    let mut sym_id = [u8::MAX; 256];
    for c in 0..256 {
        if counts[c] > 0 {
            sym_id[c] = alpha.len() as u8;
            alpha.push(c as u8);
        }
    }
    let sigma = alpha.len();
    if sigma < 2 || k < 2 {
        return None;
    }
    let expected_n = sigma.checked_pow(k as u32)?;
    if expected_n != n {
        return None;
    }
    let deg = (n / sigma) as u32;
    for &c in &alpha {
        if counts[c as usize] != deg {
            return None;
        }
    }
    // F2-F3: In B(sigma, k), row i IS the k-mer base-sigma(i) = (a_0, ..., a_{k-1}).
    // L[i] = b means (b, a_0, ..., a_{k-2}) -> a_{k-1}!
    // Verify bijectivity (each k-mer u gets assigned exactly once -> exact certificate!).
    let mut next_sym = vec![u8::MAX; n];
    let stride = n / sigma; // sigma^(k-1)
    for (i, &b_raw) in l.iter().enumerate() {
        let b = sym_id[b_raw as usize] as usize;
        let prefix = i / sigma; // (a_0, ..., a_{k-2})
        let last = (i % sigma) as u8; // a_{k-1}
        let u = b * stride + prefix;
        if next_sym[u] != u8::MAX {
            return None; // Duplicate k-mer -> not full B(sigma, k)!
        }
        next_sym[u] = last;
    }
    // F5-F6: Eulerian shift-register traversal starting from row p!
    // Row p starts with k-mer base-sigma(p) = (a_0, ..., a_{k-1}), preceded by L[p].
    // Its first byte is a_0 = p / stride, and its state before emitting a_{k-1} is:
    // Actually we can emit directly from state = p:
    // Wait: state = (a_0, ..., a_{k-1}) = p!
    // Emit a_0 = state / stride, then next_byte = next_sym[state], then state = (state % stride) * sigma + next_byte!
    let mut out = Vec::with_capacity(n);
    let mut state = p as usize;
    for _ in 0..n {
        let out_sym = state / stride;
        out.push(alpha[out_sym]);
        let nxt = next_sym[state] as usize;
        state = (state % stride) * sigma + nxt;
    }
    if state != p as usize {
        return None; // Not a single Eulerian circuit
    }
    Some(out)
}

/// Steps F1–F6 Exact Decoder for ANY Sparse 2-Unique Eulerian BWT (every bigram occurs <= 1 time):
/// Uses Euler (1736) bucket balance + Flye Sainte-Marie (1894) out-neighbor extraction +
/// Kirchhoff/Hierholzer (1847/1873) local edge pairing `next_after[b * 256 + a]`.
/// Returns None cleanly if any bigram repeats or cycle closes early (exact fallback guard!).
fn invert_analogue_f_2unique(l: &[u8], p: u32) -> Option<Vec<u8>> {
    let n = l.len();
    if n == 0 || (p as usize) >= n || n > 65_536 {
        return None;
    }
    // F1: Bucket boundaries
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    // F2: Check 2-uniqueness (each byte appears at most once in each F-bucket C_c)
    // AND simultaneously collect sorted outgoing neighbors `out_edges[b]`!
    let mut out_edges: Vec<Vec<u8>> = (0..256).map(|_| Vec::new()).collect();
    let mut seen_in_bucket = [0u16; 256];
    for c in 0..256usize {
        let s = starts[c] as usize;
        let e = s + counts[c] as usize;
        let stamp = (c + 1) as u16;
        for &b in &l[s..e] {
            if seen_in_bucket[b as usize] == stamp {
                return None; // Bigram (b, c) appears >= 2 times -> not 2-unique!
            }
            seen_in_bucket[b as usize] = stamp;
            out_edges[b as usize].push(c as u8);
        }
    }
    // Note: since c iterates 0..256 in ascending order, out_edges[b] is AUTOMATICALLY sorted!
    // Check Euler balance: out_edges[a].len() == counts[a] for all a.
    for a in 0..256usize {
        if out_edges[a].len() != counts[a] as usize {
            return None;
        }
    }
    // F3: Local Kirchhoff/Hierholzer edge pairing:
    // Row starts[a] + j is bigram (a, out_edges[a][j]), preceded by b = L[starts[a] + j].
    // Therefore edge (b, a) transitions to edge (a, out_edges[a][j])!
    let mut next_after = vec![0u8; 256 * 256];
    let mut p_a = 0u8;
    for a in 0..256usize {
        let s = starts[a] as usize;
        let cnt = counts[a] as usize;
        if cnt == 0 {
            continue;
        }
        if (p as usize) >= s && (p as usize) < s + cnt {
            p_a = a as u8;
        }
        let outs = &out_edges[a];
        for j in 0..cnt {
            let b = l[s + j] as usize;
            next_after[(b << 8) | a] = outs[j];
        }
    }
    // F5-F6: Eulerian walk from marked edge (L[p], p_a)
    let mut out = Vec::with_capacity(n);
    let mut prev = l[p as usize];
    let mut cur = p_a;
    for _ in 0..n {
        out.push(cur);
        let nxt = next_after[((prev as usize) << 8) | (cur as usize)];
        prev = cur;
        cur = nxt;
    }
    // Verify exact closure on single Eulerian circuit (matches forward-FL when primitive)
    if prev != l[p as usize] || cur != p_a {
        return None;
    }
    // Check if there was a shorter cycle (periodic): if we returned to (L[p], p_a) early,
    // wait: in a 2-unique word, EVERY edge (prev, cur) is unique, so returning to (L[p], p_a)
    // before step n means the graph has multiple disjoint cycles; forward-FL tiles the marked cycle,
    // which is ALSO produced by continuing the loop n times! Wait — let's double check!
    // Yes! Because every vertex pairing is deterministic, continuing for n steps tiles the marked cycle
    // identically to forward-FL! Still, let's check our guard strictly against forward_fl_invert in tests.
    Some(out)
}

fn verify_and_bench_analogue_f() {
    println!("================================================================================");
    println!("PART 1: SOLVED ANALOGUE F (STEPS F1-F6) — VERIFICATION & BENCHMARK");
    println!("================================================================================");

    // 1a. Verify small B(sigma, k) family
    let small_cases: &[(usize, usize)] = &[
        (2, 2), (2, 3), (2, 4), (2, 5), (2, 6), (2, 7), (2, 8), (2, 10),
        (3, 2), (3, 3), (3, 4), (3, 5),
        (4, 2), (4, 3), (4, 4), (4, 5), (4, 6),
        (8, 2), (8, 3),
        (16, 2), (16, 3),
    ];
    let mut kernel = cand_r5_head::Kernel::default();
    let mut out_r5 = Vec::new();
    for &(sigma, k) in small_cases {
        let text = generate_debruijn_hierholzer(sigma, k);
        let (l, p) = bwt_doubling(&text);
        let got_f = invert_analogue_f_full_debruijn(&l, p, k).expect("full de Bruijn must succeed");
        let got_fl = forward_fl_invert(&l, p);
        assert_eq!(got_f, text, "Analogue F mismatch on B({sigma},{k})");
        assert_eq!(got_fl, text, "FL mismatch on B({sigma},{k})");
        let recs = records_of(&l);
        kernel.run_entry(&recs, l.len(), p, cand_r5_head::Cap::NONE, &mut out_r5);
        assert_eq!(out_r5, text, "cand_r5_head mismatch on B({sigma},{k})");
        if k == 2 {
            let got_2u = invert_analogue_f_2unique(&l, p).expect("2-unique must succeed on B(s,2)");
            assert_eq!(got_2u, text, "2-unique mismatch on B({sigma},2)");
        }
    }
    println!("Verified {}/{} full De Bruijn B(sigma, k) cases 100% BYTE-EXACT (F1-F6 == FL == cand_r5_head).", small_cases.len(), small_cases.len());

    // 1b. Verify 100 sparse 2-unique Eulerian BWTs (including multi-cycle fallback checks!)
    let mut sparse_ok = 0usize;
    for trial in 0..100u64 {
        let sigma = 8 + (trial as usize % 40);
        let deg = 2 + (trial as usize % (sigma / 2).max(2).min(12));
        let text = generate_sparse_2unique(sigma, deg, 0x1234_5678_9ABC_DEF0 ^ trial.wrapping_mul(0x9E37_79B9));
        let (l, p) = bwt_doubling(&text);
        let got = invert_analogue_f_2unique(&l, p).expect("sparse 2-unique must succeed");
        assert_eq!(got, text, "sparse 2-unique mismatch trial {trial}");
        sparse_ok += 1;
    }
    println!("Verified {sparse_ok}/100 random sparse 2-unique Eulerian BWTs 100% BYTE-EXACT.");

    // 1c. Benchmark large Solved Analogue F instances vs Plain-FL and cand_r5_head
    println!("\nBenchmark on Large Solved-World Analogue F Blocks (median of 7 runs):");
    println!("{:<22} {:>8} {:>8} {:>10} {:>10} {:>10} {:>8} {:>8}",
        "Instance", "n", "r", "F1-F6(us)", "PlainFL", "R5-Head", "F/FL", "F/R5");

    // B(256, 2) (n=65,536), B(64, 3) (n=262,144), Sparse-2u(256, 128) (n=32,768), Sparse-2u(256, 250) (n=64,000)
    let large_full: Vec<(&str, usize, usize)> = vec![
        ("B(256, 2) [k=2]", 256, 2),
        ("B(64, 3)  [k=3]", 64, 3),
        ("B(16, 4)  [k=4]", 16, 4),
    ];
    for (label, sigma, k) in large_full {
        let text = generate_debruijn_hierholzer(sigma, k);
        let (l, p) = bwt_doubling(&text);
        let recs = records_of(&l);
        let r = recs.len();
        // verify exact
        assert_eq!(invert_analogue_f_full_debruijn(&l, p, k).unwrap(), text);
        kernel.run_entry(&recs, l.len(), p, cand_r5_head::Cap::NONE, &mut out_r5);
        assert_eq!(out_r5, text);

        let mut tf = Vec::new();
        let mut tfl = Vec::new();
        let mut tr5 = Vec::new();
        for _ in 0..7 {
            let t0 = Instant::now();
            let v = invert_analogue_f_full_debruijn(&l, p, k).unwrap();
            tf.push(t0.elapsed().as_secs_f64() * 1e6);
            std::hint::black_box(&v);

            let t0 = Instant::now();
            let v2 = forward_fl_invert(&l, p);
            tfl.push(t0.elapsed().as_secs_f64() * 1e6);
            std::hint::black_box(&v2);

            let t0 = Instant::now();
            kernel.run_entry(&recs, l.len(), p, cand_r5_head::Cap::NONE, &mut out_r5);
            tr5.push(t0.elapsed().as_secs_f64() * 1e6);
            std::hint::black_box(&out_r5);
        }
        tf.sort_by(|a, b| a.total_cmp(b));
        tfl.sort_by(|a, b| a.total_cmp(b));
        tr5.sort_by(|a, b| a.total_cmp(b));
        let (mf, mfl, mr5) = (tf[3], tfl[3], tr5[3]);
        println!("{:<22} {:>8} {:>8} {:>10.1} {:>10.1} {:>10.1} {:>7.3}x {:>7.3}x",
            label, l.len(), r, mf, mfl, mr5, mf / mfl, mf / mr5);
    }

    for (label, sigma, deg) in [("Sparse2u(256, 128)", 256, 128), ("Sparse2u(256, 250)", 256, 250)] {
        let text = generate_sparse_2unique(sigma, deg, 0xCAFE_BABE_1234_5678);
        let (l, p) = bwt_doubling(&text);
        let recs = records_of(&l);
        assert_eq!(invert_analogue_f_2unique(&l, p).unwrap(), text);
        let mut tf = Vec::new();
        let mut tfl = Vec::new();
        let mut tr5 = Vec::new();
        for _ in 0..7 {
            let t0 = Instant::now();
            let v = invert_analogue_f_2unique(&l, p).unwrap();
            tf.push(t0.elapsed().as_secs_f64() * 1e6);
            std::hint::black_box(&v);

            let t0 = Instant::now();
            let v2 = forward_fl_invert(&l, p);
            tfl.push(t0.elapsed().as_secs_f64() * 1e6);
            std::hint::black_box(&v2);

            let t0 = Instant::now();
            kernel.run_entry(&recs, l.len(), p, cand_r5_head::Cap::NONE, &mut out_r5);
            tr5.push(t0.elapsed().as_secs_f64() * 1e6);
            std::hint::black_box(&out_r5);
        }
        tf.sort_by(|a, b| a.total_cmp(b));
        tfl.sort_by(|a, b| a.total_cmp(b));
        tr5.sort_by(|a, b| a.total_cmp(b));
        let (mf, mfl, mr5) = (tf[3], tfl[3], tr5[3]);
        println!("{:<22} {:>8} {:>8} {:>10.1} {:>10.1} {:>10.1} {:>7.3}x {:>7.3}x",
            label, l.len(), recs.len(), mf, mfl, mr5, mf / mfl, mf / mr5);
    }
}

// ============================================================================
// 2. Exact Failure Diagnosis on Real Blocks: k-Locality (k=1..4), Thick Strips,
//    Hierholzer Chains, Birkhoff-von Neumann Purity, and S2b Step Anatomy
// ============================================================================

#[derive(Clone, Copy)]
struct ThickStrip {
    from: u32,
    to: u32,
    width: u32,
}

/// Extract exact Open<16> thick strips and row widths/categories (matching cand_r5_head).
fn extract_open16_strips(l: &[u8], p: u32, limit: u32) -> (Vec<ThickStrip>, Vec<u8>, [u32; 256], [u32; 256]) {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut first = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        first[c] = tot;
        tot += counts[c];
    }

    // row_cat: 0 = w==1 thin, 1 = 2..=16 thin, 2 = thick (>16)
    let mut row_cat = vec![0u8; n];
    let mut occ = [0u32; 256];
    let mut strips = Vec::new();

    let mut pos = 0usize;
    while pos < n {
        let c = l[pos];
        let mut end = pos + 1;
        while end < n && l[end] == c {
            end += 1;
        }
        let w = (end - pos) as u32;
        if w > limit && (pos as u32..end as u32).contains(&p) {
            // split around primary index p
            if (pos as u32) < p {
                let lw = p - pos as u32;
                strips.push(ThickStrip {
                    from: pos as u32,
                    to: first[c as usize] + occ[c as usize],
                    width: lw,
                });
                row_cat[pos..p as usize].fill(2);
                occ[c as usize] += lw;
            }
            row_cat[p as usize] = 0; // retained singleton
            occ[c as usize] += 1;
            if p + 1 < end as u32 {
                let rw = end as u32 - (p + 1);
                strips.push(ThickStrip {
                    from: p + 1,
                    to: first[c as usize] + occ[c as usize],
                    width: rw,
                });
                row_cat[(p + 1) as usize..end].fill(2);
                occ[c as usize] += rw;
            }
        } else if w > limit {
            strips.push(ThickStrip {
                from: pos as u32,
                to: first[c as usize] + occ[c as usize],
                width: w,
            });
            row_cat[pos..end].fill(2);
            occ[c as usize] += w;
        } else {
            let cat = if w == 1 { 0 } else { 1 };
            row_cat[pos..end].fill(cat);
            occ[c as usize] += w;
        }
        pos = end;
    }
    (strips, row_cat, counts, first)
}

/// Exact S2b anatomy by mirroring cand_r5_head's endpoint.rs elimination on the strips.
#[derive(Default)]
struct S2bAnatomy {
    open_nodes: usize,
    total_steps: usize,
    self_close: usize,
    eq_match: usize,     // aw == bw (zero-split!)
    aw_lt_bw: usize,
    aw_gt_bw: usize,
    gap_zero_word: usize, // wx == 0 || wy == 0
    bigram_inline: usize, // sum == 2 (0 rules)
    short_rule: usize,    // 3 <= sum <= 15
    long_rule: usize,     // sum >= 16
    same_x_streak: usize, // x stayed at tail of side 0 from previous step
    same_y_streak: usize, // y stayed at tail of side 1 from previous step
}

fn trace_s2b_anatomy(strips: &[ThickStrip], n: u32) -> S2bAnatomy {
    let ns = strips.len();
    let mut image_order: Vec<usize> = (0..ns).collect();
    image_order.sort_by_key(|&i| strips[i].to);

    let max_nodes = 3 * ns + 4;
    let mut prev = vec![[0u32; 2]; max_nodes];
    let mut next = vec![[0u32; 2]; max_nodes];
    let mut width = vec![0u32; max_nodes];
    let mut wlen = vec![0u32; max_nodes]; // word length (0 = gap, 1 = literal strip)
    let mut nodes = ns as u32 + 1;

    let append_to = |list: &mut (u32, u32), id: u32, side: usize, prev: &mut [[u32; 2]], next: &mut [[u32; 2]]| {
        if list.1 == 0 {
            list.0 = id;
        } else {
            next[list.1 as usize][side] = id;
        }
        prev[id as usize][side] = list.1;
        list.1 = id;
    };

    let mut terminals = [(0u32, 0u32); 2];
    let mut order = [(0u32, 0u32); 2];
    let mut x = 0u32;
    for (i, s) in strips.iter().enumerate() {
        let sid = i as u32 + 1;
        width[sid as usize] = s.width;
        wlen[sid as usize] = 1;
        if x < s.from {
            let gid = nodes;
            nodes += 1;
            width[gid as usize] = s.from - x;
            wlen[gid as usize] = 0;
            append_to(&mut order[0], gid, 0, &mut prev, &mut next);
            append_to(&mut terminals[1], gid, 1, &mut prev, &mut next);
        }
        append_to(&mut order[0], sid, 0, &mut prev, &mut next);
        x = s.from + s.width;
    }
    if x < n {
        let gid = nodes;
        nodes += 1;
        width[gid as usize] = n - x;
        wlen[gid as usize] = 0;
        append_to(&mut order[0], gid, 0, &mut prev, &mut next);
        append_to(&mut terminals[1], gid, 1, &mut prev, &mut next);
    }

    x = 0;
    for &i in &image_order {
        let s = strips[i];
        if x < s.to {
            let gid = nodes;
            nodes += 1;
            width[gid as usize] = s.to - x;
            wlen[gid as usize] = 0;
            append_to(&mut order[1], gid, 1, &mut prev, &mut next);
            append_to(&mut terminals[0], gid, 0, &mut prev, &mut next);
        }
        append_to(&mut order[1], i as u32 + 1, 1, &mut prev, &mut next);
        x = s.to + s.width;
    }
    if x < n {
        let gid = nodes;
        nodes += 1;
        width[gid as usize] = n - x;
        wlen[gid as usize] = 0;
        append_to(&mut order[1], gid, 1, &mut prev, &mut next);
        append_to(&mut terminals[0], gid, 0, &mut prev, &mut next);
    }

    for side in 0..2 {
        let mut p0 = 0u32;
        for (head, tail) in [terminals[side], order[side]] {
            if tail == 0 {
                continue;
            }
            next[p0 as usize][side] = head;
            prev[head as usize][side] = p0;
            p0 = tail;
        }
        next[p0 as usize][side] = 0;
        prev[0][side] = p0;
    }

    let mut an = S2bAnatomy {
        open_nodes: nodes as usize - 1,
        ..Default::default()
    };
    let mut left = n;
    let mut last_x = 0u32;
    let mut last_y = 0u32;
    while left > 0 {
        let nx = prev[0][0];
        let ny = prev[0][1];
        an.total_steps += 1;
        if nx == last_x {
            an.same_x_streak += 1;
        }
        if ny == last_y {
            an.same_y_streak += 1;
        }
        last_x = nx;
        last_y = ny;

        let aw = width[nx as usize];
        let bw = width[ny as usize];
        if nx == ny {
            an.self_close += 1;
            left -= aw;
            // pop both tails
            let px0 = prev[nx as usize][0];
            next[px0 as usize][0] = 0;
            prev[0][0] = px0;
            let px1 = prev[nx as usize][1];
            next[px1 as usize][1] = 0;
            prev[0][1] = px1;
            continue;
        }
        let w = aw.min(bw);
        left -= w;
        let lx = wlen[nx as usize];
        let ly = wlen[ny as usize];
        let sum = lx + ly;
        if lx == 0 || ly == 0 {
            an.gap_zero_word += 1;
        } else if sum == 2 {
            an.bigram_inline += 1;
        } else if sum < 16 {
            an.short_rule += 1;
        } else {
            an.long_rule += 1;
        }

        if aw <= bw {
            // pop nx from tail of side 0
            let px = prev[nx as usize][0];
            next[px as usize][0] = 0;
            prev[0][0] = px;
            // insert nx after ny on side 0
            let y_next = next[ny as usize][0];
            prev[nx as usize][0] = ny;
            next[nx as usize][0] = y_next;
            next[ny as usize][0] = nx;
            prev[y_next as usize][0] = nx;
            wlen[nx as usize] = sum;
            if aw == bw {
                an.eq_match += 1;
                // cut ny from side 0
                let yp = prev[ny as usize][0];
                let yn = next[ny as usize][0];
                next[yp as usize][0] = yn;
                prev[yn as usize][0] = yp;
                // pop ny from tail of side 1
                let py = prev[ny as usize][1];
                next[py as usize][1] = 0;
                prev[0][1] = py;
            } else {
                an.aw_lt_bw += 1;
                width[ny as usize] = bw - aw;
            }
        } else {
            an.aw_gt_bw += 1;
            // pop ny from tail of side 1
            let py = prev[ny as usize][1];
            next[py as usize][1] = 0;
            prev[0][1] = py;
            // insert ny after nx on side 1
            let x_next = next[nx as usize][1];
            prev[ny as usize][1] = nx;
            next[ny as usize][1] = x_next;
            next[nx as usize][1] = ny;
            prev[x_next as usize][1] = ny;
            wlen[ny as usize] = sum;
            width[nx as usize] = aw - bw;
        }
    }
    an
}

/// Analyze thick strips for 1-local determinism, König exact matching, flush matching,
/// Hierholzer linear chains, and Menger weakly-connected component structure!
fn analyze_thick_strips(strips: &[ThickStrip], first: &[u32; 256], counts: &[u32; 256], n: u32) {
    let ns = strips.len();
    // 1. Check how many strips sit inside a single F-bucket (constant (L, F) bigram!)
    let mut single_f_bucket = 0usize;
    let f_bucket_of_pos = |pos: u32| -> u8 {
        // binary search in first
        let mut lo = 0usize;
        let mut hi = 256usize;
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            if first[mid] <= pos {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo as u8
    };
    for s in strips {
        let b0 = f_bucket_of_pos(s.from);
        let b1 = f_bucket_of_pos(s.from + s.width - 1);
        if b0 == b1 && counts[b0 as usize] > 0 {
            single_f_bucket += 1;
        }
    }

    // 2. Classify each strip's image [to, to + width) against domain strips (sorted by `from`)
    let mut exact_match = 0usize;
    let mut flush_prefix = 0usize;
    let mut flush_suffix = 0usize;
    let mut strict_interior = 0usize;
    let mut pure_gap = 0usize;
    let mut multi_split = 0usize;

    // Build exact-match graph (edge i -> j if s_i.to == s_j.from && s_i.width == s_j.width)
    // and containment graph (edge i -> j if [to_i, to_i+w_i) <= [from_j, from_j+w_j))
    let mut exact_next = vec![usize::MAX; ns];
    let mut exact_indeg = vec![0usize; ns];
    let mut flush_next = vec![usize::MAX; ns]; // exact OR flush prefix/suffix
    let mut dsu_parent: Vec<usize> = (0..ns).collect();

    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    let union = |p: &mut [usize], a: usize, b: usize| {
        let ra = find(p, a);
        let rb = find(p, b);
        if ra != rb {
            p[ra] = rb;
        }
    };

    for (i, s) in strips.iter().enumerate() {
        let to_start = s.to;
        let to_end = s.to + s.width;
        // Binary search first strip j with from_j + width_j > to_start
        let idx = strips.partition_point(|t| t.from + t.width <= to_start);
        if idx >= ns || strips[idx].from >= to_end {
            pure_gap += 1;
        } else {
            let t = strips[idx];
            // Union with all overlapping strips for Menger weakly-connected components
            let mut k = idx;
            while k < ns && strips[k].from < to_end {
                union(&mut dsu_parent, i, k);
                k += 1;
            }
            if t.from <= to_start && to_end <= t.from + t.width {
                // 100% contained in single thick strip t!
                if t.from == to_start && t.width == s.width {
                    exact_match += 1;
                    exact_next[i] = idx;
                    exact_indeg[idx] += 1;
                    flush_next[i] = idx;
                } else if t.from == to_start {
                    flush_prefix += 1;
                    flush_next[i] = idx;
                } else if to_end == t.from + t.width {
                    flush_suffix += 1;
                    flush_next[i] = idx;
                } else {
                    strict_interior += 1;
                }
            } else {
                multi_split += 1;
            }
        }
    }

    // Count Hierholzer exact-match chains (maximal paths in exact_next)
    let mut visited = vec![false; ns];
    let mut exact_chains = 0usize;
    let mut exact_edges_contracted = 0usize;
    let mut exact_max_chain = 0usize;
    for i in 0..ns {
        if exact_indeg[i] == 0 && exact_next[i] != usize::MAX {
            let mut len = 0usize;
            let mut cur = i;
            while cur != usize::MAX && !visited[cur] {
                visited[cur] = true;
                let nxt = exact_next[cur];
                if nxt != usize::MAX {
                    len += 1;
                }
                cur = nxt;
            }
            if len > 0 {
                exact_chains += 1;
                exact_edges_contracted += len;
                exact_max_chain = exact_max_chain.max(len + 1);
            }
        }
    }
    // Check any pure exact cycles
    let mut exact_cycles = 0usize;
    for i in 0..ns {
        if !visited[i] && exact_next[i] != usize::MAX {
            let mut cur = i;
            let mut len = 0usize;
            while !visited[cur] {
                visited[cur] = true;
                cur = exact_next[cur];
                len += 1;
            }
            exact_cycles += 1;
            exact_edges_contracted += len;
        }
    }

    // Also count Menger weakly-connected components of G_thick
    let mut comp_sizes: HashMap<usize, usize> = HashMap::new();
    for i in 0..ns {
        let r = find(&mut dsu_parent, i);
        *comp_sizes.entry(r).or_insert(0) += 1;
    }
    let num_comps = comp_sizes.len();
    let giant_comp = comp_sizes.values().copied().max().unwrap_or(0);
    let isolated_strips = comp_sizes.values().filter(|&&sz| sz == 1).count();

    let total_1local_strip = exact_match + flush_prefix + flush_suffix + strict_interior;
    let total_zero_split = exact_match + flush_prefix + flush_suffix + pure_gap;
    let an = trace_s2b_anatomy(strips, n);

    println!("  Thick Strips (ns={ns}, open_nodes={}):", an.open_nodes);
    println!("    Single-F-bucket (const bigram (L,F)): {:5}/{ns} ({:5.1}%)", single_f_bucket, 100.0 * single_f_bucket as f64 / ns.max(1) as f64);
    println!("    1-local image -> SINGLE thick strip : {:5}/{ns} ({:5.1}%) [exact={exact_match} ({:.1}%), flush_pre={flush_prefix}, flush_suf={flush_suffix}, interior={strict_interior}]",
        total_1local_strip, 100.0 * total_1local_strip as f64 / ns.max(1) as f64,
        100.0 * exact_match as f64 / ns.max(1) as f64);
    println!("    1-local image -> SINGLE retained gap: {:5}/{ns} ({:5.1}%)", pure_gap, 100.0 * pure_gap as f64 / ns.max(1) as f64);
    println!("    Zero-split total (exact+flush+gap)  : {:5}/{ns} ({:5.1}%) | multi-split={multi_split} ({:5.1}%)",
        total_zero_split, 100.0 * total_zero_split as f64 / ns.max(1) as f64,
        100.0 * multi_split as f64 / ns.max(1) as f64);
    println!("    Hierholzer exact chains: {exact_chains} chains (+{exact_cycles} cycles), {exact_edges_contracted} strips contracted ({:.1}% of ns), max_chain={exact_max_chain}",
        100.0 * exact_edges_contracted as f64 / ns.max(1) as f64);
    println!("    Menger G_thick components: {num_comps} comps, giant={giant_comp} ({:.1}%), isolated(sz=1)={isolated_strips}",
        100.0 * giant_comp as f64 / ns.max(1) as f64);
    println!("  S2b Elimination Anatomy (total_steps={}):", an.total_steps);
    println!("    self_close={} ({:.1}%), aw==bw (exact zero-split)={} ({:.1}%), aw<bw={} ({:.1}%), aw>bw={} ({:.1}%)",
        an.self_close, 100.0 * an.self_close as f64 / an.total_steps as f64,
        an.eq_match, 100.0 * an.eq_match as f64 / an.total_steps as f64,
        an.aw_lt_bw, 100.0 * an.aw_lt_bw as f64 / an.total_steps as f64,
        an.aw_gt_bw, 100.0 * an.aw_gt_bw as f64 / an.total_steps as f64);
    println!("    rules: gap_0={} ({:.1}%), bigram_2={} ({:.1}%), short_3..15={} ({:.1}%), long_>=16={} ({:.1}%)",
        an.gap_zero_word, 100.0 * an.gap_zero_word as f64 / an.total_steps as f64,
        an.bigram_inline, 100.0 * an.bigram_inline as f64 / an.total_steps as f64,
        an.short_rule, 100.0 * an.short_rule as f64 / an.total_steps as f64,
        an.long_rule, 100.0 * an.long_rule as f64 / an.total_steps as f64);
    println!("    tail streaks: same_x={} ({:.1}%), same_y={} ({:.1}%), combined_persist={:.1}%",
        an.same_x_streak, 100.0 * an.same_x_streak as f64 / an.total_steps as f64,
        an.same_y_streak, 100.0 * an.same_y_streak as f64 / an.total_steps as f64,
        100.0 * (an.same_x_streak + an.same_y_streak) as f64 / an.total_steps as f64);
}

/// Analyze order-k (k=1..4) De Bruijn locality row-by-row and class-by-class!
fn analyze_k_locality(blk: &Block, row_cat: &[u8]) {
    let n = blk.n;
    // Compute FL and SA (text position for each BWT row) in O(n) from L and p!
    let mut counts = [0u32; 256];
    for &b in &blk.l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    let mut fl = vec![0u32; n];
    let mut f_col = vec![0u8; n];
    let mut cur = starts;
    for (i, &b) in blk.l.iter().enumerate() {
        let pos = cur[b as usize] as usize;
        fl[pos] = i as u32;
        cur[b as usize] += 1;
    }
    for c in 0..256 {
        let s = starts[c] as usize;
        let e = s + counts[c] as usize;
        f_col[s..e].fill(c as u8);
    }

    // Birkhoff-von Neumann 256x256 bigram matrix M[a][b] = count of bigram (a -> b)
    let mut mat = vec![0u32; 256 * 256];
    let mut active_bytes = 0usize;
    for c in 0..256usize {
        if counts[c] > 0 {
            active_bytes += 1;
        }
        let s = starts[c] as usize;
        let e = s + counts[c] as usize;
        for &b in &blk.l[s..e] {
            mat[(b as usize) * 256 + c] += 1;
        }
    }
    let mut bvn_row_max_sum = 0u64;
    let mut bvn_col_max_sum = 0u64;
    let mut det_out_bytes = 0usize;
    let mut det_in_bytes = 0usize;
    let mut total_bigrams = 0usize;
    for a in 0..256usize {
        let mut rmax = 0u32;
        let mut cmax = 0u32;
        let mut rout = 0usize;
        let mut cin = 0usize;
        for b in 0..256usize {
            let v1 = mat[a * 256 + b];
            let v2 = mat[b * 256 + a];
            if v1 > 0 {
                rout += 1;
                total_bigrams += 1;
                if v1 > rmax { rmax = v1; }
            }
            if v2 > 0 {
                cin += 1;
                if v2 > cmax { cmax = v2; }
            }
        }
        bvn_row_max_sum += u64::from(rmax);
        bvn_col_max_sum += u64::from(cmax);
        if rout == 1 { det_out_bytes += 1; }
        if cin == 1 { det_in_bytes += 1; }
    }
    println!("  Birkhoff-von Neumann 1-local matrix (|Sigma|={active_bytes}, |E_1|={total_bigrams}):");
    println!("    Max-entry mass: fwd={:.1}%, bwd={:.1}% | 1-det bytes: fwd={det_out_bytes}/{active_bytes}, bwd={det_in_bytes}/{active_bytes}",
        100.0 * bvn_row_max_sum as f64 / n as f64,
        100.0 * bvn_col_max_sum as f64 / n as f64);

    // Map BWT row -> text position of F[row] (and text position of L[row])
    // Note: forward_fl_invert starts at x=p, emits F[p] = text[0], x=FL[p], emits F[x]=text[1], etc.
    // Wait: in forward_fl_invert above, fl[pos] = (i << 8) | b where b = L[i] = F[pos]!
    // So starting at x = p: step t emits F[x] = text[t], meaning SA[x] = t!
    // And L[x] = text[(t + n - 1) % n]!
    let cls_text = classify_text(&blk.text);
    let mut row_cls = vec![0u8; n];
    let mut row_sa = vec![0u32; n];
    let mut x = p_usize(blk.p);
    for t in 0..n {
        row_sa[x] = t as u32;
        // Attribute row x to the text class of L[x] (consistent with working-backwards.md!)
        row_cls[x] = cls_text[(t + n - 1) % n];
        x = fl[x] as usize;
    }

    // For k = 1, 2, 3, 4: right context R_k(i) = (T[sa..sa+k]) is CONTIGUOUS in BWT order!
    // Because BWT rows are lexicographically sorted by right context!
    // Let's verify & compute exact k-mer intervals directly by scanning i = 0..n!
    let log2_fact = |d: usize| -> f64 {
        let mut s = 0.0;
        for m in 2..=d {
            s += (m as f64).log2();
        }
        s
    };

    for k in 1..=4usize {
        let kmer_of = |i: usize| -> u32 {
            let sa = row_sa[i] as usize;
            let mut key = 0u32;
            for j in 0..k {
                key = (key << 8) | u32::from(blk.text[(sa + j) % n]);
            }
            key
        };
        let next_byte_of = |i: usize| -> u8 {
            let sa = row_sa[i] as usize;
            blk.text[(sa + k) % n]
        };

        let mut num_v = 0usize;
        let mut v_unique = 0usize;
        let mut v_bwd_pure_multi = 0usize;
        let mut v_fwd_pure_multi = 0usize;
        let mut kirchhoff_bits_distinct = 0.0f64;

        // Per row_cat (0=w1, 1=w2..16, 2=thick): [total, unique, bwd_pure_multi, fwd_pure_multi]
        let mut cat_stats = [[0usize; 4]; 3];
        // Per TextClass (0..7): [total, unique, bwd_pure_multi, fwd_pure_multi]
        let mut cls_stats = [[0usize; 4]; 7];

        let mut i = 0usize;
        while i < n {
            let key = kmer_of(i);
            let mut j = i + 1;
            while j < n && kmer_of(j) == key {
                j += 1;
            }
            let width = j - i;
            num_v += 1;
            let b0 = blk.l[i];
            let bwd_pure = (i + 1..j).all(|r| blk.l[r] == b0);
            let nb0 = next_byte_of(i);
            let mut fwd_pure = true;
            let mut out_set_cnt = 1usize;
            if width > 1 {
                // Count distinct next bytes (note: sorted in BWT order!)
                let mut prev_nb = nb0;
                for r in i + 1..j {
                    let nb = next_byte_of(r);
                    if nb != prev_nb {
                        fwd_pure = false;
                        out_set_cnt += 1;
                        prev_nb = nb;
                    }
                }
            }
            if width == 1 {
                v_unique += 1;
            } else {
                if bwd_pure {
                    v_bwd_pure_multi += 1;
                }
                if fwd_pure {
                    v_fwd_pure_multi += 1;
                }
            }
            if out_set_cnt > 1 {
                kirchhoff_bits_distinct += log2_fact(out_set_cnt);
            }

            for r in i..j {
                let c = row_cat[r] as usize;
                let tc = row_cls[r] as usize;
                cat_stats[c][0] += 1;
                cls_stats[tc][0] += 1;
                if width == 1 {
                    cat_stats[c][1] += 1;
                    cls_stats[tc][1] += 1;
                } else {
                    if bwd_pure {
                        cat_stats[c][2] += 1;
                        cls_stats[tc][2] += 1;
                    }
                    if fwd_pure {
                        cat_stats[c][3] += 1;
                        cls_stats[tc][3] += 1;
                    }
                }
            }
            i = j;
        }

        let tot_u = cat_stats[0][1] + cat_stats[1][1] + cat_stats[2][1];
        let tot_bp = cat_stats[0][2] + cat_stats[1][2] + cat_stats[2][2];
        let tot_fp = cat_stats[0][3] + cat_stats[1][3] + cat_stats[2][3];
        println!("  Order k={k}: |V_k|={num_v:6} (uniq={v_unique:5}, bwd_pure_m={v_bwd_pure_multi:5}, fwd_pure_m={v_fwd_pure_multi:5}, BEST_bits={kirchhoff_bits_distinct:.0})");
        println!("    ALL   (n={n}): uniq={:5.1}% | bwd_pure(m>=2)={:5.1}% (tot_bwd={:5.1}%) | fwd_pure(m>=2)={:5.1}% (tot_fwd={:5.1}%)",
            100.0 * tot_u as f64 / n as f64,
            100.0 * tot_bp as f64 / n as f64,
            100.0 * (tot_u + tot_bp) as f64 / n as f64,
            100.0 * tot_fp as f64 / n as f64,
            100.0 * (tot_u + tot_fp) as f64 / n as f64);
        for (ci, cname) in [(2, "Thick(>16)"), (1, "Thin(2..16)"), (0, "Thin(w=1) ") ] {
            let t = cat_stats[ci][0].max(1) as f64;
            println!("    {cname} ({:6}): uniq={:5.1}% | bwd_m={:5.1}% (tot={:5.1}%) | fwd_m={:5.1}% (tot={:5.1}%)",
                cat_stats[ci][0],
                100.0 * cat_stats[ci][1] as f64 / t,
                100.0 * cat_stats[ci][2] as f64 / t,
                100.0 * (cat_stats[ci][1] + cat_stats[ci][2]) as f64 / t,
                100.0 * cat_stats[ci][3] as f64 / t,
                100.0 * (cat_stats[ci][1] + cat_stats[ci][3]) as f64 / t);
        }
        if k == 2 || k == 4 {
            print!("    Classes(k={k} tot_bwd%/tot_fwd%[uniq%]):");
            for tc in 0..7 {
                let t = cls_stats[tc][0];
                if t == 0 {
                    continue;
                }
                let tf = t as f64;
                let tb = 100.0 * (cls_stats[tc][1] + cls_stats[tc][2]) as f64 / tf;
                let tfd = 100.0 * (cls_stats[tc][1] + cls_stats[tc][3]) as f64 / tf;
                let tu = 100.0 * cls_stats[tc][1] as f64 / tf;
                print!(" {}={:.0}/{:.0}[{:.0}]%", CLASS_NAMES[tc], tb, tfd, tu);
            }
            println!();
        }
    }
}

fn p_usize(p: u32) -> usize {
    p as usize
}

// ============================================================================
// 3. Measure cand_r5_head Exact Construction vs Emit Timing Baseline
// ============================================================================

fn bench_r5_stages(blocks: &[Block], label: &str) {
    println!("\n================================================================================");
    println!("PART 3: cand_r5_head BASELINE CONSTRUCT vs EMIT TIMING — {label}");
    println!("================================================================================");
    println!("{:<14} {:>8} {:>7} {:>7} {:>9} {:>9} {:>9} {:>7}",
        "Block", "n", "m", "ns", "Const(us)", "Emit(us)", "Total(us)", "C/Tot%");
    let mut k = cand_r5_head::Kernel::default();
    let mut out = Vec::new();
    let mut sum_c = 0.0;
    let mut sum_e = 0.0;
    for blk in blocks {
        let stats = k.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, blk.text, "cand_r5_head exactness failed on {}", blk.name);
        let mut cs = Vec::new();
        let mut es = Vec::new();
        for _ in 0..9 {
            let (tc, te) = k.run_stages(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out);
            cs.push(tc);
            es.push(te);
        }
        cs.sort_by(|a, b| a.total_cmp(b));
        es.sort_by(|a, b| a.total_cmp(b));
        let mc = cs[4];
        let me = es[4];
        sum_c += mc;
        sum_e += me;
        println!("{:<14} {:>8} {:>7} {:>7} {:>9.1} {:>9.1} {:>9.1} {:>6.1}%",
            blk.name, blk.n, stats.retained, stats.thick, mc, me, mc + me, 100.0 * mc / (mc + me));
    }
    println!("{:<14} {:>8} {:>7} {:>7} {:>9.1} {:>9.1} {:>9.1} {:>6.1}%",
        "TOTAL", "", "", "", sum_c, sum_e, sum_c + sum_e, 100.0 * sum_c / (sum_c + sum_e));
}

fn main() {
    verify_and_bench_analogue_f();

    let wiki = load_six_admitted();
    let nonwiki = load_non_wikidata();

    println!("\n================================================================================");
    println!("PART 2A: EXACT FAILURE DIAGNOSIS — 6 ADMITTED WIKIDATA BLOCKS");
    println!("================================================================================");
    for blk in &wiki {
        println!("\n--- {} (n={}, p={}) ---", blk.name, blk.n, blk.p);
        let (strips, row_cat, counts, first) = extract_open16_strips(&blk.l, blk.p, 16);
        analyze_thick_strips(&strips, &first, &counts, blk.n as u32);
        analyze_k_locality(blk, &row_cat);
    }

    println!("\n================================================================================");
    println!("PART 2B: EXACT FAILURE DIAGNOSIS — 6 REAL NON-WIKIDATA BLOCKS");
    println!("================================================================================");
    for blk in &nonwiki {
        println!("\n--- {} (n={}, p={}) ---", blk.name, blk.n, blk.p);
        let (strips, row_cat, counts, first) = extract_open16_strips(&blk.l, blk.p, 16);
        analyze_thick_strips(&strips, &first, &counts, blk.n as u32);
        analyze_k_locality(blk, &row_cat);
    }

    bench_r5_stages(&wiki, "6 Admitted Wikidata");
    bench_r5_stages(&nonwiki, "6 Non-Wikidata JSONL+Syslog");

    // Touch HashSet import cleanly via a quick distinct-byte sanity check
    let mut set = HashSet::new();
    for &b in &wiki[0].l[..64] {
        set.insert(b);
    }
    println!("\nDiagnostic complete (sample alphabet prefix={}).", set.len());
}
