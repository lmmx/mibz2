//! Rauzy output census: pricing the Rauzy machinery with what it produces (LIMIT sweep &
//! adaptive rules, word representation & dictionary dedup, cheaper induction steps).
//!
//! Smooth-zone characterisation for claim `rauzy-output`
//! (`docs/journal/2026-11-24-algorithm-level/rauzy-census/results-2/README.md`).
//! Zero changes to `src/`, zero `unsafe`.
//!
//! Usage:
//!   rauzy_output_census limit FILE.bz2 [--repeats N] [--forced] [--x4]
//!   rauzy_output_census words FILE.bz2 [--repeats N] [--forced] [--x4]
//!   rauzy_output_census steps FILE.bz2 [--repeats N] [--forced] [--x4]
//!   rauzy_output_census all   FILE.bz2 [--repeats N] [--forced] [--x4]

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::hint::black_box;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Production Open constants (mirroring src/open.rs, src/open/endpoint.rs,
// src/open/outq.rs, src/cells_lazy.rs)
// ---------------------------------------------------------------------------

const PROD_LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const RULE: usize = 4;
const NARROW_NODES: usize = 1 << 16;
const ID_BITS: u32 = 22;
const ID_MASK: u32 = (1 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

const LAZY_DENSITY: (usize, usize) = (3, 20);
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

const CHUNK: usize = 512;
const CHUNK_BYTES: usize = 2 * CHUNK;
const NONE_CHUNK: u32 = u32::MAX;
const EPOCH: u32 = 1 << 16;

// i9-10900K baseline anchors on wikidata x4 (character-stage1/report.txt & plan.md)
const I9_WALL_1W_MS: f64 = 1306.0;
const I9_STAGE1_1W_MS: f64 = 1283.1;
const I9_A6A_MS: f64 = 200.3;
const I9_A6B_MS: f64 = 216.0;
const I9_A6C_MS: f64 = 161.4;
const I9_A7_MS: f64 = 490.3;
const I9_CE_MS: f64 = I9_A6A_MS + I9_A6B_MS + I9_A6C_MS + I9_A7_MS; // 1068.0 ms
const GATE_PCT: f64 = 3.0;
const GATE_MS: f64 = I9_WALL_1W_MS * (GATE_PCT / 100.0); // 39.18 ms

#[derive(Clone, Copy, Debug, PartialEq)]
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
    records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n
}

fn open_cap(records: usize, n: usize, p: u32, forced: bool) -> Option<Cap> {
    if n == 0 || n > 900_000 || p as usize >= n {
        None
    } else if forced || lazy_counts_admit(records, n) {
        Some(Cap::NONE)
    } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
        Some(Cap::dense(n))
    } else {
        None
    }
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::with_capacity(l.len() / 4 + 16);
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

#[inline(always)]
fn literal(byte: u8) -> u32 {
    (1 << ID_BITS) | (u32::from(byte) + 1)
}

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

impl Strip {
    #[inline(always)]
    fn unpack(w: &[u32]) -> Self {
        Strip {
            from: w[0],
            to: w[1],
            width: w[2] >> 8,
            byte: (w[2] & 0xFF) as u8,
        }
    }
}

#[derive(Clone, Copy)]
struct Band(u64);

impl Band {
    #[inline(always)]
    fn new(end: u32, dst: u32, code: u32) -> Self {
        Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
    }
    #[inline(always)]
    fn end(self) -> u32 {
        (self.0 & 0xF_FFFF) as u32
    }
    #[inline(always)]
    fn dst(self) -> u32 {
        ((self.0 >> 20) & 0xF_FFFF) as u32
    }
    #[inline(always)]
    fn code(self) -> u32 {
        (self.0 >> 40) as u32
    }
}

#[inline(always)]
fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
    let mut slot = [0; INLINE];
    slot[..4].copy_from_slice(&start.to_le_bytes());
    slot[4..8].copy_from_slice(&end.to_le_bytes());
    slot
}

#[inline(always)]
fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
    let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
    let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
    &spelling[start..end]
}

#[inline(always)]
fn long_bounds(slot: &[u8; INLINE]) -> (usize, usize) {
    let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
    let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
    (start, end)
}

fn begin_inline(words: usize, inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
    if inline16.len() < 256 {
        inline16.clear();
        inline16.extend((0..=255u8).map(|c| {
            let mut v = [0; INLINE];
            v[0] = c;
            v
        }));
        lens.clear();
        lens.resize(256, 1);
    }
    inline16.truncate(256);
    lens.truncate(256);
    inline16.reserve_exact(words);
    lens.reserve_exact(words);
}

fn begin_inline_with_back(
    words: usize,
    inline16: &mut Vec<[u8; INLINE]>,
    inline_back: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
) {
    begin_inline(words, inline16, lens);
    if inline_back.len() < 256 {
        inline_back.clear();
        inline_back.extend((0..=255u8).map(|c| {
            let mut v = [0; INLINE];
            v[INLINE - 1] = c;
            v
        }));
    }
    inline_back.truncate(256);
    inline_back.reserve_exact(words);
}

#[inline(always)]
fn shift_back(entry: [u8; INLINE], len: u8) -> [u8; INLINE] {
    if len == LONG {
        entry
    } else {
        (u128::from_le_bytes(entry) << (8 * (INLINE - usize::from(len)))).to_le_bytes()
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

#[inline(always)]
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

// ---------------------------------------------------------------------------
// OutQ16 queues (mirroring src/open/outq.rs)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Spill {
    next: Vec<u32>,
    ovf: Vec<[u8; CHUNK_BYTES]>,
    cuts: Vec<[u32; 256]>,
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
    slow_pushes: usize,
}

#[derive(Clone, Copy)]
struct Heads {
    heads: [u32; 256],
    in_out: u32,
}

impl<'a> Queues<'a> {
    fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
        spill.ovf.clear();
        spill.cuts.clear();
        Queues {
            in_out: (out.len() / CHUNK_BYTES) as u32,
            out,
            spill,
            chunks: 0,
            heads: [NONE_CHUNK; 256],
            tails: [NONE_CHUNK; 256],
            fill: [CHUNK as u32; 256],
            limit,
            over: false,
            slow_pushes: 0,
        }
    }

    fn heads(&self) -> Heads {
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }

    #[inline(always)]
    fn push<const FAST64: bool>(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * CHUNK_BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else if FAST64 && w <= 64 && f + 64 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * CHUNK_BYTES + 2 * f;
            for (i, d) in self.out[at..at + 2 * w as usize]
                .chunks_exact_mut(2)
                .enumerate()
            {
                d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else {
            self.slow_pushes += 1;
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
                self.take_chunk(a);
            }
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
            let chunk = self.chunk_mut(self.tails[a]);
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

    fn take_chunk(&mut self, a: usize) {
        let id = self.chunks;
        self.chunks += 1;
        if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
            self.spill.ovf.push([0; CHUNK_BYTES]);
        }
        if self.spill.next.len() <= id as usize {
            self.spill.next.resize(id as usize + 1, NONE_CHUNK);
        }
        self.spill.next[id as usize] = NONE_CHUNK;
        if self.heads[a] == NONE_CHUNK {
            self.heads[a] = id;
        } else {
            self.spill.next[self.tails[a] as usize] = id;
        }
        self.tails[a] = id;
        self.fill[a] = 0;
    }

    fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
        if id < self.in_out {
            let at = id as usize * CHUNK_BYTES;
            &mut self.out[at..at + CHUNK_BYTES]
        } else {
            &mut self.spill.ovf[(id - self.in_out) as usize][..]
        }
    }
}

struct QueueReader<'a> {
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

impl<'a> QueueReader<'a> {
    fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
        QueueReader {
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
    fn wire_xor(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
        mark: u32,
        predecessor: &mut u32,
    ) {
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = self.epoch << 16;
            let at = self.id as usize * CHUNK_BYTES + 2 * self.at;
            let bytes = &self.out[at..at + 2 * span];
            let mut before = *predecessor;
            for (i, s) in bytes.chunks_exact(2).enumerate() {
                let u = dst_start + i;
                let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                if v == mark {
                    before = u as u32;
                }
                edges[u] ^= pack | v;
                edges[v as usize] ^= u as u32;
            }
            *predecessor = before;
            self.at += span;
            self.read += span as u32;
        } else {
            self.wire_xor_slow(edges, dst_start, span, pack, mark, predecessor);
        }
    }

    #[cold]
    #[inline(never)]
    fn wire_xor_slow(
        &mut self,
        edges: &mut [u32],
        mut dst_start: usize,
        mut left: usize,
        pack: u32,
        mark: u32,
        predecessor: &mut u32,
    ) {
        while left > 0 {
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
            let take = left
                .min(CHUNK - self.at)
                .min((self.cut - self.read) as usize);
            let chunk: &[u8] = if self.id < self.in_out {
                let at = self.id as usize * CHUNK_BYTES;
                &self.out[at..at + CHUNK_BYTES]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][..]
            };
            let hi = self.epoch << 16;
            for (i, s) in chunk[2 * self.at..2 * (self.at + take)]
                .chunks_exact(2)
                .enumerate()
            {
                let u = dst_start + i;
                let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                if v == mark {
                    *predecessor = u as u32;
                }
                edges[u] ^= pack | v;
                edges[v as usize] ^= u as u32;
            }
            self.at += take;
            self.read += take as u32;
            dst_start += take;
            left -= take;
        }
    }

    #[inline(always)]
    fn wire(&mut self, dst: &mut [u32], pack: u32) {
        let span = dst.len();
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = pack | (self.epoch << 16);
            let at = self.id as usize * CHUNK_BYTES + 2 * self.at;
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
            let chunk: &[u8] = if self.id < self.in_out {
                let at = self.id as usize * CHUNK_BYTES;
                &self.out[at..at + CHUNK_BYTES]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][..]
            };
            let hi = pack | (self.epoch << 16);
            let (now, rest) = dst.split_at_mut(take);
            for (e, s) in now
                .iter_mut()
                .zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
            {
                *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
            }
            self.at += take;
            self.read += take as u32;
            dst = rest;
        }
    }
}

// ---------------------------------------------------------------------------
// First pass & LIMIT policies
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    runs: usize,
}

impl Default for First {
    fn default() -> Self {
        First {
            thick_rows: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            runs: 0,
        }
    }
}

struct Pass<'a> {
    queues: Queues<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    f: First,
}

