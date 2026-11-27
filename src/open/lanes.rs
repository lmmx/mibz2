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
//!
//! Each active lane owns a disjoint `Box<[u8; SLAB]>` slab (lazily allocated and recycled in a
//! per-worker `Arena` pool). Fixed compile-time slab size `SLAB = 32 KiB` lets `o <= SLAB - INLINE`
//! eliminate slab bounds and overflow checks in safe Rust, `LONG` words that fit in the current
//! slab (`> 99%` of long words) stay inline without spilling to cold code, and `#[inline(never)]`
//! keeps `run<K>` compact with zero register spills across `K = 4, 3, 2, 1`.
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
/// One lane's slab (compile-time constant for bounds-check elimination)
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
    /// Reusable fixed-size 32 KiB slabs, lazily grown up to actual block demand
    pool: Vec<Box<[u8; SLAB]>>,
    /// Slabs retired during the current block walk, indexed by `slab_id: u16`
    used: Vec<Box<[u8; SLAB]>>,
    /// `(segment, slab_id, start, end)` of each closed slice, in forward order per segment
    slices: Vec<(u8, u16, u16, u16)>,
    /// Per-lane pending `(segment, start, end)` slices on the lane's currently open slab
    pending: [Vec<(u8, u16, u16)>; LANES],
    /// Total slabs handed out during the current walk (active + `used`), bounded by `max_slabs`
    allocated: usize,
}

impl Arena {
    fn reset(&mut self) {
        self.pool.append(&mut self.used);
        self.slices.clear();
        for p in &mut self.pending {
            p.clear();
        }
        self.allocated = 0;
    }

    fn alloc_slab(&mut self, max_slabs: usize) -> Option<Box<[u8; SLAB]>> {
        if self.allocated >= max_slabs {
            return None;
        }
        self.allocated += 1;
        match self.pool.pop() {
            Some(b) => Some(b),
            None => Some(vec![0u8; SLAB].into_boxed_slice().try_into().unwrap()),
        }
    }
}

struct Lane {
    /// the lane's own fixed-size 32 KiB slab
    slab: Box<[u8; SLAB]>,
    /// the next row
    j: u32,
    /// where its open slice starts and its cursor within `slab` (`0..=SLAB`)
    start: u16,
    off: u16,
    /// active segment index (`0..ANCHORS`)
    seg: u8,
    /// index into `Arena::pending` (`0..LANES`)
    slot: u8,
}

impl Lane {
    fn flush_slab(&mut self, arena: &mut Arena, replacement: Box<[u8; SLAB]>) {
        let pending = &mut arena.pending[usize::from(self.slot)];
        if self.off > self.start {
            pending.push((self.seg, self.start, self.off));
        }
        let old = std::mem::replace(&mut self.slab, replacement);
        let slab_id = arena.used.len() as u16;
        arena.used.push(old);
        for (seg, a, b) in pending.drain(..) {
            arena.slices.push((seg, slab_id, a, b));
        }
        self.start = 0;
        self.off = 0;
    }

    fn retire(self, arena: &mut Arena) {
        let pending = &mut arena.pending[usize::from(self.slot)];
        if self.off > self.start {
            pending.push((self.seg, self.start, self.off));
        }
        let slab_id = arena.used.len() as u16;
        arena.used.push(self.slab);
        for (seg, a, b) in pending.drain(..) {
            arena.slices.push((seg, slab_id, a, b));
        }
    }
}

