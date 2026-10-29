//! Research only, untimed (lmmx/mibz2#79): a census of ψ for the line-blocked record
//! designs, measured before any of them is built (the "Ψ representation atlas",
//! docs/journal/2026-10-29-succinct-review.md §12–13). ψ (the T-vector's link, T[j] >> 8)
//! is the concatenation, in F order, of each byte's sorted positions in L, so G consecutive
//! rows of one bucket are G increasing positions of one byte. A record holds G rows in 2G
//! bytes (16 bits a row): a 64-bit header (base 20, byte 8, format 2, the G rows'
//! segment-start mask up to 32 bits; at G = 64 the mask moves into the payload's spare
//! bits) and a payload of 16G − 64 bits. Each group is classified as:
//! - RUN: its G positions consecutive (span G − 1): ψ(j) = base + (j mod G), no payload;
//! - FOR-k: its offsets from the base fit k = 16 − 64/G bits (14 at G = 32);
//! - EF-ℓ: Elias–Fano with ℓ = 14 − 64/G low bits (12 at G = 32) and a 2G-bit unary high
//!   part, so its high parts are at most G;
//! - an exception: it crosses a bucket boundary (no single byte), or its span is too wide.
//!   Its record points to its G entries stored whole, 32 bits each (a 20-bit ψ, the byte,
//!   the flag), so it costs 16 + 32 bits a row.
//!
//! ψ's translation runs (ψ(j + 1) = ψ(j) + 1 inside a bucket) are exactly L's runs: two
//! consecutive occurrences of byte c are adjacent in L if and only if their rows are
//! consecutive in c's bucket with consecutive ψ. So the run lengths of L are measured too.
//!
//! The groups' shapes at G = 32 (#99): each aligned 32-row group's plateaus (the breaks of
//! R(j) = ψ(j) − j inside the group, plus one; a break is a j with ψ(j + 1) ≠ ψ(j) + 1, so
//! an F-bucket boundary is one unless ψ happens to stay consecutive across it), jointly
//! with its bucket status (inside one bucket, or crossing) and its span (max − min of ψ:
//! FOR-14 below 2^14, EF-12 below 33 · 2^12, else too wide, as the classes above). Counted
//! from the real ψ, every group, including the crossing ones; d (every break in the block)
//! is counted too, to check against §17's d ÷ K₃₂.
//!
//! ```text
//! psi_census FILE.bz2 [OUT.jsonl]     the bench corpus: everything below, one JSON line a
//!                                     block in OUT
//! psi_census --inputs                 the same classes on other inputs (atlas V): random,
//!                                     a 4-letter alphabet, text, source code, an ELF
//!                                     binary, already-compressed data (the corpus's bz2
//!                                     bytes), periodic data
//! ```

#![allow(clippy::needless_range_loop)] // (a research census, indexed as written)

use std::io::Write;

use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

const GROUPS: [usize; 3] = [16, 32, 64];
const FULL: usize = 899_000;

/// Rows by class at one group size: RUN, FOR (not RUN), EF (not FOR), crossing, wide
#[derive(Default, Clone, Copy)]
struct Classes {
    rows: u64,
    run: u64,
    for_k: u64,
    ef: u64,
    crossing: u64,
    wide: u64,
    /// groups, all and by class: RUN, ordinary (FOR or EF, not RUN), exception
    groups: u64,
    run_g: u64,
    ord_g: u64,
    exc_g: u64,
}

