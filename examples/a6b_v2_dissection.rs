#![forbid(unsafe_code)]
//! Post-#1046 A6b second-pass dissection & redesign probe (`a6b-v2-dissection`).
//!
//! Smooth-zone characterisation of `#1046`'s winning A6b (`comb_nobc_soa_gen`, PR #1053),
//! which takes ~17% of the `#1062` W4 composed worker stack (~155 ms 1W / ~22 ms/w 8W
//! on i9-10900K: ~24 ms setup, ~75 ms induction steps, ~55 ms dictionary).
//!
//! Frozen Stop Test (pre-data, 2026-11-27; RULES C1a/C2):
//! - Baseline: exact `#1046` (`comb_nobc_soa_gen`), paired with `#1057` unmodified SIMD
//!   `Reader::wire` (A6c) + `S=16` sampled-anchor dynamic 4-lane forward emission (A7),
//!   warm reusable buffers, 233 production-admitted Open blocks of `/tmp/wikidata.bz2`,
//!   7 interleaved repeats pinned to CPU 0 (`taskset -c 0`).
//! - Correctness: 100% safe Rust (`#![forbid(unsafe_code)]`), 100% byte-exact vs `bwt::walk`
//!   on all 431/431 wikidata blocks (forced Open) + genuine single-cycle synthetic stress suite
//!   (ns==0, 40k spiral widest=39999, deep/duplicate long words, non-ASCII digrams, aw==bw
//!   cascades, gen wraparound).
//! - PASS iff a candidate safe-Rust second-pass redesign cuts:
//!   1. `#1046`'s isolated A6b median by >= 15.0% (disjoint [min..max] spreads) on ALL THREE
//!      builds: baseline x86-64, `-C target-cpu=native`, and JCC-aligned native control
//!      (`-C target-cpu=native -C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp`);
//!   2. full `A6b + A6c + 4-lane A7` with non-overlapping [min..max] spreads on ALL THREE
//!      builds (confirmed on both wall clock and Linux `/proc/thread-self/schedstat` CPU clock);
//!   3. zero A7 regression (`A7_delta <= +0.5%` on all three builds).
//! - STOP otherwise (record scoped dead-end row in `frontier/dead-ends.tsv`).
//!
//! Usage:
//!   a6b_v2_dissection <wikidata.bz2> [--repeats 7]

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::hint::black_box;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Safe on-CPU thread clock (Linux /proc/thread-self/schedstat, zero unsafe)
// ---------------------------------------------------------------------------

struct CpuClock {
    #[cfg(target_os = "linux")]
    file: std::fs::File,
    #[cfg(not(target_os = "linux"))]
    origin: Instant,
}

impl Default for CpuClock {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuClock {
    fn new() -> Self {
        #[cfg(target_os = "linux")]
        {
            CpuClock {
                file: std::fs::File::open("/proc/thread-self/schedstat")
                    .expect("open /proc/thread-self/schedstat"),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            CpuClock {
                origin: Instant::now(),
            }
        }
    }

    #[inline(always)]
    fn now_ns(&self) -> u64 {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::FileExt;
            let mut buf = [0u8; 64];
            let n = self.file.read_at(&mut buf, 0).expect("read schedstat");
            let mut val = 0u64;
            for &b in &buf[..n] {
                if !b.is_ascii_digit() {
                    break;
                }
                val = val * 10 + u64::from(b - b'0');
            }
            val
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.origin.elapsed().as_nanos() as u64
        }
    }
}

// ---------------------------------------------------------------------------
// Open constants (matching src/open.rs, #1046, #1057, #1062)
// ---------------------------------------------------------------------------

const PROD_LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const CERTIFIED_WIDTH: u32 = 32_768;

const LAZY_DENSITY: (usize, usize) = (3, 20);
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

const CHUNK: usize = 512;
const CHUNK_BYTES: usize = 2 * CHUNK;
const NONE_CHUNK: u32 = u32::MAX;
const EPOCH: u32 = 1 << 16;

// u128 inline15 encoding (exact #1046):
//   byte 15 == 0       => empty word (entire u128 == 0)
//   byte 15 in 1..=15  => inline payload of length `tag` in bytes 0..tag, rest 0
//   byte 15 == 0xFF    => long rule handle: bits 0..32 = rule_id (>= 257), bits 32..64 = exact len
const U128_PAYLOAD_MASK: u128 = (1u128 << 120) - 1;
const U128_LONG_TAG: u128 = 0xFFu128 << 120;

// u64 SoA-inline7 encoding (V3):
//   byte 7 == 0        => empty word (entire u64 == 0)
//   byte 7 in 1..=7    => inline payload of length `tag` in bytes 0..tag, rest 0
//   byte 7 in 8..=15   => inline-15 rule handle: bits 0..24 = rule_id (>= 257), exact len = tag
//   byte 7 == 0xFF     => long rule handle: bits 0..24 = rule_id (>= 257), bits 24..56 = exact len
const U64_PAYLOAD_MASK: u64 = (1u64 << 56) - 1;
const U64_ID_MASK: u64 = (1u64 << 24) - 1;
const U64_LONG_TAG: u64 = 0xFFu64 << 56;

const GEN_PAIR_SLOTS: usize = 1024;
const GEN_CONTENT_SLOTS: usize = 4096;

// 4-lane sampled anchor constants (#1057)
const S_ANCHORS: usize = 16;
const MAX_ANCHORS: usize = 32;
const ANCHOR_TAG: u32 = 1u32 << 31;
const SLAB_SIZE: usize = 32 * 1024;

// i9-10900K post-#1046 calibration anchors (1W / 8W)
const I9_A6B_1046_1W_MS: f64 = 155.0;
const I9_A6B_1046_8W_MS: f64 = 22.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

fn open_cap(records: usize, n: usize, p: u32) -> Option<Cap> {
    if n == 0 || n > 900_000 || p as usize >= n {
        None
    } else if lazy_counts_admit(records, n) {
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
fn literal_u128(byte: u8) -> u128 {
    (1u128 << 120) | u128::from(byte)
}

#[inline(always)]
fn literal_u64(byte: u8) -> u64 {
    (1u64 << 56) | u64::from(byte)
}

#[derive(Default, Clone, Copy)]
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
    let start = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]) as usize;
    let end = u32::from_le_bytes([slot[4], slot[5], slot[6], slot[7]]) as usize;
    &spelling[start..end]
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

// ---------------------------------------------------------------------------
// OutQ16 & A6a / A6c (exact mirror of src/open/outq.rs)
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
}

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
            let at = tail as usize * CHUNK_BYTES + 2 * f;
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
    fn wire(&mut self, dst: &mut [u32], pack: u32) {
        let span = dst.len();
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let base = self.id as usize * CHUNK_BYTES + 2 * self.at;
            let src = &self.out[base..base + 2 * span];
            let hi = pack | (self.epoch * EPOCH);
            for (d, bytes) in dst.iter_mut().zip(src.chunks_exact(2)) {
                *d = hi | u32::from(u16::from_le_bytes([bytes[0], bytes[1]]));
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
            if self.at == CHUNK {
                self.id = self.spill.next[self.id as usize];
                self.at = 0;
            }
            if self.read == self.cut {
                self.epoch += 1;
                self.cut = self
                    .spill
                    .cuts
                    .get(self.epoch as usize)
                    .map_or(u32::MAX, |c| c[self.a]);
            }
            let take = dst
                .len()
                .min(CHUNK - self.at)
                .min((self.cut - self.read) as usize);
            let chunk = if self.id < self.in_out {
                let at = self.id as usize * CHUNK_BYTES;
                &self.out[at..at + CHUNK_BYTES]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][..]
            };
            let hi = pack | (self.epoch * EPOCH);
            for (d, bytes) in dst[..take]
                .iter_mut()
                .zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
            {
                *d = hi | u32::from(u16::from_le_bytes([bytes[0], bytes[1]]));
            }
            self.at += take;
            self.read += take as u32;
            dst = &mut dst[take..];
        }
    }
}

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
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        self.f.runs += 1;
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

// ---------------------------------------------------------------------------
// 4-lane forward emission (#1057 raw slabs, zero unsafe)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct SampledScratch {
    slices: Vec<(usize, usize, u16)>,
    ordered: Vec<(usize, usize)>,
}

struct SlabAlloc<'a> {
    arena: &'a mut [u8],
    top: usize,
    slices: &'a mut Vec<(usize, usize, u16)>,
    seg_head: [u16; MAX_ANCHORS],
    seg_tail: [u16; MAX_ANCHORS],
}

