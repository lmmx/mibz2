//! Research only, never used by the decoder: design E, the line table
//! (docs/journal/2026-11-10-by-data-class.md §3; registered in
//! docs/journal/2026-11-10-design-e-registration.md).
//!
//! The inverse BWT walked in the LF direction, i ← LF(i), from a table of one 64-byte line
//! a 32 consecutive rows of the last column L:
//! - the line's 32 bytes of L;
//! - a 32-bit mask of the rows that are the first of their byte in the line;
//! - up to 9 bases of 3 bytes, in first-occurrence order: C(c) plus the copies of c in the
//!   lines before, and the line's count of distinct bytes in its last byte.
//!
//! A step from row i, at offset o in its line, with c = L[i]: `eq` is the line's rows equal
//! to c (one compare a half line, and a mask); c's slot is the first-occurrence bits below
//! `eq`'s lowest bit, counted; LF(i) is the slot's base plus the bits of `eq` below o,
//! counted. A line with more than 9 distinct bytes keeps 8 bases and, in the ninth's place,
//! an index into an overflow list holding the rest: a second access for the rows whose byte
//! is past the eighth.
//!
//! The walk emits the block's bytes last to first. Its segmented form is `bwt::walk_segments`'s
//! in the other direction: pseudo-random rows (and `orig_ptr`) are segment starts, a lane
//! walks LF from a start until the next row is a start, and a segment's bytes, reversed, are
//! the forward text from the start it ran into up to its own. So the forward output is the
//! segments joined from `orig_ptr`'s, each preceded by the segment that ran into it. That
//! holds on every cycle of the permutation, as the forward join's does: exactly
//! `bwt::walk`'s output for any last column whose counts are its own.
//!
//! Soundness of the table is the tiling of the run records' argument
//! (2026-11-10-backward-table.md §1) at line granularity: a byte's bases are its count
//! before each line, so after the last line each byte's counter must stand at its bucket's
//! end. `LineTable::build` checks exactly that, for all 256 bytes, which proves the counts
//! are `ll`'s own and so that LF is a permutation of the rows: every row the walk reaches is
//! below n, which is what its unchecked reads rely on.

use crate::bwt::{prefetch, random_rows, PF_L1};

/// Rows a line
pub const ROWS: usize = 32;
/// Bases held in the line
const INLINE: usize = 9;
/// Blocks shorter than this are walked plainly, as `bwt::MIN_SEGMENTED`
pub const MIN_SEGMENTED: usize = crate::bwt::MIN_SEGMENTED;

/// One line of 32 rows (64 bytes, one cache line)
#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct Line {
    /// the rows' bytes of L (0 past the block's end)
    bytes: [u8; ROWS],
    /// the rows that are the first of their byte in the line
    first: u32,
    /// 9 bases of 3 bytes (bytes 0-26), then the count of distinct bytes (byte 27). With more
    /// than 9 distinct bytes, the ninth base's place holds the overflow index.
    bases: [u8; 28],
}

const _: () = assert!(std::mem::size_of::<Line>() == 64);

impl Line {
    const EMPTY: Line = Line {
        bytes: [0; ROWS],
        first: 0,
        bases: [0; 28],
    };

    /// The 24-bit value in slot `s` (at most 8)
    #[inline(always)]
    fn slot(&self, s: usize) -> u32 {
        debug_assert!(s < INLINE);
        // (one 4-byte load: slot 8's fourth byte is the distinct count, masked off)
        let b: [u8; 4] = self.bases[3 * s..3 * s + 4].try_into().unwrap();
        u32::from_le_bytes(b) & 0xff_ffff
    }

    fn set_slot(&mut self, s: usize, v: u32) {
        debug_assert!(s < INLINE && v < 1 << 24);
        self.bases[3 * s..3 * s + 3].copy_from_slice(&v.to_le_bytes()[..3]);
    }

    /// The line's count of distinct bytes
    #[inline(always)]
    pub fn distinct(&self) -> usize {
        self.bases[27] as usize
    }
}

/// The mask of `bytes` equal to `c`: bit j for byte j
#[inline(always)]
pub fn eq_mask(bytes: &[u8; ROWS], c: u8) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        use core::arch::x86_64::*;
        // SAFETY: SSE2 is in x86_64's baseline; the two unaligned loads read bytes[0..16]
        // and bytes[16..32], inside the array
        unsafe {
            let p = bytes.as_ptr() as *const __m128i;
            let k = _mm_set1_epi8(c as i8);
            let lo = _mm_movemask_epi8(_mm_cmpeq_epi8(_mm_loadu_si128(p), k)) as u32;
            let hi = _mm_movemask_epi8(_mm_cmpeq_epi8(_mm_loadu_si128(p.add(1)), k)) as u32;
            lo | hi << 16
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    eq_mask_swar(bytes, c)
}

