//! The inverse BWT in place (`InverseBwt::invert_in_place`, the production-shaped P87:
//! docs/journal/2026-11-01-p87-production-protocol.md). `Segmented` writes each block's bytes
//! over `ll` once its table is built, so the worker never allocates `rle`. These check that
//! it decodes exactly what libbzip2 does, and what the separate-buffer path (`Plain`, which
//! still inverts into `rle`) does, on every sample shape:
//! - at block sizes from short (the plain walk) to full;
//! - with one scratch reused across streams of different levels;
//! - through the parallel decoder, including corrupt input and a decode cancelled mid-stream;
//! - on the error paths, ending in the same error after the same output.
//!
//! With the `trace` feature, they also check from the scratch's own buffers that `rle` is
//! never allocated, and how `ll` grows across levels.

mod common;
use common::{compress, craft, oracle, Rng, Spec};
use mibz2::bwt::{Cooperative, InverseBwt, Plain, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_with, ErrorKind, Kernels, Scratch, StreamDecoder};

/// `decode_with`'s loop with the caller's scratch: the output (or the error and the output
/// kept) and the scratch after it
fn run<I: InverseBwt>(data: &[u8], s: &mut Scratch<I>) -> (Vec<u8>, Option<ErrorKind>) {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut out = Vec::new();
    let mut syms = Vec::new();
    loop {
        match sc.next(syms) {
            Ok(Event::Block { symbols, .. }) => {
                if let Err(e) = mibz2::decode_block(&symbols, k, s, &mut out) {
                    return (out, Some(e));
                }
                syms = symbols.syms;
            }
            Ok(Event::Done) => return (out, None),
            Err(e) => return (out, Some(e.kind)),
        }
    }
}

#[test]
fn every_sample_decodes_exactly_in_place() {
    let k = Kernels::detect();
    let mut rng = Rng(0x87_0e);
    for (name, data) in common::samples(&mut rng) {
        for level in [1, 9] {
            let z = compress(&data, level);
            assert!(oracle(&z).unwrap() == data, "{name} -{level}: the oracle");
            let in_place = decode_with::<Segmented>(&z, k).unwrap();
            assert!(in_place == data, "{name} -{level}: in place");
            let separate = decode_with::<Plain>(&z, k).unwrap();
            assert!(separate == data, "{name} -{level}: the separate buffer");
        }
    }
}

#[test]
fn a_scratch_reused_across_levels_and_short_blocks() {
    // streams at levels 1, 9, 2 and 9 again (blocks of 100k, 900k, 200k rows), each ending
    // in a short block (the plain walk, under MIN_SEGMENTED rows); one scratch throughout
    let mut rng = Rng(0x87_0f);
    let samples = common::samples(&mut rng);
    let text = &samples.iter().find(|(n, _)| n == "text").unwrap().1;
    let runs = &samples.iter().find(|(n, _)| n == "runs").unwrap().1;
    let mut z = Vec::new();
    let mut want = Vec::new();
    for (level, part) in [
        (1, &text[..350_000]),
        (9, &text[..]),
        (2, &runs[..]),
        (9, &text[..70_000]),
    ] {
        z.extend(compress(part, level));
        want.extend_from_slice(part);
    }
    assert!(oracle(&z).unwrap() == want);
    let mut s = Scratch::<Segmented>::default();
    let (out, err) = run(&z, &mut s);
    assert_eq!(err, None);
    assert!(out == want, "in place, one scratch");
}

#[test]
fn errors_end_as_the_separate_buffer_path_does() {
    // a good block, then a wrong CRC, or a primary index past the block (each its own case)
    let mut rng = Rng(0x87_10);
    let good: Vec<u8> = rng.bytes(200_000);
    let good_z = compress(&good, 9);
    let ll: Vec<u8> = (0..100_000u32).map(|i| (i * 7 % 251) as u8).collect();
    let mut bad_crc = Spec::of(ll.clone(), 17);
    bad_crc.crc = Some(0x1234_5678);
    for (i, spec) in [bad_crc, Spec::of(ll.clone(), ll.len() as u32)]
        .into_iter()
        .enumerate()
    {
        let mut z = good_z.clone();
        z.extend(craft(9, &[spec], None));
        let (sep, sep_err) = run(&z, &mut Scratch::<Plain>::default());
        let (inp, inp_err) = run(&z, &mut Scratch::<Segmented>::default());
        assert!(sep_err.is_some(), "case {i}: decoded");
        assert_eq!(inp_err, sep_err, "case {i}: the error");
        assert!(inp == sep && sep == good, "case {i}: the output kept");
    }
}

