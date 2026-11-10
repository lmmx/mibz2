//! Each stage of a decode timed on its own, on one thread: the scanner (headers and
//! Huffman), MTF (with RLE2), the T-vector build, the walk, RLE1 and the CRC, summed over a
//! file's blocks, as cycles a decoded byte at a nominal clock. A diagnostic, for finding
//! where the cycles go: the CI benchmark (the whole decoder, A/B) decides what is faster.
//!
//! ```text
//! phases FILE.bz2 [PASSES] [GHZ]   (PASSES: decodes of the file, each stage's fastest
//!                                   pass kept)
//! ```
//!
//! A pass decodes the file once in decode order, block by block, as `decode` and the
//! parallel decoder's workers do: the scanner's next block, MTF, the build, the walk, RLE1
//! (into an output buffer reused block to block) and the CRC, each stage timed as it runs.
//! So each stage meets the caches as a decode leaves them: the scratch and buffers warm
//! from the block before, never from the same block timed moments before. A stage's figure
//! is its time summed over the blocks in its fastest pass. (Before bench 2.0.0 each stage
//! kept its best of repeats on the same block, and the walks ran back to back on it: a
//! warm state no decode reaches, which flattered the walk; docs/journal/
//! 2026-10-27-walk-sweep.md.)
//!
//! The room walk's last step, joining its segments into the block's bytes in order, is
//! also timed on its own (`walk_segments_room_parts`, `SegScratch::assemble`): a row
//! within the walk's, which still counts both. The
//! walks not used (the plain one, the bitmap-segmented one and the flagged one) are timed
//! in passes of their own, the same way, each in the walk's place: MTF, the build, the
//! walk timed, then RLE1 and the CRC, which check its output. So is the room walk with its
//! assembly timed as one, the check that timing them apart changed neither, its passes
//! interleaved with the split ones (which goes first alternating). The pass that `decode`
//! runs is checked against `decode`'s output.
//!
//! Research rows (design E, the line table: src/line_table.rs, docs/journal/
//! 2026-11-10-design-e-registration.md), never in the total: passes of their own in which
//! E's table is built in the T-vector's place and walked backward in the walk's, segmented
//! in the bitmap walk's shape (12 lanes, 256 segments) and in the flagged walk's (32 lanes,
//! 512 segments, each lane's next line prefetched); RLE1 and the CRC check their output.
//! Each row not in the total names the stage it stands against (`against:`), which
//! bench/stages.py records and bench/report.py sets it beside, paired by run.

use std::time::Instant;

use mibz2::bwt::{
    build, walk, walk_fast, walk_segments_flagged, walk_segments_room, walk_segments_room_parts,
    SegScratch, TVector,
};
use mibz2::line_table::{LineTable, Scratch, BITMAP_SHAPE, FLAGGED_SHAPE};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, Kernels};

#[derive(Default, Clone, Copy)]
struct Times {
    scan: f64,
    mtf: f64,
    build: f64,
    walk: f64,
    assemble: f64,
    rle: f64,
    crc: f64,
}

/// A walk in the decoder's place: the T-vector, its `orig_ptr`, the walk's output, scratch
type Walk = fn(&mut TVector, u32, &mut Vec<u8>, &mut SegScratch);

fn plain(tt: &mut TVector, p: u32, out: &mut Vec<u8>, _: &mut SegScratch) {
    walk(tt, p, out)
}
fn bitmap(tt: &mut TVector, p: u32, out: &mut Vec<u8>, s: &mut SegScratch) {
    walk_fast(tt, p, out, s)
}

