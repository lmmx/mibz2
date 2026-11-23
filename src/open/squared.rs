//! Research only (`squared-trace`): the T² two-ended emit arms of MONODROMY record 93
//! (`docs/journal/monodromy/2026-11-23-squared-trace-arm.md` at c3dab1ef, its addendum at
//! b36ec511), for the bench-emit-cost-sq diagnostic
//! (`docs/journal/2026-11-23-emit-cost-sq-registration.md`). They run only on the blocks the
//! open kernel already emits from both ends (`QUEUE && !wide && m >= 65,536`), into the same
//! `out` with the same lo/hi layout as `emit_bidirectional`, and write the same bytes. Without
//! the feature none of this is compiled and the kernel is as shipped.
//!
//! **The tables.** `edges[j] = code(j) << bits | (T(j) ^ T⁻¹(j))` as wired today. The T² arms
//! also record `succ[u] = T(u)` in `wire_xor` (construction), then build
//! `x2[j] = T²(j) ^ T⁻²(j)`: zero `x2`, then for each j, v₂ = succ[succ[j]], x2[j] ^= v₂,
//! x2[v₂] ^= j. Peak scratch grows by `succ` and `x2`, 8m bytes.
//!
//! **The T² walk** (`sq_rounds`). The front holds (f, h = T⁻¹f, f₋₂ = T⁻²f) and the back
//! (b, k = Tb, b₊₂ = T²b). A round reads g = T(f) = (edges[f] & mask) ^ h and
//! g′ = T⁻¹(b) = (edges[b] & mask) ^ k, emits code(f), code(g) at lo and code(b), code(g′)
//! ending at hi (each word copied from `inline16` or `spelling` exactly as `emit_bidirectional`
//! copies it), then steps f to x2[f] ^ f₋₂ = T²f and b to x2[b] ^ b₊₂ = T⁻²b. The dependent
//! chain is one load an end a round, `x2[f]` and `x2[b]`; those two loads come first in the
//! round's program order, so that the next round's can issue while this round's copies wait
//! (with them last, the round's ~100 instructions kept the next chain load out of the
//! out-of-order window: twice base's emit time on the 2 MiB-L2 container, against about base's
//! with them first).
//!
//! **The hand-back.** Before each round the four rows are checked: with R rows left between
//! the ends (front at cycle position s, back at L − 1 − s, R = L − 2s), R ≥ 4 makes f, g, g′, b
//! four distinct rows, and each R < 4 shows as a coincidence: R = 3 as g = g′, R = 2 as g = b
//! (and f = g′), R = 1 as f = b, and R = 0 (the ends have crossed, a step taken) as f = k. On
//! any of them, or when fewer than four 16-byte words of room are left (`hi − lo < 64`), the
//! exact state (prev, front) = (h, f), (back, next) = (b, k) goes to `single_steps`, a copy of
//! `emit_bidirectional`'s single-step loop that resumes from a state, which finishes the cycle,
//! and `finish` does `emit_bidirectional`'s `copy_within` and `repeat`. That covers odd cycle
//! lengths, short (periodic) cycles and the meeting point. The block's CRC checks every byte.
//!
//! **The shadow arm** (record 93 §5, the addendum's batch of 4): the base two-ended walk runs
//! (`single_steps`' step), and after each of its steps (one row from each end) 4 iterations of
//! the square pass run, in order: first the zero-fill, one iteration a 64-byte line of `x2`
//! (16 entries), then the scatter, one iteration a row j. Once the pass is complete the walk
//! switches to `sq_rounds` from its exact current state; `x2` is read only then. If the walk
//! ends first, the pass is abandoned. Its whole cost lands in `emit_ns`.
//!
//! **Safety:** no `unsafe`, as in the rest of `open`: every index is bounds-checked, and
//! every loop is bounded by the table's length, whatever the construction (a wrong one gives a
//! wrong block, which the CRC rejects, or a bounds panic, as the shipped emitters do).
use super::{long_word, repeat, INLINE, LONG};
use crate::emit_cost::Arm;

