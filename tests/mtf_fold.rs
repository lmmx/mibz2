//! Research only (`mtf-fold`): the parallel decoder with the MTF-into-construction fold on and
//! off, on the test corpora at several levels, whole, cut short and with a byte changed: one
//! output and one error (kind, block and bit) either way, the output libbzip2's, and the fold
//! taken on the blocks it is for. One test, as the fold's switch and counters are the
//! process's.
mod common;
use common::{compress, drive, oracle};
use mibz2::bwt::{ForcedOpenCells, OpenCells};
use mibz2::runs::fold;
use mibz2::{Kernels, StreamDecoder};

fn decode(data: &[u8], k: Kernels, forced: bool) -> (Vec<u8>, Result<(), mibz2::Error>) {
    let dec = if forced {
        StreamDecoder::with_inverse::<ForcedOpenCells>(4, 16, k)
    } else {
        StreamDecoder::with_inverse::<OpenCells>(4, 16, k)
    };
    drive(&dec, data, 1 << 20, 1 << 16)
}

#[test]
fn fold_on_and_off_one_output_and_one_error() {
    let mut kernels = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        kernels.push(Kernels::detect());
    }
    let mut inputs = Vec::new();
    for (name, data) in common::corpora(1) {
        for level in [1, 9] {
            let z = compress(&data, level);
            let mut spoiled = z.clone();
            let at = spoiled.len() / 3;
            spoiled[at] ^= 0x10;
            let cut = z[..z.len() * 2 / 3].to_vec();
            inputs.push((format!("{name} -{level}"), z));
            inputs.push((format!("{name} -{level}, a byte changed"), spoiled));
            inputs.push((format!("{name} -{level}, cut"), cut));
        }
    }
    fold::take_counts();
    let mut on_counts = fold::Counts::default();
    for (what, z) in &inputs {
        let want = oracle(z);
        for &k in &kernels {
            for forced in [false, true] {
                fold::set_enabled(true);
                let on = decode(z, k, forced);
                let c = fold::take_counts();
                on_counts.folded += c.folded;
                on_counts.bailed += c.bailed;
                on_counts.not_open += c.not_open;
                fold::set_enabled(false);
                let off = decode(z, k, forced);
                let c = fold::take_counts();
                assert!(
                    c == fold::Counts::default(),
                    "{what}: folded while off: {c:?}"
                );
                assert!(
                    on.1 == off.1,
                    "{what}, {k:?}: {:?} against {:?}",
                    on.1,
                    off.1
                );
                assert!(on.0 == off.0, "{what}, {k:?}: the bytes");
                if let (Ok(w), Ok(())) = (&want, &on.1) {
                    assert!(&on.0 == w, "{what}, {k:?}: not libbzip2's bytes");
                }
                if want.is_ok() {
                    assert!(on.1.is_ok(), "{what}, {k:?}: {:?}", on.1);
                }
            }
        }
    }
    fold::set_enabled(true);
    eprintln!("{on_counts:?}");
    assert!(on_counts.folded > 50, "{on_counts:?}");
}
