#![forbid(unsafe_code)]
//! Wormhole Cycle 18 — Wormhole Route 6 for Stage `S3 + S4`:
//!   - `A1`, `A2`: `origin/master` (`90e671966`) `mibz2::bwt::OpenCells` (native A/A control).
//!   - `M0` (`MasterClone`): same-crate verbatim `origin/master`.
//!   - `C17` (`Lazy15`): Cycle 17's promoted `GradedRV + Lazy-Prefix15-DirectInline16` (`Vec<Strip>` + `Vec<Band>`).
//!   - `B1` (`StripBandMux + BidiBottom`): Chaitin (1982) Live-Range Multiplexing unifying `Vec<Strip>`
//!     (`S1..S2a`) and `Vec<Band>` (`S3..S4`) into ONE shared `sb_buf: Vec<u32>` (`-49..98 KiB` heap
//!     scratch saved per worker = `-0.5..1.0 MiB` at 10w!) + Pohl (1971) 2-Way Meet-in-the-Middle
//!     walk on the circular doubly-linked `side-1` list (`2x` pointer-chase MLP on `bottom`!).
//!   - `B2` (`FusedS3S4-TailArena`): places `Elim` at `edges[m..]` (`Theorem 18.1` test: `+400..585 KiB`
//!     extra scratch) to stream `Reader::wire` directly inside `S3` with `0` `Band` materialization.
//!
//! Stage `S5` (`emit`) is 100% verbatim identical to `origin/master` across all arms.

use mibz2::bwt::{ForcedOpenCells, InverseBwt, LazyCells, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::fs;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const CHUNK: usize = 512;
const BYTES: usize = 2 * CHUNK;
const NONE: u32 = u32::MAX;
const EPOCH: u32 = 1 << 16;
const NARROW_NODES: usize = 1 << 16;
const RULE4: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

#[derive(Clone, Copy, PartialEq)]
struct Cap {
    m: u32,
    strips: usize,
}
impl Cap {
    const NONE: Cap = Cap {
        m: u32::MAX,
        strips: usize::MAX,
    };
    fn dense(n: usize) -> Cap {
        Cap {
            m: DENSE_M.min((n as u64 * 3 / 10) as u32),
            strips: DENSE_STRIPS,
        }
    }
}
fn lazy_counts_admit(records: usize, n: usize) -> bool {
    records * 64 >= n && records * 20 < n * 3
}

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

#[derive(Clone, Copy)]
struct Band {
    end: u32,
    delta: u32,
    code: u32,
}

#[derive(Default)]
struct Spill {
    next: Vec<u32>,
    ovf: Vec<[u8; BYTES]>,
    cuts: Vec<[u32; 256]>,
}
struct Heads {
    heads: [u32; 256],
    in_out: u32,
}
struct Queues<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    chunks: u32,
    heads: [u32; 256],
    tails: [u32; 256],
    fill: [u32; 256],
    limit: u32,
    over: bool,
}
impl<'a> Queues<'a> {
    fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
        spill.ovf.clear();
        spill.cuts.clear();
        Self {
            in_out: (out.len() / BYTES) as u32,
            out,
            spill,
            chunks: 0,
            heads: [NONE; 256],
            tails: [NONE; 256],
            fill: [CHUNK as u32; 256],
            limit,
            over: false,
        }
    }
    fn heads(&self) -> Heads {
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }
    #[inline(always)]
    fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }
    #[cold]
    #[inline(never)]
    fn push_slow(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
        if self.over || m + w > self.limit {
            self.over = true;
            return;
        }
        let mut left = w;
        while left > 0 {
            if self.fill[a] as usize == CHUNK {
                let id = self.chunks;
                self.chunks += 1;
                if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                    self.spill.ovf.push([0; BYTES]);
                }
                if self.spill.next.len() <= id as usize {
                    self.spill.next.resize(id as usize + 1, NONE);
                }
                self.spill.next[id as usize] = NONE;
                if self.heads[a] == NONE {
                    self.heads[a] = id;
                } else {
                    self.spill.next[self.tails[a] as usize] = id;
                }
                self.tails[a] = id;
                self.fill[a] = 0;
            }
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
            let id = self.tails[a];
            let chunk = if id < self.in_out {
                let at = id as usize * BYTES;
                &mut self.out[at..at + BYTES]
            } else {
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            };
            for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                .chunks_exact_mut(2)
                .enumerate()
            {
                d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
            }
            self.fill[a] += take;
            m += take;
            left -= take;
            if m % EPOCH == 0 {
                let mut cut = *thin;
                cut[a] += w - left;
                self.spill.cuts.push(cut);
            }
        }
    }
}

struct Reader<'a> {
    out: &'a [u8],
    spill: &'a Spill,
    in_out: u32,
    a: usize,
    id: u32,
    at: usize,
    read: u32,
    epoch: u32,
    cut: u32,
}
impl<'a> Reader<'a> {
    fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
        Self {
            out,
            spill,
            in_out: heads.in_out,
            a,
            id: heads.heads[a],
            at: 0,
            read: 0,
            epoch: 0,
            cut: spill.cuts.first().map_or(u32::MAX, |c| c[a]),
        }
    }
    #[inline(always)]
    fn wire(&mut self, dst: &mut [u32], pack: u32) {
        let span = dst.len();
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = pack | (self.epoch << 16);
            let at = self.id as usize * BYTES + 2 * self.at;
            for (e, s) in dst
                .iter_mut()
                .zip(self.out[at..at + 2 * span].chunks_exact(2))
            {
                *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
            }
            self.at += span;
            self.read += span as u32;
        } else {
            self.wire_slow(dst, pack);
        }
    }
    #[cold]
    #[inline(never)]
    fn wire_slow(&mut self, mut dst: &mut [u32], pack: u32) {
        while !dst.is_empty() {
            while self.read == self.cut {
                self.epoch += 1;
                self.cut = self
                    .spill
                    .cuts
                    .get(self.epoch as usize)
                    .map_or(u32::MAX, |c| c[self.a]);
            }
            if self.at == CHUNK {
                self.id = self.spill.next[self.id as usize];
                self.at = 0;
            }
            let take = dst
                .len()
                .min(CHUNK - self.at)
                .min((self.cut - self.read) as usize);
            let hi = pack | (self.epoch << 16);
            let chunk = if self.id < self.in_out {
                let at = self.id as usize * BYTES;
                &self.out[at..at + BYTES]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][..]
            };
            for (e, s) in dst[..take]
                .iter_mut()
                .zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
            {
                *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
            }
            self.at += take;
            self.read += take as u32;
            dst = &mut dst[take..];
        }
    }
}

trait Links {
    const STRIDE: usize;
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
}
struct Narrow;
struct Wide;
impl Links for Narrow {
    const STRIDE: usize = 4;
    const BOTTOM: usize = 1;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] & 0xFFFF
    }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] >> 16
    }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF_0000) | to;
    }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
}
impl Links for Wide {
    const STRIDE: usize = 6;
    const BOTTOM: usize = 2;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side]
    }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side + 1]
    }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side] = to;
    }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
}

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

