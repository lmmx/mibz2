#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod master_nobi;
mod cand_c40_trim;
mod cand_c41_iota8;
mod cand_c41_iota4;
mod cand_c41_onebr;
mod cand_c41_fastjoin;
mod cand_c41_combo;
mod cand_c41_nobi40;
mod cand_c41_nobi41;

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
    let mut kc40 = cand_c40_trim::Kernel::default();
    let mut ki8 = cand_c41_iota8::Kernel::default();
    let mut ki4 = cand_c41_iota4::Kernel::default();
    let mut k1b = cand_c41_onebr::Kernel::default();
    let mut kfj = cand_c41_fastjoin::Kernel::default();
    let mut kcb = cand_c41_combo::Kernel::default();
    let mut knb = master_nobi::Kernel::default();
    let mut kn40 = cand_c41_nobi40::Kernel::default();
    let mut kn41 = cand_c41_nobi41::Kernel::default();

    for b in six.iter().chain([&dense]) {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kc40.run_entry(&b.records, b.n, b.p, cand_c40_trim::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        ki8.run_entry(&b.records, b.n, b.p, cand_c41_iota8::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        ki4.run_entry(&b.records, b.n, b.p, cand_c41_iota4::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        k1b.run_entry(&b.records, b.n, b.p, cand_c41_onebr::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kfj.run_entry(&b.records, b.n, b.p, cand_c41_fastjoin::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kcb.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kn40.run_entry(&b.records, b.n, b.p, cand_c41_nobi40::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kn41.run_entry(&b.records, b.n, b.p, cand_c41_nobi41::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
    }
    println!("PASS: 6 Wikidata + dense-300k 100% byte-exact across all 11 kernels!");

    println!("\n=== 1. 1T INTERLEAVED A/A & A/B + PRE SPLIT (50 rounds/block) ===");
    let names = ["MasterA1", "MasterA2", "Neon-C40", "Iota8", "Iota4", "OneBr", "FastJoin", "C41-Cmb", "Mst-Nobi", "Nobi-C40", "Nobi-C41"];
    let mut sum1t = [0.0f64; 11];
    let mut csum = [0.0f64; 11];
    let mut wsum = [0.0f64; 11];
    for b in &six {
        let mut s: [Vec<f64>; 11] = Default::default();
        for i in 0..11 * 50 {
            let m = (i + i / 11) % 11;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                2 => { kc40.run_entry(&b.records, b.n, b.p, cand_c40_trim::Cap::NONE, &mut out); }
                3 => { ki8.run_entry(&b.records, b.n, b.p, cand_c41_iota8::Cap::NONE, &mut out); }
                4 => { ki4.run_entry(&b.records, b.n, b.p, cand_c41_iota4::Cap::NONE, &mut out); }
                5 => { k1b.run_entry(&b.records, b.n, b.p, cand_c41_onebr::Cap::NONE, &mut out); }
                6 => { kfj.run_entry(&b.records, b.n, b.p, cand_c41_fastjoin::Cap::NONE, &mut out); }
                7 => { kcb.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out); }
                8 => { knb.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                9 => { kn40.run_entry(&b.records, b.n, b.p, cand_c41_nobi40::Cap::NONE, &mut out); }
                _ => { kn41.run_entry(&b.records, b.n, b.p, cand_c41_nobi41::Cap::NONE, &mut out); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let med: [f64; 11] = std::array::from_fn(|k| pct(&mut s[k], 1, 2));
        for k in 0..11 { sum1t[k] += med[k]; }

        let mut sc: [Vec<f64>; 11] = Default::default();
        let mut sw: [Vec<f64>; 11] = Default::default();
        for i in 0..11 * 35 {
            let m = (i + i / 11) % 11;
            let (c, w) = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out),
                1 => km2.run_stages(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out),
                2 => kc40.run_stages(&b.records, b.n, b.p, cand_c40_trim::Cap::NONE, &mut out),
                3 => ki8.run_stages(&b.records, b.n, b.p, cand_c41_iota8::Cap::NONE, &mut out),
                4 => ki4.run_stages(&b.records, b.n, b.p, cand_c41_iota4::Cap::NONE, &mut out),
                5 => k1b.run_stages(&b.records, b.n, b.p, cand_c41_onebr::Cap::NONE, &mut out),
                6 => kfj.run_stages(&b.records, b.n, b.p, cand_c41_fastjoin::Cap::NONE, &mut out),
                7 => kcb.run_stages(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out),
                8 => knb.run_stages(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out),
                9 => kn40.run_stages(&b.records, b.n, b.p, cand_c41_nobi40::Cap::NONE, &mut out),
                _ => kn41.run_stages(&b.records, b.n, b.p, cand_c41_nobi41::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        for k in 0..11 { csum[k] += pct(&mut sc[k], 1, 2); wsum[k] += pct(&mut sw[k], 1, 2); }
        println!("{:14} | M1={:6.1} | A/A={:.3} C40={:.3} I8={:.3} I4={:.3} 1B={:.3} FJ={:.3} C41={:.3} | N40={:.3} N41={:.3}",
            b.name, med[0], med[1]/med[0], med[2]/med[0], med[3]/med[0], med[4]/med[0],
            med[5]/med[0], med[6]/med[0], med[7]/med[0], med[9]/med[8], med[10]/med[8]);
    }
    for k in 0..11 {
        let bt = if k >= 8 { sum1t[8] } else { sum1t[0] };
        let bc = if k >= 8 { csum[8] } else { csum[0] };
        let prev_c = if k >= 8 { csum[9] } else { csum[2] };
        println!("1T_SUM {:10} | tot={:7.1} ({:.4}x) | pre={:6.1} ({:.4}x, {:+.1} vs C40) | emit={:6.1}",
            names[k], sum1t[k], sum1t[k]/bt, csum[k], csum[k]/bc, csum[k]-prev_c, wsum[k]);
    }

    println!("\n=== 2. 2T CONCURRENT BARRIER (35 passes) ===");
    let names2 = ["MasterA1", "MasterA2", "Neon-C40", "Iota8", "Iota4", "C41-Cmb", "Mst-Nobi", "Nobi-C40"];
    let mut r2t: [Vec<f64>; 8] = Default::default();
    for pass in 0..35 {
        for off in 0..8 {
            let m = (pass + off) % 8;
            let bar = Barrier::new(2);
            let t = std::thread::scope(|sc| {
                let b_ref = &six; let bar_ref = &bar;
                let h = sc.spawn(move || {
                    let mut out = Vec::with_capacity(900_064);
                    let mut k0 = master_a1::Kernel::default();
                    let mut k1 = master_a2::Kernel::default();
                    let mut k2 = cand_c40_trim::Kernel::default();
                    let mut k3 = cand_c41_iota8::Kernel::default();
                    let mut k4 = cand_c41_iota4::Kernel::default();
                    let mut k5 = cand_c41_combo::Kernel::default();
                    let mut k6 = master_nobi::Kernel::default();
                    let mut k7 = cand_c41_nobi40::Kernel::default();
                    for b in b_ref {
                        match m {
                            0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { k2.run_entry(&b.records, b.n, b.p, cand_c40_trim::Cap::NONE, &mut out); }
                            3 => { k3.run_entry(&b.records, b.n, b.p, cand_c41_iota8::Cap::NONE, &mut out); }
                            4 => { k4.run_entry(&b.records, b.n, b.p, cand_c41_iota4::Cap::NONE, &mut out); }
                            5 => { k5.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out); }
                            6 => { k6.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                            _ => { k7.run_entry(&b.records, b.n, b.p, cand_c41_nobi40::Cap::NONE, &mut out); }
                        }
                    }
                    bar_ref.wait();
                    let t0 = Instant::now();
                    for b in b_ref.iter().rev() {
                        match m {
                            0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { k2.run_entry(&b.records, b.n, b.p, cand_c40_trim::Cap::NONE, &mut out); }
                            3 => { k3.run_entry(&b.records, b.n, b.p, cand_c41_iota8::Cap::NONE, &mut out); }
                            4 => { k4.run_entry(&b.records, b.n, b.p, cand_c41_iota4::Cap::NONE, &mut out); }
                            5 => { k5.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out); }
                            6 => { k6.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                            _ => { k7.run_entry(&b.records, b.n, b.p, cand_c41_nobi40::Cap::NONE, &mut out); }
                        }
                    }
                    t0.elapsed().as_secs_f64() * 1e6
                });
                let mut out = Vec::with_capacity(900_064);
                let mut k0 = master_a1::Kernel::default();
                let mut k1 = master_a2::Kernel::default();
                let mut k2 = cand_c40_trim::Kernel::default();
                let mut k3 = cand_c41_iota8::Kernel::default();
                let mut k4 = cand_c41_iota4::Kernel::default();
                let mut k5 = cand_c41_combo::Kernel::default();
                let mut k6 = master_nobi::Kernel::default();
                let mut k7 = cand_c41_nobi40::Kernel::default();
                for b in b_ref {
                    match m {
                        0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { k2.run_entry(&b.records, b.n, b.p, cand_c40_trim::Cap::NONE, &mut out); }
                        3 => { k3.run_entry(&b.records, b.n, b.p, cand_c41_iota8::Cap::NONE, &mut out); }
                        4 => { k4.run_entry(&b.records, b.n, b.p, cand_c41_iota4::Cap::NONE, &mut out); }
                        5 => { k5.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out); }
                        6 => { k6.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                        _ => { k7.run_entry(&b.records, b.n, b.p, cand_c41_nobi40::Cap::NONE, &mut out); }
                    }
                }
                bar_ref.wait();
                let t0 = Instant::now();
                for b in b_ref {
                    match m {
                        0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { k2.run_entry(&b.records, b.n, b.p, cand_c40_trim::Cap::NONE, &mut out); }
                        3 => { k3.run_entry(&b.records, b.n, b.p, cand_c41_iota8::Cap::NONE, &mut out); }
                        4 => { k4.run_entry(&b.records, b.n, b.p, cand_c41_iota4::Cap::NONE, &mut out); }
                        5 => { k5.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out); }
                        6 => { k6.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                        _ => { k7.run_entry(&b.records, b.n, b.p, cand_c41_nobi40::Cap::NONE, &mut out); }
                    }
                }
                (t0.elapsed().as_secs_f64() * 1e6).max(h.join().unwrap())
            });
            r2t[m].push(t);
        }
    }
    let p10_0 = pct(&mut r2t[0], 1, 10); let p50_0 = pct(&mut r2t[0], 1, 2);
    let p10_6 = pct(&mut r2t[6], 1, 10); let p50_6 = pct(&mut r2t[6], 1, 2);
    for k in 0..8 {
        let p10 = pct(&mut r2t[k], 1, 10);
        let p50 = pct(&mut r2t[k], 1, 2);
        let (b10, b50) = if k >= 6 { (p10_6, p50_6) } else { (p10_0, p50_0) };
        println!("2T {:10} | p10={:8.1} us ({:.4}x) | p50={:8.1} us ({:.4}x)",
            names2[k], p10, p10/b10, p50, p50/b50);
    }
}
