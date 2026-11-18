// The one-scan writer (study 5, `docs/journal/2026-11-18-cells-live-reuse.md`, #579; MONODROMY
// record 67; #145 QUEUED 6986629119, CHECKPOINT 6986665685 on #314): the writer of every
// full-period block, eager and lazy (`try_invert`), the shipped `write_cells` its fallback on
// any other. Included in `cells.rs`; the research feature `owned-writer` adds its comparison
// harness (`examples/writer_owned.rs`), the two writers on each block's finished grammar.
//
// **The schedule.** On a full-period block (the cycle is the whole block) whose word is the
// last stored rule, every stored rule is reachable from the word, and each rule's *owner*,
// the parent at its last use, gives it a usable output occurrence:
// - the rules are read once, last to first, each from its position (the word's: where the
//   walk puts it), its children's destinations being the position and the position plus the
//   left child's length, less the period past it (a wrap, on the marked spine only);
// - an arena word or the marked byte is written at its destination;
// - a rule child is *owned* by this parent unless the next stored rule has the very same
//   descriptor on the same side (the child survived the cut): then this is its one position;
// - otherwise the child is copied there later: a command (the child's descriptor and the
//   destination) is stored over the rule records already read, from the high end down, so
//   the commands end in increasing parent order. The next rule's two descriptors are carried
//   in registers: its record may be a command by then (MONODROMY record 67: never re-read it);
// - then the commands are run first to last, each copying its child's bytes from the child's
//   position (filled by then: every unfilled piece of it is a command of an earlier parent).
//
// Any other block (a cycle shorter than the block, or a word that is not the last rule) is
// written by the shipped writer (`write_cells`): the full-period guard costs one test a block.
//
// **Memory safety.** The writer trusts what the shipped writer trusts: the induction's rule
// records (a child names an arena word below the rules, with 16 readable bytes at its offset,
// or an earlier rule's record), and nothing more. Everything it adds is checked locally, so no
// out-of-bounds access depends on the schedule's correctness proof:
// - a rule is visited only at a position below the period: one never assigned (NIL, its table
//   filled first) or out of range writes nothing;
// - its right child's destination is reduced below the period (a position and a length each
//   below it, one subtraction);
// - an arena word is written at a destination below the period, at most 16 bytes, and the
//   output has 32 bytes past the block (`p == n`);
// - a command is stored only over a record already read (`tail > j`, so at most one a parent);
// - a command copies only if its source and destination both lie within the period (a
//   child's NIL position, or a marked child's length with the mark bit, fails the test).
// A violation, which a full-period grammar never makes (the tests check every full-period
// block of every binary column to 10 rows and ternary to 7, eager and lazy), leaves bytes
// unwritten: a wrong block, which the block's CRC rejects, never an access out of bounds.


/// (research only, `owned-writer`) the one-scan writer's own phases, summed over its calls:
/// [full-period blocks, fallback blocks, rules, commands, scan ns, copy ns]
#[cfg(feature = "owned-writer")]
static OWNED_PHASES: [std::sync::atomic::AtomicU64; 6] = [const { std::sync::atomic::AtomicU64::new(0) }; 6];

