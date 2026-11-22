//! Cycle 55 Phase II Finale: Headroom Oracle v10 (`8` Arms), Classical Atlas Extension `C55-D6` (`CA-56`),
//! and Exactness / Working-Set / Cachegrind / Multi-Worker Verification Harness for **`C55-DramUltra-v2`**.
//!
//! 1. **Classical Atlas Extension `C55-D6` (`CA-56`)**:
//!    - **A. M. Legendre (1787)** convex-dual rate function `I_Leg(E) = sup_{beta} { beta*E - Psi(beta) }`
//!      on `CA-53`'s canonical free energy `Psi(beta)` (verifying exact Legendre-Gibbs duality
//!      `I_Leg(E) == D_KL(P_{beta*} || P_0)` to `< 1e-11`, Pythagorean contraction
//!      `I_1 = D_KL(P || P_{beta*}) + I_Leg(E)` to `< 1e-10`, and Legendre biconjugate involution
//!      `Psi(beta*) == beta*E - I_Leg(E)` to `< 1e-12`!).
//!    - **A. A. Markov (1906, 1913)** 2nd-order chain extension on cyclic `CAP=C` record-length triples
//!      `(R_{t-2}, R_{t-1}, R_t)`: exact 2nd-order flow balance & Perron-Frobenius stationarity (`< 1e-13`),
//!      2nd-order entropy `H_2 = H(R_t | R_{t-1}, R_{t-2})`, 2nd-order conditional mutual information
//!      `I_2 = H_1 - H_2 >= 0`, 2nd-order Boltzmann-Markov multiplicity `(1/R) log2 W_2 <= H_2`,
//!      and strict 8-level hierarchy (`W_2 <= H_2 <= H_1 <= H_Gibbs <= H_marg <= H_Joint <= min(H_Zeta, H_Geom)`).
//!    - **Theorem D6-2A & D6-2B**: Exact order-`k` record-length invariance (closing Cycle 54 Boundary Note #1
//!      with explicit linear vs cyclic wrap check `0 <= m_min_lin - m_min_cyc <= L`) + explicit primitive
//!      single-cycle BWT witness pairs matching ELEMENT-FOR-ELEMENT record/run sequences (`x1 == x2` / `wa == wb`,
//!      hence ALL linear AND cyclic order-`k` transition tensors!) yet hitting `100.0%` of the `m`-gap (`22x..802x`)
//!      and `9x..48x` `S2b` grammar divergence!
//!    - Extends Chain D (`CA-26->27->31->37->41->44->47->50->53->CA-56`, **10 Pure-CA nodes / 9 links**) AND
//!      Chain A+N+P+R+D4+D5 (`CA-05->...->CA-46->CA-50->CA-53->CA-56`, **NEW 13-node / 12-link Pure-CA record!**).
//! 2. **Headroom Oracle v10 (`8` Arms)**:
//!    - Arm 0: `R5-Head-A1`
//!    - Arm 1: `R5-Head-A2(AA)`
//!    - Arm 2: `C54-PipeLean-v2` (`L=16`, `U32`, `S4_8WayEpoch`, `S5_UnalignedBack_1Ptr` + `ZeroSpill_ConstBits`)
//!    - Arm 3: `C54-DramLean-v1` (`L=12`, `U16Tag4`, `SlimS4-8`, `Pipe1Rem16_LE16`)
//!    - Arm 4: `Abl-Fused-L12` (`L=12`, `U16Tag4`, `S4_8WayEpoch` + `S4_SlowEpoch8`, `S5_UnalignedBack_1Ptr`, `TightSteps`)
//!    - Arm 5: `Abl-Pow2-L12` (`Abl-Fused-L12` + Zero-Memset `S5_Pow2Dict2048`)
//!    - Arm 6: `Abl-Pow2-L10` (`L=10` + Zero-Memset `S5_Pow2Dict2048` + `S4_SlowEpoch8` + `TightSteps`)
//!    - Arm 7: **`C55-DramUltra-v2`** (`3-Tier Density-Adaptive L in {8,10,24,32}` + Zero-Memset `S5_Pow2Dict2048`
//!      + `S4_SlowEpoch8` + `C54-PipeLean-v2` + `C54-DramLean-v1` + `TightSteps`!)

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
const ID_BITS: u32 = 24;
const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;

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

fn begin_inline_padded(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, zero_memset: bool) -> usize {
    if lens.len() < 256 || inline16.len() < 257 {
        inline16.clear();
        inline16.reserve_exact(257);
        inline16.push([0; INLINE]);
        inline16.extend((0..=255u8).map(|c| { let mut v = [0; INLINE]; v[0] = c; v }));
        lens.clear();
        lens.reserve_exact(256);
        lens.resize(256, 1);
    }
    if !zero_memset {
        inline16.truncate(257);
        lens.truncate(256);
    }
    256
}

#[cold]
#[inline(never)]
fn grow_step_inline_padded(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
    let target = (lens.len() + 1).div_ceil(step) * step;
    if target > lens.capacity() { lens.reserve_exact(target - lens.len()); }
    let target_il = target + 1;
    if target_il > inline16.capacity() { inline16.reserve_exact(target_il - inline16.len()); }
}

