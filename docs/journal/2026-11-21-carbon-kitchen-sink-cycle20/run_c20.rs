#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_mn16_direct;
mod cand_operad;

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

fn exhaustive_operad_verify() {
    let mut kop = cand_operad::Kernel::default();
    let mut out = Vec::new();
    let mut checks = 0usize;
    for len in 1..=11usize {
        for bits in 0..(1u32 << len) {
            let text: Vec<u8> = (0..len).map(|i| ((bits >> i) & 1) as u8).collect();
            let (l, p) = bwt_of(&text);
            let recs = records_of(&l);
            for lim in [1u32, 2, 4, 16] {
                kop.try_run::<true, true>(&recs, len, p, lim, cand_operad::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text, "Operad mismatch len={len} lim={lim}");
                checks += 1;
            }
        }
    }
    println!("EXHAUSTIVE OPERAD VERIFY OK: {checks} binary BWT checks (incl. >15B rules at lim=1)!");
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
    exhaustive_operad_verify();
    let (six, extra) = load_blocks();
    let mut out = Vec::with_capacity(900_064);

    println!("\n=== 1. FRESH & CUMULATIVE SCRATCH + PARITY AUDIT (Operad-MN16 vs MN16-Direct vs Master 0.5.11) ===");
    for b in six.iter().chain(extra.iter()) {
        let mut fm = master_a1::Kernel::default();
        let mut fmn = cand_mn16_direct::Kernel::default();
        let mut fop = cand_operad::Kernel::default();
        let cm = fm.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let cmn = fmn.run_entry(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let cop = fop.run_entry(&b.records, b.n, b.p, cand_operad::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "Operad fresh mismatch on {}", b.name);
        let (sm, _, _) = fm.scratch_report();
        let (smn, _, _) = fmn.scratch_report();
        let (sop, dop, _) = fop.scratch_report();
        println!("BLK {:22} | words: {:4} -> {:4} | dict_L1={:4.1} KiB | fresh_KiB: master={:6.1} mn16={:6.1} operad={:6.1}",
            b.name, cm.words, cop.words, dop as f64 / 1024.0, sm as f64 / 1024.0, smn as f64 / 1024.0, sop as f64 / 1024.0);
        let _ = cmn;
    }

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kmn = cand_mn16_direct::Kernel::default();
    let mut kop = cand_operad::Kernel::default();
    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
        kmn.run_entry(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out);
        kop.run_entry(&b.records, b.n, b.p, cand_operad::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
    }
    let (sm, _, pm) = km1.scratch_report();
    let (smn, _, pmn) = kmn.scratch_report();
    let (sop, _, pop) = kop.scratch_report();
    println!("\nCUMULATIVE 6-BLOCK SCRATCH:");
    println!("  master_0.5.11: {:7} B ({:6.1} KiB) | {}", sm, sm as f64 / 1024.0, pm);
    println!("  mn16_direct  : {:7} B ({:6.1} KiB) | {}", smn, smn as f64 / 1024.0, pmn);
    println!("  operad_mn16  : {:7} B ({:6.1} KiB) | {}", sop, sop as f64 / 1024.0, pop);

    println!("\n=== 2. STAGE ATTRIBUTION (Operad-MN16 vs Master 0.5.11, 45 pairs/block) ===");
    let reps = 45usize;
    let (mut scm, mut swm, mut sco, mut swo) = (0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain(extra.iter()) {
        let (mut mc, mut mw, mut oc, mut ow) = (vec![], vec![], vec![], vec![]);
        for i in 0..2 * reps {
            if (i + i / 2) % 2 == 0 {
                let (c, w) = km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                mc.push(c); mw.push(w);
            } else {
                let (c, w) = kop.run_stages(&b.records, b.n, b.p, cand_operad::Cap::NONE, &mut out);
                oc.push(c); ow.push(w);
            }
        }
        let (mc10, mw10, oc10, ow10) = (pct(&mut mc, 1, 10), pct(&mut mw, 1, 10), pct(&mut oc, 1, 10), pct(&mut ow, 1, 10));
        if b.name.ends_with(".L") { scm += mc10; swm += mw10; sco += oc10; swo += ow10; }
        println!("STAGE {:22} | construct: {:6.1} -> {:6.1} ({:.3}x, {:+.1} us) | walk/emit: {:6.1} -> {:6.1} ({:.3}x, {:+.1} us)",
            b.name, mc10, oc10, oc10 / mc10, oc10 - mc10, mw10, ow10, ow10 / mw10, ow10 - mw10);
    }
    println!("STAGE 6-BLOCK SUM            | construct: {:6.1} -> {:6.1} ({:.4}x, {:+.1} us) | walk/emit: {:6.1} -> {:6.1} ({:.4}x, {:+.1} us)",
        scm, sco, sco / scm, sco - scm, swm, swo, swo / swm, swo - swm);

    println!("\n=== 3. END-TO-END 1T & 2T A/A vs A/B (60 pairs/block) ===");
    let reps = 60usize;
    let (mut saar, mut saac, mut smnr, mut smnc, mut sopr, mut sopc) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain(extra.iter()) {
        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (aar, aac) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kmn.run_entry(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (mnr, mnc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kop.run_entry(&b.records, b.n, b.p, cand_operad::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (opr, opc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        if b.name.ends_with(".L") { saar += aar; saac += aac; smnr += mnr; smnc += mnc; sopr += opr; sopc += opc; }
        println!("1T {:22} | A/A={:.3} | MN16-Direct={:.3} | Operad-MN16={:.3} ({:.1}/{:.1} us)",
            b.name, aac / aar, mnc / mnr, opc / opr, opc, opr);
    }
    println!("1T 6-BLOCK TOTAL          | A/A={:.4} | MN16-Direct={:.4} | Operad-MN16={:.4}",
        saac / saar, smnc / smnr, sopc / sopr);

    let workers = 2usize;
    let passes = 40usize;
    for (label, mode) in [("2T A/A (Master1 vs Master2)", 0), ("2T A/B (Master1 vs MN16-Direct)", 1), ("2T A/B (Master1 vs Operad-MN16)", 2)] {
        let barrier = Barrier::new(workers);
        let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default();
                    let mut km2 = master_a2::Kernel::default();
                    let mut kmn = cand_mn16_direct::Kernel::default();
                    let mut kop = cand_operad::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        kmn.run_entry(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out);
                        kop.run_entry(&b.records, b.n, b.p, cand_operad::Cap::NONE, &mut out);
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
                            else if mode == 1 { kmn.run_entry(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out); }
                            else { kop.run_entry(&b.records, b.n, b.p, cand_operad::Cap::NONE, &mut out); }
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
