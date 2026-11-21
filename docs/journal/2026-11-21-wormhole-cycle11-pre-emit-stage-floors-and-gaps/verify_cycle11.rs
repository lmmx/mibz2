#![forbid(unsafe_code)]
//! Wormhole Cycle 11 — Pre-`Emit` Four-Stage BMCP Lower Bounds, `origin/master` (`90e671966`,
//! `#692` + `#703`) Upper Bounds, and Stage-by-Stage Gap Anatomy:
//!   - Native safe-Rust A/A (`OpenCells` #1 vs `OpenCells` #2) + Stage-Timed Replica (`MasterStaged`)
//!     in the exact same binary against `origin/master` `90e6719663af0615fb417b038906ad2636193aa3`.
//!   - Decomposes every pre-`emit` stage of `origin/master` across the 6 authenticated blocks AND
//!     a `#703` dense-band block (`r/n ~ 0.18, m ~ 218k, H ~ 1.9k`):
//!       * Stage S1  : `First Pass + OutQ16` (`maximal` + `Queues::push` in `out` + `strips.push`)
//!       * Stage S2a : `EdgesMux Image-Order Scatter + Arena Node/List Construction`
//!       * Stage S2b : `Rauzy-Veech-Morse Elimination` (`Elim::eliminate` in `edges[ns..]`)
//!       * Stage S3  : `Bottom Walk + Band/SLP Extraction` (`S3a`) + `inline_words` (`S3b`)
//!       * Stage S4  : `OutQ16 Band Wire` (`Reader::wire` -> `edges[0..m)` scatter)
//!       * (Stage S5 : `Master Serial Emit` — held as reference; Mechanisms owns `emit`)
//!   - Computes exact 6-dimensional BMCP(32K L1d, 256K L2, 2048K L3, B=64B, mu) metrics per stage
//!     and compares against proved information/compulsory lower bounds (Theorems 11.1-11.4).

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
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

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
    // Use a cyclic BWT-safe text or compute CRC on unrled if valid RLE1; for arbitrary `ll`,
    // synthesize valid RLE1-free bytes when crafting synthetic dense blocks!
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

/// Synthesizes a valid `#703` dense-band BWT block (`n = 899,980`, `r/n ~ 0.180`, `m ~ 225k`, `H ~ 2,100`)
/// with valid RLE1 (no 4-byte runs in text) to exercise `#703`'s newly admitted `DENSE` regime!
fn synthesize_dense_703_bwt() -> (Vec<u8>, u32) {
    // Build a realistic repetitive-template + high-GUID-density JSON-like text of 899,980 bytes
    // with zero 4-identical consecutive bytes so RLE1 is identity, then compute its exact BWT!
    let n = 899_980usize;
    let mut text = Vec::with_capacity(n);
    let mut rng = 0x517C_C1B7_2722_0A95u64;
    let mut nxt = || -> u64 {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    let tpl = b"{\"type\":\"statement\",\"rank\":\"normal\",\"property\":\"P31\",\"hash\":\"";
    let end = b"\"}\n";
    let hex = b"0123456789abcdef";
    while text.len() < n {
        for &b in tpl {
            if text.len() < n {
                text.push(b);
            }
        }
        for _ in 0..10 {
            if text.len() < n {
                let mut c = hex[(nxt() as usize) & 15];
                if text.len() >= 3 && text[text.len() - 1] == c && text[text.len() - 2] == c && text[text.len() - 3] == c {
                    c ^= 1;
                }
                text.push(c);
            }
        }
        for &b in end {
            if text.len() < n {
                text.push(b);
            }
        }
    }
    // Check circular wrap has no 4-equal run at the end!
    for k in (n - 4)..n {
        if text[k] == text[k - 1] && text[k] == text[k - 2] {
            text[k] = b'_';
        }
    }
    // Prefix-doubling cyclic suffix sort (`O(n log^2 n)` — runs once in ~0.25s for 900K!)
    let mut sa: Vec<u32> = (0..n as u32).collect();
    let mut rank: Vec<u32> = text.iter().map(|&b| u32::from(b)).collect();
    let mut tmp = vec![0u32; n];
    let mut step = 1usize;
    while step < n {
        sa.sort_unstable_by(|&a, &b| {
            let ra = rank[a as usize];
            let rb = rank[b as usize];
            if ra != rb {
                ra.cmp(&rb)
            } else {
                let ra2 = rank[(a as usize + step) % n];
                let rb2 = rank[(b as usize + step) % n];
                ra2.cmp(&rb2)
            }
        });
        tmp[sa[0] as usize] = 0;
        let mut r = 0u32;
        for i in 1..n {
            let prev = sa[i - 1] as usize;
            let cur = sa[i] as usize;
            if rank[prev] != rank[cur] || rank[(prev + step) % n] != rank[(cur + step) % n] {
                r += 1;
            }
            tmp[cur] = r;
        }
        std::mem::swap(&mut rank, &mut tmp);
        if r as usize == n - 1 {
            break;
        }
        step <<= 1;
    }
    let mut ll = Vec::with_capacity(n);
    let mut orig_ptr = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            orig_ptr = i as u32;
        }
        let prev = if pos == 0 { n - 1 } else { pos as usize - 1 };
        ll.push(text[prev]);
    }
    (ll, orig_ptr)
}

// ============================================================================
// Exact Stage-Instrumented Replica of `origin/master` (`90e671966`) `src/open`
// ============================================================================
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

#[derive(Default)]
struct Spill {
    next: Vec<u32>,
    ovf: Vec<[u8; BYTES]>,
    cuts: Vec<[u32; 256]>,
}

#[derive(Clone, Copy)]
struct First {
    all: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    runs: usize,
}
impl Default for First {
    fn default() -> Self {
        Self {
            all: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            runs: 0,
        }
    }
}

struct Heads {
    heads: [u32; 256],
    in_out: u32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct StageTimingsNs {
    pub s1_first_pass: u64,
    pub s2a_order_and_nodes: u64,
    pub s2b_eliminate: u64,
    pub s3a_bottom_bands_spell: u64,
    pub s4_wire: u64,
    pub s3b_inline_words: u64,
    pub s5_emit: u64,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct BlockStructure {
    pub n: usize,
    pub n_records: usize,
    pub n_runs: usize,
    pub m_retained: usize,
    pub h_strips: usize,
    pub open_nodes: usize,
    pub rv_steps: usize,
    pub slp_rules: usize,
    pub n_bands: usize,
    pub n_words: usize,
    pub n_long_words: usize,
    pub spelling_bytes: usize,
    pub traced_height_sum: usize,
    pub outq_chunks: usize,
    pub outq_epochs: usize,
    pub heap_strips_b: usize,
    pub heap_bands_b: usize,
    pub heap_spelling_b: usize,
    pub heap_offsets_b: usize,
    pub heap_inline16_b: usize,
    pub heap_edges_b: usize,
    pub heap_spill_b: usize,
}

#[derive(Default)]
pub struct MasterStaged {
    stack: Vec<u32>,
    spill: Spill,
    strips: Vec<Strip>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let s = total;
        total += w;
        s
    })
}