impl Classes {
    fn add(&mut self, o: &Classes) {
        self.rows += o.rows;
        self.run += o.run;
        self.for_k += o.for_k;
        self.ef += o.ef;
        self.crossing += o.crossing;
        self.wide += o.wide;
        self.groups += o.groups;
        self.run_g += o.run_g;
        self.ord_g += o.ord_g;
        self.exc_g += o.exc_g;
    }
    fn exceptions(&self) -> u64 {
        self.crossing + self.wide
    }
    /// bits a row: 16 for every record, and 32 more a row for an exception's entries
    fn bits(&self) -> f64 {
        16.0 + 32.0 * self.exceptions() as f64 / self.rows.max(1) as f64
    }
    /// the fixed-line design's exact bytes at group size g: a 2g-byte record a group, and
    /// an exception's g entries stored whole, 4 bytes each, beside it
    fn fixed_bytes(&self, g: usize) -> u64 {
        self.groups * 2 * g as u64 + self.exc_g * 4 * g as u64
    }
    /// the two-tier design's exact bytes: an 8-byte descriptor a group (a RUN group's
    /// whole answer), a 2g-byte payload for each FOR or EF group, and an exception's g
    /// entries whole
    fn two_tier_bytes(&self, g: usize) -> u64 {
        self.groups * 8 + self.ord_g * 2 * g as u64 + self.exc_g * 4 * g as u64
    }
    fn per_row(&self, bytes: u64) -> f64 {
        8.0 * bytes as f64 / self.rows.max(1) as f64
    }
}

/// The plateau bins: 1, 2, 3–4, 5–8, 9+
const PBINS: [&str; 5] = ["1", "2", "3–4", "5–8", "9+"];
/// The joint table's columns: inside one bucket by span (FOR-14, RUN included; EF-12; too
/// wide), and crossing a bucket boundary (any span)
const COLS: [&str; 4] = [
    "one bucket, FOR-14 span",
    "one bucket, EF-12 span",
    "one bucket, too wide",
    "crossing",
];

fn pbin(plateaus: u32) -> usize {
    match plateaus {
        1 => 0,
        2 => 1,
        3..=4 => 2,
        5..=8 => 3,
        _ => 4,
    }
}

/// One group's shape: its plateaus (breaks inside the group + 1), whether it crosses an
/// F-bucket boundary, and its span (max − min of ψ over the group)
fn group_shape(psi: &[u32], f: &[u8], a: usize, end: usize) -> (u32, bool, u32) {
    let g = &psi[a..end];
    let breaks = g
        .windows(2)
        .filter(|w| w[1] != w[0].wrapping_add(1))
        .count() as u32;
    let (lo, hi) = g
        .iter()
        .fold((u32::MAX, 0), |(lo, hi), &x| (lo.min(x), hi.max(x)));
    (breaks + 1, f[a] != f[end - 1], hi - lo)
}

/// The groups' shapes at G = 32, one block or summed: the joint table of plateau bin ×
/// column, by rows and by groups; rows by exact plateau count (one bucket / crossing);
/// crossing rows by span class; and the break counts
#[derive(Default, Clone, Copy)]
struct Shape {
    rows: [[u64; 4]; 5],
    groups: [[u64; 4]; 5],
    /// rows by exact plateau count 1..=32 (index plateaus − 1): [one bucket, crossing]
    hist: [[u64; 32]; 2],
    /// crossing rows whose span is FOR-14, EF-12, too wide
    cross_span: [u64; 3],
    /// every break in the block (d), and those inside an aligned group
    d: u64,
    breaks_in: u64,
    /// groups not a translation group (a break inside, or crossing: the census's K₃₂), and
    /// groups with a break inside (ψ's own rule, a crossing group with no break excepted)
    k32: u64,
    k32_psi: u64,
}

impl Shape {
    fn add(&mut self, o: &Shape) {
        for p in 0..5 {
            for c in 0..4 {
                self.rows[p][c] += o.rows[p][c];
                self.groups[p][c] += o.groups[p][c];
            }
        }
        for b in 0..2 {
            for k in 0..32 {
                self.hist[b][k] += o.hist[b][k];
            }
        }
        for k in 0..3 {
            self.cross_span[k] += o.cross_span[k];
        }
        self.d += o.d;
        self.breaks_in += o.breaks_in;
        self.k32 += o.k32;
        self.k32_psi += o.k32_psi;
    }
    fn total_rows(&self) -> u64 {
        self.rows.iter().flatten().sum()
    }
    fn total_groups(&self) -> u64 {
        self.groups.iter().flatten().sum()
    }
}