#[cold]
#[inline(never)]
fn grow(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

impl<L: Links> Elim<'_, L> {
    #[inline(always)]
    fn width(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2]
    }
    #[inline(always)]
    fn set_width(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w;
    }
    #[inline(always)]
    fn word(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1]
    }
    #[inline(always)]
    fn set_word(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w;
    }
    #[inline(always)]
    fn bottom(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::BOTTOM]
    }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, x: u32) {
        self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
    }
    #[inline(always)]
    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes;
        self.nodes += 1;
        self.set_width(id, width);
        self.set_word(id, word);
        id
    }
    #[inline(always)]
    fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
        let (head, tail) = list;
        if *tail == 0 {
            *head = id;
        } else {
            L::set_next(self.a, *tail, side, id);
        }
        L::set_prev(self.a, id, side, *tail);
        *tail = id;
    }
    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, second] {
            if tail == 0 {
                continue;
            }
            L::set_next(self.a, prev, side, head);
            L::set_prev(self.a, head, side, prev);
            prev = tail;
        }
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }
    #[inline(always)]
    fn cut(&mut self, id: u32, side: usize) {
        let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
        L::set_next(self.a, p, side, n);
        L::set_prev(self.a, n, side, p);
    }
    #[inline(always)]
    fn pop_tail(&mut self, id: u32, side: usize) {
        let p = L::prev(self.a, id, side);
        L::set_next(self.a, p, side, 0);
        L::set_prev(self.a, 0, side, p);
    }
    #[inline(always)]
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }
    #[inline(always)]
    fn rule4(&self, w: u32) -> usize {
        self.base + (w - 257) as usize * RULE4
    }
    #[inline(always)]
    fn len4(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            self.a[self.rule4(w) + 2]
        }
    }
    #[inline(always)]
    fn join_m<const GRADED: bool>(&mut self, lh: u32, rh: u32) -> u32 {
        if lh == 0 {
            return rh;
        }
        if rh == 0 {
            return lh;
        }
        if GRADED {
            const ID22_MASK: u32 = (1 << 22) - 1;
            let lid = lh & ID22_MASK;
            let rid = rh & ID22_MASK;
            let ll = lh >> 22;
            let rl = rh >> 22;
            let exact_len = if ll < 1023 && rl < 1023 {
                ll + rl
            } else {
                self.len4(lid) + self.len4(rid)
            };
            let id = ((self.top - self.base) / RULE4) as u32 + 257;
            self.a[self.top..self.top + RULE4].copy_from_slice(&[lid, rid, exact_len, u32::MAX]);
            self.top += RULE4;
            (exact_len.min(1023) << 22) | id
        } else {
            let id = ((self.top - self.base) / RULE4) as u32 + 257;
            let len = self.len4(lh) + self.len4(rh);
            self.a[self.top..self.top + RULE4].copy_from_slice(&[lh, rh, len, u32::MAX]);
            self.top += RULE4;
            id
        }
    }
    fn power_m<const GRADED: bool>(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join_m::<GRADED>(acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = self.join_m::<GRADED>(base, base);
        }
    }
    fn eliminate<const SPIRALS: bool, const GRADED: bool>(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let room = if SPIRALS { SPIRAL_RULES * RULE4 } else { RULE4 };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if SPIRALS && x != y {
                if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    let head = self.power_m::<GRADED>(self.word(x), q);
                    let word = self.join_m::<GRADED>(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    let tail = self.power_m::<GRADED>(self.word(y), q);
                    let word = self.join_m::<GRADED>(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            left -= w;
            if x == y {
                if GRADED {
                    self.pop_tail(x, 0);
                    self.pop_tail(x, 1);
                } else {
                    self.cut(x, 0);
                    self.cut(x, 1);
                }
                continue;
            }
            let word = self.join_m::<GRADED>(self.word(x), self.word(y));
            if aw <= bw {
                if GRADED {
                    self.pop_tail(x, 0);
                } else {
                    self.cut(x, 0);
                }
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    if GRADED {
                        self.pop_tail(y, 1);
                    } else {
                        self.cut(y, 1);
                    }
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                if GRADED {
                    self.pop_tail(y, 1);
                } else {
                    self.cut(y, 1);
                }
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        *remaining = left;
        done
    }

    fn append4(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule4(id);
                let rule = &mut self.a[r..r + RULE4];
                if rule[3] == u32::MAX {
                    rule[3] = dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = rule[3] as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }

    /// Pohl (1971) 2-Way Meet-in-the-Middle walk on circular doubly-linked `side-1` list (`MLP = 2`!):
    /// Reads BOTH forward (`f_id`) and backward (`b_id`) links BEFORE issuing `set_bottom` stores!
    fn compute_bottom_bidi(&mut self, m: u32) {
        let mut f_id = L::next(self.a, 0, 1);
        let mut b_id = L::prev(self.a, 0, 1);
        let mut f_x = 0u32;
        let mut b_x = m;
        while f_id != 0 && f_id != b_id {
            let f_next = L::next(self.a, f_id, 1);
            let b_prev = L::prev(self.a, b_id, 1);
            let f_w = self.width(f_id);
            let b_w = self.width(b_id);
            b_x -= b_w;
            self.set_bottom(f_id, f_x);
            self.set_bottom(b_id, b_x);
            f_x += f_w;
            if f_next == b_id {
                debug_assert_eq!(f_x, b_x);
                f_id = 0;
                break;
            }
            f_id = f_next;
            b_id = b_prev;
        }
        if f_id != 0 && f_id == b_id {
            debug_assert_eq!(f_x + self.width(f_id), b_x);
            self.set_bottom(f_id, f_x);
        }
    }
}

#[derive(Clone, Copy)]
struct First {
    all: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
}
impl Default for First {
    fn default() -> Self {
        Self {
            all: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
        }
    }
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
    let mut pos = 0;
    let mut i = 0;
    while i < records.len() {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < records.len() && (records[i] >> 8) as u8 == c {
            width += u32::from(records[i] & 255);
            i += 1;
        }
        visit(c, pos, width);
        pos += width;
    }
}

#[inline(always)]
fn split(
    c: u8,
    pos: u32,
    width: u32,
    p: u32,
    limit: u32,
    mut visit: impl FnMut(u8, u32, u32, bool),
) {
    let end = pos + width;
    if width > limit && (pos..end).contains(&p) {
        if pos < p {
            visit(c, pos, p - pos, false);
        }
        visit(c, p, 1, true);
        if p + 1 < end {
            visit(c, p + 1, end - p - 1, false);
        }
    } else {
        visit(c, pos, width, width <= limit);
    }
}

struct Pass<'a, const MUX_SB: bool> {
    queues: Queues<'a>,
    strips: &'a mut Vec<Strip>,
    sb_buf: &'a mut Vec<u32>,
    f: First,
}
impl<const MUX_SB: bool> Pass<'_, MUX_SB> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        if width <= limit {
            self.piece(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece(c, pos, width, false, p);
        } else {
            self.take_marked(c, pos, width, p, limit);
        }
    }
    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |c, x, w, keep| {
            self.piece(c, x, w, keep, p)
        });
    }
    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
            self.queues.push(a, f.m, w, &f.thin);
            f.thin[a] += w;
            f.m += w;
        } else {
            if MUX_SB {
                self.sb_buf
                    .extend_from_slice(&[x, f.all[a], w, u32::from(c)]);
            } else {
                self.strips.push(Strip {
                    from: x,
                    to: f.all[a],
                    width: w,
                    byte: c,
                });
            }
            f.thick[a] += 1;
        }
        f.all[a] += w;
    }
    fn strip_count(&self) -> usize {
        if MUX_SB {
            self.sb_buf.len() / 4
        } else {
            self.strips.len()
        }
    }
}

