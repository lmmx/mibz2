// Research only (`grammar-replay`): the second ChatGPT relay's grammar replay harness (#145, LOG
// 6924013198, item 1; the relay's §5, §6 and §7). Included in `cells.rs`.
//
// After each lazy induction, before the shipped writer runs, the block's grammar is replayed
// through three writers on the same rules, each timed (the fastest of `reps`, interleaved) and
// its bytes checked against the shipped one's:
// - **W16**, the shipped `write_cells`, on today's 16-byte rules (two 64-bit descriptors);
// - **W12**, 12-byte rules as the 12-byte rules arm stores them (#506, `cells_w12.rs`): two
//   48-bit children, a 27-bit reference (a rule's index, or a word's arena offset) under the
//   length and mark (the descriptor's low half rotated by one), read field by field;
// - **W10**, 10-byte rules (MONODROMY's HANDOFF 6925942307): two 40-bit tagged children, the
//   arena tag and the mark, then a rule's 18-bit index and 20-bit length, or an arena word's
//   22-bit offset from the arena and 5-bit length;
// - **W8**, 8-byte rules: two 32-bit references (the mark, a leaf flag, and a rule's index or
//   a leaf's arena offset and length less one), the rules' lengths rebuilt by a forward pass
//   over the rules before the shipped passes, the pass charged.
// The compact rules are built from the 16-byte ones, uncharged: an induction storing them would
// make them directly. That induction's own cost is not measured here.
//
// From the same capture:
// - **the fit:** whether each writer's tables (W8: lengths and first places, 8 bytes a rule;
//   W16 and W12: first places, 4) fit in the top slots the induction actually touched;
// - **§6, tiny words:** the words the induction made, by length (counted at each cell);
// - **§7, common children:** runs of consecutive rules with the same left, or the same right,
//   child, and what grouping them would save under stated header costs.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed};
use std::sync::Mutex;

/// How many times each writer runs a block (the fastest is kept)
static REPLAY_REPS: AtomicUsize = AtomicUsize::new(3);
/// The words the induction made, by length 0..=16 (unmarked, kept as bytes), 17: marked or
/// longer (kept as rules): of the blocks replayed, and of the induction under way (committed
/// when its block is replayed, dropped when the next induction starts)
static REPLAY_WORDS: [AtomicU64; 18] = [const { AtomicU64::new(0) }; 18];
static REPLAY_PENDING: [AtomicU64; 18] = [const { AtomicU64::new(0) }; 18];
static REPLAY_STATS: Mutex<ReplayStats> = Mutex::new(ReplayStats::new());

/// A §7 grouping's prices: a group's header, and its seek index entry, in bytes
const GROUP_PRICES: [(u64, u64); 2] = [(8, 4), (16, 4)];

/// The replay's totals over the blocks it saw
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct ReplayStats {
    pub blocks: usize,
    /// blocks not replayed (a reference past the compact formats' bits)
    pub skipped: usize,
    pub letters: u64,
    pub bytes: u64,
    pub rules: u64,
    /// the writers' time (ns), the fastest of the reps a block, summed: W16, W12, W8, W10
    pub ns: [u64; 4],
    /// blocks whose bytes differ from W16's: W12, W8, W10
    pub differ: [usize; 3],
    /// the top slots the induction touched (bytes), summed
    pub touched: u64,
    /// blocks whose tables fit in the touched top slots, and the bytes past them, summed: for
    /// 4 bytes a rule (W16, W12) and 8 (W8)
    pub fit: [usize; 2],
    pub past: [u64; 2],
    /// blocks whose tables fit in the top slots' whole room (the writers' own placement)
    pub room: [usize; 2],
    /// §7: rules in runs (of 2 or more) sharing the left child, the right child; runs
    pub run_rules: [u64; 2],
    pub runs: [u64; 2],
    /// §7: the longest run, and runs by length (1..=8, 9: longer), the greedy partition's
    pub run_len: [u64; 10],
    /// §7: bytes saved by grouping the greedy partition's runs, per `GROUP_PRICES`, over
    /// 16-byte rules and over 8-byte rules
    pub saved16: [u64; 2],
    pub saved8: [u64; 2],
}

impl ReplayStats {
    const fn new() -> Self {
        ReplayStats {
            blocks: 0,
            skipped: 0,
            letters: 0,
            bytes: 0,
            rules: 0,
            ns: [0; 4],
            differ: [0; 3],
            touched: 0,
            fit: [0; 2],
            past: [0; 2],
            room: [0; 2],
            run_rules: [0; 2],
            runs: [0; 2],
            run_len: [0; 10],
            saved16: [0; 2],
            saved8: [0; 2],
        }
    }
}

