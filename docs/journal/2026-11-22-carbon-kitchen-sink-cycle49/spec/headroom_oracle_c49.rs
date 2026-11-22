//! Cycle 49 Headroom Oracle v4 (4-Corpus `1T / 2W / 4W` Benchmark, Persistent WS & Exactness Suite).
//!
//! Compares all 6 core arms (+ A/A paired control = 7 arms) across 4 real corpora + 441-block union:
//!   1. `6` Admitted Wikidata blocks (`/home/antigravity/admitted-bwt/`)
//!   2. `431` `wikidata.bz2` blocks (`233` full + `198` tiny/column)
//!   3. `6` Non-Wikidata JSONL/Log blocks (`5` JSONL + `1` syslog)
//!   4. `4` Non-Wikidata Code/Docs blocks (`code_and_docs.bz2` -> `code_docs_blocks.bin`)
//!   5. `441`-block non-overlapping union (`431` Wiki + `6` JSONL/Log + `4` Code/Docs)
//!
//! Arms:
//!   - `R5-Head-A1` / `R5-Head-A2(AA)` (verbatim `cand_r5_head`, `ID_BITS=22`, `L=16`, `BiMode::AlwaysAt(65_536)`)
//!   - `R5-Nobi` (verbatim `cand_r5_nobi`, `ID_BITS=22`, `L=16`, `μ=1`)
//!   - `F-Inline3` (`CS-06`: `ID_BITS=24`, `SATURATED=255`, `RULE_BASE=0`, `L=16`, `AlwaysAt(65_536)`)
//!   - `I3+OT64` (Cycle 48 `CS-09`: `F-Inline3 + S3a-OutTail64 + TrimWS`, `L=16`, `AlwaysAt(65_536)`)
//!   - `C48-Fused` (Cycle 48 `CS-12`: `F-Inline3 + PlainRunWalk + PostS1ExactMd + 2-stage L-selector + recs/n>=0.28`)
//!   - `C49-Fused-OT64` (Cycle 49 Super-Candidate: `F-Inline3 + S3a-OutTail64 + TrimWS + Zero-WS PlainRunWalk +
//!     PostS1ExactMd + 2-stage L-selector + Principled Markov-MacMahon 17*nrec>=13*n & post-S1 8*m+2*r_thin>9*n`)
//!
//! Classical Foundations (Weiss-audited, ZERO relabels):
//!   - `CA-05` Cauchy (1815, 1844): exact forward-`FL` disjoint cycle closure & periodic fast path
//!   - `CA-26` Gauss (1809): OLS cost laws for `μ ∈ {1,2}`, `L ∈ {16,24,32}`, and `Open` vs `Seg8-FL`
//!   - `CA-27` Chebyshev (1867) / Markov (1884) & `CA-31` MacMahon (1915) / de Moivre (1730):
//!     exact composition-slack identity `2*(r_max - ns) - m = w1 - S2 - 2*delta_p <= w1`
//!   - `CA-32` (C49 extension of `CA-27 -> CA-31`): exact pre-`S1` Markov-MacMahon lower bound
//!     `m/n >= (17*(nrec/n) - 1)/16` and `3*m + r_thin >= (17*nrec - n)/4` (`L = 16 <= 63`).
//!
//! Safe Rust: `#![forbid(unsafe_code)]`, strictly forward-`FL` exact on ALL valid & malformed inputs.
#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_nobi/mod.rs"]
mod cand_r5_nobi;

use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::time::Instant;

// ============================================================================
// 1. Unified Const-Generic Open<F-Inline3, OUTTAIL> Kernel
//    - `OUTTAIL = false`: `F-Inline3` / `C48-Fused` (`self.bands` + fixed reserves)
//    - `OUTTAIL = true`:  `I3+OT64` / `C49-Fused-OT64` (`S3a-OutTail64 + TrimWS`)
// ============================================================================

