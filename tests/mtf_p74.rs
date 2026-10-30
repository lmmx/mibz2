//! The research P74 kernels (`mtf::p74`, P74 Gate 1) against the scalar ones, both arms:
//! `p74::mtf` against `mtf::mtf_scalar` and `p74::mtf_checkpoints` against
//! `mtf::mtf_checkpoints_scalar`, the same result, error kind, last column, counts and
//! checkpoints, on tests/mtf_checkpoints.rs's edge cases at every limit, the crate's
//! samples, runs on checkpoints, and symbol streams written directly: every alphabet size
//! from 1 to 256, each rank path alone and their crossings (15/16, 31/32), the tail's
//! deepest ranks, long runs and refused ones.

mod common;
use common::*;
use mibz2::block::Symbols;
use mibz2::mtf::{self, p74::Arm, CHECKPOINT};
use mibz2::stream::{Event, Scanner, SliceSource};

const ARMS: [Arm; 2] = [Arm::Branch, Arm::Free];

fn blocks(z: &[u8]) -> Vec<Symbols> {
    let mut sc = Scanner::new(SliceSource(z));
    let mut v = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        v.push(symbols);
    }
    v
}

fn limited(b: &Symbols, max: usize) -> Symbols {
    Symbols {
        crc: b.crc,
        orig_ptr: b.orig_ptr,
        seq_to_unseq: b.seq_to_unseq,
        syms: b.syms.clone(),
        eob: b.eob,
        max_len: max,
    }
}

/// Both arms and both kernels on `b`: whether the scalar kernel accepted it
fn agree(b: &Symbols, what: &str) -> bool {
    let (mut ll0, mut c0) = (Vec::new(), [0u32; 256]);
    let r0 = mtf::mtf_scalar(b, &mut ll0, &mut c0);
    let (mut lc0, mut cc0, mut cps0) = (Vec::new(), [0u32; 256], Vec::new());
    let rc0 = mtf::mtf_checkpoints_scalar(b, &mut lc0, &mut cc0, &mut cps0);
    for arm in ARMS {
        // the outputs start dirty, as a worker's kept buffers are
        let (mut ll, mut c) = (vec![7u8; 5], [3u32; 256]);
        let r = mtf::p74::mtf(b, &mut ll, &mut c, arm);
        assert_eq!(r, r0, "{what}, max {}, {arm:?}: the result", b.max_len);
        if r0.is_ok() {
            assert!(
                ll == ll0,
                "{what}, max {}, {arm:?}: the last column",
                b.max_len
            );
            assert!(c == c0, "{what}, max {}, {arm:?}: the counts", b.max_len);
        }
        let (mut ll, mut c, mut cps) = (vec![7u8; 5], [3u32; 256], vec![[1u32; 256]; 2]);
        let r = mtf::p74::mtf_checkpoints(b, &mut ll, &mut c, &mut cps, arm);
        assert_eq!(
            r, rc0,
            "{what}, max {}, {arm:?}: checkpoints' result",
            b.max_len
        );
        if rc0.is_ok() {
            assert!(
                ll == lc0,
                "{what}, max {}, {arm:?}: checkpoints' last column",
                b.max_len
            );
            assert!(
                c == cc0,
                "{what}, max {}, {arm:?}: checkpoints' counts",
                b.max_len
            );
            assert!(
                cps == cps0,
                "{what}, max {}, {arm:?}: the checkpoints",
                b.max_len
            );
        }
    }
    r0.is_ok()
}

/// `b` at its own limit and cut ones (tests/mtf_checkpoints.rs's)
fn at_limits(b: &Symbols, what: &str) -> (usize, usize) {
    let (mut runs, mut refused) = (0, 0);
    let mut ll = Vec::new();
    let Ok(n) = mtf::mtf_scalar(b, &mut ll, &mut [0; 256]) else {
        return (1, !agree(b, what) as usize);
    };
    let mut limits = vec![b.max_len, n, n.div_ceil(CHECKPOINT) * CHECKPOINT];
    if n > 0 {
        limits.push(n - 1);
    }
    if n > CHECKPOINT && n % CHECKPOINT != 0 {
        limits.push(n / CHECKPOINT * CHECKPOINT);
    }
    for m in limits {
        runs += 1;
        refused += !agree(&limited(b, m), what) as usize;
    }
    (runs, refused)
}

fn check(z: &[u8], what: &str) -> (usize, usize) {
    let (mut runs, mut refused) = (0, 0);
    for b in blocks(z) {
        let (r, f) = at_limits(&b, what);
        (runs, refused) = (runs + r, refused + f);
    }
    (runs, refused)
}

