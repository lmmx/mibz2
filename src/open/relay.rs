//! The relay emit: the walk from the marked row in eight lanes of one worker (the Wormhole
//! line's `DietRelay8Cells`, cycles 8–10 on `research/carbon-wormhole`: zero-bit cuts, the
//! relay, post-slop boundary restoration). The serial `emit` follows one chain of dependent
//! loads through `edges`; this one follows eight at once, in the same table, with no staging
//! buffer: every byte is written straight to its place in `out`.
//!
//! **The cuts.** A retained row `j` is a cut when `j` and the marked row agree in their low
//! `shift` bits, `shift = bits - CUT_BITS`: no bit of `edges` marks it, and the marked row is
//! always one. There are 128 to 256 of them on a block of 1,024 rows or more. A cut's
//! segment is the walk from it up to the next cut (exclusive), so the block is the marked
//! row's segment, then the segment of the cut it reaches, and so on back to the marked row.
//! Cuts on another cycle of the retained rows (a periodic or non-BWT column) are scouted and
//! never chained.
//!
//! **The relay (phase 1).** Lane 0 emits from the marked row, the only place whose offset in
//! `out` is known at the start. Lanes 1..8 scout unclaimed cuts meanwhile, summing only their
//! segments' lengths and steps. When lane 0 reaches a cut that is
//! - scouted: its segment becomes a task at lane 0's offset, and lane 0 jumps past it to
//!   the cut it ends at;
//! - being scouted: the scouted part becomes a task, and lane 0 resumes where the scout is
//!   (the scout takes another cut);
//! - unclaimed: lane 0 keeps emitting through it.
//!
//! **The emitters (phase 2).** Eight lanes emit the tasks, each from its cut's row at its
//! known offset. A task's steps are known from its scout, so the lanes run in step with no
//! end checks; the one task cut short by `n` is emitted alone, to `n`.
//!
//! **Post-slop restoration (phase 3).** Every emitter copies a short word as 16 bytes
//! unconditionally (`emit`'s copy), so a piece of `out` written by one lane may be overrun
//! by up to 15 bytes of the lane before it. Each piece's first 16 bytes are rewritten
//! exactly, from its row, at the end: the bytes the walk spells from a row are the same
//! whichever lane wrote them first, so the order does not matter.
//!
//! **Exactly `emit`'s output, or no output.** The relay gives up (returns None, and `emit`
//! walks serially) on any walk the serial one would treat differently: one longer than the
//! serial walk's `m` steps, a cut met twice, scouts walking more than `m` steps in all, or an
//! empty walk. On the kernel's constructions (permutations of the retained rows) none of these
//! happens, and the relay's bytes are the serial walk's.
use super::{INLINE, LONG};

/// Cuts number `2^CUT_BITS` at most (`shift = bits - CUT_BITS`): the cycle's k = 8
const CUT_BITS: u32 = 8;
const MAX_CUTS: usize = 1 << CUT_BITS;
/// The lanes of one worker: the cycle's μ = 8 (12 spilled lane state on x86-64, cycle 9)
const MU: usize = 8;
/// The fewest retained rows the relay takes (fewer walk serially)
pub(super) const RELAY_MIN: usize = 1024;

const FREE: u8 = 0;
const SCOUTING: u8 = 1;
const SCOUTED: u8 = 2;
/// The marked row's cut, a cut lane 0 emits through, or a scouted segment it has placed
const OWNED: u8 = 3;
const IDLE: usize = usize::MAX;

/// The words of the block, as `emit` reads them
#[derive(Clone, Copy)]
pub(super) struct Words<'a> {
    pub spelling: &'a [u8],
    pub offsets: &'a [u32],
    pub inline16: &'a [[u8; INLINE]],
    pub lens: &'a [u8],
}

