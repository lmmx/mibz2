//! The compact30 inverse BWT (`Compact30`): the default kernel. A block's successor list is
//! held as one 64-byte record per 30 rows instead of the T-vector's 4 bytes per row, built
//! straight from the last column in one scan, then walked as production's room walk walks
//! the T-vector. Exactly `bwt::walk`'s output.
//!
//! Ported from the research branch `research/compact30-combined` at 888eccf: the records,
//! labels and walk from `examples/c30walk.rs` (`C30::walk::<false>`, `room::<false>`,
//! `plain`), and the constructor from `examples/c30walk/ctor2c.rs` (`Ctor2c::build::<false>`:
//! constructor v2 with the scanner's U2 tile, the split descriptor arrays and v3's
//! epoch-tagged finaliser), with `ctor.rs`'s `canonicalise`. The counted builds, their
//! statistics and the research harnesses are left out; the hot paths are otherwise the
//! research code's, unchanged (the instruction ledger was measured on their compiled form:
//! docs/journal/2026-11-04-compact30-combined-build.md).
//!
//! **The representation:**
//! - **Records:** one 64-byte, 64-aligned record per 30 rows in F order (`Rec`): 30 `u16`
//!   fields and a `u32` header at byte 60.
//! - **Labels:** row r is labelled phi(r) = 32⌊r/30⌋ + r mod 30. Its field is at byte
//!   2·phi and its record's header at 2·(phi & !31) + 60. Successors are stored as labels.
//! - **An ordinary record:** header H = (B << 8) | F, where B is the least successor label
//!   of its 30 rows and F their common first-column byte. A field holds an offset ≤ 65,533,
//!   so the successor is (H >> 8) + offset and the row's output byte is H's low byte.
//! - **A RAW record** (its rows' bytes differ, or their successors span more than 65,533):
//!   header RAW_BIT | group index; every field 0xFFFF. Its group's 32 `u32` entries are
//!   `(successor label << 8) | F[row]` (`RawG`, 128 bytes), in increasing group order.
//! - **Markers:** a field of 0xFFFE marks a segment start; the walk saves the field and
//!   restores it at the end.
//! - **Parking:** one record of 0xFFFF fields after the last, at label PARK = 32·G.
//!
//! **The emission convention:** `F[r] = L[Ψ(r)]`, so `walk`'s output `L[Ψ(p)], L[Ψ²(p)], …`
//! from orig_ptr = p equals `F[p], F[Ψ(p)], …`. A lane at row x emits `F[x]` and moves to Ψ(x);
//! the first segment starts at phi(orig_ptr).
//!
//! **Where it can't represent a block** it runs production's kernel (`bwt::Segmented`):
//! a block of more than `MAX_N` rows (983,010; bzip2's blocks are at most 900,000), or
//! counts that are not the last column's own (the constructor refuses them, before any
//! record is published). Blocks below `bwt::MIN_SEGMENTED` rows are walked plainly on the
//! records, as production walks them plainly on the T-vector.
//!
//! **The unchecked accesses** (each has its argument at the site):
//! - the constructor's raw record arena (`W::dst`), written below its reserved capacity
//!   `g_count + 1` and published by `set_len` only once every record is whole; the
//!   descriptors' field pointers, each inside one open portion of one record;
//! - U2, the scanner's second tile segment (`ll.get_unchecked(i + n1..i + 16)`);
//! - `canonicalise`'s headers;
//! - the walk's field, header and RAW-entry loads, from labels that are live rows or PARK;
//!   its output writes, under production's room-walk contract (`bwt`, Ownership, Spare,
//!   Handoffs, Initialised, Lifetimes).

use crate::bwt::{self, random_rows, InverseBwt, SegScratch, Segmented, MIN_SEGMENTED, PF_L1};
use std::ptr;

const LANES: usize = 32;
const SEGMENTS: usize = 512;
const ROWS: usize = 30;
const START: u16 = 0xFFFE;
const RAWF: u16 = 0xFFFF;
const MAX_OFF: u32 = 65_533;
const RAW_BIT: u32 = 1 << 28;
const M20: u32 = (1 << 20) - 1;

/// One record: 30 fields, the header at byte 60
#[repr(C, align(64))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Rec {
    f: [u16; 30],
    h: u32,
}

/// A RAW group's entries: 30 used, padded to 32
#[repr(C, align(64))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RawG([u32; 32]);

const _: () = {
    assert!(std::mem::size_of::<Rec>() == 64);
    assert!(std::mem::align_of::<Rec>() == 64);
    assert!(std::mem::offset_of!(Rec, h) == 60);
    assert!(std::mem::offset_of!(Rec, f) == 0);
    assert!(std::mem::size_of::<RawG>() == 128);
};

#[inline(always)]
fn phi(r: usize) -> u32 {
    (32 * (r / ROWS) + r % ROWS) as u32
}

#[cfg(test)]
fn phi_inv(x: u32) -> usize {
    ROWS * (x >> 5) as usize + (x & 31) as usize
}

/// A hint that the record bytes at `p` are read soon: production's L1 prefetch
/// (`bwt::prefetch`), at a byte address
#[inline(always)]
fn prefetch(p: *const u8) {
    bwt::prefetch::<PF_L1>(p as *const u32, 0)
}

// Counts of the rarer transitions, in test builds only (per thread, so each test reads its
// own), so a test can show it reached them; nothing in other builds
#[cfg(test)]
pub(crate) mod hits {
    use std::cell::Cell;
    thread_local! {
        pub static RAW_ARRIVALS: Cell<usize> = const { Cell::new(0) };
        pub static RESUMPTIONS: Cell<usize> = const { Cell::new(0) };
        pub static WAIT_TURNS_FULL: Cell<usize> = const { Cell::new(0) };
        pub static WAIT_TURNS_TAIL: Cell<usize> = const { Cell::new(0) };
        pub static REPLACEMENTS: Cell<usize> = const { Cell::new(0) };
        pub static GROWS_FULL: Cell<usize> = const { Cell::new(0) };
        pub static GROWS_TAIL: Cell<usize> = const { Cell::new(0) };
        pub static MIDROUND_ENDS: Cell<usize> = const { Cell::new(0) };
        pub static LEAF: Cell<usize> = const { Cell::new(0) };
        pub static SLOW_ORDINARY: Cell<usize> = const { Cell::new(0) };
        pub static SLOW_EPOCH_SEARCH: Cell<usize> = const { Cell::new(0) };
        pub static WIDE: Cell<usize> = const { Cell::new(0) };
        pub static PRESET_RAW: Cell<usize> = const { Cell::new(0) };
        pub static FORCED: Cell<usize> = const { Cell::new(0) };
        pub static BULK: Cell<usize> = const { Cell::new(0) };
        pub static BULK_RAW: Cell<usize> = const { Cell::new(0) };
        pub static TILES_SECOND: Cell<usize> = const { Cell::new(0) };
        pub static TAILS: Cell<usize> = const { Cell::new(0) };
        pub static FALLBACKS: Cell<usize> = const { Cell::new(0) };
    }
    pub fn get(c: &'static std::thread::LocalKey<Cell<usize>>) -> usize {
        c.with(Cell::get)
    }
}
#[cfg(test)]
macro_rules! hit {
    ($c:ident) => {
        hits::$c.with(|c| c.set(c.get() + 1))
    };
}
#[cfg(not(test))]
macro_rules! hit {
    ($c:ident) => {};
}

// ---------------------------------------------------------------------------------------
// The constructor (research `Ctor2c::build::<false>`)
// ---------------------------------------------------------------------------------------

/// The largest block whose PARK label, 32·⌈n/30⌉, fits 20 bits
pub(crate) const MAX_N: usize = ROWS * (M20 as usize / 32);
/// Source rows per epoch: 30 × 2048, so φ advances by exactly 65,536 per epoch
const EPOCH: usize = 61_440;
const MAX_EPOCHS: usize = MAX_N.div_ceil(EPOCH);
/// `raw`'s discriminator (v3): set for an ordinary candidate's epoch tag, clear for a
/// preset RAW group's index (< 2^20)
const ORD: u32 = 0x8000_0000;
/// A descriptor's `raw` before its first opening (v3). Never read: only a nonempty portion
/// is finished, and every portion is opened (which sets `raw`) before its first append.
const UNOPENED: u32 = u32::MAX;

/// The ordinary tag of source epoch e (< 16): bit 31, and e in bits 24–27, where the final
/// header's 65536·e term of y << 8 lands (v3)
#[inline(always)]
const fn epoch_tag(e: usize) -> u32 {
    ORD | (e as u32) << 24
}

const _: () = {
    assert!(MAX_EPOCHS == 16);
    // an ordinary tag can never be a RAW index, and its epoch bits fit below the
    // discriminator
    assert!(M20 < ORD && epoch_tag(MAX_EPOCHS - 1) & !ORD == ((MAX_EPOCHS - 1) as u32) << 24);
    assert!(EPOCH % 16 == 0 && EPOCH % ROWS == 0);
};

/// The thirty complete-record patterns: field j of pattern p is φ(i + j) − φ(i) for a
/// source row i with i mod 30 = p
static PATTERNS: [Rec; 30] = patterns();

const fn patterns() -> [Rec; 30] {
    let mut t = [Rec { f: [0; 30], h: 0 }; 30];
    let mut p = 0;
    while p < 30 {
        let mut j = 0;
        while j < 30 {
            t[p].f[j] = (j + 2 * ((p + j >= 30) as usize)) as u16;
            j += 1;
        }
        p += 1;
    }
    t
}

/// The research oracle's mixer, for the forced-RAW predicate (tests only force groups)
fn mix(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    x ^ (x >> 33)
}

/// Whether group g of n rows is forced RAW at `permille` (0 in production: none is)
fn forced(g: usize, n: usize, permille: u32) -> bool {
    permille > 0 && (mix(g as u64 ^ (n as u64) << 32) % 1000) < permille as u64
}

/// A full tile's two qwords, read from its 16-byte array at constant offsets (scanner)
#[inline(always)]
fn halves(t: &[u8; 16]) -> (u64, u64) {
    let (lo, hi) = t.split_at(8);
    (
        u64::from_ne_bytes(lo.try_into().unwrap()),
        u64::from_ne_bytes(hi.try_into().unwrap()),
    )
}

