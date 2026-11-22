#![forbid(unsafe_code)]

//! Cycle 51 P3 — Attacking the #1 Remaining Working-Set Bottleneck (`edges` = ~80% of WS):
//! Exact Per-Phase (`S2a`/`S2b` vs `S4`) Attribution, `CompactEdges3` (`12 B`/rule),
//! `CompactEdges-Var234` (`8/12/16 B`/rule), `ZeroHdr`, `OutScratch`, & Classical Tree-Bound Test.
//!
//! Strictly modern `2020s` CS systems engineering (`CS` — ZERO classical relabels!).
//! Also directly tests the classical Cayley (1859) / Kirchhoff (1847) leaf-internal tree
//! identity `#internal <= open_nodes - 1` on `S2b` grammar joins across all real blocks and
//! Fibonacci `F_25..F_29` (`CA-35`), verifying whether `S2b` obeys or violates it.
//!
//! Benchmarks 7 arms apples-to-apples (`1T` stage-by-stage + `1W/2W/4W` paired + A/A control):
//!   - Arm 0 (`C50-Deep-A1`): Cycle 50 `#729` DRAM/cache record (`CS-22`, `RULE=4`, `Vec<u32>` `spill.next`)
//!   - Arm 1 (`C50-Deep-A2`): Exact A/A control (identical to Arm 0)
//!   - Arm 2 (`CE3-Fixed12`): Strict `3 u32` (`12 B`)/rule for ALL rules (`4..=12` inline 3w,
//!                            `13..=254` `[l, r, memo]` 3w, `>=255` `[l, r, len/sat_idx]` 3w +
//!                            `sat_memo`), raw word-offset `id = top - base`, upfront `ensure_step_u8`,
//!                            compact `Vec<u16>` `spill.next`
//!   - Arm 3 (`CE-Var234`):   Variable `2/3/4`-word raw-offset `CompactEdges` (`4..=8` 2w=`8 B`,
//!                            `9..=12` 3w=`12 B`, `13..=15` 4w=`16 B`, `16..=254` 3w=`12 B`,
//!                            `>=255` 4w=`16 B`), upfront `ensure_step_u8`, `Vec<u16>` `spill.next`
//!   - Arm 4 (`CE-Var+ZeroH`): Arm 3 + `S2a` moves `strips_raw` & `image_order` into `out_tail` so
//!                            `Elim` starts at `self.edges[0]` (`hdr = 0`, `-4*ns` words!)
//!   - Arm 5 (`OutScr-Rules`): Arm 3 + `S2b` `CE-Var234` grammar rules borrowed inside `out_tail`
//!                            via 100% safe `as_chunks_mut::<4>()` + `from_ne_bytes`/`to_ne_bytes`
//!                            (seamless fallback to `self.edges`)
//!   - Arm 6 (`OutScr-Full`):  Arm 3 + BOTH `S2a` `nodes` & `S2b` `CE-Var234` `rules` borrowed
//!                            inside `out_tail` via safe `from_ne_bytes`/`to_ne_bytes`
//!                            (seamless fallback to `self.edges`)
//!
//! Strictly `#![forbid(unsafe_code)]`, `-D warnings`, forward-`FL` exact on malformed inputs.

use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE4: usize = 4;
const RULE3: usize = 3;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const SPELLING_STEP: usize = 4096;
const INLINE_STEP: usize = 128;

// F-Inline3 (`CS-06`): 8-bit grade (1..=255), 24-bit inline payload / rule id.
const ID_BITS: u32 = 24;
const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS; // 255

const ARM_C50_A1: u8 = 0;
const ARM_C50_A2: u8 = 1;
const ARM_CE3_FIXED12: u8 = 2;
const ARM_CE_VAR234: u8 = 3;
const ARM_CE_VAR_ZEROH: u8 = 4;
const ARM_OUTSCR_RULES: u8 = 5;
const ARM_OUTSCR_FULL: u8 = 6;
const ARM_CE34_FAST: u8 = 7;
const ARM_CE34_ZEROH: u8 = 8;

const NUM_ARMS: usize = 9;

const ARM_NAMES: [&str; NUM_ARMS] = [
    "C50-Deep-A1",
    "C50-Deep-A2",
    "CE3-Fixed12",
    "CE-Var234",
    "CE-Var+ZeroH",
    "OutScr-Rules",
    "OutScr-Full",
    "CE34-Fast",
    "CE34+ZeroH",
];

#[inline(always)]
const fn literal(byte: u8) -> u32 {
    (1u32 << ID_BITS) | (byte as u32)
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
        First {
            thick_rows: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            mark_byte: 0,
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
        inline16.extend((0..=255u8).map(|c| {
            let mut v = [0; INLINE];
            v[0] = c;
            v
        }));
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
    if need > v.capacity() {
        grow_step_u8(v, need, step);
    }
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
    if lens.len() == lens.capacity() {
        grow_step_inline(inline16, lens, step);
    }
}

#[cold]
#[inline(never)]
fn grow_edges_step(a: &mut Vec<u32>, need: usize) {
    let target = need.div_ceil(8192) * 8192;
    if target > a.capacity() {
        a.reserve_exact(target - a.len());
    }
    a.resize(target, 0);
}

fn spelling_hash_long(bytes: &[u8]) -> u64 {
    let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ part(bytes.len() - 8)
            .rotate_left(29)
            .wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

// ============================================================================
// OutQ16 Thin-Rank Queues (U32 vs Compact U16 `next`)
// ============================================================================

mod outq {
    const CHUNK: usize = 512;
    pub const BYTES: usize = 2 * CHUNK;
    const NONE32: u32 = u32::MAX;
    const NONE16: u16 = u16::MAX;
    const EPOCH: u32 = 1 << 16;

    #[derive(Default)]
    pub struct Spill {
        pub next32: Vec<u32>,
        pub next16: Vec<u16>,
        pub ovf: Vec<[u8; BYTES]>,
        pub cuts: Vec<[u32; 256]>,
    }

    pub struct Queues<'a, const U16_NEXT: bool> {
        out: &'a mut [u8],
        spill: &'a mut Spill,
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

    impl<'a, const U16_NEXT: bool> Queues<'a, U16_NEXT> {
        pub fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
            spill.ovf.clear();
            spill.cuts.clear();
            let in_out = (out.len() / BYTES) as u32;
            let out = &mut out[..in_out as usize * BYTES];
            Queues {
                in_out,
                out,
                spill,
                chunks: 0,
                heads: [NONE32; 256],
                pt: [u32::MAX as u64; 256],
                limit,
                over: false,
            }
        }

        pub fn over(&self) -> bool {
            self.over
        }

        pub fn heads(&self) -> Heads {
            Heads {
                heads: self.heads,
                in_out: self.in_out,
                chunks: self.chunks,
            }
        }

        #[inline(always)]
        pub fn thin_of(&self, a: usize) -> u32 {
            (self.pt[a] >> 32) as u32
        }

        pub fn extract_thin(&self) -> [u32; 256] {
            std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
        }

        #[inline(always)]
        pub fn push_thin(&mut self, a: usize, m: u32, w: u32) {
            let v = self.pt[a];
            let p = v as u32;
            let f = (p & 1023) as usize;
            if (w <= 16) & (f <= CHUNK - 16) & ((m as u16) < 65_520) {
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
            if self.over || m + w > self.limit {
                self.over = true;
                return;
            }
            let mut left = w;
            while left > 0 {
                let p = self.pt[a] as u32;
                if p == u32::MAX || (p & 1023) as usize == CHUNK {
                    if U16_NEXT && self.chunks >= u32::from(NONE16) {
                        self.over = true;
                        return;
                    }
                    self.take_chunk_pt(a);
                }
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & 1023) as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(p >> 10);
                for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                    .chunks_exact_mut(2)
                    .enumerate()
                {
                    d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                }
                let tu = u64::from(take);
                self.pt[a] = v + ((tu << 32) | tu);
                m += take;
                left -= take;
                if m % EPOCH == 0 {
                    let cut: [u32; 256] = std::array::from_fn(|b| (self.pt[b] >> 32) as u32);
                    if self.spill.cuts.len() == self.spill.cuts.capacity() {
                        self.spill.cuts.reserve_exact(1);
                    }
                    self.spill.cuts.push(cut);
                }
            }
        }

        fn take_chunk_pt(&mut self, a: usize) {
            let id = self.chunks;
            self.chunks += 1;
            if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                if self.spill.ovf.len() == self.spill.ovf.capacity() {
                    let target = (self.spill.ovf.len() + 1).div_ceil(16) * 16;
                    self.spill.ovf.reserve_exact(target - self.spill.ovf.len());
                }
                self.spill.ovf.push([0; BYTES]);
            }
            if U16_NEXT {
                if self.spill.next16.len() <= id as usize {
                    let target = (id as usize + 1).div_ceil(256) * 256;
                    if target > self.spill.next16.capacity() {
                        self.spill
                            .next16
                            .reserve_exact(target - self.spill.next16.len());
                    }
                    self.spill.next16.resize(target, NONE16);
                }
                self.spill.next16[id as usize] = NONE16;
                let hi = self.pt[a] & !0xFFFF_FFFFu64;
                if self.heads[a] == NONE32 {
                    self.heads[a] = id;
                } else {
                    self.spill.next16[((self.pt[a] as u32) >> 10) as usize] = id as u16;
                }
                self.pt[a] = hi | u64::from(id << 10);
            } else {
                if self.spill.next32.len() <= id as usize {
                    let target = (id as usize + 1).div_ceil(256) * 256;
                    if target > self.spill.next32.capacity() {
                        self.spill
                            .next32
                            .reserve_exact(target - self.spill.next32.len());
                    }
                    self.spill.next32.resize(target, NONE32);
                }
                self.spill.next32[id as usize] = NONE32;
                let hi = self.pt[a] & !0xFFFF_FFFFu64;
                if self.heads[a] == NONE32 {
                    self.heads[a] = id;
                } else {
                    self.spill.next32[((self.pt[a] as u32) >> 10) as usize] = id;
                }
                self.pt[a] = hi | u64::from(id << 10);
            }
        }

        #[cold]
        #[inline(never)]
        fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
            if id < self.in_out {
                let at = id as usize * BYTES;
                &mut self.out[at..at + BYTES]
            } else {
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            }
        }
    }

    pub struct Reader<'a, const U16_NEXT: bool> {
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

    impl<'a, const U16_NEXT: bool> Reader<'a, U16_NEXT> {
        #[inline(always)]
        pub fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
            Reader {
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
        fn next_chunk_id(&self, cur: u32) -> u32 {
            if U16_NEXT {
                u32::from(self.spill.next16[cur as usize])
            } else {
                self.spill.next32[cur as usize]
            }
        }

        #[inline(always)]
        pub fn wire_xor_nomark(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack: u32,
        ) {
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut i = 0usize;
                while i + 16 <= span {
                    let b = &bytes[2 * i..2 * i + 32];
                    let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                    let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                    let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                    let v0 = hi | (q0 as u32 & 0xFFFF);
                    let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q0 >> 48) as u32);
                    let v4 = hi | (q1 as u32 & 0xFFFF);
                    let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                    let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                    let v7 = hi | ((q1 >> 48) as u32);
                    let v8 = hi | (q2 as u32 & 0xFFFF);
                    let v9 = hi | ((q2 >> 16) as u32 & 0xFFFF);
                    let v10 = hi | ((q2 >> 32) as u32 & 0xFFFF);
                    let v11 = hi | ((q2 >> 48) as u32);
                    let v12 = hi | (q3 as u32 & 0xFFFF);
                    let v13 = hi | ((q3 >> 16) as u32 & 0xFFFF);
                    let v14 = hi | ((q3 >> 32) as u32 & 0xFFFF);
                    let v15 = hi | ((q3 >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 16];
                    eu[0] ^= pack | v0;   eu[1] ^= pack | v1;   eu[2] ^= pack | v2;   eu[3] ^= pack | v3;
                    eu[4] ^= pack | v4;   eu[5] ^= pack | v5;   eu[6] ^= pack | v6;   eu[7] ^= pack | v7;
                    eu[8] ^= pack | v8;   eu[9] ^= pack | v9;   eu[10] ^= pack | v10; eu[11] ^= pack | v11;
                    eu[12] ^= pack | v12; eu[13] ^= pack | v13; eu[14] ^= pack | v14; eu[15] ^= pack | v15;
                    edges[v0 as usize] ^= u0 as u32;        edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32;  edges[v3 as usize] ^= (u0 + 3) as u32;
                    edges[v4 as usize] ^= (u0 + 4) as u32;  edges[v5 as usize] ^= (u0 + 5) as u32;
                    edges[v6 as usize] ^= (u0 + 6) as u32;  edges[v7 as usize] ^= (u0 + 7) as u32;
                    edges[v8 as usize] ^= (u0 + 8) as u32;  edges[v9 as usize] ^= (u0 + 9) as u32;
                    edges[v10 as usize] ^= (u0 + 10) as u32; edges[v11 as usize] ^= (u0 + 11) as u32;
                    edges[v12 as usize] ^= (u0 + 12) as u32; edges[v13 as usize] ^= (u0 + 13) as u32;
                    edges[v14 as usize] ^= (u0 + 14) as u32; edges[v15 as usize] ^= (u0 + 15) as u32;
                    i += 16;
                }
                if i + 8 <= span {
                    let b = &bytes[2 * i..2 * i + 16];
                    let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                    let v0 = hi | (q0 as u32 & 0xFFFF);
                    let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q0 >> 48) as u32);
                    let v4 = hi | (q1 as u32 & 0xFFFF);
                    let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                    let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                    let v7 = hi | ((q1 >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 8];
                    eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                    eu[4] ^= pack | v4; eu[5] ^= pack | v5; eu[6] ^= pack | v6; eu[7] ^= pack | v7;
                    edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                    edges[v4 as usize] ^= (u0 + 4) as u32; edges[v5 as usize] ^= (u0 + 5) as u32;
                    edges[v6 as usize] ^= (u0 + 6) as u32; edges[v7 as usize] ^= (u0 + 7) as u32;
                    i += 8;
                }
                if i + 4 <= span {
                    let b = &bytes[2 * i..2 * i + 8];
                    let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let v0 = hi | (q as u32 & 0xFFFF);
                    let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 4];
                    eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                    edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                    i += 4;
                }
                while i < span {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                    i += 1;
                }
                self.at += span;
                self.read += span as u32;
            } else {
                let mut dummy = 0u32;
                self.wire_xor_slow(edges, dst_start, span, pack, u32::MAX, &mut dummy);
            }
        }

        #[inline(always)]
        pub fn wire_xor(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
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
                    self.id = self.next_chunk_id(self.id);
                    self.at = 0;
                }
                let take = left
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
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
        pub fn wire(&mut self, dst: &mut [u32], pack: u32) {
            let span = dst.len();
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
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
                    self.id = self.next_chunk_id(self.id);
                    self.at = 0;
                }
                let take = dst
                    .len()
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
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
}

// ============================================================================
// Safe Word Storage Abstraction: Native `&mut [u32]` vs Safe `OutScratch` `&mut [[u8; 4]]`
// ============================================================================

trait WordBuf {
    fn len_w(&self) -> usize;
    fn get_w(&self, i: usize) -> u32;
    fn set_w(&mut self, i: usize, v: u32);
    fn sub_assign_w(&mut self, i: usize, delta: u32) {
        let v = self.get_w(i).wrapping_sub(delta);
        self.set_w(i, v);
    }
}

