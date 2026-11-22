//! Cycle 52 Cumulative Headroom Oracle v7 + `PackedEdgeRule32` Census & SoA/AoS Ablation +
//! Chebyshev (1874) / Markov (1884) / Stieltjes (1894) Canonical 3-Moment Theorem (`C52-D3`,
//! Extending Chain D `CA-26→27→31→37→41→44` to `7` Pure-`CA` Nodes / `6` Links!).
//!
//! Compares Cycle 52's unified **`C52-SuperDropIn-v3`** head-to-head against ALL prior champions
//! AND `PackedEdgeRule32` SoA / AoS ablations (`8` arms total) at `1T`, `2W`, and `4W` across ALL
//! 4 real corpora + `441`-block union + `243`-full union:
//!   0. `R5-Head-A1`         — verbatim `cand_r5_head` (`ID_BITS=22`, `L=16`, `AlwaysAt(65_536)`)
//!   1. `R5-Head-A2(AA)`     — paired A/A noise control
//!   2. `C50-DropIn-v1`      — Cycle 50 `CS-24` (`CA-37/41 + Spec4 + Compact u16 OutQ + K-TailUnw`)
//!   3. `C51-DropIn-v2`      — Cycle 51 `CS-31` (`CS-22 + CS-24` + Hoisted `istk`/`memo_rec` + Tiered `TinyFL`)
//!   4. `C51-Compact3-v1`    — Cycle 51 `CS-27` (`CE-Var234 + ZeroH` + Upfront `ensure_step_u8` + `0.28/dense`)
//!   5. `C52-v3-UnpSoA`      — Ablation: `C52-SuperDropIn-v3` with `PackedEdgeRule32` OFF (separate `edges` + `rules16`)
//!   6. `C52-v3-UnpAoS`      — Ablation: `C52-SuperDropIn-v3` with AoS `(u32, u16)` (`8 B/row`) in `S4`/`S5`
//!   7. `C52-SuperDropIn-v3` — Cycle 52 Unified Champion (`CE-Var234 + ZeroH` + Hoisted `OrphanFree` + Upfront
//!                             `ensure_step_u8` + `PackedEdgeRule32-Sentinel` + Slice-Bound-Eliminated Peeled `S5` +
//!                             4-Tier Stack `TinyFL` + `CA-05/26/27/31/37/41` Cross-Domain Selector)
//!
//! Also verifies:
//!   - **`PackedEdgeRule32` `447`-Block Census:** exact `max_m`, `bits`, `max_codes`, `avg_codes`,
//!     `wide` count (`0/447`), `>4095` count (`0/447`), and `S5` row-length / `S5`-live WS breakdown;
//!   - **Atlas Extension `C52-D3` (Chebyshev 1874 / Markov 1884 / Stieltjes 1894):**
//!     * Cycle 51 Referee Boundary Note #3 exact validity domain `2*L*(L-1) <= (C-1)*(C-2)` (`C=3..25`),
//!     * Stieltjes shifted-origin Hankel positivity ($\det H_1, \det H_1^{(1)}, \det H_1^{(C)} \ge 0$),
//!       exact 3-point principal representation $\{1, \xi^*, C\}$ & 2-point Gauss–Christoffel nodes,
//!     * Exact `i128` 3-moment cubic bounds (`U3_m`, `U3_rt`, `L3_rt_sub`, `L3_rt_sup_S`, `L3_rt_sup_T`,
//!       `L3_m`, `L3_lin`) across `105,000` extremal+random multisets + ALL `447` real blocks + `dense-300k`,
//!     * Honest `O(nrec)` `S_2+S_3` scan timing & runtime kill verdict;
//!   - **Exhaustive Exactness Gate (Release + Overflow-Checked Debug):** `10,190` small valid + `345+`
//!     malformed + `PackedEdgeRule32-Sentinel` forced & natural `num_codes > 4095` overflow tests +
//!     `80` OutTail poison/fallback + `16M` guard + ALL `447/447` real blocks 100% byte-exact!
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
// 1. Shared Low-Level Helpers & OutQ16 (Compact u16 `next` + Exact `cuts`)
// ============================================================================

