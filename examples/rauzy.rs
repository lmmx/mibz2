//! MONODROMY, mission 1 (docs/journal/monodromy/2026-11-10-rauzy-veech.md): the inverse BWT by
//! Rauzy-Veech induction on the LF interval exchange. A witness and a census; nothing in the
//! decoder calls it.
//!
//! ```text
//! rauzy FILE.bz2 [EVERY] [MIN_ROWS] [ROUNDS]     one JSON line a block (stdout)
//! ```
//!
//! **The object.** LF is a translation on each run of L: an interval exchange on r letters
//! (the runs). Its data are two orders of the letters and a length each: the TOP order is the
//! runs as they lie in L, the BOTTOM order is their images, the runs sorted stably by byte.
//!
//! **One step.** A = the last letter on top, B = the last on the bottom. The longer wins and
//! loses the other's length; the loser's return word becomes (A's word)(B's word) in text
//! order; one list move re-seats the loser. The first-return map of LF to the shortened prefix
//! [0, m) is again an interval exchange on the same letters. No row and no interval is ever
//! pushed through LF: a step reads two lengths and relinks one node.
//!
//! **Run to one point** it leaves a straight-line program for the block: one binary rule a
//! step. The block is then written by expanding it, each rule's second and later occurrences
//! copied from where the first was written.
//!
//! Every block's bytes are checked against `bwt::walk`.

