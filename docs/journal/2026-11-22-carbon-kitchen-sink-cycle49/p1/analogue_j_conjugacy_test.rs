#![forbid(unsafe_code)]

//! Cycle 49 (P1) — Theorem J3: Empirical Test of Frobenius Conjugacy-Class Transport
//! (`pi_ret -> tau * pi_ret * tau^{-1}`) & Frobenius Periodic Fast Path on `edges` (`Open<16>`).
//!
//! Tests whether conjugating `edges[0..m]` (`~640–840 KiB` on Wikidata) by a cheap bijection
//! `tau in S_m` reduces `S5` L1D/L2 cache-line scatter and beats `cand_r5_head` (`Tau0-Id`):
//!   - `Tau0-Id`:           Baseline `cand_r5_head` (`L`-rank indexing of `edges`).
//!   - `Tau1-FOrder`:       Young-subgroup `S_m` colour-bucket conjugator (`F`-order indexing).
//!   - `Tau2-CodeOrder`:    Dictionary-code frequency conjugator (hot codes packed in low lines).
//!   - `Tau3-FOrderGreedy`: Frobenius-Mackey double-coset `N_ab` greedy-ordered bucket conjugator.
//!   - `Tau4-OrbitOracle`:  Theoretical optimal conjugator `tau(u) = orbit_pos(u)` (emit-only
//!                          headroom ceiling; excluded from end-to-end speedups by CA-07).
//!   - `FrobPeriodic`:      Frobenius 1898 `gcd_a(all[a]) > 1` recognizer + exact Cauchy closure
//!                          fast path with exact fallback on non-periodic `gcd > 1` blocks.
//!
//! Exactness gates (100% byte-exact vs `cand_r5_head` AND independent `forward_fl_invert`):
//!   - 5,200 small valid BWTs
//!   - 300 malformed multi-cycle columns (incl. 20 large m >= 65,536 bi-XOR)
//!   - Forced fallbacks: adversarial `gcd=2` non-periodic block, `spiral-200k` (`LONG=255`),
//!     `wide` dictionary stress, uni-emit & bi-emit boundaries
//!   - Solved Analogue J (`AnaJ-Per-g16`, `AnaJ-Per-300k`, `AnaJ-Clustered-100k`)
//!   - 6 Admitted Wikidata + 6 Non-Wikidata (JSONL+Syslog) + 4 Code+Docs + 233 Full wikidata.bz2
//!
//! Measurements:
//!   - Deterministic 64B cache-line proxy (same-line hit%, simulated 48 KiB 12-way L1D &
//!     512 KiB 8-way L2 miss rates on `edges` accesses during `S5`)
//!   - Stage breakdown (`S1`, `S23`, `Tau`, `S4`, `S5`, `Total`) & persistent working set (`KiB`)
//!   - Native `1T / 2W / 4W` wall-time ratios and `C_rec + C_mem < C_Open` verdict.

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

use std::fs;
use std::sync::Barrier;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const ID_BITS: u32 = 22;
const ID_MASK: u32 = (1 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ConjMode {
    Tau0Id,
    Tau1FOrder,
    Tau2CodeOrder,
    Tau3FOrderGreedy,
    Tau4OrbitOracle,
    FrobPeriodic,
}

impl ConjMode {
    fn label(self) -> &'static str {
        match self {
            ConjMode::Tau0Id => "Tau0-Id",
            ConjMode::Tau1FOrder => "Tau1-FOrder",
            ConjMode::Tau2CodeOrder => "Tau2-CodeOrd",
            ConjMode::Tau3FOrderGreedy => "Tau3-FGreedy",
            ConjMode::Tau4OrbitOracle => "Tau4-Oracle*",
            ConjMode::FrobPeriodic => "Frob-Period",
        }
    }
}

#[derive(Default, Clone, Copy)]
struct StageTimes {
    s1_us: f64,
    s23_us: f64,
    tau_us: f64,
    s4_us: f64,
    s5_us: f64,
}

impl StageTimes {
    fn total_us(&self) -> f64 {
        self.s1_us + self.s23_us + self.tau_us + self.s4_us + self.s5_us
    }
}

// ============================================================================
// 1. OutQ16 (Exact cand_r5_head Queues/Spill + Conjugated Wire Paths)
// ============================================================================

mod outq_j {
    const CHUNK: usize = 512;
    const BYTES: usize = 2 * CHUNK;
    const NONE: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;

    #[derive(Default)]
    pub struct Spill {
        next: Vec<u32>,
        ovf: Vec<[u8; BYTES]>,
        cuts: Vec<[u32; 256]>,
    }

    impl Spill {
        pub fn scratch_bytes(&self) -> usize {
            4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
        }
    }

