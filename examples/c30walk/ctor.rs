//! The direct compact30 constructor: Astra's design (report 188), authorised as one
//! compile-and-correctness prototype by report 189. Untimed, research only, not wired into
//! the decoder. The frozen representation, walker and oracle (`C30::build`) are used as
//! they are; this module only adds to them.
//!
//!     cargo run --release --example c30walk -- ctor-check OUT.jsonl FILE.bz2 [FILE.bz2 ...]
//!     cargo run --release --example c30walk -- ctor-adversarial OUT.jsonl
//!     cargo run --release --example c30walk -- ctor-synthetic [--quick]
//!
//! **The constructor (`Ctor::build`)**, from the last column L and its counts to `rec` and
//! `raw`, without a successor table:
//! - **Validation:** the counts must sum to n, n < 2²⁴ (production's one-pass condition) and
//!   n ≤ 983,010 (the oracle's PARK label fits 20 bits). Otherwise it returns false at once.
//! - **Cursors:** each byte's bucket has an encoded cursor x = φ(next slot) and an encoded
//!   limit φ(bucket end). A group is x >> 5, a slot x & 31.
//! - **Crossing groups:** every bucket boundary 0 < B < n that is not a multiple of 30 makes
//!   group B / 30 RAW before L is read (deduplicated). In a forced-RAW test configuration,
//!   the oracle's `mix` predicate then marks further groups RAW.
//! - **The scan:** production's 16-byte uniformity test. A uniform chunk is coalesced with
//!   the following chunks of the same byte; the first chunk that fails is kept, its two
//!   words reused by the next iteration. The run is checked against its bucket's limit once
//!   (x_end = x + m + 2⌊(x mod 32 + m) / 30⌋, the run's one division), then split into a
//!   prefix fragment, complete groups and a suffix fragment. A nonuniform chunk, and the
//!   last n mod 16 rows, take the one-row path.
//! - **Complete groups** take pattern p = i mod 30 of the source row, whose fields are
//!   φ(i + j) − φ(i) = j + 2·[p + j ≥ 30], with header (φ(i) << 8) | byte, advancing by
//!   32 << 8 per group. No header is read and no RAW test made (natural configuration).
//! - **Fragments** write exactly their own fields. A group's first row (slot 0) sets its
//!   header unless the group is RAW (the bitmap). A later fragment reads the header, and if
//!   its last label exceeds base + 65,533 the group is promoted before the fragment is
//!   written: its earlier fields become RAW entries.
//! - **The end:** every cursor must equal its limit; the last group's unused fields are set
//!   to 0xFFFF, PARK is written, and the RAW groups are put in increasing-group order in place
//!   (word 31 a temporary destination tag, cycle rotation with one carry `RawG`, word 31 then
//!   zeroed). Only then is the record array's length set.
//!
//! `build::<true>` is the same constructor with untimed structural counters and coverage
//! tallies, and with its bounds checks as assertions; `build::<false>` has neither and is
//! the one whose code is inspected.

use super::{
    adversarial, blocks, compress, mix, phi, Counts, RawG, Rec, C30, CONFIGS, M20, MAX_OFF,
    MIN_SEGMENTED, RAWF, RAW_BIT, ROWS,
};
use mibz2::bwt::{InverseBwt, Segmented};
use std::io::Write;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The largest block whose PARK label, 32·⌈n/30⌉, fits 20 bits
const MAX_N: usize = ROWS * (M20 as usize / 32);

/// The thirty complete-record patterns. Pattern p's field j is φ(i + j) − φ(i) for a
/// source row i with i mod 30 = p; the header is zero.
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

/// The oracle's forced-RAW predicate, exactly as `C30::build` evaluates it
fn forced(g: usize, n: usize, permille: u32) -> bool {
    permille > 0 && (mix(g as u64 ^ (n as u64) << 32) % 1000) < permille as u64
}

fn rd(ll: &[u8], i: usize) -> u64 {
    u64::from_ne_bytes(ll[i..i + 8].try_into().unwrap())
}

/// The run-length histogram's bins, by their upper bounds (rows)
const BINS: [usize; 11] = [
    16,
    32,
    48,
    64,
    128,
    256,
    512,
    1024,
    4096,
    65_536,
    usize::MAX,
];

fn bin(m: usize) -> usize {
    BINS.iter().position(|&b| m <= b).unwrap()
}

/// The constructor's untimed structural counters and coverage tallies (`build::<true>`)
#[derive(Clone, Default, Debug)]
pub struct Stats {
    pub blocks: u64,
    pub rows: u64,
    /// rows emitted through complete-record patterns
    pub rows_complete: u64,
    /// rows of complete groups routed to the RAW writer (forced-RAW configurations only)
    pub rows_complete_raw: u64,
    /// rows written through a run's prefix or suffix fragment, and of those into RAW groups
    pub rows_fragment: u64,
    pub rows_fragment_raw: u64,
    /// rows on the one-row path (nonuniform chunks and the last n mod 16 rows), and of those
    /// into RAW groups
    pub rows_scalar: u64,
    pub rows_scalar_raw: u64,
    /// coalesced runs, their rows, and their lengths' histogram (count, rows, complete rows)
    pub runs: u64,
    pub run_rows: u64,
    pub run_hist: [u64; 11],
    pub run_hist_rows: [u64; 11],
    pub run_hist_complete: [u64; 11],
    /// complete groups, and the runs with at least one
    pub complete_groups: u64,
    pub complete_batches: u64,
    pub prefix_fragments: u64,
    pub suffix_fragments: u64,
    pub wide_promotions: u64,
    /// promotions by a fragment of two or more rows whose first offset fits and last doesn't
    pub promotions_partway: u64,
    pub reconstructed_fields: u64,
    pub raw_crossing_created: u64,
    pub raw_forced_created: u64,
    pub raw_groups: u64,
    /// RAW groups moved by the canonical ordering, its nontrivial cycles, the longest
    pub raw_moved: u64,
    pub raw_cycles: u64,
    pub raw_max_cycle: u64,
    /// coverage: fragments by (start slot, length), ordinary and RAW
    pub cov_frag: [[u64; 31]; 30],
    pub cov_frag_raw: [[u64; 31]; 30],
    /// coverage: complete groups by source phase
    pub cov_phase: [u64; 30],
    /// coverage: one-row writes by destination slot, ordinary and RAW
    pub cov_scalar: [u64; 30],
    pub cov_scalar_raw: [u64; 30],
    /// coverage: promotions by the prefix length reconstructed, through a fragment of a run
    /// and through the one-row path
    pub cov_promo_frag: [u64; 30],
    pub cov_promo_one: [u64; 30],
    /// capacities, in elements: requested by the latest build, and allocated after it;
    /// the maxima over builds; and the number of builds that had to allocate
    pub rec_req: usize,
    pub rec_cap: usize,
    pub raw_req: usize,
    pub raw_cap: usize,
    pub raw_len: usize,
    pub bitmap_req: usize,
    pub bitmap_cap: usize,
    pub max_rec_req: usize,
    pub max_rec_cap: usize,
    pub max_raw_req: usize,
    pub max_raw_cap: usize,
    pub max_raw_len: usize,
    pub max_bitmap_cap: usize,
    pub rec_allocs: u64,
    pub raw_allocs: u64,
    /// RAW pushes beyond the reserved capacity (a reallocation; expected zero)
    pub raw_overflows: u64,
    /// the allocator's usable bytes behind the record and RAW arenas (glibc
    /// `malloc_usable_size`, read by the harness after a build), the maxima
    pub max_rec_usable: usize,
    pub max_raw_usable: usize,
}

impl Stats {
    pub fn add(&mut self, o: &Stats) {
        macro_rules! sum {
            ($($f:ident),*) => { $( self.$f += o.$f; )* };
        }
        macro_rules! max {
            ($($f:ident),*) => { $( self.$f = self.$f.max(o.$f); )* };
        }
        macro_rules! arr {
            ($($f:ident),*) => { $( for (a, b) in self.$f.iter_mut().zip(o.$f.iter()) { *a += b; } )* };
        }
        sum!(
            blocks,
            rows,
            rows_complete,
            rows_complete_raw,
            rows_fragment,
            rows_fragment_raw,
            rows_scalar,
            rows_scalar_raw,
            runs,
            run_rows,
            complete_groups,
            complete_batches,
            prefix_fragments,
            suffix_fragments,
            wide_promotions,
            promotions_partway,
            reconstructed_fields,
            raw_crossing_created,
            raw_forced_created,
            raw_groups,
            raw_moved,
            raw_cycles,
            rec_allocs,
            raw_allocs,
            raw_overflows
        );
        max!(
            raw_max_cycle,
            max_rec_req,
            max_rec_cap,
            max_raw_req,
            max_raw_cap,
            max_raw_len,
            max_bitmap_cap,
            max_rec_usable,
            max_raw_usable
        );
        arr!(
            run_hist,
            run_hist_rows,
            run_hist_complete,
            cov_phase,
            cov_scalar,
            cov_scalar_raw,
            cov_promo_frag,
            cov_promo_one
        );
        for (a, b) in self.cov_frag.iter_mut().zip(o.cov_frag.iter()) {
            for (x, y) in a.iter_mut().zip(b) {
                *x += y;
            }
        }
        for (a, b) in self.cov_frag_raw.iter_mut().zip(o.cov_frag_raw.iter()) {
            for (x, y) in a.iter_mut().zip(b) {
                *x += y;
            }
        }
    }

