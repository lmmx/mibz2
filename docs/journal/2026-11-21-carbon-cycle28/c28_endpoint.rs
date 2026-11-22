//! Carbon Cycle 28 artifact (`endpoint.rs` paired with `c27_lean_regburst6.rs` + `master_ref/outq.rs`):
//! - `Elim::replace` on `aw == bw` (`x != y`): replaces `insert(y, x, 0) + cut(y, 0)` (8 packed-`u16` RMWs
//!   with back-to-back store-to-load aliasing on `a[y * 4]`) with a direct 4-RMW in-place splice.
//! - `#[inline(always)]` on `Elim::node` across the 3 initial strip/terminal arena-construction loops.
//! All code is `#![forbid(unsafe_code)]`.
#![forbid(unsafe_code)]
use super::{Band, Counts, Kernel};

trait Links {
    const STRIDE: usize;
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
}

struct Narrow;
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

const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
pub(super) const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

#[derive(Default)]
pub(super) struct Endpoint {
    stack: Vec<u32>,
}

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

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
    #[inline(always)]
    fn bottom(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::BOTTOM]
    }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, x: u32) {
        self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
    }

    #[inline(always)]
    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes;
        self.nodes += 1;
        self.set_width(id, width);
        self.set_word(id, word);
        id
    }

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
    fn replace(&mut self, old: u32, id: u32, side: usize) {
        let (p, n) = (L::prev(self.a, old, side), L::next(self.a, old, side));
        L::set_prev(self.a, id, side, p);
        L::set_next(self.a, id, side, n);
        L::set_next(self.a, p, side, id);
        L::set_prev(self.a, n, side, id);
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
                self.cut(x, 0);
                self.cut(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw < bw {
                self.cut(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                self.set_width(y, bw - w);
            } else if aw > bw {
                self.cut(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            } else {
                self.cut(x, 0);
                self.replace(y, x, 0);
                self.set_word(x, word);
                self.cut(y, 1);
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
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn wide(&self, ns: usize) -> bool {
        Self::nodes_most(ns) > NARROW_NODES
    }

    pub(super) fn arena_words(&self, ns: usize, m: usize) -> usize {
        let stride = if self.wide(ns) {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
    }

    pub(super) fn induced(
        &mut self,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        let most = Self::nodes_most(self.strips.len());
        if self.wide(self.strips.len()) {
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
            bands,
            spelling,
            offsets,
            edges,
            ..
        } = self;
        let ns = strips.len();
        let nodes_words = L::STRIDE * most;
        assert!(edges.len() >= ns + nodes_words);
        let (image_order, arena) = edges.split_at_mut(ns);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0;
        for s in strips.iter() {
            e.node(s.width, u32::from(s.byte) + 1);
            widest = widest.max(s.width);
        }
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0;
        for (i, s) in strips.iter().enumerate() {
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
            let s = strips[i as usize];
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
        stats.open_nodes = e.nodes as usize - 1;
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        let (mut remaining, mut steps) = (n, 0);
        let spirals = widest > CERTIFIED_WIDTH;
        stats.spirals = spirals;
        while !if spirals {
            e.eliminate::<true>(&mut remaining, &mut steps)
        } else {
            e.eliminate::<false>(&mut remaining, &mut steps)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(edges, ns + top + RULE);
            e = Elim {
                a: &mut edges[ns..],
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
