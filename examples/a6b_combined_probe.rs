//! Whole-A6b combined redesign probe (`a6b-combined-redesign`).
//!
//! Smooth-zone characterisation of combined improvements across all three A6b sub-phases
//! (`A6b-setup`, `A6b-steps`, `A6b-dict`) against shipped `src/open.rs` + `src/open/endpoint.rs`,
//! priced end-to-end across full `C + E` (`A6a + A6b + A6c + A7`) under both shipped
//! bidirectional emission and matched forward emission.
//!
//! Zero edits to `src/`, zero `unsafe`.
//!
//! Usage:
//!   a6b_combined_probe corpora/wikidata.bz2 [--repeats 7]

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::hint::black_box;
use std::time::Instant;

/// High-resolution on-CPU thread clock (100% safe Rust, zero `unsafe`).
/// On Linux, reads kernel `se.sum_exec_runtime` (nanoseconds) from `/proc/thread-self/schedstat`
/// via `read_at(0)`, excluding scheduler preemption stalls on shared VMs. Falls back to `Instant`
/// on other platforms.
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
// Production Open constants (mirroring src/open.rs, src/open/endpoint.rs, src/open/outq.rs)
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

// i9-10900K baseline anchors on wikidata x4 (character-stage1/report.txt & results-2/README.md)
const I9_WALL_1W_MS: f64 = 1306.0;
const I9_A6A_MS: f64 = 200.3;
const I9_A6B_MS: f64 = 216.0;
const I9_A6B_SETUP_MS: f64 = 41.7;
const I9_A6B_STEPS_MS: f64 = 110.4;
const I9_A6B_DICT_MS: f64 = 63.9;
const I9_A6C_MS: f64 = 161.4;
const I9_A7_MS: f64 = 490.3;
const I9_CE_MS: f64 = I9_A6A_MS + I9_A6B_MS + I9_A6C_MS + I9_A7_MS; // 1068.0 ms
const I9_8W_A6B_MS: f64 = 30.4;
const I9_8W_A7_MS: f64 = 69.5;
const GATE_PCT: f64 = 3.0;
const GATE_MS: f64 = I9_WALL_1W_MS * (GATE_PCT / 100.0); // 39.18 ms

// u128 inline15 encoding: byte 15 (bits 120..128) is tag:
//   0        => empty word (entire u128 == 0)
//   1..=15   => inline payload of length `tag` in bytes 0..tag, bytes tag..15 == 0
//   0xFF     => long rule handle: bits 0..32 = rule_id (>= 257), bits 32..64 = exact length
const U128_PAYLOAD_MASK: u128 = (1u128 << 120) - 1;
const U128_LONG_TAG: u128 = 0xFFu128 << 120;

// u64 inline7 encoding: byte 7 (bits 56..64) is tag:
//   0        => empty word (entire u64 == 0)
//   1..=7    => inline payload of length `tag` in bytes 0..tag, bytes tag..7 == 0
//   0xFF     => rule handle: bits 0..24 = rule_id (>= 257), bits 24..48 = exact length
const U64_PAYLOAD_MASK: u64 = (1u64 << 56) - 1;
const U64_RULE_TAG: u64 = 0xFFu64 << 56;
const U64_ID_MASK: u64 = (1u64 << 24) - 1;

const GEN_PAIR_SLOTS: usize = 1024;
const GEN_CONTENT_SLOTS: usize = 4096;

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
fn literal(byte: u8) -> u32 {
    (1 << ID_BITS) | (u32::from(byte) + 1)
}

#[inline(always)]
fn literal_u128(byte: u8) -> u128 {
    (1u128 << 120) | u128::from(byte)
}

