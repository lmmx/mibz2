#![forbid(unsafe_code)]
//! Census for the MTF-into-construction fold (A5 + A6a):
//! characterises how often the 127-length cap splits runs on real blocks, what A6a
//! (`OpenCells` first pass) recomputes, and what folding A5 (`runs::front`) directly into
//! A6a (`Pass::take` + `OutQ16` + thick strips) saves.
//!
//! Smooth-zone characterisation only: zero changes to `src/`, zero `unsafe`.
//!
//! Usage:
//!   mtf_fold_census stats FILE.bz2 [--forced]
//!   mtf_fold_census bench FILE.bz2 [--forced] [--repeats N]

use mibz2::block::Symbols;
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, ErrorKind, Kernels};
use std::hint::black_box;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Production constants (mirroring src/runs.rs, src/open.rs, src/open/outq.rs,
// src/cells_lazy.rs verbatim)
// ---------------------------------------------------------------------------

const CAP: u32 = 127;
const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;
const LIMIT: u32 = 16;
const INLINE: usize = 16;

const LAZY_DENSITY: (usize, usize) = (3, 20);
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

const CHUNK: usize = 512;
const CHUNK_BYTES: usize = 2 * CHUNK;
const NONE_CHUNK: u32 = u32::MAX;
const EPOCH: u32 = 1 << 16;

// i9-10900K wikidata x4 anchors:
// - character-stage1/report.txt (k=1): worker busy 1280.1 ms, A5 132.4 ms, A6a 200.3 ms (sum 332.7 ms)
// - character-stage1/report.txt (k=8): worker busy 184.7 ms/w, A5 19.3 ms/w, A6a 28.2 ms/w (sum 47.5 ms/w)
// - ideal-parts/results/full (k=1): wall 1306 ms, M alone room 7.5% (~98 ms), A6a 200.3 ms (sum ~298.3 ms)
const I9_WALL_1W_MS: f64 = 1280.1;
const I9_A5_STAGE1_1W_MS: f64 = 132.4;
const I9_A6A_STAGE1_1W_MS: f64 = 200.3;
const I9_M_IDEAL_1W_MS: f64 = 98.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

fn lazy_counts_admit(records: usize, n: usize) -> bool {
    records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n
}

fn open_cap(records: usize, n: usize, p: u32, forced: bool) -> Option<Cap> {
    if n == 0 || n > 900_000 || p as usize >= n {
        None
    } else if forced || lazy_counts_admit(records, n) {
        Some(Cap::NONE)
    } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
        Some(Cap::dense(n))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Exact mirror of OutQ16 (src/open/outq.rs)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Spill {
    next: Vec<u32>,
    ovf: Vec<[u8; CHUNK_BYTES]>,
    cuts: Vec<[u32; 256]>,
}

struct Queues<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    chunks: u32,
    heads: [u32; 256],
    tails: [u32; 256],
    fill: [u32; 256],
    limit: u32,
    over: bool,
    // Stats counters (only updated when TRACK_STATS is true)
    fast_pushes: u64,
    slow_pushes: u64,
    w1_pushes: u64,
}

struct Heads {
    heads: [u32; 256],
    in_out: u32,
}

impl<'a> Queues<'a> {
    fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
        spill.ovf.clear();
        spill.cuts.clear();
        Queues {
            in_out: (out.len() / CHUNK_BYTES) as u32,
            out,
            spill,
            chunks: 0,
            heads: [NONE_CHUNK; 256],
            tails: [NONE_CHUNK; 256],
            fill: [CHUNK as u32; 256],
            limit,
            over: false,
            fast_pushes: 0,
            slow_pushes: 0,
            w1_pushes: 0,
        }
    }

    fn heads(&self) -> Heads {
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }

    #[inline(always)]
    fn push<const W1_FAST: bool, const TRACK: bool>(
        &mut self,
        a: usize,
        m: u32,
        w: u32,
        thin: &[u32; 256],
    ) {
        if TRACK && w == 1 {
            self.w1_pushes += 1;
        }
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if W1_FAST && w == 1 && f < CHUNK && lo + 1 < EPOCH && tail < self.in_out {
            if TRACK {
                self.fast_pushes += 1;
            }
            let at = tail as usize * CHUNK_BYTES + 2 * f;
            self.out[at..at + 2].copy_from_slice(&(lo as u16).to_le_bytes());
            self.fill[a] = (f as u32) + 1;
            return;
        }
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            if TRACK {
                self.fast_pushes += 1;
            }
            let at = tail as usize * CHUNK_BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else {
            if TRACK {
                self.slow_pushes += 1;
            }
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
                self.take_chunk(a);
            }
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
            let chunk = self.chunk_mut(self.tails[a]);
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

    fn take_chunk(&mut self, a: usize) {
        let id = self.chunks;
        self.chunks += 1;
        if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
            self.spill.ovf.push([0; CHUNK_BYTES]);
        }
        if self.spill.next.len() <= id as usize {
            self.spill.next.resize(id as usize + 1, NONE_CHUNK);
        }
        self.spill.next[id as usize] = NONE_CHUNK;
        if self.heads[a] == NONE_CHUNK {
            self.heads[a] = id;
        } else {
            self.spill.next[self.tails[a] as usize] = id;
        }
        self.tails[a] = id;
        self.fill[a] = 0;
    }

    fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
        if id < self.in_out {
            let at = id as usize * CHUNK_BYTES;
            &mut self.out[at..at + CHUNK_BYTES]
        } else {
            &mut self.spill.ovf[(id - self.in_out) as usize][..]
        }
    }
}

/// Exact reader for OutQ16 (used strictly for 100% state-equality verification).
fn drain_queue(out: &[u8], spill: &Spill, heads: &Heads, a: usize, count: u32) -> Vec<u32> {
    let mut res = Vec::with_capacity(count as usize);
    let mut id = heads.heads[a];
    let mut at = 0usize;
    let mut read = 0u32;
    let mut epoch = 0u32;
    let mut cut = spill.cuts.first().map_or(u32::MAX, |c| c[a]);
    let mut left = count as usize;
    while left > 0 {
        while read == cut {
            epoch += 1;
            cut = spill.cuts.get(epoch as usize).map_or(u32::MAX, |c| c[a]);
        }
        if at == CHUNK {
            id = spill.next[id as usize];
            at = 0;
        }
        let take = left.min(CHUNK - at).min((cut - read) as usize);
        let chunk: &[u8] = if id < heads.in_out {
            let off = id as usize * CHUNK_BYTES;
            &out[off..off + CHUNK_BYTES]
        } else {
            &spill.ovf[(id - heads.in_out) as usize][..]
        };
        let hi = epoch << 16;
        for s in chunk[2 * at..2 * (at + take)].chunks_exact(2) {
            res.push(hi | u32::from(u16::from_le_bytes([s[0], s[1]])));
        }
        at += take;
        read += take as u32;
        left -= take;
    }
    res
}

// ---------------------------------------------------------------------------
// Exact mirror of Open A6a (src/open.rs:119-266, 350-374)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    runs: usize,
}

impl Default for First {
    fn default() -> Self {
        First {
            thick_rows: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            runs: 0,
        }
    }
}

#[inline(always)]
fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
    let mut pos = 0;
    let mut i = 0;
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
fn split_run(
    c: u8,
    pos: u32,
    width: u32,
    p: u32,
    limit: u32,
    mut visit: impl FnMut(u8, u32, u32, bool),
) {
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

struct Pass<'a> {
    queues: Queues<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    f: First,
}

