// The first-occurrence writer (study 6, `docs/journal/2026-11-18-cells-owner-schedules.md`, #582;
// MONODROMY record 69): research only, a third writer beside the shipped `write_cells` and the
// one-scan `write_owned` (#581), for the writer-only comparison of #145 QUEUED 6988752241
// (CHECKPOINT 6989270254 on #314). Included in `cells.rs` with tests and with the research
// feature `owned-writer`; the shipped path never calls it.
//
// **The walk.** The root's spelling is read from its left end, the output cursor starting at the
// rotation the marked walk gives and wrapping once, after the marked byte. Each rule's owner is
// its first occurrence in that order:
// - an arena word or the marked byte is written at the cursor;
// - on first entering a rule, its two operands are read and the current output offset is stored
//   in its first field (the offset in the high half, the low half zero: a genuine operand has a
//   positive length, so the tag is told from it even at offset zero). A short left operand is
//   written at once and the walk goes on to the right; a rule left operand is entered, the right
//   one saved on a stack;
// - on a later visit, the tag names the source, already complete (an incomplete expansion is an
//   ancestor of the visit, and a grammar has no cycle), and the bytes are copied at once.
// No deferred commands, no position table. The stack holds at most one right operand per rule on
// the open path: at most R - 1, which the comb family (L = 1^(n-1) 0 at row n - 2) attains.
// Study 6 found the walk exact on short-period blocks too, so its only guard is a root that is a
// rule; a cycle shorter than the block is then repeated to the block, as `write_cells` does.
//
// **Memory safety,** on the premise `write_cells` rests on (the rule records are the induction's
// own, valid whatever the input): the cursor is kept below the period, every arena write is at a
// cursor below the period with at most 16 bytes (the output has 32 past the block), a copy is
// made only where its source and destination lie within the period, and the stack is bounded by
// its capacity (the rule count). A violation leaves a wrong block, never an access out of bounds.

/// (research only, `owned-writer`) the first-occurrence writer's calls, ns and deepest stack
#[cfg(feature = "owned-writer")]
static FIRST_PHASES: [std::sync::atomic::AtomicU64; 3] = [const { std::sync::atomic::AtomicU64::new(0) }; 3];

/// (research only, `owned-writer`) the first-occurrence writer's [calls, ns, deepest stack]
#[cfg(feature = "owned-writer")]
#[doc(hidden)]
pub fn first_phases() -> [u64; 3] {
    std::array::from_fn(|i| FIRST_PHASES[i].load(std::sync::atomic::Ordering::Relaxed))
}

impl Kernel {
    /// `write_cells`' output by study 6's first-occurrence walk, the shipped writer's where the
    /// word is not a rule. The rule records' first fields hold the tags afterwards (the writer
    /// is their last reader).
    #[inline(never)]
    fn write_first(&mut self, word: u32, len: u32, n: usize, out: &mut Vec<u8>) {
        let p = (len & !MARK) as usize;
        let ro = self.lay.rules;
        let nr = (self.rule_end - ro) / 16;
        if nr == 0 || (word as usize) < ro || p == 0 || p > n {
            return self.write_cells(word, len, n, out);
        }
        #[cfg(feature = "owned-writer")]
        let t0 = std::time::Instant::now();
        out.resize(n + 32, 0);
        let dst = out.as_mut_ptr();
        let base = self.buf.as_mut_ptr();
        let mut stack = std::mem::take(&mut self.first_stack);
        stack.clear();
        stack.reserve(nr);
        let cap = stack.capacity().min(nr);
        let st = stack.as_mut_ptr();
        let mut peak = 0usize;
        // SAFETY: rules lie in [ro, rule_end), 16 bytes each, each child an arena word below `ro`
        // (at most 16 bytes, 16 readable) or an earlier record (the induction's premise); the
        // cursor stays below `p`, `out` has `p + 32` bytes at least (`p <= n`), a copy is checked
        // against the period on both sides, and the stack's `cap` entries are its own
        unsafe {
            // the row's place in the word (as `write_cells` finds it)
            let (mut w, mut j) = (word as usize, 0usize);
            while w >= ro {
                let (a, b) = (rd64(base, w), rd64(base, w + 8));
                if a as u32 & MARK != 0 {
                    w = (a >> 32) as usize;
                } else {
                    j += a as u32 as usize;
                    w = (b >> 32) as usize;
                }
            }
            let start = (j + SHIFT) % p;
            let mut c = (p - start) % p;
            let mut d: u64 = (word as u64) << 32 | len as u64;
            let mut sp = 0usize;
            loop {
                let r = (d >> 32) as usize;
                if r < ro {
                    // an arena word, or the marked byte
                    let l = (d as u32 & !MARK) as usize;
                    copy_small(dst.add(c), base.add(r), l);
                    c += l;
                } else {
                    let a = rd64(base, r);
                    if a as u32 == 0 {
                        // a later visit: the first occurrence is complete, copy it
                        let f = (a >> 32) as usize;
                        let l = d as u32 as usize;
                        if f <= p && l <= p - f && l <= p - c {
                            copy_long(dst.add(c), dst.add(f), l);
                        }
                        c += l;
                    } else {
                        // the first entry: tag it, then its left operand, the right one after
                        let b = rd64(base, r + 8);
                        wr64(base, r, (c as u64) << 32);
                        if (a >> 32) as usize >= ro {
                            if sp < cap {
                                *st.add(sp) = b;
                                sp += 1;
                                peak = peak.max(sp);
                            }
                            d = a;
                        } else {
                            let l = (a as u32 & !MARK) as usize;
                            copy_small(dst.add(c), base.add((a >> 32) as usize), l);
                            c += l;
                            if c >= p {
                                c -= p;
                                if c >= p {
                                    break;
                                }
                            }
                            d = b;
                        }
                        continue;
                    }
                }
                if c >= p {
                    c -= p;
                    if c >= p {
                        break;
                    }
                }
                if sp == 0 {
                    break;
                }
                sp -= 1;
                d = *st.add(sp);
            }
        }
        self.first_stack = stack;
        #[cfg(test)]
        {
            self.first_taken += 1;
            self.first_peak = self.first_peak.max(peak);
        }
        // a cycle shorter than the block: the block is its copies
        let mut have = p.min(n);
        while have < n {
            let more = have.min(n - have);
            out.copy_within(..more, have);
            have += more;
        }
        out.truncate(n);
        #[cfg(feature = "owned-writer")]
        {
            use std::sync::atomic::Ordering::Relaxed as R;
            FIRST_PHASES[0].fetch_add(1, R);
            FIRST_PHASES[1].fetch_add(t0.elapsed().as_nanos() as u64, R);
            FIRST_PHASES[2].fetch_max(peak as u64, R);
        }
        let _ = peak;
    }
}