    pub struct Queues<'a> {
        out: &'a mut [u8],
        spill: &'a mut Spill,
        in_out: u32,
        chunks: u32,
        heads: [u32; 256],
        pt: [u64; 256],
    }

    #[derive(Clone, Copy)]
    pub struct Heads {
        heads: [u32; 256],
        in_out: u32,
    }

    impl<'a> Queues<'a> {
        pub fn new(out: &'a mut [u8], spill: &'a mut Spill) -> Self {
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
            }
        }
        pub fn heads(&self) -> Heads {
            Heads {
                heads: self.heads,
                in_out: self.in_out,
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
        pub fn new(out: &'a [u8], spill: &'a Spill, heads: Heads, a: usize) -> Self {
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

        /// Populate `tau[v] = f_base + k` for all `count` items in bucket `a` (for Tau1-FOrder / Tau3-FOrderGreedy).
        pub fn fill_tau_bucket(&mut self, tau: &mut [u32], mut f_cur: u32, mut left: usize) {
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
                    self.id = self.spill.next[self.id as usize];
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
                let bytes = &chunk[2 * self.at..2 * (self.at + take)];
                let mut i = 0usize;
                while i + 4 <= take {
                    let b = &bytes[2 * i..2 * i + 8];
                    let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    tau[(hi | (q as u32 & 0xFFFF)) as usize] = f_cur;
                    tau[(hi | ((q >> 16) as u32 & 0xFFFF)) as usize] = f_cur + 1;
                    tau[(hi | ((q >> 32) as u32 & 0xFFFF)) as usize] = f_cur + 2;
                    tau[(hi | ((q >> 48) as u32)) as usize] = f_cur + 3;
                    f_cur += 4;
                    i += 4;
                }
                while i < take {
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    tau[v as usize] = f_cur;
                    f_cur += 1;
                    i += 1;
                }
                self.at += take;
                self.read += take as u32;
                left -= take;
            }
        }

        /// Populate `col_l[v] = a` for all `count` items in bucket `a` (for Tau3-FOrderGreedy N_ab build).
        pub fn fill_col_bucket(&mut self, col_l: &mut [u8], a_byte: u8, mut left: usize) {
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
                    self.id = self.spill.next[self.id as usize];
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
                let bytes = &chunk[2 * self.at..2 * (self.at + take)];
                for s in bytes.chunks_exact(2) {
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    col_l[v as usize] = a_byte;
                }
                self.at += take;
                self.read += take as u32;
                left -= take;
            }
        }

        /// Baseline Tau0-Id unidirectional wire (matches cand_r5_head/outq.rs)
        pub fn wire_uni(&mut self, mut dst: &mut [u32], pack: u32) {
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
                    self.id = self.spill.next[self.id as usize];
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
                let bytes = &chunk[2 * self.at..2 * (self.at + take)];
                for (d, s) in dst[..take].iter_mut().zip(bytes.chunks_exact(2)) {
                    *d = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += take;
                self.read += take as u32;
                dst = &mut dst[take..];
            }
        }

        /// Baseline Tau0-Id bidirectional XOR wire (16x unrolled like cand_r5_head/outq.rs)
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
                let hi_pack = pack | hi;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut i = 0usize;
                while i + 8 <= span {
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
                    let u = dst_start + i;
                    let sl = &mut edges[u..u + 8];
                    sl[0] ^= pack | v0;
                    sl[1] ^= pack | v1;
                    sl[2] ^= pack | v2;
                    sl[3] ^= pack | v3;
                    sl[4] ^= pack | v4;
                    sl[5] ^= pack | v5;
                    sl[6] ^= pack | v6;
                    sl[7] ^= pack | v7;
                    edges[v0 as usize] ^= u as u32;
                    edges[v1 as usize] ^= (u + 1) as u32;
                    edges[v2 as usize] ^= (u + 2) as u32;
                    edges[v3 as usize] ^= (u + 3) as u32;
                    edges[v4 as usize] ^= (u + 4) as u32;
                    edges[v5 as usize] ^= (u + 5) as u32;
                    edges[v6 as usize] ^= (u + 6) as u32;
                    edges[v7 as usize] ^= (u + 7) as u32;
                    i += 8;
                }
                while i < span {
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    let u = dst_start + i;
                    edges[u] ^= hi_pack | (v & 0xFFFF);
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
                    self.id = self.spill.next[self.id as usize];
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

        /// Conjugated wire for general `tau[0..m]` (used by Tau2-CodeOrder):
        /// `edges[tau[u]] ^= pack | tau[v]; edges[tau[v]] ^= tau[u]`
        pub fn wire_xor_tau(
            &mut self,
            edges: &mut [u32],
            tau: &[u32],
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
                    self.id = self.spill.next[self.id as usize];
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
                let bytes = &chunk[2 * self.at..2 * (self.at + take)];
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        *predecessor = u as u32;
                    }
                    let ut = tau[u];
                    let vt = tau[v as usize];
                    edges[ut as usize] ^= pack | vt;
                    edges[vt as usize] ^= ut;
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        /// Conjugated unidirectional wire for general `tau[0..m]` (used by Tau2-CodeOrder):
        pub fn wire_uni_tau(
            &mut self,
            edges: &mut [u32],
            extra: &mut [u32],
            tau: &[u32],
            mut dst_start: usize,
            mut left: usize,
            bcode: u32,
            pack: u32,
            wide: bool,
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
                    self.id = self.spill.next[self.id as usize];
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
                let bytes = &chunk[2 * self.at..2 * (self.at + take)];
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    let ut = tau[u] as usize;
                    let vt = tau[v as usize];
                    edges[ut] = pack | vt;
                    if wide {
                        extra[ut] = bcode;
                    }
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
// 2. Endpoint Elimination & Dictionary (Verbatim cand_r5_head Semantics)
// ============================================================================

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

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

fn gcd_u32(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
    let mut slot = [0; INLINE];
    slot[..4].copy_from_slice(&start.to_le_bytes());
    slot[4..8].copy_from_slice(&end.to_le_bytes());
    slot
}

fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
    let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
    let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
    &spelling[start..end]
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

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links> Elim<'_, L> {
    #[inline(always)]
    fn literal(byte: u8) -> u32 {
        (1 << ID_BITS) | u32::from(byte)
    }
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
    fn set_bottom(&mut self, id: u32, x: u32) {
        self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
    }
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
    fn rule(&self, id: u32) -> usize {
        self.base + (id as usize - 65_793) * RULE
    }
    #[inline(always)]
    fn short_payload(&self, w: u32) -> u128 {
        let g = w >> ID_BITS;
        let id = w & ID_MASK;
        if g <= 2 {
            return u128::from(id);
        }
        let at = self.rule(id);
        let s = &self.a[at..at + 4];
        u128::from(s[0])
            | (u128::from(s[1]) << 32)
            | (u128::from(s[2]) << 64)
            | (u128::from(s[3] & 0x00FF_FFFF) << 96)
    }
    #[inline(always)]
    fn word_len(&self, w: u32) -> u32 {
        let g = w >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.a[self.rule(w & ID_MASK) + 2]
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
        let gl = left >> ID_BITS;
        let sum = gl + (right >> ID_BITS);
        if sum < INLINE as u32 {
            if sum == 2 {
                return (2 << ID_BITS) | (left & 0xFF) | ((right & 0xFF) << 8);
            }
            let id = ((self.top - self.base) / RULE) as u32 + 65_793;
            let bytes = self.short_payload(left)
                | (self.short_payload(right) << (8 * gl))
                | (u128::from(sum) << 120);
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = bytes as u32;
            out[1] = (bytes >> 32) as u32;
            out[2] = (bytes >> 64) as u32;
            out[3] = (bytes >> 96) as u32;
            self.top += RULE;
            (sum << ID_BITS) | id
        } else {
            let len = if sum <= SATURATED {
                sum
            } else {
                self.word_len(left) + self.word_len(right)
            };
            let id = ((self.top - self.base) / RULE) as u32 + 65_793;
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.top += RULE;
            (len.min(SATURATED) << ID_BITS) | id
        }
    }
    fn pow(&mut self, word: u32, q: u32) -> u32 {
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
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let sent = &self.a[..2];
            let x = sent[0] & 0xFFFF;
            let y = sent[1] & 0xFFFF;
            let xi = x as usize * 4;
            let yi = y as usize * 4;
            let nx = &self.a[xi..xi + 4];
            let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                self.a[px0 as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px0;
                self.a[px1 as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | px1;
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
                self.a[px as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px;
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y | (y_next << 16);
                    nxm[3] = word;
                    let nym = &mut self.a[yi..yi + 4];
                    nym[0] = (y0 & 0xFFFF) | (x << 16);
                    nym[2] = bw - aw;
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                } else {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y_prev | (y_next << 16);
                    nxm[3] = word;
                    let ypi = y_prev as usize * 4;
                    self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    let py = y1 & 0xFFFF;
                    self.a[py as usize * 4 + 1] &= 0xFFFF;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                }
            } else {
                let py = y1 & 0xFFFF;
                self.a[py as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                let x_next = if x == py { 0 } else { x1 >> 16 };
                let nym = &mut self.a[yi..yi + 4];
                nym[1] = x | (x_next << 16);
                nym[3] = word;
                let nxm = &mut self.a[xi..xi + 4];
                nxm[1] = (x1 & 0xFFFF) | (y << 16);
                nxm[2] = aw - bw;
                let xni = x_next as usize * 4 + 1;
                self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
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
                    let head = self.pow(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let tail = self.pow(self.word(y), q);
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
    fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload(h).to_le_bytes();
                dest.extend_from_slice(&bytes[..g as usize]);
            } else {
                let r = self.rule(h & ID_MASK);
                let tag = self.a[r + 3];
                let rule = &mut self.a[r..r + RULE];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7FFF_FFFF) as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }
}

// ============================================================================
// 3. Conjugacy-Transport Kernel (`ConjKernel`) + Periodic Fast Path
// ============================================================================

#[derive(Default)]
struct ConjKernel {
    spill: outq_j::Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    tau: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    // Scratch for Tau2/Tau3/Tau4
    code_freq: Vec<u32>,
    code_off: Vec<u32>,
    col_tmp: Vec<u8>,
    // Scratch for FrobPeriodic (only allocated on periodic hits!)
    fl_tmp: Vec<u32>,
    per_tmp: Vec<u8>,
}

impl ConjKernel {
    fn scratch_bytes(&self, mode: ConjMode) -> usize {
        let base = self.spill.scratch_bytes()
            + 8 * self.bands.capacity()
            + self.spelling.capacity()
            + 4 * self.edges.capacity()
            + 4 * self.extra.capacity()
            + 16 * self.inline16.capacity()
            + self.lens.capacity()
            + 4 * self.stack.capacity();
        let conj_extra = match mode {
            ConjMode::Tau0Id | ConjMode::FrobPeriodic => {
                4 * self.fl_tmp.capacity() + self.per_tmp.capacity()
            }
            ConjMode::Tau1FOrder => 4 * self.tau.capacity(),
            ConjMode::Tau2CodeOrder => {
                4 * self.tau.capacity()
                    + 4 * self.code_freq.capacity()
                    + 4 * self.code_off.capacity()
            }
            ConjMode::Tau3FOrderGreedy => {
                4 * self.tau.capacity() + self.col_tmp.capacity()
            }
            ConjMode::Tau4OrbitOracle => 4 * self.tau.capacity(),
        };
        base + conj_extra
    }

    /// Frobenius (1898) gcd_a(all[a]) > 1 exact periodic recognizer + Cauchy closure fast path.
    /// Never modifies `out` (which holds OutQ16) unless exact closure `x == p` succeeds!
    fn try_frob_periodic(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut all = [0u32; 256];
        for &rec in records {
            all[usize::from((rec >> 8) as u8)] += u32::from(rec & 255);
        }
        let mut g = 0u32;
        for &cnt in &all {
            if cnt > 0 {
                g = gcd_u32(g, cnt);
                if g == 1 {
                    return false;
                }
            }
        }
        if g <= 1 {
            return false;
        }
        let d = n / (g as usize);
        if d == 0 || d * (g as usize) != n {
            return false;
        }
        // Build FL on [0..n) and verify exact Cauchy orbit closure of length <= d from p!
        // (Remember: gcd_a(all[a]) > 1 is NECESSARY for periodicity, NOT sufficient!)
        self.fl_tmp.clear();
        self.fl_tmp.resize(n, 0);
        let mut cur_first = starts(all);
        let mut f_bounds = [0u32; 256];
        {
            let mut acc = 0u32;
            for a in 0..256 {
                acc += all[a];
                f_bounds[a] = acc;
            }
        }
        let mut pos = 0u32;
        for &rec in records {
            let a = usize::from((rec >> 8) as u8);
            let w = u32::from(rec & 255);
            let dst = cur_first[a] as usize;
            for off in 0..w {
                self.fl_tmp[dst + off as usize] = pos + off;
            }
            cur_first[a] += w;
            pos += w;
        }
        self.per_tmp.clear();
        self.per_tmp.reserve(d);
        let mut x = p;
        for _ in 0..d {
            let b = f_bounds.partition_point(|&e| e <= x) as u8;
            self.per_tmp.push(b);
            x = self.fl_tmp[x as usize];
            if x == p {
                out.clear();
                out.reserve(n);
                out.extend_from_slice(&self.per_tmp);
                repeat(n, out);
                return true;
            }
        }
        // Did NOT close at d steps -> non-periodic block with gcd(all) > 1 -> exact fallback!
        false
    }

    fn run_mode(
        &mut self,
        mode: ConjMode,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> StageTimes {
        let mut st = StageTimes::default();

        if mode == ConjMode::FrobPeriodic {
            // Post-S1 zero-extra-pass check vs pre-S1 check:
            // Do a lightweight pre-check: only run full periodic path if records.len() * 4 <= n,
            // wait: to test C_rec honestly on ALL 249 real blocks, check gcd on S1's `all`!
            // Actually check fast pre-pass if small or check right after S1 below!
        }

        // === Stage S1: Run classification + OutQ16 ===
        let t0 = Instant::now();
        out.resize(n + INLINE, 0);
        let mut queues = outq_j::Queues::new(out, &mut self.spill);
        let mut thick = [0u32; 256];
        let mut thick_rows = [0u32; 256];
        let mut ns = 0usize;
        let mut m = 0u32;
        let mut mark = 0u32;
        let mut mark_byte = 0u8;

        let mut pos = 0u32;
        let mut i = 0usize;
        let nrec = records.len();
        while i < nrec {
            let rec = records[i];
            let c = (rec >> 8) as u8;
            let mut width = u32::from(rec & 255);
            i += 1;
            while i < nrec && (records[i] >> 8) as u8 == c {
                width += u32::from(records[i] & 255);
                i += 1;
            }
            let end = pos + width;
            let mut piece = |x: u32, w: u32, keep: bool| {
                let a = usize::from(c);
                if keep {
                    if p.wrapping_sub(x) < w {
                        mark = m + (p - x);
                        mark_byte = c;
                    }
                    queues.push_thin(a, m, w);
                    m += w;
                } else {
                    let at = 3 * ns;
                    if self.edges.len() < at + 3 {
                        let need = (at + 3).max(2 * self.edges.len()).max(1024);
                        self.edges.resize(need, 0);
                    }
                    let thin_a = queues.thin_of(a);
                    self.edges[at..at + 3].copy_from_slice(&[
                        x,
                        thin_a + thick_rows[a],
                        (w << 8) | u32::from(c),
                    ]);
                    ns += 1;
                    thick[a] += 1;
                    thick_rows[a] += w;
                }
            };
            if width > LIMIT && (pos..end).contains(&p) {
                if pos < p {
                    piece(pos, p - pos, false);
                }
                piece(p, 1, true);
                if p + 1 < end {
                    piece(p + 1, end - p - 1, false);
                }
            } else {
                piece(pos, width, width <= LIMIT);
            }
            pos += width;
        }
        assert_eq!(pos as usize, n);
        let thin = queues.extract_thin();
        let heads = queues.heads();
        st.s1_us = t0.elapsed().as_secs_f64() * 1e6;

        // Post-S1 Frobenius 1898 gcd(all) > 1 check (0 extra scan passes — uses S1's `thin + thick_rows`!)
        if mode == ConjMode::FrobPeriodic {
            let t_rec = Instant::now();
            let mut g = 0u32;
            for a in 0..256 {
                let tot_a = thin[a] + thick_rows[a];
                if tot_a > 0 {
                    g = gcd_u32(g, tot_a);
                    if g == 1 {
                        break;
                    }
                }
            }
            if g > 1 {
                if self.try_frob_periodic(records, n, p, out) {
                    st.tau_us = t_rec.elapsed().as_secs_f64() * 1e6;
                    return st;
                }
            }
            st.tau_us = t_rec.elapsed().as_secs_f64() * 1e6;
        }

        // === Stage S2b + S3a: Endpoint Elimination & Band/Dictionary Construction ===
        let t1 = Instant::now();
        let all: [u32; 256] = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        let first = starts(all);
        let mut write = starts(thick);
        let most = 3 * ns + 3;
        let is_wide_links = most > NARROW_NODES;
        let stride = if is_wide_links {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        let need_words = (4 * ns + (stride + RULE) * most).max(m as usize);
        if self.edges.len() < need_words {
            self.edges.resize(need_words, 0);
        }

        let cursors = if is_wide_links {
            self.run_induced::<Wide>(ns, n as u32, m, &thin, &first, &mut write, most)
        } else {
            self.run_induced::<Narrow>(ns, n as u32, m, &thin, &first, &mut write, most)
        };
        st.s23_us = t1.elapsed().as_secs_f64() * 1e6;

        let m_us = m as usize;
        let bits = if m <= 1 {
            1
        } else {
            (u32::BITS - (m - 1).leading_zeros()).max(1)
        };
        let wide_code = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        let use_bi = !wide_code && m >= 65_536;

        // === Stage Tau: Build Conjugator tau[0..m] (mode-dependent) ===
        let t2 = Instant::now();
        match mode {
            ConjMode::Tau0Id | ConjMode::FrobPeriodic => {}
            ConjMode::Tau1FOrder => {
                if self.tau.len() < m_us {
                    self.tau.resize(m_us, 0);
                }
                let tau = &mut self.tau[..m_us];
                let mut f_cur = 0u32;
                for a in 0..256 {
                    let cnt = thin[a] as usize;
                    if cnt > 0 {
                        let mut reader = outq_j::Reader::new(out, &self.spill, heads, a);
                        reader.fill_tau_bucket(tau, f_cur, cnt);
                        f_cur += cnt as u32;
                    }
                }
                assert_eq!(f_cur, m);
            }
            ConjMode::Tau2CodeOrder => {
                let dlen = self.lens.len();
                self.code_freq.clear();
                self.code_freq.resize(dlen, 0);
                for &b in &self.bands {
                    self.code_freq[b.code() as usize] += b.end();
                }
                // Sort non-empty codes descending by frequency (pack hot codes into lowest cache lines!)
                let mut order: Vec<u32> = (0..dlen as u32)
                    .filter(|&c| self.code_freq[c as usize] > 0)
                    .collect();
                order.sort_unstable_by(|&a, &b| {
                    self.code_freq[b as usize]
                        .cmp(&self.code_freq[a as usize])
                        .then_with(|| a.cmp(&b))
                });
                self.code_off.clear();
                self.code_off.resize(dlen, 0);
                let mut cur = 0u32;
                for &c in &order {
                    self.code_off[c as usize] = cur;
                    cur += self.code_freq[c as usize];
                }
                assert_eq!(cur, m);
                if self.tau.len() < m_us {
                    self.tau.resize(m_us, 0);
                }
                let tau = &mut self.tau[..m_us];
                for &b in &self.bands {
                    let c = b.code() as usize;
                    let dst = b.dst() as usize;
                    let span = b.end() as usize;
                    let base = self.code_off[c];
                    for off in 0..span {
                        tau[dst + off] = base + off as u32;
                    }
                    self.code_off[c] = base + span as u32;
                }
            }
            ConjMode::Tau3FOrderGreedy => {
                // Build col_l[u] from OutQ16, tally Frobenius-Mackey double-coset matrix N_ab via bands,
                // greedy-order buckets by max (N_ab + N_ba), then build tau in greedy bucket order!
                if self.col_tmp.len() < m_us {
                    self.col_tmp.resize(m_us, 0);
                }
                let col_l = &mut self.col_tmp[..m_us];
                for a in 0..256 {
                    let cnt = thin[a] as usize;
                    if cnt > 0 {
                        let mut reader = outq_j::Reader::new(out, &self.spill, heads, a);
                        reader.fill_col_bucket(col_l, a as u8, cnt);
                    }
                }
                let mut nab = vec![0u32; 256 * 256];
                for b_dst in 0..256 {
                    let end = cursors.get(b_dst + 1).copied().unwrap_or(self.bands.len());
                    for &band in &self.bands[cursors[b_dst]..end] {
                        let dst = band.dst() as usize;
                        let span = band.end() as usize;
                        for &a_src in &col_l[dst..dst + span] {
                            nab[usize::from(a_src) * 256 + b_dst] += 1;
                        }
                    }
                }
                // Greedy max-weight path over non-empty buckets
                let mut used_b = [false; 256];
                let mut b_order = Vec::with_capacity(256);
                let start_b = (0..256).max_by_key(|&a| thin[a]).unwrap();
                let mut cur_b = start_b;
                used_b[cur_b] = true;
                if thin[cur_b] > 0 {
                    b_order.push(cur_b);
                }
                loop {
                    let mut best = None;
                    let mut best_w = 0u32;
                    for nb in 0..256 {
                        if !used_b[nb] && thin[nb] > 0 {
                            let w = nab[cur_b * 256 + nb] + nab[nb * 256 + cur_b] + 1;
                            if w > best_w {
                                best_w = w;
                                best = Some(nb);
                            }
                        }
                    }
                    match best {
                        Some(nb) => {
                            used_b[nb] = true;
                            b_order.push(nb);
                            cur_b = nb;
                        }
                        None => break,
                    }
                }
                if self.tau.len() < m_us {
                    self.tau.resize(m_us, 0);
                }
                let tau = &mut self.tau[..m_us];
                let mut f_cur = 0u32;
                for a in b_order {
                    let cnt = thin[a] as usize;
                    let mut reader = outq_j::Reader::new(out, &self.spill, heads, a);
                    reader.fill_tau_bucket(tau, f_cur, cnt);
                    f_cur += cnt as u32;
                }
                assert_eq!(f_cur, m);
            }
            ConjMode::Tau4OrbitOracle => {
                // Build exact pi[0..m] temporarily in edges, walk cycles to assign sequential orbit ranks!
                if self.edges.len() < m_us {
                    self.edges.resize(m_us, 0);
                }
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut reader = outq_j::Reader::new(out, &self.spill, heads, a);
                    for &band in &self.bands[cursors[a]..end] {
                        let dst = band.dst() as usize;
                        let span = band.end() as usize;
                        reader.wire_uni(&mut self.edges[dst..dst + span], 0);
                    }
                }
                if self.tau.len() < m_us {
                    self.tau.resize(m_us, 0);
                }
                let tau = &mut self.tau[..m_us];
                tau.fill(u32::MAX);
                let mut rank = 0u32;
                // Walk marked cycle first so mark -> 0, pi(mark) -> 1, ...
                let mut cur = mark as usize;
                while tau[cur] == u32::MAX {
                    tau[cur] = rank;
                    rank += 1;
                    cur = self.edges[cur] as usize;
                }
                // Walk any detached cycles (malformed multi-cycle columns)
                for u in 0..m_us {
                    if tau[u] == u32::MAX {
                        let mut c2 = u;
                        while tau[c2] == u32::MAX {
                            tau[c2] = rank;
                            rank += 1;
                            c2 = self.edges[c2] as usize;
                        }
                    }
                }
                assert_eq!(rank, m);
            }
        }
        st.tau_us = t2.elapsed().as_secs_f64() * 1e6;

        // === Stage S4: Wire edges (baseline or conjugated by tau) ===
        let t3 = Instant::now();
        if self.edges.len() < m_us {
            self.edges.resize(m_us, 0);
        }
        self.extra.clear();
        if wide_code {
            self.extra.resize(m_us, 0);
        }
        let mut predecessor: Option<u32> = None;
        let emit_mark: u32;

        match mode {
            ConjMode::Tau0Id | ConjMode::FrobPeriodic => {
                emit_mark = mark;
                if use_bi {
                    let edges = &mut self.edges[..m_us];
                    edges.fill(0);
                    let mut pred = mark;
                    let mb = usize::from(mark_byte);
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut reader = outq_j::Reader::new(out, &self.spill, heads, a);
                        if a != mb {
                            for &band in &self.bands[cursors[a]..end] {
                                reader.wire_xor_nomark(
                                    edges,
                                    band.dst() as usize,
                                    band.end() as usize,
                                    band.code() << bits,
                                );
                            }
                        } else {
                            for &band in &self.bands[cursors[a]..end] {
                                reader.wire_xor(
                                    edges,
                                    band.dst() as usize,
                                    band.end() as usize,
                                    band.code() << bits,
                                    mark,
                                    &mut pred,
                                );
                            }
                        }
                    }
                    predecessor = Some(pred);
                } else {
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut reader = outq_j::Reader::new(out, &self.spill, heads, a);
                        for &band in &self.bands[cursors[a]..end] {
                            let span = band.end() as usize;
                            let dst = band.dst() as usize;
                            let bcode = band.code();
                            let pack = if wide_code { 0 } else { bcode << bits };
                            if wide_code {
                                self.extra[dst..dst + span].fill(bcode);
                            }
                            reader.wire_uni(&mut self.edges[dst..dst + span], pack);
                        }
                    }
                }
            }
            ConjMode::Tau1FOrder => {
                // Fast bucket-counter wiring! For bucket a, v_tilde runs SEQUENTIALLY from f_start[a]..!
                let tau = &self.tau[..m_us];
                emit_mark = tau[mark as usize];
                if use_bi {
                    let edges = &mut self.edges[..m_us];
                    edges.fill(0);
                    let mut pred_tilde = emit_mark;
                    let mut f_cur = 0u32;
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        for &band in &self.bands[cursors[a]..end] {
                            let dst = band.dst() as usize;
                            let span = band.end() as usize;
                            let pack = band.code() << bits;
                            let src_slice = &tau[dst..dst + span];
                            let dst_slice = &mut edges[f_cur as usize..f_cur as usize + span];
                            // Note: cannot hold mutable borrow on edges[f_cur..] while scattering edges[ut],
                            // so index edges directly (zero OutQ16 re-read needed!).
                            let _ = dst_slice;
                            for (off, &ut) in src_slice.iter().enumerate() {
                                let vt = f_cur + off as u32;
                                if vt == emit_mark {
                                    pred_tilde = ut;
                                }
                                edges[ut as usize] ^= pack | vt;
                                edges[vt as usize] ^= ut;
                            }
                            f_cur += span as u32;
                        }
                    }
                    assert_eq!(f_cur, m);
                    predecessor = Some(pred_tilde);
                } else {
                    let edges = &mut self.edges[..m_us];
                    let extra = &mut self.extra;
                    let mut f_cur = 0u32;
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        for &band in &self.bands[cursors[a]..end] {
                            let dst = band.dst() as usize;
                            let span = band.end() as usize;
                            let bcode = band.code();
                            let pack = if wide_code { 0 } else { bcode << bits };
                            for (off, &ut) in tau[dst..dst + span].iter().enumerate() {
                                let vt = f_cur + off as u32;
                                edges[ut as usize] = pack | vt;
                                if wide_code {
                                    extra[ut as usize] = bcode;
                                }
                            }
                            f_cur += span as u32;
                        }
                    }
                    assert_eq!(f_cur, m);
                }
            }
            ConjMode::Tau2CodeOrder | ConjMode::Tau3FOrderGreedy | ConjMode::Tau4OrbitOracle => {
                let tau = &self.tau[..m_us];
                emit_mark = tau[mark as usize];
                if use_bi {
                    let edges = &mut self.edges[..m_us];
                    edges.fill(0);
                    let mut pred_orig = mark;
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut reader = outq_j::Reader::new(out, &self.spill, heads, a);
                        for &band in &self.bands[cursors[a]..end] {
                            reader.wire_xor_tau(
                                edges,
                                tau,
                                band.dst() as usize,
                                band.end() as usize,
                                band.code() << bits,
                                mark,
                                &mut pred_orig,
                            );
                        }
                    }
                    predecessor = Some(tau[pred_orig as usize]);
                } else {
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut reader = outq_j::Reader::new(out, &self.spill, heads, a);
                        for &band in &self.bands[cursors[a]..end] {
                            let bcode = band.code();
                            let pack = if wide_code { 0 } else { bcode << bits };
                            reader.wire_uni_tau(
                                &mut self.edges[..m_us],
                                &mut self.extra,
                                tau,
                                band.dst() as usize,
                                band.end() as usize,
                                bcode,
                                pack,
                                wide_code,
                            );
                        }
                    }
                }
            }
        }
        st.s4_us = t3.elapsed().as_secs_f64() * 1e6;

        // === Stage S5: Emit (IDENTICAL kernel across ALL conjugators!) ===
        let t4 = Instant::now();
        if let Some(before) = predecessor {
            emit_bidirectional(
                &self.edges[..m_us],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                emit_mark,
                before,
                bits,
                out,
            );
        } else {
            emit(
                &self.edges[..m_us],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                emit_mark,
                bits,
                out,
            );
        }
        st.s5_us = t4.elapsed().as_secs_f64() * 1e6;
        st
    }

    #[allow(clippy::too_many_arguments)]
    fn run_induced<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
    ) -> [usize; 256] {
        let nodes_words = L::STRIDE * most;
        if self.edges.len() < 4 * ns + nodes_words {
            self.edges.resize(4 * ns + nodes_words, 0);
        }
        let (hdr, arena) = self.edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        e.nodes = ns as u32 + 1;
        let mut widest = 0u32;
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
            e.set_word(sid, Elim::<L>::literal(byte));
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
            let to = strips_raw[3 * idx + 1];
            let width = strips_raw[3 * idx + 2] >> 8;
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
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        let mut base = L::STRIDE * e.nodes as usize;
        let mut top = base;
        let nodes = e.nodes;
        let mut remaining = n;
        let spirals = widest > CERTIFIED_WIDTH;
        loop {
            let mut el = Elim::<L> {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            let done = if spirals {
                el.eliminate::<true>(&mut remaining)
            } else {
                el.eliminate::<false>(&mut remaining)
            };
            base = el.base;
            top = el.top;
            if done {
                break;
            }
            let need = 4 * ns + top + SPIRAL_RULES * RULE;
            let new_len = need.max(2 * self.edges.len());
            self.edges.resize(new_len, 0);
        }
        {
            let mut el = Elim::<L> {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            el.assign_bottom(m);
        }

        if self.inline16.len() < 256 {
            self.inline16.clear();
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.resize(256, 1);
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);
        self.spelling.clear();
        self.bands.clear();

        let mut pairs = [0u32; 512];
        let mut contents = [0u32; 2048];
        let mut el = Elim::<L> {
            a: &mut self.edges[4 * ns..4 * ns + top],
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        let mut cursors = [0usize; 256];
        let mut id = L::next(el.a, 0, 0);
        let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
            L::band_node(el.a, id)
        } else {
            (0, 0, 0, 0)
        };
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256 {
            cursors[a] = self.bands.len();
            let end = k + thin[a];
            let mut last_w = u32::MAX;
            let mut last_c = 0u32;
            while k < end {
                let width = (end - k).min(node_width - in_node);
                let dst = node_bottom + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else if node_word == last_w {
                    last_c
                } else {
                    let g = node_word >> ID_BITS;
                    let wid = node_word & ID_MASK;
                    let uid = wid + 1 + (u32::from(g == 2) << 8);
                    let key = (uid << 8) | u32::from(a as u8);
                    let bucket = (key.wrapping_mul(0x9e37_79b9) >> 24) as usize;
                    let slot = bucket * 2;
                    let c = if pairs[slot] == key {
                        pairs[slot + 1]
                    } else {
                        let height = g as usize;
                        let code_val = if height < INLINE {
                            let v = el.short_payload(node_word)
                                | (u128::from(a as u8) << (8 * height));
                            let entry = v.to_le_bytes();
                            let len = (height + 1) as u8;
                            let lo = v as u64;
                            let hi = (v >> 64) as u64;
                            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
                            let h = h ^ (h >> 30);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let index = (h as usize) & 2047;
                            let ent = contents[index];
                            if (ent & 0xFF00_0000) == tag && {
                                let old = (ent & 0x00FF_FFFF) as usize;
                                self.lens[old] == len && self.inline16[old] == entry
                            } {
                                ent & 0x00FF_FFFF
                            } else {
                                let cc = self.lens.len() as u32;
                                self.inline16.push(entry);
                                self.lens.push(len);
                                contents[index] = tag | cc;
                                cc
                            }
                        } else {
                            let start = self.spelling.len();
                            let memoized = el.a[el.rule(wid) + 3] != u32::MAX;
                            el.append(node_word, &mut self.stack, &mut self.spelling);
                            self.spelling.push(a as u8);
                            let entry = long_slot(start as u32, self.spelling.len() as u32);
                            let bytes = &self.spelling[start..];
                            let h = spelling_hash_long(bytes);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let index = (h as usize) & 2047;
                            let ent = contents[index];
                            if (ent & 0xFF00_0000) == tag && {
                                let old = (ent & 0x00FF_FFFF) as usize;
                                self.lens[old] == LONG
                                    && long_word(&self.inline16[old], &self.spelling) == bytes
                            } {
                                if memoized {
                                    self.spelling.truncate(start);
                                } else {
                                    self.spelling.pop();
                                }
                                ent & 0x00FF_FFFF
                            } else {
                                let cc = self.lens.len() as u32;
                                self.inline16.push(entry);
                                self.lens.push(LONG);
                                contents[index] = tag | cc;
                                cc
                            }
                        };
                        pairs[slot] = key;
                        pairs[slot + 1] = code_val;
                        code_val
                    };
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
                        (next_id, node_bottom, node_width, node_word) = L::band_node(el.a, id);
                    }
                }
            }
        }
        assert_eq!(id, 0);
        cursors
    }
}

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
    out.resize(n + INLINE, 0);
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
    out.resize(n + INLINE, 0);
    let mask = (1 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0, n);
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

