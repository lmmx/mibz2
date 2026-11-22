#![forbid(unsafe_code)]

//! Wormhole Cycle 19 Verification & Native Safe-Rust Confrontation Harness:
//! 1. Obligation #5 — Upstream `runs::front` `CAP = 127` Record-Splitting Boundary Audit
//!    (`CAP127-u16` vs `CAP255-u16` vs `Unsplit-u32` across 6 authenticated blocks + `dense-703.syn`).
//! 2. Unified `WormholePreEmitCells: InverseBwt` Synthesis:
//!    - `[u32; 3]` `StripBandMux` (`12 B` packed `Strip` `[from, to, (width << 8) | byte]` multiplexed
//!      with `12 B` `Band` `[end, delta, code]` in `sb_buf: Vec<[u32; 3]>` — `-60..125 KiB` scratch),
//!    - Constant-Sentinel `pop_tail` + `(len10 << 22) | id22` Graded Semiring Handle (`Theorem 15.1`),
//!    - Pohl (1971) 2-Way Bidirectional `side-1` Bottom Walk (`D = H / 2`),
//!    - `Theorem 17.1` Prefix-15 Fixed-Stack `[u32; 16]` Direct `inline16` Unfold (`0.0 us` `S3b`),
//!    - Full `Narrow` + `Wide` links, `SPIRALS`, `wide` `extra`, `#703` `Cap::dense`, and `LazyCells` fallback,
//!    - `S5 emit` held 100% verbatim identical to `origin/master` (`90e671966`).

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
// 1. Minimal Valid BZip2 Block Crafter (`Runs` + `Symbols` for `invert_runs`)
// ============================================================================

struct BitWriter {
    buf: Vec<u8>,
    acc: u64,
    bits: u32,
}
impl BitWriter {
    fn new() -> Self {
        Self {
            buf: Vec::new(),
            acc: 0,
            bits: 0,
        }
    }
    fn push(&mut self, width: u32, val: u64) {
        if width == 0 {
            return;
        }
        let mask = if width == 64 {
            u64::MAX
        } else {
            (1u64 << width) - 1
        };
        self.acc = (self.acc << width) | (val & mask);
        self.bits += width;
        while self.bits >= 8 {
            self.bits -= 8;
            self.buf.push((self.acc >> self.bits) as u8);
        }
    }
    fn finish(mut self) -> Vec<u8> {
        if self.bits > 0 {
            self.buf.push((self.acc << (8 - self.bits)) as u8);
        }
        self.buf
    }
}

fn plain_inverse_bwt(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut base = [0u32; 256];
    let mut acc = 0u32;
    for i in 0..256 {
        base[i] = acc;
        acc += counts[i];
    }
    let mut occ = [0u32; 256];
    let mut lf = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        lf[i] = base[b as usize] + occ[b as usize];
        occ[b as usize] += 1;
    }
    let mut out = vec![0u8; n];
    let mut x = p as usize;
    for k in (0..n).rev() {
        out[k] = l[x];
        x = lf[x] as usize;
    }
    out
}

fn craft_bz2_block(l: &[u8], orig_ptr: u32) -> (Vec<u8>, u64, usize) {
    let n = l.len();
    let rle1 = plain_inverse_bwt(l, orig_ptr);
    let kernels = Kernels::detect();
    let mut unrled = Vec::with_capacity(n);
    let block_crc = mibz2::rle::unrle(&rle1, &mut unrled, kernels)
        .map(|_| mibz2::crc::crc32(&unrled, kernels))
        .unwrap_or(0);

    let mut present = [false; 256];
    for &b in l {
        present[b as usize] = true;
    }
    let mut map = Vec::new();
    for b in 0..=255u8 {
        if present[b as usize] {
            map.push(b);
        }
    }
    if map.len() < 2 {
        for b in 0..=255u8 {
            if !present[b as usize] {
                map.push(b);
                present[b as usize] = true;
                break;
            }
        }
    }
    map.sort_unstable();
    let ninuse = map.len();
    let alpha = ninuse + 2;
    let eob = (ninuse + 1) as u16;

    let mut mtf_list = map.clone();
    let mut syms: Vec<u16> = Vec::with_capacity(n + 8);
    let mut i = 0;
    while i < n {
        let b = l[i];
        let idx = mtf_list.iter().position(|&x| x == b).unwrap();
        if idx == 0 {
            let mut run = 0usize;
            while i < n && l[i] == b {
                run += 1;
                i += 1;
            }
            let mut v = run;
            while v > 0 {
                let rem = (v - 1) & 1;
                syms.push(rem as u16);
                v = (v - 1) >> 1;
            }
        } else {
            mtf_list.remove(idx);
            mtf_list.insert(0, b);
            syms.push((idx + 1) as u16);
            i += 1;
            let mut run = 0usize;
            while i < n && l[i] == b {
                run += 1;
                i += 1;
            }
            let mut v = run;
            while v > 0 {
                let rem = (v - 1) & 1;
                syms.push(rem as u16);
                v = (v - 1) >> 1;
            }
        }
    }
    syms.push(eob);

    let mut code_len = 1u32;
    while (1usize << code_len) < alpha {
        code_len += 1;
    }
    let n_groups = 2usize;
    let n_selectors = syms.len().div_ceil(50);

    let mut bw = BitWriter::new();
    bw.push(32, 0x425a_6839);
    let bit_offset = 32u64;
    bw.push(48, 0x3141_5926_5359);
    bw.push(32, block_crc as u64);
    bw.push(1, 0);
    bw.push(24, orig_ptr as u64);

    let mut group_used = [false; 16];
    for g in 0..16 {
        for sub in 0..16 {
            if present[g * 16 + sub] {
                group_used[g] = true;
            }
        }
        bw.push(1, u64::from(group_used[g]));
    }
    for g in 0..16 {
        if group_used[g] {
            for sub in 0..16 {
                bw.push(1, u64::from(present[g * 16 + sub]));
            }
        }
    }

    bw.push(3, n_groups as u64);
    bw.push(15, n_selectors as u64);
    for _ in 0..n_selectors {
        bw.push(1, 0);
    }
    for _ in 0..n_groups {
        bw.push(5, code_len as u64);
        for _ in 0..alpha {
            bw.push(1, 0);
        }
    }
    for &s in &syms {
        bw.push(code_len, s as u64);
    }
    let bit_end = bw.buf.len() * 8 + bw.bits as usize;
    bw.push(48, 0x1772_4538_5090);
    bw.push(32, block_crc as u64);
    let bytes = bw.finish();
    (bytes, bit_offset, bit_end - bit_offset as usize)
}

