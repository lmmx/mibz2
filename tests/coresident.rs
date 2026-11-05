//! The co-resident walk (`bwt::Coresident`: the room walk's shape where the parallel
//! decoder's workers outnumber the physical cores, `parallel::Shape`) and the shapes its CI
//! arms take (`bwt::Shaped<L, PF>`), against libbzip2 and the shape alone
//! (`bwt::Segmented`):
//! - every shape on the corpora and the samples, at levels 1, 6 and 9, sequentially;
//! - through the parallel decoder, valid streams byte for byte and a corrupt one ending in
//!   the shape alone's error, after its bytes;
//! - the default decoder on this machine, at 1, 2, the CPUs and twice them: its workers'
//!   shape the one for their number, and every entry point exact.

mod common;
use common::*;
use mibz2::bwt::{Coresident, InverseBwt, Segmented, Shaped, PF_L1, PF_L2};
use mibz2::parallel::{Plan, Shape};
use mibz2::{decode_with, BzReader, Error, Kernels, StreamDecoder};
use std::io::Read;

type Decode = fn(&[u8], Kernels) -> Result<Vec<u8>, Error>;

/// The shapes: the shape alone (as `Segmented` and as `Shaped`), the co-resident arms'
/// (16 lanes into L2, the default; 16 into L1; 24 and 32 into L2) and `Coresident`
fn shapes() -> Vec<(&'static str, Decode)> {
    vec![
        ("alone (Segmented)", decode_with::<Segmented>),
        ("32 lanes, L1 (Shaped)", decode_with::<Shaped<32, PF_L1>>),
        ("16 lanes, L2", decode_with::<Shaped<16, PF_L2>>),
        ("16 lanes, L1", decode_with::<Shaped<16, PF_L1>>),
        ("24 lanes, L2", decode_with::<Shaped<24, PF_L2>>),
        ("32 lanes, L2", decode_with::<Shaped<32, PF_L2>>),
        ("Coresident", decode_with::<Coresident>),
    ]
}

#[test]
fn every_shape_on_the_corpora_and_samples() {
    let k = Kernels::detect();
    let mut rng = Rng(0xc0_1e5);
    let mut inputs: Vec<(String, Vec<u8>, u32)> = Vec::new();
    for (name, data) in corpora(1) {
        for level in [1, 6, 9] {
            inputs.push((name.to_string(), data.clone(), level));
        }
    }
    for (name, data) in samples(&mut rng) {
        for level in [1, 9] {
            inputs.push((name.clone(), data.clone(), level));
        }
    }
    for (name, data, level) in &inputs {
        let z = compress(data, *level);
        assert!(oracle(&z).unwrap() == *data, "{name} -{level}: the oracle");
        for (shape, decode) in shapes() {
            assert!(decode(&z, k).unwrap() == *data, "{name} -{level}: {shape}");
        }
    }
}

/// `z` through a 4-worker decoder of kernel `I`, fed 100,003 bytes at a time: the bytes
/// read, and the error
fn par<I: InverseBwt + 'static>(z: &[u8]) -> (Vec<u8>, Option<Error>) {
    let dec = StreamDecoder::with_inverse::<I>(4, 16, Kernels::detect());
    let mut out = Vec::new();
    let err = std::thread::scope(|s| {
        s.spawn(|| {
            for c in z.chunks(100_003) {
                if dec.write(c.to_vec()).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut buf = vec![0u8; 1 << 20];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break None,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => {
                    dec.cancel();
                    break Some(e);
                }
            }
        }
    });
    dec.join();
    (out, err)
}

#[test]
fn every_shape_through_the_parallel_decoder() {
    let mut rng = Rng(0xc0_1e6);
    let samples = samples(&mut rng);
    let text = &samples.iter().find(|(n, _)| n == "text").unwrap().1;
    let runs = &samples.iter().find(|(n, _)| n == "runs").unwrap().1;
    type Par = fn(&[u8]) -> (Vec<u8>, Option<Error>);
    let shapes: [(&str, Par); 6] = [
        ("alone (Segmented)", par::<Segmented>),
        ("16 lanes, L2", par::<Shaped<16, PF_L2>>),
        ("16 lanes, L1", par::<Shaped<16, PF_L1>>),
        ("24 lanes, L2", par::<Shaped<24, PF_L2>>),
        ("32 lanes, L2", par::<Shaped<32, PF_L2>>),
        ("Coresident", par::<Coresident>),
    ];
    for (name, data, level) in [("text", text, 9), ("text", text, 1), ("runs", runs, 9)] {
        let z = compress(data, level);
        for (shape, decode) in shapes {
            let (out, err) = decode(&z);
            assert_eq!(err, None, "{name} -{level}: {shape}");
            assert!(&out == data, "{name} -{level}: {shape}");
        }
    }
    // a corrupt stream: the shape alone's error, after the same bytes
    let mut bad = compress(text, 1);
    let at = block_starts(&bad)[4] as usize / 8 + 200;
    bad[at] ^= 0x5a;
    let (want, want_err) = par::<Segmented>(&bad);
    assert!(want_err.is_some(), "corrupt input decoded");
    for (shape, decode) in shapes {
        let (out, err) = decode(&bad);
        assert_eq!(err, want_err, "{shape}: the error");
        assert!(out == want, "{shape}: the bytes before it");
    }
}

/// The default decoder (`StreamDecoder::new`, so `BzReader::parallel` and
/// `decode_parallel`) on this machine: production's workers in the shape for their number
/// (`Shape::for_workers`), a team plan's in none, decoding exactly through every entry
/// point. Which shape that is depends on the machine (twice the CPUs is co-resident where
/// the topology is read and the plan is workers); the rule itself, on given cores, is
/// `parallel`'s unit tests'
#[test]
fn the_default_decoders_shape_here() {
    let mut rng = Rng(0xc0_1e7);
    let samples = samples(&mut rng);
    let text = &samples.iter().find(|(n, _)| n == "text").unwrap().1;
    let z = compress(text, 9);
    let cpus = std::thread::available_parallelism().map_or(1, |n| n.get());
    for t in [1, 2, cpus, 2 * cpus] {
        let plan = Plan::for_threads(t);
        let want = match plan {
            Plan::Workers(w) => Some(Shape::for_workers(w)),
            Plan::Teams { .. } => None,
        };
        let dec = StreamDecoder::new(t, 4 * t, Kernels::detect());
        assert_eq!(dec.shape(), want, "{t} threads: {plan:?}");
        eprintln!("{t} threads on {cpus} CPUs: {plan:?}, {want:?}");
        drop(dec);
        let out = parallel(&z, t, 4 * t, 1 << 20, 1 << 16, Kernels::detect()).unwrap();
        assert!(&out == text, "{t} threads: StreamDecoder::new");
        assert!(
            &mibz2::decode_parallel(&z, t).unwrap() == text,
            "{t} threads: decode_parallel"
        );
        let mut out = Vec::new();
        BzReader::parallel(std::io::Cursor::new(z.clone()), t)
            .read_to_end(&mut out)
            .unwrap();
        assert!(&out == text, "{t} threads: BzReader::parallel");
    }
}
