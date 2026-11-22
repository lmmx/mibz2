#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod ab_all_pos;
mod cand_links_only;
mod cand_cmov;
mod cand_cmov_cold;
mod cand_diet_cmov;

use std::sync::Barrier;
use std::time::Instant;

struct Block { name: String, n: usize, p: u32, records: Vec<u16>, expected: Vec<u8> }
fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() { if (*rec >> 8) as u8 == b && (*rec & 255) < 127 { *rec += 1; continue; } }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}
fn plain_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len(); let mut counts = [0u32; 256]; let mut rank_in_l = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() { rank_in_l[i] = counts[b as usize]; counts[b as usize] += 1; }
    let mut starts = [0u32; 256]; let mut tot = 0u32;
    for c in 0..256 { starts[c] = tot; tot += counts[c]; }
    let mut out = vec![0u8; n]; let mut x = p as usize;
    for k in (0..n).rev() { let b = l[x]; out[k] = b; x = (starts[b as usize] + rank_in_l[x]) as usize; }
    out
}
fn bwt_encode(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len(); let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| { for k in 0..n { let ca = text[(a+k)%n]; let cb = text[(b+k)%n]; if ca != cb { return ca.cmp(&cb); } } a.cmp(&b) });
    let mut l = Vec::with_capacity(n); let mut p = 0u32;
    for (i, &s) in sa.iter().enumerate() { if s == 0 { p = i as u32; } l.push(text[(s + n - 1) % n]); }
    (l, p)
}
fn make_dense_300k() -> Block {
    let n = 300_000usize; let mut text = Vec::with_capacity(n); let mut s = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..n { s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); text.push(if i % 9 == 0 { b' ' } else { b'a' + ((s >> 33) % 26) as u8 }); }
    let mut sa: Vec<u32> = (0..n as u32).collect();
    sa.sort_by(|&a, &b| { let (ia, ib) = (a as usize, b as usize); for k in 0..64 { let ca = text[(ia+k)%n]; let cb = text[(ib+k)%n]; if ca != cb { return ca.cmp(&cb); } } ia.cmp(&ib) });
    let mut l = Vec::with_capacity(n); let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() { if pos == 0 { p = i as u32; } l.push(text[(pos as usize + n - 1) % n]); }
    let expected = plain_invert(&l, p); assert_eq!(expected, text);
    let records = records_of(&l);
    Block { name: "dense-300k".into(), n, p, records, expected }
}
fn load_six() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let mut specs = vec![]; let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") { file = v.trim_end_matches("\",").to_string(); }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") { specs.push((file.clone(), v.trim_end_matches(',').parse::<u32>().unwrap())); }
    }
    specs.into_iter().map(|(f, p)| {
        let l = std::fs::read(format!("{dir}/{f}")).unwrap();
        let expected = plain_invert(&l, p);
        let records = records_of(&l);
        Block { name: f, n: l.len(), p, records, expected }
    }).collect()
}
fn pct(v: &mut [f64], num: usize, den: usize) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[(v.len() * num) / den] }

