//! B5 evidence for the run-fed research (the `run-fed` feature; Guardian D044 on #269):
//! `mtf::mtf_runs` (scalar and AVX2) → `run_stage::sort` → `run_stage::fill` (both
//! kernels) and `fill_rows`, and `run_stage::build` with its `takes_runs` fallback, each
//! against the production `mtf::mtf` → `bwt::build` table on the same kernel, word for
//! word, with the same errors (and the same panics, for symbols outside the scanner's
//! contract). Nothing here changes the code under test.
//!
//! - `miri_*`: the emitter's record stores and every exit path, small enough for Miri.
//!   With `MIBZ2_EXPECT_AVX2` set they assert the AVX2 kernel was among those run
//!   (`miri.yml`'s AVX2 step), and every test counts the cases each kernel ran.
//! - `fuzz_*`: seeded, reproducible streams (run-heavy, alternating, single-symbol,
//!   max-alphabet, mixed, corrupt), each case classified by the path it took (accepted,
//!   fallback, refused, error, panic) and every path asserted reached. `MIBZ2_B5_FUZZ_SECS`
//!   runs further seeds for that many seconds after the fixed ones.
//! - `representation_limit`: blocks of 2^24 - 5 rows (taken) and over 2^24 (refused by
//!   `sort` when fed directly, and sent to the column's path by `build`).
#![cfg(feature = "run-fed")]

use mibz2::block::Symbols;
use mibz2::bwt::{build, TVector};
use mibz2::run_stage::{self, Scratch};
use mibz2::{mtf, ErrorKind, Kernels};
use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Once;

const MAX_RUN_WEIGHT: u64 = 2 * 1024 * 1024;

fn kernels() -> Vec<Kernels> {
    let mut v = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        v.push(Kernels::detect());
    }
    if std::env::var_os("MIBZ2_EXPECT_AVX2").is_some() {
        assert!(Kernels::detect().avx2, "AVX2 expected, and not detected");
        assert!(v.iter().any(|k| k.avx2), "AVX2 expected among the kernels");
    }
    v
}

thread_local!(static QUIET: Cell<bool> = const { Cell::new(false) });

/// `f` with its panic caught, and its message kept off stderr (an expected panic: the
/// scalar kernels index past their list on a symbol above 256)
fn caught<T>(f: impl FnOnce() -> T) -> std::thread::Result<T> {
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !QUIET.with(|q| q.get()) {
                prev(info)
            }
        }));
    });
    QUIET.with(|q| q.set(true));
    let r = catch_unwind(AssertUnwindSafe(f));
    QUIET.with(|q| q.set(false));
    r
}

/// The paths the cases took, a kernel run each
#[derive(Default, Debug, Clone, Copy)]
struct Paths {
    /// kernel runs, and those on AVX2
    runs: u64,
    avx2: u64,
    /// `build` took the records' path, and its table was the production one
    accepted: u64,
    /// `takes_runs` false: `build` took the column's path; the records still sorted and
    /// filled directly against the production table where they tile
    fallback: u64,
    /// `takes_runs` true, `mtf_runs` succeeded, `sort` refused its records: `build`'s
    /// fallback after its `debug_assert` (a panic in a debug build, the column's path in
    /// a release one)
    refused: u64,
    /// the same error from `mtf_runs`, `mtf` and `build`; by kind
    run_too_long: u64,
    block_too_long: u64,
    other_error: u64,
    /// both `mtf` and `mtf_runs` panicked (symbols outside the scanner's contract)
    panicked: u64,
    /// accepted by `sort` although not the column's maximal runs (still the right table)
    non_maximal: u64,
    /// the records' direct sort → fill compared, all shares included
    direct_fills: u64,
    /// the AVX2 fill reached through `build` or directly
    avx2_fills: u64,
}

impl Paths {
    fn add(&mut self, o: &Paths) {
        self.runs += o.runs;
        self.avx2 += o.avx2;
        self.accepted += o.accepted;
        self.fallback += o.fallback;
        self.refused += o.refused;
        self.run_too_long += o.run_too_long;
        self.block_too_long += o.block_too_long;
        self.other_error += o.other_error;
        self.panicked += o.panicked;
        self.non_maximal += o.non_maximal;
        self.direct_fills += o.direct_fills;
        self.avx2_fills += o.avx2_fills;
    }
}