fn build_runs(l: &[u8], orig_ptr: u32) -> Runs {
    let (bz2, _bit_off, _bit_len) = craft_bz2_block(l, orig_ptr);
    let mut sc = Scanner::new(SliceSource(&bz2));
    let mut runs = Runs::default();
    match sc.next(Vec::new()).expect("valid bz2") {
        Event::Block { symbols, .. } => {
            assert_eq!(symbols.orig_ptr, orig_ptr);
            let n_decoded = runs::front(&symbols, &mut runs, Kernels::detect()).expect("front ok");
            assert_eq!(n_decoded, l.len());
            runs
        }
        _ => panic!("expected block"),
    }
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
// 3. Links (`Narrow` & `Wide`) and `Elim` Engine (`M0` vs `WormholePreEmit`)
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

fn grow(v: &mut Vec<u32>, need: usize) {
    if need > v.len() {
        let to = v.len().saturating_mul(2).max(need);
        v.reserve_exact(to - v.len());
        v.resize(to, 0);
    }
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

// ============================================================================
// 4. Shared Helpers & Verbatim `origin/master` `emit` + `inline_words`
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

#[derive(Default, Clone, Copy)]
struct StageUs {
    s1: f64,
    s2: f64,
    s3: f64,
    s4: f64,
    s5: f64,
}

// ============================================================================
// 5. Same-Crate Verbatim `origin/master` Clone (`M0CloneCells: InverseBwt`)
// ============================================================================

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
    last_us: StageUs,
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
    fn induced<L: Links, const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        t_s2: Option<Instant>,
    ) -> [usize; 256] {
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
        if let Some(t0) = t_s2 {
            self.last_us.s2 = t0.elapsed().as_nanos() as f64 / 1e3;
        }
        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
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
        if let Some(t0) = t_s3a {
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
        if let Some(t0) = t_s1 {
            self.last_us.s1 = t0.elapsed().as_nanos() as f64 / 1e3;
        }

        let t_s2 = if TIMED { Some(Instant::now()) } else { None };
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
            self.induced::<Narrow, TIMED>(ns, n as u32, &thin, t_s2)
        } else {
            self.induced::<Wide, TIMED>(ns, n as u32, &thin, t_s2)
        };

        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
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
        if let Some(t0) = t_s4 {
            self.last_us.s4 = t0.elapsed().as_nanos() as f64 / 1e3;
        }

        let t_s3b = if TIMED { Some(Instant::now()) } else { None };
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
        );
        if let Some(t0) = t_s3b {
            self.last_us.s3 += t0.elapsed().as_nanos() as f64 / 1e3;
        }

        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
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
        if let Some(t0) = t_s5 {
            self.last_us.s5 = t0.elapsed().as_nanos() as f64 / 1e3;
        }
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

// ============================================================================
// 6. Unified `WormholePreEmitCells: InverseBwt` (`[u32; 3]` StripBandMux +
//    GradedRV + pop_tail + Pohl BidiBottom + Lazy15-DirectInline16)
// ============================================================================

#[derive(Default)]
struct WormholePreEmitKernel {
    stack: Vec<u32>,
    spill: Spill,
    /// Chaitin lifetime-multiplexed `[u32; 3]` buffer:
    /// - `S1 -> S2a`: thick `Strip` = `[from, to, (width << 8) | byte]` (`12 B/strip`)
    /// - `S3 -> S4`: `Band` = `[end, delta, code]` (`12 B/band`)
    sb_buf: Vec<[u32; 3]>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    force_wide_links: bool,
    force_wide_extra: bool,
    last_us: StageUs,
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
            // Theorem 17.1: `<= 14` internal rule nodes & stack depth `<= 15`, `0 B` spelling!
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

    fn induced<L: Links, const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        t_s2: Option<Instant>,
    ) -> [usize; 256] {
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
        // Thick `Strip`s in `sb_buf` are now 100% dead — recycle `sb_buf` for `Band`s!
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
        if let Some(t0) = t_s2 {
            self.last_us.s2 = t0.elapsed().as_nanos() as f64 / 1e3;
        }

        // === STAGE S3 (`BidiBottom` + `Lazy15-DirectInline16` into `sb_buf`, `0.0 us` `S3b`) ===
        let t_s3 = if TIMED { Some(Instant::now()) } else { None };
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
        if let Some(t0) = t_s3 {
            self.last_us.s3 = t0.elapsed().as_nanos() as f64 / 1e3;
        }
        cursors
    }

    fn finish_and_walk<const TIMED: bool>(
        &mut self,
        f: First,
        heads: Heads,
        n: usize,
        out: &mut Vec<u8>,
    ) {
        let t_s2 = if TIMED { Some(Instant::now()) } else { None };
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
            self.induced::<Narrow, TIMED>(ns, n as u32, m, &thin, t_s2)
        } else {
            self.induced::<Wide, TIMED>(ns, n as u32, m, &thin, t_s2)
        };

        // === STAGE S4 (`OutQ16` `Reader::wire` over `sb_buf` `[end, delta, code]`) ===
        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
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
        if let Some(t0) = t_s4 {
            self.last_us.s4 = t0.elapsed().as_nanos() as f64 / 1e3;
        }

        // === STAGE S5 (`origin/master` `emit` 100% verbatim untouched!) ===
        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
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
        if let Some(t0) = t_s5 {
            self.last_us.s5 = t0.elapsed().as_nanos() as f64 / 1e3;
        }
    }

    fn try_run_u16<const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> bool {
        let t_s1 = if TIMED { Some(Instant::now()) } else { None };
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
        if let Some(t0) = t_s1 {
            self.last_us.s1 = t0.elapsed().as_nanos() as f64 / 1e3;
        }
        self.finish_and_walk::<TIMED>(f, heads, n, out);
        true
    }

    /// Diagnostic variant for Obligation #5: unsplit `u32` maximal-run records (`(c << 24) | width24`),
    /// eliminating `maximal()`'s inner coalescing loop at the cost of `2x` record-buffer bandwidth.
    fn run_unsplit_u32(&mut self, max_recs: &[u32], n: usize, p: u32, out: &mut Vec<u8>) {
        let t_s1 = Instant::now();
        out.resize(n + INLINE, 0);
        self.sb_buf.clear();
        let mut pass = PassMux {
            queues: Queues::new(&mut out[..], &mut self.spill, u32::MAX),
            sb_buf: &mut self.sb_buf,
            f: First::default(),
        };
        let mut pos = 0u32;
        for &rec in max_recs {
            let c = (rec >> 24) as u8;
            let width = rec & 0x00FF_FFFF;
            pass.take(c, pos, width, p, LIMIT);
            pos += width;
        }
        let (f, heads) = (pass.f, pass.queues.heads());
        self.last_us.s1 = t_s1.elapsed().as_nanos() as f64 / 1e3;
        self.finish_and_walk::<false>(f, heads, n, out);
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
            if self.open.try_run_u16::<false>(r.records(), n, p, cap, out) {
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
}

// ============================================================================
// 7. Upstream Record Encoders (`CAP=255` `u16` & `Unsplit-u32`) & Gate G1
// ============================================================================

fn encode_cap_u16(l: &[u8], cap: usize) -> Vec<u16> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < l.len() {
        let c = l[i];
        let mut j = i + 1;
        while j < l.len() && l[j] == c {
            j += 1;
        }
        let mut left = j - i;
        while left > 0 {
            let take = left.min(cap);
            out.push(((c as u16) << 8) | (take as u16));
            left -= take;
        }
        i = j;
    }
    out
}

