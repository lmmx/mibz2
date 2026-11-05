//! `mtf::mtf` on the SIMD kernel against the scalar one and against a reference written
//! here from libbzip2's loop (a `Vec` list, `remove` and `insert`): the same result, error
//! kind included, and on success the same last column and counts, on symbol streams made
//! to reach every path of the AVX2 list (each index below 32, the second level's every
//! row and row edge, far moves back to back), on random streams of several index
//! distributions, on corrupt ones (a run's weight refused, a block past its limit, at
//! every limit around its length) and on real blocks.

mod common;
use common::*;
use mibz2::block::Symbols;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, ErrorKind, Kernels};

const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;

/// libbzip2's loop as `mtf_scalar` makes its checks, with a list that is a `Vec`
fn reference(b: &Symbols) -> Result<(Vec<u8>, [u32; 256]), ErrorKind> {
    let mut list: Vec<u8> = b.seq_to_unseq.to_vec();
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let (mut run, mut weight) = (0u64, 1u64);
    for &s in &b.syms {
        if s <= 1 {
            if weight >= MAX_RUN_WEIGHT as u64 {
                return Err(ErrorKind::RunTooLong);
            }
            run += (s as u64 + 1) * weight;
            weight *= 2;
            continue;
        }
        if run != 0 {
            if ll.len() as u64 + run > b.max_len as u64 {
                return Err(ErrorKind::BlockTooLong);
            }
            counts[list[0] as usize] += run as u32;
            ll.extend(std::iter::repeat(list[0]).take(run as usize));
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        if ll.len() >= b.max_len {
            return Err(ErrorKind::BlockTooLong);
        }
        let v = list.remove(s as usize - 1);
        list.insert(0, v);
        counts[v as usize] += 1;
        ll.push(v);
    }
    Ok((ll, counts))
}

fn kernels() -> Vec<Kernels> {
    let mut v = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        v.push(Kernels::detect());
    }
    v
}

/// Every kernel against the reference on `b`; whether it was accepted
fn agree(b: &Symbols, what: &str) -> bool {
    let want = reference(b);
    for k in kernels() {
        // the outputs start dirty, as a worker's kept buffers are
        let (mut ll, mut c) = (vec![7u8; 100], [3u32; 256]);
        let r = mtf::mtf(b, &mut ll, &mut c, k);
        match &want {
            Ok((wl, wc)) => {
                assert_eq!(
                    r,
                    Ok(wl.len()),
                    "{what}, max {}, {k:?}: the result",
                    b.max_len
                );
                assert!(
                    ll == *wl,
                    "{what}, max {}, {k:?}: the last column",
                    b.max_len
                );
                assert!(c == *wc, "{what}, max {}, {k:?}: the counts", b.max_len);
            }
            Err(e) => assert_eq!(r, Err(e.clone()), "{what}, max {}, {k:?}", b.max_len),
        }
    }
    want.is_ok()
}

/// A block of `syms` over `n_in_use` bytes (a random ascending choice, as the stream's
/// bitmap gives them), its limit `max`
fn block(rng: &mut Rng, n_in_use: usize, syms: Vec<u16>, max: usize) -> Symbols {
    let mut used = [false; 256];
    let mut left = n_in_use;
    while left > 0 {
        let c = rng.below(256) as usize;
        if !used[c] {
            used[c] = true;
            left -= 1;
        }
    }
    let mut seq_to_unseq = [0u8; 256];
    for (i, c) in (0..256).filter(|&c| used[c]).enumerate() {
        seq_to_unseq[i] = c as u8;
    }
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq,
        syms,
        eob: n_in_use as u16 + 1,
        max_len: max,
    }
}

/// `b` at its own limit and at limits around its length (and its runs' ends), so that
/// each check refuses it somewhere: (accepted, refused) counts
fn at_limits(b: &Symbols, what: &str) -> (usize, usize) {
    let n = reference(b).map_or(0, |(ll, _)| ll.len());
    let mut limits = vec![b.max_len, n, n + 1, n + 63, n + 64, n + 65];
    for d in [1, 2, 31, 32, 33, 63, 64, 65, 100] {
        if n >= d {
            limits.push(n - d);
        }
    }
    limits.push(n / 2);
    limits.push(n / 3);
    let (mut ok, mut refused) = (0, 0);
    for m in limits {
        let c = Symbols {
            crc: b.crc,
            orig_ptr: b.orig_ptr,
            seq_to_unseq: b.seq_to_unseq,
            syms: b.syms.clone(),
            eob: b.eob,
            max_len: m,
        };
        if agree(&c, what) {
            ok += 1;
        } else {
            refused += 1;
        }
    }
    (ok, refused)
}