use mibz2::bwt::{build, walk, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::time::Instant;

const NIL: u32 = u32::MAX;

/// A straight-line program in text order: symbol s < 256 is the byte s; symbol 256 + j is rule
/// j, the expansion of `left[j]` then that of `right[j]`, `len[j]` bytes
#[derive(Default)]
struct Grammar {
    left: Vec<u32>,
    right: Vec<u32>,
    len: Vec<u32>,
}

impl Grammar {
    fn clear(&mut self) {
        self.left.clear();
        self.right.clear();
        self.len.clear();
    }
    #[inline(always)]
    fn rule(&mut self, l: u32, r: u32, len: u32) -> u32 {
        self.left.push(l);
        self.right.push(r);
        self.len.push(len);
        255 + self.left.len() as u32
    }
}

/// The induction's state, kept from block to block
#[derive(Default)]
struct Induction {
    lam: Vec<u32>,
    h: Vec<u32>,
    word: Vec<u32>,
    tprev: Vec<u32>,
    tnext: Vec<u32>,
    bprev: Vec<u32>,
    bnext: Vec<u32>,
    byte: Vec<u8>,
}

#[derive(Default, Clone, Copy)]
struct Counts {
    runs: u64,
    steps: u64,
    top: u64,
    bottom: u64,
    equal: u64,
    dropped: u64,
    /// maximal runs of consecutive steps with one winner (Zorich's steps)
    streaks: u64,
    longest: u64,
    /// the letters left when the tracked point's cycle closed
    left: u64,
    /// steps that leave the same two letters last (the loser was already just after the
    /// winner): q in a row are one division, and one run-length rule
    repeats: u64,
    /// maximal groups of such steps
    divisions: u64,
    /// the longest chain of steps each needing the one before: a step needs the last step
    /// that touched either of its letters, and the step before it in its own segment (the
    /// stretch between two coincidences of the top and bottom partitions, ended by an equal
    /// step). Steps not so ordered commute: this is the induction's depth with every
    /// coincidence worked at once.
    depth: u64,
    /// the most steps in one segment
    widest: u64,
    /// the same chain counted in whole segments: a segment needs the segments that last
    /// re-seated or shortened its letters. Rounds of independent two-pointer merges.
    rounds: u64,
    /// segments that meet a letter they re-seated themselves (not one merge: a Euclid
    /// stretch), and the steps in them
    selfish: u64,
    selfish_steps: u64,
}

/// What the induction leaves: the block is `period`-periodic, one period the expansion of
/// `root` rotated left by `period - t0`
struct Closed {
    root: u32,
    t0: u32,
    period: u32,
}

/// The runs of `l` as the letters of LF's interval exchange, into `s`; the letter holding row
/// `p` and p's offset in it
fn letters(l: &[u8], p: u32, s: &mut Induction) -> (u32, u32) {
    s.lam.clear();
    s.byte.clear();
    let (mut trk, mut off) = (0u32, 0u32);
    let mut i = 0usize;
    let n = l.len();
    while i < n {
        let c = l[i];
        let mut j = i + 1;
        while j < n && l[j] == c {
            j += 1;
        }
        if (i..j).contains(&(p as usize)) {
            trk = s.lam.len() as u32;
            off = p - i as u32;
        }
        s.lam.push((j - i) as u32);
        s.byte.push(c);
        i = j;
    }
    (trk, off)
}

/// Rauzy-Veech induction of LF (letters in `s.lam`, `s.byte`) to the cycle of the tracked
/// point (letter `trk`, offset `off`): its rules into `g`
fn induce(
    s: &mut Induction,
    mut trk: u32,
    mut off: u32,
    g: &mut Grammar,
    c: &mut Counts,
) -> Closed {
    let r = s.lam.len();
    c.runs = r as u64;
    g.clear();
    s.h.clear();
    s.h.resize(r, 1);
    s.word.clear();
    s.word.extend(s.byte.iter().map(|&b| b as u32));
    // the top order: the runs as they lie
    s.tprev.clear();
    s.tnext.clear();
    for i in 0..r as u32 {
        s.tprev.push(if i == 0 { NIL } else { i - 1 });
        s.tnext.push(if i + 1 == r as u32 { NIL } else { i + 1 });
    }
    // the bottom order: the runs sorted stably by byte
    let mut cnt = [0u32; 257];
    for &b in &s.byte {
        cnt[b as usize + 1] += 1;
    }
    for k in 0..256 {
        cnt[k + 1] += cnt[k];
    }
    let mut order = vec![0u32; r];
    for (i, &b) in s.byte.iter().enumerate() {
        order[cnt[b as usize] as usize] = i as u32;
        cnt[b as usize] += 1;
    }
    s.bprev.clear();
    s.bprev.resize(r, NIL);
    s.bnext.clear();
    s.bnext.resize(r, NIL);
    for k in 0..r {
        if k > 0 {
            s.bprev[order[k] as usize] = order[k - 1];
        }
        if k + 1 < r {
            s.bnext[order[k] as usize] = order[k + 1];
        }
    }
    let (lam, h, word) = (&mut s.lam, &mut s.h, &mut s.word);
    let (tprev, tnext, bprev, bnext) = (&mut s.tprev, &mut s.tnext, &mut s.bprev, &mut s.bnext);
    let mut tlast = r as u32 - 1;
    let mut blast = order[r - 1];
    let mut left = r as u64;
    let mut v = 0u32;
    let (mut winner, mut streak) = (NIL, 0u64);
    let mut repeating = false;
    let mut touch = vec![0u32; r];
    let (mut seg_depth, mut seg_steps) = (0u32, 0u64);
    // by whole segments: the round that last touched a letter, the segment that last
    // re-seated it, and the current segment's letters
    let mut born = vec![0u32; r];
    let mut seated = vec![u32::MAX; r];
    let mut members: Vec<u32> = Vec::new();
    let (mut seg, mut seg_round, mut seg_selfish) = (0u32, 0u32, false);
    loop {
        let (a, b) = (tlast, blast);
        let (ai, bi) = (a as usize, b as usize);
        if a == b {
            // the last letter is sent to itself: its points' cycles are closed
            if trk == a {
                c.left = left;
                c.longest = c.longest.max(streak);
                return Closed {
                    root: word[ai],
                    t0: v,
                    period: h[ai],
                };
            }
            // (cycles the walk is not on: a block of a repeated word)
            c.dropped += 1;
            left -= 1;
            tlast = tprev[ai];
            tnext[tlast as usize] = NIL;
            blast = bprev[bi];
            bnext[blast as usize] = NIL;
            continue;
        }
        c.steps += 1;
        let (la, lb) = (lam[ai], lam[bi]);
        let d = 1 + touch[ai].max(touch[bi]).max(seg_depth);
        touch[ai] = d;
        touch[bi] = d;
        c.depth = c.depth.max(d as u64);
        seg_steps += 1;
        c.widest = c.widest.max(seg_steps);
        seg_round = seg_round.max(born[ai]).max(born[bi]);
        seg_selfish |= seated[ai] == seg || seated[bi] == seg;
        members.push(a);
        members.push(b);
        // (the loser is re-seated: top wins moves B, otherwise A moves or, when equal, stays as B)
        seated[if la > lb { bi } else { ai }] = seg;
        if la == lb {
            seated[bi] = seg;
            let round = seg_round + 1;
            for &x in &members {
                born[x as usize] = round;
            }
            c.rounds = c.rounds.max(round as u64);
            if seg_selfish {
                c.selfish += 1;
                c.selfish_steps += seg_steps;
            }
            members.clear();
            seg += 1;
            seg_round = 0;
            seg_selfish = false;
            seg_depth = 0;
            seg_steps = 0;
        } else {
            seg_depth = d;
        }
        let id = g.rule(word[ai], word[bi], h[ai] + h[bi]);
        let w = if la > lb { a } else { b };
        if w == winner {
            streak += 1;
        } else {
            c.streaks += 1;
            c.longest = c.longest.max(streak);
            winner = w;
            streak = 1;
        }
        if la > lb {
            // top wins: B's points go on through A's cut-off right end
            c.top += 1;
            if trk == a && off >= la - lb {
                trk = b;
                off -= la - lb;
                v += h[bi];
            }
            word[bi] = id;
            h[bi] += h[ai];
            lam[ai] = la - lb;
            let p = bprev[bi];
            if p == a {
                c.repeats += 1;
                c.divisions += !repeating as u64;
            }
            repeating = p == a;
            if p != a {
                blast = p;
                bnext[p as usize] = NIL;
                let nx = bnext[ai];
                bnext[ai] = b;
                bprev[bi] = a;
                bnext[bi] = nx;
                bprev[nx as usize] = b;
            }
        } else if la < lb {
            // bottom wins: B's right end, as wide as A, takes A's name and goes on through A
            c.bottom += 1;
            if trk == a {
                v += h[bi];
            } else if trk == b && off >= lb - la {
                trk = a;
                off -= lb - la;
            }
            word[ai] = id;
            h[ai] += h[bi];
            lam[bi] = lb - la;
            let p = tprev[ai];
            if p == b {
                c.repeats += 1;
                c.divisions += !repeating as u64;
            }
            repeating = p == b;
            if p != b {
                tlast = p;
                tnext[p as usize] = NIL;
                let nx = tnext[bi];
                tnext[bi] = a;
                tprev[ai] = b;
                tnext[ai] = nx;
                tprev[nx as usize] = a;
            }
        } else {
            // equal: B goes on through all of A, and A is gone
            c.equal += 1;
            repeating = false;
            left -= 1;
            if trk == a {
                trk = b;
                v += h[bi];
            }
            word[bi] = id;
            h[bi] += h[ai];
            let p = tprev[ai];
            tlast = p;
            tnext[p as usize] = NIL;
            let q = bprev[bi];
            if q == a {
                let pa = bprev[ai];
                bprev[bi] = pa;
                if pa != NIL {
                    bnext[pa as usize] = b;
                }
            } else {
                blast = q;
                bnext[q as usize] = NIL;
                let (pa, na) = (bprev[ai], bnext[ai]);
                bprev[bi] = pa;
                bnext[bi] = na;
                if pa != NIL {
                    bnext[pa as usize] = b;
                }
                bprev[na as usize] = b;
            }
        }
    }
}

/// The expansion of `root` into `out` (its length the root's): a rule's first occurrence is
/// expanded, its later ones copied from there. Returns (copies, bytes copied, deepest stack).
fn expand(
    g: &Grammar,
    root: u32,
    out: &mut [u8],
    first: &mut Vec<u32>,
    stack: &mut Vec<u32>,
) -> (u64, u64, u64) {
    first.clear();
    first.resize(g.left.len(), NIL);
    stack.clear();
    stack.push(root);
    let (mut pos, mut copies, mut copied, mut deepest) = (0usize, 0u64, 0u64, 0u64);
    while let Some(s) = stack.pop() {
        if s < 256 {
            out[pos] = s as u8;
            pos += 1;
            continue;
        }
        let j = (s - 256) as usize;
        let f = first[j];
        if f != NIL {
            let len = g.len[j] as usize;
            out.copy_within(f as usize..f as usize + len, pos);
            pos += len;
            copies += 1;
            copied += len as u64;
        } else {
            first[j] = pos as u32;
            stack.push(g.right[j]);
            stack.push(g.left[j]);
            deepest = deepest.max(stack.len() as u64);
        }
    }
    debug_assert_eq!(pos, out.len());
    (copies, copied, deepest)
}

/// The block from what the induction left: `period` bytes expanded, rotated to the walk's
/// start, repeated to n
fn write_block(
    g: &Grammar,
    cl: &Closed,
    n: usize,
    out: &mut Vec<u8>,
    tmp: &mut Vec<u8>,
    first: &mut Vec<u32>,
    stack: &mut Vec<u32>,
) -> (u64, u64, u64) {
    let p = cl.period as usize;
    tmp.clear();
    tmp.resize(p, 0);
    let st = expand(g, cl.root, tmp, first, stack);
    out.clear();
    let cut = (p - cl.t0 as usize % p) % p;
    while out.len() < n {
        out.extend_from_slice(&tmp[cut..]);
        out.extend_from_slice(&tmp[..cut]);
    }
    out.truncate(n);
    st
}

// ---------------------------------------------------------------------------------------
// The kernel: the same induction with a letter's state in one 32-byte node, a word of at
// most SMALL bytes kept as bytes (an arena), and only longer words as rules. The block is
// written from the rules in two sequential passes: where each rule first lies (last rule
// to first), then its bytes (first to last).

const SMALL: u32 = 16;
/// How many letters ahead of an order's last letter the kernel fetches
const AHEAD: usize = 8;
/// The arena's first word: the 256 single bytes, after 16 bytes of slack for a load that
/// ends at a word's end
const BYTES: u32 = 16;

#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
struct Node {
    lam: u32,
    /// its word's length: its points' return time
    len: u32,
    /// its word: an arena offset if len <= SMALL, else a rule
    word: u32,
    _pad: u32,
    /// its neighbours in the top order [0] and the bottom order [1]; the node past the last
    /// letter stands for "none", and takes the writes a step does not mean
    prev: [u32; 2],
    next: [u32; 2],
}

/// A long word: `left` then `right` (each an arena offset if its length is at most SMALL,
/// else a rule), `llen` and `len - llen` bytes
#[repr(C)]
#[derive(Clone, Copy)]
struct Rule {
    left: u32,
    right: u32,
    llen: u32,
    len: u32,
}

#[derive(Default)]
struct Kernel {
    nodes: Vec<Node>,
    order: Vec<u32>,
    arena: Vec<u8>,
    rules: Vec<Rule>,
    pos: Vec<u32>,
    tmp: Vec<u8>,
}

#[inline(always)]
fn prefetch<T>(p: *const T) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_mm_prefetch(p as *const i8, core::arch::x86_64::_MM_HINT_T0);
    }
    #[cfg(not(target_arch = "x86_64"))]
    let _ = p;
}