#[test]
fn avx2_is_what_is_tested() {
    // the arms fall back to the scalar kernels without AVX2, which would test nothing
    #[cfg(target_arch = "x86_64")]
    if !std::is_x86_feature_detected!("avx2") {
        eprintln!("no AVX2: the P74 tests compare the scalar kernels with themselves");
    }
}

#[test]
fn edge_cases_at_every_limit() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let cp = CHECKPOINT;
    let mut zeros_then = vec![0u8; 2 * cp];
    zeros_then.push(7);
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("one byte", b"x".to_vec()),
        ("two bytes", b"ab".to_vec()),
        ("a run of 1 MB", vec![0u8; 1 << 20]),
        ("2 checkpoints of zeros, then a byte", zeros_then),
        ("a run ending on a checkpoint", vec![9u8; cp]),
        ("period 3", b"abc".repeat(300_000)),
        ("random, 3 checkpoints", rng.bytes(3 * cp)),
        ("random, 3 checkpoints less 1", rng.bytes(3 * cp - 1)),
        ("random, 3 checkpoints and 1", rng.bytes(3 * cp + 1)),
        ("random, 1 MB", rng.bytes(1 << 20)),
        (
            "runs of 1-300",
            (0..4000u32)
                .flat_map(|i| {
                    std::iter::repeat((i % 7) as u8 + b'a').take(1 + (i * 37 % 300) as usize)
                })
                .collect(),
        ),
    ];
    let (mut runs, mut refused) = (0, 0);
    for (name, data) in &cases {
        for level in [1, 9] {
            let (r, f) = check(&compress(data, level), &format!("{name}, level {level}"));
            (runs, refused) = (runs + r, refused + f);
        }
    }
    assert!(refused > 0 && refused < runs, "{refused} of {runs} refused");
}

#[test]
fn every_shape() {
    let mut rng = Rng(0x1234_5678_9abc_def1);
    for (name, data) in samples(&mut rng) {
        for level in [1, 9] {
            check(&compress(&data, level), &format!("{name}, level {level}"));
        }
    }
}

#[test]
fn every_corpus() {
    for (name, data) in corpora(1) {
        check(&compress(&data, 9), name);
    }
}

#[test]
fn runs_on_checkpoints() {
    let cp = CHECKPOINT;
    let run = |from: usize, to: usize, tail: &[u8]| {
        let mut ll = vec![b'x'; from];
        ll.resize(to, b'a');
        ll.extend_from_slice(tail);
        ll
    };
    let cases = [
        run(5, 2 * cp, b"b"),
        run(5, 3 * cp, b"bc"),
        run(cp, 2 * cp, b"b"),
        run(5, 2 * cp, b""),
        run(5, 2 * cp + 1, b"b"),
        run(5, 2 * cp - 1, b"b"),
        run(5, cp + 32, b"b"),
        run(5, cp + 33, b"b"),
        run(cp - 32, cp + 1, b"b"),
    ];
    for (i, ll) in cases.into_iter().enumerate() {
        let z = craft(9, &[Spec::of(ll, 0)], None);
        let (runs, refused) = check(&z, &format!("case {i}"));
        assert!(
            runs > 0 && refused < runs,
            "case {i}: {refused} of {runs} refused"
        );
    }
}

/// A run of `len` (at least 1) as RUNA/RUNB symbols: bijective base 2, least digit first
fn run_symbols(mut len: u32, out: &mut Vec<u16>) {
    while len > 0 {
        if len & 1 == 1 {
            out.push(0);
            len = (len - 1) / 2;
        } else {
            out.push(1);
            len = (len - 2) / 2;
        }
    }
}

/// A block of `alpha` symbols in use, its symbols written directly: each explicit symbol's
/// rank from `rank` (taken modulo the alphabet's ranks, 1..alpha), a run of
/// 1..=`runs` before each with probability 1/3 (none if `runs` is 0), then the end
fn written(
    rng: &mut Rng,
    alpha: usize,
    tokens: usize,
    runs: u32,
    rank: &mut dyn FnMut(&mut Rng) -> usize,
) -> Symbols {
    let eob = alpha as u16 + 1;
    let mut syms = Vec::new();
    for _ in 0..tokens {
        if runs > 0 && rng.below(3) == 0 {
            run_symbols(1 + rng.below(runs as u64) as u32, &mut syms);
        }
        if alpha > 1 {
            let q = 1 + rank(rng) % (alpha - 1);
            syms.push(q as u16 + 1);
        }
    }
    if runs > 0 {
        run_symbols(1 + rng.below(runs as u64) as u32, &mut syms);
    }
    syms.push(eob);
    // the bytes in use, any `alpha` of them in order
    let mut used: Vec<u8> = (0..=255u8).collect();
    for i in (1..256).rev() {
        used.swap(i, rng.below(i as u64 + 1) as usize);
    }
    let mut used = used[..alpha].to_vec();
    used.sort_unstable();
    let mut seq_to_unseq = [0u8; 256];
    seq_to_unseq[..alpha].copy_from_slice(&used);
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq,
        syms,
        eob,
        max_len: 900_000,
    }
}