/// `eq_mask` by one 32-byte compare (AVX2): only where the caller has AVX2 enabled
///
/// # Safety
/// The CPU must have AVX2.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn eq_mask_avx2(bytes: &[u8; ROWS], c: u8) -> u32 {
    use core::arch::x86_64::*;
    // SAFETY: AVX2 (the caller's); the load reads bytes[0..32], the array
    unsafe {
        let v = _mm256_loadu_si256(bytes.as_ptr() as *const __m256i);
        _mm256_movemask_epi8(_mm256_cmpeq_epi8(v, _mm256_set1_epi8(c as i8))) as u32
    }
}

/// `eq_mask`, or `eq_mask_avx2` when `W` (set only inside a function with AVX2 enabled)
#[inline(always)]
fn eq_mask_w<const W: bool>(bytes: &[u8; ROWS], c: u8) -> u32 {
    #[cfg(target_arch = "x86_64")]
    if W {
        // SAFETY: W is true only on the paths `dispatch` takes when the CPU has AVX2
        return unsafe { eq_mask_avx2(bytes, c) };
    }
    eq_mask(bytes, c)
}

/// The kernel `build`, `walk` and `segments` run: compiled with AVX2, POPCNT and BMI1-2
/// where the CPU has them (chosen at run time, as the decoder's kernels are), else with
/// POPCNT, else for the target's baseline (whose `count_ones` is a dozen instructions).
/// Miri runs the baseline's, the same source but for `eq_mask_avx2`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kernel {
    Avx2,
    Popcnt,
    Baseline,
}

impl Kernel {
    pub fn detect() -> Kernel {
        #[cfg(target_arch = "x86_64")]
        {
            use std::is_x86_feature_detected as has;
            if has!("avx2") && has!("popcnt") && has!("bmi1") && has!("bmi2") {
                return Kernel::Avx2;
            }
            if has!("popcnt") {
                return Kernel::Popcnt;
            }
        }
        Kernel::Baseline
    }

    /// Whether this CPU can run the kernel
    pub fn available(self) -> bool {
        matches!(
            (self, Kernel::detect()),
            (Kernel::Baseline, _) | (_, Kernel::Avx2) | (Kernel::Popcnt, Kernel::Popcnt)
        )
    }

    /// The kernels this CPU can run
    pub fn all() -> Vec<Kernel> {
        [Kernel::Avx2, Kernel::Popcnt, Kernel::Baseline]
            .into_iter()
            .filter(|k| k.available())
            .collect()
    }
}

/// `eq_mask` in plain integer arithmetic (every target; the x86_64 build's check)
#[inline(always)]
pub fn eq_mask_swar(bytes: &[u8; ROWS], c: u8) -> u32 {
    const LOW7: u64 = 0x7f7f_7f7f_7f7f_7f7f;
    let k = u64::from_ne_bytes([c; 8]);
    let mut m = 0u32;
    for w in 0..4 {
        let b = &bytes[8 * w..8 * w + 8];
        let v = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]) ^ k;
        // each byte's high bit set exactly where v's byte is zero (no carry between bytes)
        let z = !(((v & LOW7) + LOW7) | v | LOW7);
        m |= ((z.wrapping_mul(0x0002_0408_1020_4081) >> 56) as u32 & 0xff) << (8 * w);
    }
    m
}

/// The line table of a block: its lines, the overflow list, the block's length
#[derive(Default)]
pub struct LineTable {
    lines: Vec<Line>,
    over: Vec<u32>,
    n: usize,
}

/// The census of a built table (untimed counts, `examples/line_table.rs census`)
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Census {
    /// rows, which are the walk's steps (each row is stepped from once a walk of a block)
    pub rows: u64,
    pub lines: u64,
    /// rows whose step reads its line only
    pub one_access: u64,
    /// rows whose step reads its line and the overflow list
    pub two_access: u64,
    /// lines with at most 9 distinct bytes (no overflow)
    pub lines_inline: u64,
    /// the lines' distinct bytes, summed
    pub distinct: u64,
    /// the table's bytes: 64 a line, 4 an overflow entry
    pub bytes: u64,
}