/// 16 bytes from `src` to `dst`, whatever the word's length (both have 16 bytes there)
#[inline(always)]
unsafe fn copy16(dst: *mut u8, src: *const u8) {
    unsafe { core::ptr::copy_nonoverlapping(src, dst, 16) }
}

/// Exactly `len` <= 16 bytes from `src` to `dst`: 16 are read at each, and the bytes past
/// `len` are written back as they were
#[inline(always)]
unsafe fn copy_small(dst: *mut u8, src: *const u8, len: usize) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::*;
        static MASK: [u8; 32] = [
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let m = _mm_loadu_si128(MASK.as_ptr().add(16 - len) as *const __m128i);
        let d = _mm_loadu_si128(dst as *const __m128i);
        let s = _mm_loadu_si128(src as *const __m128i);
        _mm_storeu_si128(
            dst as *mut __m128i,
            _mm_or_si128(_mm_and_si128(m, s), _mm_andnot_si128(m, d)),
        );
    }
    #[cfg(not(target_arch = "x86_64"))]
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, len)
    }
}

/// Exactly `len` >= 16 bytes from `src` to `dst` (apart): 16 at a time, the last 16 anchored
/// at the end
#[inline(always)]
unsafe fn copy_long(dst: *mut u8, src: *const u8, len: usize) {
    unsafe {
        let mut i = 0;
        while i + 16 <= len {
            copy16(dst.add(i), src.add(i));
            i += 16;
        }
        copy16(dst.add(len - 16), src.add(len - 16));
    }
}