/// One decode of the file in decode order with `w` as the walk, each stage timed (with
/// `w` None, the room walk the decoder uses, its segments' assembly timed apart); returns
/// the stages' times and, if `keep`, the decoded bytes
fn pass(data: &[u8], k: Kernels, w: Option<Walk>, keep: bool) -> (Times, usize, usize, Vec<u8>) {
    let mut tm = Times::default();
    let mut sc = Scanner::new(SliceSource(data));
    let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
    let (mut tt, mut seg) = (TVector::new(), SegScratch::default());
    let (mut walked, mut out, mut decoded) = (Vec::new(), Vec::new(), Vec::new());
    let (mut n, mut blocks) = (0, 0);
    let time = |slot: &mut f64, f: &mut dyn FnMut()| {
        let t = Instant::now();
        f();
        *slot += t.elapsed().as_secs_f64();
    };
    loop {
        let mut ev = None;
        time(&mut tm.scan, &mut || {
            ev = Some(sc.next(std::mem::take(&mut syms)).unwrap())
        });
        let Some(Event::Block { symbols: b, .. }) = ev else {
            break;
        };
        time(&mut tm.mtf, &mut || {
            mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        });
        time(&mut tm.build, &mut || build(&ll, &counts, &mut tt));
        match w {
            Some(w) => time(&mut tm.walk, &mut || {
                w(&mut tt, b.orig_ptr, &mut walked, &mut seg)
            }),
            None => {
                let mut segmented = false;
                time(&mut tm.walk, &mut || {
                    segmented = walk_segments_room_parts(&mut tt, b.orig_ptr, &mut walked, &mut seg)
                });
                if segmented {
                    time(&mut tm.assemble, &mut || seg.assemble(&mut walked));
                }
            }
        }
        time(&mut tm.rle, &mut || {
            out.clear();
            rle::unrle(&walked, &mut out, k).unwrap();
        });
        let mut c = 0;
        time(&mut tm.crc, &mut || c = crc::crc32(&out, k));
        assert_eq!(c, b.crc, "a block's CRC");
        n += out.len();
        blocks += 1;
        if keep {
            decoded.extend_from_slice(&out);
        }
        syms = b.syms;
    }
    (tm, n, blocks, decoded)
}

