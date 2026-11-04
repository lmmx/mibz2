//! The compact30 combined build, assembly 1 (the research charter's §6.3, with v3's
//! finaliser under §7): constructor v2 (`ctor2.rs` at `8beffe2`) with three components,
//! each carried from its pinned isolated module and nothing else changed. Untimed, research
//! only, not wired into the decoder; `ctor2.rs`, `ctor3.rs`, `ctor2su.rs` and `ctor2d.rs`
//! are unchanged.
//!
//!     cargo run --release --example c30walk -- ctor2c-check OUT.jsonl FILE.bz2 [FILE.bz2 ...]
//!     cargo run --release --example c30walk -- ctor2c-adversarial OUT.jsonl
//!     cargo run --release --example c30walk -- ctor2c-synthetic [--quick | --miri CLASS]
//!     cargo run --release --example c30walk -- ctor2c-count [--only v2|c | --counted] FILE.bz2 ...
//!
//! **The components:**
//! - **The scanner with U2** (prototype 1's U1/U2 trial, `ctor2su.rs` at `ba261ef`): each
//!   epoch's unread source is a shrinking slice `rest`, a full tile is split off it with
//!   `split_first_chunk::<16>()` and its two qwords read from the 16-byte array; a
//!   nonuniform full tile goes through `tile16`, whose first segment is a do-while over the
//!   window and whose second is `ll.get_unchecked(i + n1..i + 16)` (U2, unchanged: the
//!   full-tile contract i + 16 ≤ end ≤ `ll.len()` and n1 = min(16, 30 − p) ≤ 16). The tail
//!   keeps v2's checked `tile`. U2 is the module's only unchecked access; U1 and U3 are
//!   absent.
//! - **The hot descriptor arrays** (prototype 2, `ctor2d.rs` at `7620c0c`): v2's `Desc` is
//!   stored as `wr[c]`, `en[c]` and `cold[c] = {w0, r0, lim, g, raw}`, each field exactly
//!   once, and assembled on reading (`W::desc`).
//! - **v3's epoch-tagged finaliser** (`ctor3.rs` at `dbf5139`): `raw` holds a preset RAW
//!   group's index (high bit clear) or the ordinary candidate's tag `epoch_tag(e)`;
//!   `active_tag` is set with each epoch's snapshot; `finish` is the leaf (30 fields and
//!   `raw == active_tag`), everything else `finish_slow` (v2's finish with the high-bit
//!   test). On the split storage the leaf reads `cold[c].raw`, `cold[c].w0` and `wr[c]`
//!   where v3 read `desc[c].raw`, `.w0` and `.write`: the same values, by the bijection.
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

/// A full tile's two qwords, read from its 16-byte array at constant offsets (scanner)
#[inline(always)]
fn halves(t: &[u8; 16]) -> (u64, u64) {
    let (lo, hi) = t.split_at(8);
    (
        u64::from_ne_bytes(lo.try_into().unwrap()),
        u64::from_ne_bytes(hi.try_into().unwrap()),
    )
}

/// The untimed structural counters (`build::<true>` only): v2's fields, which keep v2's
/// meanings (compared with v2's counted build field by field), then the components' own
macro_rules! stats {
    ($($f:ident),* $(,)?) => {
        #[derive(Clone, Default, Debug)]
        pub struct Stats2c {
            /// coverage: the portions' first slot and length (ordinary and RAW)
            pub cov_portion: [[u64; 31]; 30],
            pub max_ordinary_span: u64,
            pub min_wide_span: u64,
            /// the combined build's own (counted build only): nonuniform full tiles by their
            /// first segment's length n1 (index n1 ≤ 16), and `tile16`'s refills by the first
            /// segment's row position k (LLVM unrolls that do-while by position)
            pub tile_n1: [u64; 17],
            pub tile_refills_first_at: [u64; 16],
            $(pub $f: u64,)*
        }

        impl Stats2c {
            pub fn add(&mut self, o: &Stats2c) {
                $( self.$f += o.$f; )*
                for (a, b) in self.tile_n1.iter_mut().zip(o.tile_n1.iter()) {
                    *a += b;
                }
                for (a, b) in self.tile_refills_first_at.iter_mut().zip(o.tile_refills_first_at.iter()) {
                    *a += b;
                }
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
                let mut s = format!(
                    "\"max_ordinary_span\":{},\"min_wide_span\":{}",
                    self.max_ordinary_span, self.min_wide_span
                );
                $( s.push_str(&format!(",\"{}\":{}", stringify!($f), self.$f)); )*
                for (j, x) in self.tile_n1.iter().enumerate() {
                    s.push_str(&format!(",\"tile_n1_{j}\":{x}"));
                }
                for (k, x) in self.tile_refills_first_at.iter().enumerate() {
                    s.push_str(&format!(",\"tile_refills_first_at_{k}\":{x}"));
                }
                s
            }
        }
    };
}