    fn json(&self) -> String {
        let a = |v: &[u64]| format!("{v:?}");
        format!(
            "\"blocks\":{},\"rows\":{},\"rows_complete\":{},\"rows_complete_raw\":{},\"rows_fragment\":{},\"rows_fragment_raw\":{},\"rows_scalar\":{},\"rows_scalar_raw\":{},\"runs\":{},\"run_rows\":{},\"run_bins_upper\":[16,32,48,64,128,256,512,1024,4096,65536,null],\"run_hist\":{},\"run_hist_rows\":{},\"run_hist_complete_rows\":{},\"complete_groups\":{},\"complete_batches\":{},\"prefix_fragments\":{},\"suffix_fragments\":{},\"wide_promotions\":{},\"promotions_partway\":{},\"reconstructed_fields\":{},\"raw_crossing_created\":{},\"raw_forced_created\":{},\"raw_groups\":{},\"raw_moved\":{},\"raw_cycles\":{},\"raw_max_cycle\":{},\"max_rec_req\":{},\"max_rec_cap\":{},\"max_raw_req\":{},\"max_raw_cap\":{},\"max_raw_len\":{},\"max_bitmap_cap\":{},\"rec_allocs\":{},\"raw_allocs\":{},\"raw_overflows\":{},\"max_rec_usable_bytes\":{},\"max_raw_usable_bytes\":{}",
            self.blocks,
            self.rows,
            self.rows_complete,
            self.rows_complete_raw,
            self.rows_fragment,
            self.rows_fragment_raw,
            self.rows_scalar,
            self.rows_scalar_raw,
            self.runs,
            self.run_rows,
            a(&self.run_hist),
            a(&self.run_hist_rows),
            a(&self.run_hist_complete),
            self.complete_groups,
            self.complete_batches,
            self.prefix_fragments,
            self.suffix_fragments,
            self.wide_promotions,
            self.promotions_partway,
            self.reconstructed_fields,
            self.raw_crossing_created,
            self.raw_forced_created,
            self.raw_groups,
            self.raw_moved,
            self.raw_cycles,
            self.raw_max_cycle,
            self.max_rec_req,
            self.max_rec_cap,
            self.max_raw_req,
            self.max_raw_cap,
            self.max_raw_len,
            self.max_bitmap_cap,
            self.rec_allocs,
            self.raw_allocs,
            self.raw_overflows,
            self.max_rec_usable,
            self.max_raw_usable
        )
    }
}

/// The constructor's reusable scratch: the RAW bitmap, and the counters
#[derive(Default)]
pub struct Ctor {
    bitmap: Vec<u64>,
    pub st: Stats,
}

/// A bounds condition: asserted in the counted build, debug-asserted in the other
macro_rules! chk {
    ($c:ident, $($t:tt)*) => {
        if $c { assert!($($t)*) } else { debug_assert!($($t)*) }
    };
}

/// One build's state
struct W<'a, const COUNT: bool> {
    /// the record arena: `g_count + 1` records of capacity, none yet initialised
    dst: *mut Rec,
    g_count: usize,
    raw: &'a mut Vec<RawG>,
    bitmap: &'a mut [u64],
    /// the buckets' encoded cursors and limits
    x: [u32; 256],
    xl: [u32; 256],
    /// the source row's label φ(i) and phase i mod 30
    y: u32,
    p: usize,
    st: &'a mut Stats,
}

