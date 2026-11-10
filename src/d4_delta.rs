//! Design D at k = 4 in the Optimiser's delta packing ("D4-delta"): a counting probe, research
//! only. The decoder never calls it. `docs/journal/2026-11-10-d4-delta-count-registration.md`.
//!
//! **The walk is backward.** From row `orig_ptr`, i ← LF(i) emits L[i] as the block's bytes last
//! to first (`docs/journal/2026-11-10-backward-table.md` §1). LF⁴ is a translation on each
//! maximal interval of rows whose four bytes L[i], L[LF i], L[LF² i], L[LF³ i] agree: a piece.
//!
//! **A piece's record,** one `u64`:
//! - bits 0–19: delta = (LFʲ(first) − first) mod 2²⁰, j its length, so that the next row is
//!   (i + record) & (2²⁰ − 1) for every row i of the piece;
//! - bits 20–51: the four bytes, in the order the walk emits them;
//! - bits 52–54: j, the bytes the record emits (1 to 4);
//! - bit 55 (`SHORT`): j < 4, the row's path reaches a stop before four steps;
//! - bit 56 (`START`): the row is a segment's start.
//!
//! **The directory,** 12 bytes a 64 rows: a word with a bit at each record's first row, and
//! the count of records before the word, less one. Row i's record is
//! `rank[i >> 6] + popcnt(bits[i >> 6] << (63 − (i & 63)))`.
//!
//! **The build** (`D4Table::build`): the runs of the column (level 1), then LF² by composing
//! level 1 with itself (one monotone cursor a byte), then LF⁴ by composing level 2 with itself
//! (the first covering piece found through a directory over level 2). The level-4 pieces go
//! straight to their records, split where a singleton falls.
//!
//! **Stops.** Y is `segments` − 1 pseudo-random rows and `orig_ptr`. The proper stops are
//! M = LF³(Y); rows y, LF y, LF² y become singletons that stop at the first proper stop their
//! path reaches, so every walk lands on a proper stop exactly. The segments start at M, each a
//! singleton flagged `START`, and a segment is the walk from its start to the next. The output
//! is `orig_ptr`'s first three bytes, then the segments from z = LF³(`orig_ptr`)'s round its
//! cycle, from the offset Ψⁿ(`orig_ptr`) gives (0 when the cycle's length divides n, as in
//! every block a text gives), n bytes in all, reversed.

use crate::bwt::{random_rows, MIN_SEGMENTED};

/// A row's delta field, and rows' range: bzip2's blocks are at most 900,000 rows
const MASK: u32 = (1 << 20) - 1;
const LEN_SHIFT: u32 = 52;
const SHORT: u64 = 1 << 55;
const START: u64 = 1 << 56;
const STOP: u64 = SHORT | START;
/// The room walk's lanes and segments, as production's (`bwt::FLAGGED_LANES`, `_SEGMENTS`)
pub const LANES: usize = 32;
pub const SEGMENTS: usize = 512;
/// Offsets a lane stages in the tile before its bytes are copied out: 4 bytes an offset, so a
/// lane's tile row is 256 bytes and the tile 8 KiB at 32 lanes, as production's
const TO: usize = 64;

/// A level-1 piece (a run of the column): its first row, its LF base, its byte
#[derive(Clone, Copy, Default)]
struct P1 {
    s: u32,
    b: u32,
    c: u32,
}

/// A level-2 piece: its first row, its LF² base, its LF base, its two bytes
#[derive(Clone, Copy, Default)]
struct P2 {
    s: u32,
    b: u32,
    m: u32,
    w: u32,
}

/// A singleton: its row, its length (0: the piece's four steps), its delta, its flags
#[derive(Clone, Copy)]
struct Single {
    row: u32,
    len: u32,
    delta: u32,
    start: bool,
}

/// The build's scratch, kept from block to block
#[derive(Default)]
struct Scratch {
    l1: Vec<P1>,
    l2: Vec<P2>,
    bits2: Vec<u64>,
    rank2: Vec<u32>,
    ys: Vec<u32>,
    chains: Vec<[u32; 4]>,
    single: Vec<Single>,
}

/// The table's sizes, for the census
#[derive(Clone, Copy, Debug, Default)]
pub struct Census {
    pub rows: u64,
    /// level-1, level-2 and level-4 pieces (r1, r2, r4: before the splits)
    pub r1: u64,
    pub r2: u64,
    pub r4: u64,
    /// records, splits included
    pub records: u64,
    /// singletons (stops' predecessors and starts) and starts
    pub singles: u64,
    pub starts: u64,
    /// the directory's bytes (12 a 64 rows)
    pub dir_bytes: u64,
    /// the split reserve's bytes: 2 records for each of 3 (segments) singletons and
    /// (segments + 1) starts
    pub reserve_bytes: u64,
    /// the doubling scratch's allocated bytes: level 1, level 2, its directory
    pub scratch_bytes: u64,
    /// the records' allocated bytes
    pub records_alloc: u64,
}

impl Census {
    /// The admitted capacity: level-4 pieces' records, the directory, the split reserve
    pub fn capacity(&self) -> u64 {
        8 * self.r4 + self.dir_bytes + self.reserve_bytes
    }
    /// The bytes the table holds: records (splits included) and directory
    pub fn used(&self) -> u64 {
        8 * self.records + self.dir_bytes
    }
}

