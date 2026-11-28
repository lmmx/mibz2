//! The line census (docs/journal/2026-11-10-by-data-class.md): how many distinct k-step words a
//! cache line's worth of consecutive rows holds. It counts; it times nothing, and nothing in
//! the decoder calls it.
//!
//! ```text
//! line_census FILE.bz2 [EVERY]     pooled over every EVERY-th full block (default 1)
//! ```
//!
//! **The objects.** Row i's k-step word is L[i], L[LF(i)], …, L[LF^(k-1)(i)]: the k bytes the
//! backward walk emits from row i. Rows that share a word are mapped by LF^k in order onto
//! consecutive rows, so within a line of W consecutive rows LF^k(i) is one base a distinct
//! word plus the count of that word earlier in the line. A line-local table stores the words
//! and one base a distinct word; what it costs is set by the distinct words a line.
//!
//! **What is counted,** for k in 1, 2, 4 and W in 8, 16, 32: the histogram of distinct words
//! in the aligned lines of W rows, as the share of lines at or under each capacity, and the
//! mean.

use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

const KS: [usize; 3] = [1, 2, 4];
const WS: [usize; 3] = [8, 16, 32];
const CHUNKS: [u32; 5] = [4, 6, 8, 10, 12];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1]).expect("read");
    let every: usize = args.get(2).map_or(1, |s| s.parse().unwrap());
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    // hist[k][w][d]: lines of WS[w] rows holding d distinct KS[k]-step words
    let mut hist = vec![vec![vec![0u64; 33]; WS.len()]; KS.len()];
    let (mut blocks, mut rows, mut runs, mut seen) = (0usize, 0usize, 0usize, 0usize);
    let mut stretches = [0u64; CHUNKS.len()];
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        syms = b.syms;
        let n = ll.len();
        seen += 1;
        if n < 100_000 || (seen - 1) % every != 0 {
            continue;
        }
        blocks += 1;
        rows += n;
        // distinct bytes of L in aligned chunks of 16 to 4096 rows: the occurrences of a byte
        // inside one chunk are consecutive rows of its bucket, and their links share every bit
        // above the chunk's. So this is the count of (bucket, chunk) stretches.
        for (ci, sh) in CHUNKS.iter().enumerate() {
            for chunk in ll.chunks(1 << sh) {
                let mut seen = [false; 256];
                for &c in chunk {
                    if !seen[c as usize] {
                        seen[c as usize] = true;
                        stretches[ci] += 1;
                    }
                }
            }
        }
        runs += 1 + ll.windows(2).filter(|w| w[0] != w[1]).count();
        // LF
        let mut next = [0u32; 256];
        let mut cum = 0u32;
        for c in 0..256 {
            next[c] = cum;
            cum += counts[c];
        }
        let mut lf = vec![0u32; n];
        for (i, &c) in ll.iter().enumerate() {
            lf[i] = next[c as usize];
            next[c as usize] += 1;
        }
        // words[i]: the k-step word of row i, k = 1, 2, 4, as a u32
        let w1: Vec<u32> = ll.iter().map(|&c| c as u32).collect();
        let w2: Vec<u32> = (0..n).map(|i| w1[i] | w1[lf[i] as usize] << 8).collect();
        let lf2: Vec<u32> = (0..n).map(|i| lf[lf[i] as usize]).collect();
        let w4: Vec<u32> = (0..n).map(|i| w2[i] | w2[lf2[i] as usize] << 16).collect();
        for (ki, words) in [&w1, &w2, &w4].into_iter().enumerate() {
            for (wi, &w) in WS.iter().enumerate() {
                for line in words.chunks_exact(w) {
                    let mut d = 0;
                    for (j, x) in line.iter().enumerate() {
                        if !line[..j].contains(x) {
                            d += 1;
                        }
                    }
                    hist[ki][wi][d] += 1;
                }
            }
        }
    }
    println!(
        "{blocks} blocks, {rows} rows, {:.4} runs a row",
        runs as f64 / rows as f64
    );
    for (ci, sh) in CHUNKS.iter().enumerate() {
        println!(
            "chunks of {:4} rows: {:.4} (bucket, chunk) stretches a row",
            1 << sh,
            stretches[ci] as f64 / rows as f64
        );
    }
    for (ki, &kk) in KS.iter().enumerate() {
        for (wi, &w) in WS.iter().enumerate() {
            let h = &hist[ki][wi];
            let lines: u64 = h.iter().sum();
            let mean = h
                .iter()
                .enumerate()
                .map(|(d, &c)| d as u64 * c)
                .sum::<u64>() as f64
                / lines as f64;
            let at = |cap: usize| h[..=cap.min(32)].iter().sum::<u64>() as f64 / lines as f64;
            println!(
                "k={kk} W={w:2}: mean {mean:5.2} distinct a line; lines at or under 2: {:.4}  3: {:.4}  4: {:.4}  5: {:.4}  6: {:.4}  7: {:.4}  8: {:.4}  9: {:.4}  10: {:.4}  12: {:.4}  16: {:.4}",
                at(2), at(3), at(4), at(5), at(6), at(7), at(8), at(9), at(10), at(12), at(16)
            );
        }
    }
}
