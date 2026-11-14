//! The Cells census (Cooperative Cells' screen, rule: #145 LOG 6878326831): an untimed count,
//! one JSON line a block, of what the cells kernel did with it. Needs the `census` feature.
//!
//! ```text
//! cells_census LABEL FILE.bz2 [--quiet]
//! ```
//!
//! Every block is decoded by `decode_block` with `bwt::Cells` on this thread, as `decode`
//! does, and its CRC checked. For a block that reached Cells (`runs::takes_runs`), the
//! kernel's census record (src/cells.rs, `Census`) gives the admission outcome and, for an
//! admitted block, the grammar and the writer's work replayed: the top word's factors (the
//! marked spine's plain factors and the row's byte, at their output places) and the writer's
//! long copies of earlier output. From them, for k = 2, 3, 4: the split of the block's output
//! into k spans at top-level factor boundaries whose largest deviation from n/k is least, each
//! span's share of n, and the bytes each span copies from an earlier span.
//!
//! `attempt` counts Cells attempts in order: a callgrind run with `--dump-before` on
//! `Kernel::slots` writes one dump an attempt, the k-th dump (from 1) holding attempt k-1's
//! slots, induction and writer (bench script: docs/journal/2026-11-14-cells-census/).
//! `--quiet` writes no records (for that run).

use mibz2::bwt::{Cells, CellsCensus};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_block, Kernels, Scratch};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let label = &args[1];
    let data = std::fs::read(&args[2]).expect("read the input");
    let quiet = args.iter().any(|a| a == "--quiet");
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data[..]));
    let mut s = Scratch::<Cells>::default();
    let mut out = Vec::new();
    let mut syms = Vec::new();
    let mut attempt = 0usize;
    loop {
        match sc.next(syms).expect("scan") {
            Event::Block { index, symbols, .. } => {
                out.clear();
                decode_block(&symbols, k, &mut s, &mut out).expect("decode the block");
                let census = s.take_cells_census();
                if !quiet {
                    println!(
                        "{}",
                        record(label, index, attempt, out.len(), census.as_ref())
                    );
                }
                attempt += usize::from(census.is_some());
                syms = symbols.syms;
            }
            Event::Done => break,
        }
    }
    eprintln!("cells_census {label}: {attempt} Cells attempts");
}

fn record(
    label: &str,
    index: u64,
    attempt: usize,
    decoded: usize,
    c: Option<&CellsCensus>,
) -> String {
    let head = format!("{{\"corpus\":\"{label}\",\"block\":{index},\"decoded\":{decoded}");
    let Some(c) = c else {
        return format!("{head},\"outcome\":\"not_runs\"}}");
    };
    let mut s = format!(
        "{head},\"attempt\":{attempt},\"n\":{},\"r\":{},\"letters\":{},\"outcome\":\"{}\"",
        c.n, c.records, c.letters, c.outcome
    );
    if c.outcome != "admitted" {
        s.push('}');
        return s;
    }
    let (n, p) = (c.n, c.period);
    // the block as written: the period's factors and copies, then the period's copies
    let mut top: Vec<(u64, u64)> = Vec::new();
    let mut copies: Vec<(u64, u64, u64)> = c
        .copies
        .iter()
        .map(|&(d, f, l)| (d as u64, f as u64, l as u64))
        .collect();
    let mut m = 0;
    while m * p < n {
        for &(d, l) in &c.top {
            let at = (m * p) as u64 + d as u64;
            if at < n as u64 {
                top.push((at, l as u64));
            }
        }
        m += 1;
    }
    let mut have = p.min(n);
    while have < n {
        let more = have.min(n - have);
        copies.push((have as u64, 0, more as u64));
        have += more;
    }
    let long: u64 = c.copies.iter().map(|x| x.2 as u64).sum();
    let reps = (n - p.min(n)) as u64;
    let accounted = c.small_bytes as u64 + long == p as u64;
    let largest = c.top.iter().map(|x| x.1).max().unwrap_or(0);
    s += &format!(
        ",\"cells\":{},\"period\":{p},\"rules\":{},\"rules_expanded\":{},\"max_depth\":{},\"spine\":{},\"top_factors\":{},\"largest_top_factor\":{largest},\"small_bytes\":{},\"long_copy_bytes\":{long},\"long_copies\":{},\"repetition_bytes\":{reps},\"accounted\":{accounted}",
        c.cells, c.rules, c.rules_expanded, c.max_depth, c.spine, c.top.len(), c.small_bytes, c.copies.len()
    );
    let mut cuts: Vec<u64> = top.iter().map(|x| x.0).filter(|&x| x > 0).collect();
    cuts.push(n as u64);
    cuts.sort_unstable();
    cuts.dedup();
    for k in [2usize, 3, 4] {
        let split = best_split(&cuts, n as u64, k);
        let shares: Vec<String> = split
            .windows(2)
            .map(|w| format!("{:.5}", (w[1] - w[0]) as f64 / n as f64))
            .collect();
        let dev = split
            .windows(2)
            .map(|w| ((w[1] - w[0]) as f64 * k as f64 / n as f64 - 1.0).abs())
            .fold(0.0, f64::max);
        let (cross, within) = dependence(&split, &copies);
        let ends: Vec<String> = split.iter().map(|x| x.to_string()).collect();
        s += &format!(
            ",\"k{k}\":{{\"cuts\":[{}],\"shares\":[{}],\"max_dev\":{dev:.5},\"within_10pct\":{},\"cross_span_copy_bytes\":{cross:?},\"same_span_copy_bytes\":{within:?}}}",
            ends.join(","),
            shares.join(","),
            dev <= 0.10
        );
    }
    s.push('}');
    s
}

