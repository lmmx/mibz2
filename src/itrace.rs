//! Research only, behind the `induction-trace` feature (off by default): a record of the
//! shipped lazy Cells kernel's induction, block by block, for MONODROMY's study of the
//! induction's actual paths (#145, QUEUED 6915302968). Without the feature this module is
//! not compiled, and the kernel's hooks are not either.
//!
//! **What the kernel gives.** While a thread records (`Recording`), `induce_lazy` logs each
//! cell it makes, as `(ra, rb, la, lb)`: what was left of the top and bottom front letters,
//! in rows, and their words' lengths with the mark bit. It also logs each letter that closes
//! (the same letter last in both orders), with its word's length; the last closes with the
//! mark, and the induction ends there. The kernel names no letter: its slots are reused.
//!
//! **What the replay adds.** `record` builds the same letters from the same run records as
//! the kernel's first stage, and runs the same induction on them, safely, as an interval
//! exchange: two linked orders, a letter's λ, and its word. A step cuts a cell of width
//! min(λA, λB) from A, last on top, and B, last on the bottom. The cell's word is A's then
//! B's. It lies at the right end of B's place on top and of A's place on the bottom, and
//! both letters lose its width. A letter at 0 leaves both orders; a letter last in both
//! orders at the start of a step with both letters whole closes and leaves them. Each letter
//! has an id: 0..r for the block's letters in top order, r + i for the cell of step i.
//!
//! **The two checks.** A record is returned only if the replay's steps equal the kernel's
//! log, field for field and in order, and the final word expands to the kernel's output,
//! byte for byte. Anything else is an error, and nothing is written.
//!
//! `examples/induction_trace.rs` writes the records; `bench/irec.py` reads them.

use crate::block::Symbols;
use crate::bwt::{ForcedLazyCells, InverseBwt, LazyCells};
use crate::runs::{front, Runs};
use crate::Kernels;
use std::cell::RefCell;

/// The mark: the bit a word's length carries when the word holds the walk's row
const MARK: u32 = 1 << 31;
/// The longest word kept as bytes in the arena (`cells.rs`'s `SMALL`); longer, or marked,
/// it is a rule
const SMALL: u32 = 16;
/// Where the block starts after the walk's row's byte in the cycle's word (`cells.rs`'s
/// `SHIFT`)
const SHIFT: usize = 1;

/// The kernel's log of one induction
#[derive(Default, Debug)]
struct Log {
    /// a cell: (ra, rb, la, lb)
    cells: Vec<[u32; 4]>,
    /// a closed letter: (cells made before it, its word's length with the mark)
    closed: Vec<(u32, u32)>,
}

thread_local! {
    static LOG: RefCell<Option<Log>> = const { RefCell::new(None) };
}

/// (the kernel's hook) a cell made
#[inline(always)]
pub(crate) fn cell(ra: u64, rb: u64, la: u64, lb: u64) {
    LOG.with(|l| {
        if let Some(log) = l.borrow_mut().as_mut() {
            log.cells.push([ra as u32, rb as u32, la as u32, lb as u32]);
        }
    });
}

/// (the kernel's hook) a letter closed, its word of length `len` (with the mark)
#[inline(always)]
pub(crate) fn closed(len: u64) {
    LOG.with(|l| {
        if let Some(log) = l.borrow_mut().as_mut() {
            let at = log.cells.len() as u32;
            log.closed.push((at, len as u32));
        }
    });
}

/// The kernel's log of the inductions `f` runs on this thread
fn logged<T>(f: impl FnOnce() -> T) -> (T, Log) {
    LOG.with(|l| *l.borrow_mut() = Some(Log::default()));
    let t = f();
    let log = LOG.with(|l| l.borrow_mut().take()).unwrap_or_default();
    (t, log)
}

/// A step's kind
pub const EQUAL: u32 = 0;
/// A wins: B ends
pub const TOP_WINS: u32 = 1;
/// B wins: A ends
pub const BOTTOM_WINS: u32 = 2;

