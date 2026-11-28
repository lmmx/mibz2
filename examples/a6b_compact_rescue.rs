#![forbid(unsafe_code)]
//! Forensic L3/RSS Dissection & Compact-Footprint Rescue of `A6b` (`a6b-compact-l3-rescue`).
//!
//! 100% safe Rust (`#![forbid(unsafe_code)]`), zero `src/` edits.
//! Investigates `#1053`'s physical i9-10900K `STOP/KILL` (`origin/evidence` `a6e427b1`:
//! `wikidata/1` `-1.76%` / `+24.7 ms`, `wikidata/8/S` `worker_busy = 1.0106`, RSS `+524 KiB/w`
//! at 8W / `+546 KiB/w` at 10W) and benchmarks 6 compact-footprint safe-Rust rescue mechanisms
//! against shipped `S0_default` and `#1053` `S1_1053`.
//!
//! Usage:
//!   a6b_compact_rescue audit <wikidata.bz2>
//!   a6b_compact_rescue rss <wikidata.bz2>
//!   a6b_compact_rescue verify <wikidata.bz2> [guard_dir]
//!   a6b_compact_rescue bench <wikidata.bz2> [--repeats 7] [--guards <guard_dir>]

use mibz2::block::Symbols;
use mibz2::bwt::{build, walk, TVector};
use mibz2::runs::Runs;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, runs, Kernels};
use std::hint::black_box;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

// ---------------------------------------------------------------------------
// On-CPU thread clock (/proc/thread-self/schedstat) & /proc/self/status RSS
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

fn read_proc_rss_kib() -> (u64, u64) {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let mut rss = 0u64;
    let mut hwm = 0u64;
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            rss = rest
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("VmHWM:") {
            hwm = rest
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }
    }
    (rss, hwm)
}

// ---------------------------------------------------------------------------
// Constants matching src/open.rs & src/open/endpoint.rs
// ---------------------------------------------------------------------------

const PROD_LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const CERTIFIED_WIDTH: u32 = 32_768;
const RULE: usize = 4;
const ID_BITS: u32 = 22;
const ID_MASK: u32 = (1 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;
const SPIRAL_RULES: usize = 2 * 20 + 1;

const LAZY_DENSITY: (usize, usize) = (3, 20);
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

const CHUNK: usize = 512;
const CHUNK_BYTES: usize = 2 * CHUNK;
const NONE_CHUNK: u32 = u32::MAX;
const EPOCH: u32 = 1 << 16;

const U128_PAYLOAD_MASK: u128 = (1u128 << 120) - 1;
const U128_LONG_TAG: u128 = 0xFFu128 << 120;

const U64_PAYLOAD_MASK: u64 = (1u64 << 56) - 1;
const U64_ID_MASK: u64 = (1u64 << 24) - 1;
const U64_LONG_TAG: u64 = 0xFFu64 << 56;

const GEN_PAIR_SLOTS: usize = 1024;
const GEN_CONTENT_SLOTS: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    S0Default,
    S1Redesign1053,
    C1VecSoaV2,
    C2TieredSlabV2,
    C3CompactWord8Soa,
    C4FusedSetupOnShipped,
    C5EdgesOverlaySoaU64,
    C6EdgesOverlaySoaU128,
}

const ALL_ARMS: [Arm; 8] = [
    Arm::S0Default,
    Arm::S1Redesign1053,
    Arm::C1VecSoaV2,
    Arm::C2TieredSlabV2,
    Arm::C3CompactWord8Soa,
    Arm::C4FusedSetupOnShipped,
    Arm::C5EdgesOverlaySoaU64,
    Arm::C6EdgesOverlaySoaU128,
];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::S0Default => "S0_default",
            Arm::S1Redesign1053 => "S1_1053",
            Arm::C1VecSoaV2 => "C1_vec_soa_v2",
            Arm::C2TieredSlabV2 => "C2_tiered_slab_v2",
            Arm::C3CompactWord8Soa => "C3_compact_word8_soa",
            Arm::C4FusedSetupOnShipped => "C4_fused_setup_on_shipped",
            Arm::C5EdgesOverlaySoaU64 => "C5_edges_overlay_soa_u64",
            Arm::C6EdgesOverlaySoaU128 => "C6_edges_overlay_soa_u128",
        }
    }

    fn from_name(s: &str) -> Option<Self> {
        ALL_ARMS.iter().copied().find(|a| a.name() == s)
    }
}

// ---------------------------------------------------------------------------
// Open helpers (Band, Strip, OutQ16, bidi emit, admission)
// ---------------------------------------------------------------------------

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

fn open_cap(records: usize, n: usize, p: u32) -> Option<Cap> {
    if n == 0 || n > 900_000 || p as usize >= n {
        None
    } else if records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n {
        Some(Cap::NONE)
    } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
        Some(Cap::dense(n))
    } else {
        None
    }
}

fn within_cap(records: &[u16], p: u32, limit: u32, cap: Cap) -> bool {
    let mut m = 0u32;
    let mut strips = 0usize;
    maximal(records, |_, pos, width| {
        split(0, pos, width, p, limit, |_, _, w, keep| {
            if keep {
                m += w;
            } else {
                strips += 1;
            }
        });
    });
    m <= cap.m && strips <= cap.strips
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

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

impl Strip {
    #[inline(always)]
    fn pack(self, dst: &mut [u32]) {
        dst[0] = self.from;
        dst[1] = self.to;
        dst[2] = (self.width << 8) | u32::from(self.byte);
    }
    #[inline(always)]
    fn unpack(src: &[u32]) -> Self {
        Strip {
            from: src[0],
            to: src[1],
            width: src[2] >> 8,
            byte: (src[2] & 0xFF) as u8,
        }
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

#[inline(always)]
fn literal_shipped(byte: u8) -> u32 {
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

// ---------------------------------------------------------------------------
// OutQ16 (A6a + A6c) & Shipped Bidirectional A7 Emit
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Spill {
    next: Vec<u32>,
    ovf: Vec<[u8; CHUNK_BYTES]>,
    cuts: Vec<[u32; 256]>,
}

impl Spill {
    fn capacity_bytes(&self) -> usize {
        self.next.capacity() * 4
            + self.ovf.capacity() * CHUNK_BYTES
            + self.cuts.capacity() * 256 * 4
    }
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

    #[inline(always)]
    fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
        if id < self.in_out {
            let at = id as usize * CHUNK_BYTES;
            &mut self.out[at..at + CHUNK_BYTES]
        } else {
            &mut self.spill.ovf[(id - self.in_out) as usize]
        }
    }
}

struct QReader<'a> {
    out: &'a [u8],
    spill: &'a Spill,
    in_out: u32,
    chunk: u32,
    at: usize,
    a: usize,
    seen: u32,
    epoch: u32,
    next_cut: usize,
    cut: u32,
}

impl<'a> QReader<'a> {
    fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
        let cut = spill.cuts.first().map_or(u32::MAX, |c| c[a]);
        QReader {
            out,
            spill,
            in_out: heads.in_out,
            chunk: heads.heads[a],
            at: 0,
            a,
            seen: 0,
            epoch: 0,
            next_cut: 0,
            cut,
        }
    }

    #[inline(always)]
    fn chunk_bytes(&self) -> &'a [u8] {
        if self.chunk < self.in_out {
            let at = self.chunk as usize * CHUNK_BYTES;
            &self.out[at..at + CHUNK_BYTES]
        } else {
            &self.spill.ovf[(self.chunk - self.in_out) as usize]
        }
    }

    #[inline(always)]
    fn advance_epoch(&mut self) {
        while self.seen == self.cut {
            self.epoch += EPOCH;
            self.next_cut += 1;
            self.cut = self
                .spill
                .cuts
                .get(self.next_cut)
                .map_or(u32::MAX, |c| c[self.a]);
        }
    }

    #[inline(always)]
    fn wire_xor(
        &mut self,
        edges: &mut [u32],
        mut dst: usize,
        mut len: usize,
        pack: u32,
        mark: u32,
        predecessor: &mut u32,
    ) {
        while len > 0 {
            self.advance_epoch();
            let take = len
                .min(CHUNK - self.at)
                .min((self.cut - self.seen) as usize);
            let bytes = self.chunk_bytes();
            let src = &bytes[2 * self.at..2 * (self.at + take)];
            for (k, pair) in src.chunks_exact(2).enumerate() {
                let d = dst + k;
                let j_row = self.epoch | u32::from(u16::from_le_bytes([pair[0], pair[1]]));
                edges[d] ^= pack | j_row;
                edges[j_row as usize] ^= d as u32;
                if j_row == mark {
                    *predecessor = d as u32;
                }
            }
            self.at += take;
            if self.at == CHUNK {
                self.chunk = self.spill.next[self.chunk as usize];
                self.at = 0;
            }
            dst += take;
            self.seen += take as u32;
            len -= take;
        }
    }

    #[inline(always)]
    fn wire(&mut self, mut dst: &mut [u32], pack: u32) {
        while !dst.is_empty() {
            self.advance_epoch();
            let take = dst
                .len()
                .min(CHUNK - self.at)
                .min((self.cut - self.seen) as usize);
            let bytes = self.chunk_bytes();
            let base = pack | self.epoch;
            let (head, tail) = dst.split_at_mut(take);
            for (e, pair) in head
                .iter_mut()
                .zip(bytes[2 * self.at..2 * (self.at + take)].chunks_exact(2))
            {
                *e = base | u32::from(u16::from_le_bytes([pair[0], pair[1]]));
            }
            self.at += take;
            if self.at == CHUNK {
                self.chunk = self.spill.next[self.chunk as usize];
                self.at = 0;
            }
            self.seen += take as u32;
            dst = tail;
        }
    }
}

