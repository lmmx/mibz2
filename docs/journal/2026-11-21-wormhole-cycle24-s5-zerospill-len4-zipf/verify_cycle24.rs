//! Wormhole (Neon) Cycle 24 — `S5 Emit` Attack #1: Zero-Spill Loop-Inversion, Edge-Packed GPR Len4/Len3, and Zipf Code Packing
//!
//! Ablates on top of Cycle 23 champion `G2: S* + F4 + GradedOperad` (`MODE = 9`):
//!   - `M0`: `0.5.11` clone (`MODE = 0`)
//!   - `G2`: Cycle 23 champion (`GradedOperad + F4 NoSpill`, `MODE = 9`)
//!   - `E1`: `G2 + Loop-Inversion True Zero-Spill NoCall emit` (`MODE = 11`)
//!   - `E2a`: `E1 + Edge-Packed GPR Len4` (`bits + code_bits <= 28`, `MODE = 12`)
//!   - `E2b`: `E1 + Edge-Packed GPR Len4/Len3` (`<= 28` Len4, `== 29` Len3, `MODE = 13`)
//!   - `E3`: `E1 + Zipf Frequency-Ordered inline16 Code Remap` (`MODE = 14`)
//!
//! All verified against `origin/master` `8ff1187af` `OpenCells` (`A1:Master`, `A2:Master` A/A control).

#![forbid(unsafe_code)]

//! Wormhole/Neon Cycle 21 — Independent Surgical Audit of `Neon-C29` vs `#714` + `L4/L5`
//! against `origin/master` (`8ff1187af6a920f7d57fd338680db3a5633a3ac5`, `mibz2 0.5.11`).
//!
//! Arms compiled into one `#![forbid(unsafe_code)]` binary and invoked via `InverseBwt::invert_runs`:
//!   - `A1: Master`       — `origin/master` (`8ff1187af`) `mibz2::bwt::OpenCells`
//!   - `A2: Ctrl`         — A/A control (`mibz2::bwt::OpenCells`)
//!   - `M0: Clone`        — same-crate verbatim `0.5.11` `OpenCells` (isolates cross-crate inlining)
//!   - `W1: #714+L4+L5`   — Wormhole Round 3 (`GradedHandle + pop_tail + Prefix15 + BidiBottom + StripBandMux`)
//!   - `N1: OperadDedup`  — `Neon-C29` `RULE=4 u128` Operad-MN16 + `0 B offsets` + L1D `pair_ht/str_ht` dedup + `pop_tail`
//!   - `N2: +MuxBand64+L4`— `N1` + `MuxStrips` (`edges[0..3*ns]`) + `Band64` (`8 B`) + `48 KiB spelling` + `L4 BidiBottom`
//!   - `N3: +All+Pos (S*)`— `N2` + `Neon-C29` Pass-1 `all = thin + thick_sum` + `pos = (tail << 10) | fill` ($\mu = 1$ stack!)
//!   - `W2: Graded+AllPos`— `#714` `GradedHandle + Lazy15` on `N3`'s exact infrastructure (`Dedup + MuxBand64 + L4 + All+Pos`)
//!   - `N4: Full Neon-C29`— `N3` + `Fused-XOR` ($\mu = 2$ two-ended walk + `wire_xor`, rejected at 4W in `#718`)

use mibz2::bwt::{ForcedOpenCells, InverseBwt, LazyCells, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::env;
use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const CHUNK: usize = 512;
const BYTES: usize = 2 * CHUNK;
const EPOCH: u32 = 1 << 16;
const NONE: u32 = u32::MAX;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const RULE4: usize = 4;
const NARROW_NODES: usize = 1 << 16;

const LAZY_DENSITY: (usize, usize) = (3, 20);
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

const PAIR_BITS: u32 = 10;
const PAIR_LEN: usize = 1 << (PAIR_BITS as usize + 1); // 2048 u32s (8 KiB)
const STR_BITS: u32 = 12;
const STR_MASK: usize = (1 << STR_BITS) - 1;
const STR_LEN: usize = 1 << STR_BITS; // 4096 u32s (16 KiB)
const HT_WORDS: usize = PAIR_LEN + STR_LEN; // 6144 u32s (24 KiB)

#[derive(Clone, Copy, PartialEq, Eq)]
struct Cap {
    m: u32,
    strips: usize,
}
impl Cap {
    const NONE: Cap = Cap {
        m: u32::MAX,
        strips: usize::MAX,
    };
    fn dense(n: usize) -> Cap {
        Cap {
            m: DENSE_M.min((n as u64 * 3 / 10) as u32),
            strips: DENSE_STRIPS,
        }
    }
    fn classify(records: usize, n: usize, p: u32, forced: bool) -> Option<Cap> {
        if n == 0 || n > 900_000 || p as usize >= n {
            None
        } else if forced || (records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n) {
            Some(Cap::NONE)
        } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
            Some(Cap::dense(n))
        } else {
            None
        }
    }
}

// ============================================================================
// 1. SHA-256 Auth & Minimal Valid BZip2 Block Crafter
// ============================================================================

fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
    )
}

#[derive(Default)]
struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}
impl BitWriter {
    fn put(&mut self, v: u64, bits: u32) {
        if bits == 0 {
            return;
        }
        let mask = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
        self.acc = (self.acc << bits) | (v & mask);
        self.n += bits;
        while self.n >= 8 {
            self.n -= 8;
            self.out.push((self.acc >> self.n) as u8);
        }
    }
    fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.out.push((self.acc << (8 - self.n)) as u8);
        }
        self.out
    }
}

fn plain_invert(ll: &[u8], orig_ptr: u32) -> Vec<u8> {
    let n = ll.len();
    let mut counts = [0usize; 256];
    for &c in ll {
        counts[c as usize] += 1;
    }
    let mut cum = [0usize; 256];
    let mut s = 0;
    for i in 0..256 {
        cum[i] = s;
        s += counts[i];
    }
    let mut psi = vec![0u32; n];
    for (i, &c) in ll.iter().enumerate() {
        psi[cum[c as usize]] = i as u32;
        cum[c as usize] += 1;
    }
    let mut p = psi[orig_ptr as usize];
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(ll[p as usize]);
        p = psi[p as usize];
    }
    out
}

fn build_runs(ll: &[u8], orig_ptr: u32, kernels: Kernels) -> Runs {
    let pre_rle1 = plain_invert(ll, orig_ptr);
    let mut unrled = Vec::with_capacity(pre_rle1.len());
    let crc = mibz2::rle::unrle(&pre_rle1, &mut unrled, kernels)
        .map(|_| mibz2::crc::crc32(&unrled, kernels))
        .unwrap_or(0);

    let mut present = [false; 256];
    for &c in ll {
        present[c as usize] = true;
    }
    let mut used: Vec<u8> = (0..=255u8).filter(|&c| present[c as usize]).collect();
    if used.len() < 2 {
        for b in 0..=255u8 {
            if !present[b as usize] {
                used.push(b);
                break;
            }
        }
        used.sort_unstable();
    }
    let mut list = used.clone();
    let mut syms = Vec::with_capacity(ll.len() / 2 + 8);
    let mut run = 0usize;
    let flush = |run: &mut usize, syms: &mut Vec<u16>| {
        let mut r = *run;
        while r > 0 {
            syms.push(((r - 1) & 1) as u16);
            r = (r - 1) >> 1;
        }
        *run = 0;
    };
    for &c in ll {
        let j = list.iter().position(|&x| x == c).unwrap();
        if j == 0 {
            run += 1;
            continue;
        }
        flush(&mut run, &mut syms);
        list.remove(j);
        list.insert(0, c);
        syms.push(j as u16 + 1);
    }
    flush(&mut run, &mut syms);
    syms.push(used.len() as u16 + 1);

    let alpha = used.len() + 2;
    let code_len = (usize::BITS - (alpha - 1).leading_zeros()).max(1);
    let mut w = BitWriter::default();
    w.put(0x425a_6839, 32);
    w.put(0x3141_5926_5359, 48);
    w.put(crc as u64, 32);
    w.put(0, 1);
    w.put(orig_ptr as u64, 24);
    let mut in16 = 0u64;
    for &c in &used {
        in16 |= 0x8000 >> (c / 16);
    }
    w.put(in16, 16);
    for i in 0..16 {
        if in16 & (0x8000 >> i) != 0 {
            let mut word = 0u64;
            for &c in &used {
                if (c / 16) as u64 == i {
                    word |= 0x8000 >> (c % 16);
                }
            }
            w.put(word, 16);
        }
    }
    let n_sel = syms.len().div_ceil(50).max(1);
    w.put(2, 3);
    w.put(n_sel as u64, 15);
    for _ in 0..n_sel {
        w.put(0, 1);
    }
    for _ in 0..2 {
        w.put(code_len as u64, 5);
        for _ in 0..alpha {
            w.put(0, 1);
        }
    }
    for &s in &syms {
        w.put(s as u64, code_len);
    }
    w.put(0x1772_4538_5090, 48);
    w.put(crc as u64, 32);
    let bz2 = w.finish();

    let mut sc = Scanner::new(SliceSource(&bz2));
    let mut runs = Runs::default();
    match sc.next(Vec::new()).expect("valid bz2") {
        Event::Block { symbols, .. } => {
            assert_eq!(symbols.orig_ptr, orig_ptr);
            let n_dec = runs::front(&symbols, &mut runs, kernels).expect("front ok");
            assert_eq!(n_dec, ll.len());
            runs
        }
        _ => panic!("expected block"),
    }
}

// ============================================================================
// 2. OutQ16 (`POS_FUSED = false` master vs `POS_FUSED = true` Neon-C29 Bit-Lattice)
// ============================================================================

#[derive(Default)]
struct Spill {
    next: Vec<u32>,
    ovf: Vec<[u8; BYTES]>,
    cuts: Vec<[u32; 256]>,
}
struct Heads {
    heads: [u32; 256],
    in_out: u32,
}

struct QueuesMaster<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    chunks: u32,
    limit: u32,
    over: bool,
    heads: [u32; 256],
    tails: [u32; 256],
    fill: [u32; 256],
}
impl<'a> QueuesMaster<'a> {
    fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
        let in_out = (out.len() / BYTES) as u32;
        spill.ovf.clear();
        spill.cuts.clear();
        Self {
            out,
            spill,
            in_out,
            chunks: 0,
            limit,
            over: false,
            heads: [NONE; 256],
            tails: [NONE; 256],
            fill: [CHUNK as u32; 256],
        }
    }
    fn heads(&self) -> Heads {
        Heads { heads: self.heads, in_out: self.in_out }
    }
    #[inline(always)]
    fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = f as u32 + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }
    #[cold]
    #[inline(never)]
    fn push_slow(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
        if self.over || m + w > self.limit {
            self.over = true;
            return;
        }
        let mut left = w;
        while left > 0 {
            if self.fill[a] as usize == CHUNK {
                let id = self.chunks;
                self.chunks += 1;
                if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                    self.spill.ovf.push([0; BYTES]);
                }
                if self.spill.next.len() <= id as usize {
                    self.spill.next.resize(id as usize + 1, NONE);
                }
                self.spill.next[id as usize] = NONE;
                if self.heads[a] == NONE {
                    self.heads[a] = id;
                } else {
                    self.spill.next[self.tails[a] as usize] = id;
                }
                self.tails[a] = id;
                self.fill[a] = 0;
            }
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
            let id = self.tails[a];
            let chunk = if id < self.in_out {
                let at = id as usize * BYTES;
                &mut self.out[at..at + BYTES]
            } else {
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            };
            for (i, d) in chunk[2 * f..2 * (f + take as usize)].chunks_exact_mut(2).enumerate() {
                d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
            }
            self.fill[a] += take;
            m += take;
            left -= take;
            if m % EPOCH == 0 {
                let mut cut = *thin;
                cut[a] += w - left;
                self.spill.cuts.push(cut);
            }
        }
    }
}

/// Neon-C29 Bit-Lattice Cursor Queue (`pos[a] = (tail << 10) | fill`, `at = p + (p & 1023)`)
struct QueuesPos<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    in_out_lim: u32,
    chunks: u32,
    limit: u32,
    over: bool,
    heads: [u32; 256],
    pos: [u32; 256],
}
impl<'a> QueuesPos<'a> {
    fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
        let in_out = (out.len() / BYTES) as u32;
        spill.ovf.clear();
        spill.cuts.clear();
        Self {
            out,
            spill,
            in_out,
            in_out_lim: in_out << 10,
            chunks: 0,
            limit,
            over: false,
            heads: [NONE; 256],
            pos: [u32::MAX; 256],
        }
    }
    fn heads(&self) -> Heads {
        Heads { heads: self.heads, in_out: self.in_out }
    }
    #[inline(always)]
    fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let p = self.pos[a];
        let f = (p & 1023) as usize;
        let lo = m & (EPOCH - 1);
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && p < self.in_out_lim {
            let at = p as usize + f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.pos[a] = p + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }
    #[cold]
    #[inline(never)]
    fn push_slow(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
        if self.over || m + w > self.limit {
            self.over = true;
            return;
        }
        let mut left = w;
        while left > 0 {
            let p = self.pos[a];
            if p == u32::MAX || (p & 1023) as usize == CHUNK {
                let id = self.chunks;
                self.chunks += 1;
                if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                    self.spill.ovf.push([0; BYTES]);
                }
                if self.spill.next.len() <= id as usize {
                    self.spill.next.resize(id as usize + 1, NONE);
                }
                self.spill.next[id as usize] = NONE;
                if self.heads[a] == NONE {
                    self.heads[a] = id;
                } else {
                    let prev_tail = (p >> 10) as usize;
                    self.spill.next[prev_tail] = id;
                }
                self.pos[a] = id << 10;
            }
            let p = self.pos[a];
            let f = (p & 1023) as usize;
            let tail = p >> 10;
            let lo = m & (EPOCH - 1);
            let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
            let chunk = if tail < self.in_out {
                let at = tail as usize * BYTES;
                &mut self.out[at..at + BYTES]
            } else {
                &mut self.spill.ovf[(tail - self.in_out) as usize][..]
            };
            for (i, d) in chunk[2 * f..2 * (f + take as usize)].chunks_exact_mut(2).enumerate() {
                d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
            }
            self.pos[a] = p + take;
            m += take;
            left -= take;
            if m % EPOCH == 0 {
                let mut cut = *thin;
                cut[a] += w - left;
                self.spill.cuts.push(cut);
            }
        }
    }
}