/// `MODE`:
///   - `0`: `M0` (`origin/master` clone)
///   - `1`: `C17` (`GradedRV + Lazy15`, separate `Vec<Strip>` + `Vec<Band>`)
///   - `2`: `B1` (`StripBandMux + BidiBottom` on top of `C17`: `-49..98 KiB` scratch + `2x` MLP `bottom`!)
///   - `3`: `B2` (`FusedS3S4-TailArena` test: `Elim` placed at `edges[m..]` so `S3` wires `edges[0..m)` inline!)
#[derive(Default)]
struct CandKernel<const MODE: usize> {
    stack: Vec<u32>,
    spill: Spill,
    strips: Vec<Strip>,
    bands: Vec<Band>,
    sb_buf: Vec<u32>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    last_s34_us: f64,
}

impl<const MODE: usize> CandKernel<MODE> {
    const MUX_SB: bool = (MODE >= 2);

    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }
    fn wide_links(ns: usize) -> bool {
        Self::nodes_most(ns) > NARROW_NODES
    }
    fn scratch_bytes(&self, out_cap: usize) -> usize {
        out_cap
            + 4 * self.stack.capacity()
            + 4 * self.spill.next.capacity()
            + BYTES * self.spill.ovf.capacity()
            + 1024 * self.spill.cuts.capacity()
            + 16 * self.strips.capacity()
            + 12 * self.bands.capacity()
            + 4 * self.sb_buf.capacity()
            + self.spelling.capacity()
            + 4 * self.offsets.capacity()
            + 4 * self.edges.capacity()
            + 4 * self.extra.capacity()
            + 16 * self.inline16.capacity()
            + self.lens.capacity()
    }

    fn init_inline_literals(&mut self) {
        if self.inline16.len() < 256 {
            self.inline16.clear();
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.resize(256, 1);
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);
    }

    #[inline(always)]
    fn emit_short_or_long<L: Links>(
        e: &mut Elim<'_, L>,
        raw_word: u32,
        a: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
        offsets: &mut Vec<u32>,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
    ) -> u32 {
        const ID22_MASK: u32 = (1 << 22) - 1;
        let node_id = raw_word & ID22_MASK;
        let h_len = raw_word >> 22;
        if h_len == 1 {
            let mut v = [0u8; 16];
            v[0] = (node_id - 1) as u8;
            v[1] = a;
            inline16.push(v);
            lens.push(2);
            offsets.push(spelling.len() as u32);
        } else if h_len == 2 {
            let r = e.rule4(node_id);
            let mut v = [0u8; 16];
            v[0] = (e.a[r] - 1) as u8;
            v[1] = (e.a[r + 1] - 1) as u8;
            v[2] = a;
            inline16.push(v);
            lens.push(3);
            offsets.push(spelling.len() as u32);
        } else if h_len <= 15 {
            let mut v = [0u8; 16];
            let mut st = [0u32; 16];
            st[0] = node_id;
            let mut sp = 1usize;
            let mut vp = 0usize;
            while sp > 0 {
                sp -= 1;
                let cur = st[sp];
                if cur <= 256 {
                    v[vp] = (cur - 1) as u8;
                    vp += 1;
                } else {
                    let r = e.rule4(cur);
                    st[sp] = e.a[r + 1];
                    st[sp + 1] = e.a[r];
                    sp += 2;
                }
            }
            v[vp] = a;
            inline16.push(v);
            lens.push((vp + 1) as u8);
            offsets.push(spelling.len() as u32);
        } else {
            e.append4(node_id, stack, spelling);
            spelling.push(a);
            offsets.push(spelling.len() as u32);
            inline16.push([0u8; 16]);
            lens.push(LONG);
        }
        (inline16.len() - 1) as u32
    }

    #[allow(clippy::too_many_arguments)]
    fn run_s2_s3_s4<L: Links, const TIMED: bool>(
        &mut self,
        n: u32,
        ns: usize,
        first: &[u32; 256],
        thick: &[u32; 256],
        thin: &[u32; 256],
        m: u32,
        bits: u32,
        out_sl: &[u8],
        heads: &Heads,
    ) {
        let graded = MODE >= 1;
        if MODE >= 1 {
            self.init_inline_literals();
        }
        let most = Self::nodes_most(ns);
        let arena_only = ns + (L::STRIDE + RULE4) * most;
        // In `MODE == 3` (`FusedS3S4-TailArena`), place `image_order + arena` AFTER `edges[0..m)`!
        let prefix_off = if MODE == 3 { m as usize } else { 0 };
        let need = if MODE == 3 {
            prefix_off + arena_only
        } else {
            arena_only.max(m as usize)
        };
        if self.edges.len() < need {
            self.edges.clear();
            self.edges.reserve_exact(need);
            self.edges.resize(need, 0);
        }

        let mut write = starts(*thick);
        if Self::MUX_SB {
            for i in 0..ns {
                let a = self.sb_buf[4 * i + 3] as usize;
                self.sb_buf[4 * i + 1] += first[a];
                self.edges[prefix_off + write[a] as usize] = i as u32;
                write[a] += 1;
            }
        } else {
            for (i, s) in self.strips.iter_mut().enumerate() {
                let a = usize::from(s.byte);
                s.to += first[a];
                self.edges[prefix_off + write[a] as usize] = i as u32;
                write[a] += 1;
            }
        }

        let (_, tail_region) = self.edges.split_at_mut(prefix_off);
        let (image_order, arena) = tail_region.split_at_mut(ns);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0u32;
        if Self::MUX_SB {
            for chunk in self.sb_buf.chunks_exact(4) {
                let s_w = chunk[2];
                let s_b = chunk[3];
                e.node(s_w, (1u32 << 22) | (s_b + 1));
                widest = widest.max(s_w);
            }
        } else {
            for s in self.strips.iter() {
                let lit = if graded {
                    (1u32 << 22) | (u32::from(s.byte) + 1)
                } else {
                    u32::from(s.byte) + 1
                };
                e.node(s.width, lit);
                widest = widest.max(s.width);
            }
        }

        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0u32;
        if Self::MUX_SB {
            for (i, chunk) in self.sb_buf.chunks_exact(4).enumerate() {
                let s_from = chunk[0];
                let s_w = chunk[2];
                if x < s_from {
                    widest = widest.max(s_from - x);
                    let id = e.node(s_from - x, 0);
                    e.append_to(&mut order[0], id, 0);
                    e.append_to(&mut terminals[1], id, 1);
                }
                e.append_to(&mut order[0], i as u32 + 1, 0);
                x = s_from + s_w;
            }
        } else {
            for (i, s) in self.strips.iter().enumerate() {
                if x < s.from {
                    widest = widest.max(s.from - x);
                    let id = e.node(s.from - x, 0);
                    e.append_to(&mut order[0], id, 0);
                    e.append_to(&mut terminals[1], id, 1);
                }
                e.append_to(&mut order[0], i as u32 + 1, 0);
                x = s.from + s.width;
            }
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }

        x = 0;
        if Self::MUX_SB {
            for &i in image_order.iter() {
                let base_i = 4 * (i as usize);
                let s_to = self.sb_buf[base_i + 1];
                let s_w = self.sb_buf[base_i + 2];
                if x < s_to {
                    widest = widest.max(s_to - x);
                    let id = e.node(s_to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = s_to + s_w;
            }
            // `Strips` in `sb_buf` are NOW 100% DEAD — clear `sb_buf` to reuse its L2-warm buffer for `Band`s!
            self.sb_buf.clear();
        } else {
            for &i in image_order.iter() {
                let s = self.strips[i as usize];
                if x < s.to {
                    widest = widest.max(s.to - x);
                    let id = e.node(s.to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = s.to + s.width;
            }
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[1], id, 1);
            e.append_to(&mut terminals[0], id, 0);
        }
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        let mut remaining = n;
        let spirals = widest > CERTIFIED_WIDTH;
        let room = if spirals { SPIRAL_RULES * RULE4 } else { RULE4 };
        while !match (spirals, graded) {
            (false, false) => e.eliminate::<false, false>(&mut remaining),
            (true, false) => e.eliminate::<true, false>(&mut remaining),
            (false, true) => e.eliminate::<false, true>(&mut remaining),
            (true, true) => e.eliminate::<true, true>(&mut remaining),
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(&mut self.edges, prefix_off + ns + top + room);
            e = Elim {
                a: &mut self.edges[prefix_off + ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }

        // === STAGE S3 + S4 ===
        let t_s34 = if TIMED { Some(Instant::now()) } else { None };
        if MODE >= 2 {
            e.compute_bottom_bidi(m);
        } else {
            let mut id = L::next(e.a, 0, 1);
            x = 0;
            while id != 0 {
                let next = L::next(e.a, id, 1);
                let w = e.width(id);
                e.set_bottom(id, x);
                x += w;
                id = next;
            }
            assert_eq!(x, m);
        }

        // Check whether `MODE == 3` (`FusedS3S4-TailArena`) can guarantee `!wide` upfront:
        let (e_base, e_top, e_nodes) = (e.base, e.top, e.nodes);
        let max_words_bound = e_nodes; // at most 1 word per surviving side-0 node + <=256 symbol splits
        let fused_direct = MODE == 3 && (255 + max_words_bound + 256) <= (u32::MAX >> bits);

        if fused_direct {
            self.extra.clear();
            let (edges_out, tail_reg) = self.edges.split_at_mut(m as usize);
            let mut e_f = Elim::<L> {
                a: &mut tail_reg[ns..],
                base: e_base,
                top: e_top,
                nodes: e_nodes,
                _links: std::marker::PhantomData,
            };
            let mut id = L::next(e_f.a, 0, 0);
            let mut in_node = 0u32;
            let mut k = 0u32;
            for a in 0..256usize {
                let end = k + thin[a];
                let mut queue = Reader::new(out_sl, &self.spill, heads, a);
                while k < end {
                    let node_width = e_f.width(id);
                    let raw_word = e_f.word(id);
                    let width = (end - k).min(node_width - in_node);
                    let dst = (e_f.bottom(id) + in_node) as usize;
                    let code = if raw_word == 0 {
                        a as u32
                    } else {
                        Self::emit_short_or_long(
                            &mut e_f,
                            raw_word,
                            a as u8,
                            &mut self.stack,
                            &mut self.spelling,
                            &mut self.offsets,
                            &mut self.inline16,
                            &mut self.lens,
                        )
                    };
                    queue.wire(&mut edges_out[dst..dst + width as usize], code << bits);
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = L::next(e_f.a, id, 0);
                        in_node = 0;
                    }
                }
            }
            if let Some(t0) = t_s34 {
                self.last_s34_us = t0.elapsed().as_nanos() as f64 / 1e3;
            }
            return;
        }

        // Sequential `S3 -> S4` (`MODE 0, 1, 2`, plus fallback if `MODE 3` was `wide`):
        let mut cursors = [0usize; 256];
        let mut words = 0usize;
        let mut id = L::next(e.a, 0, 0);
        let mut in_node = 0u32;
        let mut k = 0u32;
        self.bands.clear();
        for a in 0..256usize {
            cursors[a] = if Self::MUX_SB {
                self.sb_buf.len() / 3
            } else {
                self.bands.len()
            };
            let end = k + thin[a];
            while k < end {
                let node_width = e.width(id);
                let raw_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                let dst = e.bottom(id) + in_node;
                let code = if raw_word == 0 {
                    a as u32
                } else {
                    words += 1;
                    if MODE == 0 {
                        e.append4(raw_word, &mut self.stack, &mut self.spelling);
                        self.spelling.push(a as u8);
                        self.offsets.push(self.spelling.len() as u32);
                        254 + self.offsets.len() as u32
                    } else {
                        Self::emit_short_or_long(
                            &mut e,
                            raw_word,
                            a as u8,
                            &mut self.stack,
                            &mut self.spelling,
                            &mut self.offsets,
                            &mut self.inline16,
                            &mut self.lens,
                        )
                    }
                };
                if Self::MUX_SB {
                    self.sb_buf
                        .extend_from_slice(&[k + width, dst.wrapping_sub(k), code]);
                } else {
                    self.bands.push(Band {
                        end: k + width,
                        delta: dst.wrapping_sub(k),
                        code,
                    });
                }
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }

        let wide = (255 + words as u32) > (u32::MAX >> bits);
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        k = 0;
        if Self::MUX_SB {
            let total_bands = self.sb_buf.len() / 3;
            for a in 0..256usize {
                let end_b = cursors.get(a + 1).copied().unwrap_or(total_bands);
                let mut queue = Reader::new(out_sl, &self.spill, heads, a);
                for tr in self.sb_buf[3 * cursors[a]..3 * end_b].chunks_exact(3) {
                    let b_end = tr[0];
                    let b_delta = tr[1];
                    let b_code = tr[2];
                    let span = (b_end - k) as usize;
                    let dst = k.wrapping_add(b_delta) as usize;
                    let pack = if wide { 0 } else { b_code << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(b_code);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                    k = b_end;
                }
            }
        } else {
            for a in 0..256usize {
                let end_b = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = Reader::new(out_sl, &self.spill, heads, a);
                for b in &self.bands[cursors[a]..end_b] {
                    let span = (b.end - k) as usize;
                    let dst = k.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(b.code);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                    k = b.end;
                }
            }
        }

        if MODE == 0 {
            self.init_inline_literals();
            let n_w = self.offsets.len().saturating_sub(1);
            self.inline16.reserve_exact(n_w);
            self.lens.reserve_exact(n_w);
            for w in self.offsets.windows(2) {
                let word = &self.spelling[w[0] as usize..w[1] as usize];
                let mut v = [0; INLINE];
                let head = word.len().min(INLINE);
                v[..head].copy_from_slice(&word[..head]);
                self.inline16.push(v);
                self.lens
                    .push(if word.len() <= INLINE { word.len() as u8 } else { LONG });
            }
        }
        if let Some(t0) = t_s34 {
            self.last_s34_us = t0.elapsed().as_nanos() as f64 / 1e3;
        }
    }

    fn try_run<const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<()> {
        out.resize(n + INLINE, 0);
        self.strips.clear();
        self.sb_buf.clear();
        let mux_sb = Self::MUX_SB;
        let (f, heads, over, ns) = if mux_sb {
            let mut pass = Pass::<true> {
                queues: Queues::new(&mut out[..], &mut self.spill, cap.m),
                strips: &mut self.strips,
                sb_buf: &mut self.sb_buf,
                f: First::default(),
            };
            maximal(records, |c, pos, width| {
                pass.take(c, pos, width, p, LIMIT);
            });
            let ns = pass.strip_count();
            (pass.f, pass.queues.heads(), pass.queues.over, ns)
        } else {
            let mut pass = Pass::<false> {
                queues: Queues::new(&mut out[..], &mut self.spill, cap.m),
                strips: &mut self.strips,
                sb_buf: &mut self.sb_buf,
                f: First::default(),
            };
            maximal(records, |c, pos, width| {
                pass.take(c, pos, width, p, LIMIT);
            });
            let ns = pass.strip_count();
            (pass.f, pass.queues.heads(), pass.queues.over, ns)
        };
        if over || f.m > cap.m || ns > cap.strips {
            self.strips.clear();
            self.strips.shrink_to(cap.strips);
            self.sb_buf.clear();
            self.sb_buf.shrink_to(4 * cap.strips);
            return None;
        }

        let First {
            all,
            thin,
            thick,
            m,
            mark,
        } = f;
        let first = starts(all);
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        if Self::wide_links(ns) {
            self.run_s2_s3_s4::<Wide, TIMED>(
                n as u32,
                ns,
                &first,
                &thick,
                &thin,
                m,
                bits,
                &out[..],
                &heads,
            );
        } else {
            self.run_s2_s3_s4::<Narrow, TIMED>(
                n as u32,
                ns,
                &first,
                &thick,
                &thin,
                m,
                bits,
                &out[..],
                &heads,
            );
        }

        // 100% Verbatim `origin/master` `emit`!
        out.resize(n + INLINE, 0);
        let mut at = 0;
        let mask = (1 << bits) - 1;
        let mut j = mark;
        let edges_sl = &self.edges[..m as usize];
        for _ in 0..edges_sl.len() {
            let edge = edges_sl[j as usize];
            let code = if self.extra.is_empty() {
                edge >> bits
            } else {
                self.extra[j as usize]
            } as usize;
            j = edge & mask;
            let len = self.lens[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&self.inline16[code]);
                at += usize::from(len);
            } else {
                let w = code - 256;
                let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if j == mark || at >= n {
                break;
            }
        }
        out.truncate(at.min(n));
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
        Some(())
    }
}

struct CandOpenCells<const MODE: usize, const FORCED: bool = false> {
    open: CandKernel<MODE>,
    lazy: LazyCells,
}
impl<const MODE: usize, const FORCED: bool> Default for CandOpenCells<MODE, FORCED> {
    fn default() -> Self {
        Self {
            open: CandKernel::default(),
            lazy: LazyCells::default(),
        }
    }
}
impl<const MODE: usize, const FORCED: bool> CandOpenCells<MODE, FORCED> {
    fn cap(records: usize, n: usize, p: u32) -> Option<Cap> {
        if n == 0 || n > 900_000 || p as usize >= n {
            None
        } else if FORCED || lazy_counts_admit(records, n) {
            Some(Cap::NONE)
        } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
            Some(Cap::dense(n))
        } else {
            None
        }
    }
}
impl<const MODE: usize, const FORCED: bool> InverseBwt for CandOpenCells<MODE, FORCED> {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if let Some(cap) = Self::cap(r.records().len(), n, p) {
            if self
                .open
                .try_run::<false>(r.records(), n, p, cap, out)
                .is_some()
            {
                return true;
            }
        }
        self.lazy.invert_runs(r, n, p, out, k)
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(l, c, p, out);
    }
}

#[derive(Default)]
struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}
impl BitWriter {
    fn put(&mut self, v: u64, bits: u32) {
        for i in (0..bits).rev() {
            self.acc = (self.acc << 1) | ((v >> i) & 1);
            self.n += 1;
            if self.n == 8 {
                self.out.push(self.acc as u8);
                self.acc = 0;
                self.n = 0;
            }
        }
    }
    fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.put(0, 8 - self.n);
        }
        self.out
    }
}

