//! `mtf::mtf_checkpoints` against `mtf::mtf`, on the SIMD and the scalar kernels: the
//! same result, error kind, last column and counts on every block, with each block's limit
//! also cut to its length less one, to its last whole checkpoint (the block's end and a
//! checkpoint the same limit) and raised to its next; and checkpoints that are the counts
//! of every `CHECKPOINT` bytes of `mtf`'s last column.

mod common;
use common::*;
use mibz2::block::Symbols;
use mibz2::mtf::{self, CHECKPOINT};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;

fn kernels() -> Vec<Kernels> {
    let mut v = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        v.push(Kernels::detect());
    }
    v
}

fn blocks(z: &[u8]) -> Vec<Symbols> {
    let mut sc = Scanner::new(SliceSource(z));
    let mut v = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        v.push(symbols);
    }
    v
}

/// `b` with its limit set to `max`
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

/// Both functions on `b` at each kernel: whether `mtf` accepted it
fn agree(b: &Symbols, what: &str) -> bool {
    let mut ok = false;
    for k in kernels() {
        let (mut ll0, mut c0) = (Vec::new(), [0u32; 256]);
        let r0 = mtf::mtf(b, &mut ll0, &mut c0, k);
        // the outputs start dirty, as a worker's kept buffers are
        let (mut ll, mut c, mut cps) = (vec![7u8; 5], [3u32; 256], vec![[1u32; 256]; 2]);
        let r = mtf::mtf_checkpoints(b, &mut ll, &mut c, &mut cps, k);
        assert_eq!(r, r0, "{what}, max {}, {k:?}: the result", b.max_len);
        if r0.is_ok() {
            assert!(
                ll == ll0,
                "{what}, max {}, {k:?}: the last column",
                b.max_len
            );
            assert!(c == c0, "{what}, max {}, {k:?}: the counts", b.max_len);
            let mut cum = [0u32; 256];
            let mut want = vec![cum];
            for chunk in ll0.chunks_exact(CHECKPOINT) {
                chunk.iter().for_each(|&x| cum[x as usize] += 1);
                want.push(cum);
            }
            assert!(
                cps == want,
                "{what}, max {}, {k:?}: the checkpoints",
                b.max_len
            );
            ok = true;
        }
    }
    ok
}

/// Each block of the stream `z`, at its own limit and the cut ones
fn check(z: &[u8], what: &str) -> (usize, usize) {
    let (mut runs, mut refused) = (0, 0);
    for b in blocks(z) {
        let mut ll = Vec::new();
        let n = mtf::mtf(&b, &mut ll, &mut [0; 256], Kernels::scalar()).unwrap();
        let mut limits = vec![b.max_len, n, n.div_ceil(CHECKPOINT) * CHECKPOINT];
        if n > 0 {
            limits.push(n - 1);
        }
        if n > CHECKPOINT && n % CHECKPOINT != 0 {
            limits.push(n / CHECKPOINT * CHECKPOINT);
        }
        for m in limits {
            runs += 1;
            refused += !agree(&limited(&b, m), what) as usize;
        }
    }
    (runs, refused)
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
    // the cut limits must reach `mtf`'s refusal as well as its success
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

/// Last columns no encoder need write, from the block writer: runs that cross one or more
/// checkpoints and end exactly on a later one, start on one, or end the block on one (the
/// split run's own checkpoint and the block's last one)
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
        (
            "a run across 1 checkpoint, ending on the next",
            run(5, 2 * cp, b"b"),
        ),
        (
            "a run across 2 checkpoints, ending on the next",
            run(5, 3 * cp, b"bc"),
        ),
        ("a run from a checkpoint to the next", run(cp, 2 * cp, b"b")),
        (
            "a run across 1 checkpoint, ending the block on the next",
            run(5, 2 * cp, b""),
        ),
        (
            "a run ending a byte past a checkpoint",
            run(5, 2 * cp + 1, b"b"),
        ),
        (
            "a run ending a byte short of a checkpoint",
            run(5, 2 * cp - 1, b"b"),
        ),
    ];
    for (name, ll) in cases {
        let z = craft(9, &[Spec::of(ll, 0)], None);
        let (runs, refused) = check(&z, name);
        assert!(
            runs > 0 && refused < runs,
            "{name}: {refused} of {runs} refused"
        );
    }
}