/// The column's maximal runs as `mtf_runs` records them, closed by its length
fn runs_of(ll: &[u8]) -> Vec<u32> {
    let mut v = Vec::new();
    for (i, &x) in ll.iter().enumerate() {
        if i == 0 || x != ll[i - 1] {
            v.push((i as u32) << 8 | x as u32);
        }
    }
    v.push((ll.len() as u32) << 8);
    v
}

/// `b` on every kernel: production against the run-fed stage, every comparison asserted
fn case(b: &Symbols, what: &str, p: &mut Paths) {
    for k in kernels() {
        one(b, k, what, p);
    }
}

fn one(b: &Symbols, k: Kernels, what: &str, p: &mut Paths) {
    p.runs += 1;
    p.avx2 += k.avx2 as u64;
    let what = format!("{what}, max {}, {k:?}", b.max_len);
    // production: the column and its table (buffers dirty, as a worker's kept ones are)
    let want = caught(|| {
        let (mut ll, mut c) = (vec![7u8; 33], [3u32; 256]);
        mtf::mtf(b, &mut ll, &mut c, k).map(|n| {
            let mut tt = TVector::new();
            build(&ll, &c, &mut tt);
            (n, ll, c, tt.as_slice().to_vec())
        })
    });
    // the emitter alone, into a dirty buffer
    let got = caught(|| {
        let (mut recs, mut c) = (vec![9u32; 5], [1u32; 256]);
        let r = mtf::mtf_runs(b, &mut recs, &mut c, k);
        (r, recs, c)
    });
    // the whole stage
    let staged = caught(|| {
        let (mut recs, mut s, mut ll, mut c) =
            (vec![5u32; 3], Scratch::default(), vec![1u8; 9], [2u32; 256]);
        let mut tt = TVector::new();
        let r = run_stage::build(b, &mut recs, &mut s, &mut ll, &mut c, &mut tt, k);
        (r, tt.as_slice().to_vec())
    });
    let (want, got) = match (want, got) {
        (Err(_), Err(_)) => {
            assert!(staged.is_err(), "{what}: the stage didn't panic as mtf did");
            p.panicked += 1;
            return;
        }
        (Ok(w), Ok(g)) => (w, g),
        (w, g) => panic!(
            "{what}: mtf panicked {}, mtf_runs panicked {}",
            w.is_err(),
            g.is_err()
        ),
    };
    let (r, recs, rc) = got;
    // the bound the raw stores rely on, on every exit
    assert!(
        recs.len() <= b.syms.len() + 1,
        "{what}: {} records for {} symbols",
        recs.len(),
        b.syms.len()
    );
    let (n, ll, wc, wt) = match want {
        Err(e) => {
            assert_eq!(r, Err(e.clone()), "{what}: mtf_runs' error");
            let (sr, _) = staged.expect("the stage panicked on an error");
            assert_eq!(sr.map(|x| x.0), Err(e.clone()), "{what}: the stage's error");
            match e {
                ErrorKind::RunTooLong => p.run_too_long += 1,
                ErrorKind::BlockTooLong => p.block_too_long += 1,
                _ => p.other_error += 1,
            }
            return;
        }
        Ok(w) => w,
    };
    assert_eq!(r, Ok(n), "{what}: mtf_runs' result");
    assert!(rc == wc, "{what}: mtf_runs' counts");
    let maximal = recs == runs_of(&ll);
    // the records, sorted and filled directly, every kernel and 1-3 shares
    let mut s = Scratch::default();
    let sorted = run_stage::sort(&recs, &rc, n, &mut s);
    if let Some(t) = &sorted {
        assert_eq!(t.n(), n, "{what}");
        if !maximal {
            p.non_maximal += 1;
        }
        for fk in kernels() {
            let mut tt = TVector::new();
            run_stage::fill(t, &mut tt, fk);
            assert!(
                tt.as_slice() == wt,
                "{what}: the records' table, fill {fk:?}"
            );
            p.avx2_fills += fk.avx2 as u64;
        }
        for shares in 1..=3u32 {
            let mut out = Vec::new();
            for j in 0..shares {
                let (q0, q1) = (n as u32 * j / shares, n as u32 * (j + 1) / shares);
                run_stage::fill_rows(t, q0, q1, &mut out);
            }
            assert!(out == wt, "{what}: {shares} shares");
        }
        p.direct_fills += 1;
    } else {
        // the column's maximal runs always tile: only malformed records are refused, or
        // a block past the records' 24 bits (which `takes_runs` never sends here)
        assert!(
            !maximal || n >= 1 << 24,
            "{what}: sort refused the column's own runs"
        );
        // and the one malformation found: a leading run that never closes (no end of
        // block), so its record opens a run of no rows at row 0
        assert!(
            n >= 1 << 24 || (n == 0 && recs.len() == 2 && recs[1] == 0),
            "{what}: refused records {:?}",
            &recs[..recs.len().min(8)]
        );
    }
    let taken = run_stage::takes_runs(b);
    match staged {
        Err(_) => {
            // only `build`'s debug_assert on refused records may stop it
            assert!(
                cfg!(debug_assertions) && taken && sorted.is_none(),
                "{what}: the stage panicked"
            );
            p.refused += 1;
        }
        Ok((sr, st)) => {
            let (sn, path) = sr.unwrap_or_else(|e| panic!("{what}: the stage's error {e:?}"));
            assert_eq!(sn, n, "{what}: the stage's length");
            assert!(st == wt, "{what}: the stage's table");
            assert_eq!(path, taken && sorted.is_some(), "{what}: the stage's path");
            if path {
                p.accepted += 1;
                p.avx2_fills += k.avx2 as u64;
            } else if taken {
                p.refused += 1;
            } else {
                p.fallback += 1;
            }
        }
    }
}

