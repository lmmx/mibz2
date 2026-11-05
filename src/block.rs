//! A block's header and its Huffman-coded symbols, read in order from the stream: the one
//! part of the decode that must be sequential, since a block's end is known only once its
//! symbols are decoded. Every check libbzip2 1.0.8 makes here (`decompress.c`) is made
//! the same way, in the same terms, so a stream libbzip2 refuses is refused here too.

use super::bits::{Bits, Reg};
use super::ErrorKind;

/// A block header's magic (pi) and the stream end's (sqrt(pi))
pub const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
pub const END_MAGIC: u64 = 0x1772_4538_5090;

/// libbzip2's bounds
const MAX_GROUPS: usize = 6;
const MAX_ALPHA: usize = 258;
const MAX_SELECTORS: usize = 18002;
const MAX_CODE_LEN: usize = 23; // (the arrays' size; codes are at most 20 bits)
const GROUP: usize = 50;
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

/// A group's Huffman code as libbzip2 decodes it (`BZ2_hbCreateDecodeTables` and
/// `GET_MTF_VAL`): canonical limits by code length, with a direct table for the codes of
/// at most LUT_BITS bits. The table holds exactly what libbzip2's bit-by-bit loop gives
/// for each LUT_BITS-bit prefix: a symbol and its length, an error, or "longer"; and
/// where a second whole code follows the first within the prefix, that one too.
struct Huff {
    lut: [u32; 1 << LUT_BITS],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
}

const LUT_BITS: u32 = 10;
/// A table entry: the bits its symbols take in bits 0-4 (bit 5 clear), the first's alone
/// in bits 6-10, how many symbols (1 or 2) in bits 11-12, the first in bits 13-21 and the
/// second (if any) in bits 22-30. 0: longer than LUT_BITS (decoded bit by bit); BAD: a
/// code libbzip2 refuses. A pair's first symbol is never the block's end (EOB), so a
/// pair is always taken whole.
const BAD: u32 = 1 << 31;
const LEN1: u32 = 6;
const COUNT: u32 = 11;
const SYM1: u32 = 13;
const SYM2: u32 = 22;
/// Table lookups after each refill: each takes at most LUT_BITS bits of the 56 a refill
/// leaves (a code decoded bit by bit refills after it)
const STEPS: usize = 4;

impl Huff {
    fn new(lens: &[u8]) -> Huff {
        let alpha = lens.len();
        let min_len = *lens.iter().min().unwrap() as usize;
        let max_len = *lens.iter().max().unwrap() as usize;
        let mut perm = [0u16; MAX_ALPHA];
        let mut pp = 0;
        for i in min_len..=max_len {
            for (j, &l) in lens.iter().enumerate() {
                if l as usize == i {
                    perm[pp] = j as u16;
                    pp += 1;
                }
            }
        }
        let mut base = [0i32; MAX_CODE_LEN];
        for &l in lens {
            base[l as usize + 1] += 1;
        }
        for i in 1..MAX_CODE_LEN {
            base[i] += base[i - 1];
        }
        let mut limit = [0i32; MAX_CODE_LEN];
        let mut vec = 0i32;
        for i in min_len..=max_len {
            vec += base[i + 1] - base[i];
            limit[i] = vec - 1;
            vec <<= 1;
        }
        for i in min_len + 1..=max_len {
            base[i] = ((limit[i - 1] + 1) << 1) - base[i];
        }
        let mut h = Huff {
            lut: [0; 1 << LUT_BITS],
            limit,
            base,
            perm,
            min_len: min_len as u32,
            alpha: alpha as i32,
        };
        // libbzip2's loop takes the first length `zn` (from min_len) whose `zn`-bit prefix
        // is at most limit[zn]. The prefixes not taken by then are always those at or
        // above some `a` (an upper range: a prefix past the limit has children past twice
        // it), so each length takes the range a..=limit[zn], and the rest go on.
        let mut a = 0i64;
        for zn in h.min_len..=LUT_BITS {
            let top = (1i64 << zn) - 1;
            let hi = (h.limit[zn as usize] as i64).min(top);
            for p in a..=hi {
                let e = match h.symbol(zn, p as i32) {
                    Some(s) => (s as u32) << SYM1 | 1 << COUNT | zn << LEN1 | zn,
                    None => BAD,
                };
                let sh = LUT_BITS - zn;
                h.lut[(p << sh) as usize..((p + 1) << sh) as usize].fill(e);
            }
            a = 2 * a.max(hi + 1);
        }
        // a second code where the first, not the block's end, leaves room for one
        let eob = (alpha - 1) as u32;
        let mask = (1u32 << LUT_BITS) - 1;
        for w in 0..1u32 << LUT_BITS {
            let e = h.lut[w as usize];
            if e >> COUNT & 3 == 0 || e >> SYM1 & 0x1ff == eob {
                continue;
            }
            let l1 = e & 31;
            // (its first symbol only: the entry may be a pair already)
            let e2 = h.lut[(w << l1 & mask) as usize];
            let l2 = e2 >> LEN1 & 31;
            if e2 >> COUNT & 3 == 0 || l1 + l2 > LUT_BITS {
                continue;
            }
            h.lut[w as usize] = (e2 >> SYM1 & 0x1ff) << SYM2
                | e & (0x1ff << SYM1)
                | 2 << COUNT
                | l1 << LEN1
                | (l1 + l2);
        }
        h
    }