impl<const COUNT: bool> W<'_, COUNT> {
    #[inline(always)]
    fn is_raw(&self, g: usize) -> bool {
        self.bitmap[g >> 6] >> (g & 63) & 1 != 0
    }

    /// Group g's header. SAFETY: g < g_count, and its header written in this build
    #[inline(always)]
    unsafe fn header(&self, g: usize) -> u32 {
        chk!(COUNT, g < self.g_count);
        ptr::addr_of!((*self.dst.add(g)).h).read()
    }

    /// SAFETY: g < g_count
    #[inline(always)]
    unsafe fn set_header(&mut self, g: usize, h: u32) {
        chk!(COUNT, g < self.g_count);
        ptr::addr_of_mut!((*self.dst.add(g)).h).write(h)
    }

    /// Group g's first field. SAFETY: g < g_count
    #[inline(always)]
    unsafe fn fields(&self, g: usize) -> *mut u16 {
        chk!(COUNT, g < self.g_count);
        ptr::addr_of_mut!((*self.dst.add(g)).f) as *mut u16
    }

    /// The source advanced by k ≤ 30 rows: one wrap adjustment
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

    /// Group g made RAW, all 64 bytes of its record written, with a zeroed `RawG`.
    /// SAFETY: g < g_count, and g not already RAW
    unsafe fn create_raw(&mut self, g: usize) -> u32 {
        chk!(COUNT, g < self.g_count && !self.is_raw(g));
        let idx = self.raw.len() as u32;
        if COUNT && self.raw.len() == self.raw.capacity() {
            self.st.raw_overflows += 1;
        }
        self.raw.push(RawG([0; 32]));
        self.dst.add(g).write(Rec {
            f: [RAWF; 30],
            h: RAW_BIT | idx,
        });
        self.bitmap[g >> 6] |= 1 << (g & 63);
        idx
    }

    /// Ordinary group g, header h and fields 0..j written, promoted to RAW before its next
    /// write: a zeroed `RawG` appended and its j fields reconstructed into it as RAW entries
    /// (base + offset, F), then the record replaced. SAFETY: g < g_count, g not RAW, and its
    /// fields 0..j written in this build
    #[cold]
    #[inline(never)]
    unsafe fn promote(&mut self, g: usize, j: usize, h: u32, one: bool) -> u32 {
        chk!(COUNT, j >= 1 && j < ROWS);
        chk!(COUNT, g < self.g_count && !self.is_raw(g));
        // the old base and F byte, then a zeroed RawG holding the j reconstructed entries
        let (base, fb) = (h >> 8, h & 0xFF);
        let f = self.fields(g);
        let idx = self.raw.len() as u32;
        if COUNT && self.raw.len() == self.raw.capacity() {
            self.st.raw_overflows += 1;
        }
        self.raw.push(RawG([0; 32]));
        let e = &mut self.raw[idx as usize].0;
        for (t, w) in e[..j].iter_mut().enumerate() {
            *w = (base + f.add(t).read() as u32) << 8 | fb;
        }
        // then the record: every field 0xFFFF, the provisional RAW header, the bit
        self.dst.add(g).write(Rec {
            f: [RAWF; 30],
            h: RAW_BIT | idx,
        });
        self.bitmap[g >> 6] |= 1 << (g & 63);
        if COUNT {
            self.st.wide_promotions += 1;
            self.st.reconstructed_fields += j as u64;
            if one {
                self.st.cov_promo_one[j] += 1;
            } else {
                self.st.cov_promo_frag[j] += 1;
            }
        }
        idx
    }

    /// k rows of byte c (1 ≤ k ≤ 30 − j) at group g's slots j.., from the source rows at
    /// label y, phase p; the source then advanced by k. Writes exactly k fields (or k RAW
    /// entries), never past slot j + k − 1. SAFETY: g < g_count; the slots are the next k of
    /// c's bucket, checked against its limit
    #[inline(always)]
    unsafe fn fragment(&mut self, c: u8, g: usize, j: usize, k: usize) {
        chk!(
            COUNT,
            g < self.g_count && j < ROWS && k >= 1 && j + k <= ROWS
        );
        let pat = &PATTERNS[self.p].f;
        let y = self.y;
        let idx = if j == 0 {
            if self.is_raw(g) {
                Some(self.header(g) & M20)
            } else {
                // the group's first row: its header
                self.set_header(g, y << 8 | c as u32);
                let f = self.fields(g);
                for (t, &o) in pat[..k].iter().enumerate() {
                    f.add(t).write(o);
                }
                None
            }
        } else {
            let h = self.header(g);
            if h & RAW_BIT != 0 {
                Some(h & M20)
            } else {
                chk!(COUNT, h as u8 == c);
                let d = y - (h >> 8);
                // the labels increase, so the last one bounds every offset
                if d + pat[k - 1] as u32 > MAX_OFF {
                    if COUNT && k >= 2 && d <= MAX_OFF {
                        self.st.promotions_partway += 1;
                    }
                    Some(self.promote(g, j, h, false))
                } else {
                    let f = self.fields(g).add(j);
                    let d = d as u16;
                    for (t, &o) in pat[..k].iter().enumerate() {
                        f.add(t).write(o + d);
                    }
                    None
                }
            }
        };
        if let Some(idx) = idx {
            let e = &mut self.raw[idx as usize].0[j..j + k];
            for (w, &o) in e.iter_mut().zip(&pat[..k]) {
                *w = (y + o as u32) << 8 | c as u32;
            }
        }
        if COUNT {
            if idx.is_some() {
                self.st.cov_frag_raw[j][k] += 1;
            } else {
                self.st.cov_frag[j][k] += 1;
            }
        }
        self.advance(k);
    }

    /// `whole` complete groups of byte c from group g, from the pattern of the source's
    /// phase: no header read, no RAW test. SAFETY: g + whole ≤ g_count, the groups' slots
    /// the next 30·whole of c's bucket, checked against its limit
    #[inline(always)]
    unsafe fn complete(&mut self, c: u8, g: usize, whole: usize) {
        chk!(COUNT, g + whole <= self.g_count);
        let mut r = PATTERNS[self.p];
        r.h = self.y << 8 | c as u32;
        let d = self.dst.add(g);
        for t in 0..whole {
            d.add(t).write(r);
            r.h += 32 << 8;
        }
        self.y += 32 * whole as u32;
        if COUNT {
            self.st.rows_complete += 30 * whole as u64;
            self.st.complete_groups += whole as u64;
            self.st.complete_batches += 1;
            self.st.cov_phase[self.p] += whole as u64;
        }
    }

    /// `complete` in a forced-RAW configuration: a group marked RAW takes the RAW writer
    #[inline(never)]
    unsafe fn complete_checked(&mut self, c: u8, g: usize, whole: usize) {
        chk!(COUNT, g + whole <= self.g_count);
        let mut any = false;
        for t in 0..whole {
            if self.is_raw(g + t) {
                self.fragment(c, g + t, 0, ROWS);
                if COUNT {
                    self.st.rows_complete_raw += 30;
                    self.st.cov_frag_raw[0][30] -= 1;
                }
            } else {
                let mut r = PATTERNS[self.p];
                r.h = self.y << 8 | c as u32;
                self.dst.add(g + t).write(r);
                self.y += 32;
                any = true;
                if COUNT {
                    self.st.rows_complete += 30;
                    self.st.complete_groups += 1;
                    self.st.cov_phase[self.p] += 1;
                }
            }
        }
        if COUNT && any {
            self.st.complete_batches += 1;
        }
    }

    /// A coalesced run of m rows of byte c: one limit check, then the prefix fragment, the
    /// complete groups and the suffix fragment. False if it overflows its bucket, before
    /// any write.
    #[inline(always)]
    unsafe fn run(&mut self, c: u8, m: usize, forced: bool) -> bool {
        let ci = c as usize;
        let x = self.x[ci];
        let j = (x & 31) as usize;
        let tot = j + m;
        // the run's one division
        let q = tot / ROWS;
        let x_end = x + m as u32 + 2 * q as u32;
        if x_end > self.xl[ci] {
            return false;
        }
        self.x[ci] = x_end;
        if COUNT {
            self.st.runs += 1;
            self.st.run_rows += m as u64;
            self.st.run_hist[bin(m)] += 1;
            self.st.run_hist_rows[bin(m)] += m as u64;
        }
        let c0 = if COUNT { self.st.rows_complete } else { 0 };
        let mut g = (x >> 5) as usize;
        let mut whole = q;
        if j != 0 {
            let k = if q == 0 { m } else { ROWS - j };
            self.fragment(c, g, j, k);
            if COUNT {
                self.st.prefix_fragments += 1;
                self.st.rows_fragment += k as u64;
                if self.is_raw(g) {
                    self.st.rows_fragment_raw += k as u64;
                }
            }
            if q == 0 {
                return true;
            }
            g += 1;
            whole -= 1;
        }
        if whole > 0 {
            if forced {
                self.complete_checked(c, g, whole);
            } else {
                self.complete(c, g, whole);
            }
            g += whole;
        }
        let r = tot - ROWS * q;
        if r > 0 {
            self.fragment(c, g, 0, r);
            if COUNT {
                self.st.suffix_fragments += 1;
                self.st.rows_fragment += r as u64;
                if self.is_raw(g) {
                    self.st.rows_fragment_raw += r as u64;
                }
            }
        }
        if COUNT {
            self.st.run_hist_complete[bin(m)] += self.st.rows_complete - c0;
        }
        true
    }

    /// The one-row path: byte c at its bucket's next slot. False if the bucket is full,
    /// before any write.
    #[inline(always)]
    unsafe fn one(&mut self, c: u8) -> bool {
        let ci = c as usize;
        let x = self.x[ci];
        if x >= self.xl[ci] {
            return false;
        }
        // the next slot: an increment, skipping the two header labels after slot 29
        self.x[ci] = x + 1 + 2 * ((x & 31) == 29) as u32;
        let (g, j) = ((x >> 5) as usize, (x & 31) as usize);
        let y = self.y;
        let e = y << 8 | c as u32;
        let idx = if j == 0 {
            if self.is_raw(g) {
                Some(self.header(g) & M20)
            } else {
                self.set_header(g, e);
                self.fields(g).write(0);
                None
            }
        } else {
            let h = self.header(g);
            if h & RAW_BIT != 0 {
                Some(h & M20)
            } else {
                chk!(COUNT, h as u8 == c);
                let d = y - (h >> 8);
                if d > MAX_OFF {
                    Some(self.promote(g, j, h, true))
                } else {
                    self.fields(g).add(j).write(d as u16);
                    None
                }
            }
        };
        if let Some(idx) = idx {
            self.raw[idx as usize].0[j] = e;
        }
        if COUNT {
            self.st.rows_scalar += 1;
            if idx.is_some() {
                self.st.rows_scalar_raw += 1;
                self.st.cov_scalar_raw[j] += 1;
            } else {
                self.st.cov_scalar[j] += 1;
            }
        }
        self.advance(1);
        true
    }
}