fn repeat_periodic(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_bidi_or_fwd(
    predecessor: Option<u32>,
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
    if let Some(before) = predecessor {
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let (mut lo, mut hi) = (0usize, n);
        for pair in 0..=edges.len() / 2 {
            if lo == hi || (pair != 0 && front == next) {
                break;
            }
            let ef = edges[front as usize];
            let cf = (ef >> bits) as usize;
            let single = front == back;
            let lf = lens[cf];
            if lf != LONG && hi - lo >= INLINE {
                out[lo..lo + INLINE].copy_from_slice(&inline16[cf]);
                lo += usize::from(lf);
            } else {
                let word = if lf == LONG {
                    long_word(&inline16[cf], spelling)
                } else {
                    &inline16[cf][..usize::from(lf)]
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
                let shifted = u128::from_le_bytes(inline16[cb]) << (8 * (INLINE - usize::from(lb)));
                out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                hi -= usize::from(lb);
            } else {
                let word = if lb == LONG {
                    long_word(&inline16[cb], spelling)
                } else {
                    &inline16[cb][..usize::from(lb)]
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
            repeat_periodic(n, out);
        } else {
            out.truncate(n);
        }
    } else {
        let mut i = mark as usize;
        let mut at = 0usize;
        while at < n {
            let e = edges[i];
            let code = if wide {
                extra[i] as usize
            } else {
                (e >> bits) as usize
            };
            i = (e & mask) as usize;
            let len = lens[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                let w = long_word(&inline16[code], spelling);
                out[at..at + w.len()].copy_from_slice(w);
                at += w.len();
            }
            if i == mark as usize && at < n {
                out.truncate(at);
                repeat_periodic(n, out);
                return;
            }
        }
        out.truncate(n);
    }
}

// ---------------------------------------------------------------------------
// Shared A6b types & hashing helpers
// ---------------------------------------------------------------------------

#[derive(Default, Clone, Copy)]
struct NodeLinks {
    prev0: u16,
    next0: u16,
    prev1: u16,
    next1: u16,
}

#[derive(Default, Clone, Copy)]
struct PhaseNs {
    setup: u64,
    steps: u64,
    bot: u64,
    dict: u64,
}

impl PhaseNs {
    fn add(&mut self, o: PhaseNs) {
        self.setup += o.setup;
        self.steps += o.steps;
        self.bot += o.bot;
        self.dict += o.dict;
    }
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
fn short_u64_hash(low: u64, len: u8) -> u64 {
    let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
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

// ---------------------------------------------------------------------------
// Generation caches: 1053 (pairs+contents) vs V2 (gen8+fp8 contents_4k only)
// ---------------------------------------------------------------------------

struct GenCache1053 {
    pairs: Vec<u64>,
    contents: Vec<u32>,
    pair_gen: u64,
    content_gen: u32,
}

impl Default for GenCache1053 {
    fn default() -> Self {
        GenCache1053 {
            pairs: vec![0; GEN_PAIR_SLOTS],
            contents: vec![0; GEN_CONTENT_SLOTS],
            pair_gen: 0,
            content_gen: 0,
        }
    }
}

impl GenCache1053 {
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

    fn capacity_bytes(&self) -> usize {
        self.pairs.capacity() * 8 + self.contents.capacity() * 4
    }
}

/// V2 compact generation-tagged content cache: 4096 x u32 = 16 KiB ONLY (no pairs).
/// Packs `(gen8 << 24) | (fp8 << 16) | code16`.
struct GenCacheV2 {
    contents: Box<[u32; GEN_CONTENT_SLOTS]>,
    gen8: u32,
}

impl Default for GenCacheV2 {
    fn default() -> Self {
        GenCacheV2 {
            contents: vec![0u32; GEN_CONTENT_SLOTS]
                .into_boxed_slice()
                .try_into()
                .ok()
                .unwrap(),
            gen8: 0,
        }
    }
}

impl GenCacheV2 {
    #[inline(always)]
    fn next_block(&mut self) {
        self.gen8 += 1;
        if self.gen8 >= 256 {
            self.contents.fill(0);
            self.gen8 = 1;
        }
    }

    fn capacity_bytes(&self) -> usize {
        GEN_CONTENT_SLOTS * 4
    }
}

// ---------------------------------------------------------------------------
// Fixed64kSoa (#1053) and TieredSlab (C2 / C3)
// ---------------------------------------------------------------------------

fn boxed_array<T: Copy, const N: usize>(val: T) -> Box<[T; N]> {
    vec![val; N].into_boxed_slice().try_into().ok().unwrap()
}

struct Fixed64kSoa {
    links: Box<[NodeLinks; NARROW_NODES]>,
    widths: Box<[u32; NARROW_NODES]>,
    words128: Box<[u128; NARROW_NODES]>,
}

impl Fixed64kSoa {
    fn new() -> Self {
        Fixed64kSoa {
            links: boxed_array(NodeLinks::default()),
            widths: boxed_array(0),
            words128: boxed_array(0),
        }
    }
    fn virtual_bytes(&self) -> usize {
        NARROW_NODES * (std::mem::size_of::<NodeLinks>() + 4 + 16)
    }
}

#[derive(Default)]
enum TieredSoaU128 {
    #[default]
    None,
    K8 {
        links: Box<[NodeLinks; 8192]>,
        widths: Box<[u32; 8192]>,
        words: Box<[u128; 8192]>,
    },
    K16 {
        links: Box<[NodeLinks; 16384]>,
        widths: Box<[u32; 16384]>,
        words: Box<[u128; 16384]>,
    },
    K32 {
        links: Box<[NodeLinks; 32768]>,
        widths: Box<[u32; 32768]>,
        words: Box<[u128; 32768]>,
    },
    K64 {
        links: Box<[NodeLinks; 65536]>,
        widths: Box<[u32; 65536]>,
        words: Box<[u128; 65536]>,
    },
}

impl TieredSoaU128 {
    fn cap(&self) -> usize {
        match self {
            TieredSoaU128::None => 0,
            TieredSoaU128::K8 { .. } => 8192,
            TieredSoaU128::K16 { .. } => 16384,
            TieredSoaU128::K32 { .. } => 32768,
            TieredSoaU128::K64 { .. } => 65536,
        }
    }
    fn ensure(&mut self, need: usize) {
        if self.cap() >= need {
            return;
        }
        *self = if need <= 8192 {
            TieredSoaU128::K8 {
                links: boxed_array(NodeLinks::default()),
                widths: boxed_array(0),
                words: boxed_array(0),
            }
        } else if need <= 16384 {
            TieredSoaU128::K16 {
                links: boxed_array(NodeLinks::default()),
                widths: boxed_array(0),
                words: boxed_array(0),
            }
        } else if need <= 32768 {
            TieredSoaU128::K32 {
                links: boxed_array(NodeLinks::default()),
                widths: boxed_array(0),
                words: boxed_array(0),
            }
        } else {
            TieredSoaU128::K64 {
                links: boxed_array(NodeLinks::default()),
                widths: boxed_array(0),
                words: boxed_array(0),
            }
        };
    }
    fn capacity_bytes(&self) -> usize {
        self.cap() * (std::mem::size_of::<NodeLinks>() + 4 + 16)
    }
}

#[derive(Default)]
enum TieredSoaU64 {
    #[default]
    None,
    K8 {
        links: Box<[NodeLinks; 8192]>,
        widths: Box<[u32; 8192]>,
        words: Box<[u64; 8192]>,
    },
    K16 {
        links: Box<[NodeLinks; 16384]>,
        widths: Box<[u32; 16384]>,
        words: Box<[u64; 16384]>,
    },
    K32 {
        links: Box<[NodeLinks; 32768]>,
        widths: Box<[u32; 32768]>,
        words: Box<[u64; 32768]>,
    },
    K64 {
        links: Box<[NodeLinks; 65536]>,
        widths: Box<[u32; 65536]>,
        words: Box<[u64; 65536]>,
    },
}

impl TieredSoaU64 {
    fn cap(&self) -> usize {
        match self {
            TieredSoaU64::None => 0,
            TieredSoaU64::K8 { .. } => 8192,
            TieredSoaU64::K16 { .. } => 16384,
            TieredSoaU64::K32 { .. } => 32768,
            TieredSoaU64::K64 { .. } => 65536,
        }
    }
    fn ensure(&mut self, need: usize) {
        if self.cap() >= need {
            return;
        }
        *self = if need <= 8192 {
            TieredSoaU64::K8 {
                links: boxed_array(NodeLinks::default()),
                widths: boxed_array(0),
                words: boxed_array(0),
            }
        } else if need <= 16384 {
            TieredSoaU64::K16 {
                links: boxed_array(NodeLinks::default()),
                widths: boxed_array(0),
                words: boxed_array(0),
            }
        } else if need <= 32768 {
            TieredSoaU64::K32 {
                links: boxed_array(NodeLinks::default()),
                widths: boxed_array(0),
                words: boxed_array(0),
            }
        } else {
            TieredSoaU64::K64 {
                links: boxed_array(NodeLinks::default()),
                widths: boxed_array(0),
                words: boxed_array(0),
            }
        };
    }
    fn capacity_bytes(&self) -> usize {
        self.cap() * (std::mem::size_of::<NodeLinks>() + 4 + 8)
    }
}

// ---------------------------------------------------------------------------
// Per-arm Endpoint Scratch (strictly separate per arm so RSS is clean!)
// ---------------------------------------------------------------------------

enum EndpointScratch {
    S0Default {
        stack: Vec<u32>,
    },
    S1Redesign1053 {
        stack: Vec<u32>,
        image_order64: Vec<u64>,
        fixed_soa: Option<Fixed64kSoa>,
        rules: Vec<[u32; 4]>,
        gen_cache: GenCache1053,
    },
    C1VecSoaV2 {
        stack: Vec<u32>,
        image_order64: Vec<u64>,
        links: Vec<NodeLinks>,
        widths: Vec<u32>,
        words128: Vec<u128>,
        rules: Vec<[u32; 4]>,
        gen_cache: GenCacheV2,
    },
    C2TieredSlabV2 {
        stack: Vec<u32>,
        image_order64: Vec<u64>,
        slab: TieredSoaU128,
        rules: Vec<[u32; 4]>,
        gen_cache: GenCacheV2,
    },
    C3CompactWord8Soa {
        stack: Vec<u32>,
        image_order64: Vec<u64>,
        slab: TieredSoaU64,
        rules: Vec<[u32; 4]>,
        gen_cache: GenCacheV2,
    },
    C4FusedSetupOnShipped {
        stack: Vec<u32>,
        gen_cache: GenCacheV2,
    },
    C5EdgesOverlaySoaU64 {
        stack: Vec<u32>,
        gen_cache: GenCacheV2,
    },
    C6EdgesOverlaySoaU128 {
        stack: Vec<u32>,
        gen_cache: GenCacheV2,
    },
}

impl EndpointScratch {
    fn new(arm: Arm) -> Self {
        match arm {
            Arm::S0Default => EndpointScratch::S0Default { stack: Vec::new() },
            Arm::S1Redesign1053 => EndpointScratch::S1Redesign1053 {
                stack: Vec::new(),
                image_order64: Vec::new(),
                fixed_soa: None,
                rules: Vec::new(),
                gen_cache: GenCache1053::default(),
            },
            Arm::C1VecSoaV2 => EndpointScratch::C1VecSoaV2 {
                stack: Vec::new(),
                image_order64: Vec::new(),
                links: Vec::new(),
                widths: Vec::new(),
                words128: Vec::new(),
                rules: Vec::new(),
                gen_cache: GenCacheV2::default(),
            },
            Arm::C2TieredSlabV2 => EndpointScratch::C2TieredSlabV2 {
                stack: Vec::new(),
                image_order64: Vec::new(),
                slab: TieredSoaU128::default(),
                rules: Vec::new(),
                gen_cache: GenCacheV2::default(),
            },
            Arm::C3CompactWord8Soa => EndpointScratch::C3CompactWord8Soa {
                stack: Vec::new(),
                image_order64: Vec::new(),
                slab: TieredSoaU64::default(),
                rules: Vec::new(),
                gen_cache: GenCacheV2::default(),
            },
            Arm::C4FusedSetupOnShipped => EndpointScratch::C4FusedSetupOnShipped {
                stack: Vec::new(),
                gen_cache: GenCacheV2::default(),
            },
            Arm::C5EdgesOverlaySoaU64 => EndpointScratch::C5EdgesOverlaySoaU64 {
                stack: Vec::new(),
                gen_cache: GenCacheV2::default(),
            },
            Arm::C6EdgesOverlaySoaU128 => EndpointScratch::C6EdgesOverlaySoaU128 {
                stack: Vec::new(),
                gen_cache: GenCacheV2::default(),
            },
        }
    }

    fn arm(&self) -> Arm {
        match self {
            EndpointScratch::S0Default { .. } => Arm::S0Default,
            EndpointScratch::S1Redesign1053 { .. } => Arm::S1Redesign1053,
            EndpointScratch::C1VecSoaV2 { .. } => Arm::C1VecSoaV2,
            EndpointScratch::C2TieredSlabV2 { .. } => Arm::C2TieredSlabV2,
            EndpointScratch::C3CompactWord8Soa { .. } => Arm::C3CompactWord8Soa,
            EndpointScratch::C4FusedSetupOnShipped { .. } => Arm::C4FusedSetupOnShipped,
            EndpointScratch::C5EdgesOverlaySoaU64 { .. } => Arm::C5EdgesOverlaySoaU64,
            EndpointScratch::C6EdgesOverlaySoaU128 { .. } => Arm::C6EdgesOverlaySoaU128,
        }
    }

    fn capacity_bytes(&self) -> usize {
        match self {
            EndpointScratch::S0Default { stack } => stack.capacity() * 4,
            EndpointScratch::S1Redesign1053 {
                stack,
                image_order64,
                fixed_soa,
                rules,
                gen_cache,
            } => {
                stack.capacity() * 4
                    + image_order64.capacity() * 8
                    + fixed_soa.as_ref().map_or(0, Fixed64kSoa::virtual_bytes)
                    + rules.capacity() * 16
                    + gen_cache.capacity_bytes()
            }
            EndpointScratch::C1VecSoaV2 {
                stack,
                image_order64,
                links,
                widths,
                words128,
                rules,
                gen_cache,
            } => {
                stack.capacity() * 4
                    + image_order64.capacity() * 8
                    + links.capacity() * 8
                    + widths.capacity() * 4
                    + words128.capacity() * 16
                    + rules.capacity() * 16
                    + gen_cache.capacity_bytes()
            }
            EndpointScratch::C2TieredSlabV2 {
                stack,
                image_order64,
                slab,
                rules,
                gen_cache,
            } => {
                stack.capacity() * 4
                    + image_order64.capacity() * 8
                    + slab.capacity_bytes()
                    + rules.capacity() * 16
                    + gen_cache.capacity_bytes()
            }
            EndpointScratch::C3CompactWord8Soa {
                stack,
                image_order64,
                slab,
                rules,
                gen_cache,
            } => {
                stack.capacity() * 4
                    + image_order64.capacity() * 8
                    + slab.capacity_bytes()
                    + rules.capacity() * 16
                    + gen_cache.capacity_bytes()
            }
            EndpointScratch::C4FusedSetupOnShipped { stack, gen_cache }
            | EndpointScratch::C5EdgesOverlaySoaU64 { stack, gen_cache }
            | EndpointScratch::C6EdgesOverlaySoaU128 { stack, gen_cache } => {
                stack.capacity() * 4 + gen_cache.capacity_bytes()
            }
        }
    }

    fn force_gen_near_wrap(&mut self) {
        match self {
            EndpointScratch::S0Default { .. } => {}
            EndpointScratch::S1Redesign1053 { gen_cache, .. } => {
                gen_cache.content_gen = (1u32 << 16) - 2;
                gen_cache.pair_gen = (1u64 << 18) - 2;
            }
            EndpointScratch::C1VecSoaV2 { gen_cache, .. }
            | EndpointScratch::C2TieredSlabV2 { gen_cache, .. }
            | EndpointScratch::C3CompactWord8Soa { gen_cache, .. }
            | EndpointScratch::C4FusedSetupOnShipped { gen_cache, .. }
            | EndpointScratch::C5EdgesOverlaySoaU64 { gen_cache, .. }
            | EndpointScratch::C6EdgesOverlaySoaU128 { gen_cache, .. } => {
                gen_cache.gen8 = 254;
            }
        }
    }
}

struct WorkerState {
    runs: Runs,
    spill: Spill,
    edges: Vec<u32>,
    extra: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    out: Vec<u8>,
    rle_out: Vec<u8>,
    endpoint: EndpointScratch,
}

impl WorkerState {
    fn new(arm: Arm) -> Self {
        WorkerState {
            runs: Runs::default(),
            spill: Spill::default(),
            edges: Vec::new(),
            extra: Vec::new(),
            bands: Vec::new(),
            spelling: Vec::new(),
            inline16: Vec::new(),
            lens: Vec::new(),
            out: Vec::new(),
            rle_out: Vec::new(),
            endpoint: EndpointScratch::new(arm),
        }
    }

    fn kernel_capacity_bytes(&self) -> usize {
        self.edges.capacity() * 4
            + self.extra.capacity() * 4
            + self.bands.capacity() * 8
            + self.spelling.capacity()
            + self.inline16.capacity() * INLINE
            + self.lens.capacity()
            + self.spill.capacity_bytes()
            + self.out.capacity()
            + self.rle_out.capacity()
    }
}

// ---------------------------------------------------------------------------
// Arm S0: Exact Shipped src/open/endpoint.rs (Narrow / Wide over edges)
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
fn grow_u32(a: &mut Vec<u32>, need: usize) {
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

    fn eliminate<const SPIRALS: bool, const DIRECT_EQ: bool>(
        &mut self,
        remaining: &mut u32,
        steps: &mut usize,
    ) -> bool {
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
            if DIRECT_EQ {
                if aw > bw {
                    self.pop_tail(y, 1);
                    self.insert(x, y, 1);
                    self.set_word(y, word);
                    self.set_width(x, aw - bw);
                } else if aw < bw {
                    self.pop_tail(x, 0);
                    self.insert(y, x, 0);
                    self.set_word(x, word);
                    self.set_width(y, bw - aw);
                } else {
                    self.pop_tail(x, 0);
                    let yp0 = L::prev(self.a, y, 0);
                    let yn0 = L::next(self.a, y, 0);
                    L::set_prev(self.a, x, 0, yp0);
                    L::set_next(self.a, x, 0, yn0);
                    L::set_next(self.a, yp0, 0, x);
                    L::set_prev(self.a, yn0, 0, x);
                    self.pop_tail(y, 1);
                    self.set_word(x, word);
                }
            } else if aw <= bw {
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

    /// Left-spine DFS with stack [u32; 64] fast path (for C4).
    fn append_fast(&mut self, word: u32, spill: &mut Vec<u32>, dest: &mut Vec<u8>) {
        let mut stk = [0u32; 64];
        let mut sp = 0usize;
        spill.clear();
        let mut cur = word;
        loop {
            if cur <= 256 {
                dest.push((cur - 1) as u8);
            } else {
                let r = self.rule(cur);
                let tag = self.a[r + 3];
                if tag >> 28 == 0 {
                    let bytes = self.short_payload(cur).to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                } else if tag == u32::MAX {
                    let r0 = self.a[r];
                    let r1 = self.a[r + 1];
                    self.a[r + 3] = 0x8000_0000 | dest.len() as u32;
                    if sp < 64 {
                        stk[sp] = r1;
                        sp += 1;
                    } else {
                        spill.push(r1);
                    }
                    cur = r0;
                    continue;
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let len = self.a[r + 2] as usize;
                    dest.extend_from_within(start..start + len);
                }
            }
            if let Some(next) = spill.pop() {
                cur = next;
            } else if sp > 0 {
                sp -= 1;
                cur = stk[sp];
            } else {
                break;
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

#[allow(clippy::too_many_arguments)]
fn induced_shipped<L: Links>(
    edges: &mut Vec<u32>,
    bands: &mut Vec<Band>,
    spelling: &mut Vec<u8>,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    ns: usize,
    n: u32,
    m: u32,
    thin: &[u32; 256],
    force_spirals: Option<bool>,
    ph: Option<&mut PhaseNs>,
) -> ([usize; 256], usize, usize) {
    let t0 = ph.as_ref().map(|_| Instant::now());
    let most = 3 * ns + 3;
    let nodes_words = L::STRIDE * most;
    let need = (4 * ns + (L::STRIDE + RULE) * most).max(m as usize);
    if edges.len() < need {
        edges.resize(need, 0);
    }
    let (hdr, arena) = edges.split_at_mut(4 * ns);
    let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
    let mut e = Elim::<L> {
        a: arena,
        base: 0,
        top: 0,
        nodes: 0,
        _links: std::marker::PhantomData,
    };
    e.node(0, 0);
    let mut widest = 0;
    for i in 0..ns {
        let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
        e.node(s.width, literal_shipped(s.byte));
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
    let n_nodes = e.nodes as usize;
    for side in 0..2 {
        e.close(side, terminals[side], order[side]);
    }
    e.base = L::STRIDE * n_nodes;
    e.top = e.base;
    let _ = nodes_words;
    let t1 = t0.map(|_| Instant::now());

    let (mut remaining, mut steps) = (n, 0);
    let spirals = force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
    while !if spirals {
        e.eliminate::<true, false>(&mut remaining, &mut steps)
    } else {
        e.eliminate::<false, false>(&mut remaining, &mut steps)
    } {
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        grow_u32(edges, 4 * ns + top + RULE);
        e = Elim {
            a: &mut edges[4 * ns..],
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
    }
    let n_rules = (e.top - e.base) / RULE;
    let t2 = t0.map(|_| Instant::now());

    e.assign_bottom(m);
    let t3 = t0.map(|_| Instant::now());

    const PAIR_WORDS: usize = 2 * 1024;
    const CACHE_WORDS: usize = PAIR_WORDS + 4096;
    let (base, top, nodes) = (e.base, e.top, e.nodes);
    let (header, arena) = edges.split_at_mut(4 * ns);
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
    begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);

    let mut cursors = [0; 256];
    let mut id = L::next(e.a, 0, 0);
    let mut in_node = 0;
    let mut k = 0;
    for a in 0..256 {
        cursors[a] = bands.len();
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
                let wid = node_word & ID_MASK;
                let key = (wid << 8) | (a as u32);
                let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
                if !pairs.is_empty() && pairs[slot] == key {
                    pairs[slot + 1]
                } else {
                    let start = spelling.len();
                    let memoized =
                        node_word >> ID_BITS >= INLINE as u32 && e.a[e.rule(wid) + 3] != u32::MAX;
                    let (entry, len) = e.inline_word(node_word, a as u8, stack, spelling);
                    if contents.is_empty() {
                        let code = lens.len() as u32;
                        inline16.push(entry);
                        lens.push(len);
                        code
                    } else {
                        let bytes = if len == LONG {
                            &spelling[start..]
                        } else {
                            &entry[..len as usize]
                        };
                        let index =
                            spelling_hash(&entry, len, bytes) as usize & (contents.len() - 1);
                        let old = contents[index] as usize;
                        let same = old != 0
                            && lens[old] == len
                            && if len == LONG {
                                long_word(&inline16[old], spelling) == bytes
                            } else {
                                inline16[old] == entry
                            };
                        let code = if same {
                            if len == LONG {
                                if memoized {
                                    spelling.truncate(start);
                                } else {
                                    spelling.pop();
                                }
                            }
                            old as u32
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(entry);
                            lens.push(len);
                            contents[index] = code;
                            code
                        };
                        pairs[slot] = key;
                        pairs[slot + 1] = code;
                        code
                    }
                }
            };
            bands.push(Band::new(k + width, dst, code));
            in_node += width;
            k += width;
            if in_node == node_width {
                id = L::next(e.a, id, 0);
                in_node = 0;
            }
        }
    }
    assert_eq!(id, 0);
    if let Some(p) = ph {
        let t4 = Instant::now();
        p.setup += t1.unwrap().duration_since(t0.unwrap()).as_nanos() as u64;
        p.steps += t2.unwrap().duration_since(t1.unwrap()).as_nanos() as u64;
        p.bot += t3.unwrap().duration_since(t2.unwrap()).as_nanos() as u64;
        p.dict += t4.duration_since(t3.unwrap()).as_nanos() as u64;
    }
    (cursors, n_nodes - 1, n_rules)
}

// ---------------------------------------------------------------------------
// Arm C4: Fused setup + image_order64 in edges + direct aw==bw + GenCacheV2
//         on top of shipped edges-overlaid Narrow arena (0 extra node/rule RAM)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn induced_c4_fused_on_shipped(
    edges: &mut Vec<u32>,
    bands: &mut Vec<Band>,
    spelling: &mut Vec<u8>,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    gen_cache: &mut GenCacheV2,
    ns: usize,
    n: u32,
    m: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    thin: &[u32; 256],
    force_spirals: Option<bool>,
    ph: Option<&mut PhaseNs>,
) -> ([usize; 256], usize, usize) {
    let t0 = ph.as_ref().map(|_| Instant::now());
    let most = 3 * ns + 3;
    let hdr_words = 5 * ns; // 3*ns strips_raw + 2*ns packed image_order64
    let need = (hdr_words + (Narrow::STRIDE + RULE) * most).max(m as usize);
    if edges.len() < need {
        edges.resize(need, 0);
    }
    let (hdr, arena) = edges.split_at_mut(hdr_words);
    let (strips_raw, img_u32) = hdr.split_at_mut(3 * ns);
    let mut e = Elim::<Narrow> {
        a: arena,
        base: 0,
        top: 0,
        nodes: (1 + ns) as u32,
        _links: std::marker::PhantomData,
    };
    e.a[0] = 0;
    e.a[1] = 0;
    e.set_width(0, 0);
    e.set_word(0, 0);

    let mut widest = 0u32;
    let mut ord0_tail = 0u32;
    let (mut term1_head, mut term1_tail) = (0u32, 0u32);
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
        let sid = (i + 1) as u32;

        e.set_width(sid, width);
        e.set_word(sid, literal_shipped(byte));
        widest = widest.max(width);

        let pos = write[a] as usize;
        write[a] = (pos + 1) as u32;
        let packed = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);
        img_u32[2 * pos] = packed as u32;
        img_u32[2 * pos + 1] = (packed >> 32) as u32;

        if x < from {
            let gap = from - x;
            widest = widest.max(gap);
            let gid = e.node(gap, 0);
            Narrow::set_next(e.a, ord0_tail, 0, gid);
            Narrow::set_prev(e.a, gid, 0, ord0_tail);
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                Narrow::set_next(e.a, term1_tail, 1, gid);
            }
            Narrow::set_prev(e.a, gid, 1, term1_tail);
            term1_tail = gid;
        }
        Narrow::set_next(e.a, ord0_tail, 0, sid);
        Narrow::set_prev(e.a, sid, 0, ord0_tail);
        ord0_tail = sid;
        x = from + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = e.node(gap, 0);
        Narrow::set_next(e.a, ord0_tail, 0, gid);
        Narrow::set_prev(e.a, gid, 0, ord0_tail);
        ord0_tail = gid;
        if term1_tail == 0 {
            term1_head = gid;
        } else {
            Narrow::set_next(e.a, term1_tail, 1, gid);
        }
        Narrow::set_prev(e.a, gid, 1, term1_tail);
        term1_tail = gid;
    }
    let ord0_head = if ord0_tail == 0 {
        0
    } else {
        Narrow::next(e.a, 0, 0)
    };

    let mut ord1_tail = 0u32;
    let (mut term0_head, mut term0_tail) = (0u32, 0u32);
    x = 0;
    for pos in 0..ns {
        let packed = u64::from(img_u32[2 * pos]) | (u64::from(img_u32[2 * pos + 1]) << 32);
        let to = (packed & 0xF_FFFF) as u32;
        let width = ((packed >> 20) & 0xF_FFFF) as u32;
        let sid = (packed >> 40) as u32;
        if x < to {
            let gap = to - x;
            widest = widest.max(gap);
            let gid = e.node(gap, 0);
            Narrow::set_next(e.a, ord1_tail, 1, gid);
            Narrow::set_prev(e.a, gid, 1, ord1_tail);
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                Narrow::set_next(e.a, term0_tail, 0, gid);
            }
            Narrow::set_prev(e.a, gid, 0, term0_tail);
            term0_tail = gid;
        }
        Narrow::set_next(e.a, ord1_tail, 1, sid);
        Narrow::set_prev(e.a, sid, 1, ord1_tail);
        ord1_tail = sid;
        x = to + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = e.node(gap, 0);
        Narrow::set_next(e.a, ord1_tail, 1, gid);
        Narrow::set_prev(e.a, gid, 1, ord1_tail);
        ord1_tail = gid;
        if term0_tail == 0 {
            term0_head = gid;
        } else {
            Narrow::set_next(e.a, term0_tail, 0, gid);
        }
        Narrow::set_prev(e.a, gid, 0, term0_tail);
        term0_tail = gid;
    }
    let ord1_head = if ord1_tail == 0 {
        0
    } else {
        Narrow::next(e.a, 0, 1)
    };
    e.close(0, (term0_head, term0_tail), (ord0_head, ord0_tail));
    e.close(1, (term1_head, term1_tail), (ord1_head, ord1_tail));
    let n_nodes = e.nodes as usize;
    e.base = Narrow::STRIDE * n_nodes;
    e.top = e.base;
    let t1 = t0.map(|_| Instant::now());

    let (mut remaining, mut steps) = (n, 0);
    let spirals = force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
    while !if spirals {
        e.eliminate::<true, true>(&mut remaining, &mut steps)
    } else {
        e.eliminate::<false, true>(&mut remaining, &mut steps)
    } {
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        grow_u32(edges, hdr_words + top + RULE);
        e = Elim {
            a: &mut edges[hdr_words..],
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
    }
    let n_rules = (e.top - e.base) / RULE;
    let t2 = t0.map(|_| Instant::now());

    e.assign_bottom(m);
    let t3 = t0.map(|_| Instant::now());

    gen_cache.next_block();
    let gen_prefix = gen_cache.gen8 << 24;
    begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);

    let mut cursors = [0usize; 256];
    let mut id = Narrow::next(e.a, 0, 0);
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        cursors[a] = bands.len();
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
                let wid = node_word & ID_MASK;
                let height = (node_word >> ID_BITS) as usize;
                if height < INLINE {
                    let entry_u128 = e.short_payload(wid) | (u128::from(a as u8) << (8 * height));
                    let len = (height + 1) as u8;
                    let h = if height < 8 {
                        short_u64_hash(entry_u128 as u64, len)
                    } else {
                        short_u128_hash(entry_u128, len)
                    };
                    let idx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
                    let tag = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
                    let slot = gen_cache.contents[idx];
                    if (slot & 0xFFFF_0000) == tag {
                        let old = (slot & 0xFFFF) as usize;
                        if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                            old as u32
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(entry_u128.to_le_bytes());
                            lens.push(len);
                            if code < 65_536 {
                                gen_cache.contents[idx] = tag | code;
                            }
                            code
                        }
                    } else {
                        let code = lens.len() as u32;
                        inline16.push(entry_u128.to_le_bytes());
                        lens.push(len);
                        if code < 65_536 {
                            gen_cache.contents[idx] = tag | code;
                        }
                        code
                    }
                } else {
                    let start = spelling.len();
                    let memoized = e.a[e.rule(wid) + 3] != u32::MAX;
                    e.append_fast(wid, stack, spelling);
                    spelling.push(a as u8);
                    let end_s = spelling.len();
                    let bytes = &spelling[start..end_s];
                    let h = long_bytes_hash(bytes);
                    let idx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
                    let tag = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
                    let slot = gen_cache.contents[idx];
                    if (slot & 0xFFFF_0000) == tag {
                        let old = (slot & 0xFFFF) as usize;
                        if lens[old] == LONG && long_word(&inline16[old], spelling) == bytes {
                            if memoized {
                                spelling.truncate(start);
                            } else {
                                spelling.pop();
                            }
                            old as u32
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(long_slot(start as u32, end_s as u32));
                            lens.push(LONG);
                            if code < 65_536 {
                                gen_cache.contents[idx] = tag | code;
                            }
                            code
                        }
                    } else {
                        let code = lens.len() as u32;
                        inline16.push(long_slot(start as u32, end_s as u32));
                        lens.push(LONG);
                        if code < 65_536 {
                            gen_cache.contents[idx] = tag | code;
                        }
                        code
                    }
                }
            };
            bands.push(Band::new(k + width, dst, code));
            in_node += width;
            k += width;
            if in_node == node_width {
                id = Narrow::next(e.a, id, 0);
                in_node = 0;
            }
        }
    }
    assert_eq!(id, 0);
    if let Some(p) = ph {
        let t4 = Instant::now();
        p.setup += t1.unwrap().duration_since(t0.unwrap()).as_nanos() as u64;
        p.steps += t2.unwrap().duration_since(t1.unwrap()).as_nanos() as u64;
        p.bot += t3.unwrap().duration_since(t2.unwrap()).as_nanos() as u64;
        p.dict += t4.duration_since(t3.unwrap()).as_nanos() as u64;
    }
    (cursors, n_nodes - 1, n_rules)
}

// ---------------------------------------------------------------------------
// Shared u128 & u64 Rule Joins + Fast Left-Spine Append (for S1, C1, C2, C3)
// ---------------------------------------------------------------------------

#[inline(always)]
fn join_u128_1053(rules: &mut Vec<[u32; 4]>, left: u128, right: u128) -> u128 {
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
    join_u128_long_vec(rules, left, right, ll, rl)
}

#[inline(always)]
fn join_u128_v2(rules: &mut Vec<[u32; 4]>, left: u128, right: u128) -> u128 {
    let ll = (left >> 120) as u32;
    let rl = (right >> 120) as u32;
    if (ll + rl < 16) | (ll.min(rl) == 0) {
        return (left + (u128::from(rl) << 120)) | (right << (8 * (ll & 15)));
    }
    join_u128_long_vec(rules, left, right, ll, rl)
}

#[cold]
#[inline(never)]
fn join_u128_long_vec(
    rules: &mut Vec<[u32; 4]>,
    left: u128,
    right: u128,
    ll: u32,
    rl: u32,
) -> u128 {
    let (lid, l_len) = if ll == 0xFF {
        (left as u32, (left >> 32) as u32)
    } else if ll == 1 {
        (u32::from(left as u8) + 1, 1)
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
    let (rid, r_len) = if left == right {
        (lid, l_len)
    } else if rl == 0xFF {
        (right as u32, (right >> 32) as u32)
    } else if rl == 1 {
        (u32::from(right as u8) + 1, 1)
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

fn power_u128_vec(rules: &mut Vec<[u32; 4]>, word: u128, q: u32) -> u128 {
    let (mut acc, mut base, mut e) = (0u128, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_u128_v2(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_u128_v2(rules, base, base);
    }
}

fn fold_left_power_u128(rules: &mut Vec<[u32; 4]>, x: u128, mut q: u32, mut y: u128) -> u128 {
    if x == 0 {
        return y;
    }
    let xl = (x >> 120) as u32;
    if xl < 16 {
        while q > 0 {
            let yl = (y >> 120) as u32;
            if yl >= 16 || xl + yl >= 16 {
                break;
            }
            y = if y == 0 {
                x
            } else {
                (x + (u128::from(yl) << 120)) | (y << (8 * xl))
            };
            q -= 1;
        }
    }
    if q == 0 {
        return y;
    }
    let head = power_u128_vec(rules, x, q);
    join_u128_v2(rules, head, y)
}

fn fold_right_power_u128(rules: &mut Vec<[u32; 4]>, mut x: u128, y: u128, mut q: u32) -> u128 {
    if y == 0 {
        return x;
    }
    let yl = (y >> 120) as u32;
    if yl < 16 {
        while q > 0 {
            let xl = (x >> 120) as u32;
            if xl >= 16 || xl + yl >= 16 {
                break;
            }
            x = if x == 0 {
                y
            } else {
                (x + (u128::from(yl) << 120)) | (y << (8 * xl))
            };
            q -= 1;
        }
    }
    if q == 0 {
        return x;
    }
    let tail = power_u128_vec(rules, y, q);
    join_u128_v2(rules, x, tail)
}

#[inline(always)]
fn join_u64_v2(rules: &mut Vec<[u32; 4]>, left: u64, right: u64) -> u64 {
    let ll = (left >> 56) as u32;
    let rl = (right >> 56) as u32;
    if (ll + rl < 8) | (ll.min(rl) == 0) {
        return (left + (u64::from(rl) << 56)) | (right << (8 * (ll & 7)));
    }
    join_u64_long_vec(rules, left, right, ll, rl)
}

#[inline(always)]
fn u64_short_payload(rules: &[[u32; 4]], w: u64, len: u32) -> u128 {
    if len < 8 {
        u128::from(w & U64_PAYLOAD_MASK)
    } else {
        let id = (w & U64_ID_MASK) as u32;
        let r = rules[(id - 257) as usize];
        u128::from(r[0])
            | (u128::from(r[1]) << 32)
            | (u128::from(r[2]) << 64)
            | (u128::from(r[3] & 0x00ff_ffff) << 96)
    }
}

#[cold]
#[inline(never)]
fn join_u64_long_vec(rules: &mut Vec<[u32; 4]>, left: u64, right: u64, ll: u32, rl: u32) -> u64 {
    if ll < 16 && rl < 16 && ll + rl < 16 {
        let sum = ll + rl;
        let p = u64_short_payload(rules, left, ll)
            | (u64_short_payload(rules, right, rl) << (8 * ll))
            | (u128::from(sum) << 120);
        let id = rules.len() as u32 + 257;
        rules.push([
            p as u32,
            (p >> 32) as u32,
            (p >> 64) as u32,
            (p >> 96) as u32,
        ]);
        return (u64::from(sum) << 56) | u64::from(id);
    }
    let (lid, l_len) = if ll == 0xFF {
        (
            (left & U64_ID_MASK) as u32,
            ((left >> 24) & 0xFFFF_FFFF) as u32,
        )
    } else if ll == 1 {
        (u32::from(left as u8) + 1, 1)
    } else if ll < 8 {
        let id = rules.len() as u32 + 257;
        let p = left & U64_PAYLOAD_MASK;
        rules.push([p as u32, (p >> 32) as u32, 0, ll << 24]);
        (id, ll)
    } else {
        ((left & U64_ID_MASK) as u32, ll)
    };
    let (rid, r_len) = if left == right {
        (lid, l_len)
    } else if rl == 0xFF {
        (
            (right & U64_ID_MASK) as u32,
            ((right >> 24) & 0xFFFF_FFFF) as u32,
        )
    } else if rl == 1 {
        (u32::from(right as u8) + 1, 1)
    } else if rl < 8 {
        let id = rules.len() as u32 + 257;
        let p = right & U64_PAYLOAD_MASK;
        rules.push([p as u32, (p >> 32) as u32, 0, rl << 24]);
        (id, rl)
    } else {
        ((right & U64_ID_MASK) as u32, rl)
    };
    let total_len = l_len + r_len;
    let id = rules.len() as u32 + 257;
    rules.push([lid, rid, total_len, u32::MAX]);
    U64_LONG_TAG | (u64::from(total_len) << 24) | u64::from(id)
}

fn power_u64_vec(rules: &mut Vec<[u32; 4]>, word: u64, q: u32) -> u64 {
    let (mut acc, mut base, mut e) = (0u64, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_u64_v2(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_u64_v2(rules, base, base);
    }
}

fn fold_left_power_u64(rules: &mut Vec<[u32; 4]>, x: u64, mut q: u32, mut y: u64) -> u64 {
    if x == 0 {
        return y;
    }
    let xl = (x >> 56) as u32;
    if xl < 8 {
        while q > 0 {
            let yl = (y >> 56) as u32;
            if yl >= 8 || xl + yl >= 8 {
                break;
            }
            y = if y == 0 {
                x
            } else {
                (x + (u64::from(yl) << 56)) | (y << (8 * xl))
            };
            q -= 1;
        }
    }
    if q == 0 {
        return y;
    }
    let head = power_u64_vec(rules, x, q);
    join_u64_v2(rules, head, y)
}

fn fold_right_power_u64(rules: &mut Vec<[u32; 4]>, mut x: u64, y: u64, mut q: u32) -> u64 {
    if y == 0 {
        return x;
    }
    let yl = (y >> 56) as u32;
    if yl < 8 {
        while q > 0 {
            let xl = (x >> 56) as u32;
            if xl >= 8 || xl + yl >= 8 {
                break;
            }
            x = if x == 0 {
                y
            } else {
                (x + (u64::from(yl) << 56)) | (y << (8 * xl))
            };
            q -= 1;
        }
    }
    if q == 0 {
        return x;
    }
    let tail = power_u64_vec(rules, y, q);
    join_u64_v2(rules, x, tail)
}

fn append_rules_1053(rules: &mut [[u32; 4]], word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
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

fn append_rules_v2(rules: &mut [[u32; 4]], word: u32, spill: &mut Vec<u32>, dest: &mut Vec<u8>) {
    let mut stk = [0u32; 64];
    let mut sp = 0usize;
    spill.clear();
    let mut cur = word;
    loop {
        if cur <= 256 {
            dest.push((cur - 1) as u8);
        } else {
            let idx = (cur - 257) as usize;
            let tag = rules[idx][3];
            if tag >> 28 == 0 {
                let r = rules[idx];
                let bytes = (u128::from(r[0])
                    | (u128::from(r[1]) << 32)
                    | (u128::from(r[2]) << 64)
                    | (u128::from(r[3] & 0x00ff_ffff) << 96))
                    .to_le_bytes();
                dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
            } else if tag == u32::MAX {
                let r0 = rules[idx][0];
                let r1 = rules[idx][1];
                rules[idx][3] = 0x8000_0000 | dest.len() as u32;
                if sp < 64 {
                    stk[sp] = r1;
                    sp += 1;
                } else {
                    spill.push(r1);
                }
                cur = r0;
                continue;
            } else {
                let start = (tag & 0x7fff_ffff) as usize;
                let len = rules[idx][2] as usize;
                dest.extend_from_within(start..start + len);
            }
        }
        if let Some(next) = spill.pop() {
            cur = next;
        } else if sp > 0 {
            sp -= 1;
            cur = stk[sp];
        } else {
            break;
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn dict_code_u128_v2(
    word: u128,
    byte: u8,
    rules: &mut [[u32; 4]],
    gen_cache: &mut GenCacheV2,
    gen_prefix: u32,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
) -> u32 {
    let tag = (word >> 120) as u32;
    if tag < 8 {
        let low = (word as u64) | (u64::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let h = short_u64_hash(low, len);
        let idx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
        let tag_word = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
        let slot = gen_cache.contents[idx];
        if (slot & 0xFFFF_0000) == tag_word {
            let old = (slot & 0xFFFF) as usize;
            if lens[old] == len && u64::from_le_bytes(inline16[old][..8].try_into().unwrap()) == low
            {
                return old as u32;
            }
        }
        let code = lens.len() as u32;
        inline16.push(u128::from(low).to_le_bytes());
        lens.push(len);
        if code < 65_536 {
            gen_cache.contents[idx] = tag_word | code;
        }
        code
    } else if tag < 16 {
        let entry_u128 = (word & U128_PAYLOAD_MASK) | (u128::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let h = short_u128_hash(entry_u128, len);
        let idx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
        let tag_word = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
        let slot = gen_cache.contents[idx];
        if (slot & 0xFFFF_0000) == tag_word {
            let old = (slot & 0xFFFF) as usize;
            if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                return old as u32;
            }
        }
        let code = lens.len() as u32;
        inline16.push(entry_u128.to_le_bytes());
        lens.push(len);
        if code < 65_536 {
            gen_cache.contents[idx] = tag_word | code;
        }
        code
    } else {
        dict_code_long_v2(
            word as u32,
            byte,
            rules,
            gen_cache,
            gen_prefix,
            inline16,
            lens,
            spelling,
            stack,
        )
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn dict_code_u64_v2(
    word: u64,
    byte: u8,
    rules: &mut [[u32; 4]],
    gen_cache: &mut GenCacheV2,
    gen_prefix: u32,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
) -> u32 {
    let tag = (word >> 56) as u32;
    if tag < 8 {
        let low = (word & U64_PAYLOAD_MASK) | (u64::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let h = short_u64_hash(low, len);
        let idx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
        let tag_word = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
        let slot = gen_cache.contents[idx];
        if (slot & 0xFFFF_0000) == tag_word {
            let old = (slot & 0xFFFF) as usize;
            if lens[old] == len && u64::from_le_bytes(inline16[old][..8].try_into().unwrap()) == low
            {
                return old as u32;
            }
        }
        let code = lens.len() as u32;
        inline16.push(u128::from(low).to_le_bytes());
        lens.push(len);
        if code < 65_536 {
            gen_cache.contents[idx] = tag_word | code;
        }
        code
    } else if tag < 16 {
        let id = (word & U64_ID_MASK) as u32;
        let r = rules[(id - 257) as usize];
        let payload = u128::from(r[0])
            | (u128::from(r[1]) << 32)
            | (u128::from(r[2]) << 64)
            | (u128::from(r[3] & 0x00ff_ffff) << 96);
        let entry_u128 = payload | (u128::from(byte) << (8 * tag));
        let len = (tag + 1) as u8;
        let h = short_u128_hash(entry_u128, len);
        let idx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
        let tag_word = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
        let slot = gen_cache.contents[idx];
        if (slot & 0xFFFF_0000) == tag_word {
            let old = (slot & 0xFFFF) as usize;
            if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                return old as u32;
            }
        }
        let code = lens.len() as u32;
        inline16.push(entry_u128.to_le_bytes());
        lens.push(len);
        if code < 65_536 {
            gen_cache.contents[idx] = tag_word | code;
        }
        code
    } else {
        dict_code_long_v2(
            (word & U64_ID_MASK) as u32,
            byte,
            rules,
            gen_cache,
            gen_prefix,
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
fn dict_code_long_v2(
    id: u32,
    byte: u8,
    rules: &mut [[u32; 4]],
    gen_cache: &mut GenCacheV2,
    gen_prefix: u32,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
) -> u32 {
    let start = spelling.len();
    let memoized = rules[(id - 257) as usize][3] != u32::MAX;
    append_rules_v2(rules, id, stack, spelling);
    spelling.push(byte);
    let end = spelling.len();
    let bytes = &spelling[start..end];
    let h = long_bytes_hash(bytes);
    let idx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
    let tag_word = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
    let slot = gen_cache.contents[idx];
    if (slot & 0xFFFF_0000) == tag_word {
        let old = (slot & 0xFFFF) as usize;
        if lens[old] == LONG && long_word(&inline16[old], spelling) == bytes {
            if memoized {
                spelling.truncate(start);
            } else {
                spelling.pop();
            }
            return old as u32;
        }
    }
    let code = lens.len() as u32;
    inline16.push(long_slot(start as u32, end as u32));
    lens.push(LONG);
    if code < 65_536 {
        gen_cache.contents[idx] = tag_word | code;
    }
    code
}

// ---------------------------------------------------------------------------
// Arm S1: Exact #1053 (Fixed64kSoa + GenCache1053)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn induced_s1_1053(
    strips_raw: &[u32],
    bands: &mut Vec<Band>,
    spelling: &mut Vec<u8>,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    img_buf: &mut Vec<u64>,
    fixed_soa: &mut Option<Fixed64kSoa>,
    rules: &mut Vec<[u32; 4]>,
    gen_cache: &mut GenCache1053,
    ns: usize,
    n: u32,
    m: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    thin: &[u32; 256],
    force_spirals: Option<bool>,
    ph: Option<&mut PhaseNs>,
) -> ([usize; 256], usize, usize) {
    let t0 = ph.as_ref().map(|_| Instant::now());
    let soa = fixed_soa.get_or_insert_with(Fixed64kSoa::new);
    if img_buf.len() < ns {
        img_buf.resize(ns, 0);
    }
    let img = &mut img_buf[..ns];
    let links = &mut *soa.links;
    let widths = &mut *soa.widths;
    let words = &mut *soa.words128;

    let (widest, nodes_used) = setup_soa_u128_fixed::<NARROW_NODES>(
        strips_raw, ns, n, first, write, img, links, widths, words,
    );
    let t1 = t0.map(|_| Instant::now());

    let spirals = force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
    if spirals {
        elim_soa_u128_fixed::<NARROW_NODES, true, false>(links, widths, words, rules, n);
    } else {
        elim_soa_u128_fixed::<NARROW_NODES, false, false>(links, widths, words, rules, n);
    }
    let n_rules = rules.len();
    let t2 = t0.map(|_| Instant::now());

    assign_bottom_soa_fixed::<NARROW_NODES>(links, widths, m);
    let t3 = t0.map(|_| Instant::now());

    gen_cache.next_block();
    begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
    let mut cursors = [0usize; 256];
    let rslice = &mut rules[..];
    let mut id = links[0].next0;
    let mut node_width = widths[usize::from(id)];
    let mut node_word = words[usize::from(id)];
    let l0 = links[usize::from(id)];
    let mut node_bottom = u32::from(l0.prev1) | (u32::from(l0.next1) << 16);
    let mut node_next = l0.next0;
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        cursors[a] = bands.len();
        let end = k + thin[a];
        while k < end {
            let width = (end - k).min(node_width - in_node);
            assert!(width > 0);
            let dst = node_bottom + in_node;
            let code = if node_word == 0 {
                a as u32
            } else {
                let tag = (node_word >> 120) as u32;
                if tag < 16 {
                    let entry_u128 =
                        (node_word & U128_PAYLOAD_MASK) | (u128::from(a as u8) << (8 * tag));
                    let len = (tag + 1) as u8;
                    let idx = (short_u128_hash(entry_u128, len) as usize) & (GEN_CONTENT_SLOTS - 1);
                    let g = gen_cache.content_gen;
                    let slot = gen_cache.contents[idx];
                    if (slot >> 16) == g {
                        let old = (slot & 0xFFFF) as usize;
                        if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                            old as u32
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(entry_u128.to_le_bytes());
                            lens.push(len);
                            if code < 65_536 {
                                gen_cache.contents[idx] = (g << 16) | code;
                            }
                            code
                        }
                    } else {
                        let code = lens.len() as u32;
                        inline16.push(entry_u128.to_le_bytes());
                        lens.push(len);
                        if code < 65_536 {
                            gen_cache.contents[idx] = (g << 16) | code;
                        }
                        code
                    }
                } else {
                    dict_code_long_1053(
                        node_word as u32,
                        a as u8,
                        rslice,
                        gen_cache,
                        inline16,
                        lens,
                        spelling,
                        stack,
                    )
                }
            };
            bands.push(Band::new(k + width, dst, code));
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
    if let Some(p) = ph {
        let t4 = Instant::now();
        p.setup += t1.unwrap().duration_since(t0.unwrap()).as_nanos() as u64;
        p.steps += t2.unwrap().duration_since(t1.unwrap()).as_nanos() as u64;
        p.bot += t3.unwrap().duration_since(t2.unwrap()).as_nanos() as u64;
        p.dict += t4.duration_since(t3.unwrap()).as_nanos() as u64;
    }
    (cursors, nodes_used - 1, n_rules)
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn dict_code_long_1053(
    id: u32,
    byte: u8,
    rules: &mut [[u32; 4]],
    gen_cache: &mut GenCache1053,
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
    append_rules_1053(rules, id, stack, spelling);
    spelling.push(byte);
    let end = spelling.len();
    let entry = long_slot(start as u32, end as u32);
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
    if id < (1 << 22) && code < 65_536 {
        gen_cache.pairs[p_idx] =
            (gen_cache.pair_gen << 46) | (u64::from(code) << 30) | u64::from(key);
    }
    code
}

// ---------------------------------------------------------------------------
// Generic fixed-CAP SoA helpers (used by S1 at 64K, C2 at 8/16/32/64K, C3 u64)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn setup_soa_u128_fixed<const CAP: usize>(
    strips_raw: &[u32],
    ns: usize,
    n: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    img: &mut [u64],
    links: &mut [NodeLinks; CAP],
    widths: &mut [u32; CAP],
    words: &mut [u128; CAP],
) -> (u32, usize) {
    let mask = CAP - 1;
    links[0] = NodeLinks::default();
    widths[0] = 0;
    words[0] = 0;
    let mut nodes = (1 + ns) as u32;
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
        let si = usize::from(sid) & mask;
        widths[si] = width;
        words[si] = literal_u128(byte);
        widest = widest.max(width);
        let pos = write[a] as usize;
        write[a] = (pos + 1) as u32;
        img[pos] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);
        if x < from {
            let gap = from - x;
            widest = widest.max(gap);
            let gid = nodes as u16;
            nodes += 1;
            let gi = usize::from(gid) & mask;
            widths[gi] = gap;
            words[gi] = 0;
            links[usize::from(ord0_tail) & mask].next0 = gid;
            links[gi].prev0 = ord0_tail;
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                links[usize::from(term1_tail) & mask].next1 = gid;
            }
            links[gi].prev1 = term1_tail;
            term1_tail = gid;
        }
        links[usize::from(ord0_tail) & mask].next0 = sid;
        links[si].prev0 = ord0_tail;
        ord0_tail = sid;
        x = from + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes as u16;
        nodes += 1;
        let gi = usize::from(gid) & mask;
        widths[gi] = gap;
        words[gi] = 0;
        links[usize::from(ord0_tail) & mask].next0 = gid;
        links[gi].prev0 = ord0_tail;
        ord0_tail = gid;
        if term1_tail == 0 {
            term1_head = gid;
        } else {
            links[usize::from(term1_tail) & mask].next1 = gid;
        }
        links[gi].prev1 = term1_tail;
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
        let si = usize::from(sid) & mask;
        if x < to {
            let gap = to - x;
            widest = widest.max(gap);
            let gid = nodes as u16;
            nodes += 1;
            let gi = usize::from(gid) & mask;
            widths[gi] = gap;
            words[gi] = 0;
            links[usize::from(ord1_tail) & mask].next1 = gid;
            links[gi].prev1 = ord1_tail;
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                links[usize::from(term0_tail) & mask].next0 = gid;
            }
            links[gi].prev0 = term0_tail;
            term0_tail = gid;
        }
        links[usize::from(ord1_tail) & mask].next1 = sid;
        links[si].prev1 = ord1_tail;
        ord1_tail = sid;
        x = to + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes as u16;
        nodes += 1;
        let gi = usize::from(gid) & mask;
        widths[gi] = gap;
        words[gi] = 0;
        links[usize::from(ord1_tail) & mask].next1 = gid;
        links[gi].prev1 = ord1_tail;
        ord1_tail = gid;
        if term0_tail == 0 {
            term0_head = gid;
        } else {
            links[usize::from(term0_tail) & mask].next0 = gid;
        }
        links[gi].prev0 = term0_tail;
        term0_tail = gid;
    }
    let ord1_head = if ord1_tail == 0 { 0 } else { links[0].next1 };
    let mut prev = 0u16;
    for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
        if t == 0 {
            continue;
        }
        links[usize::from(prev) & mask].next0 = h;
        links[usize::from(h) & mask].prev0 = prev;
        prev = t;
    }
    links[usize::from(prev) & mask].next0 = 0;
    links[0].prev0 = prev;

    prev = 0u16;
    for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
        if t == 0 {
            continue;
        }
        links[usize::from(prev) & mask].next1 = h;
        links[usize::from(h) & mask].prev1 = prev;
        prev = t;
    }
    links[usize::from(prev) & mask].next1 = 0;
    links[0].prev1 = prev;
    (widest, nodes as usize)
}

fn elim_soa_u128_fixed<const CAP: usize, const SPIRALS: bool, const V2_JOIN: bool>(
    links: &mut [NodeLinks; CAP],
    widths: &mut [u32; CAP],
    words: &mut [u128; CAP],
    rules: &mut Vec<[u32; 4]>,
    n: u32,
) {
    let mask = CAP - 1;
    rules.clear();
    let mut left = n;
    while left != 0 {
        let x = links[0].prev0;
        let y = links[0].prev1;
        let xi = usize::from(x) & mask;
        let yi = usize::from(y) & mask;
        let aw = widths[xi];
        let bw = widths[yi];
        if SPIRALS && x != y {
            if aw > bw && links[yi].prev1 == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                assert!(q * bw < aw && q * bw <= left);
                words[yi] = fold_left_power_u128(rules, words[xi], q, words[yi]);
                widths[xi] = aw - q * bw;
                left -= q * bw;
                continue;
            }
            if aw < bw && links[xi].prev0 == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                assert!(q * aw < bw && q * aw <= left);
                words[xi] = fold_right_power_u128(rules, words[xi], words[yi], q);
                widths[yi] = bw - q * aw;
                left -= q * aw;
                continue;
            }
        }
        let w = aw.min(bw);
        assert!(w > 0 && w <= left);
        left -= w;
        if x == y {
            let xp = links[xi].prev0;
            links[usize::from(xp) & mask].next0 = 0;
            links[0].prev0 = xp;
            let yp = links[yi].prev1;
            links[usize::from(yp) & mask].next1 = 0;
            links[0].prev1 = yp;
            continue;
        }
        let word = if V2_JOIN {
            join_u128_v2(rules, words[xi], words[yi])
        } else {
            join_u128_1053(rules, words[xi], words[yi])
        };
        if aw > bw {
            let yp = links[yi].prev1;
            links[usize::from(yp) & mask].next1 = 0;
            links[0].prev1 = yp;
            let xn = links[xi].next1;
            links[yi].prev1 = x;
            links[yi].next1 = xn;
            links[xi].next1 = y;
            links[usize::from(xn) & mask].prev1 = y;
            words[yi] = word;
            widths[xi] = aw - bw;
        } else if aw < bw {
            let xp = links[xi].prev0;
            links[usize::from(xp) & mask].next0 = 0;
            links[0].prev0 = xp;
            let yn = links[yi].next0;
            links[xi].prev0 = y;
            links[xi].next0 = yn;
            links[yi].next0 = x;
            links[usize::from(yn) & mask].prev0 = x;
            words[xi] = word;
            widths[yi] = bw - aw;
        } else {
            let xp = links[xi].prev0;
            links[usize::from(xp) & mask].next0 = 0;
            links[0].prev0 = xp;
            let yp0 = links[yi].prev0;
            let yn0 = links[yi].next0;
            links[xi].prev0 = yp0;
            links[xi].next0 = yn0;
            links[usize::from(yp0) & mask].next0 = x;
            links[usize::from(yn0) & mask].prev0 = x;
            let yp1 = links[yi].prev1;
            links[usize::from(yp1) & mask].next1 = 0;
            links[0].prev1 = yp1;
            words[xi] = word;
        }
    }
}

fn assign_bottom_soa_fixed<const CAP: usize>(
    links: &mut [NodeLinks; CAP],
    widths: &[u32; CAP],
    rows: u32,
) {
    let mask = CAP - 1;
    let mut first = links[0].next1;
    let mut last = links[0].prev1;
    let (mut lo, mut hi) = (0u32, rows);
    while first != last {
        let fi = usize::from(first) & mask;
        let li = usize::from(last) & mask;
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
        let fi = usize::from(first) & mask;
        links[fi].prev1 = lo as u16;
        links[fi].next1 = (lo >> 16) as u16;
        lo += widths[fi];
    }
    assert_eq!(lo, hi);
}

#[allow(clippy::too_many_arguments)]
fn run_soa_u128_fixed_v2<const CAP: usize>(
    strips_raw: &[u32],
    bands: &mut Vec<Band>,
    spelling: &mut Vec<u8>,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    img: &mut [u64],
    links: &mut [NodeLinks; CAP],
    widths: &mut [u32; CAP],
    words: &mut [u128; CAP],
    rules: &mut Vec<[u32; 4]>,
    gen_cache: &mut GenCacheV2,
    ns: usize,
    n: u32,
    m: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    thin: &[u32; 256],
    force_spirals: Option<bool>,
    ph: Option<&mut PhaseNs>,
) -> ([usize; 256], usize, usize) {
    let t0 = ph.as_ref().map(|_| Instant::now());
    let (widest, nodes_used) =
        setup_soa_u128_fixed::<CAP>(strips_raw, ns, n, first, write, img, links, widths, words);
    let t1 = t0.map(|_| Instant::now());
    let spirals = force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
    if spirals {
        elim_soa_u128_fixed::<CAP, true, true>(links, widths, words, rules, n);
    } else {
        elim_soa_u128_fixed::<CAP, false, true>(links, widths, words, rules, n);
    }
    let n_rules = rules.len();
    let t2 = t0.map(|_| Instant::now());
    assign_bottom_soa_fixed::<CAP>(links, widths, m);
    let t3 = t0.map(|_| Instant::now());

    gen_cache.next_block();
    let gen_prefix = gen_cache.gen8 << 24;
    begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
    let mask = CAP - 1;
    let rslice = &mut rules[..];
    let mut cursors = [0usize; 256];
    let mut id = links[0].next0;
    let mut idx = usize::from(id) & mask;
    let mut node_width = widths[idx];
    let mut node_word = words[idx];
    let l0 = links[idx];
    let mut node_bottom = u32::from(l0.prev1) | (u32::from(l0.next1) << 16);
    let mut node_next = l0.next0;
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        cursors[a] = bands.len();
        let end = k + thin[a];
        while k < end {
            let width = (end - k).min(node_width - in_node);
            assert!(width > 0);
            let dst = node_bottom + in_node;
            let code = if node_word == 0 {
                a as u32
            } else {
                dict_code_u128_v2(
                    node_word, a as u8, rslice, gen_cache, gen_prefix, inline16, lens, spelling,
                    stack,
                )
            };
            bands.push(Band::new(k + width, dst, code));
            in_node += width;
            k += width;
            if in_node == node_width {
                id = node_next;
                in_node = 0;
                if id != 0 {
                    idx = usize::from(id) & mask;
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
    if let Some(p) = ph {
        let t4 = Instant::now();
        p.setup += t1.unwrap().duration_since(t0.unwrap()).as_nanos() as u64;
        p.steps += t2.unwrap().duration_since(t1.unwrap()).as_nanos() as u64;
        p.bot += t3.unwrap().duration_since(t2.unwrap()).as_nanos() as u64;
        p.dict += t4.duration_since(t3.unwrap()).as_nanos() as u64;
    }
    (cursors, nodes_used - 1, n_rules)
}

// ---------------------------------------------------------------------------
// Arm C1: Dynamic Vec-sized SoA u128 + V2 wins
// ---------------------------------------------------------------------------

fn elim_c1_vec_u128<const SPIRALS: bool>(
    links: &mut [NodeLinks],
    widths: &mut [u32],
    words: &mut [u128],
    rules: &mut Vec<[u32; 4]>,
    n: u32,
) {
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
                assert!(q * bw < aw && q * bw <= left);
                words[yi] = fold_left_power_u128(rules, words[xi], q, words[yi]);
                widths[xi] = aw - q * bw;
                left -= q * bw;
                continue;
            }
            if aw < bw && links[xi].prev0 == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                assert!(q * aw < bw && q * aw <= left);
                words[xi] = fold_right_power_u128(rules, words[xi], words[yi], q);
                widths[yi] = bw - q * aw;
                left -= q * aw;
                continue;
            }
        }
        let w = aw.min(bw);
        assert!(w > 0 && w <= left);
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
        let word = join_u128_v2(rules, words[xi], words[yi]);
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

#[allow(clippy::too_many_arguments)]
fn induced_c1_vec_soa_v2(
    strips_raw: &[u32],
    bands: &mut Vec<Band>,
    spelling: &mut Vec<u8>,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    img_buf: &mut Vec<u64>,
    links_v: &mut Vec<NodeLinks>,
    widths_v: &mut Vec<u32>,
    words_v: &mut Vec<u128>,
    rules: &mut Vec<[u32; 4]>,
    gen_cache: &mut GenCacheV2,
    ns: usize,
    n: u32,
    m: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    thin: &[u32; 256],
    force_spirals: Option<bool>,
    ph: Option<&mut PhaseNs>,
) -> ([usize; 256], usize, usize) {
    let t0 = ph.as_ref().map(|_| Instant::now());
    let most = 3 * ns + 3;
    if img_buf.len() < ns {
        img_buf.resize(ns, 0);
    }
    if links_v.len() < most {
        links_v.resize(most, NodeLinks::default());
        widths_v.resize(most, 0);
        words_v.resize(most, 0);
    }
    let img = &mut img_buf[..ns];
    let links = &mut links_v[..most];
    let widths = &mut widths_v[..most];
    let words = &mut words_v[..most];

    links[0] = NodeLinks::default();
    widths[0] = 0;
    words[0] = 0;
    let mut nodes = (1 + ns) as u32;
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
        let si = usize::from(sid);
        widths[si] = width;
        words[si] = literal_u128(byte);
        widest = widest.max(width);
        let pos = write[a] as usize;
        write[a] = (pos + 1) as u32;
        img[pos] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);
        if x < from {
            let gap = from - x;
            widest = widest.max(gap);
            let gid = nodes as u16;
            nodes += 1;
            let gi = usize::from(gid);
            widths[gi] = gap;
            words[gi] = 0;
            links[usize::from(ord0_tail)].next0 = gid;
            links[gi].prev0 = ord0_tail;
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                links[usize::from(term1_tail)].next1 = gid;
            }
            links[gi].prev1 = term1_tail;
            term1_tail = gid;
        }
        links[usize::from(ord0_tail)].next0 = sid;
        links[si].prev0 = ord0_tail;
        ord0_tail = sid;
        x = from + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes as u16;
        nodes += 1;
        let gi = usize::from(gid);
        widths[gi] = gap;
        words[gi] = 0;
        links[usize::from(ord0_tail)].next0 = gid;
        links[gi].prev0 = ord0_tail;
        ord0_tail = gid;
        if term1_tail == 0 {
            term1_head = gid;
        } else {
            links[usize::from(term1_tail)].next1 = gid;
        }
        links[gi].prev1 = term1_tail;
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
        let si = usize::from(sid);
        if x < to {
            let gap = to - x;
            widest = widest.max(gap);
            let gid = nodes as u16;
            nodes += 1;
            let gi = usize::from(gid);
            widths[gi] = gap;
            words[gi] = 0;
            links[usize::from(ord1_tail)].next1 = gid;
            links[gi].prev1 = ord1_tail;
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                links[usize::from(term0_tail)].next0 = gid;
            }
            links[gi].prev0 = term0_tail;
            term0_tail = gid;
        }
        links[usize::from(ord1_tail)].next1 = sid;
        links[si].prev1 = ord1_tail;
        ord1_tail = sid;
        x = to + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes as u16;
        nodes += 1;
        let gi = usize::from(gid);
        widths[gi] = gap;
        words[gi] = 0;
        links[usize::from(ord1_tail)].next1 = gid;
        links[gi].prev1 = ord1_tail;
        ord1_tail = gid;
        if term0_tail == 0 {
            term0_head = gid;
        } else {
            links[usize::from(term0_tail)].next0 = gid;
        }
        links[gi].prev0 = term0_tail;
        term0_tail = gid;
    }
    let ord1_head = if ord1_tail == 0 { 0 } else { links[0].next1 };
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
    prev = 0u16;
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
    let nodes_used = nodes as usize;
    let t1 = t0.map(|_| Instant::now());

    let spirals = force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
    if spirals {
        elim_c1_vec_u128::<true>(links, widths, words, rules, n);
    } else {
        elim_c1_vec_u128::<false>(links, widths, words, rules, n);
    }
    let n_rules = rules.len();
    let t2 = t0.map(|_| Instant::now());

    // assign bottom
    let mut first_n = links[0].next1;
    let mut last_n = links[0].prev1;
    let (mut lo, mut hi) = (0u32, m);
    while first_n != last_n {
        let fi = usize::from(first_n);
        let li = usize::from(last_n);
        let next = links[fi].next1;
        let prev = links[li].prev1;
        let wf = widths[fi];
        hi -= widths[li];
        links[fi].prev1 = lo as u16;
        links[fi].next1 = (lo >> 16) as u16;
        links[li].prev1 = hi as u16;
        links[li].next1 = (hi >> 16) as u16;
        lo += wf;
        if next == last_n {
            assert_eq!(lo, hi);
            first_n = 0;
            break;
        }
        first_n = next;
        last_n = prev;
    }
    if first_n != 0 {
        let fi = usize::from(first_n);
        links[fi].prev1 = lo as u16;
        links[fi].next1 = (lo >> 16) as u16;
        lo += widths[fi];
        assert_eq!(lo, hi);
    }
    let t3 = t0.map(|_| Instant::now());

    gen_cache.next_block();
    let gen_prefix = gen_cache.gen8 << 24;
    begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
    let rslice = &mut rules[..];
    let mut cursors = [0usize; 256];
    let mut id = links[0].next0;
    let mut node_width = widths[usize::from(id)];
    let mut node_word = words[usize::from(id)];
    let l0 = links[usize::from(id)];
    let mut node_bottom = u32::from(l0.prev1) | (u32::from(l0.next1) << 16);
    let mut node_next = l0.next0;
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        cursors[a] = bands.len();
        let end = k + thin[a];
        while k < end {
            let width = (end - k).min(node_width - in_node);
            assert!(width > 0);
            let dst = node_bottom + in_node;
            let code = if node_word == 0 {
                a as u32
            } else {
                dict_code_u128_v2(
                    node_word, a as u8, rslice, gen_cache, gen_prefix, inline16, lens, spelling,
                    stack,
                )
            };
            bands.push(Band::new(k + width, dst, code));
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
    if let Some(p) = ph {
        let t4 = Instant::now();
        p.setup += t1.unwrap().duration_since(t0.unwrap()).as_nanos() as u64;
        p.steps += t2.unwrap().duration_since(t1.unwrap()).as_nanos() as u64;
        p.bot += t3.unwrap().duration_since(t2.unwrap()).as_nanos() as u64;
        p.dict += t4.duration_since(t3.unwrap()).as_nanos() as u64;
    }
    (cursors, nodes_used - 1, n_rules)
}

// ---------------------------------------------------------------------------
// Arm C3: Compact Word8 (u64 inline-7) Tiered SoA + V2 wins
// ---------------------------------------------------------------------------

fn elim_soa_u64_fixed<const CAP: usize, const SPIRALS: bool>(
    links: &mut [NodeLinks; CAP],
    widths: &mut [u32; CAP],
    words: &mut [u64; CAP],
    rules: &mut Vec<[u32; 4]>,
    n: u32,
) {
    let mask = CAP - 1;
    rules.clear();
    let mut left = n;
    while left != 0 {
        let x = links[0].prev0;
        let y = links[0].prev1;
        let xi = usize::from(x) & mask;
        let yi = usize::from(y) & mask;
        let aw = widths[xi];
        let bw = widths[yi];
        if SPIRALS && x != y {
            if aw > bw && links[yi].prev1 == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                assert!(q * bw < aw && q * bw <= left);
                words[yi] = fold_left_power_u64(rules, words[xi], q, words[yi]);
                widths[xi] = aw - q * bw;
                left -= q * bw;
                continue;
            }
            if aw < bw && links[xi].prev0 == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                assert!(q * aw < bw && q * aw <= left);
                words[xi] = fold_right_power_u64(rules, words[xi], words[yi], q);
                widths[yi] = bw - q * aw;
                left -= q * aw;
                continue;
            }
        }
        let w = aw.min(bw);
        assert!(w > 0 && w <= left);
        left -= w;
        if x == y {
            let xp = links[xi].prev0;
            links[usize::from(xp) & mask].next0 = 0;
            links[0].prev0 = xp;
            let yp = links[yi].prev1;
            links[usize::from(yp) & mask].next1 = 0;
            links[0].prev1 = yp;
            continue;
        }
        let word = join_u64_v2(rules, words[xi], words[yi]);
        if aw > bw {
            let yp = links[yi].prev1;
            links[usize::from(yp) & mask].next1 = 0;
            links[0].prev1 = yp;
            let xn = links[xi].next1;
            links[yi].prev1 = x;
            links[yi].next1 = xn;
            links[xi].next1 = y;
            links[usize::from(xn) & mask].prev1 = y;
            words[yi] = word;
            widths[xi] = aw - bw;
        } else if aw < bw {
            let xp = links[xi].prev0;
            links[usize::from(xp) & mask].next0 = 0;
            links[0].prev0 = xp;
            let yn = links[yi].next0;
            links[xi].prev0 = y;
            links[xi].next0 = yn;
            links[yi].next0 = x;
            links[usize::from(yn) & mask].prev0 = x;
            words[xi] = word;
            widths[yi] = bw - aw;
        } else {
            let xp = links[xi].prev0;
            links[usize::from(xp) & mask].next0 = 0;
            links[0].prev0 = xp;
            let yp0 = links[yi].prev0;
            let yn0 = links[yi].next0;
            links[xi].prev0 = yp0;
            links[xi].next0 = yn0;
            links[usize::from(yp0) & mask].next0 = x;
            links[usize::from(yn0) & mask].prev0 = x;
            let yp1 = links[yi].prev1;
            links[usize::from(yp1) & mask].next1 = 0;
            links[0].prev1 = yp1;
            words[xi] = word;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_soa_u64_fixed_v2<const CAP: usize>(
    strips_raw: &[u32],
    bands: &mut Vec<Band>,
    spelling: &mut Vec<u8>,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    img: &mut [u64],
    links: &mut [NodeLinks; CAP],
    widths: &mut [u32; CAP],
    words: &mut [u64; CAP],
    rules: &mut Vec<[u32; 4]>,
    gen_cache: &mut GenCacheV2,
    ns: usize,
    n: u32,
    m: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    thin: &[u32; 256],
    force_spirals: Option<bool>,
    ph: Option<&mut PhaseNs>,
) -> ([usize; 256], usize, usize) {
    let t0 = ph.as_ref().map(|_| Instant::now());
    let mask = CAP - 1;
    links[0] = NodeLinks::default();
    widths[0] = 0;
    words[0] = 0;
    let mut nodes = (1 + ns) as u32;
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
        let si = usize::from(sid) & mask;
        widths[si] = width;
        words[si] = literal_u64(byte);
        widest = widest.max(width);
        let pos = write[a] as usize;
        write[a] = (pos + 1) as u32;
        img[pos] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);
        if x < from {
            let gap = from - x;
            widest = widest.max(gap);
            let gid = nodes as u16;
            nodes += 1;
            let gi = usize::from(gid) & mask;
            widths[gi] = gap;
            words[gi] = 0;
            links[usize::from(ord0_tail) & mask].next0 = gid;
            links[gi].prev0 = ord0_tail;
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                links[usize::from(term1_tail) & mask].next1 = gid;
            }
            links[gi].prev1 = term1_tail;
            term1_tail = gid;
        }
        links[usize::from(ord0_tail) & mask].next0 = sid;
        links[si].prev0 = ord0_tail;
        ord0_tail = sid;
        x = from + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes as u16;
        nodes += 1;
        let gi = usize::from(gid) & mask;
        widths[gi] = gap;
        words[gi] = 0;
        links[usize::from(ord0_tail) & mask].next0 = gid;
        links[gi].prev0 = ord0_tail;
        ord0_tail = gid;
        if term1_tail == 0 {
            term1_head = gid;
        } else {
            links[usize::from(term1_tail) & mask].next1 = gid;
        }
        links[gi].prev1 = term1_tail;
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
        let si = usize::from(sid) & mask;
        if x < to {
            let gap = to - x;
            widest = widest.max(gap);
            let gid = nodes as u16;
            nodes += 1;
            let gi = usize::from(gid) & mask;
            widths[gi] = gap;
            words[gi] = 0;
            links[usize::from(ord1_tail) & mask].next1 = gid;
            links[gi].prev1 = ord1_tail;
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                links[usize::from(term0_tail) & mask].next0 = gid;
            }
            links[gi].prev0 = term0_tail;
            term0_tail = gid;
        }
        links[usize::from(ord1_tail) & mask].next1 = sid;
        links[si].prev1 = ord1_tail;
        ord1_tail = sid;
        x = to + width;
    }
    if x < n {
        let gap = n - x;
        widest = widest.max(gap);
        let gid = nodes as u16;
        nodes += 1;
        let gi = usize::from(gid) & mask;
        widths[gi] = gap;
        words[gi] = 0;
        links[usize::from(ord1_tail) & mask].next1 = gid;
        links[gi].prev1 = ord1_tail;
        ord1_tail = gid;
        if term0_tail == 0 {
            term0_head = gid;
        } else {
            links[usize::from(term0_tail) & mask].next0 = gid;
        }
        links[gi].prev0 = term0_tail;
        term0_tail = gid;
    }
    let ord1_head = if ord1_tail == 0 { 0 } else { links[0].next1 };
    let mut prev = 0u16;
    for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
        if t == 0 {
            continue;
        }
        links[usize::from(prev) & mask].next0 = h;
        links[usize::from(h) & mask].prev0 = prev;
        prev = t;
    }
    links[usize::from(prev) & mask].next0 = 0;
    links[0].prev0 = prev;
    prev = 0u16;
    for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
        if t == 0 {
            continue;
        }
        links[usize::from(prev) & mask].next1 = h;
        links[usize::from(h) & mask].prev1 = prev;
        prev = t;
    }
    links[usize::from(prev) & mask].next1 = 0;
    links[0].prev1 = prev;
    let nodes_used = nodes as usize;
    let t1 = t0.map(|_| Instant::now());

    let spirals = force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
    if spirals {
        elim_soa_u64_fixed::<CAP, true>(links, widths, words, rules, n);
    } else {
        elim_soa_u64_fixed::<CAP, false>(links, widths, words, rules, n);
    }
    let n_rules = rules.len();
    let t2 = t0.map(|_| Instant::now());
    assign_bottom_soa_fixed::<CAP>(links, widths, m);
    let t3 = t0.map(|_| Instant::now());

    gen_cache.next_block();
    let gen_prefix = gen_cache.gen8 << 24;
    begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
    let rslice = &mut rules[..];
    let mut cursors = [0usize; 256];
    let mut id = links[0].next0;
    let mut idx = usize::from(id) & mask;
    let mut node_width = widths[idx];
    let mut node_word = words[idx];
    let l0 = links[idx];
    let mut node_bottom = u32::from(l0.prev1) | (u32::from(l0.next1) << 16);
    let mut node_next = l0.next0;
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        cursors[a] = bands.len();
        let end = k + thin[a];
        while k < end {
            let width = (end - k).min(node_width - in_node);
            assert!(width > 0);
            let dst = node_bottom + in_node;
            let code = if node_word == 0 {
                a as u32
            } else {
                dict_code_u64_v2(
                    node_word, a as u8, rslice, gen_cache, gen_prefix, inline16, lens, spelling,
                    stack,
                )
            };
            bands.push(Band::new(k + width, dst, code));
            in_node += width;
            k += width;
            if in_node == node_width {
                id = node_next;
                in_node = 0;
                if id != 0 {
                    idx = usize::from(id) & mask;
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
    if let Some(p) = ph {
        let t4 = Instant::now();
        p.setup += t1.unwrap().duration_since(t0.unwrap()).as_nanos() as u64;
        p.steps += t2.unwrap().duration_since(t1.unwrap()).as_nanos() as u64;
        p.bot += t3.unwrap().duration_since(t2.unwrap()).as_nanos() as u64;
        p.dict += t4.duration_since(t3.unwrap()).as_nanos() as u64;
    }
    (cursors, nodes_used - 1, n_rules)
}

// ---------------------------------------------------------------------------
// Arms C5 & C6: Wormhole-reclaimed edges-overlaid tiered power-of-two SoA
// (100% safe Rust, zero bounds checks via TryInto<&mut [u32; CAP]>, ZERO
// extra node/rule/image_order heap — only +16 KiB GenCacheV2!)
// ---------------------------------------------------------------------------

/// Safe u32-slice rule arena overlaid inside `edges` (grows `edges` only on cold overflow).
struct EdgesRules<'a> {
    buf: &'a mut [u32],
    top: usize, // in u32 words (always multiple of 4)
}

impl<'a> EdgesRules<'a> {
    #[inline(always)]
    fn push(&mut self, r: [u32; 4]) -> u32 {
        let id = (self.top / 4) as u32 + 257;
        let t = self.top;
        self.buf[t] = r[0];
        self.buf[t + 1] = r[1];
        self.buf[t + 2] = r[2];
        self.buf[t + 3] = r[3];
        self.top = t + 4;
        id
    }

    #[inline(always)]
    fn room(&self) -> usize {
        self.buf.len() - self.top
    }

    #[inline(always)]
    fn get_inline(&self, id: u32) -> u128 {
        let r = (id - 257) as usize * 4;
        u128::from(self.buf[r])
            | (u128::from(self.buf[r + 1]) << 32)
            | (u128::from(self.buf[r + 2]) << 64)
            | (u128::from(self.buf[r + 3] & 0x00ff_ffff) << 96)
    }

    fn append_fast(&mut self, word: u32, spill: &mut Vec<u32>, dest: &mut Vec<u8>) {
        let mut stk = [0u32; 64];
        let mut sp = 0usize;
        spill.clear();
        let mut cur = word;
        loop {
            if cur <= 256 {
                dest.push((cur - 1) as u8);
            } else {
                let r = (cur - 257) as usize * 4;
                let tag = self.buf[r + 3];
                if tag >> 28 == 0 {
                    let bytes = (u128::from(self.buf[r])
                        | (u128::from(self.buf[r + 1]) << 32)
                        | (u128::from(self.buf[r + 2]) << 64)
                        | (u128::from(tag & 0x00ff_ffff) << 96))
                        .to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                } else if tag == u32::MAX {
                    let r0 = self.buf[r];
                    let r1 = self.buf[r + 1];
                    self.buf[r + 3] = 0x8000_0000 | dest.len() as u32;
                    if sp < 64 {
                        stk[sp] = r1;
                        sp += 1;
                    } else {
                        spill.push(r1);
                    }
                    cur = r0;
                    continue;
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let len = self.buf[r + 2] as usize;
                    dest.extend_from_within(start..start + len);
                }
            }
            if let Some(next) = spill.pop() {
                cur = next;
            } else if sp > 0 {
                sp -= 1;
                cur = stk[sp];
            } else {
                break;
            }
        }
    }
}

#[inline(always)]
fn join_u64_edges(rules: &mut EdgesRules<'_>, left: u64, right: u64) -> u64 {
    let ll = (left >> 56) as u32;
    let rl = (right >> 56) as u32;
    if (ll + rl < 8) | (ll.min(rl) == 0) {
        return (left + (u64::from(rl) << 56)) | (right << (8 * (ll & 7)));
    }
    join_u64_long_edges(rules, left, right, ll, rl)
}

#[cold]
#[inline(never)]
fn join_u64_long_edges(rules: &mut EdgesRules<'_>, left: u64, right: u64, ll: u32, rl: u32) -> u64 {
    if ll < 16 && rl < 16 && ll + rl < 16 {
        let sum = ll + rl;
        let lp = if ll < 8 {
            u128::from(left & U64_PAYLOAD_MASK)
        } else {
            rules.get_inline((left & U64_ID_MASK) as u32)
        };
        let rp = if rl < 8 {
            u128::from(right & U64_PAYLOAD_MASK)
        } else {
            rules.get_inline((right & U64_ID_MASK) as u32)
        };
        let p = lp | (rp << (8 * ll)) | (u128::from(sum) << 120);
        let id = rules.push([
            p as u32,
            (p >> 32) as u32,
            (p >> 64) as u32,
            (p >> 96) as u32,
        ]);
        return (u64::from(sum) << 56) | u64::from(id);
    }
    let (lid, l_len) = if ll == 0xFF {
        (
            (left & U64_ID_MASK) as u32,
            ((left >> 24) & 0xFFFF_FFFF) as u32,
        )
    } else if ll == 1 {
        (u32::from(left as u8) + 1, 1)
    } else if ll < 8 {
        let p = left & U64_PAYLOAD_MASK;
        let id = rules.push([p as u32, (p >> 32) as u32, 0, ll << 24]);
        (id, ll)
    } else {
        ((left & U64_ID_MASK) as u32, ll)
    };
    let (rid, r_len) = if left == right {
        (lid, l_len)
    } else if rl == 0xFF {
        (
            (right & U64_ID_MASK) as u32,
            ((right >> 24) & 0xFFFF_FFFF) as u32,
        )
    } else if rl == 1 {
        (u32::from(right as u8) + 1, 1)
    } else if rl < 8 {
        let p = right & U64_PAYLOAD_MASK;
        let id = rules.push([p as u32, (p >> 32) as u32, 0, rl << 24]);
        (id, rl)
    } else {
        ((right & U64_ID_MASK) as u32, rl)
    };
    let total_len = l_len + r_len;
    let id = rules.push([lid, rid, total_len, u32::MAX]);
    U64_LONG_TAG | (u64::from(total_len) << 24) | u64::from(id)
}

fn power_u64_edges(rules: &mut EdgesRules<'_>, word: u64, q: u32) -> u64 {
    let (mut acc, mut base, mut e) = (0u64, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_u64_edges(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_u64_edges(rules, base, base);
    }
}

fn fold_left_u64_edges(rules: &mut EdgesRules<'_>, x: u64, mut q: u32, mut y: u64) -> u64 {
    if x == 0 {
        return y;
    }
    let xl = (x >> 56) as u32;
    if xl < 8 {
        while q > 0 {
            let yl = (y >> 56) as u32;
            if yl >= 8 || xl + yl >= 8 {
                break;
            }
            y = if y == 0 {
                x
            } else {
                (x + (u64::from(yl) << 56)) | (y << (8 * xl))
            };
            q -= 1;
        }
    }
    if q == 0 {
        return y;
    }
    let head = power_u64_edges(rules, x, q);
    join_u64_edges(rules, head, y)
}

fn fold_right_u64_edges(rules: &mut EdgesRules<'_>, mut x: u64, y: u64, mut q: u32) -> u64 {
    if y == 0 {
        return x;
    }
    let yl = (y >> 56) as u32;
    if yl < 8 {
        while q > 0 {
            let xl = (x >> 56) as u32;
            if xl >= 8 || xl + yl >= 8 {
                break;
            }
            x = if x == 0 {
                y
            } else {
                (x + (u64::from(yl) << 56)) | (y << (8 * xl))
            };
            q -= 1;
        }
    }
    if q == 0 {
        return x;
    }
    let tail = power_u64_edges(rules, y, q);
    join_u64_edges(rules, x, tail)
}

#[inline(always)]
fn join_u128_edges(rules: &mut EdgesRules<'_>, left: u128, right: u128) -> u128 {
    let ll = (left >> 120) as u32;
    let rl = (right >> 120) as u32;
    if (ll + rl < 16) | (ll.min(rl) == 0) {
        return (left + (u128::from(rl) << 120)) | (right << (8 * (ll & 15)));
    }
    join_u128_long_edges(rules, left, right, ll, rl)
}

#[cold]
#[inline(never)]
fn join_u128_long_edges(
    rules: &mut EdgesRules<'_>,
    left: u128,
    right: u128,
    ll: u32,
    rl: u32,
) -> u128 {
    let (lid, l_len) = if ll == 0xFF {
        (left as u32, (left >> 32) as u32)
    } else if ll == 1 {
        (u32::from(left as u8) + 1, 1)
    } else {
        let id = rules.push([
            left as u32,
            (left >> 32) as u32,
            (left >> 64) as u32,
            (left >> 96) as u32,
        ]);
        (id, ll)
    };
    let (rid, r_len) = if left == right {
        (lid, l_len)
    } else if rl == 0xFF {
        (right as u32, (right >> 32) as u32)
    } else if rl == 1 {
        (u32::from(right as u8) + 1, 1)
    } else {
        let id = rules.push([
            right as u32,
            (right >> 32) as u32,
            (right >> 64) as u32,
            (right >> 96) as u32,
        ]);
        (id, rl)
    };
    let total_len = l_len + r_len;
    let id = rules.push([lid, rid, total_len, u32::MAX]);
    U128_LONG_TAG | (u128::from(total_len) << 32) | u128::from(id)
}

fn power_u128_edges(rules: &mut EdgesRules<'_>, word: u128, q: u32) -> u128 {
    let (mut acc, mut base, mut e) = (0u128, word, q);
    loop {
        if e & 1 == 1 {
            acc = join_u128_edges(rules, acc, base);
        }
        e >>= 1;
        if e == 0 {
            break acc;
        }
        base = join_u128_edges(rules, base, base);
    }
}

fn fold_left_u128_edges(rules: &mut EdgesRules<'_>, x: u128, mut q: u32, mut y: u128) -> u128 {
    if x == 0 {
        return y;
    }
    let xl = (x >> 120) as u32;
    if xl < 16 {
        while q > 0 {
            let yl = (y >> 120) as u32;
            if yl >= 16 || xl + yl >= 16 {
                break;
            }
            y = if y == 0 {
                x
            } else {
                (x + (u128::from(yl) << 120)) | (y << (8 * xl))
            };
            q -= 1;
        }
    }
    if q == 0 {
        return y;
    }
    let head = power_u128_edges(rules, x, q);
    join_u128_edges(rules, head, y)
}

fn fold_right_u128_edges(rules: &mut EdgesRules<'_>, mut x: u128, y: u128, mut q: u32) -> u128 {
    if y == 0 {
        return x;
    }
    let yl = (y >> 120) as u32;
    if yl < 16 {
        while q > 0 {
            let xl = (x >> 120) as u32;
            if xl >= 16 || xl + yl >= 16 {
                break;
            }
            x = if x == 0 {
                y
            } else {
                (x + (u128::from(yl) << 120)) | (y << (8 * xl))
            };
            q -= 1;
        }
    }
    if q == 0 {
        return x;
    }
    let tail = power_u128_edges(rules, y, q);
    join_u128_edges(rules, x, tail)
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn dict_code_long_edges(
    id: u32,
    byte: u8,
    rules: &mut EdgesRules<'_>,
    gen_cache: &mut GenCacheV2,
    gen_prefix: u32,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
) -> u32 {
    let start = spelling.len();
    let memoized = rules.buf[(id - 257) as usize * 4 + 3] != u32::MAX;
    rules.append_fast(id, stack, spelling);
    spelling.push(byte);
    let end = spelling.len();
    let bytes = &spelling[start..end];
    let h = long_bytes_hash(bytes);
    let idx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
    let tag_word = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
    let slot = gen_cache.contents[idx];
    if (slot & 0xFFFF_0000) == tag_word {
        let old = (slot & 0xFFFF) as usize;
        if lens[old] == LONG && long_word(&inline16[old], spelling) == bytes {
            if memoized {
                spelling.truncate(start);
            } else {
                spelling.pop();
            }
            return old as u32;
        }
    }
    let code = lens.len() as u32;
    inline16.push(long_slot(start as u32, end as u32));
    lens.push(LONG);
    if code < 65_536 {
        gen_cache.contents[idx] = tag_word | code;
    }
    code
}

// Inline helpers for unpacked 4-word u32 links ([prev0, next0, prev1, next1]) inside &[u32; CAP4]
// Zero read-modify-write, zero bit-packing shifts, 1 cache line per node, zero bounds checks!
#[inline(always)]
fn lu_prev0<const CAP4: usize>(l: &[u32; CAP4], id: u32) -> u32 {
    l[((id as usize) << 2) & (CAP4 - 1)]
}
#[inline(always)]
fn lu_next0<const CAP4: usize>(l: &[u32; CAP4], id: u32) -> u32 {
    l[(((id as usize) << 2) & (CAP4 - 1)) | 1]
}
#[inline(always)]
fn lu_prev1<const CAP4: usize>(l: &[u32; CAP4], id: u32) -> u32 {
    l[(((id as usize) << 2) & (CAP4 - 1)) | 2]
}
#[inline(always)]
fn lu_next1<const CAP4: usize>(l: &[u32; CAP4], id: u32) -> u32 {
    l[(((id as usize) << 2) & (CAP4 - 1)) | 3]
}
#[inline(always)]
fn lu_set_prev0<const CAP4: usize>(l: &mut [u32; CAP4], id: u32, to: u32) {
    l[((id as usize) << 2) & (CAP4 - 1)] = to;
}
#[inline(always)]
fn lu_set_next0<const CAP4: usize>(l: &mut [u32; CAP4], id: u32, to: u32) {
    l[(((id as usize) << 2) & (CAP4 - 1)) | 1] = to;
}
#[inline(always)]
fn lu_set_prev1<const CAP4: usize>(l: &mut [u32; CAP4], id: u32, to: u32) {
    l[(((id as usize) << 2) & (CAP4 - 1)) | 2] = to;
}
#[inline(always)]
fn lu_set_next1<const CAP4: usize>(l: &mut [u32; CAP4], id: u32, to: u32) {
    l[(((id as usize) << 2) & (CAP4 - 1)) | 3] = to;
}

#[inline(always)]
fn load_u64_pair<const CAP2: usize>(w: &[u32; CAP2], idx: usize) -> u64 {
    let b = (idx << 1) & (CAP2 - 1);
    u64::from(w[b]) | (u64::from(w[b | 1]) << 32)
}
#[inline(always)]
fn store_u64_pair<const CAP2: usize>(w: &mut [u32; CAP2], idx: usize, val: u64) {
    let b = (idx << 1) & (CAP2 - 1);
    w[b] = val as u32;
    w[b | 1] = (val >> 32) as u32;
}

#[inline(always)]
fn load_u128_quad<const CAP4: usize>(w: &[u32; CAP4], idx: usize) -> u128 {
    let b = (idx << 2) & (CAP4 - 1);
    u128::from(w[b])
        | (u128::from(w[b | 1]) << 32)
        | (u128::from(w[b | 2]) << 64)
        | (u128::from(w[b | 3]) << 96)
}
#[inline(always)]
fn store_u128_quad<const CAP4: usize>(w: &mut [u32; CAP4], idx: usize, val: u128) {
    let b = (idx << 2) & (CAP4 - 1);
    w[b] = val as u32;
    w[b | 1] = (val >> 32) as u32;
    w[b | 2] = (val >> 64) as u32;
    w[b | 3] = (val >> 96) as u32;
}

/// Cache-line stagger (16 u32 = 64B) between power-of-two arrays inside `edges`
/// to eliminate 4K-aliasing / L1d set-associativity conflicts across links/widths/words!
const PAD_U32: usize = 16;

fn elim_c5_edges_u64<
    const CAP: usize,
    const CAP2: usize,
    const CAP4: usize,
    const SPIRALS: bool,
>(
    links: &mut [u32; CAP4],
    widths: &mut [u32; CAP],
    words: &mut [u32; CAP2],
    rules: &mut EdgesRules<'_>,
    left_ref: &mut u32,
) {
    let mask = CAP - 1;
    let room_need = if SPIRALS { (SPIRAL_RULES + 4) * 4 } else { 12 };
    let mut left = *left_ref;
    while left != 0 {
        if rules.room() < room_need {
            break;
        }
        let x = links[0];
        let y = links[2];
        let xi = (x as usize) & mask;
        let yi = (y as usize) & mask;
        let aw = widths[xi];
        let bw = widths[yi];
        if SPIRALS && x != y {
            if aw > bw && lu_prev1::<CAP4>(links, y) == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                assert!(q * bw < aw && q * bw <= left);
                let wx = load_u64_pair::<CAP2>(words, xi);
                let wy = load_u64_pair::<CAP2>(words, yi);
                let nw = fold_left_u64_edges(rules, wx, q, wy);
                store_u64_pair::<CAP2>(words, yi, nw);
                widths[xi] = aw - q * bw;
                left -= q * bw;
                continue;
            }
            if aw < bw && lu_prev0::<CAP4>(links, x) == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                assert!(q * aw < bw && q * aw <= left);
                let wx = load_u64_pair::<CAP2>(words, xi);
                let wy = load_u64_pair::<CAP2>(words, yi);
                let nw = fold_right_u64_edges(rules, wx, wy, q);
                store_u64_pair::<CAP2>(words, xi, nw);
                widths[yi] = bw - q * aw;
                left -= q * aw;
                continue;
            }
        }
        let w = aw.min(bw);
        assert!(w > 0 && w <= left);
        left -= w;
        if x == y {
            let xp = lu_prev0::<CAP4>(links, x);
            lu_set_next0::<CAP4>(links, xp, 0);
            links[0] = xp;
            let yp = lu_prev1::<CAP4>(links, y);
            lu_set_next1::<CAP4>(links, yp, 0);
            links[2] = yp;
            continue;
        }
        let wx = load_u64_pair::<CAP2>(words, xi);
        let wy = load_u64_pair::<CAP2>(words, yi);
        let word = join_u64_edges(rules, wx, wy);
        if aw > bw {
            let yp = lu_prev1::<CAP4>(links, y);
            lu_set_next1::<CAP4>(links, yp, 0);
            links[2] = yp;
            let xn = lu_next1::<CAP4>(links, x);
            lu_set_prev1::<CAP4>(links, y, x);
            lu_set_next1::<CAP4>(links, y, xn);
            lu_set_next1::<CAP4>(links, x, y);
            lu_set_prev1::<CAP4>(links, xn, y);
            store_u64_pair::<CAP2>(words, yi, word);
            widths[xi] = aw - bw;
        } else if aw < bw {
            let xp = lu_prev0::<CAP4>(links, x);
            lu_set_next0::<CAP4>(links, xp, 0);
            links[0] = xp;
            let yn = lu_next0::<CAP4>(links, y);
            lu_set_prev0::<CAP4>(links, x, y);
            lu_set_next0::<CAP4>(links, x, yn);
            lu_set_next0::<CAP4>(links, y, x);
            lu_set_prev0::<CAP4>(links, yn, x);
            store_u64_pair::<CAP2>(words, xi, word);
            widths[yi] = bw - aw;
        } else {
            let xp = lu_prev0::<CAP4>(links, x);
            lu_set_next0::<CAP4>(links, xp, 0);
            links[0] = xp;
            let yp0 = lu_prev0::<CAP4>(links, y);
            let yn0 = lu_next0::<CAP4>(links, y);
            lu_set_prev0::<CAP4>(links, x, yp0);
            lu_set_next0::<CAP4>(links, x, yn0);
            lu_set_next0::<CAP4>(links, yp0, x);
            lu_set_prev0::<CAP4>(links, yn0, x);
            let yp1 = lu_prev1::<CAP4>(links, y);
            lu_set_next1::<CAP4>(links, yp1, 0);
            links[2] = yp1;
            store_u64_pair::<CAP2>(words, xi, word);
        }
    }
    *left_ref = left;
}

fn elim_c6_edges_u128<const CAP: usize, const CAP4: usize, const SPIRALS: bool>(
    links: &mut [u32; CAP4],
    widths: &mut [u32; CAP],
    words: &mut [u32; CAP4],
    rules: &mut EdgesRules<'_>,
    left_ref: &mut u32,
) {
    let mask = CAP - 1;
    let room_need = if SPIRALS { (SPIRAL_RULES + 4) * 4 } else { 12 };
    let mut left = *left_ref;
    while left != 0 {
        if rules.room() < room_need {
            break;
        }
        let x = links[0];
        let y = links[2];
        let xi = (x as usize) & mask;
        let yi = (y as usize) & mask;
        let aw = widths[xi];
        let bw = widths[yi];
        if SPIRALS && x != y {
            if aw > bw && lu_prev1::<CAP4>(links, y) == x && (aw - 1) / bw >= 2 {
                let q = (aw - 1) / bw;
                assert!(q * bw < aw && q * bw <= left);
                let wx = load_u128_quad::<CAP4>(words, xi);
                let wy = load_u128_quad::<CAP4>(words, yi);
                let nw = fold_left_u128_edges(rules, wx, q, wy);
                store_u128_quad::<CAP4>(words, yi, nw);
                widths[xi] = aw - q * bw;
                left -= q * bw;
                continue;
            }
            if aw < bw && lu_prev0::<CAP4>(links, x) == y && (bw - 1) / aw >= 2 {
                let q = (bw - 1) / aw;
                assert!(q * aw < bw && q * aw <= left);
                let wx = load_u128_quad::<CAP4>(words, xi);
                let wy = load_u128_quad::<CAP4>(words, yi);
                let nw = fold_right_u128_edges(rules, wx, wy, q);
                store_u128_quad::<CAP4>(words, xi, nw);
                widths[yi] = bw - q * aw;
                left -= q * aw;
                continue;
            }
        }
        let w = aw.min(bw);
        assert!(w > 0 && w <= left);
        left -= w;
        if x == y {
            let xp = lu_prev0::<CAP4>(links, x);
            lu_set_next0::<CAP4>(links, xp, 0);
            links[0] = xp;
            let yp = lu_prev1::<CAP4>(links, y);
            lu_set_next1::<CAP4>(links, yp, 0);
            links[2] = yp;
            continue;
        }
        let wx = load_u128_quad::<CAP4>(words, xi);
        let wy = load_u128_quad::<CAP4>(words, yi);
        let word = join_u128_edges(rules, wx, wy);
        if aw > bw {
            let yp = lu_prev1::<CAP4>(links, y);
            lu_set_next1::<CAP4>(links, yp, 0);
            links[2] = yp;
            let xn = lu_next1::<CAP4>(links, x);
            lu_set_prev1::<CAP4>(links, y, x);
            lu_set_next1::<CAP4>(links, y, xn);
            lu_set_next1::<CAP4>(links, x, y);
            lu_set_prev1::<CAP4>(links, xn, y);
            store_u128_quad::<CAP4>(words, yi, word);
            widths[xi] = aw - bw;
        } else if aw < bw {
            let xp = lu_prev0::<CAP4>(links, x);
            lu_set_next0::<CAP4>(links, xp, 0);
            links[0] = xp;
            let yn = lu_next0::<CAP4>(links, y);
            lu_set_prev0::<CAP4>(links, x, y);
            lu_set_next0::<CAP4>(links, x, yn);
            lu_set_next0::<CAP4>(links, y, x);
            lu_set_prev0::<CAP4>(links, yn, x);
            store_u128_quad::<CAP4>(words, xi, word);
            widths[yi] = bw - aw;
        } else {
            let xp = lu_prev0::<CAP4>(links, x);
            lu_set_next0::<CAP4>(links, xp, 0);
            links[0] = xp;
            let yp0 = lu_prev0::<CAP4>(links, y);
            let yn0 = lu_next0::<CAP4>(links, y);
            lu_set_prev0::<CAP4>(links, x, yp0);
            lu_set_next0::<CAP4>(links, x, yn0);
            lu_set_next0::<CAP4>(links, yp0, x);
            lu_set_prev0::<CAP4>(links, yn0, x);
            let yp1 = lu_prev1::<CAP4>(links, y);
            lu_set_next1::<CAP4>(links, yp1, 0);
            links[2] = yp1;
            store_u128_quad::<CAP4>(words, xi, word);
        }
    }
    *left_ref = left;
}

fn assign_bottom_edges_unpacked<const CAP: usize, const CAP4: usize>(
    links: &mut [u32; CAP4],
    widths: &[u32; CAP],
    rows: u32,
) {
    let mask = CAP - 1;
    let mut first = links[3]; // next1 of 0
    let mut last = links[2]; // prev1 of 0
    let (mut lo, mut hi) = (0u32, rows);
    while first != last {
        let fi = (first as usize) & mask;
        let li = (last as usize) & mask;
        let next = lu_next1::<CAP4>(links, first);
        let prev = lu_prev1::<CAP4>(links, last);
        let wf = widths[fi];
        hi -= widths[li];
        lu_set_prev1::<CAP4>(links, first, lo);
        lu_set_prev1::<CAP4>(links, last, hi);
        lo += wf;
        if next == last {
            assert_eq!(lo, hi);
            return;
        }
        first = next;
        last = prev;
    }
    if first != 0 {
        let fi = (first as usize) & mask;
        lu_set_prev1::<CAP4>(links, first, lo);
        lo += widths[fi];
    }
    assert_eq!(lo, hi);
}

#[allow(clippy::too_many_arguments)]
fn run_c5_edges_u64<const CAP: usize, const CAP2: usize, const CAP4: usize>(
    edges: &mut Vec<u32>,
    bands: &mut Vec<Band>,
    spelling: &mut Vec<u8>,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    gen_cache: &mut GenCacheV2,
    ns: usize,
    n: u32,
    m: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    thin: &[u32; 256],
    force_spirals: Option<bool>,
    ph: Option<&mut PhaseNs>,
) -> ([usize; 256], usize, usize) {
    let t0 = ph.as_ref().map(|_| Instant::now());
    let hdr_words = 5 * ns;
    let soa_words = CAP4 + PAD_U32 + CAP + PAD_U32 + CAP2 + PAD_U32;
    let rules_base = hdr_words + soa_words;
    let init_rules_words = 4 * (ns + 256);
    let need = (rules_base + init_rules_words).max(m as usize);
    if edges.len() < need {
        edges.resize(need, 0);
    }

    let (widest, nodes_used) = {
        let (hdr, rest) = edges.split_at_mut(hdr_words);
        let (strips_raw, img_u32) = hdr.split_at_mut(3 * ns);
        let (l_s, rest) = rest.split_at_mut(CAP4 + PAD_U32);
        let (wi_s, rest) = rest.split_at_mut(CAP + PAD_U32);
        let (wo_s, _) = rest.split_at_mut(CAP2 + PAD_U32);
        let links: &mut [u32; CAP4] = (&mut l_s[..CAP4]).try_into().unwrap();
        let widths: &mut [u32; CAP] = (&mut wi_s[..CAP]).try_into().unwrap();
        let words: &mut [u32; CAP2] = (&mut wo_s[..CAP2]).try_into().unwrap();

        let mask = CAP - 1;
        links[0] = 0;
        links[1] = 0;
        links[2] = 0;
        links[3] = 0;
        widths[0] = 0;
        store_u64_pair::<CAP2>(words, 0, 0);

        let mut nodes = (1 + ns) as u32;
        let mut widest = 0u32;
        let mut ord0_tail = 0u32;
        let (mut term1_head, mut term1_tail) = (0u32, 0u32);
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
            let sid = (i + 1) as u32;
            let si = (sid as usize) & mask;
            widths[si] = width;
            store_u64_pair::<CAP2>(words, si, literal_u64(byte));
            widest = widest.max(width);
            let pos = write[a] as usize;
            write[a] = (pos + 1) as u32;
            let packed = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);
            img_u32[2 * pos] = packed as u32;
            img_u32[2 * pos + 1] = (packed >> 32) as u32;
            if x < from {
                let gap = from - x;
                widest = widest.max(gap);
                let gid = nodes;
                nodes += 1;
                let gi = (gid as usize) & mask;
                widths[gi] = gap;
                store_u64_pair::<CAP2>(words, gi, 0);
                lu_set_next0::<CAP4>(links, ord0_tail, gid);
                lu_set_prev0::<CAP4>(links, gid, ord0_tail);
                ord0_tail = gid;
                if term1_tail == 0 {
                    term1_head = gid;
                } else {
                    lu_set_next1::<CAP4>(links, term1_tail, gid);
                }
                lu_set_prev1::<CAP4>(links, gid, term1_tail);
                term1_tail = gid;
            }
            lu_set_next0::<CAP4>(links, ord0_tail, sid);
            lu_set_prev0::<CAP4>(links, sid, ord0_tail);
            ord0_tail = sid;
            x = from + width;
        }
        if x < n {
            let gap = n - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            let gi = (gid as usize) & mask;
            widths[gi] = gap;
            store_u64_pair::<CAP2>(words, gi, 0);
            lu_set_next0::<CAP4>(links, ord0_tail, gid);
            lu_set_prev0::<CAP4>(links, gid, ord0_tail);
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                lu_set_next1::<CAP4>(links, term1_tail, gid);
            }
            lu_set_prev1::<CAP4>(links, gid, term1_tail);
            term1_tail = gid;
        }
        let ord0_head = if ord0_tail == 0 { 0 } else { links[1] };
        let mut ord1_tail = 0u32;
        let (mut term0_head, mut term0_tail) = (0u32, 0u32);
        x = 0;
        for pos in 0..ns {
            let packed = u64::from(img_u32[2 * pos]) | (u64::from(img_u32[2 * pos + 1]) << 32);
            let to = (packed & 0xF_FFFF) as u32;
            let width = ((packed >> 20) & 0xF_FFFF) as u32;
            let sid = (packed >> 40) as u32;
            if x < to {
                let gap = to - x;
                widest = widest.max(gap);
                let gid = nodes;
                nodes += 1;
                let gi = (gid as usize) & mask;
                widths[gi] = gap;
                store_u64_pair::<CAP2>(words, gi, 0);
                lu_set_next1::<CAP4>(links, ord1_tail, gid);
                lu_set_prev1::<CAP4>(links, gid, ord1_tail);
                ord1_tail = gid;
                if term0_tail == 0 {
                    term0_head = gid;
                } else {
                    lu_set_next0::<CAP4>(links, term0_tail, gid);
                }
                lu_set_prev0::<CAP4>(links, gid, term0_tail);
                term0_tail = gid;
            }
            lu_set_next1::<CAP4>(links, ord1_tail, sid);
            lu_set_prev1::<CAP4>(links, sid, ord1_tail);
            ord1_tail = sid;
            x = to + width;
        }
        if x < n {
            let gap = n - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            let gi = (gid as usize) & mask;
            widths[gi] = gap;
            store_u64_pair::<CAP2>(words, gi, 0);
            lu_set_next1::<CAP4>(links, ord1_tail, gid);
            lu_set_prev1::<CAP4>(links, gid, ord1_tail);
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                lu_set_next0::<CAP4>(links, term0_tail, gid);
            }
            lu_set_prev0::<CAP4>(links, gid, term0_tail);
            term0_tail = gid;
        }
        let ord1_head = if ord1_tail == 0 { 0 } else { links[3] };
        let mut prev = 0u32;
        for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
            if t == 0 {
                continue;
            }
            lu_set_next0::<CAP4>(links, prev, h);
            lu_set_prev0::<CAP4>(links, h, prev);
            prev = t;
        }
        lu_set_next0::<CAP4>(links, prev, 0);
        links[0] = prev;

        prev = 0u32;
        for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
            if t == 0 {
                continue;
            }
            lu_set_next1::<CAP4>(links, prev, h);
            lu_set_prev1::<CAP4>(links, h, prev);
            prev = t;
        }
        lu_set_next1::<CAP4>(links, prev, 0);
        links[2] = prev;
        (widest, nodes as usize)
    };
    let t1 = t0.map(|_| Instant::now());

    let spirals = force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
    let room_need = if spirals { (SPIRAL_RULES + 4) * 4 } else { 12 };
    let mut left = n;
    let mut rules_top = 0usize;
    while left != 0 {
        let (_, rest) = edges.split_at_mut(hdr_words);
        let (l_s, rest) = rest.split_at_mut(CAP4 + PAD_U32);
        let (wi_s, rest) = rest.split_at_mut(CAP + PAD_U32);
        let (wo_s, rules_buf) = rest.split_at_mut(CAP2 + PAD_U32);
        let links: &mut [u32; CAP4] = (&mut l_s[..CAP4]).try_into().unwrap();
        let widths: &mut [u32; CAP] = (&mut wi_s[..CAP]).try_into().unwrap();
        let words: &mut [u32; CAP2] = (&mut wo_s[..CAP2]).try_into().unwrap();
        let mut rules = EdgesRules {
            buf: rules_buf,
            top: rules_top,
        };
        if spirals {
            elim_c5_edges_u64::<CAP, CAP2, CAP4, true>(links, widths, words, &mut rules, &mut left);
        } else {
            elim_c5_edges_u64::<CAP, CAP2, CAP4, false>(
                links, widths, words, &mut rules, &mut left,
            );
        }
        rules_top = rules.top;
        if left != 0 {
            grow_u32(edges, rules_base + rules_top + room_need);
        }
    }
    let n_rules = rules_top / 4;
    let t2 = t0.map(|_| Instant::now());

    let (_, rest) = edges.split_at_mut(hdr_words);
    let (l_s, rest) = rest.split_at_mut(CAP4 + PAD_U32);
    let (wi_s, rest) = rest.split_at_mut(CAP + PAD_U32);
    let (wo_s, rules_buf) = rest.split_at_mut(CAP2 + PAD_U32);
    let links: &mut [u32; CAP4] = (&mut l_s[..CAP4]).try_into().unwrap();
    let widths: &[u32; CAP] = (&wi_s[..CAP]).try_into().unwrap();
    let words: &[u32; CAP2] = (&wo_s[..CAP2]).try_into().unwrap();

    assign_bottom_edges_unpacked::<CAP, CAP4>(links, widths, m);
    let t3 = t0.map(|_| Instant::now());

    let mask = CAP - 1;
    let mut rules = EdgesRules {
        buf: rules_buf,
        top: rules_top,
    };
    gen_cache.next_block();
    let gen_prefix = gen_cache.gen8 << 24;
    begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
    let mut cursors = [0usize; 256];
    let mut id = links[1]; // next0 of 0
    let mut idx = (id as usize) & mask;
    let mut node_width = widths[idx];
    let mut node_word = load_u64_pair::<CAP2>(words, idx);
    let mut node_bottom = lu_prev1::<CAP4>(links, id);
    let mut node_next = lu_next0::<CAP4>(links, id);
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        cursors[a] = bands.len();
        let end = k + thin[a];
        while k < end {
            let width = (end - k).min(node_width - in_node);
            assert!(width > 0);
            let dst = node_bottom + in_node;
            let code = if node_word == 0 {
                a as u32
            } else {
                let tag = (node_word >> 56) as u32;
                if tag < 8 {
                    let low = (node_word & U64_PAYLOAD_MASK) | (u64::from(a as u8) << (8 * tag));
                    let len = (tag + 1) as u8;
                    let h = short_u64_hash(low, len);
                    let cidx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
                    let tag_word = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
                    let slot = gen_cache.contents[cidx];
                    if (slot & 0xFFFF_0000) == tag_word {
                        let old = (slot & 0xFFFF) as usize;
                        if lens[old] == len
                            && u64::from_le_bytes(inline16[old][..8].try_into().unwrap()) == low
                        {
                            old as u32
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(u128::from(low).to_le_bytes());
                            lens.push(len);
                            if code < 65_536 {
                                gen_cache.contents[cidx] = tag_word | code;
                            }
                            code
                        }
                    } else {
                        let code = lens.len() as u32;
                        inline16.push(u128::from(low).to_le_bytes());
                        lens.push(len);
                        if code < 65_536 {
                            gen_cache.contents[cidx] = tag_word | code;
                        }
                        code
                    }
                } else if tag < 16 {
                    let payload = rules.get_inline((node_word & U64_ID_MASK) as u32);
                    let entry_u128 = payload | (u128::from(a as u8) << (8 * tag));
                    let len = (tag + 1) as u8;
                    let h = short_u128_hash(entry_u128, len);
                    let cidx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
                    let tag_word = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
                    let slot = gen_cache.contents[cidx];
                    if (slot & 0xFFFF_0000) == tag_word {
                        let old = (slot & 0xFFFF) as usize;
                        if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                            old as u32
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(entry_u128.to_le_bytes());
                            lens.push(len);
                            if code < 65_536 {
                                gen_cache.contents[cidx] = tag_word | code;
                            }
                            code
                        }
                    } else {
                        let code = lens.len() as u32;
                        inline16.push(entry_u128.to_le_bytes());
                        lens.push(len);
                        if code < 65_536 {
                            gen_cache.contents[cidx] = tag_word | code;
                        }
                        code
                    }
                } else {
                    dict_code_long_edges(
                        (node_word & U64_ID_MASK) as u32,
                        a as u8,
                        &mut rules,
                        gen_cache,
                        gen_prefix,
                        inline16,
                        lens,
                        spelling,
                        stack,
                    )
                }
            };
            bands.push(Band::new(k + width, dst, code));
            in_node += width;
            k += width;
            if in_node == node_width {
                id = node_next;
                in_node = 0;
                if id != 0 {
                    idx = (id as usize) & mask;
                    node_width = widths[idx];
                    node_word = load_u64_pair::<CAP2>(words, idx);
                    node_bottom = lu_prev1::<CAP4>(links, id);
                    node_next = lu_next0::<CAP4>(links, id);
                }
            }
        }
    }
    assert_eq!(id, 0);
    if let Some(p) = ph {
        let t4 = Instant::now();
        p.setup += t1.unwrap().duration_since(t0.unwrap()).as_nanos() as u64;
        p.steps += t2.unwrap().duration_since(t1.unwrap()).as_nanos() as u64;
        p.bot += t3.unwrap().duration_since(t2.unwrap()).as_nanos() as u64;
        p.dict += t4.duration_since(t3.unwrap()).as_nanos() as u64;
    }
    (cursors, nodes_used - 1, n_rules)
}

