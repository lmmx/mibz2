#![forbid(unsafe_code)]

mod bits_safe;
mod huff_monoid;
mod open_copy;

use bits_safe::Reg;
use huff_monoid::{
    read_block_huff_mtf_bl, read_block_monoid_bl, tokens_to_column, tokens_to_maxruns,
    MaxRunsBlock, MonoidItem, RunTokens,
};
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item, Symbols, BLOCK_MAGIC, END_MAGIC};
use mibz2::bwt::{InverseBwt, LazyCells, OpenCells, Plain};
use mibz2::{crc, mtf, rle, runs, Kernels};
use open_copy::{Cap, OpenKernel, OpenTimings};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---- Small-instance exact .bz2 encoder for exhaustive verification ----
struct BitWriter {
    buf: Vec<u8>,
    acc: u64,
    nbits: u32,
}
impl BitWriter {
    fn new() -> Self {
        BitWriter {
            buf: Vec::new(),
            acc: 0,
            nbits: 0,
        }
    }
    fn put(&mut self, val: u64, bits: u32) {
        self.acc = (self.acc << bits) | (val & ((1u64 << bits) - 1));
        self.nbits += bits;
        while self.nbits >= 8 {
            self.nbits -= 8;
            self.buf.push(((self.acc >> self.nbits) & 0xff) as u8);
            self.acc &= (1u64 << self.nbits) - 1;
        }
    }
    fn finish(mut self) -> Vec<u8> {
        if self.nbits > 0 {
            self.buf.push(((self.acc << (8 - self.nbits)) & 0xff) as u8);
        }
        self.buf
    }
}

fn huffman_lengths(freqs: &[u64]) -> Vec<u8> {
    let alpha = freqs.len();
    let mut w: Vec<u64> = freqs.iter().map(|&f| f.max(1)).collect();
    loop {
        let mut heap: BinaryHeap<Reverse<(u64, Vec<usize>)>> = w
            .iter()
            .enumerate()
            .map(|(i, &wt)| Reverse((wt, vec![i])))
            .collect();
        let mut lens = vec![0u8; alpha];
        while heap.len() > 1 {
            let Reverse((w1, s1)) = heap.pop().unwrap();
            let Reverse((w2, s2)) = heap.pop().unwrap();
            for &i in s1.iter().chain(s2.iter()) {
                lens[i] += 1;
            }
            let mut merged = s1;
            merged.extend(s2);
            heap.push(Reverse((w1 + w2, merged)));
        }
        if lens.iter().all(|&l| l <= 17) {
            return lens;
        }
        for x in &mut w {
            *x = 1 + (*x >> 1);
        }
    }
}

fn canonical_codes(lens: &[u8]) -> Vec<u32> {
    let alpha = lens.len();
    let min_l = *lens.iter().min().unwrap();
    let max_l = *lens.iter().max().unwrap();
    let mut codes = vec![0u32; alpha];
    let mut code = 0u32;
    for l in min_l..=max_l {
        for s in 0..alpha {
            if lens[s] == l {
                codes[s] = code;
                code += 1;
            }
        }
        code <<= 1;
    }
    codes
}

fn encode_column_as_bz2(l: &[u8], orig_ptr: u32, block_crc: u32) -> Vec<u8> {
    let mut used = [false; 256];
    for &c in l {
        used[c as usize] = true;
    }
    let mut list: Vec<u8> = (0..=255u8).filter(|&c| used[c as usize]).collect();
    let alpha = list.len() + 2;
    let eob = (alpha - 1) as u16;
    let mut syms = Vec::new();
    let mut zeros = 0u32;
    let flush = |z: &mut u32, syms: &mut Vec<u16>| {
        while *z > 0 {
            syms.push(((*z - 1) & 1) as u16);
            *z = (*z - 1) >> 1;
        }
    };
    for &c in l {
        let i = list.iter().position(|&x| x == c).unwrap();
        if i == 0 {
            zeros += 1;
        } else {
            flush(&mut zeros, &mut syms);
            syms.push((i + 1) as u16);
            list.remove(i);
            list.insert(0, c);
        }
    }
    flush(&mut zeros, &mut syms);
    syms.push(eob);

    let mut freqs = vec![0u64; alpha];
    for &s in &syms {
        freqs[s as usize] += 1;
    }
    let lens = huffman_lengths(&freqs);
    let codes = canonical_codes(&lens);
    let n_groups = 2usize;
    let n_sel = syms.len().div_ceil(50).max(1);

    let mut bw = BitWriter::new();
    for &b in b"BZh9" {
        bw.put(b as u64, 8);
    }
    bw.put(BLOCK_MAGIC >> 24, 24);
    bw.put(BLOCK_MAGIC & 0xff_ffff, 24);
    bw.put(block_crc as u64, 32);
    bw.put(0, 1);
    bw.put(orig_ptr as u64, 24);
    let mut in_use16 = 0u32;
    for i in 0..16 {
        if (0..16).any(|j| used[i * 16 + j]) {
            in_use16 |= 0x8000 >> i;
        }
    }
    bw.put(in_use16 as u64, 16);
    for i in 0..16 {
        if in_use16 & (0x8000 >> i) != 0 {
            let mut w = 0u32;
            for j in 0..16 {
                if used[i * 16 + j] {
                    w |= 0x8000 >> j;
                }
            }
            bw.put(w as u64, 16);
        }
    }
    bw.put(n_groups as u64, 3);
    bw.put(n_sel as u64, 15);
    let mut mtf_g = [0usize, 1];
    for idx in 0..n_sel {
        let g = idx & 1;
        let j = mtf_g.iter().position(|&x| x == g).unwrap();
        mtf_g.copy_within(0..j, 1);
        mtf_g[0] = g;
        for _ in 0..j {
            bw.put(1, 1);
        }
        bw.put(0, 1);
    }
    for _ in 0..n_groups {
        let mut curr = lens[0] as i32;
        bw.put(curr as u64, 5);
        for &l in &lens {
            while curr < l as i32 {
                bw.put(0b10, 2);
                curr += 1;
            }
            while curr > l as i32 {
                bw.put(0b11, 2);
                curr -= 1;
            }
            bw.put(0, 1);
        }
    }
    for &s in &syms {
        bw.put(codes[s as usize] as u64, lens[s as usize] as u32);
    }
    bw.put(END_MAGIC >> 24, 24);
    bw.put(END_MAGIC & 0xff_ffff, 24);
    bw.put(block_crc as u64, 32);
    bw.finish()
}

