//! Research only (`forward-anchor-lanes`; claim `forward-emission-bounded-slab-rescue`):
//! the open kernel's emission walk over forward links from sampled anchors, four lanes at a time,
//! with strictly bounded per-worker slab scratch (`<= 4 * SLAB` active slabs + `< ANCHORS * SLAB`
//! tail buffer, zero retained full-block slab pool).
//!
//! The shipped walk follows one cursor (or, for `m >= 65,536`, two cursors over XOR links) from
//! the marked row, so each step's load waits on the previous one. Here 16 rows of the retained
//! cycle, `mark + k*m/16 (mod m)`, are tagged with bit 31 of their edge. Four lanes start at
//! four of them and walk forward independently, so four loads are in flight.
//!
//! Each active lane owns one fixed-size `Box<[u8; SLAB]>` slab (`SLAB = 4 KiB`, 4 slabs = 16 KiB
//! per worker, fitting inside half of L1d). Whenever a lane fills its `SLAB`-byte buffer for its
//! current segment, it flushes that exact `SLAB`-byte chunk straight into `out` (already allocated
//! by the caller) and reuses the same slab. When a lane reaches a tagged row (finishing a segment),
//! its `< SLAB` tail bytes are appended to a bounded `tails` scratch (`< 16 * 4 KiB = 64 KiB`).
//! After all segments finish, the full `SLAB`-byte chunks in `out` are permuted in place into
//! cycle order (using one recycled lane slab as the temporary buffer, zero extra allocation), and
//! a single backward pass shifts each segment's chunks right while inserting its tail.
//!
//! **Bounds.** A valid construction's edges are a permutation of the retained rows, so the
//! segments partition the rows on cycles holding an anchor: at most `m` steps in all, and at most
//! `n` bytes (every row's word is part of the block). The walk counts both. Past either, it
//! clears the tags and falls back to the shipped bounded walk (`super::emit`), whose wrong
//! block the CRC rejects.

#![forbid(unsafe_code)]

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
/// One lane's slab (compile-time constant for bounds-check elimination; 4 * 4 KiB = 16 KiB fits L1d)
const SLAB: usize = 4 * 1024;
/// Lanes walking at once
const LANES: usize = 4;
/// Maximum retained `tails` capacity (`ANCHORS * SLAB = 64 KiB`)
const MAX_TAILS: usize = ANCHORS * SLAB;
/// The fewest retained rows the walk takes: 16 distinct anchors need `m >= 16`, and below a few
/// hundred rows the setup costs more than the walk
const MIN_ROWS: u32 = 256;

/// Whether a block's walk can run here: narrow edges, every code below bit 31 beside `bits`
/// link bits, and enough retained rows
pub(super) fn fits(wide: bool, words: usize, m: u32, bits: u32) -> bool {
    !wide && m >= MIN_ROWS && words - 1 <= ((u32::MAX >> 1) >> bits) as usize
}

/// The walk's per-worker buffers, kept across blocks and statically bounded to `<= 80 KiB`
/// (`4 * 4 KiB` active lane slabs + `<= 64 KiB` segment tails + tiny chunk metadata).
#[derive(Default)]
pub(super) struct Arena {
    /// At most `LANES` (`4`) reusable `4 KiB` slabs (`<= 16 KiB` total)
    slabs: Vec<Box<[u8; SLAB]>>,
    /// Segment owner (`0..ANCHORS`) for each full `SLAB`-byte chunk flushed into `out`
    chunk_seg: Vec<u8>,
    /// Contiguous `< SLAB` tail bytes for the `<= ANCHORS` finished segments (`<= 64 KiB`)
    tails: Vec<u8>,
    /// Permutation scratch for in-place chunk reordering in `out`
    order: Vec<u16>,
    /// Visited bitmap for cycle-following in-place chunk permutation
    visited: Vec<bool>,
}

impl Arena {
    fn reset(&mut self) {
        self.chunk_seg.clear();
        self.tails.clear();
        self.order.clear();
        self.visited.clear();
    }

    fn take_slab(&mut self) -> Box<[u8; SLAB]> {
        match self.slabs.pop() {
            Some(b) => b,
            None => vec![0u8; SLAB].into_boxed_slice().try_into().unwrap(),
        }
    }

    fn return_slab(&mut self, slab: Box<[u8; SLAB]>) {
        debug_assert!(self.slabs.len() < LANES);
        self.slabs.push(slab);
    }

