//! Each stage of a decode timed on its own, on one thread: the scanner (headers and
//! Huffman), MTF (with RLE2), the T-vector build, the walk, RLE1 and the CRC, summed over a
//! file's blocks, as cycles a decoded byte at a nominal clock. A diagnostic, for finding
//! where the cycles go: the CI benchmark (the whole decoder, A/B) decides what is faster.
//!
//! ```text
//! phases FILE.bz2 [REPEATS] [GHZ]   (REPEATS: the scanner's pass and each block's stages
//!                                    run again, best kept)
//! ```
//!
//! The stages are the library's own functions, in the order `decode_block` runs them; the
//! output is checked against `decode`'s, so a stage timed is a stage that is right. The
//! walks are timed side by side: the plain one, the bitmap-segmented one, and the flagged
//! one that `Segmented` uses.

use std::time::Instant;

use mibz2::bwt::{build, walk, walk_fast, walk_segments_flagged, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, Kernels};

#[derive(Default, Clone, Copy)]
struct Times {
    scan: f64,
    mtf: f64,
    build: f64,
    walk_plain: f64,
    walk_bitmap: f64,
    walk_flagged: f64,
    rle: f64,
    crc: f64,
}

fn best(repeats: usize, mut f: impl FnMut()) -> f64 {
    (0..repeats)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64()
        })
        .fold(f64::INFINITY, f64::min)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("usage: phases FILE.bz2 [REPEATS] [GHZ]");
    let repeats: usize = args.get(2).map_or(3, |s| s.parse().unwrap());
    let ghz: f64 = args.get(3).map_or(2.8, |s| s.parse().unwrap());
    let data = std::fs::read(path).unwrap();
    let k = Kernels::detect();

    // The scanner, timed over the whole file with one symbols buffer reused, as `decode`
    // reuses it; then its blocks' symbols kept for the rest
    let mut tm = Times {
        scan: best(repeats, || {
            let mut sc = Scanner::new(SliceSource(&data));
            let mut syms = Vec::new();
            while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
                syms = symbols.syms;
            }
        }),
        ..Times::default()
    };
    let mut blocks = Vec::new();
    let mut sc = Scanner::new(SliceSource(&data));
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        blocks.push(symbols);
    }

    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let (mut tt, mut seg) = (TVector::new(), SegScratch::default());
    let (mut plain, mut bitmap, mut flagged, mut out) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut decoded = Vec::new();
    for b in &blocks {
        tm.mtf += best(repeats, || {
            mtf::mtf(b, &mut ll, &mut counts, k).unwrap();
        });
        tm.build += best(repeats, || build(&ll, &counts, &mut tt));
        tm.walk_plain += best(repeats, || walk(&tt, b.orig_ptr, &mut plain));
        tm.walk_bitmap += best(repeats, || {
            walk_fast(&tt, b.orig_ptr, &mut bitmap, &mut seg)
        });
        tm.walk_flagged += best(repeats, || {
            walk_segments_flagged(&mut tt, b.orig_ptr, &mut flagged, &mut seg)
        });
        assert!(plain == bitmap && plain == flagged, "the walks differ");
        tm.rle += best(repeats, || {
            out.clear();
            rle::unrle(&plain, &mut out, k).unwrap();
        });
        let mut c = 0;
        tm.crc += best(repeats, || c = crc::crc32(&out, k));
        assert_eq!(c, b.crc, "a block's CRC");
        decoded.extend_from_slice(&out);
    }
    assert!(
        decoded == mibz2::decode(&data).unwrap(),
        "the stages' output differs from decode's"
    );

    let n = decoded.len() as f64;
    let cb = |s: f64| s * ghz * 1e9 / n;
    let rows = [
        ("scanner (headers, Huffman)", tm.scan),
        ("MTF + RLE2", tm.mtf),
        ("T-vector build", tm.build),
        ("walk, flagged (used)", tm.walk_flagged),
        ("RLE1", tm.rle),
        ("CRC", tm.crc),
    ];
    let total: f64 = rows.iter().map(|r| r.1).sum();
    println!(
        "{path}: {} blocks, {:.1} MB -> {:.1} MB ({:.2}x); c/B of output at {ghz} GHz, best of {repeats}",
        blocks.len(),
        data.len() as f64 / 1e6,
        n / 1e6,
        n / data.len() as f64,
    );
    for (name, s) in rows {
        println!("  {name:28} {:6.2} c/B  {:5.1}%", cb(s), 100.0 * s / total);
    }
    println!(
        "  {:28} {:6.2} c/B  (not in the total)",
        "walk, bitmap (before #4)",
        cb(tm.walk_bitmap)
    );
    println!(
        "  {:28} {:6.2} c/B  (not in the total)",
        "walk, plain",
        cb(tm.walk_plain)
    );
    println!(
        "  {:28} {:6.2} c/B  = {:.0} MB/s on one thread",
        "total",
        cb(total),
        n / total / 1e6
    );
}