impl SlabAlloc<'_> {
    #[inline(always)]
    fn take_slab(&mut self, min_size: usize) -> (usize, usize) {
        let size = SLAB_SIZE.max(min_size);
        let start = self.top;
        self.top = start + size;
        (start, self.top)
    }

    #[inline(always)]
    fn push_slice(&mut self, seg: usize, start: usize, end: usize) {
        if start == end {
            return;
        }
        let idx = self.slices.len() as u16;
        self.slices.push((start, end, u16::MAX));
        let tail = self.seg_tail[seg];
        if tail == u16::MAX {
            self.seg_head[seg] = idx;
        } else {
            self.slices[tail as usize].2 = idx;
        }
        self.seg_tail[seg] = idx;
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_slow_raw(
        &mut self,
        seg: usize,
        code: usize,
        len: u8,
        inline16: &[[u8; INLINE]],
        spelling: &[u8],
        slice_start: &mut usize,
        at: &mut usize,
        end: &mut usize,
    ) {
        if len != LONG {
            self.push_slice(seg, *slice_start, *at);
            let (ns, ne) = self.take_slab(INLINE);
            *slice_start = ns;
            *at = ns;
            *end = ne;
            self.arena[ns..ns + INLINE].copy_from_slice(&inline16[code]);
            *at = ns + usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            if *at + word.len() + INLINE > *end {
                self.push_slice(seg, *slice_start, *at);
                let (ns, ne) = self.take_slab(word.len() + INLINE);
                *slice_start = ns;
                *at = ns;
                *end = ne;
            }
            self.arena[*at..*at + word.len()].copy_from_slice(word);
            *at += word.len();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_fwd_1lane(
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
    let mask = (1u32 << bits) - 1;
    let wide = !extra.is_empty();
    let mut j = mark;
    let mut at = 0usize;
    while at < n {
        let edge = edges[j as usize];
        let code = if wide {
            extra[j as usize] as usize
        } else {
            (edge >> bits) as usize
        };
        j = edge & mask;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            out[at..at + word.len()].copy_from_slice(word);
            at += word.len();
        }
    }
    out.truncate(n);
}

#[inline(always)]
fn tag_uniform_anchors(
    edges: &mut [u32],
    m: usize,
    mark: u32,
    bits: u32,
    wide: bool,
    dict_len: usize,
    anchors: &mut [u32; MAX_ANCHORS],
) -> usize {
    let can_tag =
        !wide && m >= S_ANCHORS * 8 && (dict_len - 1) <= ((u32::MAX >> 1) >> bits) as usize;
    if !can_tag {
        return 0;
    }
    let m32 = m as u32;
    anchors[0] = mark;
    for (k, slot) in anchors.iter_mut().enumerate().take(S_ANCHORS).skip(1) {
        let off = ((k as u64 * m as u64) / S_ANCHORS as u64) as u32;
        let mut a = mark + off;
        if a >= m32 {
            a -= m32;
        }
        *slot = a;
    }
    for &a in &anchors[..S_ANCHORS] {
        edges[a as usize] |= ANCHOR_TAG;
    }
    S_ANCHORS
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_4lane_raw(
    edges: &mut [u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    m: usize,
    mark: u32,
    bits: u32,
    anchors: &[u32],
    arena: &mut Vec<u8>,
    scratch: &mut SampledScratch,
) {
    let s = anchors.len();
    if s < 4 {
        for &a in anchors {
            edges[a as usize] &= !ANCHOR_TAG;
        }
        emit_fwd_1lane(edges, extra, spelling, inline16, lens, n, mark, bits, arena);
        scratch.ordered.clear();
        scratch.ordered.push((0, arena.len()));
        return;
    }

    let mask = (1u32 << bits) - 1;
    let want_arena = n + 8 * SLAB_SIZE + 4 * INLINE;
    if arena.len() < want_arena {
        if want_arena > arena.capacity() {
            arena.reserve_exact(want_arena - arena.len());
        }
        arena.resize(want_arena, 0);
    }
    scratch.slices.clear();
    scratch.ordered.clear();

    let mut alloc = SlabAlloc {
        arena,
        top: 0,
        slices: &mut scratch.slices,
        seg_head: [u16::MAX; MAX_ANCHORS],
        seg_tail: [u16::MAX; MAX_ANCHORS],
    };

    let mut seg_end_row = [u32::MAX; MAX_ANCHORS];
    let mut lj = [0u32; 4];
    let mut lseg = [0usize; 4];
    let mut lstart = [0usize; 4];
    let mut lat = [0usize; 4];
    let mut lend = [0usize; 4];
    let mut lactive = [false; 4];

    let mut next_seg = 0usize;
    let mut steps_done = 0usize;
    let mut budget_exceeded = false;

    macro_rules! start_seg {
        ($lane:expr, $seg:expr) => {{
            lseg[$lane] = $seg;
            lstart[$lane] = lat[$lane];
            let edge = edges[anchors[$seg] as usize] & !ANCHOR_TAG;
            lj[$lane] = edge & mask;
            let code = (edge >> bits) as usize;
            let len = lens[code];
            if len != LONG && lat[$lane] + INLINE <= lend[$lane] {
                let at = lat[$lane];
                alloc.arena[at..at + INLINE].copy_from_slice(&inline16[code]);
                lat[$lane] = at + usize::from(len);
            } else {
                alloc.emit_slow_raw(
                    $seg,
                    code,
                    len,
                    inline16,
                    spelling,
                    &mut lstart[$lane],
                    &mut lat[$lane],
                    &mut lend[$lane],
                );
            }
            steps_done += 1;
        }};
    }

    macro_rules! handle_hit {
        ($lane:expr) => {{
            let mut keep = true;
            while (edges[lj[$lane] as usize] as i32) < 0 {
                let finished = lseg[$lane];
                seg_end_row[finished] = lj[$lane];
                alloc.push_slice(finished, lstart[$lane], lat[$lane]);
                if next_seg < s {
                    if steps_done >= m {
                        lactive[$lane] = false;
                        keep = false;
                        break;
                    }
                    let seg = next_seg;
                    next_seg += 1;
                    start_seg!($lane, seg);
                } else {
                    lactive[$lane] = false;
                    keep = false;
                    break;
                }
            }
            keep
        }};
    }

    for lane in 0..4 {
        let (base, end) = alloc.take_slab(0);
        lstart[lane] = base;
        lat[lane] = base;
        lend[lane] = end;
        lactive[lane] = true;
        let seg = next_seg;
        next_seg += 1;
        start_seg!(lane, seg);
    }
    for lane in 0..4 {
        if (edges[lj[lane] as usize] as i32) < 0 {
            let _ = handle_hit!(lane);
        }
    }

    if lactive[0] && lactive[1] && lactive[2] && lactive[3] {
        let (mut j0, mut j1, mut j2, mut j3) = (lj[0], lj[1], lj[2], lj[3]);
        let (mut a0, mut a1, mut a2, mut a3) = (lat[0], lat[1], lat[2], lat[3]);
        let (mut e0, mut e1, mut e2, mut e3) = (lend[0], lend[1], lend[2], lend[3]);

        loop {
            let max_iters = (m - steps_done) / 4;
            let mut done = 0usize;
            let mut hit = false;
            for _ in 0..max_iters {
                let ed0 = edges[j0 as usize];
                let ed1 = edges[j1 as usize];
                let ed2 = edges[j2 as usize];
                let ed3 = edges[j3 as usize];
                if ((ed0 | ed1 | ed2 | ed3) as i32) < 0 {
                    hit = true;
                    break;
                }
                j0 = ed0 & mask;
                j1 = ed1 & mask;
                j2 = ed2 & mask;
                j3 = ed3 & mask;

                let c0 = (ed0 >> bits) as usize;
                let l0 = lens[c0];
                if l0 != LONG && a0 + INLINE <= e0 {
                    alloc.arena[a0..a0 + INLINE].copy_from_slice(&inline16[c0]);
                    a0 += usize::from(l0);
                } else {
                    alloc.emit_slow_raw(
                        lseg[0],
                        c0,
                        l0,
                        inline16,
                        spelling,
                        &mut lstart[0],
                        &mut a0,
                        &mut e0,
                    );
                }

                let c1 = (ed1 >> bits) as usize;
                let l1 = lens[c1];
                if l1 != LONG && a1 + INLINE <= e1 {
                    alloc.arena[a1..a1 + INLINE].copy_from_slice(&inline16[c1]);
                    a1 += usize::from(l1);
                } else {
                    alloc.emit_slow_raw(
                        lseg[1],
                        c1,
                        l1,
                        inline16,
                        spelling,
                        &mut lstart[1],
                        &mut a1,
                        &mut e1,
                    );
                }

                let c2 = (ed2 >> bits) as usize;
                let l2 = lens[c2];
                if l2 != LONG && a2 + INLINE <= e2 {
                    alloc.arena[a2..a2 + INLINE].copy_from_slice(&inline16[c2]);
                    a2 += usize::from(l2);
                } else {
                    alloc.emit_slow_raw(
                        lseg[2],
                        c2,
                        l2,
                        inline16,
                        spelling,
                        &mut lstart[2],
                        &mut a2,
                        &mut e2,
                    );
                }

                let c3 = (ed3 >> bits) as usize;
                let l3 = lens[c3];
                if l3 != LONG && a3 + INLINE <= e3 {
                    alloc.arena[a3..a3 + INLINE].copy_from_slice(&inline16[c3]);
                    a3 += usize::from(l3);
                } else {
                    alloc.emit_slow_raw(
                        lseg[3],
                        c3,
                        l3,
                        inline16,
                        spelling,
                        &mut lstart[3],
                        &mut a3,
                        &mut e3,
                    );
                }
                done += 1;
            }
            steps_done += done * 4;
            lj = [j0, j1, j2, j3];
            lat = [a0, a1, a2, a3];
            lend = [e0, e1, e2, e3];
            if !hit
                && ((edges[j0 as usize]
                    | edges[j1 as usize]
                    | edges[j2 as usize]
                    | edges[j3 as usize]) as i32)
                    >= 0
            {
                budget_exceeded = true;
                break;
            }
            let mut all_ok = true;
            for lane in 0..4 {
                if (edges[lj[lane] as usize] as i32) < 0 && !handle_hit!(lane) {
                    all_ok = false;
                }
            }
            if !all_ok {
                break;
            }
            (j0, j1, j2, j3) = (lj[0], lj[1], lj[2], lj[3]);
            (a0, a1, a2, a3) = (lat[0], lat[1], lat[2], lat[3]);
            (e0, e1, e2, e3) = (lend[0], lend[1], lend[2], lend[3]);
        }
    }

    let mut act = [0usize; 4];
    let mut n_act = 0usize;
    for (lane, &active) in lactive.iter().enumerate() {
        if active {
            act[n_act] = lane;
            n_act += 1;
        }
    }

    while !budget_exceeded && n_act == 3 {
        let (p0, p1, p2) = (act[0], act[1], act[2]);
        let (mut j0, mut j1, mut j2) = (lj[p0], lj[p1], lj[p2]);
        let (mut a0, mut a1, mut a2) = (lat[p0], lat[p1], lat[p2]);
        let (mut e0, mut e1, mut e2) = (lend[p0], lend[p1], lend[p2]);
        loop {
            let ed0 = edges[j0 as usize];
            let ed1 = edges[j1 as usize];
            let ed2 = edges[j2 as usize];
            if ((ed0 | ed1 | ed2) as i32) < 0 {
                (lj[p0], lj[p1], lj[p2]) = (j0, j1, j2);
                (lat[p0], lat[p1], lat[p2]) = (a0, a1, a2);
                (lend[p0], lend[p1], lend[p2]) = (e0, e1, e2);
                break;
            }
            if steps_done + 3 > m {
                budget_exceeded = true;
                break;
            }
            j0 = ed0 & mask;
            j1 = ed1 & mask;
            j2 = ed2 & mask;
            for (p, ed, a, e) in [
                (p0, ed0, &mut a0, &mut e0),
                (p1, ed1, &mut a1, &mut e1),
                (p2, ed2, &mut a2, &mut e2),
            ] {
                let c = (ed >> bits) as usize;
                let l = lens[c];
                if l != LONG && *a + INLINE <= *e {
                    alloc.arena[*a..*a + INLINE].copy_from_slice(&inline16[c]);
                    *a += usize::from(l);
                } else {
                    alloc.emit_slow_raw(lseg[p], c, l, inline16, spelling, &mut lstart[p], a, e);
                }
            }
            steps_done += 3;
        }
        if budget_exceeded {
            break;
        }
        let mut next_act = [0usize; 4];
        let mut new_n = 0;
        for &lane in &act[..n_act] {
            if (edges[lj[lane] as usize] as i32) < 0 {
                if handle_hit!(lane) {
                    next_act[new_n] = lane;
                    new_n += 1;
                }
            } else {
                next_act[new_n] = lane;
                new_n += 1;
            }
        }
        act = next_act;
        n_act = new_n;
    }

    while !budget_exceeded && n_act == 2 {
        let (p0, p1) = (act[0], act[1]);
        let (mut j0, mut j1) = (lj[p0], lj[p1]);
        let (mut a0, mut a1) = (lat[p0], lat[p1]);
        let (mut e0, mut e1) = (lend[p0], lend[p1]);
        loop {
            let ed0 = edges[j0 as usize];
            let ed1 = edges[j1 as usize];
            if ((ed0 | ed1) as i32) < 0 {
                (lj[p0], lj[p1]) = (j0, j1);
                (lat[p0], lat[p1]) = (a0, a1);
                (lend[p0], lend[p1]) = (e0, e1);
                break;
            }
            if steps_done + 2 > m {
                budget_exceeded = true;
                break;
            }
            j0 = ed0 & mask;
            j1 = ed1 & mask;
            for (p, ed, a, e) in [(p0, ed0, &mut a0, &mut e0), (p1, ed1, &mut a1, &mut e1)] {
                let c = (ed >> bits) as usize;
                let l = lens[c];
                if l != LONG && *a + INLINE <= *e {
                    alloc.arena[*a..*a + INLINE].copy_from_slice(&inline16[c]);
                    *a += usize::from(l);
                } else {
                    alloc.emit_slow_raw(lseg[p], c, l, inline16, spelling, &mut lstart[p], a, e);
                }
            }
            steps_done += 2;
        }
        if budget_exceeded {
            break;
        }
        let mut next_act = [0usize; 4];
        let mut new_n = 0;
        for &lane in &act[..n_act] {
            if (edges[lj[lane] as usize] as i32) < 0 {
                if handle_hit!(lane) {
                    next_act[new_n] = lane;
                    new_n += 1;
                }
            } else {
                next_act[new_n] = lane;
                new_n += 1;
            }
        }
        act = next_act;
        n_act = new_n;
    }

    while !budget_exceeded && n_act == 1 {
        let p0 = act[0];
        let mut j0 = lj[p0];
        let mut a0 = lat[p0];
        let mut e0 = lend[p0];
        loop {
            let ed0 = edges[j0 as usize];
            if (ed0 as i32) < 0 {
                lj[p0] = j0;
                lat[p0] = a0;
                lend[p0] = e0;
                break;
            }
            if steps_done + 1 > m {
                budget_exceeded = true;
                break;
            }
            j0 = ed0 & mask;
            let c = (ed0 >> bits) as usize;
            let l = lens[c];
            if l != LONG && a0 + INLINE <= e0 {
                alloc.arena[a0..a0 + INLINE].copy_from_slice(&inline16[c]);
                a0 += usize::from(l);
            } else {
                alloc.emit_slow_raw(
                    lseg[p0],
                    c,
                    l,
                    inline16,
                    spelling,
                    &mut lstart[p0],
                    &mut a0,
                    &mut e0,
                );
            }
            steps_done += 1;
        }
        if budget_exceeded {
            break;
        }
        if !handle_hit!(p0) {
            n_act = 0;
        }
    }

    for &a in anchors {
        edges[a as usize] &= !ANCHOR_TAG;
    }

    let mut order = [0usize; MAX_ANCHORS];
    let mut chain_len = 0usize;
    let mut cycle_closed = false;
    if !budget_exceeded {
        let mut cur = 0usize;
        for _ in 0..s {
            order[chain_len] = cur;
            chain_len += 1;
            let end_row = seg_end_row[cur];
            let Some(nxt) = anchors.iter().position(|&a| a == end_row) else {
                break;
            };
            cur = nxt;
            if cur == 0 {
                cycle_closed = true;
                break;
            }
        }
    }

    if !cycle_closed || chain_len == 0 || steps_done != m {
        emit_fwd_1lane(edges, extra, spelling, inline16, lens, n, mark, bits, arena);
        scratch.ordered.clear();
        scratch.ordered.push((0, arena.len()));
        return;
    }

    for &seg in &order[..chain_len] {
        let mut idx = alloc.seg_head[seg];
        while idx != u16::MAX {
            let (st, en, nxt) = alloc.slices[idx as usize];
            scratch.ordered.push((st, en));
            idx = nxt;
        }
    }
}

// ---------------------------------------------------------------------------
// Hash functions & shared A6b state
// ---------------------------------------------------------------------------

#[inline(always)]
fn short_u128_hash_base(entry_u128: u128, len: u8) -> u64 {
    let low = entry_u128 as u64;
    let high = (entry_u128 >> 64) as u64;
    let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ u64::from(len).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

/// Fast 1-multiply hash for `len <= 8` (`u64` key) into 4096 slots + 8-bit fingerprint.
#[inline(always)]
fn fast_u64_slot4k_fp(key: u64, len: u8) -> (usize, u32) {
    let h = (key ^ (u64::from(len) << 56)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let m = h ^ (h >> 28);
    ((m >> 52) as usize, ((m >> 44) & 0xFF) as u32)
}

/// Fast 1-multiply hash for `u128` (`len 1..=16`) into 4096 slots + 8-bit fingerprint.
#[inline(always)]
fn fast_u128_slot4k_fp(entry_u128: u128, len: u8) -> (usize, u32) {
    let low = entry_u128 as u64;
    let high = (entry_u128 >> 64) as u64;
    let mixed = low ^ high.rotate_left(29) ^ (u64::from(len) << 56);
    let h = mixed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let m = h ^ (h >> 28);
    ((m >> 52) as usize, ((m >> 44) & 0xFF) as u32)
}

#[inline(always)]
fn long_bytes_hash_base(bytes: &[u8]) -> u64 {
    let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let low = part(0);
    let high = part(bytes.len() - 8);
    let middle = part(bytes.len() / 2 - 4);
    let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ middle.wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

#[inline(always)]
fn long_bytes_slot4k_fp(bytes: &[u8]) -> (usize, u32) {
    let len = bytes.len();
    let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let low = part(0);
    let high = part(len - 8);
    let mid = part((len >> 1) - 4);
    let mixed = low ^ high.rotate_left(29) ^ mid.rotate_left(13) ^ (len as u64).rotate_left(47);
    let h = mixed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let m = h ^ (h >> 28);
    ((m >> 52) as usize, ((m >> 44) & 0xFF) as u32)
}

#[inline(always)]
fn long_prefix_plus_byte_slot4k_fp(prefix: &[u8], byte: u8) -> (usize, u32) {
    let plen = prefix.len();
    let total_len = plen + 1;
    let low = u64::from_le_bytes(prefix[0..8].try_into().unwrap());
    let high = u64::from_le_bytes([
        prefix[plen - 7],
        prefix[plen - 6],
        prefix[plen - 5],
        prefix[plen - 4],
        prefix[plen - 3],
        prefix[plen - 2],
        prefix[plen - 1],
        byte,
    ]);
    let mid_off = (total_len >> 1) - 4;
    let mid = u64::from_le_bytes(prefix[mid_off..mid_off + 8].try_into().unwrap());
    let mixed =
        low ^ high.rotate_left(29) ^ mid.rotate_left(13) ^ (total_len as u64).rotate_left(47);
    let h = mixed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let m = h ^ (h >> 28);
    ((m >> 52) as usize, ((m >> 44) & 0xFF) as u32)
}

#[derive(Default, Clone, Copy)]
struct NodeLinks {
    prev0: u16,
    next0: u16,
    prev1: u16,
    next1: u16,
}

#[derive(Default, Clone, Copy)]
struct Link2 {
    prev: u16,
    next: u16,
}

/// 16-byte cache-aligned AoS node (`V4_aos16`): packs all 4 links + `width: u16` + 48-bit inline-5 word
/// into ONE 16B struct (124 KiB total live across `setup`, `steps`, `assign_bottom`, `emit_bands` ->
/// 1 cache line per node access instead of 3!).
///
/// `ww: u64` layout (bits 0..16 = `width: u16`, bits 16..64 = `word48`):
///   - `tag = (ww >> 56) as u32`
///   - `tag == 0`:      empty word (bits 16..64 == 0)
///   - `tag in 1..=5`:  inline payload of `tag` bytes in bits `16..(16 + 8*tag)`, rest 0
///   - `tag in 6..=15`: inline-15 rule handle: bits `16..40` = 24-bit `rule_id (>= 257)`, exact len = `tag`
///   - `tag == 0xFF`:   long rule handle: bits `16..36` = 20-bit `rule_id`, bits `36..56` = 20-bit exact len
#[repr(C, align(16))]
#[derive(Default, Clone, Copy)]
struct Node16 {
    prev0: u16,
    next0: u16,
    prev1: u16,
    next1: u16,
    ww: u64,
}

const U48_WORD_MASK: u64 = !0xFFFFu64;
const U48_PAYLOAD_MASK: u64 = (1u64 << 40) - 1;
const U48_LONG_ID_MASK: u64 = (1u64 << 20) - 1;

#[inline(always)]
fn literal_u48(byte: u8) -> u64 {
    (1u64 << 56) | (u64::from(byte) << 16)
}

fn boxed_array_64k<T: Copy>(val: T) -> Box<[T; NARROW_NODES]> {
    vec![val; NARROW_NODES]
        .into_boxed_slice()
        .try_into()
        .ok()
        .unwrap()
}

fn boxed_array<T: Copy, const N: usize>(val: T) -> Box<[T; N]> {
    vec![val; N].into_boxed_slice().try_into().ok().unwrap()
}

struct GenCache {
    pairs: Vec<u64>,
    contents_vec: Vec<u32>,
    /// Fast 16 KiB direct-mapped table storing `(gen8 << 24) | (fp8 << 16) | code16`.
    contents_4k: Box<[u32; GEN_CONTENT_SLOTS]>,
    pair_gen: u64,
    content_gen: u32,
    content_gen8: u32,
}

impl Default for GenCache {
    fn default() -> Self {
        GenCache {
            pairs: vec![0; GEN_PAIR_SLOTS],
            contents_vec: vec![0; GEN_CONTENT_SLOTS],
            contents_4k: boxed_array(0),
            pair_gen: 0,
            content_gen: 0,
            content_gen8: 0,
        }
    }
}

impl GenCache {
    #[inline(always)]
    fn next_block(&mut self, use_pairs: bool) {
        if use_pairs {
            self.pair_gen += 1;
            if self.pair_gen >= (1u64 << 18) {
                self.pairs.fill(0);
                self.pair_gen = 1;
            }
            self.content_gen += 1;
            if self.content_gen >= (1u32 << 16) {
                self.contents_vec.fill(0);
                self.content_gen = 1;
            }
        } else {
            self.content_gen8 += 1;
            if self.content_gen8 >= 256 {
                self.contents_4k.fill(0);
                self.content_gen8 = 1;
            }
        }
    }
}

/// Safe zero-bounds-check 64K rule store (falls back to Vec if a block exceeds 65,536 rules).
struct RuleStore {
    fast: Box<[u128; NARROW_NODES]>,
    len: usize,
    overflow: Vec<u128>,
}

impl RuleStore {
    fn new() -> Self {
        RuleStore {
            fast: boxed_array_64k(0),
            len: 0,
            overflow: Vec::new(),
        }
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.len = 0;
        self.overflow.clear();
    }

    #[inline(always)]
    fn push(&mut self, val: u128) -> u32 {
        let idx = self.len;
        self.len = idx + 1;
        if idx < NARROW_NODES {
            self.fast[usize::from(idx as u16)] = val;
        } else {
            self.push_overflow(val);
        }
        idx as u32 + 257
    }

    #[cold]
    #[inline(never)]
    fn push_overflow(&mut self, val: u128) {
        self.overflow.push(val);
    }

    #[inline(always)]
    fn get(&self, id: u32) -> u128 {
        let idx = (id - 257) as usize;
        if idx < NARROW_NODES {
            self.fast[usize::from(idx as u16)]
        } else {
            self.overflow[idx - NARROW_NODES]
        }
    }

    #[inline(always)]
    fn is_fast_only(&self) -> bool {
        self.overflow.is_empty()
    }
}

struct Fixed64kSoa {
    links: Box<[NodeLinks; NARROW_NODES]>,
    links0: Box<[Link2; NARROW_NODES]>,
    links1: Box<[Link2; NARROW_NODES]>,
    widths: Box<[u32; NARROW_NODES]>,
    words128: Box<[u128; NARROW_NODES]>,
    words64: Box<[u64; NARROW_NODES]>,
    nodes16: Box<[Node16; NARROW_NODES]>,
    img64: Box<[u64; NARROW_NODES]>,
    rules: RuleStore,
    bands_64k: Box<[Band; NARROW_NODES]>,
    inline16_64k: Box<[[u8; INLINE]; NARROW_NODES]>,
    lens_64k: Box<[u8; NARROW_NODES]>,
    fstack: [u32; 64],
}

impl Fixed64kSoa {
    fn new() -> Self {
        let mut inline16_64k = boxed_array_64k([0u8; INLINE]);
        let mut lens_64k = boxed_array_64k(0u8);
        for c in 0..=255u8 {
            inline16_64k[usize::from(c)][0] = c;
            lens_64k[usize::from(c)] = 1;
        }
        Fixed64kSoa {
            links: boxed_array_64k(NodeLinks::default()),
            links0: boxed_array_64k(Link2::default()),
            links1: boxed_array_64k(Link2::default()),
            widths: boxed_array_64k(0),
            words128: boxed_array_64k(0),
            words64: boxed_array_64k(0),
            nodes16: boxed_array_64k(Node16::default()),
            img64: boxed_array_64k(0),
            rules: RuleStore::new(),
            bands_64k: boxed_array_64k(Band::default()),
            inline16_64k,
            lens_64k,
            fstack: [0u32; 64],
        }
    }
}

// ---------------------------------------------------------------------------
// Baseline #1046 join + V2 u128 branchless join + V3 u64 inline7 branchless join
// ---------------------------------------------------------------------------

#[inline(always)]
fn join_u128_base(rules: &mut Vec<[u32; 4]>, left: u128, right: u128) -> u128 {
    if left == 0 {
        return right;
    }
    if right == 0 {
        return left;
    }
    let ll = (left >> 120) as u32;
    let rl = (right >> 120) as u32;
    if ll + rl < 16 {
        return (left + (u128::from(rl) << 120)) | (right << (8 * ll));
    }
    join_u128_long_base(rules, left, right, ll, rl)
}

#[cold]
#[inline(never)]
fn join_u128_long_base(
    rules: &mut Vec<[u32; 4]>,
    left: u128,
    right: u128,
    ll: u32,
    rl: u32,
) -> u128 {
    let (lid, l_len) = if ll == 0xFF {
        (left as u32, (left >> 32) as u32)
    } else if ll == 1 {
        ((left as u8) as u32 + 1, 1)
    } else {
        let id = rules.len() as u32 + 257;
        rules.push([
            left as u32,
            (left >> 32) as u32,
            (left >> 64) as u32,
            (left >> 96) as u32,
        ]);
        (id, ll)
    };
    let (rid, r_len) = if rl == 0xFF {
        (right as u32, (right >> 32) as u32)
    } else if rl == 1 {
        ((right as u8) as u32 + 1, 1)
    } else {
        let id = rules.len() as u32 + 257;
        rules.push([
            right as u32,
            (right >> 32) as u32,
            (right >> 64) as u32,
            (right >> 96) as u32,
        ]);
        (id, rl)
    };
    let total_len = l_len + r_len;
    let id = rules.len() as u32 + 257;
    rules.push([lid, rid, total_len, u32::MAX]);
    U128_LONG_TAG | (u128::from(total_len) << 32) | u128::from(id)
}

/// Single-branch u128 join covering empty operands (even paired with LONG!) + `<= 15B` inline
/// with ZERO out-of-line calls (`~85.3%` of all steps!).
#[inline(always)]
fn join_branchless_u128(rules: &mut RuleStore, left: u128, right: u128) -> u128 {
    let ll = (left >> 120) as u32;
    let rl = (right >> 120) as u32;
    if (ll + rl < 16) | (ll.min(rl) == 0) {
        return (left + (u128::from(rl) << 120)) | (right << (8 * (ll & 15)));
    }
    join_long_u128_store(rules, left, right, ll, rl)
}

#[inline(never)]
fn join_long_u128_store(rules: &mut RuleStore, left: u128, right: u128, ll: u32, rl: u32) -> u128 {
    let (lid, l_len) = if ll == 0xFF {
        (left as u32, (left >> 32) as u32)
    } else if ll == 1 {
        ((left as u8) as u32 + 1, 1)
    } else {
        (rules.push(left), ll)
    };
    let (rid, r_len) = if rl == 0xFF {
        (right as u32, (right >> 32) as u32)
    } else if rl == 1 {
        ((right as u8) as u32 + 1, 1)
    } else {
        (rules.push(right), rl)
    };
    let total_len = l_len + r_len;
    let packed = u128::from(lid)
        | (u128::from(rid) << 32)
        | (u128::from(total_len) << 64)
        | (u128::from(u32::MAX) << 96);
    let id = rules.push(packed);
    U128_LONG_TAG | (u128::from(total_len) << 32) | u128::from(id)
}

/// V3 SoA-inline7 single-branch 64-bit join:
/// Covers empty operands (even paired with 8..15B/LONG!) + `<= 7B` inline in 64-bit instructions!
#[inline(always)]
fn join_branchless_u64(rules: &mut RuleStore, left: u64, right: u64) -> u64 {
    let ll = (left >> 56) as u32;
    let rl = (right >> 56) as u32;
    if (ll + rl < 8) | (ll.min(rl) == 0) {
        return (left + (u64::from(rl) << 56)) | (right << (8 * (ll & 7)));
    }
    join_long_u64_store(rules, left, right, ll, rl)
}

#[inline(never)]
fn join_long_u64_store(rules: &mut RuleStore, left: u64, right: u64, ll: u32, rl: u32) -> u64 {
    if ll + rl < 16 {
        let l_bytes = if ll <= 7 {
            u128::from(left & U64_PAYLOAD_MASK)
        } else {
            rules.get((left & U64_ID_MASK) as u32) & U128_PAYLOAD_MASK
        };
        let r_bytes = if rl <= 7 {
            u128::from(right & U64_PAYLOAD_MASK)
        } else {
            rules.get((right & U64_ID_MASK) as u32) & U128_PAYLOAD_MASK
        };
        let sum = ll + rl;
        let packed = l_bytes | (r_bytes << (8 * ll)) | (u128::from(sum) << 120);
        let id = rules.push(packed);
        return (u64::from(sum) << 56) | u64::from(id);
    }
    let (lid, l_len) = if ll == 0xFF {
        ((left & U64_ID_MASK) as u32, ((left >> 24) as u32))
    } else if ll == 1 {
        ((left as u8) as u32 + 1, 1)
    } else if ll <= 7 {
        let packed = u128::from(left & U64_PAYLOAD_MASK) | (u128::from(ll) << 120);
        (rules.push(packed), ll)
    } else {
        ((left & U64_ID_MASK) as u32, ll)
    };
    let (rid, r_len) = if rl == 0xFF {
        ((right & U64_ID_MASK) as u32, ((right >> 24) as u32))
    } else if rl == 1 {
        ((right as u8) as u32 + 1, 1)
    } else if rl <= 7 {
        let packed = u128::from(right & U64_PAYLOAD_MASK) | (u128::from(rl) << 120);
        (rules.push(packed), rl)
    } else {
        ((right & U64_ID_MASK) as u32, rl)
    };
    let total_len = l_len + r_len;
    let packed = u128::from(lid)
        | (u128::from(rid) << 32)
        | (u128::from(total_len) << 64)
        | (u128::from(u32::MAX) << 96);
    let id = rules.push(packed);
    U64_LONG_TAG | (u64::from(total_len) << 24) | u64::from(id)
}

/// V4 AoS16 single-branch 48-bit inline-5 join (payload in bits 16..56, tag in bits 56..64, bits 0..16 == 0).
#[inline(always)]
fn join_branchless_u48(rules: &mut RuleStore, left: u64, right: u64) -> u64 {
    let ll = (left >> 56) as u32;
    let rl = (right >> 56) as u32;
    if (ll + rl < 6) | (ll.min(rl) == 0) {
        return (left + (u64::from(rl) << 56)) | (right << (8 * (ll & 7)));
    }
    join_long_u48_store(rules, left, right, ll, rl)
}

#[inline(never)]
fn join_long_u48_store(rules: &mut RuleStore, left: u64, right: u64, ll: u32, rl: u32) -> u64 {
    if ll + rl < 16 {
        let l_bytes = if ll <= 5 {
            u128::from((left >> 16) & U48_PAYLOAD_MASK)
        } else {
            rules.get(((left >> 16) & U64_ID_MASK) as u32) & U128_PAYLOAD_MASK
        };
        let r_bytes = if rl <= 5 {
            u128::from((right >> 16) & U48_PAYLOAD_MASK)
        } else {
            rules.get(((right >> 16) & U64_ID_MASK) as u32) & U128_PAYLOAD_MASK
        };
        let sum = ll + rl;
        let packed = l_bytes | (r_bytes << (8 * ll)) | (u128::from(sum) << 120);
        let id = rules.push(packed);
        return (u64::from(sum) << 56) | (u64::from(id) << 16);
    }
    let (lid, l_len) = if ll == 0xFF {
        (
            ((left >> 16) & U48_LONG_ID_MASK) as u32,
            ((left >> 36) & U48_LONG_ID_MASK) as u32,
        )
    } else if ll == 1 {
        (((left >> 16) as u8) as u32 + 1, 1)
    } else if ll <= 5 {
        let packed = u128::from((left >> 16) & U48_PAYLOAD_MASK) | (u128::from(ll) << 120);
        (rules.push(packed), ll)
    } else {
        (((left >> 16) & U64_ID_MASK) as u32, ll)
    };
    let (rid, r_len) = if rl == 0xFF {
        (
            ((right >> 16) & U48_LONG_ID_MASK) as u32,
            ((right >> 36) & U48_LONG_ID_MASK) as u32,
        )
    } else if rl == 1 {
        (((right >> 16) as u8) as u32 + 1, 1)
    } else if rl <= 5 {
        let packed = u128::from((right >> 16) & U48_PAYLOAD_MASK) | (u128::from(rl) << 120);
        (rules.push(packed), rl)
    } else {
        (((right >> 16) & U64_ID_MASK) as u32, rl)
    };
    let total_len = l_len + r_len;
    let packed = u128::from(lid)
        | (u128::from(rid) << 32)
        | (u128::from(total_len) << 64)
        | (u128::from(u32::MAX) << 96);
    let id = rules.push(packed);
    debug_assert!(id < (1 << 20) && total_len < (1 << 20));
    U64_LONG_TAG | (u64::from(total_len) << 36) | (u64::from(id) << 16)
}

fn power_u128_base(rules: &mut Vec<[u32; 4]>, word: u128, q: u32) -> u128 {
    let (mut acc, mut base, mut e) = (0u128, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_u128_base(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_u128_base(rules, base, base);
    }
}

fn power_u128_store(rules: &mut RuleStore, word: u128, q: u32) -> u128 {
    let (mut acc, mut base, mut e) = (0u128, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_branchless_u128(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_branchless_u128(rules, base, base);
    }
}

fn power_u64_store(rules: &mut RuleStore, word: u64, q: u32) -> u64 {
    let (mut acc, mut base, mut e) = (0u64, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_branchless_u64(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_branchless_u64(rules, base, base);
    }
}

fn power_u48_store(rules: &mut RuleStore, word: u64, q: u32) -> u64 {
    let (mut acc, mut base, mut e) = (0u64, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_branchless_u48(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_branchless_u48(rules, base, base);
    }
}

#[inline(always)]
fn rules_short_payload(rules: &[[u32; 4]], id: u32) -> u128 {
    if id <= 256 {
        return u128::from(id - 1);
    }
    let r = rules[(id - 257) as usize];
    u128::from(r[0])
        | (u128::from(r[1]) << 32)
        | (u128::from(r[2]) << 64)
        | (u128::from(r[3] & 0x00ff_ffff) << 96)
}

fn append_rules_base(rules: &mut [[u32; 4]], word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
    stack.clear();
    stack.push(word);
    while let Some(id) = stack.pop() {
        if id <= 256 {
            dest.push((id - 1) as u8);
        } else {
            let idx = (id - 257) as usize;
            let tag = rules[idx][3];
            if tag >> 28 == 0 {
                let bytes = rules_short_payload(rules, id).to_le_bytes();
                dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                continue;
            }
            if tag == u32::MAX {
                rules[idx][3] = 0x8000_0000 | dest.len() as u32;
                stack.push(rules[idx][1]);
                stack.push(rules[idx][0]);
            } else {
                let start = (tag & 0x7fff_ffff) as usize;
                let len = rules[idx][2] as usize;
                dest.extend_from_within(start..start + len);
            }
        }
    }
}

/// Zero-bounds-check flat DFS expansion over `Box<[u128; 65536]>` + pre-sized `dest` + `[u32; 64]` stack.
#[inline(always)]
fn append_rules_flat_64k(
    rules: &mut [u128; NARROW_NODES],
    word: u32,
    expected_len: usize,
    fstack: &mut [u32; 64],
    spill_stack: &mut Vec<u32>,
    dest: &mut Vec<u8>,
    dest_len: &mut usize,
) {
    let start_pos = *dest_len;
    let need = start_pos + expected_len + 17;
    if dest.len() < need {
        let new_len = need.max(dest.len() * 2).max(256 * 1024);
        dest.resize(new_len, 0);
    }
    let buf = &mut dest[..need];
    let mut pos = start_pos;
    let mut sp = 0usize;

    let mut id = word;
    loop {
        loop {
            if id <= 256 {
                buf[pos] = (id - 1) as u8;
                pos += 1;
                break;
            }
            let idx = usize::from((id - 257) as u16);
            let r = rules[idx];
            let tag = (r >> 96) as u32;
            if tag >> 28 == 0 {
                let len = (tag >> 24) as usize;
                buf[pos..pos + 16].copy_from_slice(&r.to_le_bytes());
                pos += len;
                break;
            }
            if tag == u32::MAX {
                let memo = (0x8000_0000u32 | pos as u32) as u128;
                rules[idx] = (r & ((1u128 << 96) - 1)) | (memo << 96);
                let right = (r >> 32) as u32;
                if sp < 64 {
                    fstack[sp] = right;
                    sp += 1;
                } else {
                    // Cold stack spill (>64 deep; max observed on wikidata is 16)
                    spill_stack.clear();
                    spill_stack.extend_from_slice(&fstack[..64]);
                    spill_stack.push(right);
                    append_rules_flat_spill(rules, r as u32, spill_stack, buf, &mut pos);
                    debug_assert_eq!(pos, start_pos + expected_len);
                    *dest_len = pos;
                    return;
                }
                id = r as u32;
            } else {
                let s = (tag & 0x7fff_ffff) as usize;
                let len = (r >> 64) as u32 as usize;
                buf.copy_within(s..s + len, pos);
                pos += len;
                break;
            }
        }
        if sp > 0 {
            sp -= 1;
            id = fstack[sp];
        } else {
            break;
        }
    }
    debug_assert_eq!(pos, start_pos + expected_len);
    *dest_len = pos;
}

#[cold]
#[inline(never)]
fn append_rules_flat_spill(
    rules: &mut [u128; NARROW_NODES],
    first_id: u32,
    spill_stack: &mut Vec<u32>,
    buf: &mut [u8],
    pos: &mut usize,
) {
    spill_stack.push(first_id);
    while let Some(id) = spill_stack.pop() {
        if id <= 256 {
            buf[*pos] = (id - 1) as u8;
            *pos += 1;
        } else {
            let idx = usize::from((id - 257) as u16);
            let r = rules[idx];
            let tag = (r >> 96) as u32;
            if tag >> 28 == 0 {
                let len = (tag >> 24) as usize;
                buf[*pos..*pos + 16].copy_from_slice(&r.to_le_bytes());
                *pos += len;
            } else if tag == u32::MAX {
                let memo = (0x8000_0000u32 | *pos as u32) as u128;
                rules[idx] = (r & ((1u128 << 96) - 1)) | (memo << 96);
                spill_stack.push((r >> 32) as u32);
                spill_stack.push(r as u32);
            } else {
                let s = (tag & 0x7fff_ffff) as usize;
                let len = (r >> 64) as u32 as usize;
                buf.copy_within(s..s + len, *pos);
                *pos += len;
            }
        }
    }
}

#[cold]
#[inline(never)]
fn append_rules_flat_overflow(
    rules: &mut RuleStore,
    word: u32,
    expected_len: usize,
    spill_stack: &mut Vec<u32>,
    dest: &mut Vec<u8>,
    dest_len: &mut usize,
) {
    let start_pos = *dest_len;
    let need = start_pos + expected_len + 17;
    if dest.len() < need {
        dest.resize(need.max(dest.len() * 2), 0);
    }
    let mut pos = start_pos;
    spill_stack.clear();
    spill_stack.push(word);
    while let Some(id) = spill_stack.pop() {
        if id <= 256 {
            dest[pos] = (id - 1) as u8;
            pos += 1;
        } else {
            let idx = (id - 257) as usize;
            let r = rules.get(id);
            let tag = (r >> 96) as u32;
            if tag >> 28 == 0 {
                let len = (tag >> 24) as usize;
                dest[pos..pos + 16].copy_from_slice(&r.to_le_bytes());
                pos += len;
            } else if tag == u32::MAX {
                let memo = (0x8000_0000u32 | pos as u32) as u128;
                let nv = (r & ((1u128 << 96) - 1)) | (memo << 96);
                if idx < NARROW_NODES {
                    rules.fast[idx] = nv;
                } else {
                    rules.overflow[idx - NARROW_NODES] = nv;
                }
                spill_stack.push((r >> 32) as u32);
                spill_stack.push(r as u32);
            } else {
                let s = (tag & 0x7fff_ffff) as usize;
                let len = (r >> 64) as u32 as usize;
                dest.copy_within(s..s + len, pos);
                pos += len;
            }
        }
    }
    *dest_len = pos;
}

// ---------------------------------------------------------------------------
// Setup: base #1046 vs fast u128/u64 (const-generic SPLIT_LINKS & U64_WORDS)
// ---------------------------------------------------------------------------

fn setup_soa_64k_base(
    strips_raw: &[u32],
    ns: usize,
    n: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    img_buf: &mut Vec<u64>,
    soa: &mut Fixed64kSoa,
) -> (u32, u16) {
    if img_buf.len() < ns {
        img_buf.resize(ns, 0);
    }
    let img = &mut img_buf[..ns];
    let links = &mut *soa.links;
    let widths = &mut *soa.widths;
    let words = &mut *soa.words128;

    links[0] = NodeLinks::default();
    widths[0] = 0;
    words[0] = 0;

    let mut nodes = (1 + ns) as u16;
    let mut widest = 0u32;
    let mut ord0_tail = 0u16;
    let (mut term1_head, mut term1_tail) = (0u16, 0u16);
    let mut x = 0u32;

    for i in 0..ns {
        let base = 3 * i;
        let from = strips_raw[base];
        let to_rel = strips_raw[base + 1];
        let meta = strips_raw[base + 2];
        let byte = (meta & 0xFF) as u8;
        let width = meta >> 8;
        let a = usize::from(byte);
        let to = to_rel + first[a];
        let sid = (i + 1) as u16;

        widths[usize::from(sid)] = width;
        words[usize::from(sid)] = literal_u128(byte);
        widest = widest.max(width);

        let pos = write[a] as usize;
        write[a] = (pos + 1) as u32;
        img[pos] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);

        if x < from {
            let gap = from - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            widths[usize::from(gid)] = gap;
            words[usize::from(gid)] = 0;
            links[usize::from(ord0_tail)].next0 = gid;
            links[usize::from(gid)].prev0 = ord0_tail;
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                links[usize::from(term1_tail)].next1 = gid;
            }
            links[usize::from(gid)].prev1 = term1_tail;
            term1_tail = gid;
        }
        links[usize::from(ord0_tail)].next0 = sid;
        links[usize::from(sid)].prev0 = ord0_tail;
        ord0_tail = sid;
        x = from + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes;
        nodes += 1;
        widths[usize::from(gid)] = gap;
        words[usize::from(gid)] = 0;
        links[usize::from(ord0_tail)].next0 = gid;
        links[usize::from(gid)].prev0 = ord0_tail;
        ord0_tail = gid;
        if term1_tail == 0 {
            term1_head = gid;
        } else {
            links[usize::from(term1_tail)].next1 = gid;
        }
        links[usize::from(gid)].prev1 = term1_tail;
        term1_tail = gid;
    }
    let ord0_head = if ord0_tail == 0 { 0 } else { links[0].next0 };

    let mut ord1_tail = 0u16;
    let (mut term0_head, mut term0_tail) = (0u16, 0u16);
    x = 0;
    for &packed in img.iter() {
        let to = (packed & 0xF_FFFF) as u32;
        let width = ((packed >> 20) & 0xF_FFFF) as u32;
        let sid = (packed >> 40) as u16;
        if x < to {
            let gap = to - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            widths[usize::from(gid)] = gap;
            words[usize::from(gid)] = 0;
            links[usize::from(ord1_tail)].next1 = gid;
            links[usize::from(gid)].prev1 = ord1_tail;
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                links[usize::from(term0_tail)].next0 = gid;
            }
            links[usize::from(gid)].prev0 = term0_tail;
            term0_tail = gid;
        }
        links[usize::from(ord1_tail)].next1 = sid;
        links[usize::from(sid)].prev1 = ord1_tail;
        ord1_tail = sid;
        x = to + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes;
        nodes += 1;
        widths[usize::from(gid)] = gap;
        words[usize::from(gid)] = 0;
        links[usize::from(ord1_tail)].next1 = gid;
        links[usize::from(gid)].prev1 = ord1_tail;
        ord1_tail = gid;
        if term0_tail == 0 {
            term0_head = gid;
        } else {
            links[usize::from(term0_tail)].next0 = gid;
        }
        links[usize::from(gid)].prev0 = term0_tail;
        term0_tail = gid;
    }
    let ord1_head = if ord1_tail == 0 { 0 } else { links[0].next1 };

    {
        let mut prev = 0u16;
        for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
            if t == 0 {
                continue;
            }
            links[usize::from(prev)].next0 = h;
            links[usize::from(h)].prev0 = prev;
            prev = t;
        }
        links[usize::from(prev)].next0 = 0;
        links[0].prev0 = prev;
    }
    {
        let mut prev = 0u16;
        for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
            if t == 0 {
                continue;
            }
            links[usize::from(prev)].next1 = h;
            links[usize::from(h)].prev1 = prev;
            prev = t;
        }
        links[usize::from(prev)].next1 = 0;
        links[0].prev1 = prev;
    }
    (widest, nodes)
}

fn setup_soa_64k_fast<const SPLIT_LINKS: bool, const U64_WORDS: bool>(
    strips_raw: &[u32],
    ns: usize,
    n: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    soa: &mut Fixed64kSoa,
) -> (u32, u16) {
    let links = &mut *soa.links;
    let links0 = &mut *soa.links0;
    let links1 = &mut *soa.links1;
    let widths = &mut *soa.widths;
    let words128 = &mut *soa.words128;
    let words64 = &mut *soa.words64;
    let img64 = &mut *soa.img64;

    if SPLIT_LINKS {
        links0[0] = Link2::default();
        links1[0] = Link2::default();
    } else {
        links[0] = NodeLinks::default();
    }
    widths[0] = 0;
    if U64_WORDS {
        words64[0] = 0;
    } else {
        words128[0] = 0;
    }

    let term1_first = (1 + ns) as u16;
    let mut nodes = term1_first;
    let mut widest = 0u32;
    let mut ord0_tail = 0u16;
    let mut x = 0u32;

    for (idx, chunk) in strips_raw[..3 * ns].chunks_exact(3).enumerate() {
        let from = chunk[0];
        let to_rel = chunk[1];
        let meta = chunk[2];
        let byte = (meta & 0xFF) as u8;
        let width = meta >> 8;
        let a = usize::from(byte);
        let to = to_rel + first[a];
        let sid = (idx + 1) as u16;

        widths[usize::from(sid)] = width;
        if U64_WORDS {
            words64[usize::from(sid)] = literal_u64(byte);
        } else {
            words128[usize::from(sid)] = literal_u128(byte);
        }
        widest = widest.max(width);

        let pos = write[a] as u16;
        write[a] = u32::from(pos) + 1;
        img64[usize::from(pos)] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);

        if x < from {
            let gap = from - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            widths[usize::from(gid)] = gap;
            if SPLIT_LINKS {
                links0[usize::from(ord0_tail)].next = gid;
                links0[usize::from(gid)].prev = ord0_tail;
                links1[usize::from(gid)] = Link2 {
                    prev: gid.wrapping_sub(1),
                    next: gid.wrapping_add(1),
                };
            } else {
                links[usize::from(ord0_tail)].next0 = gid;
                links[usize::from(gid)] = NodeLinks {
                    prev0: ord0_tail,
                    next0: 0,
                    prev1: gid.wrapping_sub(1),
                    next1: gid.wrapping_add(1),
                };
            }
            ord0_tail = gid;
        }
        if SPLIT_LINKS {
            links0[usize::from(ord0_tail)].next = sid;
            links0[usize::from(sid)].prev = ord0_tail;
        } else {
            links[usize::from(ord0_tail)].next0 = sid;
            links[usize::from(sid)].prev0 = ord0_tail;
        }
        ord0_tail = sid;
        x = from + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes;
        nodes += 1;
        widths[usize::from(gid)] = gap;
        if SPLIT_LINKS {
            links0[usize::from(ord0_tail)].next = gid;
            links0[usize::from(gid)].prev = ord0_tail;
            links1[usize::from(gid)] = Link2 {
                prev: gid.wrapping_sub(1),
                next: gid.wrapping_add(1),
            };
        } else {
            links[usize::from(ord0_tail)].next0 = gid;
            links[usize::from(gid)] = NodeLinks {
                prev0: ord0_tail,
                next0: 0,
                prev1: gid.wrapping_sub(1),
                next1: gid.wrapping_add(1),
            };
        }
        ord0_tail = gid;
    }
    let (term1_head, term1_tail) = if nodes > term1_first {
        (term1_first, nodes - 1)
    } else {
        (0, 0)
    };
    let ord0_head = if ord0_tail == 0 {
        0
    } else if SPLIT_LINKS {
        links0[0].next
    } else {
        links[0].next0
    };

    let term0_first = nodes;
    let mut ord1_tail = 0u16;
    x = 0;
    for &packed in &img64[..ns] {
        let to = (packed & 0xF_FFFF) as u32;
        let width = ((packed >> 20) & 0xF_FFFF) as u32;
        let sid = (packed >> 40) as u16;
        if x < to {
            let gap = to - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            widths[usize::from(gid)] = gap;
            if SPLIT_LINKS {
                links1[usize::from(ord1_tail)].next = gid;
                links1[usize::from(gid)].prev = ord1_tail;
                links0[usize::from(gid)] = Link2 {
                    prev: gid.wrapping_sub(1),
                    next: gid.wrapping_add(1),
                };
            } else {
                links[usize::from(ord1_tail)].next1 = gid;
                links[usize::from(gid)] = NodeLinks {
                    prev0: gid.wrapping_sub(1),
                    next0: gid.wrapping_add(1),
                    prev1: ord1_tail,
                    next1: 0,
                };
            }
            ord1_tail = gid;
        }
        if SPLIT_LINKS {
            links1[usize::from(ord1_tail)].next = sid;
            links1[usize::from(sid)].prev = ord1_tail;
        } else {
            links[usize::from(ord1_tail)].next1 = sid;
            links[usize::from(sid)].prev1 = ord1_tail;
        }
        ord1_tail = sid;
        x = to + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes;
        nodes += 1;
        widths[usize::from(gid)] = gap;
        if SPLIT_LINKS {
            links1[usize::from(ord1_tail)].next = gid;
            links1[usize::from(gid)].prev = ord1_tail;
            links0[usize::from(gid)] = Link2 {
                prev: gid.wrapping_sub(1),
                next: gid.wrapping_add(1),
            };
        } else {
            links[usize::from(ord1_tail)].next1 = gid;
            links[usize::from(gid)] = NodeLinks {
                prev0: gid.wrapping_sub(1),
                next0: gid.wrapping_add(1),
                prev1: ord1_tail,
                next1: 0,
            };
        }
        ord1_tail = gid;
    }
    if U64_WORDS {
        words64[usize::from(term1_first)..usize::from(nodes)].fill(0);
    } else {
        words128[usize::from(term1_first)..usize::from(nodes)].fill(0);
    }
    let (term0_head, term0_tail) = if nodes > term0_first {
        (term0_first, nodes - 1)
    } else {
        (0, 0)
    };
    let ord1_head = if ord1_tail == 0 {
        0
    } else if SPLIT_LINKS {
        links1[0].next
    } else {
        links[0].next1
    };

    if SPLIT_LINKS {
        let mut prev = 0u16;
        for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
            if t == 0 {
                continue;
            }
            links0[usize::from(prev)].next = h;
            links0[usize::from(h)].prev = prev;
            prev = t;
        }
        links0[usize::from(prev)].next = 0;
        links0[0].prev = prev;

        let mut prev = 0u16;
        for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
            if t == 0 {
                continue;
            }
            links1[usize::from(prev)].next = h;
            links1[usize::from(h)].prev = prev;
            prev = t;
        }
        links1[usize::from(prev)].next = 0;
        links1[0].prev = prev;
    } else {
        let mut prev = 0u16;
        for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
            if t == 0 {
                continue;
            }
            links[usize::from(prev)].next0 = h;
            links[usize::from(h)].prev0 = prev;
            prev = t;
        }
        links[usize::from(prev)].next0 = 0;
        links[0].prev0 = prev;

        let mut prev = 0u16;
        for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
            if t == 0 {
                continue;
            }
            links[usize::from(prev)].next1 = h;
            links[usize::from(h)].prev1 = prev;
            prev = t;
        }
        links[usize::from(prev)].next1 = 0;
        links[0].prev1 = prev;
    }
    (widest, nodes)
}

// ---------------------------------------------------------------------------
// Elimination: Base #1046 vs V2 (u128) vs V3 (u64 SoA-inline7)
// ---------------------------------------------------------------------------

fn eliminate_base<const SPIRALS: bool>(soa: &mut Fixed64kSoa, rules: &mut Vec<[u32; 4]>, n: u32) {
    let links = &mut *soa.links;
    let widths = &mut *soa.widths;
    let words = &mut *soa.words128;
    rules.clear();
    let mut left = n;
    while left != 0 {
        let x = links[0].prev0;
        let y = links[0].prev1;
        let xi = usize::from(x);
        let yi = usize::from(y);
        let aw = widths[xi];
        let bw = widths[yi];
        if SPIRALS && x != y {
            if aw > bw && links[yi].prev1 == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                let head = power_u128_base(rules, words[xi], q);
                words[yi] = join_u128_base(rules, head, words[yi]);
                widths[xi] = aw - q * bw;
                left -= q * bw;
                continue;
            }
            if aw < bw && links[xi].prev0 == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                let tail = power_u128_base(rules, words[yi], q);
                words[xi] = join_u128_base(rules, words[xi], tail);
                widths[yi] = bw - q * aw;
                left -= q * aw;
                continue;
            }
        }
        let w = aw.min(bw);
        left -= w;
        if x == y {
            let xp = links[xi].prev0;
            links[usize::from(xp)].next0 = 0;
            links[0].prev0 = xp;
            let yp = links[yi].prev1;
            links[usize::from(yp)].next1 = 0;
            links[0].prev1 = yp;
            continue;
        }
        let word = join_u128_base(rules, words[xi], words[yi]);
        if aw > bw {
            let yp = links[yi].prev1;
            links[usize::from(yp)].next1 = 0;
            links[0].prev1 = yp;
            let xn = links[xi].next1;
            links[yi].prev1 = x;
            links[yi].next1 = xn;
            links[xi].next1 = y;
            links[usize::from(xn)].prev1 = y;
            words[yi] = word;
            widths[xi] = aw - bw;
        } else if aw < bw {
            let xp = links[xi].prev0;
            links[usize::from(xp)].next0 = 0;
            links[0].prev0 = xp;
            let yn = links[yi].next0;
            links[xi].prev0 = y;
            links[xi].next0 = yn;
            links[yi].next0 = x;
            links[usize::from(yn)].prev0 = x;
            words[xi] = word;
            widths[yi] = bw - aw;
        } else {
            let xp = links[xi].prev0;
            links[usize::from(xp)].next0 = 0;
            links[0].prev0 = xp;
            let yp0 = links[yi].prev0;
            let yn0 = links[yi].next0;
            links[xi].prev0 = yp0;
            links[xi].next0 = yn0;
            links[usize::from(yp0)].next0 = x;
            links[usize::from(yn0)].prev0 = x;
            let yp1 = links[yi].prev1;
            links[usize::from(yp1)].next1 = 0;
            links[0].prev1 = yp1;
            words[xi] = word;
        }
    }
}

