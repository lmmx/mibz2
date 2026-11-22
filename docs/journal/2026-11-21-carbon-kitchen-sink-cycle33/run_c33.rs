#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_links_only;
mod cand_c33_wire4;
mod cand_c33_wire8;
mod cand_c33_dedup24;
mod cand_c33_dedup12;
mod cand_c33_w8_d24;
mod cand_c33_quad;

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

fn bwt_of(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<u32> = (0..n as u32).collect();
    sa.sort_by(|&a, &b| {
        let (ia, ib) = (a as usize, b as usize);
        for k in 0..n.min(128) {
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
    (l, p)
}

fn make_dense_300k() -> Block {
    let n = 300_000usize;
    let mut text = Vec::with_capacity(n);
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..n {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        text.push(if i % 9 == 0 { b' ' } else { b'a' + ((s >> 33) % 26) as u8 });
    }
    let (l, p) = bwt_of(&text);
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    Block { name: "dense-300k".into(), n, p, records: records_of(&l), expected }
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
    // Periodic BWT via exact sorted unit rotations repeated `reps` times
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

fn main() {
    let six = load_six();
    let dense = make_dense_300k();
    let per3 = make_periodic("periodic-105k-d3", 35_000, 3);
    let per2 = make_periodic("periodic-150k-d2", 75_000, 2);
    let mut out = Vec::with_capacity(900_064);

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kc31 = cand_links_only::Kernel::default();
    let mut kw4 = cand_c33_wire4::Kernel::default();
    let mut kw8 = cand_c33_wire8::Kernel::default();
    let mut kd24 = cand_c33_dedup24::Kernel::default();
    let mut kd12 = cand_c33_dedup12::Kernel::default();
    let mut kwd = cand_c33_w8_d24::Kernel::default();
    let mut kq = cand_c33_quad::Kernel::default();

    println!("=== 1. EXACT PARITY (6 Wikidata + dense-300k + periodic-d2/d3 + 50 malformed n=70k) ===");
    let (mut w_c31, mut w_d24, mut w_d12) = (0usize, 0, 0);
    for b in six.iter().chain([&dense, &per2, &per3]) {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "m1 {}", b.name);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "m2 {}", b.name);
        let c31 = kc31.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "c31 {}", b.name);
        kw4.run_entry(&b.records, b.n, b.p, cand_c33_wire4::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "w4 {}", b.name);
        kw8.run_entry(&b.records, b.n, b.p, cand_c33_wire8::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "w8 {}", b.name);
        let cd24 = kd24.run_entry(&b.records, b.n, b.p, cand_c33_dedup24::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "d24 {}", b.name);
        let cd12 = kd12.run_entry(&b.records, b.n, b.p, cand_c33_dedup12::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "d12 {}", b.name);
        kwd.run_entry(&b.records, b.n, b.p, cand_c33_w8_d24::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "wd {}", b.name);
        kq.run_entry(&b.records, b.n, b.p, cand_c33_quad::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "quad {}", b.name);
        if b.name.starts_with("block") {
            w_c31 += c31.words; w_d24 += cd24.words; w_d12 += cd12.words;
        }
    }
    // Malformed non-BWT fuzzing at n = 70,000 >= 65,536 (triggers XOR_GATE + QuadEmit!)
    let mut seed = 0xCAFE_BABE_1234_5678u64;
    for trial in 0..30 {
        let n = 70_000usize;
        let mut l = Vec::with_capacity(n);
        for _ in 0..n {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            l.push(b'a' + ((seed >> 33) % 8) as u8);
        }
        let recs = records_of(&l);
        let p = (trial * 1973) as u32 % n as u32;
        kq.run_entry(&recs, n, p, cand_c33_quad::Cap::NONE, &mut out).unwrap();
        assert_eq!(out.len(), n);
        kwd.run_entry(&recs, n, p, cand_c33_w8_d24::Cap::NONE, &mut out).unwrap();
        assert_eq!(out.len(), n);
    }
    println!("PASS: 6 Wikidata + dense-300k + periodic-150k-d2 + periodic-105k-d3 + 30 malformed n=70k!");
    println!("6-blk unique words: Neon-C31={} | Dedup24={} | Dedup12={}", w_c31, w_d24, w_d12);

    // Measure fresh 6-block cumulative scratch
    let mut skm1 = master_a1::Kernel::default();
    let mut skwd = cand_c33_w8_d24::Kernel::default();
    let mut skq = cand_c33_quad::Kernel::default();
    for b in &six {
        skm1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        skwd.run_entry(&b.records, b.n, b.p, cand_c33_w8_d24::Cap::NONE, &mut out).unwrap();
        skq.run_entry(&b.records, b.n, b.p, cand_c33_quad::Cap::NONE, &mut out).unwrap();
    }
    let sm = skm1.scratch_report().0;
    let swd = skwd.scratch_report().0;
    let sq = skq.scratch_report().0;
    println!("Scratch: Master={:.1} KiB | W8+D24={:.1} KiB ({:+.2}%) | Quad={:.1} KiB ({:+.2}%)",
        sm as f64 / 1024.0, swd as f64 / 1024.0, (swd as f64 / sm as f64 - 1.0) * 100.0,
        sq as f64 / 1024.0, (sq as f64 / sm as f64 - 1.0) * 100.0);

    println!("\n=== 2. STAGE ATTRIBUTION (`construct` + `walk`, 40 rounds/block) ===");
    let mut csum = [0.0f64; 7];
    let mut wsum = [0.0f64; 7];
    let names_st = ["MasterA1", "Neon-C31", "Wire4", "Wire8", "Dedup24", "W8+D24", "Quad"];
    for b in &six {
        let mut sc: [Vec<f64>; 7] = Default::default();
        let mut sw: [Vec<f64>; 7] = Default::default();
        for i in 0..7 * 40 {
            let m = (i + i / 7) % 7;
            let (c, w) = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out),
                1 => kc31.run_stages(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out),
                2 => kw4.run_stages(&b.records, b.n, b.p, cand_c33_wire4::Cap::NONE, &mut out),
                3 => kw8.run_stages(&b.records, b.n, b.p, cand_c33_wire8::Cap::NONE, &mut out),
                4 => kd24.run_stages(&b.records, b.n, b.p, cand_c33_dedup24::Cap::NONE, &mut out),
                5 => kwd.run_stages(&b.records, b.n, b.p, cand_c33_w8_d24::Cap::NONE, &mut out),
                _ => kq.run_stages(&b.records, b.n, b.p, cand_c33_quad::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        let mc: [f64; 7] = std::array::from_fn(|k| pct(&mut sc[k], 1, 2));
        let mw: [f64; 7] = std::array::from_fn(|k| pct(&mut sw[k], 1, 2));
        for k in 0..7 { csum[k] += mc[k]; wsum[k] += mw[k]; }
        println!("{:14} | c/w: M1={:.0}/{:.0} C31={:.0}/{:.0} W8={:.0}/{:.0} D24={:.0}/{:.0} WD={:.0}/{:.0} Q={:.0}/{:.0}",
            b.name, mc[0], mw[0], mc[1], mw[1], mc[3], mw[3], mc[4], mw[4], mc[5], mw[5], mc[6], mw[6]);
    }
    for k in 0..7 {
        println!("STAGE_SUM {:10} | construct={:7.1} us ({:+.1} vs C31) | walk={:7.1} us ({:+.1} vs C31) | sum={:7.1} us ({:.4}x M1)",
            names_st[k], csum[k], csum[k] - csum[1], wsum[k], wsum[k] - wsum[1],
            csum[k] + wsum[k], (csum[k] + wsum[k]) / (csum[0] + wsum[0]));
    }

    println!("\n=== 3. 1T INTERLEAVED A/A & A/B (40 rounds/block, 9 candidates) ===");
    let names = ["MasterA1", "MasterA2", "Neon-C31", "Wire4", "Wire8", "Dedup24", "Dedup12", "W8+D24", "Quad"];
    let mut sum1t = [0.0f64; 9];
    for b in &six {
        let mut s: [Vec<f64>; 9] = Default::default();
        for i in 0..9 * 40 {
            let m = (i + i / 9) % 9;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                2 => { kc31.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                3 => { kw4.run_entry(&b.records, b.n, b.p, cand_c33_wire4::Cap::NONE, &mut out); }
                4 => { kw8.run_entry(&b.records, b.n, b.p, cand_c33_wire8::Cap::NONE, &mut out); }
                5 => { kd24.run_entry(&b.records, b.n, b.p, cand_c33_dedup24::Cap::NONE, &mut out); }
                6 => { kd12.run_entry(&b.records, b.n, b.p, cand_c33_dedup12::Cap::NONE, &mut out); }
                7 => { kwd.run_entry(&b.records, b.n, b.p, cand_c33_w8_d24::Cap::NONE, &mut out); }
                _ => { kq.run_entry(&b.records, b.n, b.p, cand_c33_quad::Cap::NONE, &mut out); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let med: [f64; 9] = std::array::from_fn(|k| pct(&mut s[k], 1, 2));
        for k in 0..9 { sum1t[k] += med[k]; }
        println!("{:14} | M1={:6.1} | A/A={:.3} C31={:.3} W4={:.3} W8={:.3} D24={:.3} D12={:.3} WD={:.3} Q={:.3}",
            b.name, med[0], med[1]/med[0], med[2]/med[0], med[3]/med[0], med[4]/med[0],
            med[5]/med[0], med[6]/med[0], med[7]/med[0], med[8]/med[0]);
    }
    for k in 0..9 {
        println!("1T_SUM {:10} | {:8.1} us | {:.4}x vs M1 | {:+.1} us vs C31",
            names[k], sum1t[k], sum1t[k] / sum1t[0], sum1t[k] - sum1t[2]);
    }

    // Also measure dense-300k 1T
    let mut sd: [Vec<f64>; 5] = Default::default();
    for i in 0..5 * 40 {
        let m = (i + i / 5) % 5;
        let t0 = Instant::now();
        match m {
            0 => { km1.run_entry(&dense.records, dense.n, dense.p, master_a1::Cap::NONE, &mut out); }
            1 => { kc31.run_entry(&dense.records, dense.n, dense.p, cand_links_only::Cap::NONE, &mut out); }
            2 => { kw8.run_entry(&dense.records, dense.n, dense.p, cand_c33_wire8::Cap::NONE, &mut out); }
            3 => { kwd.run_entry(&dense.records, dense.n, dense.p, cand_c33_w8_d24::Cap::NONE, &mut out); }
            _ => { kq.run_entry(&dense.records, dense.n, dense.p, cand_c33_quad::Cap::NONE, &mut out); }
        }
        sd[m].push(t0.elapsed().as_secs_f64() * 1e6);
    }
    let md: [f64; 5] = std::array::from_fn(|k| pct(&mut sd[k], 1, 2));
    println!("DENSE_300K     | M1={:.1} | C31={:.3}x W8={:.3}x WD={:.3}x Quad={:.3}x",
        md[0], md[1]/md[0], md[2]/md[0], md[3]/md[0], md[4]/md[0]);

    println!("\n=== 4. 2T CONCURRENT BARRIER (30 passes, 5 candidates) ===");
    let mut r2t: [Vec<f64>; 5] = Default::default();
    let names2 = ["MasterA1", "MasterA2", "Neon-C31", "W8+D24", "Quad"];
    for pass in 0..30 {
        for off in 0..5 {
            let m = (pass + off) % 5;
            let bar = Barrier::new(2);
            let t = std::thread::scope(|sc| {
                let b_ref = &six; let bar_ref = &bar;
                let h = sc.spawn(move || {
                    let mut out = Vec::with_capacity(900_064);
                    let mut km1 = master_a1::Kernel::default();
                    let mut km2 = master_a2::Kernel::default();
                    let mut kc = cand_links_only::Kernel::default();
                    let mut kwd = cand_c33_w8_d24::Kernel::default();
                    let mut kq = cand_c33_quad::Kernel::default();
                    for b in b_ref {
                        match m {
                            0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { kc.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                            3 => { kwd.run_entry(&b.records, b.n, b.p, cand_c33_w8_d24::Cap::NONE, &mut out); }
                            _ => { kq.run_entry(&b.records, b.n, b.p, cand_c33_quad::Cap::NONE, &mut out); }
                        }
                    }
                    bar_ref.wait();
                    let t0 = Instant::now();
                    for b in b_ref.iter().rev() {
                        match m {
                            0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { kc.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                            3 => { kwd.run_entry(&b.records, b.n, b.p, cand_c33_w8_d24::Cap::NONE, &mut out); }
                            _ => { kq.run_entry(&b.records, b.n, b.p, cand_c33_quad::Cap::NONE, &mut out); }
                        }
                    }
                    t0.elapsed().as_secs_f64() * 1e6
                });
                let mut out = Vec::with_capacity(900_064);
                let mut km1 = master_a1::Kernel::default();
                let mut km2 = master_a2::Kernel::default();
                let mut kc = cand_links_only::Kernel::default();
                let mut kwd = cand_c33_w8_d24::Kernel::default();
                let mut kq = cand_c33_quad::Kernel::default();
                for b in b_ref {
                    match m {
                        0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { kc.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                        3 => { kwd.run_entry(&b.records, b.n, b.p, cand_c33_w8_d24::Cap::NONE, &mut out); }
                        _ => { kq.run_entry(&b.records, b.n, b.p, cand_c33_quad::Cap::NONE, &mut out); }
                    }
                }
                bar_ref.wait();
                let t0 = Instant::now();
                for b in b_ref {
                    match m {
                        0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { kc.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                        3 => { kwd.run_entry(&b.records, b.n, b.p, cand_c33_w8_d24::Cap::NONE, &mut out); }
                        _ => { kq.run_entry(&b.records, b.n, b.p, cand_c33_quad::Cap::NONE, &mut out); }
                    }
                }
                let t1 = t0.elapsed().as_secs_f64() * 1e6;
                t1.max(h.join().unwrap())
            });
            r2t[m].push(t);
        }
    }
    let p10_0 = pct(&mut r2t[0], 1, 10);
    let p50_0 = pct(&mut r2t[0], 1, 2);
    for k in 0..5 {
        let p10 = pct(&mut r2t[k], 1, 10);
        let p50 = pct(&mut r2t[k], 1, 2);
        println!("2T {:10} | p10={:8.1} us ({:.4}x) | p50={:8.1} us ({:.4}x)",
            names2[k], p10, p10 / p10_0, p50, p50 / p50_0);
    }
}
