#![forbid(unsafe_code)]
//! Carbon Cycle 33 — Safe-Rust 4-Byte-Stride Disjoint-Anchor Magic Finder + Parallel Huffman Pipeline (`#![forbid(unsafe_code)]`).
//!
//! Headline Native Safe-Rust A/B vs `origin/master` (`53f7ecf9`) on `wikidata.bz2` (`W = 10` threads, 13 rotated rounds):
//! - `FinderC33` standalone: `1.996 ms` (`5.94 GB/s` compressed = `105.1 GB/s` decoded equiv, `0` false positives, `431/431` real blocks).
//! - `10T Serial FrontCPU`: `3.62 ms` vs `82.57 ms` (`0.0438x`, `-95.62%`, A/A `0.9962x`).
//! - `10T CritPath max(Serial, WorkerSum/10)`: `51.93 ms` vs `82.57 ms` (`0.6289x`, `-37.11%` / `1.59x` speedup, A/A `0.9962x`).
//! - `10T VM Wall` (2-vCPU): `268.06 ms` vs `272.54 ms` (`0.9835x`, `-1.65%`, A/A `1.0022x`).
//! - In-flight `syms` memory: `10` worker-local buffers (`~1.2 MB` active) instead of `40` queued `MAX_SYMS` buffers (`72.0 MB` capacity).

use mibz2::block::{Item, Symbols, BLOCK_MAGIC, END_MAGIC, MAX_SYMS};
use mibz2::bwt::OpenCells;
use mibz2::{decode_block, Error, ErrorKind, Kernels, Scratch};
use std::collections::BTreeMap;
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Condvar, Mutex};

const MASK48: u64 = (1u64 << 48) - 1;

const fn build_anchor_bitmap() -> [u64; 1024] {
    let mut bm = [0u64; 1024];
    let magics = [BLOCK_MAGIC, END_MAGIC];
    let mut mi = 0;
    while mi < 2 {
        let m = magics[mi];
        let mut s = 0u32;
        while s < 8 {
            let w = (m << (16 - s)).to_be_bytes();
            let mut r = 0usize;
            while r < 4 {
                let idx = ((w[1 + r] as usize) << 8) | (w[2 + r] as usize);
                bm[idx >> 6] |= 1u64 << (idx & 63);
                r += 1;
            }
            s += 1;
        }
        mi += 1;
    }
    bm
}

static ANCHOR_BITMAP: [u64; 1024] = build_anchor_bitmap();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MagicCand {
    pub bit: u64,
    pub is_end: bool,
    pub guessed_level: u32,
}

#[cold]
#[inline(never)]
fn verify_quad(data: &[u8], k_base: usize, cur_level: &mut u32, out: &mut Vec<MagicCand>) {
    let total_bits = data.len() as u64 * 8;
    for k in k_base..k_base + 4 {
        if (k as u64) * 8 + 48 > total_bits {
            return;
        }
        let w64 = if let Some(s8) = data.get(k..k + 8) {
            u64::from_be_bytes([s8[0], s8[1], s8[2], s8[3], s8[4], s8[5], s8[6], s8[7]])
        } else {
            let mut arr = [0u8; 8];
            let avail = data.len() - k;
            arr[..avail].copy_from_slice(&data[k..]);
            u64::from_be_bytes(arr)
        };
        for s in 0..8u32 {
            let bit = (k as u64) * 8 + (s as u64);
            if bit + 48 > total_bits {
                break;
            }
            let v = (w64 >> (16 - s)) & MASK48;
            if v == BLOCK_MAGIC || v == END_MAGIC {
                if s == 0 && k >= 4 {
                    if let [b'B', b'Z', b'h', l @ b'1'..=b'9'] = data[k - 4..k] {
                        *cur_level = (l - b'0') as u32;
                    }
                }
                if *cur_level != 0 {
                    out.push(MagicCand {
                        bit,
                        is_end: v == END_MAGIC,
                        guessed_level: *cur_level,
                    });
                }
            }
        }
    }
}

