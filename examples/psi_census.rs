//! Research only, untimed (lmmx/mibz2#79): a census of ψ for the line-blocked record
//! designs, measured before any of them is built. ψ (the T-vector's link, T[j] >> 8) is
//! the concatenation, in F order, of each byte's sorted positions in L, so 32 consecutive
//! rows of one bucket are 32 increasing positions of one byte. Each 32-row group, one
//! aligned 64-byte record, is classified as:
//! - FOR-14: its span (last − first position) below 2^14, the 32 offsets in 14 bits each;
//! - EF-12: its span below 33 · 2^12, Elias–Fano with 12 low bits and a 64-bit unary high
//!   part (32 ones, the high parts at most 32);
//! - an exception: it crosses a bucket boundary, or its span is wider than EF-12 allows.
//!
//! Rows are counted, not groups (a block's last group can be short). Also measured: the F
//! windows of 2^k rows (k = 4..8) that hold a bucket boundary, the boundaries in each, and
//! the rows a window-plus-step lookup (#91's `FTable`) steps for.
//!
//! ```text
//! psi_census FILE.bz2 [OUT.jsonl]     one JSON line a block in OUT, the summary on stdout
//! ```

use std::io::Write;

use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

const GROUP: usize = 32;
const FOR_SPAN: u32 = 1 << 14;
const EF_SPAN: u32 = 33 << 12;

#[derive(Default, Clone)]
struct Tally {
    blocks: usize,
    rows: u64,
    for14: u64,
    ef12: u64,
    crossing: u64,
    wide: u64,
    /// rows by the log2 of their group's span (0 for a span below 2)
    span_log2: [u64; 21],
    /// per k = 4..8: windows, mixed windows, rows in mixed windows, rows stepped
    windows: [[u64; 4]; 5],
    /// per k: mixed windows by their boundary count (1, 2, 3, 4+)
    per_window: [[u64; 4]; 5],
}

impl Tally {
    fn add(&mut self, o: &Tally) {
        self.blocks += o.blocks;
        self.rows += o.rows;
        self.for14 += o.for14;
        self.ef12 += o.ef12;
        self.crossing += o.crossing;
        self.wide += o.wide;
        for i in 0..21 {
            self.span_log2[i] += o.span_log2[i];
        }
        for k in 0..5 {
            for x in 0..4 {
                self.windows[k][x] += o.windows[k][x];
                self.per_window[k][x] += o.per_window[k][x];
            }
        }
    }
}

fn census(ll: &[u8], counts: &[u32; 256]) -> (Tally, usize) {
    let n = ll.len();
    let mut t = Tally {
        blocks: 1,
        rows: n as u64,
        ..Default::default()
    };
    // ψ by a counting sort: row C[c] + r holds the position of the r-th c in L
    let mut start = [0u32; 257];
    for c in 0..256 {
        start[c + 1] = start[c] + counts[c];
    }
    let mut pos = start;
    let mut psi = vec![0u32; n];
    for (i, &c) in ll.iter().enumerate() {
        psi[pos[c as usize] as usize] = i as u32;
        pos[c as usize] += 1;
    }
    // each row's bucket, as its byte
    let mut f = vec![0u8; n];
    for c in 0..256 {
        f[start[c] as usize..start[c + 1] as usize].fill(c as u8);
    }
    for g in (0..n).step_by(GROUP) {
        let end = (g + GROUP).min(n);
        let rows = (end - g) as u64;
        if f[g] != f[end - 1] {
            t.crossing += rows;
            continue;
        }
        let span = psi[end - 1] - psi[g];
        t.span_log2[(32 - span.leading_zeros()).saturating_sub(1).min(20) as usize] += rows;
        if span < FOR_SPAN {
            t.for14 += rows;
        } else if span < EF_SPAN {
            t.ef12 += rows;
        } else {
            t.wide += rows;
        }
    }
    // the bucket boundaries strictly inside the block: the starts of non-empty buckets but
    // the first
    let bounds: Vec<usize> = (0..256)
        .filter(|&c| counts[c] > 0 && start[c] > 0)
        .map(|c| start[c] as usize)
        .collect();
    for (ki, k) in (4..=8).enumerate() {
        let w = 1usize << k;
        let windows = n.div_ceil(w);
        let mut per = vec![0u32; windows];
        let mut first = vec![usize::MAX; windows];
        for &b in &bounds {
            if b % w != 0 {
                per[b / w] += 1;
                first[b / w] = first[b / w].min(b);
            }
        }
        t.windows[ki][0] += windows as u64;
        for x in 0..windows {
            if per[x] > 0 {
                let wend = ((x + 1) * w).min(n);
                t.windows[ki][1] += 1;
                t.windows[ki][2] += (wend - x * w) as u64;
                t.windows[ki][3] += (wend - first[x]) as u64;
                t.per_window[ki][(per[x] as usize - 1).min(3)] += 1;
            }
        }
    }
    let sigma = counts.iter().filter(|&&c| c > 0).count();
    (t, sigma)
}