fn plain_invert(ll: &[u8], orig_ptr: u32) -> Vec<u8> {
    let n = ll.len();
    let mut counts = [0usize; 256];
    for &c in ll {
        counts[c as usize] += 1;
    }
    let mut cum = [0usize; 256];
    let mut s = 0;
    for i in 0..256 {
        cum[i] = s;
        s += counts[i];
    }
    let mut psi = vec![0u32; n];
    for (i, &c) in ll.iter().enumerate() {
        psi[cum[c as usize]] = i as u32;
        cum[c as usize] += 1;
    }
    let mut p = psi[orig_ptr as usize];
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(ll[p as usize]);
        p = psi[p as usize];
    }
    out
}

fn mtf_symbols(ll: &[u8], used: &[u8]) -> Vec<u16> {
    let mut list: Vec<u8> = used.to_vec();
    let mut syms = Vec::with_capacity(ll.len());
    let mut run = 0u32;
    let flush = |run: &mut u32, syms: &mut Vec<u16>| {
        if *run > 0 {
            let mut z = *run - 1;
            loop {
                syms.push((z & 1) as u16);
                if z < 2 {
                    break;
                }
                z = (z - 2) / 2;
            }
            *run = 0;
        }
    };
    for &c in ll {
        let j = list.iter().position(|&x| x == c).unwrap();
        if j == 0 {
            run += 1;
            continue;
        }
        flush(&mut run, &mut syms);
        list.remove(j);
        list.insert(0, c);
        syms.push(j as u16 + 1);
    }
    flush(&mut run, &mut syms);
    syms.push(used.len() as u16 + 1);
    syms
}