fn symbols(n_in_use: usize, syms: Vec<u16>, max_len: usize) -> Symbols {
    let mut seq_to_unseq = [0u8; 256];
    // a fixed scatter of the bytes in use (ascending, as the stream's bitmap gives them)
    let mut c = 0usize;
    for (i, e) in seq_to_unseq.iter_mut().enumerate().take(n_in_use) {
        c = if n_in_use == 256 {
            i
        } else {
            (c + 1).max(i * 256 / n_in_use.max(1))
        };
        *e = c as u8;
    }
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq,
        syms,
        eob: n_in_use as u16 + 1,
        max_len,
    }
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

/// The longest run a weight allows: every symbol RUNB, 21 of them (weight 2^20 last)
fn max_run() -> (Vec<u16>, u64) {
    let v = vec![1u16; 21];
    (v, 2 * (MAX_RUN_WEIGHT - 1))
}

// ---- Miri: the emitter's record stores and every exit ------------------------------

/// The emitter's exits, both kernels, each asserted: the normal end, a leading run, the
/// empty input, the end-of-block alone, each error kind at each of its checks, the limit
/// exactly met and one short, no end-of-block (after a literal, and after a leading run:
/// the records `sort` refuses), the clamp of an index above the list (AVX2) and the
/// scalar panic there, and the longest run a weight allows, at its limit and past it
#[test]
fn miri_emitter_exits() {
    let mut p = Paths::default();
    let eob = 4u16; // three bytes in use
    let e = |syms: &[u16], max: usize| symbols(3, syms.to_vec(), max);
    let cases: Vec<(&str, Symbols)> = vec![
        ("the empty input", e(&[], 100)),
        ("the end alone", e(&[eob], 100)),
        ("a literal, the end", e(&[2, eob], 100)),
        ("a leading run, the end", e(&[0, 1, eob], 100)),
        (
            "a leading run, literals, runs",
            e(&[1, 2, 0, 3, 3, 1, 1, 2, eob], 100),
        ),
        ("literals alternating", e(&[2, 2, 2, 3, 2, 3, eob], 100)),
        ("symbols after the end", e(&[2, 0, eob, 3, 3, 0], 100)),
        ("no end, a run last", e(&[2, 3, 0, 1], 100)),
        ("no end, a literal last", e(&[0, 2, 3], 100)),
        ("no end, a leading run alone", e(&[0, 1], 100)),
        ("no end, a leading run, then the end missing", e(&[1], 1)),
        ("at the limit, a literal last", e(&[0, 2, 3, eob], 4)),
        ("past the limit at a literal", e(&[0, 2, 3, eob], 3)),
        ("at the limit, a run last", e(&[2, 0, 1, eob], 6)),
        ("past the limit at a run", e(&[2, 0, 1, eob], 5)),
        ("a leading run past the limit", e(&[1, 1, eob], 5)),
        ("a limit of 0, the end alone", e(&[eob], 0)),
        ("a limit of 0, a literal", e(&[2, eob], 0)),
        ("a weight too heavy", {
            let mut v = vec![2u16];
            v.extend([0u16; 22]);
            v.push(eob);
            e(&v, 100)
        }),
        ("a leading weight too heavy", {
            let mut v = vec![1u16; 22];
            v.push(eob);
            e(&v, 100)
        }),
        (
            "a symbol above the end, inside the list",
            e(&[200, 3, 256, eob], 100),
        ),
        ("an end of 0 (never met)", {
            let mut b = e(&[2, 3, 0, 2], 100);
            b.eob = 0;
            b
        }),
        ("an end of 1 (a run, never met)", {
            let mut b = e(&[2, 1, 3, 1], 100);
            b.eob = 1;
            b
        }),
        // (scalar: an index past the list, a panic, in mtf as in mtf_runs; AVX2: clamped)
        ("a symbol past the list", e(&[2, 300, 3, eob], 100)),
        ("u16::MAX", e(&[0, u16::MAX, eob], 100)),
        // (a map with a byte twice: a literal can then name the open run's byte, and its
        // records are no longer maximal runs; still a tiling, and the same table)
        ("a byte twice in the map", {
            let mut b = e(&[2, 2, 0, 3, 2, 2, eob], 100);
            b.seq_to_unseq[..3].copy_from_slice(&[9, 9, 4]);
            b
        }),
    ];
    for (what, b) in &cases {
        case(b, what, &mut p);
    }
    // the second level's every row (all 256 bytes in use), each index up then down
    let mut v: Vec<u16> = (2..=256).collect();
    v.extend((2..=256).rev());
    v.extend(run_syms(70));
    v.extend([33, 2, 64, 65, 96, 128, 160, 192, 224, 256, 257]);
    case(&symbols(256, v, 2000), "max alphabet, every index", &mut p);
    // the representation's ends without a table: the longest run, alone and after a
    // literal, at its limit and one past, checked against the column it stands for
    let (run, len) = max_run();
    for k in kernels() {
        for lead in [false, true] {
            let mut syms = if lead { vec![] } else { vec![3u16] };
            syms.extend(&run);
            syms.push(2);
            syms.push(eob);
            let total = len as usize + 1 + !lead as usize;
            let b = e(&syms, total);
            let (mut recs, mut c) = (Vec::new(), [0u32; 256]);
            assert_eq!(mtf::mtf_runs(&b, &mut recs, &mut c, k), Ok(total), "{k:?}");
            let front = if lead {
                b.seq_to_unseq[0]
            } else {
                b.seq_to_unseq[2]
            };
            let mut want = vec![front as u32];
            let after = if lead {
                b.seq_to_unseq[1]
            } else {
                b.seq_to_unseq[0]
            };
            want.push((total as u32 - 1) << 8 | after as u32);
            want.push((total as u32) << 8);
            assert_eq!(recs, want, "{k:?}, lead {lead}: the longest run's records");
            assert_eq!(c[front as usize] as u64, len + !lead as u64, "{k:?}");
            let mut s = Scratch::default();
            let t = run_stage::sort(&recs, &c, total, &mut s).expect("the longest run sorts");
            assert_eq!(t.fo().len(), 3);
            let b = e(&syms, total - 1);
            assert_eq!(
                mtf::mtf_runs(&b, &mut recs, &mut c, k),
                Err(ErrorKind::BlockTooLong),
                "{k:?}"
            );
            p.runs += 2;
            p.avx2 += 2 * k.avx2 as u64;
        }
    }
    eprintln!("miri_emitter_exits: {p:?}");
    assert!(p.accepted > 0 && p.run_too_long > 0 && p.block_too_long > 0);
    assert!(
        p.refused > 0 && p.panicked > 0,
        "the refused records, the panics"
    );
    assert!(p.non_maximal > 0, "records that are no maximal runs");
    if Kernels::detect().avx2 {
        assert!(p.avx2 > 0 && p.avx2_fills > 0);
    }
}

