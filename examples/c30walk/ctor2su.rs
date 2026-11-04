//! The U1/U2 trial of the compact30 research charter (§6.1, row 1; the scanner journal's §7,
//! §9 and §10): prototype 1 (`ctor2s.rs` at `7f74e9f`, copied) with one change. A
//! nonuniform full tile goes through `tile16`: its first segment is a do-while over the
//! 16-byte window (C4, safe), and its second is taken with `get_unchecked` (U2), under the
//! full-tile contract i + 16 ≤ end ≤ `ll.len()` and n1 = min(16, 30 − p) ≤ 16. U1 is
//! authorised but, after repair 1, not used. No `assert_unchecked` (U3 is not authorised)
//! and no other unchecked access. The tail keeps v2's checked `tile`. Untimed, research only, not wired
//! into the decoder; `ctor2s.rs` and `ctor2.rs` are unchanged.
//!
//!     cargo run --release --example c30walk -- ctor2su-check OUT.jsonl FILE.bz2 [FILE.bz2 ...]
//!     cargo run --release --example c30walk -- ctor2su-adversarial OUT.jsonl
//!     cargo run --release --example c30walk -- ctor2su-synthetic [--quick | --miri CLASS]
//!     cargo run --release --example c30walk -- ctor2su-count [--only v2|su] FILE.bz2 ...
//!
//! What follows is prototype 1's description.
//!
//! **The change (everything else is v2's):** each epoch's unread source is a shrinking
//! slice, `rest`. A full tile is taken with `split_first_chunk::<16>()`, which is the loop's
//! own bounds test; its two qwords are read from the 16-byte array at constant offsets; a
//! run's lookahead does the same on the remainder; a held tile stays at the front of
//! `rest`. A nonuniform full tile, and the tail, go through v2's own `tile` at their row
//! index `end − rest.len()`, its checked slicing unchanged (attempt 3 of three; see the
//! prototype's journal). No new `unsafe` operation: every source access is a safe slice or
//! array access.
//!
//! What follows is v2's description.
//!
//! **What changes from v1:** incomplete records no longer form offsets row by row.
//! - **The append:** every row that the bulk emitter doesn't place (fragment and scalar
//!   rows) writes the low 16 bits of its successor label, `u = φ(i) mod 65536`, straight
//!   into its final field, through a per-character write descriptor: next-field pointer,
//!   portion end, portion start rank, bucket limit, the portion's group and RAW index. A
//!   portion is a character's slots within one group: a whole group, or the character's
//!   share of a crossing group. The append makes no header load, RAW test, offset or span
//!   test, and keeps no encoded destination cursor.
//! - **Source labels** come per 16-byte tile: within a tile, `u` is the tile's base plus
//!   the row's offset, plus 2 after the tile's one possible label hole.
//! - **Epochs:** φ(i) = 65536⌊i/61440⌋ + φ(i mod 61440). At the start of each 61,440-row
//!   source epoch e, the next destination rank of every character is saved:
//!   A_e[c] = C[c] + #{i < 61440e : L[i] = c}. A received rank r of bucket c then has
//!   epoch h(r) = max{e : A_e[c] ≤ r}, the greatest such e, which handles a character
//!   absent for several epochs (repeated snapshots), and its full label is
//!   y = 65536·h(r) + u. Runs are split at epoch boundaries. Temporary labels lie in
//!   0..=65533 (φ(61439) mod 65536 = 65533), so they never equal 0xFFFE or 0xFFFF.
//! - **Finishing:** each portion is finished once, when its character's next occurrence
//!   finds it full, or at the end of input.
//!   - **Fast case:** the latest snapshot ≤ the portion's first rank, so every row is in
//!     the current epoch.
//!   - **An ordinary group's** first and last full labels decide it. If the span is
//!     ≤ 65,533, the header is (y_first << 8) | c and each field becomes (u_j − u_0) mod
//!     65536: all 60 field bytes are read into a register-held copy before any is written,
//!     and nothing at or past byte 60 (the header) is touched; a partial last group reads
//!     only its own fields.
//!   - **A wide group** (span > 65,533, from full labels) becomes RAW at completion: a
//!     zeroed `RawG` is appended, its entries reconstructed epoch interval by epoch
//!     interval, and the record replaced.
//!   - **A crossing (or forced) group's portion** is written into its preallocated
//!     `RawG`, and only that portion's fields are restored to 0xFFFF.
//! - **Kept from v1:** the validation, the RAW bound and allocation policy, the crossing
//!   and forced pre-marking, production's 16-byte uniformity test with run coalescing, the
//!   complete-record emitter (unchanged records, never normalised afterwards), the
//!   canonical RAW ordering (`ctor::canonicalise`, called as it is), the last group's
//!   padding and PARK. Records holding temporary labels are never published: the length
//!   is set only after every portion is finished.
//!
//! **The workspace:** 16 epochs × 256 × u32 of snapshots (16 KiB), 256 descriptors, the
//! bitmap and canonicalise's one carry. There is no successor table, F array or second
//! scan of L.

