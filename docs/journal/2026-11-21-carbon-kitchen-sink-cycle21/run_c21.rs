#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_muxband;
mod cand_diet2;

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

fn bwt_of(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<u32> = (0..n as u32).collect();
    let mut rank: Vec<u32> = text.iter().map(|&b| u32::from(b)).collect();
    let mut tmp = vec![0u32; n];
    let mut k = 1usize;
    while k < n {
        let r = &rank;
        sa.sort_unstable_by_key(|&i| {
            let j = (i as usize + k) % n;
            (r[i as usize], r[j])
        });
        let mut cur = 0u32;
        tmp[sa[0] as usize] = 0;
        for idx in 1..n {
            let a = sa[idx - 1] as usize;
            let b = sa[idx] as usize;
            if (r[a], r[(a + k) % n]) != (r[b], r[(b + k) % n]) { cur += 1; }
            tmp[b] = cur;
        }
        rank.copy_from_slice(&tmp);
        if cur as usize + 1 == n { break; }
        k <<= 1;
    }
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (row, &s) in sa.iter().enumerate() {
        if s == 0 { p = row as u32; }
        let prev = if s == 0 { n - 1 } else { s as usize - 1 };
        l.push(text[prev]);
    }
    (l, p)
}

/// Exact Berlekamp-Massey linear complexity over GF(257) for sequence `s` (mapped to `1..=256` in F_257)
fn berlekamp_massey_f257(s: &[u8]) -> usize {
    const P: u32 = 257;
    fn mod_pow(mut b: u32, mut e: u32) -> u32 {
        let mut r = 1u32;
        while e > 0 {
            if e & 1 == 1 { r = (r * b) % P; }
            b = (b * b) % P;
            e >>= 1;
        }
        r
    }
    let n = s.len();
    let mut c = vec![0u32; n + 1];
    let mut b = vec![0u32; n + 1];
    c[0] = 1;
    b[0] = 1;
    let (mut l, mut m, mut bb) = (0usize, 1usize, 1u32);
    for idx in 0..n {
        let mut d = u32::from(s[idx]) + 1;
        for i in 1..=l {
            d = (d + c[i] * (u32::from(s[idx - i]) + 1)) % P;
        }
        if d == 0 {
            m += 1;
        } else if 2 * l <= idx {
            let t = c.clone();
            let coef = (P - (d * mod_pow(bb, P - 2)) % P) % P;
            for i in 0..(n + 1 - m) {
                if b[i] != 0 {
                    c[i + m] = (c[i + m] + coef * b[i]) % P;
                }
            }
            l = idx + 1 - l;
            b = t;
            bb = d;
            m = 1;
        } else {
            let coef = (P - (d * mod_pow(bb, P - 2)) % P) % P;
            for i in 0..(n + 1 - m) {
                if b[i] != 0 {
                    c[i + m] = (c[i + m] + coef * b[i]) % P;
                }
            }
            m += 1;
        }
    }
    l
}