#[allow(clippy::too_many_arguments)]
fn run_c6_edges_u128<const CAP: usize, const CAP4: usize>(
    edges: &mut Vec<u32>,
    bands: &mut Vec<Band>,
    spelling: &mut Vec<u8>,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    gen_cache: &mut GenCacheV2,
    ns: usize,
    n: u32,
    m: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    thin: &[u32; 256],
    force_spirals: Option<bool>,
    ph: Option<&mut PhaseNs>,
) -> ([usize; 256], usize, usize) {
    let t0 = ph.as_ref().map(|_| Instant::now());
    let hdr_words = 5 * ns;
    let soa_words = CAP4 + PAD_U32 + CAP + PAD_U32 + CAP4 + PAD_U32;
    let rules_base = hdr_words + soa_words;
    let init_rules_words = 4 * (ns + 256);
    let need = (rules_base + init_rules_words).max(m as usize);
    if edges.len() < need {
        edges.resize(need, 0);
    }

    let (widest, nodes_used) = {
        let (hdr, rest) = edges.split_at_mut(hdr_words);
        let (strips_raw, img_u32) = hdr.split_at_mut(3 * ns);
        let (l_s, rest) = rest.split_at_mut(CAP4 + PAD_U32);
        let (wi_s, rest) = rest.split_at_mut(CAP + PAD_U32);
        let (wo_s, _) = rest.split_at_mut(CAP4 + PAD_U32);
        let links: &mut [u32; CAP4] = (&mut l_s[..CAP4]).try_into().unwrap();
        let widths: &mut [u32; CAP] = (&mut wi_s[..CAP]).try_into().unwrap();
        let words: &mut [u32; CAP4] = (&mut wo_s[..CAP4]).try_into().unwrap();

        let mask = CAP - 1;
        links[0] = 0;
        links[1] = 0;
        links[2] = 0;
        links[3] = 0;
        widths[0] = 0;
        store_u128_quad::<CAP4>(words, 0, 0);

        let mut nodes = (1 + ns) as u32;
        let mut widest = 0u32;
        let mut ord0_tail = 0u32;
        let (mut term1_head, mut term1_tail) = (0u32, 0u32);
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
            let sid = (i + 1) as u32;
            let si = (sid as usize) & mask;
            widths[si] = width;
            store_u128_quad::<CAP4>(words, si, literal_u128(byte));
            widest = widest.max(width);
            let pos = write[a] as usize;
            write[a] = (pos + 1) as u32;
            let packed = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);
            img_u32[2 * pos] = packed as u32;
            img_u32[2 * pos + 1] = (packed >> 32) as u32;
            if x < from {
                let gap = from - x;
                widest = widest.max(gap);
                let gid = nodes;
                nodes += 1;
                let gi = (gid as usize) & mask;
                widths[gi] = gap;
                store_u128_quad::<CAP4>(words, gi, 0);
                lu_set_next0::<CAP4>(links, ord0_tail, gid);
                lu_set_prev0::<CAP4>(links, gid, ord0_tail);
                ord0_tail = gid;
                if term1_tail == 0 {
                    term1_head = gid;
                } else {
                    lu_set_next1::<CAP4>(links, term1_tail, gid);
                }
                lu_set_prev1::<CAP4>(links, gid, term1_tail);
                term1_tail = gid;
            }
            lu_set_next0::<CAP4>(links, ord0_tail, sid);
            lu_set_prev0::<CAP4>(links, sid, ord0_tail);
            ord0_tail = sid;
            x = from + width;
        }
        if x < n {
            let gap = n - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            let gi = (gid as usize) & mask;
            widths[gi] = gap;
            store_u128_quad::<CAP4>(words, gi, 0);
            lu_set_next0::<CAP4>(links, ord0_tail, gid);
            lu_set_prev0::<CAP4>(links, gid, ord0_tail);
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                lu_set_next1::<CAP4>(links, term1_tail, gid);
            }
            lu_set_prev1::<CAP4>(links, gid, term1_tail);
            term1_tail = gid;
        }
        let ord0_head = if ord0_tail == 0 { 0 } else { links[1] };
        let mut ord1_tail = 0u32;
        let (mut term0_head, mut term0_tail) = (0u32, 0u32);
        x = 0;
        for pos in 0..ns {
            let packed = u64::from(img_u32[2 * pos]) | (u64::from(img_u32[2 * pos + 1]) << 32);
            let to = (packed & 0xF_FFFF) as u32;
            let width = ((packed >> 20) & 0xF_FFFF) as u32;
            let sid = (packed >> 40) as u32;
            if x < to {
                let gap = to - x;
                widest = widest.max(gap);
                let gid = nodes;
                nodes += 1;
                let gi = (gid as usize) & mask;
                widths[gi] = gap;
                store_u128_quad::<CAP4>(words, gi, 0);
                lu_set_next1::<CAP4>(links, ord1_tail, gid);
                lu_set_prev1::<CAP4>(links, gid, ord1_tail);
                ord1_tail = gid;
                if term0_tail == 0 {
                    term0_head = gid;
                } else {
                    lu_set_next0::<CAP4>(links, term0_tail, gid);
                }
                lu_set_prev0::<CAP4>(links, gid, term0_tail);
                term0_tail = gid;
            }
            lu_set_next1::<CAP4>(links, ord1_tail, sid);
            lu_set_prev1::<CAP4>(links, sid, ord1_tail);
            ord1_tail = sid;
            x = to + width;
        }
        if x < n {
            let gap = n - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            let gi = (gid as usize) & mask;
            widths[gi] = gap;
            store_u128_quad::<CAP4>(words, gi, 0);
            lu_set_next1::<CAP4>(links, ord1_tail, gid);
            lu_set_prev1::<CAP4>(links, gid, ord1_tail);
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                lu_set_next0::<CAP4>(links, term0_tail, gid);
            }
            lu_set_prev0::<CAP4>(links, gid, term0_tail);
            term0_tail = gid;
        }
        let ord1_head = if ord1_tail == 0 { 0 } else { links[3] };
        let mut prev = 0u32;
        for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
            if t == 0 {
                continue;
            }
            lu_set_next0::<CAP4>(links, prev, h);
            lu_set_prev0::<CAP4>(links, h, prev);
            prev = t;
        }
        lu_set_next0::<CAP4>(links, prev, 0);
        links[0] = prev;

        prev = 0u32;
        for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
            if t == 0 {
                continue;
            }
            lu_set_next1::<CAP4>(links, prev, h);
            lu_set_prev1::<CAP4>(links, h, prev);
            prev = t;
        }
        lu_set_next1::<CAP4>(links, prev, 0);
        links[2] = prev;
        (widest, nodes as usize)
    };
    let t1 = t0.map(|_| Instant::now());

    let spirals = force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
    let room_need = if spirals { (SPIRAL_RULES + 4) * 4 } else { 12 };
    let mut left = n;
    let mut rules_top = 0usize;
    while left != 0 {
        let (_, rest) = edges.split_at_mut(hdr_words);
        let (l_s, rest) = rest.split_at_mut(CAP4 + PAD_U32);
        let (wi_s, rest) = rest.split_at_mut(CAP + PAD_U32);
        let (wo_s, rules_buf) = rest.split_at_mut(CAP4 + PAD_U32);
        let links: &mut [u32; CAP4] = (&mut l_s[..CAP4]).try_into().unwrap();
        let widths: &mut [u32; CAP] = (&mut wi_s[..CAP]).try_into().unwrap();
        let words: &mut [u32; CAP4] = (&mut wo_s[..CAP4]).try_into().unwrap();
        let mut rules = EdgesRules {
            buf: rules_buf,
            top: rules_top,
        };
        if spirals {
            elim_c6_edges_u128::<CAP, CAP4, true>(links, widths, words, &mut rules, &mut left);
        } else {
            elim_c6_edges_u128::<CAP, CAP4, false>(links, widths, words, &mut rules, &mut left);
        }
        rules_top = rules.top;
        if left != 0 {
            grow_u32(edges, rules_base + rules_top + room_need);
        }
    }
    let n_rules = rules_top / 4;
    let t2 = t0.map(|_| Instant::now());

    let (_, rest) = edges.split_at_mut(hdr_words);
    let (l_s, rest) = rest.split_at_mut(CAP4 + PAD_U32);
    let (wi_s, rest) = rest.split_at_mut(CAP + PAD_U32);
    let (wo_s, rules_buf) = rest.split_at_mut(CAP4 + PAD_U32);
    let links: &mut [u32; CAP4] = (&mut l_s[..CAP4]).try_into().unwrap();
    let widths: &[u32; CAP] = (&wi_s[..CAP]).try_into().unwrap();
    let words: &[u32; CAP4] = (&wo_s[..CAP4]).try_into().unwrap();

    assign_bottom_edges_unpacked::<CAP, CAP4>(links, widths, m);
    let t3 = t0.map(|_| Instant::now());

    let mask = CAP - 1;
    let mut rules = EdgesRules {
        buf: rules_buf,
        top: rules_top,
    };
    gen_cache.next_block();
    let gen_prefix = gen_cache.gen8 << 24;
    begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
    let mut cursors = [0usize; 256];
    let mut id = links[1];
    let mut idx = (id as usize) & mask;
    let mut node_width = widths[idx];
    let mut node_word = load_u128_quad::<CAP4>(words, idx);
    let mut node_bottom = lu_prev1::<CAP4>(links, id);
    let mut node_next = lu_next0::<CAP4>(links, id);
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        cursors[a] = bands.len();
        let end = k + thin[a];
        while k < end {
            let width = (end - k).min(node_width - in_node);
            assert!(width > 0);
            let dst = node_bottom + in_node;
            let code = if node_word == 0 {
                a as u32
            } else {
                let tag = (node_word >> 120) as u32;
                if tag < 8 {
                    let low = (node_word as u64) | (u64::from(a as u8) << (8 * tag));
                    let len = (tag + 1) as u8;
                    let h = short_u64_hash(low, len);
                    let cidx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
                    let tag_word = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
                    let slot = gen_cache.contents[cidx];
                    if (slot & 0xFFFF_0000) == tag_word {
                        let old = (slot & 0xFFFF) as usize;
                        if lens[old] == len
                            && u64::from_le_bytes(inline16[old][..8].try_into().unwrap()) == low
                        {
                            old as u32
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(u128::from(low).to_le_bytes());
                            lens.push(len);
                            if code < 65_536 {
                                gen_cache.contents[cidx] = tag_word | code;
                            }
                            code
                        }
                    } else {
                        let code = lens.len() as u32;
                        inline16.push(u128::from(low).to_le_bytes());
                        lens.push(len);
                        if code < 65_536 {
                            gen_cache.contents[cidx] = tag_word | code;
                        }
                        code
                    }
                } else if tag < 16 {
                    let entry_u128 =
                        (node_word & U128_PAYLOAD_MASK) | (u128::from(a as u8) << (8 * tag));
                    let len = (tag + 1) as u8;
                    let h = short_u128_hash(entry_u128, len);
                    let cidx = (h as usize) & (GEN_CONTENT_SLOTS - 1);
                    let tag_word = gen_prefix | (((h >> 12) as u32 & 0xFF) << 16);
                    let slot = gen_cache.contents[cidx];
                    if (slot & 0xFFFF_0000) == tag_word {
                        let old = (slot & 0xFFFF) as usize;
                        if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                            old as u32
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(entry_u128.to_le_bytes());
                            lens.push(len);
                            if code < 65_536 {
                                gen_cache.contents[cidx] = tag_word | code;
                            }
                            code
                        }
                    } else {
                        let code = lens.len() as u32;
                        inline16.push(entry_u128.to_le_bytes());
                        lens.push(len);
                        if code < 65_536 {
                            gen_cache.contents[cidx] = tag_word | code;
                        }
                        code
                    }
                } else {
                    dict_code_long_edges(
                        node_word as u32,
                        a as u8,
                        &mut rules,
                        gen_cache,
                        gen_prefix,
                        inline16,
                        lens,
                        spelling,
                        stack,
                    )
                }
            };
            bands.push(Band::new(k + width, dst, code));
            in_node += width;
            k += width;
            if in_node == node_width {
                id = node_next;
                in_node = 0;
                if id != 0 {
                    idx = (id as usize) & mask;
                    node_width = widths[idx];
                    node_word = load_u128_quad::<CAP4>(words, idx);
                    node_bottom = lu_prev1::<CAP4>(links, id);
                    node_next = lu_next0::<CAP4>(links, id);
                }
            }
        }
    }
    assert_eq!(id, 0);
    if let Some(p) = ph {
        let t4 = Instant::now();
        p.setup += t1.unwrap().duration_since(t0.unwrap()).as_nanos() as u64;
        p.steps += t2.unwrap().duration_since(t1.unwrap()).as_nanos() as u64;
        p.bot += t3.unwrap().duration_since(t2.unwrap()).as_nanos() as u64;
        p.dict += t4.duration_since(t3.unwrap()).as_nanos() as u64;
    }
    (cursors, nodes_used - 1, n_rules)
}

