#![forbid(unsafe_code)]

#[allow(dead_code)]
mod master_a1;
#[allow(dead_code)]
mod master_a2;
mod cand_neon_diet;
mod cand_neon_bi;

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

fn bwt_small(text: &[u8]) -> (Vec<u8>, u32) {
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

fn make_spiral_200k() -> Block {
    // Exact single-cycle BWT of `a^50000 b^49999 a^50001 b^50000` (n = 200,000, p = 1):
    // Thick strips of widths 99,999 and 99,996 (> CERTIFIED_WIDTH = 32,768) trigger `eliminate::<true>`!
    let mut l = Vec::with_capacity(200_000);
    l.push(b'b');
    l.push(b'b');
    l.extend(std::iter::repeat(b'a').take(99_999));
    l.extend(std::iter::repeat(b'b').take(99_996));
    l.push(b'a');
    l.push(b'b');
    l.push(b'a');
    let p = 1u32;
    let expected = plain_invert(&l, p);
    Block { name: "spiral-200k".into(), n: l.len(), p, records: records_of(&l), expected }
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
    let names = ["MasterA1", "MasterA2", "Neon-Diet", "Neon-Bi"];
    let mut res: [Vec<f64>; 4] = Default::default();
    for pass in 0..passes {
        for off in 0..4 {
            let m = (pass + off) % 4;
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
                        let mut kd = cand_neon_diet::Kernel::default();
                        let mut kb = cand_neon_bi::Kernel::default();
                        for b in b_ref {
                            match m {
                                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                                2 => { kd.run_entry(&b.records, b.n, b.p, cand_neon_diet::Cap::NONE, &mut out); }
                                _ => { kb.run_entry(&b.records, b.n, b.p, cand_neon_bi::Cap::NONE, &mut out); }
                            }
                        }
                        bar_ref.wait();
                        let t0 = Instant::now();
                        for idx in 0..b_ref.len() {
                            let b = &b_ref[(idx + tid) % b_ref.len()];
                            match m {
                                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                                2 => { kd.run_entry(&b.records, b.n, b.p, cand_neon_diet::Cap::NONE, &mut out); }
                                _ => { kb.run_entry(&b.records, b.n, b.p, cand_neon_bi::Cap::NONE, &mut out); }
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
    for k in 0..4 {
        let p10 = pct(&mut res[k], 1, 10);
        let p50 = pct(&mut res[k], 1, 2);
        println!("{}T {:10} | p10={:8.1} us ({:.4}x) | p50={:8.1} us ({:.4}x)",
            threads, names[k], p10, p10 / p10_0, p50, p50 / p50_0);
    }
}

fn main() {
    println!("=== 1. EXHAUSTIVE VERIFICATION (67,968+ BWTs + 1,100 malformed + #703 Cap::dense) ===");
    let mut kd = cand_neon_diet::Kernel::default();
    let mut kb = cand_neon_bi::Kernel::default();
    let mut out = Vec::with_capacity(900_064);
    let mut checked = 0usize;

    // 1a. All binary strings n = 1..=14 (32,766 strings)
    let mut text = Vec::new();
    for n in 1..=14usize {
        for mask in 0..(1u32 << n) {
            text.clear();
            for j in 0..n { text.push(b'a' + ((mask >> j) & 1) as u8); }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            let lim = if (mask & 3) == 0 { 1 } else if (mask & 3) == 1 { 2 } else { 4 };
            kd.run_limit(&recs, n, p, lim, cand_neon_diet::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, text, "diet bin n={n} mask={mask}");
            kb.run_limit(&recs, n, p, lim, cand_neon_bi::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, text, "bi bin n={n} mask={mask}");
            checked += 1;
        }
    }
    // 1b. All ternary strings n = 1..=9 (29,523 strings)
    for n in 1..=9usize {
        let total = 3u32.pow(n as u32);
        for mut code in 0..total {
            text.clear();
            for _ in 0..n { text.push(b'a' + (code % 3) as u8); code /= 3; }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            let lim = if (checked & 1) == 0 { 1 } else { 2 };
            kd.run_limit(&recs, n, p, lim, cand_neon_diet::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, text, "diet ter n={n}");
            kb.run_limit(&recs, n, p, lim, cand_neon_bi::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, text, "bi ter n={n}");
            checked += 1;
        }
    }
    // 1c. 5,679 random BWTs n = 10..=120 -> exact 67,968 total BWTs!
    let mut seed = 0x517C_C1B7_2722_0A95u64;
    for t in 0..5_679usize {
        let n = 10 + (t % 111);
        let alpha = 2 + (t % 7);
        text.clear();
        for _ in 0..n {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            text.push(b'a' + ((seed >> 33) as usize % alpha) as u8);
        }
        let (l, p) = bwt_small(&text);
        let recs = records_of(&l);
        let lim = [1u32, 2, 4, 16][t & 3];
        kd.run_limit(&recs, n, p, lim, cand_neon_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, text);
        kb.run_limit(&recs, n, p, lim, cand_neon_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, text);
        checked += 1;
    }

    // 1d. 1,000 small malformed non-BWT columns + 100 large malformed n=70,000 (>= XOR_GATE!)
    for t in 0..1_000usize {
        let n = 1 + (t % 512);
        let mut l = Vec::with_capacity(n);
        for _ in 0..n {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            l.push(b'a' + ((seed >> 33) % 12) as u8);
        }
        let recs = records_of(&l);
        let p = (t as u32 * 37) % n as u32;
        kd.run_limit(&recs, n, p, 2, cand_neon_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out.len(), n);
        kb.run_limit(&recs, n, p, 2, cand_neon_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out.len(), n);
    }
    for t in 0..100usize {
        let n = 70_000usize;
        let mut l = Vec::with_capacity(n);
        for _ in 0..n {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            l.push(b'a' + ((seed >> 33) % 10) as u8);
        }
        let recs = records_of(&l);
        let p = (t as u32 * 997) % n as u32;
        kd.run_entry(&recs, n, p, cand_neon_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out.len(), n);
        kb.run_entry(&recs, n, p, cand_neon_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out.len(), n);
    }

    // 1e. #703 Cap::dense exact refusal scratch test on fresh kernels!
    let dense300 = make_dense_300k();
    let mut ref_m = master_a1::Kernel::default();
    let mut ref_d = cand_neon_diet::Kernel::default();
    let mut ref_b = cand_neon_bi::Kernel::default();
    let cap_m = master_a1::Cap::dense(dense300.n);
    let cap_d = cand_neon_diet::Cap::dense(dense300.n);
    let cap_b = cand_neon_bi::Cap::dense(dense300.n);
    assert!(ref_m.run_entry(&dense300.records, dense300.n, dense300.p, cap_m, &mut out).is_none());
    assert!(ref_d.run_entry(&dense300.records, dense300.n, dense300.p, cap_d, &mut out).is_none());
    assert!(ref_b.run_entry(&dense300.records, dense300.n, dense300.p, cap_b, &mut out).is_none());
    println!(
        "PASS: {} exhaustive BWTs + 1,000 small malformed + 100 large (n=70k) malformed!",
        checked
    );
    println!(
        "#703 Cap::dense refusal scratch: Master={} B | Neon-Diet={} B | Neon-Bi={} B",
        ref_m.scratch_report().0, ref_d.scratch_report().0, ref_b.scratch_report().0
    );

    // 2. Full corpus parity + fresh cumulative 6-block scratch breakdown
    let six = load_six();
    let per2 = make_periodic("periodic-150k-d2", 75_000, 2);
    let per3 = make_periodic("periodic-105k-d3", 35_000, 3);
    let spiral = make_spiral_200k();

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kdf = cand_neon_diet::Kernel::default();
    let mut kbf = cand_neon_bi::Kernel::default();

    println!("\n=== 2. 6-BLOCK SCRATCH & PER-BLOCK ACTIVE L1D DICT ===");
    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        let cd = kdf.run_entry(&b.records, b.n, b.p, cand_neon_diet::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        let cb = kbf.run_entry(&b.records, b.n, b.p, cand_neon_bi::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        assert_eq!(cd.retained, cb.retained);
        let (_, dm, _) = km1.scratch_report();
        let (_, db, _) = kbf.scratch_report();
        println!("{:14} | m={:6} | words={:4} | Dict L1D: Master={:5.1} KiB -> Neon={:4.1} KiB (<32 KiB: {})",
            b.name, cb.retained, cb.words, dm as f64 / 1024.0, db as f64 / 1024.0, db < 32_768);
    }
    let (sm, _, sm_str) = km1.scratch_report();
    let (sd, _, sd_str) = kdf.scratch_report();
    let (sb, _, sb_str) = kbf.scratch_report();
    println!("SCRATCH Master    : {:6.1} KiB ({sm} B) | {sm_str}", sm as f64 / 1024.0);
    println!("SCRATCH Neon-Diet : {:6.1} KiB ({sd} B, {:+.2}%) | {sd_str}", sd as f64 / 1024.0, (sd as f64 / sm as f64 - 1.0) * 100.0);
    println!("SCRATCH Neon-Bi   : {:6.1} KiB ({sb} B, {:+.2}%) | {sb_str}", sb as f64 / 1024.0, (sb as f64 / sm as f64 - 1.0) * 100.0);

    // Check stress blocks parity
    for b in [&dense300, &per2, &per3, &spiral] {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        let c = kbf.run_entry(&b.records, b.n, b.p, cand_neon_bi::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kdf.run_entry(&b.records, b.n, b.p, cand_neon_diet::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        if b.name == "spiral-200k" { assert!(c.spirals, "spiral-200k must exercise spiral contraction"); }
    }

    println!("\n=== 3. HIGH-PRECISION 1T A/A & A/B + STAGES (80 rounds/block + stress corpus) ===");
    let names = ["MasterA1", "MasterA2", "Neon-Diet", "Neon-Bi"];
    let mut sum1t = [0.0f64; 4];
    let mut csum = [0.0f64; 4];
    let mut wsum = [0.0f64; 4];
    for b in &six {
        let mut s: [Vec<f64>; 4] = Default::default();
        for i in 0..4 * 80 {
            let m = (i + i / 4) % 4;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                2 => { kdf.run_entry(&b.records, b.n, b.p, cand_neon_diet::Cap::NONE, &mut out); }
                _ => { kbf.run_entry(&b.records, b.n, b.p, cand_neon_bi::Cap::NONE, &mut out); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let med: [f64; 4] = std::array::from_fn(|k| pct(&mut s[k], 1, 2));
        for k in 0..4 { sum1t[k] += med[k]; }

        let mut sc: [Vec<f64>; 4] = Default::default();
        let mut sw: [Vec<f64>; 4] = Default::default();
        for i in 0..4 * 60 {
            let m = (i + i / 4) % 4;
            let (c, w) = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out),
                1 => km2.run_stages(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out),
                2 => kdf.run_stages(&b.records, b.n, b.p, cand_neon_diet::Cap::NONE, &mut out),
                _ => kbf.run_stages(&b.records, b.n, b.p, cand_neon_bi::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        let mc: [f64; 4] = std::array::from_fn(|k| pct(&mut sc[k], 1, 2));
        let mw: [f64; 4] = std::array::from_fn(|k| pct(&mut sw[k], 1, 2));
        for k in 0..4 { csum[k] += mc[k]; wsum[k] += mw[k]; }
        println!("{:14} | M1={:6.1} | A/A={:.4}x | Diet={:.4}x (c={:.0} w={:.0}) | Bi={:.4}x (c={:.0} w={:.0})",
            b.name, med[0], med[1]/med[0], med[2]/med[0], mc[2], mw[2], med[3]/med[0], mc[3], mw[3]);
    }
    for k in 0..4 {
        println!("1T_SUM {:10} | {:8.1} us ({:.4}x M1) | c={:6.1} ({:.4}x) w={:6.1} ({:.4}x)",
            names[k], sum1t[k], sum1t[k] / sum1t[0], csum[k], csum[k] / csum[0], wsum[k], wsum[k] / wsum[0]);
    }

    for b in [&dense300, &per2, &per3, &spiral] {
        let mut s: [Vec<f64>; 4] = Default::default();
        for i in 0..4 * 40 {
            let m = (i + i / 4) % 4;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                2 => { kdf.run_entry(&b.records, b.n, b.p, cand_neon_diet::Cap::NONE, &mut out); }
                _ => { kbf.run_entry(&b.records, b.n, b.p, cand_neon_bi::Cap::NONE, &mut out); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let med: [f64; 4] = std::array::from_fn(|k| pct(&mut s[k], 1, 2));
        println!("STRESS {:16} | M1={:6.1} | A/A={:.4}x | Diet={:.4}x | Bi={:.4}x",
            b.name, med[0], med[1]/med[0], med[2]/med[0], med[3]/med[0]);
    }

    println!("\n=== 4. HIGH-PRECISION 2T (50 passes) & 4T (30 passes) CONCURRENT BARRIERS ===");
    bench_mt(&six, 2, 50);
    bench_mt(&six, 4, 30);
}
