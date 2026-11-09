//! examples/fuzz.rs's MODE run-fed-direct (built with `--features run-fed`): C31's direct
//! placement, `run_stage::build_direct` and `run_stage::place`, differentially fuzzed
//! (docs/journal/2026-11-09-run-fed-direct.md §10, Guardian D055).
//!
//! Three sources of cases, each case's input its seed's and its index's alone
//! (`MIBZ2_FUZZ_CASE=N` replays case N):
//! - **streams:** the classic corpus's streams and crafted run-heavy ones, edited (or not),
//!   then scanned; every block the scanner yields;
//! - **columns:** a last column shaped to reach the placement's edges (one symbol, all 256,
//!   r = n, the empty block, short buckets and runs that end within 8 rows of a bucket's
//!   end), its MTF symbols at a limit chosen around its length, so that some cases are
//!   refused (`BlockTooLong`), some fall back (`takes_runs` refuses), some carry a run too
//!   long (`RunTooLong`), and most take the records;
//! - **tilings:** a column's records, maximal or split, tampered with (starts moved, records
//!   swapped, dropped, duplicated or rewritten, the closing record or a count moved, n
//!   off by one), and `place` held to an independent oracle of acceptance and of the table
//!   (`tiling_ok`, then the column the records spell, through `bwt::build`).
//!
//! Each block from the first two goes through `build_direct` on the scalar kernel and the
//! detected one, each with buffers kept dirty from case to case (and every fourth case
//! fresh ones too), against production's `mtf::mtf` then `bwt::build`: the same result,
//! error kind included, and on success the same table, and the path `takes_runs`'s. A
//! records' block that comes back by the column (a refused tiling, hidden by the right
//! table) is a failure, and so is a panic (the fuzz profile keeps `build_direct`'s debug
//! assertion). Blocks are counted by path, by records and by the fallback, and those whose
//! placement made one-slot tail stores (the ownership tail reached).
//!
//! A failure is saved, minimised (symbols removed while it still fails, or the column's
//! bytes, the tampering replayed), to OUT_DIR. `MIBZ2_FUZZ_INJECT=1` makes the comparison
//! fail on purpose, so the saving and the minimiser can be seen working.

use crate::common::*;
use mibz2::block::Symbols;
use mibz2::bwt::{build, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, run_stage, Kernels};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

/// One kernel's buffers, kept from case to case (never cleared between them)
struct Dirty {
    recs: Vec<u32>,
    ll: Vec<u8>,
    counts: [u32; 256],
    tt: TVector,
}

impl Default for Dirty {
    fn default() -> Self {
        Dirty {
            recs: Vec::new(),
            ll: Vec::new(),
            counts: [0; 256],
            tt: TVector::new(),
        }
    }
}

#[derive(Default)]
struct Stats {
    cases: [u64; 3],
    blocks: u64,
    records: u64,
    fallback: u64,
    refused: BTreeMap<String, u64>,
    tails: u64,
    empty: u64,
    tilings_valid: u64,
    tilings_tampered_valid: u64,
    tilings_refused: u64,
    failures: u64,
}

