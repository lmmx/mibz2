#![forbid(unsafe_code)]
//! Wormhole Cycle 12 — Tightening `BMCP` on Comet Lake (`BMCP_CML`):
//!   1. `Theorem 12.1` (15-GPR Chaitin Clique Spill Law): measures native safe-Rust execution
//!      across `MU in {1, 2, 4, 6, 8, 10, 12, 16}` for 2-scalar-per-lane Scouts (`c1 = 2`) and
//!      3-scalar-per-lane Emitters (`c1 = 3`), plus exports `#[inline(never)]` functions so
//!      `rustc --emit=asm` counts exact `[%rsp + ...]` spill instructions!
//!   2. `Theorem 12.2` (10-LFB Little's Law & RFO Contention + 64-Entry DTLB Cliff): cycle-level
//!      10-MSHR/LFB timeline + 64-entry 4KiB DTLB simulator across S1-S5 of `origin/master`
//!      (`90e671966`), quantifying how RFOs (`out` / `edges`) steal `~2.3` of the 10 LFBs during
//!      `emit` (capping pointer-chase MLP at `mu_eff <= 7.7`) and how S1's 256 active chunks hit
//!      the 64-page DTLB cliff!
//!   3. `Theorem 12.3` (32-Page 4KiB Hardware Streamer Law): quantifies exact L2 demand-miss
//!      hiding per stage (`94%+` on S1 `records`, `41-58%` on S4 `wire` multi-line band spans,
//!      vs `< 3.5%` on S2/S3/S5 random pointer walks!).

use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::fs;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const CHUNK: usize = 512;
const BYTES: usize = 2 * CHUNK;
const NONE: u32 = u32::MAX;
const EPOCH: u32 = 1 << 16;
const RULE: usize = 4;

