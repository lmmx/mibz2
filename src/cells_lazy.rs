// Cells with lazy letters (`LazyCells`): the workers' kernel from `LAZY_FROM_THREADS` threads
// (`parallel.rs`). A letter no front has reached has no slot, so the induction's live memory
// is a slot for each letter in flight, a third of the letters at most on Wikidata, where
// `slots` lays one out for every letter. It costs instructions and saves memory: slower on
// few threads, faster where the threads contend for the cache (the i9 run, #145 LOG
// 6903318530). Included in `cells.rs`.

/// The kernel a `CellsVariant` runs: Cells as shipped but for this.
pub(crate) const LAZY: u8 = 1;

/// The Cells kernel with another first stage and induction (`V`): `LAZY`, or (research
/// only, `memory-lever`) `STAGED_RULES`. Admission, budget, fallback and writer are Cells'
/// (`FORCED`: `ForcedCells`', for tests and fuzzing only).
#[doc(hidden)]
pub struct CellsVariant<const V: u8, const FORCED: bool = false>(CellsKernel<FORCED>);
/// Cells with lazy letters: a slot only for a letter in flight (`Kernel::induce_lazy`).
/// The same output and the same refusals as `Cells`, block for block.
pub type LazyCells = CellsVariant<LAZY>;
/// Test/fuzz route: lazy letters with `ForcedCells`' open admission
#[doc(hidden)]
pub type ForcedLazyCells = CellsVariant<LAZY, true>;

impl<const V: u8, const F: bool> Default for CellsVariant<V, F> {
    fn default() -> Self {
        let mut cells = CellsKernel::<F>::default();
        cells.state.kernel.variant = V;
        CellsVariant(cells)
    }
}

impl<const V: u8, const F: bool> CellsVariant<V, F> {
    pub fn counts(&self) -> Counts {
        self.0.counts()
    }
    /// The most slots the last lazy induction took from each pool, and the block's letters
    pub fn pool(&self) -> (usize, usize) {
        (self.0.state.kernel.lz.peak, self.0.state.kernel.letters)
    }
}

impl<const V: u8, const F: bool> InverseBwt for CellsVariant<V, F> {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.0.invert(ll, counts, p, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.0.invert_in_place(ll, counts, p)
    }
    fn reads_runs(&self) -> bool {
        self.0.reads_runs()
    }
    #[cfg(any(test, feature = "trace"))]
    fn took_cells(&self) -> bool {
        self.0.took_cells()
    }
    fn invert_runs(
        &mut self,
        r: &crate::runs::Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: crate::Kernels,
    ) -> bool {
        self.0.invert_runs(r, n, p, out, k)
    }
    #[cfg(any(test, feature = "trace"))]
    fn buffers(&self) -> Vec<crate::trace::Buffer> {
        self.0.buffers()
    }
}

/// In a bottom entry (`slots_lazy`): the entry holds a slot's offset, not an untouched letter
const TAG: u32 = 1 << 31;
/// In an untouched letter's bottom entry: the letter is the walk's row
const ROW: u32 = 1 << 30;
/// An untouched letter's bottom entry holds its length in this many low bits, and its byte
/// in the eight above them
const LEN_BITS: u32 = 22;

/// What `slots_lazy` leaves for `induce_lazy`, besides the layout.
#[derive(Clone, Copy)]
struct Lazy {
    /// the top order's bytes, one a letter: an offset in the output buffer, where the
    /// bottom entries (4 bytes a letter in bottom order, from `BPAD`) end
    tby: usize,
    /// the end of the bottom slots' pool, in the output buffer
    bend: usize,
    /// a byte's last letter the top front has not reached, as its bottom entry's index
    nt: [u32; 256],
    /// the most slots the last induction took from each pool
    peak: usize,
}
impl Default for Lazy {
    fn default() -> Self {
        Lazy {
            tby: 0,
            bend: 0,
            nt: [0; 256],
            peak: 0,
        }
    }
}

