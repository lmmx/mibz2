// The B5 evidence for the open kernel's port (#145 CHECKPOINT 7004759994): what it decodes,
// what it admits, and how much scratch it keeps. Beside `checks.rs`, the journal's own arrow
// audit, ported unchanged.
use super::*;
use crate::bwt::{InverseBwt, Plain};

/// The plain inverse of column `l` at row `p`: the oracle
pub(super) fn plain(l: &[u8], p: u32) -> Vec<u8> {
    let mut counts = [0u32; 256];
    for &c in l {
        counts[c as usize] += 1;
    }
    let mut out = Vec::new();
    Plain::default().invert(l, &counts, p, &mut out);
    out
}

/// The shipped instance on column `l` at row `p`, into an output poisoned first so that a
/// stale byte from an earlier block cannot pass
fn shipped(k: &mut Kernel, l: &[u8], p: u32, out: &mut Vec<u8>) {
    let r = Runs::of_column(l);
    out.clear();
    out.extend(plain(l, p).iter().map(|b| !b));
    k.run::<true, true>(r.records(), l.len(), p, LIMIT, out);
}

/// A deterministic generator (an LCG): no dependency, the same columns on every machine
pub(super) struct Lcg(pub(super) u64);
impl Lcg {
    pub(super) fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
    pub(super) fn below(&mut self, n: u32) -> u32 {
        self.next() % n.max(1)
    }
}

/// A column of `n` rows: runs of geometric-ish length around `mean` over `alphabet` bytes
pub(super) fn runs_column(g: &mut Lcg, n: usize, alphabet: u32, mean: u32) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    while l.len() < n {
        let c = g.below(alphabet) as u8;
        let w = 1 + g.below(2 * mean) as usize;
        l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
    }
    l
}

/// Every binary column through 12 rows and every ternary column through 7, at every row, by
/// the shipped instance (limit 16), exactly the plain inverse. Arbitrary columns, BWTs or not:
/// the construction's `assert!`s hold on every one of them.
#[test]
fn every_small_column_at_every_row() {
    let mut k = Kernel::default();
    let mut out = Vec::new();
    let mut cases = 0usize;
    for (base, len) in [(2usize, 12usize), (3, 7)] {
        for n in 1..=len {
            for mut z in 0..base.pow(n as u32) {
                let l: Vec<u8> = (0..n)
                    .map(|_| {
                        let c = z % base;
                        z /= base;
                        c as u8
                    })
                    .collect();
                for p in 0..n as u32 {
                    shipped(&mut k, &l, p, &mut out);
                    assert_eq!(out, plain(&l, p), "column {l:?}, row {p}");
                    cases += 1;
                }
            }
        }
    }
    // the sum of n 2^n for n 1..=12 and of n 3^n for n 1..=7
    assert_eq!(cases, 90_114 + 21_324);
}

/// Arbitrary columns of 64 to 70,000 rows, runs from 1 to beyond the record cap (127), on
/// 1 to 256 bytes, at the first, last, middle and random rows, through one kernel reused:
/// exactly the plain inverse
#[test]
fn arbitrary_columns_through_one_kernel() {
    let mut g = Lcg(0x6f70_656e);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    for &n in &[64usize, 127, 128, 129, 255, 1000, 4097, 70_000] {
        for &alphabet in &[1u32, 2, 3, 17, 256] {
            for &mean in &[1u32, 3, 16, 17, 64, 130, 400] {
                let l = runs_column(&mut g, n, alphabet, mean);
                for p in [0, n as u32 - 1, n as u32 / 2, g.below(n as u32)] {
                    shipped(&mut k, &l, p, &mut out);
                    assert_eq!(
                        out,
                        plain(&l, p),
                        "n {n}, alphabet {alphabet}, mean {mean}, row {p}"
                    );
                }
            }
        }
    }
}

/// Full-size blocks with more thick strips than `Narrow` node links serve (21,844), so the
/// elimination takes `Wide` links of itself: runs of exactly 17, and thin runs between thick
/// ones; and one kernel taking `Narrow` then `Wide` then `Narrow` again
#[test]
fn wide_links_on_full_blocks() {
    let n = 900_000usize;
    let mut g = Lcg(0x7769_6465);
    let seventeen: Vec<u8> = (0..n).map(|i| (i / 17 % 3) as u8).collect();
    let mut mixed = Vec::with_capacity(n);
    let mut c = 0u8;
    while mixed.len() < n {
        let w = if g.below(2) == 0 {
            1 + g.below(8)
        } else {
            17 + g.below(8)
        };
        c = (c + 1 + g.below(2) as u8) % 3;
        mixed.extend(std::iter::repeat(c).take(w as usize));
    }
    mixed.truncate(n);
    let narrow = runs_column(&mut g, n, 60, 12);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    for l in [&narrow, &seventeen, &mixed, &narrow] {
        for p in [0, n as u32 / 3, n as u32 - 1] {
            shipped(&mut k, l, p, &mut out);
            assert!(out == plain(l, p), "row {p}");
        }
    }
    let r = Runs::of_column(&seventeen);
    let s = k.run::<true, true>(r.records(), n, 5, LIMIT, &mut out);
    assert!(3 * s.thick + 3 > 1 << 16, "{} thick strips", s.thick);
}

