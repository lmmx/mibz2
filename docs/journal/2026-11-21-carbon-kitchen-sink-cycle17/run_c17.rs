#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_xor;

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

fn exhaustive_verify() {
    let mut k_xor = cand_xor::Kernel::default();
    let mut out = Vec::new();
    let mut bwt_checked = 0usize;
    let mut non_bwt_safe = 0usize;

    // 1. All binary strings of length 1..=11 (both primitive AND periodic BWTs!) across limits 1, 2, 4, 16
    for len in 1..=11usize {
        for bits in 0..(1u32 << len) {
            let text: Vec<u8> = (0..len).map(|i| ((bits >> i) & 1) as u8).collect();
            let (l, p) = bwt_of(&text);
            let recs = records_of(&l);
            for lim in [1u32, 2, 4, 16] {
                k_xor.try_run::<true, true>(&recs, len, p, lim, cand_xor::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text, "BWT mismatch len={len} bits={bits} lim={lim}");
                bwt_checked += 1;
            }
        }
    }
    // 2. All ternary strings of length 1..=7
    for len in 1..=7usize {
        let mut tot = 1u32;
        for _ in 0..len { tot *= 3; }
        for mut v in 0..tot {
            let mut text = Vec::with_capacity(len);
            for _ in 0..len { text.push((v % 3) as u8); v /= 3; }
            let (l, p) = bwt_of(&text);
            let recs = records_of(&l);
            for lim in [1u32, 2, 16] {
                k_xor.try_run::<true, true>(&recs, len, p, lim, cand_xor::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text, "Ternary BWT mismatch len={len}");
                bwt_checked += 1;
            }
        }
    }
    // 3. Arbitrary non-BWT binary columns of length 1..=8 at EVERY p in 0..len: verify 100% panic-freedom & exact length `n`!
    for len in 1..=8usize {
        for bits in 0..(1u32 << len) {
            let l: Vec<u8> = (0..len).map(|i| ((bits >> i) & 1) as u8).collect();
            let recs = records_of(&l);
            for p in 0..len as u32 {
                for lim in [1u32, 2, 4, 16] {
                    k_xor.try_run::<true, true>(&recs, len, p, lim, cand_xor::Cap::NONE, &mut out).unwrap();
                    assert_eq!(out.len(), len);
                    non_bwt_safe += 1;
                }
            }
        }
    }
    // 4. Periodic 900 KB block (`U^6` of 150 KB authentic slice!)
    println!("EXHAUSTIVE VERIFY OK: bwt_exact_checks={bwt_checked} arbitrary_column_panic_free_checks={non_bwt_safe}");
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
    // Periodic block U^5 (150 KB = 5 x 30 KB authentic slice)
    let mut per_src = Vec::with_capacity(150_000);
    for _ in 0..5 {
        per_src.extend_from_slice(&six[0].expected[..30_000]);
    }
    let (pl, pp) = bwt_of(&per_src);
    let prec = records_of(&pl);
    extra.push(Block { name: "periodic-150k(d=5)".into(), n: pl.len(), p: pp, records: prec, expected: per_src });
    (six, extra)
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn main() {
    exhaustive_verify();
    let (six, extra) = load_blocks();
    let mut out = Vec::with_capacity(900_064);

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kxor = cand_xor::Kernel::default();

    for b in six.iter().chain(extra.iter()) {
        let mut fx = cand_xor::Kernel::default();
        fx.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "fresh XOR-BiEmit mismatch on {}", b.name);

        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kxor.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "reused XOR-BiEmit mismatch on {}", b.name);
    }

    let mut km_cum = master_a1::Kernel::default();
    let mut kx_cum = cand_xor::Kernel::default();
    for b in &six {
        km_cum.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
        kx_cum.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out);
    }
    let (sm, _, mp) = km_cum.scratch_report();
    let (sx, _, xp) = kx_cum.scratch_report();
    println!("CUMULATIVE 6-BLOCK SCRATCH (EXACT BYTE PARITY):");
    println!("  master_0.5.11: {:7} B ({:6.1} KiB) | {}", sm, sm as f64 / 1024.0, mp);
    println!("  xor_biemit   : {:7} B ({:6.1} KiB) | {}", sx, sx as f64 / 1024.0, xp);
    assert_eq!(sm, sx, "XOR-BiEmit must have 0 bytes extra scratch!");

    println!("\n=== STAGE ATTRIBUTION: construct_us (with XOR wire) vs walk_us (mu=2 BiEmit), 45 pairs/block ===");
    let reps = 45usize;
    let (mut sc_m, mut sw_m, mut sc_x, mut sw_x) = (0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain(extra.iter()) {
        let (mut mc, mut mw, mut xc, mut xw) = (vec![], vec![], vec![], vec![]);
        for i in 0..2 * reps {
            if (i + i / 2) % 2 == 0 {
                let (c, w) = km_cum.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                mc.push(c); mw.push(w);
            } else {
                let (c, w) = kx_cum.run_stages(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out);
                xc.push(c); xw.push(w);
            }
        }
        let (mc10, mw10, xc10, xw10) = (pct(&mut mc, 1, 10), pct(&mut mw, 1, 10), pct(&mut xc, 1, 10), pct(&mut xw, 1, 10));
        if b.name.ends_with(".L") {
            sc_m += mc10; sw_m += mw10; sc_x += xc10; sw_x += xw10;
        }
        println!("STAGE {:22} | construct: {:6.1} -> {:6.1} ({:.3}x, {:+.1} us) | walk/emit: {:6.1} -> {:6.1} ({:.3}x, {:+.1} us)",
            b.name, mc10, xc10, xc10 / mc10, xc10 - mc10, mw10, xw10, xw10 / mw10, xw10 - mw10);
    }
    println!("STAGE 6-BLOCK SUM      | construct: {:6.1} -> {:6.1} ({:.4}x, {:+.1} us) | walk/emit: {:6.1} -> {:6.1} ({:.4}x, {:+.1} us)",
        sc_m, sc_x, sc_x / sc_m, sc_x - sc_m, sw_m, sw_x, sw_x / sw_m, sw_x - sw_m);

    println!("\n=== END-TO-END 1T & 2T INTERLEAVED A/A vs A/B (60 pairs/block) ===");
    let reps = 60usize;
    let (mut saa_r, mut saa_c, mut sx_r, mut sx_c) = (0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain(extra.iter()) {
        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km_cum.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (aar, aac) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km_cum.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kx_cum.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (xr, xc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));
        if b.name.ends_with(".L") {
            saa_r += aar; saa_c += aac; sx_r += xr; sx_c += xc;
        }
        println!("1T {:22} | A/A={:.3} | XOR-BiEmit={:.3} ({:.1} / {:.1} us)", b.name, aac / aar, xc / xr, xc, xr);
    }
    println!("1T 6-BLOCK TOTAL          | A/A={:.4} | XOR-BiEmit={:.4} ({:.1} / {:.1} us)", saa_c / saa_r, sx_c / sx_r, sx_c, sx_r);

    let workers = 2usize;
    let passes = 40usize;
    for (label, mode) in [("2T A/A (Master1 vs Master2)", 0), ("2T A/B (Master1 vs XOR-BiEmit)", 1)] {
        let barrier = Barrier::new(workers);
        let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default();
                    let mut km2 = master_a2::Kernel::default();
                    let mut kx = cand_xor::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        kx.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out);
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
                            } else {
                                kx.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out);
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
