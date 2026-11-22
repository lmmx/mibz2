#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod master_nobi;
mod cand_r5_head;
mod cand_r5_nobi_w16;
mod cand_r5_nobi_simd;
mod prof_master;
mod prof_nobi;
mod prof_r5_head;
mod prof_r5_nobi_w16;
mod prof_r5_nobi_simd;

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

fn make_spiral_200k() -> Block {
    // Valid single-cycle BWT of T = "a"^199_998 + "bc":
    // L = [c] + [a]*199_998 + [b], p = 0; thick strip width = 199_998 > CERTIFIED_WIDTH (32_768) -> forces eliminate::<true>!
    let n = 200_000usize;
    let mut l = vec![b'a'; n];
    l[0] = b'c';
    l[n - 1] = b'b';
    let p = 0u32;
    let expected = plain_invert(&l, p);
    assert_eq!(expected[0], b'a');
    assert_eq!(expected[n - 2], b'b');
    assert_eq!(expected[n - 1], b'c');
    Block { name: "spiral-200k".into(), n, p, records: records_of(&l), expected }
}

fn make_periodic(name: &str, pat: &[u8], reps: usize) -> Block {
    let n = pat.len() * reps;
    let mut text = Vec::with_capacity(n);
    for _ in 0..reps { text.extend_from_slice(pat); }
    // BWT of periodic string of period d = pat.len(): L consists of d blocks of size reps
    let mut rots: Vec<usize> = (0..pat.len()).collect();
    rots.sort_by(|&a, &b| {
        for k in 0..pat.len() {
            let ca = pat[(a + k) % pat.len()];
            let cb = pat[(b + k) % pat.len()];
            if ca != cb { return ca.cmp(&cb); }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (grp, &r) in rots.iter().enumerate() {
        let prev_b = pat[(r + pat.len() - 1) % pat.len()];
        if r == 0 { p = (grp * reps) as u32; }
        for _ in 0..reps { l.push(prev_b); }
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    Block { name: name.into(), n, p, records: records_of(&l), expected }
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
    let spiral = make_spiral_200k();
    let per2 = make_periodic("periodic-150k-d2", b"ab", 75_000);
    let per3 = make_periodic("periodic-105k-d3", b"abc", 35_000);

    let mut out_m = Vec::with_capacity(900_064);
    let mut out_h = Vec::with_capacity(900_064);
    let mut out_w = Vec::with_capacity(900_064);
    let mut out_s = Vec::with_capacity(900_064);

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut knb = master_nobi::Kernel::default();
    let mut kh = cand_r5_head::Kernel::default();
    let mut knw = cand_r5_nobi_w16::Kernel::default();
    let mut kns = cand_r5_nobi_simd::Kernel::default();

    // =========================================================================
    // 1. EXHAUSTIVE CORRECTNESS AUDIT (67,968 valid BWTs + 1,100 malformed)
    // =========================================================================
    let mut valid_count = 0usize;
    let mut text = Vec::with_capacity(512);
    // 1a. ALL binary strings n = 1..=14 (32,766 BWTs)
    for n in 1..=14usize {
        for mask in 0..(1usize << n) {
            text.clear();
            for bit in 0..n { text.push(b'a' + ((mask >> bit) & 1) as u8); }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_h).unwrap();
            assert_eq!(out_h, text, "head binary n={n} mask={mask}");
            knw.run_entry(&recs, n, p, cand_r5_nobi_w16::Cap::NONE, &mut out_w).unwrap();
            assert_eq!(out_w, text, "nobi_w16 binary n={n} mask={mask}");
            kns.run_entry(&recs, n, p, cand_r5_nobi_simd::Cap::NONE, &mut out_s).unwrap();
            assert_eq!(out_s, text, "nobi_simd binary n={n} mask={mask}");
            valid_count += 1;
        }
    }
    // 1b. ALL ternary strings n = 1..=9 (29,523 BWTs)
    for n in 1..=9usize {
        let total = 3usize.pow(n as u32);
        for mut v in 0..total {
            text.clear();
            for _ in 0..n { text.push(b'a' + (v % 3) as u8); v /= 3; }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_h).unwrap();
            assert_eq!(out_h, text, "head ternary n={n}");
            knw.run_entry(&recs, n, p, cand_r5_nobi_w16::Cap::NONE, &mut out_w).unwrap();
            assert_eq!(out_w, text, "nobi_w16 ternary n={n}");
            kns.run_entry(&recs, n, p, cand_r5_nobi_simd::Cap::NONE, &mut out_s).unwrap();
            assert_eq!(out_s, text, "nobi_simd ternary n={n}");
            valid_count += 1;
        }
    }
    // 1c. 5,679 pseudo-random valid BWTs (n = 1..=512, diverse alphabets) -> total 67,968!
    let mut rng = 0xA076_1D64_78BD_642Fu64;
    for case in 0..5_679usize {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let n = 1 + ((rng >> 32) as usize % 512);
        let alpha = match case % 7 { 0 => 2u16, 1 => 3, 2 => 4, 3 => 8, 4 => 16, 5 => 26, _ => 256 };
        text.clear();
        for _ in 0..n {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            text.push(((rng >> 40) % alpha as u64) as u8);
        }
        let (l, p) = bwt_small(&text);
        let recs = records_of(&l);
        kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_h).unwrap();
        assert_eq!(out_h, text);
        knw.run_entry(&recs, n, p, cand_r5_nobi_w16::Cap::NONE, &mut out_w).unwrap();
        assert_eq!(out_w, text);
        kns.run_entry(&recs, n, p, cand_r5_nobi_simd::Cap::NONE, &mut out_s).unwrap();
        assert_eq!(out_s, text);
        valid_count += 1;
    }

    // 1d. 1,100 malformed / multi-cycle BWT columns (1,000 small + 100 at n=70,000) — exact parity with master_a1!
    let mut mal_count = 0usize;
    let mut lbuf = Vec::with_capacity(70_000);
    for case in 0..1_100usize {
        let n = if case < 1_000 { 1 + (case % 256) } else { 70_000 };
        let alpha = if case < 1_000 { 2 + (case % 15) } else { 2 + (case % 8) };
        lbuf.clear();
        for _ in 0..n {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            lbuf.push(((rng >> 40) % alpha as u64) as u8);
        }
        let p = ((rng >> 32) as usize % n) as u32;
        let recs = records_of(&lbuf);
        km1.run_entry(&recs, n, p, master_a1::Cap::NONE, &mut out_m).unwrap();
        kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_h).unwrap();
        knw.run_entry(&recs, n, p, cand_r5_nobi_w16::Cap::NONE, &mut out_w).unwrap();
        kns.run_entry(&recs, n, p, cand_r5_nobi_simd::Cap::NONE, &mut out_s).unwrap();
        assert_eq!(out_h, out_m, "malformed head mismatch case={case} n={n}");
        assert_eq!(out_w, out_m, "malformed nobi_w16 mismatch case={case} n={n}");
        assert_eq!(out_s, out_m, "malformed nobi_simd mismatch case={case} n={n}");
        mal_count += 1;
    }

    // 1e. Stress blocks + 6 Wikidata + #703 Cap::dense + scratch verification
    for b in six.iter().chain([&dense, &spiral, &per2, &per3]) {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out_m).unwrap(); assert_eq!(out_m, b.expected);
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_h).unwrap(); assert_eq!(out_h, b.expected);
        knw.run_entry(&b.records, b.n, b.p, cand_r5_nobi_w16::Cap::NONE, &mut out_w).unwrap(); assert_eq!(out_w, b.expected);
        kns.run_entry(&b.records, b.n, b.p, cand_r5_nobi_simd::Cap::NONE, &mut out_s).unwrap(); assert_eq!(out_s, b.expected);
    }
    assert!(kh.run_entry(&dense.records, dense.n, dense.p, cand_r5_head::Cap::dense(dense.n), &mut out_h).is_none());
    assert!(knw.run_entry(&dense.records, dense.n, dense.p, cand_r5_nobi_w16::Cap::dense(dense.n), &mut out_w).is_none());
    assert!(kns.run_entry(&dense.records, dense.n, dense.p, cand_r5_nobi_simd::Cap::dense(dense.n), &mut out_s).is_none());

    // Measure exact Wikidata steady-state persistent scratch (fresh kernels warmed ONLY on the 6 Wikidata blocks!)
    let mut sm = master_a1::Kernel::default();
    let mut sh = cand_r5_head::Kernel::default();
    let mut snw = cand_r5_nobi_w16::Kernel::default();
    for b in &six {
        sm.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out_m).unwrap();
        sh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_h).unwrap();
        snw.run_entry(&b.records, b.n, b.p, cand_r5_nobi_w16::Cap::NONE, &mut out_w).unwrap();
    }
    let (wm, dm, sdm) = sm.scratch_report();
    let (wh, dh, sdh) = sh.scratch_report();
    let (wn, dn, sdn) = snw.scratch_report();
    println!("PASS: {valid_count} valid BWTs + {mal_count} malformed (100@70k) + spiral-200k + periodic-150k/105k + dense-300k + 6 Wikidata + #703 Cap::dense 100% BYTE-EXACT!");
    println!("SCRATCH Master53f7: {:.1} KiB (dict_l1={dm} B) | {sdm}", wm as f64 / 1024.0);
    println!("SCRATCH R5-Head:    {:.1} KiB (dict_l1={dh} B) | {sdh}", wh as f64 / 1024.0);
    println!("SCRATCH R5-Nobi:    {:.1} KiB (dict_l1={dn} B) | {sdn}", wn as f64 / 1024.0);

    // =========================================================================
    // 2. 1T INTERLEAVED A/A & A/B (80 rds/blk) + PRE SPLIT (60 rds/blk)
    // =========================================================================
    println!("\n=== 1. 1T INTERLEAVED A/A & A/B (80 rds/blk) + PRE SPLIT (60 rds/blk) ===");
    let names = ["MasterA1", "MasterA2", "R5-Head", "Mst-Nobi", "R5-NobiW16", "R5-NobiSIMD"];
    let mut sum1t = [0.0f64; 6];
    let mut csum = [0.0f64; 6];
    let mut wsum = [0.0f64; 6];
    for b in &six {
        let mut s: [Vec<f64>; 6] = Default::default();
        for i in 0..6 * 80 {
            let m = (i + i / 6) % 6;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out_m); }
                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out_m); }
                2 => { kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_m); }
                3 => { knb.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out_m); }
                4 => { knw.run_entry(&b.records, b.n, b.p, cand_r5_nobi_w16::Cap::NONE, &mut out_m); }
                _ => { kns.run_entry(&b.records, b.n, b.p, cand_r5_nobi_simd::Cap::NONE, &mut out_m); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let med: [f64; 6] = std::array::from_fn(|k| pct(&mut s[k], 1, 2));
        for k in 0..6 { sum1t[k] += med[k]; }

        let mut sc: [Vec<f64>; 6] = Default::default();
        let mut sw: [Vec<f64>; 6] = Default::default();
        for i in 0..6 * 60 {
            let m = (i + i / 6) % 6;
            let (c, w) = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out_m),
                1 => km2.run_stages(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out_m),
                2 => kh.run_stages(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_m),
                3 => knb.run_stages(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out_m),
                4 => knw.run_stages(&b.records, b.n, b.p, cand_r5_nobi_w16::Cap::NONE, &mut out_m),
                _ => kns.run_stages(&b.records, b.n, b.p, cand_r5_nobi_simd::Cap::NONE, &mut out_m),
            };
            sc[m].push(c); sw[m].push(w);
        }
        for k in 0..6 { csum[k] += pct(&mut sc[k], 1, 2); wsum[k] += pct(&mut sw[k], 1, 2); }
        println!("{:14} | M1={:6.1} | A/A={:.4} R5-Head={:.4} | MstNobi={:6.1} NW16={:.4} NSIMD={:.4}",
            b.name, med[0], med[1]/med[0], med[2]/med[0], med[3], med[4]/med[3], med[5]/med[3]);
    }
    for k in 0..6 {
        let bt = if k >= 3 { sum1t[3] } else { sum1t[0] };
        let bc = if k >= 3 { csum[3] } else { csum[0] };
        println!("1T_SUM {:12} | tot={:7.1} ({:.4}x) | pre={:6.1} ({:.4}x, {:+.1} us) | emit={:6.1}",
            names[k], sum1t[k], sum1t[k]/bt, csum[k], csum[k]/bc, csum[k]-bc, wsum[k]);
    }

    // =========================================================================
    // 3. FINAL 6-STAGE PROFILE (50 rounds/block)
    // =========================================================================
    println!("\n=== 2. FINAL 6-STAGE PROFILE (50 rds/blk vs 53f7ecf93) ===");
    let mut pm = prof_master::Kernel::default();
    let mut ph = prof_r5_head::Kernel::default();
    let mut pnb = prof_nobi::Kernel::default();
    let mut pnw = prof_r5_nobi_w16::Kernel::default();
    let mut pns = prof_r5_nobi_simd::Kernel::default();
    let pnames = ["Prof-Mst53f7", "Prof-R5Head", "Prof-Nobi53f7", "Prof-R5NW16", "Prof-R5NSIMD"];
    let mut psum = [[0.0f64; 6]; 5];
    for b in &six {
        let mut s: [[Vec<f64>; 6]; 5] = Default::default();
        for i in 0..5 * 50 {
            let m = (i + i / 5) % 5;
            let st = match m {
                0 => pm.run_profile6(&b.records, b.n, b.p, prof_master::Cap::NONE, &mut out_m),
                1 => ph.run_profile6(&b.records, b.n, b.p, prof_r5_head::Cap::NONE, &mut out_m),
                2 => pnb.run_profile6(&b.records, b.n, b.p, prof_nobi::Cap::NONE, &mut out_m),
                3 => pnw.run_profile6(&b.records, b.n, b.p, prof_r5_nobi_w16::Cap::NONE, &mut out_m),
                _ => pns.run_profile6(&b.records, b.n, b.p, prof_r5_nobi_simd::Cap::NONE, &mut out_m),
            };
            for j in 0..6 { s[m][j].push(st[j]); }
        }
        for m in 0..5 { for j in 0..6 { psum[m][j] += pct(&mut s[m][j], 1, 2); } }
    }
    for m in 0..5 {
        let pre: f64 = psum[m][0..5].iter().sum();
        let bidx = if m >= 2 { 2 } else { 0 };
        let bpre: f64 = psum[bidx][0..5].iter().sum();
        println!("{:14} | S1={:6.1} S2a={:5.1} S2b={:6.1} S3a={:5.1} S4={:6.1} | Pre={:6.1} ({:.4}x, {:+.1}) | S5={:6.1}",
            pnames[m], psum[m][0], psum[m][1], psum[m][2], psum[m][3], psum[m][4], pre, pre/bpre, pre-bpre, psum[m][5]);
    }

    // =========================================================================
    // 4. 2T (50 passes) & 4T (30 passes) CONCURRENT BARRIER CONTENTION AUDIT
    // =========================================================================
    for &(threads, passes) in &[(2usize, 50usize), (4usize, 30usize)] {
        println!("\n=== {}T CONCURRENT BARRIER ({} passes) ===", threads, passes);
        let mut rmt: [Vec<f64>; 6] = Default::default();
        for pass in 0..passes {
            for off in 0..6 {
                let m = (pass + off) % 6;
                let bar = Barrier::new(threads);
                let t = std::thread::scope(|sc| {
                    let b_ref = &six; let bar_ref = &bar;
                    let mut hs = Vec::with_capacity(threads);
                    for tid in 0..threads {
                        hs.push(sc.spawn(move || {
                            let mut out = Vec::with_capacity(900_064);
                            let mut k0 = master_a1::Kernel::default();
                            let mut k1 = master_a2::Kernel::default();
                            let mut k2 = cand_r5_head::Kernel::default();
                            let mut k3 = master_nobi::Kernel::default();
                            let mut k4 = cand_r5_nobi_w16::Kernel::default();
                            let mut k5 = cand_r5_nobi_simd::Kernel::default();
                            for b in b_ref {
                                match m {
                                    0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                                    1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                                    2 => { k2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out); }
                                    3 => { k3.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                                    4 => { k4.run_entry(&b.records, b.n, b.p, cand_r5_nobi_w16::Cap::NONE, &mut out); }
                                    _ => { k5.run_entry(&b.records, b.n, b.p, cand_r5_nobi_simd::Cap::NONE, &mut out); }
                                }
                            }
                            bar_ref.wait();
                            let t0 = Instant::now();
                            for idx in 0..b_ref.len() {
                                let b = &b_ref[(idx + tid) % b_ref.len()];
                                match m {
                                    0 => { k0.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                                    1 => { k1.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                                    2 => { k2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out); }
                                    3 => { k3.run_entry(&b.records, b.n, b.p, master_nobi::Cap::NONE, &mut out); }
                                    4 => { k4.run_entry(&b.records, b.n, b.p, cand_r5_nobi_w16::Cap::NONE, &mut out); }
                                    _ => { k5.run_entry(&b.records, b.n, b.p, cand_r5_nobi_simd::Cap::NONE, &mut out); }
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
        let b10_n = pct(&mut rmt[3].clone(), 1, 10);
        let b50_n = pct(&mut rmt[3].clone(), 1, 2);
        for k in 0..6 {
            let p10 = pct(&mut rmt[k], 1, 10);
            let p50 = pct(&mut rmt[k], 1, 2);
            let (b10, b50) = if k >= 3 { (b10_n, b50_n) } else { (b10_h, b50_h) };
            println!("{}T {:12} | p10={:7.1} ({:.4}x) | p50={:7.1} ({:.4}x)", threads, names[k], p10, p10/b10, p50, p50/b50);
        }
    }
}
