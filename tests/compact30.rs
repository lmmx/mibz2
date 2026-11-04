//! The default inverse-BWT kernel (`bwt::Compact30`) against production's T-vector kernel
//! (`bwt::Segmented`), block by block: on the corpora and the sample shapes at three
//! levels, and on crafted blocks of every distinct-symbol regime at the walk's and the
//! format's length boundaries. Each block's inversion must be byte for byte the same.

mod common;
use common::*;
use mibz2::bwt::{Compact30, InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;

/// Every block of `data` inverted by both kernels (each kept from block to block, as a
/// worker keeps it), compared; the number of blocks
fn blocks_agree(data: &[u8], what: &str) -> usize {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let (mut c30, mut seg) = (Compact30::default(), Segmented::default());
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let (mut got, mut want) = (Vec::new(), Vec::new());
    let mut syms = Vec::new();
    let mut blocks = 0;
    while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
        mibz2::mtf::mtf(&symbols, &mut ll, &mut counts, k).unwrap();
        seg.invert(&ll, &counts, symbols.orig_ptr, &mut want);
        c30.invert(&ll, &counts, symbols.orig_ptr, &mut got);
        assert!(
            got == want,
            "{what}: block {blocks} ({} rows) differs",
            ll.len()
        );
        blocks += 1;
        syms = symbols.syms;
    }
    blocks
}

#[test]
fn corpora_block_by_block() {
    let mut blocks = 0;
    let mut inputs: Vec<(String, Vec<u8>)> = corpora(1)
        .into_iter()
        .map(|(n, d)| (n.to_string(), d))
        .collect();
    inputs.extend(samples(&mut Rng(0xc30_c30)));
    for (name, data) in &inputs {
        for level in [1, 6, 9] {
            let z = compress(data, level);
            blocks += blocks_agree(&z, &format!("{name} level {level}"));
            assert!(
                mibz2::decode_with::<Compact30>(&z, Kernels::detect()).unwrap() == *data,
                "{name} level {level}: decode"
            );
        }
    }
    assert!(blocks > 100, "{blocks} blocks");
}

/// Crafted blocks (their last columns as given, which no encoder need write): one byte,
/// two, a few and every byte, in runs or not, periodic (lists of several cycles), at
/// lengths around MIN_SEGMENTED (2^16), an epoch (61,440) and the format's largest block;
/// decoded by both kernels and the plain reference
#[test]
fn crafted_blocks_every_regime() {
    let mut rng = Rng(0x2026_1104);
    for n in [
        1usize, 29, 30, 31, 61_439, 61_440, 61_441, 65_535, 65_536, 65_537, 899_999, 900_000,
    ] {
        for sigma in [1u64, 2, 9, 256] {
            for shape in 0..3 {
                let mut ll = Vec::with_capacity(n);
                while ll.len() < n {
                    let b = (rng.below(sigma) * (256 / sigma)) as u8;
                    let run = match shape {
                        0 => 1,
                        1 => [1, 2, 16, 17, 30, 31, 33, 1000][rng.below(8) as usize],
                        _ => 1,
                    };
                    ll.extend(std::iter::repeat_n(b, run.min(n - ll.len())));
                }
                if shape == 2 {
                    // periodic: a string repeated, whose list is several cycles
                    let p = 1 + rng.below(31) as usize;
                    for i in p..n {
                        ll[i] = ll[i % p];
                    }
                }
                let what = format!("n {n} sigma {sigma} shape {shape}");
                for orig_ptr in [0, rng.below(n as u64) as u32, n as u32 - 1] {
                    let z = craft(9, &[Spec::of(ll.clone(), orig_ptr)], None);
                    let want = reference(&ll, orig_ptr);
                    let got = mibz2::decode_with::<Compact30>(&z, Kernels::detect()).ok();
                    let seg = mibz2::decode_with::<Segmented>(&z, Kernels::detect()).ok();
                    assert!(
                        got == seg,
                        "{what} orig_ptr {orig_ptr}: Compact30 vs Segmented"
                    );
                    assert!(got == want, "{what} orig_ptr {orig_ptr}: vs the reference");
                }
            }
        }
    }
}