struct Reader<'a> {
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
    fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
        Self {
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
    fn wire(&mut self, dst: &mut [u32], pack: u32) {
        let span = dst.len();
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
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
            let hi = pack | (self.epoch << 16);
            let chunk = if self.id < self.in_out {
                let at = self.id as usize * BYTES;
                &self.out[at..at + BYTES]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][..]
            };
            for (e, s) in dst[..take].iter_mut().zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2)) {
                *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
            }
            self.at += take;
            self.read += take as u32;
            dst = &mut dst[take..];
        }
    }
    #[inline(always)]
    fn wire_xor(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
        mark: u32,
        j_neg1: &mut u32,
    ) {
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = self.epoch << 16;
            let at = self.id as usize * BYTES + 2 * self.at;
            let bytes = &self.out[at..at + 2 * span];
            if (mark & !0xFFFF) != hi {
                let mut i = 0usize;
                while i + 4 <= span {
                    let b = &bytes[2 * i..2 * i + 8];
                    let v0 = hi | u32::from(u16::from_le_bytes([b[0], b[1]]));
                    let v1 = hi | u32::from(u16::from_le_bytes([b[2], b[3]]));
                    let v2 = hi | u32::from(u16::from_le_bytes([b[4], b[5]]));
                    let v3 = hi | u32::from(u16::from_le_bytes([b[6], b[7]]));
                    let u0 = dst_start + i;
                    edges[u0] ^= pack | v0;
                    edges[u0 + 1] ^= pack | v1;
                    edges[u0 + 2] ^= pack | v2;
                    edges[u0 + 3] ^= pack | v3;
                    edges[v0 as usize] ^= u0 as u32;
                    edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32;
                    edges[v3 as usize] ^= (u0 + 3) as u32;
                    i += 4;
                }
                while i < span {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                    i += 1;
                }
            } else {
                let mut jn = *j_neg1;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        jn = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                *j_neg1 = jn;
            }
            self.at += span;
            self.read += span as u32;
        } else {
            self.wire_xor_slow(edges, dst_start, span, pack, mark, j_neg1);
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
        j_neg1: &mut u32,
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
            let hi = self.epoch << 16;
            let chunk = if self.id < self.in_out {
                let at = self.id as usize * BYTES;
                &self.out[at..at + BYTES]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][..]
            };
            for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
                let u = dst_start + i;
                let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                if v == mark {
                    *j_neg1 = u as u32;
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
}

// ============================================================================
// 3. Links, Elim (`RULE_MODE`: 0=Master, 1=Graded#714, 2=Operad-MN16)
// ============================================================================

trait Links {
    const STRIDE: usize;
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32);
}
struct Narrow;
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
    const BOTTOM: usize = 1;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 4 + side] & 0xFFFF }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 4 + side] >> 16 }
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
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32) {
        a[id as usize * 4 + side] = prev | (next << 16);
    }
}
impl Links for Wide {
    const STRIDE: usize = 6;
    const BOTTOM: usize = 2;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side] }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side + 1] }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side] = to; }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side + 1] = to; }
    #[inline(always)]
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32) {
        a[id as usize * 6 + 2 * side] = prev;
        a[id as usize * 6 + 2 * side + 1] = next;
    }
}

