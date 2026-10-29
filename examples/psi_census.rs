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
}

impl Classes {
    fn add(&mut self, o: &Classes) {
        self.rows += o.rows;
        self.run += o.run;
        self.for_k += o.for_k;
        self.ef += o.ef;
        self.crossing += o.crossing;
        self.wide += o.wide;
    }
    fn exceptions(&self) -> u64 {
        self.crossing + self.wide
    }
    /// bits a row: 16 for every record, and 32 more a row for an exception's entries
    fn bits(&self) -> f64 {
        16.0 + 32.0 * self.exceptions() as f64 / self.rows.max(1) as f64
    }
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
            if f[a] != f[end - 1] {
                t.crossing += rows;
                continue;
            }
            let span = psi[end - 1] - psi[a];
            if span as usize == end - a - 1 {
                t.run += rows;
            } else if span < for_span {
                t.for_k += rows;
            } else if span < ef_span {
                t.ef += rows;
            } else {
                t.wide += rows;
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
    let mut per_block: Vec<[f64; 4]> = Vec::new();
    let mut sigmas = Vec::new();
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
                t.bits(),
            ]);
            sigmas.push(c.sigma as f64);
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
            writeln!(
                o,
                "{{\"block\": {i}, \"rows\": {n}, \"sigma\": {}, \"runs\": {}, \"run\": {}, \"for14\": {}, \"ef12\": {}, \"crossing\": {}, \"wide\": {}}}",
                c.sigma, c.runs, t.run, t.for_k, t.ef, t.crossing, t.wide
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
    let bits = col(3);
    let mut b = bits.clone();
    b.sort_by(|a, c| a.partial_cmp(c).unwrap());
    println!(
        "| bits a row | {:.2} / {:.2} / {:.2} / {:.2} / {:.2} |",
        b[0],
        b[b.len() / 10],
        b[b.len() / 2],
        b[b.len() * 9 / 10],
        b[b.len() - 1]
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