/// Unified fast elimination:
/// - `U64_WORDS = false`: V2 `words128` + single-branch `join_branchless_u128` + `RuleStore`
/// - `U64_WORDS = true`:  V3 `words64` (SoA-inline7, 62 KiB live -> 100% L2-resident!) + 64-bit `join_branchless_u64`
/// - `LAZY_SENT = false`: base link maintenance (exact sentinel updates)
/// - `LAZY_SENT = true`:  register-resident `(x, y, aw, bw)` + lazy sentinel close
/// - `SPLIT_LINKS = true`: 4B `links0` + 4B `links1` (31 KiB/side)
fn eliminate_unified<
    const SPIRALS: bool,
    const U64_WORDS: bool,
    const LAZY_SENT: bool,
    const SPLIT_LINKS: bool,
>(
    soa: &mut Fixed64kSoa,
    n: u32,
) {
    let links = &mut *soa.links;
    let links0 = &mut *soa.links0;
    let links1 = &mut *soa.links1;
    let widths = &mut *soa.widths;
    let words128 = &mut *soa.words128;
    let words64 = &mut *soa.words64;
    let rules = &mut soa.rules;
    rules.clear();

    if !LAZY_SENT {
        let mut left = n;
        while left != 0 {
            let x = links[0].prev0;
            let y = links[0].prev1;
            let xi = usize::from(x);
            let yi = usize::from(y);
            let aw = widths[xi];
            let bw = widths[yi];
            if SPIRALS && x != y {
                if aw > bw && links[yi].prev1 == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    if U64_WORDS {
                        let head = power_u64_store(rules, words64[xi], q);
                        words64[yi] = join_branchless_u64(rules, head, words64[yi]);
                    } else {
                        let head = power_u128_store(rules, words128[xi], q);
                        words128[yi] = join_branchless_u128(rules, head, words128[yi]);
                    }
                    widths[xi] = aw - q * bw;
                    left -= q * bw;
                    continue;
                }
                if aw < bw && links[xi].prev0 == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    if U64_WORDS {
                        let tail = power_u64_store(rules, words64[yi], q);
                        words64[xi] = join_branchless_u64(rules, words64[xi], tail);
                    } else {
                        let tail = power_u128_store(rules, words128[yi], q);
                        words128[xi] = join_branchless_u128(rules, words128[xi], tail);
                    }
                    widths[yi] = bw - q * aw;
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            left -= w;
            if x == y {
                let xp = links[xi].prev0;
                links[usize::from(xp)].next0 = 0;
                links[0].prev0 = xp;
                let yp = links[yi].prev1;
                links[usize::from(yp)].next1 = 0;
                links[0].prev1 = yp;
                continue;
            }
            if U64_WORDS {
                let word = join_branchless_u64(rules, words64[xi], words64[yi]);
                if aw > bw {
                    let yp = links[yi].prev1;
                    links[usize::from(yp)].next1 = 0;
                    links[0].prev1 = yp;
                    let xn = links[xi].next1;
                    links[yi].prev1 = x;
                    links[yi].next1 = xn;
                    links[xi].next1 = y;
                    links[usize::from(xn)].prev1 = y;
                    words64[yi] = word;
                    widths[xi] = aw - bw;
                } else if aw < bw {
                    let xp = links[xi].prev0;
                    links[usize::from(xp)].next0 = 0;
                    links[0].prev0 = xp;
                    let yn = links[yi].next0;
                    links[xi].prev0 = y;
                    links[xi].next0 = yn;
                    links[yi].next0 = x;
                    links[usize::from(yn)].prev0 = x;
                    words64[xi] = word;
                    widths[yi] = bw - aw;
                } else {
                    let xp = links[xi].prev0;
                    links[usize::from(xp)].next0 = 0;
                    links[0].prev0 = xp;
                    let yp0 = links[yi].prev0;
                    let yn0 = links[yi].next0;
                    links[xi].prev0 = yp0;
                    links[xi].next0 = yn0;
                    links[usize::from(yp0)].next0 = x;
                    links[usize::from(yn0)].prev0 = x;
                    let yp1 = links[yi].prev1;
                    links[usize::from(yp1)].next1 = 0;
                    links[0].prev1 = yp1;
                    words64[xi] = word;
                }
            } else {
                let word = join_branchless_u128(rules, words128[xi], words128[yi]);
                if aw > bw {
                    let yp = links[yi].prev1;
                    links[usize::from(yp)].next1 = 0;
                    links[0].prev1 = yp;
                    let xn = links[xi].next1;
                    links[yi].prev1 = x;
                    links[yi].next1 = xn;
                    links[xi].next1 = y;
                    links[usize::from(xn)].prev1 = y;
                    words128[yi] = word;
                    widths[xi] = aw - bw;
                } else if aw < bw {
                    let xp = links[xi].prev0;
                    links[usize::from(xp)].next0 = 0;
                    links[0].prev0 = xp;
                    let yn = links[yi].next0;
                    links[xi].prev0 = y;
                    links[xi].next0 = yn;
                    links[yi].next0 = x;
                    links[usize::from(yn)].prev0 = x;
                    words128[xi] = word;
                    widths[yi] = bw - aw;
                } else {
                    let xp = links[xi].prev0;
                    links[usize::from(xp)].next0 = 0;
                    links[0].prev0 = xp;
                    let yp0 = links[yi].prev0;
                    let yn0 = links[yi].next0;
                    links[xi].prev0 = yp0;
                    links[xi].next0 = yn0;
                    links[usize::from(yp0)].next0 = x;
                    links[usize::from(yn0)].prev0 = x;
                    let yp1 = links[yi].prev1;
                    links[usize::from(yp1)].next1 = 0;
                    links[0].prev1 = yp1;
                    words128[xi] = word;
                }
            }
        }
        return;
    }

    // LAZY_SENT = true: lean register-resident (x, y, aw, bw) + single join + lazy sentinel close
    let mut left = n;
    let mut x = if SPLIT_LINKS {
        links0[0].prev
    } else {
        links[0].prev0
    };
    let mut y = if SPLIT_LINKS {
        links1[0].prev
    } else {
        links[0].prev1
    };
    let mut aw = widths[usize::from(x)];
    let mut bw = widths[usize::from(y)];

    while left != 0 {
        let xi = usize::from(x);
        let yi = usize::from(y);
        if SPIRALS && x != y {
            let yp1 = if SPLIT_LINKS {
                links1[yi].prev
            } else {
                links[yi].prev1
            };
            if aw > bw && yp1 == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                if U64_WORDS {
                    let head = power_u64_store(rules, words64[xi], q);
                    words64[yi] = join_branchless_u64(rules, head, words64[yi]);
                } else {
                    let head = power_u128_store(rules, words128[xi], q);
                    words128[yi] = join_branchless_u128(rules, head, words128[yi]);
                }
                aw -= q * bw;
                left -= q * bw;
                continue;
            }
            let xp0 = if SPLIT_LINKS {
                links0[xi].prev
            } else {
                links[xi].prev0
            };
            if aw < bw && xp0 == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                if U64_WORDS {
                    let tail = power_u64_store(rules, words64[yi], q);
                    words64[xi] = join_branchless_u64(rules, words64[xi], tail);
                } else {
                    let tail = power_u128_store(rules, words128[yi], q);
                    words128[xi] = join_branchless_u128(rules, words128[xi], tail);
                }
                bw -= q * aw;
                left -= q * aw;
                continue;
            }
        }
        if x == y {
            widths[xi] = aw;
            left -= aw;
            if SPLIT_LINKS {
                x = links0[xi].prev;
                y = links1[yi].prev;
            } else {
                x = links[xi].prev0;
                y = links[yi].prev1;
            }
            aw = widths[usize::from(x)];
            bw = widths[usize::from(y)];
            continue;
        }
        if U64_WORDS {
            let word = join_branchless_u64(rules, words64[xi], words64[yi]);
            if aw > bw {
                left -= bw;
                aw -= bw;
                words64[yi] = word;
                if SPLIT_LINKS {
                    let yp = links1[yi].prev;
                    if yp != x {
                        widths[yi] = bw;
                        let xn = links1[xi].next;
                        links1[yi] = Link2 { prev: x, next: xn };
                        links1[xi].next = y;
                        links1[usize::from(xn)].prev = y;
                        y = yp;
                        bw = widths[usize::from(y)];
                    }
                } else {
                    let yp = links[yi].prev1;
                    if yp != x {
                        widths[yi] = bw;
                        let xn = links[xi].next1;
                        links[yi].prev1 = x;
                        links[yi].next1 = xn;
                        links[xi].next1 = y;
                        links[usize::from(xn)].prev1 = y;
                        y = yp;
                        bw = widths[usize::from(y)];
                    }
                }
            } else if aw < bw {
                left -= aw;
                bw -= aw;
                words64[xi] = word;
                if SPLIT_LINKS {
                    let xp = links0[xi].prev;
                    if xp != y {
                        widths[xi] = aw;
                        let yn = links0[yi].next;
                        links0[xi] = Link2 { prev: y, next: yn };
                        links0[yi].next = x;
                        links0[usize::from(yn)].prev = x;
                        x = xp;
                        aw = widths[usize::from(x)];
                    }
                } else {
                    let xp = links[xi].prev0;
                    if xp != y {
                        widths[xi] = aw;
                        let yn = links[yi].next0;
                        links[xi].prev0 = y;
                        links[xi].next0 = yn;
                        links[yi].next0 = x;
                        links[usize::from(yn)].prev0 = x;
                        x = xp;
                        aw = widths[usize::from(x)];
                    }
                }
            } else {
                left -= aw;
                widths[xi] = aw;
                words64[xi] = word;
                if SPLIT_LINKS {
                    let xp = links0[xi].prev;
                    let yp0 = links0[yi].prev;
                    let yp1 = links1[yi].prev;
                    if xp == y {
                        links0[xi].prev = yp0;
                        links0[usize::from(yp0)].next = x;
                        y = yp1;
                        bw = widths[usize::from(y)];
                    } else {
                        let yn0 = links0[yi].next;
                        links0[xi] = Link2 {
                            prev: yp0,
                            next: yn0,
                        };
                        links0[usize::from(yp0)].next = x;
                        links0[usize::from(yn0)].prev = x;
                        x = xp;
                        y = yp1;
                        aw = widths[usize::from(x)];
                        bw = widths[usize::from(y)];
                    }
                } else {
                    let xp = links[xi].prev0;
                    let yp0 = links[yi].prev0;
                    let yp1 = links[yi].prev1;
                    if xp == y {
                        links[xi].prev0 = yp0;
                        links[usize::from(yp0)].next0 = x;
                        y = yp1;
                        bw = widths[usize::from(y)];
                    } else {
                        let yn0 = links[yi].next0;
                        links[xi].prev0 = yp0;
                        links[xi].next0 = yn0;
                        links[usize::from(yp0)].next0 = x;
                        links[usize::from(yn0)].prev0 = x;
                        x = xp;
                        y = yp1;
                        aw = widths[usize::from(x)];
                        bw = widths[usize::from(y)];
                    }
                }
            }
        } else {
            let word = join_branchless_u128(rules, words128[xi], words128[yi]);
            if aw > bw {
                left -= bw;
                aw -= bw;
                words128[yi] = word;
                if SPLIT_LINKS {
                    let yp = links1[yi].prev;
                    if yp != x {
                        widths[yi] = bw;
                        let xn = links1[xi].next;
                        links1[yi] = Link2 { prev: x, next: xn };
                        links1[xi].next = y;
                        links1[usize::from(xn)].prev = y;
                        y = yp;
                        bw = widths[usize::from(y)];
                    }
                } else {
                    let yp = links[yi].prev1;
                    if yp != x {
                        widths[yi] = bw;
                        let xn = links[xi].next1;
                        links[yi].prev1 = x;
                        links[yi].next1 = xn;
                        links[xi].next1 = y;
                        links[usize::from(xn)].prev1 = y;
                        y = yp;
                        bw = widths[usize::from(y)];
                    }
                }
            } else if aw < bw {
                left -= aw;
                bw -= aw;
                words128[xi] = word;
                if SPLIT_LINKS {
                    let xp = links0[xi].prev;
                    if xp != y {
                        widths[xi] = aw;
                        let yn = links0[yi].next;
                        links0[xi] = Link2 { prev: y, next: yn };
                        links0[yi].next = x;
                        links0[usize::from(yn)].prev = x;
                        x = xp;
                        aw = widths[usize::from(x)];
                    }
                } else {
                    let xp = links[xi].prev0;
                    if xp != y {
                        widths[xi] = aw;
                        let yn = links[yi].next0;
                        links[xi].prev0 = y;
                        links[xi].next0 = yn;
                        links[yi].next0 = x;
                        links[usize::from(yn)].prev0 = x;
                        x = xp;
                        aw = widths[usize::from(x)];
                    }
                }
            } else {
                left -= aw;
                widths[xi] = aw;
                words128[xi] = word;
                if SPLIT_LINKS {
                    let xp = links0[xi].prev;
                    let yp0 = links0[yi].prev;
                    let yp1 = links1[yi].prev;
                    if xp == y {
                        links0[xi].prev = yp0;
                        links0[usize::from(yp0)].next = x;
                        y = yp1;
                        bw = widths[usize::from(y)];
                    } else {
                        let yn0 = links0[yi].next;
                        links0[xi] = Link2 {
                            prev: yp0,
                            next: yn0,
                        };
                        links0[usize::from(yp0)].next = x;
                        links0[usize::from(yn0)].prev = x;
                        x = xp;
                        y = yp1;
                        aw = widths[usize::from(x)];
                        bw = widths[usize::from(y)];
                    }
                } else {
                    let xp = links[xi].prev0;
                    let yp0 = links[yi].prev0;
                    let yp1 = links[yi].prev1;
                    if xp == y {
                        links[xi].prev0 = yp0;
                        links[usize::from(yp0)].next0 = x;
                        y = yp1;
                        bw = widths[usize::from(y)];
                    } else {
                        let yn0 = links[yi].next0;
                        links[xi].prev0 = yp0;
                        links[xi].next0 = yn0;
                        links[usize::from(yp0)].next0 = x;
                        links[usize::from(yn0)].prev0 = x;
                        x = xp;
                        y = yp1;
                        aw = widths[usize::from(x)];
                        bw = widths[usize::from(y)];
                    }
                }
            }
        }
    }

    if SPLIT_LINKS {
        if x != 0 {
            widths[usize::from(x)] = aw;
            links0[usize::from(x)].next = 0;
        } else {
            links0[0].next = 0;
        }
        links0[0].prev = x;
        if y != 0 {
            widths[usize::from(y)] = bw;
            links1[usize::from(y)].next = 0;
        } else {
            links1[0].next = 0;
        }
        links1[0].prev = y;
    } else {
        if x != 0 {
            widths[usize::from(x)] = aw;
            links[usize::from(x)].next0 = 0;
        } else {
            links[0].next0 = 0;
        }
        links[0].prev0 = x;
        if y != 0 {
            widths[usize::from(y)] = bw;
            links[usize::from(y)].next1 = 0;
        } else {
            links[0].next1 = 0;
        }
        links[0].prev1 = y;
    }
}