// ---------------------------------------------------------------------------
// Unified A6a + A6b + A6c + A7 + A9 Worker Pipeline
// ---------------------------------------------------------------------------

struct BlockData {
    symbols: Symbols,
    records: Vec<u16>,
    n: usize,
    p: u32,
    crc: u32,
    admitted: bool,
    // Pre-built A6a snapshot for Part 1 isolated A6b benchmarking
    strips_raw: Vec<u32>,
    image_order: Vec<u32>,
    ns: usize,
    m: u32,
    first: [u32; 256],
    write_init: [u32; 256],
    thin: [u32; 256],
}

#[allow(clippy::type_complexity)]
fn build_a6a_snapshot(
    records: &[u16],
    n: usize,
    p: u32,
) -> (
    Vec<u32>,
    Vec<u32>,
    usize,
    u32,
    [u32; 256],
    [u32; 256],
    [u32; 256],
) {
    let mut all = [0u32; 256];
    let mut thin = [0u32; 256];
    let mut thick = [0u32; 256];
    let mut ns = 0usize;
    let mut m = 0u32;
    let mut strips_raw: Vec<u32> = Vec::new();
    maximal(records, |c, pos, width| {
        split(c, pos, width, p, PROD_LIMIT, |c, pos, w, keep| {
            let a = usize::from(c);
            let to_rel = all[a];
            all[a] += w;
            if keep {
                thin[a] += w;
                m += w;
            } else {
                thick[a] += 1;
                strips_raw.extend_from_slice(&[pos, to_rel, (w << 8) | u32::from(c)]);
                ns += 1;
            }
        });
    });
    let first = starts(all);
    let write_init = starts(thick);
    let mut write = write_init;
    let mut image_order = vec![0u32; ns];
    for i in 0..ns {
        let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
        let a = usize::from(s.byte);
        let pos = write[a] as usize;
        write[a] = (pos + 1) as u32;
        image_order[pos] = i as u32;
    }
    let _ = n;
    (strips_raw, image_order, ns, m, first, write_init, thin)
}