/// The cold part of a character's descriptor (the hot `write` and `end` are in `W::wr`
/// and `W::en`). `raw` is v3's: a preset RAW group's index (high bit clear), or the
/// ordinary candidate's tag `epoch_tag(e)` (high bit set), or `UNOPENED`, never read.
#[derive(Clone, Copy)]
struct Cold {
    w0: *mut u16,
    r0: u32,
    lim: u32,
    g: u32,
    raw: u32,
}

/// A character's write descriptor, assembled from the split storage. Empty (no open
/// portion) when `w0 == write == end` (null); otherwise its open portion is the fields
/// `[w0, end)` of group `g`, starting at destination rank `r0`, of which `[w0, write)`
/// hold temporary labels.
#[derive(Clone, Copy)]
struct Desc {
    write: *mut u16,
    end: *mut u16,
    w0: *mut u16,
    r0: u32,
    g: u32,
    raw: u32,
}

/// The constructor's reusable scratch: the RAW bitmap and the epoch snapshots
#[derive(Default)]
struct Ctor {
    bitmap: Vec<u64>,
    snaps: Vec<[u32; 256]>,
}

/// One build's state
struct W<'a> {
    /// the record arena: `g_count + 1` records of capacity, none yet initialised
    dst: *mut Rec,
    g_count: usize,
    raw: &'a mut Vec<RawG>,
    bitmap: &'a mut [u64],
    snaps: &'a mut Vec<[u32; 256]>,
    /// the hot descriptor arrays: each character's next field and its portion's end
    wr: [*mut u16; 256],
    en: [*mut u16; 256],
    /// the rest of each descriptor
    cold: [Cold; 256],
    /// the source row's full label φ(i) and phase i mod 30
    y: u32,
    p: usize,
    /// `epoch_tag(e)` for the current source epoch e, set with its snapshot (v3)
    active_tag: u32,
}

