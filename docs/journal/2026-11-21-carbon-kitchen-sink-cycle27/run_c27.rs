#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_cks_diet;
mod cand_cks_bi;
mod cand_i15_diet;
mod cand_i15_bi;
mod prof_bi;

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

fn make_periodic_150k() -> Block {
    let p_len = 30_000usize;
    let d = 5usize;
    let mut base = Vec::with_capacity(p_len);
    let mut s = 0xCAFE_BABE_DEAD_BEEFu64;
    for i in 0..p_len {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        let run = ((s >> 58) % 24 + 1) as usize;
        let ch = b'a' + ((i + (s >> 40) as usize) % 8) as u8;
        for _ in 0..run.min(p_len - base.len()) { base.push(ch); }
        if base.len() >= p_len { break; }
    }
    base.truncate(p_len);
    let text = base.repeat(d);
    let n = text.len();
    let mut sa_base: Vec<usize> = (0..p_len).collect();
    sa_base.sort_by(|&a, &b| {
        for k in 0..p_len {
            let ca = base[(a + k) % p_len];
            let cb = base[(b + k) % p_len];
            if ca != cb { return ca.cmp(&cb); }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for &sb in &sa_base {
        for copy in 0..d {
            if copy * p_len + sb == 0 { p = l.len() as u32; }
            l.push(base[(sb + p_len - 1) % p_len]);
        }
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    let records = records_of(&l);
    Block { name: "periodic-150k".into(), n, p, records, expected }
}

fn load_six() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let mut specs = vec![];
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") {
            file = v.trim_end_matches("\",").to_string();
        }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            specs.push((file.clone(), v.trim_end_matches(',').parse::<u32>().unwrap()));
        }
    }
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = std::fs::read(format!("{dir}/{f}")).unwrap();
            let expected = plain_invert(&l, p);
            let records = records_of(&l);
            Block { name: f, n: l.len(), p, records, expected }
        })
        .collect()
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn main() {
    let six = load_six();
    let dense = make_dense_300k();
    let per150 = make_periodic_150k();
    let mut out = Vec::with_capacity(900_064);

    println!("=== 1. EXHAUSTIVE PARITY (BINARY 1..=10, NON-BWT, 6 BLOCKS, DENSE, PERIODIC) ===");
    let mut kd_chk = cand_i15_diet::Kernel::default();
    let mut kb_chk = cand_i15_bi::Kernel::default();
    let mut chk = 0usize;
    for len in 1..=10 {
        for mask in 0..(1usize << len) {
            let text: Vec<u8> = (0..len).map(|i| b'a' + ((mask >> i) & 1) as u8).collect();
            let (l, p) = bwt_encode(&text);
            let recs = records_of(&l);
            for &lim in &[1u32, 4, 16] {
                kd_chk.try_run::<true, true>(&recs, len, p, lim, cand_i15_diet::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                kb_chk.try_run::<true, true>(&recs, len, p, lim, cand_i15_bi::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                chk += 2;
            }
        }
    }
    for sb in [&dense, &per150] {
        kd_chk.run_entry(&sb.records, sb.n, sb.p, cand_i15_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, sb.expected);
        kb_chk.run_entry(&sb.records, sb.n, sb.p, cand_i15_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, sb.expected);
    }
    println!("Verified {chk} exhaustive BWTs + dense-300k + periodic-150k!");

    println!("\n=== 2. 6-PHASE MICROSECOND PROFILE OF CKS-BiOperad & wlen==16 CENSUS ===");
    let mut kprof = prof_bi::Kernel::default();
    for b in &six { kprof.run_entry(&b.records, b.n, b.p, prof_bi::Cap::NONE, &mut out); }
    let mut sum_p = [0.0f64; 6];
    let (mut tot_l16, mut tot_m) = (0usize, 0usize);
    for b in &six {
        let mut samples = [vec![], vec![], vec![], vec![], vec![], vec![]];
        let (mut l16, mut m) = (0, 0);
        for _ in 0..30 {
            let (ph, c16, cm) = kprof.profile_6phases(&b.records, b.n, b.p, &mut out);
            for j in 0..6 { samples[j].push(ph[j]); }
            l16 = c16; m = cm;
        }
        let p10: [f64; 6] = std::array::from_fn(|j| pct(&mut samples[j], 1, 10));
        for j in 0..6 { sum_p[j] += p10[j]; }
        tot_l16 += l16; tot_m += m;
        println!(
            "PROF {:12} | pass1={:5.1} finish={:4.1} elim={:5.1} dedup={:5.1} wire_xor={:5.1} walk={:5.1} | wlen16={:4}/{:6} ({:.2}%)",
            b.name, p10[0], p10[1], p10[2], p10[3], p10[4], p10[5], l16, m, 100.0 * l16 as f64 / m as f64
        );
    }
    let tot_t: f64 = sum_p.iter().sum();
    println!(
        "PROF 6-BLK SUM    | pass1={:6.1} ({:4.1}%) finish={:5.1} ({:3.1}%) elim={:6.1} ({:4.1}%) dedup={:6.1} ({:4.1}%) wire_xor={:6.1} ({:4.1}%) walk={:6.1} ({:4.1}%) | wlen16={}/{} ({:.3}%)",
        sum_p[0], 100.0 * sum_p[0] / tot_t,
        sum_p[1], 100.0 * sum_p[1] / tot_t,
        sum_p[2], 100.0 * sum_p[2] / tot_t,
        sum_p[3], 100.0 * sum_p[3] / tot_t,
        sum_p[4], 100.0 * sum_p[4] / tot_t,
        sum_p[5], 100.0 * sum_p[5] / tot_t,
        tot_l16, tot_m, 100.0 * tot_l16 as f64 / tot_m as f64
    );

    println!("\n=== 3. CUMULATIVE 6-BLOCK WORKER SCRATCH AUDIT ===");
    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kdiet = cand_cks_diet::Kernel::default();
    let mut kbi = cand_cks_bi::Kernel::default();
    let mut ki15d = cand_i15_diet::Kernel::default();
    let mut ki15b = cand_i15_bi::Kernel::default();
    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kdiet.run_entry(&b.records, b.n, b.p, cand_cks_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kbi.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        ki15d.run_entry(&b.records, b.n, b.p, cand_i15_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "Inline15-Diet mismatch on {}", b.name);
        ki15b.run_entry(&b.records, b.n, b.p, cand_i15_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "Inline15-Bi mismatch on {}", b.name);
    }
    let (sm, _, bm) = km1.scratch_report();
    let (sbi, _, bbi) = kbi.scratch_report();
    let (si15, _, bi15) = ki15b.scratch_report();
    println!("SCRATCH Master 0.5.11       : {:7.1} KiB | {}", sm as f64 / 1024.0, bm);
    println!("SCRATCH CKS-BiOperad (C25)  : {:7.1} KiB ({:+.2}%) | {}", sbi as f64 / 1024.0, (sbi as f64 / sm as f64 - 1.0) * 100.0, bbi);
    println!("SCRATCH Inline15-ZeroLens   : {:7.1} KiB ({:+.2}%) | {}", si15 as f64 / 1024.0, (si15 as f64 / sm as f64 - 1.0) * 100.0, bi15);

    println!("\n=== 4. STAGE ATTRIBUTION (`construct` vs `walk`, 50 rounds/block) ===");
    let (mut scm, mut swm, mut scb, mut swb, mut sci, mut swi) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in &six {
        let (mut mc, mut mw, mut bc, mut bw, mut ic, mut iw) = (vec![], vec![], vec![], vec![], vec![], vec![]);
        for i in 0..3 * 50 {
            match (i + i / 3) % 3 {
                0 => { let (c, w) = km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); mc.push(c); mw.push(w); }
                1 => { let (c, w) = kbi.run_stages(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out); bc.push(c); bw.push(w); }
                _ => { let (c, w) = ki15b.run_stages(&b.records, b.n, b.p, cand_i15_bi::Cap::NONE, &mut out); ic.push(c); iw.push(w); }
            }
        }
        let (mc10, mw10) = (pct(&mut mc, 1, 10), pct(&mut mw, 1, 10));
        let (bc10, bw10) = (pct(&mut bc, 1, 10), pct(&mut bw, 1, 10));
        let (ic10, iw10) = (pct(&mut ic, 1, 10), pct(&mut iw, 1, 10));
        scm += mc10; swm += mw10; scb += bc10; swb += bw10; sci += ic10; swi += iw10;
        println!(
            "STAGE {:12} | construct: M={:6.1} Bi={:6.1} I15-Bi={:6.1} ({:+.1}) | walk: M={:6.1} Bi={:6.1} I15-Bi={:6.1} ({:+.1}, {:.3}x M)",
            b.name, mc10, bc10, ic10, ic10 - bc10, mw10, bw10, iw10, iw10 - bw10, iw10 / mw10
        );
    }
    println!(
        "STAGE 6-BLK SUM    | construct: M={:6.1} Bi={:6.1} I15-Bi={:6.1} ({:+.1}) | walk: M={:6.1} Bi={:6.1} ({:.4}x) I15-Bi={:6.1} ({:.4}x, {:+.1} vs Bi)",
        scm, scb, sci, sci - scb, swm, swb, swb / swm, swi, swi / swm, swi - swb
    );

    println!("\n=== 5. 1T INTERLEAVED A/A & A/B (70 pairs/block + dense-300k + periodic-150k) ===");
    let reps = 70usize;
    let (mut saar, mut saac, mut sdr, mut sdc, mut sd15r, mut sd15c, mut sbr, mut sbc, mut sb15r, mut sb15c) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain([&dense, &per150]) {
        let is_six = b.n >= 890_000;
        let run_p = |mode: usize, km1: &mut master_a1::Kernel, km2: &mut master_a2::Kernel, kdiet: &mut cand_cks_diet::Kernel, ki15d: &mut cand_i15_diet::Kernel, kbi: &mut cand_cks_bi::Kernel, ki15b: &mut cand_i15_bi::Kernel, out: &mut Vec<u8>| -> (f64, f64) {
            let (mut ta, mut tb) = (vec![], vec![]);
            for i in 0..2 * reps {
                let t = Instant::now();
                if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
                else {
                    match mode {
                        0 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, out); }
                        1 => { kdiet.run_entry(&b.records, b.n, b.p, cand_cks_diet::Cap::NONE, out); }
                        2 => { ki15d.run_entry(&b.records, b.n, b.p, cand_i15_diet::Cap::NONE, out); }
                        3 => { kbi.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, out); }
                        _ => { ki15b.run_entry(&b.records, b.n, b.p, cand_i15_bi::Cap::NONE, out); }
                    }
                    tb.push(t.elapsed().as_secs_f64() * 1e6);
                }
            }
            (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10))
        };
        let (aar, aac) = run_p(0, &mut km1, &mut km2, &mut kdiet, &mut ki15d, &mut kbi, &mut ki15b, &mut out);
        let (dr, dc) = run_p(1, &mut km1, &mut km2, &mut kdiet, &mut ki15d, &mut kbi, &mut ki15b, &mut out);
        let (d15r, d15c) = run_p(2, &mut km1, &mut km2, &mut kdiet, &mut ki15d, &mut kbi, &mut ki15b, &mut out);
        let (br, bc) = run_p(3, &mut km1, &mut km2, &mut kdiet, &mut ki15d, &mut kbi, &mut ki15b, &mut out);
        let (b15r, b15c) = run_p(4, &mut km1, &mut km2, &mut kdiet, &mut ki15d, &mut kbi, &mut ki15b, &mut out);
        if is_six {
            saar += aar; saac += aac; sdr += dr; sdc += dc; sd15r += d15r; sd15c += d15c;
            sbr += br; sbc += bc; sb15r += b15r; sb15c += b15c;
        }
        println!(
            "1T {:14} | A/A={:.3} | Diet={:.3} | I15-Diet={:.3} | BiOp={:.3} | I15-Bi={:.3}",
            b.name, aac / aar, dc / dr, d15c / d15r, bc / br, b15c / b15r
        );
    }
    println!(
        "1T 6-BLK SUM    | A/A={:.4} | Diet={:.4} | I15-Diet={:.4} | BiOp={:.4} | I15-Bi={:.4}",
        saac / saar, sdc / sdr, sd15c / sd15r, sbc / sbr, sb15c / sb15r
    );

    println!("\n=== 6. 2T CONCURRENT BARRIER A/A & A/B (45 passes/worker) ===");
    let workers = 2usize;
    let passes = 45usize;
    for (label, mode) in [
        ("2T A/A (Master1 vs Master2)", 0),
        ("2T A/B (Master1 vs DietOperad)", 1),
        ("2T A/B (Master1 vs I15-Diet)", 2),
        ("2T A/B (Master1 vs BiOperad)", 3),
        ("2T A/B (Master1 vs I15-Bi)", 4),
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
                        let mut kd = cand_cks_diet::Kernel::default();
                        let mut kd15 = cand_i15_diet::Kernel::default();
                        let mut kb = cand_cks_bi::Kernel::default();
                        let mut kb15 = cand_i15_bi::Kernel::default();
                        let mut out = Vec::with_capacity(900_064);
                        for b in six_ref {
                            km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                            km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                            kd.run_entry(&b.records, b.n, b.p, cand_cks_diet::Cap::NONE, &mut out);
                            kd15.run_entry(&b.records, b.n, b.p, cand_i15_diet::Cap::NONE, &mut out);
                            kb.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out);
                            kb15.run_entry(&b.records, b.n, b.p, cand_i15_bi::Cap::NONE, &mut out);
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
                                else if mode == 1 { kd.run_entry(&b.records, b.n, b.p, cand_cks_diet::Cap::NONE, &mut out); }
                                else if mode == 2 { kd15.run_entry(&b.records, b.n, b.p, cand_i15_diet::Cap::NONE, &mut out); }
                                else if mode == 3 { kb.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out); }
                                else { kb15.run_entry(&b.records, b.n, b.p, cand_i15_bi::Cap::NONE, &mut out); }
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