/// The replay's totals so far, and the induction's words by length (0..=16 kept as bytes, 17
/// kept as rules)
#[doc(hidden)]
pub fn replay_stats() -> (ReplayStats, [u64; 18]) {
    let s = REPLAY_STATS.lock().unwrap().clone();
    (s, std::array::from_fn(|i| REPLAY_WORDS[i].load(Relaxed)))
}

/// Each writer's reps a block (at least 1)
#[doc(hidden)]
pub fn replay_reps(reps: usize) {
    REPLAY_REPS.store(reps.max(1), Relaxed);
}

/// (the induction) a word made, of length `len` with the mark
#[inline(always)]
fn replay_word(len: u64) {
    let i = if len <= SMALL as u64 { len as usize } else { 17 };
    REPLAY_PENDING[i].fetch_add(1, Relaxed);
}

/// (the induction's start) the words of an induction not replayed are dropped
#[inline(always)]
fn replay_start() {
    for c in &REPLAY_PENDING {
        c.store(0, Relaxed);
    }
}

/// A W8 reference: the mark, the leaf flag, then a rule's index, or a leaf's length less one
/// (4 bits) and arena offset (26 bits)
const LEAF: u32 = 1 << 30;
const IDX: u32 = LEAF - 1;
const LEAF_AT: u32 = (1 << 26) - 1;
// A W12 child is `cells_w12.rs`'s layout, with its constants (`RULE12`, `C12_REF`, `C12_LM`,
// `C12_LEN`)
/// A W10 child: the arena tag, the mark; a rule's index and length, or an arena word's offset
/// and length
const C10_ARENA: u64 = 1 << 39;
const C10_MARK: u64 = 1 << 38;
const C10_MASK: u64 = (1 << 40) - 1;
const C10_IDX: u64 = (1 << 18) - 1;
const C10_LEN: u64 = (1 << 20) - 1;
const C10_OFF: u64 = (1 << 22) - 1;

#[inline(always)]
fn len8(r: u32, lens: *const u32) -> u32 {
    if r & LEAF != 0 {
        ((r >> 26) & 15) + 1
    } else {
        // SAFETY: a rule's index is below the rules' count, the length table's
        unsafe { *lens.add((r & IDX) as usize) }
    }
}

