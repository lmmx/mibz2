#![forbid(unsafe_code)]

//! Cycle 54 P3 — `C54-PipeLean-v2` Single-Feature & Fusion Ablation Harness (`A00`–`A15`):
//! Systematically ablates orthogonal `S1`–`S5` & `U16Dict` optimizations on top of Cycle 53's
//! undisputed `#729` champion `C53-PipeLean-v1` (`A00`, with `A01` as an exact A/A control):
//! 1. `A02_U16Dict`: Spec's `4 KiB` stack-local `u16` `Dictionary::contents` (`[u16; 2048]`,
//!    `-50%` table, lossless `contents_u32_fb` fallback for `code >= 4,096`, zero `CACHE_WORDS`
//!    `edges` tail growth) — directly targets `L2 = 24.8 B/B` while preserving `L3 <= 14.5` &
//!    `W=8 DRAM <= 1.29`.
//! 2. `A03_S5_ZeroSpill`: Zero-spill in-place `ef`/`eb` `Pipe1` + Const-`BITS` (`17/18/19`)
//!    specialization in `emit_bidirectional` (eliminates 13 stack spill/reload insns/iter!).
//! 3. `A04_S5_ColdPipe1In`: P1's promoted `CS-40` `Cold-Pipe1+In` (`1-step XOR + hoisted
//!    `lens`/`inline16` pre-reads`) fused into `SlimS5_Pipe1_Rem16_LE16` (`+0 B` WS).
//! 4. `A05_S5_Pipe2`: P1's `S-2Cur-Pipe2` (`2-step XOR lookahead`) inside `SlimS5_Rem16_LE16`.
//! 5. `A06_S5_BackU64`: `lb <= 8` `u64` backward right-align shift + `& 63` shift bound on
//!    `9..=16` (`4` insns vs `14` insns for `u128` shift, `+0 B` WS!).
//! 6. `A07_S5_LitFast`: Direct 1-byte literal fast path (`cf/cb < 256`) + `BackU64` (`+0 B` WS;
//!    tests `Ir` reduction vs branch-prediction cost on Xeon).
//! 7. `A08_S4_SplitEpoch`: Split Pass-1 AVX2 sequential XOR + Pass-2 `[u32; 65536]` epoch-slice
//!    scattered XOR (exact by commutativity/associativity of `(Z_2^32, ^)` — eliminates 100% of
//!    scattered bounds checks & `hi | lo` OR ops in full epochs!).
//! 8. `A09_S4_Slim16`: Spec's 16-way `SlimS4_16` (direct A/B vs `A08_S4_SplitEpoch`).
//! 9. `A10_S1_HoistOutM`: Hoists `m` & `out: &mut [u8]` out of `Pass`/`Queues` in `stage1_scan`
//!    + cold-outlined `grow_edges_s1_step` (eliminates 4 per-record stack load/stores!).
//! 10. `A11_S2b_JoinU64`: `u64` fast path for `sum <= 8` in `Elim::join()` + `debug_assert!`
//!     in `eliminate_narrow_plain`.
//! 11. `A12`–`A15`: Fused candidate combinations (`BackU64+ColdPipe1In`, `Fused_Core`,
//!     `Fused_U16Dict`, `Fused_U16_CIn`).
//!
//! Strictly honest `2020s` modern CS systems engineering (`0` classical relabels!).
//! CLI modes:
//!   - `--exact-only`: run full exactness suite (`5,760` valid + `330` malformed + `65` detached
//!     `d=1..65` + forced & natural `sentinel` + forced & natural `u16` dict fallback + `3` stress
//!     + `447/447` real blocks across all arms).
//!   - `--profile-arm <idx> [--proxy10|--proxy6|--adm6|--blk34|--blk254] [--warmup] [--iters <N>]`
//!     (calls `#[inline(never)] profile_exec` for zero-noise `callgrind` L2 & 2M LLC profiling).

use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::time::Instant;

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
const CACHE_WORDS: usize = PAIR_WORDS + 2048;
const U16_DICT_CAP: u32 = 4_096;

const ID_BITS: u32 = 24;
const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS; // 255

// Const-generic feature flags
const BIT_U16_DICT: u32 = 1 << 0;
const BIT_S5_ZERO_SPILL: u32 = 1 << 1;
const BIT_S5_COLD_PIPE1_IN: u32 = 1 << 2;
const BIT_S5_PIPE2: u32 = 1 << 3;
const BIT_S5_BACK_U64: u32 = 1 << 4;
const BIT_S5_LIT_FAST: u32 = 1 << 5;
const BIT_S4_SPLIT_EPOCH: u32 = 1 << 6;
const BIT_S4_SLIM16: u32 = 1 << 7;
const BIT_S1_LOCAL_M: u32 = 1 << 8;
const BIT_S2B_DBG_ASSERT: u32 = 1 << 9;
const BIT_S5_UNAL_BACK: u32 = 1 << 10;
const BIT_S4_8WAY_EPOCH: u32 = 1 << 11;

const CFG_A00: u32 = 0;
const CFG_A01: u32 = 0;
const CFG_A02: u32 = BIT_U16_DICT;
const CFG_A03: u32 = BIT_S5_ZERO_SPILL;
const CFG_A04: u32 = BIT_S5_ZERO_SPILL | BIT_S5_COLD_PIPE1_IN;
const CFG_A05: u32 = BIT_S5_ZERO_SPILL | BIT_S5_PIPE2;
const CFG_A06: u32 = BIT_S5_ZERO_SPILL | BIT_S5_BACK_U64;
const CFG_A07: u32 = BIT_S5_ZERO_SPILL | BIT_S5_BACK_U64 | BIT_S5_LIT_FAST;
const CFG_A08: u32 = BIT_S5_ZERO_SPILL | BIT_S5_UNAL_BACK;
const CFG_A09: u32 = BIT_S5_ZERO_SPILL | BIT_S5_UNAL_BACK | BIT_S5_PIPE2;
const CFG_A10: u32 = BIT_S5_ZERO_SPILL | BIT_S5_UNAL_BACK | BIT_S5_COLD_PIPE1_IN;
const CFG_A11: u32 = BIT_S4_8WAY_EPOCH;
const CFG_A12: u32 = BIT_S4_SPLIT_EPOCH;
const CFG_A13: u32 = BIT_S4_SLIM16;
const CFG_A14: u32 = BIT_S1_LOCAL_M;
const CFG_A15: u32 = BIT_S2B_DBG_ASSERT;
const CFG_A16: u32 = BIT_S4_8WAY_EPOCH | BIT_S5_ZERO_SPILL | BIT_S5_UNAL_BACK;
const CFG_A17: u32 = CFG_A16 | BIT_S2B_DBG_ASSERT;
const CFG_A18: u32 = CFG_A16 | BIT_U16_DICT;
const CFG_A19: u32 = BIT_S4_8WAY_EPOCH | BIT_S5_ZERO_SPILL | BIT_S5_UNAL_BACK | BIT_S5_PIPE2;

const NUM_ARMS: usize = 20;
const ARM_NAMES: [&str; NUM_ARMS] = [
    "A00_C53_PipeLean_v1",
    "A01_AA_Control",
    "A02_U16Dict",
    "A03_S5_ZeroSpill",
    "A04_S5_ColdPipe1In",
    "A05_S5_Pipe2",
    "A06_S5_BackU64",
    "A07_S5_LitFast",
    "A08_S5_UnalignedBack",
    "A09_S5_UnalBack_Pipe2",
    "A10_S5_UnalBack_CIn",
    "A11_S4_8WayEpoch",
    "A12_S4_SplitEpoch",
    "A13_S4_Slim16",
    "A14_S1_LocalM",
    "A15_S2b_DbgAssert",
    "A16_C54_v2_Clean",
    "A17_C54_v2_Dbg",
    "A18_C54_v2_U16Dict",
    "A19_C54_v2_Pipe2",
];

#[inline(always)]
const fn has(cfg: u32, bit: u32) -> bool {
    (cfg & bit) != 0
}

#[inline(always)]
const fn literal(byte: u8) -> u32 {
    (1u32 << ID_BITS) | (byte as u32)
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
    if inline16.len() < 257 {
        inline16.clear();
        inline16.reserve_exact(257);
        inline16.push([0; INLINE]); // +16 B leading guard for zero-shift UnalignedBack emit
        inline16.extend((0..=255u8).map(|c| {
            let mut v = [0; INLINE];
            v[0] = c;
            v
        }));
        lens.clear();
        lens.reserve_exact(256);
        lens.resize(256, 1);
    }
    inline16.truncate(257);
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
    inline16.reserve_exact((target + 1).saturating_sub(inline16.len()));
    lens.reserve_exact(target.saturating_sub(lens.len()));
}