/// The queues in `out` (`outq.rs`) wherever they can go wrong: blocks under one chunk (every
/// chunk an overflow chunk), blocks whose queues outgrow `out` (most retained rows), the
/// retained ranks across every multiple of 2^16 by a few rows each way (an epoch's cut, with
/// runs of 1 and of 16 rows straddling it), and thin runs longer than 16 (limits 40 and 1025)
/// across chunk and epoch ends; induced and traced, at three rows, against the plain inverse,
/// through one kernel reused (its outputs left longer or shorter by the block before)
#[test]
fn queues_in_out_overflow_and_epochs() {
    let mut g = Lcg(0x6f75_7471);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    let mut overflowed = 0;
    let mut cases: Vec<Vec<u8>> = vec![];
    for n in [1usize, 2, 17, 511, 512, 513, 1023, 1024, 1025, 3000] {
        cases.push(runs_column(&mut g, n, 3, 2));
    }
    for e in [1usize, 2, 3] {
        for d in [-17i64, -16, -1, 0, 1, 15, 16, 17] {
            let n = ((e << 16) as i64 + d) as usize;
            // every run one row: every row retained, m = n
            cases.push((0..n).map(|i| (i % 2) as u8).collect());
            // runs of 16 of three bytes: every row retained, runs straddling the cut
            cases.push((0..n).map(|i| (i / 16 % 3) as u8).collect());
        }
    }
    cases.push(runs_column(&mut g, 400_000, 40, 3));
    cases.push((0..900_000).map(|_| g.below(256) as u8).collect());
    cases.push(runs_column(&mut g, 900_000, 60, 12));
    for l in &cases {
        let n = l.len();
        let r = Runs::of_column(l);
        for p in [0, n as u32 / 2, n as u32 - 1] {
            let want = plain(l, p);
            for limit in [LIMIT, 40, 1025] {
                out.resize(g.below(2 * n as u32 + 40) as usize, 0xA5);
                k.run::<true, true>(r.records(), n, p, limit, &mut out);
                assert!(out == want, "n {n}, row {p}, limit {limit}");
                overflowed += usize::from(k.spill.overflow() > 0);
            }
            if n <= 70_000 {
                k.run::<false, true>(r.records(), n, p, LIMIT, &mut out);
                assert!(out == want, "traced: n {n}, row {p}");
            }
        }
    }
    assert!(overflowed > 0);
}

/// On a Wikidata-like full block the queues all fit in `out`: no overflow chunk
#[test]
fn queues_fit_in_out_on_full_blocks() {
    let mut g = Lcg(0x6669_7473);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    let l = runs_column(&mut g, 900_000, 60, 12);
    let r = Runs::of_column(&l);
    let s = k.run::<true, true>(r.records(), l.len(), 1234, LIMIT, &mut out);
    assert!(out == plain(&l, 1234));
    assert!(s.retained > 1 << 16, "{} retained", s.retained);
    assert_eq!(k.spill.overflow(), 0);
}

/// `OpenCells` admits a block outright exactly where `LazyCells`' census passes its two
/// record-count gates (neither the density line nor the sparse gate refuses it), on columns
/// across both lines, and opens no other but in the dense band; and its output is the plain
/// inverse's whichever route the block took
#[test]
fn open_cells_admits_on_lazy_cells_count_gates() {
    let mut g = Lcg(0x6761_7465);
    let mut cells = OpenCells::default();
    let mut lazy = LazyCells::default();
    let (mut opened, mut refused) = (0, 0);
    for _ in 0..600 {
        let n = 64 + g.below(40_000) as usize;
        let mean = 1 + g.below(200);
        let alphabet = 1 + g.below(40);
        let l = runs_column(&mut g, n, alphabet, mean);
        let p = g.below(n as u32);
        let r = Runs::of_column(&l);
        let before = lazy.counts();
        let mut want = Vec::new();
        assert!(lazy.invert_runs(&r, n, p, &mut want, Kernels::detect()));
        let after = lazy.counts();
        let gates_refuse = after.density > before.density || after.sparse > before.sparse;
        let admits = OpenCells::admits(r.records().len(), n, p);
        assert_eq!(
            admits,
            !gates_refuse,
            "n {n}, records {}",
            r.records().len()
        );
        let was = cells.opened;
        let mut out = Vec::new();
        assert!(cells.invert_runs(&r, n, p, &mut out, Kernels::detect()));
        // (the dense band, `dense.rs`: above the density line, the open kernel within its cap)
        let dense = OpenCells::cap(r.records().len(), n, p) == Some(Cap::dense(n));
        assert!(admits || dense || cells.opened == was);
        assert!(!admits || cells.opened > was);
        assert_eq!(out, want);
        assert_eq!(out, plain(&l, p));
        if admits {
            opened += 1;
        } else {
            refused += 1;
        }
    }
    assert!(
        opened > 100 && refused > 100,
        "both routes: {opened} opened, {refused} not"
    );
}

