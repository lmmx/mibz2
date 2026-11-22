#![forbid(unsafe_code)]
//! A block's header and its Huffman-coded symbols, read in order from the stream: the one
//! part of the decode that must be sequential, since a block's end is known only once its
//! symbols are decoded. Every check libbzip2 1.0.8 makes here (`decompress.c`) is made
//! the same way, in the same terms, so a stream libbzip2 refuses is refused here too.

use super::bits::Bits;
use super::huff::{Huff, GROUP, MAX_ALPHA};
use super::ErrorKind;
use std::cell::RefCell;

/// A block header's magic (pi) and the stream end's (sqrt(pi))
pub const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
pub const END_MAGIC: u64 = 0x1772_4538_5090;

/// libbzip2's bounds
const MAX_GROUPS: usize = 6;
const MAX_SELECTORS: usize = 18002;
/// The most symbols a block can hold (every selector used, the last ending in EOB)
pub const MAX_SYMS: usize = MAX_SELECTORS * GROUP;

/// A block's header and symbols (phase 1's output), for a worker to finish
pub struct Symbols {
    /// the CRC the block stores, of its decoded bytes
    pub crc: u32,
    pub orig_ptr: u32,
    /// the bytes in use, in order: an MTF value's byte
    pub seq_to_unseq: [u8; 256],
    /// the MTF/RLE2 symbols, the last of them `eob`
    pub syms: Vec<u16>,
    pub eob: u16,
    /// the stream's block size bound: 100,000 x its level
    pub max_len: usize,
}

// (a block's symbols move through by value, once a block: boxing them would allocate
// a block's worth for nothing)
#[allow(clippy::large_enum_variant)]
pub enum Item {
    Block(Symbols),
    /// the stream's end, and the combined CRC it stores
    End(u32),
}

/// Reusable 54 KiB front-end scratch (`18 KiB` selectors + `6 x 6 KiB` L1d-resident `Huff` tables).
pub struct ScanScratch {
    selectors: Box<[u8; MAX_SELECTORS]>,
    tables: Box<[Huff; MAX_GROUPS]>,
}

impl Default for ScanScratch {
    fn default() -> Self {
        Self {
            selectors: Box::new([0u8; MAX_SELECTORS]),
            tables: Box::new(std::array::from_fn(|_| Huff::default())),
        }
    }
}

thread_local! {
    static TLS_SCAN_SCRATCH: RefCell<ScanScratch> = RefCell::new(ScanScratch::default());
}

/// Drop-in `read_item` reusing thread-local `ScanScratch` for callers without explicit scratch.
pub fn read_item(br: &mut Bits, level: u32, syms: Vec<u16>) -> Result<Item, (ErrorKind, Vec<u16>)> {
    TLS_SCAN_SCRATCH.with(|cell| {
        if let Ok(mut sc) = cell.try_borrow_mut() {
            read_item_with_scratch(br, level, syms, &mut sc)
        } else {
            let mut sc = ScanScratch::default();
            read_item_with_scratch(br, level, syms, &mut sc)
        }
    })
}

/// The next item of a stream of level `level` (1-9) from `br`, at a block header or the
/// stream's end: a block's header and its symbols (into `syms`, reused), or the end and
/// its CRC. A block found to run past the input (`br.overran()`) is the caller's to judge.
///
/// Zero-allocation `read_item` using explicit `ScanScratch` (`Scanner<S>` and parallel workers).
#[inline]
pub fn read_item_with_scratch(
    br: &mut Bits,
    level: u32,
    syms: Vec<u16>,
    scratch: &mut ScanScratch,
) -> Result<Item, (ErrorKind, Vec<u16>)> {
    read_item_core(br, level, syms, scratch, false)
}

/// `read_item_with_scratch` without upfront `MAX_SYMS` (`1.80 MiB`) reservation: grows `syms`
/// in quantized steps up to `(sel_cap * GROUP + 66).min(MAX_SYMS)` for bounded-memory parallel scanning.
#[inline]
pub(crate) fn read_item_tight(
    br: &mut Bits,
    level: u32,
    syms: Vec<u16>,
    scratch: &mut ScanScratch,
) -> Result<Item, (ErrorKind, Vec<u16>)> {
    read_item_core(br, level, syms, scratch, true)
}

