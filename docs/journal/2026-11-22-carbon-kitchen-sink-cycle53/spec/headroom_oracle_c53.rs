//! Cycle 53 Cumulative Headroom Oracle v8 + `CHUNK in {256, 512, 1024}` & Compact `u16`
//! `Dictionary::contents` & `SlimS5_Pipe1_Rem16` Ablation + Classical Hölder (1889) /
//! Jensen (1906) Moment-Entropy Ceiling & Marginal-Moment Information Barrier (`C53-D4`,
//! Extending Chain D `CA-26→27→31→37→41→44→47` to `8` Pure-`CA` Nodes / `7` Links!).
//!
//! Compares Cycle 53's unified **`C53-SuperPipe-v4`** head-to-head against Cycle 52's two
//! live `#729` record-holders AND every controlled ablation (`10` arms total) at `1T`, `2W`,
//! and `4W` across ALL 4 real corpora + `441`-block union + `243`-full union:
//!   0. `R5-Head-A1`         — verbatim `cand_r5_head` (`ID_BITS=22`, `L=16`, `AlwaysAt(65_536)`)
//!   1. `R5-Head-A2(AA)`     — paired A/A noise control
//!   2. `C52-Lean-Compact`   — Cycle 52 P3 (`CS-35`: `CHUNK=512`, `U32-InEdges`, `SlimS4-8`, `SlimS5_Rem16`)
//!   3. `C52-SuperDropIn-v3` — Cycle 52 Spec (`CS-37`: `CHUNK=512`, `U32-InEdges`, `V3-16`, `V3-Peeled`)
//!   4. `Abl-Chunk256`       — `CHUNK=256`, `U16-Stack-Tag4`, `SlimS4-8`, `SlimS5_Pipe1_Rem16`
//!   5. `Abl-Chunk1024`      — `CHUNK=1024`, `U16-Stack-Tag4`, `SlimS4-8`, `SlimS5_Pipe1_Rem16`
//!   6. `Abl-U16Raw`         — `CHUNK=512`, `U16-Stack-Raw` (`<65534`), `SlimS4-8`, `SlimS5_Pipe1_Rem16`
//!   7. `Abl-U32Dict`        — `CHUNK=512`, `U32-InEdges`, `SlimS4-8`, `SlimS5_Pipe1_Rem16` (isolates `u16` dict)
//!   8. `Abl-NoPipe1`        — `CHUNK=512`, `U16-Stack-Tag4`, `SlimS4-8`, `SlimS5_Rem16` (isolates `Pipe1`)
//!   9. `C53-SuperPipe-v4`   — Cycle 53 Unified Champion (winning `CHUNK` + `u16` `contents` +
//!                             `SlimS4` + `SlimS5_Pipe1_Rem16` + `PackedEdgeRule32-Sentinel` + 4-tier `TinyFL`)
//!
//! Also supports `--proxy-arm <name>` for deterministic `valgrind --tool=cachegrind` (`256 KiB` L2,
//! `2 MiB` `W=10` LLC) on the exact `#729` `6+4` proxy blocks (`34,108,178,254,327,391,60,163,265,372`).
//!
//! Safe Rust: `#![forbid(unsafe_code)]`, strictly forward-`FL` exact on ALL valid & malformed inputs.
#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::time::Instant;

// ============================================================================
// 1. Shared Low-Level Constants & Helpers
// ============================================================================

const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const MAX_RULE_WORDS: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const INLINE_STACK: usize = 64;
const MEMO_REC_CAP: usize = 32;
const SPELLING_STEP: usize = 4096;
const INLINE_STEP: usize = 128;
const PAIR_WORDS: usize = 512;
const CACHE_WORDS_U32: usize = PAIR_WORDS + 2048;
const CACHE_WORDS_U16: usize = PAIR_WORDS + 1024;
const ID_BITS: u32 = 24;
const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;

// 256-byte .rodata mask table for branchless 4-word CE-Var234 `short_payload_h`.
const SHORT_MASK: [u128; 16] = [
    0x0000_0000_0000_0000_0000_0000_0000_0000,
    0x0000_0000_0000_0000_0000_0000_0000_00FF,
    0x0000_0000_0000_0000_0000_0000_0000_FFFF,
    0x0000_0000_0000_0000_0000_0000_00FF_FFFF,
    0x0000_0000_0000_0000_0000_0000_FFFF_FFFF,
    0x0000_0000_0000_0000_0000_00FF_FFFF_FFFF,
    0x0000_0000_0000_0000_0000_FFFF_FFFF_FFFF,
    0x0000_0000_0000_0000_00FF_FFFF_FFFF_FFFF,
    0x0000_0000_0000_0000_FFFF_FFFF_FFFF_FFFF,
    0x0000_0000_0000_00FF_FFFF_FFFF_FFFF_FFFF,
    0x0000_0000_0000_FFFF_FFFF_FFFF_FFFF_FFFF,
    0x0000_0000_00FF_FFFF_FFFF_FFFF_FFFF_FFFF,
    0x0000_0000_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
    0x0000_00FF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
    0x0000_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
    0x00FF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
];

#[inline(always)]
const fn literal(byte: u8) -> u32 {
    (1u32 << ID_BITS) | (byte as u32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BiMode {
    AlwaysAt(u32),
    PostS1ExactMd {
        r_max_cap: u32,
        ns_max: u32,
        md_max: i64,
        md_neg_cap: i64,
        ns_neg_max: u32,
    },
}

impl BiMode {
    #[inline(always)]
    fn should_use_bi(self, m: u32, ns: u32, r_max: u32) -> bool {
        match self {
            BiMode::AlwaysAt(thr) => m >= thr,
            BiMode::PostS1ExactMd {
                r_max_cap,
                ns_max,
                md_max,
                md_neg_cap,
                ns_neg_max,
            } => {
                if m < 65_536 {
                    return false;
                }
                let md_w1 = 2i64 * (i64::from(r_max) - i64::from(ns)) - i64::from(m);
                !((r_max <= r_max_cap && ns <= ns_max && md_w1 <= md_max)
                    || (md_w1 < md_neg_cap && ns <= ns_neg_max))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cap {
    m: u32,
    strips: usize,
    post_s1_crossover: bool,
}

impl Cap {
    const NONE: Cap = Cap { m: u32::MAX, strips: usize::MAX, post_s1_crossover: false };
    const C53_CROSSOVER: Cap = Cap { m: u32::MAX, strips: usize::MAX, post_s1_crossover: true };
}

#[derive(Default, Clone, Copy, Debug)]
struct Counts {
    m: usize,
    bits: u32,
    num_codes: usize,
    wide: bool,
    used_bi: bool,
    used_tail: bool,
    used_u32_fallback: bool,
    outq_tail_bytes: usize,
    outq_spill_used: usize,
    contents_bytes: usize,
    spelling_used: usize,
    used_bytes: usize,
    s5_live_bytes: usize,
    rule_words: usize,
    s2b_steps: usize,
}

#[derive(Clone, Copy)]
struct Band(u64);

impl Band {
    #[inline(always)]
    fn new(end: u32, dst: u32, code: u32) -> Self {
        Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
    }
    #[inline(always)]
    fn end(self) -> u32 { (self.0 & 0xF_FFFF) as u32 }
    #[inline(always)]
    fn dst(self) -> u32 { ((self.0 >> 20) & 0xF_FFFF) as u32 }
    #[inline(always)]
    fn code(self) -> u32 { (self.0 >> 40) as u32 }
}

#[derive(Clone, Copy)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    mark_byte: u8,
}

impl Default for First {
    fn default() -> Self {
        Self { thick_rows: [0; 256], thin: [0; 256], thick: [0; 256], m: 0, mark: 0, mark_byte: 0 }
    }
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| { let s = total; total += w; s })
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

fn begin_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
    if inline16.len() < 256 {
        inline16.clear();
        inline16.reserve_exact(256);
        inline16.extend((0..=255u8).map(|c| { let mut v = [0; INLINE]; v[0] = c; v }));
        lens.clear();
        lens.reserve_exact(256);
        lens.resize(256, 1);
    }
    inline16.truncate(256);
    lens.truncate(256);
}

#[cold]
#[inline(never)]
fn grow_step_u8(v: &mut Vec<u8>, need: usize, step: usize) {
    let target = need.div_ceil(step) * step;
    v.reserve_exact(target - v.len());
}

#[inline(always)]
fn ensure_step_u8(v: &mut Vec<u8>, add: usize, step: usize) {
    let need = v.len() + add;
    if need > v.capacity() { grow_step_u8(v, need, step); }
}

#[cold]
#[inline(never)]
fn grow_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
    let target = (lens.len() + 1).div_ceil(step) * step;
    inline16.reserve_exact(target - inline16.len());
    lens.reserve_exact(target - lens.len());
}

#[inline(always)]
fn ensure_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
    if lens.len() == lens.capacity() { grow_step_inline(inline16, lens, step); }
}

#[cold]
#[inline(never)]
fn grow_edges_step(a: &mut Vec<u32>, need: usize) {
    let target = need.div_ceil(8192) * 8192;
    if target > a.capacity() { a.reserve_exact(target - a.len()); }
    a.resize(target, 0);
}

fn spelling_hash_long(bytes: &[u8]) -> u64 {
    let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ part(bytes.len() - 8).rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

#[inline(always)]
fn invert_tiny_fl_with_buf(records: &[u16], n: usize, p: u32, fl: &mut [u32], out: &mut Vec<u8>) -> bool {
    let mut counts = [0u32; 256];
    for &rec in records { counts[(rec >> 8) as usize] += u32::from(rec & 255); }
    let mut st = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 { st[c] = tot; tot += counts[c]; }
    if tot as usize != n { return false; }
    let mut pos = 0usize;
    for &rec in records {
        let (b, len) = ((rec >> 8) as usize, (rec & 255) as usize);
        let s0 = st[b] as usize; st[b] += len as u32;
        let (mut i, bu) = (pos as u32, b as u32);
        for slot in &mut fl[s0..s0 + len] { *slot = (i << 8) | bu; i += 1; }
        pos += len;
    }
    out.resize(n, 0);
    let mut x = p as usize;
    for slot in &mut out[..n] { let v = fl[x]; *slot = v as u8; x = (v >> 8) as usize; }
    true
}

#[inline(never)]
fn invert_tiny_fl_64(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    let mut buf = [0u32; 64]; invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
}
#[inline(never)]
fn invert_tiny_fl_256(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    let mut buf = [0u32; 256]; invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
}
#[inline(never)]
fn invert_tiny_fl_512(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    let mut buf = [0u32; 512]; invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
}
#[inline(never)]
fn invert_tiny_fl_1536(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    let mut buf = [0u32; 1536]; invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
}

// ============================================================================
// 2. Parameterized `OutQ16` Supporting `CHUNK in {256, 512, 1024}` + `S4_MODE`
// ============================================================================

mod outq_gen {
    const NONE32: u32 = u32::MAX;
    const NONE16: u16 = u16::MAX;
    const EPOCH: u32 = 1 << 16;

    pub struct Spill<const BYTES: usize> {
        pub next: Vec<u16>,
        pub ovf: Vec<[u8; BYTES]>,
        pub cuts: Vec<[u32; 256]>,
    }

    impl<const BYTES: usize> Default for Spill<BYTES> {
        fn default() -> Self {
            Self { next: Vec::new(), ovf: Vec::new(), cuts: Vec::new() }
        }
    }

    impl<const BYTES: usize> Spill<BYTES> {
        pub fn capacity_bytes(&self) -> usize {
            2 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
        }
        pub fn used_bytes(&self, chunks: u32) -> usize {
            2 * (chunks as usize) + BYTES * self.ovf.len() + 1024 * self.cuts.len()
        }
    }

    pub struct Queues<'a, const LOG_CHUNK: u32, const BYTES: usize> {
        out: &'a mut [u8],
        spill: &'a mut Spill<BYTES>,
        in_out: u32,
        chunks: u32,
        heads: [u32; 256],
        pt: [u64; 256],
        limit: u32,
        over: bool,
    }

    pub struct Heads {
        heads: [u32; 256],
        in_out: u32,
        pub chunks: u32,
    }

    impl<'a, const LOG_CHUNK: u32, const BYTES: usize> Queues<'a, LOG_CHUNK, BYTES> {
        const CHUNK: usize = 1usize << LOG_CHUNK;
        const SHIFT: u32 = LOG_CHUNK + 1;
        const MASK: u32 = (BYTES as u32) - 1;

        pub fn new(out: &'a mut [u8], spill: &'a mut Spill<BYTES>, limit: u32) -> Self {
            debug_assert_eq!(BYTES, 2 * Self::CHUNK);
            spill.ovf.clear();
            spill.cuts.clear();
            let in_out = (out.len() / BYTES) as u32;
            let out = &mut out[..in_out as usize * BYTES];
            Queues {
                in_out, out, spill, chunks: 0,
                heads: [NONE32; 256], pt: [u32::MAX as u64; 256], limit, over: false,
            }
        }
        pub fn over(&self) -> bool { self.over }
        pub fn heads(&self) -> Heads { Heads { heads: self.heads, in_out: self.in_out, chunks: self.chunks } }
        #[inline(always)]
        pub fn thin_of(&self, a: usize) -> u32 { (self.pt[a] >> 32) as u32 }
        pub fn extract_thin(&self) -> [u32; 256] { std::array::from_fn(|a| (self.pt[a] >> 32) as u32) }

        #[inline(always)]
        pub fn push_thin(&mut self, a: usize, m: u32, w: u32) {
            let v = self.pt[a];
            let p = v as u32;
            let f = (p & Self::MASK) as usize;
            if (w <= 16) & (f <= Self::CHUNK - 16) & ((m as u16) < 65_520) {
                let at = p as usize + f;
                if let Some(dst) = self.out.get_mut(at..at + 32) {
                    let lo = m as u16;
                    let iota: [u16; 16] = std::array::from_fn(|i| lo.wrapping_add(i as u16));
                    for (d, x) in dst.chunks_exact_mut(2).zip(iota) {
                        d.copy_from_slice(&x.to_le_bytes());
                    }
                    let wu = u64::from(w);
                    self.pt[a] = v + ((wu << 32) | wu);
                    return;
                }
            }
            self.push_slow_pt(a, m, w);
        }

        #[cold]
        #[inline(never)]
        fn push_slow_pt(&mut self, a: usize, mut m: u32, w: u32) {
            if self.over || m + w > self.limit { self.over = true; return; }
            let mut left = w;
            while left > 0 {
                let p = self.pt[a] as u32;
                if p == u32::MAX || (p & Self::MASK) as usize == Self::CHUNK {
                    if self.chunks >= u32::from(NONE16) { self.over = true; return; }
                    self.take_chunk_pt(a);
                }
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & Self::MASK) as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((Self::CHUNK - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(p >> Self::SHIFT);
                for (i, d) in chunk[2 * f..2 * (f + take as usize)].chunks_exact_mut(2).enumerate() {
                    d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                }
                let tu = u64::from(take);
                self.pt[a] = v + ((tu << 32) | tu);
                m += take;
                left -= take;
                if m % EPOCH == 0 {
                    let cut: [u32; 256] = std::array::from_fn(|b| (self.pt[b] >> 32) as u32);
                    if self.spill.cuts.len() == self.spill.cuts.capacity() { self.spill.cuts.reserve_exact(1); }
                    self.spill.cuts.push(cut);
                }
            }
        }

        fn take_chunk_pt(&mut self, a: usize) {
            let id = self.chunks;
            self.chunks += 1;
            if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                if self.spill.ovf.len() == self.spill.ovf.capacity() {
                    let ovf_step = (16_384 / BYTES).max(1);
                    let target = (self.spill.ovf.len() + 1).div_ceil(ovf_step) * ovf_step;
                    self.spill.ovf.reserve_exact(target - self.spill.ovf.len());
                }
                self.spill.ovf.push([0; BYTES]);
            }
            let idu = id as usize;
            if self.spill.next.len() <= idu {
                let target = (idu + 1).div_ceil(256) * 256;
                if target > self.spill.next.capacity() {
                    self.spill.next.reserve_exact(target - self.spill.next.len());
                }
                self.spill.next.resize(target, NONE16);
            }
            self.spill.next[idu] = NONE16;
            let hi = self.pt[a] & !0xFFFF_FFFFu64;
            if self.heads[a] == NONE32 { self.heads[a] = id; }
            else { self.spill.next[((self.pt[a] as u32) >> Self::SHIFT) as usize] = id as u16; }
            self.pt[a] = hi | u64::from(id << Self::SHIFT);
        }

        #[cold]
        #[inline(never)]
        fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
            if id < self.in_out { let at = id as usize * BYTES; &mut self.out[at..at + BYTES] }
            else { &mut self.spill.ovf[(id - self.in_out) as usize][..] }
        }
    }

    pub struct Reader<'a, const LOG_CHUNK: u32, const BYTES: usize> {
        out: &'a [u8],
        spill: &'a Spill<BYTES>,
        in_out: u32,
        a: usize,
        id: u32,
        at: usize,
        read: u32,
        epoch: u32,
        cut: u32,
    }

    impl<'a, const LOG_CHUNK: u32, const BYTES: usize> Reader<'a, LOG_CHUNK, BYTES> {
        const CHUNK: usize = 1usize << LOG_CHUNK;

        #[inline(always)]
        pub fn new(out: &'a [u8], spill: &'a Spill<BYTES>, heads: &Heads, a: usize) -> Self {
            Reader {
                out, spill, in_out: heads.in_out, a, id: heads.heads[a],
                at: 0, read: 0, epoch: 0, cut: spill.cuts.first().map_or(u32::MAX, |c| c[a]),
            }
        }
        #[inline(always)]
        fn next_chunk(&self, id: usize) -> u32 { u32::from(self.spill.next[id]) }
        #[inline(always)]
        fn cut_at(&self, epoch: usize) -> u32 { self.spill.cuts.get(epoch).map_or(u32::MAX, |c| c[self.a]) }

        /// S4_MODE = 0: V3-16way (u64 loads + scalar slow),
        /// S4_MODE = 1: SlimS4-8way (`as_chunks::<2>()` + `pack_hi` + `hi==0` + 8-way batched slow),
        /// S4_MODE = 2: SlimS4-16way (`as_chunks::<2>()` + 16-way `eu[0..16]` + `pack_hi` + `hi==0` + 8-way batched slow).
        #[inline(always)]
        pub fn wire_xor_nomark<const S4_MODE: u8>(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack: u32,
        ) {
            if self.at + span <= Self::CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                if S4_MODE == 0 {
                    let bytes = &self.out[at..at + 2 * span];
                    let mut i = 0usize;
                    while i + 16 <= span {
                        let b = &bytes[2 * i..2 * i + 32];
                        let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                        let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                        let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                        let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                        let (v0, v1, v2, v3) = (hi | (q0 as u32 & 0xFFFF), hi | ((q0 >> 16) as u32 & 0xFFFF), hi | ((q0 >> 32) as u32 & 0xFFFF), hi | ((q0 >> 48) as u32));
                        let (v4, v5, v6, v7) = (hi | (q1 as u32 & 0xFFFF), hi | ((q1 >> 16) as u32 & 0xFFFF), hi | ((q1 >> 32) as u32 & 0xFFFF), hi | ((q1 >> 48) as u32));
                        let (v8, v9, v10, v11) = (hi | (q2 as u32 & 0xFFFF), hi | ((q2 >> 16) as u32 & 0xFFFF), hi | ((q2 >> 32) as u32 & 0xFFFF), hi | ((q2 >> 48) as u32));
                        let (v12, v13, v14, v15) = (hi | (q3 as u32 & 0xFFFF), hi | ((q3 >> 16) as u32 & 0xFFFF), hi | ((q3 >> 32) as u32 & 0xFFFF), hi | ((q3 >> 48) as u32));
                        let u0 = dst_start + i;
                        let eu = &mut edges[u0..u0 + 16];
                        eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                        eu[4] ^= pack | v4; eu[5] ^= pack | v5; eu[6] ^= pack | v6; eu[7] ^= pack | v7;
                        eu[8] ^= pack | v8; eu[9] ^= pack | v9; eu[10] ^= pack | v10; eu[11] ^= pack | v11;
                        eu[12] ^= pack | v12; eu[13] ^= pack | v13; eu[14] ^= pack | v14; eu[15] ^= pack | v15;
                        edges[v0 as usize] ^= u0 as u32; edges[v1 as usize] ^= (u0 + 1) as u32;
                        edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                        edges[v4 as usize] ^= (u0 + 4) as u32; edges[v5 as usize] ^= (u0 + 5) as u32;
                        edges[v6 as usize] ^= (u0 + 6) as u32; edges[v7 as usize] ^= (u0 + 7) as u32;
                        edges[v8 as usize] ^= (u0 + 8) as u32; edges[v9 as usize] ^= (u0 + 9) as u32;
                        edges[v10 as usize] ^= (u0 + 10) as u32; edges[v11 as usize] ^= (u0 + 11) as u32;
                        edges[v12 as usize] ^= (u0 + 12) as u32; edges[v13 as usize] ^= (u0 + 13) as u32;
                        edges[v14 as usize] ^= (u0 + 14) as u32; edges[v15 as usize] ^= (u0 + 15) as u32;
                        i += 16;
                    }
                    if i + 8 <= span {
                        let b = &bytes[2 * i..2 * i + 16];
                        let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                        let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                        let (v0, v1, v2, v3) = (hi | (q0 as u32 & 0xFFFF), hi | ((q0 >> 16) as u32 & 0xFFFF), hi | ((q0 >> 32) as u32 & 0xFFFF), hi | ((q0 >> 48) as u32));
                        let (v4, v5, v6, v7) = (hi | (q1 as u32 & 0xFFFF), hi | ((q1 >> 16) as u32 & 0xFFFF), hi | ((q1 >> 32) as u32 & 0xFFFF), hi | ((q1 >> 48) as u32));
                        let u0 = dst_start + i;
                        let eu = &mut edges[u0..u0 + 8];
                        eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                        eu[4] ^= pack | v4; eu[5] ^= pack | v5; eu[6] ^= pack | v6; eu[7] ^= pack | v7;
                        edges[v0 as usize] ^= u0 as u32; edges[v1 as usize] ^= (u0 + 1) as u32;
                        edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                        edges[v4 as usize] ^= (u0 + 4) as u32; edges[v5 as usize] ^= (u0 + 5) as u32;
                        edges[v6 as usize] ^= (u0 + 6) as u32; edges[v7 as usize] ^= (u0 + 7) as u32;
                        i += 8;
                    }
                    while i < span {
                        let u = dst_start + i;
                        let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                        edges[u] ^= pack | v; edges[v as usize] ^= u as u32;
                        i += 1;
                    }
                } else {
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    let mut i = 0usize;
                    if hi == 0 {
                        if S4_MODE == 2 {
                            let (chunks16, _) = pairs.as_chunks::<16>();
                            for ch in chunks16 {
                                let v0 = u32::from(u16::from_le_bytes(ch[0]));
                                let v1 = u32::from(u16::from_le_bytes(ch[1]));
                                let v2 = u32::from(u16::from_le_bytes(ch[2]));
                                let v3 = u32::from(u16::from_le_bytes(ch[3]));
                                let v4 = u32::from(u16::from_le_bytes(ch[4]));
                                let v5 = u32::from(u16::from_le_bytes(ch[5]));
                                let v6 = u32::from(u16::from_le_bytes(ch[6]));
                                let v7 = u32::from(u16::from_le_bytes(ch[7]));
                                let v8 = u32::from(u16::from_le_bytes(ch[8]));
                                let v9 = u32::from(u16::from_le_bytes(ch[9]));
                                let v10 = u32::from(u16::from_le_bytes(ch[10]));
                                let v11 = u32::from(u16::from_le_bytes(ch[11]));
                                let v12 = u32::from(u16::from_le_bytes(ch[12]));
                                let v13 = u32::from(u16::from_le_bytes(ch[13]));
                                let v14 = u32::from(u16::from_le_bytes(ch[14]));
                                let v15 = u32::from(u16::from_le_bytes(ch[15]));
                                let u0 = dst_start + i;
                                let eu = &mut edges[u0..u0 + 16];
                                eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                                eu[4] ^= pack | v4; eu[5] ^= pack | v5; eu[6] ^= pack | v6; eu[7] ^= pack | v7;
                                eu[8] ^= pack | v8; eu[9] ^= pack | v9; eu[10] ^= pack | v10; eu[11] ^= pack | v11;
                                eu[12] ^= pack | v12; eu[13] ^= pack | v13; eu[14] ^= pack | v14; eu[15] ^= pack | v15;
                                edges[v0 as usize] ^= u0 as u32; edges[v1 as usize] ^= (u0 + 1) as u32;
                                edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                                edges[v4 as usize] ^= (u0 + 4) as u32; edges[v5 as usize] ^= (u0 + 5) as u32;
                                edges[v6 as usize] ^= (u0 + 6) as u32; edges[v7 as usize] ^= (u0 + 7) as u32;
                                edges[v8 as usize] ^= (u0 + 8) as u32; edges[v9 as usize] ^= (u0 + 9) as u32;
                                edges[v10 as usize] ^= (u0 + 10) as u32; edges[v11 as usize] ^= (u0 + 11) as u32;
                                edges[v12 as usize] ^= (u0 + 12) as u32; edges[v13 as usize] ^= (u0 + 13) as u32;
                                edges[v14 as usize] ^= (u0 + 14) as u32; edges[v15 as usize] ^= (u0 + 15) as u32;
                                i += 16;
                            }
                        }
                        let (chunks8, _) = pairs[i..].as_chunks::<8>();
                        for ch in chunks8 {
                            let v0 = u32::from(u16::from_le_bytes(ch[0]));
                            let v1 = u32::from(u16::from_le_bytes(ch[1]));
                            let v2 = u32::from(u16::from_le_bytes(ch[2]));
                            let v3 = u32::from(u16::from_le_bytes(ch[3]));
                            let v4 = u32::from(u16::from_le_bytes(ch[4]));
                            let v5 = u32::from(u16::from_le_bytes(ch[5]));
                            let v6 = u32::from(u16::from_le_bytes(ch[6]));
                            let v7 = u32::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                            eu[4] ^= pack | v4; eu[5] ^= pack | v5; eu[6] ^= pack | v6; eu[7] ^= pack | v7;
                            edges[v0 as usize] ^= u0 as u32; edges[v1 as usize] ^= (u0 + 1) as u32;
                            edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                            edges[v4 as usize] ^= (u0 + 4) as u32; edges[v5 as usize] ^= (u0 + 5) as u32;
                            edges[v6 as usize] ^= (u0 + 6) as u32; edges[v7 as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let v = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack | v; edges[v as usize] ^= u as u32;
                            i += 1;
                        }
                    } else {
                        let pack_hi = pack | hi;
                        if S4_MODE == 2 {
                            let (chunks16, _) = pairs.as_chunks::<16>();
                            for ch in chunks16 {
                                let l0 = u32::from(u16::from_le_bytes(ch[0]));
                                let l1 = u32::from(u16::from_le_bytes(ch[1]));
                                let l2 = u32::from(u16::from_le_bytes(ch[2]));
                                let l3 = u32::from(u16::from_le_bytes(ch[3]));
                                let l4 = u32::from(u16::from_le_bytes(ch[4]));
                                let l5 = u32::from(u16::from_le_bytes(ch[5]));
                                let l6 = u32::from(u16::from_le_bytes(ch[6]));
                                let l7 = u32::from(u16::from_le_bytes(ch[7]));
                                let l8 = u32::from(u16::from_le_bytes(ch[8]));
                                let l9 = u32::from(u16::from_le_bytes(ch[9]));
                                let l10 = u32::from(u16::from_le_bytes(ch[10]));
                                let l11 = u32::from(u16::from_le_bytes(ch[11]));
                                let l12 = u32::from(u16::from_le_bytes(ch[12]));
                                let l13 = u32::from(u16::from_le_bytes(ch[13]));
                                let l14 = u32::from(u16::from_le_bytes(ch[14]));
                                let l15 = u32::from(u16::from_le_bytes(ch[15]));
                                let u0 = dst_start + i;
                                let eu = &mut edges[u0..u0 + 16];
                                eu[0] ^= pack_hi | l0; eu[1] ^= pack_hi | l1; eu[2] ^= pack_hi | l2; eu[3] ^= pack_hi | l3;
                                eu[4] ^= pack_hi | l4; eu[5] ^= pack_hi | l5; eu[6] ^= pack_hi | l6; eu[7] ^= pack_hi | l7;
                                eu[8] ^= pack_hi | l8; eu[9] ^= pack_hi | l9; eu[10] ^= pack_hi | l10; eu[11] ^= pack_hi | l11;
                                eu[12] ^= pack_hi | l12; eu[13] ^= pack_hi | l13; eu[14] ^= pack_hi | l14; eu[15] ^= pack_hi | l15;
                                edges[(hi | l0) as usize] ^= u0 as u32; edges[(hi | l1) as usize] ^= (u0 + 1) as u32;
                                edges[(hi | l2) as usize] ^= (u0 + 2) as u32; edges[(hi | l3) as usize] ^= (u0 + 3) as u32;
                                edges[(hi | l4) as usize] ^= (u0 + 4) as u32; edges[(hi | l5) as usize] ^= (u0 + 5) as u32;
                                edges[(hi | l6) as usize] ^= (u0 + 6) as u32; edges[(hi | l7) as usize] ^= (u0 + 7) as u32;
                                edges[(hi | l8) as usize] ^= (u0 + 8) as u32; edges[(hi | l9) as usize] ^= (u0 + 9) as u32;
                                edges[(hi | l10) as usize] ^= (u0 + 10) as u32; edges[(hi | l11) as usize] ^= (u0 + 11) as u32;
                                edges[(hi | l12) as usize] ^= (u0 + 12) as u32; edges[(hi | l13) as usize] ^= (u0 + 13) as u32;
                                edges[(hi | l14) as usize] ^= (u0 + 14) as u32; edges[(hi | l15) as usize] ^= (u0 + 15) as u32;
                                i += 16;
                            }
                        }
                        let (chunks8, _) = pairs[i..].as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = u32::from(u16::from_le_bytes(ch[0]));
                            let l1 = u32::from(u16::from_le_bytes(ch[1]));
                            let l2 = u32::from(u16::from_le_bytes(ch[2]));
                            let l3 = u32::from(u16::from_le_bytes(ch[3]));
                            let l4 = u32::from(u16::from_le_bytes(ch[4]));
                            let l5 = u32::from(u16::from_le_bytes(ch[5]));
                            let l6 = u32::from(u16::from_le_bytes(ch[6]));
                            let l7 = u32::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | l0; eu[1] ^= pack_hi | l1; eu[2] ^= pack_hi | l2; eu[3] ^= pack_hi | l3;
                            eu[4] ^= pack_hi | l4; eu[5] ^= pack_hi | l5; eu[6] ^= pack_hi | l6; eu[7] ^= pack_hi | l7;
                            edges[(hi | l0) as usize] ^= u0 as u32; edges[(hi | l1) as usize] ^= (u0 + 1) as u32;
                            edges[(hi | l2) as usize] ^= (u0 + 2) as u32; edges[(hi | l3) as usize] ^= (u0 + 3) as u32;
                            edges[(hi | l4) as usize] ^= (u0 + 4) as u32; edges[(hi | l5) as usize] ^= (u0 + 5) as u32;
                            edges[(hi | l6) as usize] ^= (u0 + 6) as u32; edges[(hi | l7) as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | lo; edges[(hi | lo) as usize] ^= u as u32;
                            i += 1;
                        }
                    }
                }
                self.at += span;
                self.read += span as u32;
            } else if S4_MODE == 0 {
                let mut dummy = 0u32;
                self.wire_xor_slow(edges, dst_start, span, pack, u32::MAX, &mut dummy);
            } else {
                self.wire_xor_slow_nomark(edges, dst_start, span, pack);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_slow_nomark(&mut self, edges: &mut [u32], mut dst_start: usize, mut left: usize, pack: u32) {
            while left > 0 {
                while self.read == self.cut { self.epoch += 1; self.cut = self.cut_at(self.epoch as usize); }
                if self.at == Self::CHUNK { self.id = self.next_chunk(self.id as usize); self.at = 0; }
                let take = left.min(Self::CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES; &self.out[at..at + BYTES]
                } else { &self.spill.ovf[(self.id - self.in_out) as usize][..] };
                let hi = self.epoch << 16;
                let pack_hi = pack | hi;
                let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                let mut i = 0usize;
                let (chunks8, _) = pairs.as_chunks::<8>();
                for ch in chunks8 {
                    let l0 = u32::from(u16::from_le_bytes(ch[0]));
                    let l1 = u32::from(u16::from_le_bytes(ch[1]));
                    let l2 = u32::from(u16::from_le_bytes(ch[2]));
                    let l3 = u32::from(u16::from_le_bytes(ch[3]));
                    let l4 = u32::from(u16::from_le_bytes(ch[4]));
                    let l5 = u32::from(u16::from_le_bytes(ch[5]));
                    let l6 = u32::from(u16::from_le_bytes(ch[6]));
                    let l7 = u32::from(u16::from_le_bytes(ch[7]));
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 8];
                    eu[0] ^= pack_hi | l0; eu[1] ^= pack_hi | l1; eu[2] ^= pack_hi | l2; eu[3] ^= pack_hi | l3;
                    eu[4] ^= pack_hi | l4; eu[5] ^= pack_hi | l5; eu[6] ^= pack_hi | l6; eu[7] ^= pack_hi | l7;
                    edges[(hi | l0) as usize] ^= u0 as u32; edges[(hi | l1) as usize] ^= (u0 + 1) as u32;
                    edges[(hi | l2) as usize] ^= (u0 + 2) as u32; edges[(hi | l3) as usize] ^= (u0 + 3) as u32;
                    edges[(hi | l4) as usize] ^= (u0 + 4) as u32; edges[(hi | l5) as usize] ^= (u0 + 5) as u32;
                    edges[(hi | l6) as usize] ^= (u0 + 6) as u32; edges[(hi | l7) as usize] ^= (u0 + 7) as u32;
                    i += 8;
                }
                for &p in &pairs[i..] {
                    let u = dst_start + i;
                    let lo = u32::from(u16::from_le_bytes(p));
                    edges[u] ^= pack_hi | lo; edges[(hi | lo) as usize] ^= u as u32;
                    i += 1;
                }
                self.at += take; self.read += take as u32; dst_start += take; left -= take;
            }
        }

        #[inline(always)]
        pub fn wire_xor(&mut self, edges: &mut [u32], dst_start: usize, span: usize, pack: u32, mark: u32, predecessor: &mut u32) {
            if self.at + span <= Self::CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                let mut before = *predecessor;
                for (i, &s) in pairs.iter().enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes(s));
                    if v == mark { before = u as u32; }
                    edges[u] ^= pack | v; edges[v as usize] ^= u as u32;
                }
                *predecessor = before;
                self.at += span; self.read += span as u32;
            } else {
                self.wire_xor_slow(edges, dst_start, span, pack, mark, predecessor);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_slow(&mut self, edges: &mut [u32], mut dst_start: usize, mut left: usize, pack: u32, mark: u32, predecessor: &mut u32) {
            while left > 0 {
                while self.read == self.cut { self.epoch += 1; self.cut = self.cut_at(self.epoch as usize); }
                if self.at == Self::CHUNK { self.id = self.next_chunk(self.id as usize); self.at = 0; }
                let take = left.min(Self::CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES; &self.out[at..at + BYTES]
                } else { &self.spill.ovf[(self.id - self.in_out) as usize][..] };
                let hi = self.epoch << 16;
                let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                for (i, &s) in pairs.iter().enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes(s));
                    if v == mark { *predecessor = u as u32; }
                    edges[u] ^= pack | v; edges[v as usize] ^= u as u32;
                }
                self.at += take; self.read += take as u32; dst_start += take; left -= take;
            }
        }

        #[inline(always)]
        pub fn wire(&mut self, dst: &mut [u32], pack: u32) {
            let span = dst.len();
            if self.at + span <= Self::CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = pack | (self.epoch << 16);
                let at = self.id as usize * BYTES + 2 * self.at;
                let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                for (e, &s) in dst.iter_mut().zip(pairs) { *e = hi | u32::from(u16::from_le_bytes(s)); }
                self.at += span; self.read += span as u32;
            } else {
                self.wire_slow(dst, pack);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_slow(&mut self, mut dst: &mut [u32], pack: u32) {
            while !dst.is_empty() {
                while self.read == self.cut { self.epoch += 1; self.cut = self.cut_at(self.epoch as usize); }
                if self.at == Self::CHUNK { self.id = self.next_chunk(self.id as usize); self.at = 0; }
                let take = dst.len().min(Self::CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES; &self.out[at..at + BYTES]
                } else { &self.spill.ovf[(self.id - self.in_out) as usize][..] };
                let hi = pack | (self.epoch << 16);
                let (now, rest) = dst.split_at_mut(take);
                let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                for (e, &s) in now.iter_mut().zip(pairs) { *e = hi | u32::from(u16::from_le_bytes(s)); }
                self.at += take; self.read += take as u32; dst = rest;
            }
        }
    }
}