fn shape_of(psi: &[u32], f: &[u8]) -> Shape {
    const G: usize = 32;
    let (for_span, ef_span) = (1u32 << 14, 33u32 << 12);
    let n = psi.len();
    let mut s = Shape {
        d: psi
            .windows(2)
            .filter(|w| w[1] != w[0].wrapping_add(1))
            .count() as u64,
        ..Shape::default()
    };
    for a in (0..n).step_by(G) {
        let end = (a + G).min(n);
        let rows = (end - a) as u64;
        let (plateaus, crossing, span) = group_shape(psi, f, a, end);
        let col = if crossing {
            3
        } else if span < for_span {
            0
        } else if span < ef_span {
            1
        } else {
            2
        };
        let p = pbin(plateaus);
        s.rows[p][col] += rows;
        s.groups[p][col] += 1;
        s.hist[crossing as usize][plateaus as usize - 1] += rows;
        if crossing {
            s.cross_span[(span >= for_span) as usize + (span >= ef_span) as usize] += rows;
        }
        s.breaks_in += plateaus as u64 - 1;
        s.k32 += (plateaus > 1 || crossing) as u64;
        s.k32_psi += (plateaus > 1) as u64;
    }
    s
}

/// One block's census: the classes at each group size, the F windows (k = 4..8: windows,
/// mixed, rows in mixed windows, rows stepped; mixed windows by boundary count 1/2/3/4+),
/// the rows by their L-run's length (log2), and the runs' count
struct Census {
    classes: [Classes; 3],
    windows: [[u64; 4]; 5],
    per_window: [[u64; 4]; 5],
    run_log2: [u64; 21],
    runs: u64,
    sigma: usize,
    shape: Shape,
}

fn psi_of(ll: &[u8], counts: &[u32; 256]) -> (Vec<u32>, [u32; 257]) {
    let mut start = [0u32; 257];
    for c in 0..256 {
        start[c + 1] = start[c] + counts[c];
    }
    let mut pos = start;
    let mut psi = vec![0u32; ll.len()];
    for (i, &c) in ll.iter().enumerate() {
        psi[pos[c as usize] as usize] = i as u32;
        pos[c as usize] += 1;
    }
    (psi, start)
}

fn census(ll: &[u8], counts: &[u32; 256]) -> Census {
    let n = ll.len();
    let (psi, start) = psi_of(ll, counts);
    let mut f = vec![0u8; n];
    for c in 0..256 {
        f[start[c] as usize..start[c + 1] as usize].fill(c as u8);
    }
    let mut classes = [Classes::default(); 3];
    for (gi, &g) in GROUPS.iter().enumerate() {
        let for_span = 1u32 << (16 - 64 / g);
        let ef_span = (g as u32 + 1) << (14 - 64 / g);
        let t = &mut classes[gi];
        t.rows = n as u64;
        for a in (0..n).step_by(g) {
            let end = (a + g).min(n);
            let rows = (end - a) as u64;
            t.groups += 1;
            if f[a] != f[end - 1] {
                t.crossing += rows;
                t.exc_g += 1;
                continue;
            }
            let span = psi[end - 1] - psi[a];
            if span as usize == end - a - 1 {
                t.run += rows;
                t.run_g += 1;
            } else if span < for_span {
                t.for_k += rows;
                t.ord_g += 1;
            } else if span < ef_span {
                t.ef += rows;
                t.ord_g += 1;
            } else {
                t.wide += rows;
                t.exc_g += 1;
            }
        }
    }
    let mut windows = [[0u64; 4]; 5];
    let mut per_window = [[0u64; 4]; 5];
    let bounds: Vec<usize> = (0..256)
        .filter(|&c| counts[c] > 0 && start[c] > 0)
        .map(|c| start[c] as usize)
        .collect();
    for (ki, k) in (4..=8).enumerate() {
        let w = 1usize << k;
        let nw = n.div_ceil(w);
        let mut per = vec![0u32; nw];
        let mut first = vec![usize::MAX; nw];
        for &b in &bounds {
            if b % w != 0 {
                per[b / w] += 1;
                first[b / w] = first[b / w].min(b);
            }
        }
        windows[ki][0] += nw as u64;
        for x in 0..nw {
            if per[x] > 0 {
                let wend = ((x + 1) * w).min(n);
                windows[ki][1] += 1;
                windows[ki][2] += (wend - x * w) as u64;
                windows[ki][3] += (wend - first[x]) as u64;
                per_window[ki][(per[x] as usize - 1).min(3)] += 1;
            }
        }
    }
    let mut run_log2 = [0u64; 21];
    let mut runs = 0u64;
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && ll[j] == ll[i] {
            j += 1;
        }
        let len = (j - i) as u32;
        run_log2[(31 - len.leading_zeros()).min(20) as usize] += len as u64;
        runs += 1;
        i = j;
    }
    Census {
        classes,
        windows,
        per_window,
        run_log2,
        runs,
        sigma: counts.iter().filter(|&&c| c > 0).count(),
        shape: shape_of(&psi, &f),
    }
}

