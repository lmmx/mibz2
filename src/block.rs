//! A block's header and its Huffman-coded symbols, read in order from the stream: the one
//! part of the decode that must be sequential, since a block's end is known only once its
//! symbols are decoded. Every check libbzip2 1.0.8 makes here (`decompress.c`) is made
//! the same way, in the same terms, so a stream libbzip2 refuses is refused here too.

use std::mem::MaybeUninit;

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

// (a block's symbols move through by value, once a block: boxing them would allocate
// a block's worth for nothing)
#[allow(clippy::large_enum_variant)]
pub enum Item {
    Block(Symbols),
    /// the stream's end, and the combined CRC it stores
    End(u32),
}

/// A group's Huffman code as libbzip2 decodes it (`BZ2_hbCreateDecodeTables` and
/// `GET_MTF_VAL`): canonical limits by code length, for the codes decoded bit by bit, and
/// the group's direct table (`Lut`, built by `build_lut`) for the rest.
struct Huff {
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
}

/// The bits a table lookup takes: the table has 2^LUT_BITS entries
const LUT_BITS: u32 = 11;
type Lut = [u64; 1 << LUT_BITS];
/// A table entry: the whole codes (up to three) its LUT_BITS-bit window begins with, as
/// libbzip2's bit-by-bit loop decodes them one after another, stopping before a code it
/// refuses, before the block's end (EOB) and before a code that runs past the window:
/// - bits 0-3: the bits of the entry's codes (bits 4-5 clear, so the entry as a shift
///   count consumes them);
/// - bits 6-7: how many codes (1-3); 0 when the window's first code is not decoded here
///   (longer than the window, refused, or EOB), and bits 8-12 then give the code length
///   the bit-by-bit loop resumes at, every shorter one having been ruled out;
/// - bits 8-11: the first code's bits; bits 12-15: the first two's (the first's, if one);
/// - bits 16-63: the symbols, 16 bits each, in order (0 where none).
///
/// So an entry's symbols are stored as one 64-bit word and a group's last symbols take an
/// entry's first one or two only (its 50 symbols are this table's, the next group's
/// another's).
const CNT: u32 = 6;
const F1: u32 = 8;
const F2: u32 = 12;
const SYMS: u32 = 16;
/// Table lookups after each refill: each takes at most LUT_BITS bits of the 56 a refill
/// leaves (a code decoded bit by bit refills after it)
const STEPS: usize = 4;
const _: () = assert!(STEPS as u32 * LUT_BITS <= 56 && LUT_BITS < 15);

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
        Huff {
            limit,
            base,
            perm,
            min_len: min_len as u32,
            alpha: alpha as i32,
        }
    }

    /// The group's table into `t` (`codes`: scratch). libbzip2's loop takes the first
    /// length `zn` (from min_len) whose `zn`-bit prefix is at most limit[zn]. The prefixes
    /// not taken by then are always those at or above some `a` (an upper range: a prefix
    /// past the limit has children past twice it), so each length takes the range
    /// a..=limit[zn], and the rest go on. So the codes of at most `r` bits, in that order
    /// (by length, then value), tile the `r`-bit values from 0 up, each code of `l` bits
    /// 2^(r - l) of them, and the values past them are codes longer than `r` bits. The
    /// table is that tiling three deep: each code of a window's first `r` = LUT_BITS bits,
    /// then each code of the `r` left after it, then each of the `r` left after both.
    /// Each entry is written once.
    #[inline(never)]
    fn build_lut(&self, eob: u16, codes: &mut Vec<u32>, t: &mut Lut) {
        const K: u32 = LUT_BITS;
        // each code of at most K bits: its length (bits 0-4) and, if a lookup may decode
        // it (libbzip2 takes it, and it is not EOB), bit 8 and its symbol (bits 16-31);
        // the codes of at most r bits are codes[..upto[r]]
        codes.clear();
        let mut upto = [0usize; K as usize + 1];
        let mut a = 0i64;
        for zn in self.min_len..=K {
            let hi = (self.limit[zn as usize] as i64).min((1i64 << zn) - 1);
            for v in a..=hi {
                codes.push(match self.symbol(zn, v as i32) {
                    Some(s) if s != eob => zn | 1 << 8 | (s as u32) << 16,
                    _ => zn,
                });
            }
            upto[zn as usize] = codes.len();
            a = 2 * a.max(hi + 1);
        }
        let sym = |c: u32| (c >> 16) as u64;
        let mut x0 = 0;
        for &c1 in &codes[..upto[K as usize]] {
            let l1 = c1 & 31;
            let w1 = &mut t[x0..x0 + (1 << (K - l1))];
            x0 += w1.len();
            if c1 & 1 << 8 == 0 {
                // refused, or EOB: decoded bit by bit, from its length on
                w1.fill((l1 as u64) << F1);
                continue;
            }
            let l1 = l1 as u64;
            let e1 = l1 | 1 << CNT | l1 << F1 | l1 << F2 | sym(c1) << SYMS;
            let r1 = K - l1 as u32;
            let mut x1 = 0;
            for &c2 in &codes[..upto[r1 as usize]] {
                let l2 = c2 & 31;
                let w2 = &mut w1[x1..x1 + (1 << (r1 - l2))];
                x1 += w2.len();
                if c2 & 1 << 8 == 0 {
                    w2.fill(e1);
                    continue;
                }
                let e2 = (e1 + l2 as u64 + (1 << CNT) + ((l2 as u64) << F2))
                    | sym(c2) << (SYMS + 16);
                let r2 = r1 - l2;
                let mut x2 = 0;
                for &c3 in &codes[..upto[r2 as usize]] {
                    let l3 = c3 & 31;
                    let w3 = &mut w2[x2..x2 + (1 << (r2 - l3))];
                    x2 += w3.len();
                    w3.fill(if c3 & 1 << 8 == 0 {
                        e2
                    } else {
                        (e2 + l3 as u64 + (1 << CNT)) | sym(c3) << (SYMS + 32)
                    });
                }
                // (codes longer than r2: the window holds two)
                w2[x2..].fill(e2);
            }
            w1[x1..].fill(e1);
        }
        // codes longer than the window: the loop resumes past it
        t[x0..].fill((self.min_len.max(K + 1) as u64) << F1);
    }

    /// The symbol of a code (libbzip2's perm index check; an index past the alphabet,
    /// from an oversubscribed code, would read a stale entry there, and is refused here)
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    /// One code bit by bit, as libbzip2's loop takes it, from length `zn` on (min_len, or
    /// a length every shorter one has been ruled out below)
    #[cold]
    fn decode_from(&self, br: &mut Bits, zn: u32) -> Result<u16, ErrorKind> {
        let mut zn = zn;
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

/// An entry's symbols into `out` from `n`: one 64-bit store of four lanes (the fourth 0,
/// and those past the symbols taken overwritten by the next), or as many lanes as `out`
/// has left at its end
#[inline(always)]
fn put(out: &mut [MaybeUninit<u16>], n: usize, v: u64) {
    if let Some(d) = out.get_mut(n..n + 4) {
        d[0].write(v as u16);
        d[1].write((v >> 16) as u16);
        d[2].write((v >> 32) as u16);
        d[3].write((v >> 48) as u16);
    } else {
        for (i, d) in out[n..].iter_mut().enumerate() {
            d.write((v >> (16 * i)) as u16);
        }
    }
}

/// The tables a block's symbols are decoded with: built anew for each block, their memory
/// kept from block to block (a scanner's)
pub(crate) struct Tables {
    luts: Box<[Lut]>,
    huffs: Vec<Huff>,
    codes: Vec<u32>,
}

impl Default for Tables {
    fn default() -> Self {
        Tables {
            luts: vec![[0u64; 1 << LUT_BITS]; MAX_GROUPS].into_boxed_slice(),
            huffs: Vec::with_capacity(MAX_GROUPS),
            codes: Vec::with_capacity(1 << LUT_BITS),
        }
    }
}

#[cfg(any(test, feature = "trace"))]
impl Tables {
    /// The bytes they hold (the footprint ledger's figure)
    pub(crate) const BYTES: usize =
        MAX_GROUPS * (std::mem::size_of::<Lut>() + std::mem::size_of::<Huff>())
            + (4 << LUT_BITS);
}

/// The next item of a stream of level `level` (1-9) from `br`, at a block header or the
/// stream's end: a block's header and its symbols (into `syms`, reused), or the end and
/// its CRC. A block found to run past the input (`br.overran()`) is the caller's to judge.
pub fn read_item(
    br: &mut Bits,
    level: u32,
    syms: Vec<u16>,
) -> Result<Item, (ErrorKind, Vec<u16>)> {
    read_item_with(br, level, syms, &mut Tables::default())
}

/// `read_item`, its tables in `tables` (kept from block to block by the caller)
pub(crate) fn read_item_with(
    br: &mut Bits,
    level: u32,
    syms: Vec<u16>,
    tables: &mut Tables,
) -> Result<Item, (ErrorKind, Vec<u16>)> {
    read_item_impl::<false>(br, level, syms, tables)
}

/// With BY_BITS (the tests' reference), each symbol decoded bit by bit as libbzip2 does
fn read_item_impl<const BY_BITS: bool>(
    br: &mut Bits,
    level: u32,
    mut syms: Vec<u16>,
    tables: &mut Tables,
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
    let eob = (alpha - 1) as u16;
    let Tables { luts, huffs, codes } = tables;
    huffs.clear();
    let mut lens = [0u8; MAX_ALPHA];
    for g in 0..n_groups {
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
        let h = Huff::new(&lens[..alpha]);
        if !BY_BITS {
            h.build_lut(eob, codes, &mut luts[g]);
        }
        huffs.push(h);
    }
    syms.clear();
    syms.reserve(MAX_SYMS);
    // the symbols written into `syms`' spare capacity (MAX_SYMS: every selector's group
    // whole), then its length set once
    let out = &mut syms.spare_capacity_mut()[..MAX_SYMS];
    let mut n = 0;
    let mut sel = 0;
    let res = 'groups: loop {
        let Some(&g) = selectors.get(sel) else {
            break Err(ErrorKind::SelectorOverrun);
        };
        let (t, h) = (&luts[g as usize], &huffs[g as usize]);
        sel += 1;
        let mut left = GROUP;
        // a code the table does not decode: bit by bit, from the length its entry gives
        macro_rules! by_bits {
            ($zn:expr) => {
                match h.decode_from(br, $zn) {
                    Ok(s) => {
                        out[n].write(s);
                        n += 1;
                        left -= 1;
                        if s == eob {
                            break 'groups Ok(());
                        }
                    }
                    Err(k) => break 'groups Err(k),
                }
            };
        }
        if BY_BITS {
            while left > 0 {
                by_bits!(h.min_len);
            }
        }
        // whole entries while the group has room for any (three symbols a lookup)
        while left >= 3 * STEPS {
            br.refill();
            for _ in 0..STEPS {
                let e = t[br.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT & 3) as usize;
                if c != 0 {
                    br.skip_short((e & 63) as u32);
                    put(out, n, e >> SYMS);
                    n += c;
                    left -= c;
                } else {
                    by_bits!((e >> F1 & 31) as u32);
                    br.refill();
                }
            }
        }
        // the group's last symbols: an entry's first `left` at most, its bits those of
        // its first one, its first two, or all
        while left > 0 {
            let e = t[br.peek(LUT_BITS) as usize];
            let c = (e >> CNT & 3) as usize;
            if c != 0 {
                let take = c.min(left);
                let field = (4 * left.min(3) as u32 + 4) & 15;
                br.skip_short((e >> field & 15) as u32);
                put(out, n, e >> SYMS);
                n += take;
                left -= take;
            } else {
                by_bits!((e >> F1 & 31) as u32);
            }
        }
        if br.overran() {
            break Err(ErrorKind::Truncated);
        }
    };
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

    /// A Huffman code's lengths for `alpha` symbols from weights (at most 20 bits), as
    /// bzip2's encoder makes them: the weights packaged two at a time, halved until no
    /// length passes 20
    fn huffman(mut w: Vec<u64>) -> Vec<u8> {
        let alpha = w.len();
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

    /// Code lengths for `alpha` symbols (3-258): a real code's (a Huffman code of random
    /// weights, skewed as an MTF stream's or not), all equal, one short code and the rest
    /// 20 bits, or any lengths 1-20 at all (oversubscribed or incomplete, as a corrupt
    /// block's)
    fn lengths_for(rng: &mut Rng, alpha: usize) -> Vec<u8> {
        match rng.below(8) {
            0 | 1 => {
                let lo = 1 + rng.below(20) as u8;
                let hi = lo + rng.below(21 - lo as u64) as u8;
                (0..alpha)
                    .map(|_| lo + rng.below((hi - lo + 1) as u64) as u8)
                    .collect()
            }
            2 => vec![1 + rng.below(20) as u8; alpha],
            3 => {
                let mut l = vec![20u8; alpha];
                l[rng.below(alpha as u64) as usize] = 1 + rng.below(3) as u8;
                l
            }
            4 => {
                // an MTF stream's: the first few symbols far likelier
                let w = (0..alpha)
                    .map(|i| 1 + (1u64 << 20 >> (2 * i).min(20)) + rng.below(4))
                    .collect();
                huffman(w)
            }
            _ => huffman(
                (0..alpha)
                    .map(|_| {
                        let top = 1 << rng.below(16);
                        1 + rng.below(top)
                    })
                    .collect(),
            ),
        }
    }

    fn lengths(rng: &mut Rng) -> Vec<u8> {
        let alpha = 3 + rng.below(256) as usize;
        lengths_for(rng, alpha)
    }

    /// The entry libbzip2's loop gives window `w`: its codes one after another, each taken
    /// by the loop from min_len, while each lies whole within the window, up to three,
    /// stopping before a refused code or EOB; with none, the length the loop resumes at
    fn reference_entry(h: &Huff, eob: u16, w: u32) -> u64 {
        const K: u32 = LUT_BITS;
        // `n` bits from bit `pos` of the window (zeros past it)
        let bits = |pos: u32, n: u32| ((w as u64) << (64 - K) << pos >> (64 - n)) as i32;
        let (mut pos, mut syms, mut lens) = (0, Vec::new(), Vec::new());
        while syms.len() < 3 {
            let mut zn = h.min_len;
            let taken = loop {
                if pos + zn > K {
                    break None;
                }
                if bits(pos, zn) <= h.limit[zn as usize] {
                    break Some(zn);
                }
                zn += 1;
            };
            let Some(zn) = taken else { break };
            match h.symbol(zn, bits(pos, zn)) {
                Some(s) if s != eob => {
                    syms.push(s as u64);
                    lens.push(zn as u64);
                    pos += zn;
                }
                _ => break,
            }
        }
        if syms.is_empty() {
            let mut zn = h.min_len;
            while zn <= K && bits(0, zn) > h.limit[zn as usize] {
                zn += 1;
            }
            return (zn as u64) << F1;
        }
        let f2 = lens[0] + lens.get(1).copied().unwrap_or(0);
        let mut e = lens.iter().sum::<u64>()
            | (syms.len() as u64) << CNT
            | lens[0] << F1
            | f2 << F2;
        for (i, s) in syms.iter().enumerate() {
            e |= s << (SYMS + 16 * i as u32);
        }
        e
    }

    #[test]
    fn table_is_libbzip2s_loop_window_by_window() {
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        let mut t = Box::new([0u64; 1 << LUT_BITS]);
        let mut codes = Vec::new();
        let mut cases: Vec<Vec<u8>> = (0..1500).map(|_| lengths(&mut rng)).collect();
        // the smallest alphabet (one byte in use) in every shape; one code of each length
        // and the rest 20 bits; every length 20
        for _ in 0..200 {
            cases.push(lengths_for(&mut rng, 3));
        }
        for l in 1..=20 {
            let mut c = vec![20u8; 40];
            c[(l as usize * 7) % 40] = l;
            cases.push(c);
            cases.push(vec![l; 3]);
        }
        for (case, lens) in cases.iter().enumerate() {
            let h = Huff::new(lens);
            let eob = (lens.len() - 1) as u16;
            // (a stale table from the case before, so an entry not written shows)
            h.build_lut(eob, &mut codes, &mut t);
            for (w, &e) in t.iter().enumerate() {
                assert_eq!(
                    e,
                    reference_entry(&h, eob, w as u32),
                    "case {case}: lengths {lens:?}, window {w:#x}"
                );
            }
        }
    }

    /// Bits, MSB first, into bytes
    #[derive(Default)]
    struct Writer {
        out: Vec<u8>,
        acc: u64,
        n: u32,
    }
    impl Writer {
        fn put(&mut self, v: u64, bits: u32) {
            for i in (0..bits).rev() {
                self.acc = self.acc << 1 | (v >> i & 1);
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
                self.out.push((self.acc << (8 - self.n)) as u8);
            }
            self.out
        }
    }

    /// A block's header up to its symbols: `n_in_use` bytes in use, the groups' code
    /// lengths and the selectors given
    fn header(w: &mut Writer, n_in_use: usize, tables: &[Vec<u8>], selectors: &[u8]) {
        w.put(BLOCK_MAGIC, 48);
        w.put(0x1234_5678, 32);
        w.put(0, 1);
        w.put(7, 24);
        let used: Vec<usize> = (0..n_in_use).map(|i| i * 256 / n_in_use).collect();
        let mut in_use16 = 0u64;
        for &b in &used {
            in_use16 |= 0x8000 >> (b / 16);
        }
        w.put(in_use16, 16);
        for i in 0..16 {
            if in_use16 & (0x8000 >> i) != 0 {
                let mut word = 0;
                for &b in used.iter().filter(|&&b| b / 16 == i) {
                    word |= 0x8000 >> (b % 16);
                }
                w.put(word, 16);
            }
        }
        w.put(tables.len() as u64, 3);
        w.put(selectors.len() as u64, 15);
        let mut mtf: Vec<u8> = (0..tables.len() as u8).collect();
        for &s in selectors {
            let j = mtf.iter().position(|&x| x == s).unwrap();
            w.put((1 << (j + 1)) - 2, j as u32 + 1);
            mtf.remove(j);
            mtf.insert(0, s);
        }
        for lens in tables {
            let mut curr = lens[0] as i32;
            w.put(curr as u64, 5);
            for &l in lens {
                while curr < l as i32 {
                    w.put(0b10, 2);
                    curr += 1;
                }
                while curr > l as i32 {
                    w.put(0b11, 2);
                    curr -= 1;
                }
                w.put(0, 1);
            }
        }
    }

    /// The canonical codes of a complete code's lengths (libbzip2's order: by length, then
    /// symbol), as (code, length) by symbol
    fn canonical(lens: &[u8]) -> Vec<(u64, u32)> {
        let mut codes = vec![(0, 0); lens.len()];
        let mut code = 0u64;
        for l in 1..=20u8 {
            for (s, _) in lens.iter().enumerate().filter(|&(_, &x)| x == l) {
                codes[s] = (code, l as u32);
                code += 1;
            }
            code <<= 1;
        }
        codes
    }

    /// The block decoded both ways: the tables', and bit by bit (libbzip2's loop for every
    /// symbol); the same symbols or the same error, at the same bit
    fn same_both_ways(data: &[u8], what: &str) -> Option<Vec<u16>> {
        let mut tables = Tables::default();
        let (mut a, mut b) = (Bits::at(data, 0), Bits::at(data, 0));
        let fast = read_item_impl::<false>(&mut a, 9, Vec::new(), &mut tables);
        let slow = read_item_impl::<true>(&mut b, 9, Vec::new(), &mut tables);
        assert_eq!(a.overran(), b.overran(), "{what}");
        match (fast, slow) {
            (Ok(Item::Block(x)), Ok(Item::Block(y))) => {
                assert_eq!(x.syms, y.syms, "{what}");
                assert_eq!(x.eob, y.eob, "{what}");
                assert_eq!(a.bit_pos(), b.bit_pos(), "{what}");
                Some(x.syms)
            }
            (Err((k, _)), Err((l, _))) => {
                assert_eq!(k, l, "{what}");
                None
            }
            _ => panic!("{what}: one way decoded, the other not"),
        }
    }

    #[test]
    fn blocks_decode_as_bit_by_bit() {
        let mut rng = Rng(0x2545_f491_4f6c_dd1d);
        for case in 0..600 {
            let n_in_use = 1 + rng.below(256) as usize;
            let alpha = n_in_use + 2;
            let n_groups = 2 + rng.below(5) as usize;
            let tables: Vec<Vec<u8>> = (0..n_groups)
                .map(|_| lengths_for(&mut rng, alpha))
                .collect();
            let n_sel = 1 + rng.below(if case % 10 == 0 { 3000 } else { 60 }) as usize;
            let selectors: Vec<u8> = (0..n_sel)
                .map(|_| rng.below(n_groups as u64) as u8)
                .collect();
            let mut w = Writer::default();
            header(&mut w, n_in_use, &tables, &selectors);
            let mut data = w.finish();
            if case % 2 == 0 {
                // the symbols' bits at random (any code at all, EOB where it falls)
                let more = 1 + rng.below(4000) as usize;
                data.extend((0..more).map(|_| rng.next() as u8));
            } else {
                // a real code's symbols, an MTF stream's mostly: each group's own code,
                // EOB at a random place or none (the selectors run out, or the input)
                let mut w = Writer::default();
                let codes: Vec<_> = tables.iter().map(|l| canonical(l)).collect();
                let complete = tables.iter().all(|l| {
                    l.iter().map(|&x| 1u64 << (20 - x)).sum::<u64>() == 1 << 20
                });
                let total = n_sel * GROUP;
                let end = rng.below(total as u64 + 10) as usize;
                for i in 0..total.min(end + 1) {
                    let c = &codes[selectors[i / GROUP] as usize];
                    let s = if i == end {
                        alpha - 1
                    } else if rng.below(4) != 0 {
                        rng.below(3) as usize
                    } else {
                        rng.below(alpha as u64 - 1) as usize
                    };
                    if complete {
                        w.put(c[s].0, c[s].1);
                    } else {
                        w.put(rng.next(), 1 + rng.below(20) as u32);
                    }
                }
                data.extend(w.finish());
                if rng.below(4) == 0 {
                    // cut short
                    let cut = rng.below(data.len() as u64) as usize;
                    data.truncate(cut);
                }
            }
            same_both_ways(&data, &format!("case {case}"));
        }
    }

    /// Every selector's group used (18,002, the most), EOB the very last symbol: the
    /// symbols written up to the buffer's end, as encoded
    #[test]
    fn a_block_of_every_symbol_it_can_hold() {
        let mut rng = Rng(0x0dd_b1a5_e5ba_5e11);
        for shape in 0..3 {
            let alpha = [3, 40, 258][shape];
            let tables: Vec<Vec<u8>> = (0..6)
                .map(|g| {
                    huffman(
                        (0..alpha)
                            .map(|i| 1 + (1u64 << 16 >> (i + g).min(16)) + rng.below(3))
                            .collect(),
                    )
                })
                .collect();
            let selectors: Vec<u8> = (0..MAX_SELECTORS).map(|_| rng.below(6) as u8).collect();
            let mut w = Writer::default();
            header(&mut w, alpha - 2, &tables, &selectors);
            let codes: Vec<_> = tables.iter().map(|l| canonical(l)).collect();
            let want: Vec<u16> = (0..MAX_SYMS)
                .map(|i| {
                    if i == MAX_SYMS - 1 {
                        alpha as u16 - 1
                    } else {
                        rng.below(alpha as u64 - 1) as u16
                    }
                })
                .collect();
            for (i, &s) in want.iter().enumerate() {
                let (c, l) = codes[selectors[i / GROUP] as usize][s as usize];
                w.put(c, l);
            }
            let data = w.finish();
            let got = same_both_ways(&data, &format!("shape {shape}")).unwrap();
            assert_eq!(got, want);
        }
    }
}
