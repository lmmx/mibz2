#![forbid(unsafe_code)]
//! Wormhole Cycle 14 — Wormhole Route 2 for `Stage S1 + S4`:
//! Eliminating the 256-Queue Scatter via Kakutani (1943) Conjugate Thin-Run Streams in Dead `out`
//!   - `A1`, `A2`: `origin/master` (`90e671966`) `mibz2::bwt::OpenCells` (`OutQ16`, A/A control).
//!   - `B1` (`Unqueued`, `QUEUE = false` from `src/open.rs:486-514`): `0` queue writes in `S1`,
//!     re-scans `records` in `S4` across `256` `stream[a]` / `cursors[a]` positions.
//!   - `B2` (`Kakutani-1Stream`): Kakutani (1943) induced-return single sequential `(sym: u8, w: u8)`
//!     thin-run log aliased in `out[0..2*r_thin]` (`0 B` scratch on all admitted blocks, `1` active
//!     `4 KiB` page in `S1`, `0` `maximal`/`split` re-scan in `S4`, single-band fast path).
//!   - `B3` (`Radix16-Kakutani`): McIlroy–Bostic–McIlroy (1993) 16-bucket nibble-grouped `u32`
//!     run-token super-queues in `out` (`(sym_lo4 << 28) | ((w - 1) << 24) | m24`), using only `16`
//!     active streams (`<= 32`-page streamer & `<= 64`-entry DTLB in BOTH `S1` and `S4`!) and `0`
//!     `Spill::cuts` epoch checks!
//!
//! Stages `S2`, `S3`, and `S5` (`emit`) are 100% verbatim identical to `origin/master`.

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
const R16_CHUNK: usize = 256; // 256 u32s = 1024 B = BYTES!
const NONE: u32 = u32::MAX;
const EPOCH: u32 = 1 << 16;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
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
    ovf_log2: Vec<[u8; 2]>,
}

// ============================================================================
// Mode 0 (`OutQ16`), Mode 2 (`Kakutani-1Stream`), Mode 3 (`Radix16-Kakutani`) State
// ============================================================================
struct Transport<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    chunks: u32,
    heads256: [u32; 256],
    tails256: [u32; 256],
    fill256: [u32; 256],
    // Mode 2 (`Kakutani-1Stream`) cursor (in thin-run pairs):
    log_runs: usize,
    // Mode 3 (`Radix16-Kakutani`) 16 super-queues (`192 B` total state!):
    heads16: [u32; 16],
    tails16: [u32; 16],
    fill16: [u32; 16],
    counts16: [u32; 16],
    limit: u32,
    over: bool,
}

struct SavedTransport {
    in_out: u32,
    heads256: [u32; 256],
    log_runs: usize,
    heads16: [u32; 16],
    counts16: [u32; 16],
}

impl<'a> Transport<'a> {
    fn new<const MODE: usize>(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
        spill.ovf.clear();
        spill.cuts.clear();
        spill.ovf_log2.clear();
        Self {
            in_out: (out.len() / BYTES) as u32,
            out,
            spill,
            chunks: 0,
            heads256: if MODE == 0 { [NONE; 256] } else { [0; 256] },
            tails256: if MODE == 0 { [NONE; 256] } else { [0; 256] },
            fill256: if MODE == 0 { [CHUNK as u32; 256] } else { [0; 256] },
            log_runs: 0,
            heads16: [NONE; 16],
            tails16: [NONE; 16],
            fill16: [R16_CHUNK as u32; 16],
            counts16: [0; 16],
            limit,
            over: false,
        }
    }

    fn finish(self) -> (SavedTransport, bool) {
        (
            SavedTransport {
                in_out: self.in_out,
                heads256: self.heads256,
                log_runs: self.log_runs,
                heads16: self.heads16,
                counts16: self.counts16,
            },
            self.over,
        )
    }