const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const INLINE_STACK: usize = 64;
const MEMO_REC_CAP: usize = 32;
const PAIR_WORDS: usize = 512;
const CACHE_WORDS: usize = PAIR_WORDS + 2048;
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
    const C52_CROSSOVER: Cap = Cap { m: u32::MAX, strips: usize::MAX, post_s1_crossover: true };
    fn dense(n: usize) -> Cap {
        Cap { m: 256_000u32.min((n as u64 * 3 / 10) as u32), strips: 16_384, post_s1_crossover: false }
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct Counts {
    m: usize,
    bits: u32,
    num_codes: usize,
    wide: bool,
    used_bi: bool,
    used_tail: bool,
    spelling_used: usize,
    used_bytes: usize,
    s5_live_bytes: usize,
    arena_est_gt_m: bool,
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

mod outq {
    const CHUNK: usize = 512;
    pub const BYTES: usize = 2 * CHUNK;
    const NONE32: u32 = u32::MAX;
    const NONE16: u16 = u16::MAX;
    const EPOCH: u32 = 1 << 16;

    #[derive(Default)]
    pub struct Spill {
        pub next: Vec<u16>,
        pub ovf: Vec<[u8; BYTES]>,
        pub cuts: Vec<[u32; 256]>,
    }

    impl Spill {
        pub fn capacity_bytes(&self) -> usize {
            2 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
        }
        pub fn used_bytes(&self, chunks: u32) -> usize {
            2 * (chunks as usize) + BYTES * self.ovf.len() + 1024 * self.cuts.len()
        }
    }

    pub struct Queues<'a> {
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

    impl<'a> Queues<'a> {
        pub fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
            spill.ovf.clear();
            spill.cuts.clear();
            let in_out = (out.len() / BYTES) as u32;
            let out = &mut out[..in_out as usize * BYTES];
            Queues { in_out, out, spill, chunks: 0, heads: [NONE32; 256], pt: [u32::MAX as u64; 256], limit, over: false }
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
            if self.over || m + w > self.limit { self.over = true; return; }
            let mut left = w;
            while left > 0 {
                let p = self.pt[a] as u32;
                if p == u32::MAX || (p & 1023) as usize == CHUNK {
                    if self.chunks >= u32::from(NONE16) { self.over = true; return; }
                    self.take_chunk_pt(a);
                }
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & 1023) as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(p >> 10);
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
            if self.heads[a] == NONE32 { self.heads[a] = id; }
            else { self.spill.next[((self.pt[a] as u32) >> 10) as usize] = id as u16; }
            self.pt[a] = hi | u64::from(id << 10);
        }

        #[cold]
        #[inline(never)]
        fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
            if id < self.in_out { let at = id as usize * BYTES; &mut self.out[at..at + BYTES] }
            else { &mut self.spill.ovf[(id - self.in_out) as usize][..] }
        }
    }

    pub struct Reader<'a> {
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
        pub fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
            Reader {
                out, spill, in_out: heads.in_out, a, id: heads.heads[a],
                at: 0, read: 0, epoch: 0, cut: spill.cuts.first().map_or(u32::MAX, |c| c[a]),
            }
        }
        #[inline(always)]
        fn next_chunk(&self, id: usize) -> u32 { u32::from(self.spill.next[id]) }
        #[inline(always)]
        fn cut_at(&self, epoch: usize) -> u32 { self.spill.cuts.get(epoch).map_or(u32::MAX, |c| c[self.a]) }

        #[inline(always)]
        pub fn wire_xor_nomark(&mut self, edges: &mut [u32], dst_start: usize, span: usize, pack: u32) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
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
                if i + 4 <= span {
                    let b = &bytes[2 * i..2 * i + 8];
                    let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let (v0, v1, v2, v3) = (hi | (q as u32 & 0xFFFF), hi | ((q >> 16) as u32 & 0xFFFF), hi | ((q >> 32) as u32 & 0xFFFF), hi | ((q >> 48) as u32));
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
        pub fn wire_xor(&mut self, edges: &mut [u32], dst_start: usize, span: usize, pack: u32, mark: u32, predecessor: &mut u32) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                let mut before = *predecessor;
                for (i, &s) in pairs.iter().enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes(s));
                    if v == mark { before = u as u32; }
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
        fn wire_xor_slow(&mut self, edges: &mut [u32], mut dst_start: usize, mut left: usize, pack: u32, mark: u32, predecessor: &mut u32) {
            while left > 0 {
                while self.read == self.cut { self.epoch += 1; self.cut = self.cut_at(self.epoch as usize); }
                if self.at == CHUNK { self.id = self.next_chunk(self.id as usize); self.at = 0; }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES; &self.out[at..at + BYTES]
                } else { &self.spill.ovf[(self.id - self.in_out) as usize][..] };
                let hi = self.epoch << 16;
                let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                for (i, &s) in pairs.iter().enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes(s));
                    if v == mark { *predecessor = u as u32; }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                self.at += take; self.read += take as u32; dst_start += take; left -= take;
            }
        }

        /// Ablation helper: AoS `(u32, u16)` wiring (`8 B/row`, `S4`/`S5` co-located unpacked)
        pub fn wire_xor_aos(&mut self, aos: &mut [(u32, u16)], dst_start: usize, mut left: usize, code: u16, mark: u32, predecessor: &mut u32) {
            let mut u0 = dst_start;
            while left > 0 {
                while self.read == self.cut { self.epoch += 1; self.cut = self.cut_at(self.epoch as usize); }
                if self.at == CHUNK { self.id = self.next_chunk(self.id as usize); self.at = 0; }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES; &self.out[at..at + BYTES]
                } else { &self.spill.ovf[(self.id - self.in_out) as usize][..] };
                let hi = self.epoch << 16;
                let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                for (i, &s) in pairs.iter().enumerate() {
                    let u = u0 + i;
                    let v = hi | u32::from(u16::from_le_bytes(s));
                    if v == mark { *predecessor = u as u32; }
                    aos[u].0 ^= v;
                    aos[u].1 = code;
                    aos[v as usize].0 ^= u as u32;
                }
                self.at += take; self.read += take as u32; u0 += take; left -= take;
            }
        }

        #[inline(always)]
        pub fn wire(&mut self, dst: &mut [u32], pack: u32) {
            let span = dst.len();
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
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
                if self.at == CHUNK { self.id = self.next_chunk(self.id as usize); self.at = 0; }
                let take = dst.len().min(CHUNK - self.at).min((self.cut - self.read) as usize);
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
// 2. Engine A (Fixed RULE=4): Faithfully Implements `C50-DropIn-v1` & `C51-DropIn-v2`
// ============================================================================

mod engine_rule4 {
    use super::*;
    const RULE: usize = 4;

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
        #[inline(always)] fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) { let s = &a[id as usize * 4..id as usize * 4 + 4]; (s[0] >> 16, s[1], s[2], s[3]) }
    }
    impl Links for Wide {
        const STRIDE: usize = 6; const IS_NARROW: bool = false; const BOTTOM: usize = 2;
        #[inline(always)] fn prev(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side] }
        #[inline(always)] fn next(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side + 1] }
        #[inline(always)] fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side] = to; }
        #[inline(always)] fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side + 1] = to; }
        #[inline(always)] fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) { let s = &a[id as usize * 6..id as usize * 6 + 6]; (s[1], s[2], s[4], s[5]) }
    }

    struct Elim<'a, L: Links> { a: &'a mut [u32], base: usize, top: usize, nodes: u32, _l: std::marker::PhantomData<L> }
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
                let (next, prev, w) = (L::next(self.a, first, 1), L::prev(self.a, last, 1), self.width(first));
                hi -= self.width(last); self.set_bottom(first, lo); self.set_bottom(last, hi); lo += w;
                if next == last { assert_eq!(lo, hi); return; }
                first = next; last = prev;
            }
            if first != 0 { self.set_bottom(first, lo); lo += self.width(first); }
            assert_eq!(lo, hi);
        }
        #[inline(always)] fn node(&mut self, w: u32, wd: u32) -> u32 { let id = self.nodes; self.nodes += 1; self.set_width(id, w); self.set_word(id, wd); id }
        #[inline(always)] fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
            if list.1 == 0 { list.0 = id; } else { L::set_next(self.a, list.1, side, id); }
            L::set_prev(self.a, id, side, list.1); list.1 = id;
        }
        fn close(&mut self, side: usize, a: (u32, u32), b: (u32, u32)) {
            let mut prev = 0;
            for (h, t) in [a, b] { if t == 0 { continue; } L::set_next(self.a, prev, side, h); L::set_prev(self.a, h, side, prev); prev = t; }
            L::set_next(self.a, prev, side, 0); L::set_prev(self.a, 0, side, prev);
        }
        #[inline(always)] fn cut(&mut self, id: u32, side: usize) { let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side)); L::set_next(self.a, p, side, n); L::set_prev(self.a, n, side, p); }
        #[inline(always)] fn pop_tail(&mut self, id: u32, side: usize) { let p = L::prev(self.a, id, side); L::set_next(self.a, p, side, 0); L::set_prev(self.a, 0, side, p); }
        #[inline(always)] fn insert(&mut self, at: u32, id: u32, side: usize) {
            let n = L::next(self.a, at, side); L::set_prev(self.a, id, side, at); L::set_next(self.a, id, side, n); L::set_next(self.a, at, side, id); L::set_prev(self.a, n, side, id);
        }
        #[inline(always)] fn rule(&self, id: u32) -> usize { self.base + id as usize * RULE }
        #[inline(always)] fn len_handle(&self, h: u32) -> u32 { let g = h >> ID_BITS; if g < SATURATED { g } else { self.a[self.rule(h & ID_MASK) + 2] } }
        #[inline(always)] fn short_payload_h(&self, h: u32) -> u128 {
            let (g, id) = (h >> ID_BITS, h & ID_MASK);
            if g <= 3 { return u128::from(id); }
            let r = self.rule(id); let s = &self.a[r..r + 4];
            u128::from(s[0]) | (u128::from(s[1]) << 32) | (u128::from(s[2]) << 64) | (u128::from(s[3] & 0x00ff_ffff) << 96)
        }
        #[inline(always)] fn join(&mut self, left: u32, right: u32) -> u32 {
            if left == 0 { return right; } if right == 0 { return left; }
            let ll = left >> ID_BITS; let sum = ll + (right >> ID_BITS);
            if sum < INLINE as u32 {
                if sum <= 3 { return (sum << ID_BITS) | (left & ID_MASK) | ((right & ID_MASK) << (8 * ll)); }
                let id = ((self.top - self.base) / RULE) as u32;
                let bytes = self.short_payload_h(left) | (self.short_payload_h(right) << (8 * ll)) | (u128::from(sum) << 120);
                let out = &mut self.a[self.top..self.top + RULE];
                out[0] = bytes as u32; out[1] = (bytes >> 32) as u32; out[2] = (bytes >> 64) as u32; out[3] = (bytes >> 96) as u32;
                self.top += RULE; (sum << ID_BITS) | id
            } else {
                let len = if sum <= SATURATED { sum } else { self.len_handle(left) + self.len_handle(right) };
                let id = ((self.top - self.base) / RULE) as u32;
                let out = &mut self.a[self.top..self.top + RULE];
                out[0] = left; out[1] = right; out[2] = len; out[3] = u32::MAX;
                self.top += RULE; (len.min(SATURATED) << ID_BITS) | id
            }
        }
        fn power(&mut self, word: u32, q: u32) -> u32 {
            let (mut acc, mut base, mut e) = (0, word, q);
            loop { if e & 1 == 1 { acc = self.join(acc, base); } e >>= 1; if e == 0 { break acc; } base = self.join(base, base); }
        }
        #[inline(never)]
        fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
            let mut left = *remaining;
            let done = loop {
                if left == 0 { break true; }
                if self.top + RULE > self.a.len() { break false; }
                let (x, y) = (self.a[0] & 0xFFFF, self.a[1] & 0xFFFF);
                let (xi, yi) = (x as usize * 4, y as usize * 4);
                let (x0, x1, aw, wx) = (self.a[xi], self.a[xi + 1], self.a[xi + 2], self.a[xi + 3]);
                if x == y {
                    left -= aw;
                    let (px0, px1) = (x0 & 0xFFFF, x1 & 0xFFFF);
                    self.a[px0 as usize * 4] &= 0xFFFF; self.a[0] = (self.a[0] & 0xFFFF_0000) | px0;
                    self.a[px1 as usize * 4 + 1] &= 0xFFFF; self.a[1] = (self.a[1] & 0xFFFF_0000) | px1;
                    continue;
                }
                let (y0, y1, bw, wy) = (self.a[yi], self.a[yi + 1], self.a[yi + 2], self.a[yi + 3]);
                let w = aw.min(bw); left -= w;
                let word = self.join(wx, wy);
                if aw <= bw {
                    let px = x0 & 0xFFFF;
                    self.a[px as usize * 4] &= 0xFFFF; self.a[0] = (self.a[0] & 0xFFFF_0000) | px;
                    if aw < bw {
                        let y_next = if y == px { 0 } else { y0 >> 16 };
                        self.a[xi] = y | (y_next << 16); self.a[xi + 3] = word;
                        self.a[yi] = (y0 & 0xFFFF) | (x << 16); self.a[yi + 2] = bw - aw;
                        let yni = y_next as usize * 4; self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    } else {
                        let y_next = if y == px { 0 } else { y0 >> 16 }; let y_prev = y0 & 0xFFFF;
                        self.a[xi] = y_prev | (y_next << 16); self.a[xi + 3] = word;
                        let ypi = y_prev as usize * 4; self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                        let yni = y_next as usize * 4; self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                        let py = y1 & 0xFFFF; self.a[py as usize * 4 + 1] &= 0xFFFF; self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                    }
                } else {
                    let py = y1 & 0xFFFF; self.a[py as usize * 4 + 1] &= 0xFFFF; self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                    let x_next = if x == py { 0 } else { x1 >> 16 };
                    self.a[yi + 1] = x | (x_next << 16); self.a[yi + 3] = word;
                    self.a[xi + 1] = (x1 & 0xFFFF) | (y << 16); self.a[xi + 2] = aw - bw;
                    let xni = x_next as usize * 4 + 1; self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
                }
            };
            *remaining = left; done
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
                        let head = self.power(self.word(x), q);
                        let word = self.join(head, self.word(y));
                        self.set_word(y, word); self.set_width(x, aw - q * bw); left -= q * bw; continue;
                    }
                    if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                        let q = (bw - 1) / aw;
                        let tail = self.power(self.word(y), q);
                        let word = self.join(self.word(x), tail);
                        self.set_word(x, word); self.set_width(y, bw - q * aw); left -= q * aw; continue;
                    }
                }
                let w = aw.min(bw); left -= w;
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
        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn append_tail<const TRACK_MEMO: bool, const SP_STEP: usize>(
            &mut self, handle: u32, istk: &mut [u32; INLINE_STACK], ovf: &mut Vec<u32>,
            dest: &mut Vec<u8>, memo_rec: &mut [u32; MEMO_REC_CAP], memo_n: &mut usize, memo_ovf: &mut bool,
        ) {
            let (mut sp, mut used_heap, mut h) = (0usize, false, handle);
            loop {
                let g = h >> ID_BITS;
                if g < INLINE as u32 {
                    let b = self.short_payload_h(h).to_le_bytes();
                    ensure_step_u8(dest, g as usize, SP_STEP);
                    dest.extend_from_slice(&b[..g as usize]);
                    if used_heap && !ovf.is_empty() { h = ovf.pop().unwrap(); continue; }
                    if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                    return;
                }
                let r = self.rule(h & ID_MASK);
                let tag = self.a[r + 3];
                if tag != u32::MAX {
                    let (st, rl) = ((tag & 0x7fff_ffff) as usize, self.a[r + 2] as usize);
                    ensure_step_u8(dest, rl, SP_STEP);
                    dest.extend_from_within(st..st + rl);
                    if used_heap && !ovf.is_empty() { h = ovf.pop().unwrap(); continue; }
                    if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                    return;
                }
                self.a[r + 3] = 0x8000_0000 | (dest.len() as u32);
                if TRACK_MEMO {
                    if *memo_n < MEMO_REC_CAP { memo_rec[*memo_n] = r as u32; *memo_n += 1; } else { *memo_ovf = true; }
                }
                let (left, right) = (self.a[r], self.a[r + 1]);
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
                        if used_heap && !ovf.is_empty() { h = ovf.pop().unwrap(); continue; }
                        if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                        return;
                    }
                    h = right;
                } else {
                    if sp < INLINE_STACK { istk[sp] = right; sp += 1; }
                    else { if !used_heap { ovf.clear(); used_heap = true; } ovf.push(right); }
                    h = left;
                }
            }
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
                if self.ns < self.cap_strips {
                    let at = 3 * self.ns;
                    if self.edges.len() < at + 3 {
                        let need = (at + 3).div_ceil(4096) * 4096;
                        if need > self.edges.capacity() { self.edges.reserve_exact(need - self.edges.len()); }
                        self.edges.resize(need, 0);
                    }
                    let ta = self.queues.thin_of(a);
                    self.edges[at..at + 3].copy_from_slice(&[x, ta + f.thick_rows[a], (w << 8) | u32::from(c)]);
                }
                self.ns += 1; f.thick[a] += 1; f.thick_rows[a] += w;
            }
        }
    }

    struct Dictionary<'a, const ORPHAN_FREE: bool> {
        pairs: &'a mut [u32; 512],
        contents: &'a mut [u32; 2048],
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        spelling: &'a mut Vec<u8>,
        istk: &'a mut [u32; INLINE_STACK],
        memo_rec: &'a mut [u32; MEMO_REC_CAP],
        stack: &'a mut Vec<u32>,
    }
    impl<const ORPHAN_FREE: bool> Dictionary<'_, ORPHAN_FREE> {
        #[inline(always)]
        fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
            let bucket = ((handle.wrapping_mul(0x9e37_79b9) ^ (u32::from(byte).wrapping_mul(0x85eb_ca6b))) >> 24) as usize;
            let slot = bucket * 2;
            let val = self.pairs[slot + 1];
            if self.pairs[slot] == handle && (val >> 24) == u32::from(byte) { return val & 0x00FF_FFFF; }
            let c = self.code_miss(e, handle, handle & ID_MASK, byte);
            self.pairs[slot] = handle; self.pairs[slot + 1] = (u32::from(byte) << 24) | c; c
        }
        fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8) -> u32 {
            let sp_step = if ORPHAN_FREE { 4096usize } else { 8192usize };
            let il_step = if ORPHAN_FREE { 128usize } else { 256usize };
            let height = (handle >> ID_BITS) as usize;
            if height < INLINE {
                let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
                let entry = v.to_le_bytes(); let len = (height + 1) as u8;
                let hh = ((v as u64) ^ ((v >> 64) as u64).rotate_left(29) ^ (u64::from(len) << 56)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                let hh = hh ^ (hh >> 30);
                let tag = (((hh >> 56) as u32) | 0x80) << 24;
                let idx = (hh as usize) & 2047;
                let ent = self.contents[idx];
                if (ent & 0xFF00_0000) == tag && { let old = (ent & 0x00FF_FFFF) as usize; self.lens[old] == len && self.inline16[old] == entry } {
                    return ent & 0x00FF_FFFF;
                }
                ensure_step_inline(self.inline16, self.lens, il_step);
                let c = self.lens.len() as u32; self.inline16.push(entry); self.lens.push(len); self.contents[idx] = tag | c;
                return c;
            }
            let st = self.spelling.len();
            let r = e.rule(id);
            let tag_r = e.a[r + 3];
            let (mut memo_n, mut memo_ovf) = (0usize, false);
            let was_memo = tag_r != u32::MAX;
            if was_memo {
                let (p0, rl) = ((tag_r & 0x7fff_ffff) as usize, e.a[r + 2] as usize);
                ensure_step_u8(self.spelling, rl + 1, sp_step);
                self.spelling.extend_from_within(p0..p0 + rl);
                self.spelling.push(byte);
            } else {
                if ORPHAN_FREE {
                    e.append_tail::<true, 4096>(handle, self.istk, self.stack, self.spelling, self.memo_rec, &mut memo_n, &mut memo_ovf);
                } else {
                    e.append_tail::<false, 8192>(handle, self.istk, self.stack, self.spelling, self.memo_rec, &mut memo_n, &mut memo_ovf);
                }
                ensure_step_u8(self.spelling, 1, sp_step);
                self.spelling.push(byte);
            }
            let entry = long_slot(st as u32, self.spelling.len() as u32);
            let bytes = &self.spelling[st..];
            let hh = spelling_hash_long(bytes);
            let tag = (((hh >> 56) as u32) | 0x80) << 24;
            let idx = (hh as usize) & 2047;
            let ent = self.contents[idx];
            if (ent & 0xFF00_0000) == tag && { let old = (ent & 0x00FF_FFFF) as usize; self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes } {
                if was_memo { self.spelling.truncate(st); }
                else if ORPHAN_FREE && !memo_ovf {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    let old_st = u32::from_le_bytes(self.inline16[old][..4].try_into().unwrap());
                    let delta = (st as u32) - old_st;
                    for &rm in &self.memo_rec[..memo_n] { e.a[rm as usize + 3] -= delta; }
                    self.spelling.truncate(st);
                } else { self.spelling.pop(); }
                return ent & 0x00FF_FFFF;
            }
            ensure_step_inline(self.inline16, self.lens, il_step);
            let c = self.lens.len() as u32; self.inline16.push(entry); self.lens.push(LONG); self.contents[idx] = tag | c; c
        }
    }

    #[derive(Default)]
    pub struct Kernel<const ORPHAN_FREE: bool, const TIGHT_ARENA: bool, const TIERED_TINY: bool> {
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

    impl<const ORPHAN_FREE: bool, const TIGHT_ARENA: bool, const TIERED_TINY: bool>
        Kernel<ORPHAN_FREE, TIGHT_ARENA, TIERED_TINY>
    {
        pub fn scratch_report(&self) -> (usize, usize, String) {
            let sp = self.spill.capacity_bytes();
            let bd = 8 * self.bands.capacity();
            let s = self.spelling.capacity();
            let ed = 4 * self.edges.capacity();
            let ex = 4 * self.extra.capacity();
            let il = 16 * self.inline16.capacity() + self.lens.capacity();
            let st = 4 * self.stack.capacity();
            let tot = sp + bd + s + ed + ex + il + st;
            (tot, ed, format!("ed={:.1}K sp={:.1}K il={:.1}K q={:.1}K bd={:.1}K", ed as f64 / 1024.0, s as f64 / 1024.0, il as f64 / 1024.0, sp as f64 / 1024.0, bd as f64 / 1024.0))
        }

        #[inline(never)]
        pub fn invert_tiny_fl(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
            if n == 0 || (p as usize) >= n || records.len() > 16_000_000 { return false; }
            if records.len() == 1 {
                if usize::from(records[0] & 255) != n { return false; }
                out.clear(); out.resize(n, (records[0] >> 8) as u8); return true;
            }
            if TIERED_TINY {
                if n <= 64 { return invert_tiny_fl_64(records, n, p, out); }
                if n <= 256 { return invert_tiny_fl_256(records, n, p, out); }
                if n <= 1536 { return invert_tiny_fl_1536(records, n, p, out); }
            }
            if self.edges.len() < n { let add = n - self.edges.len(); self.edges.reserve_exact(add); self.edges.resize(n, 0); }
            invert_tiny_fl_with_buf(records, n, p, &mut self.edges[..n], out)
        }

        #[inline(never)]
        pub fn try_run_const<const LIM: u32>(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, bi_mode: BiMode, out: &mut Vec<u8>) -> Option<Counts> {
            let nrec = records.len();
            if n == 0 || n > 900_000 || (p as usize) >= n || nrec > 16_000_000 { return None; }
            out.resize(n + INLINE, 0);
            let (ns, f, heads, over, extra_127, tot_pos) = {
                let mut pass = Pass {
                    queues: outq::Queues::new(&mut out[..], &mut self.spill, cap.m.min(n as u32)),
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
        fn finish(&mut self, f: &First, ns: usize, n: usize, r_max: u32, heads: &outq::Heads, bi_mode: BiMode, out: &mut Vec<u8>) -> Option<Counts> {
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            if all.iter().sum::<u32>() != n as u32 { return None; }
            let first = starts(all);
            let mut write = starts(f.thick);
            let m = f.m as usize;
            let most = 3 * ns + 3;
            let use_narrow = most <= NARROW_NODES;
            let stride = if use_narrow { Narrow::STRIDE } else { Wide::STRIDE };
            let arena_est = if TIGHT_ARENA { 4 * ns + stride * most } else { 4 * ns + (stride + RULE) * most };
            let need = arena_est.max(m);
            if self.edges.len() < need { let add = need - self.edges.len(); self.edges.reserve_exact(add); self.edges.resize(need, 0); }
            self.bands.clear(); self.spelling.clear();
            let tail_offset = (heads.chunks as usize).min(out.len() / outq::BYTES) * outq::BYTES;
            let (cursors, used_tail, nbands, top_words) = if use_narrow {
                self.induced_with::<Narrow>(ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..])
            } else {
                self.induced_with::<Wide>(ns, n as u32, f.m, &f.thin, &first, &mut write, most, &mut out[tail_offset..])
            };
            let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
            let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
            if self.edges.len() < m { self.edges.resize(m, 0); }
            self.extra.clear();
            if wide { self.extra.resize(m, 0); }
            self.predecessor = None;
            let use_xor = !wide && bi_mode.should_use_bi(f.m, ns as u32, r_max);
            let (out_q, out_tail) = out.split_at(tail_offset);
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
            let s5_live = 4 * m + 4 * self.extra.len() + 16 * self.inline16.len() + self.lens.len() + self.spelling.len();
            let used_bytes = 4 * (4 * ns + top_words).max(m) + 4 * self.extra.len() + 8 * self.bands.len()
                + self.spelling.len() + 16 * self.inline16.len() + self.lens.len() + self.spill.used_bytes(heads.chunks);
            if let Some(before) = self.predecessor {
                Self::emit_bi(&self.edges[..m], &self.spelling, &self.inline16, &self.lens, n, f.mark, before, bits, out);
            } else {
                Self::emit_uni(&self.edges[..m], &self.extra, &self.spelling, &self.inline16, &self.lens, n, f.mark, bits, out);
            }
            Some(Counts {
                m, bits, num_codes: self.lens.len(), wide, used_bi: use_xor, used_tail,
                spelling_used: self.spelling.len(), used_bytes, s5_live_bytes: s5_live, arena_est_gt_m: arena_est > m,
            })
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_tail(&mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, cursors: &[usize; 256], nbands: usize, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads) -> u32 {
            let edges = &mut self.edges[..m]; edges.fill(0);
            let (bs, _) = out_tail.as_chunks::<8>();
            let mut pred = mark; let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &raw in &bs[cursors[a]..end] { let b = Band(u64::from_le_bytes(raw)); q.wire_xor_nomark(edges, b.dst() as usize, b.end() as usize, b.code() << bits); }
                } else {
                    for &raw in &bs[cursors[a]..end] { let b = Band(u64::from_le_bytes(raw)); q.wire_xor(edges, b.dst() as usize, b.end() as usize, b.code() << bits, mark, &mut pred); }
                }
            }
            pred
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_vec(&mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, cursors: &[usize; 256], out_q: &[u8], heads: &outq::Heads) -> u32 {
            let edges = &mut self.edges[..m]; edges.fill(0);
            let mut pred = mark; let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &b in &self.bands[cursors[a]..end] { q.wire_xor_nomark(edges, b.dst() as usize, b.end() as usize, b.code() << bits); }
                } else {
                    for &b in &self.bands[cursors[a]..end] { q.wire_xor(edges, b.dst() as usize, b.end() as usize, b.code() << bits, mark, &mut pred); }
                }
            }
            pred
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_tail(&mut self, m: usize, wide: bool, bits: u32, cursors: &[usize; 256], nbands: usize, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads) {
            let edges = &mut self.edges[..m]; let (bs, _) = out_tail.as_chunks::<8>();
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                for &raw in &bs[cursors[a]..end] {
                    let b = Band(u64::from_le_bytes(raw)); let (sp, dst, bc) = (b.end() as usize, b.dst() as usize, b.code());
                    if wide { self.extra[dst..dst + sp].fill(bc); }
                    q.wire(&mut edges[dst..dst + sp], if wide { 0 } else { bc << bits });
                }
            }
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_vec(&mut self, m: usize, wide: bool, bits: u32, cursors: &[usize; 256], out_q: &[u8], heads: &outq::Heads) {
            let edges = &mut self.edges[..m];
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let (sp, dst, bc) = (b.end() as usize, b.dst() as usize, b.code());
                    if wide { self.extra[dst..dst + sp].fill(bc); }
                    q.wire(&mut edges[dst..dst + sp], if wide { 0 } else { bc << bits });
                }
            }
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_with<L: Links>(&mut self, ns: usize, n: u32, m: u32, thin: &[u32; 256], first: &[u32; 256], write: &mut [u32; 256], most: usize, out_tail: &mut [u8]) -> ([usize; 256], bool, usize, usize) {
            let mut widest = 0u32;
            let (base, mut top, nodes, open_nodes) = {
                let (hdr, arena) = self.edges.split_at_mut(4 * ns);
                let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
                let mut e = Elim::<L> { a: arena, base: 0, top: 0, nodes: 0, _l: std::marker::PhantomData };
                e.node(0, 0); e.nodes = ns as u32 + 1;
                let (mut terms, mut ord) = ([(0, 0); 2], [(0, 0); 2]);
                let mut x = 0u32;
                for i in 0..ns {
                    let sw = &mut strips_raw[3 * i..3 * i + 3];
                    let (from, wbyte) = (sw[0], sw[2]);
                    let (byte, width) = ((wbyte & 0xFF) as u8, wbyte >> 8);
                    let a = usize::from(byte);
                    sw[1] += first[a];
                    let di = write[a] as usize; image_order[di] = i as u32; write[a] = di as u32 + 1;
                    let sid = i as u32 + 1; e.set_width(sid, width); e.set_word(sid, literal(byte));
                    widest = widest.max(width);
                    if x < from { let gw = from - x; widest = widest.max(gw); let gid = e.node(gw, 0); e.append_to(&mut ord[0], gid, 0); e.append_to(&mut terms[1], gid, 1); }
                    e.append_to(&mut ord[0], sid, 0); x = from + width;
                }
                if x < n { widest = widest.max(n - x); let id = e.node(n - x, 0); e.append_to(&mut ord[0], id, 0); e.append_to(&mut terms[1], id, 1); }
                x = 0;
                for &i in image_order.iter() {
                    let sw = &strips_raw[3 * i as usize..3 * i as usize + 3];
                    let (to, width) = (sw[1], sw[2] >> 8);
                    if x < to { widest = widest.max(to - x); let id = e.node(to - x, 0); e.append_to(&mut ord[1], id, 1); e.append_to(&mut terms[0], id, 0); }
                    e.append_to(&mut ord[1], i + 1, 1); x = to + width;
                }
                if x < n { widest = widest.max(n - x); let id = e.node(n - x, 0); e.append_to(&mut ord[1], id, 1); e.append_to(&mut terms[0], id, 0); }
                let open_nodes = e.nodes as usize - 1;
                for side in 0..2 { e.close(side, terms[side], ord[side]); }
                let base = L::STRIDE * e.nodes as usize;
                let _ = most;
                (base, base, e.nodes, open_nodes)
            };
            let spirals = widest > CERTIFIED_WIDTH;
            let mut remaining = n;
            loop {
                let mut e = Elim::<L> { a: &mut self.edges[4 * ns..], base, top, nodes, _l: std::marker::PhantomData };
                let done = if spirals { e.eliminate::<true>(&mut remaining) } else { e.eliminate::<false>(&mut remaining) };
                top = e.top;
                if done { break; }
                grow_edges_step(&mut self.edges, 4 * ns + top + SPIRAL_RULES * RULE);
            }
            let (header, arena) = self.edges.split_at_mut(4 * ns);
            let (live, tail_arena) = arena.split_at_mut(top);
            let cache = if header.len() >= CACHE_WORDS { header } else { tail_arena };
            let mut e = Elim::<L> { a: live, base, top, nodes, _l: std::marker::PhantomData };
            e.assign_bottom(m);
            begin_inline(&mut self.inline16, &mut self.lens);
            let max_bands = open_nodes + 256;
            let use_tail = out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec = [0u32; MEMO_REC_CAP];
            let sp_step = if ORPHAN_FREE { 4096usize } else { 8192usize };
            let il_step = if ORPHAN_FREE { 128usize } else { 256usize };
            if cache.len() < CACHE_WORDS {
                let (bs, _) = out_tail.as_chunks_mut::<8>();
                let (mut id, mut in_node, mut k) = (L::next(e.a, 0, 0), 0, 0);
                for a in 0..256 {
                    cursors[a] = if use_tail { nbands } else { self.bands.len() };
                    let end = k + thin[a];
                    while k < end {
                        let (nid, nb, nw, nwd) = L::band_node(e.a, id);
                        let width = (end - k).min(nw - in_node);
                        let dst = nb + in_node;
                        let code = if nwd == 0 { a as u32 } else {
                            let h = (nwd >> ID_BITS) as usize;
                            let (entry, len) = if h < INLINE {
                                ((e.short_payload_h(nwd) | (u128::from(a as u8) << (8 * h))).to_le_bytes(), (h + 1) as u8)
                            } else {
                                let st = self.spelling.len() as u32;
                                let (mut dn, mut do_v) = (0usize, false);
                                e.append_tail::<false, 4096>(nwd, &mut istk, &mut self.stack, &mut self.spelling, &mut memo_rec, &mut dn, &mut do_v);
                                ensure_step_u8(&mut self.spelling, 1, sp_step);
                                self.spelling.push(a as u8);
                                (long_slot(st, self.spelling.len() as u32), LONG)
                            };
                            ensure_step_inline(&mut self.inline16, &mut self.lens, il_step);
                            let c = self.lens.len() as u32;
                            self.inline16.push(entry); self.lens.push(len); c
                        };
                        if use_tail { bs[nbands] = Band::new(width, dst, code).0.to_le_bytes(); nbands += 1; }
                        else { self.bands.push(Band::new(width, dst, code)); }
                        in_node += width; k += width;
                        if in_node == nw { id = nid; in_node = 0; }
                    }
                }
            } else {
                cache[..CACHE_WORDS].fill(0);
                let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
                let mut dict = Dictionary::<ORPHAN_FREE> {
                    pairs: p_sl.try_into().unwrap(), contents: c_sl.try_into().unwrap(),
                    inline16: &mut self.inline16, lens: &mut self.lens, spelling: &mut self.spelling,
                    istk: &mut istk, memo_rec: &mut memo_rec, stack: &mut self.stack,
                };
                let mut id = L::next(e.a, 0, 0);
                let (mut nid, mut nb, mut nw, mut nwd) = if id != 0 { L::band_node(e.a, id) } else { (0, 0, 0, 0) };
                let (mut in_node, mut k) = (0u32, 0u32);
                if use_tail {
                    let (band_slots, _) = out_tail[..max_bands * 8].as_chunks_mut::<8>();
                    for a in 0..256 {
                        cursors[a] = nbands;
                        let end = k + thin[a];
                        let (mut last_w, mut last_c) = (0u32, 0u32);
                        while k < end {
                            let width = (end - k).min(nw - in_node);
                            let dst = nb + in_node;
                            let code = if nwd == 0 { a as u32 } else if nwd == last_w { last_c } else {
                                let c = dict.code(&mut e, nwd, a as u8); last_w = nwd; last_c = c; c
                            };
                            band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes(); nbands += 1;
                            in_node += width; k += width;
                            if in_node == nw { id = nid; in_node = 0; if id != 0 { (nid, nb, nw, nwd) = L::band_node(e.a, id); } }
                        }
                    }
                } else {
                    for a in 0..256 {
                        cursors[a] = self.bands.len();
                        let end = k + thin[a];
                        let (mut last_w, mut last_c) = (0u32, 0u32);
                        while k < end {
                            let width = (end - k).min(nw - in_node);
                            let dst = nb + in_node;
                            let code = if nwd == 0 { a as u32 } else if nwd == last_w { last_c } else {
                                let c = dict.code(&mut e, nwd, a as u8); last_w = nwd; last_c = c; c
                            };
                            self.bands.push(Band::new(width, dst, code));
                            in_node += width; k += width;
                            if in_node == nw { id = nid; in_node = 0; if id != 0 { (nid, nb, nw, nwd) = L::band_node(e.a, id); } }
                        }
                    }
                }
            }
            (cursors, use_tail, nbands, top)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_uni(edges: &[u32], extra: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
            let (mut at, mask, mut j) = (0usize, (1 << bits) - 1, mark);
            for _ in 0..edges.len() {
                let edge = edges[j as usize];
                let code = if extra.is_empty() { edge >> bits } else { extra[j as usize] } as usize;
                j = edge & mask;
                let len = lens[code];
                if len != LONG { out[at..at + INLINE].copy_from_slice(&inline[code]); at += usize::from(len); }
                else { let w = long_word(&inline[code], spelling); let t = w.len().min(n - at); out[at..at + t].copy_from_slice(&w[..t]); at += t; }
                if j == mark || at >= n { break; }
            }
            out.truncate(at.min(n)); repeat(n, out);
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi(edges: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, before: u32, bits: u32, out: &mut Vec<u8>) {
            let mask = (1 << bits) - 1;
            let (mut prev, mut front, mut back, mut next, mut lo, mut hi) = (before, mark, before, mark, 0usize, n);
            for _ in 0..=edges.len() / 2 {
                let ef = edges[front as usize]; let cf = (ef >> bits) as usize; let single = front == back; let lf = lens[cf];
                if lf != LONG && hi - lo >= INLINE { out[lo..lo + INLINE].copy_from_slice(&inline[cf]); lo += usize::from(lf); }
                else { let w = if lf == LONG { long_word(&inline[cf], spelling) } else { &inline[cf][..usize::from(lf)] }; let t = w.len().min(hi - lo); out[lo..lo + t].copy_from_slice(&w[..t]); lo += t; }
                if single || lo == hi { break; }
                let eb = edges[back as usize]; let cb = (eb >> bits) as usize; let lb = lens[cb];
                if lb != LONG && hi - lo >= INLINE {
                    let sh = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                    out[hi - INLINE..hi].copy_from_slice(&sh.to_le_bytes()); hi -= usize::from(lb);
                } else { let w = if lb == LONG { long_word(&inline[cb], spelling) } else { &inline[cb][..usize::from(lb)] }; let t = w.len().min(hi - lo); out[hi - t..hi].copy_from_slice(&w[w.len() - t..]); hi -= t; }
                (prev, front) = (front, (ef & mask) ^ prev);
                (next, back) = (back, (eb & mask) ^ next);
                if lo == hi || front == next { break; }
            }
            if lo < hi { out.copy_within(hi..n, lo); out.truncate(lo + (n - hi)); repeat(n, out); } else { out.truncate(n); }
        }
    }
}

// ============================================================================
// 3. Engine B (`CompactEdges-Var234` + `ZeroH` + `PackedEdgeRule32-Sentinel` +
//    SoA/AoS Ablation): Powers `C51-Compact3-v1`, `C52-v3-UnpSoA`,
//    `C52-v3-UnpAoS`, & `C52-SuperDropIn-v3`!
// ============================================================================

mod engine_c52 {
    use super::*;
    const MAX_RULE_WORDS: usize = 4;
    const SPELLING_STEP: usize = 4096;
    const INLINE_STEP: usize = 128;

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
        #[inline(always)] fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) { let s = &a[id as usize * 4..id as usize * 4 + 4]; (s[0] >> 16, s[1], s[2], s[3]) }
    }
    impl Links for Wide {
        const STRIDE: usize = 6; const IS_NARROW: bool = false; const BOTTOM: usize = 2;
        #[inline(always)] fn prev(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side] }
        #[inline(always)] fn next(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side + 1] }
        #[inline(always)] fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side] = to; }
        #[inline(always)] fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side + 1] = to; }
        #[inline(always)] fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) { let s = &a[id as usize * 6..id as usize * 6 + 6]; (s[1], s[2], s[4], s[5]) }
    }

    struct Elim<'n, 'r, L: Links> {
        nodes_buf: &'n mut [u32],
        rules_buf: &'r mut [u32],
        rule_top: usize,
        nodes: u32,
        id_overflow: bool,
        _l: std::marker::PhantomData<L>,
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
                let (next, prev, w) = (L::next(self.nodes_buf, first, 1), L::prev(self.nodes_buf, last, 1), self.width(first));
                hi -= self.width(last); self.set_bottom(first, lo); self.set_bottom(last, hi); lo += w;
                if next == last { assert_eq!(lo, hi); return; }
                first = next; last = prev;
            }
            if first != 0 { self.set_bottom(first, lo); lo += self.width(first); }
            assert_eq!(lo, hi);
        }
        #[inline(always)] fn node(&mut self, w: u32, wd: u32) -> u32 { let id = self.nodes; self.nodes += 1; self.set_width(id, w); self.set_word(id, wd); id }
        #[inline(always)] fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
            if list.1 == 0 { list.0 = id; } else { L::set_next(self.nodes_buf, list.1, side, id); }
            L::set_prev(self.nodes_buf, id, side, list.1); list.1 = id;
        }
        fn close(&mut self, side: usize, a: (u32, u32), b: (u32, u32)) {
            let mut prev = 0;
            for (h, t) in [a, b] { if t == 0 { continue; } L::set_next(self.nodes_buf, prev, side, h); L::set_prev(self.nodes_buf, h, side, prev); prev = t; }
            L::set_next(self.nodes_buf, prev, side, 0); L::set_prev(self.nodes_buf, 0, side, prev);
        }
        #[inline(always)] fn cut(&mut self, id: u32, side: usize) { let (p, n) = (L::prev(self.nodes_buf, id, side), L::next(self.nodes_buf, id, side)); L::set_next(self.nodes_buf, p, side, n); L::set_prev(self.nodes_buf, n, side, p); }
        #[inline(always)] fn pop_tail(&mut self, id: u32, side: usize) { let p = L::prev(self.nodes_buf, id, side); L::set_next(self.nodes_buf, p, side, 0); L::set_prev(self.nodes_buf, 0, side, p); }
        #[inline(always)] fn insert(&mut self, at: u32, id: u32, side: usize) {
            let n = L::next(self.nodes_buf, at, side); L::set_prev(self.nodes_buf, id, side, at); L::set_next(self.nodes_buf, id, side, n); L::set_next(self.nodes_buf, at, side, id); L::set_prev(self.nodes_buf, n, side, id);
        }
        #[inline(always)] fn len_handle(&self, h: u32) -> u32 { let g = h >> ID_BITS; if g < SATURATED { g } else { self.rules_buf[(h & ID_MASK) as usize + 2] } }
        #[inline(always)] fn short_payload_h(&self, h: u32) -> u128 {
            let (g, id) = (h >> ID_BITS, h & ID_MASK);
            if g <= 3 { return u128::from(id); }
            let r = id as usize;
            if g <= 8 { let sl = &self.rules_buf[r..r + 2]; u128::from(sl[0]) | (u128::from(sl[1]) << 32) }
            else if g <= 12 { let sl = &self.rules_buf[r..r + 3]; u128::from(sl[0]) | (u128::from(sl[1]) << 32) | (u128::from(sl[2]) << 64) }
            else { let sl = &self.rules_buf[r..r + 4]; u128::from(sl[0]) | (u128::from(sl[1]) << 32) | (u128::from(sl[2]) << 64) | (u128::from(sl[3]) << 96) }
        }
        #[inline(always)] fn join(&mut self, left: u32, right: u32) -> u32 {
            if left == 0 { return right; } if right == 0 { return left; }
            let ll = left >> ID_BITS; let sum = ll + (right >> ID_BITS);
            if sum <= 3 { return (sum << ID_BITS) | (left & ID_MASK) | ((right & ID_MASK) << (8 * ll)); }
            let t = self.rule_top; let id = t as u32;
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
                self.rule_top = t + 3; (sum << ID_BITS) | id
            } else {
                let len = if sum == SATURATED { sum } else { self.len_handle(left) + self.len_handle(right) };
                let out = &mut self.rules_buf[t..t + 4];
                out[0] = left; out[1] = right; out[2] = len; out[3] = u32::MAX;
                self.rule_top = t + 4; (SATURATED << ID_BITS) | id
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
            let limit = self.rules_buf.len().min(max_id_top);
            let done = loop {
                if left == 0 { break true; }
                if self.rule_top + MAX_RULE_WORDS > limit {
                    if self.rule_top + MAX_RULE_WORDS > max_id_top { self.id_overflow = true; }
                    break false;
                }
                let (x, y) = (self.nodes_buf[0] & 0xFFFF, self.nodes_buf[1] & 0xFFFF);
                let (xi, yi) = (x as usize * 4, y as usize * 4);
                let (x0, x1, aw, wx) = (self.nodes_buf[xi], self.nodes_buf[xi + 1], self.nodes_buf[xi + 2], self.nodes_buf[xi + 3]);
                if x == y {
                    left -= aw;
                    let (px0, px1) = (x0 & 0xFFFF, x1 & 0xFFFF);
                    self.nodes_buf[px0 as usize * 4] &= 0xFFFF; self.nodes_buf[0] = (self.nodes_buf[0] & 0xFFFF_0000) | px0;
                    self.nodes_buf[px1 as usize * 4 + 1] &= 0xFFFF; self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | px1;
                    continue;
                }
                let (y0, y1, bw, wy) = (self.nodes_buf[yi], self.nodes_buf[yi + 1], self.nodes_buf[yi + 2], self.nodes_buf[yi + 3]);
                let w = aw.min(bw); left -= w;
                let word = self.join(wx, wy);
                if aw <= bw {
                    let px = x0 & 0xFFFF;
                    self.nodes_buf[px as usize * 4] &= 0xFFFF; self.nodes_buf[0] = (self.nodes_buf[0] & 0xFFFF_0000) | px;
                    if aw < bw {
                        let y_next = if y == px { 0 } else { y0 >> 16 };
                        self.nodes_buf[xi] = y | (y_next << 16); self.nodes_buf[xi + 3] = word;
                        self.nodes_buf[yi] = (y0 & 0xFFFF) | (x << 16); self.nodes_buf[yi + 2] = bw - aw;
                        let yni = y_next as usize * 4; self.nodes_buf[yni] = (self.nodes_buf[yni] & 0xFFFF_0000) | x;
                    } else {
                        let y_next = if y == px { 0 } else { y0 >> 16 }; let y_prev = y0 & 0xFFFF;
                        self.nodes_buf[xi] = y_prev | (y_next << 16); self.nodes_buf[xi + 3] = word;
                        let ypi = y_prev as usize * 4; self.nodes_buf[ypi] = (self.nodes_buf[ypi] & 0xFFFF) | (x << 16);
                        let yni = y_next as usize * 4; self.nodes_buf[yni] = (self.nodes_buf[yni] & 0xFFFF_0000) | x;
                        let py = y1 & 0xFFFF; self.nodes_buf[py as usize * 4 + 1] &= 0xFFFF; self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | py;
                    }
                } else {
                    let py = y1 & 0xFFFF; self.nodes_buf[py as usize * 4 + 1] &= 0xFFFF; self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | py;
                    let x_next = if x == py { 0 } else { x1 >> 16 };
                    self.nodes_buf[yi + 1] = x | (x_next << 16); self.nodes_buf[yi + 3] = word;
                    self.nodes_buf[xi + 1] = (x1 & 0xFFFF) | (y << 16); self.nodes_buf[xi + 2] = aw - bw;
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
            let limit = self.rules_buf.len().min(max_id_top);
            let done = loop {
                if left == 0 { break true; }
                if self.rule_top + room > limit {
                    if self.rule_top + room > max_id_top { self.id_overflow = true; }
                    break false;
                }
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
                let w = aw.min(bw); left -= w;
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
        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn append_tail<const TRACK_MEMO: bool>(
            &mut self, handle: u32, istk: &mut [u32; INLINE_STACK], ovf: &mut Vec<u32>,
            dest: &mut Vec<u8>, memo_rec: &mut [u32; MEMO_REC_CAP], memo_n: &mut usize, memo_ovf: &mut bool,
        ) {
            let (mut sp, mut used_heap, mut h) = (0usize, false, handle);
            loop {
                let g = h >> ID_BITS;
                if g < INLINE as u32 {
                    let b = self.short_payload_h(h).to_le_bytes();
                    dest.extend_from_slice(&b[..g as usize]);
                    if used_heap && !ovf.is_empty() { h = ovf.pop().unwrap(); continue; }
                    if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                    return;
                }
                let r = (h & ID_MASK) as usize;
                let m_off = if g < SATURATED { r + 2 } else { r + 3 };
                let tag = self.rules_buf[m_off];
                if tag != u32::MAX {
                    let st = (tag & 0x7fff_ffff) as usize;
                    let rl = if g < SATURATED { g as usize } else { self.rules_buf[r + 2] as usize };
                    dest.extend_from_within(st..st + rl);
                    if used_heap && !ovf.is_empty() { h = ovf.pop().unwrap(); continue; }
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
                        if used_heap && !ovf.is_empty() { h = ovf.pop().unwrap(); continue; }
                        if sp > 0 { sp -= 1; h = istk[sp]; continue; }
                        return;
                    }
                    h = right;
                } else {
                    if sp < INLINE_STACK { istk[sp] = right; sp += 1; }
                    else { if !used_heap { ovf.clear(); used_heap = true; } ovf.push(right); }
                    h = left;
                }
            }
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
                if self.ns < self.cap_strips {
                    let at = 3 * self.ns;
                    if self.edges.len() < at + 3 {
                        let need = (at + 3).div_ceil(4096) * 4096;
                        if need > self.edges.capacity() { self.edges.reserve_exact(need - self.edges.len()); }
                        self.edges.resize(need, 0);
                    }
                    let ta = self.queues.thin_of(a);
                    self.edges[at..at + 3].copy_from_slice(&[x, ta + f.thick_rows[a], (w << 8) | u32::from(c)]);
                }
                self.ns += 1; f.thick[a] += 1; f.thick_rows[a] += w;
            }
        }
    }

    struct Dictionary<'a> {
        pairs: &'a mut [u32; 512],
        contents: &'a mut [u32; 2048],
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
            if self.pairs[slot] == handle && (val >> 24) == u32::from(byte) { return val & 0x00FF_FFFF; }
            let c = self.code_miss(e, handle, handle & ID_MASK, byte);
            self.pairs[slot] = handle; self.pairs[slot + 1] = (u32::from(byte) << 24) | c; c
        }
        fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8) -> u32 {
            let height = (handle >> ID_BITS) as usize;
            if height < INLINE {
                let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
                let entry = v.to_le_bytes(); let len = (height + 1) as u8;
                let hh = ((v as u64) ^ ((v >> 64) as u64).rotate_left(29) ^ (u64::from(len) << 56)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                let hh = hh ^ (hh >> 30);
                let tag = (((hh >> 56) as u32) | 0x80) << 24;
                let idx = (hh as usize) & 2047;
                let ent = self.contents[idx];
                if (ent & 0xFF00_0000) == tag && { let old = (ent & 0x00FF_FFFF) as usize; self.lens[old] == len && self.inline16[old] == entry } {
                    return ent & 0x00FF_FFFF;
                }
                ensure_step_inline(self.inline16, self.lens, INLINE_STEP);
                let c = self.lens.len() as u32; self.inline16.push(entry); self.lens.push(len); self.contents[idx] = tag | c;
                return c;
            }
            let st = self.spelling.len();
            let g = handle >> ID_BITS;
            let r = id as usize;
            let m_off = if g < SATURATED { r + 2 } else { r + 3 };
            let tag_r = e.rules_buf[m_off];
            let rlen = if g < SATURATED { g as usize } else { e.rules_buf[r + 2] as usize };
            ensure_step_u8(self.spelling, rlen + 1, SPELLING_STEP);
            let (mut memo_n, mut memo_ovf) = (0usize, false);
            let was_memo = tag_r != u32::MAX;
            if was_memo {
                let p0 = (tag_r & 0x7fff_ffff) as usize;
                self.spelling.extend_from_within(p0..p0 + rlen);
            } else {
                e.append_tail::<true>(handle, self.istk, self.stack, self.spelling, self.memo_rec, &mut memo_n, &mut memo_ovf);
            }
            self.spelling.push(byte);
            let entry = long_slot(st as u32, self.spelling.len() as u32);
            let bytes = &self.spelling[st..];
            let hh = spelling_hash_long(bytes);
            let tag = (((hh >> 56) as u32) | 0x80) << 24;
            let idx = (hh as usize) & 2047;
            let ent = self.contents[idx];
            if (ent & 0xFF00_0000) == tag && { let old = (ent & 0x00FF_FFFF) as usize; self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes } {
                if was_memo { self.spelling.truncate(st); }
                else if !memo_ovf {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    let old_st = u32::from_le_bytes(self.inline16[old][..4].try_into().unwrap());
                    let delta = (st as u32) - old_st;
                    for &rm in &self.memo_rec[..memo_n] { e.rules_buf[rm as usize] -= delta; }
                    self.spelling.truncate(st);
                } else { self.spelling.pop(); }
                return ent & 0x00FF_FFFF;
            }
            ensure_step_inline(self.inline16, self.lens, INLINE_STEP);
            let c = self.lens.len() as u32; self.inline16.push(entry); self.lens.push(LONG); self.contents[idx] = tag | c; c
        }
    }

    /// `S5_MODE`: `0` = `PackedEdgeRule32-Sentinel` (single-`u32`), `1` = Unpacked SoA (`edges` + `rules16`),
    /// `2` = Unpacked AoS (`(u32, u16)` 8 B/row).
    #[derive(Default)]
    pub struct Kernel<const TIERED_TINY: bool, const S5_MODE: u8> {
        spill: outq::Spill,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        rules16: Vec<u16>,
        aos: Vec<(u32, u16)>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        stack: Vec<u32>,
        predecessor: Option<u32>,
        pub force_band_vec: bool,
        pub force_sentinel_cap: Option<u32>,
    }

    impl<const TIERED_TINY: bool, const S5_MODE: u8> Kernel<TIERED_TINY, S5_MODE> {
        pub fn scratch_report(&self) -> (usize, usize, String) {
            let sp = self.spill.capacity_bytes();
            let bd = 8 * self.bands.capacity();
            let s = self.spelling.capacity();
            let ed = 4 * self.edges.capacity();
            let ex = 4 * self.extra.capacity() + 2 * self.rules16.capacity() + 8 * self.aos.capacity();
            let il = 16 * self.inline16.capacity() + self.lens.capacity();
            let st = 4 * self.stack.capacity();
            let tot = sp + bd + s + ed + ex + il + st;
            (tot, ed, format!("ed={:.1}K ex={:.1}K sp={:.1}K il={:.1}K q={:.1}K", ed as f64 / 1024.0, ex as f64 / 1024.0, s as f64 / 1024.0, il as f64 / 1024.0, sp as f64 / 1024.0))
        }

        #[inline(never)]
        pub fn invert_tiny_fl(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
            if n == 0 || (p as usize) >= n || records.len() > 16_000_000 { return false; }
            if records.len() == 1 {
                if usize::from(records[0] & 255) != n { return false; }
                out.clear(); out.resize(n, (records[0] >> 8) as u8); return true;
            }
            if TIERED_TINY {
                if n <= 64 { return invert_tiny_fl_64(records, n, p, out); }
                if n <= 256 { return invert_tiny_fl_256(records, n, p, out); }
                if n <= 512 { return invert_tiny_fl_512(records, n, p, out); }
                if n <= 1536 { return invert_tiny_fl_1536(records, n, p, out); }
            }
            if self.edges.len() < n { let add = n - self.edges.len(); self.edges.reserve_exact(add); self.edges.resize(n, 0); }
            invert_tiny_fl_with_buf(records, n, p, &mut self.edges[..n], out)
        }

        #[inline(never)]
        pub fn try_run_const<const LIM: u32>(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, bi_mode: BiMode, out: &mut Vec<u8>) -> Option<Counts> {
            let nrec = records.len();
            if n == 0 || n > 900_000 || (p as usize) >= n || nrec > 16_000_000 { return None; }
            out.resize(n + INLINE, 0);
            let (ns, f, heads, over, extra_127, tot_pos) = {
                let mut pass = Pass {
                    queues: outq::Queues::new(&mut out[..], &mut self.spill, cap.m.min(n as u32)),
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
        fn finish(&mut self, f: &First, ns: usize, n: usize, r_max: u32, heads: &outq::Heads, bi_mode: BiMode, out: &mut Vec<u8>) -> Option<Counts> {
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            if all.iter().sum::<u32>() != n as u32 { return None; }
            let first = starts(all);
            let mut write = starts(f.thick);
            let m = f.m as usize;
            let most = 3 * ns + 3;
            let use_narrow = most <= NARROW_NODES;
            let stride = if use_narrow { Narrow::STRIDE } else { Wide::STRIDE };
            self.bands.clear(); self.spelling.clear();
            let tail_offset = (heads.chunks as usize).min(out.len() / outq::BYTES) * outq::BYTES;
            let use_zero_hdr = out.len() - tail_offset >= 16 * ns;
            let hdr_w = if use_zero_hdr { 0 } else { 4 * ns };
            let arena_est = hdr_w + stride * most;
            let need = arena_est.max(m);
            if self.edges.len() < need { let add = need - self.edges.len(); self.edges.reserve_exact(add); self.edges.resize(need, 0); }
            let (ok_ind, cursors, used_tail, nbands, live_words) = if use_narrow {
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

            let s5_live;
            let used_bytes;
            if S5_MODE == 1 && use_xor && !wide && self.lens.len() <= u16::MAX as usize {
                let pred = self.wire_bi_soa(m, f.mark, f.mark_byte, &cursors, nbands, used_tail, out_tail, out_q, heads);
                s5_live = 4 * m + 2 * m + 16 * self.inline16.len() + self.lens.len() + self.spelling.len();
                used_bytes = 4 * live_words.max(m) + 2 * m + 8 * self.bands.len() + self.spelling.len() + 16 * self.inline16.len() + self.lens.len() + self.spill.used_bytes(heads.chunks);
                Self::emit_bi_soa(&self.edges[..m], &self.rules16[..m], &self.spelling, &self.inline16, &self.lens, n, f.mark, pred, out);
            } else if S5_MODE == 2 && use_xor && !wide && self.lens.len() <= u16::MAX as usize {
                let pred = self.wire_bi_aos(m, f.mark, &cursors, nbands, used_tail, out_tail, out_q, heads);
                s5_live = 8 * m + 16 * self.inline16.len() + self.lens.len() + self.spelling.len();
                used_bytes = 4 * live_words.max(m) + 8 * m + 8 * self.bands.len() + self.spelling.len() + 16 * self.inline16.len() + self.lens.len() + self.spill.used_bytes(heads.chunks);
                Self::emit_bi_aos(&self.aos[..m], &self.spelling, &self.inline16, &self.lens, n, f.mark, pred, out);
            } else {
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
                s5_live = 4 * m + 4 * self.extra.len() + 16 * self.inline16.len() + self.lens.len() + self.spelling.len();
                used_bytes = 4 * live_words.max(m) + 4 * self.extra.len() + 8 * self.bands.len() + self.spelling.len() + 16 * self.inline16.len() + self.lens.len() + self.spill.used_bytes(heads.chunks);
                if let Some(before) = self.predecessor {
                    if !wide {
                        Self::emit_bi(&self.edges[..m], &self.spelling, &self.inline16, &self.lens, n, f.mark, before, bits, out);
                    } else {
                        Self::emit_bi_sentinel(&self.edges[..m], &self.extra, &self.spelling, &self.inline16, &self.lens, n, f.mark, before, bits, sentinel, out);
                    }
                } else {
                    Self::emit_uni(&self.edges[..m], &self.extra, &self.spelling, &self.inline16, &self.lens, n, f.mark, bits, sentinel, out);
                }
            }
            Some(Counts {
                m, bits, num_codes: self.lens.len(), wide, used_bi: use_xor, used_tail,
                spelling_used: self.spelling.len(), used_bytes, s5_live_bytes: s5_live, arena_est_gt_m: arena_est > m,
            })
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_tail(&mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, cursors: &[usize; 256], nbands: usize, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads) -> u32 {
            let edges = &mut self.edges[..m]; edges.fill(0);
            let (bs, _) = out_tail.as_chunks::<8>();
            let mut pred = mark; let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &raw in &bs[cursors[a]..end] { let b = Band(u64::from_le_bytes(raw)); q.wire_xor_nomark(edges, b.dst() as usize, b.end() as usize, b.code() << bits); }
                } else {
                    for &raw in &bs[cursors[a]..end] { let b = Band(u64::from_le_bytes(raw)); q.wire_xor(edges, b.dst() as usize, b.end() as usize, b.code() << bits, mark, &mut pred); }
                }
            }
            pred
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_vec(&mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, cursors: &[usize; 256], out_q: &[u8], heads: &outq::Heads) -> u32 {
            let edges = &mut self.edges[..m]; edges.fill(0);
            let mut pred = mark; let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &b in &self.bands[cursors[a]..end] { q.wire_xor_nomark(edges, b.dst() as usize, b.end() as usize, b.code() << bits); }
                } else {
                    for &b in &self.bands[cursors[a]..end] { q.wire_xor(edges, b.dst() as usize, b.end() as usize, b.code() << bits, mark, &mut pred); }
                }
            }
            pred
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_sentinel(&mut self, m: usize, mark: u32, mark_byte: u8, bits: u32, sentinel: u32, cursors: &[usize; 256], nbands: usize, used_tail: bool, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads) -> u32 {
            let edges = &mut self.edges[..m]; edges.fill(0);
            let extra = &mut self.extra[..m];
            let mut pred = mark; let mb = usize::from(mark_byte);
            if used_tail {
                let (bs, _) = out_tail.as_chunks::<8>();
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                    let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &raw in &bs[cursors[a]..end] {
                        let b = Band(u64::from_le_bytes(raw)); let (dst, sp, bc) = (b.dst() as usize, b.end() as usize, b.code());
                        let pack = if bc < sentinel { bc << bits } else { extra[dst..dst + sp].fill(bc); sentinel << bits };
                        if a != mb { q.wire_xor_nomark(edges, dst, sp, pack); } else { q.wire_xor(edges, dst, sp, pack, mark, &mut pred); }
                    }
                }
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let (dst, sp, bc) = (b.dst() as usize, b.end() as usize, b.code());
                        let pack = if bc < sentinel { bc << bits } else { extra[dst..dst + sp].fill(bc); sentinel << bits };
                        if a != mb { q.wire_xor_nomark(edges, dst, sp, pack); } else { q.wire_xor(edges, dst, sp, pack, mark, &mut pred); }
                    }
                }
            }
            pred
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_soa(&mut self, m: usize, mark: u32, mark_byte: u8, cursors: &[usize; 256], nbands: usize, used_tail: bool, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads) -> u32 {
            let edges = &mut self.edges[..m]; edges.fill(0);
            self.rules16.resize(m, 0);
            let rules16 = &mut self.rules16[..m];
            let mut pred = mark; let mb = usize::from(mark_byte);
            if used_tail {
                let (bs, _) = out_tail.as_chunks::<8>();
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                    let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                    if a != mb {
                        for &raw in &bs[cursors[a]..end] { let b = Band(u64::from_le_bytes(raw)); let (dst, sp, bc) = (b.dst() as usize, b.end() as usize, b.code() as u16); rules16[dst..dst + sp].fill(bc); q.wire_xor_nomark(edges, dst, sp, 0); }
                    } else {
                        for &raw in &bs[cursors[a]..end] { let b = Band(u64::from_le_bytes(raw)); let (dst, sp, bc) = (b.dst() as usize, b.end() as usize, b.code() as u16); rules16[dst..dst + sp].fill(bc); q.wire_xor(edges, dst, sp, 0, mark, &mut pred); }
                    }
                }
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                    if a != mb {
                        for &b in &self.bands[cursors[a]..end] { let (dst, sp, bc) = (b.dst() as usize, b.end() as usize, b.code() as u16); rules16[dst..dst + sp].fill(bc); q.wire_xor_nomark(edges, dst, sp, 0); }
                    } else {
                        for &b in &self.bands[cursors[a]..end] { let (dst, sp, bc) = (b.dst() as usize, b.end() as usize, b.code() as u16); rules16[dst..dst + sp].fill(bc); q.wire_xor(edges, dst, sp, 0, mark, &mut pred); }
                    }
                }
            }
            pred
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_aos(&mut self, m: usize, mark: u32, cursors: &[usize; 256], nbands: usize, used_tail: bool, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads) -> u32 {
            self.aos.clear(); self.aos.resize(m, (0, 0));
            let aos = &mut self.aos[..m];
            let mut pred = mark;
            if used_tail {
                let (bs, _) = out_tail.as_chunks::<8>();
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                    let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &raw in &bs[cursors[a]..end] { let b = Band(u64::from_le_bytes(raw)); q.wire_xor_aos(aos, b.dst() as usize, b.end() as usize, b.code() as u16, mark, &mut pred); }
                }
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &b in &self.bands[cursors[a]..end] { q.wire_xor_aos(aos, b.dst() as usize, b.end() as usize, b.code() as u16, mark, &mut pred); }
                }
            }
            pred
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_tail(&mut self, m: usize, wide: bool, bits: u32, sentinel: u32, cursors: &[usize; 256], nbands: usize, out_tail: &[u8], out_q: &[u8], heads: &outq::Heads) {
            let edges = &mut self.edges[..m]; let (bs, _) = out_tail.as_chunks::<8>();
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
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
        fn wire_uni_vec(&mut self, m: usize, wide: bool, bits: u32, sentinel: u32, cursors: &[usize; 256], out_q: &[u8], heads: &outq::Heads) {
            let edges = &mut self.edges[..m];
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut q = outq::Reader::new(out_q, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let (sp, dst, bc) = (b.end() as usize, b.dst() as usize, b.code());
                    let pack = if !wide || bc < sentinel { bc << bits } else { self.extra[dst..dst + sp].fill(bc); sentinel << bits };
                    q.wire(&mut edges[dst..dst + sp], pack);
                }
            }
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_with<L: Links>(&mut self, ns: usize, n: u32, m: u32, thin: &[u32; 256], first: &[u32; 256], write: &mut [u32; 256], most: usize, use_zero_hdr: bool, out_tail: &mut [u8]) -> (bool, [usize; 256], bool, usize, usize) {
            let nodes_words = L::STRIDE * most;
            let hdr_words = if use_zero_hdr { 0 } else { 4 * ns };
            let mut widest = 0u32;
            let (base, nodes, open_nodes) = if use_zero_hdr {
                let (ot_words, _) = out_tail.as_chunks_mut::<4>();
                let (strips_ot, rest_ot) = ot_words.split_at_mut(3 * ns);
                let image_ot = &mut rest_ot[..ns];
                for (d, &s) in strips_ot.iter_mut().zip(&self.edges[..3 * ns]) { *d = s.to_ne_bytes(); }
                let (nodes_sl, rules_dummy) = self.edges.split_at_mut(nodes_words);
                let mut e = Elim::<L> { nodes_buf: nodes_sl, rules_buf: rules_dummy, rule_top: 0, nodes: 0, id_overflow: false, _l: std::marker::PhantomData };
                e.node(0, 0); e.nodes = ns as u32 + 1;
                let (mut terms, mut ord) = ([(0, 0); 2], [(0, 0); 2]);
                let mut x = 0u32;
                for i in 0..ns {
                    let sw = &mut strips_ot[3 * i..3 * i + 3];
                    let (from, to0, wbyte) = (u32::from_ne_bytes(sw[0]), u32::from_ne_bytes(sw[1]), u32::from_ne_bytes(sw[2]));
                    let (byte, width) = ((wbyte & 0xFF) as u8, wbyte >> 8);
                    let a = usize::from(byte);
                    sw[1] = (to0 + first[a]).to_ne_bytes();
                    let di = write[a] as usize; image_ot[di] = (i as u32).to_ne_bytes(); write[a] = di as u32 + 1;
                    let sid = i as u32 + 1; e.set_width(sid, width); e.set_word(sid, literal(byte));
                    widest = widest.max(width);
                    if x < from { let gw = from - x; widest = widest.max(gw); let gid = e.node(gw, 0); e.append_to(&mut ord[0], gid, 0); e.append_to(&mut terms[1], gid, 1); }
                    e.append_to(&mut ord[0], sid, 0); x = from + width;
                }
                if x < n { widest = widest.max(n - x); let id = e.node(n - x, 0); e.append_to(&mut ord[0], id, 0); e.append_to(&mut terms[1], id, 1); }
                x = 0;
                for slot in image_ot.iter() {
                    let i = u32::from_ne_bytes(*slot);
                    let sw = &strips_ot[3 * i as usize..3 * i as usize + 3];
                    let (to, width) = (u32::from_ne_bytes(sw[1]), u32::from_ne_bytes(sw[2]) >> 8);
                    if x < to { widest = widest.max(to - x); let id = e.node(to - x, 0); e.append_to(&mut ord[1], id, 1); e.append_to(&mut terms[0], id, 0); }
                    e.append_to(&mut ord[1], i + 1, 1); x = to + width;
                }
                if x < n { widest = widest.max(n - x); let id = e.node(n - x, 0); e.append_to(&mut ord[1], id, 1); e.append_to(&mut terms[0], id, 0); }
                let open_nodes = e.nodes as usize - 1;
                for side in 0..2 { e.close(side, terms[side], ord[side]); }
                (L::STRIDE * e.nodes as usize, e.nodes, open_nodes)
            } else {
                let (hdr, arena) = self.edges.split_at_mut(4 * ns);
                let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
                let (nodes_sl, rules_dummy) = arena.split_at_mut(nodes_words);
                let mut e = Elim::<L> { nodes_buf: nodes_sl, rules_buf: rules_dummy, rule_top: 0, nodes: 0, id_overflow: false, _l: std::marker::PhantomData };
                e.node(0, 0); e.nodes = ns as u32 + 1;
                let (mut terms, mut ord) = ([(0, 0); 2], [(0, 0); 2]);
                let mut x = 0u32;
                for i in 0..ns {
                    let sw = &mut strips_raw[3 * i..3 * i + 3];
                    let (from, wbyte) = (sw[0], sw[2]);
                    let (byte, width) = ((wbyte & 0xFF) as u8, wbyte >> 8);
                    let a = usize::from(byte);
                    sw[1] += first[a];
                    let di = write[a] as usize; image_order[di] = i as u32; write[a] = di as u32 + 1;
                    let sid = i as u32 + 1; e.set_width(sid, width); e.set_word(sid, literal(byte));
                    widest = widest.max(width);
                    if x < from { let gw = from - x; widest = widest.max(gw); let gid = e.node(gw, 0); e.append_to(&mut ord[0], gid, 0); e.append_to(&mut terms[1], gid, 1); }
                    e.append_to(&mut ord[0], sid, 0); x = from + width;
                }
                if x < n { widest = widest.max(n - x); let id = e.node(n - x, 0); e.append_to(&mut ord[0], id, 0); e.append_to(&mut terms[1], id, 1); }
                x = 0;
                for &i in image_order.iter() {
                    let sw = &strips_raw[3 * i as usize..3 * i as usize + 3];
                    let (to, width) = (sw[1], sw[2] >> 8);
                    if x < to { widest = widest.max(to - x); let id = e.node(to - x, 0); e.append_to(&mut ord[1], id, 1); e.append_to(&mut terms[0], id, 0); }
                    e.append_to(&mut ord[1], i + 1, 1); x = to + width;
                }
                if x < n { widest = widest.max(n - x); let id = e.node(n - x, 0); e.append_to(&mut ord[1], id, 1); e.append_to(&mut terms[0], id, 0); }
                let open_nodes = e.nodes as usize - 1;
                for side in 0..2 { e.close(side, terms[side], ord[side]); }
                (L::STRIDE * e.nodes as usize, e.nodes, open_nodes)
            };
            let spirals = widest > CERTIFIED_WIDTH;
            let (mut remaining, mut rule_top) = (n, 0usize);
            loop {
                let (nodes_sl, rules_sl) = self.edges[hdr_words..].split_at_mut(base);
                let mut e = Elim::<L> { nodes_buf: nodes_sl, rules_buf: rules_sl, rule_top, nodes, id_overflow: false, _l: std::marker::PhantomData };
                let done = if spirals { e.eliminate::<true>(&mut remaining) } else { e.eliminate::<false>(&mut remaining) };
                rule_top = e.rule_top;
                if e.id_overflow { return (false, [0; 256], false, 0, 0); }
                if done { break; }
                grow_edges_step(&mut self.edges, hdr_words + base + rule_top + SPIRAL_RULES * MAX_RULE_WORDS);
            }
            if use_zero_hdr && ns >= 64 && self.edges.len() < base + rule_top + CACHE_WORDS {
                let need_c = base + rule_top + CACHE_WORDS;
                self.edges.reserve_exact(need_c - self.edges.len());
                self.edges.resize(need_c, 0);
            }
            let (header, arena) = self.edges.split_at_mut(hdr_words);
            let (live, tail_arena) = arena.split_at_mut(base + rule_top);
            let (nodes_sl, rules_sl) = live.split_at_mut(base);
            let mut e = Elim::<L> { nodes_buf: nodes_sl, rules_buf: rules_sl, rule_top, nodes, id_overflow: false, _l: std::marker::PhantomData };
            e.assign_bottom(m);
            let cache = if header.len() >= CACHE_WORDS { header } else { tail_arena };
            begin_inline(&mut self.inline16, &mut self.lens);
            let max_bands = open_nodes + 256;
            let use_tail = !self.force_band_vec && out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec = [0u32; MEMO_REC_CAP];
            if cache.len() < CACHE_WORDS {
                let (bs, _) = out_tail.as_chunks_mut::<8>();
                let (mut id, mut in_node, mut k) = (L::next(e.nodes_buf, 0, 0), 0, 0);
                for a in 0..256 {
                    cursors[a] = if use_tail { nbands } else { self.bands.len() };
                    let end = k + thin[a];
                    while k < end {
                        let (nid, nb, nw, nwd) = L::band_node(e.nodes_buf, id);
                        let width = (end - k).min(nw - in_node);
                        let dst = nb + in_node;
                        let code = if nwd == 0 { a as u32 } else {
                            let h = (nwd >> ID_BITS) as usize;
                            let (entry, len) = if h < INLINE {
                                ((e.short_payload_h(nwd) | (u128::from(a as u8) << (8 * h))).to_le_bytes(), (h + 1) as u8)
                            } else {
                                let st = self.spelling.len() as u32;
                                let total = e.len_handle(nwd) as usize;
                                ensure_step_u8(&mut self.spelling, total + 1, SPELLING_STEP);
                                let (mut dn, mut do_v) = (0usize, false);
                                e.append_tail::<false>(nwd, &mut istk, &mut self.stack, &mut self.spelling, &mut memo_rec, &mut dn, &mut do_v);
                                self.spelling.push(a as u8);
                                (long_slot(st, self.spelling.len() as u32), LONG)
                            };
                            ensure_step_inline(&mut self.inline16, &mut self.lens, INLINE_STEP);
                            let c = self.lens.len() as u32; self.inline16.push(entry); self.lens.push(len); c
                        };
                        if use_tail { bs[nbands] = Band::new(width, dst, code).0.to_le_bytes(); nbands += 1; }
                        else { self.bands.push(Band::new(width, dst, code)); }
                        in_node += width; k += width;
                        if in_node == nw { id = nid; in_node = 0; }
                    }
                }
            } else {
                cache[..CACHE_WORDS].fill(0);
                let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
                let mut dict = Dictionary {
                    pairs: p_sl.try_into().unwrap(), contents: c_sl.try_into().unwrap(),
                    inline16: &mut self.inline16, lens: &mut self.lens, spelling: &mut self.spelling,
                    istk: &mut istk, memo_rec: &mut memo_rec, stack: &mut self.stack,
                };
                let mut id = L::next(e.nodes_buf, 0, 0);
                let (mut nid, mut nb, mut nw, mut nwd) = if id != 0 { L::band_node(e.nodes_buf, id) } else { (0, 0, 0, 0) };
                let (mut in_node, mut k) = (0u32, 0u32);
                if use_tail {
                    let (band_slots, _) = out_tail[..max_bands * 8].as_chunks_mut::<8>();
                    for a in 0..256 {
                        cursors[a] = nbands;
                        let end = k + thin[a];
                        let (mut last_w, mut last_c) = (0u32, 0u32);
                        while k < end {
                            let width = (end - k).min(nw - in_node);
                            let dst = nb + in_node;
                            let code = if nwd == 0 { a as u32 } else if nwd == last_w { last_c } else {
                                let c = dict.code(&mut e, nwd, a as u8); last_w = nwd; last_c = c; c
                            };
                            band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes(); nbands += 1;
                            in_node += width; k += width;
                            if in_node == nw { id = nid; in_node = 0; if id != 0 { (nid, nb, nw, nwd) = L::band_node(e.nodes_buf, id); } }
                        }
                    }
                } else {
                    for a in 0..256 {
                        cursors[a] = self.bands.len();
                        let end = k + thin[a];
                        let (mut last_w, mut last_c) = (0u32, 0u32);
                        while k < end {
                            let width = (end - k).min(nw - in_node);
                            let dst = nb + in_node;
                            let code = if nwd == 0 { a as u32 } else if nwd == last_w { last_c } else {
                                let c = dict.code(&mut e, nwd, a as u8); last_w = nwd; last_c = c; c
                            };
                            self.bands.push(Band::new(width, dst, code));
                            in_node += width; k += width;
                            if in_node == nw { id = nid; in_node = 0; if id != 0 { (nid, nb, nw, nwd) = L::band_node(e.nodes_buf, id); } }
                        }
                    }
                }
            }
            (true, cursors, use_tail, nbands, hdr_words + base + rule_top)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_uni(edges: &[u32], extra: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, bits: u32, sentinel: u32, out: &mut Vec<u8>) {
            let out_buf = &mut out[..n + INLINE];
            let (mut at, mask, mut j, has_ex) = (0usize, (1 << bits) - 1, mark, !extra.is_empty());
            for _ in 0..edges.len() {
                let edge = edges[j as usize];
                let raw = edge >> bits;
                let code = if has_ex && raw == sentinel { extra[j as usize] } else { raw } as usize;
                j = edge & mask;
                let len = lens[code];
                if len != LONG { out_buf[at..at + INLINE].copy_from_slice(&inline[code]); at += usize::from(len); }
                else { let w = long_word(&inline[code], spelling); let t = w.len().min(n - at); out_buf[at..at + t].copy_from_slice(&w[..t]); at += t; }
                if j == mark || at >= n { break; }
            }
            out.truncate(at.min(n)); repeat(n, out);
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi(edges: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, before: u32, bits: u32, out: &mut Vec<u8>) {
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

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi_soa(edges: &[u32], rules16: &[u16], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, before: u32, out: &mut Vec<u8>) {
            let out_buf = &mut out[..n];
            let (mut prev, mut front, mut back, mut next, mut lo, mut hi) = (before, mark, before, mark, 0usize, n);
            for _ in 0..=edges.len() / 2 {
                let ef = edges[front as usize]; let cf = rules16[front as usize] as usize; let single = front == back; let lf = lens[cf];
                if lf != LONG && hi - lo >= INLINE { out_buf[lo..lo + INLINE].copy_from_slice(&inline[cf]); lo += usize::from(lf); }
                else { let w = if lf == LONG { long_word(&inline[cf], spelling) } else { &inline[cf][..usize::from(lf)] }; let t = w.len().min(hi - lo); out_buf[lo..lo + t].copy_from_slice(&w[..t]); lo += t; }
                if single || lo == hi { break; }
                let eb = edges[back as usize]; let cb = rules16[back as usize] as usize; let lb = lens[cb];
                if lb != LONG && hi - lo >= INLINE {
                    let sh = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                    out_buf[hi - INLINE..hi].copy_from_slice(&sh.to_le_bytes()); hi -= usize::from(lb);
                } else { let w = if lb == LONG { long_word(&inline[cb], spelling) } else { &inline[cb][..usize::from(lb)] }; let t = w.len().min(hi - lo); out_buf[hi - t..hi].copy_from_slice(&w[w.len() - t..]); hi -= t; }
                (prev, front) = (front, ef ^ prev);
                (next, back) = (back, eb ^ next);
                if lo == hi || front == next { break; }
            }
            if lo < hi { out.copy_within(hi..n, lo); out.truncate(lo + (n - hi)); repeat(n, out); } else { out.truncate(n); }
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_bi_aos(aos: &[(u32, u16)], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, before: u32, out: &mut Vec<u8>) {
            let out_buf = &mut out[..n];
            let (mut prev, mut front, mut back, mut next, mut lo, mut hi) = (before, mark, before, mark, 0usize, n);
            for _ in 0..=aos.len() / 2 {
                let (ef, cf16) = aos[front as usize]; let cf = cf16 as usize; let single = front == back; let lf = lens[cf];
                if lf != LONG && hi - lo >= INLINE { out_buf[lo..lo + INLINE].copy_from_slice(&inline[cf]); lo += usize::from(lf); }
                else { let w = if lf == LONG { long_word(&inline[cf], spelling) } else { &inline[cf][..usize::from(lf)] }; let t = w.len().min(hi - lo); out_buf[lo..lo + t].copy_from_slice(&w[..t]); lo += t; }
                if single || lo == hi { break; }
                let (eb, cb16) = aos[back as usize]; let cb = cb16 as usize; let lb = lens[cb];
                if lb != LONG && hi - lo >= INLINE {
                    let sh = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                    out_buf[hi - INLINE..hi].copy_from_slice(&sh.to_le_bytes()); hi -= usize::from(lb);
                } else { let w = if lb == LONG { long_word(&inline[cb], spelling) } else { &inline[cb][..usize::from(lb)] }; let t = w.len().min(hi - lo); out_buf[hi - t..hi].copy_from_slice(&w[w.len() - t..]); hi -= t; }
                (prev, front) = (front, ef ^ prev);
                (next, back) = (back, eb ^ next);
                if lo == hi || front == next { break; }
            }
            if lo < hi { out.copy_within(hi..n, lo); out.truncate(lo + (n - hi)); repeat(n, out); } else { out.truncate(n); }
        }
    }
}

// ============================================================================
// 4. Corpus Loaders, Synthetic Generators, Reference FL & All 8 Arm Wrappers
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
    specs.into_iter().map(|(f, p)| {
        let l = fs::read(format!("{dir}/{f}")).unwrap();
        let expected = plain_invert_fl(&l, p);
        Block { name: f, n: l.len(), p, route_class: 0, records: records_of(&l), expected: Some(expected) }
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
        blocks.push(Block { name: format!("{prefix}-{idx}"), n, p, route_class: rc, records, expected });
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
    Block { name: "dense-300k".into(), n, p, route_class: 3, records: records_of(&l), expected: Some(expected) }
}

fn make_spiral_200k() -> Block {
    let n = 200_000usize;
    let mut l = vec![b'a'; n]; l[0] = b'c'; l[n - 1] = b'b';
    let expected = plain_invert_fl(&l, 0);
    Block { name: "spiral-200k".into(), n, p: 0, route_class: 2, records: records_of(&l), expected: Some(expected) }
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
    Block { name: name.into(), n, p, route_class: 2, records: records_of(&l), expected: Some(expected) }
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

#[derive(Default)]
struct ArmRule4<const ORPH: bool, const TIGHT: bool, const TIER: bool> {
    open: engine_rule4::Kernel<ORPH, TIGHT, TIER>,
    seg: InterleavedWalk,
}
impl<const ORPH: bool, const TIGHT: bool, const TIER: bool> ArmRule4<ORPH, TIGHT, TIER> {
    #[inline(never)]
    fn run_stats(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (u8, Option<Counts>) {
        if n < 32_768 { assert!(self.open.invert_tiny_fl(records, n, p, out)); return (1, None); }
        if records.len() * 64 < n && try_short_periodic(records, n, p, out) { return (2, None); }
        if records.len() * 17 >= n * 13 { self.seg.invert(records, n, p, out); return (3, None); }
        let (lim, _) = select_limit(records, n);
        let res = match lim {
            16 => self.open.try_run_const::<16>(records, n, p, Cap::C52_CROSSOVER, POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, Cap::C52_CROSSOVER, POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, Cap::C52_CROSSOVER, POST_S1_BI, out),
        };
        if let Some(st) = res {
            let rt = match (lim, st.used_bi) { (16, true) => 0, (16, false) => 5, (24, _) => 6, _ => 7 };
            return (rt, Some(st));
        }
        self.seg.invert(records, n, p, out); (3, None)
    }
}

#[derive(Default)]
struct ArmCompact3 {
    open: engine_c52::Kernel<false, 0>,
    seg: InterleavedWalk,
}
impl ArmCompact3 {
    #[inline(never)]
    fn run_stats(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (u8, Option<Counts>) {
        if n < 32_768 { assert!(self.open.invert_tiny_fl(records, n, p, out)); return (1, None); }
        if records.len() * 25 >= n * 7 { self.seg.invert(records, n, p, out); return (3, None); }
        let cap = if records.len() * 20 < n * 3 { Cap::NONE } else { Cap::dense(n) };
        if let Some(st) = self.open.try_run_const::<16>(records, n, p, cap, BiMode::AlwaysAt(65_536), out) {
            return (0, Some(st));
        }
        self.seg.invert(records, n, p, out); (3, None)
    }
}

#[derive(Default)]
struct ArmC52V3<const S5_MODE: u8> {
    open: engine_c52::Kernel<true, S5_MODE>,
    seg: InterleavedWalk,
    force_fallback: bool,
}
impl<const S5_MODE: u8> ArmC52V3<S5_MODE> {
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
            16 => self.open.try_run_const::<16>(records, n, p, Cap::C52_CROSSOVER, POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, Cap::C52_CROSSOVER, POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, Cap::C52_CROSSOVER, POST_S1_BI, out),
        };
        if let Some(st) = res {
            let rt = match (lim, st.used_bi) { (16, true) => 0, (16, false) => 5, (24, _) => 6, _ => 7 };
            return (rt, Some(st));
        }
        self.seg.invert(records, n, p, out); (3, None)
    }
}

// ============================================================================
// 5. Classical Atlas Extension `C52-D3` (Extending Chain D `CA-26→27→31→37→41→44`
//    to `7` Pure-`CA` Nodes / `6` Links!):
//    Chebyshev (1874) / Markov (1884) / Stieltjes (1894) Canonical 3-Moment
//    Discrete Support, Hankel Positivity, Principal Representations & Exact Cubic
//    Bounds + Cycle 51 Referee Boundary Note #3 Proof!
// ============================================================================

#[derive(Default, Clone, Copy)]
struct Moment3 {
    m0: i128,
    m1: i128,
    m2: i128,
    m3: i128,
    m_raw: i128,
    rt_raw: i128,
    m: i128,
    rt: i128,
}

fn profile_moments3(records: &[u16], p: u32, lim: u32, cap: u32) -> Moment3 {
    let mut pr = Moment3 { m0: records.len() as i128, ..Default::default() };
    for &rec in records {
        let w = i128::from(rec & 255);
        let w2 = w * w;
        pr.m2 += w2;
        pr.m3 += w2 * w;
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
        if w <= lim { pr.m_raw += wi; pr.rt_raw += 1; pr.m += wi; pr.rt += 1; }
        else if p >= pos && p < pos + w { pr.m += 1; pr.rt += 1; }
        pos += w;
    }
    pr.m1 = i128::from(pos);
    pr
}

struct D3Bounds {
    rt_lb_c2: f64,
    rt_lb_c3: f64,
    m_lb_c3: f64,
    lin_lb_c2: f64,
    lin_lb_c3: f64,
    m_ub_c2: f64,
    m_ub_c3: f64,
    rt_ub_c2: f64,
    rt_ub_c3: f64,
}

fn verify_c52_d3_exact_i128(pr: &Moment3, lim: i128, cap: i128) -> D3Bounds {
    let (m0, m1, m2, m3) = (pr.m0, pr.m1, pr.m2, pr.m3);

    // 1. Stieltjes (1894) Hankel Positivity & 3-Point Principal Representation {1, xi*, C}
    let det_h1 = m0 * m2 - m1 * m1;
    let det_h1_lo = (m1 - m0) * (m3 - m2) - (m2 - m1) * (m2 - m1);
    let det_h1_hi = (cap * m0 - m1) * (cap * m2 - m3) - (cap * m1 - m2) * (cap * m1 - m2);
    assert!(det_h1 >= 0 && det_h1_lo >= 0 && det_h1_hi >= 0, "Stieltjes Hankel det < 0");
    let a0 = -m2 + (cap + 1) * m1 - cap * m0;
    let a1 = -m3 + (cap + 1) * m2 - cap * m1;
    assert!(a0 >= 0);
    if a0 > 0 {
        // Exact 3-point canonical principal representation {1, xi* = a1/a0, C}
        assert!(a1 >= a0 && a1 <= cap * a0);
        // Verify non-negative principal weights p1, pC in exact i128:
        // p1 * (C-1)*(a1-a0)^2 = sum (C-w)*(a0*w - a1)^2 >= 0
        // pC * (C-1)*(C*a0-a1)^2 = sum (w-1)*(a0*w - a1)^2 >= 0
        if a0 <= 1_000_000 {
            let p1_scaled = a0 * a0 * (cap * m2 - m3) - 2 * a0 * a1 * (cap * m1 - m2) + a1 * a1 * (cap * m0 - m1);
            let pc_scaled = a0 * a0 * (m3 - m2) - 2 * a0 * a1 * (m2 - m1) + a1 * a1 * (m1 - m0);
            assert!(p1_scaled >= 0 && pc_scaled >= 0, "Principal weight < 0");
        }
    }
    // 2-Point Gauss-Christoffel-Stieltjes quadrature nodes eta1, eta2 in [1, C] when det_h1 > 0
    if det_h1 > 0 {
        let s = (m0 * m3 - m1 * m2) as f64 / (det_h1 as f64);
        let p_prod = (m1 * m3 - m2 * m2) as f64 / (det_h1 as f64);
        let disc = (s * s - 4.0 * p_prod).max(0.0).sqrt();
        let (eta1, eta2) = (0.5 * (s - disc), 0.5 * (s + disc));
        assert!(eta1 >= 1.0 - 1e-7 && eta2 <= cap as f64 + 1e-7 && eta1 <= eta2 + 1e-7);
    }

    // 2. CA-44 2-moment baseline (upgraded with Boundary Note #3 exact domain 2*L*(L-1) <= (C-1)*(C-2)!)
    let delta2 = m2 - (lim + 2) * m1 + (lim + 1) * m0;
    let rt_mk = if 2 * lim + 1 <= cap { (((lim + 1) * m0 - m1).max(0) as f64) / (lim as f64) }
                else { (((cap + 1) * m0 - 2 * m1).max(0) as f64) / ((cap - 1) as f64) };
    let mut rt_qstar = 0.0f64;
    if lim <= cap - 2 && 2 * lim * (lim - 1) <= (cap - 1) * (cap - 2) {
        let denom = lim * (cap - 1) * (cap - lim - 1);
        let rhs = (cap - 1) * (cap - lim - 1) * ((lim + 1) * m0 - m1) + (cap - 2 * lim - 1) * delta2;
        assert!(denom * pr.rt_raw >= rhs, "Boundary Note #3 q* violated");
        rt_qstar = (rhs.max(0) as f64) / (denom as f64);
    }
    let (mut rt_cant, mut lin_cant) = (0.0f64, 0.0f64);
    if lim <= cap - 2 && (lim + 1) * m0 > m1 {
        let mu = m1 as f64 / m0 as f64;
        let sig2 = (m2 as f64 / m0 as f64 - mu * mu).max(0.0);
        let gap = (lim + 1) as f64 - mu;
        let t_star = (sig2 + gap * gap) / gap;
        let t_cap = (2.0f64.sqrt() + 1.0) * ((cap - lim - 1) as f64);
        let t = t_star.min(t_cap);
        rt_cant = (m0 as f64) * (2.0 * gap / t - (sig2 + gap * gap) / (t * t));
        let (a, b) = (8.0f64, 2.0f64);
        let rhs_r = ((a * ((cap + lim) as f64) + 2.0 * b) / (a * ((lim + 1) as f64) + b)).sqrt();
        let tl = t_star.min((2.0 * ((lim + 1) as f64 + b / a)).min(((cap - lim - 1) as f64) / (rhs_r - 1.0)));
        lin_cant = (m0 as f64) * ((a * mu + b) - (a * ((lim + 1) as f64) + b) * (sig2 + (tl - gap) * (tl - gap)) / (tl * tl));
    }
    let rt_lb_c2 = rt_mk.max(rt_qstar).max(rt_cant);
    let lin_lb_c2 = (10.0 * rt_lb_c2).max(lin_cant);
    let m_u1 = ((cap * m1 - m2) as f64) / ((cap - lim) as f64);
    let m_u2 = (((cap * cap + lim) * m1 - cap * m2 - lim * cap * m0) as f64) / (((cap - 1) * (cap - lim)) as f64);
    let m_ub_c2 = (m1.min(lim * m0) as f64).min(m_u1).min(m_u2);
    let rt_rc = (m0 as f64) - ((m2 - (lim + 1) * m1 + lim * m0) as f64) / (((cap - 1) * (cap - lim)) as f64);
    let rt_ub_c2 = (m0 as f64).min(rt_rc);

    // 3. C52-D3 Exact i128 3-Moment Cubic Upper Bounds: U3_m & U3_rt touching {1, L, k, k+1}
    let (mut m_ub_c3, mut rt_ub_c3) = (m_ub_c2, rt_ub_c2);
    if lim >= 2 && lim <= cap - 2 {
        for k in (lim + 1)..cap {
            let (d1, dl) = (k * (k - 1), (k - lim) * (k + 1 - lim));
            let denom = (lim - 1) * d1 * dl;
            let (am, bm) = (lim * d1 - dl, lim * (dl - d1));
            let rhs_m = am * m3 + (bm - (2 * k + 1) * am) * m2 + (k * (k + 1) * am - (2 * k + 1) * bm) * m1 + (k * (k + 1) * bm) * m0;
            assert!(denom * pr.m_raw <= rhs_m, "U3_m violated");
            m_ub_c3 = m_ub_c3.min((rhs_m as f64) / (denom as f64));

            let (ar, br) = (d1 - dl, lim * dl - d1);
            let rhs_r = ar * m3 + (br - (2 * k + 1) * ar) * m2 + (k * (k + 1) * ar - (2 * k + 1) * br) * m1 + (k * (k + 1) * br) * m0;
            assert!(denom * pr.rt_raw <= rhs_r, "U3_rt violated");
            rt_ub_c3 = rt_ub_c3.min((rhs_r as f64) / (denom as f64));
        }
    } else if lim == 1 && cap >= 3 {
        for k in 2..cap {
            let denom = (k - 1) * k * (cap - 1);
            let rhs = -m3 + (cap + 2 * k + 1) * m2 - (cap * (2 * k + 1) + k * (k + 1)) * m1 + (cap * k * (k + 1)) * m0;
            assert!(denom * pr.m_raw <= rhs, "U3(L=1) violated");
            let v = (rhs as f64) / (denom as f64);
            m_ub_c3 = m_ub_c3.min(v); rt_ub_c3 = rt_ub_c3.min(v);
        }
    }

    // 4. C52-D3 Exact i128 3-Moment Cubic Lower Bounds on r_thin_raw, m_raw, & 8*m + 2*rt
    let mut rt_lb_c3 = rt_lb_c2;
    if lim >= 2 && lim <= cap - 2 {
        for k in 1..lim {
            let (el, ec) = ((lim + 1 - k) * (lim - k), (cap - k) * (cap - k - 1));
            if 2 * el <= ec {
                let denom = (cap - lim - 1) * el * ec;
                let (a, b) = (2 * el - ec, cap * ec - 2 * (lim + 1) * el);
                let rhs = (-a) * m3 + ((2 * k + 1) * a - b) * m2 + ((2 * k + 1) * b - k * (k + 1) * a) * m1 + (denom - k * (k + 1) * b) * m0;
                assert!(denom * pr.rt_raw >= rhs, "L3_rt_sub violated");
                rt_lb_c3 = rt_lb_c3.max((rhs.max(0) as f64) / (denom as f64));
            }
        }
    }
    if lim <= cap - 3 {
        for j in (lim + 2)..cap {
            let (lhs_c, rhs_c) = ((cap - lim - 1) * (cap - j) * (cap - j - 1), lim * j * (j - 1));
            if lhs_c >= rhs_c {
                let rhs = -m3 + (lim + 2 * j + 2) * m2 - ((lim + 1) * (2 * j + 1) + j * (j + 1)) * m1 + ((lim + 1) * j * (j + 1)) * m0;
                assert!(rhs_c * pr.rt_raw >= rhs, "L3_rt_sup_S violated");
                rt_lb_c3 = rt_lb_c3.max((rhs.max(0) as f64) / (rhs_c as f64));
            }
        }
    }
    if lim <= cap - 3 && 2 * lim * (lim - 1) <= (cap - 1) * (cap - 2) {
        let mut best_x = lim + 2;
        let mut best_num = (cap - 1) * (cap - lim - 1) - (cap - 2 * lim - 1) * (best_x - 1);
        let mut best_den = (best_x - 1) * (cap - best_x);
        for x in (lim + 3)..cap {
            let nx = (cap - 1) * (cap - lim - 1) - (cap - 2 * lim - 1) * (x - 1);
            let dx = (x - 1) * (cap - x);
            if nx * best_den < best_num * dx { best_x = x; best_num = nx; best_den = dx; }
        }
        if best_num > 0 {
            let denom = lim * (cap - 1) * (cap - lim - 1) * best_den;
            let q_rhs = (cap - 1) * (cap - lim - 1) * ((lim + 1) * m0 - m1) + (cap - 2 * lim - 1) * delta2;
            let d3 = -m3 + (cap + lim + 2) * m2 - (cap * (lim + 2) + lim + 1) * m1 + cap * (lim + 1) * m0;
            let rhs = best_den * q_rhs + best_num * d3;
            assert!(denom * pr.rt_raw >= rhs, "L3_rt_sup_T violated at x*={best_x}");
            rt_lb_c3 = rt_lb_c3.max((rhs.max(0) as f64) / (denom as f64));
        }
    }
    let mut m_lb_c3 = rt_lb_c3;
    let mut lin_lb_c3 = lin_lb_c2.max(10.0 * rt_lb_c3);
    if lim >= 2 && lim <= cap - 2 {
        for k in 1..lim {
            let (el, ec) = ((lim + 1 - k) * (lim - k), (cap - k) * (cap - k - 1));
            if (cap + lim) * el <= (lim + 1) * ec {
                let denom = (cap - lim - 1) * el * ec;
                let (a, b) = ((cap + lim) * el - (lim + 1) * ec, (lim + 1) * cap * ec - (cap + lim) * (lim + 1) * el);
                let rhs = (-a) * m3 + ((2 * k + 1) * a - b) * m2 + (denom + (2 * k + 1) * b - k * (k + 1) * a) * m1 - (k * (k + 1) * b) * m0;
                assert!(denom * pr.m_raw >= rhs, "L3_m violated");
                m_lb_c3 = m_lb_c3.max((rhs.max(0) as f64) / (denom as f64));
            }
            let (fl1, fcm) = (8 * (lim + 1) + 2, (8 * cap + 2) + (8 * lim + 2));
            if fcm * el <= fl1 * ec {
                let denom = (cap - lim - 1) * el * ec;
                let (a, b) = (fcm * el - fl1 * ec, fl1 * cap * ec - fcm * (lim + 1) * el);
                let rhs = (-a) * m3 + ((2 * k + 1) * a - b) * m2 + (8 * denom + (2 * k + 1) * b - k * (k + 1) * a) * m1 + (2 * denom - k * (k + 1) * b) * m0;
                assert!(denom * (8 * pr.m_raw + 2 * pr.rt_raw) >= rhs, "L3_lin violated");
                lin_lb_c3 = lin_lb_c3.max((rhs.max(0) as f64) / (denom as f64));
            }
        }
    }
    lin_lb_c3 = lin_lb_c3.max(8.0 * m_lb_c3 + 2.0 * rt_lb_c3);
    D3Bounds { rt_lb_c2, rt_lb_c3, m_lb_c3, lin_lb_c2, lin_lb_c3, m_ub_c2, m_ub_c3, rt_ub_c2, rt_ub_c3 }
}

fn run_c52_d3_three_moment_audit(six: &[Block], wiki431: &[Block], non_wiki: &[Block], code_docs: &[Block], dense: &Block) {
    println!("\n=== ATLAS EXTENSION C52-D3 (CHEBYSHEV 1874 / MARKOV 1884 / STIELTJES 1894) ===");
    println!("=== EXTENDING CHAIN D (`CA-26 -> CA-27 -> CA-31 -> CA-37 -> CA-41 -> CA-44 -> C52-D3`, 7 PURE-CA NODES) ===");

    // 1. Cycle 51 Referee Boundary Note #3 Exact IFF proof & w=2 witness across ALL C=3..=25, L=1..=C-2
    let (mut bn3_valid, mut bn3_invalid, mut bn3_w2_fail) = (0usize, 0usize, 0usize);
    for c in 3..=25i128 {
        for l in 1..=(c - 2) {
            let denom = l * (c - 1) * (c - l - 1);
            let mut all_ok = true;
            for x in 1..=c {
                let num = (c - 1) * (c - l - 1) * (l + 1 - x) + (c - 2 * l - 1) * (x - 1) * (x - l - 1);
                if (x <= l && num > denom) || (x > l && num > 0) { all_ok = false; }
            }
            let formula = 2 * l * (l - 1) <= (c - 1) * (c - 2);
            assert_eq!(all_ok, formula);
            if formula { bn3_valid += 1; } else {
                bn3_invalid += 1;
                let num_w2 = (c - 1) * (c - l - 1) * (l - 1) + (c - 2 * l - 1) * (1 - l);
                assert!(num_w2 > denom, "invalid pair must fail at w=2");
                bn3_w2_fail += 1;
            }
        }
    }
    println!("  Boundary Note #3 EXACT IFF (`2*L*(L-1) <= (C-1)*(C-2)`): {bn3_valid} valid / {bn3_invalid} invalid across C=3..=25 (0 mismatches, {bn3_w2_fail}/{bn3_invalid} w=2 witnesses!)");

    // 2. Exact 4-point canonical tightness witnesses for U3_m, U3_rt, L3_rt_sub, L3_rt_sup_S
    {
        // U3_m & U3_rt exact tight on {1, L, k, k+1} for C=127, L=16, k=40
        let mut l_col = Vec::new();
        l_col.extend(std::iter::repeat_n(b'a', 1));
        l_col.extend(std::iter::repeat_n(b'b', 16));
        l_col.extend(std::iter::repeat_n(b'c', 40));
        l_col.extend(std::iter::repeat_n(b'd', 41));
        let pr = profile_moments3(&records_of_cap(&l_col, 127), 0, 16, 127);
        let d = verify_c52_d3_exact_i128(&pr, 16, 127);
        assert!((d.m_ub_c3 - pr.m_raw as f64).abs() < 1e-9, "U3_m must be EXACT TIGHT on {{1,16,40,41}}");
        assert!((d.rt_ub_c3 - pr.rt_raw as f64).abs() < 1e-9, "U3_rt must be EXACT TIGHT on {{1,16,40,41}}");
        assert!(d.m_ub_c3 < d.m_ub_c2 - 1e-6, "U3_m must strictly beat CA-44 U2 on {{1,16,40,41}}");

        // L3_rt_sub exact tight on {k, k+1, L+1, C+k} for C=127, L=16, k=3 (130 splits into (127, 3) -> Q(127)+Q(3) = -1+1 = 0!)
        l_col.clear();
        l_col.extend(std::iter::repeat_n(b'a', 3));
        l_col.extend(std::iter::repeat_n(b'b', 4));
        l_col.extend(std::iter::repeat_n(b'c', 17));
        l_col.extend(std::iter::repeat_n(b'd', 130));
        let pr2 = profile_moments3(&records_of_cap(&l_col, 127), 0, 16, 127);
        let d2 = verify_c52_d3_exact_i128(&pr2, 16, 127);
        assert!((d2.rt_lb_c3 - pr2.rt_raw as f64).abs() < 1e-9, "L3_rt_sub must be EXACT TIGHT on {{3,4,17,130}}");

        // L3_rt_sup_S exact tight on {1, L+1, j, j+1} for C=127, L=16, j=50
        l_col.clear();
        l_col.extend(std::iter::repeat_n(b'a', 1));
        l_col.extend(std::iter::repeat_n(b'b', 17));
        l_col.extend(std::iter::repeat_n(b'c', 50));
        l_col.extend(std::iter::repeat_n(b'd', 51));
        let pr3 = profile_moments3(&records_of_cap(&l_col, 127), 0, 16, 127);
        let d3 = verify_c52_d3_exact_i128(&pr3, 16, 127);
        assert!((d3.rt_lb_c3 - pr3.rt_raw as f64).abs() < 1e-9, "L3_rt_sup_S must be EXACT TIGHT on {{1,17,50,51}}");
        assert!(d3.rt_lb_c3 > d3.rt_lb_c2 + 1e-6, "L3_rt_sup_S must strictly beat CA-44 on {{1,17,50,51}}");
    }

    // 3. 105,000 extremal & random multisets verified in exact i128
    let caps = [5u32, 7, 8, 15, 16, 31, 63, 127];
    let mut rng = 0xC52D_3C4E_B75E_1894u64;
    let mut next_u64 = || -> u64 { rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); rng };
    let mut recs_buf = Vec::with_capacity(512);
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
        let pr = profile_moments3(&recs_buf, p, lim, c);
        verify_c52_d3_exact_i128(&pr, i128::from(lim), i128::from(c));
    }
    println!("  Exact i128 Stress PASS: 105,000 extremal+random multisets + 3 exact 4-point tightness witnesses + Stieltjes Hankel & Principal Reps (0 violations!)");

    // 4. ALL 447 Real Blocks + dense-300k at L=16, CAP=127
    let w233: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();
    let w198: Vec<&Block> = wiki431.iter().filter(|b| b.route_class >= 2).collect();
    let w431_r: Vec<&Block> = wiki431.iter().collect();
    let six_r: Vec<&Block> = six.iter().collect();
    let nw_r: Vec<&Block> = non_wiki.iter().collect();
    let cd_r: Vec<&Block> = code_docs.iter().collect();
    let dense_r = vec![dense];

    println!("  Corpus         | N   | rt LB (C2 -> C3)       | m LB (C3) | 8m+2rt (C2 -> C3)     | >9n(act/C2/C3) | m UB (C2 -> C3)          | rt UB (C2 -> C3) | m<=0.3n(C2->C3)");
    for (name, blks) in [
        ("6 Admitted", &six_r), ("6 JSONL/Log", &nw_r), ("4 Code/Docs", &cd_r),
        ("233 FullWiki", &w233), ("198 TinyWiki", &w198), ("431 AllWiki", &w431_r), ("dense-300k", &dense_r),
    ] {
        let (mut rt2, mut rt3, mut ml3, mut l2, mut l3, mut mu2, mut mu3, mut ru2, mut ru3) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let (mut act9, mut c29, mut c39, mut adm2, mut adm3, mut cnt) = (0usize, 0, 0, 0, 0, 0);
        for &b in blks {
            let pr = profile_moments3(&b.records, b.p, 16, 127);
            let d = verify_c52_d3_exact_i128(&pr, 16, 127);
            let act_lin = (8 * pr.m + 2 * pr.rt) as f64;
            let th = (9 * pr.m1) as f64;
            if act_lin > th { act9 += 1; }
            if d.lin_lb_c2 > th { c29 += 1; }
            if d.lin_lb_c3 > th { c39 += 1; }
            if d.m_ub_c2 + 1.0 <= 0.30 * (pr.m1 as f64) { adm2 += 1; }
            if d.m_ub_c3 + 1.0 <= 0.30 * (pr.m1 as f64) { adm3 += 1; }
            if pr.rt > 0 && pr.m > 0 {
                rt2 += d.rt_lb_c2 / pr.rt as f64; rt3 += d.rt_lb_c3 / pr.rt as f64; ml3 += d.m_lb_c3 / pr.m as f64;
                l2 += d.lin_lb_c2 / act_lin; l3 += d.lin_lb_c3 / act_lin;
                mu2 += (d.m_ub_c2 + 1.0) / pr.m as f64; mu3 += (d.m_ub_c3 + 1.0) / pr.m as f64;
                ru2 += (d.rt_ub_c2 + 1.0) / pr.rt as f64; ru3 += (d.rt_ub_c3 + 1.0) / pr.rt as f64;
                cnt += 1;
            }
        }
        let c = cnt.max(1) as f64;
        println!(
            "  {:14} | {:3} | {:6.4} -> {:6.4} ({:4.2}x) | {:6.4}    | {:6.4} -> {:6.4} ({:4.2}x) | {:3}/{:3}/{:3}    | {:5.3}x -> {:5.3}x ({:4.2}x) | {:5.3}x -> {:5.3}x | {:3} -> {:3}",
            name, blks.len(), rt2 / c, rt3 / c, rt3 / rt2.max(1e-9), ml3 / c,
            l2 / c, l3 / c, l3 / l2.max(1e-9), act9, c29, c39,
            mu2 / c, mu3 / c, mu2 / mu3.max(1e-9), ru2 / c, ru3 / c, adm2, adm3
        );
    }

    // 5. Honest O(nrec) S_2+S_3 scan timing -> killed for runtime selection
    let mut chk = 0u64;
    for (name, blks, reps) in [("6 Admitted", &six_r, 300usize), ("233 FullWiki", &w233, 30), ("6 JSONL/Log", &nw_r, 300), ("4 Code/Docs", &cd_r, 150)] {
        let t0 = Instant::now();
        for _ in 0..reps {
            for &b in blks {
                let (mut s2, mut s3) = (0u64, 0u64);
                for &rec in &b.records { let w = u64::from(rec & 255); let w2 = w * w; s2 += w2; s3 += w2 * w; }
                chk = chk.wrapping_add(s2 ^ s3);
            }
        }
        println!("  S_2+S_3 O(nrec) scan cost [{:12}]: {:6.2} us/blk (vs ~0.001 us O(1) CA-37 -> KILLED for runtime; chk={})", name, t0.elapsed().as_secs_f64() * 1e6 / (reps * blks.len()) as f64, chk & 0xFF);
    }
}

// ============================================================================
// 6. Exhaustive Exactness, `PackedEdgeRule32` Census & WS Attribution
// ============================================================================

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[(v.len() * num) / den]
}

fn run_exactness_census_and_ws(six: &[Block], wiki431: &[Block], non_wiki: &[Block], code_docs: &[Block], dense: &Block) {
    let mut kh = cand_r5_head::Kernel::default();
    let mut c50 = ArmRule4::<false, false, false>::default();
    let mut c51v2 = ArmRule4::<true, true, true>::default();
    let mut c51c3 = ArmCompact3::default();
    let mut c52_soa = ArmC52V3::<1>::default();
    let mut c52_aos = ArmC52V3::<2>::default();
    let mut c52 = ArmC52V3::<0>::default();
    let mut out_ref = Vec::with_capacity(900_064);
    let mut out_k = Vec::with_capacity(900_064);

    // 6a. 10,190 small valid BWTs
    let mut valid_cnt = 0usize;
    let mut text = Vec::with_capacity(512);
    for n in 1..=12usize {
        for mask in 0..(1usize << n) {
            text.clear();
            for bit in 0..n { text.push(b'a' + ((mask >> bit) & 1) as u8); }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            out_k.clear(); out_k.resize(n + 64, 0xAA);
            c52.open.try_run_const::<16>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
            assert_eq!(out_k, text, "C52 Open(bi) binary mismatch n={n}");
            out_k.clear(); out_k.resize(n + 64, 0x55);
            c52.run_stats(&recs, n, p, &mut out_k);
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
        c52.open.try_run_const::<16>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, text);
        c52.open.try_run_const::<32>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(u32::MAX), &mut out_k).unwrap();
        assert_eq!(out_k, text);
        c52.run_stats(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        valid_cnt += 1;
    }

    // 6b. 16M guard + PackedEdgeRule32-Sentinel forced AND natural wide >4095 tests!
    {
        let big = vec![0u16; 16_000_001];
        assert!(!c52.open.invert_tiny_fl(&big, 100, 0, &mut out_k));
        assert!(c52.open.try_run_const::<16>(&big, 100, 0, Cap::NONE, POST_S1_BI, &mut out_k).is_none());

        // Natural wide > 4095 synthetic block (m > 524_288 -> bits=20 -> sentinel=4095, AND >4,500 distinct thick words!)
        // Construct n ~ 760k with 4,800 distinct thick runs (len 20) preceded by distinct thin patterns + 600k thin rows!
        let mut l_wide = Vec::with_capacity(760_000);
        let mut sr = 0x5EED_C520_4095_BEEFu64;
        for g in 0..5_000usize {
            let sym = ((g % 240) + 1) as u8;
            l_wide.resize(l_wide.len() + 20, sym);
            // Insert 110 random thin bytes between thick strips so m ~ 550k (bits=20, sentinel=4095)
            // and >4,500 thick strips land on distinct thin bytes yielding >4,095 codes!
            for _ in 0..110 {
                sr = sr.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                l_wide.push(((sr >> 40) % 240 + 1) as u8);
            }
        }
        let nw_sz = l_wide.len();
        let rw = records_of(&l_wide);
        let want_w = plain_invert_fl(&l_wide, 42);
        let st_w = c52.open.try_run_const::<16>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, want_w, "natural wide >4095 bi-sentinel mismatch!");
        assert!(st_w.m >= (1 << 19) && st_w.bits == 20, "expected bits=20 (m={})", st_w.m);
        // Also test forced sentinel=260 across BOTH bi & uni AND OutTail & BandVec!
        c52.open.force_sentinel_cap = Some(260);
        let st_fb = c52.open.try_run_const::<16>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert!(st_fb.wide && st_fb.used_bi);
        assert_eq!(out_k, want_w, "forced sentinel=260 bi+tail mismatch!");
        c52.open.force_band_vec = true;
        let st_fv = c52.open.try_run_const::<16>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert!(st_fv.wide && st_fv.used_bi && !st_fv.used_tail);
        assert_eq!(out_k, want_w, "forced sentinel=260 bi+vec mismatch!");
        let st_fu = c52.open.try_run_const::<16>(&rw, nw_sz, 42, Cap::NONE, BiMode::AlwaysAt(u32::MAX), &mut out_k).unwrap();
        assert!(st_fu.wide && !st_fu.used_bi);
        assert_eq!(out_k, want_w, "forced sentinel=260 uni mismatch!");
        c52.open.force_band_vec = false;
        c52.open.force_sentinel_cap = None;
        println!(
            "\nPACKED-EDGE SENTINEL STRESS PASS: m={} bits={} num_codes={} natural_wide={} + forced sentinel=260 (bi+tail, bi+vec, uni) 100% BYTE-EXACT!",
            st_w.m, st_w.bits, st_w.num_codes, st_w.wide
        );
    }

    // 6c. Stress + 6 Admitted + 6 JSONL/Log + 4 Code/Docs + 80 OutTail Poison/Resize/Fallback
    let spiral = make_spiral_200k();
    let per2 = make_periodic("periodic-150k-d2", b"ab", 75_000);
    let per3 = make_periodic("periodic-105k-d3", b"abc", 35_000);
    let stress = [dense, &spiral, &per2, &per3];
    let mut ot_poison = 0usize;
    for b in six.iter().chain(stress.iter().copied()).chain(non_wiki.iter()).chain(code_docs.iter()) {
        let want = b.expected.as_ref().unwrap();
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_k).unwrap(); assert_eq!(&out_k, want);
        c50.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(&out_k, want);
        c51v2.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(&out_k, want);
        c51c3.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(&out_k, want);
        c52_soa.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(&out_k, want);
        c52_aos.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(&out_k, want);

        let mut p_aa = vec![0xAAu8; b.n + 4096];
        c52.run_stats(&b.records, b.n, b.p, &mut p_aa); assert_eq!(&p_aa, want);
        let mut fresh = Vec::new();
        c52.run_stats(&b.records, b.n, b.p, &mut fresh); assert_eq!(&fresh, want);
        c52.open.force_band_vec = true;
        let st_fb = c52.open.try_run_const::<16>(&b.records, b.n, b.p, Cap::NONE, BiMode::AlwaysAt(65_536), &mut out_k).unwrap();
        assert!(!st_fb.used_tail); assert_eq!(&out_k, want);
        c52.open.force_band_vec = false;
        c52.force_fallback = true;
        assert_eq!(c52.run_stats(&b.records, b.n, b.p, &mut out_k).0, 0); assert_eq!(&out_k, want);
        c52.force_fallback = false;
        ot_poison += 4;
    }

    // 6d. ALL 431 wikidata.bz2 blocks byte-exact across ALL arms
    for (idx, b) in wiki431.iter().enumerate() {
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        c50.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(out_k, out_ref, "wiki C50 idx={idx}");
        c51v2.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(out_k, out_ref, "wiki C51v2 idx={idx}");
        c51c3.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(out_k, out_ref, "wiki C51c3 idx={idx}");
        c52_soa.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(out_k, out_ref, "wiki C52-SoA idx={idx}");
        c52_aos.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(out_k, out_ref, "wiki C52-AoS idx={idx}");
        c52.run_stats(&b.records, b.n, b.p, &mut out_k); assert_eq!(out_k, out_ref, "wiki C52 idx={idx}");
    }

    // 6e. 345+ Malformed / Multi-Cycle Forward-FL Columns + Peeled S5 Cycles + ZeroH=false + 4-Tier TinyFL
    let mut mal_cnt = 0usize;
    let mut check_mal = |l: &[u8], p: u32, tag: &str| {
        let n = l.len();
        let recs = records_of(l);
        kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        let mut pbuf = vec![0xAAu8; n + 64];
        c52.open.try_run_const::<16>(&recs, n, p, Cap::NONE, BiMode::AlwaysAt(0), &mut pbuf).unwrap();
        assert_eq!(pbuf, out_ref, "malformed C52 bi mismatch {tag}");
        pbuf.clear(); pbuf.resize(n + 64, 0x55);
        c52.run_stats(&recs, n, p, &mut pbuf);
        assert_eq!(pbuf, out_ref, "malformed C52 entry mismatch {tag}");
        mal_cnt += 1;
    };
    check_mal(b"bbaabb", 0, "bbaabb");
    check_mal(b"abbaab", 3, "abbaab");
    for &per in &[253usize, 254, 255, 256, 257, 258, 512] {
        let n = per * 32; let mut l = vec![0u8; n]; l[0] = 2; l[n - 1] = 1;
        check_mal(&l, 0, &format!("sat255-{per}"));
    }
    {
        let mut lt = Vec::new();
        for i in 0..400usize { lt.resize(lt.len() + 20, ((i % 200) + 1) as u8); }
        lt.push(0); check_mal(&lt, 0, "tight-arena");
    }
    for &d in &[1usize, 2, 3, 4, 5, 7, 16, 17] {
        let sz = 80_000usize; let mut lm = vec![0u8; sz];
        for i in 0..d { lm[i] = ((i + 1) % d) as u8; }
        for (i, slot) in lm.iter_mut().enumerate().skip(d) { *slot = (d + (i % 120)) as u8; }
        for slot in &mut lm[sz - 12_000..sz] { *slot = 250; }
        check_mal(&lm, 0, &format!("peeled-d{d}"));
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

    // 6f. 447-Block `PackedEdgeRule32` Census & Deterministic WS Attribution (`PeakCap`, `MaxUsed`, `S5`-live WS)
    let full_wiki: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();
    let tiny_wiki: Vec<&Block> = wiki431.iter().filter(|b| b.route_class >= 2).collect();
    let all_wiki: Vec<&Block> = wiki431.iter().collect();
    let six_r: Vec<&Block> = six.iter().collect();
    let nw_r: Vec<&Block> = non_wiki.iter().collect();
    let cd_r: Vec<&Block> = code_docs.iter().collect();

    println!("\n447-BLOCK `PackedEdgeRule32` CENSUS (LIM=16, Exact `m`/`bits`/`num_codes`/`wide`):");
    println!("  Corpus         | N   | max_m   | bits  | max_codes | avg_codes | wide | >4095 | S5_live_avg(Packed vs SoA vs AoS)");
    let mut kc_pk = engine_c52::Kernel::<true, 0>::default();
    let mut kc_sa = engine_c52::Kernel::<true, 1>::default();
    let mut kc_ao = engine_c52::Kernel::<true, 2>::default();
    for (cname, blks) in [
        ("6 Admitted", &six_r), ("6 JSONL/Log", &nw_r), ("4 Code/Docs", &cd_r),
        ("233 FullWiki", &full_wiki), ("198 TinyWiki", &tiny_wiki), ("431 AllWiki", &all_wiki),
    ] {
        let (mut mm, mut min_b, mut max_b, mut mc, mut sc, mut wc, mut gt4) = (0usize, 32u32, 0u32, 0usize, 0usize, 0usize, 0usize);
        let (mut s5_p, mut s5_s, mut s5_a) = (0usize, 0usize, 0usize);
        for &b in blks {
            let sp = kc_pk.try_run_const::<16>(&b.records, b.n, b.p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
            let ss = kc_sa.try_run_const::<16>(&b.records, b.n, b.p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
            let sa = kc_ao.try_run_const::<16>(&b.records, b.n, b.p, Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
            mm = mm.max(sp.m); min_b = min_b.min(sp.bits); max_b = max_b.max(sp.bits);
            mc = mc.max(sp.num_codes); sc += sp.num_codes;
            if sp.wide { wc += 1; }
            if sp.num_codes > 4095 { gt4 += 1; }
            s5_p += sp.s5_live_bytes; s5_s += ss.s5_live_bytes; s5_a += sa.s5_live_bytes;
        }
        let d = blks.len() as f64 * 1024.0;
        println!(
            "  {:14} | {:3} | {:7} | {:2}..{:2} | {:9} | {:9.1} | {:4} | {:5} | {:6.1}K vs {:6.1}K vs {:6.1}K",
            cname, blks.len(), mm, min_b, max_b, mc, sc as f64 / blks.len() as f64, wc, gt4,
            s5_p as f64 / d, s5_s as f64 / d, s5_a as f64 / d
        );
    }

    println!("\nDETERMINISTIC SCRATCH WORKING SET (`PeakCap`, `MaxUsed`, `S5`-live avg, `sp_avg`, `edges_cap`, `est>m`):");
    for (cname, blks) in [
        ("6 Admitted", &six_r), ("233 FullWiki", &full_wiki), ("431 ALL Wiki", &all_wiki),
        ("6 JSONL/Log", &nw_r), ("4 Code/Docs", &cd_r),
    ] {
        let mut kh_w = cand_r5_head::Kernel::default();
        let mut c50_w = ArmRule4::<false, false, false>::default();
        let mut c51v2_w = ArmRule4::<true, true, true>::default();
        let mut c51c3_w = ArmCompact3::default();
        let mut c52s_w = ArmC52V3::<1>::default();
        let mut c52a_w = ArmC52V3::<2>::default();
        let mut c52_w = ArmC52V3::<0>::default();
        let mut out_w = Vec::with_capacity(900_064);

        let mut stat = [(0usize, 0usize, 0usize, 0usize, 0usize); 6]; // (maxu, s5_sum, sp_sum, n, gt)
        for &b in blks {
            kh_w.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_w);
            let res = [
                c50_w.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                c51v2_w.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                c51c3_w.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                c52s_w.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                c52a_w.run_stats(&b.records, b.n, b.p, &mut out_w).1,
                c52_w.run_stats(&b.records, b.n, b.p, &mut out_w).1,
            ];
            for (idx, r) in res.into_iter().enumerate() {
                if let Some(st) = r {
                    stat[idx].0 = stat[idx].0.max(st.used_bytes);
                    stat[idx].1 += st.s5_live_bytes;
                    stat[idx].2 += st.spelling_used;
                    stat[idx].3 += 1;
                    if st.arena_est_gt_m { stat[idx].4 += 1; }
                }
            }
        }
        let (h_cap, _, _) = kh_w.scratch_report();
        let reps = [
            ("C50-DropIn-v1", c50_w.open.scratch_report(), c50_w.seg.scratch_bytes()),
            ("C51-DropIn-v2", c51v2_w.open.scratch_report(), c51v2_w.seg.scratch_bytes()),
            ("C51-Compact3-v1", c51c3_w.open.scratch_report(), c51c3_w.seg.scratch_bytes()),
            ("C52-v3-UnpSoA", c52s_w.open.scratch_report(), c52s_w.seg.scratch_bytes()),
            ("C52-v3-UnpAoS", c52a_w.open.scratch_report(), c52a_w.seg.scratch_bytes()),
            ("C52-SuperDropIn-v3", c52_w.open.scratch_report(), c52_w.seg.scratch_bytes()),
        ];
        println!("  [{cname}] (R5-Head PeakCap = {:.2}K)", h_cap as f64 / 1024.0);
        for (idx, (aname, (ocap, ec, sdesc), segb)) in reps.iter().enumerate() {
            let tot = ocap + segb;
            let n = stat[idx].3.max(1) as f64;
            println!(
                "    {:18} | PeakCap={:7.2}K (seg={:5.1}K) | MaxUsed={:7.2}K | S5_live={:6.1}K | sp={:5.2}K | ed={:7.2}K | est>m={:2}/{} | {} ({:+.2}K vs Head)",
                aname, tot as f64 / 1024.0, *segb as f64 / 1024.0,
                stat[idx].0 as f64 / 1024.0, stat[idx].1 as f64 / (n * 1024.0),
                stat[idx].2 as f64 / (n * 1024.0), *ec as f64 / 1024.0,
                stat[idx].4, stat[idx].3, sdesc, (tot as isize - h_cap as isize) as f64 / 1024.0
            );
        }
    }

    // 6g. C_recognise telemetry
    println!("\nC_RECOGNISE TELEMETRY (`Spec4` select_limit + O(1) CA-37):");
    let mut chk = 0u64;
    for (label, list, reps) in [("6 Admitted", &six_r, 8_000usize), ("233 FullWiki", &full_wiki, 800), ("6 JSONL/Log", &nw_r, 8_000), ("4 Code/Docs", &cd_r, 8_000)] {
        let (mut l16, mut l24, mut l32, mut s_sum) = (0usize, 0usize, 0usize, 0usize);
        for &b in list { let (l, s) = select_limit(&b.records, b.n); s_sum += s as usize; match l { 16 => l16 += 1, 24 => l24 += 1, _ => l32 += 1 } }
        let t0 = Instant::now();
        for _ in 0..reps { for &b in list { chk = chk.wrapping_add(u64::from(select_limit(&b.records, b.n).0)); } }
        println!("  C_rec {:12}: {:5.1} ns/blk | avg_samples={:5.1} | L16={} L24={} L32={} (chk={})",
            label, t0.elapsed().as_nanos() as f64 / (reps * list.len()) as f64, s_sum as f64 / list.len() as f64, l16, l24, l32, chk & 0xFF);
    }
}

// ============================================================================
// 7. Headroom Oracle v7 Benchmarks (1T, 2W, 4W) Across All 4 Corpora + Unions
// ============================================================================

const NUM_ARMS: usize = 8;
const ARM_NAMES: [&str; NUM_ARMS] = [
    "R5-Head-A1",
    "R5-Head-A2(AA)",
    "C50-DropIn-v1",
    "C51-DropIn-v2",
    "C51-Compact3-v1",
    "C52-v3-UnpSoA",
    "C52-v3-UnpAoS",
    "C52-SuperDropIn-v3",
];

#[derive(Default)]
struct ArmSuite {
    kh1: cand_r5_head::Kernel,
    kh2: cand_r5_head::Kernel,
    c50: ArmRule4<false, false, false>,
    c51v2: ArmRule4<true, true, true>,
    c51c3: ArmCompact3,
    c52_soa: ArmC52V3<1>,
    c52_aos: ArmC52V3<2>,
    c52: ArmC52V3<0>,
}

impl ArmSuite {
    #[inline(always)]
    fn run_arm(&mut self, arm: usize, b: &Block, out: &mut Vec<u8>) -> u8 {
        match arm {
            0 => { self.kh1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out); 0 }
            1 => { self.kh2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out); 0 }
            2 => self.c50.run_stats(&b.records, b.n, b.p, out).0,
            3 => self.c51v2.run_stats(&b.records, b.n, b.p, out).0,
            4 => self.c51c3.run_stats(&b.records, b.n, b.p, out).0,
            5 => self.c52_soa.run_stats(&b.records, b.n, b.p, out).0,
            6 => self.c52_aos.run_stats(&b.records, b.n, b.p, out).0,
            _ => self.c52.run_stats(&b.records, b.n, b.p, out).0,
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
    println!("\n=== HEADROOM ORACLE v7 (1T INTERLEAVED, {rds} rds/blk): {label} ===");
    let mut suite = ArmSuite::default();
    let mut out = Vec::with_capacity(900_064);
    for b in blocks.iter().take(6) { for arm in 0..NUM_ARMS { suite.run_arm(arm, b, &mut out); } }

    let (mut tot, mut full_tot, mut oracle_us, mut full_oracle_us) = ([0.0f64; NUM_ARMS], [0.0f64; NUM_ARMS], 0.0f64, 0.0f64);
    let mut by_rc = [[0.0f64; 4]; NUM_ARMS];
    let mut c52_paths = [0usize; 8];

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
        c52_paths[p7 as usize] += 1;
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
                "  {:12} | H1={:6.1} | C51v2={:6.1} ({:.4}x) | C51c3={:6.1} ({:.4}x) | SoA={:.4}x | AoS={:.4}x | C52(p{})={:6.1} ({:.4}x) | Orcl={:6.1}",
                b.name, meds[0], meds[3], meds[3] / meds[0], meds[4], meds[4] / meds[0],
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
        "  {:18} | TOTAL={:8.1} us ({:.4}x) | C52 capture of oracle headroom = {:.1}%",
        "Hindsight-Oracle", oracle_us, oracle_us / base,
        if base > oracle_us { (base - tot[7]) / (base - oracle_us) * 100.0 } else { 0.0 }
    );
    if show_rc {
        let fb = full_tot[0];
        println!(
            "  233-Full-Wiki 1T   | Head={:.2}ms | C50={:.4}x | C51v2={:.4}x | C51c3={:.4}x | SoA={:.4}x | AoS={:.4}x | C52={:.4}x | Oracle={:.4}x",
            fb / 1e3, full_tot[2] / fb, full_tot[3] / fb, full_tot[4] / fb, full_tot[5] / fb, full_tot[6] / fb, full_tot[7] / fb, full_oracle_us / fb
        );
    }
    println!("  C52 routes [L16-bi, tiny, per, seg8, dense-bi, L16-nobi, L24, L32]: {:?}", c52_paths);
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
    let verify_only = std::env::args().any(|a| a == "--verify-only");
    let six = load_six_admitted();
    let wiki431 = load_bin_corpus(&resolve_corpus_path("wikidata_431_blocks.bin"), "wiki", false);
    let non_wiki = load_bin_corpus(&resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin"), "nonwiki", true);
    let code_docs = load_bin_corpus(&resolve_corpus_path("non_wikidata/code_docs_blocks.bin"), "codedoc", true);
    let dense = make_dense_300k();

    println!(
        "Cycle 52 Headroom Oracle v7 loaded: {} admitted, {} wikidata.bz2, {} JSONL/Log, {} Code/Docs blocks (verify_only={})",
        six.len(), wiki431.len(), non_wiki.len(), code_docs.len(), verify_only
    );

    // 1. Classical Atlas Extension `C52-D3` (Chebyshev 1874 / Markov 1884 / Stieltjes 1894)
    run_c52_d3_three_moment_audit(&six, &wiki431, &non_wiki, &code_docs, &dense);

    // 2. Exhaustive Exactness + `PackedEdgeRule32` 447-Block Census + Deterministic WS + C_recognise
    run_exactness_census_and_ws(&six, &wiki431, &non_wiki, &code_docs, &dense);

    if verify_only {
        println!("\nVERIFY-ONLY COMPLETE (100% PASS)!");
        return;
    }

    // 3. Interleaved 1T Headroom Oracle v7 across all 4 corpora
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
        let lt4 = if sum4_441[arm] < sum4_41_base(sum4_441[0]) { "YES" } else { "NO" };
        println!(
            "    {:18} | 1T={:7.2}ms ({:.4}x, <Open? {:3}) | 2W={:7.2}ms ({:.4}x, <Open? {:3}) | 4W={:7.2}ms ({:.4}x, <Open? {:3})",
            ARM_NAMES[arm],
            sum1_441[arm] / 1e3, sum1_441[arm] / sum1_441[0], lt1,
            sum2_441[arm] / 1e3, sum2_441[arm] / sum2_441[0], lt2,
            sum4_441[arm] / 1e3, sum4_441[arm] / sum4_441[0], lt4,
        );
    }
    println!(
        "    {:18} | 1T={:7.2}ms ({:.4}x) | 6-Admitted 1T/2W/4W C52-SuperDropIn-v3 = {:.4}x / {:.4}x / {:.4}x",
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

#[inline(always)]
fn sum4_41_base(x: f64) -> f64 { x }
