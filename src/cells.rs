//! Rauzy–Veech cells inversion, ported from `examples/rauzy.rs`.
//! Admission is private to this type; Segmented and cooperative routes are unchanged.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::bwt::{InverseBwt, Segmented};

/// Calling-thread default cells kernel, also selected with `decode_with::<bwt::Cells>`.
pub type Cells = CellsKernel<false>;
/// Test/fuzz route: open admission (including all-singleton columns), budget n.
#[doc(hidden)]
pub type ForcedCells = CellsKernel<true>;

/// Route counts for evidence; local to a kernel, with no shared instrumentation.
#[derive(Clone, Copy, Debug, Default)]
pub struct Counts {
    pub admitted: usize,
    pub density: usize,
    pub sparse: usize,
    pub floor: usize,
    pub budget: usize,
    pub invalid: usize,
    pub short_cycles: usize,
    pub cells: usize,
}

/// Implementation type; use `Cells` in decoding, `ForcedCells` only in tests/fuzzing.
#[derive(Default)]
pub struct CellsKernel<const FORCED: bool> {
    state: Box<State>,
}
#[derive(Default)]
struct State {
    fallback: Segmented,
    kernel: Kernel,
    counts: Counts,
    /// the last successful run-route inversion was Cells' own (the ledger's route)
    took: bool,
    /// research only (`dose`): the memory dose streamed per admitted block
    #[cfg(feature = "dose")]
    pad: crate::dose::Pad,
}
#[derive(Default)]
struct Kernel {
    buf: Vec<u8>,
    lay: Layout,
    rule_end: usize,
    letters: usize,
    /// which first stage and induction run: 0 `slots` and `induce_cells`, `LAZY` the lazy
    /// letters' (`cells_lazy.rs`), or (research only, `memory-lever`) `STAGED_RULES`
    variant: u8,
    /// the lazy letters' state
    lz: Lazy,
    /// (test only) how many times the last staged induction sent rules on; whether it is
    /// to panic right after its first send (the fence's unwinding exit)
    #[cfg(all(test, feature = "memory-lever"))]
    rule_sends: usize,
    #[cfg(all(test, feature = "memory-lever"))]
    panic_after_send: bool,
    /// this call laid the bottom slots out in its output buffer (their canary is there)
    bottom_laid: bool,
    /// (test only) the writer kept the rules' first places in the dead top slots
    #[cfg(test)]
    pos_in_tops: bool,
    /// (test only) how many of the lazy induction's hinted slots were then read at the place
    /// they were hinted for
    #[cfg(test)]
    hints_met: usize,
    pos: Vec<u32>,
    #[cfg(test)]
    writer_reach: WriterReach,
    #[cfg(test)]
    materialization: Option<Vec<Materialization>>,
    /// (test only) room for `budget` fresh slots after each order's letters, the layout the
    /// original induction (`induce_cells_original`) needs, for comparisons against it
    #[cfg(test)]
    fresh_room: bool,
    /// (test only) bottom slots of `SLOT` bytes holding every field, as the top slots do:
    /// the layout the original induction (`induce_cells_original`) reads
    #[cfg(test)]
    wide_bottom: bool,
}
#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
struct Materialization {
    state: [u64; 16],
    payload: Vec<u8>,
    slots: Vec<u8>,
}
#[cfg(test)]
include!("cells_original_test.rs");
// Cells with lazy letters, the workers' kernel on many threads
include!("cells_lazy.rs");
// (research only: the memory-lever experiment's staged kernel)
#[cfg(feature = "memory-lever")]
include!("cells_lever.rs");

