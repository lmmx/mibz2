// The memory-lever experiment's remaining kernel (`memory-lever` feature; #145 QUEUED
// 6901989466): Cells with its rules staged and stored past the cache. Included in `cells.rs`
// only with the feature; it is not a default anywhere. (The experiment's other kernel, lazy
// letters, is `cells_lazy.rs`.)

/// The kernel a `CellsVariant` runs: Cells as shipped but for this.
pub(crate) const STAGED_RULES: u8 = 2;

/// Cells with its rules staged and stored past the cache (`Kernel::induce_cells_staged`)
#[doc(hidden)]
pub type StagedCells = CellsVariant<STAGED_RULES>;
/// Test/fuzz route: the staged kernel with `ForcedCells`' open admission
#[doc(hidden)]
pub type ForcedStagedCells = CellsVariant<STAGED_RULES, true>;

/// How many bytes of rules the induction stages before it sends them on: a power of two, and
/// a multiple of a rule's 16 bytes (fewer in tests, so that short columns send too)
const STAGE: usize = if cfg!(any(test, miri)) { 128 } else { 1024 };
/// Whether the induction stages its rules: only where staged rules can then be stored past
/// the cache. Elsewhere it writes each rule to its place, as before.
const STAGED: bool = cfg!(target_arch = "x86_64");

/// The rules the induction by cells has made and not yet sent to their region.
///
/// The induction writes a rule once and never reads one; the writer reads them all
/// afterwards. So the rules are kept out of the cache while the induction runs: they are
/// written here, in a buffer small enough to stay in the first-level cache, and sent on
/// when it is full, as whole 64-byte lines with stores that go past the cache (x86-64;
/// elsewhere, and under Miri, with plain copies). Sent one rule at a time, such a store
/// would share the core's fill buffers with the induction's cache misses, and a line would
/// be written out in pieces.
///
/// The stage is a ring: the rule at offset `rp` of the kernel's buffer lies at
/// `rp % STAGE`. A rule's offset is a multiple of 16 (`slots`), so no rule wraps, and the
/// stage is full when a rule ends on a multiple of `STAGE`.
#[repr(C, align(64))]
struct Stage([u8; STAGE]);

/// Sends the staged rules at offsets `from..to` of the kernel's buffer on to it: plain
/// stores up to the first line boundary, whole lines past the cache, plain stores for what
/// is left of a line.
///
/// # Safety
/// `base` is the kernel's buffer and `from..to` lies in its rules' region; `stage` is the
/// `Stage`'s bytes; `from..to` does not cross a multiple of `STAGE` (so it is contiguous
/// in the ring) and holds rules the induction staged.
#[cold]
#[inline(never)]
unsafe fn send_rules(base: *mut u8, stage: *const u8, from: usize, to: usize) {
    debug_assert!(from <= to && to - from <= STAGE - from % STAGE);
    let n = to - from;
    // SAFETY: `n` bytes are readable at `src`, inside the stage, and writable at `dst`,
    // inside the buffer; the two are distinct allocations
    unsafe {
        let src = stage.add(from % STAGE);
        // (`base` may lie before its buffer, `Layout::shift`: the sum is inside it)
        let dst = base.wrapping_add(from);
        let head = dst.align_offset(64).min(n);
        core::ptr::copy_nonoverlapping(src, dst, head);
        #[allow(unused_mut)]
        let mut i = head;
        #[cfg(all(target_arch = "x86_64", not(miri)))]
        while i + 64 <= n {
            use core::arch::x86_64::{__m128i, _mm_loadu_si128, _mm_stream_si128};
            // (`dst + i` is 64-aligned; the stage's bytes are read unaligned)
            debug_assert_eq!(dst.add(i) as usize % 64, 0, "a whole line");
            let (s, d) = (src.add(i).cast::<__m128i>(), dst.add(i).cast::<__m128i>());
            _mm_stream_si128(d, _mm_loadu_si128(s));
            _mm_stream_si128(d.add(1), _mm_loadu_si128(s.add(1)));
            _mm_stream_si128(d.add(2), _mm_loadu_si128(s.add(2)));
            _mm_stream_si128(d.add(3), _mm_loadu_si128(s.add(3)));
            i += 64;
        }
        core::ptr::copy_nonoverlapping(src.add(i), dst.add(i), n - i);
        // (test only: the model. Whole lines were sent past the cache, or would have been
        // outside Miri. From 127 bytes on there is a whole line wherever the buffer lies, so
        // a full stage's send is marked whatever the allocator did)
        #[cfg(test)]
        if n - head >= 64 || n >= 127 {
            PENDING.with(|p| p.set(true));
        }
    }
}