stats!(
    // v2's (B2's) fields
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
    // the scanner's scan paths (A, J, H, Q, S, X, tails), its unchecked tiles, and its
    // S62-F01 counters
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
    epochs_untailed,
    bitmap_memsets,
    // the descriptor split's path counters and its S62-F01 counters (those not above)
    refills,
    refill_finishes,
    outer_iterations,
    fills,
    runs_end_fill,
    runs_end_complete,
    run_opens,
    tile_rows_second,
    sweep_checks,
    init_descs,
    pad_blocks,
    bitmap_in_place,
    bitmap_reserves,
    bitmap_reserves_single,
    rec_grows,
    rec_grow_frees,
    raw_grows,
    snaps_reserves,
    // v3's: leaf finishes; v2-fast finishes with 30 fields (the eligible cases, recorded
    // independently of the leaf's test); slow entries by class; ordinary and preset-RAW
    // openings
    leaf_fast,
    v2_fast_full,
    slow_entries,
    slow_ordinary_old_epoch,
    slow_ordinary_partial_same_epoch,
    slow_raw_preset,
    opens_ordinary,
    opens_raw,
    // the combined build's own changed-path counters (counted build only): `finish` calls
    // from `run`'s close; the crossing pre-marking scan of the 256 counts; the final close
    // loop's iterations
    run_finishes,
    crossing_scan,
    final_closes,
    // the partial normalisation's scalar iterations in `finish_slow`: its compiled loop takes
    // 16 fields at a time while k ≥ 16, then 4 at a time once k ≥ 4, and the rest (k mod 4,
    // or all k when k < 4) one at a time
    norm_partial_scalar,
);

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

/// A character's write descriptor, as v2 has it, assembled from the split storage. Empty
/// (no open portion) when `w0 == write == end` (null); otherwise its open portion is the
/// fields `[w0, end)` of group `g`, starting at destination rank `r0`, of which
/// `[w0, write)` hold temporary labels.
#[derive(Clone, Copy)]
struct Desc {
    write: *mut u16,
    end: *mut u16,
    w0: *mut u16,
    r0: u32,
    g: u32,
    raw: u32,
}

/// The constructor's reusable scratch: the RAW bitmap, the epoch snapshots, the counters
#[derive(Default)]
pub struct Ctor2c {
    bitmap: Vec<u64>,
    snaps: Vec<[u32; 256]>,
    pub st: Stats2c,
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
    st: &'a mut Stats2c,
}

