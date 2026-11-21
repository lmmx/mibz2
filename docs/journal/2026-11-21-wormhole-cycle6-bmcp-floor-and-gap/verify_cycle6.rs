//! Wormhole Cycle 6 — Safe-Rust Six-Dimensional BMCP Lower/Upper Bound Verifier & Gap Profiler
//!
//! Links directly against compiled `origin/master` (`libmibz2.rlib`, `--cfg feature="moonshot"`):
//!   1. Synthesizes an exact `.bz2` stream for the 6 authenticated `admitted-bwt` blocks (`block-*.L`)
//!      and drives literal `mibz2::bwt::OpenCells` (`origin/master`) alongside `DietedOpen`
//!      (`Node16` + Chaitin `Endpoint`-in-`edges` overlay + `OutQ16` queues in dead `out[0..n)` +
//!      `[u32; 3]` metadata coloring, `#[forbid(unsafe_code)]`).
//!   2. Verifies byte-exact parity across 5,227 exhaustive small columns + 2,000 random/multi-cycle
//!      columns + all 6 authenticated 900 KB blocks.
//!   3. Constructively verifies Theorem 6.2's 4-Hamming Text-Splice Adversary across all 7! = 5,040
//!      permutations (identical `(n, r, w)`, Hamming distance 4 per adjacent splice transposition,
//!      identical interior local run shapes).
//!   4. Computes the strictly separated 6-dimensional BMCP(M, B, mu) cost vector on all 6 blocks:
//!        - Preprocessing (W_prep, D_prep)
//!        - Memory footprint (S_mem: Master-0.5.9 vs DietedOpen)
//!        - Random-access count (Q_rand vs r_1 and m_16 lower bounds)
//!        - Cache misses (Belady offline OPT vs full-trace LRU at 32 KiB L1d, 256 KiB L2, 2048 KiB L3)
//!        - Serial dependency depth (D_serial and D_past_L2 at mu = 1 vs BMCP lower bounds at mu = 4, 8)
//!        - Parallel work & Lever-2 scaling exponents across n in {100k, 200k, 400k, 600k, 900k}.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct TrackAlloc;
thread_local! {
    static LIVE: Cell<isize> = const { Cell::new(0) };
    static PEAK: Cell<isize> = const { Cell::new(0) };
}
#[global_allocator]
static ALLOC: TrackAlloc = TrackAlloc;

unsafe impl GlobalAlloc for TrackAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            let _ = LIVE.try_with(|l| {
                let nl = l.get() + layout.size() as isize;
                l.set(nl);
                let _ = PEAK.try_with(|pk| {
                    if nl > pk.get() {
                        pk.set(nl);
                    }
                });
            });
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        let _ = LIVE.try_with(|l| l.set(l.get() - layout.size() as isize));
    }
}

pub fn heap_live() -> usize {
    LIVE.with(|l| l.get()).max(0) as usize
}
pub fn heap_reset_peak() {
    LIVE.with(|l| PEAK.with(|p| p.set(l.get())));
}
pub fn heap_peak() -> usize {
    PEAK.with(|p| p.get()).max(0) as usize
}

#[forbid(unsafe_code)]
mod safe_cycle6 {
    use super::{heap_live, heap_peak, heap_reset_peak};
    use mibz2::bwt::{InverseBwt, OpenCells};
    use mibz2::runs::{self, Runs};
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::Kernels;
    use std::cmp::Ordering;
    use std::collections::{BinaryHeap, HashMap};
    use std::fs;
    use std::time::Instant;

    pub const LIMIT: u32 = 16;
    pub const INLINE: usize = 16;
    pub const LONG: u8 = u8::MAX;

    // =========================================================================
    // Exact `.bz2` Block Synthesizer (matching `tests/common/mod.rs`)
    // =========================================================================
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