#[allow(clippy::too_many_arguments)]
fn dispatch_a6b(
    w: &mut WorkerState,
    ns: usize,
    n: u32,
    m: u32,
    first: &[u32; 256],
    write: &mut [u32; 256],
    thin: &[u32; 256],
    force_spirals: Option<bool>,
    ph: Option<&mut PhaseNs>,
) -> ([usize; 256], usize, usize) {
    w.bands.clear();
    w.spelling.clear();
    if n >= 65_536 {
        w.spelling.reserve_exact(48 * 1024);
    }
    let most = 3 * ns + 3;
    let is_wide = most > NARROW_NODES;
    if is_wide {
        // Fallback to Wide (identical across all arms; requires strips_raw+image_order in edges[0..4*ns])
        let mut wr = *write;
        let need = (4 * ns + (Wide::STRIDE + RULE) * most).max(m as usize);
        if w.edges.len() < need {
            w.edges.resize(need, 0);
        }
        // Unpack relative `to` -> absolute `to` and build image_order in edges[3*ns..4*ns]
        if w.endpoint.arm() != Arm::S0Default {
            for i in 0..ns {
                let s = Strip::unpack(&w.edges[3 * i..3 * i + 3]);
                let a = usize::from(s.byte);
                w.edges[3 * i + 1] = s.to + first[a];
                let pos = wr[a] as usize;
                wr[a] = (pos + 1) as u32;
                w.edges[3 * ns + pos] = i as u32;
            }
        }
        let stack = match &mut w.endpoint {
            EndpointScratch::S0Default { stack }
            | EndpointScratch::S1Redesign1053 { stack, .. }
            | EndpointScratch::C1VecSoaV2 { stack, .. }
            | EndpointScratch::C2TieredSlabV2 { stack, .. }
            | EndpointScratch::C3CompactWord8Soa { stack, .. }
            | EndpointScratch::C4FusedSetupOnShipped { stack, .. }
            | EndpointScratch::C5EdgesOverlaySoaU64 { stack, .. }
            | EndpointScratch::C6EdgesOverlaySoaU128 { stack, .. } => stack,
        };
        return induced_shipped::<Wide>(
            &mut w.edges,
            &mut w.bands,
            &mut w.spelling,
            &mut w.inline16,
            &mut w.lens,
            stack,
            ns,
            n,
            m,
            thin,
            force_spirals,
            ph,
        );
    }

    match &mut w.endpoint {
        EndpointScratch::S0Default { stack } => induced_shipped::<Narrow>(
            &mut w.edges,
            &mut w.bands,
            &mut w.spelling,
            &mut w.inline16,
            &mut w.lens,
            stack,
            ns,
            n,
            m,
            thin,
            force_spirals,
            ph,
        ),
        EndpointScratch::S1Redesign1053 {
            stack,
            image_order64,
            fixed_soa,
            rules,
            gen_cache,
        } => induced_s1_1053(
            &w.edges[..3 * ns],
            &mut w.bands,
            &mut w.spelling,
            &mut w.inline16,
            &mut w.lens,
            stack,
            image_order64,
            fixed_soa,
            rules,
            gen_cache,
            ns,
            n,
            m,
            first,
            write,
            thin,
            force_spirals,
            ph,
        ),
        EndpointScratch::C1VecSoaV2 {
            stack,
            image_order64,
            links,
            widths,
            words128,
            rules,
            gen_cache,
        } => induced_c1_vec_soa_v2(
            &w.edges[..3 * ns],
            &mut w.bands,
            &mut w.spelling,
            &mut w.inline16,
            &mut w.lens,
            stack,
            image_order64,
            links,
            widths,
            words128,
            rules,
            gen_cache,
            ns,
            n,
            m,
            first,
            write,
            thin,
            force_spirals,
            ph,
        ),
        EndpointScratch::C2TieredSlabV2 {
            stack,
            image_order64,
            slab,
            rules,
            gen_cache,
        } => {
            if image_order64.len() < ns {
                image_order64.resize(ns, 0);
            }
            slab.ensure(most);
            let img = &mut image_order64[..ns];
            match slab {
                TieredSoaU128::K8 {
                    links,
                    widths,
                    words,
                } => run_soa_u128_fixed_v2::<8192>(
                    &w.edges[..3 * ns],
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    img,
                    links,
                    widths,
                    words,
                    rules,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                ),
                TieredSoaU128::K16 {
                    links,
                    widths,
                    words,
                } => run_soa_u128_fixed_v2::<16384>(
                    &w.edges[..3 * ns],
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    img,
                    links,
                    widths,
                    words,
                    rules,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                ),
                TieredSoaU128::K32 {
                    links,
                    widths,
                    words,
                } => run_soa_u128_fixed_v2::<32768>(
                    &w.edges[..3 * ns],
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    img,
                    links,
                    widths,
                    words,
                    rules,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                ),
                TieredSoaU128::K64 {
                    links,
                    widths,
                    words,
                } => run_soa_u128_fixed_v2::<65536>(
                    &w.edges[..3 * ns],
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    img,
                    links,
                    widths,
                    words,
                    rules,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                ),
                TieredSoaU128::None => unreachable!(),
            }
        }
        EndpointScratch::C3CompactWord8Soa {
            stack,
            image_order64,
            slab,
            rules,
            gen_cache,
        } => {
            if image_order64.len() < ns {
                image_order64.resize(ns, 0);
            }
            slab.ensure(most);
            let img = &mut image_order64[..ns];
            match slab {
                TieredSoaU64::K8 {
                    links,
                    widths,
                    words,
                } => run_soa_u64_fixed_v2::<8192>(
                    &w.edges[..3 * ns],
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    img,
                    links,
                    widths,
                    words,
                    rules,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                ),
                TieredSoaU64::K16 {
                    links,
                    widths,
                    words,
                } => run_soa_u64_fixed_v2::<16384>(
                    &w.edges[..3 * ns],
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    img,
                    links,
                    widths,
                    words,
                    rules,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                ),
                TieredSoaU64::K32 {
                    links,
                    widths,
                    words,
                } => run_soa_u64_fixed_v2::<32768>(
                    &w.edges[..3 * ns],
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    img,
                    links,
                    widths,
                    words,
                    rules,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                ),
                TieredSoaU64::K64 {
                    links,
                    widths,
                    words,
                } => run_soa_u64_fixed_v2::<65536>(
                    &w.edges[..3 * ns],
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    img,
                    links,
                    widths,
                    words,
                    rules,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                ),
                TieredSoaU64::None => unreachable!(),
            }
        }
        EndpointScratch::C4FusedSetupOnShipped { stack, gen_cache } => induced_c4_fused_on_shipped(
            &mut w.edges,
            &mut w.bands,
            &mut w.spelling,
            &mut w.inline16,
            &mut w.lens,
            stack,
            gen_cache,
            ns,
            n,
            m,
            first,
            write,
            thin,
            force_spirals,
            ph,
        ),
        EndpointScratch::C5EdgesOverlaySoaU64 { stack, gen_cache } => {
            if most <= 4096 {
                run_c5_edges_u64::<4096, 8192, 16384>(
                    &mut w.edges,
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                )
            } else if most <= 8192 {
                run_c5_edges_u64::<8192, 16384, 32768>(
                    &mut w.edges,
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                )
            } else if most <= 16384 {
                run_c5_edges_u64::<16384, 32768, 65536>(
                    &mut w.edges,
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                )
            } else if most <= 32768 {
                run_c5_edges_u64::<32768, 65536, 131072>(
                    &mut w.edges,
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                )
            } else {
                run_c5_edges_u64::<65536, 131072, 262144>(
                    &mut w.edges,
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                )
            }
        }
        EndpointScratch::C6EdgesOverlaySoaU128 { stack, gen_cache } => {
            if most <= 4096 {
                run_c6_edges_u128::<4096, 16384>(
                    &mut w.edges,
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                )
            } else if most <= 8192 {
                run_c6_edges_u128::<8192, 32768>(
                    &mut w.edges,
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                )
            } else if most <= 16384 {
                run_c6_edges_u128::<16384, 65536>(
                    &mut w.edges,
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                )
            } else if most <= 32768 {
                run_c6_edges_u128::<32768, 131072>(
                    &mut w.edges,
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                )
            } else {
                run_c6_edges_u128::<65536, 262144>(
                    &mut w.edges,
                    &mut w.bands,
                    &mut w.spelling,
                    &mut w.inline16,
                    &mut w.lens,
                    stack,
                    gen_cache,
                    ns,
                    n,
                    m,
                    first,
                    write,
                    thin,
                    force_spirals,
                    ph,
                )
            }
        }
    }
}