impl<const COUNT: bool> W<'_, COUNT> {
    /// Whether `one(c, _)` takes its refill detour (the open portion is full, or there is
    /// none): read by the counted build's counters only
    #[inline(always)]
    fn refills(&self, c: u8) -> bool {
        self.wr[c as usize] == self.en[c as usize]
    }

    /// v2's descriptor of character c, assembled from the split storage
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

    /// Save every character's next destination rank: the snapshot A_e; then the epoch's tag
    fn snapshot(&mut self) {
        let mut a = [0u32; 256];
        for (c, x) in a.iter_mut().enumerate() {
            *x = self.rank(c);
        }
        self.snaps.push(a);
        // the epoch being entered: its index is its snapshot's, before any of its rows
        self.active_tag = epoch_tag(self.snaps.len() - 1);
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
        let lim = self.cold[c].lim;
        chk!(COUNT, r < lim);
        let g = r as usize / ROWS;
        let re = lim.min(((g + 1) * ROWS) as u32);
        let w0 = self.field(r);
        let raw = if self.is_raw(g) {
            if COUNT {
                self.st.opens_raw += 1;
            }
            ptr::addr_of!((*self.dst.add(g)).h).read() & M20
        } else {
            // an ordinary candidate covers its whole group: a non-crossing group's slots
            // all belong to one bucket, whose earlier portions end at group boundaries.
            // Its tag is the current epoch's: the opening is followed at once by the
            // portion's first append (refill → one, or run → open → fill), with no epoch
            // change between
            chk!(COUNT, r as usize % ROWS == 0);
            if COUNT {
                self.st.opens_ordinary += 1;
            }
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
            chk!(COUNT, c < 256);
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
        if COUNT {
            self.st.refills += 1;
            self.st.refill_finishes += (self.wr[c] != self.cold[c].w0) as u64;
        }
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
    /// Why the leaf is exact (v3's release-mode argument, its journal's §3, unchanged):
    /// `raw == active_tag` has the high bit set, so the portion is ordinary, and was opened
    /// in the current epoch e; its first append followed the opening at once, so every one
    /// of its rows was received in epoch e (finishing happens within e or at its end), and
    /// its labels are 65536·e + u with u ≤ 65,533 increasing: the span is ul − u0 ≤ 65,533
    /// and the first full label is y0 = 65536·e + u0, so y0 << 8 = (tag & !ORD) | u0 << 8.
    /// 30 fields means `w0` is its record's field 0 (a portion never passes its group's
    /// end), so the header is at `w0 + 60` bytes, and `[w0, w0 + 30)` were all written.
    /// On the split storage, `cold[c].raw`, `cold[c].w0` and `wr[c]` are v3's `desc[c].raw`,
    /// `.w0` and `.write`, each held exactly once.
    #[inline(never)]
    unsafe fn finish(&mut self, c: u8) {
        let k = &self.cold[c as usize];
        let tag = k.raw;
        if tag == self.active_tag && self.wr[c as usize] as usize - k.w0 as usize == 2 * ROWS {
            let p = k.w0;
            if COUNT {
                self.count_leaf(c as usize);
            }
            let u0 = p.read();
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

    /// The counted build's checks and counters for a leaf finish (v3's): v2's
    /// classification of the same portion must have been fast, and its span ordinary
    #[cold]
    #[inline(never)]
    unsafe fn count_leaf(&mut self, c: usize) {
        let d = self.desc(c);
        let last = self.snaps.len() - 1;
        assert!(
            self.snaps[last][c] <= d.r0,
            "a leaf finish v2 would not call fast"
        );
        assert!(d.raw & ORD != 0 && (d.raw & !ORD) >> 24 == last as u32);
        assert!(d.write == d.end && d.r0 as usize % ROWS == 0);
        let span = d.w0.add(ROWS - 1).read() as u64 - d.w0.read() as u64;
        assert!(
            span <= MAX_OFF as u64,
            "a same-epoch portion with a wide span"
        );
        let st = &mut self.st;
        st.leaf_fast += 1;
        st.v2_fast_full += 1;
        st.cov_portion[0][ROWS] += 1;
        st.fin_fast += 1;
        st.norm_full += 1;
        st.norm_fields += ROWS as u64;
        st.max_ordinary_span = st.max_ordinary_span.max(span);
    }

    /// Finish c's full pending portion: classify it from its full labels, then write its
    /// final form (once). v2's `finish`, with the ordinary/RAW test on `raw`'s high bit
    /// (v3's `finish_slow`)
    #[cold]
    #[inline(never)]
    unsafe fn finish_slow(&mut self, c: usize) {
        let d = self.desc(c);
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
            self.st.slow_entries += 1;
            if d.raw & ORD != 0 {
                // the opening epoch is the first row's (the leaf's premise)
                assert!(
                    (d.raw & !ORD) >> 24 == h0 as u32,
                    "an opening epoch is not h(r0)"
                );
                if d.raw != self.active_tag {
                    self.st.slow_ordinary_old_epoch += 1;
                } else {
                    // a same-epoch ordinary portion reaches here only when partial
                    assert!(k < ROWS, "an eligible portion missed the leaf");
                    self.st.slow_ordinary_partial_same_epoch += 1;
                }
            } else {
                assert!((d.raw as usize) < self.raw.len());
                self.st.slow_raw_preset += 1;
            }
        }
        if d.raw & ORD != 0 {
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
                        // an eligible portion here would be a leaf miss (also asserted above)
                        self.st.v2_fast_full += (k == ROWS) as u64;
                    } else {
                        self.st.fin_ordinary += 1;
                    }
                    self.st.norm_full += (k == ROWS) as u64;
                    self.st.norm_partial += (k < ROWS) as u64;
                    self.st.norm_fields += k as u64;
                    if k < ROWS {
                        self.st.norm_partial_scalar += if k < 4 { k } else { k % 4 } as u64;
                    }
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
        if self.wr[ci] == self.en[ci] && !self.refill(ci) {
            return false;
        }
        // the pointer re-read after the store, as v2's `d.write = d.write.add(1)` (the
        // descriptor prototype's attempt 2)
        self.wr[ci].write(u);
        self.wr[ci] = self.wr[ci].add(1);
        if COUNT {
            self.st.rows_scalar += 1;
        }
        true
    }

    /// v2's tile, unchanged, for the tail only (full nonuniform tiles take `tile16`)
    #[inline(always)]
    unsafe fn tile(&mut self, ll: &[u8], i: usize, len: usize) -> bool {
        // the checked path is the tail's only: a tail never takes U2
        chk!(COUNT, len < 16);
        if COUNT {
            self.st.tails += 1;
            self.st.tail_rows += len as u64;
            self.st.tails_len1 += (len == 1) as u64;
            self.st.tails_len_ge2 += (len >= 2) as u64;
            self.st.tails_even += (len % 2 == 0) as u64;
            self.st.tails_second_segment += (len > len.min(ROWS - self.p)) as u64;
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

    /// A nonuniform full tile (the scanner's U2 form, unchanged): `t` is the 16-byte chunk
    /// that `split_first_chunk` split off `rest`, at row index i
    ///
    /// The contract (the scanner journal's §9.2): t = `ll[i..i + 16]`, so
    /// i + 16 ≤ end ≤ `ll.len()`; and n1 = min(16, 30 − p) ≤ 16 for any p. Then
    /// U2 = [i + n1, i + 16) is well-ordered and inside `ll`. (U1, [i, i + n1), is not
    /// used: the do-while reads the first segment from `t`.)
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
        if COUNT {
            self.st.tile_n1[n1] += 1;
        }
        // SAFETY (U2): i + n1 ≤ i + 16 ≤ ll.len() by the contract above
        let s2 = unsafe { ll.get_unchecked(i + n1..i + 16) };
        // C4, safe: the first segment as a do-while over the 16-byte window, t[k] with
        // k < n1 ≤ 16 (U1 is not used). p < 30, so n1 ≥ 1, matters for correctness only:
        // were n1 0, this would append one wrong row, reading in bounds.
        let mut k = 0usize;
        loop {
            if COUNT {
                self.st.tile_refills_first += self.refills(t[k]) as u64;
                self.st.tile_refills_first_at[k] += self.refills(t[k]) as u64;
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
                self.st.tile_rows_second += 1;
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
        let w = self.wr[c];
        chk!(COUNT, m <= (self.en[c] as usize - w as usize) / 2);
        for (t, &o) in pat[..m].iter().enumerate() {
            w.add(t).write((y + o as u32) as u16);
        }
        self.wr[c] = w.add(m);
        self.advance(m);
        if COUNT {
            self.st.rows_fragment += m as u64;
            self.st.fills += 1;
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
        if self.rank(ci) as usize + m > self.cold[ci].lim as usize {
            return false;
        }
        if COUNT {
            self.st.runs += 1;
            self.st.run_rows += m as u64;
        }
        loop {
            let room = (self.en[ci] as usize - self.wr[ci] as usize) / 2;
            let k = room.min(m);
            if k > 0 {
                self.fill(ci, k);
                m -= k;
            }
            if m == 0 {
                if COUNT {
                    self.st.runs_end_fill += 1;
                }
                return true;
            }
            if COUNT {
                self.st.run_finishes += (self.wr[ci] != self.cold[ci].w0) as u64;
            }
            self.close(ci);
            let r = self.cold[ci].r0 as usize;
            if r % ROWS == 0 && m >= ROWS {
                let whole = m / ROWS;
                self.complete(c, r / ROWS, whole, forced_cfg);
                m -= ROWS * whole;
                self.cold[ci].r0 = (r + ROWS * whole) as u32;
                if m == 0 {
                    if COUNT {
                        self.st.runs_end_complete += 1;
                    }
                    return true;
                }
            }
            let r = self.cold[ci].r0;
            if COUNT {
                self.st.run_opens += 1;
            }
            self.open(ci, r);
        }
    }
}

impl Ctor2c {
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
            if COUNT {
                self.st.rec_grows += 1;
                self.st.rec_grow_frees += (rec.capacity() > 0) as u64;
            }
            *rec = Vec::new();
            rec.reserve_exact(g_count + 1);
        }
        if raw.capacity() < raw_req {
            if COUNT {
                self.st.raw_grows += 1;
            }
            *raw = Vec::new();
            raw.reserve_exact(raw_req);
        }
        let Ctor2c { bitmap, snaps, st } = self;
        bitmap.clear();
        if COUNT {
            let words = g_count.div_ceil(64);
            let grows = bitmap.capacity() < words;
            st.bitmap_in_place += (words >= 1 && !grows) as u64;
            st.bitmap_memsets += (words >= 2) as u64;
            st.bitmap_reserves += grows as u64;
            st.bitmap_reserves_single += (grows && words == 1) as u64;
        }
        bitmap.resize(g_count.div_ceil(64), 0);
        snaps.clear();
        if COUNT {
            st.snaps_reserves += (snaps.capacity() < MAX_EPOCHS) as u64;
        }
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
            if COUNT {
                st.init_descs += 1;
            }
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
                wr: [ptr::null_mut(); 256],
                en: [ptr::null_mut(); 256],
                cold,
                y: 0,
                p: 0,
                // replaced by epoch 0's tag at the first snapshot, before any opening
                active_tag: UNOPENED,
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
                if COUNT {
                    st.pad_blocks += 1;
                }
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
            if COUNT {
                w.st.crossing_scan += 1;
            }
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
                if COUNT {
                    w.st.outer_iterations += 1;
                }
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
                    // the scanner's tile, at the tile's row index in `ll`
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
            if COUNT {
                w.st.sweep_checks += 1;
            }
            if w.rank(c) != w.cold[c].lim {
                return false;
            }
        }
        for c in 0..256 {
            if COUNT {
                w.st.final_finishes += (w.wr[c] != w.cold[c].w0) as u64;
                w.st.final_closes += 1;
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
    ctor: Ctor2c,
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

/// The combined build's counters against v2's counted build on the same input: every v2 field
/// equal (the row classification, run coalescing and splitting, finish classes,
/// normalisation, RAW work, epochs, spans and portion shapes), and the scan paths'
/// identities for a block of n rows: full tiles T = Σ over epochs ⌊rows/16⌋ = ⌊n/16⌋ (an
/// epoch is 3,840 tiles), fresh + lookahead = T, nonuniform S = T − run_rows/16, runs R =
/// held reuses H + runs ending with no full tile Q, fresh = S + Q, tail rows = n − 16T
fn same_as_v2(what: &str, s: &Stats2c, v: &super::ctor2::Stats2, n: usize) {
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
    assert!(s.bitmap_memsets <= s.blocks, "{what}: bitmap fills");
    // the descriptor prototype's identities (its S62-F01 counters' partitions)
    assert_eq!(s.outer_iterations, s.tiles_nonuniform + s.runs, "{what}: outer iterations");
    assert_eq!(
        s.runs_end_fill + s.runs_end_complete,
        s.runs,
        "{what}: run endings"
    );
    assert_eq!(s.sweep_checks, 256 * s.blocks, "{what}: sweep checks");
    assert_eq!(
        s.init_descs,
        256 * s.blocks,
        "{what}: descriptors initialised"
    );
    assert!(s.fills_vector <= s.fills, "{what}: vector fills");
    assert!(
        s.tile_rows_second + s.tail_rows <= s.rows_scalar,
        "{what}: scalar rows by segment"
    );
    assert!(
        s.tile_refills_second + s.tail_refills_first + s.tail_refills_second <= s.refills,
        "{what}: refills by site"
    );
    assert!(s.refill_finishes <= s.refills, "{what}: refill finishes");
    assert!(s.pad_blocks <= s.blocks, "{what}: padded blocks");
    assert!(
        s.bitmap_reserves_single <= s.bitmap_reserves,
        "{what}: bitmap reserves"
    );
    // v3's identities: every eligible finish (v2-fast, 30 fields) took the leaf, and
    // nothing else did
    assert_eq!(s.leaf_fast, s.v2_fast_full, "{what}: leaf finishes");
    // v2's fast finishes are the leaf's and the same-epoch partial ones
    assert_eq!(
        s.fin_fast,
        s.leaf_fast + s.slow_ordinary_partial_same_epoch,
        "{what}: fast"
    );
    // every finish is the leaf's or the slow path's
    let finishes = s.fin_fast + s.fin_ordinary + s.fin_wide + s.fin_crossing + s.fin_forced;
    assert_eq!(
        finishes,
        s.leaf_fast + s.slow_entries,
        "{what}: finish total"
    );
    assert_eq!(
        s.slow_entries,
        s.slow_ordinary_old_epoch + s.slow_ordinary_partial_same_epoch + s.slow_raw_preset,
        "{what}: slow classes"
    );
    assert_eq!(
        s.slow_raw_preset,
        s.fin_crossing + s.fin_forced,
        "{what}: preset RAW"
    );
    assert!(s.final_finishes <= finishes, "{what}: final finishes");
    // the combined build's own counters: their partitions of the existing counters
    assert_eq!(
        s.tile_n1.iter().sum::<u64>(),
        s.tiles_nonuniform,
        "{what}: tiles by n1"
    );
    assert!(
        (0..17).all(|j| s.tile_n1[j] == 0 || (j >= 2 && j % 2 == 0)),
        "{what}: n1 is even, 2..=16"
    );
    assert_eq!(
        s.tile_refills_first_at.iter().sum::<u64>(),
        s.tile_refills_first,
        "{what}: first-segment refills by position"
    );
    assert_eq!(s.crossing_scan, 256 * s.blocks, "{what}: crossing scan");
    assert_eq!(s.final_closes, 256 * s.blocks, "{what}: final closes");
    assert!(
        s.run_finishes <= s.run_opens + s.runs_end_complete,
        "{what}: run finishes"
    );
}

/// One block, one configuration: the oracle; Ctor2c's counted build into a poisoned reused
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
) -> Stats2c {
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
        panic!("{what} pm {pm}: Ctor2c's counted build differs: {d}");
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
        panic!("{what} pm {pm}: Ctor2c's uncounted build differs: {d}");
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

fn check_input(name: &str, data: &[u8], jl: &mut std::fs::File, traffic: bool) -> Stats2c {
    let bs = blocks(data);
    let mut nat = Stats2c::default();
    for &(pm, extra) in &CONFIGS {
        let next = AtomicUsize::new(0);
        let results: Vec<(Stats2c, Checked, V1Traffic)> = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..3)
                .map(|_| {
                    sc.spawn(|| {
                        let mut a = Arenas::default();
                        let (mut st, mut ck, mut v1) = (
                            Stats2c::default(),
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
            Stats2c::default(),
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
    st: Stats2c,
    ck: Checked,
    cases: u64,
    poisoned: bool,
    classes: Vec<(String, u64)>,
}

impl Synth {
    fn case(&mut self, what: &str, ll: &[u8], pm: u32) -> Stats2c {
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
            return Stats2c::default();
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
    fn case_at(&mut self, what: &str, ll: &[u8], pm: u32, off: usize) -> Stats2c {
        let mut buf = vec![0xEEu8; off];
        buf.extend_from_slice(ll);
        self.case(&format!("{what} offset {off}"), &buf[off..], pm)
    }

    fn mark(&mut self, name: &str, from: u64) {
        println!(
            "ctor2c-synthetic: {name}: {} cases exact",
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
            let tail_ok = |st: &Stats2c, l: usize, full: u64| {
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

/// The descriptor prototype's registered classes (its pre-registration §0.4), run as
/// `descriptor:CLASS` under Miri, all of them in the full and quick suites
const DESC_CLASSES: [&str; 9] = [
    "tiny",
    "slots",
    "allbytes",
    "crossing",
    "runs",
    "epoch",
    "pending",
    "malformed",
    "reuse",
];

fn desc_class(s: &mut Synth, class: &str) {
    let from = s.cases;
    match class {
        // 1. tiny and empty inputs
        "tiny" => {
            for n in [0usize, 1, 2, 15, 16, 17, 31, 32, 33] {
                for sigma in [1usize, 2] {
                    let v: Vec<u8> = (0..n).map(|k| ((k / 3) % sigma) as u8 * 5).collect();
                    s.case(&format!("descriptor tiny n {n} sigma {sigma}"), &v, 0);
                }
            }
        }
        // 2. a refill at every slot: characters interleaved so that each one's portions
        //    close at every slot of its groups
        "slots" => {
            for sigma in [2usize, 3] {
                for shift in 0..30usize {
                    let n = 90 + shift;
                    let v: Vec<u8> = (0..n)
                        .map(|k| ((k * 7 + shift) % sigma) as u8 + 1)
                        .collect();
                    s.case(
                        &format!("descriptor slots sigma {sigma} shift {shift}"),
                        &v,
                        0,
                    );
                }
            }
        }
        // 3. all 256 byte values in one block: every hot and cold index, 255 included
        "allbytes" => {
            let mut v: Vec<u8> = (0..=255u8).collect();
            v.extend((0..=255u8).rev());
            v.extend((0..512usize).map(|k| (k * 37 % 256) as u8));
            s.case("descriptor all 256 bytes", &v, 0);
        }
        // 4. crossing groups: a bucket boundary at every offset, n = 61
        "crossing" => {
            for bnd in 1..61usize {
                let mut v = vec![1u8; bnd];
                v.extend(std::iter::repeat(2u8).take(61 - bnd));
                let rev: Vec<u8> = v.iter().rev().copied().collect();
                s.case(&format!("descriptor crossing B {bnd}"), &v, 0);
                s.case(&format!("descriptor crossing B {bnd} reversed"), &rev, 0);
            }
        }
        // 5. runs: a run's fill, close, complete and open, ending inside a group and
        //    spanning several
        "runs" => {
            for (p, m) in [
                (0usize, 16usize),
                (5, 32),
                (13, 48),
                (29, 64),
                (3, 96),
                (7, 160),
            ] {
                let mut v = Vec::new();
                filler(&mut v, p);
                v.extend(std::iter::repeat(1u8).take(m));
                filler(&mut v, 23);
                v.push(1);
                filler(&mut v, 9);
                s.case(&format!("descriptor runs P {p} m {m}"), &v, 0);
            }
        }
        // 6. an epoch boundary: the snapshot reads the split arrays
        "epoch" => {
            let n = EPOCH + 64;
            let mut v = Vec::with_capacity(n);
            filler(&mut v, n);
            for t in 0..5 {
                v[EPOCH - 30 + 7 * t] = 1;
                v[EPOCH + 7 * t] = 1;
            }
            for x in &mut v[EPOCH - 48..EPOCH + 48] {
                *x = 4;
            }
            s.case("descriptor epoch boundary", &v, 0);
        }
        // 7. pending portions finished by the final sweep
        "pending" => {
            for count in [1usize, 29, 30, 31] {
                let n = 200 + count;
                let mut v = Vec::with_capacity(n);
                filler(&mut v, n);
                for t in 0..count {
                    v[n - 1 - 3 * t.min(60)] = 9;
                }
                s.case(&format!("descriptor pending count {count}"), &v, 0);
            }
        }
        // 8. malformed counts: rejected by both builds, both arrays empty
        "malformed" => {
            let mut v = Vec::new();
            filler(&mut v, 40);
            v.extend(std::iter::repeat(3u8).take(33));
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
        // 9. reuse: sizes up and down on one constructor, n = 0 included
        "reuse" => {
            for (k, n) in [200usize, 1, 0, 95, 31, 0, 150].into_iter().enumerate() {
                let v: Vec<u8> = (0..n).map(|x| ((x / 5 + k) % 4) as u8 * 3).collect();
                s.case(
                    &format!("descriptor reuse {k} n {n}"),
                    &v,
                    [0, 1000, 50][k % 3],
                );
            }
        }
        c => panic!("unknown descriptor class {c}"),
    }
    s.mark(&format!("descriptor class {class}"), from);
}

/// The slow finish's preset-RAW test (v3's discriminator)
fn is_preset_raw(raw: u32) -> bool {
    raw & ORD == 0
}

/// v3's targeted Miri selection (`ctor3-synthetic --miri`, 134 cases), as its four marked
/// groups, each runnable on its own as `v3:CLASS`: small blocks, the tag/RAW dispatch
/// counterexample (tiny, and a later epoch's partial portion), and the exact span cases
/// 3 → 61,440 (65,533, ordinary) and 2 → 61,440 (65,534, RAW)
const V3_CLASSES: [&str; 4] = ["small", "tag-tiny", "span", "tag-epoch1"];

fn v3_class(s: &mut Synth, class: &str) {
    let from = s.cases;
    match class {
        "small" => {
            let mut rng = Rng(0xc303);
            for n in 0..=40usize {
                for sigma in [1usize, 2, 3] {
                    let mut v = Vec::with_capacity(n);
                    while v.len() < n {
                        let len = [1, 2, 7, 16, 17, 30, 32, 33][rng.below(8)];
                        let b = (10 * rng.below(sigma)) as u8;
                        v.extend(std::iter::repeat(b).take(len.min(n - v.len())));
                    }
                    s.case(&format!("miri small n {n} sigma {sigma}"), &v, 0);
                }
            }
            for n in [1usize, 2, 31, 61] {
                s.case(&format!("miri small n {n} forced"), &vec![5u8; n], 1000);
            }
        }
        "tag-tiny" => {
            for n in 1..=4usize {
                let v: Vec<u8> = (0..n).map(|k| (k % 2) as u8 * 7).collect();
                s.case(&format!("tag dispatch tiny n {n}"), &v, 0);
            }
        }
        "span" => {
            for (label, first, want_wide) in [
                ("span 3 -> 61440 (65,533, ordinary)", 3usize, false),
                ("span 2 -> 61440 (65,534, RAW)", 2usize, true),
            ] {
                let mut ones: Vec<usize> = (0..29).map(|t| first + 2 * t).collect();
                ones.push(61_440);
                let st = s.case(label, &placed(61_440 + 200, &ones, 0), 0);
                assert_eq!(st.fin_wide, want_wide as u64, "{label}: wide groups");
            }
        }
        "tag-epoch1" => {
            let n = EPOCH + 300 + 1;
            let mut v = Vec::with_capacity(n);
            filler(&mut v, n);
            v[EPOCH + 10] = 9;
            let st = s.case("tag dispatch epoch 1 count 1", &v, 0);
            assert!(st.slow_ordinary_partial_same_epoch >= 1);
        }
        c => panic!("unknown v3 class {c}"),
    }
    s.mark(&format!("v3 class {class}"), from);
}

/// The union of the components' registered Miri classes, as `COMPONENT:CLASS`
fn union_classes() -> Vec<String> {
    let mut v: Vec<String> = SCANNER_CLASSES.iter().map(|c| format!("scanner:{c}")).collect();
    v.extend(DESC_CLASSES.iter().map(|c| format!("descriptor:{c}")));
    v.extend(V3_CLASSES.iter().map(|c| format!("v3:{c}")));
    v
}

fn run_class(s: &mut Synth, class: &str) {
    match class.split_once(':') {
        Some(("scanner", c)) => scanner_class(s, c),
        Some(("descriptor", c)) => desc_class(s, c),
        Some(("v3", c)) => v3_class(s, c),
        _ => panic!("unknown class {class}: COMPONENT:CLASS, COMPONENT one of scanner, descriptor, v3"),
    }
}

/// `ctor2c-synthetic --miri CLASS`: one registered class of the union (`COMPONENT:CLASS`),
/// a component's classes (`COMPONENT`), or all of them (`all`); unpoisoned arenas
fn synthetic_miri(class: &str) {
    let mut s = Synth {
        a: Arenas::default(),
        st: Stats2c::default(),
        ck: Checked::default(),
        cases: 0,
        poisoned: !cfg!(miri),
        classes: Vec::new(),
    };
    let classes: Vec<String> = if class == "all" {
        union_classes()
    } else if !class.contains(':') {
        union_classes()
            .into_iter()
            .filter(|c| c.starts_with(&format!("{class}:")))
            .collect()
    } else {
        vec![class.to_string()]
    };
    assert!(!classes.is_empty(), "no class {class}");
    for c in &classes {
        run_class(&mut s, c);
    }
    println!(
        "ctor2c-synthetic --miri {class}: {} cases, {} array comparisons, {} walk outputs, {} restorations: every check exact; fresh {}, lookahead {}, held {}, no-full-tile runs {}, nonuniform {}, second segments {}, tails {}; scalar rows {}, fragment rows {}, runs {}, refills {}, crossing {}, forced {}, epochs {}; leaf {}, slow {} (old epoch {}, partial {}, preset RAW {}), wide {}",
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
        s.st.tails,
        s.st.rows_scalar,
        s.st.rows_fragment,
        s.st.runs,
        s.st.refills,
        s.st.fin_crossing,
        s.st.fin_forced,
        s.st.epochs,
        s.st.leaf_fast,
        s.st.slow_entries,
        s.st.slow_ordinary_old_epoch,
        s.st.slow_ordinary_partial_same_epoch,
        s.st.slow_raw_preset,
        s.st.fin_wide
    );
}

fn synthetic(quick: bool) {
    let mut s = Synth {
        a: Arenas::default(),
        st: Stats2c::default(),
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

    // the scanner trial's registered classes (its pre-registration §0.4)
    for c in SCANNER_CLASSES {
        scanner_class(&mut s, c);
    }

    // the descriptor prototype's registered classes (its pre-registration §0.4)
    for c in DESC_CLASSES {
        desc_class(&mut s, c);
    }

    // v3: the tag/RAW dispatch (the audit's counterexample). A tagged ordinary portion
    // that reaches the slow finish (a one-row block; a partial group, in epoch 0 and in a
    // later epoch, so with tags 0x8000_0000, 0x8100_0000, …) must take the ordinary path:
    // with v2's `== NO_RAW` test it would be read as RAW index 2^31 or above
    let from = s.cases;
    for n in 1..=6usize {
        for sigma in [1usize, 2] {
            let v: Vec<u8> = (0..n).map(|k| (k % sigma) as u8 * 7).collect();
            s.case(&format!("tag dispatch tiny n {n} sigma {sigma}"), &v, 0);
        }
    }
    let ks: &[usize] = if quick { &[1] } else { &[1, 2, 5, 15] };
    for &k in ks {
        for count in [1usize, 7, 29] {
            // byte 9's bucket (the last) starts at n − count, a multiple of 30, so its group
            // is ordinary, and partial: one portion, its rows all in epoch k, finished at
            // the end
            let n = EPOCH * k + 300 + count;
            if n > MAX_N {
                continue;
            }
            let mut v = Vec::with_capacity(n);
            filler(&mut v, n);
            for t in 0..count {
                v[EPOCH * k + 10 + 3 * t] = 9;
            }
            let st = s.case(&format!("tag dispatch epoch {k} count {count}"), &v, 0);
            assert!(st.slow_ordinary_partial_same_epoch >= 1);
        }
    }
    assert!(!is_preset_raw(epoch_tag(0)) && epoch_tag(0) != UNOPENED);
    for e in 0..MAX_EPOCHS {
        // what v2's test would have concluded: not NO_RAW, so a RAW index
        assert!(epoch_tag(e) != u32::MAX && !is_preset_raw(epoch_tag(e)));
    }
    s.mark(
        "the tag/RAW dispatch (tagged ordinary portions in the slow finish)",
        from,
    );

    // v3: full ordinary groups opened in one epoch and finished in a later one (the old-
    // epoch slow path): 30 rows of byte 1 all before an epoch boundary b, finished at the
    // next arrival after b; and 29 before b with 1 after (a multi-epoch ordinary group)
    let from = s.cases;
    let ks: &[usize] = if quick { &[1] } else { &[1, 2, 3, 14] };
    for &k in ks {
        let b = EPOCH * k;
        if b + 400 > MAX_N {
            continue;
        }
        for (label, ones) in [
            ("all before, next arrival after", {
                let mut o: Vec<usize> = (0..30).map(|t| b - 90 + 3 * t).collect();
                o.push(b + 50);
                o
            }),
            ("29 before, 1 after", {
                let mut o: Vec<usize> = (0..29).map(|t| b - 87 + 3 * t).collect();
                o.push(b + 3);
                o
            }),
            ("all before, finished at the end of a later epoch", {
                (0..30).map(|t| b - 90 + 3 * t).collect::<Vec<usize>>()
            }),
        ] {
            let st = s.case(
                &format!("later finish k {k}: {label}"),
                &placed(b + 300, &ones, 0),
                0,
            );
            assert!(st.slow_ordinary_old_epoch >= 1, "{label}");
        }
    }
    s.mark(
        "ordinary groups finished in a later epoch than opened",
        from,
    );


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
    println!("ctor2c-synthetic: malformed counts: {rejected} rejected, {accepted} accepted (equal to ll's own after the change)");
    s.mark("malformed counts and sizes", from);

    let st = &s.st;
    let shapes: usize = (0..30)
        .map(|j| (1..=ROWS - j).filter(|&k| st.cov_portion[j][k] > 0).count())
        .sum();
    println!(
        "ctor2c-synthetic coverage: finished fast {}, ordinary {}, wide {}, crossing {}, forced {}; multi-epoch portions {}; epoch searches {}; runs split at an epoch {}; portion (first slot, length) shapes {shapes}; max ordinary span {}, min wide span {}; epochs {}",
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
        "ctor2c-synthetic: {} cases, {} array comparisons, {} walk outputs, {} restorations: every check exact ({})",
        s.cases,
        s.ck.arrays,
        s.ck.walks,
        s.ck.restorations,
        if quick { "quick" } else { "full" }
    );
    println!("ctor2c-synthetic counters: {{{}}}", st.json());
}

/// `ctor2c-check`, `ctor2c-adversarial`, `ctor2c-synthetic`, `ctor2c-count`
pub fn main(mode: &str, args: &[String]) {
    match mode {
        // S62-F01: the counted build alone, in the `--only c` process's order and arenas
        // (one Ctor2c, one pair of arrays, the blocks in order), so the paths that depend on
        // arena history (regrowth, reserves) see the same history as the profile; one jsonl
        // record per file on stdout, with the output digest of `--only c`
        "count" if args.first().map(String::as_str) == Some("--counted") => {
            let mut c2c = Ctor2c::default();
            let (mut r, mut w) = (Vec::new(), Vec::new());
            for file in &args[1..] {
                let data = std::fs::read(file).unwrap();
                let bs = blocks(&data);
                let mut h = 0xcbf2_9ce4_8422_2325u64;
                c2c.st = Stats2c::default();
                for (ll, counts, _, _) in &bs {
                    assert!(c2c.build::<true>(ll, counts, 0, &mut r, &mut w));
                    for &b in bytes(&r).iter().chain(bytes(&w)) {
                        h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
                    }
                }
                let name = std::path::Path::new(file)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                println!(
                    "{{\"input\":{name:?},\"permille\":0,\"extra_starts\":false,\"sequential\":true,\"blocks_built\":{},\"fnv1a\":\"{h:016x}\",{}}}",
                    bs.len(),
                    c2c.st.json()
                );
            }
        }
        // the ledger's cross-check: each block of each file built once by v2's and once by
        // the combined uncounted build, their outputs compared; run under callgrind for an
        // exact count of the instructions each `build::<false>` executes (untimed)
        "count" if args.first().map(String::as_str) == Some("--only") => {
            // one builder per process (§9.4): the same decode, the same arenas, and only
            // the named builder's calls
            let which = args[1].as_str();
            let (mut v2, mut c2s) = (super::ctor2::Ctor2::default(), Ctor2c::default());
            let (mut r, mut w) = (Vec::new(), Vec::new());
            for file in &args[2..] {
                let data = std::fs::read(file).unwrap();
                let bs = blocks(&data);
                let mut h = 0xcbf2_9ce4_8422_2325u64;
                for (ll, counts, _, _) in &bs {
                    match which {
                        "v2" => assert!(v2.build::<false>(ll, counts, 0, &mut r, &mut w)),
                        "c" => assert!(c2s.build::<false>(ll, counts, 0, &mut r, &mut w)),
                        x => panic!("--only {x}: v2 or c"),
                    }
                    for &b in bytes(&r).iter().chain(bytes(&w)) {
                        h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
                    }
                }
                println!(
                    "ctor2c-count --only {which}: {file}: {} blocks, output fnv1a {h:016x}",
                    bs.len()
                );
            }
        }
        "count" => {
            let (mut v2, mut c2s) = (super::ctor2::Ctor2::default(), Ctor2c::default());
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
                    "ctor2c-count: {file}: {} blocks built by both, identical",
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
            let mut total = Stats2c::default();
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
        m => panic!("unknown mode ctor2c-{m}"),
    }
}