impl Words<'_> {
    /// The length of `code`'s word
    #[inline(always)]
    fn len(&self, code: usize) -> usize {
        let len = self.lens[code];
        if len != LONG {
            usize::from(len)
        } else {
            let w = code - 256;
            (self.offsets[w + 1] - self.offsets[w]) as usize
        }
    }

    /// `code`'s word at `at`, a short one as 16 bytes (overrunning its end), a long one
    /// clamped to `end`: the new cursor
    #[inline(always)]
    fn put(&self, code: usize, out: &mut [u8], at: usize, end: usize) -> usize {
        let len = self.lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&self.inline16[code]);
            at + usize::from(len)
        } else {
            self.put_long(code, out, at, end)
        }
    }

    /// A long word (the cold path), clamped to `end`
    #[cold]
    #[inline(never)]
    fn put_long(&self, code: usize, out: &mut [u8], at: usize, end: usize) -> usize {
        let w = code - 256;
        let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
        let take = word.len().min(end - at);
        out[at..at + take].copy_from_slice(&word[..take]);
        at + take
    }

    /// `code`'s word at `at`, exactly: at most `rem` bytes. The bytes written
    #[inline]
    fn put_exact(&self, code: usize, out: &mut [u8], at: usize, rem: usize) -> usize {
        let len = self.lens[code];
        let word: &[u8] = if len != LONG {
            &self.inline16[code][..usize::from(len)]
        } else {
            let w = code - 256;
            &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize]
        };
        let take = word.len().min(rem);
        out[at..at + take].copy_from_slice(&word[..take]);
        take
    }
}

/// The cuts of `m` retained rows of `bits` bits (`bits > CUT_BITS`) at the marked row: their
/// shift, their low bits (the mark's), and how many there are. The cuts are the rows
/// `(q << shift) | low` below `m`, `q` the cut's index, so `j` is one iff
/// `(j ^ mark) & (2^shift - 1) == 0`.
pub(super) fn cuts(m: usize, bits: u32, mark: u32) -> (u32, u32, usize) {
    let shift = bits - CUT_BITS;
    let low = mark & ((1u32 << shift) - 1);
    let n_cuts = (m - low as usize).div_ceil(1 << shift);
    assert!(n_cuts <= MAX_CUTS);
    (shift, low, n_cuts)
}

