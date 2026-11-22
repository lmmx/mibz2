#![forbid(unsafe_code)]

//! Decisive Cycle 47 Escape Test — Analogue E: Selective High-Yield Ephemeral
//! Copy-Ribbon Fusion Directly Over `cand_r5_head/`.
//!
//! Cycle 47 — Programme Investigator P1 (`research/carbon-kitchen-sink`).
//!
//! Verifies:
//! 1. Hard Exactness Gate:
//!    - 5,200+ small valid BWTs (exhaustive binary/ternary + random + periodic + Analogue E)
//!    - 250 malformed multi-cycle columns verified byte-for-byte against `cand_r5_head` (forward-FL)
//!    - Verified across BOTH MLP=1 and MLP=8, BOTH Unidirectional and Bidirectional-XOR!
//!    - 6 admitted Wikidata blocks + 6 real Non-Wikidata JSONL/log blocks + Solved Analogue E (300k/900k)
//! 2. Stage-by-Stage Timing Decomposition & Cost Barrier Micro-Accounting:
//!    - S1 / S2+S3 / S4 / S4.5_ribbon (MLP=1 vs MLP=8) / S4.6_xor / S5_emit
//!    - Exact ns/probe, ns/trace-step, ns/hop-saved vs Xeon S5 and i9 #145 (5.79 ns/row)
//! 3. Same-Binary Paired Alternating 1T / 2W / 4W Benchmarks with A/A Control:
//!    - `R5-Head-A1` vs `R5-Head-A2` (A/A) vs `R5-Nobi` vs `Ctrl-Nobi` vs MLP=8 Ephemeral Ribbon configs
//!    - Exact `C_recognise + C_member < C_Open` report across Wikidata, Non-Wikidata, and Analogue E.

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_nobi/mod.rs"]
mod cand_r5_nobi;

mod ribbon_kernel;

use ribbon_kernel::{RibbonConfig, StageTimesUs};
use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    expected: Vec<u8>,
}

// ============================================================================
// 1. Helpers, Forward-FL Reference, Synthetic & Real Corpus Loaders
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
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
        }
        l.push(text[(pos + n - 1) % n]);
    }
    (l, p)
}

fn build_bwt_fast(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rank: Vec<u32> = text.iter().map(|&b| u32::from(b)).collect();
    let mut tmp = vec![0u32; n];
    let mut cnt = vec![0usize; n.max(256)];
    let mut order = vec![0usize; n];

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
            if rank[a] != rank[b] || rank[(a + k) % n] != rank[(b + k) % n] {
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
    let (l_bytes, p) = build_bwt_fast(&text);
    let records = records_of(&l_bytes);
    Block {
        name: name.to_string(),
        n: l_bytes.len(),
        p,
        records,
        expected: text,
    }
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
                expected,
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
            expected,
        });
    }
    assert_eq!(off, raw.len());
    blocks
}

// ============================================================================
// 2. Hard Exactness Gate (5,200+ Small Valid + 250 Malformed Forward-FL)
// ============================================================================

