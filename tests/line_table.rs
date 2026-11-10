//! Design E, the line table (src/line_table.rs): its walks give `bwt::walk`'s bytes on every
//! block of the bench's corpora (tests/common's `corpora`, which `ab gen` writes) and on a
//! fuzz smoke: those corpora's streams mutated (bits flipped, bytes set, spans cut), and
//! crafted blocks of random columns, whose `orig_ptr` cycle need not divide the block.
//! Research only: the decoder never uses it.

mod common;
use common::*;
use mibz2::bwt::{build, walk, TVector};
use mibz2::line_table::{LineTable, Scratch};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

/// Every block the scanner and MTF accept, until the first error: E's walks against
/// `bwt::walk`; returns the blocks checked
fn check_blocks(data: &[u8], ctx: &str) -> usize {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
    let (mut tt, mut e, mut s) = (TVector::new(), LineTable::new(), Scratch::default());
    let (mut want, mut got) = (Vec::new(), Vec::new());
    let mut checked = 0;
    loop {
        let Ok(Event::Block { symbols: b, .. }) = sc.next(std::mem::take(&mut syms)) else {
            return checked;
        };
        if mtf::mtf(&b, &mut ll, &mut counts, k).is_err() || b.orig_ptr as usize >= ll.len() {
            return checked;
        }
        build(&ll, &counts, &mut tt);
        walk(&tt, b.orig_ptr, &mut want);
        assert!(
            e.build(&ll, &counts),
            "{ctx}: E's build refused MTF's column"
        );
        e.walk(b.orig_ptr, &mut got);
        assert!(got == want, "{ctx}: block {checked}: the plain walk");
        e.walk_segments::<12, false>(b.orig_ptr, &mut got, &mut s, 256);
        assert!(got == want, "{ctx}: block {checked}: 12 lanes");
        e.segments::<32, true>(b.orig_ptr, &mut got, &mut s, 512);
        assert!(got == want, "{ctx}: block {checked}: 32 lanes, forced");
        e.segments::<5, false>(b.orig_ptr, &mut got, &mut s, 7);
        assert!(got == want, "{ctx}: block {checked}: 5 lanes, 7 segments");
        checked += 1;
        syms = b.syms;
    }
}

#[test]
fn bench_corpora() {
    for (name, data) in corpora(1) {
        for level in [1, 9] {
            let z = compress(&data, level);
            let ctx = format!("{name} level {level}");
            let blocks = check_blocks(&z, &ctx);
            assert!(blocks >= 1, "{ctx}: no block checked");
            assert!(mibz2::decode(&z).unwrap() == data, "{ctx}: decode");
        }
    }
}

#[test]
fn fuzz_smoke() {
    let mut rng = Rng(0x005e_ed0e_117a_b1e5_u64);
    let mut corpus: Vec<Vec<u8>> = Vec::new();
    for (_, data) in samples(&mut rng) {
        let small = &data[..data.len().min(150_000)];
        corpus.push(compress(small, 1));
        corpus.push(compress(small, 9));
    }
    // crafted blocks: random columns over small alphabets and all 256 bytes, any orig_ptr
    for (n, alphabet) in [
        (5usize, 3u64),
        (300, 2),
        (5000, 9),
        (70_000, 3),
        (70_000, 256),
    ] {
        let ll: Vec<u8> = (0..n)
            .map(|_| (rng.below(alphabet) as u8).wrapping_mul(7))
            .collect();
        let p = rng.below(n as u64) as u32;
        corpus.push(craft(9, &[Spec::of(ll, p)], None));
    }
    let mut valid = 0;
    for z in &corpus {
        valid += check_blocks(z, "valid");
    }
    let mut checked = 0;
    for case in 0..1000 {
        let mut z = corpus[rng.below(corpus.len() as u64) as usize].clone();
        // past the stream header, so most cases reach a block
        let at = 4 + rng.below((z.len() - 4) as u64) as usize;
        match case % 3 {
            0 => z[at] ^= 1 << rng.below(8),
            1 => z[at] = rng.next() as u8,
            _ => {
                let len = (rng.below(32) as usize).min(z.len() - at);
                z.drain(at..at + len);
            }
        }
        checked += check_blocks(&z, &format!("mutated case {case}"));
    }
    // (a mutated stream reaches its blocks up to the one the edit breaks)
    assert!(
        valid >= 20 && checked >= 500,
        "the smoke checked {valid} valid and {checked} mutated blocks"
    );
    println!("{valid} valid and {checked} mutated blocks checked");
}