impl Kernel {
    /// The replay of the block the lazy induction just made (word `word`, marked length `len`):
    /// above. The shipped writer runs after it.
    #[inline(never)]
    fn replay(&mut self, word: u32, len: u32, n: usize, out: &mut Vec<u8>) {
        for (w, c) in REPLAY_WORDS.iter().zip(&REPLAY_PENDING) {
            w.fetch_add(c.swap(0, Relaxed), Relaxed);
        }
        let reps = REPLAY_REPS.load(Relaxed);
        let (ro, arena) = (self.lay.rules, self.lay.arena);
        let nr = (self.rule_end - ro) / 16;
        let base = self.buf.as_ptr();
        // SAFETY: the rules lie below the rules' end
        let rule = |j: usize| unsafe { (rd64(base, ro + 16 * j), rd64(base, ro + 16 * j + 8)) };
        // the compact rules (uncharged)
        let mut ok = (word as usize) >= ro && (word as usize - ro) % 16 == 0 && (len & !MARK) < 1 << 20;
        let mut r8 = vec![0u32; 2 * nr];
        let mut r12 = vec![0u8; 12 * nr + 8];
        let mut r10 = vec![0u8; 10 * nr + 8];
        let c8 = |d: u64| -> Option<u32> {
            let (o, l) = ((d >> 32) as usize, d as u32);
            let (mark, pl) = (l & MARK, l & !MARK);
            if o >= ro {
                ((o - ro) % 16 == 0 && (o - ro) / 16 < nr).then(|| mark | ((o - ro) / 16) as u32)
            } else {
                ((1..=SMALL).contains(&pl) && o >= arena && o - arena <= LEAF_AT as usize)
                    .then(|| mark | LEAF | (pl - 1) << 26 | (o - arena) as u32)
            }
        };
        let c12 = |d: u64| -> Option<u64> {
            let (o, l) = ((d >> 32) as usize, d as u32);
            let r = if o >= ro { (o - ro) / 16 } else { o };
            ((l & !MARK) as u64 <= C12_LEN && (r as u64) <= C12_REF)
                .then_some(r as u64 | (l.rotate_left(1) as u64) << C12_LM)
        };
        let c10 = |d: u64| -> Option<u64> {
            let (o, l) = ((d >> 32) as usize, d as u32);
            let (mark, pl) = (if l & MARK != 0 { C10_MARK } else { 0 }, (l & !MARK) as u64);
            if o >= ro {
                let i = ((o - ro) / 16) as u64;
                (i <= C10_IDX && pl <= C10_LEN).then_some(mark | i << 20 | pl)
            } else {
                let off = o.wrapping_sub(arena) as u64;
                ((1..=SMALL as u64).contains(&pl) && off <= C10_OFF).then_some(C10_ARENA | mark | off << 5 | pl)
            }
        };
        for j in 0..nr {
            let (a, b) = rule(j);
            match (c8(a), c8(b), c12(a), c12(b), c10(a), c10(b)) {
                (Some(x), Some(y), Some(u), Some(v), Some(e), Some(f)) => {
                    (r8[2 * j], r8[2 * j + 1]) = (x, y);
                    // (as the arm stores them: 8 bytes, then 8 more where the first child ends)
                    r12[12 * j..12 * j + 8].copy_from_slice(&u.to_le_bytes());
                    r12[12 * j + 6..12 * j + 14].copy_from_slice(&v.to_le_bytes());
                    r10[10 * j..10 * j + 8].copy_from_slice(&e.to_le_bytes());
                    r10[10 * j + 5..10 * j + 13].copy_from_slice(&f.to_le_bytes());
                }
                _ => ok = false,
            }
        }
        let root = (word as u64) << 32 | len as u64;
        let (root8, root12, root10) = (c8(root), c12(root), c10(root));
        let mut st = REPLAY_STATS.lock().unwrap();
        if !ok || root8.is_none() || root12.is_none() || root10.is_none() {
            st.skipped += 1;
            return;
        }
        drop(st);
        let (root8, root12, root10) = (root8.unwrap(), root12.unwrap(), root10.unwrap());
        // the writers, interleaved, the fastest of the reps each
        let mut best = [u64::MAX; 4];
        let mut want = Vec::new();
        let mut differ = [false; 3];
        for rep in 0..reps {
            for w in 0..4 {
                let t = std::time::Instant::now();
                match w {
                    0 => self.write_cells(word, len, n, out),
                    1 => self.replay_w12(&r12, nr, root12, n, out),
                    2 => self.write_w8(&r8, root8, n, out),
                    _ => self.write_w10(&r10, nr, root10, n, out),
                }
                best[w] = best[w].min(t.elapsed().as_nanos() as u64);
                if w == 0 && rep == 0 {
                    want.clear();
                    want.extend_from_slice(out);
                } else if w > 0 && out[..] != want[..] {
                    differ[w - 1] = true;
                }
            }
        }
        // (`out` ends as W10 left it, its bytes W16's unless counted as differing; the shipped
        // writer runs next)
        // the fit: the top slots the induction touched, and the whole room
        let touched = SLOT * self.lz.peak;
        let room = self.lay.ends[0] - self.lay.tops;
        // §7: runs of consecutive rules sharing a child; the greedy partition
        let mut runs = [0u64; 2];
        let mut run_rules = [0u64; 2];
        for side in 0..2 {
            let mut k = 1u64;
            for j in 1..=nr {
                let same = j < nr && {
                    let (x, y) = (rule(j - 1), rule(j));
                    if side == 0 { x.0 == y.0 } else { x.1 == y.1 }
                };
                if same {
                    k += 1;
                } else {
                    if k >= 2 {
                        runs[side] += 1;
                        run_rules[side] += k;
                    }
                    k = 1;
                }
            }
        }
        let mut run_len = [0u64; 10];
        let mut saved16 = [0u64; 2];
        let mut saved8 = [0u64; 2];
        let mut j = 0;
        while j < nr {
            let run = |side: usize| {
                let mut k = 1;
                while j + k < nr && {
                    let (x, y) = (rule(j + k - 1), rule(j + k));
                    if side == 0 { x.0 == y.0 } else { x.1 == y.1 }
                } {
                    k += 1;
                }
                k
            };
            let k = run(0).max(run(1));
            if k >= 2 {
                run_len[k.min(9)] += 1;
                let k = k as u64;
                for (i, &(head, index)) in GROUP_PRICES.iter().enumerate() {
                    // 16-byte rules: k rules, 16k, as one shared child and k others, 8 + 8k;
                    // 8-byte rules: 8k against 4 + 4k; each with its header and index entry
                    saved16[i] += (16 * k).saturating_sub(8 + 8 * k + head + index);
                    saved8[i] += (8 * k).saturating_sub(4 + 4 * k + head / 2 + index);
                }
            } else {
                run_len[1] += 1;
            }
            j += k;
        }
        let mut st = REPLAY_STATS.lock().unwrap();
        st.blocks += 1;
        st.letters += self.letters as u64;
        st.bytes += n as u64;
        st.rules += nr as u64;
        for (t, b) in st.ns.iter_mut().zip(best) {
            *t += b;
        }
        for (t, d) in st.differ.iter_mut().zip(differ) {
            *t += d as usize;
        }
        for w in 0..2 {
            let need = 4 * (w + 1) * nr + 3;
            st.fit[w] += (need <= touched) as usize;
            st.past[w] += need.saturating_sub(touched) as u64;
            st.room[w] += (need <= room) as usize;
            st.runs[w] += runs[w];
            st.run_rules[w] += run_rules[w];
            st.saved16[w] += saved16[w];
            st.saved8[w] += saved8[w];
        }
        st.touched += touched as u64;
        for (t, k) in st.run_len.iter_mut().zip(run_len) {
            *t += k;
        }
    }