// ---------------------------------------------------------------------------
// Assign bottom: combined links vs split links
// ---------------------------------------------------------------------------

fn assign_bottom_base(
    links: &mut [NodeLinks; NARROW_NODES],
    widths: &[u32; NARROW_NODES],
    rows: u32,
) {
    let mut first = links[0].next1;
    let mut last = links[0].prev1;
    let (mut lo, mut hi) = (0u32, rows);
    while first != last {
        let fi = usize::from(first);
        let li = usize::from(last);
        let next = links[fi].next1;
        let prev = links[li].prev1;
        let wf = widths[fi];
        hi -= widths[li];
        links[fi].prev1 = lo as u16;
        links[fi].next1 = (lo >> 16) as u16;
        links[li].prev1 = hi as u16;
        links[li].next1 = (hi >> 16) as u16;
        lo += wf;
        if next == last {
            assert_eq!(lo, hi);
            return;
        }
        first = next;
        last = prev;
    }
    if first != 0 {
        let fi = usize::from(first);
        links[fi].prev1 = lo as u16;
        links[fi].next1 = (lo >> 16) as u16;
        lo += widths[fi];
    }
    assert_eq!(lo, hi);
}

fn assign_bottom_split(
    links1: &mut [Link2; NARROW_NODES],
    widths: &[u32; NARROW_NODES],
    rows: u32,
) {
    let mut first = links1[0].next;
    let mut last = links1[0].prev;
    let (mut lo, mut hi) = (0u32, rows);
    while first != last {
        let fi = usize::from(first);
        let li = usize::from(last);
        let next = links1[fi].next;
        let prev = links1[li].prev;
        let wf = widths[fi];
        hi -= widths[li];
        links1[fi] = Link2 {
            prev: lo as u16,
            next: (lo >> 16) as u16,
        };
        links1[li] = Link2 {
            prev: hi as u16,
            next: (hi >> 16) as u16,
        };
        lo += wf;
        if next == last {
            assert_eq!(lo, hi);
            return;
        }
        first = next;
        last = prev;
    }
    if first != 0 {
        let fi = usize::from(first);
        links1[fi] = Link2 {
            prev: lo as u16,
            next: (lo >> 16) as u16,
        };
        lo += widths[fi];
    }
    assert_eq!(lo, hi);
}