    /// The symbol of a code (libbzip2's perm index check; an index past the alphabet,
    /// from an oversubscribed code, would read a stale entry there, and is refused here)
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    /// One symbol (a pair's first only), from a reader's state `r` on its slice `data`
    #[inline(always)]
    fn decode(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
        let e = self.lut[r.peek(data, LUT_BITS) as usize];
        if e >> COUNT & 3 != 0 {
            r.skip_short(e >> LEN1 & 31);
            return Ok((e >> SYM1 & 0x1ff) as u16);
        }
        let (s, after) = self.decode_entry_slow(e, *r, data);
        *r = after;
        s
    }

    /// A table entry with no symbol: an error, or a code longer than the table's bits. The
    /// reader's state comes in and goes out by value (with the state after), so that the
    /// symbol loop's stays in registers across the call.
    #[cold]
    #[inline(never)]
    fn decode_entry_slow(&self, e: u32, r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg) {
        if e == BAD {
            return (Err(ErrorKind::BadHuffmanCode), r);
        }
        let mut br = Bits::of(data, r);
        let s = self.decode_slow(&mut br);
        br.refill();
        (s, br.parts().1)
    }

    #[cold]
    fn decode_slow(&self, br: &mut Bits) -> Result<u16, ErrorKind> {
        let mut zn = self.min_len;
        let mut zvec = br.peek(zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            if zn > 20 {
                return Err(ErrorKind::BadHuffmanCode);
            }
            zvec = br.peek(zn) as i32;
        }
        br.skip(zn);
        self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode)
    }
}

