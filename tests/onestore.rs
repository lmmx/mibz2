//! The one-store room walks (`bwt::walk_segments_room_onestore_with_prefetch` and its
//! byte-tile twin, the walk sweep's `room1w` and `room1b`) on every block of the corpora
//! the decoder's tests use (tests/corpus.rs: text, random, one byte, a pattern, runs and
//! records, by libbzip2 at levels 1, 6 and 9): each block's output exactly the plain
//! walk's and the room walk's, at the sweep's settings and at others.

mod common;
use common::*;
use mibz2::bwt::{
    build, walk, walk_segments_room_onestore_bytes_with_prefetch,
    walk_segments_room_onestore_with_prefetch, walk_segments_room_with_prefetch, SegScratch,
    TVector, PF_L1, PF_L2, PF_OFF,
};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

type Walk = fn(&mut TVector, u32, &mut Vec<u8>, &mut SegScratch, usize);

#[test]
fn onestore_walks_on_every_corpus() {
    let walks: [(&str, Walk, usize); 7] = [
        (
            "room1w",
            walk_segments_room_onestore_with_prefetch::<32, PF_L1>,
            512,
        ),
        (
            "room1b",
            walk_segments_room_onestore_bytes_with_prefetch::<32, PF_L1>,
            512,
        ),
        (
            "room1w, 12 lanes, L2, 256 segments",
            walk_segments_room_onestore_with_prefetch::<12, PF_L2>,
            256,
        ),
        (
            "room1b, 12 lanes, L2, 256 segments",
            walk_segments_room_onestore_bytes_with_prefetch::<12, PF_L2>,
            256,
        ),
        (
            "room1w, 64 lanes, no prefetch, 2048 segments",
            walk_segments_room_onestore_with_prefetch::<64, PF_OFF>,
            2048,
        ),
        (
            "room1b, 64 lanes, no prefetch, 2048 segments",
            walk_segments_room_onestore_bytes_with_prefetch::<64, PF_OFF>,
            2048,
        ),
        ("room", walk_segments_room_with_prefetch::<32, PF_L1>, 512),
    ];
    let k = Kernels::detect();
    let (mut ll, mut freq) = (Vec::new(), [0u32; 256]);
    let (mut tt, mut seg) = (TVector::new(), SegScratch::default());
    let (mut plain, mut out) = (Vec::new(), Vec::new());
    let mut blocks = 0;
    for (name, data) in corpora(1) {
        for level in [1, 6, 9] {
            let z = compress(&data, level);
            let mut sc = Scanner::new(SliceSource(&z));
            let mut total = 0;
            let mut bi = 0;
            while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
                mtf::mtf(&symbols, &mut ll, &mut freq, k).unwrap();
                build(&ll, &freq, &mut tt);
                walk(&tt, symbols.orig_ptr, &mut plain);
                for (what, f, segments) in walks {
                    f(&mut tt, symbols.orig_ptr, &mut out, &mut seg, segments);
                    assert!(
                        out == plain,
                        "{name} level {level} block {bi}: the {what} walk"
                    );
                }
                total += plain.len();
                bi += 1;
            }
            assert!(
                bi > 0 && total > 0,
                "{name} level {level}: no blocks walked"
            );
            blocks += bi;
        }
    }
    // six corpora at three levels, each of one block or more
    assert!(blocks >= 18);
}
