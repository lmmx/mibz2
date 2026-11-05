//! `mtf::mtf` on the detected kernel (AVX2, whose list's first 32 bytes are registers)
//! against `mtf::mtf_scalar`, on symbol streams no encoder need write: every index
//! distribution (all below 16, all in 16..32, all at 32 or more, the boundaries of the
//! registers and of the 16-byte chunks behind them, uniform, mixed), with runs of every
//! length, alphabets of every size, and limits and run weights that make `mtf` refuse
//! the block (`BlockTooLong`, `RunTooLong`) at every kind of step. The same result, error
//! kind, last column (on refusal too) and counts.

mod common;
use common::Rng;
use mibz2::block::Symbols;
use mibz2::mtf;
use mibz2::{ErrorKind, Kernels};

/// The two kernels on `b`: `mtf`'s result
fn agree(b: &Symbols, what: &str) -> Result<usize, ErrorKind> {
    let (mut ll0, mut c0) = (Vec::new(), [0u32; 256]);
    let r0 = mtf::mtf_scalar(b, &mut ll0, &mut c0);
    // the outputs start dirty, as a worker's kept buffers are
    let (mut ll, mut c) = (vec![7u8; 100], [3u32; 256]);
    let r = mtf::mtf(b, &mut ll, &mut c, Kernels::detect());
    assert_eq!(r, r0, "{what}: the result");
    assert!(ll == ll0, "{what}: the last column");
    if r0.is_ok() {
        assert!(c == c0, "{what}: the counts");
    }
    r0
}

/// An index for `n_in_use` bytes (1..n_in_use) by shape
fn index(rng: &mut Rng, shape: usize, n_in_use: usize) -> usize {
    let top = n_in_use - 1; // the largest index
    let pick = |rng: &mut Rng, lo: usize, hi: usize| {
        let (lo, hi) = (lo.min(top), hi.min(top));
        lo + rng.below((hi - lo + 1) as u64) as usize
    };
    match shape {
        // all small: the first register only
        0 => pick(rng, 1, 15),
        // the second register only
        1 => pick(rng, 16, 31),
        // all past the registers, in memory
        2 => pick(rng, 32, 255),
        // the registers' and the chunks' boundaries
        3 => {
            let edges = [
                1, 14, 15, 16, 17, 30, 31, 32, 33, 47, 48, 63, 64, 128, 239, 240, 255,
            ];
            edges[rng.below(edges.len() as u64) as usize].min(top)
        }
        // uniform over the alphabet
        4 => pick(rng, 1, 255),
        // as compressed text: mostly small, a tail of large
        5 => match rng.below(100) {
            0..=84 => pick(rng, 1, 15),
            85..=97 => pick(rng, 16, 31),
            _ => pick(rng, 32, 255),
        },
        // alternating classes, each symbol: small, second register, memory
        _ => match rng.below(3) {
            0 => pick(rng, 1, 15),
            1 => pick(rng, 16, 31),
            _ => pick(rng, 32, 255),
        },
    }
}

/// A run's RUNA/RUNB symbols (1..=2^k - 1 + ...), `len` of them
fn run_syms(rng: &mut Rng, len: usize, syms: &mut Vec<u16>) {
    for _ in 0..len {
        syms.push(rng.below(2) as u16);
    }
}

