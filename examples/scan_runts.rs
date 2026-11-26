//! (research only, `explore`) The serial scanner's time a block, split by block size: the
//! runt blocks Wikimedia's compressor leaves between full ones (a 900,000-byte input chunk
//! whose RLE1 rows pass bzip2's 899,981-row block, its overflow a block of its own) against
//! the full ones. docs/journal/2026-11-26-explore/scan-lookups-and-runts.md.
//!
//!     scan_runts FILE.bz2 [PASSES]
//!
//! Each pass scans the whole file once in order (`stream::Scanner`, as the decoder's
//! scanner thread does), timing each block's `next`; a block's figure is its fastest pass.
//! Prints one line a block (index, symbols, ns) to stdout and a summary by class to stderr.
use mibz2::stream::{Event, Scanner, SliceSource};
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).expect("read the input");
    let passes: usize = a.get(2).map_or(9, |s| s.parse().unwrap());
    let mut best: Vec<(usize, f64)> = Vec::new();
    for pass in 0..passes {
        let mut sc = Scanner::new(SliceSource(&data));
        let mut syms = Vec::new();
        let mut i = 0;
        loop {
            let t = Instant::now();
            let ev = sc.next(std::mem::take(&mut syms)).unwrap();
            let ns = t.elapsed().as_nanos() as f64;
            let Event::Block { symbols: b, .. } = ev else {
                break;
            };
            if pass == 0 {
                best.push((b.syms.len(), ns));
            } else if ns < best[i].1 {
                best[i].1 = ns;
            }
            syms = b.syms;
            i += 1;
        }
    }
    let mut cls = [(0usize, 0usize, 0f64); 3];
    for (i, &(s, ns)) in best.iter().enumerate() {
        println!("{i}\t{s}\t{ns:.0}");
        let c = if s < 1000 {
            0
        } else if s < 100_000 {
            1
        } else {
            2
        };
        cls[c].0 += 1;
        cls[c].1 += s;
        cls[c].2 += ns;
    }
    let tot: f64 = cls.iter().map(|c| c.2).sum();
    for (name, c) in ["runt (<1k syms)", "mid", "full (>=100k syms)"]
        .iter()
        .zip(cls)
    {
        if c.0 > 0 {
            eprintln!(
                "{name}: {} blocks, {} syms, {:.3} ms ({:.2}% of scan), {:.2} us/block, {:.2} ns/sym",
                c.0, c.1, c.2 / 1e6, 100.0 * c.2 / tot, c.2 / c.0 as f64 / 1e3, c.2 / c.1 as f64
            );
        }
    }
    eprintln!("total scan {:.3} ms over {} blocks", tot / 1e6, best.len());
}