/// Full Open pipeline: A6a (OutQ16 + strips) + A6b (arm) + A6c (wire_xor/wire) + A7 (bidi emit).
/// Optionally preceded by real `runs::front` (M/A5) and followed by `rle::unrle + crc::crc32` (A9).
fn run_open_block(
    w: &mut WorkerState,
    records: &[u16],
    n: usize,
    p: u32,
    force_spirals: Option<bool>,
    run_front_and_rle: Option<(&Symbols, u32, Kernels)>,
) -> usize {
    if let Some((syms, _, k)) = run_front_and_rle {
        let _ = runs::front(syms, &mut w.runs, k).expect("runs::front");
    }
    let arm = w.endpoint.arm();
    w.out.resize(n + INLINE, 0);
    let mut queues = Queues::new(&mut w.out, &mut w.spill, u32::MAX);
    // Keep edges.len() across blocks (matching src/open.rs W1 memory diet).
    let mut all = [0u32; 256];
    let mut thin = [0u32; 256];
    let mut thick = [0u32; 256];
    let mut ns = 0usize;
    let mut m = 0u32;
    let mut mark = 0u32;

    maximal(records, |c, pos, width| {
        split(c, pos, width, p, PROD_LIMIT, |c, pos, width, keep| {
            let a = usize::from(c);
            let to = all[a];
            all[a] += width;
            if keep {
                if pos <= p && p < pos + width {
                    mark = m + (p - pos);
                }
                queues.push(a, m, width, &thin);
                thin[a] += width;
                m += width;
            } else {
                thick[a] += 1;
                let base = 3 * ns;
                if w.edges.len() < base + 3 {
                    w.edges.resize(base + 3, 0);
                }
                Strip {
                    from: pos,
                    to,
                    width,
                    byte: c,
                }
                .pack(&mut w.edges[base..base + 3]);
                ns += 1;
            }
        });
    });
    let heads = queues.heads();
    let first = starts(all);
    let mut write = starts(thick);

    // S0_default builds absolute `to` and `image_order` in edges[3*ns..4*ns] before `induced`
    if arm == Arm::S0Default {
        let most = 3 * ns + 3;
        let stride = if most > NARROW_NODES {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        let need = (4 * ns + (stride + RULE) * most).max(m as usize);
        if w.edges.len() < need {
            w.edges.resize(need, 0);
        }
        for i in 0..ns {
            let s = Strip::unpack(&w.edges[3 * i..3 * i + 3]);
            let a = usize::from(s.byte);
            w.edges[3 * i + 1] = s.to + first[a];
            let pos = write[a] as usize;
            write[a] = (pos + 1) as u32;
            w.edges[3 * ns + pos] = i as u32;
        }
    }

    let (cursors, _, _) = dispatch_a6b(
        w,
        ns,
        n as u32,
        m,
        &first,
        &mut write,
        &thin,
        force_spirals,
        None,
    );

    let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
    let wide = w.lens.len() - 1 > (u32::MAX >> bits) as usize;
    if w.edges.len() < m as usize {
        w.edges.resize(m as usize, 0);
    }
    w.extra.clear();
    if wide {
        w.extra.resize(m as usize, 0);
    }

    let mut predecessor = None;
    if !wide && m >= 65_536 {
        let edges = &mut w.edges[..m as usize];
        edges.fill(0);
        let mut pred = mark;
        let mut k = 0;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(w.bands.len());
            let mut queue = QReader::new(&w.out, &w.spill, &heads, a);
            for &band in &w.bands[cursors[a]..end] {
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
            let end = cursors.get(a + 1).copied().unwrap_or(w.bands.len());
            let mut queue = QReader::new(&w.out, &w.spill, &heads, a);
            for &b in &w.bands[cursors[a]..end] {
                let bend = b.end();
                let span = (bend - k) as usize;
                let dst = b.dst() as usize;
                let bcode = b.code();
                let pack = if wide { 0 } else { bcode << bits };
                if wide {
                    w.extra[dst..dst + span].fill(bcode);
                }
                queue.wire(&mut w.edges[dst..dst + span], pack);
                k = bend;
            }
        }
    }

    emit_bidi_or_fwd(
        predecessor,
        &w.edges[..m as usize],
        &w.extra,
        &w.spelling,
        &w.inline16,
        &w.lens,
        n,
        mark,
        bits,
        &mut w.out,
    );

    if let Some((_, want_crc, k)) = run_front_and_rle {
        w.rle_out.clear();
        rle::unrle(&w.out, &mut w.rle_out, k).expect("unrle");
        let got_crc = crc::crc32(&w.rle_out, k);
        assert_eq!(got_crc, want_crc, "block CRC mismatch");
        w.rle_out.len()
    } else {
        w.out.len()
    }
}

/// Isolated A6b step on pre-extracted A6a snapshot (matching `a6b_v2_dissection` Part 1).
fn run_isolated_a6b(
    w: &mut WorkerState,
    blk: &BlockData,
    ph: Option<&mut PhaseNs>,
) -> (usize, usize) {
    let arm = w.endpoint.arm();
    let ns = blk.ns;
    let most = 3 * ns + 3;
    if arm == Arm::S0Default {
        let need = (4 * ns + (Narrow::STRIDE + RULE) * most).max(blk.m as usize);
        if w.edges.len() < need {
            w.edges.resize(need, 0);
        }
        for i in 0..ns {
            let base = 3 * i;
            let from = blk.strips_raw[base];
            let to_rel = blk.strips_raw[base + 1];
            let meta = blk.strips_raw[base + 2];
            let a = usize::from((meta & 0xFF) as u8);
            w.edges[base] = from;
            w.edges[base + 1] = to_rel + blk.first[a];
            w.edges[base + 2] = meta;
        }
        w.edges[3 * ns..4 * ns].copy_from_slice(&blk.image_order);
    } else {
        let need = (3 * ns).max(blk.m as usize);
        if w.edges.len() < need {
            w.edges.resize(need, 0);
        }
        w.edges[..3 * ns].copy_from_slice(&blk.strips_raw);
    }
    let mut write = blk.write_init;
    let (_, n_nodes, n_rules) = dispatch_a6b(
        w,
        ns,
        blk.n as u32,
        blk.m,
        &blk.first,
        &mut write,
        &blk.thin,
        None,
        ph,
    );
    (n_nodes, n_rules)
}

// ---------------------------------------------------------------------------
// Corpus loading & synthetic stress suite
// ---------------------------------------------------------------------------

fn load_blocks(path: &Path) -> Vec<BlockData> {
    let raw = std::fs::read(path).expect("read bz2 file");
    let k = Kernels::detect();
    let mut scanner = Scanner::new(SliceSource(&raw));
    let mut blocks = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    loop {
        match scanner.next(Vec::new()).expect("scan block") {
            Event::Done => break,
            Event::Block { symbols, .. } => {
                let n = mtf::mtf(&symbols, &mut ll, &mut counts, k).expect("mtf");
                let records = records_of(&ll[..n]);
                let cap = open_cap(records.len(), n, symbols.orig_ptr);
                let admitted = cap.is_some_and(|c| {
                    c.m == u32::MAX || within_cap(&records, symbols.orig_ptr, PROD_LIMIT, c)
                });
                let (strips_raw, image_order, ns, m, first, write_init, thin) =
                    build_a6a_snapshot(&records, n, symbols.orig_ptr);
                let p = symbols.orig_ptr;
                let block_crc = symbols.crc;
                blocks.push(BlockData {
                    symbols,
                    records,
                    n,
                    p,
                    crc: block_crc,
                    admitted,
                    strips_raw,
                    image_order,
                    ns,
                    m,
                    first,
                    write_init,
                    thin,
                });
            }
        }
    }
    blocks
}

fn plain_inverse(records: &[u16], n: usize, p: u32) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &rec in records {
        let b = (rec >> 8) as u8;
        let c = usize::from(rec & 255);
        l.resize(l.len() + c, b);
    }
    let mut counts = [0u32; 256];
    for &b in &l {
        counts[usize::from(b)] += 1;
    }
    let mut tt = TVector::default();
    build(&l, &counts, &mut tt);
    let mut out = Vec::with_capacity(n);
    walk(&tt, p, &mut out);
    out
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    fn below(&mut self, bound: u32) -> u32 {
        ((self.next() >> 32) as u32) % bound
    }
}

