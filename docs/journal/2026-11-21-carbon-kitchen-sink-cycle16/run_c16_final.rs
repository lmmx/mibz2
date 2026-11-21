#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_mn16_direct;
mod cand_mn16_diet;

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
            if (r[a], r[(a + k) % n]) != (r[b], r[(b + k) % n]) {
                cur += 1;
            }
            tmp[b] = cur;
        }
        rank.copy_from_slice(&tmp);
        if cur as usize + 1 == n {
            break;
        }
        k <<= 1;
    }
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (row, &s) in sa.iter().enumerate() {
        if s == 0 {
            p = row as u32;
        }
        let prev = if s == 0 { n - 1 } else { s as usize - 1 };
        l.push(text[prev]);
    }
    (l, p)
}

fn load_blocks() -> (Vec<Block>, Vec<Block>) {
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
    let mut six = Vec::new();
    for (f, p) in specs {
        let l = std::fs::read(format!("{dir}/{f}")).unwrap();
        let expected = plain_invert(&l, p);
        let records = records_of(&l);
        six.push(Block { name: f, n: l.len(), p, records, expected });
    }
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
        n: dl.len(),
        p: dp,
        records: drec,
        expected: dense_src,
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
    let mut out = Vec::with_capacity(900_064);

    println!("=== 1. PARITY, WORD QUOTIENT & SCRATCH AUDIT ===");
    for b in six.iter().chain(extra.iter()) {
        let mut fm = master_a1::Kernel::default();
        let mut fd = cand_mn16_direct::Kernel::default();
        let mut fdiet = cand_mn16_diet::Kernel::default();
        let cm = fm.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let cd = fd.run_entry(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let cdiet = fdiet.run_entry(&b.records, b.n, b.p, cand_mn16_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let (sm, dm, _) = fm.scratch_report();
        let (sd, dd, _) = fd.scratch_report();
        let (sdiet, _, _) = fdiet.scratch_report();
        println!(
            "BLK {:22} | words: {:4} -> {:4} (exact={:4}) | dict_L1_KiB: {:5.1} -> {:5.1} | fresh_KiB: {:6.1} -> {:6.1} (diet {:6.1})",
            b.name, cm.words, cd.words, cdiet.words,
            dm as f64 / 1024.0, dd as f64 / 1024.0,
            sm as f64 / 1024.0, sd as f64 / 1024.0, sdiet as f64 / 1024.0
        );
    }

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kdir = cand_mn16_direct::Kernel::default();
    let mut kdiet = cand_mn16_diet::Kernel::default();
    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
        kdir.run_entry(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out);
        kdiet.run_entry(&b.records, b.n, b.p, cand_mn16_diet::Cap::NONE, &mut out);
    }
    let (cm_tot, _, cm_p) = km1.scratch_report();
    let (cd_tot, _, cd_p) = kdir.scratch_report();
    let (cdiet_tot, _, cdiet_p) = kdiet.scratch_report();
    println!("\nCUMULATIVE 6-BLOCK SCRATCH:");
    println!("  master_0.5.11   : {:7} B ({:6.1} KiB) | {}", cm_tot, cm_tot as f64 / 1024.0, cm_p);
    println!("  mn16_direct     : {:7} B ({:6.1} KiB) | {}", cd_tot, cd_tot as f64 / 1024.0, cd_p);
    println!("  mn16_diet       : {:7} B ({:6.1} KiB) | {}", cdiet_tot, cdiet_tot as f64 / 1024.0, cdiet_p);

    println!("\n=== 2. STAGE ATTRIBUTION (construct_us vs walk_us, 40 interleaved pairs/block) ===");
    let reps = 40usize;
    let (mut sc_m, mut sw_m, mut sc_d, mut sw_d) = (0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain(extra.iter().take(1)) {
        let (mut mc, mut mw, mut dc, mut dw) = (vec![], vec![], vec![], vec![]);
        for i in 0..2 * reps {
            if (i + i / 2) % 2 == 0 {
                let (c, w) = km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                mc.push(c); mw.push(w);
            } else {
                let (c, w) = kdir.run_stages(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out);
                dc.push(c); dw.push(w);
            }
        }
        let (mc10, mw10, dc10, dw10) = (pct(&mut mc, 1, 10), pct(&mut mw, 1, 10), pct(&mut dc, 1, 10), pct(&mut dw, 1, 10));
        if b.name.ends_with(".L") {
            sc_m += mc10; sw_m += mw10; sc_d += dc10; sw_d += dw10;
        }
        println!("STAGE {:22} | construct: {:6.1} -> {:6.1} ({:.3}x) | walk/emit: {:6.1} -> {:6.1} ({:.3}x)",
            b.name, mc10, dc10, dc10 / mc10, mw10, dw10, dw10 / mw10);
    }
    println!("STAGE 6-BLOCK SUM      | construct: {:6.1} -> {:6.1} ({:.4}x) | walk/emit: {:6.1} -> {:6.1} ({:.4}x)",
        sc_m, sc_d, sc_d / sc_m, sw_m, sw_d, sw_d / sw_m);

    println!("\n=== 3. END-TO-END 1T & 2T A/A vs A/B (60 interleaved pairs) ===");
    let reps = 60usize;
    let (mut saa_r, mut saa_c, mut sdir_r, mut sdir_c, mut sdiet_r, mut sdiet_c) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
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
            else { kdir.run_entry(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (dr, dc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kdiet.run_entry(&b.records, b.n, b.p, cand_mn16_diet::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (tr_d, tc_d) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        if b.name.ends_with(".L") {
            saa_r += aar; saa_c += aac; sdir_r += dr; sdir_c += dc; sdiet_r += tr_d; sdiet_c += tc_d;
        }
        println!("1T {:22} | A/A={:.3} | MN16-Direct={:.3} ({:.1}/{:.1}) | MN16-Diet={:.3} ({:.1}/{:.1})",
            b.name, aac / aar, dc / dr, dc, dr, tc_d / tr_d, tc_d, tr_d);
    }
    println!("1T 6-BLOCK TOTAL          | A/A={:.4} | MN16-Direct={:.4} | MN16-Diet={:.4}",
        saa_c / saa_r, sdir_c / sdir_r, sdiet_c / sdiet_r);

    let workers = 2usize;
    let passes = 40usize;
    for (label, mode) in [("2T A/A (Master1 vs Master2)", 0), ("2T A/B (Master1 vs MN16-Direct)", 1), ("2T A/B (Master1 vs MN16-Diet)", 2)] {
        let barrier = Barrier::new(workers);
        let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default();
                    let mut km2 = master_a2::Kernel::default();
                    let mut kd = cand_mn16_direct::Kernel::default();
                    let mut kdt = cand_mn16_diet::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        kd.run_entry(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out);
                        kdt.run_entry(&b.records, b.n, b.p, cand_mn16_diet::Cap::NONE, &mut out);
                    }
                    let (mut tr, mut tc) = (vec![], vec![]);
                    for pass in 0..(2 * passes + 4) {
                        let which = (pass + pass / 2) % 2;
                        bar.wait();
                        let t0 = Instant::now();
                        for i in 0..six_ref.len() {
                            let b = &six_ref[(i + w * 3) % six_ref.len()];
                            if which == 0 {
                                km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                            } else if mode == 0 {
                                km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                            } else if mode == 1 {
                                kd.run_entry(&b.records, b.n, b.p, cand_mn16_direct::Cap::NONE, &mut out);
                            } else {
                                kdt.run_entry(&b.records, b.n, b.p, cand_mn16_diet::Cap::NONE, &mut out);
                            }
                        }
                        let dt = t0.elapsed().as_secs_f64() * 1e6;
                        if pass >= 4 {
                            if which == 0 { tr.push(dt); } else { tc.push(dt); }
                        }
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