impl Kernel {
    /// `write_cells`' output by the one-scan schedule above, the shipped writer's on any
    /// block it does not take. The rule records are its scratch: they hold its commands
    /// afterwards (the writer is their last reader).
    #[inline(never)]
    fn write_owned(&mut self, word: u32, len: u32, n: usize, out: &mut Vec<u8>) {
        let p = (len & !MARK) as usize;
        let ro = self.lay.rules;
        let nr = (self.rule_end - ro) / 16;
        // the full-period guard: the whole block is the cycle and its word the last rule
        if p != n || nr == 0 || word as usize != self.rule_end - 16 {
            #[cfg(feature = "owned-writer")]
            OWNED_PHASES[1].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return self.write_cells(word, len, n, out);
        }
        #[cfg(test)]
        {
            self.owned_taken += 1;
        }
        #[cfg(feature = "owned-writer")]
        let t0 = std::time::Instant::now();
        out.resize(n + 32, 0);
        let dst = out.as_mut_ptr();
        let base = self.buf.as_mut_ptr();
        // the positions: in the dead top slots where they fit, else a vector of their own,
        // NIL until assigned (as `write_cells` keeps its first places)
        let pos = self.positions_table(nr);
        // SAFETY: rules lie in [ro, rule_end), 16 bytes each; a rule's descriptors name words
        // written before it (an arena word below `ro` with 16 bytes at its offset, or a rule's
        // record); `pos` holds `nr` entries; every write through `dst` is at a position below
        // the period, at most 16 bytes for a word kept as bytes, and `out` has 32 bytes past
        // `n` (`p == n`); a command's copy is checked against the period on both sides
        unsafe {
            let rule = |w: usize| (rd64(base, w), rd64(base, w + 8));
            // the row's place in the word (as `write_cells` finds it)
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
            *pos.add(nr - 1) = ((p - start) % p) as u32;
            let tail = owned_scan(base, dst, pos, ro, nr, p as u32);
            #[cfg(feature = "owned-writer")]
            let commands = nr - tail;
            #[cfg(feature = "owned-writer")]
            let t1 = std::time::Instant::now();
            owned_copies(base, dst, pos, ro, tail, nr, p);
            #[cfg(feature = "owned-writer")]
            {
                let t2 = std::time::Instant::now();
                OWNED_LAST_COMMANDS.store(commands as u64, std::sync::atomic::Ordering::Relaxed);
                OWNED_PHASES[0].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                OWNED_PHASES[2].fetch_add(nr as u64, std::sync::atomic::Ordering::Relaxed);
                OWNED_PHASES[3].fetch_add(commands as u64, std::sync::atomic::Ordering::Relaxed);
                OWNED_PHASES[4].fetch_add((t1 - t0).as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
                OWNED_PHASES[5].fetch_add((t2 - t1).as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
            }
        }
        out.truncate(n);
    }

    /// The writers' position table for `nr` rules, all NIL: in the dead top slots where they
    /// fit (4-aligned), else in `self.pos` (as `write_cells` places its first places)
    fn positions_table(&mut self, nr: usize) -> *mut u32 {
        let base = self.buf.as_mut_ptr();
        // SAFETY: the top slots' region is [tops, ends[0]) of the buffer; `nr` entries of 4
        // bytes from its first 4-aligned byte fit it when `in_tops`
        unsafe {
            let skip = base.add(self.lay.tops).align_offset(4);
            let in_tops = skip < 4 && skip + 4 * nr <= self.lay.ends[0] - self.lay.tops;
            #[cfg(test)]
            {
                self.pos_in_tops = in_tops;
            }
            if in_tops {
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
            }
        }
    }
}

/// The one scan, last rule to first, from the word's position in `pos` (its last entry): each
/// rule's arena children written, its owned children's positions set, its surviving children's
/// commands stored over the records already read, from the high end down. The lowest command
/// record's index is returned. (A function of its own in the research build, so a profile
/// counts the scan apart from the commands.)
///
/// # Safety
/// `base` holds `nr` rule records of 16 bytes from `ro`, each child an arena word below `ro`
/// (16 readable bytes at its offset) or a record among them; `pos` holds `nr` entries, the last
/// below `p`; `dst` has `p + 32` writable bytes.
#[cfg_attr(feature = "owned-writer", inline(never))]
unsafe fn owned_scan(base: *mut u8, dst: *mut u8, pos: *mut u32, ro: usize, nr: usize, p: u32) -> usize {
    let mut tail = nr;
    let (mut na, mut nb) = (u64::MAX, u64::MAX);
    // SAFETY: as the caller provides (above); a destination is written only below `p`
    unsafe {
        for j in (0..nr).rev() {
            let at = ro + 16 * j;
            let (a, b) = (rd64(base, at), rd64(base, at + 8));
            let q = *pos.add(j);
            if q < p {
                let mut right = q + (a as u32 & !MARK);
                if right >= p {
                    right -= p;
                }
                for (w, later, d) in [(a, na, q), (b, nb, right)] {
                    let r = (w >> 32) as usize;
                    if r < ro {
                        // an arena word, or the marked byte (its length 1 with the mark)
                        copy_small(dst.add(d as usize), base.add(r), (w as u32 & !MARK) as usize);
                    } else if w == later {
                        // the child survived the cut: copied here once its own place is
                        // filled. At most one per parent (a cut has one surviving child), so
                        // `tail` stays above every record still to read; a second from the
                        // same parent, which no induction makes, would be dropped, never
                        // written over a record not yet read
                        if tail > j {
                            tail -= 1;
                            wr64(base, ro + 16 * tail, w);
                            wr64(base, ro + 16 * tail + 8, d as u64);
                        }
                    } else {
                        // this parent owns the child: its one position
                        *pos.add((r - ro) / 16) = d;
                    }
                }
            }
            (na, nb) = (a, b);
        }
    }
    tail
}

/// The commands in records `tail..nr`, in increasing parent order: each child's bytes copied
/// from its position to the command's destination, where both lie within the period `p`
///
/// # Safety
/// As `owned_scan`'s, after it: records `tail..nr` are its commands.
#[cfg_attr(feature = "owned-writer", inline(never))]
unsafe fn owned_copies(base: *mut u8, dst: *mut u8, pos: *mut u32, ro: usize, tail: usize, nr: usize, p: usize) {
    // SAFETY: a command's child names a record (its index below `nr`); a copy is made only
    // where its source and destination lie within the period
    unsafe {
        for i in tail..nr {
            let at = ro + 16 * i;
            let (w, d) = (rd64(base, at), rd64(base, at + 8) as usize);
            let f = *pos.add(((w >> 32) as usize - ro) / 16) as usize;
            let l = w as u32 as usize;
            if f <= p && l <= p - f && d <= p && l <= p - d {
                copy_long(dst.add(d), dst.add(f), l);
            }
        }
    }
}

/// (research only, `owned-writer`) the commands of the one-scan writer's last call
#[cfg(feature = "owned-writer")]
static OWNED_LAST_COMMANDS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// (research only, `owned-writer`) each compared block: [rows, rules, full period, shipped ns,
/// one-scan ns, commands] (the fastest of the reps each)
#[cfg(feature = "owned-writer")]
static OWNED_BLOCKS: std::sync::Mutex<Vec<[u64; 6]>> = std::sync::Mutex::new(Vec::new());

/// (research only, `owned-writer`) the blocks compared since the last call (see `OWNED_BLOCKS`)
#[cfg(feature = "owned-writer")]
#[doc(hidden)]
pub fn owned_blocks() -> Vec<[u64; 6]> {
    std::mem::take(&mut *OWNED_BLOCKS.lock().unwrap())
}

/// (research only, `owned-writer`) the one-scan writer's phases so far: [full-period blocks,
/// fallback blocks, rules, commands, scan ns, copy ns]
#[cfg(feature = "owned-writer")]
#[doc(hidden)]
pub fn owned_phases() -> [u64; 6] {
    std::array::from_fn(|i| OWNED_PHASES[i].load(std::sync::atomic::Ordering::Relaxed))
}

/// (research only, `owned-writer`) which writers the comparison runs after each induction, and
/// how many times: `mode` 0 both (timed, interleaved, the fastest of the reps), 1 the shipped
/// writer alone, 2 the one-scan writer alone, 3 nothing (the baseline the counters subtract);
/// every mode restores the rules before each rep, uncharged by the timing but counted alike
#[cfg(feature = "owned-writer")]
static OWNED_MODE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[cfg(feature = "owned-writer")]
static OWNED_REPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(3);
/// (research only, `owned-writer`) the comparison's totals: [blocks, full-period blocks,
/// blocks whose bytes differ, shipped ns (fastest of the reps, summed), one-scan ns, rules,
/// bytes; the same three times for the fallback blocks: shipped ns, one-scan ns, bytes]
#[cfg(feature = "owned-writer")]
static OWNED_AB: [std::sync::atomic::AtomicU64; 10] = [const { std::sync::atomic::AtomicU64::new(0) }; 10];

/// (research only, `owned-writer`) set the comparison's mode and reps (see `OWNED_MODE`)
#[cfg(feature = "owned-writer")]
#[doc(hidden)]
pub fn owned_mode(mode: usize, reps: usize) {
    OWNED_MODE.store(mode, std::sync::atomic::Ordering::Relaxed);
    OWNED_REPS.store(reps.max(1), std::sync::atomic::Ordering::Relaxed);
}

/// (research only, `owned-writer`) the comparison's totals so far (see `OWNED_AB`)
#[cfg(feature = "owned-writer")]
#[doc(hidden)]
pub fn owned_ab() -> [u64; 10] {
    std::array::from_fn(|i| OWNED_AB[i].load(std::sync::atomic::Ordering::Relaxed))
}

#[cfg(feature = "owned-writer")]
impl Kernel {
    /// (research only) the writers on the block the induction just made, before the shipped
    /// writer runs: each rep restores the rule records the one-scan writer overwrites
    #[inline(never)]
    fn compare_owned(&mut self, word: u32, len: u32, n: usize, out: &mut Vec<u8>) {
        let (mode, reps) = (OWNED_MODE.load(std::sync::atomic::Ordering::Relaxed), OWNED_REPS.load(std::sync::atomic::Ordering::Relaxed));
        let rules = self.lay.rules..self.rule_end;
        let saved = self.buf[rules.clone()].to_vec();
        let full = (len & !MARK) as usize == n && self.rule_end > self.lay.rules
            && word as usize == self.rule_end - 16;
        let mut best = [u64::MAX; 2];
        let mut want = Vec::new();
        let mut differ = false;
        for rep in 0..reps {
            #[allow(clippy::needless_range_loop)]
            for w in 0..2 {
                let run = match mode {
                    0 => true,
                    1 => w == 0,
                    2 => w == 1,
                    _ => false,
                };
                // (uncharged) the output poisoned, so no writer passes on bytes another left,
                // then the rules restored: as an induction leaves them, last written
                out.clear();
                out.resize(n + 32, 0x5A ^ rep as u8 ^ (w as u8) << 7);
                self.buf[rules.clone()].copy_from_slice(&saved);
                if !run {
                    continue;
                }
                let t = std::time::Instant::now();
                if w == 0 {
                    self.write_cells(word, len, n, out);
                } else {
                    self.write_owned(word, len, n, out);
                }
                best[w] = best[w].min(t.elapsed().as_nanos() as u64);
                if mode == 0 {
                    if w == 0 && rep == 0 {
                        want.clear();
                        want.extend_from_slice(out);
                    } else if w == 1 && out[..] != want[..] {
                        differ = true;
                    }
                }
            }
        }
        self.buf[rules].copy_from_slice(&saved);
        let k = if full { 3 } else { 7 };
        OWNED_AB[0].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        OWNED_AB[1].fetch_add(full as u64, std::sync::atomic::Ordering::Relaxed);
        OWNED_AB[2].fetch_add(differ as u64, std::sync::atomic::Ordering::Relaxed);
        for (i, ns) in best.iter().enumerate() {
            if *ns != u64::MAX {
                OWNED_AB[k + i].fetch_add(*ns, std::sync::atomic::Ordering::Relaxed);
            }
        }
        OWNED_BLOCKS.lock().unwrap().push([
            n as u64,
            ((self.rule_end - self.lay.rules) / 16) as u64,
            full as u64,
            best[0],
            best[1],
            if full { OWNED_LAST_COMMANDS.load(std::sync::atomic::Ordering::Relaxed) } else { 0 },
        ]);
        if full {
            OWNED_AB[5].fetch_add(((self.rule_end - self.lay.rules) / 16) as u64, std::sync::atomic::Ordering::Relaxed);
            OWNED_AB[6].fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
        } else {
            OWNED_AB[9].fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
        }
    }
}
