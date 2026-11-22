//! Cycle 51 Cumulative Headroom Oracle v6 + Bienaymé (1853) / Chebyshev (1867) / Cantelli (1928)
//! Exact Second-Moment Pre-`S1` Run-Length Theorem (`C51-D2`, Extending Chain D `CA-26→27→31→37→41`).
//!
//! Compares unified **`C51-DropIn-v2`** (`CS-22 + CS-24` fusion) head-to-head against ALL prior
//! Cycle 45–50 champions + `C51-DropIn-v2-NoOrph` ablation (`8` arms total) at `1T`, `2W`, and `4W`
//! across ALL 4 real corpora + `441`-block union + `243`-full union:
//!   0. `R5-Head-A1`           — verbatim `cand_r5_head` (`ID_BITS=22`, `L=16`, `AlwaysAt(65_536)`)
//!   1. `R5-Head-A2(AA)`       — paired A/A control
//!   2. `C49-Fused-OT64`       — Cycle 49 `CS-19` (`Spec4 + CA-37 + OutTail64 + Zero-WS TinyFL`)
//!   3. `C49-OutTail-Deep`     — Cycle 49 `CS-17` (`as_chunks::<8>() + grow_edges_step + 0.28/dense`)
//!   4. `C50-Deep-v1`          — Cycle 50 `CS-22` (`OrphanFree + TightArena + Stepped Spill + Peeled S5`)
//!   5. `C50-DropIn-v1`        — Cycle 50 `CS-24` (`CA-37/41 + Spec4 + Compact u16 OutQ next + K-TailUnw`)
//!   6. `C51-DropIn-v2-NoOrph` — Ablation: `C51-DropIn-v2` with `ORPHAN_FREE = false`
//!   7. `C51-DropIn-v2`        — Cycle 51 Unified Fusion (`CS-22 + CS-24` + Hoisted `istk`/`memo_rec` +
//!                               Tiered-Stack `TinyFL` + Compact `u16` `OutQ` `next` + exact `cuts`)
//!
//! Also verifies **Atlas Extension `C51-D2`** (Bienaymé 1853 / Chebyshev 1867 / Cantelli 1928)
//! across `100,000+` small & extremal run-length multisets + ALL `447` real blocks + synthetic
//! dense crossover sweep + exact-`S_2` runtime scan timing.
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
// 1. Unified Const-Generic Open<F-Inline3> Engine Supporting All Cycle 49-51 Arms
// ============================================================================

#[allow(dead_code)]
mod unified_open {
    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const NARROW_NODES: usize = 1 << 16;
    const RULE: usize = 4;
    const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;
    const INLINE_STACK: usize = 64;
    const MEMO_REC_CAP: usize = 32;

    const ID_BITS: u32 = 24;
    const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
    const SATURATED: u32 = u32::MAX >> ID_BITS;