/// The emitter's buffer kept from block to block (as a worker keeps it): no capacity, a
/// short one, a long one, after an error, with a larger block then a smaller, both
/// kernels, every result the production one's
#[test]
fn miri_emitter_kept_buffers() {
    let mut p = Paths::default();
    for k in kernels() {
        let mut recs: Vec<u32> = Vec::new();
        let mut c = [0u32; 256];
        let blocks = [
            symbols(256, (2..=257).collect(), 1000),
            symbols(3, vec![1, 1, 1, 1, 4], 1000),
            symbols(5, [vec![0u16; 22], vec![6]].concat(), 1000),
            symbols(2, vec![2, 2, 0, 2, 3], 1000),
            symbols(1, vec![], 1000),
            symbols(40, (0..600u16).map(|i| 2 + (i * 7) % 40).collect(), 2),
            symbols(
                40,
                (0..600u16).map(|i| 2 + (i * 7) % 40).chain([41]).collect(),
                1000,
            ),
        ];
        for b in &blocks {
            let r = mtf::mtf_runs(b, &mut recs, &mut c, k);
            let (mut ll, mut wc) = (Vec::new(), [0u32; 256]);
            let w = mtf::mtf(b, &mut ll, &mut wc, k);
            assert_eq!(r, w, "{k:?}");
            assert!(recs.len() <= b.syms.len() + 1);
            if r.is_ok() {
                assert_eq!(recs, runs_of(&ll), "{k:?}");
                assert!(c == wc);
            }
            one(b, k, "kept", &mut p);
        }
    }
    eprintln!("miri_emitter_kept_buffers: {p:?}");
}

