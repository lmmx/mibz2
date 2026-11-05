//! The inverse BWT in place (`InverseBwt::invert_in_place`, the production-shaped P87:
//! docs/journal/2026-11-01-p87-production-protocol.md). `Segmented` writes each block's bytes
//! over `ll` once its table is built, so the worker never allocates `rle`. These check that
//! it decodes exactly what libbzip2 does, and what the separate-buffer path (`Plain`, which
//! still inverts into `rle`) does, on every sample shape:
//! - at block sizes from short (the plain walk) to full;
//! - with one scratch reused across streams of different levels;
//! - through the parallel decoder, including corrupt input and a decode cancelled mid-stream;
//! - on the error paths, ending in the same error after the same output, and with a
//!   scratch reused after an error;
//! - against `Segmented::invert` into a buffer of its own, block by block (the algorithm
//!   held fixed, only the output's storage changed), with `ll`'s buffer unmoved;
//! - for a kernel that falls back (`Plain`, `Cooperative`): false, and nothing changed;
//! - for the split (`InverseBwt::invert_split`, RLE1 over the walk's segments with no copy
//!   joining them): the same bytes, `ll` untouched on the chain's path, RLE1 and the CRC
//!   over the chain as over the joined block, and the decoder's errors on that path.
//!
//! With the `trace` feature, they also check from the scratch's own buffers that `rle` is
//! never allocated, and how `ll` grows across levels.

mod common;
use common::{compress, craft, oracle, Rng, Spec};
use mibz2::bwt::{Cooperative, InverseBwt, Inverted, Plain, Segmented};
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

/// Every block of `z`, as the decoder sees it after MTF: its last column (with the capacity
/// MTF reserved), its counts and its primary index
fn last_columns(z: &[u8]) -> Vec<(Vec<u8>, [u32; 256], u32)> {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(z));
    let mut v = Vec::new();
    let mut syms = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        let n = mibz2::mtf::mtf(&symbols, &mut ll, &mut counts, k).unwrap();
        assert_eq!(n, ll.len());
        assert!((symbols.orig_ptr as usize) < n);
        v.push((ll, counts, symbols.orig_ptr));
        syms = symbols.syms;
    }
    v
}

/// The transformation itself, the algorithm held fixed: `Segmented::invert` into a buffer of
/// its own against `Segmented::invert_in_place` over `ll`, block by block. One kernel of
/// each is reused throughout, as a worker reuses its scratch. On every block in place must
/// succeed, give exactly the separate buffer's bytes, and leave `ll`'s buffer where it was
/// (same pointer and capacity: no reallocation, the success contract)
#[test]
fn segmented_in_place_equals_segmented_into_its_own_buffer() {
    let mut rng = Rng(0x87_13);
    let (mut sep, mut inp) = (Segmented::default(), Segmented::default());
    let mut out = Vec::new();
    let (mut blocks, mut short) = (0, 0);
    for (name, data) in common::samples(&mut rng) {
        for level in [1, 9] {
            for (i, (mut ll, counts, orig_ptr)) in last_columns(&compress(&data, level))
                .into_iter()
                .enumerate()
            {
                sep.invert(&ll, &counts, orig_ptr, &mut out);
                let (ptr, cap, n) = (ll.as_ptr(), ll.capacity(), ll.len());
                assert!(
                    inp.invert_in_place(&mut ll, &counts, orig_ptr),
                    "{name} -{level} block {i}: Segmented fell back"
                );
                assert!(ll == out, "{name} -{level} block {i}: the bytes differ");
                assert_eq!(ll.len(), n, "{name} -{level} block {i}: the length");
                assert_eq!(ll.as_ptr(), ptr, "{name} -{level} block {i}: ll moved");
                assert_eq!(
                    ll.capacity(),
                    cap,
                    "{name} -{level} block {i}: ll's capacity"
                );
                blocks += 1;
                short += (n < 1 << 16) as usize; // bwt::MIN_SEGMENTED: the plain walk below it
            }
        }
    }
    // both walks were exercised: the segmented one and the plain one for short blocks
    assert!(
        short > 0 && blocks > short,
        "{blocks} blocks, {short} short"
    );
}