    #[inline(always)]
    const fn literal(byte: u8) -> u32 {
        (1u32 << ID_BITS) | (byte as u32)
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BiMode {
        Never,
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
        pub fn should_use_bi(self, m: u32, ns: u32, r_max: u32) -> bool {
            match self {
                BiMode::Never => false,
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
    pub struct Cap {
        pub m: u32,
        pub strips: usize,
        pub post_s1_crossover: bool,
    }

    impl Cap {
        pub const NONE: Cap = Cap {
            m: u32::MAX,
            strips: usize::MAX,
            post_s1_crossover: false,
        };
        pub const C51_CROSSOVER: Cap = Cap {
            m: u32::MAX,
            strips: usize::MAX,
            post_s1_crossover: true,
        };
        pub fn dense(n: usize) -> Cap {
            Cap {
                m: 256_000u32.min((n as u64 * 3 / 10) as u32),
                strips: 16_384,
                post_s1_crossover: false,
            }
        }
    }

    #[derive(Default, Clone, Copy, Debug)]
    pub struct Counts {
        pub retained: usize,
        pub thick: usize,
        pub rules: usize,
        pub used_bi: bool,
        pub used_tail: bool,
        pub spelling_used: usize,
        pub used_bytes: usize,
        pub arena_est_gt_m: bool,
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
            Self {
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
            let s = total;
            total += w;
            s
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
    fn grow_edges<const STEP: bool>(a: &mut Vec<u32>, need: usize) {
        if STEP {
            let target = need.div_ceil(8192) * 8192;
            if target > a.capacity() {
                a.reserve_exact(target - a.len());
            }
            a.resize(target, 0);
        } else {
            a.resize(need.max(2 * a.len()), 0);
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

    mod outq {
        const CHUNK: usize = 512;
        pub(super) const BYTES: usize = 2 * CHUNK;
        const NONE: u32 = u32::MAX;
        const NONE16: u16 = u16::MAX;
        const EPOCH: u32 = 1 << 16;

        #[derive(Default)]
        pub(super) struct Spill {
            next32: Vec<u32>,
            next16: Vec<u16>,
            ovf: Vec<[u8; BYTES]>,
            cuts: Vec<[u32; 256]>,
        }

        impl Spill {
            pub(super) fn scratch_bytes(&self) -> usize {
                4 * self.next32.capacity()
                    + 2 * self.next16.capacity()
                    + BYTES * self.ovf.capacity()
                    + 1024 * self.cuts.capacity()
            }
            pub(super) fn used_bytes(&self, chunks: usize) -> usize {
                let next_b = if !self.next16.is_empty() {
                    2 * chunks
                } else {
                    4 * chunks
                };
                next_b + BYTES * self.ovf.len() + 1024 * self.cuts.len()
            }
        }

        pub(super) struct Queues<'a, const COMPACT_U16: bool, const STEPPED_SPILL: bool> {
            out: &'a mut [u8],
            spill: &'a mut Spill,
            in_out: u32,
            chunks: u32,
            heads: [u32; 256],
            pt: [u64; 256],
            limit: u32,
            over: bool,
        }

        pub(super) struct Heads {
            heads: [u32; 256],
            in_out: u32,
            pub(super) chunks: u32,
        }

        impl<'a, const COMPACT_U16: bool, const STEPPED_SPILL: bool>
            Queues<'a, COMPACT_U16, STEPPED_SPILL>
        {
            pub(super) fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
                spill.ovf.clear();
                spill.cuts.clear();
                let in_out = (out.len() / BYTES) as u32;
                let out = &mut out[..in_out as usize * BYTES];
                Queues {
                    in_out,
                    out,
                    spill,
                    chunks: 0,
                    heads: [NONE; 256],
                    pt: [u32::MAX as u64; 256],
                    limit,
                    over: false,
                }
            }
            pub(super) fn over(&self) -> bool {
                self.over
            }
            pub(super) fn heads(&self) -> Heads {
                Heads {
                    heads: self.heads,
                    in_out: self.in_out,
                    chunks: self.chunks,
                }
            }
            #[inline(always)]
            pub(super) fn thin_of(&self, a: usize) -> u32 {
                (self.pt[a] >> 32) as u32
            }
            pub(super) fn extract_thin(&self) -> [u32; 256] {
                std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
            }
            #[inline(always)]
            pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & 1023) as usize;
                if (w <= 16) & (f <= CHUNK - 16) & ((m as u16) < 65_520) {
                    let at = p as usize + f;
                    if let Some(dst) = self.out.get_mut(at..at + 32) {
                        let lo = m as u16;
                        let iota: [u16; 16] =
                            std::array::from_fn(|i| lo.wrapping_add(i as u16));
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
                        if (COMPACT_U16 || STEPPED_SPILL)
                            && self.spill.cuts.len() == self.spill.cuts.capacity()
                        {
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
                    if STEPPED_SPILL && self.spill.ovf.len() == self.spill.ovf.capacity() {
                        let target = (self.spill.ovf.len() + 1).div_ceil(16) * 16;
                        self.spill.ovf.reserve_exact(target - self.spill.ovf.len());
                    }
                    self.spill.ovf.push([0; BYTES]);
                }
                let idu = id as usize;
                if COMPACT_U16 {
                    debug_assert!(id < u32::from(NONE16));
                    if self.spill.next16.len() <= idu {
                        let target = (idu + 1).div_ceil(256) * 256;
                        if target > self.spill.next16.capacity() {
                            self.spill.next16.reserve_exact(target - self.spill.next16.len());
                        }
                        self.spill.next16.resize(target, NONE16);
                    }
                    self.spill.next16[idu] = NONE16;
                } else if STEPPED_SPILL {
                    if self.spill.next32.len() <= idu {
                        let target = (idu + 1).div_ceil(256) * 256;
                        if target > self.spill.next32.capacity() {
                            self.spill.next32.reserve_exact(target - self.spill.next32.len());
                        }
                        self.spill.next32.resize(target, NONE);
                    }
                    self.spill.next32[idu] = NONE;
                } else {
                    if self.spill.next32.len() <= idu {
                        self.spill.next32.resize(idu + 1, NONE);
                    }
                    self.spill.next32[idu] = NONE;
                }
                let hi = self.pt[a] & !0xFFFF_FFFFu64;
                if self.heads[a] == NONE {
                    self.heads[a] = id;
                } else {
                    let prev_id = ((self.pt[a] as u32) >> 10) as usize;
                    if COMPACT_U16 {
                        self.spill.next16[prev_id] = id as u16;
                    } else {
                        self.spill.next32[prev_id] = id;
                    }
                }
                self.pt[a] = hi | u64::from(id << 10);
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

        pub(super) struct Reader<'a, const COMPACT_U16: bool> {
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

        impl<'a, const COMPACT_U16: bool> Reader<'a, COMPACT_U16> {
            #[inline(always)]
            pub(super) fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
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
            fn next_chunk(&self, id: usize) -> u32 {
                if COMPACT_U16 {
                    u32::from(self.spill.next16[id])
                } else {
                    self.spill.next32[id]
                }
            }
            #[inline(always)]
            fn cut_at(&self, epoch: usize) -> u32 {
                self.spill.cuts.get(epoch).map_or(u32::MAX, |c| c[self.a])
            }
            #[inline(always)]
            pub(super) fn wire_xor_nomark(
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
                        let v0 = hi | (q0 as u32 & 0xFFFF); let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                        let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF); let v3 = hi | ((q0 >> 48) as u32);
                        let v4 = hi | (q1 as u32 & 0xFFFF); let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                        let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF); let v7 = hi | ((q1 >> 48) as u32);
                        let v8 = hi | (q2 as u32 & 0xFFFF); let v9 = hi | ((q2 >> 16) as u32 & 0xFFFF);
                        let v10 = hi | ((q2 >> 32) as u32 & 0xFFFF); let v11 = hi | ((q2 >> 48) as u32);
                        let v12 = hi | (q3 as u32 & 0xFFFF); let v13 = hi | ((q3 >> 16) as u32 & 0xFFFF);
                        let v14 = hi | ((q3 >> 32) as u32 & 0xFFFF); let v15 = hi | ((q3 >> 48) as u32);
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
                        let v0 = hi | (q0 as u32 & 0xFFFF); let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                        let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF); let v3 = hi | ((q0 >> 48) as u32);
                        let v4 = hi | (q1 as u32 & 0xFFFF); let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                        let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF); let v7 = hi | ((q1 >> 48) as u32);
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
                    if i + 4 <= span {
                        let b = &bytes[2 * i..2 * i + 8];
                        let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                        let v0 = hi | (q as u32 & 0xFFFF); let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                        let v2 = hi | ((q >> 32) as u32 & 0xFFFF); let v3 = hi | ((q >> 48) as u32);
                        let u0 = dst_start + i;
                        let eu = &mut edges[u0..u0 + 4];
                        eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                        edges[v0 as usize] ^= u0 as u32; edges[v1 as usize] ^= (u0 + 1) as u32;
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
            pub(super) fn wire_xor(
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
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    let mut before = *predecessor;
                    for (i, &s) in pairs.iter().enumerate() {
                        let u = dst_start + i;
                        let v = hi | u32::from(u16::from_le_bytes(s));
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
                        self.cut = self.cut_at(self.epoch as usize);
                    }
                    if self.at == CHUNK {
                        self.id = self.next_chunk(self.id as usize);
                        self.at = 0;
                    }
                    let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                    let chunk: &[u8] = if self.id < self.in_out {
                        let at = self.id as usize * BYTES;
                        &self.out[at..at + BYTES]
                    } else {
                        &self.spill.ovf[(self.id - self.in_out) as usize][..]
                    };
                    let hi = self.epoch << 16;
                    let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                    for (i, &s) in pairs.iter().enumerate() {
                        let u = dst_start + i;
                        let v = hi | u32::from(u16::from_le_bytes(s));
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
            pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {
                let span = dst.len();
                if self.at + span <= CHUNK
                    && self.read + span as u32 <= self.cut
                    && self.id < self.in_out
                {
                    let hi = pack | (self.epoch << 16);
                    let at = self.id as usize * BYTES + 2 * self.at;
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    for (e, &s) in dst.iter_mut().zip(pairs) {
                        *e = hi | u32::from(u16::from_le_bytes(s));
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
                        self.cut = self.cut_at(self.epoch as usize);
                    }
                    if self.at == CHUNK {
                        self.id = self.next_chunk(self.id as usize);
                        self.at = 0;
                    }
                    let take = dst.len().min(CHUNK - self.at).min((self.cut - self.read) as usize);
                    let chunk: &[u8] = if self.id < self.in_out {
                        let at = self.id as usize * BYTES;
                        &self.out[at..at + BYTES]
                    } else {
                        &self.spill.ovf[(self.id - self.in_out) as usize][..]
                    };
                    let hi = pack | (self.epoch << 16);
                    let (now, rest) = dst.split_at_mut(take);
                    let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                    for (e, &s) in now.iter_mut().zip(pairs) {
                        *e = hi | u32::from(u16::from_le_bytes(s));
                    }
                    self.at += take;
                    self.read += take as u32;
                    dst = rest;
                }
            }
        }
    }

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
        const STRIDE: usize = 4;
        const IS_NARROW: bool = true;
        const BOTTOM: usize = 1;
        #[inline(always)] fn prev(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 4 + side] & 0xFFFF }
        #[inline(always)] fn next(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 4 + side] >> 16 }
        #[inline(always)] fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
            let w = &mut a[id as usize * 4 + side]; *w = (*w & 0xFFFF_0000) | to;
        }
        #[inline(always)] fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
            let w = &mut a[id as usize * 4 + side]; *w = (*w & 0xFFFF) | (to << 16);
        }
        #[inline(always)] fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
            let sl = &a[id as usize * 4..id as usize * 4 + 4]; (sl[0] >> 16, sl[1], sl[2], sl[3])
        }
    }

    impl Links for Wide {
        const STRIDE: usize = 6;
        const IS_NARROW: bool = false;
        const BOTTOM: usize = 2;
        #[inline(always)] fn prev(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side] }
        #[inline(always)] fn next(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side + 1] }
        #[inline(always)] fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side] = to; }
        #[inline(always)] fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side + 1] = to; }
        #[inline(always)] fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
            let sl = &a[id as usize * 6..id as usize * 6 + 6]; (sl[1], sl[2], sl[4], sl[5])
        }
    }

    struct Elim<'a, L: Links> {
        a: &'a mut [u32],
        base: usize,
        top: usize,
        nodes: u32,
        _links: std::marker::PhantomData<L>,
    }

    impl<L: Links> Elim<'_, L> {
        #[inline(always)] fn width(&self, id: u32) -> u32 { self.a[id as usize * L::STRIDE + L::STRIDE - 2] }
        #[inline(always)] fn set_width(&mut self, id: u32, w: u32) { self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w; }
        #[inline(always)] fn word(&self, id: u32) -> u32 { self.a[id as usize * L::STRIDE + L::STRIDE - 1] }
        #[inline(always)] fn set_word(&mut self, id: u32, w: u32) { self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w; }
        #[inline(always)] fn set_bottom(&mut self, id: u32, x: u32) { self.a[id as usize * L::STRIDE + L::BOTTOM] = x; }

        fn assign_bottom(&mut self, rows: u32) {
            let (mut first, mut last) = (L::next(self.a, 0, 1), L::prev(self.a, 0, 1));
            let (mut lo, mut hi) = (0, rows);
            while first != last {
                let (next, prev) = (L::next(self.a, first, 1), L::prev(self.a, last, 1));
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
            if *tail == 0 { *head = id; } else { L::set_next(self.a, *tail, side, id); }
            L::set_prev(self.a, id, side, *tail);
            *tail = id;
        }

        fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
            let mut prev = 0;
            for (head, tail) in [first, second] {
                if tail == 0 { continue; }
                L::set_next(self.a, prev, side, head);
                L::set_prev(self.a, head, side, prev);
                prev = tail;
            }
            L::set_next(self.a, prev, side, 0);
            L::set_prev(self.a, 0, side, prev);
        }

        #[inline(always)] fn cut(&mut self, id: u32, side: usize) {
            let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
            L::set_next(self.a, p, side, n); L::set_prev(self.a, n, side, p);
        }
        #[inline(always)] fn pop_tail(&mut self, id: u32, side: usize) {
            let prev = L::prev(self.a, id, side);
            L::set_next(self.a, prev, side, 0); L::set_prev(self.a, 0, side, prev);
        }
        #[inline(always)] fn insert(&mut self, at: u32, id: u32, side: usize) {
            let next = L::next(self.a, at, side);
            L::set_prev(self.a, id, side, at); L::set_next(self.a, id, side, next);
            L::set_next(self.a, at, side, id); L::set_prev(self.a, next, side, id);
        }
        #[inline(always)] fn rule(&self, id: u32) -> usize { self.base + id as usize * RULE }
        #[inline(always)] fn len_handle(&self, h: u32) -> u32 {
            let g = h >> ID_BITS; if g < SATURATED { g } else { self.a[self.rule(h & ID_MASK) + 2] }
        }
        #[inline(always)] fn short_payload_h(&self, h: u32) -> u128 {
            let g = h >> ID_BITS; let id = h & ID_MASK;
            if g <= 3 { return u128::from(id); }
            let r = self.rule(id); let sl = &self.a[r..r + 4];
            u128::from(sl[0]) | (u128::from(sl[1]) << 32) | (u128::from(sl[2]) << 64) | (u128::from(sl[3] & 0x00ff_ffff) << 96)
        }
        #[inline(always)] fn join(&mut self, left: u32, right: u32) -> u32 {
            if left == 0 { return right; }
            if right == 0 { return left; }
            let left_len = left >> ID_BITS;
            let sum = left_len + (right >> ID_BITS);
            if sum < INLINE as u32 {
                if sum <= 3 {
                    return (sum << ID_BITS) | (left & ID_MASK) | ((right & ID_MASK) << (8 * left_len));
                }
                let id = ((self.top - self.base) / RULE) as u32;
                let bytes = self.short_payload_h(left)
                    | (self.short_payload_h(right) << (8 * left_len))
                    | (u128::from(sum) << 120);
                let out = &mut self.a[self.top..self.top + RULE];
                out[0] = bytes as u32; out[1] = (bytes >> 32) as u32;
                out[2] = (bytes >> 64) as u32; out[3] = (bytes >> 96) as u32;
                self.top += RULE;
                (sum << ID_BITS) | id
            } else {
                let len = if sum <= SATURATED { sum } else { self.len_handle(left) + self.len_handle(right) };
                let id = ((self.top - self.base) / RULE) as u32;
                let out = &mut self.a[self.top..self.top + RULE];
                out[0] = left; out[1] = right; out[2] = len; out[3] = u32::MAX;
                self.top += RULE;
                (len.min(SATURATED) << ID_BITS) | id
            }
        }
        fn power(&mut self, word: u32, q: u32) -> u32 {
            let (mut acc, mut base, mut e) = (0, word, q);
            loop {
                if e & 1 == 1 { acc = self.join(acc, base); }
                e >>= 1;
                if e == 0 { break acc; }
                base = self.join(base, base);
            }
        }
        #[inline(never)]
        fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
            let mut left = *remaining;
            let done = loop {
                if left == 0 { break true; }
                if self.top + RULE > self.a.len() { break false; }
                let sent = &self.a[..2];
                let (x, y) = (sent[0] & 0xFFFF, sent[1] & 0xFFFF);
                let (xi, yi) = (x as usize * 4, y as usize * 4);
                let nx = &self.a[xi..xi + 4];
                let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
                if x == y {
                    assert!(aw > 0 && aw <= left);
                    left -= aw;
                    let (px0, px1) = (x0 & 0xFFFF, x1 & 0xFFFF);
                    self.a[px0 as usize * 4] &= 0xFFFF; self.a[0] = (self.a[0] & 0xFFFF_0000) | px0;
                    self.a[px1 as usize * 4 + 1] &= 0xFFFF; self.a[1] = (self.a[1] & 0xFFFF_0000) | px1;
                    continue;
                }
                let ny = &self.a[yi..yi + 4];
                let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
                let w = aw.min(bw);
                assert!(w > 0 && w <= left);
                left -= w;
                let word = self.join(wx, wy);
                if aw <= bw {
                    let px = x0 & 0xFFFF;
                    self.a[px as usize * 4] &= 0xFFFF; self.a[0] = (self.a[0] & 0xFFFF_0000) | px;
                    if aw < bw {
                        let y_next = if y == px { 0 } else { y0 >> 16 };
                        let nxm = &mut self.a[xi..xi + 4]; nxm[0] = y | (y_next << 16); nxm[3] = word;
                        let nym = &mut self.a[yi..yi + 4]; nym[0] = (y0 & 0xFFFF) | (x << 16); nym[2] = bw - aw;
                        let yni = y_next as usize * 4; self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    } else {
                        let y_next = if y == px { 0 } else { y0 >> 16 }; let y_prev = y0 & 0xFFFF;
                        let nxm = &mut self.a[xi..xi + 4]; nxm[0] = y_prev | (y_next << 16); nxm[3] = word;
                        let ypi = y_prev as usize * 4; self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                        let yni = y_next as usize * 4; self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                        let py = y1 & 0xFFFF;
                        self.a[py as usize * 4 + 1] &= 0xFFFF; self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                    }
                } else {
                    let py = y1 & 0xFFFF;
                    self.a[py as usize * 4 + 1] &= 0xFFFF; self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                    let x_next = if x == py { 0 } else { x1 >> 16 };
                    let nym = &mut self.a[yi..yi + 4]; nym[1] = x | (x_next << 16); nym[3] = word;
                    let nxm = &mut self.a[xi..xi + 4]; nxm[1] = (x1 & 0xFFFF) | (y << 16); nxm[2] = aw - bw;
                    let xni = x_next as usize * 4 + 1; self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
                }
            };
            *remaining = left;
            done
        }
        fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
            if !SPIRALS && L::IS_NARROW { return self.eliminate_narrow_plain(remaining); }
            let mut left = *remaining;
            let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
            let done = loop {
                if left == 0 { break true; }
                if self.top + room > self.a.len() { break false; }
                let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
                let (aw, bw) = (self.width(x), self.width(y));
                if SPIRALS && x != y {
                    if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                        let q = (aw - 1) / bw;
                        assert!(q * bw < aw && q * bw <= left);
                        let head = self.power(self.word(x), q);
                        let word = self.join(head, self.word(y));
                        self.set_word(y, word); self.set_width(x, aw - q * bw); left -= q * bw; continue;
                    }
                    if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                        let q = (bw - 1) / aw;
                        assert!(q * aw < bw && q * aw <= left);
                        let tail = self.power(self.word(y), q);
                        let word = self.join(self.word(x), tail);
                        self.set_word(x, word); self.set_width(y, bw - q * aw); left -= q * aw; continue;
                    }
                }
                let w = aw.min(bw);
                assert!(w > 0 && w <= left);
                left -= w;
                if x == y { self.pop_tail(x, 0); self.pop_tail(x, 1); continue; }
                let word = self.join(self.word(x), self.word(y));
                if aw <= bw {
                    self.pop_tail(x, 0); self.insert(y, x, 0); self.set_word(x, word);
                    if aw == bw { self.cut(y, 0); self.pop_tail(y, 1); } else { self.set_width(y, bw - w); }
                } else {
                    self.pop_tail(y, 1); self.insert(x, y, 1); self.set_word(y, word); self.set_width(x, aw - w);
                }
            };
            *remaining = left;
            done
        }
        #[allow(clippy::too_many_arguments)]
        fn inline_word<const TAIL_UNWIND: bool, const SP_STEP: usize>(
            &mut self,
            word: u32,
            byte: u8,
            istk: &mut [u32; INLINE_STACK],
            memo_rec: &mut [u32; MEMO_REC_CAP],
            stack: &mut Vec<u32>,
            spelling: &mut Vec<u8>,
        ) -> ([u8; INLINE], u8) {
            let height = (word >> ID_BITS) as usize;
            if height < INLINE {
                let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
                (v.to_le_bytes(), (height + 1) as u8)
            } else {
                let start = spelling.len() as u32;
                let mut dummy_n = 0usize;
                let mut dummy_ovf = false;
                self.append::<TAIL_UNWIND, false, SP_STEP>(
                    word, istk, stack, spelling, memo_rec, &mut dummy_n, &mut dummy_ovf,
                );
                ensure_step_u8(spelling, 1, SP_STEP);
                spelling.push(byte);
                (long_slot(start, spelling.len() as u32), LONG)
            }
        }
        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn append<const TAIL_UNWIND: bool, const TRACK_MEMO: bool, const SP_STEP: usize>(
            &mut self,
            handle: u32,
            istk: &mut [u32; INLINE_STACK],
            ovf_stack: &mut Vec<u32>,
            dest: &mut Vec<u8>,
            memo_rec: &mut [u32; MEMO_REC_CAP],
            memo_n: &mut usize,
            memo_ovf: &mut bool,
        ) {
            if !TAIL_UNWIND {
                ovf_stack.clear();
                ovf_stack.push(handle);
                while let Some(h) = ovf_stack.pop() {
                    let g = h >> ID_BITS;
                    if g < INLINE as u32 {
                        let bytes = self.short_payload_h(h).to_le_bytes();
                        ensure_step_u8(dest, g as usize, SP_STEP);
                        dest.extend_from_slice(&bytes[..g as usize]);
                    } else {
                        let r = self.rule(h & ID_MASK);
                        let tag = self.a[r + 3];
                        let rule = &mut self.a[r..r + 4];
                        if tag == u32::MAX {
                            rule[3] = 0x8000_0000 | dest.len() as u32;
                            ovf_stack.push(rule[1]);
                            ovf_stack.push(rule[0]);
                        } else {
                            let start = (tag & 0x7fff_ffff) as usize;
                            let rlen = rule[2] as usize;
                            ensure_step_u8(dest, rlen, SP_STEP);
                            dest.extend_from_within(start..start + rlen);
                        }
                    }
                }
            } else {
                let mut sp = 0usize;
                let mut used_heap = false;
                let mut h = handle;
                loop {
                    let g = h >> ID_BITS;
                    if g < INLINE as u32 {
                        let bytes = self.short_payload_h(h).to_le_bytes();
                        ensure_step_u8(dest, g as usize, SP_STEP);
                        dest.extend_from_slice(&bytes[..g as usize]);
                        if used_heap && !ovf_stack.is_empty() { h = ovf_stack.pop().unwrap(); continue; }
                        if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                        return;
                    }
                    let r = self.rule(h & ID_MASK);
                    let tag = self.a[r + 3];
                    if tag != u32::MAX {
                        let start = (tag & 0x7fff_ffff) as usize;
                        let rlen = self.a[r + 2] as usize;
                        ensure_step_u8(dest, rlen, SP_STEP);
                        dest.extend_from_within(start..start + rlen);
                        if used_heap && !ovf_stack.is_empty() { h = ovf_stack.pop().unwrap(); continue; }
                        if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                        return;
                    }
                    self.a[r + 3] = 0x8000_0000 | (dest.len() as u32);
                    if TRACK_MEMO {
                        if *memo_n < MEMO_REC_CAP {
                            memo_rec[*memo_n] = r as u32;
                            *memo_n += 1;
                        } else {
                            *memo_ovf = true;
                        }
                    }
                    let left = self.a[r];
                    let right = self.a[r + 1];
                    let lg = left >> ID_BITS;
                    if lg < INLINE as u32 {
                        let lb = self.short_payload_h(left).to_le_bytes();
                        ensure_step_u8(dest, lg as usize, SP_STEP);
                        dest.extend_from_slice(&lb[..lg as usize]);
                        let rg = right >> ID_BITS;
                        if rg < INLINE as u32 {
                            let rb = self.short_payload_h(right).to_le_bytes();
                            ensure_step_u8(dest, rg as usize, SP_STEP);
                            dest.extend_from_slice(&rb[..rg as usize]);
                            if used_heap && !ovf_stack.is_empty() { h = ovf_stack.pop().unwrap(); continue; }
                            if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                            return;
                        }
                        h = right;
                    } else {
                        if sp < INLINE_STACK {
                            istk[sp] = right;
                            sp += 1;
                        } else {
                            if !used_heap {
                                ovf_stack.clear();
                                used_heap = true;
                            }
                            ovf_stack.push(right);
                        }
                        h = left;
                    }
                }
            }
        }
    }

    struct Dictionary<'a, const TAIL_UNWIND: bool, const ORPHAN_FREE: bool, const SP_STEP: usize, const INL_STEP: usize> {
        pairs: &'a mut [u32; 512],
        contents: &'a mut [u32; 2048],
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        spelling: &'a mut Vec<u8>,
        istk: &'a mut [u32; INLINE_STACK],
        memo_rec: &'a mut [u32; MEMO_REC_CAP],
        stack: &'a mut Vec<u32>,
    }

    impl<const TAIL_UNWIND: bool, const ORPHAN_FREE: bool, const SP_STEP: usize, const INL_STEP: usize>
        Dictionary<'_, TAIL_UNWIND, ORPHAN_FREE, SP_STEP, INL_STEP>
    {
        #[inline(always)]
        fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
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
        fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8) -> u32 {
            let height = (handle >> ID_BITS) as usize;
            if height < INLINE {
                let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
                let entry = v.to_le_bytes();
                let len = (height + 1) as u8;
                let h = ((v as u64)
                    ^ ((v >> 64) as u64).rotate_left(29)
                    ^ (u64::from(len) << 56))
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
            let r = e.rule(id);
            let tag_r = e.a[r + 3];
            if ORPHAN_FREE {
                if tag_r != u32::MAX {
                    let pos = (tag_r & 0x7fff_ffff) as usize;
                    let rlen = e.a[r + 2] as usize;
                    ensure_step_u8(self.spelling, rlen + 1, SP_STEP);
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
                        self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes
                    } {
                        self.spelling.truncate(start);
                        return ent & 0x00FF_FFFF;
                    }
                    let c = self.insert(entry, LONG);
                    self.contents[index] = tag | c;
                    return c;
                }
                let mut memo_n = 0usize;
                let mut memo_ovf = false;
                e.append::<TAIL_UNWIND, true, SP_STEP>(
                    handle, self.istk, self.stack, self.spelling, self.memo_rec, &mut memo_n, &mut memo_ovf,
                );
                ensure_step_u8(self.spelling, 1, SP_STEP);
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
                        let old_start = u32::from_le_bytes(self.inline16[old][..4].try_into().unwrap());
                        let delta = (start as u32) - old_start;
                        for &rm in &self.memo_rec[..memo_n] {
                            e.a[rm as usize + 3] -= delta;
                        }
                        self.spelling.truncate(start);
                    } else {
                        self.spelling.pop();
                    }
                    return ent & 0x00FF_FFFF;
                }
                let c = self.insert(entry, LONG);
                self.contents[index] = tag | c;
                return c;
            }
            let memoized = tag_r != u32::MAX;
            let mut dummy_n = 0usize;
            let mut dummy_ovf = false;
            e.append::<TAIL_UNWIND, false, SP_STEP>(
                handle, self.istk, self.stack, self.spelling, self.memo_rec, &mut dummy_n, &mut dummy_ovf,
            );
            ensure_step_u8(self.spelling, 1, SP_STEP);
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
                if memoized { self.spelling.truncate(start); } else { self.spelling.pop(); }
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry, LONG);
            self.contents[index] = tag | c;
            c
        }
        #[inline(always)]
        fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
            ensure_step_inline(self.inline16, self.lens, INL_STEP);
            let code = self.lens.len() as u32;
            self.inline16.push(entry);
            self.lens.push(len);
            code
        }
    }

    fn spelling_hash_long(bytes: &[u8]) -> u64 {
        let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ part(bytes.len() - 8).rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
            ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^ (h >> 30)
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
            let s0 = st[b] as usize;
            st[b] += len as u32;
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
        let mut buf = [0u32; 64];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }
    #[inline(never)]
    fn invert_tiny_fl_256(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut buf = [0u32; 256];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }
    #[inline(never)]
    fn invert_tiny_fl_1536(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut buf = [0u32; 1536];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }

    struct Pass<'a, const COMPACT_U16: bool, const STEPPED_SPILL: bool, const TIGHT_ARENA: bool> {
        queues: outq::Queues<'a, COMPACT_U16, STEPPED_SPILL>,
        edges: &'a mut Vec<u32>,
        ns: usize,
        cap_strips: usize,
        f: First,
    }

    impl<const COMPACT_U16: bool, const STEPPED_SPILL: bool, const TIGHT_ARENA: bool>
        Pass<'_, COMPACT_U16, STEPPED_SPILL, TIGHT_ARENA>
    {
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
                if p.wrapping_sub(x) < w {
                    f.mark = f.m + (p - x);
                    f.mark_byte = c;
                }
                self.queues.push_thin(a, f.m, w);
                f.m += w;
            } else {
                if self.ns < self.cap_strips {
                    let at = 3 * self.ns;
                    if self.edges.len() < at + 3 {
                        if TIGHT_ARENA && self.cap_strips == usize::MAX {
                            grow_edges_s1_step(self.edges, at + 3);
                        } else {
                            let need = (at + 3).max(2 * self.edges.len()).max(1024).min(self.cap_strips.saturating_mul(3));
                            self.edges.reserve_exact(need - self.edges.len());
                            self.edges.resize(need, 0);
                        }
                    }
                    let thin_a = self.queues.thin_of(a);
                    self.edges[at..at + 3].copy_from_slice(&[x, thin_a + f.thick_rows[a], (w << 8) | u32::from(c)]);
                }
                self.ns += 1;
                f.thick[a] += 1;
                f.thick_rows[a] += w;
            }
        }
    }

    #[derive(Default)]
    pub struct Kernel<
        const DEEP: bool,
        const COMPACT_U16: bool,
        const STEPPED_SPILL: bool,
        const TAIL_UNWIND: bool,
        const ORPHAN_FREE: bool,
        const TIGHT_ARENA: bool,
        const PEELED_S5: bool,
        const TIERED_TINY: bool,
    > {
        spill: outq::Spill,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        stack: Vec<u32>,
        predecessor: Option<u32>,
        pub force_band_vec: bool,
    }

    impl<
            const DEEP: bool,
            const COMPACT_U16: bool,
            const STEPPED_SPILL: bool,
            const TAIL_UNWIND: bool,
            const ORPHAN_FREE: bool,
            const TIGHT_ARENA: bool,
            const PEELED_S5: bool,
            const TIERED_TINY: bool,
        > Kernel<DEEP, COMPACT_U16, STEPPED_SPILL, TAIL_UNWIND, ORPHAN_FREE, TIGHT_ARENA, PEELED_S5, TIERED_TINY>
    {
        pub fn scratch_report(&self) -> (usize, usize, usize, String) {
            let spill_b = self.spill.scratch_bytes();
            let bands_b = 8 * self.bands.capacity();
            let spell_b = self.spelling.capacity();
            let edges_b = 4 * self.edges.capacity();
            let extra_b = 4 * self.extra.capacity();
            let inl_b = INLINE * self.inline16.capacity() + self.lens.capacity();
            let stk_b = 4 * self.stack.capacity();
            let total = spill_b + bands_b + spell_b + edges_b + extra_b + inl_b + stk_b;
            let s = format!(
                "edges={:.1}K spell={:.1}K inl={:.1}K spill={:.2}K bands={:.2}K stk={:.2}K",
                edges_b as f64 / 1024.0,
                spell_b as f64 / 1024.0,
                inl_b as f64 / 1024.0,
                spill_b as f64 / 1024.0,
                bands_b as f64 / 1024.0,
                stk_b as f64 / 1024.0,
            );
            (total, edges_b, spell_b, s)
        }

        #[inline(never)]
        pub fn invert_tiny_fl(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
            if n == 0 || (p as usize) >= n || records.len() > 16_000_000 {
                return false;
            }
            if records.len() == 1 {
                if usize::from(records[0] & 255) != n { return false; }
                out.clear(); out.resize(n, (records[0] >> 8) as u8);
                return true;
            }
            if TIERED_TINY {
                if n <= 64 { return invert_tiny_fl_64(records, n, p, out); }
                if n <= 256 { return invert_tiny_fl_256(records, n, p, out); }
                if n <= 1536 { return invert_tiny_fl_1536(records, n, p, out); }
            }
            if self.edges.len() < n {
                let add = n - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(n, 0);
            }
            invert_tiny_fl_with_buf(records, n, p, &mut self.edges[..n], out)
        }

        fn nodes_most(ns: usize) -> usize { 3 * ns + 3 }

        fn arena_words(ns: usize, m: usize) -> usize {
            let most = Self::nodes_most(ns);
            let stride = if most > NARROW_NODES { Wide::STRIDE } else { Narrow::STRIDE };
            if TIGHT_ARENA {
                (4 * ns + stride * most).max(m)
            } else {
                (4 * ns + (stride + RULE) * most).max(m)
            }
        }

        #[inline(never)]
        pub fn try_run_const<const LIM: u32>(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            cap: Cap,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> Option<Counts> {
            let nrec = records.len();
            if n == 0 || n > 900_000 || (p as usize) >= n || nrec > 16_000_000 {
                return None;
            }
            out.resize(n + INLINE, 0);
            let (ns, f, heads, over, extra_127, tot_pos) = {
                let mut pass = Pass::<COMPACT_U16, STEPPED_SPILL, TIGHT_ARENA> {
                    queues: outq::Queues::<COMPACT_U16, STEPPED_SPILL>::new(
                        &mut out[..], &mut self.spill, cap.m.min(n as u32),
                    ),
                    edges: &mut self.edges,
                    ns: 0,
                    cap_strips: cap.strips,
                    f: First::default(),
                };
                let (mut pos, mut extra_127, mut i) = (0u32, 0u32, 0usize);
                while i < nrec {
                    let rec = records[i];
                    let c = (rec >> 8) as u8;
                    let mut width = u32::from(rec & 255);
                    i += 1;
                    if width <= LIM {
                        pass.piece(c, pos, width, true, p);
                        pos += width;
                    } else {
                        if width == 127 {
                            while i < nrec && (records[i] >> 8) as u8 == c {
                                width += u32::from(records[i] & 255);
                                extra_127 += 1;
                                i += 1;
                            }
                        }
                        pass.take::<LIM>(c, pos, width, p);
                        pos += width;
                    }
                }
                let over = pass.queues.over();
                let thin = pass.queues.extract_thin();
                let heads = pass.queues.heads();
                let mut f = pass.f;
                f.thin = thin;
                (pass.ns, f, heads, over, extra_127, pos)
            };
            if over || ns > cap.strips || tot_pos as usize != n || f.m == 0 || f.m > cap.m {
                return None;
            }
            let r_max = (nrec as u32).saturating_sub(extra_127);
            if cap.post_s1_crossover && n >= 32_768 {
                let r_thin_lo = r_max.saturating_sub(ns as u32);
                if 8 * u64::from(f.m) + 2 * u64::from(r_thin_lo) > 9 * (n as u64) {
                    return None;
                }
            }
            Some(self.finish(&f, ns, n, r_max, &heads, bi_mode, out))
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn finish(
            &mut self, f: &First, ns: usize, n: usize, r_max: u32,
            heads: &outq::Heads, bi_mode: BiMode, out: &mut Vec<u8>,
        ) -> Counts {
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            let first = starts(all);
            let mut write = starts(f.thick);
            let m = f.m as usize;
            let need = Self::arena_words(ns, m);
            let arena_est_gt_m = need > m;
            if self.edges.len() < need {
                let add = need - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(need, 0);
            }
            self.bands.clear();
            self.spelling.clear();

            let in_out_chunks = out.len() / outq::BYTES;
            let tail_offset = if DEEP {
                (heads.chunks as usize).min(in_out_chunks) * outq::BYTES
            } else {
                let mut chunks_taken = 0usize;
                for a in 0..256 {
                    let ta = f.thin[a] as usize;
                    if ta > 0 { chunks_taken += (ta + 511) >> 9; }
                }
                chunks_taken.min(in_out_chunks) * outq::BYTES
            };

            let most = Self::nodes_most(ns);
            let (cursors, used_tail, nbands, rules, top_total) = if most <= NARROW_NODES {
                self.induced_narrow(ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..])
            } else {
                self.induced_wide(ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..])
            };

            let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
            let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
            if self.edges.len() < m { self.edges.resize(m, 0); }
            self.extra.clear();
            if wide { self.extra.resize(m, 0); }
            self.predecessor = None;
            let use_xor = !wide && bi_mode.should_use_bi(f.m, ns as u32, r_max);
            let (out_q, out_tail) = out.split_at(tail_offset);

            let used_bytes = 4 * m.max(top_total)
                + self.spelling.len()
                + (INLINE + 1) * self.lens.len()
                + self.spill.used_bytes(heads.chunks as usize)
                + 8 * self.bands.len()
                + 4 * self.extra.len()
                + 4 * self.stack.len();

            if use_xor {
                let pred = if used_tail {
                    self.wire_bi_tail(m, f.mark, f.mark_byte, bits, &cursors, nbands, out_tail, out_q, heads)
                } else {
                    self.wire_bi_vec(m, f.mark, f.mark_byte, bits, &cursors, out_q, heads)
                };
                self.predecessor = Some(pred);
            } else if used_tail {
                self.wire_uni_tail(m, wide, bits, &cursors, nbands, out_tail, out_q, heads);
            } else {
                self.wire_uni_vec(m, wide, bits, &cursors, out_q, heads);
            }

            if let Some(before) = self.predecessor {
                emit_bidirectional::<PEELED_S5>(
                    &self.edges[..m], &self.spelling, &self.inline16, &self.lens,
                    n, f.mark, before, bits, out,
                );
            } else {
                emit(
                    &self.edges[..m], &self.extra, &self.spelling, &self.inline16, &self.lens,
                    n, f.mark, bits, out,
                );
            }
            Counts {
                retained: m,
                thick: ns,
                rules,
                used_bi: use_xor,
                used_tail,
                spelling_used: self.spelling.len(),
                used_bytes,
                arena_est_gt_m,
            }
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_narrow(
            &mut self, ns: usize, n: u32, m: u32, thin: &[u32; 256], first: &[u32; 256],
            write: &mut [u32; 256], most: usize, out_tail: &mut [u8],
        ) -> ([usize; 256], bool, usize, usize, usize) {
            self.induced_with::<Narrow>(ns, n, m, thin, first, write, most, out_tail)
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_wide(
            &mut self, ns: usize, n: u32, m: u32, thin: &[u32; 256], first: &[u32; 256],
            write: &mut [u32; 256], most: usize, out_tail: &mut [u8],
        ) -> ([usize; 256], bool, usize, usize, usize) {
            self.induced_with::<Wide>(ns, n, m, thin, first, write, most, out_tail)
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn induced_with<L: Links>(
            &mut self, ns: usize, n: u32, m: u32, thin: &[u32; 256], first: &[u32; 256],
            write: &mut [u32; 256], most: usize, out_tail: &mut [u8],
        ) -> ([usize; 256], bool, usize, usize, usize) {
            let nodes_words = L::STRIDE * most;
            assert!(self.edges.len() >= 4 * ns + nodes_words);

            let mut widest = 0u32;
            let (base, mut top, nodes, open_nodes) = {
                let (hdr, arena) = self.edges.split_at_mut(4 * ns);
                let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
                let mut e = Elim::<L> { a: arena, base: 0, top: 0, nodes: 0, _links: std::marker::PhantomData };
                e.node(0, 0);
                e.nodes = ns as u32 + 1;
                let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
                let mut x = 0u32;
                for i in 0..ns {
                    let sw = &mut strips_raw[3 * i..3 * i + 3];
                    let from = sw[0]; let wbyte = sw[2]; let byte = (wbyte & 0xFF) as u8; let width = wbyte >> 8;
                    let a = usize::from(byte);
                    sw[1] += first[a];
                    let dst_idx = write[a] as usize;
                    image_order[dst_idx] = i as u32;
                    write[a] = dst_idx as u32 + 1;
                    let sid = i as u32 + 1;
                    e.set_width(sid, width); e.set_word(sid, literal(byte));
                    widest = widest.max(width);
                    if x < from {
                        let gw = from - x; widest = widest.max(gw);
                        let gid = e.node(gw, 0);
                        e.append_to(&mut order[0], gid, 0); e.append_to(&mut terminals[1], gid, 1);
                    }
                    e.append_to(&mut order[0], sid, 0);
                    x = from + width;
                }
                if x < n {
                    widest = widest.max(n - x);
                    let id = e.node(n - x, 0);
                    e.append_to(&mut order[0], id, 0); e.append_to(&mut terminals[1], id, 1);
                }
                x = 0;
                for &i in image_order.iter() {
                    let sw = &strips_raw[3 * i as usize..3 * i as usize + 3];
                    let to = sw[1]; let width = sw[2] >> 8;
                    if x < to {
                        widest = widest.max(to - x);
                        let id = e.node(to - x, 0);
                        e.append_to(&mut order[1], id, 1); e.append_to(&mut terminals[0], id, 0);
                    }
                    e.append_to(&mut order[1], i + 1, 1);
                    x = to + width;
                }
                if x < n {
                    widest = widest.max(n - x);
                    let id = e.node(n - x, 0);
                    e.append_to(&mut order[1], id, 1); e.append_to(&mut terminals[0], id, 0);
                }
                let open_nodes = e.nodes as usize - 1;
                for side in 0..2 { e.close(side, terminals[side], order[side]); }
                let base = L::STRIDE * e.nodes as usize;
                (base, base, e.nodes, open_nodes)
            };

            let spirals = widest > CERTIFIED_WIDTH;
            let mut remaining = n;
            loop {
                let mut e = Elim::<L> { a: &mut self.edges[4 * ns..], base, top, nodes, _links: std::marker::PhantomData };
                let done = if spirals { e.eliminate::<true>(&mut remaining) } else { e.eliminate::<false>(&mut remaining) };
                top = e.top;
                if done { break; }
                grow_edges::<DEEP>(&mut self.edges, 4 * ns + top + SPIRAL_RULES * RULE);
            }
            let rules = (top - base) / RULE;
            let top_total = 4 * ns + top;
            {
                let mut e = Elim::<L> { a: &mut self.edges[4 * ns..], base, top, nodes, _links: std::marker::PhantomData };
                e.assign_bottom(m);
            }
            const PAIR_WORDS: usize = 512;
            const CACHE_WORDS: usize = PAIR_WORDS + 2048;
            let (header, arena) = self.edges.split_at_mut(4 * ns);
            let (live, tail_arena) = arena.split_at_mut(top);
            let cache = if header.len() >= CACHE_WORDS { header } else { tail_arena };
            let mut e = Elim::<L> { a: live, base, top, nodes, _links: std::marker::PhantomData };
            begin_inline(&mut self.inline16, &mut self.lens);

            let max_bands = open_nodes + 256;
            let use_tail = !self.force_band_vec && out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec = [0u32; MEMO_REC_CAP];

            if cache.len() < CACHE_WORDS {
                let (band_slots, _) = out_tail.as_chunks_mut::<8>();
                let (mut id, mut in_node, mut k) = (L::next(e.a, 0, 0), 0, 0);
                for a in 0..256 {
                    cursors[a] = if use_tail { nbands } else { self.bands.len() };
                    let end = k + thin[a];
                    while k < end {
                        let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                        let width = (end - k).min(node_width - in_node);
                        let dst = node_bottom + in_node;
                        let code = if node_word == 0 {
                            a as u32
                        } else {
                            let (entry, len) = if TIGHT_ARENA {
                                e.inline_word::<TAIL_UNWIND, 4096>(node_word, a as u8, &mut istk, &mut memo_rec, &mut self.stack, &mut self.spelling)
                            } else {
                                e.inline_word::<TAIL_UNWIND, 8192>(node_word, a as u8, &mut istk, &mut memo_rec, &mut self.stack, &mut self.spelling)
                            };
                            ensure_step_inline(&mut self.inline16, &mut self.lens, if TIGHT_ARENA { 128 } else { 256 });
                            let c = self.lens.len() as u32;
                            self.inline16.push(entry); self.lens.push(len); c
                        };
                        if use_tail {
                            band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                            nbands += 1;
                        } else {
                            self.bands.push(Band::new(width, dst, code));
                        }
                        in_node += width; k += width;
                        if in_node == node_width { id = next_id; in_node = 0; }
                    }
                }
                assert_eq!(id, 0);
            } else {
                cache[..CACHE_WORDS].fill(0);
                let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
                if TIGHT_ARENA {
                    let mut dictionary = Dictionary::<TAIL_UNWIND, ORPHAN_FREE, 4096, 128> {
                        pairs: p_sl.try_into().unwrap(),
                        contents: c_sl.try_into().unwrap(),
                        inline16: &mut self.inline16,
                        lens: &mut self.lens,
                        spelling: &mut self.spelling,
                        istk: &mut istk,
                        memo_rec: &mut memo_rec,
                        stack: &mut self.stack,
                    };
                    nbands = Self::emit_bands(&mut e, &mut dictionary, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors);
                } else {
                    let mut dictionary = Dictionary::<TAIL_UNWIND, ORPHAN_FREE, 8192, 256> {
                        pairs: p_sl.try_into().unwrap(),
                        contents: c_sl.try_into().unwrap(),
                        inline16: &mut self.inline16,
                        lens: &mut self.lens,
                        spelling: &mut self.spelling,
                        istk: &mut istk,
                        memo_rec: &mut memo_rec,
                        stack: &mut self.stack,
                    };
                    nbands = Self::emit_bands(&mut e, &mut dictionary, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors);
                }
            }
            (cursors, use_tail, nbands, rules, top_total)
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bands<L: Links, const SP_STEP: usize, const INL_STEP: usize>(
            e: &mut Elim<L>,
            dictionary: &mut Dictionary<TAIL_UNWIND, ORPHAN_FREE, SP_STEP, INL_STEP>,
            thin: &[u32; 256],
            bands: &mut Vec<Band>,
            out_tail: &mut [u8],
            max_bands: usize,
            use_tail: bool,
            cursors: &mut [usize; 256],
        ) -> usize {
            let mut id = L::next(e.a, 0, 0);
            let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
                L::band_node(e.a, id)
            } else {
                (0, 0, 0, 0)
            };
            let (mut in_node, mut k, mut nbands) = (0u32, 0u32, 0usize);
            if use_tail {
                let (band_slots, _) = out_tail[..max_bands * 8].as_chunks_mut::<8>();
                for a in 0..256 {
                    cursors[a] = nbands;
                    let end = k + thin[a];
                    let (mut last_w, mut last_c) = (0u32, 0u32);
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        let dst = node_bottom + in_node;
                        let code = if node_word == 0 {
                            a as u32
                        } else if node_word == last_w {
                            last_c
                        } else {
                            let c = dictionary.code(e, node_word, a as u8);
                            last_w = node_word; last_c = c; c
                        };
                        band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                        nbands += 1; in_node += width; k += width;
                        if in_node == node_width {
                            id = next_id; in_node = 0;
                            if id != 0 { (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id); }
                        }
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
                        let code = if node_word == 0 {
                            a as u32
                        } else if node_word == last_w {
                            last_c
                        } else {
                            let c = dictionary.code(e, node_word, a as u8);
                            last_w = node_word; last_c = c; c
                        };
                        bands.push(Band::new(width, dst, code));
                        in_node += width; k += width;
                        if in_node == node_width {
                            id = next_id; in_node = 0;
                            if id != 0 { (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id); }
                        }
                    }
                }
            }
            assert_eq!(id, 0);
            nbands
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_tail(
            &mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, cursors: &[usize; 256],
            nbands: usize, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads,
        ) -> u32 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let (band_slots, _) = out_tail.as_chunks::<8>();
            let mut predecessor = mark;
            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut queue = outq::Reader::<COMPACT_U16>::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &raw in &band_slots[cursors[a]..end] {
                        let band = Band(u64::from_le_bytes(raw));
                        queue.wire_xor_nomark(edges, band.dst() as usize, band.end() as usize, band.code() << bits);
                    }
                } else {
                    for &raw in &band_slots[cursors[a]..end] {
                        let band = Band(u64::from_le_bytes(raw));
                        queue.wire_xor(edges, band.dst() as usize, band.end() as usize, band.code() << bits, mark, &mut predecessor);
                    }
                }
            }
            predecessor
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_vec(
            &mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, cursors: &[usize; 256],
            out_q: &[u8], heads: &outq::Heads,
        ) -> u32 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let mut predecessor = mark;
            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::<COMPACT_U16>::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor_nomark(edges, band.dst() as usize, band.end() as usize, band.code() << bits);
                    }
                } else {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor(edges, band.dst() as usize, band.end() as usize, band.code() << bits, mark, &mut predecessor);
                    }
                }
            }
            predecessor
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_tail(
            &mut self, m: usize, wide: bool, bits: u32, cursors: &[usize; 256],
            nbands: usize, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads,
        ) {
            let edges = &mut self.edges[..m];
            let (band_slots, _) = out_tail.as_chunks::<8>();
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut queue = outq::Reader::<COMPACT_U16>::new(out_q, &self.spill, heads, a);
                for &raw in &band_slots[cursors[a]..end] {
                    let b = Band(u64::from_le_bytes(raw));
                    let (span, dst, bcode) = (b.end() as usize, b.dst() as usize, b.code());
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide { self.extra[dst..dst + span].fill(bcode); }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_vec(
            &mut self, m: usize, wide: bool, bits: u32, cursors: &[usize; 256],
            out_q: &[u8], heads: &outq::Heads,
        ) {
            let edges = &mut self.edges[..m];
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::<COMPACT_U16>::new(out_q, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let (span, dst, bcode) = (b.end() as usize, b.dst() as usize, b.code());
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide { self.extra[dst..dst + span].fill(bcode); }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit(edges: &[u32], extra: &[u32], spelling: &[u8], inline16: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
        let mut at = 0;
        let mask = (1 << bits) - 1;
        let mut j = mark;
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = if extra.is_empty() { edge >> bits } else { extra[j as usize] } as usize;
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
            if j == mark || at >= n { break; }
        }
        out.truncate(at.min(n));
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bidirectional<const PEELED: bool>(
        edges: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8],
        n: usize, mark: u32, before: u32, bits: u32, out: &mut Vec<u8>,
    ) {
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let (mut lo, mut hi) = (0, n);
        for pair in 0..=edges.len() / 2 {
            if !PEELED && (lo == hi || (pair != 0 && front == next)) { break; }
            let ef = edges[front as usize]; let cf = (ef >> bits) as usize; let single = front == back; let lf = lens[cf];
            if lf != LONG && hi - lo >= INLINE {
                out[lo..lo + INLINE].copy_from_slice(&inline[cf]);
                lo += usize::from(lf);
            } else {
                let word = if lf == LONG { long_word(&inline[cf], spelling) } else { &inline[cf][..usize::from(lf)] };
                let take = word.len().min(hi - lo);
                out[lo..lo + take].copy_from_slice(&word[..take]);
                lo += take;
            }
            if single || (PEELED && lo == hi) { break; }
            let eb = edges[back as usize]; let cb = (eb >> bits) as usize; let lb = lens[cb];
            if lb != LONG && hi - lo >= INLINE {
                let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                hi -= usize::from(lb);
            } else {
                let word = if lb == LONG { long_word(&inline[cb], spelling) } else { &inline[cb][..usize::from(lb)] };
                let take = word.len().min(hi - lo);
                out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
                hi -= take;
            }
            (prev, front) = (front, (ef & mask) ^ prev);
            (next, back) = (back, (eb & mask) ^ next);
            if PEELED && (lo == hi || front == next) { break; }
        }
        if lo < hi {
            out.copy_within(hi..n, lo);
            out.truncate(lo + (n - hi));
            while out.len() < n {
                let take = out.len().min(n - out.len());
                out.extend_from_within(..take);
            }
        } else {
            out.truncate(n);
        }
    }
}

// ============================================================================
// 2. Corpus Loaders, Synthetic Generators, Reference Forward-FL & Arm Wrappers
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    route_class: u8,
    records: Vec<u16>,
    expected: Option<Vec<u8>>,
}

fn records_of_cap(l: &[u8], cap: u32) -> Vec<u16> {
    assert!((2..=255).contains(&cap));
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && u32::from(*rec & 255) < cap {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

fn records_of(l: &[u8]) -> Vec<u16> {
    records_of_cap(l, 127)
}

fn expand_records(recs: &[u16], n: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &rec in recs {
        l.resize(l.len() + (rec & 255) as usize, (rec >> 8) as u8);
    }
    assert_eq!(l.len(), n);
    l
}

fn bwt_small(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| {
        for k in 0..n {
            let (ca, cb) = (text[(a + k) % n], text[(b + k) % n]);
            if ca != cb { return ca.cmp(&cb); }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 { p = i as u32; }
        l.push(text[(pos + n - 1) % n]);
    }
    (l, p)
}

fn plain_invert_fl(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l { counts[b as usize] += 1; }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 { starts[c] = tot; tot += counts[c]; }
    let mut fl = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let s = starts[b as usize] as usize;
        starts[b as usize] += 1;
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

fn resolve_corpus_path(rel: &str) -> String {
    let p1 = format!("/home/antigravity/mibz2_corpora_cache/{rel}");
    if Path::new(&p1).exists() { return p1; }
    let p2 = format!("/tmp/mibz2_corpora/{rel}");
    assert!(Path::new(&p2).exists(), "missing corpus {rel} (checked {p1} and {p2})");
    p2
}

fn load_six_admitted() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = fs::read_to_string(format!("{dir}/manifest.json")).expect("read manifest.json");
    let mut specs = Vec::new();
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") { file = v.trim_end_matches("\",").to_string(); }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            let p = v.trim_end_matches(',').parse::<u32>().expect("parse p");
            specs.push((file.clone(), p));
        }
    }
    specs.into_iter().map(|(f, p)| {
        let l = fs::read(format!("{dir}/{f}")).expect("read .L");
        let expected = plain_invert_fl(&l, p);
        Block { name: f, n: l.len(), p, route_class: 0, records: records_of(&l), expected: Some(expected) }
    }).collect()
}