impl WordBuf for [u32] {
    #[inline(always)]
    fn len_w(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn get_w(&self, i: usize) -> u32 {
        self[i]
    }
    #[inline(always)]
    fn set_w(&mut self, i: usize, v: u32) {
        self[i] = v;
    }
    #[inline(always)]
    fn sub_assign_w(&mut self, i: usize, delta: u32) {
        self[i] -= delta;
    }
}

impl WordBuf for [[u8; 4]] {
    #[inline(always)]
    fn len_w(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn get_w(&self, i: usize) -> u32 {
        u32::from_ne_bytes(self[i])
    }
    #[inline(always)]
    fn set_w(&mut self, i: usize, v: u32) {
        self[i] = v.to_ne_bytes();
    }
}

// ============================================================================
// Links (Narrow 4-word / Wide 6-word) over any `WordBuf`
// ============================================================================

trait Links {
    const STRIDE: usize;
    const IS_NARROW: bool;
    const BOTTOM: usize;
    fn prev<W: WordBuf + ?Sized>(a: &W, id: u32, side: usize) -> u32;
    fn next<W: WordBuf + ?Sized>(a: &W, id: u32, side: usize) -> u32;
    fn set_prev<W: WordBuf + ?Sized>(a: &mut W, id: u32, side: usize, to: u32);
    fn set_next<W: WordBuf + ?Sized>(a: &mut W, id: u32, side: usize, to: u32);
    fn band_node<W: WordBuf + ?Sized>(a: &W, id: u32) -> (u32, u32, u32, u32);
}

struct Narrow;
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
    const IS_NARROW: bool = true;
    const BOTTOM: usize = 1;
    #[inline(always)]
    fn prev<W: WordBuf + ?Sized>(a: &W, id: u32, side: usize) -> u32 {
        a.get_w(id as usize * 4 + side) & 0xFFFF
    }
    #[inline(always)]
    fn next<W: WordBuf + ?Sized>(a: &W, id: u32, side: usize) -> u32 {
        a.get_w(id as usize * 4 + side) >> 16
    }
    #[inline(always)]
    fn set_prev<W: WordBuf + ?Sized>(a: &mut W, id: u32, side: usize, to: u32) {
        let idx = id as usize * 4 + side;
        let w = a.get_w(idx);
        a.set_w(idx, (w & 0xFFFF_0000) | to);
    }
    #[inline(always)]
    fn set_next<W: WordBuf + ?Sized>(a: &mut W, id: u32, side: usize, to: u32) {
        let idx = id as usize * 4 + side;
        let w = a.get_w(idx);
        a.set_w(idx, (w & 0xFFFF) | (to << 16));
    }
    #[inline(always)]
    fn band_node<W: WordBuf + ?Sized>(a: &W, id: u32) -> (u32, u32, u32, u32) {
        let base = id as usize * 4;
        (
            a.get_w(base) >> 16,
            a.get_w(base + 1),
            a.get_w(base + 2),
            a.get_w(base + 3),
        )
    }
}

impl Links for Wide {
    const STRIDE: usize = 6;
    const IS_NARROW: bool = false;
    const BOTTOM: usize = 2;
    #[inline(always)]
    fn prev<W: WordBuf + ?Sized>(a: &W, id: u32, side: usize) -> u32 {
        a.get_w(id as usize * 6 + 2 * side)
    }
    #[inline(always)]
    fn next<W: WordBuf + ?Sized>(a: &W, id: u32, side: usize) -> u32 {
        a.get_w(id as usize * 6 + 2 * side + 1)
    }
    #[inline(always)]
    fn set_prev<W: WordBuf + ?Sized>(a: &mut W, id: u32, side: usize, to: u32) {
        a.set_w(id as usize * 6 + 2 * side, to);
    }
    #[inline(always)]
    fn set_next<W: WordBuf + ?Sized>(a: &mut W, id: u32, side: usize, to: u32) {
        a.set_w(id as usize * 6 + 2 * side + 1, to);
    }
    #[inline(always)]
    fn band_node<W: WordBuf + ?Sized>(a: &W, id: u32) -> (u32, u32, u32, u32) {
        let base = id as usize * 6;
        (
            a.get_w(base + 1),
            a.get_w(base + 2),
            a.get_w(base + 4),
            a.get_w(base + 5),
        )
    }
}

// Rule encoding mode:
//   0 = C50 Fixed-4 (`RULE4 = 4`, `id = (top - base) / 4`)
//   1 = CE3 Fixed-3 (`RULE3 = 3`, `id = top - base`, `INLINE_ARENA = 12`, `>=255` via `sat_memo`)
//   2 = CE-Var234 (`id = top - base`, `4..=8` 2w, `9..=12` 3w, `13..=15` 4w, `16..=254` 3w, `>=255` 4w)
const RMODE_C50_4: u8 = 0;
const RMODE_CE3_FIXED: u8 = 1;
const RMODE_CE_VAR234: u8 = 2;
const RMODE_CE34_FAST: u8 = 3;

#[derive(Default, Clone, Copy)]
struct RuleDiag {
    s2b_steps: usize,
    total_joins: usize,
    r_1_3: usize,
    r_4_8: usize,
    r_9_12: usize,
    r_13_15: usize,
    r_16_254: usize,
    r_255p: usize,
}

impl RuleDiag {
    fn num_alloc(&self) -> usize {
        self.r_4_8 + self.r_9_12 + self.r_13_15 + self.r_16_254 + self.r_255p
    }
    fn words_c50(&self) -> usize {
        self.num_alloc() * 4
    }
    fn words_ce3(&self) -> usize {
        self.num_alloc() * 3
    }
    fn words_var234(&self) -> usize {
        self.r_4_8 * 2 + (self.r_9_12 + self.r_16_254) * 3 + (self.r_13_15 + self.r_255p) * 4
    }
}

// ============================================================================
// Unified `Elim` over Nodes buffer `N` and Rules buffer `R` (can be same or split!)
// ============================================================================

struct ElimSplit<'n, 'r, L: Links, N: WordBuf + ?Sized, R: WordBuf + ?Sized, const RMODE: u8, const DIAG: bool> {
    nodes_buf: &'n mut N,
    rules_buf: &'r mut R,
    rule_top: usize,
    nodes: u32,
    id_overflow: bool,
    diag: RuleDiag,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links, N: WordBuf + ?Sized, R: WordBuf + ?Sized, const RMODE: u8, const DIAG: bool>
    ElimSplit<'_, '_, L, N, R, RMODE, DIAG>
{
    #[inline(always)]
    fn width(&self, id: u32) -> u32 {
        self.nodes_buf.get_w(id as usize * L::STRIDE + L::STRIDE - 2)
    }
    #[inline(always)]
    fn set_width(&mut self, id: u32, w: u32) {
        self.nodes_buf.set_w(id as usize * L::STRIDE + L::STRIDE - 2, w);
    }
    #[inline(always)]
    fn word(&self, id: u32) -> u32 {
        self.nodes_buf.get_w(id as usize * L::STRIDE + L::STRIDE - 1)
    }
    #[inline(always)]
    fn set_word(&mut self, id: u32, w: u32) {
        self.nodes_buf.set_w(id as usize * L::STRIDE + L::STRIDE - 1, w);
    }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, x: u32) {
        self.nodes_buf.set_w(id as usize * L::STRIDE + L::BOTTOM, x);
    }

