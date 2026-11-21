//! The open complement eliminated on `prev`-only tapes (Carbon cycle 9's `R4-DGIM`,
//! `docs/journal/2026-11-20-carbon-cycle9-r4-dgim.md`; `endpoint.rs` is the doubly linked
//! original, kept for the tests and as the fallback switch `Kernel::endpoint_lists`).
//!
//! - **Tapes:** the top chain `P0 · A0` and the bottom chain `P1 · A1` are laid out as
//!   arrays in decreasing elimination order, each slot linked to its predecessor only: a top
//!   slot `{prev, width, word, other}` (16 bytes), a bottom slot `{prev, top}` (8 bytes). The
//!   frontiers walk down the tapes; MONODROMY's rule R4 overwrites the partner slot only after
//!   reading it (cycle 9's Theorem 9.2). Slot 0 is the null; the first 16 are padding.
//! - **No bottom coordinates array:** after the elimination, a backward scan of the bottom
//!   chain stores each surviving node's retained coordinate in its top slot's dead `other`.
//! - **Words inline:** a word of at most 64 bytes is its bytes in an arena (`len << 24 |
//!   offset`), joined by copying; longer ones are binary rules, expanded once as before. A
//!   block's word of at most 16 bytes goes straight into `inline16`, never `spelling`.
//! - **Bands backward:** the surviving top chain is read from its end (the bytes 255 down to
//!   0), then the bands reversed into the order `finish` reads.
//!
//! The FAR = 8 software prefetch of the C is not ported: safe Rust has no prefetch, and the
//! tapes (H thick strips, 75-243 KiB) sit in L2.
//!
//! **Safety:** no `unsafe`; every index bounds-checked, and every width taken asserted
//! positive and within what remains, so a broken construction panics rather than loops.
use super::{Band, Counts, Kernel, INLINE, LONG};

#[derive(Default, Clone, Copy)]
struct Top {
    prev: u32,
    width: u32,
    word: u32,
    other: u32,
}

#[derive(Default, Clone, Copy)]
struct Bot {
    prev: u32,
    top: u32,
}

#[derive(Clone, Copy)]
struct Rule {
    left: u32,
    right: u32,
    len: u32,
    first: u32,
}

/// Slots before the first used one (0 is the null)
const PAD: usize = 16;
/// The longest word kept in the arena
const CAP: u32 = 64;
/// A word that is a rule
const RULE: u32 = 1 << 31;
/// The arena's offsets fit 24 bits
const ARENA_MAX: usize = 0x00FF_F000;

#[derive(Default)]
pub(super) struct R4 {
    tops: Vec<Top>,
    bots: Vec<Bot>,
    strip_top: Vec<u32>,
    arena: Vec<u8>,
    rules: Vec<Rule>,
    stack: Vec<u32>,
    rev: Vec<Band>,
}

impl R4 {
    /// (test only) the scratch it keeps, in bytes
    #[cfg(test)]
    pub(super) fn scratch(&self) -> usize {
        use std::mem::size_of;
        size_of::<Top>() * self.tops.capacity()
            + size_of::<Bot>() * self.bots.capacity()
            + 4 * (self.strip_top.capacity() + self.stack.capacity())
            + self.arena.capacity()
            + size_of::<Rule>() * self.rules.capacity()
            + size_of::<Band>() * self.rev.capacity()
    }

    fn len(&self, w: u32) -> u32 {
        if w == 0 {
            0
        } else if w & RULE == 0 {
            w >> 24
        } else {
            self.rules[(w & !RULE) as usize].len
        }
    }

    fn join(&mut self, a: u32, b: u32) -> u32 {
        if a == 0 {
            return b;
        }
        if b == 0 {
            return a;
        }
        if (a | b) & RULE == 0 {
            let (al, bl) = (a >> 24, b >> 24);
            let tl = al + bl;
            if tl <= CAP && self.arena.len() + tl as usize <= ARENA_MAX {
                let off = self.arena.len() as u32;
                let (ao, bo) = ((a & 0xFF_FFFF) as usize, (b & 0xFF_FFFF) as usize);
                self.arena.extend_from_within(ao..ao + al as usize);
                self.arena.extend_from_within(bo..bo + bl as usize);
                return (tl << 24) | off;
            }
        }
        let id = self.rules.len() as u32;
        let len = self.len(a) + self.len(b);
        self.rules.push(Rule {
            left: a,
            right: b,
            len,
            first: u32::MAX,
        });
        RULE | id
    }

