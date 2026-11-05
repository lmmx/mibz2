//! `mtf::mtf`'s AVX2 kernel against its scalar twin, on symbol streams made directly
//! rather than by an encoder, so that they reach what no encoder writes: the same result,
//! error kind, last column and counts on every stream, and the checkpointed kernel's
//! result the same too. Random streams of every mix of runs and byte symbols, each at
//! limits around its length; then the edges of the AVX2 kernel's window loop: runs of 21
//! symbols (the most allowed) and 22 (refused) at every offset from a window's start, runs
//! across the block's limit, the end-of-block symbol early, late or missing, indices at
//! the top of the list and past the alphabet in use.

mod common;
use common::Rng;
use mibz2::block::Symbols;
use mibz2::mtf;
use mibz2::{ErrorKind, Kernels};

const RUNA: u16 = 0;
const RUNB: u16 = 1;

/// A block of `used` bytes in use (spread over 0-255, increasing as a header gives
/// them), its end-of-block symbol `eob`, its limit `max`
fn block(used: usize, eob: u16, syms: Vec<u16>, max: usize) -> Symbols {
    let mut seq_to_unseq = [0u8; 256];
    for (i, x) in seq_to_unseq.iter_mut().take(used).enumerate() {
        *x = (i * 256 / used.max(1)) as u8;
    }
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq,
        syms,
        eob,
        max_len: max,
    }
}

/// Both kernels (and the checkpointed one) on `b`: the scalar result
fn agree(b: &Symbols, what: &str) -> Result<usize, ErrorKind> {
    let (mut ll0, mut c0) = (Vec::new(), [0u32; 256]);
    let r0 = mtf::mtf(b, &mut ll0, &mut c0, Kernels::scalar());
    let k = Kernels::detect();
    if k.avx2 {
        // the outputs start dirty, as a worker's kept buffers are
        let (mut ll, mut c) = (vec![7u8; 100], [3u32; 256]);
        let r = mtf::mtf(b, &mut ll, &mut c, k);
        assert_eq!(r, r0, "{what}, max {}: the result", b.max_len);
        if r0.is_ok() {
            assert!(ll == ll0, "{what}, max {}: the last column", b.max_len);
            assert!(c == c0, "{what}, max {}: the counts", b.max_len);
        }
        let (mut ll, mut c, mut cps) = (vec![7u8; 5], [3u32; 256], Vec::new());
        let r = mtf::mtf_checkpoints(b, &mut ll, &mut c, &mut cps, k);
        assert_eq!(r, r0, "{what}, max {}: the checkpointed result", b.max_len);
        if r0.is_ok() {
            assert!(
                ll == ll0,
                "{what}, max {}: the checkpointed column",
                b.max_len
            );
        }
    }
    r0
}

/// `syms` at a generous limit, then at limits around its length (and the length a run
/// would end at if it overflows): the results seen
fn at_limits(used: usize, eob: u16, syms: &[u16], what: &str) -> Vec<Result<usize, ErrorKind>> {
    let big = 1 << 23;
    let r = agree(&block(used, eob, syms.to_vec(), big), what);
    let mut seen = vec![r.clone()];
    let Ok(len) = r else { return seen };
    let mut limits = vec![
        len,
        len + 1,
        len + 63,
        len + 64,
        len + 65,
        len / 2,
        0,
        1,
        64,
    ];
    for d in [1, 2, 63, 64, 65, 66, 128, 1000] {
        if len >= d {
            limits.push(len - d);
        }
    }
    for m in limits {
        seen.push(agree(&block(used, eob, syms.to_vec(), m), what));
    }
    seen
}

/// A run of `len` run symbols, its digits from `bits` (bit k set: the kth is RUNB; RUNA
/// past the 64th)
fn run(len: usize, bits: u64) -> Vec<u16> {
    (0..len)
        .map(|k| bits.checked_shr(k as u32).unwrap_or(0) as u16 & 1)
        .collect()
}

/// Random byte symbols and runs: a byte symbol's index mostly small, now and then past
/// 32 (the copy); a run's digits random, at most `max_run` of them
fn random_stream(rng: &mut Rng, used: usize, n: usize, p_run: u64, max_run: u64) -> Vec<u16> {
    let mut v = Vec::with_capacity(n + 32);
    while v.len() < n {
        if used < 2 || rng.below(100) < p_run {
            let len = 1 + rng.below(max_run) as usize;
            let len = if rng.below(4) == 0 { len } else { len.min(4) };
            v.extend(run(len, rng.next()));
        }
        if used >= 2 {
            let top = if rng.below(20) == 0 {
                used
            } else {
                used.min(32)
            };
            v.push(2 + rng.below(top as u64 - 1) as u16);
        }
    }
    v
}

#[test]
fn random_streams() {
    let mut rng = Rng(0x5eed_0fa1_15ca_1a00);
    let mut refused = 0;
    for &used in &[1usize, 2, 3, 17, 40, 255, 256] {
        for &p_run in &[0u64, 20, 50, 80, 100] {
            for &n in &[0usize, 1, 63, 64, 65, 200, 5000] {
                let mut s = random_stream(&mut rng, used, n, p_run, 21);
                s.push(used as u16 + 1);
                let what = format!("used {used}, runs {p_run}%, {n} symbols");
                refused += at_limits(used, used as u16 + 1, &s, &what)
                    .iter()
                    .filter(|r| r.is_err())
                    .count();
            }
        }
    }
    assert!(refused > 0);
}