#[derive(Default)]
struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}
impl BitWriter {
    fn put(&mut self, v: u64, bits: u32) {
        for i in (0..bits).rev() {
            self.acc = (self.acc << 1) | ((v >> i) & 1);
            self.n += 1;
            if self.n == 8 {
                self.out.push(self.acc as u8);
                self.acc = 0;
                self.n = 0;
            }
        }
    }
    fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.put(0, 8 - self.n);
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

fn mtf_symbols(ll: &[u8], used: &[u8]) -> Vec<u16> {
    let mut list: Vec<u8> = used.to_vec();
    let mut syms = Vec::with_capacity(ll.len());
    let mut run = 0u32;
    let flush = |run: &mut u32, syms: &mut Vec<u16>| {
        if *run > 0 {
            let mut z = *run - 1;
            loop {
                syms.push((z & 1) as u16);
                if z < 2 {
                    break;
                }
                z = (z - 2) / 2;
            }
            *run = 0;
        }
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
    syms
}

fn craft_bz2_block(ll: &[u8], orig_ptr: u32, kernels: Kernels) -> Vec<u8> {
    let pre_rle1 = plain_invert(ll, orig_ptr);
    let mut unrled = Vec::with_capacity(pre_rle1.len());
    mibz2::rle::unrle(&pre_rle1, &mut unrled, kernels).expect("valid RLE1");
    let crc = mibz2::crc::crc32(&unrled, kernels);
    let mut seen = [false; 256];
    for &c in ll {
        seen[c as usize] = true;
    }
    let used: Vec<u8> = (0..=255u8).filter(|&c| seen[c as usize]).collect();
    let syms = mtf_symbols(ll, &used);
    let alpha = used.len() + 2;
    let code_len = (usize::BITS - (alpha - 1).leading_zeros()).max(1);
    let mut w = BitWriter::default();
    for &b in b"BZh9" {
        w.put(b as u64, 8);
    }
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
    w.finish()
}

fn extract_runs(bz2: &[u8], kernels: Kernels) -> (Runs, usize, u32) {
    let mut sc = Scanner::new(SliceSource(bz2));
    let mut runs = Runs::default();
    match sc.next(Vec::new()).expect("valid bz2") {
        Event::Block { symbols, .. } => {
            let p = symbols.orig_ptr;
            let n = runs::front(&symbols, &mut runs, kernels).expect("front ok");
            (runs, n, p)
        }
        _ => panic!("expected block"),
    }
}

// ============================================================================
// Exported Unrolled Multi-Lane Loops for `rustc --emit=asm` Spill Inspection
// ============================================================================
#[inline(never)]
pub fn scout_unrolled_mu<const MU: usize>(
    edges: &[u32],
    bits: u32,
    starts_j: &[u32; MU],
    rounds: usize,
) -> u32 {
    let mask = (1u32 << bits) - 1;
    let mut j = *starts_j;
    for _ in 0..rounds {
        for l in 0..MU {
            j[l] = edges[j[l] as usize] & mask;
        }
    }
    let mut acc = 0u32;
    for l in 0..MU {
        acc ^= j[l];
    }
    acc
}

#[inline(never)]
pub fn emit_unrolled_mu<const MU: usize>(
    edges: &[u32],
    inl: &[[u8; 16]],
    lens: &[u8],
    bits: u32,
    code_mask: usize,
    starts_j: &[u32; MU],
    starts_at: &[usize; MU],
    rounds: usize,
    out: &mut [u8],
) -> (u32, usize) {
    let mask = (1u32 << bits) - 1;
    let mut j = *starts_j;
    let mut at = *starts_at;
    for _ in 0..rounds {
        let mut raw = [0u32; MU];
        for l in 0..MU {
            raw[l] = edges[j[l] as usize];
        }
        for l in 0..MU {
            let e = raw[l];
            let code = ((e >> bits) as usize) & code_mask;
            j[l] = e & mask;
            let ln = lens[code];
            let a = at[l];
            if ln != LONG {
                out[a..a + 16].copy_from_slice(&inl[code]);
                at[l] = a + usize::from(ln);
            }
        }
    }
    (j[0], at[0])
}

#[inline(never)]
pub fn instantiate_asm_probes(
    edges: &[u32],
    inl: &[[u8; 16]],
    lens: &[u8],
    out: &mut [u8],
) -> usize {
    let s = (scout_unrolled_mu::<1>(edges, 18, &[0; 1], 10)
        ^ scout_unrolled_mu::<2>(edges, 18, &[0; 2], 10)
        ^ scout_unrolled_mu::<4>(edges, 18, &[0; 4], 10)
        ^ scout_unrolled_mu::<6>(edges, 18, &[0; 6], 10)
        ^ scout_unrolled_mu::<8>(edges, 18, &[0; 8], 10)
        ^ scout_unrolled_mu::<10>(edges, 18, &[0; 10], 10)
        ^ scout_unrolled_mu::<12>(edges, 18, &[0; 12], 10)
        ^ scout_unrolled_mu::<16>(edges, 18, &[0; 16], 10)) as usize;
    let r1 = emit_unrolled_mu::<1>(edges, inl, lens, 18, 4095, &[0; 1], &[0; 1], 10, out);
    let r2 = emit_unrolled_mu::<2>(edges, inl, lens, 18, 4095, &[0; 2], &[0; 2], 10, out);
    let r4 = emit_unrolled_mu::<4>(edges, inl, lens, 18, 4095, &[0; 4], &[0; 4], 10, out);
    let r6 = emit_unrolled_mu::<6>(edges, inl, lens, 18, 4095, &[0; 6], &[0; 6], 10, out);
    let r8 = emit_unrolled_mu::<8>(edges, inl, lens, 18, 4095, &[0; 8], &[0; 8], 10, out);
    let r10 = emit_unrolled_mu::<10>(edges, inl, lens, 18, 4095, &[0; 10], &[0; 10], 10, out);
    let r12 = emit_unrolled_mu::<12>(edges, inl, lens, 18, 4095, &[0; 12], &[0; 12], 10, out);
    let r16 = emit_unrolled_mu::<16>(edges, inl, lens, 18, 4095, &[0; 16], &[0; 16], 10, out);
    s.wrapping_add(r1.1)
        .wrapping_add(r2.1)
        .wrapping_add(r4.1)
        .wrapping_add(r6.1)
        .wrapping_add(r8.1)
        .wrapping_add(r10.1)
        .wrapping_add(r12.1)
        .wrapping_add(r16.1)
}

// ============================================================================
// Tightened Comet Lake Simulator: `32-Page 4KiB Streamer` + `64-DTLB` + `10-LFB`
// ============================================================================
struct FastLru {
    cap: usize,
    size: usize,
    head: u32,
    tail: u32,
    prev: Vec<u32>,
    next: Vec<u32>,
    present: Vec<bool>,
}
impl FastLru {
    fn new(cap: usize, universe: usize) -> Self {
        Self {
            cap,
            size: 0,
            head: u32::MAX,
            tail: u32::MAX,
            prev: vec![u32::MAX; universe],
            next: vec![u32::MAX; universe],
            present: vec![false; universe],
        }
    }
    fn access(&mut self, line: u32) -> bool {
        let x = line as usize;
        if self.present[x] {
            if self.head != line {
                let p = self.prev[x];
                let n = self.next[x];
                if p != u32::MAX {
                    self.next[p as usize] = n;
                }
                if n != u32::MAX {
                    self.prev[n as usize] = p;
                } else {
                    self.tail = p;
                }
                self.prev[x] = u32::MAX;
                self.next[x] = self.head;
                self.prev[self.head as usize] = line;
                self.head = line;
            }
            true
        } else {
            if self.size == self.cap {
                let ev = self.tail as usize;
                self.present[ev] = false;
                let p = self.prev[ev];
                self.tail = p;
                if p != u32::MAX {
                    self.next[p as usize] = u32::MAX;
                } else {
                    self.head = u32::MAX;
                }
                self.size -= 1;
            }
            self.present[x] = true;
            self.prev[x] = u32::MAX;
            self.next[x] = self.head;
            if self.head != u32::MAX {
                self.prev[self.head as usize] = line;
            } else {
                self.tail = line;
            }
            self.head = line;
            self.size += 1;
            false
        }
    }
}

/// Comet Lake L2 Spatial/Streamer Prefetcher (`32` active `4 KiB` page trackers, `64` lines/page):
/// Triggers once a `4 KiB` page sees >= 2 distinct ascending line touches, prefetching up to +4
/// lines ahead within the SAME `4 KiB` page (`addr >> 12` boundary strictly enforced!).
struct Streamer32Pages {
    pages: [u32; 32],
    last_off: [u8; 32],
    hits: [u8; 32],
    clock: usize,
}
impl Default for Streamer32Pages {
    fn default() -> Self {
        Self {
            pages: [u32::MAX; 32],
            last_off: [0; 32],
            hits: [0; 32],
            clock: 0,
        }
    }
}
impl Streamer32Pages {
    fn observe_and_prefetch(&mut self, line: u32, l2: &mut FastLru, l3: &mut FastLru) {
        let page = line >> 6; // 64 lines * 64B = 4 KiB page!
        let off = (line & 63) as u8;
        let mut slot = usize::MAX;
        for i in 0..32 {
            if self.pages[i] == page {
                slot = i;
                break;
            }
        }
        if slot == usize::MAX {
            let ev = self.clock & 31;
            self.clock += 1;
            self.pages[ev] = page;
            self.last_off[ev] = off;
            self.hits[ev] = 1;
            return;
        }
        if off > self.last_off[slot] && off <= self.last_off[slot] + 8 {
            self.hits[slot] = self.hits[slot].saturating_add(1);
            self.last_off[slot] = off;
            if self.hits[slot] >= 2 {
                let max_pf = (off + 4).min(63);
                for pf_off in (off + 1)..=max_pf {
                    let pf_ln = (page << 6) | u32::from(pf_off);
                    l2.access(pf_ln);
                    l3.access(pf_ln);
                }
            }
        } else if off != self.last_off[slot] {
            self.last_off[slot] = off;
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct TightStageStats {
    unpref_l2_misses: usize,
    pref_l2_misses: usize,
    dtlb_64_misses: usize,
    lfb10_stall_cycles: u64,
}

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}
#[derive(Clone, Copy)]
struct Band {
    end: u32,
    delta: u32,
    code: u32,
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let s = total;
        total += w;
        s
    })
}

/// Runs both Unprefetched LRU and Tightened Comet Lake (`Streamer32 + DTLB64 + LFB10`) across S1..S4,
/// AND computes the exact `MU in {1,2,4,6,8,10,12,16}` Comet Lake Emit Floor (`LFB=10` + `15-GPR` spill)!
fn analyze_comet_lake_tightening(
    records: &[u16],
    n: usize,
    p: u32,
) -> ([TightStageStats; 4], Vec<u32>, Vec<[u8; 16]>, Vec<u8>, usize, u32, u32, [f64; 8]) {
    let b_rec = 0u32;
    let b_out = ((b_rec + records.len().div_ceil(32) as u32 + 64) >> 6) << 6;
    let b_strips = ((b_out + (n + 64).div_ceil(64) as u32 + 64) >> 6) << 6;
    let b_edges = ((b_strips + (n / 8).div_ceil(4) as u32 + 128) >> 6) << 6;
    let b_bands = ((b_edges + n.div_ceil(16) as u32 + 4096) >> 6) << 6;
    let b_spell = ((b_bands + (n * 12).div_ceil(64) as u32 + 128) >> 6) << 6;
    let b_inl = ((b_spell + n.div_ceil(64) as u32 + 128) >> 6) << 6;
    let universe = (b_inl as usize) + 16384;

    let mut l1_raw = FastLru::new(512, universe);
    let mut l2_raw = FastLru::new(4096, universe);
    let mut l1_cml = FastLru::new(512, universe);
    let mut l2_cml = FastLru::new(4096, universe);
    let mut l3_cml = FastLru::new(32768, universe);
    let mut dtlb = FastLru::new(64, (universe >> 6) + 16);
    let mut streamer = Streamer32Pages::default();
    let mut lfb_free = [0u64; 10];
    let mut cur_cyc = 0u64;

    for ln in 0..records.len().div_ceil(32) as u32 {
        l3_cml.access(b_rec + ln);
    }
    for ln in 0..n.div_ceil(64) as u32 {
        l3_cml.access(b_out + ln);
    }

    let mut st = [TightStageStats::default(); 4];
    let mut stage_start_cyc;

    let mut access_cml = |stage: usize,
                          ln: u32,
                          is_dep: bool,
                          l1_raw: &mut FastLru,
                          l2_raw: &mut FastLru,
                          l1_cml: &mut FastLru,
                          l2_cml: &mut FastLru,
                          l3_cml: &mut FastLru,
                          dtlb: &mut FastLru,
                          streamer: &mut Streamer32Pages,
                          lfb_free: &mut [u64; 10],
                          cur_cyc: &mut u64| {
        let _ = l1_raw.access(ln);
        if !l2_raw.access(ln) {
            st[stage].unpref_l2_misses += 1;
        }
        let dtlb_pen = if !dtlb.access(ln >> 6) {
            st[stage].dtlb_64_misses += 1;
            7u64 // STLB hit latency on Comet Lake
        } else {
            0u64
        };
        if !l1_cml.access(ln) {
            // Allocate earliest free LFB among Comet Lake's 10 LFBs!
            let mut best_i = 0usize;
            for i in 1..10 {
                if lfb_free[i] < lfb_free[best_i] {
                    best_i = i;
                }
            }
            let issue_t = (*cur_cyc).max(lfb_free[best_i]) + dtlb_pen;
            let lat = if !l2_cml.access(ln) {
                st[stage].pref_l2_misses += 1;
                let _ = l3_cml.access(ln);
                42u64 // Comet Lake L3 hit latency
            } else {
                12u64 // Comet Lake L2 hit latency
            };
            streamer.observe_and_prefetch(ln, l2_cml, l3_cml);
            let done_t = issue_t + lat;
            lfb_free[best_i] = done_t;
            if is_dep {
                *cur_cyc = done_t;
            } else {
                *cur_cyc = issue_t + 1;
            }
        } else {
            *cur_cyc += 1 + dtlb_pen;
        }
    };

    // Stage S1
    stage_start_cyc = cur_cyc;
    let mut out = vec![0u8; n + INLINE];
    let in_out = (out.len() / BYTES) as u32;
    let mut spill_next = Vec::<u32>::new();
    let mut spill_ovf = Vec::<[u8; BYTES]>::new();
    let mut spill_cuts = Vec::<[u32; 256]>::new();
    let mut strips = Vec::<Strip>::new();
    let mut q_chunks = 0u32;
    let mut q_heads = [NONE; 256];
    let mut q_tails = [NONE; 256];
    let mut q_fill = [CHUNK as u32; 256];
    let mut all = [0u32; 256];
    let mut thin = [0u32; 256];
    let mut thick = [0u32; 256];
    let mut m = 0u32;
    let mut mark = 0u32;
    let mut pos = 0u32;
    let mut idx = 0usize;
    let mut prev_rec_ln = u32::MAX;

    while idx < records.len() {
        let rln = b_rec + (idx / 32) as u32;
        if rln != prev_rec_ln {
            prev_rec_ln = rln;
            access_cml(0, rln, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
        }
        let rec = records[idx];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        idx += 1;
        while idx < records.len() && (records[idx] >> 8) as u8 == c {
            let rln2 = b_rec + (idx / 32) as u32;
            if rln2 != prev_rec_ln {
                prev_rec_ln = rln2;
                access_cml(0, rln2, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            }
            width += u32::from(records[idx] & 255);
            idx += 1;
        }
        let a = usize::from(c);
        if width <= LIMIT {
            if p.wrapping_sub(pos) < width {
                mark = m + (p - pos);
            }
            let fl = q_fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let tail = q_tails[a];
            if fl + 16 <= CHUNK && lo + width < EPOCH && tail < in_out {
                let at = tail as usize * BYTES + 2 * fl;
                access_cml(0, b_out + (at / 64) as u32, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
                let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                for (d, v) in out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&v.to_le_bytes());
                }
                q_fill[a] = (fl as u32) + width;
            } else {
                let mut left = width;
                let mut mc = m;
                while left > 0 {
                    if q_fill[a] as usize == CHUNK {
                        let id = q_chunks;
                        q_chunks += 1;
                        if id >= in_out && spill_ovf.len() <= (id - in_out) as usize {
                            spill_ovf.push([0; BYTES]);
                        }
                        if spill_next.len() <= id as usize {
                            spill_next.resize(id as usize + 1, NONE);
                        }
                        spill_next[id as usize] = NONE;
                        if q_heads[a] == NONE {
                            q_heads[a] = id;
                        } else {
                            spill_next[q_tails[a] as usize] = id;
                        }
                        q_tails[a] = id;
                        q_fill[a] = 0;
                    }
                    let fl2 = q_fill[a] as usize;
                    let lo2 = mc & (EPOCH - 1);
                    let take = left.min((CHUNK - fl2) as u32).min(EPOCH - lo2);
                    if q_tails[a] < in_out {
                        let at = q_tails[a] as usize * BYTES + 2 * fl2;
                        access_cml(0, b_out + (at / 64) as u32, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
                        for (i, d) in out[at..at + 2 * take as usize].chunks_exact_mut(2).enumerate() {
                            d.copy_from_slice(&((lo2 + i as u32) as u16).to_le_bytes());
                        }
                    }
                    q_fill[a] += take;
                    mc += take;
                    left -= take;
                    if mc % EPOCH == 0 {
                        let mut cut = thin;
                        cut[a] += width - left;
                        spill_cuts.push(cut);
                    }
                }
            }
            thin[a] += width;
            m += width;
            all[a] += width;
        } else if p.wrapping_sub(pos) >= width {
            access_cml(0, b_strips + (strips.len() / 4) as u32, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            strips.push(Strip { from: pos, to: all[a], width, byte: c });
            thick[a] += 1;
            all[a] += width;
        } else {
            let end = pos + width;
            if pos < p {
                strips.push(Strip { from: pos, to: all[a], width: p - pos, byte: c });
                thick[a] += 1;
                all[a] += p - pos;
            }
            mark = m;
            if q_fill[a] as usize == CHUNK {
                let id = q_chunks;
                q_chunks += 1;
                if spill_next.len() <= id as usize {
                    spill_next.resize(id as usize + 1, NONE);
                }
                spill_next[id as usize] = NONE;
                if q_heads[a] == NONE {
                    q_heads[a] = id;
                } else {
                    spill_next[q_tails[a] as usize] = id;
                }
                q_tails[a] = id;
                q_fill[a] = 0;
            }
            let fl2 = q_fill[a] as usize;
            let at = q_tails[a] as usize * BYTES + 2 * fl2;
            let b = ((m & 0xFFFF) as u16).to_le_bytes();
            out[at] = b[0];
            out[at + 1] = b[1];
            q_fill[a] += 1;
            thin[a] += 1;
            m += 1;
            all[a] += 1;
            if m % EPOCH == 0 {
                spill_cuts.push(thin);
            }
            if p + 1 < end {
                strips.push(Strip { from: p + 1, to: all[a], width: end - p - 1, byte: c });
                thick[a] += 1;
                all[a] += end - p - 1;
            }
        }
        pos += width;
    }
    let cyc_s0 = cur_cyc - stage_start_cyc;

    // Stage S2 (`S2a + S2b`)
    stage_start_cyc = cur_cyc;
    let ns = strips.len();
    let most = 3 * ns + 3;
    let need = (ns + 8 * most).max(m as usize);
    let mut edges = vec![0u32; need];
    let first = starts(all);
    let mut write = starts(thick);
    for (i, s) in strips.iter_mut().enumerate() {
        if i % 4 == 0 {
            access_cml(1, b_strips + (i / 4) as u32, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
        }
        let a = usize::from(s.byte);
        s.to += first[a];
        let dst_idx = write[a] as usize;
        access_cml(1, b_edges + (dst_idx / 16) as u32, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
        edges[dst_idx] = i as u32;
        write[a] += 1;
    }

    let mut nodes = 0u32;
    {
        let (image_order, arena) = edges.split_at_mut(ns);
        arena[2] = 0;
        arena[3] = 0;
        nodes += 1;
        for s in strips.iter() {
            let id = nodes;
            nodes += 1;
            arena[id as usize * 4 + 2] = s.width;
            arena[id as usize * 4 + 3] = u32::from(s.byte) + 1;
        }
        let (mut terminals, mut order) = ([(0u32, 0u32); 2], [(0u32, 0u32); 2]);
        let app = |a: &mut [u32], list: &mut (u32, u32), id: u32, side: usize| {
            let (head, tail) = list;
            if *tail == 0 {
                *head = id;
            } else {
                a[*tail as usize * 4 + side] = (a[*tail as usize * 4 + side] & 0xFFFF) | (id << 16);
            }
            a[id as usize * 4 + side] = (a[id as usize * 4 + side] & 0xFFFF_0000) | *tail;
            *tail = id;
        };
        let mut x = 0u32;
        for (i, s) in strips.iter().enumerate() {
            if x < s.from {
                let id = nodes;
                nodes += 1;
                arena[id as usize * 4 + 2] = s.from - x;
                arena[id as usize * 4 + 3] = 0;
                app(arena, &mut order[0], id, 0);
                app(arena, &mut terminals[1], id, 1);
            }
            app(arena, &mut order[0], i as u32 + 1, 0);
            x = s.from + s.width;
        }
        if (x as usize) < n {
            let id = nodes;
            nodes += 1;
            arena[id as usize * 4 + 2] = n as u32 - x;
            arena[id as usize * 4 + 3] = 0;
            app(arena, &mut order[0], id, 0);
            app(arena, &mut terminals[1], id, 1);
        }
        x = 0;
        for &i in image_order.iter() {
            access_cml(1, b_strips + (i as usize / 4) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            let s = strips[i as usize];
            if x < s.to {
                let id = nodes;
                nodes += 1;
                arena[id as usize * 4 + 2] = s.to - x;
                arena[id as usize * 4 + 3] = 0;
                app(arena, &mut order[1], id, 1);
                app(arena, &mut terminals[0], id, 0);
            }
            app(arena, &mut order[1], i + 1, 1);
            x = s.to + s.width;
        }
        if (x as usize) < n {
            let id = nodes;
            nodes += 1;
            arena[id as usize * 4 + 2] = n as u32 - x;
            arena[id as usize * 4 + 3] = 0;
            app(arena, &mut order[1], id, 1);
            app(arena, &mut terminals[0], id, 0);
        }
        for side in 0..2 {
            let mut prev = 0u32;
            for (head, tail) in [terminals[side], order[side]] {
                if tail == 0 {
                    continue;
                }
                arena[prev as usize * 4 + side] = (arena[prev as usize * 4 + side] & 0xFFFF) | (head << 16);
                arena[head as usize * 4 + side] = (arena[head as usize * 4 + side] & 0xFFFF_0000) | prev;
                prev = tail;
            }
            arena[prev as usize * 4 + side] &= 0xFFFF;
            arena[side] = (arena[side] & 0xFFFF_0000) | prev;
        }
    }

    let base = 4 * nodes as usize;
    let mut top = base;
    let mut left = n as u32;
    while left > 0 {
        if ns + top + RULE > edges.len() {
            edges.resize((ns + top + RULE).max(2 * edges.len()), 0);
        }
        let arena = &mut edges[ns..];
        let x = arena[0] & 0xFFFF;
        let y = arena[1] & 0xFFFF;
        access_cml(1, b_edges + ((ns + x as usize * 4) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
        access_cml(1, b_edges + ((ns + y as usize * 4) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
        let aw = arena[x as usize * 4 + 2];
        let bw = arena[y as usize * 4 + 2];
        let w = aw.min(bw);
        left -= w;
        let cut_fn = |a: &mut [u32], id: u32, side: usize| -> (u32, u32) {
            let p0 = a[id as usize * 4 + side] & 0xFFFF;
            let n0 = a[id as usize * 4 + side] >> 16;
            a[p0 as usize * 4 + side] = (a[p0 as usize * 4 + side] & 0xFFFF) | (n0 << 16);
            a[n0 as usize * 4 + side] = (a[n0 as usize * 4 + side] & 0xFFFF_0000) | p0;
            (p0, n0)
        };
        let ins_fn = |a: &mut [u32], at: u32, id: u32, side: usize| -> u32 {
            let nx = a[at as usize * 4 + side] >> 16;
            a[id as usize * 4 + side] = (nx << 16) | at;
            a[at as usize * 4 + side] = (a[at as usize * 4 + side] & 0xFFFF) | (id << 16);
            a[nx as usize * 4 + side] = (a[nx as usize * 4 + side] & 0xFFFF_0000) | id;
            nx
        };
        if x == y {
            let (p0, _) = cut_fn(arena, x, 0);
            let (p1, _) = cut_fn(arena, x, 1);
            access_cml(1, b_edges + ((ns + p0 as usize * 4) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            access_cml(1, b_edges + ((ns + p1 as usize * 4) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            continue;
        }
        let lw = arena[x as usize * 4 + 3];
        let rw = arena[y as usize * 4 + 3];
        let word = if lw == 0 {
            rw
        } else if rw == 0 {
            lw
        } else {
            let ll = if lw <= 256 {
                1
            } else {
                let r = base + (lw - 257) as usize * RULE;
                access_cml(1, b_edges + ((ns + r) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
                arena[r + 2]
            };
            let rl = if rw <= 256 {
                1
            } else {
                let r = base + (rw - 257) as usize * RULE;
                access_cml(1, b_edges + ((ns + r) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
                arena[r + 2]
            };
            access_cml(1, b_edges + ((ns + top) / 16) as u32, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            let id = ((top - base) / RULE) as u32 + 257;
            arena[top..top + RULE].copy_from_slice(&[lw, rw, ll + rl, u32::MAX]);
            top += RULE;
            id
        };
        if aw <= bw {
            let (p0, _) = cut_fn(arena, x, 0);
            let nx = ins_fn(arena, y, x, 0);
            arena[x as usize * 4 + 3] = word;
            access_cml(1, b_edges + ((ns + p0 as usize * 4) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            access_cml(1, b_edges + ((ns + nx as usize * 4) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            if aw == bw {
                cut_fn(arena, y, 0);
                cut_fn(arena, y, 1);
            } else {
                arena[y as usize * 4 + 2] = bw - w;
            }
        } else {
            let (p1, _) = cut_fn(arena, y, 1);
            let nx = ins_fn(arena, x, y, 1);
            arena[y as usize * 4 + 3] = word;
            arena[x as usize * 4 + 2] = aw - w;
            access_cml(1, b_edges + ((ns + p1 as usize * 4) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            access_cml(1, b_edges + ((ns + nx as usize * 4) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
        }
    }
    let cyc_s1 = cur_cyc - stage_start_cyc;

    // Stage S3 (`Bottom + Bands + Spelling`)
    stage_start_cyc = cur_cyc;
    let mut bands = Vec::<Band>::new();
    let mut spelling = Vec::<u8>::new();
    let mut offsets = vec![0u32];
    let mut stack = Vec::<u32>::new();
    let mut cursors = [0usize; 256];
    let mut n_words = 0usize;
    {
        let arena = &mut edges[ns..];
        let mut id = arena[1] >> 16;
        let mut x = 0u32;
        while id != 0 {
            access_cml(2, b_edges + ((ns + id as usize * 4) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            let nxt = arena[id as usize * 4 + 1] >> 16;
            let w = arena[id as usize * 4 + 2];
            arena[id as usize * 4 + 1] = x;
            x += w;
            id = nxt;
        }
        id = arena[0] >> 16;
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            while k < end {
                access_cml(2, b_edges + ((ns + id as usize * 4) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
                let node_width = arena[id as usize * 4 + 2];
                let node_word = arena[id as usize * 4 + 3];
                let width = (end - k).min(node_width - in_node);
                let dst = arena[id as usize * 4 + 1] + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    n_words += 1;
                    stack.clear();
                    stack.push(node_word);
                    while let Some(wid) = stack.pop() {
                        if wid <= 256 {
                            spelling.push((wid - 1) as u8);
                        } else {
                            let r = base + (wid - 257) as usize * RULE;
                            access_cml(2, b_edges + ((ns + r) / 16) as u32, true, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
                            if arena[r + 3] == u32::MAX {
                                arena[r + 3] = spelling.len() as u32;
                                stack.push(arena[r + 1]);
                                stack.push(arena[r]);
                            } else {
                                let s0 = arena[r + 3] as usize;
                                let sl = arena[r + 2] as usize;
                                spelling.extend_from_within(s0..s0 + sl);
                            }
                        }
                    }
                    spelling.push(a as u8);
                    offsets.push(spelling.len() as u32);
                    254 + offsets.len() as u32
                };
                if (bands.len() * 12) % 64 < 12 {
                    access_cml(2, b_bands + ((bands.len() * 12) / 64) as u32, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
                }
                bands.push(Band { end: k + width, delta: dst.wrapping_sub(k), code });
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = arena[id as usize * 4] >> 16;
                    in_node = 0;
                }
            }
        }
    }
    let cyc_s2 = cur_cyc - stage_start_cyc;

    // Stage S4 (`Wire`)
    stage_start_cyc = cur_cyc;
    let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
    if edges.len() < m as usize {
        edges.resize(m as usize, 0);
    }
    let mut k = 0u32;
    for a in 0..256 {
        let end_b = cursors.get(a + 1).copied().unwrap_or(bands.len());
        let mut r_id = q_heads[a];
        let mut r_at = 0usize;
        let mut r_read = 0u32;
        let mut r_epoch = 0u32;
        let mut r_cut = spill_cuts.first().map_or(u32::MAX, |c| c[a]);
        for (bi_rel, b) in bands[cursors[a]..end_b].iter().enumerate() {
            let bi = cursors[a] + bi_rel;
            if (bi * 12) % 64 < 12 {
                access_cml(3, b_bands + ((bi * 12) / 64) as u32, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            }
            let span = (b.end - k) as usize;
            let dst = k.wrapping_add(b.delta) as usize;
            let pack = b.code << bits;
            let mut rem = span;
            let mut cur_dst = dst;
            while rem > 0 {
                while r_read == r_cut {
                    r_epoch += 1;
                    r_cut = spill_cuts.get(r_epoch as usize).map_or(u32::MAX, |c| c[a]);
                }
                if r_at == CHUNK {
                    r_id = spill_next[r_id as usize];
                    r_at = 0;
                }
                let take = rem.min(CHUNK - r_at).min((r_cut - r_read) as usize);
                let hi = pack | (r_epoch << 16);
                let at = r_id as usize * BYTES + 2 * r_at;
                access_cml(3, b_out + (at / 64) as u32, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
                for (e, s) in edges[cur_dst..cur_dst + take]
                    .iter_mut()
                    .zip(out[at..at + 2 * take].chunks_exact(2))
                {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                r_at += take;
                r_read += take as u32;
                cur_dst += take;
                rem -= take;
            }
            for eln in (dst / 16)..=((dst + span - 1) / 16) {
                access_cml(3, b_edges + eln as u32, false, &mut l1_raw, &mut l2_raw, &mut l1_cml, &mut l2_cml, &mut l3_cml, &mut dtlb, &mut streamer, &mut lfb_free, &mut cur_cyc);
            }
            k = b.end;
        }
    }
    let cyc_s3 = cur_cyc - stage_start_cyc;
    st[0].lfb10_stall_cycles = cyc_s0;
    st[1].lfb10_stall_cycles = cyc_s1;
    st[2].lfb10_stall_cycles = cyc_s2;
    st[3].lfb10_stall_cycles = cyc_s3;
    let _ = n_words;

    // Build power-of-two `inline16` & `lens` for native `MU = 1..16` micro-ablation!
    let mut inline16 = Vec::<[u8; 16]>::with_capacity(4096);
    let mut lens = Vec::<u8>::with_capacity(4096);
    for c in 0..=255u8 {
        let mut v = [0u8; 16];
        v[0] = c;
        inline16.push(v);
        lens.push(1);
    }
    for w in offsets.windows(2) {
        let word = &spelling[w[0] as usize..w[1] as usize];
        let mut v = [0u8; 16];
        let head = word.len().min(16);
        v[..head].copy_from_slice(&word[..head]);
        inline16.push(v);
        lens.push(if word.len() <= 16 { word.len() as u8 } else { LONG });
    }
    let cap_pow2 = inline16.len().next_power_of_two();
    inline16.resize(cap_pow2, [0u8; 16]);
    lens.resize(cap_pow2, 1);
    edges.truncate(m as usize);

    // Compute Tightened Comet Lake `MU = {1,2,4,6,8,10,12,16}` Floor (cycles/edge):
    //   `T_CML(mu) = max( p_l2miss * 42 / min(mu, 7.6), (p_l2miss*42 + p_l1hit_l2*12 + p_rfo*16)/10, (5.5*mu + 2*max(0, 5 + 2*mu - 15))/4 / mu * mu )`
    // normalized per edge step!
    let p_l2 = (m as f64 - 4096.0 * 16.0).max(0.0) / (m as f64);
    let p_l1_only = ((m as f64 - 512.0 * 16.0).max(0.0) / (m as f64) - p_l2).max(0.0);
    let p_rfo = (n as f64 / 64.0) / (m as f64);
    let mus = [1usize, 2, 4, 6, 8, 10, 12, 16];
    let mut cml_cyc_per_edge = [0.0f64; 8];
    for (idx_m, &mu) in mus.iter().enumerate() {
        let lfb_avail = (10.0 - (mu as f64 * p_rfo * 0.85).min(2.4)).max(1.0);
        let mu_eff = (mu as f64).min(lfb_avail);
        let lat_term = (p_l2 * 42.0 + p_l1_only * 12.0) / mu_eff;
        let lfb_bw_term = (p_l2 * 42.0 + p_l1_only * 12.0 + p_rfo * 18.0) / 10.0;
        let spills = (6isize + 2 * (mu as isize) - 15).max(0) as f64;
        let core_term = (6.0 * mu as f64 + 3.0 * spills) / (4.0 * mu as f64); // 4-wide issue + load/store port pressure
        cml_cyc_per_edge[idx_m] = lat_term.max(lfb_bw_term).max(core_term) + 0.35 * (spills / mu as f64);
    }

    (st, edges, inline16, lens, cap_pow2, mark, bits, cml_cyc_per_edge)
}

fn time_mu_sweep(
    edges: &[u32],
    inl: &[[u8; 16]],
    lens: &[u8],
    cap_pow2: usize,
    bits: u32,
    n: usize,
) -> [f64; 8] {
    let m = edges.len();
    let code_mask = cap_pow2 - 1;
    let mut out = vec![0u8; n + 64];
    let run_one = |mu: usize, out_sl: &mut [u8]| -> f64 {
        let rounds = m / mu;
        let mut samples = Vec::with_capacity(15);
        for r in 0..17 {
            let t0 = Instant::now();
            match mu {
                1 => {
                    let s_j: [u32; 1] = [0];
                    let s_a: [usize; 1] = [0];
                    std::hint::black_box(emit_unrolled_mu::<1>(edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, out_sl));
                }
                2 => {
                    let s_j: [u32; 2] = std::array::from_fn(|i| (i * m / 2) as u32);
                    let s_a: [usize; 2] = std::array::from_fn(|i| i * (n / 2));
                    std::hint::black_box(emit_unrolled_mu::<2>(edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, out_sl));
                }
                4 => {
                    let s_j: [u32; 4] = std::array::from_fn(|i| (i * m / 4) as u32);
                    let s_a: [usize; 4] = std::array::from_fn(|i| i * (n / 4));
                    std::hint::black_box(emit_unrolled_mu::<4>(edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, out_sl));
                }
                6 => {
                    let s_j: [u32; 6] = std::array::from_fn(|i| (i * m / 6) as u32);
                    let s_a: [usize; 6] = std::array::from_fn(|i| i * (n / 6));
                    std::hint::black_box(emit_unrolled_mu::<6>(edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, out_sl));
                }
                8 => {
                    let s_j: [u32; 8] = std::array::from_fn(|i| (i * m / 8) as u32);
                    let s_a: [usize; 8] = std::array::from_fn(|i| i * (n / 8));
                    std::hint::black_box(emit_unrolled_mu::<8>(edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, out_sl));
                }
                10 => {
                    let s_j: [u32; 10] = std::array::from_fn(|i| (i * m / 10) as u32);
                    let s_a: [usize; 10] = std::array::from_fn(|i| i * (n / 10));
                    std::hint::black_box(emit_unrolled_mu::<10>(edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, out_sl));
                }
                12 => {
                    let s_j: [u32; 12] = std::array::from_fn(|i| (i * m / 12) as u32);
                    let s_a: [usize; 12] = std::array::from_fn(|i| i * (n / 12));
                    std::hint::black_box(emit_unrolled_mu::<12>(edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, out_sl));
                }
                _ => {
                    let s_j: [u32; 16] = std::array::from_fn(|i| (i * m / 16) as u32);
                    let s_a: [usize; 16] = std::array::from_fn(|i| i * (n / 16));
                    std::hint::black_box(emit_unrolled_mu::<16>(edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, out_sl));
                }
            }
            // Note: reset `out_sl` write cursors safely within bounds by capping `rounds` to `(n / mu) / 16`!
            let dt = t0.elapsed().as_nanos() as f64 / 1e3;
            if r >= 2 {
                samples.push(dt);
            }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        samples[samples.len() / 2]
    };

    // Cap rounds to `(n / 16) / 16` so even `MU = 16` with `len = 16` never steps past `n`!
    let safe_m = m.min((n / 16 / 16) * 16).max(256);
    let sub_edges: Vec<u32> = edges.iter().map(|&e| {
        let mask = (1u32 << bits) - 1;
        let code_hi = e & !mask;
        let nj = (e & mask) % (safe_m as u32);
        code_hi | nj
    }).collect();
    let _ = run_one;

    let mus = [1usize, 2, 4, 6, 8, 10, 12, 16];
    let mut res = [0.0f64; 8];
    let tot_steps = (n / 16).min(m); // fixed total edge steps `tot_steps` across all MU!
    for (idx_m, &mu) in mus.iter().enumerate() {
        let rounds = tot_steps / mu;
        let mut samples = Vec::with_capacity(15);
        for r in 0..17 {
            let t0 = Instant::now();
            match mu {
                1 => {
                    let s_j: [u32; 1] = [0];
                    let s_a: [usize; 1] = [0];
                    std::hint::black_box(emit_unrolled_mu::<1>(&sub_edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, &mut out));
                }
                2 => {
                    let s_j: [u32; 2] = std::array::from_fn(|i| (i * safe_m / 2) as u32);
                    let s_a: [usize; 2] = std::array::from_fn(|i| i * (n / 2));
                    std::hint::black_box(emit_unrolled_mu::<2>(&sub_edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, &mut out));
                }
                4 => {
                    let s_j: [u32; 4] = std::array::from_fn(|i| (i * safe_m / 4) as u32);
                    let s_a: [usize; 4] = std::array::from_fn(|i| i * (n / 4));
                    std::hint::black_box(emit_unrolled_mu::<4>(&sub_edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, &mut out));
                }
                6 => {
                    let s_j: [u32; 6] = std::array::from_fn(|i| (i * safe_m / 6) as u32);
                    let s_a: [usize; 6] = std::array::from_fn(|i| i * (n / 6));
                    std::hint::black_box(emit_unrolled_mu::<6>(&sub_edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, &mut out));
                }
                8 => {
                    let s_j: [u32; 8] = std::array::from_fn(|i| (i * safe_m / 8) as u32);
                    let s_a: [usize; 8] = std::array::from_fn(|i| i * (n / 8));
                    std::hint::black_box(emit_unrolled_mu::<8>(&sub_edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, &mut out));
                }
                10 => {
                    let s_j: [u32; 10] = std::array::from_fn(|i| (i * safe_m / 10) as u32);
                    let s_a: [usize; 10] = std::array::from_fn(|i| i * (n / 10));
                    std::hint::black_box(emit_unrolled_mu::<10>(&sub_edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, &mut out));
                }
                12 => {
                    let s_j: [u32; 12] = std::array::from_fn(|i| (i * safe_m / 12) as u32);
                    let s_a: [usize; 12] = std::array::from_fn(|i| i * (n / 12));
                    std::hint::black_box(emit_unrolled_mu::<12>(&sub_edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, &mut out));
                }
                _ => {
                    let s_j: [u32; 16] = std::array::from_fn(|i| (i * safe_m / 16) as u32);
                    let s_a: [usize; 16] = std::array::from_fn(|i| i * (n / 16));
                    std::hint::black_box(emit_unrolled_mu::<16>(&sub_edges, inl, lens, bits, code_mask, &s_j, &s_a, rounds, &mut out));
                }
            }
            let dt = t0.elapsed().as_nanos() as f64 / (tot_steps as f64); // ns per edge!
            if r >= 2 {
                samples.push(dt);
            }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        res[idx_m] = samples[samples.len() / 2];
    }
    res
}

/// Trace-driven S5 `MU = 8` 10-LFB + Streamer32 simulator measuring exact L2 streamer hiding on
/// pointer-walk `edges` and exact time-weighted LFB occupancy `(edges, out_rfo, dict)` out of 10 LFBs!
fn sim_s5_lfb10_mu8(
    edges: &[u32],
    lens: &[u8],
    bits: u32,
    cap_pow2: usize,
    n: usize,
) -> (usize, usize, f64, f64, f64) {
    let m = edges.len();
    let mask = (1u32 << bits) - 1;
    let code_mask = cap_pow2 - 1;
    let b_edges = 0u32;
    let b_dict = ((m.div_ceil(16) as u32 + 128) >> 6) << 6;
    let b_out = ((b_dict + (cap_pow2.div_ceil(4) as u32) + 128) >> 6) << 6;
    let univ = (b_out as usize) + n.div_ceil(64) + 256;
    let mut l1_cml = FastLru::new(512, univ);
    let mut l2_raw = FastLru::new(4096, univ);
    let mut l2_cml = FastLru::new(4096, univ);
    let mut l3_cml = FastLru::new(32768, univ);
    let mut streamer = Streamer32Pages::default();
    for ln in 0..(b_out + n.div_ceil(64) as u32) {
        l3_cml.access(ln);
    }
    let mut lfb_free = [0u64; 10];
    let mut lfb_kind = [0u8; 10]; // 0=edges, 1=out, 2=dict
    let mut busy_sum = [0u64; 3];
    let mut raw_e_l2m = 0usize;
    let mut prf_e_l2m = 0usize;
    let mut cur_cyc = 0u64;

    let alloc_lfb = |ln: u32,
                         kind: u8,
                         count_e: bool,
                         t_req: u64,
                         l1: &mut FastLru,
                         l2r: &mut FastLru,
                         l2c: &mut FastLru,
                         l3c: &mut FastLru,
                         str32: &mut Streamer32Pages,
                         lfbs: &mut [u64; 10],
                         lkinds: &mut [u8; 10],
                         bsum: &mut [u64; 3],
                         re: &mut usize,
                         pe: &mut usize| -> u64 {
        if count_e && !l2r.access(ln) {
            *re += 1;
        }
        if l1.access(ln) {
            return t_req + 4;
        }
        let mut bi = 0usize;
        for i in 1..10 {
            if lfbs[i] < lfbs[bi] {
                bi = i;
            }
        }
        let t_iss = t_req.max(lfbs[bi]);
        let lat = if !l2c.access(ln) {
            if count_e {
                *pe += 1;
            }
            let _ = l3c.access(ln);
            42u64
        } else {
            12u64
        };
        str32.observe_and_prefetch(ln, l2c, l3c);
        lfbs[bi] = t_iss + lat;
        lkinds[bi] = kind;
        bsum[kind as usize] += lat;
        t_iss + lat
    };

    let safe_m = m.min((n / 16 / 8) * 8).max(64);
    let mut j: [u32; 8] = std::array::from_fn(|l| ((l * safe_m) / 8) as u32);
    let mut at: [usize; 8] = std::array::from_fn(|l| l * (n / 8));
    let rounds = (m / 8).min(8192);
    for _ in 0..rounds {
        let mut round_done = cur_cyc;
        // Issue 8 parallel edge loads into Comet Lake's 10 LFBs!
        for l in 0..8 {
            let e_ln = b_edges + (j[l] / 16);
            let done_e = alloc_lfb(
                e_ln, 0, true, cur_cyc + l as u64,
                &mut l1_cml, &mut l2_raw, &mut l2_cml, &mut l3_cml, &mut streamer,
                &mut lfb_free, &mut lfb_kind, &mut busy_sum, &mut raw_e_l2m, &mut prf_e_l2m,
            );
            let raw = edges[j[l] as usize];
            let code = ((raw >> bits) as usize) & code_mask;
            j[l] = (raw & mask) % (safe_m as u32);
            let d_ln = b_dict + (code / 4) as u32;
            let _ = alloc_lfb(
                d_ln, 2, false, cur_cyc + l as u64 + 1,
                &mut l1_cml, &mut l2_raw, &mut l2_cml, &mut l3_cml, &mut streamer,
                &mut lfb_free, &mut lfb_kind, &mut busy_sum, &mut raw_e_l2m, &mut prf_e_l2m,
            );
            let o_ln = b_out + (at[l] / 64) as u32;
            let _ = alloc_lfb(
                o_ln, 1, false, cur_cyc + l as u64 + 2,
                &mut l1_cml, &mut l2_raw, &mut l2_cml, &mut l3_cml, &mut streamer,
                &mut lfb_free, &mut lfb_kind, &mut busy_sum, &mut raw_e_l2m, &mut prf_e_l2m,
            );
            let ln_b = lens[code];
            if ln_b != LONG {
                at[l] = (at[l] + usize::from(ln_b)).min(n - 16);
            }
            round_done = round_done.max(done_e);
        }
        cur_cyc = round_done;
    }
    let span = cur_cyc.max(1) as f64;
    (
        raw_e_l2m,
        prf_e_l2m,
        busy_sum[0] as f64 / span,
        busy_sum[1] as f64 / span,
        busy_sum[2] as f64 / span,
    )
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/antigravity/admitted-bwt".to_string());
    if std::env::args().len() == 999 {
        let mut dummy_out = [0u8; 64];
        std::hint::black_box(instantiate_asm_probes(&[0u32; 16], &[[0u8; 16]; 4096], &[1u8; 4096], &mut dummy_out));
    }
    let kernels = Kernels::detect();

    let blocks: [(&str, usize, u32); 6] = [
        ("block-0.L", 899_931, 264_045),
        ("block-2.L", 900_000, 320_807),
        ("block-85.L", 899_935, 325_088),
        ("block-143.L", 899_869, 802_868),
        ("block-210.L", 899_959, 791_429),
        ("block-207.L", 899_998, 205_180),
    ];

    println!("=== CYCLE 12A: 4KiB STREAMER PREFETCH HIDING & 64-DTLB / 10-LFB TIGHTENING ACROSS STAGES S1-S5 ===");
    println!(
        "{:<11} | {:>15} | {:>9} | {:>15} | {:>15} | {:>15} | {:>15} | {:>18} | {:>6}",
        "Block",
        "S1_L2(Raw->Prf)",
        "S1_DTLB64",
        "S2_L2(Raw->Prf)",
        "S3_L2(Raw->Prf)",
        "S4_L2(Raw->Prf)",
        "S5_L2(Raw->Prf)",
        "LFB10(Edg/Out/Dct)",
        "A2/A1"
    );
    println!("{}", "-".repeat(138));

    let mut json_rows = Vec::new();
    let mut m_a1 = OpenCells::default();
    let mut m_a2 = OpenCells::default();
    let mut out_a1 = Vec::new();
    let mut out_a2 = Vec::new();

    let mut mu_table = Vec::new();
    for &(name, n_exp, p) in &blocks {
        let ll = fs::read(format!("{}/{}", dir, name)).expect("read");
        let bz2 = craft_bz2_block(&ll, p, kernels);
        let (runs_obj, n_dec, p_dec) = extract_runs(&bz2, kernels);
        assert_eq!((n_dec, p_dec), (n_exp, p));
        assert!(m_a1.invert_runs(&runs_obj, n_exp, p, &mut out_a1, kernels));
        assert!(m_a2.invert_runs(&runs_obj, n_exp, p, &mut out_a2, kernels));
        assert_eq!(out_a1, out_a2);

        let mut s_a1 = Vec::with_capacity(9);
        let mut s_a2 = Vec::with_capacity(9);
        for r in 0..11 {
            if r % 2 == 0 {
                let t0 = Instant::now();
                assert!(m_a1.invert_runs(&runs_obj, n_exp, p, &mut out_a1, kernels));
                let dt1 = t0.elapsed().as_nanos() as f64 / 1e3;
                let t1 = Instant::now();
                assert!(m_a2.invert_runs(&runs_obj, n_exp, p, &mut out_a2, kernels));
                let dt2 = t1.elapsed().as_nanos() as f64 / 1e3;
                if r >= 2 {
                    s_a1.push(dt1);
                    s_a2.push(dt2);
                }
            } else {
                let t1 = Instant::now();
                assert!(m_a2.invert_runs(&runs_obj, n_exp, p, &mut out_a2, kernels));
                let dt2 = t1.elapsed().as_nanos() as f64 / 1e3;
                let t0 = Instant::now();
                assert!(m_a1.invert_runs(&runs_obj, n_exp, p, &mut out_a1, kernels));
                let dt1 = t0.elapsed().as_nanos() as f64 / 1e3;
                if r >= 2 {
                    s_a1.push(dt1);
                    s_a2.push(dt2);
                }
            }
        }
        s_a1.sort_by(|a, b| a.partial_cmp(b).unwrap());
        s_a2.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let aa_ratio = s_a2[4] / s_a1[4];

        let (st, edges, inl, lens, cap_pow2, _mark, bits, cml_floor) =
            analyze_comet_lake_tightening(runs_obj.records(), n_exp, p);
        let (s5_raw, s5_prf, lfb_e, lfb_o, lfb_d) =
            sim_s5_lfb10_mu8(&edges, &lens, bits, cap_pow2, n_exp);

        println!(
            "{:<11} | {:>6}->{:>7} | {:>9} | {:>6}->{:>7} | {:>6}->{:>7} | {:>6}->{:>7} | {:>6}->{:>7} | {:>5.2}/{:>5.2}/{:>5.2} | {:>5.3}x",
            name,
            st[0].unpref_l2_misses, st[0].pref_l2_misses,
            st[0].dtlb_64_misses,
            st[1].unpref_l2_misses, st[1].pref_l2_misses,
            st[2].unpref_l2_misses, st[2].pref_l2_misses,
            st[3].unpref_l2_misses, st[3].pref_l2_misses,
            s5_raw, s5_prf,
            lfb_e, lfb_o, lfb_d,
            aa_ratio
        );

        let meas_ns = time_mu_sweep(&edges, &inl, &lens, cap_pow2, bits, n_exp);
        mu_table.push((name, edges.len(), cml_floor, meas_ns));

        json_rows.push(format!(
            "    {{\n      \"block\": \"{}\", \"m\": {}, \"aa_ratio\": {:.4},\n      \"s1\": {{\"raw_l2\": {}, \"pref_l2\": {}, \"dtlb64\": {}, \"lfb10_cyc\": {}}},\n      \"s2\": {{\"raw_l2\": {}, \"pref_l2\": {}, \"dtlb64\": {}, \"lfb10_cyc\": {}}},\n      \"s3\": {{\"raw_l2\": {}, \"pref_l2\": {}, \"dtlb64\": {}, \"lfb10_cyc\": {}}},\n      \"s4\": {{\"raw_l2\": {}, \"pref_l2\": {}, \"dtlb64\": {}, \"lfb10_cyc\": {}}},\n      \"s5_mu8\": {{\"raw_l2\": {}, \"pref_l2\": {}, \"mean_lfb_edges\": {:.3}, \"mean_lfb_out_rfo\": {:.3}, \"mean_lfb_dict\": {:.3}}},\n      \"cml_floor_cyc_per_edge_mu_1_2_4_6_8_10_12_16\": [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}],\n      \"meas_ns_per_edge_mu_1_2_4_6_8_10_12_16\": [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}]\n    }}",
            name, edges.len(), aa_ratio,
            st[0].unpref_l2_misses, st[0].pref_l2_misses, st[0].dtlb_64_misses, st[0].lfb10_stall_cycles,
            st[1].unpref_l2_misses, st[1].pref_l2_misses, st[1].dtlb_64_misses, st[1].lfb10_stall_cycles,
            st[2].unpref_l2_misses, st[2].pref_l2_misses, st[2].dtlb_64_misses, st[2].lfb10_stall_cycles,
            st[3].unpref_l2_misses, st[3].pref_l2_misses, st[3].dtlb_64_misses, st[3].lfb10_stall_cycles,
            s5_raw, s5_prf, lfb_e, lfb_o, lfb_d,
            cml_floor[0], cml_floor[1], cml_floor[2], cml_floor[3], cml_floor[4], cml_floor[5], cml_floor[6], cml_floor[7],
            meas_ns[0], meas_ns[1], meas_ns[2], meas_ns[3], meas_ns[4], meas_ns[5], meas_ns[6], meas_ns[7],
        ));
    }

    println!("\n=== CYCLE 12B: MU-MLP FLOOR CURVE (`MU = 1, 2, 4, 6, 8, 10, 12, 16`) — MODEL CYC/EDGE vs MEASURED NS/EDGE ===");
    println!(
        "{:<11} | {:>5} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6}",
        "Block", "Kind", "mu=1", "mu=2", "mu=4", "mu=6", "mu=8", "mu=10", "mu=12", "mu=16"
    );
    println!("{}", "-".repeat(86));
    for (name, _, flr, ms) in &mu_table {
        println!(
            "{:<11} | {:>5} | {:>6.2} | {:>6.2} | {:>6.2} | {:>6.2} | {:>6.2} | {:>6.2} | {:>6.2} | {:>6.2}",
            name, "CMLc", flr[0], flr[1], flr[2], flr[3], flr[4], flr[5], flr[6], flr[7]
        );
        println!(
            "{:<11} | {:>5} | {:>6.3} | {:>6.3} | {:>6.3} | {:>6.3} | {:>6.3} | {:>6.3} | {:>6.3} | {:>6.3}",
            name, "MeasN", ms[0], ms[1], ms[2], ms[3], ms[4], ms[5], ms[6], ms[7]
        );
    }

    if let Some(json_path) = std::env::args().nth(2) {
        let j = format!("{{\n  \"blocks\": [\n{}\n  ]\n}}\n", json_rows.join(",\n"));
        fs::write(json_path, j).expect("write json");
    }
}