use super::ctor::canonicalise;
use super::{
    adversarial, blocks, compress, mix, phi, Counts, RawG, Rec, C30, CONFIGS, M20, MAX_OFF, RAWF,
    RAW_BIT, ROWS,
};
use mibz2::bwt::{InverseBwt, Segmented};
use std::io::Write;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The largest block whose PARK label, 32·⌈n/30⌉, fits 20 bits (v1's limit)
const MAX_N: usize = ROWS * (M20 as usize / 32);
/// Source rows per epoch: 30 × 2048, so φ advances by exactly 65,536 per epoch
const EPOCH: usize = 61_440;
const MAX_EPOCHS: usize = MAX_N.div_ceil(EPOCH);
const NO_RAW: u32 = u32::MAX;

const _: () = {
    assert!(MAX_EPOCHS == 16);
    assert!(EPOCH % 16 == 0 && EPOCH % ROWS == 0);
};

/// The thirty complete-record patterns (copied from v1): field j of pattern p is
/// φ(i + j) − φ(i) for a source row i with i mod 30 = p
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

/// The oracle's forced-RAW predicate, exactly as `C30::build` (and v1) evaluates it
fn forced(g: usize, n: usize, permille: u32) -> bool {
    permille > 0 && (mix(g as u64 ^ (n as u64) << 32) % 1000) < permille as u64
}

/// A full tile's two qwords, read from its 16-byte array at constant offsets
#[inline(always)]
fn halves(t: &[u8; 16]) -> (u64, u64) {
    let (lo, hi) = t.split_at(8);
    (
        u64::from_ne_bytes(lo.try_into().unwrap()),
        u64::from_ne_bytes(hi.try_into().unwrap()),
    )
}

/// The untimed structural counters (`build::<true>` only)
#[derive(Clone, Default, Debug)]
pub struct Stats2su {
    pub blocks: u64,
    pub rows: u64,
    /// rows emitted as complete records by the bulk emitter (ordinary), and as RAW entries
    /// of forced complete groups
    pub rows_bulk: u64,
    pub rows_bulk_raw: u64,
    /// rows appended as temporary labels: from runs' fragments, and one row at a time
    pub rows_fragment: u64,
    pub rows_scalar: u64,
    pub runs: u64,
    pub run_rows: u64,
    pub complete_groups: u64,
    /// portions finished, by class
    pub fin_fast: u64,
    pub fin_ordinary: u64,
    pub fin_wide: u64,
    pub fin_crossing: u64,
    pub fin_forced: u64,
    /// ordinary normalisation: full groups (one 60-byte vector pass each), partial groups,
    /// and the fields normalised
    pub norm_full: u64,
    pub norm_partial: u64,
    pub norm_fields: u64,
    /// RAW finishing: fields read, entries written, fields restored to 0xFFFF
    pub raw_fields_read: u64,
    pub raw_entries_written: u64,
    pub fields_restored: u64,
    /// snapshot lookups made for a non-fast portion (epoch searches)
    pub epoch_searches: u64,
    pub epochs: u64,
    pub runs_split_at_epoch: u64,
    pub raw_groups: u64,
    pub raw_moved: u64,
    pub raw_cycles: u64,
    /// coverage: the portions' first slot and length (ordinary and RAW), wide spans seen
    pub cov_portion: [[u64; 31]; 30],
    pub max_ordinary_span: u64,
    pub min_wide_span: u64,
    pub multi_epoch_portions: u64,
    /// the prototype's scan paths (counted build only): fresh qword pairs (A), lookahead
    /// pairs (J), held pairs reused (H), runs ending with no full tile left (Q), nonuniform
    /// full tiles (S), of which with a nonempty second segment (X), tails and tail rows
    pub fresh_pairs: u64,
    pub lookahead_pairs: u64,
    pub held_reuses: u64,
    pub runs_no_full_tile: u64,
    pub tiles_nonuniform: u64,
    pub tiles_second_segment: u64,
    pub tails: u64,
    pub tail_rows: u64,
    /// the trial's unchecked tiles: `tile16` calls (U2 once per call)
    pub unchecked_tiles: u64,
    /// S62-F01's changed-path counters (counted build only; the scanner journal's §11):
    /// refills from `tile16`'s first and second segments, and from the tail's;
    /// the tail's second-segment rows and tails with one; tails of length 1, ≥ 2 and even
    /// (its advance loop, unrolled by two); `fill` calls with m ≥ 8 (its vector loop);
    /// `complete` calls (the run's whole-group branch); runs with a lookahead tile, with
    /// none, and those whose lookahead exhausts the epoch; portions finished by the final
    /// close; crossing groups preset RAW; epochs with a full tile, without one, and
    /// without a tail
    pub tile_refills_first: u64,
    pub tile_refills_second: u64,
    pub tail_refills_first: u64,
    pub tail_refills_second: u64,
    pub tail_rows_second: u64,
    pub tails_second_segment: u64,
    pub tails_len1: u64,
    pub tails_len_ge2: u64,
    pub tails_even: u64,
    pub fills_vector: u64,
    pub complete_calls: u64,
    pub runs_lookahead: u64,
    pub runs_no_lookahead: u64,
    pub runs_exhausted: u64,
    pub final_finishes: u64,
    pub preset_crossing: u64,
    pub epochs_tiled: u64,
    pub epochs_short: u64,
    pub epochs_untailed: u64,
}

impl Stats2su {
    pub fn add(&mut self, o: &Stats2su) {
        macro_rules! sum {
            ($($f:ident),*) => { $( self.$f += o.$f; )* };
        }
        sum!(
            blocks,
            rows,
            rows_bulk,
            rows_bulk_raw,
            rows_fragment,
            rows_scalar,
            runs,
            run_rows,
            complete_groups,
            fin_fast,
            fin_ordinary,
            fin_wide,
            fin_crossing,
            fin_forced,
            norm_full,
            norm_partial,
            norm_fields,
            raw_fields_read,
            raw_entries_written,
            fields_restored,
            epoch_searches,
            epochs,
            runs_split_at_epoch,
            raw_groups,
            raw_moved,
            raw_cycles,
            multi_epoch_portions,
            fresh_pairs,
            lookahead_pairs,
            held_reuses,
            runs_no_full_tile,
            tiles_nonuniform,
            tiles_second_segment,
            tails,
            tail_rows,
            unchecked_tiles,
            tile_refills_first,
            tile_refills_second,
            tail_refills_first,
            tail_refills_second,
            tail_rows_second,
            tails_second_segment,
            tails_len1,
            tails_len_ge2,
            tails_even,
            fills_vector,
            complete_calls,
            runs_lookahead,
            runs_no_lookahead,
            runs_exhausted,
            final_finishes,
            preset_crossing,
            epochs_tiled,
            epochs_short,
            epochs_untailed
        );
        self.max_ordinary_span = self.max_ordinary_span.max(o.max_ordinary_span);
        self.min_wide_span = match (self.min_wide_span, o.min_wide_span) {
            (0, x) | (x, 0) => x,
            (a, b) => a.min(b),
        };
        for (a, b) in self.cov_portion.iter_mut().zip(o.cov_portion.iter()) {
            for (x, y) in a.iter_mut().zip(b) {
                *x += y;
            }
        }
    }

    fn json(&self) -> String {
        format!(
            "\"blocks\":{},\"rows\":{},\"rows_bulk\":{},\"rows_bulk_raw\":{},\"rows_fragment\":{},\"rows_scalar\":{},\"runs\":{},\"run_rows\":{},\"complete_groups\":{},\"fin_fast\":{},\"fin_ordinary\":{},\"fin_wide\":{},\"fin_crossing\":{},\"fin_forced\":{},\"norm_full\":{},\"norm_partial\":{},\"norm_fields\":{},\"raw_fields_read\":{},\"raw_entries_written\":{},\"fields_restored\":{},\"epoch_searches\":{},\"epochs\":{},\"runs_split_at_epoch\":{},\"raw_groups\":{},\"raw_moved\":{},\"raw_cycles\":{},\"multi_epoch_portions\":{},\"max_ordinary_span\":{},\"min_wide_span\":{},\"fresh_pairs\":{},\"lookahead_pairs\":{},\"held_reuses\":{},\"runs_no_full_tile\":{},\"tiles_nonuniform\":{},\"tiles_second_segment\":{},\"tails\":{},\"tail_rows\":{},\"unchecked_tiles\":{},\"tile_refills_first\":{},\"tile_refills_second\":{},\"tail_refills_first\":{},\"tail_refills_second\":{},\"tail_rows_second\":{},\"tails_second_segment\":{},\"tails_len1\":{},\"tails_len_ge2\":{},\"tails_even\":{},\"fills_vector\":{},\"complete_calls\":{},\"runs_lookahead\":{},\"runs_no_lookahead\":{},\"runs_exhausted\":{},\"final_finishes\":{},\"preset_crossing\":{},\"epochs_tiled\":{},\"epochs_short\":{},\"epochs_untailed\":{}",
            self.blocks,
            self.rows,
            self.rows_bulk,
            self.rows_bulk_raw,
            self.rows_fragment,
            self.rows_scalar,
            self.runs,
            self.run_rows,
            self.complete_groups,
            self.fin_fast,
            self.fin_ordinary,
            self.fin_wide,
            self.fin_crossing,
            self.fin_forced,
            self.norm_full,
            self.norm_partial,
            self.norm_fields,
            self.raw_fields_read,
            self.raw_entries_written,
            self.fields_restored,
            self.epoch_searches,
            self.epochs,
            self.runs_split_at_epoch,
            self.raw_groups,
            self.raw_moved,
            self.raw_cycles,
            self.multi_epoch_portions,
            self.max_ordinary_span,
            self.min_wide_span,
            self.fresh_pairs,
            self.lookahead_pairs,
            self.held_reuses,
            self.runs_no_full_tile,
            self.tiles_nonuniform,
            self.tiles_second_segment,
            self.tails,
            self.tail_rows,
            self.unchecked_tiles,
            self.tile_refills_first,
            self.tile_refills_second,
            self.tail_refills_first,
            self.tail_refills_second,
            self.tail_rows_second,
            self.tails_second_segment,
            self.tails_len1,
            self.tails_len_ge2,
            self.tails_even,
            self.fills_vector,
            self.complete_calls,
            self.runs_lookahead,
            self.runs_no_lookahead,
            self.runs_exhausted,
            self.final_finishes,
            self.preset_crossing,
            self.epochs_tiled,
            self.epochs_short,
            self.epochs_untailed
        )
    }
}

/// A character's write descriptor. Empty (no open portion) when `w0 == write == end`
/// (null); otherwise its open portion is the fields `[w0, end)` of group `g`, starting at
/// destination rank `r0`, of which `[w0, write)` hold temporary labels. `raw` is the
/// group's provisional RAW index if it is preset RAW (crossing or forced), else `NO_RAW`.
#[derive(Clone, Copy)]
struct Desc {
    write: *mut u16,
    end: *mut u16,
    w0: *mut u16,
    r0: u32,
    lim: u32,
    g: u32,
    raw: u32,
}

impl Desc {
    /// The character's next destination rank: the portion's start plus its rows
    #[inline(always)]
    fn rank(&self) -> u32 {
        self.r0 + ((self.write as usize - self.w0 as usize) / 2) as u32
    }
}

/// The constructor's reusable scratch: the RAW bitmap, the epoch snapshots, the counters
#[derive(Default)]
pub struct Ctor2su {
    bitmap: Vec<u64>,
    snaps: Vec<[u32; 256]>,
    pub st: Stats2su,
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
    n: usize,
    raw: &'a mut Vec<RawG>,
    bitmap: &'a mut [u64],
    snaps: &'a mut Vec<[u32; 256]>,
    desc: [Desc; 256],
    /// the source row's full label φ(i) and phase i mod 30
    y: u32,
    p: usize,
    st: &'a mut Stats2su,
}

impl<const COUNT: bool> W<'_, COUNT> {
    /// Whether `one(c, _)` takes its refill detour (the open portion is full, or there is
    /// none): read by the counted build's counters only
    #[inline(always)]
    fn refills(&self, c: u8) -> bool {
        self.desc[c as usize].write == self.desc[c as usize].end
    }

    #[inline(always)]
    fn is_raw(&self, g: usize) -> bool {
        self.bitmap[g >> 6] >> (g & 63) & 1 != 0
    }

    /// The field of destination rank r. SAFETY: r < n ≤ 30·g_count
    #[inline(always)]
    unsafe fn field(&self, r: u32) -> *mut u16 {
        let (g, j) = (r as usize / ROWS, r as usize % ROWS);
        chk!(COUNT, g < self.g_count);
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

    /// Group g made RAW (all 64 bytes of its record written) with a zeroed `RawG`
    unsafe fn create_raw(&mut self, g: usize) -> u32 {
        chk!(COUNT, g < self.g_count && !self.is_raw(g));
        let idx = self.raw.len() as u32;
        self.raw.push(RawG([0; 32]));
        self.dst.add(g).write(Rec {
            f: [RAWF; 30],
            h: RAW_BIT | idx,
        });
        self.bitmap[g >> 6] |= 1 << (g & 63);
        idx
    }

    /// Save every character's next destination rank: the snapshot A_e
    fn snapshot(&mut self) {
        let mut a = [0u32; 256];
        for (x, d) in a.iter_mut().zip(self.desc.iter()) {
            *x = d.rank();
        }
        self.snaps.push(a);
        if COUNT {
            self.st.epochs += 1;
        }
    }

    /// The epoch of received rank r of bucket c: the greatest e with A_e[c] ≤ r
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
        let lim = self.desc[c].lim;
        chk!(COUNT, r < lim);
        let g = r as usize / ROWS;
        let re = lim.min(((g + 1) * ROWS) as u32);
        let w0 = self.field(r);
        let raw = if self.is_raw(g) {
            ptr::addr_of!((*self.dst.add(g)).h).read() & M20
        } else {
            // an ordinary candidate covers its whole group: a non-crossing group's slots
            // all belong to one bucket, whose earlier portions end at group boundaries
            chk!(COUNT, r as usize % ROWS == 0);
            NO_RAW
        };
        let d = &mut self.desc[c];
        d.w0 = w0;
        d.write = w0;
        d.end = w0.add((re - r) as usize);
        d.r0 = r;
        d.g = g as u32;
        d.raw = raw;
    }

    /// Finish c's pending portion (if any) and leave the descriptor empty at its next rank
    #[inline(always)]
    unsafe fn close(&mut self, c: usize) {
        let d = self.desc[c];
        if d.write != d.w0 {
            self.finish(c);
        }
        let d = &mut self.desc[c];
        d.r0 = d.rank();
        d.w0 = ptr::null_mut();
        d.write = ptr::null_mut();
        d.end = ptr::null_mut();
    }

    /// The one-row path's slow edge: the open portion is full (or there is none). Finish
    /// it, and open the next, or reject an exhausted bucket before any write.
    #[cold]
    #[inline(never)]
    unsafe fn refill(&mut self, c: usize) -> bool {
        self.close(c);
        let r = self.desc[c].r0;
        if r >= self.desc[c].lim {
            return false;
        }
        self.open(c, r);
        true
    }

    /// Finish c's full pending portion: classify it from its full labels, then write its
    /// final form (once)
    #[inline(never)]
    unsafe fn finish(&mut self, c: usize) {
        let d = self.desc[c];
        let k = (d.write as usize - d.w0 as usize) / 2;
        chk!(COUNT, k >= 1 && d.write == d.end);
        let (r0, r1) = (d.r0, d.r0 + k as u32 - 1);
        let g = d.g as usize;
        let j0 = r0 as usize - ROWS * g;
        let last = self.snaps.len() - 1;
        let fast = self.snaps[last][c] <= r0;
        let (h0, h1) = if fast {
            (last, last)
        } else {
            if COUNT {
                self.st.epoch_searches += 1;
            }
            (self.epoch_of(c, r0), self.epoch_of(c, r1))
        };
        if COUNT {
            self.st.cov_portion[j0][k] += 1;
            self.st.multi_epoch_portions += (h0 != h1) as u64;
        }
        if d.raw == NO_RAW {
            // an ordinary candidate: its whole group, j0 = 0
            let u0 = d.w0.read();
            let ul = d.w0.add(k - 1).read();
            let y0 = 65536 * h0 as u32 + u0 as u32;
            let yl = 65536 * h1 as u32 + ul as u32;
            // classified from the full labels, before any modular subtraction
            let span = yl - y0;
            if span <= MAX_OFF {
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
                if COUNT {
                    if fast {
                        self.st.fin_fast += 1;
                    } else {
                        self.st.fin_ordinary += 1;
                    }
                    self.st.norm_full += (k == ROWS) as u64;
                    self.st.norm_partial += (k < ROWS) as u64;
                    self.st.norm_fields += k as u64;
                    self.st.max_ordinary_span = self.st.max_ordinary_span.max(span as u64);
                }
            } else {
                // wide: RAW at completion, the entries reconstructed, the record replaced
                let idx = self.raw.len();
                self.raw.push(RawG([0; 32]));
                self.write_entries(c, idx, d.w0, j0, k, r0, h0, h1);
                self.dst.add(g).write(Rec {
                    f: [RAWF; 30],
                    h: RAW_BIT | idx as u32,
                });
                self.bitmap[g >> 6] |= 1 << (g & 63);
                if COUNT {
                    self.st.fin_wide += 1;
                    let w = span as u64;
                    self.st.min_wide_span = if self.st.min_wide_span == 0 {
                        w
                    } else {
                        self.st.min_wide_span.min(w)
                    };
                }
            }
        } else {
            // a preset RAW group's portion: into its RAW block, then only this portion's
            // fields restored to 0xFFFF (other characters own the other slots)
            self.write_entries(c, d.raw as usize, d.w0, j0, k, r0, h0, h1);
            for t in 0..k {
                d.w0.add(t).write(RAWF);
            }
            if COUNT {
                self.st.fields_restored += k as u64;
                // a crossing portion doesn't cover its whole group; a forced group's does
                if k < ROWS.min(self.n - ROWS * g) {
                    self.st.fin_crossing += 1;
                } else {
                    self.st.fin_forced += 1;
                }
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
        if COUNT {
            self.st.raw_fields_read += k as u64;
            self.st.raw_entries_written += k as u64;
        }
    }

    /// The one-row append: the temporary label into the character's next field
    #[inline(always)]
    unsafe fn one(&mut self, c: u8, u: u16) -> bool {
        let ci = c as usize;
        if self.desc[ci].write == self.desc[ci].end && !self.refill(ci) {
            return false;
        }
        let d = &mut self.desc[ci];
        d.write.write(u);
        d.write = d.write.add(1);
        if COUNT {
            self.st.rows_scalar += 1;
        }
        true
    }

    /// v2's tile, unchanged, for the tail only (the trial's full nonuniform tiles take
    /// `tile16`)
    #[inline(always)]
    unsafe fn tile(&mut self, ll: &[u8], i: usize, len: usize) -> bool {
        // the checked path is the tail's only: a tail never takes U1 or U2
        chk!(COUNT, len < 16);
        if COUNT {
            if len == 16 {
                self.st.tiles_nonuniform += 1;
                self.st.tiles_second_segment += (ROWS - self.p < 16) as u64;
            } else {
                self.st.tails += 1;
                self.st.tail_rows += len as u64;
                self.st.tails_len1 += (len == 1) as u64;
                self.st.tails_len_ge2 += (len >= 2) as u64;
                self.st.tails_even += (len % 2 == 0) as u64;
                self.st.tails_second_segment += (len > len.min(ROWS - self.p)) as u64;
            }
        }
        let u = self.y as u16;
        let n1 = len.min(ROWS - self.p);
        for (t, &c) in ll[i..i + n1].iter().enumerate() {
            if COUNT {
                self.st.tail_refills_first += self.refills(c) as u64;
            }
            if !self.one(c, u.wrapping_add(t as u16)) {
                return false;
            }
        }
        for (t, &c) in ll[i + n1..i + len].iter().enumerate() {
            if COUNT {
                self.st.tail_refills_second += self.refills(c) as u64;
                self.st.tail_rows_second += 1;
            }
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

    /// A nonuniform full tile (the trial): `t` is the 16-byte chunk that
    /// `split_first_chunk` split off `rest`, at row index i
    ///
    /// The contract (the scanner journal's §9.2): t = `ll[i..i + 16]`, so
    /// i + 16 ≤ end ≤ `ll.len()`; and n1 = min(16, 30 − p) ≤ 16 for any p. Then
    /// U2 = [i + n1, i + 16) is well-ordered and inside `ll`. (U1, [i, i + n1), is
    /// authorised but not used: the do-while reads the first segment from `t`.)
    #[inline(always)]
    unsafe fn tile16(&mut self, ll: &[u8], i: usize, t: &[u8; 16]) -> bool {
        // the dominating full-tile contract, checked in the counted build and under
        // debug assertions (Miri): the tile is the chunk split off `rest`
        chk!(COUNT, ptr::eq(t.as_ptr(), ll.as_ptr().wrapping_add(i)));
        chk!(COUNT, self.p < ROWS);
        if COUNT {
            self.st.tiles_nonuniform += 1;
            self.st.tiles_second_segment += (ROWS - self.p < 16) as u64;
            self.st.unchecked_tiles += 1;
        }
        let u = self.y as u16;
        let n1 = 16usize.min(ROWS.wrapping_sub(self.p));
        // SAFETY (U2): i + n1 ≤ i + 16 ≤ ll.len() by the contract above
        let s2 = unsafe { ll.get_unchecked(i + n1..i + 16) };
        // C4, safe: the first segment as a do-while over the 16-byte window, t[k] with
        // k < n1 ≤ 16 (repair 1: U1 is not used; the scanner journal's §10). p < 30, so
        // n1 ≥ 1, matters for correctness only: were n1 0, this would append one wrong
        // row, reading in bounds.
        let mut k = 0usize;
        loop {
            if COUNT {
                self.st.tile_refills_first += self.refills(t[k]) as u64;
            }
            if !self.one(t[k], u.wrapping_add(k as u16)) {
                return false;
            }
            k += 1;
            if k >= n1 {
                break;
            }
        }
        for (t2, &c) in s2.iter().enumerate() {
            if COUNT {
                self.st.tile_refills_second += self.refills(c) as u64;
            }
            if !self.one(c, u.wrapping_add((n1 + t2) as u16 + 2)) {
                return false;
            }
        }
        self.advance(16);
        true
    }

    /// Append m (≤ the open portion's room) temporary labels of a run, from the source's
    /// pattern
    #[inline(always)]
    unsafe fn fill(&mut self, c: usize, m: usize) {
        let pat = &PATTERNS[self.p].f;
        let y = self.y;
        let d = &mut self.desc[c];
        chk!(COUNT, m <= (d.end as usize - d.write as usize) / 2);
        for (t, &o) in pat[..m].iter().enumerate() {
            d.write.add(t).write((y + o as u32) as u16);
        }
        d.write = d.write.add(m);
        self.advance(m);
        if COUNT {
            self.st.rows_fragment += m as u64;
            // the uncounted listing's vector loop is entered for m ≥ 8
            self.st.fills_vector += (m >= 8) as u64;
        }
    }

    /// `whole` complete groups of byte c from group g (v1's emitter, copied), or in a
    /// forced configuration each preset group's RAW entries directly
    #[inline(always)]
    unsafe fn complete(&mut self, c: u8, g: usize, whole: usize, forced_cfg: bool) {
        chk!(COUNT, g + whole <= self.g_count);
        if COUNT {
            self.st.complete_calls += 1;
        }
        if !forced_cfg {
            let mut r = PATTERNS[self.p];
            r.h = self.y << 8 | c as u32;
            let d = self.dst.add(g);
            for t in 0..whole {
                d.add(t).write(r);
                r.h += 32 << 8;
            }
            self.y += 32 * whole as u32;
            if COUNT {
                self.st.rows_bulk += 30 * whole as u64;
                self.st.complete_groups += whole as u64;
            }
            return;
        }
        for t in 0..whole {
            let gt = g + t;
            if self.is_raw(gt) {
                let idx = (ptr::addr_of!((*self.dst.add(gt)).h).read() & M20) as usize;
                let pat = &PATTERNS[self.p].f;
                let y = self.y;
                for (e, &o) in self.raw[idx].0[..ROWS].iter_mut().zip(pat) {
                    *e = (y + o as u32) << 8 | c as u32;
                }
                if COUNT {
                    self.st.rows_bulk_raw += 30;
                }
            } else {
                let mut r = PATTERNS[self.p];
                r.h = self.y << 8 | c as u32;
                self.dst.add(gt).write(r);
                if COUNT {
                    self.st.rows_bulk += 30;
                    self.st.complete_groups += 1;
                }
            }
            self.y += 32;
        }
    }

    /// A coalesced run of m rows of byte c (within one epoch): one limit check, then the
    /// open portion's remainder, complete groups, and the suffix in a new portion
    #[inline(always)]
    unsafe fn run(&mut self, c: u8, mut m: usize, forced_cfg: bool) -> bool {
        let ci = c as usize;
        let d = self.desc[ci];
        if d.rank() as usize + m > d.lim as usize {
            return false;
        }
        if COUNT {
            self.st.runs += 1;
            self.st.run_rows += m as u64;
        }
        loop {
            let d = self.desc[ci];
            let room = (d.end as usize - d.write as usize) / 2;
            let k = room.min(m);
            if k > 0 {
                self.fill(ci, k);
                m -= k;
            }
            if m == 0 {
                return true;
            }
            self.close(ci);
            let r = self.desc[ci].r0 as usize;
            if r % ROWS == 0 && m >= ROWS {
                let whole = m / ROWS;
                self.complete(c, r / ROWS, whole, forced_cfg);
                m -= ROWS * whole;
                self.desc[ci].r0 = (r + ROWS * whole) as u32;
                if m == 0 {
                    return true;
                }
            }
            let r = self.desc[ci].r0;
            self.open(ci, r);
        }
    }
}

impl Ctor2su {
    /// `rec` and `raw` for the last column `ll` and its `counts`, exactly as `C30::build`
    /// gives them for the same `permille`. False, with both arrays empty, if the counts
    /// don't sum to n, n is out of range, or a bucket overflows (the counts are not `ll`'s
    /// own), as v1.
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
        let Ctor2su { bitmap, snaps, st } = self;
        bitmap.clear();
        bitmap.resize(g_count.div_ceil(64), 0);
        snaps.clear();
        snaps.reserve(MAX_EPOCHS);
        let mut desc = [Desc {
            write: ptr::null_mut(),
            end: ptr::null_mut(),
            w0: ptr::null_mut(),
            r0: 0,
            lim: 0,
            g: 0,
            raw: NO_RAW,
        }; 256];
        let mut cum = 0u32;
        for (d, &k) in desc.iter_mut().zip(counts.iter()) {
            d.r0 = cum;
            cum += k;
            d.lim = cum;
        }
        let ok = {
            let mut w = W::<COUNT> {
                dst: rec.as_mut_ptr(),
                g_count,
                n,
                raw: &mut *raw,
                bitmap,
                snaps,
                desc,
                y: 0,
                p: 0,
                st: &mut *st,
            };
            // SAFETY: every record index below is under g_count + 1, the arena's capacity
            unsafe { Self::scatter(&mut w, ll, counts, permille) }
        };
        if !ok {
            raw.clear();
            return false;
        }
        // SAFETY: as above; every portion is finished, so no temporary label remains
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
            let (moved, cycles, _) = canonicalise(dst, bitmap, raw);
            if COUNT {
                st.raw_moved += moved;
                st.raw_cycles += cycles;
            }
            rec.set_len(g_count + 1);
        }
        if COUNT {
            st.blocks += 1;
            st.rows += n as u64;
            st.raw_groups += raw.len() as u64;
        }
        true
    }

    /// The preset RAW groups, then the scan of L epoch by epoch, then every pending
    /// portion finished. False at the first overflowing bucket (before its write) or if
    /// any bucket ends short.
    #[inline(always)]
    unsafe fn scatter<const COUNT: bool>(
        w: &mut W<'_, COUNT>,
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
                    if COUNT {
                        w.st.preset_crossing += 1;
                    }
                    w.create_raw(g);
                    last = g;
                }
            }
        }
        let forced_cfg = permille > 0;
        if forced_cfg {
            for g in 0..w.g_count {
                if !w.is_raw(g) && forced(g, n, permille) {
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
            if COUNT {
                w.st.epochs_tiled += (rest.len() >= 16) as u64;
                w.st.epochs_short += (rest.len() < 16) as u64;
            }
            let mut held: Option<(u64, u64)> = None;
            while let Some((t, after)) = rest.split_first_chunk::<16>() {
                // `t` is read only on the fresh path; a nonuniform tile is re-read from `ll`
                let (a, b) = match held.take() {
                    Some(v) => {
                        if COUNT {
                            w.st.held_reuses += 1;
                        }
                        v
                    }
                    None => {
                        if COUNT {
                            w.st.fresh_pairs += 1;
                        }
                        halves(t)
                    }
                };
                if a == b && a == a.rotate_left(8) {
                    if COUNT {
                        w.st.runs_lookahead += (after.len() >= 16) as u64;
                        w.st.runs_no_lookahead += (after.len() < 16) as u64;
                    }
                    let mut r2 = after;
                    while let Some((t2, after2)) = r2.split_first_chunk::<16>() {
                        let (a2, b2) = halves(t2);
                        if COUNT {
                            w.st.lookahead_pairs += 1;
                        }
                        if a2 == a && b2 == a {
                            r2 = after2;
                        } else {
                            held = Some((a2, b2));
                            break;
                        }
                    }
                    if COUNT {
                        w.st.runs_no_full_tile += held.is_none() as u64;
                        w.st.runs_exhausted += (held.is_none() && after.len() >= 16) as u64;
                        if r2.is_empty() && end < n && ll[end] == a as u8 {
                            w.st.runs_split_at_epoch += 1;
                        }
                    }
                    // the run's rows: those its tiles span
                    if !w.run(a as u8, rest.len() - r2.len(), forced_cfg) {
                        return false;
                    }
                    rest = r2;
                } else {
                    // the trial's tile, at the tile's row index in `ll`
                    if !w.tile16(ll, end - rest.len(), t) {
                        return false;
                    }
                    rest = after;
                }
            }
            if COUNT {
                w.st.epochs_untailed += rest.is_empty() as u64;
            }
            if !rest.is_empty() && !w.tile(ll, end - rest.len(), rest.len()) {
                return false;
            }
            i = end;
        }
        for c in 0..256 {
            if w.desc[c].rank() != w.desc[c].lim {
                return false;
            }
        }
        for c in 0..256 {
            if COUNT {
                w.st.final_finishes += (w.desc[c].write != w.desc[c].w0) as u64;
            }
            w.close(c);
        }
        true
    }
}

// ---------------------------------------------------------------------------------------
// The correctness harness
// ---------------------------------------------------------------------------------------

fn bytes<T>(v: &[T]) -> &[u8] {
    // SAFETY: Rec and RawG have no padding
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v)) }
}

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

