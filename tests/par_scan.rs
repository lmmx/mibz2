//! The parallel front end (`par_scan::decode_parallel_slice_with`, `decode_parscan`, the
//! `mibz2-parscan` arm) against the sequential decoder: the same bytes before the end or
//! the first error, and the same result (`Error { kind, block, bit }`), on valid streams of
//! every shape, on files with false magics and false stream headers, and on mutated files;
//! and a panic in the caller's callback reaches the caller, the workers stopped.
//! examples/par_scan.rs's `check`, `adversarial` and `fuzz` hold its own copy of the
//! harness to the same; this holds the library's.

mod common;
use common::*;
use mibz2::{Error, Kernels};
use std::io::Read;

/// The sequential decoder as a reader: the bytes before its end or first error, and which
fn sequential(data: &[u8]) -> (Vec<u8>, Result<(), Error>) {
    let mut out = Vec::new();
    let res = match mibz2::BzReader::new(data).read_to_end(&mut out) {
        Ok(_) => Ok(()),
        Err(e) => Err(e
            .get_ref()
            .and_then(|x| x.downcast_ref::<Error>())
            .cloned()
            .expect("the decoder's own error")),
    };
    (out, res)
}

/// The front end's chunks up to its end or first error, and that result
fn front_end(data: &[u8], workers: usize, slots: usize) -> (Vec<u8>, Result<(), Error>) {
    let mut out = Vec::new();
    let res =
        mibz2::par_scan::decode_parallel_slice_with(data, workers, slots, Kernels::detect(), |c| {
            out.extend_from_slice(c)
        });
    (out, res)
}

/// The front end at `workers` and `slots` against the sequential decoder, and
/// `decode_parscan` against `decode`
fn same(name: &str, data: &[u8], workers: usize, slots: usize) {
    let (want, want_res) = sequential(data);
    let (got, res) = front_end(data, workers, slots);
    assert!(
        got == want,
        "{name} ({workers} workers, {slots} slots): the bytes differ ({} against {})",
        got.len(),
        want.len()
    );
    assert_eq!(
        res, want_res,
        "{name} ({workers} workers, {slots} slots): the result"
    );
    let whole = mibz2::decode(data);
    assert_eq!(
        mibz2::decode_parscan(data, workers),
        whole,
        "{name}: decode_parscan at {workers}"
    );
}

/// The bytes whose in-use bitmap, read as 16-bit words from range 0, is `words`
fn bytes_for_map(words: &[u16]) -> Vec<u8> {
    let mut v = Vec::new();
    for (i, w) in words.iter().enumerate() {
        for j in 0..16 {
            if w & (0x8000 >> j) != 0 {
                v.push((i * 16 + j) as u8);
            }
        }
    }
    v
}

/// A valid .bz2 every block of which holds, in its header's in-use bitmap, "BZh9" and the
/// block magic, back to back, then the end magic (examples/par_scan.rs's `magic_in_map`)
fn magic_in_map(rng: &mut Rng, n: usize, level: u32) -> Vec<u8> {
    let set = bytes_for_map(&[
        0x425a, 0x6839, 0x3141, 0x5926, 0x5359, 0x1772, 0x4538, 0x5090,
    ]);
    let mut v: Vec<u8> = Vec::with_capacity(n);
    for i in 0..n {
        let mut b = if i % 2000 < set.len() {
            set[i % 2000]
        } else {
            let (x, y) = (rng.below(set.len() as u64), rng.below(set.len() as u64));
            set[x.min(y) as usize]
        };
        if v.ends_with(&[b, b, b]) {
            b = set[(set.iter().position(|&x| x == b).unwrap() + 1) % set.len()];
        }
        v.push(b);
    }
    compress(&v, level)
}

#[test]
fn valid_streams_of_every_shape() {
    let mut rng = Rng(0x5eed_0037);
    for (name, data) in samples(&mut rng) {
        // (a prefix of the large ones: a few blocks at level 1 is enough)
        let data = &data[..data.len().min(400_000)];
        for level in [1, 9] {
            let z = compress(data, level);
            for (workers, slots) in [(1, 1), (2, 2), (3, 12), (4, 16), (8, 1)] {
                let (got, res) = front_end(&z, workers, slots);
                assert_eq!(res, Ok(()), "{name} level {level} at {workers}");
                assert!(got == data, "{name} level {level} at {workers}: differs");
            }
        }
    }
}

