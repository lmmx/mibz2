//! Eliminate the open complement with an auxiliary interval exchange.
//! Its prefix contains terminal coordinates only. All narrow-run wiring is absent.
//!
//! **Where its state lives** (the Wormhole line's memory diet, W1,
//! `docs/journal/2026-11-21-wormhole-cycle5-pigeonhole-chaitin-synthesis.md` on
//! `research/carbon-wormhole`): the elimination's nodes and rules are dead before the kernel's
//! `edges` is first written, so they are kept in `edges`' own buffer, as `u32` words, and
//! `edges` is then written over them. A node is four words when every node id fits in 16 bits
//! (`Narrow`: a side's `prev` and `next` links in one word, then its width and
//! word), six otherwise (`Wide`). The two lists are linked as the nodes are made, with no
//! staging lists. After the elimination a node's side-1 links, read once more by the walk
//! along the bottom, give way to its bottom coordinate.
use super::{begin_inline, Band, Counts, Kernel, Strip, INLINE, LONG};

/// A node's links in the arena
trait Links {
    /// `u32` words a node takes
    const STRIDE: usize;
    const IS_NARROW: bool;
    /// The word that holds a node's bottom coordinate once the elimination is done (one of
    /// its side-1 links, read before it is written)
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32);
}

/// Every node id below 2^16: four words a node, a word per side (`prev` in its low half,
/// `next` in its high half), then the width and the word
struct Narrow;
/// Any node id: six words a node, `prev` and `next` per side, then the width and the word
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
    const IS_NARROW: bool = true;
    const BOTTOM: usize = 1;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] & 0xFFFF
    }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] >> 16
    }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF_0000) | to;
    }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 4..id as usize * 4 + 4];
        (sl[0] >> 16, sl[1], sl[2], sl[3])
    }
}

impl Links for Wide {
    const STRIDE: usize = 6;
    const IS_NARROW: bool = false;
    const BOTTOM: usize = 2;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side]
    }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side + 1]
    }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side] = to;
    }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 6..id as usize * 6 + 6];
        (sl[1], sl[2], sl[4], sl[5])
    }
}

/// The node count up to which `Narrow` links serve: ids 0..=65535
const NARROW_NODES: usize = 1 << 16;

/// Four arena words hold either 15 payload bytes plus a length byte, or
/// two child ids, their combined length and a tagged spelling offset. An
/// unexpanded long rule has the sentinel u32::MAX in the offset field.
const RULE: usize = 4;

// Only live node handles carry a length. Rule children remain plain ids. Each
// ordinary step consumes at least one row and makes at most one rule. A spiral
// of q steps consumes at least q rows and makes at most 2q rules by squaring,
// so every id is at most 256 + 2 * 900_000 and fits in the low 22 bits.
const ID_BITS: u32 = 22;
const ID_MASK: u32 = (1 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;

#[inline(always)]
fn literal(byte: u8) -> u32 {
    (1 << ID_BITS) | u32::from(byte)
}

/// The width certificate's line (the Mechanisms line's cycle 19, Theorem 19.2a,
/// `docs/journal/2026-11-21-carbon-cycle19-unified-synthesis.md` on
/// `research/carbon-word-transport`): no step of the elimination widens a node, so a run of
/// steps that peels one node against the same partner (a spiral) takes fewer steps than the
/// widest node is wide. A block whose nodes are all at most this wide runs the plain loop;
/// a wider one runs the loop that contracts each spiral into one step (`eliminate::<true>`).
pub(super) const CERTIFIED_WIDTH: u32 = 32_768;
/// The rules one contracted spiral may make: a power by squaring (two a bit of the count,
/// which is below 2^20) and the join after it
const SPIRAL_RULES: usize = 2 * 20 + 1;

/// The elimination's scratch kept outside `edges`: the stack of `append`
#[derive(Default)]
pub(super) struct Endpoint {
    stack: Vec<u32>,
    /// (test only) make every elimination take `Wide` links
    #[cfg(test)]
    pub(super) force_wide: bool,
    /// (test only) start every elimination with no room for a rule, so that the arena grows
    #[cfg(test)]
    pub(super) tight: bool,
    /// (test only) make every elimination contract its spirals (`Some(true)`) or run the
    /// plain loop (`Some(false)`), whatever its widths
    #[cfg(test)]
    pub(super) spirals: Option<bool>,
    /// (test only) poison a region of `edges` at a point of the construction (`Clobber`)
    #[cfg(test)]
    pub(super) clobber: Clobber,
}

/// (test only) where the lifetime tests poison `edges` (`mux.rs`): after each region's last
/// read, which the output must not see, or (the controls) before it, which it must
#[cfg(test)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub(super) enum Clobber {
    #[default]
    None,
    /// the image order, once the node lists are made (dead)
    OrderAfterNodes,
    /// the image order, once written and before the node lists are made (live: a control)
    OrderBeforeNodes,
    /// all of `edges`, once the bands are made and before the wiring (dead)
    AllBeforeWire,
    /// the arena, once the elimination is done and before the bands are read off it (live:
    /// a control)
    ArenaBeforeBands,
}

