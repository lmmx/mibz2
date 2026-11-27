//! Research only (`forward-anchor-lanes`; claim `forward-emission`, registration
//! `docs/journal/2026-11-27-forward-emission/registration.md`): the open kernel's emission walk
//! over forward links from sampled anchors, four lanes at a time.
//!
//! The shipped walk follows one cursor (or, for `m >= 65,536`, two cursors over XOR links) from
//! the marked row, so each step's load waits on the previous one. Here 16 rows of the retained
//! cycle, `mark + k*m/16 (mod m)`, are tagged with bit 31 of their edge. Four lanes start at
//! four of them and walk forward independently, so four loads are in flight. A lane that reaches
//! a tagged row has finished a segment; it starts the next unstarted anchor in the same slab.
//! The segments are chained back into cycle order from the marked row's and copied into `out`.
//! Each lane writes into 32 KiB slabs of a reused per-worker arena. A word that does not fit
//! the rest of a slab is split across two.
//!
//! The probe it ports is `examples/forward_4lane_probe.rs` (#1050): its "stitch" arm, with the
//! same anchors, lanes and slabs, written afresh here with no `unsafe`.
//!
//! **Bounds.** A valid construction's edges are a permutation of the retained rows, so the
//! segments partition the rows on cycles holding an anchor: at most `m` steps in all, and at most
//! `n` bytes (every row's word is part of the block). The walk counts both. Past either, it
//! clears the tags and falls back to the shipped bounded walk (`super::emit`), whose wrong
//! block the CRC rejects.

use super::{emit, long_word, repeat, INLINE, LONG};

#[cfg(test)]
thread_local! {
    /// Blocks this thread's walks have taken (the tests' check that they engage)
    static TAKEN: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Anchors sampled per block
const ANCHORS: usize = 16;
/// The tag on an anchor's edge: bit 31, free when every code fits in `31 - bits` bits
const TAG: u32 = 1 << 31;
/// One lane's slab
const SLAB: usize = 32 * 1024;
/// Lanes walking at once
const LANES: usize = 4;
/// The fewest retained rows the walk takes: 16 distinct anchors need `m >= 16`, and below a few
/// hundred rows the slabs cost more than the walk
const MIN_ROWS: u32 = 256;

/// Whether a block's walk can run here: narrow edges, every code below bit 31 beside `bits`
/// link bits, and enough retained rows
pub(super) fn fits(wide: bool, words: usize, m: u32, bits: u32) -> bool {
    !wide && m >= MIN_ROWS && words - 1 <= ((u32::MAX >> 1) >> bits) as usize
}

/// The walk's per-worker buffers, kept across blocks
#[derive(Default)]
pub(super) struct Arena {
    bytes: Vec<u8>,
    /// `(segment, start, end)` of each closed slice of `bytes`, in the order closed: a
    /// segment's slices are in its own order
    slices: Vec<(u8, u32, u32)>,
}

#[derive(Clone, Copy, Default)]
struct Lane {
    /// the next row
    j: u32,
    seg: usize,
    /// where its open slice starts, its cursor and its slab's end, in `bytes`
    start: usize,
    at: usize,
    end: usize,
}

struct Walk<'a> {
    edges: &'a [u32],
    spelling: &'a [u8],
    inline: &'a [[u8; INLINE]],
    lens: &'a [u8],
    bits: u32,
    mask: u32,
    bytes: &'a mut [u8],
    slices: &'a mut Vec<(u8, u32, u32)>,
    /// the next free slab
    top: usize,
    steps: usize,
    m: usize,
    anchors: [u32; ANCHORS],
    /// each segment's end: the tagged row it reached
    ends: [u32; ANCHORS],
    next: usize,
    bad: bool,
}