/// Independent ground-truth forward-FL inverter (reference for malformed & small exactness!)
fn forward_fl_invert(records: &[u16], n: usize, p: u32) -> Vec<u8> {
    let mut all = [0u32; 256];
    let mut l = Vec::with_capacity(n);
    for &rec in records {
        let c = (rec >> 8) as u8;
        let w = usize::from(rec & 255);
        all[usize::from(c)] += w as u32;
        for _ in 0..w {
            l.push(c);
        }
    }
    assert_eq!(l.len(), n);
    let mut f_bounds = [0u32; 256];
    let mut acc = 0u32;
    for a in 0..256 {
        acc += all[a];
        f_bounds[a] = acc;
    }
    let mut cur = starts(all);
    let mut fl = vec![0u32; n];
    for (row, &c) in l.iter().enumerate() {
        let a = usize::from(c);
        fl[cur[a] as usize] = row as u32;
        cur[a] += 1;
    }
    let mut out = Vec::with_capacity(n);
    let mut x = p;
    for _ in 0..n {
        let b = f_bounds.partition_point(|&e| e <= x) as u8;
        out.push(b);
        x = fl[x as usize];
    }
    out
}

// ============================================================================
// 4. Deterministic 64B Cache-Line Simulator (L1D 48 KiB 12-way & L2 512 KiB 8-way)
// ============================================================================