/// The RAW groups put in increasing-group order, in place. Each RAW record's provisional
/// index is replaced by its rank among the RAW groups, which is stored in its `RawG`'s word
/// 31 (padding) as the destination tag; the groups are then permuted by cycle rotation with
/// one carry `RawG`, and every word 31 is zeroed again. Returns (groups moved, nontrivial
/// cycles, the longest cycle).
///
/// SAFETY: every group whose bit is set has its record's header written, holding its
/// provisional index; the set bits and the RAW groups correspond one to one; word 31 of
/// every RAW group is zero.
pub unsafe fn canonicalise(dst: *mut Rec, bitmap: &[u64], raw: &mut [RawG]) -> (u64, u64, u64) {
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

impl Ctor {
    /// `rec` and `raw` for the last column `ll` and its `counts`, exactly as `C30::build`
    /// gives them for the same `permille`. False, with both arrays empty, if the counts
    /// don't sum to n, n is out of range, or a bucket overflows (the counts are not `ll`'s
    /// own): production's one-pass fallback condition. Capacities are reused; an
    /// insufficient allocation is released before its replacement is made.
    #[inline(never)]
    pub fn build<const COUNT: bool>(
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
        let (mut x, mut xl) = ([0u32; 256], [0u32; 256]);
        let (mut cum, mut sigma) = (0usize, 0usize);
        for c in 0..256 {
            x[c] = phi(cum);
            cum += counts[c] as usize;
            xl[c] = phi(cum);
            sigma += (counts[c] > 0) as usize;
        }
        let forced_cfg = permille > 0;
        // the natural RAW bound, or every group in a forced-RAW configuration
        let raw_req = if forced_cfg {
            g_count
        } else if n == 0 {
            0
        } else {
            g_count.min(sigma * (phi(n - 1) as usize / 65_534) + sigma - 1)
        };
        let mut rec_alloc = false;
        if rec.capacity() < g_count + 1 {
            *rec = Vec::new();
            rec.reserve_exact(g_count + 1);
            rec_alloc = true;
        }
        let mut raw_alloc = false;
        if raw.capacity() < raw_req {
            *raw = Vec::new();
            raw.reserve_exact(raw_req);
            raw_alloc = true;
        }
        let words = g_count.div_ceil(64);
        let Ctor { bitmap, st } = self;
        bitmap.clear();
        bitmap.resize(words, 0);
        assert_eq!(rec.as_ptr() as usize % 64, 0, "records 64-byte aligned");
        let ok = {
            let mut w = W::<COUNT> {
                dst: rec.as_mut_ptr(),
                g_count,
                raw: &mut *raw,
                bitmap,
                x,
                xl,
                y: 0,
                p: 0,
                st: &mut *st,
            };
            // SAFETY: every record index below is under g_count + 1, the arena's capacity;
            // see the write-up for the initialisation argument
            unsafe { Self::scatter(&mut w, ll, counts, permille) }
        };
        if !ok {
            raw.clear();
            return false;
        }
        // SAFETY: as above
        unsafe {
            let dst = rec.as_mut_ptr();
            // the last group's unused fields
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
            let (moved, cycles, longest) = canonicalise(dst, bitmap, raw);
            if COUNT {
                st.raw_moved += moved;
                st.raw_cycles += cycles;
                st.raw_max_cycle = st.raw_max_cycle.max(longest);
            }
            // every record below g_count + 1 is now fully initialised
            rec.set_len(g_count + 1);
        }
        assert_eq!(raw.as_ptr() as usize % 64, 0, "RAW groups 64-byte aligned");
        if COUNT {
            st.blocks += 1;
            st.rows += n as u64;
            st.raw_groups += raw.len() as u64;
            st.rec_req = g_count + 1;
            st.rec_cap = rec.capacity();
            st.raw_req = raw_req;
            st.raw_cap = raw.capacity();
            st.raw_len = raw.len();
            st.bitmap_req = words;
            st.bitmap_cap = bitmap.capacity();
            st.max_rec_req = st.max_rec_req.max(st.rec_req);
            st.max_rec_cap = st.max_rec_cap.max(st.rec_cap);
            st.max_raw_req = st.max_raw_req.max(raw_req);
            st.max_raw_cap = st.max_raw_cap.max(st.raw_cap);
            st.max_raw_len = st.max_raw_len.max(st.raw_len);
            st.max_bitmap_cap = st.max_bitmap_cap.max(st.bitmap_cap);
            st.rec_allocs += rec_alloc as u64;
            st.raw_allocs += raw_alloc as u64;
        }
        true
    }

    /// The RAW groups known up front, then the scan of L. False at the first overflowing
    /// bucket (before its write) or if any cursor ends short of its limit.
    #[inline(always)]
    unsafe fn scatter<const COUNT: bool>(
        w: &mut W<'_, COUNT>,
        ll: &[u8],
        counts: &[u32; 256],
        permille: u32,
    ) -> bool {
        let n = ll.len();
        // the crossing groups, from the bucket boundaries
        let mut cum = 0usize;
        let mut last = usize::MAX;
        for &c in counts.iter() {
            cum += c as usize;
            if cum > 0 && cum < n && cum % ROWS != 0 {
                let g = cum / ROWS;
                if g != last {
                    w.create_raw(g);
                    last = g;
                    if COUNT {
                        w.st.raw_crossing_created += 1;
                    }
                }
            }
        }
        let forced_cfg = permille > 0;
        if forced_cfg {
            for g in 0..w.g_count {
                if !w.is_raw(g) && forced(g, n, permille) {
                    w.create_raw(g);
                    if COUNT {
                        w.st.raw_forced_created += 1;
                    }
                }
            }
        }
        let mut i = 0;
        let mut held: Option<(u64, u64)> = None;
        while i + 16 <= n {
            let (a, b) = match held.take() {
                Some(v) => v,
                None => (rd(ll, i), rd(ll, i + 8)),
            };
            if a == b && a == a.rotate_left(8) {
                // coalesce the following chunks of the same byte; keep the first that fails
                let mut e = i + 16;
                while e + 16 <= n {
                    let (a2, b2) = (rd(ll, e), rd(ll, e + 8));
                    if a2 == a && b2 == a {
                        e += 16;
                    } else {
                        held = Some((a2, b2));
                        break;
                    }
                }
                if !w.run(a as u8, e - i, forced_cfg) {
                    return false;
                }
                i = e;
            } else {
                for &c in &ll[i..i + 16] {
                    if !w.one(c) {
                        return false;
                    }
                }
                i += 16;
            }
        }
        for &c in &ll[i..] {
            if !w.one(c) {
                return false;
            }
        }
        w.x == w.xl
    }
}

// ---------------------------------------------------------------------------------------
// The correctness harness
// ---------------------------------------------------------------------------------------

fn bytes<T>(v: &[T]) -> &[u8] {
    // SAFETY: Rec and RawG have no padding (their sizes are their fields' sizes)
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v)) }
}

/// The first difference between the oracle's arrays and the candidate's, if any
fn diff(o: &C30, rec: &[Rec], raw: &[RawG]) -> Option<String> {
    if o.rec.len() != rec.len() {
        return Some(format!("rec length {} vs {}", o.rec.len(), rec.len()));
    }
    if o.raw.len() != raw.len() {
        return Some(format!("raw length {} vs {}", o.raw.len(), raw.len()));
    }
    if bytes(&o.rec) != bytes(rec) {
        let g = (0..rec.len())
            .find(|&g| bytes(&o.rec[g..g + 1]) != bytes(&rec[g..g + 1]))
            .unwrap();
        return Some(format!(
            "record {g} of {}: oracle h {:#x} f {:?}, candidate h {:#x} f {:?}",
            rec.len(),
            o.rec[g].h,
            o.rec[g].f,
            rec[g].h,
            rec[g].f
        ));
    }
    if bytes(&o.raw) != bytes(raw) {
        let k = (0..raw.len()).find(|&k| o.raw[k].0 != raw[k].0).unwrap();
        return Some(format!(
            "RAW group {k}: oracle {:?}, candidate {:?}",
            o.raw[k].0, raw[k].0
        ));
    }
    None
}

/// The groups' classes from the arrays, by the oracle's precedence (crossing, then wide,
/// then forced): (ordinary, crossing, wide, forced)
fn classify(rec: &[Rec], raw: &[RawG], counts: &[u32; 256], n: usize) -> [usize; 4] {
    let g_count = n.div_ceil(ROWS);
    let mut crossing = vec![false; g_count];
    let mut cum = 0usize;
    for &c in counts {
        cum += c as usize;
        if cum > 0 && cum < n && cum % ROWS != 0 {
            crossing[cum / ROWS] = true;
        }
    }
    let mut k = [0usize; 4];
    for g in 0..g_count {
        let h = rec[g].h;
        if h & RAW_BIT == 0 {
            k[0] += 1;
        } else if crossing[g] {
            k[1] += 1;
        } else {
            let e = &raw[(h & M20) as usize].0;
            let rows = (n - ROWS * g).min(ROWS);
            if (e[rows - 1] >> 8) - (e[0] >> 8) > MAX_OFF {
                k[2] += 1;
            } else {
                k[3] += 1;
            }
        }
    }
    k
}

/// Fills the record arena's spare capacity with a pattern no build writes, so a field the
/// constructor leaves unwritten shows as a difference
fn poison(rec: &mut Vec<Rec>) {
    rec.clear();
    for s in rec.spare_capacity_mut() {
        s.write(Rec {
            f: [0xA5A5; 30],
            h: 0xA5A5_A5A5,
        });
    }
}