impl Walk<'_> {
    #[inline(always)]
    fn word(&mut self, lane: &mut Lane, edge: u32) {
        let code = (edge >> self.bits) as usize;
        lane.j = edge & self.mask;
        let len = self.lens[code];
        if len != LONG && lane.at + INLINE <= lane.end {
            self.bytes[lane.at..lane.at + INLINE].copy_from_slice(&self.inline[code]);
            lane.at += usize::from(len);
        } else {
            self.word_slow(lane, code, len);
        }
    }

    /// A word past its slab's last 16 bytes, or a long one: split across slabs as needed
    #[cold]
    #[inline(never)]
    fn word_slow(&mut self, lane: &mut Lane, code: usize, len: u8) {
        let inline = self.inline;
        let spelling = self.spelling;
        let mut word = if len == LONG {
            long_word(&inline[code], spelling)
        } else {
            &inline[code][..usize::from(len)]
        };
        loop {
            let take = word.len().min(lane.end - lane.at);
            self.bytes[lane.at..lane.at + take].copy_from_slice(&word[..take]);
            lane.at += take;
            word = &word[take..];
            if word.is_empty() {
                return;
            }
            self.close(lane);
            if !self.slab(lane) {
                return;
            }
        }
    }

    fn close(&mut self, lane: &Lane) {
        if lane.at > lane.start {
            self.slices
                .push((lane.seg as u8, lane.start as u32, lane.at as u32));
        }
    }

    /// A fresh slab for `lane`, or `false` (and the walk bad) past the byte budget
    fn slab(&mut self, lane: &mut Lane) -> bool {
        if self.top + SLAB > self.bytes.len() {
            self.bad = true;
            lane.at = lane.end;
            return false;
        }
        (lane.start, lane.at, lane.end) = (self.top, self.top, self.top + SLAB);
        self.top += SLAB;
        true
    }

    /// Start the next anchor's segment on `lane` (at its cursor) and settle it: `false` once no
    /// anchor is left, or the walk is bad
    fn start(&mut self, lane: &mut Lane) -> bool {
        loop {
            if self.next == ANCHORS || self.bad || self.steps >= self.m {
                self.bad |= self.next < ANCHORS;
                return false;
            }
            lane.seg = self.next;
            lane.start = lane.at;
            self.next += 1;
            let edge = self.edges[self.anchors[lane.seg] as usize] & !TAG;
            self.word(lane, edge);
            self.steps += 1;
            if !self.hit(lane) {
                return !self.bad;
            }
        }
    }

    /// If `lane`'s next row is tagged, end its segment there: `true`
    #[inline(always)]
    fn hit(&mut self, lane: &Lane) -> bool {
        if self.edges[lane.j as usize] & TAG == 0 {
            return false;
        }
        self.ends[lane.seg] = lane.j;
        self.close(lane);
        true
    }

    /// `K` lanes in step until one reaches a tagged row; `false` if the step budget runs out
    /// first. The lanes' rows and cursors are kept in locals here, so the `K` loads of a step
    /// are issued together.
    #[inline(always)]
    fn run<const K: usize>(&mut self, lanes: &mut [Lane; LANES]) -> bool {
        let (edges, inline, lens) = (self.edges, self.inline, self.lens);
        let (bits, mask) = (self.bits, self.mask);
        let mut j: [u32; K] = std::array::from_fn(|i| lanes[i].j);
        let mut at: [usize; K] = std::array::from_fn(|i| lanes[i].at);
        let mut end: [usize; K] = std::array::from_fn(|i| lanes[i].end);
        let iters = (self.m - self.steps) / K;
        let mut done = 0;
        let mut hit = false;
        let mut bytes = std::mem::take(&mut self.bytes);
        'walk: while done < iters {
            let e: [u32; K] = std::array::from_fn(|i| edges[j[i] as usize]);
            if (e.iter().fold(0, |a, &b| a | b) as i32) < 0 {
                hit = true;
                break;
            }
            done += 1;
            for i in 0..K {
                let code = (e[i] >> bits) as usize;
                j[i] = e[i] & mask;
                let len = lens[code];
                if len != LONG && at[i] + INLINE <= end[i] {
                    bytes[at[i]..at[i] + INLINE].copy_from_slice(&inline[code]);
                    at[i] += usize::from(len);
                } else {
                    self.bytes = bytes;
                    let mut lane = lanes[i];
                    (lane.j, lane.at, lane.end) = (j[i], at[i], end[i]);
                    self.word_slow(&mut lane, code, len);
                    lanes[i] = lane;
                    (at[i], end[i]) = (lane.at, lane.end);
                    bytes = std::mem::take(&mut self.bytes);
                    if self.bad {
                        break 'walk;
                    }
                }
            }
        }
        self.bytes = bytes;
        self.steps += K * done;
        for i in 0..K {
            (lanes[i].j, lanes[i].at, lanes[i].end) = (j[i], at[i], end[i]);
        }
        if self.bad {
            return false;
        }
        // with fewer than `K` steps left, a valid walk has a lane at a tagged row now
        hit || (0..K).any(|i| edges[j[i] as usize] & TAG != 0)
    }
}