fn main() {
    let six = load_six();
    let dense = make_dense_300k();
    let mut out = Vec::with_capacity(900_064);

    // Exhaustive binary 1..=10 verification
    let mut kc_chk = cand_cmov::Kernel::default();
    let mut kd_chk = cand_diet_cmov::Kernel::default();
    let mut chk = 0usize;
    for len in 1..=10 {
        for mask in 0..(1usize << len) {
            let text: Vec<u8> = (0..len).map(|i| b'a' + ((mask >> i) & 1) as u8).collect();
            let (l, p) = bwt_encode(&text);
            let recs = records_of(&l);
            for &lim in &[1u32, 4, 16] {
                kc_chk.try_run::<true, true>(&recs, len, p, lim, cand_cmov::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                kd_chk.try_run::<true, true>(&recs, len, p, lim, cand_diet_cmov::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                chk += 2;
            }
        }
    }
    println!("Verified {chk} exhaustive BWTs for CMOV-Elim!");

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kbase = ab_all_pos::Kernel::default();
    let mut klnk = cand_links_only::Kernel::default();
    let mut kcmov = cand_cmov::Kernel::default();
    let mut kcold = cand_cmov_cold::Kernel::default();
    let mut kdiet = cand_diet_cmov::Kernel::default();

    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kbase.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        klnk.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kcmov.run_entry(&b.records, b.n, b.p, cand_cmov::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kcold.run_entry(&b.records, b.n, b.p, cand_cmov_cold::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kdiet.run_entry(&b.records, b.n, b.p, cand_diet_cmov::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
    }
    {
        let mut t1 = cand_cmov::Kernel::default();
        let mut t2 = cand_diet_cmov::Kernel::default();
        t1.run_entry(&dense.records, dense.n, dense.p, cand_cmov::Cap::NONE, &mut out).unwrap(); assert_eq!(out, dense.expected);
        t2.run_entry(&dense.records, dense.n, dense.p, cand_diet_cmov::Cap::NONE, &mut out).unwrap(); assert_eq!(out, dense.expected);
    }
    let (sm, _, _) = km1.scratch_report();
    let (sc, _, _) = kcmov.scratch_report();
    let (sd, _, _) = kdiet.scratch_report();
    println!("SCRATCH Master={:.1} KiB | CMOV-Bi={:.1} KiB ({:+.2}%) | CMOV-Diet={:.1} KiB ({:+.2}%)",
        sm as f64 / 1024.0, sc as f64 / 1024.0, (sc as f64 / sm as f64 - 1.0) * 100.0,
        sd as f64 / 1024.0, (sd as f64 / sm as f64 - 1.0) * 100.0);

    println!("\n=== 1. CONSTRUCT STAGE ATTRIBUTION (60 rounds/block, delta vs Neon-C29) ===");
    let mut csum = [0.0f64; 6];
    for b in &six {
        let mut s = [vec![], vec![], vec![], vec![], vec![], vec![]];
        for i in 0..6 * 60 {
            let m = (i + i / 6) % 6;
            let c = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).0,
                1 => kbase.run_stages(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out).0,
                2 => klnk.run_stages(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out).0,
                3 => kcmov.run_stages(&b.records, b.n, b.p, cand_cmov::Cap::NONE, &mut out).0,
                4 => kcold.run_stages(&b.records, b.n, b.p, cand_cmov_cold::Cap::NONE, &mut out).0,
                _ => kdiet.run_stages(&b.records, b.n, b.p, cand_diet_cmov::Cap::NONE, &mut out).0,
            };
            s[m].push(c);
        }
        let p10: [f64; 6] = std::array::from_fn(|j| pct(&mut s[j], 1, 10));
        for j in 0..6 { csum[j] += p10[j]; }
        println!(
            "CONST {:12} | M={:6.1} C29={:6.1} | Links={:6.1} ({:+.1}) | CMOV={:6.1} ({:+.1}) | CMOV2={:6.1} ({:+.1}) | DietCMOV={:6.1} ({:.3}x M)",
            b.name, p10[0], p10[1], p10[2], p10[2]-p10[1], p10[3], p10[3]-p10[1], p10[4], p10[4]-p10[1], p10[5], p10[5]/p10[0]
        );
    }
    println!(
        "CONST 6-BLK SUM    | M={:6.1} C29={:6.1} ({:.3}x) | Links={:6.1} ({:+.1}) | CMOV={:6.1} ({:.3}x, {:+.1}) | CMOV2={:6.1} ({:+.1}) | DietCMOV={:6.1} ({:.4}x M)",
        csum[0], csum[1], csum[1]/csum[0],
        csum[2], csum[2]-csum[1],
        csum[3], csum[3]/csum[0], csum[3]-csum[1],
        csum[4], csum[4]-csum[1],
        csum[5], csum[5]/csum[0]
    );

    println!("\n=== 2. 1T INTERLEAVED A/A & A/B (70 pairs/block + dense-300k) ===");
    let reps = 70usize;
    let mut sr = [0.0f64; 6]; let mut sc2 = [0.0f64; 6];
    for b in six.iter().chain(std::iter::once(&dense)) {
        let is_six = b.n >= 890_000;
        let mut r = [0.0; 6]; let mut c = [0.0; 6];
        for mode in 0..6 {
            let (mut ta, mut tb) = (vec![], vec![]);
            for i in 0..2 * reps {
                let t = Instant::now();
                if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
                else {
                    match mode {
                        0 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        1 => { kbase.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out); }
                        2 => { klnk.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                        3 => { kcmov.run_entry(&b.records, b.n, b.p, cand_cmov::Cap::NONE, &mut out); }
                        4 => { kcold.run_entry(&b.records, b.n, b.p, cand_cmov_cold::Cap::NONE, &mut out); }
                        _ => { kdiet.run_entry(&b.records, b.n, b.p, cand_diet_cmov::Cap::NONE, &mut out); }
                    }
                    tb.push(t.elapsed().as_secs_f64() * 1e6);
                }
            }
            r[mode] = pct(&mut ta, 1, 10); c[mode] = pct(&mut tb, 1, 10);
            if is_six { sr[mode] += r[mode]; sc2[mode] += c[mode]; }
        }
        println!(
            "1T {:12} | A/A={:.3} | C29={:.3} | Links={:.3} | CMOV-Bi={:.3} | CMOV2={:.3} | Diet-CMOV={:.3}",
            b.name, c[0]/r[0], c[1]/r[1], c[2]/r[2], c[3]/r[3], c[4]/r[4], c[5]/r[5]
        );
    }
    println!(
        "1T 6-BLK SUM  | A/A={:.4} | C29={:.4} | Links={:.4} | CMOV-Bi={:.4} | CMOV2={:.4} | Diet-CMOV={:.4}",
        sc2[0]/sr[0], sc2[1]/sr[1], sc2[2]/sr[2], sc2[3]/sr[3], sc2[4]/sr[4], sc2[5]/sr[5]
    );

    println!("\n=== 3. 2T CONCURRENT BARRIER A/A & A/B (45 passes/worker) ===");
    let workers = 2usize; let passes = 45usize;
    for (label, mode) in [
        ("2T A/A (M1 vs M2)", 0),
        ("2T A/B (M1 vs Neon-C29)", 1),
        ("2T A/B (M1 vs Links-Only)", 2),
        ("2T A/B (M1 vs CMOV-Bi)", 3),
        ("2T A/B (M1 vs Diet-CMOV)", 4),
    ] {
        let barrier = Barrier::new(workers); let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default(); let mut km2 = master_a2::Kernel::default();
                    let mut kb = ab_all_pos::Kernel::default(); let mut kl = cand_links_only::Kernel::default();
                    let mut kc = cand_cmov::Kernel::default(); let mut kd = cand_diet_cmov::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        kb.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out);
                        kl.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out);
                        kc.run_entry(&b.records, b.n, b.p, cand_cmov::Cap::NONE, &mut out);
                        kd.run_entry(&b.records, b.n, b.p, cand_diet_cmov::Cap::NONE, &mut out);
                    }
                    let (mut tr, mut tc) = (vec![], vec![]);
                    for pass in 0..(2*passes + 4) {
                        let which = (pass + pass/2) % 2;
                        bar.wait(); let t0 = Instant::now();
                        for i in 0..six_ref.len() {
                            let b = &six_ref[(i + w*3) % six_ref.len()];
                            if which == 0 { km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            else if mode == 0 { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            else if mode == 1 { kb.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out); }
                            else if mode == 2 { kl.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                            else if mode == 3 { kc.run_entry(&b.records, b.n, b.p, cand_cmov::Cap::NONE, &mut out); }
                            else { kd.run_entry(&b.records, b.n, b.p, cand_diet_cmov::Cap::NONE, &mut out); }
                        }
                        let dt = t0.elapsed().as_secs_f64()*1e6;
                        if pass >= 4 { if which == 0 { tr.push(dt); } else { tc.push(dt); } }
                    }
                    (tr, tc)
                })
            }).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut ar: Vec<f64> = res.iter().flat_map(|x| x.0.clone()).collect();
        let mut ac: Vec<f64> = res.iter().flat_map(|x| x.1.clone()).collect();
        println!("{:28} | p10={:.4} ({:.1}/{:.1}) | p50={:.4} ({:.1}/{:.1})",
            label, pct(&mut ac, 1, 10)/pct(&mut ar, 1, 10), pct(&mut ac, 1, 10), pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 2)/pct(&mut ar, 1, 2), pct(&mut ac, 1, 2), pct(&mut ar, 1, 2));
    }
}
