//! The decoder against libbzip2: valid streams of every level and shape byte for byte, on
//! the SIMD and the scalar kernels, sequential and parallel; streams no encoder writes
//! (from a block writer); and malformed streams, which must fail as libbzip2's do.

mod common;
use common::*;
use mibz2::{ErrorKind, Kernels};

fn kernels() -> Vec<Kernels> {
    let mut v = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        v.push(Kernels::detect());
    }
    v
}

fn kind(r: Result<Vec<u8>, mibz2::Error>) -> ErrorKind {
    r.expect_err("an error").kind
}

#[test]
fn every_level_and_shape() {
    let mut rng = Rng(0x1234_5678_9abc_def1);
    for (name, data) in samples(&mut rng) {
        for level in [1, 2, 5, 9] {
            let z = compress(&data, level);
            assert_eq!(oracle(&z).unwrap(), data);
            for k in kernels() {
                let got = ours(&z, k)
                    .unwrap()
                    .unwrap_or_else(|e| panic!("{name} level {level} {k:?}: {e}"));
                assert!(got == data, "{name} level {level} {k:?}: differs");
            }
            let got = parallel(&z, 3, 4, 100_003, 65_536, Kernels::detect()).unwrap();
            assert!(got == data, "{name} level {level}: parallel differs");
        }
    }
}

#[test]
fn parallel_any_chunking() {
    let mut rng = Rng(77);
    let data = samples(&mut rng).swap_remove(3).1; // text
    let z = compress(&data, 9);
    let mut multi = z.clone();
    multi.extend_from_slice(&compress(b"second stream\n", 1));
    multi.extend_from_slice(&compress(&data[..500_000], 3));
    let mut want = data.clone();
    want.extend_from_slice(b"second stream\n");
    want.extend_from_slice(&data[..500_000]);
    for (threads, slots, chunk, read) in [
        (1, 1, 1 << 20, 1 << 20),
        (4, 8, 7, 3),
        (2, 2, 4096, 1),
        (8, 16, 1 << 23, 16 << 20),
        (4, 4, 65_537, 999),
    ] {
        if chunk < 100 && read < 10 {
            // (a byte at a time both ways is slow: a prefix is enough)
            let small = compress(&data[..50_000], 1);
            assert_eq!(
                parallel(&small, threads, slots, chunk, read, Kernels::detect()).unwrap(),
                &data[..50_000]
            );
            continue;
        }
        let got = parallel(&multi, threads, slots, chunk, read, Kernels::detect()).unwrap();
        assert!(
            got == want,
            "threads {threads} slots {slots} chunk {chunk} read {read}"
        );
    }
}

#[test]
fn multi_stream() {
    let a = compress(b"first\n", 9);
    let b = compress(b"", 1); // a stream with no blocks
    let c = compress(&vec![b'z'; 2_000_000], 2);
    let all = [a.clone(), b.clone(), c.clone(), a.clone()].concat();
    let want = [
        b"first\n".to_vec(),
        vec![b'z'; 2_000_000],
        b"first\n".to_vec(),
    ]
    .concat();
    assert_eq!(oracle(&all).unwrap(), want);
    for k in kernels() {
        assert_eq!(ours(&all, k).unwrap().unwrap(), want);
    }
    assert_eq!(
        parallel(&all, 2, 2, 1000, 4096, Kernels::detect()).unwrap(),
        want
    );
    assert_eq!(ours(&b, Kernels::scalar()).unwrap().unwrap(), b"");
}

#[test]
fn crafted_blocks_match_libbzip2() {
    // last columns no encoder writes: random permutations (lists of several cycles too),
    // runs ending a block, any primary index; their CRCs right, so libbzip2 decodes them
    let mut rng = Rng(99);
    let mut unfinished = 0;
    for case in 0..300 {
        let n = match case % 6 {
            0 => 1 + rng.below(10) as usize,
            1 => 1 + rng.below(1000) as usize,
            2 => 65_536 + rng.below(100_000) as usize,
            3 => 899_000 + rng.below(1000) as usize,
            _ => 1 + rng.below(300_000) as usize,
        };
        let alphabet = 1 + rng.below(if case % 2 == 0 { 3 } else { 256 }) as u64;
        let mut ll: Vec<u8> = (0..n).map(|_| (rng.below(alphabet) * 7) as u8).collect();
        if case % 5 == 0 {
            // periodic: a string repeated, whose list is several cycles
            let p = 1 + rng.below(5) as usize;
            for i in p..n {
                ll[i] = ll[i % p];
            }
        }
        let orig_ptr = rng.below(n as u64) as u32;
        let blocks = [Spec::of(ll.clone(), orig_ptr)];
        let z = craft(9, &blocks, None);
        let Some(want) = reference(&ll, orig_ptr) else {
            // (it ends inside a run: refused by both)
            assert!(oracle(&z).is_err(), "case {case}: oracle");
            for k in kernels() {
                assert_eq!(
                    kind(ours(&z, k).unwrap()),
                    ErrorKind::RunUnfinished,
                    "case {case}"
                );
            }
            unfinished += 1;
            continue;
        };
        assert_eq!(
            oracle(&z).as_ref().map(Vec::len),
            Ok(want.len()),
            "case {case}: oracle"
        );
        assert!(
            oracle(&z).unwrap() == want,
            "case {case}: oracle differs from the reference"
        );
        for k in kernels() {
            let got = ours(&z, k)
                .unwrap()
                .unwrap_or_else(|e| panic!("case {case} n {n} {k:?}: {e}"));
            assert!(got == want, "case {case} n {n} {k:?}");
        }
        // and the plain walk, the kernel the segmented one must equal
        assert!(
            mibz2::decode_with::<mibz2::bwt::Plain>(&z, Kernels::scalar()).unwrap() == want,
            "case {case}: plain"
        );
    }
    assert!(unfinished > 0 && unfinished < 100, "{unfinished}");
}

