//! P87's process-wide arm through the parallel decoder (feature `p87`): the arm and
//! placement set and read back; each arm's 4-worker decode (`with_inverse::<Segmented>`,
//! as `Plan::Workers(4)` builds it) exactly libbzip2's, including corrupt input ending in
//! the same error after the same bytes; a decode cancelled mid-stream ends cleanly in
//! every arm. One test function: the arm is process-wide (this file is its own process).
#![cfg(feature = "p87")]

mod common;
use common::{compress, oracle, Rng};
use mibz2::bwt::Segmented;
use mibz2::p87::{self, Arm};
use mibz2::{Kernels, StreamDecoder};

/// `z` through a 4-worker decoder of this process's arm: the bytes read, and the error
fn decode(z: &[u8]) -> (Vec<u8>, Option<String>) {
    let dec = StreamDecoder::with_inverse::<Segmented>(4, 8, Kernels::detect());
    let mut out = Vec::new();
    let err = std::thread::scope(|s| {
        s.spawn(|| {
            for c in z.chunks(1 << 20) {
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
                    break Some(format!("{:?}", e.kind));
                }
            }
        }
    });
    dec.join();
    (out, err)
}

#[test]
fn every_arm_through_the_parallel_decoder() {
    let mut rng = Rng(0x8710);
    let samples = common::samples(&mut rng);
    let text = &samples.iter().find(|(n, _)| n == "text").unwrap().1;
    let runs = &samples.iter().find(|(n, _)| n == "runs").unwrap().1;
    let mut inputs = Vec::new();
    for (name, data, level) in [("text", text, 9), ("text", text, 1), ("runs", runs, 9)] {
        inputs.push((format!("{name} -{level}"), compress(data, level)));
    }
    // a corrupt stream: a good one with a byte flipped in its fifth block's data
    let mut bad = compress(text, 1);
    let at = common::block_starts(&bad)[4] as usize / 8 + 200;
    bad[at] ^= 0x5a;
    inputs.push(("text -1, corrupted".into(), bad));

    for arm in [Arm::Shipped, Arm::IntoRle, Arm::IntoLl] {
        for rle_first in [false, true] {
            p87::set(arm, rle_first);
            assert_eq!(p87::arm(), (arm, rle_first), "set, read back");
            for (name, z) in &inputs {
                let (out, err) = decode(z);
                match oracle(z) {
                    Ok(want) => {
                        assert_eq!(err, None, "{name} {arm:?}/{rle_first}");
                        assert!(out == want, "{name} {arm:?}/{rle_first}: output differs");
                    }
                    Err(_) => {
                        assert!(err.is_some(), "{name} {arm:?}: corrupt input decoded");
                        // the shipped path's bytes before the error, in every arm
                        p87::set(Arm::Shipped, false);
                        let (shipped, shipped_err) = decode(z);
                        p87::set(arm, rle_first);
                        assert_eq!(err, shipped_err, "{name} {arm:?}: the error");
                        assert!(out == shipped, "{name} {arm:?}: the bytes before it");
                    }
                }
            }
            // cancelled after its first bytes, a feeder still writing: the feeder's next
            // write fails, the threads end, nothing hangs or panics
            let z = &inputs[0].1;
            let dec = StreamDecoder::with_inverse::<Segmented>(4, 8, Kernels::detect());
            std::thread::scope(|s| {
                s.spawn(|| {
                    for c in z.chunks(64 << 10) {
                        if dec.write(c.to_vec()).is_err() {
                            return;
                        }
                    }
                    dec.close();
                });
                let mut buf = vec![0u8; 1 << 16];
                assert!(dec.read(&mut buf).unwrap() > 0, "{arm:?}: the first bytes");
                dec.cancel();
                assert!(dec.read(&mut buf).is_err(), "{arm:?}: a read after cancel");
            });
            dec.join();
        }
    }
    p87::set(Arm::Shipped, false);
}