// ============================================================================
// 3. Unified Parameterized `engine_c53` (`CHUNK` × `DICT_MODE` × `S4_MODE` × `S5_MODE` × `S1_GROW`)
// ============================================================================

mod engine_c53 {
    use super::*;

    trait Links {
        const STRIDE: usize;
        const IS_NARROW: bool;
        const BOTTOM: usize;
        fn prev(a: &[u32], id: u32, side: usize) -> u32;
        fn next(a: &[u32], id: u32, side: usize) -> u32;
        fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
        fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
        fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32);
    }
    struct Narrow;
    struct Wide;
    impl Links for Narrow {
        const STRIDE: usize = 4; const IS_NARROW: bool = true; const BOTTOM: usize = 1;
        #[inline(always)] fn prev(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 4 + side] & 0xFFFF }
        #[inline(always)] fn next(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 4 + side] >> 16 }
        #[inline(always)] fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) { let w = &mut a[id as usize * 4 + side]; *w = (*w & 0xFFFF_0000) | to; }
        #[inline(always)] fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) { let w = &mut a[id as usize * 4 + side]; *w = (*w & 0xFFFF) | (to << 16); }
        #[inline(always)] fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) { let sl = &a[id as usize * 4..id as usize * 4 + 4]; (sl[0] >> 16, sl[1], sl[2], sl[3]) }
    }
    impl Links for Wide {
        const STRIDE: usize = 6; const IS_NARROW: bool = false; const BOTTOM: usize = 2;
        #[inline(always)] fn prev(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side] }
        #[inline(always)] fn next(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side + 1] }
        #[inline(always)] fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side] = to; }
        #[inline(always)] fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side + 1] = to; }
        #[inline(always)] fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) { let sl = &a[id as usize * 6..id as usize * 6 + 6]; (sl[1], sl[2], sl[4], sl[5]) }
    }

    struct Elim<'n, 'r, L: Links> {
        nodes_buf: &'n mut [u32],
        rules_buf: &'r mut [u32],
        rule_top: usize,
        nodes: u32,
        steps: usize,
        id_overflow: bool,
        _links: std::marker::PhantomData<L>,
    }

    impl<L: Links> Elim<'_, '_, L> {
        #[inline(always)] fn width(&self, id: u32) -> u32 { self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 2] }
        #[inline(always)] fn set_width(&mut self, id: u32, w: u32) { self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 2] = w; }
        #[inline(always)] fn word(&self, id: u32) -> u32 { self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 1] }
        #[inline(always)] fn set_word(&mut self, id: u32, w: u32) { self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 1] = w; }
        #[inline(always)] fn set_bottom(&mut self, id: u32, x: u32) { self.nodes_buf[id as usize * L::STRIDE + L::BOTTOM] = x; }

        fn assign_bottom(&mut self, rows: u32) {
            let (mut first, mut last) = (L::next(self.nodes_buf, 0, 1), L::prev(self.nodes_buf, 0, 1));
            let (mut lo, mut hi) = (0, rows);
            while first != last {
                let (next, prev, width) = (L::next(self.nodes_buf, first, 1), L::prev(self.nodes_buf, last, 1), self.width(first));
                hi -= self.width(last); self.set_bottom(first, lo); self.set_bottom(last, hi); lo += width;
                if next == last { assert_eq!(lo, hi); return; }
                first = next; last = prev;
            }
            if first != 0 { self.set_bottom(first, lo); lo += self.width(first); }
            assert_eq!(lo, hi);
        }

        #[inline(always)] fn node(&mut self, width: u32, word: u32) -> u32 { let id = self.nodes; self.nodes += 1; self.set_width(id, width); self.set_word(id, word); id }
        #[inline(always)] fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
            if list.1 == 0 { list.0 = id; } else { L::set_next(self.nodes_buf, list.1, side, id); }
            L::set_prev(self.nodes_buf, id, side, list.1); list.1 = id;
        }
        fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
            let mut prev = 0;
            for (head, tail) in [first, second] { if tail == 0 { continue; } L::set_next(self.nodes_buf, prev, side, head); L::set_prev(self.nodes_buf, head, side, prev); prev = tail; }
            L::set_next(self.nodes_buf, prev, side, 0); L::set_prev(self.nodes_buf, 0, side, prev);
        }
        #[inline(always)] fn cut(&mut self, id: u32, side: usize) { let (p, n) = (L::prev(self.nodes_buf, id, side), L::next(self.nodes_buf, id, side)); L::set_next(self.nodes_buf, p, side, n); L::set_prev(self.nodes_buf, n, side, p); }
        #[inline(always)] fn pop_tail(&mut self, id: u32, side: usize) { let prev = L::prev(self.nodes_buf, id, side); L::set_next(self.nodes_buf, prev, side, 0); L::set_prev(self.nodes_buf, 0, side, prev); }
        #[inline(always)] fn insert(&mut self, at: u32, id: u32, side: usize) {
            let next = L::next(self.nodes_buf, at, side);
            L::set_prev(self.nodes_buf, id, side, at); L::set_next(self.nodes_buf, id, side, next);
            L::set_next(self.nodes_buf, at, side, id); L::set_prev(self.nodes_buf, next, side, id);
        }
        #[inline(always)] fn len_handle(&self, h: u32) -> u32 { let g = h >> ID_BITS; if g < SATURATED { g } else { self.rules_buf[(h & ID_MASK) as usize + 2] } }

        #[inline(always)]
        fn short_payload_h(&self, h: u32) -> u128 {
            let (g, id) = (h >> ID_BITS, h & ID_MASK);
            if g <= 3 { return u128::from(id); }
            let r = id as usize;
            let sl = &self.rules_buf[r..r + 4];
            let raw = u128::from(sl[0]) | (u128::from(sl[1]) << 32) | (u128::from(sl[2]) << 64) | (u128::from(sl[3]) << 96);
            raw & SHORT_MASK[(g as usize) & 15]
        }

        #[inline(always)]
        fn join(&mut self, left: u32, right: u32) -> u32 {
            if left == 0 { return right; }
            if right == 0 { return left; }
            let ll = left >> ID_BITS;
            let sum = ll + (right >> ID_BITS);
            if sum <= 3 { return (sum << ID_BITS) | (left & ID_MASK) | ((right & ID_MASK) << (8 * ll)); }
            let t = self.rule_top;
            let id = t as u32;
            if sum < INLINE as u32 {
                let bytes = self.short_payload_h(left) | (self.short_payload_h(right) << (8 * ll));
                let out = &mut self.rules_buf[t..t + 4];
                out[0] = bytes as u32; out[1] = (bytes >> 32) as u32;
                if sum <= 8 { self.rule_top = t + 2; }
                else if sum <= 12 { out[2] = (bytes >> 64) as u32; self.rule_top = t + 3; }
                else { out[2] = (bytes >> 64) as u32; out[3] = (bytes >> 96) as u32; self.rule_top = t + 4; }
                (sum << ID_BITS) | id
            } else if sum < SATURATED {
                let out = &mut self.rules_buf[t..t + 3];
                out[0] = left; out[1] = right; out[2] = u32::MAX;
                self.rule_top = t + 3;
                (sum << ID_BITS) | id
            } else {
                let len = if sum == SATURATED { sum } else { self.len_handle(left) + self.len_handle(right) };
                let out = &mut self.rules_buf[t..t + 4];
                out[0] = left; out[1] = right; out[2] = len; out[3] = u32::MAX;
                self.rule_top = t + 4;
                (SATURATED << ID_BITS) | id
            }
        }

        fn power(&mut self, word: u32, q: u32) -> u32 {
            let (mut acc, mut base, mut e) = (0, word, q);
            loop { if e & 1 == 1 { acc = self.join(acc, base); } e >>= 1; if e == 0 { break acc; } base = self.join(base, base); }
        }

        #[inline(never)]
        fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
            let mut left = *remaining;
            let max_id_top = ID_MASK as usize + 1;
            let done = loop {
                if left == 0 { break true; }
                if self.rule_top + MAX_RULE_WORDS > self.rules_buf.len() { break false; }
                if self.rule_top + MAX_RULE_WORDS > max_id_top { self.id_overflow = true; break false; }
                self.steps += 1;
                let sent = &self.nodes_buf[..2];
                let (x, y) = (sent[0] & 0xFFFF, sent[1] & 0xFFFF);
                let (xi, yi) = (x as usize * 4, y as usize * 4);
                let nx = &self.nodes_buf[xi..xi + 4];
                let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
                if x == y {
                    assert!(aw > 0 && aw <= left);
                    left -= aw;
                    let (px0, px1) = (x0 & 0xFFFF, x1 & 0xFFFF);
                    self.nodes_buf[px0 as usize * 4] &= 0xFFFF; self.nodes_buf[0] = (self.nodes_buf[0] & 0xFFFF_0000) | px0;
                    self.nodes_buf[px1 as usize * 4 + 1] &= 0xFFFF; self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | px1;
                    continue;
                }
                let ny = &self.nodes_buf[yi..yi + 4];
                let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
                let w = aw.min(bw); assert!(w > 0 && w <= left); left -= w;
                let word = self.join(wx, wy);
                if aw <= bw {
                    let px = x0 & 0xFFFF;
                    self.nodes_buf[px as usize * 4] &= 0xFFFF; self.nodes_buf[0] = (self.nodes_buf[0] & 0xFFFF_0000) | px;
                    if aw < bw {
                        let y_next = if y == px { 0 } else { y0 >> 16 };
                        let nxm = &mut self.nodes_buf[xi..xi + 4]; nxm[0] = y | (y_next << 16); nxm[3] = word;
                        let nym = &mut self.nodes_buf[yi..yi + 4]; nym[0] = (y0 & 0xFFFF) | (x << 16); nym[2] = bw - aw;
                        let yni = y_next as usize * 4; self.nodes_buf[yni] = (self.nodes_buf[yni] & 0xFFFF_0000) | x;
                    } else {
                        let y_next = if y == px { 0 } else { y0 >> 16 }; let y_prev = y0 & 0xFFFF;
                        let nxm = &mut self.nodes_buf[xi..xi + 4]; nxm[0] = y_prev | (y_next << 16); nxm[3] = word;
                        let ypi = y_prev as usize * 4; self.nodes_buf[ypi] = (self.nodes_buf[ypi] & 0xFFFF) | (x << 16);
                        let yni = y_next as usize * 4; self.nodes_buf[yni] = (self.nodes_buf[yni] & 0xFFFF_0000) | x;
                        let py = y1 & 0xFFFF; self.nodes_buf[py as usize * 4 + 1] &= 0xFFFF; self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | py;
                    }
                } else {
                    let py = y1 & 0xFFFF; self.nodes_buf[py as usize * 4 + 1] &= 0xFFFF; self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | py;
                    let x_next = if x == py { 0 } else { x1 >> 16 };
                    let nym = &mut self.nodes_buf[yi..yi + 4]; nym[1] = x | (x_next << 16); nym[3] = word;
                    let nxm = &mut self.nodes_buf[xi..xi + 4]; nxm[1] = (x1 & 0xFFFF) | (y << 16); nxm[2] = aw - bw;
                    let xni = x_next as usize * 4 + 1; self.nodes_buf[xni] = (self.nodes_buf[xni] & 0xFFFF_0000) | y;
                }
            };
            *remaining = left; done
        }

        fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
            if !SPIRALS && L::IS_NARROW { return self.eliminate_narrow_plain(remaining); }
            let mut left = *remaining;
            let room = if SPIRALS { SPIRAL_RULES * MAX_RULE_WORDS } else { MAX_RULE_WORDS };
            let max_id_top = ID_MASK as usize + 1;
            let done = loop {
                if left == 0 { break true; }
                if self.rule_top + room > self.rules_buf.len() { break false; }
                if self.rule_top + room > max_id_top { self.id_overflow = true; break false; }
                self.steps += 1;
                let (x, y) = (L::prev(self.nodes_buf, 0, 0), L::prev(self.nodes_buf, 0, 1));
                let (aw, bw) = (self.width(x), self.width(y));
                if SPIRALS && x != y {
                    if aw > bw && L::prev(self.nodes_buf, y, 1) == x && (aw - 1) / bw >= 2 {
                        let q = (aw - 1) / bw;
                        let head = self.power(self.word(x), q);
                        let word = self.join(head, self.word(y));
                        self.set_word(y, word); self.set_width(x, aw - q * bw); left -= q * bw; continue;
                    }
                    if aw < bw && L::prev(self.nodes_buf, x, 0) == y && (bw - 1) / aw >= 2 {
                        let q = (bw - 1) / aw;
                        let tail = self.power(self.word(y), q);
                        let word = self.join(self.word(x), tail);
                        self.set_word(x, word); self.set_width(y, bw - q * aw); left -= q * aw; continue;
                    }
                }
                let w = aw.min(bw); assert!(w > 0 && w <= left); left -= w;
                if x == y { self.pop_tail(x, 0); self.pop_tail(x, 1); continue; }
                let word = self.join(self.word(x), self.word(y));
                if aw <= bw {
                    self.pop_tail(x, 0); self.insert(y, x, 0); self.set_word(x, word);
                    if aw == bw { self.cut(y, 0); self.pop_tail(y, 1); } else { self.set_width(y, bw - w); }
                } else {
                    self.pop_tail(y, 1); self.insert(x, y, 1); self.set_word(y, word); self.set_width(x, aw - w);
                }
            };
            *remaining = left; done
        }

        #[allow(clippy::too_many_arguments)]
        fn inline_word(&mut self, word: u32, byte: u8, istk: &mut [u32; INLINE_STACK], memo_rec: &mut [u32; MEMO_REC_CAP], stack: &mut Vec<u32>, spelling: &mut Vec<u8>) -> ([u8; INLINE], u8) {
            let height = (word >> ID_BITS) as usize;
            if height < INLINE {
                let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
                (v.to_le_bytes(), (height + 1) as u8)
            } else {
                let start = spelling.len() as u32;
                let total = self.len_handle(word) as usize;
                ensure_step_u8(spelling, total + 1, SPELLING_STEP);
                let (mut dn, mut dovf) = (0usize, false);
                self.append_tail_unw::<false>(word, istk, stack, spelling, memo_rec, &mut dn, &mut dovf);
                spelling.push(byte);
                (long_slot(start, spelling.len() as u32), LONG)
            }
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn append_tail_unw<const TRACK_MEMO: bool>(&mut self, handle: u32, istk: &mut [u32; INLINE_STACK], ovf_stack: &mut Vec<u32>, dest: &mut Vec<u8>, memo_rec: &mut [u32; MEMO_REC_CAP], memo_n: &mut usize, memo_ovf: &mut bool) {
            let (mut sp, mut used_heap, mut h) = (0usize, false, handle);
            loop {
                let g = h >> ID_BITS;
                if g < INLINE as u32 {
                    let bytes = self.short_payload_h(h).to_le_bytes();
                    dest.extend_from_slice(&bytes[..g as usize]);
                    if used_heap && !ovf_stack.is_empty() { h = ovf_stack.pop().unwrap(); continue; }
                    if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                    return;
                }
                let r = (h & ID_MASK) as usize;
                let m_off = if g < SATURATED { r + 2 } else { r + 3 };
                let tag = self.rules_buf[m_off];
                if tag != u32::MAX {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = if g < SATURATED { g as usize } else { self.rules_buf[r + 2] as usize };
                    dest.extend_from_within(start..start + rlen);
                    if used_heap && !ovf_stack.is_empty() { h = ovf_stack.pop().unwrap(); continue; }
                    if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                    return;
                }
                self.rules_buf[m_off] = 0x8000_0000 | (dest.len() as u32);
                if TRACK_MEMO {
                    if *memo_n < MEMO_REC_CAP { memo_rec[*memo_n] = m_off as u32; *memo_n += 1; } else { *memo_ovf = true; }
                }
                let (left, right) = (self.rules_buf[r], self.rules_buf[r + 1]);
                let lg = left >> ID_BITS;
                if lg < INLINE as u32 {
                    let lb = self.short_payload_h(left).to_le_bytes();
                    dest.extend_from_slice(&lb[..lg as usize]);
                    let rg = right >> ID_BITS;
                    if rg < INLINE as u32 {
                        let rb = self.short_payload_h(right).to_le_bytes();
                        dest.extend_from_slice(&rb[..rg as usize]);
                        if used_heap && !ovf_stack.is_empty() { h = ovf_stack.pop().unwrap(); continue; }
                        if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                        return;
                    }
                    h = right;
                } else {
                    if sp < INLINE_STACK { istk[sp] = right; sp += 1; }
                    else { if !used_heap { ovf_stack.clear(); used_heap = true; } ovf_stack.push(right); }
                    h = left;
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // Dictionary supporting 4 `contents` modes:
    //   DICT_MODE = 0: `[u32; 2048]` in `edges` cache (`8 KiB`)
    //   DICT_MODE = 1: `[u16; 2048]` on stack (`4 KiB`, raw `code < 65_534`, exact `u32` fallback)
    //   DICT_MODE = 2: `[u16; 2048]` on stack (`4 KiB`, 4-bit tag + 12-bit `code < 4_096`, exact `u32` fallback)
    //   DICT_MODE = 3: `[u32; 1024]` packed `2×u16` in `edges` cache (`4 KiB`, 4-bit tag, exact `u32` fallback)
    // ------------------------------------------------------------------------
    enum ContentsStore<'a> {
        U32(&'a mut [u32; 2048]),
        U16Raw { tbl: &'a mut [u16; 2048], fb: &'a mut Vec<u32>, cap: u32 },
        U16Tag4 { tbl: &'a mut [u16; 2048], fb: &'a mut Vec<u32>, cap: u32 },
        U16Packed { tbl: &'a mut [u32; 1024], fb: &'a mut Vec<u32>, cap: u32 },
    }

    impl ContentsStore<'_> {
        #[inline(always)]
        fn probe(&self, index: usize, h: u64) -> Option<usize> {
            match self {
                ContentsStore::U32(tbl) => {
                    let tag = (((h >> 56) as u32) | 0x80) << 24;
                    let ent = tbl[index];
                    if (ent & 0xFF00_0000) == tag { Some((ent & 0x00FF_FFFF) as usize) } else { None }
                }
                ContentsStore::U16Raw { tbl, fb, .. } => {
                    if fb.is_empty() {
                        let ent = tbl[index];
                        if ent != 0 { Some(ent as usize) } else { None }
                    } else {
                        let ent = fb[index];
                        if ent != 0 { Some(ent as usize) } else { None }
                    }
                }
                ContentsStore::U16Tag4 { tbl, fb, .. } => {
                    if fb.is_empty() {
                        let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
                        let ent = tbl[index];
                        if (ent & 0xF000) == tag4 { Some((ent & 0x0FFF) as usize) } else { None }
                    } else {
                        let ent = fb[index];
                        if ent != 0 { Some(ent as usize) } else { None }
                    }
                }
                ContentsStore::U16Packed { tbl, fb, .. } => {
                    if fb.is_empty() {
                        let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
                        let w = tbl[index >> 1];
                        let ent = if (index & 1) == 0 { w as u16 } else { (w >> 16) as u16 };
                        if (ent & 0xF000) == tag4 { Some((ent & 0x0FFF) as usize) } else { None }
                    } else {
                        let ent = fb[index];
                        if ent != 0 { Some(ent as usize) } else { None }
                    }
                }
            }
        }

        #[inline(always)]
        fn store(&mut self, index: usize, h: u64, code: u32) {
            match self {
                ContentsStore::U32(tbl) => {
                    let tag = (((h >> 56) as u32) | 0x80) << 24;
                    tbl[index] = tag | code;
                }
                ContentsStore::U16Raw { tbl, fb, cap } => {
                    if fb.is_empty() && code < *cap {
                        tbl[index] = code as u16;
                    } else {
                        Self::store_u32_fallback_raw(tbl, fb, index, code);
                    }
                }
                ContentsStore::U16Tag4 { tbl, fb, cap } => {
                    if fb.is_empty() && code < *cap {
                        let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
                        tbl[index] = tag4 | (code as u16);
                    } else {
                        Self::store_u32_fallback_tag4(tbl, fb, index, code);
                    }
                }
                ContentsStore::U16Packed { tbl, fb, cap } => {
                    if fb.is_empty() && code < *cap {
                        let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
                        let ent = u32::from(tag4 | (code as u16));
                        let slot = &mut tbl[index >> 1];
                        if (index & 1) == 0 { *slot = (*slot & 0xFFFF_0000) | ent; }
                        else { *slot = (*slot & 0xFFFF) | (ent << 16); }
                    } else {
                        Self::store_u32_fallback_packed(tbl, fb, index, code);
                    }
                }
            }
        }

        #[cold]
        #[inline(never)]
        fn store_u32_fallback_raw(tbl: &[u16; 2048], fb: &mut Vec<u32>, index: usize, code: u32) {
            if fb.is_empty() {
                fb.reserve_exact(2048);
                fb.extend(tbl.iter().map(|&v| u32::from(v)));
            }
            fb[index] = code;
        }

        #[cold]
        #[inline(never)]
        fn store_u32_fallback_tag4(tbl: &[u16; 2048], fb: &mut Vec<u32>, index: usize, code: u32) {
            if fb.is_empty() {
                fb.reserve_exact(2048);
                fb.extend(tbl.iter().map(|&v| if v != 0 { u32::from(v & 0x0FFF) } else { 0 }));
            }
            fb[index] = code;
        }

        #[cold]
        #[inline(never)]
        fn store_u32_fallback_packed(tbl: &[u32; 1024], fb: &mut Vec<u32>, index: usize, code: u32) {
            if fb.is_empty() {
                fb.reserve_exact(2048);
                for &w in tbl.iter() {
                    let lo = w as u16;
                    let hi = (w >> 16) as u16;
                    fb.push(if lo != 0 { u32::from(lo & 0x0FFF) } else { 0 });
                    fb.push(if hi != 0 { u32::from(hi & 0x0FFF) } else { 0 });
                }
            }
            fb[index] = code;
        }
    }

    struct Dictionary<'a> {
        pairs: &'a mut [u32; 512],
        contents: ContentsStore<'a>,
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        spelling: &'a mut Vec<u8>,
        istk: &'a mut [u32; INLINE_STACK],
        memo_rec: &'a mut [u32; MEMO_REC_CAP],
        stack: &'a mut Vec<u32>,
    }

    impl Dictionary<'_> {
        #[inline(always)]
        fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
            let bucket = ((handle.wrapping_mul(0x9e37_79b9) ^ (u32::from(byte).wrapping_mul(0x85eb_ca6b))) >> 24) as usize;
            let slot = bucket * 2;
            let val = self.pairs[slot + 1];
            if self.pairs[slot] == handle && (val >> 24) == u32::from(byte) {
                return val & 0x00FF_FFFF;
            }
            let code = self.code_miss(e, handle, handle & ID_MASK, byte);
            self.pairs[slot] = handle;
            self.pairs[slot + 1] = (u32::from(byte) << 24) | code;
            code
        }

        fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8) -> u32 {
            let height = (handle >> ID_BITS) as usize;
            if height < INLINE {
                let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
                let entry = v.to_le_bytes();
                let len = (height + 1) as u8;
                let h = ((v as u64) ^ ((v >> 64) as u64).rotate_left(29) ^ (u64::from(len) << 56))
                    .wrapping_mul(0xBF58_476D_1CE4_E5B9);
                let h = h ^ (h >> 30);
                let index = (h as usize) & 2047;
                if let Some(old) = self.contents.probe(index, h) {
                    if self.lens[old] == len && self.inline16[old] == entry {
                        return old as u32;
                    }
                }
                let c = self.insert(entry, len);
                self.contents.store(index, h, c);
                return c;
            }
            let start = self.spelling.len();
            let g = handle >> ID_BITS;
            let r = id as usize;
            let m_off = if g < SATURATED { r + 2 } else { r + 3 };
            let tag_r = e.rules_buf[m_off];
            let memoized = tag_r != u32::MAX;
            let rlen = if g < SATURATED { g as usize } else { e.rules_buf[r + 2] as usize };
            ensure_step_u8(self.spelling, rlen + 1, SPELLING_STEP);
            let (mut memo_n, mut memo_ovf) = (0usize, false);
            if memoized {
                let pos = (tag_r & 0x7fff_ffff) as usize;
                self.spelling.extend_from_within(pos..pos + rlen);
            } else {
                e.append_tail_unw::<true>(handle, self.istk, self.stack, self.spelling, self.memo_rec, &mut memo_n, &mut memo_ovf);
            }
            self.spelling.push(byte);
            let entry = long_slot(start as u32, self.spelling.len() as u32);
            let bytes = &self.spelling[start..];
            let h = spelling_hash_long(bytes);
            let index = (h as usize) & 2047;
            if let Some(old) = self.contents.probe(index, h) {
                if self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes {
                    if memoized {
                        self.spelling.truncate(start);
                    } else if !memo_ovf {
                        let old_start = u32::from_le_bytes(self.inline16[old][..4].try_into().unwrap());
                        let delta = (start as u32) - old_start;
                        for &rm in &self.memo_rec[..memo_n] { e.rules_buf[rm as usize] -= delta; }
                        self.spelling.truncate(start);
                    } else {
                        self.spelling.pop();
                    }
                    return old as u32;
                }
            }
            let c = self.insert(entry, LONG);
            self.contents.store(index, h, c);
            c
        }

        #[inline(always)]
        fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
            ensure_step_inline(self.inline16, self.lens, INLINE_STEP);
            let code = self.lens.len() as u32;
            self.inline16.push(entry);
            self.lens.push(len);
            code
        }
    }

    struct Pass<'a, const LOG_CHUNK: u32, const BYTES: usize, const S1_GROW: u8> {
        queues: outq_gen::Queues<'a, LOG_CHUNK, BYTES>,
        edges: &'a mut Vec<u32>,
        ns: usize,
        cap_strips: usize,
        f: First,
    }

    impl<const LOG_CHUNK: u32, const BYTES: usize, const S1_GROW: u8> Pass<'_, LOG_CHUNK, BYTES, S1_GROW> {
        #[inline(always)]
        fn take<const LIM: u32>(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            if width <= LIM { self.piece(c, pos, width, true, p); return; }
            if p.wrapping_sub(pos) >= width { self.piece(c, pos, width, false, p); }
            else { self.take_marked(c, pos, width, p); }
        }
        #[cold]
        #[inline(never)]
        fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            let end = pos + width;
            if pos < p { self.piece(c, pos, p - pos, false, p); }
            self.piece(c, p, 1, true, p);
            if p + 1 < end { self.piece(c, p + 1, end - p - 1, false, p); }
        }
        #[inline(always)]
        fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
            let a = usize::from(c);
            let f = &mut self.f;
            if keep {
                if p.wrapping_sub(x) < w { f.mark = f.m + (p - x); f.mark_byte = c; }
                self.queues.push_thin(a, f.m, w);
                f.m += w;
            } else {
                let at = 3 * self.ns;
                if self.ns < self.cap_strips {
                    if self.edges.len() < at + 3 {
                        if S1_GROW == 0 {
                            let need = (at + 3).max(2 * self.edges.len()).max(1024);
                            self.edges.reserve_exact(need - self.edges.len());
                            self.edges.resize(need, 0);
                        } else {
                            grow_edges_s1_step(self.edges, at + 3);
                        }
                    }
                    let thin_a = self.queues.thin_of(a);
                    self.edges[at..at + 3].copy_from_slice(&[x, thin_a + f.thick_rows[a], (w << 8) | u32::from(c)]);
                }
                self.ns += 1; f.thick[a] += 1; f.thick_rows[a] += w;
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn grow_edges_s1_step(a: &mut Vec<u32>, need: usize) {
        let target = need.div_ceil(4096) * 4096;
        if target > a.capacity() {
            a.reserve_exact(target - a.len());
        }
        a.resize(target, 0);
    }

    #[cold]
    #[inline(never)]
    fn emit_long_front(slot: &[u8; INLINE], spelling: &[u8], rem: &mut [u8]) -> usize {
        let word = long_word(slot, spelling);
        let take = word.len().min(rem.len());
        rem[..take].copy_from_slice(&word[..take]);
        take
    }

    #[cold]
    #[inline(never)]
    fn emit_long_back(slot: &[u8; INLINE], spelling: &[u8], rem: &mut [u8]) -> usize {
        let word = long_word(slot, spelling);
        let rlen = rem.len();
        let take = word.len().min(rlen);
        rem[rlen - take..].copy_from_slice(&word[word.len() - take..]);
        take
    }

    #[cold]
    #[inline(never)]
    fn emit_tail_back(slot: &[u8; INLINE], lb: u8, spelling: &[u8], rem: &mut [u8]) -> usize {
        let word = if lb == LONG { long_word(slot, spelling) } else { &slot[..usize::from(lb)] };
        let rlen = rem.len();
        let take = word.len().min(rlen);
        rem[rlen - take..].copy_from_slice(&word[word.len() - take..]);
        take
    }

    pub struct Kernel<const LOG_CHUNK: u32, const BYTES: usize, const DICT_MODE: u8, const S4_MODE: u8, const S5_MODE: u8, const S1_GROW: u8> {
        spill: outq_gen::Spill<BYTES>,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        stack: Vec<u32>,
        contents_u32_fb: Vec<u32>,
        predecessor: Option<u32>,
        pub force_band_vec: bool,
        pub force_sentinel_cap: Option<u32>,
        pub force_u16_cap: Option<u32>,
    }

    impl<const LOG_CHUNK: u32, const BYTES: usize, const DICT_MODE: u8, const S4_MODE: u8, const S5_MODE: u8, const S1_GROW: u8> Default
        for Kernel<LOG_CHUNK, BYTES, DICT_MODE, S4_MODE, S5_MODE, S1_GROW>
    {
        fn default() -> Self {
            Self {
                spill: outq_gen::Spill::default(),
                bands: Vec::new(),
                spelling: Vec::new(),
                edges: Vec::new(),
                extra: Vec::new(),
                inline16: Vec::new(),
                lens: Vec::new(),
                stack: Vec::new(),
                contents_u32_fb: Vec::new(),
                predecessor: None,
                force_band_vec: false,
                force_sentinel_cap: None,
                force_u16_cap: None,
            }
        }
    }

    impl<const LOG_CHUNK: u32, const BYTES: usize, const DICT_MODE: u8, const S4_MODE: u8, const S5_MODE: u8, const S1_GROW: u8>
        Kernel<LOG_CHUNK, BYTES, DICT_MODE, S4_MODE, S5_MODE, S1_GROW>
    {
        pub fn scratch_report(&self) -> (usize, usize, String) {
            let sp = self.spill.capacity_bytes();
            let bd = 8 * self.bands.capacity();
            let s = self.spelling.capacity();
            let ed = 4 * self.edges.capacity();
            let ex = 4 * self.extra.capacity() + 4 * self.contents_u32_fb.capacity();
            let il = 16 * self.inline16.capacity() + self.lens.capacity();
            let st = 4 * self.stack.capacity();
            let tot = sp + bd + s + ed + ex + il + st;
            (tot, ed, format!("ed={:.1}K sp={:.1}K il={:.1}K q={:.1}K fb={:.1}K", ed as f64 / 1024.0, s as f64 / 1024.0, il as f64 / 1024.0, sp as f64 / 1024.0, ex as f64 / 1024.0))
        }

        #[inline(never)]
        pub fn invert_tiny_fl(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
            if n == 0 || (p as usize) >= n || records.len() > 16_000_000 { return false; }
            if records.len() == 1 {
                if usize::from(records[0] & 255) != n { return false; }
                out.clear(); out.resize(n, (records[0] >> 8) as u8); return true;
            }
            if n <= 64 { return invert_tiny_fl_64(records, n, p, out); }
            if n <= 256 { return invert_tiny_fl_256(records, n, p, out); }
            if n <= 512 { return invert_tiny_fl_512(records, n, p, out); }
            if n <= 1536 { return invert_tiny_fl_1536(records, n, p, out); }
            if self.edges.len() < n { let add = n - self.edges.len(); self.edges.reserve_exact(add); self.edges.resize(n, 0); }
            invert_tiny_fl_with_buf(records, n, p, &mut self.edges[..n], out)
        }

        #[inline(never)]
        pub fn try_run_const<const LIM: u32>(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, bi_mode: BiMode, out: &mut Vec<u8>) -> Option<Counts> {
            let nrec = records.len();
            if n == 0 || n > 900_000 || (p as usize) >= n || nrec > 16_000_000 { return None; }
            out.resize(n + INLINE, 0);
            let (ns, f, heads, over, extra_127, tot_pos) = {
                let mut pass = Pass::<LOG_CHUNK, BYTES, S1_GROW> {
                    queues: outq_gen::Queues::new(&mut out[..], &mut self.spill, cap.m.min(n as u32)),
                    edges: &mut self.edges, ns: 0, cap_strips: cap.strips, f: First::default(),
                };
                let (mut pos, mut extra_127, mut i) = (0u32, 0u32, 0usize);
                while i < nrec {
                    let rec = records[i]; let c = (rec >> 8) as u8; let mut width = u32::from(rec & 255); i += 1;
                    if width <= LIM {
                        pass.piece(c, pos, width, true, p); pos += width;
                    } else {
                        if width == 127 {
                            while i < nrec && (records[i] >> 8) as u8 == c { width += u32::from(records[i] & 255); extra_127 += 1; i += 1; }
                        }
                        pass.take::<LIM>(c, pos, width, p); pos += width;
                    }
                }
                let over = pass.queues.over(); let thin = pass.queues.extract_thin(); let heads = pass.queues.heads();
                let mut f = pass.f; f.thin = thin; (pass.ns, f, heads, over, extra_127, pos)
            };
            if over || ns > cap.strips || tot_pos as usize != n || f.m == 0 { return None; }
            let r_max = (nrec as u32).saturating_sub(extra_127);
            let r_thin_lo = r_max.saturating_sub(ns as u32);
            if cap.post_s1_crossover && n >= 32_768 && 8 * u64::from(f.m) + 2 * u64::from(r_thin_lo) > 9 * (n as u64) {
                return None;
            }
            self.finish(&f, ns, n, r_max, &heads, bi_mode, out)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn finish(&mut self, f: &First, ns: usize, n: usize, r_max: u32, heads: &outq_gen::Heads, bi_mode: BiMode, out: &mut Vec<u8>) -> Option<Counts> {
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            if all.iter().sum::<u32>() != n as u32 { return None; }
            let first = starts(all);
            let mut write = starts(f.thick);
            let m = f.m as usize;
            let most = 3 * ns + 3;
            let use_narrow = most <= NARROW_NODES;
            let stride = if use_narrow { Narrow::STRIDE } else { Wide::STRIDE };
            self.bands.clear(); self.spelling.clear(); self.contents_u32_fb.clear();

            let tail_offset = (heads.chunks as usize).min(out.len() / BYTES) * BYTES;
            let use_zero_hdr = out.len() - tail_offset >= 16 * ns;
            let hdr_w = if use_zero_hdr { 0 } else { 4 * ns };
            let need = (hdr_w + stride * most).max(m);
            if self.edges.len() < need { let add = need - self.edges.len(); self.edges.reserve_exact(add); self.edges.resize(need, 0); }

            let (ok_ind, cursors, used_tail, nbands, live_words, rule_words, s2b_steps) = if use_narrow {
                self.induced_with::<Narrow>(ns, n as u32, f.m, &f.thin, &first, &mut write, most, use_zero_hdr, &mut out[tail_offset..])
            } else {
                self.induced_with::<Wide>(ns, n as u32, f.m, &f.thin, &first, &mut write, most, use_zero_hdr, &mut out[tail_offset..])
            };
            if !ok_ind { return None; }

            let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
            let sentinel = self.force_sentinel_cap.unwrap_or(u32::MAX >> bits).min(u32::MAX >> bits);
            let wide = self.lens.len() - 1 > sentinel as usize;
            if self.edges.len() < m { self.edges.resize(m, 0); }
            self.extra.clear();
            if wide { self.extra.resize(m, 0); }
            self.predecessor = None;
            let use_xor = bi_mode.should_use_bi(f.m, ns as u32, r_max);
            let (out_q, out_tail) = out.split_at(tail_offset);

            if use_xor {
                let pred = if !wide {
                    if used_tail {
                        self.wire_bi_tail(m, f.mark, f.mark_byte, bits, &cursors, nbands, out_tail, out_q, heads)
                    } else {
                        self.wire_bi_vec(m, f.mark, f.mark_byte, bits, &cursors, out_q, heads)
                    }
                } else {
                    self.wire_bi_sentinel(m, f.mark, f.mark_byte, bits, sentinel, &cursors, nbands, used_tail, out_tail, out_q, heads)
                };
                self.predecessor = Some(pred);
            } else if used_tail {
                self.wire_uni_tail(m, wide, bits, sentinel, &cursors, nbands, out_tail, out_q, heads);
            } else {
                self.wire_uni_vec(m, wide, bits, sentinel, &cursors, out_q, heads);
            }

            let outq_spill_used = self.spill.used_bytes(heads.chunks);
            let used_u32_fallback = !self.contents_u32_fb.is_empty();
            let contents_bytes = if DICT_MODE == 0 || used_u32_fallback { 8192 } else { 4096 };
            let s5_live = 4 * m + 4 * self.extra.len() + 16 * self.inline16.len() + self.lens.len() + self.spelling.len();
            let used_bytes = 4 * live_words.max(m) + 4 * self.extra.len() + 8 * self.bands.len()
                + self.spelling.len() + 16 * self.inline16.len() + self.lens.len() + outq_spill_used
                + 4 * self.contents_u32_fb.len();

            if let Some(before) = self.predecessor {
                if !wide {
                    if S5_MODE == 0 {
                        Self::emit_bi_v3(&self.edges[..m], &self.spelling, &self.inline16, &self.lens, n, f.mark, before, bits, out);
                    } else if S5_MODE == 1 {
                        Self::emit_bi_rem16(&self.edges[..m], &self.spelling, &self.inline16, &self.lens, n, f.mark, before, bits, out);
                    } else {
                        Self::emit_bi_pipe1_rem16(&self.edges[..m], &self.spelling, &self.inline16, &self.lens, n, f.mark, before, bits, out);
                    }
                } else {
                    Self::emit_bi_sentinel(&self.edges[..m], &self.extra, &self.spelling, &self.inline16, &self.lens, n, f.mark, before, bits, sentinel, out);
                }
            } else {
                Self::emit_uni(&self.edges[..m], &self.extra, &self.spelling, &self.inline16, &self.lens, n, f.mark, bits, sentinel, out);
            }

            Some(Counts {
                m, bits, num_codes: self.lens.len(), wide, used_bi: use_xor, used_tail,
                used_u32_fallback, outq_tail_bytes: tail_offset, outq_spill_used,
                contents_bytes, spelling_used: self.spelling.len(), used_bytes,
                s5_live_bytes: s5_live, rule_words, s2b_steps,
            })
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn induced_with<L: Links>(
            &mut self, ns: usize, n: u32, m: u32, thin: &[u32; 256], first: &[u32; 256],
            write: &mut [u32; 256], most: usize, use_zero_hdr: bool, out_tail: &mut [u8],
        ) -> (bool, [usize; 256], bool, usize, usize, usize, usize) {
            let nodes_words = L::STRIDE * most;
            let hdr_words = if use_zero_hdr { 0 } else { 4 * ns };
            assert!(self.edges.len() >= hdr_words + nodes_words);

            let mut widest = 0u32;
            let (base, nodes, open_nodes) = if use_zero_hdr {
                let (ot_words, _) = out_tail.as_chunks_mut::<4>();
                let (strips_ot, rest_ot) = ot_words.split_at_mut(3 * ns);
                let image_ot = &mut rest_ot[..ns];
                for (d, &s) in strips_ot.iter_mut().zip(&self.edges[..3 * ns]) { *d = s.to_ne_bytes(); }
                let (nodes_sl, rules_dummy) = self.edges.split_at_mut(nodes_words);
                let mut e = Elim::<L> { nodes_buf: nodes_sl, rules_buf: rules_dummy, rule_top: 0, nodes: 0, steps: 0, id_overflow: false, _links: std::marker::PhantomData };
                e.node(0, 0); e.nodes = ns as u32 + 1;
                let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
                let mut x = 0u32;
                for i in 0..ns {
                    let from = u32::from_ne_bytes(strips_ot[3 * i]);
                    let to0 = u32::from_ne_bytes(strips_ot[3 * i + 1]);
                    let wbyte = u32::from_ne_bytes(strips_ot[3 * i + 2]);
                    let (byte, width) = ((wbyte & 0xFF) as u8, wbyte >> 8);
                    let a = usize::from(byte);
                    strips_ot[3 * i + 1] = (to0 + first[a]).to_ne_bytes();
                    let dst_idx = write[a] as usize; image_ot[dst_idx] = (i as u32).to_ne_bytes(); write[a] = dst_idx as u32 + 1;
                    let sid = i as u32 + 1; e.set_width(sid, width); e.set_word(sid, literal(byte)); widest = widest.max(width);
                    if x < from { let gw = from - x; widest = widest.max(gw); let gid = e.node(gw, 0); e.append_to(&mut order[0], gid, 0); e.append_to(&mut terminals[1], gid, 1); }
                    e.append_to(&mut order[0], sid, 0); x = from + width;
                }
                if x < n { widest = widest.max(n - x); let id = e.node(n - x, 0); e.append_to(&mut order[0], id, 0); e.append_to(&mut terminals[1], id, 1); }
                x = 0;
                for slot in image_ot.iter() {
                    let i = u32::from_ne_bytes(*slot); let idx = i as usize;
                    let to = u32::from_ne_bytes(strips_ot[3 * idx + 1]);
                    let width = u32::from_ne_bytes(strips_ot[3 * idx + 2]) >> 8;
                    if x < to { widest = widest.max(to - x); let id = e.node(to - x, 0); e.append_to(&mut order[1], id, 1); e.append_to(&mut terminals[0], id, 0); }
                    e.append_to(&mut order[1], i + 1, 1); x = to + width;
                }
                if x < n { widest = widest.max(n - x); let id = e.node(n - x, 0); e.append_to(&mut order[1], id, 1); e.append_to(&mut terminals[0], id, 0); }
                let open_nodes = e.nodes as usize - 1;
                for side in 0..2 { e.close(side, terminals[side], order[side]); }
                (L::STRIDE * e.nodes as usize, e.nodes, open_nodes)
            } else {
                let (hdr, arena) = self.edges.split_at_mut(4 * ns);
                let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
                let (nodes_sl, rules_dummy) = arena.split_at_mut(nodes_words);
                let mut e = Elim::<L> { nodes_buf: nodes_sl, rules_buf: rules_dummy, rule_top: 0, nodes: 0, steps: 0, id_overflow: false, _links: std::marker::PhantomData };
                e.node(0, 0); e.nodes = ns as u32 + 1;
                let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
                let mut x = 0u32;
                for i in 0..ns {
                    let sw = &mut strips_raw[3 * i..3 * i + 3];
                    let (from, wbyte) = (sw[0], sw[2]);
                    let (byte, width) = ((wbyte & 0xFF) as u8, wbyte >> 8);
                    let a = usize::from(byte); sw[1] += first[a];
                    let dst_idx = write[a] as usize; image_order[dst_idx] = i as u32; write[a] = dst_idx as u32 + 1;
                    let sid = i as u32 + 1; e.set_width(sid, width); e.set_word(sid, literal(byte)); widest = widest.max(width);
                    if x < from { let gw = from - x; widest = widest.max(gw); let gid = e.node(gw, 0); e.append_to(&mut order[0], gid, 0); e.append_to(&mut terminals[1], gid, 1); }
                    e.append_to(&mut order[0], sid, 0); x = from + width;
                }
                if x < n { widest = widest.max(n - x); let id = e.node(n - x, 0); e.append_to(&mut order[0], id, 0); e.append_to(&mut terminals[1], id, 1); }
                x = 0;
                for &i in image_order.iter() {
                    let idx = i as usize; let sw = &strips_raw[3 * idx..3 * idx + 3];
                    let (to, width) = (sw[1], sw[2] >> 8);
                    if x < to { widest = widest.max(to - x); let id = e.node(to - x, 0); e.append_to(&mut order[1], id, 1); e.append_to(&mut terminals[0], id, 0); }
                    e.append_to(&mut order[1], i + 1, 1); x = to + width;
                }
                if x < n { widest = widest.max(n - x); let id = e.node(n - x, 0); e.append_to(&mut order[1], id, 1); e.append_to(&mut terminals[0], id, 0); }
                let open_nodes = e.nodes as usize - 1;
                for side in 0..2 { e.close(side, terminals[side], order[side]); }
                (L::STRIDE * e.nodes as usize, e.nodes, open_nodes)
            };

            let spirals = widest > CERTIFIED_WIDTH;
            let (mut remaining, mut rule_top, mut s2b_steps) = (n, 0usize, 0usize);
            loop {
                let (nodes_sl, rules_sl) = self.edges[hdr_words..].split_at_mut(base);
                let mut e = Elim::<L> { nodes_buf: nodes_sl, rules_buf: rules_sl, rule_top, nodes, steps: 0, id_overflow: false, _links: std::marker::PhantomData };
                let done = if spirals { e.eliminate::<true>(&mut remaining) } else { e.eliminate::<false>(&mut remaining) };
                rule_top = e.rule_top; s2b_steps += e.steps;
                if e.id_overflow { return (false, [0; 256], false, 0, 0, 0, 0); }
                if done { break; }
                grow_edges_step(&mut self.edges, hdr_words + base + rule_top + SPIRAL_RULES * MAX_RULE_WORDS);
            }

            let pad = 2usize;
            let min_live = hdr_words + base + rule_top + pad;
            if self.edges.len() < min_live {
                self.edges.reserve_exact(min_live - self.edges.len());
                self.edges.resize(min_live, 0);
            }
            let needed_cache_words = match DICT_MODE {
                0 => CACHE_WORDS_U32,
                3 => CACHE_WORDS_U16,
                _ => 0,
            };
            if needed_cache_words > 0 && use_zero_hdr && ns >= 64 && self.edges.len() < base + rule_top + pad + needed_cache_words {
                let need_c = base + rule_top + pad + needed_cache_words;
                self.edges.reserve_exact(need_c - self.edges.len());
                self.edges.resize(need_c, 0);
            }

            let (header, arena) = self.edges.split_at_mut(hdr_words);
            let (live, tail_arena) = arena.split_at_mut(base + rule_top + pad);
            let (nodes_sl, rules_sl) = live.split_at_mut(base);
            let mut e = Elim::<L> { nodes_buf: nodes_sl, rules_buf: rules_sl, rule_top, nodes, steps: 0, id_overflow: false, _links: std::marker::PhantomData };
            e.assign_bottom(m);

            begin_inline(&mut self.inline16, &mut self.lens);
            let max_bands = open_nodes + 256;
            let use_tail = !self.force_band_vec && out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec = [0u32; MEMO_REC_CAP];

            if DICT_MODE == 1 || DICT_MODE == 2 {
                // Stack-local 2 KiB pairs + 4 KiB u16 contents = 6 KiB in L1D, ZERO edges tail growth!
                let mut pairs_stk = [0u32; 512];
                let mut contents_stk = [0u16; 2048];
                let store = if DICT_MODE == 1 {
                    let cap = self.force_u16_cap.unwrap_or(65_534).min(65_534);
                    ContentsStore::U16Raw { tbl: &mut contents_stk, fb: &mut self.contents_u32_fb, cap }
                } else {
                    let cap = self.force_u16_cap.unwrap_or(4_096).min(4_096);
                    ContentsStore::U16Tag4 { tbl: &mut contents_stk, fb: &mut self.contents_u32_fb, cap }
                };
                let mut dictionary = Dictionary {
                    pairs: &mut pairs_stk, contents: store,
                    inline16: &mut self.inline16, lens: &mut self.lens, spelling: &mut self.spelling,
                    istk: &mut istk, memo_rec: &mut memo_rec, stack: &mut self.stack,
                };
                Self::emit_bands_dict(&mut e, &mut dictionary, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors, &mut nbands);
            } else {
                let cache = if header.len() >= needed_cache_words { header } else { tail_arena };
                if cache.len() < needed_cache_words {
                    let (band_slots, _) = out_tail.as_chunks_mut::<8>();
                    let (mut id, mut in_node, mut k) = (L::next(e.nodes_buf, 0, 0), 0u32, 0u32);
                    for a in 0..256 {
                        cursors[a] = if use_tail { nbands } else { self.bands.len() };
                        let end = k + thin[a];
                        while k < end {
                            let (next_id, node_bottom, node_width, node_word) = L::band_node(e.nodes_buf, id);
                            let width = (end - k).min(node_width - in_node);
                            let dst = node_bottom + in_node;
                            let code = if node_word == 0 { a as u32 } else {
                                let (entry, len) = e.inline_word(node_word, a as u8, &mut istk, &mut memo_rec, &mut self.stack, &mut self.spelling);
                                ensure_step_inline(&mut self.inline16, &mut self.lens, INLINE_STEP);
                                let c = self.lens.len() as u32; self.inline16.push(entry); self.lens.push(len); c
                            };
                            if use_tail { band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes(); nbands += 1; }
                            else { self.bands.push(Band::new(width, dst, code)); }
                            in_node += width; k += width;
                            if in_node == node_width { id = next_id; in_node = 0; }
                        }
                    }
                    assert_eq!(id, 0);
                } else {
                    cache[..needed_cache_words].fill(0);
                    let (p_sl, c_sl) = cache[..needed_cache_words].split_at_mut(PAIR_WORDS);
                    let pairs: &mut [u32; 512] = p_sl.try_into().unwrap();
                    let store = if DICT_MODE == 0 {
                        ContentsStore::U32(c_sl.try_into().unwrap())
                    } else {
                        let cap = self.force_u16_cap.unwrap_or(4_096).min(4_096);
                        ContentsStore::U16Packed { tbl: c_sl.try_into().unwrap(), fb: &mut self.contents_u32_fb, cap }
                    };
                    let mut dictionary = Dictionary {
                        pairs, contents: store,
                        inline16: &mut self.inline16, lens: &mut self.lens, spelling: &mut self.spelling,
                        istk: &mut istk, memo_rec: &mut memo_rec, stack: &mut self.stack,
                    };
                    Self::emit_bands_dict(&mut e, &mut dictionary, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors, &mut nbands);
                }
            }
            (true, cursors, use_tail, nbands, hdr_words + base + rule_top + pad, rule_top, s2b_steps)
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bands_dict<L: Links>(
            e: &mut Elim<L>, dictionary: &mut Dictionary, thin: &[u32; 256],
            bands: &mut Vec<Band>, out_tail: &mut [u8], max_bands: usize,
            use_tail: bool, cursors: &mut [usize; 256], nbands: &mut usize,
        ) {
            let mut id = L::next(e.nodes_buf, 0, 0);
            let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 { L::band_node(e.nodes_buf, id) } else { (0, 0, 0, 0) };
            let (mut in_node, mut k) = (0u32, 0u32);
            if use_tail {
                let (band_slots, _) = out_tail[..max_bands * 8].as_chunks_mut::<8>();
                for a in 0..256 {
                    cursors[a] = *nbands;
                    let end = k + thin[a];
                    let (mut last_w, mut last_c) = (0u32, 0u32);
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        let dst = node_bottom + in_node;
                        let code = if node_word == 0 { a as u32 }
                            else if node_word == last_w { last_c }
                            else { let c = dictionary.code(e, node_word, a as u8); last_w = node_word; last_c = c; c };
                        band_slots[*nbands] = Band::new(width, dst, code).0.to_le_bytes();
                        *nbands += 1; in_node += width; k += width;
                        if in_node == node_width { id = next_id; in_node = 0; if id != 0 { (next_id, node_bottom, node_width, node_word) = L::band_node(e.nodes_buf, id); } }
                    }
                }
            } else {
                for a in 0..256 {
                    cursors[a] = bands.len();
                    let end = k + thin[a];
                    let (mut last_w, mut last_c) = (0u32, 0u32);
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        let dst = node_bottom + in_node;
                        let code = if node_word == 0 { a as u32 }
                            else if node_word == last_w { last_c }
                            else { let c = dictionary.code(e, node_word, a as u8); last_w = node_word; last_c = c; c };
                        bands.push(Band::new(width, dst, code));
                        in_node += width; k += width;
                        if in_node == node_width { id = next_id; in_node = 0; if id != 0 { (next_id, node_bottom, node_width, node_word) = L::band_node(e.nodes_buf, id); } }
                    }
                }
            }
            assert_eq!(id, 0);
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_tail(&mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, cursors: &[usize; 256], nbands: usize, out_tail: &[u8], out_q: &[u8], heads: &outq_gen::Heads) -> u32 {
            let edges = &mut self.edges[..m]; edges.fill(0);
            let (bs, _) = out_tail.as_chunks::<8>();
            let mut pred = mark; let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut q = outq_gen::Reader::<LOG_CHUNK, BYTES>::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &raw in &bs[cursors[a]..end] { let b = Band(u64::from_le_bytes(raw)); q.wire_xor_nomark::<S4_MODE>(edges, b.dst() as usize, b.end() as usize, b.code() << bits); }
                } else {
                    for &raw in &bs[cursors[a]..end] { let b = Band(u64::from_le_bytes(raw)); q.wire_xor(edges, b.dst() as usize, b.end() as usize, b.code() << bits, mark, &mut pred); }
                }
            }
            pred
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_vec(&mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, cursors: &[usize; 256], out_q: &[u8], heads: &outq_gen::Heads) -> u32 {
            let edges = &mut self.edges[..m]; edges.fill(0);
            let mut pred = mark; let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut q = outq_gen::Reader::<LOG_CHUNK, BYTES>::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &b in &self.bands[cursors[a]..end] { q.wire_xor_nomark::<S4_MODE>(edges, b.dst() as usize, b.end() as usize, b.code() << bits); }
                } else {
                    for &b in &self.bands[cursors[a]..end] { q.wire_xor(edges, b.dst() as usize, b.end() as usize, b.code() << bits, mark, &mut pred); }
                }
            }
            pred
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_sentinel(&mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, sentinel: u32, cursors: &[usize; 256], nbands: usize, used_tail: bool, out_tail: &[u8], out_q: &[u8], heads: &outq_gen::Heads) -> u32 {
            let edges = &mut self.edges[..m]; edges.fill(0);
            let extra = &mut self.extra[..m];
            let mut pred = mark; let mb = usize::from(mark_byte);
            if used_tail {
                let (bs, _) = out_tail.as_chunks::<8>();
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                    let mut q = outq_gen::Reader::<LOG_CHUNK, BYTES>::new(out_q, &self.spill, heads, a);
                    for &raw in &bs[cursors[a]..end] {
                        let b = Band(u64::from_le_bytes(raw)); let (dst, sp, bc) = (b.dst() as usize, b.end() as usize, b.code());
                        let pack = if bc < sentinel { bc << bits } else { extra[dst..dst + sp].fill(bc); sentinel << bits };
                        if a != mb { q.wire_xor_nomark::<S4_MODE>(edges, dst, sp, pack); } else { q.wire_xor(edges, dst, sp, pack, mark, &mut pred); }
                    }
                }
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut q = outq_gen::Reader::<LOG_CHUNK, BYTES>::new(out_q, &self.spill, heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let (dst, sp, bc) = (b.dst() as usize, b.end() as usize, b.code());
                        let pack = if bc < sentinel { bc << bits } else { extra[dst..dst + sp].fill(bc); sentinel << bits };
                        if a != mb { q.wire_xor_nomark::<S4_MODE>(edges, dst, sp, pack); } else { q.wire_xor(edges, dst, sp, pack, mark, &mut pred); }
                    }
                }
            }
            pred
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_tail(&mut self, m: usize, wide: bool, bits: u32, sentinel: u32, cursors: &[usize; 256], nbands: usize, out_tail: &[u8], out_q: &[u8], heads: &outq_gen::Heads) {
            let edges = &mut self.edges[..m];
            let (bs, _) = out_tail.as_chunks::<8>();
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut q = outq_gen::Reader::<LOG_CHUNK, BYTES>::new(out_q, &self.spill, heads, a);
                for &raw in &bs[cursors[a]..end] {
                    let b = Band(u64::from_le_bytes(raw)); let (sp, dst, bc) = (b.end() as usize, b.dst() as usize, b.code());
                    let pack = if !wide || bc < sentinel { bc << bits } else { self.extra[dst..dst + sp].fill(bc); sentinel << bits };
                    q.wire(&mut edges[dst..dst + sp], pack);
                }
            }
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_vec(&mut self, m: usize, wide: bool, bits: u32, sentinel: u32, cursors: &[usize; 256], out_q: &[u8], heads: &outq_gen::Heads) {
            let edges = &mut self.edges[..m];
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut q = outq_gen::Reader::<LOG_CHUNK, BYTES>::new(out_q, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let (sp, dst, bc) = (b.end() as usize, b.dst() as usize, b.code());
                    let pack = if !wide || bc < sentinel { bc << bits } else { self.extra[dst..dst + sp].fill(bc); sentinel << bits };
                    q.wire(&mut edges[dst..dst + sp], pack);
                }
            }
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_uni(edges: &[u32], extra: &[u32], spelling: &[u8], inline16: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, bits: u32, sentinel: u32, out: &mut Vec<u8>) {
            let mask = (1 << bits) - 1;
            let mut j = mark;
            if extra.is_empty() {
                let len_c = lens.len().min(inline16.len());
                let (lens, inline16) = (&lens[..len_c], &inline16[..len_c]);
                let mut rem: &mut [u8] = &mut out[..n];
                let mut done = false;
                while rem.len() >= INLINE {
                    let edge = edges[j as usize]; let code = (edge >> bits) as usize; j = edge & mask;
                    let len = lens[code];
                    if len != LONG { rem[..INLINE].copy_from_slice(&inline16[code]); rem = &mut rem[usize::from(len)..]; }
                    else { let take = emit_long_front(&inline16[code], spelling, rem); rem = &mut rem[take..]; }
                    if j == mark { done = true; break; }
                }
                if !done {
                    while !rem.is_empty() {
                        let edge = edges[j as usize]; let code = (edge >> bits) as usize; j = edge & mask;
                        let len = lens[code];
                        let word = if len == LONG { long_word(&inline16[code], spelling) } else { &inline16[code][..usize::from(len)] };
                        let take = word.len().min(rem.len()); rem[..take].copy_from_slice(&word[..take]); rem = &mut rem[take..];
                        if j == mark { break; }
                    }
                }
                let written = n - rem.len(); out.truncate(written); repeat(n, out); return;
            }
            let out_buf = &mut out[..n + INLINE];
            let mut at = 0usize;
            for _ in 0..edges.len() {
                let edge = edges[j as usize]; let rc = edge >> bits;
                let code = if rc == sentinel { extra[j as usize] } else { rc } as usize;
                j = edge & mask;
                let len = lens[code];
                if len != LONG { out_buf[at..at + INLINE].copy_from_slice(&inline16[code]); at += usize::from(len); }
                else { let w = long_word(&inline16[code], spelling); let t = w.len().min(n - at); out_buf[at..at + t].copy_from_slice(&w[..t]); at += t; }
                if j == mark || at >= n { break; }
            }
            out.truncate(at.min(n)); repeat(n, out);
        }

        /// S5_MODE = 0: C52-SuperDropIn-v3 peeled slice-bound-eliminated bidirectional emit.
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi_v3(edges: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, before: u32, bits: u32, out: &mut Vec<u8>) {
            let out_buf = &mut out[..n];
            let mask = (1 << bits) - 1;
            let (mut prev, mut front, mut back, mut next, mut lo, mut hi) = (before, mark, before, mark, 0usize, n);
            for _ in 0..=edges.len() / 2 {
                let ef = edges[front as usize]; let cf = (ef >> bits) as usize; let single = front == back; let lf = lens[cf];
                if lf != LONG && hi - lo >= INLINE { out_buf[lo..lo + INLINE].copy_from_slice(&inline[cf]); lo += usize::from(lf); }
                else { let w = if lf == LONG { long_word(&inline[cf], spelling) } else { &inline[cf][..usize::from(lf)] }; let t = w.len().min(hi - lo); out_buf[lo..lo + t].copy_from_slice(&w[..t]); lo += t; }
                if single || lo == hi { break; }
                let eb = edges[back as usize]; let cb = (eb >> bits) as usize; let lb = lens[cb];
                if lb != LONG && hi - lo >= INLINE {
                    let sh = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                    out_buf[hi - INLINE..hi].copy_from_slice(&sh.to_le_bytes()); hi -= usize::from(lb);
                } else { let w = if lb == LONG { long_word(&inline[cb], spelling) } else { &inline[cb][..usize::from(lb)] }; let t = w.len().min(hi - lo); out_buf[hi - t..hi].copy_from_slice(&w[w.len() - t..]); hi -= t; }
                (prev, front) = (front, (ef & mask) ^ prev);
                (next, back) = (back, (eb & mask) ^ next);
                if lo == hi || front == next { break; }
            }
            if lo < hi { out.copy_within(hi..n, lo); out.truncate(lo + (n - hi)); repeat(n, out); } else { out.truncate(n); }
        }

        /// S5_MODE = 1: `SlimS5_Rem16` (C52-Lean-Compact shrinking-slice + cold-outlined LONG).
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi_rem16(edges: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, before: u32, bits: u32, out: &mut Vec<u8>) {
            let mask = (1 << bits) - 1;
            let (mut prev, mut front, mut back, mut next) = (before, mark, before, mark);
            let out_base = out.as_ptr() as usize;
            let mut rem: &mut [u8] = &mut out[..n];
            let len_c = lens.len().min(inline.len());
            let (lens, inline) = (&lens[..len_c], &inline[..len_c]);
            let mut done = false;
            while rem.len() >= 2 * INLINE {
                let ef = edges[front as usize]; let cf = (ef >> bits) as usize; let single = front == back; let lf = lens[cf];
                if lf != LONG {
                    rem[..INLINE].copy_from_slice(&inline[cf]); rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline[cf], spelling, rem); rem = &mut rem[take..];
                    if single || rem.is_empty() { done = true; break; }
                    if rem.len() < INLINE {
                        let eb = edges[back as usize]; let cb = (eb >> bits) as usize;
                        let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                        let rlen = rem.len(); rem = &mut rem[..rlen - take_b];
                        (prev, front) = (front, (ef & mask) ^ prev); (next, back) = (back, (eb & mask) ^ next);
                        if front == next { done = true; }
                        break;
                    }
                }
                if single { done = true; break; }
                let eb = edges[back as usize]; let cb = (eb >> bits) as usize; let rlen = rem.len(); let lb = lens[cb];
                if lb != LONG {
                    let sh = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                    rem[rlen - INLINE..].copy_from_slice(&sh.to_le_bytes()); rem = &mut rem[..rlen - usize::from(lb)];
                } else {
                    let take = emit_long_back(&inline[cb], spelling, rem); rem = &mut rem[..rlen - take];
                }
                (prev, front) = (front, (ef & mask) ^ prev);
                (next, back) = (back, (eb & mask) ^ next);
                if front == next { done = true; break; }
            }
            if !done {
                while !rem.is_empty() {
                    let ef = edges[front as usize]; let cf = (ef >> bits) as usize; let single = front == back; let lf = lens[cf];
                    let word = if lf == LONG { long_word(&inline[cf], spelling) } else { &inline[cf][..usize::from(lf)] };
                    let take = word.len().min(rem.len()); rem[..take].copy_from_slice(&word[..take]); rem = &mut rem[take..];
                    if single || rem.is_empty() { break; }
                    let eb = edges[back as usize]; let cb = (eb >> bits) as usize;
                    let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                    let rlen = rem.len(); rem = &mut rem[..rlen - take_b];
                    (prev, front) = (front, (ef & mask) ^ prev); (next, back) = (back, (eb & mask) ^ next);
                    if front == next { break; }
                }
            }
            let rem_len = rem.len();
            let lo_rem = rem.as_ptr() as usize - out_base;
            let hi_rem = lo_rem + rem_len;
            if rem_len > 0 { out.copy_within(hi_rem..n, lo_rem); out.truncate(lo_rem + (n - hi_rem)); repeat(n, out); }
            else { out.truncate(n); }
        }

        /// S5_MODE = 2: **`SlimS5_Pipe1_Rem16`** — Zero-spill register-tight fusion of
        /// `Q-2Cur-Pipe1` (1-step software-pipelined `edges[front]` + `edges[back]` loads overlapped
        /// with current emit, `+0 B` WS!) + `SlimS5_Rem16` (shrinking slice `rem: &mut [u8]` +
        /// cold-outlined `LONG` helpers)!
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi_pipe1_rem16(edges: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, before: u32, bits: u32, out: &mut Vec<u8>) {
            let mask = (1 << bits) - 1;
            let (mut prev, mut front, mut back, mut next) = (before, mark, before, mark);
            let out_base = out.as_ptr() as usize;
            let mut rem: &mut [u8] = &mut out[..n];
            let len_c = lens.len().min(inline.len());
            let (lens, inline) = (&lens[..len_c], &inline[..len_c]);
            let mut done = false;
            if front != back && rem.len() >= 2 * INLINE {
                let mut ef = edges[front as usize];
                let mut eb = edges[back as usize];
                while rem.len() >= 2 * INLINE {
                    let cf = (ef >> bits) as usize;
                    if front == back {
                        let lf = lens[cf];
                        if lf != LONG {
                            rem[..INLINE].copy_from_slice(&inline[cf]);
                            rem = &mut rem[usize::from(lf)..];
                        } else {
                            let take = emit_long_front(&inline[cf], spelling, rem);
                            rem = &mut rem[take..];
                        }
                        done = true;
                        break;
                    }
                    let cb = (eb >> bits) as usize;
                    let f_next = (ef & mask) ^ prev;
                    let b_next = (eb & mask) ^ next;
                    prev = front; front = f_next;
                    next = back;  back = b_next;
                    // Issue next-iteration pointer-chase loads BEFORE emitting current words,
                    // overwriting ef/eb in place so zero extra live registers cross the emit!
                    ef = edges[front as usize];
                    eb = edges[back as usize];
                    let lf = lens[cf];
                    if lf != LONG {
                        rem[..INLINE].copy_from_slice(&inline[cf]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline[cf], spelling, rem);
                        rem = &mut rem[take..];
                        if rem.is_empty() { done = true; break; }
                        if rem.len() < INLINE {
                            let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                            let rlen = rem.len();
                            rem = &mut rem[..rlen - take_b];
                            if front == next { done = true; }
                            break;
                        }
                    }
                    let rlen = rem.len();
                    let lb = lens[cb];
                    if lb != LONG {
                        let sh = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                        rem[rlen - INLINE..].copy_from_slice(&sh.to_le_bytes());
                        rem = &mut rem[..rlen - usize::from(lb)];
                    } else {
                        let take = emit_long_back(&inline[cb], spelling, rem);
                        rem = &mut rem[..rlen - take];
                    }
                    if front == next { done = true; break; }
                }
            }
            if !done {
                while !rem.is_empty() {
                    let ef = edges[front as usize]; let cf = (ef >> bits) as usize; let single = front == back; let lf = lens[cf];
                    let word = if lf == LONG { long_word(&inline[cf], spelling) } else { &inline[cf][..usize::from(lf)] };
                    let take = word.len().min(rem.len()); rem[..take].copy_from_slice(&word[..take]); rem = &mut rem[take..];
                    if single || rem.is_empty() { break; }
                    let eb = edges[back as usize]; let cb = (eb >> bits) as usize;
                    let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                    let rlen = rem.len(); rem = &mut rem[..rlen - take_b];
                    (prev, front) = (front, (ef & mask) ^ prev); (next, back) = (back, (eb & mask) ^ next);
                    if front == next { break; }
                }
            }
            let rem_len = rem.len();
            let lo_rem = rem.as_ptr() as usize - out_base;
            let hi_rem = lo_rem + rem_len;
            if rem_len > 0 { out.copy_within(hi_rem..n, lo_rem); out.truncate(lo_rem + (n - hi_rem)); repeat(n, out); }
            else { out.truncate(n); }
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi_sentinel(edges: &[u32], extra: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, before: u32, bits: u32, sentinel: u32, out: &mut Vec<u8>) {
            let out_buf = &mut out[..n];
            let mask = (1 << bits) - 1;
            let (mut prev, mut front, mut back, mut next, mut lo, mut hi) = (before, mark, before, mark, 0usize, n);
            for _ in 0..=edges.len() / 2 {
                let ef = edges[front as usize];
                let cf_raw = ef >> bits;
                let cf = if cf_raw == sentinel { extra[front as usize] } else { cf_raw } as usize;
                let single = front == back; let lf = lens[cf];
                if lf != LONG && hi - lo >= INLINE { out_buf[lo..lo + INLINE].copy_from_slice(&inline[cf]); lo += usize::from(lf); }
                else { let w = if lf == LONG { long_word(&inline[cf], spelling) } else { &inline[cf][..usize::from(lf)] }; let t = w.len().min(hi - lo); out_buf[lo..lo + t].copy_from_slice(&w[..t]); lo += t; }
                if single || lo == hi { break; }
                let eb = edges[back as usize];
                let cb_raw = eb >> bits;
                let cb = if cb_raw == sentinel { extra[back as usize] } else { cb_raw } as usize;
                let lb = lens[cb];
                if lb != LONG && hi - lo >= INLINE {
                    let sh = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                    out_buf[hi - INLINE..hi].copy_from_slice(&sh.to_le_bytes()); hi -= usize::from(lb);
                } else { let w = if lb == LONG { long_word(&inline[cb], spelling) } else { &inline[cb][..usize::from(lb)] }; let t = w.len().min(hi - lo); out_buf[hi - t..hi].copy_from_slice(&w[w.len() - t..]); hi -= t; }
                (prev, front) = (front, (ef & mask) ^ prev);
                (next, back) = (back, (eb & mask) ^ next);
                if lo == hi || front == next { break; }
            }
            if lo < hi { out.copy_within(hi..n, lo); out.truncate(lo + (n - hi)); repeat(n, out); } else { out.truncate(n); }
        }
    }
}

