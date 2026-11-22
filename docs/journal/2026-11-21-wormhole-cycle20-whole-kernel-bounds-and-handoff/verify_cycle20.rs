#![forbid(unsafe_code)]

//! Wormhole Cycle 20 — Final Round 3 Whole-Kernel Verification & Multi-Worker Confrontation:
//! 1. SHA-256 / Length / Primary-Index authentication of the 6 Wikidata blocks against `manifest.json`.
//! 2. `14,462` Exhaustive & Stress Cases Byte-Exact against `origin/master` `OpenCells` & `plain_invert`:
//!    - All `14,204` exhaustive binary (`n <= 9`) and ternary (`n <= 6`) columns at every primary index `p`,
//!    - `250` pseudo-random multi-cycle, periodic, and multi-epoch (`m > 65,536`) columns via `InverseBwt::invert_runs`,
//!    - `SPIRALS` (`widest = 50,000 > CERTIFIED_WIDTH` + deep spiral `word_len > 1,023` saturating graded handle),
//!    - Forced `Wide` links (`STRIDE = 6`) + forced `wide` `extra` (`self.extra`),
//!    - `#703` `Cap::dense` admitted, `#703` `Cap::dense` refused -> `LazyCells` fallback, and column-route `invert`/`invert_in_place`.
//! 3. Full 21-Block Multi-Regime `.bz2` Stream via `mibz2::BzReader::parallel_workers_with::<WormholePreEmitCells>`
//!    at `W = 1, 2, 4, 8` workers (`18` authenticated Wikidata blocks + `#703` dense admitted + `#703` dense refused +
//!    high-entropy column-route block), verified byte-exact and CRC32-exact against `OpenCells`.
//! 4. `25`-Round 1-Worker & 2-Worker Barrier-Synchronized Native Safe-Rust Confrontation via `InverseBwt::invert_runs`
//!    (`A1: Master OpenCells`, `A2: MasterCtrl OpenCells`, `M0: CloneCells`, `W3: WormholePreEmitCells`),
//!    holding `origin/master`'s `emit` (`S5`) 100% verbatim untouched across all arms.

use mibz2::bwt::{ForcedOpenCells, InverseBwt, LazyCells, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{BzReader, Kernels};
use std::env;
use std::fs;
use std::hint::black_box;
use std::io::{Cursor, Read};
use std::path::Path;
use std::sync::Barrier;
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
// 1. Pure Safe-Rust SHA-256 (`manifest.json` Authentication) & BZip2 Crafter
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
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
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
        let mask = if bits == 64 {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        };
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

fn rle1_encode(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        let b = raw[i];
        let mut run = 1usize;
        while i + run < raw.len() && raw[i + run] == b && run < 255 {
            run += 1;
        }
        if run >= 4 {
            out.extend_from_slice(&[b, b, b, b, (run - 4) as u8]);
        } else {
            out.extend(std::iter::repeat_n(b, run));
        }
        i += run;
    }
    out
}

/// Forward cyclic BWT on modest-size buffers (`n <= 40,000`) for RLE1-valid `BzReader` stream blocks!
fn forward_bwt(raw: &[u8]) -> (Vec<u8>, u32) {
    let rle = rle1_encode(raw);
    let n = rle.len();
    let mut rot: Vec<usize> = (0..n).collect();
    rot.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = rle[(a + k) % n];
            let cb = rle[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut ll = Vec::with_capacity(n);
    let mut orig_ptr = 0u32;
    for (rank, &start) in rot.iter().enumerate() {
        if start == 0 {
            orig_ptr = rank as u32;
        }
        ll.push(rle[(start + n - 1) % n]);
    }
    (ll, orig_ptr)
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

fn write_bz2_block_body(w: &mut BitWriter, ll: &[u8], orig_ptr: u32, kernels: Kernels, strict_rle1: bool) -> u32 {
    let pre_rle1 = plain_invert(ll, orig_ptr);
    let mut unrled = Vec::with_capacity(pre_rle1.len());
    let crc = if strict_rle1 {
        mibz2::rle::unrle(&pre_rle1, &mut unrled, kernels).expect("valid RLE1 for BzReader");
        mibz2::crc::crc32(&unrled, kernels)
    } else {
        mibz2::rle::unrle(&pre_rle1, &mut unrled, kernels)
            .map(|_| mibz2::crc::crc32(&unrled, kernels))
            .unwrap_or(0)
    };

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
    crc
}

fn build_runs(ll: &[u8], orig_ptr: u32, kernels: Kernels) -> Runs {
    let mut w = BitWriter::default();
    w.put(0x425a_6839, 32);
    let crc = write_bz2_block_body(&mut w, ll, orig_ptr, kernels, false);
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

fn craft_stream_bz2(blocks: &[(&[u8], u32)], kernels: Kernels) -> Vec<u8> {
    let mut w = BitWriter::default();
    w.put(0x425a_6839, 32);
    let mut combined = 0u32;
    for &(ll, p) in blocks {
        let crc = write_bz2_block_body(&mut w, ll, p, kernels, true);
        combined = combined.rotate_left(1) ^ crc;
    }
    w.put(0x1772_4538_5090, 48);
    w.put(combined as u64, 32);
    w.finish()
}

// ============================================================================
// 2. Verbatim `origin/master` OutQ16 (`outq.rs`)
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

struct Queues<'a> {
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
impl<'a> Queues<'a> {
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
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }
    fn over(&self) -> bool {
        self.over
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
            for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                .chunks_exact_mut(2)
                .enumerate()
            {
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
                self.id = self.spill.next[self.id as usize];
                self.at = 0;
            }
            let take = dst
                .len()
                .min(CHUNK - self.at)
                .min((self.cut - self.read) as usize);
            let hi = pack | (self.epoch << 16);
            let chunk = if self.id < self.in_out {
                let at = self.id as usize * BYTES;
                &self.out[at..at + BYTES]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][..]
            };
            for (e, s) in dst[..take]
                .iter_mut()
                .zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
            {
                *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
            }
            self.at += take;
            self.read += take as u32;
            dst = &mut dst[take..];
        }
    }
}

