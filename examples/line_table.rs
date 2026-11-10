//! Design E, the line table (src/line_table.rs; docs/journal/2026-11-10-design-e-registration.md):
//! its check against the decoder's walk, and its census. Untimed: nothing here is a timing.
//!
//! ```text
//! line_table check FILE.bz2 ...    every block: E's walks (plain, and segmented in the
//!                                  bitmap walk's and the flagged walk's shapes, on every
//!                                  kernel the CPU runs) against bwt::walk byte for byte,
//!                                  then RLE1 and the block's CRC
//! line_table census FILE.bz2 ...   per file: the steps E serves from one line access and
//!                                  from two (the line and the overflow list), against the
//!                                  T-vector's one, and each table's bytes a row
//! line_table count FILE.bz2 BLOCKS for callgrind: the first BLOCKS blocks' builds and
//!                                  walks (E's, and the T-vector's with the room walk),
//!                                  each in a function of its own, whose inclusive
//!                                  instruction counts are the steps' costs
//! ```

use mibz2::bwt::{build, walk, walk_fast, walk_segments_room, SegScratch, TVector};
use mibz2::line_table::{Census, Kernel, LineTable, Scratch, BITMAP_SHAPE, FLAGGED_SHAPE};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, Kernels};

/// Each block of the file: its last column, its counts, `orig_ptr` and CRC
fn blocks(data: &[u8], mut f: impl FnMut(&[u8], &[u32; 256], u32, u32)) {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        f(&ll, &counts, b.orig_ptr, b.crc);
        syms = b.syms;
    }
}

fn check(path: &str, data: &[u8]) {
    let k = Kernels::detect();
    let (mut tt, mut e, mut s) = (TVector::new(), LineTable::new(), Scratch::default());
    let (mut want, mut got, mut out) = (Vec::new(), Vec::new(), Vec::new());
    let (mut nblocks, mut rows, mut walks) = (0, 0u64, 0u64);
    blocks(data, |ll, counts, orig, crc_want| {
        build(ll, counts, &mut tt);
        walk(&tt, orig, &mut want);
        assert!(
            e.build(ll, counts),
            "{path}: E's build refused a block's column"
        );
        let mut same = |got: &Vec<u8>, what: &str| {
            assert!(
                got == &want,
                "{path}: block {nblocks}: {what} differs from bwt::walk"
            );
            walks += 1;
        };
        e.walk(orig, &mut got);
        same(&got, "the plain walk");
        let (l, m) = BITMAP_SHAPE;
        assert_eq!(l, 12);
        e.walk_segments::<12, false>(orig, &mut got, &mut s, m);
        same(&got, "the segmented walk, bitmap's shape");
        e.segments::<12, false>(orig, &mut got, &mut s, m);
        same(&got, "the segmented walk, bitmap's shape, forced");
        let (l, m) = FLAGGED_SHAPE;
        assert_eq!(l, 32);
        e.walk_segments::<32, true>(orig, &mut got, &mut s, m);
        same(&got, "the segmented walk, flagged's shape");
        e.segments::<32, true>(orig, &mut got, &mut s, m);
        same(&got, "the segmented walk, flagged's shape, forced");
        // every kernel this CPU runs (AVX2, POPCNT, the baseline), its own table each
        for kernel in Kernel::all() {
            assert!(e.build_with(kernel, ll, counts));
            e.segments_with::<32, true>(kernel, orig, &mut got, &mut s, m);
            same(&got, "the segmented walk, flagged's shape, each kernel");
            e.walk_with(kernel, orig, &mut got);
            same(&got, "the plain walk, each kernel");
        }
        out.clear();
        rle::unrle(&got, &mut out, k).unwrap();
        assert_eq!(
            crc::crc32(&out, k),
            crc_want,
            "{path}: block {nblocks}: CRC"
        );
        nblocks += 1;
        rows += ll.len() as u64;
    });
    println!(
        "{path}: {nblocks} blocks, {rows} rows: E's walks equal bwt::walk's on every block \
         ({walks} walks), and RLE1's output matches every block's CRC"
    );
}

