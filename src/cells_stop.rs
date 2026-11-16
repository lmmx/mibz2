// Stop and walk: the lazy kernel's induction stopped once it has made ⌈β r⌉ cells, and the
// rest of the block written by walking the stopped exchange with one lane (MONODROMY, #145
// HANDOFF 6916839849; `docs/journal/monodromy/2026-11-16-stop-walk-reopened.md` on #485). Bench
// arms only: the default kernel is unchanged. Included in `cells.rs`.
//
// **The stop.** `induce_lazy` counts its cells down from a budget on every cell already. The
// arms start that counter at ⌈β r⌉ (r the block's letters) where that is below the budget, and
// keep the rest aside. Once the counter is out, each further cell takes one from the reserve,
// in the counter's cold path, until both fronts' letters are whole (the next fresh pair); then
// the induction stops. No word past that moment is made: about a third of the word bytes at
// β = 1.5 on Wikidata (the entry's §0). The test on every cell is the shipped one.
//
// **The stopped exchange.** At that moment no front holds part of a letter, and a front never
// leaves a live letter, so every live letter is one no front has read: a cell, in its two
// slots, or a letter of the block's own, at its stream place. Both orders are read from the
// fronts' links and streams, without changing them, into tables in top order: each letter's
// first point (as a bit, with a rank directory), the offset from its top interval to its
// bottom one, its word's length and its word. T sends x in a letter's top interval to
// x + offset. The domain [0, m) is the letters' rows, summed.
//
// **The walk (the entries' Lemmas 1 and 2).** For a point x of a letter whose word is w,
// place(T x) = place(x) − |w| (mod n): w lies just below x's place. One lane walks T from x_μ,
// the marked letter's one point, whose place the marked word's rules and `SHIFT` give, writing
// each point's word ending at its place, downward. After m points it is back at x_μ, n bytes
// lower: once round the block.
// - *A point's stores reach below its word* (by up to `COPY` bytes), into places the walk
//   has not reached yet, because places only fall. Only near place 0, where the walk wraps to
//   n, and in its last `COPY` bytes, which lie just above where it began, is a word written
//   exactly.
// - *Each point copies its word from one place,* `COPY` bytes whatever its length and 16 at a
//   time past that, top down, so the walk does not branch on the word: a word kept as bytes
//   from the arena; a rule from where the walk first met the letter and expanded it. The two
//   occurrences are disjoint and the first lies above, and the copy reads below the first
//   place only where it writes below this one, which the walk writes over later.
// - *The walk is one chain,* point to point: a rank, a popcount and the letter's offset,
//   which is kept apart from the rest of its record so that the chain's loads stay in the
//   first-level cache. It is latency-bound: about half its time at β = 1.5.
//
// **When it does not apply.** If the walk comes back to x_μ before m points or not n bytes
// lower (the stopped exchange is several cycles: the block is a shorter cycle's copies), or
// anything read does not add up, the block is decoded again by the shipped induction and
// writer. If the induction ends before its stop, the writer is the shipped one.

/// The kernel `CellsVariant<V>` runs for V from `LAZY_STOP14`: lazy letters, stopped after
/// ⌈β r⌉ cells, β = 1.4, 1.5, 1.6
pub(crate) const LAZY_STOP14: u8 = 20;
pub(crate) const LAZY_STOP15: u8 = 21;
pub(crate) const LAZY_STOP16: u8 = 22;

/// Lazy letters, the induction stopped after ⌈1.4 r⌉, ⌈1.5 r⌉ or ⌈1.6 r⌉ cells (r the block's
/// letters) and the rest of the block walked (a bench arm; the same output as `LazyCells`)
#[doc(hidden)]
pub type StopLazyCells14 = CellsVariant<LAZY_STOP14>;
#[doc(hidden)]
pub type StopLazyCells15 = CellsVariant<LAZY_STOP15>;
#[doc(hidden)]
pub type StopLazyCells16 = CellsVariant<LAZY_STOP16>;
/// The same with `ForcedCells`' open admission, for tests and fuzzing
#[doc(hidden)]
pub type ForcedStopLazyCells14 = CellsVariant<LAZY_STOP14, true>;
#[doc(hidden)]
pub type ForcedStopLazyCells15 = CellsVariant<LAZY_STOP15, true>;
#[doc(hidden)]
pub type ForcedStopLazyCells16 = CellsVariant<LAZY_STOP16, true>;