fn craft_bz2_block(ll: &[u8], orig_ptr: u32, kernels: Kernels) -> Vec<u8> {
    let pre_rle1 = plain_invert(ll, orig_ptr);
    let mut unrled = Vec::with_capacity(pre_rle1.len());
    let crc = mibz2::rle::unrle(&pre_rle1, &mut unrled, kernels)
        .map(|_| mibz2::crc::crc32(&unrled, kernels))
        .unwrap_or(0);
    let mut seen = [false; 256];
    for &c in ll {
        seen[c as usize] = true;
    }
    let used: Vec<u8> = (0..=255u8).filter(|&c| seen[c as usize]).collect();
    let syms = mtf_symbols(ll, &used);
    let alpha = used.len() + 2;
    let code_len = (usize::BITS - (alpha - 1).leading_zeros()).max(1);
    let mut w = BitWriter::default();
    for &b in b"BZh9" {
        w.put(b as u64, 8);
    }
    w.put(0x3141_5926_5359, 48);
    w.put(crc as u64, 32);
    w.put(0, 1);
    w.put(orig_ptr as u64, 24);
    let mut in16 = 0u64;
    for &c in &used {
        in16 |= 0x8000 >> (c / 16);
    }
    w.put(in16, 16);
    for i in 0..16 {
        if in16 & (0x8000 >> i) != 0 {
            let mut word = 0u64;
            for &c in &used {
                if (c / 16) as u64 == i {
                    word |= 0x8000 >> (c % 16);
                }
            }
            w.put(word, 16);
        }
    }
    let n_sel = syms.len().div_ceil(50).max(1);
    w.put(2, 3);
    w.put(n_sel as u64, 15);
    for _ in 0..n_sel {
        w.put(0, 1);
    }
    for _ in 0..2 {
        w.put(code_len as u64, 5);
        for _ in 0..alpha {
            w.put(0, 1);
        }
    }
    for &s in &syms {
        w.put(s as u64, code_len);
    }
    w.put(0x1772_4538_5090, 48);
    w.put(crc as u64, 32);
    w.finish()
}