impl Pass<'_> {
    #[inline(always)]
    fn take<const QUEUE: bool, const W1_FAST: bool, const TRACK: bool>(
        &mut self,
        c: u8,
        pos: u32,
        width: u32,
        p: u32,
        limit: u32,
    ) {
        self.f.runs += 1;
        if width <= limit {
            self.piece::<QUEUE, W1_FAST, TRACK>(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece::<QUEUE, W1_FAST, TRACK>(c, pos, width, false, p);
        } else {
            self.take_marked::<QUEUE, W1_FAST, TRACK>(c, pos, width, p, limit);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked<const QUEUE: bool, const W1_FAST: bool, const TRACK: bool>(
        &mut self,
        c: u8,
        pos: u32,
        width: u32,
        p: u32,
        limit: u32,
    ) {
        split_run(c, pos, width, p, limit, |c, x, w, keep| {
            self.piece::<QUEUE, W1_FAST, TRACK>(c, x, w, keep, p)
        });
    }

    #[inline(always)]
    fn piece<const QUEUE: bool, const W1_FAST: bool, const TRACK: bool>(
        &mut self,
        c: u8,
        x: u32,
        w: u32,
        keep: bool,
        p: u32,
    ) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
            if QUEUE {
                self.queues.push::<W1_FAST, TRACK>(a, f.m, w, &f.thin);
            }
            f.thin[a] += w;
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
                self.edges[at..at + 3].copy_from_slice(&[
                    x,
                    f.thin[a] + f.thick_rows[a],
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
struct A6aScratch {
    spill: Spill,
    edges: Vec<u32>,
    out: Vec<u8>,
}

struct A6aResult {
    n: usize,
    f: First,
    ns: usize,
    over: bool,
    heads: Heads,
    chunks: u32,
    fast_pushes: u64,
    slow_pushes: u64,
    w1_pushes: u64,
}

fn run_a6a_from_records<const QUEUE: bool, const W1_FAST: bool, const TRACK: bool>(
    records: &[u16],
    n: usize,
    p: u32,
    cap: Cap,
    sc: &mut A6aScratch,
) -> A6aResult {
    sc.out.resize(n + INLINE, 0);
    let mut pass = Pass {
        queues: Queues::new(&mut sc.out[..], &mut sc.spill, cap.m),
        edges: &mut sc.edges,
        ns: 0,
        cap_strips: cap.strips,
        f: First::default(),
    };
    maximal(records, |c, pos, width| {
        pass.take::<QUEUE, W1_FAST, TRACK>(c, pos, width, p, LIMIT);
    });
    let heads = pass.queues.heads();
    let over = pass.queues.over;
    let chunks = pass.queues.chunks;
    let fast_pushes = pass.queues.fast_pushes;
    let slow_pushes = pass.queues.slow_pushes;
    let w1_pushes = pass.queues.w1_pushes;
    A6aResult {
        n,
        f: pass.f,
        ns: pass.ns,
        over,
        heads,
        chunks,
        fast_pushes,
        slow_pushes,
        w1_pushes,
    }
}

fn run_a6a_from_maximal<const QUEUE: bool, const W1_FAST: bool>(
    max_runs: &[(u8, u32)],
    n: usize,
    p: u32,
    cap: Cap,
    sc: &mut A6aScratch,
) -> A6aResult {
    sc.out.resize(n + INLINE, 0);
    let mut pass = Pass {
        queues: Queues::new(&mut sc.out[..], &mut sc.spill, cap.m),
        edges: &mut sc.edges,
        ns: 0,
        cap_strips: cap.strips,
        f: First::default(),
    };
    let mut pos = 0u32;
    for &(c, width) in max_runs {
        pass.take::<QUEUE, W1_FAST, false>(c, pos, width, p, LIMIT);
        pos += width;
    }
    let heads = pass.queues.heads();
    let over = pass.queues.over;
    let chunks = pass.queues.chunks;
    A6aResult {
        n,
        f: pass.f,
        ns: pass.ns,
        over,
        heads,
        chunks,
        fast_pushes: 0,
        slow_pushes: 0,
        w1_pushes: 0,
    }
}

// ---------------------------------------------------------------------------
// Safe A5 mirrors & direct folded A5+A6a prototypes (zero unsafe)
// ---------------------------------------------------------------------------

struct RunsMirror {
    runs: Vec<u16>,
    counts: [u32; 256],
    per: [u32; 256],
    lit: Vec<u64>,
    one: Vec<u64>,
}

impl Default for RunsMirror {
    fn default() -> Self {
        RunsMirror {
            runs: Vec::new(),
            counts: [0; 256],
            per: [0; 256],
            lit: Vec::new(),
            one: Vec::new(),
        }
    }
}

/// Exact safe mirror of `src/runs.rs::front_scalar` (writes cap-127 `runs`, `counts`, `per`).
fn front_scalar_mirror(b: &Symbols, r: &mut RunsMirror) -> Result<usize, ErrorKind> {
    #[inline(always)]
    fn close(open: (u8, u32), runs: &mut Vec<u16>, per: &mut [u32; 256]) {
        let (c, mut left) = open;
        while left > 0 {
            let len = left.min(CAP);
            runs.push((u16::from(c) << 8) | len as u16);
            per[usize::from(c)] += 1;
            left -= len;
        }
    }
    r.runs.clear();
    r.counts = [0; 256];
    r.per = [0; 256];
    let mut list: [u8; 256] = std::array::from_fn(|i| i as u8);
    let (mut run, mut weight) = (0u32, 1u32);
    let mut n = 0usize;
    let mut open = (0u8, 0u32);
    for &s in &b.syms {
        if s <= 1 {
            if weight >= MAX_RUN_WEIGHT {
                return Err(ErrorKind::RunTooLong);
            }
            run += (u32::from(s) + 1) * weight;
            weight <<= 1;
            continue;
        }
        if run != 0 {
            if n + run as usize > b.max_len {
                return Err(ErrorKind::BlockTooLong);
            }
            let c = b.seq_to_unseq[usize::from(list[0])];
            open = (c, open.1 + run);
            r.counts[usize::from(c)] += run;
            n += run as usize;
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        if n >= b.max_len {
            return Err(ErrorKind::BlockTooLong);
        }
        let nn = (usize::from(s - 1)).min(255);
        let v = list[nn];
        list.copy_within(0..nn, 1);
        list[0] = v;
        let c = b.seq_to_unseq[usize::from(v)];
        close(open, &mut r.runs, &mut r.per);
        open = (c, 1);
        r.counts[usize::from(c)] += 1;
        n += 1;
    }
    close(open, &mut r.runs, &mut r.per);
    Ok(n)
}

/// Direct folded scalar A5 + A6a: emits maximal runs straight into `Pass::take`
/// with no `runs: Vec<u16>`, no cap-127 splitting, no `counts`/`per`, no `maximal()` rescan.
fn folded_scalar_a5_a6a<const W1_FAST: bool>(
    b: &Symbols,
    p: u32,
    cap: Cap,
    sc: &mut A6aScratch,
) -> Result<A6aResult, ErrorKind> {
    sc.out.resize(b.max_len + INLINE, 0);
    let mut pass = Pass {
        queues: Queues::new(&mut sc.out[..], &mut sc.spill, cap.m),
        edges: &mut sc.edges,
        ns: 0,
        cap_strips: cap.strips,
        f: First::default(),
    };
    let mut list: [u8; 256] = std::array::from_fn(|i| i as u8);
    let (mut run, mut weight) = (0u32, 1u32);
    let mut n = 0usize;
    let mut pos = 0u32;
    let mut open = (0u8, 0u32);
    for &s in &b.syms {
        if s <= 1 {
            if weight >= MAX_RUN_WEIGHT {
                return Err(ErrorKind::RunTooLong);
            }
            run += (u32::from(s) + 1) * weight;
            weight <<= 1;
            continue;
        }
        if run != 0 {
            if n + run as usize > b.max_len {
                return Err(ErrorKind::BlockTooLong);
            }
            let c = b.seq_to_unseq[usize::from(list[0])];
            open = (c, open.1 + run);
            n += run as usize;
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        if n >= b.max_len {
            return Err(ErrorKind::BlockTooLong);
        }
        let nn = (usize::from(s - 1)).min(255);
        let v = list[nn];
        list.copy_within(0..nn, 1);
        list[0] = v;
        let c = b.seq_to_unseq[usize::from(v)];
        if open.1 != 0 {
            pass.take::<true, W1_FAST, false>(open.0, pos, open.1, p, LIMIT);
            pos += open.1;
        }
        open = (c, 1);
        n += 1;
    }
    if open.1 != 0 {
        pass.take::<true, W1_FAST, false>(open.0, pos, open.1, p, LIMIT);
    }
    let heads = pass.queues.heads();
    let over = pass.queues.over;
    let chunks = pass.queues.chunks;
    Ok(A6aResult {
        n,
        f: pass.f,
        ns: pass.ns,
        over,
        heads,
        chunks,
        fast_pushes: 0,
        slow_pushes: 0,
        w1_pushes: 0,
    })
}

// Masks for moving byte index `k` (0..16) to byte 0 of a little-endian `u128`.
// LOW_MASK[k] keeps bytes 0..k (shifted up by 8); HI_MASK[k] keeps bytes (k+1)..16 unchanged.
const LOW_MASK: [u128; 16] = {
    let mut t = [0u128; 16];
    let mut k = 0;
    while k < 16 {
        t[k] = if k == 0 { 0 } else { (1u128 << (8 * k)) - 1 };
        k += 1;
    }
    t
};

const HI_MASK: [u128; 16] = {
    let mut t = [0u128; 16];
    let mut k = 0;
    while k < 16 {
        t[k] = if k == 15 {
            0
        } else {
            !((1u128 << (8 * (k + 1))) - 1)
        };
        k += 1;
    }
    t
};

/// Safe 2-level MTF list matching `front_avx2`'s algorithm:
/// front 16 bytes in `f0: u128`, next 16 in `f1: u128`, remaining 224 in `tail`.
struct MtfU128 {
    f0: u128,
    f1: u128,
    tail: [u8; 224],
}

impl MtfU128 {
    #[inline(always)]
    fn new(s2u: &[u8; 256]) -> Self {
        let mut a = [0u8; 16];
        let mut b = [0u8; 16];
        a.copy_from_slice(&s2u[..16]);
        b.copy_from_slice(&s2u[16..32]);
        let mut tail = [0u8; 224];
        tail.copy_from_slice(&s2u[32..]);
        MtfU128 {
            f0: u128::from_le_bytes(a),
            f1: u128::from_le_bytes(b),
            tail,
        }
    }

    #[inline(always)]
    fn move_to_front(&mut self, s: usize) -> u8 {
        let nn = s - 1;
        if nn < 16 {
            let v = (self.f0 >> (8 * nn)) & 0xff;
            self.f0 = (self.f0 & HI_MASK[nn]) | ((self.f0 & LOW_MASK[nn]) << 8) | v;
            v as u8
        } else if nn < 32 {
            let k = nn - 16;
            let v = (self.f1 >> (8 * k)) & 0xff;
            let carry = self.f0 >> 120;
            self.f0 = (self.f0 << 8) | v;
            self.f1 = (self.f1 & HI_MASK[k]) | ((self.f1 & LOW_MASK[k]) << 8) | carry;
            v as u8
        } else {
            self.move_tail(nn.min(255) - 32)
        }
    }

    #[cold]
    #[inline(never)]
    fn move_tail(&mut self, t: usize) -> u8 {
        let v = self.tail[t];
        let carry1 = (self.f1 >> 120) as u8;
        let carry0 = self.f0 >> 120;
        self.tail.copy_within(0..t, 1);
        self.tail[0] = carry1;
        self.f1 = (self.f1 << 8) | carry0;
        self.f0 = (self.f0 << 8) | u128::from(v);
        v
    }
}

/// Pass 1 of bitmap MTF: build `lit` (not RUNA/RUNB, below EOB) and `one` (not RUNA, up to EOB).
/// Returns `Some(end)` where `end` is EOB's index, or `None` if no EOB found.
#[inline(always)]
fn build_bitmaps_safe(b: &Symbols, lit: &mut Vec<u64>, one: &mut Vec<u64>) -> Option<usize> {
    let m = b.syms.len();
    let words = m / 64 + 3;
    lit.clear();
    lit.resize(words, 0);
    one.clear();
    one.resize(words, 0);
    let eob = b.eob;
    let mut end = usize::MAX;
    let mut wi = 0usize;
    let mut chunks = b.syms.chunks_exact(64);
    for chunk in &mut chunks {
        let mut l = 0u64;
        let mut o = 0u64;
        for (bit, &s) in chunk.iter().enumerate() {
            let bit_mask = 1u64 << bit;
            if s != 0 {
                o |= bit_mask;
            }
            if s > 1 {
                if s == eob {
                    end = wi * 64 + bit;
                    break;
                }
                l |= bit_mask;
            }
        }
        lit[wi] = l;
        one[wi] = o;
        wi += 1;
        if end != usize::MAX {
            return Some(end);
        }
    }
    let rem = chunks.remainder();
    let mut l = 0u64;
    let mut o = 0u64;
    for (bit, &s) in rem.iter().enumerate() {
        let bit_mask = 1u64 << bit;
        if s != 0 {
            o |= bit_mask;
        }
        if s > 1 {
            if s == eob {
                end = wi * 64 + bit;
                break;
            }
            l |= bit_mask;
        }
    }
    lit[wi] = l;
    one[wi] = o;
    if end == usize::MAX {
        None
    } else {
        Some(end)
    }
}

#[inline(always)]
fn bzhi_safe(val: u64, nbits: u32) -> u64 {
    if nbits >= 64 {
        val
    } else {
        val & ((1u64 << nbits) - 1)
    }
}

#[inline(always)]
fn extract_run_cross(one: &[u64], q: usize, q2: usize) -> u64 {
    let from = q + 1;
    let word = from >> 6;
    let shift = from & 63;
    let bits = if shift == 0 {
        one[word]
    } else {
        (one[word] >> shift) | (one[word + 1] << (64 - shift))
    };
    bzhi_safe(bits, (q2 - q) as u32)
}

/// Safe bitmap + 2-level `u128` MTF writing cap-127 `runs`, `counts`, `per`
/// (exact safe mirror of `front_avx2`'s algorithm and outputs).
fn front_bitmap_u128(b: &Symbols, f: &mut RunsMirror) -> Option<usize> {
    let m = b.syms.len();
    if b.max_len >= 1 << 31 || m >= 1 << 24 {
        return None;
    }
    let end = build_bitmaps_safe(b, &mut f.lit, &mut f.one)?;
    let mut mtf = MtfU128::new(&b.seq_to_unseq);
    let mut slack = b.max_len / CAP as usize + 1;
    f.runs.clear();
    f.runs.reserve(m + slack + 2);
    let mut cnt = [0u64; 256];
    let mut big = 0u64;

    macro_rules! push_long {
        ($c:expr, $total:expr) => {{
            let (c, total): (u8, u64) = ($c, $total);
            let recs = total.div_ceil(u64::from(CAP));
            let more = recs.saturating_sub(1) as usize;
            big += total;
            if more > slack || big > b.max_len as u64 {
                return None;
            }
            slack -= more;
            cnt[usize::from(c)] += total | (recs << 32);
            let mut left = total;
            while left > 0 {
                let len = left.min(u64::from(CAP));
                f.runs.push((u16::from(c) << 8) | len as u16);
                left -= len;
            }
        }};
    }

    macro_rules! emit_rec {
        ($c:expr, $total:expr, $span:expr) => {{
            let (c, total): (u8, u64) = ($c, $total);
            if total > u64::from(CAP) {
                if $span > 22 {
                    return None;
                }
                push_long!(c, total);
            } else {
                f.runs.push((u16::from(c) << 8) | total as u16);
                cnt[usize::from(c)] += total | (1u64 << 32);
            }
        }};
    }

    let last = end / 64;
    let (mut wi, mut w) = (0usize, f.lit[0]);
    macro_rules! seek {
        () => {{
            loop {
                if w != 0 {
                    let t = w.trailing_zeros() as usize;
                    w &= w - 1;
                    break Some(t);
                }
                if wi == last {
                    break None;
                }
                wi += 1;
                w = f.lit[wi];
            }
        }};
    }

    let first = seek!();
    let q0 = first.map_or(end, |t| wi * 64 + t);
    if q0 > 21 {
        return None;
    }
    let init_len = bzhi_safe(f.one[0], q0 as u32 + 1) - 1;
    if init_len > 0 {
        push_long!(b.seq_to_unseq[0], init_len);
    }

    if let Some(mut tz) = first {
        let mut after = f.one[wi] >> 1;
        let mut base = wi * 64;
        loop {
            let s = usize::from(b.syms[base + tz]);
            let c = mtf.move_to_front(s);
            if w != 0 {
                let t2 = w.trailing_zeros() as usize;
                w &= w - 1;
                let total = bzhi_safe(after, t2 as u32) >> tz;
                emit_rec!(c, total, t2 - tz);
                tz = t2;
            } else {
                let q = base + tz;
                let next = seek!();
                let q2 = next.map_or(end, |t| wi * 64 + t);
                if q2 - q > 22 {
                    return None;
                }
                let total = extract_run_cross(&f.one, q, q2);
                emit_rec!(c, total, q2 - q);
                match next {
                    Some(t) => {
                        tz = t;
                        after = f.one[wi] >> 1;
                        base = wi * 64;
                    }
                    None => break,
                }
            }
        }
    }

    let n: u64 = cnt.iter().map(|&x| x & 0xffff_ffff).sum();
    if n > b.max_len as u64 {
        return None;
    }
    for (c, &x) in cnt.iter().enumerate() {
        f.counts[c] = x as u32;
        f.per[c] = (x >> 32) as u32;
    }
    Some(n as usize)
}

/// Direct folded bitmap + 2-level `u128` MTF into A6a (`Pass::take` + `OutQ16` + `edges`):
/// zero intermediate `runs: Vec<u16>`, zero cap-127 splitting, zero `counts`/`per`, zero `maximal()` rescan!
fn folded_bitmap_u128_a5_a6a<const W1_FAST: bool>(
    b: &Symbols,
    p: u32,
    cap: Cap,
    lit: &mut Vec<u64>,
    one: &mut Vec<u64>,
    sc: &mut A6aScratch,
) -> Option<A6aResult> {
    let m = b.syms.len();
    if b.max_len >= 1 << 31 || m >= 1 << 24 {
        return None;
    }
    let end = build_bitmaps_safe(b, lit, one)?;
    sc.out.resize(b.max_len + INLINE, 0);
    let mut pass = Pass {
        queues: Queues::new(&mut sc.out[..], &mut sc.spill, cap.m),
        edges: &mut sc.edges,
        ns: 0,
        cap_strips: cap.strips,
        f: First::default(),
    };
    let mut mtf = MtfU128::new(&b.seq_to_unseq);
    let mut pos = 0u64;
    let max_len = b.max_len as u64;

    macro_rules! feed {
        ($c:expr, $total:expr, $span:expr) => {{
            let total: u64 = $total;
            if $span > 22 || pos + total > max_len {
                return None;
            }
            pass.take::<true, W1_FAST, false>($c, pos as u32, total as u32, p, LIMIT);
            pos += total;
        }};
    }

    let last = end / 64;
    let (mut wi, mut w) = (0usize, lit[0]);
    macro_rules! seek {
        () => {{
            loop {
                if w != 0 {
                    let t = w.trailing_zeros() as usize;
                    w &= w - 1;
                    break Some(t);
                }
                if wi == last {
                    break None;
                }
                wi += 1;
                w = lit[wi];
            }
        }};
    }

    let first = seek!();
    let q0 = first.map_or(end, |t| wi * 64 + t);
    if q0 > 21 {
        return None;
    }
    let init_len = bzhi_safe(one[0], q0 as u32 + 1) - 1;
    if init_len > 0 {
        feed!(b.seq_to_unseq[0], init_len, q0 + 1);
    }

    if let Some(mut tz) = first {
        let mut after = one[wi] >> 1;
        let mut base = wi * 64;
        loop {
            let s = usize::from(b.syms[base + tz]);
            let c = mtf.move_to_front(s);
            if w != 0 {
                let t2 = w.trailing_zeros() as usize;
                w &= w - 1;
                let total = bzhi_safe(after, t2 as u32) >> tz;
                feed!(c, total, t2 - tz);
                tz = t2;
            } else {
                let q = base + tz;
                let next = seek!();
                let q2 = next.map_or(end, |t| wi * 64 + t);
                let total = extract_run_cross(one, q, q2);
                feed!(c, total, q2 - q);
                match next {
                    Some(t) => {
                        tz = t;
                        after = one[wi] >> 1;
                        base = wi * 64;
                    }
                    None => break,
                }
            }
        }
    }

    let heads = pass.queues.heads();
    let over = pass.queues.over;
    let chunks = pass.queues.chunks;
    Some(A6aResult {
        n: pos as usize,
        f: pass.f,
        ns: pass.ns,
        over,
        heads,
        chunks,
        fast_pushes: 0,
        slow_pushes: 0,
        w1_pushes: 0,
    })
}

/// Compact 4-byte packed maximal runs (`(width << 8) | c`) into A6a: removes 8B tuple padding.
fn run_a6a_from_packed_maximal<const QUEUE: bool>(
    packed: &[u32],
    n: usize,
    p: u32,
    cap: Cap,
    sc: &mut A6aScratch,
) -> A6aResult {
    sc.out.resize(n + INLINE, 0);
    let mut pass = Pass {
        queues: Queues::new(&mut sc.out[..], &mut sc.spill, cap.m),
        edges: &mut sc.edges,
        ns: 0,
        cap_strips: cap.strips,
        f: First::default(),
    };
    let mut pos = 0u32;
    for &entry in packed {
        let c = (entry & 0xff) as u8;
        let width = entry >> 8;
        pass.take::<QUEUE, false, false>(c, pos, width, p, LIMIT);
        pos += width;
    }
    let heads = pass.queues.heads();
    let over = pass.queues.over;
    let chunks = pass.queues.chunks;
    A6aResult {
        n,
        f: pass.f,
        ns: pass.ns,
        over,
        heads,
        chunks,
        fast_pushes: 0,
        slow_pushes: 0,
        w1_pushes: 0,
    }
}

/// Pass 2 only (pre-built `lit`/`one` bitmaps): pure MTF + RLE2 decoding with NO `runs` write,
/// NO cap-127 splitting, NO `counts`/`per` update. Measures irreducible MTF+RLE2 work.
fn p2_mtf_core(b: &Symbols, lit: &[u64], one: &[u64], end: usize) -> Option<(usize, u64)> {
    let mut mtf = MtfU128::new(&b.seq_to_unseq);
    let mut pos = 0u64;
    let mut acc = 0u64;
    let last = end / 64;
    let (mut wi, mut w) = (0usize, lit[0]);
    macro_rules! seek {
        () => {{
            loop {
                if w != 0 {
                    let t = w.trailing_zeros() as usize;
                    w &= w - 1;
                    break Some(t);
                }
                if wi == last {
                    break None;
                }
                wi += 1;
                w = lit[wi];
            }
        }};
    }
    let first = seek!();
    let q0 = first.map_or(end, |t| wi * 64 + t);
    if q0 > 21 {
        return None;
    }
    let init_len = bzhi_safe(one[0], q0 as u32 + 1) - 1;
    if init_len > 0 {
        pos += init_len;
        acc = acc.wrapping_add(u64::from(b.seq_to_unseq[0]) ^ init_len);
    }
    if let Some(mut tz) = first {
        let mut after = one[wi] >> 1;
        let mut base = wi * 64;
        loop {
            let s = usize::from(b.syms[base + tz]);
            let c = mtf.move_to_front(s);
            if w != 0 {
                let t2 = w.trailing_zeros() as usize;
                w &= w - 1;
                let total = bzhi_safe(after, t2 as u32) >> tz;
                pos += total;
                acc = acc.wrapping_add(u64::from(c) ^ total);
                tz = t2;
            } else {
                let q = base + tz;
                let next = seek!();
                let q2 = next.map_or(end, |t| wi * 64 + t);
                let total = extract_run_cross(one, q, q2);
                pos += total;
                acc = acc.wrapping_add(u64::from(c) ^ total);
                match next {
                    Some(t) => {
                        tz = t;
                        after = one[wi] >> 1;
                        base = wi * 64;
                    }
                    None => break,
                }
            }
        }
    }
    Some((pos as usize, acc))
}

/// Pass 2 only (pre-built `lit`/`one` bitmaps): writes cap-127 `runs` into a pre-sized slice
/// (no `Vec::push` growth check) plus `cnt`/`counts`/`per`, matching `front_avx2` Pass 2.
fn p2_mtf_with_runs(
    b: &Symbols,
    lit: &[u64],
    one: &[u64],
    end: usize,
    f: &mut RunsMirror,
) -> Option<usize> {
    let m = b.syms.len();
    let mut slack = b.max_len / CAP as usize + 1;
    let need = m + slack + 2;
    if f.runs.len() < need {
        f.runs.resize(need, 0);
    }
    let out = &mut f.runs[..need];
    let mut out_len = 0usize;
    let mut mtf = MtfU128::new(&b.seq_to_unseq);
    let mut cnt = [0u64; 256];
    let mut big = 0u64;

    macro_rules! push_long {
        ($c:expr, $total:expr) => {{
            let (c, total): (u8, u64) = ($c, $total);
            let recs = total.div_ceil(u64::from(CAP));
            let more = recs.saturating_sub(1) as usize;
            big += total;
            if more > slack || big > b.max_len as u64 {
                return None;
            }
            slack -= more;
            cnt[usize::from(c)] += total | (recs << 32);
            let mut left = total;
            while left > 0 {
                let len = left.min(u64::from(CAP));
                out[out_len] = (u16::from(c) << 8) | len as u16;
                out_len += 1;
                left -= len;
            }
        }};
    }

    macro_rules! emit_rec {
        ($c:expr, $total:expr, $span:expr) => {{
            let (c, total): (u8, u64) = ($c, $total);
            if total > u64::from(CAP) {
                if $span > 22 {
                    return None;
                }
                push_long!(c, total);
            } else {
                out[out_len] = (u16::from(c) << 8) | total as u16;
                out_len += 1;
                cnt[usize::from(c)] += total | (1u64 << 32);
            }
        }};
    }

    let last = end / 64;
    let (mut wi, mut w) = (0usize, lit[0]);
    macro_rules! seek {
        () => {{
            loop {
                if w != 0 {
                    let t = w.trailing_zeros() as usize;
                    w &= w - 1;
                    break Some(t);
                }
                if wi == last {
                    break None;
                }
                wi += 1;
                w = lit[wi];
            }
        }};
    }

    let first = seek!();
    let q0 = first.map_or(end, |t| wi * 64 + t);
    if q0 > 21 {
        return None;
    }
    let init_len = bzhi_safe(one[0], q0 as u32 + 1) - 1;
    if init_len > 0 {
        push_long!(b.seq_to_unseq[0], init_len);
    }

    if let Some(mut tz) = first {
        let mut after = one[wi] >> 1;
        let mut base = wi * 64;
        loop {
            let s = usize::from(b.syms[base + tz]);
            let c = mtf.move_to_front(s);
            if w != 0 {
                let t2 = w.trailing_zeros() as usize;
                w &= w - 1;
                let total = bzhi_safe(after, t2 as u32) >> tz;
                emit_rec!(c, total, t2 - tz);
                tz = t2;
            } else {
                let q = base + tz;
                let next = seek!();
                let q2 = next.map_or(end, |t| wi * 64 + t);
                if q2 - q > 22 {
                    return None;
                }
                let total = extract_run_cross(one, q, q2);
                emit_rec!(c, total, q2 - q);
                match next {
                    Some(t) => {
                        tz = t;
                        after = one[wi] >> 1;
                        base = wi * 64;
                    }
                    None => break,
                }
            }
        }
    }

    let n: u64 = cnt.iter().map(|&x| x & 0xffff_ffff).sum();
    if n > b.max_len as u64 {
        return None;
    }
    f.runs.truncate(out_len);
    for (c, &x) in cnt.iter().enumerate() {
        f.counts[c] = x as u32;
        f.per[c] = (x >> 32) as u32;
    }
    Some(n as usize)
}

/// Pass 2 only (pre-built `lit`/`one` bitmaps): directly folded into A6a (`Pass::take`).
fn p2_folded_a5_a6a<const QUEUE: bool>(
    b: &Symbols,
    lit: &[u64],
    one: &[u64],
    end: usize,
    p: u32,
    cap: Cap,
    sc: &mut A6aScratch,
) -> Option<A6aResult> {
    sc.out.resize(b.max_len + INLINE, 0);
    let mut pass = Pass {
        queues: Queues::new(&mut sc.out[..], &mut sc.spill, cap.m),
        edges: &mut sc.edges,
        ns: 0,
        cap_strips: cap.strips,
        f: First::default(),
    };
    let mut mtf = MtfU128::new(&b.seq_to_unseq);
    let mut pos = 0u64;
    let max_len = b.max_len as u64;

    macro_rules! feed {
        ($c:expr, $total:expr, $span:expr) => {{
            let total: u64 = $total;
            if $span > 22 || pos + total > max_len {
                return None;
            }
            pass.take::<QUEUE, false, false>($c, pos as u32, total as u32, p, LIMIT);
            pos += total;
        }};
    }

    let last = end / 64;
    let (mut wi, mut w) = (0usize, lit[0]);
    macro_rules! seek {
        () => {{
            loop {
                if w != 0 {
                    let t = w.trailing_zeros() as usize;
                    w &= w - 1;
                    break Some(t);
                }
                if wi == last {
                    break None;
                }
                wi += 1;
                w = lit[wi];
            }
        }};
    }

    let first = seek!();
    let q0 = first.map_or(end, |t| wi * 64 + t);
    if q0 > 21 {
        return None;
    }
    let init_len = bzhi_safe(one[0], q0 as u32 + 1) - 1;
    if init_len > 0 {
        feed!(b.seq_to_unseq[0], init_len, q0 + 1);
    }

    if let Some(mut tz) = first {
        let mut after = one[wi] >> 1;
        let mut base = wi * 64;
        loop {
            let s = usize::from(b.syms[base + tz]);
            let c = mtf.move_to_front(s);
            if w != 0 {
                let t2 = w.trailing_zeros() as usize;
                w &= w - 1;
                let total = bzhi_safe(after, t2 as u32) >> tz;
                feed!(c, total, t2 - tz);
                tz = t2;
            } else {
                let q = base + tz;
                let next = seek!();
                let q2 = next.map_or(end, |t| wi * 64 + t);
                let total = extract_run_cross(one, q, q2);
                feed!(c, total, q2 - q);
                match next {
                    Some(t) => {
                        tz = t;
                        after = one[wi] >> 1;
                        base = wi * 64;
                    }
                    None => break,
                }
            }
        }
    }

    let heads = pass.queues.heads();
    let over = pass.queues.over;
    let chunks = pass.queues.chunks;
    Some(A6aResult {
        n: pos as usize,
        f: pass.f,
        ns: pass.ns,
        over,
        heads,
        chunks,
        fast_pushes: 0,
        slow_pushes: 0,
        w1_pushes: 0,
    })
}

// ---------------------------------------------------------------------------
// Corpus loading and exactness helpers
// ---------------------------------------------------------------------------

struct BlockData {
    index: u64,
    symbols: Symbols,
    takes_runs: bool,
    n: usize,
    cap_prod: Option<Cap>,
    cap_maximal: Option<Cap>,
    records: Vec<u16>,
    max_runs: Vec<(u8, u32)>,
    packed_max: Vec<u32>,
    lit: Vec<u64>,
    one: Vec<u64>,
    end: usize,
}

fn load_blocks(path: &str, forced: bool) -> Vec<BlockData> {
    let raw = std::fs::read(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    let mut scanner = Scanner::new(SliceSource(&raw));
    let mut blocks = Vec::new();
    let mut mirror = RunsMirror::default();
    let mut prod_runs = Runs::default();
    let k = Kernels::detect();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    loop {
        match scanner.next(Vec::new()).expect("scan error") {
            Event::Done => break,
            Event::Block { index, symbols, .. } => {
                let tr = runs::takes_runs(&symbols);
                let (n, records, max_runs, packed_max, cap_prod, cap_maximal, lit, one, end) = if tr
                    || forced
                {
                    let n_prod = runs::front(&symbols, &mut prod_runs, k).expect("prod front");
                    let n_mir = front_scalar_mirror(&symbols, &mut mirror).expect("scalar mirror");
                    assert_eq!(n_prod, n_mir, "block {index}: prod front vs mirror n");
                    // Verify exact records against mtf::mtf
                    let n_mtf = mtf::mtf(&symbols, &mut ll, &mut counts, k).expect("mtf::mtf");
                    assert_eq!(n_mtf, n_prod, "block {index}: mtf vs front n");
                    assert_eq!(counts, mirror.counts, "block {index}: counts mismatch");

                    let mut mruns = Vec::with_capacity(mirror.runs.len());
                    let mut pmax = Vec::with_capacity(mirror.runs.len());
                    maximal(&mirror.runs, |c, _pos, width| {
                        mruns.push((c, width));
                        pmax.push((width << 8) | u32::from(c));
                    });
                    let mut check_pos = 0usize;
                    for &(c, w) in &mruns {
                        let end_p = check_pos + w as usize;
                        assert!(
                            ll[check_pos..end_p].iter().all(|&b| b == c),
                            "block {index}: maximal run mismatch vs mtf column"
                        );
                        check_pos = end_p;
                    }
                    assert_eq!(check_pos, n_mtf);

                    let mut lit_v = Vec::new();
                    let mut one_v = Vec::new();
                    let end_idx =
                        build_bitmaps_safe(&symbols, &mut lit_v, &mut one_v).expect("eob");

                    let cap_p = open_cap(mirror.runs.len(), n_prod, symbols.orig_ptr, forced);
                    let cap_m = open_cap(mruns.len(), n_prod, symbols.orig_ptr, forced);
                    (
                        n_prod,
                        mirror.runs.clone(),
                        mruns,
                        pmax,
                        cap_p,
                        cap_m,
                        lit_v,
                        one_v,
                        end_idx,
                    )
                } else {
                    let n_mtf = mtf::mtf(&symbols, &mut ll, &mut counts, k).expect("mtf::mtf");
                    (
                        n_mtf,
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        None,
                        None,
                        Vec::new(),
                        Vec::new(),
                        0,
                    )
                };
                blocks.push(BlockData {
                    index,
                    symbols,
                    takes_runs: tr,
                    n,
                    cap_prod,
                    cap_maximal,
                    records,
                    max_runs,
                    packed_max,
                    lit,
                    one,
                    end,
                });
            }
        }
    }
    blocks
}

fn verify_state_equal(
    block_idx: u64,
    label: &str,
    ref_res: &A6aResult,
    ref_sc: &A6aScratch,
    cand_res: &A6aResult,
    cand_sc: &A6aScratch,
) {
    assert_eq!(
        ref_res.n, cand_res.n,
        "block {block_idx} ({label}): n mismatch"
    );
    assert_eq!(
        ref_res.f, cand_res.f,
        "block {block_idx} ({label}): First mismatch"
    );
    assert_eq!(
        ref_res.ns, cand_res.ns,
        "block {block_idx} ({label}): ns mismatch"
    );
    assert_eq!(
        ref_res.over, cand_res.over,
        "block {block_idx} ({label}): over mismatch"
    );
    let edge_words = 3 * ref_res.ns.min(ref_sc.edges.len() / 3);
    assert_eq!(
        &ref_sc.edges[..edge_words],
        &cand_sc.edges[..edge_words],
        "block {block_idx} ({label}): edges strips mismatch"
    );
    assert_eq!(
        ref_sc.spill.cuts, cand_sc.spill.cuts,
        "block {block_idx} ({label}): OutQ16 epoch cuts mismatch"
    );
    if !ref_res.over {
        for a in 0..256 {
            let cnt = ref_res.f.thin[a];
            if cnt == 0 {
                continue;
            }
            let q_ref = drain_queue(&ref_sc.out, &ref_sc.spill, &ref_res.heads, a, cnt);
            let q_cand = drain_queue(&cand_sc.out, &cand_sc.spill, &cand_res.heads, a, cnt);
            assert_eq!(
                q_ref, q_cand,
                "block {block_idx} ({label}): OutQ16 bucket {a} rank stream mismatch"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Statistics helpers
// ---------------------------------------------------------------------------

fn percentile_u64(sorted: &[u64], pct: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() - 1) as f64 * pct).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn median_min_max(vals: &[f64]) -> (f64, f64, f64) {
    let mut s = vals.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    let med = if s.len() % 2 == 1 {
        s[s.len() / 2]
    } else {
        0.5 * (s[s.len() / 2 - 1] + s[s.len() / 2])
    };
    (med, s[0], *s.last().unwrap())
}

// ---------------------------------------------------------------------------
// Subcommand 1: structural census (`stats`)
// ---------------------------------------------------------------------------

const BUCKET_LABELS: [&str; 11] = [
    "1",
    "2",
    "3..=4",
    "5..=8",
    "9..=16",
    "17..=32",
    "33..=64",
    "65..=127",
    "128..=255",
    "256..=1024",
    ">1024",
];

fn bucket_of(w: u32) -> usize {
    match w {
        1 => 0,
        2 => 1,
        3..=4 => 2,
        5..=8 => 3,
        9..=16 => 4,
        17..=32 => 5,
        33..=64 => 6,
        65..=127 => 7,
        128..=255 => 8,
        256..=1024 => 9,
        _ => 10,
    }
}

fn run_stats(path: &str, forced: bool) {
    let blocks = load_blocks(path, forced);
    let total_blocks = blocks.len();
    let total_rows: u64 = blocks.iter().map(|b| b.n as u64).sum();
    let takes_runs_blocks = blocks.iter().filter(|b| b.takes_runs).count();
    let mut open_none = 0usize;
    let mut open_dense_pass = 0usize;
    let mut open_dense_over = 0usize;
    let mut run_refused = 0usize;
    let mut admission_diff_blocks = 0usize;
    let mut admission_diff_details = Vec::new();

    let mut sc = A6aScratch::default();

    // Aggregates over Open-admitted blocks (and also all takes_runs blocks)
    let mut open_blocks = 0usize;
    let mut open_rows = 0u64;
    let mut open_syms = 0u64;
    let mut open_lits = 0u64;
    let mut open_run_syms = 0u64;
    let mut open_records = 0u64;
    let mut open_max_runs = 0u64;
    let mut open_split_runs = 0u64;
    let mut open_split_rows = 0u64;
    let mut open_split_records = 0u64;
    let mut max_run_len_seen = 0u32;

    let mut bucket_runs = [0u64; 11];
    let mut bucket_rows = [0u64; 11];
    let mut bucket_recs = [0u64; 11];

    // Branch predictability for `width <= LIMIT`
    let mut thin_thick_transitions = 0u64;
    let mut b2_misses = 0u64;
    let mut total_max_run_steps = 0u64;

    // A6a & OutQ16 metrics
    let mut total_m = 0u64;
    let mut total_ns = 0u64;
    let mut total_fast_pushes = 0u64;
    let mut total_slow_pushes = 0u64;
    let mut total_w1_pushes = 0u64;
    let mut total_chunks = 0u64;
    let mut total_ovf = 0u64;
    let mut total_cuts = 0u64;

    let mut per_blk_runs_kb = Vec::new();
    let mut per_blk_syms_kb = Vec::new();
    let mut per_blk_bitmap_kb = Vec::new();
    let mut per_blk_edges_kb = Vec::new();
    let mut per_blk_chunks = Vec::new();
    let mut per_blk_cuts = Vec::new();
    let mut per_blk_active_buckets = Vec::new();
    let mut all_max_run_lens = Vec::new();

    // Also track cap-127 splits across ALL takes_runs blocks
    let mut tr_rows = 0u64;
    let mut tr_records = 0u64;
    let mut tr_max_runs = 0u64;
    let mut tr_split_runs = 0u64;
    let mut tr_split_rows = 0u64;

    for b in &blocks {
        if b.takes_runs {
            tr_rows += b.n as u64;
            tr_records += b.records.len() as u64;
            tr_max_runs += b.max_runs.len() as u64;
            for &(_, w) in &b.max_runs {
                if w > CAP {
                    tr_split_runs += 1;
                    tr_split_rows += u64::from(w);
                }
            }
        }
        if b.takes_runs && b.cap_prod != b.cap_maximal {
            admission_diff_blocks += 1;
            admission_diff_details.push((
                b.index,
                b.n,
                b.records.len(),
                b.max_runs.len(),
                b.cap_prod,
                b.cap_maximal,
            ));
        }
        let Some(cap) = b.cap_prod else {
            if b.takes_runs {
                run_refused += 1;
            }
            continue;
        };
        if !b.takes_runs && !forced {
            continue;
        }
        let res = run_a6a_from_records::<true, false, true>(
            &b.records,
            b.n,
            b.symbols.orig_ptr,
            cap,
            &mut sc,
        );
        if res.over || res.f.m > cap.m || res.ns > cap.strips {
            open_dense_over += 1;
            continue;
        }
        if cap == Cap::NONE {
            open_none += 1;
        } else {
            open_dense_pass += 1;
        }

        open_blocks += 1;
        open_rows += b.n as u64;
        open_syms += b.symbols.syms.len() as u64;
        for &s in &b.symbols.syms {
            if s <= 1 {
                open_run_syms += 1;
            } else if s == b.symbols.eob {
                break;
            } else {
                open_lits += 1;
            }
        }
        open_records += b.records.len() as u64;
        open_max_runs += b.max_runs.len() as u64;

        let mut prev_thin = true;
        let mut b2_state: u8 = 3; // strongly thin
        for (idx_r, &(_, w)) in b.max_runs.iter().enumerate() {
            let bi = bucket_of(w);
            let recs = u64::from(w.div_ceil(CAP));
            bucket_runs[bi] += 1;
            bucket_rows[bi] += u64::from(w);
            bucket_recs[bi] += recs;
            if w > CAP {
                open_split_runs += 1;
                open_split_rows += u64::from(w);
                open_split_records += recs;
            }
            if w > max_run_len_seen {
                max_run_len_seen = w;
            }
            all_max_run_lens.push(u64::from(w));

            let is_thin = w <= LIMIT;
            if idx_r > 0 && is_thin != prev_thin {
                thin_thick_transitions += 1;
            }
            prev_thin = is_thin;
            let pred_thin = b2_state >= 2;
            if pred_thin != is_thin {
                b2_misses += 1;
            }
            b2_state = if is_thin {
                (b2_state + 1).min(3)
            } else {
                b2_state.saturating_sub(1)
            };
            total_max_run_steps += 1;
        }

        total_m += u64::from(res.f.m);
        total_ns += res.ns as u64;
        total_fast_pushes += res.fast_pushes;
        total_slow_pushes += res.slow_pushes;
        total_w1_pushes += res.w1_pushes;
        total_chunks += u64::from(res.chunks);
        total_ovf += sc.spill.ovf.len() as u64;
        total_cuts += sc.spill.cuts.len() as u64;

        per_blk_runs_kb.push((b.records.len() * 2) as u64);
        per_blk_syms_kb.push((b.symbols.syms.len() * 2) as u64);
        per_blk_bitmap_kb.push(((b.symbols.syms.len() / 64 + 3) * 16) as u64);
        per_blk_edges_kb.push((res.ns * 12) as u64);
        per_blk_chunks.push(u64::from(res.chunks));
        per_blk_cuts.push(sc.spill.cuts.len() as u64);
        let active = res.f.thin.iter().filter(|&&x| x > 0).count() as u64;
        per_blk_active_buckets.push(active);
    }

    per_blk_runs_kb.sort_unstable();
    per_blk_syms_kb.sort_unstable();
    per_blk_bitmap_kb.sort_unstable();
    per_blk_edges_kb.sort_unstable();
    per_blk_chunks.sort_unstable();
    per_blk_cuts.sort_unstable();
    per_blk_active_buckets.sort_unstable();
    all_max_run_lens.sort_unstable();

    println!("=== MTF-FOLD STRUCTURAL CENSUS: {path} (forced={forced}) ===");
    println!(
        "Blocks: total={total_blocks}, total_rows={total_rows}, takes_runs={takes_runs_blocks}, \
         open_admitted={open_blocks} (Cap::NONE={open_none}, Cap::dense_pass={open_dense_pass}, \
         Cap::dense_over={open_dense_over}, run_refused={run_refused}), open_rows={open_rows} ({:.3}%)",
        100.0 * open_rows as f64 / total_rows.max(1) as f64
    );
    println!(
        "Admission sensitivity (records.len() vs maximal_runs in OpenCells::cap): {} blocks differ",
        admission_diff_blocks
    );
    for (idx, n, recs, mruns, cp, cm) in admission_diff_details.iter().take(10) {
        println!(
            "  diff block {idx}: n={n}, recs={recs} (r/n={:.4}), max_runs={mruns} (mr/n={:.4}), cap_prod={cp:?}, cap_max={cm:?}",
            *recs as f64 / *n as f64,
            *mruns as f64 / *n as f64
        );
    }

    println!("\n--- Cap-127 run splitting (all takes_runs blocks) ---");
    let tr_extra = tr_records.saturating_sub(tr_max_runs);
    println!(
        "takes_runs: rows={tr_rows}, records={tr_records}, maximal_runs={tr_max_runs}, \
         extra_records={tr_extra} (+{:.2}%), split_runs={tr_split_runs} ({:.2}% of max_runs), \
         split_rows={tr_split_rows} ({:.2}% of rows)",
        100.0 * tr_extra as f64 / tr_max_runs.max(1) as f64,
        100.0 * tr_split_runs as f64 / tr_max_runs.max(1) as f64,
        100.0 * tr_split_rows as f64 / tr_rows.max(1) as f64
    );

    println!("\n--- Cap-127 run splitting (Open-admitted blocks) ---");
    let extra_recs = open_records.saturating_sub(open_max_runs);
    println!(
        "Symbols: total={open_syms}, literals={open_lits} ({:.2}%), run_syms={open_run_syms} ({:.2}%)",
        100.0 * open_lits as f64 / open_syms.max(1) as f64,
        100.0 * open_run_syms as f64 / open_syms.max(1) as f64
    );
    println!(
        "Runs: records={open_records}, maximal_runs={open_max_runs}, extra_cap127_records={extra_recs} (+{:.2}% records / {:.2}% of emitted records)",
        100.0 * extra_recs as f64 / open_max_runs.max(1) as f64,
        100.0 * extra_recs as f64 / open_records.max(1) as f64
    );
    println!(
        "Split maximal runs (width > 127): count={open_split_runs} ({:.2}% of maximal runs), \
         rows={open_split_rows} ({:.2}% of Open rows), records_emitted={open_split_records} ({:.2}% of records), \
         mean_recs_per_split={:.2}, max_run_len={max_run_len_seen}",
        100.0 * open_split_runs as f64 / open_max_runs.max(1) as f64,
        100.0 * open_split_rows as f64 / open_rows.max(1) as f64,
        100.0 * open_split_records as f64 / open_records.max(1) as f64,
        open_split_records as f64 / open_split_runs.max(1) as f64
    );

    let thin_runs: u64 = bucket_runs[0..=4].iter().sum();
    let thin_rows: u64 = bucket_rows[0..=4].iter().sum();
    let thick_runs: u64 = bucket_runs[5..].iter().sum();
    let thick_rows: u64 = bucket_rows[5..].iter().sum();
    println!(
        "Classification: thin (<=16) = {thin_runs} runs ({:.2}%), {thin_rows} rows ({:.2}%); \
         thick (>16) = {thick_runs} runs ({:.2}%), {thick_rows} rows ({:.2}%)",
        100.0 * thin_runs as f64 / open_max_runs.max(1) as f64,
        100.0 * thin_rows as f64 / open_rows.max(1) as f64,
        100.0 * thick_runs as f64 / open_max_runs.max(1) as f64,
        100.0 * thick_rows as f64 / open_rows.max(1) as f64
    );
    println!(
        "Branch `width <= 16`: transitions={thin_thick_transitions} ({:.2}%), 2-bit predictor misses={b2_misses} ({:.2}%)",
        100.0 * thin_thick_transitions as f64 / total_max_run_steps.max(1) as f64,
        100.0 * b2_misses as f64 / total_max_run_steps.max(1) as f64
    );

    println!("\n--- Maximal run-length histogram (Open-admitted blocks) ---");
    println!(
        "{:<12} {:>12} {:>8} {:>14} {:>8} {:>12} {:>8}",
        "bucket", "max_runs", "run%", "rows", "row%", "u16_recs", "extra"
    );
    for i in 0..11 {
        let extra = bucket_recs[i].saturating_sub(bucket_runs[i]);
        println!(
            "{:<12} {:>12} {:>7.2}% {:>14} {:>7.2}% {:>12} {:>8}",
            BUCKET_LABELS[i],
            bucket_runs[i],
            100.0 * bucket_runs[i] as f64 / open_max_runs.max(1) as f64,
            bucket_rows[i],
            100.0 * bucket_rows[i] as f64 / open_rows.max(1) as f64,
            bucket_recs[i],
            extra
        );
    }

    println!(
        "\nMaximal run length percentiles (by run count): p25={} p50={} p75={} p90={} p95={} p99={} max={}",
        percentile_u64(&all_max_run_lens, 0.25),
        percentile_u64(&all_max_run_lens, 0.50),
        percentile_u64(&all_max_run_lens, 0.75),
        percentile_u64(&all_max_run_lens, 0.90),
        percentile_u64(&all_max_run_lens, 0.95),
        percentile_u64(&all_max_run_lens, 0.99),
        percentile_u64(&all_max_run_lens, 1.00),
    );

    println!("\n--- Memory traffic & A6a recomputation (Open-admitted blocks) ---");
    let total_pushes = total_fast_pushes + total_slow_pushes;
    println!(
        "Intermediate `runs: Vec<u16>`: total={:.2} MiB written+read; per-block KiB p50={:.1} p90={:.1} max={:.1}",
        (open_records * 2) as f64 / (1024.0 * 1024.0),
        percentile_u64(&per_blk_runs_kb, 0.50) as f64 / 1024.0,
        percentile_u64(&per_blk_runs_kb, 0.90) as f64 / 1024.0,
        percentile_u64(&per_blk_runs_kb, 1.00) as f64 / 1024.0,
    );
    println!(
        "Input `syms: Vec<u16>`: total={:.2} MiB; per-block KiB p50={:.1} p90={:.1} max={:.1}",
        (open_syms * 2) as f64 / (1024.0 * 1024.0),
        percentile_u64(&per_blk_syms_kb, 0.50) as f64 / 1024.0,
        percentile_u64(&per_blk_syms_kb, 0.90) as f64 / 1024.0,
        percentile_u64(&per_blk_syms_kb, 1.00) as f64 / 1024.0,
    );
    println!(
        "A5 bitmaps (`lit`+`one`): per-block KiB p50={:.1} p90={:.1} max={:.1}; unused `counts`+`per` = 2.0 KiB/blk ({:.2} MiB total)",
        percentile_u64(&per_blk_bitmap_kb, 0.50) as f64 / 1024.0,
        percentile_u64(&per_blk_bitmap_kb, 0.90) as f64 / 1024.0,
        percentile_u64(&per_blk_bitmap_kb, 1.00) as f64 / 1024.0,
        (open_blocks * 2048) as f64 / (1024.0 * 1024.0),
    );
    println!(
        "A6a thick strips (`edges`): ns={total_ns} ({:.2} MiB); per-block KiB p50={:.1} p90={:.1} max={:.1}",
        (total_ns * 12) as f64 / (1024.0 * 1024.0),
        percentile_u64(&per_blk_edges_kb, 0.50) as f64 / 1024.0,
        percentile_u64(&per_blk_edges_kb, 0.90) as f64 / 1024.0,
        percentile_u64(&per_blk_edges_kb, 1.00) as f64 / 1024.0,
    );
    println!(
        "A6a OutQ16: retained m={total_m} ({:.2} MiB exact 2B ranks), pushes={total_pushes} \
         (fast={total_fast_pushes} {:.2}%, slow={total_slow_pushes} {:.2}%, w==1={total_w1_pushes} {:.2}%), \
         32B-iota written={:.2} MiB (write amp {:.2}x)",
        (total_m * 2) as f64 / (1024.0 * 1024.0),
        100.0 * total_fast_pushes as f64 / total_pushes.max(1) as f64,
        100.0 * total_slow_pushes as f64 / total_pushes.max(1) as f64,
        100.0 * total_w1_pushes as f64 / total_pushes.max(1) as f64,
        (total_fast_pushes * 32) as f64 / (1024.0 * 1024.0),
        (total_fast_pushes * 32) as f64 / (total_m * 2).max(1) as f64,
    );
    println!(
        "A6a OutQ16 chunks/blk p50={} p90={} max={} (total={total_chunks}, ovf={total_ovf}), \
         epoch cuts/blk p50={} p90={} max={} (total={total_cuts}), active buckets/blk p50={} p90={} max={}",
        percentile_u64(&per_blk_chunks, 0.50),
        percentile_u64(&per_blk_chunks, 0.90),
        percentile_u64(&per_blk_chunks, 1.00),
        percentile_u64(&per_blk_cuts, 0.50),
        percentile_u64(&per_blk_cuts, 0.90),
        percentile_u64(&per_blk_cuts, 1.00),
        percentile_u64(&per_blk_active_buckets, 0.50),
        percentile_u64(&per_blk_active_buckets, 0.90),
        percentile_u64(&per_blk_active_buckets, 1.00),
    );
}

// ---------------------------------------------------------------------------
// Subcommand 2: exact-verified timing census (`bench`)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Arm {
    ProdA5Avx2Alone,
    ProdA5ScalarAlone,
    BitmapPass1Alone,
    BitmapU128A5Alone,
    ProdA6aAlone,
    A6aFromMaximal,
    A6aFromPackedMax,
    A6aNoOutq,
    A6aMaximalNoOutq,
    A6aW1Fast,
    ProdA5Avx2PlusA6a,
    UnfoldedScalar,
    FoldedScalar,
    UnfoldedBitmapU128,
    FoldedBitmapU128,
    P2MtfCore,
    P2MtfWithRuns,
    P2Unfolded,
    P2Folded,
    P2FoldedNoOutq,
}

const ALL_ARMS: [Arm; 20] = [
    Arm::ProdA5Avx2Alone,
    Arm::ProdA5ScalarAlone,
    Arm::BitmapPass1Alone,
    Arm::BitmapU128A5Alone,
    Arm::ProdA6aAlone,
    Arm::A6aFromMaximal,
    Arm::A6aFromPackedMax,
    Arm::A6aNoOutq,
    Arm::A6aMaximalNoOutq,
    Arm::A6aW1Fast,
    Arm::ProdA5Avx2PlusA6a,
    Arm::UnfoldedScalar,
    Arm::FoldedScalar,
    Arm::UnfoldedBitmapU128,
    Arm::FoldedBitmapU128,
    Arm::P2MtfCore,
    Arm::P2MtfWithRuns,
    Arm::P2Unfolded,
    Arm::P2Folded,
    Arm::P2FoldedNoOutq,
];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::ProdA5Avx2Alone => "prod_a5_avx2_alone",
            Arm::ProdA5ScalarAlone => "prod_a5_scalar_alone",
            Arm::BitmapPass1Alone => "bitmap_pass1_alone",
            Arm::BitmapU128A5Alone => "bitmap_u128_a5_alone",
            Arm::ProdA6aAlone => "prod_a6a_alone",
            Arm::A6aFromMaximal => "a6a_from_maximal",
            Arm::A6aFromPackedMax => "a6a_from_packed_max",
            Arm::A6aNoOutq => "a6a_no_outq",
            Arm::A6aMaximalNoOutq => "a6a_maximal_no_outq",
            Arm::A6aW1Fast => "a6a_w1_fast",
            Arm::ProdA5Avx2PlusA6a => "prod_a5_avx2_plus_a6a",
            Arm::UnfoldedScalar => "unfolded_scalar",
            Arm::FoldedScalar => "folded_scalar",
            Arm::UnfoldedBitmapU128 => "unfolded_bitmap_u128",
            Arm::FoldedBitmapU128 => "folded_bitmap_u128",
            Arm::P2MtfCore => "p2_mtf_core",
            Arm::P2MtfWithRuns => "p2_mtf_with_runs",
            Arm::P2Unfolded => "p2_unfolded",
            Arm::P2Folded => "p2_folded",
            Arm::P2FoldedNoOutq => "p2_folded_no_outq",
        }
    }
}

fn run_bench(path: &str, forced: bool, repeats: usize) {
    let all_blocks = load_blocks(path, forced);
    // Select admitted Open blocks (where A5 + A6a both run and succeed)
    let mut ref_sc = A6aScratch::default();
    let mut blocks: Vec<&BlockData> = Vec::new();
    let mut open_rows = 0u64;
    let mut open_max_runs = 0u64;
    let mut open_records = 0u64;
    for b in &all_blocks {
        if !b.takes_runs && !forced {
            continue;
        }
        let Some(cap) = b.cap_prod else { continue };
        let res = run_a6a_from_records::<true, false, false>(
            &b.records,
            b.n,
            b.symbols.orig_ptr,
            cap,
            &mut ref_sc,
        );
        if res.over || res.f.m > cap.m || res.ns > cap.strips {
            continue;
        }
        blocks.push(b);
        open_rows += b.n as u64;
        open_max_runs += b.max_runs.len() as u64;
        open_records += b.records.len() as u64;
    }

    assert!(!blocks.is_empty(), "no admitted Open blocks in {path}");

    // Full 100% state-equality verification pass across EVERY block before any timing!
    let mut cand_sc = A6aScratch::default();
    let mut mir = RunsMirror::default();
    let mut lit = Vec::new();
    let mut one = Vec::new();
    for b in &blocks {
        let cap = b.cap_prod.unwrap();
        let ref_res = run_a6a_from_records::<true, false, false>(
            &b.records,
            b.n,
            b.symbols.orig_ptr,
            cap,
            &mut ref_sc,
        );

        // 1) a6a_from_maximal
        let r_max = run_a6a_from_maximal::<true, false>(
            &b.max_runs,
            b.n,
            b.symbols.orig_ptr,
            cap,
            &mut cand_sc,
        );
        verify_state_equal(
            b.index,
            "a6a_from_maximal",
            &ref_res,
            &ref_sc,
            &r_max,
            &cand_sc,
        );

        // 2) a6a_w1_fast
        let r_w1 = run_a6a_from_records::<true, true, false>(
            &b.records,
            b.n,
            b.symbols.orig_ptr,
            cap,
            &mut cand_sc,
        );
        verify_state_equal(b.index, "a6a_w1_fast", &ref_res, &ref_sc, &r_w1, &cand_sc);

        // 3) unfolded_bitmap_u128
        let n_bm = front_bitmap_u128(&b.symbols, &mut mir).expect("front_bitmap_u128");
        assert_eq!(n_bm, b.n, "block {}: front_bitmap_u128 n", b.index);
        assert_eq!(
            mir.runs, b.records,
            "block {}: front_bitmap_u128 records mismatch",
            b.index
        );
        let r_ubm = run_a6a_from_records::<true, false, false>(
            &mir.runs,
            n_bm,
            b.symbols.orig_ptr,
            cap,
            &mut cand_sc,
        );
        verify_state_equal(
            b.index,
            "unfolded_bitmap_u128",
            &ref_res,
            &ref_sc,
            &r_ubm,
            &cand_sc,
        );

        // 4) folded_scalar
        let r_fs = folded_scalar_a5_a6a::<false>(&b.symbols, b.symbols.orig_ptr, cap, &mut cand_sc)
            .expect("folded_scalar");
        verify_state_equal(b.index, "folded_scalar", &ref_res, &ref_sc, &r_fs, &cand_sc);

        // 5) folded_bitmap_u128
        let r_fbm = folded_bitmap_u128_a5_a6a::<false>(
            &b.symbols,
            b.symbols.orig_ptr,
            cap,
            &mut lit,
            &mut one,
            &mut cand_sc,
        )
        .expect("folded_bitmap_u128");
        verify_state_equal(
            b.index,
            "folded_bitmap_u128",
            &ref_res,
            &ref_sc,
            &r_fbm,
            &cand_sc,
        );

        // 6) a6a_from_packed_max
        let r_pmax = run_a6a_from_packed_maximal::<true>(
            &b.packed_max,
            b.n,
            b.symbols.orig_ptr,
            cap,
            &mut cand_sc,
        );
        verify_state_equal(
            b.index,
            "a6a_from_packed_max",
            &ref_res,
            &ref_sc,
            &r_pmax,
            &cand_sc,
        );

        // 7) p2_mtf_with_runs & p2_folded
        let n_p2 = p2_mtf_with_runs(&b.symbols, &b.lit, &b.one, b.end, &mut mir)
            .expect("p2_mtf_with_runs");
        assert_eq!(n_p2, b.n, "block {}: p2_mtf_with_runs n", b.index);
        assert_eq!(
            mir.runs, b.records,
            "block {}: p2_mtf_with_runs records mismatch",
            b.index
        );
        let r_p2f = p2_folded_a5_a6a::<true>(
            &b.symbols,
            &b.lit,
            &b.one,
            b.end,
            b.symbols.orig_ptr,
            cap,
            &mut cand_sc,
        )
        .expect("p2_folded");
        verify_state_equal(b.index, "p2_folded", &ref_res, &ref_sc, &r_p2f, &cand_sc);
    }

    let k = Kernels::detect();
    let mut prod_runs = Runs::default();
    let mut mir = RunsMirror::default();
    let mut sc = A6aScratch::default();
    let mut lit = Vec::new();
    let mut one = Vec::new();

    // Warm-up pass across all arms (pre-warms scratch Vec capacities too, matching worker steady state)
    let mut run_arm = |arm: Arm| -> (f64, u64) {
        let t0 = Instant::now();
        let mut checksum = 0u64;
        for b in &blocks {
            let cap = b.cap_prod.unwrap();
            let p = b.symbols.orig_ptr;
            match arm {
                Arm::ProdA5Avx2Alone => {
                    let n = runs::front(&b.symbols, &mut prod_runs, k).unwrap();
                    checksum = checksum.wrapping_add(n as u64);
                }
                Arm::ProdA5ScalarAlone => {
                    let n = runs::front_scalar(&b.symbols, &mut prod_runs).unwrap();
                    checksum = checksum.wrapping_add(n as u64);
                }
                Arm::BitmapPass1Alone => {
                    let e = build_bitmaps_safe(&b.symbols, &mut lit, &mut one).unwrap();
                    checksum = checksum.wrapping_add(e as u64 ^ lit[0]);
                }
                Arm::BitmapU128A5Alone => {
                    let n = front_bitmap_u128(&b.symbols, &mut mir).unwrap();
                    checksum = checksum.wrapping_add(n as u64 ^ mir.runs.len() as u64);
                }
                Arm::ProdA6aAlone => {
                    let r = run_a6a_from_records::<true, false, false>(
                        &b.records, b.n, p, cap, &mut sc,
                    );
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::A6aFromMaximal => {
                    let r = run_a6a_from_maximal::<true, false>(&b.max_runs, b.n, p, cap, &mut sc);
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::A6aFromPackedMax => {
                    let r =
                        run_a6a_from_packed_maximal::<true>(&b.packed_max, b.n, p, cap, &mut sc);
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::A6aNoOutq => {
                    let r = run_a6a_from_records::<false, false, false>(
                        &b.records, b.n, p, cap, &mut sc,
                    );
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::A6aMaximalNoOutq => {
                    let r = run_a6a_from_maximal::<false, false>(&b.max_runs, b.n, p, cap, &mut sc);
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::A6aW1Fast => {
                    let r =
                        run_a6a_from_records::<true, true, false>(&b.records, b.n, p, cap, &mut sc);
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::ProdA5Avx2PlusA6a => {
                    let n = runs::front(&b.symbols, &mut prod_runs, k).unwrap();
                    let r =
                        run_a6a_from_records::<true, false, false>(&b.records, n, p, cap, &mut sc);
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::UnfoldedScalar => {
                    let n = front_scalar_mirror(&b.symbols, &mut mir).unwrap();
                    let r =
                        run_a6a_from_records::<true, false, false>(&mir.runs, n, p, cap, &mut sc);
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::FoldedScalar => {
                    let r = folded_scalar_a5_a6a::<false>(&b.symbols, p, cap, &mut sc).unwrap();
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::UnfoldedBitmapU128 => {
                    let n = front_bitmap_u128(&b.symbols, &mut mir).unwrap();
                    let r =
                        run_a6a_from_records::<true, false, false>(&mir.runs, n, p, cap, &mut sc);
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::FoldedBitmapU128 => {
                    let r = folded_bitmap_u128_a5_a6a::<false>(
                        &b.symbols, p, cap, &mut lit, &mut one, &mut sc,
                    )
                    .unwrap();
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::P2MtfCore => {
                    let (n, acc) = p2_mtf_core(&b.symbols, &b.lit, &b.one, b.end).unwrap();
                    checksum = checksum.wrapping_add(n as u64 ^ acc);
                }
                Arm::P2MtfWithRuns => {
                    let n = p2_mtf_with_runs(&b.symbols, &b.lit, &b.one, b.end, &mut mir).unwrap();
                    checksum = checksum.wrapping_add(n as u64 ^ mir.runs.len() as u64);
                }
                Arm::P2Unfolded => {
                    let n = p2_mtf_with_runs(&b.symbols, &b.lit, &b.one, b.end, &mut mir).unwrap();
                    let r =
                        run_a6a_from_records::<true, false, false>(&mir.runs, n, p, cap, &mut sc);
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::P2Folded => {
                    let r = p2_folded_a5_a6a::<true>(
                        &b.symbols, &b.lit, &b.one, b.end, p, cap, &mut sc,
                    )
                    .unwrap();
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
                Arm::P2FoldedNoOutq => {
                    let r = p2_folded_a5_a6a::<false>(
                        &b.symbols, &b.lit, &b.one, b.end, p, cap, &mut sc,
                    )
                    .unwrap();
                    checksum = checksum.wrapping_add(u64::from(r.f.m) ^ r.ns as u64);
                }
            }
        }
        let elapsed_ms = t0.elapsed().as_secs_f64() * 1e3;
        black_box(checksum);
        (elapsed_ms, checksum)
    };

    for &arm in &ALL_ARMS {
        let _ = run_arm(arm);
    }

    let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); ALL_ARMS.len()];
    for rep in 0..repeats {
        let indices: Vec<usize> = if rep % 2 == 0 {
            (0..ALL_ARMS.len()).collect()
        } else {
            (0..ALL_ARMS.len()).rev().collect()
        };
        for ai in indices {
            let (ms, _) = run_arm(ALL_ARMS[ai]);
            times[ai].push(ms);
        }
    }

    println!("=== MTF-FOLD TIMING CENSUS: {path} (forced={forced}, repeats={repeats}) ===");
    println!(
        "Verified 100% bit-for-bit A6a state-identical across all {} admitted Open blocks \
         (rows={open_rows}, max_runs={open_max_runs}, records={open_records}, Kernels::avx2={})",
        blocks.len(),
        k.avx2()
    );
    println!(
        "\n{:<24} {:>10} {:>19} {:>10} {:>12} {:>10}",
        "arm", "med_ms", "min..max_ms", "ns/row", "ns/max_run", "vs_prod"
    );
    let prod_idx = 10; // ProdA5Avx2PlusA6a
    let (prod_med, _, _) = median_min_max(&times[prod_idx]);
    for (ai, &arm) in ALL_ARMS.iter().enumerate() {
        let (med, lo, hi) = median_min_max(&times[ai]);
        let ns_row = med * 1e6 / open_rows as f64;
        let ns_run = med * 1e6 / open_max_runs as f64;
        let ratio = med / prod_med;
        println!(
            "{:<24} {:>10.2} {:>9.2}..{:<8.2} {:>10.3} {:>12.2} {:>9.3}x",
            arm.name(),
            med,
            lo,
            hi,
            ns_row,
            ns_run,
            ratio
        );
    }

    let paired_ratio = |num_i: usize, den_i: usize| -> (f64, f64, f64) {
        let ratios: Vec<f64> = (0..repeats)
            .map(|r| times[num_i][r] / times[den_i][r])
            .collect();
        median_min_max(&ratios)
    };
    let paired_diff_ms = |a_i: usize, b_i: usize| -> (f64, f64, f64) {
        let diffs: Vec<f64> = (0..repeats)
            .map(|r| times[a_i][r] - times[b_i][r])
            .collect();
        median_min_max(&diffs)
    };

    // Indices:
    // 0: ProdA5Avx2Alone, 1: ProdA5ScalarAlone, 2: BitmapPass1Alone, 3: BitmapU128A5Alone,
    // 4: ProdA6aAlone, 5: A6aFromMaximal, 6: A6aFromPackedMax, 7: A6aNoOutq, 8: A6aMaximalNoOutq,
    // 9: A6aW1Fast, 10: ProdA5Avx2PlusA6a, 11: UnfoldedScalar, 12: FoldedScalar,
    // 13: UnfoldedBitmapU128, 14: FoldedBitmapU128,
    // 15: P2MtfCore, 16: P2MtfWithRuns, 17: P2Unfolded, 18: P2Folded, 19: P2FoldedNoOutq
    println!("\n--- Paired per-repeat ratios & structural decomposition ---");
    let (r_max_a6a, r_max_lo, r_max_hi) = paired_ratio(5, 4);
    let (r_pmax_a6a, r_pmax_lo, r_pmax_hi) = paired_ratio(6, 4);
    let (r_noq_a6a, r_noq_lo, r_noq_hi) = paired_ratio(7, 4);
    let (r_mnoq_a6a, r_mnoq_lo, r_mnoq_hi) = paired_ratio(8, 4);
    let (r_w1_a6a, r_w1_lo, r_w1_hi) = paired_ratio(9, 4);
    let (d_pmax_a6a, d_pmax_lo, d_pmax_hi) = paired_diff_ms(4, 6);

    println!(
        "A6a decomposition (vs prod_a6a_alone):\n  \
         a6a_from_maximal    / prod_a6a = {r_max_a6a:.3}x [{r_max_lo:.3}..{r_max_hi:.3}] (8B tuple maximal)\n  \
         a6a_from_packed_max / prod_a6a = {r_pmax_a6a:.3}x [{r_pmax_lo:.3}..{r_pmax_hi:.3}] (4B packed maximal: maximal() = {:.1}%, {d_pmax_a6a:+.2} ms [{d_pmax_lo:+.2}..{d_pmax_hi:+.2}])\n  \
         a6a_no_outq         / prod_a6a = {r_noq_a6a:.3}x [{r_noq_lo:.3}..{r_noq_hi:.3}] (OutQ16::push share   = {:.1}%)\n  \
         a6a_max_no_outq     / prod_a6a = {r_mnoq_a6a:.3}x [{r_mnoq_lo:.3}..{r_mnoq_hi:.3}] (classify+edges alone = {:.1}%)\n  \
         a6a_w1_fast         / prod_a6a = {r_w1_a6a:.3}x [{r_w1_lo:.3}..{r_w1_hi:.3}] (w==1 2B fast path    = {:+.1}%)",
        (1.0 - r_pmax_a6a) * 100.0,
        (1.0 - r_noq_a6a) * 100.0,
        r_mnoq_a6a * 100.0,
        (1.0 - r_w1_a6a) * 100.0,
    );

    let (r_fsc, r_fsc_lo, r_fsc_hi) = paired_ratio(12, 11);
    let (d_fsc, d_fsc_lo, d_fsc_hi) = paired_diff_ms(11, 12);
    let (r_fbm, r_fbm_lo, r_fbm_hi) = paired_ratio(14, 13);
    let (d_fbm, d_fbm_lo, d_fbm_hi) = paired_diff_ms(13, 14);
    let (r_p2f, r_p2f_lo, r_p2f_hi) = paired_ratio(18, 17);
    let (d_p2f, d_p2f_lo, d_p2f_hi) = paired_diff_ms(17, 18);
    let (d_a5_emit, d_a5_lo, d_a5_hi) = paired_diff_ms(16, 15);

    println!(
        "\nDirect A5+A6a fold (apples-to-apples, L2-hot intermediate `runs`):\n  \
         folded_scalar / unfolded_scalar           = {r_fsc:.3}x [{r_fsc_lo:.3}..{r_fsc_hi:.3}] (saved {d_fsc:+.2} ms [{d_fsc_lo:+.2}..{d_fsc_hi:+.2}], {:+.1}%)\n  \
         folded_bitmap_u128 / unfolded_bitmap_u128 = {r_fbm:.3}x [{r_fbm_lo:.3}..{r_fbm_hi:.3}] (saved {d_fbm:+.2} ms [{d_fbm_lo:+.2}..{d_fbm_hi:+.2}], {:+.1}%)\n  \
         p2_folded / p2_unfolded (pre-built bm)    = {r_p2f:.3}x [{r_p2f_lo:.3}..{r_p2f_hi:.3}] (saved {d_p2f:+.2} ms [{d_p2f_lo:+.2}..{d_p2f_hi:+.2}], {:+.1}%)\n  \
         A5 emit overhead (p2_with_runs - p2_core) = {d_a5_emit:+.2} ms [{d_a5_lo:+.2}..{d_a5_hi:+.2}]",
        (1.0 - r_fsc) * 100.0,
        (1.0 - r_fbm) * 100.0,
        (1.0 - r_p2f) * 100.0,
    );

    let (a5_avx2_med, _, _) = median_min_max(&times[0]);
    let (a6a_med, _, _) = median_min_max(&times[4]);
    let (p2_core_med, _, _) = median_min_max(&times[15]);
    let (p2_runs_med, _, _) = median_min_max(&times[16]);
    let (p2_unf_med, _, _) = median_min_max(&times[17]);
    let (p2_fld_med, _, _) = median_min_max(&times[18]);
    let (p2_fld_noq_med, _, _) = median_min_max(&times[19]);
    let (a6a_mnoq_med, _, _) = median_min_max(&times[8]);

    let prod_sum = a5_avx2_med + a6a_med;
    // Breakdown of p2 fold saving into:
    //  (1) A5 runs write + cap-127 split + counts/per = d_a5_emit
    //  (2) A6a maximal() rescan + merge = d_pmax_a6a
    //  (3) L2 round-trip + OoO ILP overlap = d_p2f - d_a5_emit - d_pmax_a6a
    let ilp_cache = d_p2f - d_a5_emit - d_pmax_a6a;
    // Conservative (non-ILP) floor = d_a5_emit + d_pmax_a6a
    let cons_save = d_a5_emit + d_pmax_a6a;
    let fold_frac_p2 = d_p2f / prod_sum;
    let fold_frac_cons = cons_save / prod_sum;
    let stage1_sum = I9_A5_STAGE1_1W_MS + I9_A6A_STAGE1_1W_MS;
    let ideal_sum = I9_M_IDEAL_1W_MS + I9_A6A_STAGE1_1W_MS;

    println!("\n--- Pass-2 surgical decomposition & cross-machine i9-10900K anchor ---");
    println!(
        "VM medians: prod_a5_avx2={a5_avx2_med:.2} ms, prod_a6a={a6a_med:.2} ms, sum={prod_sum:.2} ms;\n  \
         p2_mtf_core={p2_core_med:.2} ms, p2_mtf_with_runs={p2_runs_med:.2} ms, p2_unfolded={p2_unf_med:.2} ms, p2_folded={p2_fld_med:.2} ms\n  \
         ILP check (no OutQ16): p2_core + a6a_max_no_outq = {:.2} ms vs p2_folded_no_outq = {p2_fld_noq_med:.2} ms (overlap = {:+.2} ms)",
        p2_core_med + a6a_mnoq_med,
        (p2_core_med + a6a_mnoq_med) - p2_fld_noq_med
    );
    println!(
        "Pass-2 fold savings breakdown ({d_p2f:+.2} ms total = {:.1}% of prod A5+A6a):\n  \
         (1) A5 emit (cap-127 split + runs write + counts/per): {d_a5_emit:+.2} ms ({:.1}% of prod A5+A6a)\n  \
         (2) A6a maximal() rescan + merge (vs 4B packed max):   {d_pmax_a6a:+.2} ms ({:.1}% of prod A5+A6a)\n  \
         (3) L2 round-trip elimination + MTF/A6a OoO overlap:   {ilp_cache:+.2} ms ({:.1}% of prod A5+A6a)",
        fold_frac_p2 * 100.0,
        (d_a5_emit / prod_sum) * 100.0,
        (d_pmax_a6a / prod_sum) * 100.0,
        (ilp_cache / prod_sum) * 100.0,
    );
    println!(
        "Implied i9-10900K 1w savings on wikidata x4 (prod A5+A6a = {stage1_sum:.1} ms stage1 / {ideal_sum:.1} ms M-ideal+A6a):\n  \
         Full Pass-2 fold ({:.1}% of A5+A6a): {:.1} ms stage1 ({:.2}% 1w wall) / {:.1} ms ideal-anchor ({:.2}% 1w wall)\n  \
         Conservative floor (no extra ILP, {:.1}% of A5+A6a): {:.1} ms stage1 ({:.2}% 1w wall) / {:.1} ms ideal-anchor ({:.2}% 1w wall)",
        fold_frac_p2 * 100.0,
        fold_frac_p2 * stage1_sum,
        100.0 * fold_frac_p2 * stage1_sum / I9_WALL_1W_MS,
        fold_frac_p2 * ideal_sum,
        100.0 * fold_frac_p2 * ideal_sum / I9_WALL_1W_MS,
        fold_frac_cons * 100.0,
        fold_frac_cons * stage1_sum,
        100.0 * fold_frac_cons * stage1_sum / I9_WALL_1W_MS,
        fold_frac_cons * ideal_sum,
        100.0 * fold_frac_cons * ideal_sum / I9_WALL_1W_MS,
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage:");
        eprintln!("  {} stats FILE.bz2 [--forced]", args[0]);
        eprintln!("  {} bench FILE.bz2 [--forced] [--repeats N]", args[0]);
        std::process::exit(1);
    }
    let cmd = args[1].as_str();
    let path = args[2].as_str();
    let mut forced = false;
    let mut repeats = 7usize;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--forced" => forced = true,
            "--repeats" => {
                i += 1;
                repeats = args[i].parse().expect("valid --repeats");
            }
            other => {
                eprintln!("Unknown arg: {other}");
                std::process::exit(1);
            }
        }
        i += 1;
    }
    match cmd {
        "stats" => run_stats(path, forced),
        "bench" => run_bench(path, forced, repeats),
        _ => {
            eprintln!("Unknown subcommand: {cmd}");
            std::process::exit(1);
        }
    }
}