#[inline(always)]
fn ensure_step_inline_padded(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
    if lens.len() == lens.capacity() || inline16.len() == inline16.capacity() {
        grow_step_inline_padded(inline16, lens, step);
    }
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
        tight: bool,
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

        pub fn new(out: &'a mut [u8], spill: &'a mut Spill<BYTES>, limit: u32, tight: bool) -> Self {
            debug_assert_eq!(BYTES, 2 * Self::CHUNK);
            spill.ovf.clear();
            spill.cuts.clear();
            let in_out = (out.len() / BYTES) as u32;
            let out = &mut out[..in_out as usize * BYTES];
            Queues {
                in_out, out, spill, chunks: 0,
                heads: [NONE32; 256], pt: [u32::MAX as u64; 256], limit, over: false, tight,
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
                    let ovf_step = ((if self.tight { 8_192 } else { 16_384 }) / BYTES).max(1);
                    let target = (self.spill.ovf.len() + 1).div_ceil(ovf_step) * ovf_step;
                    self.spill.ovf.reserve_exact(target - self.spill.ovf.len());
                }
                self.spill.ovf.push([0; BYTES]);
            }
            let idu = id as usize;
            if self.spill.next.len() <= idu {
                let step = if self.tight { 128 } else { 256 };
                let target = (idu + 1).div_ceil(step) * step;
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
                if S4_MODE >= 3 {
                    let hi_u = hi as usize;
                    let pack_hi = pack | hi;
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    let mut i = 0usize;
                    if hi_u + (EPOCH as usize) <= edges.len() {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = usize::from(u16::from_le_bytes(ch[0])); let l1 = usize::from(u16::from_le_bytes(ch[1]));
                            let l2 = usize::from(u16::from_le_bytes(ch[2])); let l3 = usize::from(u16::from_le_bytes(ch[3]));
                            let l4 = usize::from(u16::from_le_bytes(ch[4])); let l5 = usize::from(u16::from_le_bytes(ch[5]));
                            let l6 = usize::from(u16::from_le_bytes(ch[6])); let l7 = usize::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | (l0 as u32); eu[1] ^= pack_hi | (l1 as u32);
                            eu[2] ^= pack_hi | (l2 as u32); eu[3] ^= pack_hi | (l3 as u32);
                            eu[4] ^= pack_hi | (l4 as u32); eu[5] ^= pack_hi | (l5 as u32);
                            eu[6] ^= pack_hi | (l6 as u32); eu[7] ^= pack_hi | (l7 as u32);
                            let ep: &mut [u32; EPOCH as usize] = (&mut edges[hi_u..hi_u + (EPOCH as usize)]).try_into().unwrap();
                            let u32_0 = u0 as u32;
                            ep[l0] ^= u32_0; ep[l1] ^= u32_0 + 1; ep[l2] ^= u32_0 + 2; ep[l3] ^= u32_0 + 3;
                            ep[l4] ^= u32_0 + 4; ep[l5] ^= u32_0 + 5; ep[l6] ^= u32_0 + 6; ep[l7] ^= u32_0 + 7;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i; let lo = usize::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | (lo as u32);
                            let ep: &mut [u32; EPOCH as usize] = (&mut edges[hi_u..hi_u + (EPOCH as usize)]).try_into().unwrap();
                            ep[lo] ^= u as u32;
                            i += 1;
                        }
                    } else {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = usize::from(u16::from_le_bytes(ch[0])); let l1 = usize::from(u16::from_le_bytes(ch[1]));
                            let l2 = usize::from(u16::from_le_bytes(ch[2])); let l3 = usize::from(u16::from_le_bytes(ch[3]));
                            let l4 = usize::from(u16::from_le_bytes(ch[4])); let l5 = usize::from(u16::from_le_bytes(ch[5]));
                            let l6 = usize::from(u16::from_le_bytes(ch[6])); let l7 = usize::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | (l0 as u32); eu[1] ^= pack_hi | (l1 as u32);
                            eu[2] ^= pack_hi | (l2 as u32); eu[3] ^= pack_hi | (l3 as u32);
                            eu[4] ^= pack_hi | (l4 as u32); eu[5] ^= pack_hi | (l5 as u32);
                            eu[6] ^= pack_hi | (l6 as u32); eu[7] ^= pack_hi | (l7 as u32);
                            let ep = &mut edges[hi_u..];
                            let u32_0 = u0 as u32;
                            ep[l0] ^= u32_0; ep[l1] ^= u32_0 + 1; ep[l2] ^= u32_0 + 2; ep[l3] ^= u32_0 + 3;
                            ep[l4] ^= u32_0 + 4; ep[l5] ^= u32_0 + 5; ep[l6] ^= u32_0 + 6; ep[l7] ^= u32_0 + 7;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i; let lo = usize::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | (lo as u32);
                            edges[hi_u + lo] ^= u as u32;
                            i += 1;
                        }
                    }
                } else if S4_MODE == 0 {
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
                if S4_MODE == 4 { self.wire_xor_slow_nomark_epoch8(edges, dst_start, span, pack); } else { self.wire_xor_slow_nomark(edges, dst_start, span, pack); }
            }
        }


        #[cold]
        #[inline(never)]
        fn wire_xor_slow_nomark_epoch8(&mut self, edges: &mut [u32], mut dst_start: usize, mut left: usize, pack: u32) {
            while left > 0 {
                while self.read == self.cut { self.epoch += 1; self.cut = self.cut_at(self.epoch as usize); }
                if self.at == Self::CHUNK { self.id = self.next_chunk(self.id as usize); self.at = 0; }
                let take = left.min(Self::CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES; &self.out[at..at + BYTES]
                } else { &self.spill.ovf[(self.id - self.in_out) as usize][..] };
                let hi = self.epoch << 16;
                let hi_u = hi as usize;
                let pack_hi = pack | hi;
                let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                let mut i = 0usize;
                if hi_u + (EPOCH as usize) <= edges.len() {
                    let (chunks8, _) = pairs.as_chunks::<8>();
                    for ch in chunks8 {
                        let l0 = usize::from(u16::from_le_bytes(ch[0])); let l1 = usize::from(u16::from_le_bytes(ch[1]));
                        let l2 = usize::from(u16::from_le_bytes(ch[2])); let l3 = usize::from(u16::from_le_bytes(ch[3]));
                        let l4 = usize::from(u16::from_le_bytes(ch[4])); let l5 = usize::from(u16::from_le_bytes(ch[5]));
                        let l6 = usize::from(u16::from_le_bytes(ch[6])); let l7 = usize::from(u16::from_le_bytes(ch[7]));
                        let u0 = dst_start + i;
                        let eu = &mut edges[u0..u0 + 8];
                        eu[0] ^= pack_hi | (l0 as u32); eu[1] ^= pack_hi | (l1 as u32);
                        eu[2] ^= pack_hi | (l2 as u32); eu[3] ^= pack_hi | (l3 as u32);
                        eu[4] ^= pack_hi | (l4 as u32); eu[5] ^= pack_hi | (l5 as u32);
                        eu[6] ^= pack_hi | (l6 as u32); eu[7] ^= pack_hi | (l7 as u32);
                        let ep: &mut [u32; EPOCH as usize] = (&mut edges[hi_u..hi_u + (EPOCH as usize)]).try_into().unwrap();
                        let u32_0 = u0 as u32;
                        ep[l0] ^= u32_0; ep[l1] ^= u32_0 + 1; ep[l2] ^= u32_0 + 2; ep[l3] ^= u32_0 + 3;
                        ep[l4] ^= u32_0 + 4; ep[l5] ^= u32_0 + 5; ep[l6] ^= u32_0 + 6; ep[l7] ^= u32_0 + 7;
                        i += 8;
                    }
                    for &p in &pairs[i..] {
                        let u = dst_start + i; let lo = usize::from(u16::from_le_bytes(p));
                        edges[u] ^= pack_hi | (lo as u32);
                        let ep: &mut [u32; EPOCH as usize] = (&mut edges[hi_u..hi_u + (EPOCH as usize)]).try_into().unwrap();
                        ep[lo] ^= u as u32;
                        i += 1;
                    }
                } else {
                    let (chunks8, _) = pairs.as_chunks::<8>();
                    for ch in chunks8 {
                        let l0 = usize::from(u16::from_le_bytes(ch[0])); let l1 = usize::from(u16::from_le_bytes(ch[1]));
                        let l2 = usize::from(u16::from_le_bytes(ch[2])); let l3 = usize::from(u16::from_le_bytes(ch[3]));
                        let l4 = usize::from(u16::from_le_bytes(ch[4])); let l5 = usize::from(u16::from_le_bytes(ch[5]));
                        let l6 = usize::from(u16::from_le_bytes(ch[6])); let l7 = usize::from(u16::from_le_bytes(ch[7]));
                        let u0 = dst_start + i;
                        let eu = &mut edges[u0..u0 + 8];
                        eu[0] ^= pack_hi | (l0 as u32); eu[1] ^= pack_hi | (l1 as u32);
                        eu[2] ^= pack_hi | (l2 as u32); eu[3] ^= pack_hi | (l3 as u32);
                        eu[4] ^= pack_hi | (l4 as u32); eu[5] ^= pack_hi | (l5 as u32);
                        eu[6] ^= pack_hi | (l6 as u32); eu[7] ^= pack_hi | (l7 as u32);
                        let ep = &mut edges[hi_u..];
                        let u32_0 = u0 as u32;
                        ep[l0] ^= u32_0; ep[l1] ^= u32_0 + 1; ep[l2] ^= u32_0 + 2; ep[l3] ^= u32_0 + 3;
                        ep[l4] ^= u32_0 + 4; ep[l5] ^= u32_0 + 5; ep[l6] ^= u32_0 + 6; ep[l7] ^= u32_0 + 7;
                        i += 8;
                    }
                    for &p in &pairs[i..] {
                        let u = dst_start + i; let lo = usize::from(u16::from_le_bytes(p));
                        edges[u] ^= pack_hi | (lo as u32);
                        edges[hi_u + lo] ^= u as u32;
                        i += 1;
                    }
                }
                self.at += take; self.read += take as u32; dst_start += take; left -= take;
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

mod engine_c55 {
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
            if g <= 8 {
                let sl = &self.rules_buf[r..r + 2];
                u128::from(sl[0]) | (u128::from(sl[1]) << 32)
            } else if g <= 12 {
                let sl = &self.rules_buf[r..r + 3];
                u128::from(sl[0]) | (u128::from(sl[1]) << 32) | (u128::from(sl[2]) << 64)
            } else {
                let sl = &self.rules_buf[r..r + 4];
                u128::from(sl[0]) | (u128::from(sl[1]) << 32) | (u128::from(sl[2]) << 64) | (u128::from(sl[3]) << 96)
            }
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
        fn inline_word(&mut self, word: u32, byte: u8, istk: &mut [u32; INLINE_STACK], memo_rec: &mut [u32], stack: &mut Vec<u32>, spelling: &mut Vec<u8>) -> ([u8; INLINE], u8) {
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
        fn append_tail_unw<const TRACK_MEMO: bool>(&mut self, handle: u32, istk: &mut [u32; INLINE_STACK], ovf_stack: &mut Vec<u32>, dest: &mut Vec<u8>, memo_rec: &mut [u32], memo_n: &mut usize, memo_ovf: &mut bool) {
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
                    if *memo_n < memo_rec.len() { memo_rec[*memo_n] = m_off as u32; *memo_n += 1; } else { *memo_ovf = true; }
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
        U16Tag4 { tbl: &'a mut [u16; 2048], fb: &'a mut Vec<u32>, cap: u32 },
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
            }
        }

        #[inline(always)]
        fn store(&mut self, index: usize, h: u64, code: u32) {
            match self {
                ContentsStore::U32(tbl) => {
                    let tag = (((h >> 56) as u32) | 0x80) << 24;
                    tbl[index] = tag | code;
                }
                ContentsStore::U16Tag4 { tbl, fb, cap } => {
                    if fb.is_empty() && code < *cap {
                        let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
                        tbl[index] = tag4 | (code as u16);
                    } else {
                        Self::store_u32_fallback_tag4(tbl, fb, index, code);
                    }
                }
            }
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
    }

    struct Dictionary<'a, const PADDED: bool, const ZERO_MEMSET: bool, const TIGHT: bool> {
        pairs: &'a mut [u32; 512],
        contents: ContentsStore<'a>,
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        num_codes: &'a mut usize,
        spelling: &'a mut Vec<u8>,
        istk: &'a mut [u32; INLINE_STACK],
        memo_rec: &'a mut [u32],
        stack: &'a mut Vec<u32>,
    }

    impl<const PADDED: bool, const ZERO_MEMSET: bool, const TIGHT: bool> Dictionary<'_, PADDED, ZERO_MEMSET, TIGHT> {
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
                    if self.lens[old] == len && self.inline16[old + usize::from(PADDED)] == entry {
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
            ensure_step_u8(self.spelling, rlen + 1, if TIGHT { 1024 } else { SPELLING_STEP });
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
                if self.lens[old] == LONG && long_word(&self.inline16[old + usize::from(PADDED)], self.spelling) == bytes {
                    if memoized {
                        self.spelling.truncate(start);
                    } else if !memo_ovf {
                        let old_start = u32::from_le_bytes(self.inline16[old + usize::from(PADDED)][..4].try_into().unwrap());
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
            let step = if TIGHT { 64 } else { INLINE_STEP };
            if ZERO_MEMSET {
                let code = *self.num_codes;
                *self.num_codes = code + 1;
                if code < self.lens.len() {
                    self.inline16[code + usize::from(PADDED)] = entry;
                    self.lens[code] = len;
                } else {
                    if PADDED { ensure_step_inline_padded(self.inline16, self.lens, step); }
                    else { ensure_step_inline(self.inline16, self.lens, step); }
                    self.inline16.push(entry);
                    self.lens.push(len);
                }
                code as u32
            } else {
                if PADDED { ensure_step_inline_padded(self.inline16, self.lens, step); }
                else { ensure_step_inline(self.inline16, self.lens, step); }
                let code = self.lens.len() as u32;
                self.inline16.push(entry);
                self.lens.push(len);
                *self.num_codes = self.lens.len();
                code
            }
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
                            grow_edges_s1_step::<S1_GROW>(self.edges, at + 3);
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
    fn grow_edges_s1_step<const S1_GROW: u8>(a: &mut Vec<u32>, need: usize) {
        let step = if S1_GROW >= 2 { 2048 } else { 4096 };
        let target = need.div_ceil(step) * step;
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
        num_codes: usize,
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
                num_codes: 0,
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
                    queues: outq_gen::Queues::new(&mut out[..], &mut self.spill, cap.m.min(n as u32), false),
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
            let wide = self.num_codes - 1 > sentinel as usize;
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
            let ncodes = self.num_codes;
            let s5_live = 4 * m + 4 * self.extra.len() + 16 * (ncodes + usize::from(S5_MODE >= 3)) + ncodes + self.spelling.len();
            let used_bytes = 4 * live_words.max(m) + 4 * self.extra.len() + 8 * self.bands.len()
                + self.spelling.len() + 16 * (ncodes + usize::from(S5_MODE >= 3)) + ncodes + outq_spill_used
                + 4 * self.contents_u32_fb.len();

            if let Some(before) = self.predecessor {
                if !wide {
                    if S5_MODE == 4 {
                        if ncodes <= 2048 && (bits == 17 || bits == 18 || bits == 19) {
                            if self.lens.len() < 2048 {
                                if self.lens.capacity() < 2048 {
                                    self.lens.reserve_exact(2048 - self.lens.len());
                                    self.inline16.reserve_exact(2049 - self.inline16.len());
                                }
                                self.lens.resize(2048, 1);
                                self.inline16.resize(2049, [0; INLINE]);
                            }
                            Self::emit_bi_pow2(&self.edges[..m], &self.spelling, &self.inline16[..2049], &self.lens[..2048], n, f.mark, before, bits, out);
                        } else {
                            Self::emit_bi_unaligned(&self.edges[..m], &self.spelling, &self.inline16[..ncodes + 1], &self.lens[..ncodes], n, f.mark, before, bits, out);
                        }
                    } else if S5_MODE == 3 {
                        Self::emit_bi_unaligned(&self.edges[..m], &self.spelling, &self.inline16[..ncodes + 1], &self.lens[..ncodes], n, f.mark, before, bits, out);
                    } else if S5_MODE == 0 {
                        Self::emit_bi_v3(&self.edges[..m], &self.spelling, &self.inline16, &self.lens, n, f.mark, before, bits, out);
                    } else if S5_MODE == 1 {
                        Self::emit_bi_rem16(&self.edges[..m], &self.spelling, &self.inline16, &self.lens, n, f.mark, before, bits, out);
                    } else {
                        Self::emit_bi_pipe1_rem16(&self.edges[..m], &self.spelling, &self.inline16, &self.lens, n, f.mark, before, bits, out);
                    }
                } else {
                    let il_sl = if S5_MODE >= 3 { &self.inline16[1..ncodes + 1] } else { &self.inline16[..ncodes] };
                    Self::emit_bi_sentinel(&self.edges[..m], &self.extra, &self.spelling, il_sl, &self.lens[..ncodes], n, f.mark, before, bits, sentinel, out);
                }
            } else {
                let il_sl = if S5_MODE >= 3 { &self.inline16[1..ncodes + 1] } else { &self.inline16[..ncodes] };
                Self::emit_uni(&self.edges[..m], &self.extra, &self.spelling, il_sl, &self.lens[..ncodes], n, f.mark, bits, sentinel, out);
            }

            Some(Counts {
                m, bits, num_codes: ncodes, wide, used_bi: use_xor, used_tail,
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
                let gneed = hdr_words + base + rule_top + SPIRAL_RULES * MAX_RULE_WORDS;
                if S1_GROW >= 2 { let t = gneed.div_ceil(2048) * 2048; if t > self.edges.capacity() { self.edges.reserve_exact(t - self.edges.len()); } self.edges.resize(t, 0); } else { grow_edges_step(&mut self.edges, gneed); }
            }

            let pad = 0usize;
            let min_live = hdr_words + base + rule_top + pad;
            if self.edges.len() < min_live {
                self.edges.reserve_exact(min_live - self.edges.len());
                self.edges.resize(min_live, 0);
            }
            let needed_cache_words = if DICT_MODE == 0 { CACHE_WORDS_U32 } else { 0 };
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

            if S5_MODE >= 3 {
                self.num_codes = begin_inline_padded(&mut self.inline16, &mut self.lens, S5_MODE == 4);
            } else {
                begin_inline(&mut self.inline16, &mut self.lens);
                self.num_codes = 256;
            }
            let max_bands = open_nodes + 256;
            let use_tail = !self.force_band_vec && out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec64 = [0u32; 64];
            let memo_rec: &mut [u32] = if S1_GROW >= 2 { &mut memo_rec64[..] } else { &mut memo_rec64[..MEMO_REC_CAP] };

            if DICT_MODE == 2 {
                let mut pairs_stk = [0u32; 512];
                let mut contents_stk = [0u16; 2048];
                let cap = self.force_u16_cap.unwrap_or(4_096).min(4_096);
                let store = ContentsStore::U16Tag4 { tbl: &mut contents_stk, fb: &mut self.contents_u32_fb, cap };
                if S5_MODE == 4 {
                    if S1_GROW >= 2 {
                        let mut d = Dictionary::<true, true, true> { pairs: &mut pairs_stk, contents: store, inline16: &mut self.inline16, lens: &mut self.lens, num_codes: &mut self.num_codes, spelling: &mut self.spelling, istk: &mut istk, memo_rec, stack: &mut self.stack };
                        Self::emit_bands_dict(&mut e, &mut d, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors, &mut nbands);
                    } else {
                        let mut d = Dictionary::<true, true, false> { pairs: &mut pairs_stk, contents: store, inline16: &mut self.inline16, lens: &mut self.lens, num_codes: &mut self.num_codes, spelling: &mut self.spelling, istk: &mut istk, memo_rec, stack: &mut self.stack };
                        Self::emit_bands_dict(&mut e, &mut d, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors, &mut nbands);
                    }
                } else if S5_MODE == 3 {
                    if S1_GROW >= 2 {
                        let mut d = Dictionary::<true, false, true> { pairs: &mut pairs_stk, contents: store, inline16: &mut self.inline16, lens: &mut self.lens, num_codes: &mut self.num_codes, spelling: &mut self.spelling, istk: &mut istk, memo_rec, stack: &mut self.stack };
                        Self::emit_bands_dict(&mut e, &mut d, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors, &mut nbands);
                    } else {
                        let mut d = Dictionary::<true, false, false> { pairs: &mut pairs_stk, contents: store, inline16: &mut self.inline16, lens: &mut self.lens, num_codes: &mut self.num_codes, spelling: &mut self.spelling, istk: &mut istk, memo_rec, stack: &mut self.stack };
                        Self::emit_bands_dict(&mut e, &mut d, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors, &mut nbands);
                    }
                } else {
                    let mut d = Dictionary::<false, false, false> { pairs: &mut pairs_stk, contents: store, inline16: &mut self.inline16, lens: &mut self.lens, num_codes: &mut self.num_codes, spelling: &mut self.spelling, istk: &mut istk, memo_rec, stack: &mut self.stack };
                    Self::emit_bands_dict(&mut e, &mut d, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors, &mut nbands);
                }
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
                                let (entry, len) = e.inline_word(node_word, a as u8, &mut istk, memo_rec, &mut self.stack, &mut self.spelling);
                                if S5_MODE >= 3 { ensure_step_inline_padded(&mut self.inline16, &mut self.lens, INLINE_STEP); }
                                else { ensure_step_inline(&mut self.inline16, &mut self.lens, INLINE_STEP); }
                                let c = self.lens.len() as u32; self.inline16.push(entry); self.lens.push(len); self.num_codes = self.lens.len(); c
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
                    let store = ContentsStore::U32(c_sl.try_into().unwrap());
                    if S5_MODE >= 3 {
                        let mut d = Dictionary::<true, false, false> { pairs, contents: store, inline16: &mut self.inline16, lens: &mut self.lens, num_codes: &mut self.num_codes, spelling: &mut self.spelling, istk: &mut istk, memo_rec, stack: &mut self.stack };
                        Self::emit_bands_dict(&mut e, &mut d, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors, &mut nbands);
                    } else {
                        let mut d = Dictionary::<false, false, false> { pairs, contents: store, inline16: &mut self.inline16, lens: &mut self.lens, num_codes: &mut self.num_codes, spelling: &mut self.spelling, istk: &mut istk, memo_rec, stack: &mut self.stack };
                        Self::emit_bands_dict(&mut e, &mut d, thin, &mut self.bands, out_tail, max_bands, use_tail, &mut cursors, &mut nbands);
                    }
                }
            }
            (true, cursors, use_tail, nbands, hdr_words + base + rule_top + pad, rule_top, s2b_steps)
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bands_dict<L: Links, const PADDED: bool, const ZERO_MEMSET: bool, const TIGHT: bool>(
            e: &mut Elim<L>, dictionary: &mut Dictionary<'_, PADDED, ZERO_MEMSET, TIGHT>, thin: &[u32; 256],
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
                    let ta = thin[a];
                    if ta == 0 { continue; }
                    let end = k + ta;
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
                    let ta = thin[a];
                    if ta == 0 { continue; }
                    let end = k + ta;
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
                    if lf <= INLINE as u8 {
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
                    if lb <= INLINE as u8 {
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


        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi_unaligned_const<const BITS: u32>(
            edges: &[u32], spelling: &[u8], inline_padded: &[[u8; INLINE]], lens: &[u8],
            n: usize, mark: u32, before: u32, bits_rt: u32, out: &mut Vec<u8>,
        ) {
            let shift = if BITS != 0 { BITS } else { bits_rt };
            let mask = (1u32 << shift) - 1;
            let (mut prev, mut front, mut back, mut next) = (before, mark, before, mark);
            let out_base = out.as_ptr() as usize;
            let mut rem: &mut [u8] = &mut out[..n];
            let len_c = lens.len().min(inline_padded.len().saturating_sub(1));
            let lens = &lens[..len_c];
            let inline_flat: &[u8] = inline_padded[..len_c + 1].as_flattened();
            let mut done = false;
            if rem.len() >= 2 * INLINE {
                let mut ef = edges[front as usize];
                let mut eb = edges[back as usize];
                while rem.len() >= 2 * INLINE {
                    if front == back {
                        let cf = (ef >> shift) as usize;
                        let lf = lens[cf];
                        if lf <= INLINE as u8 {
                            let off_f = (cf + 1) << 4;
                            rem[..INLINE].copy_from_slice(&inline_flat[off_f..off_f + INLINE]);
                            rem = &mut rem[usize::from(lf)..];
                        } else {
                            let take = emit_long_front(&inline_padded[cf + 1], spelling, rem);
                            rem = &mut rem[take..];
                        }
                        done = true; break;
                    }
                    let cf = (ef >> shift) as usize;
                    let cb = (eb >> shift) as usize;
                    let f_next = (ef & mask) ^ prev;
                    let b_next = (eb & mask) ^ next;
                    prev = front; front = f_next;
                    next = back;  back = b_next;
                    ef = edges[front as usize];
                    eb = edges[back as usize];
                    let lf = lens[cf];
                    if lf <= INLINE as u8 {
                        let off_f = (cf + 1) << 4;
                        rem[..INLINE].copy_from_slice(&inline_flat[off_f..off_f + INLINE]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline_padded[cf + 1], spelling, rem);
                        rem = &mut rem[take..];
                        if rem.is_empty() { done = true; break; }
                        if rem.len() < INLINE {
                            let take_b = emit_tail_back(&inline_padded[cb + 1], lens[cb], spelling, rem);
                            let rlen = rem.len(); rem = &mut rem[..rlen - take_b];
                            if front == next { done = true; }
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
                    if front == next { done = true; break; }
                }
            }
            if !done {
                let inline = &inline_padded[1..len_c + 1];
                while !rem.is_empty() {
                    let ef = edges[front as usize]; let cf = (ef >> shift) as usize; let single = front == back; let lf = lens[cf];
                    let word = if lf == LONG { long_word(&inline[cf], spelling) } else { &inline[cf][..usize::from(lf)] };
                    let take = word.len().min(rem.len()); rem[..take].copy_from_slice(&word[..take]); rem = &mut rem[take..];
                    if single || rem.is_empty() { break; }
                    let eb = edges[back as usize]; let cb = (eb >> shift) as usize;
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

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi_unaligned(
            edges: &[u32], spelling: &[u8], inline_padded: &[[u8; INLINE]], lens: &[u8],
            n: usize, mark: u32, before: u32, bits: u32, out: &mut Vec<u8>,
        ) {
            match bits {
                17 => Self::emit_bi_unaligned_const::<17>(edges, spelling, inline_padded, lens, n, mark, before, 17, out),
                18 => Self::emit_bi_unaligned_const::<18>(edges, spelling, inline_padded, lens, n, mark, before, 18, out),
                19 => Self::emit_bi_unaligned_const::<19>(edges, spelling, inline_padded, lens, n, mark, before, 19, out),
                _  => Self::emit_bi_unaligned_const::<0>(edges, spelling, inline_padded, lens, n, mark, before, bits, out),
            }
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi_pow2_const<const BITS: u32>(
            edges: &[u32], spelling: &[u8], inline_arr: &[[u8; INLINE]; 2049], lens_arr: &[u8; 2048],
            n: usize, mark: u32, before: u32, out: &mut Vec<u8>,
        ) {
            let mask = (1u32 << BITS) - 1;
            let mut front = mark;
            let mut back = before;
            let mut pn: u64 = (u64::from(mark) << 32) | u64::from(before);
            let out_base = out.as_ptr() as usize;
            let mut rem: &mut [u8] = &mut out[..n];
            let inline_flat: &[u8; 2049 * INLINE] = inline_arr.as_flattened().try_into().unwrap();
            let mut done = false;
            if rem.len() >= 2 * INLINE {
                let mut ef = edges[front as usize];
                let mut eb = edges[back as usize];
                while rem.len() >= 2 * INLINE {
                    if front == back {
                        let cf = ((ef >> BITS) & 2047) as usize;
                        let lf = lens_arr[cf];
                        if lf <= INLINE as u8 {
                            let off_f = (cf + 1) << 4;
                            rem[..INLINE].copy_from_slice(&inline_flat[off_f..off_f + INLINE]);
                            rem = &mut rem[usize::from(lf)..];
                        } else {
                            let take = emit_long_front(&inline_arr[cf + 1], spelling, rem);
                            rem = &mut rem[take..];
                        }
                        done = true; break;
                    }
                    let cf = ((ef >> BITS) & 2047) as usize;
                    let cb = ((eb >> BITS) & 2047) as usize;
                    let prev = pn as u32;
                    let next = (pn >> 32) as u32;
                    let f_next = (ef & mask) ^ prev;
                    let b_next = (eb & mask) ^ next;
                    pn = (u64::from(back) << 32) | u64::from(front);
                    front = f_next;
                    back = b_next;
                    ef = edges[front as usize];
                    eb = edges[back as usize];
                    let lf = lens_arr[cf];
                    if lf <= INLINE as u8 {
                        let off_f = (cf + 1) << 4;
                        rem[..INLINE].copy_from_slice(&inline_flat[off_f..off_f + INLINE]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline_arr[cf + 1], spelling, rem);
                        rem = &mut rem[take..];
                        if rem.is_empty() { done = true; break; }
                        if rem.len() < INLINE {
                            let take_b = emit_tail_back(&inline_arr[cb + 1], lens_arr[cb], spelling, rem);
                            let rlen = rem.len(); rem = &mut rem[..rlen - take_b];
                            if front == (pn >> 32) as u32 { done = true; }
                            break;
                        }
                    }
                    let lb = lens_arr[cb];
                    let rlen = rem.len();
                    if lb <= INLINE as u8 {
                        let lbu = usize::from(lb);
                        let off = (cb << 4) + lbu;
                        rem[rlen - INLINE..].copy_from_slice(&inline_flat[off..off + INLINE]);
                        rem = &mut rem[..rlen - lbu];
                    } else {
                        let take = emit_long_back(&inline_arr[cb + 1], spelling, rem);
                        rem = &mut rem[..rlen - take];
                    }
                    if front == (pn >> 32) as u32 { done = true; break; }
                }
            }
            if !done {
                let mut prev = pn as u32;
                let mut next = (pn >> 32) as u32;
                while !rem.is_empty() {
                    let ef = edges[front as usize]; let cf = ((ef >> BITS) & 2047) as usize; let single = front == back; let lf = lens_arr[cf];
                    let word = if lf == LONG { long_word(&inline_arr[cf + 1], spelling) } else { &inline_arr[cf + 1][..usize::from(lf)] };
                    let take = word.len().min(rem.len()); rem[..take].copy_from_slice(&word[..take]); rem = &mut rem[take..];
                    if single || rem.is_empty() { break; }
                    let eb = edges[back as usize]; let cb = ((eb >> BITS) & 2047) as usize;
                    let take_b = emit_tail_back(&inline_arr[cb + 1], lens_arr[cb], spelling, rem);
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

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi_pow2(
            edges: &[u32], spelling: &[u8], inline_padded: &[[u8; INLINE]], lens: &[u8],
            n: usize, mark: u32, before: u32, bits: u32, out: &mut Vec<u8>,
        ) {
            let inline_arr: &[[u8; INLINE]; 2049] = inline_padded.try_into().unwrap();
            let lens_arr: &[u8; 2048] = lens.try_into().unwrap();
            match bits {
                17 => Self::emit_bi_pow2_const::<17>(edges, spelling, inline_arr, lens_arr, n, mark, before, out),
                18 => Self::emit_bi_pow2_const::<18>(edges, spelling, inline_arr, lens_arr, n, mark, before, out),
                _  => Self::emit_bi_pow2_const::<19>(edges, spelling, inline_arr, lens_arr, n, mark, before, out),
            }
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
fn select_limit_with_def<const DEF_LIM: u32, const ADAPTIVE_L8: bool>(records: &[u16], n: usize) -> (u32, u32) {
    let nrec = records.len();
    if n < 32_768 || nrec < 256 { return (DEF_LIM, 0); }
    if ADAPTIVE_L8 && (nrec as u64) * 100 > (n as u64) * 24 { return (12, 0); }
    if ADAPTIVE_L8 && (nrec as u64) * 100 >= (n as u64) * 14 { return (8, 0); }
    let step = nrec / 256;
    let mut idx = step / 2;
    let (mut s_w1, mut s_r17) = (0u32, 0u32);
    let step4 = step * 4;
    for _ in 0..64 {
        let w = u32::from(records[idx] & 255);
        s_w1 += u32::from(w == 1); s_r17 += u32::from(w > 16 && w <= 32); idx += step4;
    }
    if ((nrec as u64 * u64::from(s_w1)) >> 6) > 39_500 || s_r17 > 2 { return (DEF_LIM, 64); }
    idx = step / 2; s_w1 = 0; s_r17 = 0;
    for _ in 0..256 {
        let w = u32::from(records[idx] & 255);
        s_w1 += u32::from(w == 1); s_r17 += u32::from(w > 16 && w <= 32); idx += step;
    }
    let est_w1 = (nrec as u64 * u64::from(s_w1)) >> 8;
    if est_w1 <= 33_500 {
        if s_r17 <= 3 { (32, 256) } else if s_r17 <= 5 { (24, 256) } else { (DEF_LIM, 256) }
    } else { (DEF_LIM, 256) }
}

const POST_S1_BI: BiMode = BiMode::PostS1ExactMd {
    r_max_cap: 50_000, ns_max: 2_100, md_max: 16_000, md_neg_cap: -18_000, ns_neg_max: 2_400,
};

struct ArmC55<const DEF_LIM: u32, const ADAPTIVE_L8: bool, const DICT_MODE: u8, const S4_MODE: u8, const S5_MODE: u8, const S1_GROW: u8> {
    open: engine_c55::Kernel<9, 1024, DICT_MODE, S4_MODE, S5_MODE, S1_GROW>,
    seg: InterleavedWalk,
    force_fallback: bool,
}

impl<const DEF_LIM: u32, const ADAPTIVE_L8: bool, const DICT_MODE: u8, const S4_MODE: u8, const S5_MODE: u8, const S1_GROW: u8> Default
    for ArmC55<DEF_LIM, ADAPTIVE_L8, DICT_MODE, S4_MODE, S5_MODE, S1_GROW>
{
    fn default() -> Self { Self { open: engine_c55::Kernel::default(), seg: InterleavedWalk::default(), force_fallback: false } }
}

impl<const DEF_LIM: u32, const ADAPTIVE_L8: bool, const DICT_MODE: u8, const S4_MODE: u8, const S5_MODE: u8, const S1_GROW: u8>
    ArmC55<DEF_LIM, ADAPTIVE_L8, DICT_MODE, S4_MODE, S5_MODE, S1_GROW>
{
    #[inline(never)]
    fn run_stats(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (u8, Option<Counts>) {
        if self.force_fallback {
            let st = self.open.try_run_const::<10>(records, n, p, Cap::NONE, BiMode::AlwaysAt(65_536), out).expect("forced fallback");
            return (0, Some(st));
        }
        if n < 32_768 { assert!(self.open.invert_tiny_fl(records, n, p, out)); return (1, None); }
        if records.len() * 64 < n && try_short_periodic(records, n, p, out) { return (2, None); }
        let rej_lim: u64 = if ADAPTIVE_L8 { 12 } else { u64::from(DEF_LIM) };
        let lhs = (records.len() as u64) * 4 * (rej_lim + 1);
        let rhs = (n as u64) * (3 * rej_lim + 4);
        if lhs >= rhs { self.seg.invert(records, n, p, out); return (3, None); }
        let (lim, _) = select_limit_with_def::<DEF_LIM, ADAPTIVE_L8>(records, n);
        let res = match lim {
            8  => self.open.try_run_const::<8>(records, n, p, Cap::C53_CROSSOVER, POST_S1_BI, out),
            10 => self.open.try_run_const::<10>(records, n, p, Cap::C53_CROSSOVER, POST_S1_BI, out),
            12 => self.open.try_run_const::<12>(records, n, p, Cap::C53_CROSSOVER, POST_S1_BI, out),
            16 => self.open.try_run_const::<16>(records, n, p, Cap::C53_CROSSOVER, POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, Cap::C53_CROSSOVER, POST_S1_BI, out),
            _  => self.open.try_run_const::<32>(records, n, p, Cap::C53_CROSSOVER, POST_S1_BI, out),
        };
        if let Some(st) = res {
            let rt = match (lim, st.used_bi) {
                (8, _) => 4,
                (10 | 12 | 16, true) => 0,
                (10 | 12 | 16, false) => 5,
                (24, _) => 6,
                _ => 7,
            };
            return (rt, Some(st));
        }
        self.seg.invert(records, n, p, out); (3, None)
    }
}

// ============================================================================
// 5. Classical Atlas Extension `C55-D6` (`CA-56` — Extending Chain D
//    `CA-26→27→31→37→41→44→47→50→53→CA-56` to `10` Pure-`CA` Nodes / `9` Links
//    AND Chain A+N+P+R+D4+D5 to a NEW `13`-Node / `12`-Link Pure-`CA` Record!):
//    A. M. Legendre (1787) Convex-Dual Rate Function & A. A. Markov (1906, 1913)
//    2nd-Order Chain Extension on `CA-53` + Exact Order-`k` Record-Length Invariance!
// ============================================================================

struct LnFactTable { tbl: Vec<f64> }
impl LnFactTable {
    fn new(max_n: usize) -> Self {
        let mut tbl = Vec::with_capacity(max_n + 1);
        tbl.push(0.0);
        let mut acc = 0.0f64;
        for k in 1..=max_n { acc += (k as f64).ln(); tbl.push(acc); }
        Self { tbl }
    }
    #[inline]
    fn get(&mut self, n: usize) -> f64 {
        if n >= self.tbl.len() {
            let mut acc = *self.tbl.last().unwrap();
            for k in self.tbl.len()..=n { acc += (k as f64).ln(); self.tbl.push(acc); }
        }
        self.tbl[n]
    }
}

fn jensen_maxent_ceilings(mu: f64, mu_log: f64, cap: usize) -> (f64, f64, f64) {
    let inv_ln2 = std::f64::consts::LOG2_E;
    let cf = cap as f64;
    if cap <= 1 || mu <= 1.0 + 1e-12 || mu >= cf - 1e-12 { return (0.0, 0.0, 0.0); }
    let ln_k: Vec<f64> = (1..=cap).map(|k| (k as f64).ln()).collect();
    let eval_geom = |theta: f64| -> (f64, f64) {
        let mut max_e = f64::NEG_INFINITY;
        for k in 1..=cap { max_e = max_e.max(-theta * k as f64); }
        let (mut z, mut ek) = (0.0, 0.0);
        for k in 1..=cap { let w = (-theta * k as f64 - max_e).exp(); z += w; ek += w * k as f64; }
        (max_e + z.ln() + theta * mu, ek / z)
    };
    let (mut lo, mut hi) = (-20.0f64, 20.0f64);
    for _ in 0..80 { let mid = 0.5 * (lo + hi); if eval_geom(mid).1 > mu { lo = mid; } else { hi = mid; } }
    let theta_star = 0.5 * (lo + hi);
    let h_geom = eval_geom(theta_star).0 * inv_ln2;

    let eval_zeta = |alpha: f64| -> (f64, f64) {
        let mut max_e = f64::NEG_INFINITY;
        for &lk in &ln_k { max_e = max_e.max(-alpha * lk); }
        let (mut z, mut el) = (0.0, 0.0);
        for &lk in &ln_k { let w = (-alpha * lk - max_e).exp(); z += w; el += w * lk; }
        (max_e + z.ln() + alpha * mu_log, el / z)
    };
    let (mut alo, mut ahi) = (-25.0f64, 25.0f64);
    for _ in 0..80 { let mid = 0.5 * (alo + ahi); if eval_zeta(mid).1 > mu_log { alo = mid; } else { ahi = mid; } }
    let alpha_star = 0.5 * (alo + ahi);
    let h_zeta = eval_zeta(alpha_star).0 * inv_ln2;

    let eval_joint = |th: f64, al: f64| -> (f64, f64, f64, f64, f64, f64) {
        let mut max_e = f64::NEG_INFINITY;
        for (i, &lk) in ln_k.iter().enumerate() { max_e = max_e.max(-th * (i + 1) as f64 - al * lk); }
        let (mut z, mut ek, mut el, mut ek2, mut ekl, mut el2) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for (i, &lk) in ln_k.iter().enumerate() {
            let kf = (i + 1) as f64;
            let w = (-th * kf - al * lk - max_e).exp();
            z += w; ek += w * kf; el += w * lk; ek2 += w * kf * kf; ekl += w * kf * lk; el2 += w * lk * lk;
        }
        let (mk, ml) = (ek / z, el / z);
        let obj = max_e + z.ln() + th * mu + al * mu_log;
        (obj, mu - mk, mu_log - ml, (ek2 / z - mk * mk).max(1e-12), ekl / z - mk * ml, (el2 / z - ml * ml).max(1e-12))
    };
    let (mut th, mut al) = if h_geom <= h_zeta { (theta_star, 0.0) } else { (0.0, alpha_star) };
    let mut best_obj = eval_joint(th, al).0;
    for _ in 0..40 {
        let (obj, g1, g2, h11, h12, h22) = eval_joint(th, al);
        best_obj = best_obj.min(obj);
        if g1.abs() < 1e-10 && g2.abs() < 1e-10 { break; }
        let det = h11 * h22 - h12 * h12;
        if det <= 1e-18 { break; }
        let (dth, dal) = (-(h22 * g1 - h12 * g2) / det, -(-h12 * g1 + h11 * g2) / det);
        let (mut step, mut imp) = (1.0, false);
        while step >= 1e-6 {
            let (nth, nal) = ((th + step * dth).clamp(-30.0, 30.0), (al + step * dal).clamp(-30.0, 30.0));
            let nobj = eval_joint(nth, nal).0;
            if nobj <= obj + 1e-13 { th = nth; al = nal; best_obj = best_obj.min(nobj); imp = true; break; }
            step *= 0.5;
        }
        if !imp { break; }
    }
    let h_joint = (best_obj * inv_ln2).min(h_geom).min(h_zeta);
    (h_geom, h_zeta, h_joint)
}

fn legendre_convex_dual_rate(row_a: &[(f64, f64)], e_bar: f64) -> (f64, f64, f64, f64, i8) {
    let inv_ln2 = std::f64::consts::LOG2_E;
    let eval_psi = |beta: f64| -> (f64, f64) {
        let eb = beta.exp();
        let (mut psi, mut mean_e) = (0.0, 0.0);
        for &(pi_i, q1_i) in row_a {
            if pi_i <= 0.0 { continue; }
            let q0_i = (1.0 - q1_i).max(0.0);
            let z = q0_i + q1_i * eb;
            if z > 0.0 { psi += pi_i * z.ln(); mean_e += pi_i * (q1_i * eb) / z; }
        }
        (psi, mean_e)
    };
    let (_, e0) = eval_psi(0.0);
    if (e_bar - e0).abs() < 1e-13 { return (0.0, 0.0, 0.0, 0.0, 0); }
    let (mut e_min, mut e_max) = (0.0, 0.0);
    for &(pi_i, q1_i) in row_a {
        if q1_i >= 1.0 - 1e-15 { e_min += pi_i; }
        if q1_i > 1e-15 { e_max += pi_i; }
    }
    if e_bar <= e_min + 1e-14 {
        let mut i_nats = 0.0;
        for &(pi_i, q1_i) in row_a {
            let q0_i = (1.0 - q1_i).max(0.0);
            if pi_i > 0.0 && q0_i > 0.0 { i_nats -= pi_i * q0_i.ln(); }
        }
        let bits = (i_nats * inv_ln2).max(0.0);
        return (-45.0, -i_nats, bits, bits, -1);
    }
    if e_bar >= e_max - 1e-14 {
        let mut i_nats = 0.0;
        for &(pi_i, q1_i) in row_a {
            if pi_i > 0.0 && q1_i > 0.0 { i_nats -= pi_i * q1_i.ln(); }
        }
        let bits = (i_nats * inv_ln2).max(0.0);
        return (45.0, -i_nats, bits, bits, 1);
    }
    let (mut lo, mut hi) = (-45.0f64, 45.0f64);
    for _ in 0..95 { let mid = 0.5 * (lo + hi); if eval_psi(mid).1 < e_bar { lo = mid; } else { hi = mid; } }
    let beta_star = 0.5 * (lo + hi);
    let (psi_star, _) = eval_psi(beta_star);
    let i_nats = (beta_star * e_bar - psi_star).max(0.0);
    let eb = beta_star.exp();
    let mut dkl_nats = 0.0;
    for &(pi_i, q1_i) in row_a {
        if pi_i <= 0.0 { continue; }
        let q0_i = (1.0 - q1_i).max(0.0);
        let z = q0_i + q1_i * eb;
        if z > 0.0 {
            let p0 = q0_i / z;
            let p1 = (q1_i * eb) / z;
            if p0 > 0.0 { dkl_nats += pi_i * p0 * (1.0 / z).ln(); }
            if p1 > 0.0 { dkl_nats += pi_i * p1 * (eb / z).ln(); }
        }
    }
    (beta_star, psi_star, i_nats * inv_ln2, dkl_nats.max(0.0) * inv_ln2, 0)
}

#[derive(Default, Clone)]
struct D6Result {
    h_geom: f64,
    h_zeta: f64,
    h_joint: f64,
    h_marg: f64,
    h_gibbs_tt: f64,
    h_1: f64,
    h_2: f64,
    w2_boltz_rate: f64,
    i_1: f64,
    i_2: f64,
    i_leg_tt: f64,
    i_leg_ct: f64,
    max_leg_err: f64,
    max_pyth_err: f64,
    max_biconj_err: f64,
    max_pf2_err: f64,
    m_act: i128,
    m_min_lin: i128,
    m_max: i128,
    wrap_delta: i128,
}

fn verify_c55_d6_on_records(
    records: &[u16], p: u32, lim: u32, cap: u32, check_maxent: bool, lnf: &mut LnFactTable,
) -> D6Result {
    let r = records.len();
    assert!(r >= 1);
    let c = cap as usize;
    let mut row_cnt = vec![0usize; c + 1];
    let mut col_cnt = vec![0usize; c + 1];
    let mut trans = vec![0usize; (c + 1) * (c + 1)];
    let mut triples_raw: Vec<u32> = Vec::with_capacity(r);
    let (mut c_cap, mut s1, mut s_log, mut e_tt_num, mut e_ct_num) = (0usize, 0u64, 0.0f64, 0usize, 0usize);
    let mut thin_ws = Vec::new();

    for idx in 0..r {
        let u = usize::from(records[idx] & 255);
        let v = usize::from(records[(idx + 1) % r] & 255);
        let w = usize::from(records[(idx + 2) % r] & 255);
        assert!(u >= 1 && u <= c && v >= 1 && v <= c && w >= 1 && w <= c);
        row_cnt[u] += 1;
        col_cnt[v] += 1;
        trans[u * (c + 1) + v] += 1;
        triples_raw.push(((u as u32) << 16) | ((v as u32) << 8) | (w as u32));
        s1 += u as u64;
        s_log += (u as f64).ln();
        if u == c { c_cap += 1; }
        if u <= lim as usize { thin_ws.push(u as u32); }
        if (u > lim as usize) != (v > lim as usize) { e_tt_num += 1; }
        if u == c && v <= lim as usize { e_ct_num += 1; }
    }
    assert_eq!(row_cnt, col_cnt);

    let rf = r as f64;
    let inv_ln2 = std::f64::consts::LOG2_E;
    let (mut h_marg, mut h_1, mut log2_w1) = (0.0f64, 0.0f64, 0.0f64);
    for u in 1..=c {
        let nu = row_cnt[u];
        if nu == 0 { continue; }
        let piu = nu as f64 / rf;
        h_marg -= piu * piu.log2();
        log2_w1 += lnf.get(nu) * inv_ln2;
        for v in 1..=c {
            let nuv = trans[u * (c + 1) + v];
            if nuv > 0 {
                let puv = nuv as f64 / nu as f64;
                h_1 -= piu * puv * puv.log2();
                log2_w1 -= lnf.get(nuv) * inv_ln2;
            }
        }
    }
    let i_1 = (h_marg - h_1).max(0.0);
    assert!(log2_w1 / rf <= h_1 + 1e-9 && h_1 <= h_marg + 1e-10);

    let p_thin = thin_ws.len() as f64 / rf;
    let p_thick = 1.0 - p_thin;
    let e_tt_bar = e_tt_num as f64 / rf;
    let (b_tt, psi_tt, i_leg_tt, dkl_tt, bnd_tt) = legendre_convex_dual_rate(&[(p_thin, p_thick), (p_thick, p_thin)], e_tt_bar);
    let h_gibbs_tt = h_marg - i_leg_tt;

    let p_cap = c_cap as f64 / rf;
    let e_ct_bar = e_ct_num as f64 / rf;
    let (b_ct, psi_ct, i_leg_ct, dkl_ct, bnd_ct) = legendre_convex_dual_rate(&[(p_cap, p_thin), (1.0 - p_cap, 0.0)], e_ct_bar);
    let h_gibbs_ct = h_marg - i_leg_ct;

    let max_leg_err = (i_leg_tt - dkl_tt).abs().max((i_leg_ct - dkl_ct).abs());
    let mut max_biconj_err = 0.0f64;
    if bnd_tt == 0 { max_biconj_err = max_biconj_err.max((psi_tt - (b_tt * e_tt_bar - i_leg_tt / inv_ln2)).abs()); }
    if bnd_ct == 0 { max_biconj_err = max_biconj_err.max((psi_ct - (b_ct * e_ct_bar - i_leg_ct / inv_ln2)).abs()); }
    assert!(max_leg_err < 1e-10 && max_biconj_err < 1e-11);

    let check_pyth = |beta: f64, bnd: i8, is_ct: bool, i_leg_bits: f64| -> f64 {
        let eb = beta.exp();
        let mut dkl_p_tilt_nats = 0.0;
        for u in 1..=c {
            let nu = row_cnt[u];
            if nu == 0 { continue; }
            let piu = nu as f64 / rf;
            let q1 = if is_ct { if u == c { p_thin } else { 0.0 } } else if u <= lim as usize { p_thick } else { p_thin };
            let q0 = (1.0 - q1).max(0.0);
            let z = q0 + q1 * eb;
            for v in 1..=c {
                let nuv = trans[u * (c + 1) + v];
                if nuv == 0 { continue; }
                let puv = nuv as f64 / nu as f64;
                let piv = row_cnt[v] as f64 / rf;
                let e_uv = if is_ct { u == c && v <= lim as usize } else { (u > lim as usize) != (v > lim as usize) };
                let ptilt = if bnd == -1 { assert!(!e_uv); piv / q0 }
                    else if bnd == 1 { if q1 > 1e-15 { assert!(e_uv); piv / q1 } else { piv } }
                    else { piv * (if e_uv { eb } else { 1.0 }) / z };
                dkl_p_tilt_nats += piu * puv * (puv / ptilt).ln();
            }
        }
        let dkl_p_tilt_bits = dkl_p_tilt_nats * inv_ln2;
        assert!(dkl_p_tilt_bits >= -1e-9);
        (i_1 - (dkl_p_tilt_bits + i_leg_bits)).abs()
    };
    let max_pyth_err = check_pyth(b_tt, bnd_tt, false, i_leg_tt).max(check_pyth(b_ct, bnd_ct, true, i_leg_ct));
    assert!(max_pyth_err < 1e-9 && h_1 <= h_gibbs_tt + 1e-8 && h_1 <= h_gibbs_ct + 1e-8);

    triples_raw.sort_unstable();
    let mut pair_out = vec![0usize; (c + 1) * (c + 1)];
    let mut pair_in = vec![0usize; (c + 1) * (c + 1)];
    let mut pf2_col = vec![0.0f64; (c + 1) * (c + 1)];
    let (mut h_2, mut log2_w2) = (0.0f64, 0.0f64);
    for &nuv in &trans { if nuv > 0 { log2_w2 += lnf.get(nuv) * inv_ln2; } }
    let mut ti = 0usize;
    while ti < triples_raw.len() {
        let key = triples_raw[ti];
        let mut tj = ti + 1;
        while tj < triples_raw.len() && triples_raw[tj] == key { tj += 1; }
        let cnt = tj - ti;
        let (u, v, w) = (((key >> 16) & 0xFF) as usize, ((key >> 8) & 0xFF) as usize, (key & 0xFF) as usize);
        let nuv = trans[u * (c + 1) + v];
        pair_out[u * (c + 1) + v] += cnt;
        pair_in[v * (c + 1) + w] += cnt;
        let p_w_uv = cnt as f64 / nuv as f64;
        h_2 -= (cnt as f64 / rf) * p_w_uv.log2();
        log2_w2 -= lnf.get(cnt) * inv_ln2;
        pf2_col[v * (c + 1) + w] += (nuv as f64 / rf) * p_w_uv;
        ti = tj;
    }
    assert_eq!(pair_out, trans);
    assert_eq!(pair_in, trans);
    let mut max_pf2_err = 0.0f64;
    for (idx, &nuv) in trans.iter().enumerate() { max_pf2_err = max_pf2_err.max((pf2_col[idx] - nuv as f64 / rf).abs()); }
    assert!(max_pf2_err < 1e-13);
    let w2_boltz_rate = log2_w2 / rf;
    let i_2 = (h_1 - h_2).max(0.0);
    assert!(w2_boltz_rate <= h_2 + 1e-9 && h_2 <= h_1 + 1e-10);

    thin_ws.sort_unstable_by(|a, b| b.cmp(a));
    let m_rec: i128 = thin_ws.iter().map(|&w| i128::from(w)).sum();
    let top_sum: i128 = thin_ws.iter().take(c_cap).map(|&w| i128::from(w)).sum();
    let mut adj_abs_cyc: i128 = 0;
    for v in 1..=(lim as usize).min(c) { adj_abs_cyc += (v as i128) * (trans[c * (c + 1) + v] as i128); }
    let wrap_abs: i128 = if usize::from(records[r - 1] & 255) == c && usize::from(records[0] & 255) <= lim as usize {
        i128::from(records[0] & 255)
    } else { 0 };
    let adj_abs_lin = adj_abs_cyc - wrap_abs;
    assert!(adj_abs_lin <= adj_abs_cyc && adj_abs_cyc <= top_sum);
    let has_thick = thin_ws.len() < r;
    let m_min_marg = (m_rec - top_sum).max(1);
    let m_min_cyc = (m_rec - adj_abs_cyc).max(1);
    let m_min_lin = (m_rec - adj_abs_lin).max(1);
    let m_max = m_rec + if has_thick { 1 } else { 0 };
    let wrap_delta = m_min_lin - m_min_cyc;
    assert!(wrap_delta >= 0 && wrap_delta <= i128::from(lim));

    let (mut pos, mut i, mut m_act) = (0u32, 0usize, 0i128);
    while i < r {
        let rec = records[i]; let sym = (rec >> 8) as u8; let mut w = u32::from(rec & 255); i += 1;
        if w == cap { while i < r && (records[i] >> 8) as u8 == sym { w += u32::from(records[i] & 255); i += 1; } }
        if w <= lim { m_act += i128::from(w); }
        else if p >= pos && p < pos + w { m_act += 1; }
        pos += w;
    }
    assert!(m_min_marg <= m_min_cyc && m_min_cyc <= m_min_lin && m_min_lin <= m_act && m_act <= m_max);

    let (h_geom, h_zeta, h_joint) = if check_maxent {
        let (hg, hz, hj) = jensen_maxent_ceilings(s1 as f64 / rf, s_log / rf, c);
        assert!(h_marg <= hg + 1e-6 && h_marg <= hz + 1e-6 && h_marg <= hj + 1e-5);
        (hg, hz, hj)
    } else { (0.0, 0.0, 0.0) };

    D6Result {
        h_geom, h_zeta, h_joint, h_marg, h_gibbs_tt, h_1, h_2, w2_boltz_rate,
        i_1, i_2, i_leg_tt, i_leg_ct, max_leg_err, max_pyth_err, max_biconj_err, max_pf2_err,
        m_act, m_min_lin, m_max, wrap_delta,
    }
}

fn triple_signature_cyc(seq: &[usize]) -> Vec<(usize, usize, usize, usize)> {
    let r = seq.len();
    let mut ts: Vec<(usize, usize, usize)> = (0..r).map(|i| (seq[i], seq[(i + 1) % r], seq[(i + 2) % r])).collect();
    ts.sort_unstable();
    let mut out = Vec::new();
    let mut i = 0;
    while i < ts.len() { let mut j = i + 1; while j < ts.len() && ts[j] == ts[i] { j += 1; } out.push((ts[i].0, ts[i].1, ts[i].2, j - i)); i = j; }
    out
}

fn triple_signature_lin(seq: &[usize]) -> Vec<(usize, usize, usize, usize)> {
    if seq.len() < 3 { return Vec::new(); }
    let mut ts: Vec<(usize, usize, usize)> = (0..seq.len() - 2).map(|i| (seq[i], seq[i + 1], seq[i + 2])).collect();
    ts.sort_unstable();
    let mut out = Vec::new();
    let mut i = 0;
    while i < ts.len() { let mut j = i + 1; while j < ts.len() && ts[j] == ts[i] { j += 1; } out.push((ts[i].0, ts[i].1, ts[i].2, j - i)); i = j; }
    out
}

fn run_c55_d6_legendre_markov2_audit(six: &[Block], wiki431: &[Block], non_wiki: &[Block], code_docs: &[Block], dense: &Block) {
    println!("\n=== ATLAS EXTENSION C55-D6 (`CA-56`: LEGENDRE 1787 CONVEX-DUAL RATE & MARKOV 1906/1913 2ND-ORDER CHAIN) ===");
    println!("=== EXTENDING CHAIN D (`CA-26→27→31→37→41→44→47→50→53→CA-56`, 10 PURE-CA NODES / 9 LINKS!) ===");
    println!("=== AND CHAIN A+N+P+R+D4+D5 (`CA-05→…→CA-46→CA-50→CA-53→CA-56`, NEW 13-NODE / 12-LINK PURE-CA RECORD!) ===");

    let mut lnf = LnFactTable::new(4096);

    println!("  [Theorem D6-2A: Element-for-Element Identical Record Sequences `x1==x2` (All Linear & Cyclic Order-k Tensors Equal), 100% `m` Gap]");
    for &(cap, lim, q) in &[(15u32, 4u32, 5usize), (31, 8, 8), (127, 16, 20), (127, 16, 50)] {
        let mut t1 = vec![0u8];
        for j in (1..=2 * q).rev() {
            let w = if j % 2 == 1 { cap as usize } else { lim as usize };
            t1.extend(std::iter::repeat_n(j as u8, w));
        }
        let pos1 = t1.iter().position(|&b| b == 1).unwrap() + 1;
        let mut t1_rot = Vec::with_capacity(t1.len());
        t1_rot.extend_from_slice(&t1[pos1..]); t1_rot.extend_from_slice(&t1[..pos1]);
        let (l1, p1) = bwt_small(&t1_rot);
        assert!(is_single_cycle_bwt(&l1) && plain_invert_fl(&l1, p1) == t1_rot);

        let mut t2 = vec![0u8];
        for j in (1..=q).rev() { t2.extend(std::iter::repeat_n(j as u8, (cap + lim) as usize)); }
        let mut t2_rot = Vec::with_capacity(t2.len());
        t2_rot.extend_from_slice(&t2[1..]); t2_rot.push(t2[0]);
        let (l2, p2) = bwt_small(&t2_rot);
        assert!(is_single_cycle_bwt(&l2) && plain_invert_fl(&l2, p2) == t2_rot);

        let (r1, r2) = (records_of_cap(&l1, cap), records_of_cap(&l2, cap));
        let x1: Vec<usize> = r1.iter().map(|&x| usize::from(x & 255)).collect();
        let x2: Vec<usize> = r2.iter().map(|&x| usize::from(x & 255)).collect();
        assert_eq!(x1, x2);
        assert_eq!(triple_signature_cyc(&x1), triple_signature_cyc(&x2));
        assert_eq!(triple_signature_lin(&x1), triple_signature_lin(&x2));

        let d1 = verify_c55_d6_on_records(&r1, p1, lim, cap, true, &mut lnf);
        let d2 = verify_c55_d6_on_records(&r2, p2, lim, cap, true, &mut lnf);
        assert!((d1.h_2 - d2.h_2).abs() < 1e-12 && (d1.i_2 - d2.i_2).abs() < 1e-12 && (d1.i_leg_tt - d2.i_leg_tt).abs() < 1e-12);
        assert_eq!(d1.m_act, d1.m_max);
        assert_eq!(d2.m_act, d2.m_min_lin);
        println!(
            "    (C={:3}, L={:2}, q={:2}, n={:4}): x1==x2 EXACT (lin+cyc N^(2)_X eq!) | H_2={:.3} H_1={:.3} I_2={:.3}b I_LegTT={:.3}b | m(B1)={:3}==m_max vs m(B2)={:1}==m_min_lin ({:5.1}x, 100.0%!)",
            cap, lim, q, l1.len(), d1.h_2, d1.h_1, d1.i_2, d1.i_leg_tt, d1.m_act, d2.m_act, d1.m_act as f64 / d2.m_act as f64
        );
    }

    println!("  [Theorem D6-2B: Element-for-Element `wa==wb` & `xa==xb` (All Linear & Cyclic Order-k Tensors Equal) & `m=2`, Divergent `S2b` Grammar]");
    let mut k_diag = engine_c55::Kernel::<9, 1024, 2, 1, 2, 1>::default();
    let mut out_d = Vec::new();
    for &(fp, fq) in &[(55usize, 89usize), (89, 144), (144, 233), (233, 377)] {
        let mut cw = Vec::with_capacity(fp + fq);
        let mut y = 0usize;
        for i in 0..(fp + fq) { if ((i + 1) * fq) / (fp + fq) > y { cw.push(2u8); y += 1; } else { cw.push(1u8); } }
        let min_r = (0..cw.len()).min_by(|&a, &b| {
            for k in 0..cw.len() { let (ca, cb) = (cw[(a + k) % cw.len()], cw[(b + k) % cw.len()]); if ca != cb { return ca.cmp(&cb); } }
            a.cmp(&b)
        }).unwrap();
        let mut tb = Vec::with_capacity(1 + cw.len());
        tb.push(0u8); tb.extend_from_slice(&cw[min_r..]); tb.extend_from_slice(&cw[..min_r]);
        let (lb, pb) = bwt_small(&tb);
        assert!(is_single_cycle_bwt(&lb) && plain_invert_fl(&lb, pb) == tb);

        let mut ta = Vec::with_capacity(tb.len());
        ta.push(0u8);
        ta.extend(std::iter::repeat_n(1u8, fq - 1));
        ta.extend(std::iter::repeat_n(2u8, fp));
        ta.push(3u8);
        let (la, pa) = bwt_small(&ta);
        assert!(is_single_cycle_bwt(&la) && plain_invert_fl(&la, pa) == ta);

        let merged_runs = |l: &[u8]| -> Vec<usize> {
            let (mut v, mut i) = (Vec::new(), 0usize);
            while i < l.len() { let b = l[i]; let mut w = 0; while i < l.len() && l[i] == b { i += 1; w += 1; } v.push(w); }
            v
        };
        let (wa, wb) = (merged_runs(&la), merged_runs(&lb));
        assert_eq!(wa, wb);
        let (ra, rb) = (records_of(&la), records_of(&lb));
        let xa: Vec<usize> = ra.iter().map(|&r| usize::from(r & 255)).collect();
        let xb: Vec<usize> = rb.iter().map(|&r| usize::from(r & 255)).collect();
        assert_eq!(xa, xb);
        assert_eq!(triple_signature_lin(&wa), triple_signature_lin(&wb));
        assert_eq!(triple_signature_cyc(&wa), triple_signature_cyc(&wb));

        let da = verify_c55_d6_on_records(&ra, pa, 16, 127, true, &mut lnf);
        let db = verify_c55_d6_on_records(&rb, pb, 16, 127, true, &mut lnf);
        assert!((da.h_2 - db.h_2).abs() < 1e-12 && (da.i_2 - db.i_2).abs() < 1e-12);

        let st_a = k_diag.try_run_const::<16>(&ra, la.len(), pa, Cap::NONE, BiMode::AlwaysAt(0), &mut out_d).unwrap();
        assert_eq!(out_d, ta);
        let st_b = k_diag.try_run_const::<16>(&rb, lb.len(), pb, Cap::NONE, BiMode::AlwaysAt(0), &mut out_d).unwrap();
        assert_eq!(out_d, tb);
        assert_eq!(st_a.m, st_b.m);
        println!(
            "    Fib({:3},{:3}) n={:3}: wa==wb & xa==xb EXACT (lin+cyc N^(2) eq!) & m={} | Clustered(A): steps={:3}, rw={:4} vs Christoffel(B): steps={:2} ({:4.1}x), rw={:2} ({:4.1}x)!",
            fp, fq, la.len(), st_a.m, st_a.s2b_steps, st_a.rule_words, st_b.s2b_steps,
            st_a.s2b_steps as f64 / st_b.s2b_steps.max(1) as f64, st_b.rule_words,
            st_a.rule_words as f64 / st_b.rule_words.max(1) as f64
        );
    }

    let (mut max_leg_e, mut max_pyth_e, mut max_biconj_e, mut max_pf2_e, mut max_wrap) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0i128);
    {
        let det: Vec<u16> = [1u16, 2, 3, 1, 2, 3].iter().enumerate().map(|(i, &w)| ((i as u16 + 1) << 8) | w).collect();
        let d_det = verify_c55_d6_on_records(&det, 0, 2, 5, true, &mut lnf);
        assert!(d_det.h_2.abs() < 1e-12 && d_det.h_1.abs() < 1e-12 && d_det.i_2.abs() < 1e-12);
        let db3: Vec<u16> = [1u16, 1, 1, 2, 1, 2, 2, 2].iter().enumerate().map(|(i, &w)| ((i as u16 + 1) << 8) | w).collect();
        let d_db3 = verify_c55_d6_on_records(&db3, 0, 1, 5, true, &mut lnf);
        assert!(d_db3.i_1.abs() < 1e-12 && d_db3.i_2.abs() < 1e-12 && (d_db3.h_2 - 1.0).abs() < 1e-12);
        let db2: Vec<u16> = [1u16, 1, 2, 2].iter().enumerate().map(|(i, &w)| ((i as u16 + 1) << 8) | w).collect();
        let d_db2 = verify_c55_d6_on_records(&db2, 0, 1, 5, true, &mut lnf);
        assert!(d_db2.i_1.abs() < 1e-12 && d_db2.h_2.abs() < 1e-12 && (d_db2.i_2 - 1.0).abs() < 1e-12);
    }
    let caps = [5u32, 7, 8, 15, 16, 31, 63, 127];
    let mut rng = 0xC55D_6178_7190_6191u64;
    let mut next_u64 = || -> u64 { rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); rng };
    let mut recs_buf = Vec::with_capacity(512);
    let mut maxent_chk = 0usize;
    for case in 0..105_000usize {
        let c = caps[case % caps.len()];
        let lim = 1 + ((next_u64() >> 32) as u32 % (c - 1));
        let r = 3 + ((next_u64() >> 40) as usize % 36);
        let mode = case % 7;
        recs_buf.clear();
        let (mut sym, mut tot) = (1u8, 0u32);
        for j in 0..r {
            let w: u32 = match mode {
                0 => 1 + ((next_u64() >> 32) as u32 % (3 * c)),
                1 => { let ch = [1, 2, lim, lim + 1, c - 1, c, c + 1, 2 * c + 1]; ch[((next_u64() >> 32) as usize) % ch.len()] }
                2 => if j < r / 2 { 1 } else { c + lim },
                3 => if j % 2 == 0 { 1 } else { c + lim },
                4 => if j % 3 == 0 { c + 1 + ((next_u64() >> 32) as u32 % lim) } else { 1 + ((next_u64() >> 32) as u32 % lim) },
                5 => if j % 2 == 0 { c } else { 1 + ((next_u64() >> 32) as u32 % c) },
                _ => 1 + ((next_u64() >> 32) as u32 % (lim + 3).min(c)),
            };
            tot += w;
            let mut left = w;
            while left > c { recs_buf.push((u16::from(sym) << 8) | (c as u16)); left -= c; }
            recs_buf.push((u16::from(sym) << 8) | (left as u16));
            sym = sym.wrapping_add(1); if sym == 255 { sym = 1; }
        }
        let p = ((next_u64() >> 32) as u32) % tot;
        let chk_e = case < 2_000;
        let d = verify_c55_d6_on_records(&recs_buf, p, lim, c, chk_e, &mut lnf);
        max_leg_e = max_leg_e.max(d.max_leg_err);
        max_pyth_e = max_pyth_e.max(d.max_pyth_err);
        max_biconj_e = max_biconj_e.max(d.max_biconj_err);
        max_pf2_e = max_pf2_e.max(d.max_pf2_err);
        max_wrap = max_wrap.max(d.wrap_delta);
        if chk_e { maxent_chk += 1; }
    }
    println!(
        "  Exact Stress PASS: 105,000 cyclic chains + {maxent_chk} MaxEnt (0 violations!) | max_err: Leg={:.1e} Pyth={:.1e} Biconj={:.1e} PF2={:.1e} | max_wrap_delta={}",
        max_leg_e, max_pyth_e, max_biconj_e, max_pf2_e, max_wrap
    );

    let w233: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();
    let w198: Vec<&Block> = wiki431.iter().filter(|b| b.route_class >= 2).collect();
    let w431_r: Vec<&Block> = wiki431.iter().collect();
    let six_r: Vec<&Block> = six.iter().collect();
    let nw_r: Vec<&Block> = non_wiki.iter().collect();
    let cd_r: Vec<&Block> = code_docs.iter().collect();
    let dense_r = vec![dense];

    println!("  Corpus         | N   | W_2   / H_2   / H_1   / H_Gtt / H_m   / H_Jnt / H_Zt  / H_Gm (b) | I_1   | I_2   | ILegTT | ILegCT | WrapD | LinGap[L16/L12/L10]");
    for (name, blks) in [
        ("6 Admitted", &six_r), ("6 JSONL/Log", &nw_r), ("4 Code/Docs", &cd_r),
        ("233 FullWiki", &w233), ("198 TinyWiki", &w198), ("431 AllWiki", &w431_r), ("dense-300k", &dense_r),
    ] {
        let (mut w2, mut h2, mut h1, mut hgtt, mut hm, mut hj, mut hz, mut hgm) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let (mut i1, mut i2, mut iltt, mut ilct, mut g16, mut g12, mut g10) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut wrap_hits = 0usize;
        let mut cnt = 0usize;
        for &b in blks {
            let d16 = verify_c55_d6_on_records(&b.records, b.p, 16, 127, true, &mut lnf);
            let d12 = verify_c55_d6_on_records(&b.records, b.p, 12, 127, false, &mut lnf);
            let d10 = verify_c55_d6_on_records(&b.records, b.p, 10, 127, false, &mut lnf);
            w2 += d16.w2_boltz_rate; h2 += d16.h_2; h1 += d16.h_1; hgtt += d16.h_gibbs_tt;
            hm += d16.h_marg; hj += d16.h_joint; hz += d16.h_zeta; hgm += d16.h_geom;
            i1 += d16.i_1; i2 += d16.i_2; iltt += d16.i_leg_tt; ilct += d16.i_leg_ct;
            if d16.wrap_delta > 0 { wrap_hits += 1; }
            g16 += (d16.m_max - d16.m_min_lin) as f64 / d16.m_act.max(1) as f64;
            g12 += (d12.m_max - d12.m_min_lin) as f64 / d12.m_act.max(1) as f64;
            g10 += (d10.m_max - d10.m_min_lin) as f64 / d10.m_act.max(1) as f64;
            cnt += 1;
        }
        let c = cnt.max(1) as f64;
        println!(
            "  {:14} | {:3} | {:.3}/{:.3}/{:.3}/{:.3}/{:.3}/{:.3}/{:.3}/{:.3} | {:.3} | {:.3} | {:.4} | {:.4} | {:5} | {:5.2}%/{:5.2}%/{:5.2}%",
            name, blks.len(), w2 / c, h2 / c, h1 / c, hgtt / c, hm / c, hj / c, hz / c, hgm / c,
            i1 / c, i2 / c, iltt / c, ilct / c, wrap_hits, (g16 / c) * 100.0, (g12 / c) * 100.0, (g10 / c) * 100.0
        );
    }
}

// ============================================================================
// 6. Arm Suite, Exactness Gate, WS Attribution & Headroom Oracle v10 (`8` Arms)
// ============================================================================

const NUM_ARMS: usize = 8;
const ARM_NAMES: [&str; NUM_ARMS] = [
    "R5-Head-A1",
    "R5-Head-A2(AA)",
    "C54-PipeLean-v2",
    "C54-DramLean-v1",
    "Abl-Fused-L12",
    "Abl-Pow2-L12",
    "Abl-Pow2-L10",
    "C55-DramUltra-v2",
];

#[derive(Default)]
struct ArmSuite {
    kh1: cand_r5_head::Kernel,
    kh2: cand_r5_head::Kernel,
    c54_pl2: ArmC55<16, false, 0, 3, 3, 1>,
    c54_dl1: ArmC55<12, false, 2, 1, 2, 1>,
    abl_f12: ArmC55<12, false, 2, 4, 3, 2>,
    abl_p12: ArmC55<12, false, 2, 4, 4, 2>,
    abl_p10: ArmC55<10, false, 2, 4, 4, 2>,
    c55_du2: ArmC55<10, true,  2, 4, 4, 2>,
}

impl ArmSuite {
    #[inline(always)]
    fn run_arm(&mut self, arm: usize, b: &Block, out: &mut Vec<u8>) -> u8 {
        match arm {
            0 => { self.kh1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out); 0 }
            1 => { self.kh2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out); 0 }
            2 => self.c54_pl2.run_stats(&b.records, b.n, b.p, out).0,
            3 => self.c54_dl1.run_stats(&b.records, b.n, b.p, out).0,
            4 => self.abl_f12.run_stats(&b.records, b.n, b.p, out).0,
            5 => self.abl_p12.run_stats(&b.records, b.n, b.p, out).0,
            6 => self.abl_p10.run_stats(&b.records, b.n, b.p, out).0,
            _ => self.c55_du2.run_stats(&b.records, b.n, b.p, out).0,
        }
    }
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[(v.len() * num) / den]
}

fn run_proxy_arm(arm_name: &str, wiki431: &[Block]) {
    const PROXY_IDS: [u32; 10] = [34, 108, 178, 254, 327, 391, 60, 163, 265, 372];
    let warm = wiki431.iter().find(|b| b.idx == 1).unwrap();
    let blks: Vec<&Block> = PROXY_IDS.iter().map(|&id| wiki431.iter().find(|b| b.idx == id).unwrap()).collect();
    let arm_idx = match arm_name {
        "pl2" => 2, "dl1" => 3, "fused12" => 4, "pow2_12" => 5, "pow2_10" => 6, "dramultra" => 7,
        _ => panic!("unknown proxy arm: {arm_name}"),
    };
    let mut suite = ArmSuite::default();
    let mut out = Vec::with_capacity(900_064);
    suite.run_arm(arm_idx, warm, &mut out);
    let mut chk = 0u64;
    for &b in &blks {
        suite.run_arm(arm_idx, b, &mut out);
        chk = chk.wrapping_add(u64::from(out[0]) ^ u64::from(*out.last().unwrap()));
    }
    println!("proxy_arm={arm_name} chk={chk}");
}

fn run_exactness_and_ws(six: &[Block], wiki431: &[Block], non_wiki: &[Block], code_docs: &[Block], dense: &Block) {
    let mut kh = cand_r5_head::Kernel::default();
    let mut suite = ArmSuite::default();
    let mut out_ref = Vec::with_capacity(900_064);
    let mut out_k = Vec::with_capacity(900_064);

    let mut valid_cnt = 0usize;
    let mut text = Vec::with_capacity(512);
    for n in 1..=12usize {
        for mask in 0..(1usize << n) {
            text.clear();
            for bit in 0..n { text.push(b'a' + ((mask >> bit) & 1) as u8); }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            out_k.clear(); out_k.resize(n + 64, 0xAA);
            suite.c55_du2.open.try_run_const::<10>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
            assert_eq!(out_k, text);
            out_k.clear(); out_k.resize(n + 64, 0x55);
            suite.c55_du2.run_stats(&recs, n, p, &mut out_k);
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
        suite.c55_du2.open.try_run_const::<8>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, text);
        suite.c55_du2.open.try_run_const::<10>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, text);
        suite.c55_du2.open.try_run_const::<32>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(u32::MAX), &mut out_k).unwrap();
        assert_eq!(out_k, text);
        suite.c55_du2.run_stats(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        valid_cnt += 1;
    }

    {
        let big = vec![0u16; 16_000_001];
        assert!(!suite.c55_du2.open.invert_tiny_fl(&big, 100, 0, &mut out_k));
        assert!(suite.c55_du2.open.try_run_const::<10>(&big, 100, 0, Cap::NONE, POST_S1_BI, &mut out_k).is_none());

        let mut l_wide = Vec::with_capacity(760_000);
        let mut sr = 0x5EED_C550_4095_BEEFu64;
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
        let st_w = suite.c55_du2.open.try_run_const::<10>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, want_w);
        assert!(st_w.m >= (1 << 19) && st_w.bits == 20 && st_w.wide && st_w.used_u32_fallback);

        suite.c55_du2.open.force_u16_cap = Some(300);
        let st_u16t = suite.c55_du2.open.try_run_const::<10>(&six[0].records, six[0].n, six[0].p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert!(st_u16t.used_u32_fallback);
        assert_eq!(&out_k, six[0].expected.as_ref().unwrap());
        suite.c55_du2.open.force_u16_cap = None;

        suite.c55_du2.open.force_sentinel_cap = Some(260);
        let st_fb = suite.c55_du2.open.try_run_const::<10>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert!(st_fb.wide && st_fb.used_bi); assert_eq!(out_k, want_w);
        suite.c55_du2.open.force_band_vec = true;
        let st_fv = suite.c55_du2.open.try_run_const::<10>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert!(st_fv.wide && st_fv.used_bi && !st_fv.used_tail); assert_eq!(out_k, want_w);
        let st_fu = suite.c55_du2.open.try_run_const::<10>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(u32::MAX), &mut out_k).unwrap();
        assert!(st_fu.wide && !st_fu.used_bi); assert_eq!(out_k, want_w);
        suite.c55_du2.open.force_band_vec = false;
        suite.c55_du2.open.force_sentinel_cap = None;

        println!(
            "\nSENTINEL + U16-CONTENTS + POW2 FALLBACK STRESS PASS: natural wide m={} bits={} num_codes={} u32_fb={} + forced u16_cap=300 + forced sentinel=260 100% BYTE-EXACT!",
            st_w.m, st_w.bits, st_w.num_codes, st_w.used_u32_fallback
        );
    }

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
        suite.c55_du2.run_stats(&b.records, b.n, b.p, &mut p_aa); assert_eq!(&p_aa, want);
        let mut fresh = Vec::new();
        suite.c55_du2.run_stats(&b.records, b.n, b.p, &mut fresh); assert_eq!(&fresh, want);
        suite.c55_du2.open.force_band_vec = true;
        let st_fb = suite.c55_du2.open.try_run_const::<10>(&b.records, b.n, b.p, Cap::NONE, BiMode::AlwaysAt(65_536), &mut out_k).unwrap();
        assert!(!st_fb.used_tail); assert_eq!(&out_k, want);
        suite.c55_du2.open.force_band_vec = false;
        suite.c55_du2.force_fallback = true;
        assert_eq!(suite.c55_du2.run_stats(&b.records, b.n, b.p, &mut out_k).0, 0); assert_eq!(&out_k, want);
        suite.c55_du2.force_fallback = false;
        ot_poison += 4;
    }

    let (mut max_codes_447, mut u32_fb_hits_447, mut seg_fb_447) = (0usize, 0usize, 0usize);
    for (idx, b) in wiki431.iter().enumerate() {
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        for arm in 2..NUM_ARMS {
            suite.run_arm(arm, b, &mut out_k);
            assert_eq!(out_k, out_ref, "wiki arm={} idx={idx}", ARM_NAMES[arm]);
        }
        let (rt, st_opt) = suite.c55_du2.run_stats(&b.records, b.n, b.p, &mut out_k);
        if rt == 3 { seg_fb_447 += 1; }
        if let Some(st) = st_opt {
            max_codes_447 = max_codes_447.max(st.num_codes);
            if st.used_u32_fallback { u32_fb_hits_447 += 1; }
        }
    }
    for b in six.iter().chain(non_wiki.iter()).chain(code_docs.iter()) {
        let (rt, st_opt) = suite.c55_du2.run_stats(&b.records, b.n, b.p, &mut out_k);
        if rt == 3 { seg_fb_447 += 1; }
        if let Some(st) = st_opt {
            max_codes_447 = max_codes_447.max(st.num_codes);
            if st.used_u32_fallback { u32_fb_hits_447 += 1; }
        }
    }

    let mut mal_cnt = 0usize;
    let mut check_mal = |l: &[u8], p: u32, tag: &str| {
        let n = l.len();
        let recs = records_of(l);
        kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        let mut pbuf = vec![0xAAu8; n + 64];
        suite.c55_du2.open.try_run_const::<10>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut pbuf).unwrap();
        assert_eq!(pbuf, out_ref, "malformed C55 L=10 bi-pow2 mismatch {tag}");
        pbuf.clear(); pbuf.resize(n + 64, 0x55);
        suite.c55_du2.run_stats(&recs, n, p, &mut pbuf);
        assert_eq!(pbuf, out_ref, "malformed C55 entry mismatch {tag}");
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
        "EXACTNESS PASS: {valid_cnt} small valid + {mal_cnt} malformed + {ot_poison} OutTail poison/fallback + 4 stress + {} Adm + {} Wiki + {} JSONL + {} Code/Docs 100% BYTE-EXACT! (C55-DramUltra-v2 max_codes_447={max_codes_447} < 4096, u32_fb_hits_447={u32_fb_hits_447}, seg_fb_447={seg_fb_447})",
        six.len(), wiki431.len(), non_wiki.len(), code_docs.len()
    );

    let full_wiki: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();
    let all_wiki: Vec<&Block> = wiki431.iter().collect();
    let six_r: Vec<&Block> = six.iter().collect();
    let nw_r: Vec<&Block> = non_wiki.iter().collect();
    let cd_r: Vec<&Block> = code_docs.iter().collect();

    println!("\nDETERMINISTIC SCRATCH & OUTQ WORKING SET (`PeakCap`, `MaxUsed`, `OutQ_live`, `S5`, `dict`, `sp`, `ed`):");
    for (cname, blks) in [
        ("6 Admitted", &six_r), ("233 FullWiki", &full_wiki), ("431 ALL Wiki", &all_wiki),
        ("6 JSONL/Log", &nw_r), ("4 Code/Docs", &cd_r),
    ] {
        let mut kh_w = cand_r5_head::Kernel::default();
        let mut s_w = ArmSuite::default();
        let mut out_w = Vec::with_capacity(900_064);
        let mut stat = [(0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize); 6];
        for &b in blks {
            kh_w.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_w);
            let res = [
                s_w.c54_pl2.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.c54_dl1.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.abl_f12.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.abl_p12.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.abl_p10.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                s_w.c55_du2.run_stats(&b.records, b.n, b.p, &mut out_w).1,
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
            (ARM_NAMES[2], s_w.c54_pl2.open.scratch_report(), s_w.c54_pl2.seg.scratch_bytes()),
            (ARM_NAMES[3], s_w.c54_dl1.open.scratch_report(), s_w.c54_dl1.seg.scratch_bytes()),
            (ARM_NAMES[4], s_w.abl_f12.open.scratch_report(), s_w.abl_f12.seg.scratch_bytes()),
            (ARM_NAMES[5], s_w.abl_p12.open.scratch_report(), s_w.abl_p12.seg.scratch_bytes()),
            (ARM_NAMES[6], s_w.abl_p10.open.scratch_report(), s_w.abl_p10.seg.scratch_bytes()),
            (ARM_NAMES[7], s_w.c55_du2.open.scratch_report(), s_w.c55_du2.seg.scratch_bytes()),
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
    println!("\n=== HEADROOM ORACLE v10 (1T INTERLEAVED, {rds} rds/blk): {label} ===");
    let mut suite = ArmSuite::default();
    let mut out = Vec::with_capacity(900_064);
    for b in blocks.iter().take(6) { for arm in 0..NUM_ARMS { suite.run_arm(arm, b, &mut out); } }

    let (mut tot, mut full_tot, mut oracle_us, mut full_oracle_us) = ([0.0f64; NUM_ARMS], [0.0f64; NUM_ARMS], 0.0f64, 0.0f64);
    let mut by_rc = [[0.0f64; 4]; NUM_ARMS];
    let mut c55_paths = [0usize; 8];

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
        c55_paths[p7 as usize] += 1;
        let mut meds = [0.0f64; NUM_ARMS];
        for arm in 0..NUM_ARMS {
            let m = pct(&mut s[arm], 1, 2);
            meds[arm] = m; tot[arm] += m; by_rc[arm][rc] += m;
            if b.route_class < 2 { full_tot[arm] += m; }
        }
        let best = meds[0].min(meds[2]).min(meds[3]).min(meds[4]).min(meds[5]).min(meds[6]).min(meds[7]);
        oracle_us += best;
        if b.route_class < 2 { full_oracle_us += best; }
        if show_per_block {
            println!(
                "  {:12} | H1={:6.1} | PL2={:.4}x | DL1={:.4}x | F12={:.4}x | P12={:.4}x | P10={:.4}x | DU2(p{})={:6.1} ({:.4}x) | Orcl={:6.1}",
                b.name, meds[0], meds[2] / meds[0], meds[3] / meds[0], meds[4] / meds[0],
                meds[5] / meds[0], meds[6] / meds[0], p7, meds[7], meds[7] / meds[0], best
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
        "  {:18} | TOTAL={:8.1} us ({:.4}x) | C55-DramUltra-v2 capture of oracle headroom = {:.1}%",
        "Hindsight-Oracle", oracle_us, oracle_us / base,
        if base > oracle_us { (base - tot[7]) / (base - oracle_us) * 100.0 } else { 0.0 }
    );
    if show_rc {
        let fb = full_tot[0];
        println!(
            "  233-Full-Wiki 1T   | Head={:.2}ms | PL2={:.4}x | DL1={:.4}x | F12={:.4}x | P12={:.4}x | P10={:.4}x | DU2={:.4}x | Oracle={:.4}x",
            fb / 1e3, full_tot[2] / fb, full_tot[3] / fb, full_tot[4] / fb, full_tot[5] / fb,
            full_tot[6] / fb, full_tot[7] / fb, full_oracle_us / fb
        );
    }
    println!("  C55-DramUltra-v2 routes [L10-bi, tiny, per, seg8, L8-dense, L10-nobi, L24, L32]: {:?}", c55_paths);
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
        let wiki431 = load_bin_corpus(&resolve_corpus_path("wikidata_431_blocks.bin"), "wiki", false);
        run_proxy_arm(arm_name, &wiki431);
        return;
    }
    let verify_only = args.iter().any(|a| a == "--verify-only");
    let six = load_six_admitted();
    let wiki431 = load_bin_corpus(&resolve_corpus_path("wikidata_431_blocks.bin"), "wiki", false);
    let non_wiki = load_bin_corpus(&resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin"), "nonwiki", true);
    let code_docs = load_bin_corpus(&resolve_corpus_path("non_wikidata/code_docs_blocks.bin"), "codedoc", true);
    let dense = make_dense_300k();

    println!(
        "Cycle 55 Headroom Oracle v10 loaded: {} admitted, {} wikidata.bz2, {} JSONL/Log, {} Code/Docs blocks (verify_only={})",
        six.len(), wiki431.len(), non_wiki.len(), code_docs.len(), verify_only
    );

    run_c55_d6_legendre_markov2_audit(&six, &wiki431, &non_wiki, &code_docs, &dense);
    run_exactness_and_ws(&six, &wiki431, &non_wiki, &code_docs, &dense);

    if verify_only {
        println!("\nVERIFY-ONLY COMPLETE (100% PASS)!");
        return;
    }

    let r1_six = bench_1t_interleaved("6 ADMITTED WIKIDATA BLOCKS", &six, 19, false, true);
    let r1_nw = bench_1t_interleaved("6 NON-WIKIDATA JSONL/LOG BLOCKS", &non_wiki, 19, false, true);
    let r1_cd = bench_1t_interleaved("4 NON-WIKIDATA CODE/DOCS BLOCKS", &code_docs, 19, false, true);
    let r1_w431 = bench_1t_interleaved("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 5, true, false);

    let (m2_six, m4_six) = bench_mt_suite("6 ADMITTED WIKIDATA BLOCKS", &six, 15, 11);
    let (m2_nw, m4_nw) = bench_mt_suite("6 NON-WIKIDATA JSONL/LOG BLOCKS", &non_wiki, 15, 11);
    let (m2_cd, m4_cd) = bench_mt_suite("4 NON-WIKIDATA CODE/DOCS BLOCKS", &code_docs, 15, 11);
    let (m2_w431, m4_w431) = bench_mt_suite("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 5, 5);

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
            "    {:18} | 1T={:7.2}ms ({:.4}x, <Open? {:3}) | 2W={:7.2}ms ({:.4}x, <Open? {:3}) | 4W={:7.2}ms ({:.4}x, <Open? {:3})",
            ARM_NAMES[arm],
            sum1_441[arm] / 1e3, sum1_441[arm] / sum1_441[0], lt1,
            sum2_441[arm] / 1e3, sum2_441[arm] / sum2_441[0], lt2,
            sum4_441[arm] / 1e3, sum4_441[arm] / sum4_441[0], lt4,
        );
    }
    println!(
        "    {:18} | 1T={:7.2}ms ({:.4}x) | 6-Admitted 1T/2W/4W C55-DramUltra-v2 = {:.4}x / {:.4}x / {:.4}x",
        "Hindsight-Oracle", orcl1_441 / 1e3, orcl1_441 / sum1_441[0],
        r1_six.tot_us[7] / r1_six.tot_us[0], m2_six[7] / m2_six[0], m4_six[7] / m4_six[0]
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