    fn assign_bottom(&mut self, rows: u32) {
        let mut first = L::next(self.nodes_buf, 0, 1);
        let mut last = L::prev(self.nodes_buf, 0, 1);
        let (mut lo, mut hi) = (0, rows);
        while first != last {
            let next = L::next(self.nodes_buf, first, 1);
            let prev = L::prev(self.nodes_buf, last, 1);
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
            L::set_next(self.nodes_buf, *tail, side, id);
        }
        L::set_prev(self.nodes_buf, id, side, *tail);
        *tail = id;
    }

    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, second] {
            if tail == 0 {
                continue;
            }
            L::set_next(self.nodes_buf, prev, side, head);
            L::set_prev(self.nodes_buf, head, side, prev);
            prev = tail;
        }
        L::set_next(self.nodes_buf, prev, side, 0);
        L::set_prev(self.nodes_buf, 0, side, prev);
    }

    #[inline(always)]
    fn cut(&mut self, id: u32, side: usize) {
        let (p, n) = (L::prev(self.nodes_buf, id, side), L::next(self.nodes_buf, id, side));
        L::set_next(self.nodes_buf, p, side, n);
        L::set_prev(self.nodes_buf, n, side, p);
    }

    #[inline(always)]
    fn pop_tail(&mut self, id: u32, side: usize) {
        let prev = L::prev(self.nodes_buf, id, side);
        L::set_next(self.nodes_buf, prev, side, 0);
        L::set_prev(self.nodes_buf, 0, side, prev);
    }

    #[inline(always)]
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.nodes_buf, at, side);
        L::set_prev(self.nodes_buf, id, side, at);
        L::set_next(self.nodes_buf, id, side, next);
        L::set_next(self.nodes_buf, at, side, id);
        L::set_prev(self.nodes_buf, next, side, id);
    }

    #[inline(always)]
    fn rule_idx(&self, id: u32) -> usize {
        if RMODE == RMODE_C50_4 {
            id as usize * RULE4
        } else {
            id as usize
        }
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            let r = self.rule_idx(h & ID_MASK);
            self.rules_buf.get_w(r + 2)
        }
    }

    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 3 {
            return u128::from(id);
        }
        let r = self.rule_idx(id);
        if RMODE == RMODE_C50_4 {
            u128::from(self.rules_buf.get_w(r))
                | (u128::from(self.rules_buf.get_w(r + 1)) << 32)
                | (u128::from(self.rules_buf.get_w(r + 2)) << 64)
                | (u128::from(self.rules_buf.get_w(r + 3) & 0x00ff_ffff) << 96)
        } else if RMODE == RMODE_CE3_FIXED {
            // For g in 4..=12, stored inline in 3 words; for 13..=15, binary node!
            if g <= 12 {
                u128::from(self.rules_buf.get_w(r))
                    | (u128::from(self.rules_buf.get_w(r + 1)) << 32)
                    | (u128::from(self.rules_buf.get_w(r + 2)) << 64)
            } else {
                self.expand_short_13_15(h)
            }
        } else if RMODE == RMODE_CE34_FAST {
            // RMODE_CE34_FAST: 4..=12 => 3w (90.3% of short rules!), 13..=15 => 4w
            let lo = u128::from(self.rules_buf.get_w(r))
                | (u128::from(self.rules_buf.get_w(r + 1)) << 32)
                | (u128::from(self.rules_buf.get_w(r + 2)) << 64);
            if g <= 12 {
                lo
            } else {
                lo | (u128::from(self.rules_buf.get_w(r + 3)) << 96)
            }
        } else {
            // RMODE_CE_VAR234: 4..=8 => 2w, 9..=12 => 3w, 13..=15 => 4w
            let lo = u128::from(self.rules_buf.get_w(r))
                | (u128::from(self.rules_buf.get_w(r + 1)) << 32);
            if g <= 8 {
                lo
            } else if g <= 12 {
                lo | (u128::from(self.rules_buf.get_w(r + 2)) << 64)
            } else {
                lo | (u128::from(self.rules_buf.get_w(r + 2)) << 64)
                    | (u128::from(self.rules_buf.get_w(r + 3)) << 96)
            }
        }
    }

    #[cold]
    fn expand_short_13_15(&self, h: u32) -> u128 {
        let mut stk = [0u32; 16];
        let mut sp = 0usize;
        let mut cur = h;
        let mut out = 0u128;
        let mut shift = 0u32;
        loop {
            let g = cur >> ID_BITS;
            let id = cur & ID_MASK;
            if g <= 12 {
                let v = if g <= 3 {
                    u128::from(id)
                } else {
                    let r = id as usize;
                    u128::from(self.rules_buf.get_w(r))
                        | (u128::from(self.rules_buf.get_w(r + 1)) << 32)
                        | (u128::from(self.rules_buf.get_w(r + 2)) << 64)
                };
                out |= v << shift;
                shift += 8 * g;
                if sp == 0 {
                    return out;
                }
                sp -= 1;
                cur = stk[sp];
            } else {
                let r = id as usize;
                stk[sp] = self.rules_buf.get_w(r + 1);
                sp += 1;
                cur = self.rules_buf.get_w(r);
            }
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
        let left_len = left >> ID_BITS;
        let sum = left_len + (right >> ID_BITS);
        if DIAG {
            self.diag.total_joins += 1;
            if sum <= 3 {
                self.diag.r_1_3 += 1;
            } else if sum <= 8 {
                self.diag.r_4_8 += 1;
            } else if sum <= 12 {
                self.diag.r_9_12 += 1;
            } else if sum <= 15 {
                self.diag.r_13_15 += 1;
            } else if sum < SATURATED {
                self.diag.r_16_254 += 1;
            } else {
                self.diag.r_255p += 1;
            }
        }
        if sum <= 3 {
            return (sum << ID_BITS) | (left & ID_MASK) | ((right & ID_MASK) << (8 * left_len));
        }

        if RMODE == RMODE_C50_4 {
            let id = (self.rule_top / RULE4) as u32;
            let t = self.rule_top;
            if sum < INLINE as u32 {
                let bytes = self.short_payload_h(left)
                    | (self.short_payload_h(right) << (8 * left_len))
                    | (u128::from(sum) << 120);
                self.rules_buf.set_w(t, bytes as u32);
                self.rules_buf.set_w(t + 1, (bytes >> 32) as u32);
                self.rules_buf.set_w(t + 2, (bytes >> 64) as u32);
                self.rules_buf.set_w(t + 3, (bytes >> 96) as u32);
                self.rule_top = t + RULE4;
                (sum << ID_BITS) | id
            } else {
                let len = if sum <= SATURATED {
                    sum
                } else {
                    self.len_handle(left) + self.len_handle(right)
                };
                self.rules_buf.set_w(t, left);
                self.rules_buf.set_w(t + 1, right);
                self.rules_buf.set_w(t + 2, len);
                self.rules_buf.set_w(t + 3, u32::MAX);
                self.rule_top = t + RULE4;
                (len.min(SATURATED) << ID_BITS) | id
            }
        } else if RMODE == RMODE_CE3_FIXED {
            let t = self.rule_top;
            let id = t as u32;
            if sum <= 12 {
                let bytes = self.short_payload_h(left)
                    | (self.short_payload_h(right) << (8 * left_len));
                self.rules_buf.set_w(t, bytes as u32);
                self.rules_buf.set_w(t + 1, (bytes >> 32) as u32);
                self.rules_buf.set_w(t + 2, (bytes >> 64) as u32);
                self.rule_top = t + RULE3;
                (sum << ID_BITS) | id
            } else if sum < SATURATED {
                self.rules_buf.set_w(t, left);
                self.rules_buf.set_w(t + 1, right);
                self.rules_buf.set_w(t + 2, u32::MAX);
                self.rule_top = t + RULE3;
                (sum << ID_BITS) | id
            } else {
                let len = if sum == SATURATED {
                    sum
                } else {
                    self.len_handle(left) + self.len_handle(right)
                };
                self.rules_buf.set_w(t, left);
                self.rules_buf.set_w(t + 1, right);
                self.rules_buf.set_w(t + 2, len); // bit 31 = 0 => unmemoized exact len!
                self.rule_top = t + RULE3;
                (SATURATED << ID_BITS) | id
            }
        } else {
            // RMODE_CE_VAR234 / RMODE_CE34_FAST:
            //   CE34_FAST: 4..=12 => 3w, 13..=15 => 4w, 16..=254 => 3w, >=255 => 4w
            //   CE_VAR234: 4..=8 => 2w, 9..=12 => 3w, 13..=15 => 4w, 16..=254 => 3w, >=255 => 4w
            let t = self.rule_top;
            let id = t as u32;
            if sum < INLINE as u32 {
                let bytes = self.short_payload_h(left)
                    | (self.short_payload_h(right) << (8 * left_len));
                self.rules_buf.set_w(t, bytes as u32);
                self.rules_buf.set_w(t + 1, (bytes >> 32) as u32);
                if RMODE == RMODE_CE34_FAST {
                    self.rules_buf.set_w(t + 2, (bytes >> 64) as u32);
                    if sum <= 12 {
                        self.rule_top = t + 3;
                    } else {
                        self.rules_buf.set_w(t + 3, (bytes >> 96) as u32);
                        self.rule_top = t + 4;
                    }
                } else if sum <= 8 {
                    self.rule_top = t + 2;
                } else if sum <= 12 {
                    self.rules_buf.set_w(t + 2, (bytes >> 64) as u32);
                    self.rule_top = t + 3;
                } else {
                    self.rules_buf.set_w(t + 2, (bytes >> 64) as u32);
                    self.rules_buf.set_w(t + 3, (bytes >> 96) as u32);
                    self.rule_top = t + 4;
                }
                (sum << ID_BITS) | id
            } else if sum < SATURATED {
                self.rules_buf.set_w(t, left);
                self.rules_buf.set_w(t + 1, right);
                self.rules_buf.set_w(t + 2, u32::MAX);
                self.rule_top = t + 3;
                (sum << ID_BITS) | id
            } else {
                let len = if sum == SATURATED {
                    sum
                } else {
                    self.len_handle(left) + self.len_handle(right)
                };
                self.rules_buf.set_w(t, left);
                self.rules_buf.set_w(t + 1, right);
                self.rules_buf.set_w(t + 2, len);
                self.rules_buf.set_w(t + 3, u32::MAX);
                self.rule_top = t + 4;
                (SATURATED << ID_BITS) | id
            }
        }
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

    #[inline(always)]
    fn max_rule_words() -> usize {
        if RMODE == RMODE_CE3_FIXED {
            RULE3
        } else {
            RULE4
        }
    }

    #[inline(never)]
    fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let step_room = Self::max_rule_words();
        let max_id_top = if RMODE == RMODE_C50_4 {
            usize::MAX
        } else {
            ID_MASK as usize + 1
        };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.rule_top + step_room > self.rules_buf.len_w() {
                break false;
            }
            if self.rule_top + step_room > max_id_top {
                self.id_overflow = true;
                break false;
            }
            if DIAG {
                self.diag.s2b_steps += 1;
            }
            let x = self.nodes_buf.get_w(0) & 0xFFFF;
            let y = self.nodes_buf.get_w(1) & 0xFFFF;
            let xi = x as usize * 4;
            let yi = y as usize * 4;
            let (x0, x1, aw, wx) = (
                self.nodes_buf.get_w(xi),
                self.nodes_buf.get_w(xi + 1),
                self.nodes_buf.get_w(xi + 2),
                self.nodes_buf.get_w(xi + 3),
            );
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                let v0 = self.nodes_buf.get_w(px0 as usize * 4) & 0xFFFF;
                self.nodes_buf.set_w(px0 as usize * 4, v0);
                let s0 = (self.nodes_buf.get_w(0) & 0xFFFF_0000) | px0;
                self.nodes_buf.set_w(0, s0);
                let v1 = self.nodes_buf.get_w(px1 as usize * 4 + 1) & 0xFFFF;
                self.nodes_buf.set_w(px1 as usize * 4 + 1, v1);
                let s1 = (self.nodes_buf.get_w(1) & 0xFFFF_0000) | px1;
                self.nodes_buf.set_w(1, s1);
                continue;
            }
            let (y0, y1, bw, wy) = (
                self.nodes_buf.get_w(yi),
                self.nodes_buf.get_w(yi + 1),
                self.nodes_buf.get_w(yi + 2),
                self.nodes_buf.get_w(yi + 3),
            );
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            let word = self.join(wx, wy);
            if aw <= bw {
                let px = x0 & 0xFFFF;
                let vpx = self.nodes_buf.get_w(px as usize * 4) & 0xFFFF;
                self.nodes_buf.set_w(px as usize * 4, vpx);
                let s0 = (self.nodes_buf.get_w(0) & 0xFFFF_0000) | px;
                self.nodes_buf.set_w(0, s0);
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    self.nodes_buf.set_w(xi, y | (y_next << 16));
                    self.nodes_buf.set_w(xi + 3, word);
                    self.nodes_buf.set_w(yi, (y0 & 0xFFFF) | (x << 16));
                    self.nodes_buf.set_w(yi + 2, bw - aw);
                    let yni = y_next as usize * 4;
                    let v = (self.nodes_buf.get_w(yni) & 0xFFFF_0000) | x;
                    self.nodes_buf.set_w(yni, v);
                } else {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    self.nodes_buf.set_w(xi, y_prev | (y_next << 16));
                    self.nodes_buf.set_w(xi + 3, word);
                    let ypi = y_prev as usize * 4;
                    let vp = (self.nodes_buf.get_w(ypi) & 0xFFFF) | (x << 16);
                    self.nodes_buf.set_w(ypi, vp);
                    let yni = y_next as usize * 4;
                    let vn = (self.nodes_buf.get_w(yni) & 0xFFFF_0000) | x;
                    self.nodes_buf.set_w(yni, vn);
                    let py = y1 & 0xFFFF;
                    let vpy = self.nodes_buf.get_w(py as usize * 4 + 1) & 0xFFFF;
                    self.nodes_buf.set_w(py as usize * 4 + 1, vpy);
                    let s1 = (self.nodes_buf.get_w(1) & 0xFFFF_0000) | py;
                    self.nodes_buf.set_w(1, s1);
                }
            } else {
                let py = y1 & 0xFFFF;
                let vpy = self.nodes_buf.get_w(py as usize * 4 + 1) & 0xFFFF;
                self.nodes_buf.set_w(py as usize * 4 + 1, vpy);
                let s1 = (self.nodes_buf.get_w(1) & 0xFFFF_0000) | py;
                self.nodes_buf.set_w(1, s1);
                let x_next = if x == py { 0 } else { x1 >> 16 };
                self.nodes_buf.set_w(yi + 1, x | (x_next << 16));
                self.nodes_buf.set_w(yi + 3, word);
                self.nodes_buf.set_w(xi + 1, (x1 & 0xFFFF) | (y << 16));
                self.nodes_buf.set_w(xi + 2, aw - bw);
                let xni = x_next as usize * 4 + 1;
                let vn = (self.nodes_buf.get_w(xni) & 0xFFFF_0000) | y;
                self.nodes_buf.set_w(xni, vn);
            }
        };
        *remaining = left;
        done
    }

    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
        if !SPIRALS && L::IS_NARROW {
            return self.eliminate_narrow_plain(remaining);
        }
        let mut left = *remaining;
        let step_room = Self::max_rule_words();
        let room = if SPIRALS {
            SPIRAL_RULES * step_room
        } else {
            step_room
        };
        let max_id_top = if RMODE == RMODE_C50_4 {
            usize::MAX
        } else {
            ID_MASK as usize + 1
        };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.rule_top + room > self.rules_buf.len_w() {
                break false;
            }
            if self.rule_top + room > max_id_top {
                self.id_overflow = true;
                break false;
            }
            if DIAG {
                self.diag.s2b_steps += 1;
            }
            let (x, y) = (L::prev(self.nodes_buf, 0, 0), L::prev(self.nodes_buf, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if SPIRALS && x != y {
                if aw > bw && L::prev(self.nodes_buf, y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.nodes_buf, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
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
        *remaining = left;
        done
    }

    fn inline_word(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
        sat_memo: &mut Vec<(u32, u32)>,
    ) -> ([u8; INLINE], u8) {
        let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
            (v.to_le_bytes(), (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            if RMODE != RMODE_C50_4 {
                let total = self.len_handle_s3a(word, sat_memo) as usize;
                ensure_step_u8(spelling, total + 1, SPELLING_STEP);
            }
            let mut dummy_rec = [0u32; 32];
            let mut dummy_n = 0usize;
            let mut dummy_ovf = false;
            self.append_tail_unw::<false>(
                word,
                stack,
                spelling,
                sat_memo,
                &mut dummy_rec,
                &mut dummy_n,
                &mut dummy_ovf,
            );
            if RMODE == RMODE_C50_4 {
                ensure_step_u8(spelling, 1, SPELLING_STEP);
            }
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }

    #[inline(always)]
    fn len_handle_s3a(&self, h: u32, sat_memo: &[(u32, u32)]) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else if RMODE == RMODE_CE3_FIXED {
            let tag = self.rules_buf.get_w(self.rule_idx(h & ID_MASK) + 2);
            if (tag & 0x8000_0000) == 0 {
                tag
            } else {
                sat_memo[(tag & 0x7FFF_FFFF) as usize].1
            }
        } else {
            self.rules_buf.get_w(self.rule_idx(h & ID_MASK) + 2)
        }
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn append_tail_unw<const TRACK_MEMO: bool>(
        &mut self,
        handle: u32,
        stack: &mut Vec<u32>,
        dest: &mut Vec<u8>,
        sat_memo: &mut Vec<(u32, u32)>,
        memo_rec: &mut [u32; 32],
        memo_n: &mut usize,
        memo_ovf: &mut bool,
    ) {
        let mut local = [0u32; 16];
        let mut sp = 0usize;
        let mut used_heap = false;
        let mut h = handle;
        loop {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                if RMODE == RMODE_C50_4 {
                    ensure_step_u8(dest, g as usize, SPELLING_STEP);
                }
                dest.extend_from_slice(&bytes[..g as usize]);
                if used_heap && !stack.is_empty() {
                    h = stack.pop().unwrap();
                    continue;
                }
                if sp > 0 {
                    sp -= 1;
                    h = local[sp];
                    continue;
                }
                return;
            }
            let r = self.rule_idx(h & ID_MASK);
            if RMODE == RMODE_C50_4 {
                let tag = self.rules_buf.get_w(r + 3);
                if tag != u32::MAX {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = self.rules_buf.get_w(r + 2) as usize;
                    ensure_step_u8(dest, rlen, SPELLING_STEP);
                    dest.extend_from_within(start..start + rlen);
                    if used_heap && !stack.is_empty() {
                        h = stack.pop().unwrap();
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = local[sp];
                        continue;
                    }
                    return;
                }
                self.rules_buf.set_w(r + 3, 0x8000_0000 | (dest.len() as u32));
                if TRACK_MEMO {
                    if *memo_n < 32 {
                        memo_rec[*memo_n] = (r + 3) as u32;
                        *memo_n += 1;
                    } else {
                        *memo_ovf = true;
                    }
                }
            } else if RMODE == RMODE_CE3_FIXED {
                let tag = self.rules_buf.get_w(r + 2);
                if g < SATURATED {
                    if tag != u32::MAX {
                        let start = (tag & 0x7fff_ffff) as usize;
                        dest.extend_from_within(start..start + g as usize);
                        if used_heap && !stack.is_empty() {
                            h = stack.pop().unwrap();
                            continue;
                        }
                        if sp > 0 {
                            sp -= 1;
                            h = local[sp];
                            continue;
                        }
                        return;
                    }
                    self.rules_buf.set_w(r + 2, 0x8000_0000 | (dest.len() as u32));
                    if TRACK_MEMO {
                        if *memo_n < 32 {
                            memo_rec[*memo_n] = (r + 2) as u32;
                            *memo_n += 1;
                        } else {
                            *memo_ovf = true;
                        }
                    }
                } else {
                    if (tag & 0x8000_0000) != 0 {
                        let (start, rlen) = sat_memo[(tag & 0x7fff_ffff) as usize];
                        let s = start as usize;
                        dest.extend_from_within(s..s + rlen as usize);
                        if used_heap && !stack.is_empty() {
                            h = stack.pop().unwrap();
                            continue;
                        }
                        if sp > 0 {
                            sp -= 1;
                            h = local[sp];
                            continue;
                        }
                        return;
                    }
                    let idx = sat_memo.len() as u32;
                    sat_memo.push((dest.len() as u32, tag));
                    self.rules_buf.set_w(r + 2, 0x8000_0000 | idx);
                    if TRACK_MEMO {
                        if *memo_n < 32 {
                            memo_rec[*memo_n] = 0x8000_0000 | idx;
                            *memo_n += 1;
                        } else {
                            *memo_ovf = true;
                        }
                    }
                }
            } else {
                // RMODE_CE_VAR234
                let m_off = if g < SATURATED { r + 2 } else { r + 3 };
                let tag = self.rules_buf.get_w(m_off);
                if tag != u32::MAX {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = if g < SATURATED {
                        g as usize
                    } else {
                        self.rules_buf.get_w(r + 2) as usize
                    };
                    dest.extend_from_within(start..start + rlen);
                    if used_heap && !stack.is_empty() {
                        h = stack.pop().unwrap();
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = local[sp];
                        continue;
                    }
                    return;
                }
                self.rules_buf.set_w(m_off, 0x8000_0000 | (dest.len() as u32));
                if TRACK_MEMO {
                    if *memo_n < 32 {
                        memo_rec[*memo_n] = m_off as u32;
                        *memo_n += 1;
                    } else {
                        *memo_ovf = true;
                    }
                }
            }

            let left = self.rules_buf.get_w(r);
            let right = self.rules_buf.get_w(r + 1);
            let lg = left >> ID_BITS;
            if lg < INLINE as u32 {
                let lb = self.short_payload_h(left).to_le_bytes();
                if RMODE == RMODE_C50_4 {
                    ensure_step_u8(dest, lg as usize, SPELLING_STEP);
                }
                dest.extend_from_slice(&lb[..lg as usize]);
                let rg = right >> ID_BITS;
                if rg < INLINE as u32 {
                    let rb = self.short_payload_h(right).to_le_bytes();
                    if RMODE == RMODE_C50_4 {
                        ensure_step_u8(dest, rg as usize, SPELLING_STEP);
                    }
                    dest.extend_from_slice(&rb[..rg as usize]);
                    if used_heap && !stack.is_empty() {
                        h = stack.pop().unwrap();
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = local[sp];
                        continue;
                    }
                    return;
                }
                h = right;
            } else {
                if sp < 16 {
                    local[sp] = right;
                    sp += 1;
                } else {
                    if !used_heap {
                        stack.clear();
                        used_heap = true;
                    }
                    stack.push(right);
                }
                h = left;
            }
        }
    }
}

struct Dictionary<'a> {
    pairs: &'a mut [u32; 512],
    contents: &'a mut [u32; 2048],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
    sat_memo: &'a mut Vec<(u32, u32)>,
    memo_ovf_hits: usize,
}

impl Dictionary<'_> {
    #[inline(always)]
    fn code<L: Links, N: WordBuf + ?Sized, R: WordBuf + ?Sized, const RMODE: u8, const DIAG: bool>(
        &mut self,
        e: &mut ElimSplit<L, N, R, RMODE, DIAG>,
        handle: u32,
        byte: u8,
    ) -> u32 {
        let bucket = ((handle.wrapping_mul(0x9e37_79b9)
            ^ (u32::from(byte).wrapping_mul(0x85eb_ca6b)))
            >> 24) as usize;
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

    fn code_miss<
        L: Links,
        N: WordBuf + ?Sized,
        R: WordBuf + ?Sized,
        const RMODE: u8,
        const DIAG: bool,
    >(
        &mut self,
        e: &mut ElimSplit<L, N, R, RMODE, DIAG>,
        handle: u32,
        id: u32,
        byte: u8,
    ) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
            let entry = v.to_le_bytes();
            let len = (height + 1) as u8;
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let h = h ^ (h >> 30);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == len && self.inline16[old] == entry
            } {
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry, len);
            self.contents[index] = tag | c;
            return c;
        }

        let start = self.spelling.len();
        let g = handle >> ID_BITS;
        let r = e.rule_idx(id);

        // Check fast-path already-memoized root handle
        let already: Option<(usize, usize)> = if RMODE == RMODE_C50_4 {
            let tag_r = e.rules_buf.get_w(r + 3);
            if tag_r != u32::MAX {
                Some(((tag_r & 0x7fff_ffff) as usize, e.rules_buf.get_w(r + 2) as usize))
            } else {
                None
            }
        } else if RMODE == RMODE_CE3_FIXED {
            let tag_r = e.rules_buf.get_w(r + 2);
            if g < SATURATED {
                if tag_r != u32::MAX {
                    Some(((tag_r & 0x7fff_ffff) as usize, g as usize))
                } else {
                    None
                }
            } else if (tag_r & 0x8000_0000) != 0 {
                let (p0, rl) = self.sat_memo[(tag_r & 0x7fff_ffff) as usize];
                Some((p0 as usize, rl as usize))
            } else {
                None
            }
        } else {
            let m_off = if g < SATURATED { r + 2 } else { r + 3 };
            let tag_r = e.rules_buf.get_w(m_off);
            if tag_r != u32::MAX {
                let rlen = if g < SATURATED {
                    g as usize
                } else {
                    e.rules_buf.get_w(r + 2) as usize
                };
                Some(((tag_r & 0x7fff_ffff) as usize, rlen))
            } else {
                None
            }
        };

        if let Some((pos, rlen)) = already {
            ensure_step_u8(self.spelling, rlen + 1, SPELLING_STEP);
            self.spelling.extend_from_within(pos..pos + rlen);
            self.spelling.push(byte);
            let entry = long_slot(start as u32, self.spelling.len() as u32);
            let bytes = &self.spelling[start..];
            let h = spelling_hash_long(bytes);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == LONG
                    && long_word(&self.inline16[old], self.spelling) == bytes
            } {
                self.spelling.truncate(start);
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry, LONG);
            self.contents[index] = tag | c;
            return c;
        }

        // Unmemoized handle: upfront reserve (for non-C50) + K-TailUnw + OrphanFree delta-repoint
        if RMODE != RMODE_C50_4 {
            let total = e.len_handle_s3a(handle, self.sat_memo) as usize;
            ensure_step_u8(self.spelling, total + 1, SPELLING_STEP);
        }
        let mut memo_rec = [0u32; 32];
        let mut memo_n = 0usize;
        let mut memo_ovf = false;
        e.append_tail_unw::<true>(
            handle,
            self.stack,
            self.spelling,
            self.sat_memo,
            &mut memo_rec,
            &mut memo_n,
            &mut memo_ovf,
        );
        if RMODE == RMODE_C50_4 {
            ensure_step_u8(self.spelling, 1, SPELLING_STEP);
        }
        self.spelling.push(byte);
        let entry = long_slot(start as u32, self.spelling.len() as u32);
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = (h as usize) & 2047;
        let ent = self.contents[index];
        if (ent & 0xFF00_0000) == tag && {
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes
        } {
            if !memo_ovf {
                let old = (ent & 0x00FF_FFFF) as usize;
                let old_start =
                    u32::from_le_bytes(self.inline16[old][..4].try_into().unwrap());
                let delta = (start as u32) - old_start;
                for &rm in &memo_rec[..memo_n] {
                    if RMODE == RMODE_CE3_FIXED && (rm & 0x8000_0000) != 0 {
                        self.sat_memo[(rm & 0x7FFF_FFFF) as usize].0 -= delta;
                    } else {
                        e.rules_buf.sub_assign_w(rm as usize, delta);
                    }
                }
                self.spelling.truncate(start);
            } else {
                self.memo_ovf_hits += 1;
                self.spelling.pop();
            }
            return ent & 0x00FF_FFFF;
        }
        let c = self.insert(entry, LONG);
        self.contents[index] = tag | c;
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

// ============================================================================
// Pass (S1) & Telemetry
// ============================================================================

struct Pass<'a, const U16_NEXT: bool> {
    queues: outq::Queues<'a, U16_NEXT>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    f: First,
}

impl<const U16_NEXT: bool> Pass<'_, U16_NEXT> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        if width <= LIMIT {
            self.piece(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece(c, pos, width, false, p);
        } else {
            self.take_marked(c, pos, width, p);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        let end = pos + width;
        if pos < p {
            self.piece(c, pos, p - pos, false, p);
        }
        self.piece(c, p, 1, true, p);
        if p + 1 < end {
            self.piece(c, p + 1, end - p - 1, false, p);
        }
    }

    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
                f.mark_byte = c;
            }
            self.queues.push_thin(a, f.m, w);
            f.m += w;
        } else {
            let at = 3 * self.ns;
            if self.edges.len() < at + 3 {
                let need = (at + 3).max(2 * self.edges.len()).max(1024);
                self.edges.reserve_exact(need - self.edges.len());
                self.edges.resize(need, 0);
            }
            let thin_a = self.queues.thin_of(a);
            self.edges[at..at + 3].copy_from_slice(&[
                x,
                thin_a + f.thick_rows[a],
                (w << 8) | u32::from(c),
            ]);
            self.ns += 1;
            f.thick[a] += 1;
            f.thick_rows[a] += w;
        }
    }
}

#[derive(Default, Clone, Copy)]
struct StageTimes {
    s1_us: f64,
    s2a_us: f64,
    s2b_us: f64,
    s3a_us: f64,
    s4_us: f64,
    s5_us: f64,
}