#[test]
fn block_size_levels() {
    // every level 1-9: a block of exactly 100,000 x level bytes decodes; one byte more is
    // refused by both
    for level in 1..=9u8 {
        let max = 100_000 * level as usize;
        let ll: Vec<u8> = (0..max).map(|i| (i % 251) as u8).collect();
        let z = craft(level, &[Spec::of(ll.clone(), 3)], None);
        assert!(oracle(&z).is_ok());
        assert_eq!(
            ours(&z, Kernels::detect()).unwrap().unwrap(),
            reference(&ll, 3).unwrap()
        );
        let mut over = ll.clone();
        over.push(1);
        let z = craft(level, &[Spec::of(over, 3)], None);
        assert!(oracle(&z).is_err());
        for k in kernels() {
            assert_eq!(
                kind(ours(&z, k).unwrap()),
                ErrorKind::BlockTooLong,
                "level {level}"
            );
        }
        // and a real stream's header lowered below its blocks' size
        let real = compress(
            &vec![7u8; max + 50_000]
                .iter()
                .enumerate()
                .map(|(i, _)| (i * 31 % 253) as u8)
                .collect::<Vec<_>>(),
            9,
        );
        let mut low = real.clone();
        low[3] = b'0' + level;
        if level < 9 {
            assert!(oracle(&low).is_err());
            assert_eq!(
                kind(ours(&low, Kernels::detect()).unwrap()),
                ErrorKind::BlockTooLong
            );
        }
    }
}

#[test]
fn oversized_runs() {
    // RUNA x 21: a run's weight reaches 2^21, refused before it can overflow (libbzip2's
    // check); RUNB x 20 then RUNA: under it, but longer than the block may be
    let mut s = Spec::of(vec![b'a'; 10], 0);
    s.in_use = Some(vec![b'a']);
    s.syms = Some([vec![0u16; 22], vec![2]].concat());
    let z = craft(9, &[s], Some(0));
    assert!(oracle(&z).is_err());
    for k in kernels() {
        assert_eq!(kind(ours(&z, k).unwrap()), ErrorKind::RunTooLong);
    }
    let mut s = Spec::of(vec![b'a'; 10], 0);
    s.in_use = Some(vec![b'a']);
    s.syms = Some([vec![1u16; 20], vec![2]].concat());
    let z = craft(9, &[s], Some(0));
    assert!(oracle(&z).is_err());
    for k in kernels() {
        assert_eq!(kind(ours(&z, k).unwrap()), ErrorKind::BlockTooLong);
    }
    // the most a block holds as one run: 900,000 bytes of one value
    let mut s = Spec::of(vec![b'q'; 900_000], 5);
    s.in_use = Some(vec![b'q']);
    let z = craft(9, &[s], None);
    let want = reference(&vec![b'q'; 900_000], 5).unwrap();
    assert_eq!(oracle(&z).unwrap(), want);
    assert_eq!(ours(&z, Kernels::detect()).unwrap().unwrap(), want);
}