impl Kernel {
    /// The letters of `l`'s interval exchange, top and bottom orders linked; the letter
    /// holding row `p` and p's offset in it
    fn letters(&mut self, l: &[u8], p: u32) -> (u32, u32) {
        let n = l.len();
        self.nodes.clear();
        let (mut trk, mut off) = (0u32, 0u32);
        let mut cnt = [0u32; 257];
        let mut i = 0usize;
        while i < n {
            let c = l[i];
            let mut j = i + 1;
            // eight bytes at a time while they all equal c
            let pat = u64::from_ne_bytes([c; 8]);
            while j + 8 <= n {
                let x = u64::from_le_bytes(l[j..j + 8].try_into().unwrap()) ^ pat;
                if x != 0 {
                    j += (x.trailing_zeros() / 8) as usize;
                    break;
                }
                j += 8;
            }
            while j < n && l[j] == c {
                j += 1;
            }
            if (i..j).contains(&(p as usize)) {
                trk = self.nodes.len() as u32;
                off = p - i as u32;
            }
            let k = self.nodes.len() as u32;
            self.nodes.push(Node {
                lam: (j - i) as u32,
                len: 1,
                word: BYTES + c as u32,
                _pad: 0,
                prev: [k.wrapping_sub(1), 0],
                next: [k + 1, 0],
            });
            cnt[c as usize + 1] += 1;
            i = j;
        }
        let r = self.nodes.len();
        let none = r as u32;
        self.nodes[0].prev[0] = none;
        for k in 0..256 {
            cnt[k + 1] += cnt[k];
        }
        self.order.clear();
        self.order.resize(r, 0);
        for i in 0..r {
            let c = (self.nodes[i].word - BYTES) as usize;
            self.order[cnt[c] as usize] = i as u32;
            cnt[c] += 1;
        }
        for k in 0..r {
            let i = self.order[k] as usize;
            self.nodes[i].prev[1] = if k > 0 { self.order[k - 1] } else { none };
            self.nodes[i].next[1] = if k + 1 < r { self.order[k + 1] } else { none };
        }
        self.nodes.push(Node::default());
        (trk, off)
    }