/// One decode of the file in decode order with design E's line table in the T-vector's
/// place and its segmented walk (`L` lanes, `segments` segments, `PF`: prefetch) in the
/// walk's; returns E's build's and walk's times. RLE1 and the CRC check every block.
fn pass_e<const L: usize, const PF: bool>(data: &[u8], k: Kernels, segments: usize) -> (f64, f64) {
    let (mut build_s, mut walk_s) = (0.0, 0.0);
    let mut sc = Scanner::new(SliceSource(data));
    let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
    let (mut e, mut s) = (LineTable::new(), Scratch::default());
    let (mut walked, mut out) = (Vec::new(), Vec::new());
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        let t = Instant::now();
        assert!(e.build(&ll, &counts), "E's build refused a block's column");
        build_s += t.elapsed().as_secs_f64();
        let t = Instant::now();
        e.walk_segments::<L, PF>(b.orig_ptr, &mut walked, &mut s, segments);
        walk_s += t.elapsed().as_secs_f64();
        out.clear();
        rle::unrle(&walked, &mut out, k).unwrap();
        assert_eq!(crc::crc32(&out, k), b.crc, "a block's CRC, from E's walk");
        syms = b.syms;
    }
    (build_s, walk_s)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("usage: phases FILE.bz2 [PASSES] [GHZ]");
    let passes: usize = args.get(2).map_or(3, |s| s.parse().unwrap());
    let ghz: f64 = args.get(3).map_or(2.8, |s| s.parse().unwrap());
    assert!(passes >= 1, "PASSES: at least 1");
    let data = std::fs::read(path).unwrap();
    let k = Kernels::detect();

    // Each stage's fastest pass; the used walk's passes first, then each other walk's
    let fastest = |a: Times, b: Times| Times {
        scan: a.scan.min(b.scan),
        mtf: a.mtf.min(b.mtf),
        build: a.build.min(b.build),
        walk: a.walk.min(b.walk),
        assemble: a.assemble.min(b.assemble),
        rle: a.rle.min(b.rle),
        crc: a.crc.min(b.crc),
    };
    let inf = Times {
        scan: f64::INFINITY,
        mtf: f64::INFINITY,
        build: f64::INFINITY,
        walk: f64::INFINITY,
        assemble: f64::INFINITY,
        rle: f64::INFINITY,
        crc: f64::INFINITY,
    };
    let mut tm = inf;
    let (mut n, mut blocks) = (0, 0);
    // The check that timing the assembly apart leaves the walk as it was: the room walk
    // with its assembly timed as one, in passes interleaved with the split ones, which of
    // the two goes first alternating, so that neither always follows the other
    let mut as_one = f64::INFINITY;
    let check = || pass(&data, k, Some(walk_segments_room), false).0.walk;
    for i in 0..passes {
        if i % 2 == 1 {
            as_one = as_one.min(check());
        }
        let (t, bytes, count, decoded) = pass(&data, k, None, i == 0);
        if i == 0 {
            assert!(
                decoded == mibz2::decode(&data).unwrap(),
                "the stages' output differs from decode's"
            );
        }
        tm = fastest(tm, t);
        (n, blocks) = (bytes, count);
        if i % 2 == 0 {
            as_one = as_one.min(check());
        }
    }
    let others: [(&str, Walk); 3] = [
        ("walk, flagged (before #37)", walk_segments_flagged),
        ("walk, bitmap (before #4)", bitmap),
        ("walk, plain", plain),
    ];
    let others = others.map(|(name, w)| {
        let best = (0..passes)
            .map(|_| pass(&data, k, Some(w), false).0.walk)
            .fold(f64::INFINITY, f64::min);
        (name, best)
    });
    // Design E (research): each stage's fastest pass, as above, E's build from every pass
    let (mut e_build, mut e_walk12, mut e_walk32) = (f64::INFINITY, f64::INFINITY, f64::INFINITY);
    assert_eq!((BITMAP_SHAPE.0, FLAGGED_SHAPE.0), (12, 32));
    for _ in 0..passes {
        let (b, w) = pass_e::<12, false>(&data, k, BITMAP_SHAPE.1);
        (e_build, e_walk12) = (e_build.min(b), e_walk12.min(w));
        let (b, w) = pass_e::<32, true>(&data, k, FLAGGED_SHAPE.1);
        (e_build, e_walk32) = (e_build.min(b), e_walk32.min(w));
    }

    let n = n as f64;
    let cb = |s: f64| s * ghz * 1e9 / n;
    let rows = [
        ("scanner (headers, Huffman)", tm.scan),
        ("MTF + RLE2", tm.mtf),
        ("T-vector build", tm.build),
        ("walk, room (used)", tm.walk + tm.assemble),
        ("RLE1", tm.rle),
        ("CRC", tm.crc),
    ];
    let total: f64 = rows.iter().map(|r| r.1).sum();
    println!(
        "{path}: {blocks} blocks, {:.1} MB -> {:.1} MB ({:.2}x); c/B of output at {ghz} GHz, \
         decode order, each stage's fastest of {passes} passes",
        data.len() as f64 / 1e6,
        n / 1e6,
        n / data.len() as f64,
    );
    for (name, s) in rows {
        println!("  {name:28} {:6.2} c/B  {:5.1}%", cb(s), 100.0 * s / total);
    }
    println!(
        "  {:28} {:6.2} c/B  {:5.1}%  (within the walk)",
        "segment assembly",
        cb(tm.assemble),
        100.0 * tm.assemble / total
    );
    println!(
        "  {:28} {:6.2} c/B  (not in the total: the walk's row, timed as one; against: walk)",
        "walk, room + assembly as one",
        cb(as_one)
    );
    for (name, s) in others {
        println!(
            "  {name:28} {:6.2} c/B  (not in the total; against: walk)",
            cb(s)
        );
    }
    for (name, s, against) in [
        ("line table E: build", e_build, "build"),
        ("line table E: walk, 12 lanes", e_walk12, "walk"),
        ("line table E: walk, 32 lanes, prefetch", e_walk32, "walk"),
        (
            "line table E: build + walk, 32 lanes",
            e_build + e_walk32,
            "build+walk",
        ),
    ] {
        println!(
            "  {name:28} {:6.2} c/B  (not in the total; against: {against})",
            cb(s)
        );
    }
    println!(
        "  {:28} {:6.2} c/B  = {:.0} MB/s on one thread",
        "total",
        cb(total),
        n / total / 1e6
    );
}