// ============================================================================
// 3. Links (`Narrow` & `Wide`), `Elim`, `M0CloneCells`, & `WormholePreEmitCells`
// ============================================================================

trait Links {
    const STRIDE: usize;
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
}
struct Narrow;
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
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
}
impl Links for Wide {
    const STRIDE: usize = 6;
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
    fn width(&self, id: u32) -> u32 {
        self.a[L::STRIDE * id as usize + L::STRIDE - 2]
    }
    #[inline(always)]
    fn set_width(&mut self, id: u32, w: u32) {
        self.a[L::STRIDE * id as usize + L::STRIDE - 2] = w;
    }
    #[inline(always)]
    fn word(&self, id: u32) -> u32 {
        self.a[L::STRIDE * id as usize + L::STRIDE - 1]
    }
    #[inline(always)]
    fn set_word(&mut self, id: u32, w: u32) {
        self.a[L::STRIDE * id as usize + L::STRIDE - 1] = w;
    }
    #[inline(always)]
    fn bottom(&self, id: u32) -> u32 {
        self.a[L::STRIDE * id as usize + L::BOTTOM]
    }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, b: u32) {
        self.a[L::STRIDE * id as usize + L::BOTTOM] = b;
    }
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
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }
    #[inline(always)]
    fn rule4(&self, w: u32) -> usize {
        self.base + (w - 257) as usize * RULE4
    }
    #[inline(always)]
    fn len4(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            self.a[self.rule4(w) + 2]
        }
    }
    #[inline(always)]
    fn join_m<const GRADED: bool>(&mut self, lh: u32, rh: u32) -> u32 {
        if lh == 0 {
            return rh;
        }
        if rh == 0 {
            return lh;
        }
        if GRADED {
            const ID22_MASK: u32 = (1 << 22) - 1;
            let lid = lh & ID22_MASK;
            let rid = rh & ID22_MASK;
            let ll = lh >> 22;
            let rl = rh >> 22;
            let exact_len = if ll < 1023 && rl < 1023 {
                ll + rl
            } else {
                self.len4(lid) + self.len4(rid)
            };
            let id = ((self.top - self.base) / RULE4) as u32 + 257;
            self.a[self.top..self.top + RULE4].copy_from_slice(&[lid, rid, exact_len, u32::MAX]);
            self.top += RULE4;
            (exact_len.min(1023) << 22) | id
        } else {
            let id = ((self.top - self.base) / RULE4) as u32 + 257;
            let len = self.len4(lh) + self.len4(rh);
            self.a[self.top..self.top + RULE4].copy_from_slice(&[lh, rh, len, u32::MAX]);
            self.top += RULE4;
            id
        }
    }
    fn power_m<const GRADED: bool>(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join_m::<GRADED>(acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = self.join_m::<GRADED>(base, base);
        }
    }
    fn eliminate<const SPIRALS: bool, const GRADED: bool>(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let room = if SPIRALS { SPIRAL_RULES * RULE4 } else { RULE4 };
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
                    let head = self.power_m::<GRADED>(self.word(x), q);
                    let word = self.join_m::<GRADED>(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    let tail = self.power_m::<GRADED>(self.word(y), q);
                    let word = self.join_m::<GRADED>(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            left -= w;
            if x == y {
                if GRADED {
                    self.pop_tail(x, 0);
                    self.pop_tail(x, 1);
                } else {
                    self.cut(x, 0);
                    self.cut(x, 1);
                }
                continue;
            }
            let word = self.join_m::<GRADED>(self.word(x), self.word(y));
            if aw <= bw {
                if GRADED {
                    self.pop_tail(x, 0);
                } else {
                    self.cut(x, 0);
                }
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    if GRADED {
                        self.pop_tail(y, 1);
                    } else {
                        self.cut(y, 1);
                    }
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                if GRADED {
                    self.pop_tail(y, 1);
                } else {
                    self.cut(y, 1);
                }
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        *remaining = left;
        done
    }
    fn append4(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
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
                debug_assert_eq!(f_x, b_x);
                f_id = 0;
                break;
            }
            f_id = f_next;
            b_id = b_prev;
        }
        if f_id != 0 && f_id == b_id {
            debug_assert_eq!(f_x + self.width(f_id), b_x);
            self.set_bottom(f_id, f_x);
        }
    }
}

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
        Self {
            all: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
        }
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
        if pos < p {
            visit(c, pos, p - pos, false);
        }
        visit(c, p, 1, true);
        if p + 1 < end {
            visit(c, p + 1, end - p - 1, false);
        }
    } else {
        visit(c, pos, width, width <= limit);
    }
}

fn inline_words(
    spelling: &[u8],
    offsets: &[u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
) {
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
        lens.push(if word.len() <= INLINE {
            word.len() as u8
        } else {
            LONG
        });
    }
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit(
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
            let w = code - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
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

#[derive(Clone, Copy)]
struct Strip16 {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}
#[derive(Clone, Copy)]
struct Band12 {
    end: u32,
    delta: u32,
    code: u32,
}

#[derive(Default)]
struct M0Kernel {
    stack: Vec<u32>,
    spill: Spill,
    strips: Vec<Strip16>,
    bands: Vec<Band12>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
}

struct Pass16<'a> {
    queues: Queues<'a>,
    strips: &'a mut Vec<Strip16>,
    f: First,
}
impl Pass16<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        if width <= limit {
            self.piece(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece(c, pos, width, false, p);
        } else {
            self.take_marked(c, pos, width, p, limit);
        }
    }
    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |sc, x, w, keep| {
            self.piece(sc, x, w, keep, p);
        });
    }
    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
            self.queues.push(a, f.m, w, &f.thin);
            f.thin[a] += w;
            f.m += w;
        } else {
            self.strips.push(Strip16 {
                from: x,
                to: f.all[a],
                width: w,
                byte: c,
            });
            f.thick[a] += 1;
        }
        f.all[a] += w;
    }
}