    /// The first places' table, 4 bytes a rule (and as much again for W8's lengths, after
    /// them): in the dead top slots if they hold it, else a vector of the kernel's own
    fn replay_table(&mut self, words: usize) -> *mut u32 {
        let base = self.buf.as_mut_ptr();
        // SAFETY: the top slots lie in the buffer
        let skip = unsafe { base.add(self.lay.tops) }.align_offset(4);
        if skip < 4 && skip + 4 * words <= self.lay.ends[0] - self.lay.tops {
            // SAFETY: as above, `4 words` bytes from the first 4-aligned one
            unsafe { base.add(self.lay.tops + skip) as *mut u32 }
        } else {
            if self.pos.len() < words {
                self.pos = vec![NIL; words];
            }
            self.pos.as_mut_ptr()
        }
    }

    /// W12: `write_cells` on 12-byte rules, as `cells_w12.rs` stores and reads them
    #[inline(never)]
    fn replay_w12(&mut self, r12: &[u8], nr: usize, root: u64, n: usize, out: &mut Vec<u8>) {
        // a child's reference (a rule's index, or an arena offset), its length, its mark
        let at = |c: u64| (c & C12_REF) as usize;
        let ln = |c: u64| (c >> (C12_LM + 1)) as usize;
        let marked = |c: u64| (c >> C12_LM) & 1 != 0;
        let rp = r12.as_ptr();
        // SAFETY: a rule's 12 bytes lie in `r12`, read as 8 at its start and 8 at 4 in (8
        // bytes of slack past the last)
        let rule = |j: usize| unsafe {
            (rd64(rp, RULE12 * j) & ((1 << 48) - 1), rd64(rp, RULE12 * j + 4) >> 16)
        };
        let leaf = |c: u64| at(c);
        // (a plain child is a word kept as bytes if 16 bytes or fewer; on the marked spine, a
        // short marked word is still a rule, and only the row's own byte ends it)
        let short = |c: u64| ln(c) <= SMALL as usize;
        self.write_packed(nr, root, n, out, rule, at, ln, marked, short, |c| ln(c) == 1, leaf)
    }

    /// W10: `write_cells` on 10-byte rules of two tagged 40-bit children (HANDOFF 6925942307)
    #[inline(never)]
    fn write_w10(&mut self, r10: &[u8], nr: usize, root: u64, n: usize, out: &mut Vec<u8>) {
        let arena = self.lay.arena;
        // a child's tag; its reference (a rule's index, or the arena word's offset in the
        // buffer); its length; its mark
        let is_arena = |c: u64| c & C10_ARENA != 0;
        let at = move |c: u64| {
            if c & C10_ARENA != 0 {
                arena + ((c >> 5) & C10_OFF) as usize
            } else {
                ((c >> 20) & C10_IDX) as usize
            }
        };
        let ln = |c: u64| if c & C10_ARENA != 0 { (c & 31) as usize } else { (c & C10_LEN) as usize };
        let marked = |c: u64| c & C10_MARK != 0;
        let rp = r10.as_ptr();
        // SAFETY: a rule's 10 bytes lie in `r10`, read as 8 at its start and 8 at 2 in (8
        // bytes of slack past the last)
        let rule = |j: usize| unsafe { (rd64(rp, 10 * j) & C10_MASK, rd64(rp, 10 * j + 2) >> 24) };
        self.write_packed(nr, root, n, out, rule, at, ln, marked, is_arena, is_arena, at)
    }

