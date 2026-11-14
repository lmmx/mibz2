// Test-only original d4f0e8f induction; generated/verified by bench/cells-selective/verify.py.
impl Kernel {
    fn induce_cells_original(&mut self, budget: usize) -> Result<(u32, u32, usize), Refusal> {
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
                    #[cfg(test)]
                    self.record_materialization([
                        ra,
                        rb,
                        aw,
                        bw,
                        cw,
                        top as u64,
                        rp as u64,
                        ft as u64,
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
                        wr64(base, ts + 16, bs as u64);
                        wr32(base, bs + 4, ra as u32);
                        wr64(base, bs + 8, cw);
                        wr64(base, bs + 16, ts as u64);
                        #[cfg(test)]
                        self.record_selected_slots(ts, bs);
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
                        #[cfg(test)]
                        self.record_selected_slots(ts, bs);
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
                        #[cfg(test)]
                        self.record_selected_slots(ts, bs);
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
}