#[inline(always)]
fn literal_u64(byte: u8) -> u64 {
    (1u64 << 56) | u64::from(byte)
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
// OutQ16 (exact mirror of src/open/outq.rs)
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
            let base = self.id as usize * CHUNK_BYTES + 2 * self.at;
            let src = &self.out[base..base + 2 * span];
            let hi = self.epoch * EPOCH;
            for (u, bytes) in (dst_start as u32..).zip(src.chunks_exact(2)) {
                let v = hi | u32::from(u16::from_le_bytes([bytes[0], bytes[1]]));
                edges[u as usize] ^= pack | v;
                edges[v as usize] ^= u;
                if v == mark {
                    *predecessor = u;
                }
            }
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
        mut span: usize,
        pack: u32,
        mark: u32,
        predecessor: &mut u32,
    ) {
        while span > 0 {
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
            let take = span
                .min(CHUNK - self.at)
                .min((self.cut - self.read) as usize);
            let chunk = if self.id < self.in_out {
                let at = self.id as usize * CHUNK_BYTES;
                &self.out[at..at + CHUNK_BYTES]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][..]
            };
            let hi = self.epoch * EPOCH;
            for (u, bytes) in
                (dst_start as u32..).zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
            {
                let v = hi | u32::from(u16::from_le_bytes([bytes[0], bytes[1]]));
                edges[u as usize] ^= pack | v;
                edges[v as usize] ^= u;
                if v == mark {
                    *predecessor = u;
                }
            }
            self.at += take;
            self.read += take as u32;
            dst_start += take;
            span -= take;
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
// Shipped Endpoint (exact mirror of src/open/endpoint.rs)
// ---------------------------------------------------------------------------

#[derive(Default, Clone, Copy)]
struct ShippedStats {
    words: usize,
    copies: usize,
    traced: usize,
    thick_area: usize,
    unit_work: usize,
    longest: usize,
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
}

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

#[inline(always)]
fn short_u128_hash(entry_u128: u128, len: u8) -> u64 {
    let low = entry_u128 as u64;
    let high = (entry_u128 >> 64) as u64;
    let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ u64::from(len).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

#[inline(always)]
fn short_u64_hash(entry_u64: u64, len: u8) -> u64 {
    let h = entry_u64.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ u64::from(len).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

#[inline(always)]
fn long_bytes_hash(bytes: &[u8]) -> u64 {
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

struct Dictionary<'a> {
    pairs: &'a mut [u32],
    contents: &'a mut [u32],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl Dictionary<'_> {
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        if handle == 0 {
            return u32::from(byte);
        }
        let id = handle & ID_MASK;
        let key = (id << 8) | u32::from(byte);
        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
        if !self.pairs.is_empty() && self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }

        let start = self.spelling.len();
        let memoized = handle >> ID_BITS >= INLINE as u32 && e.a[e.rule(id) + 3] != u32::MAX;
        let (entry, len) = e.inline_word(handle, byte, self.stack, self.spelling);
        if self.contents.is_empty() {
            return self.insert(entry, len);
        }
        let bytes = if len == LONG {
            &self.spelling[start..]
        } else {
            &entry[..len as usize]
        };
        let index = spelling_hash(&entry, len, bytes) as usize & (self.contents.len() - 1);
        let old = self.contents[index] as usize;
        let same = old != 0
            && self.lens[old] == len
            && if len == LONG {
                long_word(&self.inline16[old], self.spelling) == bytes
            } else {
                self.inline16[old] == entry
            };
        let code = if same {
            if len == LONG {
                if memoized {
                    self.spelling.truncate(start);
                } else {
                    self.spelling.pop();
                }
            }
            old as u32
        } else {
            let code = self.insert(entry, len);
            self.contents[index] = code;
            code
        };
        self.pairs[slot] = key;
        self.pairs[slot + 1] = code;
        code
    }

    #[inline(always)]
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}

// ---------------------------------------------------------------------------
// Redesigned A6b data structures (direct u16 links, safe 64K no-BC arrays, inline7/15, gen-cache)
// ---------------------------------------------------------------------------

#[derive(Default, Clone, Copy)]
struct NodeLinks {
    prev0: u16,
    next0: u16,
    prev1: u16,
    next1: u16,
}

#[derive(Default, Clone, Copy)]
#[repr(C, align(32))]
struct Node32 {
    prev0: u16,
    next0: u16,
    prev1: u16,
    next1: u16,
    width: u32,
    _pad: u32,
    word: u128,
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
struct Node24 {
    prev0: u16,
    next0: u16,
    prev1: u16,
    next1: u16,
    width: u32,
    _pad: u32,
    word: u64,
}

fn boxed_array_64k<T: Copy>(val: T) -> Box<[T; NARROW_NODES]> {
    vec![val; NARROW_NODES]
        .into_boxed_slice()
        .try_into()
        .ok()
        .unwrap()
}

/// Generation-tagged L1d-hot caches (zero per-block fill!).
/// - `pairs`: 1024 x u64 (8 KiB), packed as `(gen18 << 46) | (code16 << 30) | key30`.
/// - `contents`: 4096 x u32 (16 KiB), packed as `(gen16 << 16) | code16`.
struct GenCache {
    pairs: Vec<u64>,
    contents: Vec<u32>,
    pair_gen: u64,
    content_gen: u32,
}

impl Default for GenCache {
    fn default() -> Self {
        GenCache {
            pairs: vec![0; GEN_PAIR_SLOTS],
            contents: vec![0; GEN_CONTENT_SLOTS],
            pair_gen: 0,
            content_gen: 0,
        }
    }
}

impl GenCache {
    #[inline(always)]
    fn next_block(&mut self) {
        self.pair_gen += 1;
        if self.pair_gen >= (1u64 << 18) {
            self.pairs.fill(0);
            self.pair_gen = 1;
        }
        self.content_gen += 1;
        if self.content_gen >= (1u32 << 16) {
            self.contents.fill(0);
            self.content_gen = 1;
        }
    }
}

struct Fixed64kSoa {
    links: Box<[NodeLinks; NARROW_NODES]>,
    widths: Box<[u32; NARROW_NODES]>,
    words128: Box<[u128; NARROW_NODES]>,
}

impl Fixed64kSoa {
    fn new() -> Self {
        Fixed64kSoa {
            links: boxed_array_64k(NodeLinks::default()),
            widths: boxed_array_64k(0),
            words128: boxed_array_64k(0),
        }
    }
}

/// Persistent reusable state for redesigned Narrow A6b arms.
#[derive(Default)]
struct RedesignState {
    image_order64: Vec<u64>,
    links: Vec<NodeLinks>,
    widths: Vec<u32>,
    words32: Vec<u32>,
    words128: Vec<u128>,
    fixed_soa: Option<Fixed64kSoa>,
    fixed_aos32: Option<Box<[Node32; NARROW_NODES]>>,
    fixed_aos24: Option<Box<[Node24; NARROW_NODES]>>,
    rules: Vec<[u32; 4]>,
    gen_cache: GenCache,
    zero_pairs: Vec<u32>,
    zero_contents: Vec<u32>,
}

#[inline(always)]
fn rules_len(rules: &[[u32; 4]], w: u32) -> u32 {
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

#[inline(always)]
fn join_u32_rules(rules: &mut Vec<[u32; 4]>, left: u32, right: u32) -> u32 {
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
        rules_len(rules, left) + rules_len(rules, right)
    };
    let rule = if len < INLINE as u32 {
        let bytes = rules_short_payload(rules, left)
            | (rules_short_payload(rules, right) << (8 * left_len))
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

fn power_u32_rules(rules: &mut Vec<[u32; 4]>, word: u32, q: u32) -> u32 {
    let (mut acc, mut base, mut e) = (0, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_u32_rules(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_u32_rules(rules, base, base);
    }
}

#[inline(always)]
fn join_u128(rules: &mut Vec<[u32; 4]>, left: u128, right: u128) -> u128 {
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
    join_u128_long(rules, left, right, ll, rl)
}

#[cold]
#[inline(never)]
fn join_u128_long(rules: &mut Vec<[u32; 4]>, left: u128, right: u128, ll: u32, rl: u32) -> u128 {
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

fn power_u128(rules: &mut Vec<[u32; 4]>, word: u128, q: u32) -> u128 {
    let (mut acc, mut base, mut e) = (0u128, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_u128(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_u128(rules, base, base);
    }
}

/// Inline7 u64 join: single 64-bit register, 1-cycle shift + add + or for len <= 7!
#[inline(always)]
fn join_u64(rules: &mut Vec<[u32; 4]>, left: u64, right: u64) -> u64 {
    if left == 0 {
        return right;
    }
    if right == 0 {
        return left;
    }
    let ll = (left >> 56) as u32;
    let rl = (right >> 56) as u32;
    if ll + rl <= 7 {
        return (left + (u64::from(rl) << 56)) | (right << (8 * ll));
    }
    join_u64_long(rules, left, right, ll, rl)
}

#[cold]
#[inline(never)]
fn join_u64_long(rules: &mut Vec<[u32; 4]>, left: u64, right: u64, ll: u32, rl: u32) -> u64 {
    let l_len = if ll <= 7 {
        ll
    } else {
        ((left >> 24) & U64_ID_MASK) as u32
    };
    let r_len = if rl <= 7 {
        rl
    } else {
        ((right >> 24) & U64_ID_MASK) as u32
    };
    let total_len = l_len + r_len;
    let rule = if total_len < INLINE as u32 {
        let lp = if ll <= 7 {
            u128::from(left & U64_PAYLOAD_MASK)
        } else {
            rules_short_payload(rules, (left & U64_ID_MASK) as u32)
        };
        let rp = if rl <= 7 {
            u128::from(right & U64_PAYLOAD_MASK)
        } else {
            rules_short_payload(rules, (right & U64_ID_MASK) as u32)
        };
        let bytes = lp | (rp << (8 * l_len)) | (u128::from(total_len) << 120);
        [
            bytes as u32,
            (bytes >> 32) as u32,
            (bytes >> 64) as u32,
            (bytes >> 96) as u32,
        ]
    } else {
        let lid = if ll == 0xFF {
            (left & U64_ID_MASK) as u32
        } else if ll == 1 {
            ((left as u8) as u32) + 1
        } else {
            let id = rules.len() as u32 + 257;
            let p = left & U64_PAYLOAD_MASK;
            rules.push([p as u32, (p >> 32) as u32, 0, ll << 24]);
            id
        };
        let rid = if rl == 0xFF {
            (right & U64_ID_MASK) as u32
        } else if rl == 1 {
            ((right as u8) as u32) + 1
        } else {
            let id = rules.len() as u32 + 257;
            let p = right & U64_PAYLOAD_MASK;
            rules.push([p as u32, (p >> 32) as u32, 0, rl << 24]);
            id
        };
        [lid, rid, total_len, u32::MAX]
    };
    let id = rules.len() as u32 + 257;
    rules.push(rule);
    U64_RULE_TAG | (u64::from(total_len) << 24) | u64::from(id)
}

fn power_u64(rules: &mut Vec<[u32; 4]>, word: u64, q: u32) -> u64 {
    let (mut acc, mut base, mut e) = (0u64, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_u64(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_u64(rules, base, base);
    }
}

fn append_rules(rules: &mut [[u32; 4]], word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
    stack.clear();
    stack.push(word);
    while let Some(id) = stack.pop() {
        if id <= 256 {
            dest.push((id - 1) as u8);
        } else {
            let idx = (id - 257) as usize;
            let tag = rules[idx][3];
            if tag >> 28 == 0 {
                let r = rules[idx];
                let bytes = (u128::from(r[0])
                    | (u128::from(r[1]) << 32)
                    | (u128::from(r[2]) << 64)
                    | (u128::from(r[3] & 0x00ff_ffff) << 96))
                    .to_le_bytes();
                dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                continue;
            }
            if tag == u32::MAX {
                let r0 = rules[idx][0];
                let r1 = rules[idx][1];
                rules[idx][3] = 0x8000_0000 | dest.len() as u32;
                stack.push(r1);
                stack.push(r0);
            } else {
                let start = (tag & 0x7fff_ffff) as usize;
                let len = rules[idx][2] as usize;
                dest.extend_from_within(start..start + len);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fused setup + elimination + assign_bottom for Vec SoA and Safe-NoBC 64K layouts
// ---------------------------------------------------------------------------

fn setup_soa_vec<const U128: bool>(
    strips_raw: &[u32],
    ns: usize,
    n: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    most: usize,
    st: &mut RedesignState,
) -> (usize, u32) {
    if st.image_order64.len() < ns {
        st.image_order64.resize(ns, 0);
    }
    if st.links.len() < most {
        st.links.resize(most, NodeLinks::default());
    }
    if st.widths.len() < most {
        st.widths.resize(most, 0);
    }
    if U128 {
        if st.words128.len() < most {
            st.words128.resize(most, 0);
        }
    } else if st.words32.len() < most {
        st.words32.resize(most, 0);
    }

    let links = &mut st.links[..most];
    let widths = &mut st.widths[..most];
    let img = &mut st.image_order64[..ns];

    links[0] = NodeLinks::default();
    widths[0] = 0;
    if U128 {
        st.words128[0] = 0;
    } else {
        st.words32[0] = 0;
    }

    let mut nodes = 1 + ns;
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

        widths[sid as usize] = width;
        if U128 {
            st.words128[sid as usize] = literal_u128(byte);
        } else {
            st.words32[sid as usize] = literal(byte);
        }
        widest = widest.max(width);

        let pos = write[a] as usize;
        write[a] = (pos + 1) as u32;
        img[pos] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);

        if x < from {
            let gap = from - x;
            widest = widest.max(gap);
            let gid = nodes as u16;
            nodes += 1;
            widths[gid as usize] = gap;
            if U128 {
                st.words128[gid as usize] = 0;
            } else {
                st.words32[gid as usize] = 0;
            }
            links[ord0_tail as usize].next0 = gid;
            links[gid as usize].prev0 = ord0_tail;
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                links[term1_tail as usize].next1 = gid;
            }
            links[gid as usize].prev1 = term1_tail;
            term1_tail = gid;
        }
        links[ord0_tail as usize].next0 = sid;
        links[sid as usize].prev0 = ord0_tail;
        ord0_tail = sid;
        x = from + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes as u16;
        nodes += 1;
        widths[gid as usize] = gap;
        if U128 {
            st.words128[gid as usize] = 0;
        } else {
            st.words32[gid as usize] = 0;
        }
        links[ord0_tail as usize].next0 = gid;
        links[gid as usize].prev0 = ord0_tail;
        ord0_tail = gid;
        if term1_tail == 0 {
            term1_head = gid;
        } else {
            links[term1_tail as usize].next1 = gid;
        }
        links[gid as usize].prev1 = term1_tail;
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
            let gid = nodes as u16;
            nodes += 1;
            widths[gid as usize] = gap;
            if U128 {
                st.words128[gid as usize] = 0;
            } else {
                st.words32[gid as usize] = 0;
            }
            links[ord1_tail as usize].next1 = gid;
            links[gid as usize].prev1 = ord1_tail;
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                links[term0_tail as usize].next0 = gid;
            }
            links[gid as usize].prev0 = term0_tail;
            term0_tail = gid;
        }
        links[ord1_tail as usize].next1 = sid;
        links[sid as usize].prev1 = ord1_tail;
        ord1_tail = sid;
        x = to + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes as u16;
        nodes += 1;
        widths[gid as usize] = gap;
        if U128 {
            st.words128[gid as usize] = 0;
        } else {
            st.words32[gid as usize] = 0;
        }
        links[ord1_tail as usize].next1 = gid;
        links[gid as usize].prev1 = ord1_tail;
        ord1_tail = gid;
        if term0_tail == 0 {
            term0_head = gid;
        } else {
            links[term0_tail as usize].next0 = gid;
        }
        links[gid as usize].prev0 = term0_tail;
        term0_tail = gid;
    }
    let ord1_head = if ord1_tail == 0 { 0 } else { links[0].next1 };

    {
        let mut prev = 0u16;
        for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
            if t == 0 {
                continue;
            }
            links[prev as usize].next0 = h;
            links[h as usize].prev0 = prev;
            prev = t;
        }
        links[prev as usize].next0 = 0;
        links[0].prev0 = prev;
    }
    {
        let mut prev = 0u16;
        for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
            if t == 0 {
                continue;
            }
            links[prev as usize].next1 = h;
            links[h as usize].prev1 = prev;
            prev = t;
        }
        links[prev as usize].next1 = 0;
        links[0].prev1 = prev;
    }

    (nodes, widest)
}

/// Safe zero-bounds-check setup for fixed 64K SoA (`Box<[T; 65536]>` indexed by `usize::from(u16)`).
fn setup_soa_64k(
    strips_raw: &[u32],
    ns: usize,
    n: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    img_buf: &mut Vec<u64>,
    soa: &mut Fixed64kSoa,
) -> u32 {
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
    widest
}

/// Safe zero-bounds-check setup for fixed 64K AoS32 (`inline15`).
fn setup_aos32_64k(
    strips_raw: &[u32],
    ns: usize,
    n: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    img_buf: &mut Vec<u64>,
    nd: &mut [Node32; NARROW_NODES],
) -> u32 {
    if img_buf.len() < ns {
        img_buf.resize(ns, 0);
    }
    let img = &mut img_buf[..ns];

    nd[0] = Node32::default();
    let mut nodes = (1 + ns) as u16;
    let mut widest = 0u32;
    let mut ord0_tail = 0u16;
    let (mut term1_head, mut term1_tail) = (0u16, 0u16);
    let mut x = 0u32;

    for i in 0..ns {
        let base = 3 * i;
        let from = strips_raw[base];
        let to_rel = strips_raw[base + 1];
        let mword = strips_raw[base + 2];
        let byte = (mword & 0xFF) as u8;
        let width = mword >> 8;
        let a = usize::from(byte);
        let to = to_rel + first[a];
        let sid = (i + 1) as u16;

        nd[usize::from(sid)].width = width;
        nd[usize::from(sid)].word = literal_u128(byte);
        widest = widest.max(width);

        let pos = write[a] as usize;
        write[a] = (pos + 1) as u32;
        img[pos] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);

        if x < from {
            let gap = from - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            nd[usize::from(gid)].width = gap;
            nd[usize::from(gid)].word = 0;
            nd[usize::from(ord0_tail)].next0 = gid;
            nd[usize::from(gid)].prev0 = ord0_tail;
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                nd[usize::from(term1_tail)].next1 = gid;
            }
            nd[usize::from(gid)].prev1 = term1_tail;
            term1_tail = gid;
        }
        nd[usize::from(ord0_tail)].next0 = sid;
        nd[usize::from(sid)].prev0 = ord0_tail;
        ord0_tail = sid;
        x = from + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes;
        nodes += 1;
        nd[usize::from(gid)].width = gap;
        nd[usize::from(gid)].word = 0;
        nd[usize::from(ord0_tail)].next0 = gid;
        nd[usize::from(gid)].prev0 = ord0_tail;
        ord0_tail = gid;
        if term1_tail == 0 {
            term1_head = gid;
        } else {
            nd[usize::from(term1_tail)].next1 = gid;
        }
        nd[usize::from(gid)].prev1 = term1_tail;
        term1_tail = gid;
    }
    let ord0_head = if ord0_tail == 0 { 0 } else { nd[0].next0 };

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
            nd[usize::from(gid)].width = gap;
            nd[usize::from(gid)].word = 0;
            nd[usize::from(ord1_tail)].next1 = gid;
            nd[usize::from(gid)].prev1 = ord1_tail;
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                nd[usize::from(term0_tail)].next0 = gid;
            }
            nd[usize::from(gid)].prev0 = term0_tail;
            term0_tail = gid;
        }
        nd[usize::from(ord1_tail)].next1 = sid;
        nd[usize::from(sid)].prev1 = ord1_tail;
        ord1_tail = sid;
        x = to + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes;
        nd[usize::from(gid)].width = gap;
        nd[usize::from(gid)].word = 0;
        nd[usize::from(ord1_tail)].next1 = gid;
        nd[usize::from(gid)].prev1 = ord1_tail;
        ord1_tail = gid;
        if term0_tail == 0 {
            term0_head = gid;
        } else {
            nd[usize::from(term0_tail)].next0 = gid;
        }
        nd[usize::from(gid)].prev0 = term0_tail;
        term0_tail = gid;
    }
    let ord1_head = if ord1_tail == 0 { 0 } else { nd[0].next1 };

    {
        let mut prev = 0u16;
        for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
            if t == 0 {
                continue;
            }
            nd[usize::from(prev)].next0 = h;
            nd[usize::from(h)].prev0 = prev;
            prev = t;
        }
        nd[usize::from(prev)].next0 = 0;
        nd[0].prev0 = prev;
    }
    {
        let mut prev = 0u16;
        for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
            if t == 0 {
                continue;
            }
            nd[usize::from(prev)].next1 = h;
            nd[usize::from(h)].prev1 = prev;
            prev = t;
        }
        nd[usize::from(prev)].next1 = 0;
        nd[0].prev1 = prev;
    }
    widest
}