fn plain_inv(l: &[u8], p: u32) -> Vec<u8> {
    let mut counts = [0u32; 256];
    for &c in l {
        counts[c as usize] += 1;
    }
    let mut out = Vec::new();
    Plain::default().invert(l, &counts, p, &mut out);
    out
}

fn verify_exhaustive() {
    let mut ok = OpenKernel::default();
    let mut rt = RunTokens::default();
    let mut rt_sc = RunTokens::default();
    let mut mrb = MaxRunsBlock::default();
    let mut maxruns = Vec::new();
    let mut out = Vec::new();
    let mut cases = 0usize;

    for (base, max_n) in [(2usize, 8usize), (3, 5)] {
        for n in 1..=max_n {
            for z in 0..base.pow(n as u32) {
                let mut tmp = z;
                let l: Vec<u8> = (0..n)
                    .map(|_| {
                        let c = tmp % base;
                        tmp /= base;
                        (c * 73 + 5) as u8
                    })
                    .collect();
                let bz2 = encode_column_as_bz2(&l, 0, 0);
                let mut reg = Reg::at(&bz2, 32);
                assert!(matches!(
                    read_block_monoid_bl(&bz2, &mut reg, 9, &mut rt, false).unwrap(),
                    MonoidItem::Block
                ));
                assert_eq!(rt.n, n);
                tokens_to_maxruns(&rt, &mut maxruns);

                let mut reg2 = Reg::at(&bz2, 32);
                assert!(matches!(
                    read_block_huff_mtf_bl(&bz2, &mut reg2, 9, &mut mrb, &mut rt_sc).unwrap(),
                    MonoidItem::Block
                ));
                assert_eq!(mrb.n, n);
                assert_eq!(mrb.runs, maxruns);
                assert_eq!(reg.bit_pos(), reg2.bit_pos());

                for p in 0..n as u32 {
                    let want = plain_inv(&l, p);
                    rt.orig_ptr = p;
                    ok.run_from_tokens_direct(&rt, Cap::NONE, &mut out, None)
                        .unwrap();
                    assert_eq!(out, want, "B0 mismatch n={n} z={z} p={p}");
                    ok.run_from_tokens_striped(&rt, Cap::NONE, &mut out, None)
                        .unwrap();
                    assert_eq!(out, want, "B1 mismatch n={n} z={z} p={p}");
                    ok.run_from_maxruns(&maxruns, n, p, Cap::NONE, &mut out, None)
                        .unwrap();
                    assert_eq!(out, want, "B2/C mismatch n={n} z={z} p={p}");
                    cases += 1;
                }
            }
        }
    }

    // Synthetic edge cases: leading runs > 255, alternating runs crossing 50-sym boundary, high-entropy
    let mut edge_cols: Vec<Vec<u8>> = Vec::new();
    edge_cols.push(vec![42u8; 600]);
    let mut c2 = vec![10u8; 300];
    for i in 0..80 {
        c2.extend(std::iter::repeat((10 + (i % 20) + 1) as u8).take(1 + (i * 17) % 290));
    }
    edge_cols.push(c2);
    let mut x = 0x1234_5678_9abc_def0u64;
    let c3: Vec<u8> = (0..2000)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x as u8
        })
        .collect();
    edge_cols.push(c3);

    for (idx, l) in edge_cols.iter().enumerate() {
        let n = l.len();
        for &p in &[0u32, (n / 2) as u32, (n - 1) as u32] {
            let want = plain_inv(l, p);
            let bz2 = encode_column_as_bz2(l, p, 0);
            let mut reg = Reg::at(&bz2, 32);
            assert!(matches!(
                read_block_monoid_bl(&bz2, &mut reg, 9, &mut rt, false).unwrap(),
                MonoidItem::Block
            ));
            let mut br = Bits::at(&bz2, 32);
            let Item::Block(syms) = read_item(&mut br, 9, Vec::new()).unwrap() else {
                panic!()
            };
            assert_eq!(reg.bit_pos(), br.bit_pos(), "bit_pos mismatch edge {idx}");
            ok.run_from_tokens_direct(&rt, Cap::NONE, &mut out, None)
                .unwrap();
            assert_eq!(out, want, "edge {idx} B0");
            ok.run_from_tokens_striped(&rt, Cap::NONE, &mut out, None)
                .unwrap();
            assert_eq!(out, want, "edge {idx} B1");
            let mut ll = Vec::new();
            let mut cnt = [0u32; 256];
            tokens_to_column(&rt, &mut ll, &mut cnt);
            assert_eq!(&ll, l, "edge {idx} column");
            let mut ll2 = Vec::new();
            let mut cnt2 = [0u32; 256];
            mtf::mtf(&syms, &mut ll2, &mut cnt2, Kernels::detect()).unwrap();
            assert_eq!(ll, ll2);
            assert_eq!(cnt, cnt2);
            cases += 1;
        }
    }

    println!(
        "[VERIFIED] Exhaustive binary n<=8 (3,586) + ternary n<=5 (1,641) + 9 edge cases = {cases} cases ALL 100% EXACT!"
    );
}

#[derive(Clone, Copy, Debug)]
struct BlockLoc {
    index: usize,
    start_bit: u64,
    level: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockClass {
    OpenAdmitNone,
    OpenAdmitDense,
    RunRefusedLazy,
    ColumnRoute,
}

fn scan_all_blocks(data: &[u8]) -> Vec<BlockLoc> {
    let mut locs = Vec::new();
    for (i, sp) in mibz2::scan(data).enumerate() {
        let sp = sp.expect("valid bz2");
        locs.push(BlockLoc {
            index: i,
            start_bit: sp.start_bit,
            level: sp.level,
        });
    }
    locs
}

struct MasterWorker {
    syms: Vec<u16>,
    runs: runs::Runs,
    ll: Vec<u8>,
    counts: [u32; 256],
    open_cells: OpenCells,
    out: Vec<u8>,
    k: Kernels,
}

impl MasterWorker {
    fn new() -> Self {
        MasterWorker {
            syms: Vec::new(),
            runs: runs::Runs::default(),
            ll: Vec::new(),
            counts: [0; 256],
            open_cells: OpenCells::default(),
            out: Vec::new(),
            k: Kernels::detect(),
        }
    }

