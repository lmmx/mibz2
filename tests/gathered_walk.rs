//! The walk sweep's gathered round (`bwt::walk_segments_room_gathered`, examples/sweep.rs)
//! on every block of the test corpora (text, random, one byte, a pattern, runs, records),
//! each compressed by reference bzip2 at levels 1, 6 and 9: block by block, as the sweep
//! checks its settings, the same bytes as the plain walk's, at the room walk's segment
//! counts. Where the CPU has no AVX2 the gathered walk is the room walk's own step, and
//! the test says so.

mod common;
use common::*;
use mibz2::bwt::{build, gather_available, walk, walk_segments_room_gathered, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

#[test]
fn gathered_walk_on_every_corpus_block() {
    if !gather_available() {
        eprintln!("no AVX2 here: the gathered walk runs as the room walk");
    }
    let k = Kernels::detect();
    let (mut ll, mut freq) = (Vec::new(), [0u32; 256]);
    let (mut tt, mut seg) = (TVector::new(), SegScratch::default());
    let (mut plain, mut out) = (Vec::new(), Vec::new());
    let mut blocks = 0;
    for (name, data) in corpora(1) {
        for level in [1, 6, 9] {
            let z = compress(&data, level);
            let mut sc = Scanner::new(SliceSource(&z));
            let mut bi = 0;
            let mut total = 0;
            while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
                mtf::mtf(&symbols, &mut ll, &mut freq, k).unwrap();
                build(&ll, &freq, &mut tt);
                walk(&tt, symbols.orig_ptr, &mut plain);
                for segments in [256, 512, 1024, 2048] {
                    walk_segments_room_gathered(
                        &mut tt,
                        symbols.orig_ptr,
                        &mut out,
                        &mut seg,
                        segments,
                    );
                    assert!(
                        out == plain,
                        "{name} level {level} block {bi}: {segments} segments"
                    );
                }
                total += plain.len();
                bi += 1;
                blocks += 1;
            }
            assert!(bi > 0 || data.is_empty(), "{name} level {level}: no blocks");
            assert!(total > 0, "{name} level {level}: no bytes");
        }
    }
    assert!(blocks > 0);
}