/// Per-call reach, compiled out of production. Counts the actual writer branches,
/// not column lengths or inferred grammar properties.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
struct WriterReach {
    period: usize,
    seeds: usize,
    descending: usize,
    propagated: usize,
    expanded: usize,
    long_copies: usize,
    spine_recopies: usize,
    repetitions: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Refusal {
    Density,
    Sparse,
    Floor,
    Budget,
    Invalid,
}

impl<const F: bool> CellsKernel<F> {
    pub fn counts(&self) -> Counts {
        self.state.counts
    }
    fn attempt(&mut self, recs: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let budget = if F { n } else { n / 25 * 4 + n % 25 * 4 / 25 };
        let result = self.state.kernel.try_invert(recs, n, p, budget, F, out);
        let counts = &mut self.state.counts;
        match result {
            Ok((cells, period)) => {
                counts.admitted += 1;
                counts.cells += cells;
                counts.short_cycles += usize::from(period < n);
                true
            }
            Err(reason) => {
                match reason {
                    Refusal::Density => counts.density += 1,
                    Refusal::Sparse => counts.sparse += 1,
                    Refusal::Floor => counts.floor += 1,
                    Refusal::Budget => counts.budget += 1,
                    Refusal::Invalid => counts.invalid += 1,
                }
                false
            }
        }
    }
}

impl<const F: bool> InverseBwt for CellsKernel<F> {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        if F {
            let mut recs = Vec::new();
            for &c in ll {
                if let Some(last) = recs.last_mut() {
                    if *last >> 8 == c as u16 && *last & 255 < 127 {
                        *last += 1;
                        continue;
                    }
                }
                recs.push((c as u16) << 8 | 1);
            }
            if self.attempt(&recs, ll.len(), p, out) {
                return;
            }
        }
        self.state.fallback.invert(ll, counts, p, out);
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        // Forced fuzzing uses invert(), even on columns outside takes_runs().
        if F {
            return false;
        }
        self.state.fallback.invert_in_place(ll, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    #[cfg(any(test, feature = "trace"))]
    fn took_cells(&self) -> bool {
        self.state.took
    }
    fn invert_runs(
        &mut self,
        r: &crate::runs::Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: crate::Kernels,
    ) -> bool {
        self.state.took = self.attempt(r.records(), n, p, out);
        if self.state.took {
            #[cfg(feature = "dose")]
            self.state.pad.apply();
            return true;
        }
        // Exactly the shipped body, including its tiling failure. Admission refusal itself
        // never escapes as false to decode_block's front-invariant assertion.
        self.state.fallback.invert_runs(r, n, p, out, k)
    }
    #[cfg(any(test, feature = "trace"))]
    fn buffers(&self) -> Vec<crate::trace::Buffer> {
        let mut v = self.state.fallback.buffers();
        v.push(crate::trace::Buffer::of(
            "cells.buf",
            &self.state.kernel.buf,
        ));
        v.push(crate::trace::Buffer::of(
            "cells.pos",
            &self.state.kernel.pos,
        ));
        v
    }
}
/// A letter's place in the top order, for the induction by cells: 20 bytes. The two orders
/// are two regions of slots, the top one in the kernel's buffer and the bottom one in the
/// block's output buffer, dead until the writer; a letter has one slot in each. The orders
/// are only ever read from their ends, so a slot is read once, when the induction's front
/// reaches it.
///
/// | bytes | |
/// |---|---|
/// | 0..4 | the slot before it in its order (an offset in the buffer) |
/// | 4..8 | the letter's length |
/// | 8..12 | its word's length |
/// | 12..16 | its word: an offset in the buffer, of bytes if the length is at most SMALL, else of a rule |
/// | 16..20 | the letter's slot in the bottom order (an offset in the output buffer) |
///
/// A bottom slot is 8 bytes (`BSLOT`): the slot before it, and the letter's top slot. The
/// letter's length and word are read through the second, from its top slot: a letter's
/// fields are read once, on the side the fronts reach first, and the side reached second is
/// overwritten unread by the letter's own first cell. So when the bottom front reaches a
/// letter first, its top slot still holds it; when the top front does, the bottom copy was
/// never needed (MONODROMY, 2026-11-15, rule R4 and layout A8).
///
/// Every field is a 4-byte offset or length: the layout refuses a buffer whose total would
/// reach `u32::MAX` (`slots`), and Cells then declines the block.
const SLOT: usize = 20;
/// A bottom slot: the slot before it in the bottom order, and the letter's top slot
const BSLOT: usize = 8;
/// How many slots on from an order's last letter the induction by cells fetches for
const FAR: usize = 12;
/// How many slots on from the bottom order's last letter it fetches the word for, read
/// from the top slot fetched `FAR - NEAR` steps before
const NEAR: usize = 6;
/// The buffer's first bytes, unused: a read `FAR` slots before an order's first stays inside
const PAD: usize = 4096;
/// The output buffer's first bytes before the bottom slots, when they lie there: a read
/// `FAR` bottom slots before the bottom order's first stays inside
const BPAD: usize = 128;
/// The mark, in a word's length: the word holds the walk's row
const MARK: u32 = 1 << 31;
/// Where the block begins in the cycle's word, from the walk's row's place
const SHIFT: usize = 1;

/// The regions of the induction by cells' one buffer, as offsets in it.
#[derive(Clone, Copy, Default)]
struct Layout {
    /// the top order's slots
    tops: usize,
    /// the bottom order's slots: an offset in the block's output buffer (`bout`), else in
    /// this one
    bots: usize,
    /// the bottom order's slots lie in the output buffer, which nothing reads or writes
    /// until the writer, and which the writer then overwrites: always, but for the test
    /// layouts that compare against the original induction (`fresh_room`, `wide_bottom`)
    bout: bool,
    /// the words kept as bytes: 16 bytes of slack, the 256 single bytes, 16 more, the rest
    arena: usize,
    /// the words kept as rules, 16 bytes each
    rules: usize,
    ends: [usize; 4],
}

#[inline(always)]
unsafe fn rd64(base: *const u8, at: usize) -> u64 {
    // SAFETY: caller supplies an initialized in-bounds integer field; alignment is not required.
    unsafe { (base.add(at) as *const u64).read_unaligned() }
}

#[inline(always)]
unsafe fn rd32(base: *const u8, at: usize) -> u32 {
    // SAFETY: caller supplies an initialized in-bounds integer field; alignment is not required.
    unsafe { (base.add(at) as *const u32).read_unaligned() }
}

#[inline(always)]
unsafe fn wr64(base: *mut u8, at: usize, v: u64) {
    // SAFETY: caller supplies an initialized in-bounds integer field; alignment is not required.
    unsafe { (base.add(at) as *mut u64).write_unaligned(v) }
}

#[inline(always)]
unsafe fn wr32(base: *mut u8, at: usize, v: u32) {
    // SAFETY: caller supplies an initialized in-bounds integer field; alignment is not required.
    unsafe { (base.add(at) as *mut u32).write_unaligned(v) }
}

/// A slot's bytes 16..: the letter's slot in the other order (4 bytes)
#[inline(always)]
unsafe fn rd_other(base: *const u8, slot: usize) -> usize {
    // SAFETY: caller supplies an initialized in-bounds slot; alignment is not required.
    unsafe { rd32(base, slot + 16) as usize }
}

/// Sets a slot's bytes 16..: the letter's slot in the other order, `other`, an offset in
/// the buffer below `u32::MAX` (the layout's total is)
#[inline(always)]
unsafe fn wr_other(base: *mut u8, slot: usize, other: usize) {
    debug_assert!(other < u32::MAX as usize);
    // SAFETY: caller supplies an in-bounds slot; alignment is not required.
    unsafe { wr32(base, slot + 16, other as u32) }
}

/// A bottom slot: (the slot before it in the bottom order, the letter's top slot)
#[inline(always)]
unsafe fn rd_bottom(base: *const u8, slot: usize) -> (usize, usize) {
    // SAFETY: caller supplies an initialized in-bounds bottom slot; alignment is not required.
    let v = unsafe { rd64(base, slot) };
    (v as u32 as usize, (v >> 32) as usize)
}

const SMALL: u32 = 16;
const BYTES: u32 = 16;
const NIL: u32 = u32::MAX;
const GAP: usize = if cfg!(any(miri, debug_assertions)) {
    64
} else {
    0
};

#[inline(always)]
fn prefetch(p: *const u8) {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    // SAFETY: prefetch is only a non-faulting hint; no pointed-to value is accessed.
    unsafe {
        core::arch::x86_64::_mm_prefetch(p.cast(), core::arch::x86_64::_MM_HINT_T0);
    }
    let _ = p;
}

#[inline(always)]
unsafe fn copy16(dst: *mut u8, src: *const u8) {
    // SAFETY: caller provides 16 initialized readable/writable bytes. Read precedes write,
    // so overlap is allowed, and neither pointer needs alignment.
    unsafe {
        dst.cast::<[u8; 16]>()
            .write_unaligned(src.cast::<[u8; 16]>().read_unaligned());
    }
}

#[inline(always)]
unsafe fn copy_small(dst: *mut u8, src: *const u8, len: usize) {
    // SAFETY: caller provides len initialized bytes in distinct allocations (arena/output).
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, len);
    }
}

#[inline(always)]
unsafe fn copy_long(dst: *mut u8, src: *const u8, len: usize) {
    // SAFETY: caller provides len initialized bytes of output on each side. copy permits
    // overlap; the grammar's first-occurrence order ensures the source is complete.
    unsafe {
        core::ptr::copy(src, dst, len);
    }
}

impl Kernel {
    /// A bottom slot's size: `BSLOT`, or (test only) `SLOT` for the original induction
    #[inline(always)]
    fn bslot(&self) -> usize {
        #[cfg(test)]
        if self.wide_bottom {
            return SLOT;
        }
        BSLOT
    }

    #[cfg(test)]
    fn record_materialization(&mut self, state: [u64; 16]) {
        if let Some(log) = &mut self.materialization {
            let cw = state[4];
            assert_eq!(
                cw as u32 <= SMALL,
                (cw >> 32) < self.lay.rules as u64,
                "selected representation"
            );
            let size = if cw as u32 <= SMALL {
                cw as u32 as usize
            } else {
                16
            };
            let at = (cw >> 32) as usize;
            // a rule is its two factors (state 2 and 3): a staged rule is not in the buffer
            // yet (`memory-lever`), so no rule is read from it here; the comparison tests
            // compare the rules' region once the induction is over
            let payload = if cw as u32 <= SMALL {
                self.buf[at..at + size].to_vec()
            } else {
                [state[2].to_ne_bytes(), state[3].to_ne_bytes()].concat()
            };
            log.push(Materialization {
                state,
                payload,
                slots: Vec::new(),
            });
        }
    }

    /// The cell's (length, word) as its top slot holds them, after checking that the two
    /// slots name each other (and, in the wide layout, that the bottom slot holds the same)
    #[cfg(test)]
    fn record_selected_slots(&mut self, ts: usize, bs: usize) {
        let bb = self.buf.as_ptr();
        self.record_selected_slots_in(ts, bs, bb);
    }

    /// The same, the bottom slot `bs` read from `bb`'s buffer (the output's, if it lies there)
    #[cfg(test)]
    fn record_selected_slots_in(&mut self, ts: usize, bs: usize, bb: *const u8) {
        let wide = self.bslot() == SLOT;
        if let Some(log) = &mut self.materialization {
            let read = |at: usize| u32::from_ne_bytes(self.buf[at..at + 4].try_into().unwrap());
            // SAFETY: `bs` is a bottom slot the induction just wrote, in the bottom region
            let bread = |at: usize| unsafe { rd32(bb, at) };
            let fields = &self.buf[ts + 4..ts + 16];
            if wide {
                assert_eq!(&self.buf[bs + 4..bs + 16], fields);
                assert_eq!(read(bs + 16) as usize, ts);
            } else {
                assert_eq!(bread(bs + 4) as usize, ts);
            }
            assert_eq!(read(ts + 16) as usize, bs);
            log.last_mut().unwrap().slots.extend_from_slice(fields);
        }
    }

    /// (test only) each rule's first place as the last writer left it, wherever it kept them
    #[cfg(test)]
    fn positions(&self) -> Vec<u32> {
        let nr = (self.rule_end - self.lay.rules) / 16;
        if self.pos_in_tops {
            let skip = self.buf[self.lay.tops..].as_ptr().align_offset(4);
            let at = |j: usize| self.lay.tops + skip + 4 * j;
            (0..nr)
                .map(|j| u32::from_ne_bytes(self.buf[at(j)..at(j) + 4].try_into().unwrap()))
                .collect()
        } else {
            self.pos[..nr].to_vec()
        }
    }