/// The allocator's usable size of a vector's allocation (0 if it has none, or off glibc)
fn usable<T>(v: &Vec<T>) -> usize {
    #[cfg(all(target_os = "linux", target_env = "gnu", not(miri)))]
    {
        extern "C" {
            fn malloc_usable_size(p: *mut std::ffi::c_void) -> usize;
        }
        if v.capacity() == 0 || std::mem::size_of::<T>() == 0 {
            return 0;
        }
        // SAFETY: the pointer is a live allocation of the system allocator (glibc malloc)
        unsafe { malloc_usable_size(v.as_ptr() as *mut std::ffi::c_void) }
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu", not(miri))))]
    {
        let _ = v;
        0
    }
}

fn counts_dbg(c: &Counts) -> String {
    format!("{c:?}")
}

/// One worker's arenas, reused from block to block (repeated construction with changing
/// sizes)
#[derive(Default)]
struct Arenas {
    oracle: C30,
    cand: C30,
    ctor: Ctor,
    out: Vec<u8>,
    out_o: Vec<u8>,
    prod: Segmented,
    want: Vec<u8>,
}

/// What `check_block` checked
#[derive(Default, Clone, Copy)]
struct Checked {
    array_checks: u64,
    walk_checks: u64,
    restoration_checks: u64,
}

impl Checked {
    fn add(&mut self, o: &Checked) {
        self.array_checks += o.array_checks;
        self.walk_checks += o.walk_checks;
        self.restoration_checks += o.restoration_checks;
    }
}

/// One block, one configuration:
/// 1. the oracle's build;
/// 2. the counted build, into the (optionally poisoned) reused arena: its arrays against
///    the oracle's, byte for byte and in length, and its classes against the oracle's;
/// 3. the uncounted build, again into the same arena: the same comparison;
/// 4. with `want`, the unchanged walker on the candidate's arrays, counted and audited:
///    each output against production's and the oracle's (counted, with the transition
///    counts), and the arrays against the oracle's again after each walk.
#[allow(clippy::too_many_arguments)]
fn check_block(
    a: &mut Arenas,
    what: &str,
    ll: &[u8],
    counts: &[u32; 256],
    op: u32,
    want: Option<&[u8]>,
    pm: u32,
    extra: bool,
    poisoned: bool,
    ck: &mut Checked,
) -> Stats {
    let n = ll.len();
    let ob = a.oracle.build(ll, counts, pm);
    if poisoned {
        poison(&mut a.cand.rec);
    }
    let before = std::mem::take(&mut a.ctor.st);
    assert!(
        a.ctor
            .build::<true>(ll, counts, pm, &mut a.cand.rec, &mut a.cand.raw),
        "{what}: the counted build rejected valid counts"
    );
    let mut st = std::mem::replace(&mut a.ctor.st, before);
    st.max_rec_usable = usable(&a.cand.rec);
    st.max_raw_usable = usable(&a.cand.raw);
    if let Some(d) = diff(&a.oracle, &a.cand.rec, &a.cand.raw) {
        panic!("{what} pm {pm}: the counted build differs: {d}");
    }
    let k = classify(&a.cand.rec, &a.cand.raw, counts, n);
    assert_eq!(
        k,
        [ob.ordinary, ob.raw_crossing, ob.raw_wide, ob.raw_forced],
        "{what} pm {pm}: the classes differ"
    );
    if poisoned {
        poison(&mut a.cand.rec);
    }
    assert!(
        a.ctor
            .build::<false>(ll, counts, pm, &mut a.cand.rec, &mut a.cand.raw),
        "{what}: the uncounted build rejected valid counts"
    );
    if let Some(d) = diff(&a.oracle, &a.cand.rec, &a.cand.raw) {
        panic!("{what} pm {pm}: the uncounted build differs: {d}");
    }
    ck.array_checks += 2;
    if let Some(want) = want {
        a.cand.n = n;
        let (mut co, mut cc) = (Counts::default(), Counts::default());
        a.oracle.walk::<true>(op, extra, &mut a.out_o, &mut co);
        a.cand.walk::<true>(op, extra, &mut a.out, &mut cc);
        assert!(
            a.out == want,
            "{what} pm {pm} extra {extra}: counted walk differs from production"
        );
        assert!(
            a.out == a.out_o,
            "{what} pm {pm} extra {extra}: counted walk differs from the oracle's"
        );
        assert_eq!(
            counts_dbg(&co),
            counts_dbg(&cc),
            "{what}: transition counts differ"
        );
        if let Some(d) = diff(&a.oracle, &a.cand.rec, &a.cand.raw) {
            panic!("{what} pm {pm}: after the counted walk: {d}");
        }
        let mut dummy = Counts::default();
        a.cand.walk::<false>(op, extra, &mut a.out, &mut dummy);
        assert!(
            a.out == want,
            "{what} pm {pm} extra {extra}: audited walk differs from production"
        );
        if let Some(d) = diff(&a.oracle, &a.cand.rec, &a.cand.raw) {
            panic!("{what} pm {pm}: after the audited walk: {d}");
        }
        ck.walk_checks += 3;
        ck.restoration_checks += 2;
    }
    st
}

/// The corpora's (or adversarial inputs') blocks under every configuration
fn check_input(name: &str, data: &[u8], jl: &mut std::fs::File) -> Vec<(u32, bool, Stats)> {
    let bs = blocks(data);
    let mut res = Vec::new();
    for &(pm, extra) in &CONFIGS {
        let next = AtomicUsize::new(0);
        let threads = 3usize;
        let results: Vec<(Stats, Checked, usize)> = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..threads)
                .map(|_| {
                    sc.spawn(|| {
                        let mut a = Arenas::default();
                        let (mut st, mut ck, mut segb) = (Stats::default(), Checked::default(), 0);
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some((ll, counts, op, want)) = bs.get(i) else {
                                break;
                            };
                            let what = format!("{name} block {i}");
                            let s = check_block(
                                &mut a,
                                &what,
                                ll,
                                counts,
                                *op,
                                Some(want.as_slice()),
                                pm,
                                extra,
                                true,
                                &mut ck,
                            );
                            st.add(&s);
                            segb += (ll.len() >= MIN_SEGMENTED) as usize;
                        }
                        (st, ck, segb)
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let (mut st, mut ck, mut segb) = (Stats::default(), Checked::default(), 0);
        for (s, c, b) in &results {
            st.add(s);
            ck.add(c);
            segb += b;
        }
        writeln!(
            jl,
            "{{\"input\":{name:?},\"permille\":{pm},\"extra_starts\":{extra},\"segmented_blocks\":{segb},\"array_checks\":{},\"walk_checks\":{},\"restoration_checks\":{},{}}}",
            ck.array_checks,
            ck.walk_checks,
            ck.restoration_checks,
            st.json()
        )
        .unwrap();
        println!(
            "{name}: permille {pm}, extra starts {extra}: {} blocks ({segb} segmented), {} rows: {} array comparisons, {} walk outputs, {} restorations exact; complete {:.2}%, fragment {:.2}%, one-row {:.2}%; runs {}, promotions {}, RAW {} (moved {})",
            st.blocks,
            st.rows,
            ck.array_checks,
            ck.walk_checks,
            ck.restoration_checks,
            pct(st.rows_complete, st.rows),
            pct(st.rows_fragment, st.rows),
            pct(st.rows_scalar, st.rows),
            st.runs,
            st.wide_promotions,
            st.raw_groups,
            st.raw_moved
        );
        res.push((pm, extra, st));
    }
    res
}

fn pct(a: u64, b: u64) -> f64 {
    if b == 0 {
        0.0
    } else {
        100.0 * a as f64 / b as f64
    }
}

/// The natural configuration's counters, one row per input, and the total
fn table(rows: &[(String, Stats)]) {
    println!();
    println!("| input | blocks | rows | complete | fragment | one-row | runs | run rows | wide promotions | reconstructed | RAW groups | moved | cycles |");
    println!("|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|");
    let mut t = Stats::default();
    for (name, s) in rows {
        t.add(s);
        line(name, s);
    }
    line("**total**", &t);
}

fn line(name: &str, s: &Stats) {
    println!(
        "| {name} | {} | {} | {:.3}% | {:.3}% | {:.3}% | {} | {:.3}% | {} | {} | {} | {} | {} |",
        s.blocks,
        s.rows,
        pct(s.rows_complete, s.rows),
        pct(s.rows_fragment, s.rows),
        pct(s.rows_scalar, s.rows),
        s.runs,
        pct(s.run_rows, s.rows),
        s.wide_promotions,
        s.reconstructed_fields,
        s.raw_groups,
        s.raw_moved,
        s.raw_cycles
    );
}

// ---------------------------------------------------------------------------------------
// Direct synthetic L/counts cases
// ---------------------------------------------------------------------------------------

fn hist(ll: &[u8]) -> [u32; 256] {
    let mut c = [0u32; 256];
    for &b in ll {
        c[b as usize] += 1;
    }
    c
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = mix(self.0.wrapping_add(0x9e37_79b9_7f4a_7c15));
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Filler that never forms a uniform chunk: bytes 2 and 3 alternating
fn filler(v: &mut Vec<u8>, len: usize) {
    v.extend((0..len).map(|k| 2 + (k & 1) as u8));
}

/// The synthetic suite's driver: every case checked as `check_block` does, the walks
/// against production's own inversion of the same L and counts
struct Synth {
    a: Arenas,
    st: Stats,
    ck: Checked,
    cases: u64,
    walk: bool,
    poisoned: bool,
}

impl Synth {
    fn case(&mut self, what: &str, ll: &[u8], pm: u32) -> Stats {
        let counts = hist(ll);
        let n = ll.len();
        let op = if n == 0 { 0 } else { (n / 2) as u32 };
        let want = if self.walk && n > 0 {
            let mut w = std::mem::take(&mut self.a.want);
            self.a.prod.invert(ll, &counts, op, &mut w);
            Some(w)
        } else {
            None
        };
        let s = if n == 0 {
            // the oracle needs n ≥ 1 (its RAW bound); the empty block is PARK alone
            assert!(self.a.ctor.build::<true>(
                ll,
                &counts,
                pm,
                &mut self.a.cand.rec,
                &mut self.a.cand.raw
            ));
            assert_eq!(self.a.cand.rec.len(), 1);
            assert!(self.a.cand.raw.is_empty());
            assert!(
                self.a.cand.rec[0]
                    == Rec {
                        f: [RAWF; 30],
                        h: RAW_BIT | M20
                    }
            );
            Stats::default()
        } else {
            check_block(
                &mut self.a,
                what,
                ll,
                &counts,
                op,
                want.as_deref(),
                pm,
                true,
                self.poisoned,
                &mut self.ck,
            )
        };
        if let Some(w) = want {
            self.a.want = w;
        }
        self.st.add(&s);
        self.cases += 1;
        s
    }
}

/// The span and promotion cases: bucket 1 (the smallest byte present, so starting at
/// slot 0 when no byte 0 is present) gets `early` rows at sources 0, 2, 4, …, then rows
/// from `late`; the rest of L is filler. Byte 0 (`a0` rows, at the end) shifts the
/// destination.
fn span_case(
    early: &[usize],
    late: &[usize],
    run: Option<(usize, usize)>,
    n: usize,
    a0: usize,
) -> Vec<u8> {
    let mut v = Vec::with_capacity(n);
    filler(&mut v, n - a0);
    for &s in early.iter().chain(late) {
        v[s] = 1;
    }
    if let Some((b, m)) = run {
        for s in &mut v[b..b + m] {
            *s = 1;
        }
    }
    v.extend(std::iter::repeat(0u8).take(a0));
    v
}

fn synthetic(quick: bool) {
    let mut s = Synth {
        a: Arenas::default(),
        st: Stats::default(),
        ck: Checked::default(),
        cases: 0,
        walk: true,
        poisoned: !quick,
    };
    let mut classes: Vec<(String, u64)> = Vec::new();
    let mut mark = |s: &Synth, name: &str, from: u64| {
        classes.push((name.to_string(), s.cases - from));
        println!("ctor-synthetic: {name}: {} cases exact", s.cases - from);
    };
    let mut rng = Rng(0xc30c);

    // 1. small random blocks: every n from 0 to 130 (every partial-record length), a few
    //    alphabets and run structures
    let from = s.cases;
    let (nmax, seeds) = if quick { (64, 2) } else { (130, 8) };
    for n in 0..=nmax {
        for sigma in [1usize, 2, 3, 5] {
            for _ in 0..seeds {
                let mut v = Vec::with_capacity(n);
                while v.len() < n {
                    let len = [1, 2, 7, 16, 17, 30, 32, 33, 48, 60][rng.below(10)];
                    let b = (10 * rng.below(sigma)) as u8;
                    v.extend(std::iter::repeat(b).take(len.min(n - v.len())));
                }
                s.case(&format!("small n {n} sigma {sigma}"), &v, 0);
            }
        }
    }
    mark(&s, "small blocks, n 0..=130", from);

    // 2. a bucket boundary at every offset: two bytes, B rows of the first
    let from = s.cases;
    let ns: &[usize] = if quick {
        &[31, 61]
    } else {
        &[30, 59, 60, 61, 90, 97, 150]
    };
    for &n in ns {
        for bnd in 1..n {
            let mut sorted = vec![1u8; bnd];
            sorted.extend(std::iter::repeat(2u8).take(n - bnd));
            let rev: Vec<u8> = sorted.iter().rev().copied().collect();
            let alt: Vec<u8> = (0..n)
                .map(|k| if (k * 7 + 3) % n < bnd { 1 } else { 2 })
                .collect();
            for v in [sorted, rev, alt] {
                s.case(&format!("boundary n {n} B {bnd}"), &v, 0);
            }
        }
    }
    mark(&s, "bucket boundary at every offset", from);

    // 3. source phase × destination slot × run length: filler(P), then a run of byte 1,
    //    filler, then A rows of byte 0 (which puts byte 1's bucket at slot A)
    let from = s.cases;
    let (ps, ms): (Vec<usize>, Vec<usize>) = if quick {
        ((0..=48).step_by(13).collect(), vec![16, 32, 61])
    } else {
        (
            (0..=240).collect(),
            vec![
                1, 15, 16, 17, 29, 30, 31, 32, 33, 47, 48, 59, 60, 61, 64, 90, 120, 150, 256,
            ],
        )
    };
    let avals: Vec<usize> = if quick {
        vec![0, 1, 29]
    } else {
        (0..30).collect()
    };
    for &p in &ps {
        for &m in &ms {
            for &a0 in &avals {
                let mut v = Vec::new();
                filler(&mut v, p);
                v.extend(std::iter::repeat(1u8).take(m));
                filler(&mut v, 37);
                v.extend(std::iter::repeat(0u8).take(a0));
                s.case(&format!("phase P {p} m {m} A {a0}"), &v, 0);
            }
        }
    }
    mark(&s, "source phase x destination slot x run length", from);

    // 4. equal-run lengths 1..=200 (around 16, 30, 32, 60 and longer) at 32 offsets
    let from = s.cases;
    let (ms, offs, avals): (Vec<usize>, usize, &[usize]) = if quick {
        (vec![1, 15, 16, 17, 29, 30, 31, 32, 33, 47, 60], 2, &[0, 29])
    } else {
        ((1..=200).collect(), 32, &[0, 1, 13, 29])
    };
    for &m in &ms {
        for off in 0..offs {
            for &a0 in avals {
                let mut v = Vec::new();
                filler(&mut v, off);
                v.extend(std::iter::repeat(1u8).take(m));
                filler(&mut v, 5);
                v.extend(std::iter::repeat(1u8).take(m / 2));
                filler(&mut v, 3);
                v.extend(std::iter::repeat(0u8).take(a0));
                s.case(&format!("run m {m} off {off} A {a0}"), &v, 0);
            }
        }
    }
    mark(&s, "equal-run lengths 1..=200", from);

    if !quick {
        // 5. spans of exactly 65,533 and 65,534 (and around them), one-row rows: the group's
        //    first row at source a, 28 more at a + 2, a + 4, …, and its last at b
        let from = s.cases;
        let mut seen = [0u64; 2];
        for a in [0usize, 1, 2, 3, 5, 17, 29] {
            for a0 in [0usize, 7, 29] {
                // bucket 1's rows before the critical group, so that it starts at a slot 0
                let pre = (ROWS - a0 % ROWS) % ROWS;
                for target in 65_530u32..=65_538 {
                    let fb = phi(a + 100) + target;
                    if fb & 31 >= 30 {
                        continue;
                    }
                    let b = super::phi_inv(fb);
                    let early: Vec<usize> = (0..pre).map(|t| 1 + 2 * t).collect();
                    let group: Vec<usize> = (0..29).map(|t| a + 100 + 2 * t).chain([b]).collect();
                    let v = span_case(&early, &group, None, b + 50, a0);
                    let st = s.case(&format!("span one-row a {a} A {a0} target {target}"), &v, 0);
                    if target == 65_533 {
                        assert_eq!(st.wide_promotions, 0, "span 65,533 is ordinary");
                        seen[0] += 1;
                    }
                    if target == 65_534 {
                        assert_eq!(st.wide_promotions, 1, "span 65,534 is wide");
                        seen[1] += 1;
                    }
                }
            }
        }
        assert!(seen[0] > 0 && seen[1] > 0);
        println!(
            "ctor-synthetic: spans 65,533 (ordinary) in {} cases and 65,534 (wide) in {} cases",
            seen[0], seen[1]
        );
        mark(&s, "spans 65,530..=65,538 through one-row writes", from);

        // 6. promotion after every prefix length t = 1..=29, through a run's prefix
        //    fragment whose endpoint crosses the limit at every position: t early rows from
        //    source a, then a coalesced run at b (16-aligned) filling the group's other slots
        let from = s.cases;
        let mut exact_limit = 0;
        for a in 0..4usize {
            for t in 1..=29usize {
                for bk in 0..12usize {
                    let b = 61_440 - 128 + 16 * bk;
                    let early: Vec<usize> = (0..t).map(|u| a + 2 * u).collect();
                    let v = span_case(&early, &[], Some((b, 64)), b + 64 + 100, 0);
                    let st = s.case(&format!("promotion run a {a} t {t} b {b}"), &v, 0);
                    let last = phi(b + 29 - t) - phi(a);
                    assert_eq!(
                        st.wide_promotions,
                        (last > MAX_OFF) as u64,
                        "a {a} t {t} b {b}"
                    );
                    exact_limit += (last == MAX_OFF) as u64;
                }
            }
        }
        println!(
            "ctor-synthetic: a fragment ending exactly at offset 65,533 in {exact_limit} cases"
        );
        mark(
            &s,
            "promotion through a run fragment, every prefix length",
            from,
        );

        // 7. promotion through the one-row path after every prefix length
        let from = s.cases;
        for t in 1..=29usize {
            for a in [0usize, 3] {
                let early: Vec<usize> = (0..t).map(|u| a + 2 * u).collect();
                let late: Vec<usize> = (0..ROWS - t).map(|u| 61_500 + 2 * u).collect();
                let v = span_case(&early, &late, None, 61_700, 0);
                let st = s.case(&format!("promotion one-row a {a} t {t}"), &v, 0);
                assert_eq!(st.wide_promotions, 1);
                assert_eq!(st.reconstructed_fields, t as u64);
            }
        }
        mark(
            &s,
            "promotion through the one-row path, every prefix length",
            from,
        );

        // 8. RAW discovery order unlike group order: rare bytes spread over the block (wide
        //    groups, discovered late) among run-heavy filler (crossing groups up front)
        let from = s.cases;
        let (mut cyc, mut longest, mut moved) = (0u64, 0u64, 0u64);
        for seed in 0..6u64 {
            let mut r = Rng(seed);
            let n = [300_000usize, 450_001, 899_999][seed as usize % 3];
            let mut v = Vec::with_capacity(n);
            while v.len() < n {
                let b = 100 + r.below(4) as u8;
                let len = [1, 16, 40, 200, 1000][r.below(5)];
                v.extend(std::iter::repeat(b).take(len.min(n - v.len())));
            }
            for k in 0..(8 * (seed as usize + 3) * 45) {
                let at = r.below(n);
                v[at] = 1 + (k % (8 + seed as usize)) as u8;
            }
            for pm in [0, 50, 1000] {
                let st = s.case(&format!("permutation seed {seed} pm {pm}"), &v, pm);
                println!(
                    "ctor-synthetic: permutation seed {seed} n {n} pm {pm}: {} RAW groups ({} wide promotions), {} moved in {} cycles, the longest {}",
                    st.raw_groups, st.wide_promotions, st.raw_moved, st.raw_cycles, st.raw_max_cycle
                );
                cyc += st.raw_cycles;
                moved += st.raw_moved;
                longest = longest.max(st.raw_max_cycle);
            }
        }
        assert!(cyc > 0 && longest > 2, "nontrivial permutation cycles");
        println!("ctor-synthetic: permutations: {moved} groups moved in {cyc} cycles, the longest {longest}");
        mark(&s, "RAW discovery order unlike group order", from);

        // 9. one-symbol blocks
        let from = s.cases;
        for &n in &[
            1usize, 2, 15, 16, 17, 29, 30, 31, 32, 47, 48, 60, 61, 65_536, 899_999, 900_000,
        ] {
            for b in [0u8, 97, 255] {
                s.case(&format!("one symbol n {n} byte {b}"), &vec![b; n], 0);
            }
        }
        mark(&s, "one-symbol blocks", from);

        // 10. 900,000-row blocks (and the largest the oracle allows), several structures,
        //     each under the natural and the forced-RAW configurations
        let from = s.cases;
        for (kind, sigma) in [
            ("random", 256usize),
            ("random", 4),
            ("runs", 20),
            ("runs", 98),
            ("runs", 2),
        ] {
            for &n in &[900_000usize, MAX_N] {
                let mut v = Vec::with_capacity(n);
                while v.len() < n {
                    let b = rng.below(sigma) as u8;
                    let len = if kind == "random" {
                        1
                    } else {
                        [1, 3, 16, 31, 64, 500, 4000][rng.below(7)]
                    };
                    v.extend(std::iter::repeat(b).take(len.min(n - v.len())));
                }
                for pm in [0, 50, 1000] {
                    s.case(&format!("{kind} sigma {sigma} n {n} pm {pm}"), &v, pm);
                }
            }
        }
        mark(&s, "900,000-row and 983,010-row blocks", from);
    }

    // 11. repeated construction with changing sizes into the same arenas (unpoisoned and
    //     poisoned), the forced configuration changing the RAW requirement
    let from = s.cases;
    let sizes: &[usize] = if quick {
        &[200, 1, 95, 31, 150, 0, 61]
    } else {
        &[
            900_000, 1, 899_990, 31, 65_537, 30, 900_000, 59, 500_000, 0, 65_536, 899_999,
        ]
    };
    for round in 0..2 {
        s.poisoned = round == 1 && !quick;
        for (k, &n) in sizes.iter().enumerate() {
            let mut v = Vec::with_capacity(n);
            while v.len() < n {
                let b = (rng.below(6) * 40) as u8;
                let len = [1, 16, 30, 64, 300][rng.below(5)];
                v.extend(std::iter::repeat(b).take(len.min(n - v.len())));
            }
            let pm = [0, 1000, 50][k % 3];
            s.case(&format!("repeated round {round} size {n} pm {pm}"), &v, pm);
        }
    }
    s.poisoned = !quick;
    mark(&s, "repeated construction with changing sizes", from);

    // 12. malformed counts: each must be rejected before any write out of bounds, both
    //     arrays left empty, exactly when the counts are not ll's own
    let from = s.cases;
    let mut rejected = 0u64;
    let mut accepted = 0u64;
    let trials = if quick { 40 } else { 4000 };
    for t in 0..trials {
        let n = if quick {
            1 + rng.below(90)
        } else {
            [1, 29, 31, 95, 300, 4000, 70_000][t % 7]
        };
        let mut v = Vec::with_capacity(n);
        while v.len() < n {
            let b = (rng.below(4) * 3) as u8;
            let len = [1, 16, 32, 45][rng.below(4)];
            v.extend(std::iter::repeat(b).take(len.min(n - v.len())));
        }
        let own = hist(&v);
        let mut c = own;
        match t % 5 {
            0 => c[rng.below(256)] += 1,
            1 => {
                let b = v[rng.below(n)] as usize;
                c[b] -= 1;
            }
            2 | 3 => {
                // the same total, moved from one byte to another
                let from_b = v[rng.below(n)] as usize;
                let k = 1 + rng.below(c[from_b] as usize);
                c[from_b] -= k as u32;
                c[rng.below(256)] += k as u32;
            }
            _ => {
                // a permutation of the counts among the bytes present
                c.swap(0, 3);
                c.swap(6, 9);
            }
        }
        let pm = [0, 50, 1000][t % 3];
        let ok =
            s.a.ctor
                .build::<true>(&v, &c, pm, &mut s.a.cand.rec, &mut s.a.cand.raw);
        let ok2 =
            s.a.ctor
                .build::<false>(&v, &c, pm, &mut s.a.cand.rec, &mut s.a.cand.raw);
        assert_eq!(ok, ok2);
        assert_eq!(
            ok,
            c == own,
            "case {t}: rejected exactly when the counts are not ll's own"
        );
        if ok {
            accepted += 1;
        } else {
            rejected += 1;
            assert!(s.a.cand.rec.is_empty() && s.a.cand.raw.is_empty());
        }
        s.cases += 1;
    }
    if !quick {
        // the size limits: n ≥ 2^24, and one row past the largest PARK label
        for n in [MAX_N + 1, 1 << 24] {
            let v = vec![7u8; n];
            let c = hist(&v);
            assert!(!s
                .a
                .ctor
                .build::<true>(&v, &c, 0, &mut s.a.cand.rec, &mut s.a.cand.raw));
            assert!(s.a.cand.rec.is_empty() && s.a.cand.raw.is_empty());
            rejected += 1;
            s.cases += 1;
        }
    }
    println!("ctor-synthetic: malformed counts: {rejected} rejected, {accepted} accepted (counts equal to ll's own after the change)");
    mark(&s, "malformed counts and sizes", from);

    // 13. the canonical ordering on its own
    let from = s.cases;
    canon_tests(&mut rng, quick, &mut s.cases);
    mark(&s, "RAW canonical ordering, independently", from);

    // coverage. A run's fragment is its prefix (j, 30 − j), its suffix (0, r), or the whole
    // run inside one group, which (a run being whole 16-row chunks) is (j, 16) with
    // j ≤ 13: 71 reachable (start slot, length) shapes. The one-row path is counted apart.
    let st = &s.st;
    let reachable: Vec<(usize, usize)> = (0..30)
        .flat_map(|j| (1..=ROWS - j).map(move |k| (j, k)))
        .filter(|&(j, k)| j == 0 && k < ROWS || j > 0 && (j + k == ROWS || k == 16))
        .collect();
    let cov = |m: &[[u64; 31]; 30]| -> (usize, usize, Vec<(usize, usize)>) {
        let all = (0..30)
            .map(|j| (1..=ROWS - j).filter(|&k| m[j][k] > 0).count())
            .sum();
        let missing: Vec<(usize, usize)> = reachable
            .iter()
            .copied()
            .filter(|&(j, k)| m[j][k] == 0)
            .collect();
        (all, reachable.len() - missing.len(), missing)
    };
    let (fa, fr, fm) = cov(&st.cov_frag);
    let (ra, rr, rm) = cov(&st.cov_frag_raw);
    println!(
        "ctor-synthetic coverage: complete-group source phases {}/30; run fragments (start slot, length), ordinary {fr}/{} reachable ({fa} shapes seen), RAW {rr}/{} reachable ({ra} seen); one-row slots ordinary {}/30, RAW {}/30; promotion prefix lengths through a fragment {}/29, through one row {}/29; partway promotions {}",
        st.cov_phase.iter().filter(|&&x| x > 0).count(),
        reachable.len(),
        reachable.len(),
        st.cov_scalar.iter().filter(|&&x| x > 0).count(),
        st.cov_scalar_raw.iter().filter(|&&x| x > 0).count(),
        st.cov_promo_frag[1..].iter().filter(|&&x| x > 0).count(),
        st.cov_promo_one[1..].iter().filter(|&&x| x > 0).count(),
        st.promotions_partway
    );
    if !fm.is_empty() || !rm.is_empty() {
        println!("ctor-synthetic: reachable fragments not covered: ordinary {fm:?}, RAW {rm:?}");
    }
    println!(
        "ctor-synthetic: {} cases, {} array comparisons, {} walk outputs, {} restorations: every check exact ({})",
        s.cases,
        s.ck.array_checks,
        s.ck.walk_checks,
        s.ck.restoration_checks,
        if quick { "quick" } else { "full" }
    );
    println!("ctor-synthetic counters: {{{}}}", st.json());
}

/// The canonical ordering, tested apart from the constructor: records whose RAW groups
/// hold provisional indices in a chosen permutation, each group's entries naming its
/// group; afterwards the headers must hold ranks in group order, each RAW group must hold
/// its group's entries, words 30 and 31 zero, and the ordinary records untouched.
fn canon_tests(rng: &mut Rng, quick: bool, cases: &mut u64) {
    let shapes: &[&str] = &[
        "identity",
        "reverse",
        "rotate",
        "pairs",
        "random",
        "one-cycle",
    ];
    let sizes: &[(usize, usize)] = if quick {
        &[(1, 1), (5, 3), (70, 9)]
    } else {
        &[
            (1, 0),
            (1, 1),
            (2, 2),
            (64, 64),
            (65, 7),
            (130, 40),
            (1000, 300),
            (30_000, 1200),
        ]
    };
    let (mut moved_total, mut cycles_total) = (0u64, 0u64);
    for &(g_count, r) in sizes {
        for shape in shapes {
            // the RAW groups: r distinct groups
            let mut gs: Vec<usize> = (0..g_count).collect();
            for k in (1..gs.len()).rev() {
                gs.swap(k, rng.below(k + 1));
            }
            let mut gs: Vec<usize> = gs[..r].to_vec();
            gs.sort_unstable();
            // perm[rank] = provisional index
            let mut perm: Vec<usize> = (0..r).collect();
            match *shape {
                "identity" => {}
                "reverse" => perm.reverse(),
                "rotate" => perm.rotate_left(r.min(1)),
                "pairs" => {
                    for k in (0..r.saturating_sub(1)).step_by(2) {
                        perm.swap(k, k + 1);
                    }
                }
                "random" => {
                    for k in (1..r).rev() {
                        perm.swap(k, rng.below(k + 1));
                    }
                }
                _ => {
                    // one cycle through every index (Sattolo)
                    for k in (1..r).rev() {
                        perm.swap(k, rng.below(k));
                    }
                }
            }
            let mut rec: Vec<Rec> = (0..g_count + 1)
                .map(|g| Rec {
                    f: [g as u16; 30],
                    h: (g as u32) << 8 | 0x5a,
                })
                .collect();
            let mut bitmap = vec![0u64; g_count.div_ceil(64)];
            let mut raw = vec![RawG([0; 32]); r];
            for (rank, &g) in gs.iter().enumerate() {
                let prov = perm[rank];
                rec[g] = Rec {
                    f: [RAWF; 30],
                    h: RAW_BIT | prov as u32,
                };
                bitmap[g >> 6] |= 1 << (g & 63);
                for (j, e) in raw[prov].0[..30].iter_mut().enumerate() {
                    *e = (g as u32) << 8 | j as u32;
                }
            }
            let fixed = (0..r).filter(|&k| perm[k] == k).count() as u64;
            // SAFETY: rec is initialised; one provisional index per set bit; word 31 zero
            let (moved, cycles, _) = unsafe { canonicalise(rec.as_mut_ptr(), &bitmap, &mut raw) };
            assert_eq!(moved, r as u64 - fixed, "{shape} {g_count}/{r}: moved");
            moved_total += moved;
            cycles_total += cycles;
            let mut rank = 0;
            for (g, rc) in rec.iter().enumerate() {
                if g < g_count && bitmap[g >> 6] >> (g & 63) & 1 != 0 {
                    assert_eq!(rc.h, RAW_BIT | rank as u32, "{shape}: header of {g}");
                    assert!(rc.f == [RAWF; 30]);
                    let e = &raw[rank].0;
                    for j in 0..30 {
                        assert_eq!(
                            e[j],
                            (g as u32) << 8 | j as u32,
                            "{shape}: group {g} entry {j}"
                        );
                    }
                    assert_eq!((e[30], e[31]), (0, 0), "{shape}: padding of rank {rank}");
                    rank += 1;
                } else {
                    assert!(
                        *rc == Rec {
                            f: [g as u16; 30],
                            h: (g as u32) << 8 | 0x5a
                        },
                        "{shape}: ordinary record {g} changed"
                    );
                }
            }
            assert_eq!(rank, r);
            *cases += 1;
        }
    }
    println!("ctor-synthetic: canonical ordering: {moved_total} groups moved in {cycles_total} cycles, all in exact order, every padding word zero");
}

/// `ctor-check`, `ctor-adversarial`, `ctor-synthetic`
pub fn main(mode: &str, args: &[String]) {
    match mode {
        "synthetic" => synthetic(args.iter().any(|a| a == "--quick")),
        "check" | "adversarial" => {
            let mut jl = std::fs::File::create(&args[0]).unwrap();
            let mut nat: Vec<(String, Stats)> = Vec::new();
            let mut forced_rows: Vec<(String, Stats)> = Vec::new();
            let mut run = |name: String, data: &[u8], jl: &mut std::fs::File| {
                for (pm, extra, st) in check_input(&name, data, jl) {
                    if pm == 0 && !extra {
                        nat.push((name.clone(), st));
                    } else if pm == 1000 {
                        forced_rows.push((name.clone(), st));
                    }
                }
            };
            if mode == "check" {
                for file in &args[1..] {
                    let data = std::fs::read(file).unwrap();
                    let name = std::path::Path::new(file)
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();
                    run(name, &data, &mut jl);
                }
            } else {
                for (name, data) in adversarial() {
                    for level in [1, 9] {
                        let z = compress(&data, level);
                        run(format!("{name} -{level}"), &z, &mut jl);
                    }
                }
            }
            println!("\nthe natural configuration (permille 0):");
            table(&nat);
            let mut t = Stats::default();
            for (_, s) in &nat {
                t.add(s);
            }
            println!(
                "\nrun-length histogram, all inputs (natural): bins (upper bound) {:?}",
                &BINS[..10]
            );
            println!("runs {:?}", t.run_hist);
            println!("rows {:?}", t.run_hist_rows);
            println!("complete-record rows {:?}", t.run_hist_complete);
            println!("\nthe forced configuration (permille 1000):");
            table(&forced_rows);
            println!("\ntotals (natural): {{{}}}", t.json());
        }
        m => panic!("unknown mode ctor-{m}"),
    }
}