impl W<'_> {
    /// Character c's descriptor, assembled from the split storage
    #[inline(always)]
    fn desc(&self, c: usize) -> Desc {
        let k = self.cold[c];
        Desc {
            write: self.wr[c],
            end: self.en[c],
            w0: k.w0,
            r0: k.r0,
            g: k.g,
            raw: k.raw,
        }
    }

    /// Character c's next destination rank
    #[inline(always)]
    fn rank(&self, c: usize) -> u32 {
        self.cold[c].r0 + ((self.wr[c] as usize - self.cold[c].w0 as usize) / 2) as u32
    }

    #[inline(always)]
    fn is_raw(&self, g: usize) -> bool {
        self.bitmap[g >> 6] >> (g & 63) & 1 != 0
    }

    /// The field of destination rank r. SAFETY: r < n ≤ 30·g_count, so the record is
    /// inside the arena's capacity (`addr_of_mut!` makes no reference to its uninitialised
    /// bytes)
    #[inline(always)]
    unsafe fn field(&self, r: u32) -> *mut u16 {
        let (g, j) = (r as usize / ROWS, r as usize % ROWS);
        debug_assert!(g < self.g_count);
        (ptr::addr_of_mut!((*self.dst.add(g)).f) as *mut u16).add(j)
    }

    #[inline(always)]
    fn advance(&mut self, k: usize) {
        let p = self.p + k;
        if p >= ROWS {
            self.p = p - ROWS;
            self.y += k as u32 + 2;
        } else {
            self.p = p;
            self.y += k as u32;
        }
    }

    /// Group g made RAW (all 64 bytes of its record written) with a zeroed `RawG`.
    /// SAFETY: g < g_count, inside the arena's capacity
    unsafe fn create_raw(&mut self, g: usize) -> u32 {
        debug_assert!(g < self.g_count && !self.is_raw(g));
        let idx = self.raw.len() as u32;
        self.raw.push(RawG([0; 32]));
        self.dst.add(g).write(Rec {
            f: [RAWF; 30],
            h: RAW_BIT | idx,
        });
        self.bitmap[g >> 6] |= 1 << (g & 63);
        idx
    }

    /// Save every character's next destination rank: the snapshot A_e; then the epoch's tag
    fn snapshot(&mut self) {
        let mut a = [0u32; 256];
        for (c, x) in a.iter_mut().enumerate() {
            *x = self.rank(c);
        }
        self.snaps.push(a);
        // the epoch being entered: its index is its snapshot's, before any of its rows
        self.active_tag = epoch_tag(self.snaps.len() - 1);
    }

    /// The epoch of received rank r of bucket c: the greatest e with `A_e[c] ≤ r`
    #[inline(always)]
    fn epoch_of(&self, c: usize, r: u32) -> usize {
        let mut e = self.snaps.len() - 1;
        while self.snaps[e][c] > r {
            e -= 1;
        }
        e
    }

    /// Open a portion of character c at rank r (< its limit): its group's slots from r
    /// to the group's end or the bucket's. SAFETY: r < lim ≤ n
    #[inline(always)]
    unsafe fn open(&mut self, c: usize, r: u32) {
        let lim = self.cold[c].lim;
        debug_assert!(r < lim);
        let g = r as usize / ROWS;
        let re = lim.min(((g + 1) * ROWS) as u32);
        let w0 = self.field(r);
        let raw = if self.is_raw(g) {
            // a RAW group's record was written whole by `create_raw`
            ptr::addr_of!((*self.dst.add(g)).h).read() & M20
        } else {
            // an ordinary candidate covers its whole group: a non-crossing group's slots
            // all belong to one bucket, whose earlier portions end at group boundaries.
            // Its tag is the current epoch's: the opening is followed at once by the
            // portion's first append (refill → one, or run → open → fill), with no epoch
            // change between
            debug_assert!(r as usize % ROWS == 0);
            self.active_tag
        };
        self.wr[c] = w0;
        self.en[c] = w0.add((re - r) as usize);
        self.cold[c] = Cold {
            w0,
            r0: r,
            lim,
            g: g as u32,
            raw,
        };
    }

    /// Finish c's pending portion (if any) and leave the descriptor empty at its next rank
    #[inline(always)]
    unsafe fn close(&mut self, c: usize) {
        if self.wr[c] != self.cold[c].w0 {
            debug_assert!(c < 256);
            self.finish(c as u8);
        }
        self.cold[c].r0 = self.rank(c);
        self.cold[c].w0 = ptr::null_mut();
        self.wr[c] = ptr::null_mut();
        self.en[c] = ptr::null_mut();
    }

    /// The one-row path's slow edge: the open portion is full (or there is none). Finish
    /// it, and open the next, or reject an exhausted bucket before any write.
    #[cold]
    #[inline(never)]
    unsafe fn refill(&mut self, c: usize) -> bool {
        self.close(c);
        let r = self.cold[c].r0;
        if r >= self.cold[c].lim {
            return false;
        }
        self.open(c, r);
        true
    }

    /// Finish c's full pending portion (nonempty: its callers check). v3's fast leaf: an
    /// ordinary candidate opened in the current epoch holding 30 fields, finished with no
    /// snapshot read and no span test; anything else goes to `finish_slow`.
    ///
    /// Why the leaf is exact (v3's release-mode argument): `raw == active_tag` has the high
    /// bit set, so the portion is ordinary, and was opened in the current epoch e; its first
    /// append followed the opening at once, so every one of its rows was received in epoch
    /// e (finishing happens within e or at its end), and its labels are 65536·e + u with
    /// u ≤ 65,533 increasing: the span is ul − u0 ≤ 65,533 and the first full label is
    /// y0 = 65536·e + u0, so y0 << 8 = (tag & !ORD) | u0 << 8. 30 fields means `w0` is its
    /// record's field 0 (a portion never passes its group's end), so the header is at
    /// `w0 + 60` bytes, and `[w0, w0 + 30)` were all written.
    #[inline(never)]
    unsafe fn finish(&mut self, c: u8) {
        let k = &self.cold[c as usize];
        let tag = k.raw;
        if tag == self.active_tag && self.wr[c as usize] as usize - k.w0 as usize == 2 * ROWS {
            hit!(LEAF);
            let p = k.w0;
            let u0 = p.read();
            // SAFETY: the record's header, at byte 60 of the record whose field 0 is p
            (p as *mut u8)
                .add(60)
                .cast::<u32>()
                .write((tag & !ORD) | (u0 as u32) << 8 | c as u32);
            // the 60 field bytes, all read before any is written (v2's five chunks)
            let fp = p as *mut [u16; 30];
            let mut f = fp.read();
            for x in f.iter_mut() {
                *x = x.wrapping_sub(u0);
            }
            fp.write(f);
            return;
        }
        self.finish_slow(c as usize)
    }

    /// Finish c's full pending portion: classify it from its full labels, then write its
    /// final form (once). v2's `finish`, with the ordinary/RAW test on `raw`'s high bit
    /// (v3's `finish_slow`)
    #[cold]
    #[inline(never)]
    unsafe fn finish_slow(&mut self, c: usize) {
        let d = self.desc(c);
        let k = (d.write as usize - d.w0 as usize) / 2;
        debug_assert!(k >= 1 && d.write == d.end);
        let (r0, r1) = (d.r0, d.r0 + k as u32 - 1);
        let g = d.g as usize;
        let j0 = r0 as usize - ROWS * g;
        let last = self.snaps.len() - 1;
        let fast = self.snaps[last][c] <= r0;
        let (h0, h1) = if fast {
            (last, last)
        } else {
            hit!(SLOW_EPOCH_SEARCH);
            (self.epoch_of(c, r0), self.epoch_of(c, r1))
        };
        if d.raw & ORD != 0 {
            // an ordinary candidate: its whole group, j0 = 0
            let u0 = d.w0.read();
            let ul = d.w0.add(k - 1).read();
            let y0 = 65536 * h0 as u32 + u0 as u32;
            let yl = 65536 * h1 as u32 + ul as u32;
            // classified from the full labels, before any modular subtraction
            let span = yl - y0;
            if span <= MAX_OFF {
                hit!(SLOW_ORDINARY);
                ptr::addr_of_mut!((*self.dst.add(g)).h).write(y0 << 8 | c as u32);
                if k == ROWS {
                    // the 60 field bytes, all read before any is written; nothing at or
                    // past byte 60 (the header) is read or written
                    let fp = d.w0 as *mut [u16; 30];
                    let mut f = fp.read();
                    for x in f.iter_mut() {
                        *x = x.wrapping_sub(u0);
                    }
                    fp.write(f);
                } else {
                    // a partial last group: its own k fields only
                    for t in 0..k {
                        let p = d.w0.add(t);
                        p.write(p.read().wrapping_sub(u0));
                    }
                }
            } else {
                hit!(WIDE);
                // wide: RAW at completion, the entries reconstructed, the record replaced
                let idx = self.raw.len();
                self.raw.push(RawG([0; 32]));
                self.write_entries(c, idx, d.w0, j0, k, r0, h0, h1);
                self.dst.add(g).write(Rec {
                    f: [RAWF; 30],
                    h: RAW_BIT | idx as u32,
                });
                self.bitmap[g >> 6] |= 1 << (g & 63);
            }
        } else {
            hit!(PRESET_RAW);
            // a preset RAW group's portion: into its RAW block, then only this portion's
            // fields restored to 0xFFFF (other characters own the other slots)
            self.write_entries(c, d.raw as usize, d.w0, j0, k, r0, h0, h1);
            for t in 0..k {
                d.w0.add(t).write(RAWF);
            }
        }
    }

    /// A portion's RAW entries, `(y << 8) | c` with y = 65536·h(r) + u, epoch interval by
    /// epoch interval from h0 to h1. SAFETY: the portion's k fields hold temporary labels
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    unsafe fn write_entries(
        &mut self,
        c: usize,
        idx: usize,
        w0: *mut u16,
        j0: usize,
        k: usize,
        r0: u32,
        h0: usize,
        h1: usize,
    ) {
        let mut e = h0;
        let mut next = if e < h1 {
            self.snaps[e + 1][c]
        } else {
            u32::MAX
        };
        let entries = &mut self.raw[idx].0;
        for t in 0..k {
            let r = r0 + t as u32;
            while r >= next {
                e += 1;
                next = if e < h1 {
                    self.snaps[e + 1][c]
                } else {
                    u32::MAX
                };
            }
            let y = 65536 * e as u32 + w0.add(t).read() as u32;
            entries[j0 + t] = y << 8 | c as u32;
        }
    }

    /// The one-row append: the temporary label into the character's next field.
    /// SAFETY: after a refill (or without one) `wr[c] < en[c]`, inside the open portion
    #[inline(always)]
    unsafe fn one(&mut self, c: u8, u: u16) -> bool {
        let ci = c as usize;
        if self.wr[ci] == self.en[ci] && !self.refill(ci) {
            return false;
        }
        // the pointer re-read after the store, as v2's `d.write = d.write.add(1)` (the
        // descriptor prototype's attempt 2)
        self.wr[ci].write(u);
        self.wr[ci] = self.wr[ci].add(1);
        true
    }

    /// v2's tile, unchanged, for the tail only (full nonuniform tiles take `tile16`)
    #[inline(always)]
    unsafe fn tile(&mut self, ll: &[u8], i: usize, len: usize) -> bool {
        // the checked path is the tail's only: a tail never takes U2
        debug_assert!(len < 16);
        hit!(TAILS);
        let u = self.y as u16;
        let n1 = len.min(ROWS - self.p);
        for (t, &c) in ll[i..i + n1].iter().enumerate() {
            if !self.one(c, u.wrapping_add(t as u16)) {
                return false;
            }
        }
        for (t, &c) in ll[i + n1..i + len].iter().enumerate() {
            if !self.one(c, u.wrapping_add((n1 + t) as u16 + 2)) {
                return false;
            }
        }
        if len == 16 {
            self.advance(16);
        } else {
            for _ in 0..len {
                self.advance(1);
            }
        }
        true
    }

    /// A nonuniform full tile (the scanner's U2 form, unchanged): `t` is the 16-byte chunk
    /// that `split_first_chunk` split off `rest`, at row index i
    ///
    /// The contract (the scanner journal's §9.2): t = `ll[i..i + 16]`, so
    /// i + 16 ≤ end ≤ `ll.len()`; and n1 = min(16, 30 − p) ≤ 16 for any p. Then
    /// U2 = [i + n1, i + 16) is well-ordered and inside `ll`. (U1, [i, i + n1), is not
    /// used: the do-while reads the first segment from `t`.)
    #[inline(always)]
    unsafe fn tile16(&mut self, ll: &[u8], i: usize, t: &[u8; 16]) -> bool {
        // the dominating full-tile contract, checked under debug assertions (Miri): the
        // tile is the chunk split off `rest`
        debug_assert!(ptr::eq(t.as_ptr(), ll.as_ptr().wrapping_add(i)));
        debug_assert!(self.p < ROWS);
        let u = self.y as u16;
        let n1 = 16usize.min(ROWS.wrapping_sub(self.p));
        // SAFETY (U2): i + n1 ≤ i + 16 ≤ ll.len() by the contract above: `t` is
        // `ll[i..i + 16]`, split off the epoch's unread rows `ll[i..end]` with end ≤
        // ll.len() by `split_first_chunk`, which returns a tile only when 16 bytes remain
        let s2 = unsafe { ll.get_unchecked(i + n1..i + 16) };
        #[cfg(test)]
        if n1 < 16 {
            hit!(TILES_SECOND);
        }
        // C4, safe: the first segment as a do-while over the 16-byte window, t[k] with
        // k < n1 ≤ 16 (U1 is not used). p < 30, so n1 ≥ 1, matters for correctness only:
        // were n1 0, this would append one wrong row, reading in bounds.
        let mut k = 0usize;
        loop {
            if !self.one(t[k], u.wrapping_add(k as u16)) {
                return false;
            }
            k += 1;
            if k >= n1 {
                break;
            }
        }
        for (t2, &c) in s2.iter().enumerate() {
            if !self.one(c, u.wrapping_add((n1 + t2) as u16 + 2)) {
                return false;
            }
        }
        self.advance(16);
        true
    }

    /// Append m (≤ the open portion's room) temporary labels of a run, from the source's
    /// pattern. SAFETY: m ≤ the room left in c's open portion
    #[inline(always)]
    unsafe fn fill(&mut self, c: usize, m: usize) {
        let pat = &PATTERNS[self.p].f;
        let y = self.y;
        let w = self.wr[c];
        debug_assert!(m <= (self.en[c] as usize - w as usize) / 2);
        for (t, &o) in pat[..m].iter().enumerate() {
            w.add(t).write((y + o as u32) as u16);
        }
        self.wr[c] = w.add(m);
        self.advance(m);
    }

    /// `whole` complete groups of byte c from group g (v1's emitter), or in a forced
    /// configuration each preset group's RAW entries directly. SAFETY: g + whole ≤ g_count
    #[inline(always)]
    unsafe fn complete(&mut self, c: u8, g: usize, whole: usize, forced_cfg: bool) {
        debug_assert!(g + whole <= self.g_count);
        if !forced_cfg {
            hit!(BULK);
            let mut r = PATTERNS[self.p];
            r.h = self.y << 8 | c as u32;
            let d = self.dst.add(g);
            for t in 0..whole {
                d.add(t).write(r);
                r.h += 32 << 8;
            }
            self.y += 32 * whole as u32;
            return;
        }
        for t in 0..whole {
            let gt = g + t;
            if self.is_raw(gt) {
                hit!(BULK_RAW);
                let idx = (ptr::addr_of!((*self.dst.add(gt)).h).read() & M20) as usize;
                let pat = &PATTERNS[self.p].f;
                let y = self.y;
                for (e, &o) in self.raw[idx].0[..ROWS].iter_mut().zip(pat) {
                    *e = (y + o as u32) << 8 | c as u32;
                }
            } else {
                let mut r = PATTERNS[self.p];
                r.h = self.y << 8 | c as u32;
                self.dst.add(gt).write(r);
            }
            self.y += 32;
        }
    }

    /// A coalesced run of m rows of byte c (within one epoch): one limit check, then the
    /// open portion's remainder, complete groups, and the suffix in a new portion
    #[inline(always)]
    unsafe fn run(&mut self, c: u8, mut m: usize, forced_cfg: bool) -> bool {
        let ci = c as usize;
        if self.rank(ci) as usize + m > self.cold[ci].lim as usize {
            return false;
        }
        loop {
            let room = (self.en[ci] as usize - self.wr[ci] as usize) / 2;
            let k = room.min(m);
            if k > 0 {
                self.fill(ci, k);
                m -= k;
            }
            if m == 0 {
                return true;
            }
            self.close(ci);
            let r = self.cold[ci].r0 as usize;
            if r % ROWS == 0 && m >= ROWS {
                let whole = m / ROWS;
                self.complete(c, r / ROWS, whole, forced_cfg);
                m -= ROWS * whole;
                self.cold[ci].r0 = (r + ROWS * whole) as u32;
                if m == 0 {
                    return true;
                }
            }
            let r = self.cold[ci].r0;
            self.open(ci, r);
        }
    }
}