fn synthetic_stress_cases() -> Vec<(&'static str, Vec<u8>, u32)> {
    let mut cases = Vec::new();
    // 1. ns == 0 (all thin runs)
    cases.push(("ns_zero", (0..2000).map(|i| (i % 7) as u8).collect(), 123));
    // 2. 40k spiral (widest = 39,999 > CERTIFIED_WIDTH)
    let mut sp40 = vec![3u8; 40_000];
    sp40[0] = 1;
    sp40[19_999] = 2;
    cases.push(("spiral_40k", sp40, 7));
    // 3. deep/duplicate long words
    let comb: Vec<u8> = (0..18_000)
        .map(|i| {
            if i % 40 < 20 {
                (i / 40 % 4) as u8
            } else {
                4 + (i % 3) as u8
            }
        })
        .collect();
    cases.push(("deep_long_words", comb, 333));
    // 4. non-ASCII digrams
    let mut non_ascii = Vec::with_capacity(12_000);
    for i in 0..400 {
        let a = 128 + (i % 64) as u8;
        let b = 192 + ((i * 3) % 63) as u8;
        for _ in 0..20 {
            non_ascii.push(a);
        }
        for _ in 0..10 {
            non_ascii.push(b);
        }
    }
    cases.push(("non_ascii_digrams", non_ascii, 77));
    // 5. equal-width strip cascade (aw == bw)
    let mut eq = Vec::with_capacity(16_000);
    for i in 0..400 {
        let c = (i % 16) as u8;
        for _ in 0..40 {
            eq.push(c);
        }
    }
    cases.push(("equal_width_cascade", eq, 19));
    // 6. 70k wide spiral (widest = 69,999 > 65,535)
    let mut sp70 = vec![9u8; 70_000];
    sp70[0] = 4;
    sp70[35_000] = 5;
    cases.push(("spiral_70k", sp70, 11));
    // 7. random mixed runs
    let mut g = Lcg(0x6136_6231);
    let mut mix = Vec::with_capacity(30_000);
    while mix.len() < 30_000 {
        let b = g.below(16) as u8;
        let len = if g.below(3) == 0 {
            17 + g.below(30) as usize
        } else {
            1 + g.below(6) as usize
        };
        let take = len.min(30_000 - mix.len());
        mix.resize(mix.len() + take, b);
    }
    cases.push(("mixed_runs_30k", mix, 999));
    cases
}

// ---------------------------------------------------------------------------
// Subcommands: audit, rss, verify, bench
// ---------------------------------------------------------------------------