    /// The canaries after each region; the bottom region's, when it lies in the output
    /// buffer, only while `out` is given (the writer overwrites it)
    fn check_canaries(&self, out: Option<&[u8]>) {
        #[cfg(any(miri, debug_assertions))]
        if !self.buf.is_empty() {
            for (i, end) in self.lay.ends.into_iter().enumerate() {
                let region = match (i == 1 && self.lay.bout, out) {
                    (false, _) => &self.buf[..],
                    (true, Some(out)) if self.bottom_laid => out,
                    (true, _) => continue,
                };
                assert!(
                    region[end..end + GAP].iter().all(|&x| x == 0xA5),
                    "cells canary"
                );
            }
            // (the lazy kernel's bottom slots' pool, in the output buffer)
            if let (Some(out), true) = (
                out,
                self.bottom_laid && matches!(self.variant, LAZY | LAZY_HINT),
            ) {
                let end = self.lz.bend;
                assert!(
                    out[end..end + GAP].iter().all(|&x| x == 0xA5),
                    "cells canary"
                );
            }
        }
        let _ = out;
    }
    // Safe boundary: validates record tiling and primary row before raw memory access.
    // A refusal before the slots are laid out (every gate but the budget and a malformed
    // word) leaves out byte-for-byte untouched; after it, out holds the bottom slots, and
    // every fallback rewrites it whole (it clears it first).
    fn try_invert(
        &mut self,
        recs: &[u16],
        n: usize,
        p: u32,
        budget: usize,
        forced: bool,
        out: &mut Vec<u8>,
    ) -> Result<(usize, usize), Refusal> {
        // Bound all sizing arithmetic, including callers in unit tests.
        if budget > n || n >= 1 << 24 || recs.len() > n {
            return Err(Refusal::Invalid);
        }
        // the kernel's first stage and induction: the shipped ones, or the lazy letters'
        let result = self
            .first_stage(n, recs, p, budget, forced, out)
            .and_then(|()| {
                // (research only, `phase-counters`: the induction's scope)
                #[cfg(feature = "phase-counters")]
                let _scope = crate::phase::Scope::new(crate::phase::Phase::Induction);
                self.induce_variant(budget, out)
            });
        self.check_canaries(Some(out));
        let (word, len, cells) = result?;
        let period = (len & !MARK) as usize;
        if len & MARK == 0 || period == 0 || period > n {
            return Err(Refusal::Invalid);
        }
        {
            // (research only, `phase-counters`: the writer's scope)
            #[cfg(feature = "phase-counters")]
            let _scope = crate::phase::Scope::new(crate::phase::Phase::Writer);
            self.write_cells(word, len, n, out);
        }
        self.check_canaries(None);
        Ok((cells, period))
    }
    /// The cells kernel's first stage, from the front's run records (`byte << 8 | length`,
    /// a run longer than 127 rows cut into several: `runs::front`'s format): the letters as
    /// slots, both orders, in the one buffer the induction by cells works in. The top order
    /// is the runs as they lie, the bottom order is sorted stably by byte, so that the
    /// original letters of each order lie in memory in the order the induction reads them.
    /// Two passes over the records; the sort is the first one's counting.
    ///
    /// A record that goes on with the byte of the one before it is the same run. Row `p`,
    /// the walk's, is cut out as a letter of its own. Its word is marked, and the mark rides
    /// in the length of every word made with it: such a word is kept as a rule whatever its
    /// length, so the row's place in the cycle's word is read off the rules afterwards, and
    /// the induction follows no point.
    ///
    /// Regions are bounded by the records and cell budget; one counter limits growth
    /// of both slot regions, the byte arena and the rules.
    #[inline(never)]
    fn slots(
        &mut self,
        n: usize,
        recs: &[u16],
        p: u32,
        budget: usize,
        forced: bool,
        out: &mut Vec<u8>,
    ) -> Result<(), Refusal> {
        self.bottom_laid = false;
        if n == 0 || n >= 1 << 24 || p as usize >= n {
            return Err(Refusal::Invalid);
        }
        // This first gate reads only the record count, before validation or allocation.
        if !forced && recs.len() * 10 >= n {
            return Err(Refusal::Density);
        }
        // The second reads it too: a column of very long runs (fewer than one record per 64
        // rows: periodic or one-byte blocks) keeps Segmented, whose walk is already cheap
        // there and which Cells loses to. Admitted Wikidata blocks lie at r/n 0.059..0.094,
        // the synthetic pattern's at 0.0079 (Optimiser 98b553c3, the 64r < n screen).
        if !forced && recs.len() * 64 < n {
            return Err(Refusal::Sparse);
        }
        // a byte's letters; the record that holds the walk's row, and its run's first row
        let mut cnt = [0u32; 257];
        let (mut prev, mut row, mut start) = (256usize, 0u32, 0u32);
        let (mut at, mut at_start) = (0usize, 0u32);
        let mut empty = false;
        for (i, &rec) in recs.iter().enumerate() {
            let (c, len) = ((rec >> 8) as usize, (rec & 0xFF) as u32);
            if c != prev {
                cnt[c + 1] += 1;
                start = row;
            }
            prev = c;
            if p.wrapping_sub(row) < len {
                (at, at_start) = (i, start);
            }
            empty |= len == 0 || len > 127;
            row = row.checked_add(len).ok_or(Refusal::Invalid)?;
        }
        if row as usize != n || empty {
            return Err(Refusal::Invalid);
        }
        // that run: records ia..ib, `total` rows, the walk's row `off` rows in
        let cz = (recs[at] >> 8) as usize;
        let (mut ia, mut ib) = (at, at + 1);
        while ia > 0 && (recs[ia - 1] >> 8) as usize == cz {
            ia -= 1;
        }
        while ib < recs.len() && (recs[ib] >> 8) as usize == cz {
            ib += 1;
        }
        let total: u32 = recs[ia..ib].iter().map(|&x| (x & 0xFF) as u32).sum();
        let off = p - at_start;
        cnt[cz + 1] += (off > 0) as u32 + (total - off > 1) as u32;
        let letters = cnt.iter().map(|&v| v as usize).sum::<usize>();
        // Each order's region holds its letters' slots and nothing more: the induction takes
        // no fresh slot, since a cell that needs one takes the slot the front just read and
        // let go (`induce_cells`). Before, a region also had room for a fresh slot a cell.
        #[cfg(test)]
        let room = letters + if self.fresh_room { budget } else { 0 };
        #[cfg(not(test))]
        let room = letters;
        // the bottom order's slots are `BSLOT` bytes (`SLOT` in the original's wide layout);
        // a test kernel with `fresh_room` keeps room for wide ones, so that the arena and the
        // rules lie at the same offsets for the original and the candidate
        let bslot = self.bslot();
        let wide = bslot == SLOT;
        let cap = room * SLOT;
        #[cfg(test)]
        let bcap = room * if self.fresh_room { SLOT } else { bslot };
        #[cfg(not(test))]
        let bcap = room * bslot;
        // the bottom slots lie in the output buffer, but in the original's layouts
        #[cfg(test)]
        let bout = !(self.fresh_room || wide);
        #[cfg(not(test))]
        let bout = true;
        let tops = PAD;
        let (bots, arena) = if bout {
            (BPAD, tops + cap + GAP)
        } else {
            (tops + cap + GAP, tops + cap + GAP + bcap + GAP)
        };
        let rules = arena + BYTES as usize + 256 + 16 + 16 * budget + 32 + GAP;
        // (research only, `memory-lever`: the staged kernel's rules start at a multiple of
        // 16, so that every rule's offset is one, `Stage`)
        #[cfg(feature = "memory-lever")]
        let rules = if self.variant == STAGED_RULES {
            (rules + 15) & !15
        } else {
            rules
        };
        let allocation = rules + 16 * budget + 16 + GAP;
        // (every offset a slot holds is 4 bytes, the other order's included: a layout whose
        // total would reach u32::MAX is refused here, and Cells declines the block)
        if allocation >= u32::MAX as usize {
            return Err(Refusal::Invalid);
        }
        debug_assert!(
            allocation < u32::MAX as usize,
            "the layout fits 4-byte offsets"
        );
        if letters - 1 > budget {
            return Err(Refusal::Floor);
        }
        let lay = Layout {
            tops,
            bots,
            bout,
            arena,
            rules,
            ends: [tops + cap, bots + bcap, rules - GAP, allocation - GAP],
        };
        // Reuse the high-water allocation, but never geometrically over-allocate. Release
        // the old buffer before growing so both large allocations are not live at once.
        if self.buf.len() < allocation {
            self.buf = Vec::new();
            self.buf = vec![0; allocation];
        }
        if bout && out.len() < bots + bcap + GAP {
            out.resize(bots + bcap + GAP, 0);
        }
        self.bottom_laid = bout;
        self.lay = lay;
        self.letters = letters;
        for b in 0..256 {
            self.buf[arena + BYTES as usize + b] = b as u8;
        }
        for (i, end) in lay.ends.into_iter().enumerate() {
            if i == 1 && bout {
                out[end..end + GAP].fill(0xA5);
            } else {
                self.buf[end..end + GAP].fill(0xA5);
            }
        }
        // a byte's next bottom slot, as its offset in the buffer
        for k in 0..256 {
            cnt[k + 1] += cnt[k];
            cnt[k] = (lay.bots + cnt[k] as usize * bslot) as u32;
        }
        debug_assert_eq!(self.letters, cnt[256] as usize);
        let base = self.buf.as_mut_ptr();
        // the bottom slots' buffer
        let bb = if bout { out.as_mut_ptr() } else { base };
        let first = lay.arena as u64 + BYTES as u64;
        let mut t = PAD;
        // SAFETY: the letters number at most the records and two, and a record is a row at
        // least: their slots lie inside each order's region
        unsafe {
            macro_rules! letter {
                ($lam:expr, $c:expr) => {{
                    let (lam, c): (u64, usize) = ($lam as u64, $c);
                    let k = *cnt.get_unchecked(c) as usize;
                    *cnt.get_unchecked_mut(c) = (k + bslot) as u32;
                    let lw = 1 | (first + c as u64) << 32;
                    // (the slot before each order's first is set after the pass)
                    wr64(base, t, t.wrapping_sub(SLOT) as u32 as u64 | lam << 32);
                    wr64(base, t + 8, lw);
                    wr_other(base, t, k);
                    if wide {
                        wr64(base, k, k.wrapping_sub(SLOT) as u32 as u64 | lam << 32);
                        wr64(base, k + 8, lw);
                        wr_other(base, k, t);
                    } else {
                        // the bottom slot: the slot before it, and the letter's top slot
                        wr64(
                            bb,
                            k,
                            k.wrapping_sub(BSLOT) as u32 as u64 | (t as u64) << 32,
                        );
                    }
                    t += SLOT;
                }};
            }
            let nrec = recs.len();
            let mut i = 0;
            while i < nrec {
                if i == ia {
                    // the walk's row's run: what is before the row, the row, what is after
                    if off > 0 {
                        letter!(off, cz);
                    }
                    let tz = t;
                    letter!(1u32, cz);
                    wr32(base, tz + 8, 1 | MARK);
                    if wide {
                        wr32(base, rd_other(base, tz) + 8, 1 | MARK);
                    }
                    if total - off > 1 {
                        letter!(total - off - 1, cz);
                    }
                    i = ib;
                    continue;
                }
                let rec = *recs.get_unchecked(i);
                let c = (rec >> 8) as usize;
                let mut lam = (rec & 0xFF) as u32;
                i += 1;
                while i < nrec && (*recs.get_unchecked(i) >> 8) as usize == c {
                    lam += (*recs.get_unchecked(i) & 0xFF) as u32;
                    i += 1;
                }
                letter!(lam, c);
            }
            // (the slot before an order's first is never followed: one letter is marked, it
            // is never dropped, so the induction returns before it could leave an order;
            // these two only keep the value inside the buffer)
            wr32(base, PAD, PAD as u32);
            wr32(bb, lay.bots, lay.bots as u32);
        }
        Ok(())
    }