/// A block's D4-delta table
#[derive(Default)]
pub struct D4Table {
    n: usize,
    orig: u32,
    /// LF³(orig_ptr), a proper stop: the walk emits orig_ptr's first three bytes, then walks
    /// from here
    z: u32,
    recs: Vec<u64>,
    bits: Vec<u64>,
    rank: Vec<u32>,
    /// the segments' starts, ascending (empty: built for the plain walk only)
    starts: Vec<u32>,
    census: Census,
    s: Scratch,
}

/// The walks' buffers, kept from block to block
#[derive(Default)]
pub struct WalkScratch {
    bufs: Vec<Vec<u8>>,
    next: Vec<usize>,
    plain: Vec<u8>,
}

/// One lane: the row it reads next, its segment, where its bytes go (p up to end)
#[derive(Clone, Copy)]
struct Lane {
    row: u32,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

/// popcnt and BMI2 on this CPU (not under Miri, which runs the portable code)
fn fast() -> bool {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    {
        std::is_x86_feature_detected!("popcnt") && std::is_x86_feature_detected!("bmi2")
    }
    #[cfg(not(all(target_arch = "x86_64", not(miri))))]
    {
        false
    }
}

/// The index of the piece holding row i, in a directory of `bits` and `rank`.
///
/// # Safety
/// i >> 6 is below both arrays' lengths
#[inline(always)]
unsafe fn piece(bits: *const u64, rank: *const u32, i: u32) -> usize {
    let d = (i >> 6) as usize;
    let w = *bits.add(d);
    let r = *rank.add(d);
    r.wrapping_add((w << (63 - (i & 63))).count_ones()) as usize
}

/// `src` reversed into `dst`, both `len` bytes
///
/// # Safety
/// src..src + len readable, dst..dst + len writable, not overlapping
#[inline(always)]
unsafe fn copy_reversed(src: *const u8, dst: *mut u8, len: usize) {
    let mut k = 0;
    while k + 8 <= len {
        let v = (src.add(k) as *const u64).read_unaligned();
        (dst.add(len - k - 8) as *mut u64).write_unaligned(v.swap_bytes());
        k += 8;
    }
    while k < len {
        *dst.add(len - 1 - k) = *src.add(k);
        k += 1;
    }
}

/// Prefetch the line holding `p` into L1 (a hint: it changes no value read)
#[inline(always)]
fn prefetch(p: *const u64) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: SSE is in x86_64's baseline; a prefetch never faults
    unsafe {
        use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
        _mm_prefetch::<_MM_HINT_T0>(p as *const i8)
    };
    #[cfg(not(target_arch = "x86_64"))]
    let _ = p;
}

impl D4Table {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// The last build's sizes
    pub fn census(&self) -> Census {
        self.census
    }

    /// The table of the column `ll`, whose bytes `counts` counts, walked from `orig_ptr`, with
    /// `segments` segments (0 or 1: the one stop LF³(orig_ptr), as the plain walk needs). Refused (false, and an empty table)
    /// unless the counts are `ll`'s own, the block has fewer than 2²⁰ rows and `orig_ptr` is
    /// one of them.
    pub fn build(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        orig_ptr: u32,
        segments: usize,
    ) -> bool {
        if fast() {
            #[cfg(target_arch = "x86_64")]
            // SAFETY: the CPU has popcnt and BMI2 (`fast`)
            return unsafe { self.build_fast(ll, counts, orig_ptr, segments) };
        }
        self.build_impl(ll, counts, orig_ptr, segments)
    }

