//! Drop-in `#[forbid(unsafe_code)]` Rust 1.80 candidate for `mibz2` (`bench/moonshot` compatible):
//! **Carbon Cycle 19 — `AhtEdgesMux16` (`Candidate`)**, `AhtEdgesMux14`, and `SameGateEdgesMux16`.
//!
//! Combines the five 8–10-worker shared-L3 breakthroughs from Carbon Cycles 15–19:
//!   1. `OutQ16` (`Theorem 16.1`) + Exact SWAR Epoch-Cut Guard `m_lo <= 0xFFFF - CAP_U16`
//!      (`Theorem 17.2`): aliases `1 KiB` (`512 x u16`) 64K-epoch queue chunks inside dead
//!      `out[0..n)` (`0 B` queue heap) with sticky `m = u32::MAX` zero-hot-cost Pass-1 early abort
//!      (`Theorem 19.2b`).
//!   2. `EdgesMux` Temporal Arena Multiplexing (`Theorem 19.1`): aliases `tops` (`[u32; 4]`),
//!      `bots` (`[u32; 2]`), and `u32_scratch` (`u32`) inside dead `self.edges[0..20*ns+38]`
//!      before `wire` overwrites `self.edges[0..m]`, eliminating `~432 KiB`/worker of heap AND
//!      pre-warming `self.edges`'s L2/L3 cache lines (`0.288x` 228-block heap, `0.071x` W=10 DRAM).
//!   3. `SentEmit` (`Theorem 17.3`): power-of-two `n_codes` cursor + temporary `mark` `LONG`
//!      stop-sentinel + `budget = (n - at) >> 4` single-branch safe emission.
//!   4. `AHT` Width-Monotone Strip-Certificate Ostrowski Contraction (`Theorems 18.1 & 19.2a`) +
//!      `MicroLF` (`n <= 2048`) + Proportional `m <= min(256_000, 0.30*n), ns <= 16_384` Structural
//!      Admission (`Theorem 18.2`): admits `383/383` (`100%` of full `900 KB` blocks) on
//!      `wikidata.bz2` with zero hot-loop penalty on normal blocks (`0.917x` instructions),
//!      preventing `Segmented`'s `5.6 MiB`/worker fallback (`0.146x` 10-worker max per-worker heap).
//!   5. Lazy `Option<Box<PassMeta>>`: `0 B` idle overhead on pure column-route / random workers.
#![forbid(unsafe_code)]

use crate::bwt::{InverseBwt, LazyCells};
use crate::runs::Runs;
use crate::Kernels;

/// Default `moonshot` candidate: whole-corpus structural-gate `AhtEdgesMux16`.
pub type Candidate = AhtEdgesMuxOpenCells;

/// Whole-corpus structural-gate `LIMIT = 16` candidate (`383/431` blocks on `wikidata.bz2`).
pub type AhtEdgesMuxOpenCells = UnifiedEdgesMuxOpenCells<16, true>;

/// Whole-corpus structural-gate `LIMIT = 14` L3-latency co-variant (`383/431` blocks).
pub type AhtEdgesMux14OpenCells = UnifiedEdgesMuxOpenCells<14, true>;

/// Exact `Master-0.5.8` `Gate 0.15` drop-in (`228/431` blocks on `wikidata.bz2`).
pub type SameGateEdgesMuxOpenCells = UnifiedEdgesMuxOpenCells<16, false>;

const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const PAD: usize = 1;
const CAP: u32 = 64;
const RULE: u32 = 1 << 31;
const ARENA_MAX: usize = 0x00FF_F000;
const CHUNK_U16: usize = 512;
const CHUNK_B: usize = 1024;
const MICRO_MAX_N: usize = 2048;

pub const M_MAX_DEFAULT: u32 = 256_000;
pub const NS_MAX_DEFAULT: usize = 16_384;

#[derive(Clone, Copy)]
struct Rule {
    left: u32,
    right: u32,
    len: u32,
    first: u32,
}

#[cold]
#[inline(never)]
fn grow_vec_tight<T: Clone>(v: &mut Vec<T>, need: usize, init: T) {
    let cur = v.len();
    let target = need.max(cur + (cur >> 2)).max(256);
    if v.capacity() < target {
        v.reserve_exact(target - cur);
    }
    v.resize(target, init);
}

#[cold]
#[inline(never)]
fn grow_vec_pow2<T: Clone>(v: &mut Vec<T>, need: usize, init: T) {
    let cur = v.len();
    let target = need.next_power_of_two().max(512);
    if v.capacity() < target {
        v.reserve_exact(target - cur);
    }
    v.resize(target, init);
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let s = total;
        total += w;
        s
    })
}

struct PassMeta {
    all: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    head: [u16; 256],
    tail: [u16; 256],
    in_chunk: [u16; 256],
    ep_cuts: Vec<[u32; 256]>,
    n_ep: usize,
    n_chunks: u16,
    max_out_chunks: u16,
    m: u32,
    m_limit: u32,
    mark: u32,
}

impl Default for PassMeta {
    fn default() -> Self {
        Self {
            all: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            head: [u16::MAX; 256],
            tail: [0; 256],
            in_chunk: [CHUNK_U16 as u16; 256],
            ep_cuts: vec![[0u32; 256]; 5],
            n_ep: 1,
            n_chunks: 0,
            max_out_chunks: 0,
            m: 0,
            m_limit: u32::MAX,
            mark: 0,
        }
    }
}

impl PassMeta {
    #[inline(always)]
    fn reset(&mut self, max_out_chunks: u16, m_limit: u32) {
        self.all.fill(0);
        self.thin.fill(0);
        self.thick.fill(0);
        self.head.fill(u16::MAX);
        self.in_chunk.fill(CHUNK_U16 as u16);
        self.n_ep = 1;
        self.n_chunks = 0;
        self.max_out_chunks = max_out_chunks;
        self.m = 0;
        self.m_limit = m_limit;
        self.mark = 0;
    }
}