    fn append(&mut self, word: u32, dest: &mut Vec<u8>) {
        self.stack.clear();
        self.stack.push(word);
        while let Some(w) = self.stack.pop() {
            if w & RULE == 0 {
                let off = (w & 0xFF_FFFF) as usize;
                dest.extend_from_slice(&self.arena[off..off + (w >> 24) as usize]);
            } else {
                let rule = &mut self.rules[(w & !RULE) as usize];
                if rule.first == u32::MAX {
                    rule.first = dest.len() as u32;
                    let (l, r) = (rule.left, rule.right);
                    self.stack.push(r);
                    self.stack.push(l);
                } else {
                    let start = rule.first as usize;
                    dest.extend_from_within(start..start + rule.len as usize);
                }
            }
        }
    }
}

impl Kernel {
    /// `induced` on the R4 tapes: the bands, in `finish`'s order, and each byte's first band.
    /// The block's words go into `inline16`/`lens` (and, past 16 bytes, `spelling`).
    pub(super) fn induced_r4(
        &mut self,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        let e = &mut self.r4;
        let ns = self.strips.len();
        let slots = PAD + 3 * ns + 16;
        e.tops.clear();
        e.tops.resize(slots, Top::default());
        e.bots.clear();
        e.bots.resize(slots, Bot::default());
        e.strip_top.clear();
        e.strip_top.resize(ns, 0);
        e.rules.clear();
        e.arena.clear();
        e.arena.extend(0..=255u8);
        let (tops, bots) = (&mut e.tops, &mut e.bots);

        // A0 (and its gaps' bottom slots, P1), in `from` order
        let p0_base = PAD as u32;
        let a0_base = p0_base + ns as u32 + 2;
        let (mut t, mut b) = (a0_base, PAD as u32);
        let mut x = 0;
        let gap = |tops: &mut Vec<Top>, bots: &mut Vec<Bot>, t: &mut u32, b: &mut u32, w| {
            tops[*t as usize] = Top {
                prev: *t - 1,
                width: w,
                word: 0,
                other: *b,
            };
            bots[*b as usize] = Bot {
                prev: *b - 1,
                top: *t,
            };
            *t += 1;
            *b += 1;
        };
        for (i, s) in self.strips.iter().enumerate() {
            if x < s.from {
                gap(tops, bots, &mut t, &mut b, s.from - x);
            }
            e.strip_top[i] = t;
            tops[t as usize] = Top {
                prev: t - 1,
                width: s.width,
                word: (1 << 24) | u32::from(s.byte),
                other: 0,
            };
            t += 1;
            x = s.from + s.width;
        }
        if x < n {
            gap(tops, bots, &mut t, &mut b, n - x);
        }
        let a0_end = t;
        // P0 (and its bottom slots) and the strips' bottom slots, A1, in image order
        let mut p = p0_base;
        x = 0;
        for &i in &self.image_order {
            let s = self.strips[i as usize];
            if x < s.to {
                gap(tops, bots, &mut p, &mut b, s.to - x);
            }
            let st = e.strip_top[i as usize];
            tops[st as usize].other = b;
            bots[b as usize] = Bot {
                prev: b - 1,
                top: st,
            };
            b += 1;
            x = s.to + s.width;
        }
        if x < n {
            gap(tops, bots, &mut p, &mut b, n - x);
        }
        assert!(p < a0_base && a0_end > a0_base && b > PAD as u32);
        if p > p0_base {
            tops[p0_base as usize].prev = 0;
        }
        bots[PAD].prev = 0;
        tops[a0_base as usize].prev = if p > p0_base { p - 1 } else { 0 };
        stats.open_nodes = (p - p0_base + a0_end - a0_base) as usize;

        // the elimination, right to left
        let (mut sa, mut sb) = (a0_end - 1, b - 1);
        let mut remaining = n;
        let mut steps = 0;
        let take = |remaining: &mut u32, w: u32| {
            assert!(w > 0 && w <= *remaining);
            *remaining -= w;
        };
        while remaining > 0 {
            let ta = e.tops[sa as usize];
            let (mut na, mut ra, mut aw, mut bs) = (ta.prev, ta.width, ta.word, ta.other);
            let bb = e.bots[sb as usize];
            let (mut nb, mut ts) = (bb.prev, bb.top);
            let tb = e.tops[ts as usize];
            let (mut rb, mut bw) = (tb.width, tb.word);
            let (mut fa, mut fb) = (sa, sb);
            steps += 1;
            if bs == sb {
                take(&mut remaining, ra);
                (sa, sb) = (na, nb);
                continue;
            }
            loop {
                let wd = e.join(aw, bw);
                if ra == rb {
                    take(&mut remaining, ra);
                    let top = &mut e.tops[ts as usize];
                    (top.width, top.word, top.other) = (ra, wd, bs);
                    e.bots[bs as usize].top = ts;
                    (sa, sb) = (na, nb);
                    break;
                } else if ra > rb {
                    take(&mut remaining, rb);
                    let top = &mut e.tops[ts as usize];
                    (top.width, top.word, top.other) = (rb, wd, bs);
                    e.bots[fb as usize].prev = e.bots[bs as usize].prev;
                    e.bots[bs as usize] = Bot { prev: fb, top: ts };
                    bs = fb;
                    ra -= rb;
                    fb = nb;
                    let bn = e.bots[nb as usize];
                    (nb, ts) = (bn.prev, bn.top);
                    let tn = e.tops[ts as usize];
                    (rb, bw) = (tn.width, tn.word);
                } else {
                    take(&mut remaining, ra);
                    e.bots[bs as usize].top = ts;
                    e.tops[fa as usize].prev = e.tops[ts as usize].prev;
                    e.tops[ts as usize] = Top {
                        prev: fa,
                        width: ra,
                        word: wd,
                        other: bs,
                    };
                    ts = fa;
                    rb -= ra;
                    fa = na;
                    let tn = e.tops[na as usize];
                    (na, ra, aw, bs) = (tn.prev, tn.width, tn.word, tn.other);
                }
                steps += 1;
            }
        }
        stats.steps = steps;
        stats.rules = e.rules.len();

        // the surviving nodes' retained coordinates, into their top slots' `other`
        let m = stats.retained as u32;
        let mut xb = m;
        while sb != 0 {
            let s = e.bots[sb as usize];
            let w = e.tops[s.top as usize].width;
            assert!(w <= xb);
            xb -= w;
            e.tops[s.top as usize].other = xb;
            sb = s.prev;
        }
        assert_eq!(xb, 0);

        // the literals' inline words, once; the block's words from 256
        if self.inline16.len() < 256 {
            self.inline16.clear();
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.resize(256, 1);
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);

        // the bands, from the end of the surviving top chain
        let mut thin_start = [0u32; 256];
        let mut acc = 0;
        for a in 0..256 {
            thin_start[a] = acc;
            acc += thin[a];
        }
        let mut ends = [0usize; 256];
        e.rev.clear();
        let mut rem = if sa != 0 {
            e.tops[sa as usize].width
        } else {
            0
        };
        let mut kc = m;
        for a in (0..256).rev() {
            let start = thin_start[a];
            while kc > start {
                let nd = e.tops[sa as usize];
                let width = (kc - start).min(rem);
                assert!(width > 0);
                rem -= width;
                let end = kc;
                kc -= width;
                let dst = nd.other + rem;
                let code = if nd.word == 0 {
                    a as u32
                } else {
                    let o0 = self.spelling.len();
                    e.append(nd.word, &mut self.spelling);
                    self.spelling.push(a as u8);
                    let height = self.spelling.len() - o0;
                    stats.words += 1;
                    stats.copies += width as usize;
                    stats.traced += height;
                    stats.thick_area += width as usize * height;
                    stats.unit_work += usize::from(width == 1) * height;
                    stats.longest = stats.longest.max(height);
                    let mut v = [0; INLINE];
                    if nd.word & RULE == 0 && height <= INLINE {
                        v[..height].copy_from_slice(&self.spelling[o0..]);
                        self.spelling.truncate(o0);
                        self.lens.push(height as u8);
                    } else {
                        self.lens.push(LONG);
                    }
                    self.inline16.push(v);
                    self.offsets.push(self.spelling.len() as u32);
                    254 + self.offsets.len() as u32
                };
                e.rev.push(Band {
                    end,
                    delta: dst.wrapping_sub(kc),
                    code,
                });
                if rem == 0 {
                    sa = nd.prev;
                    rem = if sa != 0 {
                        e.tops[sa as usize].width
                    } else {
                        0
                    };
                }
            }
            ends[a] = e.rev.len();
        }
        assert_eq!(sa, 0);
        // reversed: byte a's bands were rev[ends[a + 1]..ends[a]] (ends[256] = 0)
        let total = e.rev.len();
        self.bands.extend(e.rev.iter().rev());
        let mut cursors = [0usize; 256];
        for a in 0..256 {
            cursors[a] = total - ends[a];
        }
        cursors
    }
}