/// The block's `n` bytes into `out`, from forward `edges` (narrow, `fits`) and the marked row
#[allow(clippy::too_many_arguments)]
pub(super) fn walk(
    edges: &mut [u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    arena: &mut Arena,
    out: &mut Vec<u8>,
) {
    let m = edges.len();
    #[cfg(test)]
    TAKEN.with(|t| t.set(t.get() + 1));
    debug_assert!(m >= MIN_ROWS as usize && (mark as usize) < m);
    let anchors: [u32; ANCHORS] = std::array::from_fn(|k| {
        let a = mark as u64 + (k as u64 * m as u64) / ANCHORS as u64;
        (a % m as u64) as u32
    });
    for &a in &anchors {
        edges[a as usize] |= TAG;
    }
    let slabs = n.div_ceil(SLAB) + LANES + 1;
    if arena.bytes.len() < slabs * SLAB {
        arena.bytes.resize(slabs * SLAB, 0);
    }
    arena.slices.clear();
    let mut w = Walk {
        edges,
        spelling,
        inline,
        lens,
        bits,
        mask: (1 << bits) - 1,
        bytes: &mut arena.bytes[..slabs * SLAB],
        slices: &mut arena.slices,
        top: 0,
        steps: 0,
        m,
        anchors,
        ends: [u32::MAX; ANCHORS],
        next: 0,
        bad: false,
    };
    let mut lanes = [Lane::default(); LANES];
    let mut live = 0;
    for _ in 0..LANES {
        let mut lane = Lane::default();
        if w.slab(&mut lane) && w.start(&mut lane) {
            lanes[live] = lane;
            live += 1;
        }
    }
    while live > 0 && !w.bad {
        let ok = match live {
            4 => w.run::<4>(&mut lanes),
            3 => w.run::<3>(&mut lanes),
            2 => w.run::<2>(&mut lanes),
            _ => w.run::<1>(&mut lanes),
        };
        if !ok {
            w.bad = true;
            break;
        }
        let mut i = 0;
        while i < live {
            let mut lane = lanes[i];
            if w.hit(&lane) && !w.start(&mut lane) {
                live -= 1;
                lanes[i] = lanes[live];
                continue;
            }
            lanes[i] = lane;
            i += 1;
        }
    }
    let (bad, ends) = (w.bad, w.ends);
    for &a in &anchors {
        edges[a as usize] &= !TAG;
    }
    if bad {
        emit(edges, &[], spelling, inline, lens, n, mark, bits, out);
        return;
    }
    // the marked row's cycle, segment by segment from its own (segment 0)
    out.clear();
    let mut seg = 0;
    for _ in 0..ANCHORS {
        for &(s, a, b) in arena.slices.iter() {
            if usize::from(s) == seg {
                let take = (b - a) as usize;
                let take = take.min(n - out.len());
                out.extend_from_slice(&arena.bytes[a as usize..a as usize + take]);
            }
        }
        if out.len() >= n || ends[seg] == mark {
            break;
        }
        match anchors.iter().position(|&a| a == ends[seg]) {
            Some(next) => seg = next,
            None => break,
        }
    }
    repeat(n, out);
}

#[cfg(test)]
mod tests {
    use super::super::b5::{plain, runs_column, Lcg};
    use super::super::{Kernel, Runs, LIMIT};
    use super::*;

    fn taken() -> usize {
        TAKEN.with(|t| t.get())
    }

    /// The four-lane walk through the shipped instance and the unqueued ones, into a poisoned
    /// output: exactly the plain inverse, on arbitrary columns (many cycles or one), and the
    /// walk engaged on the blocks past `MIN_ROWS` retained rows
    fn check(k: &mut Kernel, l: &[u8], p: u32, out: &mut Vec<u8>) {
        let r = Runs::of_column(l);
        let want = plain(l, p);
        for arm in 0..3 {
            out.clear();
            out.extend(want.iter().map(|b| !b));
            match arm {
                0 => k.run::<true, true>(r.records(), l.len(), p, LIMIT, out),
                1 => k.run::<true, false>(r.records(), l.len(), p, LIMIT, out),
                _ => k.run::<false, true>(r.records(), l.len(), p, LIMIT, out),
            };
            assert!(*out == want, "n {}, row {p}, arm {arm}", l.len());
        }
    }

    #[test]
    fn lanes_match_the_plain_inverse() {
        let mut g = Lcg(0x6c61_6e65);
        let mut k = Kernel::default();
        let mut out = Vec::new();
        let before = taken();
        for &n in &[300usize, 1000, 4097, 33_000, 70_000, 200_000] {
            for &alphabet in &[1u32, 2, 3, 17, 256] {
                for &mean in &[1u32, 3, 16, 17, 130] {
                    let l = runs_column(&mut g, n, alphabet, mean);
                    for p in [0, n as u32 - 1, g.below(n as u32)] {
                        check(&mut k, &l, p, &mut out);
                    }
                }
            }
        }
        assert!(taken() - before > 100, "walk engaged {}", taken() - before);
    }

    /// Periodic blocks (the marked row's cycle shorter than the block, anchors off it), and
    /// full-size ones, one kernel reused
    #[test]
    fn lanes_on_periodic_and_full_blocks() {
        let mut k = Kernel::default();
        let mut out = Vec::new();
        let mut g = Lcg(0x7065_7269);
        for unit in [&b"abac"[..], b"aaab", b"ab", b"abcdefghij"] {
            let text: Vec<u8> = unit.iter().copied().cycle().take(4000).collect();
            // a column of a periodic text: the BWT of `unit^r` is `unit`'s BWT, each byte r times
            let mut l = Vec::new();
            let mut rot: Vec<usize> = (0..text.len()).collect();
            rot.sort_by(|&a, &b| {
                let ra = text[a..].iter().chain(&text[..a]);
                let rb = text[b..].iter().chain(&text[..b]);
                ra.cmp(rb)
            });
            for &r in &rot {
                l.push(text[(r + text.len() - 1) % text.len()]);
            }
            for p in [0, 1, 1999, 3999] {
                check(&mut k, &l, p, &mut out);
            }
        }
        for _ in 0..3 {
            let n = 900_000;
            let (alphabet, mean) = (1 + g.below(200), 1 + g.below(20));
            let l = runs_column(&mut g, n, alphabet, mean);
            check(&mut k, &l, g.below(n as u32), &mut out);
        }
    }

    /// Edges that are no permutation (every row to row 1, a cycle with no anchor; and a row
    /// pointing past the anchors' cycle back into itself): the walk ends, falls back to the
    /// bounded walk, gives `n` bytes and leaves no tag behind
    #[test]
    fn invalid_edges_fall_back_and_untag() {
        let m = 512usize;
        let bits = 10;
        let inline: Vec<[u8; INLINE]> = (0..=255u8)
            .map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            })
            .collect();
        let lens = vec![1u8; 256];
        let mut arena = Arena::default();
        let mut out = Vec::new();
        for shape in 0..3u32 {
            let mut edges: Vec<u32> = (0..m as u32)
                .map(|j| {
                    let next = match shape {
                        0 => 1,
                        1 => (j + 1) % 7,
                        _ => (j * 3 + 1) % m as u32,
                    };
                    ((j & 255) << bits) | next
                })
                .collect();
            let before = edges.clone();
            for n in [m, 3 * m, 100_000] {
                walk(
                    &mut edges,
                    &[],
                    &inline,
                    &lens,
                    n,
                    0,
                    bits,
                    &mut arena,
                    &mut out,
                );
                assert_eq!(out.len(), n);
                assert_eq!(edges, before);
            }
        }
    }

    /// Miri's case: one block past `MIN_ROWS` through the shipped instance
    #[test]
    fn miri_forward_anchor_lanes() {
        let mut g = Lcg(0x6d69_7269);
        let mut k = Kernel::default();
        let mut out = Vec::new();
        let before = taken();
        for alphabet in [3, 40] {
            let l = runs_column(&mut g, 700, alphabet, 2);
            check(&mut k, &l, 5, &mut out);
        }
        assert!(taken() > before);
    }
}