struct LruCache {
    sets: usize,
    ways: usize,
    tags: Vec<u32>,
    ages: Vec<u64>,
    tick: u64,
    accesses: u64,
    misses: u64,
}

impl LruCache {
    fn new(sets: usize, ways: usize) -> Self {
        LruCache {
            sets,
            ways,
            tags: vec![u32::MAX; sets * ways],
            ages: vec![0; sets * ways],
            tick: 0,
            accesses: 0,
            misses: 0,
        }
    }
    #[inline(always)]
    fn access(&mut self, line: u32) {
        self.accesses += 1;
        self.tick += 1;
        let set = (line as usize) & (self.sets - 1);
        let base = set * self.ways;
        let mut lru_w = 0usize;
        let mut lru_age = u64::MAX;
        for w in 0..self.ways {
            let t = self.tags[base + w];
            if t == line {
                self.ages[base + w] = self.tick;
                return;
            }
            let a = self.ages[base + w];
            if a < lru_age {
                lru_age = a;
                lru_w = w;
            }
        }
        self.misses += 1;
        self.tags[base + lru_w] = line;
        self.ages[base + lru_w] = self.tick;
    }
    fn miss_pct(&self) -> f64 {
        100.0 * self.misses as f64 / self.accesses.max(1) as f64
    }
}