/// The gates' edges, exactly: the density line at 3 records in 20 rows, the sparse gate at
/// 1 in 64, and the kernel's entry condition
#[test]
fn open_cells_gate_edges() {
    assert!(OpenCells::admits(16, 1000, 0) && OpenCells::admits(149, 1000, 999));
    assert!(
        !OpenCells::admits(150, 1000, 0),
        "r/n 0.15 is refused, as LazyCells refuses it"
    );
    assert!(
        !OpenCells::admits(15, 1000, 0),
        "fewer than one record in 64 rows (960 < 1000)"
    );
    assert!(OpenCells::admits(16, 1024, 0) && !OpenCells::admits(15, 1024, 0));
    assert!(
        !OpenCells::admits(10, 100, 100),
        "the row outside the block"
    );
    assert!(!OpenCells::admits(0, 0, 0));
    assert!(OpenCells::admits(100_000, 900_000, 0) && !OpenCells::admits(100_000, 900_001, 0));
}

/// The scratch the kernel keeps, in bytes, across every buffer it owns
fn scratch(k: &Kernel) -> usize {
    use std::mem::size_of;
    let e = &k.endpoint;
    k.spill.scratch()
        + 4 * k.directory.capacity()
        + size_of::<Tile>() * k.tiles.capacity()
        + size_of::<Band>() * k.bands.capacity()
        + k.spelling.capacity()
        + 4 * (k.offsets.capacity() + k.edges.capacity() + k.extra.capacity())
        + 16 * k.inline16.capacity()
        + k.lens.capacity()
        + e.scratch()
}

/// Worst-case scratch: on every shape the B5 study names (Wikidata-like runs, text-like dense
/// columns, one byte, a period, a comb of thin runs between thick ones, a random column),
/// one block's scratch stays within 40 bytes a row, the worst-case bound the B5 note derives
/// (`docs/journal/2026-11-19-open-port-b5.md`); the measured worst is printed.
#[test]
fn worst_case_scratch_per_row() {
    let n = 200_000usize;
    let mut g = Lcg(0x7363_7261);
    let comb: Vec<u8> = (0..n)
        .map(|i| if i % 34 < 17 { (i / 34 % 2) as u8 } else { 2 })
        .collect();
    let shapes: Vec<(&str, Vec<u8>)> = vec![
        ("wikidata-like runs", runs_column(&mut g, n, 60, 12)),
        ("text-like", runs_column(&mut g, n, 90, 1)),
        ("one byte", vec![7; n]),
        ("period 3", (0..n).map(|i| (i % 3) as u8).collect()),
        ("thick and thin comb", comb),
        ("random", (0..n).map(|_| g.below(256) as u8).collect()),
        (
            "runs of 17",
            runs_column(&mut g, n, 2, 9)
                .iter()
                .flat_map(|&c| [c; 17])
                .take(n)
                .collect(),
        ),
    ];
    let mut worst = (0.0f64, "");
    for (name, l) in &shapes {
        let mut k = Kernel::default();
        let mut out = Vec::new();
        let p = (n / 3) as u32;
        shipped(&mut k, l, p, &mut out);
        assert_eq!(out, plain(l, p), "{name}");
        let per = scratch(&k) as f64 / n as f64;
        if per > worst.0 {
            worst = (per, name);
        }
        assert!(per <= 40.0, "{name}: {per:.1} bytes a row");
    }
    println!("worst scratch: {:.2} bytes a row ({})", worst.0, worst.1);
}