/// Two poison patterns (alternated), so a missed field can't coincide with both
fn poison(rec: &mut Vec<Rec>, alt: bool) {
    rec.clear();
    let v = if alt { 0x5A5A } else { 0xA5A5 };
    for s in rec.spare_capacity_mut() {
        s.write(Rec {
            f: [v; 30],
            h: (v as u32) << 16 | v as u32,
        });
    }
}

#[derive(Default)]
struct Arenas {
    oracle: C30,
    cand: C30,
    ctor: Ctor2su,
    /// constructor v2 (`ctor2.rs`, unchanged, called read-only), for the counters' comparison
    v2: super::ctor2::Ctor2,
    v2rec: Vec<Rec>,
    v2raw: Vec<RawG>,
    out: Vec<u8>,
    out_o: Vec<u8>,
    prod: Segmented,
    want: Vec<u8>,
    flips: u64,
}

#[derive(Default, Clone, Copy)]
struct Checked {
    arrays: u64,
    walks: u64,
    restorations: u64,
}

impl Checked {
    fn add(&mut self, o: &Checked) {
        self.arrays += o.arrays;
        self.walks += o.walks;
        self.restorations += o.restorations;
    }
}

/// The prototype's counters against v2's counted build on the same input: every v2 field
/// equal (the row classification, run coalescing and splitting, finish classes,
/// normalisation, RAW work, epochs, spans and portion shapes), and the scan paths'
/// identities for a block of n rows: full tiles T = Σ over epochs ⌊rows/16⌋ = ⌊n/16⌋ (an
/// epoch is 3,840 tiles), fresh + lookahead = T, nonuniform S = T − run_rows/16, runs R =
/// held reuses H + runs ending with no full tile Q, fresh = S + Q, tail rows = n − 16T
fn same_as_v2(what: &str, s: &Stats2su, v: &super::ctor2::Stats2, n: usize) {
    macro_rules! eq {
        ($($f:ident),*) => { $( assert_eq!(s.$f, v.$f, "{what}: {} differs from v2's", stringify!($f)); )* };
    }
    eq!(
        blocks,
        rows,
        rows_bulk,
        rows_bulk_raw,
        rows_fragment,
        rows_scalar,
        runs,
        run_rows,
        complete_groups,
        fin_fast,
        fin_ordinary,
        fin_wide,
        fin_crossing,
        fin_forced,
        norm_full,
        norm_partial,
        norm_fields,
        raw_fields_read,
        raw_entries_written,
        fields_restored,
        epoch_searches,
        epochs,
        runs_split_at_epoch,
        raw_groups,
        raw_moved,
        raw_cycles,
        max_ordinary_span,
        min_wide_span,
        multi_epoch_portions
    );
    assert!(
        s.cov_portion == v.cov_portion,
        "{what}: portion shapes differ from v2's"
    );
    let t = (n / 16) as u64;
    assert_eq!(s.fresh_pairs + s.lookahead_pairs, t, "{what}: full tiles");
    assert_eq!(
        s.tiles_nonuniform,
        t - s.run_rows / 16,
        "{what}: nonuniform tiles"
    );
    assert_eq!(s.run_rows % 16, 0, "{what}: run rows");
    assert_eq!(
        s.runs,
        s.held_reuses + s.runs_no_full_tile,
        "{what}: run endings"
    );
    assert_eq!(
        s.fresh_pairs,
        s.tiles_nonuniform + s.runs_no_full_tile,
        "{what}: fresh pairs"
    );
    assert_eq!(s.tail_rows, (n % 16) as u64, "{what}: tail rows");
    assert_eq!(s.tails, (n % 16 > 0) as u64, "{what}: tails");
    assert_eq!(
        s.unchecked_tiles, s.tiles_nonuniform,
        "{what}: the unchecked form runs exactly on the nonuniform full tiles"
    );
    assert_eq!(
        s.rows_scalar,
        16 * s.tiles_nonuniform + s.tail_rows,
        "{what}: scalar rows"
    );
    // S62-F01's counters: their partitions of the existing counters
    assert_eq!(
        s.runs_lookahead + s.runs_no_lookahead,
        s.runs,
        "{what}: run lookahead entry"
    );
    assert_eq!(
        s.runs_exhausted + s.runs_no_lookahead,
        s.runs_no_full_tile,
        "{what}: run endings with no full tile"
    );
    assert_eq!(
        s.epochs_tiled + s.epochs_short,
        s.epochs,
        "{what}: epoch entry"
    );
    assert_eq!(s.epochs_untailed + s.tails, s.epochs, "{what}: epoch tails");
    assert_eq!(
        s.tails_len1 + s.tails_len_ge2,
        s.tails,
        "{what}: tail lengths"
    );
    assert!(s.tails_even <= s.tails_len_ge2, "{what}: even tails");
    assert!(
        s.tail_rows_second <= s.tail_rows,
        "{what}: tail second-segment rows"
    );
    assert!(
        s.tails_second_segment <= s.tails,
        "{what}: tail second segments"
    );
    assert!(
        s.tile_refills_first + s.tile_refills_second + s.tail_refills_first + s.tail_refills_second
            <= s.rows_scalar,
        "{what}: scalar refills"
    );
    assert!(
        s.preset_crossing <= s.raw_groups,
        "{what}: preset crossing groups"
    );
}

