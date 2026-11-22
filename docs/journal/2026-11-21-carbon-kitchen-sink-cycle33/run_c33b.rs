#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_links_only;
mod cand_c33_wire8;
mod cand_c33_wire16;
mod cand_c33_w8_d12;
mod cand_c33_w16_d12;

use std::sync::Barrier;
use std::time::Instant;

struct Block { name: String, n: usize, p: u32, records: Vec<u16>, expected: Vec<u8> }
fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() { if (*rec >> 8) as u8 == b && (*rec & 255) < 127 { *rec += 1; continue; } }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
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
    let mut out = Vec::with_capacity(900_064);
    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kc31 = cand_links_only::Kernel::default();
    let mut kw8 = cand_c33_wire8::Kernel::default();
    let mut kw16 = cand_c33_wire16::Kernel::default();
    let mut kw8d = cand_c33_w8_d12::Kernel::default();
    let mut kw16d = cand_c33_w16_d12::Kernel::default();

    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kc31.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kw8.run_entry(&b.records, b.n, b.p, cand_c33_wire8::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kw16.run_entry(&b.records, b.n, b.p, cand_c33_wire16::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kw8d.run_entry(&b.records, b.n, b.p, cand_c33_w8_d12::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kw16d.run_entry(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
    }

    let names = ["MasterA1", "MasterA2", "Neon-C31", "Wire8", "Wire16", "W8+D12", "W16+D12"];
    println!("=== 1T INTERLEAVED CONFIRMATION (60 rounds/block) ===");
    let mut sum1t = [0.0f64; 7];
    let mut csum = [0.0f64; 7];
    let mut wsum = [0.0f64; 7];
    for b in &six {
        let mut s: [Vec<f64>; 7] = Default::default();
        for i in 0..7 * 60 {
            let m = (i + i / 7) % 7;
            let t0 = Instant::now();
            match m {
                0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                2 => { kc31.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                3 => { kw8.run_entry(&b.records, b.n, b.p, cand_c33_wire8::Cap::NONE, &mut out); }
                4 => { kw16.run_entry(&b.records, b.n, b.p, cand_c33_wire16::Cap::NONE, &mut out); }
                5 => { kw8d.run_entry(&b.records, b.n, b.p, cand_c33_w8_d12::Cap::NONE, &mut out); }
                _ => { kw16d.run_entry(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out); }
            }
            s[m].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let med: [f64; 7] = std::array::from_fn(|k| pct(&mut s[k], 1, 2));
        for k in 0..7 { sum1t[k] += med[k]; }
        println!("{:14} | M1={:6.1} | A/A={:.3} C31={:.3} W8={:.3} W16={:.3} W8D={:.3} W16D={:.3}",
            b.name, med[0], med[1]/med[0], med[2]/med[0], med[3]/med[0], med[4]/med[0], med[5]/med[0], med[6]/med[0]);

        let mut sc: [Vec<f64>; 7] = Default::default();
        let mut sw: [Vec<f64>; 7] = Default::default();
        for i in 0..7 * 40 {
            let m = (i + i / 7) % 7;
            let (c, w) = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out),
                1 => km2.run_stages(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out),
                2 => kc31.run_stages(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out),
                3 => kw8.run_stages(&b.records, b.n, b.p, cand_c33_wire8::Cap::NONE, &mut out),
                4 => kw16.run_stages(&b.records, b.n, b.p, cand_c33_wire16::Cap::NONE, &mut out),
                5 => kw8d.run_stages(&b.records, b.n, b.p, cand_c33_w8_d12::Cap::NONE, &mut out),
                _ => kw16d.run_stages(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        for k in 0..7 { csum[k] += pct(&mut sc[k], 1, 2); wsum[k] += pct(&mut sw[k], 1, 2); }
    }
    for k in 0..7 {
        println!("1T_SUM {:10} | {:8.1} us ({:.4}x M1, {:+.1} vs C31) | c={:6.1} w={:6.1}",
            names[k], sum1t[k], sum1t[k] / sum1t[0], sum1t[k] - sum1t[2], csum[k], wsum[k]);
    }

    println!("\n=== 2T CONCURRENT BARRIER CONFIRMATION (40 passes) ===");
    let mut r2t: [Vec<f64>; 7] = Default::default();
    for pass in 0..40 {
        for off in 0..7 {
            let m = (pass + off) % 7;
            let bar = Barrier::new(2);
            let t = std::thread::scope(|sc| {
                let b_ref = &six; let bar_ref = &bar;
                let h = sc.spawn(move || {
                    let mut out = Vec::with_capacity(900_064);
                    let mut km1 = master_a1::Kernel::default();
                    let mut km2 = master_a2::Kernel::default();
                    let mut kc = cand_links_only::Kernel::default();
                    let mut kw8 = cand_c33_wire8::Kernel::default();
                    let mut kw16 = cand_c33_wire16::Kernel::default();
                    let mut kw8d = cand_c33_w8_d12::Kernel::default();
                    let mut kw16d = cand_c33_w16_d12::Kernel::default();
                    for b in b_ref {
                        match m {
                            0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { kc.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                            3 => { kw8.run_entry(&b.records, b.n, b.p, cand_c33_wire8::Cap::NONE, &mut out); }
                            4 => { kw16.run_entry(&b.records, b.n, b.p, cand_c33_wire16::Cap::NONE, &mut out); }
                            5 => { kw8d.run_entry(&b.records, b.n, b.p, cand_c33_w8_d12::Cap::NONE, &mut out); }
                            _ => { kw16d.run_entry(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out); }
                        }
                    }
                    bar_ref.wait();
                    let t0 = Instant::now();
                    for b in b_ref.iter().rev() {
                        match m {
                            0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            2 => { kc.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                            3 => { kw8.run_entry(&b.records, b.n, b.p, cand_c33_wire8::Cap::NONE, &mut out); }
                            4 => { kw16.run_entry(&b.records, b.n, b.p, cand_c33_wire16::Cap::NONE, &mut out); }
                            5 => { kw8d.run_entry(&b.records, b.n, b.p, cand_c33_w8_d12::Cap::NONE, &mut out); }
                            _ => { kw16d.run_entry(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out); }
                        }
                    }
                    t0.elapsed().as_secs_f64() * 1e6
                });
                let mut out = Vec::with_capacity(900_064);
                let mut km1 = master_a1::Kernel::default();
                let mut km2 = master_a2::Kernel::default();
                let mut kc = cand_links_only::Kernel::default();
                let mut kw8 = cand_c33_wire8::Kernel::default();
                let mut kw16 = cand_c33_wire16::Kernel::default();
                let mut kw8d = cand_c33_w8_d12::Kernel::default();
                let mut kw16d = cand_c33_w16_d12::Kernel::default();
                for b in b_ref {
                    match m {
                        0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { kc.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                        3 => { kw8.run_entry(&b.records, b.n, b.p, cand_c33_wire8::Cap::NONE, &mut out); }
                        4 => { kw16.run_entry(&b.records, b.n, b.p, cand_c33_wire16::Cap::NONE, &mut out); }
                        5 => { kw8d.run_entry(&b.records, b.n, b.p, cand_c33_w8_d12::Cap::NONE, &mut out); }
                        _ => { kw16d.run_entry(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out); }
                    }
                }
                bar_ref.wait();
                let t0 = Instant::now();
                for b in b_ref {
                    match m {
                        0 => { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                        1 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        2 => { kc.run_entry(&b.records, b.n, b.p, cand_links_only::Cap::NONE, &mut out); }
                        3 => { kw8.run_entry(&b.records, b.n, b.p, cand_c33_wire8::Cap::NONE, &mut out); }
                        4 => { kw16.run_entry(&b.records, b.n, b.p, cand_c33_wire16::Cap::NONE, &mut out); }
                        5 => { kw8d.run_entry(&b.records, b.n, b.p, cand_c33_w8_d12::Cap::NONE, &mut out); }
                        _ => { kw16d.run_entry(&b.records, b.n, b.p, cand_c33_w16_d12::Cap::NONE, &mut out); }
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
    for k in 0..7 {
        let p10 = pct(&mut r2t[k], 1, 10);
        let p50 = pct(&mut r2t[k], 1, 2);
        println!("2T {:10} | p10={:8.1} us ({:.4}x) | p50={:8.1} us ({:.4}x)",
            names[k], p10, p10 / p10_0, p50, p50 / p50_0);
    }
}
