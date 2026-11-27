//! (research only, `explore`) The decoder's two main stations priced on one input, on one
//! thread: the serial scanner (`stream::Scanner::next`: headers, selectors, Huffman) against the
//! worker's whole block (`decode_block` with production's `OpenCells`: MTF/RLE2, construction,
//! emission, RLE1 and the CRC). docs/journal/2026-11-27-explore/station-balance-off-wikidata.md.
//!
//!     station_balance FILE.bz2 [PASSES]
//!
//! Each pass decodes the file once in order, timing each block's `next` and its `decode_block`
//! separately, as `decode_with` interleaves them. Per block the fastest pass is kept for each
//! station. Prints one tab-separated summary line:
//! file, blocks, full blocks, symbols, output bytes, input bytes, S ms, W ms, W/S (the worker
//! count at which W/k = S, the operational balance point), ns a symbol (S), ns an output byte (W).
//! A characterisation of station demand, not a wall-clock prediction: the lower bound
//! max(S, W/k, D) is not an equality (frontier/DECODER.md).
use mibz2::stream::{Event, Scanner, SliceSource};
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).expect("read the input");
    let passes: usize = a.get(2).map_or(5, |s| s.parse().unwrap());
    let k = mibz2::Kernels::detect();
    let mut s_best: Vec<f64> = Vec::new();
    let mut w_best: Vec<f64> = Vec::new();
    let mut nsyms: Vec<usize> = Vec::new();
    let mut nout: Vec<usize> = Vec::new();
    let mut scratch = mibz2::Scratch::<mibz2::bwt::OpenCells>::default();
    for pass in 0..passes {
        let mut sc = Scanner::new(SliceSource(&data));
        let mut syms = Vec::new();
        let mut out = Vec::new();
        let mut i = 0;
        loop {
            let t = Instant::now();
            let ev = sc.next(std::mem::take(&mut syms)).unwrap();
            let s_ns = t.elapsed().as_nanos() as f64;
            let Event::Block { symbols: b, .. } = ev else {
                break;
            };
            out.clear();
            let t = Instant::now();
            mibz2::decode_block(&b, k, &mut scratch, &mut out).expect("block decodes");
            let w_ns = t.elapsed().as_nanos() as f64;
            std::hint::black_box(&out);
            if pass == 0 {
                s_best.push(s_ns);
                w_best.push(w_ns);
                nsyms.push(b.syms.len());
                nout.push(out.len());
            } else {
                assert_eq!(
                    nout[i],
                    out.len(),
                    "the same block decodes to the same length"
                );
                s_best[i] = s_best[i].min(s_ns);
                w_best[i] = w_best[i].min(w_ns);
            }
            syms = b.syms;
            i += 1;
        }
    }
    let s: f64 = s_best.iter().sum::<f64>() / 1e6;
    let w: f64 = w_best.iter().sum::<f64>() / 1e6;
    let syms: usize = nsyms.iter().sum();
    let outb: usize = nout.iter().sum();
    let full = nout.iter().filter(|&&n| n >= 100_000).count();
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{:.2}\t{:.2}\t{:.2}\t{:.3}\t{:.3}",
        a[1],
        nout.len(),
        full,
        syms,
        outb,
        data.len(),
        s,
        w,
        w / s,
        s * 1e6 / syms as f64,
        w * 1e6 / outb as f64
    );
}