fn pct(x: u64, of: u64) -> String {
    format!("{:.2}%", 100.0 * x as f64 / of.max(1) as f64)
}

fn summary(name: &str, t: &Tally) {
    let r = t.rows;
    let exc = t.crossing + t.wide;
    // an exception's record points to its 32 entries stored whole, 24 bits each
    let bits = 16.0 + 24.0 * exc as f64 / r.max(1) as f64;
    println!(
        "| {name} | {} | {:.1} M | {} | {} | {} | {} | {} | {bits:.2} |",
        t.blocks,
        r as f64 / 1e6,
        pct(t.for14, r),
        pct(t.ef12, r),
        pct(t.crossing, r),
        pct(t.wide, r),
        pct(exc, r)
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(args.get(1).expect("psi_census FILE.bz2 [OUT.jsonl]")).unwrap();
    let mut out = args.get(2).map(|p| std::fs::File::create(p).unwrap());
    let kern = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    // all blocks, and by size (rows) and by alphabet (distinct bytes)
    let names = [
        "all",
        "rows < 300 k",
        "300-700 k",
        "> 700 k",
        "σ ≤ 64",
        "σ 65-128",
        "σ > 128",
    ];
    let mut tallies = vec![Tally::default(); names.len()];
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut i = 0;
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        mtf::mtf(&symbols, &mut ll, &mut counts, kern).unwrap();
        let (t, sigma) = census(&ll, &counts);
        let n = ll.len();
        let size = if n < 300_000 {
            1
        } else if n <= 700_000 {
            2
        } else {
            3
        };
        let alpha = if sigma <= 64 {
            4
        } else if sigma <= 128 {
            5
        } else {
            6
        };
        for x in [0, size, alpha] {
            tallies[x].add(&t);
        }
        if let Some(o) = out.as_mut() {
            writeln!(
                o,
                "{{\"block\": {i}, \"rows\": {n}, \"sigma\": {sigma}, \"for14\": {}, \"ef12\": {}, \"crossing\": {}, \"wide\": {}}}",
                t.for14, t.ef12, t.crossing, t.wide
            )
            .unwrap();
        }
        i += 1;
    }
    println!("## The 32-row ψ records: rows by class\n");
    println!(
        "| blocks | n | rows | FOR-14 | EF-12 | crossing a boundary | span too wide | all exceptions | bits a row, exceptions at 24 |"
    );
    println!("|---|--:|--:|--:|--:|--:|--:|--:|--:|");
    for (name, t) in names.iter().zip(&tallies) {
        if t.blocks > 0 {
            summary(name, t);
        }
    }
    let t = &tallies[0];
    println!("\n## Rows by their group's span (all blocks; groups inside one bucket)\n");
    println!("| span | rows |");
    println!("|---|--:|");
    let inside = t.rows - t.crossing;
    let mut cum = 0;
    for (b, &x) in t.span_log2.iter().enumerate() {
        if x > 0 {
            cum += x;
            println!(
                "| [2^{b}, 2^{}) | {} (cumulative {}) |",
                b + 1,
                pct(x, inside),
                pct(cum, inside)
            );
        }
    }
    println!("\n## F windows of 2^k rows holding a bucket boundary (all blocks)\n");
    println!(
        "| k | windows | mixed | rows in mixed windows | rows stepped (past a window's first boundary) | mixed with 1 / 2 / 3 / 4+ boundaries |"
    );
    println!("|--:|--:|--:|--:|--:|--:|");
    for (ki, k) in (4..=8).enumerate() {
        let w = &t.windows[ki];
        let p = &t.per_window[ki];
        println!(
            "| {k} | {} | {} | {} | {} | {} / {} / {} / {} |",
            w[0],
            pct(w[1], w[0]),
            pct(w[2], t.rows),
            pct(w[3], t.rows),
            p[0],
            p[1],
            p[2],
            p[3]
        );
    }
}