fn pct(x: u64, of: u64) -> String {
    format!("{:.2}%", 100.0 * x as f64 / of.max(1) as f64)
}

fn class_row(name: &str, g: usize, t: &Classes) -> String {
    let r = t.rows;
    format!(
        "| {name} | {g} | {} | {} | {} | {} | {} | {} | {:.2} |",
        pct(t.run, r),
        pct(t.for_k, r),
        pct(t.ef, r),
        pct(t.crossing, r),
        pct(t.wide, r),
        pct(t.exceptions(), r),
        t.bits()
    )
}

fn class_header(first: &str) {
    println!(
        "| {first} | G | RUN | FOR-(16−64/G) | EF-(14−64/G) | crossing | too wide | exceptions | bits a row |"
    );
    println!("|---|--:|--:|--:|--:|--:|--:|--:|--:|");
}

fn blocks_of(data: &[u8], kern: Kernels, mut each: impl FnMut(&[u8], &[u32; 256])) {
    let mut sc = Scanner::new(SliceSource(data));
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        mtf::mtf(&symbols, &mut ll, &mut counts, kern).unwrap();
        each(&ll, &counts);
    }
}

fn quantiles(mut v: Vec<f64>) -> String {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| v[((v.len() - 1) as f64 * p).round() as usize];
    format!(
        "{:.1}% / {:.1}% / {:.1}% / {:.1}% / {:.1}%",
        q(0.0),
        q(0.1),
        q(0.5),
        q(0.9),
        q(1.0)
    )
}