/// The split, the algorithm held fixed: `Segmented::invert_split` against
/// `Segmented::invert` into a buffer of its own, block by block, on every sample. The
/// chain's pieces joined are the separate buffer's bytes, with `ll` untouched (bytes,
/// length, buffer); in place (a short block, or one whose chain would join a short cycle
/// round again), `invert_in_place`'s contract. RLE1 and the CRC over the chain are those
/// over the joined bytes, by both kernels. Both paths, and the plain walk, are taken.
#[test]
fn segmented_split_equals_segmented_into_its_own_buffer() {
    let mut rng = Rng(0x87_1a);
    let (mut sep, mut split) = (Segmented::default(), Segmented::default());
    let mut out = Vec::new();
    let (mut chained, mut in_place) = (0, 0);
    let mut samples = common::samples(&mut rng);
    // a string repeated: r0's cycle is short, joined round again
    samples.push(("repeated".into(), b"abcdefg".repeat(40_000)));
    for (name, data) in samples {
        for level in [1, 9] {
            for (i, (mut ll, counts, orig_ptr)) in last_columns(&compress(&data, level))
                .into_iter()
                .enumerate()
            {
                let what = format!("{name} -{level} block {i}");
                sep.invert(&ll, &counts, orig_ptr, &mut out);
                let before = ll.clone();
                let (ptr, cap, n) = (ll.as_ptr(), ll.capacity(), ll.len());
                match split.invert_split(&mut ll, &counts, orig_ptr) {
                    Some(Inverted::Chain(c)) => {
                        assert!(c.clone().flatten().eq(out.iter()), "{what}: the pieces");
                        for k in [Kernels::scalar(), Kernels::detect()] {
                            let (mut whole, mut got) = (b"x".to_vec(), b"x".to_vec());
                            let want = mibz2::rle::unrle(&out, &mut whole, k);
                            let mut crc = mibz2::crc::Running::new(1);
                            let r = mibz2::rle::unrle_chain(c.clone(), n, &mut got, &mut crc, k);
                            assert_eq!(r, want, "{what}: RLE1's result");
                            assert!(got == whole, "{what}: RLE1's bytes, {k:?}");
                            let crc = crc.finish(&got, k);
                            assert_eq!(crc, mibz2::crc::crc32(&whole[1..], k), "{what}");
                        }
                        assert!(ll == before, "{what}: ll changed on the chain's path");
                        chained += 1;
                    }
                    Some(Inverted::InPlace) => {
                        assert!(ll == out, "{what}: the bytes differ");
                        in_place += 1;
                    }
                    None => panic!("{what}: Segmented fell back"),
                }
                assert_eq!(ll.len(), n, "{what}: the length");
                assert_eq!((ll.as_ptr(), ll.capacity()), (ptr, cap), "{what}: ll moved");
            }
        }
    }
    assert!(
        chained > 0 && in_place > 0,
        "{chained} chained, {in_place} in place"
    );
}

/// The decoder's errors on the chain's path, against the separate buffer's (`Plain`):
/// crafted blocks long enough to segment whose walk ends in a run's 4 bytes with no count
/// (RunUnfinished), and blocks with a wrong stored CRC (BlockCrc, its computed CRC the
/// same), each after a good block: the same error, after the same bytes (the output
/// truncated back to the good block's end), on one thread and through the 4-worker
/// parallel reader. One scratch is kept through every error and then decodes a good
/// stream exactly. Each block was checked to take the chain.
#[test]
fn errors_on_the_chain_end_as_the_separate_buffer_path_does() {
    let mut rng = Rng(0x87_17);
    let good: Vec<u8> = rng.bytes(150_000);
    let good_z = compress(&good, 9);
    let (mut unfinished, mut bad_crc) = (0, 0);
    let mut kernel = Segmented::default();
    let mut shared = Scratch::<Segmented>::default();
    for case in 0..400 {
        if unfinished >= 3 && bad_crc >= 3 {
            break;
        }
        let n = 70_000 + rng.below(30_000) as usize;
        let alphabet = 1 + rng.below(3);
        let ll: Vec<u8> = (0..n).map(|_| (rng.below(alphabet) * 5) as u8).collect();
        let orig_ptr = rng.below(n as u64) as u32;
        let ends_unfinished = common::reference(&ll, orig_ptr).is_none();
        if !ends_unfinished && bad_crc >= 3 {
            continue;
        }
        // the chain's path, by the kernel itself
        let mut counts = [0u32; 256];
        for &c in &ll {
            counts[c as usize] += 1;
        }
        let mut l2 = ll.clone();
        if !matches!(
            kernel.invert_split(&mut l2, &counts, orig_ptr),
            Some(Inverted::Chain(_))
        ) {
            continue;
        }
        let mut spec = Spec::of(ll, orig_ptr);
        if !ends_unfinished {
            spec.crc = Some(0x1234_5678);
        }
        let mut z = good_z.clone();
        z.extend(craft(9, &[spec], None));
        let (sep, sep_err) = run(&z, &mut Scratch::<Plain>::default());
        let (got, err) = run(&z, &mut Scratch::<Segmented>::default());
        let want = if ends_unfinished {
            unfinished += 1;
            Some(ErrorKind::RunUnfinished)
        } else {
            bad_crc += 1;
            assert!(matches!(
                sep_err,
                Some(ErrorKind::BlockCrc {
                    stored: 0x1234_5678,
                    ..
                })
            ));
            sep_err.clone()
        };
        assert_eq!(sep_err, want, "case {case}: the separate buffer's error");
        assert_eq!(err, sep_err, "case {case}: the error");
        assert!(got == sep && sep == good, "case {case}: the output kept");
        // one scratch through the error, then a good stream
        let (got, err) = run(&z, &mut shared);
        assert_eq!(err, sep_err, "case {case}: the error, the scratch kept");
        assert!(
            got == good,
            "case {case}: the output kept, the scratch kept"
        );
        let (again, err) = run(&good_z, &mut shared);
        assert_eq!(err, None, "case {case}: after the error");
        assert!(again == good, "case {case}: after the error, one scratch");
        // the parallel reader: the same error after the same bytes
        let (par, par_err) = parallel::<Segmented>(&z);
        let (par_sep, par_sep_err) = parallel::<Plain>(&z);
        assert!(
            par_err.is_some(),
            "case {case}: the parallel reader decoded it"
        );
        assert_eq!(
            par_err, par_sep_err,
            "case {case}: the parallel reader's error"
        );
        assert!(
            par == par_sep,
            "case {case}: the parallel reader's bytes before it"
        );
    }
    assert!(
        unfinished >= 3 && bad_crc >= 3,
        "{unfinished} unfinished, {bad_crc} bad CRCs"
    );
}

