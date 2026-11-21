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
use super::{Band, Counts, Kernel};

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
            self.a[self.rule(w) + 2]
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
        let len = self.len(left) + self.len(right);
        self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);
        self.top += RULE;
        id
    }

    /// The elimination's steps until `remaining` is 0 (true), or until the arena has no room
    /// for a rule (false: it is grown, and the elimination resumed)
    fn eliminate(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            taken += 1;
            if x == y {
                self.cut(x, 0);
                self.cut(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.cut(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.cut(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.cut(y, 1);
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
                let rule = &mut self.a[r..r + RULE];
                if rule[3] == u32::MAX {
                    rule[3] = dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = rule[3] as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }
}

impl Kernel {
    pub(super) fn induced(
        &mut self,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        // a node per strip, at most one gap before each strip and one after the last, on each
        // side, and the sentinel
        let most = 3 * self.strips.len() + 3;
        #[cfg(test)]
        let wide = most > NARROW_NODES || self.endpoint.force_wide;
        #[cfg(not(test))]
        let wide = most > NARROW_NODES;
        if wide {
            self.induced_with::<Wide>(n, thin, stats, most)
        } else {
            self.induced_with::<Narrow>(n, thin, stats, most)
        }
    }

    fn induced_with<L: Links>(
        &mut self,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
        most: usize,
    ) -> [usize; 256] {
        let Kernel {
            endpoint,
            strips,
            image_order,
            bands,
            spelling,
            offsets,
            edges,
            ..
        } = self;
        // `edges` is written only once the elimination is done: its buffer is the arena. It is
        // sized for the nodes, as many rules (the blocks make about one a node; more grow it),
        // and the edges that follow.
        let nodes_words = L::STRIDE * most;
        let need = (nodes_words + RULE * most).max(stats.retained);
        if edges.len() < need {
            // (cold: the first blocks) exact, not doubled; nothing in it is kept
            edges.clear();
            edges.reserve_exact(need);
            edges.resize(need, 0);
        }
        #[cfg(test)]
        if endpoint.tight {
            edges.truncate(nodes_words);
        }
        let mut e = Elim::<L> {
            a: &mut edges[..],
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        for s in strips.iter() {
            e.node(s.width, u32::from(s.byte) + 1);
        }
        // the lists, as the nodes are made: on side 0 the image gaps (its terminals) then the
        // domain order, on side 1 the domain gaps then the image order
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0;
        for (i, s) in strips.iter().enumerate() {
            if x < s.from {
                let id = e.node(s.from - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            e.append_to(&mut order[0], i as u32 + 1, 0);
            x = s.from + s.width;
        }
        if x < n {
            let id = e.node(n - x, 0);
            e.append_to(&mut order[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }
        x = 0;
        for &i in image_order.iter() {
            let s = strips[i as usize];
            if x < s.to {
                let id = e.node(s.to - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i + 1, 1);
            x = s.to + s.width;
        }
        if x < n {
            let id = e.node(n - x, 0);
            e.append_to(&mut order[1], id, 1);
            e.append_to(&mut terminals[0], id, 0);
        }
        stats.open_nodes = e.nodes as usize - 1;
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        // the rules follow the nodes made
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        let (mut remaining, mut steps) = (n, 0);
        while !e.eliminate(&mut remaining, &mut steps) {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(edges, top + RULE);
            e = Elim {
                a: &mut edges[..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        stats.steps = steps;
        stats.rules = (e.top - e.base) / RULE;

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
                // (the port's) every band takes a row at least, so the loop always ends; a wrong
                // construction panics here rather than looping (the mutation check's finding)
                assert!(width > 0);
                let dst = e.bottom(id) + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    let height = e.len(node_word) as usize;
                    stats.words += 1;
                    stats.copies += width as usize;
                    stats.traced += height;
                    stats.thick_area += width as usize * height;
                    stats.unit_work += usize::from(width == 1) * height;
                    stats.longest = stats.longest.max(height);
                    e.append(node_word, &mut endpoint.stack, spelling);
                    spelling.push(a as u8);
                    offsets.push(spelling.len() as u32);
                    254 + offsets.len() as u32
                };
                bands.push(Band {
                    end: k + width,
                    delta: dst.wrapping_sub(k),
                    code,
                });
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        cursors
    }
}