/// A run of `len` bytes as RUNA/RUNB (bijective base 2, least significant first)
fn run_syms(mut len: u64) -> Vec<u16> {
    let mut v = Vec::new();
    while len > 0 {
        if len & 1 == 1 {
            v.push(0);
            len = (len - 1) / 2;
        } else {
            v.push(1);
            len = (len - 2) / 2;
        }
    }
    v
}

#[test]
fn every_index_and_row_edge() {
    let mut rng = Rng(0x5eed_0f21_e7e1);
    let mut cases: Vec<(String, Vec<u16>, usize)> = Vec::new();
    // each index alone, repeated (a move of index nn, then the list as it leaves it)
    for nn in 1..256u16 {
        let mut v = vec![nn + 1; 300];
        v.push(257);
        cases.push((format!("index {nn} repeated"), v, 256));
    }
    // every index up and down, then the far ones back to back with near ones between
    let mut up: Vec<u16> = (2..=256).collect();
    up.extend((2..=256).rev());
    up.push(257);
    cases.push(("every index, up and down".into(), up, 256));
    // the second level's row edges (t = nn - 32 at 0, 31, 32, 63, ...), each then a near
    // index, a far one, and an index of each first-level class
    let edges: Vec<u16> = [
        32, 33, 47, 48, 63, 64, 95, 96, 127, 128, 159, 160, 191, 192, 223, 224, 254, 255,
    ]
    .to_vec();
    let mut v = Vec::new();
    for _ in 0..20 {
        for &e in &edges {
            for &after in &[1u16, 15, 16, 31, 32, 255, e] {
                v.push(e + 1);
                v.push(after + 1);
            }
        }
    }
    v.push(257);
    cases.push(("row edges".into(), v, 256));
    // far moves only, back to back, every index of the second level in random order
    let v: Vec<u16> = (0..20_000)
        .map(|_| 33 + rng.below(224) as u16)
        .chain([257])
        .collect();
    cases.push(("far moves back to back".into(), v, 256));
    // the first level's boundary: 15, 16, 31 and 32 in every order
    let v: Vec<u16> = (0..20_000)
        .map(|_| [15u16, 16, 31, 32, 1, 17][rng.below(6) as usize] + 1)
        .chain([257])
        .collect();
    cases.push(("first level's edges".into(), v, 256));
    // runs between moves, of the lengths whose writes change path (64 and 65 bytes)
    let mut v = Vec::new();
    for len in [1u64, 2, 3, 31, 32, 33, 63, 64, 65, 66, 127, 128, 1000] {
        for &nn in &[1u16, 16, 31, 32, 200] {
            v.extend(run_syms(len));
            v.push(nn + 1);
        }
    }
    v.extend(run_syms(77));
    v.push(257);
    cases.push(("runs of each write path".into(), v, 256));
    let (mut ok, mut refused) = (0, 0);
    for (name, syms, n_in_use) in cases {
        let b = block(&mut rng, n_in_use, syms, 900_000);
        let (o, r) = at_limits(&b, &name);
        (ok, refused) = (ok + o, refused + r);
    }
    assert!(ok > 0 && refused > 0, "{ok} accepted, {refused} refused");
}