/// The fallback contract: a kernel that doesn't invert in place returns false and changes
/// nothing (`ll`'s bytes, length, buffer and capacity), and its next `invert` is unaffected
#[test]
fn a_kernel_that_falls_back_changes_nothing() {
    let mut rng = Rng(0x87_14);
    let samples = common::samples(&mut rng);
    let text = &samples.iter().find(|(n, _)| n == "text").unwrap().1;
    let (mut plain, mut coop) = (Plain::default(), Cooperative::<2>::default());
    let (mut want, mut got) = (Vec::new(), Vec::new());
    for (mut ll, counts, orig_ptr) in last_columns(&compress(&text[..1_000_000], 9)) {
        let before = ll.clone();
        let (ptr, cap) = (ll.as_ptr(), ll.capacity());
        assert!(!plain.invert_in_place(&mut ll, &counts, orig_ptr), "Plain");
        assert!(
            !coop.invert_in_place(&mut ll, &counts, orig_ptr),
            "Cooperative"
        );
        assert!(ll == before, "ll's bytes changed on fallback");
        // and the split's default is that answer: None, nothing changed
        assert!(
            plain.invert_split(&mut ll, &counts, orig_ptr).is_none(),
            "Plain split"
        );
        assert!(
            coop.invert_split(&mut ll, &counts, orig_ptr).is_none(),
            "Coop split"
        );
        assert!(ll == before, "ll's bytes changed on the split's fallback");
        assert_eq!(
            (ll.as_ptr(), ll.capacity()),
            (ptr, cap),
            "ll's buffer changed"
        );
        // the separate-buffer call that follows gives what a fresh kernel gives
        plain.invert(&ll, &counts, orig_ptr, &mut got);
        Plain::default().invert(&ll, &counts, orig_ptr, &mut want);
        assert!(got == want, "Plain after a fallback");
    }
}

/// A scratch reused after a failed block (a wrong CRC, then a primary index past the
/// block): the next stream still decodes exactly, in place
#[test]
fn a_scratch_reused_after_an_error_decodes_exactly() {
    let mut rng = Rng(0x87_15);
    let good: Vec<u8> = rng.bytes(300_000);
    let good_z = compress(&good, 9);
    let ll: Vec<u8> = (0..100_000u32).map(|i| (i * 7 % 251) as u8).collect();
    let mut bad_crc = Spec::of(ll.clone(), 17);
    bad_crc.crc = Some(0x1234_5678);
    let mut s = Scratch::<Segmented>::default();
    for spec in [bad_crc, Spec::of(ll.clone(), ll.len() as u32)] {
        let (_, err) = run(&craft(9, &[spec], None), &mut s);
        assert!(err.is_some(), "the bad block decoded");
        let (out, err) = run(&good_z, &mut s);
        assert_eq!(err, None);
        assert!(out == good, "after an error, one scratch");
    }
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
