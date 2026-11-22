#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_c33_w16_d12;
mod cand_c34_diet;
mod cand_c34_bi0;
mod cand_c34_hoistp;

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
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    let mut out = vec![0u8; n];
    let mut x = p as usize;
    for k in (0..n).rev() {
        let b = l[x];
        out[k] = b;
        x = (starts[b as usize] + rank_in_l[x]) as usize;
    }
    out
}

fn make_dense(n: usize) -> Block {
    let mut text = Vec::with_capacity(n);
    let mut s = 0x9E37_79B9_7F4A_7C15u64 ^ (n as u64);
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
    Block { name: format!("dense-{}k", n / 1000), n, p, records: records_of(&l), expected }
}

fn make_periodic(name: &str, period_len: usize, reps: usize) -> Block {
    let mut unit = Vec::with_capacity(period_len);
    let mut s = 0x1234_5678_9ABC_DEF1u64 ^ (period_len as u64);
    for i in 0..period_len {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        unit.push(if i % 7 == 0 { b'_' } else { b'a' + ((s >> 33) % 16) as u8 });
    }
    let n = period_len * reps;
    let mut text = Vec::with_capacity(n);
    for _ in 0..reps { text.extend_from_slice(&unit); }
    let mut sa: Vec<usize> = (0..period_len).collect();
    sa.sort_by(|&a, &b| {
        for k in 0..period_len {
            let ca = unit[(a + k) % period_len];
            let cb = unit[(b + k) % period_len];
            if ca != cb { return ca.cmp(&cb); }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        let ch = unit[(pos + period_len - 1) % period_len];
        if pos == 0 { p = (i * reps) as u32; }
        for _ in 0..reps { l.push(ch); }
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    Block { name: name.into(), n, p, records: records_of(&l), expected }
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

fn bench_mt(six: &[Block], threads: usize, passes: usize) {
    let names = ["MasterA1", "MasterA2", "Neon-Diet", "Neon-C33", "HoistP"];
    let mut res: [Vec<f64>; 5] = Default::default();
    for pass in 0..passes {
        for off in 0..5 {
            let m = (pass + off) % 5;
            let bar = Barrier::new(threads);
            let t = std::thread::scope(|sc| {
                let mut handles = Vec::with_capacity(threads);
                for tid in 0..threads {
                    let b_ref = six;
                    let bar_ref = &bar;
                    handles.push(sc.spawn(move || {
                        let mut out = Vec::with_capacity(900_064);
                        let mut km1 = master_a1::Kernel::default();
                        let mut km2 = master_a2::Kernel::default();
                        let mut kd = cand_c34_diet::Kernel::default();
                        let mut kb = cand_c33_w16_d12::Kernel::default();
                        let mut kh = cand_c34_hoistp::Kernel::default();
                        for b in b_ref {
                            match m {
                                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                                2 => { kd.run_entry(&b.records, b.n, b.p, cand_c34_diet::Cap::NONE, &mut out); }
                                3 => { kb.run_entry(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out); }
                                _ => { kh.run_entry(&b.records, b.n, b.p, cand_c34_hoistp::Cap::NONE, &mut out); }
                            }
                        }
                        bar_ref.wait();
                        let t0 = Instant::now();
                        for idx in 0..b_ref.len() {
                            let b = &b_ref[(idx + tid) % b_ref.len()];
                            match m {
                                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                                2 => { kd.run_entry(&b.records, b.n, b.p, cand_c34_diet::Cap::NONE, &mut out); }
                                3 => { kb.run_entry(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out); }
                                _ => { kh.run_entry(&b.records, b.n, b.p, cand_c34_hoistp::Cap::NONE, &mut out); }
                            }
                        }
                        t0.elapsed().as_secs_f64() * 1e6
                    }));
                }
                handles.into_iter().map(|h| h.join().unwrap()).fold(0.0f64, f64::max)
            });
            res[m].push(t);
        }
    }
    let p10_0 = pct(&mut res[0], 1, 10);
    let p50_0 = pct(&mut res[0], 1, 2);
    for k in 0..5 {
        let p10 = pct(&mut res[k], 1, 10);
        let p50 = pct(&mut res[k], 1, 2);
        println!("{}T {:10} | p10={:8.1} us ({:.4}x) | p50={:8.1} us ({:.4}x)",
            threads, names[k], p10, p10 / p10_0, p50, p50 / p50_0);
    }
}

fn main() {
    let six = load_six();
    let mut out = Vec::with_capacity(900_064);

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kd = cand_c34_diet::Kernel::default();
    let mut kb = cand_c33_w16_d12::Kernel::default();
    let mut kb0 = cand_c34_bi0::Kernel::default();
    let mut kh = cand_c34_hoistp::Kernel::default();

    println!("=== 1. PARITY & SCRATCH (6 Wikidata + edge p=0, p=n-1 + dense sweep) ===");
    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kd.run_entry(&b.records, b.n, b.p, cand_c34_diet::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kb.run_entry(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kb0.run_entry(&b.records, b.n, b.p, cand_c34_bi0::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kh.run_entry(&b.records, b.n, b.p, cand_c34_hoistp::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
    }
    // Check HoistP on edge p=0 and p=n-1 (on a valid block with p shifted -> check parity vs MasterA1!)
    for &test_p in &[0u32, 1, 17, six[0].n as u32 - 1] {
        let mut out_m = Vec::new();
        km1.run_entry(&six[0].records, six[0].n, test_p, master_a1::Cap::NONE, &mut out_m).unwrap();
        kh.run_entry(&six[0].records, six[0].n, test_p, cand_c34_hoistp::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, out_m, "HoistP edge p={test_p}");
    }
    let sm = km1.scratch_report().0;
    let sd = kd.scratch_report().0;
    let sb = kb.scratch_report().0;
    let sh = kh.scratch_report().0;
    println!("Scratch: Master={:.1} KiB | Neon-Diet={:.1} KiB ({:+.2}%) | Neon-C33={:.1} KiB ({:+.2}%) | HoistP={:.1} KiB ({:+.2}%)",
        sm as f64 / 1024.0, sd as f64 / 1024.0, (sd as f64 / sm as f64 - 1.0) * 100.0,
        sb as f64 / 1024.0, (sb as f64 / sm as f64 - 1.0) * 100.0,
        sh as f64 / 1024.0, (sh as f64 / sm as f64 - 1.0) * 100.0);

    println!("\n=== 2. 1T INTERLEAVED A/A & A/B + STAGES (60 rounds/block) ===");
    let names = ["MasterA1", "MasterA2", "Neon-Diet", "Neon-C33", "HoistP"];
    let mut sum1t = [0.0f64; 5];
    let mut csum = [0.0f64; 5];
    let mut wsum = [0.0f64; 5];
    for b in &six {
        let mut s: [Vec<f64>; 5] = Default::default();
        for i in 0..5 * 60 {
            let m = (i + i / 5) % 5;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                2 => { kd.run_entry(&b.records, b.n, b.p, cand_c34_diet::Cap::NONE, &mut out); }
                3 => { kb.run_entry(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out); }
                _ => { kh.run_entry(&b.records, b.n, b.p, cand_c34_hoistp::Cap::NONE, &mut out); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let med: [f64; 5] = std::array::from_fn(|k| pct(&mut s[k], 1, 2));
        for k in 0..5 { sum1t[k] += med[k]; }
        println!("{:14} | M1={:6.1} | A/A={:.3} Diet={:.3} C33={:.3} HoistP={:.3}",
            b.name, med[0], med[1]/med[0], med[2]/med[0], med[3]/med[0], med[4]/med[0]);

        let mut sc: [Vec<f64>; 5] = Default::default();
        let mut sw: [Vec<f64>; 5] = Default::default();
        for i in 0..5 * 40 {
            let m = (i + i / 5) % 5;
            let (c, w) = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out),
                1 => km2.run_stages(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out),
                2 => kd.run_stages(&b.records, b.n, b.p, cand_c34_diet::Cap::NONE, &mut out),
                3 => kb.run_stages(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out),
                _ => kh.run_stages(&b.records, b.n, b.p, cand_c34_hoistp::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        for k in 0..5 { csum[k] += pct(&mut sc[k], 1, 2); wsum[k] += pct(&mut sw[k], 1, 2); }
    }
    for k in 0..5 {
        println!("1T_SUM {:10} | {:8.1} us ({:.4}x M1) | c={:6.1} ({:.4}x) w={:6.1} ({:.4}x)",
            names[k], sum1t[k], sum1t[k] / sum1t[0], csum[k], csum[k]/csum[0], wsum[k], wsum[k]/wsum[0]);
    }

    println!("\n=== 3. XOR_GATE PHASE-TRANSITION CURVE (m = 18K..270K, 50 rounds/block) ===");
    let sweep_blocks = vec![
        make_dense(20_000),
        make_dense(35_000),
        make_dense(50_000),
        make_dense(65_000),
        make_dense(80_000),
        make_dense(100_000),
        make_dense(150_000),
        make_dense(300_000),
        make_periodic("per-105k-d3", 35_000, 3),
        make_periodic("per-150k-d2", 75_000, 2),
    ];
    for b in &sweep_blocks {
        let cnt = kd.run_entry(&b.records, b.n, b.p, cand_c34_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kb0.run_entry(&b.records, b.n, b.p, cand_c34_bi0::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let mut s: [Vec<f64>; 3] = Default::default();
        let mut cw: [Vec<f64>; 4] = Default::default();
        for i in 0..3 * 50 {
            let m = (i + i / 3) % 3;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                1 => { kd.run_entry(&b.records, b.n, b.p, cand_c34_diet::Cap::NONE, &mut out); }
                _ => { kb0.run_entry(&b.records, b.n, b.p, cand_c34_bi0::Cap::NONE, &mut out); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        for i in 0..2 * 30 {
            let m = (i + i / 2) % 2;
            let (c, w) = if m == 0 {
                kd.run_stages(&b.records, b.n, b.p, cand_c34_diet::Cap::NONE, &mut out)
            } else {
                kb0.run_stages(&b.records, b.n, b.p, cand_c34_bi0::Cap::NONE, &mut out)
            };
            cw[2*m].push(c); cw[2*m+1].push(w);
        }
        let m0 = pct(&mut s[0], 1, 2);
        let md = pct(&mut s[1], 1, 2);
        let mb = pct(&mut s[2], 1, 2);
        let cd = pct(&mut cw[0], 1, 2); let wd = pct(&mut cw[1], 1, 2);
        let cb = pct(&mut cw[2], 1, 2); let wb = pct(&mut cw[3], 1, 2);
        println!("GATE {:12} | m={:7} (4m={:5.1} KiB) | M1={:6.1} Diet={:6.1} ({:.3}x) Bi={:6.1} ({:.3}x) | Bi/Diet={:.4}x | dc={:+.1} dw={:+.1}",
            b.name, cnt.retained, cnt.retained as f64 * 4.0 / 1024.0,
            m0, md, md/m0, mb, mb/m0, mb/md, cb - cd, wb - wd);
    }

    println!("\n=== 4. MULTI-THREAD CONTENTION SCALING (2T & 4T CONCURRENT BARRIERS) ===");
    bench_mt(&six, 2, 40);
    bench_mt(&six, 4, 25);
}