/// What the first pass over a block's records finds (`census`).
struct Census {
    /// a byte's letters, at its index plus one
    cnt: [u32; 257],
    letters: usize,
    /// the walk's row's run: records `ia..ib`, of byte `cz`, `total` rows, the row `off`
    /// rows in
    ia: usize,
    ib: usize,
    cz: usize,
    off: u32,
    total: u32,
}

/// The first pass over the records: the gates that read only their count, their tiling of
/// the block's `n` rows, each byte's letters, and the run that holds the walk's row `p`,
/// which is cut into the row and what lies before and after it.
fn census(n: usize, recs: &[u16], p: u32, forced: bool) -> Result<Census, Refusal> {
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
    Ok(Census {
        cnt,
        letters,
        ia,
        ib,
        cz,
        off,
        total,
    })
}

impl Kernel {
    /// The first stage of the kernel's variant
    fn first_stage(
        &mut self,
        n: usize,
        recs: &[u16],
        p: u32,
        budget: usize,
        forced: bool,
        out: &mut Vec<u8>,
    ) -> Result<(), Refusal> {
        if self.variant == LAZY {
            self.slots_lazy(n, recs, p, budget, forced, out)
        } else {
            self.slots(n, recs, p, budget, forced, out)
        }
    }

    /// The induction of the kernel's variant
    fn induce_variant(
        &mut self,
        budget: usize,
        out: &mut Vec<u8>,
    ) -> Result<(u32, u32, usize), Refusal> {
        match self.variant {
            LAZY => self.induce_lazy(budget, out),
            #[cfg(feature = "memory-lever")]
            STAGED_RULES => self.induce_cells_staged(budget, out),
            _ => self.induce_cells(budget, out),
        }
    }