fn read_item_core(
    br: &mut Bits,
    level: u32,
    mut syms: Vec<u16>,
    scratch: &mut ScanScratch,
    tight: bool,
) -> Result<Item, (ErrorKind, Vec<u16>)> {
    macro_rules! fail {
        ($k:expr) => {
            return Err(($k, syms))
        };
    }
    let magic = (br.bits(24) as u64) << 24 | br.bits(24) as u64;
    if magic == END_MAGIC {
        return Ok(Item::End(br.bits(32)));
    }
    if magic != BLOCK_MAGIC {
        fail!(ErrorKind::BadBlockMagic);
    }
    let crc = br.bits(32);
    if br.bit() {
        // bzip2 0.9.0 and earlier randomised a block it could not sort fast; nothing has
        // written one since 0.9.5 (1999), and neither does Wikimedia
        fail!(ErrorKind::Randomised);
    }
    let orig_ptr = br.bits(24);
    let max_len = 100_000 * level as usize;
    if orig_ptr as usize > 10 + max_len {
        fail!(ErrorKind::BadOrigPtr);
    }

    let (data, mut r) = br.parts();
    let data_len = data.len();

    let in_use16 = r.bits(data, 16) as u16;
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
    if n_in_use == 0 {
        br.set(r);
        fail!(ErrorKind::NoSymbols);
    }
    let alpha = n_in_use + 2;
    let n_groups = r.bits(data, 3) as usize;
    if !(2..=MAX_GROUPS).contains(&n_groups) {
        br.set(r);
        fail!(ErrorKind::BadGroups);
    }
    let n_sel = r.bits(data, 15) as usize;
    if n_sel < 1 {
        br.set(r);
        fail!(ErrorKind::BadSelectors);
    }

    // the selectors, move-to-front coded; those past MAX_SELECTORS are read and dropped
    // (as libbzip2 1.0.8 does: some encoders round their number up)
    let mut mtf: u64 = 0x0000_0504_0302_0100;
    let sel_cap = n_sel.min(MAX_SELECTORS);
    let sel_buf = &mut scratch.selectors[..sel_cap];
    let mut used_groups = 0u8;
    // (`i` runs past `sel_buf`'s end: the selectors past MAX_SELECTORS are read, not kept)
    #[allow(clippy::needless_range_loop)]
    for i in 0..n_sel {
        if r.live < 8 {
            r.refill(data);
        }
        let j = (!r.buf).leading_zeros() as usize;
        if j >= n_groups {
            r.skip_short(n_groups as u32);
            br.set(r);
            fail!(ErrorKind::BadSelector);
        }
        r.skip_short((j + 1) as u32);
        if i < MAX_SELECTORS {
            let sh = (j * 8) as u32;
            let v = ((mtf >> sh) & 0xff) as u8;
            let low_mask = (1u64 << sh) - 1;
            mtf = (mtf & !((low_mask << 8) | 0xff)) | ((mtf & low_mask) << 8) | (v as u64);
            sel_buf[i] = v;
            used_groups |= 1u8 << v;
        }
        if r.pos > data_len && r.overran(data) {
            br.set(r);
            fail!(ErrorKind::Truncated);
        }
    }

    let mut lens = [0u8; MAX_ALPHA];
    for g in 0..n_groups {
        let mut curr = r.bits(data, 5) as i32;
        for l in lens[..alpha].iter_mut() {
            if r.live < 8 {
                r.refill(data);
            }
            if (r.buf >> 63) == 0 && (1..=20).contains(&curr) {
                r.skip_short(1);
                *l = curr as u8;
                continue;
            }
            loop {
                if !(1..=20).contains(&curr) {
                    br.set(r);
                    fail!(ErrorKind::BadCodeLength);
                }
                if !r.bit(data) {
                    break;
                }
                if r.bit(data) {
                    curr -= 1;
                } else {
                    curr += 1;
                }
                if r.pos > data_len && r.overran(data) {
                    br.set(r);
                    fail!(ErrorKind::Truncated);
                }
            }
            *l = curr as u8;
        }
        if (used_groups >> g) & 1 != 0 {
            scratch.tables[g].build_into(&lens[..alpha]);
        }
    }

    let eob = (alpha - 1) as u16;
    let max_out = (sel_cap * GROUP + 66).min(MAX_SYMS);
    if !tight {
        if syms.capacity() < MAX_SYMS {
            syms.clear();
            syms.reserve(MAX_SYMS);
        }
    } else if syms.capacity() < max_out {
        let cap_target = max_out.max(65_536).next_power_of_two().min(MAX_SYMS);
        syms.reserve_exact(cap_target.saturating_sub(syms.len()));
    }
    let sel_buf = &scratch.selectors[..sel_cap];
    let mut n = 0usize;
    let mut sel = 0usize;
    let mut tail_buf = [0u16; 66];

    let res = loop {
        if sel >= sel_cap {
            break Err(ErrorKind::SelectorOverrun);
        }
        if n + 66 > syms.len() && syms.len() < max_out {
            syms.resize((n + 8192).min(max_out), 0);
        }
        let t = &scratch.tables[sel_buf[sel] as usize];
        sel += 1;
        let use_tail = n + 66 > max_out;
        let g_res = {
            let g_out: &mut [u16; 66] = if !use_tail {
                (&mut syms[n..n + 66]).try_into().unwrap()
            } else {
                &mut tail_buf
            };
            if r.pos + 264 <= data_len {
                let base_pos = r.pos;
                let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
                t.decode_group_win(&mut r, g_in, data, base_pos, g_out)
            } else {
                t.decode_group_slice(&mut r, data, g_out)
            }
        };
        let (k, hit_eob) = match g_res {
            Ok(v) => v,
            Err(err) => break Err(err),
        };
        if use_tail {
            syms[n..n + k].copy_from_slice(&tail_buf[..k]);
        }
        n += k;
        if hit_eob {
            break Ok(());
        }
    };

    br.set(r);
    if let Err(err) = res {
        fail!(err);
    }
    syms.truncate(n);
    Ok(Item::Block(Symbols {
        crc,
        orig_ptr,
        seq_to_unseq,
        syms,
        eob,
        max_len,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::huff::{BAD, CNT_SH, EOB_FAST, LEN1, LUT_BITS, SUB_PTR, SYM1, SYM2};

    /// A small xorshift, for the test's code lengths and bits
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    /// Code lengths: a real code's (a Huffman code of random weights, at most 20 bits),
    /// or any lengths 1-20 at all (oversubscribed or incomplete, as a corrupt block's)
    fn lengths(rng: &mut Rng) -> Vec<u8> {
        let alpha = 3 + rng.below(256) as usize;
        lengths_of(rng, alpha)
    }

    /// `lengths` for an alphabet of `alpha` symbols
    fn lengths_of(rng: &mut Rng, alpha: usize) -> Vec<u8> {
        if rng.below(3) == 0 {
            let lo = 1 + rng.below(20) as u8;
            let hi = lo + rng.below(21 - lo as u64) as u8;
            return (0..alpha)
                .map(|_| lo + rng.below((hi - lo + 1) as u64) as u8)
                .collect();
        }
        // package the weights two at a time, as bzip2's encoder does, until no length
        // passes 20
        let mut w: Vec<u64> = (0..alpha)
            .map(|_| {
                let top = 1 << rng.below(16);
                1 + rng.below(top)
            })
            .collect();
        loop {
            let mut lens = vec![0u8; alpha];
            let mut nodes: Vec<(u64, Vec<usize>)> =
                w.iter().enumerate().map(|(i, &x)| (x, vec![i])).collect();
            while nodes.len() > 1 {
                nodes.sort_by_key(|b| std::cmp::Reverse(b.0));
                let (a, b) = (nodes.pop().unwrap(), nodes.pop().unwrap());
                for &i in a.1.iter().chain(&b.1) {
                    lens[i] += 1;
                }
                nodes.push((a.0 + b.0, [a.1, b.1].concat()));
            }
            if lens.iter().all(|&l| l <= 20) {
                return lens;
            }
            for x in w.iter_mut() {
                *x = 1 + *x / 2;
            }
        }
    }

    /// libbzip2's bit-by-bit decode (`GET_MTF_VAL`), as the table must give
    fn by_bits(h: &Huff, br: &mut Bits) -> Result<u16, ErrorKind> {
        h.decode_slow(br)
    }

    /// `Huff::decode` on a reader
    fn decode(h: &Huff, br: &mut Bits) -> Result<u16, ErrorKind> {
        let (data, mut r) = br.parts();
        let s = h.decode(&mut r, data);
        br.set(r);
        s
    }

    /// The table's way: a pair taken where two or more symbols are still wanted
    fn by_table(h: &Huff, br: &mut Bits, want: usize, out: &mut Vec<u16>) -> Option<ErrorKind> {
        while out.len() < want {
            if want - out.len() >= 2 {
                let e = h.lut[br.peek(LUT_BITS) as usize];
                if e >> CNT_SH == 2 {
                    br.skip_short(e & 31);
                    out.push((e >> SYM1 & 0x1ff) as u16);
                    out.push((e >> SYM2 & 0x1ff) as u16);
                    continue;
                }
            }
            match decode(h, br) {
                Ok(s) => out.push(s),
                Err(k) => return Some(k),
            }
        }
        None
    }

    #[test]
    fn table_decodes_as_libbzip2() {
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        for case in 0..3000 {
            let lens = lengths(&mut rng);
            let h = Huff::new(&lens);
            let eob = (lens.len() - 1) as u16;
            let data: Vec<u8> = (0..400).map(|_| rng.next() as u8).collect();
            let want = 1 + rng.below(120) as usize;
            let (mut a, mut b) = (Bits::at(&data, 0), Bits::at(&data, 0));
            let mut fast = Vec::new();
            let err = by_table(&h, &mut a, want, &mut fast);
            let mut slow = Vec::new();
            let mut err2 = None;
            while slow.len() < want {
                match by_bits(&h, &mut b) {
                    Ok(s) => slow.push(s),
                    Err(k) => {
                        err2 = Some(k);
                        break;
                    }
                }
            }
            assert_eq!(fast, slow, "case {case}: lengths {lens:?}");
            assert_eq!(err, err2, "case {case}");
            if err.is_none() {
                assert_eq!(a.bit_pos(), b.bit_pos(), "case {case}");
            }
            for (w, &e) in h.lut.iter().enumerate() {
                // each entry's first symbol, as libbzip2's loop gives for its prefix (the
                // table's former, entry by entry, build)
                let mut zn = h.min_len;
                let want = loop {
                    if zn > LUT_BITS {
                        break 0;
                    }
                    let zvec = (w as u32 >> (LUT_BITS - zn)) as i32;
                    if zvec <= h.limit[zn as usize] {
                        break match h.symbol(zn, zvec) {
                            Some(s) if s == eob => EOB_FAST | zn,
                            Some(s) => (s as u32) << SYM1 | 1 << CNT_SH | zn << LEN1 | zn,
                            None => BAD,
                        };
                    }
                    zn += 1;
                };
                let first = if (e >> CNT_SH) != 0 {
                    e & (0x1ff << SYM1) | 1 << CNT_SH | e & (31 << LEN1) | e >> LEN1 & 31
                } else if (e & SUB_PTR) != 0 {
                    0
                } else if e == BAD && lens.iter().copied().max().unwrap_or(0) as u32 <= LUT_BITS {
                    // when max_l <= LUT_BITS, prefixes past limit[max_l] mark BAD directly (0 bits consumed, matching decode_slow)
                    if want == 0 {
                        0
                    } else {
                        e
                    }
                } else {
                    e
                };
                assert_eq!(first, want, "case {case}: entry {w}");
                // a pair never begins with the block's end
                assert!(e >> CNT_SH != 2 || (e >> SYM1 & 0x1ff) as u16 != eob);
            }
        }
    }

    /// MSB-first bits, for a test's blocks
    #[derive(Default)]
    struct Writer {
        out: Vec<u8>,
        n: u64,
    }
    impl Writer {
        fn put(&mut self, v: u64, bits: u32) {
            for i in (0..bits).rev() {
                if self.n % 8 == 0 {
                    self.out.push(0);
                }
                if v >> i & 1 == 1 {
                    *self.out.last_mut().unwrap() |= 0x80 >> (self.n % 8);
                }
                self.n += 1;
            }
        }
    }

    /// A block's symbols read as they were before the symbol loop held the reader's state
    /// in registers: a table lookup at a time on the reader itself, a pair where two or
    /// more of the group are still wanted, a code longer than the table's bits decoded
    /// bit by bit (libbzip2's loop), each group's end checked for an overrun
    fn symbols_in_memory(
        br: &mut Bits,
        tables: &[Huff],
        selectors: &[u8],
        eob: u16,
    ) -> (Result<(), ErrorKind>, Vec<u16>) {
        let mut out = Vec::new();
        let mut sel = selectors.iter();
        let res = 'groups: loop {
            let Some(&g) = sel.next() else {
                break Err(ErrorKind::SelectorOverrun);
            };
            let t = &tables[g as usize];
            let mut left = GROUP;
            while left > 0 {
                let e = t.lut[br.peek(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c == 2 && left >= 2 {
                    br.skip_short(e & 31);
                    out.extend([(e >> SYM1 & 0x1ff) as u16, (e >> SYM2 & 0x1ff) as u16]);
                    left -= 2;
                } else if c != 0 {
                    br.skip_short(e >> LEN1 & 31);
                    out.push((e >> SYM1 & 0x1ff) as u16);
                    left -= 1;
                } else if (e & EOB_FAST) != 0 {
                    br.skip_short(e & 63);
                    out.push(eob);
                    break 'groups Ok(());
                } else if (e & BAD) != 0 {
                    break 'groups Err(ErrorKind::BadHuffmanCode);
                } else {
                    let s = t.decode_slow(br);
                    br.refill();
                    match s {
                        Ok(s) => out.push(s),
                        Err(k) => break 'groups Err(k),
                    }
                    left -= 1;
                }
                if out.last() == Some(&eob) {
                    break 'groups Ok(());
                }
            }
            if br.overran() {
                break Err(ErrorKind::Truncated);
            }
        };
        (res, out)
    }

    #[test]
    fn symbols_read_as_in_memory() {
        // blocks written whole (header, selectors, code tables), any code lengths (real
        // codes, and corrupt ones whose table holds BAD entries), then symbols coded with
        // them and ending in EOB, or random bits, a bit flipped, the input cut short: the
        // symbol loop, its reader's state in registers, gives the same symbols, the same
        // error and the same end bit (so the same `overran`) as the reader in memory
        let mut rng = Rng(0x2545_f491_4f6c_dd1d);
        let mut seen = [0usize; 6]; // ok, BadHuffmanCode, SelectorOverrun, overran, other, long codes
        for case in 0..4000 {
            let alpha = 3 + rng.below(256) as usize;
            let n_groups = 2 + rng.below(5) as usize;
            let lens: Vec<Vec<u8>> = (0..n_groups).map(|_| lengths_of(&mut rng, alpha)).collect();
            let tables: Vec<Huff> = lens.iter().map(|l| Huff::new(l)).collect();
            let n_sel = 1 + rng.below(40) as usize;
            let selectors: Vec<u8> = (0..n_sel)
                .map(|_| rng.below(n_groups as u64) as u8)
                .collect();
            let eob = (alpha - 1) as u16;
            let mut w = Writer::default();
            let lead = rng.below(8) as u32;
            w.put(rng.next(), lead);
            let start = w.n;
            w.put(BLOCK_MAGIC, 48);
            w.put(rng.below(1u64 << 32), 32);
            w.put(0, 1);
            w.put(rng.below(900_000), 24);
            // the bytes in use: the first alpha - 2
            let used = alpha - 2;
            let words = used.div_ceil(16);
            w.put(((1u64 << words) - 1) << (16 - words), 16);
            for i in 0..words {
                let k = (used - 16 * i).min(16);
                w.put(((1u64 << k) - 1) << (16 - k), 16);
            }
            w.put(n_groups as u64, 3);
            w.put(n_sel as u64, 15);
            let mut mtf: Vec<u8> = (0..n_groups as u8).collect();
            for &g in &selectors {
                let j = mtf.iter().position(|&x| x == g).unwrap();
                w.put((1 << (j + 1)) - 2, j as u32 + 1);
                mtf.remove(j);
                mtf.insert(0, g);
            }
            for l in &lens {
                let mut curr = l[0] as i32;
                w.put(curr as u64, 5);
                for &x in l {
                    while curr != x as i32 {
                        let up = (x as i32) > curr;
                        w.put(if up { 0b10 } else { 0b11 }, 2);
                        curr += if up { 1 } else { -1 };
                    }
                    w.put(0, 1);
                }
            }
            let sym_start = w.n;
            let complete = lens
                .iter()
                .all(|l| l.iter().map(|&x| 1u64 << (20 - x)).sum::<u64>() <= 1 << 20);
            if complete && rng.below(4) != 0 {
                // symbols (small ones likelier, as MTF values are), the last EOB, each in
                // its group's code (libbzip2's canonical code: by length, then symbol)
                let m = 1 + rng.below((n_sel * GROUP) as u64) as usize;
                for i in 0..m {
                    let s = if i + 1 == m {
                        eob
                    } else {
                        let k = rng.below(9);
                        (rng.below(1 << k) % (alpha as u64 - 1)) as u16
                    };
                    let t = &tables[selectors[i / GROUP] as usize];
                    let l = &lens[selectors[i / GROUP] as usize];
                    let at = t.perm[..alpha].iter().position(|&p| p == s).unwrap();
                    let zn = l[s as usize] as u32;
                    w.put((at as i32 + t.base[zn as usize]) as u64, zn);
                }
            }
            for _ in 0..rng.below(200) {
                w.put(rng.next(), 64);
            }
            let mut data = w.out;
            let end = data.len() as u64 * 8;
            if rng.below(3) == 0 && end > sym_start {
                let bit = sym_start + rng.below(end - sym_start);
                data[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
            }
            // (cut past the code tables' last byte: zeros read in place of a table's bits
            // would give other tables than the reference's)
            let first = sym_start.div_ceil(8);
            if rng.below(3) == 0 {
                let cut = first + rng.below(data.len() as u64 - first + 1);
                data.truncate(cut as usize);
            }
            let mut a = Bits::at(&data, start);
            let got = read_item(&mut a, 9, Vec::new());
            let mut b = Bits::at(&data, sym_start);
            let (want, syms) = symbols_in_memory(&mut b, &tables, &selectors, eob);
            let ctx = format!("case {case}: alpha {alpha} groups {n_groups} selectors {n_sel}");
            assert_eq!(a.bit_pos(), b.bit_pos(), "{ctx}");
            assert_eq!(a.overran(), b.overran(), "{ctx}");
            match (got, want) {
                (Ok(Item::Block(x)), Ok(())) => {
                    assert!(x.syms == syms, "{ctx}");
                    assert_eq!(x.eob, eob, "{ctx}");
                    seen[0] += 1;
                    seen[5] += syms
                        .iter()
                        .enumerate()
                        .filter(|&(i, &s)| {
                            lens[selectors[i / GROUP] as usize][s as usize] as u32 > LUT_BITS
                        })
                        .count();
                }
                (Err((k, _)), Err(k2)) => {
                    assert_eq!(k, k2, "{ctx}");
                    seen[match k {
                        ErrorKind::BadHuffmanCode => 1,
                        ErrorKind::SelectorOverrun => 2,
                        ErrorKind::Truncated => 3,
                        _ => 4,
                    }] += 1;
                }
                (got, want) => panic!(
                    "{ctx}: {:?} against {want:?}",
                    got.map(|_| ()).map_err(|e| e.0)
                ),
            }
            if a.overran() {
                seen[3] += 1;
            }
        }
        // every way out of the loop taken, and codes longer than the table's bits read
        assert!(
            seen[..4].iter().all(|&k| k > 100) && seen[5] > 1000,
            "{seen:?}"
        );
    }
}