#[cold]
#[inline(never)]
fn grow(v: &mut Vec<u32>, need: usize) {
    v.resize(need.max(2 * v.len()), 0);
}

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<'a, L: Links> Elim<'a, L> {
    #[inline(always)]
    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes;
        self.nodes += 1;
        let o = L::STRIDE * id as usize;
        self.a[o..o + L::STRIDE - 2].fill(0);
        self.a[o + L::STRIDE - 2] = width;
        self.a[o + L::STRIDE - 1] = word;
        id
    }
    #[inline(always)]
    fn width(&self, id: u32) -> u32 { self.a[L::STRIDE * id as usize + L::STRIDE - 2] }
    #[inline(always)]
    fn set_width(&mut self, id: u32, w: u32) { self.a[L::STRIDE * id as usize + L::STRIDE - 2] = w; }
    #[inline(always)]
    fn word(&self, id: u32) -> u32 { self.a[L::STRIDE * id as usize + L::STRIDE - 1] }
    #[inline(always)]
    fn set_word(&mut self, id: u32, w: u32) { self.a[L::STRIDE * id as usize + L::STRIDE - 1] = w; }
    #[inline(always)]
    fn bottom(&self, id: u32) -> u32 { self.a[L::STRIDE * id as usize + L::BOTTOM] }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, b: u32) { self.a[L::STRIDE * id as usize + L::BOTTOM] = b; }
    #[inline(always)]
    fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
        if list.0 == 0 {
            *list = (id, id);
        } else {
            L::set_next(self.a, list.1, side, id);
            L::set_prev(self.a, id, side, list.1);
            list.1 = id;
        }
    }
    fn close(&mut self, side: usize, term: (u32, u32), ord: (u32, u32)) {
        match (term.0 != 0, ord.0 != 0) {
            (false, false) => {}
            (true, false) => {
                L::set_next(self.a, 0, side, term.0);
                L::set_prev(self.a, term.0, side, 0);
                L::set_next(self.a, term.1, side, 0);
                L::set_prev(self.a, 0, side, term.1);
            }
            (false, true) => {
                L::set_next(self.a, 0, side, ord.0);
                L::set_prev(self.a, ord.0, side, 0);
                L::set_next(self.a, ord.1, side, 0);
                L::set_prev(self.a, 0, side, ord.1);
            }
            (true, true) => {
                L::set_next(self.a, 0, side, term.0);
                L::set_prev(self.a, term.0, side, 0);
                L::set_next(self.a, term.1, side, ord.0);
                L::set_prev(self.a, ord.0, side, term.1);
                L::set_next(self.a, ord.1, side, 0);
                L::set_prev(self.a, 0, side, ord.1);
            }
        }
    }
    #[inline(always)]
    fn cut(&mut self, id: u32, side: usize) {
        let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
        L::set_next(self.a, p, side, n);
        L::set_prev(self.a, n, side, p);
    }
    #[inline(always)]
    fn pop_tail(&mut self, id: u32, side: usize) {
        let p = L::prev(self.a, id, side);
        L::set_next(self.a, p, side, 0);
        L::set_prev(self.a, 0, side, p);
    }
    #[inline(always)]
    fn insert<const SET_LINKS: bool>(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        if SET_LINKS {
            L::set_links(self.a, id, side, at, next);
        } else {
            L::set_prev(self.a, id, side, at);
            L::set_next(self.a, id, side, next);
        }
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }
    #[inline(always)]
    fn short_u64(&self, w: u32) -> u64 {
        if w <= 256 {
            u64::from(w - 1)
        } else {
            let r = self.rule4(w);
            u64::from(self.a[r]) | (u64::from(self.a[r + 1]) << 32)
        }
    }
    #[inline(always)]
    fn short_u128_raw(&self, w: u32) -> u128 {
        if w <= 256 {
            u128::from(w - 1)
        } else {
            let r = self.rule4(w);
            u128::from(self.a[r])
                | (u128::from(self.a[r + 1]) << 32)
                | (u128::from(self.a[r + 2]) << 64)
                | (u128::from(self.a[r + 3]) << 96)
        }
    }
    #[inline(always)]
    fn len_graded_raw(&self, h: u32) -> u32 {
        let l = h >> 22;
        if l < 1023 { l } else { self.a[self.rule4(h & 0x003F_FFFF) + 2] }
    }
    #[inline(always)]
    fn rule4(&self, w: u32) -> usize { self.base + (w - 257) as usize * RULE4 }

    #[inline(always)]
    fn len_plain(&self, w: u32) -> u32 {
        if w <= 256 { u32::from(w != 0) } else { self.a[self.rule4(w) + 2] }
    }
    #[inline(always)]
    fn len_operad(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            let r = self.rule4(w);
            let r3 = self.a[r + 3];
            if (r3 >> 28) == 0 { r3 >> 24 } else { self.a[r + 2] }
        }
    }
    #[inline(always)]
    fn short_u128(&self, w: u32) -> u128 {
        if w <= 256 {
            u128::from(w - 1)
        } else {
            let r = self.rule4(w);
            u128::from(self.a[r])
                | (u128::from(self.a[r + 1]) << 32)
                | (u128::from(self.a[r + 2]) << 64)
                | (u128::from(self.a[r + 3] & 0x00FF_FFFF) << 96)
        }
    }

    /// `RULE_MODE`:
    ///   `0` = Master
    ///   `1` = Graded `#714`
    ///   `2` = Operad-MN16 (`RULE=4 u128`, tagged)
    ///   `3` = `G2: GradedOperad` (`#714 (len10<<22)|id22` + `Operad-MN16 u128 <= 15`)
    ///   `4` = `G3: Tagless Tiered GradedOperad` (`u64 <= 8` / `u128 <= 16`, tagless!)
    #[inline(always)]
    fn join_m<const RULE_MODE: u8>(&mut self, lh: u32, rh: u32) -> u32 {
        if lh == 0 { return rh; }
        if rh == 0 { return lh; }
        const ID22_MASK: u32 = (1 << 22) - 1;
        if RULE_MODE == 4 {
            let ll = lh >> 22;
            let rl = rh >> 22;
            let id = ((self.top - self.base) / RULE4) as u32 + 257;
            let tot = ll + rl;
            if tot <= 8 {
                let lid = lh & ID22_MASK;
                let rid = rh & ID22_MASK;
                let vl = self.short_u64(lid);
                let vr = self.short_u64(rid);
                let v = vl | (vr << (8 * ll));
                self.a[self.top] = v as u32;
                self.a[self.top + 1] = (v >> 32) as u32;
                self.top += RULE4;
                (tot << 22) | id
            } else if tot <= 16 {
                let lid = lh & ID22_MASK;
                let rid = rh & ID22_MASK;
                let vl = if ll <= 8 { u128::from(self.short_u64(lid)) } else { self.short_u128_raw(lid) };
                let vr = if rl <= 8 { u128::from(self.short_u64(rid)) } else { self.short_u128_raw(rid) };
                let v = vl | (vr << (8 * ll));
                self.a[self.top] = v as u32;
                self.a[self.top + 1] = (v >> 32) as u32;
                self.a[self.top + 2] = (v >> 64) as u32;
                self.a[self.top + 3] = (v >> 96) as u32;
                self.top += RULE4;
                (tot << 22) | id
            } else {
                let exact_len = if ll < 1023 && rl < 1023 {
                    tot
                } else {
                    self.len_graded_raw(lh) + self.len_graded_raw(rh)
                };
                self.a[self.top..self.top + RULE4].copy_from_slice(&[lh, rh, exact_len, u32::MAX]);
                self.top += RULE4;
                (exact_len.min(1023) << 22) | id
            }
        } else if RULE_MODE == 3 {
            let lid = lh & ID22_MASK;
            let rid = rh & ID22_MASK;
            let ll = lh >> 22;
            let rl = rh >> 22;
            let id = ((self.top - self.base) / RULE4) as u32 + 257;
            let tot = ll + rl;
            if tot <= 15 {
                let vl = self.short_u128(lid);
                let vr = self.short_u128(rid);
                let v = vl | (vr << (8 * ll)) | ((tot as u128) << 120);
                self.a[self.top] = v as u32;
                self.a[self.top + 1] = (v >> 32) as u32;
                self.a[self.top + 2] = (v >> 64) as u32;
                self.a[self.top + 3] = (v >> 96) as u32;
                self.top += RULE4;
                (tot << 22) | id
            } else {
                let exact_len = if ll < 1023 && rl < 1023 {
                    tot
                } else {
                    self.len_operad(lid) + self.len_operad(rid)
                };
                self.a[self.top..self.top + RULE4].copy_from_slice(&[lid, rid, exact_len, u32::MAX]);
                self.top += RULE4;
                (exact_len.min(1023) << 22) | id
            }
        } else if RULE_MODE == 1 {
            let lid = lh & ID22_MASK;
            let rid = rh & ID22_MASK;
            let ll = lh >> 22;
            let rl = rh >> 22;
            let exact_len = if ll < 1023 && rl < 1023 {
                ll + rl
            } else {
                self.len_plain(lid) + self.len_plain(rid)
            };
            let id = ((self.top - self.base) / RULE4) as u32 + 257;
            self.a[self.top..self.top + RULE4].copy_from_slice(&[lid, rid, exact_len, u32::MAX]);
            self.top += RULE4;
            (exact_len.min(1023) << 22) | id
        } else if RULE_MODE == 2 {
            let id = ((self.top - self.base) / RULE4) as u32 + 257;
            let ll = self.len_operad(lh);
            let rl = self.len_operad(rh);
            let len = ll + rl;
            if len <= 15 {
                let vl = self.short_u128(lh);
                let vr = self.short_u128(rh);
                let v = vl | (vr << (8 * ll)) | ((len as u128) << 120);
                self.a[self.top] = v as u32;
                self.a[self.top + 1] = (v >> 32) as u32;
                self.a[self.top + 2] = (v >> 64) as u32;
                self.a[self.top + 3] = (v >> 96) as u32;
            } else {
                self.a[self.top..self.top + RULE4].copy_from_slice(&[lh, rh, len, u32::MAX]);
            }
            self.top += RULE4;
            id
        } else {
            let id = ((self.top - self.base) / RULE4) as u32 + 257;
            let len = self.len_plain(lh) + self.len_plain(rh);
            self.a[self.top..self.top + RULE4].copy_from_slice(&[lh, rh, len, u32::MAX]);
            self.top += RULE4;
            id
        }
    }
    fn power_m<const RULE_MODE: u8>(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join_m::<RULE_MODE>(acc, base);
            }
            e >>= 1;
            if e == 0 { break acc; }
            base = self.join_m::<RULE_MODE>(base, base);
        }
    }
    fn eliminate<const SPIRALS: bool, const RULE_MODE: u8, const POP_TAIL: bool, const SET_LINKS: bool>(
        &mut self,
        remaining: &mut u32,
    ) -> bool {
        let mut left = *remaining;
        let room = if SPIRALS { SPIRAL_RULES * RULE4 } else { RULE4 };
        let done = loop {
            if left == 0 { break true; }
            if self.top + room > self.a.len() { break false; }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if SPIRALS && x != y {
                if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    let head = self.power_m::<RULE_MODE>(self.word(x), q);
                    let word = self.join_m::<RULE_MODE>(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    let tail = self.power_m::<RULE_MODE>(self.word(y), q);
                    let word = self.join_m::<RULE_MODE>(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            left -= w;
            if x == y {
                if POP_TAIL { self.pop_tail(x, 0); self.pop_tail(x, 1); }
                else { self.cut(x, 0); self.cut(x, 1); }
                continue;
            }
            let word = self.join_m::<RULE_MODE>(self.word(x), self.word(y));
            if aw <= bw {
                if POP_TAIL { self.pop_tail(x, 0); } else { self.cut(x, 0); }
                self.insert::<SET_LINKS>(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    if POP_TAIL { self.pop_tail(y, 1); } else { self.cut(y, 1); }
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                if POP_TAIL { self.pop_tail(y, 1); } else { self.cut(y, 1); }
                self.insert::<SET_LINKS>(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        *remaining = left;
        done
    }

    fn append_plain(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule4(id);
                let rule = &mut self.a[r..r + RULE4];
                if rule[3] == u32::MAX {
                    rule[3] = dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = rule[3] as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }

    fn append_operad(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule4(id);
                let r3 = self.a[r + 3];
                if (r3 >> 28) == 0 {
                    let tag = (r3 >> 24) as usize;
                    let v = u128::from(self.a[r])
                        | (u128::from(self.a[r + 1]) << 32)
                        | (u128::from(self.a[r + 2]) << 64)
                        | (u128::from(r3 & 0x00FF_FFFF) << 96);
                    let bytes = v.to_le_bytes();
                    dest.extend_from_slice(&bytes[..tag]);
                } else if r3 == u32::MAX {
                    self.a[r + 3] = 0x8000_0000 | (dest.len() as u32);
                    let right = self.a[r + 1];
                    let left = self.a[r];
                    stack.push(right);
                    stack.push(left);
                } else {
                    let start = (r3 & 0x0FFF_FFFF) as usize;
                    let len = self.a[r + 2] as usize;
                    dest.extend_from_within(start..start + len);
                }
            }
        }
    }

    fn append_graded_operad(&mut self, word_handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        const ID22_MASK: u32 = (1 << 22) - 1;
        stack.clear();
        stack.push(word_handle);
        while let Some(h) = stack.pop() {
            let hl = (h >> 22) as usize;
            let id = h & ID22_MASK;
            if hl <= 1 {
                dest.push((id - 1) as u8);
            } else if hl <= 8 {
                let v = self.short_u64(id);
                dest.extend_from_slice(&v.to_le_bytes()[..hl]);
            } else if hl <= 16 {
                let v = self.short_u128_raw(id);
                dest.extend_from_slice(&v.to_le_bytes()[..hl]);
            } else {
                let r = self.rule4(id);
                let r3 = self.a[r + 3];
                if r3 == u32::MAX {
                    self.a[r + 3] = dest.len() as u32;
                    let right = self.a[r + 1];
                    let left = self.a[r];
                    stack.push(right);
                    stack.push(left);
                } else {
                    let start = r3 as usize;
                    let len = self.a[r + 2] as usize;
                    dest.extend_from_within(start..start + len);
                }
            }
        }
    }

    fn compute_bottom_serial(&mut self) {
        let mut id = L::next(self.a, 0, 1);
        let mut x = 0u32;
        while id != 0 {
            let next = L::next(self.a, id, 1);
            let w = self.width(id);
            self.set_bottom(id, x);
            x += w;
            id = next;
        }
    }

    fn compute_bottom_bidi(&mut self, m: u32) {
        let mut f_id = L::next(self.a, 0, 1);
        let mut b_id = L::prev(self.a, 0, 1);
        let mut f_x = 0u32;
        let mut b_x = m;
        while f_id != 0 && f_id != b_id {
            let f_next = L::next(self.a, f_id, 1);
            let b_prev = L::prev(self.a, b_id, 1);
            let f_w = self.width(f_id);
            let b_w = self.width(b_id);
            b_x -= b_w;
            self.set_bottom(f_id, f_x);
            self.set_bottom(b_id, b_x);
            f_x += f_w;
            if f_next == b_id {
                f_id = 0;
                break;
            }
            f_id = f_next;
            b_id = b_prev;
        }
        if f_id != 0 && f_id == b_id {
            self.set_bottom(f_id, f_x);
        }
    }
}

// ============================================================================
// 4. Common Helpers, Hashers, & Emit Kernels
// ============================================================================

#[derive(Clone, Copy)]
struct First {
    all: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
}
impl Default for First {
    fn default() -> Self {
        Self { all: [0; 256], thin: [0; 256], thick: [0; 256], m: 0, mark: 0 }
    }
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut out = [0u32; 256];
    let mut sum = 0u32;
    for a in 0..256 {
        out[a] = sum;
        sum += counts[a];
    }
    out
}

fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
    let mut pos = 0u32;
    let mut i = 0usize;
    while i < records.len() {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < records.len() && (records[i] >> 8) as u8 == c {
            width += u32::from(records[i] & 255);
            i += 1;
        }
        visit(c, pos, width);
        pos += width;
    }
}

#[inline(always)]
fn split(c: u8, pos: u32, width: u32, p: u32, limit: u32, mut visit: impl FnMut(u8, u32, u32, bool)) {
    let end = pos + width;
    if width > limit && (pos..end).contains(&p) {
        if pos < p { visit(c, pos, p - pos, false); }
        visit(c, p, 1, true);
        if p + 1 < end { visit(c, p + 1, end - p - 1, false); }
    } else {
        visit(c, pos, width, width <= limit);
    }
}

#[inline(always)]
fn hash_u128(v: u128, wlen: u32) -> u64 {
    let lo = v as u64;
    let hi = (v >> 64) as u64;
    let h = lo.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ hi.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ (u64::from(wlen)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

#[inline(always)]
fn hash_long(w: &[u8]) -> u64 {
    let n = w.len();
    let lo = u64::from_le_bytes([w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7]]);
    let hi = u64::from_le_bytes([
        w[n - 8], w[n - 7], w[n - 6], w[n - 5], w[n - 4], w[n - 3], w[n - 2], w[n - 1],
    ]);
    let mid = u64::from_le_bytes([
        w[n / 2 - 4], w[n / 2 - 3], w[n / 2 - 2], w[n / 2 - 1],
        w[n / 2], w[n / 2 + 1], w[n / 2 + 2], w[n / 2 + 3],
    ]);
    let h = lo.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ hi.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ mid.wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

fn inline_words(spelling: &[u8], offsets: &[u32], inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
    if inline16.len() < 256 {
        inline16.clear();
        inline16.extend((0..=255u8).map(|c| {
            let mut v = [0; INLINE];
            v[0] = c;
            v
        }));
        lens.clear();
        lens.resize(256, 1);
    }
    inline16.truncate(256);
    lens.truncate(256);
    let words = offsets.len().saturating_sub(1);
    inline16.reserve_exact(words);
    lens.reserve_exact(words);
    for w in offsets.windows(2) {
        let word = &spelling[w[0] as usize..w[1] as usize];
        let mut v = [0; INLINE];
        let head = word.len().min(INLINE);
        v[..head].copy_from_slice(&word[..head]);
        inline16.push(v);
        lens.push(if word.len() <= INLINE { word.len() as u8 } else { LONG });
    }
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_offsets(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let mask = (1u32 << bits) - 1;
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
            let w = code - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n { break; }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_no_off(
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
    let mut at = 0usize;
    let mask = (1u32 << bits) - 1;
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
            let meta = &inline16[code];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n { break; }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[cold]
#[inline(never)]
fn emit_long_no_off(
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    code: usize,
    out_buf: &mut [u8],
    at: usize,
    n: usize,
) -> usize {
    let meta = &inline16[code];
    let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
    let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
    let word = &spelling[s..e];
    let take = word.len().min(n - at);
    out_buf[at..at + take].copy_from_slice(&word[..take]);
    at + take
}

#[allow(dead_code)]
#[cold]
#[inline(never)]
fn emit_long_offsets(
    spelling: &[u8],
    offsets: &[u32],
    code: usize,
    out_buf: &mut [u8],
    at: usize,
    n: usize,
) -> usize {
    let w = code - 256;
    let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
    let take = word.len().min(n - at);
    out_buf[at..at + take].copy_from_slice(&word[..take]);
    at + take
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_no_off_nospill(
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
    let out_buf = &mut out[..n + INLINE];
    let mut at = 0usize;
    let mask = (1u32 << bits) - 1;
    let mut j = mark;
    if extra.is_empty() {
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = (edge >> bits) as usize;
            j = edge & mask;
            let len = lens[code];
            if len != LONG {
                out_buf[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                at = emit_long_no_off(spelling, inline16, code, out_buf, at, n);
            }
            if j == mark || at >= n { break; }
        }
    } else {
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = extra[j as usize] as usize;
            j = edge & mask;
            let len = lens[code];
            if len != LONG {
                out_buf[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                at = emit_long_no_off(spelling, inline16, code, out_buf, at, n);
            }
            if j == mark || at >= n { break; }
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(dead_code)]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_offsets_nospill(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let out_buf = &mut out[..n + INLINE];
    let mut at = 0usize;
    let mask = (1u32 << bits) - 1;
    let mut j = mark;
    if extra.is_empty() {
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = (edge >> bits) as usize;
            j = edge & mask;
            let len = lens[code];
            if len != LONG {
                out_buf[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                at = emit_long_offsets(spelling, offsets, code, out_buf, at, n);
            }
            if j == mark || at >= n { break; }
        }
    } else {
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = extra[j as usize] as usize;
            j = edge & mask;
            let len = lens[code];
            if len != LONG {
                out_buf[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                at = emit_long_offsets(spelling, offsets, code, out_buf, at, n);
            }
            if j == mark || at >= n { break; }
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_zerospill(
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
    if !extra.is_empty() || edges.is_empty() {
        emit_no_off_nospill(edges, extra, spelling, inline16, lens, n, mark, bits, out);
        return;
    }
    out.resize(n + INLINE, 0);
    let out_buf = &mut out[..n + INLINE];
    let dict_len = lens.len().min(inline16.len());
    let lens = &lens[..dict_len];
    let inline16 = &inline16[..dict_len];
    let mask = (1u32 << bits) - 1;
    let m = edges.len();
    let mut j = mark;
    let mut at = 0usize;
    let mut steps = 0usize;
    while steps < m && at < n {
        let mut long_code = usize::MAX;
        while steps < m && at < n {
            let edge = edges[j as usize];
            let code = (edge >> bits) as usize;
            j = edge & mask;
            steps += 1;
            let len = lens[code];
            if len != LONG {
                out_buf[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
                if j == mark {
                    long_code = usize::MAX - 1;
                    break;
                }
            } else {
                long_code = code;
                break;
            }
        }
        if long_code == usize::MAX - 1 {
            break;
        }
        if long_code != usize::MAX {
            at = emit_long_no_off(spelling, inline16, long_code, out_buf, at, n);
            if j == mark {
                break;
            }
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_zerospill_lenp<const SHIFT: u32>(
    edges: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    code_mask: u32,
    out: &mut Vec<u8>,
) {
    if edges.is_empty() {
        out.truncate(0);
        return;
    }
    out.resize(n + INLINE, 0);
    let out_buf = &mut out[..n + INLINE];
    let dict_len = lens.len().min(inline16.len());
    let lens = &lens[..dict_len];
    let inline16 = &inline16[..dict_len];
    let mask = (1u32 << bits) - 1;
    let m = edges.len();
    let mut j = mark;
    let mut at = 0usize;
    let mut steps = 0usize;
    while steps < m && at < n {
        let mut slow_code = usize::MAX;
        while steps < m && at < n {
            let edge = edges[j as usize];
            let len = (edge >> SHIFT) as usize;
            let code = ((edge >> bits) & code_mask) as usize;
            j = edge & mask;
            steps += 1;
            if len != 0 {
                out_buf[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += len;
                if j == mark {
                    slow_code = usize::MAX - 1;
                    break;
                }
            } else {
                slow_code = code;
                break;
            }
        }
        if slow_code == usize::MAX - 1 {
            break;
        }
        if slow_code != usize::MAX {
            let l = lens[slow_code];
            if l != LONG {
                out_buf[at..at + INLINE].copy_from_slice(&inline16[slow_code]);
                at += usize::from(l);
            } else {
                at = emit_long_no_off(spelling, inline16, slow_code, out_buf, at, n);
            }
            if j == mark {
                break;
            }
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_xor_bi_no_off(
    edges: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    j_neg1: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let mut prev_f = j_neg1;
    let mut cur_f = mark;
    let mut next_b = mark;
    let mut cur_b = j_neg1;
    let mut at_f = 0usize;
    let mut at_b = n;

    let max_pairs = edges.len() / 2 + 1;
    let mut started = false;
    let mut rem_pairs = max_pairs;
    while rem_pairs > 0 && cur_f != cur_b && (!started || cur_f != next_b) && at_f + 32 <= at_b {
        started = true;
        rem_pairs -= 1;
        let ef = edges[cur_f as usize];
        let eb = edges[cur_b as usize];
        let cf = (ef >> bits) as usize;
        let cb = (eb >> bits) as usize;
        let nf = (ef & mask) ^ prev_f;
        let pb = (eb & mask) ^ next_b;
        prev_f = cur_f;
        cur_f = nf;
        next_b = cur_b;
        cur_b = pb;

        let lf = lens[cf];
        if lf != LONG {
            out[at_f..at_f + INLINE].copy_from_slice(&inline16[cf]);
            at_f += usize::from(lf);
        } else {
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }

        let lb = lens[cb];
        if lb != LONG {
            let lbu = usize::from(lb);
            if at_f + INLINE <= at_b {
                let raw = u128::from_le_bytes(inline16[cb]);
                let shifted = if lbu == 16 { raw } else { raw << ((16 - lbu) * 8) };
                out[at_b - INLINE..at_b].copy_from_slice(&shifted.to_le_bytes());
                at_b -= lbu;
            } else {
                let take = lbu.min(at_b - at_f);
                out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
                at_b -= take;
            }
        } else {
            let meta = &inline16[cb];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }
    }

    while rem_pairs > 0 && cur_f != cur_b && (!started || cur_f != next_b) && at_f < at_b {
        started = true;
        rem_pairs -= 1;
        let ef = edges[cur_f as usize];
        let eb = edges[cur_b as usize];
        let cf = (ef >> bits) as usize;
        let cb = (eb >> bits) as usize;
        let nf = (ef & mask) ^ prev_f;
        let pb = (eb & mask) ^ next_b;
        prev_f = cur_f;
        cur_f = nf;
        next_b = cur_b;
        cur_b = pb;

        let lf = lens[cf];
        if lf != LONG {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&inline16[cf][..take]);
            at_f += take;
        } else {
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }

        let lb = lens[cb];
        if lb != LONG {
            let lbu = usize::from(lb);
            let take = lbu.min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
            at_b -= take;
        } else {
            let meta = &inline16[cb];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }
    }

    if cur_f == cur_b && at_f < at_b {
        let cf = (edges[cur_f as usize] >> bits) as usize;
        let lf = lens[cf];
        if lf != LONG {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&inline16[cf][..take]);
            at_f += take;
        } else {
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }
    }

    if at_f < at_b {
        let right_len = n - at_b;
        if right_len > 0 {
            out.copy_within(at_b..n, at_f);
        }
        let period = at_f + right_len;
        out.truncate(period);
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

// ============================================================================
// 5. Parameterized Audit Kernel Covering All 7 Candidate Modes
// ============================================================================
// MODE:
//   0 = M0:Clone           (verbatim 0.5.11 master)
//   1 = W1:#714+L4+L5      (Wormhole Cycle 20: GradedHandle + pop_tail + Prefix15 + BidiBottom + [u32;3] StripBandMux)
//   2 = N1:OperadDedup     (Neon-C29 Operad-MN16 RULE=4 u128 + L1D word dedup + 0 B offsets + pop_tail; master Pass1/Strips/Bands/emit)
//   3 = N2:+MuxBand64+L4   (N1 + MuxStrips in edges[0..3*ns] + Band64 u64 + 48 KiB spelling + L4 BidiBottom)
//   4 = N3:+All+Pos (S*)   (N2 + Pass-1 all=thin+thick_sum + pos=(tail<<10)|fill; complete mu=1 construction stack!)
//   5 = W2:Graded+AllPos   (#714 GradedHandle + Lazy15 + L1D word dedup + 0 B offsets + MuxStrips + Band64 + L4 + All+Pos!)
//   6 = N4:Full Neon-C29   (N3 + Fused-XOR mu=2 bidirectional walk + wire_xor)

#[derive(Clone, Copy)]
struct Strip16 { from: u32, to: u32, width: u32, byte: u8 }
#[derive(Clone, Copy)]
struct Band12 { end: u32, delta: u32, code: u32 }
#[derive(Clone, Copy)]
struct Band64(u64);
impl Band64 {
    #[inline(always)]
    fn new(end: u32, dst: u32, code: u32) -> Self {
        Band64(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
    }
    #[inline(always)]
    fn end(self) -> u32 { (self.0 & 0xF_FFFF) as u32 }
    #[inline(always)]
    fn dst(self) -> u32 { ((self.0 >> 20) & 0xF_FFFF) as u32 }
    #[inline(always)]
    fn code(self) -> u32 { (self.0 >> 40) as u32 }
}

#[derive(Default, Clone, Copy)]
struct StageUs { s1: f64, s2: f64, s3: f64, s4: f64, s5: f64 }

struct AuditKernel<const MODE: u8> {
    stack: Vec<u32>,
    spill: Spill,
    strips16: Vec<Strip16>,
    bands12: Vec<Band12>,
    sb_buf: Vec<[u32; 3]>,
    bands64: Vec<Band64>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    last_us: StageUs,
    last_m: u32,
    last_mark: u32,
    last_bits: u32,
}

impl<const MODE: u8> Default for AuditKernel<MODE> {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            spill: Spill::default(),
            strips16: Vec::new(),
            bands12: Vec::new(),
            sb_buf: Vec::new(),
            bands64: Vec::new(),
            spelling: Vec::new(),
            offsets: Vec::new(),
            edges: Vec::new(),
            extra: Vec::new(),
            inline16: Vec::new(),
            lens: Vec::new(),
            last_us: StageUs::default(),
            last_m: 0,
            last_mark: 0,
            last_bits: 0,
        }
    }
}

struct PassStd<'a> {
    queues: QueuesMaster<'a>,
    strips: &'a mut Vec<Strip16>,
    f: First,
}
impl PassStd<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        if width <= limit { self.piece(c, pos, width, true, p); return; }
        if p.wrapping_sub(pos) >= width { self.piece(c, pos, width, false, p); }
        else { self.take_marked(c, pos, width, p, limit); }
    }
    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |sc, x, w, keep| self.piece(sc, x, w, keep, p));
    }
    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w { f.mark = f.m + (p - x); }
            self.queues.push(a, f.m, w, &f.thin);
            f.thin[a] += w;
            f.m += w;
        } else {
            self.strips.push(Strip16 { from: x, to: f.all[a], width: w, byte: c });
            f.thick[a] += 1;
        }
        f.all[a] += w;
    }
}

struct PassSb<'a> {
    queues: QueuesMaster<'a>,
    sb_buf: &'a mut Vec<[u32; 3]>,
    f: First,
}
impl PassSb<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        if width <= limit { self.piece(c, pos, width, true, p); return; }
        if p.wrapping_sub(pos) >= width { self.piece(c, pos, width, false, p); }
        else { self.take_marked(c, pos, width, p, limit); }
    }
    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |sc, x, w, keep| self.piece(sc, x, w, keep, p));
    }
    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w { f.mark = f.m + (p - x); }
            self.queues.push(a, f.m, w, &f.thin);
            f.thin[a] += w;
            f.m += w;
        } else {
            self.sb_buf.push([x, f.all[a], (w << 8) | u32::from(c)]);
            f.thick[a] += 1;
        }
        f.all[a] += w;
    }
}

struct PassMuxStd<'a> {
    queues: QueuesMaster<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    strip_over: bool,
    f: First,
}
impl PassMuxStd<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        if width <= limit { self.piece(c, pos, width, true, p); return; }
        if p.wrapping_sub(pos) >= width { self.piece(c, pos, width, false, p); }
        else { self.take_marked(c, pos, width, p, limit); }
    }
    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |sc, x, w, keep| self.piece(sc, x, w, keep, p));
    }
    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w { f.mark = f.m + (p - x); }
            self.queues.push(a, f.m, w, &f.thin);
            f.thin[a] += w;
            f.m += w;
        } else {
            if self.ns >= self.cap_strips {
                self.strip_over = true;
            } else {
                let idx = 3 * self.ns;
                if self.edges.len() < idx + 3 {
                    let grow_to = (idx + 3).max(2 * self.edges.len()).max(1024);
                    let capped = if self.cap_strips < (usize::MAX / 4) { grow_to.min(3 * self.cap_strips + 3) } else { grow_to };
                    self.edges.resize(capped.max(idx + 3), 0);
                }
                self.edges[idx] = x;
                self.edges[idx + 1] = f.all[a];
                self.edges[idx + 2] = (w << 8) | u32::from(c);
                self.ns += 1;
            }
            f.thick[a] += 1;
        }
        f.all[a] += w;
    }
}

/// Neon-C29 Pass-1: Algebraic `all = thin + thick_sum` (`0` `all[a]` RMWs on thin runs!)
/// + Bit-Lattice `QueuesPos` (`pos = (tail << 10) | fill`) + `MuxStrips` in `edges[0..3*ns]`!
struct PassMuxAllPos<'a> {
    queues: QueuesPos<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    strip_over: bool,
    f: First, // `f.all[a]` holds ONLY `thick_sum[a]` during Pass 1!
}
impl PassMuxAllPos<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        if width <= limit { self.piece(c, pos, width, true, p); return; }
        if p.wrapping_sub(pos) >= width { self.piece(c, pos, width, false, p); }
        else { self.take_marked(c, pos, width, p, limit); }
    }
    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |sc, x, w, keep| self.piece(sc, x, w, keep, p));
    }
    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w { f.mark = f.m + (p - x); }
            self.queues.push(a, f.m, w, &f.thin);
            f.thin[a] += w;
            f.m += w;
        } else {
            if self.ns >= self.cap_strips {
                self.strip_over = true;
            } else {
                let idx = 3 * self.ns;
                if self.edges.len() < idx + 3 {
                    let grow_to = (idx + 3).max(2 * self.edges.len()).max(1024);
                    let capped = if self.cap_strips < (usize::MAX / 4) { grow_to.min(3 * self.cap_strips + 3) } else { grow_to };
                    self.edges.resize(capped.max(idx + 3), 0);
                }
                self.edges[idx] = x;
                self.edges[idx + 1] = f.thin[a] + f.all[a];
                self.edges[idx + 2] = (w << 8) | u32::from(c);
                self.ns += 1;
            }
            f.thick[a] += 1;
            f.all[a] += w;
        }
    }
}

