#![forbid(unsafe_code)]
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
    #[cfg(feature = "a6b-endpoint-redesign")]
    redesign: redesign::State,
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
        #[cfg(feature = "a6b-endpoint-redesign")]
        {
            4 * self.stack.capacity() + self.redesign.scratch()
        }
        #[cfg(not(feature = "a6b-endpoint-redesign"))]
        {
            4 * self.stack.capacity()
        }
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

        if height < INLINE {
            let mut bytes = self.short_payload(id).to_le_bytes();
            bytes[height] = byte;
            (bytes, (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append(id, stack, spelling);
            spelling.push(byte);
            (super::long_slot(start, spelling.len() as u32), LONG)
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

/// Two direct-mapped tables in the dead edge arena. The first compares both
/// the rule id and the retained byte. The second compares the word itself.
struct Dictionary<'a> {
    pairs: &'a mut [u32],
    contents: &'a mut [u32],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl Dictionary<'_> {
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        if handle == 0 {
            return u32::from(byte);
        }
        let id = handle & ID_MASK;
        let key = (id << 8) | u32::from(byte);
        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
        if !self.pairs.is_empty() && self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }

        let start = self.spelling.len();
        let memoized = handle >> ID_BITS >= INLINE as u32 && e.a[e.rule(id) + 3] != u32::MAX;
        let (entry, len) = e.inline_word(handle, byte, self.stack, self.spelling);
        if self.contents.is_empty() {
            return self.insert(entry, len);
        }
        let bytes = if len == LONG {
            &self.spelling[start..]
        } else {
            &entry[..len as usize]
        };
        let index = spelling_hash(&entry, len, bytes) as usize & (self.contents.len() - 1);
        let old = self.contents[index] as usize;
        let same = old != 0
            && self.lens[old] == len
            && if len == LONG {
                super::long_word(&self.inline16[old], self.spelling) == bytes
            } else {
                self.inline16[old] == entry
            };
        let code = if same {
            // Fresh child memo entries may point into this expansion. Preserve
            // those bytes until the arena is retired.
            if len == LONG {
                if memoized {
                    self.spelling.truncate(start);
                } else {
                    self.spelling.pop();
                }
            }
            old as u32
        } else {
            let code = self.insert(entry, len);
            self.contents[index] = code;
            code
        };
        self.pairs[slot] = key;
        self.pairs[slot + 1] = code;
        code
    }

    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}

/// A cache collision only evicts a candidate. Sharing requires equal length and bytes.
fn spelling_hash(entry: &[u8; INLINE], len: u8, bytes: &[u8]) -> u64 {
    let (low, high, middle) = if len == LONG {
        let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        (part(0), part(bytes.len() - 8), part(bytes.len() / 2 - 4))
    } else {
        let value = u128::from_le_bytes(*entry);
        (value as u64, (value >> 64) as u64, 0)
    };
    let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ middle.wrapping_mul(0xD6E8_FEB8_6659_FD93)
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
    pub(super) fn wide(&self, ns: usize) -> bool {
        #[cfg(test)]
        let wide = Self::nodes_most(ns) > NARROW_NODES || self.endpoint.force_wide;
        #[cfg(not(test))]
        let wide = Self::nodes_most(ns) > NARROW_NODES;
        wide
    }

    /// Whether the elimination of `ns` strips uses the zero-extra-RSS edges-overlaid AoS fast path.
    pub(super) fn use_overlay_aos(&self, ns: usize) -> bool {
        #[cfg(feature = "a6b-endpoint-redesign")]
        {
            let _ = ns;
            false
        }
        #[cfg(not(feature = "a6b-endpoint-redesign"))]
        {
            if self.wide(ns) || ns < 64 {
                return false;
            }
            #[cfg(test)]
            if self.endpoint.tight || self.endpoint.clobber != Clobber::None {
                return false;
            }
            true
        }
    }

    /// The words `edges` is sized to for the induced construction of `ns` strips and `m`
    /// retained rows: packed strips and image order (`4 * ns` words), then the arena for the
    /// nodes and as many rules (the blocks make about one a node; more grow it); and at least
    /// the `m` edges written over them all
    pub(super) fn arena_words(&self, ns: usize, m: usize) -> usize {
        #[cfg(feature = "a6b-endpoint-redesign")]
        if !self.wide(ns) {
            return (3 * ns).max(m);
        }
        if self.use_overlay_aos(ns) {
            return (5 * ns).max(4096).max(m);
        }
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
        // After elimination the strip/order prefix and unused arena tail are
        // available for two direct-mapped caches. Small blocks need neither.
        const PAIR_WORDS: usize = 2 * 1024;
        const CACHE_WORDS: usize = PAIR_WORDS + 4096;
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let (header, arena) = edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail
        };
        let (pairs, contents) = if cache.len() >= CACHE_WORDS {
            cache[..CACHE_WORDS].fill(0);
            cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS)
        } else {
            (&mut [][..], &mut [][..])
        };
        let mut e = Elim::<L> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
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
                let code = dictionary.code(&mut e, node_word, a as u8);
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
        cursors
    }
}

#[cfg(not(feature = "a6b-endpoint-redesign"))]
mod overlay_aos {
    use super::super::{begin_inline, long_slot, long_word, Band, Counts, Kernel, INLINE, LONG};
    use super::{spelling_hash, CERTIFIED_WIDTH, RULE, SPIRAL_RULES};

    const U64_PAYLOAD_MASK: u64 = (1u64 << 56) - 1;
    const U64_ID_MASK: u64 = (1u64 << 24) - 1;
    const U64_LONG_TAG: u64 = 0xFFu64 << 56;
    const CONTENT_SLOTS: usize = 4096;

    #[inline(always)]
    fn literal_u64(byte: u8) -> u64 {
        (1u64 << 56) | u64::from(byte)
    }

    #[inline(always)]
    fn short_u64_hash(low: u64, len: u8) -> u64 {
        let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ u64::from(len).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^ (h >> 30)
    }

    #[inline(always)]
    fn short_u128_hash(entry_u128: u128, len: u8) -> u64 {
        let low = entry_u128 as u64;
        let high = (entry_u128 >> 64) as u64;
        let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ u64::from(len).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^ (h >> 30)
    }

    #[inline(always)]
    fn reserve_exact_len(v: &mut Vec<u32>, need: usize) {
        if v.len() < need {
            let add = need - v.len();
            v.reserve_exact(add);
            v.resize(need, 0);
        }
    }