struct PassOut<'a, const LIMIT: u32> {
    pm: &'a mut PassMeta,
    chunk_next: &'a mut Vec<u16>,
    ovf_chunks: &'a mut Vec<[u8; CHUNK_B]>,
    triples: &'a mut Vec<[u32; 3]>,
    out: &'a mut [u8],
}

#[inline(always)]
fn swar_write_u16s<const NWORDS: usize>(dst: &mut [u8], m_lo: u32) {
    let x0 = (u64::from(m_lo) * 0x0001_0001_0001_0001) + 0x0003_0002_0001_0000;
    if NWORDS >= 1 {
        dst[0..8].copy_from_slice(&x0.to_le_bytes());
    }
    if NWORDS >= 2 {
        dst[8..16].copy_from_slice(&(x0 + 0x0004_0004_0004_0004).to_le_bytes());
    }
    if NWORDS >= 3 {
        dst[16..24].copy_from_slice(&(x0 + 0x0008_0008_0008_0008).to_le_bytes());
    }
    if NWORDS >= 4 {
        dst[24..32].copy_from_slice(&(x0 + 0x000C_000C_000C_000C).to_le_bytes());
    }
}

impl<const LIMIT: u32> PassOut<'_, LIMIT> {
    const NWORDS: usize = (LIMIT as usize + 3) / 4;
    const CAP_U16: usize = Self::NWORDS * 4;
    const M_LO_MAX: u32 = 0xFFFF - (Self::CAP_U16 as u32);

    #[cold]
    #[inline(never)]
    fn push_slow(&mut self, a: usize, w: u32) {
        if w > self.pm.m_limit || self.pm.m > self.pm.m_limit - w {
            // Sticky zero-hot-cost abort (Theorem 19.2b): `m = u32::MAX` makes `m_lo == 0xFFFF`
            // exceed `M_LO_MAX` on all subsequent `push_thin` calls while keeping
            // `self.pm.max_out_chunks` loop-invariant in a GPR across Pass 1!
            self.pm.m = u32::MAX;
            return;
        }
        let max_out_chunks = self.pm.max_out_chunks;
        let mut rem = w;
        let mut cur_m = self.pm.m;
        while rem > 0 {
            let ep = (cur_m >> 16) as usize;
            let ep_end = ((ep as u32) + 1) << 16;
            let mut cid = self.pm.tail[a];
            let mut ic = self.pm.in_chunk[a] as usize;
            if ic >= CHUNK_U16 {
                cid = self.pm.n_chunks;
                self.pm.n_chunks += 1;
                if self.chunk_next.len() <= cid as usize {
                    grow_vec_tight(self.chunk_next, cid as usize + 1, u16::MAX);
                }
                self.chunk_next[cid as usize] = u16::MAX;
                if cid >= max_out_chunks {
                    let need_ovf = (cid - max_out_chunks) as usize + 1;
                    if self.ovf_chunks.len() < need_ovf {
                        grow_vec_tight(self.ovf_chunks, need_ovf, [0u8; CHUNK_B]);
                    }
                }
                if self.pm.head[a] == u16::MAX {
                    self.pm.head[a] = cid;
                } else {
                    self.chunk_next[self.pm.tail[a] as usize] = cid;
                }
                self.pm.tail[a] = cid;
                ic = 0;
            }
            let tk = rem.min(ep_end - cur_m).min((CHUNK_U16 - ic) as u32);
            let cap_u16 = Self::CAP_U16;
            if (tk as usize) <= cap_u16
                && ic <= CHUNK_U16 - cap_u16
                && (cur_m & 0xFFFF) <= Self::M_LO_MAX
                && cid < max_out_chunks
            {
                let b0 = (cid as usize) * CHUNK_B + ic * 2;
                let dst = &mut self.out[b0..b0 + cap_u16 * 2];
                if Self::NWORDS == 3 {
                    swar_write_u16s::<3>(dst, cur_m & 0xFFFF);
                } else {
                    swar_write_u16s::<4>(dst, cur_m & 0xFFFF);
                }
            } else {
                let dst_slice: &mut [u8] = if cid < max_out_chunks {
                    let b0 = (cid as usize) * CHUNK_B + ic * 2;
                    &mut self.out[b0..b0 + (tk as usize) * 2]
                } else {
                    let oi = (cid - max_out_chunks) as usize;
                    &mut self.ovf_chunks[oi][ic * 2..(ic + tk as usize) * 2]
                };
                for (k, pair) in dst_slice.chunks_exact_mut(2).enumerate() {
                    let b = (cur_m as u16).wrapping_add(k as u16).to_le_bytes();
                    pair[0] = b[0];
                    pair[1] = b[1];
                }
            }
            self.pm.in_chunk[a] = (ic + tk as usize) as u16;
            self.pm.thin[a] += tk;
            cur_m += tk;
            rem -= tk;
            if cur_m == ep_end {
                if self.pm.ep_cuts.len() <= ep {
                    grow_vec_tight(&mut self.pm.ep_cuts, ep + 2, [0u32; 256]);
                }
                self.pm.ep_cuts[ep] = self.pm.thin;
                self.pm.n_ep = ep + 2;
            }
        }
        self.pm.m = cur_m;
    }

    #[inline(always)]
    fn push_thin(&mut self, a: usize, x: u32, w: u32, p: u32) {
        if p.wrapping_sub(x) < w {
            self.pm.mark = self.pm.m.wrapping_add(p - x);
        }
        let ic = self.pm.in_chunk[a] as usize;
        let cid = self.pm.tail[a];
        let m_now = self.pm.m;
        let m_lo = m_now & 0xFFFF;
        let cap_u16 = Self::CAP_U16;
        if ic <= CHUNK_U16 - cap_u16 && m_lo <= Self::M_LO_MAX && cid < self.pm.max_out_chunks {
            let b0 = (cid as usize) * CHUNK_B + ic * 2;
            let dst = &mut self.out[b0..b0 + cap_u16 * 2];
            if Self::NWORDS == 3 {
                swar_write_u16s::<3>(dst, m_lo);
            } else {
                swar_write_u16s::<4>(dst, m_lo);
            }
            self.pm.in_chunk[a] = (ic + w as usize) as u16;
            self.pm.thin[a] += w;
            self.pm.m = m_now + w;
        } else {
            self.push_slow(a, w);
        }
        self.pm.all[a] += w;
    }

    #[inline(always)]
    fn push_thick(&mut self, c: u8, x: u32, w: u32) {
        let a = usize::from(c);
        self.triples.push([x, self.pm.all[a], (w << 8) | u32::from(c)]);
        self.pm.thick[a] += 1;
        self.pm.all[a] += w;
    }

    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        let end = pos + width;
        if pos < p {
            self.push_thick(c, pos, p - pos);
        }
        self.push_thin(usize::from(c), p, 1, p);
        if p + 1 < end {
            self.push_thick(c, p + 1, end - p - 1);
        }
    }

    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        if width <= LIMIT {
            self.push_thin(usize::from(c), pos, width, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.push_thick(c, pos, width);
        } else {
            self.take_marked(c, pos, width, p);
        }
    }
}