fn corpus(path: &str, out: Option<&str>) {
    let data = std::fs::read(path).unwrap();
    let kern = Kernels::detect();
    let mut out = out.map(|p| std::fs::File::create(p).unwrap());
    let (mut full, mut tiny) = ([Classes::default(); 3], [Classes::default(); 3]);
    let (mut nfull, mut ntiny) = (0usize, 0usize);
    let mut windows = [[0u64; 4]; 5];
    let mut per_window = [[0u64; 4]; 5];
    let mut run_log2 = [0u64; 21];
    let (mut runs, mut rows) = (0u64, 0u64);
    let mut per_block: Vec<[f64; 6]> = Vec::new();
    let mut sigmas = Vec::new();
    let mut shape = Shape::default();
    let mut per_shape: Vec<[f64; 6]> = Vec::new();
    let mut i = 0;
    blocks_of(&data, kern, |ll, counts| {
        let c = census(ll, counts);
        let n = ll.len();
        let into = if n >= FULL { &mut full } else { &mut tiny };
        for g in 0..3 {
            into[g].add(&c.classes[g]);
        }
        if n >= FULL {
            nfull += 1;
            let t = &c.classes[1];
            let r = t.rows as f64;
            per_block.push([
                100.0 * t.run as f64 / r,
                100.0 * (t.run + t.for_k) as f64 / r,
                100.0 * t.exceptions() as f64 / r,
                t.per_row(t.fixed_bytes(32)),
                t.per_row(t.two_tier_bytes(32)),
                100.0 * (t.rows - t.run) as f64 / r,
            ]);
            sigmas.push(c.sigma as f64);
            // the joint table must reproduce the classes exactly
            let h = &c.shape;
            let col = |k: usize| h.rows.iter().map(|p| p[k]).sum::<u64>();
            assert_eq!(h.rows[0][0] + h.rows[0][1] + h.rows[0][2], t.run);
            assert_eq!(col(0), t.run + t.for_k);
            assert_eq!((col(1), col(2), col(3)), (t.ef, t.wide, t.crossing));
            assert_eq!(h.k32, t.groups - t.run_g);
            shape.add(h);
            let two = (h.rows[1][0] + h.rows[1][1] + h.rows[1][2]) as f64;
            per_shape.push([
                100.0 * two / r,
                100.0 * two / (t.rows - t.run) as f64,
                100.0 * h.rows[1][0] as f64 / r,
                h.d as f64 / h.k32 as f64,
                h.breaks_in as f64 / h.k32 as f64,
                h.breaks_in as f64 / h.k32_psi as f64,
            ]);
        } else {
            ntiny += 1;
        }
        for k in 0..5 {
            for x in 0..4 {
                windows[k][x] += c.windows[k][x];
                per_window[k][x] += c.per_window[k][x];
            }
        }
        for b in 0..21 {
            run_log2[b] += c.run_log2[b];
        }
        runs += c.runs;
        rows += n as u64;
        if let Some(o) = out.as_mut() {
            let t = &c.classes[1];
            let h = &c.shape;
            writeln!(
                o,
                "{{\"block\": {i}, \"rows\": {n}, \"sigma\": {}, \"runs\": {}, \"run\": {}, \"for14\": {}, \"ef12\": {}, \"crossing\": {}, \"wide\": {}, \"d\": {}, \"breaks_in\": {}, \"k32\": {}, \"k32_psi\": {}, \"joint_rows\": {:?}, \"joint_groups\": {:?}}}",
                c.sigma, c.runs, t.run, t.for_k, t.ef, t.crossing, t.wide,
                h.d, h.breaks_in, h.k32, h.k32_psi, h.rows, h.groups
            )
            .unwrap();
        }
        i += 1;
    });
    println!("# The ψ census ({path})\n");
    println!(
        "{} blocks: {nfull} full (≥ {FULL} rows), {ntiny} short; {:.1} M rows; σ of the full blocks {}\n",
        nfull + ntiny,
        rows as f64 / 1e6,
        {
            let mut s = sigmas.clone();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            format!("{}–{}", s[0], s[s.len() - 1])
        }
    );
    println!(
        "## 1. Rows by record class, at each group size (16 bits a row; exceptions at 16 + 32)\n"
    );
    class_header("blocks");
    for (gi, &g) in GROUPS.iter().enumerate() {
        println!("{}", class_row("full", g, &full[gi]));
    }
    for (gi, &g) in GROUPS.iter().enumerate() {
        println!("{}", class_row("short", g, &tiny[gi]));
    }
    println!("\n## 2. Per full block at G = 32: min / p10 / median / p90 / max\n");
    println!("| measure | across the {nfull} full blocks |");
    println!("|---|--:|");
    let col = |k: usize| per_block.iter().map(|b| b[k]).collect::<Vec<f64>>();
    println!("| RUN rows | {} |", quantiles(col(0)));
    println!("| RUN or FOR-14 rows | {} |", quantiles(col(1)));
    println!("| exception rows | {} |", quantiles(col(2)));
    let fmt = |mut v: Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q = |p: f64| v[((v.len() - 1) as f64 * p).round() as usize];
        format!(
            "{:.2} / {:.2} / {:.2} / {:.2} / {:.2}",
            q(0.0),
            q(0.1),
            q(0.5),
            q(0.9),
            q(1.0)
        )
    };
    println!("| fixed-line bits a row (exact) | {} |", fmt(col(3)));
    println!("| two-tier bits a row (exact) | {} |", fmt(col(4)));
    println!(
        "| two-tier steps taking a second dependent load (rows not in a RUN group) | {} |",
        quantiles(col(5))
    );
    let t = &full[1];
    println!("\n## 2a. The designs' exact sizes, the full blocks at G = 32\n");
    println!("| design | bytes, all full blocks | bits a row | a full block's table |");
    println!("|---|--:|--:|--:|");
    let per_block_mb = |b: u64| b as f64 / nfull as f64 / 1e6;
    for (name, b) in [
        ("today's T-vector (32 bits)", t.rows * 4),
        ("24-bit links", t.rows * 3),
        ("fixed-line records", t.fixed_bytes(32)),
        ("two-tier (descriptors + payloads)", t.two_tier_bytes(32)),
    ] {
        println!(
            "| {name} | {:.1} MB | {:.2} | {:.3} MB |",
            b as f64 / 1e6,
            t.per_row(b),
            per_block_mb(b)
        );
    }
    println!(
        "\nThe two-tier design's parts, a full block: descriptors {:.0} KB, payloads {:.0} KB, exception entries {:.0} KB; groups: {} RUN, {} FOR or EF, {} exceptions",
        t.groups as f64 * 8.0 / nfull as f64 / 1e3,
        t.ord_g as f64 * 64.0 / nfull as f64 / 1e3,
        t.exc_g as f64 * 128.0 / nfull as f64 / 1e3,
        t.run_g,
        t.ord_g,
        t.exc_g
    );
    println!(
        "\n## 3. L's runs (= ψ's translation runs): {runs} runs over {:.1} M rows, n/r = {:.2}\n",
        rows as f64 / 1e6,
        rows as f64 / runs as f64
    );
    println!("| run length | rows | cumulative |");
    println!("|---|--:|--:|");
    let mut cum = 0;
    for (k, &x) in run_log2.iter().enumerate() {
        if x > 0 {
            cum += x;
            println!(
                "| [2^{k}, 2^{}) | {} | {} |",
                k + 1,
                pct(x, rows),
                pct(cum, rows)
            );
        }
    }
    println!("\n## 4. F windows of 2^k rows holding a bucket boundary (all blocks)\n");
    println!(
        "| k | windows | mixed | rows in mixed windows | rows stepped | mixed with 1 / 2 / 3 / 4+ boundaries |"
    );
    println!("|--:|--:|--:|--:|--:|--:|");
    for (ki, k) in (4..=8).enumerate() {
        let w = &windows[ki];
        let p = &per_window[ki];
        println!(
            "| {k} | {} | {} | {} | {} | {} / {} / {} / {} |",
            w[0],
            pct(w[1], w[0]),
            pct(w[2], rows),
            pct(w[3], rows),
            p[0],
            p[1],
            p[2],
            p[3]
        );
    }
    shape_report(&shape, &per_shape, nfull);
}

