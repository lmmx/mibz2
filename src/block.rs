//! A block's header and its Huffman-coded symbols, read in order from the stream: the one
//! part of the decode that must be sequential, since a block's end is known only once its
//! symbols are decoded. Every check libbzip2 1.0.8 makes here (`decompress.c`) is made
//! the same way, in the same terms, so a stream libbzip2 refuses is refused here too.

use super::bits::Bits;
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

pub enum Item {
    Block(Symbols),
    /// the stream's end, and the combined CRC it stores
    End(u32),
}

/// A group's Huffman code as libbzip2 decodes it (`BZ2_hbCreateDecodeTables` and
/// `GET_MTF_VAL`): canonical limits by code length, with a direct table for the codes of
/// at most LUT_BITS bits. The table holds exactly what libbzip2's bit-by-bit loop gives
/// for each LUT_BITS-bit prefix: a symbol and its length, an error, or "longer".
struct Huff {
    lut: [u32; 1 << LUT_BITS],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
}

const LUT_BITS: u32 = 10;
/// A table entry: the code's length in bits 16-20 and its symbol below; 0: longer than
/// LUT_BITS (decoded bit by bit); BAD: a code libbzip2 refuses
const BAD: u32 = u32::MAX;

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
        let mut h = Huff { lut: [0; 1 << LUT_BITS], limit, base, perm, min_len: min_len as u32, alpha: alpha as i32 };
        for w in 0..1u32 << LUT_BITS {
            let mut zn = h.min_len;
            h.lut[w as usize] = loop {
                if zn > LUT_BITS {
                    break 0;
                }
                let zvec = (w >> (LUT_BITS - zn)) as i32;
                if zvec <= h.limit[zn as usize] {
                    break match h.symbol(zn, zvec) {
                        Some(s) => zn << 16 | s as u32,
                        None => BAD,
                    };
                }
                zn += 1;
            };
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

    #[inline(always)]
    fn decode(&self, br: &mut Bits) -> Result<u16, ErrorKind> {
        let e = self.lut[br.peek(LUT_BITS) as usize];
        if e != 0 && e != BAD {
            br.skip(e >> 16);
            return Ok(e as u16);
        }
        if e == BAD {
            return Err(ErrorKind::BadHuffmanCode);
        }
        self.decode_slow(br)
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
pub fn read_item(br: &mut Bits, level: u32, mut syms: Vec<u16>) -> Result<Item, (ErrorKind, Vec<u16>)> {
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
    let mut sel = 0;
    'groups: loop {
        let Some(&g) = selectors.get(sel) else { fail!(ErrorKind::SelectorOverrun) };
        let t = &tables[g as usize];
        sel += 1;
        for _ in 0..GROUP {
            let s = match t.decode(br) {
                Ok(s) => s,
                Err(k) => fail!(k),
            };
            syms.push(s);
            if s == eob {
                break 'groups;
            }
        }
        if br.overran() {
            fail!(ErrorKind::Truncated);
        }
    }
    Ok(Item::Block(Symbols { crc, orig_ptr, seq_to_unseq, syms, eob, max_len }))
}