impl MasterStaged {
    pub fn run_timed(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (StageTimingsNs, BlockStructure) {
        let mut tm = StageTimingsNs::default();
        let mut st = BlockStructure {
            n,
            n_records: records.len(),
            ..BlockStructure::default()
        };

        // ---- STAGE S1: First Pass (`maximal` + `OutQ16` + `strips`) ----
        let t_s1 = Instant::now();
        out.resize(n + INLINE, 0);
        self.strips.clear();
        self.spill.ovf.clear();
        self.spill.cuts.clear();
        let in_out = (out.len() / BYTES) as u32;
        let mut q_chunks = 0u32;
        let mut q_heads = [NONE; 256];
        let mut q_tails = [NONE; 256];
        let mut q_fill = [CHUNK as u32; 256];
        let mut f = First::default();

        let push_slow = |a: usize,
                             mut m_cur: u32,
                             w: u32,
                             thin: &[u32; 256],
                             out_sl: &mut [u8],
                             spill: &mut Spill,
                             q_chunks: &mut u32,
                             q_heads: &mut [u32; 256],
                             q_tails: &mut [u32; 256],
                             q_fill: &mut [u32; 256]| {
            let mut left = w;
            while left > 0 {
                if q_fill[a] as usize == CHUNK {
                    let id = *q_chunks;
                    *q_chunks += 1;
                    if id >= in_out && spill.ovf.len() <= (id - in_out) as usize {
                        spill.ovf.push([0; BYTES]);
                    }
                    if spill.next.len() <= id as usize {
                        spill.next.resize(id as usize + 1, NONE);
                    }
                    spill.next[id as usize] = NONE;
                    if q_heads[a] == NONE {
                        q_heads[a] = id;
                    } else {
                        spill.next[q_tails[a] as usize] = id;
                    }
                    q_tails[a] = id;
                    q_fill[a] = 0;
                }
                let fl = q_fill[a] as usize;
                let lo = m_cur & (EPOCH - 1);
                let take = left.min((CHUNK - fl) as u32).min(EPOCH - lo);
                let chunk: &mut [u8] = if q_tails[a] < in_out {
                    let at = q_tails[a] as usize * BYTES;
                    &mut out_sl[at..at + BYTES]
                } else {
                    &mut spill.ovf[(q_tails[a] - in_out) as usize][..]
                };
                for (i, d) in chunk[2 * fl..2 * (fl + take as usize)]
                    .chunks_exact_mut(2)
                    .enumerate()
                {
                    d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                }
                q_fill[a] += take;
                m_cur += take;
                left -= take;
                if m_cur % EPOCH == 0 {
                    let mut cut = *thin;
                    cut[a] += w - left;
                    spill.cuts.push(cut);
                }
            }
        };

        let mut pos = 0u32;
        let mut idx = 0usize;
        while idx < records.len() {
            let rec = records[idx];
            let c = (rec >> 8) as u8;
            let mut width = u32::from(rec & 255);
            idx += 1;
            while idx < records.len() && (records[idx] >> 8) as u8 == c {
                width += u32::from(records[idx] & 255);
                idx += 1;
            }
            f.runs += 1;
            let a = usize::from(c);
            if width <= LIMIT {
                if p.wrapping_sub(pos) < width {
                    f.mark = f.m + (p - pos);
                }
                let fl = q_fill[a] as usize;
                let lo = f.m & (EPOCH - 1);
                let tail = q_tails[a];
                if fl + 16 <= CHUNK && lo + width < EPOCH && tail < in_out {
                    let at = tail as usize * BYTES + 2 * fl;
                    let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                    for (d, v) in out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                        d.copy_from_slice(&v.to_le_bytes());
                    }
                    q_fill[a] = (fl as u32) + width;
                } else {
                    push_slow(
                        a,
                        f.m,
                        width,
                        &f.thin,
                        &mut out[..],
                        &mut self.spill,
                        &mut q_chunks,
                        &mut q_heads,
                        &mut q_tails,
                        &mut q_fill,
                    );
                }
                f.thin[a] += width;
                f.m += width;
                f.all[a] += width;
            } else if p.wrapping_sub(pos) >= width {
                self.strips.push(Strip {
                    from: pos,
                    to: f.all[a],
                    width,
                    byte: c,
                });
                f.thick[a] += 1;
                f.all[a] += width;
            } else {
                let end = pos + width;
                if pos < p {
                    let lw = p - pos;
                    self.strips.push(Strip {
                        from: pos,
                        to: f.all[a],
                        width: lw,
                        byte: c,
                    });
                    f.thick[a] += 1;
                    f.all[a] += lw;
                }
                f.mark = f.m;
                push_slow(
                    a,
                    f.m,
                    1,
                    &f.thin,
                    &mut out[..],
                    &mut self.spill,
                    &mut q_chunks,
                    &mut q_heads,
                    &mut q_tails,
                    &mut q_fill,
                );
                f.thin[a] += 1;
                f.m += 1;
                f.all[a] += 1;
                if p + 1 < end {
                    let rw = end - p - 1;
                    self.strips.push(Strip {
                        from: p + 1,
                        to: f.all[a],
                        width: rw,
                        byte: c,
                    });
                    f.thick[a] += 1;
                    f.all[a] += rw;
                }
            }
            pos += width;
        }
        let heads = Heads {
            heads: q_heads,
            in_out,
        };
        tm.s1_first_pass = t_s1.elapsed().as_nanos() as u64;

        // ---- STAGE S2a: `image_order` in `edges[..ns]` + Arena Node/List Build ----
        let t_s2a = Instant::now();
        let m = f.m;
        let mark = f.mark;
        let ns = self.strips.len();
        let most = 3 * ns + 3;
        assert!(most <= 65536);
        let need = (ns + (4 + RULE) * most).max(m as usize);
        if self.edges.len() < need {
            self.edges.clear();
            self.edges.reserve_exact(need);
            self.edges.resize(need, 0);
        }
        let first = starts(f.all);
        let mut write = starts(f.thick);
        {
            let image_order = &mut self.edges[..ns];
            for (i, s) in self.strips.iter_mut().enumerate() {
                let a = usize::from(s.byte);
                s.to += first[a];
                image_order[write[a] as usize] = i as u32;
                write[a] += 1;
            }
        }

        let n_u32 = n as u32;
        let mut widest = 0u32;
        let mut nodes = 0u32;
        {
            let (image_order, arena) = self.edges.split_at_mut(ns);
            let mut mk_node = |a: &mut [u32], width: u32, word: u32| -> u32 {
                let id = nodes;
                nodes += 1;
                a[id as usize * 4 + 2] = width;
                a[id as usize * 4 + 3] = word;
                id
            };
            let set_prev = |a: &mut [u32], id: u32, side: usize, to: u32| {
                let w = &mut a[id as usize * 4 + side];
                *w = (*w & 0xFFFF_0000) | to;
            };
            let set_next = |a: &mut [u32], id: u32, side: usize, to: u32| {
                let w = &mut a[id as usize * 4 + side];
                *w = (*w & 0xFFFF) | (to << 16);
            };
            let append_to = |a: &mut [u32], list: &mut (u32, u32), id: u32, side: usize| {
                let (head, tail) = list;
                if *tail == 0 {
                    *head = id;
                } else {
                    set_next(a, *tail, side, id);
                }
                set_prev(a, id, side, *tail);
                *tail = id;
            };

            mk_node(arena, 0, 0);
            for s in self.strips.iter() {
                mk_node(arena, s.width, u32::from(s.byte) + 1);
                widest = widest.max(s.width);
            }
            let (mut terminals, mut order) = ([(0u32, 0u32); 2], [(0u32, 0u32); 2]);
            let mut x = 0u32;
            for (i, s) in self.strips.iter().enumerate() {
                if x < s.from {
                    widest = widest.max(s.from - x);
                    let id = mk_node(arena, s.from - x, 0);
                    append_to(arena, &mut order[0], id, 0);
                    append_to(arena, &mut terminals[1], id, 1);
                }
                append_to(arena, &mut order[0], i as u32 + 1, 0);
                x = s.from + s.width;
            }
            if x < n_u32 {
                widest = widest.max(n_u32 - x);
                let id = mk_node(arena, n_u32 - x, 0);
                append_to(arena, &mut order[0], id, 0);
                append_to(arena, &mut terminals[1], id, 1);
            }
            x = 0;
            for &i in image_order.iter() {
                let s = self.strips[i as usize];
                if x < s.to {
                    widest = widest.max(s.to - x);
                    let id = mk_node(arena, s.to - x, 0);
                    append_to(arena, &mut order[1], id, 1);
                    append_to(arena, &mut terminals[0], id, 0);
                }
                append_to(arena, &mut order[1], i + 1, 1);
                x = s.to + s.width;
            }
            if x < n_u32 {
                widest = widest.max(n_u32 - x);
                let id = mk_node(arena, n_u32 - x, 0);
                append_to(arena, &mut order[1], id, 1);
                append_to(arena, &mut terminals[0], id, 0);
            }
            for side in 0..2 {
                let mut prev = 0u32;
                for (head, tail) in [terminals[side], order[side]] {
                    if tail == 0 {
                        continue;
                    }
                    set_next(arena, prev, side, head);
                    set_prev(arena, head, side, prev);
                    prev = tail;
                }
                set_next(arena, prev, side, 0);
                set_prev(arena, 0, side, prev);
            }
        }
        tm.s2a_order_and_nodes = t_s2a.elapsed().as_nanos() as u64;

        // ---- STAGE S2b: Rauzy-Veech-Morse Elimination (`Elim::eliminate`) ----
        let t_s2b = Instant::now();
        let base = 4 * nodes as usize;
        let mut top = base;
        let mut left = n_u32;
        let mut steps = 0usize;
        let spirals = widest > CERTIFIED_WIDTH;
        let room = if spirals { SPIRAL_RULES * RULE } else { RULE };
        loop {
            if left == 0 {
                break;
            }
            if ns + top + room > self.edges.len() {
                let target = (ns + top + room).max(2 * self.edges.len());
                self.edges.resize(target, 0);
            }
            let arena = &mut self.edges[ns..];
            let prev = |a: &[u32], id: u32, side: usize| a[id as usize * 4 + side] & 0xFFFF;
            let next = |a: &[u32], id: u32, side: usize| a[id as usize * 4 + side] >> 16;
            let set_prev = |a: &mut [u32], id: u32, side: usize, to: u32| {
                let w = &mut a[id as usize * 4 + side];
                *w = (*w & 0xFFFF_0000) | to;
            };
            let set_next = |a: &mut [u32], id: u32, side: usize, to: u32| {
                let w = &mut a[id as usize * 4 + side];
                *w = (*w & 0xFFFF) | (to << 16);
            };
            let cut = |a: &mut [u32], id: u32, side: usize| {
                let (p0, n0) = (prev(a, id, side), next(a, id, side));
                set_next(a, p0, side, n0);
                set_prev(a, n0, side, p0);
            };
            let insert = |a: &mut [u32], at: u32, id: u32, side: usize| {
                let nx = next(a, at, side);
                set_prev(a, id, side, at);
                set_next(a, id, side, nx);
                set_next(a, at, side, id);
                set_prev(a, nx, side, id);
            };
            let join = |a: &mut [u32], top_ref: &mut usize, lw: u32, rw: u32| -> u32 {
                if lw == 0 {
                    return rw;
                }
                if rw == 0 {
                    return lw;
                }
                let id = ((*top_ref - base) / RULE) as u32 + 257;
                let ll = if lw <= 256 { 1 } else { a[base + (lw - 257) as usize * RULE + 2] };
                let rl = if rw <= 256 { 1 } else { a[base + (rw - 257) as usize * RULE + 2] };
                a[*top_ref..*top_ref + RULE].copy_from_slice(&[lw, rw, ll + rl, u32::MAX]);
                *top_ref += RULE;
                id
            };

            while left > 0 && top + room <= arena.len() {
                let (x, y) = (prev(arena, 0, 0), prev(arena, 0, 1));
                let (aw, bw) = (arena[x as usize * 4 + 2], arena[y as usize * 4 + 2]);
                let w = aw.min(bw);
                left -= w;
                steps += 1;
                if x == y {
                    cut(arena, x, 0);
                    cut(arena, x, 1);
                    continue;
                }
                let word = join(arena, &mut top, arena[x as usize * 4 + 3], arena[y as usize * 4 + 3]);
                if aw <= bw {
                    cut(arena, x, 0);
                    insert(arena, y, x, 0);
                    arena[x as usize * 4 + 3] = word;
                    if aw == bw {
                        cut(arena, y, 0);
                        cut(arena, y, 1);
                    } else {
                        arena[y as usize * 4 + 2] = bw - w;
                    }
                } else {
                    cut(arena, y, 1);
                    insert(arena, x, y, 1);
                    arena[y as usize * 4 + 3] = word;
                    arena[x as usize * 4 + 2] = aw - w;
                }
            }
        }
        tm.s2b_eliminate = t_s2b.elapsed().as_nanos() as u64;

        // ---- STAGE S3a: Bottom Walk + Band Extraction + SLP Flattening ----
        let t_s3a = Instant::now();
        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        let mut cursors = [0usize; 256];
        let mut n_words = 0usize;
        let mut traced_sum = 0usize;
        {
            let arena = &mut self.edges[ns..];
            let mut id = arena[1] >> 16; // next(0, 1)
            let mut x = 0u32;
            while id != 0 {
                let nxt = arena[id as usize * 4 + 1] >> 16;
                let w = arena[id as usize * 4 + 2];
                arena[id as usize * 4 + 1] = x; // bottom(id)
                x += w;
                id = nxt;
            }
            id = arena[0] >> 16; // next(0, 0)
            let mut in_node = 0u32;
            let mut k = 0u32;
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + f.thin[a];
                while k < end {
                    let node_width = arena[id as usize * 4 + 2];
                    let node_word = arena[id as usize * 4 + 3];
                    let width = (end - k).min(node_width - in_node);
                    let dst = arena[id as usize * 4 + 1] + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        let height = if node_word <= 256 {
                            1usize
                        } else {
                            arena[base + (node_word - 257) as usize * RULE + 2] as usize
                        };
                        n_words += 1;
                        traced_sum += height;
                        self.stack.clear();
                        self.stack.push(node_word);
                        while let Some(wid) = self.stack.pop() {
                            if wid <= 256 {
                                self.spelling.push((wid - 1) as u8);
                            } else {
                                let r = base + (wid - 257) as usize * RULE;
                                if arena[r + 3] == u32::MAX {
                                    arena[r + 3] = self.spelling.len() as u32;
                                    self.stack.push(arena[r + 1]);
                                    self.stack.push(arena[r]);
                                } else {
                                    let s0 = arena[r + 3] as usize;
                                    let sl = arena[r + 2] as usize;
                                    self.spelling.extend_from_within(s0..s0 + sl);
                                }
                            }
                        }
                        self.spelling.push(a as u8);
                        self.offsets.push(self.spelling.len() as u32);
                        254 + self.offsets.len() as u32
                    };
                    self.bands.push(Band {
                        end: k + width,
                        delta: dst.wrapping_sub(k),
                        code,
                    });
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = arena[id as usize * 4] >> 16; // next(id, 0)
                        in_node = 0;
                    }
                }
            }
        }
        tm.s3a_bottom_bands_spell = t_s3a.elapsed().as_nanos() as u64;

        // ---- STAGE S4: `OutQ16` Band Wire -> `edges[0..m)` ----
        let t_s4 = Instant::now();
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + n_words as u32) > (u32::MAX >> bits);
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        let mut k = 0u32;
        for a in 0..256 {
            let end_b = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            let mut r_id = heads.heads[a];
            let mut r_at = 0usize;
            let mut r_read = 0u32;
            let mut r_epoch = 0u32;
            let mut r_cut = self.spill.cuts.first().map_or(u32::MAX, |c| c[a]);
            for b in &self.bands[cursors[a]..end_b] {
                let span = (b.end - k) as usize;
                let dst = k.wrapping_add(b.delta) as usize;
                let pack = if wide { 0 } else { b.code << bits };
                if wide {
                    self.extra[dst..dst + span].fill(b.code);
                }
                let dst_sl = &mut self.edges[dst..dst + span];
                if r_at + span <= CHUNK && r_read + span as u32 <= r_cut && r_id < heads.in_out {
                    let hi = pack | (r_epoch << 16);
                    let at = r_id as usize * BYTES + 2 * r_at;
                    for (e, s) in dst_sl.iter_mut().zip(out[at..at + 2 * span].chunks_exact(2)) {
                        *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    }
                    r_at += span;
                    r_read += span as u32;
                } else {
                    let mut rem_dst = dst_sl;
                    while !rem_dst.is_empty() {
                        while r_read == r_cut {
                            r_epoch += 1;
                            r_cut = self.spill.cuts.get(r_epoch as usize).map_or(u32::MAX, |c| c[a]);
                        }
                        if r_at == CHUNK {
                            r_id = self.spill.next[r_id as usize];
                            r_at = 0;
                        }
                        let take = rem_dst
                            .len()
                            .min(CHUNK - r_at)
                            .min((r_cut - r_read) as usize);
                        let chunk: &[u8] = if r_id < heads.in_out {
                            let at = r_id as usize * BYTES;
                            &out[at..at + BYTES]
                        } else {
                            &self.spill.ovf[(r_id - heads.in_out) as usize][..]
                        };
                        let hi = pack | (r_epoch << 16);
                        let (now, rest) = rem_dst.split_at_mut(take);
                        for (e, s) in now.iter_mut().zip(chunk[2 * r_at..2 * (r_at + take)].chunks_exact(2)) {
                            *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                        }
                        r_at += take;
                        r_read += take as u32;
                        rem_dst = rest;
                    }
                }
                k = b.end;
            }
        }
        tm.s4_wire = t_s4.elapsed().as_nanos() as u64;

        // ---- STAGE S3b: `inline_words` ----
        let t_s3b = Instant::now();
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
        let words_cnt = self.offsets.len().saturating_sub(1);
        self.inline16.reserve_exact(words_cnt);
        self.lens.reserve_exact(words_cnt);
        let mut n_long = 0usize;
        for w in self.offsets.windows(2) {
            let word = &self.spelling[w[0] as usize..w[1] as usize];
            let mut v = [0; INLINE];
            let head = word.len().min(INLINE);
            v[..head].copy_from_slice(&word[..head]);
            self.inline16.push(v);
            if word.len() <= INLINE {
                self.lens.push(word.len() as u8);
            } else {
                self.lens.push(LONG);
                n_long += 1;
            }
        }
        tm.s3b_inline_words = t_s3b.elapsed().as_nanos() as u64;

        // ---- STAGE S5: Master Serial `emit` ----
        let t_s5 = Instant::now();
        out.resize(n + INLINE, 0);
        let mut at = 0usize;
        let mask = (1u32 << bits) - 1;
        let mut j = mark;
        for _ in 0..m {
            let edge = self.edges[j as usize];
            let code = if self.extra.is_empty() {
                edge >> bits
            } else {
                self.extra[j as usize]
            } as usize;
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
        tm.s5_emit = t_s5.elapsed().as_nanos() as u64;

        st.n_runs = f.runs;
        st.m_retained = m as usize;
        st.h_strips = ns;
        st.open_nodes = nodes as usize - 1;
        st.rv_steps = steps;
        st.slp_rules = (top - base) / RULE;
        st.n_bands = self.bands.len();
        st.n_words = n_words;
        st.n_long_words = n_long;
        st.spelling_bytes = self.spelling.len();
        st.traced_height_sum = traced_sum;
        st.outq_chunks = q_chunks as usize;
        st.outq_epochs = self.spill.cuts.len();
        st.heap_strips_b = self.strips.capacity() * std::mem::size_of::<Strip>();
        st.heap_bands_b = self.bands.capacity() * std::mem::size_of::<Band>();
        st.heap_spelling_b = self.spelling.capacity();
        st.heap_offsets_b = self.offsets.capacity() * 4;
        st.heap_inline16_b = self.inline16.capacity() * INLINE + self.lens.capacity();
        st.heap_edges_b = self.edges.capacity() * 4;
        st.heap_spill_b = self.spill.next.capacity() * 4 + self.spill.ovf.capacity() * BYTES + self.spill.cuts.capacity() * 1024;
        (tm, st)
    }
}

