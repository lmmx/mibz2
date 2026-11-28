// The dense band (`DENSE`, `Cap::dense`): what `OpenCells` admits above `LazyCells`' density
// line, the first pass's cap and its refusal, exactly at its edges.
use super::b5::{plain, Lcg};
use super::*;

/// A column of `n` rows of Wikidata's dense kind: short runs (1 or 2 rows) and, a run in
/// `long` (more short runs with a larger `long`), long ones (17 to 400 rows), over 40 bytes.
/// Near `long` 40 it lies in the dense band within the cap (r/n about 0.155, 0.22 n retained);
/// from about 60 it is past the cap, and from about 100 past the band.
pub(super) fn dense_column(g: &mut Lcg, n: usize, long: u32) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    let mut c = 0u8;
    while l.len() < n {
        c = (c + 1 + g.below(39) as u8) % 40;
        let w = if g.below(long) == 0 {
            17 + g.below(384)
        } else {
            1 + g.below(2)
        } as usize;
        l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
    }
    l
}

/// The two-tier record-count gates' edges: sparse below 1/64, outright `Cap::NONE` for
/// `1/64 <= r/n < 0.10`, capped `Cap::dense(n)` (`m <= 0.27 * n`) for `0.10 <= r/n < 0.21`
/// (retaining all 5 `wikidata` dense blocks at `r/n <= 0.206`, `m/n <= 0.257`), `None`
/// (`Segmented`) for `r/n >= 0.21`, and the kernel's entry condition
#[test]
fn dense_band_gate_edges() {
    type O = OpenCells;
    assert_eq!(
        O::cap(15, 1000, 0),
        None,
        "the sparse gate holds (r/n < 1/64)"
    );
    assert_eq!(
        O::cap(16, 1000, 0),
        Some(Cap::NONE),
        "1/64 <= r/n < 0.10 outright"
    );
    assert_eq!(O::cap(99, 1000, 999), Some(Cap::NONE), "r/n 0.099 outright");
    assert_eq!(
        O::cap(100, 1000, 0),
        Some(Cap::dense(1000)),
        "r/n 0.10 enters capped band"
    );
    assert_eq!(
        O::cap(209, 1000, 999),
        Some(Cap::dense(1000)),
        "r/n 0.209 in capped band (covers wikidata's 5 dense blocks up to 0.206)"
    );
    assert_eq!(
        O::cap(210, 1000, 0),
        None,
        "r/n 0.21 goes straight to Segmented"
    );
    assert_eq!(
        O::cap(280, 1000, 0),
        None,
        "r/n 0.28 goes straight to Segmented"
    );
    assert_eq!(O::cap(120, 1000, 1000), None, "the row outside the block");
    assert_eq!(O::cap(0, 0, 0), None);
    assert_eq!(O::cap(120_000, 900_001, 0), None);
    assert_eq!(
        O::cap(120_000, 900_000, 0),
        Some(Cap {
            m: 243_000,
            strips: 16_384,
        })
    );
    assert_eq!(Cap::dense(1_000).m, 270, "exact 0.27 * n cap at n=1,000");
    assert_eq!(
        Cap::dense(800_000).m,
        216_000,
        "exact 0.27 * n cap at n=800,000"
    );
    assert_eq!(
        Cap::dense(900_000).m,
        243_000,
        "exact 0.27 * n cap at n=900,000 (> wikidata max 231,600)"
    );
    assert_eq!(ForcedOpenCells::cap(500, 1000, 0), Some(Cap::NONE));
}