    /// The induction, to the tracked point's cycle. Returns (the cycle's word, its length,
    /// t0, steps).
    ///
    /// A step without a data-dependent branch. With la, lb the last letters' lengths:
    /// - la > lb: B takes the word and moves, in the bottom order, to just after A;
    /// - la < lb: A takes the word and moves, in the top order, to just after B;
    /// - la = lb: as la < lb, and then B, now of no length, is unlinked from both orders (A
    ///   sits where B sat on top, and stays where it was on the bottom).
    ///
    /// The writes a case does not mean go to the node that stands for "none".
    fn induce(&mut self, mut trk: u32, mut off: u32) -> (u32, u32, u32, u64) {
        let r = self.nodes.len() - 1;
        let none = r as u32;
        self.rules.clear();
        self.arena.clear();
        self.arena
            .resize((BYTES as usize + 256 + 64).max(self.arena.capacity()), 0);
        for b in 0..256 {
            self.arena[BYTES as usize + b] = b as u8;
        }
        let mut top = BYTES as usize + 256;
        let order = &self.order[..];
        let nd = self.nodes.as_mut_ptr();
        let (mut tl, mut bl) = (r as u32 - 1, order[r - 1]);
        // A letter met at the end of an order may lie anywhere (the bottom order is sorted by
        // byte, and a loser is re-seated far from where it was): two pointers run AHEAD links
        // down the orders in front of the last letters, fetching what they pass. A stale
        // pointer only fetches the wrong lines.
        let (mut at, mut ab) = (tl, bl);
        // SAFETY: links are letters or `none`, all below nodes.len()
        unsafe {
            for _ in 0..AHEAD {
                at = (*nd.add(at as usize)).prev[0];
                ab = (*nd.add(ab as usize)).prev[1];
            }
        }
        let mut v = 0u32;
        let mut steps = 0u64;
        // (a rule is written every step, and kept only if its word is long)
        self.rules.reserve(r + 1);
        let mut nr = 0usize;
        loop {
            let (a, b) = (tl, bl);
            // SAFETY: a, b and every link read below are letters or `none`, all below
            // nodes.len(); a node's links are only ever set to such values
            unsafe {
                let (pa, pb) = (nd.add(a as usize), nd.add(b as usize));
                if a == b {
                    if trk == a {
                        self.rules.set_len(nr);
                        return ((*pa).word, (*pa).len, v, steps);
                    }
                    tl = (*pa).prev[0];
                    (*nd.add(tl as usize)).next[0] = none;
                    bl = (*pb).prev[1];
                    (*nd.add(bl as usize)).next[1] = none;
                    continue;
                }
                steps += 1;
                let (la, lb) = ((*pa).lam, (*pb).lam);
                let (na, nb) = ((*pa).len, (*pb).len);
                let (wa, wb) = ((*pa).word, (*pb).word);
                let (a_tprev, b_bprev) = ((*pa).prev[0], (*pb).prev[1]);
                let len = na + nb;
                let small = len <= SMALL;
                // the loser's word, A's then B's: as bytes, and as a rule; one of them is kept
                if top + 64 > self.arena.len() {
                    let grown = self.arena.len() * 2;
                    self.arena.resize(grown, 0);
                }
                if nr == self.rules.capacity() {
                    self.rules.set_len(nr);
                    self.rules.reserve(nr.max(1024));
                }
                let base = self.arena.as_mut_ptr();
                let (sa, sb, sn) = if small {
                    (wa, wb, na)
                } else {
                    (BYTES, BYTES, 0)
                };
                copy16(base.add(top), base.add(sa as usize));
                copy16(base.add(top + sn as usize), base.add(sb as usize));
                *self.rules.as_mut_ptr().add(nr) = Rule {
                    left: wa,
                    right: wb,
                    llen: na,
                    len,
                };
                let id = if small { top as u32 } else { nr as u32 };
                top += if small { len as usize } else { 0 };
                nr += !small as usize;
                let (gt, eq) = (la > lb, la == lb);
                if trk == a {
                    if gt {
                        if off >= la - lb {
                            trk = b;
                            off -= la - lb;
                            v += nb;
                        }
                    } else {
                        v += nb;
                    }
                } else if trk == b && !gt && off >= lb - la {
                    trk = a;
                    off -= lb - la;
                }
                // the loser takes the word, the winner loses the loser's length
                let (lo, wi) = if gt { (b, a) } else { (a, b) };
                let (plo, pwi) = (nd.add(lo as usize), nd.add(wi as usize));
                (*plo).word = id;
                (*plo).len = len;
                (*pwi).lam = la.abs_diff(lb);
                // the loser moves, in the order it is last in, to just after the winner
                let side = gt as usize;
                let p = if gt { b_bprev } else { a_tprev };
                let c = p != wi;
                let nx = (*pwi).next[side];
                let last = if c { p } else { lo };
                // (a last letter's `next` is never read: nothing is put after a last letter)
                (*pwi).next[side] = lo;
                (*plo).prev[side] = wi;
                (*plo).next[side] = if c { nx } else { none };
                (*nd.add(if c { nx } else { none } as usize)).prev[side] = lo;
                // equal: B, of no length now, leaves both orders
                let b_tprev = (*pb).prev[0];
                (*nd.add(if eq { a } else { none } as usize)).prev[0] = b_tprev;
                (*nd.add(if eq { b_tprev } else { none } as usize)).next[0] = a;
                let ntl = if gt { tl } else { last };
                let nbl = if gt {
                    last
                } else if eq {
                    b_bprev
                } else {
                    bl
                };
                let (qt, qb) = (
                    (*nd.add(at as usize)).prev[0],
                    (*nd.add(ab as usize)).prev[1],
                );
                at = if ntl != tl { qt } else { at };
                ab = if nbl != bl { qb } else { ab };
                prefetch(nd.add(at as usize));
                prefetch(nd.add(ab as usize));
                tl = ntl;
                bl = nbl;
            }
        }
    }