fn run_exactness_gate(configs: &[RibbonConfig]) {
    let mut r5 = cand_r5_head::Kernel::default();
    let mut rk = ribbon_kernel::Kernel::default();
    let mut out_r5 = Vec::new();
    let mut out_rk = Vec::new();

    // Stress-test aggressive ribbon contraction (w_min=2, min_steps=2, min_yield=2)
    // across BOTH MLP=1 and MLP=8, and BOTH unidirectional and forced-bidirectional!
    let stress_uni_m1 = RibbonConfig {
        name: "Stress-Uni-M1",
        enabled: true,
        use_mlp8: false,
        w_min: 2,
        stride_mask: 0,
        min_steps: 2,
        min_yield: 2,
        l1_dict_cap_bytes: 1 << 20,
        added_spell_cap_bytes: 1 << 20,
        use_bi_xor: false,
        wald_sprt: false,
        wald_window: 64,
        wald_min_hits: 0,
        force_bi_small: false,
    };
    let stress_uni_m8 = RibbonConfig {
        name: "Stress-Uni-M8",
        use_mlp8: true,
        ..stress_uni_m1
    };
    let stress_bi_m8 = RibbonConfig {
        name: "Stress-Bi-M8",
        use_mlp8: true,
        use_bi_xor: true,
        force_bi_small: true,
        ..stress_uni_m1
    };
    let stress_list = [&stress_uni_m1, &stress_uni_m8, &stress_bi_m8];

    let mut valid_count = 0usize;
    let mut malformed_count = 0usize;
    let mut rng = XorShift64(0xC47E_5CA9_E111_0001);

    // 2A. Exhaustive binary columns length 1..=8 (510 columns)
    for n in 1..=8usize {
        for mask in 0..(1u32 << n) {
            let mut l = Vec::with_capacity(n);
            for bit in 0..n {
                l.push(if ((mask >> bit) & 1) == 0 { b'a' } else { b'b' });
            }
            let recs = records_of(&l);
            let p = (mask as usize % n) as u32;
            let ref_fl = plain_invert_fl(&l, p);
            r5.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_r5)
                .expect("r5");
            assert_eq!(out_r5, ref_fl);
            for cfg in &stress_list {
                rk.run(&recs, n, p, cfg, &mut out_rk);
                assert_eq!(out_rk, out_r5, "{} mismatch on binary n={n} mask={mask}", cfg.name);
            }
            valid_count += 1;
        }
    }

    // 2B. Exhaustive ternary columns length 5 (243 columns)
    for idx in 0..243usize {
        let mut x = idx;
        let mut l = [0u8; 5];
        for slot in &mut l {
            *slot = b'a' + (x % 3) as u8;
            x /= 3;
        }
        let recs = records_of(&l);
        let p = (idx % 5) as u32;
        let ref_fl = plain_invert_fl(&l, p);
        r5.run_entry(&recs, 5, p, cand_r5_head::Cap::NONE, &mut out_r5)
            .expect("r5");
        assert_eq!(out_r5, ref_fl);
        for cfg in &stress_list {
            rk.run(&recs, 5, p, cfg, &mut out_rk);
            assert_eq!(out_rk, out_r5, "{} mismatch on ternary idx={idx}", cfg.name);
        }
        valid_count += 1;
    }

    // 2C. 4,500 random / periodic / multi-copy dictionary-substituted valid BWTs
    for case in 0..4500usize {
        let text: Vec<u8> = match case % 4 {
            0 => {
                let len = 4 + (rng.next() as usize % 80);
                let sigma = 2 + (rng.next() as usize % 8);
                (0..len).map(|_| b'a' + (rng.next() as usize % sigma) as u8).collect()
            }
            1 => {
                let d = 1 + (rng.next() as usize % 12);
                let q = 2 + (rng.next() as usize % 6);
                let u: Vec<u8> = (0..d).map(|_| b'a' + (rng.next() as usize % 4) as u8).collect();
                u.repeat(q)
            }
            2 => {
                let k = 2 + (rng.next() as usize % 6);
                let wlen = 4 + (rng.next() as usize % 14);
                let mut words = Vec::with_capacity(k);
                for _ in 0..k {
                    words.push((0..wlen).map(|_| b'0' + (rng.next() as usize % 16) as u8).collect::<Vec<u8>>());
                }
                let mut t = Vec::new();
                let reps = 4 + (rng.next() as usize % 20);
                for r in 0..reps {
                    t.extend_from_slice(b"{\"k\":\"");
                    t.extend_from_slice(&words[(rng.next() as usize) % k]);
                    t.push(b'"');
                    t.push(if (r & 1) == 0 { b',' } else { b';' });
                }
                t
            }
            _ => {
                let len = 30 + (rng.next() as usize % 180);
                let mut t = Vec::with_capacity(len);
                let pat = [b"alpha123", b"beta4567", b"gamma890", b"deltaXYZ"];
                while t.len() < len {
                    t.extend_from_slice(pat[(rng.next() as usize) % 4]);
                    if (rng.next() & 3) == 0 {
                        t.push(b'a' + (rng.next() as usize % 26) as u8);
                    }
                }
                t.truncate(len);
                t
            }
        };
        let (l, p) = bwt_small(&text);
        let recs = records_of(&l);
        r5.run_entry(&recs, l.len(), p, cand_r5_head::Cap::NONE, &mut out_r5)
            .expect("r5");
        assert_eq!(out_r5, text);
        for cfg in &stress_list {
            rk.run(&recs, l.len(), p, cfg, &mut out_rk);
            assert_eq!(out_rk, text, "{} failed valid case {case}", cfg.name);
        }
        valid_count += 1;
    }

    // 2D. 250 crafted malformed multi-cycle non-BWT columns (forward-FL parity vs cand_r5_head)
    let mut malformed_l_list: Vec<(Vec<u8>, u32)> = vec![
        (b"bbaabb".to_vec(), 0),
        (b"bbaabb".to_vec(), 3),
        (b"bbbbaaaabbbb".to_vec(), 2),
        (b"aaabbbcccdddeee".to_vec(), 7),
        (b"ababababccddeeffccddeeff".to_vec(), 5),
    ];
    for mcase in 0..245usize {
        let l: Vec<u8> = match mcase % 3 {
            0 => {
                let half = 6 + (rng.next() as usize % 60);
                let mult = 2 + (rng.next() as usize % 5);
                let base: Vec<u8> = (0..half).map(|_| b'a' + (rng.next() as usize % 6) as u8).collect();
                let mut out = Vec::with_capacity(half * mult);
                for &b in &base {
                    for _ in 0..mult {
                        out.push(b);
                    }
                }
                out
            }
            1 => {
                let mut out = Vec::new();
                let runs = 8 + (rng.next() as usize % 40);
                for _ in 0..runs {
                    let c = b'a' + (rng.next() as usize % 10) as u8;
                    let w = match rng.next() % 5 {
                        0 => 1,
                        1..=3 => 2 + (rng.next() as usize % 15),
                        _ => 17 + (rng.next() as usize % 25),
                    };
                    out.resize(out.len() + w, c);
                }
                out
            }
            _ => {
                let len = 8 + (rng.next() as usize % 120);
                let sigma = 2 + (rng.next() as usize % 12);
                (0..len).map(|_| b'0' + (rng.next() as usize % sigma) as u8).collect()
            }
        };
        let p = (rng.next() as usize % l.len()) as u32;
        malformed_l_list.push((l, p));
    }

    for (mi, (l, p)) in malformed_l_list.iter().enumerate() {
        let recs = records_of(l);
        let ref_fl = plain_invert_fl(l, *p);
        r5.run_entry(&recs, l.len(), *p, cand_r5_head::Cap::NONE, &mut out_r5)
            .expect("r5");
        assert_eq!(out_r5, ref_fl, "r5 mismatch on malformed #{mi}");
        for cfg in &stress_list {
            rk.run(&recs, l.len(), *p, cfg, &mut out_rk);
            assert_eq!(out_rk, out_r5, "{} mismatch on malformed #{mi}", cfg.name);
        }
        for cfg in configs {
            rk.run(&recs, l.len(), *p, cfg, &mut out_rk);
            assert_eq!(out_rk, out_r5, "{} mismatch on malformed #{mi}", cfg.name);
        }
        malformed_count += 1;
    }

    println!(
        "HARD EXACTNESS GATE PASS: {} small valid/exhaustive + {} malformed multi-cycle columns (MLP=1 & MLP=8, Uni & Bi, 100% forward-FL parity).",
        valid_count, malformed_count
    );
}