// ---------------------------------------------------------------------------
// V4 AoS16: 16-byte cache-aligned Node16 (links + width:u16 + inline-5 word)
// ---------------------------------------------------------------------------

fn setup_aos16(
    strips_raw: &[u32],
    ns: usize,
    n: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    soa: &mut Fixed64kSoa,
) -> (u32, u16) {
    let nodes16 = &mut *soa.nodes16;
    let img64 = &mut *soa.img64;

    nodes16[0] = Node16::default();

    let term1_first = (1 + ns) as u16;
    let mut nodes = term1_first;
    let mut widest = 0u32;
    let mut ord0_tail = 0u16;
    let mut x = 0u32;

    for (idx, chunk) in strips_raw[..3 * ns].chunks_exact(3).enumerate() {
        let from = chunk[0];
        let to_rel = chunk[1];
        let meta = chunk[2];
        let byte = (meta & 0xFF) as u8;
        let width = meta >> 8;
        let a = usize::from(byte);
        let to = to_rel + first[a];
        let sid = (idx + 1) as u16;

        widest = widest.max(width);
        let pos = write[a] as u16;
        write[a] = u32::from(pos) + 1;
        img64[usize::from(pos)] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);

        if x < from {
            let gap = from - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            nodes16[usize::from(ord0_tail)].next0 = gid;
            nodes16[usize::from(gid)] = Node16 {
                prev0: ord0_tail,
                next0: 0,
                prev1: gid.wrapping_sub(1),
                next1: gid.wrapping_add(1),
                ww: u64::from(gap as u16),
            };
            ord0_tail = gid;
        }
        nodes16[usize::from(ord0_tail)].next0 = sid;
        nodes16[usize::from(sid)] = Node16 {
            prev0: ord0_tail,
            next0: 0,
            prev1: 0,
            next1: 0,
            ww: literal_u48(byte) | u64::from(width as u16),
        };
        ord0_tail = sid;
        x = from + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes;
        nodes += 1;
        nodes16[usize::from(ord0_tail)].next0 = gid;
        nodes16[usize::from(gid)] = Node16 {
            prev0: ord0_tail,
            next0: 0,
            prev1: gid.wrapping_sub(1),
            next1: gid.wrapping_add(1),
            ww: u64::from(gap as u16),
        };
        ord0_tail = gid;
    }
    let (term1_head, term1_tail) = if nodes > term1_first {
        (term1_first, nodes - 1)
    } else {
        (0, 0)
    };
    let ord0_head = if ord0_tail == 0 { 0 } else { nodes16[0].next0 };

    let term0_first = nodes;
    let mut ord1_tail = 0u16;
    x = 0;
    for &packed in &img64[..ns] {
        let to = (packed & 0xF_FFFF) as u32;
        let width = ((packed >> 20) & 0xF_FFFF) as u32;
        let sid = (packed >> 40) as u16;
        if x < to {
            let gap = to - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            nodes16[usize::from(ord1_tail)].next1 = gid;
            nodes16[usize::from(gid)] = Node16 {
                prev0: gid.wrapping_sub(1),
                next0: gid.wrapping_add(1),
                prev1: ord1_tail,
                next1: 0,
                ww: u64::from(gap as u16),
            };
            ord1_tail = gid;
        }
        nodes16[usize::from(ord1_tail)].next1 = sid;
        nodes16[usize::from(sid)].prev1 = ord1_tail;
        ord1_tail = sid;
        x = to + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes;
        nodes += 1;
        nodes16[usize::from(ord1_tail)].next1 = gid;
        nodes16[usize::from(gid)] = Node16 {
            prev0: gid.wrapping_sub(1),
            next0: gid.wrapping_add(1),
            prev1: ord1_tail,
            next1: 0,
            ww: u64::from(gap as u16),
        };
        ord1_tail = gid;
    }
    let (term0_head, term0_tail) = if nodes > term0_first {
        (term0_first, nodes - 1)
    } else {
        (0, 0)
    };
    let ord1_head = if ord1_tail == 0 { 0 } else { nodes16[0].next1 };

    {
        let mut prev = 0u16;
        for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
            if t == 0 {
                continue;
            }
            nodes16[usize::from(prev)].next0 = h;
            nodes16[usize::from(h)].prev0 = prev;
            prev = t;
        }
        nodes16[usize::from(prev)].next0 = 0;
        nodes16[0].prev0 = prev;
    }
    {
        let mut prev = 0u16;
        for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
            if t == 0 {
                continue;
            }
            nodes16[usize::from(prev)].next1 = h;
            nodes16[usize::from(h)].prev1 = prev;
            prev = t;
        }
        nodes16[usize::from(prev)].next1 = 0;
        nodes16[0].prev1 = prev;
    }
    (widest, nodes)
}

fn eliminate_aos16<const SPIRALS: bool>(soa: &mut Fixed64kSoa, n: u32) {
    let nodes = &mut *soa.nodes16;
    let rules = &mut soa.rules;
    rules.clear();

    let mut left = n;
    let mut x = nodes[0].prev0;
    let mut y = nodes[0].prev1;
    let xww = nodes[usize::from(x)].ww;
    let yww = nodes[usize::from(y)].ww;
    let mut aw = (xww & 0xFFFF) as u32;
    let mut bw = (yww & 0xFFFF) as u32;
    let mut wx = xww & U48_WORD_MASK;
    let mut wy = yww & U48_WORD_MASK;

    while left != 0 {
        let xi = usize::from(x);
        let yi = usize::from(y);
        if SPIRALS && x != y {
            if aw > bw && nodes[yi].prev1 == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                let head = power_u48_store(rules, wx, q);
                wy = join_branchless_u48(rules, head, wy);
                nodes[yi].ww = wy | u64::from(bw as u16);
                aw -= q * bw;
                left -= q * bw;
                continue;
            }
            if aw < bw && nodes[xi].prev0 == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                let tail = power_u48_store(rules, wy, q);
                wx = join_branchless_u48(rules, wx, tail);
                nodes[xi].ww = wx | u64::from(aw as u16);
                bw -= q * aw;
                left -= q * aw;
                continue;
            }
        }
        if x == y {
            nodes[xi].ww = wx | u64::from(aw as u16);
            left -= aw;
            x = nodes[xi].prev0;
            y = nodes[yi].prev1;
            let nxww = nodes[usize::from(x)].ww;
            let nyww = nodes[usize::from(y)].ww;
            aw = (nxww & 0xFFFF) as u32;
            wx = nxww & U48_WORD_MASK;
            bw = (nyww & 0xFFFF) as u32;
            wy = nyww & U48_WORD_MASK;
            continue;
        }
        let word = join_branchless_u48(rules, wx, wy);
        if aw > bw {
            left -= bw;
            aw -= bw;
            let yp = nodes[yi].prev1;
            if yp != x {
                nodes[yi].ww = word | u64::from(bw as u16);
                let xn = nodes[xi].next1;
                nodes[yi].prev1 = x;
                nodes[yi].next1 = xn;
                nodes[xi].next1 = y;
                nodes[usize::from(xn)].prev1 = y;
                y = yp;
                let nww = nodes[usize::from(y)].ww;
                bw = (nww & 0xFFFF) as u32;
                wy = nww & U48_WORD_MASK;
            } else {
                wy = word;
                nodes[yi].ww = word | u64::from(bw as u16);
            }
        } else if aw < bw {
            left -= aw;
            bw -= aw;
            let xp = nodes[xi].prev0;
            if xp != y {
                nodes[xi].ww = word | u64::from(aw as u16);
                let yn = nodes[yi].next0;
                nodes[xi].prev0 = y;
                nodes[xi].next0 = yn;
                nodes[yi].next0 = x;
                nodes[usize::from(yn)].prev0 = x;
                x = xp;
                let nww = nodes[usize::from(x)].ww;
                aw = (nww & 0xFFFF) as u32;
                wx = nww & U48_WORD_MASK;
            } else {
                wx = word;
                nodes[xi].ww = word | u64::from(aw as u16);
            }
        } else {
            left -= aw;
            nodes[xi].ww = word | u64::from(aw as u16);
            let xp = nodes[xi].prev0;
            let yp0 = nodes[yi].prev0;
            let yp1 = nodes[yi].prev1;
            if xp == y {
                wx = word;
                nodes[xi].prev0 = yp0;
                nodes[usize::from(yp0)].next0 = x;
                y = yp1;
                let nww = nodes[usize::from(y)].ww;
                bw = (nww & 0xFFFF) as u32;
                wy = nww & U48_WORD_MASK;
            } else {
                let yn0 = nodes[yi].next0;
                nodes[xi].prev0 = yp0;
                nodes[xi].next0 = yn0;
                nodes[usize::from(yp0)].next0 = x;
                nodes[usize::from(yn0)].prev0 = x;
                x = xp;
                y = yp1;
                let nxww = nodes[usize::from(x)].ww;
                let nyww = nodes[usize::from(y)].ww;
                aw = (nxww & 0xFFFF) as u32;
                wx = nxww & U48_WORD_MASK;
                bw = (nyww & 0xFFFF) as u32;
                wy = nyww & U48_WORD_MASK;
            }
        }
    }

    if x != 0 {
        nodes[usize::from(x)].ww = wx | u64::from(aw as u16);
        nodes[usize::from(x)].next0 = 0;
    } else {
        nodes[0].next0 = 0;
    }
    nodes[0].prev0 = x;
    if y != 0 {
        nodes[usize::from(y)].ww = wy | u64::from(bw as u16);
        nodes[usize::from(y)].next1 = 0;
    } else {
        nodes[0].next1 = 0;
    }
    nodes[0].prev1 = y;
}

fn assign_bottom_aos16(nodes: &mut [Node16; NARROW_NODES], rows: u32) {
    let mut first = nodes[0].next1;
    let mut last = nodes[0].prev1;
    let (mut lo, mut hi) = (0u32, rows);
    while first != last {
        let fi = usize::from(first);
        let li = usize::from(last);
        let next = nodes[fi].next1;
        let prev = nodes[li].prev1;
        let wf = (nodes[fi].ww & 0xFFFF) as u32;
        hi -= (nodes[li].ww & 0xFFFF) as u32;
        nodes[fi].prev1 = lo as u16;
        nodes[fi].next1 = (lo >> 16) as u16;
        nodes[li].prev1 = hi as u16;
        nodes[li].next1 = (hi >> 16) as u16;
        lo += wf;
        if next == last {
            assert_eq!(lo, hi);
            return;
        }
        first = next;
        last = prev;
    }
    if first != 0 {
        let fi = usize::from(first);
        nodes[fi].prev1 = lo as u16;
        nodes[fi].next1 = (lo >> 16) as u16;
        lo += (nodes[fi].ww & 0xFFFF) as u32;
    }
    assert_eq!(lo, hi);
}