    /// The block from the induction's word (`word`, `period` bytes, the walk `t0` steps up
    /// its cycle), into `out`
    fn write(&mut self, word: u32, period: u32, t0: u32, n: usize, out: &mut Vec<u8>) {
        let p = period as usize;
        self.tmp.clear();
        self.tmp.resize(p + 32, 0);
        let base = self.tmp.as_mut_ptr();
        let arena = self.arena.as_ptr();
        if period <= SMALL {
            unsafe { copy_small(base, arena.add(word as usize), p) };
        } else {
            // where each rule first lies: a rule's children lie no later than inside it
            let nr = self.rules.len();
            self.pos.clear();
            self.pos.resize(nr, NIL);
            self.pos[word as usize] = 0;
            for j in (0..nr).rev() {
                let q = self.pos[j];
                let rl = self.rules[j];
                if q == NIL {
                    continue;
                }
                if rl.llen > SMALL {
                    let x = &mut self.pos[rl.left as usize];
                    *x = (*x).min(q);
                }
                if rl.len - rl.llen > SMALL {
                    let x = &mut self.pos[rl.right as usize];
                    *x = (*x).min(q + rl.llen);
                }
            }
            // its bytes: each child's from the arena, or from where the child first lies
            for j in 0..nr {
                let q = self.pos[j];
                if q == NIL {
                    continue;
                }
                let rl = self.rules[j];
                let (ll, rr) = (rl.llen as usize, (rl.len - rl.llen) as usize);
                unsafe {
                    let d = base.add(q as usize);
                    if rl.llen <= SMALL {
                        copy_small(d, arena.add(rl.left as usize), ll);
                    } else {
                        let f = self.pos[rl.left as usize];
                        if f != q {
                            copy_long(d, base.add(f as usize), ll);
                        }
                    }
                    let d = d.add(ll);
                    if rl.len - rl.llen <= SMALL {
                        copy_small(d, arena.add(rl.right as usize), rr);
                    } else {
                        let f = self.pos[rl.right as usize];
                        if f != q + rl.llen {
                            copy_long(d, base.add(f as usize), rr);
                        }
                    }
                }
            }
        }
        out.clear();
        let cut = (p - t0 as usize % p) % p;
        while out.len() < n {
            out.extend_from_slice(&self.tmp[cut..p]);
            out.extend_from_slice(&self.tmp[..cut]);
        }
        out.truncate(n);
    }
}

// ---------------------------------------------------------------------------------------
// The block's CRC from the grammar alone. CRC-32/BZIP2 is a homomorphism: with R(w) = w(x)
// x^32 mod P and X(w) = x^(8|w|) mod P, R(uv) = R(u) X(v) + R(v) and X(uv) = X(u) X(v). So a
// rule costs two multiplications mod P, and the block's CRC S + (the grammar's depth) of
// them, whatever n is.

/// a(x) b(x) mod P, bit 31 the coefficient of x^31: Horner over b's bits, high to low
fn mulmod(a: u32, mut b: u32) -> u32 {
    let mut acc = 0u32;
    for _ in 0..32 {
        acc = (acc << 1)
            ^ if acc & 0x8000_0000 != 0 {
                mibz2::crc::POLY
            } else {
                0
            };
        if b & 0x8000_0000 != 0 {
            acc ^= a;
        }
        b <<= 1;
    }
    acc
}