#[test]
fn bad_header_fields() {
    let ll = b"banana_bandana".to_vec();
    let ok = craft(9, &[Spec::of(ll.clone(), 4)], None);
    assert!(oracle(&ok).is_ok());
    // a primary index at the block's length, and past the level's bound
    for (orig, want) in [
        (14, ErrorKind::BadOrigPtr),
        (900_011, ErrorKind::BadOrigPtr),
    ] {
        let mut s = Spec::of(ll.clone(), orig);
        s.crc = Some(0);
        let z = craft(9, &[s], Some(0));
        assert!(oracle(&z).is_err());
        assert_eq!(kind(ours(&z, Kernels::detect()).unwrap()), want);
    }
    // an empty block (EOB first)
    let mut s = Spec::of(vec![], 0);
    s.in_use = Some(vec![b'a']);
    s.syms = Some(vec![2]);
    let z = craft(9, &[s], Some(0));
    assert!(oracle(&z).is_err());
    assert_eq!(
        kind(ours(&z, Kernels::detect()).unwrap()),
        ErrorKind::BadOrigPtr
    );
    // no symbols in use
    let mut s = Spec::of(vec![], 0);
    s.in_use = Some(vec![]);
    s.syms = Some(vec![1]);
    let z = craft(9, &[s], Some(0));
    assert!(oracle(&z).is_err());
    assert_eq!(
        kind(ours(&z, Kernels::detect()).unwrap()),
        ErrorKind::NoSymbols
    );
    // symbols outrunning the selectors: no EOB within the selectors' groups
    let mut s = Spec::of(vec![b'a'; 3], 0);
    s.in_use = Some(vec![b'a', b'b']);
    s.syms = Some(vec![2; 50]);
    s.crc = Some(0);
    let z = craft(9, &[s], Some(0));
    assert!(oracle(&z).is_err());
    assert_eq!(
        kind(ours(&z, Kernels::detect()).unwrap()),
        ErrorKind::SelectorOverrun
    );
}

#[test]
fn randomised_blocks_refused() {
    let ll = b"randomised".to_vec();
    let mut s = Spec::of(ll, 2);
    s.randomised = true;
    let z = craft(9, &[s], None);
    for k in kernels() {
        assert_eq!(kind(ours(&z, k).unwrap()), ErrorKind::Randomised);
    }
    assert_eq!(
        parallel(&z, 2, 2, 7, 100, Kernels::detect())
            .unwrap_err()
            .kind,
        ErrorKind::Randomised
    );
}