// ============================================================================
// 3. Paired Alternating Multi-Worker Benchmark Harness (1T / 2W / 4W, A/A Control)
// ============================================================================

#[derive(Clone, Copy)]
enum Arm<'a> {
    R5Head,
    R5Nobi,
    Ribbon(&'a RibbonConfig),
}

fn run_arm_workers_once(arm: Arm<'_>, blocks: &Arc<Vec<Block>>, workers: usize) -> f64 {
    if workers == 1 {
        let mut out = Vec::with_capacity(900_032);
        let t0 = Instant::now();
        match arm {
            Arm::R5Head => {
                let mut k = cand_r5_head::Kernel::default();
                for b in blocks.iter() {
                    k.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out)
                        .expect("r5_head");
                }
            }
            Arm::R5Nobi => {
                let mut k = cand_r5_nobi::Kernel::default();
                for b in blocks.iter() {
                    k.run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, &mut out)
                        .expect("r5_nobi");
                }
            }
            Arm::Ribbon(cfg) => {
                let mut k = ribbon_kernel::Kernel::default();
                for b in blocks.iter() {
                    k.run(&b.records, b.n, b.p, cfg, &mut out);
                }
            }
        }
        return t0.elapsed().as_secs_f64() * 1e6;
    }

    let bar = Arc::new(Barrier::new(workers));
    let mut handles = Vec::with_capacity(workers);
    let arm_tag = match arm {
        Arm::R5Head => 0usize,
        Arm::R5Nobi => 1usize,
        Arm::Ribbon(_) => 2usize,
    };
    let cfg_opt = match arm {
        Arm::Ribbon(c) => Some(*c),
        _ => None,
    };
    for _ in 0..workers {
        let b_arc = Arc::clone(blocks);
        let bar_c = Arc::clone(&bar);
        handles.push(thread::spawn(move || {
            let mut out = Vec::with_capacity(900_032);
            match arm_tag {
                0 => {
                    let mut k = cand_r5_head::Kernel::default();
                    bar_c.wait();
                    let t0 = Instant::now();
                    for b in b_arc.iter() {
                        k.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out)
                            .expect("r5");
                    }
                    t0.elapsed().as_secs_f64() * 1e6
                }
                1 => {
                    let mut k = cand_r5_nobi::Kernel::default();
                    bar_c.wait();
                    let t0 = Instant::now();
                    for b in b_arc.iter() {
                        k.run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, &mut out)
                            .expect("nobi");
                    }
                    t0.elapsed().as_secs_f64() * 1e6
                }
                _ => {
                    let cfg = cfg_opt.unwrap();
                    let mut k = ribbon_kernel::Kernel::default();
                    bar_c.wait();
                    let t0 = Instant::now();
                    for b in b_arc.iter() {
                        k.run(&b.records, b.n, b.p, &cfg, &mut out);
                    }
                    t0.elapsed().as_secs_f64() * 1e6
                }
            }
        }));
    }
    let mut max_us = 0.0f64;
    for h in handles {
        let u = h.join().unwrap();
        if u > max_us {
            max_us = u;
        }
    }
    max_us
}