impl Ctor {
    /// `rec` and `raw` for the last column `ll` and its `counts`, with `permille` of the
    /// groups that would be ordinary forced RAW (0 in production; the tests force some, to
    /// reach the RAW paths on small blocks). False, with both arrays empty, if the counts
    /// don't sum to n, n is out of range (above `MAX_N`), or a bucket overflows (the counts
    /// are not `ll`'s own).
    #[inline(never)]
    fn build(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        permille: u32,
        rec: &mut Vec<Rec>,
        raw: &mut Vec<RawG>,
    ) -> bool {
        rec.clear();
        raw.clear();
        let n = ll.len();
        if n >= 1 << 24 || n > MAX_N || counts.iter().map(|&c| c as usize).sum::<usize>() != n {
            return false;
        }
        let g_count = n.div_ceil(ROWS);
        let forced_cfg = permille > 0;
        let sigma = counts.iter().filter(|&&c| c > 0).count();
        let raw_req = if forced_cfg {
            g_count
        } else if n == 0 {
            0
        } else {
            g_count.min(sigma * (phi(n - 1) as usize / 65_534) + sigma - 1)
        };
        if rec.capacity() < g_count + 1 {
            *rec = Vec::new();
            rec.reserve_exact(g_count + 1);
        }
        if raw.capacity() < raw_req {
            *raw = Vec::new();
            raw.reserve_exact(raw_req);
        }
        let Ctor { bitmap, snaps } = self;
        bitmap.clear();
        bitmap.resize(g_count.div_ceil(64), 0);
        snaps.clear();
        snaps.reserve(MAX_EPOCHS);
        let mut cold = [Cold {
            w0: ptr::null_mut(),
            r0: 0,
            lim: 0,
            g: 0,
            raw: UNOPENED,
        }; 256];
        let mut cum = 0u32;
        for (d, &k) in cold.iter_mut().zip(counts.iter()) {
            d.r0 = cum;
            cum += k;
            d.lim = cum;
        }
        let ok = {
            let mut w = W {
                dst: rec.as_mut_ptr(),
                g_count,
                raw: &mut *raw,
                bitmap,
                snaps,
                wr: [ptr::null_mut(); 256],
                en: [ptr::null_mut(); 256],
                cold,
                y: 0,
                p: 0,
                // replaced by epoch 0's tag at the first snapshot, before any opening
                active_tag: UNOPENED,
            };
            // SAFETY: every record index below is under g_count + 1, the arena's capacity
            unsafe { Self::scatter(&mut w, ll, counts, permille) }
        };
        if !ok {
            raw.clear();
            return false;
        }
        // SAFETY: as above; every portion is finished, so no temporary label remains, and
        // every record below g_count was written whole (a complete group by `complete`, a
        // RAW one by `create_raw` or a wide finish, an ordinary one's 30 fields by its
        // portions and its header by its finish, a partial last group's missing fields
        // here); then PARK at g_count
        unsafe {
            let dst = rec.as_mut_ptr();
            let used = n - ROWS * g_count.saturating_sub(1);
            if g_count > 0 && used < ROWS {
                let f = ptr::addr_of_mut!((*dst.add(g_count - 1)).f) as *mut u16;
                for t in used..ROWS {
                    f.add(t).write(RAWF);
                }
            }
            dst.add(g_count).write(Rec {
                f: [RAWF; 30],
                h: RAW_BIT | M20,
            });
            let _ = canonicalise(dst, bitmap, raw);
            rec.set_len(g_count + 1);
        }
        true
    }

    /// The preset RAW groups, then the scan of L epoch by epoch, then every pending
    /// portion finished. False at the first overflowing bucket (before its write) or if
    /// any bucket ends short.
    #[inline(always)]
    unsafe fn scatter(w: &mut W<'_>, ll: &[u8], counts: &[u32; 256], permille: u32) -> bool {
        let n = ll.len();
        let mut cum = 0usize;
        let mut last = usize::MAX;
        for &c in counts.iter() {
            cum += c as usize;
            if cum > 0 && cum < n && cum % ROWS != 0 {
                let g = cum / ROWS;
                if g != last {
                    w.create_raw(g);
                    last = g;
                }
            }
        }
        let forced_cfg = permille > 0;
        if forced_cfg {
            for g in 0..w.g_count {
                if !w.is_raw(g) && forced(g, n, permille) {
                    hit!(FORCED);
                    w.create_raw(g);
                }
            }
        }
        let mut i = 0;
        while i < n {
            // a new epoch: the snapshot before any of its rows
            w.snapshot();
            let end = n.min(i + EPOCH);
            // the epoch's unread rows: a full tile is split off its front, the split being
            // the loop's own bounds test; a held tile stays at the front
            let mut rest: &[u8] = &ll[i..end];
            let mut held: Option<(u64, u64)> = None;
            while let Some((t, after)) = rest.split_first_chunk::<16>() {
                // `t` is read only on the fresh path; a nonuniform tile is re-read from `ll`
                let (a, b) = match held.take() {
                    Some(v) => v,
                    None => halves(t),
                };
                if a == b && a == a.rotate_left(8) {
                    let mut r2 = after;
                    while let Some((t2, after2)) = r2.split_first_chunk::<16>() {
                        let (a2, b2) = halves(t2);
                        if a2 == a && b2 == a {
                            r2 = after2;
                        } else {
                            held = Some((a2, b2));
                            break;
                        }
                    }
                    // the run's rows: those its tiles span
                    if !w.run(a as u8, rest.len() - r2.len(), forced_cfg) {
                        return false;
                    }
                    rest = r2;
                } else {
                    // the scanner's tile, at the tile's row index in `ll`
                    if !w.tile16(ll, end - rest.len(), t) {
                        return false;
                    }
                    rest = after;
                }
            }
            if !rest.is_empty() && !w.tile(ll, end - rest.len(), rest.len()) {
                return false;
            }
            i = end;
        }
        for c in 0..256 {
            if w.rank(c) != w.cold[c].lim {
                return false;
            }
        }
        for c in 0..256 {
            w.close(c);
        }
        true
    }
}

/// The RAW groups put in increasing-group order, in place (research `ctor::canonicalise`).
/// Each RAW record's provisional index is replaced by its rank among the RAW groups, which
/// is stored in its `RawG`'s word 31 (padding) as the destination tag; the groups are then
/// permuted by cycle rotation with one carry `RawG`, and every word 31 is zeroed again.
/// Returns (groups moved, nontrivial cycles, the longest cycle).
///
/// SAFETY: every group whose bit is set has its record's header written, holding its
/// provisional index; the set bits and the RAW groups correspond one to one; word 31 of
/// every RAW group is zero.
unsafe fn canonicalise(dst: *mut Rec, bitmap: &[u64], raw: &mut [RawG]) -> (u64, u64, u64) {
    let mut rank = 0u32;
    for (w, &word) in bitmap.iter().enumerate() {
        let mut b = word;
        while b != 0 {
            let g = 64 * w + b.trailing_zeros() as usize;
            b &= b - 1;
            let hp = ptr::addr_of_mut!((*dst.add(g)).h);
            let old = (hp.read() & M20) as usize;
            debug_assert_eq!(raw[old].0[31], 0);
            raw[old].0[31] = rank;
            hp.write(RAW_BIT | rank);
            rank += 1;
        }
    }
    assert_eq!(rank as usize, raw.len(), "one RAW group per set bit");
    let (mut moved, mut cycles, mut longest) = (0u64, 0u64, 0u64);
    for s in 0..raw.len() {
        if raw[s].0[31] as usize == s {
            continue;
        }
        // the cycle through s: each swap puts the carried group at its destination and
        // picks up the one it displaces
        let mut carry = raw[s];
        let mut len = 0u64;
        loop {
            let d = carry.0[31] as usize;
            std::mem::swap(&mut carry, &mut raw[d]);
            len += 1;
            if d == s {
                break;
            }
        }
        moved += len;
        cycles += 1;
        longest = longest.max(len);
    }
    for (k, g) in raw.iter_mut().enumerate() {
        debug_assert_eq!(g.0[31] as usize, k);
        g.0[31] = 0;
    }
    (moved, cycles, longest)
}

// ---------------------------------------------------------------------------------------
// The walk (research `C30::walk::<false>`, `room::<false>`, `plain`)
// ---------------------------------------------------------------------------------------

/// A lane: its encoded row, the RAW entry it waits on, its segment and output pointers
/// (32 bytes)
#[derive(Clone, Copy)]
struct Lane {
    row: u32,
    pend: u32,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

const _: () = assert!(std::mem::size_of::<Lane>() == 32);

/// What a slow step found
enum Slow {
    Byte(u8),
    Start,
    Wait,
}

/// The records, the RAW groups and the walk's scratch
#[derive(Default)]
struct C30 {
    n: usize,
    rec: Vec<Rec>,
    raw: Vec<RawG>,
    s: SegScratch,
    /// the marked starts' saved fields, and their (successor << 8) | F
    saved: Vec<u16>,
    succ: Vec<u32>,
}

impl C30 {
    fn park(&self) -> u32 {
        32 * (self.rec.len() as u32 - 1)
    }

    /// Ψ(x) and `F[x]` from the records, no start marked (checked indexing)
    fn next(&self, x: u32) -> (u32, u8) {
        let r = &self.rec[(x >> 5) as usize];
        let o = r.f[(x & 31) as usize];
        if o < START {
            ((r.h >> 8) + o as u32, r.h as u8)
        } else {
            debug_assert!(r.h & RAW_BIT != 0);
            let e = self.raw[(r.h & M20) as usize].0[(x & 31) as usize];
            (e >> 8, e as u8)
        }
    }

    #[cfg(test)]
    fn is_raw(&self, x: u32) -> bool {
        self.rec[(x >> 5) as usize].f[(x & 31) as usize] == RAWF
    }

    /// The plain walk, for a block under MIN_SEGMENTED rows
    fn plain(&self, orig_ptr: u32, out: &mut Vec<u8>) {
        out.clear();
        out.reserve(self.n);
        let mut x = phi(orig_ptr as usize);
        for _ in 0..self.n {
            let (s, b) = self.next(x);
            out.push(b);
            x = s;
        }
    }

    /// The segments' starts, as labels: production's random rows (`bwt::random_rows`) and
    /// the first
    fn starts(&mut self, orig_ptr: u32) {
        let st = &mut self.s.starts;
        random_rows(self.n, SEGMENTS - 1, st);
        for x in st.iter_mut() {
            *x = phi(*x as usize);
        }
        st.push(phi(orig_ptr as usize));
        st.sort_unstable();
        st.dedup();
    }

    /// The segmented walk into the segments, then their assembly
    /// (`SegScratch::assemble`); the plain walk below MIN_SEGMENTED rows
    fn walk(&mut self, orig_ptr: u32, out: &mut Vec<u8>) {
        if self.n < MIN_SEGMENTED {
            self.plain(orig_ptr, out);
            return;
        }
        self.starts(orig_ptr);
        self.room(orig_ptr);
        self.s.assemble(out);
    }