// ---------------------------------------------------------------------------
// Dictionary: Base #1046 vs V2/V3/V4 gen8+fp8 `contents_4k` + zero-bounds-check 64K dict
// ---------------------------------------------------------------------------

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn dict_code_base(
    word: u128,
    byte: u8,
    rules: &mut [[u32; 4]],
    gen_cache: &mut GenCache,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
) -> u32 {
    let tag = (word >> 120) as u32;
    if tag < 16 {
        let entry_u128 = (word & U128_PAYLOAD_MASK) | (u128::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let idx = (short_u128_hash_base(entry_u128, len) as usize) & (GEN_CONTENT_SLOTS - 1);
        let g = gen_cache.content_gen;
        let slot = gen_cache.contents_vec[idx];
        if (slot >> 16) == g {
            let old = (slot & 0xFFFF) as usize;
            if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                return old as u32;
            }
        }
        let code = lens.len() as u32;
        inline16.push(entry_u128.to_le_bytes());
        lens.push(len);
        if code < 65_536 {
            gen_cache.contents_vec[idx] = (g << 16) | code;
        }
        code
    } else {
        dict_code_long_base(
            word as u32,
            byte,
            rules,
            gen_cache,
            inline16,
            lens,
            spelling,
            stack,
        )
    }
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn dict_code_long_base(
    id: u32,
    byte: u8,
    rules: &mut [[u32; 4]],
    gen_cache: &mut GenCache,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
) -> u32 {
    let key = (id << 8) | u32::from(byte);
    let p_idx = (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
    if id < (1 << 22) {
        let packed = gen_cache.pairs[p_idx];
        if (packed >> 46) == gen_cache.pair_gen && ((packed as u32) & 0x3FFF_FFFF) == key {
            return ((packed >> 30) & 0xFFFF) as u32;
        }
    }
    let start = spelling.len();
    let memoized = rules[(id - 257) as usize][3] != u32::MAX;
    append_rules_base(rules, id, stack, spelling);
    spelling.push(byte);
    let end = spelling.len();
    let entry = long_slot(start as u32, end as u32);
    let bytes = &spelling[start..end];
    let idx = (long_bytes_hash_base(bytes) as usize) & (GEN_CONTENT_SLOTS - 1);
    let g = gen_cache.content_gen;
    let slot = gen_cache.contents_vec[idx];
    if (slot >> 16) == g {
        let old = (slot & 0xFFFF) as usize;
        if lens[old] == LONG && long_word(&inline16[old], spelling) == bytes {
            if memoized {
                spelling.truncate(start);
            } else {
                spelling.pop();
            }
            let code = old as u32;
            if id < (1 << 22) && code < 65_536 {
                gen_cache.pairs[p_idx] =
                    (gen_cache.pair_gen << 46) | (u64::from(code) << 30) | u64::from(key);
            }
            return code;
        }
    }
    let code = lens.len() as u32;
    inline16.push(entry);
    lens.push(LONG);
    if code < 65_536 {
        gen_cache.contents_vec[idx] = (g << 16) | code;
    }
    if id < (1 << 22) && code < 65_536 {
        gen_cache.pairs[p_idx] =
            (gen_cache.pair_gen << 46) | (u64::from(code) << 30) | u64::from(key);
    }
    code
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn dict_code_long_memoized_64k(
    r: u128,
    tag: u32,
    byte: u8,
    gen_cache: &mut GenCache,
    inline16: &mut [[u8; INLINE]; NARROW_NODES],
    lens: &mut [u8; NARROW_NODES],
    dict_len: &mut u16,
    spelling_buf: &mut Vec<u8>,
    spelling_len: &mut usize,
) -> u32 {
    let mstart = (tag & 0x7fff_ffff) as usize;
    let rlen = (r >> 64) as u32 as usize;
    let (idx, fp8) = long_prefix_plus_byte_slot4k_fp(&spelling_buf[mstart..mstart + rlen], byte);
    let gfp = (gen_cache.content_gen8 << 8) | fp8;
    let slot = gen_cache.contents_4k[idx];
    if (slot >> 16) == gfp {
        let old = usize::from(slot as u16);
        if lens[old] == LONG {
            let ow = long_word(&inline16[old], spelling_buf);
            if ow.len() == rlen + 1
                && ow[rlen] == byte
                && ow[..rlen] == spelling_buf[mstart..mstart + rlen]
            {
                return old as u32;
            }
        }
    }
    let start = *spelling_len;
    let need = start + rlen + 1;
    if spelling_buf.len() < need {
        let new_len = need.max(spelling_buf.len() * 2).max(256 * 1024);
        spelling_buf.resize(new_len, 0);
    }
    spelling_buf.copy_within(mstart..mstart + rlen, start);
    spelling_buf[start + rlen] = byte;
    *spelling_len = need;
    let code = *dict_len;
    inline16[usize::from(code)] = long_slot(start as u32, need as u32);
    lens[usize::from(code)] = LONG;
    *dict_len = code + 1;
    gen_cache.contents_4k[idx] = (gfp << 16) | u32::from(code);
    u32::from(code)
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn dict_code_long_flat_64k(
    id: u32,
    expected_len: usize,
    byte: u8,
    rules: &mut RuleStore,
    fstack: &mut [u32; 64],
    gen_cache: &mut GenCache,
    inline16: &mut [[u8; INLINE]; NARROW_NODES],
    lens: &mut [u8; NARROW_NODES],
    dict_len: &mut u16,
    spelling_buf: &mut Vec<u8>,
    spelling_len: &mut usize,
    spill_stack: &mut Vec<u32>,
) -> u32 {
    let r = rules.get(id);
    let tag = (r >> 96) as u32;
    if tag != u32::MAX {
        return dict_code_long_memoized_64k(
            r,
            tag,
            byte,
            gen_cache,
            inline16,
            lens,
            dict_len,
            spelling_buf,
            spelling_len,
        );
    }

    let start = *spelling_len;
    if rules.is_fast_only() {
        append_rules_flat_64k(
            &mut rules.fast,
            id,
            expected_len,
            fstack,
            spill_stack,
            spelling_buf,
            spelling_len,
        );
    } else {
        append_rules_flat_overflow(
            rules,
            id,
            expected_len,
            spill_stack,
            spelling_buf,
            spelling_len,
        );
    }
    let end_prefix = *spelling_len;
    spelling_buf[end_prefix] = byte;
    let end = end_prefix + 1;
    *spelling_len = end;

    let bytes = &spelling_buf[start..end];
    let (idx, fp8) = long_bytes_slot4k_fp(bytes);
    let gfp = (gen_cache.content_gen8 << 8) | fp8;
    let slot = gen_cache.contents_4k[idx];
    if (slot >> 16) == gfp {
        let old = usize::from(slot as u16);
        if lens[old] == LONG && long_word(&inline16[old], spelling_buf) == bytes {
            *spelling_len = end_prefix;
            return old as u32;
        }
    }
    let code = *dict_len;
    inline16[usize::from(code)] = long_slot(start as u32, end as u32);
    lens[usize::from(code)] = LONG;
    *dict_len = code + 1;
    gen_cache.contents_4k[idx] = (gfp << 16) | u32::from(code);
    u32::from(code)
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn dict_code_u128_fast_64k(
    word: u128,
    byte: u8,
    rules: &mut RuleStore,
    fstack: &mut [u32; 64],
    gen_cache: &mut GenCache,
    inline16: &mut [[u8; INLINE]; NARROW_NODES],
    lens: &mut [u8; NARROW_NODES],
    dict_len: &mut u16,
    spelling_buf: &mut Vec<u8>,
    spelling_len: &mut usize,
    spill_stack: &mut Vec<u32>,
) -> u32 {
    let tag = (word >> 120) as u32;
    if tag < 8 {
        let key = (word as u64) | (u64::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let (idx, fp8) = fast_u64_slot4k_fp(key, len);
        let gfp = (gen_cache.content_gen8 << 8) | fp8;
        let slot = gen_cache.contents_4k[idx];
        if (slot >> 16) == gfp {
            let old = usize::from(slot as u16);
            if lens[old] == len {
                let s = inline16[old];
                let old_lo = u64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]);
                if old_lo == key {
                    return old as u32;
                }
            }
        }
        let code = *dict_len;
        inline16[usize::from(code)] = u128::from(key).to_le_bytes();
        lens[usize::from(code)] = len;
        *dict_len = code + 1;
        gen_cache.contents_4k[idx] = (gfp << 16) | u32::from(code);
        u32::from(code)
    } else if tag < 16 {
        let entry_u128 = (word & U128_PAYLOAD_MASK) | (u128::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let (idx, fp8) = fast_u128_slot4k_fp(entry_u128, len);
        let gfp = (gen_cache.content_gen8 << 8) | fp8;
        let slot = gen_cache.contents_4k[idx];
        if (slot >> 16) == gfp {
            let old = usize::from(slot as u16);
            if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                return old as u32;
            }
        }
        let code = *dict_len;
        inline16[usize::from(code)] = entry_u128.to_le_bytes();
        lens[usize::from(code)] = len;
        *dict_len = code + 1;
        gen_cache.contents_4k[idx] = (gfp << 16) | u32::from(code);
        u32::from(code)
    } else {
        dict_code_long_flat_64k(
            word as u32,
            (word >> 32) as usize,
            byte,
            rules,
            fstack,
            gen_cache,
            inline16,
            lens,
            dict_len,
            spelling_buf,
            spelling_len,
            spill_stack,
        )
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn dict_code_u64_fast_64k(
    word: u64,
    byte: u8,
    rules: &mut RuleStore,
    fstack: &mut [u32; 64],
    gen_cache: &mut GenCache,
    inline16: &mut [[u8; INLINE]; NARROW_NODES],
    lens: &mut [u8; NARROW_NODES],
    dict_len: &mut u16,
    spelling_buf: &mut Vec<u8>,
    spelling_len: &mut usize,
    spill_stack: &mut Vec<u32>,
) -> u32 {
    let tag = (word >> 56) as u32;
    if tag < 8 {
        let key = (word & U64_PAYLOAD_MASK) | (u64::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let (idx, fp8) = fast_u64_slot4k_fp(key, len);
        let gfp = (gen_cache.content_gen8 << 8) | fp8;
        let slot = gen_cache.contents_4k[idx];
        if (slot >> 16) == gfp {
            let old = usize::from(slot as u16);
            if lens[old] == len {
                let s = inline16[old];
                let old_lo = u64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]);
                if old_lo == key {
                    return old as u32;
                }
            }
        }
        let code = *dict_len;
        inline16[usize::from(code)] = u128::from(key).to_le_bytes();
        lens[usize::from(code)] = len;
        *dict_len = code + 1;
        gen_cache.contents_4k[idx] = (gfp << 16) | u32::from(code);
        u32::from(code)
    } else if tag < 16 {
        let r = rules.get((word & U64_ID_MASK) as u32);
        let entry_u128 = (r & U128_PAYLOAD_MASK) | (u128::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let (idx, fp8) = fast_u128_slot4k_fp(entry_u128, len);
        let gfp = (gen_cache.content_gen8 << 8) | fp8;
        let slot = gen_cache.contents_4k[idx];
        if (slot >> 16) == gfp {
            let old = usize::from(slot as u16);
            if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                return old as u32;
            }
        }
        let code = *dict_len;
        inline16[usize::from(code)] = entry_u128.to_le_bytes();
        lens[usize::from(code)] = len;
        *dict_len = code + 1;
        gen_cache.contents_4k[idx] = (gfp << 16) | u32::from(code);
        u32::from(code)
    } else {
        dict_code_long_flat_64k(
            (word & U64_ID_MASK) as u32,
            ((word >> 24) as u32) as usize,
            byte,
            rules,
            fstack,
            gen_cache,
            inline16,
            lens,
            dict_len,
            spelling_buf,
            spelling_len,
            spill_stack,
        )
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn dict_code_u48_fast_64k(
    word: u64,
    byte: u8,
    rules: &mut RuleStore,
    fstack: &mut [u32; 64],
    gen_cache: &mut GenCache,
    inline16: &mut [[u8; INLINE]; NARROW_NODES],
    lens: &mut [u8; NARROW_NODES],
    dict_len: &mut u16,
    spelling_buf: &mut Vec<u8>,
    spelling_len: &mut usize,
    spill_stack: &mut Vec<u32>,
) -> u32 {
    let tag = (word >> 56) as u32;
    if tag < 6 {
        let key = ((word >> 16) & U48_PAYLOAD_MASK) | (u64::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let (idx, fp8) = fast_u64_slot4k_fp(key, len);
        let gfp = (gen_cache.content_gen8 << 8) | fp8;
        let slot = gen_cache.contents_4k[idx];
        if (slot >> 16) == gfp {
            let old = usize::from(slot as u16);
            if lens[old] == len {
                let s = inline16[old];
                let old_lo = u64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]);
                if old_lo == key {
                    return old as u32;
                }
            }
        }
        let code = *dict_len;
        inline16[usize::from(code)] = u128::from(key).to_le_bytes();
        lens[usize::from(code)] = len;
        *dict_len = code + 1;
        gen_cache.contents_4k[idx] = (gfp << 16) | u32::from(code);
        u32::from(code)
    } else if tag < 16 {
        let r = rules.get(((word >> 16) & U64_ID_MASK) as u32);
        let entry_u128 = (r & U128_PAYLOAD_MASK) | (u128::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let (idx, fp8) = fast_u128_slot4k_fp(entry_u128, len);
        let gfp = (gen_cache.content_gen8 << 8) | fp8;
        let slot = gen_cache.contents_4k[idx];
        if (slot >> 16) == gfp {
            let old = usize::from(slot as u16);
            if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                return old as u32;
            }
        }
        let code = *dict_len;
        inline16[usize::from(code)] = entry_u128.to_le_bytes();
        lens[usize::from(code)] = len;
        *dict_len = code + 1;
        gen_cache.contents_4k[idx] = (gfp << 16) | u32::from(code);
        u32::from(code)
    } else {
        dict_code_long_flat_64k(
            ((word >> 16) & U48_LONG_ID_MASK) as u32,
            ((word >> 36) & U48_LONG_ID_MASK) as usize,
            byte,
            rules,
            fstack,
            gen_cache,
            inline16,
            lens,
            dict_len,
            spelling_buf,
            spelling_len,
            spill_stack,
        )
    }
}

// ---------------------------------------------------------------------------
// Arms & ProbeKernel
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    /// Exact #1046 baseline (`comb_nobc_soa_gen`)
    Base1046,
    /// V2 u128 + zero-call empty join + base sentinel (8B) + tag<8 64b dict + gen8+fp8 + 64K dict/bands/rules
    V2U128Base,
    /// V2 u128 + zero-call empty join + lazy sentinel (8B) + tag<8 64b dict + gen8+fp8 + 64K dict/bands/rules
    V2U128Opt,
    /// V3 SoA-inline7 u64 + zero-call empty join + lazy sentinel (8B) + gen8+fp8 + 64K dict/bands/rules
    V3I7Opt,
    /// V3 SoA-inline7 u64 + zero-call empty join + lazy sentinel (split 4B) + gen8+fp8 + 64K dict/bands/rules
    V3I7Spl4Opt,
    /// V4 AoS16 16-byte cache-aligned Node16 (negative control: inline-5 spills & 16B link stride)
    V4Aos16,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Base1046 => "base_1046",
            Arm::V2U128Base => "V2_u128_base",
            Arm::V2U128Opt => "V2_u128_lazy",
            Arm::V3I7Opt => "V3_i7_lazy",
            Arm::V3I7Spl4Opt => "V3_i7_spl4",
            Arm::V4Aos16 => "V4_aos16",
        }
    }

    fn split_links(self) -> bool {
        matches!(self, Arm::V3I7Spl4Opt)
    }

    fn u64_words(self) -> bool {
        matches!(self, Arm::V3I7Opt | Arm::V3I7Spl4Opt)
    }
}

#[derive(Default)]
struct ProbeKernel {
    spill: Spill,
    edges: Vec<u32>,
    extra: Vec<u32>,
    bands: Vec<Band>,
    bands_len: usize,
    use_bands_64k: bool,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    dict_len: usize,
    use_dict_64k: bool,
    spelling: Vec<u8>,
    spelling_len: usize,
    stack: Vec<u32>,
    rules_base: Vec<[u32; 4]>,
    img_buf: Vec<u64>,
    soa: Option<Fixed64kSoa>,
    gen_cache: GenCache,
    sampled: SampledScratch,
    emit_arena: Vec<u8>,
}

#[derive(Default, Clone, Copy)]
struct PhaseNs {
    a6a: u64,
    setup: u64,
    steps: u64,
    bottom: u64,
    dict: u64,
    a6c: u64,
    a7: u64,
}

#[derive(Default, Clone, Copy)]
struct DictStats {
    dict_entries: usize,
    long_entries: usize,
    spelling_bytes: usize,
}

impl ProbeKernel {
    fn emit_bands_base(&mut self, thin: &[u32; 256], n: u32) -> [usize; 256] {
        self.gen_cache.next_block(true);
        self.spelling.clear();
        begin_inline(
            if n >= 65_536 { 1536 } else { 0 },
            &mut self.inline16,
            &mut self.lens,
        );
        self.bands.clear();
        self.use_bands_64k = false;
        self.use_dict_64k = false;

        let mut cursors = [0usize; 256];
        let soa = self.soa.as_ref().unwrap();
        let links = &*soa.links;
        let widths = &*soa.widths;
        let words = &*soa.words128;
        let rules = &mut self.rules_base[..];

        let mut id = links[0].next0;
        let mut node_width = widths[usize::from(id)];
        let mut node_word = words[usize::from(id)];
        let l0 = links[usize::from(id)];
        let mut node_bottom = u32::from(l0.prev1) | (u32::from(l0.next1) << 16);
        let mut node_next = l0.next0;
        let mut in_node = 0u32;
        let mut k = 0u32;

        for a in 0..256 {
            cursors[a] = self.bands.len();
            let end = k + thin[a];
            while k < end {
                let width = (end - k).min(node_width - in_node);
                let dst = node_bottom + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    dict_code_base(
                        node_word,
                        a as u8,
                        rules,
                        &mut self.gen_cache,
                        &mut self.inline16,
                        &mut self.lens,
                        &mut self.spelling,
                        &mut self.stack,
                    )
                };
                self.bands.push(Band::new(k + width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = node_next;
                    in_node = 0;
                    if id != 0 {
                        let l = links[usize::from(id)];
                        node_width = widths[usize::from(id)];
                        node_word = words[usize::from(id)];
                        node_bottom = u32::from(l.prev1) | (u32::from(l.next1) << 16);
                        node_next = l.next0;
                    }
                }
            }
        }
        assert_eq!(id, 0);
        self.spelling_len = self.spelling.len();
        self.bands_len = self.bands.len();
        self.dict_len = self.lens.len();
        cursors
    }

    fn emit_bands_fast<const SPLIT_LINKS: bool, const U64_WORDS: bool>(
        &mut self,
        thin: &[u32; 256],
        nodes: u16,
    ) -> [usize; 256] {
        assert!(
            usize::from(nodes) + 512 < NARROW_NODES,
            "exceeded 64K narrow dict/bands capacity"
        );
        self.gen_cache.next_block(false);
        self.spelling_len = 0;
        self.use_bands_64k = true;
        self.use_dict_64k = true;

        let mut cursors = [0usize; 256];
        let soa = self.soa.as_mut().unwrap();
        let links = &*soa.links;
        let links0 = &*soa.links0;
        let links1 = &*soa.links1;
        let widths = &*soa.widths;
        let words128 = &*soa.words128;
        let words64 = &*soa.words64;
        let rules = &mut soa.rules;
        let bands_64k = &mut *soa.bands_64k;
        let inline16_64k = &mut *soa.inline16_64k;
        let lens_64k = &mut *soa.lens_64k;
        let fstack = &mut soa.fstack;
        let mut blen = 0u16;
        let mut dlen = 256u16;

        let mut id = if SPLIT_LINKS {
            links0[0].next
        } else {
            links[0].next0
        };
        let mut node_width = widths[usize::from(id)];
        let mut nw128 = if U64_WORDS {
            0u128
        } else {
            words128[usize::from(id)]
        };
        let mut nw64 = if U64_WORDS {
            words64[usize::from(id)]
        } else {
            0u64
        };
        let (mut node_bottom, mut node_next) = if SPLIT_LINKS {
            let l1 = links1[usize::from(id)];
            (
                u32::from(l1.prev) | (u32::from(l1.next) << 16),
                links0[usize::from(id)].next,
            )
        } else {
            let l0 = links[usize::from(id)];
            (u32::from(l0.prev1) | (u32::from(l0.next1) << 16), l0.next0)
        };
        let mut in_node = 0u32;
        let mut k = 0u32;

        for a in 0..256 {
            cursors[a] = usize::from(blen);
            let end = k + thin[a];
            while k < end {
                let rem_node = node_width - in_node;
                let rem_a = end - k;
                let dst = node_bottom + in_node;
                let code = if U64_WORDS {
                    if nw64 == 0 {
                        a as u32
                    } else {
                        dict_code_u64_fast_64k(
                            nw64,
                            a as u8,
                            rules,
                            fstack,
                            &mut self.gen_cache,
                            inline16_64k,
                            lens_64k,
                            &mut dlen,
                            &mut self.spelling,
                            &mut self.spelling_len,
                            &mut self.stack,
                        )
                    }
                } else if nw128 == 0 {
                    a as u32
                } else {
                    dict_code_u128_fast_64k(
                        nw128,
                        a as u8,
                        rules,
                        fstack,
                        &mut self.gen_cache,
                        inline16_64k,
                        lens_64k,
                        &mut dlen,
                        &mut self.spelling,
                        &mut self.spelling_len,
                        &mut self.stack,
                    )
                };
                if rem_node <= rem_a {
                    k += rem_node;
                    bands_64k[usize::from(blen)] = Band::new(k, dst, code);
                    blen += 1;
                    id = node_next;
                    in_node = 0;
                    if id != 0 {
                        node_width = widths[usize::from(id)];
                        if U64_WORDS {
                            nw64 = words64[usize::from(id)];
                        } else {
                            nw128 = words128[usize::from(id)];
                        }
                        if SPLIT_LINKS {
                            let l1 = links1[usize::from(id)];
                            node_bottom = u32::from(l1.prev) | (u32::from(l1.next) << 16);
                            node_next = links0[usize::from(id)].next;
                        } else {
                            let l0 = links[usize::from(id)];
                            node_bottom = u32::from(l0.prev1) | (u32::from(l0.next1) << 16);
                            node_next = l0.next0;
                        }
                    }
                } else {
                    k = end;
                    bands_64k[usize::from(blen)] = Band::new(k, dst, code);
                    blen += 1;
                    in_node += rem_a;
                }
            }
        }
        assert_eq!(id, 0);
        self.bands_len = usize::from(blen);
        self.dict_len = usize::from(dlen);
        cursors
    }

    fn emit_bands_aos16(&mut self, thin: &[u32; 256], nodes: u16) -> [usize; 256] {
        assert!(
            usize::from(nodes) + 512 < NARROW_NODES,
            "exceeded 64K narrow dict/bands capacity"
        );
        self.gen_cache.next_block(false);
        self.spelling_len = 0;
        self.use_bands_64k = true;
        self.use_dict_64k = true;

        let mut cursors = [0usize; 256];
        let soa = self.soa.as_mut().unwrap();
        let nodes16 = &*soa.nodes16;
        let rules = &mut soa.rules;
        let bands_64k = &mut *soa.bands_64k;
        let inline16_64k = &mut *soa.inline16_64k;
        let lens_64k = &mut *soa.lens_64k;
        let fstack = &mut soa.fstack;
        let mut blen = 0u16;
        let mut dlen = 256u16;

        let mut id = nodes16[0].next0;
        let n0 = nodes16[usize::from(id)];
        let mut node_width = (n0.ww & 0xFFFF) as u32;
        let mut node_word = n0.ww & U48_WORD_MASK;
        let mut node_bottom = u32::from(n0.prev1) | (u32::from(n0.next1) << 16);
        let mut node_next = n0.next0;
        let mut in_node = 0u32;
        let mut k = 0u32;

        for a in 0..256 {
            cursors[a] = usize::from(blen);
            let end = k + thin[a];
            while k < end {
                let rem_node = node_width - in_node;
                let rem_a = end - k;
                let dst = node_bottom + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    dict_code_u48_fast_64k(
                        node_word,
                        a as u8,
                        rules,
                        fstack,
                        &mut self.gen_cache,
                        inline16_64k,
                        lens_64k,
                        &mut dlen,
                        &mut self.spelling,
                        &mut self.spelling_len,
                        &mut self.stack,
                    )
                };
                if rem_node <= rem_a {
                    k += rem_node;
                    bands_64k[usize::from(blen)] = Band::new(k, dst, code);
                    blen += 1;
                    id = node_next;
                    in_node = 0;
                    if id != 0 {
                        let nd = nodes16[usize::from(id)];
                        node_width = (nd.ww & 0xFFFF) as u32;
                        node_word = nd.ww & U48_WORD_MASK;
                        node_bottom = u32::from(nd.prev1) | (u32::from(nd.next1) << 16);
                        node_next = nd.next0;
                    }
                } else {
                    k = end;
                    bands_64k[usize::from(blen)] = Band::new(k, dst, code);
                    blen += 1;
                    in_node += rem_a;
                }
            }
        }
        assert_eq!(id, 0);
        self.bands_len = usize::from(blen);
        self.dict_len = usize::from(dlen);
        cursors
    }

    #[inline(never)]
    fn run_a6b_only(&mut self, inp: &A6bInput, arm: Arm, ph: &mut PhaseNs) -> [usize; 256] {
        let t1 = Instant::now();
        let n = inp.n;
        let ns = inp.ns;
        let mut write = inp.write_init;

        let soa = self.soa.get_or_insert_with(Fixed64kSoa::new);
        let mut used_aos16 = false;
        let (widest, nodes) = match arm {
            Arm::Base1046 => setup_soa_64k_base(
                &inp.strips_raw,
                ns,
                n,
                &inp.first,
                &mut write,
                &mut self.img_buf,
                soa,
            ),
            Arm::V2U128Base | Arm::V2U128Opt => setup_soa_64k_fast::<false, false>(
                &inp.strips_raw,
                ns,
                n,
                &inp.first,
                &mut write,
                soa,
            ),
            Arm::V3I7Opt => setup_soa_64k_fast::<false, true>(
                &inp.strips_raw,
                ns,
                n,
                &inp.first,
                &mut write,
                soa,
            ),
            Arm::V3I7Spl4Opt => setup_soa_64k_fast::<true, true>(
                &inp.strips_raw,
                ns,
                n,
                &inp.first,
                &mut write,
                soa,
            ),
            Arm::V4Aos16 => {
                let (w, nd) = setup_aos16(&inp.strips_raw, ns, n, &inp.first, &mut write, soa);
                if w <= 65_535 {
                    used_aos16 = true;
                    (w, nd)
                } else {
                    let mut w2 = inp.write_init;
                    setup_soa_64k_fast::<false, true>(
                        &inp.strips_raw,
                        ns,
                        n,
                        &inp.first,
                        &mut w2,
                        soa,
                    )
                }
            }
        };
        let t2 = Instant::now();

        let spirals = widest > CERTIFIED_WIDTH;
        let soa = self.soa.as_mut().unwrap();
        match (arm, spirals, used_aos16) {
            (Arm::Base1046, false, _) => eliminate_base::<false>(soa, &mut self.rules_base, n),
            (Arm::Base1046, true, _) => eliminate_base::<true>(soa, &mut self.rules_base, n),
            (Arm::V2U128Base, false, _) => eliminate_unified::<false, false, false, false>(soa, n),
            (Arm::V2U128Base, true, _) => eliminate_unified::<true, false, false, false>(soa, n),
            (Arm::V2U128Opt, false, _) => eliminate_unified::<false, false, true, false>(soa, n),
            (Arm::V2U128Opt, true, _) => eliminate_unified::<true, false, true, false>(soa, n),
            (Arm::V3I7Opt, false, _) => eliminate_unified::<false, true, true, false>(soa, n),
            (Arm::V3I7Opt, true, _) => eliminate_unified::<true, true, true, false>(soa, n),
            (Arm::V3I7Spl4Opt, false, _) => eliminate_unified::<false, true, true, true>(soa, n),
            (Arm::V3I7Spl4Opt, true, _) => eliminate_unified::<true, true, true, true>(soa, n),
            (Arm::V4Aos16, false, true) => eliminate_aos16::<false>(soa, n),
            (Arm::V4Aos16, true, true) => eliminate_aos16::<true>(soa, n),
            (Arm::V4Aos16, false, false) => eliminate_unified::<false, true, true, false>(soa, n),
            (Arm::V4Aos16, true, false) => eliminate_unified::<true, true, true, false>(soa, n),
        }
        let t3 = Instant::now();

        let soa = self.soa.as_mut().unwrap();
        if used_aos16 {
            assign_bottom_aos16(&mut soa.nodes16, inp.m);
        } else if arm.split_links() {
            assign_bottom_split(&mut soa.links1, &soa.widths, inp.m);
        } else {
            assign_bottom_base(&mut soa.links, &soa.widths, inp.m);
        }
        let t4 = Instant::now();

        let cursors = if arm == Arm::Base1046 {
            self.emit_bands_base(&inp.thin, n)
        } else if used_aos16 {
            self.emit_bands_aos16(&inp.thin, nodes)
        } else if arm.split_links() {
            self.emit_bands_fast::<true, true>(&inp.thin, nodes)
        } else if arm.u64_words() || arm == Arm::V4Aos16 {
            self.emit_bands_fast::<false, true>(&inp.thin, nodes)
        } else {
            self.emit_bands_fast::<false, false>(&inp.thin, nodes)
        };
        let t5 = Instant::now();

        ph.setup += (t2 - t1).as_nanos() as u64;
        ph.steps += (t3 - t2).as_nanos() as u64;
        ph.bottom += (t4 - t3).as_nanos() as u64;
        ph.dict += (t5 - t4).as_nanos() as u64;
        cursors
    }

    #[inline(never)]
    fn run_block(
        &mut self,
        b: &BlockInput,
        arm: Arm,
        out_queues: &mut Vec<u8>,
        ph: &mut PhaseNs,
        ds: &mut DictStats,
        force_spirals: Option<bool>,
    ) {
        let t0 = Instant::now();
        out_queues.resize(b.n + INLINE, 0);
        let mut pass = Pass {
            queues: Queues::new(out_queues, &mut self.spill, u32::MAX),
            edges: &mut self.edges,
            ns: 0,
            cap_strips: usize::MAX,
            f: First::default(),
        };
        maximal(&b.records, |c, pos, width| {
            pass.take(c, pos, width, b.orig_ptr, PROD_LIMIT);
        });
        let heads = pass.queues.heads();
        let ns = pass.ns;
        let f = pass.f;
        let t1 = Instant::now();

        let n = b.n as u32;
        let all: [u32; 256] = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        let first = starts(all);
        let write_init = starts(f.thick);
        let mut write = write_init;
        let most = 3 * ns + 3;
        assert!(most <= NARROW_NODES, "block exceeded 64K narrow nodes");

        let soa = self.soa.get_or_insert_with(Fixed64kSoa::new);
        let strips_raw = &self.edges[..3 * ns];
        let mut used_aos16 = false;
        let (widest, nodes) = match arm {
            Arm::Base1046 => setup_soa_64k_base(
                strips_raw,
                ns,
                n,
                &first,
                &mut write,
                &mut self.img_buf,
                soa,
            ),
            Arm::V2U128Base | Arm::V2U128Opt => {
                setup_soa_64k_fast::<false, false>(strips_raw, ns, n, &first, &mut write, soa)
            }
            Arm::V3I7Opt => {
                setup_soa_64k_fast::<false, true>(strips_raw, ns, n, &first, &mut write, soa)
            }
            Arm::V3I7Spl4Opt => {
                setup_soa_64k_fast::<true, true>(strips_raw, ns, n, &first, &mut write, soa)
            }
            Arm::V4Aos16 => {
                let (w, nd) = setup_aos16(strips_raw, ns, n, &first, &mut write, soa);
                if w <= 65_535 {
                    used_aos16 = true;
                    (w, nd)
                } else {
                    let mut w2 = write_init;
                    setup_soa_64k_fast::<false, true>(strips_raw, ns, n, &first, &mut w2, soa)
                }
            }
        };
        let t2 = Instant::now();

        let spirals = force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
        let soa = self.soa.as_mut().unwrap();
        match (arm, spirals, used_aos16) {
            (Arm::Base1046, false, _) => eliminate_base::<false>(soa, &mut self.rules_base, n),
            (Arm::Base1046, true, _) => eliminate_base::<true>(soa, &mut self.rules_base, n),
            (Arm::V2U128Base, false, _) => eliminate_unified::<false, false, false, false>(soa, n),
            (Arm::V2U128Base, true, _) => eliminate_unified::<true, false, false, false>(soa, n),
            (Arm::V2U128Opt, false, _) => eliminate_unified::<false, false, true, false>(soa, n),
            (Arm::V2U128Opt, true, _) => eliminate_unified::<true, false, true, false>(soa, n),
            (Arm::V3I7Opt, false, _) => eliminate_unified::<false, true, true, false>(soa, n),
            (Arm::V3I7Opt, true, _) => eliminate_unified::<true, true, true, false>(soa, n),
            (Arm::V3I7Spl4Opt, false, _) => eliminate_unified::<false, true, true, true>(soa, n),
            (Arm::V3I7Spl4Opt, true, _) => eliminate_unified::<true, true, true, true>(soa, n),
            (Arm::V4Aos16, false, true) => eliminate_aos16::<false>(soa, n),
            (Arm::V4Aos16, true, true) => eliminate_aos16::<true>(soa, n),
            (Arm::V4Aos16, false, false) => eliminate_unified::<false, true, true, false>(soa, n),
            (Arm::V4Aos16, true, false) => eliminate_unified::<true, true, true, false>(soa, n),
        }
        let t3 = Instant::now();

        let soa = self.soa.as_mut().unwrap();
        if used_aos16 {
            assign_bottom_aos16(&mut soa.nodes16, f.m);
        } else if arm.split_links() {
            assign_bottom_split(&mut soa.links1, &soa.widths, f.m);
        } else {
            assign_bottom_base(&mut soa.links, &soa.widths, f.m);
        }
        let t4 = Instant::now();

        let cursors = if arm == Arm::Base1046 {
            self.emit_bands_base(&f.thin, n)
        } else if used_aos16 {
            self.emit_bands_aos16(&f.thin, nodes)
        } else if arm.split_links() {
            self.emit_bands_fast::<true, true>(&f.thin, nodes)
        } else if arm.u64_words() || arm == Arm::V4Aos16 {
            self.emit_bands_fast::<false, true>(&f.thin, nodes)
        } else {
            self.emit_bands_fast::<false, false>(&f.thin, nodes)
        };
        let t5 = Instant::now();

        let dict_len = self.dict_len;
        let (inline_slice, lens_slice): (&[[u8; INLINE]], &[u8]) = if self.use_dict_64k {
            let soa = self.soa.as_ref().unwrap();
            (&soa.inline16_64k[..dict_len], &soa.lens_64k[..dict_len])
        } else {
            (&self.inline16[..dict_len], &self.lens[..dict_len])
        };

        ds.dict_entries += dict_len - 256;
        ds.long_entries += lens_slice[256..].iter().filter(|&&l| l == LONG).count();
        ds.spelling_bytes += self.spelling_len;

        // --- A6c ---
        let m = f.m as usize;
        if self.edges.len() < m {
            self.edges.resize(m, 0);
        }
        let bits = m.max(2).next_power_of_two().trailing_zeros();
        let wide = (dict_len - 1) > (u32::MAX >> bits) as usize;
        if wide {
            self.extra.resize(m, 0);
        } else {
            self.extra.clear();
        }
        let mut anchors = [0u32; MAX_ANCHORS];
        let bands_slice: &[Band] = if self.use_bands_64k {
            &self.soa.as_ref().unwrap().bands_64k[..self.bands_len]
        } else {
            &self.bands[..self.bands_len]
        };
        for a in 0..256 {
            let mut reader = QueueReader::new(out_queues, &self.spill, &heads, a);
            let b_start = cursors[a];
            let b_end = if a + 1 < 256 {
                cursors[a + 1]
            } else {
                bands_slice.len()
            };
            let mut prev_end = if b_start == 0 {
                0
            } else {
                bands_slice[b_start - 1].end()
            };
            for &band in &bands_slice[b_start..b_end] {
                let end = band.end();
                let span = (end - prev_end) as usize;
                let dst = band.dst() as usize;
                let code = band.code();
                if wide {
                    reader.wire(&mut self.edges[dst..dst + span], 0);
                    self.extra[dst..dst + span].fill(code);
                } else {
                    reader.wire(&mut self.edges[dst..dst + span], code << bits);
                }
                prev_end = end;
            }
        }
        let n_anchors = tag_uniform_anchors(
            &mut self.edges,
            m,
            f.mark,
            bits,
            wide,
            dict_len,
            &mut anchors,
        );
        let t6 = Instant::now();

        // --- A7 (4-lane forward raw emit) ---
        emit_4lane_raw(
            &mut self.edges,
            &self.extra,
            &self.spelling[..self.spelling_len],
            inline_slice,
            lens_slice,
            b.n,
            m,
            f.mark,
            bits,
            &anchors[..n_anchors],
            &mut self.emit_arena,
            &mut self.sampled,
        );
        let t7 = Instant::now();

        ph.a6a += (t1 - t0).as_nanos() as u64;
        ph.setup += (t2 - t1).as_nanos() as u64;
        ph.steps += (t3 - t2).as_nanos() as u64;
        ph.bottom += (t4 - t3).as_nanos() as u64;
        ph.dict += (t5 - t4).as_nanos() as u64;
        ph.a6c += (t6 - t5).as_nanos() as u64;
        ph.a7 += (t7 - t6).as_nanos() as u64;
    }

    fn verify_output(&self, expected: &[u8]) -> bool {
        let mut pos = 0usize;
        for &(st, en) in &self.sampled.ordered {
            let slice = &self.emit_arena[st..en];
            if pos + slice.len() > expected.len() || &expected[pos..pos + slice.len()] != slice {
                return false;
            }
            pos += slice.len();
        }
        pos == expected.len()
    }
}

// ---------------------------------------------------------------------------
// Corpus loading & synthetic stress suite
// ---------------------------------------------------------------------------

struct A6bInput {
    n: u32,
    m: u32,
    ns: usize,
    thin: [u32; 256],
    first: [u32; 256],
    write_init: [u32; 256],
    strips_raw: Vec<u32>,
}

struct BlockInput {
    index: usize,
    n: usize,
    orig_ptr: u32,
    admitted: bool,
    records: Vec<u16>,
    walked: Vec<u8>,
}

fn load_all_blocks(path: &str) -> Vec<BlockInput> {
    let data = std::fs::read(path).expect("failed to read bz2 corpus");
    let kernels = Kernels::detect();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut tt = TVector::new();
    let mut probe_kernel = ProbeKernel::default();
    let mut probe_out = Vec::new();
    let mut blocks = Vec::new();

    while let Event::Block { symbols: b, .. } = scanner.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, kernels).unwrap();
        let n = ll.len();
        let recs = records_of(&ll);
        let admitted = if let Some(cap) = open_cap(recs.len(), n, b.orig_ptr) {
            if cap == Cap::NONE {
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
                    pass.take(c, pos, width, b.orig_ptr, PROD_LIMIT);
                });
                !pass.queues.over && pass.f.m <= cap.m && pass.ns <= cap.strips
            }
        } else {
            false
        };
        let mut walked = Vec::new();
        build(&ll, &counts, &mut tt);
        walk(&tt, b.orig_ptr, &mut walked);
        blocks.push(BlockInput {
            index: blocks.len(),
            n,
            orig_ptr: b.orig_ptr,
            admitted,
            records: recs,
            walked,
        });
        syms = b.syms;
    }
    blocks
}

