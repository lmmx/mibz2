//! The walk as eight interleaved lanes (Carbon cycles 2 and 11, `BVSMC` and `BP4K`,
//! `docs/journal/2026-11-21-open-lanes.md`). The serial walk follows one pointer chain, one
//! load in flight. Here the retained rows are cut at `S` rows spread evenly from the marked
//! one: each cut row's code is replaced by a sentinel (`sbase + q`), so a segment ends when its
//! walk reaches the next cut, found by the same `lens` lookup the 16-byte emit makes (a
//! sentinel's length is `LONG`, the cold path). Eight lanes walk eight segments at once; a lane
//! that ends takes the next segment. The segments of the marked row's cycle, in the order the
//! sentinels link them, are the block.
//!
//! Two ways to place a segment's bytes before its offset is known:
//!
//! - **Two passes** (`BVSMC-2P`): the first walks every segment for its byte length and its
//!   successor only; a prefix sum along the marked cycle gives each segment its slice of `out`,
//!   and the second pass writes it there. No scratch, two walks.
//! - **One pass into 4 KiB pages** (`BP4K-BVSMC-1P`): the marked segment is written in place
//!   from `out[0]`; every other segment into a chain of 4 KiB pages, taken top-down from the
//!   part of `out` the marked segment has not reached ("dead" `out`) while it stays two pages
//!   clear of it, and from an overflow past `out[n + 16]` after that. The pages are then moved
//!   to their places in an order where no page is overwritten before it is read (Kahn's order
//!   on the pages: a target page is written once the page there hosts nothing still unread),
//!   a page being evicted to the overflow when every remaining one waits on another.
//!
//! **Safety:** no `unsafe`. Every store is a bounds-checked slice copy into `out`, and every
//! offset is checked against its segment's end (two passes) or its page's (one pass). The cuts
//! are written into `edges`/`extra` and restored before return. A walk that has not ended after
//! `m + 1` rounds (a construction that is not a permutation) is abandoned, the tables restored,
//! and the block left to the serial walk.
use super::{repeat, INLINE, LONG};

/// Lanes
pub(super) const K: usize = 8;
/// Segments, two passes (cycle 2's `BVSMC-2P(8, 64)`)
pub(super) const S2: usize = 64;
/// Segments, one pass (cycle 11's `BP4K(48)`)
pub(super) const S1: usize = 48;
/// A page of the one-pass walk
pub(super) const PAGE: usize = 4096;
/// Overflow pages `out` is reserved for past `n + 16` (256 KiB)
const RESERVE: usize = 64;
const NONE: u32 = u32::MAX;

/// The tables a walk reads, with the cuts in place
#[derive(Clone, Copy)]
pub(super) struct Tables<'t> {
    pub edges: &'t [u32],
    pub extra: &'t [u32],
    pub spelling: &'t [u8],
    pub offsets: &'t [u32],
    pub inline16: &'t [[u8; INLINE]],
    pub lens: &'t [u8],
    pub bits: u32,
}

impl Tables<'_> {
    #[inline(always)]
    fn mask(&self) -> u32 {
        (1 << self.bits) - 1
    }

    /// A code's word
    #[inline(always)]
    fn word(&self, code: usize) -> &[u8] {
        let len = self.lens[code];
        if len != LONG {
            &self.inline16[code][..usize::from(len)]
        } else {
            let w = code - 256;
            &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize]
        }
    }
}

/// An extent of the one-pass walk: a page of a segment's bytes, at `at` in `out` (a page of
/// dead `out` when `phys` is set, an overflow page past `n + 16` otherwise)
#[derive(Clone, Copy)]
struct Extent {
    at: usize,
    len: u32,
    next: u32,
    phys: u32,
    slot: u32,
}