    /// The room walk. Starts are in `self.s.starts` (sorted, distinct labels of rows below
    /// n, phi(orig_ptr) among them); each is marked 0xFFFE (its field saved) and restored
    /// after the tail.
    ///
    /// What its unchecked accesses rely on:
    /// - **Labels:** a lane's row is a live label (a start, or a successor stored in the
    ///   records, every one a row below n: the constructor stores only φ of rows) or PARK.
    ///   So x & 31 < 30 and x >> 5 ≤ G: its field (byte 2x) and its record's header (byte
    ///   2(x & !31) + 60) are inside the records' `64(G + 1)` initialised bytes.
    /// - **RAW entries:** a RAW row's record (field 0xFFFF, not PARK) has header RAW_BIT |
    ///   idx with idx below the RAW groups' count (canonicalised), so its entry 32·idx +
    ///   (x & 31) is in the RAW area; PARK is tested before any RAW index is taken, and a
    ///   lane is parked only with `pend` set to such an entry.
    /// - **Output:** production's room walk's contract (`bwt::walk_segments_room_with_
    ///   prefetch`): Ownership, Spare, Handoffs, Initialised, Lifetimes. A waiting turn
    ///   writes nothing and steps its lane's base back one, so its offset's slot is the next
    ///   byte's.
    #[inline(never)]
    fn room(&mut self, orig_ptr: u32) {
        let n = self.n;
        let park = self.park();
        let ns = self.s.starts.len();
        // each start's (successor << 8) | F and its field, before marking
        self.succ.clear();
        self.saved.clear();
        for i in 0..ns {
            let x = self.s.starts[i];
            let (s, b) = self.next(x);
            self.succ.push(s << 8 | b as u32);
            self.saved
                .push(self.rec[(x >> 5) as usize].f[(x & 31) as usize]);
        }
        for i in 0..ns {
            let x = self.s.starts[i];
            self.rec[(x >> 5) as usize].f[(x & 31) as usize] = START;
        }
        let rec = self.rec.as_ptr() as *const u8;
        let raw = self.raw.as_ptr() as *const u32;
        // the physical-access audit, checked in debug builds: every field and header read
        // is inside the records' initialised storage, and every RAW entry read is inside
        // the RAW area
        let (rec_bytes, raw_entries) = (64 * self.rec.len(), 32 * self.raw.len());
        let in_rec = move |x: u32| {
            debug_assert!(x & 31 < 30, "label {x} on a header position");
            debug_assert!(
                2 * (x as usize) + 2 <= rec_bytes,
                "field of {x} past the records"
            );
            debug_assert!(
                2 * (x & !31) as usize + 64 <= rec_bytes,
                "header of {x} past the records"
            );
        };
        let s = &mut self.s;
        let succ = &self.succ;
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let starts = &s.starts;
        let seg_of = |x: u32| starts.binary_search(&x).unwrap_or(usize::MAX);
        let bufs = &mut s.bufs;
        let next = &mut s.next;

        // SAFETY (every record and RAW access below): a lane's row is a live label or
        // PARK, so its field, its record's header and any RAW index are in bounds (the
        // Labels and RAW entries conditions above)
        let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
            // SAFETY: the buffer has `cap` > 0 spare bytes
            unsafe { *base = succ[seg] as u8 };
            Lane {
                row: succ[seg] >> 8,
                pend: 0,
                seg,
                p: unsafe { base.add(1) },
                end: unsafe { base.add(b.capacity()) },
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
            // SAFETY: p is within b's allocation, every byte below it written; p and end
            // are taken again from the grown buffer
            let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
            unsafe { b.set_len(len) };
            b.reserve(b.capacity());
            l.p = unsafe { b.as_mut_ptr().add(len) };
            l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        };
        // the sentinel's three cases; START is tested first, PARK before any RAW index
        let slow = |l: &mut Lane, o: u16, h: u32| -> Slow {
            let x = l.row;
            if o == START {
                Slow::Start
            } else if x == park {
                debug_assert!(
                    (l.pend as usize) < raw_entries,
                    "a pending entry past the RAW area"
                );
                // SAFETY: pend was set from a RAW row's own header (below), in the RAW area
                let e = unsafe { *raw.add(l.pend as usize) };
                l.row = e >> 8;
                hit!(RESUMPTIONS);
                Slow::Byte(e as u8)
            } else {
                debug_assert!(h & RAW_BIT != 0 && o == RAWF);
                // a RAW row's own header, never the parking record's dummy index
                debug_assert!(x != park);
                l.pend = 32 * (h & M20) + (x & 31);
                debug_assert!(
                    (l.pend as usize) < raw_entries,
                    "a RAW index past the RAW area"
                );
                // SAFETY: pend is in the RAW area (RAW entries, above)
                prefetch(unsafe { raw.add(l.pend as usize) } as *const u8);
                l.row = park;
                hit!(RAW_ARRIVALS);
                Slow::Wait
            }
        };

        let mut queued = 0;
        let mut lanes: [Lane; LANES] = std::array::from_fn(|_| Lane {
            row: 0,
            pend: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        });
        let mut active = 0;
        while active < LANES && queued < ns {
            lanes[active] = take(queued, bufs);
            prefetch(unsafe { rec.add(2 * lanes[active].row as usize) });
            queued += 1;
            active += 1;
        }
        if active == LANES {
            'full: loop {
                // The pass's room: what every lane has spare, and no more than a buffer
                // taken mid-pass has after its first byte (`cap` - 1)
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        hit!(GROWS_FULL);
                        grow(l, bufs);
                    }
                    room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
                }
                debug_assert!(room > 0);
                let mut base: [*mut u8; LANES] = std::array::from_fn(|k| lanes[k].p);
                for off in 0..room {
                    for k in 0..LANES {
                        let l = &mut lanes[k];
                        let x = l.row;
                        in_rec(x);
                        // the ordinary step: field and header, both addressed from x
                        let o = unsafe { *(rec.add(2 * x as usize) as *const u16) };
                        let h = unsafe { *(rec.add(2 * (x & !31) as usize + 60) as *const u32) };
                        let byte = if o < START {
                            l.row = (h >> 8) + o as u32;
                            h as u8
                        } else {
                            match slow(l, o, h) {
                                Slow::Byte(b) => b,
                                Slow::Start => {
                                    l.p = base[k].wrapping_add(off);
                                    finish(l, bufs, next);
                                    if queued == ns {
                                        // the lanes before this one wrote at `off`, those
                                        // after it not yet
                                        for (j, lj) in lanes.iter_mut().enumerate() {
                                            if j < k {
                                                lj.p = base[j].wrapping_add(off + 1);
                                            } else if j > k {
                                                lj.p = base[j].wrapping_add(off);
                                            }
                                        }
                                        hit!(MIDROUND_ENDS);
                                        break 'full;
                                    }
                                    *l = take(queued, bufs);
                                    prefetch(unsafe { rec.add(2 * l.row as usize) });
                                    queued += 1;
                                    // its next byte, at offset off + 1, goes at its p
                                    base[k] = l.p.wrapping_sub(off + 1);
                                    hit!(REPLACEMENTS);
                                    continue;
                                }
                                Slow::Wait => {
                                    // nothing written this turn: its next byte takes
                                    // this offset's slot
                                    base[k] = base[k].wrapping_sub(1);
                                    hit!(WAIT_TURNS_FULL);
                                    continue;
                                }
                            }
                        };
                        prefetch(unsafe { rec.add(2 * l.row as usize) });
                        // SAFETY: base + off is below the lane's end: room is at most its
                        // spare at the pass's start (or, for a buffer taken mid-pass, its
                        // capacity after its first byte), and a waiting turn moved base
                        // back by the offset it did not write
                        debug_assert!(base[k].wrapping_add(off) < l.end);
                        unsafe { *base[k].wrapping_add(off) = byte };
                    }
                }
                for (l, b) in lanes.iter_mut().zip(base) {
                    l.p = b.wrapping_add(room);
                }
            }
        }
        let mut live: Vec<Lane> = lanes[..active]
            .iter()
            .copied()
            .filter(|l| l.seg != usize::MAX && next[l.seg] == usize::MAX)
            .collect();
        // A pass can end with a lane's buffer exactly full: the tail writes before it
        // tests, so each lane comes into it with a byte to spare
        for l in live.iter_mut() {
            if l.p == l.end {
                hit!(GROWS_TAIL);
                grow(l, bufs);
            }
            debug_assert!(l.p < l.end);
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let x = l.row;
                in_rec(x);
                let o = unsafe { *(rec.add(2 * x as usize) as *const u16) };
                let h = unsafe { *(rec.add(2 * (x & !31) as usize + 60) as *const u32) };
                let byte = if o < START {
                    l.row = (h >> 8) + o as u32;
                    h as u8
                } else {
                    match slow(l, o, h) {
                        Slow::Byte(b) => b,
                        Slow::Start => {
                            finish(l, bufs, next);
                            live.swap_remove(i);
                            continue;
                        }
                        Slow::Wait => {
                            hit!(WAIT_TURNS_TAIL);
                            i += 1;
                            continue;
                        }
                    }
                };
                prefetch(unsafe { rec.add(2 * l.row as usize) });
                // SAFETY: p < end (the tail's entry and the grow below)
                unsafe {
                    *l.p = byte;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    hit!(GROWS_TAIL);
                    grow(l, bufs);
                }
                i += 1;
            }
        }
        // the markers stay until every segment has ended; restore them now
        for i in 0..ns {
            let x = self.s.starts[i];
            self.rec[(x >> 5) as usize].f[(x & 31) as usize] = self.saved[i];
        }
        self.s.first = self
            .s
            .starts
            .binary_search(&phi(orig_ptr as usize))
            .expect("the first segment's start");
        self.s.n = n;
    }
}

// ---------------------------------------------------------------------------------------
// The kernel
// ---------------------------------------------------------------------------------------

/// The compact30 inverse BWT, the default: the records built from the last column
/// (`Ctor::build`), then walked (`C30::walk`: the room walk, or the plain walk below
/// `bwt::MIN_SEGMENTED` rows). Exactly `bwt::walk`'s output.
///
/// Where the records can't represent the block (more than `MAX_N` rows, or counts that
/// are not the last column's own, which the constructor refuses before publishing any
/// record), the block is inverted by production's one-core kernel, `bwt::Segmented`
/// (`bwt::build`, then its room walk), so its output, and its behaviour on counts no
/// decoder makes, are `Segmented`'s.
#[derive(Default)]
pub struct Compact30 {
    ctor: Ctor,
    c: C30,
    fallback: Segmented,
}