/// Whether a variant's first stage is the lazy letters'
#[inline(always)]
fn lazy_first(v: u8) -> bool {
    v == LAZY || (LAZY_STOP14..=LAZY_STOP16).contains(&v)
}

/// Where `induce_lazy` keeps, in the kernel's buffer, the cell counter's value below which
/// it stops: in the padding below the top slots (`PAD`), which the lazy kernel does not use
const STOP_AT: usize = 8;
const _: () = assert!(STOP_AT + 8 <= PAD);
/// A cell's bottom slot once the bottom order's pass has read it (`tables`): no offset is this
/// (the kernel's layouts stay below `u32::MAX`)
const VISITED: u32 = u32::MAX;

/// Where an induction stopped: both fronts' links, both streams' next places, and what the
/// top stream's byte-to-entry map had reached; the block's rows
#[derive(Default)]
struct Stop {
    active: bool,
    na: usize,
    nb: usize,
    ti: usize,
    kb: usize,
    nt: Vec<u32>,
    n: usize,
}

/// A 64-point word of the rank bitmap: the letters' first points as bits, and the letters
/// that start before it, less one (so that a point's letter is `before` plus a popcount)
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rank {
    bits: u64,
    before: u32,
    _pad: u32,
}

/// A letter, in top order, as the walk writes it at each point: where its word's bytes end,
/// to be copied from (a word kept as bytes: in the arena; else where the walk first wrote it;
/// 0: not yet), and its word's length. (The offset to its bottom interval, which the walk's
/// chain from point to point needs, is apart, in `Walk::dls`, small enough to stay in the
/// first-level cache; its word itself, needed only once, in `Walk::words`.) A tuple, so that
/// a table of them is allocated as zeroed pages (`tables`).
type Rec = (usize, u32);

/// The walk's tables and scratch, kept from block to block
#[derive(Default)]
struct Walk {
    /// a letter of the block's own still in the bottom stream: its bottom interval's end,
    /// counted from the bottom order's right end, by its entry's index (a cell's is kept in
    /// its bottom slot, `tables`)
    j_ent: Vec<u32>,
    /// the rank bitmap over [0, m), and the letters in top order
    rank: Vec<Rank>,
    recs: Vec<Rec>,
    dls: Vec<i32>,
    words: Vec<u64>,
    /// the words being expanded past the frame's depth: (word, end)
    stack: Vec<(u64, usize)>,
    /// a word written exactly, through here
    tmp: Vec<u8>,
}

impl Walk {
    /// The letter whose top interval holds x (x < m)
    #[inline(always)]
    fn letter(&self, x: u32) -> usize {
        // SAFETY: x < m, and `rank` covers [0, m) (`tables`)
        let r = unsafe { *self.rank.get_unchecked((x >> 6) as usize) };
        r.before.wrapping_add((r.bits & (u64::MAX >> (63 - (x & 63)))).count_ones()) as usize
    }
}

impl Kernel {
    /// The stopped exchange read into `walk`'s tables, then walked into `out`. Err: it does
    /// not apply (`out` may be written: the fallback rewrites it whole).
    fn walk_stopped(&mut self, out: &mut Vec<u8>) -> Result<(), ()> {
        let st = std::mem::take(&mut self.stop);
        let n = st.n;
        let mut w = std::mem::take(&mut self.walk);
        let r = self.tables(&st, &mut out[..], &mut w).and_then(|(mu, xmu, m)| {
            // its word ends where the block's place 0 lies after the walk's row's byte and
            // `SHIFT`
            let (j, wmu) = self.marked_offset(w.words[mu])?;
            let p = (wmu + 2 * n - j - SHIFT) % n;
            if out.len() < n + 32 {
                out.resize(n + 32, 0);
            }
            walk(&self.buf, self.lay.rules, &mut w, xmu, p, n, m, out)?;
            out.truncate(n);
            Ok(())
        });
        self.walk = w;
        self.stop = Stop { nt: st.nt, ..Stop::default() };
        r
    }

