#![forbid(unsafe_code)]

//! Cycle 52 P3 — Exact Feature-by-Feature Instruction (`Ir`), Cache (`L2`/`L3`), Working-Set,
//! & Per-Stage Wall Ablation (`C50-DropIn-v1` -> `C51-DropIn-v2` / `C51-Compact3-v1` ->
//! `C52-Lean-Compact`).
//!
//! Strictly modern `2020s` CS systems engineering (`CS` — ZERO classical relabels!).
//!
//! Isolates EVERY individual delta between:
//!   - `C50-DropIn-v1`   (`#729` `instr = 0.888`, `L3 = 14.9`, `W=8 DRAM = 1.37`, `10/10` `llc_miss`)
//!   - `C51-DropIn-v2`   (`#729` `instr = 0.901`, `L3 = 14.8`, `W=8 DRAM = 1.29`, `10/10` `llc_miss`)
//!   - `C51-Compact3-v1` (`#729` `instr = 0.901`, `L3 = 14.6`, `W=1 = 0.909`, `6/6` full `W=10`)
//!
//! And tests safe-Rust instruction-slimming mechanisms in `S2b`/`S3a`/`S4`/`S5`:
//!   - `A00_C50_DropIn_v1`:    Exact Cycle 50 `C50-DropIn-v1` (`instr = 0.888` baseline)
//!   - `A01_PeeledS5`:         `A00` + ONLY peeled `S5` exit
//!   - `A02_TightArena`:       `A00` + ONLY `TightArena` (`arena_min_words` + `4096`/`128` + `16`-ovf)
//!   - `A03_OrphFull_Hoist`:   `A00` + ONLY full `OrphanFree` (hoisted `istk`/`memo_rec`)
//!   - `A04_OrphFull_Local`:   `A00` + ONLY full `OrphanFree` (stack-local `[u32;16]`/`[u32;32]` + upfront `sp`)
//!   - `A05_OrphMemoOnlyFast`: `A00` + ONLY memoized-root-only `truncate(start)` (`TRACK_MEMO=false` + upfront `sp`)
//!   - `A06_TieredTinyFL`:     `A00` + ONLY tiered-stack `TinyFL` (`[u32; 64/256/1536]`)
//!   - `A07_C51_DropIn_v2`:    Exact Cycle 51 `C51-DropIn-v2` (`Tight + OrphFull_Hoist + PeeledS5 + TieredTiny`)
//!   - `A08_CE_Var234`:        `A07` + `CE-Var234` (`2/3/4w` rules, `hdr=4*ns`, upfront `sp`)
//!   - `A09_CE_Var234_ZeroH`:  `A08` + `ZeroH` (`hdr=0` via `out_tail`)
//!   - `A10_Var234_Lean`:      `A09` + branchless `short_payload_h` + unified `code_miss` tail + `OrphMemoOnlyFast`
//!   - `A11_Var234_LeanOrphF`: `A09` + branchless `short_payload_h` + unified `code_miss` tail + full `OrphanFree` (hoisted + upfront `sp`)
//!   - `A12_SlimS4`:           `A11` + safe-Rust `S4` `wire_xor_nomark` slimming (`as_chunks::<2>()` + `pack_hi` + `hi==0` fast path)
//!   - `A13_SlimS5_FastLoop`:  `A11` + safe-Rust `S5` `emit_bidirectional` `while hi-lo >= 32` + `&mut out[..n]` + unified `len_c`
//!   - `A14_SlimS5_LitFast`:   `A13` + `cf < 256` / `cb < 256` 1-byte literal fast path (skips `lens`/`inline`/`u128` shift on ~81.6% of rows)
//!   - `A15_C52_Lean_Compact`: Full fused candidate (`CE-Var234` branchless + `ZeroH` + `TieredTinyFL` + `OrphFull` hoisted/upfront + `SlimS4` + `SlimS5_LitFast`)
//!   - `A16_C52_Lean_NoLitBr`: Full fused candidate WITHOUT `<256` branch (`A12` + `A13`, keeping `S5` branchless)
//!
//! Supports CLI flags:
//!   - `--exact-only`: run full exactness suite (`5,760+` small + `330+` malformed + stress + `447` real) and exit
//!   - `--profile-arm <idx> [--proxy10|--proxy6|--tiny4|--adm6] [--iters <N>]`: run a single arm for `valgrind --tool=callgrind` / `cachegrind`
//!
//! Strictly `#![forbid(unsafe_code)]`, `-D warnings`, forward-`FL` exact on all malformed inputs.

use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::time::Instant;

const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE4: usize = 4;
const MAX_RULE_WORDS: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const INLINE_STACK: usize = 64;
const MEMO_REC_CAP: usize = 32;
const PAIR_WORDS: usize = 512;
const CACHE_WORDS: usize = PAIR_WORDS + 2048;

// F-Inline3 (`CS-06`): 8-bit grade (1..=255), 24-bit inline payload / rule id.
const ID_BITS: u32 = 24;
const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS; // 255

// Const-generic configuration bits
const BIT_VAR234: u32 = 1 << 0;
const BIT_ZEROH: u32 = 1 << 1;
const BIT_TIGHT: u32 = 1 << 2;
const BIT_ORPH_FULL: u32 = 1 << 3;
const BIT_ORPH_MEMO_FAST: u32 = 1 << 4;
const BIT_LOCAL_STK: u32 = 1 << 5;
const BIT_UPFRONT_SP: u32 = 1 << 6;
const BIT_PEELED_S5: u32 = 1 << 7;
const BIT_TIERED_TINY: u32 = 1 << 8;
const BIT_TBL_MASK_CLEAN: u32 = 1 << 9;
const BIT_SLIM_S4: u32 = 1 << 10;
const BIT_SLIM_S5_REM: u32 = 1 << 11;
const BIT_SLIM_S5_U64: u32 = 1 << 12;
const BIT_SLIM_S5_LIT: u32 = 1 << 13;

const CFG_A00: u32 = 0;
const CFG_A01: u32 = BIT_PEELED_S5;
const CFG_A02: u32 = BIT_TIGHT;
const CFG_A03: u32 = BIT_ORPH_FULL | BIT_ORPH_MEMO_FAST;
const CFG_A04: u32 = BIT_ORPH_FULL | BIT_ORPH_MEMO_FAST | BIT_LOCAL_STK | BIT_UPFRONT_SP;
const CFG_A05: u32 = BIT_ORPH_MEMO_FAST | BIT_UPFRONT_SP;
const CFG_A06: u32 = BIT_TIERED_TINY;
const CFG_A07: u32 = BIT_TIGHT | BIT_ORPH_FULL | BIT_ORPH_MEMO_FAST | BIT_PEELED_S5 | BIT_TIERED_TINY;
const CFG_A08: u32 = CFG_A07 | BIT_VAR234 | BIT_UPFRONT_SP;
const CFG_A09: u32 = CFG_A08 | BIT_ZEROH;
const CFG_A10: u32 = BIT_TIGHT
    | BIT_ORPH_FULL
    | BIT_ORPH_MEMO_FAST
    | BIT_TIERED_TINY
    | BIT_VAR234
    | BIT_ZEROH
    | BIT_UPFRONT_SP
    | BIT_TBL_MASK_CLEAN;
const CFG_A11: u32 = CFG_A10 | BIT_SLIM_S4;
const CFG_A12: u32 = CFG_A10 | BIT_SLIM_S5_REM;
const CFG_A13: u32 = CFG_A10 | BIT_SLIM_S5_REM | BIT_SLIM_S5_U64;
const CFG_A14: u32 = CFG_A10 | BIT_SLIM_S5_REM | BIT_SLIM_S5_LIT;
const CFG_A15: u32 = CFG_A10 | BIT_SLIM_S4 | BIT_SLIM_S5_REM;
const CFG_A16: u32 = CFG_A10 | BIT_SLIM_S4 | BIT_SLIM_S5_REM | BIT_SLIM_S5_U64;

const NUM_ARMS: usize = 17;
const ARM_NAMES: [&str; NUM_ARMS] = [
    "A00_C50_DropIn_v1",
    "A01_PeeledS5",
    "A02_TightArena",
    "A03_OrphFull_Hoist",
    "A04_OrphFull_Local",
    "A05_OrphMemoOnly",
    "A06_TieredTinyFL",
    "A07_C51_DropIn_v2",
    "A08_CE_Var234",
    "A09_Var234_ZeroH",
    "A10_CleanS1_TblMsk",
    "A11_SlimS4_Full",
    "A12_SlimS5_Rem16",
    "A13_SlimS5_RemU64",
    "A14_SlimS5_LitKill",
    "A15_C52_Lean_Rem16",
    "A16_C52_Lean_U64",
];

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

#[inline(always)]
const fn has(cfg: u32, bit: u32) -> bool {
    (cfg & bit) != 0
}

#[derive(Clone, Copy, Default)]
struct StageTimes {
    s1: f64,
    s2a: f64,
    s2b: f64,
    s3a: f64,
    s4: f64,
    s5: f64,
    tiny: f64,
    total: f64,
}

impl StageTimes {
    fn add(&mut self, o: &Self) {
        self.s1 += o.s1;
        self.s2a += o.s2a;
        self.s2b += o.s2b;
        self.s3a += o.s3a;
        self.s4 += o.s4;
        self.s5 += o.s5;
        self.tiny += o.tiny;
        self.total += o.total;
    }
}

#[derive(Clone, Copy, Default)]
struct WsSnap {
    edges_cap_b: usize,
    rules_used_b: usize,
    spelling_cap_b: usize,
    spelling_used_b: usize,
    inline_cap_b: usize,
    total_cap_b: usize,
    total_used_b: usize,
    est_gt_m: usize,
    s2b_grows: usize,
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

#[cold]
#[inline(never)]
fn grow_edges_s1_step(a: &mut Vec<u32>, need: usize) {
    let target = need.div_ceil(4096) * 4096;
    if target > a.capacity() {
        a.reserve_exact(target - a.len());
    }
    a.resize(target, 0);
}

// ----------------------------------------------------------------------------
// OutQ16 (compact Vec<u16> next + exact cuts + optional 16-step ovf + SlimS4)
// ----------------------------------------------------------------------------
mod outq {
    const CHUNK: usize = 512;
    pub(super) const BYTES: usize = 2 * CHUNK;
    const NONE32: u32 = u32::MAX;
    const NONE16: u16 = u16::MAX;
    const EPOCH: u32 = 1 << 16;

    #[derive(Default)]
    pub(super) struct Spill {
        pub(super) next: Vec<u16>,
        pub(super) ovf: Vec<[u8; BYTES]>,
        pub(super) cuts: Vec<[u32; 256]>,
    }

    impl Spill {
        pub(super) fn cap_bytes(&self) -> usize {
            self.next.capacity() * 2 + self.ovf.capacity() * BYTES + self.cuts.capacity() * 1024
        }
        pub(super) fn used_bytes(&self) -> usize {
            self.next.len() * 2 + self.ovf.len() * BYTES + self.cuts.len() * 1024
        }
    }

    pub(super) struct Queues<'a, const STEP_OVF: bool> {
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