#[allow(dead_code)]
mod unified_open {
    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const NARROW_NODES: usize = 1 << 16;
    const RULE: usize = 4;
    const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;

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
        pub const C49_CROSSOVER: Cap = Cap {
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
        pub r_max: u32,
        pub extra_127: u32,
        pub used_bi: bool,
        pub used_tail: bool,
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

    fn begin_inline<const TRIM: bool>(
        words_hint: usize,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
    ) {
        if inline16.len() < 256 {
            inline16.clear();
            if TRIM {
                inline16.reserve_exact(256);
            }
            inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            lens.clear();
            if TRIM {
                lens.reserve_exact(256);
            }
            lens.resize(256, 1);
        }
        inline16.truncate(256);
        lens.truncate(256);
        if !TRIM {
            inline16.reserve_exact(words_hint);
            lens.reserve_exact(words_hint);
        }
    }

    #[cold]
    #[inline(never)]
    fn grow_step_u8(v: &mut Vec<u8>, need: usize, step: usize) {
        let target = ((need + step - 1) / step) * step;
        v.reserve_exact(target - v.len());
    }

    #[inline(always)]
    fn reserve_step_u8<const TRIM: bool>(v: &mut Vec<u8>, add: usize, step: usize) {
        if TRIM {
            let need = v.len() + add;
            if need > v.capacity() {
                grow_step_u8(v, need, step);
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn grow_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
        let target = ((lens.len() + 1 + step - 1) / step) * step;
        inline16.reserve_exact(target - inline16.len());
        lens.reserve_exact(target - lens.len());
    }

    #[inline(always)]
    fn reserve_step_inline<const TRIM: bool>(
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        step: usize,
    ) {
        if TRIM && lens.len() == lens.capacity() {
            grow_step_inline(inline16, lens, step);
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(a: &mut Vec<u32>, need: usize) {
        a.resize(need.max(2 * a.len()), 0);
    }

    mod outq {
        const CHUNK: usize = 512;
        pub(super) const BYTES: usize = 2 * CHUNK;
        const NONE: u32 = u32::MAX;
        const EPOCH: u32 = 1 << 16;

        #[derive(Default)]
        pub(super) struct Spill {
            next: Vec<u32>,
            ovf: Vec<[u8; BYTES]>,
            cuts: Vec<[u32; 256]>,
        }

        impl Spill {
            pub(super) fn scratch_bytes(&self) -> usize {
                4 * self.next.capacity()
                    + BYTES * self.ovf.capacity()
                    + 1024 * self.cuts.capacity()
            }
        }

        pub(super) struct Queues<'a> {
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
        }

        impl<'a> Queues<'a> {
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
                        self.spill.cuts.push(cut);
                    }
                }
            }
            fn take_chunk_pt(&mut self, a: usize) {
                let id = self.chunks;
                self.chunks += 1;
                if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                    self.spill.ovf.push([0; BYTES]);
                }
                if self.spill.next.len() <= id as usize {
                    self.spill.next.resize(id as usize + 1, NONE);
                }
                self.spill.next[id as usize] = NONE;
                let hi = self.pt[a] & !0xFFFF_FFFFu64;
                if self.heads[a] == NONE {
                    self.heads[a] = id;
                } else {
                    self.spill.next[((self.pt[a] as u32) >> 10) as usize] = id;
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

        pub(super) struct Reader<'a> {
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
                        self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                    }
                    if self.at == CHUNK {
                        self.id = self.spill.next[self.id as usize];
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
                    for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
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
            pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {
                let span = dst.len();
                if self.at + span <= CHUNK
                    && self.read + span as u32 <= self.cut
                    && self.id < self.in_out
                {
                    let hi = pack | (self.epoch << 16);
                    let at = self.id as usize * BYTES + 2 * self.at;
                    for (e, s) in dst.iter_mut().zip(self.out[at..at + 2 * span].chunks_exact(2)) {
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
                        self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                    }
                    if self.at == CHUNK {
                        self.id = self.spill.next[self.id as usize];
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
                    for (e, s) in now.iter_mut().zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2)) {
                        *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
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
        fn inline_word<const TRIM: bool>(
            &mut self,
            word: u32,
            byte: u8,
            stack: &mut Vec<u32>,
            spelling: &mut Vec<u8>,
        ) -> ([u8; INLINE], u8) {
            let height = (word >> ID_BITS) as usize;
            if height < INLINE {
                let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
                (v.to_le_bytes(), (height + 1) as u8)
            } else {
                let start = spelling.len() as u32;
                self.append::<TRIM>(word, stack, spelling);
                reserve_step_u8::<TRIM>(spelling, 1, 8192);
                spelling.push(byte);
                (long_slot(start, spelling.len() as u32), LONG)
            }
        }
        fn append<const TRIM: bool>(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
            stack.clear();
            stack.push(handle);
            while let Some(h) = stack.pop() {
                let g = h >> ID_BITS;
                if g < INLINE as u32 {
                    let bytes = self.short_payload_h(h).to_le_bytes();
                    reserve_step_u8::<TRIM>(dest, g as usize, 8192);
                    dest.extend_from_slice(&bytes[..g as usize]);
                } else {
                    let r = self.rule(h & ID_MASK);
                    let tag = self.a[r + 3];
                    let rule = &mut self.a[r..r + 4];
                    if tag == u32::MAX {
                        rule[3] = 0x8000_0000 | dest.len() as u32;
                        stack.push(rule[1]);
                        stack.push(rule[0]);
                    } else {
                        let start = (tag & 0x7fff_ffff) as usize;
                        let rlen = rule[2] as usize;
                        reserve_step_u8::<TRIM>(dest, rlen, 8192);
                        dest.extend_from_within(start..start + rlen);
                    }
                }
            }
        }
    }

    struct Dictionary<'a, const TRIM: bool> {
        pairs: &'a mut [u32; 512],
        contents: &'a mut [u32; 2048],
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        spelling: &'a mut Vec<u8>,
        stack: &'a mut Vec<u32>,
    }

    impl<const TRIM: bool> Dictionary<'_, TRIM> {
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
            let memoized = e.a[e.rule(id) + 3] != u32::MAX;
            e.append::<TRIM>(handle, self.stack, self.spelling);
            reserve_step_u8::<TRIM>(self.spelling, 1, 8192);
            self.spelling.push(byte);
            let entry = long_slot(start as u32, self.spelling.len() as u32);
            let bytes = &self.spelling[start..];
            let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
            let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
                ^ part(bytes.len() - 8)
                    .rotate_left(29)
                    .wrapping_mul(0x94D0_49BB_1331_11EB)
                ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
                ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let h = h ^ (h >> 30);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes
            } {
                if memoized {
                    self.spelling.truncate(start);
                } else {
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
            reserve_step_inline::<TRIM>(self.inline16, self.lens, 256);
            let code = self.lens.len() as u32;
            self.inline16.push(entry);
            self.lens.push(len);
            code
        }
    }

    struct Pass<'a> {
        queues: outq::Queues<'a>,
        edges: &'a mut Vec<u32>,
        ns: usize,
        cap_strips: usize,
        f: First,
    }

    impl Pass<'_> {
        #[inline(always)]
        fn take<const LIM: u32>(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            if width <= LIM {
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
                    let thin_a = self.queues.thin_of(a);
                    self.edges[at..at + 3].copy_from_slice(&[
                        x,
                        thin_a + f.thick_rows[a],
                        (w << 8) | u32::from(c),
                    ]);
                }
                self.ns += 1;
                f.thick[a] += 1;
                f.thick_rows[a] += w;
            }
        }
    }

    #[derive(Default)]
    pub struct Kernel<const OUTTAIL: bool> {
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

    impl<const OUTTAIL: bool> Kernel<OUTTAIL> {
        pub fn scratch_report(&self) -> (usize, usize, String) {
            let spill_b = self.spill.scratch_bytes();
            let bands_b = 8 * self.bands.capacity();
            let spell_b = self.spelling.capacity();
            let edges_b = 4 * self.edges.capacity();
            let extra_b = 4 * self.extra.capacity();
            let inl_b = INLINE * self.inline16.capacity() + self.lens.capacity();
            let stk_b = 4 * self.stack.capacity();
            let total = spill_b + bands_b + spell_b + edges_b + extra_b + inl_b + stk_b;
            let s = format!(
                "edges={:.1}K bands={:.2}K spell={:.1}K inl={:.1}K spill={:.1}K stk={:.1}K",
                edges_b as f64 / 1024.0,
                bands_b as f64 / 1024.0,
                spell_b as f64 / 1024.0,
                inl_b as f64 / 1024.0,
                spill_b as f64 / 1024.0,
                stk_b as f64 / 1024.0,
            );
            (total, inl_b, s)
        }