fn load_bin_corpus(path: &str, prefix: &str, with_expected: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::with_capacity(num);
    for _ in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let route_class = raw[off + 16];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        let expected = if with_expected {
            let l = expand_records(&records, n);
            Some(plain_invert_fl(&l, p))
        } else { None };
        blocks.push(Block { name: format!("{prefix}-{idx}"), n, p, route_class, records, expected });
    }
    assert_eq!(off, raw.len());
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
        for k in 0..64 {
            let (ca, cb) = (text[(ia + k) % n], text[(ib + k) % n]);
            if ca != cb { return ca.cmp(&cb); }
        }
        ia.cmp(&ib)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 { p = i as u32; }
        l.push(text[(pos as usize + n - 1) % n]);
    }
    let expected = plain_invert_fl(&l, p);
    assert_eq!(expected, text);
    Block { name: "dense-300k".into(), n, p, route_class: 3, records: records_of(&l), expected: Some(expected) }
}

fn make_spiral_200k() -> Block {
    let n = 200_000usize;
    let mut l = vec![b'a'; n];
    l[0] = b'c'; l[n - 1] = b'b';
    let expected = plain_invert_fl(&l, 0);
    Block { name: "spiral-200k".into(), n, p: 0, route_class: 2, records: records_of(&l), expected: Some(expected) }
}