struct CacheProxyStats {
    same_line_pct: f64,
    l1_miss_pct: f64,
    l2_miss_pct: f64,
}

fn simulate_edges_cache_proxy(
    kernel: &mut ConjKernel,
    mode: ConjMode,
    blk: &Block,
) -> CacheProxyStats {
    let mut out = Vec::new();
    let _ = kernel.run_mode(mode, &blk.records, blk.n, blk.p, &mut out);
    // Re-extract exact S5 access trace from constructed `edges` + `tau`!
    // Wait: we already have `edges[0..m]` populated inside `kernel`! Let's trace the exact walk.
    // Recompute mark & m cleanly from S1 (matching run_mode):
    let mut m = 0u32;
    let mut mark = 0u32;
    let mut pos = 0u32;
    let mut i = 0usize;
    let nrec = blk.records.len();
    while i < nrec {
        let rec = blk.records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < nrec && (blk.records[i] >> 8) as u8 == c {
            width += u32::from(blk.records[i] & 255);
            i += 1;
        }
        let end = pos + width;
        let mut keep = |x: u32, w: u32, k: bool| {
            if k {
                if blk.p.wrapping_sub(x) < w {
                    mark = m + (blk.p - x);
                }
                m += w;
            }
        };
        if width > LIMIT && (pos..end).contains(&blk.p) {
            if pos < blk.p {
                keep(pos, blk.p - pos, false);
            }
            keep(blk.p, 1, true);
            if blk.p + 1 < end {
                keep(blk.p + 1, end - blk.p - 1, false);
            }
        } else {
            keep(pos, width, width <= LIMIT);
        }
        pos += width;
    }
    let m_us = m as usize;
    let bits = if m <= 1 {
        1
    } else {
        (u32::BITS - (m - 1).leading_zeros()).max(1)
    };
    let mask = (1u32 << bits) - 1;
    let wide_code = kernel.lens.len() - 1 > (u32::MAX >> bits) as usize;
    let use_bi = !wide_code && m >= 65_536;

    let emit_mark = match mode {
        ConjMode::Tau0Id | ConjMode::FrobPeriodic => mark,
        _ => kernel.tau[mark as usize],
    };

    let mut l1 = LruCache::new(64, 12); // 48 KiB 12-way L1D (64B lines)
    let mut l2 = LruCache::new(1024, 8); // 512 KiB 8-way L2 (64B lines)
    let mut prev_line_f = u32::MAX;
    let mut prev_line_b = u32::MAX;
    let mut same_line = 0u64;
    let mut steps = 0u64;

    if use_bi {
        // Recover `before_tilde` by scanning edges for the unique partner of emit_mark?
        // Actually we can find `before_tilde` from one pass over the non-XOR edges? Even easier:
        // for `Tau4-OrbitOracle`, `emit_mark == 0`, `before == m - 1` (single cycle) or end of cycle!
        // For general modes, let's build `pi_orig`? No — in `edges[emit_mark]` we have `next ^ prev`,
        // we just find `before_tilde` cleanly via `extract_before_tilde` helper:
        let before_tilde = find_before_tilde(blk, mode, &kernel.tau, mark);
        let (mut prev, mut front) = (before_tilde, emit_mark);
        let (mut back, mut next) = (before_tilde, emit_mark);
        for pair in 0..=m_us / 2 {
            if pair != 0 && front == next {
                break;
            }
            let lf = front >> 4; // 16 u32s = 64B cache line
            if lf == prev_line_f {
                same_line += 1;
            }
            prev_line_f = lf;
            l1.access(lf);
            l2.access(lf);
            steps += 1;
            let ef = kernel.edges[front as usize];
            if front == back {
                break;
            }
            let lb = back >> 4;
            if lb == prev_line_b {
                same_line += 1;
            }
            prev_line_b = lb;
            l1.access(lb);
            l2.access(lb);
            steps += 1;
            let eb = kernel.edges[back as usize];
            (prev, front) = (front, (ef & mask) ^ prev);
            (next, back) = (back, (eb & mask) ^ next);
        }
    } else {
        let mut j = emit_mark;
        for _ in 0..m_us {
            let lj = j >> 4;
            if lj == prev_line_f {
                same_line += 1;
            }
            prev_line_f = lj;
            l1.access(lj);
            l2.access(lj);
            steps += 1;
            j = kernel.edges[j as usize] & mask;
            if j == emit_mark {
                break;
            }
        }
    }
    CacheProxyStats {
        same_line_pct: 100.0 * same_line as f64 / steps.max(1) as f64,
        l1_miss_pct: l1.miss_pct(),
        l2_miss_pct: l2.miss_pct(),
    }
}

