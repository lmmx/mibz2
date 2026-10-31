//! Which MTF kernel each block ran (`mtf::selections`, process-wide, so this file is its
//! own test process with one test): the default and `MtfKernel::Shipped` reach exactly
//! `mtf::mtf` (plain) and `mtf::mtf_checkpoints` (cooperative), and nothing of P74-C;
//! `MtfKernel::P74c` reaches only P74-C's entries, each block once.
mod common;
use common::*;
use mibz2::bwt::{Cooperative, Segmented};
use mibz2::mtf::selections::{snapshot, NAMES};
use mibz2::{Kernels, MtfKernel};

fn delta(f: impl FnOnce()) -> [u64; 10] {
    let a = snapshot();
    f();
    let b = snapshot();
    std::array::from_fn(|i| b[i] - a[i])
}

fn named(d: [u64; 10]) -> Vec<(&'static str, u64)> {
    NAMES
        .iter()
        .copied()
        .zip(d)
        .filter(|&(_, n)| n != 0)
        .collect()
}

#[test]
fn each_setting_reaches_its_kernel_only() {
    let text = &corpora(1)[0].1;
    let z = compress(&text[..3_000_000.min(text.len())], 9);
    let blocks = block_starts(&z).len() as u64;
    assert!(blocks >= 2, "{blocks} blocks");
    let want = oracle(&z).unwrap();
    let k = Kernels::detect();
    // the defaults: decode, decode_parallel, StreamDecoder::new, BzReader
    let d = delta(|| {
        assert!(mibz2::decode(&z).unwrap() == want);
        assert!(mibz2::decode_parallel(&z, 2).unwrap() == want);
        assert!(parallel(&z, 3, 6, 1 << 20, 1 << 16, Kernels::detect()).unwrap() == want);
    });
    assert_eq!(
        named(d),
        vec![("shipped_plain", 3 * blocks)],
        "the defaults"
    );
    let d = delta(|| {
        assert!(
            mibz2::decode_with::<Segmented>(&z, k.with_mtf(MtfKernel::Shipped)).unwrap() == want
        )
    });
    assert_eq!(named(d), vec![("shipped_plain", blocks)]);
    let d = delta(|| assert!(mibz2::decode_with::<Cooperative<2>>(&z, k).unwrap() == want));
    assert_eq!(named(d), vec![("shipped_checkpoints", blocks)]);
    // P74-C: each block selected once, then bypassed or combined (without AVX2, shipped)
    let p = k.with_mtf(MtfKernel::P74c);
    let d = delta(|| assert!(mibz2::decode_with::<Segmented>(&z, p).unwrap() == want));
    assert_eq!(d[0] + d[1] + d[3], 0, "{:?}", named(d));
    assert_eq!(d[2], blocks);
    assert_eq!(d[4] + d[5] + d[6], blocks, "{:?}", named(d));
    assert_eq!(d[7] + d[8] + d[9], 0);
    if k.avx2 {
        assert!(
            d[5] > 0,
            "a level-9 text block runs the combined kernel: {:?}",
            named(d)
        );
    }
    let d = delta(|| assert!(mibz2::decode_with::<Cooperative<2>>(&z, p).unwrap() == want));
    assert_eq!(d[0] + d[1] + d[2] + d[4] + d[5] + d[6], 0, "{:?}", named(d));
    assert_eq!((d[3], d[7] + d[8] + d[9]), (blocks, blocks));
    let d = delta(|| {
        assert!(
            mibz2::decode_with::<Segmented>(&z, Kernels::scalar().with_mtf(MtfKernel::P74c))
                .unwrap()
                == want
        )
    });
    assert_eq!(
        (d[2], d[4] + d[6], d[5]),
        (blocks, blocks, 0),
        "{:?}",
        named(d)
    );
    assert_eq!(d[0] + d[1] + d[3] + d[7] + d[8] + d[9], 0);
}