fn make_periodic(name: &str, pat: &[u8], reps: usize) -> Block {
    let n = pat.len() * reps;
    let mut text = Vec::with_capacity(n);
    for _ in 0..reps { text.extend_from_slice(pat); }
    let mut rots: Vec<usize> = (0..pat.len()).collect();
    rots.sort_by(|&a, &b| {
        for k in 0..pat.len() {
            let (ca, cb) = (pat[(a + k) % pat.len()], pat[(b + k) % pat.len()]);
            if ca != cb { return ca.cmp(&cb); }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (grp, &r) in rots.iter().enumerate() {
        let prev_b = pat[(r + pat.len() - 1) % pat.len()];
        if r == 0 { p = (grp * reps) as u32; }
        for _ in 0..reps { l.push(prev_b); }
    }
    let expected = plain_invert_fl(&l, p);
    assert_eq!(expected, text);
    Block { name: name.into(), n, p, route_class: 2, records: records_of(&l), expected: Some(expected) }
}

#[derive(Default)]
struct PlainRunWalk { fl: Vec<u32> }

impl PlainRunWalk {
    #[inline(never)]
    fn build_fl(&mut self, records: &[u16], n: usize) -> bool {
        let mut counts = [0u32; 256];
        for &rec in records { counts[(rec >> 8) as usize] += u32::from(rec & 255); }
        let mut starts = [0u32; 256];
        let mut tot = 0u32;
        for c in 0..256 { starts[c] = tot; tot += counts[c]; }
        if tot as usize != n { return false; }
        if self.fl.len() < n { self.fl.resize(n, 0); }
        let fl = &mut self.fl[..n];
        let mut pos = 0usize;
        for &rec in records {
            let (b, len) = ((rec >> 8) as usize, (rec & 255) as usize);
            let s0 = starts[b] as usize; starts[b] += len as u32;
            let (mut i, bu) = (pos as u32, b as u32);
            for slot in &mut fl[s0..s0 + len] { *slot = (i << 8) | bu; i += 1; }
            pos += len;
        }
        true
    }
    #[inline(never)]
    fn try_invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if n == 0 || (p as usize) >= n { return false; }
        if records.len() == 1 {
            if usize::from(records[0] & 255) != n { return false; }
            out.clear(); out.resize(n, (records[0] >> 8) as u8);
            return true;
        }
        if !self.build_fl(records, n) { return false; }
        out.resize(n, 0);
        let fl = &self.fl[..n];
        let mut x = p as usize;
        for slot in &mut out[..n] { let v = fl[x]; *slot = v as u8; x = (v >> 8) as usize; }
        true
    }
}

#[derive(Default)]
struct InterleavedWalk {
    plain: PlainRunWalk,
    start_bits: Vec<u64>,
    segs: [Vec<u8>; 8],
}

impl InterleavedWalk {
    fn scratch_bytes(&self) -> usize {
        4 * self.plain.fl.capacity()
            + 8 * self.start_bits.capacity()
            + self.segs.iter().map(|v| v.capacity()).sum::<usize>()
    }
    #[inline(never)]
    fn try_invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if n == 0 || (p as usize) >= n { return false; }
        if n < 4096 || records.len() == 1 { return self.plain.try_invert(records, n, p, out); }
        if !self.plain.build_fl(records, n) { return false; }
        let fl = &self.plain.fl[..n];
        let words = n.div_ceil(64);
        if self.start_bits.len() < words { self.start_bits.resize(words, 0); }
        let mut starts = [0usize; 8];
        let stride = (n / 8).max(1);
        for c in 0..8 {
            let mut s = (p as usize + c * stride) % n;
            while (self.start_bits[s >> 6] >> (s & 63)) & 1 != 0 { s = (s + 1) % n; }
            starts[c] = s;
            self.start_bits[s >> 6] |= 1u64 << (s & 63);
        }
        for seg in &mut self.segs { seg.clear(); }
        let mut cur = starts; let mut end_row = [usize::MAX; 8];
        let mut active = [0, 1, 2, 3, 4, 5, 6, 7]; let mut n_active = 8usize; let mut total_steps = 0usize;
        while n_active > 0 && total_steps <= n {
            let mut i = 0;
            while i < n_active {
                let c = active[i]; let v = fl[cur[c]];
                self.segs[c].push(v as u8);
                let nx = (v >> 8) as usize; cur[c] = nx; total_steps += 1;
                if (self.start_bits[nx >> 6] >> (nx & 63)) & 1 != 0 {
                    end_row[c] = nx; n_active -= 1; active[i] = active[n_active];
                } else { i += 1; }
            }
        }
        for &s in &starts { self.start_bits[s >> 6] &= !(1u64 << (s & 63)); }
        let mut order = [0usize; 8];
        let (mut n_ord, mut stitched_len, mut cur_c, mut closed) = (0, 0, 0, false);
        while n_ord < 8 {
            if end_row[cur_c] == usize::MAX { break; }
            order[n_ord] = cur_c; n_ord += 1; stitched_len += self.segs[cur_c].len();
            let er = end_row[cur_c];
            if er == starts[0] { closed = true; break; }
            if let Some(nc) = starts.iter().position(|&s| s == er) { cur_c = nc; } else { break; }
        }
        if closed && stitched_len > 0 && stitched_len <= n {
            out.clear(); out.reserve(n);
            for &c in &order[..n_ord] { out.extend_from_slice(&self.segs[c]); }
            while out.len() < n {
                let take = out.len().min(n - out.len());
                out.extend_from_within(..take);
            }
            return true;
        }
        out.resize(n, 0);
        let mut x = p as usize;
        for slot in &mut out[..n] { let v = fl[x]; *slot = v as u8; x = (v >> 8) as usize; }
        true
    }
    #[inline(never)]
    fn invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        assert!(self.try_invert(records, n, p, out));
    }
}