    /// `build` with production's choice of segments: `SEGMENTS` for a block the room walk
    /// segments, one below `MIN_SEGMENTED`
    pub fn build_for_walk(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32) -> bool {
        let segments = if ll.len() < MIN_SEGMENTED {
            1
        } else {
            SEGMENTS
        };
        self.build(ll, counts, orig_ptr, segments)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "popcnt,bmi1,bmi2")]
    unsafe fn build_fast(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        orig: u32,
        segs: usize,
    ) -> bool {
        self.build_impl(ll, counts, orig, segs)
    }

    #[inline(always)]
    fn build_impl(&mut self, ll: &[u8], counts: &[u32; 256], orig: u32, segments: usize) -> bool {
        let n = ll.len();
        self.n = 0;
        self.recs.clear();
        self.starts.clear();
        self.census = Census::default();
        if n == 0
            || n > MASK as usize
            || orig as usize >= n
            || counts.iter().map(|&c| c as u64).sum::<u64>() != n as u64
        {
            return false;
        }
        let (mut next, mut end) = ([0u32; 256], [0u32; 256]);
        let mut cum = 0u32;
        for c in 0..256 {
            next[c] = cum;
            cum += counts[c];
            end[c] = cum;
        }
        let s = &mut self.s;

        // 1. The runs: level 1. A run's base is its byte's counter when it starts; the
        // counter moves on by the run's length when it ends.
        let l1 = &mut s.l1;
        l1.clear();
        l1.reserve(1024);
        let c0 = ll[0];
        l1.push(P1 {
            s: 0,
            b: next[c0 as usize],
            c: c0 as u32,
        });
        let (mut open_s, mut open_c) = (0u32, c0 as usize);
        let mut boundary = |j: u32, c: u8, l1: &mut Vec<P1>| {
            next[open_c] += j - open_s;
            let c = c as usize;
            // SAFETY: capacity for this push was reserved by the caller
            unsafe {
                let len = l1.len();
                l1.as_mut_ptr().add(len).write(P1 {
                    s: j,
                    b: next[c],
                    c: c as u32,
                });
                l1.set_len(len + 1);
            }
            open_s = j;
            open_c = c;
        };
        let mut i = 1;
        while i + 8 <= n {
            let a = u64::from_le_bytes(ll[i..i + 8].try_into().unwrap());
            let b = u64::from_le_bytes(ll[i - 1..i + 7].try_into().unwrap());
            let mut x = a ^ b;
            if x != 0 {
                l1.reserve(8);
                while x != 0 {
                    let t = x.trailing_zeros() & !7;
                    boundary(i as u32 + t / 8, (a >> t) as u8, l1);
                    x &= !(0xffu64 << t);
                }
            }
            i += 8;
        }
        while i < n {
            if ll[i] != ll[i - 1] {
                l1.reserve(1);
                boundary(i as u32, ll[i], l1);
            }
            i += 1;
        }
        next[open_c] += n as u32 - open_s;
        // Validation: every byte's runs tile its bucket exactly, so the bases are LF's and
        // every image interval below lies in [0, n)
        if next != end {
            return false;
        }
        let r1 = l1.len();
        l1.push(P1 {
            s: n as u32,
            b: 0,
            c: 0,
        });

        // 2. Level 2 = level 1 ∘ level 1. A run's image [b, b + len) lies in its byte's bucket,
        // and its byte's runs' images follow each other up the bucket, so one cursor a byte
        // finds the runs that cover it.
        let l1 = &s.l1[..];
        let mut cur = [0u32; 256];
        {
            let mut k = 0usize;
            let mut lo = 0u32;
            for c in 0..256 {
                if counts[c] != 0 {
                    while l1[k + 1].s <= lo {
                        k += 1;
                    }
                    cur[c] = k as u32;
                }
                lo += counts[c];
            }
        }
        let l2 = &mut s.l2;
        l2.clear();
        l2.reserve(2 * r1 + 1);
        let p2 = l2.as_mut_ptr();
        let mut r2 = 0usize;
        let q1 = l1.as_ptr();
        for j in 0..r1 {
            // SAFETY: j + 1 <= r1, inside l1 (its sentinel at r1). The cursor q indexes the
            // run holding row `at` of the image, which lies in [0, n) (validated above), so
            // q < r1 and q + 1 <= r1. At most 2 r1 pieces are written: each is a run, or
            // starts at a run's start inside an image.
            unsafe {
                let p = *q1.add(j);
                let len = (*q1.add(j + 1)).s - p.s;
                let (b, e) = (p.b, p.b + len);
                let mut q = *cur.get_unchecked(p.c as usize) as usize;
                let mut at = b;
                loop {
                    let qq = *q1.add(q);
                    let qe = (*q1.add(q + 1)).s;
                    let stop = qe.min(e);
                    p2.add(r2).write(P2 {
                        s: p.s + (at - b),
                        b: qq.b + (at - qq.s),
                        m: at,
                        w: p.c | qq.c << 8,
                    });
                    r2 += 1;
                    if stop == qe {
                        q += 1;
                    }
                    if stop == e {
                        break;
                    }
                    at = stop;
                }
                *cur.get_unchecked_mut(p.c as usize) = q as u32;
            }
        }
        // SAFETY: r2 <= 2 r1 pieces written, below the reserved 2 r1 + 1
        unsafe {
            p2.add(r2).write(P2 {
                s: n as u32,
                ..P2::default()
            });
            l2.set_len(r2 + 1);
        }

        // 3. A directory over level 2
        let words = n.div_ceil(64);
        let l2 = &s.l2[..];
        dir(
            &mut s.bits2,
            &mut s.rank2,
            words,
            l2[..r2].iter().map(|p| p.s),
        );
        let (b2p, r2p, q2) = (s.bits2.as_ptr(), s.rank2.as_ptr(), l2.as_ptr());
        // LF and LF² of a row, through level 2
        // SAFETY: z < n, so z >> 6 < words, and the piece found holds z
        let lf12 = |z: u32| unsafe {
            let q = *q2.add(piece(b2p, r2p, z));
            (q.m + (z - q.s), q.b + (z - q.s))
        };

        // 4. The stops, their chains and the singletons. Y is orig_ptr and segments - 1
        // pseudo-random rows; the proper stops are M = LF³(Y), the segments' starts
        s.single.clear();
        self.bits.clear();
        self.bits.resize(words, 0);
        random_rows(n, segments.saturating_sub(1), &mut s.ys);
        s.ys.push(orig);
        s.ys.sort_unstable();
        s.ys.dedup();
        s.chains.clear();
        for &y in &s.ys {
            let (a, b) = lf12(y);
            let (c, _) = lf12(b);
            s.chains.push([y, a, b, c]);
        }
        // the proper stops, marked in `bits` for now
        for ch in &s.chains {
            self.bits[(ch[3] >> 6) as usize] |= 1 << (ch[3] & 63);
            if ch[0] == orig {
                self.z = ch[3];
            }
        }
        let stop = |bits: &[u64], r: u32| bits[(r >> 6) as usize] >> (r & 63) & 1 != 0;
        for ch in &s.chains {
            for i in 0..3 {
                // the first proper stop on the row's path (the chain's own end at the latest)
                let j = (1..=3 - i).find(|&j| stop(&self.bits, ch[i + j])).unwrap();
                s.single.push(Single {
                    row: ch[i],
                    len: j as u32,
                    delta: ch[i + j].wrapping_sub(ch[i]) & MASK,
                    start: false,
                });
            }
        }
        for ch in &s.chains {
            self.bits[(ch[3] >> 6) as usize] = 0;
            self.starts.push(ch[3]);
        }
        self.starts.sort_unstable();
        self.starts.dedup();
        for &m in &self.starts {
            s.single.push(Single {
                row: m,
                len: 0,
                delta: 0,
                start: true,
            });
        }
        // one entry a row: a start's flag joins its row's shortened length, if any (a row's
        // first stop is the same from every chain it is on: their paths from it are LF's)
        s.single.sort_unstable_by_key(|x| (x.row, x.start));
        s.single.dedup_by(|b, a| {
            if a.row == b.row {
                a.start |= b.start;
                if a.len == 0 {
                    (a.len, a.delta) = (b.len, b.delta);
                }
                true
            } else {
                false
            }
        });
        let nsingle = s.single.len();
        s.single.push(Single {
            row: u32::MAX,
            len: 0,
            delta: 0,
            start: false,
        });

        // 5. Level 4 = level 2 ∘ level 2, written as records, split at the singletons
        self.recs.reserve(2 * r2 + 2 * nsingle + 1);
        let rp = self.recs.as_mut_ptr();
        let bits = self.bits.as_mut_ptr();
        let mut nr = 0usize;
        let mut r4 = 0usize;
        let sp = s.single.as_ptr();
        let mut si = 0usize;
        // SAFETY (for the closures and the loop): rows are below n, so a row >> 6 is below
        // `words`; at most 2 r2 pieces, each split into at most 3 records by a singleton
        // (2 extra a singleton), so nr stays below the reserved 2 r2 + 2 nsingle + 1; the
        // singletons' list ends with a sentinel above every row
        let put = |row: u32, rec: u64, nr: &mut usize| unsafe {
            rp.add(*nr).write(rec);
            *nr += 1;
            *bits.add((row >> 6) as usize) |= 1 << (row & 63);
        };
        let full = 4u64 << LEN_SHIFT;
        for j in 0..r2 {
            // SAFETY: j + 1 <= r2 (the sentinel); the image [b, e) lies in [0, n), so the
            // piece q holding `at` has q < r2 (as in step 2)
            unsafe {
                let p = *q2.add(j);
                let len = (*q2.add(j + 1)).s - p.s;
                let (b, e) = (p.b, p.b + len);
                let mut q = piece(b2p, r2p, b);
                let mut at = b;
                loop {
                    let qq = *q2.add(q);
                    let qe = (*q2.add(q + 1)).s;
                    let stop = qe.min(e);
                    let cs = p.s + (at - b);
                    let ce = cs + (stop - at);
                    let base = qq.b + (at - qq.s);
                    let delta = base.wrapping_sub(cs) & MASK;
                    let word = (p.w | qq.w << 16) as u64;
                    let rec = delta as u64 | word << 20 | full;
                    r4 += 1;
                    if (*sp.add(si)).row >= ce {
                        put(cs, rec, &mut nr);
                    } else {
                        // the singletons inside [cs, ce), each a record of its own
                        let mut from = cs;
                        while (*sp.add(si)).row < ce {
                            let x = *sp.add(si);
                            si += 1;
                            if x.row > from {
                                put(from, rec, &mut nr);
                            }
                            let mut one = if x.len == 0 {
                                rec
                            } else {
                                x.delta as u64 | word << 20 | (x.len as u64) << LEN_SHIFT | SHORT
                            };
                            if x.start {
                                one |= START;
                            }
                            put(x.row, one, &mut nr);
                            from = x.row + 1;
                        }
                        if from < ce {
                            put(from, rec, &mut nr);
                        }
                    }
                    if stop == e {
                        break;
                    }
                    at = stop;
                    q += 1;
                }
            }
        }
        // SAFETY: nr records written, below the reserved capacity
        unsafe { self.recs.set_len(nr) };

        // 6. The final directory's ranks
        self.rank.clear();
        self.rank.reserve(words);
        let mut before = 0u32;
        for &w in &self.bits {
            self.rank.push(before.wrapping_sub(1));
            before += w.count_ones();
        }
        debug_assert_eq!(before as usize, nr);
        self.n = n;
        self.orig = orig;
        let s = &self.s;
        self.census = Census {
            rows: n as u64,
            r1: r1 as u64,
            r2: r2 as u64,
            r4: r4 as u64,
            records: nr as u64,
            singles: nsingle as u64,
            starts: self.starts.len() as u64,
            dir_bytes: 12 * words as u64,
            // (the registered bound, 2 (3 |Y| + |Y| + 1) records: one start more than M has)
            reserve_bytes: 8 * 2 * (4 * segments.max(1) as u64 + 1),
            scratch_bytes: (s.l1.capacity() * 12
                + s.l2.capacity() * 16
                + s.bits2.capacity() * 8
                + s.rank2.capacity() * 4) as u64,
            records_alloc: (self.recs.capacity() * 8) as u64,
        };
        true
    }

    /// Parents that pass each doubling whole (the census, after a build): level-1 pieces
    /// holding one level-2 piece, and level-2 pieces holding one record (a split at a
    /// singleton counts as a split); returns (whole, parents) for 1 → 2 and for 2 → 4
    pub fn unsplit(&self) -> ((u64, u64), (u64, u64)) {
        let (l1, l2) = (&self.s.l1, &self.s.l2);
        let (r1, r2) = (l1.len() - 1, l2.len() - 1);
        let mut a = 0;
        let mut k = 0;
        for j in 0..r1 {
            let mut c = 0;
            while k < r2 && l2[k].s < l1[j + 1].s {
                c += 1;
                k += 1;
            }
            a += u64::from(c == 1);
        }
        let mut b = 0;
        for j in 0..r2 {
            let (lo, hi) = (l2[j].s, l2[j + 1].s);
            let mut c = 0;
            for row in lo..hi {
                c += self.bits[(row >> 6) as usize] >> (row & 63) & 1;
                if c > 1 {
                    break;
                }
            }
            b += u64::from(c == 1);
        }
        ((a, r1 as u64), (b, r2 as u64))
    }

    /// Whether rows a and b (b may be n) lie in one record's piece (the census's heights)
    pub fn same_piece(&self, a: u32, b: u32) -> bool {
        let n = self.n as u32;
        assert!(a < n);
        // SAFETY: both rows are below n
        b < n
            && unsafe {
                piece(self.bits.as_ptr(), self.rank.as_ptr(), a)
                    == piece(self.bits.as_ptr(), self.rank.as_ptr(), b)
            }
    }

    /// The walk from z, a step a record, n bytes: each step's row and the bytes it emits (the
    /// census's heights; safe and slow)
    pub fn trace(&self, path: &mut Vec<(u32, u32)>) {
        path.clear();
        let (mut i, mut t) = (self.z, 0usize);
        while t < self.n {
            let d = (i >> 6) as usize;
            let k = self.rank[d].wrapping_add((self.bits[d] << (63 - (i & 63))).count_ones());
            let r = self.recs[k as usize];
            let len = (r >> LEN_SHIFT & 7) as u32;
            path.push((i, len));
            t += len as usize;
            i = i.wrapping_add(r as u32) & MASK;
        }
    }

    /// The walk: the room walk's lanes where the table has segments and the block is long
    /// enough, else the plain walk. Exactly `bwt::walk`'s output.
    pub fn walk(&self, out: &mut Vec<u8>, w: &mut WalkScratch) {
        if self.n < MIN_SEGMENTED {
            self.walk_plain(out, w)
        } else {
            self.walk_room::<LANES>(out, w)
        }
    }

    /// The plain walk: one record a step from `orig_ptr`, n bytes
    pub fn walk_plain(&self, out: &mut Vec<u8>, w: &mut WalkScratch) {
        if fast() {
            #[cfg(target_arch = "x86_64")]
            // SAFETY: the CPU has popcnt and BMI2
            return unsafe { self.plain_fast(out, w) };
        }
        self.plain_impl(out, w)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "popcnt,bmi1,bmi2")]
    unsafe fn plain_fast(&self, out: &mut Vec<u8>, w: &mut WalkScratch) {
        self.plain_impl(out, w)
    }

    #[inline(always)]
    fn plain_impl(&self, out: &mut Vec<u8>, w: &mut WalkScratch) {
        let n = self.n;
        out.clear();
        if n == 0 {
            return;
        }
        let buf = &mut w.plain;
        buf.clear();
        buf.reserve(n + 4);
        let (bp, rk, rc) = (self.bits.as_ptr(), self.rank.as_ptr(), self.recs.as_ptr());
        let o = buf.as_mut_ptr();
        // from z, n bytes; m, the cycle's length, is the first return to z (a proper stop,
        // so the walk lands on it exactly)
        let z = self.z;
        let (mut i, mut t, mut m) = (z, 0usize, 0usize);
        while t < n {
            // SAFETY: i < n (every record's next row is a row); t < n, and n + 4 bytes are
            // reserved for the 4-byte store
            unsafe {
                let r = *rc.add(piece(bp, rk, i));
                (o.add(t) as *mut u32).write_unaligned((r >> 20) as u32);
                t += (r >> LEN_SHIFT & 7) as usize;
                i = i.wrapping_add(r as u32) & MASK;
            }
            if m == 0 && i == z {
                m = t;
            }
        }
        assert!(m > 0, "the walk from LF³(orig_ptr) returned to it");
        out.reserve(n);
        // SAFETY: the bytes from z repeat with period m <= n, and buf holds the first n of
        // them; out has n bytes reserved, each written once
        unsafe {
            let h = (*rc.add(piece(bp, rk, self.orig)) >> 20) as u32;
            let (mut off, mut pos) = head(h, (m - n % m) % m, n, out.as_mut_ptr());
            while pos > 0 {
                let c = (m - off).min(pos);
                copy_reversed(o.add(off), out.as_mut_ptr().add(pos - c), c);
                pos -= c;
                off = 0;
            }
            out.set_len(n);
        }
    }

    /// The room walk with `L` lanes, on a table built with segments (any length of block)
    pub fn walk_room<const L: usize>(&self, out: &mut Vec<u8>, w: &mut WalkScratch) {
        if fast() {
            #[cfg(target_arch = "x86_64")]
            // SAFETY: the CPU has popcnt and BMI2
            return unsafe { self.room_fast::<L>(out, w) };
        }
        self.room_impl::<L>(out, w)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "popcnt,bmi1,bmi2")]
    unsafe fn room_fast<const L: usize>(&self, out: &mut Vec<u8>, w: &mut WalkScratch) {
        self.room_impl::<L>(out, w)
    }

    /// The room walk (production's `bwt::room_segments`, a record a step).
    ///
    /// What its unchecked accesses rely on:
    /// - **Rows:** every row read is below n: the starts are rows, and a record's next row is
    ///   LFʲ of a row (the build's composition from validated runs).
    /// - **Ownership:** each segment is taken once, by one lane, which writes only its buffer.
    /// - **Spare:** a store is 4 bytes at p with at least 4 spare. A pass's `room` offsets have
    ///   4 bytes each in every lane's spare; a lane taken mid-pass has `cap` ≥ 4 room.
    /// - **Initialised:** a lane's bytes are written consecutively from its buffer's start.
    #[inline(always)]
    fn room_impl<const L: usize>(&self, out: &mut Vec<u8>, w: &mut WalkScratch) {
        const { assert!(L <= 64 && L > 0) };
        let n = self.n;
        let starts = &self.starts[..];
        let ns = starts.len();
        let cap = 2 * n / ns + 64;
        w.bufs.resize_with(ns.max(w.bufs.len()), Vec::new);
        for b in &mut w.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        w.next.clear();
        w.next.resize(ns, usize::MAX);
        let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let (bp, rk, rc) = (self.bits.as_ptr(), self.rank.as_ptr(), self.recs.as_ptr());
        // SAFETY (rec): i < n (Rows)
        let rec = |i: u32| unsafe { *rc.add(piece(bp, rk, i)) };
        let bufs = &mut w.bufs;
        let next = &mut w.next;

        let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
            let row = starts[seg];
            let r = rec(row);
            // SAFETY: the buffer has cap >= 4 spare bytes
            unsafe {
                (base as *mut u32).write_unaligned((r >> 20) as u32);
                Lane {
                    row: row.wrapping_add(r as u32) & MASK,
                    seg,
                    p: base.add((r >> LEN_SHIFT & 7) as usize),
                    end: base.add(b.capacity()),
                }
            }
        };
        let finish = |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
            let b = &mut bufs[l.seg];
            // SAFETY: p is within b's allocation, every byte below it written
            unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
            next[l.seg] = seg_of(l.row);
        };
        let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
            let b = &mut bufs[l.seg];
            // SAFETY: p is within b's allocation, every byte below it written; p and end are
            // taken again from the grown buffer
            unsafe {
                let len = l.p.offset_from(b.as_ptr()) as usize;
                b.set_len(len);
                b.reserve(b.capacity());
                l.p = b.as_mut_ptr().add(len);
                l.end = b.as_mut_ptr().add(b.capacity());
            }
        };
        let spare = |l: &Lane| unsafe { l.end.offset_from(l.p) } as usize;

        let mut queued = 0;
        let mut lanes = [Lane {
            row: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        }; L];
        let mut active = 0;
        while active < L && queued < ns {
            lanes[active] = take(queued, bufs);
            queued += 1;
            active += 1;
        }
        if active == L {
            let mut rows = [0u32; L];
            for (k, l) in lanes.iter().enumerate() {
                rows[k] = l.row;
            }
            let mut tile = [[0u8; 4 * TO]; L];
            // copy lane l's tile row's bytes from..to into its buffer at p
            let flush = |l: &mut Lane, row: &[u8; 4 * TO], from: usize, to: usize| {
                debug_assert!(from <= to && to <= 4 * TO);
                debug_assert!(to - from <= spare(l));
                // SAFETY: from..to is inside the row; the pass's room bounds the lane's bytes
                unsafe {
                    std::ptr::copy_nonoverlapping(row.as_ptr().add(from), l.p, to - from);
                    l.p = l.p.add(to - from);
                }
            };
            'full: loop {
                let mut room = cap / 4;
                for l in lanes.iter_mut() {
                    if spare(l) < 4 {
                        grow(l, bufs);
                    }
                    room = room.min(spare(l) / 4);
                }
                debug_assert!(room > 0);
                let mut done = 0;
                while done < room {
                    let wd = (room - done).min(TO);
                    let mut st = [0usize; L];
                    for j in 0..wd {
                        let tp = tile.as_mut_ptr().cast::<u8>();
                        let rs = &mut rows;
                        let mut marked: u64 = 0;
                        macro_rules! step {
                            ($k:expr) => {{
                                let k = $k;
                                // SAFETY: k < L; rows are below n; k 4 TO + 4 j + 4 is inside
                                // the tile (j < wd <= TO)
                                unsafe {
                                    let r = rs.get_unchecked_mut(k);
                                    let i = *r;
                                    let q = rec(i);
                                    (tp.add(k * 4 * TO + 4 * j) as *mut u32)
                                        .write_unaligned((q >> 20) as u32);
                                    if q & STOP != 0 {
                                        marked |= 1 << k;
                                    } else {
                                        let nx = i.wrapping_add(q as u32) & MASK;
                                        *r = nx;
                                        prefetch(bp.add((nx >> 6) as usize));
                                    }
                                }
                            }};
                        }
                        let mut c = 0;
                        while c + 8 <= L {
                            step!(c);
                            step!(c + 1);
                            step!(c + 2);
                            step!(c + 3);
                            step!(c + 4);
                            step!(c + 5);
                            step!(c + 6);
                            step!(c + 7);
                            c += 8;
                        }
                        while c < L {
                            step!(c);
                            c += 1;
                        }
                        while marked != 0 {
                            let k = marked.trailing_zeros() as usize;
                            marked &= marked - 1;
                            let l = &mut lanes[k];
                            let i = rs[k];
                            let q = rec(i);
                            // a start: the segment ends before this offset; short: it ends
                            // with this offset's first j bytes, on a start
                            let (to, row) = if q & START != 0 {
                                (4 * j, i)
                            } else {
                                (
                                    4 * j + (q >> LEN_SHIFT & 7) as usize,
                                    (i.wrapping_add(q as u32) & MASK),
                                )
                            };
                            flush(l, &tile[k], 4 * st[k], to);
                            l.row = row;
                            finish(l, bufs, next);
                            if queued == ns {
                                // every lane but the marked ones after this one stepped at
                                // j; those still sit on their marked row, for the tail
                                for (x, lx) in lanes.iter_mut().enumerate() {
                                    if x != k {
                                        let waiting = marked >> x & 1 != 0;
                                        lx.row = rs[x];
                                        flush(
                                            lx,
                                            &tile[x],
                                            4 * st[x],
                                            4 * (j + usize::from(!waiting)),
                                        );
                                    }
                                }
                                break 'full;
                            }
                            *l = take(queued, bufs);
                            rs[k] = l.row;
                            queued += 1;
                            st[k] = j + 1;
                        }
                    }
                    for (k, l) in lanes.iter_mut().enumerate() {
                        flush(l, &tile[k], 4 * st[k], 4 * wd);
                    }
                    done += wd;
                }
            }
        }
        // the tail: the lanes still walking, one step each in turn
        let mut live: Vec<Lane> = lanes[..active]
            .iter()
            .copied()
            .filter(|l| l.seg != usize::MAX && next[l.seg] == usize::MAX)
            .collect();
        for l in live.iter_mut() {
            if spare(l) < 4 {
                grow(l, bufs);
            }
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let q = rec(l.row);
                if q & START != 0 {
                    finish(l, bufs, next);
                    live.swap_remove(i);
                    continue;
                }
                // SAFETY: at least 4 bytes spare (grown below when fewer)
                unsafe {
                    (l.p as *mut u32).write_unaligned((q >> 20) as u32);
                    l.p = l.p.add((q >> LEN_SHIFT & 7) as usize);
                }
                l.row = l.row.wrapping_add(q as u32) & MASK;
                if spare(l) < 4 {
                    grow(l, bufs);
                }
                i += 1;
            }
        }
        // the assembly. The segments from z's (a proper stop) round its cycle and back are
        // the cycle, m bytes. The output is the walk from orig_ptr, its first three bytes
        // then z's segments, from offset k = Ψ^n's distance (0 when m divides n), reversed
        // into the output from its end
        let segz = seg_of(self.z);
        let (mut m, mut g) = (0usize, segz);
        for _ in 0..ns {
            m += bufs[g].len();
            g = next[g];
            if g == segz {
                break;
            }
        }
        assert!(g == segz, "z's segments close its cycle");
        out.clear();
        out.reserve(n);
        let o = out.as_mut_ptr();
        // SAFETY: out has n bytes reserved; `head` and the copies write each of them once
        let (mut off, mut pos) =
            unsafe { head((rec(self.orig) >> 20) as u32, (m - n % m) % m, n, o) };
        let mut seg = segz;
        while pos > 0 && off >= bufs[seg].len() {
            off -= bufs[seg].len();
            seg = next[seg];
        }
        while pos > 0 {
            let b = &bufs[seg];
            let c = (b.len() - off).min(pos);
            // SAFETY: off + c <= b.len(); pos - c >= 0
            unsafe { copy_reversed(b.as_ptr().add(off), o.add(pos - c), c) };
            pos -= c;
            off = 0;
            seg = next[seg];
        }
        // SAFETY: every byte below n written
        unsafe { out.set_len(n) };
    }
}