/// Safe zero-bounds-check setup for fixed 64K AoS24 (`inline7` u64 words, fits inside 256 KiB L2!).
fn setup_aos24_64k(
    strips_raw: &[u32],
    ns: usize,
    n: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    img_buf: &mut Vec<u64>,
    nd: &mut [Node24; NARROW_NODES],
) -> u32 {
    if img_buf.len() < ns {
        img_buf.resize(ns, 0);
    }
    let img = &mut img_buf[..ns];

    nd[0] = Node24::default();
    let mut nodes = (1 + ns) as u16;
    let mut widest = 0u32;
    let mut ord0_tail = 0u16;
    let (mut term1_head, mut term1_tail) = (0u16, 0u16);
    let mut x = 0u32;

    for i in 0..ns {
        let base = 3 * i;
        let from = strips_raw[base];
        let to_rel = strips_raw[base + 1];
        let mword = strips_raw[base + 2];
        let byte = (mword & 0xFF) as u8;
        let width = mword >> 8;
        let a = usize::from(byte);
        let to = to_rel + first[a];
        let sid = (i + 1) as u16;

        nd[usize::from(sid)].width = width;
        nd[usize::from(sid)].word = literal_u64(byte);
        widest = widest.max(width);

        let pos = write[a] as usize;
        write[a] = (pos + 1) as u32;
        img[pos] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);

        if x < from {
            let gap = from - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            nd[usize::from(gid)].width = gap;
            nd[usize::from(gid)].word = 0;
            nd[usize::from(ord0_tail)].next0 = gid;
            nd[usize::from(gid)].prev0 = ord0_tail;
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                nd[usize::from(term1_tail)].next1 = gid;
            }
            nd[usize::from(gid)].prev1 = term1_tail;
            term1_tail = gid;
        }
        nd[usize::from(ord0_tail)].next0 = sid;
        nd[usize::from(sid)].prev0 = ord0_tail;
        ord0_tail = sid;
        x = from + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes;
        nodes += 1;
        nd[usize::from(gid)].width = gap;
        nd[usize::from(gid)].word = 0;
        nd[usize::from(ord0_tail)].next0 = gid;
        nd[usize::from(gid)].prev0 = ord0_tail;
        ord0_tail = gid;
        if term1_tail == 0 {
            term1_head = gid;
        } else {
            nd[usize::from(term1_tail)].next1 = gid;
        }
        nd[usize::from(gid)].prev1 = term1_tail;
        term1_tail = gid;
    }
    let ord0_head = if ord0_tail == 0 { 0 } else { nd[0].next0 };

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
            nd[usize::from(gid)].width = gap;
            nd[usize::from(gid)].word = 0;
            nd[usize::from(ord1_tail)].next1 = gid;
            nd[usize::from(gid)].prev1 = ord1_tail;
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                nd[usize::from(term0_tail)].next0 = gid;
            }
            nd[usize::from(gid)].prev0 = term0_tail;
            term0_tail = gid;
        }
        nd[usize::from(ord1_tail)].next1 = sid;
        nd[usize::from(sid)].prev1 = ord1_tail;
        ord1_tail = sid;
        x = to + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes;
        nd[usize::from(gid)].width = gap;
        nd[usize::from(gid)].word = 0;
        nd[usize::from(ord1_tail)].next1 = gid;
        nd[usize::from(gid)].prev1 = ord1_tail;
        ord1_tail = gid;
        if term0_tail == 0 {
            term0_head = gid;
        } else {
            nd[usize::from(term0_tail)].next0 = gid;
        }
        nd[usize::from(gid)].prev0 = term0_tail;
        term0_tail = gid;
    }
    let ord1_head = if ord1_tail == 0 { 0 } else { nd[0].next1 };

    {
        let mut prev = 0u16;
        for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
            if t == 0 {
                continue;
            }
            nd[usize::from(prev)].next0 = h;
            nd[usize::from(h)].prev0 = prev;
            prev = t;
        }
        nd[usize::from(prev)].next0 = 0;
        nd[0].prev0 = prev;
    }
    {
        let mut prev = 0u16;
        for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
            if t == 0 {
                continue;
            }
            nd[usize::from(prev)].next1 = h;
            nd[usize::from(h)].prev1 = prev;
            prev = t;
        }
        nd[usize::from(prev)].next1 = 0;
        nd[0].prev1 = prev;
    }
    widest
}

fn eliminate_u32_soa<const SPIRALS: bool>(
    links: &mut [NodeLinks],
    widths: &mut [u32],
    words: &mut [u32],
    rules: &mut Vec<[u32; 4]>,
    n: u32,
) -> usize {
    rules.clear();
    let mut left = n;
    let mut taken = 0usize;
    while left != 0 {
        let x = links[0].prev0 as usize;
        let y = links[0].prev1 as usize;
        let aw = widths[x];
        let bw = widths[y];
        if SPIRALS && x != y {
            if aw > bw && links[y].prev1 as usize == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                let head = power_u32_rules(rules, words[x], q);
                words[y] = join_u32_rules(rules, head, words[y]);
                widths[x] = aw - q * bw;
                left -= q * bw;
                taken += q as usize;
                continue;
            }
            if aw < bw && links[x].prev0 as usize == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                let tail = power_u32_rules(rules, words[y], q);
                words[x] = join_u32_rules(rules, words[x], tail);
                widths[y] = bw - q * aw;
                left -= q * aw;
                taken += q as usize;
                continue;
            }
        }
        let w = aw.min(bw);
        left -= w;
        taken += 1;
        if x == y {
            let xp = links[x].prev0 as usize;
            links[xp].next0 = 0;
            links[0].prev0 = xp as u16;
            let yp = links[y].prev1 as usize;
            links[yp].next1 = 0;
            links[0].prev1 = yp as u16;
            continue;
        }
        let word = join_u32_rules(rules, words[x], words[y]);
        if aw <= bw {
            let xp = links[x].prev0 as usize;
            links[xp].next0 = 0;
            links[0].prev0 = xp as u16;
            let yn = links[y].next0 as usize;
            links[x].prev0 = y as u16;
            links[x].next0 = yn as u16;
            links[y].next0 = x as u16;
            links[yn].prev0 = x as u16;
            words[x] = word;
            if aw == bw {
                let yp0 = links[y].prev0 as usize;
                let yn0 = links[y].next0 as usize;
                links[yp0].next0 = yn0 as u16;
                links[yn0].prev0 = yp0 as u16;
                let yp1 = links[y].prev1 as usize;
                links[yp1].next1 = 0;
                links[0].prev1 = yp1 as u16;
            } else {
                widths[y] = bw - w;
            }
        } else {
            let yp = links[y].prev1 as usize;
            links[yp].next1 = 0;
            links[0].prev1 = yp as u16;
            let xn = links[x].next1 as usize;
            links[y].prev1 = x as u16;
            links[y].next1 = xn as u16;
            links[x].next1 = y as u16;
            links[xn].prev1 = y as u16;
            words[y] = word;
            widths[x] = aw - w;
        }
    }
    taken
}

fn eliminate_inline15_soa_vec<const SPIRALS: bool>(
    links: &mut [NodeLinks],
    widths: &mut [u32],
    words: &mut [u128],
    rules: &mut Vec<[u32; 4]>,
    n: u32,
) -> usize {
    rules.clear();
    let mut left = n;
    let mut taken = 0usize;
    while left != 0 {
        let x = links[0].prev0 as usize;
        let y = links[0].prev1 as usize;
        let aw = widths[x];
        let bw = widths[y];
        if SPIRALS && x != y {
            if aw > bw && links[y].prev1 as usize == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                let head = power_u128(rules, words[x], q);
                words[y] = join_u128(rules, head, words[y]);
                widths[x] = aw - q * bw;
                left -= q * bw;
                taken += q as usize;
                continue;
            }
            if aw < bw && links[x].prev0 as usize == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                let tail = power_u128(rules, words[y], q);
                words[x] = join_u128(rules, words[x], tail);
                widths[y] = bw - q * aw;
                left -= q * aw;
                taken += q as usize;
                continue;
            }
        }
        let w = aw.min(bw);
        left -= w;
        taken += 1;
        if x == y {
            let xp = links[x].prev0 as usize;
            links[xp].next0 = 0;
            links[0].prev0 = xp as u16;
            let yp = links[y].prev1 as usize;
            links[yp].next1 = 0;
            links[0].prev1 = yp as u16;
            continue;
        }
        let word = join_u128(rules, words[x], words[y]);
        if aw > bw {
            let yp = links[y].prev1 as usize;
            links[yp].next1 = 0;
            links[0].prev1 = yp as u16;
            let xn = links[x].next1 as usize;
            links[y].prev1 = x as u16;
            links[y].next1 = xn as u16;
            links[x].next1 = y as u16;
            links[xn].prev1 = y as u16;
            words[y] = word;
            widths[x] = aw - bw;
        } else if aw < bw {
            let xp = links[x].prev0 as usize;
            links[xp].next0 = 0;
            links[0].prev0 = xp as u16;
            let yn = links[y].next0 as usize;
            links[x].prev0 = y as u16;
            links[x].next0 = yn as u16;
            links[y].next0 = x as u16;
            links[yn].prev0 = x as u16;
            words[x] = word;
            widths[y] = bw - aw;
        } else {
            let xp = links[x].prev0 as usize;
            links[xp].next0 = 0;
            links[0].prev0 = xp as u16;
            let yp0 = links[y].prev0 as usize;
            let yn0 = links[y].next0 as usize;
            links[x].prev0 = yp0 as u16;
            links[x].next0 = yn0 as u16;
            links[yp0].next0 = x as u16;
            links[yn0].prev0 = x as u16;
            let yp1 = links[y].prev1 as usize;
            links[yp1].next1 = 0;
            links[0].prev1 = yp1 as u16;
            words[x] = word;
        }
    }
    taken
}