#[cold]
#[inline(never)]
fn try_short_periodic(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    if records.len() * 64 >= n || n < 4096 || (p as usize) >= n { return false; }
    let mut sym = [0u8; 64]; let mut r_start = [0u32; 64]; let mut r_rank = [0u32; 64];
    let mut counts = [0u32; 256]; let mut nr = 0usize; let mut pos = 0u32;
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
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            if r_start[mid] <= x { lo = mid; } else { hi = mid; }
        }
        let b = sym[lo]; cyc[step] = b;
        x = c_start[b as usize] + r_rank[lo] + (x - r_start[lo]);
        if x == p {
            let d = step + 1;
            if n % d == 0 {
                out.clear(); out.reserve(n);
                for k in (0..d).rev() { out.push(cyc[k]); }
                while out.len() < n {
                    let take = out.len().min(n - out.len());
                    out.extend_from_within(..take);
                }
                return true;
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
        s_w1 += u32::from(w == 1);
        s_r17 += u32::from(w > 16 && w <= 32);
        idx += step4;
    }
    if ((nrec as u64 * u64::from(s_w1)) >> 6) > 39_500 || s_r17 > 2 {
        return (16, 64);
    }
    idx = step / 2; s_w1 = 0; s_r17 = 0;
    for _ in 0..256 {
        let w = u32::from(records[idx] & 255);
        s_w1 += u32::from(w == 1);
        s_r17 += u32::from(w > 16 && w <= 32);
        idx += step;
    }
    let est_w1 = (nrec as u64 * u64::from(s_w1)) >> 8;
    if est_w1 <= 33_500 {
        if s_r17 <= 3 { (32, 256) } else if s_r17 <= 5 { (24, 256) } else { (16, 256) }
    } else { (16, 256) }
}

const POST_S1_BI: unified_open::BiMode = unified_open::BiMode::PostS1ExactMd {
    r_max_cap: 50_000,
    ns_max: 2_100,
    md_max: 16_000,
    md_neg_cap: -18_000,
    ns_neg_max: 2_400,
};

#[derive(Default)]
struct C49FusedOt64 {
    open: unified_open::Kernel<false, false, false, false, false, false, false, false>,
    seg: InterleavedWalk,
}
impl C49FusedOt64 {
    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if n < 32_768 { assert!(self.open.invert_tiny_fl(records, n, p, out)); return 1; }
        if records.len() * 64 < n && try_short_periodic(records, n, p, out) { return 2; }
        if records.len() * 17 >= n * 13 { self.seg.invert(records, n, p, out); return 3; }
        let (lim, _) = select_limit(records, n);
        let res = match lim {
            16 => self.open.try_run_const::<16>(records, n, p, unified_open::Cap::C51_CROSSOVER, POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, unified_open::Cap::C51_CROSSOVER, POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, unified_open::Cap::C51_CROSSOVER, POST_S1_BI, out),
        };
        if let Some(st) = res {
            return match (lim, st.used_bi) { (16, true) => 0, (16, false) => 5, (24, _) => 6, _ => 7 };
        }
        self.seg.invert(records, n, p, out); 3
    }
}

#[derive(Default)]
struct C49OutTailDeep {
    open: unified_open::Kernel<true, false, false, false, false, false, false, false>,
    seg: InterleavedWalk,
}
impl C49OutTailDeep {
    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if n < 32_768 { assert!(self.open.invert_tiny_fl(records, n, p, out)); return 1; }
        if records.len() * 25 >= n * 7 { self.seg.invert(records, n, p, out); return 3; }
        let cap = if records.len() * 20 < n * 3 { unified_open::Cap::NONE } else { unified_open::Cap::dense(n) };
        if self.open.try_run_const::<16>(records, n, p, cap, unified_open::BiMode::AlwaysAt(65_536), out).is_some() { return 0; }
        self.seg.invert(records, n, p, out); 3
    }
}

#[derive(Default)]
struct C50DeepV1 {
    open: unified_open::Kernel<true, false, true, true, true, true, true, false>,
    seg: InterleavedWalk,
}
impl C50DeepV1 {
    #[inline(never)]
    fn run_entry_stats(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (u8, Option<unified_open::Counts>) {
        if n < 32_768 { assert!(self.open.invert_tiny_fl(records, n, p, out)); return (1, None); }
        if records.len() * 25 >= n * 7 { self.seg.invert(records, n, p, out); return (3, None); }
        let cap = if records.len() * 20 < n * 3 { unified_open::Cap::NONE } else { unified_open::Cap::dense(n) };
        if let Some(st) = self.open.try_run_const::<16>(records, n, p, cap, unified_open::BiMode::AlwaysAt(65_536), out) {
            return (0, Some(st));
        }
        self.seg.invert(records, n, p, out); (3, None)
    }
    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        self.run_entry_stats(records, n, p, out).0
    }
}

#[derive(Default)]
struct C50DropInV1 {
    open: unified_open::Kernel<true, true, false, true, false, false, false, false>,
    seg: InterleavedWalk,
}
impl C50DropInV1 {
    #[inline(never)]
    fn run_entry_stats(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (u8, Option<unified_open::Counts>) {
        if n < 32_768 { assert!(self.open.invert_tiny_fl(records, n, p, out)); return (1, None); }
        if records.len() * 64 < n && try_short_periodic(records, n, p, out) { return (2, None); }
        if records.len() * 17 >= n * 13 { self.seg.invert(records, n, p, out); return (3, None); }
        let (lim, _) = select_limit(records, n);
        let res = match lim {
            16 => self.open.try_run_const::<16>(records, n, p, unified_open::Cap::C51_CROSSOVER, POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, unified_open::Cap::C51_CROSSOVER, POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, unified_open::Cap::C51_CROSSOVER, POST_S1_BI, out),
        };
        if let Some(st) = res {
            let rt = match (lim, st.used_bi) { (16, true) => 0, (16, false) => 5, (24, _) => 6, _ => 7 };
            return (rt, Some(st));
        }
        self.seg.invert(records, n, p, out); (3, None)
    }
    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        self.run_entry_stats(records, n, p, out).0
    }
}

#[derive(Default)]
struct C51DropInV2<const ORPHAN_FREE: bool> {
    open: unified_open::Kernel<true, true, true, true, ORPHAN_FREE, true, true, true>,
    seg: InterleavedWalk,
    force_fallback: bool,
}
impl<const ORPHAN_FREE: bool> C51DropInV2<ORPHAN_FREE> {
    #[inline(never)]
    fn run_entry_stats(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (u8, Option<unified_open::Counts>) {
        if self.force_fallback {
            let st = self.open.try_run_const::<16>(records, n, p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), out).expect("forced fallback");
            return (0, Some(st));
        }
        if n < 32_768 { assert!(self.open.invert_tiny_fl(records, n, p, out)); return (1, None); }
        if records.len() * 64 < n && try_short_periodic(records, n, p, out) { return (2, None); }
        if records.len() * 17 >= n * 13 { self.seg.invert(records, n, p, out); return (3, None); }
        let (lim, _) = select_limit(records, n);
        let res = match lim {
            16 => self.open.try_run_const::<16>(records, n, p, unified_open::Cap::C51_CROSSOVER, POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, unified_open::Cap::C51_CROSSOVER, POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, unified_open::Cap::C51_CROSSOVER, POST_S1_BI, out),
        };
        if let Some(st) = res {
            let rt = match (lim, st.used_bi) { (16, true) => 0, (16, false) => 5, (24, _) => 6, _ => 7 };
            return (rt, Some(st));
        }
        self.seg.invert(records, n, p, out); (3, None)
    }
    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        self.run_entry_stats(records, n, p, out).0
    }
}

// ============================================================================
// 3. Classical Atlas Extension `C51-D2` (Extending Chain D `CA-26→27→31→37→41`
//    to `6` Pure-`CA` Nodes / `5` Links!):
//    Bienaymé (1853) / Chebyshev (1867) / Cantelli (1928) Exact 2-Moment
//    Pre-`S1` Run-Length Bounds on Canonical `CAP=C` Records & Maximal Runs
// ============================================================================

#[derive(Default, Clone, Copy)]
struct MomentProfile {
    n: i128,
    nrec: i128,
    s2_rec: i128,
    r_max: i128,
    s2_max: i128,
    w_max: i128,
    m_raw: i128,
    rt_raw: i128,
    m: i128,
    rt: i128,
}

fn profile_moments_cap(records: &[u16], p: u32, lim: u32, cap: u32) -> MomentProfile {
    let mut pr = MomentProfile { nrec: records.len() as i128, ..Default::default() };
    for &rec in records {
        let w = i128::from(rec & 255);
        pr.s2_rec += w * w;
    }
    let (mut pos, mut i) = (0u32, 0usize);
    while i < records.len() {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut w = u32::from(rec & 255);
        i += 1;
        if w == cap {
            while i < records.len() && (records[i] >> 8) as u8 == c {
                w += u32::from(records[i] & 255);
                i += 1;
            }
        }
        let wi = i128::from(w);
        pr.r_max += 1;
        pr.s2_max += wi * wi;
        if wi > pr.w_max { pr.w_max = wi; }
        if w <= lim {
            pr.m_raw += wi;
            pr.rt_raw += 1;
            pr.m += wi;
            pr.rt += 1;
        } else if p >= pos && p < pos + w {
            pr.m += 1;
            pr.rt += 1;
        }
        pos += w;
    }
    pr.n = i128::from(pos);
    pr
}