    /// Both orders read from where the induction stopped, into `w`'s tables in top order.
    /// Returns the marked letter's index, its one point, and the domain's size m.
    fn tables(&self, st: &Stop, out: &mut [u8], w: &mut Walk) -> Result<(usize, usize, usize), ()> {
        let lay = self.lay;
        let rr = self.letters;
        let Lazy { tby, .. } = self.lz;
        let (buf, olen) = (&self.buf[..], out.len());
        // (the entries and the top order's bytes lie below `tby + rr`; a top slot whose key
        // is good lies below the top pool's end, inside the buffer)
        if rr == 0
            || st.nt.len() != 256
            || st.ti > rr
            || st.kb > rr
            || tby != BPAD + 4 * rr
            || olen < tby + rr
            || buf.len() < lay.ends[0] + 4
        {
            return Err(());
        }
        let (bb, base) = (out.as_mut_ptr(), buf.as_ptr());
        let slot_key = |s: usize| -> Result<usize, ()> {
            let i = s.wrapping_sub(lay.tops);
            if i % SLOT != 0 || i / SLOT >= rr {
                return Err(());
            }
            Ok(i / SLOT)
        };
        let first1 = (lay.arena as u64 + BYTES as u64) << 32 | 1;
        // (the tables grow to what a block needs, not to r: a worker's resident memory grows
        // by its live letters, about a tenth of r, and the bottom stream's unread entries)
        if w.j_ent.len() < st.kb {
            w.j_ent.resize(st.kb, 0);
        }
        // the bottom order, right to left: each letter's bottom interval's end, from the right.
        // A letter of the block's own (every entry below `kb` not yet a cell's) keeps it in
        // `j_ent`. A cell keeps it in its bottom slot, which nothing reads after this, as
        // (end, `VISITED`) in place of (link, top slot): the top order's pass finds it there
        // through the top slot's `other`, and a slot read twice is refused
        let (mut s, mut kb, mut end) = (st.nb, st.kb, 0usize);
        // SAFETY (this block): an entry's index is below `rr`, so it and the top order's
        // bytes are read below `tby + rr`; a bottom slot is read and written only below
        // `olen - 8`; a top slot only where its key is good; `j_ent` has `st.kb` places
        unsafe {
            loop {
                let lam;
                if s == 0 {
                    if kb == 0 {
                        break;
                    }
                    kb -= 1;
                    let e = rd32(bb, BPAD + 4 * kb);
                    if e & TAG != 0 {
                        s = (e ^ TAG) as usize;
                        continue;
                    }
                    lam = (e & ((1 << LEN_BITS) - 1)) as usize;
                    end += lam;
                    *w.j_ent.get_unchecked_mut(kb) = end as u32;
                } else {
                    if s + 8 > olen {
                        return Err(());
                    }
                    let (link, ts) = rd_bottom(bb, s);
                    slot_key(ts)?;
                    lam = rd32(base, ts + 4) as usize;
                    end += lam;
                    wr64(bb, s, end as u64 | (VISITED as u64) << 32);
                    s = link;
                }
                if lam == 0 || end >= VISITED as usize {
                    return Err(());
                }
            }
        }
        let m = end;
        if m == 0 {
            return Err(());
        }
        // the top order, right to left, read straight into the walk's tables in reflected
        // coordinates, u = m − 1 − x (the ChatGPT relay's §9, #145 LOG 6923818459): read
        // right to left, the top intervals come in increasing u, so each letter's record is
        // final as it is read. A letter of λ rows, E rows of letters read before it, and its
        // bottom interval's end J from the right has top interval [E, E + λ) and bottom
        // interval [J − λ, J), so T, conjugated, is still the translation u ↦ u + (J − λ − E)
        // on it. The walk visits the same letters at the same places; only its coordinate
        // is reflected
        let nw = m / 64 + 1;
        w.rank.clear();
        w.rank.resize(nw, Rank::default());
        let (mut s, mut ti, mut nl) = (st.na, st.ti, 0usize);
        let (mut e_rows, mut mu) = (0usize, None);
        let mut nt = [0u32; 256];
        nt.copy_from_slice(&st.nt);
        // SAFETY: as above; a top slot is read only where its key is good; the records have
        // more than `nl` places (live letters are `rr` at most: a step never adds one)
        unsafe {
            loop {
                if nl >= w.recs.len() {
                    if nl >= rr {
                        return Err(());
                    }
                    let grow = (2 * nl).max(1024).min(rr);
                    w.recs.resize(grow, (0, 0));
                    w.words.resize(grow, 0);
                    w.dls.resize(grow, 0);
                }
                let (lam, word, jr);
                if s == 0 {
                    if ti == 0 {
                        break;
                    }
                    ti -= 1;
                    let c = *bb.add(tby + ti) as usize;
                    let k = nt[c] as usize;
                    nt[c] = (k as u32).wrapping_sub(1);
                    if k >= rr {
                        return Err(());
                    }
                    let e = rd32(bb, BPAD + 4 * k);
                    if e & TAG != 0 {
                        s = (e ^ TAG) as usize;
                        continue;
                    }
                    // (an entry not yet a cell's lies in the bottom stream's unread part)
                    if k >= st.kb {
                        return Err(());
                    }
                    lam = e & ((1 << LEN_BITS) - 1);
                    word = first1 + ((c as u64) << 32) + (((e & ROW) as u64) << 1);
                    jr = *w.j_ent.get_unchecked(k) as usize;
                } else {
                    slot_key(s)?;
                    let link = rd32(base, s) as usize;
                    let bs = rd_other(base, s);
                    if bs + 8 > olen {
                        return Err(());
                    }
                    let (j, seen) = rd_bottom(bb, bs);
                    if seen != VISITED as usize {
                        return Err(());
                    }
                    lam = rd32(base, s + 4);
                    word = rd64(base, s + 8);
                    jr = j;
                    s = link;
                }
                let lu = lam as usize;
                if lam == 0 || e_rows + lu > m || jr > m || jr < lu || word as u32 & !MARK == 0 {
                    return Err(());
                }
                w.rank[e_rows / 64].bits |= 1 << (e_rows % 64);
                if (word as u32) & MARK != 0 {
                    if lam != 1 || mu.is_some() {
                        return Err(());
                    }
                    mu = Some((nl, e_rows));
                }
                // a word kept as bytes is copied from the arena (16 readable bytes before its
                // end, and the buffer's padding below the arena), a rule from where the walk
                // first writes it
                let (off, len) = ((word >> 32) as usize, word as u32 & !MARK);
                let src = if off < lay.rules { base as usize + off + len as usize } else { 0 };
                *w.recs.get_unchecked_mut(nl) = (src, len);
                *w.dls.get_unchecked_mut(nl) = (jr - lu) as i32 - e_rows as i32;
                *w.words.get_unchecked_mut(nl) = word;
                e_rows += lu;
                nl += 1;
            }
        }
        if e_rows != m || nl == 0 {
            return Err(());
        }
        let mut before = 0u32;
        for r in w.rank.iter_mut() {
            r.before = before.wrapping_sub(1);
            before += r.bits.count_ones();
        }
        mu.map(|(i, x)| (i, x, m)).ok_or(())
    }