#[test]
fn every_alphabet_size() {
    let mut rng = Rng(0x7474_7474_0000_0001);
    for alpha in 1..=256usize {
        let b = written(&mut rng, alpha, 3000, 40, &mut |r| r.below(255) as usize);
        let (runs, _) = at_limits(&b, &format!("alphabet {alpha}"));
        assert!(runs > 0);
    }
}

#[test]
fn each_rank_path_and_crossing() {
    let mut rng = Rng(0x7474_7474_0000_0002);
    let ranges: [(&str, usize, usize); 7] = [
        ("ranks 1-15", 1, 15),
        ("ranks 16-31", 16, 31),
        ("ranks 32-255", 32, 255),
        ("ranks 14-17", 14, 17),
        ("ranks 30-33", 30, 33),
        ("ranks 250-255", 250, 255),
        ("ranks 1-255", 1, 255),
    ];
    for (name, lo, hi) in ranges {
        for runs in [0, 3, 40] {
            let b = written(&mut rng, 256, 20_000, runs, &mut |r| {
                lo - 1 + r.below((hi - lo + 1) as u64) as usize
            });
            at_limits(&b, &format!("{name}, runs up to {runs}"));
        }
    }
}

#[test]
fn long_and_refused_runs() {
    let mut rng = Rng(0x7474_7474_0000_0003);
    // runs past a checkpoint, and runs whose weight libbzip2 refuses
    for runs in [33, 100_000, 3 * CHECKPOINT as u32] {
        let b = written(&mut rng, 90, 50, runs, &mut |r| r.below(89) as usize);
        at_limits(&b, &format!("runs up to {runs}"));
    }
    let mut b = written(&mut rng, 4, 10, 0, &mut |r| r.below(3) as usize);
    let too_long: Vec<u16> = std::iter::repeat(1u16).take(22).collect();
    b.syms.splice(0..0, too_long);
    assert!(!agree(&b, "a run's weight past 2^21"));
}

/// A block of `alpha` symbols in use whose explicit symbols are `low` of rank 1 and
/// `high` of rank 32 (or the alphabet's last rank if it is smaller), with no runs
fn ranked(alpha: usize, low: usize, high: usize) -> Symbols {
    let eob = alpha as u16 + 1;
    let deep = 32.min(alpha - 1) as u16;
    let mut syms = vec![2u16; low];
    syms.extend(std::iter::repeat(deep + 1).take(high));
    syms.push(eob);
    let mut seq_to_unseq = [0u8; 256];
    for (i, x) in seq_to_unseq.iter_mut().enumerate() {
        *x = i as u8;
    }
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq,
        syms,
        eob,
        max_len: 900_000,
    }
}

/// The Gate 1 selector's frozen rule: P74 if A <= 32, or no explicit symbol, or 10F <= E
#[test]
fn the_selector_rule() {
    use mtf::p74::select;
    // the alphabet guard: no rank of 32 or more, whatever the ranks
    assert!(select(&ranked(32, 0, 100)));
    assert!(select(&ranked(2, 5, 5)));
    // the exact tenth: 10F <= E, with E = low + high
    assert!(select(&ranked(100, 90, 10)), "10 of 100 high: chosen");
    assert!(!select(&ranked(100, 89, 11)), "11 of 100 high: not chosen");
    assert!(select(&ranked(100, 900, 100)));
    assert!(!select(&ranked(100, 899, 101)));
    // no explicit symbol: no fallback, so P74
    assert!(select(&ranked(100, 0, 0)));
    // the rule holds RUNA/RUNB and the end out of the count
    let mut b = ranked(100, 90, 10);
    b.syms.splice(0..0, [0u16, 1, 0, 1, 1]);
    assert!(select(&b));
    // random data: ranks uniform over 1..256, about 88% of them 32 or more
    let mut rng = Rng(0x7474_7474_0000_0004);
    let b = written(&mut rng, 256, 5000, 3, &mut |r| r.below(255) as usize);
    assert!(!select(&b));
    agree(&b, "random, the selector's block");
}
