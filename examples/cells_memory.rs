//! The Cells memory census's driver (feature `memtrace`; docs/journal/2026-11-14-cells-memory/).
//!
//! ```text
//! cells_memory LABEL FILE.bz2 cells|segmented
//! ```
//!
//! Decodes every block on this thread with `decode_block`, as `decode` does, CRC checked,
//! with the Cells kernel (`cells`) or the Segmented kernel alone (`segmented`), and sets the
//! markers' mode so that only that kernel's run-record attempts are traced (src/memtrace.rs:
//! under the `memtrace` valgrind tool; natively the markers do nothing). Each block's
//! decoded length goes on stdout as one JSON line; in `cells` mode an admitted block's line
//! also has the writer's long copies by how far back their source lies (from the census
//! record): bytes and copies per power-of-two bucket of distance d - f, where d is the
//! copy's first output byte and f its source's.

use mibz2::bwt::{Cells, CellsCensus, InverseBwt, Segmented};
use mibz2::memtrace::{BLOCK, CELLS, MODE, SEGMENTED};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_block, Kernels, Scratch};
use std::sync::atomic::Ordering;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let label = &args[1];
    let data = std::fs::read(&args[2]).expect("read the input");
    match args[3].as_str() {
        "cells" => run::<Cells>(label, &data, CELLS, |s| s.take_cells_census()),
        "segmented" => run::<Segmented>(label, &data, SEGMENTED, |_| None),
        m => panic!("mode {m}: cells or segmented"),
    }
}

fn run<I: InverseBwt + Default>(
    label: &str,
    data: &[u8],
    mode: u8,
    census: impl Fn(&mut Scratch<I>) -> Option<CellsCensus>,
) {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut s = Scratch::<I>::default();
    let mut out = Vec::new();
    let mut syms = Vec::new();
    MODE.store(mode, Ordering::Relaxed);
    let path = if mode == CELLS { "cells" } else { "segmented" };
    loop {
        match sc.next(syms).expect("scan") {
            Event::Block { index, symbols, .. } => {
                BLOCK.store(index, Ordering::Relaxed);
                out.clear();
                decode_block(&symbols, k, &mut s, &mut out).expect("decode the block");
                let mut line = format!(
                    "{{\"corpus\":\"{label}\",\"path\":\"{path}\",\"block\":{index},\"decoded\":{}",
                    out.len()
                );
                if let Some(c) = census(&mut s) {
                    line += &format!(",\"n\":{},\"outcome\":\"{}\"", c.n, c.outcome);
                    if c.outcome == "admitted" {
                        line += &copies(&c);
                    }
                }
                line.push('}');
                println!("{line}");
                syms = symbols.syms;
            }
            Event::Done => break,
        }
    }
    MODE.store(0, Ordering::Relaxed);
}

/// The writer's long copies (and a short cycle's repetitions) by distance back, in buckets
/// of floor(log2(d - f))
fn copies(c: &CellsCensus) -> String {
    let (n, p) = (c.n, c.period);
    let mut all: Vec<(u64, u64, u64)> = c
        .copies
        .iter()
        .map(|&(d, f, l)| (d as u64, f as u64, l as u64))
        .collect();
    let mut have = p.min(n);
    while have < n {
        let more = have.min(n - have);
        all.push((have as u64, 0, more as u64));
        have += more;
    }
    let (mut bytes, mut count) = ([0u64; 25], [0u64; 25]);
    let mut overlap = 0u64;
    for &(d, f, l) in &all {
        let back = d - f;
        let b = (63 - back.leading_zeros() as usize).min(24);
        bytes[b] += l;
        count[b] += 1;
        // a copy whose source reaches into its own destination (an overlapping memmove)
        overlap += u64::from(f + l > d);
    }
    let j = |v: &[u64]| v.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    format!(
        ",\"period\":{p},\"small_bytes\":{},\"copy_bytes\":{},\"copies\":{},\"overlapping\":{overlap},\"back_log2_bytes\":[{}],\"back_log2_count\":[{}]",
        c.small_bytes,
        all.iter().map(|x| x.2).sum::<u64>(),
        all.len(),
        j(&bytes),
        j(&count)
    )
}
