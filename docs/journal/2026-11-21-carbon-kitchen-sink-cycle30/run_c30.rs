#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod ab_all_pos;
mod cand_w_clean;
mod cand_w_u64;
mod cand_w_lit;
mod cand_w_u64_both;

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
    let records = records_of(&l);
    Block { name: "dense-300k".into(), n, p, records, expected }
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
    let mut out = Vec::with_capacity(900_064);

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kbase = ab_all_pos::Kernel::default();
    let mut kcl = cand_w_clean::Kernel::default();
    let mut ku64 = cand_w_u64::Kernel::default();
    let mut klit = cand_w_lit::Kernel::default();
    let mut kboth = cand_w_u64_both::Kernel::default();

    for b in six.iter().chain(std::iter::once(&dense)) {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kbase.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected);
        kcl.run_entry(&b.records, b.n, b.p, cand_w_clean::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "Clean mismatch");
        ku64.run_entry(&b.records, b.n, b.p, cand_w_u64::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "U64 mismatch");
        klit.run_entry(&b.records, b.n, b.p, cand_w_lit::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "Lit mismatch");
        kboth.run_entry(&b.records, b.n, b.p, cand_w_u64_both::Cap::NONE, &mut out).unwrap(); assert_eq!(out, b.expected, "BothU64 mismatch");
    }
    println!("All 5 arms verified byte-for-byte on 6 Wikidata blocks + dense-300k!");

    println!("\n=== 1. WALK STAGE ATTRIBUTION (60 rounds/block, delta vs Neon-C29) ===");
    let mut wsum = [0.0f64; 6];
    for b in &six {
        let mut s = [vec![], vec![], vec![], vec![], vec![], vec![]];
        for i in 0..6 * 60 {
            let m = (i + i / 6) % 6;
            let w = match m {
                0 => km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).1,
                1 => kbase.run_stages(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out).1,
                2 => kcl.run_stages(&b.records, b.n, b.p, cand_w_clean::Cap::NONE, &mut out).1,
                3 => ku64.run_stages(&b.records, b.n, b.p, cand_w_u64::Cap::NONE, &mut out).1,
                4 => klit.run_stages(&b.records, b.n, b.p, cand_w_lit::Cap::NONE, &mut out).1,
                _ => kboth.run_stages(&b.records, b.n, b.p, cand_w_u64_both::Cap::NONE, &mut out).1,
            };
            s[m].push(w);
        }
        let p10: [f64; 6] = std::array::from_fn(|j| pct(&mut s[j], 1, 10));
        for j in 0..6 { wsum[j] += p10[j]; }
        println!(
            "WALK {:12} | M={:6.1} C29={:6.1} ({:.3}x) | Clean={:6.1} ({:+.1}) | BwdU64={:6.1} ({:+.1}) | Lit={:6.1} ({:+.1}) | BothU64={:6.1} ({:+.1})",
            b.name, p10[0], p10[1], p10[1]/p10[0], p10[2], p10[2]-p10[1], p10[3], p10[3]-p10[1], p10[4], p10[4]-p10[1], p10[5], p10[5]-p10[1]
        );
    }
    println!(
        "WALK 6-BLK SUM    | M={:6.1} C29={:6.1} ({:.4}x) | Clean={:6.1} ({:.4}x, {:+.1}) | BwdU64={:6.1} ({:.4}x, {:+.1}) | Lit={:6.1} ({:.4}x, {:+.1}) | BothU64={:6.1} ({:.4}x, {:+.1})",
        wsum[0], wsum[1], wsum[1]/wsum[0],
        wsum[2], wsum[2]/wsum[0], wsum[2]-wsum[1],
        wsum[3], wsum[3]/wsum[0], wsum[3]-wsum[1],
        wsum[4], wsum[4]/wsum[0], wsum[4]-wsum[1],
        wsum[5], wsum[5]/wsum[0], wsum[5]-wsum[1]
    );

    println!("\n=== 2. 1T INTERLEAVED A/A & A/B (70 pairs/block + dense-300k) ===");
    let reps = 70usize;
    let mut sr = [0.0f64; 6]; let mut sc = [0.0f64; 6];
    for b in six.iter().chain(std::iter::once(&dense)) {
        let is_six = b.n >= 890_000;
        let mut r = [0.0; 6]; let mut c = [0.0; 6];
        for mode in 0..6 {
            let (mut ta, mut tb) = (vec![], vec![]);
            for i in 0..2 * reps {
                let t = Instant::now();
                if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
                else {
                    match mode {
                        0 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        1 => { kbase.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out); }
                        2 => { kcl.run_entry(&b.records, b.n, b.p, cand_w_clean::Cap::NONE, &mut out); }
                        3 => { ku64.run_entry(&b.records, b.n, b.p, cand_w_u64::Cap::NONE, &mut out); }
                        4 => { klit.run_entry(&b.records, b.n, b.p, cand_w_lit::Cap::NONE, &mut out); }
                        _ => { kboth.run_entry(&b.records, b.n, b.p, cand_w_u64_both::Cap::NONE, &mut out); }
                    }
                    tb.push(t.elapsed().as_secs_f64() * 1e6);
                }
            }
            r[mode] = pct(&mut ta, 1, 10); c[mode] = pct(&mut tb, 1, 10);
            if is_six { sr[mode] += r[mode]; sc[mode] += c[mode]; }
        }
        println!(
            "1T {:12} | A/A={:.3} | C29={:.3} | Clean={:.3} | BwdU64={:.3} | Lit={:.3} | BothU64={:.3}",
            b.name, c[0]/r[0], c[1]/r[1], c[2]/r[2], c[3]/r[3], c[4]/r[4], c[5]/r[5]
        );
    }
    println!(
        "1T 6-BLK SUM  | A/A={:.4} | C29={:.4} | Clean={:.4} | BwdU64={:.4} | Lit={:.4} | BothU64={:.4}",
        sc[0]/sr[0], sc[1]/sr[1], sc[2]/sr[2], sc[3]/sr[3], sc[4]/sr[4], sc[5]/sr[5]
    );

    println!("\n=== 3. 2T CONCURRENT BARRIER A/A & A/B (45 passes/worker) ===");
    let workers = 2usize; let passes = 45usize;
    for (label, mode) in [
        ("2T A/A (M1 vs M2)", 0),
        ("2T A/B (M1 vs Neon-C29)", 1),
        ("2T A/B (M1 vs Walk-Clean)", 2),
        ("2T A/B (M1 vs Walk-BwdU64)", 3),
        ("2T A/B (M1 vs Walk-LitFast)", 4),
        ("2T A/B (M1 vs Walk-BothU64)", 5),
    ] {
        let barrier = Barrier::new(workers); let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default(); let mut km2 = master_a2::Kernel::default();
                    let mut kb = ab_all_pos::Kernel::default(); let mut kc = cand_w_clean::Kernel::default();
                    let mut ku = cand_w_u64::Kernel::default(); let mut kl = cand_w_lit::Kernel::default();
                    let mut kbo = cand_w_u64_both::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        kb.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out);
                        kc.run_entry(&b.records, b.n, b.p, cand_w_clean::Cap::NONE, &mut out);
                        ku.run_entry(&b.records, b.n, b.p, cand_w_u64::Cap::NONE, &mut out);
                        kl.run_entry(&b.records, b.n, b.p, cand_w_lit::Cap::NONE, &mut out);
                        kbo.run_entry(&b.records, b.n, b.p, cand_w_u64_both::Cap::NONE, &mut out);
                    }
                    let (mut tr, mut tc) = (vec![], vec![]);
                    for pass in 0..(2*passes + 4) {
                        let which = (pass + pass/2) % 2;
                        bar.wait(); let t0 = Instant::now();
                        for i in 0..six_ref.len() {
                            let b = &six_ref[(i + w*3) % six_ref.len()];
                            if which == 0 { km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            else if mode == 0 { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            else if mode == 1 { kb.run_entry(&b.records, b.n, b.p, ab_all_pos::Cap::NONE, &mut out); }
                            else if mode == 2 { kc.run_entry(&b.records, b.n, b.p, cand_w_clean::Cap::NONE, &mut out); }
                            else if mode == 3 { ku.run_entry(&b.records, b.n, b.p, cand_w_u64::Cap::NONE, &mut out); }
                            else if mode == 4 { kl.run_entry(&b.records, b.n, b.p, cand_w_lit::Cap::NONE, &mut out); }
                            else { kbo.run_entry(&b.records, b.n, b.p, cand_w_u64_both::Cap::NONE, &mut out); }
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
        println!("{:30} | p10={:.4} ({:.1}/{:.1}) | p50={:.4} ({:.1}/{:.1})",
            label, pct(&mut ac, 1, 10)/pct(&mut ar, 1, 10), pct(&mut ac, 1, 10), pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 2)/pct(&mut ar, 1, 2), pct(&mut ac, 1, 2), pct(&mut ar, 1, 2));
    }
}
