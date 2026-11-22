#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod master_nobi;
mod cand_c41_combo;
mod cand_c41_nobi41;
mod cand_c42_pair2;
mod cand_c42_s3arr;
mod cand_c42_bandw;
mod cand_c42_combo;
mod cand_c42_nobi;

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
    let mut kc41 = cand_c41_combo::Kernel::default();
    let mut kp2 = cand_c42_pair2::Kernel::default();
    let mut ks3 = cand_c42_s3arr::Kernel::default();
    let mut kbw = cand_c42_bandw::Kernel::default();
    let mut kcb = cand_c42_combo::Kernel::default();
    let mut knb = master_nobi::Kernel::default();
    let mut kn41 = cand_c41_nobi41::Kernel::default();
    let mut kn42 = cand_c42_nobi::Kernel::default();

    for b in six.iter().chain([&dense]) {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kp2.run_entry(&b.records, b.n, b.p, cand_c42_pair2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        ks3.run_entry(&b.records, b.n, b.p, cand_c42_s3arr::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kbw.run_entry(&b.records, b.n, b.p, cand_c42_bandw::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kcb.run_entry(&b.records, b.n, b.p, cand_c42_combo::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kn42.run_entry(&b.records, b.n, b.p, cand_c42_nobi::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
    }
    assert!(kcb.run_entry(&dense.records, dense.n, dense.p, cand_c42_combo::Cap::dense(dense.n), &mut out).is_none());
    println!("PASS: 6 Wikidata + dense-300k + #703 Cap::dense refusal 100% byte-exact!");

    println!("\n=== 1. 1T INTERLEAVED A/A & A/B + PRE SPLIT (50 rounds/block) ===");
    let names = ["MasterA1", "MasterA2", "Neon-C41", "Pair2", "S3Arr", "BandW", "C42-Cmb", "Mst-Nobi", "Nobi-C41", "Nobi-C42"];
    let mut sum1t = [0.0f64; 10];
    let mut csum = [0.0f64; 10];
    let mut wsum = [0.0f64; 10];
    for b in &six {
        let mut s: [Vec<f64>; 10] = Default::default();
        for i in 0..10 * 50 {
            let m = (i + i / 10) % 10;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                2 => { kc41.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out); }
                3 => { kp2.run_entry(&b.records, b.n, b.p, cand_c42_pair2::Cap::NONE, &mut out); }
                4 => { ks3.run_entry(&b.records, b.n, b.p, cand_c42_s3arr::Cap::NONE, &mut out); }
                5 => { kbw.run_entry(&b.records, b.n, b.p, cand_c42_bandw::Cap::NONE, &mut out); }
                6 => { kcb.run_entry(&b.records, b.n, b.p, cand_c42_combo::Cap::NONE, &mut out); }
                7 => { knb.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                8 => { kn41.run_entry(&b.records, b.n, b.p, cand_c41_nobi41::Cap::NONE, &mut out); }
                _ => { kn42.run_entry(&b.records, b.n, b.p, cand_c42_nobi::Cap::NONE, &mut out); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let med: [f64; 10] = std::array::from_fn(|k| pct(&mut s[k], 1, 2));
        for k in 0..10 { sum1t[k] += med[k]; }

        let mut sc: [Vec<f64>; 10] = Default::default();
        let mut sw: [Vec<f64>; 10] = Default::default();
        for i in 0..10 * 35 {
            let m = (i + i / 10) % 10;
            let (c, w) = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out),
                1 => km2.run_stages(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out),
                2 => kc41.run_stages(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out),
                3 => kp2.run_stages(&b.records, b.n, b.p, cand_c42_pair2::Cap::NONE, &mut out),
                4 => ks3.run_stages(&b.records, b.n, b.p, cand_c42_s3arr::Cap::NONE, &mut out),
                5 => kbw.run_stages(&b.records, b.n, b.p, cand_c42_bandw::Cap::NONE, &mut out),
                6 => kcb.run_stages(&b.records, b.n, b.p, cand_c42_combo::Cap::NONE, &mut out),
                7 => knb.run_stages(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out),
                8 => kn41.run_stages(&b.records, b.n, b.p, cand_c41_nobi41::Cap::NONE, &mut out),
                _ => kn42.run_stages(&b.records, b.n, b.p, cand_c42_nobi::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        for k in 0..10 { csum[k] += pct(&mut sc[k], 1, 2); wsum[k] += pct(&mut sw[k], 1, 2); }
        println!("{:14} | M1={:6.1} | A/A={:.3} C41={:.3} P2={:.3} S3A={:.3} BW={:.3} C42={:.3} | N41={:.3} N42={:.3}",
            b.name, med[0], med[1]/med[0], med[2]/med[0], med[3]/med[0], med[4]/med[0],
            med[5]/med[0], med[6]/med[0], med[8]/med[7], med[9]/med[7]);
    }
    for k in 0..10 {
        let bt = if k >= 7 { sum1t[7] } else { sum1t[0] };
        let bc = if k >= 7 { csum[7] } else { csum[0] };
        let prev_c = if k >= 7 { csum[8] } else { csum[2] };
        println!("1T_SUM {:10} | tot={:7.1} ({:.4}x) | pre={:6.1} ({:.4}x, {:+.1} vs C41) | emit={:6.1}",
            names[k], sum1t[k], sum1t[k]/bt, csum[k], csum[k]/bc, csum[k]-prev_c, wsum[k]);
    }

    println!("\n=== 2. 2T CONCURRENT BARRIER (35 passes) ===");
    let names2 = ["MasterA1", "MasterA2", "Neon-C41", "Pair2", "S3Arr", "C42-Cmb", "Mst-Nobi", "Nobi-C42"];
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
                    let mut k2 = cand_c41_combo::Kernel::default();
                    let mut k3 = cand_c42_pair2::Kernel::default();
                    let mut k4 = cand_c42_s3arr::Kernel::default();
                    let mut k5 = cand_c42_combo::Kernel::default();
                    let mut k6 = master_nobi::Kernel::default();
                    let mut k7 = cand_c42_nobi::Kernel::default();
                    for b in b_ref {
                        match m {
                            0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { k2.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out); }
                            3 => { k3.run_entry(&b.records, b.n, b.p, cand_c42_pair2::Cap::NONE, &mut out); }
                            4 => { k4.run_entry(&b.records, b.n, b.p, cand_c42_s3arr::Cap::NONE, &mut out); }
                            5 => { k5.run_entry(&b.records, b.n, b.p, cand_c42_combo::Cap::NONE, &mut out); }
                            6 => { k6.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                            _ => { k7.run_entry(&b.records, b.n, b.p, cand_c42_nobi::Cap::NONE, &mut out); }
                        }
                    }
                    bar_ref.wait();
                    let t0 = Instant::now();
                    for b in b_ref.iter().rev() {
                        match m {
                            0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { k2.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out); }
                            3 => { k3.run_entry(&b.records, b.n, b.p, cand_c42_pair2::Cap::NONE, &mut out); }
                            4 => { k4.run_entry(&b.records, b.n, b.p, cand_c42_s3arr::Cap::NONE, &mut out); }
                            5 => { k5.run_entry(&b.records, b.n, b.p, cand_c42_combo::Cap::NONE, &mut out); }
                            6 => { k6.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                            _ => { k7.run_entry(&b.records, b.n, b.p, cand_c42_nobi::Cap::NONE, &mut out); }
                        }
                    }
                    t0.elapsed().as_secs_f64() * 1e6
                });
                let mut out = Vec::with_capacity(900_064);
                let mut k0 = master_a1::Kernel::default();
                let mut k1 = master_a2::Kernel::default();
                let mut k2 = cand_c41_combo::Kernel::default();
                let mut k3 = cand_c42_pair2::Kernel::default();
                let mut k4 = cand_c42_s3arr::Kernel::default();
                let mut k5 = cand_c42_combo::Kernel::default();
                let mut k6 = master_nobi::Kernel::default();
                let mut k7 = cand_c42_nobi::Kernel::default();
                for b in b_ref {
                    match m {
                        0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { k2.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out); }
                        3 => { k3.run_entry(&b.records, b.n, b.p, cand_c42_pair2::Cap::NONE, &mut out); }
                        4 => { k4.run_entry(&b.records, b.n, b.p, cand_c42_s3arr::Cap::NONE, &mut out); }
                        5 => { k5.run_entry(&b.records, b.n, b.p, cand_c42_combo::Cap::NONE, &mut out); }
                        6 => { k6.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                        _ => { k7.run_entry(&b.records, b.n, b.p, cand_c42_nobi::Cap::NONE, &mut out); }
                    }
                }
                bar_ref.wait();
                let t0 = Instant::now();
                for b in b_ref {
                    match m {
                        0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { k2.run_entry(&b.records, b.n, b.p, cand_c41_combo::Cap::NONE, &mut out); }
                        3 => { k3.run_entry(&b.records, b.n, b.p, cand_c42_pair2::Cap::NONE, &mut out); }
                        4 => { k4.run_entry(&b.records, b.n, b.p, cand_c42_s3arr::Cap::NONE, &mut out); }
                        5 => { k5.run_entry(&b.records, b.n, b.p, cand_c42_combo::Cap::NONE, &mut out); }
                        6 => { k6.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                        _ => { k7.run_entry(&b.records, b.n, b.p, cand_c42_nobi::Cap::NONE, &mut out); }
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