fn pct(sorted: &[usize], p: f64) -> usize {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn cmd_audit(path: &Path) {
    let blocks = load_blocks(path);
    let open: Vec<&BlockData> = blocks.iter().filter(|b| b.admitted).collect();
    println!(
        "=== FORENSIC ALLOCATION & WORKING-SET AUDIT ({} total blocks, {} admitted Open) ===",
        blocks.len(),
        open.len()
    );

    let mut w_s0 = WorkerState::new(Arm::S0Default);
    let mut w_s1 = WorkerState::new(Arm::S1Redesign1053);
    let mut w_c5 = WorkerState::new(Arm::C5EdgesOverlaySoaU64);

    let mut ns_v = Vec::new();
    let mut nodes_v = Vec::new();
    let mut most_v = Vec::new();
    let mut m_v = Vec::new();
    let mut rules_s0 = Vec::new();
    let mut rules_s1 = Vec::new();
    let mut rules_u64 = Vec::new();
    let mut bands_v = Vec::new();
    let mut spell_v = Vec::new();
    let mut edges_m_kib = Vec::new();
    let mut arena_s0_kib = Vec::new();

    let mut tier_counts = [0usize; 5]; // <=4K, <=8K, <=16K, <=32K, <=64K
    let mut actual_tier_counts = [0usize; 5];

    for blk in &open {
        let (nn, rs0) = run_isolated_a6b(&mut w_s0, blk, None);
        let (_, rs1) = run_isolated_a6b(&mut w_s1, blk, None);
        let (_, ru64) = run_isolated_a6b(&mut w_c5, blk, None);
        let most = 3 * blk.ns + 3;
        ns_v.push(blk.ns);
        nodes_v.push(nn + 1);
        most_v.push(most);
        m_v.push(blk.m as usize);
        rules_s0.push(rs0);
        rules_s1.push(rs1);
        rules_u64.push(ru64);
        bands_v.push(w_s0.bands.len());
        spell_v.push(w_s0.spelling.len());
        edges_m_kib.push((blk.m as usize * 4) / 1024);
        let s0_words = (4 * blk.ns + 8 * most).max(blk.m as usize);
        arena_s0_kib.push((s0_words * 4) / 1024);

        let tb = |x: usize| {
            if x <= 4096 {
                0
            } else if x <= 8192 {
                1
            } else if x <= 16384 {
                2
            } else if x <= 32768 {
                3
            } else {
                4
            }
        };
        tier_counts[tb(most)] += 1;
        actual_tier_counts[tb(nn + 1)] += 1;
    }

    for (label, v) in [
        ("n_strips (ns)", &mut ns_v),
        ("n_nodes (actual)", &mut nodes_v),
        ("nodes_most (3*ns+3)", &mut most_v),
        ("m (retained rows)", &mut m_v),
        ("rules (S0_default)", &mut rules_s0),
        ("rules (S1_u128)", &mut rules_s1),
        ("rules (C5_u64_i7)", &mut rules_u64),
        ("n_bands", &mut bands_v),
        ("spelling (bytes)", &mut spell_v),
        ("edges 4*m (KiB)", &mut edges_m_kib),
        ("S0 arena (KiB)", &mut arena_s0_kib),
    ] {
        v.sort_unstable();
        let mean = v.iter().sum::<usize>() as f64 / v.len() as f64;
        println!(
            "  {:<22} p50={:>8}  p90={:>8}  p99={:>8}  max={:>8}  mean={:>8.1}",
            label,
            pct(v, 0.50),
            pct(v, 0.90),
            pct(v, 0.99),
            *v.last().unwrap(),
            mean
        );
    }

    println!(
        "\n  Tier distribution by nodes_most (<=4K / <=8K / <=16K / <=32K / <=64K): {} / {} / {} / {} / {}",
        tier_counts[0], tier_counts[1], tier_counts[2], tier_counts[3], tier_counts[4]
    );
    println!(
        "  Tier distribution by actual n_nodes (<=4K / <=8K / <=16K / <=32K / <=64K): {} / {} / {} / {} / {}",
        actual_tier_counts[0], actual_tier_counts[1], actual_tier_counts[2], actual_tier_counts[3], actual_tier_counts[4]
    );

    let max_nodes = *nodes_v.last().unwrap();
    let mean_nodes = nodes_v.iter().sum::<usize>() as f64 / nodes_v.len() as f64;
    let max_rules_u128 = *rules_s1.last().unwrap();
    let max_ns = *ns_v.last().unwrap();
    println!("\n=== THEORETICAL TOUCHED WORKING-SET BUDGET (per worker) ===");
    println!(
        "  S0_default: overlaid in Kernel.edges (0 KiB extra heap); avg live nodes={:.1} KiB + rules={:.1} KiB + cache=24 KiB = {:.1} KiB (inside edges)",
        mean_nodes * 16.0 / 1024.0,
        (rules_s0.iter().sum::<usize>() as f64 / rules_s0.len() as f64) * 16.0 / 1024.0,
        (mean_nodes * 16.0 + (rules_s0.iter().sum::<usize>() as f64 / rules_s0.len() as f64) * 16.0) / 1024.0 + 24.0
    );
    println!(
        "  S1_1053 (64K Box): virtual=1792 KiB; touched at max_nodes={max_nodes}: links={} KiB + widths={} KiB + words128={} KiB + rules={} KiB + img64={} KiB + gen_cache=24 KiB = {} KiB EXTRA RSS/worker!",
        (max_nodes * 8).div_ceil(4096) * 4,
        (max_nodes * 4).div_ceil(4096) * 4,
        (max_nodes * 16).div_ceil(4096) * 4,
        (max_rules_u128.next_power_of_two() * 16) / 1024,
        (max_ns.next_power_of_two() * 8) / 1024,
        (max_nodes * 8).div_ceil(4096) * 4
            + (max_nodes * 4).div_ceil(4096) * 4
            + (max_nodes * 16).div_ceil(4096) * 4
            + (max_rules_u128.next_power_of_two() * 16) / 1024
            + (max_ns.next_power_of_two() * 8) / 1024
            + 24
    );
    println!(
        "  C5_edges_overlay_soa_u64: overlaid in Kernel.edges (+16 KiB gen_cache ONLY); avg live nodes={:.1} KiB (links {:.1} + widths {:.1} + words64 {:.1}) + rules {:.1} + cache 16 = {:.1} KiB (< 256 KiB L2!)",
        mean_nodes * 20.0 / 1024.0,
        mean_nodes * 8.0 / 1024.0,
        mean_nodes * 4.0 / 1024.0,
        mean_nodes * 8.0 / 1024.0,
        (rules_u64.iter().sum::<usize>() as f64 / rules_u64.len() as f64) * 16.0 / 1024.0,
        (mean_nodes * 20.0 + (rules_u64.iter().sum::<usize>() as f64 / rules_u64.len() as f64) * 16.0) / 1024.0 + 16.0
    );
}

fn cmd_rss_child(arm: Arm, k: usize, path: &Path) {
    let blocks = load_blocks(path);
    let open: Vec<&BlockData> = blocks.iter().filter(|b| b.admitted).collect();
    let kernels = Kernels::detect();
    let (rss0, _) = read_proc_rss_kib();
    let mut workers: Vec<WorkerState> = (0..k).map(|_| WorkerState::new(arm)).collect();
    let mut checksum = 0usize;
    for (i, blk) in open.iter().enumerate() {
        let w = &mut workers[i % k];
        checksum ^= run_open_block(
            w,
            &blk.records,
            blk.n,
            blk.p,
            None,
            Some((&blk.symbols, blk.crc, kernels)),
        );
    }
    black_box(checksum);
    let (rss1, hwm1) = read_proc_rss_kib();
    let ep_cap: usize = workers.iter().map(|w| w.endpoint.capacity_bytes()).sum();
    let kern_cap: usize = workers.iter().map(WorkerState::kernel_capacity_bytes).sum();
    let edges_cap: usize = workers.iter().map(|w| w.edges.capacity() * 4).sum();
    println!(
        "{} {} {} {} {} {} {} {}",
        arm.name(),
        k,
        rss0,
        rss1,
        hwm1,
        ep_cap / 1024,
        kern_cap / 1024,
        edges_cap / 1024
    );
}

fn cmd_rss(path: &Path) {
    let exe = std::env::current_exe().expect("current_exe");
    for k in [1usize, 8] {
        println!("\n=== ISOLATED PROCESS VmRSS / VmHWM AUDIT (k = {k} workers, 233 Open wikidata blocks) ===");
        println!(
            "{:<26} | {:>8} {:>8} {:>8} {:>8} | {:>9} {:>9} {:>9}",
            "arm", "VmRSS", "dRSS/S0", "dRSS/w", "VmHWM", "ep_cap/w", "edge_c/w", "tot_c/w"
        );
        let mut s0_rss = 0i64;
        for &arm in &ALL_ARMS {
            let out = Command::new(&exe)
                .args([
                    "rss-child",
                    arm.name(),
                    &k.to_string(),
                    path.to_str().unwrap(),
                ])
                .output()
                .expect("spawn rss-child");
            assert!(
                out.status.success(),
                "rss-child failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            let line = String::from_utf8_lossy(&out.stdout);
            let cols: Vec<&str> = line.split_whitespace().collect();
            let rss1: i64 = cols[3].parse().unwrap();
            let hwm1: i64 = cols[4].parse().unwrap();
            let ep_kib: usize = cols[5].parse().unwrap();
            let kern_kib: usize = cols[6].parse().unwrap();
            let edges_kib: usize = cols[7].parse().unwrap();
            if arm == Arm::S0Default {
                s0_rss = rss1;
            }
            let drss = rss1 - s0_rss;
            let drss_w = drss as f64 / k as f64;
            let ratio = rss1 as f64 / s0_rss as f64;
            println!(
                "{:<26} | {:>7}K {:>+7}K {:>+7.1}K {:>7}K | {:>8}K {:>8}K {:>8}K  ({:.4}x)",
                arm.name(),
                rss1,
                drss,
                drss_w,
                hwm1,
                ep_kib / k,
                edges_kib / k,
                (ep_kib + kern_kib) / k,
                ratio
            );
        }
    }
}

fn cmd_verify(path: &Path, guard_dir: Option<&Path>) {
    let blocks = load_blocks(path);
    let kernels = Kernels::detect();
    let mut total_checks = 0usize;

    // 1. All 431 wikidata blocks (forced Open) across all 8 arms
    let mut wants = Vec::with_capacity(blocks.len());
    for blk in &blocks {
        wants.push(plain_inverse(&blk.records, blk.n, blk.p));
    }
    for &arm in &ALL_ARMS {
        let mut w = WorkerState::new(arm);
        for (idx, blk) in blocks.iter().enumerate() {
            run_open_block(
                &mut w,
                &blk.records,
                blk.n,
                blk.p,
                None,
                Some((&blk.symbols, blk.crc, kernels)),
            );
            assert_eq!(
                w.out,
                wants[idx],
                "wikidata blk {idx} mismatch on {}",
                arm.name()
            );
            total_checks += 1;
        }
    }

    // 2. Synthetic stress suite (spirals = false, true, None) + gen wraparound
    let stress = synthetic_stress_cases();
    for (name, col, p) in &stress {
        let recs = records_of(col);
        let want = plain_inverse(&recs, col.len(), *p);
        for &arm in &ALL_ARMS {
            let mut w = WorkerState::new(arm);
            for spirals in [Some(false), Some(true), None] {
                run_open_block(&mut w, &recs, col.len(), *p, spirals, None);
                assert_eq!(
                    w.out,
                    want,
                    "stress {name} spirals={spirals:?} mismatch on {}",
                    arm.name()
                );
                total_checks += 1;
            }
            // Generation-counter wraparound test (run 5 consecutive blocks across wrap)
            w.endpoint.force_gen_near_wrap();
            for _ in 0..5 {
                run_open_block(&mut w, &recs, col.len(), *p, None, None);
                assert_eq!(w.out, want, "gen-wrap {name} mismatch on {}", arm.name());
                total_checks += 1;
            }
        }
    }

    // 3. Guard corpora (text, runs, records, random) if provided
    if let Some(dir) = guard_dir {
        for gname in ["text", "runs", "records", "random"] {
            let gp = dir.join(format!("{gname}.bz2"));
            if !gp.exists() {
                continue;
            }
            let gblks = load_blocks(&gp);
            let mut gwants = Vec::new();
            for blk in &gblks {
                gwants.push(plain_inverse(&blk.records, blk.n, blk.p));
            }
            for &arm in &ALL_ARMS {
                let mut w = WorkerState::new(arm);
                for (idx, blk) in gblks.iter().enumerate() {
                    run_open_block(
                        &mut w,
                        &blk.records,
                        blk.n,
                        blk.p,
                        None,
                        Some((&blk.symbols, blk.crc, kernels)),
                    );
                    assert_eq!(
                        w.out,
                        gwants[idx],
                        "guard {gname} blk {idx} mismatch on {}",
                        arm.name()
                    );
                    total_checks += 1;
                }
            }
        }
    }

    println!("VERIFY PASS: {total_checks}/{total_checks} 100% byte-exact & CRC-exact checks across all 8 arms!");
}

fn stats(samples: &[f64]) -> (f64, f64, f64) {
    let mut s = samples.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = s[s.len() / 2];
    (med, s[0], *s.last().unwrap())
}

fn disjoint_tag(cand: (f64, f64, f64), base: (f64, f64, f64)) -> &'static str {
    if cand.2 < base.1 {
        "D-" // strictly faster, disjoint
    } else if cand.1 > base.2 {
        "D+" // strictly slower, disjoint
    } else {
        "o"
    }
}

fn cmd_bench(path: &Path, repeats: usize, guard_dir: Option<&Path>) {
    let blocks = load_blocks(path);
    let open: Vec<&BlockData> = blocks.iter().filter(|b| b.admitted).collect();
    let kernels = Kernels::detect();
    let clock = CpuClock::new();

    // -----------------------------------------------------------------------
    // Part 1: Isolated A6b (5 passes/repeat x repeats, warm buffers, CPU clock + sub-phase wall)
    // -----------------------------------------------------------------------
    let passes_p1 = 5usize;
    let mut workers_p1: Vec<WorkerState> = ALL_ARMS.iter().map(|&a| WorkerState::new(a)).collect();
    // Warmup 1 pass
    for w in &mut workers_p1 {
        for blk in &open {
            black_box(run_isolated_a6b(w, blk, None));
        }
    }

    let mut p1_cpu: Vec<Vec<f64>> = vec![Vec::new(); ALL_ARMS.len()];
    let mut p1_wall: Vec<Vec<f64>> = vec![Vec::new(); ALL_ARMS.len()];
    let mut p1_phases: Vec<PhaseNs> = vec![PhaseNs::default(); ALL_ARMS.len()];

    for r in 0..repeats {
        for offset in 0..ALL_ARMS.len() {
            let ai = (r + offset) % ALL_ARMS.len();
            let w = &mut workers_p1[ai];
            let c0 = clock.now_ns();
            let t0 = Instant::now();
            let mut acc = 0usize;
            for _ in 0..passes_p1 {
                for blk in &open {
                    let (nn, nr) = run_isolated_a6b(w, blk, None);
                    acc ^= nn ^ nr;
                }
            }
            black_box(acc);
            let wall_ms = t0.elapsed().as_secs_f64() * 1000.0 / passes_p1 as f64;
            let cpu_ms = (clock.now_ns() - c0) as f64 / 1e6 / passes_p1 as f64;
            p1_cpu[ai].push(cpu_ms);
            p1_wall[ai].push(wall_ms);
        }
    }
    // Single clean sub-phase wall breakdown pass
    for (ai, w) in workers_p1.iter_mut().enumerate() {
        let mut ph = PhaseNs::default();
        for _ in 0..3 {
            for blk in &open {
                let mut bp = PhaseNs::default();
                black_box(run_isolated_a6b(w, blk, Some(&mut bp)));
                ph.add(bp);
            }
        }
        p1_phases[ai] = PhaseNs {
            setup: ph.setup / 3,
            steps: ph.steps / 3,
            bot: ph.bot / 3,
            dict: ph.dict / 3,
        };
    }

    println!("\n=== PART 1: ISOLATED A6b ({repeats} interleaved repeats x {passes_p1} passes, 233 Open wikidata blocks) ===");
    println!(
        "{:<26} | {:>24} {:>8} {:>3} | {:>8} {:>8} {:>6} {:>8}",
        "arm", "A6b_cpu [min..max] ms", "vs S0", "C", "setup", "steps", "bot", "dict"
    );
    let s0_p1 = stats(&p1_cpu[0]);
    for (ai, &arm) in ALL_ARMS.iter().enumerate() {
        let st = stats(&p1_cpu[ai]);
        let delta = (st.0 / s0_p1.0 - 1.0) * 100.0;
        let tag = disjoint_tag(st, s0_p1);
        let ph = p1_phases[ai];
        println!(
            "{:<26} | {:>6.2} [{:>6.2}..{:>6.2}] {:>+7.2}% {:>3} | {:>7.2}m {:>7.2}m {:>5.2}m {:>7.2}m",
            arm.name(),
            st.0,
            st.1,
            st.2,
            delta,
            tag,
            ph.setup as f64 / 1e6,
            ph.steps as f64 / 1e6,
            ph.bot as f64 / 1e6,
            ph.dict as f64 / 1e6
        );
    }

    // -----------------------------------------------------------------------
    // Part 2: Full Open Worker Warm (k=1: runs::front + A6a + A6b + A6c + A7(bidi) + A9)
    // -----------------------------------------------------------------------
    let mut workers_p2: Vec<WorkerState> = ALL_ARMS.iter().map(|&a| WorkerState::new(a)).collect();
    for w in &mut workers_p2 {
        for blk in &open {
            black_box(run_open_block(
                w,
                &blk.records,
                blk.n,
                blk.p,
                None,
                Some((&blk.symbols, blk.crc, kernels)),
            ));
        }
    }
    let mut p2_cpu: Vec<Vec<f64>> = vec![Vec::new(); ALL_ARMS.len()];
    for r in 0..repeats {
        for offset in 0..ALL_ARMS.len() {
            let ai = (r + offset) % ALL_ARMS.len();
            let w = &mut workers_p2[ai];
            let c0 = clock.now_ns();
            let mut acc = 0usize;
            for blk in &open {
                acc ^= run_open_block(
                    w,
                    &blk.records,
                    blk.n,
                    blk.p,
                    None,
                    Some((&blk.symbols, blk.crc, kernels)),
                );
            }
            black_box(acc);
            p2_cpu[ai].push((clock.now_ns() - c0) as f64 / 1e6);
        }
    }
    println!("\n=== PART 2: FULL OPEN WORKER WARM k=1 (A5/M + A6a + A6b + A6c + A7 + A9, {repeats} repeats) ===");
    let s0_p2 = stats(&p2_cpu[0]);
    let s1_p2 = stats(&p2_cpu[1]);
    println!(
        "{:<26} | {:>26} {:>8} {:>3} | {:>8} {:>3}",
        "arm", "Full_warm_cpu [min..max]", "vs S0", "C", "vs S1", "C"
    );
    for (ai, &arm) in ALL_ARMS.iter().enumerate() {
        let st = stats(&p2_cpu[ai]);
        println!(
            "{:<26} | {:>7.2} [{:>7.2}..{:>7.2}] {:>+7.2}% {:>3} | {:>+7.2}% {:>3}",
            arm.name(),
            st.0,
            st.1,
            st.2,
            (st.0 / s0_p2.0 - 1.0) * 100.0,
            disjoint_tag(st, s0_p2),
            (st.0 / s1_p2.0 - 1.0) * 100.0,
            disjoint_tag(st, s1_p2),
        );
    }

    // -----------------------------------------------------------------------
    // Part 3: Full Open Worker L3-Cold 8-Worker Rotating Pool (k=8 independent workers)
    // -----------------------------------------------------------------------
    let mut pools_p3: Vec<Vec<WorkerState>> = ALL_ARMS
        .iter()
        .map(|&a| (0..8).map(|_| WorkerState::new(a)).collect())
        .collect();
    for pool in &mut pools_p3 {
        for (i, blk) in open.iter().enumerate() {
            black_box(run_open_block(
                &mut pool[i % 8],
                &blk.records,
                blk.n,
                blk.p,
                None,
                Some((&blk.symbols, blk.crc, kernels)),
            ));
        }
    }
    let mut p3_cpu: Vec<Vec<f64>> = vec![Vec::new(); ALL_ARMS.len()];
    for r in 0..repeats {
        for offset in 0..ALL_ARMS.len() {
            let ai = (r + offset) % ALL_ARMS.len();
            let pool = &mut pools_p3[ai];
            let c0 = clock.now_ns();
            let mut acc = 0usize;
            for (i, blk) in open.iter().enumerate() {
                acc ^= run_open_block(
                    &mut pool[i % 8],
                    &blk.records,
                    blk.n,
                    blk.p,
                    None,
                    Some((&blk.symbols, blk.crc, kernels)),
                );
            }
            black_box(acc);
            p3_cpu[ai].push((clock.now_ns() - c0) as f64 / 1e6);
        }
    }
    println!("\n=== PART 3: FULL OPEN WORKER L3-COLD 8-WORKER POOL (k=8 rotating, {repeats} repeats) ===");
    let s0_p3 = stats(&p3_cpu[0]);
    let s1_p3 = stats(&p3_cpu[1]);
    println!(
        "{:<26} | {:>26} {:>8} {:>3} | {:>8} {:>3} | {:>10}",
        "arm", "Full_cold8_cpu [min..max]", "vs S0", "C", "vs S1", "C", "pool_ep_K"
    );
    for (ai, &arm) in ALL_ARMS.iter().enumerate() {
        let st = stats(&p3_cpu[ai]);
        let ep_k: usize = pools_p3[ai]
            .iter()
            .map(|w| w.endpoint.capacity_bytes())
            .sum::<usize>()
            / 1024;
        println!(
            "{:<26} | {:>7.2} [{:>7.2}..{:>7.2}] {:>+7.2}% {:>3} | {:>+7.2}% {:>3} | {:>9}K",
            arm.name(),
            st.0,
            st.1,
            st.2,
            (st.0 / s0_p3.0 - 1.0) * 100.0,
            disjoint_tag(st, s0_p3),
            (st.0 / s1_p3.0 - 1.0) * 100.0,
            disjoint_tag(st, s1_p3),
            ep_k
        );
    }

    // -----------------------------------------------------------------------
    // Part 4: Guard Corpora (text, runs, records, random)
    // -----------------------------------------------------------------------
    if let Some(dir) = guard_dir {
        println!("\n=== PART 4: GUARD CORPORA (5 interleaved repeats, full Open pipeline on all blocks) ===");
        for gname in ["text", "runs", "records", "random"] {
            let gp = dir.join(format!("{gname}.bz2"));
            if !gp.exists() {
                continue;
            }
            let gblks = load_blocks(&gp);
            let gopen: Vec<&BlockData> = gblks.iter().filter(|b| b.admitted).collect();
            let target: Vec<&BlockData> = if gopen.is_empty() {
                gblks.iter().collect()
            } else {
                gopen
            };
            let label = if gblks.iter().any(|b| b.admitted) {
                format!("{gname} ({}/{} admitted)", target.len(), gblks.len())
            } else {
                format!("{gname} ({} forced Open)", target.len())
            };
            let mut gws: Vec<WorkerState> = ALL_ARMS.iter().map(|&a| WorkerState::new(a)).collect();
            for w in &mut gws {
                for blk in &target {
                    black_box(run_open_block(
                        w,
                        &blk.records,
                        blk.n,
                        blk.p,
                        None,
                        Some((&blk.symbols, blk.crc, kernels)),
                    ));
                }
            }
            let mut gcpu: Vec<Vec<f64>> = vec![Vec::new(); ALL_ARMS.len()];
            for r in 0..5 {
                for offset in 0..ALL_ARMS.len() {
                    let ai = (r + offset) % ALL_ARMS.len();
                    let w = &mut gws[ai];
                    let c0 = clock.now_ns();
                    let mut acc = 0usize;
                    for blk in &target {
                        acc ^= run_open_block(
                            w,
                            &blk.records,
                            blk.n,
                            blk.p,
                            None,
                            Some((&blk.symbols, blk.crc, kernels)),
                        );
                    }
                    black_box(acc);
                    gcpu[ai].push((clock.now_ns() - c0) as f64 / 1e6);
                }
            }
            let s0g = stats(&gcpu[0]);
            print!("  {:<26}:", label);
            for (ai, &arm) in ALL_ARMS.iter().enumerate() {
                let st = stats(&gcpu[ai]);
                print!(
                    " {}={:.3}x",
                    arm.name().split('_').next().unwrap(),
                    st.0 / s0g.0
                );
            }
            println!("  (S0={:.2} ms)", s0g.0);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <audit|rss|verify|bench> <wikidata.bz2> [...]",
            args[0]
        );
        std::process::exit(2);
    }
    match args[1].as_str() {
        "audit" => cmd_audit(Path::new(&args[2])),
        "rss-child" => {
            let arm = Arm::from_name(&args[2]).expect("unknown arm");
            let k: usize = args[3].parse().expect("k");
            cmd_rss_child(arm, k, Path::new(&args[4]));
        }
        "rss" => cmd_rss(Path::new(&args[2])),
        "verify" => {
            let gdir = args.get(3).map(Path::new);
            cmd_verify(Path::new(&args[2]), gdir);
        }
        "bench" => {
            let mut repeats = 7usize;
            let mut gdir: Option<&Path> = None;
            let mut i = 3;
            while i < args.len() {
                if args[i] == "--repeats" && i + 1 < args.len() {
                    repeats = args[i + 1].parse().expect("repeats");
                    i += 2;
                } else if args[i] == "--guards" && i + 1 < args.len() {
                    gdir = Some(Path::new(&args[i + 1]));
                    i += 2;
                } else {
                    i += 1;
                }
            }
            cmd_bench(Path::new(&args[2]), repeats, gdir);
        }
        other => {
            eprintln!("Unknown subcommand: {other}");
            std::process::exit(2);
        }
    }
}