    fn decode_block_full(&mut self, data: &[u8], loc: BlockLoc) -> u32 {
        let mut br = Bits::at(data, loc.start_bit);
        let syms_buf = std::mem::take(&mut self.syms);
        let Item::Block(b) = read_item(&mut br, loc.level, syms_buf).unwrap() else {
            panic!("expected block");
        };
        let crc = self.finish_symbols(&b);
        self.syms = b.syms;
        crc
    }

    fn finish_symbols(&mut self, b: &Symbols) -> u32 {
        self.out.clear();
        if runs::takes_runs(b) {
            let n = runs::front(b, &mut self.runs, self.k).unwrap();
            assert!(self
                .open_cells
                .invert_runs(&self.runs, n, b.orig_ptr, &mut self.ll, self.k));
        } else {
            let _n = mtf::mtf(b, &mut self.ll, &mut self.counts, self.k).unwrap();
            if !self
                .open_cells
                .invert_in_place(&mut self.ll, &self.counts, b.orig_ptr)
            {
                let mut rle_buf = Vec::new();
                self.open_cells
                    .invert(&self.ll, &self.counts, b.orig_ptr, &mut rle_buf);
                std::mem::swap(&mut self.ll, &mut rle_buf);
            }
        }
        rle::unrle(&self.ll, &mut self.out, self.k).unwrap();
        let computed = crc::crc32(&self.out, self.k);
        assert_eq!(computed, b.crc);
        computed
    }
}

#[derive(Clone, Copy)]
enum MonoidMode {
    Direct,
    Striped,
    MaxRuns,
}

struct MonoidWorker {
    rt: RunTokens,
    open: OpenKernel,
    lazy: LazyCells,
    maxruns: Vec<u32>,
    ll: Vec<u8>,
    counts: [u32; 256],
    out: Vec<u8>,
    k: Kernels,
    mode: MonoidMode,
}

impl MonoidWorker {
    fn new(mode: MonoidMode) -> Self {
        MonoidWorker {
            rt: RunTokens::default(),
            open: OpenKernel::default(),
            lazy: LazyCells::default(),
            maxruns: Vec::new(),
            ll: Vec::new(),
            counts: [0; 256],
            out: Vec::new(),
            k: Kernels::detect(),
            mode,
        }
    }

    fn decode_block_full(&mut self, data: &[u8], loc: BlockLoc) -> u32 {
        let mut reg = Reg::at(data, loc.start_bit);
        assert!(matches!(
            read_block_monoid_bl(data, &mut reg, loc.level, &mut self.rt, false).unwrap(),
            MonoidItem::Block
        ));
        self.finish_tokens()
    }

