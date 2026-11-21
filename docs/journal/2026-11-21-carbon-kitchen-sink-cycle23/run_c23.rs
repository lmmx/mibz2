#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_ostrowski;
mod cand_q128;
mod cand_q256;
mod cand_q1024;

use std::sync::Barrier;
use std::time::Instant;

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    expected: Vec<u8>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 { *rec += 1; continue; }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

fn plain_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    let mut rank_in_l = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() { rank_in_l[i] = counts[b as usize]; counts[b as usize] += 1; }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 { starts[c] = tot; tot += counts[c]; }
    let mut out = vec![0u8; n];
    let mut x = p as usize;
    for k in (0..n).rev() {
        let b = l[x];
        out[k] = b;
        x = (starts[b as usize] + rank_in_l[x]) as usize;
    }
    out
}

fn bwt_of(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<u32> = (0..n as u32).collect();
    let mut rank: Vec<u32> = text.iter().map(|&b| u32::from(b)).collect();
    let mut tmp = vec![0u32; n];
    let mut k = 1usize;
    while k < n {
        let r = &rank;
        sa.sort_unstable_by_key(|&i| {
            let j = (i as usize + k) % n;
            (r[i as usize], r[j])
        });
        let mut cur = 0u32;
        tmp[sa[0] as usize] = 0;
        for idx in 1..n {
            let a = sa[idx - 1] as usize;
            let b = sa[idx] as usize;
            if (r[a], r[(a + k) % n]) != (r[b], r[(b + k) % n]) { cur += 1; }
            tmp[b] = cur;
        }
        rank.copy_from_slice(&tmp);
        if cur as usize + 1 == n { break; }
        k <<= 1;
    }
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (row, &s) in sa.iter().enumerate() {
        if s == 0 { p = row as u32; }
        let prev = if s == 0 { n - 1 } else { s as usize - 1 };
        l.push(text[prev]);
    }
    (l, p)
}