/// The lanes' scratch, kept between blocks
#[derive(Default)]
pub(super) struct Lanes {
    cuts: Vec<u32>,
    raw: Vec<u32>,
    raw_extra: Vec<u32>,
    code: Vec<u32>,
    next: Vec<u32>,
    succ: Vec<u32>,
    bytes: Vec<usize>,
    chain: Vec<(u32, usize, usize)>,
    ext: Vec<Extent>,
    head: Vec<u32>,
    ord: Vec<(u32, usize, usize)>,
    phys_ord: Vec<u32>,
    refs: Vec<u8>,
    done: Vec<bool>,
    tp_first: Vec<u32>,
    tp_last: Vec<u32>,
    slot_refs: Vec<u8>,
    free: Vec<u32>,
    ready: Vec<u32>,
    /// (measurement) the most overflow pages a block used, and the evictions
    pub peak_slots: usize,
    pub evictions: usize,
}

/// Where the cuts went: the first sentinel code, the segments, and the parked lanes' row
#[derive(Clone, Copy)]
pub(super) struct Cuts {
    pub sbase: u32,
    pub segs: usize,
    pub park: u32,
}

impl Lanes {
    /// Cut the retained rows into at most `s` segments from `mark`: each cut row's code
    /// replaced by its sentinel, the sentinels' lengths (`LONG`) and the parked lanes' code
    /// (length 0) appended to the tables, and a parked row `m` appended to `edges`. `None`
    /// (nothing changed) when the sentinels' codes don't fit the packed edges.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn cut(
        &mut self,
        edges: &mut Vec<u32>,
        extra: &mut Vec<u32>,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        mark: u32,
        bits: u32,
        s: usize,
    ) -> Option<Cuts> {
        let m = edges.len();
        let wide = !extra.is_empty();
        let sbase = lens.len() as u32;
        let segs = s.min(m);
        let park_code = sbase + segs as u32;
        if m == 0 || (!wide && park_code > (u32::MAX >> bits)) {
            return None;
        }
        let mask = (1u32 << bits) - 1;
        self.cuts.clear();
        self.raw.clear();
        self.raw_extra.clear();
        self.code.clear();
        self.next.clear();
        for q in 0..segs {
            // distinct: consecutive cuts are at least m / segs >= 1 rows apart
            let v = ((mark as usize + q * m / segs) % m) as u32;
            let raw = edges[v as usize];
            let next = raw & mask;
            self.cuts.push(v);
            self.raw.push(raw);
            self.next.push(next);
            let sentinel = sbase + q as u32;
            if wide {
                self.raw_extra.push(extra[v as usize]);
                self.code.push(extra[v as usize]);
                extra[v as usize] = sentinel;
            } else {
                self.code.push(raw >> bits);
                edges[v as usize] = (sentinel << bits) | next;
            }
        }
        lens.resize(sbase as usize + segs, LONG);
        inline16.resize(sbase as usize + segs, [0; INLINE]);
        lens.push(0);
        inline16.push([0; INLINE]);
        let park = m as u32;
        if wide {
            edges.push(park & mask);
            extra.push(park_code);
        } else {
            edges.push((park_code << bits) | (park & mask));
        }
        Some(Cuts { sbase, segs, park })
    }

    /// The tables as they were before `cut`
    pub(super) fn restore(
        &mut self,
        cuts: Cuts,
        edges: &mut Vec<u32>,
        extra: &mut Vec<u32>,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
    ) {
        edges.pop();
        let wide = !extra.is_empty();
        if wide {
            extra.pop();
        }
        for (q, &v) in self.cuts.iter().enumerate() {
            edges[v as usize] = self.raw[q];
            if wide {
                extra[v as usize] = self.raw_extra[q];
            }
        }
        lens.truncate(cuts.sbase as usize);
        inline16.truncate(cuts.sbase as usize);
    }
}

/// One round's loads: each lane's edge and (wide) code
#[inline(always)]
fn load<const WIDE: bool>(t: &Tables, j: &[u32; K]) -> ([u32; K], [u32; K]) {
    let e: [u32; K] = std::array::from_fn(|l| t.edges[j[l] as usize]);
    let c: [u32; K] = if WIDE {
        std::array::from_fn(|l| t.extra[j[l] as usize])
    } else {
        std::array::from_fn(|l| e[l] >> t.bits)
    };
    (e, c)
}