    /// (the offset of the walk's row's byte in the marked word, the word's length)
    fn marked_offset(&self, word: u64) -> Result<(usize, usize), ()> {
        let len = (word as u32 & !MARK) as usize;
        let (mut h, mut j) = (word, 0usize);
        for _ in 0..=len {
            let off = (h >> 32) as usize;
            if off < self.lay.rules {
                // the walk's row's own byte
                return if h as u32 == 1 | MARK { Ok((j, len)) } else { Err(()) };
            }
            let a = u64::from_ne_bytes(self.buf.get(off..off + 8).ok_or(())?.try_into().unwrap());
            let b = u64::from_ne_bytes(self.buf.get(off + 8..off + 16).ok_or(())?.try_into().unwrap());
            if a as u32 & MARK != 0 {
                h = a;
            } else if b as u32 & MARK != 0 {
                j += (a as u32 & !MARK) as usize;
                h = b;
            } else {
                return Err(());
            }
        }
        Err(())
    }

}

/// The walk: one lane from x_μ (`xmu`), whose word ends at place `p`, each point's word
/// written ending at its place, downward, m points (`walk_stopped`). Err: it was not one
/// cycle of n bytes. With the CPU's popcount where it has one (a point's letter is a popcount).
#[allow(clippy::too_many_arguments)]
fn walk(buf: &[u8], rules: usize, w: &mut Walk, xmu: usize, p: usize, n: usize, m: usize, out: &mut [u8]) -> Result<(), ()> {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    if std::arch::is_x86_feature_detected!("popcnt") {
        // SAFETY: the CPU has popcnt
        return unsafe { walk_popcnt(buf, rules, w, xmu, p, n, m, out) };
    }
    walk_body(buf, rules, w, xmu, p, n, m, out)
}