/// `z` through a 4-worker decoder of kernel `I`: the bytes read, and the error
fn parallel<I: InverseBwt + 'static>(z: &[u8]) -> (Vec<u8>, Option<String>) {
    let dec = StreamDecoder::with_inverse::<I>(4, 8, Kernels::detect());
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
fn the_parallel_decoder_in_place() {
    let mut rng = Rng(0x87_11);
    let samples = common::samples(&mut rng);
    let text = &samples.iter().find(|(n, _)| n == "text").unwrap().1;
    let runs = &samples.iter().find(|(n, _)| n == "runs").unwrap().1;
    for (name, data, level) in [("text", text, 9), ("text", text, 1), ("runs", runs, 9)] {
        let z = compress(data, level);
        let (out, err) = parallel::<Segmented>(&z);
        assert_eq!(err, None, "{name} -{level}");
        assert!(&out == data, "{name} -{level}: in place, 4 workers");
        // the cooperative kernel keeps its separate buffer, unchanged
        let (coop, err) = parallel::<Cooperative<2>>(&z);
        assert_eq!(err, None);
        assert!(&coop == data, "{name} -{level}: cooperative");
    }
    // a corrupt stream ends in the separate-buffer path's error, after its bytes
    let mut bad = compress(text, 1);
    let at = common::block_starts(&bad)[4] as usize / 8 + 200;
    bad[at] ^= 0x5a;
    let (inp, inp_err) = parallel::<Segmented>(&bad);
    let (sep, sep_err) = parallel::<Plain>(&bad);
    assert!(inp_err.is_some(), "corrupt input decoded");
    assert_eq!(inp_err, sep_err, "the error");
    assert!(inp == sep, "the bytes before it");
    // cancelled after its first bytes, the feeder still writing: no hang, no panic
    let z = compress(text, 9);
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
        assert!(dec.read(&mut buf).unwrap() > 0, "the first bytes");
        dec.cancel();
        assert!(dec.read(&mut buf).is_err(), "a read after cancel");
    });
    dec.join();
}

/// The scratch's `ll` and `rle` capacities (the footprint ledger's, `trace` feature)
#[cfg(feature = "trace")]
fn capacities<I: InverseBwt>(s: &Scratch<I>) -> (usize, usize) {
    let b = s.buffers();
    let cap = |n: &str| b.iter().find(|x| x.name == n).unwrap().cap;
    (cap("ll"), cap("rle"))
}

#[cfg(feature = "trace")]
#[test]
fn rle_is_never_allocated_in_place_and_ll_grows_with_the_level() {
    let mut rng = Rng(0x87_12);
    let samples = common::samples(&mut rng);
    let text = &samples.iter().find(|(n, _)| n == "text").unwrap().1;
    // level 1 first: ll holds a 100k-row block
    let mut s = Scratch::<Segmented>::default();
    let (out, err) = run(&compress(&text[..400_000], 1), &mut s);
    assert!(err.is_none() && out == text[..400_000]);
    let (ll1, rle1) = capacities(&s);
    assert_eq!(rle1, 0, "rle allocated at level 1");
    assert!((100_000..900_000).contains(&ll1), "ll at level 1: {ll1}");
    // then level 9 with the same scratch: ll grows to the 900k block, rle still unallocated
    let (out, err) = run(&compress(text, 9), &mut s);
    assert!(err.is_none() && &out == text);
    let (ll9, rle9) = capacities(&s);
    assert_eq!(rle9, 0, "rle allocated at level 9");
    assert!(ll9 >= 900_000, "ll at level 9: {ll9}");
    // the separate-buffer path, for contrast, allocates rle
    let mut p = Scratch::<Plain>::default();
    run(&compress(text, 9), &mut p);
    assert!(capacities(&p).1 > 0, "Plain's rle");
}