    struct EdgesRules<'a> {
        buf: &'a mut [u32],
        top: usize,
    }

    impl<'a> EdgesRules<'a> {
        #[inline(always)]
        fn room(&self) -> usize {
            self.buf.len() - self.top
        }

        #[inline(always)]
        fn push(&mut self, fields: [u32; RULE]) -> u32 {
            let id = 257 + (self.top >> 2) as u32;
            let dst = &mut self.buf[self.top..self.top + RULE];
            dst.copy_from_slice(&fields);
            self.top += RULE;
            id
        }

        #[inline(always)]
        fn get_inline(&self, id: u32) -> u128 {
            let r = (id - 257) as usize * RULE;
            let s = &self.buf[r..r + RULE];
            u128::from(s[0])
                | (u128::from(s[1]) << 32)
                | (u128::from(s[2]) << 64)
                | (u128::from(s[3] & 0x00ff_ffff) << 96)
        }

        fn append_fast(&mut self, word: u32, spill: &mut Vec<u32>, dest: &mut Vec<u8>) {
            let mut stk = [0u32; 64];
            let mut sp = 0usize;
            let mut cur = word;
            let mut used_spill = false;
            loop {
                if cur <= 256 {
                    dest.push((cur - 1) as u8);
                } else {
                    let r = (cur - 257) as usize * RULE;
                    let tag = self.buf[r + 3];
                    if tag >> 28 == 0 {
                        let bytes = (u128::from(self.buf[r])
                            | (u128::from(self.buf[r + 1]) << 32)
                            | (u128::from(self.buf[r + 2]) << 64)
                            | (u128::from(tag & 0x00ff_ffff) << 96))
                            .to_le_bytes();
                        dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                    } else if tag == u32::MAX {
                        let r0 = self.buf[r];
                        let r1 = self.buf[r + 1];
                        self.buf[r + 3] = 0x8000_0000 | dest.len() as u32;
                        if sp < 64 {
                            stk[sp] = r1;
                            sp += 1;
                        } else {
                            if !used_spill {
                                spill.clear();
                                used_spill = true;
                            }
                            spill.push(r1);
                        }
                        cur = r0;
                        continue;
                    } else {
                        let start = (tag & 0x7fff_ffff) as usize;
                        let len = self.buf[r + 2] as usize;
                        dest.extend_from_within(start..start + len);
                    }
                }
                if used_spill {
                    if let Some(next) = spill.pop() {
                        cur = next;
                        continue;
                    }
                }
                if sp > 0 {
                    sp -= 1;
                    cur = stk[sp];
                } else {
                    break;
                }
            }
        }
    }

    #[inline(always)]
    fn join_u64(rules: &mut EdgesRules<'_>, left: u64, right: u64) -> u64 {
        let ll = (left >> 56) as u32;
        let rl = (right >> 56) as u32;
        if (ll + rl < 8) | (ll.min(rl) == 0) {
            return (left + (u64::from(rl) << 56)) | (right << (8 * (ll & 7)));
        }
        join_u64_long(rules, left, right, ll, rl)
    }

    #[cold]
    #[inline(never)]
    fn join_u64_long(rules: &mut EdgesRules<'_>, left: u64, right: u64, ll: u32, rl: u32) -> u64 {
        if ll < 16 && rl < 16 && ll + rl < 16 {
            let sum = ll + rl;
            let lp = if ll < 8 {
                u128::from(left & U64_PAYLOAD_MASK)
            } else {
                rules.get_inline((left & U64_ID_MASK) as u32)
            };
            let rp = if rl < 8 {
                u128::from(right & U64_PAYLOAD_MASK)
            } else {
                rules.get_inline((right & U64_ID_MASK) as u32)
            };
            let p = lp | (rp << (8 * ll)) | (u128::from(sum) << 120);
            let id = rules.push([
                p as u32,
                (p >> 32) as u32,
                (p >> 64) as u32,
                (p >> 96) as u32,
            ]);
            return (u64::from(sum) << 56) | u64::from(id);
        }
        let (lid, l_len) = if ll == 0xFF {
            (
                (left & U64_ID_MASK) as u32,
                ((left >> 24) & 0xFFFF_FFFF) as u32,
            )
        } else if ll == 1 {
            (u32::from(left as u8) + 1, 1)
        } else if ll < 8 {
            let p = left & U64_PAYLOAD_MASK;
            let id = rules.push([p as u32, (p >> 32) as u32, 0, ll << 24]);
            (id, ll)
        } else {
            ((left & U64_ID_MASK) as u32, ll)
        };
        let (rid, r_len) = if left == right {
            (lid, l_len)
        } else if rl == 0xFF {
            (
                (right & U64_ID_MASK) as u32,
                ((right >> 24) & 0xFFFF_FFFF) as u32,
            )
        } else if rl == 1 {
            (u32::from(right as u8) + 1, 1)
        } else if rl < 8 {
            let p = right & U64_PAYLOAD_MASK;
            let id = rules.push([p as u32, (p >> 32) as u32, 0, rl << 24]);
            (id, rl)
        } else {
            ((right & U64_ID_MASK) as u32, rl)
        };
        let total_len = l_len + r_len;
        let id = rules.push([lid, rid, total_len, u32::MAX]);
        U64_LONG_TAG | (u64::from(total_len) << 24) | u64::from(id)
    }

    fn power_u64(rules: &mut EdgesRules<'_>, word: u64, q: u32) -> u64 {
        let (mut acc, mut base, mut e) = (0u64, word, q);
        loop {
            if e & 1 == 1 {
                acc = join_u64(rules, acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = join_u64(rules, base, base);
        }
    }

    fn fold_left_u64(rules: &mut EdgesRules<'_>, x: u64, mut q: u32, mut y: u64) -> u64 {
        if x == 0 {
            return y;
        }
        let xl = (x >> 56) as u32;
        if xl < 8 {
            while q > 0 {
                let yl = (y >> 56) as u32;
                if yl >= 8 || xl + yl >= 8 {
                    break;
                }
                y = if y == 0 {
                    x
                } else {
                    (x + (u64::from(yl) << 56)) | (y << (8 * xl))
                };
                q -= 1;
            }
        }
        if q == 0 {
            return y;
        }
        let head = power_u64(rules, x, q);
        join_u64(rules, head, y)
    }

    fn fold_right_u64(rules: &mut EdgesRules<'_>, mut x: u64, y: u64, mut q: u32) -> u64 {
        if y == 0 {
            return x;
        }
        let yl = (y >> 56) as u32;
        if yl < 8 {
            while q > 0 {
                let xl = (x >> 56) as u32;
                if xl >= 8 || xl + yl >= 8 {
                    break;
                }
                x = if x == 0 {
                    y
                } else {
                    (x + (u64::from(yl) << 56)) | (y << (8 * xl))
                };
                q -= 1;
            }
        }
        if q == 0 {
            return x;
        }
        let tail = power_u64(rules, y, q);
        join_u64(rules, x, tail)
    }

    // 32-byte AoS node layout inside `&mut [u32; CAP8]`:
    //   [prev0, next0, prev1, next1, word_lo, word_hi, width, pad]
    // Placing `word` at +0x10 aligns the 64-bit word to 8 bytes and keeps all fields
    // of a node inside a single half cache line.
    #[inline(always)]
    fn base<const CAP8: usize>(id: u32) -> usize {
        ((id as usize) << 3) & (CAP8 - 1)
    }
    #[inline(always)]
    fn set_prev0<const CAP8: usize>(a: &mut [u32; CAP8], id: u32, to: u32) {
        a[base::<CAP8>(id)] = to;
    }
    #[inline(always)]
    fn set_next0<const CAP8: usize>(a: &mut [u32; CAP8], id: u32, to: u32) {
        a[base::<CAP8>(id) | 1] = to;
    }
    #[inline(always)]
    fn set_prev1<const CAP8: usize>(a: &mut [u32; CAP8], id: u32, to: u32) {
        a[base::<CAP8>(id) | 2] = to;
    }
    #[inline(always)]
    fn set_next1<const CAP8: usize>(a: &mut [u32; CAP8], id: u32, to: u32) {
        a[base::<CAP8>(id) | 3] = to;
    }
    #[inline(always)]
    fn set_word<const CAP8: usize>(a: &mut [u32; CAP8], id: u32, val: u64) {
        let b = base::<CAP8>(id);
        a[b | 4] = val as u32;
        a[b | 5] = (val >> 32) as u32;
    }
    #[inline(always)]
    fn set_width<const CAP8: usize>(a: &mut [u32; CAP8], id: u32, w: u32) {
        a[base::<CAP8>(id) | 6] = w;
    }

    /// Pass 0: unpack `strips_raw` (`0..3*ns`) into `img_u32` (`3*ns..5*ns`) sorted by image
    /// coordinate and compute the exact node count (`1 + ns + from_gaps + to_gaps`) so we pick
    /// the tightest power-of-two `CAP`.
    #[inline(never)]
    fn prepare_header_exact_nodes(
        edges: &mut [u32],
        ns: usize,
        n: u32,
        first: &[u32; 256],
        write: &mut [u32; 256],
    ) -> usize {
        let (strips_raw, rest) = edges.split_at_mut(3 * ns);
        let img_u32 = &mut rest[..2 * ns];
        let mut from_gaps = 0usize;
        let mut x = 0u32;
        for i in 0..ns {
            let b = 3 * i;
            let from = strips_raw[b];
            let to_rel = strips_raw[b + 1];
            let meta = strips_raw[b + 2];
            let a = usize::from((meta & 0xFF) as u8);
            let width = meta >> 8;
            let to = to_rel + first[a];
            if x < from {
                from_gaps += 1;
            }
            x = from + width;
            let pos = write[a] as usize;
            write[a] = (pos + 1) as u32;
            let packed =
                u64::from(to) | (u64::from(width) << 20) | (u64::from((i + 1) as u32) << 40);
            img_u32[2 * pos] = packed as u32;
            img_u32[2 * pos + 1] = (packed >> 32) as u32;
        }
        if x < n {
            from_gaps += 1;
        }
        let base_nodes = 1 + ns + from_gaps;
        let max_nodes = base_nodes + ns + 1;
        let lo_cap = base_nodes.next_power_of_two().max(2048);
        if max_nodes <= lo_cap {
            return max_nodes;
        }
        let mut to_gaps = 0usize;
        let mut y = 0u32;
        for pos in 0..ns {
            let packed = u64::from(img_u32[2 * pos]) | (u64::from(img_u32[2 * pos + 1]) << 32);
            let to = (packed & 0xF_FFFF) as u32;
            let width = ((packed >> 20) & 0xF_FFFF) as u32;
            if y < to {
                to_gaps += 1;
            }
            y = to + width;
        }
        if y < n {
            to_gaps += 1;
        }
        base_nodes + to_gaps
    }

    #[inline(never)]
    fn setup_aos<const CAP8: usize>(
        strips_raw: &[u32],
        img_u32: &[u32],
        a: &mut [u32; CAP8],
        ns: usize,
        n: u32,
    ) -> (u32, usize) {
        a[..8].fill(0);
        let mut nodes = (1 + ns) as u32;
        let mut widest = 0u32;
        let mut ord0_tail = 0u32;
        let (mut term1_head, mut term1_tail) = (0u32, 0u32);
        let mut x = 0u32;
        for i in 0..ns {
            let from = strips_raw[3 * i];
            let meta = strips_raw[3 * i + 2];
            let byte = (meta & 0xFF) as u8;
            let w = meta >> 8;
            let sid = (i + 1) as u32;
            set_word::<CAP8>(a, sid, literal_u64(byte));
            set_width::<CAP8>(a, sid, w);
            widest = widest.max(w);
            if x < from {
                let gap = from - x;
                widest = widest.max(gap);
                let gid = nodes;
                nodes += 1;
                set_word::<CAP8>(a, gid, 0);
                set_width::<CAP8>(a, gid, gap);
                set_next0::<CAP8>(a, ord0_tail, gid);
                set_prev0::<CAP8>(a, gid, ord0_tail);
                ord0_tail = gid;
                if term1_tail == 0 {
                    term1_head = gid;
                } else {
                    set_next1::<CAP8>(a, term1_tail, gid);
                }
                set_prev1::<CAP8>(a, gid, term1_tail);
                term1_tail = gid;
            }
            set_next0::<CAP8>(a, ord0_tail, sid);
            set_prev0::<CAP8>(a, sid, ord0_tail);
            ord0_tail = sid;
            x = from + w;
        }
        if x < n {
            let gap = n - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            set_word::<CAP8>(a, gid, 0);
            set_width::<CAP8>(a, gid, gap);
            set_next0::<CAP8>(a, ord0_tail, gid);
            set_prev0::<CAP8>(a, gid, ord0_tail);
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                set_next1::<CAP8>(a, term1_tail, gid);
            }
            set_prev1::<CAP8>(a, gid, term1_tail);
            term1_tail = gid;
        }
        let ord0_head = if ord0_tail == 0 { 0 } else { a[1] };
        let mut ord1_tail = 0u32;
        let (mut term0_head, mut term0_tail) = (0u32, 0u32);
        x = 0;
        for pos in 0..ns {
            let packed = u64::from(img_u32[2 * pos]) | (u64::from(img_u32[2 * pos + 1]) << 32);
            let to = (packed & 0xF_FFFF) as u32;
            let w = ((packed >> 20) & 0xF_FFFF) as u32;
            let sid = (packed >> 40) as u32;
            if x < to {
                let gap = to - x;
                widest = widest.max(gap);
                let gid = nodes;
                nodes += 1;
                set_word::<CAP8>(a, gid, 0);
                set_width::<CAP8>(a, gid, gap);
                set_next1::<CAP8>(a, ord1_tail, gid);
                set_prev1::<CAP8>(a, gid, ord1_tail);
                ord1_tail = gid;
                if term0_tail == 0 {
                    term0_head = gid;
                } else {
                    set_next0::<CAP8>(a, term0_tail, gid);
                }
                set_prev0::<CAP8>(a, gid, term0_tail);
                term0_tail = gid;
            }
            set_next1::<CAP8>(a, ord1_tail, sid);
            set_prev1::<CAP8>(a, sid, ord1_tail);
            ord1_tail = sid;
            x = to + w;
        }
        if x < n {
            let gap = n - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            set_word::<CAP8>(a, gid, 0);
            set_width::<CAP8>(a, gid, gap);
            set_next1::<CAP8>(a, ord1_tail, gid);
            set_prev1::<CAP8>(a, gid, ord1_tail);
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                set_next0::<CAP8>(a, term0_tail, gid);
            }
            set_prev0::<CAP8>(a, gid, term0_tail);
            term0_tail = gid;
        }
        let ord1_head = if ord1_tail == 0 { 0 } else { a[3] };
        let mut prev = 0u32;
        for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
            if t == 0 {
                continue;
            }
            set_next0::<CAP8>(a, prev, h);
            set_prev0::<CAP8>(a, h, prev);
            prev = t;
        }
        set_next0::<CAP8>(a, prev, 0);
        a[0] = prev;

        prev = 0u32;
        for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
            if t == 0 {
                continue;
            }
            set_next1::<CAP8>(a, prev, h);
            set_prev1::<CAP8>(a, h, prev);
            prev = t;
        }
        set_next1::<CAP8>(a, prev, 0);
        a[2] = prev;
        (widest, nodes as usize)
    }

    #[inline(never)]
    fn elim_aos<const CAP8: usize, const SPIRALS: bool>(
        a: &mut [u32; CAP8],
        rules: &mut EdgesRules<'_>,
        left_ref: &mut u32,
        #[cfg(any(test, feature = "census", feature = "induction-trace"))] steps_ref: &mut usize,
    ) {
        let mut left = *left_ref;
        #[cfg(any(test, feature = "census", feature = "induction-trace"))]
        let mut steps = *steps_ref;
        let mut x = a[0];
        let mut y = a[2];
        while left != 0 {
            let bx = base::<CAP8>(x);
            let by = base::<CAP8>(y);
            let aw = a[bx | 6];
            let bw = a[by | 6];
            if SPIRALS && x != y {
                if aw > bw && a[by | 2] == x && (aw - 1) / bw >= 2 {
                    if rules.room() < (SPIRAL_RULES + 4) * RULE {
                        break;
                    }
                    let q = (aw - 1) / bw;
                    assert!(q * bw < aw && q * bw <= left);
                    let wx = u64::from(a[bx | 4]) | (u64::from(a[bx | 5]) << 32);
                    let wy = u64::from(a[by | 4]) | (u64::from(a[by | 5]) << 32);
                    let nw = fold_left_u64(rules, wx, q, wy);
                    a[by | 4] = nw as u32;
                    a[by | 5] = (nw >> 32) as u32;
                    a[bx | 6] = aw - q * bw;
                    left -= q * bw;
                    #[cfg(any(test, feature = "census", feature = "induction-trace"))]
                    {
                        steps += q as usize;
                    }
                    continue;
                }
                if aw < bw && a[bx] == y && (bw - 1) / aw >= 2 {
                    if rules.room() < (SPIRAL_RULES + 4) * RULE {
                        break;
                    }
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let wx = u64::from(a[bx | 4]) | (u64::from(a[bx | 5]) << 32);
                    let wy = u64::from(a[by | 4]) | (u64::from(a[by | 5]) << 32);
                    let nw = fold_right_u64(rules, wx, wy, q);
                    a[bx | 4] = nw as u32;
                    a[bx | 5] = (nw >> 32) as u32;
                    a[by | 6] = bw - q * aw;
                    left -= q * aw;
                    #[cfg(any(test, feature = "census", feature = "induction-trace"))]
                    {
                        steps += q as usize;
                    }
                    continue;
                }
            }
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                #[cfg(any(test, feature = "census", feature = "induction-trace"))]
                {
                    steps += 1;
                }
                let xp = a[bx];
                set_next0::<CAP8>(a, xp, 0);
                x = xp;
                let yp = a[by | 2];
                set_next1::<CAP8>(a, yp, 0);
                y = yp;
                continue;
            }
            let wx = u64::from(a[bx | 4]) | (u64::from(a[bx | 5]) << 32);
            let wy = u64::from(a[by | 4]) | (u64::from(a[by | 5]) << 32);
            let ll = (wx >> 56) as u32;
            let rl = (wy >> 56) as u32;
            let wd = if (ll + rl < 8) | (ll.min(rl) == 0) {
                (wx + (u64::from(rl) << 56)) | (wy << (8 * (ll & 7)))
            } else {
                if rules.room() < 3 * RULE {
                    break;
                }
                join_u64_long(rules, wx, wy, ll, rl)
            };
            #[cfg(any(test, feature = "census", feature = "induction-trace"))]
            {
                steps += 1;
            }
            if aw > bw {
                assert!(bw > 0 && bw <= left);
                left -= bw;
                let yp = a[by | 2];
                a[by | 4] = wd as u32;
                a[by | 5] = (wd >> 32) as u32;
                a[bx | 6] = aw - bw;
                if yp != x {
                    set_next1::<CAP8>(a, yp, 0);
                    let xn = a[bx | 3];
                    a[by | 2] = x;
                    a[by | 3] = xn;
                    a[bx | 3] = y;
                    set_prev1::<CAP8>(a, xn, y);
                    y = yp;
                }
            } else if aw < bw {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let xp = a[bx];
                a[bx | 4] = wd as u32;
                a[bx | 5] = (wd >> 32) as u32;
                a[by | 6] = bw - aw;
                if xp != y {
                    set_next0::<CAP8>(a, xp, 0);
                    let yn = a[by | 1];
                    a[bx] = y;
                    a[bx | 1] = yn;
                    a[by | 1] = x;
                    set_prev0::<CAP8>(a, yn, x);
                    x = xp;
                }
            } else {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let xp = a[bx];
                let yp0 = a[by];
                let yp1 = a[by | 2];
                a[bx | 4] = wd as u32;
                a[bx | 5] = (wd >> 32) as u32;
                set_next1::<CAP8>(a, yp1, 0);
                if xp == y {
                    set_next0::<CAP8>(a, yp0, x);
                    a[bx] = yp0;
                } else {
                    set_next0::<CAP8>(a, xp, 0);
                    let yn0 = a[by | 1];
                    a[bx] = yp0;
                    a[bx | 1] = yn0;
                    set_next0::<CAP8>(a, yp0, x);
                    set_prev0::<CAP8>(a, yn0, x);
                    x = xp;
                }
                y = yp1;
            }
        }
        a[0] = x;
        a[2] = y;
        *left_ref = left;
        #[cfg(any(test, feature = "census", feature = "induction-trace"))]
        {
            *steps_ref = steps;
        }
    }

    #[inline(never)]
    fn assign_bottom_aos<const CAP8: usize>(a: &mut [u32; CAP8], rows: u32) {
        let mut first = a[3];
        let mut last = a[2];
        let (mut lo, mut hi) = (0u32, rows);
        while first != last {
            let bf = base::<CAP8>(first);
            let bl = base::<CAP8>(last);
            let nf = a[bf | 3];
            let pl = a[bl | 2];
            let wf = a[bf | 6];
            hi -= a[bl | 6];
            a[bf | 2] = lo;
            a[bl | 2] = hi;
            lo += wf;
            if nf == last {
                assert_eq!(lo, hi);
                return;
            }
            first = nf;
            last = pl;
        }
        if first != 0 {
            let bf = base::<CAP8>(first);
            a[bf | 2] = lo;
            lo += a[bf | 6];
        }
        assert_eq!(lo, hi);
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn dict_aos<const CAP8: usize>(
        a: &[u32; CAP8],
        rules: &mut EdgesRules<'_>,
        contents: &mut [u32; CONTENT_SLOTS],
        bands: &mut Vec<Band>,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        spelling: &mut Vec<u8>,
        stack: &mut Vec<u32>,
        thin: &[u32; 256],
        n: u32,
        #[cfg(any(test, feature = "census", feature = "induction-trace"))] stats: &mut Counts,
    ) -> [usize; 256] {
        contents.fill(0);
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
        let mut cursors = [0usize; 256];
        let mut id = a[1];
        let mut b = base::<CAP8>(id);
        let mut nw = a[b | 6];
        let mut nwd = u64::from(a[b | 4]) | (u64::from(a[b | 5]) << 32);
        let mut nbot = a[b | 2];
        let mut nnext = a[b | 1];
        let mut in_node = 0u32;
        let mut k = 0u32;
        for ch in 0..256 {
            cursors[ch] = bands.len();
            let end = k + thin[ch];
            while k < end {
                let width = (end - k).min(nw - in_node);
                assert!(width > 0);
                let dst = nbot + in_node;
                #[cfg(any(test, feature = "census", feature = "induction-trace"))]
                if nwd != 0 {
                    let tag = (nwd >> 56) as u32;
                    let height = if tag == 0xFF {
                        ((nwd >> 24) & 0xFFFF_FFFF) as usize
                    } else {
                        tag as usize
                    };
                    stats.words += 1;
                    stats.copies += width as usize;
                    stats.traced += height;
                    stats.thick_area += width as usize * height;
                    stats.unit_work += usize::from(width == 1) * height;
                    stats.longest = stats.longest.max(height);
                }
                let code = if nwd == 0 {
                    ch as u32
                } else {
                    let tag = (nwd >> 56) as u32;
                    if tag < 8 {
                        let low = (nwd & U64_PAYLOAD_MASK) | (u64::from(ch as u8) << (8 * tag));
                        let len = (tag + 1) as u8;
                        let h = short_u64_hash(low, len);
                        let cidx = (h as usize) & (CONTENT_SLOTS - 1);
                        let fp16 = (((h >> 12) as u32) & 0xFFFF_0000) | 0x8000_0000;
                        let slot = contents[cidx];
                        let mut hit = 0u32;
                        if (slot & 0xFFFF_0000) == fp16 {
                            let old = (slot & 0xFFFF) as usize;
                            if lens[old] == len
                                && u64::from_le_bytes(inline16[old][..8].try_into().unwrap()) == low
                            {
                                hit = old as u32;
                            }
                        }
                        if hit != 0 {
                            hit
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(u128::from(low).to_le_bytes());
                            lens.push(len);
                            if code < 65_536 {
                                contents[cidx] = fp16 | code;
                            }
                            code
                        }
                    } else if tag < 16 {
                        let payload = rules.get_inline((nwd & U64_ID_MASK) as u32);
                        let entry = payload | (u128::from(ch as u8) << (8 * tag));
                        let len = (tag + 1) as u8;
                        let h = short_u128_hash(entry, len);
                        let cidx = (h as usize) & (CONTENT_SLOTS - 1);
                        let fp16 = (((h >> 12) as u32) & 0xFFFF_0000) | 0x8000_0000;
                        let slot = contents[cidx];
                        let mut hit = 0u32;
                        if (slot & 0xFFFF_0000) == fp16 {
                            let old = (slot & 0xFFFF) as usize;
                            if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry {
                                hit = old as u32;
                            }
                        }
                        if hit != 0 {
                            hit
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(entry.to_le_bytes());
                            lens.push(len);
                            if code < 65_536 {
                                contents[cidx] = fp16 | code;
                            }
                            code
                        }
                    } else {
                        let start = spelling.len();
                        let rid = (nwd & U64_ID_MASK) as u32;
                        let memoized = rules.buf[(rid - 257) as usize * RULE + 3] != u32::MAX;
                        rules.append_fast(rid, stack, spelling);
                        spelling.push(ch as u8);
                        let send = spelling.len();
                        let bytes = &spelling[start..send];
                        let h = spelling_hash(&[0; INLINE], LONG, bytes);
                        let cidx = (h as usize) & (CONTENT_SLOTS - 1);
                        let fp16 = (((h >> 12) as u32) & 0xFFFF_0000) | 0x8000_0000;
                        let slot = contents[cidx];
                        let mut hit = 0u32;
                        if (slot & 0xFFFF_0000) == fp16 {
                            let old = (slot & 0xFFFF) as usize;
                            if lens[old] == LONG && long_word(&inline16[old], spelling) == bytes {
                                if memoized {
                                    spelling.truncate(start);
                                } else {
                                    spelling.pop();
                                }
                                hit = old as u32;
                            }
                        }
                        if hit != 0 {
                            hit
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(long_slot(start as u32, send as u32));
                            lens.push(LONG);
                            if code < 65_536 {
                                contents[cidx] = fp16 | code;
                            }
                            code
                        }
                    }
                };
                bands.push(Band::new(k + width, dst, code));
                in_node += width;
                k += width;
                if in_node == nw {
                    id = nnext;
                    in_node = 0;
                    if id != 0 {
                        b = base::<CAP8>(id);
                        nw = a[b | 6];
                        nwd = u64::from(a[b | 4]) | (u64::from(a[b | 5]) << 32);
                        nbot = a[b | 2];
                        nnext = a[b | 1];
                    }
                }
            }
        }
        assert_eq!(id, 0);
        cursors
    }

    impl Kernel {
        #[allow(clippy::too_many_arguments)]
        fn run_overlay_aos_cap<const CAP8: usize>(
            &mut self,
            ns: usize,
            n: u32,
            m: u32,
            thin: &[u32; 256],
            stats: &mut Counts,
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
            let hdr_words = (5 * ns).max(CONTENT_SLOTS);
            let rules_base = hdr_words + CAP8;
            let init_rules = RULE * (ns / 2 + 256);
            let need = (rules_base + init_rules).max(m as usize);
            reserve_exact_len(edges, need);

            let (widest, nodes_used) = {
                let (hdr, rest) = edges.split_at_mut(hdr_words);
                let (strips_raw, img_rest) = hdr.split_at(3 * ns);
                let img_u32 = &img_rest[..2 * ns];
                let a: &mut [u32; CAP8] = (&mut rest[..CAP8]).try_into().unwrap();
                setup_aos::<CAP8>(strips_raw, img_u32, a, ns, n)
            };
            stats.open_nodes = nodes_used - 1;
            #[cfg(test)]
            let spirals = endpoint.spirals.unwrap_or(widest > CERTIFIED_WIDTH);
            #[cfg(not(test))]
            let spirals = widest > CERTIFIED_WIDTH;
            stats.spirals = spirals;

            let room_need = if spirals {
                (SPIRAL_RULES + 4) * RULE
            } else {
                3 * RULE
            };
            let mut left = n;
            #[cfg(any(test, feature = "census", feature = "induction-trace"))]
            let mut steps = 0usize;
            let mut rules_top = 0usize;
            while left != 0 {
                let (_, rest) = edges.split_at_mut(hdr_words);
                let (a_s, rules_buf) = rest.split_at_mut(CAP8);
                let a: &mut [u32; CAP8] = a_s.try_into().unwrap();
                let mut rules = EdgesRules {
                    buf: rules_buf,
                    top: rules_top,
                };
                if spirals {
                    elim_aos::<CAP8, true>(
                        a,
                        &mut rules,
                        &mut left,
                        #[cfg(any(test, feature = "census", feature = "induction-trace"))]
                        &mut steps,
                    );
                } else {
                    elim_aos::<CAP8, false>(
                        a,
                        &mut rules,
                        &mut left,
                        #[cfg(any(test, feature = "census", feature = "induction-trace"))]
                        &mut steps,
                    );
                }
                rules_top = rules.top;
                if left != 0 {
                    let grow_to = (rules_base + rules_top + room_need).max(edges.len() + 4096);
                    reserve_exact_len(edges, grow_to);
                }
            }
            #[cfg(any(test, feature = "census", feature = "induction-trace"))]
            {
                stats.steps = steps;
            }
            stats.rules = rules_top / RULE;

            {
                let (_, rest) = edges.split_at_mut(hdr_words);
                let a: &mut [u32; CAP8] = (&mut rest[..CAP8]).try_into().unwrap();
                assign_bottom_aos::<CAP8>(a, m);
            }

            let (hdr, rest) = edges.split_at_mut(hdr_words);
            let contents: &mut [u32; CONTENT_SLOTS] =
                (&mut hdr[..CONTENT_SLOTS]).try_into().unwrap();
            let (a_s, rules_buf) = rest.split_at_mut(CAP8);
            let a: &[u32; CAP8] = (&*a_s).try_into().unwrap();
            let mut rules = EdgesRules {
                buf: &mut rules_buf[..rules_top],
                top: rules_top,
            };
            dict_aos::<CAP8>(
                a,
                &mut rules,
                contents,
                bands,
                inline16,
                lens,
                spelling,
                &mut endpoint.stack,
                thin,
                n,
                #[cfg(any(test, feature = "census", feature = "induction-trace"))]
                stats,
            )
        }

        #[allow(clippy::too_many_arguments)]
        pub(in super::super) fn induced_overlay_aos(
            &mut self,
            ns: usize,
            n: u32,
            m: u32,
            first: &[u32; 256],
            write: &mut [u32; 256],
            thin: &[u32; 256],
            stats: &mut Counts,
        ) -> [usize; 256] {
            let hdr_min = (5 * ns).max(CONTENT_SLOTS);
            reserve_exact_len(&mut self.edges, hdr_min);
            let mut write_copy = *write;
            let n_nodes =
                prepare_header_exact_nodes(&mut self.edges, ns, n, first, &mut write_copy);
            if n_nodes <= 2048 {
                self.run_overlay_aos_cap::<16384>(ns, n, m, thin, stats)
            } else if n_nodes <= 4096 {
                self.run_overlay_aos_cap::<32768>(ns, n, m, thin, stats)
            } else if n_nodes <= 8192 {
                self.run_overlay_aos_cap::<65536>(ns, n, m, thin, stats)
            } else if n_nodes <= 16384 {
                self.run_overlay_aos_cap::<131072>(ns, n, m, thin, stats)
            } else if n_nodes <= 32768 {
                self.run_overlay_aos_cap::<262144>(ns, n, m, thin, stats)
            } else {
                // Rare 32769..=65535-node narrow block: populate image_order and fall back to
                // compact Narrow so `edges` never allocates a 2 MiB CAP=65536 AoS table.
                let need = (4 * ns + (4 + RULE) * Self::nodes_most(ns)).max(m as usize);
                reserve_exact_len(&mut self.edges, need);
                let (strips_raw, rest) = self.edges.split_at_mut(3 * ns);
                let image_order = &mut rest[..ns];
                for i in 0..ns {
                    let sw = &mut strips_raw[3 * i..3 * i + 3];
                    let a = usize::from((sw[2] & 0xFF) as u8);
                    sw[1] += first[a];
                    image_order[write[a] as usize] = i as u32;
                    write[a] += 1;
                }
                self.induced(ns, n, thin, stats)
            }
        }
    }
}