pub fn run(secs: u64, seed: u64, out_dir: &Option<String>) -> u64 {
    let kernels = {
        let mut v = vec![Kernels::scalar()];
        if Kernels::detect() != Kernels::scalar() {
            v.push(Kernels::detect());
        }
        v
    };
    let inject = std::env::var_os("MIBZ2_FUZZ_INJECT").is_some();
    let only: Option<u64> = std::env::var("MIBZ2_FUZZ_CASE")
        .ok()
        .and_then(|s| s.parse().ok());
    let mut rng = Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1);
    let corpus = stream_corpus(&mut rng);
    let mut dirty: Vec<Dirty> = kernels.iter().map(|_| Dirty::default()).collect();
    let mut st = Stats::default();
    let deadline = Instant::now() + Duration::from_secs(secs);
    let mut case = 0u64;
    while Instant::now() < deadline {
        case += 1;
        if only.is_some_and(|o| o != case) {
            if only.is_some_and(|o| case > o) {
                break;
            }
            continue;
        }
        let mut r = Rng((seed ^ case.wrapping_mul(0x2545_f491_4f6c_dd1d)) | 1);
        let source = r.below(3) as usize;
        st.cases[source] += 1;
        let ctx = Ctx {
            kernels: &kernels,
            inject,
            fresh: case % 4 == 0,
        };
        match source {
            0 => {
                let mut t = corpus[r.below(corpus.len() as u64) as usize].clone();
                for _ in 0..r.below(4) {
                    edit(&mut t, &corpus, &mut r);
                }
                let mut sc = Scanner::new(SliceSource(&t));
                let mut i = 0;
                while let Ok(Event::Block { symbols, .. }) = sc.next(Vec::new()) {
                    if let Err(why) = ctx.block(&symbols, &mut dirty, &mut st) {
                        let min = minimise_syms(&symbols, &|b| {
                            ctx.block(b, &mut fresh(&kernels), &mut Stats::default())
                                .is_err()
                        });
                        save(out_dir, case, &why, &t, &syms_text(&min));
                        st.failures += 1;
                    }
                    i += 1;
                    if i >= 4 {
                        break;
                    }
                }
            }
            1 => {
                let ll = column(&mut r);
                let pick = r.next();
                let b = symbols_of(&ll, pick);
                if let Err(why) = ctx.block(&b, &mut dirty, &mut st) {
                    let min = minimise_syms(&b, &|b| {
                        ctx.block(b, &mut fresh(&kernels), &mut Stats::default())
                            .is_err()
                    });
                    save(out_dir, case, &why, &ll, &syms_text(&min));
                    st.failures += 1;
                }
            }
            _ => {
                let ll = column(&mut r);
                let ops: Vec<u64> = (0..r.below(4)).map(|_| r.next()).collect();
                let split = r.next();
                if let Err(why) = ctx.tiling(&ll, split, &ops, &mut dirty, &mut st) {
                    let min = minimise_bytes(&ll, &|l| {
                        ctx.tiling(l, split, &ops, &mut fresh(&kernels), &mut Stats::default())
                            .is_err()
                    });
                    let (recs, counts, n) = tampered(&min, split, &ops);
                    save(
                        out_dir,
                        case,
                        &why,
                        &min,
                        &format!(
                            "ops: {ops:?}\nsplit: {split}\nrecs: {recs:?}\ncounts (nonzero): \
                             {:?}\nn: {n}\n",
                            nonzero(&counts)
                        ),
                    );
                    st.failures += 1;
                }
            }
        }
    }
    println!(
        "{case} cases in {secs} s (seed {seed}, mode run-fed-direct, kernels {:?}{}), {} \
         failures",
        kernels,
        if inject { ", INJECTED failures" } else { "" },
        st.failures
    );
    println!(
        "  by source: streams {}, columns {}, tilings {}",
        st.cases[0], st.cases[1], st.cases[2]
    );
    println!(
        "  blocks through build_direct (each on every kernel): {}; by records {} (empty {}, \
         with one-slot tail stores {}), by the fallback {}; refused, as production: {:?}",
        st.blocks, st.records, st.empty, st.tails, st.fallback, st.refused
    );
    println!(
        "  tilings through place: untampered valid {}, tampered still valid {}, refused {} \
         (each as the oracle says)",
        st.tilings_valid, st.tilings_tampered_valid, st.tilings_refused
    );
    st.failures
}

fn fresh(kernels: &[Kernels]) -> Vec<Dirty> {
    kernels.iter().map(|_| Dirty::default()).collect()
}

struct Ctx<'a> {
    kernels: &'a [Kernels],
    inject: bool,
    fresh: bool,
}