/// A word's kind (`w_kind`, low bits): a single byte, kept as bytes in the arena, a rule
pub const BYTE: u32 = 0;
pub const ARENA: u32 = 1;
pub const RULE: u32 = 2;
/// `w_kind`: the word holds the walk's row
pub const MARKED: u32 = 4;
/// `w_birth` of a block's own letter; `w_death` of the final letter
pub const NONE: u32 = u32::MAX;

/// One block's record (the format `irec/1`; `bench/irec.py` reads it)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Record {
    /// rows
    pub n: u32,
    /// the walk's row
    pub p: u32,
    /// the block's letters
    pub r: u32,
    /// the letter that is the walk's row
    pub row_letter: u32,
    /// the marked letter that closed last: the cycle's
    pub final_letter: u32,
    /// its word's length: the cycle's period
    pub period: u32,
    /// whether the shipped gates (`bwt::LazyCells`) admit the block
    pub admitted: bool,
    /// a letter's byte and λ, in top order
    pub l_byte: Vec<u32>,
    pub l_lam: Vec<u32>,
    /// the bottom order, as letters
    pub l_bottom: Vec<u32>,
    /// a step's top and bottom front letters, their λ before it, and its kind
    pub s_a: Vec<u32>,
    pub s_b: Vec<u32>,
    pub s_ra: Vec<u32>,
    pub s_rb: Vec<u32>,
    pub s_type: Vec<u32>,
    /// a closed letter, after how many steps
    pub c_step: Vec<u32>,
    pub c_letter: Vec<u32>,
    /// each letter's word: its length in bytes, its kind, the cells it is a child of, the
    /// step its letter was made at and the one it ended or closed at
    pub w_len: Vec<u32>,
    pub w_kind: Vec<u32>,
    pub w_uses: Vec<u32>,
    pub w_birth: Vec<u32>,
    pub w_death: Vec<u32>,
}