#[inline(always)]
fn ensure_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
    if lens.len() == lens.capacity() || inline16.len() == inline16.capacity() {
        grow_step_inline(inline16, lens, step);
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
fn grow_edges_step(a: &mut Vec<u32>, need: usize) {
    let target = need.div_ceil(8192) * 8192;
    if target > a.capacity() {
        a.reserve_exact(target - a.len());
    }
    a.resize(target, 0);
}

#[inline(always)]
fn should_use_bi(m: u32, ns: u32, r_max: u32) -> bool {
    if m < 65_536 {
        return false;
    }
    let md_w1 = 2i64 * (i64::from(r_max) - i64::from(ns)) - i64::from(m);
    !((r_max <= 50_000 && ns <= 2_100 && md_w1 <= 16_000) || (md_w1 < -18_000 && ns <= 2_400))
}

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

#[cold]
#[inline(never)]
fn try_short_periodic(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    if records.len() * 64 >= n || n < 4096 || (p as usize) >= n {
        return false;
    }
    let mut sym = [0u8; 64];
    let mut r_start = [0u32; 64];
    let mut r_rank = [0u32; 64];
    let mut counts = [0u32; 256];
    let mut nr = 0usize;
    let mut pos = 0u32;
    for &rec in records {
        let b = (rec >> 8) as u8;
        let l = u32::from(rec & 255);
        if nr == 0 || sym[nr - 1] != b {
            if nr >= 64 {
                return false;
            }
            sym[nr] = b;
            r_start[nr] = pos;
            r_rank[nr] = counts[b as usize];
            nr += 1;
        }
        counts[b as usize] += l;
        pos += l;
    }
    let mut c_start = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        c_start[c] = tot;
        tot += counts[c];
    }
    if tot as usize != n {
        return false;
    }
    let mut cyc = [0u8; 64];
    let mut x = p;
    for step in 0..64usize {
        let mut lo = 0usize;
        let mut hi = nr;
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            if r_start[mid] <= x {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let b = sym[lo];
        cyc[step] = b;
        x = c_start[b as usize] + r_rank[lo] + (x - r_start[lo]);
        if x == p {
            let d = step + 1;
            if n % d == 0 {
                out.clear();
                out.reserve(n);
                for k in (0..d).rev() {
                    out.push(cyc[k]);
                }
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

// ============================================================================
// OutQ16 (CHUNK=512) with Hoisted-Out variant + SplitEpoch & SlimS4_16 Readers
// ============================================================================
mod outq {
    const CHUNK: usize = 512;
    pub(super) const BYTES: usize = 2 * CHUNK;
    const NONE32: u32 = u32::MAX;
    const NONE16: u16 = u16::MAX;
    const EPOCH: u32 = 1 << 16;

    #[derive(Default)]
    pub(super) struct Spill {
        next: Vec<u16>,
        ovf: Vec<[u8; BYTES]>,
        cuts: Vec<[u32; 256]>,
    }

    impl Spill {
        pub(super) fn cap_bytes(&self) -> usize {
            self.next.capacity() * 2 + self.ovf.capacity() * BYTES + self.cuts.capacity() * 1024
        }
        pub(super) fn used_bytes(&self) -> usize {
            self.next.len() * 2 + self.ovf.len() * BYTES + self.cuts.len() * 1024
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
        pub(super) chunks: u32,
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
        pub(super) fn push_thin<const MAX_W: u32>(&mut self, a: usize, m: u32, w: u32) {
            let v = self.pt[a];
            let p = v as u32;
            let f = (p & 1023) as usize;
            let ok_w = if MAX_W <= 16 { true } else { w <= 16 };
            if ok_w & (f <= CHUNK - 16) & ((m as u16) < 65_520) {
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
                if self.spill.ovf.len() == self.spill.ovf.capacity() {
                    let target = (self.spill.ovf.len() + 1).div_ceil(16) * 16;
                    self.spill.ovf.reserve_exact(target - self.spill.ovf.len());
                }
                self.spill.ovf.push([0; BYTES]);
            }
            let idu = id as usize;
            if self.spill.next.len() <= idu {
                let target = (idu + 1).div_ceil(256) * 256;
                if target > self.spill.next.capacity() {
                    self.spill
                        .next
                        .reserve_exact(target - self.spill.next.len());
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
        fn next_chunk(&self, id: usize) -> u32 {
            u32::from(self.spill.next[id])
        }

        #[inline(always)]
        fn cut_at(&self, epoch: usize) -> u32 {
            self.spill.cuts.get(epoch).map_or(u32::MAX, |c| c[self.a])
        }

        /// S4 Mode 0 (`C53-PipeLean-v1` baseline: 8-way SlimS4 + SlowHi0)
        /// S4 Mode 1 (`BIT_S4_SPLIT_EPOCH`): Pass-1 AVX2 sequential XOR + Pass-2 `[u32; 65536]` epoch-slice!
        /// S4 Mode 2 (`BIT_S4_SLIM16`): Spec's 16-way SlimS4_16!
        #[inline(always)]
        pub(super) fn wire_xor_nomark<const MODE: u8>(
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
                let hi_u32 = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                if MODE == 3 {
                    // `S4_8WayEpoch`: 8-way interleaved AVX2 sequential + `[u32; 65536]` epoch-slice!
                    let hi = hi_u32 as usize;
                    let pack_hi = pack | hi_u32;
                    let mut i = 0usize;
                    if hi + (EPOCH as usize) <= edges.len() {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = usize::from(u16::from_le_bytes(ch[0]));
                            let l1 = usize::from(u16::from_le_bytes(ch[1]));
                            let l2 = usize::from(u16::from_le_bytes(ch[2]));
                            let l3 = usize::from(u16::from_le_bytes(ch[3]));
                            let l4 = usize::from(u16::from_le_bytes(ch[4]));
                            let l5 = usize::from(u16::from_le_bytes(ch[5]));
                            let l6 = usize::from(u16::from_le_bytes(ch[6]));
                            let l7 = usize::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | (l0 as u32);
                            eu[1] ^= pack_hi | (l1 as u32);
                            eu[2] ^= pack_hi | (l2 as u32);
                            eu[3] ^= pack_hi | (l3 as u32);
                            eu[4] ^= pack_hi | (l4 as u32);
                            eu[5] ^= pack_hi | (l5 as u32);
                            eu[6] ^= pack_hi | (l6 as u32);
                            eu[7] ^= pack_hi | (l7 as u32);
                            let ep: &mut [u32; EPOCH as usize] =
                                (&mut edges[hi..hi + (EPOCH as usize)]).try_into().unwrap();
                            let u32_0 = u0 as u32;
                            ep[l0] ^= u32_0;
                            ep[l1] ^= u32_0 + 1;
                            ep[l2] ^= u32_0 + 2;
                            ep[l3] ^= u32_0 + 3;
                            ep[l4] ^= u32_0 + 4;
                            ep[l5] ^= u32_0 + 5;
                            ep[l6] ^= u32_0 + 6;
                            ep[l7] ^= u32_0 + 7;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = usize::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | (lo as u32);
                            let ep: &mut [u32; EPOCH as usize] =
                                (&mut edges[hi..hi + (EPOCH as usize)]).try_into().unwrap();
                            ep[lo] ^= u as u32;
                            i += 1;
                        }
                    } else {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = usize::from(u16::from_le_bytes(ch[0]));
                            let l1 = usize::from(u16::from_le_bytes(ch[1]));
                            let l2 = usize::from(u16::from_le_bytes(ch[2]));
                            let l3 = usize::from(u16::from_le_bytes(ch[3]));
                            let l4 = usize::from(u16::from_le_bytes(ch[4]));
                            let l5 = usize::from(u16::from_le_bytes(ch[5]));
                            let l6 = usize::from(u16::from_le_bytes(ch[6]));
                            let l7 = usize::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | (l0 as u32);
                            eu[1] ^= pack_hi | (l1 as u32);
                            eu[2] ^= pack_hi | (l2 as u32);
                            eu[3] ^= pack_hi | (l3 as u32);
                            eu[4] ^= pack_hi | (l4 as u32);
                            eu[5] ^= pack_hi | (l5 as u32);
                            eu[6] ^= pack_hi | (l6 as u32);
                            eu[7] ^= pack_hi | (l7 as u32);
                            let ep = &mut edges[hi..];
                            let u32_0 = u0 as u32;
                            ep[l0] ^= u32_0;
                            ep[l1] ^= u32_0 + 1;
                            ep[l2] ^= u32_0 + 2;
                            ep[l3] ^= u32_0 + 3;
                            ep[l4] ^= u32_0 + 4;
                            ep[l5] ^= u32_0 + 5;
                            ep[l6] ^= u32_0 + 6;
                            ep[l7] ^= u32_0 + 7;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = usize::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | (lo as u32);
                            edges[hi + lo] ^= u as u32;
                            i += 1;
                        }
                    }
                } else if MODE == 1 {
                    let hi = hi_u32 as usize;
                    let pack_hi = pack | hi_u32;
                    for (eu, &p) in edges[dst_start..dst_start + span].iter_mut().zip(pairs) {
                        *eu ^= pack_hi | u32::from(u16::from_le_bytes(p));
                    }
                    let mut u = dst_start as u32;
                    if hi + (EPOCH as usize) <= edges.len() {
                        let ep: &mut [u32; EPOCH as usize] =
                            (&mut edges[hi..hi + (EPOCH as usize)]).try_into().unwrap();
                        for &p in pairs {
                            ep[usize::from(u16::from_le_bytes(p))] ^= u;
                            u += 1;
                        }
                    } else {
                        let ep = &mut edges[hi..];
                        for &p in pairs {
                            ep[usize::from(u16::from_le_bytes(p))] ^= u;
                            u += 1;
                        }
                    }
                } else if MODE == 2 {
                    let mut i = 0usize;
                    if hi_u32 == 0 {
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
                            eu[0] ^= pack | v0;
                            eu[1] ^= pack | v1;
                            eu[2] ^= pack | v2;
                            eu[3] ^= pack | v3;
                            eu[4] ^= pack | v4;
                            eu[5] ^= pack | v5;
                            eu[6] ^= pack | v6;
                            eu[7] ^= pack | v7;
                            eu[8] ^= pack | v8;
                            eu[9] ^= pack | v9;
                            eu[10] ^= pack | v10;
                            eu[11] ^= pack | v11;
                            eu[12] ^= pack | v12;
                            eu[13] ^= pack | v13;
                            eu[14] ^= pack | v14;
                            eu[15] ^= pack | v15;
                            edges[v0 as usize] ^= u0 as u32;
                            edges[v1 as usize] ^= (u0 + 1) as u32;
                            edges[v2 as usize] ^= (u0 + 2) as u32;
                            edges[v3 as usize] ^= (u0 + 3) as u32;
                            edges[v4 as usize] ^= (u0 + 4) as u32;
                            edges[v5 as usize] ^= (u0 + 5) as u32;
                            edges[v6 as usize] ^= (u0 + 6) as u32;
                            edges[v7 as usize] ^= (u0 + 7) as u32;
                            edges[v8 as usize] ^= (u0 + 8) as u32;
                            edges[v9 as usize] ^= (u0 + 9) as u32;
                            edges[v10 as usize] ^= (u0 + 10) as u32;
                            edges[v11 as usize] ^= (u0 + 11) as u32;
                            edges[v12 as usize] ^= (u0 + 12) as u32;
                            edges[v13 as usize] ^= (u0 + 13) as u32;
                            edges[v14 as usize] ^= (u0 + 14) as u32;
                            edges[v15 as usize] ^= (u0 + 15) as u32;
                            i += 16;
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
                        let pack_hi = pack | hi_u32;
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
                            eu[0] ^= pack_hi | l0;
                            eu[1] ^= pack_hi | l1;
                            eu[2] ^= pack_hi | l2;
                            eu[3] ^= pack_hi | l3;
                            eu[4] ^= pack_hi | l4;
                            eu[5] ^= pack_hi | l5;
                            eu[6] ^= pack_hi | l6;
                            eu[7] ^= pack_hi | l7;
                            eu[8] ^= pack_hi | l8;
                            eu[9] ^= pack_hi | l9;
                            eu[10] ^= pack_hi | l10;
                            eu[11] ^= pack_hi | l11;
                            eu[12] ^= pack_hi | l12;
                            eu[13] ^= pack_hi | l13;
                            eu[14] ^= pack_hi | l14;
                            eu[15] ^= pack_hi | l15;
                            edges[(hi_u32 | l0) as usize] ^= u0 as u32;
                            edges[(hi_u32 | l1) as usize] ^= (u0 + 1) as u32;
                            edges[(hi_u32 | l2) as usize] ^= (u0 + 2) as u32;
                            edges[(hi_u32 | l3) as usize] ^= (u0 + 3) as u32;
                            edges[(hi_u32 | l4) as usize] ^= (u0 + 4) as u32;
                            edges[(hi_u32 | l5) as usize] ^= (u0 + 5) as u32;
                            edges[(hi_u32 | l6) as usize] ^= (u0 + 6) as u32;
                            edges[(hi_u32 | l7) as usize] ^= (u0 + 7) as u32;
                            edges[(hi_u32 | l8) as usize] ^= (u0 + 8) as u32;
                            edges[(hi_u32 | l9) as usize] ^= (u0 + 9) as u32;
                            edges[(hi_u32 | l10) as usize] ^= (u0 + 10) as u32;
                            edges[(hi_u32 | l11) as usize] ^= (u0 + 11) as u32;
                            edges[(hi_u32 | l12) as usize] ^= (u0 + 12) as u32;
                            edges[(hi_u32 | l13) as usize] ^= (u0 + 13) as u32;
                            edges[(hi_u32 | l14) as usize] ^= (u0 + 14) as u32;
                            edges[(hi_u32 | l15) as usize] ^= (u0 + 15) as u32;
                            i += 16;
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
                            eu[0] ^= pack_hi | l0;
                            eu[1] ^= pack_hi | l1;
                            eu[2] ^= pack_hi | l2;
                            eu[3] ^= pack_hi | l3;
                            eu[4] ^= pack_hi | l4;
                            eu[5] ^= pack_hi | l5;
                            eu[6] ^= pack_hi | l6;
                            eu[7] ^= pack_hi | l7;
                            edges[(hi_u32 | l0) as usize] ^= u0 as u32;
                            edges[(hi_u32 | l1) as usize] ^= (u0 + 1) as u32;
                            edges[(hi_u32 | l2) as usize] ^= (u0 + 2) as u32;
                            edges[(hi_u32 | l3) as usize] ^= (u0 + 3) as u32;
                            edges[(hi_u32 | l4) as usize] ^= (u0 + 4) as u32;
                            edges[(hi_u32 | l5) as usize] ^= (u0 + 5) as u32;
                            edges[(hi_u32 | l6) as usize] ^= (u0 + 6) as u32;
                            edges[(hi_u32 | l7) as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | lo;
                            edges[(hi_u32 | lo) as usize] ^= u as u32;
                            i += 1;
                        }
                    }
                } else {
                    let mut i = 0usize;
                    if hi_u32 == 0 {
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
                        let pack_hi = pack | hi_u32;
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
                            edges[(hi_u32 | l0) as usize] ^= u0 as u32;
                            edges[(hi_u32 | l1) as usize] ^= (u0 + 1) as u32;
                            edges[(hi_u32 | l2) as usize] ^= (u0 + 2) as u32;
                            edges[(hi_u32 | l3) as usize] ^= (u0 + 3) as u32;
                            edges[(hi_u32 | l4) as usize] ^= (u0 + 4) as u32;
                            edges[(hi_u32 | l5) as usize] ^= (u0 + 5) as u32;
                            edges[(hi_u32 | l6) as usize] ^= (u0 + 6) as u32;
                            edges[(hi_u32 | l7) as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | lo;
                            edges[(hi_u32 | lo) as usize] ^= u as u32;
                            i += 1;
                        }
                    }
                }
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_xor_slow_nomark::<MODE>(edges, dst_start, span, pack);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_slow_nomark<const MODE: u8>(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack: u32,
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
                let hi_u32 = self.epoch << 16;
                let room_chunk = CHUNK - self.at;
                let room_cut = (self.cut - self.read) as usize;
                let take = left.min(room_chunk).min(room_cut);
                let chunk: &[u8] = if self.id < self.in_out {
                    let base = self.id as usize * BYTES;
                    &self.out[base..base + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                if MODE == 3 {
                    let hi = hi_u32 as usize;
                    let pack_hi = pack | hi_u32;
                    let mut i = 0usize;
                    if hi + (EPOCH as usize) <= edges.len() {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = usize::from(u16::from_le_bytes(ch[0]));
                            let l1 = usize::from(u16::from_le_bytes(ch[1]));
                            let l2 = usize::from(u16::from_le_bytes(ch[2]));
                            let l3 = usize::from(u16::from_le_bytes(ch[3]));
                            let l4 = usize::from(u16::from_le_bytes(ch[4]));
                            let l5 = usize::from(u16::from_le_bytes(ch[5]));
                            let l6 = usize::from(u16::from_le_bytes(ch[6]));
                            let l7 = usize::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | (l0 as u32);
                            eu[1] ^= pack_hi | (l1 as u32);
                            eu[2] ^= pack_hi | (l2 as u32);
                            eu[3] ^= pack_hi | (l3 as u32);
                            eu[4] ^= pack_hi | (l4 as u32);
                            eu[5] ^= pack_hi | (l5 as u32);
                            eu[6] ^= pack_hi | (l6 as u32);
                            eu[7] ^= pack_hi | (l7 as u32);
                            let ep: &mut [u32; EPOCH as usize] =
                                (&mut edges[hi..hi + (EPOCH as usize)]).try_into().unwrap();
                            let u32_0 = u0 as u32;
                            ep[l0] ^= u32_0;
                            ep[l1] ^= u32_0 + 1;
                            ep[l2] ^= u32_0 + 2;
                            ep[l3] ^= u32_0 + 3;
                            ep[l4] ^= u32_0 + 4;
                            ep[l5] ^= u32_0 + 5;
                            ep[l6] ^= u32_0 + 6;
                            ep[l7] ^= u32_0 + 7;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = usize::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | (lo as u32);
                            let ep: &mut [u32; EPOCH as usize] =
                                (&mut edges[hi..hi + (EPOCH as usize)]).try_into().unwrap();
                            ep[lo] ^= u as u32;
                            i += 1;
                        }
                    } else {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = usize::from(u16::from_le_bytes(ch[0]));
                            let l1 = usize::from(u16::from_le_bytes(ch[1]));
                            let l2 = usize::from(u16::from_le_bytes(ch[2]));
                            let l3 = usize::from(u16::from_le_bytes(ch[3]));
                            let l4 = usize::from(u16::from_le_bytes(ch[4]));
                            let l5 = usize::from(u16::from_le_bytes(ch[5]));
                            let l6 = usize::from(u16::from_le_bytes(ch[6]));
                            let l7 = usize::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | (l0 as u32);
                            eu[1] ^= pack_hi | (l1 as u32);
                            eu[2] ^= pack_hi | (l2 as u32);
                            eu[3] ^= pack_hi | (l3 as u32);
                            eu[4] ^= pack_hi | (l4 as u32);
                            eu[5] ^= pack_hi | (l5 as u32);
                            eu[6] ^= pack_hi | (l6 as u32);
                            eu[7] ^= pack_hi | (l7 as u32);
                            let ep = &mut edges[hi..];
                            let u32_0 = u0 as u32;
                            ep[l0] ^= u32_0;
                            ep[l1] ^= u32_0 + 1;
                            ep[l2] ^= u32_0 + 2;
                            ep[l3] ^= u32_0 + 3;
                            ep[l4] ^= u32_0 + 4;
                            ep[l5] ^= u32_0 + 5;
                            ep[l6] ^= u32_0 + 6;
                            ep[l7] ^= u32_0 + 7;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = usize::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | (lo as u32);
                            edges[hi + lo] ^= u as u32;
                            i += 1;
                        }
                    }
                } else if MODE == 1 {
                    let hi = hi_u32 as usize;
                    let pack_hi = pack | hi_u32;
                    for (eu, &p) in edges[dst_start..dst_start + take].iter_mut().zip(pairs) {
                        *eu ^= pack_hi | u32::from(u16::from_le_bytes(p));
                    }
                    let mut u = dst_start as u32;
                    if hi + (EPOCH as usize) <= edges.len() {
                        let ep: &mut [u32; EPOCH as usize] =
                            (&mut edges[hi..hi + (EPOCH as usize)]).try_into().unwrap();
                        for &p in pairs {
                            ep[usize::from(u16::from_le_bytes(p))] ^= u;
                            u += 1;
                        }
                    } else {
                        let ep = &mut edges[hi..];
                        for &p in pairs {
                            ep[usize::from(u16::from_le_bytes(p))] ^= u;
                            u += 1;
                        }
                    }
                } else {
                    let mut i = 0usize;
                    if hi_u32 == 0 {
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
                        let pack_hi = pack | hi_u32;
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
                            edges[(hi_u32 | l0) as usize] ^= u0 as u32;
                            edges[(hi_u32 | l1) as usize] ^= (u0 + 1) as u32;
                            edges[(hi_u32 | l2) as usize] ^= (u0 + 2) as u32;
                            edges[(hi_u32 | l3) as usize] ^= (u0 + 3) as u32;
                            edges[(hi_u32 | l4) as usize] ^= (u0 + 4) as u32;
                            edges[(hi_u32 | l5) as usize] ^= (u0 + 5) as u32;
                            edges[(hi_u32 | l6) as usize] ^= (u0 + 6) as u32;
                            edges[(hi_u32 | l7) as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | lo;
                            edges[(hi_u32 | lo) as usize] ^= u as u32;
                            i += 1;
                        }
                    }
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
                let pack_hi = pack | hi;
                for (i, &p) in pairs.iter().enumerate() {
                    let u = dst_start + i;
                    let lo = u32::from(u16::from_le_bytes(p));
                    let v = hi | lo;
                    edges[u] ^= pack_hi | lo;
                    edges[v as usize] ^= u as u32;
                    if v == mark {
                        *predecessor = u as u32;
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
                let hi = self.epoch << 16;
                let room_chunk = CHUNK - self.at;
                let room_cut = (self.cut - self.read) as usize;
                let take = left.min(room_chunk).min(room_cut);
                let chunk: &[u8] = if self.id < self.in_out {
                    let base = self.id as usize * BYTES;
                    &self.out[base..base + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                for (i, &p) in pairs.iter().enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes(p));
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                    if v == mark {
                        *predecessor = u as u32;
                    }
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        #[inline(always)]
        pub(super) fn wire_uni(
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
                let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                let pack_hi = pack | hi;
                for (eu, &p) in edges[dst_start..dst_start + span].iter_mut().zip(pairs) {
                    *eu = pack_hi | u32::from(u16::from_le_bytes(p));
                }
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_uni_slow(edges, dst_start, span, pack);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_uni_slow(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack: u32,
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
                let hi = self.epoch << 16;
                let room_chunk = CHUNK - self.at;
                let room_cut = (self.cut - self.read) as usize;
                let take = left.min(room_chunk).min(room_cut);
                let chunk: &[u8] = if self.id < self.in_out {
                    let base = self.id as usize * BYTES;
                    &self.out[base..base + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                let pack_hi = pack | hi;
                for (eu, &p) in edges[dst_start..dst_start + take].iter_mut().zip(pairs) {
                    *eu = pack_hi | u32::from(u16::from_le_bytes(p));
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }
    }
}

// ============================================================================
// Endpoint Elimination (`S2aSlim` + `CE-Var234` + `S2b_JoinU64` + `OrphanFree`)
// ============================================================================
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

struct Elim<'n, 'r, L: Links, const JOIN64: bool> {
    nodes_buf: &'n mut [u32],
    rules_buf: &'r mut [u32],
    rule_top: usize,
    nodes: u32,
    id_overflow: bool,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links, const JOIN64: bool> Elim<'_, '_, L, JOIN64> {
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
        if L::IS_NARROW {
            let (nodes, _) = self.nodes_buf.as_chunks_mut::<4>();
            let mut first = nodes[0][1] >> 16;
            let mut last = nodes[0][1] & 0xFFFF;
            let (mut lo, mut hi) = (0u32, rows);
            while first != last {
                let nf = &mut nodes[first as usize];
                let next = nf[1] >> 16;
                let width = nf[2];
                nf[1] = lo;
                lo += width;
                let nl = &mut nodes[last as usize];
                let prev = nl[1] & 0xFFFF;
                hi -= nl[2];
                nl[1] = hi;
                if next == last {
                    assert_eq!(lo, hi);
                    return;
                }
                first = next;
                last = prev;
            }
            if first != 0 {
                let nf = &mut nodes[first as usize];
                nf[1] = lo;
                lo += nf[2];
            }
            assert_eq!(lo, hi);
            return;
        }
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
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.rules_buf[(h & ID_MASK) as usize + 2]
        }
    }

    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 3 {
            return u128::from(id);
        }
        let r = id as usize;
        if g <= 8 {
            let sl = &self.rules_buf[r..r + 2];
            u128::from(sl[0]) | (u128::from(sl[1]) << 32)
        } else if g <= 12 {
            let sl = &self.rules_buf[r..r + 3];
            u128::from(sl[0]) | (u128::from(sl[1]) << 32) | (u128::from(sl[2]) << 64)
        } else {
            let sl = &self.rules_buf[r..r + 4];
            u128::from(sl[0])
                | (u128::from(sl[1]) << 32)
                | (u128::from(sl[2]) << 64)
                | (u128::from(sl[3]) << 96)
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
            return (sum << ID_BITS)
                | (left & ID_MASK)
                | ((right & ID_MASK) << (8 * left_len));
        }
        let t = self.rule_top;
        let id = t as u32;
        if sum < INLINE as u32 {
            let bytes =
                self.short_payload_h(left) | (self.short_payload_h(right) << (8 * left_len));
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
        let limit = self.rules_buf.len().min(max_id_top);
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.rule_top + MAX_RULE_WORDS > limit {
                if self.rule_top + MAX_RULE_WORDS > max_id_top {
                    self.id_overflow = true;
                }
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
                if JOIN64 {
                    debug_assert!(aw > 0 && aw <= left);
                } else {
                    assert!(aw > 0 && aw <= left);
                }
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
            if JOIN64 {
                debug_assert!(w > 0 && w <= left);
            } else {
                assert!(w > 0 && w <= left);
            }
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
        let limit = self.rules_buf.len().min(max_id_top);
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.rule_top + room > limit {
                if self.rule_top + room > max_id_top {
                    self.id_overflow = true;
                }
                break false;
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
        let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
            (v.to_le_bytes(), (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            let total = self.len_handle(word) as usize;
            ensure_step_u8(spelling, total + 1, SPELLING_STEP);
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
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn append_tail_unw<const TRACK_MEMO: bool>(
        &mut self,
        handle: u32,
        istk: &mut [u32; INLINE_STACK],
        ovf_stack: &mut Vec<u32>,
        dest: &mut Vec<u8>,
        memo_rec: &mut [u32; MEMO_REC_CAP],
        memo_n: &mut usize,
        memo_ovf: &mut bool,
    ) {
        let mut sp = 0usize;
        let mut used_heap = false;
        let mut h = handle;
        loop {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
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
            let r = (h & ID_MASK) as usize;
            let m_off = if g < SATURATED { r + 2 } else { r + 3 };
            let tag = self.rules_buf[m_off];
            if tag != u32::MAX {
                let start = (tag & 0x7fff_ffff) as usize;
                let rlen = if g < SATURATED {
                    g as usize
                } else {
                    self.rules_buf[r + 2] as usize
                };
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
                dest.extend_from_slice(&lb[..lg as usize]);
                let rg = right >> ID_BITS;
                if rg < INLINE as u32 {
                    let rb = self.short_payload_h(right).to_le_bytes();
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

// ============================================================================
// Dictionaries: Baseline `edges`-backed `u32` vs Stack-Local `4 KiB` `u16`
// ============================================================================
struct DictionaryU32<'a> {
    pairs: &'a mut [u32; 512],
    contents: &'a mut [u32; 2048],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    istk: &'a mut [u32; INLINE_STACK],
    memo_rec: &'a mut [u32; MEMO_REC_CAP],
    stack: &'a mut Vec<u32>,
}

impl DictionaryU32<'_> {
    #[inline(always)]
    fn code<L: Links, const J64: bool>(
        &mut self,
        e: &mut Elim<L, J64>,
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

    fn code_miss<L: Links, const J64: bool>(
        &mut self,
        e: &mut Elim<L, J64>,
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
                self.lens[old] == len && self.inline16[old + 1] == entry
            } {
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry, len);
            self.contents[index] = tag | c;
            return c;
        }
        let start = self.spelling.len();
        let g = handle >> ID_BITS;
        let r = id as usize;
        let m_off = if g < SATURATED { r + 2 } else { r + 3 };
        let tag_r = e.rules_buf[m_off];
        if tag_r != u32::MAX {
            let pos = (tag_r & 0x7fff_ffff) as usize;
            let rlen = if g < SATURATED {
                g as usize
            } else {
                e.rules_buf[r + 2] as usize
            };
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
                    && long_word(&self.inline16[old + 1], self.spelling) == bytes
            } {
                self.spelling.truncate(start);
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry, LONG);
            self.contents[index] = tag | c;
            return c;
        }
        let total = if g < SATURATED {
            g as usize
        } else {
            e.rules_buf[r + 2] as usize
        };
        ensure_step_u8(self.spelling, total + 1, SPELLING_STEP);
        let mut memo_n = 0usize;
        let mut memo_ovf = false;
        e.append_tail_unw::<true>(
            handle,
            self.istk,
            self.stack,
            self.spelling,
            self.memo_rec,
            &mut memo_n,
            &mut memo_ovf,
        );
        self.spelling.push(byte);
        let entry = long_slot(start as u32, self.spelling.len() as u32);
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = (h as usize) & 2047;
        let ent = self.contents[index];
        if (ent & 0xFF00_0000) == tag && {
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && long_word(&self.inline16[old + 1], self.spelling) == bytes
        } {
            if !memo_ovf {
                let old = (ent & 0x00FF_FFFF) as usize;
                let old_start =
                    u32::from_le_bytes(self.inline16[old + 1][..4].try_into().unwrap());
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

struct DictionaryU16<'a> {
    pairs: &'a mut [u32; 512],
    contents_u16: &'a mut [u16; 2048],
    contents_u32_fb: &'a mut Vec<u32>,
    u16_cap: u32,
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    istk: &'a mut [u32; INLINE_STACK],
    memo_rec: &'a mut [u32; MEMO_REC_CAP],
    stack: &'a mut Vec<u32>,
}

impl DictionaryU16<'_> {
    #[inline(always)]
    fn probe_contents(&self, index: usize, h: u64) -> Option<usize> {
        if self.contents_u32_fb.is_empty() {
            let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
            let ent = self.contents_u16[index];
            if (ent & 0xF000) == tag4 {
                Some((ent & 0x0FFF) as usize)
            } else {
                None
            }
        } else {
            let ent = self.contents_u32_fb[index];
            if ent != 0 {
                Some(ent as usize)
            } else {
                None
            }
        }
    }

    #[inline(always)]
    fn store_contents(&mut self, index: usize, h: u64, code: u32) {
        if self.contents_u32_fb.is_empty() && code < self.u16_cap {
            let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
            self.contents_u16[index] = tag4 | (code as u16);
        } else {
            Self::store_contents_u32_fallback(
                self.contents_u16,
                self.contents_u32_fb,
                index,
                code,
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn store_contents_u32_fallback(
        tbl: &[u16; 2048],
        fb: &mut Vec<u32>,
        index: usize,
        code: u32,
    ) {
        if fb.is_empty() {
            fb.reserve_exact(2048);
            fb.extend(
                tbl.iter()
                    .map(|&v| if v != 0 { u32::from(v & 0x0FFF) } else { 0 }),
            );
        }
        fb[index] = code;
    }

    #[inline(always)]
    fn code<L: Links, const J64: bool>(
        &mut self,
        e: &mut Elim<L, J64>,
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

    fn code_miss<L: Links, const J64: bool>(
        &mut self,
        e: &mut Elim<L, J64>,
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
            let index = (h as usize) & 2047;
            if let Some(old) = self.probe_contents(index, h) {
                if self.lens[old] == len && self.inline16[old + 1] == entry {
                    return old as u32;
                }
            }
            let c = self.insert(entry, len);
            self.store_contents(index, h, c);
            return c;
        }
        let start = self.spelling.len();
        let g = handle >> ID_BITS;
        let r = id as usize;
        let m_off = if g < SATURATED { r + 2 } else { r + 3 };
        let tag_r = e.rules_buf[m_off];
        if tag_r != u32::MAX {
            let pos = (tag_r & 0x7fff_ffff) as usize;
            let rlen = if g < SATURATED {
                g as usize
            } else {
                e.rules_buf[r + 2] as usize
            };
            ensure_step_u8(self.spelling, rlen + 1, SPELLING_STEP);
            self.spelling.extend_from_within(pos..pos + rlen);
            self.spelling.push(byte);
            let entry = long_slot(start as u32, self.spelling.len() as u32);
            let bytes = &self.spelling[start..];
            let h = spelling_hash_long(bytes);
            let index = (h as usize) & 2047;
            if let Some(old) = self.probe_contents(index, h) {
                if self.lens[old] == LONG
                    && long_word(&self.inline16[old + 1], self.spelling) == bytes
                {
                    self.spelling.truncate(start);
                    return old as u32;
                }
            }
            let c = self.insert(entry, LONG);
            self.store_contents(index, h, c);
            return c;
        }
        let total = if g < SATURATED {
            g as usize
        } else {
            e.rules_buf[r + 2] as usize
        };
        ensure_step_u8(self.spelling, total + 1, SPELLING_STEP);
        let mut memo_n = 0usize;
        let mut memo_ovf = false;
        e.append_tail_unw::<true>(
            handle,
            self.istk,
            self.stack,
            self.spelling,
            self.memo_rec,
            &mut memo_n,
            &mut memo_ovf,
        );
        self.spelling.push(byte);
        let entry = long_slot(start as u32, self.spelling.len() as u32);
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let index = (h as usize) & 2047;
        if let Some(old) = self.probe_contents(index, h) {
            if self.lens[old] == LONG && long_word(&self.inline16[old + 1], self.spelling) == bytes
            {
                if !memo_ovf {
                    let old_start =
                        u32::from_le_bytes(self.inline16[old + 1][..4].try_into().unwrap());
                    let delta = (start as u32) - old_start;
                    for &rm in &self.memo_rec[..memo_n] {
                        e.rules_buf[rm as usize] -= delta;
                    }
                    self.spelling.truncate(start);
                } else {
                    self.spelling.pop();
                }
                return old as u32;
            }
        }
        let c = self.insert(entry, LONG);
        self.store_contents(index, h, c);
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
// 4-Tier Stack Zero-Heap TinyFL
// ============================================================================
fn invert_tiny_fl_with_buf(
    records: &[u16],
    n: usize,
    p: u32,
    fl: &mut [u32],
    out: &mut Vec<u8>,
) -> bool {
    let mut counts = [0u32; 256];
    let mut total = 0usize;
    for &rec in records {
        let len = usize::from(rec & 255);
        counts[usize::from((rec >> 8) as u8)] += len as u32;
        total += len;
    }
    if total != n {
        return false;
    }
    let mut st = starts(counts);
    let mut pos = 0u32;
    for &rec in records {
        let b = (rec >> 8) as u8;
        let len = usize::from(rec & 255);
        let bi = usize::from(b);
        let s0 = st[bi] as usize;
        st[bi] += len as u32;
        let bu = u32::from(b);
        let mut i = pos;
        for slot in &mut fl[s0..s0 + len] {
            *slot = (i << 8) | bu;
            i += 1;
        }
        pos += len as u32;
    }
    out.clear();
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
fn invert_tiny_fl_512(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    let mut buf = [0u32; 512];
    invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
}

#[inline(never)]
fn invert_tiny_fl_1536(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    let mut buf = [0u32; 1536];
    invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
}

// ============================================================================
// AblationKernel (`A00`–`A15`)
// ============================================================================
struct Pass<'a> {
    queues: outq::Queues<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    f: First,
}

impl Pass<'_> {
    #[inline(always)]
    fn take<const LIM: u32>(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        if width <= LIM {
            self.piece::<LIM>(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece::<LIM>(c, pos, width, false, p);
        } else {
            self.take_marked::<LIM>(c, pos, width, p);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked<const LIM: u32>(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        let end = pos + width;
        if pos < p {
            self.piece::<LIM>(c, pos, p - pos, false, p);
        }
        self.piece::<LIM>(c, p, 1, true, p);
        if p + 1 < end {
            self.piece::<LIM>(c, p + 1, end - p - 1, false, p);
        }
    }

    #[inline(always)]
    fn piece<const LIM: u32>(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
                f.mark_byte = c;
            }
            self.queues.push_thin::<LIM>(a, f.m, w);
            f.m += w;
        } else {
            let at = 3 * self.ns;
            if self.edges.len() < at + 3 {
                let need = (at + 3).div_ceil(4096) * 4096;
                if need > self.edges.capacity() {
                    self.edges.reserve_exact(need - self.edges.len());
                }
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
    contents_u32_fb: Vec<u32>,
    predecessor: Option<u32>,
    force_sentinel_cap: Option<u32>,
    force_u16_cap: Option<u32>,
}

impl AblationKernel {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn ws_snap(&self, rules_used_b: usize, est_gt_m: usize, s2b_grows: usize) -> WsSnap {
        let edges_cap_b = self.edges.capacity() * 4;
        let extra_cap_b = self.extra.capacity() * 4;
        let bands_cap_b = self.bands.capacity() * 8;
        let spelling_cap_b = self.spelling.capacity();
        let inline_cap_b = self.inline16.capacity() * INLINE + self.lens.capacity();
        let stack_cap_b = self.stack.capacity() * 4;
        let fb_cap_b = self.contents_u32_fb.capacity() * 4;
        let spill_cap_b = self.spill.cap_bytes();

        let total_cap_b = edges_cap_b
            + extra_cap_b
            + bands_cap_b
            + spelling_cap_b
            + inline_cap_b
            + stack_cap_b
            + fb_cap_b
            + spill_cap_b;

        let total_used_b = self.edges.len() * 4
            + self.extra.len() * 4
            + self.bands.len() * 8
            + self.spelling.len()
            + self.inline16.len() * INLINE
            + self.lens.len()
            + self.stack.len() * 4
            + self.contents_u32_fb.len() * 4
            + self.spill.used_bytes();

        WsSnap {
            edges_cap_b,
            rules_used_b,
            spelling_cap_b,
            spelling_used_b: self.spelling.len(),
            inline_cap_b,
            total_cap_b,
            total_used_b,
            est_gt_m,
            s2b_grows,
        }
    }

    #[inline(never)]
    fn invert_tiny_fl(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
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
        if n <= 64 {
            return invert_tiny_fl_64(records, n, p, out);
        }
        if n <= 256 {
            return invert_tiny_fl_256(records, n, p, out);
        }
        if n <= 512 {
            return invert_tiny_fl_512(records, n, p, out);
        }
        if n <= 1536 {
            return invert_tiny_fl_1536(records, n, p, out);
        }
        if self.edges.len() < n {
            let add = n - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(n, 0);
        }
        invert_tiny_fl_with_buf(records, n, p, &mut self.edges[..n], out)
    }

    #[inline(never)]
    fn stage1_scan<const HOIST: bool, const LIM: u32>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut [u8],
    ) -> (usize, First, outq::Heads, bool, u32, u32) {
        let nrec = records.len();
        if HOIST {
            let mut pass = Pass {
                queues: outq::Queues::new(out, &mut self.spill, n as u32),
                edges: &mut self.edges,
                ns: 0,
                f: First::default(),
            };
            let mut m = 0u32;
            let mut pos = 0u32;
            let mut extra_127 = 0u32;
            let mut i = 0usize;
            while i < nrec {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                if width <= LIM {
                    if p.wrapping_sub(pos) < width {
                        pass.f.mark = m + (p - pos);
                        pass.f.mark_byte = c;
                    }
                    pass.queues.push_thin::<LIM>(usize::from(c), m, width);
                    m += width;
                    pos += width;
                } else {
                    if width == 127 {
                        while i < nrec && (records[i] >> 8) as u8 == c {
                            width += u32::from(records[i] & 255);
                            extra_127 += 1;
                            i += 1;
                        }
                    }
                    if p.wrapping_sub(pos) >= width {
                        let a = usize::from(c);
                        let at = 3 * pass.ns;
                        if pass.edges.len() < at + 3 {
                            grow_edges_s1_step(pass.edges, at + 3);
                        }
                        let thin_a = pass.queues.thin_of(a);
                        pass.edges[at..at + 3].copy_from_slice(&[
                            pos,
                            thin_a + pass.f.thick_rows[a],
                            (width << 8) | u32::from(c),
                        ]);
                        pass.ns += 1;
                        pass.f.thick[a] += 1;
                        pass.f.thick_rows[a] += width;
                    } else {
                        pass.f.m = m;
                        pass.take_marked::<LIM>(c, pos, width, p);
                        m = pass.f.m;
                    }
                    pos += width;
                }
            }
            let over = pass.queues.over();
            let thin = pass.queues.extract_thin();
            let heads = pass.queues.heads();
            let mut f = pass.f;
            f.m = m;
            f.thin = thin;
            return (pass.ns, f, heads, over, extra_127, pos);
        }

        let mut pass = Pass {
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
                pass.piece::<LIM>(c, pos, width, true, p);
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
    fn stage2a_build<L: Links>(
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
            for (d, &s) in strips_ot.iter_mut().zip(&self.edges[..3 * ns]) {
                *d = s.to_ne_bytes();
            }
            if L::IS_NARROW {
                let (nodes_sl, _) = self.edges.split_at_mut(nodes_words);
                let (nodes, _) = nodes_sl.as_chunks_mut::<4>();
                nodes[0] = [0, 0, 0, 0];
                let mut nn = ns as u32 + 1;
                let (mut t0h, mut t0t) = (0u32, 0u32);
                let (mut t1h, mut t1t) = (0u32, 0u32);
                let (mut o0h, mut o0t) = (0u32, 0u32);
                let (mut o1h, mut o1t) = (0u32, 0u32);
                let (strips3, _) = strips_ot.as_chunks_mut::<3>();
                let mut x = 0u32;
                for (i, sw) in strips3.iter_mut().enumerate() {
                    let from = u32::from_ne_bytes(sw[0]);
                    let to0 = u32::from_ne_bytes(sw[1]);
                    let wbyte = u32::from_ne_bytes(sw[2]);
                    let byte = (wbyte & 0xFF) as u8;
                    let width = wbyte >> 8;
                    let a = usize::from(byte);
                    sw[1] = (to0 + first[a]).to_ne_bytes();
                    let dst_idx = write[a] as usize;
                    image_ot[dst_idx] = (i as u32).to_ne_bytes();
                    write[a] = dst_idx as u32 + 1;
                    let sid = i as u32 + 1;
                    widest = widest.max(width);
                    if x < from {
                        let gw = from - x;
                        widest = widest.max(gw);
                        let gid = nn;
                        nn += 1;
                        if o0t == 0 {
                            o0h = gid;
                        } else {
                            nodes[o0t as usize][0] =
                                (nodes[o0t as usize][0] & 0xFFFF) | (gid << 16);
                        }
                        if t1t == 0 {
                            t1h = gid;
                        } else {
                            nodes[t1t as usize][1] =
                                (nodes[t1t as usize][1] & 0xFFFF) | (gid << 16);
                        }
                        nodes[gid as usize] = [o0t, t1t, gw, 0];
                        o0t = gid;
                        t1t = gid;
                    }
                    if o0t == 0 {
                        o0h = sid;
                    } else {
                        nodes[o0t as usize][0] = (nodes[o0t as usize][0] & 0xFFFF) | (sid << 16);
                    }
                    nodes[sid as usize] = [o0t, 0, width, literal(byte)];
                    o0t = sid;
                    x = from + width;
                }
                if x < n {
                    let gw = n - x;
                    widest = widest.max(gw);
                    let gid = nn;
                    nn += 1;
                    if o0t == 0 {
                        o0h = gid;
                    } else {
                        nodes[o0t as usize][0] = (nodes[o0t as usize][0] & 0xFFFF) | (gid << 16);
                    }
                    if t1t == 0 {
                        t1h = gid;
                    } else {
                        nodes[t1t as usize][1] = (nodes[t1t as usize][1] & 0xFFFF) | (gid << 16);
                    }
                    nodes[gid as usize] = [o0t, t1t, gw, 0];
                    o0t = gid;
                    t1t = gid;
                }
                x = 0;
                for slot in image_ot.iter() {
                    let i = u32::from_ne_bytes(*slot);
                    let sw = &strips3[i as usize];
                    let to = u32::from_ne_bytes(sw[1]);
                    let width = u32::from_ne_bytes(sw[2]) >> 8;
                    if x < to {
                        let gw = to - x;
                        widest = widest.max(gw);
                        let gid = nn;
                        nn += 1;
                        if o1t == 0 {
                            o1h = gid;
                        } else {
                            nodes[o1t as usize][1] =
                                (nodes[o1t as usize][1] & 0xFFFF) | (gid << 16);
                        }
                        if t0t == 0 {
                            t0h = gid;
                        } else {
                            nodes[t0t as usize][0] =
                                (nodes[t0t as usize][0] & 0xFFFF) | (gid << 16);
                        }
                        nodes[gid as usize] = [t0t, o1t, gw, 0];
                        o1t = gid;
                        t0t = gid;
                    }
                    let sid = i + 1;
                    if o1t == 0 {
                        o1h = sid;
                    } else {
                        nodes[o1t as usize][1] = (nodes[o1t as usize][1] & 0xFFFF) | (sid << 16);
                    }
                    nodes[sid as usize][1] = o1t;
                    o1t = sid;
                    x = to + width;
                }
                if x < n {
                    let gw = n - x;
                    widest = widest.max(gw);
                    let gid = nn;
                    nn += 1;
                    if o1t == 0 {
                        o1h = gid;
                    } else {
                        nodes[o1t as usize][1] = (nodes[o1t as usize][1] & 0xFFFF) | (gid << 16);
                    }
                    if t0t == 0 {
                        t0h = gid;
                    } else {
                        nodes[t0t as usize][0] = (nodes[t0t as usize][0] & 0xFFFF) | (gid << 16);
                    }
                    nodes[gid as usize] = [t0t, o1t, gw, 0];
                    o1t = gid;
                    t0t = gid;
                }
                let mut close_side = |side: usize, first_p: (u32, u32), second_p: (u32, u32)| {
                    let mut prev = 0u32;
                    for (head, tail) in [first_p, second_p] {
                        if tail == 0 {
                            continue;
                        }
                        nodes[prev as usize][side] =
                            (nodes[prev as usize][side] & 0xFFFF) | (head << 16);
                        nodes[head as usize][side] =
                            (nodes[head as usize][side] & 0xFFFF_0000) | prev;
                        prev = tail;
                    }
                    nodes[prev as usize][side] &= 0xFFFF;
                    nodes[0][side] = (nodes[0][side] & 0xFFFF_0000) | prev;
                };
                close_side(0, (t0h, t0t), (o0h, o0t));
                close_side(1, (t1h, t1t), (o1h, o1t));
                let open_nodes = nn as usize - 1;
                return (4 * nn as usize, nn, open_nodes, widest);
            }
            let (nodes_sl, rules_dummy) = self.edges.split_at_mut(nodes_words);
            let mut e = Elim::<L, false> {
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
                let sw = &mut strips_ot[3 * i..3 * i + 3];
                let from = u32::from_ne_bytes(sw[0]);
                let to0 = u32::from_ne_bytes(sw[1]);
                let wbyte = u32::from_ne_bytes(sw[2]);
                let byte = (wbyte & 0xFF) as u8;
                let width = wbyte >> 8;
                let a = usize::from(byte);
                sw[1] = (to0 + first[a]).to_ne_bytes();
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
                let sw = &strips_ot[3 * idx..3 * idx + 3];
                let to = u32::from_ne_bytes(sw[1]);
                let width = u32::from_ne_bytes(sw[2]) >> 8;
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
            return (base, e.nodes, open_nodes, widest);
        }
        let (hdr, arena) = self.edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        let (nodes_sl, rules_dummy) = arena.split_at_mut(nodes_words);
        let mut e = Elim::<L, false> {
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

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn stage3a_bands<L: Links, const U16_DICT: bool>(
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
        if !U16_DICT
            && use_zero_hdr
            && ns >= 64
            && self.edges.len() < base + rule_top + CACHE_WORDS
        {
            let need_c = base + rule_top + CACHE_WORDS;
            self.edges.reserve_exact(need_c - self.edges.len());
            self.edges.resize(need_c, 0);
        }

        let (header, arena) = self.edges.split_at_mut(hdr_words);
        let (live, tail_arena) = arena.split_at_mut(base + rule_top);
        let (nodes_sl, rules_sl) = live.split_at_mut(base);
        let mut e = Elim::<L, false> {
            nodes_buf: nodes_sl,
            rules_buf: rules_sl,
            rule_top,
            nodes,
            id_overflow: false,
            _links: std::marker::PhantomData,
        };
        e.assign_bottom(m);

        begin_inline(&mut self.inline16, &mut self.lens);

        let max_bands = open_nodes + 256;
        let use_tail = out_tail.len() >= max_bands * 8;
        let mut cursors = [0usize; 256];
        let mut nbands = 0usize;
        let mut istk = [0u32; INLINE_STACK];
        let mut memo_rec = [0u32; MEMO_REC_CAP];

        if U16_DICT {
            let mut pairs = [0u32; 512];
            let mut contents_u16 = [0u16; 2048];
            let u16_cap = self
                .force_u16_cap
                .unwrap_or(U16_DICT_CAP)
                .min(U16_DICT_CAP);
            let mut dictionary = DictionaryU16 {
                pairs: &mut pairs,
                contents_u16: &mut contents_u16,
                contents_u32_fb: &mut self.contents_u32_fb,
                u16_cap,
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
                    let ta = thin[a];
                    if ta == 0 {
                        continue;
                    }
                    let end = k + ta;
                    let mut last_w = 0u32;
                    let mut last_c = 0u32;
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        debug_assert!(width > 0);
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
                    let ta = thin[a];
                    if ta == 0 {
                        continue;
                    }
                    let end = k + ta;
                    let mut last_w = 0u32;
                    let mut last_c = 0u32;
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        debug_assert!(width > 0);
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
            return (cursors, use_tail, nbands);
        }

        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail_arena
        };

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
                        ensure_step_inline(&mut self.inline16, &mut self.lens, INLINE_STEP);
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
            let mut dictionary = DictionaryU32 {
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
                    let ta = thin[a];
                    if ta == 0 {
                        continue;
                    }
                    let end = k + ta;
                    let mut last_w = 0u32;
                    let mut last_c = 0u32;
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        debug_assert!(width > 0);
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
                    let ta = thin[a];
                    if ta == 0 {
                        continue;
                    }
                    let end = k + ta;
                    let mut last_w = 0u32;
                    let mut last_c = 0u32;
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        debug_assert!(width > 0);
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

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_bi_tail<const S4_MODE: u8>(
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
        let (band_slots, _) = out_tail[..nbands * 8].as_chunks::<8>();
        let mut predecessor = 0u32;
        let mb = usize::from(mark_byte);
        for a in 0..256 {
            let start = cursors[a];
            let end = cursors.get(a + 1).copied().unwrap_or(nbands);
            if start == end {
                continue;
            }
            let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
            if a != mb {
                for &slot in &band_slots[start..end] {
                    let b = Band(u64::from_le_bytes(slot));
                    reader.wire_xor_nomark::<S4_MODE>(
                        edges,
                        b.dst() as usize,
                        b.end() as usize,
                        b.code() << bits,
                    );
                }
            } else {
                for &slot in &band_slots[start..end] {
                    let b = Band(u64::from_le_bytes(slot));
                    reader.wire_xor(
                        edges,
                        b.dst() as usize,
                        b.end() as usize,
                        b.code() << bits,
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
    fn wire_bi_vec<const S4_MODE: u8>(
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
        let mut predecessor = 0u32;
        let mb = usize::from(mark_byte);
        for a in 0..256 {
            let start = cursors[a];
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            if start == end {
                continue;
            }
            let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
            if a != mb {
                for &b in &self.bands[start..end] {
                    reader.wire_xor_nomark::<S4_MODE>(
                        edges,
                        b.dst() as usize,
                        b.end() as usize,
                        b.code() << bits,
                    );
                }
            } else {
                for &b in &self.bands[start..end] {
                    reader.wire_xor(
                        edges,
                        b.dst() as usize,
                        b.end() as usize,
                        b.code() << bits,
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
    fn wire_bi_sentinel<const S4_MODE: u8>(
        &mut self,
        m: usize,
        mark: u32,
        mark_byte: u8,
        bits: u32,
        sentinel: u32,
        cursors: &[usize; 256],
        nbands: usize,
        used_tail: bool,
        out_tail: &[u8],
        out_q: &[u8],
        heads: &outq::Heads,
    ) -> u32 {
        let edges = &mut self.edges[..m];
        edges.fill(0);
        let mut predecessor = 0u32;
        let mb = usize::from(mark_byte);
        let sent_pack = sentinel << bits;
        if used_tail {
            let (band_slots, _) = out_tail[..nbands * 8].as_chunks::<8>();
            for a in 0..256 {
                let start = cursors[a];
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                if start == end {
                    continue;
                }
                let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                for &slot in &band_slots[start..end] {
                    let b = Band(u64::from_le_bytes(slot));
                    let dst = b.dst() as usize;
                    let span = b.end() as usize;
                    let bcode = b.code();
                    let pack = if bcode < sentinel {
                        bcode << bits
                    } else {
                        self.extra[dst..dst + span].fill(bcode);
                        sent_pack
                    };
                    if a != mb {
                        reader.wire_xor_nomark::<S4_MODE>(edges, dst, span, pack);
                    } else {
                        reader.wire_xor(edges, dst, span, pack, mark, &mut predecessor);
                    }
                }
            }
        } else {
            for a in 0..256 {
                let start = cursors[a];
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                if start == end {
                    continue;
                }
                let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                for &b in &self.bands[start..end] {
                    let dst = b.dst() as usize;
                    let span = b.end() as usize;
                    let bcode = b.code();
                    let pack = if bcode < sentinel {
                        bcode << bits
                    } else {
                        self.extra[dst..dst + span].fill(bcode);
                        sent_pack
                    };
                    if a != mb {
                        reader.wire_xor_nomark::<S4_MODE>(edges, dst, span, pack);
                    } else {
                        reader.wire_xor(edges, dst, span, pack, mark, &mut predecessor);
                    }
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
        sentinel: u32,
        cursors: &[usize; 256],
        nbands: usize,
        out_tail: &[u8],
        out_q: &[u8],
        heads: &outq::Heads,
    ) {
        let edges = &mut self.edges[..m];
        let (band_slots, _) = out_tail[..nbands * 8].as_chunks::<8>();
        for a in 0..256 {
            let start = cursors[a];
            let end = cursors.get(a + 1).copied().unwrap_or(nbands);
            if start == end {
                continue;
            }
            let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
            for &slot in &band_slots[start..end] {
                let b = Band(u64::from_le_bytes(slot));
                let dst = b.dst() as usize;
                let span = b.end() as usize;
                let bcode = b.code();
                let pack = if !wide || bcode < sentinel {
                    bcode << bits
                } else {
                    self.extra[dst..dst + span].fill(bcode);
                    sentinel << bits
                };
                reader.wire_uni(edges, dst, span, pack);
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
        sentinel: u32,
        cursors: &[usize; 256],
        out_q: &[u8],
        heads: &outq::Heads,
    ) {
        let edges = &mut self.edges[..m];
        for a in 0..256 {
            let start = cursors[a];
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            if start == end {
                continue;
            }
            let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
            for &b in &self.bands[start..end] {
                let dst = b.dst() as usize;
                let span = b.end() as usize;
                let bcode = b.code();
                let pack = if !wide || bcode < sentinel {
                    bcode << bits
                } else {
                    self.extra[dst..dst + span].fill(bcode);
                    sentinel << bits
                };
                reader.wire_uni(edges, dst, span, pack);
            }
        }
    }

    fn run_cfg<const CFG: u32, const TIME: bool, const FORCE_OPEN: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> (bool, StageTimes, WsSnap) {
        let t_all = if TIME { Some(Instant::now()) } else { None };
        let mut tm = StageTimes::default();
        if n == 0 || n > 900_000 || (p as usize) >= n || records.len() > 16_000_000 {
            return (false, tm, WsSnap::default());
        }
        if !FORCE_OPEN && n < 32_768 {
            let ok = self.invert_tiny_fl(records, n, p, out);
            if let Some(t0) = t_all {
                let us = t0.elapsed().as_secs_f64() * 1e6;
                tm.tiny = us;
                tm.total = us;
            }
            return (ok, tm, self.ws_snap(0, 0, 0));
        }
        if !FORCE_OPEN && records.len() * 64 < n && try_short_periodic(records, n, p, out) {
            if let Some(t0) = t_all {
                tm.total = t0.elapsed().as_secs_f64() * 1e6;
            }
            return (true, tm, self.ws_snap(0, 0, 0));
        }
        let lim = if FORCE_OPEN {
            16
        } else {
            select_limit(records, n)
        };
        let (ok, mut st, snap) = match lim {
            16 => self.try_run_lim::<CFG, 16, TIME>(records, n, p, out),
            24 => self.try_run_lim::<CFG, 24, TIME>(records, n, p, out),
            _ => self.try_run_lim::<CFG, 32, TIME>(records, n, p, out),
        };
        if let Some(t0) = t_all {
            st.total = t0.elapsed().as_secs_f64() * 1e6;
        }
        (ok, st, snap)
    }

    #[inline(never)]
    fn try_run_lim<const CFG: u32, const LIM: u32, const TIME: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> (bool, StageTimes, WsSnap) {
        let mut tm = StageTimes::default();
        let t_s1 = if TIME { Some(Instant::now()) } else { None };
        out.resize(n + INLINE, 0);
        let (ns, f, heads, over, extra_127, tot_pos) = if has(CFG, BIT_S1_LOCAL_M) {
            self.stage1_scan::<true, LIM>(records, n, p, &mut out[..])
        } else {
            self.stage1_scan::<false, LIM>(records, n, p, &mut out[..])
        };
        if let Some(t0) = t_s1 {
            tm.s1 = t0.elapsed().as_secs_f64() * 1e6;
        }
        if over || tot_pos as usize != n || f.m == 0 {
            return (false, tm, self.ws_snap(0, 0, 0));
        }
        let r_max = (records.len() as u32).saturating_sub(extra_127);
        self.finish::<CFG, TIME>(&f, ns, n, r_max, &heads, out, &mut tm)
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn finish<const CFG: u32, const TIME: bool>(
        &mut self,
        f: &First,
        ns: usize,
        n: usize,
        r_max: u32,
        heads: &outq::Heads,
        out: &mut Vec<u8>,
        tm: &mut StageTimes,
    ) -> (bool, StageTimes, WsSnap) {
        let t_s2a = if TIME { Some(Instant::now()) } else { None };
        let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        if all.iter().sum::<u32>() != n as u32 {
            return (false, *tm, self.ws_snap(0, 0, 0));
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
        self.contents_u32_fb.clear();

        let in_out_chunks = out.len() / outq::BYTES;
        let tail_offset = (heads.chunks as usize).min(in_out_chunks) * outq::BYTES;
        let use_zero_hdr = out.len() - tail_offset >= 16 * ns;
        let hdr_words = if use_zero_hdr { 0 } else { 4 * ns };

        let need = (hdr_words + stride * most).max(m);
        if self.edges.len() < need {
            let add = need - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(need, 0);
        }

        let (base, nodes, open_nodes, widest) = if use_narrow {
            self.stage2a_build::<Narrow>(
                ns,
                n as u32,
                &first,
                &mut write,
                most,
                use_zero_hdr,
                &mut out[tail_offset..],
            )
        } else {
            self.stage2a_build::<Wide>(
                ns,
                n as u32,
                &first,
                &mut write,
                most,
                use_zero_hdr,
                &mut out[tail_offset..],
            )
        };
        if let Some(t0) = t_s2a {
            tm.s2a = t0.elapsed().as_secs_f64() * 1e6;
        }

        let t_s2b = if TIME { Some(Instant::now()) } else { None };
        let spirals = widest > CERTIFIED_WIDTH;
        let mut remaining = n as u32;
        let mut rule_top = 0usize;
        let mut s2b_grows = 0usize;
        loop {
            let (nodes_sl, rules_sl) = self.edges[hdr_words..].split_at_mut(base);
            let (done, id_ovf, new_top) = if use_narrow {
                if has(CFG, BIT_S2B_DBG_ASSERT) {
                    let mut e = Elim::<Narrow, true> {
                        nodes_buf: nodes_sl,
                        rules_buf: rules_sl,
                        rule_top,
                        nodes,
                        id_overflow: false,
                        _links: std::marker::PhantomData,
                    };
                    let d = if spirals {
                        e.eliminate::<true>(&mut remaining)
                    } else {
                        e.eliminate::<false>(&mut remaining)
                    };
                    (d, e.id_overflow, e.rule_top)
                } else {
                    let mut e = Elim::<Narrow, false> {
                        nodes_buf: nodes_sl,
                        rules_buf: rules_sl,
                        rule_top,
                        nodes,
                        id_overflow: false,
                        _links: std::marker::PhantomData,
                    };
                    let d = if spirals {
                        e.eliminate::<true>(&mut remaining)
                    } else {
                        e.eliminate::<false>(&mut remaining)
                    };
                    (d, e.id_overflow, e.rule_top)
                }
            } else {
                let mut e = Elim::<Wide, false> {
                    nodes_buf: nodes_sl,
                    rules_buf: rules_sl,
                    rule_top,
                    nodes,
                    id_overflow: false,
                    _links: std::marker::PhantomData,
                };
                let d = if spirals {
                    e.eliminate::<true>(&mut remaining)
                } else {
                    e.eliminate::<false>(&mut remaining)
                };
                (d, e.id_overflow, e.rule_top)
            };
            rule_top = new_top;
            if id_ovf {
                return (false, *tm, self.ws_snap(rule_top * 4, 0, s2b_grows));
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
        if let Some(t0) = t_s2b {
            tm.s2b = t0.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIME { Some(Instant::now()) } else { None };
        let (cursors, used_tail, nbands) = if use_narrow {
            if has(CFG, BIT_U16_DICT) {
                self.stage3a_bands::<Narrow, true>(
                    ns,
                    f.m,
                    &f.thin,
                    hdr_words,
                    base,
                    rule_top,
                    nodes,
                    open_nodes,
                    use_zero_hdr,
                    &mut out[tail_offset..],
                )
            } else {
                self.stage3a_bands::<Narrow, false>(
                    ns,
                    f.m,
                    &f.thin,
                    hdr_words,
                    base,
                    rule_top,
                    nodes,
                    open_nodes,
                    use_zero_hdr,
                    &mut out[tail_offset..],
                )
            }
        } else if has(CFG, BIT_U16_DICT) {
            self.stage3a_bands::<Wide, true>(
                ns,
                f.m,
                &f.thin,
                hdr_words,
                base,
                rule_top,
                nodes,
                open_nodes,
                use_zero_hdr,
                &mut out[tail_offset..],
            )
        } else {
            self.stage3a_bands::<Wide, false>(
                ns,
                f.m,
                &f.thin,
                hdr_words,
                base,
                rule_top,
                nodes,
                open_nodes,
                use_zero_hdr,
                &mut out[tail_offset..],
            )
        };
        if let Some(t0) = t_s3a {
            tm.s3a = t0.elapsed().as_secs_f64() * 1e6;
        }

        let t_s4 = if TIME { Some(Instant::now()) } else { None };
        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        let sentinel = self
            .force_sentinel_cap
            .unwrap_or(u32::MAX >> bits)
            .min(u32::MAX >> bits);
        let wide = self.lens.len() - 1 > sentinel as usize;
        if self.edges.len() < m {
            self.edges.resize(m, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m, 0);
        }
        self.predecessor = None;
        let use_xor = should_use_bi(f.m, ns as u32, r_max);
        let (out_q, out_tail) = out.split_at(tail_offset);

        if use_xor {
            let pred = if !wide {
                if used_tail {
                    if has(CFG, BIT_S4_8WAY_EPOCH) {
                        self.wire_bi_tail::<3>(
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
                    } else if has(CFG, BIT_S4_SPLIT_EPOCH) {
                        self.wire_bi_tail::<1>(
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
                    } else if has(CFG, BIT_S4_SLIM16) {
                        self.wire_bi_tail::<2>(
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
                        self.wire_bi_tail::<0>(
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
                } else if has(CFG, BIT_S4_8WAY_EPOCH) {
                    self.wire_bi_vec::<3>(m, f.mark, f.mark_byte, bits, &cursors, out_q, heads)
                } else if has(CFG, BIT_S4_SPLIT_EPOCH) {
                    self.wire_bi_vec::<1>(m, f.mark, f.mark_byte, bits, &cursors, out_q, heads)
                } else {
                    self.wire_bi_vec::<0>(m, f.mark, f.mark_byte, bits, &cursors, out_q, heads)
                }
            } else if has(CFG, BIT_S4_8WAY_EPOCH) {
                self.wire_bi_sentinel::<3>(
                    m,
                    f.mark,
                    f.mark_byte,
                    bits,
                    sentinel,
                    &cursors,
                    nbands,
                    used_tail,
                    out_tail,
                    out_q,
                    heads,
                )
            } else if has(CFG, BIT_S4_SPLIT_EPOCH) {
                self.wire_bi_sentinel::<1>(
                    m,
                    f.mark,
                    f.mark_byte,
                    bits,
                    sentinel,
                    &cursors,
                    nbands,
                    used_tail,
                    out_tail,
                    out_q,
                    heads,
                )
            } else {
                self.wire_bi_sentinel::<0>(
                    m,
                    f.mark,
                    f.mark_byte,
                    bits,
                    sentinel,
                    &cursors,
                    nbands,
                    used_tail,
                    out_tail,
                    out_q,
                    heads,
                )
            };
            self.predecessor = Some(pred);
        } else if used_tail {
            self.wire_uni_tail(
                m, wide, bits, sentinel, &cursors, nbands, out_tail, out_q, heads,
            );
        } else {
            self.wire_uni_vec(m, wide, bits, sentinel, &cursors, out_q, heads);
        }
        if let Some(t0) = t_s4 {
            tm.s4 = t0.elapsed().as_secs_f64() * 1e6;
        }

        let est_gt_m = usize::from(self.edges.capacity() > m);
        let snap = self.ws_snap(rule_top * 4, est_gt_m, s2b_grows);

        let t_s5 = if TIME { Some(Instant::now()) } else { None };
        if let Some(before) = self.predecessor {
            if !wide {
                emit_bidirectional_cfg::<CFG>(
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
                emit_bidirectional_sentinel(
                    &self.edges[..m],
                    &self.extra,
                    &self.spelling,
                    &self.inline16[1..],
                    &self.lens,
                    n,
                    f.mark,
                    before,
                    bits,
                    sentinel,
                    out,
                );
            }
        } else {
            emit_uni_cfg::<CFG>(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16[1..],
                &self.lens,
                n,
                f.mark,
                bits,
                sentinel,
                out,
            );
        }
        if let Some(t0) = t_s5 {
            tm.s5 = t0.elapsed().as_secs_f64() * 1e6;
        }
        (true, *tm, snap)
    }
}

// ============================================================================
// Stage 5 Emit Variants (`A00` baseline vs `ZeroSpill`, `Cold-Pipe1+In`,
// `Pipe2`, `BackU64`, `LitFast`)
// ============================================================================
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
fn emit_uni_cfg<const CFG: u32>(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    sentinel: u32,
    out: &mut Vec<u8>,
) {
    debug_assert!(out.len() >= n + INLINE);
    let mask = (1 << bits) - 1;
    let mut j = mark;
    if extra.is_empty() {
        let len_c = lens.len().min(inline16.len());
        let lens = &lens[..len_c];
        let inline16 = &inline16[..len_c];
        let mut rem: &mut [u8] = &mut out[..n];
        let mut done = false;
        if rem.len() >= INLINE {
            let mut edge = edges[j as usize];
            while rem.len() >= INLINE {
                let code = (edge >> bits) as usize;
                j = edge & mask;
                edge = edges[j as usize];
                if has(CFG, BIT_S5_LIT_FAST) && code < 256 {
                    rem[0] = code as u8;
                    rem = &mut rem[1..];
                } else {
                    let len = lens[code];
                    if len <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&inline16[code]);
                        rem = &mut rem[usize::from(len)..];
                    } else {
                        let take = emit_long_front(&inline16[code], spelling, rem);
                        rem = &mut rem[take..];
                    }
                }
                if j == mark {
                    done = true;
                    break;
                }
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
    let out_buf = &mut out[..n + INLINE];
    let mut at = 0;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let raw_code = edge >> bits;
        let code = if raw_code == sentinel {
            extra[j as usize]
        } else {
            raw_code
        } as usize;
        j = edge & mask;
        let len = lens[code];
        if len <= INLINE as u8 {
            out_buf[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            let take = word.len().min(n - at);
            out_buf[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[inline(always)]
fn emit_short_back<const CFG: u32>(
    sb: &[u8; INLINE],
    lb: u8,
    spelling: &[u8],
    rem: &mut &mut [u8],
) {
    let rlen = rem.len();
    if has(CFG, BIT_S5_BACK_U64) {
        if lb <= 8 {
            let lbu = usize::from(lb);
            let sh = ((8 - lbu) << 3) & 63;
            let v64 = u64::from_le_bytes(sb[..8].try_into().unwrap()) << sh;
            rem[rlen - 8..].copy_from_slice(&v64.to_le_bytes());
            let cur = std::mem::take(rem);
            *rem = &mut cur[..rlen - lbu];
        } else if lb <= INLINE as u8 {
            let lbu = usize::from(lb);
            let sh = ((INLINE - lbu) << 3) & 63;
            let shifted = u128::from_le_bytes(*sb) << sh;
            rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
            let cur = std::mem::take(rem);
            *rem = &mut cur[..rlen - lbu];
        } else {
            let take = emit_long_back(sb, spelling, rem);
            let cur = std::mem::take(rem);
            *rem = &mut cur[..rlen - take];
        }
    } else if lb <= INLINE as u8 {
        let lbu = usize::from(lb);
        let shifted = u128::from_le_bytes(*sb) << (8 * (INLINE - lbu));
        rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
        let cur = std::mem::take(rem);
        *rem = &mut cur[..rlen - lbu];
    } else {
        let take = emit_long_back(sb, spelling, rem);
        let cur = std::mem::take(rem);
        *rem = &mut cur[..rlen - take];
    }
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_bi_c53_baseline(
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
    let out_base = out.as_ptr() as usize;
    let mut rem: &mut [u8] = &mut out[..n];
    let len_c = lens.len().min(inline.len());
    let lens = &lens[..len_c];
    let inline = &inline[..len_c];
    let mut done = false;
    if rem.len() >= 2 * INLINE {
        let mut ef = edges[front as usize];
        let mut eb = edges[back as usize];
        while rem.len() >= 2 * INLINE {
            if front == back {
                let cf = (ef >> bits) as usize;
                let lf = lens[cf];
                if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&inline[cf]);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline[cf], spelling, rem);
                    rem = &mut rem[take..];
                }
                done = true;
                break;
            }
            let f_next = (ef & mask) ^ prev;
            let b_next = (eb & mask) ^ next;
            let ef_nxt = edges[f_next as usize];
            let eb_nxt = edges[b_next as usize];

            let cf = (ef >> bits) as usize;
            let lf = lens[cf];
            if lf <= INLINE as u8 {
                rem[..INLINE].copy_from_slice(&inline[cf]);
                rem = &mut rem[usize::from(lf)..];
            } else {
                let take = emit_long_front(&inline[cf], spelling, rem);
                rem = &mut rem[take..];
                if rem.is_empty() {
                    done = true;
                    break;
                }
                if rem.len() < INLINE {
                    let cb = (eb >> bits) as usize;
                    let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                    let rlen = rem.len();
                    rem = &mut rem[..rlen - take_b];
                    prev = front;
                    front = f_next;
                    next = back;
                    back = b_next;
                    if front == next {
                        done = true;
                    }
                    break;
                }
            }
            let cb = (eb >> bits) as usize;
            let rlen = rem.len();
            let lb = lens[cb];
            if lb <= INLINE as u8 {
                let lbu = usize::from(lb);
                let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - lbu));
                rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                rem = &mut rem[..rlen - lbu];
            } else {
                let take = emit_long_back(&inline[cb], spelling, rem);
                rem = &mut rem[..rlen - take];
            }
            prev = front;
            front = f_next;
            ef = ef_nxt;
            next = back;
            back = b_next;
            eb = eb_nxt;
            if front == next {
                done = true;
                break;
            }
        }
    }
    let (lo_rem, rem_len) = emit_bi_tail(
        edges, spelling, inline, lens, bits, mask, out_base, rem, done, prev, front, back, next,
    );
    finalize_bi_out(lo_rem, rem_len, n, out);
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn emit_bi_tail(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    bits: u32,
    mask: u32,
    out_base: usize,
    mut rem: &mut [u8],
    done: bool,
    mut prev: u32,
    mut front: u32,
    mut back: u32,
    mut next: u32,
) -> (usize, usize) {
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
    (lo_rem, rem_len)
}

#[inline(always)]
fn finalize_bi_out(lo_rem: usize, rem_len: usize, n: usize, out: &mut Vec<u8>) {
    let hi_rem = lo_rem + rem_len;
    if rem_len > 0 {
        out.copy_within(hi_rem..n, lo_rem);
        out.truncate(lo_rem + (n - hi_rem));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_bi_specialized<const CFG: u32, const BITS: u32>(
    edges: &[u32],
    spelling: &[u8],
    inline_padded: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    before: u32,
    bits_rt: u32,
    out: &mut Vec<u8>,
) {
    debug_assert!(out.len() >= n + INLINE);
    let shift = if BITS != 0 { BITS } else { bits_rt };
    let mask = (1u32 << shift) - 1;
    let out_base = out.as_ptr() as usize;
    let mut rem: &mut [u8] = &mut out[..n];
    let len_c = lens.len().min(inline_padded.len().saturating_sub(1));
    let lens = &lens[..len_c];
    let inline = &inline_padded[1..len_c + 1];
    let inline_flat: &[u8] = inline_padded[..len_c + 1].as_flattened();

    if has(CFG, BIT_S5_PIPE2) {
        let mut prev = before;
        let mut f0 = mark;
        let mut b0 = before;
        let mut next = mark;
        let mut done = false;
        if rem.len() >= 2 * INLINE {
            let mut ef0 = edges[f0 as usize];
            let mut eb0 = edges[b0 as usize];
            let mut f1 = (ef0 & mask) ^ prev;
            let mut b1 = (eb0 & mask) ^ next;
            let mut ef1 = edges[f1 as usize];
            let mut eb1 = edges[b1 as usize];
            while rem.len() >= 2 * INLINE {
                if f0 == b0 {
                    let cf = (ef0 >> shift) as usize;
                    let lf = lens[cf];
                    if lf <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&inline[cf]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline[cf], spelling, rem);
                        rem = &mut rem[take..];
                    }
                    done = true;
                    break;
                }
                let f2 = (ef1 & mask) ^ f0;
                let b2 = (eb1 & mask) ^ b0;
                let ef2 = edges[f2 as usize];
                let eb2 = edges[b2 as usize];
                let cf = (ef0 >> shift) as usize;
                let cb = (eb0 >> shift) as usize;
                let lf = lens[cf];
                if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&inline[cf]);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline[cf], spelling, rem);
                    rem = &mut rem[take..];
                    if rem.is_empty() {
                        done = true;
                        break;
                    }
                    if rem.len() < INLINE {
                        let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                        let rlen = rem.len();
                        rem = &mut rem[..rlen - take_b];
                        prev = f0;
                        f0 = f1;
                        next = b0;
                        b0 = b1;
                        if f0 == next {
                            done = true;
                        }
                        break;
                    }
                }
                let lb = lens[cb];
                if has(CFG, BIT_S5_UNAL_BACK) {
                    let rlen = rem.len();
                    if lb <= INLINE as u8 {
                        let lbu = usize::from(lb);
                        let off = (cb << 4) + lbu;
                        rem[rlen - INLINE..].copy_from_slice(&inline_flat[off..off + INLINE]);
                        rem = &mut rem[..rlen - lbu];
                    } else {
                        let take = emit_long_back(&inline[cb], spelling, rem);
                        rem = &mut rem[..rlen - take];
                    }
                } else {
                    emit_short_back::<CFG>(&inline[cb], lb, spelling, &mut rem);
                }
                prev = f0;
                f0 = f1;
                ef0 = ef1;
                f1 = f2;
                ef1 = ef2;
                next = b0;
                b0 = b1;
                eb0 = eb1;
                b1 = b2;
                eb1 = eb2;
                if f0 == next {
                    done = true;
                    break;
                }
            }
        }
        let (lo_rem, rem_len) = emit_bi_tail(
            edges, spelling, inline, lens, shift, mask, out_base, rem, done, prev, f0, b0, next,
        );
        finalize_bi_out(lo_rem, rem_len, n, out);
        return;
    }

    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let mut done = false;
    if rem.len() >= 2 * INLINE {
        let mut ef = edges[front as usize];
        let mut eb = edges[back as usize];
        while rem.len() >= 2 * INLINE {
            if front == back {
                let cf = (ef >> shift) as usize;
                let lf = lens[cf];
                if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&inline[cf]);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline[cf], spelling, rem);
                    rem = &mut rem[take..];
                }
                done = true;
                break;
            }
            let cf = (ef >> shift) as usize;
            let cb = (eb >> shift) as usize;
            let f_next = (ef & mask) ^ prev;
            let b_next = (eb & mask) ^ next;
            prev = front;
            front = f_next;
            next = back;
            back = b_next;
            // Zero-spill in-place overwrite of ef/eb BEFORE dictionary lookup & stores!
            ef = edges[front as usize];
            eb = edges[back as usize];

            if has(CFG, BIT_S5_UNAL_BACK) {
                if has(CFG, BIT_S5_COLD_PIPE1_IN) {
                    // CS-40 `Cold-Pipe1+In` + Zero-Shift `UnalignedBack` (single base pointer!):
                    let lf = lens[cf];
                    let lb = lens[cb];
                    if lf <= INLINE as u8 {
                        let off_f = (cf + 1) << 4;
                        rem[..INLINE].copy_from_slice(&inline_flat[off_f..off_f + INLINE]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline_padded[cf + 1], spelling, rem);
                        rem = &mut rem[take..];
                        if rem.is_empty() {
                            done = true;
                            break;
                        }
                        if rem.len() < INLINE {
                            let take_b = emit_tail_back(&inline_padded[cb + 1], lb, spelling, rem);
                            let rlen = rem.len();
                            rem = &mut rem[..rlen - take_b];
                            if front == next {
                                done = true;
                            }
                            break;
                        }
                    }
                    let rlen = rem.len();
                    if lb <= INLINE as u8 {
                        let lbu = usize::from(lb);
                        let off = (cb << 4) + lbu;
                        rem[rlen - INLINE..].copy_from_slice(&inline_flat[off..off + INLINE]);
                        rem = &mut rem[..rlen - lbu];
                    } else {
                        let take = emit_long_back(&inline_padded[cb + 1], spelling, rem);
                        rem = &mut rem[..rlen - take];
                    }
                } else {
                    // Single-base-pointer `UnalignedBack`: `inline_flat` serves BOTH front (`(cf+1)<<4`)
                    // AND back (`(cb<<4)+lbu`), freeing a GPR for `lens.as_ptr()`!
                    let lf = lens[cf];
                    if lf <= INLINE as u8 {
                        let off_f = (cf + 1) << 4;
                        rem[..INLINE].copy_from_slice(&inline_flat[off_f..off_f + INLINE]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline_padded[cf + 1], spelling, rem);
                        rem = &mut rem[take..];
                        if rem.is_empty() {
                            done = true;
                            break;
                        }
                        if rem.len() < INLINE {
                            let take_b =
                                emit_tail_back(&inline_padded[cb + 1], lens[cb], spelling, rem);
                            let rlen = rem.len();
                            rem = &mut rem[..rlen - take_b];
                            if front == next {
                                done = true;
                            }
                            break;
                        }
                    }
                    let lb = lens[cb];
                    let rlen = rem.len();
                    if lb <= INLINE as u8 {
                        let lbu = usize::from(lb);
                        let off = (cb << 4) + lbu;
                        rem[rlen - INLINE..].copy_from_slice(&inline_flat[off..off + INLINE]);
                        rem = &mut rem[..rlen - lbu];
                    } else {
                        let take = emit_long_back(&inline_padded[cb + 1], spelling, rem);
                        rem = &mut rem[..rlen - take];
                    }
                }
            } else if has(CFG, BIT_S5_COLD_PIPE1_IN) {
                let lf = lens[cf];
                let sf = inline[cf];
                let lb = lens[cb];
                let sb = inline[cb];
                if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&sf);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&sf, spelling, rem);
                    rem = &mut rem[take..];
                    if rem.is_empty() {
                        done = true;
                        break;
                    }
                    if rem.len() < INLINE {
                        let take_b = emit_tail_back(&sb, lb, spelling, rem);
                        let rlen = rem.len();
                        rem = &mut rem[..rlen - take_b];
                        if front == next {
                            done = true;
                        }
                        break;
                    }
                }
                emit_short_back::<CFG>(&sb, lb, spelling, &mut rem);
            } else {
                if has(CFG, BIT_S5_LIT_FAST) && cf < 256 {
                    rem[0] = cf as u8;
                    rem = &mut rem[1..];
                } else {
                    let lf = lens[cf];
                    if lf <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&inline[cf]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline[cf], spelling, rem);
                        rem = &mut rem[take..];
                        if rem.is_empty() {
                            done = true;
                            break;
                        }
                        if rem.len() < INLINE {
                            let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                            let rlen = rem.len();
                            rem = &mut rem[..rlen - take_b];
                            if front == next {
                                done = true;
                            }
                            break;
                        }
                    }
                }
                if has(CFG, BIT_S5_LIT_FAST) && cb < 256 {
                    let rlen = rem.len();
                    rem[rlen - 1] = cb as u8;
                    rem = &mut rem[..rlen - 1];
                } else {
                    let lb = lens[cb];
                    emit_short_back::<CFG>(&inline[cb], lb, spelling, &mut rem);
                }
            }
            if front == next {
                done = true;
                break;
            }
        }
    }
    let (lo_rem, rem_len) = emit_bi_tail(
        edges, spelling, inline, lens, shift, mask, out_base, rem, done, prev, front, back, next,
    );
    finalize_bi_out(lo_rem, rem_len, n, out);
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn emit_bidirectional_cfg<const CFG: u32>(
    edges: &[u32],
    spelling: &[u8],
    inline_padded: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    before: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    if !has(CFG, BIT_S5_ZERO_SPILL) {
        emit_bi_c53_baseline(
            edges,
            spelling,
            &inline_padded[1..],
            lens,
            n,
            mark,
            before,
            bits,
            out,
        );
        return;
    }
    match bits {
        17 => emit_bi_specialized::<CFG, 17>(
            edges,
            spelling,
            inline_padded,
            lens,
            n,
            mark,
            before,
            17,
            out,
        ),
        18 => emit_bi_specialized::<CFG, 18>(
            edges,
            spelling,
            inline_padded,
            lens,
            n,
            mark,
            before,
            18,
            out,
        ),
        19 => emit_bi_specialized::<CFG, 19>(
            edges,
            spelling,
            inline_padded,
            lens,
            n,
            mark,
            before,
            19,
            out,
        ),
        _ => emit_bi_specialized::<CFG, 0>(
            edges,
            spelling,
            inline_padded,
            lens,
            n,
            mark,
            before,
            bits,
            out,
        ),
    }
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_bidirectional_sentinel(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    before: u32,
    bits: u32,
    sentinel: u32,
    out: &mut Vec<u8>,
) {
    let out_buf = &mut out[..n];
    let mask = (1 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
    for _ in 0..=edges.len() / 2 {
        let ef = edges[front as usize];
        let raw_cf = ef >> bits;
        let cf = if raw_cf == sentinel {
            extra[front as usize]
        } else {
            raw_cf
        } as usize;
        let single = front == back;
        let lf = lens[cf];
        if lf <= INLINE as u8 && hi - lo >= INLINE {
            out_buf[lo..lo + INLINE].copy_from_slice(&inline[cf]);
            lo += usize::from(lf);
        } else {
            let word = if lf == LONG {
                long_word(&inline[cf], spelling)
            } else {
                &inline[cf][..usize::from(lf)]
            };
            let take = word.len().min(hi - lo);
            out_buf[lo..lo + take].copy_from_slice(&word[..take]);
            lo += take;
        }
        if single || lo == hi {
            break;
        }
        let eb = edges[back as usize];
        let raw_cb = eb >> bits;
        let cb = if raw_cb == sentinel {
            extra[back as usize]
        } else {
            raw_cb
        } as usize;
        let lb = lens[cb];
        if lb <= INLINE as u8 && hi - lo >= INLINE {
            let lbu = usize::from(lb);
            let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - lbu));
            out_buf[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= lbu;
        } else {
            let word = if lb == LONG {
                long_word(&inline[cb], spelling)
            } else {
                &inline[cb][..usize::from(lb)]
            };
            let take = word.len().min(hi - lo);
            out_buf[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
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

fn run_arm<const TIME: bool, const FORCE_OPEN: bool>(
    arm: usize,
    k: &mut AblationKernel,
    records: &[u16],
    n: usize,
    p: u32,
    out: &mut Vec<u8>,
) -> (bool, StageTimes, WsSnap) {
    match arm {
        0 => k.run_cfg::<CFG_A00, TIME, FORCE_OPEN>(records, n, p, out),
        1 => k.run_cfg::<CFG_A01, TIME, FORCE_OPEN>(records, n, p, out),
        2 => k.run_cfg::<CFG_A02, TIME, FORCE_OPEN>(records, n, p, out),
        3 => k.run_cfg::<CFG_A03, TIME, FORCE_OPEN>(records, n, p, out),
        4 => k.run_cfg::<CFG_A04, TIME, FORCE_OPEN>(records, n, p, out),
        5 => k.run_cfg::<CFG_A05, TIME, FORCE_OPEN>(records, n, p, out),
        6 => k.run_cfg::<CFG_A06, TIME, FORCE_OPEN>(records, n, p, out),
        7 => k.run_cfg::<CFG_A07, TIME, FORCE_OPEN>(records, n, p, out),
        8 => k.run_cfg::<CFG_A08, TIME, FORCE_OPEN>(records, n, p, out),
        9 => k.run_cfg::<CFG_A09, TIME, FORCE_OPEN>(records, n, p, out),
        10 => k.run_cfg::<CFG_A10, TIME, FORCE_OPEN>(records, n, p, out),
        11 => k.run_cfg::<CFG_A11, TIME, FORCE_OPEN>(records, n, p, out),
        12 => k.run_cfg::<CFG_A12, TIME, FORCE_OPEN>(records, n, p, out),
        13 => k.run_cfg::<CFG_A13, TIME, FORCE_OPEN>(records, n, p, out),
        14 => k.run_cfg::<CFG_A14, TIME, FORCE_OPEN>(records, n, p, out),
        15 => k.run_cfg::<CFG_A15, TIME, FORCE_OPEN>(records, n, p, out),
        16 => k.run_cfg::<CFG_A16, TIME, FORCE_OPEN>(records, n, p, out),
        17 => k.run_cfg::<CFG_A17, TIME, FORCE_OPEN>(records, n, p, out),
        18 => k.run_cfg::<CFG_A18, TIME, FORCE_OPEN>(records, n, p, out),
        19 => k.run_cfg::<CFG_A19, TIME, FORCE_OPEN>(records, n, p, out),
        _ => unreachable!(),
    }
}

// ============================================================================
// Corpora, Exactness, WS Audit, Multi-Worker Bench & Profile Driver
// ============================================================================
struct Block {
    name: String,
    idx: u32,
    n: usize,
    p: u32,
    records: Vec<u16>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
    for &b in l {
        if let Some(last) = recs.last_mut() {
            if (*last >> 8) as u8 == b && (*last & 255) < 127 {
                *last += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

fn ref_fl_from_records(records: &[u16], n: usize, p: u32) -> Vec<u8> {
    let mut counts = [0u32; 256];
    for &rec in records {
        counts[(rec >> 8) as usize] += u32::from(rec & 255);
    }
    let mut st = starts(counts);
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
    {
        let mut rng = SplitMix64(0xC54A_B1A7_1000_0001);
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
                assert!(ok && out == text, "arm {} failed binary n={n}", ARM_NAMES[arm]);
            }
            small_valid += 1;
        }
    }

    let mut rng = SplitMix64(0xC54E_8AC7_5760_0001);
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
        for (arm, k) in kernels.iter_mut().enumerate() {
            out.clear();
            out.resize(n + 23, if case % 2 == 0 { 0xAA } else { 0x55 });
            let (ok, _, _) = run_arm::<false, true>(arm, k, &recs, n, p, &mut out);
            assert!(ok && out == text, "arm {} failed valid case {case}", ARM_NAMES[arm]);
        }
        small_valid += 1;
    }

    let mut malformed_n = 0usize;
    for case in 0..330usize {
        let n = 1 + (rng.next() as usize % 256);
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
            let (ok, _, _) = run_arm::<false, true>(arm, k, &recs, n, p, &mut out);
            assert!(ok && out == want, "arm {} failed malformed {case}", ARM_NAMES[arm]);
        }
        malformed_n += 1;
    }

    // Detached malformed cycles d = 1..=65 across ALL distinct S5/S4 arms!
    for d in 1..=65usize {
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
        let recs = records_of(&l);
        let want = ref_fl_from_records(&recs, sz, 0);
        for &arm in &[0usize, 2, 3, 8, 9, 10, 11, 16, 17] {
            out.clear();
            out.resize(sz + 37, 0xAA);
            let (ok, _, _) = run_arm::<false, true>(arm, &mut kernels[arm], &recs, sz, 0, &mut out);
            assert!(ok && out == want, "arm {} failed detached d={d}", ARM_NAMES[arm]);
        }
    }

    // Forced sentinel AND forced u16 dict fallback!
    {
        let sz = 75_000usize;
        let mut l = vec![0u8; sz];
        for (i, slot) in l.iter_mut().enumerate() {
            *slot = ((i * 29 + (i >> 4) * 7) % 180) as u8;
        }
        for pos in (0..sz - 200).step_by(300) {
            let b = (180 + ((pos / 300) % 40)) as u8;
            for j in 0..24 {
                l[pos + j] = b;
            }
        }
        let recs = records_of(&l);
        let want = ref_fl_from_records(&recs, sz, 123);
        for &arm in &[0usize, 2, 8, 11, 16, 17] {
            kernels[arm].force_sentinel_cap = Some(260);
            kernels[arm].force_u16_cap = Some(280);
            out.clear();
            let (ok, _, _) =
                run_arm::<false, true>(arm, &mut kernels[arm], &recs, sz, 123, &mut out);
            kernels[arm].force_sentinel_cap = None;
            kernels[arm].force_u16_cap = None;
            assert!(
                ok && out == want,
                "arm {} failed forced sentinel+u16cap",
                ARM_NAMES[arm]
            );
        }
    }

    for blk in stress {
        let want = ref_fl_from_records(&blk.records, blk.n, blk.p);
        for (arm, k) in kernels.iter_mut().enumerate() {
            out.clear();
            out.resize(blk.n + 37, 0xAA);
            let (ok, _, _) = run_arm::<false, true>(arm, k, &blk.records, blk.n, blk.p, &mut out);
            assert!(
                ok && out == want,
                "arm {} failed stress {}",
                ARM_NAMES[arm],
                blk.name
            );
        }
    }

    let mut real_ok = 0usize;
    for blk in wiki6.iter().chain(nonwiki10.iter()).chain(wiki431.iter()) {
        let want = ref_fl_from_records(&blk.records, blk.n, blk.p);
        for (arm, k) in kernels.iter_mut().enumerate() {
            out.clear();
            let (ok, _, _) = run_arm::<false, false>(arm, k, &blk.records, blk.n, blk.p, &mut out);
            assert!(
                ok && out == want,
                "arm {} failed real {}",
                ARM_NAMES[arm],
                blk.name
            );
        }
        real_ok += 1;
    }

    println!(
        "[EXACTNESS PASS] {} small valid + {} malformed + 65 detached (d=1..65) + sentinel/u16fb + {} stress + {}/447 real 100% EXACT across ALL {} arms ({:.2}s)",
        small_valid,
        malformed_n,
        stress.len(),
        real_ok,
        NUM_ARMS,
        t0.elapsed().as_secs_f64()
    );
}

fn audit_ws(blocks: &[Block], label: &str) {
    println!(
        "\n=== Working Set Audit: {} ({} blocks) ===",
        label,
        blocks.len()
    );
    println!(
        "{:<22} {:>9} {:>9} {:>9} {:>8} {:>8} {:>8} {:>8} {:>6} {:>6}",
        "Arm", "PeakCapK", "MaxUsedK", "EdgesCapK", "RulesAvg", "SpAvgK", "SpCapK", "InlCapK",
        "est>m", "s2bGrw"
    );
    for arm in [0usize, 2, 8, 11, 16, 17] {
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
            let (ok, _, snap) =
                run_arm::<false, false>(arm, &mut k, &blk.records, blk.n, blk.p, &mut out);
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
            "{:<22} {:>9.2} {:>9.2} {:>9.2} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>6} {:>6}",
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

fn bench_stage_1t(blocks: &[Block], label: &str, rounds: usize, arms: &[usize]) {
    println!(
        "\n=== Per-Stage 1T Wall Ablation (p50 us/blk, {} rounds): {} ({} blocks) ===",
        rounds,
        label,
        blocks.len()
    );
    println!(
        "{:<22} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>8} {:>7}",
        "Arm", "S1", "S2a", "S2b", "S3a", "S4", "S5", "Total", "vsA00"
    );
    let mut per_arm: Vec<Vec<StageTimes>> =
        (0..arms.len()).map(|_| Vec::with_capacity(rounds)).collect();
    let mut kernels: Vec<AblationKernel> =
        (0..arms.len()).map(|_| AblationKernel::default()).collect();
    let mut out = Vec::new();
    for (slot, &arm) in arms.iter().enumerate() {
        for blk in blocks {
            run_arm::<false, false>(arm, &mut kernels[slot], &blk.records, blk.n, blk.p, &mut out);
        }
    }
    for r in 0..rounds {
        for off in 0..arms.len() {
            let slot = (off + r) % arms.len();
            let arm = arms[slot];
            let mut acc = StageTimes::default();
            for blk in blocks {
                let (_, tm, _) = run_arm::<true, false>(
                    arm,
                    &mut kernels[slot],
                    &blk.records,
                    blk.n,
                    blk.p,
                    &mut out,
                );
                std::hint::black_box(&out);
                acc.add(&tm);
            }
            per_arm[slot].push(acc);
        }
    }
    let nb = blocks.len() as f64;
    let p50 = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() / 2]
    };
    let mut base_tot = 1.0f64;
    for (slot, &arm) in arms.iter().enumerate() {
        let mut s1: Vec<f64> = per_arm[slot].iter().map(|t| t.s1 / nb).collect();
        let mut s2a: Vec<f64> = per_arm[slot].iter().map(|t| t.s2a / nb).collect();
        let mut s2b: Vec<f64> = per_arm[slot].iter().map(|t| t.s2b / nb).collect();
        let mut s3a: Vec<f64> = per_arm[slot].iter().map(|t| t.s3a / nb).collect();
        let mut s4: Vec<f64> = per_arm[slot].iter().map(|t| t.s4 / nb).collect();
        let mut s5: Vec<f64> = per_arm[slot].iter().map(|t| t.s5 / nb).collect();
        let mut tot: Vec<f64> = per_arm[slot].iter().map(|t| t.total / nb).collect();
        let t_med = p50(&mut tot);
        if slot == 0 {
            base_tot = t_med;
        }
        println!(
            "{:<22} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>8.1} {:>6.3}x",
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
        let mut res: Vec<Vec<f64>> =
            (0..arms.len()).map(|_| Vec::with_capacity(rounds)).collect();
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
                                run_arm::<false, false>(
                                    arm,
                                    &mut k,
                                    &blk.records,
                                    blk.n,
                                    blk.p,
                                    &mut out,
                                );
                            }
                            bar.wait();
                            let t0 = Instant::now();
                            for _ in 0..iters {
                                for blk in blocks {
                                    run_arm::<false, false>(
                                        arm,
                                        &mut k,
                                        &blk.records,
                                        blk.n,
                                        blk.p,
                                        &mut out,
                                    );
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

#[inline(never)]
fn profile_exec(
    arm: usize,
    k: &mut AblationKernel,
    blocks: &[Block],
    iters: usize,
    out: &mut Vec<u8>,
) -> u64 {
    let mut checksum = 0u64;
    for _ in 0..iters {
        for blk in blocks {
            out.clear();
            let (ok, _, _) = run_arm::<false, false>(arm, k, &blk.records, blk.n, blk.p, out);
            assert!(ok);
            checksum = checksum.wrapping_add(u64::from(out[0]) + u64::from(out[out.len() - 1]));
            std::hint::black_box(&*out);
        }
    }
    checksum
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let wiki431_path = resolve_path(&[
        "/tmp/mibz2_corpora/wikidata_431_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/wikidata_431_blocks.bin",
    ]);

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
        let warmup = args.iter().any(|a| a == "--warmup");
        let blocks = if args.iter().any(|a| a == "--adm6") {
            load_six_admitted()
        } else if args.iter().any(|a| a == "--blk34") {
            let wiki431 = load_bin_blocks(&wiki431_path, None, false);
            select_proxy_blocks(&wiki431, &[34])
        } else if args.iter().any(|a| a == "--blk254") {
            let wiki431 = load_bin_blocks(&wiki431_path, None, false);
            select_proxy_blocks(&wiki431, &[254])
        } else if args.iter().any(|a| a == "--proxy6") {
            let wiki431 = load_bin_blocks(&wiki431_path, None, false);
            select_proxy_blocks(&wiki431, &[34, 108, 178, 254, 327, 391])
        } else {
            let wiki431 = load_bin_blocks(&wiki431_path, None, false);
            select_proxy_blocks(&wiki431, &[34, 108, 178, 254, 327, 391, 60, 163, 265, 372])
        };
        let mut k = AblationKernel::default();
        let mut out = Vec::new();
        if warmup {
            let (ok, _, _) =
                run_arm::<false, false>(arm, &mut k, &blocks[0].records, blocks[0].n, blocks[0].p, &mut out);
            assert!(ok);
        }
        let chk = profile_exec(arm, &mut k, &blocks, iters, &mut out);
        println!(
            "PROFILE_DONE arm={} ({}) blocks={} warmup={} iters={} chk={}",
            arm,
            ARM_NAMES[arm],
            blocks.len(),
            warmup,
            iters,
            chk
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
    let nw_names = [
        "jsonl_0", "jsonl_1", "jsonl_2", "jsonl_3", "jsonl_4", "syslog_0",
    ];
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

    let all_arms: Vec<usize> = (0..NUM_ARMS).collect();
    bench_stage_1t(&proxy6, "6 Full #729 Proxy Blocks", 11, &all_arms);
    bench_stage_1t(&wiki6, "6 Admitted Wikidata", 11, &all_arms);
    bench_stage_1t(&nonwiki6, "6 Non-Wikidata (JSONL + Syslog)", 11, &all_arms);
    bench_stage_1t(&code4, "4 Code & Docs", 7, &all_arms);

    let key_arms = [0, 1, 2, 8, 16, 17, 18, 19];
    bench_multiworker(&proxy10, "6+4 #729 Proxy Blocks", 7, 3, &key_arms);
    bench_multiworker(&wiki6, "6 Admitted Wikidata", 7, 3, &key_arms);
    bench_multiworker(&nonwiki6, "6 Non-Wikidata (JSONL + Syslog)", 7, 5, &key_arms);
    bench_multiworker(&code4, "4 Code & Docs", 5, 2, &key_arms);
    bench_multiworker(&wiki233, "233 Full wikidata.bz2", 3, 1, &key_arms);
    bench_multiworker(&wiki431, "431 ALL wikidata.bz2", 3, 1, &key_arms);
}