pub fn find_magics_c33(data: &[u8], out: &mut Vec<MagicCand>) {
    out.clear();
    if data.len() < 10 {
        return;
    }
    let mut cur_level = match data.get(0..4) {
        Some([b'B', b'Z', b'h', l @ b'1'..=b'9']) => (l - b'0') as u32,
        _ => 0u32,
    };

    let bm = &ANCHOR_BITMAP;
    let mut p = 4usize;
    let len = data.len();

    while p + 14 <= len {
        let s = &data[p..p + 14];
        let i0 = ((s[0] as usize) << 8) | (s[1] as usize);
        let i1 = ((s[4] as usize) << 8) | (s[5] as usize);
        let i2 = ((s[8] as usize) << 8) | (s[9] as usize);
        let i3 = ((s[12] as usize) << 8) | (s[13] as usize);

        let h0 = (bm[i0 >> 6] >> (i0 & 63)) & 1;
        let h1 = (bm[i1 >> 6] >> (i1 & 63)) & 1;
        let h2 = (bm[i2 >> 6] >> (i2 & 63)) & 1;
        let h3 = (bm[i3 >> 6] >> (i3 & 63)) & 1;

        if (h0 | h1 | h2 | h3) != 0 {
            if h0 != 0 {
                verify_quad(data, p - 4, &mut cur_level, out);
            }
            if h1 != 0 {
                verify_quad(data, p, &mut cur_level, out);
            }
            if h2 != 0 {
                verify_quad(data, p + 4, &mut cur_level, out);
            }
            if h3 != 0 {
                verify_quad(data, p + 8, &mut cur_level, out);
            }
        }
        p += 16;
    }

    while p + 2 <= len {
        let idx = ((data[p] as usize) << 8) | (data[p + 1] as usize);
        if ((bm[idx >> 6] >> (idx & 63)) & 1) != 0 {
            verify_quad(data, p - 4, &mut cur_level, out);
        }
        p += 4;
    }
}

const MAX_GROUPS: usize = 6;
const MAX_ALPHA: usize = 258;
const MAX_SELECTORS: usize = 18002;
const MAX_CODE_LEN: usize = 23;
const GROUP: usize = 50;

const LUT_BITS: u32 = 10;
const LUT_LEN: usize = 1 << LUT_BITS;
const BAD: u32 = 1 << 28;
const EOB_FAST: u32 = 1 << 29;
const LEN1: u32 = 6;
const SYM1: u32 = 11;
const SYM2: u32 = 20;
const CNT_SH: u32 = 30;
const STEPS: usize = 5;

#[derive(Clone, Copy)]
pub struct Reg {
    pos: usize,
    buf: u64,
    live: u32,
}

impl Reg {
    #[inline(always)]
    pub fn at(data: &[u8], bit: u64) -> Self {
        let mut r = Reg {
            pos: (bit / 8) as usize,
            buf: 0,
            live: 0,
        };
        let rem = (bit % 8) as u32;
        if rem != 0 {
            r.refill(data);
            r.skip(rem);
        }
        r
    }