#[test]
fn false_magics_and_headers() {
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    let text = samples(&mut rng).swap_remove(3).1;
    // both magics and a false stream header inside every block
    let a = magic_in_map(&mut rng, 300_000, 1);
    for (w, s) in [(1, 1), (3, 12), (4, 4)] {
        same("magics in the in-use bitmap", &a, w, s);
    }
    // a whole valid block (and a stream end) spliced over a block's payload
    let host = compress(&text[..400_000], 1);
    let starts = block_starts(&host);
    let tiny = compress(b"a valid block where no block is", 9);
    let mut b = host.clone();
    let mid = ((starts[1] + starts[2]) / 16) as usize;
    b[mid..mid + tiny.len() - 4].copy_from_slice(&tiny[4..]);
    same("a valid block spliced into a payload", &b, 3, 12);
    // stray bytes between streams; an empty stream; streams of other levels
    let mut c = compress(&text[..100_000], 9);
    c.extend_from_slice(b"junk");
    c.extend_from_slice(&host);
    same("a stream, 4 stray bytes, a stream", &c, 3, 12);
    let mut d = host.clone();
    d.extend_from_slice(&compress(b"", 9));
    d.extend_from_slice(&a);
    d.extend_from_slice(&compress(&text[..30_000], 5));
    same("several streams", &d, 4, 8);
    // cut inside a block, and inside the end marker's CRC
    same(
        "cut inside a block",
        &host[..(starts[2] / 8 + 1000) as usize],
        3,
        12,
    );
    same("cut inside the stream CRC", &host[..host.len() - 2], 3, 12);
    // a randomised block: the bit after a block's CRC set
    let mut e = host.clone();
    let at = starts[2] + 80;
    e[(at / 8) as usize] |= 0x80 >> (at % 8);
    same("a block marked randomised", &e, 3, 12);
    // not a .bz2, and nothing at all
    same("not a .bz2", b"not a bz2 file at all", 2, 2);
    same("empty", b"", 2, 2);
}

#[test]
fn mutated_files() {
    let mut rng = Rng(0x0123_4567_89ab_cdef);
    let text = samples(&mut rng).swap_remove(3).1;
    let mut base = compress(&text[..120_000], 1);
    base.extend_from_slice(&compress(b"", 9));
    base.extend_from_slice(&magic_in_map(&mut rng, 120_000, 1));
    base.extend_from_slice(&compress(&text[..30_000], 9));
    let tiny = compress(b"a valid block where no block is", 9);
    let magics = [
        mibz2::block::BLOCK_MAGIC.to_be_bytes(),
        mibz2::block::END_MAGIC.to_be_bytes(),
    ];
    let starts = block_starts(&base);
    same("the file whole", &base, 3, 12);
    let cases = if cfg!(debug_assertions) { 120 } else { 600 };
    for case in 0..cases {
        let mut d = base.clone();
        for _ in 0..1 + rng.below(2) {
            if d.is_empty() {
                break;
            }
            let at = rng.below(d.len() as u64) as usize;
            match rng.below(9) {
                0 | 1 => d[at] ^= 1 << rng.below(8),
                2 => d.truncate(at),
                // a magic written over the input, or put into it
                3 => {
                    let m = &magics[rng.below(2) as usize][2..];
                    let n = m.len().min(d.len() - at);
                    d[at..at + n].copy_from_slice(&m[..n]);
                }
                4 => {
                    let m = magics[rng.below(2) as usize];
                    d.splice(at..at, m[2..].iter().copied());
                }
                // a whole valid block and stream end written over it
                5 => {
                    let n = (tiny.len() - 4).min(d.len() - at);
                    d[at..at + n].copy_from_slice(&tiny[4..4 + n]);
                }
                // bytes removed, or a block's bytes repeated
                6 => {
                    let n = (1 + rng.below(3000) as usize).min(d.len() - at);
                    d.drain(at..at + n);
                }
                7 => {
                    let i = rng.below(starts.len() as u64 - 1) as usize;
                    let (a, b) = ((starts[i] / 8) as usize, (starts[i + 1] / 8) as usize);
                    let (a, b) = (a.min(d.len()), b.min(d.len()));
                    let part = d[a..b].to_vec();
                    d.splice(a..a, part);
                }
                // a stream header's level changed
                _ => {
                    let hs: Vec<usize> = (0..d.len().saturating_sub(4))
                        .filter(|&i| &d[i..i + 3] == b"BZh")
                        .collect();
                    if !hs.is_empty() {
                        let i = hs[rng.below(hs.len() as u64) as usize];
                        d[i + 3] = b'0' + rng.below(11) as u8;
                    }
                }
            }
        }
        let workers = 1 + rng.below(4) as usize;
        let slots = 1 + rng.below(12) as usize;
        same(&format!("case {case}"), &d, workers, slots);
    }
}

#[test]
fn a_panic_in_the_callback_reaches_the_caller() {
    let mut rng = Rng(9);
    let text = samples(&mut rng).swap_remove(3).1;
    let z = compress(&text[..400_000], 1);
    for workers in [1, 4] {
        let r = std::panic::catch_unwind(|| {
            mibz2::par_scan::decode_parallel_slice_with(
                &z,
                workers,
                4 * workers,
                Kernels::detect(),
                |_| panic!("the callback's panic"),
            )
        });
        let e = r.expect_err("the panic propagates");
        assert_eq!(e.downcast_ref::<&str>(), Some(&"the callback's panic"));
    }
}