        /// Zero-WS tiny-block forward-`FL` walk (`n < 32_768`) reusing `self.edges` (`+0 B` WS!).
        #[inline(never)]
        pub fn invert_tiny_fl(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
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
            let mut starts = [0u32; 256];
            let mut tot = 0u32;
            for c in 0..256 {
                starts[c] = tot;
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
                let s0 = starts[b] as usize;
                starts[b] += len as u32;
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
                let mut pass = Pass {
                    queues: outq::Queues::new(&mut out[..], &mut self.spill, cap.m),
                    edges: &mut self.edges,
                    ns: 0,
                    cap_strips: cap.strips,
                    f: First::default(),
                };
                let mut pos = 0u32;
                let mut extra_127 = 0u32;
                let mut i = 0usize;
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
            Some(self.finish(&f, ns, n, r_max, extra_127, &heads, bi_mode, out))
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn finish(
            &mut self,
            f: &First,
            ns: usize,
            n: usize,
            r_max: u32,
            extra_127: u32,
            heads: &outq::Heads,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> Counts {
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            let first = starts(all);
            let mut write = starts(f.thick);
            let need = Self::arena_words(ns, f.m as usize);
            if self.edges.len() < need {
                let add = need - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(need, 0);
            }
            self.bands.clear();
            self.spelling.clear();
            if !OUTTAIL {
                if n >= 65_536 && self.bands.capacity() < 8192 {
                    self.bands.reserve_exact(8192);
                }
                if n >= 65_536 && self.spelling.capacity() < 48 * 1024 {
                    self.spelling.reserve_exact(48 * 1024);
                }
            }

            let tail_offset = if OUTTAIL {
                let mut chunks_taken = 0usize;
                for a in 0..256 {
                    let ta = f.thin[a] as usize;
                    if ta > 0 {
                        chunks_taken += (ta + 511) >> 9;
                    }
                }
                let in_out_chunks = out.len() / outq::BYTES;
                chunks_taken.min(in_out_chunks) * outq::BYTES
            } else {
                out.len()
            };

            let most = Self::nodes_most(ns);
            let (cursors, used_tail, tail_end, rules) = if most > NARROW_NODES {
                self.induced_with::<Wide>(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    &mut out[tail_offset..],
                )
            } else {
                self.induced_with::<Narrow>(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    &mut out[tail_offset..],
                )
            };

            let m = f.m;
            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
            if self.edges.len() < m as usize {
                self.edges.resize(m as usize, 0);
            }
            self.extra.clear();
            if wide {
                self.extra.resize(m as usize, 0);
            }
            self.predecessor = None;
            let use_xor = !wide && bi_mode.should_use_bi(m, ns as u32, r_max);
            let (out_q, out_tail) = out.split_at(tail_offset);

            if use_xor {
                let pred = if used_tail {
                    self.wire_bi_tail(
                        m as usize, f.mark, f.mark_byte, bits, &cursors, tail_end, out_tail, out_q, heads,
                    )
                } else {
                    self.wire_bi_vec(m as usize, f.mark, f.mark_byte, bits, &cursors, out_q, heads)
                };
                self.predecessor = Some(pred);
            } else if used_tail {
                self.wire_uni_tail(m as usize, wide, bits, &cursors, tail_end, out_tail, out_q, heads);
            } else {
                self.wire_uni_vec(m as usize, wide, bits, &cursors, out_q, heads);
            }

            if let Some(before) = self.predecessor {
                emit_bidirectional(
                    &self.edges[..m as usize], &self.spelling, &self.inline16, &self.lens,
                    n, f.mark, before, bits, out,
                );
            } else {
                emit(
                    &self.edges[..m as usize], &self.extra, &self.spelling, &self.inline16, &self.lens,
                    n, f.mark, bits, out,
                );
            }
            Counts {
                retained: m as usize,
                thick: ns,
                rules,
                r_max,
                extra_127,
                used_bi: use_xor,
                used_tail,
            }
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_with<L: Links>(
            &mut self,
            ns: usize,
            n: u32,
            m: u32,
            thin: &[u32; 256],
            first: &[u32; 256],
            write: &mut [u32; 256],
            most: usize,
            out_tail: &mut [u8],
        ) -> ([usize; 256], bool, usize, usize) {
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
                grow(&mut self.edges, 4 * ns + top + SPIRAL_RULES * RULE);
            }
            let rules = (top - base) / RULE;
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
            begin_inline::<OUTTAIL>(if n >= 65_536 { 1536 } else { 0 }, &mut self.inline16, &mut self.lens);

            let max_bands = open_nodes + 256;
            let use_tail = OUTTAIL && !self.force_band_vec && out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut tail_pos = 0usize;

            if cache.len() < CACHE_WORDS {
                let (mut id, mut in_node, mut k) = (L::next(e.a, 0, 0), 0, 0);
                for a in 0..256 {
                    cursors[a] = if use_tail { tail_pos } else { self.bands.len() };
                    let end = k + thin[a];
                    while k < end {
                        let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                        let width = (end - k).min(node_width - in_node);
                        let dst = node_bottom + in_node;
                        let code = if node_word == 0 {
                            a as u32
                        } else {
                            let (entry, len) = e.inline_word::<OUTTAIL>(node_word, a as u8, &mut self.stack, &mut self.spelling);
                            reserve_step_inline::<OUTTAIL>(&mut self.inline16, &mut self.lens, 256);
                            let c = self.lens.len() as u32;
                            self.inline16.push(entry); self.lens.push(len); c
                        };
                        if use_tail {
                            out_tail[tail_pos..tail_pos + 8].copy_from_slice(&Band::new(width, dst, code).0.to_le_bytes());
                            tail_pos += 8;
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
                let mut dictionary = Dictionary::<OUTTAIL> {
                    pairs: p_sl.try_into().unwrap(),
                    contents: c_sl.try_into().unwrap(),
                    inline16: &mut self.inline16,
                    lens: &mut self.lens,
                    spelling: &mut self.spelling,
                    stack: &mut self.stack,
                };
                let mut id = L::next(e.a, 0, 0);
                let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
                    L::band_node(e.a, id)
                } else {
                    (0, 0, 0, 0)
                };
                let (mut in_node, mut k) = (0u32, 0u32);
                if use_tail {
                    let mut chunks = out_tail[..max_bands * 8].chunks_exact_mut(8);
                    for a in 0..256 {
                        cursors[a] = tail_pos;
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
                                let c = dictionary.code(&mut e, node_word, a as u8);
                                last_w = node_word; last_c = c; c
                            };
                            chunks.next().unwrap().copy_from_slice(&Band::new(width, dst, code).0.to_le_bytes());
                            tail_pos += 8; in_node += width; k += width;
                            if in_node == node_width {
                                id = next_id; in_node = 0;
                                if id != 0 { (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id); }
                            }
                        }
                    }
                } else {
                    for a in 0..256 {
                        cursors[a] = self.bands.len();
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
                                let c = dictionary.code(&mut e, node_word, a as u8);
                                last_w = node_word; last_c = c; c
                            };
                            self.bands.push(Band::new(width, dst, code));
                            in_node += width; k += width;
                            if in_node == node_width {
                                id = next_id; in_node = 0;
                                if id != 0 { (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id); }
                            }
                        }
                    }
                }
                assert_eq!(id, 0);
            }
            (cursors, use_tail, tail_pos, rules)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_tail(&mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, cursors: &[usize; 256], tail_end: usize, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads) -> u32 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let mut predecessor = mark;
            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(tail_end);
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                let sl = &out_tail[cursors[a]..end];
                if a != mb {
                    for ch in sl.chunks_exact(8) {
                        let band = Band(u64::from_le_bytes(ch.try_into().unwrap()));
                        queue.wire_xor_nomark(edges, band.dst() as usize, band.end() as usize, band.code() << bits);
                    }
                } else {
                    for ch in sl.chunks_exact(8) {
                        let band = Band(u64::from_le_bytes(ch.try_into().unwrap()));
                        queue.wire_xor(edges, band.dst() as usize, band.end() as usize, band.code() << bits, mark, &mut predecessor);
                    }
                }
            }
            predecessor
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_vec(&mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, cursors: &[usize; 256], out_q: &[u8], heads: &outq::Heads) -> u32 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let mut predecessor = mark;
            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
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
        fn wire_uni_tail(&mut self, m: usize, wide: bool, bits: u32, cursors: &[usize; 256], tail_end: usize, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads) {
            let edges = &mut self.edges[..m];
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(tail_end);
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                for ch in out_tail[cursors[a]..end].chunks_exact(8) {
                    let b = Band(u64::from_le_bytes(ch.try_into().unwrap()));
                    let (span, dst, bcode) = (b.end() as usize, b.dst() as usize, b.code());
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide { self.extra[dst..dst + span].fill(bcode); }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_vec(&mut self, m: usize, wide: bool, bits: u32, cursors: &[usize; 256], out_q: &[u8], heads: &outq::Heads) {
            let edges = &mut self.edges[..m];
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
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
    fn emit_bidirectional(edges: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, before: u32, bits: u32, out: &mut Vec<u8>) {
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let (mut lo, mut hi) = (0, n);
        for pair in 0..=edges.len() / 2 {
            if lo == hi || (pair != 0 && front == next) { break; }
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
            if single { break; }
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
// 2. Corpus Loaders, Synthetic BWT Generators & Reference Forward-FL Walks
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    route_class: u8,
    records: Vec<u16>,
    expected: Option<Vec<u8>>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
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
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
        }
        l.push(text[(pos + n - 1) % n]);
    }
    (l, p)
}

fn plain_invert_fl(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
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
    if Path::new(&p1).exists() {
        return p1;
    }
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
        if let Some(v) = t.strip_prefix("\"file\": \"") {
            file = v.trim_end_matches("\",").to_string();
        }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            let p = v.trim_end_matches(',').parse::<u32>().expect("parse p");
            specs.push((file.clone(), p));
        }
    }
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = fs::read(format!("{dir}/{f}")).expect("read .L");
            let expected = plain_invert_fl(&l, p);
            Block {
                name: f,
                n: l.len(),
                p,
                route_class: 0,
                records: records_of(&l),
                expected: Some(expected),
            }
        })
        .collect()
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
        } else {
            None
        };
        blocks.push(Block {
            name: format!("{prefix}-{idx}"),
            n,
            p,
            route_class,
            records,
            expected,
        });
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
            let ca = text[(ia + k) % n];
            let cb = text[(ib + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
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
    l[0] = b'c';
    l[n - 1] = b'b';
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
            let ca = pat[(a + k) % pat.len()]; let cb = pat[(b + k) % pat.len()];
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
struct PlainRunWalk {
    fl: Vec<u32>,
}

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
            let b = (rec >> 8) as usize; let len = (rec & 255) as usize;
            let s0 = starts[b] as usize; starts[b] += len as u32;
            let mut i = pos as u32; let bu = b as u32;
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
    #[inline(never)]
    fn invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        assert!(self.try_invert(records, n, p, out));
    }
}

#[derive(Default)]
struct InterleavedWalk {
    plain: PlainRunWalk,
    start_bits: Vec<u64>,
    segs: [Vec<u8>; 8],
}

impl InterleavedWalk {
    #[inline(never)]
    fn try_invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if n == 0 || (p as usize) >= n { return false; }
        if n < 4096 || records.len() == 1 { return self.plain.try_invert(records, n, p, out); }
        if !self.plain.build_fl(records, n) { return false; }
        let fl = &self.plain.fl[..n];
        let words = (n + 63) / 64;
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
        let b = (rec >> 8) as u8; let l = u32::from(rec & 255);
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
    } else {
        (16, 256)
    }
}

const POST_S1_BI: unified_open::BiMode = unified_open::BiMode::PostS1ExactMd {
    r_max_cap: 50_000,
    ns_max: 2_100,
    md_max: 16_000,
    md_neg_cap: -18_000,
    ns_neg_max: 2_400,
};

#[derive(Default)]
struct C48Fused {
    open: unified_open::Kernel<false>,
    seg: InterleavedWalk,
    force_fallback: bool,
}

impl C48Fused {
    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if self.force_fallback {
            self.open
                .try_run_const::<16>(records, n, p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), out)
                .expect("fallback");
            return 0;
        }
        if n < 32_768 { self.seg.plain.invert(records, n, p, out); return 1; }
        if records.len() * 64 < n && try_short_periodic(records, n, p, out) { return 2; }
        if records.len() * 25 >= n * 7 { self.seg.invert(records, n, p, out); return 3; }
        let (lim, _) = select_limit(records, n);
        let cap = if records.len() * 20 >= n * 3 { unified_open::Cap::dense(n) } else { unified_open::Cap::NONE };
        let res = match lim {
            16 => self.open.try_run_const::<16>(records, n, p, cap, POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, cap, POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, cap, POST_S1_BI, out),
        };
        if let Some(st) = res {
            return match (lim, st.used_bi) { (16, true) => 0, (16, false) => 5, (24, _) => 6, _ => 7 };
        }
        self.seg.invert(records, n, p, out);
        3
    }
}