    #[inline(always)]
    fn push<const MODE: usize>(&mut self, c: u8, m: u32, w: u32, thin: &[u32; 256]) {
        match MODE {
            0 => {
                let a = usize::from(c);
                let f = self.fill256[a] as usize;
                let lo = m & (EPOCH - 1);
                let tail = self.tails256[a];
                if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
                    let at = tail as usize * BYTES + 2 * f;
                    let iota: [u16; 16] =
                        std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                    for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                        d.copy_from_slice(&v.to_le_bytes());
                    }
                    self.fill256[a] = (f as u32) + w;
                } else {
                    self.push_outq_slow(a, m, w, thin);
                }
            }
            1 => {
                // `Unqueued` (`QUEUE = false`): `0` queue work in `S1`!
                if m + w > self.limit {
                    self.over = true;
                }
            }
            2 => {
                // `Kakutani-1Stream`: 2-byte `[c, w as u8]` sequential log in `out`!
                if self.over || m + w > self.limit {
                    self.over = true;
                    return;
                }
                let idx = self.log_runs;
                self.log_runs = idx + 1;
                let at = 2 * idx;
                if at + 2 <= self.out.len() {
                    self.out[at] = c;
                    self.out[at + 1] = w as u8;
                } else {
                    self.spill.ovf_log2.push([c, w as u8]);
                }
            }
            _ => {
                // `Radix16-Kakutani`: 16 nibble super-queues, 4-byte `u32` token per thin run:
                // `token = ((c & 15) << 28) | ((w - 1) << 24) | m`
                let g = usize::from(c >> 4);
                let f = self.fill16[g] as usize;
                let tail = self.tails16[g];
                let tok = (u32::from(c & 15) << 28) | ((w - 1) << 24) | m;
                if f < R16_CHUNK && tail < self.in_out {
                    let at = tail as usize * BYTES + 4 * f;
                    self.out[at..at + 4].copy_from_slice(&tok.to_le_bytes());
                    self.fill16[g] = (f + 1) as u32;
                    self.counts16[g] += 1;
                } else {
                    self.push_r16_slow(g, tok, m, w);
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn push_r16_slow(&mut self, g: usize, tok: u32, m: u32, w: u32) {
        if self.over || m + w > self.limit {
            self.over = true;
            return;
        }
        if self.fill16[g] as usize == R16_CHUNK {
            let id = self.chunks;
            self.chunks += 1;
            if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                self.spill.ovf.push([0; BYTES]);
            }
            if self.spill.next.len() <= id as usize {
                self.spill.next.resize(id as usize + 1, NONE);
            }
            self.spill.next[id as usize] = NONE;
            if self.heads16[g] == NONE {
                self.heads16[g] = id;
            } else {
                self.spill.next[self.tails16[g] as usize] = id;
            }
            self.tails16[g] = id;
            self.fill16[g] = 0;
        }
        let f = self.fill16[g] as usize;
        let id = self.tails16[g];
        let chunk = if id < self.in_out {
            let at = id as usize * BYTES;
            &mut self.out[at..at + BYTES]
        } else {
            &mut self.spill.ovf[(id - self.in_out) as usize][..]
        };
        chunk[4 * f..4 * f + 4].copy_from_slice(&tok.to_le_bytes());
        self.fill16[g] = (f + 1) as u32;
        self.counts16[g] += 1;
    }

    #[cold]
    #[inline(never)]
    fn push_outq_slow(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
        if self.over || m + w > self.limit {
            self.over = true;
            return;
        }
        let mut left = w;
        while left > 0 {
            if self.fill256[a] as usize == CHUNK {
                let id = self.chunks;
                self.chunks += 1;
                if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                    self.spill.ovf.push([0; BYTES]);
                }
                if self.spill.next.len() <= id as usize {
                    self.spill.next.resize(id as usize + 1, NONE);
                }
                self.spill.next[id as usize] = NONE;
                if self.heads256[a] == NONE {
                    self.heads256[a] = id;
                } else {
                    self.spill.next[self.tails256[a] as usize] = id;
                }
                self.tails256[a] = id;
                self.fill256[a] = 0;
            }
            let f = self.fill256[a] as usize;
            let lo = m & (EPOCH - 1);
            let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
            let id = self.tails256[a];
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
            self.fill256[a] += take;
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

struct OutQReader<'a> {
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
impl<'a> OutQReader<'a> {
    fn new(out: &'a [u8], spill: &'a Spill, saved: &SavedTransport, a: usize) -> Self {
        Self {
            out,
            spill,
            in_out: saved.in_out,
            a,
            id: saved.heads256[a],
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

#[inline(always)]
fn wire_run_to_bands(
    a: usize,
    mut j: u32,
    w: u32,
    bits: u32,
    wide: bool,
    stream: &mut [u32; 256],
    cursors: &mut [usize; 256],
    bands: &[Band],
    edges: &mut [u32],
    extra: &mut [u32],
) {
    let z0 = stream[a];
    let end = z0 + w;
    stream[a] = end;
    let cur = cursors[a];
    let b0 = bands[cur];
    // Fast path (`94%+` of thin runs): entire thin run lies inside `bands[cur]` and `!wide`!
    if !wide && end <= b0.end {
        let dest = z0.wrapping_add(b0.delta) as usize;
        let val0 = (b0.code << bits) | j;
        let span = w as usize;
        for (i, e) in edges[dest..dest + span].iter_mut().enumerate() {
            *e = val0 + i as u32;
        }
        if end == b0.end {
            cursors[a] = cur + 1;
        }
        return;
    }
    let mut z = z0;
    while z < end {
        let b = bands[cursors[a]];
        let span = (end.min(b.end) - z) as usize;
        let dest = z.wrapping_add(b.delta) as usize;
        let pack = if wide { 0 } else { b.code << bits };
        if wide {
            extra[dest..dest + span].fill(b.code);
        }
        let val0 = pack | j;
        for (i, e) in edges[dest..dest + span].iter_mut().enumerate() {
            *e = val0 + i as u32;
        }
        j += span as u32;
        z += span as u32;
        cursors[a] += usize::from(z == b.end);
    }
}

// ============================================================================
// Exact Verbatim `origin/master` Stages S2 (`Endpoint`), S3 (`Bands/SLP`), S5 (`emit`)
// ============================================================================
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
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }
    #[inline(always)]
    fn rule(&self, w: u32) -> usize {
        self.base + (w - 257) as usize * RULE
    }
    #[inline(always)]
    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            self.a[self.rule(w) + 2]
        }
    }
    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let len = self.len(left) + self.len(right);
        self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);
        self.top += RULE;
        id
    }
    fn power(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join(acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = self.join(base, base);
        }
    }
    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
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
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            left -= w;
            if x == y {
                self.cut(x, 0);
                self.cut(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.cut(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.cut(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.cut(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        *remaining = left;
        done
    }
    fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule(id);
                let rule = &mut self.a[r..r + RULE];
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

fn each_run(records: &[u16], p: u32, limit: u32, mut visit: impl FnMut(u8, u32, u32, bool)) {
    maximal(records, |c, pos, width| {
        split(c, pos, width, p, limit, &mut visit)
    });
}

struct Pass<'a> {
    tr: Transport<'a>,
    strips: &'a mut Vec<Strip>,
    f: First,
}
impl Pass<'_> {
    #[inline(always)]
    fn take<const MODE: usize>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        if width <= limit {
            self.piece::<MODE>(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece::<MODE>(c, pos, width, false, p);
        } else {
            self.take_marked::<MODE>(c, pos, width, p, limit);
        }
    }
    #[cold]
    #[inline(never)]
    fn take_marked<const MODE: usize>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |c, x, w, keep| {
            self.piece::<MODE>(c, x, w, keep, p)
        });
    }
    #[inline(always)]
    fn piece<const MODE: usize>(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
            self.tr.push::<MODE>(c, f.m, w, &f.thin);
            f.thin[a] += w;
            f.m += w;
        } else {
            self.strips.push(Strip {
                from: x,
                to: f.all[a],
                width: w,
                byte: c,
            });
            f.thick[a] += 1;
        }
        f.all[a] += w;
    }
}

