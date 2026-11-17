// The lazy induction's body (`Kernel::induce_lazy`, `Kernel::induce_lazy_r12`), one block
// expression, included twice in `cells_lazy.rs` with `R12` a constant in scope: false for the
// 16-byte rules, true for the 12-byte rules' arm (`cells_w12.rs`). See `induce_lazy`.
{
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
        || (R12 && lay.ends[3] + GAP >= C12_REACH)
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
                } else if R12 {
                    // (the 12-byte rules' arm) the two children packed, 48 bits each
                    wr64(base, rp, pack12(aw));
                    wr64(base, rp + C12_B, pack12(bw));
                } else {
                    wr64(base, rp, aw);
                    wr64(base, rp + 8, bw);
                }
                let rule = if R12 { rule12_index(rp - lay.rules) } else { rp };
                let word = if small { top } else { rule };
                top += if small { len as usize } else { 0 };
                rp += if small { 0 } else if R12 { RULE12 } else { 16 };
                let cw = len | (word as u64) << 32;
                // (research only, `grammar-replay`: the word made, by length)
                #[cfg(feature = "grammar-replay")]
                replay_word(len);
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
