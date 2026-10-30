//! Correctness-only same-binary driver for the frozen research arms. No timing.
//! `cargo run --release --example p74_c -- FILE.bz2`
//! A later separately authorized harness can use the identical public arm signatures.
use mibz2::bwt::{build, walk, TVector};
use mibz2::mtf::{self, p74, p74_c, p74_c::Arm};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, rle, Kernels};

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: p74_c FILE.bz2 (correctness only)");
    let z = std::fs::read(path).unwrap();
    let k = Kernels::detect();
    let expected = mibz2::decode(&z).unwrap();
    let mut decoded = Vec::new();
    let mut scanner = Scanner::new(SliceSource(&z));
    let mut blocks = 0;
    while let Event::Block { symbols: b, .. } = scanner.next(Vec::new()).unwrap() {
        let (mut want, mut wc, mut wcp) = (Vec::new(), [0; 256], Vec::new());
        let n = mtf::mtf_checkpoints(&b, &mut want, &mut wc, &mut wcp, k).unwrap();
        let (mut ll, mut counts, mut cps) = (Vec::new(), [0; 256], Vec::new());
        for arm in [Arm::T0, Arm::T1, Arm::T2] {
            assert_eq!(p74_c::mtf(&b, &mut ll, &mut counts, k, arm), Ok(n));
            assert_eq!((&ll, &counts), (&want, &wc));
            assert_eq!(
                p74_c::mtf_checkpoints(&b, &mut ll, &mut counts, &mut cps, k, arm),
                Ok(n)
            );
            assert_eq!((&ll, &counts, &cps), (&want, &wc, &wcp));
        }
        // Retain Gate 1 Free in this optimized binary for the assembly comparison.
        assert_eq!(p74::mtf(&b, &mut ll, &mut counts, p74::Arm::Free), Ok(n));
        assert_eq!((&ll, &counts), (&want, &wc));
        assert_eq!(
            p74::mtf_checkpoints(&b, &mut ll, &mut counts, &mut cps, p74::Arm::Free),
            Ok(n)
        );
        assert_eq!((&ll, &counts, &cps), (&want, &wc, &wcp));
        let (mut tt, mut walked, mut out) = (TVector::new(), Vec::new(), Vec::new());
        build(&ll, &counts, &mut tt);
        walk(&tt, b.orig_ptr, &mut walked);
        rle::unrle(&walked, &mut out, k).unwrap();
        assert_eq!(crc::crc32(&out, k), b.crc);
        decoded.extend(out);
        blocks += 1;
    }
    assert_eq!(decoded, expected);
    println!(
        "T0/T1/T2 and Gate 1 Free agree: {blocks} blocks, {} decoded bytes; AVX2={}",
        decoded.len(),
        k.avx2
    );
}