impl Ctx<'_> {
    /// One block through `build_direct`, every kernel, against production
    fn block(&self, b: &Symbols, dirty: &mut [Dirty], st: &mut Stats) -> Result<(), String> {
        let (mut ll, mut counts, mut want) = (Vec::new(), [0u32; 256], TVector::new());
        let r = mtf::mtf(b, &mut ll, &mut counts, Kernels::scalar());
        if r.is_ok() {
            build(&ll, &counts, &mut want);
        }
        let runs = run_stage::takes_runs(b);
        st.blocks += 1;
        let mut path = None;
        for (k, d) in self.kernels.iter().zip(dirty.iter_mut()) {
            let mut fresh_d = Dirty::default();
            let sets: Vec<&mut Dirty> = if self.fresh {
                vec![d, &mut fresh_d]
            } else {
                vec![d]
            };
            for x in sets {
                let got = catch_unwind(AssertUnwindSafe(|| {
                    run_stage::build_direct(b, &mut x.recs, &mut x.ll, &mut x.counts, &mut x.tt, *k)
                }))
                .map_err(|_| format!("{k:?}: build_direct panicked"))?;
                match (&r, got) {
                    (Ok(n), Ok((m, by_runs))) => {
                        if m != *n {
                            return Err(format!("{k:?}: length {m}, production's {n}"));
                        }
                        if by_runs != runs {
                            return Err(format!(
                                "{k:?}: path {} where takes_runs says {}",
                                if by_runs { "records" } else { "the column" },
                                if runs {
                                    "records (a hidden fallback)"
                                } else {
                                    "the column"
                                }
                            ));
                        }
                        let inj = self.inject && *n > 20 && *n % 7 == 3;
                        if x.tt.as_slice() != want.as_slice() || inj {
                            return Err(format!(
                                "{k:?}: the table differs{}",
                                if inj { " (INJECTED)" } else { "" }
                            ));
                        }
                        path = Some(by_runs);
                    }
                    (Err(e), Err(f)) if *e == f => {}
                    (want_r, got_r) => {
                        return Err(format!(
                            "{k:?}: {got_r:?}, production's {:?}",
                            want_r.as_ref().map(|_| "Ok")
                        ))
                    }
                }
            }
        }
        match (&r, path) {
            (Ok(n), Some(true)) => {
                st.records += 1;
                st.empty += (*n == 0) as u64;
                st.tails += has_tail(&ll, &counts) as u64;
            }
            (Ok(_), _) => st.fallback += 1,
            (Err(e), _) => *st.refused.entry(format!("{e:?}")).or_default() += 1,
        }
        Ok(())
    }

    /// A column's records, tampered with, through `place`, every kernel, against the oracle
    fn tiling(
        &self,
        ll: &[u8],
        split: u64,
        ops: &[u64],
        dirty: &mut [Dirty],
        st: &mut Stats,
    ) -> Result<(), String> {
        let (recs, counts, n) = tampered(ll, split, ops);
        let ok = tiling_ok(&recs, &counts, n);
        let inj = self.inject && recs.len() == 5;
        let want = if ok {
            let l = spell(&recs);
            let mut t = TVector::new();
            build(&l, &counts, &mut t);
            Some(t)
        } else {
            None
        };
        for (k, d) in self.kernels.iter().zip(dirty.iter_mut()) {
            let got = catch_unwind(AssertUnwindSafe(|| {
                run_stage::place(&recs, &counts, n, &mut d.tt, *k)
            }))
            .map_err(|_| format!("{k:?}: place panicked"))?;
            if got != ok || inj {
                return Err(format!(
                    "{k:?}: place {} where the oracle {}{}",
                    if got { "accepts" } else { "refuses" },
                    if ok { "accepts" } else { "refuses" },
                    if inj { " (INJECTED)" } else { "" }
                ));
            }
            match &want {
                Some(t) if d.tt.as_slice() != t.as_slice() => {
                    return Err(format!("{k:?}: an accepted tiling's table differs"))
                }
                None if !d.tt.as_slice().is_empty() => {
                    return Err(format!("{k:?}: a refused tiling left a table"))
                }
                _ => {}
            }
        }
        match (ok, ops.is_empty()) {
            (true, true) => st.tilings_valid += 1,
            (true, false) => st.tilings_tampered_valid += 1,
            (false, _) => st.tilings_refused += 1,
        }
        Ok(())
    }
}