    /// `write_cells` on packed rules: `rule(j)` a rule's two children; `at`, `ln`, `marked` a
    /// child's reference, length and mark; `short` whether a plain child is a word kept as
    /// bytes (else a rule, `at` its index); `row` whether the marked spine has reached the
    /// row's byte; `leaf` a short word's offset in the buffer
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn write_packed(
        &mut self,
        nr: usize,
        root: u64,
        n: usize,
        out: &mut Vec<u8>,
        rule: impl Fn(usize) -> (u64, u64),
        at: impl Fn(u64) -> usize,
        ln: impl Fn(u64) -> usize,
        marked: impl Fn(u64) -> bool,
        short: impl Fn(u64) -> bool,
        row: impl Fn(u64) -> bool,
        leaf: impl Fn(u64) -> usize,
    ) {
        let p = ln(root);
        out.resize(n + 32, 0);
        let dst = out.as_mut_ptr();
        let pos = self.replay_table(nr);
        let base = self.buf.as_ptr();
        // SAFETY: as `write_cells`. A rule's children are words made before it: a rule's
        // index below its own, or a word kept as bytes with 16 bytes after it. A position is
        // below the period; `out` has 16 bytes past it.
        unsafe {
            core::ptr::write_bytes(pos, 0xFF, nr);
            // the row's place: down the marked rules to the marked word kept as bytes (the
            // row's single byte)
            let (mut w, mut j) = (root, 0usize);
            while !row(w) {
                let (a, b) = rule(at(w));
                if marked(a) {
                    w = a;
                } else {
                    j += ln(a);
                    w = b;
                }
            }
            let start = (j + SHIFT) % p;
            let (mut w, mut q) = (root, 0usize);
            while !row(w) {
                let (a, b) = rule(at(w));
                let la = ln(a);
                let (plain, pq, next) = if marked(a) { (b, q + la, a) } else { (a, q, b) };
                let d = (pq + p - start) % p;
                if short(plain) {
                    copy_small(dst.add(d), base.add(leaf(plain)), ln(plain));
                } else {
                    let x = pos.add(at(plain));
                    *x = (*x).min(d as u32);
                }
                if !marked(a) {
                    q += la;
                }
                w = next;
            }
            *dst.add((q + p - start) % p) = *base.add(leaf(w));
            for j in (0..nr).rev() {
                let q = *pos.add(j);
                if q == NIL {
                    continue;
                }
                let (a, b) = rule(j);
                if !short(a) {
                    let x = pos.add(at(a));
                    *x = (*x).min(q);
                }
                if !short(b) {
                    let x = pos.add(at(b));
                    *x = (*x).min(q + ln(a) as u32);
                }
            }
            for j in 0..nr {
                let q = *pos.add(j);
                if q == NIL {
                    continue;
                }
                let (a, b) = rule(j);
                let (ll, rr) = (ln(a), ln(b));
                let d = dst.add(q as usize);
                if short(a) {
                    copy_small(d, base.add(leaf(a)), ll);
                } else {
                    let f = *pos.add(at(a));
                    if f != q {
                        copy_long(d, dst.add(f as usize), ll);
                    }
                }
                let d = d.add(ll);
                if short(b) {
                    copy_small(d, base.add(leaf(b)), rr);
                } else {
                    let f = *pos.add(at(b));
                    if f != q + ll as u32 {
                        copy_long(d, dst.add(f as usize), rr);
                    }
                }
            }
            let (mut w, mut q) = (root, 0usize);
            while !row(w) {
                let (a, b) = rule(at(w));
                let la = ln(a);
                let (plain, pq, next) = if marked(a) { (b, q + la, a) } else { (a, q, b) };
                let d = (pq + p - start) % p;
                if !short(plain) {
                    let f = *pos.add(at(plain)) as usize;
                    if f != d {
                        copy_long(dst.add(d), dst.add(f), ln(plain));
                    }
                }
                if !marked(a) {
                    q += la;
                }
                w = next;
            }
        }
        let mut have = p.min(n);
        while have < n {
            let more = have.min(n - have);
            out.copy_within(..more, have);
            have += more;
        }
        out.truncate(n);
    }