fn bwt_of_string(s: &[u8]) -> (Vec<u8>, u32) {
    let n = s.len();
    let mut rots: Vec<usize> = (0..n).collect();
    rots.sort_by(|&a, &b| {
        for off in 0..n {
            let ca = s[(a + off) % n];
            let cb = s[(b + off) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut orig_ptr = 0u32;
    for (i, &r) in rots.iter().enumerate() {
        if r == 0 {
            orig_ptr = i as u32;
        }
        l.push(s[(r + n - 1) % n]);
    }
    (l, orig_ptr)
}

fn build_synthetic_stress() -> Vec<BlockInput> {
    let mut out = Vec::new();
    let mut tt = TVector::new();
    let mut add = |name_idx: usize, ll: Vec<u8>, p: u32| {
        let mut counts = [0u32; 256];
        for &b in &ll {
            counts[usize::from(b)] += 1;
        }
        let mut walked = Vec::new();
        build(&ll, &counts, &mut tt);
        walk(&tt, p, &mut walked);
        let recs = records_of(&ll);
        out.push(BlockInput {
            index: 10_000 + name_idx,
            n: ll.len(),
            orig_ptr: p,
            admitted: true,
            records: recs,
            walked,
        });
    };

    {
        let mut s = Vec::with_capacity(512);
        let mut x = 0x1234_5678u32;
        for _ in 0..512 {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            s.push(((x >> 24) % 13) as u8);
        }
        let (l, p) = bwt_of_string(&s);
        add(0, l, p);
    }
    {
        let mut v = vec![0u8; 40_000];
        v[0] = 1;
        add(1, v, 5);
    }
    {
        let mut s = Vec::with_capacity(6000);
        let patterns: [&[u8]; 4] = [
            b"the quick brown fox jumps over the lazy dog 0123456789 ",
            b"alpha beta gamma delta epsilon zeta eta theta iota kappa ",
            b"structured repetitive wikipedia xml template {{cite web}} ",
            b"the quick brown fox jumps over the lazy dog 9876543210 ",
        ];
        for rep in 0..105 {
            s.extend_from_slice(patterns[rep % 4]);
            s.push(b'A' + (rep % 7) as u8);
        }
        let (l, p) = bwt_of_string(&s);
        add(2, l, p);
    }
    {
        let mut s = Vec::with_capacity(4096);
        for rep in 0..128 {
            for k in 0..32u8 {
                let b = 0x80 | ((k.wrapping_mul(5).wrapping_add((rep % 5) as u8)) & 0x1F);
                s.push(b);
            }
        }
        let (l, p) = bwt_of_string(&s);
        add(3, l, p);
    }
    {
        let mut s = Vec::with_capacity(4096);
        for rep in 0..160 {
            for k in 0..24u8 {
                s.push(b'a' + (k % 6));
            }
            s.push(b'x' + (rep % 3) as u8);
        }
        let (l, p) = bwt_of_string(&s);
        add(4, l, p);
    }
    {
        // Wide single-cycle spiral (widest = 69,999 > 65,535): exercises V4_aos16 u16-width fallback
        let mut v = vec![0u8; 70_000];
        v[0] = 1;
        add(5, v, 5);
    }
    out
}

fn pre_extract_a6b(admitted: &[&BlockInput]) -> Vec<A6bInput> {
    let mut spill = Spill::default();
    let mut edges = Vec::new();
    let mut out_q = Vec::new();
    let mut res = Vec::with_capacity(admitted.len());
    for b in admitted {
        out_q.resize(b.n + INLINE, 0);
        let mut pass = Pass {
            queues: Queues::new(&mut out_q, &mut spill, u32::MAX),
            edges: &mut edges,
            ns: 0,
            cap_strips: usize::MAX,
            f: First::default(),
        };
        maximal(&b.records, |c, pos, width| {
            pass.take(c, pos, width, b.orig_ptr, PROD_LIMIT);
        });
        let ns = pass.ns;
        let f = pass.f;
        let all: [u32; 256] = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        res.push(A6bInput {
            n: b.n as u32,
            m: f.m,
            ns,
            thin: f.thin,
            first: starts(all),
            write_init: starts(f.thick),
            strips_raw: edges[..3 * ns].to_vec(),
        });
    }
    res
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <wikidata.bz2> [--repeats N]", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];
    let mut repeats = 7usize;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--repeats" && i + 1 < args.len() {
            repeats = args[i + 1].parse().expect("invalid repeats");
            i += 2;
        } else {
            eprintln!("Unknown arg: {}", args[i]);
            std::process::exit(1);
        }
    }

    let all_blocks = load_all_blocks(path);
    let admitted: Vec<&BlockInput> = all_blocks.iter().filter(|b| b.admitted).collect();
    let stress = build_synthetic_stress();
    println!(
        "Loaded {}: {} total blocks, {} admitted Open blocks, {} synthetic stress blocks",
        path,
        all_blocks.len(),
        admitted.len(),
        stress.len()
    );

    let arms = [
        Arm::Base1046,
        Arm::V2U128Base,
        Arm::V2U128Opt,
        Arm::V3I7Opt,
        Arm::V3I7Spl4Opt,
        Arm::V4Aos16,
    ];

    let cpu = CpuClock::new();
    let mut kernels: Vec<ProbeKernel> = arms.iter().map(|_| ProbeKernel::default()).collect();
    let mut out_q = Vec::new();
    let mut dict_stats = vec![DictStats::default(); arms.len()];
    let mut total_checks = 0usize;

    for (ai, &arm) in arms.iter().enumerate() {
        let mut d_ph = PhaseNs::default();
        let mut d_ds = DictStats::default();
        for b in &all_blocks {
            kernels[ai].run_block(b, arm, &mut out_q, &mut d_ph, &mut d_ds, None);
            assert!(
                kernels[ai].verify_output(&b.walked),
                "EXACTNESS FAILURE: arm={} block={}",
                arm.name(),
                b.index
            );
            total_checks += 1;
        }
        for b in &stress {
            for sp in [Some(false), Some(true)] {
                kernels[ai].run_block(b, arm, &mut out_q, &mut d_ph, &mut d_ds, sp);
                assert!(
                    kernels[ai].verify_output(&b.walked),
                    "STRESS FAILURE: arm={} block={} spirals={:?}",
                    arm.name(),
                    b.index,
                    sp
                );
                total_checks += 1;
            }
        }
        kernels[ai].gen_cache.content_gen = (1u32 << 16) - 2;
        kernels[ai].gen_cache.content_gen8 = 254;
        kernels[ai].gen_cache.pair_gen = (1u64 << 18) - 2;
        for _ in 0..4 {
            kernels[ai].run_block(&stress[2], arm, &mut out_q, &mut d_ph, &mut d_ds, None);
            assert!(
                kernels[ai].verify_output(&stress[2].walked),
                "GEN WRAP FAILURE: arm={}",
                arm.name()
            );
            total_checks += 1;
        }
    }
    println!(
        "Exactness PASS: {}/{} checks exact ({} wikidata + {}x2 stress + 4 gen-wrap across {} arms)",
        total_checks,
        total_checks,
        all_blocks.len(),
        stress.len(),
        arms.len()
    );

    // Part 1: Isolated A6b (5 passes per repeat -> ~400-450 ms/sample, <=0.4% schedstat tick error)
    let a6b_inputs = pre_extract_a6b(&admitted);
    const A6B_PASSES: usize = 5;
    let n_arms = arms.len();
    let mut iso_cpu = vec![Vec::with_capacity(repeats); n_arms];
    let mut iso_wall = vec![Vec::with_capacity(repeats); n_arms];
    let mut iso_setup = vec![Vec::with_capacity(repeats); n_arms];
    let mut iso_steps = vec![Vec::with_capacity(repeats); n_arms];
    let mut iso_bot = vec![Vec::with_capacity(repeats); n_arms];
    let mut iso_dict = vec![Vec::with_capacity(repeats); n_arms];

    for rep in 0..repeats {
        for off in 0..n_arms {
            let ai = (off + rep) % n_arms;
            let arm = arms[ai];
            let mut ph = PhaseNs::default();
            let c0 = cpu.now_ns();
            let w0 = Instant::now();
            for _ in 0..A6B_PASSES {
                for inp in &a6b_inputs {
                    let cur = kernels[ai].run_a6b_only(inp, arm, &mut ph);
                    black_box(cur[0]);
                }
            }
            let w_ns = w0.elapsed().as_nanos() as u64;
            let c_ns = cpu.now_ns().saturating_sub(c0);
            let div = (A6B_PASSES as f64) * 1e6;
            iso_cpu[ai].push(c_ns as f64 / div);
            iso_wall[ai].push(w_ns as f64 / div);
            iso_setup[ai].push(ph.setup as f64 / div);
            iso_steps[ai].push(ph.steps as f64 / div);
            iso_bot[ai].push(ph.bottom as f64 / div);
            iso_dict[ai].push(ph.dict as f64 / div);
        }
    }

    let (bi_cpu, bi_cpu_lo, bi_cpu_hi) = median_min_max(&iso_cpu[0]);
    let (bi_wall, bi_wall_lo, bi_wall_hi) = median_min_max(&iso_wall[0]);
    println!(
        "\n=== PART 1: ISOLATED A6b ({} repeats x {} passes/rep, 233 admitted Open blocks) ===",
        repeats, A6B_PASSES
    );
    println!(
        "{:<16} {:>22} {:>7} {:>6} {:>5} | {:>22} {:>7} | {:>6} {:>6} {:>5} {:>6} | {:>3}",
        "arm",
        "A6b_cpu [min..max]",
        "cpu_Δ%",
        "i9_1w",
        "i9_8w",
        "A6b_wall [min..max]",
        "wall_Δ%",
        "setup",
        "steps",
        "bot",
        "dict",
        "C/W"
    );
    for (ai, &arm) in arms.iter().enumerate() {
        let (cm, clo, chi) = median_min_max(&iso_cpu[ai]);
        let (wm, wlo, whi) = median_min_max(&iso_wall[ai]);
        let (su, _, _) = median_min_max(&iso_setup[ai]);
        let (st, _, _) = median_min_max(&iso_steps[ai]);
        let (bo, _, _) = median_min_max(&iso_bot[ai]);
        let (di, _, _) = median_min_max(&iso_dict[ai]);
        let cpct = 100.0 * (cm - bi_cpu) / bi_cpu;
        let wpct = 100.0 * (wm - bi_wall) / bi_wall;
        let i9_1w = I9_A6B_1046_1W_MS * (bi_cpu - cm) / bi_cpu;
        let i9_8w = I9_A6B_1046_8W_MS * (bi_cpu - cm) / bi_cpu;
        let dc = if ai == 0 {
            "—"
        } else if chi < bi_cpu_lo {
            "D"
        } else if clo > bi_cpu_hi {
            "S"
        } else {
            "o"
        };
        let dw = if ai == 0 {
            "—"
        } else if whi < bi_wall_lo {
            "D"
        } else if wlo > bi_wall_hi {
            "S"
        } else {
            "o"
        };
        println!(
            "{:<16} {:>6.2} [{:>5.2}..{:>5.2}] {:>+6.2}% {:>+6.1} {:>+5.1} | {:>6.2} [{:>5.2}..{:>5.2}] {:>+6.2}% | {:>6.2} {:>6.2} {:>5.2} {:>6.2} | {:>1}/{:>1}",
            arm.name(),
            cm,
            clo,
            chi,
            cpct,
            i9_1w,
            i9_8w,
            wm,
            wlo,
            whi,
            wpct,
            su,
            st,
            bo,
            di,
            dc,
            dw,
        );
    }

    // Part 2: Full end-to-end A6a + A6b + A6c + 4-lane forward A7 (whole-repeat CpuClock + wall)
    let mut r_a6b = vec![Vec::with_capacity(repeats); n_arms];
    let mut r_a6c = vec![Vec::with_capacity(repeats); n_arms];
    let mut r_a7 = vec![Vec::with_capacity(repeats); n_arms];
    let mut r_b67_wall = vec![Vec::with_capacity(repeats); n_arms];
    let mut r_ce_cpu = vec![Vec::with_capacity(repeats); n_arms];

    for rep in 0..repeats {
        for off in 0..n_arms {
            let ai = (off + rep) % n_arms;
            let arm = arms[ai];
            let mut ph = PhaseNs::default();
            let mut ds = DictStats::default();
            let c0 = cpu.now_ns();
            for b in &admitted {
                kernels[ai].run_block(b, arm, &mut out_q, &mut ph, &mut ds, None);
                black_box(kernels[ai].sampled.ordered.len());
            }
            let c_ns = cpu.now_ns().saturating_sub(c0);
            if rep == 0 {
                dict_stats[ai] = ds;
            }
            let ms = |ns: u64| ns as f64 / 1e6;
            let a6b_ns = ph.setup + ph.steps + ph.bottom + ph.dict;
            let b67_ns = a6b_ns + ph.a6c + ph.a7;
            r_a6b[ai].push(ms(a6b_ns));
            r_a6c[ai].push(ms(ph.a6c));
            r_a7[ai].push(ms(ph.a7));
            r_b67_wall[ai].push(ms(b67_ns));
            r_ce_cpu[ai].push(ms(c_ns));
        }
    }

    let (b_a6b, b_a6b_lo, b_a6b_hi) = median_min_max(&r_a6b[0]);
    let (b_a7, _, _) = median_min_max(&r_a7[0]);
    let (b_b67, b_b67_lo, b_b67_hi) = median_min_max(&r_b67_wall[0]);
    let (b_ce_cpu, b_ce_lo, b_ce_hi) = median_min_max(&r_ce_cpu[0]);

    println!(
        "\n=== PART 2: FULL END-TO-END A6b+A6c+4-lane A7 ({} repeats, 233 admitted blocks) ===",
        repeats
    );
    println!(
        "{:<16} {:>22} {:>7} | {:>6} {:>6} {:>6} | {:>22} {:>7} | {:>22} {:>7} | {:>5} {:>6} {:>6} {:>7}",
        "arm",
        "A6b_w [min..max]",
        "A6b_Δ%",
        "A6c_w",
        "A7_w",
        "A7_Δ%",
        "b67_w [min..max]",
        "b67_Δ%",
        "C+E_cpu [min..max]",
        "CEc_Δ%",
        "A/B/C",
        "dictE",
        "longE",
        "spellK"
    );
    for (ai, &arm) in arms.iter().enumerate() {
        let (a6b, alo, ahi) = median_min_max(&r_a6b[ai]);
        let (a6c, _, _) = median_min_max(&r_a6c[ai]);
        let (a7, _, _) = median_min_max(&r_a7[ai]);
        let (b67, blo, bhi) = median_min_max(&r_b67_wall[ai]);
        let (ce, clo, chi) = median_min_max(&r_ce_cpu[ai]);
        let f = |lo: f64, hi: f64, ref_lo: f64, ref_hi: f64| -> &'static str {
            if ai == 0 {
                "—"
            } else if hi < ref_lo {
                "D"
            } else if lo > ref_hi {
                "S"
            } else {
                "o"
            }
        };
        println!(
            "{:<16} {:>6.2} [{:>5.2}..{:>5.2}] {:>+6.2}% | {:>6.2} {:>6.2} {:>+5.2}% | {:>6.2} [{:>6.2}..{:>6.2}] {:>+6.2}% | {:>6.2} [{:>6.2}..{:>6.2}] {:>+6.2}% | {:>1}/{:>1}/{:>1} {:>6} {:>6} {:>7.1}",
            arm.name(),
            a6b,
            alo,
            ahi,
            100.0 * (a6b - b_a6b) / b_a6b,
            a6c,
            a7,
            100.0 * (a7 - b_a7) / b_a7,
            b67,
            blo,
            bhi,
            100.0 * (b67 - b_b67) / b_b67,
            ce,
            clo,
            chi,
            100.0 * (ce - b_ce_cpu) / b_ce_cpu,
            f(alo, ahi, b_a6b_lo, b_a6b_hi),
            f(blo, bhi, b_b67_lo, b_b67_hi),
            f(clo, chi, b_ce_lo, b_ce_hi),
            dict_stats[ai].dict_entries,
            dict_stats[ai].long_entries,
            dict_stats[ai].spelling_bytes as f64 / 1024.0,
        );
    }
}