impl Endpoint {
    /// (test only) the scratch it keeps, in bytes (its nodes and rules are in `edges`)
    #[cfg(test)]
    pub(super) fn scratch(&self) -> usize {
        4 * self.stack.capacity()
    }
}

/// The elimination over an arena: `STRIDE` words per node from 0, then `RULE` words per rule
/// from `base` to `top`. The arena is `edges`' buffer, whose length only grows, so neither the
/// nodes nor the edges after them are filled first. (A slice, not the `Vec`: its pointer and
/// length stay in registers across the stores.)
struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

/// The arena to `need` words at least: doubled
#[cold]
#[inline(never)]
fn grow(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

impl<L: Links> Elim<'_, L> {
    #[inline(always)]
    fn width(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2]
    }
    #[inline(always)]
    fn set_width(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w;
    }
    #[inline(always)]
    fn word(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1]
    }
    #[inline(always)]
    fn set_word(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w;
    }
    /// A node's bottom coordinate, once the elimination is done (in a side-1 link's word)
    #[inline(always)]
    fn bottom(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::BOTTOM]
    }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, x: u32) {
        self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
    }

    /// Assign exclusive prefix sums from both ends of side 1. Narrow nodes
    /// reuse both links for the coordinate, so read both links before writing.
    fn assign_bottom(&mut self, rows: u32) {
        let mut first = L::next(self.a, 0, 1);
        let mut last = L::prev(self.a, 0, 1);
        let (mut lo, mut hi) = (0, rows);
        while first != last {
            let next = L::next(self.a, first, 1);
            let prev = L::prev(self.a, last, 1);
            let width = self.width(first);
            hi -= self.width(last);
            self.set_bottom(first, lo);
            self.set_bottom(last, hi);
            lo += width;
            if next == last {
                assert_eq!(lo, hi);
                return;
            }
            first = next;
            last = prev;
        }
        if first != 0 {
            self.set_bottom(first, lo);
            lo += self.width(first);
        }
        assert_eq!(lo, hi);
    }

    /// A new node, its links unset
    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes;
        self.nodes += 1;
        self.set_width(id, width);
        self.set_word(id, word);
        id
    }

    /// Append `id` to a list on `side` whose last node is `*tail` (0: the list is empty, and
    /// `*head` is set)
    #[inline(always)]
    fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
        let (head, tail) = list;
        if *tail == 0 {
            *head = id;
        } else {
            L::set_next(self.a, *tail, side, id);
        }
        L::set_prev(self.a, id, side, *tail);
        *tail = id;
    }

    /// The circular list on `side` through the sentinel: `first`'s nodes, then `second`'s
    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, second] {
            if tail == 0 {
                continue;
            }
            L::set_next(self.a, prev, side, head);
            L::set_prev(self.a, head, side, prev);
            prev = tail;
        }
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }

    #[inline(always)]
    fn cut(&mut self, id: u32, side: usize) {
        let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
        L::set_next(self.a, p, side, n);
        L::set_prev(self.a, n, side, p);
    }

    /// The successor of a tail is the sentinel on this side.
    #[inline(always)]
    fn pop_tail(&mut self, id: u32, side: usize) {
        let prev = L::prev(self.a, id, side);
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }

    #[inline(always)]
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }

    #[inline(always)]
    fn rule(&self, id: u32) -> usize {
        self.base + (id - 65_793) as usize * RULE
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.a[self.rule(h & ID_MASK) + 2]
        }
    }

    /// Low 120 bits in spelling order for any non-empty handle with `grade < 16`.
    /// Both `grade == 1` (literal) and `grade == 2` (inline bigram) decode in 0 memory loads!
    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 2 {
            return u128::from(id);
        }
        let r = self.rule(id);
        let sl = &self.a[r..r + 4];
        u128::from(sl[0])
            | (u128::from(sl[1]) << 32)
            | (u128::from(sl[2]) << 64)
            | (u128::from(sl[3] & 0x00ff_ffff) << 96)
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let left_len = left >> ID_BITS;
        let sum = left_len + (right >> ID_BITS);
        if sum < INLINE as u32 {
            if sum == 2 {
                return (2 << ID_BITS) | (left & 0xFF) | ((right & 0xFF) << 8);
            }
            let id = ((self.top - self.base) / RULE) as u32 + 65_793;
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(sum) << 120);
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = bytes as u32;
            out[1] = (bytes >> 32) as u32;
            out[2] = (bytes >> 64) as u32;
            out[3] = (bytes >> 96) as u32;
            self.top += RULE;
            (sum << ID_BITS) | id
        } else {
            // Since left_len, right_len >= 1, `sum <= SATURATED` implies BOTH `< SATURATED`!
            let len = if sum <= SATURATED {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            let id = ((self.top - self.base) / RULE) as u32 + 65_793;
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.top += RULE;
            (len.min(SATURATED) << ID_BITS) | id
        }
    }

    /// `word` repeated `q` times (`q > 0`), by squaring: at most `SPIRAL_RULES - 1` rules
    fn power(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join(acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = self.join(base, base);
        }
    }

    /// The elimination's steps until `remaining` is 0 (true), or until the arena has no room
    /// for a rule (false: it is grown, and the elimination resumed). With `SPIRALS` (the
    /// blocks the width certificate leaves out), a spiral's steps are taken as one:
    /// - the side-0 node `x` wider than the side-1 node `y` that follows it on side 1: each
    ///   step leaves both lists as they were, prepends `x`'s word to `y`'s and takes `y`'s
    ///   width off `x`'s, while `x` stays the wider. Its `q = (aw - 1) / bw` steps give `y`
    ///   the word `x^q y` and `x` the width `aw - q bw`.
    /// - `x` narrower than `y`, which `x` follows on side 0: each step appends `y`'s word to
    ///   `x`'s and takes `x`'s width off `y`'s, for `q = (bw - 1) / aw` steps: `x y^q`, and
    ///   `y` the width `bw - q aw`.
    ///
    /// The lists, the widths and the words' spellings are exactly the plain loop's (its rules
    /// are fewer); the steps are counted as the plain loop's.
    #[inline(never)]
    fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let sent = &self.a[..2];
            let x = sent[0] & 0xFFFF;
            let y = sent[1] & 0xFFFF;
            let xi = x as usize * 4;
            let yi = y as usize * 4;
            let nx = &self.a[xi..xi + 4];
            let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                self.a[px0 as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px0;
                self.a[px1 as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | px1;
                continue;
            }
            let ny = &self.a[yi..yi + 4];
            let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            let word = self.join(wx, wy);
            if aw <= bw {
                let px = x0 & 0xFFFF;
                self.a[px as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px;
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y | (y_next << 16);
                    nxm[3] = word;
                    let nym = &mut self.a[yi..yi + 4];
                    nym[0] = (y0 & 0xFFFF) | (x << 16);
                    nym[2] = bw - aw;
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                } else {
                    // aw == bw: insert x after y on side 0, cut y from side 0, pop y from tail of side 1
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y_prev | (y_next << 16);
                    nxm[3] = word;
                    let ypi = y_prev as usize * 4;
                    self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    let py = y1 & 0xFFFF;
                    self.a[py as usize * 4 + 1] &= 0xFFFF;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                }
            } else {
                let py = y1 & 0xFFFF;
                self.a[py as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                let x_next = if x == py { 0 } else { x1 >> 16 };
                let nym = &mut self.a[yi..yi + 4];
                nym[1] = x | (x_next << 16);
                nym[3] = word;
                let nxm = &mut self.a[xi..xi + 4];
                nxm[1] = (x1 & 0xFFFF) | (y << 16);
                nxm[2] = aw - bw;
                let xni = x_next as usize * 4 + 1;
                self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
            }
        };
        *remaining = left;
        done
    }

    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        if !SPIRALS && L::IS_NARROW {
            return self.eliminate_narrow_plain(remaining);
        }
        let (mut left, mut taken) = (*remaining, *steps);
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if SPIRALS && x != y {
                if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    taken += q as usize;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    taken += q as usize;
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            if x == y {
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.pop_tail(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.pop_tail(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        (*remaining, *steps) = (left, taken);
        done
    }

    /// An excursion followed by its retained byte. Only long words occupy
    /// spelling, so memoized offsets always refer to permanent bytes.
    fn inline_word(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let height = (word >> ID_BITS) as usize;

        if height < INLINE {
            let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
            (v.to_le_bytes(), (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append(word, stack, spelling);
            spelling.push(byte);
            (super::long_slot(start, spelling.len() as u32), LONG)
        }
    }

    fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                dest.extend_from_slice(&bytes[..g as usize]);
            } else {
                let r = self.rule(h & ID_MASK);
                let tag = self.a[r + 3];
                let rule = &mut self.a[r..r + RULE];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }
}

/// Two direct-mapped tables in the dead edge arena. The first compares both
/// the rule id and the retained byte. The second compares the word itself.
struct Dictionary<'a> {
    pairs: &'a mut [u32; 512],
    contents: &'a mut [u32; 2048],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl Dictionary<'_> {
    #[inline(always)]
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        let g = handle >> ID_BITS;
        let id = handle & ID_MASK;
        let uid = id + 1 + (u32::from(g == 2) << 8);
        let key = (uid << 8) | u32::from(byte);
        let bucket = (key.wrapping_mul(0x9e37_79b9) >> 24) as usize;
        let slot = bucket * 2;
        if self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }
        self.code_miss(e, handle, id, byte, key, slot)
    }

    fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8, key: u32, slot: usize) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
            let entry = v.to_le_bytes();
            let len = (height + 1) as u8;
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let h = h ^ (h >> 30);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            let code = if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == len && self.inline16[old] == entry
            } {
                ent & 0x00FF_FFFF
            } else {
                let c = self.insert(entry, len);
                self.contents[index] = tag | c;
                c
            };
            self.pairs[slot] = key;
            self.pairs[slot + 1] = code;
            return code;
        }
        let start = self.spelling.len();
        let memoized = e.a[e.rule(id) + 3] != u32::MAX;
        e.append(handle, self.stack, self.spelling);
        self.spelling.push(byte);
        let entry = super::long_slot(start as u32, self.spelling.len() as u32);
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = (h as usize) & 2047;
        let ent = self.contents[index];
        let code = if (ent & 0xFF00_0000) == tag && {
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && super::long_word(&self.inline16[old], self.spelling) == bytes
        } {
            if memoized {
                self.spelling.truncate(start);
            } else {
                self.spelling.pop();
            }
            ent & 0x00FF_FFFF
        } else {
            let c = self.insert(entry, LONG);
            self.contents[index] = tag | c;
            c
        };
        self.pairs[slot] = key;
        self.pairs[slot + 1] = code;
        code
    }

    #[inline(always)]
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}