/// `try_run`'s cap, exactly: a block is taken at its own retained rows and thick strips and
/// refused one below either, whichever of the queues' cold path (`over`) or the pass's end
/// sees it; a refused block leaves the kernel to decode the next exactly, and takes no chunk
/// past the cap
#[test]
fn cap_is_exact_and_refusal_leaves_no_state() {
    let mut g = Lcg(0x6361_7073);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    for (n, long) in [(3_000usize, 6u32), (70_000, 4), (300_000, 8), (900_000, 5)] {
        let l = dense_column(&mut g, n, long);
        let r = Runs::of_column(&l);
        for p in [0, n as u32 / 2, n as u32 - 1] {
            let want = plain(&l, p);
            let s = k.run::<true, true>(r.records(), n, p, LIMIT, &mut out);
            assert!(out == want);
            let (m, ns) = (s.retained as u32, s.thick);
            for cap in [
                Cap { m, strips: ns },
                Cap::NONE,
                Cap {
                    m: u32::MAX,
                    strips: ns,
                },
            ] {
                out.clear();
                let t = k.try_run::<true, true>(r.records(), n, p, LIMIT, cap, &mut out);
                assert!(t.is_some() && out == want, "n {n}, at the cap {cap:?}");
            }
            for cap in [
                Cap {
                    m: m - 1,
                    strips: ns,
                },
                Cap {
                    m,
                    strips: ns.wrapping_sub(1),
                },
                Cap { m: 0, strips: 0 },
                Cap {
                    m: m / 2,
                    strips: ns,
                },
            ] {
                if cap.strips == usize::MAX {
                    continue;
                }
                let previous = k.edges.capacity();
                let t = k.try_run::<true, true>(r.records(), n, p, LIMIT, cap, &mut out);
                assert!(t.is_none(), "n {n}: past the cap {cap:?} ({m}, {ns})");
                assert!(k.edges.capacity() <= previous.max(cap.strips.saturating_mul(3)));
                // every chunk the pass took, a whole chunk of ranks within the cap, or the
                // last of a byte's, partly filled (at most one a byte)
                assert!(
                    k.spill.taken as usize <= cap.m as usize / 512 + 1 + 256,
                    "{} chunks under a cap of {}",
                    k.spill.taken as usize,
                    cap.m
                );
                // the kernel decodes the next block exactly after a refusal
                let s2 = k.run::<true, true>(r.records(), n, p, LIMIT, &mut out);
                assert!(out == want && s2.retained == s.retained);
                assert!(k
                    .try_run::<true, true>(r.records(), n, p, LIMIT, cap, &mut out)
                    .is_none());
                let other = dense_column(&mut g, 5_000, 6);
                let ro = Runs::of_column(&other);
                k.run::<true, true>(ro.records(), other.len(), 7, LIMIT, &mut out);
                assert!(out == plain(&other, 7));
            }
        }
    }
}

/// The sticky refusal in the queues' cold path: once past the cap the pass takes no chunk,
/// whatever the block's length, so a refused full block's queues stay within the cap's (no
/// overflow chunk, and every chunk within `out`)
#[test]
fn refusal_is_sticky_in_the_queues() {
    // every row retained (runs of one): m = n, far past any cap
    let n = 900_000;
    let l: Vec<u8> = (0..n).map(|i| (i % 7) as u8).collect();
    let r = Runs::of_column(&l);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    for m in [0u32, 1, 511, 512, 513, 65_535, 65_536, 200_000, 256_000] {
        let cap = Cap { m, strips: 16_384 };
        assert!(k
            .try_run::<true, true>(r.records(), n, 3, LIMIT, cap, &mut out)
            .is_none());
        assert_eq!(k.spill.overflow(), 0);
        assert!(
            k.spill.taken as usize <= m as usize / 512 + 7,
            "{} chunks under a cap of {m}",
            k.spill.taken as usize
        );
    }
}