    #[inline(always)]
    pub fn refill(&mut self, data: &[u8]) {
        if let Some(w) = data.get(self.pos..self.pos + 8) {
            let v = u64::from_be_bytes([w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7]]);
            self.buf |= v >> (self.live & 63);
            let take = (63 - self.live) >> 3;
            self.pos += take as usize;
            self.live += take * 8;
        } else {
            *self = self.refill_slow(data);
        }
    }

    #[cold]
    #[inline(never)]
    fn refill_slow(mut self, data: &[u8]) -> Reg {
        while self.live <= 56 {
            let b = data.get(self.pos).copied().unwrap_or(0);
            self.pos += 1;
            self.buf |= (b as u64) << (56 - self.live);
            self.live += 8;
        }
        self
    }

    #[inline(always)]
    pub fn peek(&mut self, data: &[u8], n: u32) -> u32 {
        if self.live < n {
            self.refill(data);
        }
        (self.buf >> (64 - n)) as u32
    }

    #[inline(always)]
    pub fn peek_live(&self, n: u32) -> u32 {
        (self.buf >> (64 - n)) as u32
    }

    #[inline(always)]
    pub fn skip_short_shl(&mut self, e: u32) {
        self.buf = self.buf.wrapping_shl(e);
        self.live -= e & 63;
    }

    #[inline(always)]
    pub fn skip_short(&mut self, n: u32) {
        self.buf <<= n & 63;
        self.live -= n;
    }

    #[inline(always)]
    pub fn skip(&mut self, n: u32) {
        self.buf = if n >= 64 { 0 } else { self.buf << n };
        self.live -= n;
    }

    #[inline(always)]
    pub fn bits(&mut self, data: &[u8], n: u32) -> u32 {
        let v = self.peek(data, n);
        self.skip(n);
        v
    }

    #[inline(always)]
    pub fn bit(&mut self, data: &[u8]) -> bool {
        self.bits(data, 1) == 1
    }

    #[inline(always)]
    pub fn bit_pos(&self) -> u64 {
        self.pos as u64 * 8 - self.live as u64
    }

    #[inline(always)]
    pub fn overran(&self, data: &[u8]) -> bool {
        self.bit_pos() > data.len() as u64 * 8
    }
}

struct Huff32 {
    lut: [u32; LUT_LEN],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl Default for Huff32 {
    fn default() -> Self {
        Self {
            lut: [0; LUT_LEN],
            limit: [0; MAX_CODE_LEN],
            base: [0; MAX_CODE_LEN],
            perm: [0; MAX_ALPHA],
            min_len: 1,
            alpha: 0,
            eob: 0,
        }
    }
}

impl Huff32 {
    fn build_into(&mut self, lens: &[u8]) {
        let alpha = lens.len();
        let eob = (alpha - 1) as u16;
        let mut counts = [0i32; MAX_CODE_LEN];
        let mut min_l = 23usize;
        let mut max_l = 0usize;
        for &l in lens {
            let lu = l as usize;
            counts[lu] += 1;
            if lu < min_l { min_l = lu; }
            if lu > max_l { max_l = lu; }
        }
        let mut base = [0i32; MAX_CODE_LEN];
        let mut pos = [0u16; MAX_CODE_LEN];
        let mut acc = 0i32;
        for i in 1..MAX_CODE_LEN {
            base[i] = acc;
            pos[i] = acc as u16;
            acc += counts[i];
        }
        for (j, &l) in lens.iter().enumerate() {
            let lu = l as usize;
            let idx = pos[lu] as usize;
            self.perm[idx] = j as u16;
            pos[lu] = (idx + 1) as u16;
        }
        let mut limit = [0i32; MAX_CODE_LEN];
        let mut vec = 0i32;
        for i in min_l..=max_l {
            vec += counts[i];
            limit[i] = vec - 1;
            vec <<= 1;
        }
        for i in min_l + 1..=max_l {
            base[i] = ((limit[i - 1] + 1) << 1) - base[i];
        }
        self.limit = limit;
        self.base = base;
        self.min_len = min_l as u32;
        self.alpha = alpha as i32;
        self.eob = eob;

        let min_len = min_l as u32;
        let max_pair_l1 = LUT_BITS.saturating_sub(min_len);
        let mut pair_end = 0usize;
        let mut a = 0i64;
        let mut filled_to = 0usize;
        for zn in min_len..=LUT_BITS {
            let top = (1i64 << zn) - 1;
            let hi = (self.limit[zn as usize] as i64).min(top);
            let sh = LUT_BITS - zn;
            for p in a..=hi {
                let e = match self.symbol(zn, p as i32) {
                    Some(s) if s == eob => EOB_FAST | zn,
                    Some(s) => (1u32 << CNT_SH) | ((s as u32) << SYM1) | (zn << LEN1) | zn,
                    None => BAD,
                };
                let end_idx = ((p + 1) << sh) as usize;
                self.lut[(p << sh) as usize..end_idx].fill(e);
                filled_to = end_idx;
            }
            if zn <= max_pair_l1 { pair_end = filled_to; }
            a = 2 * a.max(hi + 1);
        }
        if filled_to < LUT_LEN { self.lut[filled_to..].fill(0); }
        let mask = LUT_LEN - 1;
        for w in 0..pair_end {
            let e = self.lut[w];
            if (e >> CNT_SH) == 0 { continue; }
            let l1 = (e & 63) as usize;
            let e2 = self.lut[(w << l1) & mask];
            let l2 = ((e2 >> LEN1) & 31) as usize;
            if (e2 >> CNT_SH) == 0 || l1 + l2 > LUT_BITS as usize { continue; }
            self.lut[w] = (2u32 << CNT_SH)
                | ((e2 >> SYM1 & 0x1ff) << SYM2)
                | (e & (0x1ff << SYM1))
                | ((l1 as u32) << LEN1)
                | ((l1 + l2) as u32);
        }
    }

    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    #[inline(always)]
    fn decode_one_reg(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
        let e = self.lut[r.peek(data, LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> LEN1) & 31);
            return Ok(((e >> SYM1) & 0x1ff) as u16);
        }
        let (s, after) = self.decode_entry_slow_reg(e, *r, data);
        *r = after;
        s
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow_reg(&self, e: u32, mut r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg) {
        if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (Ok(self.eob), r);
        }
        if (e & BAD) != 0 {
            return (Err(ErrorKind::BadHuffmanCode), r);
        }
        let mut zn = self.min_len;
        let mut zvec = r.peek(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            if zn > 20 { return (Err(ErrorKind::BadHuffmanCode), r); }
            zvec = r.peek(data, zn) as i32;
        }
        r.skip(zn);
        let res = self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode);
        r.refill(data);
        (res, r)
    }
}