#[derive(Default)]
struct C49FusedOt64 {
    open: unified_open::Kernel<true>,
    seg: InterleavedWalk,
    force_fallback: bool,
}

impl C49FusedOt64 {
    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if self.force_fallback {
            self.open
                .try_run_const::<16>(records, n, p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), out)
                .expect("fallback");
            return 0;
        }
        if n < 32_768 {
            assert!(self.open.invert_tiny_fl(records, n, p, out));
            return 1;
        }
        if records.len() * 64 < n && try_short_periodic(records, n, p, out) {
            return 2;
        }
        // Exact pre-S1 Markov-MacMahon sound fast-reject (CA-32, L=16 <= 63):
        // `17 * nrec >= 13 * n` unconditionally guarantees `3*m + r_thin >= 3*n`.
        if records.len() * 17 >= n * 13 {
            self.seg.invert(records, n, p, out);
            return 3;
        }
        let (lim, _) = select_limit(records, n);
        let res = match lim {
            16 => self.open.try_run_const::<16>(records, n, p, unified_open::Cap::C49_CROSSOVER, POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, unified_open::Cap::C49_CROSSOVER, POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, unified_open::Cap::C49_CROSSOVER, POST_S1_BI, out),
        };
        if let Some(st) = res {
            return match (lim, st.used_bi) { (16, true) => 0, (16, false) => 5, (24, _) => 6, _ => 7 };
        }
        self.seg.invert(records, n, p, out);
        3
    }
}