fn verify_all(six: &[Block]) {
    // 1. Berlekamp-Massey / Wiedemann Krylov Degree Census over F_257
    println!("=== 1. WIEDEMANN / BERLEKAMP-MASSEY KRYLOV DEGREE CENSUS OVER F_257 ===");
    for b in six {
        let l1024 = berlekamp_massey_f257(&b.expected[..1024]);
        let l4096 = berlekamp_massey_f257(&b.expected[..4096]);
        println!("  {:14} | Lambda_F257(1024) = {:4} ({:.3} N) | Lambda_F257(4096) = {:4} ({:.3} N)",
            b.name, l1024, l1024 as f64 / 1024.0, l4096, l4096 as f64 / 4096.0);
    }

    // 2. Exhaustive BWT + non-QUEUE + Cap::dense refusal parity
    let mut kmb = cand_muxband::Kernel::default();
    let mut kd2 = cand_diet2::Kernel::default();
    let mut out = Vec::new();
    let mut checks = 0usize;
    for len in 1..=10usize {
        for bits in 0..(1u32 << len) {
            let text: Vec<u8> = (0..len).map(|i| ((bits >> i) & 1) as u8).collect();
            let (l, p) = bwt_of(&text);
            let recs = records_of(&l);
            for lim in [1u32, 4, 16] {
                kmb.try_run::<true, true>(&recs, len, p, lim, cand_muxband::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                kd2.try_run::<true, true>(&recs, len, p, lim, cand_diet2::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                kd2.try_run::<true, false>(&recs, len, p, lim, cand_diet2::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                checks += 3;
            }
        }
    }
    // Check Cap refusal leaves tiny scratch:
    let mut kref = cand_diet2::Kernel::default();
    let tight_cap = cand_diet2::Cap { m: 50_000, strips: 100 };
    assert!(kref.try_run::<true, true>(&six[0].records, six[0].n, six[0].p, 16, tight_cap, &mut out).is_none());
    assert!(kref.scratch_report().0 <= 4096, "Refused block left excess scratch: {}", kref.scratch_report().0);
    println!("EXHAUSTIVE MUXBAND + DIET2 + CAP-REFUSAL VERIFY OK: {checks} checks!");
}

fn load_blocks() -> (Vec<Block>, Vec<Block>) {
    let dir = "/home/antigravity/admitted-bwt";
    let man = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let mut specs = vec![];
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") { file = v.trim_end_matches("\",").to_string(); }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") { specs.push((file.clone(), v.trim_end_matches(',').parse::<u32>().unwrap())); }
    }
    let six: Vec<Block> = specs.into_iter().map(|(f, p)| {
        let l = std::fs::read(format!("{dir}/{f}")).unwrap();
        let expected = plain_invert(&l, p);
        let records = records_of(&l);
        Block { name: f, n: l.len(), p, records, expected }
    }).collect();

    let mut dense_src = Vec::with_capacity(300_000);
    let src1 = &six[3].expected;
    let src2 = &six[5].expected;
    let mut off = 0usize;
    while dense_src.len() < 300_000 {
        dense_src.extend_from_slice(&src1[off % (src1.len() - 64)..(off % (src1.len() - 64)) + 48]);
        dense_src.extend_from_slice(&src2[(off * 7) % (src2.len() - 64)..((off * 7) % (src2.len() - 64)) + 48]);
        off += 97;
    }
    dense_src.truncate(300_000);
    let (dl, dp) = bwt_of(&dense_src);
    let drec = records_of(&dl);
    let mut extra = vec![Block {
        name: format!("dense-300k(r/n={:.3})", drec.len() as f64 / dl.len() as f64),
        n: dl.len(), p: dp, records: drec, expected: dense_src,
    }];
    let mut sp_text = Vec::with_capacity(200_000);
    for i in 0..50_000u32 {
        sp_text.extend_from_slice(b"aaab");
        if i % 997 == 0 { sp_text.push(b'c'); }
    }
    let (sl, sp) = bwt_of(&sp_text);
    let srec = records_of(&sl);
    extra.push(Block { name: "spiral-200k".into(), n: sl.len(), p: sp, records: srec, expected: sp_text });
    (six, extra)
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn main() {
    let (six, extra) = load_blocks();
    verify_all(&six);
    let mut out = Vec::with_capacity(900_064);

    println!("\n=== 2. FRESH & CUMULATIVE SCRATCH AUDIT (MuxBand & Diet2 vs Master 0.5.11) ===");
    for b in six.iter().chain(extra.iter()) {
        let mut fm = master_a1::Kernel::default();
        let mut fmb = cand_muxband::Kernel::default();
        let mut fd2 = cand_diet2::Kernel::default();
        fm.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        fmb.run_entry(&b.records, b.n, b.p, cand_muxband::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "MuxBand fresh mismatch on {}", b.name);
        fd2.run_entry(&b.records, b.n, b.p, cand_diet2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "Diet2 fresh mismatch on {}", b.name);
        let (sm, _, _) = fm.scratch_report();
        let (smb, _, _) = fmb.scratch_report();
        let (sd2, _, _) = fd2.scratch_report();
        println!("BLK {:22} | fresh_KiB: master={:6.1} | muxband={:6.1} ({:+.1}%) | diet2(Operad+MuxBand)={:6.1} ({:+.1}%)",
            b.name, sm as f64 / 1024.0,
            smb as f64 / 1024.0, 100.0 * (smb as f64 - sm as f64) / sm as f64,
            sd2 as f64 / 1024.0, 100.0 * (sd2 as f64 - sm as f64) / sm as f64);
    }

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kmb = cand_muxband::Kernel::default();
    let mut kd2 = cand_diet2::Kernel::default();
    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
        kmb.run_entry(&b.records, b.n, b.p, cand_muxband::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kd2.run_entry(&b.records, b.n, b.p, cand_diet2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
    }
    let (sm, _, pm) = km1.scratch_report();
    let (smb, _, pmb) = kmb.scratch_report();
    let (sd2, _, pd2) = kd2.scratch_report();
    println!("\nCUMULATIVE 6-BLOCK SCRATCH:");
    println!("  master_0.5.11: {:7} B ({:6.1} KiB)         | {}", sm, sm as f64 / 1024.0, pm);
    println!("  muxband_only : {:7} B ({:6.1} KiB, {:+5.1}%) | {}", smb, smb as f64 / 1024.0, 100.0 * (smb as f64 - sm as f64) / sm as f64, pmb);
    println!("  diet2_operad : {:7} B ({:6.1} KiB, {:+5.1}%) | {}", sd2, sd2 as f64 / 1024.0, 100.0 * (sd2 as f64 - sm as f64) / sm as f64, pd2);

    println!("\n=== 3. END-TO-END 1T & 2T A/A vs A/B (60 pairs/block) ===");
    let reps = 60usize;
    let (mut saar, mut saac, mut smbr, mut smbc, mut sd2r, mut sd2c) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain(extra.iter()) {
        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (aar, aac) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kmb.run_entry(&b.records, b.n, b.p, cand_muxband::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (mbr, mbc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kd2.run_entry(&b.records, b.n, b.p, cand_diet2::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (d2r, d2c) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        if b.name.ends_with(".L") { saar += aar; saac += aac; smbr += mbr; smbc += mbc; sd2r += d2r; sd2c += d2c; }
        println!("1T {:22} | A/A={:.3} | MuxBand={:.3} ({:.1}/{:.1}) | Diet2(Operad+MuxBand)={:.3} ({:.1}/{:.1})",
            b.name, aac / aar, mbc / mbr, mbc, mbr, d2c / d2r, d2c, d2r);
    }
    println!("1T 6-BLOCK TOTAL          | A/A={:.4} | MuxBand={:.4} | Diet2(Operad+MuxBand)={:.4}",
        saac / saar, smbc / smbr, sd2c / sd2r);

    let workers = 2usize;
    let passes = 40usize;
    for (label, mode) in [("2T A/A (Master1 vs Master2)", 0), ("2T A/B (Master1 vs MuxBand)", 1), ("2T A/B (Master1 vs Diet2)", 2)] {
        let barrier = Barrier::new(workers);
        let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default();
                    let mut km2 = master_a2::Kernel::default();
                    let mut kb = cand_muxband::Kernel::default();
                    let mut kd = cand_diet2::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        kb.run_entry(&b.records, b.n, b.p, cand_muxband::Cap::NONE, &mut out);
                        kd.run_entry(&b.records, b.n, b.p, cand_diet2::Cap::NONE, &mut out);
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
                            else if mode == 1 { kb.run_entry(&b.records, b.n, b.p, cand_muxband::Cap::NONE, &mut out); }
                            else { kd.run_entry(&b.records, b.n, b.p, cand_diet2::Cap::NONE, &mut out); }
                        }
                        let dt = t0.elapsed().as_secs_f64() * 1e6;
                        if pass >= 4 { if which == 0 { tr.push(dt); } else { tc.push(dt); } }
                    }
                    (tr, tc)
                })
            }).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut ar: Vec<f64> = res.iter().flat_map(|x| x.0.clone()).collect();
        let mut ac: Vec<f64> = res.iter().flat_map(|x| x.1.clone()).collect();
        println!("{:32} | p10={:.4} ({:.1}/{:.1}) | p50={:.4} ({:.1}/{:.1})",
            label, pct(&mut ac, 1, 10) / pct(&mut ar, 1, 10), pct(&mut ac, 1, 10), pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 2) / pct(&mut ar, 1, 2), pct(&mut ac, 1, 2), pct(&mut ar, 1, 2));
    }
}