// ============================================================================
// 4. Corpus Loaders, Synthetic Generators, Reference FL & Selector Wrapper
// ============================================================================

struct Block {
    name: String,
    idx: u32,
    n: usize,
    p: u32,
    route_class: u8,
    records: Vec<u16>,
    expected: Option<Vec<u8>>,
}

fn records_of_cap(l: &[u8], cap: u32) -> Vec<u16> {
    let mut recs = Vec::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && u32::from(*rec & 255) < cap { *rec += 1; continue; }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}
fn records_of(l: &[u8]) -> Vec<u16> { records_of_cap(l, 127) }

fn expand_records(recs: &[u16], n: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &rec in recs { l.resize(l.len() + (rec & 255) as usize, (rec >> 8) as u8); }
    assert_eq!(l.len(), n); l
}

fn bwt_small(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| {
        for k in 0..n { let (ca, cb) = (text[(a + k) % n], text[(b + k) % n]); if ca != cb { return ca.cmp(&cb); } }
        a.cmp(&b)
    });
    let (mut l, mut p) = (Vec::with_capacity(n), 0u32);
    for (i, &pos) in sa.iter().enumerate() { if pos == 0 { p = i as u32; } l.push(text[(pos + n - 1) % n]); }
    (l, p)
}

fn plain_invert_fl(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l { counts[b as usize] += 1; }
    let mut st = [0u32; 256]; let mut tot = 0u32;
    for c in 0..256 { st[c] = tot; tot += counts[c]; }
    let mut fl = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() { let s = st[b as usize] as usize; st[b as usize] += 1; fl[s] = ((i as u32) << 8) | u32::from(b); }
    let mut out = vec![0u8; n]; let mut x = p as usize;
    for slot in &mut out { let v = fl[x]; *slot = v as u8; x = (v >> 8) as usize; }
    out
}