/// Whether records tile `[0, n)` as `place` requires, written independently of it: n
/// below 2^24; the empty block exactly `[0]` with no counts; otherwise starts from 0
/// rising strictly to the closing n, and each byte's run lengths summing to its count
fn tiling_ok(recs: &[u32], counts: &[u32; 256], n: usize) -> bool {
    if n >= 1 << 24 {
        return false;
    }
    if n == 0 {
        return recs == [0] && counts.iter().all(|&c| c == 0);
    }
    if recs.len() < 2 || recs[0] >> 8 != 0 || recs[recs.len() - 1] >> 8 != n as u32 {
        return false;
    }
    let mut per = [0u64; 256];
    for w in recs.windows(2) {
        let (a, b) = (w[0] >> 8, w[1] >> 8);
        if b <= a {
            return false;
        }
        per[(w[0] & 0xff) as usize] += (b - a) as u64;
    }
    per.iter().zip(counts).all(|(&p, &c)| p == c as u64)
}

/// The column valid records spell
fn spell(recs: &[u32]) -> Vec<u8> {
    let mut l = Vec::new();
    for w in recs.windows(2) {
        l.extend(std::iter::repeat_n(
            (w[0] & 0xff) as u8,
            ((w[1] >> 8) - (w[0] >> 8)) as usize,
        ));
    }
    l
}

/// A column's records (maximal, or split at points `split` picks: still a tiling), its
/// counts and n, then each of `ops` applied (each picks its edit and place by modulo, so
/// any op applies to any column)
fn tampered(ll: &[u8], split: u64, ops: &[u64]) -> (Vec<u32>, [u32; 256], usize) {
    let mut recs = Vec::new();
    let mut counts = [0u32; 256];
    let mut s = Rng(split | 1);
    let every = [0u64, 0, 3, 7, 17][(split % 5) as usize];
    for (i, &c) in ll.iter().enumerate() {
        counts[c as usize] += 1;
        let cut = every > 0 && s.below(every) == 0;
        if i == 0 || c != ll[i - 1] || cut {
            recs.push((i as u32) << 8 | c as u32);
        }
    }
    recs.push((ll.len() as u32) << 8);
    let mut n = ll.len();
    for &op in ops {
        let len = recs.len();
        let j = ((op >> 8) % len as u64) as usize;
        let v = op >> 32;
        match op % 10 {
            0 => recs[j] = recs[j].wrapping_add(256),
            1 => recs[j] = recs[j].wrapping_sub(256),
            2 if len > 1 => recs.swap(j, (j + 1) % len),
            3 if len > 1 => {
                recs.remove(j);
            }
            4 => recs.insert(j, recs[j]),
            5 => recs[j] = recs[j] & !0xff | (v as u32 & 0xff),
            6 => {
                let c = (v % 256) as usize;
                let d = ((v >> 8) % 256) as usize;
                if counts[c] > 0 {
                    counts[c] -= 1;
                    counts[d] += 1;
                }
            }
            7 => n = n.wrapping_add(1),
            8 => n = n.saturating_sub(1),
            _ => {
                // a valid edit: split a run, if one is longer than a row
                if j + 1 < len && (recs[j + 1] >> 8) > (recs[j] >> 8) + 1 && recs[j] >> 8 < n as u32
                {
                    let mid = (recs[j] & !0xff) + 256 | (recs[j] & 0xff);
                    recs.insert(j + 1, mid);
                }
            }
        }
    }
    (recs, counts, n)
}