/// The two-pass walk's first pass: per-lane state
struct Pass1 {
    j: [u32; K],
    q: [u32; K],
    b: [usize; K],
    live: [bool; K],
    next_seg: usize,
    active: usize,
}

/// The two-pass walk's second pass: per-lane state. A lane stores 16 bytes while `at < safe`,
/// that is `at + 16 <= end` (its segment's end); `safe` of a parked lane is `n + 1`, its
/// `at` is `n`, and its stores land in `out[n..n + 16]`.
struct Pass2 {
    j: [u32; K],
    at: [usize; K],
    safe: [usize; K],
    end: [usize; K],
    live: [bool; K],
    next_seg: usize,
    active: usize,
}

impl Lanes {
    /// The two-pass walk (`BVSMC-2P`). `false`: abandoned, `out` unspecified.
    pub(super) fn two_pass<const WIDE: bool>(
        &mut self,
        t: &Tables,
        cuts: Cuts,
        n: usize,
        out: &mut Vec<u8>,
    ) -> bool {
        let m = t.edges.len() - 1;
        let segs = cuts.segs;
        self.succ.clear();
        self.succ.resize(segs, NONE);
        self.bytes.clear();
        self.bytes.resize(segs, 0);
        let mut st = Pass1 {
            j: [cuts.park; K],
            q: [0; K],
            b: [0; K],
            live: [false; K],
            next_seg: 0,
            active: 0,
        };
        for l in 0..K {
            self.begin1(t, &mut st, l);
        }
        let mut rounds = 0;
        while st.active > 0 {
            rounds += 1;
            if rounds > m + 1 {
                return false;
            }
            let (e, c) = load::<WIDE>(t, &st.j);
            for l in 0..K {
                let code = c[l] as usize;
                st.j[l] = e[l] & t.mask();
                let len = t.lens[code];
                if len != LONG {
                    st.b[l] += usize::from(len);
                } else {
                    self.cold1(t, cuts, &mut st, l, code);
                }
            }
        }
        // the marked cycle's segments in order, and their slices
        self.chain.clear();
        let (mut acc, mut cs) = (0usize, 0usize);
        for _ in 0..segs {
            let end = (acc + self.bytes[cs]).min(n);
            self.chain.push((cs as u32, acc, end));
            acc = end;
            match self.succ[cs] {
                s if s as usize >= segs => return false,
                0 => break,
                s => cs = s as usize,
            }
            if acc >= n {
                break;
            }
        }
        out.resize(n + INLINE, 0);
        let mut st = Pass2 {
            j: [cuts.park; K],
            at: [n; K],
            safe: [n + 1; K],
            end: [n; K],
            live: [false; K],
            next_seg: 0,
            active: 0,
        };
        for l in 0..K {
            self.begin2(t, &mut st, l, n, out);
        }
        let mut rounds = 0;
        while st.active > 0 {
            rounds += 1;
            if rounds > m + 1 {
                return false;
            }
            let (e, c) = load::<WIDE>(t, &st.j);
            for l in 0..K {
                let code = c[l] as usize;
                st.j[l] = e[l] & t.mask();
                let len = t.lens[code];
                let at = st.at[l];
                if len != LONG && at < st.safe[l] {
                    out[at..at + INLINE].copy_from_slice(&t.inline16[code]);
                    st.at[l] = at + usize::from(len);
                } else {
                    self.cold2(t, cuts, &mut st, l, code, n, out);
                }
            }
        }
        out.truncate(acc);
        repeat(n, out);
        true
    }

    /// Pass 1: lane `l` takes the next segment, its first word counted, or parks
    fn begin1(&mut self, t: &Tables, st: &mut Pass1, l: usize) {
        if st.next_seg < self.succ.len() {
            let q = st.next_seg;
            st.next_seg += 1;
            if !st.live[l] {
                st.active += 1;
            }
            st.live[l] = true;
            st.q[l] = q as u32;
            st.b[l] = t.word(self.code[q] as usize).len();
            st.j[l] = self.next[q];
        } else {
            if st.live[l] {
                st.active -= 1;
            }
            st.live[l] = false;
            st.j[l] = t.edges.len() as u32 - 1;
        }
    }