fn is_single_cycle_bwt(l: &[u8]) -> bool {
    let n = l.len();
    if n == 0 { return false; }
    let mut counts = [0u32; 256];
    for &b in l { counts[b as usize] += 1; }
    let mut st = [0u32; 256]; let mut tot = 0u32;
    for c in 0..256 { st[c] = tot; tot += counts[c]; }
    let mut lf = vec![0usize; n];
    for (i, &b) in l.iter().enumerate() {
        let s = st[b as usize] as usize; st[b as usize] += 1; lf[i] = s;
    }
    let mut x = 0usize;
    for step in 1..=n {
        x = lf[x];
        if x == 0 { return step == n; }
    }
    false
}

fn resolve_corpus_path(rel: &str) -> String {
    let p1 = format!("/home/antigravity/mibz2_corpora_cache/{rel}");
    if Path::new(&p1).exists() { return p1; }
    let p2 = format!("/tmp/mibz2_corpora/{rel}");
    assert!(Path::new(&p2).exists(), "missing corpus {rel}"); p2
}

fn load_six_admitted() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = fs::read_to_string(format!("{dir}/manifest.json")).expect("manifest");
    let (mut specs, mut file) = (Vec::new(), String::new());
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") { file = v.trim_end_matches("\",").to_string(); }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") { specs.push((file.clone(), v.trim_end_matches(',').parse::<u32>().unwrap())); }
    }
    specs.into_iter().enumerate().map(|(i, (f, p))| {
        let l = fs::read(format!("{dir}/{f}")).unwrap();
        let expected = plain_invert_fl(&l, p);
        Block { name: f, idx: i as u32, n: l.len(), p, route_class: 0, records: records_of(&l), expected: Some(expected) }
    }).collect()
}