    /// Append a segment's `< SLAB` tail bytes into `self.tails`, growing without exceeding
    /// `MAX_TAILS` (`128 KiB`) on valid blocks.
    fn push_tail(&mut self, bytes: &[u8]) -> Option<(u32, u16)> {
        let len = bytes.len();
        if len == 0 {
            return Some((0, 0));
        }
        let start = self.tails.len();
        let need = start.checked_add(len)?;
        if need > MAX_TAILS {
            return None;
        }
        if self.tails.capacity() < need {
            // Grow in bounded steps capped at MAX_TAILS so Vec never doubles to 256 KiB.
            let target = (self.tails.capacity().saturating_mul(2))
                .max(need)
                .min(MAX_TAILS);
            self.tails.reserve_exact(target - self.tails.len());
        }
        self.tails.extend_from_slice(bytes);
        Some((start as u32, len as u16))
    }
}

struct Lane {
    /// the lane's own fixed-size `SLAB`-byte buffer (always owned by one segment `seg`)
    slab: Box<[u8; SLAB]>,
    /// the next row
    j: u32,
    /// cursor within `slab` (`0..=SLAB`)
    off: u16,
    /// active segment index (`0..ANCHORS`)
    seg: u8,
}

struct Walk<'a> {
    edges: &'a [u32],
    spelling: &'a [u8],
    inline: &'a [[u8; INLINE]],
    lens: &'a [u8],
    bits: u32,
    mask: u32,
    arena: &'a mut Arena,
    out: &'a mut Vec<u8>,
    n: usize,
    steps: usize,
    m: usize,
    anchors: [u32; ANCHORS],
    /// each segment's end: the tagged row it reached
    ends: [u32; ANCHORS],
    /// each segment's `(offset_in_tails, len)` for its final `< SLAB` bytes
    tail_span: [(u32, u16); ANCHORS],
    next: usize,
    bad: bool,
}