    /// The cells kernel's first stage with lazy letters, from the front's run records
    /// (`slots`'s input). Nothing is laid out for a letter but 5 bytes, in the block's
    /// output buffer, which is dead until the writer:
    /// - **its entry**, 4 bytes, in bottom order (sorted stably by byte). The entry of a
    ///   letter no front has reached holds its length, its byte and whether it is the
    ///   walk's row (`ROW`);
    /// - **its byte**, in top order, after the entries.
    ///
    /// A slot is taken only for a cell, from two pools (`induce_lazy`): the top slots' in the
    /// kernel's buffer, where the letters' top slots lay, and the bottom slots' in the output
    /// buffer after the bytes. Each pool has room for a slot a letter, which is never
    /// reached: the slots in use are one for each letter a front has reached that is still
    /// alive.
    ///
    /// The words' regions are `slots`'s, at the same offsets.
    #[inline(never)]
    fn slots_lazy(
        &mut self,
        n: usize,
        recs: &[u16],
        p: u32,
        budget: usize,
        forced: bool,
        out: &mut Vec<u8>,
    ) -> Result<(), Refusal> {
        self.bottom_laid = false;
        self.lz.bend = 0;
        let Census {
            mut cnt,
            letters,
            ia,
            ib,
            cz,
            off,
            total,
        } = census(n, recs, p, forced)?;
        // a length fits an entry's low bits (a bzip2 block has at most 900,000 rows)
        if n >= 1 << LEN_BITS {
            return Err(Refusal::Invalid);
        }
        // (test only, `fresh_room`: the offsets of the original induction's layout, whose
        // two orders each had room for a fresh slot a cell, so that the words lie at the
        // same offsets for the original and for this)
        #[cfg(test)]
        let room = letters + if self.fresh_room { budget } else { 0 };
        #[cfg(not(test))]
        let room = letters;
        let cap = room * SLOT;
        let tops = PAD;
        #[cfg(test)]
        let arena = tops + cap + GAP + if self.fresh_room { cap + GAP } else { 0 };
        #[cfg(not(test))]
        let arena = tops + cap + GAP;
        let rules = arena + BYTES as usize + 256 + 16 + 16 * budget + 32 + GAP;
        let allocation = rules + 16 * budget + 16 + GAP;
        // in the output buffer: the entries, the top order's bytes, the bottom slots' pool
        let tby = BPAD + 4 * letters;
        let streams = tby + letters;
        let bots = streams + GAP;
        let bend = bots + letters * BSLOT;
        // (an entry holds a slot's offset beside `TAG`: 31 bits)
        if allocation >= 1 << 31 || bend + GAP >= 1 << 31 {
            return Err(Refusal::Invalid);
        }
        if letters - 1 > budget {
            return Err(Refusal::Floor);
        }
        let lay = Layout {
            tops,
            bots,
            bout: true,
            arena,
            rules,
            ends: [tops + cap, streams, rules - GAP, allocation - GAP],
        };
        if self.buf.len() < allocation {
            self.buf = Vec::new();
            self.buf = vec![0; allocation];
        }
        if out.len() < bend + GAP {
            out.resize(bend + GAP, 0);
        }
        self.bottom_laid = true;
        self.lay = lay;
        self.letters = letters;
        for b in 0..256 {
            self.buf[arena + BYTES as usize + b] = b as u8;
        }
        for (i, end) in lay.ends.into_iter().enumerate() {
            if i == 1 {
                out[end..end + GAP].fill(0xA5);
            } else {
                self.buf[end..end + GAP].fill(0xA5);
            }
        }
        out[bend..bend + GAP].fill(0xA5);
        // a byte's next entry, as its index
        for k in 0..256 {
            cnt[k + 1] += cnt[k];
        }
        debug_assert_eq!(letters, cnt[256] as usize);
        let bb = out.as_mut_ptr();
        // the letters made so far
        let mut t = 0usize;
        // SAFETY: this pass makes the letters `census` counted, from the same records: `t`
        // stays below `letters`, and a byte's entries are as many as its letters, so each
        // index is below `letters` too. The entries are `4 * letters` bytes at `BPAD` and
        // the bytes `letters` at `tby`, inside `out`.
        unsafe {
            macro_rules! letter {
                ($lam:expr, $c:expr, $row:expr) => {{
                    let (lam, c): (u32, usize) = ($lam, $c);
                    let k = *cnt.get_unchecked(c) as usize;
                    *cnt.get_unchecked_mut(c) = k as u32 + 1;
                    debug_assert!(k < letters && t < letters && lam < 1 << LEN_BITS);
                    wr32(bb, BPAD + 4 * k, lam | (c as u32) << LEN_BITS | $row);
                    *bb.add(tby + t) = c as u8;
                    t += 1;
                }};
            }
            let nrec = recs.len();
            let mut i = 0;
            while i < nrec {
                if i == ia {
                    // the walk's row's run: what is before the row, the row, what is after
                    if off > 0 {
                        letter!(off, cz, 0);
                    }
                    letter!(1u32, cz, ROW);
                    if total - off > 1 {
                        letter!(total - off - 1, cz, 0);
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
                letter!(lam, c, 0);
            }
        }
        debug_assert_eq!(t, letters);
        // each byte's last entry (a byte with no letter is never asked for)
        let mut nt = [0u32; 256];
        for (c, last) in nt.iter_mut().enumerate() {
            *last = cnt[c].wrapping_sub(1);
        }
        self.lz = Lazy {
            tby,
            bend,
            nt,
            peak: 0,
        };
        Ok(())
    }

    /// The induction by cells with lazy letters: `induce_cells`'s steps, in its order, making
    /// its words at its offsets. Only how a letter is kept differs.
    ///
    /// **A letter no front has reached has no slot.** Its place in the top order is its
    /// byte's among the top order's bytes, read from the last one back; its place in the
    /// bottom order is its entry's (`slots_lazy`), read from the last one back. Each front
    /// reads slots while its order's links lead to slots, and takes the next place of its
    /// stream when a link is 0.
    ///
    /// **A letter is reached first by one front.** That front reads it from its entry, and
    /// takes a slot from each pool: the one its cells are written to in the other order
    /// (linked 0: before it lies the stream's next place), and its own front slot, which
    /// `induce_cells` would have read it from, to promise as that does. The first goes into
    /// the letter's entry, with `TAG`. When the other front comes to the letter's place in
    /// its stream, the entry holds that slot, and the front goes on through it: the place
    /// holds the cells cut from the letter, the last of them in that slot.
    ///
    /// The other front finds the entry without a map from one order to the other. The
    /// bottom front's place is the entry's index. The top front's is the last entry of the
    /// letter's byte that it has not reached (`nt`): within a byte the two orders agree.
    ///
    /// **Which slot an entry holds.** A front writes a letter's entry once, as it reaches
    /// the letter first, and no front reads a place of its stream twice. So an entry the
    /// top front reads with `TAG` was written by the bottom front and holds a top slot, and
    /// one the bottom front reads with `TAG` holds a bottom slot.
    ///
    /// **The pools.** A front that reaches a letter first takes a slot of each kind, and
    /// the slots `induce_cells` lets go dead are given back a slot of each kind at a time:
    /// both front slots at an equal step, and both slots of a closed letter that is not the
    /// walk's row's. So the two pools are handed out as pairs: the pair given back last
    /// (kept in its top slot: the top slot given back before it, and its bottom slot),
    /// else the next of each pool never handed out. The pairs in use number the letters a
    /// front has reached that are still alive, so pools of a slot a letter are never
    /// exhausted; the bound is checked where a new pair is taken all the same.
    ///
    /// Returns (the cycle's word, its length with the mark, cells). `slots_lazy` has run on
    /// this kernel for the block, and nothing has changed since.
    #[inline(never)]
    fn induce_lazy(
        &mut self,
        budget: usize,
        out: &mut Vec<u8>,
    ) -> Result<(u32, u32, usize), Refusal> {
        const M: u64 = 0xFFFF_FFFF;
        /// In `bs` or `fb`: no slot yet, but the entry of a letter just read from its
        /// stream (its index in the low bits). A slot's offset is below it.
        const ENTRY: usize = 1 << 31;
        let rr = self.letters;
        let lay = self.lay;
        let Lazy {
            tby, bend, mut nt, ..
        } = self.lz;
        if rr == 0
            || !self.bottom_laid
            || bend == 0
            || tby != BPAD + 4 * rr
            || self.buf.len() < lay.rules + 16
            || out.len() < bend + GAP
        {
            return Err(Refusal::Invalid);
        }
        let tend = lay.ends[0];
        let base = self.buf.as_mut_ptr();
        let bb = out.as_mut_ptr();
        // SAFETY: `slots_lazy` sized the words' regions for budget cells, and the counter is
        // checked before each cell. A top slot's offset is one the pool handed out, below
        // `tend` in the kernel's buffer; a bottom slot's is one below `bend` in the output
        // buffer: a link, a letter's other slot and a tagged entry hold only such offsets or
        // 0, each of the right buffer (see above for an entry). The top order's bytes are
        // read once each, from the last, and `ti` is checked before each; a byte's entries
        // are as many as its bytes there, so `nt` gives an entry's index below `rr`; `kb`
        // is checked before each. A word's offset is in the arena or the rules, with 16
        // bytes after it inside the buffer.
        unsafe {
            // a single byte's word, but for the byte: length 1, in the single bytes' table
            let first1 = (lay.arena as u64 + BYTES as u64) << 32 | 1;
            let mut top = lay.arena + BYTES as usize + 256 + 16;
            let mut rp = lay.rules;
            // the cells still allowed
            let mut left = budget;
            // the pools: the pair given back last, and each pool's first slot never
            // handed out
            let mut free = 0usize;
            let (mut tnew, mut bnew) = (lay.tops, lay.bots);
            // the streams: the top order's bytes and the entries not yet reached
            let (mut ti, mut kb) = (rr, rr);
            // as in `induce_cells`: the link before each letter in its order (0: the
            // stream); what is left of the letter; its (length, word); the slot the next
            // cell cut from it takes in the other order; the letters' front slots
            let (mut na, mut nb) = (0usize, 0usize);
            let (mut ra, mut rb) = (0u64, 0u64);
            let (mut aw, mut bw) = (0u64, 0u64);
            let (mut bs, mut ts) = (0usize, 0usize);
            let (mut fa, mut fb) = (0usize, 0usize);
            // A pair of slots: (top, bottom)
            macro_rules! take {
                () => {{
                    let (t, b);
                    if free != 0 {
                        t = free;
                        free = rd32(base, t) as usize;
                        b = rd32(base, t + 4) as usize;
                    } else {
                        if tnew + SLOT > tend || bnew + BSLOT > bend {
                            debug_assert!(false, "cells pools exhausted");
                            return Err(Refusal::Invalid);
                        }
                        (t, b) = (tnew, bnew);
                        tnew += SLOT;
                        bnew += BSLOT;
                    }
                    (t, b)
                }};
            }
            macro_rules! give {
                ($t:expr, $b:expr) => {{
                    let (t, b): (usize, usize) = ($t, $b);
                    wr32(base, t, free as u32);
                    wr32(base, t + 4, b as u32);
                    free = t;
                }};
            }
            // The top order's letter at link `$s`: from its slot, else the stream's next
            // place. A letter read from its entry has `bs` its entry (`ENTRY`) till it
            // takes its slots (`settle_top`).
            macro_rules! read_top {
                ($s:expr, $settle:literal) => {{
                    let mut s: usize = $s;
                    if s == 0 {
                        if ti == 0 {
                            return Err(Refusal::Invalid);
                        }
                        ti -= 1;
                        let c = *bb.add(tby + ti) as usize;
                        let k = *nt.get_unchecked(c) as usize;
                        *nt.get_unchecked_mut(c) = (k as u32).wrapping_sub(1);
                        debug_assert!(k < rr);
                        let e = rd32(bb, BPAD + 4 * k);
                        if e & TAG != 0 {
                            // the bottom front reached it first: its cells, from this slot
                            s = (e ^ TAG) as usize;
                        } else {
                            // (research only, `induction-trace`: read from its stream)
                            #[cfg(feature = "induction-trace")]
                            crate::itrace::read(0, true);
                            na = 0;
                            ra = (e & ((1 << LEN_BITS) - 1)) as u64;
                            aw = first1 + ((c as u64) << 32) + (((e & ROW) as u64) << 1);
                            if $settle {
                                (fa, bs) = take!();
                                wr32(bb, bs, 0);
                                wr32(bb, BPAD + 4 * k, TAG | bs as u32);
                            } else {
                                bs = ENTRY | k;
                            }
                        }
                    }
                    if s != 0 {
                        // (research only, `induction-trace`: read from a slot)
                        #[cfg(feature = "induction-trace")]
                        crate::itrace::read(0, false);
                        na = rd32(base, s) as usize;
                        ra = rd32(base, s + 4) as u64;
                        aw = rd64(base, s + 8);
                        bs = rd_other(base, s);
                        fa = s;
                    }
                }};
            }
            // The bottom order's letter at link `$s` (its length and word through its top
            // slot): from its slot, else the stream's next place. A letter read from its
            // entry has `fb` its entry (`ENTRY`) till it takes its slots (`settle_bottom`).
            macro_rules! read_bottom {
                ($s:expr, $settle:literal) => {{
                    let mut s: usize = $s;
                    if s == 0 {
                        if kb == 0 {
                            return Err(Refusal::Invalid);
                        }
                        kb -= 1;
                        let e = rd32(bb, BPAD + 4 * kb);
                        if e & TAG != 0 {
                            // the top front reached it first: its cells, from this slot
                            s = (e ^ TAG) as usize;
                        } else {
                            // (research only, `induction-trace`: read from its stream)
                            #[cfg(feature = "induction-trace")]
                            crate::itrace::read(1, true);
                            nb = 0;
                            rb = (e & ((1 << LEN_BITS) - 1)) as u64;
                            let c = (e >> LEN_BITS & 255) as u64;
                            bw = first1 + (c << 32) + (((e & ROW) as u64) << 1);
                            if $settle {
                                (ts, fb) = take!();
                                wr32(base, ts, 0);
                                wr32(bb, BPAD + 4 * kb, TAG | ts as u32);
                            } else {
                                fb = ENTRY | kb;
                            }
                        }
                    }
                    if s != 0 {
                        // (research only, `induction-trace`: read from a slot)
                        #[cfg(feature = "induction-trace")]
                        crate::itrace::read(1, false);
                        (nb, ts) = rd_bottom(bb, s);
                        rb = rd32(base, ts + 4) as u64;
                        bw = rd64(base, ts + 8);
                        fb = s;
                    }
                }};
            }
            // A letter just read from its entry takes its slots: where its cells go in the
            // other order (linked to the stream), which its entry now holds, and its front
            // slot.
            macro_rules! settle_top {
                () => {{
                    if bs & ENTRY != 0 {
                        let k = bs ^ ENTRY;
                        (fa, bs) = take!();
                        wr32(bb, bs, 0);
                        wr32(bb, BPAD + 4 * k, TAG | bs as u32);
                    }
                }};
            }
            macro_rules! settle_bottom {
                () => {{
                    if fb & ENTRY != 0 {
                        let k = fb ^ ENTRY;
                        (ts, fb) = take!();
                        wr32(base, ts, 0);
                        wr32(bb, BPAD + 4 * k, TAG | ts as u32);
                    }
                }};
            }
            'fresh: loop {
                // both letters whole
                read_top!(na, false);
                read_bottom!(nb, false);
                if bs == fb {
                    // one letter, last in both orders (A's bottom slot is the bottom
                    // front's, or both fronts read the one entry): its points' cycles are
                    // closed
                    // (research only, `induction-trace`: the closed letter's word length)
                    #[cfg(feature = "induction-trace")]
                    crate::itrace::closed(aw & M);
                    if aw as u32 & MARK != 0 {
                        if top + 32 > lay.ends[2] || rp + 16 > lay.ends[3] {
                            debug_assert!(false, "cells region exhausted");
                            return Err(Refusal::Invalid);
                        }
                        self.rule_end = rp;
                        self.lz.peak = (tnew - lay.tops) / SLOT;
                        return Ok(((aw >> 32) as u32, aw as u32, budget - left));
                    }
                    if bs & ENTRY == 0 {
                        give!(fa, fb);
                    }
                    continue 'fresh;
                }
                settle_top!();
                settle_bottom!();
                loop {
                    if left == 0 {
                        return Err(Refusal::Budget);
                    }
                    left -= 1;
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
                    // (research only, `induction-trace`: the cell's two letters, what was
                    // left of each and their words' lengths)
                    #[cfg(feature = "induction-trace")]
                    crate::itrace::cell(ra, rb, la, lb);
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
                        0,
                        0,
                        (budget - left) as u64,
                    ]);
                    if ra == rb {
                        // both letters end: the cell is the last cut from each, and both
                        // front slots are dead
                        wr32(base, ts + 4, ra as u32);
                        wr64(base, ts + 8, cw);
                        wr_other(base, ts, bs);
                        wr32(bb, bs + 4, ts as u32);
                        #[cfg(test)]
                        self.record_selected_slots_in(ts, bs, bb);
                        debug_assert!(fa != ts && fb != bs && fa != na && fb != nb);
                        give!(fa, fb);
                        continue 'fresh;
                    } else if ra > rb {
                        // B ends; A goes on, and its next cell is promised B's front slot
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
                        read_bottom!(nb, true);
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
                        read_top!(na, true);
                    }
                }
            }
        }
    }
}