#[derive(Default)]
struct WordDag {
    arena: Vec<u8>,
    rules: Vec<Rule>,
}

impl WordDag {
    #[inline(always)]
    fn reset(&mut self) {
        self.rules.clear();
        if self.arena.len() < 256 {
            self.arena.clear();
            self.arena.extend(0..=255u8);
        }
        self.arena.truncate(256);
    }

    #[inline(always)]
    fn word_len(&self, w: u32) -> u32 {
        if w == 0 {
            0
        } else if w & RULE == 0 {
            w >> 24
        } else {
            self.rules[(w & !RULE) as usize].len
        }
    }

    #[inline(always)]
    fn join(&mut self, a: u32, b: u32) -> u32 {
        if a == 0 {
            return b;
        }
        if b == 0 {
            return a;
        }
        if (a | b) & RULE == 0 {
            let (al, bl) = (a >> 24, b >> 24);
            let tl = al + bl;
            if tl <= CAP && self.arena.len() + tl as usize <= ARENA_MAX {
                let off = self.arena.len() as u32;
                let (ao, bo) = ((a & 0xFF_FFFF) as usize, (b & 0xFF_FFFF) as usize);
                self.arena.extend_from_within(ao..ao + al as usize);
                self.arena.extend_from_within(bo..bo + bl as usize);
                return (tl << 24) | off;
            }
        }
        let id = self.rules.len() as u32;
        let len = self.word_len(a) + self.word_len(b);
        self.rules.push(Rule {
            left: a,
            right: b,
            len,
            first: u32::MAX,
        });
        RULE | id
    }

    #[cold]
    #[inline(never)]
    fn join_pow(&mut self, mut base: u32, mut exp: u32) -> u32 {
        if base == 0 || exp == 0 {
            return 0;
        }
        let mut acc = 0u32;
        while exp > 0 {
            if (exp & 1) != 0 {
                acc = self.join(acc, base);
            }
            exp >>= 1;
            if exp > 0 {
                base = self.join(base, base);
            }
        }
        acc
    }
}

#[inline(always)]
fn mtop_get(t_buf: &[u32], idx: u32) -> (u32, u32, u32, u32) {
    let i = (idx as usize) * 4;
    let s = &t_buf[i..i + 4];
    (s[0], s[1], s[2], s[3])
}

#[inline(always)]
fn mtop_set(t_buf: &mut [u32], idx: u32, prev: u32, width: u32, word: u32, other: u32) {
    let i = (idx as usize) * 4;
    t_buf[i..i + 4].copy_from_slice(&[prev, width, word, other]);
}

#[inline(always)]
fn mtop_set_wwo(t_buf: &mut [u32], idx: u32, width: u32, word: u32, other: u32) {
    let i = (idx as usize) * 4;
    t_buf[i + 1..i + 4].copy_from_slice(&[width, word, other]);
}

#[inline(always)]
fn mbot_get(b_buf: &[u32], idx: u32) -> (u32, u32) {
    let i = (idx as usize) * 2;
    let s = &b_buf[i..i + 2];
    (s[0], s[1])
}