/// The next item of a stream of level `level` (1-9) from `br`, at a block header or the
/// stream's end: a block's header and its symbols (into `syms`, reused), or the end and
/// its CRC. A block found to run past the input (`br.overran()`) is the caller's to judge.
pub fn read_item(
    br: &mut Bits,
    level: u32,
    mut syms: Vec<u16>,
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
    let in_use16 = br.bits(16);
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for i in 0..16 {
        if in_use16 & (0x8000 >> i) != 0 {
            let w = br.bits(16);
            for j in 0..16 {
                if w & (0x8000 >> j) != 0 {
                    seq_to_unseq[n_in_use] = (i * 16 + j) as u8;
                    n_in_use += 1;
                }
            }
        }
    }
    if n_in_use == 0 {
        fail!(ErrorKind::NoSymbols);
    }
    let alpha = n_in_use + 2;
    let n_groups = br.bits(3) as usize;
    if !(2..=MAX_GROUPS).contains(&n_groups) {
        fail!(ErrorKind::BadGroups);
    }
    let n_sel = br.bits(15) as usize;
    if n_sel < 1 {
        fail!(ErrorKind::BadSelectors);
    }
    // the selectors, move-to-front coded; those past MAX_SELECTORS are read and dropped
    // (as libbzip2 1.0.8 does: some encoders round their number up)
    let mut mtf_g = [0u8, 1, 2, 3, 4, 5];
    let mut selectors = Vec::with_capacity(n_sel.min(MAX_SELECTORS));
    for i in 0..n_sel {
        let mut j = 0;
        while br.bit() {
            j += 1;
            if j >= n_groups {
                fail!(ErrorKind::BadSelector);
            }
        }
        if i < MAX_SELECTORS {
            let v = mtf_g[j];
            mtf_g.copy_within(0..j, 1);
            mtf_g[0] = v;
            selectors.push(v);
        }
        if br.overran() {
            fail!(ErrorKind::Truncated);
        }
    }
    let mut tables = Vec::with_capacity(n_groups);
    let mut lens = [0u8; MAX_ALPHA];
    for _ in 0..n_groups {
        let mut curr = br.bits(5) as i32;
        for l in lens[..alpha].iter_mut() {
            loop {
                if !(1..=20).contains(&curr) {
                    fail!(ErrorKind::BadCodeLength);
                }
                if !br.bit() {
                    break;
                }
                if br.bit() {
                    curr -= 1
                } else {
                    curr += 1
                }
                if br.overran() {
                    fail!(ErrorKind::Truncated);
                }
            }
            *l = curr as u8;
        }
        tables.push(Huff::new(&lens[..alpha]));
    }
    let eob = (alpha - 1) as u16;
    syms.clear();
    syms.reserve(MAX_SYMS);
    // the symbols written into `syms`' spare capacity (MAX_SYMS: every selector's group
    // whole), then its length set once
    let out = &mut syms.spare_capacity_mut()[..MAX_SYMS];
    let mut n = 0;
    let mut sel = 0;
    // the reader's state in locals through the symbol loop (in registers: nothing the loop
    // calls takes `br`, the cold paths take the state by value), given back at its end
    let (data, mut r) = br.parts();
    let res = 'groups: loop {
        let Some(&g) = selectors.get(sel) else {
            break Err(ErrorKind::SelectorOverrun);
        };
        let t = &tables[g as usize];
        sel += 1;
        let mut left = GROUP;
        // one table lookup: a symbol, or two where both fit the table's bits, never past
        // the group's 50th (the next group's table is another); `$live` that the bits are
        // live (fewer than STEPS lookups since a refill)
        macro_rules! step {
            ($live:expr) => {
                let e = t.lut[if $live {
                    r.peek_live(LUT_BITS)
                } else {
                    r.peek(data, LUT_BITS)
                } as usize];
                if e >> COUNT & 3 != 0 {
                    r.skip_short(e & 31);
                    let c = (e >> COUNT & 3) as usize;
                    out[n].write((e >> SYM1 & 0x1ff) as u16);
                    out[n + 1].write((e >> SYM2) as u16);
                    n += c;
                    left -= c;
                    // (the last symbol written: a pair's first is never EOB)
                    if (e >> (SYM1 + (SYM2 - SYM1) * (c as u32 - 1)) & 0x1ff) as u16 == eob {
                        break 'groups Ok(());
                    }
                } else {
                    let (s, after) = t.decode_entry_slow(e, r, data);
                    r = after;
                    let s = match s {
                        Ok(s) => s,
                        Err(k) => break 'groups Err(k),
                    };
                    out[n].write(s);
                    n += 1;
                    left -= 1;
                    if s == eob {
                        break 'groups Ok(());
                    }
                }
            };
        }
        // (the refill at the loop's foot, so that the state reaches the next lookups from
        // it in registers on both ways in: refilled at its head, the state crossed the back
        // edge in memory, a store and a reload each refill)
        if left >= 2 * STEPS {
            r.refill(data);
            loop {
                for _ in 0..STEPS {
                    step!(true);
                }
                if left < 2 * STEPS {
                    break;
                }
                r.refill(data);
            }
        }
        while left >= 2 {
            step!(false);
        }
        if left == 1 {
            let s = match t.decode(&mut r, data) {
                Ok(s) => s,
                Err(k) => break 'groups Err(k),
            };
            out[n].write(s);
            n += 1;
            if s == eob {
                break 'groups Ok(());
            }
        }
        if r.overran(data) {
            break Err(ErrorKind::Truncated);
        }
    };
    br.set(r);
    if let Err(k) = res {
        fail!(k);
    }
    // SAFETY: the first n of the spare capacity were written above
    unsafe { syms.set_len(n) };
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
                if e >> COUNT & 3 == 2 {
                    br.skip_short(e & 31);
                    out.push((e >> SYM1 & 0x1ff) as u16);
                    out.push((e >> SYM2) as u16);
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
                            Some(s) => (s as u32) << SYM1 | 1 << COUNT | zn << LEN1 | zn,
                            None => BAD,
                        };
                    }
                    zn += 1;
                };
                let first = match e >> COUNT & 3 {
                    0 => e,
                    _ => e & (0x1ff << SYM1) | 1 << COUNT | e & (31 << LEN1) | e >> LEN1 & 31,
                };
                assert_eq!(first, want, "case {case}: entry {w}");
                // a pair never begins with the block's end
                assert!(e >> COUNT & 3 != 2 || (e >> SYM1 & 0x1ff) as u16 != eob);
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
                let c = (e >> COUNT & 3) as usize;
                if c == 2 && left >= 2 {
                    br.skip_short(e & 31);
                    out.extend([(e >> SYM1 & 0x1ff) as u16, (e >> SYM2) as u16]);
                    left -= 2;
                } else if c != 0 {
                    br.skip_short(e >> LEN1 & 31);
                    out.push((e >> SYM1 & 0x1ff) as u16);
                    left -= 1;
                } else if e == BAD {
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