#[cfg(all(target_arch = "x86_64", not(miri)))]
#[target_feature(enable = "popcnt")]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
unsafe fn walk_popcnt(buf: &[u8], rules: usize, w: &mut Walk, xmu: usize, p: usize, n: usize, m: usize, out: &mut [u8]) -> Result<(), ()> {
    walk_body(buf, rules, w, xmu, p, n, m, out)
}

/// (the walk's body, `walk`)
///
/// Places are counted on a line that goes once round: v from p + n down to p, the place
/// being v − n while v > n (the first lap, down to place 0) and v after (the second, from
/// place n down to p). A word is written with 16-byte stores where it lies at least 16 bytes
/// above its lap's floor (0 in the first lap, p in the second), else exactly.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn walk_body(buf: &[u8], rules: usize, w: &mut Walk, xmu: usize, p: usize, n: usize, m: usize, out: &mut [u8]) -> Result<(), ()> {
    if p >= n || xmu >= m || out.len() < n + 32 || w.recs.is_empty() {
        return Err(());
    }
    let base = buf.as_ptr();
    let o = out.as_mut_ptr();
    let recs = w.recs.as_mut_ptr();
    let dls = w.dls.as_ptr();
    let mut stack = std::mem::take(&mut w.stack);
    let mut tmp = std::mem::take(&mut w.tmp);
    let mut st: Frame = [(0, 0); DEEP];
    let mut v = p + n;
    // `COPY` above the lap's floor on the line, and the line's offset from the place
    let (mut lim, mut lap) = if v > n { (n + COPY, n) } else { (p + COPY, 0) };
    let mut x = xmu as u32;
    let mut points = 0usize;
    // (T sends [0, m) into itself: `tables` checked each letter's two intervals lie in it)
    let r = loop {
        let i = w.letter(x);
        // SAFETY: i is a letter of the tables (`letter`)
        let (rec, dl) = unsafe { (recs.add(i), *dls.add(i)) };
        let (src, len) = unsafe { *rec };
        let len = len as usize;
        if v >= lim + len {
            let at = v - lap;
            if src != 0 {
                // SAFETY: the word's bytes end at `src`, with `COPY` readable bytes before it
                // and its whole length: a word kept as bytes in the arena (the buffer's
                // padding lies below it), else where the walk first wrote it, at least
                // `COPY` + len above its lap's floor and above this point's word. The stores
                // lie in [at − max(len, COPY) − 15, at), inside [0, n] above the floor. The
                // first `COPY` bytes are read before any is stored
                unsafe {
                    let s = src as *const u8;
                    let (a0, a1) = (ld16(s.sub(16)), ld16(s.sub(32)));
                    st16(o.add(at - 16), a0);
                    st16(o.add(at - 32), a1);
                    let mut k = COPY;
                    while k < len {
                        k += 16;
                        copy16(o.add(at - k), s.sub(k));
                    }
                }
            } else {
                // SAFETY: as above, and a rule's factors are the kernel's words
                unsafe {
                    (*rec).0 = o.add(at) as usize;
                    emit(base, rules, *w.words.get_unchecked(i), at, o, &mut st, &mut stack);
                }
            }
        } else {
            if v < p + len {
                break Err(());
            }
            let at = v - lap;
            // SAFETY: i is a letter of the tables
            let word = unsafe { *w.words.get_unchecked(i) };
            emit_exact(buf, rules, word, len, at, n, out, &mut st, &mut stack, &mut tmp);
            if v - len <= n {
                (lim, lap) = (p + COPY, 0);
            }
        }
        v -= len;
        x = (x as i32 + dl) as u32;
        points += 1;
        if x as usize == xmu {
            break if points == m && v == p { Ok(()) } else { Err(()) };
        }
    };
    w.stack = stack;
    w.tmp = tmp;
    r
}