/// Paired alternating benchmark across all arms simultaneously (forward on even rounds,
/// reverse on odd rounds) to cancel VM scheduling drift. Returns median (us, ratio_vs_arm0).
fn bench_paired_arms(
    arms: &[(&str, Arm<'_>)],
    blocks: &Arc<Vec<Block>>,
    workers: usize,
    rounds: usize,
) -> Vec<(f64, f64)> {
    // 1 warmup pass on arm 0
    let _ = run_arm_workers_once(arms[0].1, blocks, workers);

    let m = arms.len();
    let mut raw: Vec<Vec<f64>> = vec![Vec::with_capacity(rounds); m];
    let mut ratios: Vec<Vec<f64>> = vec![Vec::with_capacity(rounds); m];

    for r in 0..rounds {
        let mut round_us = vec![0.0f64; m];
        if (r & 1) == 0 {
            for i in 0..m {
                round_us[i] = run_arm_workers_once(arms[i].1, blocks, workers);
            }
        } else {
            for i in (0..m).rev() {
                round_us[i] = run_arm_workers_once(arms[i].1, blocks, workers);
            }
        }
        let base = round_us[0];
        for i in 0..m {
            raw[i].push(round_us[i]);
            ratios[i].push(round_us[i] / base);
        }
    }

    (0..m)
        .map(|i| {
            raw[i].sort_by(|a, b| a.partial_cmp(b).unwrap());
            ratios[i].sort_by(|a, b| a.partial_cmp(b).unwrap());
            (raw[i][rounds / 2], ratios[i][rounds / 2])
        })
        .collect()
}

fn main() {
    println!("=============================================================================");
    println!("CYCLE 47 P1 — DECISIVE ESCAPE TEST OVER cand_r5_head/ (MLP=1 vs MLP=8)");
    println!("Selective High-Yield Ephemeral Copy-Ribbon Fusion vs Ephemeral Cost Barrier");
    println!("=============================================================================\n");

    let configs: Vec<RibbonConfig> = vec![
        RibbonConfig::NOBI_CTRL,
        // MLP=1 optimal variants (L2-resident, minimal ALU/branch overhead)
        RibbonConfig {
            name: "M1-w4-s7-T64",
            enabled: true,
            use_mlp8: false,
            w_min: 4,
            stride_mask: 7,
            min_steps: 4,
            min_yield: 64,
            l1_dict_cap_bytes: 32_768,
            added_spell_cap_bytes: 32_768,
            use_bi_xor: false,
            wald_sprt: true,
            wald_window: 64,
            wald_min_hits: 2,
            force_bi_small: false,
        },
        RibbonConfig {
            name: "M1-w4-s3-T32",
            enabled: true,
            use_mlp8: false,
            w_min: 4,
            stride_mask: 3,
            min_steps: 4,
            min_yield: 32,
            l1_dict_cap_bytes: 32_768,
            added_spell_cap_bytes: 32_768,
            use_bi_xor: false,
            wald_sprt: true,
            wald_window: 64,
            wald_min_hits: 3,
            force_bi_small: false,
        },
        RibbonConfig {
            name: "M1-w8-s0-T64",
            enabled: true,
            use_mlp8: false,
            w_min: 8,
            stride_mask: 0,
            min_steps: 4,
            min_yield: 64,
            l1_dict_cap_bytes: 32_768,
            added_spell_cap_bytes: 32_768,
            use_bi_xor: false,
            wald_sprt: true,
            wald_window: 48,
            wald_min_hits: 2,
            force_bi_small: false,
        },
        RibbonConfig {
            name: "M1-w4-s0-64K",
            enabled: true,
            use_mlp8: false,
            w_min: 4,
            stride_mask: 0,
            min_steps: 4,
            min_yield: 32,
            l1_dict_cap_bytes: 49_152,
            added_spell_cap_bytes: 65_536,
            use_bi_xor: false,
            wald_sprt: true,
            wald_window: 64,
            wald_min_hits: 3,
            force_bi_small: false,
        },
        RibbonConfig {
            name: "BiM1-w4-s7-T64",
            enabled: true,
            use_mlp8: false,
            w_min: 4,
            stride_mask: 7,
            min_steps: 4,
            min_yield: 64,
            l1_dict_cap_bytes: 32_768,
            added_spell_cap_bytes: 32_768,
            use_bi_xor: true,
            wald_sprt: true,
            wald_window: 64,
            wald_min_hits: 2,
            force_bi_small: false,
        },
        // MLP=8 lockstep interleaved A/B ablation (proves S4.5 is L2 ALU/branch-bound, not DRAM-miss-bound)
        RibbonConfig {
            name: "M8-w4-s7-T64",
            enabled: true,
            use_mlp8: true,
            w_min: 4,
            stride_mask: 7,
            min_steps: 4,
            min_yield: 64,
            l1_dict_cap_bytes: 32_768,
            added_spell_cap_bytes: 32_768,
            use_bi_xor: false,
            wald_sprt: true,
            wald_window: 64,
            wald_min_hits: 2,
            force_bi_small: false,
        },
        RibbonConfig {
            name: "M8-w4-s3-T32",
            enabled: true,
            use_mlp8: true,
            w_min: 4,
            stride_mask: 3,
            min_steps: 4,
            min_yield: 32,
            l1_dict_cap_bytes: 32_768,
            added_spell_cap_bytes: 32_768,
            use_bi_xor: false,
            wald_sprt: true,
            wald_window: 64,
            wald_min_hits: 3,
            force_bi_small: false,
        },
        RibbonConfig {
            name: "BiM8-w4-s7-T64",
            enabled: true,
            use_mlp8: true,
            w_min: 4,
            stride_mask: 7,
            min_steps: 4,
            min_yield: 64,
            l1_dict_cap_bytes: 32_768,
            added_spell_cap_bytes: 32_768,
            use_bi_xor: true,
            wald_sprt: true,
            wald_window: 64,
            wald_min_hits: 2,
            force_bi_small: false,
        },
    ];

    // 1. HARD EXACTNESS GATE (small valid + malformed)
    run_exactness_gate(&configs);

    // 2. Load corpora & verify 100% byte-exactness on ALL real + Analogue E blocks
    let wiki_blocks = Arc::new(load_six_admitted());
    let nonw_blocks = Arc::new(load_non_wikidata());
    let anae_blocks = Arc::new(vec![
        make_analogue_e_block("AnaE-300k-K450", 450, 4, 15, 40, 0xE303),
        make_analogue_e_block("AnaE-900k-K1200", 1200, 4, 16, 40, 0xE404),
    ]);

    {
        let mut r5 = cand_r5_head::Kernel::default();
        let mut nb = cand_r5_nobi::Kernel::default();
        let mut rk = ribbon_kernel::Kernel::default();
        let mut out = Vec::new();
        for group in [&wiki_blocks, &nonw_blocks, &anae_blocks] {
            for b in group.iter() {
                r5.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, b.expected, "R5-Head mismatch on {}", b.name);
                nb.run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, b.expected, "R5-Nobi mismatch on {}", b.name);
                for cfg in &configs {
                    rk.run(&b.records, b.n, b.p, cfg, &mut out);
                    assert_eq!(out, b.expected, "{} mismatch on {}", cfg.name, b.name);
                }
            }
        }
    }
    println!(
        "REAL CORPORA EXACTNESS PASS: 6 Wikidata + 6 Non-Wikidata + 2 Solved Analogue E blocks verified across all {} configs.\n",
        configs.len()
    );

    // 3. Stage-by-stage timing & Cost-Barrier micro-accounting (7-round median per config)
    for (corpus_label, blocks) in [
        ("SOLVED ANALOGUE E (2 blks: 300k + 900k)", &anae_blocks),
        ("6 ADMITTED WIKIDATA BLOCKS", &wiki_blocks),
        ("6 REAL NON-WIKIDATA BLOCKS (5 JSONL + 1 Syslog)", &nonw_blocks),
    ] {
        println!("=============================================================================");
        println!("STAGE-BY-STAGE SWEEP & COST BARRIER — {corpus_label}");
        println!("=============================================================================");

        let (r5_pre, r5_emit, r5_tot, r5_ws, r5_l1) = {
            let mut k = cand_r5_head::Kernel::default();
            let mut out = Vec::new();
            let mut samples = Vec::new();
            for _ in 0..7 {
                let (mut sp, mut se) = (0.0, 0.0);
                for b in blocks.iter() {
                    let (c, w) = k.run_stages(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out);
                    sp += c;
                    se += w;
                }
                samples.push((sp + se, sp, se));
            }
            samples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let (t, p, e) = samples[samples.len() / 2];
            let (ws, l1, _) = k.scratch_report();
            (p, e, t, ws, l1)
        };
        let (nb_pre, nb_emit, nb_tot) = {
            let mut k = cand_r5_nobi::Kernel::default();
            let mut out = Vec::new();
            let mut samples = Vec::new();
            for _ in 0..7 {
                let (mut sp, mut se) = (0.0, 0.0);
                for b in blocks.iter() {
                    let (c, w) = k.run_stages(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, &mut out);
                    sp += c;
                    se += w;
                }
                samples.push((sp + se, sp, se));
            }
            samples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let (t, p, e) = samples[samples.len() / 2];
            (p, e, t)
        };

        println!(
            "Baseline R5-Head: Pre={:7.1}us  Emit={:7.1}us  Total={:7.1}us (1.000x)  WS={:.1}K  L1Dict={:.1}K",
            r5_pre, r5_emit, r5_tot, r5_ws as f64 / 1024.0, r5_l1 as f64 / 1024.0
        );
        println!(
            "Baseline R5-Nobi: Pre={:7.1}us  Emit={:7.1}us  Total={:7.1}us ({:.4}x)\n",
            nb_pre, nb_emit, nb_tot, nb_tot / r5_tot
        );

        println!(
            "{:<15} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} | {:>7} {:>7} | {:>6} {:>5} {:>6} {:>6} {:>6} {:>6}",
            "Config", "S1", "S23", "S4", "S45rb", "S46xr", "S5em",
            "Total", "vsR5H", "Sav%", "K_rb", "ns/prb", "ns/stp", "ns/sav", "WS_KB"
        );

        for cfg in &configs {
            let mut rk = ribbon_kernel::Kernel::default();
            let mut out = Vec::new();
            let mut rounds: Vec<(f64, StageTimesUs, usize, usize, usize, usize, usize)> = Vec::new();
            for _ in 0..7 {
                let mut sum_tm = StageTimesUs::default();
                let (mut m_tot, mut sav_tot, mut k_tot, mut prb_tot, mut stp_tot) = (0, 0, 0, 0, 0);
                for b in blocks.iter() {
                    let (st, tm) = rk.run_timed(&b.records, b.n, b.p, cfg, &mut out);
                    sum_tm.s1_pass += tm.s1_pass;
                    sum_tm.s23_elim += tm.s23_elim;
                    sum_tm.s4_wire += tm.s4_wire;
                    sum_tm.s45_ribbon += tm.s45_ribbon;
                    sum_tm.s46_xor += tm.s46_xor;
                    sum_tm.s5_emit += tm.s5_emit;
                    m_tot += st.retained;
                    sav_tot += st.hops_saved;
                    k_tot += st.ribbons_committed;
                    prb_tot += st.seeds_probed;
                    stp_tot += st.trace_steps_total;
                }
                rounds.push((sum_tm.total(), sum_tm, m_tot, sav_tot, k_tot, prb_tot, stp_tot));
            }
            rounds.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let (tot, tm, m_tot, sav_tot, k_tot, prb_tot, stp_tot) = rounds[rounds.len() / 2];
            let ws_kb = rk.scratch_bytes() as f64 / 1024.0;
            let ns_prb = if prb_tot > 0 { tm.s45_ribbon * 1000.0 / prb_tot as f64 } else { 0.0 };
            let ns_stp = if stp_tot > 0 { tm.s45_ribbon * 1000.0 / stp_tot as f64 } else { 0.0 };
            let ns_sav = if sav_tot > 0 { tm.s45_ribbon * 1000.0 / sav_tot as f64 } else { 0.0 };
            println!(
                "{:<15} {:>6.0} {:>6.0} {:>6.0} {:>6.0} {:>6.0} {:>6.0} | {:>7.0} {:>6.4}x | {:>5.1}% {:>5} {:>6.1} {:>6.1} {:>6.1} {:>6.0}",
                cfg.name,
                tm.s1_pass, tm.s23_elim, tm.s4_wire, tm.s45_ribbon, tm.s46_xor, tm.s5_emit,
                tot, tot / r5_tot,
                100.0 * sav_tot as f64 / m_tot as f64,
                k_tot, ns_prb, ns_stp, ns_sav, ws_kb
            );
        }
        println!();
    }

    // 4. Decisive Paired Alternating 1T / 2W / 4W Benchmarks with A/A Control
    println!("=============================================================================");
    println!("DECISIVE PAIRED ALTERNATING 1T / 2W / 4W BENCHMARKS WITH A/A CONTROL");
    println!("=============================================================================");

    let arms: Vec<(&str, Arm<'_>)> = vec![
        ("R5-Head-A1", Arm::R5Head),
        ("R5-Head-A2(AA)", Arm::R5Head),
        ("R5-Nobi", Arm::R5Nobi),
        (configs[0].name, Arm::Ribbon(&configs[0])), // Ctrl-Nobi
        (configs[1].name, Arm::Ribbon(&configs[1])), // M1-w4-s7-T64
        (configs[3].name, Arm::Ribbon(&configs[3])), // M1-w8-s0-T64
        (configs[4].name, Arm::Ribbon(&configs[4])), // M1-w4-s0-64K
        (configs[5].name, Arm::Ribbon(&configs[5])), // BiM1-w4-s7-T64
        (configs[6].name, Arm::Ribbon(&configs[6])), // M8-w4-s7-T64
    ];

    for (corpus_label, blocks, rounds_1t, rounds_mw) in [
        ("Solved Analogue E (300k+900k)", &anae_blocks, 9usize, 7usize),
        ("6 Admitted Wikidata", &wiki_blocks, 9usize, 7usize),
        ("6 Real Non-Wikidata", &nonw_blocks, 9usize, 7usize),
    ] {
        let res1 = bench_paired_arms(&arms, blocks, 1, rounds_1t);
        let res2 = bench_paired_arms(&arms, blocks, 2, rounds_mw);
        let res4 = bench_paired_arms(&arms, blocks, 4, rounds_mw);

        println!("\nCorpus: {corpus_label}");
        println!(
            "{:<15} | {:>8} {:>8} | {:>8} {:>8} | {:>8} {:>8} | {:>12}",
            "Arm", "1T (us)", "1T/A1", "2W (us)", "2W/A1", "4W (us)", "4W/A1", "< C_Open?"
        );
        for (idx, &(label, _)) in arms.iter().enumerate() {
            let (t1, r1) = res1[idx];
            let (t2, r2) = res2[idx];
            let (t4, r4) = res4[idx];
            let verdict = if idx <= 1 {
                "BASELINE"
            } else if r1 < 0.995 && r2 < 0.995 && r4 < 0.995 {
                "YES (ALL)"
            } else if r1 < 0.995 && r2 < 0.995 {
                "1T+2W"
            } else if r1 < 0.995 {
                "1T ONLY"
            } else {
                "NO"
            };
            println!(
                "{:<15} | {:>8.1} {:>7.4}x | {:>8.1} {:>7.4}x | {:>8.1} {:>7.4}x | {:>12}",
                label, t1, r1, t2, r2, t4, r4, verdict
            );
        }
    }

    println!("\nALL ESCAPE TEST CHECKS & BENCHMARKS COMPLETE.");
}