fn census(path: &str, data: &[u8]) {
    let mut e = LineTable::new();
    let mut k = Census::default();
    let mut nblocks = 0;
    blocks(data, |ll, counts, _, _| {
        assert!(e.build(ll, counts));
        k.add(&e.census());
        nblocks += 1;
    });
    let n = k.rows as f64;
    let pct = |x: u64, of: u64| 100.0 * x as f64 / of as f64;
    let name = std::path::Path::new(path)
        .file_name()
        .map_or(path.into(), |f| f.to_string_lossy());
    println!(
        "| {name} | {nblocks} | {} | {:.2} | {:.1}% | {:.2}% | {:.2}% | {:.3} | 4 (5 with the column) |",
        k.rows,
        k.distinct as f64 / k.lines as f64,
        pct(k.lines_inline, k.lines),
        pct(k.one_access, k.rows),
        pct(k.two_access, k.rows),
        k.bytes as f64 / n,
    );
}

#[inline(never)]
fn count_t_build(ll: &[u8], counts: &[u32; 256], tt: &mut TVector) {
    build(ll, counts, tt)
}
#[inline(never)]
fn count_t_walk(tt: &mut TVector, orig: u32, out: &mut Vec<u8>, s: &mut SegScratch) {
    walk_segments_room(tt, orig, out, s)
}
#[inline(never)]
fn count_t_walk12(tt: &TVector, orig: u32, out: &mut Vec<u8>, s: &mut SegScratch) {
    walk_fast(tt, orig, out, s)
}
#[inline(never)]
fn count_e_walk12(e: &LineTable, orig: u32, out: &mut Vec<u8>, s: &mut Scratch) {
    e.walk_segments::<12, false>(orig, out, s, BITMAP_SHAPE.1)
}
#[inline(never)]
fn count_e_build(ll: &[u8], counts: &[u32; 256], e: &mut LineTable) {
    assert!(e.build(ll, counts));
}
#[inline(never)]
fn count_e_walk(e: &LineTable, orig: u32, out: &mut Vec<u8>, s: &mut Scratch) {
    e.walk_segments::<32, true>(orig, out, s, FLAGGED_SHAPE.1)
}

/// The instructions' split, under callgrind (`count`): the four functions' inclusive counts
fn count(path: &str, data: &[u8], max: usize) {
    let (mut tt, mut ts, mut e, mut es) = (
        TVector::new(),
        SegScratch::default(),
        LineTable::new(),
        Scratch::default(),
    );
    let (mut a, mut b) = (Vec::new(), Vec::new());
    let (mut nblocks, mut rows) = (0, 0);
    blocks(data, |ll, counts, orig, _| {
        if nblocks == max {
            return;
        }
        count_t_build(ll, counts, &mut tt);
        count_t_walk(&mut tt, orig, &mut a, &mut ts);
        count_t_walk12(&tt, orig, &mut b, &mut ts);
        assert!(a == b, "{path}: the bitmap walk differs");
        count_e_build(ll, counts, &mut e);
        count_e_walk(&e, orig, &mut b, &mut es);
        assert!(a == b, "{path}: E's walk differs");
        count_e_walk12(&e, orig, &mut b, &mut es);
        assert!(a == b, "{path}: E's 12-lane walk differs");
        nblocks += 1;
        rows += ll.len();
    });
    println!("{path}: {nblocks} blocks, {rows} rows counted");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: line_table check|census FILE.bz2 ...";
    let cmd = args.get(1).expect(usage).as_str();
    assert!(args.len() > 2, "{usage}");
    if cmd == "census" {
        println!(
            "| file | blocks | rows (steps) | distinct bytes a line (mean) | lines with at most 9 \
             | steps from one access, E | from two, E | E bytes a row | T-vector bytes a row |"
        );
        println!("|---|--:|--:|--:|--:|--:|--:|--:|--:|");
    }
    for path in &args[2..] {
        let data = std::fs::read(path).unwrap();
        match cmd {
            "check" => check(path, &data),
            "census" => census(path, &data),
            "count" => {
                count(path, &data, args.get(3).map_or(3, |s| s.parse().unwrap()));
                break;
            }
            _ => panic!("{usage}"),
        }
    }
    if cmd == "census" {
        println!(
            "\nThe T-vector serves every step from one access (one 4-byte entry); E's line \
             table serves a step from one access unless its row's byte is past the eighth \
             distinct byte of a line with more than nine. Each row is stepped from once a walk."
        );
    }
}