impl InverseBwt for Compact30 {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        if self
            .ctor
            .build(ll, counts, 0, &mut self.c.rec, &mut self.c.raw)
        {
            self.c.n = ll.len();
            self.c.walk(orig_ptr, out);
        } else {
            hit!(FALLBACKS);
            self.fallback.invert(ll, counts, orig_ptr, out);
        }
    }

    #[cfg(any(test, feature = "trace"))]
    fn buffers(&self) -> Vec<crate::trace::Buffer> {
        use crate::trace::Buffer;
        let s = &self.c.s;
        let mut v = vec![
            Buffer::of("c30.rec", &self.c.rec),
            Buffer::of("c30.raw", &self.c.raw),
            Buffer::of("c30.bitmap", &self.ctor.bitmap),
            Buffer::of("c30.snaps", &self.ctor.snaps),
        ];
        if s.bufs.capacity() > 0 {
            v.push(Buffer {
                name: "seg.bufs",
                len: s.bufs.iter().map(Vec::len).sum(),
                cap: s.bufs.iter().map(Vec::capacity).sum(),
            });
            v.push(Buffer::of("seg.bufs.outer", &s.bufs));
            v.push(Buffer::of("seg.starts", &s.starts));
            v.push(Buffer::of("seg.next", &s.next));
            v.push(Buffer::of("c30.saved", &self.c.saved));
            v.push(Buffer::of("c30.succ", &self.c.succ));
        }
        // the fallback's, once it has run (never, on a decoder's blocks)
        v.extend(
            self.fallback
                .buffers()
                .into_iter()
                .filter(|b| b.cap > 0)
                .map(|b| Buffer {
                    name: match b.name {
                        "tt" => "fallback.tt",
                        "seg.bufs" => "fallback.seg.bufs",
                        "seg.bufs.outer" => "fallback.seg.bufs.outer",
                        "seg.starts" => "fallback.seg.starts",
                        "seg.next" => "fallback.seg.next",
                        "seg.bitmap" => "fallback.seg.bitmap",
                        other => other,
                    },
                    ..b
                }),
        );
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bwt::{build, walk, TVector};

    fn counts_of(ll: &[u8]) -> [u32; 256] {
        let mut c = [0u32; 256];
        for &b in ll {
            c[b as usize] += 1;
        }
        c
    }

    /// The research oracle (`C30::build`), safe and slow: the records from Ψ and F, Ψ by a
    /// counting scatter as `bwt::build` orders it, each group classified (crossing, wide,
    /// forced, or ordinary), the RAW groups in increasing group order
    fn oracle(ll: &[u8], counts: &[u32; 256], permille: u32) -> (Vec<Rec>, Vec<RawG>) {
        let n = ll.len();
        let mut ftab = [0usize; 256];
        let mut cum = 0;
        for c in 0..256 {
            ftab[c] = cum;
            cum += counts[c] as usize;
        }
        assert_eq!(cum, n);
        let (mut psi, mut f) = (vec![0u32; n], vec![0u8; n]);
        for (i, &c) in ll.iter().enumerate() {
            let s = ftab[c as usize];
            psi[s] = phi(i);
            f[s] = c;
            ftab[c as usize] += 1;
        }
        let g_count = n.div_ceil(ROWS);
        assert!(32 * g_count as u32 <= M20, "PARK fits 20 bits");
        let (mut rec, mut raw) = (Vec::new(), Vec::new());
        for g in 0..g_count {
            let (a, e) = (g * ROWS, ((g + 1) * ROWS).min(n));
            let mut r = Rec {
                f: [RAWF; 30],
                h: 0,
            };
            for &x in &psi[a..e] {
                assert!(x & 31 < 30 && phi_inv(x) < n, "successor label {x}");
            }
            let ordinary = f[a..e].iter().all(|&x| x == f[a])
                && psi[e - 1] - psi[a] <= MAX_OFF
                && !forced(g, n, permille);
            if ordinary {
                let base = psi[a];
                r.h = base << 8 | f[a] as u32;
                for j in 0..e - a {
                    r.f[j] = (psi[a + j] - base) as u16;
                }
            } else {
                r.h = RAW_BIT | raw.len() as u32;
                let mut gr = RawG([0; 32]);
                for j in 0..e - a {
                    gr.0[j] = psi[a + j] << 8 | f[a + j] as u32;
                }
                raw.push(gr);
            }
            rec.push(r);
        }
        rec.push(Rec {
            f: [RAWF; 30],
            h: RAW_BIT | M20,
        });
        (rec, raw)
    }

    /// The segments' reference, safe and slow: from each start, a lane's own steps (the
    /// start's F and then each row's until the next start), each segment's bytes and its
    /// successor, on every cycle of the list
    fn segment_oracle(c: &C30) -> Vec<(Vec<u8>, usize)> {
        let st = &c.s.starts;
        let idx = |x: u32| st.binary_search(&x).ok();
        st.iter()
            .map(|&x| {
                let (mut y, b) = c.next(x);
                let mut bytes = vec![b];
                while idx(y).is_none() {
                    let (z, b) = c.next(y);
                    bytes.push(b);
                    y = z;
                }
                (bytes, idx(y).unwrap())
            })
            .collect()
    }

    /// The research walk's adversarial starts (`C30::starts` with `extra`), as row labels:
    /// fixed rows, 40 consecutive rows, chain-consecutive triples (one-byte segments), the
    /// first rows of RAW groups (RAW starts) and rows whose successor is RAW; with `m`
    /// random rows and the first
    fn adversarial_starts(c: &C30, orig_ptr: u32, m: usize) -> Vec<u32> {
        let n = c.n;
        let mut rows = Vec::new();
        random_rows(n, m, &mut rows);
        let mut st: Vec<u32> = rows.iter().map(|&r| phi(r as usize)).collect();
        st.push(phi(orig_ptr as usize));
        let n32 = n as u32;
        for r in [0, 1, n32 - 1, n32 / 2, n32 / 2 + 1, n32 / 3, n32 / 3 + 1] {
            st.push(phi(r.min(n32 - 1) as usize));
        }
        st.extend((0..40).map(|i| phi(((n32 / 5 + i).min(n32 - 1)) as usize)));
        for i in 1..9 {
            let mut x = phi(n * i / 9);
            for _ in 0..3 {
                st.push(x);
                x = c.next(x).0;
            }
        }
        let mut found = 0;
        for g in 0..c.rec.len() - 1 {
            if c.rec[g].h & RAW_BIT != 0 {
                st.push(32 * g as u32);
                if phi_inv(32 * g as u32 + 1) < n {
                    st.push(32 * g as u32 + 1);
                }
                found += 1;
                if found == 16 {
                    break;
                }
            }
        }
        let mut found = 0;
        for r in (0..n).step_by(7) {
            let x = phi(r);
            if c.is_raw(c.next(x).0) {
                st.push(x);
                found += 1;
                if found == 16 {
                    break;
                }
            }
        }
        st.sort_unstable();
        st.dedup();
        st
    }

    /// Starts along a long cycle of the list (the longest from a few rows'), with
    /// phi(orig_ptr): in each quarter of the cycle, one long segment of 100 rows, then a
    /// start every other row, so a long segment's lane fills its buffer (2n / ns + 64
    /// bytes) while the other lanes' short segments keep the full phase running for several
    /// of its passes (None if no cycle is long enough for that: under 2,500 rows; the full
    /// phase lasts about cycle / 31 rounds)
    fn chain_starts(c: &C30, orig_ptr: u32) -> Option<Vec<u32>> {
        let n = c.n;
        let cycle = |x0: u32| {
            let mut chain = vec![x0];
            let mut x = c.next(x0).0;
            while x != x0 && chain.len() < n {
                chain.push(x);
                x = c.next(x).0;
            }
            chain
        };
        let chain = [orig_ptr as usize, 0, n / 3, 2 * n / 3, n - 1]
            .into_iter()
            .map(|r| cycle(phi(r)))
            .max_by_key(Vec::len)
            .unwrap();
        if chain.len() < 2500 {
            return None;
        }
        let q = chain.len() / 4;
        let mut st: Vec<u32> = (0..chain.len())
            .filter(|&i| i % q == 0 || (i % q >= 100 && i % 2 == 0))
            .map(|i| chain[i])
            .collect();
        st.push(phi(orig_ptr as usize));
        st.sort_unstable();
        st.dedup();
        Some(st)
    }

    /// The room walk from the given starts (labels, phi(orig_ptr) among them), checked:
    /// its output against `want`, its segments against the reference, the markers restored
    fn room_from(c: &mut C30, orig_ptr: u32, starts: Vec<u32>, want: &[u8], what: &str) {
        let before = c.rec.clone();
        c.s.starts = starts;
        c.room(orig_ptr);
        assert!(c.rec == before, "{what}: a marker not restored");
        let mut out = Vec::new();
        c.s.assemble(&mut out);
        assert!(out == want, "{what}: the room walk differs from production");
        for (i, (bytes, next)) in segment_oracle(c).into_iter().enumerate() {
            assert!(c.s.bufs[i] == bytes, "{what}: segment {i}'s bytes");
            assert_eq!(c.s.next[i], next, "{what}: segment {i}'s successor");
        }
    }

    /// One block, every way: the records against the oracle's (at `permille` forced RAW),
    /// the walk (`C30::walk`, room or plain) and the room walk on any length (production's
    /// random starts, then the adversarial ones, and few segments, so buffers grow) against
    /// `Segmented`'s output; and `Compact30` itself
    fn check(k: &mut Compact30, ll: &[u8], orig_ptr: u32, permille: u32, what: &str) {
        let counts = counts_of(ll);
        let mut want = Vec::new();
        Segmented::default().invert(ll, &counts, orig_ptr, &mut want);
        let mut tt = TVector::new();
        build(ll, &counts, &mut tt);
        let mut plain = Vec::new();
        walk(&tt, orig_ptr, &mut plain);
        assert!(plain == want, "{what}: Segmented is not the plain walk");
        let what = format!(
            "{what} (n {}, orig_ptr {orig_ptr}, permille {permille})",
            ll.len()
        );
        assert!(
            k.ctor
                .build(ll, &counts, permille, &mut k.c.rec, &mut k.c.raw),
            "{what}: refused its own counts"
        );
        let (rec, raw) = oracle(ll, &counts, permille);
        assert!(k.c.rec == rec, "{what}: records differ from the oracle's");
        assert!(
            k.c.raw == raw,
            "{what}: RAW groups differ from the oracle's"
        );
        k.c.n = ll.len();
        let mut out = Vec::new();
        k.c.walk(orig_ptr, &mut out);
        assert!(out == want, "{what}: the walk differs from production");
        assert!(k.c.rec == rec, "{what}: a marker not restored");
        let mut st = Vec::new();
        random_rows(ll.len(), SEGMENTS - 1, &mut st);
        let mut st: Vec<u32> = st.iter().map(|&r| phi(r as usize)).collect();
        st.push(phi(orig_ptr as usize));
        st.sort_unstable();
        st.dedup();
        room_from(
            &mut k.c,
            orig_ptr,
            st,
            &want,
            &format!("{what}, random starts"),
        );
        for m in [SEGMENTS - 1, 40] {
            let st = adversarial_starts(&k.c, orig_ptr, m);
            room_from(
                &mut k.c,
                orig_ptr,
                st,
                &want,
                &format!("{what}, adversarial starts {m}"),
            );
        }
        if let Some(st) = chain_starts(&k.c, orig_ptr) {
            room_from(
                &mut k.c,
                orig_ptr,
                st,
                &want,
                &format!("{what}, chain starts"),
            );
        }
        if permille == 0 {
            k.invert(ll, &counts, orig_ptr, &mut out);
            assert!(out == want, "{what}: Compact30 differs from Segmented");
        }
    }

    fn rng(seed: u64) -> impl FnMut(u64) -> u64 {
        let mut x = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
        move |m: u64| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x % m.max(1)
        }
    }

    /// A column of n rows over `sigma` distinct bytes (spread over 0..=255), in runs of the
    /// given lengths
    fn column(rnd: &mut impl FnMut(u64) -> u64, n: usize, sigma: u64, runs: &[usize]) -> Vec<u8> {
        let mut v = Vec::with_capacity(n);
        while v.len() < n {
            let b = (rnd(sigma) * (256 / sigma)) as u8;
            let len = runs[rnd(runs.len() as u64) as usize].min(n - v.len());
            v.extend(std::iter::repeat_n(b, len));
        }
        v
    }

    fn assert_reached(names: &[(&str, &'static std::thread::LocalKey<std::cell::Cell<usize>>)]) {
        for (name, c) in names {
            assert!(hits::get(c) > 0, "never reached: {name}");
        }
    }

    const WALK_HITS: [(&str, &std::thread::LocalKey<std::cell::Cell<usize>>); 8] = [
        ("a lane arriving at a RAW row", &hits::RAW_ARRIVALS),
        ("a parked lane resuming", &hits::RESUMPTIONS),
        ("a waiting turn in the full phase", &hits::WAIT_TURNS_FULL),
        ("a waiting turn in the tail", &hits::WAIT_TURNS_TAIL),
        ("a segment taken mid-pass", &hits::REPLACEMENTS),
        ("a buffer grown at a pass's start", &hits::GROWS_FULL),
        ("a buffer grown in the tail", &hits::GROWS_TAIL),
        ("the full phase ending mid-pass", &hits::MIDROUND_ENDS),
    ];
    const BUILD_HITS: [(&str, &std::thread::LocalKey<std::cell::Cell<usize>>); 8] = [
        ("the leaf finish", &hits::LEAF),
        ("an ordinary slow finish", &hits::SLOW_ORDINARY),
        ("a preset RAW portion", &hits::PRESET_RAW),
        ("a forced RAW group", &hits::FORCED),
        ("complete groups", &hits::BULK),
        ("complete groups into RAW entries", &hits::BULK_RAW),
        ("a tile's second segment", &hits::TILES_SECOND),
        ("a tail", &hits::TAILS),
    ];

    /// Every column of n ≤ 9 rows over 2 bytes, n ≤ 6 over 3 and n ≤ 4 over 5 (the bytes
    /// spread, so their buckets cross groups or not), from every primary index, at no,
    /// some and all groups forced RAW
    #[test]
    fn exhaustive_small_columns() {
        let mut k = Compact30::default();
        for (sigma, max_n) in [(2u32, 9usize), (3, 6), (5, 4)] {
            for n in 1..=max_n {
                for code in 0..(sigma as usize).pow(n as u32) {
                    let mut x = code;
                    let ll: Vec<u8> = (0..n)
                        .map(|_| {
                            let d = x % sigma as usize;
                            x /= sigma as usize;
                            [0u8, 7, 31, 200, 255][d]
                        })
                        .collect();
                    for orig_ptr in 0..n as u32 {
                        for pm in [0, 1000] {
                            check(&mut k, &ll, orig_ptr, pm, "exhaustive");
                        }
                    }
                }
            }
        }
    }

    /// Random columns of every distinct-symbol regime (one byte, two, a few, every byte),
    /// in runs (whole groups, partial ones, runs past an epoch), at lengths across the
    /// partial-record, tile and epoch boundaries, at no, some and all groups forced RAW
    #[test]
    fn random_columns_every_regime() {
        let mut k = Compact30::default();
        let mut rnd = rng(0xc30);
        let lens = [
            1usize, 2, 15, 16, 17, 29, 30, 31, 59, 60, 61, 479, 480, 481, 1000, 4097, 10_000,
        ];
        let runs: [&[usize]; 3] = [&[1], &[1, 2, 15, 16, 17, 30, 31, 32, 33], &[1, 60, 61, 300]];
        for &n in &lens {
            for sigma in [1u64, 2, 9, 256] {
                for r in runs {
                    let ll = column(&mut rnd, n, sigma, r);
                    let orig_ptr = rnd(n as u64) as u32;
                    for pm in [0, 50, 1000] {
                        check(&mut k, &ll, orig_ptr, pm, &format!("sigma {sigma}"));
                    }
                }
            }
        }
        assert_reached(&WALK_HITS);
        assert_reached(&BUILD_HITS);
    }

    /// Blocks past MIN_SEGMENTED (the room walk) and past an epoch (61,440 rows: several
    /// epochs, portions received across them, wide groups), each regime, and runs of one
    /// byte long enough to span epochs; at the walk's threshold too
    #[test]
    fn long_columns() {
        let mut k = Compact30::default();
        let mut rnd = rng(0x5eed);
        for n in [
            MIN_SEGMENTED - 1,
            MIN_SEGMENTED,
            MIN_SEGMENTED + 1,
            EPOCH - 1,
            EPOCH,
            EPOCH + 1,
            2 * EPOCH + 17,
            300_001,
        ] {
            for (sigma, runs) in [
                (1u64, &[1usize][..]),
                (2, &[1, 2, 3][..]),
                (9, &[1, 16, 33, 1000][..]),
                (256, &[1][..]),
                (3, &[70_000, 1][..]),
            ] {
                let ll = column(&mut rnd, n, sigma, runs);
                let orig_ptr = rnd(n as u64) as u32;
                check(&mut k, &ll, orig_ptr, 0, &format!("long, sigma {sigma}"));
            }
        }
        assert!(hits::get(&hits::WIDE) > 0, "never reached: a wide group");
        assert!(
            hits::get(&hits::SLOW_EPOCH_SEARCH) > 0,
            "never reached: an epoch search"
        );
        assert_reached(&WALK_HITS);
    }

    /// The research's span cases at an epoch boundary: 30 rows of byte 1 whose successor
    /// labels span exactly 65,533 (ordinary) and 65,534 (RAW, a wide group)
    #[test]
    fn spans_at_the_limit() {
        let mut k = Compact30::default();
        for (first, wide) in [(3usize, false), (2, true)] {
            let n = EPOCH + 200;
            let mut ll: Vec<u8> = (0..n).map(|i| 2 + (i & 1) as u8).collect();
            for t in 0..29 {
                ll[first + 2 * t] = 1;
            }
            ll[EPOCH] = 1;
            let before = hits::get(&hits::WIDE);
            check(&mut k, &ll, 17, 0, &format!("span from {first}"));
            check(
                &mut k,
                &ll,
                (n - 1) as u32,
                0,
                &format!("span from {first}"),
            );
            assert_eq!(hits::get(&hits::WIDE) > before, wide, "span from {first}");
        }
    }

    /// Periodic columns (lists of several cycles: walks joined round again), one byte
    /// repeated, every byte cycled, and the bzip2 block's largest length
    #[test]
    fn periodic_and_full_blocks() {
        let mut k = Compact30::default();
        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("zeros", vec![0u8; 200_000]),
            ("ab", b"ab".repeat(300_000)),
            ("abc", b"abc".repeat(300_000)),
            ("period 30", (0..30u8).cycle().take(900_000).collect()),
            ("period 31", (0..31u8).cycle().take(900_000).collect()),
            ("all bytes", (0..=255u8).cycle().take(899_981).collect()),
        ];
        for (name, ll) in cases {
            let n = ll.len() as u32;
            for orig_ptr in [0, 1, n / 2, n - 1] {
                let counts = counts_of(&ll);
                let (mut want, mut out) = (Vec::new(), Vec::new());
                Segmented::default().invert(&ll, &counts, orig_ptr, &mut want);
                k.invert(&ll, &counts, orig_ptr, &mut out);
                assert!(out == want, "{name}, orig_ptr {orig_ptr}");
            }
        }
        // a random block of the format's largest length, through the full check
        let mut rnd = rng(9);
        let ll: Vec<u8> = (0..900_000).map(|_| rnd(256) as u8).collect();
        check(&mut k, &ll, 12_345, 0, "random 900,000");
    }

    /// An inversion's outcome: its output, or None if it panicked
    fn outcome(inv: &mut dyn FnMut(&mut Vec<u8>)) -> Option<Vec<u8>> {
        let mut v = Vec::new();
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| inv(&mut v)))
            .map(|()| v)
            .ok()
    }

    /// The fallbacks: a block of more than MAX_N rows (MAX_N + 1, 2^20, 2^23, 2^24 − 1, as
    /// `bwt::build` takes any length below 2^24) and counts that are not the column's own
    /// (short; one moved from the first byte to the last), each with Segmented's outcome
    /// (its output; or its panic, which its room walk can reach on such counts, and past
    /// 2^23 rows, where a link's bit 31 is its start flag: no bzip2 block is near either);
    /// the kernel used again after each on a block it can represent
    #[test]
    fn fallbacks_are_segmented() {
        let mut k = Compact30::default();
        let mut rnd = rng(77);
        let small: Vec<u8> = (0..70_000).map(|_| rnd(5) as u8).collect();
        let again = |k: &mut Compact30| {
            let counts = counts_of(&small);
            let (mut want, mut out) = (Vec::new(), Vec::new());
            Segmented::default().invert(&small, &counts, 5, &mut want);
            let before = hits::get(&hits::FALLBACKS);
            k.invert(&small, &counts, 5, &mut out);
            assert_eq!(
                hits::get(&hits::FALLBACKS),
                before,
                "a representable block fell back"
            );
            assert!(out == want, "after a fallback");
        };
        assert!(k.ctor.build(&[], &[0; 256], 0, &mut k.c.rec, &mut k.c.raw));
        assert_eq!(
            (k.c.rec.len(), k.c.raw.len()),
            (1, 0),
            "the empty block: PARK alone"
        );
        for n in [MAX_N - 1, MAX_N, MAX_N + 1, 1 << 20, 1 << 23, (1 << 24) - 1] {
            let ll: Vec<u8> = (0..n)
                .map(|i| (rnd(3) as u8) ^ (i % 7 == 0) as u8)
                .collect();
            let counts = counts_of(&ll);
            let orig_ptr = (n / 3) as u32;
            let mut seg = Segmented::default();
            let want = outcome(&mut |v| seg.invert(&ll, &counts, orig_ptr, v));
            let before = hits::get(&hits::FALLBACKS);
            let got = outcome(&mut |v| k.invert(&ll, &counts, orig_ptr, v));
            assert!(got == want, "n {n}");
            assert!(want.is_some() || n > 1 << 23, "n {n}: Segmented panicked");
            assert_eq!(
                hits::get(&hits::FALLBACKS) - before,
                (n > MAX_N) as usize,
                "n {n}"
            );
            again(&mut k);
        }
        for ll in [
            small.clone(),
            vec![4u8; 1000],
            (0..3000).map(|i| (i % 3) as u8).collect(),
        ] {
            let counts = counts_of(&ll);
            let present: Vec<usize> = (0..256).filter(|&c| counts[c] > 0).collect();
            let (lo, hi) = (present[0], *present.last().unwrap());
            let mut short = counts;
            short[hi] -= 1;
            let mut cases = vec![short];
            if lo != hi {
                let mut moved = counts;
                moved[lo] -= 1;
                moved[hi] += 1;
                cases.push(moved);
            }
            for wrong in cases {
                assert!(
                    !k.ctor.build(&ll, &wrong, 0, &mut k.c.rec, &mut k.c.raw),
                    "wrong counts accepted"
                );
                assert!(
                    k.c.rec.is_empty() && k.c.raw.is_empty(),
                    "a refused build publishes"
                );
                let mut seg = Segmented::default();
                let want = outcome(&mut |v| seg.invert(&ll, &wrong, 1, v));
                let before = hits::get(&hits::FALLBACKS);
                let got = outcome(&mut |v| k.invert(&ll, &wrong, 1, v));
                assert_eq!(hits::get(&hits::FALLBACKS) - before, 1);
                assert!(got == want, "wrong counts: not Segmented's outcome");
                assert!(
                    want.is_some() || ll.len() >= MIN_SEGMENTED,
                    "the plain walk is defined on any table"
                );
                again(&mut k);
            }
        }
    }

    /// One kernel through blocks of varying lengths in any order (its buffers kept from
    /// block to block: shorter after longer, RAW-heavy after RAW-free)
    #[test]
    fn reused_across_blocks() {
        let mut k = Compact30::default();
        let mut rnd = rng(3);
        for round in 0..12 {
            let n = [900_000usize, 70_000, 31, 400_000, 65_536, 1][round % 6];
            let sigma = [256u64, 2, 1, 40][round % 4];
            let ll = column(&mut rnd, n, sigma, &[1, 3, 40]);
            let counts = counts_of(&ll);
            let orig_ptr = rnd(n as u64) as u32;
            let (mut want, mut out) = (Vec::new(), Vec::new());
            Segmented::default().invert(&ll, &counts, orig_ptr, &mut want);
            k.invert(&ll, &counts, orig_ptr, &mut out);
            assert!(out == want, "round {round}, n {n}");
        }
    }

    /// `check` cut to what Miri needs, so that it interprets in minutes: the records
    /// against the oracle's, the walk (plain, below MIN_SEGMENTED) and the room walk from
    /// adversarial starts (40 random ones beside them) against the plain T-vector walk, the
    /// segments against their reference, the markers restored; and `Compact30` itself
    fn check_miri(k: &mut Compact30, ll: &[u8], orig_ptr: u32, permille: u32, what: &str) {
        let counts = counts_of(ll);
        let mut tt = TVector::new();
        build(ll, &counts, &mut tt);
        let mut want = Vec::new();
        walk(&tt, orig_ptr, &mut want);
        let what = format!(
            "{what} (n {}, orig_ptr {orig_ptr}, permille {permille})",
            ll.len()
        );
        assert!(k
            .ctor
            .build(ll, &counts, permille, &mut k.c.rec, &mut k.c.raw));
        let (rec, raw) = oracle(ll, &counts, permille);
        assert!(
            k.c.rec == rec && k.c.raw == raw,
            "{what}: records differ from the oracle's"
        );
        k.c.n = ll.len();
        let mut out = Vec::new();
        k.c.walk(orig_ptr, &mut out);
        assert!(out == want, "{what}: the walk differs from production");
        let st = adversarial_starts(&k.c, orig_ptr, 40);
        room_from(
            &mut k.c,
            orig_ptr,
            st,
            &want,
            &format!("{what}, adversarial starts"),
        );
        if permille == 0 {
            k.invert(ll, &counts, orig_ptr, &mut out);
            assert!(out == want, "{what}: Compact30 differs from the plain walk");
        }
    }

    /// The constructor's unsafe writes, U2, canonicalise and the room walk's unchecked
    /// loads and writes under Miri, on small columns: accepted (each regime, partial
    /// groups, crossing buckets, forced RAW groups, every tile phase), refused before the
    /// scatter and mid-scatter, and the walk from adversarial starts (RAW starts, one-byte
    /// segments, lanes parked at RAW rows in the full phase and the tail); each transition
    /// asserted reached (a buffer's growth: `miri_compact30_grows`)
    #[test]
    fn miri_compact30_small() {
        let mut k = Compact30::default();
        let mut rnd = rng(0x313);
        for n in [1usize, 2, 16, 17, 31, 33, 61, 95] {
            for sigma in [1u64, 2, 3, 256] {
                let ll = column(&mut rnd, n, sigma, &[1, 2, 16, 17, 31]);
                for pm in [0, 1000] {
                    check_miri(&mut k, &ll, rnd(n as u64) as u32, pm, "miri small");
                }
            }
        }
        for (n, sigma, runs) in [
            (700usize, 2u64, &[1usize, 31, 61][..]),
            (900, 256, &[1][..]),
            (1300, 5, &[1, 2, 30][..]),
        ] {
            let ll = column(&mut rnd, n, sigma, runs);
            check_miri(&mut k, &ll, rnd(n as u64) as u32, 50, "miri walk");
        }
        // refused: counts not summing to n (before the scatter), and an overflowing bucket
        let ll = column(&mut rnd, 200, 3, &[1, 16, 17]);
        let counts = counts_of(&ll);
        let present: Vec<usize> = (0..256).filter(|&c| counts[c] > 0).collect();
        let (lo, hi) = (present[0], *present.last().unwrap());
        let (mut short, mut moved) = (counts, counts);
        short[hi] -= 1;
        moved[lo] -= 1;
        moved[hi] += 1;
        for wrong in [short, moved] {
            assert!(!k.ctor.build(&ll, &wrong, 0, &mut k.c.rec, &mut k.c.raw));
            assert!(k.c.rec.is_empty() && k.c.raw.is_empty());
        }
        let grows = [
            "a buffer grown at a pass's start",
            "a buffer grown in the tail",
        ];
        let walk_hits: Vec<_> = WALK_HITS
            .into_iter()
            .filter(|(name, _)| !grows.contains(name))
            .collect();
        assert_reached(&walk_hits);
        assert_reached(&BUILD_HITS);
    }

    /// The room walk's buffer growth under Miri: starts along a long cycle of a 10,000-row
    /// block (`chain_starts`), so lanes fill their buffers in the full phase and the tail
    #[test]
    fn miri_compact30_grows() {
        let mut k = Compact30::default();
        let mut rnd = rng(0x6e0);
        let ll = column(&mut rnd, 10_000, 256, &[1]);
        let counts = counts_of(&ll);
        let mut tt = TVector::new();
        build(&ll, &counts, &mut tt);
        let mut want = Vec::new();
        walk(&tt, 4321, &mut want);
        assert!(k.ctor.build(&ll, &counts, 0, &mut k.c.rec, &mut k.c.raw));
        k.c.n = ll.len();
        let st = chain_starts(&k.c, 4321).expect("a long cycle");
        room_from(&mut k.c, 4321, st, &want, "miri grows");
        assert_reached(&[
            ("a buffer grown at a pass's start", &hits::GROWS_FULL),
            ("a buffer grown in the tail", &hits::GROWS_TAIL),
            ("a segment taken mid-pass", &hits::REPLACEMENTS),
            ("the full phase ending mid-pass", &hits::MIDROUND_ENDS),
        ]);
    }

    /// The epoch paths under Miri: a block of two epochs whose byte-1 portion is received
    /// across the boundary (a slow finish with an epoch search, and a wide group)
    #[test]
    fn miri_compact30_epochs() {
        let mut k = Compact30::default();
        let n = EPOCH + 200;
        let mut ll: Vec<u8> = (0..n).map(|i| 2 + (i & 1) as u8).collect();
        for t in 0..29 {
            ll[2 + 2 * t] = 1;
        }
        ll[EPOCH] = 1;
        let counts = counts_of(&ll);
        assert!(k.ctor.build(&ll, &counts, 0, &mut k.c.rec, &mut k.c.raw));
        let (rec, raw) = oracle(&ll, &counts, 0);
        assert!(
            k.c.rec == rec && k.c.raw == raw,
            "records differ from the oracle's"
        );
        assert!(hits::get(&hits::WIDE) > 0, "never reached: a wide group");
        assert!(
            hits::get(&hits::SLOW_EPOCH_SEARCH) > 0,
            "never reached: an epoch search"
        );
    }
}
