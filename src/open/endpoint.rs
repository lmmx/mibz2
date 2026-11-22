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
    /// The word that holds a node's bottom coordinate once the elimination is done (one of
    /// its side-1 links, read before it is written)
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
}

/// Every node id below 2^16: four words a node, a word per side (`prev` in its low half,
/// `next` in its high half), then the width and the word
struct Narrow;
/// Any node id: six words a node, `prev` and `next` per side, then the width and the word
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
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
}

impl Links for Wide {
    const STRIDE: usize = 6;
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
}

/// The node count up to which `Narrow` links serve: ids 0..=65535
const NARROW_NODES: usize = 1 << 16;

/// A rule's words in the arena: left, right, len, first
const RULE: usize = 4;

// Only live node handles carry a length. Rule children remain plain ids. Each
// ordinary step consumes at least one row and makes at most one rule. A spiral
// of q steps consumes at least q rows and makes at most 2q rules by squaring,
// so every id is at most 256 + 2 * 900_000 and fits in the low 22 bits.
const ID_BITS: u32 = 22;
const ID_MASK: u32 = (1 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;

fn literal(byte: u8) -> u32 {
    (1 << ID_BITS) | (u32::from(byte) + 1)
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
    fn rule(&self, w: u32) -> usize {
        self.base + (w - 257) as usize * RULE
    }

    #[inline(always)]
    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            let r = self.rule(w);
            let tag = self.a[r + 3];
            if tag >> 28 == 0 {
                tag >> 24
            } else {
                self.a[r + 2]
            }
        }
    }

    /// The low 120 bits are in spelling order. The length byte is metadata.
    #[inline(always)]
    fn short_payload(&self, id: u32) -> u128 {
        if id <= 256 {
            return u128::from(id - 1);
        }
        let r = self.rule(id);
        u128::from(self.a[r])
            | (u128::from(self.a[r + 1]) << 32)
            | (u128::from(self.a[r + 2]) << 64)
            | (u128::from(self.a[r + 3] & 0x00ff_ffff) << 96)
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let (left_len, right_len) = (left >> ID_BITS, right >> ID_BITS);
        let (left, right) = (left & ID_MASK, right & ID_MASK);
        let len = if left_len < SATURATED && right_len < SATURATED {
            left_len + right_len
        } else {
            self.len(left) + self.len(right)
        };
        let rule = if len < INLINE as u32 {
            let bytes = self.short_payload(left)
                | (self.short_payload(right) << (8 * left_len))
                | (u128::from(len) << 120);
            [
                bytes as u32,
                (bytes >> 32) as u32,
                (bytes >> 64) as u32,
                (bytes >> 96) as u32,
            ]
        } else {
            [left, right, len, u32::MAX]
        };
        self.a[self.top..self.top + RULE].copy_from_slice(&rule);
        self.top += RULE;
        (len.min(SATURATED) << ID_BITS) | id
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
    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
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
            taken += 1;
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
        let id = word & ID_MASK;
        let height = (word >> ID_BITS) as usize;
        let mut bytes = [0; INLINE];
        if height < INLINE {
            bytes = self.short_payload(id).to_le_bytes();
            bytes[height] = byte;
            (bytes, (height + 1) as u8)
        } else {
            self.append(id, stack, spelling);
            spelling.push(byte);
            (bytes, LONG)
        }
    }

    fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule(id);
                let tag = self.a[r + 3];
                if tag >> 28 == 0 {
                    let bytes = self.short_payload(id).to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                    continue;
                }
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
    /// retained rows: strip records and their image order (`4 * ns` words), then the arena for the
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
        stats: &mut Counts,
    ) -> [usize; 256] {
        let most = Self::nodes_most(ns);
        if self.wide(ns) {
            self.induced_with::<Wide>(ns, n, thin, stats, most)
        } else {
            self.induced_with::<Narrow>(ns, n, thin, stats, most)
        }
    }

    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
        most: usize,
    ) -> [usize; 256] {
        let Kernel {
            endpoint,
            bands,
            spelling,
            offsets,
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
        // the widest node, for the width certificate (`CERTIFIED_WIDTH`)
        let mut widest = 0;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            e.node(s.width, literal(s.byte));
            widest = widest.max(s.width);
        }
        // the lists, as the nodes are made: on side 0 the image gaps (its terminals) then the
        // domain order, on side 1 the domain gaps then the image order
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            if x < s.from {
                widest = widest.max(s.from - x);
                let id = e.node(s.from - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            e.append_to(&mut order[0], i as u32 + 1, 0);
            x = s.from + s.width;
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
        let mut cursors = [0; 256];
        let mut id = L::next(e.a, 0, 0);
        let mut in_node = 0;
        let mut k = 0;
        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            while k < end {
                let node_width = e.width(id);
                let node_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                // (the port's) every band takes a row at least, so the loop always ends; a wrong
                // construction panics here rather than looping (the mutation check's finding)
                assert!(width > 0);
                let dst = e.bottom(id) + in_node;
                if node_word != 0 {
                    let grade = node_word >> ID_BITS;
                    let height = if grade == SATURATED {
                        e.len(node_word & ID_MASK)
                    } else {
                        grade
                    } as usize;
                    stats.words += 1;
                    stats.copies += width as usize;
                    stats.traced += height;
                    stats.thick_area += width as usize * height;
                    stats.unit_work += usize::from(width == 1) * height;
                    stats.longest = stats.longest.max(height);
                }
                bands.push(Band::new(k + width, dst, node_word & ID_MASK));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        // The count is now exact. Use the handles in the bands to build both
        // dictionary representations before wiring, without growing spare slots.
        begin_inline(stats.words, inline16, lens);
        for (byte, &start) in cursors.iter().enumerate() {
            let end = cursors.get(byte + 1).copied().unwrap_or(bands.len());
            for band in &mut bands[start..end] {
                let id = band.code();
                let code = if id == 0 {
                    byte as u32
                } else {
                    let code = lens.len() as u32;
                    let (bytes, len) = e.inline_word(
                        (e.len(id).min(SATURATED) << ID_BITS) | id,
                        byte as u8,
                        &mut endpoint.stack,
                        spelling,
                    );
                    inline16.push(bytes);
                    lens.push(len);
                    offsets.push(spelling.len() as u32);
                    code
                };
                *band = Band::new(band.end(), band.dst(), code);
            }
        }
        cursors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