/// The shadow arm's square-pass iterations after each base step (record 93's addendum at
/// b36ec511: fixed at 4, never tuned after the data)
pub(super) const SHADOW_BATCH: usize = 4;
/// `x2` entries a zero-fill iteration clears: one 64-byte line
const ZERO_LINE: usize = 16;

/// The arm's state in a worker's kernel, reused between blocks as `edges` is
pub(super) struct Scratch {
    /// the arm of the block in hand (`Kernel::recorded` sets it at the block's start)
    pub(super) arm: Arm,
    /// T(u), recorded in construction (the T² arms)
    succ: Vec<u32>,
    /// T²(j) ^ T⁻²(j)
    x2: Vec<u32>,
    /// the block's T² walk took at least one round
    pub(super) rounds: bool,
}

impl Default for Scratch {
    fn default() -> Self {
        Scratch {
            arm: Arm::Base,
            succ: Vec::new(),
            x2: Vec::new(),
            rounds: false,
        }
    }
}

impl Scratch {
    /// A block's start under `arm`
    pub(super) fn begin(&mut self, arm: Arm) {
        self.arm = arm;
        self.rounds = false;
    }

    /// `succ` for `m` rows in the T² arms (every entry is written by `wire_xor_succ`: the
    /// bands partition the retained ranks), `None` in `Base`
    pub(super) fn succ_for(&mut self, m: usize) -> Option<&mut [u32]> {
        if self.arm == Arm::Base {
            return None;
        }
        grow(&mut self.succ, m);
        Some(&mut self.succ[..m])
    }

    /// The arm's scratch held, bytes (capacity)
    pub(super) fn bytes(&self) -> u64 {
        4 * (self.succ.capacity() + self.x2.capacity()) as u64
    }

    /// The serial square pass (record 93 §2(b)): `x2[..m]` = T² ^ T⁻² from `succ[..m]`
    pub(super) fn square(&mut self, m: usize) {
        grow(&mut self.x2, m);
        self.x2[..m].fill(0);
        let succ = &self.succ[..m];
        let x2 = &mut self.x2[..m];
        for j in 0..m {
            let v2 = succ[succ[j] as usize];
            x2[j] ^= v2;
            x2[v2 as usize] ^= j as u32;
        }
    }

    /// The T² arms' two-ended emit of a block's marked cycle from `mark` (its predecessor
    /// `before`): `emit_bidirectional`'s output, by `sq_rounds` and the hand-back
    pub(super) fn emit(
        &mut self,
        t: &Tables<'_>,
        n: usize,
        mark: u32,
        before: u32,
        out: &mut Vec<u8>,
    ) {
        out.resize(n + INLINE, 0);
        let m = t.edges.len();
        let mut e = Ends {
            prev: before,
            front: mark,
            back: before,
            next: mark,
            lo: 0,
            hi: n,
        };
        match self.arm {
            Arm::Sq => {
                let (started, rounds) = sq_rounds(t, &self.x2[..m], out, &mut e, false);
                self.rounds = rounds;
                single_steps(t, out, &mut e, started);
            }
            Arm::SqShadow => {
                grow(&mut self.x2, m);
                let x2 = &mut self.x2[..m];
                let succ = &self.succ[..m];
                let mut pass = SquarePass::new(m);
                let (started, ended) = shadow_steps(t, succ, x2, &mut pass, out, &mut e);
                if !ended && pass.complete() {
                    let (started, rounds) = sq_rounds(t, x2, out, &mut e, started);
                    self.rounds = rounds;
                    single_steps(t, out, &mut e, started);
                }
            }
            Arm::Base => single_steps(t, out, &mut e, false),
        }
        finish(n, out, &e);
    }
}