/// The output's last bytes from the first three of the walk from orig_ptr (`h`'s low bytes),
/// from offset k of the walk: returns the offset into the walk from z that follows, and the
/// output's bytes still to write (below `pos`)
///
/// # Safety
/// o..o + n writable
#[inline(always)]
unsafe fn head(h: u32, k: usize, n: usize, o: *mut u8) -> (usize, usize) {
    let (mut pos, mut t) = (n, k);
    while t < 3 && pos > 0 {
        *o.add(pos - 1) = (h >> (8 * t)) as u8;
        pos -= 1;
        t += 1;
    }
    (t.saturating_sub(3), pos)
}

/// A directory over the pieces whose first rows `firsts` gives, ascending (the first 0):
/// `words` words of their bits and the counts before each word, less one
#[inline(always)]
fn dir(bits: &mut Vec<u64>, rank: &mut Vec<u32>, words: usize, firsts: impl Iterator<Item = u32>) {
    bits.clear();
    bits.resize(words, 0);
    for f in firsts {
        bits[(f >> 6) as usize] |= 1 << (f & 63);
    }
    rank.clear();
    rank.reserve(words);
    let mut before = 0u32;
    for &w in bits.iter() {
        rank.push(before.wrapping_sub(1));
        before += w.count_ones();
    }
}

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

    /// Every walk of the table against `bwt::walk`, on the column `ll` from `orig`
    fn check(ll: &[u8], orig: u32, segments: &[usize], lanes_small: bool) {
        let c = counts(ll);
        let mut tt = TVector::new();
        build(ll, &c, &mut tt);
        let mut want = Vec::new();
        walk(&tt, orig, &mut want);
        let (mut t, mut w, mut got) = (D4Table::new(), WalkScratch::default(), Vec::new());
        assert!(t.build(ll, &c, orig, 0));
        t.walk_plain(&mut got, &mut w);
        assert!(got == want, "plain, no segments, n {}", ll.len());
        for &s in segments {
            assert!(t.build(ll, &c, orig, s));
            t.walk_plain(&mut got, &mut w);
            assert!(got == want, "plain, {s} segments, n {}", ll.len());
            if lanes_small {
                t.walk_room::<3>(&mut got, &mut w);
                assert!(got == want, "3 lanes, {s} segments, n {}", ll.len());
            }
            t.walk_room::<8>(&mut got, &mut w);
            assert!(got == want, "8 lanes, {s} segments, n {}", ll.len());
            if !cfg!(miri) {
                t.walk_room::<32>(&mut got, &mut w);
                assert!(got == want, "32 lanes, {s} segments, n {}", ll.len());
            }
        }
    }

    fn column(rng: &mut Rng, n: usize, alphabet: u64, run: u64) -> Vec<u8> {
        let mut ll = Vec::with_capacity(n);
        while ll.len() < n {
            let b = (rng.below(alphabet) as u8).wrapping_mul(37);
            let len = 1 + rng.below(run) as usize;
            ll.extend(std::iter::repeat(b).take(len.min(n - ll.len())));
        }
        ll
    }

    #[test]
    fn random_columns() {
        let mut rng = Rng(0x0d4d_e17a_5eed_0001);
        for &n in &[
            1usize, 2, 3, 4, 5, 7, 8, 9, 17, 64, 65, 200, 1000, 5000, 70_000,
        ] {
            for &(alphabet, run) in &[(1u64, 1u64), (2, 1), (3, 5), (9, 20), (256, 1), (256, 3)] {
                let ll = column(&mut rng, n, alphabet, run);
                let orig = rng.below(n as u64) as u32;
                check(&ll, orig, &[1, 2, 7, 64, 512], n < 5000);
            }
        }
    }

    #[test]
    fn repeated_strings() {
        // several cycles: a block that is a string repeated
        let mut rng = Rng(0x0d4d_e17a_5eed_0002);
        for &(period, reps) in &[
            (1usize, 9usize),
            (2, 8),
            (3, 7),
            (5, 40),
            (16, 300),
            (100, 900),
        ] {
            let unit: Vec<u8> = (0..period).map(|_| b'a' + rng.below(4) as u8).collect();
            let text: Vec<u8> = unit.iter().copied().cycle().take(period * reps).collect();
            let (ll, orig) = bwt_of(&text);
            check(&ll, orig, &[1, 3, 64, 512], true);
        }
    }

    /// The forward BWT by sorting rotations (small texts only): the last column and the row
    /// of the text's own rotation, as bzip2 records them
    fn bwt_of(text: &[u8]) -> (Vec<u8>, u32) {
        let n = text.len();
        let twice = [text, text].concat();
        let mut rot: Vec<usize> = (0..n).collect();
        rot.sort_by(|&a, &b| twice[a..a + n].cmp(&twice[b..b + n]).then(a.cmp(&b)));
        let ll = rot.iter().map(|&r| text[(r + n - 1) % n]).collect();
        let orig = rot.iter().position(|&r| r == 0).unwrap() as u32;
        (ll, orig)
    }

    #[test]
    fn real_text() {
        let text: Vec<u8> =
            b"the walk is backward, and LF to the fourth is a translation on a piece; "
                .iter()
                .copied()
                .cycle()
                .take(3000)
                .chain(b"tail of no period".iter().copied())
                .collect();
        let (ll, orig) = bwt_of(&text);
        check(&ll, orig, &[1, 5, 64, 512], true);
    }

    #[test]
    fn refused() {
        let ll = b"banana".to_vec();
        let mut c = counts(&ll);
        let mut t = D4Table::new();
        assert!(!t.build(&ll, &c, 6, 0), "orig_ptr past the block");
        c[b'a' as usize] -= 1;
        c[b'b' as usize] += 1;
        assert!(!t.build(&ll, &c, 0, 0), "counts not the column's");
        assert!(t.is_empty());
        assert!(!t.build(&[], &[0; 256], 0, 0));
    }

    #[test]
    fn grows_mid_segment() {
        // few segments on a long block: the lanes' buffers (2 n / ns + 64) grow mid-segment
        // whenever a segment runs past twice its share
        let mut rng = Rng(0x0d4d_e17a_5eed_0003);
        let ll = column(&mut rng, 20_000, 4, 6);
        check(&ll, 17, &[2, 9], true);
    }

    #[test]
    fn miri_d4_walks() {
        let mut rng = Rng(0x0d4d_e17a_5eed_0004);
        for &n in &[1usize, 5, 40, 300] {
            for &(alphabet, run) in &[(1u64, 1u64), (3, 4), (256, 1)] {
                let ll = column(&mut rng, n, alphabet, run);
                let orig = rng.below(n as u64) as u32;
                check(&ll, orig, &[1, 4, 13], true);
            }
        }
        let (ll, orig) = bwt_of(b"abcabcabcabcabcabcabcabcabcabcab");
        check(&ll, orig, &[1, 3, 9], true);
        let (ll, orig) = bwt_of(&b"xy".repeat(20));
        check(&ll, orig, &[2, 7], true);
    }

    #[test]
    fn miri_d4_grows() {
        let mut rng = Rng(0x0d4d_e17a_5eed_0005);
        let ll = column(&mut rng, 1500, 3, 3);
        check(&ll, 3, &[4], true);
    }
}