/// One block, one configuration: the oracle; Ctor2su's counted build into a poisoned reused
/// arena, against the oracle, and its counters against v2's counted build; its uncounted
/// build, again; then with `want` the unchanged walker on its arrays (counted, audited)
/// against production and the oracle's own counted walk, and the arrays again after each
/// walk (marker restoration)
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
) -> Stats2su {
    let n = ll.len();
    let _ = a.oracle.build(ll, counts, pm);
    a.flips += 1;
    if poisoned {
        poison(&mut a.cand.rec, a.flips % 2 == 0);
    }
    let before = std::mem::take(&mut a.ctor.st);
    assert!(
        a.ctor
            .build::<true>(ll, counts, pm, &mut a.cand.rec, &mut a.cand.raw),
        "{what}: the counted build rejected valid counts"
    );
    let st = std::mem::replace(&mut a.ctor.st, before);
    if let Some(d) = diff(&a.oracle, &a.cand.rec, &a.cand.raw) {
        panic!("{what} pm {pm}: Ctor2su's counted build differs: {d}");
    }
    let before2 = std::mem::take(&mut a.v2.st);
    assert!(a
        .v2
        .build::<true>(ll, counts, pm, &mut a.v2rec, &mut a.v2raw));
    let st2 = std::mem::replace(&mut a.v2.st, before2);
    same_as_v2(&format!("{what} pm {pm}"), &st, &st2, n);
    if poisoned {
        poison(&mut a.cand.rec, a.flips % 2 == 1);
    }
    assert!(
        a.ctor
            .build::<false>(ll, counts, pm, &mut a.cand.rec, &mut a.cand.raw),
        "{what}: the uncounted build rejected valid counts"
    );
    if let Some(d) = diff(&a.oracle, &a.cand.rec, &a.cand.raw) {
        panic!("{what} pm {pm}: Ctor2su's uncounted build differs: {d}");
    }
    ck.arrays += 2;
    if let Some(want) = want {
        a.cand.n = n;
        let (mut co, mut cc) = (Counts::default(), Counts::default());
        a.oracle.walk::<true>(op, extra, &mut a.out_o, &mut co);
        a.cand.walk::<true>(op, extra, &mut a.out, &mut cc);
        assert!(
            a.out == want,
            "{what}: counted walk differs from production"
        );
        assert!(
            a.out == a.out_o,
            "{what}: counted walk differs from the oracle's"
        );
        assert_eq!(format!("{co:?}"), format!("{cc:?}"), "{what}: transitions");
        if let Some(d) = diff(&a.oracle, &a.cand.rec, &a.cand.raw) {
            panic!("{what}: after the counted walk: {d}");
        }
        let mut dummy = Counts::default();
        a.cand.walk::<false>(op, extra, &mut a.out, &mut dummy);
        assert!(
            a.out == want,
            "{what}: audited walk differs from production"
        );
        if let Some(d) = diff(&a.oracle, &a.cand.rec, &a.cand.raw) {
            panic!("{what}: after the audited walk: {d}");
        }
        ck.walks += 3;
        ck.restorations += 2;
    }
    st
}