fn spread(mut v: Vec<f64>, dp: usize) -> String {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| v[((v.len() - 1) as f64 * p).round() as usize];
    format!(
        "{:.dp$} / {:.dp$} / {:.dp$} / {:.dp$} / {:.dp$}",
        q(0.0),
        q(0.1),
        q(0.5),
        q(0.9),
        q(1.0)
    )
}

/// §5–6: the groups' shapes at G = 32, the full blocks (#99), and their cross-checks
fn shape_report(h: &Shape, per: &[[f64; 6]], nfull: usize) {
    let (rows, groups) = (h.total_rows(), h.total_groups());
    let nontr: u64 = rows - h.rows[0][..3].iter().sum::<u64>();
    println!(
        "\n## 5. The groups' shapes at G = 32: plateaus × bucket × span, the {nfull} full blocks\n"
    );
    println!(
        "{rows} rows, {groups} groups; non-translation rows (a break inside, or crossing) {}\n",
        pct(nontr, rows)
    );
    for (k, what, m, of) in [
        (1, "rows, as a share of all rows", &h.rows, rows),
        (2, "groups, as a share of all groups", &h.groups, groups),
    ] {
        println!("### 5.{k} By {what}\n");
        println!("| plateaus | {} | all |", COLS.join(" | "));
        println!("|---|--:|--:|--:|--:|--:|");
        for (p, name) in PBINS.iter().enumerate() {
            let cells: Vec<String> = m[p].iter().map(|&x| pct(x, of)).collect();
            println!(
                "| {name} | {} | {} |",
                cells.join(" | "),
                pct(m[p].iter().sum(), of)
            );
        }
        let cells: Vec<String> = (0..4)
            .map(|c| pct(m.iter().map(|p| p[c]).sum(), of))
            .collect();
        println!("| all | {} | 100.00% |\n", cells.join(" | "));
    }
    let two: u64 = h.rows[1][..3].iter().sum();
    println!("### 5.3 The two-plateau share\n");
    println!(
        "Two-plateau groups inside one bucket: {} of all rows, {} of the non-translation rows; of them FOR-14 span {}, EF-12 {}, too wide {} (of all rows)\n",
        pct(two, rows),
        pct(two, nontr),
        pct(h.rows[1][0], rows),
        pct(h.rows[1][1], rows),
        pct(h.rows[1][2], rows)
    );
    println!("| per full block | min / p10 / median / p90 / max |");
    println!("|---|--:|");
    let col = |k: usize| per.iter().map(|b| b[k]).collect::<Vec<f64>>();
    println!(
        "| two plateaus, one bucket, % of all rows | {} |",
        spread(col(0), 2)
    );
    println!(
        "| two plateaus, one bucket, % of non-translation rows | {} |",
        spread(col(1), 2)
    );
    println!(
        "| two plateaus, one bucket, FOR-14 span, % of all rows | {} |",
        spread(col(2), 2)
    );
    println!("\n### 5.4 Rows by exact plateau count (% of all rows)\n");
    println!("| plateaus | one bucket | crossing | cumulative |");
    println!("|--:|--:|--:|--:|");
    let mut cum = 0;
    for k in 1..=32 {
        let (a, b) = (h.hist[0][k - 1], h.hist[1][k - 1]);
        cum += a + b;
        if a + b > 0 {
            println!(
                "| {k} | {} | {} | {} |",
                pct(a, rows),
                pct(b, rows),
                pct(cum, rows)
            );
        }
    }
    println!(
        "\nCrossing rows by span (max − min of ψ): FOR-14 {} ({} rows), EF-12 {} ({} rows), too wide {} ({} rows) (of all rows)",
        pct(h.cross_span[0], rows),
        h.cross_span[0],
        pct(h.cross_span[1], rows),
        h.cross_span[1],
        pct(h.cross_span[2], rows),
        h.cross_span[2]
    );
    println!("\n## 6. Cross-checks (the full blocks)\n");
    println!(
        "- the joint table's columns and plateau-1 cells equal the census's classes (RUN, FOR-14, EF-12, too wide, crossing) and K₃₂ on every block (asserted)"
    );
    println!(
        "- d, every break: {}; inside an aligned group: {} ({}); at a group edge (j ≡ 31 mod 32): {}",
        h.d,
        h.breaks_in,
        pct(h.breaks_in, h.d),
        h.d - h.breaks_in
    );
    println!(
        "- K₃₂ (census rule): {}; groups with a break inside (ψ's rule): {}\n",
        h.k32, h.k32_psi
    );
    println!("| per full block | min / p10 / median / p90 / max |");
    println!("|---|--:|");
    println!("| d ÷ K₃₂ | {} |", spread(col(3), 2));
    println!("| breaks inside a group ÷ K₃₂ | {} |", spread(col(4), 2));
    println!(
        "| breaks inside a group ÷ groups with a break inside | {} |",
        spread(col(5), 2)
    );
}