impl Walk<'_> {
    /// Flush a full `SLAB`-byte chunk from `lane.slab` straight into `self.out` and reset cursor.
    #[cold]
    #[inline(never)]
    fn flush_full_chunk(&mut self, lane: &mut Lane) -> bool {
        if self.out.len() + SLAB > self.n || self.arena.chunk_seg.len() >= u16::MAX as usize {
            self.bad = true;
            lane.off = 0;
            return false;
        }
        self.out.extend_from_slice(&*lane.slab);
        self.arena.chunk_seg.push(lane.seg);
        lane.off = 0;
        true
    }

    #[inline(always)]
    fn word(&mut self, lane: &mut Lane, edge: u32) {
        let code = (edge >> self.bits) as usize;
        lane.j = edge & self.mask;
        let len = self.lens[code];
        let o = usize::from(lane.off);
        if len != LONG && o <= SLAB - INLINE {
            lane.slab[o..o + INLINE].copy_from_slice(&self.inline[code]);
            lane.off = (o + usize::from(len)) as u16;
        } else {
            self.spill_word(lane, code, len);
        }
    }

    /// A word crossing a slab boundary (or a short word in the last 15 bytes of a slab): split
    /// across full `SLAB`-byte flushes into `self.out` as needed, leaving `lane.off < SLAB`.
    #[cold]
    #[inline(never)]
    fn spill_word(&mut self, lane: &mut Lane, code: usize, len: u8) {
        let mut word = if len == LONG {
            long_word(&self.inline[code], self.spelling)
        } else {
            &self.inline[code][..usize::from(len)]
        };
        loop {
            let o = usize::from(lane.off);
            let take = word.len().min(SLAB.saturating_sub(o));
            lane.slab[o..o + take].copy_from_slice(&word[..take]);
            let next_o = o + take;
            lane.off = next_o as u16;
            word = &word[take..];
            if next_o == SLAB && !self.flush_full_chunk(lane) {
                return;
            }
            if word.is_empty() {
                return;
            }
        }
    }

    /// Start the next anchor's segment on `lane` (always at `off == 0`) and settle it: `false`
    /// once no anchor is left, or the walk is bad.
    fn start(&mut self, lane: &mut Lane) -> bool {
        loop {
            if self.next == ANCHORS || self.bad || self.steps >= self.m {
                self.bad |= self.next < ANCHORS;
                return false;
            }
            lane.seg = self.next as u8;
            lane.off = 0;
            self.next += 1;
            let edge = self.edges[self.anchors[usize::from(lane.seg)] as usize] & !TAG;
            self.word(lane, edge);
            self.steps += 1;
            if !self.hit(lane) {
                return !self.bad;
            }
        }
    }

    /// If `lane`'s next row is tagged, end its segment there and flush its remainder: `true`.
    #[inline(always)]
    fn hit(&mut self, lane: &mut Lane) -> bool {
        if self.edges[lane.j as usize] & TAG == 0 {
            return false;
        }
        let seg = usize::from(lane.seg);
        self.ends[seg] = lane.j;
        let off = usize::from(lane.off);
        if off == SLAB {
            self.flush_full_chunk(lane);
        } else if off > 0 {
            match self.arena.push_tail(&lane.slab[..off]) {
                Some(span) => self.tail_span[seg] = span,
                None => self.bad = true,
            }
            lane.off = 0;
        }
        true
    }

    /// `K` lanes in step until one reaches a tagged row; `false` if the step budget runs out
    /// first. Kept `#[inline(never)]` so each `K` compiles to one tight register-allocated loop
    /// without spilling across `K = 4, 3, 2, 1`.
    #[inline(never)]
    fn run<const K: usize>(&mut self, lanes: &mut [Lane]) -> bool {
        let lanes = &mut lanes[..K];
        let edges = self.edges;
        let nw = self.lens.len().min(self.inline.len());
        let lens = &self.lens[..nw];
        let inline = &self.inline[..nw];
        let spelling = self.spelling;
        let (bits, mask) = (self.bits, self.mask);
        let mut j: [u32; K] = std::array::from_fn(|i| lanes[i].j);
        let mut off: [usize; K] = std::array::from_fn(|i| usize::from(lanes[i].off));
        let iters = (self.m - self.steps) / K;
        let mut done = 0;
        let mut hit = false;
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
                let o = off[i];
                if len != LONG && o <= SLAB - INLINE {
                    lanes[i].slab[o..o + INLINE].copy_from_slice(&inline[code]);
                    off[i] = o + usize::from(len);
                } else if len == LONG {
                    let word = long_word(&inline[code], spelling);
                    let wlen = word.len();
                    if wlen <= SLAB - o {
                        lanes[i].slab[o..o + wlen].copy_from_slice(word);
                        off[i] = o + wlen;
                    } else {
                        lanes[i].j = j[i];
                        lanes[i].off = o as u16;
                        self.spill_word(&mut lanes[i], code, len);
                        off[i] = usize::from(lanes[i].off);
                        if self.bad {
                            break 'walk;
                        }
                    }
                } else {
                    lanes[i].j = j[i];
                    lanes[i].off = o as u16;
                    self.spill_word(&mut lanes[i], code, len);
                    off[i] = usize::from(lanes[i].off);
                    if self.bad {
                        break 'walk;
                    }
                }
            }
        }
        self.steps += K * done;
        for i in 0..K {
            lanes[i].j = j[i];
            lanes[i].off = off[i] as u16;
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
    arena.reset();
    out.clear();
    if out.capacity() < n {
        out.reserve_exact(n);
    }
    let mut w = Walk {
        edges,
        spelling,
        inline,
        lens,
        bits,
        mask: (1 << bits) - 1,
        arena,
        out,
        n,
        steps: 0,
        m,
        anchors,
        ends: [u32::MAX; ANCHORS],
        tail_span: [(0, 0); ANCHORS],
        next: 0,
        bad: false,
    };
    let mut lanes: Vec<Lane> = Vec::with_capacity(LANES);
    for _ in 0..LANES {
        let slab = w.arena.take_slab();
        let mut lane = Lane {
            slab,
            j: 0,
            off: 0,
            seg: 0,
        };
        if w.start(&mut lane) {
            lanes.push(lane);
        } else {
            w.arena.return_slab(lane.slab);
        }
    }
    while !lanes.is_empty() && !w.bad {
        let ok = match lanes.len() {
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
        while i < lanes.len() {
            if w.hit(&mut lanes[i]) && !w.start(&mut lanes[i]) {
                let retired = lanes.swap_remove(i);
                w.arena.return_slab(retired.slab);
                continue;
            }
            i += 1;
        }
    }
    for lane in lanes {
        w.arena.return_slab(lane.slab);
    }
    let (bad, ends, tail_span) = (w.bad, w.ends, w.tail_span);
    for &a in &anchors {
        edges[a as usize] &= !TAG;
    }
    if bad {
        emit(edges, &[], spelling, inline, lens, n, mark, bits, out);
        return;
    }

    // Trace the marked row's cycle of segments from segment 0.
    let mut cycle_segs = [0u8; ANCHORS];
    let mut num_cycle = 0usize;
    let mut in_cycle = [false; ANCHORS];
    let mut seg = 0usize;
    for _ in 0..ANCHORS {
        if in_cycle[seg] {
            break;
        }
        in_cycle[seg] = true;
        cycle_segs[num_cycle] = seg as u8;
        num_cycle += 1;
        if ends[seg] == mark {
            break;
        }
        match anchors.iter().position(|&a| a == ends[seg]) {
            Some(next) => seg = next,
            None => break,
        }
    }

    // Build chunk permutation: on-cycle segments' full chunks in cycle order first, followed by
    // any off-cycle chunks (on multi-cycle periodic blocks) so `order` is a bijection on `0..f`.
    let f = arena.chunk_seg.len();
    debug_assert_eq!(out.len(), f * SLAB);
    let mut seg_chunks = [0usize; ANCHORS];
    arena.order.clear();
    arena.order.reserve(f);
    for &s in &cycle_segs[..num_cycle] {
        let mut count = 0usize;
        for (chunk_id, &owner) in arena.chunk_seg.iter().enumerate() {
            if owner == s {
                arena.order.push(chunk_id as u16);
                count += 1;
            }
        }
        seg_chunks[usize::from(s)] = count;
    }
    let on_cycle_chunks = arena.order.len();
    for (chunk_id, &owner) in arena.chunk_seg.iter().enumerate() {
        if !in_cycle[usize::from(owner)] {
            arena.order.push(chunk_id as u16);
        }
    }
    debug_assert_eq!(arena.order.len(), f);

    // Permute full `SLAB`-byte chunks in place inside `out[0..f * SLAB]`, reusing one recycled
    // lane slab as the temporary buffer (`0 B` extra heap allocation; each chunk moves <= once).
    if f > 1 {
        arena.visited.clear();
        arena.visited.resize(f, false);
        let tmp = arena.slabs[0].as_mut_slice();
        for start in 0..f {
            if arena.visited[start] || usize::from(arena.order[start]) == start {
                arena.visited[start] = true;
                continue;
            }
            tmp.copy_from_slice(&out[start * SLAB..(start + 1) * SLAB]);
            let mut cur = start;
            loop {
                arena.visited[cur] = true;
                let next = usize::from(arena.order[cur]);
                if next == start {
                    out[cur * SLAB..(cur + 1) * SLAB].copy_from_slice(tmp);
                    break;
                }
                out.copy_within(next * SLAB..(next + 1) * SLAB, cur * SLAB);
                cur = next;
            }
        }
    }

    // Compute exact on-cycle byte length; if corrupt edges exceeded `n`, fall back cleanly.
    let mut cycle_bytes = on_cycle_chunks * SLAB;
    for &s in &cycle_segs[..num_cycle] {
        cycle_bytes = cycle_bytes.saturating_add(usize::from(tail_span[usize::from(s)].1));
    }
    if cycle_bytes == 0 || cycle_bytes > n {
        emit(edges, &[], spelling, inline, lens, n, mark, bits, out);
        return;
    }

    // Expand `out` from `on_cycle_chunks * SLAB` to `cycle_bytes` (only touching the trailing
    // tail bytes) and insert each segment's tail in one backward pass (`dst >= src` invariant).
    let mut src = on_cycle_chunks * SLAB;
    out.truncate(src);
    out.resize(cycle_bytes, 0);
    let mut dst = cycle_bytes;
    for &s in cycle_segs[..num_cycle].iter().rev() {
        let (t_off, t_len_u16) = tail_span[usize::from(s)];
        let t_len = usize::from(t_len_u16);
        if t_len > 0 {
            dst -= t_len;
            let t_start = t_off as usize;
            out[dst..dst + t_len].copy_from_slice(&arena.tails[t_start..t_start + t_len]);
        }
        let full_bytes = seg_chunks[usize::from(s)] * SLAB;
        if full_bytes > 0 {
            src -= full_bytes;
            dst -= full_bytes;
            if dst != src {
                out.copy_within(src..src + full_bytes, dst);
            }
        }
    }
    debug_assert_eq!(src, 0);
    debug_assert_eq!(dst, 0);

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

    /// Encode a `LONG` word descriptor in `inline` matching `super::long_word`
    fn encode_long_slot(start: usize, len: usize) -> [u8; INLINE] {
        let mut slot = [0u8; INLINE];
        slot[0..4].copy_from_slice(&(start as u32).to_ne_bytes());
        slot[4..8].copy_from_slice(&(len as u32).to_ne_bytes());
        slot
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
    /// full-size ones, one kernel reused, with bounded Arena scratch verified
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
        // Also test a periodic block > 3 * SLAB so off-cycle full chunks exist during permutation.
        {
            let mut l = Vec::with_capacity(40_000);
            // BWT of ("abac")^10000: rotations sorted by ("abac","acab","baca","caba") ->
            // last chars are 'c'^10000, 'a'^10000, 'a'^10000, 'b'^10000
            l.extend(std::iter::repeat_n(b'c', 10_000));
            l.extend(std::iter::repeat_n(b'a', 20_000));
            l.extend(std::iter::repeat_n(b'b', 10_000));
            check(&mut k, &l, 0, &mut out);
            check(&mut k, &l, 12_345, &mut out);
        }
        for _ in 0..3 {
            let n = 900_000;
            let (alphabet, mean) = (1 + g.below(200), 1 + g.below(20));
            let l = runs_column(&mut g, n, alphabet, mean);
            check(&mut k, &l, g.below(n as u32), &mut out);
        }
    }

    /// Direct boundary stress for inline `LONG` and slab-crossing `spill_word`:
    /// exact `rem == wlen` (fills slab to byte `SLAB`), `rem == wlen - 1` (splits 1 byte onto
    /// next chunk), `rem == 0`, multi-slab `LONG` words (`> 8 * SLAB`), short words in the last
    /// 15 bytes (`SLAB - 15..SLAB`), `K = 4, 3, 2, 1` drain transitions, and strict `Arena` bound.
    #[test]
    fn long_word_slab_boundaries_and_multi_slab_words() {
        let mut arena = Arena::default();
        let mut out = Vec::new();
        let mut expected = Vec::new();

        let mut spelling = Vec::new();
        let mut inline = Vec::new();
        let mut lens = Vec::new();

        let mut short1 = [0u8; INLINE];
        short1[0] = b'A';
        inline.push(short1);
        lens.push(1u8);

        let short16 = *b"0123456789abcdef";
        inline.push(short16);
        lens.push(INLINE as u8);

        let mut push_long = |len: usize, seed: u8| {
            let start = spelling.len();
            for i in 0..len {
                spelling.push(seed.wrapping_add((i * 31 + (i >> 5)) as u8));
            }
            inline.push(encode_long_slot(start, len));
            lens.push(LONG);
        };
        push_long(2048, 0x10); // divides SLAB = 4096 evenly (2 * 2048 == 4096)
        push_long(2049, 0x40); // misaligned -> hits rem == wlen - 1 and short-tail spills
        push_long(70_000, 0x90); // spans 18 slabs in a single spill_word

        let m = 320usize;
        let bits = 9u32;
        let mut codes = vec![0u32; m];
        for c in &mut codes[0..17] {
            *c = 2;
        }
        codes[17] = 1;
        codes[18] = 3;
        codes[19] = 0;
        for c in &mut codes[20..35] {
            *c = 2;
        }
        codes[35] = 3;
        codes[36] = 4;
        codes[37] = 1;
        codes[38] = 0;
        codes[39] = 3;
        for (idx, c) in codes[40..m].iter_mut().enumerate() {
            *c = match idx % 7 {
                0 | 1 => 2,
                2 => 3,
                3 => 1,
                4 => 0,
                5 if idx == 47 => 4,
                _ => 1,
            };
        }

        let mut edges = vec![0u32; m];
        let mut order: Vec<usize> = (0..m).collect();
        order[1..].rotate_left(13);
        let mut n = 0usize;
        for pos in 0..m {
            let cur = order[pos];
            let nxt = order[(pos + 1) % m];
            let code = codes[pos];
            edges[cur] = (code << bits) | (nxt as u32);
            let w = if lens[code as usize] == LONG {
                long_word(&inline[code as usize], &spelling)
            } else {
                &inline[code as usize][..usize::from(lens[code as usize])]
            };
            expected.extend_from_slice(w);
            n += w.len();
        }
        let edges_before = edges.clone();

        // Run twice reusing the same Arena to verify bounded recycling leaves zero stale state.
        for _ in 0..2 {
            out.clear();
            out.extend_from_slice(&[0xAA; 64]);
            walk(
                &mut edges, &spelling, &inline, &lens, n, 0, bits, &mut arena, &mut out,
            );
            assert_eq!(edges, edges_before, "tags must be cleared");
            assert_eq!(out.len(), n);
            assert_eq!(out, expected);
        }

        // Verify Arena scratch is strictly bounded: <= LANES slabs (16 KiB) and <= MAX_TAILS (64 KiB).
        assert!(
            arena.slabs.len() <= LANES,
            "slabs {} > LANES {}",
            arena.slabs.len(),
            LANES
        );
        assert!(
            arena.tails.capacity() <= MAX_TAILS,
            "tails cap {} > MAX_TAILS {}",
            arena.tails.capacity(),
            MAX_TAILS
        );
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

    /// Miri's case: one block past `MIN_ROWS` through the shipped instance plus exact slab-edge
    /// `LONG` boundary crossing under Miri
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