#[derive(Default)]
struct CandKernel<const MODE: usize> {
    stack: Vec<u32>,
    spill: Spill,
    strips: Vec<Strip>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    last_s1_us: f64,
    last_s4_us: f64,
}

impl<const MODE: usize> CandKernel<MODE> {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }
    fn wide_links(ns: usize) -> bool {
        Self::nodes_most(ns) > NARROW_NODES
    }
    fn arena_words(ns: usize, m: usize) -> usize {
        let stride = if Self::wide_links(ns) {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
    }

    fn induced_with<L: Links>(&mut self, n: u32, thin: &[u32; 256], m: usize) -> ([usize; 256], usize) {
        let ns = self.strips.len();
        let (image_order, arena) = self.edges.split_at_mut(ns);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0;
        for s in self.strips.iter() {
            e.node(s.width, u32::from(s.byte) + 1);
            widest = widest.max(s.width);
        }
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0;
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
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }
        x = 0;
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
        while !if spirals {
            e.eliminate::<true>(&mut remaining)
        } else {
            e.eliminate::<false>(&mut remaining)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(&mut self.edges, ns + top + RULE);
            e = Elim {
                a: &mut self.edges[ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        let mut id = L::next(e.a, 0, 1);
        x = 0;
        while id != 0 {
            let next = L::next(e.a, id, 1);
            let w = e.width(id);
            e.set_bottom(id, x);
            x += w;
            id = next;
        }
        assert_eq!(x as usize, m);
        let mut cursors = [0; 256];
        let mut words = 0usize;
        id = L::next(e.a, 0, 0);
        let mut in_node = 0;
        let mut k = 0;
        for a in 0..256 {
            cursors[a] = self.bands.len();
            let end = k + thin[a];
            while k < end {
                let node_width = e.width(id);
                let node_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                assert!(width > 0);
                let dst = e.bottom(id) + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    words += 1;
                    e.append(node_word, &mut self.stack, &mut self.spelling);
                    self.spelling.push(a as u8);
                    self.offsets.push(self.spelling.len() as u32);
                    254 + self.offsets.len() as u32
                };
                self.bands.push(Band {
                    end: k + width,
                    delta: dst.wrapping_sub(k),
                    code,
                });
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        (cursors, words)
    }

    fn try_run<const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<()> {
        let t_s1 = if TIMED { Some(Instant::now()) } else { None };
        out.resize(n + INLINE, 0);
        self.strips.clear();
        let (f, saved, over) = {
            let mut pass = Pass {
                tr: Transport::new::<MODE>(&mut out[..], &mut self.spill, cap.m),
                strips: &mut self.strips,
                f: First::default(),
            };
            maximal(records, |c, pos, width| {
                pass.take::<MODE>(c, pos, width, p, LIMIT);
            });
            let f = pass.f;
            let (saved, over) = pass.tr.finish();
            (f, saved, over)
        };
        if let Some(t0) = t_s1 {
            self.last_s1_us = t0.elapsed().as_nanos() as f64 / 1e3;
        }
        if over || f.m > cap.m || self.strips.len() > cap.strips {
            self.strips.clear();
            self.strips.shrink_to(cap.strips);
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
        let mut write = starts(thick);
        let ns = self.strips.len();
        let need = Self::arena_words(ns, m as usize);
        if self.edges.len() < need {
            self.edges.clear();
            self.edges.reserve_exact(need);
            self.edges.resize(need, 0);
        }
        let image_order = &mut self.edges[..ns];
        for (i, s) in self.strips.iter_mut().enumerate() {
            let a = usize::from(s.byte);
            s.to += first[a];
            image_order[write[a] as usize] = i as u32;
            write[a] += 1;
        }
        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        let (mut cursors, words) = if Self::wide_links(ns) {
            self.induced_with::<Wide>(n as u32, &thin, m as usize)
        } else {
            self.induced_with::<Narrow>(n as u32, &thin, m as usize)
        };

        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + words as u32) > (u32::MAX >> bits);
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }

        match MODE {
            0 => {
                let mut k = 0;
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = OutQReader::new(&out[..], &self.spill, &saved, a);
                    for b in &self.bands[cursors[a]..end] {
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
            1 => {
                // `B1: Unqueued` (`QUEUE = false` from `src/open.rs:486-514`):
                let mut stream = starts(thin);
                let mut j = 0;
                let bands = &self.bands[..];
                let edges = &mut self.edges[..m as usize];
                let extra = &mut self.extra[..];
                each_run(records, p, LIMIT, |c, _, w, keep| {
                    if !keep {
                        return;
                    }
                    wire_run_to_bands(
                        usize::from(c),
                        j,
                        w,
                        bits,
                        wide,
                        &mut stream,
                        &mut cursors,
                        bands,
                        edges,
                        extra,
                    );
                    j += w;
                });
            }
            2 => {
                // `B2: Kakutani-1Stream` (`2 B/run` sequential log in `out`):
                let mut stream = starts(thin);
                let mut j = 0u32;
                let total_runs = saved.log_runs;
                let in_out_runs = (out.len() / 2).min(total_runs);
                let bands = &self.bands[..];
                let edges = &mut self.edges[..m as usize];
                let extra = &mut self.extra[..];
                for pair in out[..2 * in_out_runs].chunks_exact(2) {
                    let a = usize::from(pair[0]);
                    let w = u32::from(pair[1]);
                    wire_run_to_bands(a, j, w, bits, wide, &mut stream, &mut cursors, bands, edges, extra);
                    j += w;
                }
                for &[c, wb] in &self.spill.ovf_log2 {
                    let w = u32::from(wb);
                    wire_run_to_bands(usize::from(c), j, w, bits, wide, &mut stream, &mut cursors, bands, edges, extra);
                    j += w;
                }
            }
            _ => {
                // `B3: Radix16-Kakutani` (`16` nibble passes, only `16` active symbols per pass!):
                let mut stream = starts(thin);
                let bands = &self.bands[..];
                let edges = &mut self.edges[..m as usize];
                let extra = &mut self.extra[..];
                for g in 0..16usize {
                    let mut rem = saved.counts16[g] as usize;
                    let mut id = saved.heads16[g];
                    let base_a = g << 4;
                    while rem > 0 {
                        let take = rem.min(R16_CHUNK);
                        let chunk = if id < saved.in_out {
                            let at = id as usize * BYTES;
                            &out[at..at + BYTES]
                        } else {
                            &self.spill.ovf[(id - saved.in_out) as usize][..]
                        };
                        for tb in chunk[..4 * take].chunks_exact(4) {
                            let tok = u32::from_le_bytes([tb[0], tb[1], tb[2], tb[3]]);
                            let a = base_a | ((tok >> 28) as usize);
                            let w = ((tok >> 24) & 15) + 1;
                            let j = tok & 0x00FF_FFFF;
                            wire_run_to_bands(a, j, w, bits, wide, &mut stream, &mut cursors, bands, edges, extra);
                        }
                        rem -= take;
                        if rem > 0 {
                            id = self.spill.next[id as usize];
                        }
                    }
                }
            }
        }
        if let Some(t0) = t_s4 {
            self.last_s4_us = t0.elapsed().as_nanos() as f64 / 1e3;
        }

        // Exact `origin/master` `inline_words` + `emit`:
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

// ============================================================================
// Bz2 Block Crafting, Gate G1 Verification, and Confrontation
// ============================================================================
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
    let mut c0 = CandOpenCells::<0, true>::default();
    let mut c1 = CandOpenCells::<1, true>::default();
    let mut c2 = CandOpenCells::<2, true>::default();
    let mut c3 = CandOpenCells::<3, true>::default();
    let mut o_ref = Vec::new();
    let mut o0 = Vec::new();
    let mut o1 = Vec::new();
    let mut o2 = Vec::new();
    let mut o3 = Vec::new();

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
                assert!(c0.invert_runs(&runs_obj, n_dec, p_dec, &mut o0, kernels));
                assert!(c1.invert_runs(&runs_obj, n_dec, p_dec, &mut o1, kernels));
                assert!(c2.invert_runs(&runs_obj, n_dec, p_dec, &mut o2, kernels));
                assert!(c3.invert_runs(&runs_obj, n_dec, p_dec, &mut o3, kernels));
                assert_eq!(o0, o_ref);
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
                assert!(c0.invert_runs(&runs_obj, n_dec, p_dec, &mut o0, kernels));
                assert!(c1.invert_runs(&runs_obj, n_dec, p_dec, &mut o1, kernels));
                assert!(c2.invert_runs(&runs_obj, n_dec, p_dec, &mut o2, kernels));
                assert!(c3.invert_runs(&runs_obj, n_dec, p_dec, &mut o3, kernels));
                assert_eq!(o0, o_ref);
                assert_eq!(o1, o_ref);
                assert_eq!(o2, o_ref);
                assert_eq!(o3, o_ref);
                cases += 1;
            }
        }
    }
    // Multi-epoch (`m > 135,000`) + chunk-wrap test:
    let mut raw = Vec::with_capacity(220_000);
    let mut rng = 0x1234_5678_9ABC_DEF0u64;
    for _ in 0..210_000 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        raw.push(b'A' + ((rng >> 58) as u8 % 24));
    }
    let (ll_ep, p_ep) = bwt_of_rle1_safe(&raw);
    let bz2_ep = craft_bz2_block(&ll_ep, p_ep, kernels);
    let (runs_ep, n_ep, _) = extract_runs(&bz2_ep, kernels);
    assert!(m_ref.invert_runs(&runs_ep, n_ep, p_ep, &mut o_ref, kernels));
    assert!(c1.invert_runs(&runs_ep, n_ep, p_ep, &mut o1, kernels));
    assert!(c2.invert_runs(&runs_ep, n_ep, p_ep, &mut o2, kernels));
    assert!(c3.invert_runs(&runs_ep, n_ep, p_ep, &mut o3, kernels));
    assert_eq!(o1, o_ref);
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
        "Gate G1 + Spill::ovf + Multi-Epoch Verification: PASS ({} cases byte-exact vs OpenCells)\n",
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

    println!("=== CYCLE 14A: STAGE S1 & S4 ISOLATED TIMINGS (`S1 / S4 / S1+S4` IN MICROSECONDS) ===");
    println!(
        "{:<13} | {:>20} | {:>20} | {:>20} | {:>20}",
        "Block",
        "M0:OutQ16(S1/S4/Sum)",
        "B1:Unq(S1/S4/Sum)",
        "B2:Kak1(S1/S4/Sum)",
        "B3:Rad16(S1/S4/Sum)"
    );
    println!("{}", "-".repeat(106));

    let mut m_a1 = OpenCells::default();
    let mut m_a2 = OpenCells::default();
    let mut c_m0 = CandOpenCells::<0>::default();
    let mut c_b1 = CandOpenCells::<1>::default();
    let mut c_b2 = CandOpenCells::<2>::default();
    let mut c_b3 = CandOpenCells::<3>::default();

    let mut o_a1 = vec![0u8; 900_016];
    let mut o_a2 = vec![0u8; 900_016];
    let mut o_m0 = vec![0u8; 900_016];
    let mut o_b1 = vec![0u8; 900_016];
    let mut o_b2 = vec![0u8; 900_016];
    let mut o_b3 = vec![0u8; 900_016];

    let mut stage_rows = Vec::new();
    for (name, runs_obj, n, p) in &suite {
        let cap = CandOpenCells::<0>::cap(runs_obj.records().len(), *n, *p).unwrap();
        let mut s0 = Vec::with_capacity(9);
        let mut s1 = Vec::with_capacity(9);
        let mut s2 = Vec::with_capacity(9);
        let mut s3 = Vec::with_capacity(9);
        for r in 0..11 {
            c_m0.open.try_run::<true>(runs_obj.records(), *n, *p, cap, &mut o_m0).unwrap();
            c_b1.open.try_run::<true>(runs_obj.records(), *n, *p, cap, &mut o_b1).unwrap();
            c_b2.open.try_run::<true>(runs_obj.records(), *n, *p, cap, &mut o_b2).unwrap();
            c_b3.open.try_run::<true>(runs_obj.records(), *n, *p, cap, &mut o_b3).unwrap();
            if r >= 2 {
                s0.push((c_m0.open.last_s1_us, c_m0.open.last_s4_us));
                s1.push((c_b1.open.last_s1_us, c_b1.open.last_s4_us));
                s2.push((c_b2.open.last_s1_us, c_b2.open.last_s4_us));
                s3.push((c_b3.open.last_s1_us, c_b3.open.last_s4_us));
            }
        }
        let med = |v: &[(f64, f64)]| -> (f64, f64) {
            let mut a: Vec<f64> = v.iter().map(|x| x.0).collect();
            let mut b: Vec<f64> = v.iter().map(|x| x.1).collect();
            a.sort_by(|x, y| x.partial_cmp(y).unwrap());
            b.sort_by(|x, y| x.partial_cmp(y).unwrap());
            (a[a.len() / 2], b[b.len() / 2])
        };
        let (m0_1, m0_4) = med(&s0);
        let (b1_1, b1_4) = med(&s1);
        let (b2_1, b2_4) = med(&s2);
        let (b3_1, b3_4) = med(&s3);
        println!(
            "{:<13} | {:>5.1}/{:>5.1}/{:>6.1} | {:>5.1}/{:>5.1}/{:>6.1} | {:>5.1}/{:>5.1}/{:>6.1} | {:>5.1}/{:>5.1}/{:>6.1}",
            name,
            m0_1, m0_4, m0_1 + m0_4,
            b1_1, b1_4, b1_1 + b1_4,
            b2_1, b2_4, b2_1 + b2_4,
            b3_1, b3_4, b3_1 + b3_4
        );
        stage_rows.push((m0_1, m0_4, b1_1, b1_4, b2_1, b2_4, b3_1, b3_4));
    }

    println!("\n=== CYCLE 14B: NATIVE SAFE-RUST END-TO-END A/B + A/A CONFRONTATION (`19` INTERLEAVED ROUNDS) ===");
    println!(
        "{:<13} | {:>9} | {:>9} | {:>7} | {:>9} | {:>7} | {:>9} | {:>7} | {:>9} | {:>7}",
        "Block",
        "A1:Master",
        "A2:Ctrl",
        "A2/A1",
        "B1:Unque",
        "B1/A1",
        "B2:Kak1S",
        "B2/A1",
        "B3:Rad16",
        "B3/A1"
    );
    println!("{}", "-".repeat(106));

    let mut json_rows = Vec::new();
    for (idx, (name, runs_obj, n, p)) in suite.iter().enumerate() {
        assert!(m_a1.invert_runs(runs_obj, *n, *p, &mut o_a1, kernels));
        assert!(m_a2.invert_runs(runs_obj, *n, *p, &mut o_a2, kernels));
        assert!(c_b1.invert_runs(runs_obj, *n, *p, &mut o_b1, kernels));
        assert!(c_b2.invert_runs(runs_obj, *n, *p, &mut o_b2, kernels));
        assert!(c_b3.invert_runs(runs_obj, *n, *p, &mut o_b3, kernels));
        assert_eq!(o_a1, o_a2);
        assert_eq!(o_b1, o_a1);
        assert_eq!(o_b2, o_a1);
        assert_eq!(o_b3, o_a1);

        let rounds = 19usize;
        let mut t_a1 = Vec::with_capacity(rounds);
        let mut t_a2 = Vec::with_capacity(rounds);
        let mut t_b1 = Vec::with_capacity(rounds);
        let mut t_b2 = Vec::with_capacity(rounds);
        let mut t_b3 = Vec::with_capacity(rounds);

        for r in 0..(rounds + 2) {
            for slot in 0..5 {
                let arm = (r + slot) % 5;
                let t0 = Instant::now();
                match arm {
                    0 => assert!(m_a1.invert_runs(runs_obj, *n, *p, &mut o_a1, kernels)),
                    1 => assert!(m_a2.invert_runs(runs_obj, *n, *p, &mut o_a2, kernels)),
                    2 => assert!(c_b1.invert_runs(runs_obj, *n, *p, &mut o_b1, kernels)),
                    3 => assert!(c_b2.invert_runs(runs_obj, *n, *p, &mut o_b2, kernels)),
                    _ => assert!(c_b3.invert_runs(runs_obj, *n, *p, &mut o_b3, kernels)),
                }
                let dt = t0.elapsed().as_nanos() as f64 / 1e3;
                if r >= 2 {
                    match arm {
                        0 => t_a1.push(dt),
                        1 => t_a2.push(dt),
                        2 => t_b1.push(dt),
                        3 => t_b2.push(dt),
                        _ => t_b3.push(dt),
                    }
                }
            }
        }
        for v in [&mut t_a1, &mut t_a2, &mut t_b1, &mut t_b2, &mut t_b3] {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }
        let ma1 = t_a1[rounds / 2];
        let ma2 = t_a2[rounds / 2];
        let mb1 = t_b1[rounds / 2];
        let mb2 = t_b2[rounds / 2];
        let mb3 = t_b3[rounds / 2];

        println!(
            "{:<13} | {:>7.1}us | {:>7.1}us | {:>6.3}x | {:>7.1}us | {:>6.3}x | {:>7.1}us | {:>6.3}x | {:>7.1}us | {:>6.3}x",
            name,
            ma1, ma2, ma2 / ma1,
            mb1, mb1 / ma1,
            mb2, mb2 / ma1,
            mb3, mb3 / ma1
        );

        let sr = stage_rows[idx];
        json_rows.push(format!(
            "    {{\n      \"block\": \"{}\",\n      \"stage_us\": {{\"m0_s1\": {:.2}, \"m0_s4\": {:.2}, \"b1_s1\": {:.2}, \"b1_s4\": {:.2}, \"b2_s1\": {:.2}, \"b2_s4\": {:.2}, \"b3_s1\": {:.2}, \"b3_s4\": {:.2}}},\n      \"e2e_us\": {{\"a1_master\": {:.2}, \"a2_ctrl\": {:.2}, \"b1_unqueued\": {:.2}, \"b2_kakutani_1s\": {:.2}, \"b3_radix16_kakutani\": {:.2}}},\n      \"ratios\": {{\"a2_a1\": {:.4}, \"b1_a1\": {:.4}, \"b2_a1\": {:.4}, \"b3_a1\": {:.4}}}\n    }}",
            name,
            sr.0, sr.1, sr.2, sr.3, sr.4, sr.5, sr.6, sr.7,
            ma1, ma2, mb1, mb2, mb3,
            ma2 / ma1, mb1 / ma1, mb2 / ma1, mb3 / ma1
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