#[cfg(feature = "a6b-endpoint-redesign")]
mod redesign {
    use super::super::{begin_inline, long_slot, long_word, Band, Counts, Kernel, INLINE, LONG};
    #[cfg(test)]
    use super::Clobber;
    use super::{CERTIFIED_WIDTH, NARROW_NODES};

    /// `u128` inline-15 encoding: top byte (bits 120..128) is tag:
    /// - `0` => empty word (`0u128`)
    /// - `1..=15` => inline payload of `tag` bytes in bits `0..(8 * tag)`, higher payload bits 0
    /// - `0xFF` => long rule handle: bits `0..32` = rule id (`>= 257`), bits `32..64` = exact length
    const U128_PAYLOAD_MASK: u128 = (1u128 << 120) - 1;
    const U128_LONG_TAG: u128 = 0xFFu128 << 120;

    const GEN_PAIR_SLOTS: usize = 1024;
    const GEN_CONTENT_SLOTS: usize = 4096;

    #[inline(always)]
    fn literal_u128(byte: u8) -> u128 {
        (1u128 << 120) | u128::from(byte)
    }

    #[derive(Default, Clone, Copy)]
    pub(super) struct NodeLinks {
        pub(super) prev0: u16,
        pub(super) next0: u16,
        pub(super) prev1: u16,
        pub(super) next1: u16,
    }