#[cfg(test)]
thread_local! {
    /// (test only) The model of the stores that go past the cache: `send_rules` sent whole
    /// lines and no fence has run since. Set wherever such a store is made (or, under
    /// Miri, would be), cleared where `sfence` runs.
    static PENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// (test only) how many fences have run on this thread
    static FENCES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// (test only) how many of them found the mark set, and cleared it
    static CLEARED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Runs `sfence` when dropped: on every way out of the staged induction, an unwinding one
/// included. `_mm_stream_si128` asks for it before any other access to the memory it wrote
/// and before code that assumes ordinary store ordering runs: here, before the writer reads
/// the rules and before the kernel's buffer can reach another thread.
struct Fence;

impl Drop for Fence {
    fn drop(&mut self) {
        #[cfg(all(target_arch = "x86_64", not(miri)))]
        // SAFETY: sfence is in every x86-64's instruction set and touches no memory
        unsafe {
            core::arch::x86_64::_mm_sfence();
        }
        #[cfg(test)]
        {
            if PENDING.with(|p| p.replace(false)) {
                CLEARED.with(|c| c.set(c.get() + 1));
            }
            FENCES.with(|f| f.set(f.get() + 1));
        }
    }
}

impl Kernel {
    /// The induction by cells with its rules staged: `induce_cells`, but for where a rule is
    /// written and what the fetch-ahead hints. A rule is not written to its place as it is
    /// made. It is staged (`Stage`), and the staged rules are sent on to the rules' region a
    /// stage at a time, past the cache where that can be done. The region holds every rule
    /// by the time this returns a word; nothing here reads a rule.
    ///
    /// The same steps as `induce`, kept another way, to the cycle of the walk's row: the
    /// closed letter whose word carries the mark.
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
    /// Every offset is a byte offset in one of two buffers: a top slot or a word kept as
    /// bytes in the kernel's; a bottom slot or a word kept as a rule in the tail's spare
    /// capacity, which no one else reads or writes until the kernel has returned. A rule is
    /// its two factors as their slots held them: (length, word) twice. The induction writes
    /// a rule and never reads one.
    ///
    /// Returns (the cycle's word, its length with the mark, cells). `slots` has run on this
    /// kernel for the block, and nothing has changed since.
    #[inline(never)]
    fn induce_cells_staged(
        &mut self,
        budget: usize,
        tail: &mut Vec<u8>,
    ) -> Result<(u32, u32, usize), Refusal> {
        const M: u64 = 0xFFFF_FFFF;
        let rr = self.letters;
        let lay = self.lay;
        // (a rule's offset is a multiple of 16: the stage's ring relies on it)
        if rr == 0 || lay.rules % 16 != 0 || self.bslot() != BSLOT {
            return Err(Refusal::Invalid);
        }
        // each buffer holds the layout's regions: this one, and the tail's spare capacity
        if self.buf.len() < lay.ends[if lay.bout { 2 } else { 3 }] + GAP
            || tail.capacity() - tail.len() < lay.tail
        {
            return Err(Refusal::Invalid);
        }
        let base = self.buf.as_mut_ptr();
        // the bottom slots' and the rules' buffer: the tail's spare capacity, which `slots`
        // sized for them
        let bb = if lay.bout {
            tail_from(tail, lay.shift)
        } else {
            base
        };
        // SAFETY: `slots` sized the words' regions for budget cells, and the counter is
        // checked before each cell; a slot's offset is a letter's slot, written by `slots`
        // inside its order's region (a promised slot is one of them, let go at the front);
        // a bottom slot's second field is a top slot, so a letter read through it is read
        // inside the top region; a word's offset is in the arena, with 16 bytes after it
        // inside the buffer, or in the rules, in the bottom slots' buffer, where only the
        // writer reads it. In the tail's spare capacity, only what `slots` and this have
        // written is read: the letters' bottom slots and the bytes before them. A rule's
        // offset is a multiple of 16 (the region's start is, checked above, and a rule is 16
        // bytes), so its place in the stage, the offset modulo `STAGE`, has its 16 bytes
        // inside the stage; `send_rules` is given offsets from the region's start to the end
        // of the last rule made, within one multiple of `STAGE`, where the stage holds those
        // rules, and the bottom slots' buffer, which the rules' region lies in.
        unsafe {
            let mut top = lay.arena + BYTES as usize + 256 + 16;
            let mut rp = lay.rules;
            // the rules made and not yet sent to their region
            let mut stage = Stage([0; STAGE]);
            let sp = stage.0.as_mut_ptr();
            let _fence = Fence;
            #[cfg(test)]
            let mut sends = 0usize;
            // each order's last letter's slot
            let (ft0, fb0) = (lay.tops + rr * SLOT, lay.bots + rr * BSLOT);
            // The cells still allowed: every cell is counted, including equal-length steps
            // which allocate no slot. (Counted down: one value in the loop, not two.)
            let mut left = budget;
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
            // The word of the letter in top slot `$t`, fetched: only a word kept as bytes.
            // A word kept as a rule is never read here, and where rules are staged its line
            // is being kept out of the cache: the single bytes' line is hinted in its place.
            let bytes = lay.arena + BYTES as usize;
            macro_rules! word_ahead {
                ($t:expr) => {{
                    let w = rd64(base, $t + 8);
                    let at = if !STAGED || w as u32 <= SMALL {
                        (w >> 32) as usize
                    } else {
                        bytes
                    };
                    prefetch(base.wrapping_add(at));
                }};
            }
            macro_rules! ahead {
                ($s:expr) => {{
                    let far = $s - FAR * SLOT;
                    prefetch(bb.wrapping_add(rd32(base, far + 16) as usize));
                    word_ahead!(far);
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
                    word_ahead!(near);
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
                        // what is staged still: the rules from the last multiple of
                        // `STAGE`, or from the region's start
                        if STAGED {
                            send_rules(bb, sp, (rp - rp % STAGE).max(lay.rules), rp);
                        }
                        #[cfg(test)]
                        {
                            self.rule_sends = sends;
                        }
                        self.rule_end = rp;
                        return Ok(((aw >> 32) as u32, aw as u32, budget - left));
                    }

                    (sa, sb) = (na, nb);
                    continue 'fresh;
                }
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
                    } else if STAGED {
                        // the rule, staged: its offset is a multiple of 16, so its 16
                        // bytes lie inside the ring
                        let q = rp % STAGE;
                        wr64(sp, q, aw);
                        wr64(sp, q + 8, bw);
                        if q == STAGE - 16 {
                            // the stage is full: all of it but what lay before the region
                            send_rules(bb, sp, (rp + 16 - STAGE).max(lay.rules), rp + 16);
                            #[cfg(test)]
                            {
                                sends += 1;
                                // (the first send the model marked: a first, short send
                                // may hold no whole line)
                                if self.panic_after_send && PENDING.with(|p| p.get()) {
                                    panic!("test: unwinding after a send");
                                }
                            }
                        }
                    } else {
                        wr64(bb, rp, aw);
                        wr64(bb, rp + 8, bw);
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
                        (budget - left) as u64,
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
}