fn load_bin_corpus(path: &str, prefix: &str, with_expected: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap();
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes(raw[8..12].try_into().unwrap()) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::with_capacity(num);
    for _ in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let rc = raw[off + 16];
        let nr = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(nr);
        for _ in 0..nr { records.push(u16::from_le_bytes([raw[off], raw[off + 1]])); off += 2; }
        let expected = if with_expected { let l = expand_records(&records, n); Some(plain_invert_fl(&l, p)) } else { None };
        blocks.push(Block { name: format!("{prefix}-{idx}"), idx, n, p, route_class: rc, records, expected });
    }
    blocks
}

fn make_dense_300k() -> Block {
    let n = 300_000usize;
    let mut text = Vec::with_capacity(n);
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..n {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        text.push(if i % 9 == 0 { b' ' } else { b'a' + ((s >> 33) % 26) as u8 });
    }
    let mut sa: Vec<u32> = (0..n as u32).collect();
    sa.sort_by(|&a, &b| {
        let (ia, ib) = (a as usize, b as usize);
        for k in 0..64 { let (ca, cb) = (text[(ia + k) % n], text[(ib + k) % n]); if ca != cb { return ca.cmp(&cb); } }
        ia.cmp(&ib)
    });
    let (mut l, mut p) = (Vec::with_capacity(n), 0u32);
    for (i, &pos) in sa.iter().enumerate() { if pos == 0 { p = i as u32; } l.push(text[(pos as usize + n - 1) % n]); }
    let expected = plain_invert_fl(&l, p);
    assert_eq!(expected, text);
    Block { name: "dense-300k".into(), idx: 0, n, p, route_class: 3, records: records_of(&l), expected: Some(expected) }
}

fn make_spiral_200k() -> Block {
    let n = 200_000usize;
    let mut l = vec![b'a'; n]; l[0] = b'c'; l[n - 1] = b'b';
    let expected = plain_invert_fl(&l, 0);
    Block { name: "spiral-200k".into(), idx: 0, n, p: 0, route_class: 2, records: records_of(&l), expected: Some(expected) }
}

fn make_periodic(name: &str, pat: &[u8], reps: usize) -> Block {
    let n = pat.len() * reps;
    let mut rots: Vec<usize> = (0..pat.len()).collect();
    rots.sort_by(|&a, &b| {
        for k in 0..pat.len() { let (ca, cb) = (pat[(a + k) % pat.len()], pat[(b + k) % pat.len()]); if ca != cb { return ca.cmp(&cb); } }
        a.cmp(&b)
    });
    let (mut l, mut p) = (Vec::with_capacity(n), 0u32);
    for (grp, &r) in rots.iter().enumerate() {
        let pb = pat[(r + pat.len() - 1) % pat.len()];
        if r == 0 { p = (grp * reps) as u32; }
        for _ in 0..reps { l.push(pb); }
    }
    let expected = plain_invert_fl(&l, p);
    Block { name: name.into(), idx: 0, n, p, route_class: 2, records: records_of(&l), expected: Some(expected) }
}

#[derive(Default)]
struct PlainRunWalk { fl: Vec<u32> }
impl PlainRunWalk {
    fn build_fl(&mut self, records: &[u16], n: usize) -> bool {
        let mut counts = [0u32; 256];
        for &rec in records { counts[(rec >> 8) as usize] += u32::from(rec & 255); }
        let mut st = [0u32; 256]; let mut tot = 0u32;
        for c in 0..256 { st[c] = tot; tot += counts[c]; }
        if tot as usize != n { return false; }
        if self.fl.len() < n { self.fl.resize(n, 0); }
        let fl = &mut self.fl[..n];
        let mut pos = 0usize;
        for &rec in records {
            let (b, len) = ((rec >> 8) as usize, (rec & 255) as usize);
            let s0 = st[b] as usize; st[b] += len as u32;
            let (mut i, bu) = (pos as u32, b as u32);
            for slot in &mut fl[s0..s0 + len] { *slot = (i << 8) | bu; i += 1; }
            pos += len;
        }
        true
    }
    fn try_invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if n == 0 || (p as usize) >= n { return false; }
        if records.len() == 1 { if usize::from(records[0] & 255) != n { return false; } out.clear(); out.resize(n, (records[0] >> 8) as u8); return true; }
        if !self.build_fl(records, n) { return false; }
        out.resize(n, 0);
        let fl = &self.fl[..n]; let mut x = p as usize;
        for slot in &mut out[..n] { let v = fl[x]; *slot = v as u8; x = (v >> 8) as usize; }
        true
    }
}

#[derive(Default)]
struct InterleavedWalk { plain: PlainRunWalk, start_bits: Vec<u64>, segs: [Vec<u8>; 8] }
impl InterleavedWalk {
    fn scratch_bytes(&self) -> usize { 4 * self.plain.fl.capacity() + 8 * self.start_bits.capacity() + self.segs.iter().map(|v| v.capacity()).sum::<usize>() }
    #[inline(never)]
    fn invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        if n < 4096 || records.len() == 1 { assert!(self.plain.try_invert(records, n, p, out)); return; }
        assert!(self.plain.build_fl(records, n));
        let fl = &self.plain.fl[..n];
        let words = n.div_ceil(64);
        if self.start_bits.len() < words { self.start_bits.resize(words, 0); }
        let mut starts = [0usize; 8];
        let stride = (n / 8).max(1);
        for c in 0..8 {
            let mut s = (p as usize + c * stride) % n;
            while (self.start_bits[s >> 6] >> (s & 63)) & 1 != 0 { s = (s + 1) % n; }
            starts[c] = s; self.start_bits[s >> 6] |= 1u64 << (s & 63);
        }
        for seg in &mut self.segs { seg.clear(); }
        let (mut cur, mut end_row, mut active, mut n_act, mut steps) = (starts, [usize::MAX; 8], [0, 1, 2, 3, 4, 5, 6, 7], 8usize, 0usize);
        while n_act > 0 && steps <= n {
            let mut i = 0;
            while i < n_act {
                let c = active[i]; let v = fl[cur[c]];
                self.segs[c].push(v as u8);
                let nx = (v >> 8) as usize; cur[c] = nx; steps += 1;
                if (self.start_bits[nx >> 6] >> (nx & 63)) & 1 != 0 { end_row[c] = nx; n_act -= 1; active[i] = active[n_act]; } else { i += 1; }
            }
        }
        for &s in &starts { self.start_bits[s >> 6] &= !(1u64 << (s & 63)); }
        let (mut order, mut n_ord, mut slen, mut cur_c, mut closed) = ([0usize; 8], 0, 0, 0, false);
        while n_ord < 8 {
            if end_row[cur_c] == usize::MAX { break; }
            order[n_ord] = cur_c; n_ord += 1; slen += self.segs[cur_c].len();
            let er = end_row[cur_c];
            if er == starts[0] { closed = true; break; }
            if let Some(nc) = starts.iter().position(|&s| s == er) { cur_c = nc; } else { break; }
        }
        if closed && slen > 0 && slen <= n {
            out.clear(); out.reserve(n);
            for &c in &order[..n_ord] { out.extend_from_slice(&self.segs[c]); }
            repeat(n, out); return;
        }
        out.resize(n, 0); let mut x = p as usize;
        for slot in &mut out[..n] { let v = fl[x]; *slot = v as u8; x = (v >> 8) as usize; }
    }
}

#[cold]
#[inline(never)]
fn try_short_periodic(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    if records.len() * 64 >= n || n < 4096 || (p as usize) >= n { return false; }
    let (mut sym, mut r_start, mut r_rank, mut counts) = ([0u8; 64], [0u32; 64], [0u32; 64], [0u32; 256]);
    let (mut nr, mut pos) = (0usize, 0u32);
    for &rec in records {
        let (b, l) = ((rec >> 8) as u8, u32::from(rec & 255));
        if nr == 0 || sym[nr - 1] != b {
            if nr >= 64 { return false; }
            sym[nr] = b; r_start[nr] = pos; r_rank[nr] = counts[b as usize]; nr += 1;
        }
        counts[b as usize] += l; pos += l;
    }
    let mut c_start = [0u32; 256]; let mut tot = 0u32;
    for c in 0..256 { c_start[c] = tot; tot += counts[c]; }
    if tot as usize != n { return false; }
    let mut cyc = [0u8; 64]; let mut x = p;
    for step in 0..64usize {
        let (mut lo, mut hi) = (0usize, nr);
        while lo + 1 < hi { let mid = (lo + hi) / 2; if r_start[mid] <= x { lo = mid; } else { hi = mid; } }
        let b = sym[lo]; cyc[step] = b;
        x = c_start[b as usize] + r_rank[lo] + (x - r_start[lo]);
        if x == p {
            let d = step + 1;
            if n % d == 0 {
                out.clear(); out.reserve(n);
                for k in (0..d).rev() { out.push(cyc[k]); }
                repeat(n, out); return true;
            }
            return false;
        }
    }
    false
}

#[inline]
fn select_limit(records: &[u16], n: usize) -> (u32, u32) {
    let nrec = records.len();
    if n < 32_768 || nrec < 256 { return (16, 0); }
    let step = nrec / 256;
    let mut idx = step / 2;
    let (mut s_w1, mut s_r17) = (0u32, 0u32);
    let step4 = step * 4;
    for _ in 0..64 {
        let w = u32::from(records[idx] & 255);
        s_w1 += u32::from(w == 1); s_r17 += u32::from(w > 16 && w <= 32); idx += step4;
    }
    if ((nrec as u64 * u64::from(s_w1)) >> 6) > 39_500 || s_r17 > 2 { return (16, 64); }
    idx = step / 2; s_w1 = 0; s_r17 = 0;
    for _ in 0..256 {
        let w = u32::from(records[idx] & 255);
        s_w1 += u32::from(w == 1); s_r17 += u32::from(w > 16 && w <= 32); idx += step;
    }
    let est_w1 = (nrec as u64 * u64::from(s_w1)) >> 8;
    if est_w1 <= 33_500 {
        if s_r17 <= 3 { (32, 256) } else if s_r17 <= 5 { (24, 256) } else { (16, 256) }
    } else { (16, 256) }
}

const POST_S1_BI: BiMode = BiMode::PostS1ExactMd {
    r_max_cap: 50_000, ns_max: 2_100, md_max: 16_000, md_neg_cap: -18_000, ns_neg_max: 2_400,
};

struct ArmC53<const LOG_CHUNK: u32, const BYTES: usize, const DICT_MODE: u8, const S4_MODE: u8, const S5_MODE: u8, const S1_GROW: u8> {
    open: engine_c53::Kernel<LOG_CHUNK, BYTES, DICT_MODE, S4_MODE, S5_MODE, S1_GROW>,
    seg: InterleavedWalk,
    force_fallback: bool,
}

impl<const LOG_CHUNK: u32, const BYTES: usize, const DICT_MODE: u8, const S4_MODE: u8, const S5_MODE: u8, const S1_GROW: u8> Default
    for ArmC53<LOG_CHUNK, BYTES, DICT_MODE, S4_MODE, S5_MODE, S1_GROW>
{
    fn default() -> Self { Self { open: engine_c53::Kernel::default(), seg: InterleavedWalk::default(), force_fallback: false } }
}