    fn boxed_array_64k<T: Copy>(val: T) -> Box<[T; NARROW_NODES]> {
        vec![val; NARROW_NODES]
            .into_boxed_slice()
            .try_into()
            .ok()
            .unwrap()
    }

    /// Fixed-size 64K SoA node tables indexed by `usize::from(u16)` for safe zero-bounds-check
    /// access (zero `unsafe`). Allocated once per worker on first Narrow block and reused.
    /// Reserves 1.75 MiB virtual (512 KiB links + 256 KiB widths + 1 MiB words128); only
    /// `0..nodes` is touched per block.
    pub(super) struct Fixed64kSoa {
        pub(super) links: Box<[NodeLinks; NARROW_NODES]>,
        pub(super) widths: Box<[u32; NARROW_NODES]>,
        pub(super) words128: Box<[u128; NARROW_NODES]>,
    }

    impl Fixed64kSoa {
        fn new() -> Self {
            Fixed64kSoa {
                links: boxed_array_64k(NodeLinks::default()),
                widths: boxed_array_64k(0),
                words128: boxed_array_64k(0),
            }
        }
    }

    /// Generation-tagged L1d-hot caches (eliminating per-block 24 KiB zero-fill):
    /// - `pairs`: 1024 x u64 (8 KiB), packed as `(gen18 << 46) | (code16 << 30) | key30`
    /// - `contents`: 4096 x u32 (16 KiB), packed as `(gen16 << 16) | code16`
    pub(super) struct GenCache {
        pub(super) pairs: Vec<u64>,
        pub(super) contents: Vec<u32>,
        pub(super) pair_gen: u64,
        pub(super) content_gen: u32,
    }