/// `v` to at least `m` entries, its capacity grown exactly (the memory diet's rule for
/// `edges`: the peak scratch is 4m bytes a table, not doubled)
fn grow(v: &mut Vec<u32>, m: usize) {
    if v.len() < m {
        v.reserve_exact(m - v.len());
        v.resize(m, 0);
    }
}

/// The tables the emit reads, as `emit_bidirectional` takes them
pub(super) struct Tables<'a> {
    pub(super) edges: &'a [u32],
    pub(super) spelling: &'a [u8],
    pub(super) inline: &'a [[u8; INLINE]],
    pub(super) lens: &'a [u8],
    pub(super) bits: u32,
}

/// The two ends' exact state, in `emit_bidirectional`'s terms
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ends {
    prev: u32,
    front: u32,
    back: u32,
    next: u32,
    lo: usize,
    hi: usize,
}

/// The word of `code` at `lo`: `emit_bidirectional`'s front copy, unchanged
#[inline(always)]
fn put_front(t: &Tables<'_>, out: &mut [u8], e: &mut Ends, cf: usize) {
    let lf = t.lens[cf];
    if lf != LONG && e.hi - e.lo >= INLINE {
        out[e.lo..e.lo + INLINE].copy_from_slice(&t.inline[cf]);
        e.lo += usize::from(lf);
    } else {
        let word = if lf == LONG {
            long_word(&t.inline[cf], t.spelling)
        } else {
            &t.inline[cf][..usize::from(lf)]
        };
        let take = word.len().min(e.hi - e.lo);
        out[e.lo..e.lo + take].copy_from_slice(&word[..take]);
        e.lo += take;
    }
}

/// The word of `code` ending at `hi`: `emit_bidirectional`'s back copy, unchanged
#[inline(always)]
fn put_back(t: &Tables<'_>, out: &mut [u8], e: &mut Ends, cb: usize) {
    let lb = t.lens[cb];
    if lb != LONG && e.hi - e.lo >= INLINE {
        let shifted = u128::from_le_bytes(t.inline[cb]) << (8 * (INLINE - usize::from(lb)));
        out[e.hi - INLINE..e.hi].copy_from_slice(&shifted.to_le_bytes());
        e.hi -= usize::from(lb);
    } else {
        let word = if lb == LONG {
            long_word(&t.inline[cb], t.spelling)
        } else {
            &t.inline[cb][..usize::from(lb)]
        };
        let take = word.len().min(e.hi - e.lo);
        out[e.hi - take..e.hi].copy_from_slice(&word[word.len() - take..]);
        e.hi -= take;
    }
}

/// One step of `emit_bidirectional`'s loop from `e`: `false` once the walk is over (its
/// checks, with `started` its `pair != 0`), the state left exactly where that loop leaves it
#[inline(always)]
fn step(t: &Tables<'_>, out: &mut [u8], e: &mut Ends, started: bool) -> bool {
    let mask = (1 << t.bits) - 1;
    if e.lo == e.hi || (started && e.front == e.next) {
        return false;
    }
    let ef = t.edges[e.front as usize];
    let single = e.front == e.back;
    put_front(t, out, e, (ef >> t.bits) as usize);
    if single {
        return false;
    }
    let eb = t.edges[e.back as usize];
    put_back(t, out, e, (eb >> t.bits) as usize);
    (e.prev, e.front) = (e.front, (ef & mask) ^ e.prev);
    (e.next, e.back) = (e.back, (eb & mask) ^ e.next);
    true
}

/// `emit_bidirectional`'s single-step loop, resumed from `e` (`started`: a step has been taken
/// on this block, its `pair != 0`), bounded as that loop is
fn single_steps(t: &Tables<'_>, out: &mut [u8], e: &mut Ends, mut started: bool) {
    for _ in 0..=t.edges.len() / 2 {
        if !step(t, out, e, started) {
            break;
        }
        started = true;
    }
}

