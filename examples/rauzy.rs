//! MONODROMY, mission 1 (docs/journal/monodromy/2026-11-10-rauzy-veech.md): the inverse BWT by
//! Rauzy-Veech induction on the LF interval exchange. A witness and a census; nothing in the
//! decoder calls it.
//!
//! ```text
//! rauzy FILE.bz2 [EVERY] [MIN_ROWS] [ROUNDS]     one JSON line a block (stdout)
//! RAUZY_JOIN=1 rauzy ...                         the kernel with its joining pre-pass
//! rauzy --arbitrary [CASES]                      columns that are no BWT, against the walk
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
//! **Three implementations of the induction** (the cells kernel on the letters with the walk's
//! row cut out: a few cells more), each checked against `bwt::walk` on every block:
//! - the reference (`letters`, `induce`, `write_block`): safe, counted, slow;
//! - the list kernel (`Kernel::link`, `induce`, `write`): both orders as linked lists, a step
//!   without a data-dependent branch. Its clock is the entry's first (1.276 of production);
//! - the cells kernel (`Kernel::slots`, `induce_cells`, `write_cells`): a step's cell is a
//!   letter written once; no letter is moved or shortened in memory; the walk's row is a
//!   letter of its own whose mark rides in the words made with it, so no point is followed
//!   and the block is written unrotated. Its clock is the entry's second (0.756).
//!
//! JSON fields (ns a row): `ns_walk` is the column's path (`bwt::build` + the room walk),
//! `ns_rec` the records' path (`runs::fill` + the room walk backward: production's since
//! 0.2.8 for a block `takes`), `ns_mtf` and `ns_front` their fronts; `k_*` are the list
//! kernel's; `c_*` the cells kernel's from the same records (`c_whole` is order + induce +
//! write timed as one span, the order stage reading the records). `par` is the turns the rounds take with 2, 4, 8 and 16
//! workers.

use mibz2::bwt::{build, walk, walk_back_room, walk_segments_room, SegScratch, TVector};
use mibz2::runs::{fill, front, takes_runs, Runs};
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

#[derive(Default)]
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
    /// the steps of each round's segments (a round's segments share no letter)
    work: Vec<u64>,
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
            if c.work.len() <= round as usize {
                c.work.resize(round as usize + 1, 0);
            }
            c.work[round as usize] += seg_steps;
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
    /// the arena's bytes in use
    top: usize,
    rules: Vec<Rule>,
    pos: Vec<u32>,
    tmp: Vec<u8>,
    // the letters as the block gives them, and the joining of whole runs before the induction
    lam: Vec<u32>,
    byte: Vec<u8>,
    link: Vec<u32>,
    heir: Vec<u32>,
    flag: Vec<u8>,
    chain: Vec<u32>,
    hword: Vec<u32>,
    hlen: Vec<u32>,
    id: Vec<u32>,
    // the induction by cells: its one buffer, the buffer's regions, the rules' end
    buf: Vec<u8>,
    lay: Layout,
    rule_end: usize,
    /// its letters: the pieces, and one more if a piece's image is cut at the walk's row
    letters: usize,
}

/// A letter's place in one order, for the induction by cells: 24 bytes. The two orders are
/// two regions of these in one buffer; a letter has one slot in each. The orders are only
/// ever read from their ends, so a slot is read once, when the induction's front reaches it.
///
/// | bytes | |
/// |---|---|
/// | 0..4 | the slot before it in its order (an offset in the buffer) |
/// | 4..8 | the letter's length |
/// | 8..12 | its word's length |
/// | 12..16 | its word: an offset in the buffer, of bytes if the length is at most SMALL, else of a rule |
/// | 16..24 | the letter's slot in the other order |
const SLOT: usize = 24;
/// How many slots on from an order's last letter the induction by cells fetches for
const FAR: usize = 12;
/// The buffer's first bytes, unused: a read `FAR` slots before an order's first stays inside
const PAD: usize = 4096;
/// The mark, in a word's length: the word holds the walk's row
const MARK: u32 = 1 << 31;
/// Where the block begins in the cycle's word, from the walk's row's place
const SHIFT: usize = 1;

/// The regions of the induction by cells' one buffer, as offsets in it.
#[derive(Clone, Copy, Default)]
struct Layout {
    /// the top order's slots
    tops: usize,
    /// the bottom order's slots
    bots: usize,
    /// the words kept as bytes: 16 bytes of slack, the 256 single bytes, 16 more, the rest
    arena: usize,
    /// the words kept as rules, 16 bytes each
    rules: usize,
}

#[inline(always)]
unsafe fn rd64(base: *const u8, at: usize) -> u64 {
    unsafe { (base.add(at) as *const u64).read_unaligned() }
}

#[inline(always)]
unsafe fn rd32(base: *const u8, at: usize) -> u32 {
    unsafe { (base.add(at) as *const u32).read_unaligned() }
}

#[inline(always)]
unsafe fn wr64(base: *mut u8, at: usize, v: u64) {
    unsafe { (base.add(at) as *mut u64).write_unaligned(v) }
}

#[inline(always)]
unsafe fn wr32(base: *mut u8, at: usize, v: u32) {
    unsafe { (base.add(at) as *mut u32).write_unaligned(v) }
}

/// `flag`: the letter is joined into its chain's head (gone from the top order)
const JOINED: u8 = 1;
/// `flag`: the letter's image is a joined letter's interval (its bottom place is gone)
const IMAGE_GONE: u8 = 2;
/// `flag`: some letter is sent exactly onto this one
const TARGET: u8 = 4;

#[inline(always)]
fn prefetch<T>(p: *const T) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_mm_prefetch(p as *const i8, core::arch::x86_64::_MM_HINT_T0);
    }
    #[cfg(not(target_arch = "x86_64"))]
    let _ = p;
}

