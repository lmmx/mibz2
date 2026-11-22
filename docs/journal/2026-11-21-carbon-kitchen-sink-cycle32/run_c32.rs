#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_links_only;
mod cand_l14;
mod cand_l20;
mod cand_l24;
mod cand_l32;

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

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kl14 = cand_l14::Kernel::default();
    let mut kl16 = cand_links_only::Kernel::default();
    let mut kl20 = cand_l20::Kernel::default();
    let mut kl24 = cand_l24::Kernel::default();
    let mut kl32 = cand_l32::Kernel::default();

    println!("=== 1. PARITY, STRUCTURAL COUNTS (m, thick) & CUMULATIVE 6-BLK SCRATCH ===");
    let (mut tm14, mut tm16, mut tm20, mut tm24, mut tm32) = (0usize, 0, 0, 0, 0);
    let (mut ts14, mut ts16, mut ts20, mut ts24, mut ts32) = (0usize, 0, 0, 0, 0);
    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        let c14 = kl14.run_entry(&b.records, b.n, b.p, cand_l14::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        let c16 = kl16.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        let c20 = kl20.run_entry(&b.records, b.n, b.p, cand_l20::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        let c24 = kl24.run_entry(&b.records, b.n, b.p, cand_l24::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        let c32 = kl32.run_entry(&b.records, b.n, b.p, cand_l32::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        tm14 += c14.retained; tm16 += c16.retained; tm20 += c20.retained; tm24 += c24.retained; tm32 += c32.retained;
        ts14 += c14.thick; ts16 += c16.thick; ts20 += c20.thick; ts24 += c24.thick; ts32 += c32.thick;
    }
    let sm = km1.scratch_report().0;
    for (lbl, sc, m, s) in [
        ("LIMIT=14", kl14.scratch_report().0, tm14, ts14),
        ("LIMIT=16 (Neon-C31)", kl16.scratch_report().0, tm16, ts16),
        ("LIMIT=20", kl20.scratch_report().0, tm20, ts20),
        ("LIMIT=24", kl24.scratch_report().0, tm24, ts24),
        ("LIMIT=32", kl32.scratch_report().0, tm32, ts32),
    ] {
        println!(
            "PARETO {:20} | scratch={:6.1} KiB ({:+.2}% vs Master) | sum_m={:7} ({:.3}x L16) | sum_thick={:5} ({:.3}x L16)",
            lbl, sc as f64 / 1024.0, (sc as f64 / sm as f64 - 1.0) * 100.0,
            m, m as f64 / tm16 as f64, s, s as f64 / ts16 as f64
        );
    }

    println!("\n=== 2. STAGE ATTRIBUTION (`construct` + `walk`, 50 rounds/block) ===");
    let mut csum = [0.0f64; 6]; let mut wsum = [0.0f64; 6];
    for b in &six {
        let mut sc = [vec![], vec![], vec![], vec![], vec![], vec![]];
        let mut sw = [vec![], vec![], vec![], vec![], vec![], vec![]];
        for i in 0..6 * 50 {
            let m = (i + i / 6) % 6;
            let (c, w) = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out),
                1 => kl14.run_stages(&b.records, b.n, b.p, cand_l14::Cap::NONE, &mut out),
                2 => kl16.run_stages(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out),
                3 => kl20.run_stages(&b.records, b.n, b.p, cand_l20::Cap::NONE, &mut out),
                4 => kl24.run_stages(&b.records, b.n, b.p, cand_l24::Cap::NONE, &mut out),
                _ => kl32.run_stages(&b.records, b.n, b.p, cand_l32::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        for j in 0..6 { csum[j] += pct(&mut sc[j], 1, 10); wsum[j] += pct(&mut sw[j], 1, 10); }
    }
    for (j, lbl) in [(0, "Master@16"), (1, "Neon@14"), (2, "Neon@16"), (3, "Neon@20"), (4, "Neon@24"), (5, "Neon@32")] {
        println!(
            "STAGE {:10} | construct={:6.1} ({:.3}x M) | walk={:6.1} ({:.3}x M) | sum={:6.1} ({:.4}x M)",
            lbl, csum[j], csum[j]/csum[0], wsum[j], wsum[j]/wsum[0], csum[j]+wsum[j], (csum[j]+wsum[j])/(csum[0]+wsum[0])
        );
    }

    println!("\n=== 3. 1T INTERLEAVED A/A & A/B (65 pairs/block + dense-300k) ===");
    let reps = 65usize;
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
                        1 => { kl14.run_entry(&b.records, b.n, b.p, cand_l14::Cap::NONE, &mut out); }
                        2 => { kl16.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                        3 => { kl20.run_entry(&b.records, b.n, b.p, cand_l20::Cap::NONE, &mut out); }
                        4 => { kl24.run_entry(&b.records, b.n, b.p, cand_l24::Cap::NONE, &mut out); }
                        _ => { kl32.run_entry(&b.records, b.n, b.p, cand_l32::Cap::NONE, &mut out); }
                    }
                    tb.push(t.elapsed().as_secs_f64() * 1e6);
                }
            }
            r[mode] = pct(&mut ta, 1, 10); c[mode] = pct(&mut tb, 1, 10);
            if is_six { sr[mode] += r[mode]; sc2[mode] += c[mode]; }
        }
        println!(
            "1T {:12} | A/A={:.3} | L14={:.3} | L16={:.3} | L20={:.3} | L24={:.3} | L32={:.3}",
            b.name, c[0]/r[0], c[1]/r[1], c[2]/r[2], c[3]/r[3], c[4]/r[4], c[5]/r[5]
        );
    }
    println!(
        "1T 6-BLK SUM  | A/A={:.4} | L14={:.4} | L16={:.4} | L20={:.4} | L24={:.4} | L32={:.4}",
        sc2[0]/sr[0], sc2[1]/sr[1], sc2[2]/sr[2], sc2[3]/sr[3], sc2[4]/sr[4], sc2[5]/sr[5]
    );

    println!("\n=== 4. 2T CONCURRENT BARRIER A/A & A/B (45 passes/worker) ===");
    let workers = 2usize; let passes = 45usize;
    for (label, mode) in [
        ("2T A/A (M1 vs M2)", 0),
        ("2T A/B (M1 vs Neon@14)", 1),
        ("2T A/B (M1 vs Neon@16)", 2),
        ("2T A/B (M1 vs Neon@20)", 3),
        ("2T A/B (M1 vs Neon@24)", 4),
        ("2T A/B (M1 vs Neon@32)", 5),
    ] {
        let barrier = Barrier::new(workers); let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default(); let mut km2 = master_a2::Kernel::default();
                    let mut k14 = cand_l14::Kernel::default(); let mut k16 = cand_links_only::Kernel::default();
                    let mut k20 = cand_l20::Kernel::default(); let mut k24 = cand_l24::Kernel::default();
                    let mut k32 = cand_l32::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        k14.run_entry(&b.records, b.n, b.p, cand_l14::Cap::NONE, &mut out);
                        k16.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out);
                        k20.run_entry(&b.records, b.n, b.p, cand_l20::Cap::NONE, &mut out);
                        k24.run_entry(&b.records, b.n, b.p, cand_l24::Cap::NONE, &mut out);
                        k32.run_entry(&b.records, b.n, b.p, cand_l32::Cap::NONE, &mut out);
                    }
                    let (mut tr, mut tc) = (vec![], vec![]);
                    for pass in 0..(2*passes + 4) {
                        let which = (pass + pass/2) % 2;
                        bar.wait(); let t0 = Instant::now();
                        for i in 0..six_ref.len() {
                            let b = &six_ref[(i + w*3) % six_ref.len()];
                            if which == 0 { km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            else if mode == 0 { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            else if mode == 1 { k14.run_entry(&b.records, b.n, b.p, cand_l14::Cap::NONE, &mut out); }
                            else if mode == 2 { k16.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                            else if mode == 3 { k20.run_entry(&b.records, b.n, b.p, cand_l20::Cap::NONE, &mut out); }
                            else if mode == 4 { k24.run_entry(&b.records, b.n, b.p, cand_l24::Cap::NONE, &mut out); }
                            else { k32.run_entry(&b.records, b.n, b.p, cand_l32::Cap::NONE, &mut out); }
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
        println!("{:26} | p10={:.4} ({:.1}/{:.1}) | p50={:.4} ({:.1}/{:.1})",
            label, pct(&mut ac, 1, 10)/pct(&mut ar, 1, 10), pct(&mut ac, 1, 10), pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 2)/pct(&mut ar, 1, 2), pct(&mut ac, 1, 2), pct(&mut ar, 1, 2));
    }
}