/// The walk from `mark` over `edges` (narrow codes only), as `emit`'s: the block's first
/// `period` bytes in `out[..period]` (sized `n + 16` by the caller), `period` returned; or
/// None, `out` partly written, where the serial walk must decide (see the module's notes)
pub(super) fn relay(
    edges: &[u32],
    w: Words,
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut [u8],
) -> Option<usize> {
    let m = edges.len();
    // `lens` and `inline16` to one length, so that one check serves both
    let k = w.lens.len().min(w.inline16.len());
    let w = Words {
        lens: &w.lens[..k],
        inline16: &w.inline16[..k],
        ..w
    };
    #[cfg(test)]
    if RELAY_OFF.with(|r| r.get()) {
        return None;
    }
    if m < RELAY_MIN || bits <= CUT_BITS || out.len() < n + INLINE {
        return None;
    }
    let (shift, low, n_cuts) = cuts(m, bits, mark);
    let cm = (1u32 << shift) - 1;
    let is_cut = |j: u32| j & cm == low;
    let row = |c: usize| ((c as u32) << shift) | low;
    let cut = |j: u32| (j >> shift) as usize;
    if n_cuts < 3 {
        return None;
    }
    let mask = (1u32 << bits) - 1;

    let mut state = [FREE; MAX_CUTS];
    // a scouted segment's bytes, steps, and the cut it ends at
    let mut seg_len = [0u32; MAX_CUTS];
    let mut seg_steps = [0u32; MAX_CUTS];
    let mut seg_next = [0u16; MAX_CUTS];
    // the emitters' tasks: from a row at an offset, a number of steps
    let mut t_row = [0u32; MAX_CUTS];
    let mut t_at = [0u32; MAX_CUTS];
    let mut t_steps = [0u32; MAX_CUTS];
    // the one task cut short by `n` (bytes [at, n) from a row), emitted alone
    let mut last: Option<(u32, usize)> = None;
    let mut tasks = 0usize;
    // every piece's start after the first (a task's, or where lane 0 resumes): restored last
    let mut b_row = [0u32; 2 * MAX_CUTS];
    let mut b_at = [0u32; 2 * MAX_CUTS];
    let mut bounds = 0usize;

    state[cut(mark)] = OWNED;
    let mut next_claim = 0usize;
    let mut claim = |state: &mut [u8; MAX_CUTS]| -> usize {
        while next_claim < n_cuts {
            let c = next_claim;
            next_claim += 1;
            if state[c] == FREE {
                state[c] = SCOUTING;
                return c;
            }
        }
        IDLE
    };
    let mut s_cut = [IDLE; MU];
    let mut s_j = [0u32; MU];
    let mut s_acc = [0u32; MU];
    // the scouts' rounds so far, and the round each scout's cut was claimed at
    let mut round = 0usize;
    let mut s_from = [0usize; MU];
    let mut scouts = 0usize;
    for s in 1..MU {
        let c = claim(&mut state);
        if c != IDLE {
            s_cut[s] = c;
            s_j[s] = row(c);
            scouts += 1;
        }
    }
    // the scouts' steps in all
    let mut scouted = 0usize;

    // a task at lane 0's offset: whole, for the emitters; or cut short by `n`, `last`
    macro_rules! task {
        ($row:expr, $at:expr, $steps:expr, $short:expr) => {
            if $short {
                last = Some(($row, $at));
            } else {
                (t_row[tasks], t_at[tasks], t_steps[tasks]) = ($row, $at as u32, $steps);
                tasks += 1;
            }
            (b_row[bounds], b_at[bounds]) = ($row, $at as u32);
            bounds += 1;
        };
    }
    // ---- phase 1: lane 0 emits from the mark while the scouts measure segments ----
    let mut j0 = mark;
    let mut at0 = 0usize;
    // the walk's steps so far: lane 0's, and the tasks' it has placed
    let mut walked = 0usize;
    let period = loop {
        // lane 0's step
        let e = edges[j0 as usize];
        j0 = e & mask;
        at0 = w.put((e >> bits) as usize, out, at0, n);
        walked += 1;
        // the scouts' steps: all seven in step (the usual case), or those still scouting
        if scouts > 0 {
            scouted += scouts;
            round += 1;
            let mut hit = false;
            if scouts == MU - 1 {
                for s in 1..MU {
                    let e = edges[s_j[s] as usize];
                    let nj = e & mask;
                    s_j[s] = nj;
                    s_acc[s] += w.len((e >> bits) as usize) as u32;
                    hit |= is_cut(nj);
                }
            } else {
                for s in 1..MU {
                    if s_cut[s] != IDLE {
                        let e = edges[s_j[s] as usize];
                        let nj = e & mask;
                        s_j[s] = nj;
                        s_acc[s] += w.len((e >> bits) as usize) as u32;
                        hit |= is_cut(nj);
                    }
                }
            }
            if hit {
                for s in 1..MU {
                    if s_cut[s] == IDLE || !is_cut(s_j[s]) {
                        continue;
                    }
                    let c = s_cut[s];
                    state[c] = SCOUTED;
                    seg_len[c] = s_acc[s];
                    seg_steps[c] = (round - s_from[s]) as u32;
                    seg_next[c] = cut(s_j[s]) as u16;
                    let c = claim(&mut state);
                    s_cut[s] = c;
                    if c == IDLE {
                        scouts -= 1;
                    } else {
                        (s_j[s], s_acc[s], s_from[s]) = (row(c), 0, round);
                    }
                }
            }
            // a scout in a cycle with no cut (not a permutation) would walk forever
            if scouted > m {
                return None;
            }
        }
        if at0 >= n {
            break n;
        }
        // lane 0 at a cut: place what is scouted, take over what is being scouted
        let mut jumped = false;
        let mut end = None;
        while is_cut(j0) {
            if j0 == mark {
                end = Some(at0);
                break;
            }
            let c = cut(j0);
            match state[c] {
                SCOUTED => {
                    let to = (at0 + seg_len[c] as usize).min(n);
                    if to > at0 {
                        task!(j0, at0, seg_steps[c], to < at0 + seg_len[c] as usize);
                    }
                    state[c] = OWNED;
                    walked += seg_steps[c] as usize;
                    at0 = to;
                    j0 = row(usize::from(seg_next[c]));
                    jumped = true;
                    if at0 >= n {
                        end = Some(n);
                        break;
                    }
                }
                FREE => {
                    state[c] = OWNED;
                    break;
                }
                SCOUTING => {
                    let s = (1..MU).find(|&s| s_cut[s] == c)?;
                    let to = (at0 + s_acc[s] as usize).min(n);
                    let steps = round - s_from[s];
                    if to > at0 {
                        task!(j0, at0, steps as u32, to < at0 + s_acc[s] as usize);
                    }
                    state[c] = OWNED;
                    walked += steps;
                    at0 = to;
                    jumped |= steps > 0;
                    j0 = s_j[s];
                    let c = claim(&mut state);
                    s_cut[s] = c;
                    if c == IDLE {
                        scouts -= 1;
                    } else {
                        (s_j[s], s_acc[s], s_from[s]) = (row(c), 0, round);
                    }
                    if at0 >= n {
                        end = Some(n);
                    }
                    break;
                }
                // a cut met twice: not a permutation
                _ => return None,
            }
        }
        if walked > m {
            return None;
        }
        if let Some(p) = end {
            break p;
        }
        if walked == m {
            // the serial walk stops here, short of the mark and of `n`
            return None;
        }
        if jumped {
            (b_row[bounds], b_at[bounds]) = (j0, at0 as u32);
            bounds += 1;
        }
    };
    if period == 0 {
        return None;
    }

    // ---- phase 2: eight emitters over the tasks ----
    // Each task is a whole segment of known steps, so the lanes run in step for as many
    // steps as the shortest has left, with no end checks; then the finished ones take the
    // next tasks.
    let mut next_t = 0usize;
    let mut e_j = [0u32; MU];
    let mut e_at = [0usize; MU];
    let mut e_rem = [0u32; MU];
    let mut active = 0usize;
    for l in 0..MU {
        if next_t < tasks {
            (e_j[l], e_at[l], e_rem[l]) = (t_row[next_t], t_at[next_t] as usize, t_steps[next_t]);
            next_t += 1;
            active += 1;
        }
    }
    while active == MU {
        let k = e_rem.iter().copied().min().unwrap_or(0);
        for _ in 0..k {
            let raw = e_j.map(|j| edges[j as usize]);
            for l in 0..MU {
                let e = raw[l];
                e_j[l] = e & mask;
                e_at[l] = w.put((e >> bits) as usize, out, e_at[l], n);
            }
        }
        for l in 0..MU {
            e_rem[l] -= k;
            if e_rem[l] == 0 {
                if next_t < tasks {
                    (e_j[l], e_at[l], e_rem[l]) =
                        (t_row[next_t], t_at[next_t] as usize, t_steps[next_t]);
                    next_t += 1;
                } else {
                    active -= 1;
                }
            }
        }
    }
    // fewer than eight left: each lane to its end, in step while any is left
    while active > 0 {
        active = 0;
        for l in 0..MU {
            if e_rem[l] == 0 {
                continue;
            }
            let e = edges[e_j[l] as usize];
            e_j[l] = e & mask;
            e_at[l] = w.put((e >> bits) as usize, out, e_at[l], n);
            e_rem[l] -= 1;
            active += usize::from(e_rem[l] > 0);
        }
    }
    // the task cut short by `n`, to `n`
    if let Some((mut j, mut at)) = last {
        for _ in 0..m {
            if at >= n {
                break;
            }
            let e = edges[j as usize];
            j = e & mask;
            at = w.put((e >> bits) as usize, out, at, n);
        }
    }

    // ---- phase 3: every piece's first 16 bytes, exactly ----
    for b in 0..bounds {
        let mut at = b_at[b] as usize;
        if at >= period {
            continue;
        }
        let mut j = b_row[b];
        let mut rem = INLINE.min(period - at);
        // (bounded: the walk from a piece's row spells its bytes; zero-length words, which
        // the kernel never makes, cannot loop it)
        for _ in 0..m {
            if rem == 0 {
                break;
            }
            let e = edges[j as usize];
            j = e & mask;
            let took = w.put_exact((e >> bits) as usize, out, at, rem);
            at += took;
            rem -= took;
        }
    }
    #[cfg(test)]
    RELAYED.with(|r| r.set(r.get() + 1));
    Some(period)
}

#[cfg(test)]
thread_local! {
    /// (test only) the walks the relay emitted on this thread
    pub(super) static RELAYED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// (test only) the relay off: every walk serial (the measurement harnesses' control)
    pub(super) static RELAY_OFF: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
mod tests;
