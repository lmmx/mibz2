//! The parallel decoder's job handoff under stress: many blocks of a few bytes (each a job
//! the scanner reads in microseconds, so the workers wait for nearly every one), with a
//! few full blocks among them (so jobs also queue behind busy workers); the decode
//! stopped part way, with jobs queued and workers waiting; and errors among the small
//! blocks, which must still come in order, after exactly the bytes before them.

mod common;
use common::*;
use mibz2::{ErrorKind, Kernels, StreamDecoder};
use std::time::Duration;

/// (a lost wake-up or a missed close would hang: each decode fails at this bound instead)
const PATIENCE: Duration = Duration::from_secs(300);

/// `f` on a thread of its own, failing if it has not returned within `PATIENCE`
fn within<T: Send + 'static>(what: &str, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)));
    });
    match rx.recv_timeout(PATIENCE) {
        Ok(Ok(v)) => v,
        Ok(Err(p)) => std::panic::resume_unwind(p),
        Err(_) => panic!("{what}: no end within {PATIENCE:?} (a worker never woken?)"),
    }
}

/// `n` streams, each of one small block (a few to a few hundred bytes), a stream of no
/// blocks every 50th, and, if `big`, a full level-9 block of text every 400th; their
/// decoded bytes, each stream's, in order
fn small_blocks(n: usize, big: bool) -> (Vec<u8>, Vec<Vec<u8>>) {
    let mut rng = Rng(0x5eed_b10c);
    let text = samples(&mut rng).swap_remove(3).1;
    let (mut z, mut parts) = (Vec::new(), Vec::new());
    for i in 0..n {
        let data: Vec<u8> = if i % 50 == 49 {
            Vec::new()
        } else if big && i % 400 == 199 {
            text[..900_000.min(text.len())].to_vec()
        } else {
            let len = 1 + rng.below(300) as usize;
            let mut d = format!("block {i}: ").into_bytes();
            d.extend((0..len).map(|j| b"abcab\n"[(i + j) % 6]));
            d
        };
        let level = if data.len() > 1000 {
            9
        } else {
            1 + (i % 9) as u32
        };
        z.extend_from_slice(&compress(&data, level));
        parts.push(data);
    }
    (z, parts)
}