fn spelling_hash_long(bytes: &[u8]) -> u64 {
    let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ part(bytes.len() - 8).rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

impl Kernel {
    /// The most nodes `ns` strips make: a node per strip, at most one gap before each strip
    /// and one after the last, on each side, and the sentinel
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    /// Whether the elimination of `ns` strips takes `Wide` links
    fn wide(&self, ns: usize) -> bool {
        #[cfg(test)]
        let wide = Self::nodes_most(ns) > NARROW_NODES || self.endpoint.force_wide;
        #[cfg(not(test))]
        let wide = Self::nodes_most(ns) > NARROW_NODES;
        wide
    }

    /// The words `edges` is sized to for the induced construction of `ns` strips and `m`
    /// retained rows: packed strips and image order (`4 * ns` words), then the arena for the
    /// nodes and as many rules (the blocks make about one a node; more grow it); and at least
    /// the `m` edges written over them all
    pub(super) fn arena_words(&self, ns: usize, m: usize) -> usize {
        let stride = if self.wide(ns) {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (4 * ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
    }

    pub(super) fn induced(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        let most = Self::nodes_most(ns);
        if self.wide(ns) {
            self.induced_with::<Wide>(ns, n, thin, first, write, stats, most)
        } else {
            self.induced_with::<Narrow>(ns, n, thin, first, write, stats, most)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        stats: &mut Counts,
        most: usize,
    ) -> [usize; 256] {
        let Kernel {
            endpoint,
            bands,
            spelling,
            edges,
            inline16,
            lens,
            ..
        } = self;
        let nodes_words = L::STRIDE * most;
        assert!(edges.len() >= 4 * ns + nodes_words);
        #[cfg(test)]
        if endpoint.tight {
            edges.truncate(4 * ns + nodes_words);
        }
        let (hdr, arena) = edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        #[cfg(test)]
        if endpoint.clobber == Clobber::OrderBeforeNodes {
            image_order.fill(0xA5A5_A5A5);
        }
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        e.nodes = ns as u32 + 1;
        let mut widest = 0u32;
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0u32;
        for i in 0..ns {
            let sw = &mut strips_raw[3 * i..3 * i + 3];
            let from = sw[0];
            let wbyte = sw[2];
            let byte = (wbyte & 0xFF) as u8;
            let width = wbyte >> 8;
            let a = usize::from(byte);
            sw[1] += first[a];
            let dst_idx = write[a] as usize;
            image_order[dst_idx] = i as u32;
            write[a] = dst_idx as u32 + 1;
            let sid = i as u32 + 1;
            e.set_width(sid, width);
            e.set_word(sid, literal(byte));
            widest = widest.max(width);
            if x < from {
                let gw = from - x;
                widest = widest.max(gw);
                let gid = e.node(gw, 0);
                e.append_to(&mut order[0], gid, 0);
                e.append_to(&mut terminals[1], gid, 1);
            }
            e.append_to(&mut order[0], sid, 0);
            x = from + width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }
        x = 0;
        for &i in image_order.iter() {
            let idx = i as usize;
            let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
            if x < s.to {
                widest = widest.max(s.to - x);
                let id = e.node(s.to - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i + 1, 1);
            x = s.to + s.width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[1], id, 1);
            e.append_to(&mut terminals[0], id, 0);
        }
        // the image order is dead from here: the arena may be grown over it, and `edges` is
        // written over both
        #[cfg(test)]
        if endpoint.clobber == Clobber::OrderAfterNodes {
            image_order.fill(0xA5A5_A5A5);
        }
        stats.open_nodes = e.nodes as usize - 1;
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        // the rules follow the nodes made
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        let (mut remaining, mut steps) = (n, 0);
        #[cfg(test)]
        let spirals = endpoint.spirals.unwrap_or(widest > CERTIFIED_WIDTH);
        #[cfg(not(test))]
        let spirals = widest > CERTIFIED_WIDTH;
        stats.spirals = spirals;
        while !if spirals {
            e.eliminate::<true>(&mut remaining, &mut steps)
        } else {
            e.eliminate::<false>(&mut remaining, &mut steps)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(edges, 4 * ns + top + RULE);
            e = Elim {
                a: &mut edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        stats.steps = steps;
        stats.rules = (e.top - e.base) / RULE;
        #[cfg(test)]
        if endpoint.clobber == Clobber::ArenaBeforeBands {
            e.a[..e.top].fill(0xA5A5_A5A5);
        }

        e.assign_bottom(stats.retained as u32);
        // After elimination the strip/order prefix and unused arena tail are
        // available for two direct-mapped caches. Small blocks need neither.
        const PAIR_WORDS: usize = 512;
        const CACHE_WORDS: usize = PAIR_WORDS + 2048;
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let (header, arena) = edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail
        };
        let mut e = Elim::<L> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
        if cache.len() < CACHE_WORDS {
            let mut cursors = [0; 256];
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = bands.len();
                let end = k + thin[a];
                while k < end {
                    let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        let (entry, len) = e.inline_word(node_word, a as u8, &mut endpoint.stack, spelling);
                        let c = lens.len() as u32;
                        inline16.push(entry);
                        lens.push(len);
                        c
                    };
                    bands.push(Band::new(width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                    }
                }
            }
            assert_eq!(id, 0);
            return cursors;
        }
        cache[..CACHE_WORDS].fill(0);
        let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
        let pairs: &mut [u32; 512] = p_sl.try_into().unwrap();
        let contents: &mut [u32; 2048] = c_sl.try_into().unwrap();
        let mut dictionary = Dictionary {
            pairs,
            contents,
            inline16,
            lens,
            spelling,
            stack: &mut endpoint.stack,
        };
        let mut cursors = [0; 256];
        let mut id = L::next(e.a, 0, 0);
        let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
            L::band_node(e.a, id)
        } else {
            (0, 0, 0, 0)
        };
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            let mut last_w = u32::MAX;
            let mut last_c = 0u32;
            while k < end {
                let width = (end - k).min(node_width - in_node);
                assert!(width > 0);
                let dst = node_bottom + in_node;

                let code = if node_word == 0 {
                    a as u32
                } else if node_word == last_w {
                    last_c
                } else {
                    let c = dictionary.code(&mut e, node_word, a as u8);
                    last_w = node_word;
                    last_c = c;
                    c
                };
                bands.push(Band::new(width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = next_id;
                    in_node = 0;
                    if id != 0 {
                        (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                    }
                }
            }
        }
        assert_eq!(id, 0);
        cursors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_collisions_compare_complete_spellings() {
        let mut arena = [0; RULE * 64];
        let mut elim = Elim::<Narrow> {
            a: &mut arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        let (mut inline, mut lens) = (vec![], vec![]);
        begin_inline(0, &mut inline, &mut lens);
        let mut dict = Dictionary {
            pairs: &mut [0; 2048],
            contents: &mut [0; 1],
            inline16: &mut inline,
            lens: &mut lens,
            spelling: &mut vec![],
            stack: &mut vec![],
        };
        for byte in 0..=255 {
            for first in [1, 241] {
                let handle = literal(first);
                let code = dict.code(&mut elim, handle, byte);
                assert_eq!(dict.code(&mut elim, handle, byte), code);
                assert_eq!(dict.inline16[code as usize][..2], [first, byte]);
            }
        }
        let left = elim.power(literal(7), 16);
        let right = elim.power(literal(8), 16);
        let c1 = dict.code(&mut elim, left, 9) as usize;
        let c2 = dict.code(&mut elim, right, 9) as usize;
        assert_ne!(c1, c2);
        assert_eq!(
            super::super::long_word(&dict.inline16[c1], dict.spelling),
            [&[7; 16][..], &[9]].concat()
        );
    }

    #[test]
    fn short_rules_keep_all_fifteen_high_bytes() {
        let mut arena = [0; 32 * RULE];
        let mut e = Elim::<Narrow> {
            a: &mut arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        let mut word = 0;
        for b in 241..=255 {
            word = e.join(literal(b), word);
        }
        let short = e.short_payload(word & ID_MASK).to_le_bytes();
        assert!(short[..15].iter().copied().eq((241..=255).rev()));
        let long = e.join(word, word);
        let mut bytes = Vec::new();
        for _ in 0..2 {
            e.append(long & ID_MASK, &mut vec![], &mut bytes);
        }
        assert_eq!(bytes, short[..15].repeat(4));
    }

    #[test]
    fn bottom_coordinates_at_odd_and_even_meetings() {
        fn check<L: Links>() {
            for count in 0..=7u32 {
                let mut arena = [0; 64];
                let mut e = Elim::<L> {
                    a: &mut arena,
                    base: 0,
                    top: 0,
                    nodes: 0,
                    _links: std::marker::PhantomData,
                };
                e.node(0, 0);
                let mut list = (0, 0);
                for width in 1..=count {
                    let id = e.node(width, 0);
                    e.append_to(&mut list, id, 1);
                }
                e.close(1, list, (0, 0));
                e.assign_bottom(count * (count + 1) / 2);
                for id in 1..=count {
                    assert_eq!(e.bottom(id), (id - 1) * id / 2);
                }
            }
        }
        check::<Narrow>();
        check::<Wide>();
    }

    #[test]
    fn graded_word_lengths_across_saturation() {
        let mut arena = [0; RULE * 40];
        let mut e = Elim::<Narrow> {
            a: &mut arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        let run = e.power(literal(255), SATURATED - 1);
        let mut handle = e.join(0, run);
        let mut spelling = Vec::new();
        for len in SATURATED - 1..=SATURATED + 2 {
            assert_eq!(handle >> ID_BITS, len.min(SATURATED));
            assert_eq!(e.len(handle & ID_MASK), len);
            // Memo offsets refer to this same growing spelling buffer.
            let start = spelling.len();
            e.append(handle & ID_MASK, &mut Vec::new(), &mut spelling);
            assert_eq!(
                &spelling[start..start + (SATURATED - 1) as usize],
                &[255; 1022]
            );
            assert!(spelling[start + 1022..].iter().all(|&b| b == 0));
            handle = e.join(handle, literal(0));
        }
    }

    #[test]
    fn prefix15_crosses_into_memoized_spelling() {
        let mut arena = [0; RULE * 40];
        let mut e = Elim::<Narrow> {
            a: &mut arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        let (mut stack, mut spelling) = (Vec::new(), Vec::new());
        let mut handle = 0;
        for n in 1..=17u8 {
            handle = e.join(handle, literal(n));
            let start = spelling.len();
            let (inline, len) = e.inline_word(handle, 255, &mut stack, &mut spelling);
            let want: Vec<_> = (1..=n).chain([255]).collect();
            if n < 16 {
                assert_eq!(&inline[..usize::from(len)], want);
                assert_eq!(spelling.len(), 0);
            } else {
                assert_eq!(len, LONG);
                assert_eq!(spelling[start..], want);
            }
        }
    }

    #[test]
    fn popping_a_tail_relinks_only_its_side() {
        let mut arena = [0; 3 * Narrow::STRIDE];
        let mut e = Elim::<Narrow> {
            a: &mut arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        for side in 0..2 {
            let mut list = (0, 0);
            e.append_to(&mut list, 1, side);
            e.append_to(&mut list, 2, side);
            e.close(side, list, (0, 0));
        }
        e.pop_tail(2, 1);
        assert_eq!(Narrow::next(e.a, 1, 1), 0);
        assert_eq!(Narrow::prev(e.a, 0, 1), 1);
        assert_eq!(Narrow::next(e.a, 1, 0), 2);
        assert_eq!(Narrow::prev(e.a, 0, 0), 2);
    }
}

impl Endpoint { pub(super) fn scratch_bytes(&self) -> usize { 4 * self.stack.capacity() } }