// ============================================================================
// 3. Exactness, OutTail Poison/Fallback, CA-31/CA-32 Bound & Persistent WS Audit
// ============================================================================

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[(v.len() * num) / den]
}

fn exact_stats(b: &Block) -> (u32, u32, u32, u32, u32, u32) {
    // (w1, r_max, extra_127, m, ns, r_thin)
    let (mut w1, mut r_max, mut extra_127, mut m, mut ns, mut r_thin) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
    let mut pos = 0u32;
    let mut i = 0usize;
    while i < b.records.len() {
        let rec = b.records[i];
        let c = (rec >> 8) as u8;
        let mut w = u32::from(rec & 255);
        i += 1;
        if w == 127 {
            while i < b.records.len() && (b.records[i] >> 8) as u8 == c {
                w += u32::from(b.records[i] & 255);
                extra_127 += 1;
                i += 1;
            }
        }
        r_max += 1;
        if w <= 16 {
            m += w; r_thin += 1; if w == 1 { w1 += 1; }
        } else if b.p >= pos && b.p < pos + w {
            if pos < b.p { ns += 1; }
            m += 1; r_thin += 1; w1 += 1;
            if b.p + 1 < pos + w { ns += 1; }
        } else {
            ns += 1;
        }
        pos += w;
    }
    (w1, r_max, extra_127, m, ns, r_thin)
}

