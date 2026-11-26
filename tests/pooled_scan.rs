//! The pooled scan (`pooled-scan`, research only): exact against the sequential decoder and
//! the shipped parallel decoder, bytes and errors (`Error { kind, block, bit }`), on every
//! path its chain can take: the pooled path itself, the shipped path below level 9, and the
//! paths for a false and a missed magic (`pooled_scan::set_test_noise`).
#![cfg(feature = "pooled-scan")]

mod common;

use common::*;
use mibz2::{pooled_scan, Error, Kernels, StreamDecoder};
use std::sync::Mutex;

/// The mode and the counters are process-wide: one test at a time
static LOCK: Mutex<()> = Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// The plain workers' decoder, pooled or as shipped, fed `chunk` and read `read` bytes at a time
fn run(
    z: &[u8],
    threads: usize,
    slots: usize,
    chunk: usize,
    read: usize,
    pooled: bool,
) -> (Vec<u8>, Result<(), Error>) {
    pooled_scan::set(pooled);
    let dec = StreamDecoder::workers(threads, slots, Kernels::detect(), true);
    pooled_scan::set(false);
    drive(&dec, z, chunk, read)
}

/// The sequential decoder's output, or its error
fn seq(z: &[u8]) -> Result<Vec<u8>, Error> {
    ours(z, Kernels::detect()).expect("no panic")
}

fn pooled_is_shipped(
    z: &[u8],
    threads: usize,
    slots: usize,
    chunk: usize,
    read: usize,
    what: &str,
) {
    let want = run(z, threads, slots, chunk, read, false);
    let got = run(z, threads, slots, chunk, read, true);
    assert_eq!(got.1, want.1, "{what}: the result");
    assert!(got.0 == want.0, "{what}: the bytes before it");
    if let Ok(()) = got.1 {
        assert!(
            Ok(got.0) == seq(z),
            "{what}: against the sequential decoder"
        );
    } else {
        let s = seq(z).unwrap_err();
        assert_eq!(
            got.1.unwrap_err().kind,
            s.kind,
            "{what}: the kind, against the sequential decoder"
        );
    }
}

fn text(n: usize) -> Vec<u8> {
    let mut rng = Rng(5);
    let t = samples(&mut rng).swap_remove(3).1;
    t.iter().cycle().take(n).copied().collect()
}