impl M0Kernel {
    fn ws_bytes(&self, out_cap: usize) -> usize {
        out_cap
            + 4 * self.stack.capacity()
            + 4 * self.spill.next.capacity()
            + BYTES * self.spill.ovf.capacity()
            + 1024 * self.spill.cuts.capacity()
            + 16 * self.strips.capacity()
            + 12 * self.bands.capacity()
            + self.spelling.capacity()
            + 4 * self.offsets.capacity()
            + 4 * self.edges.capacity()
            + 4 * self.extra.capacity()
            + INLINE * self.inline16.capacity()
            + self.lens.capacity()
    }
    fn arena_words(ns: usize, m: usize) -> usize {
        let nodes = 3 * ns + 3;
        let stride = if nodes <= NARROW_NODES {
            Narrow::STRIDE
        } else {
            Wide::STRIDE
        };
        (ns + (stride + RULE4) * nodes).max(m)
    }
    fn induced<L: Links>(&mut self, ns: usize, n: u32, thin: &[u32; 256]) -> [usize; 256] {
        let (image_order, arena) = self.edges.split_at_mut(ns);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0u32;
        for s in self.strips.iter() {
            e.node(s.width, u32::from(s.byte) + 1);
            widest = widest.max(s.width);
        }
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0u32;
        for (i, s) in self.strips.iter().enumerate() {
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
        for &i in image_order.iter() {
            let s = self.strips[i as usize];
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
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        let mut remaining = n;
        let spirals = widest > CERTIFIED_WIDTH;
        let room = if spirals { SPIRAL_RULES * RULE4 } else { RULE4 };
        while !if spirals {
            e.eliminate::<true, false>(&mut remaining)
        } else {
            e.eliminate::<false, false>(&mut remaining)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(&mut self.edges, ns + top + room);
            e = Elim {
                a: &mut self.edges[ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        let mut id = L::next(e.a, 0, 1);
        x = 0;
        while id != 0 {
            let next = L::next(e.a, id, 1);
            let w = e.width(id);
            e.set_bottom(id, x);
            x += w;
            id = next;
        }
        let mut cursors = [0usize; 256];
        id = L::next(e.a, 0, 0);
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256usize {
            cursors[a] = self.bands.len();
            let end = k + thin[a];
            while k < end {
                let node_width = e.width(id);
                let node_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                let dst = e.bottom(id) + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    e.append4(node_word, &mut self.stack, &mut self.spelling);
                    self.spelling.push(a as u8);
                    self.offsets.push(self.spelling.len() as u32);
                    254 + self.offsets.len() as u32
                };
                self.bands.push(Band12 {
                    end: k + width,
                    delta: dst.wrapping_sub(k),
                    code,
                });
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        cursors
    }
    fn try_run(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> bool {
        out.resize(n + INLINE, 0);
        self.strips.clear();
        let mut pass = Pass16 {
            queues: Queues::new(&mut out[..], &mut self.spill, cap.m),
            strips: &mut self.strips,
            f: First::default(),
        };
        maximal(records, |c, pos, width| {
            pass.take(c, pos, width, p, LIMIT);
        });
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        if over || f.m > cap.m || self.strips.len() > cap.strips {
            self.strips.clear();
            self.strips.shrink_to(cap.strips);
            return false;
        }
        let First {
            all,
            thin,
            thick,
            m,
            mark,
        } = f;
        let first = starts(all);
        let mut write = starts(thick);
        let ns = self.strips.len();
        let need = Self::arena_words(ns, m as usize);
        if self.edges.len() < need {
            self.edges.clear();
            self.edges.reserve_exact(need);
            self.edges.resize(need, 0);
        }
        for (i, s) in self.strips.iter_mut().enumerate() {
            let a = usize::from(s.byte);
            s.to += first[a];
            self.edges[write[a] as usize] = i as u32;
            write[a] += 1;
        }
        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        let cursors = if 3 * ns + 3 <= NARROW_NODES {
            self.induced::<Narrow>(ns, n as u32, &thin)
        } else {
            self.induced::<Wide>(ns, n as u32, &thin)
        };
        let words = self.offsets.len() - 1;
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + words as u32) > (u32::MAX >> bits);
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        let mut k = 0u32;
        let out_ro = &out[..];
        for a in 0..256usize {
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            let mut queue = Reader::new(out_ro, &self.spill, &heads, a);
            for b in &self.bands[cursors[a]..end] {
                let span = (b.end - k) as usize;
                let dst = k.wrapping_add(b.delta) as usize;
                let pack = if wide { 0 } else { b.code << bits };
                if wide {
                    self.extra[dst..dst + span].fill(b.code);
                }
                queue.wire(&mut self.edges[dst..dst + span], pack);
                k = b.end;
            }
        }
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
        );
        emit(
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
        true
    }
}

#[derive(Default)]
struct M0CloneCells {
    open: M0Kernel,
    lazy: LazyCells,
}
impl InverseBwt for M0CloneCells {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if let Some(cap) = Cap::classify(r.records().len(), n, p, false) {
            if self.open.try_run(r.records(), n, p, cap, out) {
                return true;
            }
        }
        self.lazy.invert_runs(r, n, p, out, k)
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(l, c, p, out);
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(l, c, p)
    }
    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        c: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.lazy.invert_checkpointed(l, c, checkpoints, p, out);
    }
}

#[derive(Default)]
struct WormholePreEmitKernel {
    stack: Vec<u32>,
    spill: Spill,
    sb_buf: Vec<[u32; 3]>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    force_wide_links: bool,
    force_wide_extra: bool,
}

struct PassMux<'a> {
    queues: Queues<'a>,
    sb_buf: &'a mut Vec<[u32; 3]>,
    f: First,
}
impl PassMux<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        if width <= limit {
            self.piece(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece(c, pos, width, false, p);
        } else {
            self.take_marked(c, pos, width, p, limit);
        }
    }
    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |sc, x, w, keep| {
            self.piece(sc, x, w, keep, p);
        });
    }
    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
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

impl WormholePreEmitKernel {
    fn ws_bytes(&self, out_cap: usize) -> usize {
        out_cap
            + 4 * self.stack.capacity()
            + 4 * self.spill.next.capacity()
            + BYTES * self.spill.ovf.capacity()
            + 1024 * self.spill.cuts.capacity()
            + 12 * self.sb_buf.capacity()
            + self.spelling.capacity()
            + 4 * self.offsets.capacity()
            + 4 * self.edges.capacity()
            + 4 * self.extra.capacity()
            + INLINE * self.inline16.capacity()
            + self.lens.capacity()
    }
    fn arena_words(&self, ns: usize, m: usize) -> usize {
        let nodes = 3 * ns + 3;
        let stride = if !self.force_wide_links && nodes <= NARROW_NODES {
            Narrow::STRIDE
        } else {
            Wide::STRIDE
        };
        (ns + (stride + RULE4) * nodes).max(m)
    }
    fn reset_inline16(&mut self) {
        if self.inline16.len() < 256 {
            self.inline16.clear();
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0u8; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.resize(256, 1);
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);
    }
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn emit_short_or_long<L: Links>(
        e: &mut Elim<'_, L>,
        raw_word: u32,
        a_byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
        offsets: &mut Vec<u32>,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
    ) -> u32 {
        let h_len = (raw_word >> 22) as usize;
        let node_id = raw_word & 0x003F_FFFF;
        let code = inline16.len() as u32;
        if h_len <= 15 {
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
            inline16.push(v);
            lens.push((h_len + 1) as u8);
            offsets.push(spelling.len() as u32);
        } else {
            let start_sp = spelling.len();
            e.append4(node_id, stack, spelling);
            spelling.push(a_byte);
            offsets.push(spelling.len() as u32);
            let mut v = [0u8; INLINE];
            v.copy_from_slice(&spelling[start_sp..start_sp + INLINE]);
            inline16.push(v);
            lens.push(LONG);
        }
        code
    }
    fn induced<L: Links>(&mut self, ns: usize, n: u32, m: u32, thin: &[u32; 256]) -> [usize; 256] {
        let (image_order, arena) = self.edges.split_at_mut(ns);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0u32;
        for s in self.sb_buf.iter() {
            let s_w = s[2] >> 8;
            let s_b = s[2] & 0xFF;
            e.node(s_w, (1u32 << 22) | (s_b + 1));
            widest = widest.max(s_w);
        }
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
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
        for &i in image_order.iter() {
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
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        let mut remaining = n;
        let spirals = widest > CERTIFIED_WIDTH;
        let room = if spirals { SPIRAL_RULES * RULE4 } else { RULE4 };
        while !if spirals {
            e.eliminate::<true, true>(&mut remaining)
        } else {
            e.eliminate::<false, true>(&mut remaining)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(&mut self.edges, ns + top + room);
            e = Elim {
                a: &mut self.edges[ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        e.compute_bottom_bidi(m);
        let mut cursors = [0usize; 256];
        let mut id = L::next(e.a, 0, 0);
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256usize {
            cursors[a] = self.sb_buf.len();
            let end = k + thin[a];
            while k < end {
                let node_width = e.width(id);
                let raw_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                let dst = e.bottom(id) + in_node;
                let code = if raw_word == 0 {
                    a as u32
                } else {
                    Self::emit_short_or_long(
                        &mut e,
                        raw_word,
                        a as u8,
                        &mut self.stack,
                        &mut self.spelling,
                        &mut self.offsets,
                        &mut self.inline16,
                        &mut self.lens,
                    )
                };
                self.sb_buf.push([k + width, dst.wrapping_sub(k), code]);
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        cursors
    }
    fn try_run_u16(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> bool {
        out.resize(n + INLINE, 0);
        self.sb_buf.clear();
        let mut pass = PassMux {
            queues: Queues::new(&mut out[..], &mut self.spill, cap.m),
            sb_buf: &mut self.sb_buf,
            f: First::default(),
        };
        maximal(records, |c, pos, width| {
            pass.take(c, pos, width, p, LIMIT);
        });
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        if over || f.m > cap.m || self.sb_buf.len() > cap.strips {
            self.sb_buf.clear();
            self.sb_buf.shrink_to(cap.strips);
            return false;
        }
        let First {
            all,
            thin,
            thick,
            m,
            mark,
        } = f;
        let first = starts(all);
        let mut write = starts(thick);
        let ns = self.sb_buf.len();
        let need = self.arena_words(ns, m as usize);
        if self.edges.len() < need {
            self.edges.clear();
            self.edges.reserve_exact(need);
            self.edges.resize(need, 0);
        }
        for (i, s) in self.sb_buf.iter_mut().enumerate() {
            let a = (s[2] & 0xFF) as usize;
            s[1] += first[a];
            self.edges[write[a] as usize] = i as u32;
            write[a] += 1;
        }
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        self.reset_inline16();
        let cursors = if !self.force_wide_links && 3 * ns + 3 <= NARROW_NODES {
            self.induced::<Narrow>(ns, n as u32, m, &thin)
        } else {
            self.induced::<Wide>(ns, n as u32, m, &thin)
        };
        let words = self.inline16.len() - 256;
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = self.force_wide_extra || (255 + words as u32) > (u32::MAX >> bits);
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        let mut k = 0u32;
        let out_ro = &out[..];
        for a in 0..256usize {
            let end = cursors.get(a + 1).copied().unwrap_or(self.sb_buf.len());
            let mut queue = Reader::new(out_ro, &self.spill, &heads, a);
            for b in &self.sb_buf[cursors[a]..end] {
                let b_end = b[0];
                let b_delta = b[1];
                let b_code = b[2];
                let span = (b_end - k) as usize;
                let dst = k.wrapping_add(b_delta) as usize;
                let pack = if wide { 0 } else { b_code << bits };
                if wide {
                    self.extra[dst..dst + span].fill(b_code);
                }
                queue.wire(&mut self.edges[dst..dst + span], pack);
                k = b_end;
            }
        }
        emit(
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
        true
    }
}

#[derive(Default)]
pub struct WormholePreEmitCellsOf<const FORCED: bool> {
    open: WormholePreEmitKernel,
    lazy: LazyCells,
    pub opened: usize,
    pub capped: usize,
}
pub type WormholePreEmitCells = WormholePreEmitCellsOf<false>;
pub type ForcedWormholePreEmitCells = WormholePreEmitCellsOf<true>;

impl<const FORCED: bool> InverseBwt for WormholePreEmitCellsOf<FORCED> {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if let Some(cap) = Cap::classify(r.records().len(), n, p, FORCED) {
            if self.open.try_run_u16(r.records(), n, p, cap, out) {
                self.opened += 1;
                return true;
            }
            self.capped += 1;
        }
        self.lazy.invert_runs(r, n, p, out, k)
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(l, c, p, out);
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(l, c, p)
    }
    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        c: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.lazy.invert_checkpointed(l, c, checkpoints, p, out);
    }
}

// ============================================================================
// 4. Exhaustive `14,462`-Case Suite & Synthetic Regime Builders
// ============================================================================

fn make_dense_admitted_col(target_n: usize) -> Vec<u8> {
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
            if l.len() >= target_n {
                break;
            }
            let mut s = (step() % 90) as u8 + 32;
            if l.last() == Some(&s) {
                s = if s == 121 { 32 } else { s + 1 };
            }
            let w_thin = if step() % 4 == 0 { 2 } else { 1 };
            l.extend(std::iter::repeat_n(s, w_thin.min(target_n - l.len())));
        }
    }
    while l.len() < target_n {
        l.push(b'x');
    }
    l
}

fn verify_full_suite(kernels: Kernels) -> usize {
    let mut wh_k = WormholePreEmitKernel::default();
    let mut master_forced = ForcedOpenCells::default();
    let mut master_normal = OpenCells::default();
    let mut wh_forced = ForcedWormholePreEmitCells::default();
    let mut wh_normal = WormholePreEmitCells::default();
    let mut out_ref = Vec::new();
    let mut out_cand = Vec::new();
    let mut cases = 0usize;

    // 1. All `14,204` exhaustive binary (`n <= 9`) and ternary (`n <= 6`) columns at every `p`
    for (alpha, max_n) in [(2u32, 9usize), (3u32, 6usize)] {
        for n in 1..=max_n {
            for code in 0..alpha.pow(n as u32) {
                let mut tmp = code;
                let ll: Vec<u8> = (0..n)
                    .map(|_| {
                        let d = (tmp % alpha) as u8;
                        tmp /= alpha;
                        d
                    })
                    .collect();
                let recs = records_of(&ll);
                for p in 0..n as u32 {
                    let want = plain_invert(&ll, p);
                    assert!(wh_k.try_run_u16(&recs, n, p, Cap::NONE, &mut out_cand));
                    assert_eq!(out_cand, want);
                    cases += 1;
                }
            }
        }
    }

    // 2. `250` pseudo-random multi-cycle, periodic, and multi-epoch (`m > 65,536`) columns
    //    verified through `Scanner -> runs::front -> InverseBwt::invert_runs` against `OpenCells`!
    let mut rng = 0xD1B5_4A32_D192_ED03u64;
    let mut nxt = || -> u64 {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    for t in 0..250usize {
        let n = if t < 245 {
            1 + (nxt() as usize % 2048)
        } else {
            // Multi-epoch `m > 65,536`!
            135_000 + (t - 245) * 8_000
        };
        let mut ll = Vec::with_capacity(n);
        if t % 9 == 0 {
            // Periodic column (short cycles, exercises `repeat()`)
            let per = 1 + (nxt() as usize % 19);
            let base: Vec<u8> = (0..per).map(|_| (nxt() % 7) as u8).collect();
            while ll.len() < n {
                ll.push(base[ll.len() % per]);
            }
        } else {
            while ll.len() < n {
                let sym = (nxt() % 16) as u8;
                let w = if nxt() % 5 == 0 {
                    17 + (nxt() as usize % 180)
                } else {
                    1 + (nxt() as usize % 12)
                };
                ll.extend(std::iter::repeat_n(sym, w.min(n - ll.len())));
            }
        }
        let p = (nxt() as usize % n) as u32;
        let runs = build_runs(&ll, p, kernels);
        assert!(master_forced.invert_runs(&runs, n, p, &mut out_ref, kernels));
        assert!(wh_forced.invert_runs(&runs, n, p, &mut out_cand, kernels));
        assert_eq!(out_cand, out_ref, "random mismatch t={t} n={n}");
        cases += 1;
    }

    // 3. `SPIRALS` (`widest = 50,000` + deep spiral `word_len > 1,023` saturating graded handle!)
    let mut spiral_l = Vec::with_capacity(55_000);
    spiral_l.extend(std::iter::repeat_n(b'A', 50_000));
    for i in 0..200u32 {
        spiral_l.push(b'B' + (i % 13) as u8);
        spiral_l.extend(std::iter::repeat_n(b'Z', 17));
    }
    let runs_sp = build_runs(&spiral_l, 25_000, kernels);
    assert!(master_normal.invert_runs(&runs_sp, spiral_l.len(), 25_000, &mut out_ref, kernels));
    assert!(wh_normal.invert_runs(&runs_sp, spiral_l.len(), 25_000, &mut out_cand, kernels));
    assert_eq!(out_cand, out_ref);
    cases += 1;

    let mut deep_spiral = Vec::with_capacity(70_000);
    deep_spiral.extend(std::iter::repeat_n(b'X', 65_000));
    deep_spiral.extend(std::iter::repeat_n(b'Y', 20));
    deep_spiral.push(b'Q');
    let runs_dsp = build_runs(&deep_spiral, 65_020, kernels);
    assert!(master_forced.invert_runs(&runs_dsp, deep_spiral.len(), 65_020, &mut out_ref, kernels));
    assert!(wh_forced.invert_runs(&runs_dsp, deep_spiral.len(), 65_020, &mut out_cand, kernels));
    assert_eq!(out_cand, out_ref);
    cases += 1;

    // 4. Forced `Wide` links (`STRIDE = 6`) + forced `wide` `extra` (`self.extra`)
    wh_forced.open.force_wide_links = true;
    wh_forced.open.force_wide_extra = true;
    assert!(master_normal.invert_runs(&runs_sp, spiral_l.len(), 25_000, &mut out_ref, kernels));
    assert!(wh_forced.invert_runs(&runs_sp, spiral_l.len(), 25_000, &mut out_cand, kernels));
    assert_eq!(out_cand, out_ref);
    wh_forced.open.force_wide_links = false;
    wh_forced.open.force_wide_extra = false;
    cases += 1;

    // 5. `#703` `Cap::dense` ADMITTED + `#703` `Cap::dense` REFUSED -> `LazyCells` + column `invert_in_place`
    let dense_admit = make_dense_admitted_col(200_000);
    let runs_da = build_runs(&dense_admit, 12_345, kernels);
    let opened_b = wh_normal.opened;
    assert!(master_normal.invert_runs(&runs_da, dense_admit.len(), 12_345, &mut out_ref, kernels));
    assert!(wh_normal.invert_runs(&runs_da, dense_admit.len(), 12_345, &mut out_cand, kernels));
    assert_eq!(wh_normal.opened, opened_b + 1);
    assert_eq!(out_cand, out_ref);
    cases += 1;

    let mut dense_refuse = Vec::with_capacity(120_000);
    while dense_refuse.len() < 120_000 {
        let sym = ((nxt() % 40) as u8) + 32;
        let w = ((nxt() % 3) as usize) + 4;
        dense_refuse.extend(std::iter::repeat_n(sym, w.min(120_000 - dense_refuse.len())));
    }
    let runs_dr = build_runs(&dense_refuse, 5_432, kernels);
    let capped_b = wh_normal.capped;
    let m_dr = master_normal.invert_runs(&runs_dr, dense_refuse.len(), 5_432, &mut out_ref, kernels);
    let w_dr = wh_normal.invert_runs(&runs_dr, dense_refuse.len(), 5_432, &mut out_cand, kernels);
    assert_eq!(w_dr, m_dr);
    assert_eq!(wh_normal.capped, capped_b + 1);
    assert_eq!(out_cand, out_ref);
    cases += 1;

    // Column-route `invert` & `invert_in_place` delegation check
    let mut col_counts = [0u32; 256];
    for &b in &spiral_l {
        col_counts[b as usize] += 1;
    }
    master_normal.invert(&spiral_l, &col_counts, 25_000, &mut out_ref);
    wh_normal.invert(&spiral_l, &col_counts, 25_000, &mut out_cand);
    assert_eq!(out_cand, out_ref);
    cases += 1;

    let mut inp_ref = spiral_l.clone();
    let mut inp_cand = spiral_l.clone();
    let r_ip1 = master_normal.invert_in_place(&mut inp_ref, &col_counts, 25_000);
    let r_ip2 = wh_normal.invert_in_place(&mut inp_cand, &col_counts, 25_000);
    assert_eq!(r_ip1, r_ip2);
    assert_eq!(inp_cand, inp_ref);
    cases += 1;

    cases
}

fn median_u64(v: &mut [u64]) -> f64 {
    v.sort_unstable();
    v[v.len() / 2] as f64 / 1e3
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let data_dir = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("/home/antigravity/admitted-bwt");
    let json_out = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("/tmp/cycle20_results.json");
    let kernels = Kernels::detect();

    // =========================================================================
    // Step 1: Authenticate all 6 Wikidata blocks against `manifest.json`
    // =========================================================================
    let blocks_auth = [
        (
            "block-0.L",
            899_931usize,
            264_045u32,
            "db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67",
        ),
        (
            "block-2.L",
            900_000usize,
            320_807u32,
            "8b47073232d1d9cff7a0688399cfd405211045e15228706f213780e083fc4174",
        ),
        (
            "block-85.L",
            899_935usize,
            325_088u32,
            "5def63ec66c914e1986a97f8206a8debd09f6e936f0bc8d8f4cab8544ff64fe8",
        ),
        (
            "block-143.L",
            899_869usize,
            802_868u32,
            "eb443421d4d7b459a1db02a00c068f4c2f1e3b7e70f85674228ea2cba30ceba2",
        ),
        (
            "block-210.L",
            899_959usize,
            791_429u32,
            "dd4d0c7d03dd6bc7ec8752ad03b2fd5607222c54bebab8d81ae98a2c277f3dcd",
        ),
        (
            "block-207.L",
            899_998usize,
            205_180u32,
            "7343ef019b44f0e838e68abc09e36d9806a0f45249cf2286702a2e482e555099",
        ),
    ];

    let mut raw_cols = Vec::with_capacity(7);
    let mut meta_list = Vec::with_capacity(7);
    for (fname, want_n, p, want_sha) in blocks_auth {
        let bytes = fs::read(Path::new(data_dir).join(fname)).expect("read authenticated block");
        assert_eq!(bytes.len(), want_n, "length mismatch on {fname}");
        assert_eq!(sha256_hex(&bytes), want_sha, "SHA-256 mismatch on {fname}");
        raw_cols.push(bytes);
        meta_list.push((fname, want_n, p));
    }
    println!("PASS Authentication: all 6 Wikidata blocks match `manifest.json` (n, primary_index, SHA-256)!");

    // =========================================================================
    // Step 2: Run `14,462` Exhaustive & Stress Cases
    // =========================================================================
    let suite_cases = verify_full_suite(kernels);
    println!(
        "PASS Gate G1 + Multi-Epoch + SPIRALS (len>1023) + WideLinks + WideExtra + #703 Cap::dense Admit/Refuse + Column Fallback: {suite_cases} cases byte-exact!"
    );

    // =========================================================================
    // Step 3: Full 21-Block Multi-Regime `.bz2` Stream via `BzReader::parallel_workers_with`
    //         at `W = 1, 2, 4, 8` Workers!
    // =========================================================================
    // Build 3 RLE1-valid synthetic blocks covering:
    //  (a) `#703` dense-band block (`r/n ~ 0.18`),
    //  (b) `#703` dense-refused -> `LazyCells` block,
    //  (c) high-entropy column-path block (`takes_runs == false`).
    let mut raw_a = Vec::with_capacity(18_000);
    for i in 0..600u32 {
        raw_a.extend_from_slice(b"{\"id\":\"Q");
        raw_a.extend_from_slice(&(10_000 + i).to_string().into_bytes());
        raw_a.extend_from_slice(b"\",\"claims\":{\"P31\":");
        raw_a.push(b'A' + (i % 26) as u8);
        raw_a.extend_from_slice(b"}}");
    }
    let (syn_a_col, syn_a_p) = forward_bwt(&raw_a);

    let mut raw_b = Vec::with_capacity(12_000);
    let mut s_rng = 0xCAFE_BABE_1234_5678u64;
    for _ in 0..3_000 {
        s_rng = s_rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let b1 = 33 + ((s_rng >> 32) % 90) as u8;
        let b2 = 33 + ((s_rng >> 48) % 90) as u8;
        raw_b.extend_from_slice(&[b1, b1, b2, b1]);
    }
    let (syn_b_col, syn_b_p) = forward_bwt(&raw_b);

    let mut raw_c = Vec::with_capacity(8_192);
    for _ in 0..8_192 {
        s_rng = s_rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        raw_c.push((s_rng >> 56) as u8);
    }
    let (syn_c_col, syn_c_p) = forward_bwt(&raw_c);

    let mut stream_blocks: Vec<(&[u8], u32)> = Vec::with_capacity(21);
    for rep in 0..3 {
        for i in 0..6 {
            stream_blocks.push((raw_cols[i].as_slice(), meta_list[i].2));
        }
        match rep {
            0 => stream_blocks.push((syn_a_col.as_slice(), syn_a_p)),
            1 => stream_blocks.push((syn_b_col.as_slice(), syn_b_p)),
            _ => stream_blocks.push((syn_c_col.as_slice(), syn_c_p)),
        }
    }
    assert_eq!(stream_blocks.len(), 21);
    let stream_bz2 = craft_stream_bz2(&stream_blocks, kernels);

    let mut ref_stream_out = Vec::new();
    BzReader::parallel_workers_with::<OpenCells>(Cursor::new(stream_bz2.clone()), 2, 8, kernels)
        .read_to_end(&mut ref_stream_out)
        .expect("OpenCells 21-block stream decode");
    let stream_crc = mibz2::crc::crc32(&ref_stream_out, kernels);

    for w_threads in [1usize, 2, 4, 8] {
        let mut cand_stream_out = Vec::with_capacity(ref_stream_out.len());
        BzReader::parallel_workers_with::<WormholePreEmitCells>(
            Cursor::new(stream_bz2.clone()),
            w_threads,
            4 * w_threads,
            kernels,
        )
        .read_to_end(&mut cand_stream_out)
        .expect("WormholePreEmitCells 21-block stream decode");
        assert_eq!(
            cand_stream_out, ref_stream_out,
            "BzReader mismatch at W={w_threads}"
        );
    }
    println!(
        "PASS Multi-Worker `BzReader::parallel_workers_with::<WormholePreEmitCells>` at W = 1, 2, 4, 8: 21 blocks ({:.2} MiB unrled, CRC32 = {:#010x}) byte-exact!",
        ref_stream_out.len() as f64 / (1024.0 * 1024.0),
        stream_crc
    );

    // =========================================================================
    // Step 4: `25`-Round 1-Worker & 2-Worker Barrier-Synchronized Confrontation
    //         (`A1: Master OpenCells`, `A2: Ctrl`, `M0: Clone`, `W3: WormholePreEmitCells`)
    // =========================================================================
    let dense_900k = make_dense_admitted_col(900_000);
    raw_cols.push(dense_900k);
    meta_list.push(("dense-703.syn", 900_000, 123_456));

    let prepared_runs: Vec<Runs> = raw_cols
        .iter()
        .zip(meta_list.iter())
        .map(|(col, &(_, _, p))| build_runs(col, p, kernels))
        .collect();

    let rounds = 25usize;
    println!(
        "\n=== CYCLE 20 CONFRONTATION ({rounds} PAIRED ROUNDS: 1-Worker & 2-Worker Barrier Sync via `InverseBwt::invert_runs`) ==="
    );
    println!(
        "{:<13} | {:>13} | {:>8} {:>6} {:>8} ({:>6}, {:>6}) | {:>8} {:>8} {:>8} ({:>6}, {:>6})",
        "Block",
        "WS(M0->W3 KB)",
        "1w_A1Mst",
        "A2/A1",
        "1w_W3",
        "W3/M0",
        "W3/A1",
        "2w_A1Mst",
        "2w_M0",
        "2w_W3",
        "W3/M0",
        "W3/A1"
    );
    println!("{}", "-".repeat(124));

    let mut json_rows = Vec::new();
    for idx in 0..7 {
        let (name, n_exp, p) = meta_list[idx];
        let runs_obj = &prepared_runs[idx];

        let (ws_m0, ws_w3) = {
            let mut km = M0Kernel::default();
            let mut kw = WormholePreEmitKernel::default();
            let mut ow = Vec::with_capacity(n_exp + INLINE);
            km.try_run(runs_obj.records(), n_exp, p, Cap::NONE, &mut ow);
            kw.try_run_u16(runs_obj.records(), n_exp, p, Cap::NONE, &mut ow);
            (km.ws_bytes(n_exp + INLINE), kw.ws_bytes(n_exp + INLINE))
        };

        // 1-Worker 4-arm rotated confrontation
        let mut a1 = OpenCells::default();
        let mut a2 = OpenCells::default();
        let mut m0 = M0CloneCells::default();
        let mut w3 = WormholePreEmitCells::default();
        let mut out1 = Vec::with_capacity(900_064);
        let mut t1: [Vec<u64>; 4] = std::array::from_fn(|_| Vec::with_capacity(rounds));
        for r in 0..(rounds + 3) {
            for rot in 0..4 {
                let arm = (r + rot) % 4;
                let t0 = Instant::now();
                match arm {
                    0 => {
                        assert!(a1.invert_runs(black_box(runs_obj), n_exp, p, &mut out1, kernels));
                    }
                    1 => {
                        assert!(a2.invert_runs(black_box(runs_obj), n_exp, p, &mut out1, kernels));
                    }
                    2 => {
                        assert!(m0.invert_runs(black_box(runs_obj), n_exp, p, &mut out1, kernels));
                    }
                    _ => {
                        assert!(w3.invert_runs(black_box(runs_obj), n_exp, p, &mut out1, kernels));
                    }
                }
                let dt = t0.elapsed().as_nanos() as u64;
                black_box(out1[0]);
                if r >= 3 {
                    t1[arm].push(dt);
                }
            }
        }
        let w1_a1 = median_u64(&mut t1[0]);
        let w1_a2 = median_u64(&mut t1[1]);
        let w1_m0 = median_u64(&mut t1[2]);
        let w1_w3 = median_u64(&mut t1[3]);

        // 2-Worker barrier-synchronized concurrent confrontation on the 2 physical vCPUs!
        let idx_peer = (idx + 3) % 7;
        let (_, n_peer, p_peer) = meta_list[idx_peer];
        let runs_peer = &prepared_runs[idx_peer];
        let mut t2: [Vec<u64>; 4] = std::array::from_fn(|_| Vec::with_capacity(rounds));
        let bar = Barrier::new(2);
        std::thread::scope(|s| {
            let bar_ref = &bar;
            let peer = s.spawn(move || {
                let mut pa1 = OpenCells::default();
                let mut pa2 = OpenCells::default();
                let mut pm0 = M0CloneCells::default();
                let mut pw3 = WormholePreEmitCells::default();
                let mut op = Vec::with_capacity(900_064);
                for r in 0..(rounds + 3) {
                    for rot in 0..4 {
                        let arm = (r + rot) % 4;
                        bar_ref.wait();
                        match arm {
                            0 => {
                                pa1.invert_runs(black_box(runs_peer), n_peer, p_peer, &mut op, kernels);
                            }
                            1 => {
                                pa2.invert_runs(black_box(runs_peer), n_peer, p_peer, &mut op, kernels);
                            }
                            2 => {
                                pm0.invert_runs(black_box(runs_peer), n_peer, p_peer, &mut op, kernels);
                            }
                            _ => {
                                pw3.invert_runs(black_box(runs_peer), n_peer, p_peer, &mut op, kernels);
                            }
                        }
                        black_box(op[0]);
                        bar_ref.wait();
                    }
                }
            });

            let mut ma1 = OpenCells::default();
            let mut ma2 = OpenCells::default();
            let mut mm0 = M0CloneCells::default();
            let mut mw3 = WormholePreEmitCells::default();
            let mut ow = Vec::with_capacity(900_064);
            for r in 0..(rounds + 3) {
                for rot in 0..4 {
                    let arm = (r + rot) % 4;
                    bar.wait();
                    let t0 = Instant::now();
                    match arm {
                        0 => {
                            ma1.invert_runs(black_box(runs_obj), n_exp, p, &mut ow, kernels);
                        }
                        1 => {
                            ma2.invert_runs(black_box(runs_obj), n_exp, p, &mut ow, kernels);
                        }
                        2 => {
                            mm0.invert_runs(black_box(runs_obj), n_exp, p, &mut ow, kernels);
                        }
                        _ => {
                            mw3.invert_runs(black_box(runs_obj), n_exp, p, &mut ow, kernels);
                        }
                    }
                    let dt = t0.elapsed().as_nanos() as u64;
                    black_box(ow[0]);
                    bar.wait();
                    if r >= 3 {
                        t2[arm].push(dt);
                    }
                }
            }
            peer.join().expect("peer ok");
        });
        let w2_a1 = median_u64(&mut t2[0]);
        let w2_a2 = median_u64(&mut t2[1]);
        let w2_m0 = median_u64(&mut t2[2]);
        let w2_w3 = median_u64(&mut t2[3]);

        println!(
            "{:<13} | {:>4}->{:>4} KB | {:>6.1}us {:>5.3}x {:>6.1}us ({:>5.3}x, {:>5.3}x) | {:>6.1}us {:>6.1}us {:>6.1}us ({:>5.3}x, {:>5.3}x)",
            name,
            ws_m0 / 1024,
            ws_w3 / 1024,
            w1_a1,
            w1_a2 / w1_a1,
            w1_w3,
            w1_w3 / w1_m0,
            w1_w3 / w1_a1,
            w2_a1,
            w2_m0,
            w2_w3,
            w2_w3 / w2_m0,
            w2_w3 / w2_a1,
        );

        json_rows.push(format!(
            "    {{\n      \"block\": \"{}\",\n      \"ws_bytes\": {{\"m0\": {}, \"w3\": {}}},\n      \"w1_us\": {{\"a1_master\": {:.2}, \"a2_ctrl\": {:.2}, \"m0_clone\": {:.2}, \"w3_pre_emit\": {:.2}}},\n      \"w1_ratios\": {{\"a2_a1\": {:.4}, \"w3_m0\": {:.4}, \"w3_a1\": {:.4}}},\n      \"w2_us\": {{\"a1_master\": {:.2}, \"a2_ctrl\": {:.2}, \"m0_clone\": {:.2}, \"w3_pre_emit\": {:.2}}},\n      \"w2_ratios\": {{\"a2_a1\": {:.4}, \"w3_m0\": {:.4}, \"w3_a1\": {:.4}}}\n    }}",
            name,
            ws_m0, ws_w3,
            w1_a1, w1_a2, w1_m0, w1_w3,
            w1_a2 / w1_a1, w1_w3 / w1_m0, w1_w3 / w1_a1,
            w2_a1, w2_a2, w2_m0, w2_w3,
            w2_a2 / w2_a1, w2_w3 / w2_m0, w2_w3 / w2_a1,
        ));
    }

    let json = format!(
        "{{\n  \"suite_cases\": {suite_cases},\n  \"bzreader_stream_blocks\": 21,\n  \"bzreader_stream_bytes\": {},\n  \"bzreader_stream_crc32\": \"{:#010x}\",\n  \"blocks\": [\n{}\n  ]\n}}\n",
        ref_stream_out.len(),
        stream_crc,
        json_rows.join(",\n")
    );
    fs::write(json_out, json).expect("write json");
}
