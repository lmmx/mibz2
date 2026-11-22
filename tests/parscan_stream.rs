//! The push-streaming front end (`par_scan`) as the default decoder runs it on the measured
//! i9 (`StreamDecoder::new` from 8 threads asked) and as its own entry points run it, against
//! the sequential decoder: the same bytes before the end or the first error, and the same
//! `Error { kind, block, bit }`. Multi-block and multi-stream input at mixed levels, block,
//! stream and end magics on the input's slab edges, every truncation of a small multi-stream
//! file, every thread count from 1 to 20, the end-of-input race, and the dispatch itself
//! (on the i9 from 8 threads; nowhere else, and never below 8).

mod common;
use common::*;
use mibz2::bwt::OpenCells;
use mibz2::parallel::{with_machine, Machine};
use mibz2::{BzReader, Error, ErrorKind, Kernels, StreamDecoder};
use std::io::Read;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

/// The sequential decoder as a reader: the bytes before its end or first error, and which
fn sequential(data: &[u8]) -> (Vec<u8>, Result<(), Error>) {
    read_all(BzReader::new(data.to_vec().as_slice()))
}

fn read_all<R: Read>(mut r: R) -> (Vec<u8>, Result<(), Error>) {
    let mut out = Vec::new();
    let res = match r.read_to_end(&mut out) {
        Ok(_) => Ok(()),
        Err(e) => Err(e
            .get_ref()
            .and_then(|x| x.downcast_ref::<Error>())
            .cloned()
            .expect("the decoder's own error")),
    };
    (out, res)
}

/// Whether this machine's kernels let the default decoder take the front end on the i9's
/// profile (`parscan_here` needs AVX2 and CLMUL on x86-64 Linux, as the service core's
/// predicate does on the same machine)
fn i9_kernels() -> bool {
    mibz2::parallel::service_target(10, Kernels::detect(), &Machine::service_target())
}

/// `dec` fed `data` cut at `cuts` (absolute byte offsets, ascending), then read to its end
/// or first error, within `limit` (a hang fails the test rather than stalling it)
fn drive_cuts(dec: StreamDecoder, data: &[u8], cuts: &[usize]) -> (Vec<u8>, Result<(), Error>) {
    let dec = Arc::new(dec);
    let (d_w, z, cuts) = (dec.clone(), data.to_vec(), cuts.to_vec());
    let feeder = std::thread::spawn(move || {
        let mut at = 0usize;
        for &c in cuts.iter().chain(std::iter::once(&z.len())) {
            let c = c.min(z.len());
            if c <= at {
                continue;
            }
            let mut b = d_w.spare();
            b.clear();
            b.extend_from_slice(&z[at..c]);
            at = c;
            if d_w.write(b).is_err() {
                return;
            }
        }
        d_w.close();
    });
    let (tx, rx) = mpsc::channel();
    let d_r = dec.clone();
    let reader = std::thread::spawn(move || {
        let mut out = Vec::new();
        let mut buf = vec![0u8; 1 << 16];
        let res = loop {
            match d_r.read(&mut buf) {
                Ok(0) => break Ok(()),
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => break Err(e),
            }
        };
        let _ = tx.send((out, res));
    });
    let got = rx
        .recv_timeout(Duration::from_secs(60))
        .expect("the decode finished (no hang)");
    reader.join().unwrap();
    dec.cancel();
    feeder.join().unwrap();
    got
}

/// The decoders under test at `t` threads: the default decoder on the i9's profile (which
/// takes the front end from 8 threads), `StreamDecoder::parscan`, and the front end's
/// helper topology forced on `t` workers (`parscan_with`)
fn decoders(t: usize) -> Vec<(String, Box<dyn Fn() -> StreamDecoder>)> {
    let k = Kernels::detect();
    vec![
        (
            format!("default on the i9 at {t}"),
            Box::new(move || {
                with_machine(Machine::service_target(), || {
                    StreamDecoder::new(t, 4 * t, k)
                })
            }),
        ),
        (
            format!("parscan at {t}"),
            Box::new(move || StreamDecoder::parscan(t, 4 * t, k)),
        ),
        (
            format!("parscan_with OpenCells at {t}"),
            Box::new(move || StreamDecoder::parscan_with::<OpenCells>(t, 4 * t, k)),
        ),
    ]
}