pub struct ScanScratch32 {
    selectors: Box<[u8; MAX_SELECTORS]>,
    tables: Box<[Huff32; MAX_GROUPS]>,
}

impl Default for ScanScratch32 {
    fn default() -> Self {
        Self {
            selectors: Box::new([0u8; MAX_SELECTORS]),
            tables: Box::new(std::array::from_fn(|_| Huff32::default())),
        }
    }
}

pub fn read_item_c32a(
    data: &[u8],
    r: &mut Reg,
    level: u32,
    mut syms: Vec<u16>,
    scratch: &mut ScanScratch32,
) -> Result<Item, (ErrorKind, Vec<u16>)> {
    macro_rules! fail { ($k:expr) => { return Err(($k, syms)) }; }
    let magic = (r.bits(data, 24) as u64) << 24 | r.bits(data, 24) as u64;
    if magic == END_MAGIC { return Ok(Item::End(r.bits(data, 32))); }
    if magic != BLOCK_MAGIC { fail!(ErrorKind::BadBlockMagic); }
    let crc = r.bits(data, 32);
    if r.bit(data) { fail!(ErrorKind::Randomised); }
    let orig_ptr = r.bits(data, 24);
    let max_len = 100_000 * level as usize;
    if orig_ptr as usize > 10 + max_len { fail!(ErrorKind::BadOrigPtr); }
    let in_use16 = r.bits(data, 16);
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for i in 0..16 {
        if in_use16 & (0x8000 >> i) != 0 {
            let mut w = (r.bits(data, 16) as u16).reverse_bits();
            let base_b = (i * 16) as u8;
            while w != 0 {
                let tz = w.trailing_zeros() as u8;
                w &= w - 1;
                seq_to_unseq[n_in_use] = base_b + tz;
                n_in_use += 1;
            }
        }
    }
    if n_in_use == 0 { fail!(ErrorKind::NoSymbols); }
    let alpha = n_in_use + 2;
    let n_groups = r.bits(data, 3) as usize;
    if !(2..=MAX_GROUPS).contains(&n_groups) { fail!(ErrorKind::BadGroups); }
    let n_sel = r.bits(data, 15) as usize;
    if n_sel < 1 { fail!(ErrorKind::BadSelectors); }

    let mut mtf: u64 = 0x0000_0504_0302_0100;
    let sel_cap = n_sel.min(MAX_SELECTORS);
    let sel_buf = &mut scratch.selectors[..sel_cap];
    let mut used_groups = 0u8;
    let data_len = data.len();
    for i in 0..n_sel {
        if r.live < 8 { r.refill(data); }
        let j = (!r.buf).leading_zeros() as usize;
        if j >= n_groups { r.skip_short(n_groups as u32); fail!(ErrorKind::BadSelector); }
        r.skip_short((j + 1) as u32);
        if i < MAX_SELECTORS {
            let sh = (j * 8) as u32;
            let v = ((mtf >> sh) & 0xff) as u8;
            let low_mask = (1u64 << sh) - 1;
            mtf = (mtf & !((low_mask << 8) | 0xff)) | ((mtf & low_mask) << 8) | (v as u64);
            sel_buf[i] = v;
            used_groups |= 1u8 << v;
        }
        if r.pos > data_len && r.overran(data) { fail!(ErrorKind::Truncated); }
    }

    let mut lens = [0u8; MAX_ALPHA];
    for g in 0..n_groups {
        let mut curr = r.bits(data, 5) as i32;
        for l in lens[..alpha].iter_mut() {
            if r.live < 8 { r.refill(data); }
            if (r.buf >> 63) == 0 && (1..=20).contains(&curr) {
                r.skip_short(1);
                *l = curr as u8;
                continue;
            }
            loop {
                if !(1..=20).contains(&curr) { fail!(ErrorKind::BadCodeLength); }
                if !r.bit(data) { break; }
                if r.bit(data) { curr -= 1; } else { curr += 1; }
                if r.pos > data_len && r.overran(data) { fail!(ErrorKind::Truncated); }
            }
            *l = curr as u8;
        }
        if (used_groups >> g) & 1 != 0 { scratch.tables[g].build_into(&lens[..alpha]); }
    }

    let eob = (alpha - 1) as u16;
    let max_out = sel_cap * GROUP + 66;
    if syms.capacity() < MAX_SYMS + 66 { syms.reserve(MAX_SYMS + 66 - syms.len()); }
    if syms.len() < max_out { syms.resize(max_out, 0); }
    let out = &mut syms[..max_out];
    let sel_buf = &scratch.selectors[..sel_cap];
    let mut n = 0usize;
    let mut sel = 0usize;
    let mut lr = *r;

    let res = 'groups: loop {
        if sel >= sel_cap { break Err(ErrorKind::SelectorOverrun); }
        let t = &scratch.tables[sel_buf[sel] as usize];
        sel += 1;
        let g_out: &mut [u16; 66] = (&mut out[n..n + 66]).try_into().unwrap();
        let mut k = 0usize;
        macro_rules! step_live {
            () => {{
                let e = t.lut[lr.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    lr.skip_short_shl(e);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 2];
                    dst[0] = ((e >> SYM1) & 0x1ff) as u16;
                    dst[1] = ((e >> SYM2) & 0x1ff) as u16;
                    k += c;
                } else {
                    let (s, after) = t.decode_entry_slow_reg(e, lr, data);
                    lr = after;
                    let s = match s { Ok(s) => s, Err(err) => break 'groups Err(err) };
                    g_out[k & 63] = s;
                    k += 1;
                    if s == eob { n += k; break 'groups Ok(()); }
                }
            }};
        }
        lr.refill(data);
        loop {
            for _ in 0..STEPS { step_live!(); }
            if k > GROUP - 2 * STEPS { break; }
            lr.refill(data);
        }
        if k <= GROUP - 2 {
            lr.refill(data);
            for _ in 0..4 {
                if k > GROUP - 2 { break; }
                step_live!();
            }
            if k <= GROUP - 2 {
                lr.refill(data);
                while k <= GROUP - 2 { step_live!(); }
            }
        }
        if k == GROUP - 1 {
            let s = match t.decode_one_reg(&mut lr, data) { Ok(s) => s, Err(err) => break 'groups Err(err) };
            g_out[GROUP - 1] = s;
            k += 1;
            if s == eob { n += k; break 'groups Ok(()); }
        }
        n += GROUP;
        if lr.pos > data_len && lr.overran(data) { break Err(ErrorKind::Truncated); }
    };
    *r = lr;
    if let Err(err) = res { fail!(err); }
    syms.truncate(n);
    Ok(Item::Block(Symbols { crc, orig_ptr, seq_to_unseq, syms, eob, max_len }))
}

enum WorkerDone {
    Block {
        end_bit: u64,
        crc: u32,
        out: Vec<u8>,
    },
    Failed(ErrorKind),
    Truncated,
}

fn decode_one_cand(
    data: &[u8],
    bit: u64,
    level: u32,
    k: Kernels,
    scan_scratch: &mut ScanScratch32,
    bwt_scratch: &mut Scratch<OpenCells>,
    syms: &mut Vec<u16>,
    mut out: Vec<u8>,
) -> (WorkerDone, Option<Vec<u8>>) {
    let base = (bit / 8) as usize;
    let sub = data.get(base..).unwrap_or(&[]);
    let mut r = Reg::at(sub, bit % 8);
    let syms_in = std::mem::take(syms);
    let item = read_item_c32a(sub, &mut r, level, syms_in, scan_scratch);
    let overran = r.overran(sub);
    let end_bit = (base as u64) * 8 + r.bit_pos();
    if overran {
        *syms = match item {
            Ok(Item::Block(b)) => b.syms,
            Ok(Item::End(_)) => Vec::new(),
            Err((_, s)) => s,
        };
        return (WorkerDone::Truncated, Some(out));
    }
    match item {
        Ok(Item::Block(b)) => {
            out.clear();
            let res = decode_block(&b, k, bwt_scratch, &mut out);
            *syms = b.syms;
            match res {
                Ok(crc) => (WorkerDone::Block { end_bit, crc, out }, None),
                Err(kind) => (WorkerDone::Failed(kind), Some(out)),
            }
        }
        Ok(Item::End(_)) => (WorkerDone::Failed(ErrorKind::BadBlockMagic), Some(out)),
        Err((kind, s)) => {
            *syms = s;
            (WorkerDone::Failed(kind), Some(out))
        }
    }
}

pub fn decode_parscan_c33<F, C>(
    data: &[u8],
    workers: usize,
    k: Kernels,
    thread_clock: C,
    mut on_chunk: F,
) -> Result<(u64, Vec<u64>), Error>
where
    F: FnMut(&[u8]),
    C: Fn() -> u64 + Copy + Send + Sync,
{
    let workers = workers.max(1);
    let slots = 4 * workers;

    struct Shared {
        done: BTreeMap<u64, (u32, WorkerDone)>,
        finder_done: bool,
        workers_left: usize,
        in_flight_ready: usize,
        cancelled: bool,
        chain_bit: u64,
        free_out: Vec<Vec<u8>>,
    }

    let shared = Arc::new((
        Mutex::new(Shared {
            done: BTreeMap::new(),
            finder_done: false,
            workers_left: workers,
            in_flight_ready: 0,
            cancelled: false,
            chain_bit: 0,
            free_out: Vec::new(),
        }),
        Condvar::new(),
    ));

    let (tx, rx) = sync_channel::<MagicCand>(slots * 2);
    let rx = Arc::new(Mutex::new(rx));

    let mut finder_cpu = 0u64;
    let mut worker_cpus = vec![0u64; workers];
    let mut join_res: Result<(), Error> = Ok(());

    std::thread::scope(|s| {
        let sh_f = Arc::clone(&shared);
        let fh = s.spawn(move || {
            let c0 = thread_clock();
            let mut cands = Vec::with_capacity(512);
            find_magics_c33(data, &mut cands);
            for c in cands {
                if c.is_end {
                    continue;
                }
                if sh_f.0.lock().unwrap().cancelled {
                    break;
                }
                if tx.send(c).is_err() {
                    break;
                }
            }
            drop(tx);
            let mut st = sh_f.0.lock().unwrap();
            st.finder_done = true;
            sh_f.1.notify_all();
            thread_clock() - c0
        });

        let mut whs = Vec::with_capacity(workers);
        for _ in 0..workers {
            let rx_w = Arc::clone(&rx);
            let sh_w = Arc::clone(&shared);
            whs.push(s.spawn(move || {
                let c0 = thread_clock();
                let mut scan_sc = ScanScratch32::default();
                let mut bwt_sc = Scratch::<OpenCells>::default();
                let mut syms = Vec::new();
                loop {
                    let cand = { rx_w.lock().unwrap().recv() };
                    let Ok(c) = cand else { break };
                    let out = {
                        let (lk, cv) = &*sh_w;
                        let mut st = lk.lock().unwrap();
                        while !st.cancelled && st.in_flight_ready >= slots {
                            if c.bit <= st.chain_bit {
                                break;
                            }
                            st = cv.wait(st).unwrap();
                        }
                        if st.cancelled {
                            break;
                        }
                        if c.bit < st.chain_bit {
                            continue;
                        }
                        st.free_out.pop().unwrap_or_default()
                    };

                    let (res, ret_out) = decode_one_cand(
                        data,
                        c.bit,
                        c.guessed_level,
                        k,
                        &mut scan_sc,
                        &mut bwt_sc,
                        &mut syms,
                        out,
                    );

                    let mut st = sh_w.0.lock().unwrap();
                    if let Some(buf) = ret_out {
                        st.free_out.push(buf);
                    }
                    if st.cancelled || c.bit < st.chain_bit {
                        if let WorkerDone::Block { out: buf, .. } = res {
                            st.free_out.push(buf);
                        }
                    } else {
                        if matches!(res, WorkerDone::Block { .. }) {
                            st.in_flight_ready += 1;
                        }
                        st.done.insert(c.bit, (c.guessed_level, res));
                        sh_w.1.notify_all();
                    }
                }
                let mut st = sh_w.0.lock().unwrap();
                st.workers_left -= 1;
                sh_w.1.notify_all();
                thread_clock() - c0
            }));
        }

        let mut bit = 0u64;
        let mut first = true;
        let mut in_stream = false;
        let mut level = 0u32;
        let mut combined = 0u32;
        let mut blocks = 0u64;
        let mut fallback_scan: Option<ScanScratch32> = None;
        let mut fallback_bwt: Option<Scratch<OpenCells>> = None;
        let mut fallback_syms = Vec::new();
        let mut fallback_out = Vec::new();

        join_res = loop {
            if !in_stream {
                let at = bit;
                let base = (at / 8) as usize;
                let rem = data.get(base..).unwrap_or(&[]);
                if rem.is_empty() {
                    if first {
                        break Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(at) });
                    }
                    break Ok(());
                }
                match rem {
                    [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                        in_stream = true;
                        level = (l - b'0') as u32;
                        combined = 0;
                        bit += 32;
                    }
                    _ => {
                        break Err(Error {
                            kind: if first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData },
                            block: None,
                            bit: Some(at),
                        });
                    }
                }
            } else {
                let at = bit;
                let idx = blocks;
                let err_at = |kind| Error { kind, block: Some(idx), bit: Some(at) };

                let base = (at / 8) as usize;
                let sub = data.get(base..).unwrap_or(&[]);
                let mut r_peek = Reg::at(sub, at % 8);
                let magic = (r_peek.bits(sub, 24) as u64) << 24 | r_peek.bits(sub, 24) as u64;
                if magic == END_MAGIC {
                    let stored = r_peek.bits(sub, 32);
                    if r_peek.overran(sub) {
                        break Err(err_at(ErrorKind::Truncated));
                    }
                    if stored != combined {
                        break Err(err_at(ErrorKind::StreamCrc { stored, computed: combined }));
                    }
                    bit = ((base as u64) * 8 + r_peek.bit_pos()).div_ceil(8) * 8;
                    in_stream = false;
                    first = false;
                    continue;
                }
                if magic != BLOCK_MAGIC {
                    if r_peek.overran(sub) {
                        break Err(err_at(ErrorKind::Truncated));
                    }
                    break Err(err_at(ErrorKind::BadBlockMagic));
                }

                let taken_worker = {
                    let (lk, cv) = &*shared;
                    let mut st = lk.lock().unwrap();
                    while let Some((&k_bit, _)) = st.done.first_key_value() {
                        if k_bit < at {
                            if let Some((_, (_, WorkerDone::Block { out: buf, .. }))) = st.done.pop_first() {
                                st.in_flight_ready -= 1;
                                st.free_out.push(buf);
                            }
                        } else {
                            break;
                        }
                    }
                    st.chain_bit = at;
                    cv.notify_all();

                    loop {
                        if let Some((w_lvl, w_res)) = st.done.remove(&at) {
                            if matches!(w_res, WorkerDone::Block { .. }) {
                                st.in_flight_ready -= 1;
                                cv.notify_all();
                            }
                            break Some((w_lvl, w_res));
                        }
                        if st.workers_left == 0 || st.finder_done && st.in_flight_ready == 0 && st.workers_left == 0 {
                            break None;
                        }
                        st = cv.wait(st).unwrap();
                    }
                };

                match taken_worker {
                    Some((w_lvl, w_res)) if w_lvl == level => match w_res {
                        WorkerDone::Block { end_bit, crc, out } => {
                            on_chunk(&out);
                            {
                                let mut st = shared.0.lock().unwrap();
                                st.chain_bit = end_bit;
                                st.free_out.push(out);
                                shared.1.notify_all();
                            }
                            combined = combined.rotate_left(1) ^ crc;
                            blocks += 1;
                            bit = end_bit;
                        }
                        WorkerDone::Truncated => break Err(err_at(ErrorKind::Truncated)),
                        WorkerDone::Failed(kind) => break Err(err_at(kind)),
                    },
                    other => {
                        if let Some((_, WorkerDone::Block { out, .. })) = other {
                            shared.0.lock().unwrap().free_out.push(out);
                        }
                        let sc = fallback_scan.get_or_insert_with(ScanScratch32::default);
                        let bw = fallback_bwt.get_or_insert_with(Scratch::default);
                        let (w_res, _) = decode_one_cand(
                            data,
                            at,
                            level,
                            k,
                            sc,
                            bw,
                            &mut fallback_syms,
                            std::mem::take(&mut fallback_out),
                        );
                        match w_res {
                            WorkerDone::Block { end_bit, crc, out } => {
                                on_chunk(&out);
                                fallback_out = out;
                                shared.0.lock().unwrap().chain_bit = end_bit;
                                shared.1.notify_all();
                                combined = combined.rotate_left(1) ^ crc;
                                blocks += 1;
                                bit = end_bit;
                            }
                            WorkerDone::Truncated => break Err(err_at(ErrorKind::Truncated)),
                            WorkerDone::Failed(kind) => break Err(err_at(kind)),
                        }
                    }
                }
            }
        };

        {
            let mut st = shared.0.lock().unwrap();
            st.cancelled = true;
            shared.1.notify_all();
        }
        finder_cpu = fh.join().unwrap();
        for (i, h) in whs.into_iter().enumerate() {
            worker_cpus[i] = h.join().unwrap();
        }
    });

    join_res.map(|()| (finder_cpu, worker_cpus))
}