/// Seeded short streams of every kind under Miri (both kernels): a few of each
#[test]
fn miri_emitter_seeded() {
    let mut p = Paths::default();
    for seed in [1u64, 2, 3] {
        let mut rng = Rng::new(seed);
        for i in 0..KINDS * 2 {
            let b = gen(&mut rng, i % KINDS, 48);
            case(&b, &format!("miri seed {seed} case {i}"), &mut p);
        }
    }
    eprintln!("miri_emitter_seeded: {p:?}");
    assert!(p.runs > 0);
}

// ---- the seeded fuzz --------------------------------------------------------------

/// xorshift64*, seeded (never 0)
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
}

const KINDS: usize = 7;

/// A stream of kind `kind` (0 run-heavy, 1 alternating, 2 single-symbol, 3 max-alphabet,
/// 4 mixed, 5 sparse literals with long runs, 6 corrupt: one of the others broken), up
/// to `len` symbols, its limit varied around its length so each check refuses somewhere
fn gen(rng: &mut Rng, kind: usize, len: usize) -> Symbols {
    // small streams (Miri's) keep their runs and limits small too
    let (bits, loose) = if len <= 64 { (6, 4096) } else { (12, 900_000) };
    let len = rng.below(len as u64 + 1) as usize;
    let mut syms = Vec::with_capacity(len + 1);
    let n_in_use = match kind {
        2 => 1,
        3 => 256,
        1 => 2 + rng.below(3) as usize,
        _ => 1 + rng.below(256) as usize,
    };
    let top = n_in_use as u64; // literal symbols are 2..=n_in_use
    let lit = |rng: &mut Rng| {
        if top < 2 {
            0
        } else {
            2 + rng.below(top - 1) as u16
        }
    };
    match kind {
        0 => {
            while syms.len() < len {
                if rng.below(8) == 0 {
                    syms.push(lit(rng));
                } else {
                    let w = rng.below(bits);
                    syms.extend(run_syms(1 + rng.below(1 << w)));
                }
            }
        }
        1 => {
            // literals of index 1 (two bytes alternating), or a run and a literal by turns
            let with_runs = rng.below(2) == 0;
            while syms.len() < len {
                if with_runs {
                    syms.push(rng.below(2) as u16);
                }
                syms.push(2);
            }
        }
        2 => {
            // one byte: one run, as long as a weight allows at most (a longer stream of
            // counts is a weight too heavy, kind 6's)
            let w = rng.below(bits + 10);
            syms.extend(run_syms(1 + rng.below(1 << w)));
        }
        3 => {
            while syms.len() < len {
                syms.push(match rng.below(4) {
                    0 => 2 + rng.below(255) as u16,
                    1 => 33 + rng.below(224) as u16,
                    2 => [16u16, 17, 32, 33, 34, 64, 65, 255, 256][rng.below(9) as usize],
                    _ => rng.below(2) as u16,
                });
            }
        }
        5 => {
            while syms.len() < len.min(512) {
                let w = rng.below(bits + 5);
                syms.extend(run_syms(1 + rng.below(1 << w)));
                syms.push(lit(rng));
            }
        }
        _ => {
            let share = rng.below(90);
            while syms.len() < len {
                if top < 2 || rng.below(100) < share {
                    syms.push(rng.below(2) as u16);
                } else {
                    syms.push(lit(rng));
                }
            }
        }
    }
    syms.truncate(len);
    let eob = n_in_use as u16 + 1;
    syms.push(eob);
    let mut b = symbols(n_in_use, syms, 0);
    // the limit: the length found, and around it, or loose
    let n = reference_len(&b);
    b.max_len = match rng.below(8) {
        0 => n,
        1 => n.saturating_sub(1),
        2 => n + 1,
        3 => n.saturating_sub(1 + rng.below(70) as usize),
        4 => loose,
        5 => 8 * b.syms.len() + rng.below(3) as usize, // takes_runs' edge
        6 => (8 * b.syms.len()).saturating_sub(1).max(n),
        _ => n + rng.below(1000) as usize,
    };
    if kind == 6 {
        corrupt(rng, &mut b);
    }
    b
}

