#![forbid(unsafe_code)]
//! (research only, `explore`) Cold primary-evidence dissection of M (`src/runs.rs::front_avx2`),
//! A6a (`src/open.rs` + `src/open/outq.rs`), and the newly-merged `mtf-fold` (`src/runs/fold.rs`).
//!
//! Usage:
//!   cargo run --release --example mtf_a6a_dissection -- corpora/wikidata.bz2 [REPEATS]
//!   cargo run --release --features mtf-fold,trace --example mtf_a6a_dissection -- corpora/wikidata.bz2 [REPEATS]
//!
//! Zero `src/` edits, `#![forbid(unsafe_code)]`, 100% exactness verified before timing.

use mibz2::block::Symbols;
use mibz2::bwt::OpenCells;
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
#[cfg(feature = "mtf-fold")]
use mibz2::StreamDecoder;
use std::hint::black_box;
use std::time::Instant;

const CAP: u32 = 127;
const LIMIT: u32 = 16;
const INLINE: usize = 16;

const LAZY_DENSITY: (usize, usize) = (3, 20);
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

const NONE_CHUNK: u32 = u32::MAX;
const EPOCH: u32 = 1 << 16;

const IOTA4: u64 = 0x0003_0002_0001_0000;
const BROADCAST4: u64 = 0x0001_0001_0001_0001;

// ---- Linux on-CPU thread clock (no unsafe) ----

fn thread_cpu_ns() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/thread-self/schedstat").ok()?;
    let first = s.split_whitespace().next()?;
    first.parse::<u64>().ok()
}

// ---- Production admission mirror (src/cells_lazy.rs + src/open.rs) ----

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
    n >= 65_536 && records * 64 >= n && records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0
}

fn open_cap(records: usize, n: usize, p: u32) -> Option<Cap> {
    if n == 0 || n > 900_000 || p as usize >= n {
        None
    } else if lazy_counts_admit(records, n) {
        Some(Cap::NONE)
    } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
        Some(Cap::dense(n))
    } else {
        None
    }
}

// ---- Exact scalar records builder (matching src/runs.rs::front_scalar) ----

fn build_records_from_syms(b: &Symbols) -> (usize, Vec<u16>) {
    let mut runs = Vec::new();
    let mut list: [u8; 256] = std::array::from_fn(|i| i as u8);
    let (mut run, mut weight) = (0u32, 1u32);
    let mut n = 0usize;
    let mut open = (0u8, 0u32);
    let close = |open: (u8, u32), runs: &mut Vec<u16>| {
        let (c, mut left) = open;
        while left > 0 {
            let len = left.min(CAP);
            runs.push((c as u16) << 8 | len as u16);
            left -= len;
        }
    };
    for &s in &b.syms {
        if s <= 1 {
            run += (s as u32 + 1) * weight;
            weight <<= 1;
            continue;
        }
        if run != 0 {
            let c = b.seq_to_unseq[list[0] as usize];
            open = (c, open.1 + run);
            n += run as usize;
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        let nn = ((s - 1) as usize).min(255);
        let v = list[nn];
        list.copy_within(0..nn, 1);
        list[0] = v;
        let c = b.seq_to_unseq[v as usize];
        close(open, &mut runs);
        open = (c, 1);
        n += 1;
    }
    close(open, &mut runs);
    (n, runs)
}

fn build_packed_maximal_runs(records: &[u16]) -> Vec<u32> {
    let mut out = Vec::with_capacity(records.len());
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
        debug_assert!(width < (1 << 24));
        out.push((u32::from(c) << 24) | width);
    }
    out
}

struct OpenBlock {
    index: u64,
    syms: Symbols,
    n: usize,
    decoded_len: usize,
    records: Vec<u16>,
    packed_max: Vec<u32>,
    cap: Cap,
}

// ---- Exact A6a state & verification ----

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

#[derive(Default)]
struct Spill {
    next: Vec<u32>,
    ovf: Vec<Vec<u8>>,
    cuts: Vec<[u32; 256]>,
}

struct Heads {
    heads: [u32; 256],
    in_out: u32,
}

