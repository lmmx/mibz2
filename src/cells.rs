//! Experimental Rauzy–Veech cells inversion, ported from `examples/rauzy.rs`.
//! Admission is private to this type; Segmented and cooperative routes are unchanged.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::bwt::{InverseBwt, Segmented};

/// Off-by-default cells kernel, selected with `decode_with::<bwt::Cells>`.
pub type Cells = CellsKernel<false>;
/// Test/fuzz route: open admission (including all-singleton columns), budget n.
#[doc(hidden)]
pub type ForcedCells = CellsKernel<true>;

/// Route counts for evidence; local to a kernel, with no shared instrumentation.
#[derive(Clone, Copy, Debug, Default)]
pub struct Counts {
    pub admitted: usize,
    pub density: usize,
    pub floor: usize,
    pub budget: usize,
    pub invalid: usize,
    pub short_cycles: usize,
    pub cells: usize,
}

/// Implementation type; use `Cells` in experiments, `ForcedCells` only in tests/fuzzing.
#[derive(Default)]
pub struct CellsKernel<const FORCED: bool> {
    state: Box<State>,
}
#[derive(Default)]
struct State {
    fallback: Segmented,
    kernel: Kernel,
    counts: Counts,
}
#[derive(Default)]
struct Kernel {
    buf: Vec<u8>,
    lay: Layout,
    rule_end: usize,
    letters: usize,
    pos: Vec<u32>,
    #[cfg(test)]
    writer_reach: WriterReach,
}
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
    fn invert_runs(
        &mut self,
        r: &crate::runs::Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: crate::Kernels,
    ) -> bool {
        if self.attempt(r.records(), n, p, out) {
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
/// A letter's place in one order, for the induction by cells: 24 bytes. The two orders are
/// two regions of these in one buffer; a letter has one slot in each. The orders are only
/// ever read from their ends, so a slot is read once, when the induction's front reaches it.
///
/// | bytes | |
/// |---|---|
/// | 0..4 | the slot before it in its order (an offset in the buffer) |
/// | 4..8 | the letter's length |
/// | 8..12 | its word's length |
/// | 12..16 | its word: an offset in the buffer, of bytes if the length is at most SMALL, else of a rule |
/// | 16..24 | the letter's slot in the other order |
const SLOT: usize = 24;
/// How many slots on from an order's last letter the induction by cells fetches for
const FAR: usize = 12;
/// The buffer's first bytes, unused: a read `FAR` slots before an order's first stays inside
const PAD: usize = 4096;
/// The mark, in a word's length: the word holds the walk's row
const MARK: u32 = 1 << 31;
/// Where the block begins in the cycle's word, from the walk's row's place
const SHIFT: usize = 1;

/// The regions of the induction by cells' one buffer, as offsets in it.
#[derive(Clone, Copy, Default)]
struct Layout {
    /// the top order's slots
    tops: usize,
    /// the bottom order's slots
    bots: usize,
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
    fn check_canaries(&self) {
        #[cfg(any(miri, debug_assertions))]
        if !self.buf.is_empty() {
            for end in self.lay.ends {
                assert!(
                    self.buf[end..end + GAP].iter().all(|&x| x == 0xA5),
                    "cells canary"
                );
            }
        }
    }
    // Safe boundary: validates record tiling and primary row before raw memory access.
    // All refusal paths precede the writer, leaving out byte-for-byte untouched.
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
        let result = self
            .slots(n, recs, p, budget, forced)
            .and_then(|()| self.induce_cells(budget));
        self.check_canaries();
        let (word, len, cells) = result?;
        let period = (len & !MARK) as usize;
        if len & MARK == 0 || period == 0 || period > n {
            return Err(Refusal::Invalid);
        }
        self.write_cells(word, len, n, out);
        self.check_canaries();
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
    ) -> Result<(), Refusal> {
        if n == 0 || n >= 1 << 24 || p as usize >= n {
            return Err(Refusal::Invalid);
        }
        // This first gate reads only the record count, before validation or allocation.
        if !forced && recs.len() * 10 >= n {
            return Err(Refusal::Density);
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
        // Layout uses records + two splits, even when adjoining records coalesce.
        let cap = (recs.len() + 2 + budget) * SLOT;
        let tops = PAD;
        let bots = tops + cap + GAP;
        let arena = bots + cap + GAP;
        let rules = arena + BYTES as usize + 256 + 16 + 16 * budget + 32 + GAP;
        let allocation = rules + 16 * budget + 16 + GAP;
        if allocation >= u32::MAX as usize {
            return Err(Refusal::Invalid);
        }
        let lay = Layout {
            tops,
            bots,
            arena,
            rules,
            ends: [tops + cap, bots + cap, rules - GAP, allocation - GAP],
        };
        // Reuse the high-water allocation, but never geometrically over-allocate. Release
        // the old buffer before growing so both large allocations are not live at once.
        if self.buf.len() < allocation {
            self.buf = Vec::new();
            self.buf = vec![0; allocation];
        }
        self.lay = lay;
        self.letters = letters;
        for b in 0..256 {
            self.buf[arena + BYTES as usize + b] = b as u8;
        }
        for end in lay.ends {
            self.buf[end..end + GAP].fill(0xA5);
        }
        // a byte's next bottom slot, as its offset in the buffer
        for k in 0..256 {
            cnt[k + 1] += cnt[k];
            cnt[k] = (lay.bots + cnt[k] as usize * SLOT) as u32;
        }
        debug_assert_eq!(self.letters, cnt[256] as usize);
        let base = self.buf.as_mut_ptr();
        let first = lay.arena as u64 + BYTES as u64;
        let mut t = PAD;
        // SAFETY: the letters number at most the records and two, and a record is a row at
        // least: their slots lie inside each order's region
        unsafe {
            macro_rules! letter {
                ($lam:expr, $c:expr) => {{
                    let (lam, c): (u64, usize) = ($lam as u64, $c);
                    let k = *cnt.get_unchecked(c) as usize;
                    *cnt.get_unchecked_mut(c) = (k + SLOT) as u32;
                    let lw = 1 | (first + c as u64) << 32;
                    // (the slot before each order's first is set after the pass)
                    wr64(base, t, t.wrapping_sub(SLOT) as u32 as u64 | lam << 32);
                    wr64(base, t + 8, lw);
                    wr64(base, t + 16, k as u64);
                    wr64(base, k, k.wrapping_sub(SLOT) as u32 as u64 | lam << 32);
                    wr64(base, k + 8, lw);
                    wr64(base, k + 16, t as u64);
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
                    let kz = rd64(base, tz + 16) as usize;
                    wr32(base, tz + 8, 1 | MARK);
                    wr32(base, kz + 8, 1 | MARK);
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
            wr32(base, lay.bots, lay.bots as u32);
        }
        if letters - 1 > budget {
            return Err(Refusal::Floor);
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
    /// each later one takes a fresh slot, and the slot before the letter moves along with
    /// the promise of that fresh slot. Nothing is shortened in memory and no letter is
    /// moved: a step writes two slots and one word, and reads one or two.
    ///
    /// Every offset is a byte offset in the one buffer: a slot, a word kept as bytes, a
    /// word kept as a rule. A rule is its two factors as their slots held them: (length,
    /// word) twice.
    ///
    /// Returns (the cycle's word, its length with the mark, cells). `slots` has run on this
    /// kernel for the block, and nothing has changed since.
    #[inline(never)]
    fn induce_cells(&mut self, budget: usize) -> Result<(u32, u32, usize), Refusal> {
        const M: u64 = 0xFFFF_FFFF;
        let rr = self.letters;
        let lay = self.lay;
        if rr == 0 || self.buf.len() < lay.rules + 16 {
            return Err(Refusal::Invalid);
        }
        let base = self.buf.as_mut_ptr();
        // SAFETY: `slots` sized every region for budget cells; the counter is checked before each cell; a slot's offset is a letter's
        // slot or a fresh one below the region's end; a word's offset is in the arena or
        // the rules, with 16 bytes after it inside the buffer.
        unsafe {
            let mut top = lay.arena + BYTES as usize + 256 + 16;
            let mut rp = lay.rules;
            // fresh slots: one a cell at most, on each side
            let (ft0, fb0) = (lay.tops + rr * SLOT, lay.bots + rr * SLOT);
            let (mut ft, mut fb) = (ft0, fb0);
            // Count every cell, including equal-length steps which allocate no slot.
            let mut cells = 0usize;
            // the slot before each letter in its order; what is left of the letter; its
            // (length, word); the slot the next cell cut from it takes in the other order
            let (mut na, mut nb);
            let (mut ra, mut rb);
            let (mut aw, mut bw);
            let (mut bs, mut ts);
            let (mut sa, mut sb) = (ft0 - SLOT, fb0 - SLOT);
            // the letter `FAR` slots on in an order, as memory has it now: its slot in
            // the other order is where its cells will be written, and its word is read then
            // (what is read there may be a slot not yet written: the address is only hinted)
            macro_rules! ahead {
                ($s:expr) => {{
                    let far = $s - FAR * SLOT;
                    prefetch(base.wrapping_add(rd32(base, far + 16) as usize));
                    prefetch(base.wrapping_add(rd32(base, far + 12) as usize));
                }};
            }
            'fresh: loop {
                // both letters whole
                na = rd32(base, sa) as usize;
                ra = rd32(base, sa + 4) as u64;
                aw = rd64(base, sa + 8);
                bs = rd64(base, sa + 16) as usize;
                nb = rd32(base, sb) as usize;
                rb = rd32(base, sb + 4) as u64;
                bw = rd64(base, sb + 8);
                ts = rd64(base, sb + 16) as usize;

                ahead!(sa);
                ahead!(sb);
                if bs == sb {
                    // one letter, last in both orders: its points' cycles are closed
                    if aw as u32 & MARK != 0 {
                        // the four regions' ends: unchecked above, and valgrind cannot see
                        // a write from one region into the next inside the one allocation
                        if ft > lay.ends[0]
                            || fb > lay.ends[1]
                            || top + 32 > lay.ends[2]
                            || rp + 16 > lay.ends[3]
                        {
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
                    // the cell's word, A's then B's: as bytes, and as a rule; one is kept
                    let (la, lb) = (aw & M, bw & M);
                    let len = la + lb;
                    copy16(base.add(top), base.add((aw >> 32) as usize));
                    copy16(
                        base.add(top + la.min(16) as usize),
                        base.add((bw >> 32) as usize),
                    );
                    wr64(base, rp, aw);
                    wr64(base, rp + 8, bw);
                    let small = len <= SMALL as u64;
                    let word = if small { top } else { rp };
                    top += if small { len as usize } else { 0 };
                    rp += if small { 0 } else { 16 };
                    let cw = len | (word as u64) << 32;
                    if ra == rb {
                        // both letters end: the cell is the last cut from each
                        wr32(base, ts + 4, ra as u32);
                        wr64(base, ts + 8, cw);
                        wr64(base, ts + 16, bs as u64);
                        wr32(base, bs + 4, ra as u32);
                        wr64(base, bs + 8, cw);
                        wr64(base, bs + 16, ts as u64);
                        (sa, sb) = (na, nb);
                        continue 'fresh;
                    } else if ra > rb {
                        // B ends; A goes on, and its next cell is promised a fresh slot
                        wr32(base, ts + 4, rb as u32);
                        wr64(base, ts + 8, cw);
                        wr64(base, ts + 16, bs as u64);
                        wr32(base, fb, rd32(base, bs));
                        wr32(base, bs, fb as u32);
                        wr32(base, bs + 4, rb as u32);
                        wr64(base, bs + 8, cw);
                        wr64(base, bs + 16, ts as u64);
                        bs = fb;
                        fb += SLOT;
                        ra -= rb;
                        // (read after the cell is written: it may be that cell)
                        let s = nb;
                        ahead!(s);
                        nb = rd32(base, s) as usize;
                        rb = rd32(base, s + 4) as u64;
                        bw = rd64(base, s + 8);
                        ts = rd64(base, s + 16) as usize;
                    } else {
                        // A ends; B goes on
                        wr32(base, bs + 4, ra as u32);
                        wr64(base, bs + 8, cw);
                        wr64(base, bs + 16, ts as u64);
                        wr32(base, ft, rd32(base, ts));
                        wr32(base, ts, ft as u32);
                        wr32(base, ts + 4, ra as u32);
                        wr64(base, ts + 8, cw);
                        wr64(base, ts + 16, bs as u64);
                        ts = ft;
                        ft += SLOT;
                        rb -= ra;
                        let s = na;
                        ahead!(s);
                        na = rd32(base, s) as usize;
                        ra = rd32(base, s + 4) as u64;
                        aw = rd64(base, s + 8);
                        bs = rd64(base, s + 16) as usize;
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
        let base = self.buf.as_ptr();
        let ro = self.lay.rules;
        // SAFETY: a rule's factors are words written before it; a word kept as bytes has
        // 16 bytes at its offset; a position is below the period; `out` has 16 bytes past
        // the period
        unsafe {
            let rule = |w: usize| (rd64(base, w), rd64(base, w + 8));
            let nr = (self.rule_end - ro) / 16;
            if self.pos.capacity() < nr {
                self.pos = Vec::new();
                self.pos = vec![NIL; nr];
            } else {
                self.pos.resize(nr, NIL);
                self.pos.fill(NIL);
            }
            let pos = self.pos.as_mut_ptr();
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
        for n in 1..=10 {
            for bits in 0..1usize << n {
                let ll: Vec<_> = (0..n).map(|i| ((bits >> i) & 1) as u8).collect();
                for p in 0..n {
                    check(&mut kernel, &ll, p as u32);
                }
            }
        }
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
                        let before = out.clone();
                        assert!(kernel
                            .try_invert(r.records(), ll.len(), p as u32, cells - 1, true, &mut out)
                            .is_err());
                        assert_eq!(out, before);
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
        let floor = Runs::of_column(&[0; 11]);
        assert_eq!(
            kernel.try_invert(floor.records(), 11, 1, 1, false, &mut out),
            Err(Refusal::Floor)
        );
        assert_eq!(out, before);
        // A cyclic two-run exchange with almost equal lengths needs many subtractions.
        let ll = [vec![1; 51], vec![0; 50]].concat();
        let r = Runs::of_column(&ll);
        assert_eq!(
            kernel.try_invert(r.records(), 101, 0, 16, false, &mut out),
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
            assert!(std::panic::catch_unwind(|| kernel.check_canaries()).is_err());
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
            .flat_map(|b| [b; 10])
            .collect();
        let r = Runs::of_column(&ll);
        assert_eq!(
            (ll.len(), r.records().len(), ll.len() * 4 / 25),
            (550, 8, 88)
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
        eprintln!("production long writer n=550 p=0: {reach:?}");
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
        kernel
            .slots(ll.len(), r.records(), 17, ll.len(), true)
            .unwrap();
        let read = |at| u32::from_ne_bytes(kernel.buf[at..at + 4].try_into().unwrap());
        for (start, other) in [
            (kernel.lay.tops, kernel.lay.bots),
            (kernel.lay.bots, kernel.lay.tops),
        ] {
            let (mut marks, mut rows) = (0, 0);
            for i in 0..kernel.letters {
                let at = start + i * SLOT;
                marks += usize::from(read(at + 8) & MARK != 0);
                rows += read(at + 4) as usize;
                let partner = read(at + 16) as usize;
                assert!(partner >= other && partner < other + kernel.letters * SLOT);
                assert_eq!(read(partner + 16) as usize, at);
            }
            assert_eq!(marks, 1);
            assert_eq!(rows, ll.len());
        }
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
}