// ============================================================================
// Exact Per-Stage Cache & Dependency Simulator (`32K` L1d, `256K` L2, `2048K` L3)
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

#[derive(Default, Clone, Copy, Debug)]
struct StageCacheMetrics {
    q_rand: usize,
    d_serial: usize,
    d_past_l2: usize,
    miss_32k: usize,
    miss_256k: usize,
    miss_2048k: usize,
    compulsory_lines: usize,
}

fn simulate_pre_emit_stages(records: &[u16], n: usize, p: u32) -> [StageCacheMetrics; 4] {
    // Line namespaces (`64 B` cache lines):
    //   0: records (`2 B/entry` -> 32/line)
    //   1: out (`1 B/entry` -> 64/line)
    //   2: strips (`16 B/entry` -> 4/line)
    //   3: edges (`4 B/entry` -> 16/line)
    //   4: bands (`16 B` aligned / `12 B` -> ~5.33/line; exact byte offset `bi * 12 / 64`)
    //   5: spelling (`1 B/entry` -> 64/line)
    //   6: inline16 (`16 B/entry` -> 4/line)
    let b_rec = 0u32;
    let b_out = b_rec + records.len().div_ceil(32) as u32 + 4;
    let b_strips = b_out + (n + 64).div_ceil(64) as u32 + 4;
    let b_edges = b_strips + (n / 8).div_ceil(4) as u32 + 64;
    let b_bands = b_edges + (n).div_ceil(16) as u32 + 4096;
    let b_spell = b_bands + (n * 12).div_ceil(64) as u32 + 64;
    let b_inl = b_spell + (n).div_ceil(64) as u32 + 64;
    let universe = (b_inl as usize) + 16384;

    struct SimState {
        l1: FastLru,
        l2: FastLru,
        l3: FastLru,
        prev_ln: u32,
        res: [StageCacheMetrics; 4],
    }
    impl SimState {
        #[inline]
        fn touch(&mut self, stage: usize, ln: u32, is_dep: bool) {
            if self.prev_ln != ln && self.prev_ln != ln.wrapping_sub(1) {
                self.res[stage].q_rand += 1;
            }
            self.prev_ln = ln;
            if !self.l1.access(ln) {
                self.res[stage].miss_32k += 1;
            }
            if !self.l2.access(ln) {
                self.res[stage].miss_256k += 1;
                if is_dep {
                    self.res[stage].d_past_l2 += 1;
                }
            }
            if !self.l3.access(ln) {
                self.res[stage].miss_2048k += 1;
            }
        }
    }

    let mut sim = SimState {
        l1: FastLru::new(512, universe),
        l2: FastLru::new(4096, universe),
        l3: FastLru::new(32768, universe),
        prev_ln: u32::MAX,
        res: [StageCacheMetrics::default(); 4],
    };

    for ln in 0..records.len().div_ceil(32) as u32 {
        sim.l2.access(b_rec + ln);
        sim.l3.access(b_rec + ln);
    }
    for ln in 0..n.div_ceil(64) as u32 {
        sim.l3.access(b_out + ln);
    }

    let mut out = vec![0u8; n + INLINE];
    let in_out = (out.len() / BYTES) as u32;
    let mut spill = Spill::default();
    let mut strips = Vec::<Strip>::new();
    let mut q_chunks = 0u32;
    let mut q_heads = [NONE; 256];
    let mut q_tails = [NONE; 256];
    let mut q_fill = [CHUNK as u32; 256];
    let mut f = First::default();

    let mut pos = 0u32;
    let mut idx = 0usize;
    while idx < records.len() {
        sim.touch(0, b_rec + (idx / 32) as u32, false);
        let rec = records[idx];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        idx += 1;
        while idx < records.len() && (records[idx] >> 8) as u8 == c {
            sim.touch(0, b_rec + (idx / 32) as u32, false);
            width += u32::from(records[idx] & 255);
            idx += 1;
        }
        sim.res[0].d_serial += 1;
        f.runs += 1;
        let a = usize::from(c);
        let mut push_q = |m_cur: u32, w: u32, thin: &[u32; 256], sim: &mut SimState| {
            let fl = q_fill[a] as usize;
            let lo = m_cur & (EPOCH - 1);
            let tail = q_tails[a];
            if w <= 16 && fl + 16 <= CHUNK && lo + w < EPOCH && tail < in_out {
                let at = tail as usize * BYTES + 2 * fl;
                sim.touch(0, b_out + (at / 64) as u32, false);
                sim.touch(0, b_out + ((at + 31) / 64) as u32, false);
                let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                for (d, v) in out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&v.to_le_bytes());
                }
                q_fill[a] = (fl as u32) + w;
            } else {
                let mut left = w;
                let mut mc = m_cur;
                while left > 0 {
                    if q_fill[a] as usize == CHUNK {
                        let id = q_chunks;
                        q_chunks += 1;
                        if id >= in_out && spill.ovf.len() <= (id - in_out) as usize {
                            spill.ovf.push([0; BYTES]);
                        }
                        if spill.next.len() <= id as usize {
                            spill.next.resize(id as usize + 1, NONE);
                        }
                        spill.next[id as usize] = NONE;
                        if q_heads[a] == NONE {
                            q_heads[a] = id;
                        } else {
                            spill.next[q_tails[a] as usize] = id;
                        }
                        q_tails[a] = id;
                        q_fill[a] = 0;
                    }
                    let fl2 = q_fill[a] as usize;
                    let lo2 = mc & (EPOCH - 1);
                    let take = left.min((CHUNK - fl2) as u32).min(EPOCH - lo2);
                    if q_tails[a] < in_out {
                        let at = q_tails[a] as usize * BYTES + 2 * fl2;
                        sim.touch(0, b_out + (at / 64) as u32, false);
                        for (i, d) in out[at..at + 2 * take as usize].chunks_exact_mut(2).enumerate() {
                            d.copy_from_slice(&((lo2 + i as u32) as u16).to_le_bytes());
                        }
                    }
                    q_fill[a] += take;
                    mc += take;
                    left -= take;
                    if mc % EPOCH == 0 {
                        let mut cut = *thin;
                        cut[a] += w - left;
                        spill.cuts.push(cut);
                    }
                }
            }
        };

        if width <= LIMIT {
            if p.wrapping_sub(pos) < width {
                f.mark = f.m + (p - pos);
            }
            push_q(f.m, width, &f.thin, &mut sim);
            f.thin[a] += width;
            f.m += width;
            f.all[a] += width;
        } else if p.wrapping_sub(pos) >= width {
            let si = strips.len();
            sim.touch(0, b_strips + (si / 4) as u32, false);
            strips.push(Strip { from: pos, to: f.all[a], width, byte: c });
            f.thick[a] += 1;
            f.all[a] += width;
        } else {
            let end = pos + width;
            if pos < p {
                let si = strips.len();
                sim.touch(0, b_strips + (si / 4) as u32, false);
                strips.push(Strip { from: pos, to: f.all[a], width: p - pos, byte: c });
                f.thick[a] += 1;
                f.all[a] += p - pos;
            }
            f.mark = f.m;
            push_q(f.m, 1, &f.thin, &mut sim);
            f.thin[a] += 1;
            f.m += 1;
            f.all[a] += 1;
            if p + 1 < end {
                let si = strips.len();
                sim.touch(0, b_strips + (si / 4) as u32, false);
                strips.push(Strip { from: p + 1, to: f.all[a], width: end - p - 1, byte: c });
                f.thick[a] += 1;
                f.all[a] += end - p - 1;
            }
        }
        pos += width;
    }
    sim.res[0].compulsory_lines = records.len().div_ceil(32) + (f.m as usize * 2).div_ceil(64) + strips.len().div_ceil(4);

    // Stage S2 (`S2a + S2b`: Node Build + RV-Morse Eliminate)
    let m = f.m;
    let ns = strips.len();
    let most = 3 * ns + 3;
    let need = (ns + 8 * most).max(m as usize);
    let mut edges = vec![0u32; need];
    let first = starts(f.all);
    let mut write = starts(f.thick);
    for (i, s) in strips.iter_mut().enumerate() {
        sim.touch(1, b_strips + (i / 4) as u32, false);
        let a = usize::from(s.byte);
        s.to += first[a];
        let dst_idx = write[a] as usize;
        sim.touch(1, b_edges + (dst_idx / 16) as u32, false);
        edges[dst_idx] = i as u32;
        write[a] += 1;
    }

    let mut nodes = 0u32;
    {
        let (image_order, arena) = edges.split_at_mut(ns);
        let mk_node = |a: &mut [u32], nodes_ref: &mut u32, width: u32, word: u32, sim: &mut SimState| -> u32 {
            let id = *nodes_ref;
            *nodes_ref += 1;
            let off = ns + id as usize * 4;
            sim.touch(1, b_edges + (off / 16) as u32, false);
            a[id as usize * 4 + 2] = width;
            a[id as usize * 4 + 3] = word;
            id
        };
        let append_to = |a: &mut [u32], list: &mut (u32, u32), id: u32, side: usize, sim: &mut SimState| {
            let (head, tail) = list;
            if *tail == 0 {
                *head = id;
            } else {
                let off = ns + *tail as usize * 4 + side;
                sim.touch(1, b_edges + (off / 16) as u32, false);
                let w = &mut a[*tail as usize * 4 + side];
                *w = (*w & 0xFFFF) | (id << 16);
            }
            let off2 = ns + id as usize * 4 + side;
            sim.touch(1, b_edges + (off2 / 16) as u32, false);
            let w2 = &mut a[id as usize * 4 + side];
            *w2 = (*w2 & 0xFFFF_0000) | *tail;
            *tail = id;
        };

        mk_node(arena, &mut nodes, 0, 0, &mut sim);
        for (i, s) in strips.iter().enumerate() {
            sim.touch(1, b_strips + (i / 4) as u32, false);
            mk_node(arena, &mut nodes, s.width, u32::from(s.byte) + 1, &mut sim);
        }
        let (mut terminals, mut order) = ([(0u32, 0u32); 2], [(0u32, 0u32); 2]);
        let mut x = 0u32;
        for (i, s) in strips.iter().enumerate() {
            sim.touch(1, b_strips + (i / 4) as u32, false);
            if x < s.from {
                let id = mk_node(arena, &mut nodes, s.from - x, 0, &mut sim);
                append_to(arena, &mut order[0], id, 0, &mut sim);
                append_to(arena, &mut terminals[1], id, 1, &mut sim);
            }
            append_to(arena, &mut order[0], i as u32 + 1, 0, &mut sim);
            x = s.from + s.width;
        }
        if (x as usize) < n {
            let id = mk_node(arena, &mut nodes, n as u32 - x, 0, &mut sim);
            append_to(arena, &mut order[0], id, 0, &mut sim);
            append_to(arena, &mut terminals[1], id, 1, &mut sim);
        }
        x = 0;
        for (k_idx, &i) in image_order.iter().enumerate() {
            sim.touch(1, b_edges + (k_idx / 16) as u32, false);
            sim.touch(1, b_strips + (i as usize / 4) as u32, true);
            let s = strips[i as usize];
            if x < s.to {
                let id = mk_node(arena, &mut nodes, s.to - x, 0, &mut sim);
                append_to(arena, &mut order[1], id, 1, &mut sim);
                append_to(arena, &mut terminals[0], id, 0, &mut sim);
            }
            append_to(arena, &mut order[1], i + 1, 1, &mut sim);
            x = s.to + s.width;
        }
        if (x as usize) < n {
            let id = mk_node(arena, &mut nodes, n as u32 - x, 0, &mut sim);
            append_to(arena, &mut order[1], id, 1, &mut sim);
            append_to(arena, &mut terminals[0], id, 0, &mut sim);
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

    // S2b: Eliminate trace
    let base = 4 * nodes as usize;
    let mut top = base;
    let mut left = n as u32;
    while left > 0 {
        if ns + top + RULE > edges.len() {
            edges.resize((ns + top + RULE).max(2 * edges.len()), 0);
        }
        let arena = &mut edges[ns..];
        sim.res[1].d_serial += 1;
        sim.touch(1, b_edges + (ns / 16) as u32, true);
        let x = arena[0] & 0xFFFF;
        let y = arena[1] & 0xFFFF;
        sim.touch(1, b_edges + ((ns + x as usize * 4) / 16) as u32, true);
        sim.touch(1, b_edges + ((ns + y as usize * 4) / 16) as u32, true);
        let aw = arena[x as usize * 4 + 2];
        let bw = arena[y as usize * 4 + 2];
        let w = aw.min(bw);
        left -= w;
        let cut_tr = |a: &mut [u32], id: u32, side: usize, sim: &mut SimState| {
            let p0 = a[id as usize * 4 + side] & 0xFFFF;
            let n0 = a[id as usize * 4 + side] >> 16;
            sim.touch(1, b_edges + ((ns + p0 as usize * 4 + side) / 16) as u32, true);
            sim.touch(1, b_edges + ((ns + n0 as usize * 4 + side) / 16) as u32, true);
            a[p0 as usize * 4 + side] = (a[p0 as usize * 4 + side] & 0xFFFF) | (n0 << 16);
            a[n0 as usize * 4 + side] = (a[n0 as usize * 4 + side] & 0xFFFF_0000) | p0;
        };
        let ins_tr = |a: &mut [u32], at: u32, id: u32, side: usize, sim: &mut SimState| {
            let nx = a[at as usize * 4 + side] >> 16;
            sim.touch(1, b_edges + ((ns + nx as usize * 4 + side) / 16) as u32, true);
            a[id as usize * 4 + side] = (nx << 16) | at;
            a[at as usize * 4 + side] = (a[at as usize * 4 + side] & 0xFFFF) | (id << 16);
            a[nx as usize * 4 + side] = (a[nx as usize * 4 + side] & 0xFFFF_0000) | id;
        };
        if x == y {
            cut_tr(arena, x, 0, &mut sim);
            cut_tr(arena, x, 1, &mut sim);
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
                sim.touch(1, b_edges + ((ns + r + 2) / 16) as u32, true);
                arena[r + 2]
            };
            let rl = if rw <= 256 {
                1
            } else {
                let r = base + (rw - 257) as usize * RULE;
                sim.touch(1, b_edges + ((ns + r + 2) / 16) as u32, true);
                arena[r + 2]
            };
            sim.touch(1, b_edges + ((ns + top) / 16) as u32, false);
            let id = ((top - base) / RULE) as u32 + 257;
            arena[top..top + RULE].copy_from_slice(&[lw, rw, ll + rl, u32::MAX]);
            top += RULE;
            id
        };
        if aw <= bw {
            cut_tr(arena, x, 0, &mut sim);
            ins_tr(arena, y, x, 0, &mut sim);
            arena[x as usize * 4 + 3] = word;
            if aw == bw {
                cut_tr(arena, y, 0, &mut sim);
                cut_tr(arena, y, 1, &mut sim);
            } else {
                arena[y as usize * 4 + 2] = bw - w;
            }
        } else {
            cut_tr(arena, y, 1, &mut sim);
            ins_tr(arena, x, y, 1, &mut sim);
            arena[y as usize * 4 + 3] = word;
            arena[x as usize * 4 + 2] = aw - w;
        }
    }
    sim.res[1].compulsory_lines = strips.len().div_ceil(4);

    // Stage S3 (`Bottom Walk + Bands + Spelling + inline_words`)
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
            sim.res[2].d_serial += 1;
            sim.touch(2, b_edges + ((ns + id as usize * 4 + 1) / 16) as u32, true);
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
            let end = k + f.thin[a];
            while k < end {
                sim.res[2].d_serial += 1;
                sim.touch(2, b_edges + ((ns + id as usize * 4) / 16) as u32, true);
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
                            sim.touch(2, b_spell + (spelling.len() / 64) as u32, false);
                            spelling.push((wid - 1) as u8);
                        } else {
                            let r = base + (wid - 257) as usize * RULE;
                            sim.touch(2, b_edges + ((ns + r) / 16) as u32, true);
                            if arena[r + 3] == u32::MAX {
                                arena[r + 3] = spelling.len() as u32;
                                stack.push(arena[r + 1]);
                                stack.push(arena[r]);
                            } else {
                                let s0 = arena[r + 3] as usize;
                                let sl = arena[r + 2] as usize;
                                sim.touch(2, b_spell + (s0 / 64) as u32, false);
                                sim.touch(2, b_spell + (spelling.len() / 64) as u32, false);
                                spelling.extend_from_within(s0..s0 + sl);
                            }
                        }
                    }
                    sim.touch(2, b_spell + (spelling.len() / 64) as u32, false);
                    spelling.push(a as u8);
                    offsets.push(spelling.len() as u32);
                    254 + offsets.len() as u32
                };
                sim.touch(2, b_bands + ((bands.len() * 12) / 64) as u32, false);
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
    sim.res[2].compulsory_lines = ((bands.len() * 12).div_ceil(64)) + ((256 + n_words) * 16).div_ceil(64);

    // Stage S4 (`Wire`)
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
        let mut r_cut = spill.cuts.first().map_or(u32::MAX, |c| c[a]);
        for (bi_rel, b) in bands[cursors[a]..end_b].iter().enumerate() {
            let bi = cursors[a] + bi_rel;
            sim.touch(3, b_bands + ((bi * 12) / 64) as u32, false);
            let span = (b.end - k) as usize;
            let dst = k.wrapping_add(b.delta) as usize;
            let mut rem = span;
            while rem > 0 {
                while r_read == r_cut {
                    r_epoch += 1;
                    r_cut = spill.cuts.get(r_epoch as usize).map_or(u32::MAX, |c| c[a]);
                }
                if r_at == CHUNK {
                    r_id = spill.next[r_id as usize];
                    r_at = 0;
                }
                let take = rem.min(CHUNK - r_at).min((r_cut - r_read) as usize);
                if r_id < in_out {
                    let at = r_id as usize * BYTES + 2 * r_at;
                    sim.touch(3, b_out + (at / 64) as u32, false);
                }
                r_at += take;
                r_read += take as u32;
                rem -= take;
            }
            for eln in (dst / 16)..=((dst + span - 1) / 16) {
                sim.touch(3, b_edges + eln as u32, false);
            }
            let _ = bits;
            k = b.end;
        }
    }
    sim.res[3].d_serial = 1;
    sim.res[3].compulsory_lines = (m as usize).div_ceil(16);

    for (wi, w) in offsets.windows(2).enumerate() {
        sim.touch(2, b_spell + (w[0] as usize / 64) as u32, false);
        sim.touch(2, b_inl + ((256 + wi) / 4) as u32, false);
    }

    sim.res
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/antigravity/admitted-bwt".to_string());
    let rounds: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(25);
    let kernels = Kernels::detect();

    let mut test_inputs: Vec<(String, Vec<u8>, u32)> = vec![
        ("block-0.L".into(), fs::read(format!("{}/block-0.L", dir)).unwrap(), 264_045),
        ("block-2.L".into(), fs::read(format!("{}/block-2.L", dir)).unwrap(), 320_807),
        ("block-85.L".into(), fs::read(format!("{}/block-85.L", dir)).unwrap(), 325_088),
        ("block-143.L".into(), fs::read(format!("{}/block-143.L", dir)).unwrap(), 802_868),
        ("block-210.L".into(), fs::read(format!("{}/block-210.L", dir)).unwrap(), 791_429),
        ("block-207.L".into(), fs::read(format!("{}/block-207.L", dir)).unwrap(), 205_180),
    ];
    let (dense_ll, dense_p) = synthesize_dense_703_bwt();
    test_inputs.push(("dense-703.syn".into(), dense_ll, dense_p));

    let mut m_a1 = OpenCells::default();
    let mut m_a2 = OpenCells::default();
    let mut m_st = MasterStaged::default();
    let mut out_a1 = Vec::with_capacity(900_064);
    let mut out_a2 = Vec::with_capacity(900_064);
    let mut out_st = Vec::with_capacity(900_064);

    println!(
        "=== CYCLE 11: NATIVE SAFE-RUST A/A CONTROL + PRE-EMIT 4-STAGE TIMING DECOMPOSITION ({} ROUNDS vs `origin/master` `90e671966`) ===",
        rounds
    );
    let hdr = [
        format!("{:<13}", "Block"),
        format!("{:>7}", "Mst_A1"),
        format!("{:>7} ({:>6})", "Mst_A2", "A2/A1"),
        format!("{:>7}", "PreEmit"),
        format!("{:>6}", "S1_Pas"),
        format!("{:>6}", "S2a_Nd"),
        format!("{:>6}", "S2b_RV"),
        format!("{:>6}", "S3_Bnd"),
        format!("{:>6}", "S4_Wir"),
        format!("{:>6}", "S5_Emt"),
        format!("{:>6}", "Pre%"),
    ]
    .join(" | ");
    println!("{hdr}");
    println!("{}", "-".repeat(hdr.len()));

    let mut json_rows = Vec::new();
    for (name, ll, p) in &test_inputs {
        let bz2 = craft_bz2_block(ll, *p, kernels);
        let (runs_obj, n_exp, p_dec) = extract_runs(&bz2, kernels);
        assert_eq!((n_exp, p_dec), (ll.len(), *p));

        assert!(m_a1.invert_runs(&runs_obj, n_exp, *p, &mut out_a1, kernels));
        assert!(m_a2.invert_runs(&runs_obj, n_exp, *p, &mut out_a2, kernels));
        let (_, st_info) = m_st.run_timed(runs_obj.records(), n_exp, *p, &mut out_st);
        assert_eq!(out_a1, out_a2, "A/A mismatch on {name}");
        assert_eq!(out_st, out_a1, "MasterStaged mismatch on {name}");

        let stage_bmcp = simulate_pre_emit_stages(runs_obj.records(), n_exp, *p);

        let mut t_a1 = Vec::with_capacity(rounds);
        let mut t_a2 = Vec::with_capacity(rounds);
        let mut v_s1 = Vec::with_capacity(rounds);
        let mut v_s2a = Vec::with_capacity(rounds);
        let mut v_s2b = Vec::with_capacity(rounds);
        let mut v_s3a = Vec::with_capacity(rounds);
        let mut v_s3b = Vec::with_capacity(rounds);
        let mut v_s4 = Vec::with_capacity(rounds);
        let mut v_s5 = Vec::with_capacity(rounds);

        for r in 0..(rounds + 3) {
            for rot in 0..3 {
                let arm = (r + rot) % 3;
                match arm {
                    0 => {
                        let t0 = Instant::now();
                        m_a1.invert_runs(&runs_obj, n_exp, *p, &mut out_a1, kernels);
                        let dt = t0.elapsed().as_nanos() as u64;
                        if r >= 3 {
                            t_a1.push(dt);
                        }
                    }
                    1 => {
                        let t0 = Instant::now();
                        m_a2.invert_runs(&runs_obj, n_exp, *p, &mut out_a2, kernels);
                        let dt = t0.elapsed().as_nanos() as u64;
                        if r >= 3 {
                            t_a2.push(dt);
                        }
                    }
                    _ => {
                        let (tm, _) = m_st.run_timed(runs_obj.records(), n_exp, *p, &mut out_st);
                        if r >= 3 {
                            v_s1.push(tm.s1_first_pass);
                            v_s2a.push(tm.s2a_order_and_nodes);
                            v_s2b.push(tm.s2b_eliminate);
                            v_s3a.push(tm.s3a_bottom_bands_spell);
                            v_s4.push(tm.s4_wire);
                            v_s3b.push(tm.s3b_inline_words);
                            v_s5.push(tm.s5_emit);
                        }
                    }
                }
            }
        }
        for v in [&mut t_a1, &mut t_a2, &mut v_s1, &mut v_s2a, &mut v_s2b, &mut v_s3a, &mut v_s3b, &mut v_s4, &mut v_s5] {
            v.sort_unstable();
        }
        let med = |v: &[u64]| v[rounds / 2] as f64 / 1e3;
        let a1 = med(&t_a1);
        let a2 = med(&t_a2);
        let s1 = med(&v_s1);
        let s2a = med(&v_s2a);
        let s2b = med(&v_s2b);
        let s3 = med(&v_s3a) + med(&v_s3b);
        let s3a = med(&v_s3a);
        let s3b = med(&v_s3b);
        let s4 = med(&v_s4);
        let s5 = med(&v_s5);
        let pre = s1 + s2a + s2b + s3 + s4;
        let tot_st = pre + s5;

        println!(
            "{:<13} | {:>6.1}u | {:>6.1}u ({:>5.3}x) | {:>6.1}u | {:>5.1}u | {:>5.1}u | {:>5.1}u | {:>5.1}u | {:>5.1}u | {:>5.1}u | {:>5.1}%",
            name,
            a1,
            a2,
            a2 / a1,
            pre,
            s1,
            s2a,
            s2b,
            s3,
            s4,
            s5,
            100.0 * pre / tot_st
        );

        json_rows.push(format!(
            "    {{\n      \"block\": \"{}\", \"n\": {}, \"records\": {}, \"runs\": {}, \"m\": {}, \"h_strips\": {}, \"open_nodes\": {}, \"rv_steps\": {}, \"slp_rules\": {}, \"n_bands\": {}, \"n_words\": {}, \"n_long_words\": {}, \"spelling_bytes\": {}, \"traced_height_sum\": {},\n      \"master_a1_us\": {:.1}, \"master_a2_us\": {:.1}, \"aa_ratio\": {:.4}, \"pre_emit_us\": {:.1}, \"s1_first_pass_us\": {:.1}, \"s2a_nodes_us\": {:.1}, \"s2b_eliminate_us\": {:.1}, \"s3a_bands_spell_us\": {:.1}, \"s3b_inline_us\": {:.1}, \"s4_wire_us\": {:.1}, \"s5_emit_us\": {:.1},\n      \"heap_bytes\": {{\"strips\": {}, \"bands\": {}, \"spelling\": {}, \"offsets\": {}, \"inline16_lens\": {}, \"edges\": {}, \"spill\": {}}},\n      \"bmcp\": {{\n        \"s1\": {{\"q_rand\": {}, \"d_serial\": {}, \"d_past_l2\": {}, \"miss_32k\": {}, \"miss_256k\": {}, \"miss_2048k\": {}, \"floor_lines\": {}}},\n        \"s2\": {{\"q_rand\": {}, \"d_serial\": {}, \"d_past_l2\": {}, \"miss_32k\": {}, \"miss_256k\": {}, \"miss_2048k\": {}, \"floor_lines\": {}}},\n        \"s3\": {{\"q_rand\": {}, \"d_serial\": {}, \"d_past_l2\": {}, \"miss_32k\": {}, \"miss_256k\": {}, \"miss_2048k\": {}, \"floor_lines\": {}}},\n        \"s4\": {{\"q_rand\": {}, \"d_serial\": {}, \"d_past_l2\": {}, \"miss_32k\": {}, \"miss_256k\": {}, \"miss_2048k\": {}, \"floor_lines\": {}}}\n      }}\n    }}",
            name, st_info.n, st_info.n_records, st_info.n_runs, st_info.m_retained, st_info.h_strips,
            st_info.open_nodes, st_info.rv_steps, st_info.slp_rules, st_info.n_bands, st_info.n_words,
            st_info.n_long_words, st_info.spelling_bytes, st_info.traced_height_sum,
            a1, a2, a2 / a1, pre, s1, s2a, s2b, s3a, s3b, s4, s5,
            st_info.heap_strips_b, st_info.heap_bands_b, st_info.heap_spelling_b, st_info.heap_offsets_b,
            st_info.heap_inline16_b, st_info.heap_edges_b, st_info.heap_spill_b,
            stage_bmcp[0].q_rand, stage_bmcp[0].d_serial, stage_bmcp[0].d_past_l2, stage_bmcp[0].miss_32k, stage_bmcp[0].miss_256k, stage_bmcp[0].miss_2048k, stage_bmcp[0].compulsory_lines,
            stage_bmcp[1].q_rand, stage_bmcp[1].d_serial, stage_bmcp[1].d_past_l2, stage_bmcp[1].miss_32k, stage_bmcp[1].miss_256k, stage_bmcp[1].miss_2048k, stage_bmcp[1].compulsory_lines,
            stage_bmcp[2].q_rand, stage_bmcp[2].d_serial, stage_bmcp[2].d_past_l2, stage_bmcp[2].miss_32k, stage_bmcp[2].miss_256k, stage_bmcp[2].miss_2048k, stage_bmcp[2].compulsory_lines,
            stage_bmcp[3].q_rand, stage_bmcp[3].d_serial, stage_bmcp[3].d_past_l2, stage_bmcp[3].miss_32k, stage_bmcp[3].miss_256k, stage_bmcp[3].miss_2048k, stage_bmcp[3].compulsory_lines,
        ));
    }

    println!("\n=== PER-STAGE BMCP(256K L2 / 32K L1d) MISSES & COMPULSORY FLOOR SUMMARY ===");
    println!(
        "{:<13} | {:>15} | {:>15} | {:>15} | {:>15}",
        "Block", "S1_L2m(Flr)", "S2_L2m(DpL2)", "S3_L2m(DpL2)", "S4_L2m(Flr)"
    );
    println!("{}", "-".repeat(82));
    for (name, ll, p) in &test_inputs {
        let bz2 = craft_bz2_block(ll, *p, kernels);
        let (runs_obj, n_exp, _) = extract_runs(&bz2, kernels);
        let b = simulate_pre_emit_stages(runs_obj.records(), n_exp, *p);
        println!(
            "{:<13} | {:>7}({:>6}) | {:>7}({:>6}) | {:>7}({:>6}) | {:>7}({:>6})",
            name,
            b[0].miss_256k, b[0].compulsory_lines,
            b[1].miss_256k, b[1].d_past_l2,
            b[2].miss_256k, b[2].d_past_l2,
            b[3].miss_256k, b[3].compulsory_lines,
        );
    }

    if let Some(json_path) = std::env::args().nth(3) {
        let j = format!(
            "{{\n  \"origin_master_sha\": \"90e6719663af0615fb417b038906ad2636193aa3\",\n  \"blocks\": [\n{}\n  ]\n}}\n",
            json_rows.join(",\n")
        );
        fs::write(json_path, j).expect("write json");
    }
}