/// One of: the end removed; the end moved early; a symbol above the end; a symbol past
/// the list (above 256); a weight past its bound; the end 0 or 1; a symbol replaced at
/// random; the limit 0; a byte twice in the map; a leading run with no end
fn corrupt(rng: &mut Rng, b: &mut Symbols) {
    let len = b.syms.len();
    let at = rng.below(len as u64) as usize;
    match rng.below(10) {
        0 => {
            b.syms.pop();
        }
        1 => {
            let eob = b.eob;
            b.syms.insert(at, eob)
        }
        2 => b
            .syms
            .insert(at, b.eob + 1 + rng.below(257 - b.eob as u64) as u16),
        3 => b
            .syms
            .insert(at, 257 + rng.below(u16::MAX as u64 - 256) as u16),
        4 => {
            for _ in 0..22 + rng.below(3) {
                b.syms.insert(at, rng.below(2) as u16);
            }
        }
        5 => b.eob = rng.below(2) as u16,
        6 => b.syms[at] = rng.below(b.eob as u64 + 2) as u16,
        7 => b.max_len = 0,
        8 => {
            // a byte twice in the map (among those in use, so literals name it)
            let top = (b.eob.max(2) as u64 - 1).min(256);
            let (i, j) = (rng.below(top) as usize, rng.below(top) as usize);
            b.seq_to_unseq[i] = b.seq_to_unseq[j];
        }
        _ => {
            b.syms = run_syms(1 + rng.below(100));
            b.max_len = 1000;
        }
    }
}

/// The block's length with no limit (the runs and literals counted to the end, or to
/// the first weight too heavy), to set limits around it
fn reference_len(b: &Symbols) -> usize {
    let (mut n, mut run, mut weight) = (0u64, 0u64, 1u64);
    for &s in &b.syms {
        if s <= 1 {
            if weight >= MAX_RUN_WEIGHT {
                break;
            }
            run += (s as u64 + 1) * weight;
            weight <<= 1;
            continue;
        }
        (n, run, weight) = (n + run, 0, 1);
        if s == b.eob {
            break;
        }
        n += 1;
    }
    n as usize
}

