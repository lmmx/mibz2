//! The compact30 inverse BWT (`Compact30`): the default kernel. A block's successor list is
//! held as one 64-byte record per 30 rows instead of the T-vector's 4 bytes per row, built
//! straight from the last column in one scan, then walked by production's staged room walk
//! (`bwt::room_segments`'s busy phase) with the records' step in place of the T-vector's.
//! Exactly `bwt::walk`'s output.
//!
//! Ported from the research branch `research/compact30-combined` at 888eccf: the records,
//! labels and walk from `examples/c30walk.rs` (`C30::walk::<false>`, `plain`, the tail of
//! `room::<false>`), and the constructor from `examples/c30walk/ctor2c.rs`
//! (`Ctor2c::build::<false>`: constructor v2 with the scanner's U2 tile, the split
//! descriptor arrays and v3's epoch-tagged finaliser), with `ctor.rs`'s `canonicalise`. The
//! counted builds, their statistics and the research harnesses are left out; the hot paths
//! are otherwise the research code's, unchanged (the instruction ledger was measured on their
//! compiled form: docs/journal/2026-11-04-compact30-combined-build.md). The busy phase is
//! master's staged one (its tile, chunks, deferred handovers and exhaustion), with the
//! records' step and one more turn class, the resolve (below).
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
//! - **Parking:** one record of 0xFFFF fields after the last, at label PARK = 32·G (the
//!   tail's only).
//!
//! **The split layout** (`Compact30Split`, `S = true`, the layout control): the same
//! fields, headers and RAW groups, placed apart in the same allocation of 64·(G + 1)
//! bytes: label x = 32g + j has its field at byte 60g + 2j (= 2·row) and group g's header
//! at 60·(G + 1) + 4g. Only the address arithmetic (`fld`, `hdr`, `pf`) and the
//! constructor's eight record writes differ; no `Rec` is formed from its bytes.
//!
//! **The emission convention:** `F[r] = L[Ψ(r)]`, so `walk`'s output `L[Ψ(p)], L[Ψ²(p)], …`
//! from orig_ptr = p equals `F[p], F[Ψ(p)], …`. A lane at row x emits `F[x]` and moves to Ψ(x);
//! the first segment starts at phi(orig_ptr).
//!
//! **The staged walk's RAW rows:** a lane that reaches a RAW row (field 0xFFFF) at offset j
//! saves the entry's index, prefetches it and emits nothing in its sweep turn (the port's
//! arrival); after every lane has stepped offset j, and before any handover, the offset's
//! resolve reads each such entry and stages its byte at column j, the column the arrival
//! left empty, and its successor as the lane's row (the port's resume, moved from the
//! lane's next turn to its own offset). So every lane again accounts for one byte per
//! offset, or a start mark, and master's flush ranges hold unchanged; nothing parked
//! crosses an offset. The tail, unstaged, parks and resumes as the port did.
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
//!   its tile and output writes, under production's room-walk contract (`bwt`, Ownership,
//!   Spare, Handoffs, Initialised, Lifetimes).

use crate::bwt::{
    self, random_rows, InverseBwt, SegScratch, Segmented, MIN_SEGMENTED, PF_L1, TILE,
};
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

// The walk's addressing, the one place the layout (`S`) appears in the walk. `rec` is the
// records' first byte; `hb` is the split layout's first header, `rec` + 60·(G + 1) (unused
// by the record layout). In the record layout (S = false) label x's field is at byte 2x
// and its record's header at 2(x & !31) + 60; in the split layout its field is at
// 2x − 4(x >> 5) (60g + 2j for x = 32g + j) and its group's header at hb + 4(x >> 5). Both
// are addressed from x alone (never from the field).

/// Label x's field. SAFETY (the callers'): x is a live label or PARK, so the field is
/// inside the records' initialised bytes
#[inline(always)]
unsafe fn fld<const S: bool>(rec: *const u8, x: u32) -> *const u16 {
    let x = x as usize;
    if S {
        rec.add(2 * x - 4 * (x >> 5)) as *const u16
    } else {
        rec.add(2 * x) as *const u16
    }
}

/// Label x's header. SAFETY: as `fld`
#[inline(always)]
unsafe fn hdr<const S: bool>(rec: *const u8, hb: *const u8, x: u32) -> *const u32 {
    let x = x as usize;
    if S {
        hb.add(4 * (x >> 5)) as *const u32
    } else {
        rec.add(2 * (x & !31) + 60) as *const u32
    }
}

/// The lines label s's step reads, prefetched: its record's (field and header share it), or
/// in the split layout its field's and its header's. A hint only, at addresses formed with
/// wrapping arithmetic (s is a live label all the same)
#[inline(always)]
fn pf<const S: bool>(rec: *const u8, hb: *const u8, s: u32) {
    let x = s as usize;
    if S {
        prefetch(rec.wrapping_add(2 * x - 4 * (x >> 5)));
        prefetch(hb.wrapping_add(4 * (x >> 5)));
    } else {
        prefetch(rec.wrapping_add(2 * x));
    }
}

/// Marks the branch it is called from as cold (rustc 1.80's way, with no
/// `std::hint::cold_path`): the step's sentinel arm, so that the ordinary step stays
/// straight-line
#[cold]
#[inline(always)]
fn cold() {}