impl<const LOG_CHUNK: u32, const BYTES: usize, const DICT_MODE: u8, const S4_MODE: u8, const S5_MODE: u8, const S1_GROW: u8>
    ArmC53<LOG_CHUNK, BYTES, DICT_MODE, S4_MODE, S5_MODE, S1_GROW>
{
    #[inline(never)]
    fn run_stats(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (u8, Option<Counts>) {
        if self.force_fallback {
            let st = self.open.try_run_const::<16>(records, n, p, Cap::NONE, BiMode::AlwaysAt(65_536), out).expect("forced fallback");
            return (0, Some(st));
        }
        if n < 32_768 { assert!(self.open.invert_tiny_fl(records, n, p, out)); return (1, None); }
        if records.len() * 64 < n && try_short_periodic(records, n, p, out) { return (2, None); }
        if records.len() * 17 >= n * 13 { self.seg.invert(records, n, p, out); return (3, None); }
        let (lim, _) = select_limit(records, n);
        let res = match lim {
            16 => self.open.try_run_const::<16>(records, n, p, Cap::C53_CROSSOVER, POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, Cap::C53_CROSSOVER, POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, Cap::C53_CROSSOVER, POST_S1_BI, out),
        };
        if let Some(st) = res {
            let rt = match (lim, st.used_bi) { (16, true) => 0, (16, false) => 5, (24, _) => 6, _ => 7 };
            return (rt, Some(st));
        }
        self.seg.invert(records, n, p, out); (3, None)
    }
}

// ============================================================================
// 5. Classical Atlas Extension `C53-D4` (Extending Chain D `CA-26→27→31→37→41→44→47`
//    to `8` Pure-`CA` Nodes / `7` Links!):
//    O. Hölder (1889) & J. L. W. V. Jensen (1906) Moment-Entropy Ceiling +
//    Exact Marginal-Moment Information Barrier Theorem (Pre-S1 Ceiling)!
// ============================================================================

#[derive(Default, Clone)]
struct MomentLogProfile {
    m0: i128,
    m1: i128,
    m2: i128,
    m3: i128,
    s_log: f64,
    m_raw: i128,
    rt_raw: i128,
    m_act: i128,
    rt_act: i128,
    m_min_x: i128,
    m_max_x: i128,
    rt_min_x: i128,
    rt_max_x: i128,
    h_obs: f64,
}

fn profile_moments_and_log(records: &[u16], p: u32, lim: u32, cap: u32) -> MomentLogProfile {
    let mut pr = MomentLogProfile { m0: records.len() as i128, ..Default::default() };
    let mut hist = vec![0u32; cap as usize + 1];
    let mut c_cap = 0usize;
    let mut thin_ws = Vec::new();
    for &rec in records {
        let w = u32::from(rec & 255);
        let wi = i128::from(w);
        pr.m2 += wi * wi;
        pr.m3 += wi * wi * wi;
        pr.s_log += (w as f64).ln();
        hist[w as usize] += 1;
        if w == cap { c_cap += 1; }
        if w <= lim { thin_ws.push(w); }
    }
    let r_f = records.len() as f64;
    let mut h_obs = 0.0f64;
    for &cnt in &hist[1..] {
        if cnt > 0 {
            let pk = cnt as f64 / r_f;
            h_obs -= pk * pk.log2();
        }
    }
    pr.h_obs = h_obs;

    // Exact infinity-moment record-multiset envelope [m_min(X), m_max(X)] & [rt_min(X), rt_max(X)]
    thin_ws.sort_unstable_by(|a, b| b.cmp(a));
    let m_rec: i128 = thin_ws.iter().map(|&w| i128::from(w)).sum();
    let top_sum: i128 = thin_ws.iter().take(c_cap).map(|&w| i128::from(w)).sum();
    let has_thick = thin_ws.len() < records.len();
    pr.m_min_x = m_rec - top_sum;
    pr.m_max_x = m_rec + if has_thick { 1 } else { 0 };
    pr.rt_min_x = (thin_ws.len().saturating_sub(c_cap)) as i128;
    pr.rt_max_x = (thin_ws.len() + if has_thick { 1 } else { 0 }) as i128;

    let (mut pos, mut i) = (0u32, 0usize);
    while i < records.len() {
        let rec = records[i]; let c = (rec >> 8) as u8; let mut w = u32::from(rec & 255); i += 1;
        if w == cap {
            while i < records.len() && (records[i] >> 8) as u8 == c { w += u32::from(records[i] & 255); i += 1; }
        }
        let wi = i128::from(w);
        if w <= lim { pr.m_raw += wi; pr.rt_raw += 1; pr.m_act += wi; pr.rt_act += 1; }
        else if p >= pos && p < pos + w { pr.m_act += 1; pr.rt_act += 1; }
        pos += w;
    }
    pr.m1 = i128::from(pos);
    pr
}

/// Exact Jensen (1906) truncated Max-Entropy ceilings on `{1..=C}` (in bits):
///   1) `H_MaxEnt(mu; C)` — truncated Euler geometric `q_theta(k) ∝ exp(-theta * k)` matching `M_1 = mu`
///   2) `H_Zeta(mu_log; C)` — truncated Pareto-Zeta `q_alpha(k) ∝ k^{-alpha}` matching `ln M_0 = mu_log`
///   3) `H_Joint(mu, mu_log; C)` — joint 2-parameter exponential family `q(k) ∝ k^{-alpha} exp(-theta * k)`
fn jensen_maxent_ceilings(mu: f64, mu_log: f64, cap: usize) -> (f64, f64, f64) {
    let inv_ln2 = std::f64::consts::LOG2_E;
    let cf = cap as f64;
    if cap <= 1 || mu <= 1.0 + 1e-12 || mu >= cf - 1e-12 {
        return (0.0, 0.0, 0.0);
    }
    let ln_k: Vec<f64> = (1..=cap).map(|k| (k as f64).ln()).collect();

    // 1. Truncated geometric (Euler CA-46): minimize f(theta) = ln Z(theta) + theta * mu
    let eval_geom = |theta: f64| -> (f64, f64) {
        let mut max_e = f64::NEG_INFINITY;
        for k in 1..=cap { max_e = max_e.max(-theta * k as f64); }
        let (mut z, mut ek) = (0.0, 0.0);
        for k in 1..=cap {
            let w = (-theta * k as f64 - max_e).exp();
            z += w; ek += w * k as f64;
        }
        (max_e + z.ln() + theta * mu, ek / z)
    };
    let (mut lo, mut hi) = (-20.0f64, 20.0f64);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let (_, mean) = eval_geom(mid);
        if mean > mu { lo = mid; } else { hi = mid; }
    }
    let theta_star = 0.5 * (lo + hi);
    let h_geom = eval_geom(theta_star).0 * inv_ln2;

    // 2. Truncated Zeta (ln M_0): minimize g(alpha) = ln Z(alpha) + alpha * mu_log
    let eval_zeta = |alpha: f64| -> (f64, f64) {
        let mut max_e = f64::NEG_INFINITY;
        for &lk in &ln_k { max_e = max_e.max(-alpha * lk); }
        let (mut z, mut el) = (0.0, 0.0);
        for &lk in &ln_k {
            let w = (-alpha * lk - max_e).exp();
            z += w; el += w * lk;
        }
        (max_e + z.ln() + alpha * mu_log, el / z)
    };
    let (mut alo, mut ahi) = (-25.0f64, 25.0f64);
    for _ in 0..80 {
        let mid = 0.5 * (alo + ahi);
        let (_, mlog) = eval_zeta(mid);
        if mlog > mu_log { alo = mid; } else { ahi = mid; }
    }
    let alpha_star = 0.5 * (alo + ahi);
    let h_zeta = eval_zeta(alpha_star).0 * inv_ln2;

    // 3. Joint 2-parameter (theta, alpha) convex dual minimization via damped Newton-Armijo
    let eval_joint = |th: f64, al: f64| -> (f64, f64, f64, f64, f64, f64) {
        let mut max_e = f64::NEG_INFINITY;
        for (i, &lk) in ln_k.iter().enumerate() { max_e = max_e.max(-th * (i + 1) as f64 - al * lk); }
        let (mut z, mut ek, mut el, mut ek2, mut ekl, mut el2) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for (i, &lk) in ln_k.iter().enumerate() {
            let kf = (i + 1) as f64;
            let w = (-th * kf - al * lk - max_e).exp();
            z += w; ek += w * kf; el += w * lk; ek2 += w * kf * kf; ekl += w * kf * lk; el2 += w * lk * lk;
        }
        let mk = ek / z; let ml = el / z;
        let obj = max_e + z.ln() + th * mu + al * mu_log;
        let g1 = mu - mk; let g2 = mu_log - ml;
        let h11 = (ek2 / z - mk * mk).max(1e-12);
        let h12 = ekl / z - mk * ml;
        let h22 = (el2 / z - ml * ml).max(1e-12);
        (obj, g1, g2, h11, h12, h22)
    };
    let (mut th, mut al) = if h_geom <= h_zeta { (theta_star, 0.0) } else { (0.0, alpha_star) };
    let mut best_obj = eval_joint(th, al).0;
    for _ in 0..40 {
        let (obj, g1, g2, h11, h12, h22) = eval_joint(th, al);
        best_obj = best_obj.min(obj);
        if g1.abs() < 1e-10 && g2.abs() < 1e-10 { break; }
        let det = h11 * h22 - h12 * h12;
        if det <= 1e-18 { break; }
        let dth = -(h22 * g1 - h12 * g2) / det;
        let dal = -(-h12 * g1 + h11 * g2) / det;
        let mut step = 1.0;
        let mut improved = false;
        while step >= 1e-6 {
            let (nth, nal) = ((th + step * dth).clamp(-30.0, 30.0), (al + step * dal).clamp(-30.0, 30.0));
            let nobj = eval_joint(nth, nal).0;
            if nobj <= obj + 1e-13 {
                th = nth; al = nal; best_obj = best_obj.min(nobj); improved = true; break;
            }
            step *= 0.5;
        }
        if !improved { break; }
    }
    let h_joint = (best_obj * inv_ln2).min(h_geom).min(h_zeta);
    (h_geom, h_zeta, h_joint)
}

struct D4Result {
    rt_mk1: f64,
    rt_c3: f64,
    rt_logmk: f64,
    rt_phi_log: f64,
    m_ub_c3: f64,
    m_ub_log: f64,
    rt_ub_c3: f64,
    rt_ub_log: f64,
    jensen_def: f64,
    h_geom: f64,
    h_zeta: f64,
    h_joint: f64,
}

fn verify_c53_d4_exact(pr: &MomentLogProfile, lim: i128, cap: i128, check_entropy: bool) -> D4Result {
    let (m0, m1, m2, m3) = (pr.m0, pr.m1, pr.m2, pr.m3);
    assert!(m0 >= 1 && m1 >= m0 && m1 <= cap * m0);

    // 1. Exact i128 Hölder (1889) / Lyapunov log-convexity chain:
    //    S_1^2 <= S_0 * S_2, S_2^2 <= S_1 * S_3, S_1^3 <= S_0^2 * S_3, S_2^3 <= S_0 * S_3^2
    assert!(m1 * m1 <= m0 * m2, "Holder S1^2 <= S0*S2 violated");
    assert!(m2 * m2 <= m1 * m3, "Holder S2^2 <= S1*S3 violated");
    assert!(m1 * m1 * m1 <= m0 * m0 * m3, "Holder S1^3 <= S0^2*S3 violated");
    assert!(m2 * m2 * m2 <= m0 * m3 * m3, "Holder S2^3 <= S0*S3^2 violated");

    // 2. Power-mean monotonicity 1 <= M_0 <= M_1 <= M_2 <= M_3 <= C (Hölder 1889 + Jensen 1906)
    let r_f = m0 as f64;
    let mu_log = pr.s_log / r_f;
    let m0_mean = mu_log.exp();
    let m1_mean = m1 as f64 / r_f;
    let m2_mean = (m2 as f64 / r_f).sqrt();
    let m3_mean = (m3 as f64 / r_f).cbrt();
    assert!(m0_mean >= 1.0 - 1e-11 && m0_mean <= m1_mean + 1e-9);
    assert!(m1_mean <= m2_mean + 1e-9 && m2_mean <= m3_mean + 1e-9 && m3_mean <= cap as f64 + 1e-9);
    let jensen_def = m1_mean.ln() - mu_log;
    assert!(jensen_def >= -1e-12, "Jensen log-deficit < 0");

    // 3. Exact Jensen (1906) Log-Markov LB with Geometric-Mean Multi-Record Threshold min(L+1, sqrt(C))
    let lf = lim as f64;
    let cf = cap as f64;
    let geom_thr = (lf + 1.0).min(cf.sqrt());
    let rt_logmk = if geom_thr > 1.0 + 1e-12 {
        (r_f - pr.s_log / geom_thr.ln()).max(0.0)
    } else { 0.0 };
    assert!(rt_logmk <= pr.rt_raw as f64 + 1e-7, "Jensen Log-Markov LB violated: {rt_logmk} > {}", pr.rt_raw);

    // 4. Exact 2-parameter Convex (S_1, S_log) LB: phi(1)=1, phi(L+1)=0, phi(C)=-1
    //    Strictly convex iff 2*L + 1 <= C (matching CA-41/CA-44 convexity boundary!)
    let mut rt_phi_log = 0.0f64;
    if lim >= 1 && 2 * lim + 1 <= cap && lim <= cap - 2 {
        let dlog = (cf - 1.0) * (lf + 1.0).ln() - lf * cf.ln();
        let beta = (cf.ln() - 2.0 * (lf + 1.0).ln()) / dlog;
        let gamma = (2.0 * lf - (cf - 1.0)) / dlog;
        assert!(gamma <= 1e-12, "gamma must be <= 0 in convex regime");
        rt_phi_log = (r_f + beta * (m1 - m0) as f64 + gamma * pr.s_log).max(0.0);
        assert!(rt_phi_log <= pr.rt_raw as f64 + 1e-6, "Jensen convex phi_log LB violated: {rt_phi_log} > {}", pr.rt_raw);
    }

    // 5. Exact Concave (S_1, S_log) Upper Bounds on m_raw & rt_raw (valid for ALL 2 <= L < C!)
    let mut m_ub_log = (m1.min(lim * m0)) as f64;
    let mut rt_ub_log = r_f;
    if lim >= 2 && lim < cap {
        let slope_l = lf.ln() / (lf - 1.0);
        let denom_c = slope_l * (cf - 1.0) - cf.ln();
        assert!(denom_c > 0.0);
        let c_m = cf / denom_c;
        let c_rt = 1.0 / denom_c;
        let u_m = (m1 as f64) + c_m * (pr.s_log - slope_l * (m1 - m0) as f64);
        let u_rt = r_f + c_rt * (pr.s_log - slope_l * (m1 - m0) as f64);
        assert!(u_m >= pr.m_raw as f64 - 1e-6, "Jensen concave m_ub_log violated");
        assert!(u_rt >= pr.rt_raw as f64 - 1e-6, "Jensen concave rt_ub_log violated");
        m_ub_log = m_ub_log.min(u_m);
        rt_ub_log = rt_ub_log.min(u_rt);
    }

    // 6. CA-37/41 (1-moment) & CA-47 (3-moment) comparison baselines
    let rt_mk1 = if 2 * lim + 1 <= cap { (((lim + 1) * m0 - m1).max(0) as f64) / lf }
                 else { (((cap + 1) * m0 - 2 * m1).max(0) as f64) / (cf - 1.0) };
    let mut rt_c3 = rt_mk1;
    let mut m_ub_c3 = (m1.min(lim * m0)) as f64;
    let mut rt_ub_c3 = r_f;
    if lim >= 2 && lim <= cap - 2 {
        let m_u1 = ((cap * m1 - m2) as f64) / ((cap - lim) as f64);
        let m_u2 = (((cap * cap + lim) * m1 - cap * m2 - lim * cap * m0) as f64) / (((cap - 1) * (cap - lim)) as f64);
        m_ub_c3 = m_ub_c3.min(m_u1).min(m_u2);
        let rt_rc = r_f - ((m2 - (lim + 1) * m1 + lim * m0) as f64) / (((cap - 1) * (cap - lim)) as f64);
        rt_ub_c3 = rt_ub_c3.min(rt_rc);
        for k in (lim + 1)..cap {
            let (d1, dl) = (k * (k - 1), (k - lim) * (k + 1 - lim));
            let denom = (lim - 1) * d1 * dl;
            let (am, bm) = (lim * d1 - dl, lim * (dl - d1));
            let rhs_m = am * m3 + (bm - (2 * k + 1) * am) * m2 + (k * (k + 1) * am - (2 * k + 1) * bm) * m1 + (k * (k + 1) * bm) * m0;
            m_ub_c3 = m_ub_c3.min((rhs_m as f64) / (denom as f64));
            let (ar, br) = (d1 - dl, lim * dl - d1);
            let rhs_r = ar * m3 + (br - (2 * k + 1) * ar) * m2 + (k * (k + 1) * ar - (2 * k + 1) * br) * m1 + (k * (k + 1) * br) * m0;
            rt_ub_c3 = rt_ub_c3.min((rhs_r as f64) / (denom as f64));
        }
        let delta2 = m2 - (lim + 2) * m1 + (lim + 1) * m0;
        if 2 * lim * (lim - 1) <= (cap - 1) * (cap - 2) {
            let denom = lim * (cap - 1) * (cap - lim - 1);
            let rhs = (cap - 1) * (cap - lim - 1) * ((lim + 1) * m0 - m1) + (cap - 2 * lim - 1) * delta2;
            rt_c3 = rt_c3.max((rhs.max(0) as f64) / (denom as f64));
        }
        for k in 1..lim {
            let (el, ec) = ((lim + 1 - k) * (lim - k), (cap - k) * (cap - k - 1));
            if 2 * el <= ec {
                let denom = (cap - lim - 1) * el * ec;
                let (a, b) = (2 * el - ec, cap * ec - 2 * (lim + 1) * el);
                let rhs = (-a) * m3 + ((2 * k + 1) * a - b) * m2 + ((2 * k + 1) * b - k * (k + 1) * a) * m1 + (denom - k * (k + 1) * b) * m0;
                rt_c3 = rt_c3.max((rhs.max(0) as f64) / (denom as f64));
            }
        }
        if lim <= cap - 3 {
            for j in (lim + 2)..cap {
                let (lhs_c, rhs_c) = ((cap - lim - 1) * (cap - j) * (cap - j - 1), lim * j * (j - 1));
                if lhs_c >= rhs_c {
                    let rhs = -m3 + (lim + 2 * j + 2) * m2 - ((lim + 1) * (2 * j + 1) + j * (j + 1)) * m1 + ((lim + 1) * j * (j + 1)) * m0;
                    rt_c3 = rt_c3.max((rhs.max(0) as f64) / (rhs_c as f64));
                }
            }
        }
    }

    // 7. Exact infinity-moment record-multiset envelope check
    assert!(pr.m_min_x <= pr.m_act && pr.m_act <= pr.m_max_x, "Record-multiset m envelope violated!");
    assert!(pr.rt_min_x <= pr.rt_act && pr.rt_act <= pr.rt_max_x, "Record-multiset rt envelope violated!");

    // 8. Optional Jensen Shannon-Entropy Ceiling check (H_obs <= H_Joint <= min(H_Geom, H_Zeta))
    let (h_geom, h_zeta, h_joint) = if check_entropy {
        let (hg, hz, hj) = jensen_maxent_ceilings(m1_mean, mu_log, cap as usize);
        assert!(pr.h_obs <= hg + 1e-6, "H_obs > H_Geom: {} > {}", pr.h_obs, hg);
        assert!(pr.h_obs <= hz + 1e-6, "H_obs > H_Zeta: {} > {}", pr.h_obs, hz);
        assert!(pr.h_obs <= hj + 1e-5, "H_obs > H_Joint: {} > {}", pr.h_obs, hj);
        (hg, hz, hj)
    } else { (0.0, 0.0, 0.0) };

    D4Result { rt_mk1, rt_c3, rt_logmk, rt_phi_log, m_ub_c3, m_ub_log, rt_ub_c3, rt_ub_log, jensen_def, h_geom, h_zeta, h_joint }
}