/// A random block: `n_in_use` bytes in use, `count` steps of shape `shape`, a run before a
/// step with probability `run_pct`%, of 1..=`run_max` symbols; ended by its end-of-block
/// symbol if `ended`
fn block(
    rng: &mut Rng,
    n_in_use: usize,
    shape: usize,
    count: usize,
    run_pct: u64,
    run_max: usize,
    ended: bool,
) -> Symbols {
    let eob = (n_in_use + 1) as u16;
    let mut s2u = [0u8; 256];
    // any bytes, repeats included: the kernels must permute whatever the table holds
    for x in s2u.iter_mut() {
        *x = rng.next() as u8;
    }
    if rng.below(2) == 0 {
        // or, as a stream's, the bytes in use in order
        let mut used: Vec<u8> = (0..=255u8).collect();
        for i in (1..256).rev() {
            used.swap(i, rng.below(i as u64 + 1) as usize);
        }
        let mut u = used[..n_in_use].to_vec();
        u.sort();
        s2u[..n_in_use].copy_from_slice(&u);
    }
    let mut syms = Vec::with_capacity(count * 2 + 1);
    for _ in 0..count {
        if rng.below(100) < run_pct {
            let len = 1 + rng.below(run_max as u64) as usize;
            run_syms(rng, len, &mut syms);
        }
        if n_in_use == 1 {
            // one byte in use: no index, so at most one run, or its symbols would join
            break;
        }
        syms.push(index(rng, shape, n_in_use) as u16 + 1);
    }
    if n_in_use > 1 && rng.below(4) == 0 {
        // a run just before the end
        let len = 1 + rng.below(run_max as u64) as usize;
        run_syms(rng, len, &mut syms);
    }
    if ended {
        syms.push(eob);
        // symbols past the end-of-block one are never read
        syms.extend((0..rng.below(4)).map(|_| rng.below(eob as u64) as u16));
    }
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq: s2u,
        syms,
        eob,
        // room for a few of the longest runs (2^22 - 2 bytes each)
        max_len: 16 << 20,
    }
}

/// `b`'s length on the scalar kernel, at no limit
fn length(b: &Symbols) -> usize {
    mtf::mtf_scalar(b, &mut Vec::new(), &mut [0; 256]).unwrap()
}

/// `b` at the limit `max`
fn at(b: &Symbols, max: usize) -> Symbols {
    Symbols {
        crc: b.crc,
        orig_ptr: b.orig_ptr,
        seq_to_unseq: b.seq_to_unseq,
        syms: b.syms.clone(),
        eob: b.eob,
        max_len: max,
    }
}

#[test]
fn every_index_distribution() {
    let mut rng = Rng(0x5eed_f00d_d00d_1234);
    let mut seen = [0usize; 3];
    for shape in 0..7 {
        for &n_in_use in &[1, 2, 3, 16, 17, 32, 33, 48, 64, 100, 200, 255, 256] {
            for (run_pct, run_max) in [(0, 1), (30, 6), (60, 12)] {
                for ended in [true, false] {
                    let b = block(&mut rng, n_in_use, shape, 3000, run_pct, run_max, ended);
                    for &s in &b.syms {
                        if s >= 2 && s < b.eob {
                            seen[((s - 1) as usize / 16).min(2)] += 1;
                        }
                    }
                    let n = length(&b);
                    let what = format!("shape {shape}, {n_in_use} in use, runs {run_pct}%");
                    // its own length (the limit met exactly) and a little more: accepted
                    agree(&at(&b, n), &what).unwrap();
                    agree(&at(&b, n + 63), &what).unwrap();
                    // and cut short: refused as the scalar kernel refuses it
                    for m in [0, 1, n / 3, n.saturating_sub(64), n.saturating_sub(1)] {
                        if m < n {
                            assert_eq!(
                                agree(&at(&b, m), &what),
                                Err(ErrorKind::BlockTooLong),
                                "{what}, max {m}"
                            );
                        }
                    }
                }
            }
        }
    }
    // each of the three paths taken many times
    assert!(seen.iter().all(|&k| k > 10_000), "{seen:?}");
}