/// 16 bytes from `src` to `dst`, whatever the word's length (both have 16 bytes there). The
/// two may overlap (a word just made is a factor of the next): the 16 are read, then written
#[inline(always)]
unsafe fn copy16(dst: *mut u8, src: *const u8) {
    unsafe { (dst as *mut [u8; 16]).write_unaligned((src as *const [u8; 16]).read_unaligned()) }
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
    /// The runs of `l` (a length and a byte each); the run holding row `p` and p's offset in
    /// it. This is what a front that emits run records gives without the column.
    fn runs(&mut self, l: &[u8], p: u32) -> (u32, u32) {
        let n = l.len();
        self.lam.clear();
        self.byte.clear();
        let (mut trk, mut off) = (0u32, 0u32);
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
                trk = self.lam.len() as u32;
                off = p - i as u32;
            }
            self.lam.push((j - i) as u32);
            self.byte.push(c);
            i = j;
        }
        (trk, off)
    }

    /// The bottom order, as an array: the runs sorted stably by byte
    fn sort(&mut self) {
        let r = self.lam.len();
        let mut cnt = [0u32; 257];
        for &c in &self.byte {
            cnt[c as usize + 1] += 1;
        }
        for k in 0..256 {
            cnt[k + 1] += cnt[k];
        }
        self.order.clear();
        self.order.resize(r, 0);
        for i in 0..r {
            let c = self.byte[i] as usize;
            self.order[cnt[c] as usize] = i as u32;
            cnt[c] += 1;
        }
    }

    /// The arena and the rules, empty
    fn reset(&mut self) {
        self.rules.clear();
        self.arena.clear();
        self.arena
            .resize((BYTES as usize + 256 + 64).max(self.arena.capacity()), 0);
        for b in 0..256 {
            self.arena[BYTES as usize + b] = b as u8;
        }
        self.top = BYTES as usize + 256;
    }

    /// The runs as the exchange's letters, both orders linked: every step is then the
    /// induction's. Returns the last letters on top and bottom.
    fn link(&mut self) -> (u32, u32) {
        self.reset();
        let r = self.lam.len();
        let none = r as u32;
        self.nodes.clear();
        for i in 0..r as u32 {
            self.nodes.push(Node {
                lam: self.lam[i as usize],
                len: 1,
                word: BYTES + self.byte[i as usize] as u32,
                _pad: 0,
                prev: [i.wrapping_sub(1), 0],
                next: [i + 1, 0],
            });
        }
        self.nodes[0].prev[0] = none;
        for k in 0..r {
            let i = self.order[k] as usize;
            self.nodes[i].prev[1] = if k > 0 { self.order[k - 1] } else { none };
            self.nodes[i].next[1] = if k + 1 < r { self.order[k + 1] } else { none };
        }
        self.nodes.push(Node::default());
        (none - 1, self.order[r - 1])
    }

    /// Before the induction: every run that LF sends exactly onto another run (its image is
    /// that run's interval) is joined with it. Such a pair is a segment of one top and one
    /// bottom interval, and the induction would spend an equal step on it; a chain of them
    /// h -> x2 -> ... -> xk is one letter, h's interval sent to xk's image, its word the
    /// chain's bytes from xk back to h. Found by one merge of the two tilings' boundaries.
    /// The letters left are then numbered afresh, in top order, and their orders linked: the
    /// induction's nodes are theirs alone.
    ///
    /// Returns (the tracked letter, the last letters on top and bottom, the tracked point's
    /// level, the letters joined away).
    fn contract(&mut self, mut trk: u32) -> (u32, u32, u32, u32, u64) {
        let r = self.lam.len();
        self.reset();
        let mut top = self.top;
        self.link.clear();
        self.link.resize(r, NIL);
        self.heir.clear();
        self.heir.resize(r, NIL);
        self.flag.clear();
        self.flag.resize(r, 0);
        self.hword.clear();
        self.hword.resize(r, 0);
        self.hlen.clear();
        self.hlen.resize(r, 0);
        // top letter i begins at pt, the image of bottom letter order[k] at pb
        let (mut i, mut k, mut pt, mut pb) = (0usize, 0usize, 0u32, 0u32);
        while i < r && k < r {
            let b = self.order[k] as usize;
            let (la, lb) = (self.lam[i], self.lam[b]);
            if pt == pb && la == lb && i != b {
                self.link[b] = i as u32;
                self.flag[i] |= TARGET;
            }
            let (et, eb) = (pt + la, pb + lb);
            if et <= eb {
                pt = et;
                i += 1;
            }
            if eb <= et {
                pb = eb;
                k += 1;
            }
        }
        // the chains, each from the letter nothing is sent onto (a closed chain, a block of
        // one repeated word, is left to the induction)
        let (mut v, mut joined) = (0u32, 0u64);
        for h in 0..r {
            if self.link[h] == NIL || self.flag[h] & TARGET != 0 {
                continue;
            }
            self.chain.clear();
            let mut x = h as u32;
            loop {
                self.chain.push(x);
                x = self.link[x as usize];
                if x == NIL {
                    break;
                }
            }
            let k = self.chain.len();
            joined += k as u64 - 1;
            if top + k + 64 > self.arena.len() {
                let grown = (self.arena.len() * 2).max(top + k + 64);
                self.arena.resize(grown, 0);
            }
            for (j, &x) in self.chain.iter().enumerate() {
                let xi = x as usize;
                self.arena[top + k - 1 - j] = self.byte[xi];
                if j > 0 {
                    self.flag[xi] |= JOINED;
                    if x == trk {
                        trk = h as u32;
                        v = j as u32;
                    }
                }
                if j + 1 < k {
                    self.flag[xi] |= IMAGE_GONE;
                }
            }
            self.heir[self.chain[k - 1] as usize] = h as u32;
            // the word: bytes if short, else 16-byte pieces under a comb of rules
            let (mut word, mut len) = (top as u32, k.min(SMALL as usize) as u32);
            let mut o = len as usize;
            while o < k {
                let piece = (k - o).min(SMALL as usize) as u32;
                self.rules.push(Rule {
                    left: word,
                    right: (top + o) as u32,
                    llen: len,
                    len: len + piece,
                });
                word = self.rules.len() as u32 - 1;
                len += piece;
                o += piece as usize;
            }
            self.hword[h] = word;
            self.hlen[h] = len;
            top += k;
        }
        self.top = top;
        // the letters left, numbered in top order; the node past the last stands for "none"
        self.nodes.clear();
        self.id.clear();
        self.id.resize(r, NIL);
        for i in 0..r {
            if self.flag[i] & JOINED != 0 {
                continue;
            }
            let k = self.nodes.len() as u32;
            self.id[i] = k;
            let (word, len) = if self.hlen[i] != 0 {
                (self.hword[i], self.hlen[i])
            } else {
                (BYTES + self.byte[i] as u32, 1)
            };
            self.nodes.push(Node {
                lam: self.lam[i],
                len,
                word,
                _pad: 0,
                prev: [k.wrapping_sub(1), 0],
                next: [k + 1, 0],
            });
        }
        let none = self.nodes.len() as u32;
        self.nodes[0].prev[0] = none;
        self.nodes.push(Node::default());
        // the bottom order: a letter whose image is gone has no place; a chain's head sits
        // where its last letter sat
        let mut last = none;
        for k in 0..r {
            let x = self.order[k] as usize;
            if self.flag[x] & IMAGE_GONE != 0 {
                continue;
            }
            let y = if self.flag[x] & JOINED != 0 {
                self.heir[x]
            } else {
                x as u32
            };
            let y = self.id[y as usize];
            self.nodes[y as usize].prev[1] = last;
            self.nodes[last as usize].next[1] = y;
            last = y;
        }
        (self.id[trk as usize], none - 1, last, v, joined)
    }

    /// The cells kernel's first stage, from the front's run records (`byte << 8 | length`,
    /// a run longer than 127 rows cut into several: `runs::front`'s format): the letters as
    /// slots, both orders, in the one buffer the induction by cells works in. The top order
    /// is the runs as they lie, the bottom order is sorted stably by byte, so that the
    /// original letters of each order lie in memory in the order the induction reads them.
    /// Two passes over the records; the sort is the first one's counting.
    ///
    /// A record that goes on with the byte of the one before it is the same run. Row `p`,
    /// the walk's, is cut out as a letter of its own. Its word is marked, and the mark rides
    /// in the length of every word made with it: such a word is kept as a rule whatever its
    /// length, so the row's place in the cycle's word is read off the rules afterwards, and
    /// the induction follows no point.
    ///
    /// The buffer's regions are sized by `n` alone (a cell removes a row at least, so there
    /// are at most n cells), and its pages are touched only where the induction writes.
    #[inline(never)]
    fn slots(&mut self, n: usize, recs: &[u16], p: u32) {
        let cap = (2 * n + 8) * SLOT;
        let lay = Layout {
            tops: PAD,
            bots: PAD + cap,
            arena: PAD + 2 * cap,
            rules: PAD + 2 * cap + ((BYTES as usize + 256 + 16 * (n + 2) + 64 + 15) & !15),
        };
        let total = lay.rules + 16 * (n + 4);
        assert!(total < u32::MAX as usize && n < MARK as usize && (p as usize) < n);
        if self.buf.len() < total {
            // zeroed pages from the allocator: none is touched here
            self.buf = vec![0u8; total];
        }
        self.lay = lay;
        for b in 0..256 {
            self.buf[lay.arena + BYTES as usize + b] = b as u8;
        }
        // a byte's letters; the record that holds the walk's row, and its run's first row
        let mut cnt = [0u32; 257];
        let (mut prev, mut row, mut start) = (256usize, 0u32, 0u32);
        let (mut at, mut at_start) = (0usize, 0u32);
        let mut empty = false;
        for (i, &rec) in recs.iter().enumerate() {
            let (c, len) = ((rec >> 8) as usize, (rec & 0xFF) as u32);
            if c != prev {
                cnt[c + 1] += 1;
                start = row;
            }
            prev = c;
            if p.wrapping_sub(row) < len {
                (at, at_start) = (i, start);
            }
            empty |= len == 0;
            row += len;
        }
        // (every letter a row at least: so at most n letters and n - 1 cells, which is what
        // the buffer's regions are sized for)
        assert!(
            row as usize == n && !empty,
            "the records do not tile the block"
        );
        // that run: records ia..ib, `total` rows, the walk's row `off` rows in
        let cz = (recs[at] >> 8) as usize;
        let (mut ia, mut ib) = (at, at + 1);
        while ia > 0 && (recs[ia - 1] >> 8) as usize == cz {
            ia -= 1;
        }
        while ib < recs.len() && (recs[ib] >> 8) as usize == cz {
            ib += 1;
        }
        let total: u32 = recs[ia..ib].iter().map(|&x| (x & 0xFF) as u32).sum();
        let off = p - at_start;
        cnt[cz + 1] += (off > 0) as u32 + (total - off > 1) as u32;
        // a byte's next bottom slot, as its offset in the buffer
        for k in 0..256 {
            cnt[k + 1] += cnt[k];
            cnt[k] = (lay.bots + cnt[k] as usize * SLOT) as u32;
        }
        self.letters = cnt[256] as usize;
        let base = self.buf.as_mut_ptr();
        let first = lay.arena as u64 + BYTES as u64;
        let mut t = PAD;
        // SAFETY: the letters number at most the records and two, and a record is a row at
        // least: their slots lie inside each order's region
        unsafe {
            macro_rules! letter {
                ($lam:expr, $c:expr) => {{
                    let (lam, c): (u64, usize) = ($lam as u64, $c);
                    let k = *cnt.get_unchecked(c) as usize;
                    *cnt.get_unchecked_mut(c) = (k + SLOT) as u32;
                    let lw = 1 | (first + c as u64) << 32;
                    // (the slot before each order's first is set after the pass)
                    wr64(base, t, t.wrapping_sub(SLOT) as u32 as u64 | lam << 32);
                    wr64(base, t + 8, lw);
                    wr64(base, t + 16, k as u64);
                    wr64(base, k, k.wrapping_sub(SLOT) as u32 as u64 | lam << 32);
                    wr64(base, k + 8, lw);
                    wr64(base, k + 16, t as u64);
                    t += SLOT;
                }};
            }
            let nrec = recs.len();
            let mut i = 0;
            while i < nrec {
                if i == ia {
                    // the walk's row's run: what is before the row, the row, what is after
                    if off > 0 {
                        letter!(off, cz);
                    }
                    let tz = t;
                    letter!(1u32, cz);
                    let kz = rd64(base, tz + 16) as usize;
                    wr32(base, tz + 8, 1 | MARK);
                    wr32(base, kz + 8, 1 | MARK);
                    if total - off > 1 {
                        letter!(total - off - 1, cz);
                    }
                    i = ib;
                    continue;
                }
                let rec = *recs.get_unchecked(i);
                let c = (rec >> 8) as usize;
                let mut lam = (rec & 0xFF) as u32;
                i += 1;
                while i < nrec && (*recs.get_unchecked(i) >> 8) as usize == c {
                    lam += (*recs.get_unchecked(i) & 0xFF) as u32;
                    i += 1;
                }
                letter!(lam, c);
            }
            // (the slot before an order's first is never followed: one letter is marked, it
            // is never dropped, so the induction returns before it could leave an order;
            // these two only keep the value inside the buffer)
            wr32(base, PAD, PAD as u32);
            wr32(base, lay.bots, lay.bots as u32);
        }
    }

    /// The induction by cells. The same steps as `induce`, kept another way, to the cycle of
    /// the walk's row: the closed letter whose word carries the mark.
    ///
    /// The last letter on top, A, and the last on the bottom, B, are held with what is left
    /// of each (ra, rb). A step is the cell at the end of both: min(ra, rb) points, word
    /// (A's)(B's). It is a letter of its own, written once and never changed: on top it
    /// lies inside B's place, on the bottom inside A's image, each time to the left of the
    /// cells already cut from the same letter. The first cell cut from a letter takes that
    /// letter's slot (so whatever pointed at the letter now points at its rightmost cell);
    /// each later one takes a fresh slot, and the slot before the letter moves along with
    /// the promise of that fresh slot. Nothing is shortened in memory and no letter is
    /// moved: a step writes two slots and one word, and reads one or two.
    ///
    /// Every offset is a byte offset in the one buffer: a slot, a word kept as bytes, a
    /// word kept as a rule. A rule is its two factors as their slots held them: (length,
    /// word) twice.
    ///
    /// Returns (the cycle's word, its length with the mark, cells). `slots` has run on this
    /// kernel for the block, and nothing has changed since.
    #[inline(never)]
    fn induce_cells<const COUNT: bool>(&mut self) -> (u32, u32, u64) {
        const M: u64 = 0xFFFF_FFFF;
        let rr = self.letters;
        let lay = self.lay;
        assert!(
            rr >= 1 && self.buf.len() >= lay.rules + 80,
            "`slots` has not run"
        );
        let base = self.buf.as_mut_ptr();
        // SAFETY: `slots` sized every region for n cells; a slot's offset is a letter's
        // slot or a fresh one below the region's end; a word's offset is in the arena or
        // the rules, with 16 bytes after it inside the buffer.
        unsafe {
            let mut top = lay.arena + BYTES as usize + 256 + 16;
            let mut rp = lay.rules;
            // fresh slots: one a cell at most, on each side
            let (ft0, fb0) = (lay.tops + rr * SLOT, lay.bots + rr * SLOT);
            let (mut ft, mut fb) = (ft0, fb0);
            // the times both letters were read whole: once at the start, once after each
            // cell that ended both, once after each closed letter left behind
            let (mut whole, mut dropped) = (0u64, 0u64);
            // the slot before each letter in its order; what is left of the letter; its
            // (length, word); the slot the next cell cut from it takes in the other order
            let (mut na, mut nb);
            let (mut ra, mut rb);
            let (mut aw, mut bw);
            let (mut bs, mut ts);
            let (mut sa, mut sb) = (ft0 - SLOT, fb0 - SLOT);
            // the letter `FAR` slots on in an order, as memory has it now: its slot in
            // the other order is where its cells will be written, and its word is read then
            // (what is read there may be a slot not yet written: the address is only hinted)
            macro_rules! ahead {
                ($s:expr) => {{
                    let far = $s - FAR * SLOT;
                    prefetch(base.wrapping_add(rd32(base, far + 16) as usize));
                    prefetch(base.wrapping_add(rd32(base, far + 12) as usize));
                }};
            }
            'fresh: loop {
                // both letters whole
                na = rd32(base, sa) as usize;
                ra = rd32(base, sa + 4) as u64;
                aw = rd64(base, sa + 8);
                bs = rd64(base, sa + 16) as usize;
                nb = rd32(base, sb) as usize;
                rb = rd32(base, sb + 4) as u64;
                bw = rd64(base, sb + 8);
                ts = rd64(base, sb + 16) as usize;
                if COUNT {
                    whole += 1;
                }
                ahead!(sa);
                ahead!(sb);
                if bs == sb {
                    // one letter, last in both orders: its points' cycles are closed
                    if aw as u32 & MARK != 0 {
                        self.rule_end = rp;
                        let cells = (whole.max(1 + dropped) - 1 - dropped)
                            + ((ft - ft0) / SLOT) as u64
                            + ((fb - fb0) / SLOT) as u64;
                        return ((aw >> 32) as u32, aw as u32, cells);
                    }
                    dropped += 1;
                    (sa, sb) = (na, nb);
                    continue 'fresh;
                }
                loop {
                    // the cell's word, A's then B's: as bytes, and as a rule; one is kept
                    let (la, lb) = (aw & M, bw & M);
                    let len = la + lb;
                    copy16(base.add(top), base.add((aw >> 32) as usize));
                    copy16(
                        base.add(top + la.min(16) as usize),
                        base.add((bw >> 32) as usize),
                    );
                    wr64(base, rp, aw);
                    wr64(base, rp + 8, bw);
                    let small = len <= SMALL as u64;
                    let word = if small { top } else { rp };
                    top += if small { len as usize } else { 0 };
                    rp += if small { 0 } else { 16 };
                    let cw = len | (word as u64) << 32;
                    if ra == rb {
                        // both letters end: the cell is the last cut from each
                        wr32(base, ts + 4, ra as u32);
                        wr64(base, ts + 8, cw);
                        wr64(base, ts + 16, bs as u64);
                        wr32(base, bs + 4, ra as u32);
                        wr64(base, bs + 8, cw);
                        wr64(base, bs + 16, ts as u64);
                        (sa, sb) = (na, nb);
                        continue 'fresh;
                    } else if ra > rb {
                        // B ends; A goes on, and its next cell is promised a fresh slot
                        wr32(base, ts + 4, rb as u32);
                        wr64(base, ts + 8, cw);
                        wr64(base, ts + 16, bs as u64);
                        wr32(base, fb, rd32(base, bs));
                        wr32(base, bs, fb as u32);
                        wr32(base, bs + 4, rb as u32);
                        wr64(base, bs + 8, cw);
                        wr64(base, bs + 16, ts as u64);
                        bs = fb;
                        fb += SLOT;
                        ra -= rb;
                        // (read after the cell is written: it may be that cell)
                        let s = nb;
                        ahead!(s);
                        nb = rd32(base, s) as usize;
                        rb = rd32(base, s + 4) as u64;
                        bw = rd64(base, s + 8);
                        ts = rd64(base, s + 16) as usize;
                    } else {
                        // A ends; B goes on
                        wr32(base, bs + 4, ra as u32);
                        wr64(base, bs + 8, cw);
                        wr64(base, bs + 16, ts as u64);
                        wr32(base, ft, rd32(base, ts));
                        wr32(base, ts, ft as u32);
                        wr32(base, ts + 4, ra as u32);
                        wr64(base, ts + 8, cw);
                        wr64(base, ts + 16, bs as u64);
                        ts = ft;
                        ft += SLOT;
                        rb -= ra;
                        let s = na;
                        ahead!(s);
                        na = rd32(base, s) as usize;
                        ra = rd32(base, s + 4) as u64;
                        aw = rd64(base, s + 8);
                        bs = rd64(base, s + 16) as usize;
                    }
                }
            }
        }
    }

    /// The block from the induction by cells' word (`word`, of marked length `len`), into
    /// `out`. The block begins `SHIFT` bytes after the walk's row's place in the word, and
    /// is written from there on. The marked rules are walked from the word down to the row
    /// three times: for the row's place; to write each short factor without the mark where
    /// it then lies and note where each long one lies; and, after `write`'s two passes
    /// (where each rule first lies, last rule to first; then its bytes, first to last), to
    /// copy each long factor that lies again later from its first place. If the cycle is
    /// shorter than the block, the block is its copies.
    ///
    /// `word` and `len` are what `induce_cells` returned for the block `slots` was given.
    #[inline(never)]
    fn write_cells(&mut self, word: u32, len: u32, n: usize, out: &mut Vec<u8>) {
        let p = (len & !MARK) as usize;
        // (kept at its length from block to block: only the slack is written here)
        out.resize(n + 32, 0);
        let dst = out.as_mut_ptr();
        let base = self.buf.as_ptr();
        let ro = self.lay.rules;
        // SAFETY: a rule's factors are words written before it; a word kept as bytes has
        // 16 bytes at its offset; a position is below the period; `out` has 16 bytes past
        // the period
        unsafe {
            let rule = |w: usize| (rd64(base, w), rd64(base, w + 8));
            let nr = (self.rule_end - ro) / 16;
            self.pos.clear();
            self.pos.resize(nr, NIL);
            let pos = self.pos.as_mut_ptr();
            // the row's place in the word
            let (mut w, mut j) = (word as usize, 0usize);
            while w >= ro {
                let (a, b) = rule(w);
                if a as u32 & MARK != 0 {
                    w = (a >> 32) as usize;
                } else {
                    j += a as u32 as usize;
                    w = (b >> 32) as usize;
                }
            }
            let start = (j + SHIFT) % p;
            // the marked rules, from the word down to the row
            let (mut w, mut q) = (word as usize, 0usize);
            while w >= ro {
                let (a, b) = rule(w);
                let la = (a as u32 & !MARK) as usize;
                let (plain, at, next) = if a as u32 & MARK != 0 {
                    (b, q + la, a)
                } else {
                    (a, q, b)
                };
                let d = (at + p - start) % p;
                let lx = plain as u32 as usize;
                if lx <= SMALL as usize {
                    copy_small(dst.add(d), base.add((plain >> 32) as usize), lx);
                } else {
                    let x = pos.add(((plain >> 32) as usize - ro) / 16);
                    *x = (*x).min(d as u32);
                }
                if a as u32 & MARK == 0 {
                    q += la;
                }
                w = (next >> 32) as usize;
            }
            *dst.add((q + p - start) % p) = *base.add(w);
            // where each rule first lies: a rule's factors lie no later than inside it
            for j in (0..nr).rev() {
                let q = *pos.add(j);
                if q == NIL {
                    continue;
                }
                let (a, b) = rule(ro + 16 * j);
                if a as u32 > SMALL {
                    let x = pos.add(((a >> 32) as usize - ro) / 16);
                    *x = (*x).min(q);
                }
                if b as u32 > SMALL {
                    let x = pos.add(((b >> 32) as usize - ro) / 16);
                    *x = (*x).min(q + a as u32);
                }
            }
            // its bytes: each factor's from the arena, or from where the factor first lies
            for j in 0..nr {
                let q = *pos.add(j);
                if q == NIL {
                    continue;
                }
                let (a, b) = rule(ro + 16 * j);
                let (ll, rr) = (a as u32 as usize, b as u32 as usize);
                let d = dst.add(q as usize);
                if ll <= SMALL as usize {
                    copy_small(d, base.add((a >> 32) as usize), ll);
                } else {
                    let f = *pos.add(((a >> 32) as usize - ro) / 16);
                    if f != q {
                        copy_long(d, dst.add(f as usize), ll);
                    }
                }
                let d = d.add(ll);
                if rr <= SMALL as usize {
                    copy_small(d, base.add((b >> 32) as usize), rr);
                } else {
                    let f = *pos.add(((b >> 32) as usize - ro) / 16);
                    if f != q + ll as u32 {
                        copy_long(d, dst.add(f as usize), rr);
                    }
                }
            }
            // the marked rules again: a long factor that lies at a later place than its
            // first is copied from there (the first walk only noted where each lies; a
            // marked rule has no place of its own, so the passes above never copy for it).
            // The first place is the least and occurrences of one word are apart, so the
            // source is whole and does not reach the place written.
            let (mut w, mut q) = (word as usize, 0usize);
            while w >= ro {
                let (a, b) = rule(w);
                let la = (a as u32 & !MARK) as usize;
                let (plain, at, next) = if a as u32 & MARK != 0 {
                    (b, q + la, a)
                } else {
                    (a, q, b)
                };
                let d = (at + p - start) % p;
                let lx = plain as u32 as usize;
                if lx > SMALL as usize {
                    let f = *pos.add(((plain >> 32) as usize - ro) / 16) as usize;
                    if f != d {
                        copy_long(dst.add(d), dst.add(f), lx);
                    }
                }
                if a as u32 & MARK == 0 {
                    q += la;
                }
                w = (next >> 32) as usize;
            }
        }
        // a cycle shorter than the block: the block is its copies
        let mut have = p.min(n);
        while have < n {
            let more = have.min(n - have);
            out.copy_within(..more, have);
            have += more;
        }
        out.truncate(n);
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
    #[inline(never)]
    fn induce(
        &mut self,
        mut trk: u32,
        mut off: u32,
        mut tl: u32,
        mut bl: u32,
        mut v: u32,
    ) -> (u32, u32, u32, u64) {
        let r = self.nodes.len() - 1;
        let none = r as u32;
        let mut top = self.top;
        let nd = self.nodes.as_mut_ptr();
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
        let mut steps = 0u64;
        // (a rule is written every step, and kept only if its word is long)
        self.rules.reserve(r + 1);
        let mut nr = self.rules.len();
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
    #[inline(never)]
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

/// The column's run records in `runs::front`'s format (`byte << 8 | length`, cut at 127 rows).
/// The decoder's own records are private to `runs::Runs`, so the cells kernel is fed these:
/// the same records, made here from the column, outside every clock.
fn records(l: &[u8], recs: &mut Vec<u16>) {
    recs.clear();
    let mut i = 0;
    while i < l.len() {
        let c = l[i];
        let mut j = i + 1;
        while j < l.len() && l[j] == c {
            j += 1;
        }
        let mut left = j - i;
        while left > 0 {
            let len = left.min(127);
            recs.push((c as u16) << 8 | len as u16);
            left -= len;
        }
        i = j;
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

/// The induction's step count without the induction, and the surface's cone points.
///
/// **The count.** Cut LF's cycle after every run start: r stretches x, LF x, ..., each ending
/// at a run start. The prefixes [0, m) the induction passes through are exactly m = y for the
/// rows y >= 1 that are a minimum-so-far of a stretch read forward (a first entry below, from
/// an image start: a bottom boundary of the induced map) or read backward (from a run start:
/// a top boundary). Read forward but not backward: a step the top wins. Backward but not
/// forward: the bottom wins. Both, the stretch's least row: an equal step. Row 0 is no step.
///
/// **The cone points.** With tau the row shift, the cycles of LF tau LF^-1 tau^-1 that are not
/// fixed points: (points moved, cycles).
///
/// None if LF is not one cycle.
fn admissible(l: &[u8]) -> Option<([u64; 3], [u64; 2])> {
    let n = l.len();
    let mut next = [0u32; 256];
    {
        let mut cnt = [0u32; 256];
        for &c in l {
            cnt[c as usize] += 1;
        }
        let mut sum = 0;
        for c in 0..256 {
            next[c] = sum;
            sum += cnt[c];
        }
    }
    let mut lf = vec![0u32; n];
    let mut inv = vec![0u32; n];
    for i in 0..n {
        let c = l[i] as usize;
        lf[i] = next[c];
        inv[next[c] as usize] = i as u32;
        next[c] += 1;
    }
    let start = |i: usize| i == 0 || l[i] != l[i - 1];
    // row 0 is a run start: the stretches are (LF 0, ..., next run start], and so on round
    let (mut top, mut bottom, mut equal) = (0u64, 0u64, 0u64);
    let mut suffix: Vec<u32> = Vec::new();
    let (mut x, mut low, mut prefix) = (lf[0], u32::MAX, 0u64);
    for step in 1..=n {
        if x < low {
            low = x;
            prefix += 1;
        }
        // (the rows left on the stack are the minima of the stretch read backward)
        while suffix.last().is_some_and(|&y| y > x) {
            suffix.pop();
        }
        suffix.push(x);
        if start(x as usize) {
            top += prefix - 1;
            bottom += suffix.len() as u64 - 1;
            equal += 1;
            suffix.clear();
            low = u32::MAX;
            prefix = 0;
        }
        if x == 0 && step < n {
            return None;
        }
        x = lf[x as usize];
    }
    if x != lf[0] {
        return None;
    }
    // (the stretch that ends at row 0 gave an "equal" that is no step)
    equal -= 1;
    let comm = |x: usize| lf[(inv[(x + n - 1) % n] as usize + 1) % n] as usize;
    let mut seen = vec![false; n];
    let (mut moved, mut cycles) = (0u64, 0u64);
    for x in 0..n {
        if seen[x] || comm(x) == x {
            continue;
        }
        cycles += 1;
        let mut y = x;
        while !seen[y] {
            seen[y] = true;
            moved += 1;
            y = comm(y);
        }
    }
    Some(([top, bottom, equal], [moved, cycles]))
}

/// The run boundaries of `l` that are also boundaries of the runs' images under LF (0 and n
/// not counted): where the top and bottom partitions of [0, n) coincide. With them, the
/// segments between two coincidences by shape: (one run, one image) = a run sent exactly onto
/// a run; (one run, several images); (several runs, one image); the rest.
fn coincidences(l: &[u8]) -> (u64, [u64; 4]) {
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
    let (mut coin, mut shapes) = (0u64, [0u64; 4]);
    // the top and bottom intervals begun since the last coincidence
    let (mut tops, mut bots) = (1u32, 1u32);
    for i in 1..=n {
        let (t, b) = (i == n || l[i] != l[i - 1], i == n || image[i]);
        if t && b {
            coin += (i < n) as u64;
            shapes[match (tops, bots) {
                (1, 1) => 0,
                (1, _) => 1,
                (_, 1) => 2,
                _ => 3,
            }] += 1;
            tops = 1;
            bots = 1;
        } else {
            tops += t as u32;
            bots += b as u32;
        }
    }
    (coin, shapes)
}

/// `rauzy --arbitrary CASES`: columns that are the BWT of nothing. A decoder is handed any
/// bytes as L and any row below n as the walk's start; LF then has cycles of several kinds,
/// and the induction meets a last letter sent onto itself while others are left (the drop).
/// Every case is checked against `bwt::walk`, by the reference and by the kernel both ways.
fn arbitrary(cases: usize) {
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut rnd = move |m: u64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed % m
    };
    let (mut tt, mut walked, mut out, mut tmp) =
        (TVector::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut ind, mut g, mut kern) = (Induction::default(), Grammar::default(), Kernel::default());
    let (mut first, mut stack) = (Vec::new(), Vec::new());
    let mut recs = Vec::new();
    let (mut dropped, mut periodic, mut many) = (0u64, 0u64, 0u64);
    for case in 0..cases {
        let n = 1 + rnd(if case % 16 == 0 { 3000 } else { 48 }) as usize;
        let sigma = [1, 2, 2, 3, 4, 26, 256][rnd(7) as usize];
        // (runs of a letter now and then, so that letters have lengths above one)
        let mut l = Vec::with_capacity(n);
        while l.len() < n {
            let c = rnd(sigma) as u8;
            let long = rnd(3) == 0;
            let copies = 1 + rnd(if long { 9 } else { 1 });
            for _ in 0..copies {
                l.push(c);
            }
        }
        l.truncate(n);
        let p = rnd(n as u64) as u32;
        let mut counts = [0u32; 256];
        for &c in &l {
            counts[c as usize] += 1;
        }
        build(&l, &counts, &mut tt);
        walked.clear();
        walk(&tt, p, &mut walked);
        let mut c = Counts::default();
        let (trk, off) = letters(&l, p, &mut ind);
        let cl = induce(&mut ind, trk, off, &mut g, &mut c);
        write_block(&g, &cl, n, &mut out, &mut tmp, &mut first, &mut stack);
        assert!(out == walked, "reference: case {case}, L = {l:?}, row {p}");
        dropped += (c.dropped > 0) as u64;
        periodic += ((cl.period as usize) < n) as u64;
        many += admissible(&l).is_none() as u64;
        for join in [false, true] {
            let (trk, off) = kern.runs(&l, p);
            kern.sort();
            let (trk, tl, bl, v) = if join {
                let (trk, tl, bl, v, _) = kern.contract(trk);
                (trk, tl, bl, v)
            } else {
                let (tl, bl) = kern.link();
                (trk, tl, bl, 0)
            };
            let (word, period, t0, _) = kern.induce(trk, off, tl, bl, v);
            kern.write(word, period, t0, n, &mut out);
            assert!(
                out == walked,
                "kernel (join {join}): case {case}, L = {l:?}, row {p}"
            );
        }
        records(&l, &mut recs);
        kern.slots(n, &recs, p);
        let (word, len, _) = kern.induce_cells::<false>();
        // (every byte wrong before this writer runs: `out` held the list kernel's bytes,
        // and a byte left unwritten would pass for written)
        out.clear();
        out.extend(walked.iter().map(|&b| !b));
        kern.write_cells(word, len, n, &mut out);
        assert!(out == walked, "cells: case {case}, L = {l:?}, row {p}");
    }
    println!(
        "{{\"cases\":{cases},\"several_cycles\":{many},\"dropped_a_letter\":{dropped},\"cycle_shorter_than_n\":{periodic}}}"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--arbitrary") {
        return arbitrary(args.get(2).map_or(100_000, |s| s.parse().unwrap()));
    }
    let data = std::fs::read(&args[1]).expect("read");
    let every: usize = args.get(2).map_or(1, |s| s.parse().unwrap());
    let min: usize = args.get(3).map_or(65536, |s| s.parse().unwrap());
    let rounds: usize = args.get(4).map_or(5, |s| s.parse().unwrap());
    // RAUZY_JOIN=1: the kernel joins the runs sent exactly onto runs before it inducts
    let join = std::env::var_os("RAUZY_JOIN").is_some();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut ll, mut counts, mut tt) = (Vec::new(), [0u32; 256], TVector::new());
    let mut seg = SegScratch::default();
    let mut syms = Vec::new();
    let (mut walked, mut fast, mut out, mut tmp) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut ind, mut g) = (Induction::default(), Grammar::default());
    let (mut first, mut stack) = (Vec::new(), Vec::new());
    let mut kern = Kernel::default();
    let (mut rr, mut tt2, mut recs) = (Runs::default(), TVector::new(), Vec::new());
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
                // production's kernel since 0.2.8 for a block the records' path takes
                // (`takes`): the LF table filled from the front's records, walked backward.
                // And the two fronts, each from the block's symbols.
                let takes = takes_runs(&b);
                let (mut t_front, mut t_mtf) = (f64::MAX, f64::MAX);
                for _ in 0..rounds {
                    let t = Instant::now();
                    front(&b, &mut rr, k).unwrap();
                    t_front = t_front.min(t.elapsed().as_secs_f64());
                    let t = Instant::now();
                    mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
                    t_mtf = t_mtf.min(t.elapsed().as_secs_f64());
                }
                let (mut t_rec, mut t_fill) = (f64::MAX, f64::MAX);
                for _ in 0..rounds {
                    let t = Instant::now();
                    let tiled = fill(&rr, n, &mut tt2, k);
                    let t1 = Instant::now();
                    assert!(tiled, "the records do not tile the block");
                    walk_back_room(&mut tt2, b.orig_ptr, &mut fast, &mut seg);
                    t_rec = t_rec.min(t.elapsed().as_secs_f64());
                    t_fill = t_fill.min((t1 - t).as_secs_f64());
                }
                assert!(
                    fast == walked,
                    "the records' path's bytes differ from the walk's"
                );
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
                let mut k_join = f64::MAX;
                let (mut k_rules, mut k_joined, mut k_steps) = (0, 0u64, 0u64);
                for _ in 0..rounds {
                    let t = Instant::now();
                    let (trk, off) = kern.runs(&ll, b.orig_ptr);
                    let t0b = Instant::now();
                    kern.sort();
                    let (trk, tl, bl, v, joined) = if join {
                        kern.contract(trk)
                    } else {
                        let (tl, bl) = kern.link();
                        (trk, tl, bl, 0, 0)
                    };
                    let t1 = Instant::now();
                    let (word, period, t0, steps) = kern.induce(trk, off, tl, bl, v);
                    let t2 = Instant::now();
                    kern.write(word, period, t0, n, &mut out);
                    let t3 = Instant::now();
                    k_runs = k_runs.min((t0b - t).as_secs_f64());
                    k_join = k_join.min((t1 - t0b).as_secs_f64());
                    k_ind = k_ind.min((t2 - t1).as_secs_f64());
                    k_joined = joined;
                    k_steps = steps;
                    k_exp = k_exp.min((t3 - t2).as_secs_f64());
                    k_rules = kern.rules.len();
                }
                assert!(out == walked, "the kernel's bytes differ from the walk's");
                // the cells' kernel: its stages, and the whole from the run list to the bytes
                // the cells' kernel, from the front's run records: its stages, and the whole
                // from the records to the bytes as one span
                records(&ll, &mut recs);
                // (every byte wrong before this writer runs: a byte left unwritten is seen)
                out.clear();
                out.extend(walked.iter().map(|&b| !b));
                let (mut c_order, mut c_ind, mut c_write, mut c_whole) =
                    (f64::MAX, f64::MAX, f64::MAX, f64::MAX);
                for _ in 0..rounds {
                    let t0 = Instant::now();
                    kern.slots(n, &recs, b.orig_ptr);
                    let t1 = Instant::now();
                    let (word, len, _) = kern.induce_cells::<false>();
                    let t2 = Instant::now();
                    kern.write_cells(word, len, n, &mut out);
                    let t3 = Instant::now();
                    c_order = c_order.min((t1 - t0).as_secs_f64());
                    c_ind = c_ind.min((t2 - t1).as_secs_f64());
                    c_write = c_write.min((t3 - t2).as_secs_f64());
                    c_whole = c_whole.min((t3 - t0).as_secs_f64());
                }
                // (its cells, counted apart from the clock)
                kern.slots(n, &recs, b.orig_ptr);
                let c_cells = kern.induce_cells::<true>().2;
                assert!(out == walked, "the cells' bytes differ from the walk's");
                // the rounds as a schedule: with P workers a round of w steps takes ceil(w / P)
                // turns; `par` is the turns at P = 2, 4, 8, 16, and `half` the rounds that
                // hold half the steps
                let par: Vec<u64> = [2u64, 4, 8, 16]
                    .iter()
                    .map(|&p| c.work.iter().map(|&w| w.div_ceil(p)).sum())
                    .collect();
                let (mut half, mut seen) = (0usize, 0u64);
                while 2 * seen < c.steps {
                    half += 1;
                    seen += c.work[half];
                }
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
                let (coin, shapes) = coincidences(&ll);
                // the count in closed form: equal to the induction's, type by type
                let closed = admissible(&ll);
                if let Some((steps, _)) = closed {
                    assert!(
                        steps == [c.top, c.bottom, c.equal],
                        "the admissible rows {steps:?} are not the induction's steps"
                    );
                }
                let cone = closed.map_or([0, 0], |x| x.1);
                let ns = |t: f64| t * 1e9 / n as f64;
                println!(
                    "{{\"n\":{n},\"runs\":{},\"steps\":{},\"top\":{},\"bottom\":{},\"equal\":{},\"dropped\":{},\"streaks\":{},\"longest\":{},\"left\":{},\"copies\":{},\"copied\":{},\"deepest\":{},\"ns_walk\":{:.3},\"ns_runs\":{:.3},\"ns_induce\":{:.3},\"ns_expand\":{:.3},\"coincide\":{},\"hist\":[{}],\"k_rules\":{},\"k_runs\":{:.3},\"k_induce\":{:.3},\"k_write\":{:.3},\"repeats\":{},\"divisions\":{},\"crc_muls\":{},\"depth\":{},\"widest\":{},\"rounds\":{},\"selfish\":{},\"selfish_steps\":{},\"shapes\":{:?},\"k_order\":{:.3},\"k_joined\":{},\"k_steps\":{},\"closed\":{},\"cone\":{:?},\"c_cells\":{},\"c_order\":{:.3},\"c_induce\":{:.3},\"c_write\":{:.3},\"c_whole\":{:.3},\"par\":{:?},\"half\":{},\"ns_rec\":{:.3},\"ns_fill\":{:.3},\"ns_front\":{:.3},\"ns_mtf\":{:.3},\"takes\":{},\"records\":{},\"c_letters\":{}}}",
                    c.runs, c.steps, c.top, c.bottom, c.equal, c.dropped, c.streaks, c.longest, c.left,
                    ex.0, ex.1, ex.2, ns(t_walk), ns(t_runs), ns(t_ind), ns(t_exp), coin, hs.join(","),
                    k_rules, ns(k_runs), ns(k_ind), ns(k_exp), c.repeats, c.divisions, crc_muls, c.depth, c.widest, c.rounds, c.selfish, c.selfish_steps, shapes, ns(k_join), k_joined, k_steps, closed.is_some(), cone, c_cells, ns(c_order), ns(c_ind), ns(c_write), ns(c_whole), par, half, ns(t_rec), ns(t_fill), ns(t_front), ns(t_mtf), takes, recs.len(), kern.letters
                );
            }
            i += 1;
        }
        syms = b.syms;
    }
}