    impl Default for GenCache {
        fn default() -> Self {
            GenCache {
                pairs: vec![0; GEN_PAIR_SLOTS],
                contents: vec![0; GEN_CONTENT_SLOTS],
                pair_gen: 0,
                content_gen: 0,
            }
        }
    }

    impl GenCache {
        #[inline(always)]
        pub(super) fn next_block(&mut self) {
            self.pair_gen += 1;
            if self.pair_gen >= (1u64 << 18) {
                self.pairs.fill(0);
                self.pair_gen = 1;
            }
            self.content_gen += 1;
            if self.content_gen >= (1u32 << 16) {
                self.contents.fill(0);
                self.content_gen = 1;
            }
        }
    }

    /// Reusable per-worker workspace for the redesigned Narrow `A6b` path.
    #[derive(Default)]
    pub(super) struct State {
        pub(super) image_order64: Vec<u64>,
        pub(super) fixed_soa: Option<Fixed64kSoa>,
        pub(super) rules: Vec<[u32; 4]>,
        pub(super) gen_cache: GenCache,
    }

    impl State {
        #[cfg(test)]
        pub(super) fn scratch(&self) -> usize {
            use std::mem::size_of;
            let soa_bytes = if self.fixed_soa.is_some() {
                NARROW_NODES * (size_of::<NodeLinks>() + size_of::<u32>() + size_of::<u128>())
            } else {
                0
            };
            size_of::<u64>() * self.image_order64.capacity()
                + size_of::<[u32; 4]>() * self.rules.capacity()
                + size_of::<u64>() * self.gen_cache.pairs.capacity()
                + size_of::<u32>() * self.gen_cache.contents.capacity()
                + soa_bytes
        }
    }

