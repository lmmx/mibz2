#![forbid(unsafe_code)]
//! Carbon Cycle 31 — Multi-Symbol Huffman LUT Width (`10..12` bits) & Arity (`Pair32` vs `Tri64`) Sweep.
//!
//! Verdict: ALL >4 KiB per-group LUT variants (`Pair32_Lut11`, `Tri64_Lut10`, `Tri64_Lut11`, `Tri64_Lut12`)
//! lose to Cycle 30's `4 KiB` `u32` `LUT_BITS = 10` (`0.8717x` ScanOnly); killed immediately.
//! Cycle 30 (`ScannerC30`) remains the active scanner champion entering Cycle 32.

// Archived Cycle 31 sweep variants (`Pair32Lut11Scanner` and `Tri64Scanner<LUT_BITS, LUT_LEN, STEPS>`)
// for reproducibility.

use mibz2::block::{Item, Symbols, BLOCK_MAGIC, END_MAGIC, MAX_SYMS};
use mibz2::stream::{Event, Source};
use mibz2::{Error, ErrorKind};

const MAX_GROUPS: usize = 6;
const MAX_ALPHA: usize = 258;
const MAX_SELECTORS: usize = 18002;
const MAX_CODE_LEN: usize = 23;
const GROUP: usize = 50;
const LOOKAHEAD: usize = 2 << 20;
const MAX_ITEM: usize = 64 << 20;

#[derive(Clone, Copy)]
pub struct Reg {
    pos: usize,
    buf: u64,
    live: u32,
}

impl Reg {
    #[inline(always)]
    pub fn at(data: &[u8], bit: u64) -> Self {
        let mut r = Reg { pos: (bit / 8) as usize, buf: 0, live: 0 };
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
        if self.live < n { self.refill(data); }
        (self.buf >> (64 - n)) as u32
    }