/// Random bytes: a block of them compresses to about its own size, so each block is longer
/// than the front end's coalescing floor (64 KiB) and a cut is a real slab edge
fn random_streams(rng: &mut Rng) -> Vec<u8> {
    let mut z = Vec::new();
    for (n, level) in [(330_000, 1u32), (120_000, 9), (210_000, 1), (5_000, 2)] {
        z.extend(compress(&rng.bytes(n), level));
    }
    z
}

/// Every candidate magic's byte (block, end and stream headers' `BZh`), by the front
/// end's own anchor scan
fn magic_bytes(z: &[u8]) -> Vec<usize> {
    let mut raw = Vec::new();
    mibz2::par_scan::find_magics_anchor_range(z, 1, 1 + (z.len() - 2) / 4, &mut raw);
    let mut v: Vec<usize> = raw.iter().map(|m| (m.bit / 8) as usize).collect();
    v.extend(
        z.windows(3)
            .enumerate()
            .filter(|(_, w)| w == b"BZh")
            .map(|(i, _)| i),
    );
    v.sort_unstable();
    v.dedup();
    v
}

#[test]
fn the_default_takes_the_front_end_on_the_i9_from_8_threads_only() {
    let k = Kernels::detect();
    for t in 1..=20 {
        // this machine (not the i9): the default decoder is the serial scanner's
        let here = StreamDecoder::new(t, 4 * t, k);
        assert_eq!(here.front_helpers(), None, "here at {t}");
        drop(here);
        let dec = with_machine(Machine::service_target(), || {
            StreamDecoder::new(t, 4 * t, k)
        });
        let want = if t >= 8 && i9_kernels() {
            Some(if dec.threads() >= 7 { 2 } else { 1 })
        } else {
            None
        };
        assert_eq!(dec.front_helpers(), want, "the i9 at {t}");
        if want.is_some() && t >= 10 {
            assert_eq!(
                dec.threads(),
                mibz2::parallel::PARSCAN_TRIMMED_WORKERS,
                "the i9's workers at {t}"
            );
        }
        // a forced plan is never replaced
        let forced = with_machine(Machine::service_target(), || {
            mibz2::parallel::with_plan(mibz2::parallel::Plan::Workers(t), || {
                StreamDecoder::new(t, 4 * t, k)
            })
        });
        assert_eq!(forced.front_helpers(), None, "forced at {t}");
    }
}

#[test]
fn multi_block_multi_stream_at_every_thread_count() {
    let mut rng = Rng(0x5eed_5747);
    let mut z = random_streams(&mut rng);
    let text: Vec<u8> = (0..400_000u32)
        .map(|i| b"the quick brown fox jumps over "[(i % 31) as usize] ^ ((i / 997) as u8 & 1))
        .collect();
    z.extend(compress(&text, 1));
    let (want, want_res) = sequential(&z);
    assert_eq!(want_res, Ok(()));
    for t in 1..=20 {
        for (name, make) in decoders(t) {
            let (got, res) = drive_cuts(make(), &z, &[]);
            assert_eq!(res, Ok(()), "{name}");
            assert!(got == want, "{name}: the bytes differ");
        }
        assert_eq!(
            mibz2::decode_parscan(&z, t).as_deref(),
            Ok(want.as_slice()),
            "decode_parscan at {t}"
        );
        let r = with_machine(Machine::service_target(), || {
            BzReader::parallel(std::io::Cursor::new(z.clone()), t)
        });
        let (got, res) = read_all(r);
        assert_eq!(res, Ok(()), "BzReader::parallel on the i9 at {t}");
        assert!(got == want, "BzReader::parallel on the i9 at {t}: differs");
    }
}

#[test]
fn magics_on_slab_edges() {
    let mut rng = Rng(0x5eed_ed9e);
    let z = random_streams(&mut rng);
    let (want, want_res) = sequential(&z);
    assert_eq!(want_res, Ok(()));
    let magics = magic_bytes(&z);
    assert!(magics.len() >= 10, "{} magics", magics.len());
    for d in -9i64..=9 {
        let cuts: Vec<usize> = magics
            .iter()
            .map(|&m| (m as i64 + d).max(1) as usize)
            .collect();
        for t in [2, 6, 8, 10, 20] {
            for (name, make) in decoders(t) {
                let (got, res) = drive_cuts(make(), &z, &cuts);
                assert_eq!(res, Ok(()), "{name}, cut {d} from each magic");
                assert!(got == want, "{name}, cut {d} from each magic: differs");
            }
        }
    }
}

