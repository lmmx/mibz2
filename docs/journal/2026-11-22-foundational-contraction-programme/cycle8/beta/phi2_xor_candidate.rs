#![forbid(unsafe_code)]
//! Investigator Beta — Cycle 8 Candidate:
//! `Phi2XorOpen`: Zero-Sort, Zero-Bigram-Hash Bijective Full `Phi^2` + Theorem B6 Universal `wire_xor`!

use crate::bwt::{InverseBwt, OpenCells};
use crate::{runs::Runs, Kernels};

pub type Candidate = Phi2XorOpen;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;

const DENSE_NUM: usize = 7;
const DENSE_DEN: usize = 25;
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0u32;
    counts.map(|w| {
        let s = total;
        total += w;
        s
    })
}

fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
    let mut slot = [0u8; INLINE];
    slot[..4].copy_from_slice(&start.to_le_bytes());
    slot[4..8].copy_from_slice(&end.to_le_bytes());
    slot
}

#[inline(always)]
fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
    let start = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]) as usize;
    let end = u32::from_le_bytes([slot[4], slot[5], slot[6], slot[7]]) as usize;
    &spelling[start..end]
}

trait Links {
    const STRIDE: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
}

struct Narrow;
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
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
const ID_BITS: u32 = 22;
const ID_MASK: u32 = (1 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

#[cold]
#[inline(never)]
fn grow(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
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
    fn make_bigram_word(&mut self, b0: u8, b1: u8) -> u32 {
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let low = u32::from(b0) | (u32::from(b1) << 8);
        self.a[self.top..self.top + RULE].copy_from_slice(&[low, 0, 0, 2u32 << 24]);
        self.top += RULE;
        (2u32 << ID_BITS) | id
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

    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
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
        *remaining = left;
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

struct ThickDict<'a> {
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    table: &'a mut [u32],
}

impl<'a> ThickDict<'a> {
    #[inline(always)]
    fn intern_short(&mut self, bytes: [u8; INLINE], len: u8) -> u32 {
        let val = u128::from_le_bytes(bytes);
        let lo = val as u64;
        let hi = (val >> 64) as u64;
        let h = lo
            .wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ hi.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ u64::from(len).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mask = self.table.len() - 1;
        let slot = ((h ^ (h >> 30)) as usize) & mask;
        let prev = self.table[slot];
        if prev != 0 {
            let idx = (prev - 1) as usize;
            if self.lens[idx] == len && self.inline16[idx] == bytes {
                return prev - 1;
            }
        }
        let slot2 = (slot + 1) & mask;
        let prev2 = self.table[slot2];
        if prev2 != 0 {
            let idx = (prev2 - 1) as usize;
            if self.lens[idx] == len && self.inline16[idx] == bytes {
                return prev2 - 1;
            }
        }
        let code = self.lens.len() as u32;
        self.inline16.push(bytes);
        self.lens.push(len);
        if prev == 0 {
            self.table[slot] = code + 1;
        } else {
            self.table[slot2] = code + 1;
        }
        code
    }

    fn intern_long(&mut self, start: u32, end: u32) -> u32 {
        let slice = &self.spelling[start as usize..end as usize];
        let l = slice.len();
        let part = |i: usize| u64::from_le_bytes(slice[i..i + 8].try_into().unwrap());
        let h = part(0)
            .wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ part(l - 8).rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ part(l / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
            ^ (l as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mask = self.table.len() - 1;
        let slot = ((h ^ (h >> 30)) as usize) & mask;
        let prev = self.table[slot];
        if prev != 0 {
            let idx = (prev - 1) as usize;
            if self.lens[idx] == LONG && long_word(&self.inline16[idx], self.spelling) == slice {
                return prev - 1;
            }
        }
        let code = self.lens.len() as u32;
        self.inline16.push(long_slot(start, end));
        self.lens.push(LONG);
        self.table[slot] = code + 1;
        code
    }

    fn compose_bigram_thick<L: Links>(
        &mut self,
        b0: u8,
        b1: u8,
        thick_handle: u32,
        e: &mut Elim<'_, L>,
        stack: &mut Vec<u32>,
    ) -> u32 {
        let id = thick_handle & ID_MASK;
        let tk_len = (thick_handle >> ID_BITS) as usize;
        if tk_len < SATURATED as usize && 2 + tk_len <= INLINE {
            let tk_bytes = e.short_payload(id);
            let combined = u128::from(b0) | (u128::from(b1) << 8) | (tk_bytes << 16);
            return self.intern_short(combined.to_le_bytes(), (2 + tk_len) as u8);
        }
        let start = self.spelling.len() as u32;
        self.spelling.push(b0);
        self.spelling.push(b1);
        e.append(id, stack, self.spelling);
        let end = self.spelling.len() as u32;
        let tot = (end - start) as usize;
        if tot <= INLINE {
            let mut b = [0u8; INLINE];
            b[..tot].copy_from_slice(&self.spelling[start as usize..end as usize]);
            self.intern_short(b, tot as u8)
        } else {
            self.intern_long(start, end)
        }
    }
}

const STRIP_THICK_BIT: u32 = 1 << 31;
const STRIP_LEN_MASK: u32 = 0x7FFF_FFFF;

#[derive(Clone, Copy)]
struct P2Strip {
    dlo: u32,
    ilo_or_reld: u32,
    len_flags: u32,
    b0_b1_code: u32,
}

#[derive(Clone, Copy)]
struct Thick2 {
    dlo: u32,
    ilo: u32,
    len: u32,
    b0: u8,
    b1: u8,
}

#[derive(Clone, Copy)]
struct Band2 {
    end_k: u32,
    dst_rank: u32,
    thick_handle: u32,
}

pub struct Phi2XorOpenOf<const LIM: u32> {
    fallback: OpenCells,
    run_len_c: Vec<u32>,
    run_l_start: Vec<u32>,
    f_order_v: Vec<u32>,
    p2_strips: Vec<P2Strip>,
    v_start: Vec<u32>,
    v_end: Vec<u32>,
    bigram_lut: Vec<u16>,
    bigram_touched: Vec<u16>,
    thick_dom: Vec<Thick2>,
    thick_im_order: Vec<u32>,
    arena: Vec<u32>,
    stack: Vec<u32>,
    bands: Vec<Band2>,
    dict_table: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
}

pub type Phi2XorOpen = Phi2XorOpenOf<LIMIT>;

impl<const LIM: u32> Default for Phi2XorOpenOf<LIM> {
    fn default() -> Self {
        Self {
            fallback: OpenCells::default(),
            run_len_c: Vec::new(),
            run_l_start: Vec::new(),
            f_order_v: Vec::new(),
            p2_strips: Vec::new(),
            v_start: Vec::new(),
            v_end: Vec::new(),
            bigram_lut: Vec::new(),
            bigram_touched: Vec::new(),
            thick_dom: Vec::new(),
            thick_im_order: Vec::new(),
            arena: Vec::new(),
            stack: Vec::new(),
            bands: Vec::new(),
            dict_table: Vec::new(),
            inline16: Vec::new(),
            lens: Vec::new(),
            spelling: Vec::new(),
            edges: Vec::new(),
            extra: Vec::new(),
        }
    }
}

impl<const LIM: u32> Phi2XorOpenOf<LIM> {
    fn ensure_luts(&mut self) {
        if self.bigram_lut.len() < 65_536 {
            self.bigram_lut.resize(65_536, 0);
        }
        if self.dict_table.len() < 16_384 {
            self.dict_table.resize(16_384, 0);
        }
    }

    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn run_endpoint<L: Links>(&mut self, ns: usize, n: u32, m2: u32, most: usize) {
        let Self {
            arena,
            thick_dom,
            thick_im_order,
            bands,
            ..
        } = self;
        let nodes_words = L::STRIDE * most;
        let need = nodes_words + (3 * ns + 64) * RULE;
        if arena.len() < need {
            arena.resize(need, 0);
        }

        let mut e = Elim::<L> {
            a: arena.as_mut_slice(),
            base: nodes_words,
            top: nodes_words,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0u32;
        for i in 0..ns {
            let s = thick_dom[i];
            let wh = e.make_bigram_word(s.b0, s.b1);
            e.node(s.len, wh);
            widest = widest.max(s.len);
        }

        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0u32;
        for i in 0..ns {
            let s = thick_dom[i];
            if x < s.dlo {
                widest = widest.max(s.dlo - x);
                let id = e.node(s.dlo - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i as u32 + 1, 1);
            x = s.dlo + s.len;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[1], id, 1);
            e.append_to(&mut terminals[0], id, 0);
        }

        x = 0;
        for &i in thick_im_order.iter() {
            let s = thick_dom[i as usize];
            if x < s.ilo {
                widest = widest.max(s.ilo - x);
                let id = e.node(s.ilo - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            e.append_to(&mut order[0], i + 1, 0);
            x = s.ilo + s.len;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }

        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        let mut remaining = n;
        let spirals = widest > CERTIFIED_WIDTH;
        while !if spirals {
            e.eliminate::<true>(&mut remaining)
        } else {
            e.eliminate::<false>(&mut remaining)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(arena, top + SPIRAL_RULES * RULE);
            e = Elim {
                a: arena.as_mut_slice(),
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }

        {
            let mut first = L::next(e.a, 0, 0);
            let mut last = L::prev(e.a, 0, 0);
            let (mut lo, mut hi) = (0u32, m2);
            while first != last {
                let next = L::next(e.a, first, 0);
                let prev = L::prev(e.a, last, 0);
                let wf = e.width(first);
                hi -= e.width(last);
                e.a[first as usize * L::STRIDE] = lo;
                e.a[last as usize * L::STRIDE] = hi;
                lo += wf;
                if next == last {
                    assert_eq!(lo, hi);
                    break;
                }
                first = next;
                last = prev;
            }
            if first != 0 && first == last {
                e.a[first as usize * L::STRIDE] = lo;
                lo += e.width(first);
                assert_eq!(lo, hi);
            }
        }

        bands.clear();
        let mut id = L::next(e.a, 0, 1);
        let mut k = 0u32;
        while id != 0 {
            let w = e.width(id);
            let dst_rank = e.a[id as usize * L::STRIDE];
            let thick_handle = e.word(id);
            k += w;
            bands.push(Band2 {
                end_k: k,
                dst_rank,
                thick_handle,
            });
            id = L::next(e.a, id, 1);
        }
        assert_eq!(k, m2);
    }

    pub fn try_invert_runs_profiled(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap_m: u32,
        cap_strips: usize,
        out: &mut Vec<u8>,
        profile: bool,
    ) -> bool {
        let t0 = if profile { Some(std::time::Instant::now()) } else { None };
        let n32 = n as u32;
        self.ensure_luts();

        for &k in &self.bigram_touched {
            self.bigram_lut[usize::from(k)] = 0;
        }
        self.bigram_touched.clear();
        self.inline16.clear();
        self.lens.clear();
        self.spelling.clear();
        self.dict_table.fill(0);

        self.run_len_c.clear();
        self.run_l_start.clear();
        let mut counts = [0u32; 256];
        let mut run_cnt = [0u32; 256];
        let mut pos = 0u32;
        let mut i = 0usize;
        let mut p_c = 0u8;
        let mut p_rel_f = 0u32;

        while i < records.len() {
            let rec = records[i];
            let c = (rec >> 8) as u8;
            let mut width = u32::from(rec & 255);
            i += 1;
            while i < records.len() && (records[i] >> 8) as u8 == c {
                width += u32::from(records[i] & 255);
                i += 1;
            }
            let a = usize::from(c);
            if p.wrapping_sub(pos) < width {
                p_c = c;
                p_rel_f = counts[a] + (p - pos);
            }
            self.run_len_c.push((width << 8) | u32::from(c));
            self.run_l_start.push(pos);
            counts[a] += width;
            run_cnt[a] += 1;
            pos += width;
        }
        if pos != n32 {
            return false;
        }
        if n == 1 {
            out.clear();
            out.push(p_c);
            return true;
        }

        let c_starts = starts(counts);
        let lf_p = c_starts[usize::from(p_c)] + p_rel_f;
        let byte_before_p = p_c;
        let r0 = self.run_len_c.len();

        let mut f_write = starts(run_cnt);
        self.f_order_v.resize(r0, 0);
        for (idx, &lc) in self.run_len_c.iter().enumerate() {
            let a = (lc & 0xFF) as usize;
            let dst = f_write[a] as usize;
            f_write[a] += 1;
            self.f_order_v[dst] = idx as u32;
        }
        let t_s1 = t0.map(|t| t.elapsed().as_secs_f64() * 1e3);

        self.p2_strips.clear();
        self.p2_strips.reserve(r0 + r0 / 3);
        self.v_start.resize(r0, 0);
        self.v_end.resize(r0, 0);

        let mut f_cur_u = c_starts;
        let mut thin_cnt_b0 = [0u32; 256];
        let mut thick_cnt_b0 = [0u32; 256];
        let mut m2 = 0u32;
        let mut ns2 = 0usize;
        let mut mark_b0 = 0usize;
        let mut mark_rel = 0u32;
        let mut lfp_b0 = 0usize;
        let mut lfp_rel = 0u32;

        let mut emit_p2 = |dlo: u32,
                           ilo: u32,
                           w: u32,
                           b0: u8,
                           b1: u8,
                           is_thin: bool,
                           p2_strips: &mut Vec<P2Strip>,
                           bigram_lut: &mut [u16],
                           bigram_touched: &mut Vec<u16>,
                           inline16: &mut Vec<[u8; INLINE]>,
                           lens: &mut Vec<u8>| -> bool {
            let a0 = usize::from(b0);
            if is_thin {
                let rel_d = thin_cnt_b0[a0];
                if p.wrapping_sub(dlo) < w {
                    mark_b0 = a0;
                    mark_rel = rel_d + (p - dlo);
                }
                if lf_p.wrapping_sub(dlo) < w {
                    lfp_b0 = a0;
                    lfp_rel = rel_d + (lf_p - dlo);
                }
                thin_cnt_b0[a0] = rel_d + w;
                m2 += w;
                if m2 > cap_m {
                    return false;
                }
                let key = (usize::from(b0) << 8) | usize::from(b1);
                let mut code16 = bigram_lut[key];
                if code16 == 0 {
                    let c = inline16.len() as u16;
                    let mut entry = [0u8; INLINE];
                    entry[0] = b0;
                    entry[1] = b1;
                    inline16.push(entry);
                    lens.push(2);
                    code16 = c + 1;
                    bigram_lut[key] = code16;
                    bigram_touched.push(key as u16);
                }
                let bcode = u32::from(code16 - 1);
                p2_strips.push(P2Strip {
                    dlo,
                    ilo_or_reld: rel_d,
                    len_flags: w,
                    b0_b1_code: u32::from(b0) | (u32::from(b1) << 8) | (bcode << 16),
                });
            } else {
                let rel_tk = thick_cnt_b0[a0];
                thick_cnt_b0[a0] = rel_tk + 1;
                ns2 += 1;
                if ns2 > cap_strips {
                    return false;
                }
                p2_strips.push(P2Strip {
                    dlo,
                    ilo_or_reld: ilo,
                    len_flags: STRIP_THICK_BIT | w,
                    b0_b1_code: u32::from(b0) | (u32::from(b1) << 8) | (rel_tk << 16),
                });
            }
            true
        };

        let mut u = 0usize;
        let mut iv = 0usize;
        let mut ou = 0u32;
        let mut ov = 0u32;
        let mut v_cur = self.f_order_v[0] as usize;
        self.v_start[v_cur] = 0;

        while u < r0 && iv < r0 {
            let lc_u = self.run_len_c[u];
            let len_u = lc_u >> 8;
            let b0 = (lc_u & 0xFF) as u8;
            let lc_v = self.run_len_c[v_cur];
            let len_v = lc_v >> 8;
            let b1 = (lc_v & 0xFF) as u8;

            let take = (len_u - ou).min(len_v - ov);
            let dlo = f_cur_u[usize::from(b0)] + ou;
            let ilo = self.run_l_start[v_cur] + ov;

            if take <= LIM {
                if !emit_p2(
                    dlo,
                    ilo,
                    take,
                    b0,
                    b1,
                    true,
                    &mut self.p2_strips,
                    &mut self.bigram_lut,
                    &mut self.bigram_touched,
                    &mut self.inline16,
                    &mut self.lens,
                ) {
                    return false;
                }
            } else {
                let has_p = p.wrapping_sub(dlo) < take;
                let has_lfp = lf_p.wrapping_sub(dlo) < take;
                if !has_p && !has_lfp {
                    if !emit_p2(
                        dlo,
                        ilo,
                        take,
                        b0,
                        b1,
                        false,
                        &mut self.p2_strips,
                        &mut self.bigram_lut,
                        &mut self.bigram_touched,
                        &mut self.inline16,
                        &mut self.lens,
                    ) {
                        return false;
                    }
                } else {
                    let mut cuts = [u32::MAX; 2];
                    let mut nc = 0usize;
                    if has_p {
                        cuts[nc] = p;
                        nc += 1;
                    }
                    if has_lfp && (!has_p || lf_p != p) {
                        cuts[nc] = lf_p;
                        nc += 1;
                    }
                    if nc == 2 && cuts[0] > cuts[1] {
                        cuts.swap(0, 1);
                    }
                    let mut cur = dlo;
                    let end = dlo + take;
                    for &cx in &cuts[..nc] {
                        if cur < cx {
                            if !emit_p2(
                                cur,
                                ilo + (cur - dlo),
                                cx - cur,
                                b0,
                                b1,
                                false,
                                &mut self.p2_strips,
                                &mut self.bigram_lut,
                                &mut self.bigram_touched,
                                &mut self.inline16,
                                &mut self.lens,
                            ) {
                                return false;
                            }
                        }
                        if !emit_p2(
                            cx,
                            ilo + (cx - dlo),
                            1,
                            b0,
                            b1,
                            true,
                            &mut self.p2_strips,
                            &mut self.bigram_lut,
                            &mut self.bigram_touched,
                            &mut self.inline16,
                            &mut self.lens,
                        ) {
                            return false;
                        }
                        cur = cx + 1;
                    }
                    if cur < end {
                        if !emit_p2(
                            cur,
                            ilo + (cur - dlo),
                            end - cur,
                            b0,
                            b1,
                            false,
                            &mut self.p2_strips,
                            &mut self.bigram_lut,
                            &mut self.bigram_touched,
                            &mut self.inline16,
                            &mut self.lens,
                        ) {
                            return false;
                        }
                    }
                }
            }

            ou += take;
            ov += take;
            if ou == len_u {
                f_cur_u[usize::from(b0)] += len_u;
                u += 1;
                ou = 0;
            }
            if ov == len_v {
                self.v_end[v_cur] = self.p2_strips.len() as u32;
                iv += 1;
                ov = 0;
                if iv < r0 {
                    v_cur = self.f_order_v[iv] as usize;
                    self.v_start[v_cur] = self.p2_strips.len() as u32;
                }
            }
        }
        assert!(u == r0 && iv == r0 && m2 > 0);

        let thin_start_b0 = starts(thin_cnt_b0);
        let mark = thin_start_b0[mark_b0] + mark_rel;
        let lfp = thin_start_b0[lfp_b0] + lfp_rel;
        let t_s1b = t0.map(|t| t.elapsed().as_secs_f64() * 1e3);

        let thick_start_b0 = starts(thick_cnt_b0);
        self.thick_dom.resize(
            ns2,
            Thick2 {
                dlo: 0,
                ilo: 0,
                len: 0,
                b0: 0,
                b1: 0,
            },
        );
        self.thick_im_order.clear();
        self.thick_im_order.reserve(ns2);

        for v in 0..r0 {
            let s0 = self.v_start[v] as usize;
            let s1 = self.v_end[v] as usize;
            for s in &self.p2_strips[s0..s1] {
                if (s.len_flags & STRIP_THICK_BIT) != 0 {
                    let b0 = (s.b0_b1_code & 0xFF) as u8;
                    let b1 = ((s.b0_b1_code >> 8) & 0xFF) as u8;
                    let rel_tk = s.b0_b1_code >> 16;
                    let slot = thick_start_b0[usize::from(b0)] + rel_tk;
                    self.thick_dom[slot as usize] = Thick2 {
                        dlo: s.dlo,
                        ilo: s.ilo_or_reld,
                        len: s.len_flags & STRIP_LEN_MASK,
                        b0,
                        b1,
                    };
                    self.thick_im_order.push(slot);
                }
            }
        }

        let most = Self::nodes_most(ns2);
        let wide_links = most > NARROW_NODES;
        if wide_links {
            self.run_endpoint::<Wide>(ns2, n32, m2, most);
        } else {
            self.run_endpoint::<Narrow>(ns2, n32, m2, most);
        }
        let t_s2 = t0.map(|t| t.elapsed().as_secs_f64() * 1e3);

        let m2_usize = m2 as usize;
        let bits = (u32::BITS - (m2 - 1).leading_zeros()).max(1);
        let max_code_u32 = u32::MAX >> bits;
        let use_xor = m2 >= 64 && (self.lens.len() as u32) <= max_code_u32;

        if self.edges.len() < m2_usize {
            self.edges.resize(m2_usize, 0);
        }

        let mut pred_mark = mark;
        let mut pred_lfp = lfp;
        let mut overflowed_u32 = false;
        let nodes_words = if wide_links { Wide::STRIDE } else { Narrow::STRIDE } * most;

        if use_xor {
            let edges = &mut self.edges[..m2_usize];
            edges.fill(0);
            let mut dict = ThickDict {
                inline16: &mut self.inline16,
                lens: &mut self.lens,
                spelling: &mut self.spelling,
                table: &mut self.dict_table,
            };
            let mut bi = 0usize;
            let mut b_start = 0u32;
            let mut cur_k = 0u32;
            let mut last_thick_handle = u32::MAX;
            let mut last_b0_b1 = u32::MAX;
            let mut last_thick_code = 0u32;

            'wire_loop: for v in 0..r0 {
                let s0 = self.v_start[v] as usize;
                let s1 = self.v_end[v] as usize;
                for s in &self.p2_strips[s0..s1] {
                    if (s.len_flags & STRIP_THICK_BIT) != 0 {
                        continue;
                    }
                    let b0 = (s.b0_b1_code & 0xFF) as usize;
                    let mut src = thin_start_b0[b0] + s.ilo_or_reld;
                    let mut rem = s.len_flags;
                    let bcode_pure = s.b0_b1_code >> 16;
                    let b0_b1_key = s.b0_b1_code & 0xFFFF;

                    while rem > 0 {
                        let bnd = self.bands[bi];
                        let step = rem.min(bnd.end_k - cur_k);
                        let mut dst = bnd.dst_rank + (cur_k - b_start);
                        let code = if bnd.thick_handle == 0 {
                            bcode_pure
                        } else if bnd.thick_handle == last_thick_handle && b0_b1_key == last_b0_b1 {
                            last_thick_code
                        } else {
                            let b0_u8 = b0 as u8;
                            let b1_u8 = ((s.b0_b1_code >> 8) & 0xFF) as u8;
                            let arena_len = self.arena.len();
                            let c = if wide_links {
                                let mut e = Elim::<Wide> {
                                    a: self.arena.as_mut_slice(),
                                    base: nodes_words,
                                    top: arena_len,
                                    nodes: 0,
                                    _links: std::marker::PhantomData,
                                };
                                dict.compose_bigram_thick(b0_u8, b1_u8, bnd.thick_handle, &mut e, &mut self.stack)
                            } else {
                                let mut e = Elim::<Narrow> {
                                    a: self.arena.as_mut_slice(),
                                    base: nodes_words,
                                    top: arena_len,
                                    nodes: 0,
                                    _links: std::marker::PhantomData,
                                };
                                dict.compose_bigram_thick(b0_u8, b1_u8, bnd.thick_handle, &mut e, &mut self.stack)
                            };
                            last_thick_handle = bnd.thick_handle;
                            last_b0_b1 = b0_b1_key;
                            last_thick_code = c;
                            c
                        };
                        if code > max_code_u32 {
                            overflowed_u32 = true;
                            break 'wire_loop;
                        }
                        let pack = code << bits;
                        for _ in 0..step {
                            if dst == mark {
                                pred_mark = src;
                            }
                            if dst == lfp {
                                pred_lfp = src;
                            }
                            edges[src as usize] ^= pack | dst;
                            edges[dst as usize] ^= src;
                            src += 1;
                            dst += 1;
                        }
                        cur_k += step;
                        rem -= step;
                        if cur_k == bnd.end_k {
                            b_start = bnd.end_k;
                            bi += 1;
                        }
                    }
                }
            }
        } else {
            overflowed_u32 = true;
        }
        let t_s3 = t0.map(|t| t.elapsed().as_secs_f64() * 1e3);

        if !overflowed_u32 {
            emit_phi2_bidirectional(
                &self.edges[..m2_usize],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                pred_mark,
                lfp,
                pred_lfp,
                byte_before_p,
                bits,
                out,
            );
        } else {
            self.extra.resize(m2_usize, 0);
            let edges = &mut self.edges[..m2_usize];
            let extra = &mut self.extra[..m2_usize];
            let mut dict = ThickDict {
                inline16: &mut self.inline16,
                lens: &mut self.lens,
                spelling: &mut self.spelling,
                table: &mut self.dict_table,
            };
            let mut bi = 0usize;
            let mut b_start = 0u32;
            let mut cur_k = 0u32;
            for v in 0..r0 {
                let s0 = self.v_start[v] as usize;
                let s1 = self.v_end[v] as usize;
                for s in &self.p2_strips[s0..s1] {
                    if (s.len_flags & STRIP_THICK_BIT) != 0 {
                        continue;
                    }
                    let b0 = (s.b0_b1_code & 0xFF) as usize;
                    let mut src = thin_start_b0[b0] + s.ilo_or_reld;
                    let mut rem = s.len_flags;
                    let bcode_pure = s.b0_b1_code >> 16;
                    while rem > 0 {
                        let bnd = self.bands[bi];
                        let step = rem.min(bnd.end_k - cur_k);
                        let mut dst = bnd.dst_rank + (cur_k - b_start);
                        let code = if bnd.thick_handle == 0 {
                            bcode_pure
                        } else {
                            let b0_u8 = b0 as u8;
                            let b1_u8 = ((s.b0_b1_code >> 8) & 0xFF) as u8;
                            let arena_len = self.arena.len();
                            if wide_links {
                                let mut e = Elim::<Wide> {
                                    a: self.arena.as_mut_slice(),
                                    base: nodes_words,
                                    top: arena_len,
                                    nodes: 0,
                                    _links: std::marker::PhantomData,
                                };
                                dict.compose_bigram_thick(b0_u8, b1_u8, bnd.thick_handle, &mut e, &mut self.stack)
                            } else {
                                let mut e = Elim::<Narrow> {
                                    a: self.arena.as_mut_slice(),
                                    base: nodes_words,
                                    top: arena_len,
                                    nodes: 0,
                                    _links: std::marker::PhantomData,
                                };
                                dict.compose_bigram_thick(b0_u8, b1_u8, bnd.thick_handle, &mut e, &mut self.stack)
                            }
                        };
                        for _ in 0..step {
                            edges[src as usize] = dst;
                            extra[src as usize] = code;
                            src += 1;
                            dst += 1;
                        }
                        cur_k += step;
                        rem -= step;
                        if cur_k == bnd.end_k {
                            b_start = bnd.end_k;
                            bi += 1;
                        }
                    }
                }
            }
            emit_phi2_single(
                edges,
                extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                lfp,
                byte_before_p,
                out,
            );
        }

        if let Some(t) = t0 {
            let t_s4 = t.elapsed().as_secs_f64() * 1e3;
            let (s1, s1b, s2, s3) = (t_s1.unwrap(), t_s1b.unwrap(), t_s2.unwrap(), t_s3.unwrap());
            eprintln!(
                "PROF(L={}) n={} r0={} r2={} m2={} ns2={} codes={} ovf={} | S1={:.3} S1b_mrg={:.3} S2_ep={:.3} S3_wire={:.3} S4_emit={:.3} TOT={:.3}ms | scratch={}KiB",
                LIM, n, r0, self.p2_strips.len(), m2, ns2, self.lens.len(), overflowed_u32,
                s1, s1b - s1, s2 - s1b, s3 - s2, t_s4 - s3, t_s4, self.scratch_kib()
            );
        }
        true
    }

    pub fn scratch_kib(&self) -> usize {
        (self.run_len_c.capacity() * 4
            + self.run_l_start.capacity() * 4
            + self.f_order_v.capacity() * 4
            + self.p2_strips.capacity() * std::mem::size_of::<P2Strip>()
            + self.v_start.capacity() * 4
            + self.v_end.capacity() * 4
            + self.bigram_lut.capacity() * 2
            + self.bigram_touched.capacity() * 2
            + self.thick_dom.capacity() * std::mem::size_of::<Thick2>()
            + self.thick_im_order.capacity() * 4
            + self.arena.capacity() * 4
            + self.stack.capacity() * 4
            + self.bands.capacity() * std::mem::size_of::<Band2>()
            + self.dict_table.capacity() * 4
            + self.inline16.capacity() * INLINE
            + self.lens.capacity()
            + self.spelling.capacity()
            + self.edges.capacity() * 4
            + self.extra.capacity() * 4)
            / 1024
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_phi2_bidirectional(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    pred_mark: u32,
    lfp: u32,
    pred_lfp: u32,
    byte_before_p: u8,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;

    if (n & 1) == 1 && mark == lfp {
        out[..n].fill(byte_before_p);
        out.truncate(n);
        return;
    }

    let (mut back, mut next_b, mut hi) = if (n & 1) == 1 {
        out[n - 1] = byte_before_p;
        (pred_lfp, lfp, n - 1)
    } else {
        (pred_mark, mark, n)
    };
    let (mut prev_f, mut front) = (pred_mark, mark);
    let mut lo = 0usize;
    let mut met_on_graph = false;
    let mut short_front_done = false;

    for pair in 0..=(edges.len() / 2 + 1) {
        if pair != 0 && front == next_b {
            met_on_graph = true;
            break;
        }
        if front == lfp {
            if lo < n {
                out[lo] = byte_before_p;
                lo += 1;
            }
            short_front_done = true;
            break;
        }
        if lo >= hi {
            break;
        }
        let ef = edges[front as usize];
        let cf = (ef >> bits) as usize;
        let single = front == back;
        let lf = lens[cf];
        if lf != LONG && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&inline[cf]);
            lo += usize::from(lf);
        } else {
            let word = if lf == LONG {
                long_word(&inline[cf], spelling)
            } else {
                &inline[cf][..usize::from(lf)]
            };
            let take = word.len().min(hi - lo);
            out[lo..lo + take].copy_from_slice(&word[..take]);
            lo += take;
        }
        let nf = (ef & mask) ^ prev_f;
        if nf == mark {
            short_front_done = true;
            break;
        }
        if nf == lfp {
            if lo < n {
                out[lo] = byte_before_p;
                lo += 1;
            }
            short_front_done = true;
            break;
        }
        if single {
            met_on_graph = true;
            break;
        }
        let eb = edges[back as usize];
        let cb = (eb >> bits) as usize;
        let lb = lens[cb];
        if nf == back {
            if lb != LONG && hi - lo >= INLINE {
                let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                hi -= usize::from(lb);
            } else {
                let word = if lb == LONG {
                    long_word(&inline[cb], spelling)
                } else {
                    &inline[cb][..usize::from(lb)]
                };
                let take = word.len().min(hi - lo);
                out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
                hi -= take;
            }
            met_on_graph = true;
            break;
        }
        if lo >= hi {
            break;
        }
        if lb != LONG && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= usize::from(lb);
        } else {
            let word = if lb == LONG {
                long_word(&inline[cb], spelling)
            } else {
                &inline[cb][..usize::from(lb)]
            };
            let take = word.len().min(hi - lo);
            out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
            hi -= take;
        }
        let pb = (eb & mask) ^ next_b;
        prev_f = front;
        front = nf;
        next_b = back;
        back = pb;
    }

    if short_front_done {
        out.truncate(lo.min(n));
        repeat(n, out);
    } else if met_on_graph {
        if lo < hi {
            out.copy_within(hi..n, lo);
            out.truncate(lo + (n - hi));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    } else {
        let mut at = 0usize;
        let (mut pf, mut cur) = (pred_mark, mark);
        for _ in 0..=edges.len() {
            if cur == lfp {
                if at < n {
                    out[at] = byte_before_p;
                    at += 1;
                }
                break;
            }
            let e = edges[cur as usize];
            let c = (e >> bits) as usize;
            let l = lens[c];
            if l != LONG && n - at >= INLINE {
                out[at..at + INLINE].copy_from_slice(&inline[c]);
                at += usize::from(l);
            } else {
                let word = if l == LONG {
                    long_word(&inline[c], spelling)
                } else {
                    &inline[c][..usize::from(l)]
                };
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            let nxt = (e & mask) ^ pf;
            pf = cur;
            cur = nxt;
            if cur == mark || at >= n {
                break;
            }
        }
        out.truncate(at.min(n));
        repeat(n, out);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_phi2_single(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    lfp: u32,
    byte_before_p: u8,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let mut cur = mark;
    for _ in 0..=edges.len() {
        if cur == lfp {
            if at < n {
                out[at] = byte_before_p;
                at += 1;
            }
            break;
        }
        let c = extra[cur as usize] as usize;
        cur = edges[cur as usize];
        let l = lens[c];
        if l != LONG && n - at >= INLINE {
            out[at..at + INLINE].copy_from_slice(&inline[c]);
            at += usize::from(l);
        } else {
            let word = if l == LONG {
                long_word(&inline[c], spelling)
            } else {
                &inline[c][..usize::from(l)]
            };
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if cur == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

impl<const LIM: u32> InverseBwt for Phi2XorOpenOf<LIM> {
    fn reads_runs(&self) -> bool {
        true
    }

    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        let recs = r.records();
        if n > 0 && n <= 900_000 && (p as usize) < n {
            let (admit, cap_m, cap_strips) = if recs.len() * 20 < n * 3 {
                (true, u32::MAX, usize::MAX)
            } else if recs.len() * DENSE_DEN < n * DENSE_NUM {
                (true, DENSE_M.min((n as u64 * 3 / 10) as u32), DENSE_STRIPS)
            } else {
                (false, 0, 0)
            };
            if admit && self.try_invert_runs_profiled(recs, n, p, cap_m, cap_strips, out, false) {
                return true;
            }
        }
        self.fallback.invert_runs(r, n, p, out, k)
    }

    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(l, c, p, out);
    }

    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(l, c, p)
    }

    fn wants_checkpoints(&self) -> bool {
        self.fallback.wants_checkpoints()
    }

    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        c: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.fallback.invert_checkpointed(l, c, checkpoints, p, out);
    }
}