    #[cold]
    #[inline(never)]
    fn cold1(&mut self, t: &Tables, cuts: Cuts, st: &mut Pass1, l: usize, code: usize) {
        if !st.live[l] {
            st.j[l] = cuts.park;
            return;
        }
        let first = cuts.sbase as usize;
        if (first..first + cuts.segs).contains(&code) {
            let q = st.q[l] as usize;
            self.succ[q] = (code - first) as u32;
            self.bytes[q] = st.b[l];
            self.begin1(t, st, l);
        } else {
            st.b[l] += t.word(code).len();
        }
    }

    /// Pass 2: lane `l` takes the next segment of the marked cycle, its first word written,
    /// or parks
    fn begin2(&mut self, t: &Tables, st: &mut Pass2, l: usize, n: usize, out: &mut [u8]) {
        if st.next_seg < self.chain.len() {
            let (q, off, end) = self.chain[st.next_seg];
            st.next_seg += 1;
            if !st.live[l] {
                st.active += 1;
            }
            st.live[l] = true;
            st.at[l] = off;
            st.end[l] = end;
            st.safe[l] = end.saturating_sub(INLINE - 1);
            put_bounded(t, st, l, self.code[q as usize] as usize, out);
            st.j[l] = self.next[q as usize];
        } else {
            if st.live[l] {
                st.active -= 1;
            }
            st.live[l] = false;
            st.j[l] = t.edges.len() as u32 - 1;
            st.at[l] = n;
            st.safe[l] = n + 1;
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn cold2(
        &mut self,
        t: &Tables,
        cuts: Cuts,
        st: &mut Pass2,
        l: usize,
        code: usize,
        n: usize,
        out: &mut [u8],
    ) {
        if !st.live[l] {
            st.j[l] = cuts.park;
            st.at[l] = n;
            return;
        }
        let first = cuts.sbase as usize;
        if (first..first + cuts.segs).contains(&code) {
            self.begin2(t, st, l, n, out);
        } else {
            put_bounded(t, st, l, code, out);
        }
    }
}

/// A word copied into lane `l`'s segment, cut at the segment's end
#[inline]
fn put_bounded(t: &Tables, st: &mut Pass2, l: usize, code: usize, out: &mut [u8]) {
    let word = t.word(code);
    let at = st.at[l];
    let take = word.len().min(st.end[l] - at);
    out[at..at + take].copy_from_slice(&word[..take]);
    st.at[l] = at + take;
}

/// The one-pass walk: per-lane state. A lane stores 16 bytes while `at < safe`: within its
/// page (`at + 16 <= base + PAGE`), or for the marked segment in place, below the lowest page
/// of dead `out` taken (`at + 16 <= lowest * PAGE`).
struct One {
    j: [u32; K],
    at: [usize; K],
    safe: [usize; K],
    base: [usize; K],
    /// the lane's extent, or `NONE` for the marked segment in place
    ext: [u32; K],
    q: [u32; K],
    live: [bool; K],
    next_seg: usize,
    active: usize,
    /// whether the marked segment is still written in place, by lane 0
    seg0_live: bool,
    seg0_len: usize,
    /// the lowest page of dead `out` taken (`n / PAGE` while none is)
    lowest: usize,
    /// overflow pages taken
    slots: usize,
    ovf: usize,
    n: usize,
    failed: bool,
}

impl Lanes {
    /// The one-pass walk into 4 KiB pages (`BP4K-BVSMC-1P`); without `DEAD`, every page from
    /// the overflow (cycle 2's `BVSMC-1P` with a page pool). `false`: abandoned, `out`
    /// unspecified.
    pub(super) fn one_pass<const WIDE: bool, const DEAD: bool>(
        &mut self,
        t: &Tables,
        cuts: Cuts,
        n: usize,
        out: &mut Vec<u8>,
    ) -> bool {
        let m = t.edges.len() - 1;
        let segs = cuts.segs;
        self.succ.clear();
        self.succ.resize(segs, NONE);
        self.head.clear();
        self.head.resize(segs, NONE);
        self.ext.clear();
        let ovf = n + INLINE;
        // room for the overflow pages a block takes (184-216 KiB on the admitted blocks),
        // reserved exactly: growing past it doubles `out`
        if out.capacity() < ovf + RESERVE * PAGE {
            out.reserve_exact(ovf + RESERVE * PAGE - out.len());
        }
        out.resize(ovf, 0);
        let mut st = One {
            j: [cuts.park; K],
            at: [n; K],
            safe: [n + 1; K],
            base: [0; K],
            ext: [NONE; K],
            q: [0; K],
            live: [false; K],
            next_seg: 0,
            active: 0,
            seg0_live: false,
            seg0_len: 0,
            lowest: n / PAGE,
            slots: 0,
            ovf,
            n,
            failed: false,
        };
        for l in 0..K {
            self.begin1p::<DEAD>(t, &mut st, l, out);
        }
        let mut rounds = 0;
        while st.active > 0 && !st.failed {
            rounds += 1;
            if rounds > m + 1 {
                return false;
            }
            let (e, c) = load::<WIDE>(t, &st.j);
            for l in 0..K {
                let code = c[l] as usize;
                st.j[l] = e[l] & t.mask();
                let len = t.lens[code];
                let at = st.at[l];
                if len != LONG && at < st.safe[l] {
                    out[at..at + INLINE].copy_from_slice(&t.inline16[code]);
                    st.at[l] = at + usize::from(len);
                } else {
                    self.cold1p::<DEAD>(t, cuts, &mut st, l, code, out);
                }
            }
        }
        if st.failed {
            return false;
        }
        self.peak_slots = st.slots;
        self.evictions = 0;
        let Some(wr) = self.stitch(&st, n, out) else {
            return false;
        };
        out.truncate(wr);
        repeat(n, out);
        true
    }

    /// A new extent: the next page of dead `out` down, while it stays two pages clear of the
    /// marked segment's bytes, else an overflow page
    fn alloc<const DEAD: bool>(&mut self, st: &mut One, out: &mut Vec<u8>) -> u32 {
        let id = self.ext.len() as u32;
        let p0 = if st.seg0_live { st.at[0] } else { st.seg0_len };
        let min_phys = (p0 + 2 * PAGE) / PAGE + 1;
        let e = if DEAD && st.lowest > min_phys {
            st.lowest -= 1;
            if st.seg0_live {
                st.safe[0] = (st.lowest * PAGE).saturating_sub(INLINE - 1);
            }
            Extent {
                at: st.lowest * PAGE,
                len: 0,
                next: NONE,
                phys: st.lowest as u32,
                slot: NONE,
            }
        } else {
            let slot = st.slots;
            st.slots += 1;
            // a bound no valid walk reaches (its bytes are at most n), so that a broken
            // construction can't grow `out` without end
            if st.slots > st.n / PAGE + 2 * K + S2 + 4 {
                st.failed = true;
            }
            let at = st.ovf + slot * PAGE;
            if out.len() < at + PAGE {
                out.resize(at + PAGE, 0);
            }
            Extent {
                at,
                len: 0,
                next: NONE,
                phys: NONE,
                slot: slot as u32,
            }
        };
        self.ext.push(e);
        id
    }

    /// Lane `l` onto a new extent `id`
    fn onto(&self, st: &mut One, l: usize, id: u32) {
        st.ext[l] = id;
        let at = self.ext[id as usize].at;
        st.base[l] = at;
        st.at[l] = at;
        st.safe[l] = at + PAGE - (INLINE - 1);
    }

    /// One pass: lane `l` takes the next segment, its first word written, or parks
    fn begin1p<const DEAD: bool>(&mut self, t: &Tables, st: &mut One, l: usize, out: &mut Vec<u8>) {
        if st.next_seg < self.succ.len() {
            let q = st.next_seg;
            st.next_seg += 1;
            if !st.live[l] {
                st.active += 1;
            }
            st.live[l] = true;
            st.q[l] = q as u32;
            if q == 0 {
                // lane 0, the marked segment in place
                st.seg0_live = true;
                st.ext[l] = NONE;
                st.base[l] = 0;
                st.at[l] = 0;
                st.safe[l] = (st.lowest * PAGE).saturating_sub(INLINE - 1);
            } else {
                let id = self.alloc::<DEAD>(st, out);
                self.head[q] = id;
                self.onto(st, l, id);
            }
            self.put::<DEAD>(t, st, l, self.code[q] as usize, out);
            st.j[l] = self.next[q];
        } else {
            if st.live[l] {
                st.active -= 1;
            }
            st.live[l] = false;
            st.j[l] = t.edges.len() as u32 - 1;
            st.at[l] = st.n;
            st.safe[l] = st.n + 1;
        }
    }

    /// A word appended to lane `l`'s segment, across pages, the slow way
    fn put<const DEAD: bool>(
        &mut self,
        t: &Tables,
        st: &mut One,
        l: usize,
        code: usize,
        out: &mut Vec<u8>,
    ) {
        let mut word = t.word(code);
        if st.ext[l] == NONE {
            let at = st.at[l];
            if at + word.len() <= st.lowest * PAGE {
                out[at..at + word.len()].copy_from_slice(word);
                st.at[l] = at + word.len();
                return;
            }
            // the marked segment has reached the pages taken: it goes on in pages
            st.seg0_len = at;
            st.seg0_live = false;
            let id = self.alloc::<DEAD>(st, out);
            self.head[st.q[l] as usize] = id;
            self.onto(st, l, id);
        }
        while !word.is_empty() {
            let mut used = st.at[l] - st.base[l];
            if used == PAGE {
                let cur = st.ext[l] as usize;
                self.ext[cur].len = PAGE as u32;
                let id = self.alloc::<DEAD>(st, out);
                self.ext[cur].next = id;
                self.onto(st, l, id);
                used = 0;
            }
            let take = word.len().min(PAGE - used);
            let at = st.at[l];
            out[at..at + take].copy_from_slice(&word[..take]);
            st.at[l] = at + take;
            word = &word[take..];
        }
    }

    #[cold]
    #[inline(never)]
    fn cold1p<const DEAD: bool>(
        &mut self,
        t: &Tables,
        cuts: Cuts,
        st: &mut One,
        l: usize,
        code: usize,
        out: &mut Vec<u8>,
    ) {
        if !st.live[l] {
            st.j[l] = cuts.park;
            st.at[l] = st.n;
            return;
        }
        let first = cuts.sbase as usize;
        if (first..first + cuts.segs).contains(&code) {
            let q = st.q[l] as usize;
            self.succ[q] = (code - first) as u32;
            if st.ext[l] == NONE {
                st.seg0_len = st.at[l];
                st.seg0_live = false;
            } else {
                let cur = st.ext[l] as usize;
                self.ext[cur].len = (st.at[l] - st.base[l]) as u32;
            }
            self.begin1p::<DEAD>(t, st, l, out);
        } else {
            self.put::<DEAD>(t, st, l, code, out);
        }
    }

    /// The pages moved into place after the one-pass walk: the bytes of the marked cycle, in
    /// order, from `out[seg0_len]`. Their count, or `None` if the links are broken.
    fn stitch(&mut self, st: &One, n: usize, out: &mut Vec<u8>) -> Option<usize> {
        let segs = self.succ.len();
        // the extents in target order, with their target offsets and lengths (clamped to n)
        self.ord.clear();
        let mut wr = st.seg0_len;
        let mut cs = 0usize;
        for _ in 0..segs {
            let mut e = self.head[cs];
            while e != NONE && wr < n {
                let x = self.ext[e as usize];
                let len = (x.len as usize).min(n - wr);
                if len > 0 {
                    self.ord.push((e, wr, len));
                    wr += len;
                }
                e = x.next;
            }
            cs = self.succ[cs] as usize;
            if cs >= segs {
                return None;
            }
            if cs == 0 || wr >= n {
                break;
            }
        }
        if wr == st.seg0_len {
            return Some(wr);
        }
        let total_tp = wr.div_ceil(PAGE);
        let first_tp = st.seg0_len / PAGE;
        let n_phys = n / PAGE;
        let n_slots = total_tp.max(n_phys);
        reset(&mut self.phys_ord, n_slots, NONE);
        reset(&mut self.refs, n_slots, 0);
        reset(&mut self.done, total_tp, false);
        reset(&mut self.tp_first, total_tp, NONE);
        reset(&mut self.tp_last, total_tp, 0);
        reset(&mut self.slot_refs, st.slots, 0);
        for (oi, &(e, t0, len)) in self.ord.iter().enumerate() {
            let (lo, hi) = (t0 / PAGE, (t0 + len - 1) / PAGE);
            for p in lo..=hi {
                self.tp_first[p] = self.tp_first[p].min(oi as u32);
                self.tp_last[p] = oi as u32;
            }
            let x = self.ext[e as usize];
            let rc = (hi - lo + 1) as u8;
            if x.phys != NONE {
                self.phys_ord[x.phys as usize] = oi as u32;
                self.refs[x.phys as usize] = rc;
            } else {
                self.slot_refs[x.slot as usize] = rc;
            }
        }
        self.free.clear();
        self.free
            .extend((0..st.slots as u32).filter(|&s| self.slot_refs[s as usize] == 0));
        self.ready.clear();
        self.ready.extend(
            (first_tp..total_tp)
                .filter(|&p| self.refs[p] == 0)
                .map(|p| p as u32),
        );
        let mut slots = st.slots;
        let (mut done, need) = (0, total_tp - first_tp);
        let mut scan = first_tp;
        while done < need {
            let Some(p) = self.ready.pop() else {
                // every page left waits on another: evict one to the overflow
                while scan < total_tp && (self.refs[scan] == 0 || self.done[scan]) {
                    scan += 1;
                }
                let chosen = (first_tp..total_tp)
                    .skip(scan - first_tp)
                    .find(|&s| self.refs[s] > 0 && !self.done[s])?;
                let oi = self.phys_ord[chosen] as usize;
                let (e, _, len) = self.ord[oi];
                let slot = self.free.pop().unwrap_or_else(|| {
                    slots += 1;
                    self.slot_refs.push(0);
                    (slots - 1) as u32
                });
                let at = st.ovf + slot as usize * PAGE;
                if out.len() < at + PAGE {
                    out.resize(at + PAGE, 0);
                }
                let x = &mut self.ext[e as usize];
                out.copy_within(x.at..x.at + len, at);
                x.at = at;
                x.phys = NONE;
                x.slot = slot;
                self.slot_refs[slot as usize] = self.refs[chosen];
                self.refs[chosen] = 0;
                self.evictions += 1;
                self.ready.push(chosen as u32);
                continue;
            };
            let p = p as usize;
            if self.done[p] {
                continue;
            }
            self.done[p] = true;
            done += 1;
            let start = p * PAGE;
            let end = (start + PAGE).min(wr);
            let from = start.max(st.seg0_len);
            if self.tp_first[p] == NONE {
                continue;
            }
            for oi in self.tp_first[p] as usize..=self.tp_last[p] as usize {
                let (e, t0, len) = self.ord[oi];
                let x = self.ext[e as usize];
                let (lo, hi) = (t0.max(from), (t0 + len).min(end));
                if lo < hi {
                    out.copy_within(x.at + (lo - t0)..x.at + (hi - t0), lo);
                }
                if x.phys != NONE {
                    let sp = x.phys as usize;
                    self.refs[sp] -= 1;
                    if self.refs[sp] == 0 && (first_tp..total_tp).contains(&sp) && !self.done[sp] {
                        self.ready.push(sp as u32);
                    }
                } else {
                    let s = x.slot as usize;
                    self.slot_refs[s] -= 1;
                    if self.slot_refs[s] == 0 {
                        self.free.push(s as u32);
                    }
                }
            }
        }
        self.peak_slots = slots;
        Some(wr)
    }
}

fn reset<T: Copy>(v: &mut Vec<T>, len: usize, x: T) {
    v.clear();
    v.resize(len, x);
}