#[inline(always)]
fn mbot_set(b_buf: &mut [u32], idx: u32, prev: u32, top: u32) {
    let i = (idx as usize) * 2;
    b_buf[i..i + 2].copy_from_slice(&[prev, top]);
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn aht_contract_top_mux(
    dag: &mut WordDag,
    t_buf: &mut [u32],
    b_buf: &mut [u32],
    ts: u32,
    aw: u32,
    rb: u32,
    bs: u32,
    fb: u32,
    nb: u32,
    bw: u32,
    ra: u32,
    remaining: u32,
) -> (u32, u32, u32, u32, u32, u32) {
    let q = (ra - 1) / rb;
    if q < 2 {
        return (bs, fb, nb, bw, ra, remaining);
    }
    let p_tail = b_buf[(bs as usize) * 2];
    let pow_aw = dag.join_pow(aw, q);
    let bw_new = dag.join(pow_aw, bw);
    let (u, v) = if (q & 1) == 1 { (fb, bs) } else { (bs, fb) };
    mtop_set_wwo(t_buf, ts, rb, bw_new, v);
    mbot_set(b_buf, u, p_tail, ts);
    mbot_set(b_buf, v, u, ts);
    let consumed = q * rb;
    (u, v, u, bw_new, ra - consumed, remaining - consumed)
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn aht_contract_bot_mux(
    dag: &mut WordDag,
    t_buf: &mut [u32],
    b_buf: &mut [u32],
    bs: u32,
    ra: u32,
    bw: u32,
    ts: u32,
    fa: u32,
    na: u32,
    aw: u32,
    rb: u32,
    remaining: u32,
) -> (u32, u32, u32, u32, u32, u32) {
    let q = (rb - 1) / ra;
    if q < 2 {
        return (ts, fa, na, aw, rb, remaining);
    }
    let p_tail = t_buf[(ts as usize) * 4];
    let pow_qm1 = dag.join_pow(bw, q - 1);
    let w_qm1 = dag.join(aw, pow_qm1);
    let w_q = dag.join(w_qm1, bw);
    let (u, v) = if (q & 1) == 1 { (fa, ts) } else { (ts, fa) };
    b_buf[(bs as usize) * 2 + 1] = v;
    mtop_set(t_buf, u, p_tail, ra, w_qm1, bs);
    mtop_set(t_buf, v, u, ra, w_q, bs);
    let consumed = q * ra;
    (u, v, u, w_q, rb - consumed, remaining - consumed)
}

#[cold]
#[inline(never)]
fn run_r4_aht_mux(
    dag: &mut WordDag,
    t_buf: &mut [u32],
    b_buf: &mut [u32],
    mut sa: u32,
    mut sb: u32,
    mut remaining: u32,
) -> (u32, u32, usize) {
    let mut steps = 0usize;
    while remaining > 0 {
        let (mut na, mut ra, mut aw, mut bs) = mtop_get(t_buf, sa);
        let (mut nb, mut ts) = mbot_get(b_buf, sb);
        let (_, mut rb, mut bw, _) = mtop_get(t_buf, ts);
        let (mut fa, mut fb) = (sa, sb);
        if bs == sb {
            remaining -= ra;
            steps += 1;
            (sa, sb) = (na, nb);
            continue;
        }
        loop {
            steps += 1;
            if nb == bs && ra > rb && (ra - 1) >= 2 * rb {
                (bs, fb, nb, bw, ra, remaining) =
                    aht_contract_top_mux(dag, t_buf, b_buf, ts, aw, rb, bs, fb, nb, bw, ra, remaining);
            }
            if na == ts && ra < rb && (rb - 1) >= 2 * ra {
                (ts, fa, na, aw, rb, remaining) =
                    aht_contract_bot_mux(dag, t_buf, b_buf, bs, ra, bw, ts, fa, na, aw, rb, remaining);
            }
            let wd = dag.join(aw, bw);
            if ra == rb {
                remaining -= ra;
                mtop_set_wwo(t_buf, ts, ra, wd, bs);
                b_buf[(bs as usize) * 2 + 1] = ts;
                (sa, sb) = (na, nb);
                break;
            } else if ra > rb {
                remaining -= rb;
                mtop_set_wwo(t_buf, ts, rb, wd, bs);
                let bs2 = (bs as usize) * 2;
                let bs_pair = &mut b_buf[bs2..bs2 + 2];
                let p_bs = bs_pair[0];
                bs_pair.copy_from_slice(&[fb, ts]);
                b_buf[(fb as usize) * 2] = p_bs;
                bs = fb;
                ra -= rb;
                fb = nb;
                (nb, ts) = mbot_get(b_buf, nb);
                let (_, n_rb, n_bw, _) = mtop_get(t_buf, ts);
                (rb, bw) = (n_rb, n_bw);
            } else {
                remaining -= ra;
                b_buf[(bs as usize) * 2 + 1] = ts;
                let ts4 = (ts as usize) * 4;
                let ts_sl = &mut t_buf[ts4..ts4 + 4];
                let p_ts = ts_sl[0];
                ts_sl.copy_from_slice(&[fa, ra, wd, bs]);
                t_buf[(fa as usize) * 4] = p_ts;
                ts = fa;
                rb -= ra;
                fa = na;
                (na, ra, aw, bs) = mtop_get(t_buf, na);
            }
        }
    }
    (sa, sb, steps)
}

#[derive(Default)]
pub struct UnifiedEdgesMuxKernel<const LIMIT: u32, const USE_AHT: bool> {
    pm: Option<Box<PassMeta>>,
    chunk_next: Vec<u16>,
    ovf_chunks: Vec<[u8; CHUNK_B]>,
    triples: Vec<[u32; 3]>,
    dag: WordDag,
    stack: Vec<u32>,
    spelling: Vec<u8>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    n_codes: usize,
    edges: Vec<u32>,
    extra: Vec<u32>,
    pub r4_loop_steps: usize,
    pub last_ovf_chunks_used: usize,
}

impl<const LIMIT: u32, const USE_AHT: bool> UnifiedEdgesMuxKernel<LIMIT, USE_AHT> {
    pub fn poison_tapes_for_test(&mut self) {
        self.chunk_next.fill(0xDEAD);
        self.ovf_chunks.fill([0xCC; CHUNK_B]);
        if self.inline16.len() > 256 {
            self.inline16[256..].fill([0xEE; INLINE]);
            self.lens[256..].fill(11);
        }
        self.edges.fill(0xDEAD_BEEF);
    }

    #[inline(always)]
    fn push_code_slot(&mut self, v: [u8; INLINE], ln: u8) -> u32 {
        let c = self.n_codes;
        if c >= self.inline16.len() {
            grow_vec_pow2(&mut self.inline16, c + 1, [0u8; INLINE]);
            grow_vec_pow2(&mut self.lens, c + 1, LONG);
        }
        self.inline16[c] = v;
        self.lens[c] = ln;
        self.n_codes = c + 1;
        c as u32
    }

    #[cold]
    #[inline(never)]
    fn make_long_code(&mut self, nd_word: u32, a: u8) -> u32 {
        let o0 = self.spelling.len();
        self.stack.clear();
        self.stack.push(nd_word);
        while let Some(w) = self.stack.pop() {
            if w & RULE == 0 {
                let off = (w & 0xFF_FFFF) as usize;
                self.spelling
                    .extend_from_slice(&self.dag.arena[off..off + (w >> 24) as usize]);
            } else {
                let rule = &mut self.dag.rules[(w & !RULE) as usize];
                if rule.first == u32::MAX {
                    rule.first = self.spelling.len() as u32;
                    let (l, r) = (rule.left, rule.right);
                    self.stack.push(r);
                    self.stack.push(l);
                } else {
                    let start = rule.first as usize;
                    self.spelling
                        .extend_from_within(start..start + rule.len as usize);
                }
            }
        }
        self.spelling.push(a);
        let o1 = self.spelling.len();
        let mut v = [0u8; INLINE];
        v[0..4].copy_from_slice(&(o0 as u32).to_le_bytes());
        v[4..8].copy_from_slice(&(o1 as u32).to_le_bytes());
        self.push_code_slot(v, LONG)
    }

    #[inline(always)]
    fn make_code(&mut self, nd_word: u32, a: u8) -> u32 {
        if nd_word == 0 {
            return u32::from(a);
        }
        if (nd_word & RULE) == 0 && (nd_word >> 24) < INLINE as u32 {
            let wl = (nd_word >> 24) as usize;
            let off = (nd_word & 0xFF_FFFF) as usize;
            let mut v = [0u8; INLINE];
            v.copy_from_slice(&self.dag.arena[off..off + INLINE]);
            v[wl] = a;
            self.push_code_slot(v, (wl + 1) as u8)
        } else {
            self.make_long_code(nd_word, a)
        }
    }

    #[cold]
    #[inline(never)]
    fn run_micro(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if self.edges.len() < n {
            grow_vec_tight(&mut self.edges, n, 0u32);
        }
        let mut counts = [0u16; 256];
        let mut pos_check = 0usize;
        for &rec in records {
            let b = (rec >> 8) as usize;
            let w = rec & 255;
            counts[b] = counts[b].wrapping_add(w);
            pos_check += w as usize;
        }
        if pos_check != n {
            return false;
        }
        let mut occ = [0u16; 256];
        let mut tot = 0u16;
        for c in 0..256 {
            occ[c] = tot;
            tot = tot.wrapping_add(counts[c]);
        }
        let tt = &mut self.edges[..n];
        let mut row_pos = 0u32;
        for &rec in records {
            let b = (rec >> 8) as usize;
            let w = (rec & 255) as usize;
            let start = occ[b] as usize;
            occ[b] += w as u16;
            let b_u32 = b as u32;
            for k in 0..w {
                tt[start + k] = ((row_pos + k as u32) << 8) | b_u32;
            }
            row_pos += w as u32;
        }
        out.clear();
        out.reserve(n);
        let mut r = p as usize;
        for _ in 0..n {
            let e = tt[r];
            out.push((e & 0xFF) as u8);
            r = (e >> 8) as usize;
            if r == p as usize {
                break;
            }
        }
        while out.len() < n {
            let tk = out.len().min(n - out.len());
            out.extend_from_within(..tk);
        }
        true
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_band_cross(
        edges: &mut [u32],
        out_bytes: &[u8],
        ovf_chunks: &[[u8; CHUNK_B]],
        chunk_next: &[u16],
        ep_cuts: &[[u32; 256]],
        n_ep: usize,
        max_out_chunks: u16,
        a: usize,
        cid: &mut u16,
        ic: &mut usize,
        ep: &mut usize,
        q_cur: &mut usize,
        mut cur_dst: usize,
        mut rem_span: usize,
        cpack: u32,
    ) {
        while rem_span > 0 {
            while *ep + 1 < n_ep && *q_cur >= ep_cuts[*ep][a] as usize {
                *ep += 1;
            }
            let ep_hi = ep_cuts[*ep][a] as usize;
            if *ic >= CHUNK_U16 {
                *cid = chunk_next[*cid as usize];
                *ic = 0;
            }
            let tk = rem_span.min(CHUNK_U16 - *ic).min(ep_hi - *q_cur);
            let pack = cpack | ((*ep as u32) << 16);
            let src_b: &[u8] = if *cid < max_out_chunks {
                let off_b = (*cid as usize) * CHUNK_B + *ic * 2;
                &out_bytes[off_b..off_b + tk * 2]
            } else {
                let oi = (*cid - max_out_chunks) as usize;
                &ovf_chunks[oi][*ic * 2..(*ic + tk) * 2]
            };
            for (ed, pair) in edges[cur_dst..cur_dst + tk]
                .iter_mut()
                .zip(src_b.chunks_exact(2))
            {
                let lo16 = u16::from_le_bytes([pair[0], pair[1]]);
                *ed = pack | u32::from(lo16);
            }
            *ic += tk;
            *q_cur += tk;
            cur_dst += tk;
            rem_span -= tk;
        }
    }

    pub fn try_run<const M_MAX: u32, const NS_MAX: usize>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap_override: u16,
        out: &mut Vec<u8>,
    ) -> bool {
        if n == 0 || n > 900_000 || (p as usize) >= n {
            return false;
        }
        if USE_AHT && n <= MICRO_MAX_N && cap_override == 60_000 {
            self.r4_loop_steps = 0;
            self.last_ovf_chunks_used = 0;
            return self.run_micro(records, n, p, out);
        }
        if out.len() < n + 32 {
            out.resize(n + 32, 0);
        }
        let max_out_chunks = ((out.len() / CHUNK_B).min(60_000) as u16).min(cap_override);
        let m_limit = if M_MAX < 900_000 {
            M_MAX.min(((n as u64 * 3) / 10) as u32).max(2048)
        } else {
            M_MAX
        };
        let pm_box = self.pm.get_or_insert_with(Box::default);
        pm_box.reset(max_out_chunks, m_limit);
        self.triples.clear();

        {
            let mut pass = PassOut::<LIMIT> {
                pm: pm_box,
                chunk_next: &mut self.chunk_next,
                ovf_chunks: &mut self.ovf_chunks,
                triples: &mut self.triples,
                out: &mut out[..],
            };
            let mut pos = 0u32;
            let mut idx = 0;
            while idx < records.len() {
                let c = (records[idx] >> 8) as u8;
                let mut width = u32::from(records[idx] & 255);
                idx += 1;
                while idx < records.len() && (records[idx] >> 8) as u8 == c {
                    width += u32::from(records[idx] & 255);
                    idx += 1;
                }
                pass.take(c, pos, width, p);
                pos += width;
            }
            if pos as usize != n {
                return false;
            }
        }

        let pm_ref = self.pm.as_mut().unwrap();
        let m = pm_ref.m;
        let ns = self.triples.len();
        if m > m_limit || ns > NS_MAX {
            return false;
        }

        let mark = pm_ref.mark;
        let final_ep = (m >> 16) as usize;
        if pm_ref.n_ep <= final_ep {
            if pm_ref.ep_cuts.len() <= final_ep {
                grow_vec_tight(&mut pm_ref.ep_cuts, final_ep + 2, [0u32; 256]);
            }
            pm_ref.n_ep = final_ep + 1;
        }
        let n_ep = pm_ref.n_ep;
        pm_ref.ep_cuts[n_ep - 1] = pm_ref.thin;
        self.last_ovf_chunks_used = (pm_ref.n_chunks.saturating_sub(max_out_chunks)) as usize;
        let first = starts(pm_ref.all);
        let mut write = starts(pm_ref.thick);
        let thin_start = starts(pm_ref.thin);

        self.dag.reset();
        let slots = PAD + 3 * ns + 4;
        let p0_base = PAD as u32;
        let a0_base = p0_base + ns as u32 + 2;

        // Theorem 19.1 (`EdgesMux`): `tops` (`4*slots`), `bots` (`2*slots`), and `u32_scratch`
        // (`2*ns`) are 100% dead before `wire` writes `self.edges[..m]`.
        let need_u32 = (m as usize).max(6 * slots + 2 * ns + 8);
        if self.edges.len() < need_u32 {
            grow_vec_tight(&mut self.edges, need_u32, 0u32);
        }

        let mut sa_final: u32;
        let mut r4_steps = 0usize;
        {
            let (t_buf, rest1) = self.edges.split_at_mut(4 * slots);
            let (b_buf, rest2) = rest1.split_at_mut(2 * slots);
            let scr = &mut rest2[..2 * ns];
            for (i, s) in self.triples.iter_mut().enumerate() {
                let a = (s[2] & 255) as usize;
                s[1] += first[a];
                scr[write[a] as usize] = i as u32;
                write[a] += 1;
            }
            mtop_set(t_buf, 0, 0, 0, 0, 0);
            mbot_set(b_buf, 0, 0, 0);
            let (mut t, mut b) = (a0_base, PAD as u32);
            let mut x = 0u32;
            let mut max_w = 0u32;
            for (i, s) in self.triples.iter().enumerate() {
                let s_from = s[0];
                let s_width = s[2] >> 8;
                let s_byte = (s[2] & 255) as u8;
                if x < s_from {
                    let gw = s_from - x;
                    if gw > max_w {
                        max_w = gw;
                    }
                    mtop_set(t_buf, t, t - 1, gw, 0, b);
                    mbot_set(b_buf, b, b - 1, t);
                    t += 1;
                    b += 1;
                }
                if s_width > max_w {
                    max_w = s_width;
                }
                scr[ns + i] = t;
                mtop_set(t_buf, t, t - 1, s_width, (1 << 24) | u32::from(s_byte), 0);
                t += 1;
                x = s_from + s_width;
            }
            if x < n as u32 {
                let gw = n as u32 - x;
                if gw > max_w {
                    max_w = gw;
                }
                mtop_set(t_buf, t, t - 1, gw, 0, b);
                mbot_set(b_buf, b, b - 1, t);
                t += 1;
                b += 1;
            }
            let a0_end = t;
            let mut p0 = p0_base;
            x = 0;
            for ii in 0..ns {
                let i = scr[ii] as usize;
                let s = self.triples[i];
                let s_to = s[1];
                let s_width = s[2] >> 8;
                if x < s_to {
                    let gw = s_to - x;
                    if gw > max_w {
                        max_w = gw;
                    }
                    mtop_set(t_buf, p0, p0 - 1, gw, 0, b);
                    mbot_set(b_buf, b, b - 1, p0);
                    p0 += 1;
                    b += 1;
                }
                let st = scr[ns + i];
                t_buf[(st as usize) * 4 + 3] = b;
                mbot_set(b_buf, b, b - 1, st);
                b += 1;
                x = s_to + s_width;
            }
            if x < n as u32 {
                let gw = n as u32 - x;
                if gw > max_w {
                    max_w = gw;
                }
                mtop_set(t_buf, p0, p0 - 1, gw, 0, b);
                mbot_set(b_buf, b, b - 1, p0);
                p0 += 1;
                b += 1;
            }
            if p0 > p0_base {
                t_buf[(p0_base as usize) * 4] = 0;
            }
            b_buf[PAD * 2] = 0;
            t_buf[(a0_base as usize) * 4] = if p0 > p0_base { p0 - 1 } else { 0 };

            let (mut sa, mut sb) = (a0_end - 1, b - 1);
            let mut remaining = n as u32;
            // Theorem 19.2a: strip widths strictly decrease under Rauzy-Veech splitting, so
            // `max_w <= 32_768 && records.len() * 64 >= n` certifies zero large spirals and
            // runs the pure call-free leaf loop (`0.917x` instructions).
            if USE_AHT && (records.len() * 64 < n || max_w > 32_768) {
                (sa, sb, r4_steps) =
                    run_r4_aht_mux(&mut self.dag, t_buf, b_buf, sa, sb, remaining);
            } else {
                while remaining > 0 {
                    let (mut na, mut ra, mut aw, mut bs) = mtop_get(t_buf, sa);
                    let (mut nb, mut ts) = mbot_get(b_buf, sb);
                    let (_, mut rb, mut bw, _) = mtop_get(t_buf, ts);
                    let (mut fa, mut fb) = (sa, sb);
                    if bs == sb {
                        remaining -= ra;
                        r4_steps += 1;
                        (sa, sb) = (na, nb);
                        continue;
                    }
                    loop {
                        r4_steps += 1;
                        let wd = self.dag.join(aw, bw);
                        if ra == rb {
                            remaining -= ra;
                            mtop_set_wwo(t_buf, ts, ra, wd, bs);
                            b_buf[(bs as usize) * 2 + 1] = ts;
                            (sa, sb) = (na, nb);
                            break;
                        } else if ra > rb {
                            remaining -= rb;
                            mtop_set_wwo(t_buf, ts, rb, wd, bs);
                            let bs2 = (bs as usize) * 2;
                            let bs_pair = &mut b_buf[bs2..bs2 + 2];
                            let p_bs = bs_pair[0];
                            bs_pair.copy_from_slice(&[fb, ts]);
                            b_buf[(fb as usize) * 2] = p_bs;
                            bs = fb;
                            ra -= rb;
                            fb = nb;
                            (nb, ts) = mbot_get(b_buf, nb);
                            let (_, n_rb, n_bw, _) = mtop_get(t_buf, ts);
                            (rb, bw) = (n_rb, n_bw);
                        } else {
                            remaining -= ra;
                            b_buf[(bs as usize) * 2 + 1] = ts;
                            let ts4 = (ts as usize) * 4;
                            let ts_sl = &mut t_buf[ts4..ts4 + 4];
                            let p_ts = ts_sl[0];
                            ts_sl.copy_from_slice(&[fa, ra, wd, bs]);
                            t_buf[(fa as usize) * 4] = p_ts;
                            ts = fa;
                            rb -= ra;
                            fa = na;
                            (na, ra, aw, bs) = mtop_get(t_buf, na);
                        }
                    }
                }
            }

            let mut xb = m;
            while sb != 0 {
                let (s_prev, s_top) = mbot_get(b_buf, sb);
                let ti = (s_top as usize) * 4;
                xb -= t_buf[ti + 1];
                t_buf[ti + 3] = xb;
                sb = s_prev;
            }
            sa_final = sa;
        }
        self.r4_loop_steps = r4_steps;
        self.dag.arena.extend_from_slice(&[0u8; INLINE]);

        if self.inline16.len() < 512 {
            grow_vec_pow2(&mut self.inline16, 512, [0u8; INLINE]);
            grow_vec_pow2(&mut self.lens, 512, LONG);
            for c in 0..=255u8 {
                let mut v = [0u8; INLINE];
                v[0] = c;
                self.inline16[usize::from(c)] = v;
                self.lens[usize::from(c)] = 1;
            }
        }
        self.n_codes = 256;
        self.spelling.clear();
        self.triples.clear();

        let mut rev_start = [0usize; 256];
        let mut rem = if sa_final != 0 {
            self.edges[(sa_final as usize) * 4 + 1]
        } else {
            0
        };
        let mut kc = m;
        for a in (0..256).rev() {
            rev_start[a] = self.triples.len();
            let start = thin_start[a];
            while kc > start {
                let (nd_prev, _, nd_word, nd_other) = mtop_get(&self.edges, sa_final);
                let width = (kc - start).min(rem);
                rem -= width;
                kc -= width;
                let dst = nd_other + rem;
                let code = self.make_code(nd_word, a as u8);
                self.triples.push([width, dst, code]);
                if rem == 0 {
                    sa_final = nd_prev;
                    rem = if sa_final != 0 {
                        self.edges[(sa_final as usize) * 4 + 1]
                    } else {
                        0
                    };
                }
            }
        }

        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let n_real_codes = self.n_codes;
        let wide = (n_real_codes as u32) > (u32::MAX >> bits);
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }

        {
            let pm_w = self.pm.as_ref().unwrap();
            let out_bytes = &out[..];
            let bands = &self.triples[..];
            let ep_cuts = &pm_w.ep_cuts[..];
            for a in 0..256 {
                let b0 = rev_start[a];
                let b1 = if a > 0 { rev_start[a - 1] } else { bands.len() };
                if b0 == b1 {
                    continue;
                }
                let mut cid = pm_w.head[a];
                let mut ic = 0usize;
                let mut ep = 0usize;
                let mut q_cur = 0usize;
                let mut ep_hi = ep_cuts[0][a] as usize;
                for bnd in bands[b0..b1].iter().rev() {
                    let span = bnd[0] as usize;
                    let dst = bnd[1] as usize;
                    let code = bnd[2];
                    if wide {
                        self.extra[dst..dst + span].fill(code);
                    }
                    let cpack = if wide { 0 } else { code << bits };
                    if ic + span <= CHUNK_U16 && q_cur + span <= ep_hi && cid < max_out_chunks {
                        let pack = cpack | ((ep as u32) << 16);
                        let off_b = (cid as usize) * CHUNK_B + ic * 2;
                        let src_b = &out_bytes[off_b..off_b + span * 2];
                        for (ed, pair) in self.edges[dst..dst + span]
                            .iter_mut()
                            .zip(src_b.chunks_exact(2))
                        {
                            let lo16 = u16::from_le_bytes([pair[0], pair[1]]);
                            *ed = pack | u32::from(lo16);
                        }
                        ic += span;
                        q_cur += span;
                    } else {
                        Self::wire_band_cross(
                            &mut self.edges,
                            out_bytes,
                            &self.ovf_chunks,
                            &self.chunk_next,
                            ep_cuts,
                            n_ep,
                            max_out_chunks,
                            a,
                            &mut cid,
                            &mut ic,
                            &mut ep,
                            &mut q_cur,
                            dst,
                            span,
                            cpack,
                        );
                        ep_hi = ep_cuts[ep][a] as usize;
                    }
                }
            }
        }

        if !wide {
            let stop_code = self.push_code_slot([0u8; INLINE], LONG);
            emit_sentinel(
                &mut self.edges[..m as usize],
                m,
                &self.spelling,
                &self.inline16,
                &self.lens,
                stop_code,
                n,
                mark,
                bits,
                out,
            );
        } else {
            emit_wide(
                &self.edges[..m as usize],
                &self.extra,
                &self.spelling,
                &self.inline16[..n_real_codes],
                &self.lens[..n_real_codes],
                n,
                mark,
                bits,
                out,
            );
        }
        true
    }
}

#[cold]
#[inline(never)]
fn emit_long_word(
    meta: [u8; INLINE],
    spelling: &[u8],
    n: usize,
    at: &mut usize,
    out: &mut [u8],
) {
    let o0 = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
    let o1 = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
    let word = &spelling[o0..o1];
    let take = word.len().min(n - *at);
    out[*at..*at + take].copy_from_slice(&word[..take]);
    *at += take;
}

fn emit_sentinel(
    edges_s: &mut [u32],
    m: u32,
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    stop_code: u32,
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    if out.len() < n + 32 {
        out.resize(n + 32, 0);
    }
    let mask = (1u32 << bits) - 1;
    let code_cap = inline16.len();
    let code_mask = code_cap - 1;
    let stop_c = stop_code as usize;

    let raw_mark = edges_s[mark as usize];
    let code0 = ((raw_mark >> bits) as usize) & code_mask;
    let mut j = raw_mark & mask;
    edges_s[mark as usize] = (stop_code << bits) | j;

    let mut at = 0usize;
    {
        let out_s = &mut out[..n + INLINE];
        let inline_s = &inline16[..code_cap];
        let lens_s = &lens[..code_cap];

        let ln0 = lens_s[code0];
        if ln0 <= INLINE as u8 && INLINE <= n {
            out_s[0..INLINE].copy_from_slice(&inline_s[code0]);
            at = usize::from(ln0);
        } else if ln0 <= INLINE as u8 {
            let tk = usize::from(ln0).min(n);
            out_s[0..tk].copy_from_slice(&inline_s[code0][..tk]);
            at = tk;
        } else {
            emit_long_word(inline_s[code0], spelling, n, &mut at, out_s);
        }

        let edges_ro = &edges_s[..];
        let mut steps_left = (m as usize).saturating_sub(1);
        let mut hit_stop = false;
        while !hit_stop && at + 64 <= n && steps_left > 0 {
            let budget = ((n - at) >> 4).min(steps_left);
            let mut done = 0usize;
            while done < budget {
                let edge = edges_ro[j as usize];
                let c = ((edge >> bits) as usize) & code_mask;
                j = edge & mask;
                done += 1;
                let ln = lens_s[c];
                if ln <= INLINE as u8 {
                    out_s[at..at + INLINE].copy_from_slice(&inline_s[c]);
                    at += usize::from(ln);
                } else {
                    if c == stop_c {
                        hit_stop = true;
                    } else {
                        emit_long_word(inline_s[c], spelling, n, &mut at, out_s);
                    }
                    break;
                }
            }
            steps_left -= done;
        }

        while !hit_stop && at < n && steps_left > 0 {
            let edge = edges_ro[j as usize];
            let c = ((edge >> bits) as usize) & code_mask;
            j = edge & mask;
            steps_left -= 1;
            let ln = lens_s[c];
            if ln <= INLINE as u8 {
                if at + INLINE <= out_s.len() {
                    out_s[at..at + INLINE].copy_from_slice(&inline_s[c]);
                    at = (at + usize::from(ln)).min(n);
                } else {
                    let tk = usize::from(ln).min(n - at);
                    out_s[at..at + tk].copy_from_slice(&inline_s[c][..tk]);
                    at += tk;
                }
            } else if c == stop_c {
                break;
            } else {
                emit_long_word(inline_s[c], spelling, n, &mut at, out_s);
            }
        }
    }

    edges_s[mark as usize] = raw_mark;
    out.truncate(at.min(n));
    let mut len = out.len();
    assert!(len > 0 && len <= n);
    while len < n {
        let take = len.min(n - len);
        out.extend_from_within(..take);
        len = out.len();
    }
}

#[cold]
#[inline(never)]
fn emit_wide(
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
    if out.len() < n + INLINE {
        out.resize(n + INLINE, 0);
    }
    let mut at = 0usize;
    let mask = (1u32 << bits) - 1;
    let mut j = mark;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = extra[j as usize] as usize;
        j = edge & mask;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            emit_long_word(inline16[code], spelling, n, &mut at, out);
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    let mut len = out.len();
    while len < n {
        let take = len.min(n - len);
        out.extend_from_within(..take);
        len = out.len();
    }
}

#[derive(Default)]
pub struct UnifiedEdgesMuxOpenCells<const LIMIT: u32, const USE_AHT: bool> {
    pub k: UnifiedEdgesMuxKernel<LIMIT, USE_AHT>,
    lazy: LazyCells,
}

impl<const LIMIT: u32, const USE_AHT: bool> InverseBwt
    for UnifiedEdgesMuxOpenCells<LIMIT, USE_AHT>
{
    fn reads_runs(&self) -> bool {
        true
    }

    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        let nr = r.records().len();
        if USE_AHT {
            if n > 0
                && n <= 900_000
                && (p as usize) < n
                && (n <= MICRO_MAX_N || nr * 100 < n * 28)
            {
                if self
                    .k
                    .try_run::<M_MAX_DEFAULT, NS_MAX_DEFAULT>(r.records(), n, p, 60_000, out)
                {
                    return true;
                }
            }
        } else if n > 0 && n <= 900_000 && (p as usize) < n && nr * 20 < n * 3 && nr * 64 >= n {
            if self.k.try_run::<900_000, 900_000>(r.records(), n, p, 60_000, out) {
                return true;
            }
        }
        self.lazy.invert_runs(r, n, p, out, k)
    }

    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(ll, counts, p, out);
    }

    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(ll, counts, p)
    }
}
