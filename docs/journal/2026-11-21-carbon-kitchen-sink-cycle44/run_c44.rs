#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod master_nobi;
mod cand_c43_base;
mod cand_c43_nbw16;
mod cand_c44_tunnel;
mod cand_c44_s2bni;
mod cand_c44_s3c8;
mod cand_c44_head;
mod cand_c44_nobi;
mod prof_master;
mod prof_nobi;
mod prof_c44_base;
mod prof_c44_head;
mod prof_c44_nobi;

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

fn bwt_of(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb { return ca.cmp(&cb); }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 { p = i as u32; }
        l.push(text[(pos + n - 1) % n]);
    }
    (l, p)
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
    let mut kb = cand_c43_base::Kernel::default();
    let mut kt = cand_c44_tunnel::Kernel::default();
    let mut ks2 = cand_c44_s2bni::Kernel::default();
    let mut kc8 = cand_c44_s3c8::Kernel::default();
    let mut kh = cand_c44_head::Kernel::default();
    let mut knb = master_nobi::Kernel::default();
    let mut knw = cand_c43_nbw16::Kernel::default();
    let mut kn44 = cand_c44_nobi::Kernel::default();

    // 1. Parity & fuzz check (2,000 small/adversarial BWTs + Wikidata + dense-300k + #703)
    let mut rng = 0x517C_C1B7_2722_0A95u64;
    for case in 0..2000usize {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let n = if case < 800 { 1 + (case % 32) } else { 33 + ((rng >> 32) as usize % 200) };
        let alpha = match case % 4 { 0 => 2u8, 1 => 3, 2 => 5, _ => 26 };
        let mut text = Vec::with_capacity(n);
        for _ in 0..n {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            text.push(b'a' + ((rng >> 40) % alpha as u64) as u8);
        }
        let (l, p) = bwt_of(&text);
        let recs = records_of(&l);
        kh.run_entry(&recs, n, p, cand_c44_head::Cap::NONE, &mut out).unwrap(); assert_eq!(out, text);
        kn44.run_entry(&recs, n, p, cand_c44_nobi::Cap::NONE, &mut out).unwrap(); assert_eq!(out, text);
        kc8.run_entry(&recs, n, p, cand_c44_s3c8::Cap::NONE, &mut out).unwrap(); assert_eq!(out, text);
    }
    for b in six.iter().chain([&dense]) {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kt.run_entry(&b.records, b.n, b.p, cand_c44_tunnel::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        ks2.run_entry(&b.records, b.n, b.p, cand_c44_s2bni::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kc8.run_entry(&b.records, b.n, b.p, cand_c44_s3c8::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kh.run_entry(&b.records, b.n, b.p, cand_c44_head::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kn44.run_entry(&b.records, b.n, b.p, cand_c44_nobi::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
    }
    assert!(kh.run_entry(&dense.records, dense.n, dense.p, cand_c44_head::Cap::dense(dense.n), &mut out).is_none());
    println!("PASS: 2000 fuzz BWTs + 6 Wikidata + dense-300k + #703 Cap::dense refusal 100% byte-exact!");

    // 2. 1T Interleaved A/A & A/B + Pre split
    println!("\n=== 1. 1T INTERLEAVED A/A & A/B + PRE SPLIT (50 rounds/block) ===");
    let names = ["MasterA1", "MasterA2", "C43-Base", "S2aTun", "S2b-NI", "S3a-C8", "C44-Head", "Mst-Nobi", "Nobi-W16", "C44-Nobi"];
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
                2 => { kb.run_entry(&b.records, b.n, b.p, cand_c43_base::Cap::NONE, &mut out); }
                3 => { kt.run_entry(&b.records, b.n, b.p, cand_c44_tunnel::Cap::NONE, &mut out); }
                4 => { ks2.run_entry(&b.records, b.n, b.p, cand_c44_s2bni::Cap::NONE, &mut out); }
                5 => { kc8.run_entry(&b.records, b.n, b.p, cand_c44_s3c8::Cap::NONE, &mut out); }
                6 => { kh.run_entry(&b.records, b.n, b.p, cand_c44_head::Cap::NONE, &mut out); }
                7 => { knb.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                8 => { knw.run_entry(&b.records, b.n, b.p, cand_c43_nbw16::Cap::NONE, &mut out); }
                _ => { kn44.run_entry(&b.records, b.n, b.p, cand_c44_nobi::Cap::NONE, &mut out); }
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
                2 => kb.run_stages(&b.records, b.n, b.p, cand_c43_base::Cap::NONE, &mut out),
                3 => kt.run_stages(&b.records, b.n, b.p, cand_c44_tunnel::Cap::NONE, &mut out),
                4 => ks2.run_stages(&b.records, b.n, b.p, cand_c44_s2bni::Cap::NONE, &mut out),
                5 => kc8.run_stages(&b.records, b.n, b.p, cand_c44_s3c8::Cap::NONE, &mut out),
                6 => kh.run_stages(&b.records, b.n, b.p, cand_c44_head::Cap::NONE, &mut out),
                7 => knb.run_stages(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out),
                8 => knw.run_stages(&b.records, b.n, b.p, cand_c43_nbw16::Cap::NONE, &mut out),
                _ => kn44.run_stages(&b.records, b.n, b.p, cand_c44_nobi::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        for k in 0..10 { csum[k] += pct(&mut sc[k], 1, 2); wsum[k] += pct(&mut sw[k], 1, 2); }
        println!("{:14} | M1={:6.1} | A/A={:.3} C43={:.3} Tun={:.3} S2b={:.3} C8={:.3} H44={:.3} | NW16={:.3} N44={:.3}",
            b.name, med[0], med[1]/med[0], med[2]/med[0], med[3]/med[0], med[4]/med[0],
            med[5]/med[0], med[6]/med[0], med[8]/med[7], med[9]/med[7]);
    }
    for k in 0..10 {
        let bt = if k >= 7 { sum1t[7] } else { sum1t[0] };
        let bc = if k >= 7 { csum[7] } else { csum[0] };
        let prev_c = if k >= 7 { csum[8] } else { csum[2] };
        println!("1T_SUM {:10} | tot={:7.1} ({:.4}x) | pre={:6.1} ({:.4}x, {:+.1} vs C43) | emit={:6.1}",
            names[k], sum1t[k], sum1t[k]/bt, csum[k], csum[k]/bc, csum[k]-prev_c, wsum[k]);
    }

    // 3. Full 6-Stage Profile (S1, S2a, S2b, S3a, S4, S5) — 40 rounds/block
    println!("\n=== 2. FULL 6-STAGE PROFILE (40 rounds/block, exact gap vs 53f7ecf93) ===");
    let mut pm = prof_master::Kernel::default();
    let mut pnb = prof_nobi::Kernel::default();
    let mut pb43 = prof_c44_base::Kernel::default();
    let mut ph44 = prof_c44_head::Kernel::default();
    let mut pn44 = prof_c44_nobi::Kernel::default();
    let pnames = ["Prof-Mst53f7", "Prof-C43Base", "Prof-C44Head", "Prof-Nobi53f7", "Prof-C44Nobi"];
    let mut psum = [[0.0f64; 6]; 5];
    for b in &six {
        let mut s: [[Vec<f64>; 6]; 5] = Default::default();
        for i in 0..5 * 40 {
            let m = (i + i / 5) % 5;
            let st = match m {
                0 => pm.run_profile6(&b.records, b.n, b.p, prof_master::Cap::NONE, &mut out),
                1 => pb43.run_profile6(&b.records, b.n, b.p, prof_c44_base::Cap::NONE, &mut out),
                2 => ph44.run_profile6(&b.records, b.n, b.p, prof_c44_head::Cap::NONE, &mut out),
                3 => pnb.run_profile6(&b.records, b.n, b.p, prof_nobi::Cap::NONE, &mut out),
                _ => pn44.run_profile6(&b.records, b.n, b.p, prof_c44_nobi::Cap::NONE, &mut out),
            };
            for j in 0..6 { s[m][j].push(st[j]); }
        }
        for m in 0..5 { for j in 0..6 { psum[m][j] += pct(&mut s[m][j], 1, 2); } }
    }
    for m in 0..5 {
        let pre: f64 = psum[m][0..5].iter().sum();
        let bidx = if m >= 3 { 3 } else { 0 };
        let bpre: f64 = psum[bidx][0..5].iter().sum();
        println!("{:14} | S1={:6.1} S2a={:5.1} S2b={:6.1} S3a={:5.1} S4={:6.1} | Pre={:6.1} ({:.4}x) | S5={:6.1}",
            pnames[m], psum[m][0], psum[m][1], psum[m][2], psum[m][3], psum[m][4], pre, pre/bpre, psum[m][5]);
    }

    // 4. 2T (35 passes) & 4T (25 passes) Concurrent Barriers
    for &(threads, passes) in &[(2usize, 35usize), (4usize, 25usize)] {
        println!("\n=== {}T CONCURRENT BARRIER ({} passes) ===", threads, passes);
        let names_mt = ["MasterA1", "MasterA2", "C43-Base", "S2aTun", "C44-Head", "Mst-Nobi", "C44-Nobi"];
        let mut rmt: [Vec<f64>; 7] = Default::default();
        for pass in 0..passes {
            for off in 0..7 {
                let m = (pass + off) % 7;
                let bar = Barrier::new(threads);
                let t = std::thread::scope(|sc| {
                    let b_ref = &six; let bar_ref = &bar;
                    let mut hs = Vec::with_capacity(threads);
                    for tid in 0..threads {
                        hs.push(sc.spawn(move || {
                            let mut out = Vec::with_capacity(900_064);
                            let mut k0 = master_a1::Kernel::default();
                            let mut k1 = master_a2::Kernel::default();
                            let mut k2 = cand_c43_base::Kernel::default();
                            let mut k3 = cand_c44_tunnel::Kernel::default();
                            let mut k4 = cand_c44_head::Kernel::default();
                            let mut k5 = master_nobi::Kernel::default();
                            let mut k6 = cand_c44_nobi::Kernel::default();
                            for b in b_ref {
                                match m {
                                    0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                                    1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                                    2 => { k2.run_entry(&b.records, b.n, b.p, cand_c43_base::Cap::NONE, &mut out); }
                                    3 => { k3.run_entry(&b.records, b.n, b.p, cand_c44_tunnel::Cap::NONE, &mut out); }
                                    4 => { k4.run_entry(&b.records, b.n, b.p, cand_c44_head::Cap::NONE, &mut out); }
                                    5 => { k5.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                                    _ => { k6.run_entry(&b.records, b.n, b.p, cand_c44_nobi::Cap::NONE, &mut out); }
                                }
                            }
                            bar_ref.wait();
                            let t0 = Instant::now();
                            for idx in 0..b_ref.len() {
                                let b = &b_ref[(idx + tid) % b_ref.len()];
                                match m {
                                    0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                                    1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                                    2 => { k2.run_entry(&b.records, b.n, b.p, cand_c43_base::Cap::NONE, &mut out); }
                                    3 => { k3.run_entry(&b.records, b.n, b.p, cand_c44_tunnel::Cap::NONE, &mut out); }
                                    4 => { k4.run_entry(&b.records, b.n, b.p, cand_c44_head::Cap::NONE, &mut out); }
                                    5 => { k5.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                                    _ => { k6.run_entry(&b.records, b.n, b.p, cand_c44_nobi::Cap::NONE, &mut out); }
                                }
                            }
                            t0.elapsed().as_secs_f64() * 1e6
                        }));
                    }
                    hs.into_iter().map(|h| h.join().unwrap()).fold(0.0f64, f64::max)
                });
                rmt[m].push(t);
            }
        }
        let b10_h = pct(&mut rmt[0].clone(), 1, 10);
        let b50_h = pct(&mut rmt[0].clone(), 1, 2);
        let b10_n = pct(&mut rmt[5].clone(), 1, 10);
        let b50_n = pct(&mut rmt[5].clone(), 1, 2);
        for k in 0..7 {
            let p10 = pct(&mut rmt[k], 1, 10);
            let p50 = pct(&mut rmt[k], 1, 2);
            let (b10, b50) = if k >= 5 { (b10_n, b50_n) } else { (b10_h, b50_h) };
            println!("{}T {:10} | p10={:7.1} ({:.4}x) | p50={:7.1} ({:.4}x)", threads, names_mt[k], p10, p10/b10, p50, p50/b50);
        }
    }
}