struct Walk<'a> {
    edges: &'a [u32],
    spelling: &'a [u8],
    inline: &'a [[u8; INLINE]],
    lens: &'a [u8],
    bits: u32,
    mask: u32,
    arena: &'a mut Arena,
    max_slabs: usize,
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
        let o = usize::from(lane.off);
        if len != LONG && o <= SLAB - INLINE {
            lane.slab[o..o + INLINE].copy_from_slice(&self.inline[code]);
            lane.off = (o + usize::from(len)) as u16;
        } else {
            self.spill_word(lane, code, len);
        }
    }

    /// A word crossing a slab boundary (or a short word in the last 15 bytes of a slab): split
    /// across slabs as needed
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
            lane.off = (o + take) as u16;
            word = &word[take..];
            if word.is_empty() {
                return;
            }
            let Some(fresh) = self.arena.alloc_slab(self.max_slabs) else {
                self.bad = true;
                lane.off = SLAB as u16;
                return;
            };
            lane.flush_slab(self.arena, fresh);
        }
    }

    /// Start the next anchor's segment on `lane` (at its cursor) and settle it: `false` once no
    /// anchor is left, or the walk is bad
    fn start(&mut self, lane: &mut Lane) -> bool {
        loop {
            if self.next == ANCHORS || self.bad || self.steps >= self.m {
                self.bad |= self.next < ANCHORS;
                return false;
            }
            lane.seg = self.next as u8;
            lane.start = lane.off;
            self.next += 1;
            let edge = self.edges[self.anchors[usize::from(lane.seg)] as usize] & !TAG;
            self.word(lane, edge);
            self.steps += 1;
            if !self.hit(lane) {
                return !self.bad;
            }
        }
    }

    /// If `lane`'s next row is tagged, end its segment there: `true`
    #[inline(always)]
    fn hit(&mut self, lane: &mut Lane) -> bool {
        if self.edges[lane.j as usize] & TAG == 0 {
            return false;
        }
        self.ends[usize::from(lane.seg)] = lane.j;
        if lane.off > lane.start {
            self.arena.pending[usize::from(lane.slot)].push((lane.seg, lane.start, lane.off));
        }
        lane.start = lane.off;
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
    let max_slabs = n
        .div_ceil(SLAB)
        .saturating_add(LANES + 1)
        .min(u16::MAX as usize);
    let mut w = Walk {
        edges,
        spelling,
        inline,
        lens,
        bits,
        mask: (1 << bits) - 1,
        arena,
        max_slabs,
        steps: 0,
        m,
        anchors,
        ends: [u32::MAX; ANCHORS],
        next: 0,
        bad: false,
    };
    let mut lanes: Vec<Lane> = Vec::with_capacity(LANES);
    for slot in 0..LANES {
        let Some(slab) = w.arena.alloc_slab(max_slabs) else {
            w.bad = true;
            break;
        };
        let mut lane = Lane {
            slab,
            j: 0,
            start: 0,
            off: 0,
            seg: 0,
            slot: slot as u8,
        };
        if w.start(&mut lane) {
            lanes.push(lane);
        } else {
            lane.retire(w.arena);
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
                retired.retire(w.arena);
                continue;
            }
            i += 1;
        }
    }
    for lane in lanes {
        lane.retire(w.arena);
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
    out.reserve(n);
    let mut seg = 0;
    for _ in 0..ANCHORS {
        for &(s, slab_id, a, b) in arena.slices.iter() {
            if usize::from(s) == seg {
                let take = usize::from(b - a).min(n - out.len());
                if take > 0 {
                    let slab = &arena.used[usize::from(slab_id)];
                    let start = usize::from(a);
                    out.extend_from_slice(&slab[start..start + take]);
                }
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

    /// Direct boundary stress for inline `LONG` and slab-crossing `spill_word`:
    /// exact `rem == wlen` (fills slab to byte 32,768), `rem == wlen - 1` (splits 1 byte onto
    /// next slab), `rem == 0` (word starts at offset 32,768), multi-slab `LONG` words (`> 2*SLAB`),
    /// short words in the last 15 bytes (`SLAB - 15..SLAB`), and `K = 4, 3, 2, 1` drain transitions.
    #[test]
    fn long_word_slab_boundaries_and_multi_slab_words() {
        let mut arena = Arena::default();
        let mut out = Vec::new();
        let mut expected = Vec::new();

        // Build spelling pool holding:
        // - code 0: short 1-byte filler
        // - code 1: short 16-byte filler
        // - code 2: 2048-byte LONG (divides SLAB = 32768 evenly -> hits rem == wlen & rem == 0!)
        // - code 3: 2049-byte LONG (misaligned -> hits rem == wlen - 1 and short-tail spills!)
        // - code 4: 70_000-byte LONG (> 2 * SLAB -> spans 3 slabs in a single spill_word!)
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
        push_long(2048, 0x10);
        push_long(2049, 0x40);
        push_long(70_000, 0x90);

        // Single 320-row cycle (> MIN_ROWS = 256) with 16 anchors; craft codes so different
        // segments hit exact slab fill (16 * 2048 == 32768), rem == wlen - 1, rem == 0,
        // 15-byte short-word tail, multi-slab 70 KB word, and unequal segment lengths (K=4->3->2->1).
        let m = 320usize;
        let bits = 9u32; // 2^9 = 512 > 320
        let mut codes = vec![0u32; m];
        // Segment 0 (rows 0..20): 16 * 2048 = 32768 exact slab fill (rem == wlen), then rem == 0 LONG
        for c in &mut codes[0..17] {
            *c = 2;
        }
        codes[17] = 1; // short16 right on fresh slab
        codes[18] = 3; // 2049 LONG
        codes[19] = 0;
        // Segment 1 (rows 20..40): 15 * 2048 + 2047? Use 15 * 2048 + 127*16 + 15*1 -> rem = 1!
        // With 16 * 2048 - 1 via 15*2048 + 127*16 + ... we have 20 rows: put 15*2048 + 1*2049 ->
        // at row 35 off = 30720, rem = 2048, wlen = 2049 => exact `rem == wlen - 1`!
        for c in &mut codes[20..35] {
            *c = 2;
        }
        codes[35] = 3; // rem == 2048 == wlen - 1 (1 byte spills to next slab!)
        codes[36] = 4; // 70,000-byte LONG spanning 3 slabs
        codes[37] = 1;
        codes[38] = 0;
        codes[39] = 3;
        // Segment 2 (rows 40..60): short word in last 15 bytes of slab (off = 32753, rem = 15 < 16)
        // 15*2048 = 30720; + 2049 = 32769 (off=1 on slab2). Instead on rows 40..60 put 16*2049:
        // after 15*2049 = 30735, + 2049 crosses by 16; then after 15*2048 + 2049 + ... mix all codes.
        for (idx, c) in codes[40..m].iter_mut().enumerate() {
            *c = match idx % 7 {
                0 | 1 => 2,
                2 => 3,
                3 => 1,
                4 => 0,
                5 if idx == 47 => 4, // second multi-slab word in a later anchor segment
                _ => 1,
            };
        }

        // Build a single cycle visiting rows in a non-trivial permutation so K=4->3->2->1 drain
        // exercises unequal segment lengths.
        let mut edges = vec![0u32; m];
        let mut order: Vec<usize> = (0..m).collect();
        // Rotate tail slightly while keeping 0 as mark
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

        // Run twice reusing the same Arena to verify slab pool recycling leaves zero stale state.
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

        // Verify lazy pool stayed bounded and recycled all slabs cleanly.
        let total_slabs = arena.pool.len() + arena.used.len();
        assert!(total_slabs <= n.div_ceil(SLAB) + LANES + 1);
        assert!(arena.used.len() <= total_slabs);
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