/// Safe zero-bounds-check elimination for fixed 64K SoA (`inline15`).
fn eliminate_inline15_soa_64k<const SPIRALS: bool>(
    soa: &mut Fixed64kSoa,
    rules: &mut Vec<[u32; 4]>,
    n: u32,
) -> usize {
    rules.clear();
    let links = &mut *soa.links;
    let widths = &mut *soa.widths;
    let words = &mut *soa.words128;
    let mut left = n;
    let mut taken = 0usize;
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
                let head = power_u128(rules, words[xi], q);
                words[yi] = join_u128(rules, head, words[yi]);
                widths[xi] = aw - q * bw;
                left -= q * bw;
                taken += q as usize;
                continue;
            }
            if aw < bw && links[xi].prev0 == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                let tail = power_u128(rules, words[yi], q);
                words[xi] = join_u128(rules, words[xi], tail);
                widths[yi] = bw - q * aw;
                left -= q * aw;
                taken += q as usize;
                continue;
            }
        }
        let w = aw.min(bw);
        left -= w;
        taken += 1;
        if x == y {
            let xp = links[xi].prev0;
            links[usize::from(xp)].next0 = 0;
            links[0].prev0 = xp;
            let yp = links[yi].prev1;
            links[usize::from(yp)].next1 = 0;
            links[0].prev1 = yp;
            continue;
        }
        let word = join_u128(rules, words[xi], words[yi]);
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
    taken
}

/// Safe zero-bounds-check elimination for fixed 64K AoS32 (`inline15`).
fn eliminate_inline15_aos32_64k<const SPIRALS: bool>(
    nd: &mut [Node32; NARROW_NODES],
    rules: &mut Vec<[u32; 4]>,
    n: u32,
) -> usize {
    rules.clear();
    let mut left = n;
    let mut taken = 0usize;
    while left != 0 {
        let x = nd[0].prev0;
        let y = nd[0].prev1;
        let xi = usize::from(x);
        let yi = usize::from(y);
        let nx = nd[xi];
        let ny = nd[yi];
        let aw = nx.width;
        let bw = ny.width;
        if SPIRALS && x != y {
            if aw > bw && ny.prev1 == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                let head = power_u128(rules, nx.word, q);
                nd[yi].word = join_u128(rules, head, ny.word);
                nd[xi].width = aw - q * bw;
                left -= q * bw;
                taken += q as usize;
                continue;
            }
            if aw < bw && nx.prev0 == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                let tail = power_u128(rules, ny.word, q);
                nd[xi].word = join_u128(rules, nx.word, tail);
                nd[yi].width = bw - q * aw;
                left -= q * aw;
                taken += q as usize;
                continue;
            }
        }
        let w = aw.min(bw);
        left -= w;
        taken += 1;
        if x == y {
            let xp = nx.prev0;
            nd[usize::from(xp)].next0 = 0;
            nd[0].prev0 = xp;
            let yp = ny.prev1;
            nd[usize::from(yp)].next1 = 0;
            nd[0].prev1 = yp;
            continue;
        }
        let word = join_u128(rules, nx.word, ny.word);
        if aw > bw {
            let yp = ny.prev1;
            nd[usize::from(yp)].next1 = 0;
            nd[0].prev1 = yp;
            let xn = nd[xi].next1;
            nd[yi].prev1 = x;
            nd[yi].next1 = xn;
            nd[xi].next1 = y;
            nd[usize::from(xn)].prev1 = y;
            nd[yi].word = word;
            nd[xi].width = aw - bw;
        } else if aw < bw {
            let xp = nx.prev0;
            nd[usize::from(xp)].next0 = 0;
            nd[0].prev0 = xp;
            let yn = nd[yi].next0;
            nd[xi].prev0 = y;
            nd[xi].next0 = yn;
            nd[yi].next0 = x;
            nd[usize::from(yn)].prev0 = x;
            nd[xi].word = word;
            nd[yi].width = bw - aw;
        } else {
            let xp = nx.prev0;
            nd[usize::from(xp)].next0 = 0;
            nd[0].prev0 = xp;
            let yp0 = nd[yi].prev0;
            let yn0 = nd[yi].next0;
            nd[xi].prev0 = yp0;
            nd[xi].next0 = yn0;
            nd[usize::from(yp0)].next0 = x;
            nd[usize::from(yn0)].prev0 = x;
            let yp1 = ny.prev1;
            nd[usize::from(yp1)].next1 = 0;
            nd[0].prev1 = yp1;
            nd[xi].word = word;
        }
    }
    taken
}

/// Safe zero-bounds-check elimination for fixed 64K AoS24 (`inline7` u64 words).
fn eliminate_inline7_aos24_64k<const SPIRALS: bool>(
    nd: &mut [Node24; NARROW_NODES],
    rules: &mut Vec<[u32; 4]>,
    n: u32,
) -> usize {
    rules.clear();
    let mut left = n;
    let mut taken = 0usize;
    while left != 0 {
        let x = nd[0].prev0;
        let y = nd[0].prev1;
        let xi = usize::from(x);
        let yi = usize::from(y);
        let nx = nd[xi];
        let ny = nd[yi];
        let aw = nx.width;
        let bw = ny.width;
        if SPIRALS && x != y {
            if aw > bw && ny.prev1 == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                let head = power_u64(rules, nx.word, q);
                nd[yi].word = join_u64(rules, head, ny.word);
                nd[xi].width = aw - q * bw;
                left -= q * bw;
                taken += q as usize;
                continue;
            }
            if aw < bw && nx.prev0 == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                let tail = power_u64(rules, ny.word, q);
                nd[xi].word = join_u64(rules, nx.word, tail);
                nd[yi].width = bw - q * aw;
                left -= q * aw;
                taken += q as usize;
                continue;
            }
        }
        let w = aw.min(bw);
        left -= w;
        taken += 1;
        if x == y {
            let xp = nx.prev0;
            nd[usize::from(xp)].next0 = 0;
            nd[0].prev0 = xp;
            let yp = ny.prev1;
            nd[usize::from(yp)].next1 = 0;
            nd[0].prev1 = yp;
            continue;
        }
        let word = join_u64(rules, nx.word, ny.word);
        if aw > bw {
            let yp = ny.prev1;
            nd[usize::from(yp)].next1 = 0;
            nd[0].prev1 = yp;
            let xn = nd[xi].next1;
            nd[yi].prev1 = x;
            nd[yi].next1 = xn;
            nd[xi].next1 = y;
            nd[usize::from(xn)].prev1 = y;
            nd[yi].word = word;
            nd[xi].width = aw - bw;
        } else if aw < bw {
            let xp = nx.prev0;
            nd[usize::from(xp)].next0 = 0;
            nd[0].prev0 = xp;
            let yn = nd[yi].next0;
            nd[xi].prev0 = y;
            nd[xi].next0 = yn;
            nd[yi].next0 = x;
            nd[usize::from(yn)].prev0 = x;
            nd[xi].word = word;
            nd[yi].width = bw - aw;
        } else {
            let xp = nx.prev0;
            nd[usize::from(xp)].next0 = 0;
            nd[0].prev0 = xp;
            let yp0 = nd[yi].prev0;
            let yn0 = nd[yi].next0;
            nd[xi].prev0 = yp0;
            nd[xi].next0 = yn0;
            nd[usize::from(yp0)].next0 = x;
            nd[usize::from(yn0)].prev0 = x;
            let yp1 = ny.prev1;
            nd[usize::from(yp1)].next1 = 0;
            nd[0].prev1 = yp1;
            nd[xi].word = word;
        }
    }
    taken
}

fn assign_bottom_links(links: &mut [NodeLinks], widths: &[u32], rows: u32) {
    let mut first = links[0].next1 as usize;
    let mut last = links[0].prev1 as usize;
    let (mut lo, mut hi) = (0u32, rows);
    while first != last {
        let next = links[first].next1 as usize;
        let prev = links[last].prev1 as usize;
        let wf = widths[first];
        hi -= widths[last];
        links[first].prev1 = lo as u16;
        links[first].next1 = (lo >> 16) as u16;
        links[last].prev1 = hi as u16;
        links[last].next1 = (hi >> 16) as u16;
        lo += wf;
        if next == last {
            assert_eq!(lo, hi);
            return;
        }
        first = next;
        last = prev;
    }
    if first != 0 {
        links[first].prev1 = lo as u16;
        links[first].next1 = (lo >> 16) as u16;
        lo += widths[first];
    }
    assert_eq!(lo, hi);
}