fn encode_unsplit_u32(l: &[u8]) -> Vec<u32> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < l.len() {
        let c = l[i];
        let mut j = i + 1;
        while j < l.len() && l[j] == c {
            j += 1;
        }
        out.push(((c as u32) << 24) | ((j - i) as u32));
        i = j;
    }
    out
}

fn verify_gate_g1_and_edges() -> usize {
    let kernels = Kernels::detect();
    let mut master_forced = ForcedOpenCells::default();
    let mut master_normal = OpenCells::default();
    let mut wh_forced = ForcedWormholePreEmitCells::default();
    let mut wh_normal = WormholePreEmitCells::default();
    let mut out_ref = Vec::new();
    let mut out_cand = Vec::new();
    let mut cases = 0usize;

    // 1. All 5,227 exhaustive binary (n <= 8) and ternary (n <= 5) columns at every p
    for n in 1..=8usize {
        for mask in 0..(1u32 << n) {
            let l: Vec<u8> = (0..n).map(|i| ((mask >> i) & 1) as u8).collect();
            for p in 0..n as u32 {
                let runs = build_runs(&l, p);
                assert!(master_forced.invert_runs(&runs, n, p, &mut out_ref, kernels));
                assert!(wh_forced.invert_runs(&runs, n, p, &mut out_cand, kernels));
                assert_eq!(out_cand, out_ref, "binary mismatch n={n} p={p}");
                cases += 1;
            }
        }
    }
    for n in 1..=5usize {
        let mut total = 1u32;
        for _ in 0..n {
            total *= 3;
        }
        for code in 0..total {
            let mut tmp = code;
            let mut l = Vec::with_capacity(n);
            for _ in 0..n {
                l.push((tmp % 3) as u8);
                tmp /= 3;
            }
            for p in 0..n as u32 {
                let runs = build_runs(&l, p);
                assert!(master_forced.invert_runs(&runs, n, p, &mut out_ref, kernels));
                assert!(wh_forced.invert_runs(&runs, n, p, &mut out_cand, kernels));
                assert_eq!(out_cand, out_ref, "ternary mismatch n={n} p={p}");
                cases += 1;
            }
        }
    }

    // 2. SPIRALS (`widest = 50,000 > CERTIFIED_WIDTH` + `word_len > 1,023` saturating graded handle!)
    let mut spiral_l = Vec::with_capacity(55_000);
    spiral_l.extend(std::iter::repeat_n(b'A', 50_000));
    for i in 0..200u32 {
        spiral_l.push(b'B' + (i % 13) as u8);
        spiral_l.extend(std::iter::repeat_n(b'Z', 17));
    }
    let runs_sp = build_runs(&spiral_l, 25_000);
    assert!(master_normal.invert_runs(&runs_sp, spiral_l.len(), 25_000, &mut out_ref, kernels));
    assert!(wh_normal.invert_runs(&runs_sp, spiral_l.len(), 25_000, &mut out_cand, kernels));
    assert_eq!(out_cand, out_ref, "spiral mismatch");
    cases += 1;

    // Deep spiral forcing word length > 1,023 (`k > 1,023` in `pow`):
    let mut deep_spiral = Vec::with_capacity(70_000);
    deep_spiral.extend(std::iter::repeat_n(b'X', 65_000));
    deep_spiral.extend(std::iter::repeat_n(b'Y', 20));
    deep_spiral.push(b'Q');
    let runs_dsp = build_runs(&deep_spiral, 65_020);
    assert!(master_forced.invert_runs(&runs_dsp, deep_spiral.len(), 65_020, &mut out_ref, kernels));
    assert!(wh_forced.invert_runs(&runs_dsp, deep_spiral.len(), 65_020, &mut out_cand, kernels));
    assert_eq!(out_cand, out_ref, "deep spiral len > 1023 mismatch");
    cases += 1;

    // 3. Forced `Wide` links (`STRIDE = 6`) + forced `wide` `extra` (`self.extra`)
    wh_forced.open.force_wide_links = true;
    wh_forced.open.force_wide_extra = true;
    assert!(wh_forced.invert_runs(&runs_sp, spiral_l.len(), 25_000, &mut out_cand, kernels));
    master_normal.invert_runs(&runs_sp, spiral_l.len(), 25_000, &mut out_ref, kernels);
    assert_eq!(out_cand, out_ref, "forced Wide links + wide extra mismatch");
    wh_forced.open.force_wide_links = false;
    wh_forced.open.force_wide_extra = false;
    cases += 1;

    // 4. `#703` `Cap::dense` ADMITTED case (`r/n = 0.20`, `m < 0.30n`)
    let dense_admit = make_dense_admitted_block();
    let runs_da = build_runs(&dense_admit, 12_345);
    let opened_before = wh_normal.opened;
    assert!(master_normal.invert_runs(&runs_da, dense_admit.len(), 12_345, &mut out_ref, kernels));
    assert!(wh_normal.invert_runs(&runs_da, dense_admit.len(), 12_345, &mut out_cand, kernels));
    assert_eq!(wh_normal.opened, opened_before + 1);
    assert_eq!(out_cand, out_ref, "Cap::dense admitted mismatch");
    cases += 1;

    // 5. `#703` `Cap::dense` REFUSED case (`r/n = 0.22`, all runs thin `w = 4..5` so `m = n > 0.30n`,
    //    verifying clean refusal & byte-exact fallback to `LazyCells`!)
    let mut dense_refuse = Vec::with_capacity(200_000);
    let mut state = 0x1234_5678u32;
    while dense_refuse.len() < 200_000 {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let sym = ((state >> 24) as u8 % 40) + 32;
        let w = ((state >> 16) as usize % 3) + 4; // w in 4..=6 -> r/n ~ 0.20, 100% thin -> m = 200k > 60k cap!
        let take = w.min(200_000 - dense_refuse.len());
        dense_refuse.extend(std::iter::repeat_n(sym, take));
    }
    let runs_dr = build_runs(&dense_refuse, 7_777);
    let capped_before = wh_normal.capped;
    let m_res = master_normal.invert_runs(&runs_dr, dense_refuse.len(), 7_777, &mut out_ref, kernels);
    let w_res = wh_normal.invert_runs(&runs_dr, dense_refuse.len(), 7_777, &mut out_cand, kernels);
    assert_eq!(w_res, m_res);
    assert_eq!(wh_normal.capped, capped_before + 1);
    assert_eq!(out_cand, out_ref, "Cap::dense refusal fallback mismatch");
    cases += 1;

    // 6. High-entropy `r/n > 0.28` refusal straight to `LazyCells`
    let mut hi_ent = Vec::with_capacity(50_000);
    for i in 0..25_000u32 {
        hi_ent.push(((i * 7 + 3) % 61) as u8);
        hi_ent.push(((i * 7 + 3) % 61) as u8);
    }
    let runs_he = build_runs(&hi_ent, 111);
    let m_he = master_normal.invert_runs(&runs_he, hi_ent.len(), 111, &mut out_ref, kernels);
    let w_he = wh_normal.invert_runs(&runs_he, hi_ent.len(), 111, &mut out_cand, kernels);
    assert_eq!(w_he, m_he);
    assert_eq!(out_cand, out_ref);
    cases += 1;

    cases
}