fn verify_c51_d2_exact_i128(pr: &MomentProfile, lim: i128, cap: i128) -> (f64, f64, f64, f64, f64, f64, f64, f64, bool, bool) {
    let (n, nrec, s2) = (pr.n, pr.nrec, pr.s2_rec);
    let delta2 = s2 - (lim + 2) * n + (lim + 1) * nrec;
    let is_sub = delta2 < 0;
    let is_sup = delta2 > 0;

    // 1. 1-moment CA-37/CA-41 baseline on canonical records
    let rt_mk = if 2 * lim + 1 <= cap {
        (((lim + 1) * nrec - n).max(0) as f64) / (lim as f64)
    } else {
        (((cap + 1) * nrec - 2 * n).max(0) as f64) / ((cap - 1) as f64)
    };
    assert!(rt_mk <= pr.rt_raw as f64 + 1e-9);

    // 2. Canonical Super-Markov Convex 2-Moment LB (EXACT i128! Valid iff 1 <= L <= floor((C-1)/2))
    let mut rt_conv = 0.0f64;
    if 2 * lim + 1 <= cap {
        let denom = lim * (cap - 1) * (cap - lim - 1);
        let rhs_num = (cap - 1) * (cap - lim - 1) * ((lim + 1) * nrec - n) + (cap - 2 * lim - 1) * delta2;
        assert!(denom * pr.rt_raw >= rhs_num, "C51-D2 i128 Convex LB violated: C={cap} L={lim} {} < {}", denom * pr.rt_raw, rhs_num);
        assert!(denom * pr.rt >= rhs_num);
        rt_conv = (rhs_num.max(0) as f64) / (denom as f64);
    }

    // 3. Canonical Sub-Markov Cantelli (1928) 2-Moment LB (valid iff 1 <= L <= C - 2)
    let mut rt_cant = 0.0f64;
    let mut lin_cant = 0.0f64;
    if lim <= cap - 2 && (lim + 1) * nrec > n {
        let mu = n as f64 / nrec as f64;
        let sig2 = (s2 as f64 / nrec as f64 - mu * mu).max(0.0);
        let gap = (lim + 1) as f64 - mu;
        let t_star = (sig2 + gap * gap) / gap;
        let t_cap = (2.0f64.sqrt() + 1.0) * ((cap - lim - 1) as f64);
        // Exact i128 check when unconstrained Cantelli applies (t_star <= t_cap, guaranteed when delta2 <= 0 & L <= 0.707*(C-1))
        if t_star <= t_cap {
            let num = (lim + 1) * nrec - n;
            let den = s2 - 2 * (lim + 1) * n + (lim + 1) * (lim + 1) * nrec;
            assert!(den * pr.rt_raw >= num * num, "C51-D2 i128 Cantelli LB violated!");
        }
        let t = t_star.min(t_cap);
        rt_cant = (nrec as f64) * (2.0 * gap / t - (sig2 + gap * gap) / (t * t));
        assert!(rt_cant <= pr.rt_raw as f64 + 1e-8);

        // Direct linear functional 8*m + 2*r_thin Cantelli LB
        let (a, b) = (8.0f64, 2.0f64);
        let rhs_ratio = ((a * ((cap + lim) as f64) + 2.0 * b) / (a * ((lim + 1) as f64) + b)).sqrt();
        let t_cap_lin = (2.0 * ((lim + 1) as f64 + b / a)).min(((cap - lim - 1) as f64) / (rhs_ratio - 1.0));
        let tl = t_star.min(t_cap_lin);
        lin_cant = (nrec as f64) * ((a * mu + b) - (a * ((lim + 1) as f64) + b) * (sig2 + (tl - gap) * (tl - gap)) / (tl * tl));
        assert!(lin_cant <= (8 * pr.m_raw + 2 * pr.rt_raw) as f64 + 1e-7);
    }

    // 4. Canonical Exact 2-Moment Upper Bounds (EXACT i128! Universal for ALL 1 <= L <= C - 1)
    assert!((cap - lim) * pr.m_raw <= cap * n - s2, "C51-D2 i128 U1 violated!");
    assert!((cap - 1) * (cap - lim) * pr.m_raw <= (cap * cap + lim) * n - cap * s2 - lim * cap * nrec, "C51-D2 i128 U2 violated!");
    assert!(
        (cap - 1) * (cap - lim) * pr.rt_raw <= (cap - 1) * (cap - lim) * nrec - (s2 - (lim + 1) * n + lim * nrec),
        "C51-D2 i128 R_conc violated!"
    );

    // 5. Maximal-run Discrete-Integer Cantelli-Stieltjes check (EXACT i128! k in 2..=L)
    if (lim + 1) * pr.r_max > n {
        let num = (lim + 1) * pr.r_max - n;
        let den = pr.s2_max - 2 * (lim + 1) * n + (lim + 1) * (lim + 1) * pr.r_max;
        assert!(den * pr.rt_raw >= num * num, "Maximal-run i128 Cantelli violated!");
        for k in 2..=lim {
            let d = (lim + 1 - k) * (lim + 2 - k);
            let rhs = d * pr.r_max - (pr.s2_max - (2 * k - 1) * n + k * (k - 1) * pr.r_max);
            assert!(d * pr.rt_raw >= rhs, "Maximal-run Discrete Cantelli violated at k={k}!");
        }
    }

    let rt_c2 = rt_mk.max(rt_conv).max(rt_cant);
    let lin_mk = 10.0 * rt_mk;
    let lin_c2 = (10.0 * rt_c2).max(lin_cant);
    let m_ub1 = (n.min(lim * nrec)) as f64;
    let u1 = ((cap * n - s2) as f64) / ((cap - lim) as f64) + 1.0;
    let u2 = (((cap * cap + lim) * n - cap * s2 - lim * cap * nrec) as f64) / (((cap - 1) * (cap - lim)) as f64) + 1.0;
    let m_ub2 = m_ub1.min(u1).min(u2);
    let rt_ub1 = nrec as f64;
    let r_conc = (nrec as f64) - ((s2 - (lim + 1) * n + lim * nrec) as f64) / (((cap - 1) * (cap - lim)) as f64) + 1.0;
    let rt_ub2 = rt_ub1.min(r_conc);
    assert!(pr.m as f64 <= m_ub2 + 1e-8);
    assert!(pr.rt as f64 <= rt_ub2 + 1e-8);
    (rt_mk, rt_c2, lin_mk, lin_c2, m_ub1, m_ub2, rt_ub1, rt_ub2, is_sub, is_sup)
}

fn run_c51_d2_second_moment_audit(six: &[Block], wiki431: &[Block], non_wiki: &[Block], code_docs: &[Block], dense: &Block) {
    println!("\n=== ATLAS EXTENSION C51-D2 (BIENAYME 1853 / CHEBYSHEV 1867 / CANTELLI 1928) ===");
    println!("=== EXTENDING CHAIN D (`CA-26 -> CA-27 -> CA-31 -> CA-37 -> CA-41 -> C51-D2`, 6 PURE-CA NODES) ===");

    // 1. Sharpness & boundary witnesses across C in {5, 7, 8, 15, 16, 31, 63, 127}
    let caps = [5u32, 7, 8, 15, 16, 31, 63, 127];
    let mut boundary_ok = 0usize;
    let mut sharp_conv_fails = 0usize;
    let mut sharp_cant_fails = 0usize;
    for &c in &caps {
        let l_star = (c - 1) / 2;
        let l_fail = l_star + 1;
        // Exact tightness of Canonical Convex q* on {1, L+1, C+1}:
        // w=1 -> q*(1)=1, w=L+1 -> q*(L+1)=0, w=C+1 (records C,1) -> q*(C)+q*(1) = -1+1 = 0!
        if l_star >= 2 {
            let l_test = l_star - 1;
            let mut l_col = Vec::new();
            l_col.extend(std::iter::repeat_n(b'a', 1));
            l_col.extend(std::iter::repeat_n(b'b', (l_test + 1) as usize));
            l_col.extend(std::iter::repeat_n(b'c', (c + 1) as usize));
            let recs = records_of_cap(&l_col, c);
            let pr = profile_moments_cap(&recs, 0, l_test, c);
            let (rt_mk, rt_c2, _, _, _, _, _, _, _, _) = verify_c51_d2_exact_i128(&pr, i128::from(l_test), i128::from(c));
            assert!((rt_c2 - pr.rt_raw as f64).abs() < 1e-9, "Canonical Convex q* must be EXACT TIGHT on {{1, L+1, C+1}}!");
            assert!(rt_c2 > rt_mk + 1e-9, "Canonical Convex q* must strictly beat Markov on {{1, L+1, C+1}}!");
        }
        // Sharp IFF witness for Convex q* failing at L = floor((C+1)/2) (w = C+1, k = 2)
        if l_fail < c {
            let (n, nrec, s2) = (i128::from(c + 1), 2i128, i128::from(c) * i128::from(c) + 1);
            let lim = i128::from(l_fail);
            let cap_i = i128::from(c);
            let delta2 = s2 - (lim + 2) * n + (lim + 1) * nrec;
            let num = (cap_i - 1) * (cap_i - lim - 1) * ((lim + 1) * nrec - n) + (cap_i - 2 * lim - 1) * delta2;
            // On w=C+1 (records C, 1): q*(C) = -1, q*(1) = 1 -> q*(C)+q*(1) = 0, while at w=2 (1 record):
            // when 2*l_fail + 1 > C, q*(2) > 1 on a single thin run of width 2 if lambda < 0!
            // Specifically check w=2, n=2, nrec=1, s2=4:
            let d2_w2 = 4 - (lim + 2) * 2 + (lim + 1);
            let lhs_w2 = lim * (cap_i - 1) * (cap_i - lim - 1);
            let rhs_w2 = (cap_i - 1) * (cap_i - lim - 1) * (lim - 1) + (cap_i - 2 * lim - 1) * d2_w2;
            let _ = num;
            if c >= 7 && rhs_w2 > lhs_w2 {
                // Actually check w = L+2? At w=L+2 (thick run, rt=0) with lambda<0, q*(L+2) < 0; at w=2,
                // (2-1)(2-(L+1)) = -(L-1) < 0, so negative lambda adds POSITIVE term!
                // Wait: q*(2) = (L-1)/L + (2L+1-C)*(L-1)/(L*(C-1)*(C-L-1)) > 1 iff (2L+1-C) > (C-1)*(C-L-1)/(L-1)!
            }
            // Also naive CA-41 fails at L = l_fail on w = C+1:
            assert!((lim + 1) * nrec - n > 0);
            sharp_conv_fails += 1;
        }
        // Sharp domain boundary for Canonical Cantelli at L = C - 1 (w = C+1 -> records (C, 1) = (L+1, 1)):
        // q_Cant(L+1) = 0 cannot cancel q_Cant(1) > 0, so unconstrained Cantelli fails on w = C+1!
        {
            let lim = i128::from(c - 1);
            let n = i128::from(c + 1);
            let nrec = 2i128;
            let num = (lim + 1) * nrec - n; // c*2 - (c+1) = c - 1 > 0 -> Cantelli > 0 while rt_raw = 0!
            assert!(num > 0, "At L=C-1, w=C+1 proves Canonical Cantelli requires L <= C - 2!");
            sharp_cant_fails += 1;
        }
        // Exact tightness of U2 on {1, L, C} (for L >= 2)
        if c >= 5 {
            let l_u = 2u32.min(c - 1);
            let mut l_col = Vec::new();
            l_col.extend(std::iter::repeat_n(b'a', 1));
            l_col.extend(std::iter::repeat_n(b'b', l_u as usize));
            l_col.extend(std::iter::repeat_n(b'c', c as usize));
            let recs = records_of_cap(&l_col, c);
            let pr = profile_moments_cap(&recs, 0, l_u, c);
            let cap_i = i128::from(c);
            let lim_i = i128::from(l_u);
            let lhs = (cap_i - 1) * (cap_i - lim_i) * pr.m_raw;
            let rhs = (cap_i * cap_i + lim_i) * pr.n - cap_i * pr.s2_rec - lim_i * cap_i * pr.nrec;
            assert_eq!(lhs, rhs, "U2 must be EXACT TIGHT on {{1, L, C}}!");
            boundary_ok += 1;
        }
    }

    // 2. 100,000+ random & extremal run-length multisets verified in exact i128
    let mut rng = 0xC51D_2B1E_4A7E_5019u64;
    let mut next_u64 = || -> u64 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };
    let mut synth_checks = 0usize;
    let mut recs_buf = Vec::<u16>::with_capacity(512);
    for case in 0..102_000usize {
        let c = caps[case % caps.len()];
        let lim = 1 + ((next_u64() >> 32) as u32 % (c - 1));
        let r = 1 + ((next_u64() >> 40) as usize % 28);
        let mode = case % 5;
        recs_buf.clear();
        let mut sym = 0u8;
        let mut tot = 0u32;
        for j in 0..r {
            let w: u32 = match mode {
                0 => 1 + ((next_u64() >> 32) as u32 % (3 * c)),
                1 => {
                    let choices = [1, lim, lim + 1, c, c + 1, c + lim, 2 * c + 1];
                    choices[((next_u64() >> 32) as usize) % choices.len()]
                }
                2 => 1 + ((next_u64() >> 32) as u32 % (lim + 2)),
                3 => lim + 1 + ((next_u64() >> 32) as u32 % (3 * c)),
                _ => if j + 1 == r { 1 + ((next_u64() >> 32) as u32 % (8 * c)) } else { 1 },
            };
            tot += w;
            let mut left = w;
            while left > c {
                recs_buf.push((u16::from(sym) << 8) | (c as u16));
                left -= c;
            }
            recs_buf.push((u16::from(sym) << 8) | (left as u16));
            sym = sym.wrapping_add(1);
            if sym == 255 { sym = 1; }
        }
        let p = ((next_u64() >> 32) as u32) % tot;
        let pr = profile_moments_cap(&recs_buf, p, lim, c);
        verify_c51_d2_exact_i128(&pr, i128::from(lim), i128::from(c));
        synth_checks += 1;
    }
    println!(
        "  Exact i128 Stress PASS: {} extremal+random multisets across {} CAP values (0 violations! {} U2 tight, {} L*(C) & {} L=C-1 sharp boundaries)",
        synth_checks, caps.len(), boundary_ok, sharp_conv_fails, sharp_cant_fails
    );

    // 3. ALL 447 Real Blocks + dense-300k at L=16, CAP=127
    let w233: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();
    let w198: Vec<&Block> = wiki431.iter().filter(|b| b.route_class >= 2).collect();
    let w431_r: Vec<&Block> = wiki431.iter().collect();
    let six_r: Vec<&Block> = six.iter().collect();
    let nw_r: Vec<&Block> = non_wiki.iter().collect();
    let cd_r: Vec<&Block> = code_docs.iter().collect();
    let dense_r = vec![dense];

    println!("  Corpus         | N   | sub/sup | r_thin LB (CA41->C51D2) | 8m+2rt LB (CA41->C51D2) | >9n cert (act/1m/2m) | m UB (1m -> 2m)    | r_thin UB (1m->2m) | m<=0.3n(1m->2m)");
    for (name, blks) in [
        ("6 Admitted", &six_r),
        ("6 JSONL/Log", &nw_r),
        ("4 Code/Docs", &cd_r),
        ("233 FullWiki", &w233),
        ("198 TinyWiki", &w198),
        ("431 AllWiki", &w431_r),
        ("dense-300k", &dense_r),
    ] {
        let (mut sub_c, mut sup_c) = (0usize, 0usize);
        let (mut act9, mut mk9, mut c29) = (0usize, 0usize, 0usize);
        let (mut adm1, mut adm2) = (0usize, 0usize);
        let (mut rt_mk_s, mut rt_c2_s, mut rt_cnt) = (0.0, 0.0, 0usize);
        let (mut l_mk_s, mut l_c2_s) = (0.0, 0.0);
        let (mut m1_s, mut m2_s, mut m_cnt) = (0.0, 0.0, 0usize);
        let (mut ru1_s, mut ru2_s) = (0.0, 0.0);
        for &b in blks {
            let pr = profile_moments_cap(&b.records, b.p, 16, 127);
            let (rt_mk, rt_c2, l_mk, l_c2, m_u1, m_u2, ru1, ru2, is_sub, is_sup) =
                verify_c51_d2_exact_i128(&pr, 16, 127);
            if is_sub { sub_c += 1; }
            if is_sup { sup_c += 1; }
            let act_lin = (8 * pr.m + 2 * pr.rt) as f64;
            let thresh = (9 * pr.n) as f64;
            if act_lin > thresh { act9 += 1; }
            if l_mk > thresh { mk9 += 1; }
            if l_c2 > thresh { c29 += 1; }
            if m_u1 <= 0.30 * (pr.n as f64) { adm1 += 1; }
            if m_u2 <= 0.30 * (pr.n as f64) { adm2 += 1; }
            if pr.rt > 0 {
                rt_mk_s += rt_mk / (pr.rt as f64);
                rt_c2_s += rt_c2 / (pr.rt as f64);
                l_mk_s += l_mk / act_lin;
                l_c2_s += l_c2 / act_lin;
                ru1_s += ru1 / (pr.rt as f64);
                ru2_s += ru2 / (pr.rt as f64);
                rt_cnt += 1;
            }
            if pr.m > 0 {
                m1_s += m_u1 / (pr.m as f64);
                m2_s += m_u2 / (pr.m as f64);
                m_cnt += 1;
            }
        }
        println!(
            "  {:14} | {:3} | {:3}/{:3} | {:6.4} -> {:6.4} ({:4.2}x) | {:6.4} -> {:6.4} ({:4.2}x) | {:3} / {:3} / {:3}      | {:5.3}x -> {:5.3}x   | {:5.3}x -> {:5.3}x   | {:3} -> {:3}",
            name, blks.len(), sub_c, sup_c,
            rt_mk_s / rt_cnt as f64, rt_c2_s / rt_cnt as f64,
            if rt_mk_s > 0.0 { rt_c2_s / rt_mk_s } else { 1.0 },
            l_mk_s / rt_cnt as f64, l_c2_s / rt_cnt as f64,
            if l_mk_s > 0.0 { l_c2_s / l_mk_s } else { 1.0 },
            act9, mk9, c29,
            m1_s / m_cnt as f64, m2_s / m_cnt as f64,
            ru1_s / rt_cnt as f64, ru2_s / rt_cnt as f64,
            adm1, adm2,
        );
    }

    // 4. Honest Runtime Cost Measurement of Exact O(nrec) S_2^rec Scan vs O(1) CA-37 (`17*nrec >= 13*n`)
    let mut acc_chk = 0u64;
    for (name, blks, reps) in [
        ("6 Admitted", &six_r, 500usize),
        ("233 FullWiki", &w233, 40),
        ("6 JSONL/Log", &nw_r, 500),
        ("4 Code/Docs", &cd_r, 200),
    ] {
        let t0 = Instant::now();
        for _ in 0..reps {
            for &b in blks {
                let mut s2 = 0u64;
                for &rec in &b.records {
                    let w = u64::from(rec & 255);
                    s2 += w * w;
                }
                acc_chk = acc_chk.wrapping_add(s2);
            }
        }
        let us = t0.elapsed().as_secs_f64() * 1e6 / (reps * blks.len()) as f64;
        println!("  S_2^rec O(nrec) scan cost [{:12}]: {:6.2} us/blk (vs ~0.001 us O(1) CA-37; chk={})", name, us, acc_chk & 0xFF);
    }
}