/// Two crafted streams (levels 9 and 1) of small blocks each
fn small_streams() -> Vec<u8> {
    let mut z = craft(
        9,
        &[
            Spec::of(b"banana_bandana".to_vec(), 3),
            Spec::of(b"mississippi river".to_vec(), 5),
        ],
        None,
    );
    z.extend(craft(
        1,
        &[
            Spec::of(b"abracadabra".to_vec(), 2),
            Spec::of(b"zzzzzzzzzzzzzzzzzzzzzzzzzzzzyx".to_vec(), 7),
            Spec::of(b"tail".to_vec(), 1),
        ],
        None,
    ));
    z
}

#[test]
fn every_truncation_of_a_small_multi_stream_file() {
    let z = small_streams();
    assert_eq!(sequential(&z).1, Ok(()));
    let mut truncated = 0;
    for len in 0..z.len() {
        let cut = &z[..len];
        let (want, want_res) = sequential(cut);
        if matches!(&want_res, Err(e) if e.kind == ErrorKind::Truncated) {
            truncated += 1;
        }
        for t in [1, 2, 3, 8, 10, 12] {
            for (name, make) in decoders(t) {
                // fed whole, and a byte at a time from the second stream's header on
                for cuts in [vec![], (len / 2..len).collect::<Vec<_>>()] {
                    let (got, res) = drive_cuts(make(), cut, &cuts);
                    assert_eq!(res, want_res, "{name}, {len} of {} bytes", z.len());
                    assert!(got == want, "{name}, {len} bytes: differs");
                }
            }
            assert_eq!(
                mibz2::decode_parscan(cut, t),
                mibz2::decode(cut),
                "decode_parscan at {t}, {len} bytes"
            );
        }
    }
    assert!(truncated > z.len() / 2, "{truncated} truncated prefixes");
}

/// A block past an end magic is first read at level 9 (its stream's level unknown until the
/// chain reaches it). Here the second stream is level 1 and its block's original pointer is
/// past level 1's bound: the sequential decoder fails it at once (`BadOrigPtr`), before the
/// truncation behind it, and so must the front end, though its level-9 read ran on into
/// the truncation
#[test]
fn a_block_read_at_the_wrong_level_is_read_again() {
    let mut z = craft(9, &[Spec::of(b"banana_bandana".to_vec(), 3)], None);
    let first = z.len();
    z.extend(craft(
        1,
        &[Spec::of(b"abracadabra".to_vec(), 150_000)],
        None,
    ));
    // (the second block's header: 4 + 6 + 4 + 1 + 3 bytes, then a little of its map)
    for len in first + 18..z.len() {
        let cut = &z[..len];
        let (want, want_res) = sequential(cut);
        assert_eq!(
            want_res.as_ref().map_err(|e| e.kind.clone()),
            Err(ErrorKind::BadOrigPtr),
            "{len} bytes"
        );
        for t in [2, 8, 10, 20] {
            for (name, make) in decoders(t) {
                for _ in 0..4 {
                    let (got, res) = drive_cuts(make(), cut, &[]);
                    assert_eq!(res, want_res, "{name}, {len} bytes");
                    assert!(got == want, "{name}, {len} bytes: differs");
                }
            }
        }
    }
}

/// The last block and the end of input together, many times over, at the helper
/// topology: no worker may see the queue's end before the last block is queued, and no
/// wake-up may be lost (each decode has a deadline)
#[test]
fn the_end_of_input_race() {
    let mut rng = Rng(0x5eed_e0f0);
    let mut z = Vec::new();
    for i in 0..6u32 {
        let n = 2_000 + rng.below(30_000) as usize;
        let data: Vec<u8> = (0..n).map(|j| b"abcab"[(j + i as usize) % 5]).collect();
        z.extend(compress(&data, 1 + i % 9));
    }
    let (want, want_res) = sequential(&z);
    assert_eq!(want_res, Ok(()));
    for round in 0..150 {
        let t = [8, 10, 12, 16, 20][round % 5];
        let chunk = 1 + rng.below(9_000) as usize;
        let cuts: Vec<usize> = (1..z.len() / chunk + 1).map(|i| i * chunk).collect();
        for (name, make) in decoders(t) {
            let (got, res) = drive_cuts(make(), &z, &cuts);
            assert_eq!(res, Ok(()), "{name}, round {round}, chunk {chunk}");
            assert!(got == want, "{name}, round {round}: differs");
        }
    }
}