    fn finish_tokens(&mut self) -> u32 {
        self.out.clear();
        let n = self.rt.n;
        let p = self.rt.orig_ptr;
        let mut done_open = false;
        if self.rt.takes_runs() {
            let cap127 = self.rt.compute_cap127();
            if let Some(cap) = Cap::for_block(cap127, n, p) {
                let res = match self.mode {
                    MonoidMode::Direct => {
                        self.open
                            .run_from_tokens_direct(&self.rt, cap, &mut self.ll, None)
                    }
                    MonoidMode::Striped => {
                        self.open
                            .run_from_tokens_striped(&self.rt, cap, &mut self.ll, None)
                    }
                    MonoidMode::MaxRuns => {
                        tokens_to_maxruns(&self.rt, &mut self.maxruns);
                        self.open
                            .run_from_maxruns(&self.maxruns, n, p, cap, &mut self.ll, None)
                    }
                };
                done_open = res.is_some();
            }
        }
        if !done_open {
            tokens_to_column(&self.rt, &mut self.ll, &mut self.counts);
            if !self.lazy.invert_in_place(&mut self.ll, &self.counts, p) {
                let mut rle_buf = Vec::new();
                self.lazy.invert(&self.ll, &self.counts, p, &mut rle_buf);
                std::mem::swap(&mut self.ll, &mut rle_buf);
            }
        }
        rle::unrle(&self.ll, &mut self.out, self.k).unwrap();
        let computed = crc::crc32(&self.out, self.k);
        assert_eq!(computed, self.rt.crc);
        computed
    }
}

fn verify_corpus(data: &[u8], locs: &[BlockLoc]) -> Vec<(BlockClass, usize, u32)> {
    let mut mw = MasterWorker::new();
    let mut b0 = MonoidWorker::new(MonoidMode::Direct);
    let mut b1 = MonoidWorker::new(MonoidMode::Striped);
    let mut b2 = MonoidWorker::new(MonoidMode::MaxRuns);
    let mut classes = Vec::with_capacity(locs.len());
    let mut total_bytes = 0usize;
    let mut total_syms = 0usize;
    let mut total_run_syms = 0usize;
    let mut total_tokens = 0usize;
    let mut total_cap127 = 0usize;
    let mut total_fast_l = 0u64;
    let mut total_slow_l = 0u64;
    let mut total_hdr_ns = 0u64;
    let mut total_lp_ns = 0u64;

    for &loc in locs {
        let c_m = mw.decode_block_full(data, loc);
        let mut reg = Reg::at(data, loc.start_bit);
        read_block_monoid_bl(data, &mut reg, loc.level, &mut b0.rt, true).unwrap();
        let c_b0 = b0.finish_tokens();
        let c_b1 = b1.decode_block_full(data, loc);
        let c_b2 = b2.decode_block_full(data, loc);
        assert_eq!(c_m, c_b0, "block {} B0 CRC", loc.index);
        assert_eq!(c_m, c_b1, "block {} B1 CRC", loc.index);
        assert_eq!(c_m, c_b2, "block {} B2 CRC", loc.index);
        assert_eq!(mw.out, b0.out, "block {} B0 bytes", loc.index);
        assert_eq!(mw.out, b1.out, "block {} B1 bytes", loc.index);
        assert_eq!(mw.out, b2.out, "block {} B2 bytes", loc.index);
        total_bytes += mw.out.len();
        total_syms += b0.rt.total_syms;
        total_run_syms += b0.rt.run_syms;
        total_tokens += b0.rt.slice().len();
        let cap127 = b0.rt.compute_cap127();
        total_cap127 += cap127;
        total_fast_l += b0.rt.fast_lookups;
        total_slow_l += b0.rt.slow_lookups;
        total_hdr_ns += b0.rt.header_ns;
        total_lp_ns += b0.rt.loop_ns;

        let cls = if !b0.rt.takes_runs() {
            BlockClass::ColumnRoute
        } else {
            match Cap::for_block(cap127, b0.rt.n, b0.rt.orig_ptr) {
                Some(Cap::NONE) => BlockClass::OpenAdmitNone,
                Some(_) => BlockClass::OpenAdmitDense,
                None => BlockClass::RunRefusedLazy,
            }
        };
        classes.push((cls, b0.rt.n, b0.rt.orig_ptr));
    }

    let tot_lookups = total_fast_l + total_slow_l;
    println!(
        "[VERIFIED] Corpus ({} blocks, {} decoded bytes): ALL ARMS 100% BYTE-IDENTICAL + CRC-MATCHED!",
        locs.len(),
        total_bytes
    );
    println!(
        "  Corpus totals: syms={} ({:.1}% RUNA/B), maximal_runs={} (vs 127-capped={}), syms/run={:.2}",
        total_syms,
        100.0 * total_run_syms as f64 / total_syms as f64,
        total_tokens,
        total_cap127,
        total_syms as f64 / total_tokens as f64
    );
    println!(
        "  HuffMonoidBL-v2 lookups: fast={} ({:.2}%), slow={}, mean syms/lookup={:.3}, hdr={:.2}ms loop={:.2}ms",
        total_fast_l,
        100.0 * total_fast_l as f64 / tot_lookups.max(1) as f64,
        total_slow_l,
        total_syms as f64 / tot_lookups.max(1) as f64,
        total_hdr_ns as f64 / 1e6,
        total_lp_ns as f64 / 1e6
    );
    classes
}

fn find_manifest_six(classes: &[(BlockClass, usize, u32)]) -> Vec<(usize, &'static str)> {
    // Exact (n, primary_index, name) from /tmp/mibz2-blocks/manifest.json
    let manifest = [
        (899931usize, 264045u32, "block-0"),
        (900000, 320807, "block-2"),
        (899935, 325088, "block-85"),
        (899869, 802868, "block-143"),
        (899998, 205180, "block-207"),
        (899959, 791429, "block-210"),
    ];
    let mut found = Vec::new();
    for &(mn, mp, name) in &manifest {
        if let Some(idx) = classes
            .iter()
            .position(|&(_, n, p)| n == mn && p == mp)
        {
            found.push((idx, name));
        }
    }
    found
}

fn run_scanner_ab(label: &str, data: &[u8], locs: &[BlockLoc], rounds: usize) {
    println!("\n=== ISOLATED SCANNER A/B (Phase 1 only, {label}, {rounds} rounds, 1T) ===");
    let mut t_m1 = Vec::new();
    let mut t_m2 = Vec::new();
    let mut t_bl = Vec::new();
    let mut syms_buf = Vec::new();
    let mut rt = RunTokens::default();
    for r in 0..rounds {
        let order = if r % 2 == 0 { [0, 1, 2] } else { [2, 1, 0] };
        for arm in order {
            let t0 = Instant::now();
            match arm {
                0 | 1 => {
                    let mut sum = 0usize;
                    for &loc in locs {
                        let mut br = Bits::at(data, loc.start_bit);
                        let sb = std::mem::take(&mut syms_buf);
                        let Item::Block(b) = read_item(&mut br, loc.level, sb).unwrap() else {
                            panic!()
                        };
                        sum += b.syms.len();
                        syms_buf = b.syms;
                    }
                    std::hint::black_box(sum);
                    let ms = t0.elapsed().as_secs_f64() * 1e3;
                    if arm == 0 {
                        t_m1.push(ms);
                    } else {
                        t_m2.push(ms);
                    }
                }
                _ => {
                    let mut sum = 0usize;
                    for &loc in locs {
                        let mut reg = Reg::at(data, loc.start_bit);
                        read_block_monoid_bl(data, &mut reg, loc.level, &mut rt, false).unwrap();
                        sum += rt.slice().len();
                    }
                    std::hint::black_box(sum);
                    t_bl.push(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
        }
    }
    let med = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (v[v.len() / 2], v[0], v[v.len() - 1])
    };
    let (m1, m1lo, m1hi) = med(&mut t_m1);
    let (m2, m2lo, m2hi) = med(&mut t_m2);
    let (bl, bllo, blhi) = med(&mut t_bl);
    println!("  scan Master-A1 : med={:6.2} ms  [{:6.2}, {:6.2}]  ratio=1.0000x", m1, m1lo, m1hi);
    println!("  scan Master-A2 : med={:6.2} ms  [{:6.2}, {:6.2}]  ratio={:.4}x (A/A)", m2, m2lo, m2hi, m2 / m1);
    println!("  scan HuffMonBL : med={:6.2} ms  [{:6.2}, {:6.2}]  ratio={:.4}x", bl, bllo, blhi, bl / m1);
}

fn run_stage_attribution(
    label: &str,
    data: &[u8],
    locs: &[BlockLoc],
    classes: &[(BlockClass, usize, u32)],
    six_indices: &[(usize, &'static str)],
) {
    println!("\n=== STAGE ATTRIBUTION: {label} (1T, best of 5 per block) ===");
    let k = Kernels::detect();
    let mut syms_buf = Vec::new();
    let mut runs = runs::Runs::default();
    let mut rt = RunTokens::default();
    let mut maxruns = Vec::new();
    let mut open_copy = OpenKernel::default();
    let mut open_prod = OpenCells::default();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut out = Vec::new();

    #[derive(Default)]
    struct Acc {
        blocks: usize,
        rows: usize,
        scan_master: Duration,
        scan_monoid_bl: Duration,
        front_master: Duration,
        mtf_u64_maxruns: Duration,
        open_prod: Duration,
        open_copy_recs: OpenTimings,
        open_b0_direct: OpenTimings,
        open_b1_striped: OpenTimings,
        open_b2_maxruns: OpenTimings,
        rle: Duration,
        crc: Duration,
    }
    let mut by_cls: [Acc; 4] = Default::default();
    let mut six_rows = Vec::new();

    let passes = 5;
    for (idx, &loc) in locs.iter().enumerate() {
        let (cls, n_blk, p_blk) = classes[idx];
        let ci = match cls {
            BlockClass::OpenAdmitNone => 0,
            BlockClass::OpenAdmitDense => 1,
            BlockClass::RunRefusedLazy => 2,
            BlockClass::ColumnRoute => 3,
        };
        let acc = &mut by_cls[ci];
        acc.blocks += 1;
        acc.rows += n_blk;

        let mut best_scan_m = Duration::MAX;
        let mut blk_syms: Option<Symbols> = None;
        for _ in 0..passes {
            let t0 = Instant::now();
            let mut br = Bits::at(data, loc.start_bit);
            let sb = std::mem::take(&mut syms_buf);
            let Item::Block(b) = read_item(&mut br, loc.level, sb).unwrap() else {
                panic!()
            };
            let d = t0.elapsed();
            if d < best_scan_m {
                best_scan_m = d;
            }
            if blk_syms.is_none() {
                blk_syms = Some(b);
            } else {
                syms_buf = b.syms;
            }
        }
        let b = blk_syms.unwrap();
        acc.scan_master += best_scan_m;

        let mut best_scan_bl = Duration::MAX;
        for _ in 0..passes {
            let t0 = Instant::now();
            let mut reg = Reg::at(data, loc.start_bit);
            read_block_monoid_bl(data, &mut reg, loc.level, &mut rt, false).unwrap();
            let d = t0.elapsed();
            if d < best_scan_bl {
                best_scan_bl = d;
            }
        }
        acc.scan_monoid_bl += best_scan_bl;

        let mut best_front = Duration::MAX;
        let is_run_route = runs::takes_runs(&b);
        for _ in 0..passes {
            let t0 = Instant::now();
            if is_run_route {
                runs::front(&b, &mut runs, k).unwrap();
            } else {
                mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
            }
            let d = t0.elapsed();
            if d < best_front {
                best_front = d;
            }
        }
        acc.front_master += best_front;

        let mut best_mtf_u64 = Duration::MAX;
        for _ in 0..passes {
            let t0 = Instant::now();
            if is_run_route {
                tokens_to_maxruns(&rt, &mut maxruns);
            } else {
                tokens_to_column(&rt, &mut ll, &mut counts);
            }
            let d = t0.elapsed();
            if d < best_mtf_u64 {
                best_mtf_u64 = d;
            }
        }
        acc.mtf_u64_maxruns += best_mtf_u64;

        let mut best_open_prod = Duration::MAX;
        for _ in 0..passes {
            let t0 = Instant::now();
            if is_run_route {
                open_prod.invert_runs(&runs, n_blk, p_blk, &mut ll, k);
            } else {
                mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
                open_prod.invert_in_place(&mut ll, &counts, p_blk);
            }
            let d = t0.elapsed();
            if d < best_open_prod {
                best_open_prod = d;
            }
        }
        acc.open_prod += best_open_prod;

        let mut best_rec: Option<OpenTimings> = None;
        let mut best_b0: Option<OpenTimings> = None;
        let mut best_b1: Option<OpenTimings> = None;
        let mut best_b2: Option<OpenTimings> = None;
        if let Some(cap) = Cap::for_block(runs.records().len(), n_blk, p_blk) {
            if is_run_route {
                for _ in 0..passes {
                    let mut tm = OpenTimings::default();
                    if open_copy
                        .run_from_records(runs.records(), n_blk, p_blk, cap, &mut ll, Some(&mut tm))
                        .is_some()
                    {
                        if best_rec.map_or(true, |b| tm.total() < b.total()) {
                            best_rec = Some(tm);
                        }
                    }
                    let mut tm0 = OpenTimings::default();
                    if open_copy
                        .run_from_tokens_direct(&rt, cap, &mut ll, Some(&mut tm0))
                        .is_some()
                    {
                        if best_b0.map_or(true, |b| tm0.total() < b.total()) {
                            best_b0 = Some(tm0);
                        }
                    }
                    let mut tm1 = OpenTimings::default();
                    if open_copy
                        .run_from_tokens_striped(&rt, cap, &mut ll, Some(&mut tm1))
                        .is_some()
                    {
                        if best_b1.map_or(true, |b| tm1.total() < b.total()) {
                            best_b1 = Some(tm1);
                        }
                    }
                    let mut tm2 = OpenTimings::default();
                    if open_copy
                        .run_from_maxruns(&maxruns, n_blk, p_blk, cap, &mut ll, Some(&mut tm2))
                        .is_some()
                    {
                        if best_b2.map_or(true, |b| tm2.total() < b.total()) {
                            best_b2 = Some(tm2);
                        }
                    }
                }
                if let (Some(r0), Some(r1), Some(r2), Some(r3)) =
                    (best_rec, best_b0, best_b1, best_b2)
                {
                    acc.open_copy_recs.add(r0);
                    acc.open_b0_direct.add(r1);
                    acc.open_b1_striped.add(r2);
                    acc.open_b2_maxruns.add(r3);
                }
            }
        }

        let mut best_rle = Duration::MAX;
        let mut best_crc = Duration::MAX;
        for _ in 0..passes {
            out.clear();
            let t0 = Instant::now();
            rle::unrle(&ll, &mut out, k).unwrap();
            let dr = t0.elapsed();
            let t1 = Instant::now();
            let c = crc::crc32(&out, k);
            let dc = t1.elapsed();
            assert_eq!(c, b.crc);
            if dr < best_rle {
                best_rle = dr;
            }
            if dc < best_crc {
                best_crc = dc;
            }
        }
        acc.rle += best_rle;
        acc.crc += best_crc;

        if let Some(&(_, name)) = six_indices.iter().find(|&&(si, _)| si == idx) {
            six_rows.push((
                name,
                idx,
                n_blk,
                rt.slice().len(),
                best_scan_m,
                best_scan_bl,
                best_front,
                best_mtf_u64,
                best_rec.unwrap_or_default(),
                best_b0.unwrap_or_default().pass1,
                best_b1.unwrap_or_default().pass1,
                best_b2.unwrap_or_default().pass1,
                best_rle + best_crc,
            ));
        }
        syms_buf = b.syms;
    }

    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    let names = [
        "OpenAdmit(NONE)",
        "OpenAdmit(DENSE)",
        "RunRefused(Lazy)",
        "ColumnRoute",
    ];
    println!(
        "{:<18} {:>5} {:>8} | {:>7} {:>7} | {:>7} {:>7} | {:>7} | {:>6} {:>6}",
        "Class", "blks", "Mrows", "scan_M", "scanBL", "front_M", "mtfU64", "OpenPrd", "RLE1", "CRC"
    );
    let mut tot_rows = 0usize;
    let (mut tsm, mut tsbl, mut tfr, mut tmu, mut top, mut trle, mut tcrc) =
        (Duration::ZERO, Duration::ZERO, Duration::ZERO, Duration::ZERO, Duration::ZERO, Duration::ZERO, Duration::ZERO);
    let (mut trec, mut tb0, mut tb1, mut tb2) =
        (OpenTimings::default(), OpenTimings::default(), OpenTimings::default(), OpenTimings::default());
    for i in 0..4 {
        let a = &by_cls[i];
        tot_rows += a.rows;
        println!(
            "{:<18} {:>5} {:>8.2} | {:>7.2} {:>7.2} | {:>7.2} {:>7.2} | {:>7.2} | {:>6.2} {:>6.2}",
            names[i],
            a.blocks,
            a.rows as f64 / 1e6,
            ms(a.scan_master),
            ms(a.scan_monoid_bl),
            ms(a.front_master),
            ms(a.mtf_u64_maxruns),
            ms(a.open_prod),
            ms(a.rle),
            ms(a.crc),
        );
        tsm += a.scan_master;
        tsbl += a.scan_monoid_bl;
        tfr += a.front_master;
        tmu += a.mtf_u64_maxruns;
        top += a.open_prod;
        trle += a.rle;
        tcrc += a.crc;
        trec.add(a.open_copy_recs);
        tb0.add(a.open_b0_direct);
        tb1.add(a.open_b1_striped);
        tb2.add(a.open_b2_maxruns);
    }
    println!(
        "{:<18} {:>5} {:>8.2} | {:>7.2} {:>7.2} | {:>7.2} {:>7.2} | {:>7.2} | {:>6.2} {:>6.2}",
        "TOTAL",
        locs.len(),
        tot_rows as f64 / 1e6,
        ms(tsm),
        ms(tsbl),
        ms(tfr),
        ms(tmu),
        ms(top),
        ms(trle),
        ms(tcrc),
    );

    if trec.total() > Duration::ZERO {
        let ad_front = by_cls[0].front_master + by_cls[1].front_master;
        let ad_mtf = by_cls[0].mtf_u64_maxruns + by_cls[1].mtf_u64_maxruns;
        println!("\n=== OPEN SUB-STAGE BREAKDOWN (Admitted blocks, ms) ===");
        println!(
            "  Open(records)  [master] : pass1={:6.2}  elim={:6.2}  wire={:6.2}  emit={:6.2}  total={:6.2}  (+front_M={:6.2})",
            ms(trec.pass1), ms(trec.elim), ms(trec.wire), ms(trec.emit), ms(trec.total()), ms(trec.total() + ad_front)
        );
        println!(
            "  Open(B0 direct)[fused]  : pass1={:6.2}  elim={:6.2}  wire={:6.2}  emit={:6.2}  total={:6.2}  (MtfU64 inside pass1)",
            ms(tb0.pass1), ms(tb0.elim), ms(tb0.wire), ms(tb0.emit), ms(tb0.total())
        );
        println!(
            "  Open(B1 stripe)[L1-256] : pass1={:6.2}  elim={:6.2}  wire={:6.2}  emit={:6.2}  total={:6.2}  (L1-256 MtfU64+pass1)",
            ms(tb1.pass1), ms(tb1.elim), ms(tb1.wire), ms(tb1.emit), ms(tb1.total())
        );
        println!(
            "  Open(B2 maxrun)[2-pass] : pass1={:6.2}  elim={:6.2}  wire={:6.2}  emit={:6.2}  total={:6.2}  (+mtfU64={:6.2}, +front_M={:6.2})",
            ms(tb2.pass1), ms(tb2.elim), ms(tb2.wire), ms(tb2.emit), ms(tb2.total()),
            ms(tb2.total() + ad_mtf), ms(tb2.total() + ad_front)
        );
    }

    if !six_rows.is_empty() {
        println!("\n=== EXACT 6 AUTHENTICATED WIKIDATA BLOCKS (us) ===");
        let us = |d: Duration| d.as_secs_f64() * 1e6;
        println!(
            "{:<10} {:>4} {:>7} {:>6} | {:>6} {:>6} | {:>6} {:>6} | {:>6} {:>6} {:>6} {:>6} | {:>6} {:>6} {:>6} | {:>6}",
            "name", "idx", "n", "r", "scanM", "scanBL", "frntM", "mtf64", "p1_rec", "elim", "wire", "emit", "B0_p1", "B1_p1", "B2_p1", "RLE+C"
        );
        for (name, idx, n, r, sm, sbl, fr, mu, rec, b0p1, b1p1, b2p1, rc) in six_rows {
            println!(
                "{:<10} {:>4} {:>7} {:>6} | {:>6.1} {:>6.1} | {:>6.1} {:>6.1} | {:>6.1} {:>6.1} {:>6.1} {:>6.1} | {:>6.1} {:>6.1} {:>6.1} | {:>6.1}",
                name, idx, n, r, us(sm), us(sbl), us(fr), us(mu),
                us(rec.pass1), us(rec.elim), us(rec.wire), us(rec.emit),
                us(b0p1), us(b1p1), us(b2p1), us(rc)
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Arm {
    MasterA1,
    MasterA2,
    MonoidBLB0Direct,
    MonoidBLB1Striped,
    MonoidBLB2MaxRuns,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::MasterA1 => "Master-A1 (v0.5.11)",
            Arm::MasterA2 => "Master-A2 (A/A ctrl)",
            Arm::MonoidBLB0Direct => "Arm-B0 (HuffBL+Dir)",
            Arm::MonoidBLB1Striped => "Arm-B1 (HuffBL+L1)",
            Arm::MonoidBLB2MaxRuns => "Arm-B2 (HuffBL+Max)",
        }
    }
}

fn run_arm_1t(arm: Arm, data: &[u8], locs: &[BlockLoc]) -> (Duration, u32) {
    let t0 = Instant::now();
    let mut crcs = Vec::with_capacity(locs.len());
    match arm {
        Arm::MasterA1 | Arm::MasterA2 => {
            let mut w = MasterWorker::new();
            for &loc in locs {
                crcs.push(w.decode_block_full(data, loc));
            }
        }
        Arm::MonoidBLB0Direct => {
            let mut w = MonoidWorker::new(MonoidMode::Direct);
            for &loc in locs {
                crcs.push(w.decode_block_full(data, loc));
            }
        }
        Arm::MonoidBLB1Striped => {
            let mut w = MonoidWorker::new(MonoidMode::Striped);
            for &loc in locs {
                crcs.push(w.decode_block_full(data, loc));
            }
        }
        Arm::MonoidBLB2MaxRuns => {
            let mut w = MonoidWorker::new(MonoidMode::MaxRuns);
            for &loc in locs {
                crcs.push(w.decode_block_full(data, loc));
            }
        }
    }
    let dt = t0.elapsed();
    (dt, mibz2::combine(crcs))
}

fn run_arm_parallel_blocks(arm: Arm, data: &[u8], locs: &[BlockLoc], workers: usize) -> (Duration, u32) {
    let t0 = Instant::now();
    let next = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let crcs = Arc::new(Mutex::new(vec![0u32; locs.len()]));
    std::thread::scope(|s| {
        for _ in 0..workers {
            let next = Arc::clone(&next);
            let crcs = Arc::clone(&crcs);
            s.spawn(move || {
                let mut local = Vec::new();
                match arm {
                    Arm::MasterA1 | Arm::MasterA2 => {
                        let mut w = MasterWorker::new();
                        loop {
                            let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if i >= locs.len() {
                                break;
                            }
                            local.push((i, w.decode_block_full(data, locs[i])));
                        }
                    }
                    Arm::MonoidBLB0Direct | Arm::MonoidBLB1Striped | Arm::MonoidBLB2MaxRuns => {
                        let mode = match arm {
                            Arm::MonoidBLB0Direct => MonoidMode::Direct,
                            Arm::MonoidBLB1Striped => MonoidMode::Striped,
                            _ => MonoidMode::MaxRuns,
                        };
                        let mut w = MonoidWorker::new(mode);
                        loop {
                            let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if i >= locs.len() {
                                break;
                            }
                            local.push((i, w.decode_block_full(data, locs[i])));
                        }
                    }
                }
                let mut g = crcs.lock().unwrap();
                for (i, c) in local {
                    g[i] = c;
                }
            });
        }
    });
    let dt = t0.elapsed();
    let comb = mibz2::combine(crcs.lock().unwrap().iter().copied());
    (dt, comb)
}

fn run_arm_pipelined(arm: Arm, data: &[u8], locs: &[BlockLoc], workers: usize) -> (Duration, u32) {
    let t0 = Instant::now();
    let crcs = Arc::new(Mutex::new(vec![0u32; locs.len()]));
    let slots = workers * 4;
    match arm {
        Arm::MasterA1 | Arm::MasterA2 => {
            let (tx, rx) = std::sync::mpsc::sync_channel::<(usize, Symbols)>(slots);
            let (ftx, frx) = std::sync::mpsc::sync_channel::<Vec<u16>>(slots);
            let rx = Arc::new(Mutex::new(rx));
            std::thread::scope(|s| {
                for _ in 0..workers {
                    let rx = Arc::clone(&rx);
                    let ftx = ftx.clone();
                    let crcs = Arc::clone(&crcs);
                    s.spawn(move || {
                        let mut w = MasterWorker::new();
                        let mut local = Vec::new();
                        loop {
                            let job = rx.lock().unwrap().recv();
                            let Ok((i, syms)) = job else { break };
                            local.push((i, w.finish_symbols(&syms)));
                            let _ = ftx.try_send(syms.syms);
                        }
                        let mut g = crcs.lock().unwrap();
                        for (i, c) in local {
                            g[i] = c;
                        }
                    });
                }
                drop(ftx);
                for (i, &loc) in locs.iter().enumerate() {
                    let mut br = Bits::at(data, loc.start_bit);
                    let reuse = frx.try_recv().unwrap_or_default();
                    let Item::Block(b) = read_item(&mut br, loc.level, reuse).unwrap() else {
                        panic!()
                    };
                    tx.send((i, b)).unwrap();
                }
                drop(tx);
            });
        }
        Arm::MonoidBLB0Direct | Arm::MonoidBLB1Striped | Arm::MonoidBLB2MaxRuns => {
            let mode = match arm {
                Arm::MonoidBLB0Direct => MonoidMode::Direct,
                Arm::MonoidBLB1Striped => MonoidMode::Striped,
                _ => MonoidMode::MaxRuns,
            };
            let (tx, rx) = std::sync::mpsc::sync_channel::<(usize, RunTokens)>(slots);
            let (ftx, frx) = std::sync::mpsc::sync_channel::<RunTokens>(slots);
            let rx = Arc::new(Mutex::new(rx));
            std::thread::scope(|s| {
                for _ in 0..workers {
                    let rx = Arc::clone(&rx);
                    let ftx = ftx.clone();
                    let crcs = Arc::clone(&crcs);
                    s.spawn(move || {
                        let mut w = MonoidWorker::new(mode);
                        let mut local = Vec::new();
                        loop {
                            let job = rx.lock().unwrap().recv();
                            let Ok((i, rt)) = job else { break };
                            w.rt = rt;
                            local.push((i, w.finish_tokens()));
                            let recycled = std::mem::take(&mut w.rt);
                            let _ = ftx.try_send(recycled);
                        }
                        let mut g = crcs.lock().unwrap();
                        for (i, c) in local {
                            g[i] = c;
                        }
                    });
                }
                drop(ftx);
                for (i, &loc) in locs.iter().enumerate() {
                    let mut reg = Reg::at(data, loc.start_bit);
                    let mut rt = frx.try_recv().unwrap_or_default();
                    read_block_monoid_bl(data, &mut reg, loc.level, &mut rt, false).unwrap();
                    tx.send((i, rt)).unwrap();
                }
                drop(tx);
            });
        }
    }
    let dt = t0.elapsed();
    let comb = mibz2::combine(crcs.lock().unwrap().iter().copied());
    (dt, comb)
}

fn median_ms(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    vals[vals.len() / 2]
}

fn run_ab_suite(label: &str, data: &[u8], locs: &[BlockLoc], rounds: usize) {
    let arms = [
        Arm::MasterA1,
        Arm::MasterA2,
        Arm::MonoidBLB0Direct,
        Arm::MonoidBLB1Striped,
        Arm::MonoidBLB2MaxRuns,
    ];
    println!("\n=== INTERLEAVED A/A + A/B SUITE: {label} ({rounds} rounds) ===");
    for &(mode_name, mode_id) in &[
        ("1T serial", 0),
        ("4W block-parallel", 1),
        ("4W scanner+workers (pooled buffers)", 2),
    ] {
        let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(rounds); arms.len()];
        let mut ref_crc: Option<u32> = None;
        for &arm in &arms {
            let _ = run_arm_1t(arm, data, &locs[..locs.len().min(6)]);
        }
        for r in 0..rounds {
            let order: Vec<usize> = if r % 2 == 0 {
                (0..arms.len()).collect()
            } else {
                (0..arms.len()).rev().collect()
            };
            for ai in order {
                let (dt, crc) = match mode_id {
                    0 => run_arm_1t(arms[ai], data, locs),
                    1 => run_arm_parallel_blocks(arms[ai], data, locs, 4),
                    _ => run_arm_pipelined(arms[ai], data, locs, 4),
                };
                if let Some(rc) = ref_crc {
                    assert_eq!(crc, rc, "CRC mismatch in {mode_name} {:?}", arms[ai]);
                } else {
                    ref_crc = Some(crc);
                }
                times[ai].push(dt.as_secs_f64() * 1e3);
            }
        }
        let mut base_vec = times[0].clone();
        let base_med = median_ms(&mut base_vec);
        println!("  -- {mode_name} --");
        for (ai, arm) in arms.iter().enumerate() {
            let mut v = times[ai].clone();
            let med = median_ms(&mut v);
            let min = v[0];
            let max = v[v.len() - 1];
            let ratio = med / base_med;
            println!(
                "    {:<24} med={:7.2} ms  [{:7.2}, {:7.2}]  ratio={:.4}x",
                arm.name(),
                med,
                min,
                max,
                ratio
            );
        }
    }
}

fn main() {
    verify_exhaustive();

    let wikidata_path = "/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2";
    let data = std::fs::read(wikidata_path).expect("read corpora/wikidata.bz2");
    let locs = scan_all_blocks(&data);
    println!(
        "Loaded corpora/wikidata.bz2: {} compressed bytes, {} blocks",
        data.len(),
        locs.len()
    );

    let classes = verify_corpus(&data, &locs);
    let six = find_manifest_six(&classes);
    println!("Matched manifest 6 blocks: {:?}", six);
    assert_eq!(six.len(), 6, "all 6 manifest blocks must match");

    run_scanner_ab("Full 431-block wikidata.bz2", &data, &locs, 7);
    run_stage_attribution("Full 431-block wikidata.bz2", &data, &locs, &classes, &six);

    // High-entropy incompressible blocks (!takes_runs column route)
    let mut hi_blocks = Vec::new();
    let mut rng = 0x9e37_79b9_7f4a_7c15u64;
    for _ in 0..4usize {
        let mut col = vec![0u8; 300_000];
        for b in &mut col {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            *b = rng as u8;
        }
        let inv = plain_inv(&col, 12345);
        let mut out_rle = Vec::new();
        rle::unrle(&inv, &mut out_rle, Kernels::detect()).unwrap();
        let c = crc::crc32(&out_rle, Kernels::detect());
        hi_blocks.push(encode_column_as_bz2(&col, 12345, c));
    }
    let hi_concat: Vec<u8> = hi_blocks.concat();
    let hi_locs = scan_all_blocks(&hi_concat);
    let hi_cls = verify_corpus(&hi_concat, &hi_locs);
    run_scanner_ab("4x300KB High-Entropy (!takes_runs)", &hi_concat, &hi_locs, 7);
    run_stage_attribution("4x300KB High-Entropy (!takes_runs)", &hi_concat, &hi_locs, &hi_cls, &[]);

    run_ab_suite("Full 431-block corpora/wikidata.bz2", &data, &locs, 5);
    let six_locs: Vec<BlockLoc> = six.iter().map(|&(i, _)| locs[i]).collect();
    run_ab_suite("Exact 6 authenticated Wikidata blocks", &data, &six_locs, 9);
}
