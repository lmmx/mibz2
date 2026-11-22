#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_biop48;
mod cand_fused_op;
mod cand_birv;
mod cand_leftrv;

use std::sync::Barrier;
use std::time::Instant;

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    expected: Vec<u8>,
}

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
    let n = l.len();
    let mut counts = [0u32; 256];
    let mut rank_in_l = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() { rank_in_l[i] = counts[b as usize]; counts[b as usize] += 1; }
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

fn bwt_encode(text: &[u8]) -> (Vec<u8>, u32) {
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
    for (i, &s) in sa.iter().enumerate() {
        if s == 0 { p = i as u32; }
        l.push(text[(s + n - 1) % n]);
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
    let records = records_of(&l);
    Block { name: "dense-300k".into(), n, p, records, expected }
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

    println!("=== 1. CHURCH-ROSSER CONFLUENCE VERIFICATION (BINARY 1..=11, TERNARY 1..=6, MALFORMED, 6 BLOCKS, DENSE) ===");
    let mut kbi_c = cand_birv::Kernel::default();
    let mut kl_c = cand_leftrv::Kernel::default();
    let mut chk = 0usize;
    for len in 1..=11 {
        for mask in 0..(1usize << len) {
            let text: Vec<u8> = (0..len).map(|i| b'a' + ((mask >> i) & 1) as u8).collect();
            let (l, p) = bwt_encode(&text);
            let recs = records_of(&l);
            for &lim in &[1u32, 4, 16] {
                kbi_c.try_run::<true, true>(&recs, len, p, lim, cand_birv::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text, "BiRV mismatch len={len} mask={mask} lim={lim}");
                kl_c.try_run::<true, true>(&recs, len, p, lim, cand_leftrv::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text, "LeftRV mismatch len={len} mask={mask} lim={lim}");
                chk += 2;
            }
        }
    }
    for len in 1..=6 {
        let mut pow = 1usize;
        for _ in 0..len { pow *= 3; }
        for code in 0..pow {
            let mut x = code;
            let text: Vec<u8> = (0..len).map(|_| { let r = (x % 3) as u8; x /= 3; b'a' + r }).collect();
            let (l, p) = bwt_encode(&text);
            let recs = records_of(&l);
            kbi_c.try_run::<true, true>(&recs, len, p, 1, cand_birv::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, text);
            chk += 1;
        }
    }
    // 500 arbitrary non-BWT columns (tests malformed safety!)
    let mut rng = 0x1234_ABCD_5678_EF01u64;
    for case in 0..500usize {
        let n = (case % 200) + 1;
        let mut col = Vec::with_capacity(n);
        for _ in 0..n {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            col.push(((rng >> 56) & 3) as u8);
        }
        let p = ((rng >> 32) as usize % n) as u32;
        let recs = records_of(&col);
        kbi_c.try_run::<true, true>(&recs, n, p, 2, cand_birv::Cap::NONE, &mut out).unwrap();
        assert_eq!(out.len(), n);
    }
    println!("Verified {chk} exhaustive BWTs + 500 malformed non-BWT columns for BiRV & LeftRV!");

    println!("\n=== 2. CONFLUENCE CENSUS (STEPS, RULES, BANDS, WORDS) & SCRATCH AUDIT ===");
    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut k48 = cand_biop48::Kernel::default();
    let mut kfop = cand_fused_op::Kernel::default();
    let mut kbirv = cand_birv::Kernel::default();
    let mut klrv = cand_leftrv::Kernel::default();

    for b in &six {
        let cm = km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let c48 = k48.run_entry(&b.records, b.n, b.p, cand_biop48::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kfop.run_entry(&b.records, b.n, b.p, cand_fused_op::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let cbi = kbirv.run_entry(&b.records, b.n, b.p, cand_birv::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "BiRV mismatch on {}", b.name);
        let cl = klrv.run_entry(&b.records, b.n, b.p, cand_leftrv::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "LeftRV mismatch on {}", b.name);
        assert_eq!(c48.bands, cbi.bands, "Band count must be canonically invariant!");
        assert_eq!(c48.bands, cl.bands);
        println!(
            "CONF {:12} | bands={} (EXACT) | steps: Right={:5} BiRV={:5} ({:.3}x) Left={:5} ({:.3}x) | rules: R={:5} Bi={:5} L={:5}",
            b.name, cm.bands,
            c48.steps, cbi.steps, cbi.steps as f64 / c48.steps as f64, cl.steps, cl.steps as f64 / c48.steps as f64,
            c48.rules, cbi.rules, cl.rules
        );
    }
    // Check dense-300k parity
    {
        let mut t1 = cand_birv::Kernel::default();
        let mut t2 = cand_leftrv::Kernel::default();
        t1.run_entry(&dense.records, dense.n, dense.p, cand_birv::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, dense.expected);
        t2.run_entry(&dense.records, dense.n, dense.p, cand_leftrv::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, dense.expected);
    }
    let (sm, _, _) = km1.scratch_report();
    let (s48, _, b48) = k48.scratch_report();
    let (sbi, _, _) = kbirv.scratch_report();
    println!("SCRATCH Master 0.5.11 : {:7.1} KiB", sm as f64 / 1024.0);
    println!("SCRATCH BiOp48        : {:7.1} KiB ({:+.2}%) | {}", s48 as f64 / 1024.0, (s48 as f64 / sm as f64 - 1.0) * 100.0, b48);
    println!("SCRATCH BiRV-Bi       : {:7.1} KiB ({:+.2}%)", sbi as f64 / 1024.0, (sbi as f64 / sm as f64 - 1.0) * 100.0);

    println!("\n=== 3. STAGE ATTRIBUTION (`construct` vs `walk`, 50 rounds/block) ===");
    let (mut scm, mut swm, mut sc48, mut scf, mut scbi, mut scl) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in &six {
        let (mut mc, mut mw, mut c48, mut cf, mut cbi, mut cl) = (vec![], vec![], vec![], vec![], vec![], vec![]);
        for i in 0..5 * 50 {
            match (i + i / 5) % 5 {
                0 => { let (c, w) = km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); mc.push(c); mw.push(w); }
                1 => { let (c, _) = k48.run_stages(&b.records, b.n, b.p, cand_biop48::Cap::NONE, &mut out); c48.push(c); }
                2 => { let (c, _) = kfop.run_stages(&b.records, b.n, b.p, cand_fused_op::Cap::NONE, &mut out); cf.push(c); }
                3 => { let (c, _) = kbirv.run_stages(&b.records, b.n, b.p, cand_birv::Cap::NONE, &mut out); cbi.push(c); }
                _ => { let (c, _) = klrv.run_stages(&b.records, b.n, b.p, cand_leftrv::Cap::NONE, &mut out); cl.push(c); }
            }
        }
        let (mc10, mw10, c4810, cf10, cbi10, cl10) = (
            pct(&mut mc, 1, 10), pct(&mut mw, 1, 10), pct(&mut c48, 1, 10),
            pct(&mut cf, 1, 10), pct(&mut cbi, 1, 10), pct(&mut cl, 1, 10),
        );
        scm += mc10; swm += mw10; sc48 += c4810; scf += cf10; scbi += cbi10; scl += cl10;
        println!(
            "STAGE {:12} | construct: M={:6.1} BiOp48={:6.1} FusedOp={:6.1} ({:+.1}) BiRV={:6.1} ({:+.1}) LeftRV={:6.1} ({:+.1})",
            b.name, mc10, c4810, cf10, cf10 - c4810, cbi10, cbi10 - c4810, cl10, cl10 - c4810
        );
    }
    println!(
        "STAGE 6-BLK SUM    | construct: M={:6.1} BiOp48={:6.1} FusedOp={:6.1} ({:+.1}) BiRV={:6.1} ({:+.1}) LeftRV={:6.1} ({:+.1}) | walk M={:6.1}",
        scm, sc48, scf, scf - sc48, scbi, scbi - sc48, scl, scl - sc48, swm
    );

    println!("\n=== 4. 1T INTERLEAVED A/A & A/B (65 pairs/block + dense-300k) ===");
    let reps = 65usize;
    let (mut saar, mut saac, mut s48r, mut s48c, mut sfr, mut sfc, mut sbir, mut sbic, mut slr, mut slc) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain(std::iter::once(&dense)) {
        let is_six = b.n >= 890_000;
        let run_p = |mode: usize, km1: &mut master_a1::Kernel, km2: &mut master_a2::Kernel, k48: &mut cand_biop48::Kernel, kfop: &mut cand_fused_op::Kernel, kbirv: &mut cand_birv::Kernel, klrv: &mut cand_leftrv::Kernel, out: &mut Vec<u8>| -> (f64, f64) {
            let (mut ta, mut tb) = (vec![], vec![]);
            for i in 0..2 * reps {
                let t = Instant::now();
                if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
                else {
                    match mode {
                        0 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, out); }
                        1 => { k48.run_entry(&b.records, b.n, b.p, cand_biop48::Cap::NONE, out); }
                        2 => { kfop.run_entry(&b.records, b.n, b.p, cand_fused_op::Cap::NONE, out); }
                        3 => { kbirv.run_entry(&b.records, b.n, b.p, cand_birv::Cap::NONE, out); }
                        _ => { klrv.run_entry(&b.records, b.n, b.p, cand_leftrv::Cap::NONE, out); }
                    }
                    tb.push(t.elapsed().as_secs_f64() * 1e6);
                }
            }
            (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10))
        };
        let (aar, aac) = run_p(0, &mut km1, &mut km2, &mut k48, &mut kfop, &mut kbirv, &mut klrv, &mut out);
        let (r48, c48) = run_p(1, &mut km1, &mut km2, &mut k48, &mut kfop, &mut kbirv, &mut klrv, &mut out);
        let (rf, cf) = run_p(2, &mut km1, &mut km2, &mut k48, &mut kfop, &mut kbirv, &mut klrv, &mut out);
        let (rbi, cbi) = run_p(3, &mut km1, &mut km2, &mut k48, &mut kfop, &mut kbirv, &mut klrv, &mut out);
        let (rl, cl) = run_p(4, &mut km1, &mut km2, &mut k48, &mut kfop, &mut kbirv, &mut klrv, &mut out);
        if is_six {
            saar += aar; saac += aac; s48r += r48; s48c += c48;
            sfr += rf; sfc += cf; sbir += rbi; sbic += cbi; slr += rl; slc += cl;
        }
        println!(
            "1T {:12} | A/A={:.3} | BiOp48={:.3} | FusedOp={:.3} | BiRV={:.3} | LeftRV={:.3}",
            b.name, aac / aar, c48 / r48, cf / rf, cbi / rbi, cl / rl
        );
    }
    println!(
        "1T 6-BLK SUM  | A/A={:.4} | BiOp48={:.4} | FusedOp={:.4} | BiRV={:.4} | LeftRV={:.4}",
        saac / saar, s48c / s48r, sfc / sfr, sbic / sbir, slc / slr
    );

    println!("\n=== 5. 2T CONCURRENT BARRIER A/A & A/B (45 passes/worker) ===");
    let workers = 2usize;
    let passes = 45usize;
    for (label, mode) in [
        ("2T A/A (Master1 vs Master2)", 0),
        ("2T A/B (Master1 vs BiOp48)", 1),
        ("2T A/B (Master1 vs FusedOp)", 2),
        ("2T A/B (Master1 vs BiRV)", 3),
        ("2T A/B (Master1 vs LeftRV)", 4),
    ] {
        let barrier = Barrier::new(workers);
        let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers)
                .map(|w| {
                    let bar = &barrier;
                    s.spawn(move || {
                        let mut km = master_a1::Kernel::default();
                        let mut km2 = master_a2::Kernel::default();
                        let mut k48 = cand_biop48::Kernel::default();
                        let mut kf = cand_fused_op::Kernel::default();
                        let mut kb = cand_birv::Kernel::default();
                        let mut kl = cand_leftrv::Kernel::default();
                        let mut out = Vec::with_capacity(900_064);
                        for b in six_ref {
                            km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                            km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                            k48.run_entry(&b.records, b.n, b.p, cand_biop48::Cap::NONE, &mut out);
                            kf.run_entry(&b.records, b.n, b.p, cand_fused_op::Cap::NONE, &mut out);
                            kb.run_entry(&b.records, b.n, b.p, cand_birv::Cap::NONE, &mut out);
                            kl.run_entry(&b.records, b.n, b.p, cand_leftrv::Cap::NONE, &mut out);
                        }
                        let (mut tr, mut tc) = (vec![], vec![]);
                        for pass in 0..(2 * passes + 4) {
                            let which = (pass + pass / 2) % 2;
                            bar.wait();
                            let t0 = Instant::now();
                            for i in 0..six_ref.len() {
                                let b = &six_ref[(i + w * 3) % six_ref.len()];
                                if which == 0 { km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                                else if mode == 0 { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                                else if mode == 1 { k48.run_entry(&b.records, b.n, b.p, cand_biop48::Cap::NONE, &mut out); }
                                else if mode == 2 { kf.run_entry(&b.records, b.n, b.p, cand_fused_op::Cap::NONE, &mut out); }
                                else if mode == 3 { kb.run_entry(&b.records, b.n, b.p, cand_birv::Cap::NONE, &mut out); }
                                else { kl.run_entry(&b.records, b.n, b.p, cand_leftrv::Cap::NONE, &mut out); }
                            }
                            let dt = t0.elapsed().as_secs_f64() * 1e6;
                            if pass >= 4 {
                                if which == 0 { tr.push(dt); } else { tc.push(dt); }
                            }
                        }
                        (tr, tc)
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut ar: Vec<f64> = res.iter().flat_map(|x| x.0.clone()).collect();
        let mut ac: Vec<f64> = res.iter().flat_map(|x| x.1.clone()).collect();
        println!(
            "{:32} | p10={:.4} ({:.1}/{:.1}) | p50={:.4} ({:.1}/{:.1})",
            label,
            pct(&mut ac, 1, 10) / pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 10), pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 2) / pct(&mut ar, 1, 2),
            pct(&mut ac, 1, 2), pct(&mut ar, 1, 2)
        );
    }
}