impl<const MODE: u8> AuditKernel<MODE> {
    const USES_MUX_STRIPS: bool = MODE >= 3;
    const USES_ALL_POS: bool = MODE >= 4;
    const USES_DEDUP: bool = MODE >= 2;
    const USES_OPERAD: bool = MODE == 2 || MODE == 3 || MODE == 4 || MODE == 6 || MODE == 7 || MODE == 8;
    const USES_GRADED_PLAIN: bool = MODE == 1 || MODE == 5;
    const USES_GRADED_OPERAD: bool = MODE == 9 || MODE >= 11;
    const USES_TIERED_OPERAD: bool = MODE == 10;
    const USES_GRADED: bool = Self::USES_GRADED_PLAIN || Self::USES_GRADED_OPERAD || Self::USES_TIERED_OPERAD;
    const USES_POP_TAIL: bool = MODE >= 1;
    const USES_SET_LINKS: bool = MODE == 8 || MODE == 10;
    const USES_BIDI_BOTTOM: bool = MODE == 1 || MODE >= 3;
    const USES_XOR_BI: bool = MODE == 6;
    const USES_NOSPILL_EMIT: bool = MODE >= 7 && MODE < 11;
    const USES_ZEROSPILL_EMIT: bool = MODE >= 11;
    const USES_LEN4_EDGE: bool = MODE == 12 || MODE == 13;
    const USES_LEN3_EDGE: bool = MODE == 13;
    const USES_ZIPF_REMAP: bool = MODE == 14;

    fn scratch_bytes(&self) -> usize {
        4 * self.stack.capacity()
            + 4 * self.spill.next.capacity()
            + BYTES * self.spill.ovf.capacity()
            + 1024 * self.spill.cuts.capacity()
            + 16 * self.strips16.capacity()
            + 12 * self.bands12.capacity()
            + 12 * self.sb_buf.capacity()
            + 8 * self.bands64.capacity()
            + self.spelling.capacity()
            + 4 * self.offsets.capacity()
            + 4 * self.edges.capacity()
            + 4 * self.extra.capacity()
            + INLINE * self.inline16.capacity()
            + self.lens.capacity()
    }