/// (R, X) of the concatenation of two words
#[inline]
fn cat(u: (u32, u32), v: (u32, u32)) -> (u32, u32) {
    (mulmod(u.0, v.1) ^ v.0, mulmod(u.1, v.1))
}

/// The CRC of the block the grammar and `cl` stand for (one period), with the count of
/// multiplications; None if the block is a repeated word
fn grammar_crc(g: &Grammar, cl: &Closed, n: usize) -> Option<(u32, u64)> {
    if cl.period as usize != n {
        return None;
    }
    // (R, X) of a byte: x^32 c mod P and x^8
    let byte = |c: u32| -> (u32, u32) {
        let mut r = c << 24;
        for _ in 0..8 {
            r = (r << 1)
                ^ if r & 0x8000_0000 != 0 {
                    mibz2::crc::POLY
                } else {
                    0
                };
        }
        (r, 0x100)
    };
    let mut rx: Vec<(u32, u32)> = Vec::with_capacity(g.left.len());
    let get = |rx: &Vec<(u32, u32)>, s: u32| {
        if s < 256 {
            byte(s)
        } else {
            rx[(s - 256) as usize]
        }
    };
    let mut muls = 0u64;
    for j in 0..g.left.len() {
        let v = cat(get(&rx, g.left[j]), get(&rx, g.right[j]));
        rx.push(v);
        muls += 2;
    }
    let len = |s: u32| {
        if s < 256 {
            1
        } else {
            g.len[(s - 256) as usize]
        }
    };
    // the block is the root's last (period - cut) bytes, then its first cut
    let cut = (n - cl.t0 as usize % n) % n;
    let mut head = (0u32, 1u32);
    let (mut cur, mut k) = (cl.root, cut as u32);
    while k > 0 {
        if k == len(cur) {
            head = cat(head, get(&rx, cur));
            muls += 2;
            break;
        }
        let j = (cur - 256) as usize;
        if k <= len(g.left[j]) {
            cur = g.left[j];
        } else {
            head = cat(head, get(&rx, g.left[j]));
            muls += 2;
            k -= len(g.left[j]);
            cur = g.right[j];
        }
    }
    let mut tail = (0u32, 1u32);
    let (mut cur, mut m) = (cl.root, (n - cut) as u32);
    while m > 0 {
        if m == len(cur) {
            tail = cat(get(&rx, cur), tail);
            muls += 2;
            break;
        }
        let j = (cur - 256) as usize;
        if m <= len(g.right[j]) {
            cur = g.right[j];
        } else {
            tail = cat(get(&rx, g.right[j]), tail);
            muls += 2;
            m -= len(g.right[j]);
            cur = g.left[j];
        }
    }
    let (r, x) = cat(tail, head);
    muls += 3;
    // the register starts all ones and is complemented at the end
    Some((!(r ^ mulmod(!0, x)), muls))
}

