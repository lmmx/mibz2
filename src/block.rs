#![deny(unsafe_op_in_unsafe_fn)]
//! A block's header and its Huffman-coded symbols, read in order from the stream: the one
//! part of the decode that must be sequential, since a block's end is known only once its
//! symbols are decoded. Every check libbzip2 1.0.8 makes here (`decompress.c`) is made
//! the same way, in the same terms, so a stream libbzip2 refuses is refused here too.

use super::bits::{Bits, Reg};
use super::huff::{Huff, CHUNK_LEN, GROUP, MAX_ALPHA};
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
/// Skip 2-symbol pair table pass on runt blocks with fewer than this many selectors.
const PAIR_GATE_SEL: usize = 64;

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

/// Reusable front-end scratch (`72 KiB` of coalesced selector runs + `6 x 6 KiB` L1d-resident
/// `Huff` tables + `2 KiB` L1d-hot symbol staging chunk).
pub struct ScanScratch {
    /// the block's selectors as `(group, count)` runs, adjacent runs of different groups
    runs: Box<[(u8, u16); MAX_SELECTORS]>,
    tables: Box<[Huff; MAX_GROUPS]>,
    chunk: Box<[u16; CHUNK_LEN]>,
}

impl Default for ScanScratch {
    fn default() -> Self {
        Self {
            runs: Box::new([(0u8, 0u16); MAX_SELECTORS]),
            tables: Box::new(std::array::from_fn(|_| Huff::default())),
            chunk: Box::new([0u16; CHUNK_LEN]),
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

/// Decode all coalesced same-selector runs of a block into `syms`, preserving exact per-group
/// `Truncated` / `bit_pos` parity whenever input overrun is reachable.
#[inline(always)]
fn decode_runs_inner(
    tables: &[Huff; MAX_GROUPS],
    runs: &[(u8, u16)],
    mut r: Reg,
    data: &[u8],
    chunk: &mut [u16; CHUNK_LEN],
    syms: &mut Vec<u16>,
) -> (Result<(), ErrorKind>, Reg) {
    let data_len = data.len();
    let mut ck = 0usize;
    for &(g, n) in runs {
        let mut rem = n as usize;
        let t = &tables[g as usize];
        while rem > 0 {
            // Each symbol consumes at most 20 bits (125 B per 50-symbol group); leaving 16 B of
            // refill/lookahead margin makes input overrun mathematically impossible across
            // `take` coalesced groups. Fast path avoids `/ 125` division completely.
            let need = rem * 125 + 16;
            let (take, check_overran) = if r.pos.saturating_add(need) <= data_len {
                (rem, false)
            } else {
                let safe_m = data_len.saturating_sub(r.pos.saturating_add(16)) / 125;
                if safe_m >= 1 {
                    (safe_m, false)
                } else {
                    (1, true)
                }
            };
            match t.decode_run(&mut r, data, chunk, &mut ck, syms, take * GROUP) {
                Ok(true) => return (Ok(()), r),
                Ok(false) => {}
                Err(err) => return (Err(err), r),
            }
            if check_overran && r.pos > data_len && r.overran(data) {
                return (Err(ErrorKind::Truncated), r);
            }
            rem -= take;
        }
    }
    (Err(ErrorKind::SelectorOverrun), r)
}

/// Block-level BMI2 specialisation of [`decode_runs_inner`]: hoists dispatch to 1 call/block
/// so LLVM emits `shlx`/`shrx` throughout the inlined `decode_run` hot loop with zero
/// `vzeroupper` and zero unsafe operations in the body (`#![deny(unsafe_op_in_unsafe_fn)]`).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
unsafe fn decode_runs_bmi2(
    tables: &[Huff; MAX_GROUPS],
    runs: &[(u8, u16)],
    r: Reg,
    data: &[u8],
    chunk: &mut [u16; CHUNK_LEN],
    syms: &mut Vec<u16>,
) -> (Result<(), ErrorKind>, Reg) {
    decode_runs_inner(tables, runs, r, data, chunk, syms)
}

fn decode_runs_scalar(
    tables: &[Huff; MAX_GROUPS],
    runs: &[(u8, u16)],
    r: Reg,
    data: &[u8],
    chunk: &mut [u16; CHUNK_LEN],
    syms: &mut Vec<u16>,
) -> (Result<(), ErrorKind>, Reg) {
    decode_runs_inner(tables, runs, r, data, chunk, syms)
}

fn decode_runs(
    tables: &[Huff; MAX_GROUPS],
    runs: &[(u8, u16)],
    r: Reg,
    data: &[u8],
    chunk: &mut [u16; CHUNK_LEN],
    syms: &mut Vec<u16>,
) -> (Result<(), ErrorKind>, Reg) {
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("bmi2") {
        // SAFETY: `std::is_x86_feature_detected!("bmi2")` guarantees the CPU executes BMI2
        // instructions (`shlx`/`shrx`). `decode_runs_bmi2` executes only safe Rust under
        // `#![deny(unsafe_op_in_unsafe_fn)]`.
        return unsafe { decode_runs_bmi2(tables, runs, r, data, chunk, syms) };
    }
    decode_runs_scalar(tables, runs, r, data, chunk, syms)
}

/// `LOW_MASK[j]`: the MTF list's bytes below position `j`
const LOW_MASK: [u64; MAX_GROUPS] = [0, 0xff, 0xffff, 0xff_ffff, 0xffff_ffff, 0xff_ffff_ffff];

/// Move MTF position `j`'s group to the front, returning it
#[inline(always)]
fn mtf_take(mtf: &mut u64, j: usize) -> u8 {
    let v = (*mtf >> (j * 8)) & 0xff;
    let lm = LOW_MASK[j];
    *mtf = (*mtf & !((lm << 8) | 0xff)) | ((*mtf & lm) << 8) | v;
    v as u8
}

/// A block's `n_sel` MTF-coded selectors into `runs` as `(group, count)` runs, adjacent runs
/// of different groups (so the decode never rescans selectors to coalesce them). Returns the
/// run count and the mask of groups used. Every `0` bit is a selector repeating the front
/// group, so where the input cannot be overrun a run of them is counted with one
/// `leading_zeros`, one iteration per run; otherwise one selector at a time, with
/// libbzip2's overrun check after each. Both read exactly the same bits, and fail at the
/// same bit with the same error.
#[inline(always)]
fn read_selectors(
    r: &mut Reg,
    data: &[u8],
    n_groups: usize,
    n_sel: usize,
    runs: &mut [(u8, u16); MAX_SELECTORS],
) -> Result<(usize, u8), ErrorKind> {
    let data_len = data.len();
    let mut mtf: u64 = 0x0000_0504_0302_0100;
    let mut n_runs = 0usize;
    let mut used = 0u8;
    // a selector takes at most `n_groups <= 6` bits, so `n_sel` bytes past the reader's
    // position (plus a refill's 8) cannot run out: no overrun check is needed
    if n_sel <= MAX_SELECTORS && r.pos.saturating_add(n_sel + 8) <= data_len {
        if r.live < 8 {
            r.refill(data);
        }
        let j = (!r.buf).leading_zeros() as usize;
        if j >= n_groups {
            r.skip_short(n_groups as u32);
            return Err(ErrorKind::BadSelector);
        }
        r.skip_short((j + 1) as u32);
        let mut g = mtf_take(&mut mtf, j);
        let mut run = 1usize;
        let mut rem = n_sel - 1;
        while rem > 0 {
            if r.live < 8 {
                r.refill(data);
            }
            // the `0` bits: selectors repeating `g` (`live` caps the bits counted to real ones)
            let k = r.buf.leading_zeros().min(r.live).min(rem as u32);
            r.skip_short(k);
            run += k as usize;
            rem -= k as usize;
            if rem == 0 || r.live < 8 {
                // (with fewer than 8 bits left the count may have stopped at `live`, not at
                // a `1`: refill and count on)
                continue;
            }
            // a `1` bit, with 8 or more live as libbzip2's reader has: a selector naming
            // another group (MTF position `j >= 1`)
            let j = (!r.buf).leading_zeros() as usize;
            if j >= n_groups {
                r.skip_short(n_groups as u32);
                return Err(ErrorKind::BadSelector);
            }
            r.skip_short((j + 1) as u32);
            rem -= 1;
            runs[n_runs] = (g, run as u16);
            n_runs += 1;
            used |= 1u8 << g;
            g = mtf_take(&mut mtf, j);
            run = 1;
        }
        runs[n_runs] = (g, run as u16);
        n_runs += 1;
        used |= 1u8 << g;
        return Ok((n_runs, used));
    }
    let mut cur: Option<(u8, usize)> = None;
    for i in 0..n_sel {
        if r.live < 8 {
            r.refill(data);
        }
        let j = (!r.buf).leading_zeros() as usize;
        if j >= n_groups {
            r.skip_short(n_groups as u32);
            return Err(ErrorKind::BadSelector);
        }
        r.skip_short((j + 1) as u32);
        if i < MAX_SELECTORS {
            let v = mtf_take(&mut mtf, j);
            used |= 1u8 << v;
            cur = match cur {
                Some((g, n)) if g == v => Some((g, n + 1)),
                Some((g, n)) => {
                    runs[n_runs] = (g, n as u16);
                    n_runs += 1;
                    Some((v, 1))
                }
                None => Some((v, 1)),
            };
        }
        if r.pos > data_len && r.overran(data) {
            return Err(ErrorKind::Truncated);
        }
    }
    if let Some((g, n)) = cur {
        runs[n_runs] = (g, n as u16);
        n_runs += 1;
    }
    Ok((n_runs, used))
}

/// An 8-bit prefix's run of complete delta-coded code lengths (`0`: same, `10`: +1,
/// `11`: -1, each length ended by a `0`): how many (up to 8) and bits they take, each one's
/// offset from the starting length, the net change, and the lowest and highest the length
/// passes through on the way (libbzip2 checks every one is within `1..=20`).
#[derive(Clone, Copy)]
#[repr(C, align(16))]
struct MultiDelta {
    bits: u8,
    count: u8,
    min_off: i8,
    max_off: i8,
    net: i8,
    offs: [i8; 8],
}

const fn build_delta_lut() -> [MultiDelta; 256] {
    let mut lut = [MultiDelta {
        bits: 0,
        count: 0,
        min_off: 0,
        max_off: 0,
        net: 0,
        offs: [0; 8],
    }; 256];
    let mut p = 0usize;
    while p < 256 {
        let mut e = lut[p];
        // `bit`: the next unread bit's index below the prefix's top (`8` once all read)
        let mut bit = 0usize;
        let mut off = 0i8;
        while e.count < 8 {
            let (mut b, mut o, mut lo, mut hi) = (bit, off, e.min_off, e.max_off);
            let mut ended = false;
            while b < 8 {
                let first = (p >> (7 - b)) & 1;
                b += 1;
                if first == 0 {
                    ended = true;
                    break;
                }
                if b == 8 {
                    break;
                }
                let second = (p >> (7 - b)) & 1;
                b += 1;
                o += if second == 1 { -1 } else { 1 };
                if o < lo {
                    lo = o;
                }
                if o > hi {
                    hi = o;
                }
            }
            if !ended {
                break;
            }
            e.offs[e.count as usize] = o;
            e.count += 1;
            e.bits = b as u8;
            e.min_off = lo;
            e.max_off = hi;
            e.net = o;
            bit = b;
            off = o;
        }
        lut[p] = e;
        p += 1;
    }
    lut
}

/// `4 KiB`, L1d-resident
static DELTA_LUT: [MultiDelta; 256] = build_delta_lut();

/// One table's `lens.len()` delta-coded code lengths. Where 8 or more remain and the next
/// 8 bits are real input (so no overrun check could fire), one `DELTA_LUT` lookup takes up
/// to 8 lengths at once, if every length it passes through is within `1..=20`; otherwise
/// one length at a time, exactly as libbzip2 (`decompress.c`) checks it.
#[inline(always)]
fn read_lengths(r: &mut Reg, data: &[u8], lens: &mut [u8]) -> Result<(), ErrorKind> {
    let data_len = data.len();
    let alpha = lens.len();
    let mut curr = r.bits(data, 5) as i32;
    let mut i = 0usize;
    while i < alpha {
        if r.live < 8 {
            r.refill(data);
        }
        if i + 8 <= alpha && r.pos <= data_len {
            let e = &DELTA_LUT[(r.buf >> 56) as usize];
            if e.count != 0 && curr + e.min_off as i32 >= 1 && curr + e.max_off as i32 <= 20 {
                let base = curr as u8;
                for (d, &o) in lens[i..i + 8].iter_mut().zip(&e.offs) {
                    *d = base.wrapping_add(o as u8);
                }
                curr += e.net as i32;
                r.skip_short(e.bits as u32);
                i += e.count as usize;
                continue;
            }
        }
        if (r.buf >> 63) == 0 && (1..=20).contains(&curr) {
            r.skip_short(1);
            lens[i] = curr as u8;
            i += 1;
            continue;
        }
        loop {
            if !(1..=20).contains(&curr) {
                return Err(ErrorKind::BadCodeLength);
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
                return Err(ErrorKind::Truncated);
            }
        }
        lens[i] = curr as u8;
        i += 1;
    }
    Ok(())
}

/// The next item of a stream of level `level` (1-9) from `br`, at a block header or the
/// stream's end: a block's header and its symbols (into `syms`, reused), or the end and
/// its CRC. A block found to run past the input (`br.overran()`) is the caller's to judge.
///
/// Zero-allocation `read_item` using explicit `ScanScratch` (`Scanner<S>` and parallel workers).
pub fn read_item_with_scratch(
    br: &mut Bits,
    level: u32,
    mut syms: Vec<u16>,
    scratch: &mut ScanScratch,
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

    // the selectors, move-to-front coded, as coalesced runs; those past MAX_SELECTORS are
    // read and dropped (as libbzip2 1.0.8 does: some encoders round their number up)
    let (n_runs, used_groups) =
        match read_selectors(&mut r, data, n_groups, n_sel, &mut scratch.runs) {
            Ok(v) => v,
            Err(k) => {
                br.set(r);
                fail!(k);
            }
        };

    let build_pairs = n_sel >= PAIR_GATE_SEL;
    let mut lens = [0u8; MAX_ALPHA];
    for g in 0..n_groups {
        if let Err(k) = read_lengths(&mut r, data, &mut lens[..alpha]) {
            br.set(r);
            fail!(k);
        }
        if (used_groups >> g) & 1 != 0 {
            scratch.tables[g].build_into(&lens[..alpha], build_pairs);
        }
    }

    let eob = (alpha - 1) as u16;
    if syms.capacity() < MAX_SYMS {
        syms.clear();
        syms.reserve(MAX_SYMS);
    } else {
        syms.clear();
    }
    let (res, r) = decode_runs(
        &scratch.tables,
        &scratch.runs[..n_runs],
        r,
        data,
        &mut scratch.chunk,
        &mut syms,
    );

    br.set(r);
    if let Err(err) = res {
        fail!(err);
    }
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
    use crate::huff::{BAD, CNT_SH, EOB_FAST, LEN1, LUT_BITS, SUB_PTR, SYM1, SYM2, SYM256};

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
                    br.skip_short((e as u8) as u32);
                    out.push(((e >> SYM1) as u8) as u16);
                    out.push(((e >> SYM2) as u8) as u16);
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

    /// Selectors as the `(group, count)` runs the decode takes
    fn runs_of(selectors: &[u8]) -> Vec<(u8, u16)> {
        let mut runs: Vec<(u8, u16)> = Vec::new();
        for &g in selectors {
            match runs.last_mut() {
                Some((h, n)) if *h == g => *n += 1,
                _ => runs.push((g, 1)),
            }
        }
        runs
    }

    /// The selector reader `read_selectors` replaced: one selector a time into a byte
    /// buffer, an overrun check after each, coalesced afterwards
    fn selectors_reference(
        r: &mut Reg,
        data: &[u8],
        n_groups: usize,
        n_sel: usize,
    ) -> Result<(Vec<(u8, u16)>, u8), ErrorKind> {
        let mut mtf: u64 = 0x0000_0504_0302_0100;
        let mut sel = Vec::new();
        let mut used = 0u8;
        for i in 0..n_sel {
            if r.live < 8 {
                r.refill(data);
            }
            let j = (!r.buf).leading_zeros() as usize;
            if j >= n_groups {
                r.skip_short(n_groups as u32);
                return Err(ErrorKind::BadSelector);
            }
            r.skip_short((j + 1) as u32);
            if i < MAX_SELECTORS {
                let sh = (j * 8) as u32;
                let v = ((mtf >> sh) & 0xff) as u8;
                let low_mask = (1u64 << sh) - 1;
                mtf = (mtf & !((low_mask << 8) | 0xff)) | ((mtf & low_mask) << 8) | (v as u64);
                sel.push(v);
                used |= 1u8 << v;
            }
            if r.pos > data.len() && r.overran(data) {
                return Err(ErrorKind::Truncated);
            }
        }
        Ok((runs_of(&sel), used))
    }

    /// The code length reader `read_lengths` replaced: one length a time
    fn lengths_reference(r: &mut Reg, data: &[u8], lens: &mut [u8]) -> Result<(), ErrorKind> {
        let data_len = data.len();
        let mut curr = r.bits(data, 5) as i32;
        for l in lens.iter_mut() {
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
                    return Err(ErrorKind::BadCodeLength);
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
                    return Err(ErrorKind::Truncated);
                }
            }
            *l = curr as u8;
        }
        Ok(())
    }

    /// Bits whose runs of `0`s are long, as real selectors' and code lengths' are
    fn biased_bits(rng: &mut Rng, n: usize) -> Vec<u8> {
        let p_one = 1 + rng.below(8);
        (0..n)
            .map(|_| (0..8).fold(0u8, |b, _| (b << 1) | (rng.below(16) < p_one) as u8))
            .collect()
    }

    #[test]
    fn selectors_and_lengths_read_as_reference() {
        // the same bits, error, end position and runs (or lengths), on both the
        // `leading_zeros`/multi-length fast paths (input to spare) and near or past the
        // input's end (each reader's overrun checks)
        readers_match_reference(0x7a3c_9f12_04d5_be61, 20000, 4000);
    }

    #[test]
    fn miri_selectors_and_lengths_read_as_reference() {
        readers_match_reference(0x0dd1_7e55_a2b9_4c03, 24, 300);
    }

    fn readers_match_reference(seed: u64, cases: usize, max_len: u64) {
        let mut rng = Rng(seed);
        let mut runs = Box::new([(0u8, 0u16); MAX_SELECTORS]);
        for case in 0..cases {
            let len = match case % 4 {
                0 => 1 + rng.below(16) as usize,
                1 => 16 + rng.below(200) as usize,
                _ => 200 + rng.below(max_len) as usize,
            };
            let data = biased_bits(&mut rng, len);
            let start = rng.below(8 * len as u64 + 16);
            let (_, r0) = Bits::at(&data, start).parts();
            if case % 2 == 0 {
                let n_groups = 2 + rng.below(5) as usize;
                let n_sel = match rng.below(8) {
                    0 if cases > 100 => MAX_SELECTORS + 1 + rng.below(100) as usize,
                    1 => 1,
                    _ => 1 + rng.below(4 * len as u64 + 1) as usize,
                };
                let (mut ra, mut rb) = (r0, r0);
                let got = read_selectors(&mut ra, &data, n_groups, n_sel, &mut runs)
                    .map(|(n, used)| (runs[..n].to_vec(), used));
                let want = selectors_reference(&mut rb, &data, n_groups, n_sel);
                let ctx =
                    format!("case {case}: len {len} start {start} groups {n_groups} n_sel {n_sel}");
                assert_eq!(got, want, "{ctx}");
                assert_eq!(ra.bit_pos(), rb.bit_pos(), "{ctx}");
            } else {
                let alpha = 3 + rng.below(256) as usize;
                let (mut ra, mut rb) = (r0, r0);
                let (mut la, mut lb) = (vec![0u8; alpha], vec![0u8; alpha]);
                let got = read_lengths(&mut ra, &data, &mut la);
                let want = lengths_reference(&mut rb, &data, &mut lb);
                let ctx = format!("case {case}: len {len} start {start} alpha {alpha}");
                assert_eq!(got, want, "{ctx}");
                assert_eq!(ra.bit_pos(), rb.bit_pos(), "{ctx}");
                if got.is_ok() {
                    assert_eq!(la, lb, "{ctx}");
                }
            }
        }
    }

    #[test]
    fn delta_lut_matches_bitwise_decode() {
        // every 8-bit prefix: its lengths, bits, net change and range as decoded one bit
        // at a time
        for (p, &e) in DELTA_LUT.iter().enumerate() {
            let mut b = 0usize;
            let mut off = 0i8;
            let (mut lo, mut hi) = (0i8, 0i8);
            for k in 0..e.count as usize {
                loop {
                    let first = (p >> (7 - b)) & 1;
                    b += 1;
                    if first == 0 {
                        break;
                    }
                    off += if (p >> (7 - b)) & 1 == 1 { -1 } else { 1 };
                    b += 1;
                    lo = lo.min(off);
                    hi = hi.max(off);
                }
                assert_eq!(e.offs[k], off, "prefix {p:08b} length {k}");
            }
            assert_eq!(
                (e.bits as usize, e.net, e.min_off, e.max_off),
                (b, off, lo, hi),
                "prefix {p:08b}"
            );
            // and no further complete length fits in the prefix (unless 8 were taken)
            if e.count < 8 {
                let mut c = b;
                let complete = loop {
                    if c >= 8 {
                        break false;
                    }
                    if (p >> (7 - c)) & 1 == 0 {
                        break true;
                    }
                    c += 2;
                };
                assert!(!complete, "prefix {p:08b}");
            }
        }
    }

    #[test]
    fn single_write_build_is_bit_identical_to_reference() {
        // the same sequence of builds into one reused table each way (so stale `SubLut`
        // entries must match too), over real, incomplete and oversubscribed codes,
        // alpha 258 (symbol 256 and EOB 257 short), one-length codes and 20-bit tails
        let mut rng = Rng(0x2545_f491_4f6c_dd1d);
        let mut fixed: Vec<Vec<u8>> = vec![
            vec![1, 1],
            vec![1, 1, 1],
            vec![20; 258],
            vec![1; 258],
            vec![9; 258],
            vec![10; 258],
            vec![11; 258],
        ];
        let mut sym256 = vec![15u8; 258];
        sym256[256] = 2;
        sym256[257] = 2;
        sym256[0] = 3;
        sym256[1] = 4;
        fixed.push(sym256);
        let mut deep: Vec<u8> = (0..258).map(|i| (1 + (i % 20)) as u8).collect();
        deep.sort_unstable();
        fixed.push(deep);
        let mut a = Huff::default();
        let mut b = Huff::default();
        for case in 0..6000 {
            let lens = if case < fixed.len() {
                fixed[case].clone()
            } else if case % 10 == 0 {
                lengths_of(&mut rng, MAX_ALPHA)
            } else {
                lengths(&mut rng)
            };
            for pairs in [true, false] {
                a.build_into(&lens, pairs);
                b.build_into_reference(&lens, pairs);
                assert!(a.identical(&b), "case {case} pairs {pairs}: {lens:?}");
            }
        }
    }

    #[test]
    fn table_decodes_as_libbzip2() {
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        for case in 0..3000 {
            let lens = if case % 20 == 0 {
                lengths_of(&mut rng, MAX_ALPHA)
            } else {
                lengths(&mut rng)
            };
            let mut h = Huff::default();
            h.build_into(&lens, case % 4 != 0);
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
                            Some(256) => SYM256 | zn,
                            Some(s) => (1 << CNT_SH) | (zn << LEN1) | ((s as u32) << SYM1) | zn,
                            None => BAD,
                        };
                    }
                    zn += 1;
                };
                let first = if (e >> CNT_SH) != 0 {
                    (1 << CNT_SH) | (e & (31 << LEN1)) | (e & (0xff << SYM1)) | ((e >> LEN1) & 31)
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
                // a pair never begins or ends with the block's end
                assert!(
                    e >> CNT_SH != 2
                        || (((e >> SYM1) as u8) as u16 != eob && ((e >> SYM2) as u8) as u16 != eob)
                );
                // fast entries keep bits 4..7 zero for low-byte skip_short_shl_u8
                assert!(e >> CNT_SH == 0 || (e as u8) <= LUT_BITS as u8);
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
                    br.skip_short((e as u8) as u32);
                    out.extend([((e >> SYM1) as u8) as u16, ((e >> SYM2) as u8) as u16]);
                    left -= 2;
                } else if c != 0 {
                    br.skip_short((e >> LEN1) & 31);
                    out.push(((e >> SYM1) as u8) as u16);
                    left -= 1;
                } else if (e & EOB_FAST) != 0 {
                    br.skip_short(e & 63);
                    out.push(eob);
                    break 'groups Ok(());
                } else if (e & SYM256) != 0 {
                    br.skip_short(e & 63);
                    out.push(256);
                    left -= 1;
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

    fn write_block(
        rng: &mut Rng,
        alpha: usize,
        lens: &[Vec<u8>],
        tables: &[Huff],
        selectors: &[u8],
        encoded_syms: Option<&[u16]>,
        tail_words: u64,
    ) -> (Vec<u8>, u64, u64) {
        let n_groups = lens.len();
        let n_sel = selectors.len();
        let mut w = Writer::default();
        let lead = rng.below(8) as u32;
        w.put(rng.next(), lead);
        let start = w.n;
        w.put(BLOCK_MAGIC, 48);
        w.put(rng.below(1u64 << 32), 32);
        w.put(0, 1);
        w.put(rng.below(900_000), 24);
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
        for &g in selectors {
            let j = mtf.iter().position(|&x| x == g).unwrap();
            w.put((1 << (j + 1)) - 2, j as u32 + 1);
            mtf.remove(j);
            mtf.insert(0, g);
        }
        for l in lens {
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
        if let Some(syms) = encoded_syms {
            for (i, &s) in syms.iter().enumerate() {
                let t = &tables[selectors[i / GROUP] as usize];
                let l = &lens[selectors[i / GROUP] as usize];
                let at = t.perm[..alpha].iter().position(|&p| p == s).unwrap();
                let zn = l[s as usize] as u32;
                w.put((at as i32 + t.base[zn as usize]) as u64, zn);
            }
        }
        for _ in 0..tail_words {
            w.put(rng.next(), 64);
        }
        (w.out, start, sym_start)
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
        let mut reuse_syms = Vec::new();
        for case in 0..4000 {
            let alpha = if case % 25 == 0 {
                MAX_ALPHA
            } else {
                3 + rng.below(256) as usize
            };
            let n_groups = 2 + rng.below(5) as usize;
            let lens: Vec<Vec<u8>> = (0..n_groups).map(|_| lengths_of(&mut rng, alpha)).collect();
            let tables: Vec<Huff> = lens.iter().map(|l| Huff::new(l)).collect();
            // Exercise both runt (`< PAIR_GATE_SEL`) and full (`>= PAIR_GATE_SEL`) blocks,
            // plus long coalesced same-selector runs crossing `CHUNK_STOP` (`> 10` groups).
            let n_sel = if case % 5 == 0 {
                PAIR_GATE_SEL + rng.below(30) as usize
            } else {
                1 + rng.below(40) as usize
            };
            let mut selectors = Vec::with_capacity(n_sel);
            let mut cur_g = rng.below(n_groups as u64) as u8;
            for _ in 0..n_sel {
                if rng.below(3) == 0 {
                    cur_g = rng.below(n_groups as u64) as u8;
                }
                selectors.push(cur_g);
            }
            let eob = (alpha - 1) as u16;
            let complete = lens
                .iter()
                .all(|l| l.iter().map(|&x| 1u64 << (20 - x)).sum::<u64>() <= 1 << 20);
            let encoded = if complete && rng.below(4) != 0 {
                let m = 1 + rng.below((n_sel * GROUP) as u64) as usize;
                let mut v = Vec::with_capacity(m);
                for i in 0..m {
                    let s = if i + 1 == m {
                        eob
                    } else if alpha == MAX_ALPHA && i % 37 == 0 {
                        256
                    } else {
                        let k = rng.below(9);
                        (rng.below(1 << k) % (alpha as u64 - 1)) as u16
                    };
                    v.push(s);
                }
                Some(v)
            } else {
                None
            };
            let tail_words = rng.below(200);
            let (mut data, start, sym_start) = write_block(
                &mut rng,
                alpha,
                &lens,
                &tables,
                &selectors,
                encoded.as_deref(),
                tail_words,
            );
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
            let got = read_item(&mut a, 9, std::mem::take(&mut reuse_syms));
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
                    reuse_syms = x.syms;
                }
                (Err((k, buf)), Err(k2)) => {
                    assert_eq!(k, k2, "{ctx}");
                    seen[match k {
                        ErrorKind::BadHuffmanCode => 1,
                        ErrorKind::SelectorOverrun => 2,
                        ErrorKind::Truncated => 3,
                        _ => 4,
                    }] += 1;
                    reuse_syms = buf;
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

    #[test]
    fn alpha_258_sym256_eob_truncation_and_overrun() {
        let mut rng = Rng(0xd1b5_4a32_d192_ed03);
        // Group 0: alpha = 258 with short `sym = 256` (len 4 -> SYM256 tag), short `EOB = 257`
        // (len 5 -> EOB_FAST), and `len = 12` codes in SubLut.
        // Group 1: alpha = 258 with `sym = 256` and `EOB = 257` at `len = 12` (SubLut).
        let mut l0 = vec![12u8; MAX_ALPHA];
        l0[0] = 2;
        l0[1] = 2;
        l0[2] = 3;
        l0[3] = 4;
        l0[256] = 4;
        l0[257] = 5;
        // Kraft check: 1/4 + 1/4 + 1/8 + 1/16 + 1/16 + 1/32 = 25/32, remaining 252 * 1/4096 < 7/32.
        let mut l1 = vec![12u8; MAX_ALPHA];
        l1[0] = 2;
        l1[1] = 2;
        l1[2] = 3;
        l1[3] = 3;
        // 256 and 257 stay at len 12 -> decoded through SubLut.
        let lens = vec![l0, l1];
        let tables: Vec<Huff> = lens.iter().map(|l| Huff::new(l)).collect();

        // 1. Long coalesced run (14 identical groups = 700 syms > CHUNK_STOP = 490) + group 1
        //    with `sym = 256` at boundary `k = target - 1` and `EOB = 257` at the end.
        let mut selectors = vec![0u8; 64];
        selectors.extend(std::iter::repeat(1u8).take(14));
        let mut syms = Vec::new();
        for i in 0..749 {
            syms.push(match i % 11 {
                0 => 256,
                1 => 255,
                2 => 0,
                3 => 1,
                4 => 2,
                5 => 3,
                _ => (i % 256) as u16,
            });
        }
        syms.push(256); // index 749 = exact group boundary (15 * 50 - 1)
        for i in 0..100 {
            syms.push(if i % 5 == 0 { 256 } else { (i % 4) as u16 });
        }
        syms.push(257); // EOB in group 1 (len 12 SubLut)
        let (data, start, sym_start) = write_block(
            &mut rng,
            MAX_ALPHA,
            &lens,
            &tables,
            &selectors,
            Some(&syms),
            8,
        );
        let mut a = Bits::at(&data, start);
        let Ok(Item::Block(blk)) = read_item(&mut a, 9, Vec::new()) else {
            panic!("expected valid alpha=258 block");
        };
        let mut b = Bits::at(&data, sym_start);
        let (want, ref_syms) = symbols_in_memory(&mut b, &tables, &selectors, 257);
        assert_eq!(want, Ok(()));
        assert_eq!(blk.syms, syms);
        assert_eq!(blk.syms, ref_syms);
        assert_eq!(a.bit_pos(), b.bit_pos());
        assert!(!a.overran());

        // 2. Truncated mid-run inside the 64-group coalesced run: exact Truncated + bit_pos parity.
        for cut_bytes in [1usize, 5, 19, 63, 128] {
            let cut = (sym_start.div_ceil(8) as usize + cut_bytes).min(data.len());
            let trunc = &data[..cut];
            let mut ta = Bits::at(trunc, start);
            let got = read_item(&mut ta, 9, Vec::new());
            let mut tb = Bits::at(trunc, sym_start);
            let (want_t, ref_t) = symbols_in_memory(&mut tb, &tables, &selectors, 257);
            assert_eq!(ta.bit_pos(), tb.bit_pos(), "cut={cut_bytes}");
            assert_eq!(ta.overran(), tb.overran(), "cut={cut_bytes}");
            match (got, want_t) {
                (Ok(Item::Block(x)), Ok(())) => assert_eq!(x.syms, ref_t),
                (Err((k1, _)), Err(k2)) => assert_eq!(k1, k2, "cut={cut_bytes}"),
                (g, w) => panic!(
                    "cut={cut_bytes}: {:?} vs {w:?}",
                    g.map(|_| ()).map_err(|e| e.0)
                ),
            }
        }

        // 3. SelectorOverrun: valid symbols without EOB across 12 coalesced groups.
        let overrun_sel = vec![0u8; 12];
        let overrun_syms: Vec<u16> = (0..12 * GROUP)
            .map(|i| if i % 7 == 0 { 256 } else { 0 })
            .collect();
        let (odata, ostart, osym_start) = write_block(
            &mut rng,
            MAX_ALPHA,
            &lens,
            &tables,
            &overrun_sel,
            Some(&overrun_syms),
            8,
        );
        let mut oa = Bits::at(&odata, ostart);
        let ogot = read_item(&mut oa, 9, Vec::new());
        let mut ob = Bits::at(&odata, osym_start);
        let (owant, _) = symbols_in_memory(&mut ob, &tables, &overrun_sel, 257);
        assert_eq!(
            ogot.map(|_| ()).map_err(|e| e.0),
            Err(ErrorKind::SelectorOverrun)
        );
        assert_eq!(owant, Err(ErrorKind::SelectorOverrun));
        assert_eq!(oa.bit_pos(), ob.bit_pos());

        // 4. alpha = 257 (`eob = 256`): symbol 256 must map to EOB_FAST, not SYM256.
        let mut l257 = vec![12u8; 257];
        l257[0] = 1;
        l257[256] = 2;
        let t257 = [Huff::new(&l257), Huff::new(&l257)];
        let sel257 = vec![0u8; 2];
        let syms257 = vec![0u16, 0, 256];
        let (d257, st257, ss257) = write_block(
            &mut rng,
            257,
            &[l257.clone(), l257],
            &t257,
            &sel257,
            Some(&syms257),
            4,
        );
        let mut a257 = Bits::at(&d257, st257);
        let Ok(Item::Block(b257)) = read_item(&mut a257, 9, Vec::new()) else {
            panic!("expected valid alpha=257 block");
        };
        let mut ref257 = Bits::at(&d257, ss257);
        let (w257, r257) = symbols_in_memory(&mut ref257, &t257, &sel257, 256);
        assert_eq!(w257, Ok(()));
        assert_eq!(b257.syms, syms257);
        assert_eq!(b257.syms, r257);
        assert_eq!(a257.bit_pos(), ref257.bit_pos());
    }

    #[test]
    fn bmi2_and_scalar_decode_runs_match_exactly() {
        let mut rng = Rng(0x517c_c1b7_2722_0a95);
        let mut sc_a = ScanScratch::default();
        let mut sc_b = ScanScratch::default();
        let mut syms_a = Vec::with_capacity(MAX_SYMS);
        let mut syms_b = Vec::with_capacity(MAX_SYMS);

        for case in 0..500 {
            let alpha = if case % 10 == 0 {
                MAX_ALPHA
            } else {
                3 + rng.below(256) as usize
            };
            let n_groups = 2 + rng.below(5) as usize;
            let build_pairs = case % 3 != 0;
            let lens: Vec<Vec<u8>> = (0..n_groups).map(|_| lengths_of(&mut rng, alpha)).collect();
            for (g, l) in lens.iter().enumerate() {
                sc_a.tables[g].build_into(l, build_pairs);
                sc_b.tables[g].build_into(l, build_pairs);
            }
            let n_sel = 1 + rng.below(80) as usize;
            let mut selectors = Vec::with_capacity(n_sel);
            let mut cur_g = rng.below(n_groups as u64) as u8;
            for _ in 0..n_sel {
                if rng.below(4) == 0 {
                    cur_g = rng.below(n_groups as u64) as u8;
                }
                selectors.push(cur_g);
            }
            let runs = runs_of(&selectors);
            let data: Vec<u8> = (0..(32 + rng.below(600) as usize))
                .map(|_| rng.next() as u8)
                .collect();
            let start_bit = rng.below(16);
            let (_, ra) = Bits::at(&data, start_bit).parts();
            let (_, rb) = Bits::at(&data, start_bit).parts();
            syms_a.clear();
            syms_b.clear();

            let (res_a, ra) =
                decode_runs_scalar(&sc_a.tables, &runs, ra, &data, &mut sc_a.chunk, &mut syms_a);
            #[cfg(target_arch = "x86_64")]
            let (res_b, rb) = if std::is_x86_feature_detected!("bmi2") {
                // SAFETY: guarded immediately by `is_x86_feature_detected!("bmi2")`.
                unsafe {
                    decode_runs_bmi2(&sc_b.tables, &runs, rb, &data, &mut sc_b.chunk, &mut syms_b)
                }
            } else {
                decode_runs_scalar(&sc_b.tables, &runs, rb, &data, &mut sc_b.chunk, &mut syms_b)
            };
            #[cfg(not(target_arch = "x86_64"))]
            let (res_b, rb) =
                decode_runs_scalar(&sc_b.tables, &runs, rb, &data, &mut sc_b.chunk, &mut syms_b);

            assert_eq!(res_a, res_b, "case {case}");
            assert_eq!(ra.bit_pos(), rb.bit_pos(), "case {case}");
            assert_eq!(ra.overran(&data), rb.overran(&data), "case {case}");
            if res_a.is_ok() {
                assert_eq!(syms_a, syms_b, "case {case}");
            }
        }
    }

    #[test]
    fn miri_decode_runs_scalar_exact() {
        let mut rng = Rng(0x1337_beef_cafe_0001);
        let mut sc = ScanScratch::default();
        let mut reuse = Vec::new();
        // Runt block (`n_sel = 2`), then coalesced full-style block (`n_sel = 66`, `alpha = 258`
        // crossing `CHUNK_STOP` flush), then truncated block.
        let mut l0 = vec![12u8; MAX_ALPHA];
        l0[0] = 2;
        l0[1] = 2;
        l0[2] = 3;
        l0[256] = 4;
        l0[257] = 4;
        let lens = vec![l0.clone(), l0];
        let tables: Vec<Huff> = lens.iter().map(|l| Huff::new(l)).collect();
        for &(n_sel, count) in &[(2usize, 12usize), (66usize, 560usize)] {
            let selectors = vec![0u8; n_sel];
            let mut syms: Vec<u16> = (0..count)
                .map(|i| if i % 23 == 0 { 256 } else { (i % 3) as u16 })
                .collect();
            syms.push(257);
            let (data, start, _) = write_block(
                &mut rng,
                MAX_ALPHA,
                &lens,
                &tables,
                &selectors,
                Some(&syms),
                4,
            );
            let mut br = Bits::at(&data, start);
            let Ok(Item::Block(blk)) = read_item_with_scratch(&mut br, 9, reuse, &mut sc) else {
                panic!("expected valid block");
            };
            assert_eq!(blk.syms, syms);
            assert!(!br.overran());
            reuse = blk.syms;
        }
    }
}