    /// W8: `write_cells` on 8-byte rules, their lengths rebuilt first
    #[inline(never)]
    fn write_w8(&mut self, r8: &[u32], root: u32, n: usize, out: &mut Vec<u8>) {
        let nr = r8.len() / 2;
        let pos = self.replay_table(2 * nr);
        // SAFETY: the table has 2 nr entries
        let lens = unsafe { pos.add(nr) };
        let base = self.buf.as_ptr();
        let leaf = self.lay.arena;
        let rp = r8.as_ptr();
        // SAFETY: as `write_cells`; a rule's two references lie in `r8`; a rule's children
        // are rules before it, so its length is rebuilt from lengths already there
        unsafe {
            let rule = |j: usize| (*rp.add(2 * j), *rp.add(2 * j + 1));
            let at = |r: u32| base.add(leaf + (r & LEAF_AT) as usize);
            // the lengths
            for j in 0..nr {
                let (a, b) = rule(j);
                *lens.add(j) = len8(a, lens) + len8(b, lens);
            }
            let p = len8(root, lens) as usize;
            out.resize(n + 32, 0);
            let dst = out.as_mut_ptr();
            core::ptr::write_bytes(pos, 0xFF, nr);
            // the row's place: down the marked rules to the marked leaf
            let (mut w, mut j) = (root, 0usize);
            while w & LEAF == 0 {
                let (a, b) = rule((w & IDX) as usize);
                if a & MARK != 0 {
                    w = a;
                } else {
                    j += len8(a, lens) as usize;
                    w = b;
                }
            }
            let start = (j + SHIFT) % p;
            let (mut w, mut q) = (root, 0usize);
            while w & LEAF == 0 {
                let (a, b) = rule((w & IDX) as usize);
                let la = len8(a, lens) as usize;
                let (plain, at_, next) = if a & MARK != 0 { (b, q + la, a) } else { (a, q, b) };
                let d = (at_ + p - start) % p;
                if plain & LEAF != 0 {
                    copy_small(dst.add(d), at(plain), len8(plain, lens) as usize);
                } else {
                    let x = pos.add((plain & IDX) as usize);
                    *x = (*x).min(d as u32);
                }
                if a & MARK == 0 {
                    q += la;
                }
                w = next;
            }
            *dst.add((q + p - start) % p) = *at(w);
            for j in (0..nr).rev() {
                let q = *pos.add(j);
                if q == NIL {
                    continue;
                }
                let (a, b) = rule(j);
                if a & LEAF == 0 {
                    let x = pos.add((a & IDX) as usize);
                    *x = (*x).min(q);
                }
                if b & LEAF == 0 {
                    let x = pos.add((b & IDX) as usize);
                    *x = (*x).min(q + len8(a, lens));
                }
            }
            for j in 0..nr {
                let q = *pos.add(j);
                if q == NIL {
                    continue;
                }
                let (a, b) = rule(j);
                let (ll, rr) = (len8(a, lens) as usize, len8(b, lens) as usize);
                let d = dst.add(q as usize);
                if a & LEAF != 0 {
                    copy_small(d, at(a), ll);
                } else {
                    let f = *pos.add((a & IDX) as usize);
                    if f != q {
                        copy_long(d, dst.add(f as usize), ll);
                    }
                }
                let d = d.add(ll);
                if b & LEAF != 0 {
                    copy_small(d, at(b), rr);
                } else {
                    let f = *pos.add((b & IDX) as usize);
                    if f != q + ll as u32 {
                        copy_long(d, dst.add(f as usize), rr);
                    }
                }
            }
            let (mut w, mut q) = (root, 0usize);
            while w & LEAF == 0 {
                let (a, b) = rule((w & IDX) as usize);
                let la = len8(a, lens) as usize;
                let (plain, at_, next) = if a & MARK != 0 { (b, q + la, a) } else { (a, q, b) };
                let d = (at_ + p - start) % p;
                if plain & LEAF == 0 {
                    let f = *pos.add((plain & IDX) as usize) as usize;
                    if f != d {
                        copy_long(dst.add(d), dst.add(f), len8(plain, lens) as usize);
                    }
                }
                if a & MARK == 0 {
                    q += la;
                }
                w = next;
            }
            let mut have = p.min(n);
            while have < n {
                let more = have.min(n - have);
                out.copy_within(..more, have);
                have += more;
            }
        }
        out.truncate(n);
    }
}