impl Record {
    /// The arrays, by name, in the order written
    pub fn arrays(&self) -> [(&'static str, &[u32]); 15] {
        [
            ("l_byte", &self.l_byte),
            ("l_lam", &self.l_lam),
            ("l_bottom", &self.l_bottom),
            ("s_a", &self.s_a),
            ("s_b", &self.s_b),
            ("s_ra", &self.s_ra),
            ("s_rb", &self.s_rb),
            ("s_type", &self.s_type),
            ("c_step", &self.c_step),
            ("c_letter", &self.c_letter),
            ("w_len", &self.w_len),
            ("w_kind", &self.w_kind),
            ("w_uses", &self.w_uses),
            ("w_birth", &self.w_birth),
            ("w_death", &self.w_death),
        ]
    }

    /// The record as `irec/1`: one JSON header line, then the arrays as little-endian u32,
    /// one after another, at the offsets the header gives (from the byte after the line)
    pub fn to_bytes(&self, corpus: &str, block: usize, index: u64) -> Vec<u8> {
        let mut offsets = Vec::new();
        let mut at = 0usize;
        let mut body = Vec::new();
        for (name, a) in self.arrays() {
            offsets.push(format!("\"{name}\":[{at},{}]", a.len()));
            for v in a {
                body.extend_from_slice(&v.to_le_bytes());
            }
            at += 4 * a.len();
        }
        let corpus: String = corpus
            .chars()
            .map(|c| {
                if c == '"' || c == '\\' || c.is_control() {
                    '_'
                } else {
                    c
                }
            })
            .collect();
        let head = format!(
            "{{\"format\":\"irec/1\",\"corpus\":\"{corpus}\",\"block\":{block},\"index\":{index},\"n\":{},\"p\":{},\"r\":{},\"steps\":{},\"closures\":{},\"row_letter\":{},\"final\":{},\"period\":{},\"admitted\":{},\"arrays\":{{{}}}}}\n",
            self.n,
            self.p,
            self.r,
            self.s_a.len(),
            self.c_step.len(),
            self.row_letter,
            self.final_letter,
            self.period,
            self.admitted,
            offsets.join(",")
        );
        let mut v = head.into_bytes();
        v.extend_from_slice(&body);
        v
    }
}

/// The block's letters from its run records, as the kernel's first stage makes them
/// (`slots_lazy`): consecutive records of one byte are one letter, but the walk's row's run,
/// which is cut into what lies before the row, the row, and what lies after. Top order; each
/// (byte, λ, is the row).
fn letters(recs: &[u16], p: u32) -> Result<Vec<(u8, u32, bool)>, String> {
    let mut runs: Vec<(u8, u32, u32)> = Vec::new(); // (byte, first row, λ)
    let mut row = 0u32;
    for &rec in recs {
        let (c, len) = ((rec >> 8) as u8, (rec & 0xFF) as u32);
        if len == 0 {
            return Err("a record of no rows".into());
        }
        match runs.last_mut() {
            Some(last) if last.0 == c => last.2 += len,
            _ => runs.push((c, row, len)),
        }
        row += len;
    }
    if p >= row {
        return Err(format!("the walk's row {p} is past the block's {row} rows"));
    }
    let mut out = Vec::with_capacity(runs.len() + 2);
    for (c, first, lam) in runs {
        if (first..first + lam).contains(&p) {
            let off = p - first;
            if off > 0 {
                out.push((c, off, false));
            }
            out.push((c, 1, true));
            if lam - off > 1 {
                out.push((c, lam - off - 1, false));
            }
        } else {
            out.push((c, lam, false));
        }
    }
    Ok(out)
}

/// A doubly linked order over letter ids
struct Order {
    prev: Vec<u32>,
    next: Vec<u32>,
    tail: u32,
}

impl Order {
    fn new(cap: usize, seq: &[u32]) -> Order {
        let mut o = Order {
            prev: vec![NONE; cap],
            next: vec![NONE; cap],
            tail: NONE,
        };
        for w in seq.windows(2) {
            o.next[w[0] as usize] = w[1];
            o.prev[w[1] as usize] = w[0];
        }
        o.tail = seq.last().copied().unwrap_or(NONE);
        o
    }
    /// `x` put right after `at`
    fn insert_after(&mut self, at: u32, x: u32) {
        let nx = self.next[at as usize];
        self.next[at as usize] = x;
        self.prev[x as usize] = at;
        self.next[x as usize] = nx;
        if nx == NONE {
            self.tail = x;
        } else {
            self.prev[nx as usize] = x;
        }
    }
    fn remove(&mut self, x: u32) {
        let (p, nx) = (self.prev[x as usize], self.next[x as usize]);
        if p != NONE {
            self.next[p as usize] = nx;
        }
        if nx != NONE {
            self.prev[nx as usize] = p;
        } else {
            self.tail = p;
        }
        self.prev[x as usize] = NONE;
        self.next[x as usize] = NONE;
    }
}

/// The record of one block: the shipped lazy kernel's induction, run with open admission
/// (`bwt::ForcedLazyCells`) and logged, the replay's letters and steps checked against the
/// log, and the replay's final word against the kernel's output. Err: the block has no
/// induction (its symbols do not tile, or the kernel refused it), or a check failed.
pub fn record(b: &Symbols, k: Kernels) -> Result<Record, String> {
    let mut runs = Runs::default();
    let n = front(b, &mut runs, k).map_err(|e| format!("the front: {e:?}"))?;
    let p = b.orig_ptr;
    if p as usize >= n {
        return Err("the walk's row is past the block".into());
    }
    // the shipped gates, not logged
    let mut shipped = LazyCells::default();
    let mut scratch = Vec::new();
    let before = shipped.counts().admitted;
    shipped.invert_runs(&runs, n, p, &mut scratch, k);
    let admitted = shipped.counts().admitted > before;
    // the forced kernel, logged
    let mut forced = ForcedLazyCells::default();
    let mut out = Vec::new();
    let before = forced.counts().admitted;
    let (_, log) = logged(|| forced.invert_runs(&runs, n, p, &mut out, k));
    if forced.counts().admitted == before {
        return Err("the forced lazy kernel refused the block".into());
    }
    out.truncate(n);
    let rec = replay(runs.records(), n as u32, p, &log)?;
    check_output(&rec, &out)?;
    Ok(Record { admitted, ..rec })
}

/// The induction on the block's letters, step for step against the kernel's log
fn replay(recs: &[u16], n: u32, p: u32, log: &Log) -> Result<Record, String> {
    let ls = letters(recs, p)?;
    let r = ls.len();
    let steps = log.cells.len();
    let cap = r + steps;
    let mut lam = vec![0u32; cap];
    // a word's length with the mark, as the kernel keeps it
    let mut wl = vec![0u32; cap];
    let mut rec = Record {
        n,
        p,
        r: r as u32,
        row_letter: NONE,
        final_letter: NONE,
        w_birth: vec![NONE; cap],
        w_death: vec![NONE; cap],
        w_uses: vec![0; cap],
        w_kind: vec![0; cap],
        ..Record::default()
    };
    for (i, &(c, l, row)) in ls.iter().enumerate() {
        lam[i] = l;
        wl[i] = 1 | if row { MARK } else { 0 };
        rec.l_byte.push(c as u32);
        rec.l_lam.push(l);
        rec.w_kind[i] = BYTE | if row { MARKED } else { 0 };
        if row {
            rec.row_letter = i as u32;
        }
    }
    // the bottom order: sorted stably by byte
    let mut bottom: Vec<u32> = (0..r as u32).collect();
    bottom.sort_by_key(|&i| ls[i as usize].0);
    let top: Vec<u32> = (0..r as u32).collect();
    let mut t = Order::new(cap, &top);
    let mut bo = Order::new(cap, &bottom);
    rec.l_bottom = bottom;
    let mut ci = 0usize; // the kernel's closures read
    let mut whole = true;
    loop {
        let (a, b) = (t.tail, bo.tail);
        if a == NONE || b == NONE {
            return Err("an order ran out before the marked letter closed".into());
        }
        if whole && a == b {
            // one letter last in both orders: its cycle closes
            let s = rec.s_a.len() as u32;
            let got = log.closed.get(ci).copied();
            if got != Some((s, wl[a as usize])) {
                return Err(format!(
                    "closure {ci}: the replay closes letter {a} (length {:#x}) after {s} steps, the kernel logged {got:?}",
                    wl[a as usize]
                ));
            }
            ci += 1;
            rec.c_step.push(s);
            rec.c_letter.push(a);
            t.remove(a);
            bo.remove(a);
            if wl[a as usize] & MARK != 0 {
                rec.final_letter = a;
                rec.period = wl[a as usize] & !MARK;
                break;
            }
            rec.w_death[a as usize] = s;
            continue;
        }
        if a == b {
            return Err(format!(
                "letter {a} is last in both orders in the middle of a step's run"
            ));
        }
        let i = rec.s_a.len();
        let Some(&want) = log.cells.get(i) else {
            return Err(format!(
                "the replay makes cell {i}; the kernel made {steps}"
            ));
        };
        let (ra, rb) = (lam[a as usize], lam[b as usize]);
        let (la, lb) = (wl[a as usize], wl[b as usize]);
        if want != [ra, rb, la, lb] {
            return Err(format!(
                "step {i}: the replay has (ra, rb, la, lb) = {:?}, the kernel {want:?} (letters {a} and {b})",
                [ra, rb, la, lb]
            ));
        }
        let w = ra.min(rb);
        let c = (r + i) as u32;
        lam[c as usize] = w;
        let len = la.wrapping_add(lb);
        wl[c as usize] = len;
        let marked = len & MARK != 0;
        rec.w_kind[c as usize] =
            if !marked && len <= SMALL { ARENA } else { RULE } | if marked { MARKED } else { 0 };
        rec.w_birth[c as usize] = i as u32;
        rec.w_uses[a as usize] += 1;
        rec.w_uses[b as usize] += 1;
        rec.s_a.push(a);
        rec.s_b.push(b);
        rec.s_ra.push(ra);
        rec.s_rb.push(rb);
        rec.s_type.push(match ra.cmp(&rb) {
            std::cmp::Ordering::Equal => EQUAL,
            std::cmp::Ordering::Greater => TOP_WINS,
            std::cmp::Ordering::Less => BOTTOM_WINS,
        });
        // the cell: on top at the right end of B's place, on the bottom of A's
        t.insert_after(b, c);
        bo.insert_after(a, c);
        lam[a as usize] -= w;
        lam[b as usize] -= w;
        for x in [a, b] {
            if lam[x as usize] == 0 {
                t.remove(x);
                bo.remove(x);
                rec.w_death[x as usize] = i as u32;
            }
        }
        // the kernel reads both letters whole again (and so looks for a closed one) only
        // after a step that ends both
        whole = ra == rb;
    }
    if ci != log.closed.len() || rec.s_a.len() != steps {
        return Err(format!(
            "the replay closed {ci} letters in {} steps, the kernel {} in {steps}",
            rec.s_a.len(),
            log.closed.len()
        ));
    }
    rec.w_len = wl.iter().map(|&x| x & !MARK).collect();
    Ok(rec)
}

/// The final word, expanded, against the kernel's output: the block starts `SHIFT` bytes
/// after the walk's row's byte in the cycle's word, and is that word's copies if the cycle
/// is shorter than the block
fn check_output(rec: &Record, out: &[u8]) -> Result<(), String> {
    let r = rec.r as usize;
    let n = rec.n as usize;
    if out.len() != n {
        return Err(format!("the kernel wrote {} bytes of {n}", out.len()));
    }
    // the cycle's word, left to right, and where the walk's row's byte lies in it
    let mut word = Vec::with_capacity(rec.period as usize);
    let mut at_row = None;
    let mut stack = vec![rec.final_letter];
    while let Some(x) = stack.pop() {
        let x = x as usize;
        if x < r {
            if x as u32 == rec.row_letter {
                at_row = Some(word.len());
            }
            word.push(rec.l_byte[x] as u8);
        } else {
            let i = x - r;
            // A's word then B's: B pushed first
            stack.push(rec.s_b[i]);
            stack.push(rec.s_a[i]);
        }
    }
    let period = word.len();
    if period != rec.period as usize || period == 0 || n % period != 0 {
        return Err(format!(
            "the cycle's word has {period} bytes; its letter says {}, the block {n}",
            rec.period
        ));
    }
    let q = at_row.ok_or("the walk's row's byte is not in the cycle's word")?;
    for (j, &byte) in out.iter().enumerate() {
        if word[(q + SHIFT + j) % period] != byte {
            return Err(format!(
                "byte {j}: the replay's word gives {:#04x}, the kernel wrote {byte:#04x}",
                word[(q + SHIFT + j) % period]
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A column's run records (`runs::front`'s format: `byte << 8 | rows`, at most 127 a record)
    fn runs_of(col: &[u8]) -> Runs {
        let mut runs = Runs::default();
        let recs = runs.records_mut();
        recs.clear();
        for &c in col {
            match recs.last_mut() {
                Some(last) if *last >> 8 == c as u16 && *last & 0xFF < 127 => *last += 1,
                _ => recs.push((c as u16) << 8 | 1),
            }
        }
        runs
    }

    /// The forced lazy kernel on a column, logged: its output and log, if it inverted it
    fn induce(runs: &Runs, n: usize, p: u32) -> Option<(Vec<u8>, Log)> {
        let k = Kernels::detect();
        let mut forced = ForcedLazyCells::default();
        let mut out = Vec::new();
        let (_, log) = logged(|| forced.invert_runs(runs, n, p, &mut out, k));
        (forced.counts().admitted == 1).then(|| {
            out.truncate(n);
            (out, log)
        })
    }

    struct Lcg(u64);
    impl Lcg {
        fn below(&mut self, m: u64) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 33) % m
        }
    }

    /// Arbitrary columns, no BWT among most of them: several cycles, so letters close before
    /// the marked one. The replay must take the kernel's steps and closures, every one, and
    /// where the marked cycle's copies fill the column, its word must give the kernel's bytes
    #[test]
    fn arbitrary_columns_replay_the_kernel() {
        let mut rng = Lcg(0x5eed);
        let (mut cases, mut early, mut outputs) = (0, 0, 0);
        for _ in 0..20_000 {
            let n = 2 + rng.below(14) as usize;
            let sigma = 2 + rng.below(3);
            let col: Vec<u8> = (0..n).map(|_| b'a' + rng.below(sigma) as u8).collect();
            let p = rng.below(n as u64) as u32;
            let runs = runs_of(&col);
            let Some((out, log)) = induce(&runs, n, p) else {
                continue;
            };
            let rec = replay(runs.records(), n as u32, p, &log)
                .unwrap_or_else(|e| panic!("{:?} p {p}: {e}", std::str::from_utf8(&col)));
            cases += 1;
            early += rec.c_step.len() - 1;
            if rec.period > 0 && n as u32 % rec.period == 0 {
                check_output(&rec, &out)
                    .unwrap_or_else(|e| panic!("{:?} p {p}: {e}", std::str::from_utf8(&col)));
                outputs += 1;
            }
        }
        assert!(
            cases > 10_000 && outputs > 1000,
            "{cases} cases, {outputs} outputs"
        );
        assert!(early > 1000, "{early} letters closed before the marked one");
    }

    /// A log that differs from the kernel's anywhere, or an output that differs by a byte,
    /// is refused
    #[test]
    fn a_record_that_differs_is_refused() {
        let col = b"bbaacbaccabbcabacbcaaabc";
        let (n, p) = (col.len(), 5);
        let runs = runs_of(col);
        let (out, log) = induce(&runs, n, p).expect("inverted");
        let rec = replay(runs.records(), n as u32, p, &log).expect("the true log");
        assert!(rec.s_a.len() > 4);
        let fresh = || Log {
            cells: log.cells.clone(),
            closed: log.closed.clone(),
        };
        for i in 0..log.cells.len() {
            for f in 0..4 {
                let mut bad = fresh();
                bad.cells[i][f] ^= 1;
                assert!(
                    replay(runs.records(), n as u32, p, &bad).is_err(),
                    "cell {i} field {f}"
                );
            }
        }
        let mut bad = fresh();
        bad.cells.pop();
        assert!(
            replay(runs.records(), n as u32, p, &bad).is_err(),
            "a cell short"
        );
        let mut bad = fresh();
        bad.cells.push([1, 1, 1, 1]);
        assert!(
            replay(runs.records(), n as u32, p, &bad).is_err(),
            "a cell more"
        );
        let mut bad = fresh();
        bad.closed.insert(0, (0, 1));
        assert!(
            replay(runs.records(), n as u32, p, &bad).is_err(),
            "a closure more"
        );
        let mut bad = fresh();
        bad.closed.pop();
        assert!(
            replay(runs.records(), n as u32, p, &bad).is_err(),
            "the final closure missing"
        );
        // a different walk's row
        assert!(replay(runs.records(), n as u32, (p + 1) % n as u32, &log).is_err());
        if n as u32 % rec.period == 0 {
            check_output(&rec, &out).expect("the true output");
            for j in 0..n {
                let mut wrong = out.clone();
                wrong[j] ^= 1;
                assert!(check_output(&rec, &wrong).is_err(), "byte {j}");
            }
        }
    }

    /// The letters, as the kernel's first stage makes them: one a run, the walk's row's run
    /// cut around the row
    #[test]
    fn letters_cut_the_walks_run() {
        let recs = [0x6103u16, 0x6202, 0x6201, 0x6104];
        // rows: aaa bb b aaaa → runs aaa (0..3), bbb (3..6), aaaa (6..10)
        assert_eq!(
            letters(&recs, 4).unwrap(),
            vec![
                (b'a', 3, false),
                (b'b', 1, false),
                (b'b', 1, true),
                (b'b', 1, false),
                (b'a', 4, false)
            ]
        );
        assert_eq!(
            letters(&recs, 0).unwrap(),
            vec![
                (b'a', 1, true),
                (b'a', 2, false),
                (b'b', 3, false),
                (b'a', 4, false)
            ]
        );
        assert_eq!(
            letters(&recs, 9).unwrap(),
            vec![
                (b'a', 3, false),
                (b'b', 3, false),
                (b'a', 3, false),
                (b'a', 1, true)
            ]
        );
        assert!(letters(&recs, 10).is_err());
    }
}