fn pct(a: u64, b: u64) -> f64 {
    if b == 0 {
        0.0
    } else {
        100.0 * a as f64 / b as f64
    }
}

/// v1's logical per-row field traffic on the same blocks, from v1's own counted build
/// (`ctor::Ctor::build::<true>`, called read-only)
#[derive(Default, Clone, Copy)]
struct V1Traffic {
    rows: u64,
    field_stores: u64,
    entry_stores: u64,
    header_loads: u64,
    field_loads: u64,
}

fn v1_traffic(ll: &[u8], counts: &[u32; 256]) -> V1Traffic {
    let mut c = super::ctor::Ctor::default();
    let (mut rec, mut raw) = (Vec::new(), Vec::new());
    assert!(c.build::<true>(ll, counts, 0, &mut rec, &mut raw));
    let s = &c.st;
    let ord_frag = s.rows_fragment - s.rows_fragment_raw;
    let ord_scalar = s.rows_scalar - s.rows_scalar_raw;
    V1Traffic {
        rows: s.rows,
        field_stores: ord_frag + ord_scalar,
        entry_stores: s.rows_fragment_raw + s.rows_scalar_raw + s.reconstructed_fields,
        // a header load per one-row write at a slot > 0, and per prefix fragment
        header_loads: s.cov_scalar[1..].iter().sum::<u64>()
            + s.cov_scalar_raw[1..].iter().sum::<u64>()
            + s.prefix_fragments,
        field_loads: s.reconstructed_fields,
    }
}