fn assign_bottom_soa_64k(
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

fn assign_bottom_aos32(nd: &mut [Node32; NARROW_NODES], rows: u32) {
    let mut first = nd[0].next1;
    let mut last = nd[0].prev1;
    let (mut lo, mut hi) = (0u32, rows);
    while first != last {
        let fi = usize::from(first);
        let li = usize::from(last);
        let next = nd[fi].next1;
        let prev = nd[li].prev1;
        let wf = nd[fi].width;
        hi -= nd[li].width;
        nd[fi].prev1 = lo as u16;
        nd[fi].next1 = (lo >> 16) as u16;
        nd[li].prev1 = hi as u16;
        nd[li].next1 = (hi >> 16) as u16;
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
        nd[fi].prev1 = lo as u16;
        nd[fi].next1 = (lo >> 16) as u16;
        lo += nd[fi].width;
    }
    assert_eq!(lo, hi);
}

fn assign_bottom_aos24(nd: &mut [Node24; NARROW_NODES], rows: u32) {
    let mut first = nd[0].next1;
    let mut last = nd[0].prev1;
    let (mut lo, mut hi) = (0u32, rows);
    while first != last {
        let fi = usize::from(first);
        let li = usize::from(last);
        let next = nd[fi].next1;
        let prev = nd[li].prev1;
        let wf = nd[fi].width;
        hi -= nd[li].width;
        nd[fi].prev1 = lo as u16;
        nd[fi].next1 = (lo >> 16) as u16;
        nd[li].prev1 = hi as u16;
        nd[li].next1 = (hi >> 16) as u16;
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
        nd[fi].prev1 = lo as u16;
        nd[fi].next1 = (lo >> 16) as u16;
        lo += nd[fi].width;
    }
    assert_eq!(lo, hi);
}

// ---------------------------------------------------------------------------
// Dictionary lookups for u128 (inline15) and u64 (inline7)
// ---------------------------------------------------------------------------

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DictMode {
    ZeroFilledDedup,
    GenDedup,
    GenNoContentsDedup,
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn dict_code_u128(
    mode: DictMode,
    word: u128,
    byte: u8,
    rules: &mut [[u32; 4]],
    gen_cache: &mut GenCache,
    zero_pairs: &mut [u32],
    zero_contents: &mut [u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
) -> u32 {
    let tag = (word >> 120) as u32;
    if tag < 16 {
        let entry_u128 = (word & U128_PAYLOAD_MASK) | (u128::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        match mode {
            DictMode::GenNoContentsDedup => {
                let code = lens.len() as u32;
                inline16.push(entry_u128.to_le_bytes());
                lens.push(len);
                code
            }
            DictMode::GenDedup => {
                let idx = (short_u128_hash(entry_u128, len) as usize) & (GEN_CONTENT_SLOTS - 1);
                let g = gen_cache.content_gen;
                let slot = gen_cache.contents[idx];
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
                    gen_cache.contents[idx] = (g << 16) | code;
                }
                code
            }
            DictMode::ZeroFilledDedup => {
                let idx = (short_u128_hash(entry_u128, len) as usize) & (zero_contents.len() - 1);
                let old = zero_contents[idx] as usize;
                if old != 0 && lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128
                {
                    return old as u32;
                }
                let code = lens.len() as u32;
                inline16.push(entry_u128.to_le_bytes());
                lens.push(len);
                zero_contents[idx] = code;
                code
            }
        }
    } else {
        dict_code_rule_long(
            mode,
            word as u32,
            byte,
            rules,
            gen_cache,
            zero_pairs,
            zero_contents,
            inline16,
            lens,
            spelling,
            stack,
        )
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn dict_code_u64(
    mode: DictMode,
    word: u64,
    byte: u8,
    rules: &mut [[u32; 4]],
    gen_cache: &mut GenCache,
    zero_pairs: &mut [u32],
    zero_contents: &mut [u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
) -> u32 {
    let tag = (word >> 56) as u32;
    if tag <= 7 {
        let entry_u64 = (word & U64_PAYLOAD_MASK) | (u64::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        match mode {
            DictMode::GenNoContentsDedup => {
                let code = lens.len() as u32;
                let mut e = [0u8; INLINE];
                e[..8].copy_from_slice(&entry_u64.to_le_bytes());
                inline16.push(e);
                lens.push(len);
                code
            }
            DictMode::GenDedup => {
                let idx = (short_u64_hash(entry_u64, len) as usize) & (GEN_CONTENT_SLOTS - 1);
                let g = gen_cache.content_gen;
                let slot = gen_cache.contents[idx];
                if (slot >> 16) == g {
                    let old = (slot & 0xFFFF) as usize;
                    if lens[old] == len
                        && u64::from_le_bytes(inline16[old][..8].try_into().unwrap()) == entry_u64
                    {
                        return old as u32;
                    }
                }
                let code = lens.len() as u32;
                let mut e = [0u8; INLINE];
                e[..8].copy_from_slice(&entry_u64.to_le_bytes());
                inline16.push(e);
                lens.push(len);
                if code < 65_536 {
                    gen_cache.contents[idx] = (g << 16) | code;
                }
                code
            }
            DictMode::ZeroFilledDedup => unreachable!(),
        }
    } else {
        let id = (word & U64_ID_MASK) as u32;
        let rlen = ((word >> 24) & U64_ID_MASK) as u32;
        if rlen < INLINE as u32 {
            // Short rule (len 8..=15)
            let entry_u128 = rules_short_payload(rules, id) | (u128::from(byte) << (8 * rlen));
            let len = (rlen + 1) as u8;
            match mode {
                DictMode::GenNoContentsDedup => {
                    let code = lens.len() as u32;
                    inline16.push(entry_u128.to_le_bytes());
                    lens.push(len);
                    code
                }
                DictMode::GenDedup => {
                    let idx = (short_u128_hash(entry_u128, len) as usize) & (GEN_CONTENT_SLOTS - 1);
                    let g = gen_cache.content_gen;
                    let slot = gen_cache.contents[idx];
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
                        gen_cache.contents[idx] = (g << 16) | code;
                    }
                    code
                }
                DictMode::ZeroFilledDedup => unreachable!(),
            }
        } else {
            dict_code_rule_long(
                mode,
                id,
                byte,
                rules,
                gen_cache,
                zero_pairs,
                zero_contents,
                inline16,
                lens,
                spelling,
                stack,
            )
        }
    }
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn dict_code_rule_long(
    mode: DictMode,
    id: u32,
    byte: u8,
    rules: &mut [[u32; 4]],
    gen_cache: &mut GenCache,
    zero_pairs: &mut [u32],
    zero_contents: &mut [u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
) -> u32 {
    let key = (id << 8) | u32::from(byte);
    let p_idx = (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
    match mode {
        DictMode::ZeroFilledDedup => {
            let slot = 2 * p_idx;
            if zero_pairs[slot] == key {
                return zero_pairs[slot + 1];
            }
        }
        DictMode::GenDedup | DictMode::GenNoContentsDedup => {
            if id < (1 << 22) {
                let packed = gen_cache.pairs[p_idx];
                if (packed >> 46) == gen_cache.pair_gen && ((packed as u32) & 0x3FFF_FFFF) == key {
                    return ((packed >> 30) & 0xFFFF) as u32;
                }
            }
        }
    }

    let start = spelling.len();
    let memoized = rules[(id - 257) as usize][3] != u32::MAX;
    append_rules(rules, id, stack, spelling);
    spelling.push(byte);
    let end = spelling.len();
    let entry = long_slot(start as u32, end as u32);

    let code = match mode {
        DictMode::GenNoContentsDedup => {
            let code = lens.len() as u32;
            inline16.push(entry);
            lens.push(LONG);
            code
        }
        DictMode::GenDedup => {
            let bytes = &spelling[start..end];
            let idx = (long_bytes_hash(bytes) as usize) & (GEN_CONTENT_SLOTS - 1);
            let g = gen_cache.content_gen;
            let slot = gen_cache.contents[idx];
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
                gen_cache.contents[idx] = (g << 16) | code;
            }
            code
        }
        DictMode::ZeroFilledDedup => {
            let bytes = &spelling[start..end];
            let idx = (long_bytes_hash(bytes) as usize) & (zero_contents.len() - 1);
            let old = zero_contents[idx] as usize;
            if old != 0 && lens[old] == LONG && long_word(&inline16[old], spelling) == bytes {
                if memoized {
                    spelling.truncate(start);
                } else {
                    spelling.pop();
                }
                let code = old as u32;
                zero_pairs[2 * p_idx] = key;
                zero_pairs[2 * p_idx + 1] = code;
                return code;
            }
            let code = lens.len() as u32;
            inline16.push(entry);
            lens.push(LONG);
            zero_contents[idx] = code;
            code
        }
    };

    match mode {
        DictMode::ZeroFilledDedup => {
            zero_pairs[2 * p_idx] = key;
            zero_pairs[2 * p_idx + 1] = code;
        }
        DictMode::GenDedup | DictMode::GenNoContentsDedup => {
            if id < (1 << 22) && code < 65_536 {
                gen_cache.pairs[p_idx] =
                    (gen_cache.pair_gen << 46) | (u64::from(code) << 30) | u64::from(key);
            }
        }
    }
    code
}

// ---------------------------------------------------------------------------
// Emission kernels (exact mirrors of src/open.rs emit & emit_bidirectional)
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

// ---------------------------------------------------------------------------
// Arm definitions & ProbeKernel
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum A6bArm {
    ShippedBaseline,
    BaselineNoStats,
    SetupU16Fused,
    Inline15SoaZeroCache,
    CombinedSoaGen,
    CombinedNoBcSoaGen,
    CombinedNoBcSoaNoDedup,
    CombinedNoBcAos32Gen,
    CombinedNoBcAos24I7Gen,
}

impl A6bArm {
    fn name(self) -> &'static str {
        match self {
            A6bArm::ShippedBaseline => "shipped_baseline",
            A6bArm::BaselineNoStats => "baseline_no_stats",
            A6bArm::SetupU16Fused => "setup_u16_fused",
            A6bArm::Inline15SoaZeroCache => "inline15_soa_zcache",
            A6bArm::CombinedSoaGen => "combined_soa_gen",
            A6bArm::CombinedNoBcSoaGen => "comb_nobc_soa_gen",
            A6bArm::CombinedNoBcSoaNoDedup => "comb_nobc_soa_nod",
            A6bArm::CombinedNoBcAos32Gen => "comb_nobc_aos32_gen",
            A6bArm::CombinedNoBcAos24I7Gen => "comb_nobc_i7_gen",
        }
    }
}

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
    fn ce(self) -> u64 {
        self.a6a + self.a6b() + self.a6c + self.a7
    }
}

#[derive(Default, Clone, Copy)]
struct RunMetrics {
    steps: usize,
    rules: usize,
    bands: usize,
    dict_entries: usize,
    long_entries: usize,
    spelling_bytes: usize,
    wide_edges: usize,
    wide_arena: usize,
    spirals: usize,
}

#[derive(Default)]
struct ProbeKernel {
    spill: Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    redesign: RedesignState,
}

impl ProbeKernel {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn arena_words(ns: usize, m: usize) -> usize {
        let most = Self::nodes_most(ns);
        let stride = if most > NARROW_NODES {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (4 * ns + (stride + RULE) * most).max(m)
    }

    #[allow(clippy::too_many_arguments)]
    fn run_block(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        arm: A6bArm,
        force_forward: bool,
        out: &mut Vec<u8>,
        metrics: &mut RunMetrics,
    ) -> PhaseNs {
        let mut ph = PhaseNs::default();

        // --- A6a: first pass (identical across all arms) ---
        let t0 = Instant::now();
        out.resize(n + INLINE, 0);
        let mut pass = Pass {
            queues: Queues::new(&mut out[..], &mut self.spill, Cap::NONE.m),
            edges: &mut self.edges,
            ns: 0,
            cap_strips: Cap::NONE.strips,
            f: First::default(),
        };
        maximal(records, |c, pos, width| {
            pass.take(c, pos, width, p, PROD_LIMIT);
        });
        let ns = pass.ns;
        let f = pass.f;
        let heads = pass.queues.heads();
        let t1 = Instant::now();
        ph.a6a = t1.duration_since(t0).as_nanos() as u64;

        let First {
            thick_rows,
            thin,
            thick,
            m,
            mark,
            ..
        } = f;
        let most = Self::nodes_most(ns);
        let wide_arena = most > NARROW_NODES;
        if wide_arena {
            metrics.wide_arena += 1;
        }

        // --- A6b: setup + steps + dict ---
        let t_setup_start = Instant::now();
        let all: [u32; 256] = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        let first = starts(all);
        let mut write = starts(thick);

        self.bands.clear();
        self.spelling.clear();
        if n >= 65_536 {
            self.spelling.reserve_exact(48 * 1024);
        }

        let cursors =
            if wide_arena || matches!(arm, A6bArm::ShippedBaseline | A6bArm::BaselineNoStats) {
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
                let track_stats = arm == A6bArm::ShippedBaseline;
                if wide_arena {
                    self.induced_shipped::<Wide>(
                        ns,
                        n as u32,
                        &thin,
                        m,
                        most,
                        track_stats,
                        t_setup_start,
                        &mut ph,
                        metrics,
                    )
                } else {
                    self.induced_shipped::<Narrow>(
                        ns,
                        n as u32,
                        &thin,
                        m,
                        most,
                        track_stats,
                        t_setup_start,
                        &mut ph,
                        metrics,
                    )
                }
            } else {
                self.induced_redesigned(
                    arm,
                    ns,
                    n as u32,
                    &thin,
                    m,
                    most,
                    &first,
                    &mut write,
                    t_setup_start,
                    &mut ph,
                    metrics,
                )
            };

        metrics.bands += self.bands.len();
        metrics.dict_entries += self.lens.len();
        metrics.long_entries += self.lens.iter().filter(|&&l| l == LONG).count();
        metrics.spelling_bytes += self.spelling.len();

        // --- A6c: retained-row wiring ---
        let t_a6c = Instant::now();
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide_edges = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if wide_edges {
            metrics.wide_edges += 1;
        }
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide_edges {
            self.extra.resize(m as usize, 0);
        }
        let mut predecessor = None;
        if !force_forward && !wide_edges && m >= 65_536 {
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
            predecessor = Some(pred);
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
        }
        let t_a7 = Instant::now();
        ph.a6c = t_a7.duration_since(t_a6c).as_nanos() as u64;

        // --- A7: walk + spelling ---
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
        let t_end = Instant::now();
        ph.a7 = t_end.duration_since(t_a7).as_nanos() as u64;

        ph
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_shipped<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        m: u32,
        most: usize,
        track_stats: bool,
        t_setup_start: Instant,
        ph: &mut PhaseNs,
        metrics: &mut RunMetrics,
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
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;

        let t_steps_start = Instant::now();
        ph.a6b_setup = t_steps_start.duration_since(t_setup_start).as_nanos() as u64;

        let (mut remaining, mut steps) = (n, 0usize);
        let spirals = widest > CERTIFIED_WIDTH;
        if spirals {
            metrics.spirals += 1;
        }
        while !if spirals {
            e.eliminate::<true>(&mut remaining, &mut steps)
        } else {
            e.eliminate::<false>(&mut remaining, &mut steps)
        } {
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

        metrics.steps += steps;
        metrics.rules += (e.top - e.base) / RULE;

        e.assign_bottom(m);
        const PAIR_WORDS: usize = 2 * 1024;
        const CACHE_WORDS: usize = PAIR_WORDS + 4096;
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail
        };
        let (pairs, contents) = if cache.len() >= CACHE_WORDS {
            cache[..CACHE_WORDS].fill(0);
            cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS)
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
        begin_inline(
            if n >= 65_536 { 1536 } else { 0 },
            &mut self.inline16,
            &mut self.lens,
        );
        let mut dictionary = Dictionary {
            pairs,
            contents,
            inline16: &mut self.inline16,
            lens: &mut self.lens,
            spelling: &mut self.spelling,
            stack: &mut self.stack,
        };
        let mut stats = ShippedStats::default();
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
                assert!(width > 0);
                let dst = e.bottom(id) + in_node;
                if track_stats && node_word != 0 {
                    let grade = node_word >> ID_BITS;
                    let height = if grade == SATURATED {
                        e.len(node_word & ID_MASK)
                    } else {
                        grade
                    } as usize;
                    stats.words += 1;
                    stats.copies += width as usize;
                    stats.traced += height;
                    stats.thick_area += width as usize * height;
                    stats.unit_work += usize::from(width == 1) * height;
                    stats.longest = stats.longest.max(height);
                }
                let code = dictionary.code(&mut e, node_word, a as u8);
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
        if track_stats {
            black_box(stats);
        }
        let t_dict_end = Instant::now();
        ph.a6b_dict = t_dict_end.duration_since(t_dict_start).as_nanos() as u64;
        cursors
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_redesigned(
        &mut self,
        arm: A6bArm,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        m: u32,
        most: usize,
        first: &[u32; 256],
        write: &mut [u32; 256],
        t_setup_start: Instant,
        ph: &mut PhaseNs,
        metrics: &mut RunMetrics,
    ) -> [usize; 256] {
        let strips_raw = &self.edges[..3 * ns];
        match arm {
            A6bArm::SetupU16Fused => {
                let (nodes, widest) = setup_soa_vec::<false>(
                    strips_raw,
                    ns,
                    n,
                    first,
                    write,
                    most,
                    &mut self.redesign,
                );
                let t_steps_start = Instant::now();
                ph.a6b_setup = t_steps_start.duration_since(t_setup_start).as_nanos() as u64;

                let spirals = widest > CERTIFIED_WIDTH;
                if spirals {
                    metrics.spirals += 1;
                }
                let steps = if spirals {
                    eliminate_u32_soa::<true>(
                        &mut self.redesign.links[..nodes],
                        &mut self.redesign.widths[..nodes],
                        &mut self.redesign.words32[..nodes],
                        &mut self.redesign.rules,
                        n,
                    )
                } else {
                    eliminate_u32_soa::<false>(
                        &mut self.redesign.links[..nodes],
                        &mut self.redesign.widths[..nodes],
                        &mut self.redesign.words32[..nodes],
                        &mut self.redesign.rules,
                        n,
                    )
                };
                let t_dict_start = Instant::now();
                ph.a6b_steps = t_dict_start.duration_since(t_steps_start).as_nanos() as u64;
                metrics.steps += steps;
                metrics.rules += self.redesign.rules.len();

                assign_bottom_links(
                    &mut self.redesign.links[..nodes],
                    &self.redesign.widths[..nodes],
                    m,
                );
                self.redesign.zero_pairs.resize(2 * 1024, 0);
                self.redesign.zero_pairs.fill(0);
                self.redesign.zero_contents.resize(4096, 0);
                self.redesign.zero_contents.fill(0);
                begin_inline(
                    if n >= 65_536 { 1536 } else { 0 },
                    &mut self.inline16,
                    &mut self.lens,
                );
                let mut cursors = [0usize; 256];
                let links = &self.redesign.links[..nodes];
                let widths = &self.redesign.widths[..nodes];
                let words = &self.redesign.words32[..nodes];
                let rules = &mut self.redesign.rules[..];
                let pairs = &mut self.redesign.zero_pairs[..];
                let contents = &mut self.redesign.zero_contents[..];

                let mut id = links[0].next0 as usize;
                let mut in_node = 0u32;
                let mut k = 0u32;
                for a in 0..256 {
                    cursors[a] = self.bands.len();
                    let end = k + thin[a];
                    while k < end {
                        let node_width = widths[id];
                        let node_word = words[id];
                        let width = (end - k).min(node_width - in_node);
                        let l = links[id];
                        let dst = (u32::from(l.prev1) | (u32::from(l.next1) << 16)) + in_node;
                        let code = if node_word == 0 {
                            a as u32
                        } else {
                            let wid = node_word & ID_MASK;
                            let key = (wid << 8) | (a as u32);
                            let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
                            if pairs[slot] == key {
                                pairs[slot + 1]
                            } else {
                                let start = self.spelling.len();
                                let height = (node_word >> ID_BITS) as usize;
                                let memoized =
                                    height >= INLINE && rules[(wid - 257) as usize][3] != u32::MAX;
                                let (entry, len) = if height < INLINE {
                                    let mut b = rules_short_payload(rules, wid).to_le_bytes();
                                    b[height] = a as u8;
                                    (b, (height + 1) as u8)
                                } else {
                                    append_rules(rules, wid, &mut self.stack, &mut self.spelling);
                                    self.spelling.push(a as u8);
                                    (long_slot(start as u32, self.spelling.len() as u32), LONG)
                                };
                                let bytes = if len == LONG {
                                    &self.spelling[start..]
                                } else {
                                    &entry[..len as usize]
                                };
                                let index = spelling_hash(&entry, len, bytes) as usize
                                    & (contents.len() - 1);
                                let old = contents[index] as usize;
                                let same = old != 0
                                    && self.lens[old] == len
                                    && if len == LONG {
                                        long_word(&self.inline16[old], &self.spelling) == bytes
                                    } else {
                                        self.inline16[old] == entry
                                    };
                                let code = if same {
                                    if len == LONG {
                                        if memoized {
                                            self.spelling.truncate(start);
                                        } else {
                                            self.spelling.pop();
                                        }
                                    }
                                    old as u32
                                } else {
                                    let code = self.lens.len() as u32;
                                    self.inline16.push(entry);
                                    self.lens.push(len);
                                    contents[index] = code;
                                    code
                                };
                                pairs[slot] = key;
                                pairs[slot + 1] = code;
                                code
                            }
                        };
                        self.bands.push(Band::new(k + width, dst, code));
                        in_node += width;
                        k += width;
                        if in_node == node_width {
                            id = l.next0 as usize;
                            in_node = 0;
                        }
                    }
                }
                assert_eq!(id, 0);
                let t_dict_end = Instant::now();
                ph.a6b_dict = t_dict_end.duration_since(t_dict_start).as_nanos() as u64;
                cursors
            }
            A6bArm::Inline15SoaZeroCache | A6bArm::CombinedSoaGen => {
                let (nodes, widest) = setup_soa_vec::<true>(
                    strips_raw,
                    ns,
                    n,
                    first,
                    write,
                    most,
                    &mut self.redesign,
                );
                let t_steps_start = Instant::now();
                ph.a6b_setup = t_steps_start.duration_since(t_setup_start).as_nanos() as u64;

                let spirals = widest > CERTIFIED_WIDTH;
                if spirals {
                    metrics.spirals += 1;
                }
                let steps = if spirals {
                    eliminate_inline15_soa_vec::<true>(
                        &mut self.redesign.links[..nodes],
                        &mut self.redesign.widths[..nodes],
                        &mut self.redesign.words128[..nodes],
                        &mut self.redesign.rules,
                        n,
                    )
                } else {
                    eliminate_inline15_soa_vec::<false>(
                        &mut self.redesign.links[..nodes],
                        &mut self.redesign.widths[..nodes],
                        &mut self.redesign.words128[..nodes],
                        &mut self.redesign.rules,
                        n,
                    )
                };
                let t_dict_start = Instant::now();
                ph.a6b_steps = t_dict_start.duration_since(t_steps_start).as_nanos() as u64;
                metrics.steps += steps;
                metrics.rules += self.redesign.rules.len();

                assign_bottom_links(
                    &mut self.redesign.links[..nodes],
                    &self.redesign.widths[..nodes],
                    m,
                );
                let dict_mode = if arm == A6bArm::Inline15SoaZeroCache {
                    self.redesign.zero_pairs.resize(2 * 1024, 0);
                    self.redesign.zero_pairs.fill(0);
                    self.redesign.zero_contents.resize(4096, 0);
                    self.redesign.zero_contents.fill(0);
                    DictMode::ZeroFilledDedup
                } else {
                    self.redesign.gen_cache.next_block();
                    DictMode::GenDedup
                };
                begin_inline(
                    if n >= 65_536 { 1536 } else { 0 },
                    &mut self.inline16,
                    &mut self.lens,
                );
                let mut cursors = [0usize; 256];
                let links = &self.redesign.links[..nodes];
                let widths = &self.redesign.widths[..nodes];
                let words = &self.redesign.words128[..nodes];
                let rules = &mut self.redesign.rules[..];

                let mut id = links[0].next0 as usize;
                let mut node_width = widths[id];
                let mut node_word = words[id];
                let l0 = links[id];
                let mut node_bottom = u32::from(l0.prev1) | (u32::from(l0.next1) << 16);
                let mut node_next = l0.next0 as usize;
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
                            dict_code_u128(
                                dict_mode,
                                node_word,
                                a as u8,
                                rules,
                                &mut self.redesign.gen_cache,
                                &mut self.redesign.zero_pairs,
                                &mut self.redesign.zero_contents,
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
                                let l = links[id];
                                node_width = widths[id];
                                node_word = words[id];
                                node_bottom = u32::from(l.prev1) | (u32::from(l.next1) << 16);
                                node_next = l.next0 as usize;
                            }
                        }
                    }
                }
                assert_eq!(id, 0);
                let t_dict_end = Instant::now();
                ph.a6b_dict = t_dict_end.duration_since(t_dict_start).as_nanos() as u64;
                cursors
            }
            A6bArm::CombinedNoBcSoaGen | A6bArm::CombinedNoBcSoaNoDedup => {
                let soa = self.redesign.fixed_soa.get_or_insert_with(Fixed64kSoa::new);
                let widest = setup_soa_64k(
                    strips_raw,
                    ns,
                    n,
                    first,
                    write,
                    &mut self.redesign.image_order64,
                    soa,
                );
                let t_steps_start = Instant::now();
                ph.a6b_setup = t_steps_start.duration_since(t_setup_start).as_nanos() as u64;

                let spirals = widest > CERTIFIED_WIDTH;
                if spirals {
                    metrics.spirals += 1;
                }
                let steps = if spirals {
                    eliminate_inline15_soa_64k::<true>(soa, &mut self.redesign.rules, n)
                } else {
                    eliminate_inline15_soa_64k::<false>(soa, &mut self.redesign.rules, n)
                };
                let t_dict_start = Instant::now();
                ph.a6b_steps = t_dict_start.duration_since(t_steps_start).as_nanos() as u64;
                metrics.steps += steps;
                metrics.rules += self.redesign.rules.len();

                assign_bottom_soa_64k(&mut soa.links, &soa.widths, m);
                self.redesign.gen_cache.next_block();
                let dict_mode = if arm == A6bArm::CombinedNoBcSoaGen {
                    DictMode::GenDedup
                } else {
                    DictMode::GenNoContentsDedup
                };
                begin_inline(
                    if n >= 65_536 { 1536 } else { 0 },
                    &mut self.inline16,
                    &mut self.lens,
                );
                let mut cursors = [0usize; 256];
                let links = &*soa.links;
                let widths = &*soa.widths;
                let words = &*soa.words128;
                let rules = &mut self.redesign.rules[..];

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
                            dict_code_u128(
                                dict_mode,
                                node_word,
                                a as u8,
                                rules,
                                &mut self.redesign.gen_cache,
                                &mut self.redesign.zero_pairs,
                                &mut self.redesign.zero_contents,
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
                                let idx = usize::from(id);
                                let l = links[idx];
                                node_width = widths[idx];
                                node_word = words[idx];
                                node_bottom = u32::from(l.prev1) | (u32::from(l.next1) << 16);
                                node_next = l.next0;
                            }
                        }
                    }
                }
                assert_eq!(id, 0);
                let t_dict_end = Instant::now();
                ph.a6b_dict = t_dict_end.duration_since(t_dict_start).as_nanos() as u64;
                cursors
            }
            A6bArm::CombinedNoBcAos32Gen => {
                let nd = self
                    .redesign
                    .fixed_aos32
                    .get_or_insert_with(|| boxed_array_64k(Node32::default()));
                let widest = setup_aos32_64k(
                    strips_raw,
                    ns,
                    n,
                    first,
                    write,
                    &mut self.redesign.image_order64,
                    nd,
                );
                let t_steps_start = Instant::now();
                ph.a6b_setup = t_steps_start.duration_since(t_setup_start).as_nanos() as u64;

                let spirals = widest > CERTIFIED_WIDTH;
                if spirals {
                    metrics.spirals += 1;
                }
                let steps = if spirals {
                    eliminate_inline15_aos32_64k::<true>(nd, &mut self.redesign.rules, n)
                } else {
                    eliminate_inline15_aos32_64k::<false>(nd, &mut self.redesign.rules, n)
                };
                let t_dict_start = Instant::now();
                ph.a6b_steps = t_dict_start.duration_since(t_steps_start).as_nanos() as u64;
                metrics.steps += steps;
                metrics.rules += self.redesign.rules.len();

                assign_bottom_aos32(nd, m);
                self.redesign.gen_cache.next_block();
                let dict_mode = DictMode::GenDedup;
                begin_inline(
                    if n >= 65_536 { 1536 } else { 0 },
                    &mut self.inline16,
                    &mut self.lens,
                );
                let mut cursors = [0usize; 256];
                let rules = &mut self.redesign.rules[..];

                let mut id = nd[0].next0;
                let mut cur = nd[usize::from(id)];
                let mut node_bottom = u32::from(cur.prev1) | (u32::from(cur.next1) << 16);
                let mut in_node = 0u32;
                let mut k = 0u32;

                for a in 0..256 {
                    cursors[a] = self.bands.len();
                    let end = k + thin[a];
                    while k < end {
                        let width = (end - k).min(cur.width - in_node);
                        let dst = node_bottom + in_node;
                        let code = if cur.word == 0 {
                            a as u32
                        } else {
                            dict_code_u128(
                                dict_mode,
                                cur.word,
                                a as u8,
                                rules,
                                &mut self.redesign.gen_cache,
                                &mut self.redesign.zero_pairs,
                                &mut self.redesign.zero_contents,
                                &mut self.inline16,
                                &mut self.lens,
                                &mut self.spelling,
                                &mut self.stack,
                            )
                        };
                        self.bands.push(Band::new(k + width, dst, code));
                        in_node += width;
                        k += width;
                        if in_node == cur.width {
                            id = cur.next0;
                            in_node = 0;
                            if id != 0 {
                                cur = nd[usize::from(id)];
                                node_bottom = u32::from(cur.prev1) | (u32::from(cur.next1) << 16);
                            }
                        }
                    }
                }
                assert_eq!(id, 0);
                let t_dict_end = Instant::now();
                ph.a6b_dict = t_dict_end.duration_since(t_dict_start).as_nanos() as u64;
                cursors
            }
            A6bArm::CombinedNoBcAos24I7Gen => {
                let nd = self
                    .redesign
                    .fixed_aos24
                    .get_or_insert_with(|| boxed_array_64k(Node24::default()));
                let widest = setup_aos24_64k(
                    strips_raw,
                    ns,
                    n,
                    first,
                    write,
                    &mut self.redesign.image_order64,
                    nd,
                );
                let t_steps_start = Instant::now();
                ph.a6b_setup = t_steps_start.duration_since(t_setup_start).as_nanos() as u64;

                let spirals = widest > CERTIFIED_WIDTH;
                if spirals {
                    metrics.spirals += 1;
                }
                let steps = if spirals {
                    eliminate_inline7_aos24_64k::<true>(nd, &mut self.redesign.rules, n)
                } else {
                    eliminate_inline7_aos24_64k::<false>(nd, &mut self.redesign.rules, n)
                };
                let t_dict_start = Instant::now();
                ph.a6b_steps = t_dict_start.duration_since(t_steps_start).as_nanos() as u64;
                metrics.steps += steps;
                metrics.rules += self.redesign.rules.len();

                assign_bottom_aos24(nd, m);
                self.redesign.gen_cache.next_block();
                let dict_mode = DictMode::GenDedup;
                begin_inline(
                    if n >= 65_536 { 1536 } else { 0 },
                    &mut self.inline16,
                    &mut self.lens,
                );
                let mut cursors = [0usize; 256];
                let rules = &mut self.redesign.rules[..];

                let mut id = nd[0].next0;
                let mut cur = nd[usize::from(id)];
                let mut node_bottom = u32::from(cur.prev1) | (u32::from(cur.next1) << 16);
                let mut in_node = 0u32;
                let mut k = 0u32;

                for a in 0..256 {
                    cursors[a] = self.bands.len();
                    let end = k + thin[a];
                    while k < end {
                        let width = (end - k).min(cur.width - in_node);
                        let dst = node_bottom + in_node;
                        let code = if cur.word == 0 {
                            a as u32
                        } else {
                            dict_code_u64(
                                dict_mode,
                                cur.word,
                                a as u8,
                                rules,
                                &mut self.redesign.gen_cache,
                                &mut self.redesign.zero_pairs,
                                &mut self.redesign.zero_contents,
                                &mut self.inline16,
                                &mut self.lens,
                                &mut self.spelling,
                                &mut self.stack,
                            )
                        };
                        self.bands.push(Band::new(k + width, dst, code));
                        in_node += width;
                        k += width;
                        if in_node == cur.width {
                            id = cur.next0;
                            in_node = 0;
                            if id != 0 {
                                cur = nd[usize::from(id)];
                                node_bottom = u32::from(cur.prev1) | (u32::from(cur.next1) << 16);
                            }
                        }
                    }
                }
                assert_eq!(id, 0);
                let t_dict_end = Instant::now();
                ph.a6b_dict = t_dict_end.duration_since(t_dict_start).as_nanos() as u64;
                cursors
            }
            A6bArm::ShippedBaseline | A6bArm::BaselineNoStats => unreachable!(),
        }
    }
}

// ---------------------------------------------------------------------------
// Corpus loading & benchmark harness
// ---------------------------------------------------------------------------

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

#[derive(Clone, Copy)]
struct ArmSummary {
    a6a_med: f64,
    setup_med: f64,
    steps_med: f64,
    dict_med: f64,
    a6b_med: f64,
    a6b_min: f64,
    a6b_max: f64,
    a6c_med: f64,
    a7_med: f64,
    ce_med: f64,
    ce_min: f64,
    ce_max: f64,
    metrics: RunMetrics,
}

fn run_suite(
    suite_label: &str,
    arms: &[A6bArm],
    force_forward: bool,
    all_blocks: &[BlockInput],
    admitted_blocks: &[&BlockInput],
    repeats: usize,
) {
    let total_rows: usize = admitted_blocks.iter().map(|b| b.n).sum();
    println!("\n================================================================================");
    println!(
        "SUITE: {} (force_forward={}, admitted_blocks={}, rows={}, repeats={})",
        suite_label,
        force_forward,
        admitted_blocks.len(),
        total_rows,
        repeats
    );
    println!("================================================================================");

    let mut kernel = ProbeKernel::default();
    let mut out = Vec::new();
    for &arm in arms {
        let mut dummy_metrics = RunMetrics::default();
        let mut checked = 0usize;
        for b in all_blocks {
            if b.n == 0 || b.n > 900_000 || b.orig_ptr as usize >= b.n {
                continue;
            }
            kernel.run_block(
                &b.records,
                b.n,
                b.orig_ptr,
                arm,
                force_forward,
                &mut out,
                &mut dummy_metrics,
            );
            assert_eq!(
                out.as_slice(),
                b.walked.as_slice(),
                "EXACTNESS FAILURE: arm={} force_forward={} block={} (n={}, admitted={})",
                arm.name(),
                force_forward,
                b.index,
                b.n,
                b.admitted
            );
            checked += 1;
        }
        assert_eq!(checked, all_blocks.len());
    }
    println!(
        "Exactness: PASS ({}/{} blocks 100% byte-identical to bwt::walk for all {} arms)",
        all_blocks.len(),
        all_blocks.len(),
        arms.len()
    );

    let mut per_arm_runs: Vec<Vec<PhaseNs>> = vec![Vec::with_capacity(repeats); arms.len()];
    let mut per_arm_metrics: Vec<RunMetrics> = vec![RunMetrics::default(); arms.len()];

    // Warm-up pass
    for &arm in arms {
        let mut m_tmp = RunMetrics::default();
        for b in admitted_blocks {
            kernel.run_block(
                &b.records,
                b.n,
                b.orig_ptr,
                arm,
                force_forward,
                &mut out,
                &mut m_tmp,
            );
            black_box(out.as_slice());
        }
    }

    let cpu_clock = CpuClock::new();
    let mut per_arm_cpu: Vec<Vec<u64>> = vec![Vec::with_capacity(repeats); arms.len()];
    for r in 0..repeats {
        for (ai, &arm) in arms.iter().enumerate() {
            let mut sum = PhaseNs::default();
            let mut met = RunMetrics::default();
            let c0 = cpu_clock.now_ns();
            for b in admitted_blocks {
                let ph = kernel.run_block(
                    &b.records,
                    b.n,
                    b.orig_ptr,
                    arm,
                    force_forward,
                    &mut out,
                    &mut met,
                );
                black_box(out.as_slice());
                sum.add(ph);
            }
            let c1 = cpu_clock.now_ns();
            per_arm_runs[ai].push(sum);
            per_arm_cpu[ai].push(c1.saturating_sub(c0));
            if r == 0 {
                per_arm_metrics[ai] = met;
            }
        }
    }

    let to_ms = |ns: u64| ns as f64 / 1e6;
    let summaries: Vec<ArmSummary> = arms
        .iter()
        .enumerate()
        .map(|(ai, _)| {
            let runs = &per_arm_runs[ai];
            let (a6a_med, _, _) =
                median_min_max(&runs.iter().map(|p| to_ms(p.a6a)).collect::<Vec<_>>());
            let (setup_med, _, _) =
                median_min_max(&runs.iter().map(|p| to_ms(p.a6b_setup)).collect::<Vec<_>>());
            let (steps_med, _, _) =
                median_min_max(&runs.iter().map(|p| to_ms(p.a6b_steps)).collect::<Vec<_>>());
            let (dict_med, _, _) =
                median_min_max(&runs.iter().map(|p| to_ms(p.a6b_dict)).collect::<Vec<_>>());
            let (a6b_med, a6b_min, a6b_max) =
                median_min_max(&runs.iter().map(|p| to_ms(p.a6b())).collect::<Vec<_>>());
            let (a6c_med, _, _) =
                median_min_max(&runs.iter().map(|p| to_ms(p.a6c)).collect::<Vec<_>>());
            let (a7_med, _, _) =
                median_min_max(&runs.iter().map(|p| to_ms(p.a7)).collect::<Vec<_>>());
            let (ce_med, ce_min, ce_max) =
                median_min_max(&runs.iter().map(|p| to_ms(p.ce())).collect::<Vec<_>>());
            ArmSummary {
                a6a_med,
                setup_med,
                steps_med,
                dict_med,
                a6b_med,
                a6b_min,
                a6b_max,
                a6c_med,
                a7_med,
                ce_med,
                ce_min,
                ce_max,
                metrics: per_arm_metrics[ai],
            }
        })
        .collect();
    let cpu_summaries: Vec<(f64, f64, f64)> = arms
        .iter()
        .enumerate()
        .map(|(ai, _)| {
            median_min_max(
                &per_arm_cpu[ai]
                    .iter()
                    .map(|&ns| to_ms(ns))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    let base = summaries[0];
    let base_cpu = cpu_summaries[0];

    println!(
        "\n--- Sub-phase medians (ms over {} admitted blocks = 1x wikidata) ---",
        admitted_blocks.len()
    );
    println!(
        "{:<24} {:>7} {:>7} {:>7} {:>7} {:>20} {:>7} {:>7} {:>7} {:>20} {:>20}",
        "arm",
        "A6a",
        "setup",
        "steps",
        "dict",
        "A6b [min..max]",
        "A6b_rat",
        "A6c",
        "A7",
        "C+E_wall [min..max]",
        "C+E_cpu [min..max]"
    );
    for ((arm, s), &(c_med, c_min, c_max)) in arms.iter().zip(&summaries).zip(&cpu_summaries) {
        println!(
            "{:<24} {:>7.2} {:>7.2} {:>7.2} {:>7.2} {:>6.2} [{:.2}..{:.2}] {:>6.4}x {:>7.2} {:>7.2} {:>6.2} [{:.2}..{:.2}] {:>6.2} [{:.2}..{:.2}] ({:.4}x)",
            arm.name(),
            s.a6a_med,
            s.setup_med,
            s.steps_med,
            s.dict_med,
            s.a6b_med,
            s.a6b_min,
            s.a6b_max,
            s.a6b_med / base.a6b_med,
            s.a6c_med,
            s.a7_med,
            s.ce_med,
            s.ce_min,
            s.ce_max,
            c_med,
            c_min,
            c_max,
            c_med / base_cpu.0,
        );
    }

    println!(
        "\n--- i9-10900K projections & gate check (gate >= {:.2} ms = {:.1}% of {:.0} ms 1W wall) ---",
        GATE_MS, GATE_PCT, I9_WALL_1W_MS
    );
    println!(
        "{:<24} {:>8} {:>8} {:>9} {:>9} {:>9} {:>8} {:>9} {:>7} {:>7}",
        "arm",
        "C+E_rat",
        "A7_Δ%",
        "i9_sub_Δ",
        "i9_anc_Δ",
        "i9_prp_Δ",
        "%1w_anc",
        "8w_wkr_Δ",
        "A6b_ok",
        "CE_ok"
    );
    for (arm, s) in arms.iter().zip(&summaries) {
        let ce_rat = s.ce_med / base.ce_med;
        let a7_reg_pct = (s.a7_med / base.a7_med - 1.0) * 100.0;

        let i9_setup_save = I9_A6B_SETUP_MS * (1.0 - s.setup_med / base.setup_med);
        let i9_steps_save = I9_A6B_STEPS_MS * (1.0 - s.steps_med / base.steps_med);
        let i9_dict_save = I9_A6B_DICT_MS * (1.0 - s.dict_med / base.dict_med);
        let i9_a7_save = I9_A7_MS * (1.0 - s.a7_med / base.a7_med);
        let i9_sub_delta = i9_setup_save + i9_steps_save + i9_dict_save + i9_a7_save;

        let i9_anc_delta = I9_A6A_MS * (1.0 - s.a6a_med / base.a6a_med)
            + I9_A6B_MS * (1.0 - s.a6b_med / base.a6b_med)
            + I9_A6C_MS * (1.0 - s.a6c_med / base.a6c_med)
            + I9_A7_MS * (1.0 - s.a7_med / base.a7_med);

        let i9_prp_delta = I9_CE_MS * (1.0 - ce_rat);
        let pct_1w_anc = i9_anc_delta / I9_WALL_1W_MS * 100.0;
        let w8_delta = I9_8W_A6B_MS * (1.0 - s.a6b_med / base.a6b_med)
            + I9_8W_A7_MS * (1.0 - s.a7_med / base.a7_med);

        let a6b_non_overlap = s.a6b_max < base.a6b_min;
        let ce_non_overlap = s.ce_max < base.ce_min;
        println!(
            "{:<24} {:>7.4}x {:>+7.2}% {:>+8.2}m {:>+8.2}m {:>+8.2}m {:>+7.2}% {:>+8.2}m {:>7} {:>7}",
            arm.name(),
            ce_rat,
            a7_reg_pct,
            i9_sub_delta,
            i9_anc_delta,
            i9_prp_delta,
            pct_1w_anc,
            w8_delta,
            if a6b_non_overlap { "DISJ" } else { "ovlp" },
            if ce_non_overlap { "DISJ" } else { "ovlp" },
        );
    }

    println!(
        "\n--- Structural metrics (summed over {} admitted blocks) ---",
        admitted_blocks.len()
    );
    println!(
        "{:<24} {:>10} {:>10} {:>10} {:>10} {:>10} {:>11} {:>6} {:>6}",
        "arm", "steps", "rules", "bands", "dict_ent", "long_ent", "spell_KiB", "wideE", "wideA"
    );
    for (arm, s) in arms.iter().zip(&summaries) {
        let m = s.metrics;
        println!(
            "{:<24} {:>10} {:>10} {:>10} {:>10} {:>10} {:>11.1} {:>6} {:>6}",
            arm.name(),
            m.steps,
            m.rules,
            m.bands,
            m.dict_entries,
            m.long_entries,
            m.spelling_bytes as f64 / 1024.0,
            m.wide_edges,
            m.wide_arena,
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <wikidata.bz2> [--repeats N] [--suite bidi|fwd|all]",
            args[0]
        );
        std::process::exit(1);
    }
    let path = &args[1];
    let mut repeats = 7usize;
    let mut suite = "all";
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--repeats" && i + 1 < args.len() {
            repeats = args[i + 1].parse().expect("invalid repeats");
            i += 2;
        } else if args[i] == "--suite" && i + 1 < args.len() {
            suite = match args[i + 1].as_str() {
                "bidi" => "bidi",
                "fwd" => "fwd",
                "all" => "all",
                other => {
                    eprintln!("Unknown suite: {other}");
                    std::process::exit(1);
                }
            };
            i += 2;
        } else {
            eprintln!("Unknown arg: {}", args[i]);
            std::process::exit(1);
        }
    }

    let all_blocks = load_all_blocks(path);
    let admitted_blocks: Vec<&BlockInput> = all_blocks.iter().filter(|b| b.admitted).collect();
    let total_all_rows: usize = all_blocks.iter().map(|b| b.n).sum();
    let total_adm_rows: usize = admitted_blocks.iter().map(|b| b.n).sum();
    println!(
        "Loaded {}: {} total blocks ({} rows); {} production-admitted Open blocks ({} rows)",
        path,
        all_blocks.len(),
        total_all_rows,
        admitted_blocks.len(),
        total_adm_rows
    );

    let bidi_arms = [
        A6bArm::ShippedBaseline,
        A6bArm::BaselineNoStats,
        A6bArm::SetupU16Fused,
        A6bArm::Inline15SoaZeroCache,
        A6bArm::CombinedSoaGen,
        A6bArm::CombinedNoBcSoaGen,
        A6bArm::CombinedNoBcSoaNoDedup,
        A6bArm::CombinedNoBcAos32Gen,
        A6bArm::CombinedNoBcAos24I7Gen,
    ];

    if suite == "all" || suite == "bidi" {
        run_suite(
            "SHIPPED BIDI (C+E)",
            &bidi_arms,
            false,
            &all_blocks,
            &admitted_blocks,
            repeats,
        );
    }

    let fwd_arms = [
        A6bArm::ShippedBaseline,
        A6bArm::CombinedSoaGen,
        A6bArm::CombinedNoBcSoaGen,
        A6bArm::CombinedNoBcSoaNoDedup,
        A6bArm::CombinedNoBcAos32Gen,
        A6bArm::CombinedNoBcAos24I7Gen,
    ];

    if suite == "all" || suite == "fwd" {
        run_suite(
            "MATCHED FORWARD EMIT (C+E)",
            &fwd_arms,
            true,
            &all_blocks,
            &admitted_blocks,
            repeats,
        );
    }
}