impl Pass<'_> {
    #[inline(always)]
    fn take<const FAST64: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        self.f.runs += 1;
        if width <= limit {
            self.piece::<FAST64>(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece::<FAST64>(c, pos, width, false, p);
        } else {
            self.take_marked::<FAST64>(c, pos, width, p, limit);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked<const FAST64: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |c, x, w, keep| {
            self.piece::<FAST64>(c, x, w, keep, p)
        });
    }

    #[inline(always)]
    fn piece<const FAST64: bool>(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
            self.queues.push::<FAST64>(a, f.m, w, &f.thin);
            f.thin[a] += w;
            f.m += w;
        } else {
            if self.ns < self.cap_strips {
                let at = 3 * self.ns;
                if self.edges.len() < at + 3 {
                    let need = (at + 3)
                        .max(2 * self.edges.len())
                        .max(1024)
                        .min(self.cap_strips.saturating_mul(3));
                    self.edges.reserve_exact(need - self.edges.len());
                    self.edges.resize(need, 0);
                }
                self.edges[at..at + 3].copy_from_slice(&[
                    x,
                    f.thin[a] + f.thick_rows[a],
                    (w << 8) | u32::from(c),
                ]);
            }
            self.ns += 1;
            f.thick[a] += 1;
            f.thick_rows[a] += w;
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum LimitPolicy {
    Fixed(u32),
    FixedFast64(u32),
    /// Zero-extra-pass rule from (records.len(), n): denser blocks use slightly higher LIMIT to
    /// prevent strip/node explosion; sparser blocks use lower LIMIT to crush m.
    PerBlockDensity,
    /// Lightweight histogram pre-pass over records minimizing (strip_cost * strips + row_cost * m)
    /// while respecting Narrow node budget (3*ns + 3 <= 65536). Pre-pass time included in A6a.
    PerBlockHist {
        strip_to_row_ratio: u32,
    },
    /// Per-byte adaptive limit[256] chosen from a lightweight pre-pass of per-byte thick/thin
    /// counts: bytes with heavier thick presence get lower limit (fewer new image gaps).
    PerByteAdaptive,
}

fn choose_block_limit_density(records_len: usize, n: usize) -> u32 {
    // r/n in [0, 0.28). Low r/n (<0.07) -> 8, mid (0.07..0.13) -> 10, high (>=0.13) -> 12
    let permille = (records_len as u64 * 1000) / (n.max(1) as u64);
    if permille < 70 {
        8
    } else if permille < 130 {
        10
    } else {
        12
    }
}

fn choose_block_limit_hist(records: &[u16], ratio: u32) -> u32 {
    const MAX_W: usize = 65;
    let mut hist = [0u32; MAX_W + 1];
    maximal(records, |_c, _pos, width| {
        let w = (width as usize).min(MAX_W);
        hist[w] += 1;
    });
    // Candidate limits: 4, 6, 8, 10, 12, 16, 24, 32
    let candidates = [4u32, 6, 8, 10, 12, 16, 24, 32];
    let mut best_l = 16u32;
    let mut best_cost = u64::MAX;
    for &l in &candidates {
        let mut thin_rows = 0u64;
        let mut thick_strips = 0u64;
        for (w, &cnt) in hist.iter().enumerate().skip(1) {
            if (w as u32) <= l {
                thin_rows += w as u64 * u64::from(cnt);
            } else {
                thick_strips += u64::from(cnt);
            }
        }
        // Penalize Wide node spillover (3*ns + 3 > 65536 -> ns > 21844)
        let wide_pen = if thick_strips > 21_844 {
            thick_strips * 10
        } else {
            0
        };
        let cost = thin_rows + thick_strips * u64::from(ratio) + wide_pen;
        if cost < best_cost {
            best_cost = cost;
            best_l = l;
        }
    }
    best_l
}

fn choose_per_byte_limits(records: &[u16]) -> [u32; 256] {
    // Cheap 1-pass per-byte summary: total runs and runs of width > 12 vs total rows per byte.
    let mut runs_c = [0u32; 256];
    let mut big_c = [0u32; 256];
    maximal(records, |c, _pos, width| {
        let a = usize::from(c);
        runs_c[a] += 1;
        if width > 12 {
            big_c[a] += 1;
        }
    });
    let mut lim = [12u32; 256];
    for a in 0..256 {
        let r = runs_c[a];
        if r == 0 {
            continue;
        }
        // Bytes where >= 15% of runs are already thick (>12) get lower limit (8): new thick
        // strips merge cleanly with existing thick regions on side 1 without splitting gaps.
        // Bytes with almost all thin runs (< 3% thick) get higher limit (16) to avoid creating
        // isolated strips that split gaps.
        if big_c[a] * 20 >= r * 3 {
            lim[a] = 8;
        } else if big_c[a] * 100 < r * 3 {
            lim[a] = 16;
        } else {
            lim[a] = 12;
        }
    }
    lim
}

// ---------------------------------------------------------------------------
// Endpoint Links & Elim (production mirror + Idea 2 & Idea 3 variants)
// ---------------------------------------------------------------------------

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

#[cold]
#[inline(never)]
fn grow(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
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

    fn assign_bottom(&mut self, rows: u32) {
        let mut first = L::next(self.a, 0, 1);
        let mut last = L::prev(self.a, 0, 1);
        let (mut lo, mut hi) = (0, rows);
        while first != last {
            let next = L::next(self.a, first, 1);
            let prev = L::prev(self.a, last, 1);
            let width = self.width(first);
            hi -= self.width(last);
            self.set_bottom(first, lo);
            self.set_bottom(last, hi);
            lo += width;
            if next == last {
                assert_eq!(lo, hi);
                return;
            }
            first = next;
            last = prev;
        }
        if first != 0 {
            self.set_bottom(first, lo);
            lo += self.width(first);
        }
        assert_eq!(lo, hi);
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
    fn pop_tail(&mut self, id: u32, side: usize) {
        let prev = L::prev(self.a, id, side);
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
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
            let r = self.rule(w);
            let tag = self.a[r + 3];
            if tag >> 28 == 0 {
                tag >> 24
            } else {
                self.a[r + 2]
            }
        }
    }

    #[inline(always)]
    fn short_payload(&self, id: u32) -> u128 {
        if id <= 256 {
            return u128::from(id - 1);
        }
        let r = self.rule(id);
        u128::from(self.a[r])
            | (u128::from(self.a[r + 1]) << 32)
            | (u128::from(self.a[r + 2]) << 64)
            | (u128::from(self.a[r + 3] & 0x00ff_ffff) << 96)
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
        let (left_len, right_len) = (left >> ID_BITS, right >> ID_BITS);
        let (left, right) = (left & ID_MASK, right & ID_MASK);
        let len = if left_len < SATURATED && right_len < SATURATED {
            left_len + right_len
        } else {
            self.len(left) + self.len(right)
        };
        let rule = if len < INLINE as u32 {
            let bytes = self.short_payload(left)
                | (self.short_payload(right) << (8 * left_len))
                | (u128::from(len) << 120);
            [
                bytes as u32,
                (bytes >> 32) as u32,
                (bytes >> 64) as u32,
                (bytes >> 96) as u32,
            ]
        } else {
            [left, right, len, u32::MAX]
        };
        self.a[self.top..self.top + RULE].copy_from_slice(&rule);
        self.top += RULE;
        (len.min(SATURATED) << ID_BITS) | id
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

    /// Production elimination loop verbatim.
    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
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
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    taken += q as usize;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    taken += q as usize;
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            taken += 1;
            if x == y {
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.pop_tail(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.pop_tail(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        (*remaining, *steps) = (left, taken);
        done
    }

    /// Ablation: exact same linked-list topology & steps, skipping join() completely.
    fn eliminate_no_join_ablat(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
        loop {
            if left == 0 {
                break;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            let w = aw.min(bw);
            left -= w;
            taken += 1;
            if x == y {
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            if aw <= bw {
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                if aw == bw {
                    self.cut(y, 0);
                    self.pop_tail(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.pop_tail(y, 1);
                self.insert(x, y, 1);
                self.set_width(x, aw - w);
            }
        }
        (*remaining, *steps) = (left, taken);
        true
    }

    /// Register-cached (x, y, aw, bw, xw, yw) — avoids sentinel 0 reloads and winner reloads.
    fn eliminate_reg_xy(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
        let mut x = L::prev(self.a, 0, 0);
        let mut y = L::prev(self.a, 0, 1);
        let mut aw = self.width(x);
        let mut bw = self.width(y);
        let mut xw = self.word(x);
        let mut yw = self.word(y);
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let w = aw.min(bw);
            left -= w;
            taken += 1;
            if x == y {
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                if left != 0 {
                    x = L::prev(self.a, 0, 0);
                    y = L::prev(self.a, 0, 1);
                    aw = self.width(x);
                    bw = self.width(y);
                    xw = self.word(x);
                    yw = self.word(y);
                }
                continue;
            }
            let word = self.join(xw, yw);
            if aw < bw {
                let xp = L::prev(self.a, x, 0);
                bw -= w;
                self.set_width(y, bw);
                self.set_word(x, word);
                if xp != y {
                    L::set_next(self.a, xp, 0, 0);
                    L::set_prev(self.a, 0, 0, xp);
                    self.insert(y, x, 0);
                    x = xp;
                    aw = self.width(x);
                    xw = self.word(x);
                } else {
                    // Spiral step: x immediately follows y on side 0, so x stays tail of side 0
                    xw = word;
                }
            } else if aw > bw {
                let yp = L::prev(self.a, y, 1);
                aw -= w;
                self.set_width(x, aw);
                self.set_word(y, word);
                if yp != x {
                    L::set_next(self.a, yp, 1, 0);
                    L::set_prev(self.a, 0, 1, yp);
                    self.insert(x, y, 1);
                    y = yp;
                    bw = self.width(y);
                    yw = self.word(y);
                } else {
                    // Spiral step: y immediately follows x on side 1, so y stays tail of side 1
                    yw = word;
                }
            } else {
                // aw == bw (37% of steps!)
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                self.cut(y, 0);
                self.pop_tail(y, 1);
                if left != 0 {
                    x = L::prev(self.a, 0, 0);
                    y = L::prev(self.a, 0, 1);
                    aw = self.width(x);
                    bw = self.width(y);
                    xw = self.word(x);
                    yw = self.word(y);
                }
            }
        };
        (*remaining, *steps) = (left, taken);
        done
    }

    /// Register-cached + hoisted next-node load BEFORE join() so next-node L2 load overlaps join().
    fn eliminate_hoist_next(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
        let mut x = L::prev(self.a, 0, 0);
        let mut y = L::prev(self.a, 0, 1);
        let mut aw = self.width(x);
        let mut bw = self.width(y);
        let mut xw = self.word(x);
        let mut yw = self.word(y);
        let mut xp = L::prev(self.a, x, 0);
        let mut yp = L::prev(self.a, y, 1);
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let w = aw.min(bw);
            left -= w;
            taken += 1;
            if x == y {
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                if left != 0 {
                    x = L::prev(self.a, 0, 0);
                    y = L::prev(self.a, 0, 1);
                    aw = self.width(x);
                    bw = self.width(y);
                    xw = self.word(x);
                    yw = self.word(y);
                    xp = L::prev(self.a, x, 0);
                    yp = L::prev(self.a, y, 1);
                }
                continue;
            }
            if aw < bw {
                if xp != y {
                    // Hoist reads of next node `xp` BEFORE join() and insert() to warm its cache line!
                    let next_aw = self.width(xp);
                    let next_xw = self.word(xp);
                    let word = self.join(xw, yw);
                    L::set_next(self.a, xp, 0, 0);
                    L::set_prev(self.a, 0, 0, xp);
                    self.insert(y, x, 0);
                    self.set_word(x, word);
                    bw -= w;
                    self.set_width(y, bw);
                    x = xp;
                    aw = next_aw;
                    xw = next_xw;
                    xp = L::prev(self.a, x, 0);
                } else {
                    let word = self.join(xw, yw);
                    self.set_word(x, word);
                    bw -= w;
                    self.set_width(y, bw);
                    xw = word;
                }
            } else if aw > bw {
                if yp != x {
                    let next_bw = self.width(yp);
                    let next_yw = self.word(yp);
                    let word = self.join(xw, yw);
                    L::set_next(self.a, yp, 1, 0);
                    L::set_prev(self.a, 0, 1, yp);
                    self.insert(x, y, 1);
                    self.set_word(y, word);
                    aw -= w;
                    self.set_width(x, aw);
                    y = yp;
                    bw = next_bw;
                    yw = next_yw;
                    yp = L::prev(self.a, y, 1);
                } else {
                    let word = self.join(xw, yw);
                    self.set_word(y, word);
                    aw -= w;
                    self.set_width(x, aw);
                    yw = word;
                }
            } else {
                // aw == bw (37% of steps!): hoist BOTH xp and yp when distinct from y/x!
                if xp != y && yp != x {
                    let next_aw = self.width(xp);
                    let next_xw = self.word(xp);
                    let next_bw = self.width(yp);
                    let next_yw = self.word(yp);
                    let word = self.join(xw, yw);
                    L::set_next(self.a, xp, 0, 0);
                    L::set_prev(self.a, 0, 0, xp);
                    self.insert(y, x, 0);
                    self.set_word(x, word);
                    self.cut(y, 0);
                    L::set_next(self.a, yp, 1, 0);
                    L::set_prev(self.a, 0, 1, yp);
                    if left != 0 {
                        x = xp;
                        y = yp;
                        aw = next_aw;
                        bw = next_bw;
                        xw = next_xw;
                        yw = next_yw;
                        xp = L::prev(self.a, x, 0);
                        yp = L::prev(self.a, y, 1);
                    }
                } else {
                    let word = self.join(xw, yw);
                    self.pop_tail(x, 0);
                    self.insert(y, x, 0);
                    self.set_word(x, word);
                    self.cut(y, 0);
                    self.pop_tail(y, 1);
                    if left != 0 {
                        x = L::prev(self.a, 0, 0);
                        y = L::prev(self.a, 0, 1);
                        aw = self.width(x);
                        bw = self.width(y);
                        xw = self.word(x);
                        yw = self.word(y);
                        xp = L::prev(self.a, x, 0);
                        yp = L::prev(self.a, y, 1);
                    }
                }
            }
        };
        (*remaining, *steps) = (left, taken);
        done
    }

    /// Branchless side-parameterized step (eliminates the unpredictable `aw <= bw` branch).
    fn eliminate_branchless_side(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            let w = aw.min(bw);
            left -= w;
            taken += 1;
            if x == y {
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw == bw {
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                self.cut(y, 0);
                self.pop_tail(y, 1);
            } else {
                let y_loses = aw > bw;
                let side = usize::from(y_loses);
                let loser = if y_loses { y } else { x };
                let winner = if y_loses { x } else { y };
                let win_w = if y_loses { aw } else { bw };
                self.pop_tail(loser, side);
                self.insert(winner, loser, side);
                self.set_word(loser, word);
                self.set_width(winner, win_w - w);
            }
        };
        (*remaining, *steps) = (left, taken);
        done
    }

    // --- Word expansion methods (production memo vs unmemoized vs undo-slack) ---

    fn inline_word(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let id = word & ID_MASK;
        let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let mut bytes = self.short_payload(id).to_le_bytes();
            bytes[height] = byte;
            (bytes, (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append(id, stack, spelling);
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }

    fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule(id);
                let tag = self.a[r + 3];
                if tag >> 28 == 0 {
                    let bytes = self.short_payload(id).to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                    continue;
                }
                let rule = &mut self.a[r..r + RULE];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }

    fn inline_word_no_memo(
        &self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let id = word & ID_MASK;
        let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let mut bytes = self.short_payload(id).to_le_bytes();
            bytes[height] = byte;
            (bytes, (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append_no_memo(id, stack, spelling);
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }

    fn append_no_memo(&self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule(id);
                let tag = self.a[r + 3];
                if tag >> 28 == 0 {
                    let bytes = self.short_payload(id).to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                } else {
                    stack.push(self.a[r + 1]);
                    stack.push(self.a[r]);
                }
            }
        }
    }

    fn inline_word_track_undo(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        undo: &mut Vec<usize>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let id = word & ID_MASK;
        let height = (word >> ID_BITS) as usize;
        undo.clear();
        if height < INLINE {
            let mut bytes = self.short_payload(id).to_le_bytes();
            bytes[height] = byte;
            (bytes, (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append_track_undo(id, stack, undo, spelling);
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }

    fn append_track_undo(
        &mut self,
        word: u32,
        stack: &mut Vec<u32>,
        undo: &mut Vec<usize>,
        dest: &mut Vec<u8>,
    ) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule(id);
                let tag = self.a[r + 3];
                if tag >> 28 == 0 {
                    let bytes = self.short_payload(id).to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                    continue;
                }
                let rule = &mut self.a[r..r + RULE];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32;
                    undo.push(r);
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Typed Narrow Elim + Compact 8-byte SLP Elim (Idea 3)
// ---------------------------------------------------------------------------

/// Separate typed `[u32; 4]` nodes & `[u32; 4]` rules (single bounds check per node).
#[derive(Default)]
struct TypedNarrowState {
    nodes: Vec<[u32; 4]>,
    nodes20: Vec<[u32; 5]>,
    rules: Vec<[u32; 4]>,
    rules8: Vec<[u32; 2]>,
    memo_start: Vec<u32>,
    memo_len: Vec<u32>,
}

#[inline(always)]
fn tn_prev(nodes: &[[u32; 4]], id: u32, side: usize) -> u32 {
    nodes[id as usize][side] & 0xFFFF
}
#[inline(always)]
fn tn_next(nodes: &[[u32; 4]], id: u32, side: usize) -> u32 {
    nodes[id as usize][side] >> 16
}
#[inline(always)]
fn tn_set_prev(nodes: &mut [[u32; 4]], id: u32, side: usize, to: u32) {
    let w = &mut nodes[id as usize][side];
    *w = (*w & 0xFFFF_0000) | to;
}
#[inline(always)]
fn tn_set_next(nodes: &mut [[u32; 4]], id: u32, side: usize, to: u32) {
    let w = &mut nodes[id as usize][side];
    *w = (*w & 0xFFFF) | (to << 16);
}
#[inline(always)]
fn tn_insert(nodes: &mut [[u32; 4]], at: u32, id: u32, side: usize) {
    let next = tn_next(nodes, at, side);
    tn_set_prev(nodes, id, side, at);
    tn_set_next(nodes, id, side, next);
    tn_set_next(nodes, at, side, id);
    tn_set_prev(nodes, next, side, id);
}
#[inline(always)]
fn tn_cut(nodes: &mut [[u32; 4]], id: u32, side: usize) {
    let links = nodes[id as usize][side];
    let p = links & 0xFFFF;
    let n = links >> 16;
    tn_set_next(nodes, p, side, n);
    tn_set_prev(nodes, n, side, p);
}

#[inline(always)]
fn tn_len(rules: &[[u32; 4]], w: u32) -> u32 {
    if w <= 256 {
        u32::from(w != 0)
    } else {
        let r = rules[(w - 257) as usize];
        if r[3] >> 28 == 0 {
            r[3] >> 24
        } else {
            r[2]
        }
    }
}

#[inline(always)]
fn tn_short_payload(rules: &[[u32; 4]], id: u32) -> u128 {
    if id <= 256 {
        return u128::from(id - 1);
    }
    let r = rules[(id - 257) as usize];
    u128::from(r[0])
        | (u128::from(r[1]) << 32)
        | (u128::from(r[2]) << 64)
        | (u128::from(r[3] & 0x00ff_ffff) << 96)
}

#[inline(always)]
fn tn_join(rules: &mut Vec<[u32; 4]>, left: u32, right: u32) -> u32 {
    if left == 0 {
        return right;
    }
    if right == 0 {
        return left;
    }
    let id = rules.len() as u32 + 257;
    let (left_len, right_len) = (left >> ID_BITS, right >> ID_BITS);
    let (left, right) = (left & ID_MASK, right & ID_MASK);
    let len = if left_len < SATURATED && right_len < SATURATED {
        left_len + right_len
    } else {
        tn_len(rules, left) + tn_len(rules, right)
    };
    let rule = if len < INLINE as u32 {
        let bytes = tn_short_payload(rules, left)
            | (tn_short_payload(rules, right) << (8 * left_len))
            | (u128::from(len) << 120);
        [
            bytes as u32,
            (bytes >> 32) as u32,
            (bytes >> 64) as u32,
            (bytes >> 96) as u32,
        ]
    } else {
        [left, right, len, u32::MAX]
    };
    rules.push(rule);
    (len.min(SATURATED) << ID_BITS) | id
}

/// Pure 8-byte SLP rule join: NO short_payload shifts, NO len < INLINE branch, NO self.len(),
/// HALF the rule arena footprint (8 B/rule vs 16 B/rule)!
#[inline(always)]
fn tn_join_8b(rules8: &mut Vec<[u32; 2]>, left: u32, right: u32) -> u32 {
    if left == 0 {
        return right;
    }
    if right == 0 {
        return left;
    }
    let id = rules8.len() as u32 + 257;
    let len = ((left >> ID_BITS) + (right >> ID_BITS)).min(SATURATED);
    rules8.push([left & ID_MASK, right & ID_MASK]);
    (len << ID_BITS) | id
}

fn tn_build_nodes(
    strips_raw: &[u32],
    image_order: &[u32],
    ns: usize,
    n: u32,
    nodes: &mut Vec<[u32; 4]>,
) -> u32 {
    nodes.clear();
    nodes.push([0, 0, 0, 0]);
    let mut widest = 0u32;
    for i in 0..ns {
        let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
        nodes.push([0, 0, s.width, literal(s.byte)]);
        widest = widest.max(s.width);
    }
    let append_to = |nodes: &mut Vec<[u32; 4]>, list: &mut (u32, u32), id: u32, side: usize| {
        let (head, tail) = list;
        if *tail == 0 {
            *head = id;
        } else {
            tn_set_next(nodes, *tail, side, id);
        }
        tn_set_prev(nodes, id, side, *tail);
        *tail = id;
    };
    let (mut terminals, mut order) = ([(0u32, 0u32); 2], [(0u32, 0u32); 2]);
    let mut x = 0u32;
    for i in 0..ns {
        let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
        if x < s.from {
            widest = widest.max(s.from - x);
            let id = nodes.len() as u32;
            nodes.push([0, 0, s.from - x, 0]);
            append_to(nodes, &mut order[0], id, 0);
            append_to(nodes, &mut terminals[1], id, 1);
        }
        append_to(nodes, &mut order[0], i as u32 + 1, 0);
        x = s.from + s.width;
    }
    if x < n {
        widest = widest.max(n - x);
        let id = nodes.len() as u32;
        nodes.push([0, 0, n - x, 0]);
        append_to(nodes, &mut order[0], id, 0);
        append_to(nodes, &mut terminals[1], id, 1);
    }
    x = 0;
    for &i in image_order.iter() {
        let idx = i as usize;
        let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
        if x < s.to {
            widest = widest.max(s.to - x);
            let id = nodes.len() as u32;
            nodes.push([0, 0, s.to - x, 0]);
            append_to(nodes, &mut order[1], id, 1);
            append_to(nodes, &mut terminals[0], id, 0);
        }
        append_to(nodes, &mut order[1], i + 1, 1);
        x = s.to + s.width;
    }
    if x < n {
        widest = widest.max(n - x);
        let id = nodes.len() as u32;
        nodes.push([0, 0, n - x, 0]);
        append_to(nodes, &mut order[1], id, 1);
        append_to(nodes, &mut terminals[0], id, 0);
    }
    for side in 0..2 {
        let mut prev = 0u32;
        for (head, tail) in [terminals[side], order[side]] {
            if tail == 0 {
                continue;
            }
            tn_set_next(nodes, prev, side, head);
            tn_set_prev(nodes, head, side, prev);
            prev = tail;
        }
        tn_set_next(nodes, prev, side, 0);
        tn_set_prev(nodes, 0, side, prev);
    }
    widest
}

fn tn_assign_bottom(nodes: &mut [[u32; 4]], rows: u32) {
    let mut first = tn_next(nodes, 0, 1);
    let mut last = tn_prev(nodes, 0, 1);
    let (mut lo, mut hi) = (0u32, rows);
    while first != last {
        let next = tn_next(nodes, first, 1);
        let prev = tn_prev(nodes, last, 1);
        let width = nodes[first as usize][2];
        hi -= nodes[last as usize][2];
        nodes[first as usize][1] = lo;
        nodes[last as usize][1] = hi;
        lo += width;
        if next == last {
            assert_eq!(lo, hi);
            return;
        }
        first = next;
        last = prev;
    }
    if first != 0 {
        nodes[first as usize][1] = lo;
        lo += nodes[first as usize][2];
    }
    assert_eq!(lo, hi);
}

fn tn_eliminate_typed(nodes: &mut [[u32; 4]], rules: &mut Vec<[u32; 4]>, n: u32) -> usize {
    rules.clear();
    let mut left = n;
    let mut taken = 0usize;
    while left != 0 {
        let x = nodes[0][0] & 0xFFFF;
        let y = nodes[0][1] & 0xFFFF;
        let nx = nodes[x as usize];
        let ny = nodes[y as usize];
        let aw = nx[2];
        let bw = ny[2];
        let w = aw.min(bw);
        left -= w;
        taken += 1;
        if x == y {
            let xp = nx[0] & 0xFFFF;
            let yp = ny[1] & 0xFFFF;
            tn_set_next(nodes, xp, 0, 0);
            tn_set_prev(nodes, 0, 0, xp);
            tn_set_next(nodes, yp, 1, 0);
            tn_set_prev(nodes, 0, 1, yp);
            continue;
        }
        let word = tn_join(rules, nx[3], ny[3]);
        if aw <= bw {
            let xp = nx[0] & 0xFFFF;
            tn_set_next(nodes, xp, 0, 0);
            tn_set_prev(nodes, 0, 0, xp);
            tn_insert(nodes, y, x, 0);
            nodes[x as usize][3] = word;
            if aw == bw {
                tn_cut(nodes, y, 0);
                let yp = ny[1] & 0xFFFF;
                tn_set_next(nodes, yp, 1, 0);
                tn_set_prev(nodes, 0, 1, yp);
            } else {
                nodes[y as usize][2] = bw - w;
            }
        } else {
            let yp = ny[1] & 0xFFFF;
            tn_set_next(nodes, yp, 1, 0);
            tn_set_prev(nodes, 0, 1, yp);
            tn_insert(nodes, x, y, 1);
            nodes[y as usize][3] = word;
            nodes[x as usize][2] = aw - w;
        }
    }
    taken
}

/// Safe speculative cache-line touch of next candidates (`prev(x,0)` and `prev(y,1)`) BEFORE
/// `join()`, testing prefetch without unsafe or extra control-flow branches.
fn tn_eliminate_typed_prefetch_touch(
    nodes: &mut [[u32; 4]],
    rules: &mut Vec<[u32; 4]>,
    n: u32,
) -> usize {
    rules.clear();
    let mut left = n;
    let mut taken = 0usize;
    let mut touch_acc = 0u32;
    while left != 0 {
        let x = nodes[0][0] & 0xFFFF;
        let y = nodes[0][1] & 0xFFFF;
        let nx = nodes[x as usize];
        let ny = nodes[y as usize];
        let aw = nx[2];
        let bw = ny[2];
        let w = aw.min(bw);
        left -= w;
        taken += 1;
        if x == y {
            let xp = nx[0] & 0xFFFF;
            let yp = ny[1] & 0xFFFF;
            tn_set_next(nodes, xp, 0, 0);
            tn_set_prev(nodes, 0, 0, xp);
            tn_set_next(nodes, yp, 1, 0);
            tn_set_prev(nodes, 0, 1, yp);
            continue;
        }
        // Speculatively touch the likely next tail node before join() runs
        let cand = if aw <= bw {
            nx[0] & 0xFFFF
        } else {
            ny[1] & 0xFFFF
        };
        touch_acc ^= nodes[cand as usize][2];
        let word = tn_join(rules, nx[3], ny[3]);
        if aw <= bw {
            let xp = nx[0] & 0xFFFF;
            tn_set_next(nodes, xp, 0, 0);
            tn_set_prev(nodes, 0, 0, xp);
            tn_insert(nodes, y, x, 0);
            nodes[x as usize][3] = word;
            if aw == bw {
                tn_cut(nodes, y, 0);
                let yp = ny[1] & 0xFFFF;
                tn_set_next(nodes, yp, 1, 0);
                tn_set_prev(nodes, 0, 1, yp);
            } else {
                nodes[y as usize][2] = bw - w;
            }
        } else {
            let yp = ny[1] & 0xFFFF;
            tn_set_next(nodes, yp, 1, 0);
            tn_set_prev(nodes, 0, 1, yp);
            tn_insert(nodes, x, y, 1);
            nodes[y as usize][3] = word;
            nodes[x as usize][2] = aw - w;
        }
    }
    black_box(touch_acc);
    taken
}

// --- 20-byte Narrow node with 7-byte inline word (cuts rules by ~60%) ---
const N20_RULE_TAG: u64 = 0xFFu64 << 56;
const N20_PAYLOAD_MASK: u64 = 0x00FF_FFFF_FFFF_FFFF;

#[inline(always)]
fn n20_word(n: [u32; 5]) -> u64 {
    u64::from(n[3]) | (u64::from(n[4]) << 32)
}
#[inline(always)]
fn n20_set_word(nodes: &mut [[u32; 5]], id: u32, w: u64) {
    nodes[id as usize][3] = w as u32;
    nodes[id as usize][4] = (w >> 32) as u32;
}
#[inline(always)]
fn n20_set_prev(nodes: &mut [[u32; 5]], id: u32, side: usize, to: u32) {
    let w = &mut nodes[id as usize][side];
    *w = (*w & 0xFFFF_0000) | to;
}
#[inline(always)]
fn n20_set_next(nodes: &mut [[u32; 5]], id: u32, side: usize, to: u32) {
    let w = &mut nodes[id as usize][side];
    *w = (*w & 0xFFFF) | (to << 16);
}
#[inline(always)]
fn n20_insert(nodes: &mut [[u32; 5]], at: u32, id: u32, side: usize) {
    let next = nodes[at as usize][side] >> 16;
    n20_set_prev(nodes, id, side, at);
    n20_set_next(nodes, id, side, next);
    n20_set_next(nodes, at, side, id);
    n20_set_prev(nodes, next, side, id);
}
#[inline(always)]
fn n20_cut(nodes: &mut [[u32; 5]], id: u32, side: usize) {
    let links = nodes[id as usize][side];
    let p = links & 0xFFFF;
    let n = links >> 16;
    n20_set_next(nodes, p, side, n);
    n20_set_prev(nodes, n, side, p);
}

#[inline(always)]
fn n20_materialize_id(rules: &mut Vec<[u32; 4]>, w: u64) -> u32 {
    let tag = (w >> 56) as u32;
    if tag == 0xFF {
        (w as u32) & ID_MASK
    } else if tag == 1 {
        ((w as u32) & 0xFF) + 1
    } else {
        let id = rules.len() as u32 + 257;
        let payload = w & N20_PAYLOAD_MASK;
        rules.push([payload as u32, (payload >> 32) as u32, 0, tag << 24]);
        id
    }
}

#[inline(always)]
fn n20_join7(rules: &mut Vec<[u32; 4]>, left: u64, right: u64) -> u64 {
    if left == 0 {
        return right;
    }
    if right == 0 {
        return left;
    }
    let ll = (left >> 56) as u32;
    let rl = (right >> 56) as u32;
    if ll + rl <= 7 {
        let payload = (left & N20_PAYLOAD_MASK) | ((right & N20_PAYLOAD_MASK) << (8 * ll));
        return (u64::from(ll + rl) << 56) | payload;
    }
    let l_len = if ll <= 7 {
        ll
    } else {
        (left as u32) >> ID_BITS
    };
    let r_len = if rl <= 7 {
        rl
    } else {
        (right as u32) >> ID_BITS
    };
    let len = if l_len < SATURATED && r_len < SATURATED {
        l_len + r_len
    } else {
        let la = if ll <= 7 {
            ll
        } else {
            tn_len(rules, (left as u32) & ID_MASK)
        };
        let ra = if rl <= 7 {
            rl
        } else {
            tn_len(rules, (right as u32) & ID_MASK)
        };
        la + ra
    };
    let rule = if len < INLINE as u32 {
        let lp = if ll <= 7 {
            u128::from(left & N20_PAYLOAD_MASK)
        } else {
            tn_short_payload(rules, (left as u32) & ID_MASK)
        };
        let rp = if rl <= 7 {
            u128::from(right & N20_PAYLOAD_MASK)
        } else {
            tn_short_payload(rules, (right as u32) & ID_MASK)
        };
        let bytes = lp | (rp << (8 * l_len)) | (u128::from(len) << 120);
        [
            bytes as u32,
            (bytes >> 32) as u32,
            (bytes >> 64) as u32,
            (bytes >> 96) as u32,
        ]
    } else {
        let lid = n20_materialize_id(rules, left);
        let rid = n20_materialize_id(rules, right);
        [lid, rid, len, u32::MAX]
    };
    let id = rules.len() as u32 + 257;
    rules.push(rule);
    N20_RULE_TAG | u64::from((len.min(SATURATED) << ID_BITS) | id)
}

fn n20_build_nodes(
    strips_raw: &[u32],
    image_order: &[u32],
    ns: usize,
    n: u32,
    nodes: &mut Vec<[u32; 5]>,
) -> u32 {
    nodes.clear();
    nodes.push([0; 5]);
    let mut widest = 0u32;
    for i in 0..ns {
        let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
        let w = (1u64 << 56) | u64::from(s.byte);
        nodes.push([0, 0, s.width, w as u32, (w >> 32) as u32]);
        widest = widest.max(s.width);
    }
    let append_to = |nodes: &mut Vec<[u32; 5]>, list: &mut (u32, u32), id: u32, side: usize| {
        let (head, tail) = list;
        if *tail == 0 {
            *head = id;
        } else {
            n20_set_next(nodes, *tail, side, id);
        }
        n20_set_prev(nodes, id, side, *tail);
        *tail = id;
    };
    let (mut terminals, mut order) = ([(0u32, 0u32); 2], [(0u32, 0u32); 2]);
    let mut x = 0u32;
    for i in 0..ns {
        let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
        if x < s.from {
            widest = widest.max(s.from - x);
            let id = nodes.len() as u32;
            nodes.push([0, 0, s.from - x, 0, 0]);
            append_to(nodes, &mut order[0], id, 0);
            append_to(nodes, &mut terminals[1], id, 1);
        }
        append_to(nodes, &mut order[0], i as u32 + 1, 0);
        x = s.from + s.width;
    }
    if x < n {
        widest = widest.max(n - x);
        let id = nodes.len() as u32;
        nodes.push([0, 0, n - x, 0, 0]);
        append_to(nodes, &mut order[0], id, 0);
        append_to(nodes, &mut terminals[1], id, 1);
    }
    x = 0;
    for &i in image_order.iter() {
        let idx = i as usize;
        let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
        if x < s.to {
            widest = widest.max(s.to - x);
            let id = nodes.len() as u32;
            nodes.push([0, 0, s.to - x, 0, 0]);
            append_to(nodes, &mut order[1], id, 1);
            append_to(nodes, &mut terminals[0], id, 0);
        }
        append_to(nodes, &mut order[1], i + 1, 1);
        x = s.to + s.width;
    }
    if x < n {
        widest = widest.max(n - x);
        let id = nodes.len() as u32;
        nodes.push([0, 0, n - x, 0, 0]);
        append_to(nodes, &mut order[1], id, 1);
        append_to(nodes, &mut terminals[0], id, 0);
    }
    for side in 0..2 {
        let mut prev = 0u32;
        for (head, tail) in [terminals[side], order[side]] {
            if tail == 0 {
                continue;
            }
            n20_set_next(nodes, prev, side, head);
            n20_set_prev(nodes, head, side, prev);
            prev = tail;
        }
        n20_set_next(nodes, prev, side, 0);
        n20_set_prev(nodes, 0, side, prev);
    }
    widest
}

fn n20_assign_bottom(nodes: &mut [[u32; 5]], rows: u32) {
    let mut first = nodes[0][1] >> 16;
    let mut last = nodes[0][1] & 0xFFFF;
    let (mut lo, mut hi) = (0u32, rows);
    while first != last {
        let next = nodes[first as usize][1] >> 16;
        let prev = nodes[last as usize][1] & 0xFFFF;
        let width = nodes[first as usize][2];
        hi -= nodes[last as usize][2];
        nodes[first as usize][1] = lo;
        nodes[last as usize][1] = hi;
        lo += width;
        if next == last {
            assert_eq!(lo, hi);
            return;
        }
        first = next;
        last = prev;
    }
    if first != 0 {
        nodes[first as usize][1] = lo;
        lo += nodes[first as usize][2];
    }
    assert_eq!(lo, hi);
}

fn n20_eliminate(nodes: &mut [[u32; 5]], rules: &mut Vec<[u32; 4]>, n: u32) -> usize {
    rules.clear();
    let mut left = n;
    let mut taken = 0usize;
    while left != 0 {
        let x = nodes[0][0] & 0xFFFF;
        let y = nodes[0][1] & 0xFFFF;
        let nx = nodes[x as usize];
        let ny = nodes[y as usize];
        let aw = nx[2];
        let bw = ny[2];
        let w = aw.min(bw);
        left -= w;
        taken += 1;
        if x == y {
            let xp = nx[0] & 0xFFFF;
            let yp = ny[1] & 0xFFFF;
            n20_set_next(nodes, xp, 0, 0);
            n20_set_prev(nodes, 0, 0, xp);
            n20_set_next(nodes, yp, 1, 0);
            n20_set_prev(nodes, 0, 1, yp);
            continue;
        }
        let word = n20_join7(rules, n20_word(nx), n20_word(ny));
        if aw <= bw {
            let xp = nx[0] & 0xFFFF;
            n20_set_next(nodes, xp, 0, 0);
            n20_set_prev(nodes, 0, 0, xp);
            n20_insert(nodes, y, x, 0);
            n20_set_word(nodes, x, word);
            if aw == bw {
                n20_cut(nodes, y, 0);
                let yp = ny[1] & 0xFFFF;
                n20_set_next(nodes, yp, 1, 0);
                n20_set_prev(nodes, 0, 1, yp);
            } else {
                nodes[y as usize][2] = bw - w;
            }
        } else {
            let yp = ny[1] & 0xFFFF;
            n20_set_next(nodes, yp, 1, 0);
            n20_set_prev(nodes, 0, 1, yp);
            n20_insert(nodes, x, y, 1);
            n20_set_word(nodes, y, word);
            nodes[x as usize][2] = aw - w;
        }
    }
    taken
}

/// Compact 8-byte SLP elimination with register-cached `(x, y)` and hoisted next-node load!
fn tn_eliminate_8b_slp(
    nodes: &mut [[u32; 4]],
    rules8: &mut Vec<[u32; 2]>,
    n: u32,
    hoist: bool,
) -> usize {
    rules8.clear();
    let mut left = n;
    let mut taken = 0usize;
    if !hoist {
        while left != 0 {
            let x = nodes[0][0] & 0xFFFF;
            let y = nodes[0][1] & 0xFFFF;
            let nx = nodes[x as usize];
            let ny = nodes[y as usize];
            let aw = nx[2];
            let bw = ny[2];
            let w = aw.min(bw);
            left -= w;
            taken += 1;
            if x == y {
                let xp = nx[0] & 0xFFFF;
                let yp = ny[1] & 0xFFFF;
                tn_set_next(nodes, xp, 0, 0);
                tn_set_prev(nodes, 0, 0, xp);
                tn_set_next(nodes, yp, 1, 0);
                tn_set_prev(nodes, 0, 1, yp);
                continue;
            }
            let word = tn_join_8b(rules8, nx[3], ny[3]);
            if aw <= bw {
                let xp = nx[0] & 0xFFFF;
                tn_set_next(nodes, xp, 0, 0);
                tn_set_prev(nodes, 0, 0, xp);
                tn_insert(nodes, y, x, 0);
                nodes[x as usize][3] = word;
                if aw == bw {
                    tn_cut(nodes, y, 0);
                    let yp = ny[1] & 0xFFFF;
                    tn_set_next(nodes, yp, 1, 0);
                    tn_set_prev(nodes, 0, 1, yp);
                } else {
                    nodes[y as usize][2] = bw - w;
                }
            } else {
                let yp = ny[1] & 0xFFFF;
                tn_set_next(nodes, yp, 1, 0);
                tn_set_prev(nodes, 0, 1, yp);
                tn_insert(nodes, x, y, 1);
                nodes[y as usize][3] = word;
                nodes[x as usize][2] = aw - w;
            }
        }
        return taken;
    }

    // Hoisted + register-cached 8-byte SLP elimination
    let mut x = nodes[0][0] & 0xFFFF;
    let mut y = nodes[0][1] & 0xFFFF;
    let mut nx = nodes[x as usize];
    let mut ny = nodes[y as usize];
    let mut aw = nx[2];
    let mut bw = ny[2];
    let mut xw = nx[3];
    let mut yw = ny[3];
    let mut xp = nx[0] & 0xFFFF;
    let mut yp = ny[1] & 0xFFFF;
    while left != 0 {
        let w = aw.min(bw);
        left -= w;
        taken += 1;
        if x == y {
            tn_set_next(nodes, xp, 0, 0);
            tn_set_prev(nodes, 0, 0, xp);
            tn_set_next(nodes, yp, 1, 0);
            tn_set_prev(nodes, 0, 1, yp);
            if left != 0 {
                x = nodes[0][0] & 0xFFFF;
                y = nodes[0][1] & 0xFFFF;
                nx = nodes[x as usize];
                ny = nodes[y as usize];
                aw = nx[2];
                bw = ny[2];
                xw = nx[3];
                yw = ny[3];
                xp = nx[0] & 0xFFFF;
                yp = ny[1] & 0xFFFF;
            }
            continue;
        }
        if aw < bw {
            if xp != y {
                let next_nx = nodes[xp as usize];
                let word = tn_join_8b(rules8, xw, yw);
                tn_set_next(nodes, xp, 0, 0);
                tn_set_prev(nodes, 0, 0, xp);
                tn_insert(nodes, y, x, 0);
                nodes[x as usize][3] = word;
                bw -= w;
                nodes[y as usize][2] = bw;
                x = xp;
                aw = next_nx[2];
                xw = next_nx[3];
                xp = nodes[x as usize][0] & 0xFFFF;
            } else {
                let word = tn_join_8b(rules8, xw, yw);
                nodes[x as usize][3] = word;
                bw -= w;
                nodes[y as usize][2] = bw;
                xw = word;
            }
        } else if aw > bw {
            if yp != x {
                let next_ny = nodes[yp as usize];
                let word = tn_join_8b(rules8, xw, yw);
                tn_set_next(nodes, yp, 1, 0);
                tn_set_prev(nodes, 0, 1, yp);
                tn_insert(nodes, x, y, 1);
                nodes[y as usize][3] = word;
                aw -= w;
                nodes[x as usize][2] = aw;
                y = yp;
                bw = next_ny[2];
                yw = next_ny[3];
                yp = nodes[y as usize][1] & 0xFFFF;
            } else {
                let word = tn_join_8b(rules8, xw, yw);
                nodes[y as usize][3] = word;
                aw -= w;
                nodes[x as usize][2] = aw;
                yw = word;
            }
        } else if xp != y && yp != x {
            let next_nx = nodes[xp as usize];
            let next_ny = nodes[yp as usize];
            let word = tn_join_8b(rules8, xw, yw);
            tn_set_next(nodes, xp, 0, 0);
            tn_set_prev(nodes, 0, 0, xp);
            tn_insert(nodes, y, x, 0);
            nodes[x as usize][3] = word;
            tn_cut(nodes, y, 0);
            tn_set_next(nodes, yp, 1, 0);
            tn_set_prev(nodes, 0, 1, yp);
            if left != 0 {
                x = xp;
                y = yp;
                aw = next_nx[2];
                bw = next_ny[2];
                xw = next_nx[3];
                yw = next_ny[3];
                xp = nodes[x as usize][0] & 0xFFFF;
                yp = nodes[y as usize][1] & 0xFFFF;
            }
        } else {
            let word = tn_join_8b(rules8, xw, yw);
            let cur_xp = nodes[x as usize][0] & 0xFFFF;
            tn_set_next(nodes, cur_xp, 0, 0);
            tn_set_prev(nodes, 0, 0, cur_xp);
            tn_insert(nodes, y, x, 0);
            nodes[x as usize][3] = word;
            tn_cut(nodes, y, 0);
            let cur_yp = nodes[y as usize][1] & 0xFFFF;
            tn_set_next(nodes, cur_yp, 1, 0);
            tn_set_prev(nodes, 0, 1, cur_yp);
            if left != 0 {
                x = nodes[0][0] & 0xFFFF;
                y = nodes[0][1] & 0xFFFF;
                nx = nodes[x as usize];
                ny = nodes[y as usize];
                aw = nx[2];
                bw = ny[2];
                xw = nx[3];
                yw = ny[3];
                xp = nx[0] & 0xFFFF;
                yp = ny[1] & 0xFFFF;
            }
        }
    }
    taken
}

/// Expand an 8-byte SLP word (short <= 15 B into `[u8; 16]`, long into `spelling` with post-order memo).
fn expand_8b_word(
    rules8: &[[u32; 2]],
    memo_start: &mut [u32],
    memo_len: &mut [u32],
    handle: u32,
    byte: u8,
    stack: &mut Vec<u32>,
    spelling: &mut Vec<u8>,
) -> ([u8; INLINE], u8) {
    let id = handle & ID_MASK;
    let height = (handle >> ID_BITS) as usize;
    if height < INLINE {
        let mut entry = [0u8; INLINE];
        let mut pos = 0usize;
        stack.clear();
        stack.push(id);
        while let Some(cur) = stack.pop() {
            if cur <= 256 {
                entry[pos] = (cur - 1) as u8;
                pos += 1;
            } else {
                let r = rules8[(cur - 257) as usize];
                stack.push(r[1]);
                stack.push(r[0]);
            }
        }
        debug_assert_eq!(pos, height);
        entry[pos] = byte;
        (entry, (pos + 1) as u8)
    } else {
        let start = spelling.len() as u32;
        stack.clear();
        stack.push(id);
        while let Some(cur) = stack.pop() {
            if cur & 0x8000_0000 != 0 {
                let r = (cur & 0x7fff_ffff) as usize;
                memo_len[r] = spelling.len() as u32 - memo_start[r];
            } else if cur <= 256 {
                spelling.push((cur - 1) as u8);
            } else {
                let r = (cur - 257) as usize;
                let mlen = memo_len[r];
                if mlen != 0 {
                    let ms = memo_start[r] as usize;
                    spelling.extend_from_within(ms..ms + mlen as usize);
                } else {
                    memo_start[r] = spelling.len() as u32;
                    stack.push(0x8000_0000 | (r as u32));
                    let pair = rules8[r];
                    stack.push(pair[1]);
                    stack.push(pair[0]);
                }
            }
        }
        spelling.push(byte);
        (long_slot(start, spelling.len() as u32), LONG)
    }
}

// ---------------------------------------------------------------------------
// Dictionary variants (Idea 2)
// ---------------------------------------------------------------------------

#[inline(always)]
fn spelling_hash(entry: &[u8; INLINE], len: u8, bytes: &[u8]) -> u64 {
    let (low, high, middle) = if len == LONG {
        let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        (part(0), part(bytes.len() - 8), part(bytes.len() / 2 - 4))
    } else {
        let value = u128::from_le_bytes(*entry);
        (value as u64, (value >> 64) as u64, 0)
    };
    let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ middle.wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WordMode {
    Prod,
    NoMemoExpand,
    TrimMemoSlack,
    NoContentsDedup,
    NoDedup,
    Pairs2048InlineOnlyDedup,
    Pairs2048LongOnlyDedup,
    BidiPreShifted,
    ProdForward,
    ProdForwardUnroll4,
    /// Skip long-word expansion in A6b; expand SLP on first use directly into `out`, copy earlier
    /// expansions via `out.copy_within` on reuse (forward emission).
    SlpEmitCopyOut,
    /// Skip long-word expansion in A6b; expand SLP via DFS directly into `out` on every long code
    /// (forward emission).
    SlpEmitDfs,
}

impl WordMode {
    fn name(self) -> &'static str {
        match self {
            WordMode::Prod => "prod_baseline",
            WordMode::NoMemoExpand => "no_memo_expand",
            WordMode::TrimMemoSlack => "dict_trim_memo_slack",
            WordMode::NoContentsDedup => "dict_no_contents_dedup",
            WordMode::NoDedup => "dict_no_dedup",
            WordMode::Pairs2048InlineOnlyDedup => "dict_p2048_inline_dedup",
            WordMode::Pairs2048LongOnlyDedup => "dict_p2048_long_dedup",
            WordMode::BidiPreShifted => "bidi_pre_shifted",
            WordMode::ProdForward => "prod_forward_emit",
            WordMode::ProdForwardUnroll4 => "prod_forward_unroll4",
            WordMode::SlpEmitCopyOut => "slp_emit_copy_out",
            WordMode::SlpEmitDfs => "slp_emit_dfs",
        }
    }

    fn is_forward(self) -> bool {
        matches!(
            self,
            WordMode::ProdForward
                | WordMode::ProdForwardUnroll4
                | WordMode::SlpEmitCopyOut
                | WordMode::SlpEmitDfs
        )
    }

    fn is_slp_emit(self) -> bool {
        matches!(self, WordMode::SlpEmitCopyOut | WordMode::SlpEmitDfs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DispatchMode {
    Prod,
    ForceForward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StepMode {
    Prod,
    NoJoinAblat,
    RegXY,
    HoistNext,
    BranchlessSide,
    TypedNarrow,
    TypedPrefetchTouch,
    Inline7Node20,
    Compact8bSlp,
    Compact8bSlpHoist,
}

impl StepMode {
    fn name(self) -> &'static str {
        match self {
            StepMode::Prod => "prod_baseline",
            StepMode::NoJoinAblat => "ablat_no_join",
            StepMode::RegXY => "steps_reg_xy",
            StepMode::HoistNext => "steps_hoist_next",
            StepMode::BranchlessSide => "steps_branchless_side",
            StepMode::TypedNarrow => "steps_typed_narrow",
            StepMode::TypedPrefetchTouch => "steps_typed_prefetch",
            StepMode::Inline7Node20 => "steps_inline7_node20",
            StepMode::Compact8bSlp => "steps_compact_8b_slp",
            StepMode::Compact8bSlpHoist => "steps_compact_8b_hoist",
        }
    }
}

// ---------------------------------------------------------------------------
// Emission loops (production mirror + pre-shifted bidi + forward + SLP emit)
// ---------------------------------------------------------------------------

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_forward(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    let mask = (1 << bits) - 1;
    let mut j = mark;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        } as usize;
        j = edge & mask;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_forward_unroll4(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    if !extra.is_empty() {
        emit_forward(edges, extra, spelling, inline16, lens, n, mark, bits, out);
        return;
    }
    out.resize(n + 4 * INLINE, 0);
    let mut at = 0usize;
    let mask = (1 << bits) - 1;
    let mut j = mark;
    let mut rem = edges.len();
    while rem >= 4 && at < n {
        let e0 = edges[j as usize];
        let c0 = (e0 >> bits) as usize;
        let j1 = e0 & mask;
        let l0 = lens[c0];
        if l0 != LONG && j1 != mark {
            let e1 = edges[j1 as usize];
            let c1 = (e1 >> bits) as usize;
            let j2 = e1 & mask;
            let l1 = lens[c1];
            if l1 != LONG && j2 != mark {
                let e2 = edges[j2 as usize];
                let c2 = (e2 >> bits) as usize;
                let j3 = e2 & mask;
                let l2 = lens[c2];
                if l2 != LONG && j3 != mark {
                    let e3 = edges[j3 as usize];
                    let c3 = (e3 >> bits) as usize;
                    let j4 = e3 & mask;
                    let l3 = lens[c3];
                    if l3 != LONG {
                        out[at..at + INLINE].copy_from_slice(&inline16[c0]);
                        at += usize::from(l0);
                        out[at..at + INLINE].copy_from_slice(&inline16[c1]);
                        at += usize::from(l1);
                        out[at..at + INLINE].copy_from_slice(&inline16[c2]);
                        at += usize::from(l2);
                        out[at..at + INLINE].copy_from_slice(&inline16[c3]);
                        at += usize::from(l3);
                        j = j4;
                        rem -= 4;
                        if j == mark || at >= n {
                            break;
                        }
                        continue;
                    }
                }
            }
        }
        // Fallback 1 step
        j = j1;
        rem -= 1;
        if l0 != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[c0]);
            at += usize::from(l0);
        } else {
            let word = long_word(&inline16[c0], spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }
    while rem > 0 && at < n && j != mark {
        let edge = edges[j as usize];
        let code = (edge >> bits) as usize;
        j = edge & mask;
        rem -= 1;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_bidirectional(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    before: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0, n);
    for pair in 0..=edges.len() / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = edges[front as usize];
        let cf = (ef >> bits) as usize;
        let single = front == back;
        let lf = lens[cf];
        if lf != LONG && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&inline[cf]);
            lo += usize::from(lf);
        } else {
            let word = if lf == LONG {
                long_word(&inline[cf], spelling)
            } else {
                &inline[cf][..usize::from(lf)]
            };
            let take = word.len().min(hi - lo);
            out[lo..lo + take].copy_from_slice(&word[..take]);
            lo += take;
        }
        if single {
            break;
        }
        let eb = edges[back as usize];
        let cb = (eb >> bits) as usize;
        let lb = lens[cb];
        if lb != LONG && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= usize::from(lb);
        } else {
            let word = if lb == LONG {
                long_word(&inline[cb], spelling)
            } else {
                &inline[cb][..usize::from(lb)]
            };
            let take = word.len().min(hi - lo);
            out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
            hi -= take;
        }
        (prev, front) = (front, (ef & mask) ^ prev);
        (next, back) = (back, (eb & mask) ^ next);
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_bidirectional_preshifted(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    inline_back: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    before: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0, n);
    for pair in 0..=edges.len() / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = edges[front as usize];
        let cf = (ef >> bits) as usize;
        let single = front == back;
        let lf = lens[cf];
        if lf != LONG && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&inline[cf]);
            lo += usize::from(lf);
        } else {
            let word = if lf == LONG {
                long_word(&inline[cf], spelling)
            } else {
                &inline[cf][..usize::from(lf)]
            };
            let take = word.len().min(hi - lo);
            out[lo..lo + take].copy_from_slice(&word[..take]);
            lo += take;
        }
        if single {
            break;
        }
        let eb = edges[back as usize];
        let cb = (eb >> bits) as usize;
        let lb = lens[cb];
        if lb != LONG && hi - lo >= INLINE {
            out[hi - INLINE..hi].copy_from_slice(&inline_back[cb]);
            hi -= usize::from(lb);
        } else {
            let word = if lb == LONG {
                long_word(&inline[cb], spelling)
            } else {
                &inline[cb][..usize::from(lb)]
            };
            let take = word.len().min(hi - lo);
            out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
            hi -= take;
        }
        (prev, front) = (front, (ef & mask) ^ prev);
        (next, back) = (back, (eb & mask) ^ next);
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

/// Emit long words directly from compact SLP into `out`!
/// When `COPY_OUT == true`, the first occurrence of a long word expands from SLP into `out` and
/// records `(at, len)` in `first_pos[code]`; every subsequent occurrence copies from `out` via
/// `out.copy_within`. When `COPY_OUT == false`, every occurrence expands via SLP DFS directly.
#[allow(clippy::too_many_arguments)]
fn emit_slp_forward<const COPY_OUT: bool>(
    edges: &[u32],
    extra: &[u32],
    slp_rules: &[[u32; 4]],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    first_pos: &mut [(u32, u32)],
    stack: &mut Vec<u32>,
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let mask = (1 << bits) - 1;
    let mut j = mark;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        } as usize;
        j = edge & mask;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else if COPY_OUT && first_pos[code].1 != 0 {
            let (src, wlen) = first_pos[code];
            let take = (wlen as usize).min(n - at);
            out.copy_within(src as usize..src as usize + take, at);
            at += take;
        } else {
            // inline16[code] stores (rule_id: u32, tail_byte: u32)
            let (root_id, tail_byte) = long_bounds(&inline16[code]);
            let start = at;
            stack.clear();
            stack.push(root_id as u32);
            while let Some(id) = stack.pop() {
                if id <= 256 {
                    if at < n {
                        out[at] = (id - 1) as u8;
                        at += 1;
                    }
                } else {
                    let r = slp_rules[(id - 257) as usize];
                    let tag = r[3];
                    if tag >> 28 == 0 {
                        let plen = (tag >> 24) as usize;
                        let bytes = (u128::from(r[0])
                            | (u128::from(r[1]) << 32)
                            | (u128::from(r[2]) << 64)
                            | (u128::from(r[3] & 0x00ff_ffff) << 96))
                            .to_le_bytes();
                        let take = plen.min(n - at);
                        out[at..at + take].copy_from_slice(&bytes[..take]);
                        at += take;
                    } else {
                        stack.push(r[1]);
                        stack.push(r[0]);
                    }
                }
            }
            if at < n {
                out[at] = tail_byte as u8;
                at += 1;
            }
            if COPY_OUT && at - start > 0 && at <= n {
                first_pos[code] = (start as u32, (at - start) as u32);
            }
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

// ---------------------------------------------------------------------------
// Kernel mirror supporting all 3 ideas + sub-phase timings
// ---------------------------------------------------------------------------

#[derive(Default, Clone, Copy)]
struct PhaseNs {
    a6a: u64,
    a6b_setup: u64,
    a6b_steps: u64,
    a6b_dict: u64,
    a6c: u64,
    a7: u64,
}

impl PhaseNs {
    fn add(&mut self, o: PhaseNs) {
        self.a6a += o.a6a;
        self.a6b_setup += o.a6b_setup;
        self.a6b_steps += o.a6b_steps;
        self.a6b_dict += o.a6b_dict;
        self.a6c += o.a6c;
        self.a7 += o.a7;
    }
    fn a6b(self) -> u64 {
        self.a6b_setup + self.a6b_steps + self.a6b_dict
    }
    fn total(self) -> u64 {
        self.a6a + self.a6b() + self.a6c + self.a7
    }
}

#[derive(Default, Clone, Copy)]
struct BlockStats {
    n: usize,
    runs: usize,
    m: usize,
    ns: usize,
    open_nodes: usize,
    steps: usize,
    rules: usize,
    bands: usize,
    dict_entries: usize,
    long_entries: usize,
    spelling_bytes: usize,
    spelling_used_bytes: usize,
    literal_codes: usize,
    long_codes: usize,
    wide_arena: usize,
    wide_edges: usize,
    bidi_blocks: usize,
    slow_pushes: usize,
    arena_bytes: usize,
}

impl BlockStats {
    fn add(&mut self, o: BlockStats) {
        self.n += o.n;
        self.runs += o.runs;
        self.m += o.m;
        self.ns += o.ns;
        self.open_nodes += o.open_nodes;
        self.steps += o.steps;
        self.rules += o.rules;
        self.bands += o.bands;
        self.dict_entries += o.dict_entries;
        self.long_entries += o.long_entries;
        self.spelling_bytes += o.spelling_bytes;
        self.spelling_used_bytes += o.spelling_used_bytes;
        self.literal_codes += o.literal_codes;
        self.long_codes += o.long_codes;
        self.wide_arena += o.wide_arena;
        self.wide_edges += o.wide_edges;
        self.bidi_blocks += o.bidi_blocks;
        self.slow_pushes += o.slow_pushes;
        self.arena_bytes += o.arena_bytes;
    }
}

#[derive(Default)]
struct OpenKernel {
    spill: Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    inline_back: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    undo: Vec<usize>,
    slp_rules: Vec<[u32; 4]>,
    first_pos: Vec<(u32, u32)>,
    cache_buf: Vec<u32>,
    typed: TypedNarrowState,
}

impl OpenKernel {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn arena_words(ns: usize, m: usize) -> usize {
        let wide = Self::nodes_most(ns) > NARROW_NODES;
        let stride = if wide { Wide::STRIDE } else { Narrow::STRIDE };
        (4 * ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
    }

    #[allow(clippy::too_many_arguments)]
    fn run_block(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit_policy: LimitPolicy,
        word_mode: WordMode,
        step_mode: StepMode,
        dispatch: DispatchMode,
        out: &mut Vec<u8>,
        collect_stats: bool,
    ) -> (PhaseNs, BlockStats) {
        let mut ph = PhaseNs::default();
        let mut st = BlockStats {
            n,
            ..BlockStats::default()
        };

        // --- A6a: first pass (including any adaptive pre-pass!) ---
        let t0 = Instant::now();
        out.resize(n + INLINE, 0);
        let (f, ns, heads, slow_pushes) = match limit_policy {
            LimitPolicy::Fixed(lim) => {
                let mut pass = Pass {
                    queues: Queues::new(&mut out[..], &mut self.spill, Cap::NONE.m),
                    edges: &mut self.edges,
                    ns: 0,
                    cap_strips: Cap::NONE.strips,
                    f: First::default(),
                };
                maximal(records, |c, pos, width| {
                    pass.take::<false>(c, pos, width, p, lim);
                });
                (
                    pass.f,
                    pass.ns,
                    pass.queues.heads(),
                    pass.queues.slow_pushes,
                )
            }
            LimitPolicy::FixedFast64(lim) => {
                let mut pass = Pass {
                    queues: Queues::new(&mut out[..], &mut self.spill, Cap::NONE.m),
                    edges: &mut self.edges,
                    ns: 0,
                    cap_strips: Cap::NONE.strips,
                    f: First::default(),
                };
                maximal(records, |c, pos, width| {
                    pass.take::<true>(c, pos, width, p, lim);
                });
                (
                    pass.f,
                    pass.ns,
                    pass.queues.heads(),
                    pass.queues.slow_pushes,
                )
            }
            LimitPolicy::PerBlockDensity => {
                let lim = choose_block_limit_density(records.len(), n);
                let mut pass = Pass {
                    queues: Queues::new(&mut out[..], &mut self.spill, Cap::NONE.m),
                    edges: &mut self.edges,
                    ns: 0,
                    cap_strips: Cap::NONE.strips,
                    f: First::default(),
                };
                maximal(records, |c, pos, width| {
                    pass.take::<false>(c, pos, width, p, lim);
                });
                (
                    pass.f,
                    pass.ns,
                    pass.queues.heads(),
                    pass.queues.slow_pushes,
                )
            }
            LimitPolicy::PerBlockHist { strip_to_row_ratio } => {
                let lim = choose_block_limit_hist(records, strip_to_row_ratio);
                let mut pass = Pass {
                    queues: Queues::new(&mut out[..], &mut self.spill, Cap::NONE.m),
                    edges: &mut self.edges,
                    ns: 0,
                    cap_strips: Cap::NONE.strips,
                    f: First::default(),
                };
                maximal(records, |c, pos, width| {
                    pass.take::<false>(c, pos, width, p, lim);
                });
                (
                    pass.f,
                    pass.ns,
                    pass.queues.heads(),
                    pass.queues.slow_pushes,
                )
            }
            LimitPolicy::PerByteAdaptive => {
                let lims = choose_per_byte_limits(records);
                let mut pass = Pass {
                    queues: Queues::new(&mut out[..], &mut self.spill, Cap::NONE.m),
                    edges: &mut self.edges,
                    ns: 0,
                    cap_strips: Cap::NONE.strips,
                    f: First::default(),
                };
                maximal(records, |c, pos, width| {
                    pass.take::<false>(c, pos, width, p, lims[usize::from(c)]);
                });
                (
                    pass.f,
                    pass.ns,
                    pass.queues.heads(),
                    pass.queues.slow_pushes,
                )
            }
        };
        let t1 = Instant::now();
        ph.a6a = t1.duration_since(t0).as_nanos() as u64;

        let First {
            thick_rows,
            thin,
            thick,
            m,
            mark,
            runs,
        } = f;
        st.runs = runs;
        st.m = m as usize;
        st.ns = ns;
        st.slow_pushes = slow_pushes;

        // --- A6b-setup: image_order + arena setup ---
        let t2 = Instant::now();
        let all: [u32; 256] = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        let first = starts(all);
        let mut write = starts(thick);
        let most = Self::nodes_most(ns);
        let wide_arena = most > NARROW_NODES;
        if wide_arena {
            st.wide_arena = 1;
        }
        let need = Self::arena_words(ns, m as usize);
        if self.edges.len() < need {
            let add = need - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(need, 0);
        }
        {
            let (strips_raw, rest_edges) = self.edges.split_at_mut(3 * ns);
            let image_order = &mut rest_edges[..ns];
            for i in 0..ns {
                let sw = &mut strips_raw[3 * i..3 * i + 3];
                let byte = (sw[2] & 0xFF) as u8;
                let a = usize::from(byte);
                sw[1] += first[a];
                image_order[write[a] as usize] = i as u32;
                write[a] += 1;
            }
        }
        self.bands.clear();
        self.spelling.clear();
        if n >= 65_536 && !word_mode.is_slp_emit() {
            self.spelling.reserve_exact(48 * 1024);
        }

        let cursors = if !wide_arena && step_mode == StepMode::Inline7Node20 {
            self.induced_inline7_node20(ns, n as u32, &thin, m, &mut ph, &mut st, t2)
        } else if !wide_arena
            && matches!(
                step_mode,
                StepMode::TypedNarrow
                    | StepMode::TypedPrefetchTouch
                    | StepMode::Compact8bSlp
                    | StepMode::Compact8bSlpHoist
            )
        {
            self.induced_typed_narrow(ns, n as u32, &thin, m, step_mode, &mut ph, &mut st, t2)
        } else if wide_arena {
            self.induced_with::<Wide>(
                ns, n as u32, &thin, m, most, word_mode, step_mode, &mut ph, &mut st, t2,
            )
        } else {
            self.induced_with::<Narrow>(
                ns, n as u32, &thin, m, most, word_mode, step_mode, &mut ph, &mut st, t2,
            )
        };

        if step_mode == StepMode::NoJoinAblat {
            return (ph, st);
        }

        st.bands = self.bands.len();
        st.dict_entries = self.lens.len();
        st.spelling_bytes = self.spelling.len();
        if collect_stats {
            st.long_entries = self.lens.iter().filter(|&&l| l == LONG).count();
            let mut used_bytes = 0usize;
            for (code, &l) in self.lens.iter().enumerate() {
                if l == LONG && !word_mode.is_slp_emit() {
                    let (s, e) = long_bounds(&self.inline16[code]);
                    used_bytes += e - s;
                }
            }
            st.spelling_used_bytes = used_bytes;
            let mut k = 0u32;
            for &b in &self.bands {
                let w = (b.end() - k) as usize;
                let code = b.code() as usize;
                if code < 256 {
                    st.literal_codes += w;
                } else if self.lens[code] == LONG {
                    st.long_codes += w;
                }
                k = b.end();
            }
        }

        // --- A6c: retained-row wiring ---
        let t5 = Instant::now();
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide_edges = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if wide_edges {
            st.wide_edges = 1;
        }
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide_edges {
            self.extra.resize(m as usize, 0);
        }
        let use_bidi =
            !wide_edges && m >= 65_536 && dispatch == DispatchMode::Prod && !word_mode.is_forward();
        let predecessor = if use_bidi {
            st.bidi_blocks = 1;
            let edges = &mut self.edges[..m as usize];
            edges.fill(0);
            let mut pred = mark;
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = QueueReader::new(out, &self.spill, &heads, a);
                for &band in &self.bands[cursors[a]..end] {
                    queue.wire_xor(
                        edges,
                        band.dst() as usize,
                        (band.end() - k) as usize,
                        band.code() << bits,
                        mark,
                        &mut pred,
                    );
                    k = band.end();
                }
            }
            Some(pred)
        } else {
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = QueueReader::new(out, &self.spill, &heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let bend = b.end();
                    let span = (bend - k) as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide_edges { 0 } else { bcode << bits };
                    if wide_edges {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                    k = bend;
                }
            }
            None
        };
        let t6 = Instant::now();
        ph.a6c = t6.duration_since(t5).as_nanos() as u64;

        // --- A7: emission (walk + spelling) ---
        let t7 = Instant::now();
        match word_mode {
            WordMode::BidiPreShifted => {
                if let Some(before) = predecessor {
                    emit_bidirectional_preshifted(
                        &self.edges[..m as usize],
                        &self.spelling,
                        &self.inline16,
                        &self.inline_back,
                        &self.lens,
                        n,
                        mark,
                        before,
                        bits,
                        out,
                    );
                } else {
                    emit_forward(
                        &self.edges[..m as usize],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        mark,
                        bits,
                        out,
                    );
                }
            }
            WordMode::ProdForwardUnroll4 => {
                emit_forward_unroll4(
                    &self.edges[..m as usize],
                    &self.extra,
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    mark,
                    bits,
                    out,
                );
            }
            WordMode::SlpEmitCopyOut => {
                self.first_pos.clear();
                self.first_pos.resize(self.lens.len(), (0, 0));
                emit_slp_forward::<true>(
                    &self.edges[..m as usize],
                    &self.extra,
                    &self.slp_rules,
                    &self.inline16,
                    &self.lens,
                    &mut self.first_pos,
                    &mut self.stack,
                    n,
                    mark,
                    bits,
                    out,
                );
            }
            WordMode::SlpEmitDfs => {
                emit_slp_forward::<false>(
                    &self.edges[..m as usize],
                    &self.extra,
                    &self.slp_rules,
                    &self.inline16,
                    &self.lens,
                    &mut self.first_pos,
                    &mut self.stack,
                    n,
                    mark,
                    bits,
                    out,
                );
            }
            _ => {
                if let Some(before) = predecessor {
                    emit_bidirectional(
                        &self.edges[..m as usize],
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        mark,
                        before,
                        bits,
                        out,
                    );
                } else {
                    emit_forward(
                        &self.edges[..m as usize],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        mark,
                        bits,
                        out,
                    );
                }
            }
        }
        let t8 = Instant::now();
        ph.a7 = t8.duration_since(t7).as_nanos() as u64;

        (ph, st)
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        m: u32,
        most: usize,
        word_mode: WordMode,
        step_mode: StepMode,
        ph: &mut PhaseNs,
        st: &mut BlockStats,
        t_setup_start: Instant,
    ) -> [usize; 256] {
        let nodes_words = L::STRIDE * most;
        assert!(self.edges.len() >= 4 * ns + nodes_words);
        let (hdr, arena) = self.edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0u32;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            e.node(s.width, literal(s.byte));
            widest = widest.max(s.width);
        }
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
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
            let idx = i as usize;
            let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
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
        st.open_nodes = e.nodes as usize - 1;
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;

        let t_steps_start = Instant::now();
        ph.a6b_setup = t_steps_start.duration_since(t_setup_start).as_nanos() as u64;

        let (mut remaining, mut steps) = (n, 0usize);
        let spirals = widest > CERTIFIED_WIDTH;
        loop {
            let done = if spirals {
                e.eliminate::<true>(&mut remaining, &mut steps)
            } else {
                match step_mode {
                    StepMode::NoJoinAblat => e.eliminate_no_join_ablat(&mut remaining, &mut steps),
                    StepMode::RegXY => e.eliminate_reg_xy(&mut remaining, &mut steps),
                    StepMode::HoistNext => e.eliminate_hoist_next(&mut remaining, &mut steps),
                    StepMode::BranchlessSide => {
                        e.eliminate_branchless_side(&mut remaining, &mut steps)
                    }
                    _ => e.eliminate::<false>(&mut remaining, &mut steps),
                }
            };
            if done {
                break;
            }
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(&mut self.edges, 4 * ns + top + RULE);
            e = Elim {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }

        let t_dict_start = Instant::now();
        ph.a6b_steps = t_dict_start.duration_since(t_steps_start).as_nanos() as u64;

        st.steps = steps;
        st.rules = (e.top - e.base) / RULE;
        st.arena_bytes = e.top * 4;

        if step_mode == StepMode::NoJoinAblat {
            return [0; 256];
        }

        e.assign_bottom(m);

        let (base, top, nodes) = (e.base, e.top, e.nodes);
        // If SLP-during-emission, copy compact rules out before edges[] is overwritten in A6c!
        if word_mode.is_slp_emit() {
            let nrules = (top - base) / RULE;
            self.slp_rules.clear();
            self.slp_rules.reserve(nrules);
            let rslice = &self.edges[4 * ns + base..4 * ns + top];
            for ch in rslice.chunks_exact(RULE) {
                self.slp_rules.push([ch[0], ch[1], ch[2], ch[3]]);
            }
        }

        // Dictionary cache setup
        let (pair_slots, content_slots) = match word_mode {
            WordMode::Pairs2048InlineOnlyDedup | WordMode::Pairs2048LongOnlyDedup => (2048, 4096),
            _ => (1024, 4096),
        };
        let pair_words = 2 * pair_slots;
        let cache_words = pair_words + content_slots;

        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let use_buf = header.len() < cache_words && tail.len() < cache_words;
        let (pairs, contents): (&mut [u32], &mut [u32]) = if !use_buf {
            let cache = if header.len() >= cache_words {
                header
            } else {
                tail
            };
            cache[..cache_words].fill(0);
            cache[..cache_words].split_at_mut(pair_words)
        } else if matches!(
            word_mode,
            WordMode::Pairs2048InlineOnlyDedup | WordMode::Pairs2048LongOnlyDedup
        ) {
            self.cache_buf.clear();
            self.cache_buf.resize(cache_words, 0);
            self.cache_buf.split_at_mut(pair_words)
        } else {
            (&mut [][..], &mut [][..])
        };

        let mut e = Elim::<L> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        if word_mode == WordMode::BidiPreShifted {
            begin_inline_with_back(
                if n >= 65_536 { 1536 } else { 0 },
                &mut self.inline16,
                &mut self.inline_back,
                &mut self.lens,
            );
        } else {
            begin_inline(
                if n >= 65_536 { 1536 } else { 0 },
                &mut self.inline16,
                &mut self.lens,
            );
        }

        let pair_shift = 32 - pair_slots.trailing_zeros();
        let mut cursors = [0usize; 256];
        let mut id = L::next(e.a, 0, 0);
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256 {
            cursors[a] = self.bands.len();
            let end = k + thin[a];
            while k < end {
                let node_width = e.width(id);
                let node_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                let dst = e.bottom(id) + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    dict_lookup_insert(
                        word_mode,
                        &mut e,
                        node_word,
                        a as u8,
                        pairs,
                        pair_shift,
                        contents,
                        &mut self.inline16,
                        &mut self.inline_back,
                        &mut self.lens,
                        &mut self.spelling,
                        &mut self.stack,
                        &mut self.undo,
                    )
                };
                self.bands.push(Band::new(k + width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        let t_dict_end = Instant::now();
        ph.a6b_dict = t_dict_end.duration_since(t_dict_start).as_nanos() as u64;
        cursors
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_typed_narrow(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        m: u32,
        step_mode: StepMode,
        ph: &mut PhaseNs,
        st: &mut BlockStats,
        t_setup_start: Instant,
    ) -> [usize; 256] {
        let (strips_raw, rest) = self.edges.split_at(3 * ns);
        let image_order = &rest[..ns];
        let widest = tn_build_nodes(strips_raw, image_order, ns, n, &mut self.typed.nodes);
        if widest > CERTIFIED_WIDTH {
            // Fall back to production spiral path if a block exceeds width certificate
            let most = Self::nodes_most(ns);
            return self.induced_with::<Narrow>(
                ns,
                n,
                thin,
                m,
                most,
                WordMode::Prod,
                StepMode::Prod,
                ph,
                st,
                t_setup_start,
            );
        }
        st.open_nodes = self.typed.nodes.len() - 1;
        let t_steps_start = Instant::now();
        ph.a6b_setup = t_steps_start.duration_since(t_setup_start).as_nanos() as u64;

        let is_8b = matches!(
            step_mode,
            StepMode::Compact8bSlp | StepMode::Compact8bSlpHoist
        );
        let steps = match step_mode {
            StepMode::TypedNarrow => {
                tn_eliminate_typed(&mut self.typed.nodes, &mut self.typed.rules, n)
            }
            StepMode::TypedPrefetchTouch => {
                tn_eliminate_typed_prefetch_touch(&mut self.typed.nodes, &mut self.typed.rules, n)
            }
            StepMode::Compact8bSlp => {
                tn_eliminate_8b_slp(&mut self.typed.nodes, &mut self.typed.rules8, n, false)
            }
            StepMode::Compact8bSlpHoist => {
                tn_eliminate_8b_slp(&mut self.typed.nodes, &mut self.typed.rules8, n, true)
            }
            _ => unreachable!(),
        };
        let t_dict_start = Instant::now();
        ph.a6b_steps = t_dict_start.duration_since(t_steps_start).as_nanos() as u64;

        st.steps = steps;
        st.rules = if is_8b {
            self.typed.rules8.len()
        } else {
            self.typed.rules.len()
        };
        st.arena_bytes = self.typed.nodes.len() * 16 + st.rules * if is_8b { 8 } else { 16 };

        tn_assign_bottom(&mut self.typed.nodes, m);

        const PAIR_WORDS: usize = 2 * 1024;
        const CACHE_WORDS: usize = PAIR_WORDS + 4096;
        if self.cache_buf.len() < CACHE_WORDS {
            self.cache_buf.resize(CACHE_WORDS, 0);
        }
        self.cache_buf[..CACHE_WORDS].fill(0);
        let (pairs, contents) = self.cache_buf[..CACHE_WORDS].split_at_mut(PAIR_WORDS);

        begin_inline(
            if n >= 65_536 { 1536 } else { 0 },
            &mut self.inline16,
            &mut self.lens,
        );

        if is_8b {
            let nr = self.typed.rules8.len();
            self.typed.memo_start.clear();
            self.typed.memo_start.resize(nr, 0);
            self.typed.memo_len.clear();
            self.typed.memo_len.resize(nr, 0);
        }

        let mut cursors = [0usize; 256];
        let mut id = tn_next(&self.typed.nodes, 0, 0);
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256 {
            cursors[a] = self.bands.len();
            let end = k + thin[a];
            while k < end {
                let node = self.typed.nodes[id as usize];
                let node_width = node[2];
                let node_word = node[3];
                let width = (end - k).min(node_width - in_node);
                let dst = node[1] + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    let rid = node_word & ID_MASK;
                    let key = (rid << 8) | (a as u32);
                    let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
                    if pairs[slot] == key {
                        pairs[slot + 1]
                    } else {
                        let start = self.spelling.len();
                        let (memoized, entry, len) = if is_8b {
                            let memoized = node_word >> ID_BITS >= INLINE as u32
                                && rid > 256
                                && self.typed.memo_len[(rid - 257) as usize] != 0;
                            let (en, ln) = expand_8b_word(
                                &self.typed.rules8,
                                &mut self.typed.memo_start,
                                &mut self.typed.memo_len,
                                node_word,
                                a as u8,
                                &mut self.stack,
                                &mut self.spelling,
                            );
                            (memoized, en, ln)
                        } else {
                            let memoized = node_word >> ID_BITS >= INLINE as u32
                                && self.typed.rules[(rid - 257) as usize][3] != u32::MAX;
                            let height = (node_word >> ID_BITS) as usize;
                            let (en, ln) = if height < INLINE {
                                let mut bytes =
                                    tn_short_payload(&self.typed.rules, rid).to_le_bytes();
                                bytes[height] = a as u8;
                                (bytes, (height + 1) as u8)
                            } else {
                                let sp_start = self.spelling.len() as u32;
                                self.stack.clear();
                                self.stack.push(rid);
                                while let Some(cur) = self.stack.pop() {
                                    if cur <= 256 {
                                        self.spelling.push((cur - 1) as u8);
                                    } else {
                                        let r = &mut self.typed.rules[(cur - 257) as usize];
                                        let tag = r[3];
                                        if tag >> 28 == 0 {
                                            let bytes = (u128::from(r[0])
                                                | (u128::from(r[1]) << 32)
                                                | (u128::from(r[2]) << 64)
                                                | (u128::from(r[3] & 0x00ff_ffff) << 96))
                                                .to_le_bytes();
                                            self.spelling
                                                .extend_from_slice(&bytes[..(tag >> 24) as usize]);
                                        } else if tag == u32::MAX {
                                            r[3] = 0x8000_0000 | self.spelling.len() as u32;
                                            self.stack.push(r[1]);
                                            self.stack.push(r[0]);
                                        } else {
                                            let ms = (tag & 0x7fff_ffff) as usize;
                                            self.spelling
                                                .extend_from_within(ms..ms + r[2] as usize);
                                        }
                                    }
                                }
                                self.spelling.push(a as u8);
                                (long_slot(sp_start, self.spelling.len() as u32), LONG)
                            };
                            (memoized, en, ln)
                        };
                        let bytes = if len == LONG {
                            &self.spelling[start..]
                        } else {
                            &entry[..len as usize]
                        };
                        let index =
                            spelling_hash(&entry, len, bytes) as usize & (contents.len() - 1);
                        let old = contents[index] as usize;
                        let same = old != 0
                            && self.lens[old] == len
                            && if len == LONG {
                                long_word(&self.inline16[old], &self.spelling) == bytes
                            } else {
                                self.inline16[old] == entry
                            };
                        let c = if same {
                            if len == LONG {
                                if memoized {
                                    self.spelling.truncate(start);
                                } else {
                                    self.spelling.pop();
                                }
                            }
                            old as u32
                        } else {
                            let c = self.lens.len() as u32;
                            self.inline16.push(entry);
                            self.lens.push(len);
                            contents[index] = c;
                            c
                        };
                        pairs[slot] = key;
                        pairs[slot + 1] = c;
                        c
                    }
                };
                self.bands.push(Band::new(k + width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = tn_next(&self.typed.nodes, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        let t_dict_end = Instant::now();
        ph.a6b_dict = t_dict_end.duration_since(t_dict_start).as_nanos() as u64;
        cursors
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_inline7_node20(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        m: u32,
        ph: &mut PhaseNs,
        st: &mut BlockStats,
        t_setup_start: Instant,
    ) -> [usize; 256] {
        let (strips_raw, rest) = self.edges.split_at(3 * ns);
        let image_order = &rest[..ns];
        let widest = n20_build_nodes(strips_raw, image_order, ns, n, &mut self.typed.nodes20);
        if widest > CERTIFIED_WIDTH {
            let most = Self::nodes_most(ns);
            return self.induced_with::<Narrow>(
                ns,
                n,
                thin,
                m,
                most,
                WordMode::Prod,
                StepMode::Prod,
                ph,
                st,
                t_setup_start,
            );
        }
        st.open_nodes = self.typed.nodes20.len() - 1;
        let t_steps_start = Instant::now();
        ph.a6b_setup = t_steps_start.duration_since(t_setup_start).as_nanos() as u64;

        let steps = n20_eliminate(&mut self.typed.nodes20, &mut self.typed.rules, n);
        let t_dict_start = Instant::now();
        ph.a6b_steps = t_dict_start.duration_since(t_steps_start).as_nanos() as u64;

        st.steps = steps;
        st.rules = self.typed.rules.len();
        st.arena_bytes = self.typed.nodes20.len() * 20 + st.rules * 16;

        n20_assign_bottom(&mut self.typed.nodes20, m);

        const PAIR_WORDS: usize = 2 * 1024;
        const CACHE_WORDS: usize = PAIR_WORDS + 4096;
        if self.cache_buf.len() < CACHE_WORDS {
            self.cache_buf.resize(CACHE_WORDS, 0);
        }
        self.cache_buf[..CACHE_WORDS].fill(0);
        let (pairs, contents) = self.cache_buf[..CACHE_WORDS].split_at_mut(PAIR_WORDS);

        begin_inline(
            if n >= 65_536 { 1536 } else { 0 },
            &mut self.inline16,
            &mut self.lens,
        );

        let mut cursors = [0usize; 256];
        let mut id = self.typed.nodes20[0][0] >> 16;
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256 {
            cursors[a] = self.bands.len();
            let end = k + thin[a];
            while k < end {
                let node = self.typed.nodes20[id as usize];
                let node_width = node[2];
                let w64 = n20_word(node);
                let width = (end - k).min(node_width - in_node);
                let dst = node[1] + in_node;
                let code = if w64 == 0 {
                    a as u32
                } else {
                    let tag = (w64 >> 56) as usize;
                    if tag <= 7 {
                        let mut entry = [0u8; INLINE];
                        let payload = (w64 & N20_PAYLOAD_MASK).to_le_bytes();
                        entry[..8].copy_from_slice(&payload);
                        entry[tag] = a as u8;
                        let len = (tag + 1) as u8;
                        let bytes = &entry[..len as usize];
                        let index =
                            spelling_hash(&entry, len, bytes) as usize & (contents.len() - 1);
                        let old = contents[index] as usize;
                        if old != 0 && self.lens[old] == len && self.inline16[old] == entry {
                            old as u32
                        } else {
                            let c = self.lens.len() as u32;
                            self.inline16.push(entry);
                            self.lens.push(len);
                            contents[index] = c;
                            c
                        }
                    } else {
                        let node_word = w64 as u32;
                        let rid = node_word & ID_MASK;
                        let key = (rid << 8) | (a as u32);
                        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
                        if pairs[slot] == key {
                            pairs[slot + 1]
                        } else {
                            let start = self.spelling.len();
                            let height = (node_word >> ID_BITS) as usize;
                            let memoized = height >= INLINE
                                && self.typed.rules[(rid - 257) as usize][3] != u32::MAX;
                            let (entry, len) = if height < INLINE {
                                let mut bytes =
                                    tn_short_payload(&self.typed.rules, rid).to_le_bytes();
                                bytes[height] = a as u8;
                                (bytes, (height + 1) as u8)
                            } else {
                                let sp_start = self.spelling.len() as u32;
                                self.stack.clear();
                                self.stack.push(rid);
                                while let Some(cur) = self.stack.pop() {
                                    if cur <= 256 {
                                        self.spelling.push((cur - 1) as u8);
                                    } else {
                                        let r = &mut self.typed.rules[(cur - 257) as usize];
                                        let rtag = r[3];
                                        if rtag >> 28 == 0 {
                                            let bytes = (u128::from(r[0])
                                                | (u128::from(r[1]) << 32)
                                                | (u128::from(r[2]) << 64)
                                                | (u128::from(r[3] & 0x00ff_ffff) << 96))
                                                .to_le_bytes();
                                            self.spelling
                                                .extend_from_slice(&bytes[..(rtag >> 24) as usize]);
                                        } else if rtag == u32::MAX {
                                            r[3] = 0x8000_0000 | self.spelling.len() as u32;
                                            self.stack.push(r[1]);
                                            self.stack.push(r[0]);
                                        } else {
                                            let ms = (rtag & 0x7fff_ffff) as usize;
                                            self.spelling
                                                .extend_from_within(ms..ms + r[2] as usize);
                                        }
                                    }
                                }
                                self.spelling.push(a as u8);
                                (long_slot(sp_start, self.spelling.len() as u32), LONG)
                            };
                            let bytes = if len == LONG {
                                &self.spelling[start..]
                            } else {
                                &entry[..len as usize]
                            };
                            let index =
                                spelling_hash(&entry, len, bytes) as usize & (contents.len() - 1);
                            let old = contents[index] as usize;
                            let same = old != 0
                                && self.lens[old] == len
                                && if len == LONG {
                                    long_word(&self.inline16[old], &self.spelling) == bytes
                                } else {
                                    self.inline16[old] == entry
                                };
                            let c = if same {
                                if len == LONG {
                                    if memoized {
                                        self.spelling.truncate(start);
                                    } else {
                                        self.spelling.pop();
                                    }
                                }
                                old as u32
                            } else {
                                let c = self.lens.len() as u32;
                                self.inline16.push(entry);
                                self.lens.push(len);
                                contents[index] = c;
                                c
                            };
                            pairs[slot] = key;
                            pairs[slot + 1] = c;
                            c
                        }
                    }
                };
                self.bands.push(Band::new(k + width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = self.typed.nodes20[id as usize][0] >> 16;
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        let t_dict_end = Instant::now();
        ph.a6b_dict = t_dict_end.duration_since(t_dict_start).as_nanos() as u64;
        cursors
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn dict_lookup_insert<L: Links>(
    mode: WordMode,
    e: &mut Elim<L>,
    handle: u32,
    byte: u8,
    pairs: &mut [u32],
    pair_shift: u32,
    contents: &mut [u32],
    inline16: &mut Vec<[u8; INLINE]>,
    inline_back: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    undo: &mut Vec<usize>,
) -> u32 {
    let id = handle & ID_MASK;
    let key = (id << 8) | u32::from(byte);
    let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> pair_shift) as usize;

    if mode != WordMode::NoDedup && !pairs.is_empty() && pairs[slot] == key {
        return pairs[slot + 1];
    }

    if mode.is_slp_emit() {
        let height = (handle >> ID_BITS) as usize;
        let (entry, len) = if height < INLINE {
            let mut bytes = e.short_payload(id).to_le_bytes();
            bytes[height] = byte;
            (bytes, (height + 1) as u8)
        } else {
            // Do NOT expand into `spelling`; store (rule_id, tail_byte) for direct SLP emit!
            (long_slot(id, u32::from(byte)), LONG)
        };
        let code = if len != LONG && !contents.is_empty() {
            let bytes = &entry[..len as usize];
            let index = spelling_hash(&entry, len, bytes) as usize & (contents.len() - 1);
            let old = contents[index] as usize;
            if old != 0 && lens[old] == len && inline16[old] == entry {
                old as u32
            } else {
                let c = lens.len() as u32;
                inline16.push(entry);
                lens.push(len);
                contents[index] = c;
                c
            }
        } else {
            let c = lens.len() as u32;
            inline16.push(entry);
            lens.push(len);
            c
        };
        if !pairs.is_empty() {
            pairs[slot] = key;
            pairs[slot + 1] = code;
        }
        return code;
    }

    let start = spelling.len();
    let memoized = handle >> ID_BITS >= INLINE as u32 && e.a[e.rule(id) + 3] != u32::MAX;
    let (entry, len) = match mode {
        WordMode::NoMemoExpand => e.inline_word_no_memo(handle, byte, stack, spelling),
        WordMode::TrimMemoSlack => e.inline_word_track_undo(handle, byte, stack, undo, spelling),
        _ => e.inline_word(handle, byte, stack, spelling),
    };

    let do_contents = match mode {
        WordMode::NoContentsDedup | WordMode::NoDedup => false,
        WordMode::Pairs2048InlineOnlyDedup => len != LONG && !contents.is_empty(),
        WordMode::Pairs2048LongOnlyDedup => len == LONG && !contents.is_empty(),
        _ => !contents.is_empty(),
    };

    let code = if !do_contents {
        let c = lens.len() as u32;
        inline16.push(entry);
        if mode == WordMode::BidiPreShifted {
            inline_back.push(shift_back(entry, len));
        }
        lens.push(len);
        c
    } else {
        let bytes = if len == LONG {
            &spelling[start..]
        } else {
            &entry[..len as usize]
        };
        let index = spelling_hash(&entry, len, bytes) as usize & (contents.len() - 1);
        let old = contents[index] as usize;
        let same = old != 0
            && lens[old] == len
            && if len == LONG {
                long_word(&inline16[old], spelling) == bytes
            } else {
                inline16[old] == entry
            };
        if same {
            if len == LONG {
                if mode == WordMode::TrimMemoSlack {
                    // Undo all child memos written during this expansion and redirect root to old!
                    for &r in undo.iter() {
                        e.a[r + 3] = u32::MAX;
                    }
                    let (old_start, _) = long_bounds(&inline16[old]);
                    e.a[e.rule(id) + 3] = 0x8000_0000 | old_start as u32;
                    spelling.truncate(start);
                } else if memoized || mode == WordMode::NoMemoExpand {
                    spelling.truncate(start);
                } else {
                    spelling.pop();
                }
            }
            old as u32
        } else {
            let c = lens.len() as u32;
            inline16.push(entry);
            if mode == WordMode::BidiPreShifted {
                inline_back.push(shift_back(entry, len));
            }
            lens.push(len);
            contents[index] = c;
            c
        }
    };

    if mode != WordMode::NoDedup && !pairs.is_empty() {
        pairs[slot] = key;
        pairs[slot + 1] = code;
    }
    code
}

// ---------------------------------------------------------------------------
// Idea 3: Standalone step diagnostics & 2-way / 4-way interleaving
// ---------------------------------------------------------------------------

struct PreparedElim {
    nodes_init: Vec<[u32; 4]>,
    n: u32,
    expected_steps: usize,
    arena_est_bytes: usize,
}

#[derive(Default)]
struct StepDiag {
    total_steps: usize,
    eq_xy: usize,
    eq_w: usize,
    lt_w: usize,
    gt_w: usize,
    aw_le_bw_2bit_miss: usize,
    join_calls: usize,
    join_zero_arg: usize,
    join_len_le_4: usize,
    join_len_le_7: usize,
    join_len_lt_16: usize,
    join_len_ge_16: usize,
    join_saturated: usize,
    blocks_gt_128k: usize,
    blocks_gt_256k: usize,
    arena_kib: Vec<f64>,
}

fn prepare_block_elim(b: &BlockInput, diag: &mut StepDiag) -> Option<PreparedElim> {
    let mut edges = Vec::new();
    let mut spill = Spill::default();
    let mut out = vec![0u8; b.n + INLINE];
    let mut pass = Pass {
        queues: Queues::new(&mut out, &mut spill, Cap::NONE.m),
        edges: &mut edges,
        ns: 0,
        cap_strips: Cap::NONE.strips,
        f: First::default(),
    };
    maximal(&b.records, |c, pos, width| {
        pass.take::<false>(c, pos, width, b.orig_ptr, PROD_LIMIT);
    });
    let ns = pass.ns;
    let f = pass.f;
    let most = OpenKernel::nodes_most(ns);
    if most > NARROW_NODES {
        return None;
    }
    let all: [u32; 256] = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
    let first = starts(all);
    let mut write = starts(f.thick);
    if edges.len() < 4 * ns {
        edges.resize(4 * ns, 0);
    }
    let (strips_raw, rest) = edges.split_at_mut(3 * ns);
    let image_order = &mut rest[..ns];
    for i in 0..ns {
        let sw = &mut strips_raw[3 * i..3 * i + 3];
        let a = usize::from((sw[2] & 0xFF) as u8);
        sw[1] += first[a];
        image_order[write[a] as usize] = i as u32;
        write[a] += 1;
    }
    let mut nodes_init = Vec::new();
    let widest = tn_build_nodes(strips_raw, image_order, ns, b.n as u32, &mut nodes_init);
    if widest > CERTIFIED_WIDTH {
        return None;
    }

    // Run once to gather exact branch & rule statistics
    let mut nodes = nodes_init.clone();
    let mut rules: Vec<[u32; 4]> = Vec::new();
    let mut left = b.n as u32;
    let mut steps = 0usize;
    let mut pred_2bit: u8 = 2; // weak taken
    while left != 0 {
        let x = nodes[0][0] & 0xFFFF;
        let y = nodes[0][1] & 0xFFFF;
        let nx = nodes[x as usize];
        let ny = nodes[y as usize];
        let aw = nx[2];
        let bw = ny[2];
        let w = aw.min(bw);
        left -= w;
        steps += 1;
        diag.total_steps += 1;
        if x == y {
            diag.eq_xy += 1;
            let xp = nx[0] & 0xFFFF;
            let yp = ny[1] & 0xFFFF;
            tn_set_next(&mut nodes, xp, 0, 0);
            tn_set_prev(&mut nodes, 0, 0, xp);
            tn_set_next(&mut nodes, yp, 1, 0);
            tn_set_prev(&mut nodes, 0, 1, yp);
            continue;
        }
        let taken_le = aw <= bw;
        let predicted = pred_2bit >= 2;
        if taken_le != predicted {
            diag.aw_le_bw_2bit_miss += 1;
        }
        pred_2bit = if taken_le {
            (pred_2bit + 1).min(3)
        } else {
            pred_2bit.saturating_sub(1)
        };
        if aw < bw {
            diag.lt_w += 1;
        } else if aw > bw {
            diag.gt_w += 1;
        } else {
            diag.eq_w += 1;
        }

        diag.join_calls += 1;
        let (lw, rw) = (nx[3], ny[3]);
        if lw == 0 || rw == 0 {
            diag.join_zero_arg += 1;
        } else {
            let ll = lw >> ID_BITS;
            let rl = rw >> ID_BITS;
            let len = if ll < SATURATED && rl < SATURATED {
                ll + rl
            } else {
                diag.join_saturated += 1;
                tn_len(&rules, lw & ID_MASK) + tn_len(&rules, rw & ID_MASK)
            };
            if len <= 4 {
                diag.join_len_le_4 += 1;
            }
            if len <= 7 {
                diag.join_len_le_7 += 1;
            }
            if len < INLINE as u32 {
                diag.join_len_lt_16 += 1;
            } else {
                diag.join_len_ge_16 += 1;
            }
        }
        let word = tn_join(&mut rules, lw, rw);
        if aw <= bw {
            let xp = nx[0] & 0xFFFF;
            tn_set_next(&mut nodes, xp, 0, 0);
            tn_set_prev(&mut nodes, 0, 0, xp);
            tn_insert(&mut nodes, y, x, 0);
            nodes[x as usize][3] = word;
            if aw == bw {
                tn_cut(&mut nodes, y, 0);
                let yp = ny[1] & 0xFFFF;
                tn_set_next(&mut nodes, yp, 1, 0);
                tn_set_prev(&mut nodes, 0, 1, yp);
            } else {
                nodes[y as usize][2] = bw - w;
            }
        } else {
            let yp = ny[1] & 0xFFFF;
            tn_set_next(&mut nodes, yp, 1, 0);
            tn_set_prev(&mut nodes, 0, 1, yp);
            tn_insert(&mut nodes, x, y, 1);
            nodes[y as usize][3] = word;
            nodes[x as usize][2] = aw - w;
        }
    }
    let arena_bytes = nodes_init.len() * 16 + rules.len() * 16;
    let kib = arena_bytes as f64 / 1024.0;
    diag.arena_kib.push(kib);
    if arena_bytes > 128 * 1024 {
        diag.blocks_gt_128k += 1;
    }
    if arena_bytes > 256 * 1024 {
        diag.blocks_gt_256k += 1;
    }
    Some(PreparedElim {
        nodes_init,
        n: b.n as u32,
        expected_steps: steps,
        arena_est_bytes: arena_bytes,
    })
}

/// Run K independent blocks' eliminations interleaved 1 step at a time on ONE core!
/// Directly tests whether single-block elimination is bottlenecked by dependent-load latency
/// vs L2 capacity (256 KiB).
fn run_interleaved<const K: usize>(preps: &[&PreparedElim], use_8b: bool) -> (u64, usize) {
    let mut ns_total = 0u64;
    let mut steps_total = 0usize;
    let mut nodes_arr: [Vec<[u32; 4]>; K] = std::array::from_fn(|_| Vec::new());
    let mut rules16_arr: [Vec<[u32; 4]>; K] = std::array::from_fn(|_| Vec::new());
    let mut rules8_arr: [Vec<[u32; 2]>; K] = std::array::from_fn(|_| Vec::new());

    let mut idx = 0usize;
    while idx < preps.len() {
        let active = (preps.len() - idx).min(K);
        let mut rem = [0u32; K];
        for lane in 0..active {
            nodes_arr[lane].clear();
            nodes_arr[lane].extend_from_slice(&preps[idx + lane].nodes_init);
            rules16_arr[lane].clear();
            rules8_arr[lane].clear();
            rem[lane] = preps[idx + lane].n;
        }

        let t0 = Instant::now();
        let mut alive = active;
        while alive > 0 {
            for lane in 0..active {
                if rem[lane] == 0 {
                    continue;
                }
                let nodes = &mut nodes_arr[lane];
                let x = nodes[0][0] & 0xFFFF;
                let y = nodes[0][1] & 0xFFFF;
                let nx = nodes[x as usize];
                let ny = nodes[y as usize];
                let aw = nx[2];
                let bw = ny[2];
                let w = aw.min(bw);
                rem[lane] -= w;
                steps_total += 1;
                if rem[lane] == 0 {
                    alive -= 1;
                }
                if x == y {
                    let xp = nx[0] & 0xFFFF;
                    let yp = ny[1] & 0xFFFF;
                    tn_set_next(nodes, xp, 0, 0);
                    tn_set_prev(nodes, 0, 0, xp);
                    tn_set_next(nodes, yp, 1, 0);
                    tn_set_prev(nodes, 0, 1, yp);
                    continue;
                }
                let word = if use_8b {
                    tn_join_8b(&mut rules8_arr[lane], nx[3], ny[3])
                } else {
                    tn_join(&mut rules16_arr[lane], nx[3], ny[3])
                };
                if aw <= bw {
                    let xp = nx[0] & 0xFFFF;
                    tn_set_next(nodes, xp, 0, 0);
                    tn_set_prev(nodes, 0, 0, xp);
                    tn_insert(nodes, y, x, 0);
                    nodes[x as usize][3] = word;
                    if aw == bw {
                        tn_cut(nodes, y, 0);
                        let yp = ny[1] & 0xFFFF;
                        tn_set_next(nodes, yp, 1, 0);
                        tn_set_prev(nodes, 0, 1, yp);
                    } else {
                        nodes[y as usize][2] = bw - w;
                    }
                } else {
                    let yp = ny[1] & 0xFFFF;
                    tn_set_next(nodes, yp, 1, 0);
                    tn_set_prev(nodes, 0, 1, yp);
                    tn_insert(nodes, x, y, 1);
                    nodes[y as usize][3] = word;
                    nodes[x as usize][2] = aw - w;
                }
            }
        }
        ns_total += t0.elapsed().as_nanos() as u64;
        for lane in 0..active {
            black_box(&nodes_arr[lane]);
            if use_8b {
                black_box(&rules8_arr[lane]);
            } else {
                black_box(&rules16_arr[lane]);
            }
        }
        idx += active;
    }
    (ns_total, steps_total)
}

// ---------------------------------------------------------------------------
// Corpus loading & summary helpers
// ---------------------------------------------------------------------------

struct BlockInput {
    index: usize,
    n: usize,
    orig_ptr: u32,
    records: Vec<u16>,
    walked: Vec<u8>,
}

fn load_blocks(path: &str, forced: bool, x4: bool) -> (Vec<BlockInput>, usize, usize) {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut tt = TVector::new();
    let mut blocks = Vec::new();
    let mut total_blocks = 0usize;
    let mut total_rows = 0usize;

    // Select the EXACT production-admitted blocks (matching Cap::dense at LIMIT=16!)
    let mut probe_kernel = OpenKernel::default();
    let mut probe_out = Vec::new();
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        let n = ll.len();
        total_blocks += 1;
        total_rows += n;
        let recs = records_of(&ll);
        if let Some(cap) = open_cap(recs.len(), n, b.orig_ptr, forced) {
            // Check Cap::dense at production LIMIT=16 so the 233-block set matches production exactly
            let admitted = if cap == Cap::NONE {
                true
            } else {
                probe_out.resize(n + INLINE, 0);
                let mut pass = Pass {
                    queues: Queues::new(&mut probe_out, &mut probe_kernel.spill, cap.m),
                    edges: &mut probe_kernel.edges,
                    ns: 0,
                    cap_strips: cap.strips,
                    f: First::default(),
                };
                maximal(&recs, |c, pos, width| {
                    pass.take::<false>(c, pos, width, b.orig_ptr, PROD_LIMIT);
                });
                !pass.queues.over && pass.f.m <= cap.m && pass.ns <= cap.strips
            };
            if admitted {
                let mut walked = Vec::new();
                build(&ll, &counts, &mut tt);
                walk(&tt, b.orig_ptr, &mut walked);
                blocks.push(BlockInput {
                    index: total_blocks - 1,
                    n,
                    orig_ptr: b.orig_ptr,
                    records: recs,
                    walked,
                });
            }
        }
        syms = b.syms;
    }
    if x4 {
        let base_len = blocks.len();
        for rep in 1..4 {
            for i in 0..base_len {
                blocks.push(BlockInput {
                    index: rep * total_blocks + blocks[i].index,
                    n: blocks[i].n,
                    orig_ptr: blocks[i].orig_ptr,
                    records: blocks[i].records.clone(),
                    walked: blocks[i].walked.clone(),
                });
            }
        }
        total_blocks *= 4;
        total_rows *= 4;
    }
    (blocks, total_blocks, total_rows)
}

fn median_min_max(xs: &[f64]) -> (f64, f64, f64) {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let med = if v.len() % 2 == 1 {
        v[v.len() / 2]
    } else {
        0.5 * (v[v.len() / 2 - 1] + v[v.len() / 2])
    };
    (med, v[0], v[v.len() - 1])
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn i9_anchored_ms(ph: &PhaseNs, base: &PhaseNs) -> f64 {
    I9_A6A_MS * (ph.a6a as f64 / base.a6a as f64)
        + I9_A6B_MS * (ph.a6b() as f64 / base.a6b() as f64)
        + I9_A6C_MS * (ph.a6c as f64 / base.a6c as f64)
        + I9_A7_MS * (ph.a7 as f64 / base.a7 as f64)
}

fn i9_prop_ms(ph: &PhaseNs, base: &PhaseNs) -> f64 {
    I9_CE_MS * (ph.total() as f64 / base.total() as f64)
}

// ---------------------------------------------------------------------------
// Command 1: LIMIT sweep & adaptive policies
// ---------------------------------------------------------------------------

struct LimitArm {
    name: &'static str,
    policy: LimitPolicy,
    dispatch: DispatchMode,
}

fn cmd_limit(blocks: &[BlockInput], repeats: usize) {
    let total_rows: usize = blocks.iter().map(|b| b.n).sum();
    println!(
        "\n=== IDEA 1: LIMIT SWEEP & ADAPTIVE RULES (blocks={}, rows={}, repeats={}) ===",
        blocks.len(),
        total_rows,
        repeats
    );

    let arms = [
        LimitArm {
            name: "limit_2",
            policy: LimitPolicy::Fixed(2),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_3",
            policy: LimitPolicy::Fixed(3),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_4",
            policy: LimitPolicy::Fixed(4),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_6",
            policy: LimitPolicy::Fixed(6),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_8",
            policy: LimitPolicy::Fixed(8),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_10",
            policy: LimitPolicy::Fixed(10),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_12",
            policy: LimitPolicy::Fixed(12),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_16_prod",
            policy: LimitPolicy::Fixed(16),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_24",
            policy: LimitPolicy::Fixed(24),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_32",
            policy: LimitPolicy::Fixed(32),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_64",
            policy: LimitPolicy::Fixed(64),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_32_fast64q",
            policy: LimitPolicy::FixedFast64(32),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "limit_64_fast64q",
            policy: LimitPolicy::FixedFast64(64),
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "adapt_density",
            policy: LimitPolicy::PerBlockDensity,
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "adapt_hist_r12",
            policy: LimitPolicy::PerBlockHist {
                strip_to_row_ratio: 12,
            },
            dispatch: DispatchMode::Prod,
        },
        LimitArm {
            name: "adapt_per_byte",
            policy: LimitPolicy::PerByteAdaptive,
            dispatch: DispatchMode::Prod,
        },
        // Fixed forward-dispatch sweep (disentangles m<65536 bidi->forward switch!)
        LimitArm {
            name: "fwd_limit_4",
            policy: LimitPolicy::Fixed(4),
            dispatch: DispatchMode::ForceForward,
        },
        LimitArm {
            name: "fwd_limit_6",
            policy: LimitPolicy::Fixed(6),
            dispatch: DispatchMode::ForceForward,
        },
        LimitArm {
            name: "fwd_limit_8",
            policy: LimitPolicy::Fixed(8),
            dispatch: DispatchMode::ForceForward,
        },
        LimitArm {
            name: "fwd_limit_12",
            policy: LimitPolicy::Fixed(12),
            dispatch: DispatchMode::ForceForward,
        },
        LimitArm {
            name: "fwd_limit_16",
            policy: LimitPolicy::Fixed(16),
            dispatch: DispatchMode::ForceForward,
        },
        LimitArm {
            name: "fwd_limit_24",
            policy: LimitPolicy::Fixed(24),
            dispatch: DispatchMode::ForceForward,
        },
    ];

    let mut kernel = OpenKernel::default();
    let mut out = Vec::new();
    let mut stats_by_arm = Vec::with_capacity(arms.len());

    // Warm-up + exact byte-identity verification against bwt::walk on EVERY block!
    for arm in &arms {
        let mut tot = BlockStats::default();
        for b in blocks {
            let (_ph, st) = kernel.run_block(
                &b.records,
                b.n,
                b.orig_ptr,
                arm.policy,
                WordMode::Prod,
                StepMode::Prod,
                arm.dispatch,
                &mut out,
                true,
            );
            assert_eq!(
                out, b.walked,
                "BYTE MISMATCH arm={} block={}",
                arm.name, b.index
            );
            tot.add(st);
        }
        stats_by_arm.push(tot);
    }
    println!(
        "# Exactness verified: 100% byte-identical to bwt::walk on all {} blocks x {} arms",
        blocks.len(),
        arms.len()
    );

    // Interleaved repeats (and track per-block best fixed LIMIT for prod and forward oracles)
    let mut runs: Vec<Vec<PhaseNs>> = vec![Vec::with_capacity(repeats); arms.len()];
    let fixed_oracle_indices = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let fwd_oracle_indices = [16usize, 17, 18, 19, 20, 21];
    let mut per_block_med_ns: Vec<Vec<u64>> =
        vec![vec![0; blocks.len()]; fixed_oracle_indices.len()];
    let mut per_block_fwd_ns: Vec<Vec<u64>> = vec![vec![0; blocks.len()]; fwd_oracle_indices.len()];

    for r in 0..repeats {
        for (ai, arm) in arms.iter().enumerate() {
            let mut sum = PhaseNs::default();
            let oracle_pos = fixed_oracle_indices.iter().position(|&x| x == ai);
            let fwd_pos = fwd_oracle_indices.iter().position(|&x| x == ai);
            for (bi, b) in blocks.iter().enumerate() {
                let (ph, _) = kernel.run_block(
                    &b.records,
                    b.n,
                    b.orig_ptr,
                    arm.policy,
                    WordMode::Prod,
                    StepMode::Prod,
                    arm.dispatch,
                    &mut out,
                    false,
                );
                black_box(&out);
                sum.add(ph);
                if let Some(op) = oracle_pos {
                    per_block_med_ns[op][bi] += ph.total();
                }
                if let Some(fp) = fwd_pos {
                    per_block_fwd_ns[fp][bi] += ph.total();
                }
            }
            runs[ai].push(sum);
        }
        eprintln!("  [limit] repeat {}/{} done", r + 1, repeats);
    }

    // Print structural table
    println!("\n--- 1A. Structural counts across LIMITs ---");
    println!(
        "{:<18} {:>10} {:>9} {:>10} {:>10} {:>8} {:>8} {:>8} {:>7} {:>7} {:>5} {:>5} {:>5}",
        "arm",
        "m(codes)",
        "strips",
        "steps",
        "bands",
        "dict/blk",
        "sp_KiB/b",
        "lit%",
        "long%",
        "m/n%",
        "wideA",
        "wideE",
        "bidi"
    );
    for (ai, arm) in arms.iter().enumerate() {
        let s = &stats_by_arm[ai];
        let nb = blocks.len() as f64;
        println!(
            "{:<18} {:>10} {:>9} {:>10} {:>10} {:>8.0} {:>8.1} {:>7.2}% {:>6.2}% {:>6.2}% {:>5} {:>5} {:>5}",
            arm.name,
            s.m,
            s.ns,
            s.steps,
            s.bands,
            s.dict_entries as f64 / nb,
            s.spelling_bytes as f64 / (1024.0 * nb),
            100.0 * s.literal_codes as f64 / s.m as f64,
            100.0 * s.long_codes as f64 / s.m as f64,
            100.0 * s.m as f64 / s.n as f64,
            s.wide_arena,
            s.wide_edges,
            s.bidi_blocks,
        );
    }

    // Find baseline (limit_16_prod = index 7)
    let base_idx = arms.iter().position(|a| a.name == "limit_16_prod").unwrap();
    let base_tot_ns: Vec<f64> = runs[base_idx].iter().map(|p| p.total() as f64).collect();
    let base_med_run = {
        let mut idx_sorted: Vec<usize> = (0..repeats).collect();
        idx_sorted.sort_by(|&a, &b| base_tot_ns[a].total_cmp(&base_tot_ns[b]));
        runs[base_idx][idx_sorted[repeats / 2]]
    };

    println!(
        "\n--- 1B. End-to-end timings (A6a + A6b + A6c + A7, ns/row median & min..max over {} repeats) ---",
        repeats
    );
    println!(
        "{:<18} {:>7} {:>7} {:>7} {:>7} {:>7} {:>19} {:>8} {:>9} {:>9} {:>8}",
        "arm",
        "A6a",
        "A6b",
        "A6c",
        "A7",
        "C+E",
        "C+E (min..max)",
        "ratio",
        "i9_anc_Δ",
        "i9_prp_Δ",
        "%1w_wall"
    );
    let rows_f = total_rows as f64;
    for (ai, arm) in arms.iter().enumerate() {
        let a6a_v: Vec<f64> = runs[ai].iter().map(|p| p.a6a as f64 / rows_f).collect();
        let a6b_v: Vec<f64> = runs[ai].iter().map(|p| p.a6b() as f64 / rows_f).collect();
        let a6c_v: Vec<f64> = runs[ai].iter().map(|p| p.a6c as f64 / rows_f).collect();
        let a7_v: Vec<f64> = runs[ai].iter().map(|p| p.a7 as f64 / rows_f).collect();
        let ce_v: Vec<f64> = runs[ai].iter().map(|p| p.total() as f64 / rows_f).collect();
        let ratio_v: Vec<f64> = (0..repeats)
            .map(|r| runs[ai][r].total() as f64 / runs[base_idx][r].total() as f64)
            .collect();
        let anc_v: Vec<f64> = (0..repeats)
            .map(|r| I9_CE_MS - i9_anchored_ms(&runs[ai][r], &runs[base_idx][r]))
            .collect();
        let prp_v: Vec<f64> = (0..repeats)
            .map(|r| I9_CE_MS - i9_prop_ms(&runs[ai][r], &runs[base_idx][r]))
            .collect();

        let (a6a_m, _, _) = median_min_max(&a6a_v);
        let (a6b_m, _, _) = median_min_max(&a6b_v);
        let (a6c_m, _, _) = median_min_max(&a6c_v);
        let (a7_m, _, _) = median_min_max(&a7_v);
        let (ce_m, ce_lo, ce_hi) = median_min_max(&ce_v);
        let (rat_m, _, _) = median_min_max(&ratio_v);
        let (anc_m, _, _) = median_min_max(&anc_v);
        let (prp_m, _, _) = median_min_max(&prp_v);
        let pct_wall = 100.0 * anc_m / I9_WALL_1W_MS;
        println!(
            "{:<18} {:>7.4} {:>7.4} {:>7.4} {:>7.4} {:>7.4} ({:>6.4}..{:>6.4}) {:>7.4}x {:>+8.1}ms {:>+8.1}ms {:>+7.2}%",
            arm.name, a6a_m, a6b_m, a6c_m, a7_m, ce_m, ce_lo, ce_hi, rat_m, anc_m, prp_m, pct_wall
        );
    }

    // Per-block oracle ceiling across fixed LIMITs (2..64)
    let mut oracle_ns = 0u64;
    let mut base_sum_ns = 0u64;
    let mut best_counts = vec![0usize; fixed_oracle_indices.len()];
    let base_pos = fixed_oracle_indices
        .iter()
        .position(|&x| x == base_idx)
        .unwrap();
    for (bi, &base_t) in per_block_med_ns[base_pos].iter().enumerate() {
        let mut best_t = u64::MAX;
        let mut best_op = base_pos;
        for (op, row) in per_block_med_ns.iter().enumerate() {
            if row[bi] < best_t {
                best_t = row[bi];
                best_op = op;
            }
        }
        oracle_ns += best_t;
        base_sum_ns += base_t;
        best_counts[best_op] += 1;
    }
    let oracle_ratio = oracle_ns as f64 / base_sum_ns as f64;
    let oracle_save_ms = I9_CE_MS * (1.0 - oracle_ratio);
    println!("\n--- 1C. Per-block oracle LIMIT upper bound (best fixed LIMIT per block) ---");
    println!(
        "Oracle ratio vs limit_16_prod: {:.4}x (implied saving {:+.1} ms = {:+.2}% of 1,306 ms wall; gate = +{:.2} ms)",
        oracle_ratio,
        oracle_save_ms,
        100.0 * oracle_save_ms / I9_WALL_1W_MS,
        GATE_MS
    );
    print!("Per-block argmin distribution (prod dispatch): ");
    for (op, &ai) in fixed_oracle_indices.iter().enumerate() {
        if best_counts[op] > 0 {
            print!("{}={} ", arms[ai].name, best_counts[op]);
        }
    }
    println!();

    // Fixed forward-dispatch per-block oracle (removes m<65536 bidi->forward mode switch!)
    let mut fwd_oracle_ns = 0u64;
    let mut fwd_base_ns = 0u64;
    let mut fwd_best_counts = vec![0usize; fwd_oracle_indices.len()];
    let fwd_base_pos = fwd_oracle_indices
        .iter()
        .position(|&x| arms[x].name == "fwd_limit_16")
        .unwrap();
    for (bi, &base_t) in per_block_fwd_ns[fwd_base_pos].iter().enumerate() {
        let mut best_t = u64::MAX;
        let mut best_fp = fwd_base_pos;
        for (fp, row) in per_block_fwd_ns.iter().enumerate() {
            if row[bi] < best_t {
                best_t = row[bi];
                best_fp = fp;
            }
        }
        fwd_oracle_ns += best_t;
        fwd_base_ns += base_t;
        fwd_best_counts[best_fp] += 1;
    }
    let fwd_oracle_ratio = fwd_oracle_ns as f64 / fwd_base_ns as f64;
    println!(
        "Fixed-forward per-block oracle vs fwd_limit_16: {:.4}x ({:+.2}% of fwd C+E)",
        fwd_oracle_ratio,
        100.0 * (1.0 - fwd_oracle_ratio)
    );
    print!("Per-block argmin distribution (fixed forward): ");
    for (fp, &ai) in fwd_oracle_indices.iter().enumerate() {
        if fwd_best_counts[fp] > 0 {
            print!("{}={} ", arms[ai].name, fwd_best_counts[fp]);
        }
    }
    println!();
    println!(
        "(Baseline median A6b breakdown: setup={:.4}, steps={:.4}, dict={:.4} ns/row)",
        base_med_run.a6b_setup as f64 / rows_f,
        base_med_run.a6b_steps as f64 / rows_f,
        base_med_run.a6b_dict as f64 / rows_f
    );
}

// ---------------------------------------------------------------------------
// Command 2: Words representation & dictionary dedup
// ---------------------------------------------------------------------------

fn cmd_words(blocks: &[BlockInput], repeats: usize) {
    let total_rows: usize = blocks.iter().map(|b| b.n).sum();
    println!(
        "\n=== IDEA 2: WORDS REPRESENTATION & DICTIONARY DEDUP (blocks={}, rows={}, repeats={}) ===",
        blocks.len(),
        total_rows,
        repeats
    );

    let modes = [
        WordMode::Prod,
        WordMode::NoMemoExpand,
        WordMode::TrimMemoSlack,
        WordMode::NoContentsDedup,
        WordMode::NoDedup,
        WordMode::Pairs2048InlineOnlyDedup,
        WordMode::Pairs2048LongOnlyDedup,
        WordMode::BidiPreShifted,
        WordMode::ProdForward,
        WordMode::ProdForwardUnroll4,
        WordMode::SlpEmitCopyOut,
        WordMode::SlpEmitDfs,
    ];

    let mut kernel = OpenKernel::default();
    let mut out = Vec::new();
    let mut stats_by_arm = Vec::with_capacity(modes.len());

    // Warm-up + exact byte-identity check
    for &mode in &modes {
        let mut tot = BlockStats::default();
        for b in blocks {
            let (_ph, st) = kernel.run_block(
                &b.records,
                b.n,
                b.orig_ptr,
                LimitPolicy::Fixed(PROD_LIMIT),
                mode,
                StepMode::Prod,
                DispatchMode::Prod,
                &mut out,
                true,
            );
            assert_eq!(
                out,
                b.walked,
                "BYTE MISMATCH word_mode={} block={}",
                mode.name(),
                b.index
            );
            tot.add(st);
        }
        stats_by_arm.push(tot);
    }
    println!(
        "# Exactness verified: 100% byte-identical to bwt::walk on all {} blocks x {} word modes",
        blocks.len(),
        modes.len()
    );

    let mut runs: Vec<Vec<PhaseNs>> = vec![Vec::with_capacity(repeats); modes.len()];
    for r in 0..repeats {
        for (ai, &mode) in modes.iter().enumerate() {
            let mut sum = PhaseNs::default();
            for b in blocks {
                let (ph, _) = kernel.run_block(
                    &b.records,
                    b.n,
                    b.orig_ptr,
                    LimitPolicy::Fixed(PROD_LIMIT),
                    mode,
                    StepMode::Prod,
                    DispatchMode::Prod,
                    &mut out,
                    false,
                );
                black_box(&out);
                sum.add(ph);
            }
            runs[ai].push(sum);
        }
        eprintln!("  [words] repeat {}/{} done", r + 1, repeats);
    }

    println!("\n--- 2A. Dictionary footprint & slack ---");
    println!(
        "{:<24} {:>9} {:>9} {:>10} {:>10} {:>8} {:>6} {:>5}",
        "mode", "dict/blk", "long/blk", "sp_tot_KB", "sp_use_KB", "slack%", "wideE", "bidi"
    );
    let nb = blocks.len() as f64;
    for (ai, &mode) in modes.iter().enumerate() {
        let s = &stats_by_arm[ai];
        let slack_pct = if s.spelling_bytes > 0 {
            100.0 * (s.spelling_bytes.saturating_sub(s.spelling_used_bytes)) as f64
                / s.spelling_bytes as f64
        } else {
            0.0
        };
        println!(
            "{:<24} {:>9.0} {:>9.0} {:>10.1} {:>10.1} {:>7.1}% {:>6} {:>5}",
            mode.name(),
            s.dict_entries as f64 / nb,
            s.long_entries as f64 / nb,
            s.spelling_bytes as f64 / (1024.0 * nb),
            s.spelling_used_bytes as f64 / (1024.0 * nb),
            slack_pct,
            s.wide_edges,
            s.bidi_blocks,
        );
    }

    let base_idx = 0usize; // prod_baseline
    let fwd_idx = modes
        .iter()
        .position(|&m| m == WordMode::ProdForward)
        .unwrap();
    let rows_f = total_rows as f64;
    println!(
        "\n--- 2B. End-to-end timings (A6a + A6b + A6c + A7, ns/row median & min..max over {} repeats) ---",
        repeats
    );
    println!(
        "{:<24} {:>7} {:>7} {:>7} {:>7} {:>19} {:>8} {:>9} {:>9} {:>8}",
        "mode",
        "A6b-dct",
        "A6b",
        "A6c",
        "A7",
        "C+E (min..max)",
        "ratio",
        "i9_anc_Δ",
        "i9_prp_Δ",
        "%1w_wall"
    );
    for (ai, &mode) in modes.iter().enumerate() {
        let dct_v: Vec<f64> = runs[ai]
            .iter()
            .map(|p| p.a6b_dict as f64 / rows_f)
            .collect();
        let a6b_v: Vec<f64> = runs[ai].iter().map(|p| p.a6b() as f64 / rows_f).collect();
        let a6c_v: Vec<f64> = runs[ai].iter().map(|p| p.a6c as f64 / rows_f).collect();
        let a7_v: Vec<f64> = runs[ai].iter().map(|p| p.a7 as f64 / rows_f).collect();
        let ce_v: Vec<f64> = runs[ai].iter().map(|p| p.total() as f64 / rows_f).collect();
        let rat_v: Vec<f64> = (0..repeats)
            .map(|r| runs[ai][r].total() as f64 / runs[base_idx][r].total() as f64)
            .collect();
        let anc_v: Vec<f64> = (0..repeats)
            .map(|r| I9_CE_MS - i9_anchored_ms(&runs[ai][r], &runs[base_idx][r]))
            .collect();
        let prp_v: Vec<f64> = (0..repeats)
            .map(|r| I9_CE_MS - i9_prop_ms(&runs[ai][r], &runs[base_idx][r]))
            .collect();

        let (dct_m, _, _) = median_min_max(&dct_v);
        let (a6b_m, _, _) = median_min_max(&a6b_v);
        let (a6c_m, _, _) = median_min_max(&a6c_v);
        let (a7_m, _, _) = median_min_max(&a7_v);
        let (ce_m, ce_lo, ce_hi) = median_min_max(&ce_v);
        let (rat_m, _, _) = median_min_max(&rat_v);
        let (anc_m, _, _) = median_min_max(&anc_v);
        let (prp_m, _, _) = median_min_max(&prp_v);
        println!(
            "{:<24} {:>7.4} {:>7.4} {:>7.4} {:>7.4} {:>6.4}({:>6.4}..{:>6.4}) {:>7.4}x {:>+8.1}ms {:>+8.1}ms {:>+7.2}%",
            mode.name(),
            dct_m,
            a6b_m,
            a6c_m,
            a7_m,
            ce_m,
            ce_lo,
            ce_hi,
            rat_m,
            anc_m,
            prp_m,
            100.0 * anc_m / I9_WALL_1W_MS
        );
    }

    // Controlled comparison of SLP-in-emit against prod_forward_emit (same forward emission!)
    println!(
        "\n--- 2C. Controlled SLP-in-emit vs prod_forward_emit (excluding forward-emit gain) ---"
    );
    for &mode in &[WordMode::SlpEmitCopyOut, WordMode::SlpEmitDfs] {
        let ai = modes.iter().position(|&m| m == mode).unwrap();
        let rat_fwd: Vec<f64> = (0..repeats)
            .map(|r| runs[ai][r].total() as f64 / runs[fwd_idx][r].total() as f64)
            .collect();
        let dct_delta: Vec<f64> = (0..repeats)
            .map(|r| (runs[ai][r].a6b_dict as f64 - runs[fwd_idx][r].a6b_dict as f64) / rows_f)
            .collect();
        let a7_delta: Vec<f64> = (0..repeats)
            .map(|r| (runs[ai][r].a7 as f64 - runs[fwd_idx][r].a7 as f64) / rows_f)
            .collect();
        let (rf, rlo, rhi) = median_min_max(&rat_fwd);
        let (dd, _, _) = median_min_max(&dct_delta);
        let (ed, _, _) = median_min_max(&a7_delta);
        println!(
            "  {:<22} vs prod_forward_emit: {:.4}x ({:.4}..{:.4}x), ΔA6b-dict={:+.4} ns/row, ΔA7={:+.4} ns/row",
            mode.name(),
            rf,
            rlo,
            rhi,
            dd,
            ed
        );
    }
}

// ---------------------------------------------------------------------------
// Command 3: Cheaper induction steps (diagnosis + optimizations + interleaving)
// ---------------------------------------------------------------------------

fn cmd_steps(blocks: &[BlockInput], repeats: usize) {
    let total_rows: usize = blocks.iter().map(|b| b.n).sum();
    println!(
        "\n=== IDEA 3: CHEAPER INDUCTION STEPS & DEPENDENT-LOAD DIAGNOSIS (blocks={}, rows={}, repeats={}) ===",
        blocks.len(),
        total_rows,
        repeats
    );

    // Part 3A: Structural & branch diagnostics + prepare standalone Narrow arenas
    let mut diag = StepDiag::default();
    let mut preps = Vec::new();
    for b in blocks {
        if let Some(p) = prepare_block_elim(b, &mut diag) {
            preps.push(p);
        }
    }
    diag.arena_kib.sort_by(|a, b| a.total_cmp(b));
    let non_eq = diag.total_steps - diag.eq_xy;
    println!(
        "--- 3A. Induction step anatomy ({} Narrow non-spiral blocks, {} steps) ---",
        preps.len(),
        diag.total_steps
    );
    println!(
        "  Arena (nodes+rules) KiB: p10={:.1}, p50={:.1}, p90={:.1}, p99={:.1}, max={:.1} (>128 KiB: {}, >256 KiB L2: {})",
        percentile(&diag.arena_kib, 0.10),
        percentile(&diag.arena_kib, 0.50),
        percentile(&diag.arena_kib, 0.90),
        percentile(&diag.arena_kib, 0.99),
        percentile(&diag.arena_kib, 1.00),
        diag.blocks_gt_128k,
        diag.blocks_gt_256k,
    );
    println!(
        "  Step branches: x==y {:.2}%, aw<bw {:.2}%, aw>bw {:.2}%, aw==bw {:.2}%; 2-bit mispred on (aw<=bw): {:.2}%",
        100.0 * diag.eq_xy as f64 / diag.total_steps as f64,
        100.0 * diag.lt_w as f64 / diag.total_steps as f64,
        100.0 * diag.gt_w as f64 / diag.total_steps as f64,
        100.0 * diag.eq_w as f64 / diag.total_steps as f64,
        100.0 * diag.aw_le_bw_2bit_miss as f64 / non_eq.max(1) as f64,
    );
    let non_zero_joins = diag.join_calls - diag.join_zero_arg;
    println!(
        "  join() mix: zero-arg {:.2}%, non-zero {:.2}% (of non-zero: len<=4 {:.1}%, len<=7 {:.1}%, len<16 {:.1}%, len>=16 {:.1}%, saturated {:.2}%)",
        100.0 * diag.join_zero_arg as f64 / diag.join_calls.max(1) as f64,
        100.0 * non_zero_joins as f64 / diag.join_calls.max(1) as f64,
        100.0 * diag.join_len_le_4 as f64 / non_zero_joins.max(1) as f64,
        100.0 * diag.join_len_le_7 as f64 / non_zero_joins.max(1) as f64,
        100.0 * diag.join_len_lt_16 as f64 / non_zero_joins.max(1) as f64,
        100.0 * diag.join_len_ge_16 as f64 / non_zero_joins.max(1) as f64,
        100.0 * diag.join_saturated as f64 / non_zero_joins.max(1) as f64,
    );

    // Part 3B: Full end-to-end (A6a + A6b + A6c + A7) step optimization arms
    let step_modes = [
        StepMode::Prod,
        StepMode::NoJoinAblat,
        StepMode::RegXY,
        StepMode::HoistNext,
        StepMode::BranchlessSide,
        StepMode::TypedNarrow,
        StepMode::TypedPrefetchTouch,
        StepMode::Inline7Node20,
        StepMode::Compact8bSlp,
        StepMode::Compact8bSlpHoist,
    ];
    let mut kernel = OpenKernel::default();
    let mut out = Vec::new();
    let mut stats_by_arm = Vec::with_capacity(step_modes.len());

    for &sm in &step_modes {
        let mut tot = BlockStats::default();
        for b in blocks {
            let (_ph, st) = kernel.run_block(
                &b.records,
                b.n,
                b.orig_ptr,
                LimitPolicy::Fixed(PROD_LIMIT),
                WordMode::Prod,
                sm,
                DispatchMode::Prod,
                &mut out,
                false,
            );
            if sm != StepMode::NoJoinAblat {
                assert_eq!(
                    out,
                    b.walked,
                    "BYTE MISMATCH step_mode={} block={}",
                    sm.name(),
                    b.index
                );
            }
            tot.add(st);
        }
        stats_by_arm.push(tot);
    }
    println!(
        "# Exactness verified: 100% byte-identical to bwt::walk on all {} blocks x {} exact step modes",
        blocks.len(),
        step_modes.len() - 1
    );

    let mut runs: Vec<Vec<PhaseNs>> = vec![Vec::with_capacity(repeats); step_modes.len()];
    for r in 0..repeats {
        for (ai, &sm) in step_modes.iter().enumerate() {
            let mut sum = PhaseNs::default();
            for b in blocks {
                let (ph, _) = kernel.run_block(
                    &b.records,
                    b.n,
                    b.orig_ptr,
                    LimitPolicy::Fixed(PROD_LIMIT),
                    WordMode::Prod,
                    sm,
                    DispatchMode::Prod,
                    &mut out,
                    false,
                );
                black_box(&out);
                sum.add(ph);
            }
            runs[ai].push(sum);
        }
        eprintln!("  [steps] e2e repeat {}/{} done", r + 1, repeats);
    }

    let base_idx = 0usize;
    let rows_f = total_rows as f64;
    let nb = blocks.len() as f64;
    let total_steps_f = stats_by_arm[0].steps as f64;
    println!(
        "\n--- 3B. End-to-end step optimization timings (median & min..max over {} repeats) ---",
        repeats
    );
    println!(
        "{:<22} {:>7} {:>8} {:>16} {:>7} {:>7} {:>7} {:>8} {:>8} {:>9} {:>8}",
        "mode",
        "arn_KB",
        "ns/step",
        "steps(min..max)",
        "A6b-dct",
        "A6b",
        "C+E",
        "stp_rat",
        "a6b_rat",
        "i9_anc_Δ",
        "%1w_wall"
    );
    for (ai, &sm) in step_modes.iter().enumerate() {
        let arn_kb = stats_by_arm[ai].arena_bytes as f64 / (1024.0 * nb);
        let nss_v: Vec<f64> = runs[ai]
            .iter()
            .map(|p| p.a6b_steps as f64 / total_steps_f)
            .collect();
        let stp_v: Vec<f64> = runs[ai]
            .iter()
            .map(|p| p.a6b_steps as f64 / rows_f)
            .collect();
        let dct_v: Vec<f64> = runs[ai]
            .iter()
            .map(|p| p.a6b_dict as f64 / rows_f)
            .collect();
        let a6b_v: Vec<f64> = runs[ai].iter().map(|p| p.a6b() as f64 / rows_f).collect();
        let ce_v: Vec<f64> = runs[ai].iter().map(|p| p.total() as f64 / rows_f).collect();
        let stp_r: Vec<f64> = (0..repeats)
            .map(|r| runs[ai][r].a6b_steps as f64 / runs[base_idx][r].a6b_steps as f64)
            .collect();
        let a6b_r: Vec<f64> = (0..repeats)
            .map(|r| runs[ai][r].a6b() as f64 / runs[base_idx][r].a6b() as f64)
            .collect();
        let anc_v: Vec<f64> = (0..repeats)
            .map(|r| I9_CE_MS - i9_anchored_ms(&runs[ai][r], &runs[base_idx][r]))
            .collect();

        let (nss_m, _, _) = median_min_max(&nss_v);
        let (stp_m, stp_lo, stp_hi) = median_min_max(&stp_v);
        let (dct_m, _, _) = median_min_max(&dct_v);
        let (a6b_m, _, _) = median_min_max(&a6b_v);
        let (ce_m, _, _) = median_min_max(&ce_v);
        let (sr_m, _, _) = median_min_max(&stp_r);
        let (ar_m, _, _) = median_min_max(&a6b_r);
        let (anc_m, _, _) = median_min_max(&anc_v);
        if sm == StepMode::NoJoinAblat {
            println!(
                "{:<22} {:>7.1} {:>8.2} {:>5.4}({:.4}..{:.4}) {:>7} {:>7} {:>7} {:>7.4}x {:>8} {:>9} {:>8}",
                sm.name(),
                arn_kb,
                nss_m,
                stp_m,
                stp_lo,
                stp_hi,
                "-",
                "-",
                "-",
                sr_m,
                "-",
                "-",
                "-"
            );
        } else {
            println!(
                "{:<22} {:>7.1} {:>8.2} {:>5.4}({:.4}..{:.4}) {:>7.4} {:>7.4} {:>7.4} {:>7.4}x {:>7.4}x {:>+8.1}ms {:>+7.2}%",
                sm.name(),
                arn_kb,
                nss_m,
                stp_m,
                stp_lo,
                stp_hi,
                dct_m,
                a6b_m,
                ce_m,
                sr_m,
                ar_m,
                anc_m,
                100.0 * anc_m / I9_WALL_1W_MS
            );
        }
    }

    // Part 3C: Inter-block 1-way vs 2-way vs 4-way interleaving (all blocks, plus L2-small vs L2-large)
    let all_refs: Vec<&PreparedElim> = preps.iter().collect();
    let small_refs: Vec<&PreparedElim> = preps
        .iter()
        .filter(|p| p.arena_est_bytes <= 120 * 1024)
        .collect();
    let large_refs: Vec<&PreparedElim> = preps
        .iter()
        .filter(|p| p.arena_est_bytes > 200 * 1024)
        .collect();
    let expected_all_steps: usize = all_refs.iter().map(|p| p.expected_steps).sum();

    println!(
        "\n--- 3C. Dependent-load chain test: 1-way vs 2-way vs 4-way inter-block interleaving ({} repeats) ---",
        repeats
    );
    let subsets: [(&str, &[&PreparedElim]); 3] = [
        ("all_blocks", &all_refs),
        ("l2_fit_<=120KiB", &small_refs),
        ("l2_near_>200KiB", &large_refs),
    ];
    println!(
        "{:<18} {:>6} {:>10} {:>16} {:>16} {:>16} {:>16}",
        "subset",
        "blks",
        "steps",
        "1-way 16B ns/stp",
        "2-way 16B ns/stp",
        "4-way 16B ns/stp",
        "2-way 8B ns/stp"
    );
    for (sname, sub) in subsets {
        if sub.is_empty() {
            continue;
        }
        let mut r1 = Vec::with_capacity(repeats);
        let mut r2 = Vec::with_capacity(repeats);
        let mut r4 = Vec::with_capacity(repeats);
        let mut r2_8 = Vec::with_capacity(repeats);
        let mut r1_8 = Vec::with_capacity(repeats);
        let mut step_cnt = 0usize;
        for _ in 0..repeats {
            let (t1, s1) = run_interleaved::<1>(sub, false);
            let (t2, s2) = run_interleaved::<2>(sub, false);
            let (t4, s4) = run_interleaved::<4>(sub, false);
            let (t1_8, s1_8) = run_interleaved::<1>(sub, true);
            let (t2_8, s2_8) = run_interleaved::<2>(sub, true);
            assert_eq!(s1, s2);
            assert_eq!(s1, s4);
            assert_eq!(s1, s1_8);
            assert_eq!(s1, s2_8);
            step_cnt = s1;
            r1.push(t1 as f64 / s1 as f64);
            r2.push(t2 as f64 / s2 as f64);
            r4.push(t4 as f64 / s4 as f64);
            r1_8.push(t1_8 as f64 / s1_8 as f64);
            r2_8.push(t2_8 as f64 / s2_8 as f64);
        }
        if sname == "all_blocks" {
            assert_eq!(step_cnt, expected_all_steps);
        }
        let (m1, lo1, hi1) = median_min_max(&r1);
        let (m2, lo2, hi2) = median_min_max(&r2);
        let (m4, lo4, hi4) = median_min_max(&r4);
        let (m1_8, _, _) = median_min_max(&r1_8);
        let (m2_8, lo2_8, hi2_8) = median_min_max(&r2_8);
        println!(
            "{:<18} {:>6} {:>10} {:>5.2}({:.2}..{:.2}) {:>5.2}({:.2}..{:.2}) {:>5.2}({:.2}..{:.2}) {:>5.2}({:.2}..{:.2})",
            sname,
            sub.len(),
            step_cnt,
            m1,
            lo1,
            hi1,
            m2,
            lo2,
            hi2,
            m4,
            lo4,
            hi4,
            m2_8,
            lo2_8,
            hi2_8
        );
        println!(
            "  ratios vs 1-way 16B: 2-way-16B={:.3}x, 4-way-16B={:.3}x, 1-way-8B={:.3}x ({:.2} ns/stp), 2-way-8B={:.3}x",
            m2 / m1,
            m4 / m1,
            m1_8 / m1,
            m1_8,
            m2_8 / m1
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <limit|words|steps|all> FILE.bz2 [--repeats N] [--forced] [--x4]",
            args[0]
        );
        std::process::exit(1);
    }
    let cmd = args[1].as_str();
    let path = args[2].as_str();
    let mut repeats = 5usize;
    let mut forced = false;
    let mut x4 = false;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--repeats" => {
                i += 1;
                repeats = args[i].parse().expect("parse --repeats");
            }
            "--forced" => forced = true,
            "--x4" => x4 = true,
            other => {
                eprintln!("Unknown arg: {other}");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let (blocks, total_blocks, total_rows) = load_blocks(path, forced, x4);
    println!(
        "# rauzy_output_census: cmd={cmd} file={path} forced={forced} x4={x4} repeats={repeats} total_blocks={total_blocks} total_rows={total_rows} open_blocks={}",
        blocks.len()
    );
    println!(
        "# i9-10900K anchor (wikidata x4): wall_1w={:.1}ms (stage1={:.1}ms), C+E={:.1}ms (A6a={:.1}, A6b={:.1}, A6c={:.1}, A7={:.1}), 3% gate={:.2}ms",
        I9_WALL_1W_MS, I9_STAGE1_1W_MS, I9_CE_MS, I9_A6A_MS, I9_A6B_MS, I9_A6C_MS, I9_A7_MS, GATE_MS
    );

    match cmd {
        "limit" => cmd_limit(&blocks, repeats),
        "words" => cmd_words(&blocks, repeats),
        "steps" => cmd_steps(&blocks, repeats),
        "all" => {
            cmd_limit(&blocks, repeats);
            cmd_words(&blocks, repeats);
            cmd_steps(&blocks, repeats);
        }
        other => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}