impl Census {
    pub fn add(&mut self, o: &Census) {
        self.rows += o.rows;
        self.lines += o.lines;
        self.one_access += o.one_access;
        self.two_access += o.two_access;
        self.lines_inline += o.lines_inline;
        self.distinct += o.distinct;
        self.bytes += o.bytes;
    }
}

impl LineTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// The block's length (0 when the last build failed)
    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// The table's bytes: its lines and its overflow list
    pub fn bytes(&self) -> usize {
        self.lines.len() * 64 + self.over.len() * 4
    }

    /// The table of the last column `ll`, whose bytes `counts` counts, in one sequential
    /// pass: a line's bytes copied, then one compare of the line a distinct byte, which
    /// records its base and adds its count in the line to its counter. False (and the table
    /// left empty) when the counts are not `ll`'s own or n is 2^24 or more.
    pub fn build(&mut self, ll: &[u8], counts: &[u32; 256]) -> bool {
        self.build_with(Kernel::detect(), ll, counts)
    }

    /// `build` on the kernel given (which this CPU must be able to run: checked)
    pub fn build_with(&mut self, k: Kernel, ll: &[u8], counts: &[u32; 256]) -> bool {
        assert!(k.available(), "{k:?} on this CPU");
        match k {
            // SAFETY: the CPU has the features (`Kernel::detect`)
            #[cfg(target_arch = "x86_64")]
            Kernel::Avx2 => unsafe { self.build_avx2(ll, counts) },
            // SAFETY: as above
            #[cfg(target_arch = "x86_64")]
            Kernel::Popcnt => unsafe { self.build_popcnt(ll, counts) },
            _ => self.build_impl::<false>(ll, counts),
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,popcnt,bmi1,bmi2")]
    unsafe fn build_avx2(&mut self, ll: &[u8], counts: &[u32; 256]) -> bool {
        self.build_impl::<true>(ll, counts)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "popcnt")]
    unsafe fn build_popcnt(&mut self, ll: &[u8], counts: &[u32; 256]) -> bool {
        self.build_impl::<false>(ll, counts)
    }

    #[inline(always)]
    fn build_impl<const W: bool>(&mut self, ll: &[u8], counts: &[u32; 256]) -> bool {
        let n = ll.len();
        self.lines.clear();
        self.over.clear();
        self.n = 0;
        if n >= 1 << 24 || counts.iter().map(|&c| c as u64).sum::<u64>() != n as u64 {
            return false;
        }
        let (mut next, mut end) = ([0u32; 256], [0u32; 256]);
        let mut cum = 0u32;
        for c in 0..256 {
            next[c] = cum;
            cum += counts[c];
            end[c] = cum;
        }
        self.lines.reserve(n.div_ceil(ROWS));
        let mut slots = [0u32; ROWS];
        for chunk in ll.chunks(ROWS) {
            let mut line = Line::EMPTY;
            line.bytes[..chunk.len()].copy_from_slice(chunk);
            let valid = if chunk.len() == ROWS {
                u32::MAX
            } else {
                (1u32 << chunk.len()) - 1
            };
            let mut rest = valid;
            let mut d = 0;
            while rest != 0 {
                let o = rest.trailing_zeros();
                let c = line.bytes[o as usize];
                let eq = eq_mask_w::<W>(&line.bytes, c) & valid;
                line.first |= 1 << o;
                slots[d] = next[c as usize];
                d += 1;
                // (wrapping: a count past its bucket fails the end check below)
                next[c as usize] = next[c as usize].wrapping_add(eq.count_ones());
                rest &= !eq;
            }
            if d <= INLINE {
                for (s, &v) in slots[..d].iter().enumerate() {
                    line.set_slot(s, v & 0xff_ffff);
                }
            } else {
                for (s, &v) in slots[..INLINE - 1].iter().enumerate() {
                    line.set_slot(s, v & 0xff_ffff);
                }
                line.set_slot(INLINE - 1, self.over.len() as u32);
                self.over.extend_from_slice(&slots[INLINE - 1..d]);
            }
            line.bases[27] = d as u8;
            self.lines.push(line);
        }
        // Every byte's counter at its bucket's end: the counts are ll's own (each byte's
        // copies in ll number exactly its count), so the bases tile [0, n) and LF is a
        // permutation of the rows
        if next != end {
            self.lines.clear();
            self.over.clear();
            return false;
        }
        self.n = n;
        true
    }

    /// One step from row `i` (below n): L[i] and LF(i)
    #[inline(always)]
    fn step<const W: bool>(&self, i: u32) -> (u8, u32) {
        debug_assert!((i as usize) < self.n);
        // SAFETY: i < n (the walk's rows are LF's values, below n once `build` succeeded),
        // and there are n.div_ceil(32) lines
        let line = unsafe { self.lines.get_unchecked(i as usize / ROWS) };
        let o = i % ROWS as u32;
        let c = line.bytes[o as usize];
        let eq = eq_mask_w::<W>(&line.bytes, c);
        let low = eq & eq.wrapping_neg();
        let s = (line.first & (low - 1)).count_ones() as usize;
        let rank = (eq & ((1u32 << o) - 1)).count_ones();
        let base = if s < INLINE - 1 || line.distinct() <= INLINE {
            line.slot(s)
        } else {
            // SAFETY: the line's ninth slot is its index into the overflow list, which holds
            // its distinct bytes past the eighth in order, and s is below its distinct count
            unsafe {
                *self
                    .over
                    .get_unchecked(line.slot(INLINE - 1) as usize + s - (INLINE - 1))
            }
        };
        (c, base + rank)
    }

    /// The census of this table: how its rows' steps are served, and its size
    pub fn census(&self) -> Census {
        let mut k = Census {
            rows: self.n as u64,
            lines: self.lines.len() as u64,
            bytes: self.bytes() as u64,
            ..Census::default()
        };
        for (li, line) in self.lines.iter().enumerate() {
            let d = line.distinct();
            k.distinct += d as u64;
            let len = (self.n - li * ROWS).min(ROWS);
            if d <= INLINE {
                k.lines_inline += 1;
                k.one_access += len as u64;
                continue;
            }
            let valid = if len == ROWS {
                u32::MAX
            } else {
                (1u32 << len) - 1
            };
            let (mut rest, mut s) = (valid, 0);
            while rest != 0 {
                let o = rest.trailing_zeros();
                let eq = eq_mask(&line.bytes, line.bytes[o as usize]) & valid;
                let rows = eq.count_ones() as u64;
                if s < INLINE - 1 {
                    k.one_access += rows;
                } else {
                    k.two_access += rows;
                }
                s += 1;
                rest &= !eq;
            }
        }
        k
    }

    /// The plain walk in the LF direction: `bwt::walk`'s output (the block's RLE1-coded
    /// bytes, first to last), written last to first (`orig_ptr` below n).
    ///
    /// From `orig_ptr` the n steps give the bytes ending at Ψⁿ(orig_ptr)'s, which is
    /// `orig_ptr` itself whenever its cycle's length divides n, as in every block a real
    /// text gives. Otherwise (a crafted or corrupt column) the walk is taken again from
    /// Ψⁿ(orig_ptr), found from the cycle's length.
    pub fn walk(&self, orig_ptr: u32, out: &mut Vec<u8>) {
        self.walk_with(Kernel::detect(), orig_ptr, out)
    }

    /// `walk` on the kernel given (which this CPU must be able to run: checked)
    pub fn walk_with(&self, k: Kernel, orig_ptr: u32, out: &mut Vec<u8>) {
        assert!(k.available(), "{k:?} on this CPU");
        match k {
            // SAFETY: the CPU has the features (`Kernel::detect`)
            #[cfg(target_arch = "x86_64")]
            Kernel::Avx2 => unsafe { self.walk_avx2(orig_ptr, out) },
            // SAFETY: as above
            #[cfg(target_arch = "x86_64")]
            Kernel::Popcnt => unsafe { self.walk_popcnt(orig_ptr, out) },
            _ => self.walk_impl::<false>(orig_ptr, out),
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,popcnt,bmi1,bmi2")]
    unsafe fn walk_avx2(&self, orig_ptr: u32, out: &mut Vec<u8>) {
        self.walk_impl::<true>(orig_ptr, out)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "popcnt")]
    unsafe fn walk_popcnt(&self, orig_ptr: u32, out: &mut Vec<u8>) {
        self.walk_impl::<false>(orig_ptr, out)
    }

    #[inline(always)]
    fn walk_impl<const W: bool>(&self, orig_ptr: u32, out: &mut Vec<u8>) {
        let n = self.n;
        out.clear();
        if n == 0 {
            return;
        }
        assert!((orig_ptr as usize) < n, "orig_ptr below the block's length");
        out.resize(n, 0);
        let mut i = orig_ptr;
        for t in (0..n).rev() {
            let (c, next) = self.step::<W>(i);
            out[t] = c;
            i = next;
        }
        if i == orig_ptr {
            return;
        }
        // The cycle's length m, then Ψⁿ(orig_ptr) = LF^((m - n mod m) mod m)(orig_ptr)
        let mut m = 1;
        i = self.step::<W>(orig_ptr).1;
        while i != orig_ptr {
            i = self.step::<W>(i).1;
            m += 1;
        }
        for _ in 0..(m - n % m) % m {
            i = self.step::<W>(i).1;
        }
        for t in (0..n).rev() {
            let (c, next) = self.step::<W>(i);
            out[t] = c;
            i = next;
        }
    }

    /// The walk split into segments walked `L` lanes at a time, then joined (`PF`: each
    /// lane's next line prefetched as soon as its row is known): exactly `walk`'s output.
    /// Blocks shorter than `MIN_SEGMENTED` are walked plainly.
    pub fn walk_segments<const L: usize, const PF: bool>(
        &self,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        s: &mut Scratch,
        segments: usize,
    ) {
        if self.n < MIN_SEGMENTED {
            self.walk(orig_ptr, out)
        } else {
            self.segments::<L, PF>(orig_ptr, out, s, segments)
        }
    }

    /// The segmented walk at any length (the tests' and Miri's entry: small blocks)
    pub fn segments<const L: usize, const PF: bool>(
        &self,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        s: &mut Scratch,
        segments: usize,
    ) {
        self.segments_with::<L, PF>(Kernel::detect(), orig_ptr, out, s, segments)
    }

    /// `segments` on the kernel given (which this CPU must be able to run: checked)
    pub fn segments_with<const L: usize, const PF: bool>(
        &self,
        k: Kernel,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        s: &mut Scratch,
        segments: usize,
    ) {
        assert!(k.available(), "{k:?} on this CPU");
        match k {
            // SAFETY: the CPU has the features (`Kernel::detect`)
            #[cfg(target_arch = "x86_64")]
            Kernel::Avx2 => unsafe { self.segments_avx2::<L, PF>(orig_ptr, out, s, segments) },
            // SAFETY: as above
            #[cfg(target_arch = "x86_64")]
            Kernel::Popcnt => unsafe { self.segments_popcnt::<L, PF>(orig_ptr, out, s, segments) },
            _ => self.segments_impl::<L, PF, false>(orig_ptr, out, s, segments),
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,popcnt,bmi1,bmi2")]
    unsafe fn segments_avx2<const L: usize, const PF: bool>(
        &self,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        s: &mut Scratch,
        segments: usize,
    ) {
        self.segments_impl::<L, PF, true>(orig_ptr, out, s, segments)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "popcnt")]
    unsafe fn segments_popcnt<const L: usize, const PF: bool>(
        &self,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        s: &mut Scratch,
        segments: usize,
    ) {
        self.segments_impl::<L, PF, false>(orig_ptr, out, s, segments)
    }

    #[inline(always)]
    fn segments_impl<const L: usize, const PF: bool, const W: bool>(
        &self,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        s: &mut Scratch,
        segments: usize,
    ) {
        let n = self.n;
        out.clear();
        if n == 0 {
            return;
        }
        assert!((orig_ptr as usize) < n, "orig_ptr below the block's length");
        assert!(segments >= 1);
        random_rows(n, segments - 1, &mut s.starts);
        s.starts.push(orig_ptr);
        s.starts.sort_unstable();
        s.starts.dedup();
        let starts = &s.starts;
        let ns = starts.len();
        s.bitmap.clear();
        s.bitmap.resize(n.div_ceil(64), 0);
        for &r in starts {
            s.bitmap[(r >> 6) as usize] |= 1 << (r & 63);
        }
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        // pred[e]: the segment that ran into start e (each start on a cycle is reached by
        // exactly one segment: its own when it is its cycle's only start)
        s.pred.clear();
        s.pred.resize(ns, usize::MAX);
        let bm = &s.bitmap;
        let bufs = &mut s.bufs;
        let pred = &mut s.pred;
        // SAFETY (both loops' bitmap reads): a row is below n (LF is a permutation of the
        // rows: `build`), and the bitmap has n.div_ceil(64) words
        let is_start =
            |row: u32| unsafe { *bm.get_unchecked((row >> 6) as usize) } >> (row & 63) & 1 != 0;
        if PF {
            self.lanes_flagged::<L, W>(starts, &is_start, bufs, pred);
        } else {
            self.lanes_bitmap::<L, W>(starts, &is_start, bufs, pred);
        }
        // The forward text from orig_ptr: each segment reversed, from the one that ran into
        // orig_ptr's start, then the one that ran into that segment's start, and so on
        out.reserve(n);
        let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let mut at = seg_of(orig_ptr);
        while out.len() < n {
            let seg = pred[at];
            let b = &bufs[seg];
            let take = b.len().min(n - out.len());
            let from = out.len();
            out.extend_from_slice(&b[b.len() - take..]);
            out[from..].reverse();
            at = seg;
        }
    }

    /// `bwt::walk_segments`'s lanes: each lane's segment and row in arrays, every lane
    /// tested for idleness each step, its bytes pushed to its segment's buffer
    #[inline(always)]
    fn lanes_bitmap<const L: usize, const W: bool>(
        &self,
        starts: &[u32],
        is_start: &impl Fn(u32) -> bool,
        bufs: &mut [Vec<u8>],
        pred: &mut [usize],
    ) {
        let ns = starts.len();
        let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let mut lane_seg = [usize::MAX; L];
        let mut lane_row = [0u32; L];
        let mut queued = 0;
        for k in 0..L {
            if queued < ns {
                lane_seg[k] = queued;
                lane_row[k] = starts[queued];
                queued += 1;
            }
        }
        let mut active = lane_seg.iter().filter(|&&x| x != usize::MAX).count();
        while active > 0 {
            for k in 0..L {
                let seg = lane_seg[k];
                if seg == usize::MAX {
                    continue;
                }
                let (c, row) = self.step::<W>(lane_row[k]);
                bufs[seg].push(c);
                lane_row[k] = row;
                if is_start(row) {
                    pred[seg_of(row)] = seg;
                    if queued < ns {
                        lane_seg[k] = queued;
                        lane_row[k] = starts[queued];
                        queued += 1;
                    } else {
                        lane_seg[k] = usize::MAX;
                        active -= 1;
                    }
                }
            }
        }
    }

    /// `bwt::walk_segments_flagged`'s lanes: each lane's next line prefetched as soon as
    /// its row is known, its bytes written through a pointer into its buffer's spare
    /// capacity (grown when full), and no test for idle lanes until the last segments are
    /// taken. (The starts stay in the bitmap: E's lines have no bit a row to flag them.)
    #[inline(always)]
    fn lanes_flagged<const L: usize, const W: bool>(
        &self,
        starts: &[u32],
        is_start: &impl Fn(u32) -> bool,
        bufs: &mut [Vec<u8>],
        pred: &mut [usize],
    ) {
        #[derive(Clone, Copy)]
        struct Lane {
            row: u32,
            seg: usize,
            p: *mut u8,
            end: *mut u8,
        }
        let ns = starts.len();
        let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let lines = self.lines.as_ptr();
        let take = |seg: usize, bufs: &mut [Vec<u8>]| -> Lane {
            let b = &mut bufs[seg];
            debug_assert!(b.is_empty() && b.capacity() > 0);
            let base = b.as_mut_ptr();
            prefetch::<PF_L1>(
                lines.wrapping_add(starts[seg] as usize / ROWS) as *const u32,
                0,
            );
            Lane {
                row: starts[seg],
                seg,
                p: base,
                // SAFETY: within (one past) the buffer's allocation
                end: unsafe { base.add(b.capacity()) },
            }
        };
        // A lane's segment ends: its length set, and the start it ran into recorded
        let finish = |l: &Lane, bufs: &mut [Vec<u8>], pred: &mut [usize]| {
            let b = &mut bufs[l.seg];
            // SAFETY: p is within b's allocation, every byte below it written
            unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
            pred[seg_of(l.row)] = l.seg;
        };
        // A lane's buffer is full: it grows, its written bytes kept
        let grow = |l: &mut Lane, bufs: &mut [Vec<u8>]| {
            #[cfg(test)]
            GROWS.with(|g| g.set(g.get() + 1));
            let b = &mut bufs[l.seg];
            // SAFETY: p is within b's allocation, every byte below it written; after the
            // reserve, p and end are taken again from the (possibly moved) allocation
            unsafe {
                let len = l.p.offset_from(b.as_ptr()) as usize;
                b.set_len(len);
                b.reserve(b.capacity());
                l.p = b.as_mut_ptr().add(len);
                l.end = b.as_mut_ptr().add(b.capacity());
            }
        };
        // One step of a lane: its byte written, its next row, and whether that row is a
        // start (its segment over)
        let step = |l: &mut Lane, bufs: &mut [Vec<u8>]| -> bool {
            let (c, row) = self.step::<W>(l.row);
            l.row = row;
            prefetch::<PF_L1>(lines.wrapping_add(row as usize / ROWS) as *const u32, 0);
            // SAFETY: p < end, within the buffer's allocation
            unsafe {
                *l.p = c;
                l.p = l.p.add(1);
            }
            if l.p == l.end {
                grow(l, bufs);
            }
            is_start(row)
        };
        let mut lanes = [Lane {
            row: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        }; L];
        let (mut queued, mut active) = (0, 0);
        while active < L && queued < ns {
            lanes[active] = take(queued, bufs);
            queued += 1;
            active += 1;
        }
        let mut done = [false; L];
        // Every lane busy while segments remain to be taken
        if active == L {
            'full: loop {
                for (k, l) in lanes.iter_mut().enumerate() {
                    if step(l, bufs) {
                        finish(l, bufs, pred);
                        if queued == ns {
                            done[k] = true;
                            break 'full;
                        }
                        *l = take(queued, bufs);
                        queued += 1;
                    }
                }
            }
        }
        // The tail: the lanes still walking, each dropped as its segment ends
        let mut live: Vec<Lane> = (0..active)
            .filter(|&k| !done[k])
            .map(|k| lanes[k])
            .collect();
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                if step(&mut live[i], bufs) {
                    finish(&live[i], bufs, pred);
                    live.swap_remove(i);
                    continue;
                }
                i += 1;
            }
        }
    }
}