/// Atlas V: the same classes on other inputs, each compressed by libbzip2 at level 9 here
fn inputs() {
    let mut x = 0x9e37_79b9_7f4a_7c15u64;
    let mut rnd = move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    let mut cases: Vec<(String, Vec<u8>)> = Vec::new();
    cases.push((
        "random bytes".into(),
        (0..2_700_000).map(|_| rnd() as u8).collect(),
    ));
    cases.push((
        "a 4-letter alphabet, random".into(),
        (0..2_700_000)
            .map(|_| b"ACGT"[(rnd() % 4) as usize])
            .collect(),
    ));
    let read_tree = |root: &str, exts: &[&str]| -> Vec<u8> {
        let mut out = Vec::new();
        let mut stack = vec![std::path::PathBuf::from(root)];
        while let Some(p) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&p) else {
                continue;
            };
            let mut es: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            es.sort();
            for e in es {
                if e.is_dir() {
                    stack.push(e);
                } else if exts.iter().any(|x| e.extension().is_some_and(|y| y == *x)) {
                    out.extend(std::fs::read(&e).unwrap_or_default());
                }
            }
        }
        out
    };
    cases.push((
        "English prose (docs/journal/*.md)".into(),
        read_tree("docs/journal", &["md"]),
    ));
    cases.push(("Rust source (src, examples, tests)".into(), {
        let mut v = read_tree("src", &["rs"]);
        v.extend(read_tree("examples", &["rs"]));
        v.extend(read_tree("tests", &["rs"]));
        v
    }));
    if let Ok(exe) = std::env::current_exe().and_then(std::fs::read) {
        cases.push(("an ELF binary (this program)".into(), exe));
    }
    cases.push((
        "already compressed (the corpus's own bz2 bytes, its first 2.7 MB)".into(),
        std::fs::read("corpora/wikidata.bz2")
            .map(|v| v[..v.len().min(2_700_000)].to_vec())
            .unwrap_or_default(),
    ));
    cases.push(("periodic, period 1000".into(), {
        let p: Vec<u8> = (0..1000).map(|_| rnd() as u8).collect();
        p.repeat(2700)
    }));
    let kern = Kernels::detect();
    let mut designs = Vec::new();
    println!("# The ψ census on other inputs (atlas V; libbzip2 level 9; all blocks)\n");
    class_header("input (MB)");
    for (name, data) in &cases {
        if data.is_empty() {
            continue;
        }
        let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(9));
        e.write_all(data).unwrap();
        let z = e.finish().unwrap();
        let mut t = [Classes::default(); 3];
        let (mut runs, mut rows) = (0u64, 0u64);
        blocks_of(&z, kern, |ll, counts| {
            let c = census(ll, counts);
            for g in 0..3 {
                t[g].add(&c.classes[g]);
            }
            runs += c.runs;
            rows += ll.len() as u64;
        });
        let label = format!(
            "{name} ({:.1} MB, n/r {:.1})",
            data.len() as f64 / 1e6,
            rows as f64 / runs.max(1) as f64
        );
        println!("{}", class_row(&label, 32, &t[1]));
        let c = &t[1];
        designs.push(format!(
            "| {label} | {:.2} | {:.2} | {} |",
            c.per_row(c.fixed_bytes(32)),
            c.per_row(c.two_tier_bytes(32)),
            pct(c.rows - c.run, c.rows)
        ));
    }
    println!("\n## The designs' exact sizes on the same inputs (G = 32)\n");
    println!("| input | fixed-line bits a row | two-tier bits a row | two-tier steps with a second dependent load |");
    println!("|---|--:|--:|--:|");
    for d in designs {
        println!("{d}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--inputs") => inputs(),
        Some(path) => corpus(path, args.get(2).map(String::as_str)),
        None => panic!("psi_census FILE.bz2 [OUT.jsonl] | psi_census --inputs"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// a group of three plateaus inside one bucket, padded to 32 rows by its last plateau
    #[test]
    fn plateaus_of_a_hand_made_group() {
        let mut psi = vec![500, 501, 502, 510, 511, 512, 530, 531];
        while psi.len() < 32 {
            psi.push(psi[psi.len() - 1] + 1);
        }
        let f = vec![7u8; 32];
        assert_eq!(group_shape(&psi, &f, 0, 32), (3, false, 555 - 500));
        // a translation group: one plateau
        let run: Vec<u32> = (100..132).collect();
        assert_eq!(group_shape(&run, &f, 0, 32), (1, false, 31));
        // a bucket boundary is a break unless ψ stays consecutive across it
        let mut g = f.clone();
        g[16..].fill(8);
        assert_eq!(group_shape(&run, &g, 0, 32), (1, true, 31));
        let mut jump = run.clone();
        jump[16..].iter_mut().for_each(|x| *x -= 90);
        assert_eq!(group_shape(&jump, &g, 0, 32), (2, true, 115 - 26));
        // a short last group
        assert_eq!(group_shape(&psi, &f, 24, 30), (1, false, 5));
        // the block's counts: d and the in-group breaks differ by the break at the group edge
        let mut two = psi.clone();
        two.extend(900..932);
        let s = shape_of(&two, &[7u8; 64]);
        assert_eq!((s.d, s.breaks_in, s.k32, s.k32_psi), (3, 2, 1, 1));
        assert_eq!(
            (s.rows[pbin(3)][0], s.rows[0][0], s.groups[2][0]),
            (32, 32, 1)
        );
    }
}
