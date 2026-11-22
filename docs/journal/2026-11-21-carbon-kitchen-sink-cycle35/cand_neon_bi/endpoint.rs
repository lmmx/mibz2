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
use super::{Band, Counts, Kernel, Strip};

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
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32);
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
    #[inline(always)]
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32) {
        a[id as usize * 4 + side] = prev | (next << 16);
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
    #[inline(always)]
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32) {
        a[id as usize * 6 + 2 * side] = prev;
        a[id as usize * 6 + 2 * side + 1] = next;
    }
}

/// The node count up to which `Narrow` links serve: ids 0..=65535
const NARROW_NODES: usize = 1 << 16;

/// A rule's words in the arena: left, right, len, first
const RULE: usize = 4;

/// The width certificate's line (the Mechanisms line's cycle 19, Theorem 19.2a,
/// `docs/journal/2026-11-21-carbon-cycle19-unified-synthesis.md` on
/// `research/carbon-word-transport`): no step of the elimination widens a node, so a run of
/// steps that peels one node against the same partner (a spiral) takes fewer steps than the
/// widest node is wide. A block whose nodes are all at most this wide runs the plain loop;
/// a wider one runs the loop that contracts each spiral into one step (`eliminate::<true>`).
const CERTIFIED_WIDTH: u32 = 32_768;
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

    #[inline(always)]
    fn cut_tail(&mut self, id: u32, side: usize) {
        // `id == prev(0, side)` => `next(id, side) == 0` identically!
        let p = L::prev(self.a, id, side);
        L::set_next(self.a, p, side, 0);
        L::set_prev(self.a, 0, side, p);
    }

    #[inline(always)]
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_links(self.a, id, side, at, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }

    #[inline(always)]
    fn rule(&self, w: u32) -> usize {
        self.base + (w - 257) as usize * RULE
    }



    /// Fused operad fetch: returns `(payload_u128, len)` when `len <= 15` (with `len` in `1..=15`),
    /// or `(0, actual_len)` when `len >= 16`. Single `w <= 256` check and single rule read!
    #[inline(always)]
    fn fetch_word(&self, w: u32) -> (u128, u32) {
        if w <= 256 {
            (u128::from(w - 1), 1)
        } else {
            let r = self.rule(w);
            let sl = &self.a[r..r + 4];
            let r3 = sl[3];
            if (r3 >> 28) == 0 {
                let v = u128::from(sl[0])
                    | (u128::from(sl[1]) << 32)
                    | (u128::from(sl[2]) << 64)
                    | (u128::from(r3 & 0x00FF_FFFF) << 96);
                (v, r3 >> 24)
            } else {
                (0, sl[2])
            }
        }
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
        let (vl, ll) = self.fetch_word(left);
        let (vr, rl) = self.fetch_word(right);
        let len = ll + rl;
        if len <= 15 {
            let v = vl | (vr << (8 * ll)) | ((len as u128) << 120);
            let out = &mut self.a[self.top..self.top + 4];
            out[0] = v as u32;
            out[1] = (v >> 32) as u32;
            out[2] = (v >> 64) as u32;
            out[3] = (v >> 96) as u32;
        } else {
            self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);
        }
        self.top += RULE;
        id
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
            left -= w;
            taken += 1;
            if x == y {
                self.cut_tail(x, 0);
                self.cut_tail(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.cut_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.cut(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.cut_tail(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        (*remaining, *steps) = (left, taken);
        done
    }

    fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule(id);
                let r3 = self.a[r + 3];
                if (r3 >> 28) == 0 {
                    // Stop-leaf! Emit up to 15 pre-assembled operad bytes in one slice copy!
                    let tag = (r3 >> 24) as usize;
                    let v = u128::from(self.a[r])
                        | (u128::from(self.a[r + 1]) << 32)
                        | (u128::from(self.a[r + 2]) << 64)
                        | (u128::from(r3 & 0x00FF_FFFF) << 96);
                    let bytes = v.to_le_bytes();
                    dest.extend_from_slice(&bytes[..tag]);
                } else if r3 == u32::MAX {
                    self.a[r + 3] = 0x8000_0000 | (dest.len() as u32);
                    let right = self.a[r + 1];
                    let left = self.a[r];
                    stack.push(right);
                    stack.push(left);
                } else {
                    let start = (r3 & 0x0FFF_FFFF) as usize;
                    let len = self.a[r + 2] as usize;
                    dest.extend_from_within(start..start + len);
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
    /// retained rows: the strips' image order (`ns` words), then the arena, sized for the
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
            edges,
            inline16,
            lens,
            ..
        } = self;
        let nodes_words = L::STRIDE * most;
        assert!(edges.len() >= 4 * ns + nodes_words);
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
            e.node(s.width, u32::from(s.byte) + 1);
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

        let mut id = L::next(e.a, 0, 1);
        x = 0;
        while id != 0 {
            let next = L::next(e.a, id, 1);
            let w = e.width(id);
            e.set_bottom(id, x);
            x += w;
            id = next;
        }
        assert_eq!(x as usize, stats.retained);
        const PAIR_BITS: u32 = 9;
        const PAIR_LEN: usize = 1 << (PAIR_BITS as usize + 1);
        const STR_BITS: u32 = 11;
        const STR_MASK: usize = (1 << STR_BITS) - 1;
        const STR_LEN: usize = 1 << STR_BITS;
        const HT_WORDS: usize = PAIR_LEN + STR_LEN;

        let (ebase, etop, enodes) = (e.base, e.top, e.nodes);
        let can_dedup = 4 * ns >= HT_WORDS || e.a.len() >= etop + HT_WORDS;
        if !can_dedup {
            // Tiny-block fallback (both 4*ns < 6144 and tail < 6144): direct operad emit into `inline16`!
            let mut cursors = [0; 256];
            id = L::next(e.a, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = bands.len();
                let end = k + thin[a];
                while k < end {
                    let node_width = e.width(id);
                    let node_word = e.word(id);
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = e.bottom(id) + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        stats.words += 1;
                        let (base_v, h_len) = e.fetch_word(node_word);
                        let c = inline16.len() as u32;
                        if h_len <= 15 {
                            let vp = base_v | (u128::from(a as u8) << (8 * h_len));
                            inline16.push(vp.to_le_bytes());
                            lens.push((h_len + 1) as u8);
                        } else {
                            let start = spelling.len();
                            e.append(node_word, &mut endpoint.stack, spelling);
                            spelling.push(a as u8);
                            let end_s = spelling.len();
                            let mut v = [0u8; 16];
                            v[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                            v[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                            inline16.push(v);
                            lens.push(u8::MAX);
                        }
                        c
                    };
                    bands.push(Band::new(k + width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = L::next(e.a, id, 0);
                        in_node = 0;
                    }
                }
            }
            assert_eq!(id, 0);
            return cursors;
        }

        // Reuse dead `edges[0..4*ns]` (hits 6/6 Wikidata blocks!) or dead tail `edges[4*ns+etop..]`:
        let (e_slice, ht_buf) = if 4 * ns >= HT_WORDS {
            let (dead_hdr, rest) = edges.split_at_mut(4 * ns);
            (&mut rest[..etop], &mut dead_hdr[..HT_WORDS])
        } else {
            let (prefix, tail) = edges.split_at_mut(4 * ns + etop);
            (&mut prefix[4 * ns..], &mut tail[..HT_WORDS])
        };
        let mut e = Elim::<L> {
            a: e_slice,
            base: ebase,
            top: etop,
            nodes: enodes,
            _links: std::marker::PhantomData,
        };
        let (pair_ht, str_ht) = ht_buf.split_at_mut(PAIR_LEN);
        pair_ht.fill(0);
        str_ht.fill(0);

        #[inline(always)]
        fn hash_u128(v: u128, wlen: u32) -> u64 {
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(wlen) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            h ^ (h >> 30)
        }

        #[inline(always)]
        fn hash_long(w: &[u8]) -> u64 {
            let n = w.len();
            let lo = u64::from_le_bytes([w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7]]);
            let hi = u64::from_le_bytes([w[n - 8], w[n - 7], w[n - 6], w[n - 5], w[n - 4], w[n - 3], w[n - 2], w[n - 1]]);
            let mid = u64::from_le_bytes([w[n / 2 - 4], w[n / 2 - 3], w[n / 2 - 2], w[n / 2 - 1], w[n / 2], w[n / 2 + 1], w[n / 2 + 2], w[n / 2 + 3]]);
            let h = lo.wrapping_mul(0xBF58_476D_1CE4_E5B9)
                ^ hi.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
                ^ mid.wrapping_mul(0xD6E8_FEB8_6659_FD93)
                ^ (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            h ^ (h >> 30)
        }

        let mut cursors = [0; 256];
        id = L::next(e.a, 0, 0);
        let mut in_node = 0;
        let mut k = 0;
        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            let mut last_w = u32::MAX;
            let mut last_c = 0u32;
            while k < end {
                let node_width = e.width(id);
                let node_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                assert!(width > 0);
                let dst = e.bottom(id) + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else if node_word == last_w {
                    last_c
                } else {
                    let pkey = (node_word << 8) | (a as u32);
                    let pslot = ((pkey.wrapping_mul(0x9E37_79B9)) >> (32 - PAIR_BITS)) as usize * 2;
                    let found_c = if pair_ht[pslot] == pkey {
                        pair_ht[pslot + 1]
                    } else {
                        let (base_v, h_len) = e.fetch_word(node_word);
                        let found_code = if h_len <= 15 {
                            // ZERO-STACK, ZERO-SPELLING, DIRECT-INLINE16 REGISTER PATH (>93% of bands)!
                            let wlen = h_len + 1;
                            let wlen_u8 = wlen as u8;
                            let v = base_v | (u128::from(a as u8) << (8 * h_len));
                            let v_bytes = v.to_le_bytes();
                            let h = hash_u128(v, wlen);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let slot = (h as usize) & STR_MASK;
                            let ent = str_ht[slot];
                            if (ent & 0xFF00_0000) == tag && {
                                let c = (ent & 0x00FF_FFFF) as usize;
                                lens[c] == wlen_u8 && inline16[c] == v_bytes
                            } {
                                ent & 0x00FF_FFFF
                            } else {
                                let c = inline16.len() as u32;
                                inline16.push(v_bytes);
                                lens.push(wlen_u8);
                                str_ht[slot] = tag | c;
                                c
                            }
                        } else {
                            // Cold LONG path (wlen >= 17, <7% of bands): `spelling` holds ONLY deduplicated >=16B expansions!
                            let already_memo = e.a[e.rule(node_word) + 3] != u32::MAX;
                            let start = spelling.len();
                            e.append(node_word, &mut endpoint.stack, spelling);
                            spelling.push(a as u8);
                            let end_s = spelling.len();
                            let wlen_long = end_s - start;
                            let h = hash_long(&spelling[start..end_s]);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let slot = (h as usize) & STR_MASK;
                            let ent = str_ht[slot];
                            if (ent & 0xFF00_0000) == tag && {
                                let c = (ent & 0x00FF_FFFF) as usize;
                                if lens[c] == u8::MAX {
                                    let meta = &inline16[c];
                                    let ps = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                                    let pe = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                                    (pe - ps) == wlen_long && spelling[ps..pe] == spelling[start..end_s]
                                } else {
                                    false
                                }
                            } {
                                if already_memo {
                                    spelling.truncate(start);
                                } else {
                                    spelling.pop();
                                }
                                ent & 0x00FF_FFFF
                            } else {
                                let c = inline16.len() as u32;
                                let mut meta = [0u8; 16];
                                meta[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                                meta[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                                inline16.push(meta);
                                lens.push(u8::MAX);
                                str_ht[slot] = tag | c;
                                c
                            }
                        };
                        pair_ht[pslot] = pkey;
                        pair_ht[pslot + 1] = found_code;
                        found_code
                    };
                    last_w = node_word;
                    last_c = found_c;
                    found_c
                };
                bands.push(Band::new(k + width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        stats.words = inline16.len() - 256;
        cursors
    }
}

impl Endpoint {
    pub(super) fn scratch_bytes(&self) -> usize {
        4 * self.stack.capacity()
    }
}