fn run_c53_d4_holder_jensen_audit(six: &[Block], wiki431: &[Block], non_wiki: &[Block], code_docs: &[Block], dense: &Block) {
    println!("\n=== ATLAS EXTENSION C53-D4 (HÖLDER 1889 / JENSEN 1906 MOMENT-ENTROPY CEILING) ===");
    println!("=== EXTENDING CHAIN D (`CA-26→27→31→37→41→44→47→C53-D4`, 8 PURE-CA NODES / 7 LINKS!) ===");

    // 1. Constructive Proof of Theorem D4-2A: Explicit Valid Primitive Single-Cycle BWT Pairs
    //    with IDENTICAL CAP=C Record-Length Multiset X (all moments S_0..S_inf identical!)
    //    achieving 100.0% of the exact theoretical envelope [m_min(X), m_max(X)]!
    println!("  [Theorem D4-2A: Explicit Valid Primitive Single-Cycle BWT Pairs — Identical Record Multiset X, Maximal `m` Gap]");
    for &(cap, lim, q) in &[(15u32, 4u32, 5usize), (31, 8, 8), (127, 16, 20), (127, 16, 50)] {
        let mut t1 = vec![0u8];
        for j in (1..=2 * q).rev() {
            let cnt = if j <= q { cap as usize } else { lim as usize };
            t1.extend(std::iter::repeat_n(j as u8, cnt));
        }
        let pos1 = t1.iter().position(|&b| b == 1).unwrap() + 1;
        let mut t1_rot = Vec::with_capacity(t1.len());
        t1_rot.extend_from_slice(&t1[pos1..]);
        t1_rot.extend_from_slice(&t1[..pos1]);
        let (l1, p1) = bwt_small(&t1_rot);
        assert!(is_single_cycle_bwt(&l1), "B1 must be valid primitive single-cycle BWT!");
        assert_eq!(plain_invert_fl(&l1, p1), t1_rot);

        let mut t2 = vec![0u8];
        for j in (1..=q).rev() {
            t2.extend(std::iter::repeat_n(j as u8, (cap + lim) as usize));
        }
        let mut t2_rot = Vec::with_capacity(t2.len());
        t2_rot.extend_from_slice(&t2[1..]);
        t2_rot.push(t2[0]);
        let (l2, p2) = bwt_small(&t2_rot);
        assert!(is_single_cycle_bwt(&l2), "B2 must be valid primitive single-cycle BWT!");
        assert_eq!(plain_invert_fl(&l2, p2), t2_rot);

        let r1 = records_of_cap(&l1, cap);
        let r2 = records_of_cap(&l2, cap);
        let mut x1: Vec<u16> = r1.iter().map(|&r| r & 255).collect();
        let mut x2: Vec<u16> = r2.iter().map(|&r| r & 255).collect();
        x1.sort_unstable(); x2.sort_unstable();
        assert_eq!(x1, x2, "Record-length multisets X1 and X2 MUST be bit-for-bit identical!");

        let pr1 = profile_moments_and_log(&r1, p1, lim, cap);
        let pr2 = profile_moments_and_log(&r2, p2, lim, cap);
        assert_eq!((pr1.m0, pr1.m1, pr1.m2, pr1.m3), (pr2.m0, pr2.m1, pr2.m2, pr2.m3));
        assert!((pr1.s_log - pr2.s_log).abs() < 1e-12);
        assert_eq!(pr1.m_act, pr1.m_max_x, "B1 must hit exact m_max(X)");
        assert_eq!(pr2.m_act, pr2.m_min_x, "B2 must hit exact m_min(X)");
        verify_c53_d4_exact(&pr1, i128::from(lim), i128::from(cap), true);
        verify_c53_d4_exact(&pr2, i128::from(lim), i128::from(cap), true);
        println!(
            "    (C={:3}, L={:2}, q={:2}, n={:4}): X1==X2 EXACT (all S_0..S_inf equal!) | m(B1)={:3} == m_max, m(B2)={:1} == m_min ({:5.1}x ratio, 100.0% of gap!)",
            cap, lim, q, l1.len(), pr1.m_act, pr2.m_act, pr1.m_act as f64 / pr2.m_act as f64
        );
    }

    // 2. Constructive Proof of Theorem D4-2B: Explicit Valid Primitive Single-Cycle BWT Pairs
    //    with IDENTICAL Merged Maximal Run-Length Multiset W AND Identical Record Multiset X AND Identical `m`,
    //    yet wildly different `S2b` grammar complexity (`steps`, `rule_words`, `num_codes`)!
    println!("  [Theorem D4-2B: Identical Merged Run-Length Multiset W & X & `m`, Divergent `S2b` Grammar Complexity]");
    let mut k_diag = engine_c53::Kernel::<9, 1024, 2, 1, 2, 0>::default();
    let mut out_d = Vec::new();
    for &(fp, fq) in &[(55usize, 89usize), (89, 144), (144, 233), (233, 377)] {
        // Construct lower Christoffel word on {1, 2} with fp ones and fq twos, rotated to lex min:
        let mut cw = Vec::with_capacity(fp + fq);
        let mut y = 0usize;
        for i in 0..(fp + fq) {
            if ((i + 1) * fq) / (fp + fq) > y { cw.push(2u8); y += 1; } else { cw.push(1u8); }
        }
        let min_r = (0..cw.len()).min_by(|&a, &b| {
            for k in 0..cw.len() {
                let (ca, cb) = (cw[(a + k) % cw.len()], cw[(b + k) % cw.len()]);
                if ca != cb { return ca.cmp(&cb); }
            }
            a.cmp(&b)
        }).unwrap();
        let mut tb = Vec::with_capacity(1 + cw.len());
        tb.push(0u8);
        tb.extend_from_slice(&cw[min_r..]);
        tb.extend_from_slice(&cw[..min_r]);
        let (lb, pb) = bwt_small(&tb);
        assert!(is_single_cycle_bwt(&lb));
        assert_eq!(plain_invert_fl(&lb, pb), tb);

        // Matched perfectly-clustered word with IDENTICAL merged maximal run-length multiset W = {1, 1, fp, fq-1}:
        // Put 0 at the end so pa lands on singleton 0 (not inside thick run 1^fp), giving st_a.m == st_b.m == 2!
        let mut ta = Vec::with_capacity(tb.len());
        ta.push(3u8);
        ta.extend(std::iter::repeat_n(2u8, fq - 1));
        ta.extend(std::iter::repeat_n(1u8, fp));
        ta.push(0u8);
        let (la, pa) = bwt_small(&ta);
        assert!(is_single_cycle_bwt(&la));
        assert_eq!(plain_invert_fl(&la, pa), ta);

        let max_runs_sorted = |l: &[u8]| -> Vec<usize> {
            let mut v = Vec::new();
            let mut i = 0;
            while i < l.len() {
                let b = l[i]; let mut w = 0;
                while i < l.len() && l[i] == b { i += 1; w += 1; }
                v.push(w);
            }
            v.sort_unstable(); v
        };
        let (wa, wb) = (max_runs_sorted(&la), max_runs_sorted(&lb));
        assert_eq!(wa, wb, "Merged maximal run-length multisets W_A and W_B MUST be identical!");
        let (ra, rb) = (records_of(&la), records_of(&lb));
        let mut xa: Vec<u16> = ra.iter().map(|&r| r & 255).collect();
        let mut xb: Vec<u16> = rb.iter().map(|&r| r & 255).collect();
        xa.sort_unstable(); xb.sort_unstable();
        assert_eq!(xa, xb, "CAP=127 record multisets X_A and X_B MUST be identical!");

        let st_a = k_diag.try_run_const::<16>(&ra, la.len(), pa, Cap::NONE, BiMode::AlwaysAt(0), &mut out_d).unwrap();
        assert_eq!(out_d, ta);
        let st_b = k_diag.try_run_const::<16>(&rb, lb.len(), pb, Cap::NONE, BiMode::AlwaysAt(0), &mut out_d).unwrap();
        assert_eq!(out_d, tb);
        assert_eq!(st_a.m, st_b.m);
        println!(
            "    Fib({:3},{:3}) n={:3}: W_A==W_B & X_A==X_B & m={} EXACT | Clustered(A): steps={:2}, rule_words={:3}, sp={:3}B vs Christoffel(B): steps={:2}, rule_words={:3}, sp={:3}B!",
            fp, fq, la.len(), st_a.m, st_a.s2b_steps, st_a.rule_words, st_a.spelling_used,
            st_b.s2b_steps, st_b.rule_words, st_b.spelling_used
        );
    }

    // 3. 105,000 Extremal & Random Multisets Verified in Exact i128 + f64 (0 violations!)
    let caps = [5u32, 7, 8, 15, 16, 31, 63, 127];
    let mut rng = 0xC53D_4188_9190_6A55u64;
    let mut next_u64 = || -> u64 { rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); rng };
    let mut recs_buf = Vec::with_capacity(512);
    let mut ent_checked = 0usize;
    for case in 0..105_000usize {
        let c = caps[case % caps.len()];
        let lim = 1 + ((next_u64() >> 32) as u32 % (c - 1));
        let r = 1 + ((next_u64() >> 40) as usize % 28);
        let mode = case % 6;
        recs_buf.clear();
        let (mut sym, mut tot) = (0u8, 0u32);
        for j in 0..r {
            let w: u32 = match mode {
                0 => 1 + ((next_u64() >> 32) as u32 % (3 * c)),
                1 => { let ch = [1, 2, lim.max(1), lim + 1, (lim + 2).min(c), c - 1, c, c + 1, 2 * c + 1]; ch[((next_u64() >> 32) as usize) % ch.len()] }
                2 => 1 + ((next_u64() >> 32) as u32 % (lim + 2)),
                3 => lim + 1 + ((next_u64() >> 32) as u32 % (3 * c)),
                4 => if j % 2 == 0 { 1 } else { (c / 2).max(lim + 1) },
                _ => if j + 1 == r { 1 + ((next_u64() >> 32) as u32 % (8 * c)) } else { 1 },
            };
            tot += w;
            let mut left = w;
            while left > c { recs_buf.push((u16::from(sym) << 8) | (c as u16)); left -= c; }
            recs_buf.push((u16::from(sym) << 8) | (left as u16));
            sym = sym.wrapping_add(1); if sym == 255 { sym = 1; }
        }
        let p = ((next_u64() >> 32) as u32) % tot;
        let pr = profile_moments_and_log(&recs_buf, p, lim, c);
        let chk_e = case < 2_000;
        verify_c53_d4_exact(&pr, i128::from(lim), i128::from(c), chk_e);
        if chk_e { ent_checked += 1; }
    }
    println!("  Exact Stress PASS: 105,000 extremal+random multisets (Holder i128 + Jensen Log-Markov + phi_log + concave UBs + X-envelope) & {ent_checked} MaxEnt ceilings (0 violations!)");

    // 4. ALL 447 Real Blocks + dense-300k at L=16, CAP=127
    let w233: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();
    let w198: Vec<&Block> = wiki431.iter().filter(|b| b.route_class >= 2).collect();
    let w431_r: Vec<&Block> = wiki431.iter().collect();
    let six_r: Vec<&Block> = six.iter().collect();
    let nw_r: Vec<&Block> = non_wiki.iter().collect();
    let cd_r: Vec<&Block> = code_docs.iter().collect();
    let dense_r = vec![dense];

    println!("  Corpus         | N   | rt LB: Mk1 / LogMk / PhiLog / C3(CA47) | m UB: Log/C3 | rt UB: Log/C3 | X-Inf Env [min, max] (gap) | H_obs/Jnt/Zeta/Geom (b) | J_def");
    for (name, blks) in [
        ("6 Admitted", &six_r), ("6 JSONL/Log", &nw_r), ("4 Code/Docs", &cd_r),
        ("233 FullWiki", &w233), ("198 TinyWiki", &w198), ("431 AllWiki", &w431_r), ("dense-300k", &dense_r),
    ] {
        let (mut r_mk1, mut r_lmk, mut r_phi, mut r_c3) = (0.0, 0.0, 0.0, 0.0);
        let (mut mu_l, mut mu_3, mut ru_l, mut ru_3, mut xm_lo, mut xm_hi, mut x_gap) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let (mut ho, mut hj, mut hz, mut hg, mut jd) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let mut cnt = 0usize;
        for &b in blks {
            let pr = profile_moments_and_log(&b.records, b.p, 16, 127);
            let d = verify_c53_d4_exact(&pr, 16, 127, true);
            if pr.rt_act > 0 && pr.m_act > 0 {
                let rta = pr.rt_act as f64;
                let ma = pr.m_act as f64;
                r_mk1 += d.rt_mk1 / rta;
                r_lmk += d.rt_logmk / rta;
                r_phi += d.rt_phi_log / rta;
                r_c3 += d.rt_c3 / rta;
                mu_l += (d.m_ub_log + 1.0) / ma;
                mu_3 += (d.m_ub_c3 + 1.0) / ma;
                ru_l += (d.rt_ub_log + 1.0) / rta;
                ru_3 += (d.rt_ub_c3 + 1.0) / rta;
                xm_lo += pr.m_min_x as f64 / ma;
                xm_hi += pr.m_max_x as f64 / ma;
                x_gap += (pr.m_max_x - pr.m_min_x) as f64 / ma;
                ho += pr.h_obs; hj += d.h_joint; hz += d.h_zeta; hg += d.h_geom; jd += d.jensen_def;
                cnt += 1;
            }
        }
        let c = cnt.max(1) as f64;
        println!(
            "  {:14} | {:3} | {:.3} / {:.3} / {:.3} / {:.3}           | {:.2}x / {:.2}x | {:.3}x/{:.3}x | [{:.4}, {:.4}] ({:5.2}%) | {:.3}/{:.3}/{:.3}/{:.3} | {:.3}",
            name, blks.len(), r_mk1 / c, r_lmk / c, r_phi / c, r_c3 / c,
            mu_l / c, mu_3 / c, ru_l / c, ru_3 / c, xm_lo / c, xm_hi / c, (x_gap / c) * 100.0,
            ho / c, hj / c, hz / c, hg / c, jd / c
        );
    }
}

// ============================================================================
// 6. Arm Suite, Exactness Gate, WS Attribution & Headroom Oracle v8
// ============================================================================

const NUM_ARMS: usize = 10;
const ARM_NAMES: [&str; NUM_ARMS] = [
    "R5-Head-A1",
    "R5-Head-A2(AA)",
    "C52-Lean-Compact",
    "C52-SuperDropIn-v3",
    "Abl-Chunk256",
    "Abl-Chunk1024",
    "Abl-U16Raw",
    "Abl-U32Dict",
    "Abl-NoPipe1",
    "C53-SuperPipe-v4",
];

// Arm parameter mapping (LOG_CHUNK, BYTES, DICT_MODE, S4_MODE, S5_MODE, S1_GROW):
//   Arm 2 (C52-Lean-Compact):   (9, 1024, 0=U32,       1=SlimS4-8,  1=Rem16,      0=Clean2x)
//   Arm 3 (C52-SuperDropIn-v3): (9, 1024, 0=U32,       0=V3-16,     0=V3-Peeled,  1=Step4096)
//   Arm 4 (Abl-Chunk256):       (8,  512, 2=U16Tag4,   2=SlimS4-16, 2=Pipe1Rem16, 1=Step4096)  [1-knob: CHUNK=256]
//   Arm 5 (Abl-Chunk1024):      (10,2048, 2=U16Tag4,   2=SlimS4-16, 2=Pipe1Rem16, 1=Step4096)  [1-knob: CHUNK=1024]
//   Arm 6 (Abl-U16Raw):         (9, 1024, 1=U16Raw,    2=SlimS4-16, 2=Pipe1Rem16, 1=Step4096)  [1-knob: U16Raw<65534]
//   Arm 7 (Abl-U32Dict):        (9, 1024, 0=U32,       2=SlimS4-16, 2=Pipe1Rem16, 1=Step4096)  [1-knob: U32-InEdges]
//   Arm 8 (Abl-NoPipe1):        (9, 1024, 2=U16Tag4,   2=SlimS4-16, 1=Rem16,      1=Step4096)  [1-knob: No Pipe1]
//   Arm 9 (C53-SuperPipe-v4):   (9, 1024, 2=U16Tag4,   2=SlimS4-16, 2=Pipe1Rem16, 1=Step4096)  [WINNING FUSION!]
#[derive(Default)]
struct ArmSuite {
    kh1: cand_r5_head::Kernel,
    kh2: cand_r5_head::Kernel,
    c52_lean: ArmC53<9, 1024, 0, 1, 1, 0>,
    c52_v3: ArmC53<9, 1024, 0, 0, 0, 1>,
    abl_c256: ArmC53<8, 512, 2, 2, 2, 1>,
    abl_c1024: ArmC53<10, 2048, 2, 2, 2, 1>,
    abl_u16raw: ArmC53<9, 1024, 1, 2, 2, 1>,
    abl_u32dict: ArmC53<9, 1024, 0, 2, 2, 1>,
    abl_nopipe1: ArmC53<9, 1024, 2, 2, 1, 1>,
    c53_v4: ArmC53<9, 1024, 2, 2, 2, 1>,
}

impl ArmSuite {
    #[inline(always)]
    fn run_arm(&mut self, arm: usize, b: &Block, out: &mut Vec<u8>) -> u8 {
        match arm {
            0 => { self.kh1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out); 0 }
            1 => { self.kh2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out); 0 }
            2 => self.c52_lean.run_stats(&b.records, b.n, b.p, out).0,
            3 => self.c52_v3.run_stats(&b.records, b.n, b.p, out).0,
            4 => self.abl_c256.run_stats(&b.records, b.n, b.p, out).0,
            5 => self.abl_c1024.run_stats(&b.records, b.n, b.p, out).0,
            6 => self.abl_u16raw.run_stats(&b.records, b.n, b.p, out).0,
            7 => self.abl_u32dict.run_stats(&b.records, b.n, b.p, out).0,
            8 => self.abl_nopipe1.run_stats(&b.records, b.n, b.p, out).0,
            _ => self.c53_v4.run_stats(&b.records, b.n, b.p, out).0,
        }
    }
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[(v.len() * num) / den]
}