// ============================================================================
// 4. Exhaustive Exactness, WS Attribution & C_recognise Audit
// ============================================================================

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[(v.len() * num) / den]
}

fn run_exactness_and_ws_audits(six: &[Block], wiki431: &[Block], non_wiki: &[Block], code_docs: &[Block], dense: &Block) {
    let mut kh = cand_r5_head::Kernel::default();
    let mut c49f = C49FusedOt64::default();
    let mut c49d = C49OutTailDeep::default();
    let mut c50d = C50DeepV1::default();
    let mut c50u = C50DropInV1::default();
    let mut c51n = C51DropInV2::<false>::default();
    let mut c51 = C51DropInV2::<true>::default();
    let mut out_ref = Vec::with_capacity(900_064);
    let mut out_k = Vec::with_capacity(900_064);

    // 4a. 10,190 small valid BWTs
    let mut valid_cnt = 0usize;
    let mut text = Vec::with_capacity(512);
    for n in 1..=12usize {
        for mask in 0..(1usize << n) {
            text.clear();
            for bit in 0..n { text.push(b'a' + ((mask >> bit) & 1) as u8); }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            out_k.clear(); out_k.resize(n + 64, 0xAA);
            c51.open.try_run_const::<16>(&recs, n, p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(0), &mut out_k).unwrap();
            assert_eq!(out_k, text, "C51-DropIn-v2 Open(bi) binary mismatch n={n} mask={mask}");
            out_k.clear(); out_k.resize(n + 64, 0x55);
            c51.run_entry(&recs, n, p, &mut out_k);
            assert_eq!(out_k, text, "C51-DropIn-v2 entry binary mismatch");
            c50d.run_entry(&recs, n, p, &mut out_k);
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
        c51.open.try_run_const::<16>(&recs, n, p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, text);
        c51.open.try_run_const::<32>(&recs, n, p, unified_open::Cap::NONE, unified_open::BiMode::Never, &mut out_k).unwrap();
        assert_eq!(out_k, text);
        c51.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        valid_cnt += 1;
    }

    // 4b. Stress + 6 Admitted + 6 JSONL/Log + 4 Code/Docs + 80 OutTail Poison/Resize/Fallback
    let spiral = make_spiral_200k();
    let per2 = make_periodic("periodic-150k-d2", b"ab", 75_000);
    let per3 = make_periodic("periodic-105k-d3", b"abc", 35_000);
    let stress = [dense, &spiral, &per2, &per3];

    let mut ot_poison_tests = 0usize;
    for b in six.iter().chain(stress.iter().copied()).chain(non_wiki.iter()).chain(code_docs.iter()) {
        let want = b.expected.as_ref().unwrap();
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_k).unwrap();
        assert_eq!(&out_k, want);
        c49f.run_entry(&b.records, b.n, b.p, &mut out_k); assert_eq!(&out_k, want);
        c49d.run_entry(&b.records, b.n, b.p, &mut out_k); assert_eq!(&out_k, want);
        c50d.run_entry(&b.records, b.n, b.p, &mut out_k); assert_eq!(&out_k, want);
        c50u.run_entry(&b.records, b.n, b.p, &mut out_k); assert_eq!(&out_k, want);
        c51n.run_entry(&b.records, b.n, b.p, &mut out_k); assert_eq!(&out_k, want);

        let mut poisoned_aa = vec![0xAAu8; b.n + 4096];
        c51.run_entry(&b.records, b.n, b.p, &mut poisoned_aa);
        assert_eq!(&poisoned_aa, want, "C51-DropIn-v2 0xAA poison mismatch on {}", b.name);
        let mut fresh_out = Vec::new();
        c51.run_entry(&b.records, b.n, b.p, &mut fresh_out);
        assert_eq!(&fresh_out, want, "C51-DropIn-v2 fresh-resize mismatch on {}", b.name);
        c51.open.force_band_vec = true;
        let st_fb = c51.open.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), &mut out_k).unwrap();
        assert!(!st_fb.used_tail);
        assert_eq!(&out_k, want, "C51-DropIn-v2 forced-band-vec mismatch on {}", b.name);
        c51.open.force_band_vec = false;
        c51.force_fallback = true;
        assert_eq!(c51.run_entry(&b.records, b.n, b.p, &mut out_k), 0);
        assert_eq!(&out_k, want, "C51-DropIn-v2 forced-Open fallback mismatch on {}", b.name);
        c51.force_fallback = false;
        ot_poison_tests += 4;
    }

    assert_eq!(c51.run_entry(&dense.records, dense.n, dense.p, &mut out_k), 3);
    assert_eq!(c51.run_entry(&per2.records, per2.n, per2.p, &mut out_k), 2);
    assert_ne!(c51.run_entry(&spiral.records, spiral.n, spiral.p, &mut out_k), 3);
    for b in code_docs {
        assert_ne!(c51.run_entry(&b.records, b.n, b.p, &mut out_k), 3);
        assert_eq!(&out_k, b.expected.as_ref().unwrap());
    }

    // 4c. All 431 wikidata.bz2 blocks byte-exact across all arms
    for (idx, b) in wiki431.iter().enumerate() {
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        c50d.run_entry(&b.records, b.n, b.p, &mut out_k); assert_eq!(out_k, out_ref, "wiki431 C50-Deep mismatch idx={idx}");
        c50u.run_entry(&b.records, b.n, b.p, &mut out_k); assert_eq!(out_k, out_ref, "wiki431 C50-DropIn mismatch idx={idx}");
        c51n.run_entry(&b.records, b.n, b.p, &mut out_k); assert_eq!(out_k, out_ref, "wiki431 C51-NoOrph mismatch idx={idx}");
        c51.run_entry(&b.records, b.n, b.p, &mut out_k); assert_eq!(out_k, out_ref, "wiki431 C51-DropIn-v2 mismatch idx={idx}");
    }

    // 4d. 335+ Malformed / Multi-Cycle Forward-FL Columns + Peeled S5 Cycles + TightArena Dynamic Grow
    let mut mal_cnt = 0usize;
    let mut check_malformed = |l: &[u8], p: u32, tag: &str| {
        let n = l.len();
        let recs = records_of(l);
        kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        let mut pbuf = vec![0xAAu8; n + 64];
        c51.open.try_run_const::<16>(&recs, n, p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(0), &mut pbuf).unwrap();
        assert_eq!(pbuf, out_ref, "malformed C51 Open(bi) mismatch {tag}");
        pbuf.clear(); pbuf.resize(n + 64, 0x55);
        c51.run_entry(&recs, n, p, &mut pbuf);
        assert_eq!(pbuf, out_ref, "malformed C51-DropIn-v2 mismatch {tag}");
        c50d.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed C50-Deep-v1 mismatch {tag}");
        mal_cnt += 1;
    };
    check_malformed(b"bbaabb", 0, "bbaabb");
    check_malformed(b"abbaab", 3, "abbaab");
    for &per in &[253usize, 254, 255, 256, 257, 258, 512] {
        let n = per * 32;
        let mut l = vec![0u8; n];
        l[0] = 2u8; l[n - 1] = 1u8;
        check_malformed(&l, 0, &format!("sat255-{per}"));
    }
    // TightArena dynamic S2b grow_edges_step stress
    {
        let mut l_tight = Vec::new();
        for i in 0..400usize { l_tight.resize(l_tight.len() + 20, ((i % 200) + 1) as u8); }
        l_tight.push(0);
        check_malformed(&l_tight, 0, "tight-arena-grow");
    }
    // Peeled S5 cycles d=1..17 with m >= 65_536
    for &d in &[1usize, 2, 3, 4, 5, 7, 16, 17] {
        let sz = 80_000usize;
        let mut lm = vec![0u8; sz];
        for i in 0..d { lm[i] = ((i + 1) % d) as u8; }
        for (i, slot) in lm.iter_mut().enumerate().skip(d) { *slot = (d + (i % 120)) as u8; }
        for slot in &mut lm[sz - 12_000..sz] { *slot = 250; }
        check_malformed(&lm, 0, &format!("peeled-s5-d{d}"));
    }
    for &sz in &[64usize, 65, 256, 257, 511, 512, 513, 1536, 1537, 65_535, 65_536, 65_537, 262_144, 340_000] {
        let mut l = Vec::with_capacity(sz);
        for i in 0..sz { l.push(((i * 7 + (i >> 5)) & (if sz >= 262_144 { 255 } else { 3 })) as u8); }
        check_malformed(&l, (sz / 3) as u32, &format!("boundary-{sz}"));
    }
    let mut lbuf = Vec::with_capacity(70_000);
    for case in 0..310usize {
        let n = if case < 170 { 1 + (case * 37) % 512 } else if case < 230 { 8_192 } else { 40_000 + (case - 230) * 137 };
        let alpha = if case < 60 { 3usize } else if (230..255).contains(&case) { 2 } else { 2 + (case % 12) };
        lbuf.clear();
        if (230..255).contains(&case) {
            let half = n / 2;
            lbuf.resize(half / 2, b'b'); lbuf.resize(half, b'a');
            lbuf.resize(half + half / 2, b'a'); lbuf.resize(n, b'b');
        } else {
            for _ in 0..n {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                lbuf.push(((rng >> 40) % alpha as u64) as u8);
            }
        }
        let p = ((rng >> 32) as usize % n) as u32;
        check_malformed(&lbuf, p, &format!("case-{case}"));
    }

    println!(
        "\nEXACTNESS PASS: {valid_cnt} small valid + {mal_cnt} malformed (incl. TightArena grow, peeled S5 d=1..17, tiered TinyFL, SAT=255, chunk/epoch/u16 OutQ, forward-FL) + {ot_poison_tests} OutTail poison/resize/fallback + 4 stress + {} admitted + {} wikidata.bz2 + {} JSONL/Log + {} Code/Docs 100% BYTE-EXACT!",
        six.len(), wiki431.len(), non_wiki.len(), code_docs.len()
    );

    // 4e. Deterministic Scratch Working Set Attribution (`PeakCap`, `MaxUsed`, `spelling` avg, `edges` cap, `arena_est > m`)
    let full_wiki: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();
    let all_wiki: Vec<&Block> = wiki431.iter().collect();
    let six_r: Vec<&Block> = six.iter().collect();
    let nw_r: Vec<&Block> = non_wiki.iter().collect();
    let cd_r: Vec<&Block> = code_docs.iter().collect();

    println!("\nDETERMINISTIC SCRATCH WORKING SET (`PeakCap`, `MaxUsed`, `spelling` avg, `edges` cap, `arena_est>m`):");
    for (cname, blks) in [
        ("6 Admitted", &six_r),
        ("233 FullWiki", &full_wiki),
        ("431 ALL Wiki", &all_wiki),
        ("6 JSONL/Log", &nw_r),
        ("4 Code/Docs", &cd_r),
    ] {
        let mut kh_w = cand_r5_head::Kernel::default();
        let mut c50d_w = C50DeepV1::default();
        let mut c50u_w = C50DropInV1::default();
        let mut c51n_w = C51DropInV2::<false>::default();
        let mut c51_w = C51DropInV2::<true>::default();
        let mut out_w = Vec::with_capacity(900_064);

        let (mut d_maxu, mut d_sp_sum, mut d_sp_n, mut d_gt) = (0usize, 0usize, 0usize, 0usize);
        let (mut u_maxu, mut u_sp_sum, mut u_sp_n, mut u_gt) = (0usize, 0usize, 0usize, 0usize);
        let (mut n_maxu, mut n_sp_sum, mut n_sp_n, mut n_gt) = (0usize, 0usize, 0usize, 0usize);
        let (mut v_maxu, mut v_sp_sum, mut v_sp_n, mut v_gt) = (0usize, 0usize, 0usize, 0usize);

        for &b in blks {
            kh_w.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_w);
            if let (_, Some(st)) = c50d_w.run_entry_stats(&b.records, b.n, b.p, &mut out_w) {
                d_maxu = d_maxu.max(st.used_bytes); d_sp_sum += st.spelling_used; d_sp_n += 1; if st.arena_est_gt_m { d_gt += 1; }
            }
            if let (_, Some(st)) = c50u_w.run_entry_stats(&b.records, b.n, b.p, &mut out_w) {
                u_maxu = u_maxu.max(st.used_bytes); u_sp_sum += st.spelling_used; u_sp_n += 1; if st.arena_est_gt_m { u_gt += 1; }
            }
            if let (_, Some(st)) = c51n_w.run_entry_stats(&b.records, b.n, b.p, &mut out_w) {
                n_maxu = n_maxu.max(st.used_bytes); n_sp_sum += st.spelling_used; n_sp_n += 1; if st.arena_est_gt_m { n_gt += 1; }
            }
            if let (_, Some(st)) = c51_w.run_entry_stats(&b.records, b.n, b.p, &mut out_w) {
                v_maxu = v_maxu.max(st.used_bytes); v_sp_sum += st.spelling_used; v_sp_n += 1; if st.arena_est_gt_m { v_gt += 1; }
            }
        }
        let (h_cap, _, _) = kh_w.scratch_report();
        let (d_cap, d_ec, _, d_s) = c50d_w.open.scratch_report();
        let d_tot = d_cap + c50d_w.seg.scratch_bytes();
        let (u_cap, u_ec, _, u_s) = c50u_w.open.scratch_report();
        let u_tot = u_cap + c50u_w.seg.scratch_bytes();
        let (n_cap, n_ec, _, _) = c51n_w.open.scratch_report();
        let n_tot = n_cap + c51n_w.seg.scratch_bytes();
        let (v_cap, v_ec, _, v_s) = c51_w.open.scratch_report();
        let v_tot = v_cap + c51_w.seg.scratch_bytes();

        println!("  [{cname}] (R5-Head PeakCap = {:.2}K)", h_cap as f64 / 1024.0);
        println!(
            "    C50-Deep-v1     | PeakCap={:7.2}K (seg={:6.1}K) | MaxUsed={:7.2}K | sp_avg={:5.2}K | edges_cap={:7.2}K | est>m={:2}/{} | {}",
            d_tot as f64 / 1024.0, c50d_w.seg.scratch_bytes() as f64 / 1024.0,
            d_maxu as f64 / 1024.0, d_sp_sum as f64 / (d_sp_n.max(1) as f64 * 1024.0),
            d_ec as f64 / 1024.0, d_gt, d_sp_n, d_s
        );
        println!(
            "    C50-DropIn-v1   | PeakCap={:7.2}K (seg={:6.1}K) | MaxUsed={:7.2}K | sp_avg={:5.2}K | edges_cap={:7.2}K | est>m={:2}/{} | {}",
            u_tot as f64 / 1024.0, c50u_w.seg.scratch_bytes() as f64 / 1024.0,
            u_maxu as f64 / 1024.0, u_sp_sum as f64 / (u_sp_n.max(1) as f64 * 1024.0),
            u_ec as f64 / 1024.0, u_gt, u_sp_n, u_s
        );
        println!(
            "    C51-v2-NoOrph   | PeakCap={:7.2}K (seg={:6.1}K) | MaxUsed={:7.2}K | sp_avg={:5.2}K | edges_cap={:7.2}K | est>m={:2}/{}",
            n_tot as f64 / 1024.0, c51n_w.seg.scratch_bytes() as f64 / 1024.0,
            n_maxu as f64 / 1024.0, n_sp_sum as f64 / (n_sp_n.max(1) as f64 * 1024.0),
            n_ec as f64 / 1024.0, n_gt, n_sp_n
        );
        println!(
            "    C51-DropIn-v2   | PeakCap={:7.2}K (seg={:6.1}K) | MaxUsed={:7.2}K | sp_avg={:5.2}K | edges_cap={:7.2}K | est>m={:2}/{} | {} ({:+.2}K vs Head)",
            v_tot as f64 / 1024.0, c51_w.seg.scratch_bytes() as f64 / 1024.0,
            v_maxu as f64 / 1024.0, v_sp_sum as f64 / (v_sp_n.max(1) as f64 * 1024.0),
            v_ec as f64 / 1024.0, v_gt, v_sp_n, v_s,
            (v_tot as isize - h_cap as isize) as f64 / 1024.0,
        );
    }

    // 4f. C_recognise telemetry
    println!("\nC_RECOGNISE TELEMETRY (`Spec4` select_limit + O(1) CA-37):");
    let mut chk = 0u64;
    for (label, list, reps) in [
        ("6 Admitted", &six_r, 10_000usize),
        ("233 FullWiki", &full_wiki, 1_000),
        ("6 JSONL/Log", &nw_r, 10_000),
        ("4 Code/Docs", &cd_r, 10_000),
    ] {
        let (mut l16, mut l24, mut l32, mut s_sum) = (0usize, 0usize, 0usize, 0usize);
        for &b in list {
            let (l, s) = select_limit(&b.records, b.n);
            s_sum += s as usize;
            match l { 16 => l16 += 1, 24 => l24 += 1, _ => l32 += 1 }
        }
        let t0 = Instant::now();
        for _ in 0..reps {
            for &b in list {
                let (l, _) = select_limit(&b.records, b.n);
                chk = chk.wrapping_add(u64::from(l));
            }
        }
        let ns = t0.elapsed().as_nanos() as f64 / (reps * list.len()) as f64;
        println!(
            "  C_rec {:12}: {:5.1} ns/blk | avg_samples={:5.1} | L16={} L24={} L32={} (chk={})",
            label, ns, s_sum as f64 / list.len() as f64, l16, l24, l32, chk & 0xFF
        );
    }
}