fn run_exactness_and_audits(six: &[Block], wiki431: &[Block], non_wiki: &[Block], code_docs: &[Block]) {
    let mut kh = cand_r5_head::Kernel::default();
    let mut kn = cand_r5_nobi::Kernel::default();
    let mut ki3 = unified_open::Kernel::<false>::default();
    let mut kot = unified_open::Kernel::<true>::default();
    let mut c48 = C48Fused::default();
    let mut c49 = C49FusedOt64::default();
    let mut seg8 = InterleavedWalk::default();
    let mut out_ref = Vec::with_capacity(900_064);
    let mut out_k = Vec::with_capacity(900_064);

    // 3a. 10,190 small valid BWTs (8,190 all binary n=1..=12 + 2,000 random n=1..=512 across 7 alphabets)
    let mut valid_cnt = 0usize;
    let mut text = Vec::with_capacity(512);
    for n in 1..=12usize {
        for mask in 0..(1usize << n) {
            text.clear();
            for bit in 0..n { text.push(b'a' + ((mask >> bit) & 1) as u8); }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            out_k.clear(); out_k.resize(n + 64, 0xAA);
            kot.try_run_const::<16>(&recs, n, p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(0), &mut out_k).unwrap();
            assert_eq!(out_k, text, "I3+OT64 binary bi mismatch n={n} mask={mask}");
            out_k.clear(); out_k.resize(n + 64, 0xAA);
            c49.run_entry(&recs, n, p, &mut out_k);
            assert_eq!(out_k, text, "C49-Fused-OT64 binary mismatch");
            c48.run_entry(&recs, n, p, &mut out_k);
            assert_eq!(out_k, text, "C48-Fused binary mismatch");
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
        kot.try_run_const::<16>(&recs, n, p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, text);
        kot.try_run_const::<32>(&recs, n, p, unified_open::Cap::NONE, unified_open::BiMode::Never, &mut out_k).unwrap();
        assert_eq!(out_k, text);
        c48.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        c49.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        valid_cnt += 1;
    }

    // 3b. Stress + 6 Admitted + 6 JSONL/Log + 4 Code/Docs + OutTail Poison/Resize + Forced Fallbacks
    let dense = make_dense_300k();
    let spiral = make_spiral_200k();
    let per2 = make_periodic("periodic-150k-d2", b"ab", 75_000);
    let per3 = make_periodic("periodic-105k-d3", b"abc", 35_000);
    let stress = [&dense, &spiral, &per2, &per3];

    let mut ot_poison_tests = 0usize;
    for b in six.iter().chain(stress.iter().copied()).chain(non_wiki.iter()).chain(code_docs.iter()) {
        let want = b.expected.as_ref().unwrap();
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_k).unwrap();
        assert_eq!(&out_k, want, "R5-Head mismatch on {}", b.name);
        kn.run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, &mut out_k).unwrap();
        assert_eq!(&out_k, want, "R5-Nobi mismatch on {}", b.name);
        ki3.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), &mut out_k).unwrap();
        assert_eq!(&out_k, want, "F-Inline3 mismatch on {}", b.name);

        // OutTail64 poison + resize exactness gate: pre-fill out with 0xAA, also test shrunk Vec::new()
        let mut poisoned = vec![0xAAu8; b.n + 4096];
        kot.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), &mut poisoned).unwrap();
        assert_eq!(&poisoned, want, "I3+OT64 poisoned mismatch on {}", b.name);
        let mut fresh_out = Vec::new();
        kot.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), &mut fresh_out).unwrap();
        assert_eq!(&fresh_out, want, "I3+OT64 fresh-resize mismatch on {}", b.name);
        // Forced band-vec fallback check
        kot.force_band_vec = true;
        let st_fb = kot.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), &mut out_k).unwrap();
        assert!(!st_fb.used_tail);
        assert_eq!(&out_k, want, "I3+OT64 forced-band-vec mismatch on {}", b.name);
        kot.force_band_vec = false;
        ot_poison_tests += 3;

        c48.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(&out_k, want, "C48-Fused mismatch on {}", b.name);
        let mut poisoned_c49 = vec![0xAAu8; b.n + 2048];
        c49.run_entry(&b.records, b.n, b.p, &mut poisoned_c49);
        assert_eq!(&poisoned_c49, want, "C49-Fused-OT64 poisoned mismatch on {}", b.name);

        // Forced fallback on C49-Fused-OT64
        c49.force_fallback = true;
        assert_eq!(c49.run_entry(&b.records, b.n, b.p, &mut out_k), 0);
        assert_eq!(&out_k, want, "C49-Fused-OT64 forced fallback mismatch on {}", b.name);
        c49.force_fallback = false;
    }

    // Verify route decisions: dense-300k -> route 3 (Seg8-FL via 17*nrec>=13*n), Code/Docs -> Open (NOT route 3!), periodic -> route 2
    assert_eq!(c49.run_entry(&dense.records, dense.n, dense.p, &mut out_k), 3, "dense-300k must route to Seg8-FL via 17*nrec>=13*n");
    assert_eq!(c49.run_entry(&per2.records, per2.n, per2.p, &mut out_k), 2, "periodic-150k-d2 must hit Cauchy periodic path");
    for b in code_docs {
        let rt = c49.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_ne!(rt, 3, "Code/Docs {} must stay in Open<16>+F-Inline3, not Seg8-FL!", b.name);
        assert_eq!(&out_k, b.expected.as_ref().unwrap());
    }

    // 3c. All 431 wikidata.bz2 blocks byte-exact across all arms
    for (idx, b) in wiki431.iter().enumerate() {
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        kot.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), &mut out_k).unwrap();
        assert_eq!(out_k, out_ref, "wiki431 I3+OT64 mismatch idx={idx}");
        c48.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(out_k, out_ref, "wiki431 C48-Fused mismatch idx={idx}");
        c49.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(out_k, out_ref, "wiki431 C49-Fused-OT64 mismatch idx={idx}");
    }

    // 3d. 305 Malformed / Multi-Cycle Forward-FL Columns + OutTail Chunk(511/512/513) & Epoch(65536) Stress
    assert!(!kot.invert_tiny_fl(&[(u16::from(b'a') << 8) | 3], 5, 0, &mut out_k));
    assert!(!seg8.try_invert(&[(u16::from(b'a') << 8) | 3], 5, 0, &mut out_k));
    assert!(kot.try_run_const::<16>(&[(u16::from(b'a') << 8) | 3], 5, 0, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(0), &mut out_k).is_none());

    let mut mal_cnt = 0usize;
    let mut check_malformed = |l: &[u8], p: u32, tag: &str| {
        let n = l.len();
        let recs = records_of(l);
        kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        let mut pbuf = vec![0xAAu8; n + 64];
        kot.try_run_const::<16>(&recs, n, p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(0), &mut pbuf).unwrap();
        assert_eq!(pbuf, out_ref, "malformed I3+OT64(bi) mismatch {tag}");
        c48.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed C48 mismatch {tag}");
        pbuf.clear(); pbuf.resize(n + 64, 0xAA);
        c49.run_entry(&recs, n, p, &mut pbuf);
        assert_eq!(pbuf, out_ref, "malformed C49-Fused-OT64 mismatch {tag}");
        mal_cnt += 1;
    };

    check_malformed(b"bbaabb", 0, "bbaabb");
    for &per in &[253usize, 254, 255, 256, 257, 258, 512] {
        let n = per * 32;
        let mut l = vec![0u8; n];
        l[0] = 2u8; l[n - 1] = 1u8;
        check_malformed(&l, 0, &format!("sat255-per-{per}"));
    }
    // OutTail chunk (511, 512, 513) & epoch (65535, 65536, 65537) boundary tests
    for &sz in &[511usize, 512, 513, 65_535, 65_536, 65_537] {
        let mut l = Vec::with_capacity(sz);
        for i in 0..sz { l.push(((i * 7 + (i >> 5)) & 3) as u8); }
        check_malformed(&l, (sz / 3) as u32, &format!("boundary-{sz}"));
    }
    let mut lbuf = Vec::with_capacity(65_536);
    for case in 0..292usize {
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

    // 3e. CA-31 Exact r_thin Bound & CA-32 Pre-S1 Markov-MacMahon Lower Bound Audit (ALL 447 real blocks)
    let mut ca31_viol = 0usize;
    let mut ca32_m_viol = 0usize;
    let mut ca32_lin_viol = 0usize;
    let mut checked = 0usize;
    for set in [six, wiki431, non_wiki, code_docs] {
        for b in set {
            let (w1, r_max, _e127, m, ns, r_thin) = exact_stats(b);
            let md_exact = 2 * (i64::from(r_max) - i64::from(ns)) - i64::from(m);
            if md_exact > i64::from(w1) { ca31_viol += 1; }
            let nrec = b.records.len() as i64;
            let ni = b.n as i64;
            // CA-32 Theorem part 1: 16*m >= 17*nrec - n
            if 16 * i64::from(m) < 17 * nrec - ni { ca32_m_viol += 1; }
            // CA-32 Theorem part 2: 4*(3*m + r_thin) >= 17*nrec - n
            if 4 * (3 * i64::from(m) + i64::from(r_thin)) < 17 * nrec - ni { ca32_lin_viol += 1; }
            checked += 1;
        }
    }
    assert_eq!(ca31_viol, 0);
    assert_eq!(ca32_m_viol, 0);
    assert_eq!(ca32_lin_viol, 0);

    // 3f. Persistent Worker Scratch Report on Fresh Kernels (6 Admitted, 233 Full Wiki, 6 JSONL/Log, 4 Code/Docs)
    println!(
        "EXACTNESS PASS: {valid_cnt} small valid + {mal_cnt} malformed (incl. 0x00, SATURATED=255, chunk 511/512/513, epoch 65536, forward-FL) + {ot_poison_tests} OutTail poison/resize/fallback + 4 stress + {} admitted + {} wikidata.bz2 + {} JSONL/Log + {} Code/Docs 100% BYTE-EXACT!",
        six.len(), wiki431.len(), non_wiki.len(), code_docs.len()
    );
    println!("CLASSICAL ATLAS BOUND AUDITS ({checked} real blocks):");
    println!("  CA-31 Exact 2*(r_max - ns) - m <= w1        : {ca31_viol}/{checked} violations (EXACT)");
    println!("  CA-32 Pre-S1 16*m >= 17*nrec - n            : {ca32_m_viol}/{checked} violations (EXACT)");
    println!("  CA-32 Pre-S1 4*(3*m + r_thin) >= 17*nrec - n: {ca32_lin_viol}/{checked} violations (EXACT)");

    println!("PERSISTENT WORKING SET (fresh kernels per corpus):");
    let full_wiki: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();
    let six_r: Vec<&Block> = six.iter().collect();
    let nw_r: Vec<&Block> = non_wiki.iter().collect();
    let cd_r: Vec<&Block> = code_docs.iter().collect();
    for (cname, blks) in [("6 Admitted", &six_r), ("233 FullWiki", &full_wiki), ("6 JSONL/Log", &nw_r), ("4 Code/Docs", &cd_r)] {
        let mut kh_w = cand_r5_head::Kernel::default();
        let mut ki3_w = unified_open::Kernel::<false>::default();
        let mut kot_w = unified_open::Kernel::<true>::default();
        let mut c48_w = C48Fused::default();
        let mut c49_w = C49FusedOt64::default();
        let mut out_w = Vec::with_capacity(900_064);
        let mut tail_hits = 0usize;
        for &b in blks {
            kh_w.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_w);
            ki3_w.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), &mut out_w);
            if let Some(st) = kot_w.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), &mut out_w) {
                if st.used_tail { tail_hits += 1; }
            }
            c48_w.run_entry(&b.records, b.n, b.p, &mut out_w);
            c49_w.run_entry(&b.records, b.n, b.p, &mut out_w);
        }
        let (h_b, _, _) = kh_w.scratch_report();
        let (i3_b, _, _) = ki3_w.scratch_report();
        let (ot_b, _, ot_s) = kot_w.scratch_report();
        let (c48_open_b, _, _) = c48_w.open.scratch_report();
        let c48_extra_b = 4 * c48_w.seg.plain.fl.capacity() + 8 * c48_w.seg.start_bits.capacity();
        let (c49_b, _, c49_s) = c49_w.open.scratch_report();
        let c49_seg_b = 4 * c49_w.seg.plain.fl.capacity() + 8 * c49_w.seg.start_bits.capacity();
        println!(
            "  {:12} | Head={:7.2}K | I3={:7.2}K | I3+OT64={:7.2}K ({:+6.2}K, tail={}/{}) | C48={:7.2}K (+seg={:.1}K) | C49={:7.2}K ({:+6.2}K vs Head, seg={:.1}K) | {}",
            cname,
            h_b as f64 / 1024.0,
            i3_b as f64 / 1024.0,
            ot_b as f64 / 1024.0,
            (ot_b as isize - h_b as isize) as f64 / 1024.0,
            tail_hits,
            blks.len(),
            (c48_open_b + c48_extra_b) as f64 / 1024.0,
            c48_extra_b as f64 / 1024.0,
            (c49_b + c49_seg_b) as f64 / 1024.0,
            ((c49_b + c49_seg_b) as isize - h_b as isize) as f64 / 1024.0,
            c49_seg_b as f64 / 1024.0,
            if cname == "233 FullWiki" { &c49_s } else { &ot_s }
        );
    }

    // 3g. C_recognise telemetry
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
// 4. Headroom Oracle v4 Benchmarks (1T, 2W, 4W) Across All 4 Corpora + 441 Union
// ============================================================================