#[test]
fn crc_errors() {
    let mut rng = Rng(5);
    let data = samples(&mut rng).swap_remove(3).1;
    let z = compress(&data, 9);
    let starts = block_starts(&z);
    assert!(starts.len() >= 3);
    // a block's stored CRC, a bit flipped: that block fails, after every block before it
    for &b in &starts {
        let mut bad = z.clone();
        let bit = b + 48 + 5;
        bad[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
        assert!(oracle(&bad).is_err());
        for k in kernels() {
            assert!(matches!(
                kind(ours(&bad, k).unwrap()),
                ErrorKind::BlockCrc { .. }
            ));
        }
        let e = parallel(&bad, 3, 3, 1 << 16, 1 << 16, Kernels::detect()).unwrap_err();
        assert!(matches!(e.kind, ErrorKind::BlockCrc { .. }));
        assert_eq!(e.bit, Some(b));
    }
    // the stream's stored CRC: the bits just before the padding
    let mut bad = z.clone();
    let n = bad.len();
    // (the combined CRC is the last 32 bits before the final byte's padding)
    let end_bits = n as u64 * 8;
    let mut found = false;
    for pad in 0..8 {
        let crc_end = end_bits - pad;
        let mut t = z.clone();
        let bit = crc_end - 1;
        t[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
        if let Err(e) = mibz2::decode(&t) {
            if matches!(e.kind, ErrorKind::StreamCrc { .. }) {
                assert!(oracle(&t).is_err());
                found = true;
                bad = t;
                break;
            }
        }
    }
    assert!(found, "a stream CRC flip");
    let e = parallel(&bad, 2, 2, 1 << 16, 1 << 16, Kernels::detect()).unwrap_err();
    assert!(matches!(e.kind, ErrorKind::StreamCrc { .. }));
}

#[test]
fn truncation_and_trailing_bytes() {
    let mut rng = Rng(11);
    let data = samples(&mut rng).swap_remove(3).1;
    let z = compress(&data[..400_000], 1);
    for cut in (0..z.len()).step_by(997).chain(z.len() - 12..z.len()) {
        assert!(oracle(&z[..cut]).is_err());
        let e = ours(&z[..cut], Kernels::detect()).unwrap().unwrap_err();
        assert!(
            matches!(e.kind, ErrorKind::Truncated | ErrorKind::Empty),
            "cut {cut}: {e}"
        );
        if cut % 5 == 0 {
            let e = parallel(&z[..cut], 2, 2, 1 << 12, 1 << 14, Kernels::detect()).unwrap_err();
            assert!(
                matches!(e.kind, ErrorKind::Truncated | ErrorKind::Empty),
                "cut {cut}: {e}"
            );
        }
    }
    for tail in [
        &b"x"[..],
        b"\0",
        b"\0\0\0\0",
        b"BZ",
        b"BZh",
        b"BZh0",
        b"BZhx",
        b"garbage",
    ] {
        let t = [z.clone(), tail.to_vec()].concat();
        assert_eq!(
            kind(ours(&t, Kernels::detect()).unwrap()),
            ErrorKind::TrailingData,
            "{tail:?}"
        );
    }
    // a stream header and no more: truncated
    let t = [z.clone(), b"BZh9".to_vec()].concat();
    assert_eq!(
        kind(ours(&t, Kernels::detect()).unwrap()),
        ErrorKind::Truncated
    );
    assert_eq!(
        kind(ours(b"", Kernels::detect()).unwrap()),
        ErrorKind::Empty
    );
    assert_eq!(
        kind(ours(b"PK\x03\x04", Kernels::detect()).unwrap()),
        ErrorKind::NotBz2
    );
    assert_eq!(
        kind(ours(b"BZh0", Kernels::detect()).unwrap()),
        ErrorKind::NotBz2
    );
}

#[test]
fn corrupted_tables_and_selectors() {
    // every bit of the first block's header and code tables flipped in turn: libbzip2 and
    // this decoder both fail, or both give the same bytes
    let mut rng = Rng(3);
    let data = samples(&mut rng).swap_remove(3).1;
    let z = compress(&data[..200_000], 9);
    let start = block_starts(&z)[0];
    let (mut both_err, mut both_ok) = (0, 0);
    for bit in start..start + 3000 {
        let mut t = z.clone();
        t[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
        let a = oracle(&t);
        let b = ours(&t, Kernels::detect()).unwrap();
        match (&a, &b) {
            (Ok(x), Ok(y)) => {
                assert!(x == y, "bit {bit}: different bytes");
                both_ok += 1
            }
            (Err(_), Err(_)) => both_err += 1,
            (Ok(_), Err(e)) => panic!("bit {bit}: libbzip2 decodes it, this decoder fails: {e}"),
            (Err(e), Ok(_)) => panic!("bit {bit}: libbzip2 fails ({e}), this decoder decodes it"),
        }
    }
    assert!(both_err > 2900, "{both_err} errors, {both_ok} decoded");
}

#[test]
fn block_ranges() {
    // every range of whole blocks, decoded alone (the API the sweep reader could use),
    // gives exactly its part of the stream; a range ending inside a block fails
    let mut rng = Rng(8);
    let data = samples(&mut rng).swap_remove(3).1;
    let z = compress(&data, 1);
    let starts = block_starts(&z);
    let mut lens = Vec::new();
    for (i, &b) in starts.iter().enumerate() {
        let mut out = Vec::new();
        let end = starts.get(i + 1).copied().unwrap_or(u64::MAX);
        let r = mibz2::decode_blocks(&z, b, end, 1, Kernels::detect(), &mut out).unwrap();
        lens.push(out.len());
        assert_eq!(r.blocks.len(), 1);
        if i + 1 == starts.len() {
            assert!(r.stream_end);
        }
    }
    let offs: Vec<usize> = std::iter::once(0)
        .chain(lens.iter().scan(0, |s, &l| {
            *s += l;
            Some(*s)
        }))
        .collect();
    for i in 0..starts.len() {
        for j in i + 1..=starts.len() {
            let end = starts.get(j).copied().unwrap_or(u64::MAX);
            let mut out = Vec::new();
            let r =
                mibz2::decode_blocks(&z, starts[i], end, 1, Kernels::scalar(), &mut out).unwrap();
            assert!(out == data[offs[i]..offs[j]], "blocks {i}..{j}");
            if j == starts.len() {
                assert!(r.stream_end);
                if i == 0 {
                    assert_eq!(
                        mibz2::combine(r.blocks.iter().map(|b| b.2)),
                        r.stored_combined
                    );
                }
            }
        }
    }
    let mut out = Vec::new();
    let e = mibz2::decode_blocks(&z, starts[0], starts[1] - 1, 1, Kernels::detect(), &mut out)
        .unwrap_err();
    assert_eq!(e.kind, ErrorKind::RangeOvershoot);
}

#[test]
fn reader_stops_early() {
    // the reader cancelling mid-stream: the writer's next write fails, nothing hangs
    let mut rng = Rng(21);
    let data = samples(&mut rng).swap_remove(3).1;
    let z = compress(&data, 9);
    let dec = mibz2::StreamDecoder::new(4, 4, Kernels::detect());
    std::thread::scope(|s| {
        let w = s.spawn(|| {
            for c in z.chunks(1000) {
                if dec.write(c.to_vec()).is_err() {
                    return false;
                }
            }
            dec.close();
            true
        });
        let mut buf = vec![0u8; 100_000];
        assert_eq!(dec.read(&mut buf).unwrap(), 100_000);
        assert_eq!(&buf[..], &data[..100_000]);
        dec.cancel();
        let _ = w.join().unwrap();
        assert_eq!(dec.read(&mut buf).unwrap_err().kind, ErrorKind::Cancelled);
    });
    dec.join();
}