/// The decode of `z` (written in `chunk`-byte pieces, read `read` bytes at a time): the
/// bytes read, and the error that ended it, if one did
fn decode(
    z: &[u8],
    threads: usize,
    slots: usize,
    chunk: usize,
    read: usize,
) -> (Vec<u8>, Option<mibz2::Error>) {
    let dec = StreamDecoder::new(threads, slots, Kernels::detect());
    let mut out = Vec::new();
    let err = std::thread::scope(|s| {
        s.spawn(|| {
            for c in z.chunks(chunk) {
                if dec.write(c.to_vec()).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut buf = vec![0u8; read];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break None,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => {
                    // (the same error again: it stays the reader's)
                    assert_eq!(dec.read(&mut buf).unwrap_err(), e);
                    dec.cancel();
                    break Some(e);
                }
            }
        }
    });
    (out, err)
}

#[test]
fn many_small_blocks_in_order() {
    let (z, parts) = small_blocks(3000, true);
    let want = parts.concat();
    for (threads, slots, chunk, read) in [
        (1, 1, 4096, 1 << 16),
        (2, 2, 97, 333),
        (3, 12, 1 << 20, 1),
        (4, 4, 64, 4096),
        (4, 16, 4096, 1 << 16),
        (8, 8, 1000, 7),
        (8, 32, 1 << 16, 1 << 20),
        (16, 16, 512, 999),
    ] {
        let z = z.clone();
        let (got, err) = within("decode", move || decode(&z, threads, slots, chunk, read));
        assert!(err.is_none(), "{threads}t window {slots}: {err:?}");
        assert!(
            got == want,
            "{threads}t window {slots} chunk {chunk} read {read}: differs"
        );
    }
}

/// The same small window decoded many times over, for the races between a job sent, a
/// worker going back to wait, and one woken
#[test]
fn many_small_blocks_repeated() {
    let (z, parts) = small_blocks(600, false);
    let want = parts.concat();
    for round in 0..20 {
        let threads = [2, 4, 8][round % 3];
        let z = z.clone();
        let (got, err) = within("decode", move || decode(&z, threads, threads, 256, 4096));
        assert!(
            err.is_none() && got == want,
            "round {round}, {threads}t: {err:?}"
        );
    }
}

/// Stopped part way, with jobs queued and workers waiting: cancelled by the reader while
/// the writer writes; dropped with the input left open (the scanner waiting for more
/// input, the workers for jobs); dropped with the input closed and nothing read (the
/// window full). Each ends, and nothing read is wrong.
#[test]
fn stopped_mid_stream() {
    let (z, parts) = small_blocks(1500, true);
    let want = parts.concat();
    for (i, threads) in [1, 2, 4, 8, 16].into_iter().enumerate() {
        // the reader cancels after some bytes; the writer's next write fails
        for stop in [0, 1, 5_000, 100_000, 1_000_000] {
            let (z, want) = (z.clone(), want.clone());
            within("cancel", move || {
                let dec = StreamDecoder::new(threads, threads, Kernels::detect());
                std::thread::scope(|s| {
                    s.spawn(|| {
                        for c in z.chunks(300 + 1000 * i) {
                            if dec.write(c.to_vec()).is_err() {
                                return;
                            }
                        }
                        dec.close();
                    });
                    let mut got = Vec::new();
                    let mut buf = vec![0u8; 777];
                    let mut end = false;
                    while got.len() < stop && !end {
                        match dec.read(&mut buf).unwrap() {
                            0 => end = true,
                            n => got.extend_from_slice(&buf[..n]),
                        }
                    }
                    assert_eq!(&got[..], &want[..got.len()]);
                    dec.cancel();
                    // (the next read fails, unless the reader had read to the end)
                    match dec.read(&mut buf) {
                        Ok(0) if end => assert_eq!(got.len(), want.len()),
                        r => assert_eq!(r.unwrap_err().kind, ErrorKind::Cancelled),
                    }
                });
                drop(dec);
            });
        }
        // the input left open: copies written at once, well past the scanner's 2 MB
        // lookahead (so it issues blocks), a little read, then the decoder dropped
        let copies = (4 << 20) / z.len() + 1;
        let (zz, want3) = (z.repeat(copies), want.repeat(copies));
        within("drop, input open", move || {
            let dec = StreamDecoder::new(threads, 4 * threads, Kernels::detect());
            dec.write(zz).unwrap();
            let mut got = Vec::new();
            let mut buf = vec![0u8; 100];
            while got.len() < 1000 {
                let n = dec.read(&mut buf).unwrap();
                assert!(n > 0);
                got.extend_from_slice(&buf[..n]);
            }
            assert_eq!(&got[..], &want3[..got.len()]);
            drop(dec);
        });
        // the input closed and nothing read: dropped with the window full of blocks
        // decoded or queued
        let zz = z.clone();
        within("drop, nothing read", move || {
            let dec = StreamDecoder::new(threads, 2 * threads, Kernels::detect());
            dec.write(zz).unwrap();
            dec.close();
            drop(dec);
        });
        // nothing written at all
        within("drop, no input", move || {
            drop(StreamDecoder::new(threads, threads, Kernels::detect()));
        });
    }
}
/// A bad block among the small ones, or the input cut inside one: the bytes read are
/// exactly the streams' before it, then its error (the block's place and first bit for a
/// block's own error), whatever the threads and window
#[test]
fn errors_in_order_among_small_blocks() {
    let (z, parts) = small_blocks(1200, true);
    let starts = block_starts(&z);
    // the streams with a block, in order: the block's index is its place in the decode
    let with_block: Vec<usize> = (0..parts.len()).filter(|&i| !parts[i].is_empty()).collect();
    assert_eq!(starts.len(), with_block.len());
    for &b in &[0, 1, 2, 500, 777, with_block.len() - 1] {
        let stream = with_block[b];
        let want = parts[..stream].concat();
        // the block's stored CRC (32 bits after its 48-bit magic), a bit flipped
        let mut bad = z.clone();
        let bit = starts[b] + 48 + 5;
        bad[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
        // the input cut 7 bytes into the block (inside its header)
        let cut = z[..(starts[b] / 8) as usize + 7].to_vec();
        for (threads, slots, chunk) in [(1, 1, 4096), (4, 4, 61), (4, 16, 1 << 16), (8, 32, 512)] {
            let bad2 = bad.clone();
            let (got, err) = within("decode", move || decode(&bad2, threads, slots, chunk, 1000));
            let e = err.expect("an error");
            assert!(
                matches!(e.kind, ErrorKind::BlockCrc { .. }),
                "block {b}, {threads}t: {e:?}"
            );
            assert_eq!((e.block, e.bit), (Some(b as u64), Some(starts[b])));
            assert!(
                got == want,
                "block {b}, {threads}t: the bytes before differ"
            );

            let cut2 = cut.clone();
            let (got, err) = within("decode", move || decode(&cut2, threads, slots, chunk, 1000));
            let e = err.expect("an error");
            assert_eq!(
                e.kind,
                mibz2::decode(&cut).unwrap_err().kind,
                "cut at block {b}"
            );
            assert!(
                got == want,
                "cut at block {b}, {threads}t: the bytes before differ"
            );
        }
    }
}