const NUM_ARMS: usize = 7;
const ARM_NAMES: [&str; NUM_ARMS] = [
    "R5-Head-A1",
    "R5-Head-A2(AA)",
    "R5-Nobi",
    "F-Inline3",
    "I3+OT64",
    "C48-Fused",
    "C49-Fused-OT64",
];

struct ArmSuite {
    kh1: cand_r5_head::Kernel,
    kh2: cand_r5_head::Kernel,
    kn: cand_r5_nobi::Kernel,
    ki3: unified_open::Kernel<false>,
    kot: unified_open::Kernel<true>,
    c48: C48Fused,
    c49: C49FusedOt64,
}

impl Default for ArmSuite {
    fn default() -> Self {
        Self {
            kh1: cand_r5_head::Kernel::default(),
            kh2: cand_r5_head::Kernel::default(),
            kn: cand_r5_nobi::Kernel::default(),
            ki3: unified_open::Kernel::<false>::default(),
            kot: unified_open::Kernel::<true>::default(),
            c48: C48Fused::default(),
            c49: C49FusedOt64::default(),
        }
    }
}

impl ArmSuite {
    #[inline(always)]
    fn run_arm(&mut self, arm: usize, b: &Block, out: &mut Vec<u8>) -> u8 {
        match arm {
            0 => { self.kh1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out); 0 }
            1 => { self.kh2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out); 0 }
            2 => { self.kn.run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, out); 5 }
            3 => { self.ki3.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), out); 0 }
            4 => { self.kot.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, unified_open::BiMode::AlwaysAt(65_536), out); 0 }
            5 => self.c48.run_entry(&b.records, b.n, b.p, out),
            _ => self.c49.run_entry(&b.records, b.n, b.p, out),
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
    println!("\n=== HEADROOM ORACLE v4 (1T INTERLEAVED, {rds} rds/blk): {label} ===");
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
    let mut c49_paths = [0usize; 8];

    for b in blocks {
        let rc = (b.route_class as usize).min(3);
        let mut s: [Vec<f64>; NUM_ARMS] = Default::default();
        let mut p6 = 0u8;
        for r in 0..rds {
            for off in 0..NUM_ARMS {
                let arm = (r + off) % NUM_ARMS;
                let t0 = Instant::now();
                let path = suite.run_arm(arm, b, &mut out);
                s[arm].push(t0.elapsed().as_secs_f64() * 1e6);
                if arm == 6 { p6 = path; }
            }
        }
        c49_paths[p6 as usize] += 1;
        let mut meds = [0.0f64; NUM_ARMS];
        for arm in 0..NUM_ARMS {
            let m = pct(&mut s[arm], 1, 2);
            meds[arm] = m;
            tot[arm] += m;
            by_rc[arm][rc] += m;
            if b.route_class < 2 { full_tot[arm] += m; }
        }
        // Hindsight oracle over genuine arms (excluding A/A duplicate arm 1)
        let best = meds[0].min(meds[2]).min(meds[3]).min(meds[4]).min(meds[5]).min(meds[6]);
        oracle_us += best;
        if b.route_class < 2 { full_oracle_us += best; }
        if show_per_block {
            println!(
                "  {:12} | H1={:6.1} | H2={:6.1} | Nobi={:6.1} | I3={:6.1} ({:.4}x) | OT64={:6.1} ({:.4}x) | C48={:6.1} ({:.4}x) | C49(p{})={:6.1} ({:.4}x) | Orcl={:6.1}",
                b.name, meds[0], meds[1], meds[2], meds[3], meds[3] / meds[0],
                meds[4], meds[4] / meds[0], meds[5], meds[5] / meds[0], p6, meds[6], meds[6] / meds[0], best
            );
        }
    }

    let base = tot[0];
    for arm in 0..NUM_ARMS {
        let lt = if arm >= 3 { if tot[arm] < base { "YES" } else { "NO" } } else { "-" };
        if show_rc {
            println!(
                "  {:16} | norm={:7.2}ms dense={:5.2}ms tiny={:4.2}ms col={:4.2}ms | TOTAL={:7.2}ms ({:.4}x) | < C_Open? {}",
                ARM_NAMES[arm], by_rc[arm][0] / 1e3, by_rc[arm][1] / 1e3, by_rc[arm][2] / 1e3, by_rc[arm][3] / 1e3,
                tot[arm] / 1e3, tot[arm] / base, lt
            );
        } else {
            println!(
                "  {:16} | TOTAL={:8.1} us ({:.4}x) | < C_Open? {}",
                ARM_NAMES[arm], tot[arm], tot[arm] / base, lt
            );
        }
    }
    println!(
        "  {:16} | TOTAL={:8.1} us ({:.4}x) | C49 capture of oracle headroom = {:.1}%",
        "Hindsight-Oracle", oracle_us, oracle_us / base,
        if base > oracle_us { (base - tot[6]) / (base - oracle_us) * 100.0 } else { 0.0 }
    );
    if show_rc {
        let fb = full_tot[0];
        println!(
            "  233-Full-Wiki 1T | Head={:.2}ms | Nobi={:.4}x | I3={:.4}x | OT64={:.4}x | C48={:.4}x | C49={:.4}x | Oracle={:.4}x",
            fb / 1e3, full_tot[2] / fb, full_tot[3] / fb, full_tot[4] / fb, full_tot[5] / fb, full_tot[6] / fb, full_oracle_us / fb
        );
    }
    println!("  C49 routes [L16-bi, tiny, per, seg8, dense-bi, L16-nobi, L24, L32]: {:?}", c49_paths);
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
            let lt = if arm >= 3 { if p50 < b50 { "YES" } else { "NO" } } else { "-" };
            println!(
                "  {}W {:16} | p10={:9.1} us ({:.4}x) | p50={:9.1} us ({:.4}x) | p50 < C_Open? {}",
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

    println!(
        "Headroom Oracle v4 (Cycle 49) loaded: {} admitted, {} wikidata.bz2, {} JSONL/Log, {} Code/Docs blocks",
        six.len(), wiki431.len(), non_wiki.len(), code_docs.len()
    );

    // 1. Exhaustive forward-FL exactness, OutTail poison/resize/fallback, CA-31/CA-32 audits & WS report
    run_exactness_and_audits(&six, &wiki431, &non_wiki, &code_docs);

    // 2. Interleaved 1T Headroom Oracle v4 across all 4 corpora
    let r1_six = bench_1t_interleaved("6 ADMITTED WIKIDATA BLOCKS", &six, 21, false, true);
    let r1_nw = bench_1t_interleaved("6 NON-WIKIDATA JSONL/LOG BLOCKS", &non_wiki, 21, false, true);
    let r1_cd = bench_1t_interleaved("4 NON-WIKIDATA CODE/DOCS BLOCKS", &code_docs, 21, false, true);
    let r1_w431 = bench_1t_interleaved("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 5, true, false);

    // 3. 2W and 4W Concurrent Barrier Benchmarks across all 4 corpora
    let (m2_six, m4_six) = bench_mt_suite("6 ADMITTED WIKIDATA BLOCKS", &six, 19, 13);
    let (m2_nw, m4_nw) = bench_mt_suite("6 NON-WIKIDATA JSONL/LOG BLOCKS", &non_wiki, 19, 13);
    let (m2_cd, m4_cd) = bench_mt_suite("4 NON-WIKIDATA CODE/DOCS BLOCKS", &code_docs, 19, 13);
    let (m2_w431, m4_w431) = bench_mt_suite("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 7, 5);

    // 4. Overall 441-Block Union (431 Wiki + 6 JSONL/Log + 4 Code/Docs) & 243 Full-Block Union
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
        let lt4 = if sum4_441[arm] < sum4_441[0] { "YES" } else { "NO" };
        println!(
            "    {:16} | 1T={:7.2}ms ({:.4}x, <Open? {:3}) | 2W={:7.2}ms ({:.4}x, <Open? {:3}) | 4W={:7.2}ms ({:.4}x, <Open? {:3})",
            ARM_NAMES[arm],
            sum1_441[arm] / 1e3, sum1_441[arm] / sum1_441[0], lt1,
            sum2_441[arm] / 1e3, sum2_441[arm] / sum2_441[0], lt2,
            sum4_441[arm] / 1e3, sum4_441[arm] / sum4_441[0], lt4
        );
    }
    println!(
        "    {:16} | 1T={:7.2}ms ({:.4}x) | 6-Admitted 1T/2W/4W C49-Fused-OT64 = {:.4}x / {:.4}x / {:.4}x",
        "Hindsight-Oracle", orcl1_441 / 1e3, orcl1_441 / sum1_441[0],
        r1_six.tot_us[6] / r1_six.tot_us[0], m2_six[6] / m2_six[0], m4_six[6] / m4_six[0]
    );
    println!("  [243 Full-Size Blocks = 233 Full Wiki + 6 JSONL/Log + 4 Code/Docs, 1T]");
    for arm in 0..NUM_ARMS {
        println!(
            "    {:16} | 1T={:7.2}ms ({:.4}x) | < C_Open? {}",
            ARM_NAMES[arm], sum1_243[arm] / 1e3, sum1_243[arm] / sum1_243[0],
            if sum1_243[arm] < sum1_243[0] { "YES" } else { "NO" }
        );
    }
    println!("    {:16} | 1T={:7.2}ms ({:.4}x)", "Hindsight-Oracle", orcl1_243 / 1e3, orcl1_243 / sum1_243[0]);
}