impl StageTimes {
    fn ind_us(&self) -> f64 {
        self.s2a_us + self.s2b_us + self.s3a_us
    }
}

#[derive(Default, Clone, Copy)]
struct BlockTelemetry {
    is_tiny: bool,
    m: usize,
    ns: usize,
    e_nodes: usize,
    most_nodes: usize,
    open_nodes: usize,
    hdr_words: usize,
    base_words: usize,
    rule_words: usize,
    edges_s2b_peak_words: usize,
    edges_peak_used_words: usize,
    used_tail_bands: bool,
    used_out_rules: bool,
    used_out_nodes: bool,
    used_zero_hdr: bool,
    spelling_len: usize,
    ncodes_stored: usize,
    n_len16: usize,
    sat_memo_len: usize,
    memo_ovf_hits: usize,
    spill_chunks: usize,
    spill_cuts: usize,
    spill_ovf_chunks: usize,
    bands_fb_len: usize,
    extra_len: usize,
    out_tail_avail: usize,
    nbands: usize,
    touched_s2b_edges_bytes: usize,
    touched_total_bytes: usize,
    rdiag: RuleDiag,
}

#[derive(Default)]
struct C51Kernel {
    spill: outq::Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    sat_memo: Vec<(u32, u32)>,
    predecessor: Option<u32>,
    // Test / forced-fallback knobs
    force_wide: bool,
    force_no_tail: bool,
    force_spirals: Option<bool>,
    force_no_outscr: bool,
}