/// The bytes each point copies whatever its word's length (a word longer copies the rest 16
/// at a time): one fixed copy, so the walk does not branch on the length
const COPY: usize = 32;

/// 16 bytes, loaded and stored
#[inline(always)]
unsafe fn ld16(p: *const u8) -> [u8; 16] {
    // SAFETY: the caller's
    unsafe { (p as *const [u8; 16]).read_unaligned() }
}
#[inline(always)]
unsafe fn st16(p: *mut u8, v: [u8; 16]) {
    // SAFETY: the caller's
    unsafe { (p as *mut [u8; 16]).write_unaligned(v) }
}

/// Pending factors a word's expansion keeps before it spills to the heap
const DEEP: usize = 32;
type Frame = [(u64, usize); DEEP];

/// A word (`h`: offset << 32 | length with the mark) written ending at `at` in `o`: a word
/// kept as bytes by one 16-byte store ending there, a rule by its factors, right to left. The
/// stores may reach 15 bytes below the word. Out of line: the walk's own loop is kept to
/// its registers.
///
/// SAFETY: [at − |word| − 16, at) lies in `o`; every word's bytes or rule lie in the buffer
/// at `base` (words kept as bytes below `rules`, with 16 bytes before each one's end).
#[inline(never)]
unsafe fn emit(base: *const u8, rules: usize, h: u64, at: usize, o: *mut u8, st: &mut Frame, stack: &mut Vec<(u64, usize)>) {
    let off = (h >> 32) as usize;
    if off < rules {
        let len = (h as u32 & !MARK) as usize;
        // SAFETY: the caller's
        unsafe { copy16(o.add(at - 16), base.add(off + len - 16)) };
        return;
    }
    // a rule: its factors right to left, on a stack in `st` (`DEEP` pending factors, the
    // walk's, so not zeroed for each word), and past that on `stack`
    let mut sp = 0usize;
    let (mut h, mut at) = (h, at);
    loop {
        let off = (h >> 32) as usize;
        if off < rules {
            let len = (h as u32 & !MARK) as usize;
            // SAFETY: the caller's
            unsafe { copy16(o.add(at - 16), base.add(off + len - 16)) };
            // the next pending factor: from the frame, else the heap
            if sp > 0 {
                sp -= 1;
                // SAFETY: sp < DEEP
                (h, at) = unsafe { *st.get_unchecked(sp) };
            } else if let Some(v) = stack.pop() {
                (h, at) = v;
            } else {
                return;
            }
        } else {
            // SAFETY: a rule is 16 bytes at its offset
            let (a, b) = unsafe { (rd64(base, off), rd64(base, off + 8)) };
            let lb = (b as u32 & !MARK) as usize;
            // B now (the right factor), A pending
            if sp < DEEP {
                // SAFETY: sp < DEEP
                unsafe { *st.get_unchecked_mut(sp) = (a, at - lb) };
                sp += 1;
            } else {
                // (deeper than the frame: the frame's pending factors, all newer than the
                // heap's, go on top of it in order, so the order is kept; this is rare)
                stack.extend_from_slice(&st[..]);
                sp = 0;
                st[sp] = (a, at - lb);
                sp += 1;
            }
            h = b;
        }
    }
}