/// `OpenCells` on columns spanning all three bands (`r/n < 0.10`, `0.10 <= r/n < 0.21`,
/// `r/n >= 0.21`): opened outright below 0.10, gated by `Cap::dense` (`m <= 0.27 * n`) in
/// `[0.10, 0.21)`, and routed straight to `Segmented` from 0.21; the plain inverse's output
/// whatever the route, through one decoder reused
#[test]
fn dense_band_routes_and_decodes_exactly() {
    let mut g = Lcg(0x6465_6e73);
    let mut cells = OpenCells::default();
    let (mut outright, mut banded, mut capped, mut refused) = (0, 0, 0, 0);
    for i in 0..240 {
        let n = if i % 12 == 0 {
            900_000
        } else {
            2_000 + g.below(120_000) as usize
        };
        let l = if i % 4 == 0 {
            // Moderate short runs (1..=6) with periodic long runs: r/n in [0.10, 0.21) with m/n > 0.27
            let mut col = Vec::with_capacity(n);
            let mut c = 0u8;
            let long = 20 + g.below(25);
            while col.len() < n {
                c = (c + 1 + g.below(39) as u8) % 40;
                let w = if g.below(long) == 0 {
                    20 + g.below(240)
                } else {
                    2 + g.below(5)
                } as usize;
                col.extend(std::iter::repeat_n(c, w.min(n - col.len())));
            }
            col
        } else {
            let long = 10 + g.below(70);
            dense_column(&mut g, n, long)
        };
        let p = g.below(n as u32);
        let r = Runs::of_column(&l);
        let recs = r.records().len();
        let (was, was_capped) = (cells.opened, cells.capped);
        let mut out = vec![0x5A; g.below(2 * n as u32) as usize];
        assert!(cells.invert_runs(&r, n, p, &mut out, Kernels::detect()));
        assert!(out == plain(&l, p), "n {n}, records {recs}");
        let opened = cells.opened > was;
        match OpenCells::cap(recs, n, p) {
            Some(Cap::NONE) => {
                assert!(opened);
                outright += 1;
            }
            Some(cap) => {
                let s = Kernel::default().run::<true, true>(r.records(), n, p, LIMIT, &mut out);
                let within = s.retained as u32 <= cap.m && s.thick <= cap.strips;
                assert_eq!(
                    opened, within,
                    "n {n}: {} retained, {} strips",
                    s.retained, s.thick
                );
                assert_eq!(cells.capped > was_capped, !within);
                if within {
                    banded += 1;
                } else {
                    capped += 1;
                }
            }
            None => {
                assert!(!opened);
                refused += 1;
            }
        }
    }
    assert!(
        outright > 5 && banded > 5 && capped > 5 && refused > 5,
        "outright {outright}, banded {banded}, capped {capped}, refused {refused}"
    );
}

/// `within_cap` matches the uncapped construction's `(retained, thick)` counts at exact boundary
/// caps across dense columns and marked-row positions (start, interior and end of a thick run,
/// including cap-127 split records), and refusing a capped-band block skips Open's first pass.
#[test]
fn within_cap_matches_construction_and_skips_first_pass() {
    let mut g = Lcg(0x7072_6563);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    for (n, long) in [(3_000usize, 6u32), (30_000, 12), (100_000, 40)] {
        let l = dense_column(&mut g, n, long);
        let r = Runs::of_column(&l);
        // Find a thick maximal run (and one spanning a cap-127 split if present) to place `p`
        // at its start, interior and end.
        let mut marks = vec![0u32, n as u32 / 2, n as u32 - 1];
        maximal(r.records(), |_, pos, width| {
            if width > LIMIT && marks.len() < 9 {
                marks.push(pos);
                marks.push(pos + width / 2);
                marks.push(pos + width - 1);
            }
        });
        for p in marks {
            let s = k.run::<true, true>(r.records(), n, p, LIMIT, &mut out);
            let (m, ns) = (s.retained as u32, s.thick);
            assert!(within_cap(r.records(), p, LIMIT, Cap { m, strips: ns }));
            assert!(!within_cap(
                r.records(),
                p,
                LIMIT,
                Cap {
                    m: m - 1,
                    strips: ns
                }
            ));
            if ns > 0 {
                assert!(!within_cap(
                    r.records(),
                    p,
                    LIMIT,
                    Cap { m, strips: ns - 1 }
                ));
            }
        }
    }

    // A refused `[0.15, 0.21)` dense-band block (`!lazy_counts_admit`, `m > 0.27 * n`) skips
    // `Kernel::try_run` entirely via `within_cap`, while a refused `[0.10, 0.15)` block skips
    // `within_cap` and refuses cleanly inside Pass 1 with identical route and output.
    let mut cells = OpenCells::default();
    let mut tested_dense_skip = false;
    let mut tested_lazy_pass1_refusal = false;
    for i in 0..120 {
        let n = 40_000usize;
        let mut l = Vec::with_capacity(n);
        let mut c = 0u8;
        let long = 18 + (i % 25);
        let short_base = if i % 2 == 0 { 2 } else { 4 };
        while l.len() < n {
            c = (c + 1 + g.below(39) as u8) % 40;
            let w = if g.below(long) == 0 {
                20 + g.below(180)
            } else {
                short_base + g.below(3)
            } as usize;
            l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
        }
        let r = Runs::of_column(&l);
        let recs = r.records().len();
        if let Some(cap) = OpenCells::cap(recs, l.len(), 7) {
            if cap.m != u32::MAX && !within_cap(r.records(), 7, LIMIT, cap) {
                let lazy = crate::cells::lazy_counts_admit(recs, l.len());
                if !lazy && !tested_dense_skip {
                    let before_taken = cells.open.spill.taken;
                    let before_capped = cells.capped;
                    let before_opened = cells.opened;
                    out.clear();
                    assert!(cells.invert_runs(&r, l.len(), 7, &mut out, Kernels::detect()));
                    assert!(out == plain(&l, 7));
                    assert_eq!(cells.capped, before_capped + 1);
                    assert_eq!(cells.opened, before_opened);
                    assert_eq!(
                        cells.open.spill.taken, before_taken,
                        "dense-band precheck refusal must not enter Open's first pass"
                    );
                    tested_dense_skip = true;
                } else if lazy && !tested_lazy_pass1_refusal {
                    let before_capped = cells.capped;
                    let before_opened = cells.opened;
                    out.clear();
                    assert!(cells.invert_runs(&r, l.len(), 7, &mut out, Kernels::detect()));
                    assert!(out == plain(&l, 7));
                    assert_eq!(cells.capped, before_capped + 1);
                    assert_eq!(cells.opened, before_opened);
                    assert!(
                        cells.open.spill.taken as usize <= cap.m as usize / 512 + 1 + 256,
                        "[0.10, 0.15) Pass-1 refusal took {} chunks under cap {}",
                        cells.open.spill.taken,
                        cap.m
                    );
                    tested_lazy_pass1_refusal = true;
                }
                if tested_dense_skip && tested_lazy_pass1_refusal {
                    break;
                }
            }
        }
    }
    assert!(
        tested_dense_skip,
        "expected at least one refused [0.15, 0.21) dense-band block"
    );
    assert!(
        tested_lazy_pass1_refusal,
        "expected at least one refused [0.10, 0.15) lazy-admitted block"
    );
}