/// Whether the cuts allow k spans each of [lo, hi] bytes: the spans' ends, or None
fn feasible(cuts: &[u64], n: u64, k: usize, lo: u64, hi: u64) -> Option<Vec<u64>> {
    // reach[i]: the cuts (indices) that end i spans, with each one's predecessor
    let mut prev_set: Vec<(u64, usize)> = vec![(0, usize::MAX)];
    let mut layers: Vec<Vec<(u64, usize)>> = Vec::new();
    for _ in 0..k {
        let mut next = Vec::new();
        for &b in cuts {
            let (a0, a1) = (b.saturating_sub(hi), b.saturating_sub(lo));
            if b < lo {
                continue;
            }
            let i = prev_set.partition_point(|x| x.0 < a0);
            if i < prev_set.len() && prev_set[i].0 <= a1 {
                next.push((b, i));
            }
        }
        layers.push(prev_set);
        prev_set = next;
        if prev_set.is_empty() {
            return None;
        }
    }
    let last = prev_set.iter().position(|x| x.0 == n)?;
    let mut ends = vec![n];
    let mut at = prev_set[last].1;
    for layer in layers.iter().rev() {
        ends.push(layer[at].0);
        at = layer[at].1;
    }
    ends.reverse();
    Some(ends)
}

/// The k spans whose largest deviation from n/k is least (to a part in 10^5 of n/k)
fn best_split(cuts: &[u64], n: u64, k: usize) -> Vec<u64> {
    let target = n as f64 / k as f64;
    let at = |t: f64| {
        let lo = (target * (1.0 - t)).ceil().max(1.0) as u64;
        let hi = (target * (1.0 + t)).floor() as u64;
        feasible(cuts, n, k, lo, hi)
    };
    let (mut lo, mut hi) = (0.0f64, k as f64);
    let mut best = at(hi).unwrap_or_else(|| vec![0, n]);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        match at(mid) {
            Some(v) => {
                best = v;
                hi = mid;
            }
            None => lo = mid,
        }
    }
    best
}

/// Per span: the bytes its long copies read from earlier spans, and from itself
fn dependence(ends: &[u64], copies: &[(u64, u64, u64)]) -> (Vec<u64>, Vec<u64>) {
    let k = ends.len() - 1;
    let (mut cross, mut within) = (vec![0u64; k], vec![0u64; k]);
    for &(d, f, l) in copies {
        for s in 0..k {
            let (x, y) = (d.max(ends[s]), (d + l).min(ends[s + 1]));
            if x >= y {
                continue;
            }
            let (sx, sy) = (f + (x - d), f + (y - d));
            let before = sy.min(ends[s]).saturating_sub(sx);
            cross[s] += before;
            within[s] += (sy - sx) - before;
        }
    }
    (cross, within)
}