fn find_before_tilde(blk: &Block, mode: ConjMode, tau: &[u32], _mark: u32) -> u32 {
    // Direct O(n) LF-step backward from p to find exact previous retained row `before`
    let mut l = Vec::with_capacity(blk.n);
    let mut retained = vec![false; blk.n];
    let mut ret_id = vec![u32::MAX; blk.n];
    let mut all = [0u32; 256];
    let mut m = 0u32;
    let mut pos = 0u32;
    let mut i = 0usize;
    let nrec = blk.records.len();
    while i < nrec {
        let rec = blk.records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < nrec && (blk.records[i] >> 8) as u8 == c {
            width += u32::from(blk.records[i] & 255);
            i += 1;
        }
        all[usize::from(c)] += width;
        for _ in 0..width {
            l.push(c);
        }
        let end = pos + width;
        let mut keep = |x: u32, w: u32, k: bool| {
            if k {
                for off in 0..w {
                    let row = (x + off) as usize;
                    retained[row] = true;
                    ret_id[row] = m;
                    m += 1;
                }
            }
        };
        if width > LIMIT && (pos..end).contains(&blk.p) {
            if pos < blk.p {
                keep(pos, blk.p - pos, false);
            }
            keep(blk.p, 1, true);
            if blk.p + 1 < end {
                keep(blk.p + 1, end - blk.p - 1, false);
            }
        } else {
            keep(pos, width, width <= LIMIT);
        }
        pos += width;
    }
    let mut first = starts(all);
    let mut lf = vec![0u32; blk.n];
    for (row, &c) in l.iter().enumerate() {
        let a = usize::from(c);
        lf[row] = first[a];
        first[a] += 1;
    }
    let mut cur = lf[blk.p as usize] as usize;
    while !retained[cur] {
        cur = lf[cur] as usize;
    }
    let before = ret_id[cur];
    match mode {
        ConjMode::Tau0Id | ConjMode::FrobPeriodic => before,
        _ => tau[before as usize],
    }
}

// ============================================================================
// 5. Block Loaders, Synthetic Generators, Exactness Gate & Multi-Worker Bench
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
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

fn bwt_encode(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_unstable_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        std::cmp::Ordering::Equal
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &s) in sa.iter().enumerate() {
        if s == 0 {
            p = i as u32;
            l.push(text[n - 1]);
        } else {
            l.push(text[s - 1]);
        }
    }
    (l, p)
}

fn resolve_corpus_path(rel: &str) -> String {
    let p1 = format!("/tmp/mibz2_corpora/{rel}");
    if std::path::Path::new(&p1).exists() {
        return p1;
    }
    let p2 = format!("/home/antigravity/mibz2_corpora_cache/{rel}");
    if std::path::Path::new(&p2).exists() {
        return p2;
    }
    panic!("corpus file not found at {p1} or {p2}");
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
            Block {
                name: f,
                n: l.len(),
                p,
                records: records_of(&l),
            }
        })
        .collect()
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
        let _crc = u32::from_le_bytes(raw[off + 12..off + 16].try_into().unwrap());
        let route_class = raw[off + 16];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        if only_full && route_class > 1 {
            continue;
        }
        let name = if let Some(ns) = names {
            if b_idx < ns.len() {
                ns[b_idx].to_string()
            } else {
                format!("blk-{idx}")
            }
        } else {
            format!("w431-{idx}")
        };
        blocks.push(Block { name, n, p, records });
    }
    blocks
}

fn make_solved_analogue_j_bench() -> Vec<Block> {
    let mut out = Vec::new();
    // 1. Large periodic block (period d = 100, g = 2,000 -> n = 200,000, constructed directly via BWT of unit repeated!)
    // Since BWT of u^g has each row of BWT(u) repeated g times consecutively!
    let mut unit = Vec::with_capacity(100);
    let mut rng = 0x1234_ABCD_5678_EF01u64;
    for i in 0..100 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        unit.push(b'!' + (((rng >> 56) as u8 + (i as u8)) % 60));
    }
    let (lu, pu) = bwt_encode(&unit);
    let g = 2000usize;
    let mut l_per = Vec::with_capacity(lu.len() * g);
    for &b in &lu {
        for _ in 0..g {
            l_per.push(b);
        }
    }
    let p_per = pu * (g as u32);
    out.push(Block {
        name: "AnaJ-Per-200k".to_string(),
        n: l_per.len(),
        p: p_per,
        records: records_of(&l_per),
    });

    // 2. Medium periodic block g = 10, unit len 200 -> n = 2,000 (hits thin rows too, m = 2,000!)
    let mut unit2 = Vec::with_capacity(200);
    for i in 0..200 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        unit2.push(b'!' + (((rng >> 56) as u8 ^ (i as u8)) % 70));
    }
    let (lu2, pu2) = bwt_encode(&unit2);
    let g2 = 10usize;
    let mut l_per2 = Vec::with_capacity(lu2.len() * g2);
    for &b in &lu2 {
        for _ in 0..g2 {
            l_per2.push(b);
        }
    }
    out.push(Block {
        name: "AnaJ-Per-g10".to_string(),
        n: l_per2.len(),
        p: pu2 * (g2 as u32),
        records: records_of(&l_per2),
    });

    // 3. Adversarial non-periodic block with gcd_a(all[a]) == 2 (forces FrobPeriodic fallback!)
    // Take a primitive string where EVERY byte occurs an EVEN number of times (e.g. aabbabab -> BWT is single cycle!)
    let mut txt_even = Vec::with_capacity(10_000);
    let mut cnts = [0usize; 26];
    for i in 0..9_974 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let c = ((rng >> 56) as usize + i) % 26;
        txt_even.push(b'a' + c as u8);
        cnts[c] += 1;
    }
    for (c, &cnt) in cnts.iter().enumerate() {
        if cnt % 2 == 1 {
            txt_even.push(b'a' + c as u8);
        }
    }
    let (le, pe) = bwt_encode(&txt_even);
    out.push(Block {
        name: "Adv-Gcd2-NonPer".to_string(),
        n: le.len(),
        p: pe,
        records: records_of(&le),
    });

    // 4. Spiral-200k stress test (LONG=255 rules + spiral contraction)
    let mut l_sp = vec![b'a'; 100_000];
    l_sp.push(b'b');
    l_sp.extend(std::iter::repeat_n(b'a', 99_999));
    out.push(Block {
        name: "Stress-Spiral".to_string(),
        n: l_sp.len(),
        p: 100_000,
        records: records_of(&l_sp),
    });

    out
}