// Counts of the rarer transitions, in test builds only (per thread, so each test reads its
// own), so a test can show it reached them; nothing in other builds
#[cfg(test)]
pub(crate) mod hits {
    use std::cell::Cell;
    macro_rules! counters {
        ($($c:ident),* $(,)?) => {
            thread_local! {
                $(pub static $c: Cell<usize> = const { Cell::new(0) };)*
            }
            /// Every counter, by name, for the C/L differential
            pub fn all() -> Vec<(&'static str, &'static std::thread::LocalKey<Cell<usize>>)> {
                vec![$((stringify!($c), &$c)),*]
            }
        };
    }
    counters! {
        // the tail (the port's slow closure): arrivals, resumptions and waiting turns
        RAW_ARRIVALS,
        RESUMPTIONS,
        WAIT_TURNS_TAIL,
        GROWS_FULL,
        GROWS_TAIL,
        // the build
        LEAF,
        SLOW_ORDINARY,
        SLOW_EPOCH_SEARCH,
        WIDE,
        PRESET_RAW,
        FORCED,
        BULK,
        BULK_RAW,
        TILES_SECOND,
        TAILS,
        FALLBACKS,
        // master's staged busy phase (bwt.rs's, the same events)
        PASS_ENDS_FULL,
        MID_PASS_TAKES,
        FULL_PHASE_ENDS_MID_PASS,
        TAIL_ENTERED_FULL,
        MARKED_SEVERAL,
        ENDS_WITH_LANES_WAITING,
        ENDS_AFTER_A_TAKE,
        LATER_STEPPED_FULL,
        LATER_STEPPED_ON_START,
        ROOM_OVER_TILE,
        TAKE_AT_CHUNK_END,
        TAKE_IN_LATER_CHUNK,
        DONE_IN_TAKING_CHUNK,
        MARK_AT_LATER_CHUNK_START,
        EXHAUSTED_AT_CHUNK_START,
        EXHAUSTED_AT_CHUNK_END,
        EXHAUSTED_AT_PASS_END,
        // the busy phase's RAW rows: an arrival and its resolve; several lanes arriving at
        // one offset; an arrival and a mark at one offset; an arrival at a later chunk's
        // first offset, at a chunk's last short of the pass's end, at the pass's last; at
        // the offset the queue runs out; a lane's first step after a take; a resolved
        // successor that is a start (marked at the next offset); a lane resolved at two
        // consecutive offsets; a start whose own record is RAW (its saved entry)
        RAW_ARRIVALS_FULL,
        RESOLVES,
        RAW_SEVERAL,
        RAW_AND_MARK,
        RAW_AT_CHUNK_START,
        RAW_AT_CHUNK_END,
        RAW_AT_PASS_END,
        RAW_AT_EXHAUSTION,
        RAW_AFTER_TAKE,
        RAW_THEN_START,
        RAW_CHAIN,
        RAW_START,
    }
    pub fn get(c: &'static std::thread::LocalKey<Cell<usize>>) -> usize {
        c.with(Cell::get)
    }
    /// Every counter's value now, in `all`'s order
    pub fn snapshot() -> Vec<usize> {
        all().iter().map(|(_, c)| get(c)).collect()
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

/// One build's state, for the record layout (S = false) or the split one (S = true)
struct W<'a, const S: bool> {
    /// the record arena: `g_count + 1` records of capacity, none yet initialised
    dst: *mut Rec,
    /// the split layout's first header, byte 60·(g_count + 1) of the arena (unused by the
    /// record layout)
    hb: *mut u8,
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

impl<const S: bool> W<'_, S> {
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
    /// bytes). In the split layout, group g's fields are bytes [60g, 60g + 60), so rank r's
    /// is byte 2r
    #[inline(always)]
    unsafe fn field(&self, r: u32) -> *mut u16 {
        let (g, j) = (r as usize / ROWS, r as usize % ROWS);
        debug_assert!(g < self.g_count);
        if S {
            (self.dst as *mut u8).add(2 * r as usize) as *mut u16
        } else {
            (ptr::addr_of_mut!((*self.dst.add(g)).f) as *mut u16).add(j)
        }
    }

    /// Group g's header (g ≤ g_count): byte 60 of its record, or in the split layout
    /// hb + 4g. SAFETY: g ≤ g_count, inside the arena's capacity
    #[inline(always)]
    unsafe fn hdr_g(&self, g: usize) -> *mut u32 {
        debug_assert!(g <= self.g_count);
        if S {
            self.hb.add(4 * g) as *mut u32
        } else {
            ptr::addr_of_mut!((*self.dst.add(g)).h)
        }
    }

    /// Group g's 30 fields, as one array (2-aligned in both layouts). SAFETY: as `hdr_g`
    #[inline(always)]
    unsafe fn fields_g(&self, g: usize) -> *mut [u16; 30] {
        debug_assert!(g <= self.g_count);
        if S {
            (self.dst as *mut u8).add(60 * g) as *mut [u16; 30]
        } else {
            ptr::addr_of_mut!((*self.dst.add(g)).f)
        }
    }

    /// Group g's whole record written: its fields and its header (in the split layout, two
    /// writes). SAFETY: as `hdr_g`
    #[inline(always)]
    unsafe fn write_rec(&self, g: usize, r: Rec) {
        if S {
            self.fields_g(g).write(r.f);
            self.hdr_g(g).write(r.h);
        } else {
            self.dst.add(g).write(r);
        }
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
        self.write_rec(
            g,
            Rec {
                f: [RAWF; 30],
                h: RAW_BIT | idx,
            },
        );
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
            self.hdr_g(g).read() & M20
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
            // SAFETY: the record's header, at byte 60 of the record whose field 0 is p (in
            // the split layout, its group's header)
            let hp = if S {
                self.hdr_g(k.g as usize)
            } else {
                (p as *mut u8).add(60).cast::<u32>()
            };
            hp.write((tag & !ORD) | (u0 as u32) << 8 | c as u32);
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
                self.hdr_g(g).write(y0 << 8 | c as u32);
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
                self.write_rec(
                    g,
                    Rec {
                        f: [RAWF; 30],
                        h: RAW_BIT | idx as u32,
                    },
                );
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
            if S {
                for t in 0..whole {
                    self.fields_g(g + t).write(r.f);
                    self.hdr_g(g + t).write(r.h);
                    r.h += 32 << 8;
                }
            } else {
                let d = self.dst.add(g);
                for t in 0..whole {
                    d.add(t).write(r);
                    r.h += 32 << 8;
                }
            }
            self.y += 32 * whole as u32;
            return;
        }
        for t in 0..whole {
            let gt = g + t;
            if self.is_raw(gt) {
                hit!(BULK_RAW);
                let idx = (self.hdr_g(gt).read() & M20) as usize;
                let pat = &PATTERNS[self.p].f;
                let y = self.y;
                for (e, &o) in self.raw[idx].0[..ROWS].iter_mut().zip(pat) {
                    *e = (y + o as u32) << 8 | c as u32;
                }
            } else {
                let mut r = PATTERNS[self.p];
                r.h = self.y << 8 | c as u32;
                self.write_rec(gt, r);
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
    /// are not `ll`'s own). In the record layout (S = false) or the split one (S = true):
    /// the same values, placed apart.
    #[inline(never)]
    fn build<const S: bool>(
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
            let dst = rec.as_mut_ptr();
            let mut w = W::<S> {
                dst,
                // (inside the capacity's 64·(g_count + 1) bytes)
                hb: (dst as *mut u8).wrapping_add(60 * (g_count + 1)),
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
            let hb = (dst as *mut u8).add(60 * (g_count + 1));
            let used = n - ROWS * g_count.saturating_sub(1);
            if g_count > 0 && used < ROWS {
                let f = if S {
                    (dst as *mut u8).add(60 * (g_count - 1)) as *mut u16
                } else {
                    ptr::addr_of_mut!((*dst.add(g_count - 1)).f) as *mut u16
                };
                for t in used..ROWS {
                    f.add(t).write(RAWF);
                }
            }
            let park = Rec {
                f: [RAWF; 30],
                h: RAW_BIT | M20,
            };
            if S {
                ((dst as *mut u8).add(60 * g_count) as *mut [u16; 30]).write(park.f);
                (hb.add(4 * g_count) as *mut u32).write(park.h);
            } else {
                dst.add(g_count).write(park);
            }
            let _ = canonicalise::<S>(dst, hb, bitmap, raw);
            rec.set_len(g_count + 1);
        }
        true
    }

    /// The preset RAW groups, then the scan of L epoch by epoch, then every pending
    /// portion finished. False at the first overflowing bucket (before its write) or if
    /// any bucket ends short.
    #[inline(always)]
    unsafe fn scatter<const S: bool>(
        w: &mut W<'_, S>,
        ll: &[u8],
        counts: &[u32; 256],
        permille: u32,
    ) -> bool {
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
/// every RAW group is zero. The headers are the records' (S = false) or the split layout's
/// from `hb` (S = true).
unsafe fn canonicalise<const S: bool>(
    dst: *mut Rec,
    hb: *mut u8,
    bitmap: &[u64],
    raw: &mut [RawG],
) -> (u64, u64, u64) {
    let mut rank = 0u32;
    for (w, &word) in bitmap.iter().enumerate() {
        let mut b = word;
        while b != 0 {
            let g = 64 * w + b.trailing_zeros() as usize;
            b &= b - 1;
            let hp = if S {
                hb.add(4 * g) as *mut u32
            } else {
                ptr::addr_of_mut!((*dst.add(g)).h)
            };
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
// The walk (research `C30::walk::<false>`, `plain` and `room::<false>`'s preamble and tail;
// its busy phase is master's staged one, `bwt::room_segments`, with the records' step)
// ---------------------------------------------------------------------------------------

/// A lane: its encoded row, the RAW entry it waits on (in the tail), its segment and
/// output pointers (32 bytes)
#[derive(Clone, Copy)]
struct Lane {
    row: u32,
    pend: u32,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

const _: () = assert!(std::mem::size_of::<Lane>() == 32);
// An offset's events in one u64: the lanes marked at a start in bits 0-31, the lanes
// arriving at a RAW row in bits 32-63; and the sweep unrolled 8 lanes a turn
const _: () = assert!(LANES == 32 && LANES % 8 == 0);

/// What a slow step found (the tail's)
enum Slow {
    Byte(u8),
    Start,
    Wait,
}

/// The records, the RAW groups and the walk's scratch, in the record layout (S = false) or
/// the split one (S = true)
#[derive(Default)]
struct C30<const S: bool> {
    n: usize,
    rec: Vec<Rec>,
    raw: Vec<RawG>,
    s: SegScratch,
    /// the marked starts' saved fields, and their (successor << 8) | F
    saved: Vec<u16>,
    succ: Vec<u32>,
}

impl<const S: bool> C30<S> {
    fn park(&self) -> u32 {
        32 * (self.rec.len() as u32 - 1)
    }

    /// The records' bytes (the split layout's checked reads): all 64·len are initialised,
    /// and `Rec` has no padding
    fn bytes(&self) -> &[u8] {
        // SAFETY: as above; u8 has no invalid values
        unsafe { std::slice::from_raw_parts(self.rec.as_ptr().cast::<u8>(), 64 * self.rec.len()) }
    }

    /// Label x's field (checked: x's group is a record, x & 31 < 30)
    fn field(&self, x: u32) -> u16 {
        let (g, j) = ((x >> 5) as usize, (x & 31) as usize);
        if S {
            assert!(g < self.rec.len() && j < ROWS, "label {x}");
            let (o, b) = (60 * g + 2 * j, self.bytes());
            u16::from_ne_bytes([b[o], b[o + 1]])
        } else {
            self.rec[g].f[j]
        }
    }

    /// Label x's field set to v (checked as `field`)
    fn set_field(&mut self, x: u32, v: u16) {
        let (g, j) = ((x >> 5) as usize, (x & 31) as usize);
        if S {
            assert!(g < self.rec.len() && j < ROWS, "label {x}");
            let o = 60 * g + 2 * j;
            // SAFETY: o + 2 ≤ 60·len ≤ 64·len, inside the records' bytes, 2-aligned (the
            // records are 64-aligned); the arena is written as u16 only
            unsafe { (self.rec.as_mut_ptr().cast::<u8>().add(o) as *mut u16).write(v) };
        } else {
            self.rec[g].f[j] = v;
        }
    }

    /// Label x's header (checked: x's group is a record)
    fn header(&self, x: u32) -> u32 {
        let g = (x >> 5) as usize;
        if S {
            assert!(g < self.rec.len(), "label {x}");
            let (o, b) = (60 * self.rec.len() + 4 * g, self.bytes());
            u32::from_ne_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
        } else {
            self.rec[g].h
        }
    }

    /// Ψ(x) and `F[x]` from the records, no start marked (checked indexing)
    fn next(&self, x: u32) -> (u32, u8) {
        let (o, h) = (self.field(x), self.header(x));
        if o < START {
            ((h >> 8) + o as u32, h as u8)
        } else {
            debug_assert!(h & RAW_BIT != 0);
            let e = self.raw[(h & M20) as usize].0[(x & 31) as usize];
            (e >> 8, e as u8)
        }
    }

    #[cfg(test)]
    fn is_raw(&self, x: u32) -> bool {
        self.field(x) == RAWF
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
    /// While all 32 lanes are busy it is master's staged busy phase (`bwt::room_segments`:
    /// passes of `room` offsets, cut into chunks of at most TILE; at offset j every lane
    /// steps, staging its byte at tile[k][j] or, at a start, being marked; then the marked
    /// lanes are handed over in lane order), with two changes: the step is the records'
    /// (two loads, field and header, both addressed from the label), and a lane that
    /// arrives at a RAW row is resolved at its own offset (below), so that every lane still
    /// accounts for exactly one byte per offset, or a mark, and master's flush ranges hold.
    /// The tail is the port's, unstaged.
    ///
    /// What its unchecked accesses rely on:
    /// - **Labels:** a lane's row is a live label (a start, or a successor stored in the
    ///   records, every one a row below n: the constructor stores only φ of rows) or, in
    ///   the tail only, PARK. So x & 31 < 30 and x >> 5 ≤ G: its field and its header are
    ///   inside the records' `64(G + 1)` initialised bytes (in either layout).
    /// - **RAW entries:** a RAW row's record (field 0xFFFF, not PARK) has header RAW_BIT |
    ///   idx with idx below the RAW groups' count (canonicalised), so its entry 32·idx +
    ///   (x & 31) is in the RAW area. In the busy phase PARK is never a row (a lane's
    ///   arrival is resolved before its next step), so a field of 0xFFFF there is a RAW
    ///   row's; in the tail PARK is tested before any RAW index is taken, and a lane is
    ///   parked only with `pend` set to such an entry.
    /// - **Output:** production's room walk's contract (`bwt::walk_segments_room_with_
    ///   prefetch`): Ownership, Spare, Handoffs, Initialised, Lifetimes; staged as master
    ///   stages it. An arrival writes no byte in its sweep turn, and its resolve writes the
    ///   byte at the same column before any flush reads the column, so no tile row has a
    ///   hole.
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
            let f = self.field(x);
            if f == RAWF {
                // a RAW start: its saved entry, so it never waits
                hit!(RAW_START);
            }
            self.saved.push(f);
        }
        for i in 0..ns {
            let x = self.s.starts[i];
            self.set_field(x, START);
        }
        let rec = self.rec.as_ptr() as *const u8;
        let hb = rec.wrapping_add(60 * self.rec.len());
        let raw = self.raw.as_ptr() as *const u32;
        // the physical-access audit, checked in debug builds: every field and header read
        // is inside the records' initialised storage, and every RAW entry read is inside
        // the RAW area
        let (rec_bytes, raw_entries) = (64 * self.rec.len(), 32 * self.raw.len());
        let in_rec = move |x: u32| {
            debug_assert!(x & 31 < 30, "label {x} on a header position");
            if S {
                let (g, fields) = ((x >> 5) as usize, rec_bytes / 64 * 60);
                debug_assert!(
                    2 * x as usize - 4 * g + 2 <= fields,
                    "field of {x} past the fields"
                );
                debug_assert!(
                    fields + 4 * g + 4 <= rec_bytes,
                    "header of {x} past the headers"
                );
            } else {
                debug_assert!(
                    2 * (x as usize) + 2 <= rec_bytes,
                    "field of {x} past the records"
                );
                debug_assert!(
                    2 * (x & !31) as usize + 64 <= rec_bytes,
                    "header of {x} past the records"
                );
            }
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
        // the tail's sentinel cases; START is tested first, PARK before any RAW index
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
            pf::<S>(rec, hb, lanes[active].row);
            queued += 1;
            active += 1;
        }
        if active == LANES {
            // While every lane is busy the rows are held together in `rows`, and the bytes
            // are staged: offset j of a chunk of at most TILE offsets puts lane k's byte at
            // tile[k][j], one base for every lane, instead of at its buffer's base + off.
            // Lane k's current segment has its bytes from st[k] in its tile row; they are
            // copied to its buffer at its p when the chunk ends or the segment does, so
            // every byte lands where the unstaged walk wrote it, and the pass's room bounds
            // the copies as it bounded the writes (master's staging, unchanged). `pend[k]`
            // is a RAW arrival's entry, live from the arrival to its offset's resolve.
            let mut rows = [0u32; LANES];
            let mut pend = [0u32; LANES];
            for (k, l) in lanes.iter().enumerate() {
                rows[k] = l.row;
            }
            let mut tile = [[0u8; TILE]; LANES];
            // copy lane k's tile row from st[k] to end into its buffer at p
            let flush = |l: &mut Lane, row: &[u8; TILE], from: usize, to: usize| {
                let len = to - from;
                // SAFETY: from..to is inside the row; the lane's buffer has room for the
                // chunk's bytes (the pass's room, as for the unstaged writes), so p + len is
                // at most its end
                debug_assert!(from <= to && to <= TILE);
                debug_assert!(len <= unsafe { l.end.offset_from(l.p) } as usize);
                unsafe {
                    std::ptr::copy_nonoverlapping(row.as_ptr().add(from), l.p, len);
                    l.p = l.p.add(len);
                }
            };
            // (tests: the lanes that arrived at a RAW row at the offset before)
            #[cfg(test)]
            let mut prev_waited = 0u32;
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
                #[cfg(test)]
                if lanes
                    .iter()
                    .any(|l| unsafe { l.end.offset_from(l.p) } as usize == room)
                {
                    hit!(PASS_ENDS_FULL);
                }
                let mut done = 0;
                if room > TILE {
                    hit!(ROOM_OVER_TILE);
                }
                while done < room {
                    let w = (room - done).min(TILE);
                    let mut st = [0usize; LANES];
                    for j in 0..w {
                        // The offset: every lane steps. A lane whose field is the start
                        // marker is only marked, and handed over below, in lane order, as
                        // master's room walk does; a lane whose field is a RAW row's arrives
                        // (its entry's index saved and prefetched, no byte) and is resolved
                        // below, before any handover
                        let rs = &mut rows;
                        let pd = &mut pend;
                        let mut ev: u64 = 0;
                        let tp = tile.as_mut_ptr().cast::<u8>();
                        macro_rules! cstep {
                            ($k:expr) => {{
                                let k = $k;
                                // SAFETY: k < LANES; rs[k] is a live label (never PARK in
                                // the busy phase), so its field and header are inside the
                                // records (Labels, above); k * TILE + j is inside the tile
                                // (j < w <= TILE)
                                unsafe {
                                    let r = rs.get_unchecked_mut(k);
                                    let x = *r;
                                    in_rec(x);
                                    debug_assert!(x != park, "PARK in the busy phase");
                                    let o = *fld::<S>(rec, x);
                                    let h = *hdr::<S>(rec, hb, x);
                                    if o < START {
                                        *tp.add(k * TILE + j) = h as u8;
                                        let s = (h >> 8) + o as u32;
                                        *r = s;
                                        pf::<S>(rec, hb, s);
                                    } else {
                                        cold();
                                        if o == START {
                                            ev |= 1 << k;
                                        } else {
                                            // a RAW row's own header (field 0xFFFF, not
                                            // PARK): the port's arrival
                                            debug_assert!(h & RAW_BIT != 0 && o == RAWF);
                                            let e = 32 * (h & M20) + (x & 31);
                                            debug_assert!(
                                                (e as usize) < raw_entries,
                                                "a RAW index past the RAW area"
                                            );
                                            *pd.get_unchecked_mut(k) = e;
                                            // SAFETY: e is in the RAW area (RAW entries)
                                            prefetch(raw.add(e as usize) as *const u8);
                                            ev |= 1 << (32 + k);
                                            hit!(RAW_ARRIVALS_FULL);
                                            #[cfg(test)]
                                            if j > 0 && st[k] == j {
                                                hit!(RAW_AFTER_TAKE);
                                            }
                                        }
                                    }
                                }
                            }};
                        }
                        let mut c = 0;
                        while c + 8 <= LANES {
                            cstep!(c);
                            cstep!(c + 1);
                            cstep!(c + 2);
                            cstep!(c + 3);
                            cstep!(c + 4);
                            cstep!(c + 5);
                            cstep!(c + 6);
                            cstep!(c + 7);
                            c += 8;
                        }
                        #[cfg(test)]
                        {
                            let (marked, waited) = (ev as u32, (ev >> 32) as u32);
                            if marked & prev_waited != 0 {
                                hit!(RAW_THEN_START);
                            }
                            if waited & prev_waited != 0 {
                                hit!(RAW_CHAIN);
                            }
                            if waited.count_ones() > 1 {
                                hit!(RAW_SEVERAL);
                            }
                            if waited != 0 && marked != 0 {
                                hit!(RAW_AND_MARK);
                            }
                            if waited != 0 && j == 0 && done > 0 {
                                hit!(RAW_AT_CHUNK_START);
                            }
                            if waited != 0 && j + 1 == w && done + w < room {
                                hit!(RAW_AT_CHUNK_END);
                            }
                            if waited != 0 && done + j + 1 == room {
                                hit!(RAW_AT_PASS_END);
                            }
                            prev_waited = waited;
                        }
                        if ev != 0 {
                            // The resolve: each lane that arrived at a RAW row at this
                            // offset, in lane order, reads its entry (the port's resume)
                            // and stages its byte at column j, which its arrival left
                            // empty; its successor is its row, and that row's record is
                            // prefetched. Before any handover, so every flush below finds
                            // the lane's byte at j
                            let mut waited = (ev >> 32) as u32;
                            while waited != 0 {
                                let k = waited.trailing_zeros() as usize;
                                waited &= waited - 1;
                                // SAFETY: pd[k] was set at this offset from a RAW row's own
                                // header, in the RAW area; k * TILE + j is inside the tile
                                unsafe {
                                    let e = *raw.add(*pd.get_unchecked(k) as usize);
                                    *tp.add(k * TILE + j) = e as u8;
                                    *rs.get_unchecked_mut(k) = e >> 8;
                                    pf::<S>(rec, hb, e >> 8);
                                }
                                hit!(RESOLVES);
                            }
                            // master's handover loop, from here to its end, unchanged but
                            // for the take (the saved successor) and its prefetch
                            let mut marked = ev & 0xFFFF_FFFF;
                            if marked != 0 && j == 0 && done > 0 {
                                hit!(MARK_AT_LATER_CHUNK_START);
                            }
                            #[cfg(test)]
                            if marked.count_ones() > 1 {
                                hit!(MARKED_SEVERAL);
                            }
                            #[cfg(test)]
                            let mut took = false;
                            while marked != 0 {
                                let k = marked.trailing_zeros() as usize;
                                marked &= marked - 1;
                                let l = &mut lanes[k];
                                l.row = rs[k];
                                if st[k] > 0 {
                                    hit!(DONE_IN_TAKING_CHUNK);
                                }
                                // its segment's bytes end before offset j
                                flush(l, &tile[k], st[k], j);
                                finish(l, bufs, next);
                                if queued == ns {
                                    // Every lane but the marked ones after this one has
                                    // its byte at j: an unmarked lane staged it (in its
                                    // step, or in the resolve), and an earlier marked
                                    // lane's new segment starts after it. A marked lane
                                    // after this one still sits on its segment's end,
                                    // without a byte at j, for the tail.
                                    for (i, li) in lanes.iter_mut().enumerate() {
                                        li.row = rs[i];
                                        if i != k {
                                            let waiting = marked >> i & 1 != 0;
                                            flush(li, &tile[i], st[i], j + usize::from(!waiting));
                                            #[cfg(test)]
                                            if i > k && !waiting {
                                                if li.p == li.end {
                                                    hit!(LATER_STEPPED_FULL);
                                                }
                                                // SAFETY: the lane's row is a live label
                                                if unsafe { *fld::<S>(rec, li.row) } == START {
                                                    hit!(LATER_STEPPED_ON_START);
                                                }
                                            }
                                        }
                                    }
                                    hit!(FULL_PHASE_ENDS_MID_PASS);
                                    #[cfg(test)]
                                    {
                                        if j == 0 && done > 0 {
                                            hit!(EXHAUSTED_AT_CHUNK_START);
                                        }
                                        if j + 1 == w && done + w < room {
                                            hit!(EXHAUSTED_AT_CHUNK_END);
                                        }
                                        if done + j + 1 == room {
                                            hit!(EXHAUSTED_AT_PASS_END);
                                        }
                                        if marked != 0 {
                                            hit!(ENDS_WITH_LANES_WAITING);
                                        }
                                        if took {
                                            hit!(ENDS_AFTER_A_TAKE);
                                        }
                                        if ev >> 32 != 0 {
                                            hit!(RAW_AT_EXHAUSTION);
                                        }
                                    }
                                    break 'full;
                                }
                                hit!(MID_PASS_TAKES);
                                *l = take(queued, bufs);
                                rs[k] = l.row;
                                pf::<S>(rec, hb, l.row);
                                queued += 1;
                                // its first byte is in its buffer; its next, at offset
                                // j + 1, is staged
                                st[k] = j + 1;
                                #[cfg(test)]
                                {
                                    took = true;
                                    if j + 1 == w {
                                        hit!(TAKE_AT_CHUNK_END);
                                    }
                                    if done > 0 {
                                        hit!(TAKE_IN_LATER_CHUNK);
                                    }
                                }
                            }
                        }
                    }
                    for (k, l) in lanes.iter_mut().enumerate() {
                        flush(l, &tile[k], st[k], w);
                    }
                    done += w;
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
                hit!(TAIL_ENTERED_FULL);
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
                let o = unsafe { *fld::<S>(rec, x) };
                let h = unsafe { *hdr::<S>(rec, hb, x) };
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
                pf::<S>(rec, hb, l.row);
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
            let (x, f) = (self.s.starts[i], self.saved[i]);
            self.set_field(x, f);
        }
        self.s.first = self
            .s
            .starts
            .binary_search(&phi(orig_ptr as usize))
            .expect("the first segment's start");
        self.s.n = n;
    }

    /// The port's unstaged room walk (b4786f66's `room`, its full phase writing at each
    /// lane's base + offset, a waiting turn moving the base back), kept as the staged
    /// walk's reference: the same starts must give the same segments. It counts nothing
    /// in `hits` (so that the staged walk's counters are its own); it returns the RAW
    /// arrivals it saw
    #[cfg(test)]
    fn room_unstaged(&mut self, orig_ptr: u32) -> usize {
        let n = self.n;
        let park = self.park();
        let ns = self.s.starts.len();
        self.succ.clear();
        self.saved.clear();
        for i in 0..ns {
            let x = self.s.starts[i];
            let (s, b) = self.next(x);
            self.succ.push(s << 8 | b as u32);
            let f = self.field(x);
            self.saved.push(f);
        }
        for i in 0..ns {
            let x = self.s.starts[i];
            self.set_field(x, START);
        }
        let rec = self.rec.as_ptr() as *const u8;
        let hb = rec.wrapping_add(60 * self.rec.len());
        let raw = self.raw.as_ptr() as *const u32;
        let raw_entries = 32 * self.raw.len();
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
        let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
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
            unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
            next[l.seg] = seg_of(l.row);
        };
        let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
            let b = &mut bufs[l.seg];
            let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
            unsafe { b.set_len(len) };
            b.reserve(b.capacity());
            l.p = unsafe { b.as_mut_ptr().add(len) };
            l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        };
        let slow = |l: &mut Lane, o: u16, h: u32| -> Slow {
            let x = l.row;
            if o == START {
                Slow::Start
            } else if x == park {
                assert!((l.pend as usize) < raw_entries);
                let e = unsafe { *raw.add(l.pend as usize) };
                l.row = e >> 8;
                Slow::Byte(e as u8)
            } else {
                assert!(h & RAW_BIT != 0 && o == RAWF && x != park);
                l.pend = 32 * (h & M20) + (x & 31);
                assert!((l.pend as usize) < raw_entries);
                l.row = park;
                Slow::Wait
            }
        };
        let mut arrivals = 0;
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
            queued += 1;
            active += 1;
        }
        if active == LANES {
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l, bufs);
                    }
                    room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
                }
                assert!(room > 0);
                let mut base: [*mut u8; LANES] = std::array::from_fn(|k| lanes[k].p);
                for off in 0..room {
                    for k in 0..LANES {
                        let l = &mut lanes[k];
                        let x = l.row;
                        let o = unsafe { *fld::<S>(rec, x) };
                        let h = unsafe { *hdr::<S>(rec, hb, x) };
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
                                        for (j, lj) in lanes.iter_mut().enumerate() {
                                            if j < k {
                                                lj.p = base[j].wrapping_add(off + 1);
                                            } else if j > k {
                                                lj.p = base[j].wrapping_add(off);
                                            }
                                        }
                                        break 'full;
                                    }
                                    *l = take(queued, bufs);
                                    queued += 1;
                                    base[k] = l.p.wrapping_sub(off + 1);
                                    continue;
                                }
                                Slow::Wait => {
                                    arrivals += 1;
                                    base[k] = base[k].wrapping_sub(1);
                                    continue;
                                }
                            }
                        };
                        assert!(base[k].wrapping_add(off) < l.end);
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
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l, bufs);
            }
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let x = l.row;
                let o = unsafe { *fld::<S>(rec, x) };
                let h = unsafe { *hdr::<S>(rec, hb, x) };
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
                            arrivals += 1;
                            i += 1;
                            continue;
                        }
                    }
                };
                unsafe {
                    *l.p = byte;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    grow(l, bufs);
                }
                i += 1;
            }
        }
        for i in 0..ns {
            let (x, f) = (self.s.starts[i], self.saved[i]);
            self.set_field(x, f);
        }
        self.s.first = self
            .s
            .starts
            .binary_search(&phi(orig_ptr as usize))
            .expect("the first segment's start");
        self.s.n = n;
        arrivals
    }
}