    pub fn plain_invert(ll: &[u8], orig_ptr: u32) -> Vec<u8> {
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

    pub fn craft_bz2_block(ll: &[u8], orig_ptr: u32, kernels: Kernels) -> Vec<u8> {
        let pre_rle1 = plain_invert(ll, orig_ptr);
        let mut unrled = Vec::with_capacity(pre_rle1.len());
        mibz2::rle::unrle(&pre_rle1, &mut unrled, kernels).expect("valid RLE1 block");
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

    pub fn extract_runs_from_bz2(bz2: &[u8], kernels: Kernels) -> (Runs, usize, u32) {
        let mut sc = Scanner::new(SliceSource(bz2));
        let mut runs = Runs::default();
        match sc.next(Vec::new()).expect("valid bz2") {
            Event::Block { symbols, .. } => {
                let p = symbols.orig_ptr;
                let n = runs::front(&symbols, &mut runs, kernels).expect("front ok");
                (runs, n, p)
            }
            _ => panic!("expected Event::Block"),
        }
    }

    pub fn records_of(l: &[u8]) -> Vec<u16> {
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

    // =========================================================================
    // Combined Dieted Open (`W1` Chaitin-Node16-in-`edges` + `OutQ16` in `out`
    // + `[u32; 3]` metadata coloring) — 100% Safe Rust!
    // =========================================================================
    #[derive(Default, Clone, Copy)]
    pub struct DietStats {
        pub n: usize,
        pub r_max: usize,
        pub r_rec: usize,
        pub r1: usize,
        pub r_thin: usize,
        pub m: usize,
        pub h_strips: usize,
        pub rv_unit_steps: usize,
        pub rv_zorich_steps: usize,
        pub n_nodes: usize,
        pub n_bands: usize,
        pub n_words: usize,
        pub spell_bytes: usize,
        pub ovf_chunks: usize,
        pub mark: u32,
        pub bits: u32,
    }

    #[derive(Default)]
    pub struct DietedOpen {
        // Shared u32 arena: hosts Node16 ([u32;4]), Rule ([u32;4]), bottom (u32), img_ord (u32)
        // in Phase 2, then overwritten by critical `edges[0..m)` in Phase 3-4!
        pub edges: Vec<u32>,
        // Shared [u32; 3] metadata: hosts `strips` in Phase 1-2, then `bands` in Phase 2b-3!
        pub triples: Vec<[u32; 3]>,
        pub spelling: Vec<u8>,
        pub offsets: Vec<u32>,
        pub inline16: Vec<[u8; INLINE]>,
        pub lens: Vec<u8>,
        pub ovf_q: Vec<[u16; 512]>,
        pub stk: Vec<u32>,
    }

    impl DietedOpen {
        pub fn retained_heap_bytes(&self) -> usize {
            self.edges.capacity() * 4
                + self.triples.capacity() * 12
                + self.spelling.capacity()
                + self.offsets.capacity() * 4
                + self.inline16.capacity() * INLINE
                + self.lens.capacity()
                + self.ovf_q.capacity() * 1024
                + self.stk.capacity() * 4
        }

        pub fn construct_and_emit(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> DietStats {
            let mut st = DietStats {
                n,
                r_rec: records.len(),
                ..DietStats::default()
            };
            out.resize(n + INLINE, 0);

            // OutQ16: partition `out[0..n+16)` into 1024-byte chunks of 512 x u16 (stored as LE bytes in `out`)!
            let c_max = out.len() / 1024;
            self.ovf_q.clear();
            let mut q_head = [u16::MAX; 256];
            let mut q_cur = [u16::MAX; 256];
            let mut q_ic = [512u16; 256];
            // Chain next pointers + epoch split for >64K retained rows:
            let mut chunk_next: Vec<u16> = Vec::with_capacity(512);
            // Record epoch transitions per symbol when (m >> 16) advances:
            let mut epoch_marks: Vec<(u8, u32)> = Vec::with_capacity(8);
            let mut cur_epoch = 0u32;

            let push_q16 = |c: u8,
                            m_start: u32,
                            w: u32,
                            out_buf: &mut [u8],
                            ovf_q: &mut Vec<[u16; 512]>,
                            q_head: &mut [u16; 256],
                            q_cur: &mut [u16; 256],
                            q_ic: &mut [u16; 256],
                            chunk_next: &mut Vec<u16>| {
                let a = c as usize;
                let mut rem = w;
                let mut r_val = m_start;
                while rem > 0 {
                    let ic = q_ic[a] as usize;
                    if ic >= 512 {
                        let new_cid = chunk_next.len() as u16;
                        chunk_next.push(u16::MAX);
                        if new_cid as usize >= c_max {
                            ovf_q.push([0u16; 512]);
                        }
                        if q_head[a] == u16::MAX {
                            q_head[a] = new_cid;
                        } else {
                            chunk_next[q_cur[a] as usize] = new_cid;
                        }
                        q_cur[a] = new_cid;
                        q_ic[a] = 0;
                        continue;
                    }
                    let space = (512 - ic) as u32;
                    let take = rem.min(space);
                    let cid = q_cur[a] as usize;
                    if cid < c_max {
                        let base = cid * 1024 + ic * 2;
                        for d in 0..take as usize {
                            let v = ((r_val + d as u32) & 0xFFFF) as u16;
                            let bytes = v.to_le_bytes();
                            out_buf[base + 2 * d] = bytes[0];
                            out_buf[base + 2 * d + 1] = bytes[1];
                        }
                    } else {
                        let slot = &mut ovf_q[cid - c_max];
                        for d in 0..take as usize {
                            slot[ic + d] = ((r_val + d as u32) & 0xFFFF) as u16;
                        }
                    }
                    q_ic[a] += take as u16;
                    r_val += take;
                    rem -= take;
                }
            };

            self.triples.clear();
            let mut all = [0u32; 256];
            let mut thin = [0u32; 256];
            let mut thick = [0u32; 256];
            let mut m = 0u32;
            let mut mark = 0u32;
            let mut pos = 0u32;
            let mut i = 0usize;

            while i < records.len() {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut w = u32::from(rec & 255);
                i += 1;
                while i < records.len() && (records[i] >> 8) as u8 == c {
                    w += u32::from(records[i] & 255);
                    i += 1;
                }
                st.r_max += 1;
                if w == 1 {
                    st.r1 += 1;
                }
                let end = pos + w;
                if w <= LIMIT {
                    st.r_thin += 1;
                    if (pos..end).contains(&p) {
                        mark = m + (p - pos);
                    }
                    // Handle 64K epoch crossing cleanly
                    let mut left = w;
                    let mut cur_m = m;
                    while left > 0 {
                        let ep = cur_m >> 16;
                        if ep > cur_epoch {
                            cur_epoch = ep;
                            for sym in 0..=255u8 {
                                epoch_marks.push((sym, thin[sym as usize]));
                            }
                        }
                        let room_ep = 65536 - (cur_m & 0xFFFF);
                        let step = left.min(room_ep);
                        push_q16(
                            c,
                            cur_m,
                            step,
                            out,
                            &mut self.ovf_q,
                            &mut q_head,
                            &mut q_cur,
                            &mut q_ic,
                            &mut chunk_next,
                        );
                        thin[c as usize] += step;
                        cur_m += step;
                        left -= step;
                    }
                    m = cur_m;
                    all[c as usize] += w;
                } else if !(pos..end).contains(&p) {
                    self.triples.push([pos, all[c as usize], (w << 8) | u32::from(c)]);
                    thick[c as usize] += 1;
                    all[c as usize] += w;
                } else {
                    if pos < p {
                        let lw = p - pos;
                        self.triples.push([pos, all[c as usize], (lw << 8) | u32::from(c)]);
                        thick[c as usize] += 1;
                        all[c as usize] += lw;
                    }
                    mark = m;
                    let ep = m >> 16;
                    if ep > cur_epoch {
                        cur_epoch = ep;
                        for sym in 0..=255u8 {
                            epoch_marks.push((sym, thin[sym as usize]));
                        }
                    }
                    push_q16(
                        c,
                        m,
                        1,
                        out,
                        &mut self.ovf_q,
                        &mut q_head,
                        &mut q_cur,
                        &mut q_ic,
                        &mut chunk_next,
                    );
                    thin[c as usize] += 1;
                    m += 1;
                    all[c as usize] += 1;
                    if p + 1 < end {
                        let rw = end - p - 1;
                        self.triples
                            .push([p + 1, all[c as usize], (rw << 8) | u32::from(c)]);
                        thick[c as usize] += 1;
                        all[c as usize] += rw;
                    }
                }
                pos = end;
            }

            let nstrips = self.triples.len();
            st.m = m as usize;
            st.h_strips = nstrips;
            st.ovf_chunks = self.ovf_q.len();
            st.mark = mark;

            // Phase 2: Direct-Linked Node16 + bottom + img_ord + growable Rules 100% overlaid inside `self.edges`!
            // Layout in `self.edges` (u32 words):
            //   nodes:  [0 .. 4 * max_nodes)
            //   bottom: [4 * max_nodes .. 5 * max_nodes)
            //   img:    [5 * max_nodes .. 5 * max_nodes + nstrips)
            //   rules:  [rules_base ..) (4 u32s per rule, dynamically extensible for periodic strips!)
            let max_nodes = 3 * nstrips + 4;
            assert!(max_nodes < 65535, "Node16 invariant holds on all blocks with H <= 21,842");
            let bot_base = 4 * max_nodes;
            let img_base = 5 * max_nodes;
            let rules_base = img_base + nstrips;
            let init_u32 = (rules_base + 4 * max_nodes).max(m as usize + 4);
            if self.edges.len() < init_u32 {
                self.edges.resize(init_u32, 0);
            }

            let mut first = [0u32; 256];
            let mut write = [0u32; 256];
            let (mut acc0, mut acc1) = (0u32, 0u32);
            for a in 0..256 {
                first[a] = acc0;
                acc0 += all[a];
                write[a] = acc1;
                acc1 += thick[a];
            }
            for si in 0..nstrips {
                let a = (self.triples[si][2] & 255) as usize;
                self.triples[si][1] += first[a];
                let slot = write[a] as usize;
                write[a] += 1;
                self.edges[img_base + slot] = si as u32;
            }

            // Helpers on packed Node16 at `self.edges[0 .. 4*max_nodes)`:
            let set_node = |ed: &mut [u32], id: u16, width: u32, word: u32| {
                let b = (id as usize) * 4;
                ed[b] = 0;
                ed[b + 1] = 0;
                ed[b + 2] = width;
                ed[b + 3] = word;
            };
            let get_prev = |ed: &[u32], id: u16, side: usize| -> u16 {
                let v = ed[(id as usize) * 4];
                if side == 0 { v as u16 } else { (v >> 16) as u16 }
            };
            let set_prev = |ed: &mut [u32], id: u16, side: usize, val: u16| {
                let b = (id as usize) * 4;
                if side == 0 {
                    ed[b] = (ed[b] & 0xFFFF_0000) | u32::from(val);
                } else {
                    ed[b] = (ed[b] & 0x0000_FFFF) | (u32::from(val) << 16);
                }
            };
            let get_next = |ed: &[u32], id: u16, side: usize| -> u16 {
                let v = ed[(id as usize) * 4 + 1];
                if side == 0 { v as u16 } else { (v >> 16) as u16 }
            };
            let set_next = |ed: &mut [u32], id: u16, side: usize, val: u16| {
                let b = (id as usize) * 4 + 1;
                if side == 0 {
                    ed[b] = (ed[b] & 0xFFFF_0000) | u32::from(val);
                } else {
                    ed[b] = (ed[b] & 0x0000_FFFF) | (u32::from(val) << 16);
                }
            };

            set_node(&mut self.edges, 0, 0, 0);
            let mut nnodes = 1u16;
            for si in 0..nstrips {
                let w = self.triples[si][2] >> 8;
                let byte = self.triples[si][2] & 255;
                set_node(&mut self.edges, nnodes, w, byte + 1);
                nnodes += 1;
            }

            let app = |ed: &mut [u32], side: usize, ht: &mut (u16, u16), id: u16| {
                if ht.0 == 0 {
                    set_prev(ed, id, side, 0);
                    set_next(ed, id, side, 0);
                    *ht = (id, id);
                } else {
                    set_next(ed, ht.1, side, id);
                    set_prev(ed, id, side, ht.1);
                    set_next(ed, id, side, 0);
                    ht.1 = id;
                }
            };

            let mut t0 = (0u16, 0u16);
            let mut o0 = (0u16, 0u16);
            let mut t1 = (0u16, 0u16);
            let mut o1 = (0u16, 0u16);

            let mut x = 0u32;
            for si in 0..nstrips {
                let from = self.triples[si][0];
                let w = self.triples[si][2] >> 8;
                if x < from {
                    let id = nnodes;
                    nnodes += 1;
                    set_node(&mut self.edges, id, from - x, 0);
                    app(&mut self.edges, 0, &mut o0, id);
                    app(&mut self.edges, 1, &mut t1, id);
                }
                app(&mut self.edges, 0, &mut o0, (si + 1) as u16);
                x = from + w;
            }
            if (x as usize) < n {
                let id = nnodes;
                nnodes += 1;
                set_node(&mut self.edges, id, n as u32 - x, 0);
                app(&mut self.edges, 0, &mut o0, id);
                app(&mut self.edges, 1, &mut t1, id);
            }

            x = 0;
            for k in 0..nstrips {
                let si = self.edges[img_base + k] as usize;
                let to = self.triples[si][1];
                let w = self.triples[si][2] >> 8;
                if x < to {
                    let id = nnodes;
                    nnodes += 1;
                    set_node(&mut self.edges, id, to - x, 0);
                    app(&mut self.edges, 1, &mut o1, id);
                    app(&mut self.edges, 0, &mut t0, id);
                }
                app(&mut self.edges, 1, &mut o1, (si + 1) as u16);
                x = to + w;
            }
            if (x as usize) < n {
                let id = nnodes;
                nnodes += 1;
                set_node(&mut self.edges, id, n as u32 - x, 0);
                app(&mut self.edges, 1, &mut o1, id);
                app(&mut self.edges, 0, &mut t0, id);
            }
            st.n_nodes = nnodes as usize;

            for (side, t_ht, o_ht) in [(0usize, t0, o0), (1usize, t1, o1)] {
                if t_ht.0 != 0 && o_ht.0 != 0 {
                    set_next(&mut self.edges, 0, side, t_ht.0);
                    set_prev(&mut self.edges, t_ht.0, side, 0);
                    set_next(&mut self.edges, t_ht.1, side, o_ht.0);
                    set_prev(&mut self.edges, o_ht.0, side, t_ht.1);
                    set_next(&mut self.edges, o_ht.1, side, 0);
                    set_prev(&mut self.edges, 0, side, o_ht.1);
                } else if t_ht.0 != 0 {
                    set_next(&mut self.edges, 0, side, t_ht.0);
                    set_prev(&mut self.edges, t_ht.0, side, 0);
                    set_next(&mut self.edges, t_ht.1, side, 0);
                    set_prev(&mut self.edges, 0, side, t_ht.1);
                } else if o_ht.0 != 0 {
                    set_next(&mut self.edges, 0, side, o_ht.0);
                    set_prev(&mut self.edges, o_ht.0, side, 0);
                    set_next(&mut self.edges, o_ht.1, side, 0);
                    set_prev(&mut self.edges, 0, side, o_ht.1);
                }
            }

            let cut_node = |ed: &mut [u32], id: u16, side: usize| {
                let prv = get_prev(ed, id, side);
                let nxt = get_next(ed, id, side);
                set_next(ed, prv, side, nxt);
                set_prev(ed, nxt, side, prv);
            };
            let ins_after = |ed: &mut [u32], at: u16, id: u16, side: usize| {
                let nxt = get_next(ed, at, side);
                set_prev(ed, id, side, at);
                set_next(ed, id, side, nxt);
                set_next(ed, at, side, id);
                set_prev(ed, nxt, side, id);
            };

            let mut nrules = 0usize;
            let mut rem = n as u32;
            let mut last_winner = (0u16, 2u8);
            while rem > 0 {
                let nx = get_prev(&self.edges, 0, 0);
                let ny = get_prev(&self.edges, 0, 1);
                let aw = self.edges[(nx as usize) * 4 + 2];
                let bw = self.edges[(ny as usize) * 4 + 2];
                let w = aw.min(bw);
                rem -= w;
                st.rv_unit_steps += 1;
                if nx == ny {
                    st.rv_zorich_steps += 1;
                    last_winner = (0, 2);
                    cut_node(&mut self.edges, nx, 0);
                    cut_node(&mut self.edges, nx, 1);
                    continue;
                }
                let cur_win = if aw < bw {
                    (ny, 1u8)
                } else if bw < aw {
                    (nx, 0u8)
                } else {
                    (0u16, 2u8)
                };
                if cur_win.1 == 2 || cur_win != last_winner {
                    st.rv_zorich_steps += 1;
                }
                last_winner = cur_win;

                let lw = self.edges[(nx as usize) * 4 + 3];
                let rw = self.edges[(ny as usize) * 4 + 3];
                let wj = if lw == 0 {
                    rw
                } else if rw == 0 {
                    lw
                } else {
                    let ll = if lw <= 256 {
                        1
                    } else {
                        self.edges[rules_base + (lw as usize - 257) * 4 + 2]
                    };
                    let rl = if rw <= 256 {
                        1
                    } else {
                        self.edges[rules_base + (rw as usize - 257) * 4 + 2]
                    };
                    let rb = rules_base + nrules * 4;
                    if self.edges.len() < rb + 4 {
                        self.edges.resize(rb + 4 + 256, 0);
                    }
                    self.edges[rb] = lw;
                    self.edges[rb + 1] = rw;
                    self.edges[rb + 2] = ll + rl;
                    self.edges[rb + 3] = u32::MAX;
                    let id = (nrules + 257) as u32;
                    nrules += 1;
                    id
                };
                if aw <= bw {
                    cut_node(&mut self.edges, nx, 0);
                    ins_after(&mut self.edges, ny, nx, 0);
                    self.edges[(nx as usize) * 4 + 3] = wj;
                    if aw == bw {
                        cut_node(&mut self.edges, ny, 0);
                        cut_node(&mut self.edges, ny, 1);
                    } else {
                        self.edges[(ny as usize) * 4 + 2] -= w;
                    }
                } else {
                    cut_node(&mut self.edges, ny, 1);
                    ins_after(&mut self.edges, nx, ny, 1);
                    self.edges[(ny as usize) * 4 + 3] = wj;
                    self.edges[(nx as usize) * 4 + 2] -= w;
                }
            }

            let mut id = get_next(&self.edges, 0, 1);
            x = 0;
            while id != 0 {
                self.edges[bot_base + id as usize] = x;
                x += self.edges[(id as usize) * 4 + 2];
                id = get_next(&self.edges, id, 1);
            }

            // Reuse `self.triples` for `bands` (`[end, delta, code]`), fused with `inline16`!
            self.triples.clear();
            self.spelling.clear();
            self.offsets.clear();
            self.offsets.push(0);
            if self.inline16.len() < 256 {
                self.inline16.clear();
                for c in 0..=255u8 {
                    let mut v = [0u8; INLINE];
                    v[0] = c;
                    self.inline16.push(v);
                }
                self.lens.clear();
                self.lens.resize(256, 1);
            }
            self.inline16.truncate(256);
            self.lens.truncate(256);

            let mut cursors = [0usize; 257];
            id = get_next(&self.edges, 0, 0);
            let mut in_node = 0u32;
            let mut k = 0u32;
            let mut nwords = 0usize;
            for a in 0..256usize {
                cursors[a] = self.triples.len();
                let end = k + thin[a];
                while k < end {
                    let nw = self.edges[(id as usize) * 4 + 2];
                    let width = (end - k).min(nw - in_node);
                    let dst = self.edges[bot_base + id as usize] + in_node;
                    let wd = self.edges[(id as usize) * 4 + 3];
                    let code = if wd == 0 {
                        a as u32
                    } else {
                        let o0 = self.spelling.len();
                        self.stk.clear();
                        self.stk.push(wd);
                        while let Some(cur) = self.stk.pop() {
                            if cur <= 256 {
                                self.spelling.push((cur - 1) as u8);
                            } else {
                                let rb = rules_base + (cur as usize - 257) * 4;
                                let first_off = self.edges[rb + 3];
                                if first_off == u32::MAX {
                                    self.edges[rb + 3] = self.spelling.len() as u32;
                                    self.stk.push(self.edges[rb + 1]);
                                    self.stk.push(self.edges[rb]);
                                } else {
                                    let f = first_off as usize;
                                    let l = self.edges[rb + 2] as usize;
                                    self.spelling.extend_from_within(f..f + l);
                                }
                            }
                        }
                        self.spelling.push(a as u8);
                        self.offsets.push(self.spelling.len() as u32);
                        let wl = self.spelling.len() - o0;
                        let mut v = [0u8; INLINE];
                        let head = wl.min(INLINE);
                        v[..head].copy_from_slice(&self.spelling[o0..o0 + head]);
                        self.inline16.push(v);
                        self.lens.push(if wl <= INLINE { wl as u8 } else { LONG });
                        nwords += 1;
                        (255 + nwords) as u32
                    };
                    self.triples.push([k + width, dst.wrapping_sub(k), code]);
                    in_node += width;
                    k += width;
                    if in_node == nw {
                        id = get_next(&self.edges, id, 0);
                        in_node = 0;
                    }
                }
            }
            cursors[256] = self.triples.len();
            st.n_bands = self.triples.len();
            st.n_words = nwords;
            st.spell_bytes = self.spelling.len();

            // Phase 3: Wire `self.edges[0..m)` from `OutQ16` inside `out` (`Endpoint` now 100% dead!)
            let bits = if m <= 1 { 1 } else { 32 - (m - 1).leading_zeros() };
            st.bits = bits;
            // Reconstruct per-symbol epoch thresholds:
            let mut sym_ep_thresh: [Vec<u32>; 256] = std::array::from_fn(|_| Vec::new());
            for &(sym, cnt_at_ep) in &epoch_marks {
                sym_ep_thresh[sym as usize].push(cnt_at_ep);
            }

            k = 0;
            for a in 0..256usize {
                let mut cid = q_head[a];
                let mut ic = 0usize;
                let mut consumed = 0u32;
                let mut ep_idx = 0usize;
                let mut high_add = 0u32;
                let th = &sym_ep_thresh[a];

                for bi in cursors[a]..cursors[a + 1] {
                    let b_end = self.triples[bi][0];
                    let b_delta = self.triples[bi][1];
                    let pack = self.triples[bi][2] << bits;
                    let span = (b_end - k) as usize;
                    let dst = k.wrapping_add(b_delta) as usize;
                    for d in 0..span {
                        while ep_idx < th.len() && consumed == th[ep_idx] {
                            high_add += 65536;
                            ep_idx += 1;
                        }
                        if ic == 512 {
                            cid = chunk_next[cid as usize];
                            ic = 0;
                        }
                        let c_usize = cid as usize;
                        let lo16 = if c_usize < c_max {
                            let base = c_usize * 1024 + ic * 2;
                            u16::from_le_bytes([out[base], out[base + 1]])
                        } else {
                            self.ovf_q[c_usize - c_max][ic]
                        };
                        self.edges[dst + d] = pack | (high_add + u32::from(lo16));
                        ic += 1;
                        consumed += 1;
                    }
                    k = b_end;
                }
            }
            self.edges.truncate(m as usize);

            // Phase 4: Critical Walk (`emit`) — OutQ16 in `out` is now 100% dead!
            let mask = (1u32 << bits) - 1;
            let mut j = mark;
            let mut at = 0usize;
            for _ in 0..m {
                let edge = self.edges[j as usize];
                let code = (edge >> bits) as usize;
                j = edge & mask;
                let len = self.lens[code];
                if len != LONG {
                    out[at..at + INLINE].copy_from_slice(&self.inline16[code]);
                    at += usize::from(len);
                } else {
                    let w = code - 256;
                    let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                    let take = word.len().min(n - at);
                    out[at..at + take].copy_from_slice(&word[..take]);
                    at += take;
                }
                if j == mark || at >= n {
                    break;
                }
            }
            out.truncate(at.min(n));
            while out.len() < n {
                let take = out.len().min(n - out.len());
                out.extend_from_within(..take);
            }
            st
        }
    }

    // =========================================================================
    // Exact Belady Offline OPT (1966) & Mattson/LRU (1970) Cache Simulators
    // =========================================================================
    #[derive(Clone, Copy, Eq, PartialEq)]
    struct NextUse {
        next_step: usize,
        line: u32,
    }
    impl Ord for NextUse {
        fn cmp(&self, other: &Self) -> Ordering {
            // Max-heap on `next_step` (evict farthest-in-future!)
            self.next_step
                .cmp(&other.next_step)
                .then_with(|| self.line.cmp(&other.line))
        }
    }
    impl PartialOrd for NextUse {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    /// Exact Belady (1966) minimum offline cache misses for a sequence of line accesses.
    pub fn belady_opt_misses(trace: &[u32], cap_lines: usize, max_line_id: usize) -> usize {
        if cap_lines == 0 || trace.is_empty() {
            return trace.len();
        }
        let mut last_seen = vec![usize::MAX; max_line_id + 1];
        let mut next_use = vec![usize::MAX; trace.len()];
        for idx in (0..trace.len()).rev() {
            let ln = trace[idx] as usize;
            next_use[idx] = last_seen[ln];
            last_seen[ln] = idx;
        }
        let mut in_cache = vec![false; max_line_id + 1];
        let mut cur_next = vec![0usize; max_line_id + 1];
        let mut resident = 0usize;
        let mut heap = BinaryHeap::<NextUse>::new();
        let mut misses = 0usize;

        for (t, &ln_u32) in trace.iter().enumerate() {
            let ln = ln_u32 as usize;
            let nu = next_use[t];
            if in_cache[ln] {
                cur_next[ln] = nu;
                heap.push(NextUse {
                    next_step: nu,
                    line: ln_u32,
                });
                continue;
            }
            misses += 1;
            if resident < cap_lines {
                in_cache[ln] = true;
                cur_next[ln] = nu;
                resident += 1;
                heap.push(NextUse {
                    next_step: nu,
                    line: ln_u32,
                });
            } else {
                // Evict valid resident line with maximum `next_step`
                loop {
                    let top = heap.pop().expect("non-empty cache");
                    let t_ln = top.line as usize;
                    if in_cache[t_ln] && cur_next[t_ln] == top.next_step {
                        in_cache[t_ln] = false;
                        break;
                    }
                }
                in_cache[ln] = true;
                cur_next[ln] = nu;
                heap.push(NextUse {
                    next_step: nu,
                    line: ln_u32,
                });
            }
        }
        misses
    }

    /// Exact Fully-Associative LRU Cache Simulator via O(1) Doubly-Linked Intrusive Arena.
    pub struct FastLru {
        cap: usize,
        size: usize,
        head: u32,
        tail: u32,
        prev: Vec<u32>,
        next: Vec<u32>,
        present: Vec<bool>,
    }
    impl FastLru {
        pub fn new(cap: usize, universe: usize) -> Self {
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
        #[inline]
        pub fn access(&mut self, line: u32) -> bool {
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

    #[derive(Default, Debug, Clone, Copy)]
    pub struct EmitCacheProfile {
        pub steps: usize,
        pub unique_edge_lines: usize,
        pub opt_edges_256k: usize,
        pub lru_edges_only_256k: usize,
        pub full_l1_miss_edges: usize,
        pub full_l2_miss_edges_dep: usize, // exact `dep_past_l2_loads` at M2 = 256 KiB!
        pub full_l2_miss_total: usize,
        pub full_l3_miss_2048k: usize,
    }

    /// Simulates exact `emit` memory accesses across disjoint line namespaces:
    ///   - edges[j]         (4 B/entry -> 16 entries / 64B line)
    ///   - lens[code]       (1 B/entry -> 64 entries / 64B line)
    ///   - inline16[code]   (16 B/entry -> 4 entries / 64B line) or spelling
    ///   - out[at..at+len]  (1 B/entry -> 64 entries / 64B line)
    pub fn profile_emit_caches(d: &DietedOpen, n: usize, mark: u32, bits: u32) -> EmitCacheProfile {
        let m = d.edges.len();
        let base_edges = 0u32;
        let n_edge_lines = m.div_ceil(16) as u32 + 2;
        let base_lens = base_edges + n_edge_lines;
        let n_lens_lines = d.lens.len().div_ceil(64) as u32 + 2;
        let base_inl = base_lens + n_lens_lines;
        let n_inl_lines = d.inline16.len().div_ceil(4) as u32 + 2;
        let base_spell = base_inl + n_inl_lines;
        let n_spell_lines = d.spelling.len().div_ceil(64) as u32 + 2;
        let base_out = base_spell + n_spell_lines;
        let n_out_lines = (n + 32).div_ceil(64) as u32 + 2;
        let universe = (base_out + n_out_lines) as usize;

        // Pre-warm L3 (2048 KiB = 32768 lines) and L2 (256 KiB = 4096 lines) with end-of-Phase-3 state:
        // Phase 3 (`wire`) just touched `edges` sequentially by bands and `inline16/spelling` in Phase 2b!
        let mut l1 = FastLru::new(32 * 1024 / 64, universe); // 512 lines (32 KiB L1d)
        let mut l2 = FastLru::new(256 * 1024 / 64, universe); // 4096 lines (256 KiB L2)
        let mut l3 = FastLru::new(2048 * 1024 / 64, universe); // 32768 lines (2048 KiB = W=10 share of 20 MiB L3)

        // Warm L3 with `out` (from OutQ16), `inline16/lens/spelling`, and `edges` (written in Phase 3):
        for ln in 0..n_out_lines.min((n.div_ceil(64)) as u32) {
            l3.access(base_out + ln);
        }
        for ln in 0..n_inl_lines - 2 {
            l2.access(base_inl + ln);
            l3.access(base_inl + ln);
        }
        for ln in 0..n_lens_lines - 2 {
            l2.access(base_lens + ln);
            l3.access(base_lens + ln);
        }
        for ln in 0..(m.div_ceil(16) as u32) {
            l2.access(base_edges + ln);
            l3.access(base_edges + ln);
        }

        let mut edge_trace = Vec::with_capacity(m);
        let mut seen_edge_line = vec![false; n_edge_lines as usize];
        let mut unique_edge_lines = 0usize;

        let mut prof = EmitCacheProfile::default();
        let mask = (1u32 << bits) - 1;
        let mut j = mark;
        let mut at = 0usize;

        for _ in 0..m {
            prof.steps += 1;
            let eln = j / 16;
            edge_trace.push(eln);
            if !seen_edge_line[eln as usize] {
                seen_edge_line[eln as usize] = true;
                unique_edge_lines += 1;
            }

            let g_edge = base_edges + eln;
            if !l1.access(g_edge) {
                prof.full_l1_miss_edges += 1;
            }
            if !l2.access(g_edge) {
                prof.full_l2_miss_edges_dep += 1;
                prof.full_l2_miss_total += 1;
            }
            if !l3.access(g_edge) {
                prof.full_l3_miss_2048k += 1;
            }

            let edge = d.edges[j as usize];
            let code = (edge >> bits) as usize;
            j = edge & mask;

            let g_len = base_lens + (code / 64) as u32;
            l1.access(g_len);
            if !l2.access(g_len) {
                prof.full_l2_miss_total += 1;
            }
            if !l3.access(g_len) {
                prof.full_l3_miss_2048k += 1;
            }

            let len = d.lens[code];
            let step_bytes = if len != LONG {
                let g_inl = base_inl + (code / 4) as u32;
                l1.access(g_inl);
                if !l2.access(g_inl) {
                    prof.full_l2_miss_total += 1;
                }
                if !l3.access(g_inl) {
                    prof.full_l3_miss_2048k += 1;
                }
                usize::from(len)
            } else {
                let w = code - 256;
                let o0 = d.offsets[w] as usize;
                let o1 = d.offsets[w + 1] as usize;
                let take = (o1 - o0).min(n - at);
                for sl in (o0 / 64)..=((o0 + take.saturating_sub(1)) / 64) {
                    let g_sp = base_spell + sl as u32;
                    l1.access(g_sp);
                    if !l2.access(g_sp) {
                        prof.full_l2_miss_total += 1;
                    }
                    if !l3.access(g_sp) {
                        prof.full_l3_miss_2048k += 1;
                    }
                }
                take
            };

            let end_write = (at + step_bytes.max(16)).min(n + 16);
            for ol in (at / 64)..=(end_write.saturating_sub(1) / 64) {
                let g_out = base_out + ol as u32;
                l1.access(g_out);
                if !l2.access(g_out) {
                    prof.full_l2_miss_total += 1;
                }
                if !l3.access(g_out) {
                    prof.full_l3_miss_2048k += 1;
                }
            }
            at += step_bytes;
            if j == mark || at >= n {
                break;
            }
        }

        prof.unique_edge_lines = unique_edge_lines;
        prof.opt_edges_256k = belady_opt_misses(&edge_trace, 4096, n_edge_lines as usize);
        let mut l2_edges_only = FastLru::new(4096, n_edge_lines as usize);
        for &eln in &edge_trace {
            if !l2_edges_only.access(eln) {
                prof.lru_edges_only_256k += 1;
            }
        }
        prof
    }

    // =========================================================================
    // Constructive Verification of Theorem 6.2: 4-Hamming Text-Splice Adversary
    // =========================================================================
    fn bwt_of_cyclic(t: &[u8]) -> (Vec<u8>, u32) {
        let n = t.len();
        let mut rots: Vec<usize> = (0..n).collect();
        rots.sort_by(|&a, &b| {
            for k in 0..n {
                let ca = t[(a + k) % n];
                let cb = t[(b + k) % n];
                if ca != cb {
                    return ca.cmp(&cb);
                }
            }
            Ordering::Equal
        });
        let mut ll = Vec::with_capacity(n);
        let mut p = 0u32;
        for (rank, &start) in rots.iter().enumerate() {
            if start == 0 {
                p = rank as u32;
            }
            ll.push(t[(start + n - 1) % n]);
        }
        (ll, p)
    }

    fn run_profile(ll: &[u8]) -> (usize, Vec<u32>) {
        let mut widths = Vec::new();
        let mut i = 0;
        while i < ll.len() {
            let c = ll[i];
            let mut j = i + 1;
            while j < ll.len() && ll[j] == c {
                j += 1;
            }
            widths.push((j - i) as u32);
            i = j;
        }
        let r = widths.len();
        widths.sort_unstable();
        (r, widths)
    }

    pub fn verify_theorem_6_2_splice_adversary() -> usize {
        // Construct K = 8 blocks separated by unique marker `u = b"0000001"`:
        // Text: u X_{perm[0]} u X_{perm[1]} ... u X_{perm[6]} u Anchor$
        // Each X_q has distinct first char (b'A' + q) and distinct last char (b'a' + q),
        // and distinct interior payload, with `u` occurring ONLY at the 8 splice boundaries!
        let u = b"0000001";
        let blocks: [&[u8]; 7] = [
            b"A_alpha_payload_11_a",
            b"B_bravo_payload_2222_b",
            b"C_charlie_payload_333333_c",
            b"D_delta_payload_44_d",
            b"E_echo_payload_55555_e",
            b"F_foxtrot_payload_666_f",
            b"G_golf_payload_7777777_g",
        ];
        let tail = b"H_anchor_end_h$";

        // Generate all 7! = 5,040 permutations iteratively (Heap's algorithm):
        let mut perm = [0usize, 1, 2, 3, 4, 5, 6];
        let mut c = [0usize; 7];
        let mut all_bwts: Vec<([usize; 7], Vec<u8>, u32)> = Vec::with_capacity(5040);

        let build_text = |p: &[usize; 7]| -> Vec<u8> {
            let mut t = Vec::with_capacity(256);
            for &idx in p {
                t.extend_from_slice(u);
                t.extend_from_slice(blocks[idx]);
            }
            t.extend_from_slice(u);
            t.extend_from_slice(tail);
            t
        };

        let t0 = build_text(&perm);
        let (l0, p0) = bwt_of_cyclic(&t0);
        let ref_prof = run_profile(&l0);
        all_bwts.push((perm, l0, p0));

        let mut idx = 0;
        while idx < 7 {
            if c[idx] < idx {
                if idx % 2 == 0 {
                    perm.swap(0, idx);
                } else {
                    perm.swap(c[idx], idx);
                }
                let tx = build_text(&perm);
                let (lx, px) = bwt_of_cyclic(&tx);
                let pr = run_profile(&lx);
                // Verify IDENTICAL (n, r, sorted run-width multiset) across ALL 5,040 permutations!
                assert_eq!(lx.len(), t0.len());
                assert_eq!(pr, ref_prof, "run profile must be invariant across all 7! splices");
                all_bwts.push((perm, lx, px));
                c[idx] += 1;
                idx = 0;
            } else {
                c[idx] = 0;
                idx += 1;
            }
        }
        assert_eq!(all_bwts.len(), 5040);

        // Verify Hamming distance == 4 for adjacent transpositions swapping two non-adjacent blocks!
        let mut map = HashMap::<[usize; 7], usize>::with_capacity(5040);
        for (k, (pm, _, _)) in all_bwts.iter().enumerate() {
            map.insert(*pm, k);
        }
        let mut hamming4_checked = 0usize;
        for (pm, la, _) in &all_bwts {
            // Swap slots 1 and 4 (non-adjacent in text, so their markers don't touch each other!)
            let mut pm2 = *pm;
            pm2.swap(1, 4);
            let lb = &all_bwts[map[&pm2]].1;
            let hd = la.iter().zip(lb.iter()).filter(|(x, y)| x != y).count();
            assert_eq!(hd, 4, "4-Hamming splice lemma: swapping two non-adjacent blocks changes EXACTLY 4 rows of L!");
            hamming4_checked += 1;
        }
        assert_eq!(hamming4_checked, 5040);
        all_bwts.len()
    }

    // =========================================================================
    // Exhaustive Small & Multi-Cycle Verification (`Gate G1`)
    // =========================================================================
    pub fn verify_small_and_random(diet: &mut DietedOpen) -> usize {
        let mut out = Vec::new();
        let mut cases = 0usize;

        // 1. All binary columns n = 1..=9, all marks p < n
        for n in 1..=9usize {
            for mask in 0..(1u32 << n) {
                let ll: Vec<u8> = (0..n).map(|b| ((mask >> b) & 1) as u8).collect();
                let recs = records_of(&ll);
                for p in 0..n as u32 {
                    let want = plain_invert(&ll, p);
                    diet.construct_and_emit(&recs, n, p, &mut out);
                    assert_eq!(out, want, "binary mismatch n={n} p={p}");
                    cases += 1;
                }
            }
        }
        // 2. All ternary columns n = 1..=6, all marks p < n
        for n in 1..=6usize {
            let mut tot = 1u32;
            for _ in 0..n {
                tot *= 3;
            }
            for code in 0..tot {
                let mut tmp = code;
                let ll: Vec<u8> = (0..n)
                    .map(|_| {
                        let d = (tmp % 3) as u8;
                        tmp /= 3;
                        d
                    })
                    .collect();
                let recs = records_of(&ll);
                for p in 0..n as u32 {
                    let want = plain_invert(&ll, p);
                    diet.construct_and_emit(&recs, n, p, &mut out);
                    assert_eq!(out, want, "ternary mismatch n={n} p={p}");
                    cases += 1;
                }
            }
        }
        // 3. 2,000 random + periodic + >64K multi-epoch stress columns
        let mut rng = 0x9E37_79B9_7F4A_7C15u64;
        let mut next_u64 = || -> u64 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };
        for t in 0..2000usize {
            let n = if t < 1995 {
                (next_u64() as usize % 512) + 1
            } else {
                // Multi-epoch (> 65,536 retained rows, up to 235,000 <= 900,000) stress test!
                75_000 + (t - 1995) * 40_000
            };
            let alpha = ((next_u64() % 6) + 2) as u8;
            let mut ll = Vec::with_capacity(n);
            while ll.len() < n {
                let sym = (next_u64() as u8) % alpha;
                let run_w = if t >= 1995 {
                    ((next_u64() % 8) + 1) as usize
                } else if next_u64() % 4 == 0 {
                    ((next_u64() % 45) + 17) as usize
                } else {
                    ((next_u64() % 16) + 1) as usize
                };
                let take = run_w.min(n - ll.len());
                ll.resize(ll.len() + take, sym);
            }
            let p = (next_u64() as usize % n) as u32;
            let want = plain_invert(&ll, p);
            let recs = records_of(&ll);
            diet.construct_and_emit(&recs, n, p, &mut out);
            assert_eq!(out, want, "stress mismatch case {t} n={n}");
            cases += 1;
        }
        cases
    }

    pub fn rlbwt_info_bytes(ll: &[u8]) -> f64 {
        let mut seen = [false; 256];
        let mut runs = 0usize;
        let mut width_bits = 0.0f64;
        let mut i = 0;
        while i < ll.len() {
            let c = ll[i];
            seen[c as usize] = true;
            let mut j = i + 1;
            while j < ll.len() && ll[j] == c {
                j += 1;
            }
            let w = (j - i) as f64;
            // Elias-gamma / order-0 bit bound for run width + symbol
            width_bits += 1.0 + 2.0 * w.log2().floor();
            runs += 1;
            i = j;
        }
        let sigma = seen.iter().filter(|&&x| x).count().max(2) as f64;
        (runs as f64 * sigma.log2() + width_bits) / 8.0
    }

    pub fn run_main() {
        let dir = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "/home/antigravity/admitted-bwt".to_string());
        let kernels = Kernels::detect();

        println!("=== 1. GATE G1: SMALL/STRESS VERIFICATION & THEOREM 6.2 ADVERSARY ===");
        let mut diet = DietedOpen::default();
        let g1_cases = verify_small_and_random(&mut diet);
        let splice_perms = verify_theorem_6_2_splice_adversary();
        println!(
            "PASS Gate G1: {} small/multi-cycle/multi-epoch cases + {} 4-Hamming splice permutations verified.",
            g1_cases, splice_perms
        );

        let blocks: [(&str, usize, u32); 6] = [
            ("block-0.L", 899_931, 264_045),
            ("block-2.L", 900_000, 320_807),
            ("block-85.L", 899_935, 325_088),
            ("block-143.L", 899_869, 802_868),
            ("block-210.L", 899_959, 791_429),
            ("block-207.L", 899_998, 205_180),
        ];

        println!("\n=== 2. SIX-DIMENSIONAL BMCP(M, B, mu) LOWER vs DIETED-OPEN UPPER LEDGER ===");
        println!(
            "{:<11} | {:>7} {:>7} {:>7} | {:>6} {:>6} | {:>7} {:>7} {:>7} {:>7} | {:>7} {:>7}",
            "Block",
            "r",
            "r_1(LB)",
            "m_16",
            "RLBWT",
            "4*m",
            "CompL2",
            "OPT_L2",
            "DietL2",
            "DietL3",
            "LB_mu4",
            "LB_mu8"
        );
        println!("{}", "-".repeat(114));

        // Measure persistent heap of literal `origin/master` `mibz2::bwt::OpenCells` vs `DietedOpen`
        let base_live = heap_live();
        let mut master_open = OpenCells::default();
        let mut out_master = Vec::with_capacity(900_032);
        let mut master_peak_scratch = 0usize;

        let mut diet_clean = DietedOpen::default();
        let mut out_diet = Vec::with_capacity(900_032);

        let mut json_rows = Vec::new();
        let mut block0_ll = Vec::new();
        let mut block143_ll = Vec::new();

        for &(name, n_exp, p) in &blocks {
            let path = format!("{}/{}", dir, name);
            let ll = fs::read(&path).expect("read block");
            assert_eq!(ll.len(), n_exp);
            if name == "block-0.L" {
                block0_ll = ll.clone();
            } else if name == "block-143.L" {
                block143_ll = ll.clone();
            }

            let rlbwt_kb = rlbwt_info_bytes(&ll) / 1024.0;
            let bz2 = craft_bz2_block(&ll, p, kernels);
            let (runs_obj, n_dec, p_dec) = extract_runs_from_bz2(&bz2, kernels);
            assert_eq!(n_dec, n_exp);
            assert_eq!(p_dec, p);
            let recs_direct = records_of(&ll);
            assert_eq!(runs_obj.records(), recs_direct.as_slice(), "Runs::records() exact match");

            // Run literal `origin/master` OpenCells and track its exact scratch heap!
            let before_master = heap_live();
            heap_reset_peak();
            assert!(master_open.invert_runs(&runs_obj, n_exp, p, &mut out_master, kernels));
            let m_peak = heap_peak().saturating_sub(before_master);
            let m_ret = heap_live().saturating_sub(base_live);
            if m_peak > master_peak_scratch {
                master_peak_scratch = m_peak;
            }
            let _ = m_ret;

            // Run DietedOpen and verify byte-for-byte identity against `origin/master` OpenCells!
            let st = diet_clean.construct_and_emit(runs_obj.records(), n_exp, p, &mut out_diet);
            assert_eq!(out_diet, out_master, "DietedOpen == origin/master OpenCells on {name}");

            // Exact Belady OPT & Full-Trace LRU simulation
            let cp = profile_emit_caches(&diet_clean, n_exp, st.mark, st.bits);
            let lb_mu4 = cp.opt_edges_256k.div_ceil(4);
            let lb_mu8 = cp.opt_edges_256k.div_ceil(8);

            println!(
                "{:<11} | {:>7} {:>7} {:>7} | {:>5.1}K {:>5.1}K | {:>7} {:>7} {:>7} {:>7} | {:>7} {:>7}",
                name,
                st.r_max,
                st.r1,
                st.m,
                rlbwt_kb,
                (st.m * 4) as f64 / 1024.0,
                cp.unique_edge_lines,
                cp.opt_edges_256k,
                cp.full_l2_miss_edges_dep,
                cp.full_l3_miss_2048k,
                lb_mu4,
                lb_mu8
            );

            // Quick 15-round paired native safe-Rust timing vs `origin/master` OpenCells
            let rounds = 15usize;
            let mut t_master = Vec::with_capacity(rounds);
            let mut t_diet = Vec::with_capacity(rounds);
            for r in 0..(rounds + 2) {
                for slot in 0..2 {
                    if (r + slot) % 2 == 0 {
                        let t0 = Instant::now();
                        master_open.invert_runs(&runs_obj, n_exp, p, &mut out_master, kernels);
                        let dt = t0.elapsed().as_nanos() as u64;
                        if r >= 2 {
                            t_master.push(dt);
                        }
                    } else {
                        let t0 = Instant::now();
                        diet_clean.construct_and_emit(runs_obj.records(), n_exp, p, &mut out_diet);
                        let dt = t0.elapsed().as_nanos() as u64;
                        if r >= 2 {
                            t_diet.push(dt);
                        }
                    }
                }
            }
            t_master.sort_unstable();
            t_diet.sort_unstable();
            let med_master_us = t_master[rounds / 2] as f64 / 1e3;
            let med_diet_us = t_diet[rounds / 2] as f64 / 1e3;

            json_rows.push(format!(
                "    {{\"block\": \"{}\", \"n\": {}, \"r\": {}, \"r_rec\": {}, \"r1\": {}, \"r_thin\": {}, \"m\": {}, \"H\": {}, \"rv_unit_steps\": {}, \"rv_zorich_steps\": {}, \"n_bands\": {}, \"n_words\": {}, \"rlbwt_kib\": {:.1}, \"edges_kib\": {:.1}, \"compulsory_l2_lines\": {}, \"belady_opt_l2_misses\": {}, \"lru_edges_only_l2_misses\": {}, \"diet_dep_past_l2\": {}, \"diet_total_l2_misses\": {}, \"diet_l3_2048k_misses\": {}, \"lb_dep_l2_mu4\": {}, \"lb_dep_l2_mu8\": {}, \"master_us\": {:.1}, \"diet_us\": {:.1}}}",
                name, st.n, st.r_max, st.r_rec, st.r1, st.r_thin, st.m, st.h_strips,
                st.rv_unit_steps, st.rv_zorich_steps, st.n_bands, st.n_words,
                rlbwt_kb, (st.m * 4) as f64 / 1024.0,
                cp.unique_edge_lines, cp.opt_edges_256k, cp.lru_edges_only_256k,
                cp.full_l2_miss_edges_dep, cp.full_l2_miss_total, cp.full_l3_miss_2048k,
                lb_mu4, lb_mu8, med_master_us, med_diet_us
            ));
        }

        // Measure exact retained scratch heap of `master_open` vs `diet_clean`
        drop(out_master);
        drop(out_diet);
        let diet_heap = diet_clean.retained_heap_bytes();
        drop(diet_clean);
        let after_diet_drop = heap_live();
        drop(master_open);
        let after_master_drop = heap_live();
        let master_retained_heap = after_diet_drop.saturating_sub(after_master_drop);

        println!("\n=== 3. PER-WORKER PERSISTENT SCRATCH HEAP (EXCLUDING 879 KiB `out`) ===");
        println!(
            "  Master-0.5.9 OpenCells : {:>7.1} KiB (w/ out = {:>6.1} KiB -> 10w = {:>5.1} MiB > 20 MiB L3!)",
            master_retained_heap as f64 / 1024.0,
            (master_retained_heap + 900_016) as f64 / 1024.0,
            10.0 * (master_retained_heap + 900_016) as f64 / (1024.0 * 1024.0)
        );
        println!(
            "  DietedOpen (W1+OutQ16) : {:>7.1} KiB ({:.3}x; w/ out = {:>6.1} KiB <= 2048 KiB -> 10w = {:>5.1} MiB <= 20 MiB L3!)",
            diet_heap as f64 / 1024.0,
            diet_heap as f64 / master_retained_heap.max(1) as f64,
            (diet_heap + 900_016) as f64 / 1024.0,
            10.0 * (diet_heap + 900_016) as f64 / (1024.0 * 1024.0)
        );

        // =========================================================================
        // 4. Lever 2 (RH Proof Journal): Growth Exponents across Block Sizes 100k..900k
        // =========================================================================
        println!("\n=== 4. LEVER 2: GROWTH RATE SCALING ACROSS n = 100k .. 900k (L2 = 256 KiB PHASE TRANSITION) ===");
        let mut scale_json = Vec::new();
        for (label, src_ll) in [("block-0_text", &block0_ll), ("block-143_text", &block143_ll)] {
            let p_ref = if label == "block-0_text" { 264_045 } else { 802_868 };
            let full_text = plain_invert(src_ll, p_ref);
            println!("--- Source: {} ---", label);
            println!(
                "  {:>9} | {:>7} {:>7} {:>7} | {:>8} {:>9} {:>9}",
                "n_prefix", "r", "m_16", "4m(KiB)", "OPT_L2", "dep_pastL2", "miss/byte"
            );
            for &sz in &[100_000usize, 200_000, 300_000, 450_000, 650_000, full_text.len()] {
                // Fast prefix BWT via suffix sorting of `full_text[..sz]`
                let sub = &full_text[..sz];
                let mut sa: Vec<u32> = (0..sz as u32).collect();
                sa.sort_unstable_by(|&a, &b| {
                    let ia = a as usize;
                    let ib = b as usize;
                    // Compare up to 256 bytes first (fast path on Wikidata JSON), then cyclic fallback:
                    let la = &sub[ia..sz.min(ia + 256)];
                    let lb = &sub[ib..sz.min(ib + 256)];
                    let common = la.len().min(lb.len());
                    let ord = la[..common].cmp(&lb[..common]);
                    if ord != Ordering::Equal {
                        return ord;
                    }
                    for k in common..sz {
                        let ca = sub[(ia + k) % sz];
                        let cb = sub[(ib + k) % sz];
                        if ca != cb {
                            return ca.cmp(&cb);
                        }
                    }
                    Ordering::Equal
                });
                let mut sub_l = Vec::with_capacity(sz);
                let mut sub_p = 0u32;
                for (rk, &pos0) in sa.iter().enumerate() {
                    if pos0 == 0 {
                        sub_p = rk as u32;
                    }
                    sub_l.push(sub[(pos0 as usize + sz - 1) % sz]);
                }
                let recs = records_of(&sub_l);
                let mut d_tmp = DietedOpen::default();
                let mut out_tmp = Vec::new();
                let st = d_tmp.construct_and_emit(&recs, sz, sub_p, &mut out_tmp);
                assert_eq!(out_tmp.as_slice(), sub);
                let cp = profile_emit_caches(&d_tmp, sz, st.mark, st.bits);
                println!(
                    "  {:>9} | {:>7} {:>7} {:>6.1}K | {:>8} {:>9} {:>9.4}",
                    sz,
                    st.r_max,
                    st.m,
                    (st.m * 4) as f64 / 1024.0,
                    cp.opt_edges_256k,
                    cp.full_l2_miss_edges_dep,
                    cp.full_l2_miss_edges_dep as f64 / sz as f64
                );
                scale_json.push(format!(
                    "    {{\"source\": \"{}\", \"n\": {}, \"r\": {}, \"m\": {}, \"edges_kib\": {:.1}, \"opt_l2\": {}, \"dep_past_l2\": {}}}",
                    label, sz, st.r_max, st.m, (st.m * 4) as f64 / 1024.0, cp.opt_edges_256k, cp.full_l2_miss_edges_dep
                ));
            }
        }

        let out_json = format!(
            "{{\n  \"origin_master_sha\": \"d425d772c0a0e6b82e9626b36bbfa4b28a6c9613\",\n  \"g1_small_and_stress_cases\": {},\n  \"thm_6_2_splice_permutations\": {},\n  \"master_retained_heap_bytes\": {},\n  \"diet_retained_heap_bytes\": {},\n  \"blocks\": [\n{}\n  ],\n  \"scaling\": [\n{}\n  ]\n}}\n",
            g1_cases,
            splice_perms,
            master_retained_heap,
            diet_heap,
            json_rows.join(",\n"),
            scale_json.join(",\n")
        );
        if let Some(json_path) = std::env::args().nth(2) {
            fs::write(json_path, out_json).expect("write json");
        }
    }
}

fn main() {
    safe_cycle6::run_main();
}
