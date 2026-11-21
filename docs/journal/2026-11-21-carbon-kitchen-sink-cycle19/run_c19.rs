#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_strat;
mod cand_mn16_direct;

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
    let mut out = Vec::with_capacity(900_064);

    println!("=== 1. SADDLE-POINT & KAC RETURN SPECTRUM across LIMIT = 10, 12, 14, 16 on 0.5.11 ===");
    for lim in [10u32, 12, 14, 16] {
        let mut km = master_a1::Kernel::default();
        let mut kmn = cand_mn16_direct::Kernel::default();
        let (mut sum_m, mut sum_ns, mut sum_words_m, mut sum_words_mn) = (0usize, 0usize, 0usize, 0usize);
        for b in &six {
            let cm = km.run_lim(&b.records, b.n, b.p, lim, master_a1::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, b.expected);
            let cmn = kmn.run_lim(&b.records, b.n, b.p, lim, cand_mn16_direct::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, b.expected);
            sum_m += cm.retained;
            sum_ns += cm.thick;
            sum_words_m += cm.words;
            sum_words_mn += cmn.words;
        }
        let (sc_m, dl_m, _) = km.scratch_report();
        let (sc_mn, dl_mn, parts_mn) = kmn.scratch_report();
        println!(
            "LIMIT={:2} | 6-blk sum_m={:7} sum_ns={:5} | sum_words: master={:5} mn16={:4} | cum_scratch_KiB: master={:6.1} mn16={:6.1} (last_dict_KiB={:4.1}/{:4.1}) | mn16_parts: {}",
            lim, sum_m, sum_ns, sum_words_m, sum_words_mn,
            sc_m as f64 / 1024.0, sc_mn as f64 / 1024.0,
            dl_m as f64 / 1024.0, dl_mn as f64 / 1024.0, parts_mn
        );
    }

    // Build two byte-stratified schedules L: Sigma -> {12..16}:
    // StratA (`Punct16_Rest12`): keep JSON structural hubs (`"`, `,`, `:`, `{`, `}`, `[`, `]`) at 16 so they don't fragment Elim,
    //   lower all other bytes (`a-z`, `0-9`, UTF-8) to 12 to shrink `m`!
    // StratB (`Punct12_Rest16`): lower JSON structural hubs to 12, keep rest at 16!
    let mut strat_a = [12u32; 256];
    for &c in b"\",:{}[]" { strat_a[c as usize] = 16; }
    let mut strat_b = [16u32; 256];
    for &c in b"\",:{}[]" { strat_b[c as usize] = 12; }

    println!("\n=== 2. INTERLEAVED 1T A/A & A/B (55 pairs/block) ACROSS UNIFORM, STRATIFIED & EPISTATIC MN16 ===");
    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut km12 = master_a1::Kernel::default();
    let mut km14 = master_a1::Kernel::default();
    let mut ksa = cand_strat::Kernel::default();
    let mut ksb = cand_strat::Kernel::default();
    let mut kmn12 = cand_mn16_direct::Kernel::default();
    let mut kmn14 = cand_mn16_direct::Kernel::default();
    let mut kmn16 = cand_mn16_direct::Kernel::default();

    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
        km12.run_lim(&b.records, b.n, b.p, 12, master_a1::Cap::NONE, &mut out);
        km14.run_lim(&b.records, b.n, b.p, 14, master_a1::Cap::NONE, &mut out);
        ksa.run_entry(&b.records, b.n, b.p, &strat_a, cand_strat::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        ksb.run_entry(&b.records, b.n, b.p, &strat_b, cand_strat::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kmn12.run_lim(&b.records, b.n, b.p, 12, cand_mn16_direct::Cap::NONE, &mut out);
        kmn14.run_lim(&b.records, b.n, b.p, 14, cand_mn16_direct::Cap::NONE, &mut out);
        kmn16.run_lim(&b.records, b.n, b.p, 16, cand_mn16_direct::Cap::NONE, &mut out);
    }
    println!("SCRATCH StratA (Punct16_Rest12)={:.1} KiB | StratB (Punct12_Rest16)={:.1} KiB",
        ksa.scratch_report().0 as f64 / 1024.0, ksb.scratch_report().0 as f64 / 1024.0);

    let reps = 55usize;
    let mut sums = [(0.0f64, 0.0f64); 8];
    for b in &six {
        let mut ratios = [0.0f64; 8];
        for arm in 0..8usize {
            let (mut ta, mut tb) = (vec![], vec![]);
            for i in 0..2 * reps {
                let t = Instant::now();
                if (i + i / 2) % 2 == 0 {
                    km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                    ta.push(t.elapsed().as_secs_f64() * 1e6);
                } else {
                    match arm {
                        0 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                        1 => { km14.run_lim(&b.records, b.n, b.p, 14, master_a1::Cap::NONE, &mut out); }
                        2 => { km12.run_lim(&b.records, b.n, b.p, 12, master_a1::Cap::NONE, &mut out); }
                        3 => { ksa.run_entry(&b.records, b.n, b.p, &strat_a, cand_strat::Cap::NONE, &mut out); }
                        4 => { ksb.run_entry(&b.records, b.n, b.p, &strat_b, cand_strat::Cap::NONE, &mut out); }
                        5 => { kmn16.run_lim(&b.records, b.n, b.p, 16, cand_mn16_direct::Cap::NONE, &mut out); }
                        6 => { kmn14.run_lim(&b.records, b.n, b.p, 14, cand_mn16_direct::Cap::NONE, &mut out); }
                        _ => { kmn12.run_lim(&b.records, b.n, b.p, 12, cand_mn16_direct::Cap::NONE, &mut out); }
                    }
                    tb.push(t.elapsed().as_secs_f64() * 1e6);
                }
            }
            let (r, c) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));
            sums[arm].0 += r;
            sums[arm].1 += c;
            ratios[arm] = c / r;
        }
        println!(
            "1T {:12} | A/A={:.3} | M@14={:.3} | M@12={:.3} | StratA={:.3} | StratB={:.3} | MN16@16={:.3} | MN16@14={:.3} | MN16@12={:.3}",
            b.name, ratios[0], ratios[1], ratios[2], ratios[3], ratios[4], ratios[5], ratios[6], ratios[7]
        );
    }
    println!(
        "1T 6-BLK SUM    | A/A={:.4} | M@14={:.4} | M@12={:.4} | StratA={:.4} | StratB={:.4} | MN16@16={:.4} | MN16@14={:.4} | MN16@12={:.4}",
        sums[0].1 / sums[0].0, sums[1].1 / sums[1].0, sums[2].1 / sums[2].0, sums[3].1 / sums[3].0,
        sums[4].1 / sums[4].0, sums[5].1 / sums[5].0, sums[6].1 / sums[6].0, sums[7].1 / sums[7].0,
    );
}