#[test]
fn long_random_blocks() {
    // whole blocks' worth, runs as long as allowed (some past 64 bytes, a fill)
    let mut rng = Rng(0x0123_4567_89ab_cdef);
    for (used, p_run, max_run) in [(60, 45, 8), (200, 60, 12), (256, 30, 21), (5, 70, 6)] {
        let mut s = random_stream(&mut rng, used, 100_000, p_run, max_run);
        s.push(used as u16 + 1);
        at_limits(used, used as u16 + 1, &s, &format!("block of {used}"));
    }
}

#[test]
fn runs_of_21_and_22_at_every_offset() {
    let mut rng = Rng(0xfeed_beef_dead_cafe);
    let used = 40;
    let eob = used as u16 + 1;
    for offset in 0..140 {
        for len in [20, 21, 22, 23, 64, 65] {
            // (a run of 20 or 21 digits is 1-4 MB: two of them at each offset)
            let bitss = if len <= 21 {
                vec![!0, rng.next()]
            } else {
                vec![0, !0, rng.next()]
            };
            for bits in bitss {
                // `offset` symbols of a random stream (whose own runs are short), the run,
                // then bytes (and short runs) past more than a window
                let mut s = random_stream(&mut rng, used, offset, 30, 3);
                s.truncate(offset);
                s.extend(run(len, bits));
                s.extend(random_stream(&mut rng, used, 150, 30, 3));
                s.push(eob);
                let what = format!("a run of {len} ({bits:x}) after {offset} symbols");
                let rs = at_limits(used, eob, &s, &what);
                if len >= 22 {
                    // refused, unless a byte symbol cut in before the 22nd (the stream
                    // before it may end in run symbols, which join it)
                    assert!(rs.iter().all(|r| r.is_err()), "{what}: {rs:?}");
                }
            }
        }
    }
}

#[test]
fn runs_across_the_limit() {
    // a run ending exactly at the limit, one past it, and one past it by a byte symbol
    // after; runs long enough to fill (past 64 bytes), each after a stretch of bytes
    let used = 9;
    let eob = used as u16 + 1;
    let mut rng = Rng(42);
    for lead in [0usize, 1, 30, 64, 65, 100, 1000] {
        for bits in [0u64, 1, 0x2a, 0x3ff, 0x1fffff, 0x15555] {
            for digits in [1usize, 5, 7, 10, 21] {
                let mut s = random_stream(&mut rng, used, lead, 10, 2);
                s.truncate(lead);
                s.extend(run(digits, bits));
                for tail in [
                    vec![eob],
                    vec![3, eob],
                    vec![2, RUNA, eob],
                    vec![5, 6, 7, eob],
                ] {
                    let t = [s.clone(), tail].concat();
                    at_limits(
                        used,
                        eob,
                        &t,
                        &format!("{digits} digits {bits:x} after {lead}"),
                    );
                }
            }
        }
    }
}

#[test]
fn the_end_of_block_early_late_or_missing() {
    let mut rng = Rng(7);
    let used = 30;
    let eob = used as u16 + 1;
    let body = random_stream(&mut rng, used, 300, 50, 5);
    for at in 0..body.len().min(260) {
        // the end-of-block symbol at `at`, then symbols it must stop before: run symbols,
        // bytes and symbols past the alphabet
        let mut s = body[..at].to_vec();
        s.push(eob);
        s.extend([RUNB, RUNA, 3, 200, 256, RUNB].iter().cycle().take(100));
        at_limits(used, eob, &s, &format!("the end at {at}"));
        // and no end at all: the stream stops at `at` (a run open at its end unwritten)
        at_limits(used, eob, &body[..at], &format!("no end, {at} symbols"));
    }
    // only runs then the end, only the end, nothing
    for digits in [1, 5, 21, 22] {
        let s = [run(digits, 0x155555), vec![eob]].concat();
        at_limits(used, eob, &s, &format!("{digits} run digits alone"));
    }
    at_limits(used, eob, &[eob], "the end alone");
    at_limits(used, eob, &[], "no symbols");
}

#[test]
fn indices_at_the_top_and_past_the_alphabet() {
    let mut rng = Rng(99);
    // every index of a full list, often, with runs between
    let mut s = Vec::new();
    for i in 0..20_000u32 {
        if rng.below(3) == 0 {
            s.extend(run(1 + rng.below(4) as usize, rng.next()));
        }
        s.push(if i % 3 == 0 {
            2 + rng.below(255) as u16
        } else {
            256 - (i % 50) as u16
        });
    }
    s.push(257);
    at_limits(256, 257, &s, "the top of a full list");
    // indices past the bytes in use (a list's tail no header gives), below and above the
    // end-of-block symbol: the kernels must still agree
    for (used, eob) in [(10, 200u16), (10, 11), (1, 2), (100, 256)] {
        let mut s = Vec::new();
        for _ in 0..5000 {
            if rng.below(2) == 0 {
                s.extend(run(1 + rng.below(3) as usize, rng.next()));
            }
            s.push(2 + rng.below(255) as u16);
        }
        s.retain(|&x| x != eob);
        s.push(eob);
        at_limits(
            used,
            eob,
            &s,
            &format!("{used} in use, end {eob}, indices to 255"),
        );
    }
}

#[test]
fn every_limit_near_the_end_of_a_block() {
    // a block whose last 200 bytes cross the window loop's 64-byte margin, cut at each
    let mut rng = Rng(1234);
    let used = 50;
    let eob = used as u16 + 1;
    let mut s = random_stream(&mut rng, used, 3000, 40, 6);
    s.push(eob);
    let len = agree(&block(used, eob, s.clone(), 1 << 23), "a block").unwrap();
    for m in len.saturating_sub(200)..=len + 70 {
        let _ = agree(
            &block(used, eob, s.clone(), m),
            &format!("limit {m} of {len}"),
        );
    }
}