/// The run boundaries of `l` that are also boundaries of the runs' images under LF (0 and n
/// not counted): where the top and bottom partitions of [0, n) coincide
fn coincidences(l: &[u8]) -> u64 {
    let n = l.len();
    let mut cnt = [0u32; 257];
    for &c in l {
        cnt[c as usize + 1] += 1;
    }
    for k in 0..256 {
        cnt[k + 1] += cnt[k];
    }
    let mut image = vec![false; n + 1];
    let mut next = cnt;
    for i in 0..n {
        let c = l[i] as usize;
        if i == 0 || l[i] != l[i - 1] {
            image[next[c] as usize] = true;
        }
        next[c] += 1;
    }
    (1..n).filter(|&i| l[i] != l[i - 1] && image[i]).count() as u64
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1]).expect("read");
    let every: usize = args.get(2).map_or(1, |s| s.parse().unwrap());
    let min: usize = args.get(3).map_or(65536, |s| s.parse().unwrap());
    let rounds: usize = args.get(4).map_or(5, |s| s.parse().unwrap());
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut ll, mut counts, mut tt) = (Vec::new(), [0u32; 256], TVector::new());
    let mut seg = SegScratch::default();
    let mut syms = Vec::new();
    let (mut walked, mut fast, mut out, mut tmp) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut ind, mut g) = (Induction::default(), Grammar::default());
    let (mut first, mut stack) = (Vec::new(), Vec::new());
    let mut kern = Kernel::default();
    let mut i = 0;
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        if ll.len() >= min {
            if i % every == 0 {
                let n = ll.len();
                build(&ll, &counts, &mut tt);
                walked.clear();
                walk(&tt, b.orig_ptr, &mut walked);
                // production's one-thread kernel: build, then the room walk
                let mut t_walk = f64::MAX;
                for _ in 0..rounds {
                    let t = Instant::now();
                    build(&ll, &counts, &mut tt);
                    walk_segments_room(&mut tt, b.orig_ptr, &mut fast, &mut seg);
                    t_walk = t_walk.min(t.elapsed().as_secs_f64());
                }
                assert!(fast == walked);
                let (mut t_runs, mut t_ind, mut t_exp) = (f64::MAX, f64::MAX, f64::MAX);
                let mut c = Counts::default();
                let mut ex = (0, 0, 0);
                let mut crc_muls = 0u64;
                for _ in 0..rounds {
                    c = Counts::default();
                    let t = Instant::now();
                    let (trk, off) = letters(&ll, b.orig_ptr, &mut ind);
                    let t1 = Instant::now();
                    let cl = induce(&mut ind, trk, off, &mut g, &mut c);
                    let t2 = Instant::now();
                    if let Some((crc, muls)) = grammar_crc(&g, &cl, n) {
                        assert!(
                            crc == mibz2::crc::crc32(&walked, k),
                            "the grammar's CRC differs from the bytes'"
                        );
                        crc_muls = muls;
                    }
                    let t2b = Instant::now();
                    ex = write_block(&g, &cl, n, &mut out, &mut tmp, &mut first, &mut stack);
                    let t3 = Instant::now();
                    t_runs = t_runs.min((t1 - t).as_secs_f64());
                    t_ind = t_ind.min((t2 - t1).as_secs_f64());
                    t_exp = t_exp.min((t3 - t2b).as_secs_f64());
                }
                assert!(out == walked, "the grammar's bytes differ from the walk's");
                let (mut k_runs, mut k_ind, mut k_exp) = (f64::MAX, f64::MAX, f64::MAX);
                let mut k_rules = 0;
                for _ in 0..rounds {
                    let t = Instant::now();
                    let (trk, off) = kern.letters(&ll, b.orig_ptr);
                    let t1 = Instant::now();
                    let (word, period, t0, _) = kern.induce(trk, off);
                    let t2 = Instant::now();
                    kern.write(word, period, t0, n, &mut out);
                    let t3 = Instant::now();
                    k_runs = k_runs.min((t1 - t).as_secs_f64());
                    k_ind = k_ind.min((t2 - t1).as_secs_f64());
                    k_exp = k_exp.min((t3 - t2).as_secs_f64());
                    k_rules = kern.rules.len();
                }
                assert!(out == walked, "the kernel's bytes differ from the walk's");
                // rule lengths: counts and total bytes by class (<=2, <=4, <=8, <=16, <=32, <=64, more)
                let mut hist = [(0u64, 0u64); 7];
                for &len in &g.len {
                    let k = match len {
                        0..=2 => 0,
                        3..=4 => 1,
                        5..=8 => 2,
                        9..=16 => 3,
                        17..=32 => 4,
                        33..=64 => 5,
                        _ => 6,
                    };
                    hist[k].0 += 1;
                    hist[k].1 += len as u64;
                }
                let hs: Vec<String> = hist.iter().map(|h| format!("[{},{}]", h.0, h.1)).collect();
                let coin = coincidences(&ll);
                let ns = |t: f64| t * 1e9 / n as f64;
                println!(
                    "{{\"n\":{n},\"runs\":{},\"steps\":{},\"top\":{},\"bottom\":{},\"equal\":{},\"dropped\":{},\"streaks\":{},\"longest\":{},\"left\":{},\"copies\":{},\"copied\":{},\"deepest\":{},\"ns_walk\":{:.3},\"ns_runs\":{:.3},\"ns_induce\":{:.3},\"ns_expand\":{:.3},\"coincide\":{},\"hist\":[{}],\"k_rules\":{},\"k_runs\":{:.3},\"k_induce\":{:.3},\"k_write\":{:.3},\"repeats\":{},\"divisions\":{},\"crc_muls\":{},\"depth\":{},\"widest\":{},\"rounds\":{},\"selfish\":{},\"selfish_steps\":{}}}",
                    c.runs, c.steps, c.top, c.bottom, c.equal, c.dropped, c.streaks, c.longest, c.left,
                    ex.0, ex.1, ex.2, ns(t_walk), ns(t_runs), ns(t_ind), ns(t_exp), coin, hs.join(","),
                    k_rules, ns(k_runs), ns(k_ind), ns(k_exp), c.repeats, c.divisions, crc_muls, c.depth, c.widest, c.rounds, c.selfish, c.selfish_steps
                );
            }
            i += 1;
        }
        syms = b.syms;
    }
}