fn extract_runs(bz2: &[u8], kernels: Kernels) -> (Runs, usize, u32) {
    let mut sc = Scanner::new(SliceSource(bz2));
    let mut runs = Runs::default();
    match sc.next(Vec::new()).expect("valid bz2") {
        Event::Block { symbols, .. } => {
            let p = symbols.orig_ptr;
            let n = runs::front(&symbols, &mut runs, kernels).expect("front ok");
            (runs, n, p)
        }
        _ => panic!("expected block"),
    }
}

fn bwt_of_rle1_safe(raw: &[u8]) -> (Vec<u8>, u32) {
    let mut text = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        let b = raw[i];
        let mut run = 1usize;
        while i + run < raw.len() && raw[i + run] == b && run < 255 {
            run += 1;
        }
        if run >= 4 {
            text.extend_from_slice(&[b, b, b, b, (run - 4) as u8]);
        } else {
            for _ in 0..run {
                text.push(b);
            }
        }
        i += run;
    }
    let n = text.len();
    let mut sa: Vec<u32> = (0..n as u32).collect();
    sa.sort_by(|&a, &b| {
        let mut ia = a as usize;
        let mut ib = b as usize;
        for _ in 0..n {
            let ca = text[ia];
            let cb = text[ib];
            if ca != cb {
                return ca.cmp(&cb);
            }
            ia += 1;
            if ia == n {
                ia = 0;
            }
            ib += 1;
            if ib == n {
                ib = 0;
            }
        }
        std::cmp::Ordering::Equal
    });
    let mut ll = vec![0u8; n];
    let mut p = 0u32;
    for (idx, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = idx as u32;
            ll[idx] = text[n - 1];
        } else {
            ll[idx] = text[pos as usize - 1];
        }
    }
    (ll, p)
}

fn build_dense_703_block() -> (Vec<u8>, u32) {
    let mut raw = Vec::with_capacity(610_000);
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    for idx in 0..8_500u32 {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        let h1 = (s >> 32) as u32;
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        let h2 = (s >> 48) as u16;
        let line = format!(
            "{{\"id\":\"Q{}\",\"claim\":\"P{}\",\"hash\":\"{:08x}{:02x}\",\"rank\":\"normal\"}}\n",
            idx,
            idx % 97,
            h1,
            h2 & 0xFF
        );
        raw.extend_from_slice(line.as_bytes());
    }
    bwt_of_rle1_safe(&raw)
}