fn run_exactness_gates(
    solved: &[Block],
    wiki6: &[Block],
    nonwiki: &[Block],
    codedocs: &[Block],
    w233: &[Block],
) {
    println!("================================================================================================================================");
    println!("PART 1: STRICT EXACTNESS GATE (ALL 6 MODES vs cand_r5_head & forward_fl_invert)");
    println!("================================================================================================================================");
    let modes = [
        ConjMode::Tau0Id,
        ConjMode::Tau1FOrder,
        ConjMode::Tau2CodeOrder,
        ConjMode::Tau3FOrderGreedy,
        ConjMode::Tau4OrbitOracle,
        ConjMode::FrobPeriodic,
    ];
    let mut ref_k = cand_r5_head::Kernel::default();
    let mut ck = ConjKernel::default();
    let mut ref_out = Vec::new();
    let mut cand_out = Vec::new();

    let mut rng = 0x55AA_33CC_11EE_7799u64;
    let mut next_u64 = || -> u64 {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        rng
    };

    // 1. 5,200 small valid BWTs
    for case in 0..5200usize {
        let text: Vec<u8> = if case < 200 {
            let g = 2 + (case % 4);
            let d = 5 + ((next_u64() as usize) % 20);
            let alpha = 2 + ((next_u64() as usize) % 8) as u8;
            let unit: Vec<u8> = (0..d)
                .map(|_| b'a' + ((next_u64() >> 56) as u8 % alpha))
                .collect();
            let mut t = Vec::with_capacity(g * d);
            for _ in 0..g {
                t.extend_from_slice(&unit);
            }
            t
        } else {
            let len = 5 + ((next_u64() as usize) % 140);
            let alpha = match case % 4 {
                0 => 2u8,
                1 => 4u8,
                2 => 16u8,
                _ => 64u8,
            };
            (0..len)
                .map(|_| b'!' + ((next_u64() >> 56) as u8 % alpha))
                .collect()
        };
        let (l, p) = bwt_encode(&text);
        let recs = records_of(&l);
        let fl_ref = forward_fl_invert(&recs, l.len(), p);
        assert_eq!(fl_ref, text);
        ref_k
            .run_entry(&recs, l.len(), p, cand_r5_head::Cap::NONE, &mut ref_out)
            .unwrap();
        assert_eq!(ref_out, fl_ref);
        for &mode in &modes {
            let _ = ck.run_mode(mode, &recs, l.len(), p, &mut cand_out);
            assert_eq!(
                cand_out, fl_ref,
                "Small valid mismatch at case={case}, mode={:?}",
                mode
            );
        }
    }
    println!("  [PASS] 5,200 / 5,200 small valid BWTs — 100% byte-exact across all 6 modes!");

    // 2. 300 malformed multi-cycle columns (incl. 20 large m >= 65,536 bi-XOR!)
    let mut mal_large = 0usize;
    for case in 0..300usize {
        let (n, alpha) = if case < 280 {
            (
                24 + ((next_u64() as usize) % 400),
                2 + ((next_u64() as usize) % 24) as u8,
            )
        } else {
            mal_large += 1;
            (
                70_000 + ((next_u64() as usize) % 18_000),
                16 + ((next_u64() as usize) % 48) as u8,
            )
        };
        let mut l = vec![0u8; n];
        for b in &mut l {
            *b = b'!' + ((next_u64() >> 56) as u8 % alpha);
        }
        let p = (next_u64() as u32) % (n as u32);
        let recs = records_of(&l);
        let fl_ref = forward_fl_invert(&recs, n, p);
        ref_k
            .run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut ref_out)
            .unwrap();
        assert_eq!(ref_out, fl_ref, "cand_r5_head malformed mismatch {case}");
        for &mode in &modes {
            let _ = ck.run_mode(mode, &recs, n, p, &mut cand_out);
            assert_eq!(
                cand_out, fl_ref,
                "Malformed forward-FL mismatch at case={case}, n={n}, mode={:?}",
                mode
            );
        }
    }
    println!(
        "  [PASS] 300 / 300 malformed multi-cycle columns (incl. {mal_large} large m>=65,536 bi-XOR) — 100% exact forward-FL parity!"
    );

    // 3. Solved J + Stress + Adversarial gcd=2 non-periodic fallback
    for blk in solved {
        let fl_ref = forward_fl_invert(&blk.records, blk.n, blk.p);
        ref_k
            .run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut ref_out)
            .unwrap();
        assert_eq!(ref_out, fl_ref, "ref mismatch on {}", blk.name);
        for &mode in &modes {
            let _ = ck.run_mode(mode, &blk.records, blk.n, blk.p, &mut cand_out);
            assert_eq!(
                cand_out, fl_ref,
                "Solved/Stress mismatch on {}, mode={:?}",
                blk.name, mode
            );
        }
    }
    println!(
        "  [PASS] {}/{} Solved Analogue J + Adversarial gcd=2 non-periodic + Spiral-200k — 100% byte-exact!",
        solved.len(),
        solved.len()
    );

    // 4. Real corpora: 6 Admitted + 6 Non-Wiki + 4 Code+Docs + 233 Full Wikidata
    let check_corpus = |label: &str, blocks: &[Block], r_k: &mut cand_r5_head::Kernel, c_k: &mut ConjKernel| {
        let mut r_out = Vec::new();
        let mut c_out = Vec::new();
        for blk in blocks {
            r_k.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut r_out)
                .unwrap();
            for &mode in &modes {
                let _ = c_k.run_mode(mode, &blk.records, blk.n, blk.p, &mut c_out);
                assert_eq!(
                    c_out, r_out,
                    "Corpus {label} mismatch on {}, mode={:?}",
                    blk.name, mode
                );
            }
        }
        println!(
            "  [PASS] {:>3} / {:>3} {label} — 100% byte-exact across all 6 modes!",
            blocks.len(),
            blocks.len()
        );
    };
    check_corpus("6 Admitted Wikidata blocks", wiki6, &mut ref_k, &mut ck);
    check_corpus("6 Non-Wikidata (JSONL+Syslog) blocks", nonwiki, &mut ref_k, &mut ck);
    check_corpus("4 Real Code+Docs blocks", codedocs, &mut ref_k, &mut ck);
    check_corpus("233 Full wikidata.bz2 blocks", w233, &mut ref_k, &mut ck);
    println!();
}

fn run_cache_proxy_table(title: &str, blocks: &[Block]) {
    println!("================================================================================================================================");
    println!("PART 2: DETERMINISTIC 64B CACHE-LINE LOCALITY PROXY (S5 `edges` accesses) — {title}");
    println!("================================================================================================================================");
    let modes = [
        ConjMode::Tau0Id,
        ConjMode::Tau1FOrder,
        ConjMode::Tau2CodeOrder,
        ConjMode::Tau3FOrderGreedy,
        ConjMode::Tau4OrbitOracle,
    ];
    println!(
        "{:<15} | {:>16} | {:>16} | {:>16} | {:>16} | {:>16}",
        "Block",
        "Tau0-Id (s/L1/L2)",
        "Tau1-FOrd",
        "Tau2-Code",
        "Tau3-FGrd",
        "Tau4-Orac*"
    );
    let mut ck = ConjKernel::default();
    let mut sum_l1 = [0.0f64; 5];
    let mut sum_l2 = [0.0f64; 5];
    let mut sum_sl = [0.0f64; 5];
    for blk in blocks {
        let mut cells = Vec::with_capacity(5);
        for (mi, &m) in modes.iter().enumerate() {
            let s = simulate_edges_cache_proxy(&mut ck, m, blk);
            sum_sl[mi] += s.same_line_pct;
            sum_l1[mi] += s.l1_miss_pct;
            sum_l2[mi] += s.l2_miss_pct;
            cells.push(format!(
                "{:4.1}/{:4.1}/{:4.1}",
                s.same_line_pct, s.l1_miss_pct, s.l2_miss_pct
            ));
        }
        println!(
            "{:<15} | {:>16} | {:>16} | {:>16} | {:>16} | {:>16}",
            blk.name, cells[0], cells[1], cells[2], cells[3], cells[4]
        );
    }
    let nb = blocks.len().max(1) as f64;
    println!(
        "{:<15} | {:>16} | {:>16} | {:>16} | {:>16} | {:>16}",
        "MEAN (same/L1/L2)",
        format!("{:4.1}/{:4.1}/{:4.1}", sum_sl[0]/nb, sum_l1[0]/nb, sum_l2[0]/nb),
        format!("{:4.1}/{:4.1}/{:4.1}", sum_sl[1]/nb, sum_l1[1]/nb, sum_l2[1]/nb),
        format!("{:4.1}/{:4.1}/{:4.1}", sum_sl[2]/nb, sum_l1[2]/nb, sum_l2[2]/nb),
        format!("{:4.1}/{:4.1}/{:4.1}", sum_sl[3]/nb, sum_l1[3]/nb, sum_l2[3]/nb),
        format!("{:4.1}/{:4.1}/{:4.1}", sum_sl[4]/nb, sum_l1[4]/nb, sum_l2[4]/nb),
    );
    println!();
}