/// (Miri-sized) a few columns through the shipped instance: Miri checks every access of the
/// ported code, which has no `unsafe` of its own
#[test]
fn miri_open_kernel_small_columns() {
    let mut k = Kernel::default();
    let mut out = Vec::new();
    for l in [
        &b"banana"[..],
        b"aaaaaaaaaaaaaaaaaaab",
        b"abababababababababab",
        b"zyxzyxzyxa",
        b"q",
    ] {
        for p in 0..l.len() as u32 {
            shipped(&mut k, l, p, &mut out);
            assert_eq!(out, plain(l, p));
        }
    }
    let mut g = Lcg(9);
    let l = runs_column(&mut g, 300, 4, 20);
    shipped(&mut k, &l, 7, &mut out);
    assert_eq!(out, plain(&l, 7));
}

/// Real streams through the decoder's own route (`decode_with::<OpenCells>`) and through four
/// workers of the shipped lazy route (`StreamDecoder::workers(4, 16, k, true)`, `OpenCells`):
/// libbzip2's encoding of each stress corpus the B5 study names, the same bytes as the input
#[test]
fn stress_streams_decode_exactly() {
    use std::io::Write;
    let mut g = Lcg(0x7374_7273);
    let mut wikidata = Vec::new();
    for i in 0..9000u32 {
        let q = g.below(50);
        writeln!(
            wikidata,
            "{{\"id\":\"Q{q}\",\"type\":\"item\",\"labels\":{{\"en\":{{\"value\":\"item {}\"}}}}}}",
            i % 97
        )
        .unwrap();
    }
    let text: Vec<u8> = include_bytes!("../open.rs")
        .iter()
        .chain(include_bytes!("../bwt.rs"))
        .copied()
        .collect();
    let corpora: Vec<(&str, Vec<u8>)> = vec![
        ("wikidata-like", wikidata),
        ("text", text.repeat(3)),
        ("onebyte", vec![b'x'; 300_000]),
        (
            "pattern",
            b"abcdefghij"
                .iter()
                .copied()
                .cycle()
                .take(400_000)
                .collect(),
        ),
        (
            "runs",
            (0..300_000u32)
                .map(|i| (i / 5000 % 7) as u8 + b'a')
                .collect(),
        ),
        (
            "records",
            (0..250_000u32)
                .map(|i| {
                    if i % 64 < 8 {
                        (i / 64 % 251) as u8
                    } else {
                        b' '
                    }
                })
                .collect(),
        ),
        ("random", (0..200_000).map(|_| g.below(256) as u8).collect()),
    ];
    let k = Kernels::detect();
    for (name, input) in &corpora {
        let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(1));
        e.write_all(input).unwrap();
        let data = e.finish().unwrap();
        assert!(
            crate::decode_with::<OpenCells>(&data, k).unwrap() == *input,
            "{name}"
        );
        use std::io::Read;
        let mut out = Vec::new();
        crate::BzReader::parallel_workers(std::io::Cursor::new(data.clone()), 4, 16, k, true)
            .read_to_end(&mut out)
            .unwrap();
        assert!(out == *input, "{name}: four workers");
    }
}

/// (ignored; run with `--ignored --nocapture`) an adversarial search for the worst scratch a
/// row and the most elimination rules a row: thick strips of widths near the limit and gaps of
/// one row, on two or three bytes, the shapes that make the elimination's steps short
#[test]
#[ignore]
fn scratch_search() {
    let mut g = Lcg(0x6164_7673);
    let (mut worst, mut most_rules) = ((0.0f64, String::new()), (0.0f64, String::new()));
    for case in 0..3000 {
        let n = 20_000 + g.below(30_000) as usize;
        let alphabet = 2 + g.below(3);
        let (lo, span) = (LIMIT + 1, 1 + g.below(40));
        let gap = 1 + g.below(3);
        let mut l = Vec::with_capacity(n);
        while l.len() < n {
            let c = g.below(alphabet) as u8;
            let w = if g.below(4) == 0 {
                gap
            } else {
                lo + g.below(span)
            } as usize;
            l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
        }
        let mut k = Kernel::default();
        let r = Runs::of_column(&l);
        let p = g.below(n as u32);
        let mut out = Vec::new();
        let s = k.run::<true, true>(r.records(), n, p, LIMIT, &mut out);
        let per = scratch(&k) as f64 / n as f64;
        let rules = s.rules as f64 / n as f64;
        let what = format!(
            "case {case}: n {n}, alphabet {alphabet}, widths {lo}..{} and {gap}",
            lo + span
        );
        if per > worst.0 {
            worst = (per, what.clone());
        }
        if rules > most_rules.0 {
            most_rules = (rules, what);
        }
    }
    println!(
        "worst scratch {:.2} bytes a row ({}); most rules {:.4} a row ({})",
        worst.0, worst.1, most_rules.0, most_rules.1
    );
}