#[test]
fn the_corpora_at_levels_9_and_1() {
    let _g = lock();
    for (name, data) in corpora(1) {
        for level in [9, 1] {
            let z = compress(&data, level);
            let blocks = block_starts(&z).len() as u64;
            for (threads, slots) in [(2, 4), (4, 8), (8, 16)] {
                let _ = pooled_scan::take();
                let (out, res) = run(&z, threads, slots, 1 << 20, 1 << 16, true);
                let st = pooled_scan::take();
                assert!(res.is_ok(), "{name} level {level} {threads}t: {res:?}");
                assert!(out == data, "{name} level {level} {threads}t: bytes");
                if level == 9 {
                    // every block from a worker's read: none read by the chain, none dropped
                    assert_eq!(
                        (st.accepted, st.fallbacks, st.degraded, st.rejected),
                        (blocks, 0, 0, 0),
                        "{name} level 9 {threads}t: {st:?}"
                    );
                } else {
                    // below level 9: the shipped path
                    assert_eq!(
                        (st.accepted, st.fallbacks),
                        (0, blocks),
                        "{name} level 1: {st:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn streams_of_mixed_levels() {
    let _g = lock();
    let t = text(2_500_000);
    let parts: Vec<(usize, u32)> = vec![(0, 9), (900_000, 9), (1_700_000, 1), (1_800_000, 9)];
    let mut z = Vec::new();
    let mut want = Vec::new();
    for (i, &(from, level)) in parts.iter().enumerate() {
        let to = parts.get(i + 1).map_or(t.len(), |p| p.0);
        z.extend(compress(&t[from..to], level));
        want.extend_from_slice(&t[from..to]);
    }
    // and an empty stream between
    z.extend(compress(b"", 9));
    for (threads, slots) in [(2, 2), (3, 6), (8, 16)] {
        let (out, res) = run(&z, threads, slots, 1 << 18, 1 << 15, true);
        assert!(res.is_ok() && out == want, "{threads}t: {res:?}");
    }
    pooled_is_shipped(&z, 4, 8, 4096, 999, "mixed levels");
}

#[test]
fn any_chunking() {
    let _g = lock();
    let t = text(3_000_000);
    let z = compress(&t, 9);
    for (threads, slots, chunk, read) in [
        (2, 2, 4096, 1 << 20),
        (4, 8, 7_001, 3),
        (3, 3, 65_537, 999),
        (8, 16, 1 << 23, 16 << 20),
        (2, 4, 1_000_003, 1),
    ] {
        let (out, res) = run(&z, threads, slots, chunk, read, true);
        assert!(
            res.is_ok() && out == t,
            "{threads}t chunk {chunk} read {read}: {res:?}"
        );
    }
    // a byte at a time: a short input
    let small = compress(&t[..120_000], 9);
    let (out, res) = run(&small, 2, 2, 1, 1 << 12, true);
    assert!(
        res.is_ok() && out == t[..120_000],
        "byte at a time: {res:?}"
    );
}

#[test]
fn false_and_missed_magics() {
    let _g = lock();
    let t = text(4_000_000);
    let mut z = compress(&t, 9);
    z.extend(compress(&t[..1_000_000], 9));
    let mut want = t.clone();
    want.extend_from_slice(&t[..1_000_000]);
    for (fake, drop) in [(1, 0), (3, 0), (0, 1), (0, 2), (2, 3), (5, 4)] {
        for (threads, slots) in [(2, 4), (4, 8)] {
            pooled_scan::set_test_noise(fake, drop);
            let _ = pooled_scan::take();
            let (out, res) = run(&z, threads, slots, 1 << 18, 1 << 16, true);
            let st = pooled_scan::take();
            pooled_scan::set_test_noise(0, 0);
            assert!(
                res.is_ok() && out == want,
                "fake every {fake}, drop every {drop}, {threads}t: {res:?}"
            );
            assert!(
                st.rejected + st.fallbacks > 0,
                "the noise reached the chain: {st:?}"
            );
        }
    }
}

#[test]
fn truncations_are_the_shipped_decoders() {
    let _g = lock();
    let t = text(2_000_000);
    let z = compress(&t, 9);
    let starts = block_starts(&z);
    let mut cuts: Vec<usize> = (0..z.len()).step_by(z.len() / 53).collect();
    // each block's first bytes and the stream's last, where a cut is subtle
    for &b in &starts {
        let b = (b / 8) as usize;
        cuts.extend(
            [b, b + 1, b + 5, b + 6, b + 7, b + 40]
                .into_iter()
                .filter(|&c| c < z.len()),
        );
    }
    cuts.extend((z.len().saturating_sub(12))..z.len());
    for cut in cuts {
        pooled_is_shipped(&z[..cut], 3, 6, 1 << 14, 1 << 14, &format!("cut {cut}"));
    }
}

#[test]
fn mutations_are_the_shipped_decoders() {
    let _g = lock();
    let t = text(1_600_000);
    let z = compress(&t, 9);
    let mut rng = Rng(0x5eed_1234_abcd_ef01);
    for case in 0..240 {
        let mut m = z.clone();
        let flips = 1 + rng.below(3);
        for _ in 0..flips {
            let i = rng.below(m.len() as u64) as usize;
            m[i] ^= 1 << rng.below(8);
        }
        pooled_is_shipped(&m, 4, 8, 1 << 16, 1 << 16, &format!("case {case}"));
    }
}

#[test]
fn headers_ends_and_trailing_data() {
    let _g = lock();
    let z = compress(&text(1_000_000), 9);
    let cases: Vec<Vec<u8>> = vec![
        Vec::new(),
        b"BZh9".to_vec(),
        b"BZh0".to_vec(),
        b"PK\x03\x04".to_vec(),
        [z.clone(), b"x".to_vec()].concat(),
        [z.clone(), b"BZh9".to_vec()].concat(),
        [z.clone(), b"garbage".to_vec()].concat(),
        compress(b"", 9),
        compress(b"a", 9),
    ];
    for (i, c) in cases.iter().enumerate() {
        pooled_is_shipped(c, 2, 4, 1 << 12, 1 << 12, &format!("case {i}"));
    }
}

#[test]
fn one_worker_is_the_shipped_path() {
    let _g = lock();
    let z = compress(&text(2_000_000), 9);
    let _ = pooled_scan::take();
    let (_, res) = run(&z, 1, 1, 1 << 20, 1 << 20, true);
    assert!(res.is_ok());
    assert_eq!(pooled_scan::take().dispatched, 0);
}

#[test]
fn the_reader_stops_early() {
    let _g = lock();
    let t = text(6_000_000);
    let z = compress(&t, 9);
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for threads in [2, 4, 8] {
            pooled_scan::set(true);
            let dec = StreamDecoder::workers(threads, 2 * threads, Kernels::detect(), true);
            pooled_scan::set(false);
            std::thread::scope(|s| {
                s.spawn(|| {
                    for c in z.chunks(1 << 16) {
                        if dec.write(c.to_vec()).is_err() {
                            return;
                        }
                    }
                    dec.close();
                });
                let mut buf = vec![0u8; 1 << 16];
                let n = dec.read(&mut buf).expect("a first read");
                assert!(n > 0);
                dec.cancel();
            });
            drop(dec);
        }
        tx.send(()).unwrap();
    });
    rx.recv_timeout(std::time::Duration::from_secs(60))
        .expect("the decoders stop and drop without a hang");
}