/// A word written exactly, ending at `at` (mod n) in `out`: expanded into `tmp` first
#[allow(clippy::too_many_arguments)]
fn emit_exact(
    buf: &[u8],
    rules: usize,
    h: u64,
    len: usize,
    at: usize,
    n: usize,
    out: &mut [u8],
    st: &mut Frame,
    stack: &mut Vec<(u64, usize)>,
    tmp: &mut Vec<u8>,
) {
    if tmp.len() < len + 16 {
        tmp.resize(len + 16, 0);
    }
    // SAFETY: `tmp` is the word's length and 16 bytes before it; the words are the kernel's
    unsafe { emit(buf.as_ptr(), rules, h, len + 16, tmp.as_mut_ptr(), st, stack) };
    let start = (at + 2 * n - len) % n;
    for (j, &v) in tmp[16..16 + len].iter().enumerate() {
        out[(start + j) % n] = v;
    }
}

#[cfg(test)]
mod stop_tests {
    use super::*;

    struct Lcg(u64);
    impl Lcg {
        fn below(&mut self, m: u64) -> u64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (self.0 >> 33) % m
        }
    }

    /// A column's run records (`runs::front`'s format)
    fn records(col: &[u8]) -> Vec<u16> {
        let mut recs: Vec<u16> = Vec::new();
        for &c in col {
            match recs.last_mut() {
                Some(last) if *last >> 8 == c as u16 && *last & 0xFF < 127 => *last += 1,
                _ => recs.push((c as u16) << 8 | 1),
            }
        }
        recs
    }

    /// Arbitrary columns, most of them no BWT: their stopped exchanges are often several
    /// cycles, so the walk refuses and the block is decoded again. Each stop arm must give the
    /// lazy kernel's result, refusal or bytes, on every one; and each path must be taken
    #[test]
    fn arbitrary_columns_as_the_lazy_kernel() {
        let mut rng = Lcg(0x5709);
        let mut lazy = Kernel { variant: LAZY, ..Kernel::default() };
        let mut arms: Vec<Kernel> = [LAZY_STOP14, LAZY_STOP15, LAZY_STOP16]
            .into_iter()
            .map(|v| Kernel { variant: v, ..Kernel::default() })
            .collect();
        for _ in 0..4000 {
            let n = 64 + rng.below(700) as usize;
            let sigma = 2 + rng.below(4);
            let col: Vec<u8> = (0..n).map(|_| b'a' + rng.below(sigma) as u8).collect();
            let p = rng.below(n as u64) as u32;
            let recs = records(&col);
            let mut want = Vec::new();
            let w = lazy.try_invert(&recs, n, p, n, true, &mut want);
            if w.is_ok() {
                want.truncate(n);
            }
            for arm in arms.iter_mut() {
                let mut got = Vec::new();
                let g = arm.try_invert(&recs, n, p, n, true, &mut got);
                assert_eq!(g.is_ok(), w.is_ok(), "variant {} n {n} p {p}", arm.variant);
                if g.is_ok() {
                    got.truncate(n);
                    assert!(got == want, "variant {} n {n} p {p}", arm.variant);
                }
            }
        }
        for arm in &arms {
            eprintln!("variant {}: {} walked, {} decoded again", arm.variant, arm.stopped, arm.stop_fallbacks);
        }
        for arm in &arms {
            assert!(arm.stopped > 0 && arm.stop_fallbacks > 100, "variant {}", arm.variant);
        }
    }
}