    /// The induction by cells. The same steps as `induce`, kept another way, to the cycle of
    /// the walk's row: the closed letter whose word carries the mark.
    ///
    /// The last letter on top, A, and the last on the bottom, B, are held with what is left
    /// of each (ra, rb). A step is the cell at the end of both: min(ra, rb) points, word
    /// (A's)(B's). It is a letter of its own, written once and never changed: on top it
    /// lies inside B's place, on the bottom inside A's image, each time to the left of the
    /// cells already cut from the same letter. The first cell cut from a letter takes that
    /// letter's slot (so whatever pointed at the letter now points at its rightmost cell);
    /// each later one takes a slot promised to it, and the slot before the letter moves
    /// along with the promise. Nothing is shortened in memory and no letter is moved: a
    /// step writes two slots and one word, and reads one or two.
    ///
    /// The promised slot is not a fresh one: it is the slot the cell's other letter was
    /// read from at the front, in the same order. A letter promises a slot only when the
    /// other letter ends, and an ended letter's front slot is dead: it is the last of its
    /// order, so no slot before it links to it, and its slot in the other order has just
    /// taken the cell, which links elsewhere. Only its fields were in registers, read when
    /// it reached the front. So the orders' regions hold the letters' slots and no more,
    /// and the lines a fresh slot would first touch are never touched.
    ///
    /// A bottom slot holds only the slot before it and the letter's top slot (`BSLOT`): a
    /// letter read at the bottom front is read through its top slot, which holds it, since
    /// a letter's fields are read only on the side reached first, and the cells cut from it
    /// are written over its top slot only after it is read. A cell's length and word are
    /// written to its top slot only.
    ///
    /// Every offset is a byte offset in one of two buffers: a top slot, a word kept as bytes
    /// or a word kept as a rule in the kernel's; a bottom slot in the output buffer (`out`),
    /// which no one reads or writes until the writer overwrites it. A rule is its two factors
    /// as their slots held them: (length, word) twice.
    ///
    /// Returns (the cycle's word, its length with the mark, cells). `slots` has run on this
    /// kernel for the block, and nothing has changed since.
    #[inline(never)]
    fn induce_cells(
        &mut self,
        budget: usize,
        out: &mut Vec<u8>,
    ) -> Result<(u32, u32, usize), Refusal> {
        const M: u64 = 0xFFFF_FFFF;
        let rr = self.letters;
        let lay = self.lay;
        if rr == 0 || self.buf.len() < lay.rules + 16 || self.bslot() != BSLOT {
            return Err(Refusal::Invalid);
        }
        if lay.bout && out.len() < lay.ends[1] + GAP {
            return Err(Refusal::Invalid);
        }
        let base = self.buf.as_mut_ptr();
        // the bottom slots' buffer: the output's, which `slots` sized for them
        let bb = if lay.bout { out.as_mut_ptr() } else { base };
        // SAFETY: `slots` sized the words' regions for budget cells, and the counter is
        // checked before each cell; a slot's offset is a letter's slot, written by `slots`
        // inside its order's region (a promised slot is one of them, let go at the front);
        // a bottom slot's second field is a top slot, so a letter read through it is read
        // inside the top region; a word's offset is in the arena or the rules, with 16
        // bytes after it inside the buffer.
        unsafe {
            let mut top = lay.arena + BYTES as usize + 256 + 16;
            let mut rp = lay.rules;
            // each order's last letter's slot
            let (ft0, fb0) = (lay.tops + rr * SLOT, lay.bots + rr * BSLOT);
            // Count every cell, including equal-length steps which allocate no slot.
            let mut cells = 0usize;
            // the slot before each letter in its order; what is left of the letter; its
            // (length, word); the slot the next cell cut from it takes in the other order
            let (mut na, mut nb);
            let (mut ra, mut rb);
            let (mut aw, mut bw);
            let (mut bs, mut ts);
            let (mut sa, mut sb) = (ft0 - SLOT, fb0 - BSLOT);
            // the slots A and B were read from at the fronts: when one of them ends, its
            // slot is the one its partner's next cell is promised
            let (mut fa, mut fb);
            // the letter `FAR` slots on in an order, as memory has it now: its slot in
            // the other order is where its cells will be written, and its word is read then
            // (what is read there may be a slot not yet written: the address is only hinted)
            macro_rules! ahead {
                ($s:expr) => {{
                    let far = $s - FAR * SLOT;
                    prefetch(bb.wrapping_add(rd32(base, far + 16) as usize));
                    prefetch(base.wrapping_add(rd32(base, far + 12) as usize));
                }};
            }
            // the last top slot: a hinted read through a bottom slot goes no further
            let last_top = ft0 - SLOT;
            // The same on the bottom, whose letter's word is in its top slot: that slot is
            // fetched `FAR` slots on, and the word `NEAR` slots on, read from the top slot
            // fetched `FAR - NEAR` steps before. (A slot not yet written may name any
            // offset: it is kept to a top slot, so that the read stays inside the buffer.)
            macro_rules! ahead_bottom {
                ($s:expr) => {{
                    let far = $s - FAR * BSLOT;
                    prefetch(base.wrapping_add(rd32(bb, far + 4) as usize));
                    let near = (rd32(bb, $s - NEAR * BSLOT + 4) as usize).min(last_top);
                    prefetch(base.wrapping_add(rd32(base, near + 12) as usize));
                }};
            }
            'fresh: loop {
                // both letters whole (B's length and word through its top slot)
                na = rd32(base, sa) as usize;
                ra = rd32(base, sa + 4) as u64;
                aw = rd64(base, sa + 8);
                bs = rd_other(base, sa);
                (nb, ts) = rd_bottom(bb, sb);
                rb = rd32(base, ts + 4) as u64;
                bw = rd64(base, ts + 8);
                (fa, fb) = (sa, sb);

                ahead!(sa);
                ahead_bottom!(sb);
                if bs == sb {
                    // one letter, last in both orders: its points' cycles are closed
                    if aw as u32 & MARK != 0 {
                        // the words' regions' ends: unchecked above, and valgrind cannot
                        // see a write from one region into the next inside the one
                        // allocation (the slots' regions take no slot after `slots`)
                        if top + 32 > lay.ends[2] || rp + 16 > lay.ends[3] {
                            debug_assert!(false, "cells region exhausted");
                            return Err(Refusal::Invalid);
                        }
                        self.rule_end = rp;
                        return Ok(((aw >> 32) as u32, aw as u32, cells));
                    }

                    (sa, sb) = (na, nb);
                    continue 'fresh;
                }
                loop {
                    if cells == budget {
                        return Err(Refusal::Budget);
                    }
                    cells += 1;
                    // The cell word is A then B; materialise only its retained representation.
                    let (la, lb) = (aw & M, bw & M);
                    let len = la + lb;
                    // MARK remains in len: even a short marked word must be a rule.
                    let small = len <= SMALL as u64;
                    if small {
                        copy16(base.add(top), base.add((aw >> 32) as usize));
                        copy16(
                            base.add(top + la.min(16) as usize),
                            base.add((bw >> 32) as usize),
                        );
                    } else {
                        wr64(base, rp, aw);
                        wr64(base, rp + 8, bw);
                    }
                    let word = if small { top } else { rp };
                    top += if small { len as usize } else { 0 };
                    rp += if small { 0 } else { 16 };
                    let cw = len | (word as u64) << 32;
                    #[cfg(test)]
                    self.record_materialization([
                        ra,
                        rb,
                        aw,
                        bw,
                        cw,
                        top as u64,
                        rp as u64,
                        fa as u64,
                        fb as u64,
                        na as u64,
                        nb as u64,
                        bs as u64,
                        ts as u64,
                        sa as u64,
                        sb as u64,
                        cells as u64,
                    ]);
                    if ra == rb {
                        // both letters end: the cell is the last cut from each
                        wr32(base, ts + 4, ra as u32);
                        wr64(base, ts + 8, cw);
                        wr_other(base, ts, bs);
                        wr32(bb, bs + 4, ts as u32);
                        #[cfg(test)]
                        self.record_selected_slots_in(ts, bs, bb);
                        (sa, sb) = (na, nb);
                        continue 'fresh;
                    } else if ra > rb {
                        // B ends; A goes on, and its next cell is promised B's front slot
                        // (B's fields, its slot before included, are in registers)
                        // (fb is a bottom slot, ts a top one: in different buffers, so
                        // not compared)
                        debug_assert!(fb != bs && fb != nb);
                        wr32(base, ts + 4, rb as u32);
                        wr64(base, ts + 8, cw);
                        wr_other(base, ts, bs);
                        wr32(bb, fb, rd32(bb, bs));
                        wr64(bb, bs, fb as u64 | (ts as u64) << 32);
                        #[cfg(test)]
                        self.record_selected_slots_in(ts, bs, bb);
                        bs = fb;
                        ra -= rb;
                        // (read after the cell is written: it may be that cell)
                        let s = nb;
                        fb = s;
                        ahead_bottom!(s);
                        (nb, ts) = rd_bottom(bb, s);
                        rb = rd32(base, ts + 4) as u64;
                        bw = rd64(base, ts + 8);
                    } else {
                        // A ends; B goes on, and is promised A's front slot
                        debug_assert!(fa != ts && fa != na);
                        wr32(bb, bs + 4, ts as u32);
                        wr32(base, fa, rd32(base, ts));
                        wr32(base, ts, fa as u32);
                        wr32(base, ts + 4, ra as u32);
                        wr64(base, ts + 8, cw);
                        wr_other(base, ts, bs);
                        #[cfg(test)]
                        self.record_selected_slots_in(ts, bs, bb);
                        ts = fa;
                        rb -= ra;
                        let s = na;
                        fa = s;
                        ahead!(s);
                        na = rd32(base, s) as usize;
                        ra = rd32(base, s + 4) as u64;
                        aw = rd64(base, s + 8);
                        bs = rd_other(base, s);
                    }
                }
            }
        }
    }

    /// The block from the induction by cells' word (`word`, of marked length `len`), into
    /// `out`. The block begins `SHIFT` bytes after the walk's row's place in the word, and
    /// is written from there on. The marked rules are walked from the word down to the row
    /// three times: for the row's place; to write each short factor without the mark where
    /// it then lies and note where each long one lies; and, after `write`'s two passes
    /// (where each rule first lies, last rule to first; then its bytes, first to last), to
    /// copy each long factor that lies again later from its first place. If the cycle is
    /// shorter than the block, the block is its copies.
    ///
    /// `word` and `len` are what `induce_cells` returned for the block `slots` was given.
    #[inline(never)]
    fn write_cells(&mut self, word: u32, len: u32, n: usize, out: &mut Vec<u8>) {
        // (test only, `memory-lever`: the writer reads the rules, so no store that went past
        // the cache may still be waiting for its fence; `cells_lever.rs`'s model)
        #[cfg(all(test, feature = "memory-lever"))]
        assert!(
            !PENDING.with(|p| p.get()),
            "rules stored past the cache are read before a fence"
        );
        #[cfg(test)]
        {
            self.writer_reach = WriterReach::default();
        }
        let p = (len & !MARK) as usize;
        #[cfg(test)]
        {
            self.writer_reach.period = p;
        }
        // (kept at its length from block to block: only the slack is written here)
        out.resize(n + 32, 0);
        let dst = out.as_mut_ptr();
        let base = self.buf.as_mut_ptr();
        let ro = self.lay.rules;
        // SAFETY: a rule's factors are words written before it; a word kept as bytes has
        // 16 bytes at its offset; a position is below the period; `out` has 16 bytes past
        // the period
        unsafe {
            let rule = |w: usize| (rd64(base, w), rd64(base, w + 8));
            let nr = (self.rule_end - ro) / 16;
            // Each rule's first place, in the top slots: the induction is over, so they are
            // dead, and their lines were the induction's last. Only if they hold too few (a
            // block of more rules than five a letter, which the forced route can make) in a
            // vector of their own.
            // (a byte buffer has no alignment of its own: the first place starts at the top
            // slots' first 4-aligned byte)
            let skip = base.add(self.lay.tops).align_offset(4);
            let in_tops = skip < 4 && skip + 4 * nr <= self.lay.ends[0] - self.lay.tops;
            let pos = if in_tops {
                let pos = base.add(self.lay.tops + skip) as *mut u32;
                core::ptr::write_bytes(pos, 0xFF, nr);
                pos
            } else {
                if self.pos.capacity() < nr {
                    self.pos = Vec::new();
                    self.pos = vec![NIL; nr];
                } else {
                    self.pos.resize(nr, NIL);
                    self.pos.fill(NIL);
                }
                self.pos.as_mut_ptr()
            };
            #[cfg(test)]
            {
                self.pos_in_tops = in_tops;
            }
            // the row's place in the word
            let (mut w, mut j) = (word as usize, 0usize);
            while w >= ro {
                let (a, b) = rule(w);
                if a as u32 & MARK != 0 {
                    w = (a >> 32) as usize;
                } else {
                    j += a as u32 as usize;
                    w = (b >> 32) as usize;
                }
            }
            let start = (j + SHIFT) % p;
            // the marked rules, from the word down to the row
            let (mut w, mut q) = (word as usize, 0usize);
            while w >= ro {
                let (a, b) = rule(w);
                let la = (a as u32 & !MARK) as usize;
                let (plain, at, next) = if a as u32 & MARK != 0 {
                    (b, q + la, a)
                } else {
                    (a, q, b)
                };
                let d = (at + p - start) % p;
                let lx = plain as u32 as usize;
                if lx <= SMALL as usize {
                    copy_small(dst.add(d), base.add((plain >> 32) as usize), lx);
                } else {
                    let x = pos.add(((plain >> 32) as usize - ro) / 16);
                    *x = (*x).min(d as u32);
                    #[cfg(test)]
                    {
                        self.writer_reach.seeds += 1;
                    }
                }
                if a as u32 & MARK == 0 {
                    q += la;
                }
                w = (next >> 32) as usize;
            }
            *dst.add((q + p - start) % p) = *base.add(w);
            // where each rule first lies: a rule's factors lie no later than inside it
            for j in (0..nr).rev() {
                let q = *pos.add(j);
                if q == NIL {
                    continue;
                }
                let (a, b) = rule(ro + 16 * j);
                #[cfg(test)]
                {
                    self.writer_reach.descending += 1;
                }
                if a as u32 > SMALL {
                    let x = pos.add(((a >> 32) as usize - ro) / 16);
                    *x = (*x).min(q);
                    #[cfg(test)]
                    {
                        self.writer_reach.propagated += 1;
                    }
                }
                if b as u32 > SMALL {
                    let x = pos.add(((b >> 32) as usize - ro) / 16);
                    *x = (*x).min(q + a as u32);
                    #[cfg(test)]
                    {
                        self.writer_reach.propagated += 1;
                    }
                }
            }
            // its bytes: each factor's from the arena, or from where the factor first lies
            for j in 0..nr {
                let q = *pos.add(j);
                if q == NIL {
                    continue;
                }
                let (a, b) = rule(ro + 16 * j);
                let (ll, rr) = (a as u32 as usize, b as u32 as usize);
                #[cfg(test)]
                {
                    self.writer_reach.expanded += 1;
                }
                let d = dst.add(q as usize);
                if ll <= SMALL as usize {
                    copy_small(d, base.add((a >> 32) as usize), ll);
                } else {
                    let f = *pos.add(((a >> 32) as usize - ro) / 16);
                    if f != q {
                        copy_long(d, dst.add(f as usize), ll);
                        #[cfg(test)]
                        {
                            self.writer_reach.long_copies += 1;
                        }
                    }
                }
                let d = d.add(ll);
                if rr <= SMALL as usize {
                    copy_small(d, base.add((b >> 32) as usize), rr);
                } else {
                    let f = *pos.add(((b >> 32) as usize - ro) / 16);
                    if f != q + ll as u32 {
                        copy_long(d, dst.add(f as usize), rr);
                        #[cfg(test)]
                        {
                            self.writer_reach.long_copies += 1;
                        }
                    }
                }
            }
            // the marked rules again: a long factor that lies at a later place than its
            // first is copied from there (the first walk only noted where each lies; a
            // marked rule has no place of its own, so the passes above never copy for it).
            // The first place is the least and occurrences of one word are apart, so the
            // source is whole and does not reach the place written.
            let (mut w, mut q) = (word as usize, 0usize);
            while w >= ro {
                let (a, b) = rule(w);
                let la = (a as u32 & !MARK) as usize;
                let (plain, at, next) = if a as u32 & MARK != 0 {
                    (b, q + la, a)
                } else {
                    (a, q, b)
                };
                let d = (at + p - start) % p;
                let lx = plain as u32 as usize;
                if lx > SMALL as usize {
                    let f = *pos.add(((plain >> 32) as usize - ro) / 16) as usize;
                    if f != d {
                        copy_long(dst.add(d), dst.add(f), lx);
                        #[cfg(test)]
                        {
                            self.writer_reach.spine_recopies += 1;
                        }
                    }
                }
                if a as u32 & MARK == 0 {
                    q += la;
                }
                w = (next >> 32) as usize;
            }
        }
        // a cycle shorter than the block: the block is its copies
        let mut have = p.min(n);
        while have < n {
            let more = have.min(n - have);
            out.copy_within(..more, have);
            #[cfg(test)]
            {
                self.writer_reach.repetitions += 1;
            }
            have += more;
        }
        out.truncate(n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runs::Runs;

    fn oracle(ll: &[u8], p: u32) -> Vec<u8> {
        let mut counts = [0; 256];
        for &c in ll {
            counts[c as usize] += 1;
        }
        let mut out = Vec::new();
        crate::bwt::Plain::default().invert(ll, &counts, p, &mut out);
        out
    }
    fn check(kernel: &mut Kernel, ll: &[u8], p: u32) -> usize {
        let r = Runs::of_column(ll);
        let want = oracle(ll, p);
        // Complement first, so a stale/missed output write cannot pass on reuse.
        let mut out: Vec<_> = want.iter().map(|b| !b).collect();
        let (cells, _) = kernel
            .try_invert(r.records(), ll.len(), p, ll.len(), true, &mut out)
            .unwrap();
        assert_eq!(out, want, "column {ll:?}, row {p}");
        cells
    }

    #[test]
    fn cells_exhaustive_arbitrary_columns() {
        let mut kernel = Kernel::default();
        // the rules' first places: in the dead top slots, or (more rules than five a letter)
        // in a vector of their own; both are reached
        let mut kept = [0usize; 2];
        for n in 1..=10 {
            for bits in 0..1usize << n {
                let ll: Vec<_> = (0..n).map(|i| ((bits >> i) & 1) as u8).collect();
                for p in 0..n {
                    check(&mut kernel, &ll, p as u32);
                    kept[usize::from(kernel.pos_in_tops)] += 1;
                }
            }
        }
        // (a short column has fewer rules than five a letter; a long Euclid-like one more)
        for ll in [
            [vec![1], vec![0; 100]].concat(),
            [vec![1; 3], vec![0; 200]].concat(),
        ] {
            for p in [0, ll.len() / 2, ll.len() - 1] {
                check(&mut kernel, &ll, p as u32);
                kept[usize::from(kernel.pos_in_tops)] += 1;
            }
        }
        assert!(kept[0] > 0 && kept[1] > 0, "{kept:?}");
    }

    #[test]
    fn miri_cells_exact_fit_and_reuse() {
        let mut kernel = Kernel::default();
        for ll in [
            vec![1, 0],
            b"bbaaa".to_vec(),
            b"abababa".to_vec(),
            vec![0; 17],
            (0..65).map(|i| (i % 3) as u8).collect(),
        ] {
            for p in [0, ll.len() / 2, ll.len() - 1] {
                let cells = check(&mut kernel, &ll, p as u32);
                let r = Runs::of_column(&ll);
                let mut out = vec![0xCC; ll.len()];
                // The floor can be conservative on arbitrary multi-cycle columns.
                if kernel.letters - 1 <= cells {
                    assert!(kernel
                        .try_invert(r.records(), ll.len(), p as u32, cells, true, &mut out)
                        .is_ok());
                    assert_eq!(out, oracle(&ll, p as u32));
                    if cells > 0 {
                        // (a refusal on the budget comes after the bottom slots were laid
                        // in out: its bytes are the fallback's to rewrite)
                        assert!(kernel
                            .try_invert(r.records(), ll.len(), p as u32, cells - 1, true, &mut out)
                            .is_err());
                    }
                }
            }
        }
    }

    #[test]
    fn miri_cells_admission_and_invalid_records() {
        let mut kernel = Kernel::default();
        let mut out = vec![13, 37];
        let before = out.clone();
        let dense = Runs::of_column(&[0; 10]);
        assert_eq!(
            kernel.try_invert(dense.records(), 10, 0, 1, false, &mut out),
            Err(Refusal::Density)
        );
        assert!(kernel.buf.is_empty(), "density gate allocates nothing");
        let sparse = Runs::of_column(&[vec![0; 500], vec![1; 500]].concat());
        assert_eq!(
            kernel.try_invert(sparse.records(), 1000, 0, 1000, false, &mut out),
            Err(Refusal::Sparse)
        );
        assert!(kernel.buf.is_empty(), "sparse gate allocates nothing");
        let floor = Runs::of_column(&[0; 11]);
        assert_eq!(
            kernel.try_invert(floor.records(), 11, 1, 1, false, &mut out),
            Err(Refusal::Floor)
        );
        assert_eq!(out, before);
        // A cyclic two-run exchange with almost equal lengths needs many subtractions.
        let ll = [vec![1; 51], vec![0; 50]].concat();
        let r = Runs::of_column(&ll);
        // (a refusal on the budget leaves out holding the bottom slots: the fallback clears
        // and rewrites it; the other gates leave it as it was)
        let mut scratch = out.clone();
        assert_eq!(
            kernel.try_invert(r.records(), 101, 0, 16, false, &mut scratch),
            Err(Refusal::Budget)
        );
        assert_eq!(out, before);
        for (recs, n, p) in [
            (vec![], 0, 0),
            (vec![0], 1, 0),
            (vec![128], 128, 0),
            (vec![1], 2, 0),
            (vec![2], 1, 0),
            (vec![1], 1, 1),
        ] {
            assert_eq!(
                kernel.try_invert(&recs, n, p, n, true, &mut out),
                Err(Refusal::Invalid)
            );
            assert_eq!(out, before);
        }
        // Canary instrumentation really detects a crossing within the allocation.
        #[cfg(any(miri, debug_assertions))]
        {
            kernel.buf[kernel.lay.ends[0]] ^= 1;
            assert!(std::panic::catch_unwind(|| kernel.check_canaries(None)).is_err());
        }
    }

    // Researcher F01 witness: BWT of (a^9 b^9)^3 c. Sorting rotations here
    // independently constructs the column; primary 0/3 deliberately select
    // different rotations rather than the source word's primary.
    fn long_writer_column() -> Vec<u8> {
        let mut word = [vec![b'a'; 9], vec![b'b'; 9]].concat().repeat(3);
        word.push(b'c');
        let n = word.len();
        let mut starts: Vec<_> = (0..n).collect();
        starts.sort_by_key(|&s| (0..n).map(|i| word[(s + i) % n]).collect::<Vec<_>>());
        starts.iter().map(|&s| word[(s + n - 1) % n]).collect()
    }

    fn assert_long_passes(reach: WriterReach) {
        assert!(reach.seeds > 0, "{reach:?}");
        assert!(reach.descending > 0, "{reach:?}");
        assert!(reach.propagated > 0, "{reach:?}");
        assert!(reach.expanded > 0, "{reach:?}");
    }

    #[test]
    fn miri_cells_long_writer() {
        let column = long_writer_column();
        let mut kernel = Kernel::default();
        let mut out = Vec::new();
        for (ll, p, expected_cells, expected_period) in [
            ([vec![1; 16], vec![0; 15]].concat(), 16, 17, 31),
            (column.clone(), 0, 37, 55),
            (column, 3, 38, 55),
        ] {
            let want = oracle(&ll, p);
            out.clear();
            out.extend(want.iter().map(|b| !b));
            let r = Runs::of_column(&ll);
            let got = kernel.try_invert(r.records(), ll.len(), p, ll.len(), true, &mut out);
            assert_eq!(got, Ok((expected_cells, expected_period)));
            // This assertion must precede reach checks: removing a copy must be
            // caught by output parity, not merely by removing a counter increment.
            assert_eq!(out, want);
            let reach = kernel.writer_reach;
            eprintln!("long writer n={} p={p}: {reach:?}", ll.len());
            assert_long_passes(reach);
            if ll.len() == 55 && p == 0 {
                assert!(reach.spine_recopies > 0, "{reach:?}");
            }
            if p == 3 {
                assert!(reach.long_copies > 0, "{reach:?}");
            }
        }
        // Retain the same arena, positions and output allocation, then shrink.
        let cap = (kernel.buf.capacity(), kernel.pos.capacity(), out.capacity());
        let ll = b"bbaaa";
        let want = oracle(ll, 2);
        out.clear();
        out.extend(want.iter().map(|b| !b));
        assert!(kernel
            .try_invert(
                Runs::of_column(ll).records(),
                ll.len(),
                2,
                ll.len(),
                true,
                &mut out
            )
            .is_ok());
        assert_eq!(out, want);
        assert_eq!(
            (kernel.buf.capacity(), kernel.pos.capacity(), out.capacity()),
            cap
        );
        assert_eq!(kernel.writer_reach.expanded, 0, "reach resets per call");
    }

    #[test]
    fn miri_cells_long_writer_production_admission() {
        let ll: Vec<_> = long_writer_column()
            .into_iter()
            .flat_map(|b| [b; 8])
            .collect();
        let r = Runs::of_column(&ll);
        assert_eq!(
            (ll.len(), r.records().len(), ll.len() * 4 / 25),
            (440, 8, 70)
        );
        let mut cells = Cells::default();
        let want = oracle(&ll, 0);
        let mut out: Vec<_> = want.iter().map(|b| !b).collect();
        assert!(cells.invert_runs(&r, ll.len(), 0, &mut out, crate::Kernels::scalar()));
        assert_eq!(out, want);
        let counts = cells.counts();
        assert_eq!(
            (counts.admitted, counts.cells, counts.short_cycles),
            (1, 57, 1)
        );
        assert_eq!(
            (counts.density, counts.floor, counts.budget, counts.invalid),
            (0, 0, 0, 0)
        );
        assert_eq!(cells.state.kernel.letters, 7);
        let reach = cells.state.kernel.writer_reach;
        assert_eq!(reach.period, 55);
        eprintln!("production long writer n=440 p=0: {reach:?}");
        assert_long_passes(reach);
        assert!(
            reach.spine_recopies > 0 && reach.repetitions > 0,
            "{reach:?}"
        );
        // Successful smaller admitted block reuses all three allocations.
        let cap = (
            cells.state.kernel.buf.capacity(),
            cells.state.kernel.pos.capacity(),
            out.capacity(),
        );
        let small = vec![b'x'; 40];
        let r = Runs::of_column(&small);
        out.fill(!b'x');
        assert!(cells.invert_runs(&r, small.len(), 0, &mut out, crate::Kernels::scalar()));
        assert_eq!(out, small);
        assert_eq!(cells.counts().admitted, 2);
        assert_eq!(
            (
                cells.state.kernel.buf.capacity(),
                cells.state.kernel.pos.capacity(),
                out.capacity()
            ),
            cap
        );
        assert_eq!(cells.state.kernel.writer_reach.expanded, 0);
    }

    #[test]
    fn miri_cells_slots_mark_and_links() {
        let ll = [vec![1; 31], vec![0; 34]].concat();
        let r = Runs::of_column(&ll);
        let mut kernel = Kernel::default();
        let mut out = Vec::new();
        kernel
            .slots(ll.len(), r.records(), 17, ll.len(), true, &mut out)
            .unwrap();
        // the top slots lie in the kernel's buffer, the bottom ones in the output buffer
        assert!(kernel.lay.bout);
        let rd = |b: &[u8], at: usize| u32::from_ne_bytes(b[at..at + 4].try_into().unwrap());
        let (tb, bb) = (&kernel.buf[..], &out[..]);
        for top in [true, false] {
            let (start, other) = if top {
                (kernel.lay.tops, kernel.lay.bots)
            } else {
                (kernel.lay.bots, kernel.lay.tops)
            };
            let (mut marks, mut rows) = (0, 0);
            let (size, osize) = if top { (SLOT, BSLOT) } else { (BSLOT, SLOT) };
            let (own, theirs) = if top { (tb, bb) } else { (bb, tb) };
            for i in 0..kernel.letters {
                let at = start + i * size;
                // a bottom slot holds the slot before it and its top slot, which the letter's
                // length and word are read from
                let (partner, fields) = if top {
                    (rd(own, at + 16) as usize, at)
                } else {
                    (rd(own, at + 4) as usize, rd(own, at + 4) as usize)
                };
                if i > 0 {
                    assert_eq!(rd(own, at) as usize, at - size);
                }
                marks += usize::from(rd(tb, fields + 8) & MARK != 0);
                rows += rd(tb, fields + 4) as usize;
                assert!(partner >= other && partner < other + kernel.letters * osize);
                assert_eq!((partner - other) % osize, 0);
                let back = if top {
                    rd(theirs, partner + 4)
                } else {
                    rd(theirs, partner + 16)
                };
                assert_eq!(back as usize, at);
            }
            assert_eq!(marks, 1);
            assert_eq!(rows, ll.len());
        }
    }

    /// A cell that leaves a letter going on is promised the slot its other letter was read
    /// from at the front: both unequal branches take one, every slot the induction touches
    /// lies among the letters' (the regions end there, canaries after them in debug and
    /// Miri), the output is exact, and a refusal on the budget leaves `out` holding the
    /// bottom slots.
    #[test]
    fn miri_cells_promised_slots_exact_fit() {
        let mut kernel = Kernel::default();
        let (mut less, mut greater) = (0, 0);
        for ll in [
            b"bbaaa".to_vec(),
            b"abababa".to_vec(),
            [vec![1; 31], vec![0; 34]].concat(),
            (0..65).map(|i| (i % 3) as u8).collect(),
            long_writer_column(),
        ] {
            for p in [0, ll.len() / 2, ll.len() - 1] {
                kernel.materialization = Some(Vec::new());
                let cells = check(&mut kernel, &ll, p as u32);
                let lay = kernel.lay;
                assert_eq!(lay.ends[0], lay.tops + kernel.letters * SLOT);
                assert_eq!(lay.ends[1], lay.bots + kernel.letters * BSLOT);
                for m in kernel.materialization.take().unwrap() {
                    let (fa, fb) = (m.state[7] as usize, m.state[8] as usize);
                    assert!(fa >= lay.tops && fa + SLOT <= lay.ends[0]);
                    assert!(fb >= lay.bots && fb + BSLOT <= lay.ends[1]);
                    less += usize::from(m.state[0] < m.state[1]);
                    greater += usize::from(m.state[0] > m.state[1]);
                }
                if cells > 0 && kernel.letters - 1 < cells {
                    let r = Runs::of_column(&ll);
                    let mut out = vec![0xCC; ll.len()];
                    assert_eq!(
                        kernel.try_invert(
                            r.records(),
                            ll.len(),
                            p as u32,
                            cells - 1,
                            true,
                            &mut out
                        ),
                        Err(Refusal::Budget)
                    );
                    // out held the bottom slots (from `BPAD` on); the fallback rewrites it
                    assert!(out.len() >= BPAD + kernel.letters * BSLOT);
                }
            }
        }
        assert!(less > 0 && greater > 0, "{less} {greater}");
    }

    /// The 8-byte bottom slot: a letter read at the bottom front is read through its top
    /// slot, and the word's fetch-ahead reads a top slot through a bottom slot that may not
    /// be written yet. With every byte the induction did not write left stale (the canaries'
    /// byte, which names no offset inside the buffer), the reads stay inside the buffer (Miri),
    /// every cell's two slots name each other (`record_selected_slots`), the bottom region is
    /// the letters' 8-byte slots and no more, and the output is exact.
    #[test]
    fn miri_cells_bottom_slots_read_through_top() {
        let mut kernel = Kernel::default();
        let mut cells = 0;
        for ll in [
            vec![1, 0],
            b"bbaaa".to_vec(),
            b"abababa".to_vec(),
            [vec![1; 31], vec![0; 34]].concat(),
            (0..65).map(|i| (i % 3) as u8).collect(),
            long_writer_column(),
        ] {
            for p in [0, ll.len() / 2, ll.len() - 1] {
                kernel.buf.fill(0xA5);
                kernel.materialization = Some(Vec::new());
                check(&mut kernel, &ll, p as u32);
                let lay = kernel.lay;
                // the bottom slots in the output buffer, the arena right after the top ones
                assert!(lay.bout);
                assert_eq!(
                    (lay.bots, lay.ends[1]),
                    (BPAD, BPAD + kernel.letters * BSLOT)
                );
                assert_eq!(lay.arena, lay.ends[0] + GAP);
                cells += kernel.materialization.take().unwrap().len();
            }
        }
        assert!(cells > 0);
    }

    /// The short-lived state in dead regions: the bottom slots in the output buffer, from
    /// `BPAD` on, which the writer then overwrites, and the rules' first places in the top
    /// slots, which the induction has let go (or, with more rules than five a letter, in a
    /// vector of their own). Over stale bytes in both buffers, every read stays inside its
    /// buffer (Miri), the output is exact, and both places for the first places are reached.
    #[test]
    fn miri_cells_dead_regions() {
        let mut kernel = Kernel::default();
        let mut kept = [0usize; 2];
        for ll in [
            vec![1, 0],
            b"bbaaa".to_vec(),
            (0..65).map(|i| (i % 3) as u8).collect(),
            [vec![1], vec![0; 100]].concat(),
            long_writer_column(),
        ] {
            for p in [0, ll.len() - 1] {
                kernel.buf.fill(0x5A);
                check(&mut kernel, &ll, p as u32);
                let lay = kernel.lay;
                assert!(lay.bout && kernel.bottom_laid);
                assert_eq!(
                    (lay.bots, lay.ends[1]),
                    (BPAD, BPAD + kernel.letters * BSLOT)
                );
                assert_eq!(lay.arena, lay.ends[0] + GAP);
                kept[usize::from(kernel.pos_in_tops)] += 1;
            }
        }
        assert!(kept[0] > 0 && kept[1] > 0, "{kept:?}");
    }

    #[test]
    fn cells_fallback_and_bound() {
        assert_eq!(std::mem::size_of::<Cells>(), std::mem::size_of::<usize>());
        let mut cells = Cells::default();
        let mut seg = Segmented::default();
        let (mut bound, mut max_budget) = (0, 0);
        for ll in [
            vec![0; 10],
            vec![0; 11],
            [vec![1; 51], vec![0; 50]].concat(),
            vec![3; 900_000],
            vec![0; 25],
        ] {
            let r = Runs::of_column(&ll);
            let mut want = Vec::new();
            let mut got = vec![42; ll.len()];
            assert!(seg.invert_runs(&r, ll.len(), 1, &mut want, crate::Kernels::scalar()));
            assert!(cells.invert_runs(&r, ll.len(), 1, &mut got, crate::Kernels::scalar()));
            assert_eq!(got, want);
            // The arena bound plus positions; fallback's table is separately accounted.
            let budget = ll.len() * 4 / 25;
            bound = bound.max(48 * r.records().len() + 80 * budget + 5000);
            max_budget = max_budget.max(budget);
            if cells.state.counts.admitted > 0 && ll.len() >= 25 {
                assert!(cells.state.kernel.buf.len() <= bound);
                assert!(cells.state.kernel.pos.capacity() <= max_budget);
            }
        }
        assert!(
            cells.counts().density > 0
                && cells.counts().floor > 0
                && cells.counts().budget > 0
                && cells.counts().admitted > 0
        );
    }
    include!("cells_selective_test.rs");
    include!("cells_lazy_test.rs");
}