fn run_proxy_arm(arm_name: &str, fresh: bool, wiki431: &[Block]) {
    const PROXY_IDS: [u32; 10] = [34, 108, 178, 254, 327, 391, 60, 163, 265, 372];
    let blks: Vec<&Block> = PROXY_IDS.iter().map(|&id| wiki431.iter().find(|b| b.idx == id).unwrap()).collect();
    let arm_idx = match arm_name {
        "lean" => 2,
        "v3" => 3,
        "c256" => 4,
        "c1024" => 5,
        "u16raw" => 6,
        "u32dict" => 7,
        "nopipe1" => 8,
        "v4" => 9,
        "c512_tag4_s48" => 10,
        "c512_tag4_s416" => 11,
        "c256_u16in" => 12,
        "c512_u16in" => 13,
        "lean_s416" => 14,
        "lean_s1step" => 15,
        "super_512_p1" => 16,
        "super_512_nop1" => 17,
        "super_256_p1" => 18,
        "super_512_raw_p1" => 19,
        "super_512_u32_p1" => 20,
        _ => panic!("unknown proxy arm: {arm_name}"),
    };
    let mut out = Vec::with_capacity(900_064);
    let mut checksum = 0u64;
    if fresh {
        for &b in &blks {
            let mut out_b = Vec::new();
            match arm_idx {
                2 => { ArmC53::<9, 1024, 0, 1, 1, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                3 => { ArmC53::<9, 1024, 0, 0, 0, 1>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                4 => { ArmC53::<8, 512, 2, 1, 2, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                5 => { ArmC53::<10, 2048, 2, 1, 2, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                6 => { ArmC53::<9, 1024, 1, 1, 2, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                7 => { ArmC53::<9, 1024, 0, 1, 2, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                8 => { ArmC53::<9, 1024, 2, 1, 1, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                9 => { ArmC53::<8, 512, 2, 2, 2, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                10 => { ArmC53::<9, 1024, 2, 1, 2, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                11 => { ArmC53::<9, 1024, 2, 2, 2, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                12 => { ArmC53::<8, 512, 3, 1, 2, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                13 => { ArmC53::<9, 1024, 3, 1, 2, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                14 => { ArmC53::<9, 1024, 0, 2, 1, 0>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                15 => { ArmC53::<9, 1024, 0, 1, 1, 1>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                16 => { ArmC53::<9, 1024, 2, 2, 2, 1>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                17 => { ArmC53::<9, 1024, 2, 2, 1, 1>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                18 => { ArmC53::<8, 512, 2, 2, 2, 1>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                19 => { ArmC53::<9, 1024, 1, 2, 2, 1>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
                _ => { ArmC53::<9, 1024, 0, 2, 2, 1>::default().run_stats(&b.records, b.n, b.p, &mut out_b); }
            }
            checksum = checksum.wrapping_add(u64::from(out_b[0]) ^ u64::from(*out_b.last().unwrap()));
        }
    } else {
        let mut a2 = ArmC53::<9, 1024, 0, 1, 1, 0>::default();
        let mut a3 = ArmC53::<9, 1024, 0, 0, 0, 1>::default();
        let mut a4 = ArmC53::<8, 512, 2, 1, 2, 0>::default();
        let mut a5 = ArmC53::<10, 2048, 2, 1, 2, 0>::default();
        let mut a6 = ArmC53::<9, 1024, 1, 1, 2, 0>::default();
        let mut a7 = ArmC53::<9, 1024, 0, 1, 2, 0>::default();
        let mut a8 = ArmC53::<9, 1024, 2, 1, 1, 0>::default();
        let mut a9 = ArmC53::<8, 512, 2, 2, 2, 0>::default();
        let mut a10 = ArmC53::<9, 1024, 2, 1, 2, 0>::default();
        let mut a11 = ArmC53::<9, 1024, 2, 2, 2, 0>::default();
        let mut a12 = ArmC53::<8, 512, 3, 1, 2, 0>::default();
        let mut a13 = ArmC53::<9, 1024, 3, 1, 2, 0>::default();
        let mut a14 = ArmC53::<9, 1024, 0, 2, 1, 0>::default();
        let mut a15 = ArmC53::<9, 1024, 0, 1, 1, 1>::default();
        let mut a16 = ArmC53::<9, 1024, 2, 2, 2, 1>::default();
        let mut a17 = ArmC53::<9, 1024, 2, 2, 1, 1>::default();
        let mut a18 = ArmC53::<8, 512, 2, 2, 2, 1>::default();
        let mut a19 = ArmC53::<9, 1024, 1, 2, 2, 1>::default();
        let mut a20 = ArmC53::<9, 1024, 0, 2, 2, 1>::default();
        for &b in &blks {
            match arm_idx {
                2 => { a2.run_stats(&b.records, b.n, b.p, &mut out); }
                3 => { a3.run_stats(&b.records, b.n, b.p, &mut out); }
                4 => { a4.run_stats(&b.records, b.n, b.p, &mut out); }
                5 => { a5.run_stats(&b.records, b.n, b.p, &mut out); }
                6 => { a6.run_stats(&b.records, b.n, b.p, &mut out); }
                7 => { a7.run_stats(&b.records, b.n, b.p, &mut out); }
                8 => { a8.run_stats(&b.records, b.n, b.p, &mut out); }
                9 => { a9.run_stats(&b.records, b.n, b.p, &mut out); }
                10 => { a10.run_stats(&b.records, b.n, b.p, &mut out); }
                11 => { a11.run_stats(&b.records, b.n, b.p, &mut out); }
                12 => { a12.run_stats(&b.records, b.n, b.p, &mut out); }
                13 => { a13.run_stats(&b.records, b.n, b.p, &mut out); }
                14 => { a14.run_stats(&b.records, b.n, b.p, &mut out); }
                15 => { a15.run_stats(&b.records, b.n, b.p, &mut out); }
                16 => { a16.run_stats(&b.records, b.n, b.p, &mut out); }
                17 => { a17.run_stats(&b.records, b.n, b.p, &mut out); }
                18 => { a18.run_stats(&b.records, b.n, b.p, &mut out); }
                19 => { a19.run_stats(&b.records, b.n, b.p, &mut out); }
                _ => { a20.run_stats(&b.records, b.n, b.p, &mut out); }
            }
            checksum = checksum.wrapping_add(u64::from(out[0]) ^ u64::from(*out.last().unwrap()));
        }
    }
    println!("proxy_arm={arm_name} fresh={fresh} chk={checksum}");
}

fn run_exactness_and_ws(six: &[Block], wiki431: &[Block], non_wiki: &[Block], code_docs: &[Block], dense: &Block) {
    let mut kh = cand_r5_head::Kernel::default();
    let mut suite = ArmSuite::default();
    let mut out_ref = Vec::with_capacity(900_064);
    let mut out_k = Vec::with_capacity(900_064);

    // 1. 10,190 small valid BWTs
    let mut valid_cnt = 0usize;
    let mut text = Vec::with_capacity(512);
    for n in 1..=12usize {
        for mask in 0..(1usize << n) {
            text.clear();
            for bit in 0..n { text.push(b'a' + ((mask >> bit) & 1) as u8); }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            out_k.clear(); out_k.resize(n + 64, 0xAA);
            suite.c53_v4.open.try_run_const::<16>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
            assert_eq!(out_k, text, "C53 Open(bi-pipe1) binary mismatch n={n}");
            out_k.clear(); out_k.resize(n + 64, 0x55);
            suite.c53_v4.run_stats(&recs, n, p, &mut out_k);
            assert_eq!(out_k, text);
            valid_cnt += 1;
        }
    }
    let mut rng = 0xA076_1D64_78BD_642Fu64;
    for case in 0..2_000usize {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let n = 1 + ((rng >> 32) as usize % 512);
        let alpha = [2u16, 3, 4, 8, 16, 26, 256][case % 7];
        text.clear();
        for _ in 0..n {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            text.push(((rng >> 40) % alpha as u64) as u8);
        }
        let (l, p) = bwt_small(&text);
        let recs = records_of(&l);
        out_k.clear(); out_k.resize(n + 32, 0xAA);
        suite.c53_v4.open.try_run_const::<16>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, text);
        suite.abl_u16raw.open.try_run_const::<16>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, text);
        suite.c53_v4.open.try_run_const::<32>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(u32::MAX), &mut out_k).unwrap();
        assert_eq!(out_k, text);
        suite.c53_v4.run_stats(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        valid_cnt += 1;
    }

    // 2. 16M Guard + PackedEdgeRule32-Sentinel + Compact u16 `contents` Overflow Fallback!
    {
        let big = vec![0u16; 16_000_001];
        assert!(!suite.c53_v4.open.invert_tiny_fl(&big, 100, 0, &mut out_k));
        assert!(suite.c53_v4.open.try_run_const::<16>(&big, 100, 0, Cap::NONE, POST_S1_BI, &mut out_k).is_none());

        // Natural wide > 4095 synthetic block (m > 524_288 -> bits=20 -> sentinel=4095, num_codes > 4095!)
        // Exercises BOTH `PackedEdgeRule32-Sentinel` AND natural `U16Tag4` -> `u32` contents fallback!
        let mut l_wide = Vec::with_capacity(760_000);
        let mut sr = 0x5EED_C520_4095_BEEFu64;
        for g in 0..5_000usize {
            let sym = ((g % 240) + 1) as u8;
            l_wide.resize(l_wide.len() + 20, sym);
            for _ in 0..110 {
                sr = sr.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                l_wide.push(((sr >> 40) % 240 + 1) as u8);
            }
        }
        let nw_sz = l_wide.len();
        let rw = records_of(&l_wide);
        let want_w = plain_invert_fl(&l_wide, 42);
        let st_w = suite.c53_v4.open.try_run_const::<16>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, want_w, "natural wide >4095 bi-sentinel + u16-tag4 fallback mismatch!");
        assert!(st_w.m >= (1 << 19) && st_w.bits == 20 && st_w.wide && st_w.used_u32_fallback);

        // Forced u16_cap = 300 on BOTH `U16Tag4` and `U16Raw` (exact fallback test!)
        suite.c53_v4.open.force_u16_cap = Some(300);
        let st_u16t = suite.c53_v4.open.try_run_const::<16>(&six[0].records, six[0].n, six[0].p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert!(st_u16t.used_u32_fallback);
        assert_eq!(&out_k, six[0].expected.as_ref().unwrap(), "forced u16_cap=300 Tag4 fallback mismatch!");
        suite.c53_v4.open.force_u16_cap = None;

        suite.abl_u16raw.open.force_u16_cap = Some(300);
        let st_u16r = suite.abl_u16raw.open.try_run_const::<16>(&six[0].records, six[0].n, six[0].p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert!(st_u16r.used_u32_fallback);
        assert_eq!(&out_k, six[0].expected.as_ref().unwrap(), "forced u16_cap=300 Raw fallback mismatch!");
        suite.abl_u16raw.open.force_u16_cap = None;

        // Forced sentinel = 260 across bi+tail, bi+vec, uni
        suite.c53_v4.open.force_sentinel_cap = Some(260);
        let st_fb = suite.c53_v4.open.try_run_const::<16>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert!(st_fb.wide && st_fb.used_bi); assert_eq!(out_k, want_w);
        suite.c53_v4.open.force_band_vec = true;
        let st_fv = suite.c53_v4.open.try_run_const::<16>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert!(st_fv.wide && st_fv.used_bi && !st_fv.used_tail); assert_eq!(out_k, want_w);
        let st_fu = suite.c53_v4.open.try_run_const::<16>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(u32::MAX), &mut out_k).unwrap();
        assert!(st_fu.wide && !st_fu.used_bi); assert_eq!(out_k, want_w);
        suite.c53_v4.open.force_band_vec = false;
        suite.c53_v4.open.force_sentinel_cap = None;

        println!(
            "\nSENTINEL + U16-CONTENTS FALLBACK STRESS PASS: natural wide m={} bits={} num_codes={} u32_fb={} + forced u16_cap=300 (Tag4+Raw) + forced sentinel=260 100% BYTE-EXACT!",
            st_w.m, st_w.bits, st_w.num_codes, st_w.used_u32_fallback
        );
    }

    // 3. Stress + 6 Adm + 6 JSONL/Log + 4 Code/Docs + 80 OutTail Poison/Resize/Fallback
    let spiral = make_spiral_200k();
    let per2 = make_periodic("periodic-150k-d2", b"ab", 75_000);
    let per3 = make_periodic("periodic-105k-d3", b"abc", 35_000);
    let stress = [dense, &spiral, &per2, &per3];
    let mut ot_poison = 0usize;
    for b in six.iter().chain(stress.iter().copied()).chain(non_wiki.iter()).chain(code_docs.iter()) {
        let want = b.expected.as_ref().unwrap();
        for arm in 0..NUM_ARMS {
            suite.run_arm(arm, b, &mut out_k);
            assert_eq!(&out_k, want, "arm={} mismatch on {}", ARM_NAMES[arm], b.name);
        }
        let mut p_aa = vec![0xAAu8; b.n + 4096];
        suite.c53_v4.run_stats(&b.records, b.n, b.p, &mut p_aa); assert_eq!(&p_aa, want);
        let mut fresh = Vec::new();
        suite.c53_v4.run_stats(&b.records, b.n, b.p, &mut fresh); assert_eq!(&fresh, want);
        suite.c53_v4.open.force_band_vec = true;
        let st_fb = suite.c53_v4.open.try_run_const::<16>(&b.records, b.n, b.p, Cap::NONE, BiMode::AlwaysAt(65_536), &mut out_k).unwrap();
        assert!(!st_fb.used_tail); assert_eq!(&out_k, want);
        suite.c53_v4.open.force_band_vec = false;
        suite.c53_v4.force_fallback = true;
        assert_eq!(suite.c53_v4.run_stats(&b.records, b.n, b.p, &mut out_k).0, 0); assert_eq!(&out_k, want);
        suite.c53_v4.force_fallback = false;
        ot_poison += 4;
    }

    // 4. ALL 431 wikidata.bz2 blocks byte-exact across ALL arms
    for (idx, b) in wiki431.iter().enumerate() {
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        for arm in 2..NUM_ARMS {
            suite.run_arm(arm, b, &mut out_k);
            assert_eq!(out_k, out_ref, "wiki arm={} idx={idx}", ARM_NAMES[arm]);
        }
    }

    // 5. 350+ Malformed / Multi-Cycle Forward-FL Columns + Detached Pipe1 Cycles (d=1..65)
    let mut mal_cnt = 0usize;
    let mut check_mal = |l: &[u8], p: u32, tag: &str| {
        let n = l.len();
        let recs = records_of(l);
        kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        let mut pbuf = vec![0xAAu8; n + 64];
        suite.c53_v4.open.try_run_const::<16>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut pbuf).unwrap();
        assert_eq!(pbuf, out_ref, "malformed C53 bi-pipe1 mismatch {tag}");
        pbuf.clear(); pbuf.resize(n + 64, 0x55);
        suite.c53_v4.run_stats(&recs, n, p, &mut pbuf);
        assert_eq!(pbuf, out_ref, "malformed C53 entry mismatch {tag}");
        mal_cnt += 1;
    };
    check_mal(b"bbaabb", 0, "bbaabb");
    check_mal(b"abbaab", 3, "abbaab");
    for &per in &[253usize, 254, 255, 256, 257, 258, 512] {
        let n = per * 32; let mut l = vec![0u8; n]; l[0] = 2; l[n - 1] = 1;
        check_mal(&l, 0, &format!("sat255-{per}"));
    }
    for &d in &[1usize, 2, 3, 4, 5, 6, 7, 8, 15, 16, 17, 31, 32, 33, 64, 65] {
        let sz = 80_000usize; let mut lm = vec![0u8; sz];
        for i in 0..d { lm[i] = ((i + 1) % d) as u8; }
        for (i, slot) in lm.iter_mut().enumerate().skip(d) { *slot = (d + (i % 120)) as u8; }
        for slot in &mut lm[sz - 12_000..sz] { *slot = 250; }
        check_mal(&lm, 0, &format!("pipe1-detached-d{d}"));
    }
    for &sz in &[64usize, 65, 256, 257, 511, 512, 513, 1536, 1537, 65_535, 65_536, 65_537, 262_144, 340_000] {
        let mut l = Vec::with_capacity(sz);
        for i in 0..sz { l.push(((i * 7 + (i >> 5)) & (if sz >= 262_144 { 255 } else { 3 })) as u8); }
        check_mal(&l, (sz / 3) as u32, &format!("bnd-{sz}"));
    }
    let mut lbuf = Vec::with_capacity(70_000);
    for case in 0..312usize {
        let n = if case < 170 { 1 + (case * 37) % 512 } else if case < 230 { 8_192 } else { 40_000 + (case - 230) * 137 };
        let alpha = if case < 60 { 3usize } else if (230..255).contains(&case) { 2 } else { 2 + (case % 12) };
        lbuf.clear();
        if (230..255).contains(&case) {
            let half = n / 2; lbuf.resize(half / 2, b'b'); lbuf.resize(half, b'a');
            lbuf.resize(half + half / 2, b'a'); lbuf.resize(n, b'b');
        } else {
            for _ in 0..n { rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); lbuf.push(((rng >> 40) % alpha as u64) as u8); }
        }
        check_mal(&lbuf, ((rng >> 32) as usize % n) as u32, &format!("m-{case}"));
    }
    println!(
        "EXACTNESS PASS: {valid_cnt} small valid + {mal_cnt} malformed + {ot_poison} OutTail poison/fallback + 4 stress + {} Adm + {} Wiki + {} JSONL + {} Code/Docs 100% BYTE-EXACT!",
        six.len(), wiki431.len(), non_wiki.len(), code_docs.len()
    );

    // 6. Deterministic WS Attribution (`PeakCap`, `MaxUsed`, `OutQ` `tail_offset` + `spill`, `contents`, `edges`, `spelling`)
    let full_wiki: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();
    let all_wiki: Vec<&Block> = wiki431.iter().collect();
    let six_r: Vec<&Block> = six.iter().collect();
    let nw_r: Vec<&Block> = non_wiki.iter().collect();
    let cd_r: Vec<&Block> = code_docs.iter().collect();

    println!("\nDETERMINISTIC SCRATCH & OUTQ WORKING SET (`PeakCap`, `MaxUsed`, `OutQ_live`, `contents`, `edges`, `spelling`):");
    for (cname, blks) in [
        ("6 Admitted", &six_r), ("233 FullWiki", &full_wiki), ("431 ALL Wiki", &all_wiki),
        ("6 JSONL/Log", &nw_r), ("4 Code/Docs", &cd_r),
    ] {
        let mut kh_w = cand_r5_head::Kernel::default();
        let mut s_w = ArmSuite::default();
        let mut out_w = Vec::with_capacity(900_064);
        // Track for arms 2..10: (maxu, outq_tail_sum, outq_spill_sum, contents_sum, sp_sum, s5_sum, n)
        let mut stat = [(0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize); 8];
        for &b in blks {
            kh_w.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_w);
            let res = [
                s_w.c52_lean.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.c52_v3.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.abl_c256.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.abl_c1024.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.abl_u16raw.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.abl_u32dict.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.abl_nopipe1.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.c53_v4.run_stats(&b.records, b.n, b.p, &mut out_w).1,
            ];
            for (idx, r) in res.into_iter().enumerate() {
                if let Some(st) = r {
                    stat[idx].0 = stat[idx].0.max(st.used_bytes);
                    stat[idx].1 += st.outq_tail_bytes;
                    stat[idx].2 += st.outq_spill_used;
                    stat[idx].3 += st.contents_bytes;
                    stat[idx].4 += st.spelling_used;
                    stat[idx].5 += st.s5_live_bytes;
                    stat[idx].6 += 1;
                }
            }
        }
        let (h_cap, _, _) = kh_w.scratch_report();
        let reps = [
            (ARM_NAMES[2], s_w.c52_lean.open.scratch_report(), s_w.c52_lean.seg.scratch_bytes()),
            (ARM_NAMES[3], s_w.c52_v3.open.scratch_report(), s_w.c52_v3.seg.scratch_bytes()),
            (ARM_NAMES[4], s_w.abl_c256.open.scratch_report(), s_w.abl_c256.seg.scratch_bytes()),
            (ARM_NAMES[5], s_w.abl_c1024.open.scratch_report(), s_w.abl_c1024.seg.scratch_bytes()),
            (ARM_NAMES[6], s_w.abl_u16raw.open.scratch_report(), s_w.abl_u16raw.seg.scratch_bytes()),
            (ARM_NAMES[7], s_w.abl_u32dict.open.scratch_report(), s_w.abl_u32dict.seg.scratch_bytes()),
            (ARM_NAMES[8], s_w.abl_nopipe1.open.scratch_report(), s_w.abl_nopipe1.seg.scratch_bytes()),
            (ARM_NAMES[9], s_w.c53_v4.open.scratch_report(), s_w.c53_v4.seg.scratch_bytes()),
        ];
        println!("  [{cname}] (R5-Head PeakCap = {:.2}K)", h_cap as f64 / 1024.0);
        for (idx, (aname, (ocap, ec, sdesc), segb)) in reps.iter().enumerate() {
            let tot = ocap + segb;
            let n = stat[idx].6.max(1) as f64;
            println!(
                "    {:18} | PeakCap={:7.2}K | MaxUsed={:7.2}K | OutQ_live={:6.1}K (spl={:5.1}K) | S5={:6.1}K | dict={:3.1}K | sp={:5.2}K | ed={:7.2}K | {}",
                aname, tot as f64 / 1024.0, stat[idx].0 as f64 / 1024.0,
                stat[idx].1 as f64 / (n * 1024.0), stat[idx].2 as f64 / (n * 1024.0),
                stat[idx].5 as f64 / (n * 1024.0), stat[idx].3 as f64 / (n * 1024.0),
                stat[idx].4 as f64 / (n * 1024.0), *ec as f64 / 1024.0, sdesc
            );
        }
    }
}

struct Bench1TResult {
    tot_us: [f64; NUM_ARMS],
    oracle_us: f64,
    full_tot_us: [f64; NUM_ARMS],
    full_oracle_us: f64,
}

fn bench_1t_interleaved(label: &str, blocks: &[Block], rds: usize, show_rc: bool, show_per_block: bool) -> Bench1TResult {
    println!("\n=== HEADROOM ORACLE v8 (1T INTERLEAVED, {rds} rds/blk): {label} ===");
    let mut suite = ArmSuite::default();
    let mut out = Vec::with_capacity(900_064);
    for b in blocks.iter().take(6) { for arm in 0..NUM_ARMS { suite.run_arm(arm, b, &mut out); } }

    let (mut tot, mut full_tot, mut oracle_us, mut full_oracle_us) = ([0.0f64; NUM_ARMS], [0.0f64; NUM_ARMS], 0.0f64, 0.0f64);
    let mut by_rc = [[0.0f64; 4]; NUM_ARMS];
    let mut c53_paths = [0usize; 8];

    for b in blocks {
        let rc = (b.route_class as usize).min(3);
        let mut s: [Vec<f64>; NUM_ARMS] = Default::default();
        let mut p9 = 0u8;
        for r in 0..rds {
            for off in 0..NUM_ARMS {
                let arm = (r + off) % NUM_ARMS;
                let t0 = Instant::now();
                let path = suite.run_arm(arm, b, &mut out);
                s[arm].push(t0.elapsed().as_secs_f64() * 1e6);
                if arm == 9 { p9 = path; }
            }
        }
        c53_paths[p9 as usize] += 1;
        let mut meds = [0.0f64; NUM_ARMS];
        for arm in 0..NUM_ARMS {
            let m = pct(&mut s[arm], 1, 2);
            meds[arm] = m; tot[arm] += m; by_rc[arm][rc] += m;
            if b.route_class < 2 { full_tot[arm] += m; }
        }
        let best = meds[0].min(meds[2]).min(meds[3]).min(meds[4]).min(meds[5]).min(meds[6]).min(meds[7]).min(meds[8]).min(meds[9]);
        oracle_us += best;
        if b.route_class < 2 { full_oracle_us += best; }
        if show_per_block {
            println!(
                "  {:12} | H1={:6.1} | Lean={:.4}x | V3={:.4}x | C256={:.4}x | C1024={:.4}x | NoP1={:.4}x | V4(p{})={:6.1} ({:.4}x) | Orcl={:6.1}",
                b.name, meds[0], meds[2] / meds[0], meds[3] / meds[0], meds[4] / meds[0],
                meds[5] / meds[0], meds[8] / meds[0], p9, meds[9], meds[9] / meds[0], best
            );
        }
    }
    let base = tot[0];
    for arm in 0..NUM_ARMS {
        let lt = if arm >= 2 { if tot[arm] < base { "YES" } else { "NO" } } else { "-" };
        if show_rc {
            println!(
                "  {:18} | norm={:7.2}ms dense={:5.2}ms tiny={:4.2}ms col={:4.2}ms | TOTAL={:7.2}ms ({:.4}x) | < C_Open? {}",
                ARM_NAMES[arm], by_rc[arm][0] / 1e3, by_rc[arm][1] / 1e3, by_rc[arm][2] / 1e3, by_rc[arm][3] / 1e3,
                tot[arm] / 1e3, tot[arm] / base, lt
            );
        } else {
            println!("  {:18} | TOTAL={:8.1} us ({:.4}x) | < C_Open? {}", ARM_NAMES[arm], tot[arm], tot[arm] / base, lt);
        }
    }
    println!(
        "  {:18} | TOTAL={:8.1} us ({:.4}x) | C53-v4 capture of oracle headroom = {:.1}%",
        "Hindsight-Oracle", oracle_us, oracle_us / base,
        if base > oracle_us { (base - tot[9]) / (base - oracle_us) * 100.0 } else { 0.0 }
    );
    if show_rc {
        let fb = full_tot[0];
        println!(
            "  233-Full-Wiki 1T   | Head={:.2}ms | Lean={:.4}x | V3={:.4}x | C256={:.4}x | C1024={:.4}x | U16R={:.4}x | U32D={:.4}x | NoP1={:.4}x | V4={:.4}x | Oracle={:.4}x",
            fb / 1e3, full_tot[2] / fb, full_tot[3] / fb, full_tot[4] / fb, full_tot[5] / fb,
            full_tot[6] / fb, full_tot[7] / fb, full_tot[8] / fb, full_tot[9] / fb, full_oracle_us / fb
        );
    }
    println!("  C53-v4 routes [L16-bi, tiny, per, seg8, dense-bi, L16-nobi, L24, L32]: {:?}", c53_paths);
    Bench1TResult { tot_us: tot, oracle_us, full_tot_us: full_tot, full_oracle_us }
}

fn bench_mt_suite(label: &str, blocks: &[Block], passes_2w: usize, passes_4w: usize) -> ([f64; NUM_ARMS], [f64; NUM_ARMS]) {
    let (mut p50_2w, mut p50_4w) = ([0.0f64; NUM_ARMS], [0.0f64; NUM_ARMS]);
    for &(threads, passes) in &[(2usize, passes_2w), (4usize, passes_4w)] {
        println!("\n=== {} — {}W CONCURRENT BARRIER ({} interleaved passes) ===", label, threads, passes);
        let mut rmt: [Vec<f64>; NUM_ARMS] = Default::default();
        for pass in 0..passes {
            for off in 0..NUM_ARMS {
                let arm = (pass + off) % NUM_ARMS;
                let bar = Barrier::new(threads);
                let t = std::thread::scope(|sc| {
                    let bar_ref = &bar;
                    let mut hs = Vec::with_capacity(threads);
                    for tid in 0..threads {
                        hs.push(sc.spawn(move || {
                            let mut out = Vec::with_capacity(900_064);
                            let mut suite = ArmSuite::default();
                            for b in blocks.iter().take(6) { suite.run_arm(arm, b, &mut out); }
                            bar_ref.wait();
                            let t0 = Instant::now();
                            for idx in 0..blocks.len() { suite.run_arm(arm, &blocks[(idx + tid) % blocks.len()], &mut out); }
                            t0.elapsed().as_secs_f64() * 1e6
                        }));
                    }
                    hs.into_iter().map(|h| h.join().unwrap()).fold(0.0f64, f64::max)
                });
                rmt[arm].push(t);
            }
        }
        let b10 = pct(&mut rmt[0].clone(), 1, 10);
        let b50 = pct(&mut rmt[0].clone(), 1, 2);
        for arm in 0..NUM_ARMS {
            let p10 = pct(&mut rmt[arm], 1, 10);
            let p50 = pct(&mut rmt[arm], 1, 2);
            if threads == 2 { p50_2w[arm] = p50; } else { p50_4w[arm] = p50; }
            let lt = if arm >= 2 { if p50 < b50 { "YES" } else { "NO" } } else { "-" };
            println!("  {}W {:18} | p10={:9.1} us ({:.4}x) | p50={:9.1} us ({:.4}x) | p50 < C_Open? {}", threads, ARM_NAMES[arm], p10, p10 / b10, p50, p50 / b50, lt);
        }
    }
    (p50_2w, p50_4w)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--proxy-arm") {
        let arm_name = args.get(pos + 1).expect("missing arm name after --proxy-arm");
        let fresh = args.iter().any(|a| a == "--fresh");
        let wiki431 = load_bin_corpus(&resolve_corpus_path("wikidata_431_blocks.bin"), "wiki", false);
        run_proxy_arm(arm_name, fresh, &wiki431);
        return;
    }
    let verify_only = args.iter().any(|a| a == "--verify-only");
    let six = load_six_admitted();
    let wiki431 = load_bin_corpus(&resolve_corpus_path("wikidata_431_blocks.bin"), "wiki", false);
    let non_wiki = load_bin_corpus(&resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin"), "nonwiki", true);
    let code_docs = load_bin_corpus(&resolve_corpus_path("non_wikidata/code_docs_blocks.bin"), "codedoc", true);
    let dense = make_dense_300k();

    println!(
        "Cycle 53 Headroom Oracle v8 loaded: {} admitted, {} wikidata.bz2, {} JSONL/Log, {} Code/Docs blocks (verify_only={})",
        six.len(), wiki431.len(), non_wiki.len(), code_docs.len(), verify_only
    );

    // 1. Classical Atlas Extension `C53-D4` (Hölder 1889 / Jensen 1906 + Information Barrier)
    run_c53_d4_holder_jensen_audit(&six, &wiki431, &non_wiki, &code_docs, &dense);

    // 2. Exhaustive Exactness + Sentinel/U16 Fallback + Deterministic WS Attribution
    run_exactness_and_ws(&six, &wiki431, &non_wiki, &code_docs, &dense);

    if verify_only {
        println!("\nVERIFY-ONLY COMPLETE (100% PASS)!");
        return;
    }

    // 3. Interleaved 1T Headroom Oracle v8 across all 4 corpora
    let r1_six = bench_1t_interleaved("6 ADMITTED WIKIDATA BLOCKS", &six, 19, false, true);
    let r1_nw = bench_1t_interleaved("6 NON-WIKIDATA JSONL/LOG BLOCKS", &non_wiki, 19, false, true);
    let r1_cd = bench_1t_interleaved("4 NON-WIKIDATA CODE/DOCS BLOCKS", &code_docs, 19, false, true);
    let r1_w431 = bench_1t_interleaved("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 5, true, false);

    // 4. 2W and 4W Concurrent Barrier Benchmarks across all 4 corpora
    let (m2_six, m4_six) = bench_mt_suite("6 ADMITTED WIKIDATA BLOCKS", &six, 15, 11);
    let (m2_nw, m4_nw) = bench_mt_suite("6 NON-WIKIDATA JSONL/LOG BLOCKS", &non_wiki, 15, 11);
    let (m2_cd, m4_cd) = bench_mt_suite("4 NON-WIKIDATA CODE/DOCS BLOCKS", &code_docs, 15, 11);
    let (m2_w431, m4_w431) = bench_mt_suite("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 5, 5);

    // 5. Overall 441-Block Union & 243 Full-Block Union Summary
    println!("\n=== OVERALL SUMMARY: C_recognise + C_member < C_Open (ALL CORPORA SUMMED) ===");
    let sum1_441: [f64; NUM_ARMS] = std::array::from_fn(|a| r1_w431.tot_us[a] + r1_nw.tot_us[a] + r1_cd.tot_us[a]);
    let orcl1_441 = r1_w431.oracle_us + r1_nw.oracle_us + r1_cd.oracle_us;
    let sum1_243: [f64; NUM_ARMS] = std::array::from_fn(|a| r1_w431.full_tot_us[a] + r1_nw.tot_us[a] + r1_cd.tot_us[a]);
    let orcl1_243 = r1_w431.full_oracle_us + r1_nw.oracle_us + r1_cd.oracle_us;
    let sum2_441: [f64; NUM_ARMS] = std::array::from_fn(|a| m2_w431[a] + m2_nw[a] + m2_cd[a]);
    let sum4_441: [f64; NUM_ARMS] = std::array::from_fn(|a| m4_w431[a] + m4_nw[a] + m4_cd[a]);

    println!("  [441 Distinct Blocks = 431 Wiki + 6 JSONL/Log + 4 Code/Docs]");
    for arm in 0..NUM_ARMS {
        let lt1 = if sum1_441[arm] < sum1_441[0] { "YES" } else { "NO" };
        let lt2 = if sum2_441[arm] < sum2_441[0] { "YES" } else { "NO" };
        let lt4 = if sum4_441[arm] < sum4_41_base(sum4_41_base(sum4_441[0])) { "YES" } else { "NO" };
        println!(
            "    {:18} | 1T={:7.2}ms ({:.4}x, <Open? {:3}) | 2W={:7.2}ms ({:.4}x, <Open? {:3}) | 4W={:7.2}ms ({:.4}x, <Open? {:3})",
            ARM_NAMES[arm],
            sum1_441[arm] / 1e3, sum1_441[arm] / sum1_441[0], lt1,
            sum2_441[arm] / 1e3, sum2_441[arm] / sum2_441[0], lt2,
            sum4_441[arm] / 1e3, sum4_441[arm] / sum4_441[0], lt4,
        );
    }
    println!(
        "    {:18} | 1T={:7.2}ms ({:.4}x) | 6-Admitted 1T/2W/4W C53-SuperPipe-v4 = {:.4}x / {:.4}x / {:.4}x",
        "Hindsight-Oracle", orcl1_441 / 1e3, orcl1_441 / sum1_441[0],
        r1_six.tot_us[9] / r1_six.tot_us[0], m2_six[9] / m2_six[0], m4_six[9] / m4_six[0]
    );
    println!("  [243 Full-Size Blocks = 233 Full Wiki + 6 JSONL/Log + 4 Code/Docs, 1T]");
    for arm in 0..NUM_ARMS {
        println!(
            "    {:18} | 1T={:7.2}ms ({:.4}x) | < C_Open? {}",
            ARM_NAMES[arm], sum1_243[arm] / 1e3, sum1_243[arm] / sum1_243[0],
            if sum1_243[arm] < sum1_243[0] { "YES" } else { "NO" }
        );
    }
    println!("    {:18} | 1T={:7.2}ms ({:.4}x)", "Hindsight-Oracle", orcl1_243 / 1e3, orcl1_243 / sum1_243[0]);
}

#[inline(always)]
fn sum4_41_base(x: f64) -> f64 { x }