    impl<'a, const STEP_OVF: bool> Queues<'a, STEP_OVF> {
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
                heads: [NONE32; 256],
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
                    if self.chunks >= u32::from(NONE16) {
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
                if STEP_OVF && self.spill.ovf.len() == self.spill.ovf.capacity() {
                    let target = (self.spill.ovf.len() + 1).div_ceil(16) * 16;
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
            if self.heads[a] == NONE32 {
                self.heads[a] = id;
            } else {
                self.spill.next[((self.pt[a] as u32) >> 10) as usize] = id as u16;
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
        pub(super) fn wire_xor_nomark<const SLIM_S4: bool>(
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
                if SLIM_S4 {
                    // Direct `as_chunks::<2>()` u16 loads (movzwl: 0 shifts, 0 masks!) +
                    // precomputed `pack_hi` + `hi == 0` specialization + 8-way batching!
                    let (pairs, _) = bytes.as_chunks::<2>();
                    let pack_hi = pack | hi;
                    let mut i = 0usize;
                    if hi == 0 {
                        let (chunks8, _) = pairs.as_chunks::<8>();
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
                            eu[0] ^= pack | v0;
                            eu[1] ^= pack | v1;
                            eu[2] ^= pack | v2;
                            eu[3] ^= pack | v3;
                            eu[4] ^= pack | v4;
                            eu[5] ^= pack | v5;
                            eu[6] ^= pack | v6;
                            eu[7] ^= pack | v7;
                            edges[v0 as usize] ^= u0 as u32;
                            edges[v1 as usize] ^= (u0 + 1) as u32;
                            edges[v2 as usize] ^= (u0 + 2) as u32;
                            edges[v3 as usize] ^= (u0 + 3) as u32;
                            edges[v4 as usize] ^= (u0 + 4) as u32;
                            edges[v5 as usize] ^= (u0 + 5) as u32;
                            edges[v6 as usize] ^= (u0 + 6) as u32;
                            edges[v7 as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let v = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack | v;
                            edges[v as usize] ^= u as u32;
                            i += 1;
                        }
                    } else {
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
                            eu[0] ^= pack_hi | l0;
                            eu[1] ^= pack_hi | l1;
                            eu[2] ^= pack_hi | l2;
                            eu[3] ^= pack_hi | l3;
                            eu[4] ^= pack_hi | l4;
                            eu[5] ^= pack_hi | l5;
                            eu[6] ^= pack_hi | l6;
                            eu[7] ^= pack_hi | l7;
                            edges[(hi | l0) as usize] ^= u0 as u32;
                            edges[(hi | l1) as usize] ^= (u0 + 1) as u32;
                            edges[(hi | l2) as usize] ^= (u0 + 2) as u32;
                            edges[(hi | l3) as usize] ^= (u0 + 3) as u32;
                            edges[(hi | l4) as usize] ^= (u0 + 4) as u32;
                            edges[(hi | l5) as usize] ^= (u0 + 5) as u32;
                            edges[(hi | l6) as usize] ^= (u0 + 6) as u32;
                            edges[(hi | l7) as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | lo;
                            edges[(hi | lo) as usize] ^= u as u32;
                            i += 1;
                        }
                    }
                } else {
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
                        edges[v0 as usize] ^= u0 as u32; edges[v1 as usize] ^= (u0 + 1) as u32;
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
                }
                self.at += span;
                self.read += span as u32;
            } else if SLIM_S4 {
                self.wire_xor_slow_nomark(edges, dst_start, span, pack);
            } else {
                let mut dummy = 0u32;
                self.wire_xor_slow(edges, dst_start, span, pack, u32::MAX, &mut dummy);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_slow_nomark(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack: u32,
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
                    self.id = u32::from(self.spill.next[self.id as usize]);
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
                    eu[0] ^= pack_hi | l0;
                    eu[1] ^= pack_hi | l1;
                    eu[2] ^= pack_hi | l2;
                    eu[3] ^= pack_hi | l3;
                    eu[4] ^= pack_hi | l4;
                    eu[5] ^= pack_hi | l5;
                    eu[6] ^= pack_hi | l6;
                    eu[7] ^= pack_hi | l7;
                    edges[(hi | l0) as usize] ^= u0 as u32;
                    edges[(hi | l1) as usize] ^= (u0 + 1) as u32;
                    edges[(hi | l2) as usize] ^= (u0 + 2) as u32;
                    edges[(hi | l3) as usize] ^= (u0 + 3) as u32;
                    edges[(hi | l4) as usize] ^= (u0 + 4) as u32;
                    edges[(hi | l5) as usize] ^= (u0 + 5) as u32;
                    edges[(hi | l6) as usize] ^= (u0 + 6) as u32;
                    edges[(hi | l7) as usize] ^= (u0 + 7) as u32;
                    i += 8;
                }
                for &p in &pairs[i..] {
                    let u = dst_start + i;
                    let lo = u32::from(u16::from_le_bytes(p));
                    edges[u] ^= pack_hi | lo;
                    edges[(hi | lo) as usize] ^= u as u32;
                    i += 1;
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
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
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = u32::from(self.spill.next[self.id as usize]);
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
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = u32::from(self.spill.next[self.id as usize]);
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

// ----------------------------------------------------------------------------
// Links & Parametrized Elim (Fixed-4 vs CE-Var234, Branchless short_payload_h)
// ----------------------------------------------------------------------------
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
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 4..id as usize * 4 + 4];
        (sl[0] >> 16, sl[1], sl[2], sl[3])
    }
}

impl Links for Wide {
    const STRIDE: usize = 6;
    const IS_NARROW: bool = false;
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
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 6..id as usize * 6 + 6];
        (sl[1], sl[2], sl[4], sl[5])
    }
}

struct Elim<'n, 'r, L: Links, const CFG: u32> {
    nodes_buf: &'n mut [u32],
    rules_buf: &'r mut [u32],
    rule_top: usize,
    nodes: u32,
    id_overflow: bool,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links, const CFG: u32> Elim<'_, '_, L, CFG> {
    #[inline(always)]
    fn width(&self, id: u32) -> u32 {
        self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 2]
    }
    #[inline(always)]
    fn set_width(&mut self, id: u32, w: u32) {
        self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 2] = w;
    }
    #[inline(always)]
    fn word(&self, id: u32) -> u32 {
        self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 1]
    }
    #[inline(always)]
    fn set_word(&mut self, id: u32, w: u32) {
        self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 1] = w;
    }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, x: u32) {
        self.nodes_buf[id as usize * L::STRIDE + L::BOTTOM] = x;
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
        let (p, n) = (
            L::prev(self.nodes_buf, id, side),
            L::next(self.nodes_buf, id, side),
        );
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
        if has(CFG, BIT_VAR234) {
            id as usize
        } else {
            id as usize * RULE4
        }
    }

    #[inline(always)]
    fn memo_off(&self, g: u32, r: usize) -> usize {
        if has(CFG, BIT_VAR234) {
            if g < SATURATED {
                r + 2
            } else {
                r + 3
            }
        } else {
            r + 3
        }
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.rules_buf[self.rule_idx(h & ID_MASK) + 2]
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
        if has(CFG, BIT_VAR234) {
            if has(CFG, BIT_TBL_MASK_CLEAN) {
                // Table-masked branchless 4-word read (1 vpand vs 8-instr 128-bit variable shift!)
                let sl = &self.rules_buf[r..r + 4];
                let raw = u128::from(sl[0])
                    | (u128::from(sl[1]) << 32)
                    | (u128::from(sl[2]) << 64)
                    | (u128::from(sl[3]) << 96);
                raw & SHORT_MASK[(g as usize) & 15]
            } else {
                let lo = u128::from(self.rules_buf[r]) | (u128::from(self.rules_buf[r + 1]) << 32);
                if g <= 8 {
                    lo
                } else if g <= 12 {
                    lo | (u128::from(self.rules_buf[r + 2]) << 64)
                } else {
                    lo | (u128::from(self.rules_buf[r + 2]) << 64)
                        | (u128::from(self.rules_buf[r + 3]) << 96)
                }
            }
        } else {
            let sl = &self.rules_buf[r..r + 4];
            u128::from(sl[0])
                | (u128::from(sl[1]) << 32)
                | (u128::from(sl[2]) << 64)
                | (u128::from(sl[3] & 0x00ff_ffff) << 96)
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
        if sum <= 3 {
            return (sum << ID_BITS) | (left & ID_MASK) | ((right & ID_MASK) << (8 * left_len));
        }
        let t = self.rule_top;
        if has(CFG, BIT_VAR234) {
            let id = t as u32;
            if sum < INLINE as u32 {
                let bytes = self.short_payload_h(left)
                    | (self.short_payload_h(right) << (8 * left_len));
                if has(CFG, BIT_TBL_MASK_CLEAN) {
                    let out = &mut self.rules_buf[t..t + 4];
                    out[0] = bytes as u32;
                    out[1] = (bytes >> 32) as u32;
                    if sum <= 8 {
                        self.rule_top = t + 2;
                    } else if sum <= 12 {
                        out[2] = (bytes >> 64) as u32;
                        self.rule_top = t + 3;
                    } else {
                        out[2] = (bytes >> 64) as u32;
                        out[3] = (bytes >> 96) as u32;
                        self.rule_top = t + 4;
                    }
                } else {
                    self.rules_buf[t] = bytes as u32;
                    self.rules_buf[t + 1] = (bytes >> 32) as u32;
                    if sum <= 8 {
                        self.rule_top = t + 2;
                    } else if sum <= 12 {
                        self.rules_buf[t + 2] = (bytes >> 64) as u32;
                        self.rule_top = t + 3;
                    } else {
                        self.rules_buf[t + 2] = (bytes >> 64) as u32;
                        self.rules_buf[t + 3] = (bytes >> 96) as u32;
                        self.rule_top = t + 4;
                    }
                }
                (sum << ID_BITS) | id
            } else if sum < SATURATED {
                let out = &mut self.rules_buf[t..t + 3];
                out[0] = left;
                out[1] = right;
                out[2] = u32::MAX;
                self.rule_top = t + 3;
                (sum << ID_BITS) | id
            } else {
                let len = if sum == SATURATED {
                    sum
                } else {
                    self.len_handle(left) + self.len_handle(right)
                };
                let out = &mut self.rules_buf[t..t + 4];
                out[0] = left;
                out[1] = right;
                out[2] = len;
                out[3] = u32::MAX;
                self.rule_top = t + 4;
                (SATURATED << ID_BITS) | id
            }
        } else {
            let id = (t / RULE4) as u32;
            if sum < INLINE as u32 {
                let bytes = self.short_payload_h(left)
                    | (self.short_payload_h(right) << (8 * left_len))
                    | (u128::from(sum) << 120);
                let out = &mut self.rules_buf[t..t + RULE4];
                out[0] = bytes as u32;
                out[1] = (bytes >> 32) as u32;
                out[2] = (bytes >> 64) as u32;
                out[3] = (bytes >> 96) as u32;
                self.rule_top = t + RULE4;
                (sum << ID_BITS) | id
            } else {
                let len = if sum <= SATURATED {
                    sum
                } else {
                    self.len_handle(left) + self.len_handle(right)
                };
                let out = &mut self.rules_buf[t..t + RULE4];
                out[0] = left;
                out[1] = right;
                out[2] = len;
                out[3] = u32::MAX;
                self.rule_top = t + RULE4;
                (len.min(SATURATED) << ID_BITS) | id
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

    #[inline(never)]
    fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let max_id_top = ID_MASK as usize + 1;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.rule_top + MAX_RULE_WORDS > self.rules_buf.len() {
                break false;
            }
            if has(CFG, BIT_VAR234) && self.rule_top + MAX_RULE_WORDS > max_id_top {
                self.id_overflow = true;
                break false;
            }
            let sent = &self.nodes_buf[..2];
            let x = sent[0] & 0xFFFF;
            let y = sent[1] & 0xFFFF;
            let xi = x as usize * 4;
            let yi = y as usize * 4;
            let nx = &self.nodes_buf[xi..xi + 4];
            let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                self.nodes_buf[px0 as usize * 4] &= 0xFFFF;
                self.nodes_buf[0] = (self.nodes_buf[0] & 0xFFFF_0000) | px0;
                self.nodes_buf[px1 as usize * 4 + 1] &= 0xFFFF;
                self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | px1;
                continue;
            }
            let ny = &self.nodes_buf[yi..yi + 4];
            let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            let word = self.join(wx, wy);
            if aw <= bw {
                let px = x0 & 0xFFFF;
                self.nodes_buf[px as usize * 4] &= 0xFFFF;
                self.nodes_buf[0] = (self.nodes_buf[0] & 0xFFFF_0000) | px;
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let nxm = &mut self.nodes_buf[xi..xi + 4];
                    nxm[0] = y | (y_next << 16);
                    nxm[3] = word;
                    let nym = &mut self.nodes_buf[yi..yi + 4];
                    nym[0] = (y0 & 0xFFFF) | (x << 16);
                    nym[2] = bw - aw;
                    let yni = y_next as usize * 4;
                    self.nodes_buf[yni] = (self.nodes_buf[yni] & 0xFFFF_0000) | x;
                } else {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    let nxm = &mut self.nodes_buf[xi..xi + 4];
                    nxm[0] = y_prev | (y_next << 16);
                    nxm[3] = word;
                    let ypi = y_prev as usize * 4;
                    self.nodes_buf[ypi] = (self.nodes_buf[ypi] & 0xFFFF) | (x << 16);
                    let yni = y_next as usize * 4;
                    self.nodes_buf[yni] = (self.nodes_buf[yni] & 0xFFFF_0000) | x;
                    let py = y1 & 0xFFFF;
                    self.nodes_buf[py as usize * 4 + 1] &= 0xFFFF;
                    self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | py;
                }
            } else {
                let py = y1 & 0xFFFF;
                self.nodes_buf[py as usize * 4 + 1] &= 0xFFFF;
                self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | py;
                let x_next = if x == py { 0 } else { x1 >> 16 };
                let nym = &mut self.nodes_buf[yi..yi + 4];
                nym[1] = x | (x_next << 16);
                nym[3] = word;
                let nxm = &mut self.nodes_buf[xi..xi + 4];
                nxm[1] = (x1 & 0xFFFF) | (y << 16);
                nxm[2] = aw - bw;
                let xni = x_next as usize * 4 + 1;
                self.nodes_buf[xni] = (self.nodes_buf[xni] & 0xFFFF_0000) | y;
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
        let room = if SPIRALS {
            SPIRAL_RULES * MAX_RULE_WORDS
        } else {
            MAX_RULE_WORDS
        };
        let max_id_top = ID_MASK as usize + 1;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.rule_top + room > self.rules_buf.len() {
                break false;
            }
            if has(CFG, BIT_VAR234) && self.rule_top + room > max_id_top {
                self.id_overflow = true;
                break false;
            }
            let (x, y) = (
                L::prev(self.nodes_buf, 0, 0),
                L::prev(self.nodes_buf, 0, 1),
            );
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

    #[allow(clippy::too_many_arguments)]
    fn inline_word(
        &mut self,
        word: u32,
        byte: u8,
        istk: &mut [u32; INLINE_STACK],
        memo_rec: &mut [u32; MEMO_REC_CAP],
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let sp_step = if has(CFG, BIT_TIGHT) { 4096 } else { 8192 };
        let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
            (v.to_le_bytes(), (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            if has(CFG, BIT_UPFRONT_SP) {
                let total = self.len_handle(word) as usize;
                ensure_step_u8(spelling, total + 1, sp_step);
            }
            let mut dummy_n = 0usize;
            let mut dummy_ovf = false;
            self.append_tail_unw::<false>(
                word,
                istk,
                stack,
                spelling,
                memo_rec,
                &mut dummy_n,
                &mut dummy_ovf,
            );
            if !has(CFG, BIT_UPFRONT_SP) {
                ensure_step_u8(spelling, 1, sp_step);
            }
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn append_tail_unw<const TRACK_MEMO: bool>(
        &mut self,
        handle: u32,
        istk_hoist: &mut [u32; INLINE_STACK],
        ovf_stack: &mut Vec<u32>,
        dest: &mut Vec<u8>,
        memo_rec: &mut [u32; MEMO_REC_CAP],
        memo_n: &mut usize,
        memo_ovf: &mut bool,
    ) {
        let sp_step = if has(CFG, BIT_TIGHT) { 4096 } else { 8192 };
        let mut local_arr = [0u32; 16];
        let istk: &mut [u32] = if has(CFG, BIT_LOCAL_STK) {
            &mut local_arr[..]
        } else {
            &mut istk_hoist[..]
        };
        let mut sp = 0usize;
        let mut used_heap = false;
        let mut h = handle;
        loop {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                if !has(CFG, BIT_UPFRONT_SP) {
                    ensure_step_u8(dest, g as usize, sp_step);
                }
                dest.extend_from_slice(&bytes[..g as usize]);
                if used_heap && !ovf_stack.is_empty() {
                    h = ovf_stack.pop().unwrap();
                    continue;
                }
                if sp > 0 {
                    sp -= 1;
                    h = istk[sp];
                    continue;
                }
                return;
            }
            let r = self.rule_idx(h & ID_MASK);
            let m_off = self.memo_off(g, r);
            let tag = self.rules_buf[m_off];
            if tag != u32::MAX {
                let start = (tag & 0x7fff_ffff) as usize;
                let rlen = if has(CFG, BIT_VAR234) {
                    if g < SATURATED {
                        g as usize
                    } else {
                        self.rules_buf[r + 2] as usize
                    }
                } else {
                    self.rules_buf[r + 2] as usize
                };
                if !has(CFG, BIT_UPFRONT_SP) {
                    ensure_step_u8(dest, rlen, sp_step);
                }
                dest.extend_from_within(start..start + rlen);
                if used_heap && !ovf_stack.is_empty() {
                    h = ovf_stack.pop().unwrap();
                    continue;
                }
                if sp > 0 {
                    sp -= 1;
                    h = istk[sp];
                    continue;
                }
                return;
            }
            self.rules_buf[m_off] = 0x8000_0000 | (dest.len() as u32);
            if TRACK_MEMO {
                if *memo_n < MEMO_REC_CAP {
                    memo_rec[*memo_n] = m_off as u32;
                    *memo_n += 1;
                } else {
                    *memo_ovf = true;
                }
            }
            let left = self.rules_buf[r];
            let right = self.rules_buf[r + 1];
            let lg = left >> ID_BITS;
            if lg < INLINE as u32 {
                let lb = self.short_payload_h(left).to_le_bytes();
                if !has(CFG, BIT_UPFRONT_SP) {
                    ensure_step_u8(dest, lg as usize, sp_step);
                }
                dest.extend_from_slice(&lb[..lg as usize]);
                let rg = right >> ID_BITS;
                if rg < INLINE as u32 {
                    let rb = self.short_payload_h(right).to_le_bytes();
                    if !has(CFG, BIT_UPFRONT_SP) {
                        ensure_step_u8(dest, rg as usize, sp_step);
                    }
                    dest.extend_from_slice(&rb[..rg as usize]);
                    if used_heap && !ovf_stack.is_empty() {
                        h = ovf_stack.pop().unwrap();
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = istk[sp];
                        continue;
                    }
                    return;
                }
                h = right;
            } else {
                if sp < istk.len() {
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

struct Dictionary<'a, const CFG: u32> {
    pairs: &'a mut [u32; 512],
    contents: &'a mut [u32; 2048],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    istk: &'a mut [u32; INLINE_STACK],
    memo_rec: &'a mut [u32; MEMO_REC_CAP],
    stack: &'a mut Vec<u32>,
}

impl<const CFG: u32> Dictionary<'_, CFG> {
    #[inline(always)]
    fn code<L: Links>(&mut self, e: &mut Elim<L, CFG>, handle: u32, byte: u8) -> u32 {
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

    fn code_miss<L: Links>(&mut self, e: &mut Elim<L, CFG>, handle: u32, id: u32, byte: u8) -> u32 {
        let sp_step = if has(CFG, BIT_TIGHT) { 4096 } else { 8192 };
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
        let m_off = e.memo_off(g, r);
        let tag_r = e.rules_buf[m_off];
        let memoized = tag_r != u32::MAX;

        if has(CFG, BIT_TBL_MASK_CLEAN) {
            // Unified single hash/lookup/insert tail (no duplicated code_miss body!)
            let mut memo_n = 0usize;
            let mut memo_ovf = false;
            if memoized {
                let pos = (tag_r & 0x7fff_ffff) as usize;
                let rlen = if has(CFG, BIT_VAR234) && g < SATURATED {
                    g as usize
                } else {
                    e.rules_buf[r + 2] as usize
                };
                ensure_step_u8(self.spelling, rlen + 1, sp_step);
                self.spelling.extend_from_within(pos..pos + rlen);
                self.spelling.push(byte);
            } else {
                if has(CFG, BIT_UPFRONT_SP) {
                    let total = if has(CFG, BIT_VAR234) && g < SATURATED {
                        g as usize
                    } else {
                        e.rules_buf[r + 2] as usize
                    };
                    ensure_step_u8(self.spelling, total + 1, sp_step);
                }
                if has(CFG, BIT_ORPH_FULL) {
                    e.append_tail_unw::<true>(
                        handle,
                        self.istk,
                        self.stack,
                        self.spelling,
                        self.memo_rec,
                        &mut memo_n,
                        &mut memo_ovf,
                    );
                } else {
                    e.append_tail_unw::<false>(
                        handle,
                        self.istk,
                        self.stack,
                        self.spelling,
                        self.memo_rec,
                        &mut memo_n,
                        &mut memo_ovf,
                    );
                }
                if !has(CFG, BIT_UPFRONT_SP) {
                    ensure_step_u8(self.spelling, 1, sp_step);
                }
                self.spelling.push(byte);
            }
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
                if memoized {
                    self.spelling.truncate(start);
                } else if has(CFG, BIT_ORPH_FULL) && !memo_ovf {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    let old_start =
                        u32::from_le_bytes(self.inline16[old][..4].try_into().unwrap());
                    let delta = (start as u32) - old_start;
                    for &rm in &self.memo_rec[..memo_n] {
                        e.rules_buf[rm as usize] -= delta;
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

        // Original C50 / C51 code_miss structure for exact A00..A09 reproduction
        if has(CFG, BIT_ORPH_MEMO_FAST) && memoized {
            let pos = (tag_r & 0x7fff_ffff) as usize;
            let rlen = if has(CFG, BIT_VAR234) && g < SATURATED {
                g as usize
            } else {
                e.rules_buf[r + 2] as usize
            };
            ensure_step_u8(self.spelling, rlen + 1, sp_step);
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

        if has(CFG, BIT_UPFRONT_SP) {
            let total = if has(CFG, BIT_VAR234) && g < SATURATED {
                g as usize
            } else {
                e.rules_buf[r + 2] as usize
            };
            ensure_step_u8(self.spelling, total + 1, sp_step);
        }
        let mut local_mrec = [0u32; MEMO_REC_CAP];
        let mrec_ref: &mut [u32; MEMO_REC_CAP] = if has(CFG, BIT_LOCAL_STK) {
            &mut local_mrec
        } else {
            self.memo_rec
        };
        let mut memo_n = 0usize;
        let mut memo_ovf = false;
        if has(CFG, BIT_ORPH_FULL) {
            e.append_tail_unw::<true>(
                handle,
                self.istk,
                self.stack,
                self.spelling,
                mrec_ref,
                &mut memo_n,
                &mut memo_ovf,
            );
        } else {
            e.append_tail_unw::<false>(
                handle,
                self.istk,
                self.stack,
                self.spelling,
                mrec_ref,
                &mut memo_n,
                &mut memo_ovf,
            );
        }
        if !has(CFG, BIT_UPFRONT_SP) {
            ensure_step_u8(self.spelling, 1, sp_step);
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
            if has(CFG, BIT_ORPH_FULL) {
                if !memo_ovf {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    let old_start =
                        u32::from_le_bytes(self.inline16[old][..4].try_into().unwrap());
                    let delta = (start as u32) - old_start;
                    for &rm in &mrec_ref[..memo_n] {
                        e.rules_buf[rm as usize] -= delta;
                    }
                    self.spelling.truncate(start);
                } else {
                    self.spelling.pop();
                }
            } else if memoized {
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
        let inl_step = if has(CFG, BIT_TIGHT) { 128 } else { 256 };
        ensure_step_inline(self.inline16, self.lens, inl_step);
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
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

// ----------------------------------------------------------------------------
// TinyFL (Heap vs Tiered-Stack)
// ----------------------------------------------------------------------------
#[inline(always)]
fn invert_tiny_fl_with_buf(
    records: &[u16],
    n: usize,
    p: u32,
    fl: &mut [u32],
    out: &mut Vec<u8>,
) -> bool {
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

// ----------------------------------------------------------------------------
// Spec4 / CA-37 / CA-05 Selector + S1..S5 Parametrized Kernel
// ----------------------------------------------------------------------------
#[inline]
fn select_limit(records: &[u16], n: usize) -> u32 {
    let nrec = records.len();
    if n < 32_768 || nrec < 256 {
        return 16;
    }
    let step = nrec / 256;
    let mut idx = step / 2;
    let mut s_w1 = 0u32;
    let mut s_r17 = 0u32;
    let step4 = step * 4;
    for _ in 0..64 {
        let w = u32::from(records[idx] & 255);
        s_w1 += u32::from(w == 1);
        s_r17 += u32::from(w > 16 && w <= 32);
        idx += step4;
    }
    let est_w1_64 = (nrec as u64 * u64::from(s_w1)) >> 6;
    if est_w1_64 > 39_500 || s_r17 > 2 {
        return 16;
    }
    idx = step / 2;
    s_w1 = 0;
    s_r17 = 0;
    for _ in 0..256 {
        let w = u32::from(records[idx] & 255);
        s_w1 += u32::from(w == 1);
        s_r17 += u32::from(w > 16 && w <= 32);
        idx += step;
    }
    let est_w1 = (nrec as u64 * u64::from(s_w1)) >> 8;
    if est_w1 <= 33_500 {
        if s_r17 <= 3 {
            32
        } else if s_r17 <= 5 {
            24
        } else {
            16
        }
    } else {
        16
    }
}

#[inline(always)]
fn should_use_bi(m: u32, ns: u32, r_max: u32) -> bool {
    if m < 65_536 {
        return false;
    }
    let md_w1 = 2i64 * (i64::from(r_max) - i64::from(ns)) - i64::from(m);
    !((r_max <= 50_000 && ns <= 2_100 && md_w1 <= 16_000)
        || (md_w1 < -18_000 && ns <= 2_400))
}

struct Pass<'a, const STEP_OVF: bool, const S1_STEP: bool> {
    queues: outq::Queues<'a, STEP_OVF>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    f: First,
}

impl<const STEP_OVF: bool, const S1_STEP: bool> Pass<'_, STEP_OVF, S1_STEP> {
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
            let at = 3 * self.ns;
            if self.edges.len() < at + 3 {
                if S1_STEP {
                    grow_edges_s1_step(self.edges, at + 3);
                } else {
                    let need = (at + 3).max(2 * self.edges.len()).max(1024);
                    self.edges.reserve_exact(need - self.edges.len());
                    self.edges.resize(need, 0);
                }
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

#[derive(Default)]
struct AblationKernel {
    spill: outq::Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    predecessor: Option<u32>,
}

impl AblationKernel {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn ws_snap(&self, edges_used_w: usize, rules_used_w: usize, est_gt_m: usize, s2b_grows: usize) -> WsSnap {
        let edges_cap_b = self.edges.capacity() * 4;
        let edges_used_b = edges_used_w * 4;
        let rules_used_b = rules_used_w * 4;
        let spelling_cap_b = self.spelling.capacity();
        let spelling_used_b = self.spelling.len();
        let inline_cap_b = self.inline16.capacity() * INLINE + self.lens.capacity();
        let inline_used_b = self.inline16.len() * INLINE + self.lens.len();
        let spill_cap_b = self.spill.cap_bytes();
        let other_cap_b = self.bands.capacity() * 8
            + self.extra.capacity() * 4
            + self.stack.capacity() * 4;
        let other_used_b =
            self.bands.len() * 8 + self.extra.len() * 4 + self.stack.len() * 4;
        let total_cap_b =
            edges_cap_b + spelling_cap_b + inline_cap_b + spill_cap_b + other_cap_b;
        let total_used_b =
            edges_used_b + spelling_used_b + inline_used_b + self.spill.used_bytes() + other_used_b;
        WsSnap {
            edges_cap_b,
            rules_used_b,
            spelling_cap_b,
            spelling_used_b,
            inline_cap_b,
            total_cap_b,
            total_used_b,
            est_gt_m,
            s2b_grows,
        }
    }

    #[inline(never)]
    fn invert_tiny_fl<const TIERED: bool>(
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
        if TIERED {
            if n <= 64 {
                return invert_tiny_fl_64(records, n, p, out);
            }
            if n <= 256 {
                return invert_tiny_fl_256(records, n, p, out);
            }
            if n <= 1536 {
                return invert_tiny_fl_1536(records, n, p, out);
            }
        }
        if self.edges.len() < n {
            let add = n - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(n, 0);
        }
        invert_tiny_fl_with_buf(records, n, p, &mut self.edges[..n], out)
    }

    #[inline(never)]
    fn run_cfg<const CFG: u32, const TIMED: bool, const FORCE_OPEN: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> (bool, StageTimes, WsSnap) {
        let mut tm = StageTimes::default();
        let t_all = if TIMED { Some(Instant::now()) } else { None };
        if n == 0 || n > 900_000 || (p as usize) >= n || records.len() > 16_000_000 {
            return (false, tm, WsSnap::default());
        }
        if !FORCE_OPEN && n < 32_768 {
            let t0 = if TIMED { Some(Instant::now()) } else { None };
            let ok = if has(CFG, BIT_TIERED_TINY) {
                self.invert_tiny_fl::<true>(records, n, p, out)
            } else {
                self.invert_tiny_fl::<false>(records, n, p, out)
            };
            if let Some(t) = t0 {
                tm.tiny = t.elapsed().as_secs_f64() * 1e6;
                tm.total = tm.tiny;
            }
            let snap = self.ws_snap(0, 0, 0, 0);
            return (ok, tm, snap);
        }
        let lim = if FORCE_OPEN {
            16
        } else {
            select_limit(records, n)
        };
        let (ok, snap) = match lim {
            16 => self.try_run_lim::<CFG, 16, TIMED>(records, n, p, out, &mut tm),
            24 => self.try_run_lim::<CFG, 24, TIMED>(records, n, p, out, &mut tm),
            _ => self.try_run_lim::<CFG, 32, TIMED>(records, n, p, out, &mut tm),
        };
        if let Some(t) = t_all {
            tm.total = t.elapsed().as_secs_f64() * 1e6;
        }
        (ok, tm, snap)
    }

    #[inline(never)]
    fn try_run_lim<const CFG: u32, const LIM: u32, const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        tm: &mut StageTimes,
    ) -> (bool, WsSnap) {
        let nrec = records.len();
        out.resize(n + INLINE, 0);
        let t1 = if TIMED { Some(Instant::now()) } else { None };
        let (ns, f, heads, over, extra_127, tot_pos) = if has(CFG, BIT_TIGHT) {
            if has(CFG, BIT_TBL_MASK_CLEAN) {
                self.stage1_scan::<true, false, LIM>(records, n, p, out)
            } else {
                self.stage1_scan::<true, true, LIM>(records, n, p, out)
            }
        } else {
            self.stage1_scan::<false, false, LIM>(records, n, p, out)
        };
        if let Some(t) = t1 {
            tm.s1 = t.elapsed().as_secs_f64() * 1e6;
        }
        if over || tot_pos as usize != n || f.m == 0 {
            return (false, WsSnap::default());
        }
        let r_max = (nrec as u32).saturating_sub(extra_127);
        self.finish::<CFG, TIMED>(&f, ns, n, r_max, &heads, out, tm)
    }

    #[inline(never)]
    fn stage1_scan<const STEP_OVF: bool, const S1_STEP: bool, const LIM: u32>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut [u8],
    ) -> (usize, First, outq::Heads, bool, u32, u32) {
        let nrec = records.len();
        let mut pass = Pass::<STEP_OVF, S1_STEP> {
            queues: outq::Queues::new(out, &mut self.spill, n as u32),
            edges: &mut self.edges,
            ns: 0,
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
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn finish<const CFG: u32, const TIMED: bool>(
        &mut self,
        f: &First,
        ns: usize,
        n: usize,
        r_max: u32,
        heads: &outq::Heads,
        out: &mut Vec<u8>,
        tm: &mut StageTimes,
    ) -> (bool, WsSnap) {
        let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        if all.iter().sum::<u32>() != n as u32 {
            return (false, WsSnap::default());
        }
        let first = starts(all);
        let mut write = starts(f.thick);
        let m = f.m as usize;
        let most = Self::nodes_most(ns);
        let use_narrow = most <= NARROW_NODES;
        let stride = if use_narrow {
            Narrow::STRIDE
        } else {
            Wide::STRIDE
        };

        self.bands.clear();
        self.spelling.clear();

        let in_out_chunks = out.len() / outq::BYTES;
        let tail_offset = (heads.chunks as usize).min(in_out_chunks) * outq::BYTES;
        let use_zero_hdr = has(CFG, BIT_ZEROH) && (out.len() - tail_offset >= 16 * ns);

        let est = if use_zero_hdr {
            stride * most
        } else if has(CFG, BIT_TIGHT) {
            4 * ns + stride * most
        } else {
            4 * ns + (stride + RULE4) * most
        };
        let est_gt_m = usize::from(est > m);
        let need = est.max(m);
        if self.edges.len() < need {
            let add = need - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(need, 0);
        }

        let (ok_ind, cursors, used_tail, nbands, s2b_words, rules_w, s2b_grows) = if use_narrow {
            self.induced_narrow::<CFG, TIMED>(
                ns,
                n as u32,
                f.m,
                &f.thin,
                &first,
                &mut write,
                most,
                use_zero_hdr,
                &mut out[tail_offset..],
                tm,
            )
        } else {
            self.induced_wide::<CFG, TIMED>(
                ns,
                n as u32,
                f.m,
                &f.thin,
                &first,
                &mut write,
                most,
                use_zero_hdr,
                &mut out[tail_offset..],
                tm,
            )
        };
        if !ok_ind {
            return (false, WsSnap::default());
        }

        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m {
            self.edges.resize(m, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m, 0);
        }
        self.predecessor = None;
        let use_xor = !wide && should_use_bi(f.m, ns as u32, r_max);
        let (out_q, out_tail) = out.split_at(tail_offset);

        let t4 = if TIMED { Some(Instant::now()) } else { None };
        if use_xor {
            let pred = if used_tail {
                if has(CFG, BIT_SLIM_S4) {
                    self.wire_bi_tail::<true>(
                        m,
                        f.mark,
                        f.mark_byte,
                        bits,
                        &cursors,
                        nbands,
                        out_tail,
                        out_q,
                        heads,
                    )
                } else {
                    self.wire_bi_tail::<false>(
                        m,
                        f.mark,
                        f.mark_byte,
                        bits,
                        &cursors,
                        nbands,
                        out_tail,
                        out_q,
                        heads,
                    )
                }
            } else if has(CFG, BIT_SLIM_S4) {
                self.wire_bi_vec::<true>(
                    m,
                    f.mark,
                    f.mark_byte,
                    bits,
                    &cursors,
                    out_q,
                    heads,
                )
            } else {
                self.wire_bi_vec::<false>(
                    m,
                    f.mark,
                    f.mark_byte,
                    bits,
                    &cursors,
                    out_q,
                    heads,
                )
            };
            self.predecessor = Some(pred);
        } else if used_tail {
            self.wire_uni_tail(m, wide, bits, &cursors, nbands, out_tail, out_q, heads);
        } else {
            self.wire_uni_vec(m, wide, bits, &cursors, out_q, heads);
        }
        if let Some(t) = t4 {
            tm.s4 = t.elapsed().as_secs_f64() * 1e6;
        }

        let edges_used_w = m.max(s2b_words);
        let snap = self.ws_snap(edges_used_w, rules_w, est_gt_m, s2b_grows);

        let t5 = if TIMED { Some(Instant::now()) } else { None };
        if let Some(before) = self.predecessor {
            emit_bidirectional::<CFG>(
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
            emit::<CFG>(
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
        if let Some(t) = t5 {
            tm.s5 = t.elapsed().as_secs_f64() * 1e6;
        }
        (true, snap)
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn induced_narrow<const CFG: u32, const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        use_zero_hdr: bool,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
    ) -> (bool, [usize; 256], bool, usize, usize, usize, usize) {
        self.induced_with::<Narrow, CFG, TIMED>(
            ns,
            n,
            m,
            thin,
            first,
            write,
            most,
            use_zero_hdr,
            out_tail,
            tm,
        )
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn induced_wide<const CFG: u32, const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        use_zero_hdr: bool,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
    ) -> (bool, [usize; 256], bool, usize, usize, usize, usize) {
        self.induced_with::<Wide, CFG, TIMED>(
            ns,
            n,
            m,
            thin,
            first,
            write,
            most,
            use_zero_hdr,
            out_tail,
            tm,
        )
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn stage2a_build<L: Links, const CFG: u32>(
        &mut self,
        ns: usize,
        n: u32,
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        use_zero_hdr: bool,
        out_tail: &mut [u8],
    ) -> (usize, u32, usize, u32) {
        let nodes_words = L::STRIDE * most;
        let mut widest = 0u32;
        if use_zero_hdr {
            let (ot_words, _) = out_tail.as_chunks_mut::<4>();
            let (strips_ot, rest_ot) = ot_words.split_at_mut(3 * ns);
            let image_ot = &mut rest_ot[..ns];
            if has(CFG, BIT_TBL_MASK_CLEAN) {
                for (d, &s) in strips_ot.iter_mut().zip(&self.edges[..3 * ns]) {
                    *d = s.to_ne_bytes();
                }
            } else {
                for i in 0..3 * ns {
                    strips_ot[i] = self.edges[i].to_ne_bytes();
                }
            }
            let (nodes_sl, rules_dummy) = self.edges.split_at_mut(nodes_words);
            let mut e = Elim::<L, CFG> {
                nodes_buf: nodes_sl,
                rules_buf: rules_dummy,
                rule_top: 0,
                nodes: 0,
                id_overflow: false,
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
            (base, e.nodes, open_nodes, widest)
        } else {
            let (hdr, arena) = self.edges.split_at_mut(4 * ns);
            let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
            let (nodes_sl, rules_dummy) = arena.split_at_mut(nodes_words);
            let mut e = Elim::<L, CFG> {
                nodes_buf: nodes_sl,
                rules_buf: rules_dummy,
                rule_top: 0,
                nodes: 0,
                id_overflow: false,
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
            (base, e.nodes, open_nodes, widest)
        }
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn stage3a_bands<L: Links, const CFG: u32>(
        &mut self,
        ns: usize,
        m: u32,
        thin: &[u32; 256],
        hdr_words: usize,
        base: usize,
        rule_top: usize,
        nodes: u32,
        open_nodes: usize,
        use_zero_hdr: bool,
        out_tail: &mut [u8],
    ) -> ([usize; 256], bool, usize) {
        let pad = if has(CFG, BIT_VAR234) && has(CFG, BIT_TBL_MASK_CLEAN) {
            2
        } else {
            0
        };
        let min_live = hdr_words + base + rule_top + pad;
        if self.edges.len() < min_live {
            self.edges.reserve_exact(min_live - self.edges.len());
            self.edges.resize(min_live, 0);
        }
        if use_zero_hdr && ns >= 64 && self.edges.len() < base + rule_top + pad + CACHE_WORDS {
            let need_c = base + rule_top + pad + CACHE_WORDS;
            self.edges.reserve_exact(need_c - self.edges.len());
            self.edges.resize(need_c, 0);
        }

        let (header, arena) = self.edges.split_at_mut(hdr_words);
        let (live, tail_arena) = arena.split_at_mut(base + rule_top + pad);
        let (nodes_sl, rules_sl) = live.split_at_mut(base);
        let mut e = Elim::<L, CFG> {
            nodes_buf: nodes_sl,
            rules_buf: rules_sl,
            rule_top,
            nodes,
            id_overflow: false,
            _links: std::marker::PhantomData,
        };
        e.assign_bottom(m);

        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail_arena
        };
        begin_inline(&mut self.inline16, &mut self.lens);

        let max_bands = open_nodes + 256;
        let use_tail = out_tail.len() >= max_bands * 8;
        let mut cursors = [0usize; 256];
        let mut nbands = 0usize;
        let mut istk = [0u32; INLINE_STACK];
        let mut memo_rec = [0u32; MEMO_REC_CAP];
        let inl_step = if has(CFG, BIT_TIGHT) { 128 } else { 256 };

        if cache.len() < CACHE_WORDS {
            let (band_slots, _) = out_tail.as_chunks_mut::<8>();
            let mut id = L::next(e.nodes_buf, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = if use_tail { nbands } else { self.bands.len() };
                let end = k + thin[a];
                while k < end {
                    let (next_id, node_bottom, node_width, node_word) =
                        L::band_node(e.nodes_buf, id);
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        let (entry, len) = e.inline_word(
                            node_word,
                            a as u8,
                            &mut istk,
                            &mut memo_rec,
                            &mut self.stack,
                            &mut self.spelling,
                        );
                        ensure_step_inline(&mut self.inline16, &mut self.lens, inl_step);
                        let c = self.lens.len() as u32;
                        self.inline16.push(entry);
                        self.lens.push(len);
                        c
                    };
                    if use_tail {
                        band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                        nbands += 1;
                    } else {
                        self.bands.push(Band::new(width, dst, code));
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
            let mut dictionary = Dictionary::<CFG> {
                pairs,
                contents,
                inline16: &mut self.inline16,
                lens: &mut self.lens,
                spelling: &mut self.spelling,
                istk: &mut istk,
                memo_rec: &mut memo_rec,
                stack: &mut self.stack,
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
                            let c = dictionary.code(&mut e, node_word, a as u8);
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
                    cursors[a] = self.bands.len();
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
                            let c = dictionary.code(&mut e, node_word, a as u8);
                            last_w = node_word;
                            last_c = c;
                            c
                        };
                        self.bands.push(Band::new(width, dst, code));
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
        }
        (cursors, use_tail, nbands)
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn induced_with<L: Links, const CFG: u32, const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        use_zero_hdr: bool,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
    ) -> (bool, [usize; 256], bool, usize, usize, usize, usize) {
        let hdr_words = if use_zero_hdr { 0 } else { 4 * ns };
        assert!(self.edges.len() >= hdr_words + L::STRIDE * most);

        let t2a = if TIMED { Some(Instant::now()) } else { None };
        let (base, nodes, open_nodes, widest) =
            self.stage2a_build::<L, CFG>(ns, n, first, write, most, use_zero_hdr, out_tail);
        if let Some(t) = t2a {
            tm.s2a = t.elapsed().as_secs_f64() * 1e6;
        }

        let t2b = if TIMED { Some(Instant::now()) } else { None };
        let spirals = widest > CERTIFIED_WIDTH;
        let mut remaining = n;
        let mut rule_top = 0usize;
        let mut s2b_grows = 0usize;
        loop {
            let (nodes_sl, rules_sl) = self.edges[hdr_words..].split_at_mut(base);
            let mut e = Elim::<L, CFG> {
                nodes_buf: nodes_sl,
                rules_buf: rules_sl,
                rule_top,
                nodes,
                id_overflow: false,
                _links: std::marker::PhantomData,
            };
            let done = if spirals {
                e.eliminate::<true>(&mut remaining)
            } else {
                e.eliminate::<false>(&mut remaining)
            };
            rule_top = e.rule_top;
            if e.id_overflow {
                return (false, [0; 256], false, 0, 0, 0, s2b_grows);
            }
            if done {
                break;
            }
            s2b_grows += 1;
            grow_edges_step(
                &mut self.edges,
                hdr_words + base + rule_top + SPIRAL_RULES * MAX_RULE_WORDS,
            );
        }
        if let Some(t) = t2b {
            tm.s2b = t.elapsed().as_secs_f64() * 1e6;
        }

        let t3a = if TIMED { Some(Instant::now()) } else { None };
        let (cursors, use_tail, nbands) = self.stage3a_bands::<L, CFG>(
            ns,
            m,
            thin,
            hdr_words,
            base,
            rule_top,
            nodes,
            open_nodes,
            use_zero_hdr,
            out_tail,
        );
        if let Some(t) = t3a {
            tm.s3a = t.elapsed().as_secs_f64() * 1e6;
        }
        let s2b_words = hdr_words + base + rule_top;
        (true, cursors, use_tail, nbands, s2b_words, rule_top, s2b_grows)
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_bi_tail<const SLIM_S4: bool>(
        &mut self,
        m: usize,
        mark: u32,
        mark_byte: u8,
        bits: u32,
        cursors: &[usize; 256],
        nbands: usize,
        out_tail: &[u8],
        out_q: &[u8],
        heads: &outq::Heads,
    ) -> u32 {
        let edges = &mut self.edges[..m];
        edges.fill(0);
        let (band_slots, _) = out_tail.as_chunks::<8>();
        let mut predecessor = mark;
        let mb = usize::from(mark_byte);
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(nbands);
            let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
            if a != mb {
                for &raw in &band_slots[cursors[a]..end] {
                    let band = Band(u64::from_le_bytes(raw));
                    queue.wire_xor_nomark::<SLIM_S4>(
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
                        &mut predecessor,
                    );
                }
            }
        }
        predecessor
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_bi_vec<const SLIM_S4: bool>(
        &mut self,
        m: usize,
        mark: u32,
        mark_byte: u8,
        bits: u32,
        cursors: &[usize; 256],
        out_q: &[u8],
        heads: &outq::Heads,
    ) -> u32 {
        let edges = &mut self.edges[..m];
        edges.fill(0);
        let mut predecessor = mark;
        let mb = usize::from(mark_byte);
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
            if a != mb {
                for &band in &self.bands[cursors[a]..end] {
                    queue.wire_xor_nomark::<SLIM_S4>(
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
                        &mut predecessor,
                    );
                }
            }
        }
        predecessor
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_uni_tail(
        &mut self,
        m: usize,
        wide: bool,
        bits: u32,
        cursors: &[usize; 256],
        nbands: usize,
        out_tail: &[u8],
        out_q: &[u8],
        heads: &outq::Heads,
    ) {
        let edges = &mut self.edges[..m];
        let (band_slots, _) = out_tail.as_chunks::<8>();
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(nbands);
            let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
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

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_uni_vec(
        &mut self,
        m: usize,
        wide: bool,
        bits: u32,
        cursors: &[usize; 256],
        out_q: &[u8],
        heads: &outq::Heads,
    ) {
        let edges = &mut self.edges[..m];
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
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

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit<const CFG: u32>(
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
    let mask = (1 << bits) - 1;
    let mut j = mark;
    if has(CFG, BIT_SLIM_S5_REM) && extra.is_empty() {
        let len_c = lens.len().min(inline16.len());
        let lens = &lens[..len_c];
        let inline16 = &inline16[..len_c];
        let mut rem: &mut [u8] = &mut out[..n];
        let mut done = false;
        while rem.len() >= INLINE {
            let edge = edges[j as usize];
            let code = (edge >> bits) as usize;
            j = edge & mask;
            let len = lens[code];
            if len != LONG {
                rem[..INLINE].copy_from_slice(&inline16[code]);
                rem = &mut rem[usize::from(len)..];
            } else {
                let take = emit_long_front(&inline16[code], spelling, rem);
                rem = &mut rem[take..];
            }
            if j == mark {
                done = true;
                break;
            }
        }
        if !done {
            while !rem.is_empty() {
                let edge = edges[j as usize];
                let code = (edge >> bits) as usize;
                j = edge & mask;
                let len = lens[code];
                let word = if len == LONG {
                    long_word(&inline16[code], spelling)
                } else {
                    &inline16[code][..usize::from(len)]
                };
                let take = word.len().min(rem.len());
                rem[..take].copy_from_slice(&word[..take]);
                rem = &mut rem[take..];
                if j == mark {
                    break;
                }
            }
        }
        let written = n - rem.len();
        out.truncate(written);
        repeat(n, out);
        return;
    }
    let mut at = 0;
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
fn emit_tail_back(
    slot: &[u8; INLINE],
    lb: u8,
    spelling: &[u8],
    rem: &mut [u8],
) -> usize {
    let word = if lb == LONG {
        long_word(slot, spelling)
    } else {
        &slot[..usize::from(lb)]
    };
    let rlen = rem.len();
    let take = word.len().min(rlen);
    rem[rlen - take..].copy_from_slice(&word[word.len() - take..]);
    take
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_bidirectional<const CFG: u32>(
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

    if has(CFG, BIT_SLIM_S5_REM) {
        let out_base = out.as_ptr() as usize;
        let mut rem: &mut [u8] = &mut out[..n];
        let len_c = lens.len().min(inline.len());
        let lens = &lens[..len_c];
        let inline = &inline[..len_c];
        let mut done = false;
        while rem.len() >= 2 * INLINE {
            let ef = edges[front as usize];
            let cf = (ef >> bits) as usize;
            let single = front == back;
            if has(CFG, BIT_SLIM_S5_LIT) && cf < 256 {
                rem[0] = cf as u8;
                rem = &mut rem[1..];
            } else {
                let lf = lens[cf];
                if lf != LONG {
                    rem[..INLINE].copy_from_slice(&inline[cf]);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline[cf], spelling, rem);
                    rem = &mut rem[take..];
                    if single || rem.is_empty() {
                        done = true;
                        break;
                    }
                    if rem.len() < INLINE {
                        let eb = edges[back as usize];
                        let cb = (eb >> bits) as usize;
                        let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                        let rlen = rem.len();
                        rem = &mut rem[..rlen - take_b];
                        (prev, front) = (front, (ef & mask) ^ prev);
                        (next, back) = (back, (eb & mask) ^ next);
                        if front == next {
                            done = true;
                        }
                        break;
                    }
                }
            }
            if single {
                done = true;
                break;
            }
            // Statically guaranteed: rem.len() >= INLINE (16) here!
            let eb = edges[back as usize];
            let cb = (eb >> bits) as usize;
            let rlen = rem.len();
            if has(CFG, BIT_SLIM_S5_LIT) && cb < 256 {
                rem[rlen - 1] = cb as u8;
                rem = &mut rem[..rlen - 1];
            } else {
                let lb = lens[cb];
                if has(CFG, BIT_SLIM_S5_U64) {
                    if lb <= 8 {
                        let w = u64::from_le_bytes(inline[cb][..8].try_into().unwrap())
                            << (8 * (8 - usize::from(lb)));
                        rem[rlen - 8..].copy_from_slice(&w.to_le_bytes());
                        rem = &mut rem[..rlen - usize::from(lb)];
                    } else if lb != LONG {
                        let shifted =
                            u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                        rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                        rem = &mut rem[..rlen - usize::from(lb)];
                    } else {
                        let take = emit_long_back(&inline[cb], spelling, rem);
                        rem = &mut rem[..rlen - take];
                    }
                } else if lb != LONG {
                    let shifted =
                        u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                    rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                    rem = &mut rem[..rlen - usize::from(lb)];
                } else {
                    let take = emit_long_back(&inline[cb], spelling, rem);
                    rem = &mut rem[..rlen - take];
                }
            }
            (prev, front) = (front, (ef & mask) ^ prev);
            (next, back) = (back, (eb & mask) ^ next);
            if front == next {
                done = true;
                break;
            }
        }
        if !done {
            while !rem.is_empty() {
                let ef = edges[front as usize];
                let cf = (ef >> bits) as usize;
                let single = front == back;
                let lf = lens[cf];
                let word = if lf == LONG {
                    long_word(&inline[cf], spelling)
                } else {
                    &inline[cf][..usize::from(lf)]
                };
                let take = word.len().min(rem.len());
                rem[..take].copy_from_slice(&word[..take]);
                rem = &mut rem[take..];
                if single || rem.is_empty() {
                    break;
                }
                let eb = edges[back as usize];
                let cb = (eb >> bits) as usize;
                let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                let rlen = rem.len();
                rem = &mut rem[..rlen - take_b];
                (prev, front) = (front, (ef & mask) ^ prev);
                (next, back) = (back, (eb & mask) ^ next);
                if front == next {
                    break;
                }
            }
        }
        let rem_len = rem.len();
        let lo_rem = rem.as_ptr() as usize - out_base;
        let hi_rem = lo_rem + rem_len;
        if rem_len > 0 {
            out.copy_within(hi_rem..n, lo_rem);
            out.truncate(lo_rem + (n - hi_rem));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
        return;
    }

    let (mut lo, mut hi) = (0usize, n);
    if has(CFG, BIT_PEELED_S5) {
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
    } else {
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

fn run_arm<const TIMED: bool, const FORCE_OPEN: bool>(
    arm: usize,
    k: &mut AblationKernel,
    records: &[u16],
    n: usize,
    p: u32,
    out: &mut Vec<u8>,
) -> (bool, StageTimes, WsSnap) {
    match arm {
        0 => k.run_cfg::<CFG_A00, TIMED, FORCE_OPEN>(records, n, p, out),
        1 => k.run_cfg::<CFG_A01, TIMED, FORCE_OPEN>(records, n, p, out),
        2 => k.run_cfg::<CFG_A02, TIMED, FORCE_OPEN>(records, n, p, out),
        3 => k.run_cfg::<CFG_A03, TIMED, FORCE_OPEN>(records, n, p, out),
        4 => k.run_cfg::<CFG_A04, TIMED, FORCE_OPEN>(records, n, p, out),
        5 => k.run_cfg::<CFG_A05, TIMED, FORCE_OPEN>(records, n, p, out),
        6 => k.run_cfg::<CFG_A06, TIMED, FORCE_OPEN>(records, n, p, out),
        7 => k.run_cfg::<CFG_A07, TIMED, FORCE_OPEN>(records, n, p, out),
        8 => k.run_cfg::<CFG_A08, TIMED, FORCE_OPEN>(records, n, p, out),
        9 => k.run_cfg::<CFG_A09, TIMED, FORCE_OPEN>(records, n, p, out),
        10 => k.run_cfg::<CFG_A10, TIMED, FORCE_OPEN>(records, n, p, out),
        11 => k.run_cfg::<CFG_A11, TIMED, FORCE_OPEN>(records, n, p, out),
        12 => k.run_cfg::<CFG_A12, TIMED, FORCE_OPEN>(records, n, p, out),
        13 => k.run_cfg::<CFG_A13, TIMED, FORCE_OPEN>(records, n, p, out),
        14 => k.run_cfg::<CFG_A14, TIMED, FORCE_OPEN>(records, n, p, out),
        15 => k.run_cfg::<CFG_A15, TIMED, FORCE_OPEN>(records, n, p, out),
        16 => k.run_cfg::<CFG_A16, TIMED, FORCE_OPEN>(records, n, p, out),
        _ => unreachable!(),
    }
}

// ----------------------------------------------------------------------------
// Corpus loaders, exactness suite, WS audit, stage ablation, & multi-worker bench
// ----------------------------------------------------------------------------
struct Block {
    name: String,
    idx: u32,
    n: usize,
    p: u32,
    records: Vec<u16>,
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

fn ref_fl_from_records(records: &[u16], n: usize, p: u32) -> Vec<u8> {
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
    assert_eq!(tot as usize, n);
    let mut fl = vec![0u32; n];
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
    let mut out = vec![0u8; n];
    let mut x = p as usize;
    for slot in &mut out {
        let v = fl[x];
        *slot = v as u8;
        x = (v >> 8) as usize;
    }
    out
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
        let end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        let p: u32 = rest[..end].parse().expect("parse primary_index");
        let l = fs::read(format!("{dir}/block-{id}.L")).expect("read .L");
        blocks.push(Block {
            name: format!("block-{id}.L"),
            idx: id as u32,
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
            idx,
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

fn make_stress_blocks() -> Vec<Block> {
    let mut out = Vec::new();
    // 1. Fibonacci F_25 (n=75025, Spirals + >=255 saturated rules)
    {
        let (q, p_cnt) = (28_657usize, 46_368usize);
        let n = q + p_cnt;
        let mut l = Vec::with_capacity(n);
        l.resize(q, b'b');
        l.resize(n, b'a');
        out.push(Block {
            name: "fib-F25".into(),
            idx: 0,
            n,
            p: 0,
            records: records_of(&l),
        });
    }
    // 2. Dense-thick low-M (forces S2b grow_edges_step + all Var234 rule tiers)
    {
        let mut rng = SplitMix64(0xC52A_B1A7_1000_0001);
        let n = 120_000usize;
        let mut l = Vec::with_capacity(n);
        let mut prev = 255u8;
        while l.len() < n {
            let mut b = (rng.next() as usize % 24) as u8;
            if b == prev {
                b = (b + 1) % 24;
            }
            prev = b;
            let rl = 30 + (rng.next() as usize % 110);
            for _ in 0..rl.min(n - l.len()) {
                l.push(b);
            }
            if l.len() < n && rng.next() % 5 == 0 {
                let gb = (b + 13) % 48;
                let gl = 1 + (rng.next() as usize % 3);
                for _ in 0..gl.min(n - l.len()) {
                    l.push(gb);
                }
                prev = gb;
            }
        }
        out.push(Block {
            name: "low-m-120k".into(),
            idx: 1,
            n,
            p: 17,
            records: records_of(&l),
        });
    }
    // 3. High-m tiny out_tail (forces ZeroH=false fallback + bi-XOR)
    {
        let n = 80_000usize;
        let mut l = Vec::with_capacity(n);
        for i in 0..n {
            l.push(((i * 31 + (i >> 3) * 17) & 63) as u8);
        }
        for pos in (0..n - 100).step_by(200) {
            let b = ((pos / 200) % 16) as u8;
            for j in 0..24 {
                l[pos + j] = b;
            }
        }
        out.push(Block {
            name: "hi-m-80k-nozh".into(),
            idx: 2,
            n,
            p: 12_345,
            records: records_of(&l),
        });
    }
    // 4. Malformed detached cycle d=1, d=2, d=17 with m >= 65536 (bi-XOR S5 edge cases)
    for &d in &[1usize, 2, 17] {
        let sz = 75_000usize;
        let mut l = vec![0u8; sz];
        for i in 0..d {
            l[i] = ((i + 1) % d) as u8;
        }
        for (i, slot) in l.iter_mut().enumerate().skip(d) {
            *slot = (d + (i % 120)) as u8;
        }
        for slot in &mut l[sz - 8_000..sz] {
            *slot = 250;
        }
        out.push(Block {
            name: format!("malformed-bi-d{d}"),
            idx: 10 + d as u32,
            n: sz,
            p: 0,
            records: records_of(&l),
        });
    }
    out
}

fn run_exactness_suite(
    wiki6: &[Block],
    nonwiki10: &[Block],
    wiki431: &[Block],
    stress: &[Block],
) {
    let t0 = Instant::now();
    let mut kernels: Vec<AblationKernel> = (0..NUM_ARMS).map(|_| AblationKernel::default()).collect();
    let mut out = Vec::new();
    let mut small_valid = 0usize;

    // 510 exhaustive binary words n=1..=8
    for n in 1..=8usize {
        for mask in 0..(1usize << n) {
            let text: Vec<u8> = (0..n)
                .map(|i| if (mask >> i) & 1 == 0 { b'a' } else { b'b' })
                .collect();
            let (l, p) = bwt_of(&text);
            let recs = records_of(&l);
            for (arm, k) in kernels.iter_mut().enumerate() {
                out.clear();
                out.resize(n + 23, 0xAA);
                let (ok, _, _) = run_arm::<false, true>(arm, k, &recs, n, p, &mut out);
                assert!(ok && out == text, "arm {} failed binary n={n} mask={mask}", ARM_NAMES[arm]);
            }
            small_valid += 1;
        }
    }

    // 5,250 random valid BWTs (total 5,760 small valid!)
    let mut rng = SplitMix64(0xC52E_8AC7_5760_0001);
    for case in 0..5250usize {
        let n = match case % 4 {
            0 => 1 + (rng.next() as usize % 32),
            1 => 33 + (rng.next() as usize % 64),
            2 => 97 + (rng.next() as usize % 128),
            _ => 225 + (rng.next() as usize % 128),
        };
        let alpha = match case % 4 {
            0 => 2,
            1 => 4,
            2 => 16,
            _ => 64,
        };
        let mut text = Vec::with_capacity(n);
        if case % 5 == 0 {
            while text.len() < n {
                let b = (rng.next() as usize % alpha) as u8;
                let rl = 1 + (rng.next() as usize % 36);
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
        let recs = records_of(&l);
        // Check representative arms on all 5,250 + all 17 arms on first 600
        let arms_to_test: &[usize] = if case < 600 {
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        } else {
            &[0, 7, 9, 10, 14, 15, 16]
        };
        for &arm in arms_to_test {
            out.clear();
            out.resize(n + 23, if case % 2 == 0 { 0xAA } else { 0x55 });
            let (ok, _, _) = run_arm::<false, true>(arm, &mut kernels[arm], &recs, n, p, &mut out);
            assert!(ok && out == text, "arm {} failed valid case {case}", ARM_NAMES[arm]);
        }
        small_valid += 1;
    }

    // 330 malformed multi-cycle columns (incl. bi-XOR & TinyFL tiers)
    let mut malformed_n = 0usize;
    for case in 0..330usize {
        let n = if case < 310 {
            2 + (rng.next() as usize % 512)
        } else if case < 322 {
            1_535 + (rng.next() as usize % 2_500)
        } else {
            66_000 + (rng.next() as usize % 8_000)
        };
        let mut l = Vec::with_capacity(n);
        if case % 3 == 0 {
            let alpha = 2 + (rng.next() as usize % 8);
            while l.len() < n {
                let b = (rng.next() as usize % alpha) as u8;
                let rl = 1 + (rng.next() as usize % 40);
                for _ in 0..rl.min(n - l.len()) {
                    l.push(b);
                }
            }
        } else {
            for _ in 0..n {
                l.push((rng.next() as usize % 32) as u8);
            }
        }
        let p = (rng.next() as usize % n) as u32;
        let recs = records_of(&l);
        let want = ref_fl_from_records(&recs, n, p);
        for (arm, k) in kernels.iter_mut().enumerate() {
            out.clear();
            out.resize(n + 31, 0x55);
            let (ok, _, _) = run_arm::<false, false>(arm, k, &recs, n, p, &mut out);
            assert!(ok && out == want, "arm {} failed malformed case {case} (n={n})", ARM_NAMES[arm]);
            if n < 32_768 && case < 80 {
                out.clear();
                out.resize(n + 31, 0xAA);
                let (ok2, _, _) = run_arm::<false, true>(arm, k, &recs, n, p, &mut out);
                assert!(ok2 && out == want, "arm {} failed force-open malformed {case}", ARM_NAMES[arm]);
            }
        }
        malformed_n += 1;
    }

    // Stress blocks (all 17 arms)
    for blk in stress {
        let want = ref_fl_from_records(&blk.records, blk.n, blk.p);
        for (arm, k) in kernels.iter_mut().enumerate() {
            out.clear();
            out.resize(blk.n + 37, 0xAA);
            let (ok, _, _) = run_arm::<false, true>(arm, k, &blk.records, blk.n, blk.p, &mut out);
            assert!(ok && out == want, "arm {} failed stress {}", ARM_NAMES[arm], blk.name);
        }
    }

    // All 447 real blocks (6 Adm + 10 Non-Wiki + 431 ALL Wiki) across ALL 17 arms!
    let mut real_ok = 0usize;
    for blk in wiki6.iter().chain(nonwiki10.iter()).chain(wiki431.iter()) {
        let want = ref_fl_from_records(&blk.records, blk.n, blk.p);
        for (arm, k) in kernels.iter_mut().enumerate() {
            out.clear();
            let (ok, _, _) = run_arm::<false, false>(arm, k, &blk.records, blk.n, blk.p, &mut out);
            assert!(ok && out == want, "arm {} failed real block {}", ARM_NAMES[arm], blk.name);
        }
        real_ok += 1;
    }

    println!(
        "[EXACTNESS PASS] {} small valid + {} malformed + {} stress + {}/447 real blocks 100% byte-exact across ALL {} arms ({:.2}s)",
        small_valid,
        malformed_n,
        stress.len(),
        real_ok,
        NUM_ARMS,
        t0.elapsed().as_secs_f64()
    );
}

fn audit_ws(blocks: &[Block], label: &str) {
    println!("\n=== Working Set Audit: {} ({} blocks) ===", label, blocks.len());
    println!(
        "{:<20} {:>9} {:>9} {:>9} {:>8} {:>8} {:>8} {:>8} {:>6} {:>6}",
        "Arm", "PeakCapK", "MaxUsedK", "EdgesCapK", "RulesAvg", "SpAvgK", "SpCapK", "InlCapK", "est>m", "s2bGrw"
    );
    for arm in 0..NUM_ARMS {
        let mut k = AblationKernel::default();
        let mut out = Vec::new();
        let mut peak_cap = 0usize;
        let mut max_used = 0usize;
        let mut edges_cap = 0usize;
        let mut sum_rules = 0usize;
        let mut sum_sp = 0usize;
        let mut sp_cap = 0usize;
        let mut inl_cap = 0usize;
        let mut est_gt_m = 0usize;
        let mut s2b_grows = 0usize;
        let mut n_full = 0usize;
        for blk in blocks {
            let (ok, _, snap) = run_arm::<false, false>(arm, &mut k, &blk.records, blk.n, blk.p, &mut out);
            assert!(ok);
            peak_cap = peak_cap.max(snap.total_cap_b);
            max_used = max_used.max(snap.total_used_b);
            edges_cap = edges_cap.max(snap.edges_cap_b);
            sp_cap = sp_cap.max(snap.spelling_cap_b);
            inl_cap = inl_cap.max(snap.inline_cap_b);
            est_gt_m += snap.est_gt_m;
            s2b_grows += snap.s2b_grows;
            if blk.n >= 32_768 {
                sum_rules += snap.rules_used_b;
                sum_sp += snap.spelling_used_b;
                n_full += 1;
            }
        }
        let d = n_full.max(1) as f64;
        println!(
            "{:<20} {:>9.2} {:>9.2} {:>9.2} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>6} {:>6}",
            ARM_NAMES[arm],
            peak_cap as f64 / 1024.0,
            max_used as f64 / 1024.0,
            edges_cap as f64 / 1024.0,
            sum_rules as f64 / d / 1024.0,
            sum_sp as f64 / d / 1024.0,
            sp_cap as f64 / 1024.0,
            inl_cap as f64 / 1024.0,
            est_gt_m,
            s2b_grows
        );
    }
}

fn bench_stage_1t(blocks: &[Block], label: &str, rounds: usize) {
    println!(
        "\n=== Per-Stage 1T Wall Ablation (p50 us/blk, {} rounds): {} ({} blocks) ===",
        rounds,
        label,
        blocks.len()
    );
    println!(
        "{:<20} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>8} {:>7}",
        "Arm", "S1", "S2a", "S2b", "S3a", "S4", "S5", "Total", "vsA00"
    );
    let mut per_arm: Vec<Vec<StageTimes>> = (0..NUM_ARMS).map(|_| Vec::with_capacity(rounds)).collect();
    let mut kernels: Vec<AblationKernel> = (0..NUM_ARMS).map(|_| AblationKernel::default()).collect();
    let mut out = Vec::new();
    // Warmup
    for arm in 0..NUM_ARMS {
        for blk in blocks {
            run_arm::<false, false>(arm, &mut kernels[arm], &blk.records, blk.n, blk.p, &mut out);
        }
    }
    for r in 0..rounds {
        for off in 0..NUM_ARMS {
            let arm = (off + r) % NUM_ARMS;
            let mut acc = StageTimes::default();
            for blk in blocks {
                let (_, tm, _) =
                    run_arm::<true, false>(arm, &mut kernels[arm], &blk.records, blk.n, blk.p, &mut out);
                std::hint::black_box(&out);
                acc.add(&tm);
            }
            per_arm[arm].push(acc);
        }
    }
    let nb = blocks.len() as f64;
    let p50 = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() / 2]
    };
    let mut base_tot = 1.0f64;
    for arm in 0..NUM_ARMS {
        let mut s1: Vec<f64> = per_arm[arm].iter().map(|t| t.s1 / nb).collect();
        let mut s2a: Vec<f64> = per_arm[arm].iter().map(|t| t.s2a / nb).collect();
        let mut s2b: Vec<f64> = per_arm[arm].iter().map(|t| t.s2b / nb).collect();
        let mut s3a: Vec<f64> = per_arm[arm].iter().map(|t| t.s3a / nb).collect();
        let mut s4: Vec<f64> = per_arm[arm].iter().map(|t| t.s4 / nb).collect();
        let mut s5: Vec<f64> = per_arm[arm].iter().map(|t| t.s5 / nb).collect();
        let mut tot: Vec<f64> = per_arm[arm].iter().map(|t| t.total / nb).collect();
        let t_med = p50(&mut tot);
        if arm == 0 {
            base_tot = t_med;
        }
        println!(
            "{:<20} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>8.1} {:>6.3}x",
            ARM_NAMES[arm],
            p50(&mut s1),
            p50(&mut s2a),
            p50(&mut s2b),
            p50(&mut s3a),
            p50(&mut s4),
            p50(&mut s5),
            t_med,
            t_med / base_tot
        );
    }
}

fn bench_multiworker(blocks: &[Block], label: &str, rounds: usize, iters: usize, arms: &[usize]) {
    println!(
        "\n=== Paired Multi-Worker Timings ({} rounds x {} iters): {} ({} blocks) ===",
        rounds,
        iters,
        label,
        blocks.len()
    );
    for &workers in &[1usize, 2, 4] {
        let mut res: Vec<Vec<f64>> = (0..arms.len()).map(|_| Vec::with_capacity(rounds)).collect();
        for r in 0..rounds {
            for off in 0..arms.len() {
                let slot = (off + r) % arms.len();
                let arm = arms[slot];
                let bar = Barrier::new(workers);
                let t_wall = std::thread::scope(|s| {
                    let mut handles = Vec::with_capacity(workers);
                    for _ in 0..workers {
                        let bar = &bar;
                        handles.push(s.spawn(move || {
                            let mut k = AblationKernel::default();
                            let mut out = Vec::new();
                            let warm_n = blocks.len().min(8);
                            for blk in &blocks[..warm_n] {
                                run_arm::<false, false>(arm, &mut k, &blk.records, blk.n, blk.p, &mut out);
                            }
                            bar.wait();
                            let t0 = Instant::now();
                            for _ in 0..iters {
                                for blk in blocks {
                                    run_arm::<false, false>(arm, &mut k, &blk.records, blk.n, blk.p, &mut out);
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
                res[slot].push(t_wall);
            }
        }
        let p50 = |v: &[f64]| {
            let mut s = v.to_vec();
            s.sort_by(|a, b| a.total_cmp(b));
            s[s.len() / 2]
        };
        print!("  {workers}W: ");
        for (slot, &arm) in arms.iter().enumerate() {
            let m = p50(&res[slot]);
            if slot == 0 {
                print!("{}={:.2}ms ", ARM_NAMES[arm], m);
            } else {
                let mut pr: Vec<f64> = (0..rounds).map(|r| res[slot][r] / res[0][r]).collect();
                pr.sort_by(|a, b| a.total_cmp(b));
                print!("| {}={:.2}({:.3}x) ", ARM_NAMES[arm], m, pr[rounds / 2]);
            }
        }
        println!();
    }
}

fn select_proxy_blocks(wiki431: &[Block], ids: &[u32]) -> Vec<Block> {
    let mut out = Vec::new();
    for &want_id in ids {
        if let Some(b) = wiki431.iter().find(|b| b.idx == want_id) {
            out.push(Block {
                name: b.name.clone(),
                idx: b.idx,
                n: b.n,
                p: b.p,
                records: b.records.clone(),
            });
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let wiki431_path = resolve_path(&[
        "/tmp/mibz2_corpora/wikidata_431_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/wikidata_431_blocks.bin",
    ]);

    // Fast single-arm profile mode for valgrind callgrind / cachegrind exact Ir & LL analysis
    if let Some(pos) = args.iter().position(|a| a == "--profile-arm") {
        let arm: usize = args
            .get(pos + 1)
            .expect("missing arm idx")
            .parse()
            .expect("parse arm idx");
        assert!(arm < NUM_ARMS);
        let iters: usize = args
            .iter()
            .position(|a| a == "--iters")
            .and_then(|p| args.get(p + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let blocks = if args.iter().any(|a| a == "--adm6") {
            load_six_admitted()
        } else if args.iter().any(|a| a == "--proxy6") {
            let wiki431 = load_bin_blocks(&wiki431_path, None, false);
            select_proxy_blocks(&wiki431, &[34, 108, 178, 254, 327, 391])
        } else if args.iter().any(|a| a == "--tiny4") {
            let wiki431 = load_bin_blocks(&wiki431_path, None, false);
            select_proxy_blocks(&wiki431, &[60, 163, 265, 372])
        } else {
            // Default: exact #729 6+4 proxy set!
            let wiki431 = load_bin_blocks(&wiki431_path, None, false);
            select_proxy_blocks(&wiki431, &[34, 108, 178, 254, 327, 391, 60, 163, 265, 372])
        };
        let mut k = AblationKernel::default();
        let mut out = Vec::new();
        let mut checksum = 0u64;
        for _ in 0..iters {
            for blk in &blocks {
                let (ok, _, _) = run_arm::<false, false>(arm, &mut k, &blk.records, blk.n, blk.p, &mut out);
                assert!(ok);
                checksum = checksum.wrapping_add(u64::from(out[0]) + u64::from(out[out.len() - 1]));
                std::hint::black_box(&out);
            }
        }
        println!(
            "PROFILE_DONE arm={} ({}) blocks={} iters={} chk={}",
            arm,
            ARM_NAMES[arm],
            blocks.len(),
            iters,
            checksum
        );
        return;
    }

    let wiki6 = load_six_admitted();
    let nonwiki_path = resolve_path(&[
        "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/non_wikidata_blocks.bin",
    ]);
    let codedocs_path = resolve_path(&[
        "/tmp/mibz2_corpora/non_wikidata/code_docs_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/code_docs_blocks.bin",
    ]);
    let nw_names = ["jsonl_0", "jsonl_1", "jsonl_2", "jsonl_3", "jsonl_4", "syslog_0"];
    let nonwiki6 = load_bin_blocks(&nonwiki_path, Some(&nw_names), false);
    let cd_names = ["code_docs_0", "code_docs_1", "code_docs_2", "code_docs_3"];
    let code4 = load_bin_blocks(&codedocs_path, Some(&cd_names), false);
    let wiki233 = load_bin_blocks(&wiki431_path, None, true);
    let wiki431 = load_bin_blocks(&wiki431_path, None, false);
    let proxy10 = select_proxy_blocks(&wiki431, &[34, 108, 178, 254, 327, 391, 60, 163, 265, 372]);
    let proxy6 = select_proxy_blocks(&wiki431, &[34, 108, 178, 254, 327, 391]);
    let stress = make_stress_blocks();

    let nonwiki10: Vec<Block> = nonwiki6
        .iter()
        .chain(code4.iter())
        .map(|b| Block {
            name: b.name.clone(),
            idx: b.idx,
            n: b.n,
            p: b.p,
            records: b.records.clone(),
        })
        .collect();

    run_exactness_suite(&wiki6, &nonwiki10, &wiki431, &stress);
    if args.iter().any(|a| a == "--exact-only") {
        return;
    }

    audit_ws(&proxy10, "6+4 #729 Proxy Blocks");
    audit_ws(&wiki431, "431 ALL wikidata.bz2");
    audit_ws(&nonwiki6, "6 Non-Wikidata (JSONL + Syslog)");
    audit_ws(&code4, "4 Code & Docs");
    audit_ws(&stress, "6 Stress Blocks");

    bench_stage_1t(&proxy6, "6 Full #729 Proxy Blocks", 11);
    bench_stage_1t(&wiki6, "6 Admitted Wikidata", 11);
    bench_stage_1t(&nonwiki6, "6 Non-Wikidata (JSONL + Syslog)", 11);
    bench_stage_1t(&code4, "4 Code & Docs", 7);

    let key_arms = [0, 7, 9, 10, 11, 12, 15, 16];
    bench_multiworker(&proxy10, "6+4 #729 Proxy Blocks", 7, 3, &key_arms);
    bench_multiworker(&wiki6, "6 Admitted Wikidata", 7, 3, &key_arms);
    bench_multiworker(&nonwiki6, "6 Non-Wikidata (JSONL + Syslog)", 7, 5, &key_arms);
    bench_multiworker(&code4, "4 Code & Docs", 5, 2, &key_arms);
    bench_multiworker(&wiki233, "233 Full wikidata.bz2", 3, 1, &key_arms);
    bench_multiworker(&wiki431, "431 ALL wikidata.bz2", 3, 1, &key_arms);
}
