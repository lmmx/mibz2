//! Targeted Miri cases for constructor v2 (reports 211-212): only the classes that the
//! interrupted `ctor2-synthetic --quick` Miri run never reached. These are the exact
//! 65,533/65,534 span cases, forced RAW, malformed counts and rejected builds, portions
//! finished at the end of input, and reuse across block sizes.
//!
//! This module only selects and runs cases. It calls the frozen `Ctor2::build` (ctor2.rs
//! at 8beffe2, unchanged) and the oracle `C30::build`, both read-only, and compares
//! every byte of `rec` and `raw`. The arenas are never poisoned, so Miri sees any read of
//! an unwritten byte. The walker is not run here: the release suites cover it.
//!
//!     cargo +nightly miri run --example c30walk -- c2miri small
//!     cargo +nightly miri run --example c30walk -- c2miri span
//!     cargo +nightly miri run --example c30walk -- c2miri large
//!
//! `small` takes minutes; each case in `span` and `large` (61,000+ rows) takes several
//! minutes under Miri. Every case prints a line as it finishes, so an interrupted run
//! still records what it covered.

use super::ctor2::{Ctor2, Stats2};
use super::{mix, phi, RawG, Rec, C30, M20, RAWF, RAW_BIT};
use std::io::Write;

const EPOCH: usize = 61_440;

struct Run {
    oracle: C30,
    ctor: Ctor2,
    rec: Vec<Rec>,
    raw: Vec<RawG>,
    cases: u64,
}

fn hist(ll: &[u8]) -> [u32; 256] {
    let mut c = [0u32; 256];
    for &b in ll {
        c[b as usize] += 1;
    }
    c
}

/// Filler that never forms a uniform chunk: bytes 2 and 3 alternating (as ctor2's suite)
fn filler(n: usize) -> Vec<u8> {
    (0..n).map(|k| 2 + (k & 1) as u8).collect()
}

fn bytes<T>(v: &[T]) -> &[u8] {
    // SAFETY: Rec and RawG have no padding (size and offset asserted in c30walk.rs)
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v)) }
}

struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 = mix(self.0.wrapping_add(0x9e37_79b9_7f4a_7c15));
        (self.0 % n as u64) as usize
    }
}

impl Run {
    /// One valid block: both builds into the reused arenas, each against the oracle byte
    /// for byte; returns the counted build's counters
    fn case(&mut self, what: &str, ll: &[u8], pm: u32) -> Stats2 {
        let counts = hist(ll);
        self.ctor.st = Stats2::default();
        assert!(
            self.ctor
                .build::<true>(ll, &counts, pm, &mut self.rec, &mut self.raw),
            "{what}: the counted build rejected valid counts"
        );
        let st = std::mem::take(&mut self.ctor.st);
        if ll.is_empty() {
            // the oracle needs n >= 1; the empty block is PARK alone
            let park = Rec {
                f: [RAWF; 30],
                h: RAW_BIT | M20,
            };
            assert!(self.raw.is_empty() && self.rec.len() == 1 && self.rec[0] == park);
        } else {
            let _ = self.oracle.build(ll, &counts, pm);
            self.same(what, "counted");
        }
        assert!(
            self.ctor
                .build::<false>(ll, &counts, pm, &mut self.rec, &mut self.raw),
            "{what}: the uncounted build rejected valid counts"
        );
        if ll.is_empty() {
            assert!(self.raw.is_empty() && self.rec.len() == 1);
        } else {
            self.same(what, "uncounted");
        }
        self.cases += 1;
        println!(
            "c2miri: {what} (n {}, pm {pm}): exact; finished fast {} ordinary {} wide {} crossing {} forced {}; epochs {}",
            ll.len(),
            st.fin_fast,
            st.fin_ordinary,
            st.fin_wide,
            st.fin_crossing,
            st.fin_forced,
            st.epochs
        );
        let _ = std::io::stdout().flush();
        st
    }

    fn same(&self, what: &str, which: &str) {
        assert_eq!(self.oracle.rec.len(), self.rec.len(), "{what} {which}: rec length");
        assert_eq!(self.oracle.raw.len(), self.raw.len(), "{what} {which}: raw length");
        assert!(bytes(&self.oracle.rec) == bytes(&self.rec), "{what} {which}: rec bytes");
        assert!(bytes(&self.oracle.raw) == bytes(&self.raw), "{what} {which}: raw bytes");
    }

    /// Counts that are not `ll`'s own: both builds must reject them and leave both
    /// arrays empty, and a valid build must follow in the same arenas
    fn malformed(&mut self, what: &str, ll: &[u8], c: &[u32; 256], pm: u32) {
        let own = hist(ll);
        let ok = self.ctor.build::<true>(ll, c, pm, &mut self.rec, &mut self.raw);
        let ok2 = self.ctor.build::<false>(ll, c, pm, &mut self.rec, &mut self.raw);
        assert_eq!(ok, ok2, "{what}: the builds disagree");
        assert_eq!(ok, *c == own, "{what}: accepted iff the counts are ll's own");
        if !ok {
            assert!(self.rec.is_empty() && self.raw.is_empty(), "{what}: arrays left");
        }
        self.cases += 1;
        println!(
            "c2miri: {what} (n {}, pm {pm}): {}",
            ll.len(),
            if ok { "accepted (own counts)" } else { "rejected, arrays empty" }
        );
        let _ = std::io::stdout().flush();
    }
}

/// L with byte 1 at `ones` (and filler elsewhere), n = last + 200, as ctor2's span cases
fn span_block(first: usize, last: usize) -> Vec<u8> {
    let mut ones: Vec<usize> = (0..29).map(|t| first + 2 * t).collect();
    ones.retain(|&x| x < last);
    ones.truncate(29);
    ones.push(last);
    let mut v = filler(last + 200);
    for &s in &ones {
        v[s] = 1;
    }
    v
}