/// A column shaped to reach the placement's edges
fn column(r: &mut Rng) -> Vec<u8> {
    let n = [
        0usize, 1, 2, 7, 8, 9, 15, 16, 17, 24, 40, 64, 200, 1000, 5000, 70_000,
    ][r.below(16) as usize];
    let sigma = [1usize, 2, 3, 4, 8, 40, 256][r.below(7) as usize];
    let alpha: Vec<u8> = {
        let base = r.next() as u8;
        (0..sigma)
            .map(|i| base.wrapping_add((i * 37) as u8))
            .collect()
    };
    match r.below(6) {
        // all 256, each once (r = n), in some order
        0 if n >= 256 => {
            let mut v: Vec<u8> = (0..=255).collect();
            for i in (1..256).rev() {
                v.swap(i, r.below(i as u64 + 1) as usize);
            }
            v.into_iter().cycle().take(n).collect()
        }
        // r = n: no two neighbours equal
        1 if sigma > 1 => {
            let mut v: Vec<u8> = Vec::with_capacity(n);
            while v.len() < n {
                let c = alpha[r.below(sigma as u64) as usize];
                if v.last() != Some(&c) {
                    v.push(c);
                }
            }
            v
        }
        // buckets sized about a multiple of 8, filled by runs whose ends fall near them
        2 => {
            let mut v = Vec::new();
            for &c in &alpha {
                let size = 8 * (1 + r.below(3) as usize) + r.below(3) as usize;
                let size = size.saturating_sub(r.below(2) as usize * 2);
                v.extend(std::iter::repeat_n(c, size));
            }
            // shuffle the rows in runs, so that L's runs cross the buckets' ends
            let mut out = Vec::new();
            while !v.is_empty() {
                let k = (1 + r.below(12) as usize).min(v.len());
                let at = r.below((v.len() - k + 1) as u64) as usize;
                out.extend(v.drain(at..at + k));
            }
            out
        }
        // runs of skewed lengths
        _ => {
            let mut v = Vec::with_capacity(n);
            while v.len() < n {
                let c = alpha[r.below(sigma as u64) as usize];
                let run = [1, 1, 1, 2, 3, 7, 8, 9, 15, 16, 17, 24][r.below(12) as usize];
                v.extend(std::iter::repeat_n(c, run));
            }
            v.truncate(n);
            v
        }
    }
}

/// A column's block: its MTF symbols over its bytes (the empty column over one byte),
/// at a limit `pick` chooses around its length, now and then with a run past the weight
/// libbzip2 allows spliced in
fn symbols_of(ll: &[u8], pick: u64) -> Symbols {
    let mut used: Vec<u8> = {
        let mut u = [false; 256];
        for &c in ll {
            u[c as usize] = true;
        }
        (0..256).filter(|&c| u[c]).map(|c| c as u8).collect()
    };
    if used.is_empty() {
        used.push((pick >> 16) as u8);
    }
    let mut syms = mtf_symbols(ll, &used);
    let n = ll.len();
    let max_len = match pick % 8 {
        0 => n,
        1 => n.saturating_sub(1),
        2 => n + 1,
        // a bound that makes `takes_runs` refuse: fewer than 8 rows a symbol
        3 => (syms.len() * 8).saturating_sub(1).max(n),
        _ => 900_000,
    };
    if pick % 29 == 0 {
        // 22 RUNBs: a run whose weight passes 2^21
        let at = ((pick >> 8) as usize) % syms.len();
        for _ in 0..22 {
            syms.insert(at, 1);
        }
    }
    let mut seq_to_unseq = [0u8; 256];
    seq_to_unseq[..used.len()].copy_from_slice(&used);
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq,
        syms,
        eob: used.len() as u16 + 1,
        max_len,
    }
}

/// Whether the placement of a column's (maximal) records makes a one-slot tail store: a
/// cell of ℓ rows from q, in a bucket ending at b, with ℓ > 8·min(⌈ℓ/8⌉, ⌊(b − q)/8⌋)
fn has_tail(ll: &[u8], counts: &[u32; 256]) -> bool {
    let (mut next, mut end) = ([0u32; 256], [0u32; 256]);
    let mut rows = 0u32;
    for c in 0..256 {
        next[c] = rows;
        rows += counts[c];
        end[c] = rows;
    }
    // L's run ends, for cutting the F intervals into cells
    let mut ends = Vec::new();
    for i in 1..=ll.len() {
        if i == ll.len() || ll[i] != ll[i - 1] {
            ends.push(i as u32);
        }
    }
    let mut i = 0;
    while i < ll.len() {
        let c = ll[i] as usize;
        let mut j = i + 1;
        while j < ll.len() && ll[j] == ll[i] {
            j += 1;
        }
        let (q0, b) = (next[c], end[c]);
        let e = q0 + (j - i) as u32;
        let mut q = q0;
        while q < e {
            let k = ends.partition_point(|&x| x <= q);
            let ce = e.min(ends[k]);
            let l = (ce - q) as usize;
            let v = l.div_ceil(8).min(((b - q) / 8) as usize);
            if l > 8 * v {
                return true;
            }
            q = ce;
        }
        next[c] = e;
        i = j;
    }
    false
}