    fn reset_inline16(&mut self) {
        if self.inline16.len() < 256 || Self::USES_ZIPF_REMAP {
            self.inline16.clear();
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0u8; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.resize(256, 1);
        } else {
            self.inline16.truncate(256);
            self.lens.truncate(256);
        }
    }

    fn zipf_remap_codes(&mut self) {
        let u = self.inline16.len();
        if u <= 256 || self.bands64.len() < 256 {
            return;
        }
        // Compute per-code frequency (total row span across bands64)
        let mut freq = vec![0u32; u];
        let mut prev_end = 0u32;
        for &b in &self.bands64 {
            let end = b.end();
            let span = end - prev_end;
            let c = b.code() as usize;
            freq[c] += span;
            prev_end = end;
        }
        // Sort codes descending by frequency so hottest 256 codes pack into first 4 KiB page!
        let mut order: Vec<u32> = (0..u as u32).collect();
        order.sort_unstable_by(|&a, &b| freq[b as usize].cmp(&freq[a as usize]));
        let mut old_to_new = vec![0u32; u];
        let mut new_inl = Vec::with_capacity(u);
        let mut new_lens = Vec::with_capacity(u);
        for (new_c, &old_c) in order.iter().enumerate() {
            old_to_new[old_c as usize] = new_c as u32;
            new_inl.push(self.inline16[old_c as usize]);
            new_lens.push(self.lens[old_c as usize]);
        }
        self.inline16.clear();
        self.inline16.extend_from_slice(&new_inl);
        self.lens.clear();
        self.lens.extend_from_slice(&new_lens);
        for b in &mut self.bands64 {
            let nc = old_to_new[b.code() as usize];
            *b = Band64::new(b.end(), b.dst(), nc);
        }
    }

    #[inline(always)]
    fn unfold_graded_u128<L: Links>(e: &Elim<'_, L>, node_id: u32, h_len: usize, a_byte: u8) -> [u8; INLINE] {
        let mut v = [0u8; INLINE];
        if h_len == 1 {
            v[0] = (node_id - 1) as u8;
            v[1] = a_byte;
        } else if h_len == 2 {
            let r = e.rule4(node_id);
            v[0] = (e.a[r] - 1) as u8;
            v[1] = (e.a[r + 1] - 1) as u8;
            v[2] = a_byte;
        } else {
            let mut st = [0u32; 16];
            st[0] = node_id;
            let mut sp = 1usize;
            let mut vp = 0usize;
            while sp > 0 {
                sp -= 1;
                let cur = st[sp];
                if cur <= 256 {
                    v[vp] = (cur - 1) as u8;
                    vp += 1;
                } else {
                    let r = e.rule4(cur);
                    st[sp] = e.a[r + 1];
                    st[sp + 1] = e.a[r];
                    sp += 2;
                }
            }
            v[h_len] = a_byte;
        }
        v
    }