    #[inline(always)]
    fn join_u128(rules: &mut Vec<[u32; 4]>, left: u128, right: u128) -> u128 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let ll = (left >> 120) as u32;
        let rl = (right >> 120) as u32;
        if ll + rl < 16 {
            return (left + (u128::from(rl) << 120)) | (right << (8 * ll));
        }
        join_u128_long(rules, left, right, ll, rl)
    }

    #[cold]
    #[inline(never)]
    fn join_u128_long(
        rules: &mut Vec<[u32; 4]>,
        left: u128,
        right: u128,
        ll: u32,
        rl: u32,
    ) -> u128 {
        let (lid, l_len) = if ll == 0xFF {
            (left as u32, (left >> 32) as u32)
        } else if ll == 1 {
            (u32::from(left as u8) + 1, 1)
        } else {
            let id = rules.len() as u32 + 257;
            rules.push([
                left as u32,
                (left >> 32) as u32,
                (left >> 64) as u32,
                (left >> 96) as u32,
            ]);
            (id, ll)
        };
        let (rid, r_len) = if left == right {
            (lid, l_len)
        } else if rl == 0xFF {
            (right as u32, (right >> 32) as u32)
        } else if rl == 1 {
            (u32::from(right as u8) + 1, 1)
        } else {
            let id = rules.len() as u32 + 257;
            rules.push([
                right as u32,
                (right >> 32) as u32,
                (right >> 64) as u32,
                (right >> 96) as u32,
            ]);
            (id, rl)
        };
        let total_len = l_len + r_len;
        let id = rules.len() as u32 + 257;
        rules.push([lid, rid, total_len, u32::MAX]);
        U128_LONG_TAG | (u128::from(total_len) << 32) | u128::from(id)
    }

    fn power_u128(rules: &mut Vec<[u32; 4]>, word: u128, q: u32) -> u128 {
        let (mut acc, mut base, mut e) = (0u128, word, q);
        loop {
            if e & 1 == 1 {
                acc = join_u128(rules, acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = join_u128(rules, base, base);
        }
    }

    /// Compute `x^q y` for spiral contraction, first absorbing inline-15 steps directly into `y`
    /// (zero rules) before squaring any remainder, so rules never exceed the plain loop's.
    fn fold_left_power(rules: &mut Vec<[u32; 4]>, x: u128, mut q: u32, mut y: u128) -> u128 {
        if x == 0 {
            return y;
        }
        let xl = (x >> 120) as u32;
        if xl < 16 {
            while q > 0 {
                let yl = (y >> 120) as u32;
                if yl >= 16 || xl + yl >= 16 {
                    break;
                }
                y = if y == 0 {
                    x
                } else {
                    (x + (u128::from(yl) << 120)) | (y << (8 * xl))
                };
                q -= 1;
            }
        }
        if q == 0 {
            return y;
        }
        let head = power_u128(rules, x, q);
        join_u128(rules, head, y)
    }

    /// Compute `x y^q` for spiral contraction, first absorbing inline-15 steps directly into `x`
    /// (zero rules) before squaring any remainder, so rules never exceed the plain loop's.
    fn fold_right_power(rules: &mut Vec<[u32; 4]>, mut x: u128, y: u128, mut q: u32) -> u128 {
        if y == 0 {
            return x;
        }
        let yl = (y >> 120) as u32;
        if yl < 16 {
            while q > 0 {
                let xl = (x >> 120) as u32;
                if xl >= 16 || xl + yl >= 16 {
                    break;
                }
                x = if x == 0 {
                    y
                } else {
                    (x + (u128::from(yl) << 120)) | (y << (8 * xl))
                };
                q -= 1;
            }
        }
        if q == 0 {
            return x;
        }
        let tail = power_u128(rules, y, q);
        join_u128(rules, x, tail)
    }

    fn append_rules(rules: &mut [[u32; 4]], word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let idx = (id - 257) as usize;
                let tag = rules[idx][3];
                if tag >> 28 == 0 {
                    let r = rules[idx];
                    let bytes = (u128::from(r[0])
                        | (u128::from(r[1]) << 32)
                        | (u128::from(r[2]) << 64)
                        | (u128::from(r[3] & 0x00ff_ffff) << 96))
                        .to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                    continue;
                }
                if tag == u32::MAX {
                    let r0 = rules[idx][0];
                    let r1 = rules[idx][1];
                    rules[idx][3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(r1);
                    stack.push(r0);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let len = rules[idx][2] as usize;
                    dest.extend_from_within(start..start + len);
                }
            }
        }
    }

    #[inline(always)]
    fn short_u128_hash(entry_u128: u128, len: u8) -> u64 {
        let low = entry_u128 as u64;
        let high = (entry_u128 >> 64) as u64;
        let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ u64::from(len).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^ (h >> 30)
    }

    #[inline(always)]
    fn long_bytes_hash(bytes: &[u8]) -> u64 {
        let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        let low = part(0);
        let high = part(bytes.len() - 8);
        let middle = part(bytes.len() / 2 - 4);
        let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ middle.wrapping_mul(0xD6E8_FEB8_6659_FD93)
            ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^ (h >> 30)
    }

    /// Fused single-pass setup over `strips_raw` with packed `u64` `image_order64`
    /// (`to:20 | width:20 | sid:16`) and branchless sentinel-0 `order` appends.
    #[allow(clippy::too_many_arguments)]
    fn setup_soa_64k(
        strips_raw: &[u32],
        ns: usize,
        n: u32,
        first: &[u32; 256],
        write: &mut [u32; 256],
        img_buf: &mut Vec<u64>,
        soa: &mut Fixed64kSoa,
        #[cfg(test)] clobber: Clobber,
    ) -> (u32, usize) {
        if img_buf.len() < ns {
            img_buf.resize(ns, 0);
        }
        let img = &mut img_buf[..ns];
        let links = &mut *soa.links;
        let widths = &mut *soa.widths;
        let words = &mut *soa.words128;

        links[0] = NodeLinks::default();
        widths[0] = 0;
        words[0] = 0;

        let mut nodes = (1 + ns) as u32;
        let mut widest = 0u32;
        let mut ord0_tail = 0u16;
        let (mut term1_head, mut term1_tail) = (0u16, 0u16);
        let mut x = 0u32;

        for i in 0..ns {
            let base = 3 * i;
            let from = strips_raw[base];
            let to_rel = strips_raw[base + 1];
            let meta = strips_raw[base + 2];
            let byte = (meta & 0xFF) as u8;
            let width = meta >> 8;
            let a = usize::from(byte);
            let to = to_rel + first[a];
            let sid = (i + 1) as u16;

            widths[usize::from(sid)] = width;
            words[usize::from(sid)] = literal_u128(byte);
            widest = widest.max(width);

            let pos = write[a] as usize;
            write[a] = (pos + 1) as u32;
            img[pos] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);

            if x < from {
                let gap = from - x;
                widest = widest.max(gap);
                let gid = nodes as u16;
                nodes += 1;
                widths[usize::from(gid)] = gap;
                words[usize::from(gid)] = 0;
                links[usize::from(ord0_tail)].next0 = gid;
                links[usize::from(gid)].prev0 = ord0_tail;
                ord0_tail = gid;
                if term1_tail == 0 {
                    term1_head = gid;
                } else {
                    links[usize::from(term1_tail)].next1 = gid;
                }
                links[usize::from(gid)].prev1 = term1_tail;
                term1_tail = gid;
            }
            links[usize::from(ord0_tail)].next0 = sid;
            links[usize::from(sid)].prev0 = ord0_tail;
            ord0_tail = sid;
            x = from + width;
        }
        if x < n {
            let gap = n - x;
            widest = widest.max(gap);
            let gid = nodes as u16;
            nodes += 1;
            widths[usize::from(gid)] = gap;
            words[usize::from(gid)] = 0;
            links[usize::from(ord0_tail)].next0 = gid;
            links[usize::from(gid)].prev0 = ord0_tail;
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                links[usize::from(term1_tail)].next1 = gid;
            }
            links[usize::from(gid)].prev1 = term1_tail;
            term1_tail = gid;
        }
        let ord0_head = if ord0_tail == 0 { 0 } else { links[0].next0 };

        #[cfg(test)]
        if clobber == Clobber::OrderBeforeNodes {
            img.fill(0xA5A5_A5A5_A5A5_A5A5);
        }

        let mut ord1_tail = 0u16;
        let (mut term0_head, mut term0_tail) = (0u16, 0u16);
        x = 0;
        for &packed in img.iter() {
            let to = (packed & 0xF_FFFF) as u32;
            let width = ((packed >> 20) & 0xF_FFFF) as u32;
            let sid = (packed >> 40) as u16;
            if x < to {
                let gap = to - x;
                widest = widest.max(gap);
                let gid = nodes as u16;
                nodes += 1;
                widths[usize::from(gid)] = gap;
                words[usize::from(gid)] = 0;
                links[usize::from(ord1_tail)].next1 = gid;
                links[usize::from(gid)].prev1 = ord1_tail;
                ord1_tail = gid;
                if term0_tail == 0 {
                    term0_head = gid;
                } else {
                    links[usize::from(term0_tail)].next0 = gid;
                }
                links[usize::from(gid)].prev0 = term0_tail;
                term0_tail = gid;
            }
            links[usize::from(ord1_tail)].next1 = sid;
            links[usize::from(sid)].prev1 = ord1_tail;
            ord1_tail = sid;
            x = to + width;
        }
        if x < n {
            let gap = n - x;
            widest = widest.max(gap);
            let gid = nodes as u16;
            nodes += 1;
            widths[usize::from(gid)] = gap;
            words[usize::from(gid)] = 0;
            links[usize::from(ord1_tail)].next1 = gid;
            links[usize::from(gid)].prev1 = ord1_tail;
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                links[usize::from(term0_tail)].next0 = gid;
            }
            links[usize::from(gid)].prev0 = term0_tail;
            term0_tail = gid;
        }
        let ord1_head = if ord1_tail == 0 { 0 } else { links[0].next1 };

        #[cfg(test)]
        if clobber == Clobber::OrderAfterNodes {
            img.fill(0xA5A5_A5A5_A5A5_A5A5);
        }

        {
            let mut prev = 0u16;
            for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
                if t == 0 {
                    continue;
                }
                links[usize::from(prev)].next0 = h;
                links[usize::from(h)].prev0 = prev;
                prev = t;
            }
            links[usize::from(prev)].next0 = 0;
            links[0].prev0 = prev;
        }
        {
            let mut prev = 0u16;
            for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
                if t == 0 {
                    continue;
                }
                links[usize::from(prev)].next1 = h;
                links[usize::from(h)].prev1 = prev;
                prev = t;
            }
            links[usize::from(prev)].next1 = 0;
            links[0].prev1 = prev;
        }
        (widest, nodes as usize)
    }

    /// Safe zero-bounds-check induction over `Fixed64kSoa` (`inline15` `u128` words + direct
    /// `aw == bw` replace), preserving `SPIRALS` contraction identically to shipped `eliminate`.
    fn eliminate_inline15_soa_64k<const SPIRALS: bool>(
        soa: &mut Fixed64kSoa,
        rules: &mut Vec<[u32; 4]>,
        n: u32,
    ) -> usize {
        rules.clear();
        let links = &mut *soa.links;
        let widths = &mut *soa.widths;
        let words = &mut *soa.words128;
        let mut left = n;
        let mut taken = 0usize;
        while left != 0 {
            let x = links[0].prev0;
            let y = links[0].prev1;
            let xi = usize::from(x);
            let yi = usize::from(y);
            let aw = widths[xi];
            let bw = widths[yi];
            if SPIRALS && x != y {
                if aw > bw && links[yi].prev1 == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    assert!(q * bw < aw && q * bw <= left);
                    words[yi] = fold_left_power(rules, words[xi], q, words[yi]);
                    widths[xi] = aw - q * bw;
                    left -= q * bw;
                    taken += q as usize;
                    continue;
                }
                if aw < bw && links[xi].prev0 == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    words[xi] = fold_right_power(rules, words[xi], words[yi], q);
                    widths[yi] = bw - q * aw;
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
                let xp = links[xi].prev0;
                links[usize::from(xp)].next0 = 0;
                links[0].prev0 = xp;
                let yp = links[yi].prev1;
                links[usize::from(yp)].next1 = 0;
                links[0].prev1 = yp;
                continue;
            }
            let word = join_u128(rules, words[xi], words[yi]);
            if aw > bw {
                let yp = links[yi].prev1;
                links[usize::from(yp)].next1 = 0;
                links[0].prev1 = yp;
                let xn = links[xi].next1;
                links[yi].prev1 = x;
                links[yi].next1 = xn;
                links[xi].next1 = y;
                links[usize::from(xn)].prev1 = y;
                words[yi] = word;
                widths[xi] = aw - bw;
            } else if aw < bw {
                let xp = links[xi].prev0;
                links[usize::from(xp)].next0 = 0;
                links[0].prev0 = xp;
                let yn = links[yi].next0;
                links[xi].prev0 = y;
                links[xi].next0 = yn;
                links[yi].next0 = x;
                links[usize::from(yn)].prev0 = x;
                words[xi] = word;
                widths[yi] = bw - aw;
            } else {
                let xp = links[xi].prev0;
                links[usize::from(xp)].next0 = 0;
                links[0].prev0 = xp;
                let yp0 = links[yi].prev0;
                let yn0 = links[yi].next0;
                links[xi].prev0 = yp0;
                links[xi].next0 = yn0;
                links[usize::from(yp0)].next0 = x;
                links[usize::from(yn0)].prev0 = x;
                let yp1 = links[yi].prev1;
                links[usize::from(yp1)].next1 = 0;
                links[0].prev1 = yp1;
                words[xi] = word;
            }
        }
        taken
    }

    /// Assign bottom coordinates from both ends of side 1 into the dead `(prev1, next1)` `u16` pair.
    fn assign_bottom_soa_64k(
        links: &mut [NodeLinks; NARROW_NODES],
        widths: &[u32; NARROW_NODES],
        rows: u32,
    ) {
        let mut first = links[0].next1;
        let mut last = links[0].prev1;
        let (mut lo, mut hi) = (0u32, rows);
        while first != last {
            let fi = usize::from(first);
            let li = usize::from(last);
            let next = links[fi].next1;
            let prev = links[li].prev1;
            let wf = widths[fi];
            hi -= widths[li];
            links[fi].prev1 = lo as u16;
            links[fi].next1 = (lo >> 16) as u16;
            links[li].prev1 = hi as u16;
            links[li].next1 = (hi >> 16) as u16;
            lo += wf;
            if next == last {
                assert_eq!(lo, hi);
                return;
            }
            first = next;
            last = prev;
        }
        if first != 0 {
            let fi = usize::from(first);
            links[fi].prev1 = lo as u16;
            links[fi].next1 = (lo >> 16) as u16;
            lo += widths[fi];
        }
        assert_eq!(lo, hi);
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn dict_code_u128(
        word: u128,
        byte: u8,
        rules: &mut [[u32; 4]],
        gen_cache: &mut GenCache,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        spelling: &mut Vec<u8>,
        stack: &mut Vec<u32>,
    ) -> u32 {
        let tag = (word >> 120) as u32;
        if tag < 16 {
            let entry_u128 = (word & U128_PAYLOAD_MASK) | (u128::from(byte) << (8 * tag));
            let len = (tag + 1) as u8;
            let idx = (short_u128_hash(entry_u128, len) as usize) & (GEN_CONTENT_SLOTS - 1);
            let g = gen_cache.content_gen;
            let slot = gen_cache.contents[idx];
            if (slot >> 16) == g {
                let old = (slot & 0xFFFF) as usize;
                if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                    return old as u32;
                }
            }
            let code = lens.len() as u32;
            inline16.push(entry_u128.to_le_bytes());
            lens.push(len);
            if code < 65_536 {
                gen_cache.contents[idx] = (g << 16) | code;
            }
            code
        } else {
            dict_code_rule_long(
                word as u32,
                byte,
                rules,
                gen_cache,
                inline16,
                lens,
                spelling,
                stack,
            )
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn dict_code_rule_long(
        id: u32,
        byte: u8,
        rules: &mut [[u32; 4]],
        gen_cache: &mut GenCache,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        spelling: &mut Vec<u8>,
        stack: &mut Vec<u32>,
    ) -> u32 {
        let key = (id << 8) | u32::from(byte);
        let p_idx = (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
        if id < (1 << 22) {
            let packed = gen_cache.pairs[p_idx];
            if (packed >> 46) == gen_cache.pair_gen && ((packed as u32) & 0x3FFF_FFFF) == key {
                return ((packed >> 30) & 0xFFFF) as u32;
            }
        }

        let start = spelling.len();
        let memoized = rules[(id - 257) as usize][3] != u32::MAX;
        append_rules(rules, id, stack, spelling);
        spelling.push(byte);
        let end = spelling.len();
        let entry = long_slot(start as u32, end as u32);

        let bytes = &spelling[start..end];
        let idx = (long_bytes_hash(bytes) as usize) & (GEN_CONTENT_SLOTS - 1);
        let g = gen_cache.content_gen;
        let slot = gen_cache.contents[idx];
        if (slot >> 16) == g {
            let old = (slot & 0xFFFF) as usize;
            if lens[old] == LONG && long_word(&inline16[old], spelling) == bytes {
                if memoized {
                    spelling.truncate(start);
                } else {
                    spelling.pop();
                }
                let code = old as u32;
                if id < (1 << 22) && code < 65_536 {
                    gen_cache.pairs[p_idx] =
                        (gen_cache.pair_gen << 46) | (u64::from(code) << 30) | u64::from(key);
                }
                return code;
            }
        }

        let code = lens.len() as u32;
        inline16.push(entry);
        lens.push(LONG);
        if code < 65_536 {
            gen_cache.contents[idx] = (g << 16) | code;
        }
        if id < (1 << 22) && code < 65_536 {
            gen_cache.pairs[p_idx] =
                (gen_cache.pair_gen << 46) | (u64::from(code) << 30) | u64::from(key);
        }
        code
    }

    impl Kernel {
        /// Redesigned Narrow `A6b` path: fused setup + `Fixed64kSoa` `inline15` `u128` induction +
        /// generation-tagged L1d dictionary. Called only when `!self.wide(ns)`.
        pub(in crate::open) fn induced_redesigned(
            &mut self,
            ns: usize,
            n: u32,
            first: &[u32; 256],
            write: &mut [u32; 256],
            thin: &[u32; 256],
            stats: &mut Counts,
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
            let strips_raw = &edges[..3 * ns];
            let redesign = &mut endpoint.redesign;
            let soa = redesign.fixed_soa.get_or_insert_with(Fixed64kSoa::new);
            let (widest, nodes_used) = setup_soa_64k(
                strips_raw,
                ns,
                n,
                first,
                write,
                &mut redesign.image_order64,
                soa,
                #[cfg(test)]
                endpoint.clobber,
            );
            stats.open_nodes = nodes_used - 1;

            #[cfg(test)]
            let spirals = endpoint.spirals.unwrap_or(widest > CERTIFIED_WIDTH);
            #[cfg(not(test))]
            let spirals = widest > CERTIFIED_WIDTH;
            stats.spirals = spirals;

            let steps = if spirals {
                eliminate_inline15_soa_64k::<true>(soa, &mut redesign.rules, n)
            } else {
                eliminate_inline15_soa_64k::<false>(soa, &mut redesign.rules, n)
            };
            stats.steps = steps;
            stats.rules = redesign.rules.len();

            #[cfg(test)]
            if endpoint.clobber == Clobber::ArenaBeforeBands {
                soa.widths[..nodes_used].fill(0xA5A5_A5A5);
            }

            assign_bottom_soa_64k(&mut soa.links, &soa.widths, stats.retained as u32);
            redesign.gen_cache.next_block();
            begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);

            let mut cursors = [0usize; 256];
            let links = &*soa.links;
            let widths = &*soa.widths;
            let words = &*soa.words128;
            let rules = &mut redesign.rules[..];

            let mut id = links[0].next0;
            let mut node_width = widths[usize::from(id)];
            let mut node_word = words[usize::from(id)];
            let l0 = links[usize::from(id)];
            let mut node_bottom = u32::from(l0.prev1) | (u32::from(l0.next1) << 16);
            let mut node_next = l0.next0;
            let mut in_node = 0u32;
            let mut k = 0u32;

            for a in 0..256 {
                cursors[a] = bands.len();
                let end = k + thin[a];
                while k < end {
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    #[cfg(any(test, feature = "census"))]
                    if node_word != 0 {
                        let tag = (node_word >> 120) as u32;
                        let height = if tag < 16 {
                            tag as usize
                        } else {
                            (node_word >> 32) as u32 as usize
                        };
                        stats.words += 1;
                        stats.copies += width as usize;
                        stats.traced += height;
                        stats.thick_area += width as usize * height;
                        stats.unit_work += usize::from(width == 1) * height;
                        stats.longest = stats.longest.max(height);
                    }
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        dict_code_u128(
                            node_word,
                            a as u8,
                            rules,
                            &mut redesign.gen_cache,
                            inline16,
                            lens,
                            spelling,
                            &mut endpoint.stack,
                        )
                    };
                    bands.push(Band::new(k + width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = node_next;
                        in_node = 0;
                        if id != 0 {
                            let idx = usize::from(id);
                            let l = links[idx];
                            node_width = widths[idx];
                            node_word = words[idx];
                            node_bottom = u32::from(l.prev1) | (u32::from(l.next1) << 16);
                            node_next = l.next0;
                        }
                    }
                }
            }
            assert_eq!(id, 0);
            cursors
        }
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

    #[cfg(feature = "a6b-endpoint-redesign")]
    #[test]
    fn redesigned_matches_wide_and_spirals_and_zero_strips() {
        use super::super::b5::{plain, runs_column, Lcg};
        use super::super::LIMIT;
        use crate::runs::Runs;

        let mut g = Lcg(0x6136_6231);
        let mut cases: Vec<(Vec<u8>, u32)> = vec![
            // ns == 0 (all thin runs)
            ((0..500).map(|i| (i % 7) as u8).collect(), 123),
            // single long run (spiral candidate)
            (
                {
                    let mut v = vec![3u8; 40_000];
                    v[0] = 1;
                    v[19_999] = 2;
                    v
                },
                7,
            ),
            // mixed dense & sparse runs (both short inline-15 and long rules)
            (runs_column(&mut g, 8_000, 12, 18), 401),
            (runs_column(&mut g, 40_000, 35, 14), 12_345),
        ];
        // Dense comb with deep words (> 15 bytes) forcing long rules & dict dedup
        let comb: Vec<u8> = (0..12_000)
            .map(|i| {
                if i % 40 < 20 {
                    (i / 40 % 4) as u8
                } else {
                    4 + (i % 3) as u8
                }
            })
            .collect();
        cases.push((comb, 333));

        let mut redesigned = Kernel::default();
        let mut wide = Kernel::default();
        wide.endpoint.force_wide = true;
        let (mut out_r, mut out_w) = (Vec::new(), Vec::new());

        for (l, p) in &cases {
            let r = Runs::of_column(l);
            let want = plain(l, *p);
            for spirals in [Some(false), Some(true), None] {
                redesigned.endpoint.spirals = spirals;
                wide.endpoint.spirals = spirals;
                let sr = redesigned.run::<true, true>(r.records(), l.len(), *p, LIMIT, &mut out_r);
                let sw = wide.run::<true, true>(r.records(), l.len(), *p, LIMIT, &mut out_w);
                assert_eq!(
                    out_r,
                    want,
                    "redesigned mismatch len={} spirals={spirals:?}",
                    l.len()
                );
                assert_eq!(
                    out_w,
                    want,
                    "wide mismatch len={} spirals={spirals:?}",
                    l.len()
                );
                assert_eq!(sr.steps, sw.steps);
                assert_eq!(sr.open_nodes, sw.open_nodes);
                assert_eq!(sr.words, sw.words);
                assert_eq!(sr.traced, sw.traced);
                assert_eq!(sr.longest, sw.longest);
                assert!(sr.rules <= sw.rules);
            }
        }
    }

    #[cfg(feature = "a6b-endpoint-redesign")]
    #[test]
    fn redesigned_gen_cache_wrap_and_reuse() {
        use super::super::b5::{plain, runs_column, Lcg};
        use super::super::LIMIT;
        use crate::runs::Runs;

        let mut g = Lcg(0x6765_6e77);
        let mut k = Kernel::default();
        let mut out = Vec::new();
        // Force both content_gen (16-bit) and pair_gen (18-bit) to wrap on the next block.
        k.endpoint.redesign.gen_cache.content_gen = (1u32 << 16) - 2;
        k.endpoint.redesign.gen_cache.pair_gen = (1u64 << 18) - 2;
        for _ in 0..5 {
            let l = runs_column(&mut g, 6_000, 8, 22);
            let p = g.below(l.len() as u32);
            let r = Runs::of_column(&l);
            k.run::<true, true>(r.records(), l.len(), p, LIMIT, &mut out);
            assert_eq!(out, plain(&l, p));
        }
        assert!(k.endpoint.redesign.gen_cache.content_gen < 10);
        assert!(k.endpoint.redesign.gen_cache.pair_gen < 10);
    }

    #[cfg(feature = "a6b-endpoint-redesign")]
    #[test]
    fn miri_a6b_endpoint_redesign() {
        use super::super::b5::{plain, runs_column, Lcg};
        use super::super::LIMIT;
        use crate::runs::Runs;

        let mut g = Lcg(0x6d69_7269);
        let mut k = Kernel::default();
        let mut out = Vec::new();
        // Small columns exercising ns == 0, inline-15 words, long rules, and SPIRALS=true/false
        let small_spiral: Vec<u8> = (0..180).map(|i| if i == 0 { 1 } else { 0 }).collect();
        let long_word: Vec<u8> = (0..400)
            .map(|i| if i % 22 < 18 { (i / 22 % 3) as u8 } else { 3 })
            .collect();
        let mixed = runs_column(&mut g, 320, 6, 18);
        for (l, p) in [
            ((0..64).map(|i| (i % 5) as u8).collect::<Vec<_>>(), 11u32),
            (small_spiral, 5),
            (long_word, 77),
            (mixed, 19),
        ] {
            let r = Runs::of_column(&l);
            let want = plain(&l, p);
            for spirals in [Some(false), Some(true)] {
                k.endpoint.spirals = spirals;
                k.run::<true, true>(r.records(), l.len(), p, LIMIT, &mut out);
                assert_eq!(out, want);
            }
        }
    }

    #[cfg(not(feature = "a6b-endpoint-redesign"))]
    #[test]
    fn overlay_aos_matches_wide_and_spirals() {
        use super::super::b5::{plain, runs_column, Lcg};
        use super::super::LIMIT;
        use crate::runs::Runs;

        let mut g = Lcg(0x6136_6232);
        let mut cases: Vec<(Vec<u8>, u32)> = vec![
            (runs_column(&mut g, 8_000, 12, 18), 401),
            (runs_column(&mut g, 40_000, 35, 14), 12_345),
        ];
        // Dense comb with >= 64 strips and deep (> 15 byte) words forcing long rules & dict dedup
        let comb: Vec<u8> = (0..12_000)
            .map(|i| {
                if i % 40 < 20 {
                    (i / 40 % 4) as u8
                } else {
                    4 + (i % 3) as u8
                }
            })
            .collect();
        cases.push((comb, 333));

        let mut overlay = Kernel::default();
        let mut wide = Kernel::default();
        wide.endpoint.force_wide = true;
        let (mut out_o, mut out_w) = (Vec::new(), Vec::new());

        for (l, p) in &cases {
            let r = Runs::of_column(l);
            let want = plain(l, *p);
            for spirals in [Some(false), Some(true), None] {
                overlay.endpoint.spirals = spirals;
                wide.endpoint.spirals = spirals;
                let so = overlay.run::<true, true>(r.records(), l.len(), *p, LIMIT, &mut out_o);
                let sw = wide.run::<true, true>(r.records(), l.len(), *p, LIMIT, &mut out_w);
                assert!(so.thick >= 64, "test case must exercise overlay_aos path");
                assert_eq!(
                    out_o,
                    want,
                    "overlay mismatch len={} spirals={spirals:?}",
                    l.len()
                );
                assert_eq!(
                    out_w,
                    want,
                    "wide mismatch len={} spirals={spirals:?}",
                    l.len()
                );
                assert_eq!(so.steps, sw.steps);
                assert_eq!(so.open_nodes, sw.open_nodes);
                assert_eq!(so.words, sw.words);
                assert_eq!(so.traced, sw.traced);
                assert_eq!(so.longest, sw.longest);
                assert!(so.rules <= sw.rules);
                assert_eq!(overlay.endpoint.scratch(), 0);
            }
        }
    }

    #[cfg(not(feature = "a6b-endpoint-redesign"))]
    #[test]
    fn miri_a6b_overlay_aos() {
        use super::super::b5::{plain, runs_column, Lcg};
        use super::super::LIMIT;
        use crate::runs::Runs;

        let mut g = Lcg(0x6d69_7232);
        let mut k = Kernel::default();
        let mut out = Vec::new();
        // >= 64 thick strips to exercise overlay_aos setup, elim (both SPIRALS=false/true),
        // assign_bottom, dict (inline-7, inline-15, long rules), and 0 B Endpoint heap.
        let comb: Vec<u8> = (0..1_600)
            .map(|i| {
                if i % 22 < 18 {
                    (i / 22 % 4) as u8
                } else {
                    4 + (i % 3) as u8
                }
            })
            .collect();
        let mixed = runs_column(&mut g, 2_400, 8, 18);
        for (l, p) in [(comb, 77u32), (mixed, 19u32)] {
            let r = Runs::of_column(&l);
            let want = plain(&l, p);
            for spirals in [Some(false), Some(true)] {
                k.endpoint.spirals = spirals;
                let s = k.run::<true, true>(r.records(), l.len(), p, LIMIT, &mut out);
                assert!(s.thick >= 64, "miri case must exercise overlay_aos");
                assert_eq!(out, want);
                assert_eq!(k.endpoint.scratch(), 0);
            }
        }
    }
}
