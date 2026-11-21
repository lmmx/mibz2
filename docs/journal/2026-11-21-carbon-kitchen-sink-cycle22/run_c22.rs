#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_xor;
mod cand_split_xor;

use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::sync::Barrier;
use std::time::Instant;

struct Block {
    name: String,
    n: usize,
    p: u32,
    l_col: Vec<u8>,
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

/// Compute exact optimal Huffman-wavelet bit-plane traversal loads (`sum_c count[c] * depth[c]`)
fn wavelet_depth_census(b: &Block, m: usize) {
    let mut counts = [0usize; 256];
    for &ch in &b.l_col { counts[ch as usize] += 1; }
    let active: Vec<usize> = counts.iter().copied().filter(|&c| c > 0).collect();
    let alpha = active.len();
    let bal_bits = usize::BITS as usize - (alpha - 1).leading_zeros() as usize;
    // Exact Huffman tree weighted path length = sum of internal node weights!
    let mut heap: BinaryHeap<Reverse<usize>> = active.into_iter().map(Reverse).collect();
    let mut huff_loads = 0usize;
    while heap.len() > 1 {
        let Reverse(a) = heap.pop().unwrap();
        let Reverse(c) = heap.pop().unwrap();
        huff_loads += a + c;
        heap.push(Reverse(a + c));
    }
    let bal_loads = b.n * bal_bits;
    println!(
        "WAVELET {:12} alpha={:3} | Bal_Wavelet_Loads={:7} ({}n = {:5.1}x Open_m) | Huff_Wavelet_Loads={:7} ({:.2}n = {:5.1}x Open_m, {:5.1}x BiEmit_D)",
        b.name, alpha,
        bal_loads, bal_bits, bal_loads as f64 / m as f64,
        huff_loads, huff_loads as f64 / b.n as f64,
        huff_loads as f64 / m as f64,
        huff_loads as f64 / ((m / 2) as f64),
    );
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
        Block { name: f, n: l.len(), p, l_col: l, records, expected }
    }).collect()
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn main() {
    let six = load_six();
    let mut out = Vec::with_capacity(900_064);

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kfx = cand_xor::Kernel::default();
    let mut ksx = cand_split_xor::Kernel::default();

    println!("=== 1. YANG-BAXTER / HUFFMAN WAVELET SERIAL-LOAD DEPTH CENSUS ===");
    for b in &six {
        let cm = km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kfx.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        ksx.run_entry(&b.records, b.n, b.p, cand_split_xor::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "Split-XOR mismatch on {}", b.name);
        wavelet_depth_census(b, cm.retained);
    }
    assert_eq!(km1.scratch_report().0, ksx.scratch_report().0);

    println!("\n=== 2. STAGE ATTRIBUTION: Fused-XOR (Cycle 17) vs Split-XOR (Cycle 22) vs Master 0.5.11 (50 pairs/block) ===");
    let reps = 50usize;
    let (mut scm, mut swm, mut scf, mut swf, mut scs, mut sws) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in &six {
        let (mut mc, mut mw, mut fc, mut fw) = (vec![], vec![], vec![], vec![]);
        for i in 0..2 * reps {
            if (i + i / 2) % 2 == 0 {
                let (c, w) = km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                mc.push(c); mw.push(w);
            } else {
                let (c, w) = kfx.run_stages(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out);
                fc.push(c); fw.push(w);
            }
        }
        let (mut mc2, mut mw2, mut sc, mut sw) = (vec![], vec![], vec![], vec![]);
        for i in 0..2 * reps {
            if (i + i / 2) % 2 == 0 {
                let (c, w) = km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                mc2.push(c); mw2.push(w);
            } else {
                let (c, w) = ksx.run_stages(&b.records, b.n, b.p, cand_split_xor::Cap::NONE, &mut out);
                sc.push(c); sw.push(w);
            }
        }
        let (mc10, mw10, fc10, fw10, sc10, sw10) = (
            pct(&mut mc, 1, 10), pct(&mut mw, 1, 10),
            pct(&mut fc, 1, 10), pct(&mut fw, 1, 10),
            pct(&mut sc, 1, 10), pct(&mut sw, 1, 10),
        );
        let _ = (mc2, mw2);
        scm += mc10; swm += mw10; scf += fc10; swf += fw10; scs += sc10; sws += sw10;
        println!(
            "STAGE {:12} | construct: master={:6.1} fused_xor={:6.1} ({:+.1}) split_xor={:6.1} ({:+.1}) | walk: {:6.1} -> {:6.1}",
            b.name, mc10, fc10, fc10 - mc10, sc10, sc10 - mc10, mw10, sw10
        );
    }
    println!(
        "STAGE 6-BLK SUM    | construct: master={:6.1} fused_xor={:6.1} ({:+.1} us) split_xor={:6.1} ({:+.1} us) | walk: {:6.1} -> {:6.1}",
        scm, scf, scf - scm, scs, scs - scm, swm, sws
    );
    let _ = swf;

    println!("\n=== 3. END-TO-END 1T & 2T A/A vs A/B (60 pairs/block) ===");
    let reps = 60usize;
    let (mut saar, mut saac, mut sfxr, mut sfxc, mut ssxr, mut ssxc) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in &six {
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
            else { kfx.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (fxr, fxc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { ksx.run_entry(&b.records, b.n, b.p, cand_split_xor::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (sxr, sxc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        saar += aar; saac += aac; sfxr += fxr; sfxc += fxc; ssxr += sxr; ssxc += sxc;
        println!("1T {:12} | A/A={:.3} | Fused-XOR={:.3} ({:.1}/{:.1}) | Split-XOR={:.3} ({:.1}/{:.1})",
            b.name, aac / aar, fxc / fxr, fxc, fxr, sxc / sxr, sxc, sxr);
    }
    println!("1T 6-BLOCK SUM  | A/A={:.4} | Fused-XOR={:.4} | Split-XOR={:.4}",
        saac / saar, sfxc / sfxr, ssxc / ssxr);

    let workers = 2usize;
    let passes = 40usize;
    for (label, mode) in [("2T A/A (Master1 vs Master2)", 0), ("2T A/B (Master1 vs Fused-XOR)", 1), ("2T A/B (Master1 vs Split-XOR)", 2)] {
        let barrier = Barrier::new(workers);
        let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default();
                    let mut km2 = master_a2::Kernel::default();
                    let mut kf = cand_xor::Kernel::default();
                    let mut ks = cand_split_xor::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        kf.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out);
                        ks.run_entry(&b.records, b.n, b.p, cand_split_xor::Cap::NONE, &mut out);
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
                            else if mode == 1 { kf.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out); }
                            else { ks.run_entry(&b.records, b.n, b.p, cand_split_xor::Cap::NONE, &mut out); }
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
