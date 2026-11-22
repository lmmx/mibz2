#![forbid(unsafe_code)]
#![allow(dead_code)]
mod master_a1;
mod master_a2;
mod cand_biop48;
mod ab_all;
mod ab_all_pos;
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
    let mut k48 = cand_biop48::Kernel::default();
    let mut kall = ab_all::Kernel::default();
    let mut kap = ab_all_pos::Kernel::default();
    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        k48.run_entry(&b.records, b.n, b.p, cand_biop48::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kall.run_entry(&b.records, b.n, b.p, ab_all::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kap.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
    }
    let reps = 70usize;
    let (mut saar, mut saac, mut s48r, mut s48c, mut sar, mut sac, mut sapr, mut sapc) = (0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0);
    for b in &six {
        let mut r = [0.0; 4]; let mut c = [0.0; 4];
        for m in 0..4 {
            let (mut ta, mut tb) = (vec![], vec![]);
            for i in 0..2*reps {
                let t = Instant::now();
                if (i + i/2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64()*1e6); }
                else {
                    match m {
                        0 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        1 => { k48.run_entry(&b.records, b.n, b.p, cand_biop48::Cap::NONE, &mut out); }
                        2 => { kall.run_entry(&b.records, b.n, b.p, ab_all::Cap::NONE, &mut out); }
                        _ => { kap.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out); }
                    }
                    tb.push(t.elapsed().as_secs_f64()*1e6);
                }
            }
            r[m] = pct(&mut ta, 1, 10); c[m] = pct(&mut tb, 1, 10);
        }
        saar += r[0]; saac += c[0]; s48r += r[1]; s48c += c[1]; sar += r[2]; sac += c[2]; sapr += r[3]; sapc += c[3];
        println!("1T {:12} | A/A={:.3} | BiOp48={:.3} | +All={:.3} | +All+Pos={:.3}", b.name, c[0]/r[0], c[1]/r[1], c[2]/r[2], c[3]/r[3]);
    }
    println!("1T 6-BLK SUM  | A/A={:.4} | BiOp48={:.4} | +All={:.4} | +All+Pos={:.4}", saac/saar, s48c/s48r, sac/sar, sapc/sapr);
    let workers = 2usize; let passes = 45usize;
    for (label, mode) in [("2T A/A (M1 vs M2)", 0), ("2T A/B (M1 vs BiOp48)", 1), ("2T A/B (M1 vs +All)", 2), ("2T A/B (M1 vs +All+Pos)", 3)] {
        let barrier = Barrier::new(workers); let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default(); let mut km2 = master_a2::Kernel::default();
                    let mut k48 = cand_biop48::Kernel::default(); let mut ka = ab_all::Kernel::default(); let mut kp = ab_all_pos::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        k48.run_entry(&b.records, b.n, b.p, cand_biop48::Cap::NONE, &mut out);
                        ka.run_entry(&b.records, b.n, b.p, ab_all::Cap::NONE, &mut out);
                        kp.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out);
                    }
                    let (mut tr, mut tc) = (vec![], vec![]);
                    for pass in 0..(2*passes + 4) {
                        let which = (pass + pass/2) % 2;
                        bar.wait(); let t0 = Instant::now();
                        for i in 0..six_ref.len() {
                            let b = &six_ref[(i + w*3) % six_ref.len()];
                            if which == 0 { km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            else if mode == 0 { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            else if mode == 1 { k48.run_entry(&b.records, b.n, b.p, cand_biop48::Cap::NONE, &mut out); }
                            else if mode == 2 { ka.run_entry(&b.records, b.n, b.p, ab_all::Cap::NONE, &mut out); }
                            else { kp.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out); }
                        }
                        let dt = t0.elapsed().as_secs_f64()*1e6;
                        if pass >= 4 { if which == 0 { tr.push(dt); } else { tc.push(dt); } }
                    }
                    (tr, tc)
                })
            }).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut ar: Vec<f64> = res.iter().flat_map(|x| x.0.clone()).collect();
        let mut ac: Vec<f64> = res.iter().flat_map(|x| x.1.clone()).collect();
        println!("{:26} | p10={:.4} ({:.1}/{:.1}) | p50={:.4} ({:.1}/{:.1})",
            label, pct(&mut ac, 1, 10)/pct(&mut ar, 1, 10), pct(&mut ac, 1, 10), pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 2)/pct(&mut ar, 1, 2), pct(&mut ac, 1, 2), pct(&mut ar, 1, 2));
    }
}