/// The fixed seeds, each run for `CASES` cases of each kind (in turn); the case's
/// longest stream doubles every 64 cases from 64 symbols, to 4096
const SEEDS: [u64; 4] = [0xb5_0001, 0xb5_0002, 0x5eed_d044, 0x0269_fd60];
const CASES: usize = 2048;

fn fuzz_seed(seed: u64, cases: usize, p: &mut Paths) {
    let mut rng = Rng::new(seed);
    for i in 0..cases {
        let kind = i % KINDS;
        let len = 64usize << ((i / 64) % 7);
        let b = gen(&mut rng, kind, len);
        case(&b, &format!("seed {seed:#x} case {i} kind {kind}"), p);
    }
}

#[test]
fn fuzz_emit_sort_fill_fallback() {
    let mut total = Paths::default();
    for seed in SEEDS {
        let mut p = Paths::default();
        fuzz_seed(seed, CASES, &mut p);
        eprintln!("seed {seed:#x}: {CASES} cases: {p:?}");
        total.add(&p);
    }
    // further seeds for a bounded time, if asked
    let secs: u64 = std::env::var("MIBZ2_B5_FUZZ_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let t0 = std::time::Instant::now();
    let (mut extra, mut seed) = (0u64, 0x1000u64);
    while t0.elapsed().as_secs() < secs {
        let mut p = Paths::default();
        fuzz_seed(seed, CASES, &mut p);
        total.add(&p);
        extra += 1;
        seed += 1;
    }
    eprintln!(
        "fuzz: {} fixed seeds x {CASES} cases, {extra} further seeds (0x1000..{seed:#x}) x {CASES} \
         in {:.1}s: {total:?}",
        SEEDS.len(),
        t0.elapsed().as_secs_f64()
    );
    assert!(total.accepted > 0, "accepted");
    assert!(total.fallback > 0, "fallback");
    assert!(total.refused > 0, "refused");
    assert!(total.run_too_long > 0 && total.block_too_long > 0, "errors");
    assert!(total.direct_fills > 0, "direct fills");
    assert!(total.non_maximal > 0, "records that are no maximal runs");
    if Kernels::detect().avx2 {
        assert!(total.avx2 > 0 && total.avx2_fills > 0, "AVX2");
    }
    assert!(total.panicked > 0, "the scalar panics past the list");
}

/// Blocks at the 24-bit representation's edge, natively (too large for Miri): four
/// longest runs and three literals, 2^24 - 5 rows (the first a leading run), taken and the production table; five
/// longest runs, past 2^24: `sort` refuses the records fed to it directly, `takes_runs`
/// sends the block to the column's path, and `build` gives the production table
#[test]
fn representation_limit() {
    let (run, len) = max_run();
    let mut p = Paths::default();
    for runs in [4usize, 5] {
        let mut syms = Vec::new();
        for i in 0..runs {
            if i > 0 {
                syms.push(2);
            }
            syms.extend(&run);
        }
        syms.push(3);
        let n = runs * len as usize + runs - 1;
        let max = if runs == 4 { (1 << 24) - 1 } else { 1 << 25 };
        let b = symbols(2, syms, max);
        assert_eq!(run_stage::takes_runs(&b), runs == 4);
        for k in kernels() {
            let (mut recs, mut c) = (Vec::new(), [0u32; 256]);
            assert_eq!(mtf::mtf_runs(&b, &mut recs, &mut c, k), Ok(n));
            let mut s = Scratch::default();
            assert_eq!(
                run_stage::sort(&recs, &c, n, &mut s).is_some(),
                runs == 4,
                "n {n}: sort"
            );
            one(&b, k, &format!("{runs} longest runs, n {n}"), &mut p);
        }
    }
    eprintln!("representation_limit: {p:?}");
    assert!(p.accepted > 0 && p.fallback > 0);
}