fn load_blocks() -> (Vec<Block>, Vec<Block>) {
    let dir = "/home/antigravity/admitted-bwt";
    let man = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let mut specs = vec![];
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") { file = v.trim_end_matches("\",").to_string(); }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") { specs.push((file.clone(), v.trim_end_matches(',').parse::<u32>().unwrap())); }
    }
    let six: Vec<Block> = specs.into_iter().map(|(f, p)| {
        let l = std::fs::read(format!("{dir}/{f}")).unwrap();
        let expected = plain_invert(&l, p);
        let records = records_of(&l);
        Block { name: f, n: l.len(), p, records, expected }
    }).collect();

    let mut dense_src = Vec::with_capacity(300_000);
    let src1 = &six[3].expected;
    let src2 = &six[5].expected;
    let mut off = 0usize;
    while dense_src.len() < 300_000 {
        dense_src.extend_from_slice(&src1[off % (src1.len() - 64)..(off % (src1.len() - 64)) + 48]);
        dense_src.extend_from_slice(&src2[(off * 7) % (src2.len() - 64)..((off * 7) % (src2.len() - 64)) + 48]);
        off += 97;
    }
    dense_src.truncate(300_000);
    let (dl, dp) = bwt_of(&dense_src);
    let drec = records_of(&dl);
    let mut extra = vec![Block {
        name: format!("dense-300k(r/n={:.3})", drec.len() as f64 / dl.len() as f64),
        n: dl.len(), p: dp, records: drec, expected: dense_src,
    }];
    let mut sp_text = Vec::with_capacity(200_000);
    for i in 0..50_000u32 {
        sp_text.extend_from_slice(b"aaab");
        if i % 997 == 0 { sp_text.push(b'c'); }
    }
    let (sl, sp) = bwt_of(&sp_text);
    let srec = records_of(&sl);
    extra.push(Block { name: "spiral-200k".into(), n: sl.len(), p: sp, records: srec, expected: sp_text });
    (six, extra)
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn main() {
    let (six, extra) = load_blocks();
    let mut out = Vec::with_capacity(900_064);

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kost = cand_ostrowski::Kernel::default();
    let mut kq128 = cand_q128::Kernel::default();
    let mut kq256 = cand_q256::Kernel::default();
    let mut kq1024 = cand_q1024::Kernel::default();

    println!("=== 1. OSTROWSKI SPIRAL RULE CENSUS & OUTQ CHUNK SCRATCH AUDIT ===");
    for b in six.iter().chain(extra.iter()) {
        let cm = km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let cost = kost.run_entry(&b.records, b.n, b.p, cand_ostrowski::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kq128.run_entry(&b.records, b.n, b.p, cand_q128::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kq256.run_entry(&b.records, b.n, b.p, cand_q256::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kq1024.run_entry(&b.records, b.n, b.p, cand_q1024::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);

        println!("BLK {:22} | steps={:6} | rules: master={:5} ostrowski(q>=4)={:5} ({:+4})",
            b.name, cm.steps, cm.rules, cost.rules, cost.rules as isize - cm.rules as isize);
    }

    // Re-measure cumulative 6-block scratch ONLY:
    let (mut c_m, mut c_o, mut c_128, mut c_256, mut c_1024) = (
        master_a1::Kernel::default(), cand_ostrowski::Kernel::default(),
        cand_q128::Kernel::default(), cand_q256::Kernel::default(), cand_q1024::Kernel::default()
    );
    for b in &six {
        c_m.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
        c_o.run_entry(&b.records, b.n, b.p, cand_ostrowski::Cap::NONE, &mut out);
        c_128.run_entry(&b.records, b.n, b.p, cand_q128::Cap::NONE, &mut out);
        c_256.run_entry(&b.records, b.n, b.p, cand_q256::Cap::NONE, &mut out);
        c_1024.run_entry(&b.records, b.n, b.p, cand_q1024::Cap::NONE, &mut out);
    }
    println!("\n6-BLOCK CUMULATIVE SCRATCH:");
    println!("  master (Q512) : {:7} B ({:.1} KiB)", c_m.scratch_report().0, c_m.scratch_report().0 as f64 / 1024.0);
    println!("  ostrowski     : {:7} B ({:.1} KiB)", c_o.scratch_report().0, c_o.scratch_report().0 as f64 / 1024.0);
    println!("  q128  (256 B) : {:7} B ({:.1} KiB)", c_128.scratch_report().0, c_128.scratch_report().0 as f64 / 1024.0);
    println!("  q256  (512 B) : {:7} B ({:.1} KiB)", c_256.scratch_report().0, c_256.scratch_report().0 as f64 / 1024.0);
    println!("  q1024 (2 KiB) : {:7} B ({:.1} KiB)", c_1024.scratch_report().0, c_1024.scratch_report().0 as f64 / 1024.0);

    println!("\n=== 2. INTERLEAVED 1T A/A & A/B (55 pairs/block) ===");
    let reps = 55usize;
    let mut sums = [(0.0f64, 0.0f64); 5];
    for b in six.iter().chain(extra.iter()) {
        let mut r_arr = [0.0f64; 5];
        for arm in 0..5usize {
            let (mut ta, mut tb) = (vec![], vec![]);
            for i in 0..2 * reps {
                let t = Instant::now();
                if (i + i / 2) % 2 == 0 {
                    c_m.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                    ta.push(t.elapsed().as_secs_f64() * 1e6);
                } else {
                    match arm {
                        0 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        1 => { c_o.run_entry(&b.records, b.n, b.p, cand_ostrowski::Cap::NONE, &mut out); }
                        2 => { c_128.run_entry(&b.records, b.n, b.p, cand_q128::Cap::NONE, &mut out); }
                        3 => { c_256.run_entry(&b.records, b.n, b.p, cand_q256::Cap::NONE, &mut out); }
                        _ => { c_1024.run_entry(&b.records, b.n, b.p, cand_q1024::Cap::NONE, &mut out); }
                    }
                    tb.push(t.elapsed().as_secs_f64() * 1e6);
                }
            }
            let (r, c) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));
            if b.name.ends_with(".L") { sums[arm].0 += r; sums[arm].1 += c; }
            r_arr[arm] = c / r;
        }
        println!("1T {:22} | A/A={:.3} | Ostrowski={:.3} | Q128={:.3} | Q256={:.3} | Q1024={:.3}",
            b.name, r_arr[0], r_arr[1], r_arr[2], r_arr[3], r_arr[4]);
    }
    println!("1T 6-BLOCK SUM            | A/A={:.4} | Ostrowski={:.4} | Q128={:.4} | Q256={:.4} | Q1024={:.4}",
        sums[0].1 / sums[0].0, sums[1].1 / sums[1].0, sums[2].1 / sums[2].0, sums[3].1 / sums[3].0, sums[4].1 / sums[4].0);

    let workers = 2usize;
    let passes = 35usize;
    for (label, mode) in [("2T A/A (Master1 vs Master2)", 0), ("2T A/B (Master1 vs Ostrowski)", 1), ("2T A/B (Master1 vs Q256)", 2), ("2T A/B (Master1 vs Q1024)", 3)] {
        let barrier = Barrier::new(workers);
        let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default();
                    let mut km2 = master_a2::Kernel::default();
                    let mut ko = cand_ostrowski::Kernel::default();
                    let mut k2 = cand_q256::Kernel::default();
                    let mut k10 = cand_q1024::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        ko.run_entry(&b.records, b.n, b.p, cand_ostrowski::Cap::NONE, &mut out);
                        k2.run_entry(&b.records, b.n, b.p, cand_q256::Cap::NONE, &mut out);
                        k10.run_entry(&b.records, b.n, b.p, cand_q1024::Cap::NONE, &mut out);
                    }
                    let (mut tr, mut tc) = (vec![], vec![]);
                    for pass in 0..(2 * passes + 4) {
                        let which = (pass + pass / 2) % 2;
                        bar.wait();
                        let t0 = Instant::now();
                        for i in 0..six_ref.len() {
                            let b = &six_ref[(i + w * 3) % six_ref.len()];
                            if which == 0 { km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            else if mode == 0 { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            else if mode == 1 { ko.run_entry(&b.records, b.n, b.p, cand_ostrowski::Cap::NONE, &mut out); }
                            else if mode == 2 { k2.run_entry(&b.records, b.n, b.p, cand_q256::Cap::NONE, &mut out); }
                            else { k10.run_entry(&b.records, b.n, b.p, cand_q1024::Cap::NONE, &mut out); }
                        }
                        let dt = t0.elapsed().as_secs_f64() * 1e6;
                        if pass >= 4 { if which == 0 { tr.push(dt); } else { tc.push(dt); } }
                    }
                    (tr, tc)
                })
            }).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut ar: Vec<f64> = res.iter().flat_map(|x| x.0.clone()).collect();
        let mut ac: Vec<f64> = res.iter().flat_map(|x| x.1.clone()).collect();
        println!("{:32} | p10={:.4} ({:.1}/{:.1}) | p50={:.4} ({:.1}/{:.1})",
            label, pct(&mut ac, 1, 10) / pct(&mut ar, 1, 10), pct(&mut ac, 1, 10), pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 2) / pct(&mut ar, 1, 2), pct(&mut ac, 1, 2), pct(&mut ar, 1, 2));
    }
}