/// `emit_bidirectional`'s ending: a short cycle's suffix moved beside its prefix and repeated
fn finish(n: usize, out: &mut Vec<u8>, e: &Ends) {
    if e.lo < e.hi {
        out.copy_within(e.hi..n, e.lo);
        out.truncate(e.lo + (n - e.hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

/// The T² rounds from `e` (record 93 §2(c)), until the hand-back's conditions: (`started` for
/// the hand-back, a round ran)
fn sq_rounds(
    t: &Tables<'_>,
    x2: &[u32],
    out: &mut [u8],
    e: &mut Ends,
    mut started: bool,
) -> (bool, bool) {
    let mask = (1 << t.bits) - 1;
    let edges = t.edges;
    let (mut f, mut h, mut b, mut k) = (e.front, e.prev, e.back, e.next);
    // f₋₂ = T⁻¹(h), as T(h) = f; b₊₂ = T(k), as T⁻¹(k) = b
    let mut f2 = (edges[h as usize] & mask) ^ f;
    let mut b2 = (edges[k as usize] & mask) ^ b;
    let mut ran = false;
    for _ in 0..=edges.len() / 4 {
        if e.hi - e.lo < 4 * INLINE || (started && f == k) {
            break;
        }
        // the dependent chain's loads first, so that the next round's can issue before this
        // round's copies retire (they are the critical path: one load an end a round)
        let nf = x2[f as usize] ^ f2;
        let nb = x2[b as usize] ^ b2;
        let ef = edges[f as usize];
        let eb = edges[b as usize];
        let g = (ef & mask) ^ h;
        let g1 = (eb & mask) ^ k;
        if f == b || g == b || g == g1 || f == g1 {
            break;
        }
        let eg = edges[g as usize];
        let eg1 = edges[g1 as usize];
        put_front(t, out, e, (ef >> t.bits) as usize);
        put_front(t, out, e, (eg >> t.bits) as usize);
        put_back(t, out, e, (eb >> t.bits) as usize);
        put_back(t, out, e, (eg1 >> t.bits) as usize);
        (f2, f, h) = (f, nf, g);
        (b2, b, k) = (b, nb, g1);
        started = true;
        ran = true;
    }
    (e.front, e.prev, e.back, e.next) = (f, h, b, k);
    (started, ran)
}

/// The square pass in iterations (the shadow arm's): the zero-fill, a line an iteration, then
/// the scatter, a row an iteration
struct SquarePass {
    it: usize,
    lines: usize,
    m: usize,
}

impl SquarePass {
    fn new(m: usize) -> Self {
        SquarePass {
            it: 0,
            lines: m.div_ceil(ZERO_LINE),
            m,
        }
    }

    fn complete(&self) -> bool {
        self.it == self.lines + self.m
    }

    /// The next `SHADOW_BATCH` iterations (fewer at the end)
    #[inline(always)]
    fn run(&mut self, succ: &[u32], x2: &mut [u32]) {
        for _ in 0..SHADOW_BATCH {
            if self.it < self.lines {
                let a = ZERO_LINE * self.it;
                x2[a..(a + ZERO_LINE).min(self.m)].fill(0);
            } else if self.it < self.lines + self.m {
                let j = self.it - self.lines;
                let v2 = succ[succ[j] as usize];
                x2[j] ^= v2;
                x2[v2 as usize] ^= j as u32;
            } else {
                return;
            }
            self.it += 1;
        }
    }
}

/// The shadow arm's first stretch: base steps, each followed by `SHADOW_BATCH` iterations of
/// the pass, until the pass is complete or the walk ends (`started`, the walk ended)
fn shadow_steps(
    t: &Tables<'_>,
    succ: &[u32],
    x2: &mut [u32],
    pass: &mut SquarePass,
    out: &mut [u8],
    e: &mut Ends,
) -> (bool, bool) {
    let mut started = false;
    for _ in 0..=t.edges.len() / 2 {
        if pass.complete() {
            return (started, false);
        }
        if !step(t, out, e, started) {
            return (started, true);
        }
        started = true;
        pass.run(succ, x2);
    }
    (started, true)
}

#[cfg(test)]
mod tests {
    use super::super::b5::{plain, runs_column, Lcg};
    use super::super::*;
    use super::*;

    /// Column `l` at row `p` by the shipped instance under `arm`, into a poisoned output
    fn under(k: &mut Kernel, arm: Arm, l: &[u8], p: u32) -> (Vec<u8>, bool, bool) {
        let r = Runs::of_column(l);
        let mut out: Vec<u8> = plain(l, p).iter().map(|b| !b).collect();
        k.recorded::<true, true>(arm, r.records(), l.len(), p, LIMIT, Cap::NONE, &mut out)
            .expect("no cap");
        (out, k.predecessor.is_some(), k.sq.rounds)
    }

    /// Every arm writes the base arm's bytes, the plain inverse's; returns (two-ended, T²
    /// rounds ran in `sq`, in `sq-shadow`)
    fn check(k: &mut Kernel, l: &[u8], p: u32, what: &str) -> (bool, bool, bool) {
        let want = plain(l, p);
        let (base, bidi, _) = under(k, Arm::Base, l, p);
        assert!(base == want, "{what}: base");
        let (sq, bidi_sq, r_sq) = under(k, Arm::Sq, l, p);
        assert!(sq == want, "{what}: sq");
        let (sh, bidi_sh, r_sh) = under(k, Arm::SqShadow, l, p);
        assert!(sh == want, "{what}: sq-shadow");
        assert_eq!((bidi, bidi), (bidi_sq, bidi_sh), "{what}: two-ended");
        (bidi, r_sq, r_sh)
    }

    /// Run-heavy columns past and below 65,536 retained rows, at many rows `p`: arbitrary
    /// columns, so the marked cycle's length is anything from 1 to the block's (odd, even,
    /// short: the repeat path), every arm against the plain inverse
    #[test]
    fn every_arm_is_the_plain_inverse() {
        let mut k = Kernel::default();
        let mut g = Lcg(0x5eed_0093);
        let (mut bidi, mut sq, mut sh) = (0, 0, 0);
        for case in 0..40 {
            let n = [70_000, 150_000, 300_000, 900_000][case % 4];
            let mean = [1, 2, 3, 8][(case / 4) % 4];
            let alphabet = 2 + g.below(60);
            let l = runs_column(&mut g, n, alphabet, mean);
            let p = g.below(n as u32);
            let (b, r1, r2) = check(&mut k, &l, p, &format!("case {case} n {n} p {p}"));
            bidi += usize::from(b);
            sq += usize::from(r1);
            sh += usize::from(r2);
        }
        assert!(bidi >= 20 && sq >= 10 && sh >= 5, "{bidi} {sq} {sh}");
    }

    /// m around 65,536: the smallest two-ended blocks and the largest forward ones
    #[test]
    fn around_the_two_ended_line() {
        let mut k = Kernel::default();
        let mut g = Lcg(0x6553_6000);
        let (mut below, mut above) = (0, 0);
        for n in (64_000..140_000).step_by(3_701) {
            let l: Vec<u8> = (0..n).map(|_| g.below(200) as u8).collect();
            let p = g.below(n as u32);
            let (b, _, _) = check(&mut k, &l, p, &format!("n {n}"));
            if b {
                above += 1;
            } else {
                below += 1;
            }
        }
        assert!(below > 0 && above > 0, "{below} {above}");
    }

    /// A block that is one long cycle (a real BWT, of a random text), and its periodic
    /// powers (each byte of the column repeated: the k-th power's BWT), so the T² walk runs to
    /// the meeting point, and short cycles take `copy_within` and `repeat`
    #[test]
    fn long_cycles_periodic_blocks_and_the_meeting_point() {
        let mut k = Kernel::default();
        let mut g = Lcg(0xb10c_cafe);
        let mut ran = 0;
        for (i, n) in [66_000usize, 99_991, 131_072, 200_003]
            .into_iter()
            .enumerate()
        {
            let text: Vec<u8> = (0..n).map(|_| b'a' + g.below(26) as u8).collect();
            let (l, p) = bwt(&text);
            assert_eq!(plain(&l, p), text);
            let (b, r1, r2) = check(&mut k, &l, p, &format!("bwt n {n}"));
            assert!(b && r1 && r2, "bwt n {n}: {b} {r1} {r2}");
            ran += 1;
            // the text repeated 2..5 times: each byte of the column repeated
            let reps = (2 + i).min(900_000 / n);
            let lp: Vec<u8> = l
                .iter()
                .flat_map(|&c| std::iter::repeat_n(c, reps))
                .collect();
            let pp = p * reps as u32;
            check(&mut k, &lp, pp, &format!("bwt n {n} x{reps}"));
        }
        assert_eq!(ran, 4);
    }

    /// Synthetic tables: one marked cycle of every length L from 1 to 70 (and some long ones)
    /// among m rows, the rest of the rows in other cycles, its words literals, 2- to 16-byte
    /// words and long ones, repeated `reps` times to `n` bytes: each arm's emit is exactly
    /// `emit_bidirectional`'s, so every remainder R = 0..3 at the meeting point and every
    /// hand-back (the coincidences and the last 64 bytes) is taken
    #[test]
    fn the_hand_back_at_every_cycle_length() {
        let mut g = Lcg(0x0dd_c7c1e);
        // the words: 256 literals, then 300 of 2..=16 bytes and 20 of 17..=40 (long)
        let mut spelling = Vec::new();
        let mut offsets = vec![0u32];
        for w in 0..320 {
            let len = if w < 300 { 2 + w % 15 } else { 17 + w % 24 };
            spelling.extend((0..len).map(|_| b'a' + g.below(26) as u8));
            offsets.push(spelling.len() as u32);
        }
        let (mut inline, mut lens) = (Vec::new(), Vec::new());
        inline_words(&spelling, &offsets, &mut inline, &mut lens);
        let codes = lens.len() as u32;
        let mut taken = [0usize; 4];
        let mut lengths: Vec<usize> = (1..=70).collect();
        lengths.extend([127, 128, 129, 130, 1001, 4096, 65_535, 70_001]);
        for (case, &cyc) in lengths.iter().enumerate() {
            let m = 70_000.max(cyc + g.below(3_000) as usize);
            // a random permutation of the rows with the marked cycle `cyc` long
            let mut order: Vec<u32> = (0..m as u32).collect();
            for i in (1..m).rev() {
                order.swap(i, g.below(i as u32 + 1) as usize);
            }
            let mut tnext = vec![0u32; m];
            for i in 0..cyc {
                tnext[order[i] as usize] = order[(i + 1) % cyc];
            }
            for i in cyc..m {
                let next = if i + 1 < m { i + 1 } else { cyc };
                tnext[order[i] as usize] = order[next];
            }
            let mut tprev = vec![0u32; m];
            for (u, &v) in tnext.iter().enumerate() {
                tprev[v as usize] = u as u32;
            }
            let bits = (u32::BITS - (m as u32 - 1).leading_zeros()).max(1);
            let code: Vec<u32> = (0..m)
                .map(|_| match g.below(10) {
                    0..=4 => g.below(256),
                    5..=8 => 256 + g.below(300),
                    _ => 256 + 300 + g.below(codes - 556),
                })
                .collect();
            let edges: Vec<u32> = (0..m)
                .map(|j| code[j] << bits | (tnext[j] ^ tprev[j]))
                .collect();
            let mark = order[0];
            let before = tprev[mark as usize];
            let period: usize = (0..cyc)
                .map(|i| match lens[code[order[i] as usize] as usize] {
                    LONG => {
                        let c = code[order[i] as usize] as usize - 256;
                        (offsets[c + 1] - offsets[c]) as usize
                    }
                    x => usize::from(x),
                })
                .sum();
            let reps = 1 + case % 3;
            let n = period * reps;
            let mut want = vec![0xEE; n + INLINE];
            emit_bidirectional(
                &edges, &spelling, &inline, &lens, n, mark, before, bits, &mut want,
            );
            let t = Tables {
                edges: &edges,
                spelling: &spelling,
                inline: &inline,
                lens: &lens,
                bits,
            };
            let mut s = Scratch::default();
            for arm in [Arm::Sq, Arm::SqShadow] {
                s.begin(arm);
                s.succ_for(m).unwrap().copy_from_slice(&tnext);
                if arm == Arm::Sq {
                    s.square(m);
                }
                let mut got = vec![0x11; n + INLINE];
                s.emit(&t, n, mark, before, &mut got);
                assert!(got == want, "cycle {cyc} m {m} reps {reps} {arm:?}");
                if arm == Arm::Sq && s.rounds {
                    // the remainder R the T² rounds left at the meeting point
                    taken[cyc % 4] += 1;
                }
            }
        }
        // every residue of the cycle's length mod 4 reached the hand-back after T² rounds
        assert!(taken.iter().all(|&c| c > 0), "{taken:?}");
    }

    /// The square pass gives T² ^ T⁻², on a random permutation
    #[test]
    fn square_is_t2_xor_tinv2() {
        let mut g = Lcg(7);
        let m = 10_000;
        let mut perm: Vec<u32> = (0..m as u32).collect();
        for i in (1..m).rev() {
            perm.swap(i, g.below(i as u32 + 1) as usize);
        }
        let mut inv = vec![0u32; m];
        for (i, &v) in perm.iter().enumerate() {
            inv[v as usize] = i as u32;
        }
        let mut s = Scratch::default();
        s.begin(Arm::Sq);
        s.succ_for(m).unwrap().copy_from_slice(&perm);
        s.square(m);
        for j in 0..m {
            let t2 = perm[perm[j] as usize];
            let ti2 = inv[inv[j] as usize];
            assert_eq!(s.x2[j], t2 ^ ti2, "{j}");
        }
        // the shadow's iterations give the same, from a dirty table
        let mut x2 = vec![0xdead_beef; m];
        let mut pass = SquarePass::new(m);
        while !pass.complete() {
            pass.run(&s.succ[..m], &mut x2);
        }
        assert_eq!(x2, s.x2[..m]);
    }

    /// The BWT of `text` with its row `p` (rotations sorted, by prefix doubling)
    pub(super) fn bwt(text: &[u8]) -> (Vec<u8>, u32) {
        let n = text.len();
        let mut rank: Vec<u32> = text.iter().map(|&c| u32::from(c)).collect();
        let mut sa: Vec<u32> = (0..n as u32).collect();
        let mut tmp = vec![0u32; n];
        let mut w = 1;
        loop {
            let key = |i: u32| (rank[i as usize], rank[(i as usize + w) % n]);
            sa.sort_unstable_by_key(|&i| key(i));
            tmp[sa[0] as usize] = 0;
            for x in 1..n {
                tmp[sa[x] as usize] =
                    tmp[sa[x - 1] as usize] + u32::from(key(sa[x - 1]) != key(sa[x]));
            }
            std::mem::swap(&mut rank, &mut tmp);
            if rank[sa[n - 1] as usize] as usize == n - 1 || w >= n {
                break;
            }
            w *= 2;
        }
        let l: Vec<u8> = sa.iter().map(|&i| text[(i as usize + n - 1) % n]).collect();
        let p = sa.iter().position(|&i| i == 0).unwrap() as u32;
        (l, p)
    }
}