    fn induced<L: Links, const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        t_s2: Option<Instant>,
    ) -> [usize; 256] {
        let hdr_words = if Self::USES_MUX_STRIPS { 4 * ns } else { ns };
        let (hdr, arena) = self.edges.split_at_mut(hdr_words);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0u32;
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);

        if Self::USES_MUX_STRIPS {
            let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
            for i in 0..ns {
                let sw = strips_raw[3 * i + 2] >> 8;
                let sb = strips_raw[3 * i + 2] & 0xFF;
                let lit = if Self::USES_GRADED { (1u32 << 22) | (sb + 1) } else { sb + 1 };
                e.node(sw, lit);
                widest = widest.max(sw);
            }
            let mut x = 0u32;
            for i in 0..ns {
                let s_from = strips_raw[3 * i];
                let s_w = strips_raw[3 * i + 2] >> 8;
                if x < s_from {
                    widest = widest.max(s_from - x);
                    let id = e.node(s_from - x, 0);
                    e.append_to(&mut order[0], id, 0);
                    e.append_to(&mut terminals[1], id, 1);
                }
                e.append_to(&mut order[0], i as u32 + 1, 0);
                x = s_from + s_w;
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
                let s_to = strips_raw[3 * idx + 1];
                let s_w = strips_raw[3 * idx + 2] >> 8;
                if x < s_to {
                    widest = widest.max(s_to - x);
                    let id = e.node(s_to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = s_to + s_w;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
        } else if MODE == 1 {
            for s in self.sb_buf.iter() {
                let sw = s[2] >> 8;
                let sb = s[2] & 0xFF;
                e.node(sw, (1u32 << 22) | (sb + 1));
                widest = widest.max(sw);
            }
            let mut x = 0u32;
            for (i, s) in self.sb_buf.iter().enumerate() {
                let s_from = s[0];
                let s_w = s[2] >> 8;
                if x < s_from {
                    widest = widest.max(s_from - x);
                    let id = e.node(s_from - x, 0);
                    e.append_to(&mut order[0], id, 0);
                    e.append_to(&mut terminals[1], id, 1);
                }
                e.append_to(&mut order[0], i as u32 + 1, 0);
                x = s_from + s_w;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            x = 0;
            for &i in hdr.iter() {
                let s = self.sb_buf[i as usize];
                let s_to = s[1];
                let s_w = s[2] >> 8;
                if x < s_to {
                    widest = widest.max(s_to - x);
                    let id = e.node(s_to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = s_to + s_w;
            }
            self.sb_buf.clear();
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
        } else {
            for s in self.strips16.iter() {
                e.node(s.width, u32::from(s.byte) + 1);
                widest = widest.max(s.width);
            }
            let mut x = 0u32;
            for (i, s) in self.strips16.iter().enumerate() {
                if x < s.from {
                    widest = widest.max(s.from - x);
                    let id = e.node(s.from - x, 0);
                    e.append_to(&mut order[0], id, 0);
                    e.append_to(&mut terminals[1], id, 1);
                }
                e.append_to(&mut order[0], i as u32 + 1, 0);
                x = s.from + s.width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            x = 0;
            for &i in hdr.iter() {
                let s = self.strips16[i as usize];
                if x < s.to {
                    widest = widest.max(s.to - x);
                    let id = e.node(s.to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = s.to + s.width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
        }

        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        let mut remaining = n;
        let spirals = widest > CERTIFIED_WIDTH;
        let room = if spirals { SPIRAL_RULES * RULE4 } else { RULE4 };
        while !match (spirals, Self::USES_TIERED_OPERAD, Self::USES_GRADED_OPERAD, Self::USES_GRADED_PLAIN, Self::USES_OPERAD, Self::USES_SET_LINKS, Self::USES_POP_TAIL) {
            (false, true,  _,     _,     _,     true,  true)  => e.eliminate::<false, 4, true,  true>(&mut remaining),
            (true,  true,  _,     _,     _,     true,  true)  => e.eliminate::<true,  4, true,  true>(&mut remaining),
            (false, false, true,  _,     _,     false, true)  => e.eliminate::<false, 3, true,  false>(&mut remaining),
            (true,  false, true,  _,     _,     false, true)  => e.eliminate::<true,  3, true,  false>(&mut remaining),
            (false, false, false, true,  _,     _,     true)  => e.eliminate::<false, 1, true,  false>(&mut remaining),
            (true,  false, false, true,  _,     _,     true)  => e.eliminate::<true,  1, true,  false>(&mut remaining),
            (false, false, false, false, true,  true,  true)  => e.eliminate::<false, 2, true,  true>(&mut remaining),
            (true,  false, false, false, true,  true,  true)  => e.eliminate::<true,  2, true,  true>(&mut remaining),
            (false, false, false, false, true,  false, true)  => e.eliminate::<false, 2, true,  false>(&mut remaining),
            (true,  false, false, false, true,  false, true)  => e.eliminate::<true,  2, true,  false>(&mut remaining),
            (false, false, false, false, false, false, false) => e.eliminate::<false, 0, false, false>(&mut remaining),
            (true,  false, false, false, false, false, false) => e.eliminate::<true,  0, false, false>(&mut remaining),
            _ => unreachable!(),
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(&mut self.edges, hdr_words + top + room);
            e = Elim {
                a: &mut self.edges[hdr_words..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        if let Some(t0) = t_s2 {
            self.last_us.s2 = t0.elapsed().as_nanos() as f64 / 1e3;
        }

        // === STAGE S3 ===
        let t_s3 = if TIMED { Some(Instant::now()) } else { None };
        if Self::USES_BIDI_BOTTOM {
            e.compute_bottom_bidi(m);
        } else {
            e.compute_bottom_serial();
        }

        let mut cursors = [0usize; 256];
        if !Self::USES_DEDUP {
            // MODE 0 (`M0`) or MODE 1 (`W1: #714+L4+L5`)
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0u32;
            let mut k = 0u32;
            for a in 0..256usize {
                cursors[a] = if MODE == 1 { self.sb_buf.len() } else { self.bands12.len() };
                let end = k + thin[a];
                while k < end {
                    let node_width = e.width(id);
                    let raw_word = e.word(id);
                    let width = (end - k).min(node_width - in_node);
                    let dst = e.bottom(id) + in_node;
                    let code = if raw_word == 0 {
                        a as u32
                    } else if MODE == 0 {
                        e.append_plain(raw_word, &mut self.stack, &mut self.spelling);
                        self.spelling.push(a as u8);
                        self.offsets.push(self.spelling.len() as u32);
                        254 + self.offsets.len() as u32
                    } else {
                        let h_len = (raw_word >> 22) as usize;
                        let node_id = raw_word & 0x003F_FFFF;
                        let c = self.inline16.len() as u32;
                        if h_len <= 15 {
                            let v = Self::unfold_graded_u128(&e, node_id, h_len, a as u8);
                            self.inline16.push(v);
                            self.lens.push((h_len + 1) as u8);
                            self.offsets.push(self.spelling.len() as u32);
                        } else {
                            let start_sp = self.spelling.len();
                            e.append_plain(node_id, &mut self.stack, &mut self.spelling);
                            self.spelling.push(a as u8);
                            self.offsets.push(self.spelling.len() as u32);
                            let mut v = [0u8; INLINE];
                            v.copy_from_slice(&self.spelling[start_sp..start_sp + INLINE]);
                            self.inline16.push(v);
                            self.lens.push(LONG);
                        }
                        c
                    };
                    if MODE == 1 {
                        self.sb_buf.push([k + width, dst.wrapping_sub(k), code]);
                    } else {
                        self.bands12.push(Band12 { end: k + width, delta: dst.wrapping_sub(k), code });
                    }
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = L::next(e.a, id, 0);
                        in_node = 0;
                    }
                }
            }
            if let Some(t0) = t_s3 {
                self.last_us.s3 = t0.elapsed().as_nanos() as f64 / 1e3;
            }
            return cursors;
        }

        // === DEDUP PATH (`MODE 2, 3, 4, 5, 6`) using dead `edges` region (`0 B` extra heap!) ===
        let (ebase, etop, enodes) = (e.base, e.top, e.nodes);
        let can_dedup = hdr_words >= HT_WORDS || e.a.len() >= etop + HT_WORDS;
        if !can_dedup {
            // Tiny-block fallback (`no dedup table needed`, exact!)
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0u32;
            let mut k = 0u32;
            for a in 0..256usize {
                cursors[a] = if Self::USES_MUX_STRIPS { self.bands64.len() } else { self.bands12.len() };
                let end = k + thin[a];
                while k < end {
                    let node_width = e.width(id);
                    let raw_word = e.word(id);
                    let width = (end - k).min(node_width - in_node);
                    let dst = e.bottom(id) + in_node;
                    let code = if raw_word == 0 {
                        a as u32
                    } else {
                        let c = self.inline16.len() as u32;
                        if Self::USES_TIERED_OPERAD {
                            let h_len = raw_word >> 22;
                            let node_id = raw_word & 0x003F_FFFF;
                            if h_len <= 15 {
                                let vu = if h_len <= 7 {
                                    u128::from(e.short_u64(node_id) | (u64::from(a as u8) << (8 * h_len)))
                                } else if h_len == 8 {
                                    u128::from(e.short_u64(node_id)) | (u128::from(a as u8) << 64)
                                } else {
                                    e.short_u128_raw(node_id) | (u128::from(a as u8) << (8 * h_len))
                                };
                                self.inline16.push(vu.to_le_bytes());
                                self.lens.push((h_len + 1) as u8);
                            } else {
                                let start = self.spelling.len();
                                e.append_graded_operad(raw_word, &mut self.stack, &mut self.spelling);
                                self.spelling.push(a as u8);
                                let end_s = self.spelling.len();
                                let mut meta = [0u8; INLINE];
                                meta[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                                meta[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                                self.inline16.push(meta);
                                self.lens.push(LONG);
                            }
                        } else if Self::USES_GRADED_OPERAD {
                            let h_len = raw_word >> 22;
                            let node_id = raw_word & 0x003F_FFFF;
                            if h_len <= 15 {
                                let vu = e.short_u128(node_id) | (u128::from(a as u8) << (8 * h_len));
                                self.inline16.push(vu.to_le_bytes());
                                self.lens.push((h_len + 1) as u8);
                            } else {
                                let start = self.spelling.len();
                                e.append_operad(node_id, &mut self.stack, &mut self.spelling);
                                self.spelling.push(a as u8);
                                let end_s = self.spelling.len();
                                let mut meta = [0u8; INLINE];
                                meta[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                                meta[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                                self.inline16.push(meta);
                                self.lens.push(LONG);
                            }
                        } else if Self::USES_GRADED_PLAIN {
                            let h_len = (raw_word >> 22) as usize;
                            let node_id = raw_word & 0x003F_FFFF;
                            if h_len <= 15 {
                                let v = Self::unfold_graded_u128(&e, node_id, h_len, a as u8);
                                self.inline16.push(v);
                                self.lens.push((h_len + 1) as u8);
                            } else {
                                let start = self.spelling.len();
                                e.append_plain(node_id, &mut self.stack, &mut self.spelling);
                                self.spelling.push(a as u8);
                                let end_s = self.spelling.len();
                                let mut meta = [0u8; INLINE];
                                meta[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                                meta[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                                self.inline16.push(meta);
                                self.lens.push(LONG);
                            }
                        } else {
                            let h_len = e.len_operad(raw_word);
                            if h_len <= 15 {
                                let vp = e.short_u128(raw_word) | (u128::from(a as u8) << (8 * h_len));
                                self.inline16.push(vp.to_le_bytes());
                                self.lens.push((h_len + 1) as u8);
                            } else {
                                let start = self.spelling.len();
                                e.append_operad(raw_word, &mut self.stack, &mut self.spelling);
                                self.spelling.push(a as u8);
                                let end_s = self.spelling.len();
                                let mut meta = [0u8; INLINE];
                                meta[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                                meta[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                                self.inline16.push(meta);
                                self.lens.push(LONG);
                            }
                        }
                        c
                    };
                    if Self::USES_MUX_STRIPS {
                        self.bands64.push(Band64::new(k + width, dst, code));
                    } else {
                        self.bands12.push(Band12 { end: k + width, delta: dst.wrapping_sub(k), code });
                    }
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = L::next(e.a, id, 0);
                        in_node = 0;
                    }
                }
            }
            if let Some(t0) = t_s3 {
                self.last_us.s3 = t0.elapsed().as_nanos() as f64 / 1e3;
            }
            return cursors;
        }

        let (e_slice, ht_buf) = if hdr_words >= HT_WORDS {
            let (dead_hdr, rest) = self.edges.split_at_mut(hdr_words);
            (&mut rest[..etop], &mut dead_hdr[..HT_WORDS])
        } else {
            let (prefix, tail) = self.edges.split_at_mut(hdr_words + etop);
            (&mut prefix[hdr_words..], &mut tail[..HT_WORDS])
        };
        let mut e2 = Elim::<L> {
            a: e_slice,
            base: ebase,
            top: etop,
            nodes: enodes,
            _links: std::marker::PhantomData,
        };
        let (pair_ht, str_ht) = ht_buf.split_at_mut(PAIR_LEN);
        pair_ht.fill(0);
        str_ht.fill(0);

        let mut id = L::next(e2.a, 0, 0);
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256usize {
            cursors[a] = if Self::USES_MUX_STRIPS { self.bands64.len() } else { self.bands12.len() };
            let end = k + thin[a];
            while k < end {
                let node_width = e2.width(id);
                let raw_word = e2.word(id);
                let width = (end - k).min(node_width - in_node);
                let dst = e2.bottom(id) + in_node;
                let code = if raw_word == 0 {
                    a as u32
                } else {
                    let node_id = if Self::USES_GRADED { raw_word & 0x003F_FFFF } else { raw_word };
                    let pkey = (node_id << 8) | (a as u32);
                    let pslot = ((pkey.wrapping_mul(0x9E37_79B9)) >> (32 - PAIR_BITS)) as usize * 2;
                    if pair_ht[pslot] == pkey {
                        pair_ht[pslot + 1]
                    } else {
                        let h_len = if Self::USES_GRADED {
                            raw_word >> 22
                        } else {
                            e2.len_operad(node_id)
                        };
                        let found_code = if h_len <= 15 {
                            let wlen = h_len + 1;
                            let wlen_u8 = wlen as u8;
                            let (v_bytes, v_u128) = if Self::USES_TIERED_OPERAD {
                                let vu = if h_len <= 7 {
                                    u128::from(e2.short_u64(node_id) | (u64::from(a as u8) << (8 * h_len)))
                                } else if h_len == 8 {
                                    u128::from(e2.short_u64(node_id)) | (u128::from(a as u8) << 64)
                                } else {
                                    e2.short_u128_raw(node_id) | (u128::from(a as u8) << (8 * h_len))
                                };
                                (vu.to_le_bytes(), vu)
                            } else if Self::USES_GRADED_OPERAD {
                                let vu = e2.short_u128(node_id) | (u128::from(a as u8) << (8 * h_len));
                                (vu.to_le_bytes(), vu)
                            } else if Self::USES_GRADED_PLAIN {
                                let vb = Self::unfold_graded_u128(&e2, node_id, h_len as usize, a as u8);
                                (vb, u128::from_le_bytes(vb))
                            } else {
                                let vu = e2.short_u128(node_id) | (u128::from(a as u8) << (8 * h_len));
                                (vu.to_le_bytes(), vu)
                            };
                            let h = hash_u128(v_u128, wlen);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let slot = (h as usize) & STR_MASK;
                            let ent = str_ht[slot];
                            if (ent & 0xFF00_0000) == tag && {
                                let c = (ent & 0x00FF_FFFF) as usize;
                                self.lens[c] == wlen_u8 && self.inline16[c] == v_bytes
                            } {
                                ent & 0x00FF_FFFF
                            } else {
                                let c = self.inline16.len() as u32;
                                self.inline16.push(v_bytes);
                                self.lens.push(wlen_u8);
                                str_ht[slot] = tag | c;
                                c
                            }
                        } else {
                            let already_memo = if Self::USES_TIERED_OPERAD {
                                h_len == 16 || e2.a[e2.rule4(node_id) + 3] != u32::MAX
                            } else {
                                e2.a[e2.rule4(node_id) + 3] != u32::MAX
                            };
                            let start = self.spelling.len();
                            if Self::USES_TIERED_OPERAD {
                                e2.append_graded_operad(raw_word, &mut self.stack, &mut self.spelling);
                            } else if Self::USES_GRADED_PLAIN {
                                e2.append_plain(node_id, &mut self.stack, &mut self.spelling);
                            } else {
                                e2.append_operad(node_id, &mut self.stack, &mut self.spelling);
                            }
                            self.spelling.push(a as u8);
                            let end_s = self.spelling.len();
                            let wlen_long = end_s - start;
                            let h = hash_long(&self.spelling[start..end_s]);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let slot = (h as usize) & STR_MASK;
                            let ent = str_ht[slot];
                            if (ent & 0xFF00_0000) == tag && {
                                let c = (ent & 0x00FF_FFFF) as usize;
                                if self.lens[c] == LONG {
                                    let meta = &self.inline16[c];
                                    let ps = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                                    let pe = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                                    (pe - ps) == wlen_long && self.spelling[ps..pe] == self.spelling[start..end_s]
                                } else {
                                    false
                                }
                            } {
                                if already_memo {
                                    self.spelling.truncate(start);
                                } else {
                                    self.spelling.pop();
                                }
                                ent & 0x00FF_FFFF
                            } else {
                                let c = self.inline16.len() as u32;
                                let mut meta = [0u8; INLINE];
                                meta[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                                meta[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                                self.inline16.push(meta);
                                self.lens.push(LONG);
                                str_ht[slot] = tag | c;
                                c
                            }
                        };
                        pair_ht[pslot] = pkey;
                        pair_ht[pslot + 1] = found_code;
                        found_code
                    }
                };
                if Self::USES_MUX_STRIPS {
                    self.bands64.push(Band64::new(k + width, dst, code));
                } else {
                    self.bands12.push(Band12 { end: k + width, delta: dst.wrapping_sub(k), code });
                }
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e2.a, id, 0);
                    in_node = 0;
                }
            }
        }
        if Self::USES_ZIPF_REMAP {
            self.zipf_remap_codes();
        }
        if let Some(t0) = t_s3 {
            self.last_us.s3 = t0.elapsed().as_nanos() as f64 / 1e3;
        }
        cursors
    }

    fn try_run<const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> bool {
        let t_s1 = if TIMED { Some(Instant::now()) } else { None };
        out.resize(n + INLINE, 0);

        let (mut f, heads, over, ns) = if Self::USES_ALL_POS {
            let mut pass = PassMuxAllPos {
                queues: QueuesPos::new(&mut out[..], &mut self.spill, cap.m),
                edges: &mut self.edges,
                ns: 0,
                cap_strips: cap.strips,
                strip_over: false,
                f: First::default(),
            };
            maximal(records, |c, pos, width| pass.take(c, pos, width, p, LIMIT));
            (pass.f, pass.queues.heads(), pass.queues.over || pass.strip_over, pass.ns)
        } else if Self::USES_MUX_STRIPS {
            let mut pass = PassMuxStd {
                queues: QueuesMaster::new(&mut out[..], &mut self.spill, cap.m),
                edges: &mut self.edges,
                ns: 0,
                cap_strips: cap.strips,
                strip_over: false,
                f: First::default(),
            };
            maximal(records, |c, pos, width| pass.take(c, pos, width, p, LIMIT));
            (pass.f, pass.queues.heads(), pass.queues.over || pass.strip_over, pass.ns)
        } else if MODE == 1 {
            self.sb_buf.clear();
            let mut pass = PassSb {
                queues: QueuesMaster::new(&mut out[..], &mut self.spill, cap.m),
                sb_buf: &mut self.sb_buf,
                f: First::default(),
            };
            maximal(records, |c, pos, width| pass.take(c, pos, width, p, LIMIT));
            let ns = pass.sb_buf.len();
            (pass.f, pass.queues.heads(), pass.queues.over, ns)
        } else {
            self.strips16.clear();
            let mut pass = PassStd {
                queues: QueuesMaster::new(&mut out[..], &mut self.spill, cap.m),
                strips: &mut self.strips16,
                f: First::default(),
            };
            maximal(records, |c, pos, width| pass.take(c, pos, width, p, LIMIT));
            let ns = pass.strips.len();
            (pass.f, pass.queues.heads(), pass.queues.over, ns)
        };

        if over || f.m > cap.m || ns > cap.strips {
            if MODE == 1 {
                self.sb_buf.clear();
                self.sb_buf.shrink_to(cap.strips);
            } else if !Self::USES_MUX_STRIPS {
                self.strips16.clear();
                self.strips16.shrink_to(cap.strips);
            }
            return false;
        }
        if let Some(t0) = t_s1 {
            self.last_us.s1 = t0.elapsed().as_nanos() as f64 / 1e3;
        }

        let t_s2 = if TIMED { Some(Instant::now()) } else { None };
        if Self::USES_ALL_POS {
            for a in 0..256 {
                f.all[a] += f.thin[a];
            }
        }
        let First { all, thin, thick, m, mark } = f;
        let first = starts(all);
        let mut write = starts(thick);
        let nodes_most = 3 * ns + 3;
        let stride = if nodes_most <= NARROW_NODES { Narrow::STRIDE } else { Wide::STRIDE };
        let hdr_words = if Self::USES_MUX_STRIPS { 4 * ns } else { ns };
        let need = (hdr_words + (stride + RULE4) * nodes_most).max(m as usize);
        if self.edges.len() < need {
            if Self::USES_MUX_STRIPS {
                let add = need - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(need, 0);
            } else {
                self.edges.clear();
                self.edges.reserve_exact(need);
                self.edges.resize(need, 0);
            }
        }

        if Self::USES_MUX_STRIPS {
            let (strips_raw, rest) = self.edges.split_at_mut(3 * ns);
            let image_order = &mut rest[..ns];
            for i in 0..ns {
                let a = (strips_raw[3 * i + 2] & 0xFF) as usize;
                strips_raw[3 * i + 1] += first[a];
                image_order[write[a] as usize] = i as u32;
                write[a] += 1;
            }
        } else if MODE == 1 {
            for (i, s) in self.sb_buf.iter_mut().enumerate() {
                let a = (s[2] & 0xFF) as usize;
                s[1] += first[a];
                self.edges[write[a] as usize] = i as u32;
                write[a] += 1;
            }
        } else {
            for (i, s) in self.strips16.iter_mut().enumerate() {
                let a = usize::from(s.byte);
                s.to += first[a];
                self.edges[write[a] as usize] = i as u32;
                write[a] += 1;
            }
        }

        self.bands12.clear();
        self.bands64.clear();
        self.spelling.clear();
        if Self::USES_MUX_STRIPS && n >= 65_536 && self.spelling.capacity() < 49_152 {
            self.spelling.reserve_exact(49_152);
        }
        if MODE <= 1 {
            self.offsets.clear();
            self.offsets.push(0);
        }
        if MODE >= 1 {
            self.reset_inline16();
        }

        let cursors = if nodes_most <= NARROW_NODES {
            self.induced::<Narrow, TIMED>(ns, n as u32, m, &thin, t_s2)
        } else {
            self.induced::<Wide, TIMED>(ns, n as u32, m, &thin, t_s2)
        };

        // === STAGE S4 ===
        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
        let words = if MODE == 0 { self.offsets.len() - 1 } else { self.inline16.len() - 256 };
        let dict_len = self.inline16.len() as u32;
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + words as u32) > (u32::MAX >> bits);
        let code_bits = if dict_len <= 1 { 1 } else { u32::BITS - (dict_len - 1).leading_zeros() };
        let edge_len_mode: u8 = if !wide && Self::USES_LEN4_EDGE && bits + code_bits <= 28 {
            4
        } else if !wide && Self::USES_LEN3_EDGE && bits + code_bits == 29 {
            3
        } else {
            0
        };
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        let xor_mode = Self::USES_XOR_BI && m >= 65_536 && !wide;
        let mut j_neg1 = mark;
        let out_ro = &out[..];
        let mut k = 0u32;

        if xor_mode {
            let edges_m = &mut self.edges[..m as usize];
            edges_m.fill(0);
            for a in 0..256usize {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands64.len());
                let mut queue = Reader::new(out_ro, &self.spill, &heads, a);
                for &b in &self.bands64[cursors[a]..end] {
                    let bend = b.end();
                    let span = (bend - k) as usize;
                    let dst = b.dst() as usize;
                    let pack = b.code() << bits;
                    queue.wire_xor(edges_m, dst, span, pack, mark, &mut j_neg1);
                    k = bend;
                }
            }
        } else if Self::USES_MUX_STRIPS {
            for a in 0..256usize {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands64.len());
                let mut queue = Reader::new(out_ro, &self.spill, &heads, a);
                for &b in &self.bands64[cursors[a]..end] {
                    let bend = b.end();
                    let span = (bend - k) as usize;
                    let dst = b.dst() as usize;
                    let code = b.code();
                    let pack = if wide {
                        0
                    } else if edge_len_mode == 4 {
                        let l = self.lens[code as usize];
                        let lp = if l <= 15 { u32::from(l) } else { 0u32 };
                        (lp << 28) | (code << bits)
                    } else if edge_len_mode == 3 {
                        let l = self.lens[code as usize];
                        let lp = if l <= 7 { u32::from(l) } else { 0u32 };
                        (lp << 29) | (code << bits)
                    } else {
                        code << bits
                    };
                    if wide { self.extra[dst..dst + span].fill(code); }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                    k = bend;
                }
            }
        } else if MODE == 1 {
            for a in 0..256usize {
                let end = cursors.get(a + 1).copied().unwrap_or(self.sb_buf.len());
                let mut queue = Reader::new(out_ro, &self.spill, &heads, a);
                for b in &self.sb_buf[cursors[a]..end] {
                    let span = (b[0] - k) as usize;
                    let dst = k.wrapping_add(b[1]) as usize;
                    let pack = if wide { 0 } else { b[2] << bits };
                    if wide { self.extra[dst..dst + span].fill(b[2]); }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                    k = b[0];
                }
            }
        } else {
            for a in 0..256usize {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands12.len());
                let mut queue = Reader::new(out_ro, &self.spill, &heads, a);
                for b in &self.bands12[cursors[a]..end] {
                    let span = (b.end - k) as usize;
                    let dst = k.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide { self.extra[dst..dst + span].fill(b.code); }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                    k = b.end;
                }
            }
        }
        if let Some(t0) = t_s4 {
            self.last_us.s4 = t0.elapsed().as_nanos() as f64 / 1e3;
        }

        if MODE == 0 {
            let t_s3b = if TIMED { Some(Instant::now()) } else { None };
            inline_words(&self.spelling, &self.offsets, &mut self.inline16, &mut self.lens);
            if let Some(t0) = t_s3b {
                self.last_us.s3 += t0.elapsed().as_nanos() as f64 / 1e3;
            }
        }

        self.last_m = m;
        self.last_mark = mark;
        self.last_bits = bits;

        // === STAGE S5 ===
        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
        if xor_mode {
            emit_xor_bi_no_off(
                &self.edges[..m as usize],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                j_neg1,
                bits,
                out,
            );
        } else if Self::USES_ZEROSPILL_EMIT {
            if edge_len_mode == 4 {
                emit_zerospill_lenp::<28>(
                    &self.edges[..m as usize],
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    mark,
                    bits,
                    (1u32 << code_bits) - 1,
                    out,
                );
            } else if edge_len_mode == 3 {
                emit_zerospill_lenp::<29>(
                    &self.edges[..m as usize],
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    mark,
                    bits,
                    (1u32 << code_bits) - 1,
                    out,
                );
            } else {
                emit_zerospill(
                    &self.edges[..m as usize],
                    &self.extra,
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    mark,
                    bits,
                    out,
                );
            }
        } else if Self::USES_NOSPILL_EMIT {
            emit_no_off_nospill(
                &self.edges[..m as usize],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
            );
        } else if Self::USES_DEDUP {
            emit_no_off(
                &self.edges[..m as usize],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
            );
        } else {
            emit_offsets(
                &self.edges[..m as usize],
                &self.extra,
                &self.spelling,
                &self.offsets,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
            );
        }
        if let Some(t0) = t_s5 {
            self.last_us.s5 = t0.elapsed().as_nanos() as f64 / 1e3;
        }
        true
    }
}

struct AuditCells<const MODE: u8, const FORCED: bool = false> {
    open: AuditKernel<MODE>,
    lazy: LazyCells,
}
impl<const MODE: u8, const FORCED: bool> Default for AuditCells<MODE, FORCED> {
    fn default() -> Self {
        Self { open: AuditKernel::default(), lazy: LazyCells::default() }
    }
}
impl<const MODE: u8, const FORCED: bool> InverseBwt for AuditCells<MODE, FORCED> {
    fn reads_runs(&self) -> bool { true }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if let Some(cap) = Cap::classify(r.records().len(), n, p, FORCED) {
            if self.open.try_run::<false>(r.records(), n, p, cap, out) {
                return true;
            }
        }
        self.lazy.invert_runs(r, n, p, out, k)
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(l, c, p, out);
    }
}




fn make_dense_703() -> Vec<u8> {
    let target_n = 900_000usize;
    let mut l = Vec::with_capacity(target_n);
    let mut rng = 0x9E37_79B9_7F4A_7C15u64;
    let mut step = || -> u64 {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    while l.len() + 256 < target_n {
        let sym_thick = (step() % 64) as u8 + 32;
        let w_thick = 320 + (step() % 180) as usize;
        l.extend(std::iter::repeat_n(sym_thick, w_thick.min(target_n - l.len())));
        for _ in 0..108 {
            if l.len() >= target_n { break; }
            let mut s = (step() % 90) as u8 + 32;
            if l.last() == Some(&s) { s = if s == 121 { 32 } else { s + 1 }; }
            let w_thin = if step() % 4 == 0 { 2 } else { 1 };
            l.extend(std::iter::repeat_n(s, w_thin.min(target_n - l.len())));
        }
    }
    while l.len() < target_n { l.push(b'x'); }
    l
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn verify_exactness(kernels: Kernels) -> usize {
    let mut ref_k = ForcedOpenCells::default();
    let mut k11 = AuditCells::<11, true>::default();
    let mut k12 = AuditCells::<12, true>::default();
    let mut k13 = AuditCells::<13, true>::default();
    let mut k14 = AuditCells::<14, true>::default();
    let mut oref = Vec::new();
    let mut oc = Vec::new();
    let mut cases = 0usize;

    for n in 1..=8usize {
        for mask in 0..(1u32 << n) {
            let l: Vec<u8> = (0..n).map(|i| ((mask >> i) & 1) as u8).collect();
            for p in 0..n as u32 {
                let runs = build_runs(&l, p, kernels);
                assert!(ref_k.invert_runs(&runs, n, p, &mut oref, kernels));
                assert!(k11.invert_runs(&runs, n, p, &mut oc, kernels)); assert_eq!(oc, oref);
                assert!(k12.invert_runs(&runs, n, p, &mut oc, kernels)); assert_eq!(oc, oref);
                assert!(k13.invert_runs(&runs, n, p, &mut oc, kernels)); assert_eq!(oc, oref);
                assert!(k14.invert_runs(&runs, n, p, &mut oc, kernels)); assert_eq!(oc, oref);
                cases += 1;
            }
        }
    }
    for n in 1..=5usize {
        let total = 3u32.pow(n as u32);
        for code in 0..total {
            let mut tmp = code;
            let mut l = Vec::with_capacity(n);
            for _ in 0..n { l.push((tmp % 3) as u8); tmp /= 3; }
            for p in 0..n as u32 {
                let runs = build_runs(&l, p, kernels);
                assert!(ref_k.invert_runs(&runs, n, p, &mut oref, kernels));
                assert!(k11.invert_runs(&runs, n, p, &mut oc, kernels)); assert_eq!(oc, oref);
                assert!(k12.invert_runs(&runs, n, p, &mut oc, kernels)); assert_eq!(oc, oref);
                assert!(k13.invert_runs(&runs, n, p, &mut oc, kernels)); assert_eq!(oc, oref);
                assert!(k14.invert_runs(&runs, n, p, &mut oc, kernels)); assert_eq!(oc, oref);
                cases += 1;
            }
        }
    }
    // Deep spiral (`widest = 65,000`) + boundary lengths 7..32 + multi-epoch
    let mut dsp = Vec::with_capacity(70_000);
    dsp.extend(std::iter::repeat_n(b'X', 65_000));
    dsp.extend(std::iter::repeat_n(b'Y', 20));
    dsp.push(b'Q');
    let rdsp = build_runs(&dsp, 65_020, kernels);
    assert!(ref_k.invert_runs(&rdsp, dsp.len(), 65_020, &mut oref, kernels));
    assert!(k11.invert_runs(&rdsp, dsp.len(), 65_020, &mut oc, kernels)); assert_eq!(oc, oref);
    assert!(k12.invert_runs(&rdsp, dsp.len(), 65_020, &mut oc, kernels)); assert_eq!(oc, oref);
    assert!(k13.invert_runs(&rdsp, dsp.len(), 65_020, &mut oc, kernels)); assert_eq!(oc, oref);
    assert!(k14.invert_runs(&rdsp, dsp.len(), 65_020, &mut oc, kernels)); assert_eq!(oc, oref);
    cases += 1;

    for wlen in [7usize, 8, 9, 15, 16, 17, 18, 32] {
        let mut col = Vec::with_capacity(4000);
        for k in 0..wlen {
            col.extend(std::iter::repeat_n(b'A' + (k % 26) as u8, 25));
        }
        col.push(b'!');
        let r = build_runs(&col, 0, kernels);
        assert!(ref_k.invert_runs(&r, col.len(), 0, &mut oref, kernels));
        assert!(k11.invert_runs(&r, col.len(), 0, &mut oc, kernels)); assert_eq!(oc, oref);
        assert!(k12.invert_runs(&r, col.len(), 0, &mut oc, kernels)); assert_eq!(oc, oref);
        assert!(k13.invert_runs(&r, col.len(), 0, &mut oc, kernels)); assert_eq!(oc, oref);
        assert!(k14.invert_runs(&r, col.len(), 0, &mut oc, kernels)); assert_eq!(oc, oref);
        cases += 1;
    }

    let mut me = Vec::with_capacity(140_000);
    let mut rng = 0xCAFE_1234_5678_9ABCu64;
    while me.len() < 140_000 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let c = ((rng >> 56) as u8) % 24;
        let w = if ((rng >> 48) % 8) == 0 { 20 + ((rng >> 32) as usize % 60) } else { 1 + ((rng >> 32) as usize % 6) };
        me.extend(std::iter::repeat_n(c, w.min(140_000 - me.len())));
    }
    let rme = build_runs(&me, 77_777, kernels);
    assert!(ref_k.invert_runs(&rme, me.len(), 77_777, &mut oref, kernels));
    assert!(k11.invert_runs(&rme, me.len(), 77_777, &mut oc, kernels)); assert_eq!(oc, oref);
    assert!(k12.invert_runs(&rme, me.len(), 77_777, &mut oc, kernels)); assert_eq!(oc, oref);
    assert!(k13.invert_runs(&rme, me.len(), 77_777, &mut oc, kernels)); assert_eq!(oc, oref);
    assert!(k14.invert_runs(&rme, me.len(), 77_777, &mut oc, kernels)); assert_eq!(oc, oref);
    cases += 1;

    cases
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let base = args.get(1).map(String::as_str).unwrap_or("/home/antigravity/admitted-bwt");
    let kernels = Kernels::detect();
    let blocks_auth = [
        ("block-0.L", 899_931usize, 264_045u32, "db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67"),
        ("block-2.L", 900_000usize, 320_807u32, "8b47073232d1d9cff7a0688399cfd405211045e15228706f213780e083fc4174"),
        ("block-85.L", 899_935usize, 325_088u32, "5def63ec66c914e1986a97f8206a8debd09f6e936f0bc8d8f4cab8544ff64fe8"),
        ("block-143.L", 899_869usize, 802_868u32, "eb443421d4d7b459a1db02a00c068f4c2f1e3b7e70f85674228ea2cba30ceba2"),
        ("block-210.L", 899_959usize, 791_429u32, "dd4d0c7d03dd6bc7ec8752ad03b2fd5607222c54bebab8d81ae98a2c277f3dcd"),
        ("block-207.L", 899_998usize, 205_180u32, "7343ef019b44f0e838e68abc09e36d9806a0f45249cf2286702a2e482e555099"),
    ];

    let mut blocks: Vec<(String, Runs, usize, u32, Vec<u8>)> = Vec::new();
    for (fname, want_n, p, want_sha) in blocks_auth {
        let l = fs::read(Path::new(base).join(fname)).expect("read block");
        assert_eq!(l.len(), want_n);
        assert_eq!(sha256_hex(&l), want_sha);
        let runs = build_runs(&l, p, kernels);
        let mut oref = Vec::new();
        assert!(OpenCells::default().invert_runs(&runs, want_n, p, &mut oref, kernels));
        blocks.push((fname.to_string(), runs, want_n, p, oref));
    }
    let dense_l = make_dense_703();
    let dense_runs = build_runs(&dense_l, 123_456, kernels);
    let mut dense_ref = Vec::new();
    assert!(OpenCells::default().invert_runs(&dense_runs, dense_l.len(), 123_456, &mut dense_ref, kernels));
    blocks.push(("dense-703.syn".to_string(), dense_runs, dense_l.len(), 123_456, dense_ref));

    // Exactness check across all 7 blocks + report edge bit budgets
    let mut out = Vec::new();
    let mut k0 = AuditKernel::<0>::default();
    let mut k9 = AuditKernel::<9>::default();
    let mut k11 = AuditKernel::<11>::default();
    let mut k12 = AuditKernel::<12>::default();
    let mut k13 = AuditKernel::<13>::default();
    let mut k14 = AuditKernel::<14>::default();
    println!("=== CYCLE 24A: EDGE BIT BUDGET & LEN-PACK ELIGIBILITY ===");
    for (name, runs, n, p, oref) in &blocks {
        let cap = Cap::classify(runs.records().len(), *n, *p, false).expect("admit");
        assert!(k0.try_run::<false>(runs.records(), *n, *p, cap, &mut out)); assert_eq!(&out, oref, "M0 {}", name);
        assert!(k9.try_run::<false>(runs.records(), *n, *p, cap, &mut out)); assert_eq!(&out, oref, "G2 {}", name);
        assert!(k11.try_run::<false>(runs.records(), *n, *p, cap, &mut out)); assert_eq!(&out, oref, "E1 {}", name);
        assert!(k12.try_run::<false>(runs.records(), *n, *p, cap, &mut out)); assert_eq!(&out, oref, "E2a {}", name);
        assert!(k13.try_run::<false>(runs.records(), *n, *p, cap, &mut out)); assert_eq!(&out, oref, "E2b {}", name);
        assert!(k14.try_run::<false>(runs.records(), *n, *p, cap, &mut out)); assert_eq!(&out, oref, "E3 {}", name);
        let u = k11.inline16.len() as u32;
        let cb = u32::BITS - (u - 1).leading_zeros();
        let free = 32i32 - (k11.last_bits as i32 + cb as i32);
        println!("{:<13} | m={:>6} bits={:>2} U={:>4} code_bits={:>2} free_bits={:>2} -> E2a={} E2b={}",
            name, k11.last_m, k11.last_bits, u, cb, free,
            if free >= 4 { "Len4" } else { "E1" },
            if free >= 4 { "Len4" } else if free == 3 { "Len3" } else { "E1" });
    }
    let exact_cases = verify_exactness(kernels);
    println!("PASS Auth (6/6 SHA-256) + Gate G1/Boundary/Spiral/Multi-Epoch: {} cases byte-exact!\n", exact_cases);

    // =========================================================================
    // PART B: PER-BLOCK S5 + 5-STAGE ATTRIBUTION (15 ROUNDS MEDIAN)
    // =========================================================================
    println!("=== CYCLE 24B: PER-BLOCK S5 EMIT TIMING (us, 15 RDS MEDIAN) ===");
    println!("{:<13} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>7}",
        "Block", "M0:0.5.11", "G2:C23", "E1:ZSpil", "E2a:Len4", "E2b:L4/3", "E3:Zipf", "E1-G2");
    println!("{}", "-".repeat(86));

    let s_rounds = 15usize;
    let mut stage_sums = [[0.0f64; 5]; 6];
    let mut s5_per_blk = vec![[0.0f64; 6]; blocks.len()];

    for bi in 0..blocks.len() {
        let (name, runs, n, p, _) = &blocks[bi];
        let cap = Cap::classify(runs.records().len(), *n, *p, false).unwrap();
        let mut sv = vec![vec![Vec::with_capacity(s_rounds); 5]; 6];
        for r in 0..s_rounds {
            for idx in 0..6 {
                let a = (r + idx) % 6;
                let us = match a {
                    0 => { k0.try_run::<true>(runs.records(), *n, *p, cap, &mut out); k0.last_us }
                    1 => { k9.try_run::<true>(runs.records(), *n, *p, cap, &mut out); k9.last_us }
                    2 => { k11.try_run::<true>(runs.records(), *n, *p, cap, &mut out); k11.last_us }
                    3 => { k12.try_run::<true>(runs.records(), *n, *p, cap, &mut out); k12.last_us }
                    4 => { k13.try_run::<true>(runs.records(), *n, *p, cap, &mut out); k13.last_us }
                    _ => { k14.try_run::<true>(runs.records(), *n, *p, cap, &mut out); k14.last_us }
                };
                sv[a][0].push(us.s1);
                sv[a][1].push(us.s2);
                sv[a][2].push(us.s3);
                sv[a][3].push(us.s4);
                sv[a][4].push(us.s5);
            }
        }
        let mut m5 = [0.0f64; 6];
        for a in 0..6 {
            for s in 0..5 {
                let med = pct(&mut sv[a][s], 1, 2);
                if s == 4 { m5[a] = med; }
                if bi < 6 { stage_sums[a][s] += med; }
            }
            s5_per_blk[bi][a] = m5[a];
        }
        println!("{:<13} | {:>7.1}u | {:>7.1}u | {:>7.1}u | {:>7.1}u | {:>7.1}u | {:>7.1}u | {:>+6.1}u",
            name, m5[0], m5[1], m5[2], m5[3], m5[4], m5[5], m5[2] - m5[1]);
    }
    println!("{}", "-".repeat(86));
    println!("{:<13} | {:>7.1}u | {:>7.1}u | {:>7.1}u | {:>7.1}u | {:>7.1}u | {:>7.1}u | {:>+6.1}u\n",
        "6-BLK S5 SUM", stage_sums[0][4], stage_sums[1][4], stage_sums[2][4], stage_sums[3][4], stage_sums[4][4], stage_sums[5][4],
        stage_sums[2][4] - stage_sums[1][4]);

    println!("=== CYCLE 24C: 5-STAGE ATTRIBUTION (6-BLK SUM us, 15 RDS MEDIAN) ===");
    let labels = [
        "M0: Clone (0.5.11)",
        "G2: C23 Champ",
        "E1: ZeroSpill",
        "E2a: +Len4",
        "E2b: +Len4/Len3",
        "E3: +ZipfRemap",
    ];
    println!("{:<20} | {:>8} | {:>8} | {:>8} | {:>8} | {:>9} | {:>8} | {:>9}",
        "Arm", "S1", "S2", "S3", "S4", "Pre(S1-4)", "S5 Emit", "Sum");
    println!("{}", "-".repeat(98));
    for a in 0..6 {
        let pre = stage_sums[a][0] + stage_sums[a][1] + stage_sums[a][2] + stage_sums[a][3];
        let tot = pre + stage_sums[a][4];
        println!("{:<20} | {:>7.1}u | {:>7.1}u | {:>7.1}u | {:>7.1}u | {:>8.1}u | {:>7.1}u | {:>8.1}u",
            labels[a], stage_sums[a][0], stage_sums[a][1], stage_sums[a][2], stage_sums[a][3], pre, stage_sums[a][4], tot);
    }
    println!("Persistent Scratch (incl. dense-703, excl. 879 KiB out): M0={:.1} KiB | G2={:.1} KiB | E1={:.1} KiB | E2a={:.1} KiB | E2b={:.1} KiB | E3={:.1} KiB\n", k0.scratch_bytes() as f64/1024.0, k9.scratch_bytes() as f64/1024.0, k11.scratch_bytes() as f64/1024.0, k12.scratch_bytes() as f64/1024.0, k13.scratch_bytes() as f64/1024.0, k14.scratch_bytes() as f64/1024.0);

    // =========================================================================
    // PART D: HEADLINE 1T `InverseBwt::invert_runs` CONFRONTATION (25 ROUNDS)
    // =========================================================================
    println!("=== CYCLE 24D: HEADLINE 1T `InverseBwt::invert_runs` CONFRONTATION (25 RDS, p50 / p10 vs A1:Master) ===");
    let rounds = 25usize;
    let mut a1 = OpenCells::default();
    let mut a2 = OpenCells::default();
    let mut c0 = AuditCells::<0>::default();
    let mut c9 = AuditCells::<9>::default();
    let mut c11 = AuditCells::<11>::default();
    let mut c12 = AuditCells::<12>::default();
    let mut c13 = AuditCells::<13>::default();
    let mut c14 = AuditCells::<14>::default();
    let mut obuf = Vec::with_capacity(900_064);

    println!("{:<13} | {:>9} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6}",
        "Block", "A1_p50", "A2/A1", "M0/A1", "G2/A1", "E1/A1", "E2a/A1", "E2b/A1", "E3/A1");
    println!("{}", "-".repeat(90));

    let mut tot_p50 = [0.0f64; 8];
    let mut tot_p10 = [0.0f64; 8];

    for bi in 0..blocks.len() {
        let (name, runs, n, p, _) = &blocks[bi];
        let mut times = vec![Vec::with_capacity(rounds); 8];
        for _ in 0..2 {
            a1.invert_runs(runs, *n, *p, &mut obuf, kernels);
            c12.invert_runs(runs, *n, *p, &mut obuf, kernels);
        }
        for r in 0..rounds {
            for k in 0..8 {
                let arm = (r + k) % 8;
                let t0 = Instant::now();
                let ok = match arm {
                    0 => a1.invert_runs(runs, *n, *p, &mut obuf, kernels),
                    1 => a2.invert_runs(runs, *n, *p, &mut obuf, kernels),
                    2 => c0.invert_runs(runs, *n, *p, &mut obuf, kernels),
                    3 => c9.invert_runs(runs, *n, *p, &mut obuf, kernels),
                    4 => c11.invert_runs(runs, *n, *p, &mut obuf, kernels),
                    5 => c12.invert_runs(runs, *n, *p, &mut obuf, kernels),
                    6 => c13.invert_runs(runs, *n, *p, &mut obuf, kernels),
                    _ => c14.invert_runs(runs, *n, *p, &mut obuf, kernels),
                };
                assert!(ok);
                black_box(&obuf);
                times[arm].push(t0.elapsed().as_nanos() as f64 / 1e3);
            }
        }
        let mut p50 = [0.0f64; 8];
        let mut p10 = [0.0f64; 8];
        for a in 0..8 {
            p50[a] = pct(&mut times[a], 1, 2);
            p10[a] = pct(&mut times[a], 1, 10);
            if bi < 6 {
                tot_p50[a] += p50[a];
                tot_p10[a] += p10[a];
            }
        }
        println!("{:<13} | {:>7.1}us | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x",
            name, p50[0], p50[1]/p50[0], p50[2]/p50[0], p50[3]/p50[0], p50[4]/p50[0], p50[5]/p50[0], p50[6]/p50[0], p50[7]/p50[0]);
    }
    println!("{}", "-".repeat(90));
    println!("{:<13} | {:>7.1}us | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x",
        "6-BLK p50/A1", tot_p50[0], tot_p50[1]/tot_p50[0], tot_p50[2]/tot_p50[0], tot_p50[3]/tot_p50[0], tot_p50[4]/tot_p50[0], tot_p50[5]/tot_p50[0], tot_p50[6]/tot_p50[0], tot_p50[7]/tot_p50[0]);
    println!("{:<13} | {:>7.1}us | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x | {:>5.3}x",
        "6-BLK p10/A1", tot_p10[0], tot_p10[1]/tot_p10[0], tot_p10[2]/tot_p10[0], tot_p10[3]/tot_p10[0], tot_p10[4]/tot_p10[0], tot_p10[5]/tot_p10[0], tot_p10[6]/tot_p10[0], tot_p10[7]/tot_p10[0]);
    println!("6-BLK p50/M0: G2={:.3}x | E1={:.3}x | E2a={:.3}x | E2b={:.3}x | E3={:.3}x",
        tot_p50[3]/tot_p50[2], tot_p50[4]/tot_p50[2], tot_p50[5]/tot_p50[2], tot_p50[6]/tot_p50[2], tot_p50[7]/tot_p50[2]);

    let json = format!(
        "{{\n  \"exact_cases\": {},\n  \"stage_sums_us\": {{\n    \"M0\": [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}],\n    \"G2\": [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}],\n    \"E1\": [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}],\n    \"E2a\": [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}],\n    \"E2b\": [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}],\n    \"E3\": [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}]\n  }},\n  \"headline_6blk_p50_us\": [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}],\n  \"headline_6blk_p10_us\": [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}]\n}}\n",
        exact_cases,
        stage_sums[0][0], stage_sums[0][1], stage_sums[0][2], stage_sums[0][3], stage_sums[0][4],
        stage_sums[1][0], stage_sums[1][1], stage_sums[1][2], stage_sums[1][3], stage_sums[1][4],
        stage_sums[2][0], stage_sums[2][1], stage_sums[2][2], stage_sums[2][3], stage_sums[2][4],
        stage_sums[3][0], stage_sums[3][1], stage_sums[3][2], stage_sums[3][3], stage_sums[3][4],
        stage_sums[4][0], stage_sums[4][1], stage_sums[4][2], stage_sums[4][3], stage_sums[4][4],
        stage_sums[5][0], stage_sums[5][1], stage_sums[5][2], stage_sums[5][3], stage_sums[5][4],
        tot_p50[0], tot_p50[1], tot_p50[2], tot_p50[3], tot_p50[4], tot_p50[5], tot_p50[6], tot_p50[7],
        tot_p10[0], tot_p10[1], tot_p10[2], tot_p10[3], tot_p10[4], tot_p10[5], tot_p10[6], tot_p10[7],
    );
    fs::write("docs/journal/2026-11-21-wormhole-cycle24-s5-zerospill-len4-zipf/cycle24_results.json", json).expect("write json");
}