fn drain_all_queues<const C: usize, const S: usize>(
    out: &[u8],
    spill: &Spill,
    heads: &Heads,
    thin: &[u32; 256],
) -> Vec<u32> {
    let total: usize = thin.iter().map(|&x| x as usize).sum();
    let mut res = Vec::with_capacity(total);
    let sbytes = 2 * S;
    for (a, &count) in thin.iter().enumerate() {
        let mut left = count as usize;
        let mut id = heads.heads[a];
        let mut at = 0usize;
        let mut read = 0u32;
        let mut epoch = 0u32;
        let mut cut = spill.cuts.first().map_or(u32::MAX, |c| c[a]);
        while left > 0 {
            while read == cut {
                epoch += 1;
                cut = spill.cuts.get(epoch as usize).map_or(u32::MAX, |c| c[a]);
            }
            if at == C {
                id = spill.next[id as usize];
                at = 0;
            }
            let take = left.min(C - at).min((cut - read) as usize);
            let chunk: &[u8] = if id < heads.in_out {
                let off = id as usize * sbytes;
                &out[off..off + sbytes]
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
    }
    res
}

// ---- OutQ16 with configurable (C = usable ranks/chunk, S = stride ranks/chunk, WIDTH) ----
//
// WIDTH:
//   0 = shipped 32B iota16 (`w <= 16`)
//   1 = 8B u64 SWAR iota4 (`w <= 4`, 92.6% of thin runs) -> 32B iota16 (`5..=16`)
//   2 = 16B [u16; 8] iota8 (`w <= 8`, 96.3% of thin runs) -> 32B iota16 (`9..=16`)

#[derive(Default, Clone, Copy, Debug)]
struct SlowStats {
    total_pushes: u64,
    slow_calls: u64,
    slow_first_chunk: u64,
    slow_tail_guard: u64,
    slow_chunk_cross: u64,
    slow_epoch_cross: u64,
    slow_ovf_tail: u64,
    chunks_taken: u64,
    ovf_chunks: u64,
    max_ovf_in_blk: usize,
    blks_with_ovf: usize,
}

struct Queues<'a, const C: usize, const S: usize> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    chunks: u32,
    heads: [u32; 256],
    tails: [u32; 256],
    fill: [u32; 256],
    limit: u32,
    over: bool,
}

impl<'a, const C: usize, const S: usize> Queues<'a, C, S> {
    fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
        let sbytes = 2 * S;
        spill.ovf.clear();
        spill.cuts.clear();
        Queues {
            in_out: (out.len() / sbytes) as u32,
            out,
            spill,
            chunks: 0,
            heads: [NONE_CHUNK; 256],
            tails: [NONE_CHUNK; 256],
            fill: [C as u32; 256],
            limit,
            over: false,
        }
    }

    fn heads(&self) -> Heads {
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }

    #[inline(always)]
    fn push<const GUARD: bool, const WIDTH: u8>(
        &mut self,
        a: usize,
        m: u32,
        w: u32,
        thin: &[u32; 256],
    ) {
        let sbytes = 2 * S;
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        match WIDTH {
            0 => {
                let fits = if GUARD {
                    f + (w as usize) <= C
                } else {
                    f + 16 <= C
                };
                if w <= 16 && fits && lo + w < EPOCH && tail < self.in_out {
                    let at = tail as usize * sbytes + 2 * f;
                    let iota: [u16; 16] =
                        std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                    for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                        d.copy_from_slice(&v.to_le_bytes());
                    }
                    self.fill[a] = (f as u32) + w;
                } else {
                    self.push_slow(a, m, w, thin);
                }
            }
            1 => {
                if w <= 4 && f + 4 <= C && lo + w < EPOCH && tail < self.in_out {
                    let at = tail as usize * sbytes + 2 * f;
                    let v = (lo as u64).wrapping_mul(BROADCAST4).wrapping_add(IOTA4);
                    self.out[at..at + 8].copy_from_slice(&v.to_le_bytes());
                    self.fill[a] = (f as u32) + w;
                } else if w <= 16 && f + 16 <= C && lo + w < EPOCH && tail < self.in_out {
                    let at = tail as usize * sbytes + 2 * f;
                    let iota: [u16; 16] =
                        std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                    for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                        d.copy_from_slice(&v.to_le_bytes());
                    }
                    self.fill[a] = (f as u32) + w;
                } else {
                    self.push_slow(a, m, w, thin);
                }
            }
            _ => {
                if w <= 8 && f + 8 <= C && lo + w < EPOCH && tail < self.in_out {
                    let at = tail as usize * sbytes + 2 * f;
                    let iota: [u16; 8] =
                        std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                    for (d, v) in self.out[at..at + 16].chunks_exact_mut(2).zip(iota) {
                        d.copy_from_slice(&v.to_le_bytes());
                    }
                    self.fill[a] = (f as u32) + w;
                } else if w <= 16 && f + 16 <= C && lo + w < EPOCH && tail < self.in_out {
                    let at = tail as usize * sbytes + 2 * f;
                    let iota: [u16; 16] =
                        std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                    for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                        d.copy_from_slice(&v.to_le_bytes());
                    }
                    self.fill[a] = (f as u32) + w;
                } else {
                    self.push_slow(a, m, w, thin);
                }
            }
        }
    }

    fn push_counted<const GUARD: bool>(
        &mut self,
        a: usize,
        m: u32,
        w: u32,
        thin: &[u32; 256],
        st: &mut SlowStats,
    ) {
        st.total_pushes += 1;
        let sbytes = 2 * S;
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        let fits = if GUARD {
            f + (w as usize) <= C
        } else {
            f + 16 <= C
        };
        if w <= 16 && fits && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * sbytes + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else {
            st.slow_calls += 1;
            if tail == NONE_CHUNK {
                st.slow_first_chunk += 1;
            } else if tail >= self.in_out {
                st.slow_ovf_tail += 1;
            } else if lo + w >= EPOCH {
                st.slow_epoch_cross += 1;
            } else if f + (w as usize) <= C {
                st.slow_tail_guard += 1;
            } else {
                st.slow_chunk_cross += 1;
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
            if self.fill[a] as usize == C {
                self.take_chunk(a);
            }
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let take = left.min((C - f) as u32).min(EPOCH - lo);
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
        let sbytes = 2 * S;
        let id = self.chunks;
        self.chunks += 1;
        if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
            self.spill.ovf.push(vec![0; sbytes]);
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
        let sbytes = 2 * S;
        if id < self.in_out {
            let at = id as usize * sbytes;
            &mut self.out[at..at + sbytes]
        } else {
            &mut self.spill.ovf[(id - self.in_out) as usize][..]
        }
    }
}

// ---- Exact mirror of shipped A6a Pass (src/open.rs) ----

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
fn split(
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

struct Pass<'a, const C: usize, const S: usize> {
    queues: Queues<'a, C, S>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    f: First,
}

impl<const C: usize, const S: usize> Pass<'_, C, S> {
    #[inline(always)]
    fn take<const QUEUE: bool, const GUARD: bool, const WIDTH: u8>(
        &mut self,
        c: u8,
        pos: u32,
        width: u32,
        p: u32,
        limit: u32,
    ) {
        self.f.runs += 1;
        if width <= limit {
            self.piece::<QUEUE, GUARD, WIDTH>(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece::<QUEUE, GUARD, WIDTH>(c, pos, width, false, p);
        } else {
            self.take_marked::<QUEUE, GUARD, WIDTH>(c, pos, width, p, limit);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked<const QUEUE: bool, const GUARD: bool, const WIDTH: u8>(
        &mut self,
        c: u8,
        pos: u32,
        width: u32,
        p: u32,
        limit: u32,
    ) {
        split(c, pos, width, p, limit, |c, x, w, keep| {
            self.piece::<QUEUE, GUARD, WIDTH>(c, x, w, keep, p)
        });
    }

    #[inline(always)]
    fn piece<const QUEUE: bool, const GUARD: bool, const WIDTH: u8>(
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
                self.queues.push::<GUARD, WIDTH>(a, f.m, w, &f.thin);
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

#[inline(never)]
fn run_a6a<
    const C: usize,
    const S: usize,
    const GUARD: bool,
    const WIDTH: u8,
    const QUEUE: bool,
>(
    records: &[u16],
    n: usize,
    p: u32,
    cap: Cap,
    out: &mut Vec<u8>,
    edges: &mut Vec<u32>,
    spill: &mut Spill,
) -> (First, usize, Heads) {
    out.resize(n + INLINE, 0);
    let mut pass = Pass::<C, S> {
        queues: Queues::<C, S>::new(&mut out[..], spill, cap.m),
        edges,
        ns: 0,
        cap_strips: cap.strips,
        f: First::default(),
    };
    maximal(records, |c, pos, width| {
        pass.take::<QUEUE, GUARD, WIDTH>(c, pos, width, p, LIMIT)
    });
    let ns = pass.ns;
    let heads = pass.queues.heads();
    (pass.f, ns, heads)
}

#[inline(never)]
fn run_a6a_packed_max<const C: usize, const S: usize, const GUARD: bool>(
    packed_max: &[u32],
    n: usize,
    p: u32,
    cap: Cap,
    out: &mut Vec<u8>,
    edges: &mut Vec<u32>,
    spill: &mut Spill,
) -> (First, usize, Heads) {
    out.resize(n + INLINE, 0);
    let mut pass = Pass::<C, S> {
        queues: Queues::<C, S>::new(&mut out[..], spill, cap.m),
        edges,
        ns: 0,
        cap_strips: cap.strips,
        f: First::default(),
    };
    let mut pos = 0u32;
    for &rec in packed_max {
        let c = (rec >> 24) as u8;
        let w = rec & 0x00ff_ffff;
        pass.take::<true, GUARD, 0>(c, pos, w, p, LIMIT);
        pos += w;
    }
    let ns = pass.ns;
    let heads = pass.queues.heads();
    (pass.f, ns, heads)
}

// ---- Structural census of M (runs::front_avx2) ----

#[derive(Default)]
struct MtfCensus {
    total_syms: u64,
    run_syms: u64,
    lit_syms: u64,
    eob_syms: u64,
    nn_eq_1: u64,
    nn_le_3: u64,
    nn_le_7: u64,
    nn_le_15: u64,
    nn_16_31: u64,
    nn_32_63: u64,
    nn_ge_64: u64,
    pred2_miss_16: u64,
    pred2_miss_32: u64,
    span_1: u64,
    span_2: u64,
    span_3: u64,
    span_4_7: u64,
    span_8_22: u64,
    lit_words_nonempty: u64,
    lit_words_total: u64,
    adj_same_byte: u64,
    syms_bytes: Vec<usize>,
}

fn collect_mtf_census(blocks: &[OpenBlock]) -> MtfCensus {
    let mut st = MtfCensus::default();
    for blk in blocks {
        let b = &blk.syms;
        st.syms_bytes.push(b.syms.len() * 2);
        let mut cnt2_16: u8 = 0;
        let mut cnt2_32: u8 = 0;
        let mut last_lit_pos: Option<usize> = None;
        let mut word_has_lit = false;
        let mut cur_word = 0usize;
        for (idx, &s) in b.syms.iter().enumerate() {
            let w = idx / 64;
            if w != cur_word {
                st.lit_words_total += 1;
                if word_has_lit {
                    st.lit_words_nonempty += 1;
                }
                cur_word = w;
                word_has_lit = false;
            }
            st.total_syms += 1;
            if s <= 1 {
                st.run_syms += 1;
                continue;
            }
            if s == b.eob {
                st.eob_syms += 1;
                if let Some(prev) = last_lit_pos {
                    let span = idx - prev;
                    match span {
                        1 => st.span_1 += 1,
                        2 => st.span_2 += 1,
                        3 => st.span_3 += 1,
                        4..=7 => st.span_4_7 += 1,
                        _ => st.span_8_22 += 1,
                    }
                }
                break;
            }
            word_has_lit = true;
            st.lit_syms += 1;
            if let Some(prev) = last_lit_pos {
                let span = idx - prev;
                match span {
                    1 => st.span_1 += 1,
                    2 => st.span_2 += 1,
                    3 => st.span_3 += 1,
                    4..=7 => st.span_4_7 += 1,
                    _ => st.span_8_22 += 1,
                }
            }
            last_lit_pos = Some(idx);

            let nn = ((s - 1) as usize).min(255);
            if nn == 1 {
                st.nn_eq_1 += 1;
            }
            if nn <= 3 {
                st.nn_le_3 += 1;
            }
            if nn <= 7 {
                st.nn_le_7 += 1;
            }
            if nn <= 15 {
                st.nn_le_15 += 1;
            } else if nn <= 31 {
                st.nn_16_31 += 1;
            } else if nn <= 63 {
                st.nn_32_63 += 1;
            } else {
                st.nn_ge_64 += 1;
            }

            let cur16 = nn >= 16;
            if (cnt2_16 >= 2) != cur16 {
                st.pred2_miss_16 += 1;
            }
            cnt2_16 = if cur16 {
                (cnt2_16 + 1).min(3)
            } else {
                cnt2_16.saturating_sub(1)
            };

            let cur32 = nn >= 32;
            if (cnt2_32 >= 2) != cur32 {
                st.pred2_miss_32 += 1;
            }
            cnt2_32 = if cur32 {
                (cnt2_32 + 1).min(3)
            } else {
                cnt2_32.saturating_sub(1)
            };
        }
        st.lit_words_total += 1;
        if word_has_lit {
            st.lit_words_nonempty += 1;
        }
        for w in blk.packed_max.windows(2) {
            if (w[0] >> 24) == (w[1] >> 24) {
                st.adj_same_byte += 1;
            }
        }
    }
    st
}

fn print_out_resize_census(blocks: &[OpenBlock]) {
    let mut seq_shipped_zero: u64 = 0;
    let mut seq_fold_zero: u64 = 0;
    let mut pool8_shipped_zero: u64 = 0;
    let mut pool8_fold_zero: u64 = 0;
    let mut cur_s = 0usize;
    let mut cur_f = 0usize;
    let mut shrunk = 0usize;
    let mut equal = 0usize;
    let mut expanded = 0usize;
    let mut pool_s: Vec<usize> = Vec::new();
    let mut pool_f: Vec<usize> = Vec::new();

    for blk in blocks {
        match blk.decoded_len.cmp(&blk.n) {
            std::cmp::Ordering::Less => shrunk += 1,
            std::cmp::Ordering::Equal => equal += 1,
            std::cmp::Ordering::Greater => expanded += 1,
        }
        let ns = blk.n + INLINE;
        let nf = blk.syms.max_len + INLINE;
        seq_shipped_zero += ns.saturating_sub(cur_s) as u64;
        cur_s = blk.decoded_len;
        seq_fold_zero += nf.saturating_sub(cur_f) as u64;
        cur_f = blk.decoded_len;

        let ps = if pool_s.len() >= 8 {
            pool_s.remove(0)
        } else {
            0
        };
        pool8_shipped_zero += ns.saturating_sub(ps) as u64;
        pool_s.push(blk.decoded_len);

        let pf = if pool_f.len() >= 8 {
            pool_f.remove(0)
        } else {
            0
        };
        pool8_fold_zero += nf.saturating_sub(pf) as u64;
        pool_f.push(blk.decoded_len);
    }

    println!(
        "\n=== 2. Output buffer resize(..., 0) zero-fill census ({} Open blocks) ===",
        blocks.len()
    );
    println!(
        "  RLE1 vs BWT n: shrunk={} ({:.1}%), equal={} ({:.1}%), expanded={} ({:.1}%)",
        shrunk,
        100.0 * shrunk as f64 / blocks.len() as f64,
        equal,
        100.0 * equal as f64 / blocks.len() as f64,
        expanded,
        100.0 * expanded as f64 / blocks.len() as f64,
    );
    println!(
        "  1-buf reuse zeroed: shipped={:.2} MiB | mtf-fold={:.2} MiB",
        seq_shipped_zero as f64 / 1048576.0,
        seq_fold_zero as f64 / 1048576.0,
    );
    println!(
        "  8-buf pool  zeroed: shipped={:.2} MiB | mtf-fold={:.2} MiB",
        pool8_shipped_zero as f64 / 1048576.0,
        pool8_fold_zero as f64 / 1048576.0,
    );
}

fn collect_slow_stats<const C: usize, const S: usize, const GUARD: bool>(
    blocks: &[OpenBlock],
) -> SlowStats {
    let mut st = SlowStats::default();
    let mut out = Vec::with_capacity(950_000);
    let mut spill = Spill::default();
    for blk in blocks {
        out.resize(blk.n + INLINE, 0);
        let mut q = Queues::<C, S>::new(&mut out[..], &mut spill, blk.cap.m);
        let mut thin = [0u32; 256];
        let mut m = 0u32;
        let p = blk.syms.orig_ptr;
        maximal(&blk.records, |c, pos, width| {
            split(c, pos, width, p, LIMIT, |cc, _, w, keep| {
                if keep {
                    let a = usize::from(cc);
                    q.push_counted::<GUARD>(a, m, w, &thin, &mut st);
                    thin[a] += w;
                    m += w;
                }
            });
        });
        st.chunks_taken += q.chunks as u64;
        let ovf = q.spill.ovf.len();
        st.ovf_chunks += ovf as u64;
        if ovf > 0 {
            st.blks_with_ovf += 1;
            st.max_ovf_in_blk = st.max_ovf_in_blk.max(ovf);
        }
    }
    st
}

struct Stats {
    min: f64,
    med: f64,
    max: f64,
}

fn summarize(vals: &[f64]) -> Stats {
    let mut v = vals.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let mid = v.len() / 2;
    let med = if v.len() % 2 == 1 {
        v[mid]
    } else {
        0.5 * (v[mid - 1] + v[mid])
    };
    Stats {
        min: v[0],
        med,
        max: *v.last().unwrap(),
    }
}

#[cfg(feature = "mtf-fold")]
fn t_crit_95(df: usize) -> f64 {
    // Two-sided 95% Student-t critical values (exact to 4 d.p.; normal 1.960 for large df).
    match df {
        0 | 1 => 12.7062,
        2 => 4.3027,
        3 => 3.1824,
        4 => 2.7764,
        5 => 2.5706,
        6 => 2.4469,
        7 => 2.3646,
        8 => 2.3060,
        9 => 2.2622,
        10 => 2.2281,
        11 => 2.2010,
        12 => 2.1788,
        13 => 2.1604,
        14 => 2.1448,
        15 => 2.1314,
        16 => 2.1199,
        17 => 2.1098,
        18 => 2.1009,
        19 => 2.0930,
        20 => 2.0860,
        21..=24 => 2.0687,
        25..=29 => 2.0484,
        30 => 2.0423,
        31..=39 => 2.0301,
        40..=59 => 2.0086,
        60..=119 => 1.9840,
        _ => 1.9600,
    }
}

#[cfg(feature = "mtf-fold")]
fn paired_geomean_ci(off: &[f64], on: &[f64]) -> (f64, f64, f64) {
    let logs: Vec<f64> = off
        .iter()
        .zip(on.iter())
        .filter(|(&a, &b)| a > 0.0 && b > 0.0)
        .map(|(&a, &b)| (b / a).ln())
        .collect();
    if logs.is_empty() {
        return (1.0, 1.0, 1.0);
    }
    let n = logs.len();
    let mean = logs.iter().sum::<f64>() / n as f64;
    if n < 2 {
        let g = mean.exp();
        return (g, g, g);
    }
    let var = logs.iter().map(|&x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1) as f64;
    let se = (var / n as f64).sqrt();
    let t = t_crit_95(n - 1);
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

#[cfg(feature = "mtf-fold")]
#[derive(Default, Clone, Copy)]
struct FoldRunSample {
    wall_ms: f64,
    #[cfg(feature = "trace")]
    wkr_cpu_ms: f64,
    #[cfg(feature = "trace")]
    wkr_run_ms: f64,
    #[cfg(feature = "trace")]
    wkr_wall_ms: f64,
    #[cfg(feature = "trace")]
    a5_ms: f64,
    #[cfg(feature = "trace")]
    a5_a6a_ms: f64,
    #[cfg(feature = "trace")]
    a6bc_a7_ms: f64,
    #[cfg(feature = "trace")]
    inv_ms: f64,
    #[cfg(feature = "trace")]
    a5_plus_inv_ms: f64,
    #[cfg(feature = "trace")]
    a9_ms: f64,
}

#[cfg(feature = "mtf-fold")]
fn run_mtf_fold_e2e(
    compressed: &[u8],
    expected_len: usize,
    expected_crc: u32,
    repeats: usize,
    worker_list: &[usize],
) {
    println!(
        "\n=== 5. Real mtf-fold end-to-end & worker-trace dissection ({repeats} interleaved repeats) ==="
    );
    let k = Kernels::detect();
    for &workers in worker_list {
        let mut off_s = Vec::with_capacity(repeats);
        let mut on_s = Vec::with_capacity(repeats);

        let run_once =
            |fold_on: bool, verify_crc: bool| -> (FoldRunSample, mibz2::runs::fold::Counts) {
                mibz2::runs::fold::set_enabled(fold_on);
                let _ = mibz2::runs::fold::take_counts();
                #[cfg(feature = "trace")]
                mibz2::trace::set_level(2);

                let t0 = Instant::now();
                {
                    let dec = StreamDecoder::workers_with::<OpenCells>(workers, 4 * workers, k);
                    std::thread::scope(|s| {
                        let dec_r = &dec;
                        let h = s.spawn(move || {
                            let mut crc = !0u32;
                            let mut len = 0usize;
                            let mut spent = Vec::new();
                            while let Some(blk) = dec_r.read_block(spent).expect("read_block") {
                                if verify_crc {
                                    crc = mibz2::crc::update_with(crc, &blk, k);
                                }
                                len += blk.len();
                                spent = blk;
                            }
                            (len, !crc)
                        });
                        for chunk in compressed.chunks(4 << 20) {
                            let mut buf = dec.spare();
                            buf.clear();
                            buf.extend_from_slice(chunk);
                            dec.write(buf).expect("write");
                        }
                        dec.close();
                        let (len, crc) = h.join().expect("join");
                        assert_eq!(len, expected_len, "length mismatch (fold={fold_on})");
                        if verify_crc {
                            assert_eq!(crc, expected_crc, "CRC mismatch (fold={fold_on})");
                        }
                    });
                    dec.join();
                }
                let wall_ms = t0.elapsed().as_secs_f64() * 1e3;
                let counts = mibz2::runs::fold::take_counts();

                #[cfg(feature = "trace")]
                let mut sample = FoldRunSample {
                    wall_ms,
                    ..FoldRunSample::default()
                };
                #[cfg(not(feature = "trace"))]
                let sample = FoldRunSample { wall_ms };

                #[cfg(feature = "trace")]
                {
                    mibz2::trace::set_level(0);
                    let traces = mibz2::trace::take();
                    let mut wkr_cpu_ns = 0u64;
                    let mut wkr_run_ns = 0u64;
                    let mut wkr_wall_ns = 0u64;
                    let mut a5_ns = 0u64;
                    let mut a5_a6a_ns = 0u64;
                    let mut a6bc_a7_ns = 0u64;
                    let mut inv_ns = 0u64;
                    let mut a9_ns = 0u64;
                    for tr in &traces {
                        if !tr
                            .marks
                            .iter()
                            .any(|m| m.kind == mibz2::trace::Kind::DecodeStart)
                        {
                            continue;
                        }
                        let mut dec_start_ns = 0u64;
                        let mut dec_start_cpu = 0u64;
                        let mut dec_start_run = 0u64;
                        let mut inv_start_ns = 0u64;
                        let mut pass1_end_ns = 0u64;
                        let mut inv_end_ns = 0u64;
                        let mut have_inv = false;
                        let mut have_p1 = false;
                        for m in &tr.marks {
                            match m.kind {
                                mibz2::trace::Kind::DecodeStart => {
                                    dec_start_ns = m.ns;
                                    dec_start_cpu = m.cpu_ns;
                                    dec_start_run = m.run_ns;
                                    have_inv = false;
                                    have_p1 = false;
                                }
                                mibz2::trace::Kind::InvertStart => {
                                    inv_start_ns = m.ns;
                                    if !have_inv {
                                        a5_ns += m.ns.saturating_sub(dec_start_ns);
                                    }
                                    have_inv = true;
                                    have_p1 = false;
                                }
                                mibz2::trace::Kind::OpenPass1End => {
                                    if have_inv {
                                        pass1_end_ns = m.ns;
                                        a5_a6a_ns += m.ns.saturating_sub(dec_start_ns);
                                        have_p1 = true;
                                    }
                                }
                                mibz2::trace::Kind::InvertEnd => {
                                    inv_end_ns = m.ns;
                                    if have_inv {
                                        inv_ns += m.ns.saturating_sub(inv_start_ns);
                                        if have_p1 {
                                            a6bc_a7_ns += m.ns.saturating_sub(pass1_end_ns);
                                        }
                                    }
                                }
                                mibz2::trace::Kind::DecodeEnd => {
                                    wkr_wall_ns += m.ns.saturating_sub(dec_start_ns);
                                    if m.cpu_ns != mibz2::trace::UNREAD
                                        && dec_start_cpu != mibz2::trace::UNREAD
                                    {
                                        wkr_cpu_ns += m.cpu_ns.saturating_sub(dec_start_cpu);
                                    }
                                    if m.run_ns != mibz2::trace::UNREAD
                                        && dec_start_run != mibz2::trace::UNREAD
                                    {
                                        wkr_run_ns += m.run_ns.saturating_sub(dec_start_run);
                                    }
                                    if have_inv {
                                        a9_ns += m.ns.saturating_sub(inv_end_ns);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    sample.wkr_cpu_ms = wkr_cpu_ns as f64 * 1e-6;
                    sample.wkr_run_ms = wkr_run_ns as f64 * 1e-6;
                    sample.wkr_wall_ms = wkr_wall_ns as f64 * 1e-6;
                    sample.a5_ms = a5_ns as f64 * 1e-6;
                    sample.a5_a6a_ms = a5_a6a_ns as f64 * 1e-6;
                    sample.a6bc_a7_ms = a6bc_a7_ns as f64 * 1e-6;
                    sample.inv_ms = inv_ns as f64 * 1e-6;
                    sample.a5_plus_inv_ms = (a5_ns + inv_ns) as f64 * 1e-6;
                    sample.a9_ms = a9_ns as f64 * 1e-6;
                }
                (sample, counts)
            };

        let (_, c_off) = run_once(false, true);
        let (_, c_on) = run_once(true, true);
        println!(
            "  workers={workers}: exactness OK (len={expected_len}, crc={expected_crc:#010x}) | OFF={c_off:?} | ON={c_on:?}"
        );

        for r in 0..repeats {
            let (a, b) = if r % 2 == 0 {
                let a = run_once(false, false).0;
                let b = run_once(true, false).0;
                (a, b)
            } else {
                let b = run_once(true, false).0;
                let a = run_once(false, false).0;
                (a, b)
            };
            off_s.push(a);
            on_s.push(b);
        }

        let report = |label: &str, f: fn(&FoldRunSample) -> f64| {
            let vo: Vec<f64> = off_s.iter().map(f).collect();
            let vn: Vec<f64> = on_s.iter().map(f).collect();
            let vr: Vec<f64> = (0..repeats).map(|i| vn[i] / vo[i]).collect();
            let so = summarize(&vo);
            let sn = summarize(&vn);
            let sr = summarize(&vr);
            let (gm, lo, hi) = paired_geomean_ci(&vo, &vn);
            println!(
                "    {:14}: OFF {:7.2} [{:6.2}..{:6.2}] | ON {:7.2} [{:6.2}..{:6.2}] | med={:.4}x | gm={:.4}x 95%CI=[{:.4}, {:.4}] ({:+.2}%)",
                label,
                so.med,
                so.min,
                so.max,
                sn.med,
                sn.min,
                sn.max,
                sr.med,
                gm,
                lo,
                hi,
                (gm - 1.0) * 100.0
            );
        };
        report("wall_ms", |x| x.wall_ms);
        #[cfg(feature = "trace")]
        {
            report("wkr_cpu_ms", |x| x.wkr_cpu_ms);
            report("wkr_sched_ms", |x| x.wkr_run_ms);
            report("wkr_wall_ms", |x| x.wkr_wall_ms);
            report("A5+A6a_ms", |x| x.a5_a6a_ms);
            report("A6bc+A7_ms", |x| x.a6bc_a7_ms);
            report("A5+A6+A7_ms", |x| x.a5_plus_inv_ms);
            report("A5_ms", |x| x.a5_ms);
            report("A6+A7_ms", |x| x.inv_ms);
            report("A9_ms", |x| x.a9_ms);
        }
    }
    mibz2::runs::fold::set_enabled(true);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <file.bz2> [repeats] [--e2e-only] [--workers W]",
            args[0]
        );
        std::process::exit(2);
    }
    let path = &args[1];
    let mut repeats: usize = 7;
    let mut e2e_only = false;
    let mut worker_override: Option<usize> = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--e2e-only" => e2e_only = true,
            "--workers" => {
                i += 1;
                worker_override = args.get(i).and_then(|s| s.parse().ok());
            }
            s => {
                if let Ok(n) = s.parse::<usize>() {
                    repeats = n;
                }
            }
        }
        i += 1;
    }
    let compressed = std::fs::read(path).expect("read bz2");
    let k = Kernels::detect();
    println!(
        "Corpus: {path} ({} bytes), repeats={repeats}, kernels={k:?}",
        compressed.len()
    );

    let ref_out = mibz2::decode(&compressed).expect("decode reference");
    let ref_crc = mibz2::crc::crc32(&ref_out, k);
    println!(
        "Reference decode: {} bytes ({:.2} MiB), crc={ref_crc:#010x}",
        ref_out.len(),
        ref_out.len() as f64 / 1048576.0
    );

    if e2e_only {
        #[cfg(feature = "mtf-fold")]
        {
            let ws = match worker_override {
                Some(w) => vec![w],
                None => vec![1, 2],
            };
            run_mtf_fold_e2e(&compressed, ref_out.len(), ref_crc, repeats, &ws);
        }
        #[cfg(not(feature = "mtf-fold"))]
        {
            let _ = worker_override;
            println!("(Re-run with `--features mtf-fold,trace` for --e2e-only.)");
        }
        return;
    }

    let mut sc = Scanner::new(SliceSource(&compressed));
    let mut blocks: Vec<OpenBlock> = Vec::new();
    let mut total_blocks = 0usize;
    let mut runs_route_blocks = 0usize;
    let mut tmp_runs = Runs::default();
    let mut tmp_inv = OpenCells::default();
    let mut tmp_out = Vec::new();
    let mut tmp_dec = Vec::new();

    loop {
        match sc.next(Vec::new()).expect("scan") {
            Event::Done => break,
            Event::Block { index, symbols, .. } => {
                total_blocks += 1;
                if runs::takes_runs(&symbols) {
                    runs_route_blocks += 1;
                    let n = runs::front(&symbols, &mut tmp_runs, k).expect("front");
                    let (n2, records) = build_records_from_syms(&symbols);
                    assert_eq!(n, n2);
                    if let Some(cap) = open_cap(records.len(), n, symbols.orig_ptr) {
                        tmp_out.clear();
                        let ok = mibz2::bwt::InverseBwt::invert_runs(
                            &mut tmp_inv,
                            &tmp_runs,
                            n,
                            symbols.orig_ptr,
                            &mut tmp_out,
                            k,
                        );
                        assert!(ok);
                        tmp_dec.clear();
                        mibz2::rle::unrle(&tmp_out, &mut tmp_dec, k).expect("unrle");
                        let crc = mibz2::crc::crc32(&tmp_dec, k);
                        assert_eq!(crc, symbols.crc);
                        let decoded_len = tmp_dec.len();
                        let packed_max = build_packed_maximal_runs(&records);
                        blocks.push(OpenBlock {
                            index,
                            syms: symbols,
                            n,
                            decoded_len,
                            records,
                            packed_max,
                            cap,
                        });
                    }
                }
            }
        }
    }

    println!(
        "Blocks: total={total_blocks}, takes_runs={runs_route_blocks}, Open-admitted={}",
        blocks.len()
    );

    // ---- 1. MTF structural census ----
    let st = collect_mtf_census(&blocks);
    let lits = st.lit_syms as f64;
    let mut syms_b = st.syms_bytes.clone();
    syms_b.sort_unstable();
    let p50 = syms_b[syms_b.len() / 2];
    let p90 = syms_b[syms_b.len() * 9 / 10];
    let pmax = *syms_b.last().unwrap();

    println!(
        "\n=== 1. M (runs::front_avx2) structural census ({} Open blocks) ===",
        blocks.len()
    );
    println!(
        "  symbols: total={} (run={} [{:.2}%], lit={} [{:.2}%], eob={})",
        st.total_syms,
        st.run_syms,
        100.0 * st.run_syms as f64 / st.total_syms as f64,
        st.lit_syms,
        100.0 * st.lit_syms as f64 / st.total_syms as f64,
        st.eob_syms,
    );
    println!(
        "  syms KiB/block: p50={:.1}, p90={:.1}, max={:.1} (total {:.2} MiB; bitmaps {:.2} MiB)",
        p50 as f64 / 1024.0,
        p90 as f64 / 1024.0,
        pmax as f64 / 1024.0,
        (st.total_syms * 2) as f64 / 1048576.0,
        (st.total_syms / 4) as f64 / 1048576.0,
    );
    println!(
        "  nn=s-1: ==1 {:.2}%, <=3 {:.2}%, <=7 {:.2}%, <=15 {:.2}%, 16..31 {:.2}%, 32..63 {:.2}%, >=64 {:.2}%",
        100.0 * st.nn_eq_1 as f64 / lits,
        100.0 * st.nn_le_3 as f64 / lits,
        100.0 * st.nn_le_7 as f64 / lits,
        100.0 * st.nn_le_15 as f64 / lits,
        100.0 * st.nn_16_31 as f64 / lits,
        100.0 * st.nn_32_63 as f64 / lits,
        100.0 * st.nn_ge_64 as f64 / lits,
    );
    println!(
        "  2-bit branch miss: (s < 17) {:.2}% | (s < 33) {:.2}% | adj_same_byte={}",
        100.0 * st.pred2_miss_16 as f64 / lits,
        100.0 * st.pred2_miss_32 as f64 / lits,
        st.adj_same_byte,
    );
    let spans = (st.span_1 + st.span_2 + st.span_3 + st.span_4_7 + st.span_8_22) as f64;
    println!(
        "  lit-to-lit spans: 1={:.2}%, 2={:.2}%, 3={:.2}%, 4..7={:.2}%, 8..22={:.2}% | 64-sym boundary={:.2}%",
        100.0 * st.span_1 as f64 / spans,
        100.0 * st.span_2 as f64 / spans,
        100.0 * st.span_3 as f64 / spans,
        100.0 * st.span_4_7 as f64 / spans,
        100.0 * st.span_8_22 as f64 / spans,
        100.0 * st.lit_words_nonempty as f64 / lits,
    );

    // ---- 2. Output buffer resize census ----
    print_out_resize_census(&blocks);

    // ---- 3. OutQ16 push_slow & guard-zone census ----
    println!("\n=== 3. OutQ16 push_slow & chunk-guard census (233 Open blocks) ===");
    let show_slow = |name: &str, s: SlowStats| {
        println!(
            "  {:18} pushes={} | slow={:7} ({:5.2}%) [first={:6}, tail_guard={:6}, cross={:6}, epoch={:5}, ovf={:5}] | chunks={:6}, ovf_chunks={:5}",
            name,
            s.total_pushes,
            s.slow_calls,
            100.0 * s.slow_calls as f64 / s.total_pushes as f64,
            s.slow_first_chunk,
            s.slow_tail_guard,
            s.slow_chunk_cross,
            s.slow_epoch_cross,
            s.slow_ovf_tail,
            s.chunks_taken,
            s.ovf_chunks,
        );
    };
    show_slow(
        "shipped_512_512",
        collect_slow_stats::<512, 512, false>(&blocks),
    );
    show_slow(
        "guard_496_512",
        collect_slow_stats::<496, 512, true>(&blocks),
    );
    show_slow(
        "plain_1024_1024",
        collect_slow_stats::<1024, 1024, false>(&blocks),
    );
    show_slow(
        "guard_1008_1024",
        collect_slow_stats::<1008, 1024, true>(&blocks),
    );

    // ---- 4. Exactness check & interleaved A5/A6a timings ----
    println!("\n=== 4. Exactness check & interleaved A5/A6a timings ({repeats} repeats) ===");
    {
        let mut out_ref = Vec::with_capacity(950_000);
        let mut edges_ref = Vec::new();
        let mut spill_ref = Spill::default();
        let mut out_c = Vec::with_capacity(950_000);
        let mut edges_c = Vec::new();
        let mut spill_c = Spill::default();

        for blk in &blocks {
            let (f_ref, ns_ref, h_ref) = run_a6a::<512, 512, false, 0, true>(
                &blk.records,
                blk.n,
                blk.syms.orig_ptr,
                blk.cap,
                &mut out_ref,
                &mut edges_ref,
                &mut spill_ref,
            );
            let q_ref = drain_all_queues::<512, 512>(&out_ref, &spill_ref, &h_ref, &f_ref.thin);

            macro_rules! verify_arm {
                ($name:expr, $c:expr, $s:expr, $call:expr) => {{
                    let (f_c, ns_c, h_c) = $call;
                    assert_eq!(f_c, f_ref, "{} First mismatch blk {}", $name, blk.index);
                    assert_eq!(ns_c, ns_ref, "{} ns mismatch blk {}", $name, blk.index);
                    assert_eq!(
                        &edges_c[..3 * ns_c],
                        &edges_ref[..3 * ns_ref],
                        "{} edges mismatch blk {}",
                        $name,
                        blk.index
                    );
                    assert_eq!(
                        spill_c.cuts, spill_ref.cuts,
                        "{} cuts mismatch blk {}",
                        $name, blk.index
                    );
                    let q_c = drain_all_queues::<$c, $s>(&out_c, &spill_c, &h_c, &f_c.thin);
                    assert_eq!(q_c, q_ref, "{} queues mismatch blk {}", $name, blk.index);
                }};
            }

            verify_arm!(
                "u64_iota4",
                512,
                512,
                run_a6a::<512, 512, false, 1, true>(
                    &blk.records,
                    blk.n,
                    blk.syms.orig_ptr,
                    blk.cap,
                    &mut out_c,
                    &mut edges_c,
                    &mut spill_c
                )
            );
            verify_arm!(
                "arr_iota8",
                512,
                512,
                run_a6a::<512, 512, false, 2, true>(
                    &blk.records,
                    blk.n,
                    blk.syms.orig_ptr,
                    blk.cap,
                    &mut out_c,
                    &mut edges_c,
                    &mut spill_c
                )
            );
            verify_arm!(
                "guard_496_512",
                496,
                512,
                run_a6a::<496, 512, true, 0, true>(
                    &blk.records,
                    blk.n,
                    blk.syms.orig_ptr,
                    blk.cap,
                    &mut out_c,
                    &mut edges_c,
                    &mut spill_c
                )
            );
            verify_arm!(
                "guard_1008_1024",
                1008,
                1024,
                run_a6a::<1008, 1024, true, 0, true>(
                    &blk.records,
                    blk.n,
                    blk.syms.orig_ptr,
                    blk.cap,
                    &mut out_c,
                    &mut edges_c,
                    &mut spill_c
                )
            );
            verify_arm!(
                "pm_shipped_512",
                512,
                512,
                run_a6a_packed_max::<512, 512, false>(
                    &blk.packed_max,
                    blk.n,
                    blk.syms.orig_ptr,
                    blk.cap,
                    &mut out_c,
                    &mut edges_c,
                    &mut spill_c
                )
            );
            verify_arm!(
                "pm_guard_496_512",
                496,
                512,
                run_a6a_packed_max::<496, 512, true>(
                    &blk.packed_max,
                    blk.n,
                    blk.syms.orig_ptr,
                    blk.cap,
                    &mut out_c,
                    &mut edges_c,
                    &mut spill_c
                )
            );
            verify_arm!(
                "pm_guard_1008_1024",
                1008,
                1024,
                run_a6a_packed_max::<1008, 1024, true>(
                    &blk.packed_max,
                    blk.n,
                    blk.syms.orig_ptr,
                    blk.cap,
                    &mut out_c,
                    &mut edges_c,
                    &mut spill_c
                )
            );
        }
        println!(
            "  100% bit-for-bit A6a state-exact verified on ALL {} Open blocks across all 7 variants!",
            blocks.len()
        );
    }

    let arm_names = [
        "prod_a5_avx2",
        "shipped_512_512",
        "a6a_no_outq",
        "u64_iota4",
        "arr_iota8",
        "guard_496_512",
        "guard_1008_1024",
        "pm_shipped_512",
        "pm_guard_496_512",
        "pm_guard_1008_1024",
    ];
    let n_arms = arm_names.len();
    let mut wall_times = vec![Vec::with_capacity(repeats); n_arms];
    let mut cpu_times = vec![Vec::with_capacity(repeats); n_arms];

    let mut runs_buf = Runs::default();
    let mut out_buf = vec![0u8; 950_000];
    let mut edges_buf = Vec::with_capacity(64 * 1024);
    let mut spill = Spill::default();

    let run_arm = |idx: usize,
                   runs_buf: &mut Runs,
                   out_buf: &mut Vec<u8>,
                   edges_buf: &mut Vec<u32>,
                   spill: &mut Spill|
     -> (f64, f64) {
        let c0 = thread_cpu_ns();
        let t0 = Instant::now();
        let mut acc: u64 = 0;
        match idx {
            0 => {
                for blk in &blocks {
                    acc = acc.wrapping_add(runs::front(&blk.syms, runs_buf, k).unwrap() as u64);
                }
            }
            1 => {
                for blk in &blocks {
                    let (f, ns, _) = run_a6a::<512, 512, false, 0, true>(
                        &blk.records,
                        blk.n,
                        blk.syms.orig_ptr,
                        blk.cap,
                        out_buf,
                        edges_buf,
                        spill,
                    );
                    acc = acc.wrapping_add(f.m as u64 + ns as u64);
                }
            }
            2 => {
                for blk in &blocks {
                    let (f, ns, _) = run_a6a::<512, 512, false, 0, false>(
                        &blk.records,
                        blk.n,
                        blk.syms.orig_ptr,
                        blk.cap,
                        out_buf,
                        edges_buf,
                        spill,
                    );
                    acc = acc.wrapping_add(f.m as u64 + ns as u64);
                }
            }
            3 => {
                for blk in &blocks {
                    let (f, ns, _) = run_a6a::<512, 512, false, 1, true>(
                        &blk.records,
                        blk.n,
                        blk.syms.orig_ptr,
                        blk.cap,
                        out_buf,
                        edges_buf,
                        spill,
                    );
                    acc = acc.wrapping_add(f.m as u64 + ns as u64);
                }
            }
            4 => {
                for blk in &blocks {
                    let (f, ns, _) = run_a6a::<512, 512, false, 2, true>(
                        &blk.records,
                        blk.n,
                        blk.syms.orig_ptr,
                        blk.cap,
                        out_buf,
                        edges_buf,
                        spill,
                    );
                    acc = acc.wrapping_add(f.m as u64 + ns as u64);
                }
            }
            5 => {
                for blk in &blocks {
                    let (f, ns, _) = run_a6a::<496, 512, true, 0, true>(
                        &blk.records,
                        blk.n,
                        blk.syms.orig_ptr,
                        blk.cap,
                        out_buf,
                        edges_buf,
                        spill,
                    );
                    acc = acc.wrapping_add(f.m as u64 + ns as u64);
                }
            }
            6 => {
                for blk in &blocks {
                    let (f, ns, _) = run_a6a::<1008, 1024, true, 0, true>(
                        &blk.records,
                        blk.n,
                        blk.syms.orig_ptr,
                        blk.cap,
                        out_buf,
                        edges_buf,
                        spill,
                    );
                    acc = acc.wrapping_add(f.m as u64 + ns as u64);
                }
            }
            7 => {
                for blk in &blocks {
                    let (f, ns, _) = run_a6a_packed_max::<512, 512, false>(
                        &blk.packed_max,
                        blk.n,
                        blk.syms.orig_ptr,
                        blk.cap,
                        out_buf,
                        edges_buf,
                        spill,
                    );
                    acc = acc.wrapping_add(f.m as u64 + ns as u64);
                }
            }
            8 => {
                for blk in &blocks {
                    let (f, ns, _) = run_a6a_packed_max::<496, 512, true>(
                        &blk.packed_max,
                        blk.n,
                        blk.syms.orig_ptr,
                        blk.cap,
                        out_buf,
                        edges_buf,
                        spill,
                    );
                    acc = acc.wrapping_add(f.m as u64 + ns as u64);
                }
            }
            9 => {
                for blk in &blocks {
                    let (f, ns, _) = run_a6a_packed_max::<1008, 1024, true>(
                        &blk.packed_max,
                        blk.n,
                        blk.syms.orig_ptr,
                        blk.cap,
                        out_buf,
                        edges_buf,
                        spill,
                    );
                    acc = acc.wrapping_add(f.m as u64 + ns as u64);
                }
            }
            _ => unreachable!(),
        }
        black_box(acc);
        let wall_ms = t0.elapsed().as_secs_f64() * 1e3;
        let cpu_ms = match (c0, thread_cpu_ns()) {
            (Some(a), Some(b)) => (b.saturating_sub(a)) as f64 * 1e-6,
            _ => wall_ms,
        };
        (wall_ms, cpu_ms)
    };

    for idx in 0..n_arms {
        let _ = run_arm(idx, &mut runs_buf, &mut out_buf, &mut edges_buf, &mut spill);
    }

    for r in 0..repeats {
        let order: Vec<usize> = if r % 2 == 0 {
            (0..n_arms).collect()
        } else {
            (0..n_arms).rev().collect()
        };
        for idx in order {
            let (w, c) = run_arm(idx, &mut runs_buf, &mut out_buf, &mut edges_buf, &mut spill);
            wall_times[idx].push(w);
            cpu_times[idx].push(c);
        }
    }

    let prod_a6a_med = summarize(&cpu_times[1]).med;
    let pm_prod_med = summarize(&cpu_times[7]).med;
    println!(
        "\n  {:20} {:>10} {:>17} {:>10} {:>17} {:>9}",
        "arm", "cpu_med", "cpu [min..max]", "wall_med", "wall [min..max]", "vs_base"
    );
    for idx in 0..n_arms {
        let sc = summarize(&cpu_times[idx]);
        let sw = summarize(&wall_times[idx]);
        let denom = if idx >= 7 { pm_prod_med } else { prod_a6a_med };
        let tag = if idx >= 7 { "pm" } else { "  " };
        println!(
            "  {:20} {:8.2}ms [{:6.2}..{:6.2}] {:8.2}ms [{:6.2}..{:6.2}] {:6.3}x {}",
            arm_names[idx],
            sc.med,
            sc.min,
            sc.max,
            sw.med,
            sw.min,
            sw.max,
            sc.med / denom,
            tag
        );
    }

    #[cfg(feature = "mtf-fold")]
    {
        let ws = match worker_override {
            Some(w) => vec![w],
            None => vec![1, 2],
        };
        run_mtf_fold_e2e(&compressed, ref_out.len(), ref_crc, repeats, &ws);
    }

    #[cfg(not(feature = "mtf-fold"))]
    {
        let _ = worker_override;
        println!(
            "\n(Re-run with `--features mtf-fold,trace` to measure real mtf-fold worker-CPU trace.)"
        );
    }
}