/// A block with a rare byte 9 at late rows only (its portions finish at the end of input)
/// and byte 1 early, as ctor2's pending class
fn pending_block(n: usize, count: usize) -> Vec<u8> {
    let mut v = filler(n);
    for t in 0..count {
        v[n - 3 - 3 * t] = 9;
    }
    for t in 0..count {
        v[1 + 2 * t] = 1;
    }
    v
}

fn runs_block(r: &mut Rng, n: usize, sigma: usize, step: u8) -> Vec<u8> {
    let mut v = Vec::with_capacity(n);
    while v.len() < n {
        let b = (r.below(sigma) as u8).wrapping_mul(step);
        let len = [1, 16, 30, 31, 64][r.below(5)];
        v.extend(std::iter::repeat(b).take(len.min(n - v.len())));
    }
    v
}

fn small(run: &mut Run) {
    let mut r = Rng(212);
    // forced RAW (no completed Miri class used permille > 0): runs, random, a bucket
    // boundary inside a group, and one symbol, at permille 50 and 1000
    for &n in &[61usize, 95, 150, 200] {
        let v = runs_block(&mut r, n, 4, 40);
        for pm in [50, 1000] {
            run.case(&format!("forced RAW, runs n {n}"), &v, pm);
        }
        let v: Vec<u8> = (0..n).map(|_| r.below(5) as u8).collect();
        run.case(&format!("forced RAW, random n {n}"), &v, 1000);
    }
    let mut v = filler(61);
    for t in 0..17 {
        v[t] = 1; // byte 1's bucket ends at 17, inside group 0
    }
    run.case("forced RAW with a crossing group", &v, 50);
    run.case("forced RAW, one symbol", &[7u8; 61], 1000);

    // malformed counts, the five kinds of ctor2's class 12, each at the three
    // configurations, then a valid build in the same arenas after each rejection
    for t in 0..15 {
        let n = [1usize, 29, 31, 61, 90][t % 5];
        let v = runs_block(&mut r, n, 4, 3);
        let own = hist(&v);
        let mut c = own;
        match t % 5 {
            0 => c[r.below(256)] += 1,
            1 => c[v[r.below(n)] as usize] -= 1,
            2 | 3 => {
                let b = v[r.below(n)] as usize;
                let k = 1 + r.below(c[b] as usize);
                c[b] -= k as u32;
                c[r.below(256)] += k as u32;
            }
            _ => {
                c.swap(0, 3);
                c.swap(6, 9);
            }
        }
        let pm = [0, 50, 1000][t % 3];
        run.malformed(&format!("malformed counts kind {}", t % 5), &v, &c, pm);
        run.case("valid build after a malformed one", &v, pm);
    }

    // portions finished only at the end of input, in partial and full last groups
    for count in [1usize, 29, 30, 31] {
        for pm in [0, 1000] {
            run.case(
                &format!("pending to the end, count {count}"),
                &pending_block(200, count),
                pm,
            );
        }
    }

    // reuse across sizes, down to 0 and back up (capacity growth after shrinking), the
    // forced configuration changing the RAW reserve
    for (k, &n) in [1usize, 0, 31, 200, 95, 0, 150, 1, 230].iter().enumerate() {
        let v = runs_block(&mut r, n, 6, 40);
        run.case(&format!("reuse step {k} size {n}"), &v, [0, 1000, 50][k % 3]);
    }
}

fn span(run: &mut Run) {
    assert_eq!(phi(61_440) - phi(3), 65_533);
    assert_eq!(phi(61_440) - phi(2), 65_534);
    let st = run.case("span 3 -> 61440 (65,533, ordinary)", &span_block(3, 61_440), 0);
    assert_eq!(st.fin_wide, 0, "65,533 must stay ordinary");
    let st = run.case("span 2 -> 61440 (65,534, RAW)", &span_block(2, 61_440), 0);
    assert_eq!(st.fin_wide, 1, "65,534 must become RAW");
}

fn large(run: &mut Run) {
    // pending portions across an epoch, finished only at the end of input
    for count in [1usize, 31] {
        run.case(
            &format!("pending to the end across an epoch, count {count}"),
            &pending_block(61_500, count),
            0,
        );
    }
    // forced RAW and reuse at epoch scale: grow, shrink, grow
    let mut r = Rng(61_500);
    let v = runs_block(&mut r, 61_500, 6, 40);
    run.case("forced RAW across an epoch", &v, 1000);
    run.case("reuse: shrink to 31 after an epoch-scale build", &[5u8; 31], 0);
    run.case("reuse: grow again across an epoch", &v, 50);
    // far wider than an epoch: low 16 bits of the difference look small
    let st = run.case(
        "far wide: 0 -> 2 epochs + 1",
        &span_block(0, 2 * EPOCH + 1),
        0,
    );
    assert_eq!(st.fin_wide, 1, "a two-epoch span must become RAW");
}

pub fn main(args: &[String]) {
    let mut run = Run {
        oracle: C30::default(),
        ctor: Ctor2::default(),
        rec: Vec::new(),
        raw: Vec::new(),
        cases: 0,
    };
    let set = args.first().map(String::as_str).unwrap_or("small");
    match set {
        "small" => small(&mut run),
        "span" => span(&mut run),
        "large" => large(&mut run),
        _ => panic!("c2miri: unknown set {set} (small, span or large)"),
    }
    println!("c2miri {set}: {} cases, every check exact", run.cases);
}