#[cfg(test)]
thread_local! {
    /// the flagged lanes' buffers grown (test builds only, so a test can show it reached them)
    static GROWS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The segmented walk's buffers, kept from block to block
#[derive(Default)]
pub struct Scratch {
    starts: Vec<u32>,
    bitmap: Vec<u64>,
    bufs: Vec<Vec<u8>>,
    pred: Vec<usize>,
}

/// The segment counts and lanes the research rows use: `bwt::walk_segments`'s shape (12
/// lanes, 256 segments, no prefetch) and the flagged walk's (32 lanes, 512 segments, each
/// lane's next line prefetched)
pub const BITMAP_SHAPE: (usize, usize) = (12, 256);
pub const FLAGGED_SHAPE: (usize, usize) = (32, 512);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bwt::{build, walk, TVector};

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

    fn counts(ll: &[u8]) -> [u32; 256] {
        let mut c = [0u32; 256];
        for &b in ll {
            c[b as usize] += 1;
        }
        c
    }

    /// E's walks against `bwt::walk` on one column, every row as `orig_ptr` up to `rows`
    fn check(ll: &[u8], origs: &[u32], segments: &[usize]) {
        let k = counts(ll);
        let mut tt = TVector::new();
        build(ll, &k, &mut tt);
        let (mut want, mut got, mut s) = (Vec::new(), Vec::new(), Scratch::default());
        let mut tables = Vec::new();
        for kernel in Kernel::all() {
            let mut e = LineTable::new();
            assert!(e.build_with(kernel, ll, &k));
            for &p in origs {
                walk(&tt, p, &mut want);
                e.walk_with(kernel, p, &mut got);
                assert_eq!(got, want, "{kernel:?} plain, n {} orig {p}", ll.len());
                for &m in segments {
                    e.segments_with::<3, false>(kernel, p, &mut got, &mut s, m);
                    assert_eq!(
                        got,
                        want,
                        "{kernel:?} 3 lanes, {m} segments, n {} orig {p}",
                        ll.len()
                    );
                    e.segments_with::<4, true>(kernel, p, &mut got, &mut s, m);
                    assert_eq!(
                        got,
                        want,
                        "{kernel:?} 4 lanes, prefetch, {m} segments, n {} orig {p}",
                        ll.len()
                    );
                }
            }
            let lines: Vec<_> = e
                .lines
                .iter()
                .map(|l| (l.bytes, l.first, l.bases))
                .collect();
            tables.push((lines, e.over));
        }
        assert!(
            tables.windows(2).all(|w| w[0] == w[1]),
            "every kernel builds the same table"
        );
    }

    #[test]
    fn eq_mask_agrees() {
        let mut r = Rng(0x1234_5678_9abc_def1);
        for _ in 0..2000 {
            let mut b = [0u8; ROWS];
            let alphabet = 1 + r.below(256) as u8 as u64;
            for x in &mut b {
                *x = r.below(alphabet) as u8 ^ (0x80 * (r.below(2) as u8));
            }
            let c = b[r.below(ROWS as u64) as usize];
            let want = (0..ROWS).fold(0u32, |m, j| m | ((b[j] == c) as u32) << j);
            assert_eq!(eq_mask(&b, c), want);
            assert_eq!(eq_mask_swar(&b, c), want);
            #[cfg(target_arch = "x86_64")]
            if Kernel::Avx2.available() {
                // SAFETY: the CPU has AVX2
                assert_eq!(unsafe { eq_mask_avx2(&b, c) }, want);
            }
            assert_eq!(
                eq_mask_swar(&b, c.wrapping_add(1)) & !eq_mask(&b, c.wrapping_add(1)),
                0
            );
        }
    }

    #[test]
    fn refuses_counts_not_its_own() {
        let ll = b"banana".to_vec();
        let mut k = counts(&ll);
        let mut e = LineTable::new();
        assert!(e.build(&ll, &k));
        k[b'a' as usize] -= 1;
        k[b'b' as usize] += 1;
        assert!(!e.build(&ll, &k));
        assert!(e.is_empty());
        k[b'b' as usize] += 1;
        assert!(!e.build(&ll, &k), "counts that don't sum to n");
    }

    /// For Miri (`cargo +nightly miri test --lib -- miri_`): the build and both walks'
    /// unchecked reads on small columns: few and many distinct bytes a line (the overflow
    /// list), a partial last line, several cycles (a string repeated) and a column whose
    /// `orig_ptr` cycle does not divide n (the plain walk's second pass)
    #[test]
    fn miri_line_table_walks() {
        let mut r = Rng(0x9e37_79b9_7f4a_7c15);
        let mut cols: Vec<Vec<u8>> = vec![
            vec![7],
            b"ab".to_vec(),
            b"banana".to_vec(),
            b"abababab".to_vec(), // several cycles
            b"aba".to_vec(),      // orig_ptr 1's cycle {1, 2} doesn't divide n = 3
        ];
        for (n, alphabet) in [(31, 3), (33, 40), (64, 256), (200, 12), (517, 2), (700, 64)] {
            cols.push((0..n).map(|_| r.below(alphabet) as u8).collect());
        }
        // every byte distinct in a line, then 9, 10 and 17 distinct
        cols.push((0..96u32).map(|i| (i * 37 % 256) as u8).collect());
        for d in [9u64, 10, 17] {
            cols.push((0..160).map(|i| (i % d) as u8 * 11).collect());
        }
        for ll in &cols {
            let n = ll.len() as u32;
            let origs: Vec<u32> = [0, n / 2, n - 1].into_iter().collect();
            check(ll, &origs, &[1, 2, 5]);
        }
    }

    /// For Miri: the flagged lanes' buffers grown mid-segment (a segment longer than its
    /// buffer's 2n/segments + 64 bytes), their bytes kept across the move
    #[test]
    fn miri_line_table_grows() {
        let mut r = Rng(0x0123_4567_89ab_cdef);
        let ll: Vec<u8> = (0..20_000).map(|_| r.below(4) as u8).collect();
        let k = counts(&ll);
        let mut tt = TVector::new();
        build(&ll, &k, &mut tt);
        let mut e = LineTable::new();
        assert!(e.build(&ll, &k));
        let (mut want, mut got, mut s) = (Vec::new(), Vec::new(), Scratch::default());
        walk(&tt, 7, &mut want);
        let before = GROWS.with(|g| g.get());
        e.segments::<4, true>(7, &mut got, &mut s, 20);
        assert_eq!(got, want);
        assert!(GROWS.with(|g| g.get()) > before, "no buffer grew");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn random_columns_every_orig() {
        let mut r = Rng(0xdead_beef_cafe_f00d);
        for round in 0..300 {
            let n = 1 + r.below(if round % 10 == 0 { 5000 } else { 300 }) as usize;
            let alphabet = [1, 2, 3, 8, 9, 10, 30, 256][round % 8];
            let ll: Vec<u8> = (0..n).map(|_| r.below(alphabet) as u8).collect();
            let origs: Vec<u32> = (0..8).map(|_| r.below(n as u64) as u32).collect();
            check(&ll, &origs, &[1, 3, 17, 64]);
        }
    }

    #[test]
    fn census_counts_rows() {
        // 10 distinct bytes in one full line: the 9th and 10th (by first occurrence) overflow
        let ll: Vec<u8> = (0..32u8).map(|i| i % 10).collect();
        let mut e = LineTable::new();
        assert!(e.build(&ll, &counts(&ll)));
        let k = e.census();
        assert_eq!(k.rows, 32);
        assert_eq!(k.two_access, 3 + 3); // bytes 8 and 9: rows 8, 18, 28 and 9, 19, 29
        assert_eq!(k.one_access, 26);
        assert_eq!(k.lines_inline, 0);
        assert_eq!(k.bytes, 64 + 4 * 2);
    }
}