#[test]
fn random_streams() {
    let mut rng = Rng(0x0dd0_ba11_c0ff_ee00);
    let (mut ok, mut refused) = (0, 0);
    for case in 0..600 {
        let n_in_use = match case % 6 {
            0 => 256,
            1 => 1 + rng.below(256) as usize,
            2 => 2 + rng.below(30) as usize,
            3 => 33 + rng.below(224) as usize,
            4 => 1,
            _ => 2,
        };
        let eob = n_in_use as u16 + 1;
        let len = rng.below(if case % 10 == 0 { 50_000 } else { 3_000 }) as usize;
        let run_share = rng.below(90);
        let dist = rng.below(4);
        let mut syms = Vec::with_capacity(len + 1);
        for _ in 0..len {
            if n_in_use < 2 || rng.below(100) < run_share {
                syms.push(rng.below(2) as u16);
                continue;
            }
            // an index of 1..n_in_use - 1: uniform, small (geometric), far, or Wikidata's
            // shape (most below 16, some to 31, a few past)
            let top = n_in_use as u64 - 1;
            let nn = match dist {
                0 => 1 + rng.below(top),
                1 => (1 + rng.below(4) * rng.below(4)).min(top),
                2 => top.min(32) + rng.below(top + 1 - top.min(32)).min(top - top.min(32)),
                _ => match rng.below(100) {
                    0..=84 => 1 + rng.below(15),
                    85..=97 => 16 + rng.below(16),
                    _ => 32 + rng.below(96),
                }
                .min(top),
            }
            .max(1);
            syms.push(nn as u16 + 1);
        }
        // most blocks end with the end-of-block symbol; some run out of symbols first
        if case % 7 != 0 {
            syms.push(eob);
        }
        let max = if case % 5 == 0 {
            rng.below(5_000) as usize
        } else {
            900_000
        };
        let b = block(&mut rng, n_in_use, syms, max);
        if agree(&b, &format!("random {case}")) {
            ok += 1;
        } else {
            refused += 1;
        }
        if case % 20 == 0 {
            let (o, r) = at_limits(&b, &format!("random {case}"));
            (ok, refused) = (ok + o, refused + r);
        }
    }
    assert!(ok > 0 && refused > 0, "{ok} accepted, {refused} refused");
}

#[test]
fn corrupt_runs_and_limits() {
    let mut rng = Rng(0x0bad_0b10_c0de);
    let mut cases: Vec<(&str, Vec<u16>, usize)> = Vec::new();
    // 21 run symbols are a weight of 2^21 still accepted; the 22nd is refused
    let mut v = vec![1u16; 21];
    v.extend([5, 257]);
    cases.push(("21 RUNB then a byte", v, 5_000_000));
    let mut v = vec![0u16; 22];
    v.extend([5, 257]);
    cases.push(("22 RUNA", v, 5_000_000));
    let mut v = vec![0u16; 21];
    v.push(1);
    cases.push(("21 RUNA, a RUNB, no end", v, 5_000_000));
    // a run past the block's end after moves on each path
    let mut v = vec![40u16, 3, 200, 17];
    v.extend(run_syms(899_996));
    v.extend([2, 257]);
    cases.push(("a run to the limit, then a byte", v, 900_000));
    let mut v = vec![40u16, 3, 200, 17];
    v.extend(run_syms(899_997));
    v.push(257);
    cases.push(("a run past the limit", v, 900_000));
    // bytes to the limit, then one more
    let v: Vec<u16> = (0..1001).map(|i| 2 + (i * 37 % 255) as u16).collect();
    cases.push(("bytes past the limit", v, 1000));
    cases.push(("no symbols", vec![], 100));
    cases.push(("the end only", vec![257], 100));
    cases.push(("a run at the end", [run_syms(500), vec![257]].concat(), 500));
    cases.push((
        "a run at the end past it",
        [run_syms(501), vec![257]].concat(),
        500,
    ));
    let (mut ok, mut refused) = (0, 0);
    for (name, syms, max) in cases {
        let b = block(&mut rng, 256, syms, max);
        if agree(&b, name) {
            ok += 1;
        } else {
            refused += 1;
        }
        let (o, r) = at_limits(&b, name);
        (ok, refused) = (ok + o, refused + r);
    }
    assert!(ok > 0 && refused > 0, "{ok} accepted, {refused} refused");
}

#[test]
fn real_blocks() {
    let mut rng = Rng(0x1234_5678_9abc_def1);
    let mut blocks = 0;
    for (name, data) in samples(&mut rng) {
        for level in [1, 9] {
            let z = compress(&data, level);
            let mut sc = Scanner::new(SliceSource(&z));
            while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
                at_limits(&symbols, &format!("{name}, level {level}"));
                blocks += 1;
            }
        }
    }
    assert!(blocks > 10);
}