// ---------------------------------------------------------------------------------------
// The kernel
// ---------------------------------------------------------------------------------------

/// The compact30 inverse BWT: the records built from the last column (`Ctor::build`), then
/// walked (`C30::walk`: the staged room walk, or the plain walk below
/// `bwt::MIN_SEGMENTED` rows). Exactly `bwt::walk`'s output. `S` is the records' layout:
/// `Compact30` (S = false, the default kernel) keeps a group's fields and header in one
/// 64-byte record; `Compact30Split` (S = true, the layout control) holds the same values
/// with the fields and the headers apart.
///
/// Where the records can't represent the block (more than `MAX_N` rows, or counts that
/// are not the last column's own, which the constructor refuses before publishing any
/// record), the block is inverted by production's one-core kernel, `bwt::Segmented`
/// (`bwt::build`, then its room walk), so its output, and its behaviour on counts no
/// decoder makes, are `Segmented`'s.
#[derive(Default)]
pub struct Compact30Of<const S: bool> {
    ctor: Ctor,
    c: C30<S>,
    fallback: Segmented,
}

/// The compact30 kernel in its record layout: the default inverse BWT
/// (`bwt::DefaultInverse`)
pub type Compact30 = Compact30Of<false>;
/// The compact30 kernel in the split layout (fields and headers apart): the layout control
pub type Compact30Split = Compact30Of<true>;