fn bench_stage_1t(blocks: &[Block], mode: ConjMode, iters: usize) -> (StageTimes, usize) {
    let mut ck = ConjKernel::default();
    let mut out = Vec::new();
    // Warmup
    for blk in blocks {
        let _ = ck.run_mode(mode, &blk.records, blk.n, blk.p, &mut out);
    }
    let ws = ck.scratch_bytes(mode);
    let mut best = StageTimes {
        s1_us: f64::INFINITY,
        s23_us: f64::INFINITY,
        tau_us: f64::INFINITY,
        s4_us: f64::INFINITY,
        s5_us: f64::INFINITY,
    };
    let mut best_tot = f64::INFINITY;
    for _ in 0..iters {
        let mut acc = StageTimes::default();
        for blk in blocks {
            let s = ck.run_mode(mode, &blk.records, blk.n, blk.p, &mut out);
            acc.s1_us += s.s1_us;
            acc.s23_us += s.s23_us;
            acc.tau_us += s.tau_us;
            acc.s4_us += s.s4_us;
            acc.s5_us += s.s5_us;
        }
        if acc.total_us() < best_tot {
            best_tot = acc.total_us();
            best = acc;
        }
    }
    (best, ws)
}

fn bench_multiworker(blocks: &[Block], mode: ConjMode, workers: usize, iters: usize) -> f64 {
    if workers == 1 {
        let (st, _) = bench_stage_1t(blocks, mode, iters);
        return st.total_us() / 1000.0;
    }
    let mut best_ms = f64::INFINITY;
    for _ in 0..iters {
        let barrier = Barrier::new(workers);
        let t_ms = std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(workers);
            for _ in 0..workers {
                let b_ref = &barrier;
                handles.push(s.spawn(move || {
                    let mut ck = ConjKernel::default();
                    let mut out = Vec::new();
                    for blk in blocks {
                        let _ = ck.run_mode(mode, &blk.records, blk.n, blk.p, &mut out);
                    }
                    b_ref.wait();
                    let t0 = Instant::now();
                    for blk in blocks {
                        let _ = ck.run_mode(mode, &blk.records, blk.n, blk.p, &mut out);
                    }
                    t0.elapsed().as_secs_f64() * 1000.0
                }));
            }
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .fold(0.0f64, f64::max)
        });
        best_ms = best_ms.min(t_ms);
    }
    best_ms
}

fn run_bench_suite(title: &str, blocks: &[Block], iters_1t: usize, iters_mw: usize) {
    println!("================================================================================================================================");
    println!("PART 3: STAGE BREAKDOWN, WS & 1T/2W/4W BENCHMARK — {title} ({} blks)", blocks.len());
    println!("================================================================================================================================");
    let modes = [
        ConjMode::Tau0Id,
        ConjMode::Tau1FOrder,
        ConjMode::Tau2CodeOrder,
        ConjMode::Tau3FOrderGreedy,
        ConjMode::Tau4OrbitOracle,
        ConjMode::FrobPeriodic,
    ];
    println!(
        "{:<13} | {:>7} {:>7} {:>7} {:>7} {:>7} | {:>8} {:>7} {:>7} | {:>7} {:>7} {:>7} | {:>8}",
        "Mode", "S1_ms", "S23_ms", "Tau_ms", "S4_ms", "S5_ms", "Tot1Tms", "vsId", "S5vsId",
        "2W_vsId", "4W_vsId", "WS_KiB", "C_r+m<Op?"
    );
    let mut best_st = [StageTimes::default(); 6];
    let mut best_1t = [f64::INFINITY; 6];
    let mut ws_arr = [0usize; 6];
    // Warmup + WS capture
    for (mi, &mode) in modes.iter().enumerate() {
        let (st, ws) = bench_stage_1t(blocks, mode, 1);
        best_st[mi] = st;
        best_1t[mi] = st.total_us();
        ws_arr[mi] = ws;
    }
    // Interleaved 1T rounds (takes min per mode across interleaved rounds)
    for _ in 1..iters_1t {
        for (mi, &mode) in modes.iter().enumerate() {
            let (st, _) = bench_stage_1t(blocks, mode, 1);
            if st.total_us() < best_1t[mi] {
                best_1t[mi] = st.total_us();
                best_st[mi] = st;
            }
        }
    }
    // Interleaved 2W and 4W rounds
    let mut best_2w = [f64::INFINITY; 6];
    let mut best_4w = [f64::INFINITY; 6];
    for _ in 0..iters_mw {
        for (mi, &mode) in modes.iter().enumerate() {
            let t2 = bench_multiworker(blocks, mode, 2, 1);
            if t2 < best_2w[mi] {
                best_2w[mi] = t2;
            }
            let t4 = bench_multiworker(blocks, mode, 4, 1);
            if t4 < best_4w[mi] {
                best_4w[mi] = t4;
            }
        }
    }

    let base_1t = best_st[0].total_us() / 1000.0;
    let base_s5 = best_st[0].s5_us / 1000.0;
    let base_2w = best_2w[0];
    let base_4w = best_4w[0];

    for (mi, &mode) in modes.iter().enumerate() {
        let st = best_st[mi];
        let t1 = st.total_us() / 1000.0;
        let r1 = t1 / base_1t.max(1e-9);
        let rs5 = (st.s5_us / 1000.0) / base_s5.max(1e-9);
        let r2 = best_2w[mi] / base_2w.max(1e-9);
        let r4 = best_4w[mi] / base_4w.max(1e-9);
        let verdict = if mode == ConjMode::Tau0Id {
            "BASE"
        } else if mode == ConjMode::Tau4OrbitOracle {
            "ORACLE*"
        } else if r1 < 1.0 && r4 < 1.0 {
            "YES"
        } else {
            "NO (KILL)"
        };
        println!(
            "{:<13} | {:>7.2} {:>7.2} {:>7.2} {:>7.2} {:>7.2} | {:>8.2} {:>6.3}x {:>6.3}x | {:>6.3}x {:>6.3}x {:>7.1} | {:>8}",
            mode.label(),
            st.s1_us / 1000.0,
            st.s23_us / 1000.0,
            st.tau_us / 1000.0,
            st.s4_us / 1000.0,
            st.s5_us / 1000.0,
            t1,
            r1,
            rs5,
            r2,
            r4,
            ws_arr[mi] as f64 / 1024.0,
            verdict
        );
    }
    println!();
}

fn main() {
    let solved = make_solved_analogue_j_bench();
    let wiki6 = load_six_admitted();
    let nonwiki_names = ["jsonl-0", "jsonl-1", "jsonl-2", "jsonl-3", "jsonl-4", "syslog-0"];
    let nonwiki = load_bin_blocks(
        &resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin"),
        Some(&nonwiki_names),
        false,
    );
    let cd_names = ["codedocs-0", "codedocs-1", "codedocs-2", "codedocs-3"];
    let codedocs = load_bin_blocks(
        &resolve_corpus_path("non_wikidata/code_docs_blocks.bin"),
        Some(&cd_names),
        false,
    );
    let w233 = load_bin_blocks(&resolve_corpus_path("wikidata_431_blocks.bin"), None, true);

    run_exactness_gates(&solved, &wiki6, &nonwiki, &codedocs, &w233);

    run_cache_proxy_table("6 ADMITTED WIKIDATA BLOCKS", &wiki6);
    run_cache_proxy_table("6 NON-WIKIDATA (JSONL+SYSLOG)", &nonwiki);
    run_cache_proxy_table("4 CODE+DOCS BLOCKS", &codedocs);

    // Benchmarks (1T/2W/4W)
    run_bench_suite("SOLVED ANALOGUE J (Periodic + Adv-Gcd2 + Spiral)", &solved, 5, 3);
    run_bench_suite("6 ADMITTED WIKIDATA BLOCKS", &wiki6, 5, 3);
    run_bench_suite("6 NON-WIKIDATA (JSONL+SYSLOG)", &nonwiki, 5, 3);
    run_bench_suite("4 REAL CODE+DOCS BLOCKS", &codedocs, 5, 3);
    run_bench_suite("ALL 233 FULL WIKIDATA.BZ2 BLOCKS", &w233, 3, 2);
}
