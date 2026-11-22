#![forbid(unsafe_code)]

use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::{runs, Kernels};
use std::time::Instant;

fn main() {
    let data = std::fs::read("/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2").unwrap();
    let k = Kernels::detect();
    let mut syms_all = Vec::new();
    for sp in mibz2::scan(&data) {
        let sp = sp.unwrap();
        let mut br = Bits::at(&data, sp.start_bit);
        let Item::Block(b) = read_item(&mut br, sp.level, Vec::new()).unwrap() else {
            panic!()
        };
        if runs::takes_runs(&b) {
            syms_all.push(b);
        }
    }
    let mut r = runs::Runs::default();
    // 1. runs::front (AVX2) vs runs::front_scalar (safe)
    let mut t_avx = Vec::new();
    let mut t_sca = Vec::new();
    for _ in 0..5 {
        let t0 = Instant::now();
        let mut tot = 0usize;
        for b in &syms_all {
            tot += runs::front(b, &mut r, k).unwrap();
        }
        std::hint::black_box(tot);
        t_avx.push(t0.elapsed().as_secs_f64() * 1e3);

        let t1 = Instant::now();
        let mut tot2 = 0usize;
        for b in &syms_all {
            tot2 += runs::front_scalar(b, &mut r).unwrap();
        }
        std::hint::black_box(tot2);
        t_sca.push(t1.elapsed().as_secs_f64() * 1e3);
    }
    t_avx.sort_by(|a, b| a.partial_cmp(b).unwrap());
    t_sca.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "Run-route blocks ({}): runs::front_avx2 med={:.2} ms [{:.2}, {:.2}] | runs::front_scalar med={:.2} ms [{:.2}, {:.2}]",
        syms_all.len(),
        t_avx[2], t_avx[0], t_avx[4],
        t_sca[2], t_sca[0], t_sca[4]
    );

    // Count exact distribution of MTF move distances and run lengths on Wikidata!
    let mut nn_lt8 = 0u64;
    let mut nn_lt16 = 0u64;
    let mut nn_lt32 = 0u64;
    let mut nn_tot = 0u64;
    let mut len_gt127 = 0u64;
    let mut len_gt255 = 0u64;
    let mut max_runs_tot = 0u64;
    for b in &syms_all {
        for &s in &b.syms {
            if s > 1 && s != b.eob {
                let nn = (s - 1) as usize;
                nn_tot += 1;
                if nn < 8 { nn_lt8 += 1; }
                if nn < 16 { nn_lt16 += 1; }
                if nn < 32 { nn_lt32 += 1; }
            }
        }
        runs::front(b, &mut r, k).unwrap();
        let recs = r.records();
        let mut i = 0;
        while i < recs.len() {
            let c = recs[i] >> 8;
            let mut w = (recs[i] & 255) as u32;
            i += 1;
            while i < recs.len() && (recs[i] >> 8) == c {
                w += (recs[i] & 255) as u32;
                i += 1;
            }
            max_runs_tot += 1;
            if w > 127 { len_gt127 += 1; }
            if w > 255 { len_gt255 += 1; }
        }
    }
    println!(
        "MTF index census ({nn_tot} literals): nn<8={:.2}%, nn<16={:.2}%, nn<32={:.2}%",
        100.0 * nn_lt8 as f64 / nn_tot as f64,
        100.0 * nn_lt16 as f64 / nn_tot as f64,
        100.0 * nn_lt32 as f64 / nn_tot as f64,
    );
    println!(
        "Maximal run length census ({max_runs_tot} runs): >127={len_gt127} ({:.2}%), >255={len_gt255} ({:.2}%)",
        100.0 * len_gt127 as f64 / max_runs_tot as f64,
        100.0 * len_gt255 as f64 / max_runs_tot as f64,
    );
}