impl<const S: bool> Compact30Of<S> {
    /// The block's records alone, the walk's input: for timing the walk apart from the
    /// build (examples/sweep.rs). False where the kernel would fall back (`invert` then
    /// runs `bwt::Segmented`), and then `walk_records` must not be called
    pub fn build_records(&mut self, ll: &[u8], counts: &[u32; 256]) -> bool {
        let ok = self
            .ctor
            .build::<S>(ll, counts, 0, &mut self.c.rec, &mut self.c.raw);
        self.c.n = if ok { ll.len() } else { 0 };
        ok
    }

    /// The walk of the records `build_records` last built, into `out` (cleared): with it,
    /// exactly `invert`'s output. `orig_ptr` is below the block's length (checked)
    pub fn walk_records(&mut self, orig_ptr: u32, out: &mut Vec<u8>) {
        assert!(
            !self.c.rec.is_empty() && (orig_ptr as usize) < self.c.n,
            "walk_records: no records built for this primary index"
        );
        self.c.walk(orig_ptr, out);
    }
}

impl<const S: bool> InverseBwt for Compact30Of<S> {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        if self
            .ctor
            .build::<S>(ll, counts, 0, &mut self.c.rec, &mut self.c.raw)
        {
            self.c.n = ll.len();
            self.c.walk(orig_ptr, out);
        } else {
            hit!(FALLBACKS);
            self.fallback.invert(ll, counts, orig_ptr, out);
        }
    }

    // The build reads `ll` only as a slice, and refuses before publishing any record; the
    // walk then reads only the records, the RAW groups and the segment scratch, so its
    // output can go over `ll`. Both writers (`plain`, `SegScratch::assemble`) clear their
    // output and write exactly n = ll.len() bytes, which fits the capacity `ll` already
    // has. A refused build runs `Segmented::invert_in_place`, which is true on every
    // block. So the success contract holds, and the kernel never returns false.
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], orig_ptr: u32) -> bool {
        if self
            .ctor
            .build::<S>(ll, counts, 0, &mut self.c.rec, &mut self.c.raw)
        {
            self.c.n = ll.len();
            self.c.walk(orig_ptr, ll);
            true
        } else {
            hit!(FALLBACKS);
            self.fallback.invert_in_place(ll, counts, orig_ptr)
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
    use std::cell::Cell;
    use std::thread::LocalKey;

    type Counter = &'static LocalKey<Cell<usize>>;

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

    /// Records as bytes, in the record layout (safe: each field and header in turn)
    fn rec_bytes(rec: &[Rec]) -> Vec<u8> {
        let mut v = Vec::with_capacity(64 * rec.len());
        for r in rec {
            for f in r.f {
                v.extend(f.to_ne_bytes());
            }
            v.extend(r.h.to_ne_bytes());
        }
        v
    }

    /// The record layout's bytes moved to the split layout's places: group g's 60 field
    /// bytes to 60g, its header to 60N + 4g
    fn relocate(b: &[u8]) -> Vec<u8> {
        let n = b.len() / 64;
        let mut v = vec![0u8; b.len()];
        for g in 0..n {
            v[60 * g..60 * g + 60].copy_from_slice(&b[64 * g..64 * g + 60]);
            v[60 * n + 4 * g..60 * n + 4 * g + 4].copy_from_slice(&b[64 * g + 60..64 * g + 64]);
        }
        v
    }

    /// Whether a kernel's arena holds exactly these records (the split layout: their bytes
    /// relocated)
    fn records_are<const S: bool>(c: &C30<S>, rec: &[Rec]) -> bool {
        if S {
            c.rec.len() == rec.len() && c.bytes() == relocate(&rec_bytes(rec))
        } else {
            c.rec == rec
        }
    }

    /// The segments' reference, safe and slow: from each start, a lane's own steps (the
    /// start's F and then each row's until the next start), each segment's bytes and its
    /// successor, on every cycle of the list; and the non-start RAW rows the segments
    /// step on (the walk's RAW arrivals, in the busy phase and the tail together)
    fn segment_oracle<const S: bool>(c: &C30<S>) -> (Vec<(Vec<u8>, usize)>, usize) {
        let st = &c.s.starts;
        let idx = |x: u32| st.binary_search(&x).ok();
        let mut raw_rows = 0;
        let segs = st
            .iter()
            .map(|&x| {
                let (mut y, b) = c.next(x);
                let mut bytes = vec![b];
                while idx(y).is_none() {
                    raw_rows += c.is_raw(y) as usize;
                    let (z, b) = c.next(y);
                    bytes.push(b);
                    y = z;
                }
                (bytes, idx(y).unwrap())
            })
            .collect();
        (segs, raw_rows)
    }

    /// The research walk's adversarial starts (`C30::starts` with `extra`), as row labels:
    /// fixed rows, 40 consecutive rows, chain-consecutive triples (one-byte segments), the
    /// first rows of RAW groups (RAW starts), rows whose successor is RAW, and the
    /// successors of RAW rows (so a lane resolved at one offset is marked at the next);
    /// with `m` random rows and the first
    fn adversarial_starts<const S: bool>(c: &C30<S>, orig_ptr: u32, m: usize) -> Vec<u32> {
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
            if c.header(32 * g as u32) & RAW_BIT != 0 {
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
        let (mut into, mut out_of) = (0, 0);
        for r in (0..n).step_by(7) {
            let x = phi(r);
            let y = c.next(x).0;
            if into < 16 && c.is_raw(y) {
                st.push(x);
                into += 1;
            }
            if out_of < 16 && c.is_raw(x) {
                st.push(y);
                out_of += 1;
            }
            if into == 16 && out_of == 16 {
                break;
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
    fn chain_starts<const S: bool>(c: &C30<S>, orig_ptr: u32) -> Option<Vec<u32>> {
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

    /// `m` random rows' labels (production's `random_rows`) and phi(orig_ptr), sorted and
    /// distinct: production's starts at `m` + 1 segments
    fn random_starts(n: usize, orig_ptr: u32, m: usize) -> Vec<u32> {
        let mut rows = Vec::new();
        random_rows(n, m, &mut rows);
        let mut st: Vec<u32> = rows.iter().map(|&r| phi(r as usize)).collect();
        st.push(phi(orig_ptr as usize));
        st.sort_unstable();
        st.dedup();
        st
    }

    /// Whether to compare each staged walk with the port's unstaged one (`room_unstaged`):
    /// always natively; under Miri, which checks the unsafe code and not the algorithm, the
    /// staged walk alone
    const REFERENCE: bool = !cfg!(miri);

    /// One room walk, for the C/L differential: its segments, their successors, the first,
    /// and every counter's change
    #[derive(PartialEq, Debug)]
    struct Walked {
        bufs: Vec<Vec<u8>>,
        next: Vec<usize>,
        first: usize,
        counts: Vec<usize>,
    }

    fn delta(before: &[usize], after: &[usize]) -> Vec<usize> {
        after.iter().zip(before).map(|(a, b)| a - b).collect()
    }

    /// A counter's value in a snapshot (or a change between two): found by its
    /// thread-local cell's address (a `thread_local!` key is a constant, so two references
    /// to it need not be one address)
    fn of(counts: &[usize], c: Counter) -> usize {
        let cell = |k: Counter| k.with(|x| x as *const Cell<usize>);
        let i = hits::all()
            .iter()
            .position(|(_, k)| cell(k) == cell(c))
            .expect("a counter");
        counts[i]
    }

    /// The room walk from the given starts (labels, phi(orig_ptr) among them), checked:
    /// its output against `want`; its segments against the reference; the markers restored
    /// (bytewise); its turn identities (every busy-phase arrival resolved, every tail wait
    /// resumed, and the arrivals the RAW rows the segments step on); and its segments
    /// against the port's unstaged walk's from the same starts
    fn room_from<const S: bool>(
        c: &mut C30<S>,
        orig_ptr: u32,
        starts: Vec<u32>,
        want: &[u8],
        what: &str,
    ) -> Walked {
        let before = c.bytes().to_vec();
        let ns = starts.len();
        c.s.starts = starts.clone();
        let h0 = hits::snapshot();
        c.room(orig_ptr);
        let d = delta(&h0, &hits::snapshot());
        assert!(c.bytes() == before, "{what}: a marker not restored");
        let mut out = Vec::new();
        c.s.assemble(&mut out);
        assert!(out == want, "{what}: the room walk differs from production");
        let (segs, raw_rows) = segment_oracle(c);
        let mut total = 0;
        for (i, (bytes, next)) in segs.iter().enumerate() {
            assert!(c.s.bufs[i] == *bytes, "{what}: segment {i}'s bytes");
            assert_eq!(c.s.next[i], *next, "{what}: segment {i}'s successor");
            total += bytes.len();
        }
        let emitted: usize = c.s.bufs[..ns].iter().map(Vec::len).sum();
        assert_eq!(emitted, total, "{what}: the bytes emitted");
        let (arr, res) = (of(&d, &hits::RAW_ARRIVALS_FULL), of(&d, &hits::RESOLVES));
        let (tail, waits, resumed) = (
            of(&d, &hits::RAW_ARRIVALS),
            of(&d, &hits::WAIT_TURNS_TAIL),
            of(&d, &hits::RESUMPTIONS),
        );
        assert_eq!(arr, res, "{what}: an arrival not resolved");
        assert!(
            tail == waits && waits == resumed,
            "{what}: the tail's turns"
        );
        assert_eq!(arr + waits, raw_rows, "{what}: the RAW rows stepped on");
        let walked = Walked {
            bufs: c.s.bufs[..ns].to_vec(),
            next: c.s.next.clone(),
            first: c.s.first,
            counts: d,
        };
        if REFERENCE {
            c.s.starts = starts;
            let arrivals = c.room_unstaged(orig_ptr);
            assert!(
                c.bytes() == before,
                "{what}: unstaged, a marker not restored"
            );
            assert!(
                c.s.bufs[..ns] == walked.bufs[..] && c.s.next == walked.next,
                "{what}: the staged walk's segments differ from the unstaged walk's"
            );
            assert_eq!(c.s.first, walked.first);
            assert_eq!(arrivals, raw_rows, "{what}: unstaged, the RAW arrivals");
        }
        walked
    }

    /// The kernel pair the tests run every block through: the record layout (C) and the
    /// split layout (L)
    #[derive(Default)]
    struct Pair {
        c: Compact30,
        l: Compact30Split,
    }

    /// One layout's run of `check`: its arena after the build, its walks, and every
    /// counter's change
    struct Run {
        arena: Vec<u8>,
        walks: Vec<Walked>,
        counts: Vec<usize>,
    }

    fn check_one<const S: bool>(
        k: &mut Compact30Of<S>,
        ll: &[u8],
        orig_ptr: u32,
        permille: u32,
        want: &[u8],
        what: &str,
    ) -> Run {
        let counts = counts_of(ll);
        let what = format!("{what} (S {S})");
        let h0 = hits::snapshot();
        assert!(
            k.ctor
                .build::<S>(ll, &counts, permille, &mut k.c.rec, &mut k.c.raw),
            "{what}: refused its own counts"
        );
        let (rec, raw) = oracle(ll, &counts, permille);
        assert!(
            records_are(&k.c, &rec),
            "{what}: records differ from the oracle's"
        );
        assert!(
            k.c.raw == raw,
            "{what}: RAW groups differ from the oracle's"
        );
        let arena = k.c.bytes().to_vec();
        k.c.n = ll.len();
        let mut out = Vec::new();
        k.c.walk(orig_ptr, &mut out);
        assert!(out == want, "{what}: the walk differs from production");
        assert!(k.c.bytes() == arena, "{what}: a marker not restored");
        let mut walks = Vec::new();
        let st = random_starts(ll.len(), orig_ptr, SEGMENTS - 1);
        walks.push(room_from(
            &mut k.c,
            orig_ptr,
            st,
            want,
            &format!("{what}, random starts"),
        ));
        for m in [SEGMENTS - 1, 40] {
            let st = adversarial_starts(&k.c, orig_ptr, m);
            walks.push(room_from(
                &mut k.c,
                orig_ptr,
                st,
                want,
                &format!("{what}, adversarial starts {m}"),
            ));
        }
        if let Some(st) = chain_starts(&k.c, orig_ptr) {
            walks.push(room_from(
                &mut k.c,
                orig_ptr,
                st,
                want,
                &format!("{what}, chain starts"),
            ));
        }
        if permille == 0 {
            k.invert(ll, &counts, orig_ptr, &mut out);
            assert!(out == want, "{what}: Compact30 differs from Segmented");
            let mut v = ll.to_vec();
            v.reserve(7);
            let (p, cap) = (v.as_ptr(), v.capacity());
            assert!(k.invert_in_place(&mut v, &counts, orig_ptr), "{what}");
            assert!(v == want, "{what}: in place differs from Segmented");
            assert_eq!((v.as_ptr(), v.capacity()), (p, cap), "{what}: ll moved");
        }
        Run {
            arena,
            walks,
            counts: delta(&h0, &hits::snapshot()),
        }
    }

    /// The C/L differential: the same walks (segments, successors, first) and the same
    /// value of every counter, and L's arena C's relocated
    fn same_runs(c: &Run, l: &Run, what: &str) {
        assert!(
            l.arena == relocate(&c.arena),
            "{what}: L's arena is not C's relocated"
        );
        assert_eq!(c.walks.len(), l.walks.len());
        for (i, (a, b)) in c.walks.iter().zip(&l.walks).enumerate() {
            assert!(a == b, "{what}: walk {i} differs between C and L");
        }
        if c.counts != l.counts {
            for ((name, _), (x, y)) in hits::all().iter().zip(c.counts.iter().zip(&l.counts)) {
                assert_eq!(x, y, "{what}: counter {name} differs between C and L");
            }
        }
    }

    /// One block, every way, in both layouts: the records against the oracle's (at
    /// `permille` forced RAW), the walk (`C30::walk`, room or plain) and the room walk on
    /// any length (production's random starts, then the adversarial ones, and few segments,
    /// so buffers grow) against `Segmented`'s output and the unstaged walk; the kernel
    /// itself, and in place; then C against L
    fn check(k: &mut Pair, ll: &[u8], orig_ptr: u32, permille: u32, what: &str) {
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
        let c = check_one(&mut k.c, ll, orig_ptr, permille, &want, &what);
        let l = check_one(&mut k.l, ll, orig_ptr, permille, &want, &what);
        same_runs(&c, &l, &what);
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

    fn assert_reached(names: &[(&str, Counter)]) {
        for (name, c) in names {
            assert!(hits::get(c) > 0, "never reached: {name}");
        }
    }

    // (a lane coming into the tail with its buffer exactly full, TAIL_ENTERED_FULL, is
    // counted but not required: as in master, these columns don't reach it)
    static WALK_HITS: [(&str, Counter); 11] = [
        (
            "a lane arriving at a RAW row in the busy phase",
            &hits::RAW_ARRIVALS_FULL,
        ),
        ("a RAW row resolved at its offset", &hits::RESOLVES),
        (
            "a lane arriving at a RAW row in the tail",
            &hits::RAW_ARRIVALS,
        ),
        ("a parked lane resuming in the tail", &hits::RESUMPTIONS),
        ("a waiting turn in the tail", &hits::WAIT_TURNS_TAIL),
        ("a segment taken mid-pass", &hits::MID_PASS_TAKES),
        ("a buffer grown at a pass's start", &hits::GROWS_FULL),
        ("a buffer grown in the tail", &hits::GROWS_TAIL),
        (
            "the full phase ending mid-pass",
            &hits::FULL_PHASE_ENDS_MID_PASS,
        ),
        ("a RAW start", &hits::RAW_START),
        ("a pass ending with a buffer full", &hits::PASS_ENDS_FULL),
    ];
    static BUILD_HITS: [(&str, Counter); 8] = [
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
        let mut k = Pair::default();
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
        let mut k = Pair::default();
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
        let mut k = Pair::default();
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
        let mut k = Pair::default();
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
        let mut k = Pair::default();
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
                k.c.invert(&ll, &counts, orig_ptr, &mut out);
                assert!(out == want, "{name}, orig_ptr {orig_ptr}");
                k.l.invert(&ll, &counts, orig_ptr, &mut out);
                assert!(out == want, "{name}, orig_ptr {orig_ptr}: split");
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
    /// the kernel used again after each on a block it can represent; in both layouts, and
    /// in place (true on every block, a refused one too: `Segmented`'s in place)
    #[test]
    fn fallbacks_are_segmented() {
        fallbacks::<false>();
        fallbacks::<true>();
    }

    fn fallbacks<const S: bool>() {
        let mut k = Compact30Of::<S>::default();
        let mut rnd = rng(77);
        let small: Vec<u8> = (0..70_000).map(|_| rnd(5) as u8).collect();
        let again = |k: &mut Compact30Of<S>| {
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
        assert!(k
            .ctor
            .build::<S>(&[], &[0; 256], 0, &mut k.c.rec, &mut k.c.raw));
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
            if n == MAX_N + 1 {
                // in place over ll, where the records can't hold the block: Segmented's
                let mut v = ll.clone();
                let (p, cap) = (v.as_ptr(), v.capacity());
                assert!(k.invert_in_place(&mut v, &counts, orig_ptr), "n {n}");
                assert!(Some(&v) == want.as_ref(), "n {n}: in place");
                assert_eq!((v.as_ptr(), v.capacity()), (p, cap), "n {n}: ll moved");
            }
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
                    !k.ctor
                        .build::<S>(&ll, &wrong, 0, &mut k.c.rec, &mut k.c.raw),
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
                if let Some(want) = &want {
                    // in place on refused counts: true, Segmented's bytes, ll unmoved
                    let mut v = ll.clone();
                    let (p, cap) = (v.as_ptr(), v.capacity());
                    assert!(k.invert_in_place(&mut v, &wrong, 1), "refused, in place");
                    assert!(v == *want, "refused, in place: not Segmented's bytes");
                    assert_eq!((v.as_ptr(), v.capacity()), (p, cap), "ll moved");
                }
                again(&mut k);
            }
        }
    }

    /// One kernel through blocks of varying lengths in any order (its buffers kept from
    /// block to block: shorter after longer, RAW-heavy after RAW-free), in both layouts
    #[test]
    fn reused_across_blocks() {
        let mut k = Pair::default();
        let mut rnd = rng(3);
        for round in 0..12 {
            let n = [900_000usize, 70_000, 31, 400_000, 65_536, 1][round % 6];
            let sigma = [256u64, 2, 1, 40][round % 4];
            let ll = column(&mut rnd, n, sigma, &[1, 3, 40]);
            let counts = counts_of(&ll);
            let orig_ptr = rnd(n as u64) as u32;
            let (mut want, mut out) = (Vec::new(), Vec::new());
            Segmented::default().invert(&ll, &counts, orig_ptr, &mut want);
            k.c.invert(&ll, &counts, orig_ptr, &mut out);
            assert!(out == want, "round {round}, n {n}");
            k.l.invert(&ll, &counts, orig_ptr, &mut out);
            assert!(out == want, "round {round}, n {n}: split");
        }
    }

    /// Short and threshold blocks through `walk` (the plain walk below MIN_SEGMENTED, the
    /// room walk from it), each fully checked
    #[test]
    fn tiny_and_threshold_blocks() {
        let mut k = Pair::default();
        let mut rnd = rng(0x71);
        for n in [
            1usize,
            2,
            29,
            30,
            31,
            59,
            60,
            61,
            MIN_SEGMENTED - 1,
            MIN_SEGMENTED,
            MIN_SEGMENTED + 1,
        ] {
            for sigma in [1u64, 3, 256] {
                let ll = column(&mut rnd, n, sigma, &[1, 2, 31]);
                for pm in [0, 300] {
                    check(&mut k, &ll, rnd(n as u64) as u32, pm, "tiny");
                }
            }
        }
    }

    /// The busy phase on small lists of several cycles (periodic columns 'ab', 'abc' and of
    /// periods 7, 30, 31 and 32; one byte throughout, whose rows are each their own cycle,
    /// so every segment is one byte; two bytes alternating), from 33 to 96 starts: segments
    /// off the first one's cycle are walked but not joined, and fewer bytes are emitted
    /// than rows. Both layouts, against each other
    #[test]
    fn busy_phase_on_multicycle_lists() {
        let mut rnd = rng(0x3c);
        let mut lists: Vec<(String, Vec<u8>)> = vec![
            ("ab".into(), b"ab".repeat(1500)),
            ("abc".into(), b"abc".repeat(1000)),
            ("constant".into(), vec![9u8; 2500]),
            (
                "alternating".into(),
                (0..2600).map(|i| [3u8, 200][i % 2]).collect(),
            ),
        ];
        for p in [7usize, 30, 31, 32] {
            let unit: Vec<u8> = (0..p).map(|_| rnd(256) as u8).collect();
            lists.push((
                format!("period {p}"),
                unit.iter().copied().cycle().take(3000 + p).collect(),
            ));
        }
        for (name, ll) in lists {
            let n = ll.len();
            let counts = counts_of(&ll);
            for pm in [0, 300] {
                let mut c = C30::<false>::default();
                let mut l = C30::<true>::default();
                assert!(Ctor::default().build::<false>(&ll, &counts, pm, &mut c.rec, &mut c.raw));
                assert!(Ctor::default().build::<true>(&ll, &counts, pm, &mut l.rec, &mut l.raw));
                c.n = n;
                l.n = n;
                for orig_ptr in [0, n as u32 / 2, n as u32 - 1] {
                    let mut tt = TVector::new();
                    build(&ll, &counts, &mut tt);
                    let mut want = Vec::new();
                    walk(&tt, orig_ptr, &mut want);
                    for m in [32usize, 40, 95] {
                        let st = random_starts(n, orig_ptr, m);
                        let what = format!("{name}, permille {pm}, orig_ptr {orig_ptr}, {m}");
                        let a = room_from(&mut c, orig_ptr, st.clone(), &want, &what);
                        let b = room_from(&mut l, orig_ptr, st, &want, &what);
                        assert!(a == b, "{what}: C and L differ");
                    }
                }
            }
        }
    }

    /// The emission rotation: production's segments start at Ψ(r) of its random rows r and
    /// at Ψ(orig_ptr), compact30's at phi(r) and phi(orig_ptr), so compact30's cuts are
    /// production's each one text byte later, but for the first, which is the same. So the
    /// segment counts are equal, and the sorted lengths too but for at most two segments,
    /// the two beside the first start (one a byte longer, one a byte shorter), unless a
    /// random row is orig_ptr or Ψ(orig_ptr) (a start deduplicated in one kernel only)
    #[test]
    fn segments_are_productions_rotated() {
        let mut rnd = rng(0x707);
        let mut compared = 0;
        for case in 0..12 {
            let n = MIN_SEGMENTED + rnd(300_000) as usize;
            let sigma = [2u64, 9, 256][case % 3];
            let ll = column(&mut rnd, n, sigma, &[1, 2, 7]);
            let counts = counts_of(&ll);
            let orig_ptr = rnd(n as u64) as u32;
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let (mut out, mut seg) = (Vec::new(), SegScratch::default());
            crate::bwt::walk_segments_room(&mut tt, orig_ptr, &mut out, &mut seg);
            let ns_p = seg.starts.len();
            let mut p: Vec<usize> = seg.bufs[..ns_p].iter().map(Vec::len).collect();
            let mut k = Compact30::default();
            let mut got = Vec::new();
            k.invert(&ll, &counts, orig_ptr, &mut got);
            assert!(got == out);
            let ns_c = k.c.s.starts.len();
            let mut c: Vec<usize> = k.c.s.bufs[..ns_c].iter().map(Vec::len).collect();
            if ns_c != ns_p {
                assert_eq!(ns_c.abs_diff(ns_p), 1, "case {case}: segment counts");
                continue;
            }
            assert_eq!(
                p.iter().sum::<usize>(),
                c.iter().sum::<usize>(),
                "case {case}"
            );
            p.sort_unstable();
            c.sort_unstable();
            // the lengths in one and not the other (as multisets)
            let (mut i, mut j, mut only_p, mut only_c) = (0, 0, Vec::new(), Vec::new());
            while i < p.len() || j < c.len() {
                if j == c.len() || (i < p.len() && p[i] < c[j]) {
                    only_p.push(p[i]);
                    i += 1;
                } else if i == p.len() || c[j] < p[i] {
                    only_c.push(c[j]);
                    j += 1;
                } else {
                    i += 1;
                    j += 1;
                }
            }
            assert!(
                only_p.len() <= 2 && only_c.len() == only_p.len(),
                "case {case}: production's {only_p:?}, compact30's {only_c:?}"
            );
            let (sp, sc): (usize, usize) = (only_p.iter().sum(), only_c.iter().sum());
            assert_eq!(sp, sc, "case {case}");
            compared += 1;
        }
        assert!(compared >= 10, "{compared} blocks compared");
    }

    /// Natural RAW clusters: a 4-8 KB island of random bytes in a column of long runs (the
    /// island's bytes' groups cross buckets or span widely, so their rows are RAW, and the
    /// walk meets them in succession), in both layouts; the RAW arrivals per emitted byte
    /// well above a text's
    #[test]
    fn clustered_raw_islands() {
        let mut k = Pair::default();
        let mut rnd = rng(0x15);
        for (n, island) in [(120_000usize, 4096usize), (200_000, 8192)] {
            let mut ll = column(&mut rnd, n, 6, &[300, 1000, 2000]);
            let at = rnd((n - island) as u64) as usize;
            for b in &mut ll[at..at + island] {
                *b = rnd(256) as u8;
            }
            let counts = counts_of(&ll);
            let orig_ptr = rnd(n as u64) as u32;
            // (both kernels through the same blocks, so their buffers' capacities, and so
            // their walks' grows, stay alike for the differential)
            let mut rho = [0f64; 2];
            for (i, r) in rho.iter_mut().enumerate() {
                let h0 = hits::snapshot();
                let mut out = Vec::new();
                if i == 0 {
                    k.c.invert(&ll, &counts, orig_ptr, &mut out);
                } else {
                    k.l.invert(&ll, &counts, orig_ptr, &mut out);
                }
                let d = delta(&h0, &hits::snapshot());
                *r = (of(&d, &hits::RAW_ARRIVALS_FULL) + of(&d, &hits::WAIT_TURNS_TAIL)) as f64
                    / n as f64;
            }
            assert!(rho[0] == rho[1], "n {n}: C and L");
            assert!(rho[0] > 0.02, "n {n}: rho {}", rho[0]);
            check(&mut k, &ll, orig_ptr, 0, &format!("island of {island}"));
        }
    }

    /// The staged busy phase's chunk edges with RAW rows (bwt.rs's
    /// `room_walk_staged_chunk_edges`, for the records): blocks long enough and starts few
    /// enough that a pass's room spans several tiles, at no, some, many and all groups
    /// forced RAW. Every state master's staged walk requires is required here too, and each
    /// of the RAW rows' (an arrival and its resolve; several at an offset; with a mark; at a
    /// later chunk's first offset, a chunk's last, the pass's last; where the queue runs
    /// out; after a take; before a start; at consecutive offsets; a RAW start) and the
    /// tail's. Every walk against the plain walk, the segment oracle and the unstaged walk,
    /// in both layouts, against each other
    #[test]
    #[cfg_attr(miri, ignore)]
    fn staged_chunk_edges_with_raw() {
        let required: [(&str, Counter); 27] = [
            ("a pass of more than one chunk", &hits::ROOM_OVER_TILE),
            ("a take at a chunk's last offset", &hits::TAKE_AT_CHUNK_END),
            ("a take in a later chunk", &hits::TAKE_IN_LATER_CHUNK),
            (
                "a segment taken and finished in one chunk",
                &hits::DONE_IN_TAKING_CHUNK,
            ),
            (
                "a lane marked at a later chunk's first offset",
                &hits::MARK_AT_LATER_CHUNK_START,
            ),
            (
                "the queue running out at a later chunk's first offset",
                &hits::EXHAUSTED_AT_CHUNK_START,
            ),
            (
                "the queue running out at a chunk's last offset",
                &hits::EXHAUSTED_AT_CHUNK_END,
            ),
            (
                "the queue running out at a pass's last offset",
                &hits::EXHAUSTED_AT_PASS_END,
            ),
            ("a lane taken mid-pass", &hits::MID_PASS_TAKES),
            ("several lanes marked at one offset", &hits::MARKED_SEVERAL),
            (
                "the queue running out with lanes waiting",
                &hits::ENDS_WITH_LANES_WAITING,
            ),
            (
                "the queue running out after a take",
                &hits::ENDS_AFTER_A_TAKE,
            ),
            (
                "a stepped later lane leaving on its segment's end",
                &hits::LATER_STEPPED_ON_START,
            ),
            ("a RAW arrival in the busy phase", &hits::RAW_ARRIVALS_FULL),
            ("a RAW row resolved", &hits::RESOLVES),
            ("several RAW arrivals at one offset", &hits::RAW_SEVERAL),
            (
                "a RAW arrival and a mark at one offset",
                &hits::RAW_AND_MARK,
            ),
            (
                "a RAW arrival at a later chunk's first offset",
                &hits::RAW_AT_CHUNK_START,
            ),
            (
                "a RAW arrival at a chunk's last offset",
                &hits::RAW_AT_CHUNK_END,
            ),
            (
                "a RAW arrival at a pass's last offset",
                &hits::RAW_AT_PASS_END,
            ),
            (
                "a RAW arrival where the queue runs out",
                &hits::RAW_AT_EXHAUSTION,
            ),
            ("a RAW arrival right after a take", &hits::RAW_AFTER_TAKE),
            (
                "a resolved successor that is a start",
                &hits::RAW_THEN_START,
            ),
            ("a lane resolved at consecutive offsets", &hits::RAW_CHAIN),
            ("a RAW start", &hits::RAW_START),
            ("a waiting turn in the tail", &hits::WAIT_TURNS_TAIL),
            ("a parked lane resuming in the tail", &hits::RESUMPTIONS),
        ];
        let mut rnd = rng(0x9e37);
        let mut cases = 0;
        while cases < 20000 && required.iter().any(|(_, c)| hits::get(c) == 0) {
            cases += 1;
            let n = 4000 + rnd(28000) as usize;
            let alphabet = 1 + rnd(if cases % 2 == 0 { 3 } else { 256 });
            let ll: Vec<u8> = (0..n).map(|_| (rnd(alphabet) * 7) as u8).collect();
            let orig_ptr = rnd(n as u64) as u32;
            let pm = [0, 50, 300, 1000][cases % 4];
            let counts = counts_of(&ll);
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let mut want = Vec::new();
            walk(&tt, orig_ptr, &mut want);
            let mut c = C30::<false>::default();
            let mut l = C30::<true>::default();
            assert!(Ctor::default().build::<false>(&ll, &counts, pm, &mut c.rec, &mut c.raw));
            assert!(Ctor::default().build::<true>(&ll, &counts, pm, &mut l.rec, &mut l.raw));
            assert!(l.bytes() == relocate(c.bytes()), "case {cases}: L's arena");
            c.n = n;
            l.n = n;
            // a pass's room is about 2n / segments + 63: under n / 150 segments it is over
            // two tiles, and over 32 of them the busy phase runs at 32 lanes
            let st = random_starts(n, orig_ptr, 33 + rnd((n / 150) as u64) as usize);
            let what = format!("case {cases}, n {n}, permille {pm}");
            let a = room_from(&mut c, orig_ptr, st.clone(), &want, &what);
            let b = room_from(&mut l, orig_ptr, st, &want, &what);
            assert!(a == b, "{what}: C and L differ");
        }
        for (name, c) in required {
            assert!(hits::get(c) > 0, "never reached in {cases} cases: {name}");
        }
    }

    /// `check` cut to what Miri needs, so that it interprets in minutes: the records
    /// against the oracle's, the walk (plain, below MIN_SEGMENTED) and the room walk from
    /// adversarial starts (40 random ones beside them) against the plain T-vector walk, the
    /// segments against their reference, the markers restored; and the kernel itself
    fn check_miri<const S: bool>(
        k: &mut Compact30Of<S>,
        ll: &[u8],
        orig_ptr: u32,
        permille: u32,
        what: &str,
    ) {
        let counts = counts_of(ll);
        let mut tt = TVector::new();
        build(ll, &counts, &mut tt);
        let mut want = Vec::new();
        walk(&tt, orig_ptr, &mut want);
        let what = format!(
            "{what} (n {}, orig_ptr {orig_ptr}, permille {permille}, S {S})",
            ll.len()
        );
        assert!(k
            .ctor
            .build::<S>(ll, &counts, permille, &mut k.c.rec, &mut k.c.raw));
        let (rec, raw) = oracle(ll, &counts, permille);
        assert!(
            records_are(&k.c, &rec) && k.c.raw == raw,
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
    /// loads and writes under Miri, on small columns, in both layouts: accepted (each
    /// regime, partial groups, crossing buckets, forced RAW groups, every tile phase),
    /// refused before the scatter and mid-scatter, and the walk from adversarial starts
    /// (RAW starts, one-byte segments, lanes arriving at RAW rows in the busy phase and the
    /// tail); each transition asserted reached (a buffer's growth: `miri_compact30_grows`)
    #[test]
    fn miri_compact30_small() {
        miri_small::<false>();
        miri_small::<true>();
        let grows = [
            "a buffer grown at a pass's start",
            "a buffer grown in the tail",
            "a pass ending with a buffer full",
        ];
        let walk_hits: Vec<_> = WALK_HITS
            .into_iter()
            .filter(|(name, _)| !grows.contains(name))
            .collect();
        assert_reached(&walk_hits);
        assert_reached(&BUILD_HITS);
    }

    fn miri_small<const S: bool>() {
        let mut k = Compact30Of::<S>::default();
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
            assert!(!k
                .ctor
                .build::<S>(&ll, &wrong, 0, &mut k.c.rec, &mut k.c.raw));
            assert!(k.c.rec.is_empty() && k.c.raw.is_empty());
        }
    }

    /// The room walk's buffer growth under Miri: starts along a long cycle (`chain_starts`)
    /// of a 10,000-row block in the record layout (the port's case) and of a 5,000-row one
    /// in the split layout, so lanes fill their buffers in the full phase and the tail;
    /// each case asserted to reach the growth transitions itself
    #[test]
    fn miri_compact30_grows() {
        miri_grows::<false>(10_000, 4321);
        miri_grows::<true>(5_000, 2_111);
    }

    fn miri_grows<const S: bool>(n: usize, orig_ptr: u32) {
        let mut k = Compact30Of::<S>::default();
        let mut rnd = rng(0x6e0);
        let ll = column(&mut rnd, n, 256, &[1]);
        let counts = counts_of(&ll);
        let mut tt = TVector::new();
        build(&ll, &counts, &mut tt);
        let mut want = Vec::new();
        walk(&tt, orig_ptr, &mut want);
        assert!(k
            .ctor
            .build::<S>(&ll, &counts, 0, &mut k.c.rec, &mut k.c.raw));
        k.c.n = ll.len();
        let st = chain_starts(&k.c, orig_ptr).expect("a long cycle");
        let d = room_from(&mut k.c, orig_ptr, st, &want, "miri grows").counts;
        for (name, c) in [
            ("a buffer grown at a pass's start", &hits::GROWS_FULL),
            ("a buffer grown in the tail", &hits::GROWS_TAIL),
            ("a segment taken mid-pass", &hits::MID_PASS_TAKES),
            (
                "the full phase ending mid-pass",
                &hits::FULL_PHASE_ENDS_MID_PASS,
            ),
        ] {
            assert!(of(&d, c) > 0, "S {S}, n {n}: never reached: {name}");
        }
    }

    /// The epoch paths under Miri, in both layouts: a block of two epochs whose byte-1
    /// portion is received across the boundary (a slow finish with an epoch search, and a
    /// wide group)
    #[test]
    fn miri_compact30_epochs() {
        let n = EPOCH + 200;
        let mut ll: Vec<u8> = (0..n).map(|i| 2 + (i & 1) as u8).collect();
        for t in 0..29 {
            ll[2 + 2 * t] = 1;
        }
        ll[EPOCH] = 1;
        let counts = counts_of(&ll);
        let (rec, raw) = oracle(&ll, &counts, 0);
        let mut c = Compact30::default();
        assert!(c
            .ctor
            .build::<false>(&ll, &counts, 0, &mut c.c.rec, &mut c.c.raw));
        assert!(
            records_are(&c.c, &rec) && c.c.raw == raw,
            "records differ from the oracle's"
        );
        let mut l = Compact30Split::default();
        assert!(l
            .ctor
            .build::<true>(&ll, &counts, 0, &mut l.c.rec, &mut l.c.raw));
        assert!(
            records_are(&l.c, &rec) && l.c.raw == raw,
            "split: records differ from the oracle's"
        );
        assert!(hits::get(&hits::WIDE) > 0, "never reached: a wide group");
        assert!(
            hits::get(&hits::SLOW_EPOCH_SEARCH) > 0,
            "never reached: an epoch search"
        );
    }

    /// One staged case for Miri: a column from `seed` of rows.0 + rnd(rows.1) rows, its
    /// records at `permille` forced RAW, walked from segments.0 + rnd(segments.1 + n / per)
    /// random starts and the first, in the record layout (`layouts[0]`), the split one
    /// (`layouts[1]`) or both (then C against L); its rows
    fn miri_staged_case(
        seed: u64,
        rows: (usize, u64),
        segments: (usize, u64),
        per: usize,
        permille: u32,
        layouts: [bool; 2],
    ) -> usize {
        let mut rnd = rng(seed);
        let n = rows.0 + rnd(rows.1) as usize;
        let alphabet = 1 + rnd(if seed % 2 == 0 { 3 } else { 256 });
        let ll: Vec<u8> = (0..n).map(|_| (rnd(alphabet) * 7) as u8).collect();
        let orig_ptr = rnd(n as u64) as u32;
        let counts = counts_of(&ll);
        let mut tt = TVector::new();
        build(&ll, &counts, &mut tt);
        let mut want = Vec::new();
        walk(&tt, orig_ptr, &mut want);
        let m = segments.0 + rnd(segments.1 + (n / per) as u64) as usize;
        let st = random_starts(n, orig_ptr, m);
        let what = format!("seed {seed}, n {n}, {m} starts, permille {permille}");
        let a = layouts[0].then(|| {
            let mut c = C30::<false>::default();
            assert!(Ctor::default().build::<false>(&ll, &counts, permille, &mut c.rec, &mut c.raw));
            c.n = n;
            room_from(&mut c, orig_ptr, st.clone(), &want, &what)
        });
        let b = layouts[1].then(|| {
            let mut l = C30::<true>::default();
            assert!(Ctor::default().build::<true>(&ll, &counts, permille, &mut l.rec, &mut l.raw));
            l.n = n;
            room_from(&mut l, orig_ptr, st.clone(), &want, &what)
        });
        if let (Some(a), Some(b)) = (a, b) {
            assert!(a == b, "{what}: C and L differ");
        }
        n
    }

    /// The staged busy phase's RAW paths under Miri: small blocks (200 to 4,200 rows) and
    /// 64 to 256 starts, with groups forced RAW (300 and 1000 per mille), one case in each
    /// layout:
    /// arrivals and their resolves, an arrival beside a mark, an arrival where the queue
    /// runs out, takes mid-pass and the busy phase ending mid-pass. The seeds were searched
    /// for natively (`search_miri_seeds`)
    #[test]
    fn miri_c30_staged_small() {
        for (seed, pm, layouts) in MIRI_SMALL {
            miri_staged_case(seed, SMALL_ROWS, (64, 192), usize::MAX, pm, layouts);
        }
        assert_reached(&MIRI_SMALL_HITS);
    }

    /// The staged busy phase's chunk edges under Miri (master's Miri cases never reach a
    /// pass of more than one chunk): blocks of 8,000-16,000 rows and 33 + rnd(n / 120)
    /// starts, so that a pass's room (2n / ns + 63) can be over TILE and the queue can last
    /// past a chunk's last offset (with 33-40 starts it runs out within the first chunk), at
    /// 300 per mille forced RAW, one case in each layout (each block is over 13,000 rows,
    /// minutes under Miri): arrivals at a later chunk's first offset and at a chunk's last,
    /// a take at a chunk's last offset, and the queue running out at a chunk's last offset.
    /// The seeds were searched for natively (`search_miri_seeds`): 5 (15,921 rows, the
    /// record layout) reaches all but the last, 298 (13,395 rows, the split layout) that
    #[test]
    fn miri_c30_staged_chunks() {
        for (seed, layouts) in MIRI_CHUNKS {
            miri_staged_case(seed, CHUNK_ROWS, (33, 0), CHUNK_PER, 300, layouts);
        }
        assert_reached(&MIRI_CHUNKS_HITS);
    }

    // (seed, permille) for `miri_c30_staged_small`, and seeds for `miri_c30_staged_chunks`,
    // with their columns' and starts' ranges
    const MIRI_SMALL: [(u64, u32, [bool; 2]); 2] =
        [(1, 300, [true, false]), (2, 1000, [false, true])];
    const SMALL_ROWS: (usize, u64) = (200, 4000);
    const MIRI_CHUNKS: [(u64, [bool; 2]); 2] = [(5, [true, false]), (298, [false, true])];
    const CHUNK_ROWS: (usize, u64) = (8000, 8000);
    const CHUNK_PER: usize = 120;
    static MIRI_SMALL_HITS: [(&str, Counter); 6] = [
        ("a RAW arrival in the busy phase", &hits::RAW_ARRIVALS_FULL),
        ("a RAW row resolved", &hits::RESOLVES),
        (
            "a RAW arrival and a mark at one offset",
            &hits::RAW_AND_MARK,
        ),
        (
            "a RAW arrival where the queue runs out",
            &hits::RAW_AT_EXHAUSTION,
        ),
        ("a lane taken mid-pass", &hits::MID_PASS_TAKES),
        (
            "the full phase ending mid-pass",
            &hits::FULL_PHASE_ENDS_MID_PASS,
        ),
    ];
    static MIRI_CHUNKS_HITS: [(&str, Counter); 5] = [
        ("a pass of more than one chunk", &hits::ROOM_OVER_TILE),
        (
            "a RAW arrival at a later chunk's first offset",
            &hits::RAW_AT_CHUNK_START,
        ),
        (
            "a RAW arrival at a chunk's last offset",
            &hits::RAW_AT_CHUNK_END,
        ),
        ("a take at a chunk's last offset", &hits::TAKE_AT_CHUNK_END),
        (
            "the queue running out at a chunk's last offset",
            &hits::EXHAUSTED_AT_CHUNK_END,
        ),
    ];

    /// In place under Miri, both layouts: a block over the plain walk and one over the room
    /// walk's staged phase are written over `ll`, which keeps its buffer
    #[test]
    fn miri_c30_in_place() {
        miri_in_place::<false>();
        miri_in_place::<true>();
    }

    fn miri_in_place<const S: bool>() {
        let mut rnd = rng(0x1a);
        let mut k = Compact30Of::<S>::default();
        for n in [97usize, 700] {
            let ll = column(&mut rnd, n, 4, &[1, 3]);
            let counts = counts_of(&ll);
            let orig_ptr = rnd(n as u64) as u32;
            let mut want = Vec::new();
            Segmented::default().invert(&ll, &counts, orig_ptr, &mut want);
            let mut v = ll.clone();
            let (p, cap) = (v.as_ptr(), v.capacity());
            assert!(k.invert_in_place(&mut v, &counts, orig_ptr));
            assert!(v == want, "n {n}: in place");
            assert_eq!((v.as_ptr(), v.capacity()), (p, cap), "n {n}: ll moved");
        }
        // and the room walk's busy phase, in place: the walk's own entry, from the records
        // built over a copy, with 40 starts written over ll
        let ll = column(&mut rnd, 1500, 3, &[1, 2]);
        let counts = counts_of(&ll);
        let mut want = Vec::new();
        Segmented::default().invert(&ll, &counts, 11, &mut want);
        assert!(k.build_records(&ll, &counts));
        let mut v = ll.clone();
        let (p, cap) = (v.as_ptr(), v.capacity());
        k.c.s.starts = random_starts(ll.len(), 11, 40);
        k.c.room(11);
        k.c.s.assemble(&mut v);
        assert!(v == want, "staged, over ll");
        assert_eq!((v.as_ptr(), v.capacity()), (p, cap), "staged: ll moved");
    }

    /// The Miri seeds' search, natively (`cargo test --release --lib -- --ignored
    /// search_miri_seeds --nocapture`): the smallest seeds whose cases reach every
    /// transition their Miri test requires
    #[test]
    #[ignore]
    fn search_miri_seeds() {
        let reached = |hits: &[(&str, Counter)]| hits.iter().all(|(_, c)| hits::get(c) > 0);
        let reset = || {
            for (_, c) in hits::all() {
                c.with(|c| c.set(0));
            }
        };
        // small: pairs of (seed, permille), the first pair whose cases reach everything
        'small: for a in 1..400u64 {
            for b in a + 1..a + 40 {
                reset();
                for (seed, pm) in [(a, 300), (b, 1000)] {
                    miri_staged_case(seed, SMALL_ROWS, (64, 192), usize::MAX, pm, [true; 2]);
                }
                if reached(&MIRI_SMALL_HITS) {
                    println!("small: [({a}, 300), ({b}, 1000)]");
                    break 'small;
                }
            }
        }
        // chunks: one seed, or two
        reset();
        let mut found = Vec::new();
        for seed in 1..2000u64 {
            let before: Vec<bool> = MIRI_CHUNKS_HITS
                .iter()
                .map(|(_, c)| hits::get(c) > 0)
                .collect();
            let n = miri_staged_case(seed, CHUNK_ROWS, (33, 0), CHUNK_PER, 300, [true; 2]);
            let after: Vec<bool> = MIRI_CHUNKS_HITS
                .iter()
                .map(|(_, c)| hits::get(c) > 0)
                .collect();
            if after != before {
                found.push((seed, n));
            }
            if reached(&MIRI_CHUNKS_HITS) {
                println!("chunks: {found:?}");
                break;
            }
        }
        for (name, c) in MIRI_CHUNKS_HITS {
            println!("chunks: {name}: {}", hits::get(c));
        }
        // each found seed alone: what it reaches
        for &(seed, n) in &found {
            reset();
            miri_staged_case(seed, CHUNK_ROWS, (33, 0), CHUNK_PER, 300, [true; 2]);
            let r: Vec<usize> = MIRI_CHUNKS_HITS.iter().map(|(_, c)| hits::get(c)).collect();
            println!("chunks: seed {seed} (n {n}) alone: {r:?}");
        }
    }

    /// The RAW rows' rate on real blocks (an offline count: the counters exist in test
    /// builds only): every block of the .bz2 files named in MIBZ2_RHO (separated by ':'),
    /// inverted by `Compact30` as a decoder inverts it, with the arrivals counted in the
    /// busy phase (each resolved) and in the tail, per emitted byte (rho), per file and the
    /// largest per block (`cargo test --release --lib -- --ignored rho_ledger --nocapture`)
    #[test]
    #[ignore]
    fn rho_ledger() {
        use crate::stream::{Event, Scanner, SliceSource};
        let files = std::env::var("MIBZ2_RHO").expect("MIBZ2_RHO: .bz2 files, ':'-separated");
        let k = crate::Kernels::detect();
        println!(
            "file              blocks       rows  busy arr  resolves  tail arr  raw starts  \
             rho (arr/B)  max block (rows)    max segmented block (rows)  RAW rows/rows"
        );
        for path in files.split(':').filter(|p| !p.is_empty()) {
            let data = std::fs::read(path).unwrap();
            let mut sc = Scanner::new(SliceSource(&data));
            let mut c30 = Compact30::default();
            let (mut ll, mut counts, mut out) = (Vec::new(), [0u32; 256], Vec::new());
            let (mut blocks, mut rows, mut max_rho, mut raw_rows) = (0, 0, (0f64, 0), 0);
            // the largest rho among the blocks the room walk segments (MIN_SEGMENTED rows or
            // more), and its block's rows
            let mut max_seg = (0f64, 0);
            let mut plain_arrivals = 0;
            let mut totals = [0usize; 4];
            let mut syms = Vec::new();
            while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
                let n = crate::mtf::mtf(&symbols, &mut ll, &mut counts, k).unwrap();
                let h0 = hits::snapshot();
                c30.invert(&ll, &counts, symbols.orig_ptr, &mut out);
                let d = delta(&h0, &hits::snapshot());
                let row = [
                    of(&d, &hits::RAW_ARRIVALS_FULL),
                    of(&d, &hits::RESOLVES),
                    of(&d, &hits::WAIT_TURNS_TAIL),
                    of(&d, &hits::RAW_START),
                ];
                assert_eq!(row[0], row[1]);
                raw_rows += (0..n).filter(|&r| c30.c.is_raw(phi(r))).count();
                for (t, v) in totals.iter_mut().zip(row) {
                    *t += v;
                }
                // a block below MIN_SEGMENTED is walked plainly (uncounted): its arrivals
                // are the RAW rows its n steps visit
                let arrivals = if n < MIN_SEGMENTED {
                    let (mut x, mut a) = (phi(symbols.orig_ptr as usize), 0);
                    for _ in 0..n {
                        a += c30.c.is_raw(x) as usize;
                        x = c30.c.next(x).0;
                    }
                    plain_arrivals += a;
                    a
                } else {
                    row[0] + row[2]
                };
                let rho = arrivals as f64 / n as f64;
                if rho > max_rho.0 {
                    max_rho = (rho, n);
                }
                if n >= MIN_SEGMENTED && rho > max_seg.0 {
                    max_seg = (rho, n);
                }
                blocks += 1;
                rows += n;
                syms = symbols.syms;
            }
            let name = std::path::Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy();
            println!(
                "{name:16}  {blocks:6}  {rows:9}  {:8}  {:8}  {:8}  {:10}  {:11.5}  {:.5} ({:7})  \
                 {:.5} ({:7})            {:.5}",
                totals[0],
                totals[1],
                totals[2],
                totals[3],
                (totals[0] + totals[2] + plain_arrivals) as f64 / rows as f64,
                max_rho.0,
                max_rho.1,
                max_seg.0,
                max_seg.1,
                raw_rows as f64 / rows as f64,
            );
        }
    }
}
