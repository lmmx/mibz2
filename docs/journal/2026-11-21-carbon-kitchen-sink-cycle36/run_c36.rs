#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod master_nobi;
mod prof_master;
mod prof_nobi;
mod cand_wire16;
mod cand_s2afuse;
mod cand_s3tag12;
mod cand_s3tag24;
mod cand_combo;
mod prof_combo;
mod cand_nobi_combo;
mod prof_nobi_combo;

use std::sync::Barrier;
use std::time::Instant;

struct Block { name: String, n: usize, p: u32, records: Vec<u16>, expected: Vec<u8> }

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

fn plain_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    let mut rank_in_l = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        rank_in_l[i] = counts[b as usize];
        counts[b as usize] += 1;
    }
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

fn make_dense_300k() -> Block {
    let n = 300_000usize;
    let mut text = Vec::with_capacity(n);
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..n {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        text.push(if i % 9 == 0 { b' ' } else { b'a' + ((s >> 33) % 26) as u8 });
    }
    let mut sa: Vec<u32> = (0..n as u32).collect();
    sa.sort_by(|&a, &b| {
        let (ia, ib) = (a as usize, b as usize);
        for k in 0..64 {
            let ca = text[(ia + k) % n];
            let cb = text[(ib + k) % n];
            if ca != cb { return ca.cmp(&cb); }
        }
        ia.cmp(&ib)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 { p = i as u32; }
        l.push(text[(pos as usize + n - 1) % n]);
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    Block { name: "dense-300k".into(), n, p, records: records_of(&l), expected }
}

fn load_six() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let mut specs = vec![];
    let mut file = String::new();
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

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn main() {
    let six = load_six();
    let dense = make_dense_300k();
    let mut out = Vec::with_capacity(900_064);

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut knb = master_nobi::Kernel::default();
    let mut kw16 = cand_wire16::Kernel::default();
    let mut ks2a = cand_s2afuse::Kernel::default();
    let mut ks3_12 = cand_s3tag12::Kernel::default();
    let mut ks3_24 = cand_s3tag24::Kernel::default();
    let mut kcb = cand_combo::Kernel::default();
    let mut kncb = cand_nobi_combo::Kernel::default();

    println!("=== 1. PARITY & SCRATCH ON NEW origin/master (53f7ecf93) ===");
    for b in six.iter().chain([&dense]) {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        knb.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kw16.run_entry(&b.records, b.n, b.p, cand_wire16::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        ks2a.run_entry(&b.records, b.n, b.p, cand_s2afuse::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        ks3_12.run_entry(&b.records, b.n, b.p, cand_s3tag12::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        ks3_24.run_entry(&b.records, b.n, b.p, cand_s3tag24::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kcb.run_entry(&b.records, b.n, b.p, cand_combo::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kncb.run_entry(&b.records, b.n, b.p, cand_nobi_combo::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
    }
    let mut f_m = master_a1::Kernel::default();
    let mut f_c = cand_combo::Kernel::default();
    for b in &six {
        f_m.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        f_c.run_entry(&b.records, b.n, b.p, cand_combo::Cap::NONE, &mut out).unwrap();
    }
    let (sm, _, sm_s) = f_m.scratch_report();
    let (sc, _, sc_s) = f_c.scratch_report();
    println!("PASS: 6 Wikidata + dense-300k 100% byte-exact!");
    println!("Master 53f7ecf93 : {:.1} KiB ({sm} B) | {sm_s}", sm as f64 / 1024.0);
    println!("C36-Combo        : {:.1} KiB ({sc} B, {:+.2}%) | {sc_s}", sc as f64 / 1024.0, (sc as f64 / sm as f64 - 1.0) * 100.0);

    println!("\n=== 2. EXACT 6-STAGE PROFILE (40 rounds/block: S1, S2a, S2b, S3a, S4, S5) ===");
    let mut pm = prof_master::Kernel::default();
    let mut pnb = prof_nobi::Kernel::default();
    let mut pcb = prof_combo::Kernel::default();
    let mut pncb = prof_nobi_combo::Kernel::default();
    let pnames = ["Master(53f7)", "Master-Nobi", "C36-Combo", "C36-NobiCmb"];
    let mut psum = [[0.0f64; 6]; 4];
    for b in &six {
        let mut st: [[Vec<f64>; 6]; 4] = Default::default();
        for i in 0..4 * 40 {
            let m = (i + i / 4) % 4;
            let v = match m {
                0 => pm.run_profile6(&b.records, b.n, b.p, prof_master::Cap::NONE, &mut out),
                1 => pnb.run_profile6(&b.records, b.n, b.p, prof_nobi::Cap::NONE, &mut out),
                2 => pcb.run_profile6(&b.records, b.n, b.p, prof_combo::Cap::NONE, &mut out),
                _ => pncb.run_profile6(&b.records, b.n, b.p, prof_nobi_combo::Cap::NONE, &mut out),
            };
            for s in 0..6 { st[m][s].push(v[s]); }
        }
        for m in 0..4 {
            for s in 0..6 { psum[m][s] += pct(&mut st[m][s], 1, 2); }
        }
    }
    for m in 0..4 {
        let pre: f64 = psum[m][0..5].iter().sum();
        let tot = pre + psum[m][5];
        println!("PROF {:12} | S1={:6.1} S2a={:5.1} S2b={:6.1} S3a={:5.1} S4={:6.1} | Pre={:6.1} | S5={:6.1} | Tot={:7.1}",
            pnames[m], psum[m][0], psum[m][1], psum[m][2], psum[m][3], psum[m][4], pre, psum[m][5], tot);
    }

    println!("\n=== 3. 1T INTERLEAVED A/A & A/B + CONSTRUCT SPLIT (50 rounds/block) ===");
    let names = ["MasterA1", "MasterA2", "Wire16", "S2aFuse", "S3Tag12", "S3Tag24", "Combo", "Mst-Nobi", "Nobi-Cmb"];
    let mut sum1t = [0.0f64; 9];
    let mut csum = [0.0f64; 9];
    let mut wsum = [0.0f64; 9];
    for b in &six {
        let mut s: [Vec<f64>; 9] = Default::default();
        for i in 0..9 * 50 {
            let m = (i + i / 9) % 9;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                2 => { kw16.run_entry(&b.records, b.n, b.p, cand_wire16::Cap::NONE, &mut out); }
                3 => { ks2a.run_entry(&b.records, b.n, b.p, cand_s2afuse::Cap::NONE, &mut out); }
                4 => { ks3_12.run_entry(&b.records, b.n, b.p, cand_s3tag12::Cap::NONE, &mut out); }
                5 => { ks3_24.run_entry(&b.records, b.n, b.p, cand_s3tag24::Cap::NONE, &mut out); }
                6 => { kcb.run_entry(&b.records, b.n, b.p, cand_combo::Cap::NONE, &mut out); }
                7 => { knb.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                _ => { kncb.run_entry(&b.records, b.n, b.p, cand_nobi_combo::Cap::NONE, &mut out); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let med: [f64; 9] = std::array::from_fn(|k| pct(&mut s[k], 1, 2));
        for k in 0..9 { sum1t[k] += med[k]; }

        let mut sc: [Vec<f64>; 9] = Default::default();
        let mut sw: [Vec<f64>; 9] = Default::default();
        for i in 0..9 * 35 {
            let m = (i + i / 9) % 9;
            let (c, w) = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out),
                1 => km2.run_stages(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out),
                2 => kw16.run_stages(&b.records, b.n, b.p, cand_wire16::Cap::NONE, &mut out),
                3 => ks2a.run_stages(&b.records, b.n, b.p, cand_s2afuse::Cap::NONE, &mut out),
                4 => ks3_12.run_stages(&b.records, b.n, b.p, cand_s3tag12::Cap::NONE, &mut out),
                5 => ks3_24.run_stages(&b.records, b.n, b.p, cand_s3tag24::Cap::NONE, &mut out),
                6 => kcb.run_stages(&b.records, b.n, b.p, cand_combo::Cap::NONE, &mut out),
                7 => knb.run_stages(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out),
                _ => kncb.run_stages(&b.records, b.n, b.p, cand_nobi_combo::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        for k in 0..9 { csum[k] += pct(&mut sc[k], 1, 2); wsum[k] += pct(&mut sw[k], 1, 2); }
        println!("{:14} | M1={:6.1} | A/A={:.3} W16={:.3} S2a={:.3} S3_12={:.3} S3_24={:.3} Cmb={:.3} | NobiCmb/Nobi={:.3}",
            b.name, med[0], med[1]/med[0], med[2]/med[0], med[3]/med[0], med[4]/med[0], med[5]/med[0], med[6]/med[0], med[8]/med[7]);
    }
    for k in 0..9 {
        let base_t = if k >= 7 { sum1t[7] } else { sum1t[0] };
        let base_c = if k >= 7 { csum[7] } else { csum[0] };
        println!("1T_SUM {:10} | tot={:7.1} ({:.4}x) | pre={:6.1} ({:.4}x, {:+.1} us) | emit={:6.1}",
            names[k], sum1t[k], sum1t[k]/base_t, csum[k], csum[k]/base_c, csum[k]-base_c, wsum[k]);
    }

    println!("\n=== 4. 2T CONCURRENT BARRIER (35 passes) ===");
    let names2 = ["MasterA1", "MasterA2", "Combo", "Mst-Nobi", "Nobi-Cmb"];
    let mut r2t: [Vec<f64>; 5] = Default::default();
    for pass in 0..35 {
        for off in 0..5 {
            let m = (pass + off) % 5;
            let bar = Barrier::new(2);
            let t = std::thread::scope(|sc| {
                let b_ref = &six; let bar_ref = &bar;
                let h = sc.spawn(move || {
                    let mut out = Vec::with_capacity(900_064);
                    let mut k0 = master_a1::Kernel::default();
                    let mut k1 = master_a2::Kernel::default();
                    let mut k2 = cand_combo::Kernel::default();
                    let mut k3 = master_nobi::Kernel::default();
                    let mut k4 = cand_nobi_combo::Kernel::default();
                    for b in b_ref {
                        match m {
                            0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { k2.run_entry(&b.records, b.n, b.p, cand_combo::Cap::NONE, &mut out); }
                            3 => { k3.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                            _ => { k4.run_entry(&b.records, b.n, b.p, cand_nobi_combo::Cap::NONE, &mut out); }
                        }
                    }
                    bar_ref.wait();
                    let t0 = Instant::now();
                    for b in b_ref.iter().rev() {
                        match m {
                            0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { k2.run_entry(&b.records, b.n, b.p, cand_combo::Cap::NONE, &mut out); }
                            3 => { k3.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                            _ => { k4.run_entry(&b.records, b.n, b.p, cand_nobi_combo::Cap::NONE, &mut out); }
                        }
                    }
                    t0.elapsed().as_secs_f64() * 1e6
                });
                let mut out = Vec::with_capacity(900_064);
                let mut k0 = master_a1::Kernel::default();
                let mut k1 = master_a2::Kernel::default();
                let mut k2 = cand_combo::Kernel::default();
                let mut k3 = master_nobi::Kernel::default();
                let mut k4 = cand_nobi_combo::Kernel::default();
                for b in b_ref {
                    match m {
                        0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { k2.run_entry(&b.records, b.n, b.p, cand_combo::Cap::NONE, &mut out); }
                        3 => { k3.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                        _ => { k4.run_entry(&b.records, b.n, b.p, cand_nobi_combo::Cap::NONE, &mut out); }
                    }
                }
                bar_ref.wait();
                let t0 = Instant::now();
                for b in b_ref {
                    match m {
                        0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { k2.run_entry(&b.records, b.n, b.p, cand_combo::Cap::NONE, &mut out); }
                        3 => { k3.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                        _ => { k4.run_entry(&b.records, b.n, b.p, cand_nobi_combo::Cap::NONE, &mut out); }
                    }
                }
                t0.elapsed().as_secs_f64().mul_add(1e6, 0.0).max(h.join().unwrap())
            });
            r2t[m].push(t);
        }
    }
    let p10_0 = pct(&mut r2t[0], 1, 10); let p50_0 = pct(&mut r2t[0], 1, 2);
    let p10_3 = pct(&mut r2t[3], 1, 10); let p50_3 = pct(&mut r2t[3], 1, 2);
    for k in 0..5 {
        let p10 = pct(&mut r2t[k], 1, 10);
        let p50 = pct(&mut r2t[k], 1, 2);
        let (b10, b50) = if k >= 3 { (p10_3, p50_3) } else { (p10_0, p50_0) };
        println!("2T {:10} | p10={:8.1} us ({:.4}x) | p50={:8.1} us ({:.4}x)",
            names2[k], p10, p10/b10, p50, p50/b50);
    }
}
