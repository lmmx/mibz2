//! Public T0/T1/T2 entries, with scalar, shipped AVX2 and libbzip2 oracles.
mod common;
use common::*;
use mibz2::block::Symbols;
use mibz2::mtf::{self, p74_c, p74_c::Arm, CHECKPOINT};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;

const ARMS: [Arm; 3] = [Arm::T0, Arm::T1, Arm::T2];

fn agree(b: &Symbols) -> Vec<u8> {
    let (mut expected, mut ec) = (Vec::new(), [0; 256]);
    let er = mtf::mtf_scalar(b, &mut expected, &mut ec);
    for k in [Kernels::scalar(), Kernels::detect()] {
        let (mut shipped, mut sc, mut scp) = (Vec::new(), [0; 256], Vec::new());
        let sr = mtf::mtf_checkpoints(b, &mut shipped, &mut sc, &mut scp, k);
        assert_eq!(sr, er);
        assert_eq!(shipped, expected);
        assert_eq!(sc, ec);
        for arm in ARMS {
            let (mut ll, mut counts, mut cps) = (vec![99; 11], [99; 256], vec![[99; 256]; 3]);
            assert_eq!(p74_c::mtf(b, &mut ll, &mut counts, k, arm), er, "{arm:?}");
            assert_eq!(ll, expected, "{arm:?} plain bytes");
            assert_eq!(counts, ec, "{arm:?} plain counts");
            assert_eq!(
                p74_c::mtf_checkpoints(b, &mut ll, &mut counts, &mut cps, k, arm),
                sr
            );
            assert_eq!(ll, shipped, "{arm:?} checkpoint bytes");
            assert_eq!(counts, sc, "{arm:?} checkpoint counts");
            assert_eq!(cps, scp, "{arm:?} checkpoint records, including errors");
            if let Ok(n) = sr {
                assert_eq!(ll.len(), n);
                assert_eq!(cps.len(), n / CHECKPOINT + 1);
                let mut cum = [0; 256];
                for (i, chunk) in ll.chunks_exact(CHECKPOINT).enumerate() {
                    for &c in chunk {
                        cum[c as usize] += 1;
                    }
                    assert_eq!(cps[i + 1], cum);
                }
            }
        }
    }
    expected
}

fn check_stream(z: &[u8]) {
    let expected = oracle(z).unwrap();
    assert_eq!(mibz2::decode(z).unwrap(), expected);
    let mut decoded = Vec::new();
    let mut sc = Scanner::new(SliceSource(z));
    while let Event::Block { mut symbols, .. } = sc.next(Vec::new()).unwrap() {
        let ll = agree(&symbols);
        decoded.extend(reference(&ll, symbols.orig_ptr).unwrap());
        let original = symbols.max_len;
        for max in [
            0,
            ll.len().saturating_sub(1),
            ll.len(),
            CHECKPOINT,
            original,
        ] {
            symbols.max_len = max;
            agree(&symbols);
        }
    }
    assert_eq!(decoded, expected);
}

fn block(syms: Vec<u16>, max_len: usize) -> Symbols {
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq: std::array::from_fn(|i| i as u8),
        syms,
        eob: 257,
        max_len,
    }
}

#[test]
fn initial_dispatch_edges_and_short_blocks() {
    for s in [0, 1, 16_383, 16_384, 16_385, 50_000] {
        for max in [0, s, 2 * s - usize::from(s > 0), 2 * s, 2 * s + 1, 900_000] {
            for sym in [0, 2, 16, 17, 32, 33, 256] {
                let mut syms = vec![sym; s];
                if let Some(last) = syms.last_mut() {
                    *last = 257;
                }
                agree(&block(syms, max));
            }
        }
    }
}

#[test]
fn all_alphabets_with_live_policy() {
    for alpha in 1..=256 {
        let mut syms: Vec<u16> = (0..16_384)
            .map(|i| {
                if alpha == 1 || i % 29 == 0 {
                    (i % 2) as u16
                } else {
                    (1 + i % (alpha - 1)) as u16 + 1
                }
            })
            .collect();
        syms.push(alpha as u16 + 1);
        let mut b = block(syms, 900_000);
        b.eob = alpha as u16 + 1;
        // A=1 with 16k run digits intentionally tests RunTooLong.
        agree(&b);
    }
}

#[test]
fn public_errors_before_and_after_live_switch() {
    for prefix in [0, 126, 127, CHECKPOINT + 1] {
        for suffix in [vec![1; 22], vec![0; 21], vec![33; 100_001], vec![0, 1]] {
            let mut syms = vec![33; prefix];
            syms.extend(suffix);
            syms.push(257);
            // Padding past EOB exercises initial dispatch without changing the stage's
            // early termination behavior. Out-of-alphabet symbols are never supplied.
            syms.resize(syms.len().max(16_384), 2);
            for max in [100_000, 900_000] {
                agree(&block(syms.clone(), max));
            }
        }
    }
}

#[test]
fn real_bzip2_corpora_and_independent_decode() {
    for (_, data) in corpora(1) {
        check_stream(&compress(&data, 9));
    }
    let mut rng = Rng(0x74c0_9876_1234_abcd);
    for (_, data) in samples(&mut rng) {
        check_stream(&compress(&data, 1));
    }
}

#[test]
fn adversarial_symbols_as_real_bzip2() {
    // These are correctness fixtures, not measured controls or selection validation.
    for shape in 0..6 {
        let mut syms: Vec<u16> = (0..20_000)
            .map(|i| match shape {
                0 => 256,
                1 => {
                    if i < 10_000 {
                        2
                    } else {
                        256
                    }
                }
                2 => {
                    if i < 10_000 {
                        256
                    } else {
                        2
                    }
                }
                3 => {
                    if i % 32 == 31 {
                        33
                    } else {
                        2
                    }
                }
                4 => [16, 17, 32, 33, 256][i % 5],
                _ => {
                    if i % 65 == 64 {
                        33
                    } else {
                        2
                    }
                }
            })
            .collect();
        if shape == 5 {
            for i in (0..syms.len()).step_by(65).rev() {
                syms.splice(i..i, [0; 6]); // 63-byte runs before fallback groups
            }
        }
        syms.push(257);
        let b = block(syms.clone(), 900_000);
        let ll = agree(&b);
        let mut spec = Spec::of(ll, 0);
        spec.in_use = Some((0..=255).collect());
        spec.syms = Some(syms);
        // Pick a primary row for which the independent RLE1 interpreter succeeds.
        spec.orig_ptr = (0..spec.ll.len() as u32)
            .find(|&p| reference(&spec.ll, p).is_some())
            .unwrap();
        check_stream(&craft(9, &[spec], None));
    }
}