fn check_input(name: &str, data: &[u8], jl: &mut std::fs::File, traffic: bool) -> Stats2su {
    let bs = blocks(data);
    let mut nat = Stats2su::default();
    for &(pm, extra) in &CONFIGS {
        let next = AtomicUsize::new(0);
        let results: Vec<(Stats2su, Checked, V1Traffic)> = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..3)
                .map(|_| {
                    sc.spawn(|| {
                        let mut a = Arenas::default();
                        let (mut st, mut ck, mut v1) = (
                            Stats2su::default(),
                            Checked::default(),
                            V1Traffic::default(),
                        );
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some((ll, counts, op, want)) = bs.get(i) else {
                                break;
                            };
                            let s = check_block(
                                &mut a,
                                &format!("{name} block {i}"),
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
                            if traffic && pm == 0 && !extra {
                                let t = v1_traffic(ll, counts);
                                v1.rows += t.rows;
                                v1.field_stores += t.field_stores;
                                v1.entry_stores += t.entry_stores;
                                v1.header_loads += t.header_loads;
                                v1.field_loads += t.field_loads;
                            }
                        }
                        (st, ck, v1)
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let (mut st, mut ck, mut v1) = (
            Stats2su::default(),
            Checked::default(),
            V1Traffic::default(),
        );
        for (s, c, t) in &results {
            st.add(s);
            ck.add(c);
            v1.rows += t.rows;
            v1.field_stores += t.field_stores;
            v1.entry_stores += t.entry_stores;
            v1.header_loads += t.header_loads;
            v1.field_loads += t.field_loads;
        }
        let v1j = if v1.rows > 0 {
            format!(
                ",\"v1_rows\":{},\"v1_field_stores\":{},\"v1_entry_stores\":{},\"v1_header_loads\":{},\"v1_field_loads\":{}",
                v1.rows, v1.field_stores, v1.entry_stores, v1.header_loads, v1.field_loads
            )
        } else {
            String::new()
        };
        writeln!(
            jl,
            "{{\"input\":{name:?},\"permille\":{pm},\"extra_starts\":{extra},\"array_checks\":{},\"walk_checks\":{},\"restoration_checks\":{},{}{v1j}}}",
            ck.arrays,
            ck.walks,
            ck.restorations,
            st.json()
        )
        .unwrap();
        println!(
            "{name}: permille {pm}, extra {extra}: {} blocks, {} rows: {} array comparisons, {} walks, {} restorations exact; bulk {:.2}%, fragment {:.2}%, scalar {:.2}%; finished fast {}, ordinary {}, wide {}, crossing {}, forced {}; epochs {}",
            st.blocks,
            st.rows,
            ck.arrays,
            ck.walks,
            ck.restorations,
            pct(st.rows_bulk, st.rows),
            pct(st.rows_fragment, st.rows),
            pct(st.rows_scalar, st.rows),
            st.fin_fast,
            st.fin_ordinary,
            st.fin_wide,
            st.fin_crossing,
            st.fin_forced,
            st.epochs
        );
        if pm == 0 && !extra {
            nat = st;
        }
    }
    nat
}

// ---------------------------------------------------------------------------------------
// Synthetic cases
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

struct Synth {
    a: Arenas,
    st: Stats2su,
    ck: Checked,
    cases: u64,
    poisoned: bool,
    classes: Vec<(String, u64)>,
}

impl Synth {
    fn case(&mut self, what: &str, ll: &[u8], pm: u32) -> Stats2su {
        let counts = hist(ll);
        let n = ll.len();
        self.cases += 1;
        if n == 0 {
            // the empty block, through both builds (the oracle needs n ≥ 1): PARK alone
            for counted in [true, false] {
                let ok = if counted {
                    self.a.ctor.build::<true>(
                        ll,
                        &counts,
                        pm,
                        &mut self.a.cand.rec,
                        &mut self.a.cand.raw,
                    )
                } else {
                    self.a.ctor.build::<false>(
                        ll,
                        &counts,
                        pm,
                        &mut self.a.cand.rec,
                        &mut self.a.cand.raw,
                    )
                };
                assert!(ok && self.a.cand.raw.is_empty() && self.a.cand.rec.len() == 1);
                assert!(
                    self.a.cand.rec[0]
                        == Rec {
                            f: [RAWF; 30],
                            h: RAW_BIT | M20
                        }
                );
            }
            return Stats2su::default();
        }
        let op = (n / 2) as u32;
        let mut w = std::mem::take(&mut self.a.want);
        self.a.prod.invert(ll, &counts, op, &mut w);
        let s = check_block(
            &mut self.a,
            what,
            ll,
            &counts,
            op,
            Some(&w),
            pm,
            true,
            self.poisoned,
            &mut self.ck,
        );
        self.a.want = w;
        self.st.add(&s);
        s
    }

    /// The case with `ll` placed at byte offset `off` of a fresh allocation (odd alignment)
    fn case_at(&mut self, what: &str, ll: &[u8], pm: u32, off: usize) -> Stats2su {
        let mut buf = vec![0xEEu8; off];
        buf.extend_from_slice(ll);
        self.case(&format!("{what} offset {off}"), &buf[off..], pm)
    }

    fn mark(&mut self, name: &str, from: u64) {
        println!(
            "ctor2su-synthetic: {name}: {} cases exact",
            self.cases - from
        );
        self.classes.push((name.into(), self.cases - from));
    }
}

/// L with bucket 1 (the smallest byte present) at the given source positions, filler
/// elsewhere, and `a0` rows of byte 0 at the end (shifting byte 1's bucket)
fn placed(n: usize, ones: &[usize], a0: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(n + a0);
    filler(&mut v, n);
    for &s in ones {
        v[s] = 1;
    }
    v.extend(std::iter::repeat(0u8).take(a0));
    v
}

/// A run of `m` rows of byte `b`
fn run_of(v: &mut Vec<u8>, b: u8, m: usize) {
    v.extend(std::iter::repeat(b).take(m));
}

/// The prototype's registered classes (pre-registration §0.4), by name, each runnable on
/// its own (`--miri CLASS`) and all of them in the full and quick suites
const SCANNER_CLASSES: [&str; 9] = [
    "tiny",
    "tails",
    "align",
    "phase",
    "held",
    "endrun",
    "epoch",
    "malformed",
    "controls",
];

/// The trial's controls input (§9.5): 32 bytes, two nonuniform full tiles, i = 0 (p = 0,
/// n1 = 16) and i = 16 (p = 16, n1 = 14)
fn controls_input() -> Vec<u8> {
    let mut v = Vec::new();
    filler(&mut v, 32);
    v
}

fn scanner_class(s: &mut Synth, class: &str) {
    let from = s.cases;
    match class {
        // 1. tiny and empty inputs
        "tiny" => {
            for n in [0usize, 1, 2, 15, 16, 17, 31, 32, 33] {
                for sigma in [1usize, 2] {
                    let v: Vec<u8> = (0..n).map(|k| ((k / 3) % sigma) as u8 * 5).collect();
                    s.case(&format!("scanner tiny n {n} sigma {sigma}"), &v, 0);
                }
            }
        }
        // 2. every tail length, after full tiles (uniform and nonuniform) and with none
        "tails" => {
            // the trial: each tail goes through the checked `tile` (its `len < 16`
            // assertion), never `tile16`, whose calls equal the nonuniform full tiles
            let tail_ok = |st: &Stats2su, l: usize, full: u64| {
                assert!(st.tails == 1 && st.tail_rows == l as u64, "tail {l}");
                assert!(
                    st.unchecked_tiles == full && st.tiles_nonuniform == full,
                    "tail {l}"
                );
            };
            for l in 1..=15usize {
                let st = s.case(
                    &format!("scanner tail {l} alone"),
                    &(0..l).map(|k| (k & 1) as u8).collect::<Vec<_>>(),
                    0,
                );
                tail_ok(&st, l, 0);
                let mut v = Vec::new();
                filler(&mut v, 16);
                v.extend((0..l).map(|k| (k % 3) as u8));
                let st = s.case(&format!("scanner tail {l} after a nonuniform tile"), &v, 0);
                tail_ok(&st, l, 1);
                let mut v = Vec::new();
                run_of(&mut v, 7, 32);
                v.extend((0..l).map(|k| 7 + (k % 2) as u8));
                let st = s.case(&format!("scanner tail {l} after a run"), &v, 0);
                tail_ok(&st, l, 0);
            }
        }
        // 3. odd alignment: the input at byte offsets 1..=7 of its allocation
        "align" => {
            let mut big = Vec::new();
            run_of(&mut big, 4, 32);
            filler(&mut big, 16);
            run_of(&mut big, 4, 48);
            filler(&mut big, 21);
            for off in 1..=7usize {
                s.case_at("scanner align tiny", &[3, 1, 2, 1, 3], 0, off);
                s.case_at("scanner align tiles", &big, 0, off);
            }
        }
        // 4. a nonuniform full tile at every reachable source phase. Full tiles start at
        //    multiples of 16, so p = 16k mod 30 is even: 0, 2, …, 28 (n1 = 16 for p ≤ 14,
        //    n1 = 30 − p = 14, …, 2 above)
        "phase" => {
            for k in 0..15usize {
                let mut v = Vec::new();
                run_of(&mut v, 9, 16 * k);
                filler(&mut v, 16);
                filler(&mut v, 5);
                assert_eq!((16 * k) % 30 % 2, 0);
                s.case(&format!("scanner phase p {}", (16 * k) % 30), &v, 0);
            }
        }
        // 5. held tiles: a run followed by another byte's uniform tile, a nonuniform tile,
        //    a tile matching in its first qword only, in its second only, and alternating
        //    one-tile runs
        "held" => {
            let mk = |next: &dyn Fn(&mut Vec<u8>)| {
                let mut v = Vec::new();
                run_of(&mut v, 6, 48);
                next(&mut v);
                filler(&mut v, 19);
                v
            };
            let cases: [(&str, Vec<u8>); 4] = [
                ("another byte's uniform tile", mk(&|v| run_of(v, 8, 16))),
                ("a nonuniform tile", mk(&|v| filler(v, 16))),
                (
                    "first qword only",
                    mk(&|v| {
                        run_of(v, 6, 8);
                        run_of(v, 8, 8);
                    }),
                ),
                (
                    "second qword only",
                    mk(&|v| {
                        run_of(v, 8, 8);
                        run_of(v, 6, 8);
                    }),
                ),
            ];
            for (label, v) in cases {
                let st = s.case(&format!("scanner held: {label}"), &v, 0);
                assert!(st.held_reuses >= 1, "{label}");
            }
            let mut v = Vec::new();
            for k in 0..8usize {
                run_of(&mut v, if k % 2 == 0 { 6 } else { 8 }, 16);
            }
            filler(&mut v, 3);
            let st = s.case("scanner held: alternating one-tile runs", &v, 0);
            assert!(st.held_reuses >= 7);
        }
        // 6. a run ending because no full tile remains: at the block's end (with and without
        //    a tail)
        "endrun" => {
            for tail in [0usize, 5] {
                let mut v = Vec::new();
                filler(&mut v, 16);
                run_of(&mut v, 5, 64);
                filler(&mut v, tail);
                let st = s.case(&format!("scanner run to the end, tail {tail}"), &v, 0);
                assert!(st.runs_no_full_tile >= 1);
            }
        }
        // 7. an epoch boundary: a run straddling 61,440 (split there), and a nonuniform
        //    tile ending exactly at it
        "epoch" => {
            let n = EPOCH + 64;
            let mut v = Vec::with_capacity(n);
            filler(&mut v, n);
            for x in &mut v[EPOCH - 48..EPOCH + 48] {
                *x = 4;
            }
            let st = s.case("scanner epoch: a run straddling 61,440", &v, 0);
            assert!(st.runs_split_at_epoch >= 1 && st.runs_no_full_tile >= 1);
            let mut v = Vec::with_capacity(n);
            run_of(&mut v, 4, EPOCH - 16);
            filler(&mut v, 16);
            run_of(&mut v, 4, 64);
            s.case("scanner epoch: a nonuniform tile ending at 61,440", &v, 0);
        }
        // 8. malformed counts: rejected by both builds, both arrays empty
        "malformed" => {
            let mut v = Vec::new();
            run_of(&mut v, 3, 48);
            filler(&mut v, 37);
            let own = hist(&v);
            for t in 0..4 {
                let mut c = own;
                match t {
                    0 => c[3] += 1,
                    1 => c[3] -= 1,
                    2 => {
                        c[3] -= 5;
                        c[200] += 5;
                    }
                    _ => c.swap(2, 3),
                }
                for counted in [true, false] {
                    let ok = if counted {
                        s.a.ctor
                            .build::<true>(&v, &c, 0, &mut s.a.cand.rec, &mut s.a.cand.raw)
                    } else {
                        s.a.ctor
                            .build::<false>(&v, &c, 0, &mut s.a.cand.rec, &mut s.a.cand.raw)
                    };
                    assert!(!ok && s.a.cand.rec.is_empty() && s.a.cand.raw.is_empty());
                }
                s.cases += 1;
            }
        }
        // 9. the trial's controls (§9.5): the 32-byte input in an exact allocation, through
        //    both builds, where the endpoint mutants must be reported; then the full checks
        "controls" => {
            let v = controls_input();
            let exact: Box<[u8]> = v.clone().into_boxed_slice();
            let counts = hist(&exact);
            for counted in [true, false] {
                let ok = if counted {
                    s.a.ctor
                        .build::<true>(&exact, &counts, 0, &mut s.a.cand.rec, &mut s.a.cand.raw)
                } else {
                    s.a.ctor.build::<false>(
                        &exact,
                        &counts,
                        0,
                        &mut s.a.cand.rec,
                        &mut s.a.cand.raw,
                    )
                };
                assert!(ok);
            }
            let st = s.case(
                "scanner controls: 32 bytes, two nonuniform full tiles",
                &v,
                0,
            );
            assert!(st.unchecked_tiles == 2 && st.tiles_second_segment == 1 && st.tails == 0);
        }
        c => panic!("unknown scanner class {c}"),
    }
    s.mark(&format!("scanner class {class}"), from);
}

/// `ctor2su-synthetic --miri CLASS`: one registered class, unpoisoned arenas
fn synthetic_miri(class: &str) {
    let mut s = Synth {
        a: Arenas::default(),
        st: Stats2su::default(),
        ck: Checked::default(),
        cases: 0,
        poisoned: !cfg!(miri),
        classes: Vec::new(),
    };
    let classes: Vec<&str> = if class == "all" {
        SCANNER_CLASSES.to_vec()
    } else {
        vec![class]
    };
    for c in classes {
        scanner_class(&mut s, c);
    }
    println!(
        "ctor2su-synthetic --miri {class}: {} cases, {} array comparisons, {} walk outputs, {} restorations: every check exact; fresh {}, lookahead {}, held {}, no-full-tile runs {}, nonuniform {}, second segments {}, tails {}",
        s.cases,
        s.ck.arrays,
        s.ck.walks,
        s.ck.restorations,
        s.st.fresh_pairs,
        s.st.lookahead_pairs,
        s.st.held_reuses,
        s.st.runs_no_full_tile,
        s.st.tiles_nonuniform,
        s.st.tiles_second_segment,
        s.st.tails
    );
}

fn synthetic(quick: bool) {
    let mut s = Synth {
        a: Arenas::default(),
        st: Stats2su::default(),
        ck: Checked::default(),
        cases: 0,
        poisoned: !cfg!(miri),
        classes: Vec::new(),
    };
    let mut rng = Rng(0xc302);

    // 1. small random blocks: every n from 0 to 130, a few alphabets and run structures
    let from = s.cases;
    let (nmax, seeds) = if quick { (64, 1) } else { (130, 8) };
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
    s.mark("small blocks, n 0..=130", from);

    // 2. a bucket boundary at every offset (crossing-group ownership by two characters)
    let from = s.cases;
    let ns: &[usize] = if quick {
        &[61]
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
    s.mark("bucket boundary at every offset", from);

    // 3. bulk-to-partial transitions: a run of m after P filler rows, the bucket at slot
    //    A, then the same byte again in a scalar tail
    let from = s.cases;
    let (ps, ms): (Vec<usize>, Vec<usize>) = if quick {
        ((0..=48).step_by(16).collect(), vec![16, 32, 61])
    } else {
        (
            (0..=240).step_by(3).collect(),
            vec![
                1, 15, 16, 17, 29, 30, 31, 32, 33, 47, 48, 59, 60, 61, 64, 90, 120, 256,
            ],
        )
    };
    let avals: Vec<usize> = if quick {
        vec![0, 29]
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
                v.push(1);
                filler(&mut v, 5);
                v.extend(std::iter::repeat(0u8).take(a0));
                s.case(&format!("transition P {p} m {m} A {a0}"), &v, 0);
            }
        }
    }
    s.mark(
        "bulk-to-partial transitions (phase x slot x run length)",
        from,
    );

    // 4. epoch boundaries: blocks of n around 61,440·k, with a rare byte placed at source
    //    rows around each boundary (and runs straddling it)
    let from = s.cases;
    let ks: &[usize] = if quick { &[1] } else { &[1, 2, 3, 5, 14, 15] };
    for &k in ks {
        let b = EPOCH * k;
        let ns: Vec<usize> = if quick {
            vec![b + 61]
        } else {
            vec![b - 1, b, b + 1, b + 30, b + 61]
        };
        for &n in &ns {
            if n > MAX_N {
                continue;
            }
            let offs: &[i64] = if quick {
                &[-1, 0, 1]
            } else {
                &[-2, -1, 0, 1, 2, 29, 30]
            };
            for &off in offs {
                let at = (b as i64 + off) as usize;
                if at + 40 >= n || at < 40 {
                    continue;
                }
                let mut v = Vec::with_capacity(n);
                filler(&mut v, n);
                // byte 1: a few rows before and after the boundary
                for t in 0..5 {
                    v[at - 30 + 7 * t] = 1;
                    v[at + 7 * t] = 1;
                }
                v[at] = 1;
                // a run of byte 4 straddling the boundary (split by the epoch)
                let r0 = (b - 48).max(16) & !15;
                for x in &mut v[r0..(r0 + 96).min(n)] {
                    *x = 4;
                }
                s.case(&format!("epoch k {k} n {n} off {off}"), &v, 0);
                if !quick {
                    s.case(&format!("epoch k {k} n {n} off {off} forced"), &v, 50);
                }
            }
        }
    }
    s.mark("epoch boundaries", from);

    // 5. absent characters across repeated snapshots: byte 1 occurs in epoch 0 and then
    //    only after several empty epochs, inside one group (the greatest-epoch rule)
    let from = s.cases;
    let gaps: &[usize] = if quick { &[1] } else { &[1, 2, 3, 6, 13] };
    let firsts: &[usize] = if quick { &[0, 29] } else { &[0, 1, 7, 29] };
    for &gap in gaps {
        for &first in firsts {
            let late = EPOCH * gap + 5;
            let n = late + 400;
            if n > MAX_N {
                continue;
            }
            // 29 rows early, one row `gap` epochs later: a wide group
            let mut ones: Vec<usize> = (0..29).map(|t| first + 2 * t + 1).collect();
            ones.push(late);
            s.case(
                &format!("absent gap {gap} first {first} wide"),
                &placed(n, &ones, 0),
                0,
            );
            // the whole group late, after absences: an ordinary group in a later epoch
            let ones: Vec<usize> = (0..30).map(|t| late + 2 * t).collect();
            s.case(
                &format!("absent gap {gap} first {first} late"),
                &placed(n, &ones, 0),
                0,
            );
            // two groups: one in epoch 0, one after the gap
            let mut ones: Vec<usize> = (0..30).map(|t| first + 2 * t + 1).collect();
            ones.extend((0..30).map(|t| late + 2 * t));
            s.case(
                &format!("absent gap {gap} first {first} two"),
                &placed(n, &ones, 0),
                0,
            );
        }
    }
    s.mark("absent characters across repeated snapshots", from);

    // 6. ordinary and wide span boundaries from full labels: the exact epoch-crossing
    //    cases (3 → 61,440 ordinary at span 65,533; 2 → 61,440 RAW at 65,534), spans around
    //    them at many first positions, and far-wider groups whose truncated u16 difference
    //    looks small
    let from = s.cases;
    let check_span = |s: &mut Synth, label: &str, first: usize, last: usize, want_wide: bool| {
        let mut ones: Vec<usize> = (0..29).map(|t| first + 2 * t).collect();
        // the 29 early rows must stay before `last`; spread them after `first`
        ones.retain(|&x| x < last);
        ones.truncate(29);
        ones.push(last);
        let n = last + 200;
        let st = s.case(label, &placed(n, &ones, 0), 0);
        assert_eq!(st.fin_wide, want_wide as u64, "{label}: wide groups");
    };
    check_span(
        &mut s,
        "span 3 -> 61440 (65,533, ordinary)",
        3,
        61_440,
        false,
    );
    check_span(&mut s, "span 2 -> 61440 (65,534, RAW)", 2, 61_440, true);
    assert_eq!(phi(61_440) - phi(3), 65_533);
    assert_eq!(phi(61_440) - phi(2), 65_534);
    if !quick {
        for first in [0usize, 1, 2, 3, 5, 17, 29, 1000, 30_001] {
            for target in 65_530u32..=65_538 {
                let fb = phi(first) + target;
                if fb & 31 >= 30 {
                    continue;
                }
                let last = super::phi_inv(fb);
                if last <= first + 60 {
                    continue;
                }
                check_span(
                    &mut s,
                    &format!("span first {first} target {target}"),
                    first,
                    last,
                    target > MAX_OFF,
                );
            }
        }
        // far wider: last = first + 61440·k (+ small), whose low-16 difference is small
        for first in [0usize, 4, 100] {
            for k in [2usize, 3, 7] {
                for extra in [0usize, 1, 31] {
                    let last = first + EPOCH * k + extra;
                    if last + 200 > MAX_N {
                        continue;
                    }
                    check_span(
                        &mut s,
                        &format!("far wide first {first} k {k} +{extra}"),
                        first,
                        last,
                        true,
                    );
                }
            }
        }
    }
    s.mark("ordinary/wide span boundaries from full labels", from);

    // 7. pending portions left open until the end of input: rare bytes whose last group
    //    never receives a later occurrence (finished only at the end), in partial and
    //    full last groups, across epochs
    let from = s.cases;
    let ns: &[usize] = if quick {
        &[61_500]
    } else {
        &[61_500, 130_000, 300_017]
    };
    for &n in ns {
        let counts: &[usize] = if quick {
            &[1, 30, 31]
        } else {
            &[1, 29, 30, 31, 59, 61]
        };
        for &count in counts {
            let mut v = Vec::with_capacity(n);
            filler(&mut v, n);
            for t in 0..count {
                v[n - 3 - 3 * t] = 9; // late rows of byte 9, its bucket after 2, 3
            }
            for t in 0..count {
                v[1 + 2 * t] = 1; // early rows of byte 1
            }
            s.case(&format!("pending n {n} count {count}"), &v, 0);
        }
    }
    s.mark("pending portions finished at the end of input", from);

    // the prototype's registered scanner classes (pre-registration §0.4)
    for c in SCANNER_CLASSES {
        scanner_class(&mut s, c);
    }

    if !quick {
        // 8. RAW discovery order unlike group order, and forced RAW
        let from = s.cases;
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
                s.case(&format!("permutation seed {seed} pm {pm}"), &v, pm);
            }
        }
        s.mark("RAW discovery order and forced RAW", from);

        // 9. one-symbol blocks
        let from = s.cases;
        for &n in &[
            1usize, 2, 15, 16, 17, 29, 30, 31, 32, 47, 48, 60, 61, 61_439, 61_440, 61_441, 65_536,
            899_999, 900_000,
        ] {
            for b in [0u8, 97, 255] {
                s.case(&format!("one symbol n {n} byte {b}"), &vec![b; n], 0);
            }
        }
        s.mark("one-symbol blocks", from);

        // 10. 900,000-row and 983,010-row blocks, natural and forced
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
        s.mark("900,000-row and 983,010-row blocks", from);
    }

    // 11. repeated construction with changing sizes, up and down, n = 0 included (both
    //     builds), poisoned and not, the forced configuration changing the RAW reserve
    let from = s.cases;
    let sizes: &[usize] = if quick {
        &[200, 1, 0, 95, 61_500, 31, 0, 150]
    } else {
        &[
            900_000, 1, 899_990, 0, 31, 65_537, 30, 900_000, 0, 59, 500_000, 61_441, 0, 65_536,
            899_999,
        ]
    };
    for round in 0..2 {
        s.poisoned = round == 1 && !cfg!(miri);
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
    s.poisoned = !cfg!(miri);
    s.mark("repeated construction with changing sizes", from);

    // 12. malformed counts: rejected exactly when the counts are not ll's own, before any
    //     write out of bounds, both arrays left empty, in both builds
    let from = s.cases;
    let (mut rejected, mut accepted) = (0u64, 0u64);
    let trials = if quick { 30 } else { 4000 };
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
            1 => c[v[rng.below(n)] as usize] -= 1,
            2 | 3 => {
                let fb = v[rng.below(n)] as usize;
                let k = 1 + rng.below(c[fb] as usize);
                c[fb] -= k as u32;
                c[rng.below(256)] += k as u32;
            }
            _ => {
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
        assert_eq!(ok, c == own, "case {t}");
        if ok {
            accepted += 1;
        } else {
            rejected += 1;
            assert!(s.a.cand.rec.is_empty() && s.a.cand.raw.is_empty());
        }
        s.cases += 1;
    }
    if !quick {
        for n in [MAX_N + 1, 1 << 24] {
            let v = vec![7u8; n];
            let c = hist(&v);
            assert!(!s
                .a
                .ctor
                .build::<false>(&v, &c, 0, &mut s.a.cand.rec, &mut s.a.cand.raw));
            assert!(s.a.cand.rec.is_empty() && s.a.cand.raw.is_empty());
            rejected += 1;
            s.cases += 1;
        }
    }
    println!("ctor2su-synthetic: malformed counts: {rejected} rejected, {accepted} accepted (equal to ll's own after the change)");
    s.mark("malformed counts and sizes", from);

    let st = &s.st;
    let shapes: usize = (0..30)
        .map(|j| (1..=ROWS - j).filter(|&k| st.cov_portion[j][k] > 0).count())
        .sum();
    println!(
        "ctor2su-synthetic coverage: finished fast {}, ordinary {}, wide {}, crossing {}, forced {}; multi-epoch portions {}; epoch searches {}; runs split at an epoch {}; portion (first slot, length) shapes {shapes}; max ordinary span {}, min wide span {}; epochs {}",
        st.fin_fast,
        st.fin_ordinary,
        st.fin_wide,
        st.fin_crossing,
        st.fin_forced,
        st.multi_epoch_portions,
        st.epoch_searches,
        st.runs_split_at_epoch,
        st.max_ordinary_span,
        st.min_wide_span,
        st.epochs
    );
    println!(
        "ctor2su-synthetic: {} cases, {} array comparisons, {} walk outputs, {} restorations: every check exact ({})",
        s.cases,
        s.ck.arrays,
        s.ck.walks,
        s.ck.restorations,
        if quick { "quick" } else { "full" }
    );
    println!("ctor2su-synthetic counters: {{{}}}", st.json());
}

/// `ctor2su-check`, `ctor2su-adversarial`, `ctor2su-synthetic`
pub fn main(mode: &str, args: &[String]) {
    match mode {
        // the ledger's cross-check: each block of each file built once by v2's and once by
        // this prototype's uncounted build, their outputs compared; run under callgrind for an
        // exact count of the instructions each `build::<false>` executes (untimed)
        "count" if args.first().map(String::as_str) == Some("--only") => {
            // one builder per process (§9.4): the same decode, the same arenas, and only
            // the named builder's calls
            let which = args[1].as_str();
            let (mut v2, mut c2s) = (super::ctor2::Ctor2::default(), Ctor2su::default());
            let (mut r, mut w) = (Vec::new(), Vec::new());
            for file in &args[2..] {
                let data = std::fs::read(file).unwrap();
                let bs = blocks(&data);
                let mut h = 0xcbf2_9ce4_8422_2325u64;
                for (ll, counts, _, _) in &bs {
                    match which {
                        "v2" => assert!(v2.build::<false>(ll, counts, 0, &mut r, &mut w)),
                        "su" => assert!(c2s.build::<false>(ll, counts, 0, &mut r, &mut w)),
                        x => panic!("--only {x}: v2 or su"),
                    }
                    for &b in bytes(&r).iter().chain(bytes(&w)) {
                        h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
                    }
                }
                println!(
                    "ctor2su-count --only {which}: {file}: {} blocks, output fnv1a {h:016x}",
                    bs.len()
                );
            }
        }
        "count" => {
            let (mut v2, mut c2s) = (super::ctor2::Ctor2::default(), Ctor2su::default());
            let (mut r1, mut w1, mut r2, mut w2) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for file in args {
                let data = std::fs::read(file).unwrap();
                let bs = blocks(&data);
                for (ll, counts, _, _) in &bs {
                    assert!(v2.build::<false>(ll, counts, 0, &mut r1, &mut w1));
                    assert!(c2s.build::<false>(ll, counts, 0, &mut r2, &mut w2));
                    assert!(bytes(&r1) == bytes(&r2) && bytes(&w1) == bytes(&w2));
                }
                println!(
                    "ctor2su-count: {file}: {} blocks built by both, identical",
                    bs.len()
                );
            }
        }
        "synthetic" if args.first().map(String::as_str) == Some("--miri") => {
            synthetic_miri(args.get(1).map_or("all", String::as_str))
        }
        "synthetic" => synthetic(args.iter().any(|a| a == "--quick")),
        "check" | "adversarial" => {
            let mut jl = std::fs::File::create(&args[0]).unwrap();
            let mut total = Stats2su::default();
            if mode == "check" {
                for file in &args[1..] {
                    let data = std::fs::read(file).unwrap();
                    let name = std::path::Path::new(file)
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();
                    let traffic = name == "wikidata-20261026.bz2";
                    total.add(&check_input(&name, &data, &mut jl, traffic));
                }
            } else {
                for (name, data) in adversarial() {
                    for level in [1, 9] {
                        let z = compress(&data, level);
                        total.add(&check_input(
                            &format!("{name} -{level}"),
                            &z,
                            &mut jl,
                            false,
                        ));
                    }
                }
            }
            println!("\ntotals (natural): {{{}}}", total.json());
        }
        m => panic!("unknown mode ctor2su-{m}"),
    }
}