/// The run-heavy and classic streams the stream cases edit
fn stream_corpus(rng: &mut Rng) -> Vec<Vec<u8>> {
    let mut corpus = Vec::new();
    for (_, data) in samples(rng) {
        let small = &data[..data.len().min(60_000)];
        corpus.push(compress(small, 9));
        corpus.push(compress(small, 1));
    }
    corpus.push(compress(b"", 9));
    corpus.push([compress(b"a", 9), compress(&[b'z'; 3000], 1)].concat());
    for _ in 0..12 {
        let ll = column(rng);
        if ll.is_empty() {
            continue;
        }
        let p = rng.below(ll.len() as u64) as u32;
        let z = craft(9, &[Spec::of(ll, p)], None);
        corpus.push(z);
    }
    corpus
}

/// One edit of a stream: a bit flipped, a byte set, a cut, an insertion, a deletion
fn edit(t: &mut Vec<u8>, corpus: &[Vec<u8>], r: &mut Rng) {
    if t.is_empty() {
        t.push(r.next() as u8);
        return;
    }
    let i = r.below(t.len() as u64) as usize;
    match r.below(6) {
        0 | 1 => t[i] ^= 1 << r.below(8),
        2 => t[i] = r.next() as u8,
        3 => t.truncate(i),
        4 => t.insert(i, r.next() as u8),
        _ => {
            let o = &corpus[r.below(corpus.len() as u64) as usize];
            let j = r.below(o.len() as u64) as usize;
            t.truncate(i);
            t.extend_from_slice(&o[j..]);
        }
    }
}

fn with_syms(b: &Symbols, syms: Vec<u16>) -> Symbols {
    Symbols {
        crc: b.crc,
        orig_ptr: b.orig_ptr,
        seq_to_unseq: b.seq_to_unseq,
        syms,
        eob: b.eob,
        max_len: b.max_len,
    }
}

/// `b` with symbols removed while `fails` holds (the last, the end of block, kept)
fn minimise_syms(b: &Symbols, fails: &dyn Fn(&Symbols) -> bool) -> Symbols {
    let mut cur = with_syms(b, b.syms.clone());
    let mut chunk = cur.syms.len() / 2;
    while chunk > 0 {
        let mut at = 0;
        while at + chunk < cur.syms.len() {
            let mut t = cur.syms.clone();
            t.drain(at..at + chunk);
            let cand = with_syms(&cur, t);
            if fails(&cand) {
                cur = cand;
            } else {
                at += chunk;
            }
        }
        chunk /= 2;
    }
    cur
}

/// `ll` with bytes removed while `fails` holds
fn minimise_bytes(ll: &[u8], fails: &dyn Fn(&[u8]) -> bool) -> Vec<u8> {
    let mut cur = ll.to_vec();
    let mut chunk = cur.len() / 2;
    while chunk > 0 {
        let mut at = 0;
        while at + chunk <= cur.len() {
            let mut t = cur.clone();
            t.drain(at..at + chunk);
            if fails(&t) {
                cur = t;
            } else {
                at += chunk;
            }
        }
        chunk /= 2;
    }
    cur
}

fn syms_text(b: &Symbols) -> String {
    format!(
        "syms: {:?}\neob: {}\nseq_to_unseq: {:?}\nmax_len: {}\n",
        b.syms,
        b.eob,
        &b.seq_to_unseq[..b.eob as usize - 1],
        b.max_len
    )
}

fn nonzero(counts: &[u32; 256]) -> Vec<(usize, u32)> {
    counts
        .iter()
        .enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(i, &c)| (i, c))
        .collect()
}

fn save(dir: &Option<String>, case: u64, why: &str, input: &[u8], witness: &str) {
    eprintln!("case {case}: {why}");
    if let Some(d) = dir {
        let _ = std::fs::create_dir_all(d);
        let _ = std::fs::write(format!("{d}/case-{case}.input"), input);
        let _ = std::fs::write(
            format!("{d}/case-{case}.witness.txt"),
            format!("case {case}: {why}\n{witness}"),
        );
    }
}