    #[inline(always)]
    pub fn peek_live(&self, n: u32) -> u32 {
        (self.buf >> (64 - n)) as u32
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

const P32_BITS: u32 = 11;
const P32_LEN: usize = 1 << P32_BITS;
const P32_BAD: u32 = 1 << 31;
const P32_EOB: u32 = 1 << 30;
const P32_LEN1: u32 = 6;
const P32_CNT: u32 = 11;
const P32_SYM1: u32 = 13;
const P32_SYM2: u32 = 22;

struct HuffPair32_11 {
    lut: [u32; P32_LEN],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl Default for HuffPair32_11 {
    fn default() -> Self {
        Self {
            lut: [0; P32_LEN],
            limit: [0; MAX_CODE_LEN],
            base: [0; MAX_CODE_LEN],
            perm: [0; MAX_ALPHA],
            min_len: 1,
            alpha: 0,
            eob: 0,
        }
    }
}

impl HuffPair32_11 {
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
        let max_pair_l1 = P32_BITS.saturating_sub(min_len);
        let mut pair_end = 0usize;
        let mut a = 0i64;
        let mut filled_to = 0usize;
        for zn in min_len..=P32_BITS {
            let top = (1i64 << zn) - 1;
            let hi = (self.limit[zn as usize] as i64).min(top);
            let sh = P32_BITS - zn;
            for p in a..=hi {
                let i = p as i32 - self.base[zn as usize];
                let e = if (0..self.alpha).contains(&i) {
                    let s = self.perm[i as usize];
                    if s == eob {
                        P32_EOB | zn
                    } else {
                        (s as u32) << P32_SYM1 | 1 << P32_CNT | zn << P32_LEN1 | zn
                    }
                } else {
                    P32_BAD
                };
                let end_idx = ((p + 1) << sh) as usize;
                self.lut[(p << sh) as usize..end_idx].fill(e);
                filled_to = end_idx;
            }
            if zn <= max_pair_l1 { pair_end = filled_to; }
            a = 2 * a.max(hi + 1);
        }
        if filled_to < P32_LEN { self.lut[filled_to..].fill(0); }
        let mask = P32_LEN - 1;
        for w in 0..pair_end {
            let e = self.lut[w];
            if e >> P32_CNT & 3 == 0 { continue; }
            let l1 = (e & 31) as usize;
            let e2 = self.lut[(w << l1) & mask];
            let l2 = (e2 >> P32_LEN1 & 31) as usize;
            if e2 >> P32_CNT & 3 == 0 || l1 + l2 > P32_BITS as usize { continue; }
            self.lut[w] = (e2 >> P32_SYM1 & 0x1ff) << P32_SYM2
                | (e & (0x1ff << P32_SYM1))
                | 2 << P32_CNT
                | (l1 as u32) << P32_LEN1
                | ((l1 + l2) as u32);
        }
    }

    #[inline(always)]
    fn decode_one(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
        let e = self.lut[r.peek(data, P32_BITS) as usize];
        if e >> P32_CNT & 3 != 0 {
            r.skip_short(e >> P32_LEN1 & 31);
            return Ok((e >> P32_SYM1 & 0x1ff) as u16);
        }
        let (s, after) = self.decode_entry_slow(e, *r, data);
        *r = after;
        s
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow(&self, e: u32, mut r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg) {
        if e & P32_EOB != 0 {
            r.skip_short(e & 31);
            return (Ok(self.eob), r);
        }
        if e == P32_BAD {
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
        let i = zvec - self.base[zn as usize];
        let res = (0..self.alpha).contains(&i).then(|| self.perm[i as usize]).ok_or(ErrorKind::BadHuffmanCode);
        r.refill(data);
        (res, r)
    }
}

pub struct Pair32Lut11Scanner<S> {
    src: S,
    bit: u64,
    first: bool,
    done: bool,
    in_stream: bool,
    level: u32,
    combined: u32,
    blocks: u64,
    streams: u64,
    lookahead: usize,
    selectors: Box<[u8; MAX_SELECTORS]>,
    tables: Box<[HuffPair32_11; MAX_GROUPS]>,
}

impl<S: Source> Pair32Lut11Scanner<S> {
    pub fn new(src: S) -> Self {
        Self {
            src, bit: 0, first: true, done: false, in_stream: false, level: 0, combined: 0,
            blocks: 0, streams: 0, lookahead: LOOKAHEAD,
            selectors: Box::new([0u8; MAX_SELECTORS]),
            tables: Box::new(std::array::from_fn(|_| HuffPair32_11::default())),
        }
    }

    pub fn next(&mut self, mut syms: Vec<u16>) -> Result<Event, Error> {
        loop {
            if self.done { return Ok(Event::Done); }
            if !self.in_stream {
                let at = self.bit;
                let err = |kind| Error { kind, block: None, bit: Some(at) };
                let (data, eof) = self.src.view(at / 8, 4).map_err(err)?;
                if data.is_empty() && eof {
                    if self.first { return Err(err(ErrorKind::Empty)); }
                    self.done = true;
                    continue;
                }
                match data {
                    [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                        self.in_stream = true;
                        self.level = (l - b'0') as u32;
                        self.combined = 0;
                        self.bit += 32;
                    }
                    _ => return Err(err(if self.first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData })),
                }
            } else {
                let at = self.bit;
                let index = self.blocks;
                let level = self.level;
                let combined = self.combined;
                let err = |kind| Error { kind, block: Some(index), bit: Some(at) };
                let (data, eof) = self.src.view(at / 8, self.lookahead).map_err(err)?;
                let mut r = Reg::at(data, at % 8);
                let item = read_item_p32_11(data, &mut r, level, syms, &mut self.selectors, &mut self.tables);
                let used = r.bit_pos() - at % 8;
                if r.overran(data) {
                    if eof { return Err(err(ErrorKind::Truncated)); }
                    if self.lookahead >= MAX_ITEM { return Err(err(ErrorKind::BlockTooLarge)); }
                    self.lookahead = (self.lookahead * 2).max(data.len() + 1).min(MAX_ITEM);
                    syms = match item { Ok(Item::Block(b)) => b.syms, Ok(Item::End(_)) => Vec::new(), Err((_, s)) => s };
                    continue;
                }
                self.bit += used;
                self.src.release(self.bit / 8);
                match item.map_err(|(kind, _)| err(kind))? {
                    Item::End(stored) => {
                        if stored != combined { return Err(err(ErrorKind::StreamCrc { stored, computed: combined })); }
                        self.bit = self.bit.div_ceil(8) * 8;
                        self.in_stream = false;
                        self.first = false;
                        self.streams += 1;
                        syms = Vec::new();
                    }
                    Item::Block(symbols) => {
                        self.combined = combined.rotate_left(1) ^ symbols.crc;
                        self.blocks += 1;
                        return Ok(Event::Block { index, bit: at, end: self.bit, stream: self.streams, level, symbols });
                    }
                }
            }
        }
    }
}

fn read_item_p32_11(
    data: &[u8],
    r: &mut Reg,
    level: u32,
    mut syms: Vec<u16>,
    selectors: &mut [u8; MAX_SELECTORS],
    tables: &mut [HuffPair32_11; MAX_GROUPS],
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
    let sel_buf = &mut selectors[..sel_cap];
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
        if (used_groups >> g) & 1 != 0 { tables[g].build_into(&lens[..alpha]); }
    }

    let eob = (alpha - 1) as u16;
    let max_out = sel_cap * GROUP + 64;
    if syms.capacity() < MAX_SYMS + 64 { syms.reserve(MAX_SYMS + 64 - syms.len()); }
    if syms.len() < max_out { syms.resize(max_out, 0); }
    let out = &mut syms[..max_out];
    let mut n = 0usize;
    let mut sel = 0usize;
    let mut lr = *r;

    let res = 'groups: loop {
        if sel >= sel_cap { break Err(ErrorKind::SelectorOverrun); }
        let t = &tables[sel_buf[sel] as usize];
        sel += 1;
        let g_out: &mut [u16; 64] = (&mut out[n..n + 64]).try_into().unwrap();
        let mut k = 0usize;
        macro_rules! step {
            ($live:expr) => {{
                let e = t.lut[if $live { lr.peek_live(P32_BITS) } else { lr.peek(data, P32_BITS) } as usize];
                if e >> P32_CNT & 3 != 0 {
                    lr.skip_short(e & 31);
                    let c = (e >> P32_CNT & 3) as usize;
                    g_out[k & 63] = (e >> P32_SYM1 & 0x1ff) as u16;
                    g_out[(k + 1) & 63] = (e >> P32_SYM2) as u16;
                    k += c;
                } else {
                    let (s, after) = t.decode_entry_slow(e, lr, data);
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
            for _ in 0..5 { step!(true); }
            if k > GROUP - 10 { break; }
            lr.refill(data);
        }
        while k <= GROUP - 2 { step!(false); }
        if k == GROUP - 1 {
            let s = match t.decode_one(&mut lr, data) { Ok(s) => s, Err(err) => break 'groups Err(err) };
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