impl C51Kernel {
    fn ws_breakdown(&self, u16_next: bool) -> (usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize) {
        let e = self.edges.capacity() * 4;
        let sp = self.spelling.capacity();
        let i16 = self.inline16.capacity() * INLINE;
        let ln = self.lens.capacity();
        let sn = if u16_next {
            self.spill.next16.capacity() * 2
        } else {
            self.spill.next32.capacity() * 4
        };
        let sc = self.spill.cuts.capacity() * 1024;
        let so = self.spill.ovf.capacity() * outq::BYTES;
        let bd = self.bands.capacity() * 8;
        let ex = self.extra.capacity() * 4;
        let st = self.stack.capacity() * 4;
        let sm = self.sat_memo.capacity() * 8;
        let tot = e + sp + i16 + ln + sn + sc + so + bd + ex + st + sm;
        (tot, e, sp, i16, ln, sn, sc, so, bd, ex, st, sm)
    }

    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    #[inline(never)]
    fn invert_tiny_fl(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> bool {
        if n == 0 || (p as usize) >= n || records.len() > 16_000_000 {
            return false;
        }
        if records.len() == 1 {
            if usize::from(records[0] & 255) != n {
                return false;
            }
            let b = (records[0] >> 8) as u8;
            out.clear();
            out.resize(n, b);
            return true;
        }
        let mut counts = [0u32; 256];
        for &rec in records {
            counts[(rec >> 8) as usize] += u32::from(rec & 255);
        }
        let mut st = [0u32; 256];
        let mut tot = 0u32;
        for c in 0..256 {
            st[c] = tot;
            tot += counts[c];
        }
        if tot as usize != n {
            return false;
        }
        if self.edges.len() < n {
            let add = n - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(n, 0);
        }
        let fl = &mut self.edges[..n];
        let mut pos = 0usize;
        for &rec in records {
            let b = (rec >> 8) as usize;
            let len = (rec & 255) as usize;
            let s0 = st[b] as usize;
            st[b] += len as u32;
            let mut i = pos as u32;
            let bu = b as u32;
            for slot in &mut fl[s0..s0 + len] {
                *slot = (i << 8) | bu;
                i += 1;
            }
            pos += len;
        }
        out.resize(n, 0);
        let mut x = p as usize;
        for slot in &mut out[..n] {
            let v = fl[x];
            *slot = v as u8;
            x = (v >> 8) as usize;
        }
        true
    }

    fn run_arm<const ARM: u8, const TIMED: bool, const DIAG: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> (bool, StageTimes, BlockTelemetry) {
        let mut tm = StageTimes::default();
        let mut tel = BlockTelemetry::default();
        if n == 0 || (p as usize) >= n {
            return (false, tm, tel);
        }
        if n < 32_768 && !self.force_wide && !self.force_no_tail && self.force_spirals.is_none() {
            let ok = self.invert_tiny_fl(records, n, p, out);
            tel.is_tiny = true;
            tel.edges_peak_used_words = n;
            return (ok, tm, tel);
        }
        let nrec = records.len();
        if nrec > n {
            return (false, tm, tel);
        }
        let u16_next = ARM >= ARM_CE3_FIXED12;
        if u16_next {
            self.run_open::<ARM, true, TIMED, DIAG>(records, n, p, out, &mut tm, &mut tel)
        } else {
            self.run_open::<ARM, false, TIMED, DIAG>(records, n, p, out, &mut tm, &mut tel)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run_open<const ARM: u8, const U16_NEXT: bool, const TIMED: bool, const DIAG: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        tm: &mut StageTimes,
        tel: &mut BlockTelemetry,
    ) -> (bool, StageTimes, BlockTelemetry) {
        let nrec = records.len();
        let t0 = if TIMED { Some(Instant::now()) } else { None };
        out.resize(n + INLINE, 0);
        let (ns, f, heads, over, tot_pos) = {
            let mut pass = Pass::<U16_NEXT> {
                queues: outq::Queues::new(&mut out[..], &mut self.spill, u32::MAX),
                edges: &mut self.edges,
                ns: 0,
                f: First::default(),
            };
            let mut pos = 0u32;
            let mut i = 0usize;
            while i < nrec {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                if width <= LIMIT {
                    pass.piece(c, pos, width, true, p);
                    pos += width;
                } else {
                    if width == 127 {
                        while i < nrec && (records[i] >> 8) as u8 == c {
                            width += u32::from(records[i] & 255);
                            i += 1;
                        }
                    }
                    pass.take(c, pos, width, p);
                    pos += width;
                }
            }
            let over = pass.queues.over();
            let thin = pass.queues.extract_thin();
            let heads = pass.queues.heads();
            let mut f = pass.f;
            f.thin = thin;
            (pass.ns, f, heads, over, pos)
        };
        if let Some(ts) = t0 {
            tm.s1_us = ts.elapsed().as_secs_f64() * 1e6;
        }
        if over || tot_pos as usize != n || f.m == 0 {
            return (false, *tm, *tel);
        }
        let ok = self.finish::<ARM, U16_NEXT, TIMED, DIAG>(ns, &f, n, &heads, out, tm, tel);
        (ok, *tm, *tel)
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn finish<const ARM: u8, const U16_NEXT: bool, const TIMED: bool, const DIAG: bool>(
        &mut self,
        ns: usize,
        f: &First,
        n: usize,
        heads: &outq::Heads,
        out: &mut Vec<u8>,
        tm: &mut StageTimes,
        tel: &mut BlockTelemetry,
    ) -> bool {
        let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        if all.iter().sum::<u32>() != n as u32 {
            return false;
        }
        let first = starts(all);
        let mut write = starts(f.thick);
        let m = f.m as usize;
        let most = Self::nodes_most(ns);
        let use_narrow = !self.force_wide && most <= NARROW_NODES;
        let stride = if use_narrow {
            Narrow::STRIDE
        } else {
            Wide::STRIDE
        };

        self.bands.clear();
        self.spelling.clear();
        self.sat_memo.clear();

        let in_out_chunks = out.len() / outq::BYTES;
        let tail_offset = (heads.chunks as usize).min(in_out_chunks) * outq::BYTES;
        let out_tail_len = out.len() - tail_offset;

        // Pre-S2a sizing depending on arm
        let need = if ARM == ARM_CE_VAR_ZEROH || ARM == ARM_CE34_ZEROH {
            if !self.force_no_outscr && out_tail_len >= 16 * ns {
                (stride * most).max(m)
            } else {
                (4 * ns + stride * most).max(m)
            }
        } else if ARM == ARM_OUTSCR_FULL {
            // Only need m for S4 (plus 3*ns already written in S1)
            m.max(3 * ns)
        } else {
            (4 * ns + stride * most).max(m)
        };
        if self.edges.len() < need {
            let add = need - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(need, 0);
        }

        let rmode: u8 = match ARM {
            ARM_C50_A1 | ARM_C50_A2 => RMODE_C50_4,
            ARM_CE3_FIXED12 => RMODE_CE3_FIXED,
            ARM_CE34_FAST | ARM_CE34_ZEROH => RMODE_CE34_FAST,
            _ => RMODE_CE_VAR234,
        };

        let res = match (use_narrow, rmode) {
            (true, RMODE_C50_4) => self.induced_dispatch::<ARM, Narrow, RMODE_C50_4, TIMED, DIAG>(
                ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..], tm, tel,
            ),
            (false, RMODE_C50_4) => self.induced_dispatch::<ARM, Wide, RMODE_C50_4, TIMED, DIAG>(
                ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..], tm, tel,
            ),
            (true, RMODE_CE3_FIXED) => {
                self.induced_dispatch::<ARM, Narrow, RMODE_CE3_FIXED, TIMED, DIAG>(
                    ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..], tm, tel,
                )
            }
            (false, RMODE_CE3_FIXED) => {
                self.induced_dispatch::<ARM, Wide, RMODE_CE3_FIXED, TIMED, DIAG>(
                    ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..], tm, tel,
                )
            }
            (true, RMODE_CE34_FAST) => {
                self.induced_dispatch::<ARM, Narrow, RMODE_CE34_FAST, TIMED, DIAG>(
                    ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..], tm, tel,
                )
            }
            (false, RMODE_CE34_FAST) => {
                self.induced_dispatch::<ARM, Wide, RMODE_CE34_FAST, TIMED, DIAG>(
                    ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..], tm, tel,
                )
            }
            (true, _) => self.induced_dispatch::<ARM, Narrow, RMODE_CE_VAR234, TIMED, DIAG>(
                ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..], tm, tel,
            ),
            (false, _) => self.induced_dispatch::<ARM, Wide, RMODE_CE_VAR234, TIMED, DIAG>(
                ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..], tm, tel,
            ),
        };
        let (ok_ind, cursors, used_tail, nbands) = res;
        if !ok_ind {
            return false;
        }

        // S4: wire
        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m {
            let add = m - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(m, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m, 0);
        }
        self.predecessor = None;
        let use_xor = !wide && f.m >= 65_536;
        if use_xor {
            self.edges[..m].fill(0);
        }
        let mut predecessor = f.mark;
        let mb = usize::from(f.mark_byte);
        let (out_q, out_tail) = out.split_at(tail_offset);

        if used_tail {
            self.wire_from_outtail64::<U16_NEXT>(
                &cursors, nbands, out_tail, heads, out_q, m, bits, wide, use_xor, f.mark, mb,
                &mut predecessor,
            );
        } else {
            self.wire_from_band64::<U16_NEXT>(
                &cursors, heads, out_q, m, bits, wide, use_xor, f.mark, mb, &mut predecessor,
            );
        }
        if use_xor {
            self.predecessor = Some(predecessor);
        }
        if let Some(ts) = t_s4 {
            tm.s4_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        // S5: emit
        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
        if let Some(before) = self.predecessor {
            emit_bidirectional(
                &self.edges[..m],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                before,
                bits,
                out,
            );
        } else {
            emit(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                bits,
                out,
            );
        }
        if let Some(ts) = t_s5 {
            tm.s5_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        if !TIMED {
            tel.m = m;
            tel.ns = ns;
            tel.most_nodes = most;
            tel.used_tail_bands = used_tail;
            tel.nbands = nbands;
            tel.out_tail_avail = out_tail_len;
            tel.spelling_len = self.spelling.len();
            tel.ncodes_stored = self.lens.len();
            tel.n_len16 = self.lens.iter().filter(|&&l| l == 16).count();
            tel.sat_memo_len = self.sat_memo.len();
            tel.spill_chunks = heads.chunks as usize;
            tel.spill_cuts = self.spill.cuts.len();
            tel.spill_ovf_chunks = self.spill.ovf.len();
            tel.bands_fb_len = self.bands.len();
            tel.extra_len = self.extra.len();
            tel.edges_peak_used_words = tel.edges_s2b_peak_words.max(m);
            let sn_bytes = if U16_NEXT {
                heads.chunks as usize * 2
            } else {
                heads.chunks as usize * 4
            };
            tel.touched_total_bytes = n
                + tel.edges_peak_used_words * 4
                + tel.spelling_len
                + tel.ncodes_stored * (INLINE + 1)
                + sn_bytes
                + tel.spill_cuts * 1024
                + tel.spill_ovf_chunks * outq::BYTES
                + tel.bands_fb_len * 8
                + tel.extra_len * 4
                + tel.sat_memo_len * 8;
        }
        true
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn induced_dispatch<
        const ARM: u8,
        L: Links,
        const RMODE: u8,
        const TIMED: bool,
        const DIAG: bool,
    >(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
        tel: &mut BlockTelemetry,
    ) -> (bool, [usize; 256], bool, usize) {
        if ARM == ARM_OUTSCR_FULL && !self.force_no_outscr {
            return self.induced_outscr_full::<L, RMODE, TIMED, DIAG>(
                ns, n, m, thin, first, write, most, out_tail, tm, tel,
            );
        }
        if ARM == ARM_OUTSCR_RULES && !self.force_no_outscr {
            return self.induced_outscr_rules::<L, RMODE, TIMED, DIAG>(
                ns, n, m, thin, first, write, most, out_tail, tm, tel,
            );
        }
        if (ARM == ARM_CE_VAR_ZEROH || ARM == ARM_CE34_ZEROH)
            && !self.force_no_outscr
            && out_tail.len() >= 16 * ns
        {
            return self.induced_zero_hdr::<L, RMODE, TIMED, DIAG>(
                ns, n, m, thin, first, write, most, out_tail, tm, tel,
            );
        }
        self.induced_edges_native::<L, RMODE, TIMED, DIAG>(
            ns, n, m, thin, first, write, most, out_tail, tm, tel,
        )
    }

    // ------------------------------------------------------------------------
    // Mode 1: Standard `self.edges` native `[u32]` (`C50-Deep`, `CE3-Fixed12`, `CE-Var234`)
    // ------------------------------------------------------------------------
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn induced_edges_native<
        L: Links,
        const RMODE: u8,
        const TIMED: bool,
        const DIAG: bool,
    >(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
        tel: &mut BlockTelemetry,
    ) -> (bool, [usize; 256], bool, usize) {
        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        let nodes_words = L::STRIDE * most;
        if self.edges.len() < 4 * ns + nodes_words {
            grow_edges_step(&mut self.edges, 4 * ns + nodes_words);
        }

        let mut widest = 0u32;
        let (base, nodes, open_nodes) = {
            let (hdr, arena) = self.edges.split_at_mut(4 * ns);
            let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
            let (nodes_sl, rules_dummy) = arena.split_at_mut(nodes_words);
            let mut e = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
                nodes_buf: nodes_sl,
                rules_buf: rules_dummy,
                rule_top: 0,
                nodes: 0,
                id_overflow: false,
                diag: RuleDiag::default(),
                _links: std::marker::PhantomData,
            };
            e.node(0, 0);
            e.nodes = ns as u32 + 1;
            let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
            let mut x = 0u32;
            for i in 0..ns {
                let sw = &mut strips_raw[3 * i..3 * i + 3];
                let from = sw[0];
                let wbyte = sw[2];
                let byte = (wbyte & 0xFF) as u8;
                let width = wbyte >> 8;
                let a = usize::from(byte);
                sw[1] += first[a];
                let dst_idx = write[a] as usize;
                image_order[dst_idx] = i as u32;
                write[a] = dst_idx as u32 + 1;
                let sid = i as u32 + 1;
                e.set_width(sid, width);
                e.set_word(sid, literal(byte));
                widest = widest.max(width);
                if x < from {
                    let gw = from - x;
                    widest = widest.max(gw);
                    let gid = e.node(gw, 0);
                    e.append_to(&mut order[0], gid, 0);
                    e.append_to(&mut terminals[1], gid, 1);
                }
                e.append_to(&mut order[0], sid, 0);
                x = from + width;
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
                let sw = &strips_raw[3 * idx..3 * idx + 3];
                let to = sw[1];
                let width = sw[2] >> 8;
                if x < to {
                    widest = widest.max(to - x);
                    let id = e.node(to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = to + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            let open_nodes = e.nodes as usize - 1;
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            let base = L::STRIDE * e.nodes as usize;
            (base, e.nodes, open_nodes)
        };
        if let Some(ts) = t_s2a {
            tm.s2a_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let spirals = self.force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
        let mut remaining = n;
        let mut rule_top = 0usize;
        let mut diag = RuleDiag::default();
        loop {
            let (nodes_sl, rules_sl) = self.edges[4 * ns..].split_at_mut(base);
            let mut e = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
                nodes_buf: nodes_sl,
                rules_buf: rules_sl,
                rule_top,
                nodes,
                id_overflow: false,
                diag,
                _links: std::marker::PhantomData,
            };
            let done = if spirals {
                e.eliminate::<true>(&mut remaining)
            } else {
                e.eliminate::<false>(&mut remaining)
            };
            rule_top = e.rule_top;
            diag = e.diag;
            if e.id_overflow {
                return (false, [0; 256], false, 0);
            }
            if done {
                break;
            }
            grow_edges_step(&mut self.edges, 4 * ns + base + rule_top + SPIRAL_RULES * RULE4);
        }
        if let Some(ts) = t_s2b {
            tm.s2b_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail_arena) = arena.split_at_mut(base + rule_top);
        let (nodes_sl, rules_sl) = live.split_at_mut(base);
        let mut e = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
            nodes_buf: nodes_sl,
            rules_buf: rules_sl,
            rule_top,
            nodes,
            id_overflow: false,
            diag,
            _links: std::marker::PhantomData,
        };
        e.assign_bottom(m);

        const PAIR_WORDS: usize = 512;
        const CACHE_WORDS: usize = PAIR_WORDS + 2048;
        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail_arena
        };

        let (cursors, use_tail, nbands, memo_ovf_hits) = run_s3a_bands(
            &mut e,
            cache,
            thin,
            open_nodes,
            self.force_no_tail,
            out_tail,
            &mut self.bands,
            &mut self.inline16,
            &mut self.lens,
            &mut self.spelling,
            &mut self.stack,
            &mut self.sat_memo,
        );
        if let Some(ts) = t_s3a {
            tm.s3a_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        if !TIMED {
            tel.e_nodes = nodes as usize;
            tel.open_nodes = open_nodes;
            tel.hdr_words = 4 * ns;
            tel.base_words = base;
            tel.rule_words = rule_top;
            tel.edges_s2b_peak_words = 4 * ns + base + rule_top;
            tel.touched_s2b_edges_bytes = (4 * ns + base + rule_top) * 4;
            tel.memo_ovf_hits = memo_ovf_hits;
            tel.rdiag = diag;
        }
        (true, cursors, use_tail, nbands)
    }

    // ------------------------------------------------------------------------
    // Mode 2: `CE-Var+ZeroH` (`strips_raw` & `image_order` copied to `out_tail`, `hdr = 0`!)
    // ------------------------------------------------------------------------
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn induced_zero_hdr<
        L: Links,
        const RMODE: u8,
        const TIMED: bool,
        const DIAG: bool,
    >(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
        tel: &mut BlockTelemetry,
    ) -> (bool, [usize; 256], bool, usize) {
        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        let nodes_words = L::STRIDE * most;
        if self.edges.len() < nodes_words {
            grow_edges_step(&mut self.edges, nodes_words);
        }

        // Copy `strips_raw` (3*ns words) to `out_tail[0..12*ns]` and use `out_tail[12*ns..16*ns]`
        // for `image_order`, freeing `self.edges[0..]` completely for `Elim` nodes + rules!
        let mut widest = 0u32;
        let (base, nodes, open_nodes) = {
            let (ot_words, _) = out_tail.as_chunks_mut::<4>();
            let (strips_ot, rest_ot) = ot_words.split_at_mut(3 * ns);
            let image_ot = &mut rest_ot[..ns];
            for i in 0..3 * ns {
                strips_ot[i] = self.edges[i].to_ne_bytes();
            }
            let (nodes_sl, rules_dummy) = self.edges.split_at_mut(nodes_words);
            let mut e = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
                nodes_buf: nodes_sl,
                rules_buf: rules_dummy,
                rule_top: 0,
                nodes: 0,
                id_overflow: false,
                diag: RuleDiag::default(),
                _links: std::marker::PhantomData,
            };
            e.node(0, 0);
            e.nodes = ns as u32 + 1;
            let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
            let mut x = 0u32;
            for i in 0..ns {
                let from = u32::from_ne_bytes(strips_ot[3 * i]);
                let to0 = u32::from_ne_bytes(strips_ot[3 * i + 1]);
                let wbyte = u32::from_ne_bytes(strips_ot[3 * i + 2]);
                let byte = (wbyte & 0xFF) as u8;
                let width = wbyte >> 8;
                let a = usize::from(byte);
                strips_ot[3 * i + 1] = (to0 + first[a]).to_ne_bytes();
                let dst_idx = write[a] as usize;
                image_ot[dst_idx] = (i as u32).to_ne_bytes();
                write[a] = dst_idx as u32 + 1;
                let sid = i as u32 + 1;
                e.set_width(sid, width);
                e.set_word(sid, literal(byte));
                widest = widest.max(width);
                if x < from {
                    let gw = from - x;
                    widest = widest.max(gw);
                    let gid = e.node(gw, 0);
                    e.append_to(&mut order[0], gid, 0);
                    e.append_to(&mut terminals[1], gid, 1);
                }
                e.append_to(&mut order[0], sid, 0);
                x = from + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            x = 0;
            for slot in image_ot.iter() {
                let i = u32::from_ne_bytes(*slot);
                let idx = i as usize;
                let to = u32::from_ne_bytes(strips_ot[3 * idx + 1]);
                let width = u32::from_ne_bytes(strips_ot[3 * idx + 2]) >> 8;
                if x < to {
                    widest = widest.max(to - x);
                    let id = e.node(to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = to + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            let open_nodes = e.nodes as usize - 1;
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            let base = L::STRIDE * e.nodes as usize;
            (base, e.nodes, open_nodes)
        };
        if let Some(ts) = t_s2a {
            tm.s2a_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let spirals = self.force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
        let mut remaining = n;
        let mut rule_top = 0usize;
        let mut diag = RuleDiag::default();
        loop {
            let (nodes_sl, rules_sl) = self.edges.split_at_mut(base);
            let mut e = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
                nodes_buf: nodes_sl,
                rules_buf: rules_sl,
                rule_top,
                nodes,
                id_overflow: false,
                diag,
                _links: std::marker::PhantomData,
            };
            let done = if spirals {
                e.eliminate::<true>(&mut remaining)
            } else {
                e.eliminate::<false>(&mut remaining)
            };
            rule_top = e.rule_top;
            diag = e.diag;
            if e.id_overflow {
                return (false, [0; 256], false, 0);
            }
            if done {
                break;
            }
            grow_edges_step(&mut self.edges, base + rule_top + SPIRAL_RULES * RULE4);
        }
        if let Some(ts) = t_s2b {
            tm.s2b_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        const CACHE_WORDS: usize = 512 + 2048;
        if ns >= 64 && self.edges.len() < base + rule_top + CACHE_WORDS {
            let need = base + rule_top + CACHE_WORDS;
            self.edges.reserve_exact(need - self.edges.len());
            self.edges.resize(need, 0);
        }
        let (live, tail_arena) = self.edges.split_at_mut(base + rule_top);
        let (nodes_sl, rules_sl) = live.split_at_mut(base);
        let mut e = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
            nodes_buf: nodes_sl,
            rules_buf: rules_sl,
            rule_top,
            nodes,
            id_overflow: false,
            diag,
            _links: std::marker::PhantomData,
        };
        e.assign_bottom(m);

        let (cursors, use_tail, nbands, memo_ovf_hits) = run_s3a_bands(
            &mut e,
            tail_arena,
            thin,
            open_nodes,
            self.force_no_tail,
            out_tail,
            &mut self.bands,
            &mut self.inline16,
            &mut self.lens,
            &mut self.spelling,
            &mut self.stack,
            &mut self.sat_memo,
        );
        if let Some(ts) = t_s3a {
            tm.s3a_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        if !TIMED {
            tel.e_nodes = nodes as usize;
            tel.open_nodes = open_nodes;
            tel.hdr_words = 0;
            tel.base_words = base;
            tel.rule_words = rule_top;
            tel.edges_s2b_peak_words = (base + rule_top).max(3 * ns);
            tel.touched_s2b_edges_bytes = (base + rule_top) * 4;
            tel.used_zero_hdr = true;
            tel.memo_ovf_hits = memo_ovf_hits;
            tel.rdiag = diag;
        }
        (true, cursors, use_tail, nbands)
    }

    // ------------------------------------------------------------------------
    // Mode 3: `OutScr-Rules` (`nodes` native in `edges`, `S2b` rules borrowed in `out_tail`!)
    // ------------------------------------------------------------------------
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn induced_outscr_rules<
        L: Links,
        const RMODE: u8,
        const TIMED: bool,
        const DIAG: bool,
    >(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
        tel: &mut BlockTelemetry,
    ) -> (bool, [usize; 256], bool, usize) {
        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        let nodes_words = L::STRIDE * most;
        if self.edges.len() < 4 * ns + nodes_words {
            grow_edges_step(&mut self.edges, 4 * ns + nodes_words);
        }

        let mut widest = 0u32;
        let (base, nodes, open_nodes) = {
            let (hdr, arena) = self.edges.split_at_mut(4 * ns);
            let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
            let (nodes_sl, rules_dummy) = arena.split_at_mut(nodes_words);
            let mut e = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
                nodes_buf: nodes_sl,
                rules_buf: rules_dummy,
                rule_top: 0,
                nodes: 0,
                id_overflow: false,
                diag: RuleDiag::default(),
                _links: std::marker::PhantomData,
            };
            e.node(0, 0);
            e.nodes = ns as u32 + 1;
            let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
            let mut x = 0u32;
            for i in 0..ns {
                let sw = &mut strips_raw[3 * i..3 * i + 3];
                let from = sw[0];
                let wbyte = sw[2];
                let byte = (wbyte & 0xFF) as u8;
                let width = wbyte >> 8;
                let a = usize::from(byte);
                sw[1] += first[a];
                let dst_idx = write[a] as usize;
                image_order[dst_idx] = i as u32;
                write[a] = dst_idx as u32 + 1;
                let sid = i as u32 + 1;
                e.set_width(sid, width);
                e.set_word(sid, literal(byte));
                widest = widest.max(width);
                if x < from {
                    let gw = from - x;
                    widest = widest.max(gw);
                    let gid = e.node(gw, 0);
                    e.append_to(&mut order[0], gid, 0);
                    e.append_to(&mut terminals[1], gid, 1);
                }
                e.append_to(&mut order[0], sid, 0);
                x = from + width;
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
                let sw = &strips_raw[3 * idx..3 * idx + 3];
                let to = sw[1];
                let width = sw[2] >> 8;
                if x < to {
                    widest = widest.max(to - x);
                    let id = e.node(to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = to + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            let open_nodes = e.nodes as usize - 1;
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            let base = L::STRIDE * e.nodes as usize;
            (base, e.nodes, open_nodes)
        };
        if let Some(ts) = t_s2a {
            tm.s2a_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        let max_bands_bytes = (open_nodes + 256) * 8;
        let rules_tail_bytes = out_tail.len().saturating_sub(max_bands_bytes);
        let can_try_out_rules = !self.force_no_tail && rules_tail_bytes >= 1024;

        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let spirals = self.force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
        let mut remaining = n;
        let mut rule_top = 0usize;
        let mut diag = RuleDiag::default();

        if can_try_out_rules {
            let (band_region, rule_bytes) = out_tail.split_at_mut(max_bands_bytes);
            let (rule_chunks, _) = rule_bytes.as_chunks_mut::<4>();
            let nodes_sl = &mut self.edges[4 * ns..4 * ns + base];
            let mut e = ElimSplit::<L, [u32], [[u8; 4]], RMODE, DIAG> {
                nodes_buf: nodes_sl,
                rules_buf: rule_chunks,
                rule_top: 0,
                nodes,
                id_overflow: false,
                diag,
                _links: std::marker::PhantomData,
            };
            let done = if spirals {
                e.eliminate::<true>(&mut remaining)
            } else {
                e.eliminate::<false>(&mut remaining)
            };
            rule_top = e.rule_top;
            diag = e.diag;
            if e.id_overflow {
                return (false, [0; 256], false, 0);
            }
            if done {
                if let Some(ts) = t_s2b {
                    tm.s2b_us = ts.elapsed().as_secs_f64() * 1e6;
                }
                let t_s3a = if TIMED { Some(Instant::now()) } else { None };
                let (header, arena) = self.edges.split_at_mut(4 * ns);
                let (nodes_sl, tail_arena) = arena.split_at_mut(base);
                let mut e = ElimSplit::<L, [u32], [[u8; 4]], RMODE, DIAG> {
                    nodes_buf: nodes_sl,
                    rules_buf: rule_chunks,
                    rule_top,
                    nodes,
                    id_overflow: false,
                    diag,
                    _links: std::marker::PhantomData,
                };
                e.assign_bottom(m);
                const CACHE_WORDS: usize = 512 + 2048;
                let cache = if header.len() >= CACHE_WORDS {
                    header
                } else {
                    tail_arena
                };
                let (cursors, use_tail, nbands, memo_ovf_hits) = run_s3a_bands(
                    &mut e,
                    cache,
                    thin,
                    open_nodes,
                    self.force_no_tail,
                    band_region,
                    &mut self.bands,
                    &mut self.inline16,
                    &mut self.lens,
                    &mut self.spelling,
                    &mut self.stack,
                    &mut self.sat_memo,
                );
                if let Some(ts) = t_s3a {
                    tm.s3a_us = ts.elapsed().as_secs_f64() * 1e6;
                }
                if !TIMED {
                    tel.e_nodes = nodes as usize;
                    tel.open_nodes = open_nodes;
                    tel.hdr_words = 4 * ns;
                    tel.base_words = base;
                    tel.rule_words = rule_top;
                    tel.edges_s2b_peak_words = 4 * ns + base;
                    tel.touched_s2b_edges_bytes = (4 * ns + base) * 4;
                    tel.used_out_rules = true;
                    tel.memo_ovf_hits = memo_ovf_hits;
                    tel.rdiag = diag;
                }
                return (true, cursors, use_tail, nbands);
            } else {
                // Out_tail filled up mid-S2b: spill already-created rules into `self.edges` and continue!
                let need_w = 4 * ns + base + rule_top + SPIRAL_RULES * RULE4;
                if self.edges.len() < need_w {
                    grow_edges_step(&mut self.edges, need_w);
                }
                let dst = &mut self.edges[4 * ns + base..4 * ns + base + rule_top];
                for (d, src) in dst.iter_mut().zip(rule_chunks[..rule_top].iter()) {
                    *d = u32::from_ne_bytes(*src);
                }
            }
        }

        // Fallback / continuation in `self.edges`
        loop {
            let (nodes_sl, rules_sl) = self.edges[4 * ns..].split_at_mut(base);
            let mut e = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
                nodes_buf: nodes_sl,
                rules_buf: rules_sl,
                rule_top,
                nodes,
                id_overflow: false,
                diag,
                _links: std::marker::PhantomData,
            };
            let done = if spirals {
                e.eliminate::<true>(&mut remaining)
            } else {
                e.eliminate::<false>(&mut remaining)
            };
            rule_top = e.rule_top;
            diag = e.diag;
            if e.id_overflow {
                return (false, [0; 256], false, 0);
            }
            if done {
                break;
            }
            grow_edges_step(&mut self.edges, 4 * ns + base + rule_top + SPIRAL_RULES * RULE4);
        }
        if let Some(ts) = t_s2b {
            tm.s2b_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail_arena) = arena.split_at_mut(base + rule_top);
        let (nodes_sl, rules_sl) = live.split_at_mut(base);
        let mut e = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
            nodes_buf: nodes_sl,
            rules_buf: rules_sl,
            rule_top,
            nodes,
            id_overflow: false,
            diag,
            _links: std::marker::PhantomData,
        };
        e.assign_bottom(m);
        const CACHE_WORDS: usize = 512 + 2048;
        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail_arena
        };
        let (cursors, use_tail, nbands, memo_ovf_hits) = run_s3a_bands(
            &mut e,
            cache,
            thin,
            open_nodes,
            self.force_no_tail,
            out_tail,
            &mut self.bands,
            &mut self.inline16,
            &mut self.lens,
            &mut self.spelling,
            &mut self.stack,
            &mut self.sat_memo,
        );
        if let Some(ts) = t_s3a {
            tm.s3a_us = ts.elapsed().as_secs_f64() * 1e6;
        }
        if !TIMED {
            tel.e_nodes = nodes as usize;
            tel.open_nodes = open_nodes;
            tel.hdr_words = 4 * ns;
            tel.base_words = base;
            tel.rule_words = rule_top;
            tel.edges_s2b_peak_words = 4 * ns + base + rule_top;
            tel.touched_s2b_edges_bytes = (4 * ns + base + rule_top) * 4;
            tel.used_out_rules = false;
            tel.memo_ovf_hits = memo_ovf_hits;
            tel.rdiag = diag;
        }
        (true, cursors, use_tail, nbands)
    }

    // ------------------------------------------------------------------------
    // Mode 4: `OutScr-Full` (BOTH `nodes` & `rules` borrowed in `out_tail` via safe `[u8; 4]`!)
    // ------------------------------------------------------------------------
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn induced_outscr_full<
        L: Links,
        const RMODE: u8,
        const TIMED: bool,
        const DIAG: bool,
    >(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
        tel: &mut BlockTelemetry,
    ) -> (bool, [usize; 256], bool, usize) {
        let max_bands_bytes = (most + 256) * 8;
        let nodes_bytes = L::STRIDE * most * 4;
        // Need room for bands + nodes + at least 1024 B of rules
        if self.force_no_tail || out_tail.len() < max_bands_bytes + nodes_bytes + 1024 {
            return self.induced_edges_native::<L, RMODE, TIMED, DIAG>(
                ns, n, m, thin, first, write, most, out_tail, tm, tel,
            );
        }

        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        // Ensure `self.edges` has 4*ns for `strips_raw` + `image_order` (and S3a cache)
        if self.edges.len() < 4 * ns {
            grow_edges_step(&mut self.edges, 4 * ns);
        }

        let (band_region, arena_bytes) = out_tail.split_at_mut(max_bands_bytes);
        let (arena_chunks, _) = arena_bytes.as_chunks_mut::<4>();
        let nodes_words = L::STRIDE * most;

        let mut widest = 0u32;
        let (base, nodes, open_nodes) = {
            let (strips_raw, image_order) = self.edges[..4 * ns].split_at_mut(3 * ns);
            let (nodes_sl, rules_dummy) = arena_chunks.split_at_mut(nodes_words);
            let mut e = ElimSplit::<L, [[u8; 4]], [[u8; 4]], RMODE, DIAG> {
                nodes_buf: nodes_sl,
                rules_buf: rules_dummy,
                rule_top: 0,
                nodes: 0,
                id_overflow: false,
                diag: RuleDiag::default(),
                _links: std::marker::PhantomData,
            };
            e.node(0, 0);
            e.nodes = ns as u32 + 1;
            let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
            let mut x = 0u32;
            for i in 0..ns {
                let sw = &mut strips_raw[3 * i..3 * i + 3];
                let from = sw[0];
                let wbyte = sw[2];
                let byte = (wbyte & 0xFF) as u8;
                let width = wbyte >> 8;
                let a = usize::from(byte);
                sw[1] += first[a];
                let dst_idx = write[a] as usize;
                image_order[dst_idx] = i as u32;
                write[a] = dst_idx as u32 + 1;
                let sid = i as u32 + 1;
                e.set_width(sid, width);
                e.set_word(sid, literal(byte));
                widest = widest.max(width);
                if x < from {
                    let gw = from - x;
                    widest = widest.max(gw);
                    let gid = e.node(gw, 0);
                    e.append_to(&mut order[0], gid, 0);
                    e.append_to(&mut terminals[1], gid, 1);
                }
                e.append_to(&mut order[0], sid, 0);
                x = from + width;
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
                let sw = &strips_raw[3 * idx..3 * idx + 3];
                let to = sw[1];
                let width = sw[2] >> 8;
                if x < to {
                    widest = widest.max(to - x);
                    let id = e.node(to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = to + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            let open_nodes = e.nodes as usize - 1;
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            let base = L::STRIDE * e.nodes as usize;
            (base, e.nodes, open_nodes)
        };
        if let Some(ts) = t_s2a {
            tm.s2a_us = ts.elapsed().as_secs_f64() * 1e6;
        }

        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let spirals = self.force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
        let mut remaining = n;
        let (nodes_sl, rules_sl) = arena_chunks.split_at_mut(base);
        let mut e = ElimSplit::<L, [[u8; 4]], [[u8; 4]], RMODE, DIAG> {
            nodes_buf: nodes_sl,
            rules_buf: rules_sl,
            rule_top: 0,
            nodes,
            id_overflow: false,
            diag: RuleDiag::default(),
            _links: std::marker::PhantomData,
        };
        let done = if spirals {
            e.eliminate::<true>(&mut remaining)
        } else {
            e.eliminate::<false>(&mut remaining)
        };
        let mut rule_top = e.rule_top;
        let mut diag = e.diag;
        if e.id_overflow {
            return (false, [0; 256], false, 0);
        }

        if done {
            if let Some(ts) = t_s2b {
                tm.s2b_us = ts.elapsed().as_secs_f64() * 1e6;
            }
            let t_s3a = if TIMED { Some(Instant::now()) } else { None };
            e.assign_bottom(m);
            let (cursors, use_tail, nbands, memo_ovf_hits) = run_s3a_bands(
                &mut e,
                &mut self.edges[..],
                thin,
                open_nodes,
                self.force_no_tail,
                band_region,
                &mut self.bands,
                &mut self.inline16,
                &mut self.lens,
                &mut self.spelling,
                &mut self.stack,
                &mut self.sat_memo,
            );
            if let Some(ts) = t_s3a {
                tm.s3a_us = ts.elapsed().as_secs_f64() * 1e6;
            }
            if !TIMED {
                tel.e_nodes = nodes as usize;
                tel.open_nodes = open_nodes;
                tel.hdr_words = 4 * ns;
                tel.base_words = base;
                tel.rule_words = rule_top;
                tel.edges_s2b_peak_words = (4 * ns).max(512 + 2048);
                tel.touched_s2b_edges_bytes = (4 * ns) * 4;
                tel.used_out_nodes = true;
                tel.used_out_rules = true;
                tel.memo_ovf_hits = memo_ovf_hits;
                tel.rdiag = diag;
            }
            return (true, cursors, use_tail, nbands);
        }

        // Spill partial out_tail arena into `self.edges` and finish natively!
        let need_w = 4 * ns + base + rule_top + SPIRAL_RULES * RULE4;
        if self.edges.len() < need_w {
            grow_edges_step(&mut self.edges, need_w);
        }
        for (d, src) in self.edges[4 * ns..4 * ns + base]
            .iter_mut()
            .zip(nodes_sl.iter())
        {
            *d = u32::from_ne_bytes(*src);
        }
        for (d, src) in self.edges[4 * ns + base..4 * ns + base + rule_top]
            .iter_mut()
            .zip(rules_sl[..rule_top].iter())
        {
            *d = u32::from_ne_bytes(*src);
        }
        loop {
            let (nsl, rsl) = self.edges[4 * ns..].split_at_mut(base);
            let mut e2 = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
                nodes_buf: nsl,
                rules_buf: rsl,
                rule_top,
                nodes,
                id_overflow: false,
                diag,
                _links: std::marker::PhantomData,
            };
            let done2 = if spirals {
                e2.eliminate::<true>(&mut remaining)
            } else {
                e2.eliminate::<false>(&mut remaining)
            };
            rule_top = e2.rule_top;
            diag = e2.diag;
            if e2.id_overflow {
                return (false, [0; 256], false, 0);
            }
            if done2 {
                break;
            }
            grow_edges_step(&mut self.edges, 4 * ns + base + rule_top + SPIRAL_RULES * RULE4);
        }
        if let Some(ts) = t_s2b {
            tm.s2b_us = ts.elapsed().as_secs_f64() * 1e6;
        }
        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail_arena) = arena.split_at_mut(base + rule_top);
        let (nsl, rsl) = live.split_at_mut(base);
        let mut e2 = ElimSplit::<L, [u32], [u32], RMODE, DIAG> {
            nodes_buf: nsl,
            rules_buf: rsl,
            rule_top,
            nodes,
            id_overflow: false,
            diag,
            _links: std::marker::PhantomData,
        };
        e2.assign_bottom(m);
        const CACHE_WORDS: usize = 512 + 2048;
        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail_arena
        };
        let (cursors, use_tail, nbands, memo_ovf_hits) = run_s3a_bands(
            &mut e2,
            cache,
            thin,
            open_nodes,
            self.force_no_tail,
            out_tail,
            &mut self.bands,
            &mut self.inline16,
            &mut self.lens,
            &mut self.spelling,
            &mut self.stack,
            &mut self.sat_memo,
        );
        if let Some(ts) = t_s3a {
            tm.s3a_us = ts.elapsed().as_secs_f64() * 1e6;
        }
        if !TIMED {
            tel.e_nodes = nodes as usize;
            tel.open_nodes = open_nodes;
            tel.hdr_words = 4 * ns;
            tel.base_words = base;
            tel.rule_words = rule_top;
            tel.edges_s2b_peak_words = 4 * ns + base + rule_top;
            tel.touched_s2b_edges_bytes = (4 * ns + base + rule_top) * 4;
            tel.memo_ovf_hits = memo_ovf_hits;
            tel.rdiag = diag;
        }
        (true, cursors, use_tail, nbands)
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_from_outtail64<const U16_NEXT: bool>(
        &mut self,
        cursors: &[usize; 256],
        nbands: usize,
        out_tail: &[u8],
        heads: &outq::Heads,
        out_q: &[u8],
        m: usize,
        bits: u32,
        wide: bool,
        use_xor: bool,
        mark: u32,
        mb: usize,
        predecessor: &mut u32,
    ) {
        let edges = &mut self.edges[..m];
        let (band_slots, _) = out_tail.as_chunks::<8>();
        if use_xor {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut queue = outq::Reader::<U16_NEXT>::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &raw in &band_slots[cursors[a]..end] {
                        let band = Band(u64::from_le_bytes(raw));
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                        );
                    }
                } else {
                    for &raw in &band_slots[cursors[a]..end] {
                        let band = Band(u64::from_le_bytes(raw));
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                            mark,
                            predecessor,
                        );
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut queue = outq::Reader::<U16_NEXT>::new(out_q, &self.spill, heads, a);
                for &raw in &band_slots[cursors[a]..end] {
                    let b = Band(u64::from_le_bytes(raw));
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_from_band64<const U16_NEXT: bool>(
        &mut self,
        cursors: &[usize; 256],
        heads: &outq::Heads,
        out_q: &[u8],
        m: usize,
        bits: u32,
        wide: bool,
        use_xor: bool,
        mark: u32,
        mb: usize,
        predecessor: &mut u32,
    ) {
        let edges = &mut self.edges[..m];
        if use_xor {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::<U16_NEXT>::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                        );
                    }
                } else {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                            mark,
                            predecessor,
                        );
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::<U16_NEXT>::new(out_q, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_s3a_bands<L: Links, N: WordBuf + ?Sized, R: WordBuf + ?Sized, const RMODE: u8, const DIAG: bool>(
    e: &mut ElimSplit<L, N, R, RMODE, DIAG>,
    cache: &mut [u32],
    thin: &[u32; 256],
    open_nodes: usize,
    force_no_tail: bool,
    out_tail: &mut [u8],
    bands: &mut Vec<Band>,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    sat_memo: &mut Vec<(u32, u32)>,
) -> ([usize; 256], bool, usize, usize) {
    const PAIR_WORDS: usize = 512;
    const CACHE_WORDS: usize = PAIR_WORDS + 2048;

    begin_inline(inline16, lens);
    let max_bands = open_nodes + 256;
    let use_tail = !force_no_tail && out_tail.len() >= max_bands * 8;
    let mut cursors = [0usize; 256];
    let mut nbands = 0usize;
    let mut memo_ovf_hits = 0usize;

    if cache.len() < CACHE_WORDS {
        let (band_slots, _) = out_tail.as_chunks_mut::<8>();
        let mut id = L::next(e.nodes_buf, 0, 0);
        let mut in_node = 0;
        let mut k = 0;
        for a in 0..256 {
            cursors[a] = if use_tail { nbands } else { bands.len() };
            let end = k + thin[a];
            while k < end {
                let (next_id, node_bottom, node_width, node_word) = L::band_node(e.nodes_buf, id);
                let width = (end - k).min(node_width - in_node);
                assert!(width > 0);
                let dst = node_bottom + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    let (entry, len) = e.inline_word(node_word, a as u8, stack, spelling, sat_memo);
                    ensure_step_inline(inline16, lens, INLINE_STEP);
                    let c = lens.len() as u32;
                    inline16.push(entry);
                    lens.push(len);
                    c
                };
                if use_tail {
                    band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                    nbands += 1;
                } else {
                    bands.push(Band::new(width, dst, code));
                }
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = next_id;
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
    } else {
        cache[..CACHE_WORDS].fill(0);
        let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
        let pairs: &mut [u32; 512] = p_sl.try_into().unwrap();
        let contents: &mut [u32; 2048] = c_sl.try_into().unwrap();
        let mut dictionary = Dictionary {
            pairs,
            contents,
            inline16,
            lens,
            spelling,
            stack,
            sat_memo,
            memo_ovf_hits: 0,
        };
        let mut id = L::next(e.nodes_buf, 0, 0);
        let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
            L::band_node(e.nodes_buf, id)
        } else {
            (0, 0, 0, 0)
        };
        let mut in_node = 0u32;
        let mut k = 0u32;
        if use_tail {
            let (band_slots, _) = out_tail[..max_bands * 8].as_chunks_mut::<8>();
            for a in 0..256 {
                cursors[a] = nbands;
                let end = k + thin[a];
                let mut last_w = 0u32;
                let mut last_c = 0u32;
                while k < end {
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else if node_word == last_w {
                        last_c
                    } else {
                        let c = dictionary.code(e, node_word, a as u8);
                        last_w = node_word;
                        last_c = c;
                        c
                    };
                    band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                    nbands += 1;
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                        if id != 0 {
                            (next_id, node_bottom, node_width, node_word) =
                                L::band_node(e.nodes_buf, id);
                        }
                    }
                }
            }
        } else {
            for a in 0..256 {
                cursors[a] = bands.len();
                let end = k + thin[a];
                let mut last_w = 0u32;
                let mut last_c = 0u32;
                while k < end {
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else if node_word == last_w {
                        last_c
                    } else {
                        let c = dictionary.code(e, node_word, a as u8);
                        last_w = node_word;
                        last_c = c;
                        c
                    };
                    bands.push(Band::new(width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                        if id != 0 {
                            (next_id, node_bottom, node_width, node_word) =
                                L::band_node(e.nodes_buf, id);
                        }
                    }
                }
            }
        }
        assert_eq!(id, 0);
        memo_ovf_hits = dictionary.memo_ovf_hits;
    }
    (cursors, use_tail, nbands, memo_ovf_hits)
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit(
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
    debug_assert!(out.len() >= n + INLINE);
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

#[inline(never)]
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
    debug_assert!(out.len() >= n + INLINE);
    let mask = (1 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0, n);
    for _ in 0..=edges.len() / 2 {
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
        if single || lo == hi {
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
        if lo == hi || front == next {
            break;
        }
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

// ============================================================================
// Reference Forward-FL, Corpora Loader, Exactness, Profiler & Benchmarks
// ============================================================================

fn forward_fl_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[usize::from(b)] += 1;
    }
    let mut st = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        st[c] = tot;
        tot += counts[c];
    }
    let mut fl = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let s = st[usize::from(b)] as usize;
        st[usize::from(b)] += 1;
        fl[s] = ((i as u32) << 8) | u32::from(b);
    }
    let mut out = vec![0u8; n];
    let mut x = p as usize;
    for slot in &mut out {
        let v = fl[x];
        *slot = v as u8;
        x = (v >> 8) as usize;
    }
    out
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
    let mut i = 0;
    while i < l.len() {
        let b = l[i];
        let mut w = 1usize;
        while i + w < l.len() && l[i + w] == b && w < 127 {
            w += 1;
        }
        recs.push((u16::from(b) << 8) | (w as u16));
        i += w;
    }
    recs
}

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
}

fn load_six_admitted() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let manifest = fs::read_to_string(format!("{dir}/manifest.json")).expect("read manifest");
    let mut blocks = Vec::new();
    for id in [0usize, 2, 85, 143, 207, 210] {
        let key = format!("\"block-{id}.L\"");
        let pos = manifest.find(&key).expect("find block");
        let sub = &manifest[pos..];
        let ppos = sub.find("\"primary_index\":").expect("find primary_index");
        let rest = sub[ppos + 16..].trim_start();
        let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        let p: u32 = rest[..end].parse().expect("parse primary_index");
        let l = fs::read(format!("{dir}/block-{id}.L")).expect("read .L");
        blocks.push(Block {
            name: format!("block-{id}.L"),
            n: l.len(),
            p,
            records: records_of(&l),
        });
    }
    blocks
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::new();
    for b_idx in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let rc = raw[off + 16];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        if only_full && rc > 1 {
            continue;
        }
        let name = names
            .and_then(|ns| ns.get(b_idx).map(|s| (*s).to_string()))
            .unwrap_or_else(|| format!("w431-{idx}"));
        blocks.push(Block {
            name,
            n,
            p,
            records,
        });
    }
    blocks
}

fn resolve_path(cands: &[&str]) -> String {
    for &p in cands {
        if Path::new(p).exists() {
            return p.to_string();
        }
    }
    panic!("could not find corpus in {:?}", cands);
}

fn run_one<const TIMED: bool, const DIAG: bool>(
    arm: usize,
    k: &mut C51Kernel,
    blk: &Block,
    out: &mut Vec<u8>,
) -> (StageTimes, BlockTelemetry) {
    let (ok, tm, tel) = match arm as u8 {
        ARM_C50_A1 => k.run_arm::<ARM_C50_A1, TIMED, DIAG>(&blk.records, blk.n, blk.p, out),
        ARM_C50_A2 => k.run_arm::<ARM_C50_A2, TIMED, DIAG>(&blk.records, blk.n, blk.p, out),
        ARM_CE3_FIXED12 => k.run_arm::<ARM_CE3_FIXED12, TIMED, DIAG>(&blk.records, blk.n, blk.p, out),
        ARM_CE_VAR234 => k.run_arm::<ARM_CE_VAR234, TIMED, DIAG>(&blk.records, blk.n, blk.p, out),
        ARM_CE_VAR_ZEROH => k.run_arm::<ARM_CE_VAR_ZEROH, TIMED, DIAG>(&blk.records, blk.n, blk.p, out),
        ARM_OUTSCR_RULES => k.run_arm::<ARM_OUTSCR_RULES, TIMED, DIAG>(&blk.records, blk.n, blk.p, out),
        ARM_OUTSCR_FULL => k.run_arm::<ARM_OUTSCR_FULL, TIMED, DIAG>(&blk.records, blk.n, blk.p, out),
        ARM_CE34_FAST => k.run_arm::<ARM_CE34_FAST, TIMED, DIAG>(&blk.records, blk.n, blk.p, out),
        ARM_CE34_ZEROH => k.run_arm::<ARM_CE34_ZEROH, TIMED, DIAG>(&blk.records, blk.n, blk.p, out),
        _ => unreachable!(),
    };
    assert!(ok, "arm {} failed on {}", ARM_NAMES[arm], blk.name);
    (tm, tel)
}

struct SplitMix64(u64);
impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

fn bwt_of(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        for off in 0..n {
            let ca = text[(a + off) % n];
            let cb = text[(b + off) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (row, &rot) in order.iter().enumerate() {
        if rot == 0 {
            p = row as u32;
        }
        l.push(text[(rot + n - 1) % n]);
    }
    (l, p)
}

fn make_fib_bwt_block(k_fib: usize) -> Block {
    let mut a = 1usize;
    let mut b = 1usize;
    for _ in 2..k_fib {
        let c = a + b;
        a = b;
        b = c;
    }
    // Christoffel Fibonacci BWT: b^{F_{k-2}} a^{F_{k-1}}, n = F_k
    let q = b - a; // F_{k-2}
    let p_cnt = a; // F_{k-1}
    let n = b;
    let mut l = Vec::with_capacity(n);
    l.resize(q, b'b');
    l.resize(q + p_cnt, b'a');
    Block {
        name: format!("Fib-F{k_fib}(n={n})"),
        n,
        p: 0,
        records: records_of(&l),
    }
}

fn make_low_m_high_ns_stress() -> Vec<Block> {
    let mut out = Vec::new();
    // 1. Fibonacci F_25..F_29 (2-run Christoffel where S2b joins >> open_nodes!)
    for kf in [25usize, 27, 29] {
        out.push(make_fib_bwt_block(kf));
    }
    // 2. Multi-strip low-m block: ~6,000 thick runs of length 100..200 with tiny thin gaps
    //    so `4*ns + top > m`! (Tests CompactEdges3 & OutScratch when S2b dominates `edges`!)
    let mut rng = SplitMix64(0xC51E_D6E3_0001_BEEF);
    for (tag, n_target, thick_lo, thick_hi, gap_max) in [
        ("LowM-DenseThick-A", 400_000usize, 40usize, 100usize, 4usize),
        ("LowM-DenseThick-B", 600_000usize, 24usize, 60usize, 2usize),
        ("LowM-LongRules-C", 500_000usize, 80usize, 260usize, 3usize),
    ] {
        let mut l = Vec::with_capacity(n_target);
        let mut prev_b = 255u8;
        while l.len() < n_target {
            let mut b = (rng.next() as usize % 32) as u8;
            if b == prev_b {
                b = (b + 1) % 32;
            }
            prev_b = b;
            let rl = thick_lo + (rng.next() as usize % (thick_hi - thick_lo + 1));
            for _ in 0..rl.min(n_target - l.len()) {
                l.push(b);
            }
            if l.len() < n_target && (rng.next() % 5 == 0) {
                let gl = 1 + (rng.next() as usize % gap_max);
                let gb = (b + 17) % 64;
                for _ in 0..gl.min(n_target - l.len()) {
                    l.push(gb);
                }
                prev_b = gb;
            }
        }
        let n = l.len();
        let p = (n / 7) as u32;
        out.push(Block {
            name: tag.to_string(),
            n,
            p,
            records: records_of(&l),
        });
    }
    out
}

fn run_exactness_suite(
    wiki6: &[Block],
    nonwiki10: &[Block],
    wiki431: &[Block],
    low_m_stress: &[Block],
) {
    println!("================================================================================");
    println!("PART 1: EXACTNESS, POISON, MALFORMED FORWARD-FL, FORCED FALLBACKS & 447 REAL BLKS");
    println!("================================================================================");
    let mut ks: Vec<C51Kernel> = (0..NUM_ARMS).map(|_| C51Kernel::default()).collect();
    let mut out_got = Vec::new();
    let mut small_cnt = 0usize;
    let mut mal_cnt = 0usize;
    let mut stress_cnt = 0usize;

    fn verify_all_arms(
        ks: &mut [C51Kernel],
        out_got: &mut Vec<u8>,
        recs: &[u16],
        n: usize,
        p: u32,
        want: &[u8],
        tag: &str,
    ) {
        let blk = Block {
            name: tag.to_string(),
            n,
            p,
            records: recs.to_vec(),
        };
        for arm in 0..NUM_ARMS {
            let poison = if arm % 2 == 0 { 0xAA } else { 0x55 };
            out_got.clear();
            out_got.resize(n + 37, poison);
            run_one::<false, false>(arm, &mut ks[arm], &blk, out_got);
            assert_eq!(
                out_got.as_slice(),
                want,
                "MISMATCH on {tag} arm={} (n={n}, p={p})",
                ARM_NAMES[arm]
            );
        }
    }

    // 1. Exhaustive binary small words n=1..=8 (510) + 5,250 random valid BWTs
    for n in 1..=8usize {
        for mask in 0..(1usize << n) {
            let text: Vec<u8> = (0..n)
                .map(|i| if (mask >> i) & 1 == 0 { b'a' } else { b'b' })
                .collect();
            let (l, p) = bwt_of(&text);
            let want = forward_fl_invert(&l, p);
            assert_eq!(want, text);
            verify_all_arms(&mut ks, &mut out_got, &records_of(&l), n, p, &want, "small-bin");
            small_cnt += 1;
        }
    }
    let mut rng = SplitMix64(0xC51E_D6E3_CAFE_5101);
    for case in 0..5250usize {
        let n = match case % 5 {
            0 => 1 + (rng.next() as usize % 64),
            1 => 65 + (rng.next() as usize % 256),
            2 => 320 + (rng.next() as usize % 1024),
            3 => 1344 + (rng.next() as usize % 2048),
            _ => 16 + (rng.next() as usize % 128),
        };
        let alpha = match case % 4 {
            0 => 2,
            1 => 4,
            2 => 16,
            _ => 256,
        };
        let mut text = Vec::with_capacity(n);
        if case % 7 == 0 {
            while text.len() < n {
                let b = (rng.next() as usize % alpha) as u8;
                let rl = 1 + (rng.next() as usize % 40);
                for _ in 0..rl.min(n - text.len()) {
                    text.push(b);
                }
            }
        } else {
            for _ in 0..n {
                text.push((rng.next() as usize % alpha) as u8);
            }
        }
        let (l, p) = bwt_of(&text);
        verify_all_arms(&mut ks, &mut out_got, &records_of(&l), n, p, &text, "small-rand");
        small_cnt += 1;
    }

    // 2. Malformed multi-cycle forward-FL columns (330 cases, including large n >= 65_536 bi-XOR!)
    for case in 0..330usize {
        let n = if case < 300 {
            2 + (rng.next() as usize % 1500)
        } else if case < 320 {
            33_000 + (rng.next() as usize % 40_000)
        } else {
            70_000 + (rng.next() as usize % 30_000)
        };
        let mut l = Vec::with_capacity(n);
        if case % 3 == 0 {
            let alpha = 2 + (rng.next() as usize % 8);
            while l.len() < n {
                let b = (rng.next() as usize % alpha) as u8;
                let rl = 1 + (rng.next() as usize % 35);
                for _ in 0..rl.min(n - l.len()) {
                    l.push(b);
                }
            }
        } else {
            for _ in 0..n {
                l.push((rng.next() as usize % 64) as u8);
            }
        }
        let p = (rng.next() as usize % n) as u32;
        let want = forward_fl_invert(&l, p);
        verify_all_arms(&mut ks, &mut out_got, &records_of(&l), n, p, &want, "malformed-fl");
        mal_cnt += 1;
    }

    // 3. Stress: forced Wide, forced NoTail, forced Spirals, forced NoOutScr, m=65536 boundary,
    //    low-m/high-ns & Fibonacci stress, and >16M-record TinyFL guard
    for &n in &[40_000usize, 65_535, 65_536, 65_537, 100_000] {
        let mut l = vec![0u8; n];
        for (i, slot) in l.iter_mut().enumerate() {
            *slot = ((i * 37 + (i >> 5) * 13) & 63) as u8;
        }
        for pos in (0..n.saturating_sub(200)).step_by(500) {
            let b = ((pos / 500) % 16) as u8;
            for j in 0..64 {
                l[pos + j] = b;
            }
        }
        let p = (n / 3) as u32;
        let want = forward_fl_invert(&l, p);
        let recs = records_of(&l);
        verify_all_arms(&mut ks, &mut out_got, &recs, n, p, &want, "stress-boundary");
        for arm in 2..NUM_ARMS {
            ks[arm].force_wide = true;
            ks[arm].force_no_tail = true;
            ks[arm].force_spirals = Some(true);
            out_got.clear();
            out_got.resize(n + 37, 0xCC);
            let blk = Block {
                name: "forced-flags".to_string(),
                n,
                p,
                records: recs.clone(),
            };
            run_one::<false, false>(arm, &mut ks[arm], &blk, &mut out_got);
            assert_eq!(out_got.as_slice(), want.as_slice(), "forced-flags mismatch arm={arm}");
            ks[arm].force_wide = false;
            ks[arm].force_no_tail = false;
            ks[arm].force_spirals = None;
            // Also force no_outscr fallback
            ks[arm].force_no_outscr = true;
            out_got.clear();
            out_got.resize(n + 37, 0x33);
            run_one::<false, false>(arm, &mut ks[arm], &blk, &mut out_got);
            assert_eq!(out_got.as_slice(), want.as_slice(), "no-outscr mismatch arm={arm}");
            ks[arm].force_no_outscr = false;
        }
        stress_cnt += 1;
    }

    // Low-m/high-ns + Fibonacci stress exactness across all 7 arms
    let mut out_ref = Vec::new();
    for blk in low_m_stress {
        out_ref.clear();
        out_ref.resize(blk.n + 37, 0xAA);
        run_one::<false, false>(0, &mut ks[0], blk, &mut out_ref);
        for arm in 1..NUM_ARMS {
            out_got.clear();
            out_got.resize(blk.n + 37, if arm % 2 == 0 { 0xAA } else { 0x55 });
            run_one::<false, false>(arm, &mut ks[arm], blk, &mut out_got);
            assert_eq!(
                out_got.as_slice(),
                out_ref.as_slice(),
                "LowM stress mismatch on {} arm={}",
                blk.name,
                ARM_NAMES[arm]
            );
        }
        stress_cnt += 1;
    }

    // >16M-record defensive guard in invert_tiny_fl
    {
        let mut k = C51Kernel::default();
        let big_empty = vec![0u16; 16_000_001];
        let mut dummy_out = Vec::new();
        assert!(!k.invert_tiny_fl(&big_empty, 100, 0, &mut dummy_out));
        stress_cnt += 1;
    }

    // 4. All 6 Admitted + 10 Non-Wiki/CodeDocs + 431 ALL Wikidata exact cross-arm parity
    for (cname, slice) in [
        ("6 Admitted", wiki6),
        ("10 Non-Wiki+CodeDocs", nonwiki10),
        ("431 ALL Wikidata", wiki431),
    ] {
        for blk in slice {
            out_ref.clear();
            out_ref.resize(blk.n + 37, 0xAA);
            run_one::<false, false>(0, &mut ks[0], blk, &mut out_ref);
            for arm in 1..NUM_ARMS {
                out_got.clear();
                out_got.resize(blk.n + 37, if arm % 2 == 0 { 0xAA } else { 0x55 });
                run_one::<false, false>(arm, &mut ks[arm], blk, &mut out_got);
                assert_eq!(
                    out_got.as_slice(),
                    out_ref.as_slice(),
                    "Real-block mismatch on {cname}/{} arm={}",
                    blk.name,
                    ARM_NAMES[arm]
                );
            }
        }
    }

    println!(
        "EXACTNESS PASS: {} small valid + {} malformed forward-FL + {} stress/forced-fallback/LowM/16M-guard + 6 Adm + 10 NonWiki/CodeDocs + 431 ALL Wikidata (ALL {} ARMS 100% BYTE-EXACT!)",
        small_cnt, mal_cnt, stress_cnt, NUM_ARMS
    );
}

fn audit_edges_phase_and_rules(blocks: &[Block], label: &str) {
    println!("\n================================================================================");
    println!("PART 2: PER-PHASE `edges` (S2a/S2b vs S4) & RULE-HISTOGRAM + CAYLEY CHECK — {} (N={})", label, blocks.len());
    println!("================================================================================");

    let mut k = C51Kernel::default();
    let mut out = Vec::new();
    let mut full = Vec::new();
    for blk in blocks {
        let (_, tel) = run_one::<false, true>(ARM_C50_A1 as usize, &mut k, blk, &mut out);
        if !tel.is_tiny {
            full.push((blk.name.as_str(), tel));
        }
    }
    let nf = full.len().max(1) as f64;
    let mut sum_m = 0usize;
    let mut sum_ns = 0usize;
    let mut sum_enodes = 0usize;
    let mut sum_most = 0usize;
    let mut sum_hdr = 0usize;
    let mut sum_base = 0usize;
    let mut sum_w_c50 = 0usize;
    let mut sum_w_ce3 = 0usize;
    let mut sum_w_var = 0usize;
    let mut sum_r13 = 0usize;
    let mut sum_r48 = 0usize;
    let mut sum_r912 = 0usize;
    let mut sum_r1315 = 0usize;
    let mut sum_r16254 = 0usize;
    let mut sum_r255p = 0usize;
    let mut sum_joins = 0usize;
    let mut sum_steps = 0usize;
    let mut gt_m_c50 = 0usize;
    let mut gt_m_ce3 = 0usize;
    let mut gt_m_var = 0usize;
    let mut gt_m_zeroh = 0usize;
    let mut cayley_alloc_viol = 0usize;
    let mut cayley_join_viol = 0usize;
    let mut max_ratio_c50 = 0.0f64;
    let mut max_ratio_var = 0.0f64;
    let mut max_sp = 0usize;
    let mut max_chunks = 0usize;
    let mut sum_len16 = 0usize;
    let mut sum_codes = 0usize;
    let mut sum_memo_ovf = 0usize;
    let mut max_join_over_open = 0.0f64;
    let mut max_alloc_over_open = 0.0f64;

    for &(name, t) in &full {
        sum_m += t.m;
        sum_ns += t.ns;
        sum_enodes += t.e_nodes;
        sum_most += t.most_nodes;
        sum_hdr += t.hdr_words;
        sum_base += t.base_words;
        let wc50 = t.rdiag.words_c50();
        let wce3 = t.rdiag.words_ce3();
        let wvar = t.rdiag.words_var234();
        sum_w_c50 += wc50;
        sum_w_ce3 += wce3;
        sum_w_var += wvar;
        sum_r13 += t.rdiag.r_1_3;
        sum_r48 += t.rdiag.r_4_8;
        sum_r912 += t.rdiag.r_9_12;
        sum_r1315 += t.rdiag.r_13_15;
        sum_r16254 += t.rdiag.r_16_254;
        sum_r255p += t.rdiag.r_255p;
        sum_joins += t.rdiag.total_joins;
        sum_steps += t.rdiag.s2b_steps;
        let p_c50 = t.hdr_words + t.base_words + wc50;
        let p_ce3 = t.hdr_words + t.base_words + wce3;
        let p_var = t.hdr_words + t.base_words + wvar;
        let p_zh = t.base_words + wvar;
        if p_c50 > t.m {
            gt_m_c50 += 1;
        }
        if p_ce3 > t.m {
            gt_m_ce3 += 1;
        }
        if p_var > t.m {
            gt_m_var += 1;
        }
        if p_zh > t.m {
            gt_m_zeroh += 1;
        }
        max_ratio_c50 = max_ratio_c50.max(p_c50 as f64 / t.m.max(1) as f64);
        max_ratio_var = max_ratio_var.max(p_var as f64 / t.m.max(1) as f64);
        max_sp = max_sp.max(t.spelling_len);
        max_chunks = max_chunks.max(t.spill_chunks);
        sum_len16 += t.n_len16;
        sum_codes += t.ncodes_stored;
        sum_memo_ovf += t.memo_ovf_hits;
        if t.rdiag.num_alloc() > t.open_nodes.saturating_sub(1) {
            cayley_alloc_viol += 1;
        }
        if t.rdiag.total_joins > t.open_nodes.saturating_sub(1) {
            cayley_join_viol += 1;
        }
        if t.open_nodes > 0 {
            max_join_over_open = max_join_over_open.max(t.rdiag.total_joins as f64 / t.open_nodes as f64);
            max_alloc_over_open = max_alloc_over_open.max(t.rdiag.num_alloc() as f64 / t.open_nodes as f64);
        }
        if blocks.len() <= 6 {
            println!(
                "  {:<18}: m={:>7} ({:>6.1}K) | ns={:>5} nodes={:>5}/{:>5} | S2b_peak(C50/CE3/Var/ZH)={:>6.1}/{:>6.1}/{:>6.1}/{:>6.1}K | rules(4-8/9-12/13-15/16-254/255+)={}/{}/{}/{}/{} | joins/open={:.2}",
                name,
                t.m,
                t.m as f64 * 4.0 / 1024.0,
                t.ns,
                t.e_nodes,
                t.most_nodes,
                p_c50 as f64 * 4.0 / 1024.0,
                p_ce3 as f64 * 4.0 / 1024.0,
                p_var as f64 * 4.0 / 1024.0,
                p_zh as f64 * 4.0 / 1024.0,
                t.rdiag.r_4_8,
                t.rdiag.r_9_12,
                t.rdiag.r_13_15,
                t.rdiag.r_16_254,
                t.rdiag.r_255p,
                t.rdiag.total_joins as f64 / t.open_nodes.max(1) as f64,
            );
        }
    }

    let tot_alloc = (sum_r48 + sum_r912 + sum_r1315 + sum_r16254 + sum_r255p).max(1) as f64;
    let avg_m_k = sum_m as f64 * 4.0 / 1024.0 / nf;
    let avg_hdr_k = sum_hdr as f64 * 4.0 / 1024.0 / nf;
    let avg_base_k = sum_base as f64 * 4.0 / 1024.0 / nf;
    let avg_rc50_k = sum_w_c50 as f64 * 4.0 / 1024.0 / nf;
    let avg_rce3_k = sum_w_ce3 as f64 * 4.0 / 1024.0 / nf;
    let avg_rvar_k = sum_w_var as f64 * 4.0 / 1024.0 / nf;
    println!(
        "  SUMMARY (full={}): avg m={:.1} KiB | avg S2a(hdr 4*ns={:.1} + base={:.1}) + rules(C50={:.1} -> CE3={:.1} [{:+.1}%] -> Var234={:.1} [{:+.1}%])",
        full.len(),
        avg_m_k,
        avg_hdr_k,
        avg_base_k,
        avg_rc50_k,
        avg_rce3_k,
        (avg_rce3_k / avg_rc50_k.max(1e-9) - 1.0) * 100.0,
        avg_rvar_k,
        (avg_rvar_k / avg_rc50_k.max(1e-9) - 1.0) * 100.0,
    );
    println!(
        "  S2b_peak/m: avg_c50={:.3} max_c50={:.3} | max_var={:.3} | S2b_peak>m (C50/CE3/Var/ZeroH) = {}/{}/{}/{} of {}",
        (avg_hdr_k + avg_base_k + avg_rc50_k) / avg_m_k.max(1e-9),
        max_ratio_c50,
        max_ratio_var,
        gt_m_c50,
        gt_m_ce3,
        gt_m_var,
        gt_m_zeroh,
        full.len()
    );
    println!(
        "  Rule Histogram (avg/blk, % of alloc): inline1..3={:.0} | 4..8={:.0}({:.1}%) 9..12={:.0}({:.1}%) 13..15={:.0}({:.1}%) 16..254={:.0}({:.1}%) 255+={:.1}({:.2}%) | steps={:.0} joins={:.0}",
        sum_r13 as f64 / nf,
        sum_r48 as f64 / nf, sum_r48 as f64 * 100.0 / tot_alloc,
        sum_r912 as f64 / nf, sum_r912 as f64 * 100.0 / tot_alloc,
        sum_r1315 as f64 / nf, sum_r1315 as f64 * 100.0 / tot_alloc,
        sum_r16254 as f64 / nf, sum_r16254 as f64 * 100.0 / tot_alloc,
        sum_r255p as f64 / nf, sum_r255p as f64 * 100.0 / tot_alloc,
        sum_steps as f64 / nf,
        sum_joins as f64 / nf,
    );
    println!(
        "  Nodes & Bounds: avg ns={:.1} e.nodes={:.1}/{:.1} ({:.1}% of 3ns+3) | Cayley viol (alloc>open-1: {}/{}, max={:.2}x; joins>open-1: {}/{}, max={:.2}x)",
        sum_ns as f64 / nf,
        sum_enodes as f64 / nf,
        sum_most as f64 / nf,
        sum_enodes as f64 * 100.0 / sum_most.max(1) as f64,
        cayley_alloc_viol, full.len(), max_alloc_over_open,
        cayley_join_viol, full.len(), max_join_over_open,
    );
    println!(
        "  S3a/Spill Limits: max_spelling={} B ({:.2}% of 2^23) | max_chunks={} (u16 cap 65535) | len==16 codes={:.1}/{:.1} | memo_ovf_hits={}",
        max_sp,
        max_sp as f64 * 100.0 / (1u32 << 23) as f64,
        max_chunks,
        sum_len16 as f64 / nf,
        sum_codes as f64 / nf,
        sum_memo_ovf
    );
}

fn audit_ws_all_arms(blocks: &[Block], label: &str) {
    println!("\n================================================================================");
    println!("PART 3: PER-BUFFER WS & TOUCHED ATTRIBUTION ACROSS ARMS — {} (N={})", label, blocks.len());
    println!("================================================================================");

    for &arm in &[
        ARM_C50_A1 as usize,
        ARM_CE3_FIXED12 as usize,
        ARM_CE_VAR234 as usize,
        ARM_CE_VAR_ZEROH as usize,
        ARM_OUTSCR_RULES as usize,
        ARM_OUTSCR_FULL as usize,
        ARM_CE34_FAST as usize,
        ARM_CE34_ZEROH as usize,
    ] {
        let mut k = C51Kernel::default();
        let mut out = Vec::new();
        let mut tels = Vec::with_capacity(blocks.len());
        let mut out_rules_hits = 0usize;
        let mut out_nodes_hits = 0usize;
        let mut zero_hdr_hits = 0usize;
        for blk in blocks {
            let (_, tel) = run_one::<false, false>(arm, &mut k, blk, &mut out);
            if !tel.is_tiny {
                if tel.used_out_rules {
                    out_rules_hits += 1;
                }
                if tel.used_out_nodes {
                    out_nodes_hits += 1;
                }
                if tel.used_zero_hdr {
                    zero_hdr_hits += 1;
                }
            }
            tels.push(tel);
        }
        let u16_next = arm as u8 >= ARM_CE3_FIXED12;
        let full: Vec<&BlockTelemetry> = tels.iter().filter(|t| !t.is_tiny).collect();
        let nf = full.len().max(1) as f64;
        let (tot_cap, e_cap, sp_cap, i16_cap, ln_cap, sn_cap, sc_cap, so_cap, bd_cap, ex_cap, st_cap, sm_cap) =
            k.ws_breakdown(u16_next);
        let max_e = tels.iter().map(|t| t.edges_peak_used_words * 4).max().unwrap_or(0);
        let max_sp = full.iter().map(|t| t.spelling_len).max().unwrap_or(0);
        let max_i16 = full.iter().map(|t| t.ncodes_stored * INLINE).max().unwrap_or(0);
        let max_ln = full.iter().map(|t| t.ncodes_stored).max().unwrap_or(0);
        let max_sn = full.iter().map(|t| t.spill_chunks * if u16_next { 2 } else { 4 }).max().unwrap_or(0);
        let max_sc = full.iter().map(|t| t.spill_cuts * 1024).max().unwrap_or(0);
        let max_so = full.iter().map(|t| t.spill_ovf_chunks * outq::BYTES).max().unwrap_or(0);
        let max_bd = full.iter().map(|t| t.bands_fb_len * 8).max().unwrap_or(0);
        let max_ex = full.iter().map(|t| t.extra_len * 4).max().unwrap_or(0);
        let max_sm = full.iter().map(|t| t.sat_memo_len * 8).max().unwrap_or(0);
        let max_tot = max_e + max_sp + max_i16 + max_ln + max_sn + max_sc + max_so + max_bd + max_ex + max_sm;

        let avg_s2b_tch = full.iter().map(|t| t.touched_s2b_edges_bytes as f64).sum::<f64>() / nf / 1024.0;
        let avg_tot_tch = full.iter().map(|t| t.touched_total_bytes as f64).sum::<f64>() / nf / 1024.0;
        let gt_2048 = full.iter().filter(|t| t.touched_total_bytes > 2048 * 1024).count();

        println!(
            "[{:<12}] PeakCap={:>7.2} KiB | MaxUsed={:>7.2} KiB | edgesCap={:>7.2} sp={:>5.2} in16={:>5.2} sn={:>4.2} sm={:>4.2} | AvgS2bEdgesTch={:>6.1}K | AvgTotTch={:>6.1}K (>2048K:{}/{}) | ZH={}/OR={}/ON={}",
            ARM_NAMES[arm],
            tot_cap as f64 / 1024.0,
            max_tot as f64 / 1024.0,
            e_cap as f64 / 1024.0,
            sp_cap as f64 / 1024.0,
            i16_cap as f64 / 1024.0,
            sn_cap as f64 / 1024.0,
            sm_cap as f64 / 1024.0,
            avg_s2b_tch,
            avg_tot_tch,
            gt_2048,
            full.len(),
            zero_hdr_hits,
            out_rules_hits,
            out_nodes_hits,
        );
        let _ = (ln_cap, sc_cap, so_cap, bd_cap, ex_cap, st_cap);
    }
}

fn bench_stage_1t(blocks: &[Block], label: &str, rounds: usize) {
    println!("\n--- Stage-by-Stage 1T Timings (p50 summed us, {rounds} paired rounds) — {label} ---");
    let mut ks: Vec<C51Kernel> = (0..NUM_ARMS).map(|_| C51Kernel::default()).collect();
    let mut out = Vec::new();

    let mut s1 = [0.0f64; NUM_ARMS];
    let mut s2a = [0.0f64; NUM_ARMS];
    let mut s2b = [0.0f64; NUM_ARMS];
    let mut s3a = [0.0f64; NUM_ARMS];
    let mut s4 = [0.0f64; NUM_ARMS];
    let mut s5 = [0.0f64; NUM_ARMS];
    let mut ind = [0.0f64; NUM_ARMS];
    let mut tot = [0.0f64; NUM_ARMS];

    let p50 = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() / 2]
    };

    for blk in blocks {
        for arm in 0..NUM_ARMS {
            run_one::<false, false>(arm, &mut ks[arm], blk, &mut out);
        }
        let mut vs1 = vec![Vec::with_capacity(rounds); NUM_ARMS];
        let mut vs2a = vec![Vec::with_capacity(rounds); NUM_ARMS];
        let mut vs2b = vec![Vec::with_capacity(rounds); NUM_ARMS];
        let mut vs3a = vec![Vec::with_capacity(rounds); NUM_ARMS];
        let mut vs4 = vec![Vec::with_capacity(rounds); NUM_ARMS];
        let mut vs5 = vec![Vec::with_capacity(rounds); NUM_ARMS];
        let mut vind = vec![Vec::with_capacity(rounds); NUM_ARMS];
        let mut vtot = vec![Vec::with_capacity(rounds); NUM_ARMS];

        for _ in 0..rounds {
            for arm in 0..NUM_ARMS {
                let (tm, _) = run_one::<true, false>(arm, &mut ks[arm], blk, &mut out);
                vs1[arm].push(tm.s1_us);
                vs2a[arm].push(tm.s2a_us);
                vs2b[arm].push(tm.s2b_us);
                vs3a[arm].push(tm.s3a_us);
                vs4[arm].push(tm.s4_us);
                vs5[arm].push(tm.s5_us);
                vind[arm].push(tm.ind_us());
            }
            for arm in 0..NUM_ARMS {
                let t0 = Instant::now();
                run_one::<false, false>(arm, &mut ks[arm], blk, &mut out);
                vtot[arm].push(t0.elapsed().as_secs_f64() * 1e6);
            }
        }
        for arm in 0..NUM_ARMS {
            s1[arm] += p50(&mut vs1[arm]);
            s2a[arm] += p50(&mut vs2a[arm]);
            s2b[arm] += p50(&mut vs2b[arm]);
            s3a[arm] += p50(&mut vs3a[arm]);
            s4[arm] += p50(&mut vs4[arm]);
            s5[arm] += p50(&mut vs5[arm]);
            ind[arm] += p50(&mut vind[arm]);
            tot[arm] += p50(&mut vtot[arm]);
        }
    }

    println!(
        "{:<13} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} | {:>8} {:>7} | {:>8} {:>7}",
        "Arm", "S1", "S2a", "S2b", "S3a", "S4", "S5", "Ind(us)", "vsA1", "Tot(us)", "vsA1"
    );
    for arm in 0..NUM_ARMS {
        println!(
            "{:<13} | {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} | {:>8.1} {:>6.3}x | {:>8.1} {:>6.3}x",
            ARM_NAMES[arm],
            s1[arm], s2a[arm], s2b[arm], s3a[arm], s4[arm], s5[arm],
            ind[arm], ind[arm] / ind[0],
            tot[arm], tot[arm] / tot[0],
        );
    }
}

fn bench_multiworker(
    blocks: &[Block],
    label: &str,
    rounds: usize,
    iters_1w: usize,
    iters_mw: usize,
) {
    println!("\n--- Multi-Worker (1W / 2W / 4W, paired {rounds} rds, p50 wall ms [vs C50-Deep-A1]) — {label} ---");
    let warm_n = blocks.len().min(16);
    for &workers in &[1usize, 2, 4] {
        let mut res: Vec<Vec<f64>> = vec![Vec::with_capacity(rounds); NUM_ARMS];
        let iters = if workers == 1 { iters_1w } else { iters_mw };
        for _ in 0..rounds {
            for arm in 0..NUM_ARMS {
                let bar = Barrier::new(workers);
                let t_wall = std::thread::scope(|s| {
                    let mut handles = Vec::new();
                    for _ in 0..workers {
                        handles.push(s.spawn(|| {
                            let mut k = C51Kernel::default();
                            let mut out = Vec::new();
                            for blk in &blocks[..warm_n] {
                                run_one::<false, false>(arm, &mut k, blk, &mut out);
                            }
                            bar.wait();
                            let t0 = Instant::now();
                            for _ in 0..iters {
                                for blk in blocks {
                                    run_one::<false, false>(arm, &mut k, blk, &mut out);
                                    std::hint::black_box(&out);
                                }
                            }
                            t0.elapsed().as_secs_f64() * 1e3 / iters as f64
                        }));
                    }
                    let mut max_ms = 0.0f64;
                    for h in handles {
                        max_ms = max_ms.max(h.join().unwrap());
                    }
                    max_ms
                });
                res[arm].push(t_wall);
            }
        }
        let p50_s = |v: &[f64]| {
            let mut s = v.to_vec();
            s.sort_by(|a, b| a.total_cmp(b));
            s[s.len() / 2]
        };
        print!("  {workers}W: ");
        for arm in 0..NUM_ARMS {
            let m = p50_s(&res[arm]);
            if arm == 0 {
                print!("{}={:.2}ms ", ARM_NAMES[arm], m);
            } else {
                let mut pr0: Vec<f64> = (0..rounds).map(|r| res[arm][r] / res[0][r]).collect();
                pr0.sort_by(|a, b| a.total_cmp(b));
                print!("| {}={:.2}({:.3}x) ", ARM_NAMES[arm], m, pr0[rounds / 2]);
            }
        }
        println!();
    }
}

fn main() {
    let wiki6 = load_six_admitted();
    let nonwiki_path = resolve_path(&[
        "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/non_wikidata_blocks.bin",
    ]);
    let codedocs_path = resolve_path(&[
        "/tmp/mibz2_corpora/non_wikidata/code_docs_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/code_docs_blocks.bin",
    ]);
    let wiki431_path = resolve_path(&[
        "/tmp/mibz2_corpora/wikidata_431_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/wikidata_431_blocks.bin",
    ]);

    let nw_names = ["jsonl_0", "jsonl_1", "jsonl_2", "jsonl_3", "jsonl_4", "syslog_0"];
    let nonwiki6 = load_bin_blocks(&nonwiki_path, Some(&nw_names), false);
    let cd_names = ["code_docs_0", "code_docs_1", "code_docs_2", "code_docs_3"];
    let code4 = load_bin_blocks(&codedocs_path, Some(&cd_names), false);
    let wiki233 = load_bin_blocks(&wiki431_path, None, true);
    let wiki431 = load_bin_blocks(&wiki431_path, None, false);
    let low_m_stress = make_low_m_high_ns_stress();

    let nonwiki10: Vec<Block> = nonwiki6
        .iter()
        .chain(code4.iter())
        .map(|b| Block {
            name: b.name.clone(),
            n: b.n,
            p: b.p,
            records: b.records.clone(),
        })
        .collect();

    run_exactness_suite(&wiki6, &nonwiki10, &wiki431, &low_m_stress);
    if std::env::args().any(|a| a == "--exact-only") {
        return;
    }

    audit_edges_phase_and_rules(&wiki6, "6 Admitted Wikidata");
    audit_edges_phase_and_rules(&nonwiki6, "6 Real Non-Wikidata (JSONL + Syslog)");
    audit_edges_phase_and_rules(&code4, "4 Real Code & Docs");
    audit_edges_phase_and_rules(&wiki233, "233 Full wikidata.bz2");
    audit_edges_phase_and_rules(&low_m_stress, "6 Synthetic Low-M / High-NS & Fib Stress");

    audit_ws_all_arms(&wiki6, "6 Admitted Wikidata");
    audit_ws_all_arms(&nonwiki6, "6 Real Non-Wikidata (JSONL + Syslog)");
    audit_ws_all_arms(&code4, "4 Real Code & Docs");
    audit_ws_all_arms(&wiki431, "431 ALL wikidata.bz2");
    audit_ws_all_arms(&low_m_stress, "6 Synthetic Low-M / High-NS & Fib Stress");

    bench_stage_1t(&wiki6, "6 Admitted Wikidata", 15);
    bench_stage_1t(&nonwiki6, "6 Real Non-Wikidata (JSONL + Syslog)", 15);
    bench_stage_1t(&code4, "4 Real Code & Docs", 9);

    bench_multiworker(&wiki6, "6 Admitted Wikidata", 9, 4, 4);
    bench_multiworker(&nonwiki6, "6 Real Non-Wikidata (JSONL + Syslog)", 9, 6, 6);
    bench_multiworker(&code4, "4 Real Code & Docs", 5, 2, 2);
    bench_multiworker(&wiki233, "233 Full wikidata.bz2", 5, 1, 1);
    bench_multiworker(&wiki431, "431 ALL wikidata.bz2", 5, 1, 1);
}