/// Deterministic route-identity census against v0.5.17: every block across generated bzip2
/// streams spanning the 6 synthetic bench corpora (`onebyte`, `pattern`, `random`, `records`,
/// `runs`, `text`) plus a structured JSON-like stream takes the exact same route (open kernel vs
/// `Segmented`) as v0.5.17's `within_cap` oracle, with byte-identical output.
#[test]
fn route_census_matches_v0517_across_corpora() {
    use crate::stream::{Event, Scanner, SliceSource};
    use bzip2::write::BzEncoder;
    use bzip2::Compression;
    use std::io::Write;

    fn bz2_compress(raw: &[u8]) -> Vec<u8> {
        let mut enc = BzEncoder::new(Vec::new(), Compression::best());
        enc.write_all(raw).unwrap();
        enc.finish().unwrap()
    }

    // Compact versions of the 6 generated bench corpora plus a structured JSON-like stream
    let mut g = Lcg(0x0517_c0de);
    let onebyte = vec![b'A'; 200_000];
    let pattern: Vec<u8> = b"alpha,beta,gamma,delta,0123456789\n"
        .iter()
        .copied()
        .cycle()
        .take(200_000)
        .collect();
    let random: Vec<u8> = (0..120_000).map(|_| g.below(256) as u8).collect();
    let mut records = Vec::with_capacity(120_000);
    for i in 0..3_000u32 {
        records.extend_from_slice(
            format!("id={i:06} val={:08x} tag=row\n", g.below(u32::MAX)).as_bytes(),
        );
    }
    let mut runs = Vec::with_capacity(180_000);
    while runs.len() < 180_000 {
        let b = b'a' + (g.below(12) as u8);
        let w = 1 + g.below(40) as usize;
        runs.extend(std::iter::repeat_n(b, w.min(180_000 - runs.len())));
    }
    let words = [
        "the ",
        "open ",
        "kernel ",
        "decodes ",
        "bzip2 ",
        "blocks ",
        "in ",
        "parallel ",
        "with ",
        "bounded ",
        "memory ",
        "and ",
        "exact ",
        "output ",
        "every ",
        "round.\n",
    ];
    let mut text = Vec::with_capacity(260_000);
    while text.len() < 260_000 {
        text.extend_from_slice(words[g.below(words.len() as u32) as usize].as_bytes());
    }
    let mut json = Vec::with_capacity(400_000);
    for i in 0..3_500u32 {
        json.extend_from_slice(
            format!(
                "{{\"type\":\"item\",\"id\":\"Q{}\",\"labels\":{{\"en\":{{\"language\":\"en\",\"value\":\"entity {}\"}}}},\"claims\":{{\"P31\":[{{\"rank\":\"normal\",\"id\":\"Q{}$1\"}}]}}}}\n",
                1000 + i,
                i % 97,
                1000 + (i % 503)
            )
            .as_bytes(),
        );
    }

    let corpora: [(&str, Vec<u8>); 7] = [
        ("onebyte", bz2_compress(&onebyte)),
        ("pattern", bz2_compress(&pattern)),
        ("random", bz2_compress(&random)),
        ("records", bz2_compress(&records)),
        ("runs", bz2_compress(&runs)),
        ("text", bz2_compress(&text)),
        ("json", bz2_compress(&json)),
    ];

    let k = Kernels::detect();
    let mut cells = OpenCells::default();
    let mut r = Runs::default();
    let mut out = Vec::new();
    let mut total_blocks = 0usize;
    let mut total_opened = 0usize;
    for (name, bz2) in &corpora {
        let mut sc = Scanner::new(SliceSource(bz2));
        while let Event::Block { symbols: s, .. } = sc.next(Vec::new()).unwrap() {
            total_blocks += 1;
            if crate::runs::takes_runs(&s) {
                let n = crate::runs::front(&s, &mut r, k).unwrap();
                let recs = r.records();
                let p = s.orig_ptr;
                let v0517_opens = OpenCells::cap(recs.len(), n, p)
                    .is_some_and(|cap| cap.m == u32::MAX || within_cap(recs, p, LIMIT, cap));
                let before_opened = cells.opened;
                out.clear();
                assert!(cells.invert_runs(&r, n, p, &mut out, k));
                let head_opens = cells.opened > before_opened;
                assert_eq!(
                    head_opens, v0517_opens,
                    "corpus {name}: route mismatch (head_opens={head_opens}, v0517_opens={v0517_opens}, n={n}, recs={})",
                    recs.len()
                );
                total_opened += usize::from(head_opens);
            }
        }
        assert_eq!(
            crate::decode_with::<OpenCells>(bz2, k).unwrap(),
            crate::decode_with::<Segmented>(bz2, k).unwrap(),
            "corpus {name}: output bytes differ from Segmented"
        );
    }
    assert!(total_blocks >= 7 && total_opened >= 1);

    if let Ok(dir) = std::env::var("MIBZ2_CENSUS_DIR") {
        let mut entries: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "bz2"))
            .collect();
        entries.sort();
        assert!(!entries.is_empty(), "no .bz2 files in {dir}");
        for path in entries {
            let bz2 = std::fs::read(&path).unwrap();
            let mut sc = Scanner::new(SliceSource(&bz2));
            let (mut blks, mut opened, mut seg) = (0usize, 0usize, 0usize);
            while let Event::Block { symbols: s, .. } = sc.next(Vec::new()).unwrap() {
                blks += 1;
                if crate::runs::takes_runs(&s) {
                    let n = crate::runs::front(&s, &mut r, k).unwrap();
                    let recs = r.records();
                    let p = s.orig_ptr;
                    let v0517_opens = OpenCells::cap(recs.len(), n, p)
                        .is_some_and(|cap| cap.m == u32::MAX || within_cap(recs, p, LIMIT, cap));
                    let before_opened = cells.opened;
                    out.clear();
                    assert!(cells.invert_runs(&r, n, p, &mut out, k));
                    let head_opens = cells.opened > before_opened;
                    assert_eq!(
                        head_opens,
                        v0517_opens,
                        "{}: route mismatch on block {blks} (n={n}, recs={})",
                        path.display(),
                        recs.len()
                    );
                    if head_opens {
                        opened += 1;
                    } else {
                        seg += 1;
                    }
                } else {
                    seg += 1;
                }
            }
            assert_eq!(
                crate::decode_with::<OpenCells>(&bz2, k).unwrap(),
                crate::decode_with::<Segmented>(&bz2, k).unwrap(),
                "{}: byte mismatch vs Segmented",
                path.display()
            );
            eprintln!(
                "CENSUS_7CORPUS {}: blocks={blks} open={opened} seg={seg} (100% route-identical to v0.5.17)",
                path.file_name().unwrap().to_string_lossy()
            );
        }
    }
}