/// Runs of every length, from one symbol to the weight's limit: the short runs' two
/// stores, the long ones' fill, each at the limit and near it
#[test]
fn runs_of_every_length() {
    let mut rng = Rng(0x0dd_ba11_cafe_beef);
    for len in 1..=22 {
        for shape in [0, 2, 5] {
            let mut b = block(&mut rng, 40, shape, 200, 50, 6, true);
            // a run of `len` symbols (RUNA/RUNB) at the start, middle and end, an index
            // either side of it so that no other run's symbols join it
            let mut r = Vec::new();
            run_syms(&mut rng, len, &mut r);
            let end = b.syms.iter().position(|&s| s == b.eob).unwrap();
            b.syms
                .splice(end..end, [4].into_iter().chain(r.iter().copied()));
            let mid = end / 2;
            b.syms.splice(
                mid..mid,
                [2].into_iter().chain(r.iter().copied()).chain([3]),
            );
            b.syms.splice(0..0, r.iter().copied().chain([2]));
            let what = format!("a run of {len} symbols, shape {shape}");
            if len == 22 {
                // the 22nd RUNA/RUNB symbol of a run would weigh 2^21: refused
                assert_eq!(agree(&b, &what), Err(ErrorKind::RunTooLong), "{what}");
                continue;
            }
            let n = length(&b);
            for m in [n, n + 1, n + 64, n - 1, n - 64.min(n), n / 2] {
                let r = agree(&at(&b, m), &format!("{what}, max {m}"));
                assert_eq!(r.is_ok(), m >= n, "{what}, max {m}");
            }
        }
    }
    // the longest run there is, 21 RUNBs (2^22 - 2 bytes): accepted
    let mut b = block(&mut rng, 40, 0, 10, 0, 1, true);
    b.syms
        .splice(0..0, std::iter::repeat_n(1u16, 21).chain([5]));
    let n = agree(&b, "21 RUNBs").unwrap();
    assert!(n >= (1 << 22) - 2);
    // and 22, anywhere: refused
    for shape in [0, 1, 2] {
        let mut b = block(&mut rng, 60, shape, 100, 20, 4, true);
        let mid = b.syms.len() / 2;
        b.syms.splice(mid..mid, std::iter::repeat_n(1u16, 22));
        assert_eq!(agree(&b, "22 RUNBs"), Err(ErrorKind::RunTooLong));
    }
}

/// The list's every byte moved through each place: long streams of indices at each
/// boundary in turn, the front read back by a run after every step
#[test]
fn every_index_then_a_run() {
    let mut rng = Rng(0x1357_9bdf_2468_ace0);
    for n_in_use in [16, 17, 31, 32, 33, 47, 48, 49, 64, 65, 128, 256] {
        let mut b = block(&mut rng, n_in_use, 0, 0, 0, 1, false);
        for round in 0..3 {
            for i in 1..n_in_use {
                // index i, then a run of the new front of round + 1 bytes
                b.syms.push(i as u16 + 1);
                b.syms.extend(std::iter::repeat_n(0u16, round));
            }
            for i in (1..n_in_use).rev() {
                b.syms.push(i as u16 + 1);
                b.syms.push(1);
            }
        }
        b.syms.push(b.eob);
        let what = format!("every index, {n_in_use} in use");
        let n = agree(&b, &what).unwrap();
        for m in [n - 1, n - 2, n / 2] {
            assert_eq!(agree(&at(&b, m), &what), Err(ErrorKind::BlockTooLong));
        }
    }
}

/// The symbols as bzip2 writes them for real data, through the scanner
#[test]
fn compressed_streams() {
    use common::{compress, samples};
    use mibz2::stream::{Event, Scanner, SliceSource};
    let mut rng = Rng(0xfeed_face_0123_4567);
    for (name, data) in samples(&mut rng) {
        for level in [1, 9] {
            let z = compress(&data, level);
            let mut sc = Scanner::new(SliceSource(&z));
            while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
                let what = format!("{name}, level {level}");
                let n = agree(&symbols, &what).unwrap();
                if n > 0 {
                    assert_eq!(
                        agree(&at(&symbols, n - 1), &what),
                        Err(ErrorKind::BlockTooLong)
                    );
                }
            }
        }
    }
}
