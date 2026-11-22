#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_c37_cap_pt64;
mod cand_c38_inl2;
mod cand_c38_inl23;
mod cand_c38_regelim;
mod cand_c38_i2_reg;
mod cand_c38_i23_reg;

use std::sync::Barrier;
use std::time::Instant;

struct Block { name: String, n: usize, p: u32, records: Vec<u16>, expected: Vec<u8> }

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
    let n = l.len(); let mut counts = [0u32; 256]; let mut rank_in_l = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() { rank_in_l[i] = counts[b as usize]; counts[b as usize] += 1; }
    let mut starts = [0u32; 256]; let mut tot = 0u32;
    for c in 0..256 { starts[c] = tot; tot += counts[c]; }
    let mut out = vec![0u8; n]; let mut x = p as usize;
    for k in (0..n).rev() { let b = l[x]; out[k] = b; x = (starts[b as usize] + rank_in_l[x]) as usize; }
    out
}

fn bwt_small(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len(); let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| { for k in 0..n { let ca = text[(a+k)%n]; let cb = text[(b+k)%n]; if ca != cb { return ca.cmp(&cb); } } a.cmp(&b) });
    let mut l = Vec::with_capacity(n); let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() { if pos == 0 { p = i as u32; } l.push(text[(pos + n - 1) % n]); }
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
    Block { name: "dense-300k".into(), n, p, records: records_of(&l), expected }
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
    let mut kc37 = cand_c37_cap_pt64::Kernel::default();
    let mut ki2 = cand_c38_inl2::Kernel::default();
    let mut ki23 = cand_c38_inl23::Kernel::default();
    let mut kreg = cand_c38_regelim::Kernel::default();
    let mut ki2r = cand_c38_i2_reg::Kernel::default();
    let mut ki23r = cand_c38_i23_reg::Kernel::default();

    println!("=== 1. EXHAUSTIVE PARITY + S2b RULE-REDUCTION CENSUS ===");
    // 2,000 random small BWTs with non-ASCII bytes too (0..255)!
    let mut seed = 0x1234_5678_9ABC_DEF0u64;
    for t in 0..2_000usize {
        let n = 10 + (t % 90);
        let mut text = Vec::with_capacity(n);
        for _ in 0..n {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let b = if (t & 1) == 0 { ((seed >> 33) % 4) as u8 + 125 } else { ((seed >> 33) % 4) as u8 + b'a' };
            text.push(b);
        }
        let (l, p) = bwt_small(&text);
        let recs = records_of(&l);
        ki2.run_entry(&recs, n, p, cand_c38_inl2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, text);
        ki23.run_entry(&recs, n, p, cand_c38_inl23::Cap::NONE, &mut out).unwrap(); assert_eq!(out, text);
        kreg.run_entry(&recs, n, p, cand_c38_regelim::Cap::NONE, &mut out).unwrap(); assert_eq!(out, text);
        ki2r.run_entry(&recs, n, p, cand_c38_i2_reg::Cap::NONE, &mut out).unwrap(); assert_eq!(out, text);
        ki23r.run_entry(&recs, n, p, cand_c38_i23_reg::Cap::NONE, &mut out).unwrap(); assert_eq!(out, text);
    }

    let (mut r_m1, mut r_i2, mut r_i23) = (0usize, 0, 0);
    for b in six.iter().chain([&dense]) {
        let cm = km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kc37.run_entry(&b.records, b.n, b.p, cand_c37_cap_pt64::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        let ci2 = ki2.run_entry(&b.records, b.n, b.p, cand_c38_inl2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        let ci23 = ki23.run_entry(&b.records, b.n, b.p, cand_c38_inl23::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kreg.run_entry(&b.records, b.n, b.p, cand_c38_regelim::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        ki2r.run_entry(&b.records, b.n, b.p, cand_c38_i2_reg::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        ki23r.run_entry(&b.records, b.n, b.p, cand_c38_i23_reg::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        if b.name.starts_with("block") {
            r_m1 += cm.rules; r_i2 += ci2.rules; r_i23 += ci23.rules;
            println!("RULES {:12} | Master={:5} | Inline2={:5} ({:.1}%) | Inline23={:5} ({:.1}%)",
                b.name, cm.rules, ci2.rules, ci2.rules as f64 / cm.rules as f64 * 100.0,
                ci23.rules, ci23.rules as f64 / cm.rules as f64 * 100.0);
        }
    }
    println!("6-BLK RULES SUM    | Master={:6} | Inline2={:6} ({:.1}%) | Inline23={:6} ({:.1}%)",
        r_m1, r_i2, r_i2 as f64 / r_m1 as f64 * 100.0, r_i23, r_i23 as f64 / r_m1 as f64 * 100.0);
    println!("PASS: 2,000 non-ASCII/ASCII small BWTs + 6 Wikidata + dense-300k 100% byte-exact!");

    println!("\n=== 2. 1T INTERLEAVED A/A & A/B + PRE SPLIT (50 rounds/block) ===");
    let names = ["MasterA1", "MasterA2", "Neon-C37", "Inline2", "Inline23", "RegElim", "I2+Reg", "I23+Reg"];
    let mut sum1t = [0.0f64; 8];
    let mut csum = [0.0f64; 8];
    let mut wsum = [0.0f64; 8];
    for b in &six {
        let mut s: [Vec<f64>; 8] = Default::default();
        for i in 0..8 * 50 {
            let m = (i + i / 8) % 8;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                2 => { kc37.run_entry(&b.records, b.n, b.p, cand_c37_cap_pt64::Cap::NONE, &mut out); }
                3 => { ki2.run_entry(&b.records, b.n, b.p, cand_c38_inl2::Cap::NONE, &mut out); }
                4 => { ki23.run_entry(&b.records, b.n, b.p, cand_c38_inl23::Cap::NONE, &mut out); }
                5 => { kreg.run_entry(&b.records, b.n, b.p, cand_c38_regelim::Cap::NONE, &mut out); }
                6 => { ki2r.run_entry(&b.records, b.n, b.p, cand_c38_i2_reg::Cap::NONE, &mut out); }
                _ => { ki23r.run_entry(&b.records, b.n, b.p, cand_c38_i23_reg::Cap::NONE, &mut out); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let med: [f64; 8] = std::array::from_fn(|k| pct(&mut s[k], 1, 2));
        for k in 0..8 { sum1t[k] += med[k]; }

        let mut sc: [Vec<f64>; 8] = Default::default();
        let mut sw: [Vec<f64>; 8] = Default::default();
        for i in 0..8 * 35 {
            let m = (i + i / 8) % 8;
            let (c, w) = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out),
                1 => km2.run_stages(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out),
                2 => kc37.run_stages(&b.records, b.n, b.p, cand_c37_cap_pt64::Cap::NONE, &mut out),
                3 => ki2.run_stages(&b.records, b.n, b.p, cand_c38_inl2::Cap::NONE, &mut out),
                4 => ki23.run_stages(&b.records, b.n, b.p, cand_c38_inl23::Cap::NONE, &mut out),
                5 => kreg.run_stages(&b.records, b.n, b.p, cand_c38_regelim::Cap::NONE, &mut out),
                6 => ki2r.run_stages(&b.records, b.n, b.p, cand_c38_i2_reg::Cap::NONE, &mut out),
                _ => ki23r.run_stages(&b.records, b.n, b.p, cand_c38_i23_reg::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        for k in 0..8 { csum[k] += pct(&mut sc[k], 1, 2); wsum[k] += pct(&mut sw[k], 1, 2); }
        println!("{:14} | M1={:6.1} | A/A={:.3} C37={:.3} I2={:.3} I23={:.3} Reg={:.3} I2R={:.3} I23R={:.3}",
            b.name, med[0], med[1]/med[0], med[2]/med[0], med[3]/med[0], med[4]/med[0],
            med[5]/med[0], med[6]/med[0], med[7]/med[0]);
    }
    for k in 0..8 {
        println!("1T_SUM {:10} | tot={:7.1} ({:.4}x M1) | pre={:6.1} ({:.4}x M1, {:+.1} vs C37) | emit={:6.1}",
            names[k], sum1t[k], sum1t[k]/sum1t[0], csum[k], csum[k]/csum[0], csum[k]-csum[2], wsum[k]);
    }

    println!("\n=== 3. 2T CONCURRENT BARRIER (35 passes) ===");
    let names2 = ["MasterA1", "MasterA2", "Neon-C37", "Inline2", "RegElim", "I2+Reg", "I23+Reg"];
    let mut r2t: [Vec<f64>; 7] = Default::default();
    for pass in 0..35 {
        for off in 0..7 {
            let m = (pass + off) % 7;
            let bar = Barrier::new(2);
            let t = std::thread::scope(|sc| {
                let b_ref = &six; let bar_ref = &bar;
                let h = sc.spawn(move || {
                    let mut out = Vec::with_capacity(900_064);
                    let mut k0 = master_a1::Kernel::default();
                    let mut k1 = master_a2::Kernel::default();
                    let mut k2 = cand_c37_cap_pt64::Kernel::default();
                    let mut k3 = cand_c38_inl2::Kernel::default();
                    let mut k4 = cand_c38_regelim::Kernel::default();
                    let mut k5 = cand_c38_i2_reg::Kernel::default();
                    let mut k6 = cand_c38_i23_reg::Kernel::default();
                    for b in b_ref {
                        match m {
                            0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { k2.run_entry(&b.records, b.n, b.p, cand_c37_cap_pt64::Cap::NONE, &mut out); }
                            3 => { k3.run_entry(&b.records, b.n, b.p, cand_c38_inl2::Cap::NONE, &mut out); }
                            4 => { k4.run_entry(&b.records, b.n, b.p, cand_c38_regelim::Cap::NONE, &mut out); }
                            5 => { k5.run_entry(&b.records, b.n, b.p, cand_c38_i2_reg::Cap::NONE, &mut out); }
                            _ => { k6.run_entry(&b.records, b.n, b.p, cand_c38_i23_reg::Cap::NONE, &mut out); }
                        }
                    }
                    bar_ref.wait();
                    let t0 = Instant::now();
                    for b in b_ref.iter().rev() {
                        match m {
                            0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { k2.run_entry(&b.records, b.n, b.p, cand_c37_cap_pt64::Cap::NONE, &mut out); }
                            3 => { k3.run_entry(&b.records, b.n, b.p, cand_c38_inl2::Cap::NONE, &mut out); }
                            4 => { k4.run_entry(&b.records, b.n, b.p, cand_c38_regelim::Cap::NONE, &mut out); }
                            5 => { k5.run_entry(&b.records, b.n, b.p, cand_c38_i2_reg::Cap::NONE, &mut out); }
                            _ => { k6.run_entry(&b.records, b.n, b.p, cand_c38_i23_reg::Cap::NONE, &mut out); }
                        }
                    }
                    t0.elapsed().as_secs_f64() * 1e6
                });
                let mut out = Vec::with_capacity(900_064);
                let mut k0 = master_a1::Kernel::default();
                let mut k1 = master_a2::Kernel::default();
                let mut k2 = cand_c37_cap_pt64::Kernel::default();
                let mut k3 = cand_c38_inl2::Kernel::default();
                let mut k4 = cand_c38_regelim::Kernel::default();
                let mut k5 = cand_c38_i2_reg::Kernel::default();
                let mut k6 = cand_c38_i23_reg::Kernel::default();
                for b in b_ref {
                    match m {
                        0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { k2.run_entry(&b.records, b.n, b.p, cand_c37_cap_pt64::Cap::NONE, &mut out); }
                        3 => { k3.run_entry(&b.records, b.n, b.p, cand_c38_inl2::Cap::NONE, &mut out); }
                        4 => { k4.run_entry(&b.records, b.n, b.p, cand_c38_regelim::Cap::NONE, &mut out); }
                        5 => { k5.run_entry(&b.records, b.n, b.p, cand_c38_i2_reg::Cap::NONE, &mut out); }
                        _ => { k6.run_entry(&b.records, b.n, b.p, cand_c38_i23_reg::Cap::NONE, &mut out); }
                    }
                }
                bar_ref.wait();
                let t0 = Instant::now();
                for b in b_ref {
                    match m {
                        0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { k2.run_entry(&b.records, b.n, b.p, cand_c37_cap_pt64::Cap::NONE, &mut out); }
                        3 => { k3.run_entry(&b.records, b.n, b.p, cand_c38_inl2::Cap::NONE, &mut out); }
                        4 => { k4.run_entry(&b.records, b.n, b.p, cand_c38_regelim::Cap::NONE, &mut out); }
                        5 => { k5.run_entry(&b.records, b.n, b.p, cand_c38_i2_reg::Cap::NONE, &mut out); }
                        _ => { k6.run_entry(&b.records, b.n, b.p, cand_c38_i23_reg::Cap::NONE, &mut out); }
                    }
                }
                (t0.elapsed().as_secs_f64() * 1e6).max(h.join().unwrap())
            });
            r2t[m].push(t);
        }
    }
    let p10_0 = pct(&mut r2t[0], 1, 10); let p50_0 = pct(&mut r2t[0], 1, 2);
    for k in 0..7 {
        let p10 = pct(&mut r2t[k], 1, 10);
        let p50 = pct(&mut r2t[k], 1, 2);
        println!("2T {:10} | p10={:8.1} us ({:.4}x) | p50={:8.1} us ({:.4}x)",
            names2[k], p10, p10/p10_0, p50, p50/p50_0);
    }
}