fn verify_gate_g1(kernels: Kernels) -> usize {
    let mut cases = 0usize;
    let mut m_ref = ForcedOpenCells::default();
    let mut c1 = CandOpenCells::<1, true>::default();
    let mut c2 = CandOpenCells::<2, true>::default();
    let mut c3 = CandOpenCells::<3, true>::default();
    let mut o_ref = Vec::new();
    let mut o1 = Vec::new();
    let mut o2 = Vec::new();
    let mut o3 = Vec::new();

    // Exhaustive 1..=8 binary and 1..=5 ternary test 0-node, 1-node, 2-node, 3-node `BidiBottom`!
    for n in 1..=8usize {
        for mask in 0..(1u32 << n) {
            let mut ll = Vec::with_capacity(n);
            for bit in 0..n {
                ll.push(b'a' + ((mask >> bit) & 1) as u8);
            }
            for p in 0..n as u32 {
                let bz2 = craft_bz2_block(&ll, p, kernels);
                let (runs_obj, n_dec, p_dec) = extract_runs(&bz2, kernels);
                assert!(m_ref.invert_runs(&runs_obj, n_dec, p_dec, &mut o_ref, kernels));
                assert!(c1.invert_runs(&runs_obj, n_dec, p_dec, &mut o1, kernels));
                assert!(c2.invert_runs(&runs_obj, n_dec, p_dec, &mut o2, kernels));
                assert!(c3.invert_runs(&runs_obj, n_dec, p_dec, &mut o3, kernels));
                assert_eq!(o1, o_ref);
                assert_eq!(o2, o_ref);
                assert_eq!(o3, o_ref);
                cases += 1;
            }
        }
    }
    for n in 1..=5usize {
        let mut tot = 1u32;
        for _ in 0..n {
            tot *= 3;
        }
        for code in 0..tot {
            let mut x = code;
            let mut ll = Vec::with_capacity(n);
            for _ in 0..n {
                ll.push(b'a' + (x % 3) as u8);
                x /= 3;
            }
            for p in 0..n as u32 {
                let bz2 = craft_bz2_block(&ll, p, kernels);
                let (runs_obj, n_dec, p_dec) = extract_runs(&bz2, kernels);
                assert!(m_ref.invert_runs(&runs_obj, n_dec, p_dec, &mut o_ref, kernels));
                assert!(c1.invert_runs(&runs_obj, n_dec, p_dec, &mut o1, kernels));
                assert!(c2.invert_runs(&runs_obj, n_dec, p_dec, &mut o2, kernels));
                assert!(c3.invert_runs(&runs_obj, n_dec, p_dec, &mut o3, kernels));
                assert_eq!(o1, o_ref);
                assert_eq!(o2, o_ref);
                assert_eq!(o3, o_ref);
                cases += 1;
            }
        }
    }

    let mut raw_m = Vec::with_capacity(5_000);
    for i in 0..250u32 {
        raw_m.extend_from_slice(format!("{{\"item\":{},\"tag\":\"wikidata_claim_p{}\"}}\n", i % 17, i % 5).as_bytes());
    }
    let (ll_m, p_m) = bwt_of_rle1_safe(&raw_m);
    let bz2_m = craft_bz2_block(&ll_m, p_m, kernels);
    let (runs_m, n_m, _) = extract_runs(&bz2_m, kernels);
    assert!(m_ref.invert_runs(&runs_m, n_m, p_m, &mut o_ref, kernels));
    assert!(c2.invert_runs(&runs_m, n_m, p_m, &mut o2, kernels));
    assert!(c3.invert_runs(&runs_m, n_m, p_m, &mut o3, kernels));
    assert_eq!(o2, o_ref);
    assert_eq!(o3, o_ref);
    cases += 1;

    let mut ll_sp = Vec::with_capacity(50_020);
    ll_sp.extend(std::iter::repeat_n(b'b', 50_000));
    ll_sp.extend_from_slice(b"abacabadabacabae");
    let bz2_sp = craft_bz2_block(&ll_sp, 50_005, kernels);
    let (runs_sp, n_sp, p_sp) = extract_runs(&bz2_sp, kernels);
    assert!(m_ref.invert_runs(&runs_sp, n_sp, p_sp, &mut o_ref, kernels));
    assert!(c2.invert_runs(&runs_sp, n_sp, p_sp, &mut o2, kernels));
    assert!(c3.invert_runs(&runs_sp, n_sp, p_sp, &mut o3, kernels));
    assert_eq!(o2, o_ref);
    assert_eq!(o3, o_ref);
    cases += 1;

    cases
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/antigravity/admitted-bwt".to_string());
    let kernels = Kernels::detect();

    let g1_cases = verify_gate_g1(kernels);
    println!(
        "Gate G1 + BidiBottom (0/1/2/3/N nodes) + StripBandMux + SPIRALS Verification: PASS ({} cases byte-exact)\n",
        g1_cases
    );

    let auth: [(&str, usize, u32); 6] = [
        ("block-0.L", 899_931, 264_045),
        ("block-2.L", 900_000, 320_807),
        ("block-85.L", 899_935, 325_088),
        ("block-143.L", 899_869, 802_868),
        ("block-210.L", 899_959, 791_429),
        ("block-207.L", 899_998, 205_180),
    ];
    let mut suite: Vec<(String, Runs, usize, u32)> = Vec::new();
    for &(name, n_exp, p) in &auth {
        let ll = fs::read(format!("{}/{}", dir, name)).expect("read");
        let bz2 = craft_bz2_block(&ll, p, kernels);
        let (r, n_dec, p_dec) = extract_runs(&bz2, kernels);
        assert_eq!((n_dec, p_dec), (n_exp, p));
        suite.push((name.to_string(), r, n_exp, p));
    }
    let (ll_d, p_d) = build_dense_703_block();
    let bz2_d = craft_bz2_block(&ll_d, p_d, kernels);
    let (r_d, n_d, _) = extract_runs(&bz2_d, kernels);
    suite.push(("dense-703.syn".to_string(), r_d, n_d, p_d));

    println!("=== CYCLE 18A: STAGE S3+S4 COMBINED TIMING & PER-WORKER WORKING-SET CENSUS (KiB) ===");
    println!(
        "{:<13} | {:>20} | {:>10} | {:>10} | {:>10} | {:>10}",
        "Block",
        "WS_KB(M0/B1_Mux/B2)",
        "M0(S34 us)",
        "C17(S34)",
        "B1:MuxBidi",
        "B2:Fused34"
    );
    println!("{}", "-".repeat(94));

    let mut m_a1 = OpenCells::default();
    let mut m_a2 = OpenCells::default();
    let mut c_m0 = CandOpenCells::<0>::default();
    let mut c_c17 = CandOpenCells::<1>::default();
    let mut c_b1 = CandOpenCells::<2>::default();
    let mut c_b2 = CandOpenCells::<3>::default();

    let mut o_a1 = vec![0u8; 900_016];
    let mut o_a2 = vec![0u8; 900_016];
    let mut o_m0 = vec![0u8; 900_016];
    let mut o_c17 = vec![0u8; 900_016];
    let mut o_b1 = vec![0u8; 900_016];
    let mut o_b2 = vec![0u8; 900_016];

    let mut stage_rows = Vec::new();
    for (name, runs_obj, n, p) in &suite {
        // Fresh per-block scratch census first:
        let mut f_m0 = CandOpenCells::<0>::default();
        let mut f_b1 = CandOpenCells::<2>::default();
        let mut f_b2 = CandOpenCells::<3>::default();
        let mut fo = vec![0u8; *n + INLINE];
        let cap = CandOpenCells::<0>::cap(runs_obj.records().len(), *n, *p).unwrap();
        f_m0.open.try_run::<false>(runs_obj.records(), *n, *p, cap, &mut fo).unwrap();
        f_b1.open.try_run::<false>(runs_obj.records(), *n, *p, cap, &mut fo).unwrap();
        f_b2.open.try_run::<false>(runs_obj.records(), *n, *p, cap, &mut fo).unwrap();
        let ws_m0 = f_m0.open.scratch_bytes(*n + INLINE);
        let ws_b1 = f_b1.open.scratch_bytes(*n + INLINE);
        let ws_b2 = f_b2.open.scratch_bytes(*n + INLINE);

        let mut s0 = Vec::with_capacity(11);
        let mut sc = Vec::with_capacity(11);
        let mut s1 = Vec::with_capacity(11);
        let mut s2 = Vec::with_capacity(11);
        for r in 0..13 {
            c_m0.open.try_run::<true>(runs_obj.records(), *n, *p, cap, &mut o_m0).unwrap();
            c_c17.open.try_run::<true>(runs_obj.records(), *n, *p, cap, &mut o_c17).unwrap();
            c_b1.open.try_run::<true>(runs_obj.records(), *n, *p, cap, &mut o_b1).unwrap();
            c_b2.open.try_run::<true>(runs_obj.records(), *n, *p, cap, &mut o_b2).unwrap();
            if r >= 2 {
                s0.push(c_m0.open.last_s34_us);
                sc.push(c_c17.open.last_s34_us);
                s1.push(c_b1.open.last_s34_us);
                s2.push(c_b2.open.last_s34_us);
            }
        }
        for v in [&mut s0, &mut sc, &mut s1, &mut s2] {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }
        println!(
            "{:<13} | {:>5}/{:>5}/{:>5}KB | {:>8.1}us | {:>8.1}us | {:>8.1}us | {:>8.1}us",
            name,
            ws_m0 / 1024,
            ws_b1 / 1024,
            ws_b2 / 1024,
            s0[5],
            sc[5],
            s1[5],
            s2[5]
        );
        stage_rows.push((ws_m0, ws_b1, ws_b2, s0[5], sc[5], s1[5], s2[5]));
    }

    println!("\n=== CYCLE 18B: NATIVE SAFE-RUST END-TO-END A/B + A/A + M0 + C17 CONFRONTATION (`21` ROUNDS) ===");
    println!(
        "{:<13} | {:>9} | {:>7} | {:>9} | {:>9} | {:>9} | {:>7} | {:>7} | {:>9} | {:>7}",
        "Block",
        "A1:Master",
        "A2/A1",
        "M0:Clone",
        "C17:Lazy",
        "B1:MuxBid",
        "B1/M0",
        "B1/A1",
        "B2:Fused",
        "B2/M0"
    );
    println!("{}", "-".repeat(118));

    let mut json_rows = Vec::new();
    for (idx, (name, runs_obj, n, p)) in suite.iter().enumerate() {
        assert!(m_a1.invert_runs(runs_obj, *n, *p, &mut o_a1, kernels));
        assert!(m_a2.invert_runs(runs_obj, *n, *p, &mut o_a2, kernels));
        assert!(c_m0.invert_runs(runs_obj, *n, *p, &mut o_m0, kernels));
        assert!(c_c17.invert_runs(runs_obj, *n, *p, &mut o_c17, kernels));
        assert!(c_b1.invert_runs(runs_obj, *n, *p, &mut o_b1, kernels));
        assert!(c_b2.invert_runs(runs_obj, *n, *p, &mut o_b2, kernels));
        assert_eq!(o_a1, o_a2);
        assert_eq!(o_m0, o_a1);
        assert_eq!(o_c17, o_a1);
        assert_eq!(o_b1, o_a1);
        assert_eq!(o_b2, o_a1);

        let rounds = 21usize;
        let mut t_a1 = Vec::with_capacity(rounds);
        let mut t_a2 = Vec::with_capacity(rounds);
        let mut t_m0 = Vec::with_capacity(rounds);
        let mut t_c17 = Vec::with_capacity(rounds);
        let mut t_b1 = Vec::with_capacity(rounds);
        let mut t_b2 = Vec::with_capacity(rounds);

        for r in 0..(rounds + 2) {
            for slot in 0..6 {
                let arm = (r + slot) % 6;
                let t0 = Instant::now();
                match arm {
                    0 => assert!(m_a1.invert_runs(runs_obj, *n, *p, &mut o_a1, kernels)),
                    1 => assert!(m_a2.invert_runs(runs_obj, *n, *p, &mut o_a2, kernels)),
                    2 => assert!(c_m0.invert_runs(runs_obj, *n, *p, &mut o_m0, kernels)),
                    3 => assert!(c_c17.invert_runs(runs_obj, *n, *p, &mut o_c17, kernels)),
                    4 => assert!(c_b1.invert_runs(runs_obj, *n, *p, &mut o_b1, kernels)),
                    _ => assert!(c_b2.invert_runs(runs_obj, *n, *p, &mut o_b2, kernels)),
                }
                let dt = t0.elapsed().as_nanos() as f64 / 1e3;
                if r >= 2 {
                    match arm {
                        0 => t_a1.push(dt),
                        1 => t_a2.push(dt),
                        2 => t_m0.push(dt),
                        3 => t_c17.push(dt),
                        4 => t_b1.push(dt),
                        _ => t_b2.push(dt),
                    }
                }
            }
        }
        for v in [&mut t_a1, &mut t_a2, &mut t_m0, &mut t_c17, &mut t_b1, &mut t_b2] {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }
        let ma1 = t_a1[rounds / 2];
        let ma2 = t_a2[rounds / 2];
        let mm0 = t_m0[rounds / 2];
        let mc17 = t_c17[rounds / 2];
        let mb1 = t_b1[rounds / 2];
        let mb2 = t_b2[rounds / 2];

        println!(
            "{:<13} | {:>7.1}us | {:>6.3}x | {:>7.1}us | {:>7.1}us | {:>7.1}us | {:>6.3}x | {:>6.3}x | {:>7.1}us | {:>6.3}x",
            name,
            ma1, ma2 / ma1,
            mm0,
            mc17,
            mb1, mb1 / mm0, mb1 / ma1,
            mb2, mb2 / mm0
        );

        let sr = stage_rows[idx];
        json_rows.push(format!(
            "    {{\n      \"block\": \"{}\", \"ws_bytes\": {{\"m0\": {}, \"b1_mux_bidi\": {}, \"b2_fused_tail\": {}}},\n      \"s34_us\": {{\"m0\": {:.2}, \"c17\": {:.2}, \"b1_mux_bidi\": {:.2}, \"b2_fused_tail\": {:.2}}},\n      \"e2e_us\": {{\"a1_master\": {:.2}, \"a2_ctrl\": {:.2}, \"m0_clone\": {:.2}, \"c17_lazy\": {:.2}, \"b1_mux_bidi\": {:.2}, \"b2_fused_tail\": {:.2}}},\n      \"ratios\": {{\"a2_a1\": {:.4}, \"c17_m0\": {:.4}, \"b1_m0\": {:.4}, \"b1_a1\": {:.4}, \"b2_m0\": {:.4}}}\n    }}",
            name, sr.0, sr.1, sr.2,
            sr.3, sr.4, sr.5, sr.6,
            ma1, ma2, mm0, mc17, mb1, mb2,
            ma2 / ma1, mc17 / mm0, mb1 / mm0, mb1 / ma1, mb2 / mm0
        ));
    }

    if let Some(json_path) = std::env::args().nth(2) {
        let j = format!(
            "{{\n  \"gate_g1_cases\": {},\n  \"blocks\": [\n{}\n  ]\n}}\n",
            g1_cases,
            json_rows.join(",\n")
        );
        fs::write(json_path, j).expect("write json");
    }
}