fn make_dense_admitted_block() -> Vec<u8> {
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

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
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
        .unwrap_or("/tmp/cycle19_results.json");

    let cases = verify_gate_g1_and_edges();
    println!("Gate G1 + SPIRALS (len>1023) + WideLinks + WideExtra + #703 Cap::dense Admit/Refuse: PASS ({cases} cases byte-exact)");

    let blocks_meta = [
        ("block-0.L", 408_455u32),
        ("block-2.L", 136_647u32),
        ("block-85.L", 514_044u32),
        ("block-143.L", 160_132u32),
        ("block-210.L", 573_497u32),
        ("block-207.L", 374_972u32),
    ];

    struct BlockInput {
        name: String,
        l: Vec<u8>,
        p: u32,
        runs_127: Runs,
        recs_255: Vec<u16>,
        recs_u32: Vec<u32>,
    }

    let mut inputs = Vec::new();
    for (fname, p) in blocks_meta {
        let path = Path::new(data_dir).join(fname);
        let l = fs::read(&path).expect("read block");
        let runs_127 = build_runs(&l, p);
        let recs_255 = encode_cap_u16(&l, 255);
        let recs_u32 = encode_unsplit_u32(&l);
        inputs.push(BlockInput {
            name: fname.to_string(),
            l,
            p,
            runs_127,
            recs_255,
            recs_u32,
        });
    }
    {
        let l = make_dense_admitted_block();
        let p = 123_456u32;
        let runs_127 = build_runs(&l, p);
        let recs_255 = encode_cap_u16(&l, 255);
        let recs_u32 = encode_unsplit_u32(&l);
        inputs.push(BlockInput {
            name: "dense-703.syn".to_string(),
            l,
            p,
            runs_127,
            recs_255,
            recs_u32,
        });
    }

    // Verify all 7 blocks byte-exact across all arms
    let kernels = Kernels::detect();
    {
        let mut a1 = OpenCells::default();
        let mut m0 = M0CloneCells::default();
        let mut w3 = WormholePreEmitCells::default();
        let mut k_u = WormholePreEmitKernel::default();
        let mut ref_out = Vec::new();
        let mut out_b = Vec::new();
        for inp in &inputs {
            assert!(a1.invert_runs(&inp.runs_127, inp.l.len(), inp.p, &mut ref_out, kernels));
            assert!(m0.invert_runs(&inp.runs_127, inp.l.len(), inp.p, &mut out_b, kernels));
            assert_eq!(out_b, ref_out);
            assert!(w3.invert_runs(&inp.runs_127, inp.l.len(), inp.p, &mut out_b, kernels));
            assert_eq!(out_b, ref_out);
            assert!(k_u.try_run_u16::<false>(&inp.recs_255, inp.l.len(), inp.p, Cap::NONE, &mut out_b));
            assert_eq!(out_b, ref_out);
            k_u.run_unsplit_u32(&inp.recs_u32, inp.l.len(), inp.p, &mut out_b);
            assert_eq!(out_b, ref_out);
        }
    }

    println!("\n=== CYCLE 19A: UPSTREAM `runs::front` `CAP=127` BOUNDARY AUDIT (`OBLIGATION #5`, 21 ROUNDS) ===");
    println!(
        "{:<13} | {:>7} | {:>14} | {:>14} | {:>13} | {:>9} | {:>9} | {:>9}",
        "Block", "r_max", "recs127(+%)", "recs255(+%)", "Buf(u16/u32)", "S1:CAP127", "S1:CAP255", "S1:u32Max"
    );
    println!("{}", "-".repeat(106));

    let mut u0_k = WormholePreEmitKernel::default();
    let mut u1_k = WormholePreEmitKernel::default();
    let mut u2_k = WormholePreEmitKernel::default();
    let mut out_tmp = Vec::with_capacity(900_016);

    struct CapRow {
        r_max: usize,
        r127: usize,
        r255: usize,
        s1_127: f64,
        s1_255: f64,
        s1_u32: f64,
        e2e_127: f64,
        e2e_255: f64,
        e2e_u32: f64,
    }
    let mut cap_rows = Vec::new();

    for inp in &inputs {
        let n = inp.l.len();
        let r_max = inp.recs_u32.len();
        let r127 = inp.runs_127.records().len();
        let r255 = inp.recs_255.len();
        let pct127 = 100.0 * (r127 - r_max) as f64 / r_max as f64;
        let pct255 = 100.0 * (r255 - r_max) as f64 / r_max as f64;

        for _ in 0..3 {
            u0_k.try_run_u16::<true>(inp.runs_127.records(), n, inp.p, Cap::NONE, &mut out_tmp);
            u1_k.try_run_u16::<true>(&inp.recs_255, n, inp.p, Cap::NONE, &mut out_tmp);
            u2_k.run_unsplit_u32(&inp.recs_u32, n, inp.p, &mut out_tmp);
        }
        let rounds = 21usize;
        let (mut s1_0, mut s1_1, mut s1_2) = (Vec::new(), Vec::new(), Vec::new());
        let (mut tot_0, mut tot_1, mut tot_2) = (Vec::new(), Vec::new(), Vec::new());
        for r in 0..rounds {
            for step in 0..3 {
                match (r + step) % 3 {
                    0 => {
                        let t0 = Instant::now();
                        u0_k.try_run_u16::<true>(
                            black_box(inp.runs_127.records()),
                            n,
                            inp.p,
                            Cap::NONE,
                            &mut out_tmp,
                        );
                        tot_0.push(t0.elapsed().as_nanos() as f64 / 1e3);
                        s1_0.push(u0_k.last_us.s1);
                        black_box(out_tmp[0]);
                    }
                    1 => {
                        let t0 = Instant::now();
                        u1_k.try_run_u16::<true>(
                            black_box(&inp.recs_255),
                            n,
                            inp.p,
                            Cap::NONE,
                            &mut out_tmp,
                        );
                        tot_1.push(t0.elapsed().as_nanos() as f64 / 1e3);
                        s1_1.push(u1_k.last_us.s1);
                        black_box(out_tmp[0]);
                    }
                    _ => {
                        let t0 = Instant::now();
                        u2_k.run_unsplit_u32(black_box(&inp.recs_u32), n, inp.p, &mut out_tmp);
                        tot_2.push(t0.elapsed().as_nanos() as f64 / 1e3);
                        s1_2.push(u2_k.last_us.s1);
                        black_box(out_tmp[0]);
                    }
                }
            }
        }
        let (m_s1_0, m_s1_1, m_s1_2) = (median(s1_0), median(s1_1), median(s1_2));
        let (m_t0, m_t1, m_t2) = (median(tot_0), median(tot_1), median(tot_2));
        println!(
            "{:<13} | {:>7} | {:>7} (+{:>4.1}%) | {:>7} (+{:>4.1}%) | {:>4}/{:>4} KiB | {:>7.1}us | {:>7.1}us | {:>7.1}us",
            inp.name,
            r_max,
            r127,
            pct127,
            r255,
            pct255,
            (2 * r127) / 1024,
            (4 * r_max) / 1024,
            m_s1_0,
            m_s1_1,
            m_s1_2,
        );
        cap_rows.push(CapRow {
            r_max,
            r127,
            r255,
            s1_127: m_s1_0,
            s1_255: m_s1_1,
            s1_u32: m_s1_2,
            e2e_127: m_t0,
            e2e_255: m_t1,
            e2e_u32: m_t2,
        });
    }

    println!("\n=== CYCLE 19B: UNIFIED `WormholePreEmitCells` 5-STAGE ATTRIBUTION & WORKING-SET CENSUS ===");
    println!(
        "{:<13} | {:>15} | {:>27} | {:>27} | {:>9}",
        "Block", "WS(M0 -> W3 KB)", "M0 (S1/S2/S3/S4 = Pre us)", "W3 (S1/S2/S3/S4 = Pre us)", "Pre W3/M0"
    );
    println!("{}", "-".repeat(104));

    let mut m0_timed = M0Kernel::default();
    let mut w3_timed = WormholePreEmitKernel::default();
    struct StageSummary {
        ws_m0: usize,
        ws_w3: usize,
        m0_s: StageUs,
        w3_s: StageUs,
    }
    let mut stage_rows = Vec::new();

    for inp in &inputs {
        let n = inp.l.len();
        let recs = inp.runs_127.records();
        for _ in 0..3 {
            m0_timed.try_run::<true>(recs, n, inp.p, Cap::NONE, &mut out_tmp);
            w3_timed.try_run_u16::<true>(recs, n, inp.p, Cap::NONE, &mut out_tmp);
        }
        let (ws_m0, ws_w3) = {
            let mut m0_ws = M0Kernel::default();
            let mut w3_ws = WormholePreEmitKernel::default();
            let mut out_ws = Vec::with_capacity(n + INLINE);
            m0_ws.try_run::<false>(recs, n, inp.p, Cap::NONE, &mut out_ws);
            w3_ws.try_run_u16::<false>(recs, n, inp.p, Cap::NONE, &mut out_ws);
            (m0_ws.ws_bytes(n + INLINE), w3_ws.ws_bytes(n + INLINE))
        };
        let rounds = 15usize;
        let (mut m_s1, mut m_s2, mut m_s3, mut m_s4, mut m_s5) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let (mut w_s1, mut w_s2, mut w_s3, mut w_s4, mut w_s5) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for r in 0..rounds {
            if r % 2 == 0 {
                m0_timed.try_run::<true>(black_box(recs), n, inp.p, Cap::NONE, &mut out_tmp);
                w3_timed.try_run_u16::<true>(black_box(recs), n, inp.p, Cap::NONE, &mut out_tmp);
            } else {
                w3_timed.try_run_u16::<true>(black_box(recs), n, inp.p, Cap::NONE, &mut out_tmp);
                m0_timed.try_run::<true>(black_box(recs), n, inp.p, Cap::NONE, &mut out_tmp);
            }
            m_s1.push(m0_timed.last_us.s1);
            m_s2.push(m0_timed.last_us.s2);
            m_s3.push(m0_timed.last_us.s3);
            m_s4.push(m0_timed.last_us.s4);
            m_s5.push(m0_timed.last_us.s5);
            w_s1.push(w3_timed.last_us.s1);
            w_s2.push(w3_timed.last_us.s2);
            w_s3.push(w3_timed.last_us.s3);
            w_s4.push(w3_timed.last_us.s4);
            w_s5.push(w3_timed.last_us.s5);
        }
        let ms = StageUs {
            s1: median(m_s1),
            s2: median(m_s2),
            s3: median(m_s3),
            s4: median(m_s4),
            s5: median(m_s5),
        };
        let ws = StageUs {
            s1: median(w_s1),
            s2: median(w_s2),
            s3: median(w_s3),
            s4: median(w_s4),
            s5: median(w_s5),
        };
        let pre_m = ms.s1 + ms.s2 + ms.s3 + ms.s4;
        let pre_w = ws.s1 + ws.s2 + ws.s3 + ws.s4;
        println!(
            "{:<13} | {:>4} -> {:>4} KB | {:>3.0}/{:>3.0}/{:>3.0}/{:>3.0} = {:>5.1}us | {:>3.0}/{:>3.0}/{:>3.0}/{:>3.0} = {:>5.1}us | {:>8.3}x",
            inp.name,
            ws_m0 / 1024,
            ws_w3 / 1024,
            ms.s1,
            ms.s2,
            ms.s3,
            ms.s4,
            pre_m,
            ws.s1,
            ws.s2,
            ws.s3,
            ws.s4,
            pre_w,
            pre_w / pre_m,
        );
        stage_rows.push(StageSummary {
            ws_m0,
            ws_w3,
            m0_s: ms,
            w3_s: ws,
        });
    }

    println!("\n=== CYCLE 19C: HEADLINE `InverseBwt::invert_runs` A/B + A/A + M0 CONFRONTATION (25 ROUNDS) ===");
    println!(
        "{:<13} | {:>9} | {:>9} | {:>7} | {:>9} | {:>9} | {:>8} | {:>8}",
        "Block", "A1:Master", "A2:Ctrl", "A2/A1", "M0:Clone", "W3:PreEmt", "W3 / M0", "W3 / A1"
    );
    println!("{}", "-".repeat(92));

    let mut a1 = OpenCells::default();
    let mut a2 = OpenCells::default();
    let mut m0 = M0CloneCells::default();
    let mut w3 = WormholePreEmitCells::default();

    let mut json_entries = Vec::new();

    for (idx, inp) in inputs.iter().enumerate() {
        let n = inp.l.len();
        for _ in 0..4 {
            a1.invert_runs(&inp.runs_127, n, inp.p, &mut out_tmp, kernels);
            a2.invert_runs(&inp.runs_127, n, inp.p, &mut out_tmp, kernels);
            m0.invert_runs(&inp.runs_127, n, inp.p, &mut out_tmp, kernels);
            w3.invert_runs(&inp.runs_127, n, inp.p, &mut out_tmp, kernels);
        }
        let rounds = 25usize;
        let (mut t_a1, mut t_a2, mut t_m0, mut t_w3) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for r in 0..rounds {
            for step in 0..4 {
                match (r + step) % 4 {
                    0 => {
                        let t0 = Instant::now();
                        a1.invert_runs(black_box(&inp.runs_127), n, inp.p, &mut out_tmp, kernels);
                        t_a1.push(t0.elapsed().as_nanos() as f64 / 1e3);
                        black_box(out_tmp[0]);
                    }
                    1 => {
                        let t0 = Instant::now();
                        a2.invert_runs(black_box(&inp.runs_127), n, inp.p, &mut out_tmp, kernels);
                        t_a2.push(t0.elapsed().as_nanos() as f64 / 1e3);
                        black_box(out_tmp[0]);
                    }
                    2 => {
                        let t0 = Instant::now();
                        m0.invert_runs(black_box(&inp.runs_127), n, inp.p, &mut out_tmp, kernels);
                        t_m0.push(t0.elapsed().as_nanos() as f64 / 1e3);
                        black_box(out_tmp[0]);
                    }
                    _ => {
                        let t0 = Instant::now();
                        w3.invert_runs(black_box(&inp.runs_127), n, inp.p, &mut out_tmp, kernels);
                        t_w3.push(t0.elapsed().as_nanos() as f64 / 1e3);
                        black_box(out_tmp[0]);
                    }
                }
            }
        }
        let (ma1, ma2, mm0, mw3) = (median(t_a1), median(t_a2), median(t_m0), median(t_w3));
        println!(
            "{:<13} | {:>7.1}us | {:>7.1}us | {:>6.3}x | {:>7.1}us | {:>7.1}us | {:>7.3}x | {:>7.3}x",
            inp.name,
            ma1,
            ma2,
            ma2 / ma1,
            mm0,
            mw3,
            mw3 / mm0,
            mw3 / ma1,
        );
        let cr = &cap_rows[idx];
        let sr = &stage_rows[idx];
        json_entries.push(format!(
            "    {{\n      \"block\": \"{}\",\n      \"upstream_cap\": {{\"r_max\": {}, \"recs_127\": {}, \"recs_255\": {}, \"s1_us\": {{\"cap127\": {:.2}, \"cap255\": {:.2}, \"unsplit_u32\": {:.2}}}, \"e2e_us\": {{\"cap127\": {:.2}, \"cap255\": {:.2}, \"unsplit_u32\": {:.2}}}}},\n      \"ws_bytes\": {{\"m0\": {}, \"w3\": {}}},\n      \"stages_m0_us\": {{\"s1\": {:.2}, \"s2\": {:.2}, \"s3\": {:.2}, \"s4\": {:.2}, \"s5\": {:.2}}},\n      \"stages_w3_us\": {{\"s1\": {:.2}, \"s2\": {:.2}, \"s3\": {:.2}, \"s4\": {:.2}, \"s5\": {:.2}}},\n      \"e2e_us\": {{\"a1_master\": {:.2}, \"a2_ctrl\": {:.2}, \"m0_clone\": {:.2}, \"w3_pre_emit\": {:.2}}},\n      \"ratios\": {{\"a2_a1\": {:.4}, \"w3_m0\": {:.4}, \"w3_a1\": {:.4}}}\n    }}",
            inp.name,
            cr.r_max, cr.r127, cr.r255, cr.s1_127, cr.s1_255, cr.s1_u32, cr.e2e_127, cr.e2e_255, cr.e2e_u32,
            sr.ws_m0, sr.ws_w3,
            sr.m0_s.s1, sr.m0_s.s2, sr.m0_s.s3, sr.m0_s.s4, sr.m0_s.s5,
            sr.w3_s.s1, sr.w3_s.s2, sr.w3_s.s3, sr.w3_s.s4, sr.w3_s.s5,
            ma1, ma2, mm0, mw3,
            ma2 / ma1, mw3 / mm0, mw3 / ma1
        ));
    }

    let json = format!(
        "{{\n  \"gate_g1_cases\": {cases},\n  \"blocks\": [\n{}\n  ]\n}}\n",
        json_entries.join(",\n")
    );
    fs::write(json_out, json).expect("write json");
}