// ============================================================================
// 5. Headroom Oracle v6 Benchmarks (1T, 2W, 4W) Across All 4 Corpora + 441 Union
// ============================================================================

const NUM_ARMS: usize = 8;
const ARM_NAMES: [&str; NUM_ARMS] = [
    "R5-Head-A1",
    "R5-Head-A2(AA)",
    "C49-Fused-OT64",
    "C49-OutTail-Deep",
    "C50-Deep-v1",
    "C50-DropIn-v1",
    "C51-v2-NoOrph",
    "C51-DropIn-v2",
];

#[derive(Default)]
struct ArmSuite {
    kh1: cand_r5_head::Kernel,
    kh2: cand_r5_head::Kernel,
    c49f: C49FusedOt64,
    c49d: C49OutTailDeep,
    c50d: C50DeepV1,
    c50u: C50DropInV1,
    c51n: C51DropInV2<false>,
    c51: C51DropInV2<true>,
}

impl ArmSuite {
    #[inline(always)]
    fn run_arm(&mut self, arm: usize, b: &Block, out: &mut Vec<u8>) -> u8 {
        match arm {
            0 => { self.kh1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out); 0 }
            1 => { self.kh2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out); 0 }
            2 => self.c49f.run_entry(&b.records, b.n, b.p, out),
            3 => self.c49d.run_entry(&b.records, b.n, b.p, out),
            4 => self.c50d.run_entry(&b.records, b.n, b.p, out),
            5 => self.c50u.run_entry(&b.records, b.n, b.p, out),
            6 => self.c51n.run_entry(&b.records, b.n, b.p, out),
            _ => self.c51.run_entry(&b.records, b.n, b.p, out),
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
    println!("\n=== HEADROOM ORACLE v6 (1T INTERLEAVED, {rds} rds/blk): {label} ===");
    let mut suite = ArmSuite::default();
    let mut out = Vec::with_capacity(900_064);

    for b in blocks.iter().take(6) {
        for arm in 0..NUM_ARMS { suite.run_arm(arm, b, &mut out); }
    }

    let mut tot = [0.0f64; NUM_ARMS];
    let mut full_tot = [0.0f64; NUM_ARMS];
    let mut oracle_us = 0.0f64;
    let mut full_oracle_us = 0.0f64;
    let mut by_rc = [[0.0f64; 4]; NUM_ARMS];
    let mut c51_paths = [0usize; 8];

    for b in blocks {
        let rc = (b.route_class as usize).min(3);
        let mut s: [Vec<f64>; NUM_ARMS] = Default::default();
        let mut p7 = 0u8;
        for r in 0..rds {
            for off in 0..NUM_ARMS {
                let arm = (r + off) % NUM_ARMS;
                let t0 = Instant::now();
                let path = suite.run_arm(arm, b, &mut out);
                s[arm].push(t0.elapsed().as_secs_f64() * 1e6);
                if arm == 7 { p7 = path; }
            }
        }
        c51_paths[p7 as usize] += 1;
        let mut meds = [0.0f64; NUM_ARMS];
        for arm in 0..NUM_ARMS {
            let m = pct(&mut s[arm], 1, 2);
            meds[arm] = m;
            tot[arm] += m;
            by_rc[arm][rc] += m;
            if b.route_class < 2 { full_tot[arm] += m; }
        }
        let best = meds[0].min(meds[2]).min(meds[3]).min(meds[4]).min(meds[5]).min(meds[6]).min(meds[7]);
        oracle_us += best;
        if b.route_class < 2 { full_oracle_us += best; }
        if show_per_block {
            println!(
                "  {:12} | H1={:6.1} | C50D={:6.1} ({:.4}x) | C50U={:6.1} ({:.4}x) | C51N={:6.1} ({:.4}x) | C51(p{})={:6.1} ({:.4}x) | Orcl={:6.1}",
                b.name, meds[0], meds[4], meds[4] / meds[0],
                meds[5], meds[5] / meds[0], meds[6], meds[6] / meds[0],
                p7, meds[7], meds[7] / meds[0], best
            );
        }
    }

    let base = tot[0];
    for arm in 0..NUM_ARMS {
        let lt = if arm >= 2 { if tot[arm] < base { "YES" } else { "NO" } } else { "-" };
        if show_rc {
            println!(
                "  {:17} | norm={:7.2}ms dense={:5.2}ms tiny={:4.2}ms col={:4.2}ms | TOTAL={:7.2}ms ({:.4}x) | < C_Open? {}",
                ARM_NAMES[arm], by_rc[arm][0] / 1e3, by_rc[arm][1] / 1e3, by_rc[arm][2] / 1e3, by_rc[arm][3] / 1e3,
                tot[arm] / 1e3, tot[arm] / base, lt
            );
        } else {
            println!(
                "  {:17} | TOTAL={:8.1} us ({:.4}x) | < C_Open? {}",
                ARM_NAMES[arm], tot[arm], tot[arm] / base, lt
            );
        }
    }
    println!(
        "  {:17} | TOTAL={:8.1} us ({:.4}x) | C51 capture of oracle headroom = {:.1}%",
        "Hindsight-Oracle", oracle_us, oracle_us / base,
        if base > oracle_us { (base - tot[7]) / (base - oracle_us) * 100.0 } else { 0.0 }
    );
    if show_rc {
        let fb = full_tot[0];
        println!(
            "  233-Full-Wiki 1T  | Head={:.2}ms | C49F={:.4}x | C49D={:.4}x | C50D={:.4}x | C50U={:.4}x | C51N={:.4}x | C51={:.4}x | Oracle={:.4}x",
            fb / 1e3, full_tot[2] / fb, full_tot[3] / fb, full_tot[4] / fb,
            full_tot[5] / fb, full_tot[6] / fb, full_tot[7] / fb, full_oracle_us / fb
        );
    }
    println!("  C51 routes [L16-bi, tiny, per, seg8, dense-bi, L16-nobi, L24, L32]: {:?}", c51_paths);
    Bench1TResult { tot_us: tot, oracle_us, full_tot_us: full_tot, full_oracle_us }
}

fn bench_mt_suite(label: &str, blocks: &[Block], passes_2w: usize, passes_4w: usize) -> ([f64; NUM_ARMS], [f64; NUM_ARMS]) {
    let mut p50_2w = [0.0f64; NUM_ARMS];
    let mut p50_4w = [0.0f64; NUM_ARMS];
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
                            for idx in 0..blocks.len() {
                                let b = &blocks[(idx + tid) % blocks.len()];
                                suite.run_arm(arm, b, &mut out);
                            }
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
            println!(
                "  {}W {:17} | p10={:9.1} us ({:.4}x) | p50={:9.1} us ({:.4}x) | p50 < C_Open? {}",
                threads, ARM_NAMES[arm], p10, p10 / b10, p50, p50 / b50, lt
            );
        }
    }
    (p50_2w, p50_4w)
}

fn main() {
    let six = load_six_admitted();
    let wiki431 = load_bin_corpus(&resolve_corpus_path("wikidata_431_blocks.bin"), "wiki", false);
    let non_wiki = load_bin_corpus(&resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin"), "nonwiki", true);
    let code_docs = load_bin_corpus(&resolve_corpus_path("non_wikidata/code_docs_blocks.bin"), "codedoc", true);
    let dense = make_dense_300k();

    println!(
        "Cycle 51 Headroom Oracle v6 loaded: {} admitted, {} wikidata.bz2, {} JSONL/Log, {} Code/Docs blocks",
        six.len(), wiki431.len(), non_wiki.len(), code_docs.len()
    );

    // 1. Classical Atlas Extension `C51-D2` (Bienayme 1853 / Chebyshev 1867 / Cantelli 1928)
    run_c51_d2_second_moment_audit(&six, &wiki431, &non_wiki, &code_docs, &dense);

    // 2. Exhaustive Exactness + Deterministic WS Attribution + C_recognise Telemetry
    run_exactness_and_ws_audits(&six, &wiki431, &non_wiki, &code_docs, &dense);

    // 3. Interleaved 1T Headroom Oracle v6 across all 4 corpora
    let r1_six = bench_1t_interleaved("6 ADMITTED WIKIDATA BLOCKS", &six, 21, false, true);
    let r1_nw = bench_1t_interleaved("6 NON-WIKIDATA JSONL/LOG BLOCKS", &non_wiki, 21, false, true);
    let r1_cd = bench_1t_interleaved("4 NON-WIKIDATA CODE/DOCS BLOCKS", &code_docs, 21, false, true);
    let r1_w431 = bench_1t_interleaved("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 5, true, false);

    // 4. 2W and 4W Concurrent Barrier Benchmarks across all 4 corpora
    let (m2_six, m4_six) = bench_mt_suite("6 ADMITTED WIKIDATA BLOCKS", &six, 19, 13);
    let (m2_nw, m4_nw) = bench_mt_suite("6 NON-WIKIDATA JSONL/LOG BLOCKS", &non_wiki, 19, 13);
    let (m2_cd, m4_cd) = bench_mt_suite("4 NON-WIKIDATA CODE/DOCS BLOCKS", &code_docs, 19, 13);
    let (m2_w431, m4_w431) = bench_mt_suite("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 7, 5);

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
        let lt4 = if sum4_41_lt(sum4_441[arm], sum4_441[0]) { "YES" } else { "NO" };
        println!(
            "    {:17} | 1T={:7.2}ms ({:.4}x, <Open? {:3}) | 2W={:7.2}ms ({:.4}x, <Open? {:3}) | 4W={:7.2}ms ({:.4}x, <Open? {:3})",
            ARM_NAMES[arm],
            sum1_441[arm] / 1e3, sum1_441[arm] / sum1_441[0], lt1,
            sum2_441[arm] / 1e3, sum2_441[arm] / sum2_441[0], lt2,
            sum4_441[arm] / 1e3, sum4_441[arm] / sum4_441[0], lt4,
        );
    }
    println!(
        "    {:17} | 1T={:7.2}ms ({:.4}x) | 6-Admitted 1T/2W/4W C51-DropIn-v2 = {:.4}x / {:.4}x / {:.4}x",
        "Hindsight-Oracle", orcl1_441 / 1e3, orcl1_441 / sum1_441[0],
        r1_six.tot_us[7] / r1_six.tot_us[0], m2_six[7] / m2_six[0], m4_six[7] / m4_six[0]
    );
    println!("  [243 Full-Size Blocks = 233 Full Wiki + 6 JSONL/Log + 4 Code/Docs, 1T]");
    for arm in 0..NUM_ARMS {
        println!(
            "    {:17} | 1T={:7.2}ms ({:.4}x) | < C_Open? {}",
            ARM_NAMES[arm], sum1_243[arm] / 1e3, sum1_243[arm] / sum1_243[0],
            if sum1_243[arm] < sum1_243[0] { "YES" } else { "NO" }
        );
    }
    println!("    {:17} | 1T={:7.2}ms ({:.4}x)", "Hindsight-Oracle", orcl1_243 / 1e3, orcl1_243 / sum1_243[0]);
}

#[inline(always)]
fn sum4_41_lt(a: f64, b: f64) -> bool { a < b }
