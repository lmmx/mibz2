#![forbid(unsafe_code)]
//! Isolated Native Safe-Rust Confrontation Against Verbatim `mibz2 0.5.8` (`origin/master:src/open.rs`)
//! Tests:
//!   0. `Native-058`       : Verbatim `origin/master` `open.rs` + `endpoint.rs` (`Open<16, true, true>`)
//!   1. `MN-Quotient16`    : `0.5.8` + Epoch-stamped 16 KiB Myhill-Nerode SLP/Band Quotient (Thm 12.2)
//!   2. `Slab256-ZeroGrow` : `0.5.8` + `256+16` Pooled Page Queue eliminating `Vec<Vec<u32>>` blowup (Thm 12.3)
//!   3. `R4@16-Serial`     : Safe-Rust `Rule-R4` elimination alone (serial walk) at `LIMIT = 16` (#675 open question)
//!   4. `R4@16+MN+Slab256` : Combined `R4@16-Serial` + `MN-Quotient16` + `Slab256` (strictly smaller working set!)

use std::fs;
use std::mem::size_of;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

// ============================================================================
// ARM 0: VERBATIM `origin/master:src/open.rs` + `src/open/endpoint.rs` (0.5.8)
// ============================================================================
mod open_058 {
    use super::{INLINE, LONG};
    use std::mem::size_of;

    #[derive(Default, Clone, Copy, Debug)]
    pub struct Counts {
        pub open_nodes: usize,
        pub steps: usize,
        pub rules: usize,
        pub n: usize,
        pub runs: usize,
        pub retained: usize,
        pub thick: usize,
        pub bands: usize,
        pub words: usize,
        pub table_bytes: usize,
    }

    #[derive(Clone, Copy)]
    struct Strip {
        from: u32,
        to: u32,
        width: u32,
        byte: u8,
    }

    #[derive(Clone, Copy)]
    struct Band {
        end: u32,
        delta: u32,
        code: u32,
    }

    #[derive(Default, Clone, Copy)]
    struct Node {
        prev: [u32; 2],
        next: [u32; 2],
        width: u32,
        word: u32,
    }

    #[derive(Clone, Copy)]
    struct Rule {
        left: u32,
        right: u32,
        len: u32,
        first: u32,
    }

    #[derive(Default)]
    struct Endpoint {
        nodes: Vec<Node>,
        rules: Vec<Rule>,
        order: [Vec<u32>; 2],
        terminals: [Vec<u32>; 2],
        bottom: Vec<u32>,
        stack: Vec<u32>,
    }

    impl Endpoint {
        fn ws_bytes(&self) -> usize {
            size_of::<Node>() * self.nodes.capacity()
                + size_of::<Rule>() * self.rules.capacity()
                + 4 * (self.order[0].capacity()
                    + self.order[1].capacity()
                    + self.terminals[0].capacity()
                    + self.terminals[1].capacity()
                    + self.bottom.capacity()
                    + self.stack.capacity())
        }

        fn node(&mut self, width: u32, word: u32) -> u32 {
            let id = self.nodes.len() as u32;
            self.nodes.push(Node {
                width,
                word,
                ..Node::default()
            });
            id
        }

        fn cut(&mut self, id: u32, side: usize) {
            let h = self.nodes[id as usize];
            self.nodes[h.prev[side] as usize].next[side] = h.next[side];
            self.nodes[h.next[side] as usize].prev[side] = h.prev[side];
        }

        fn insert(&mut self, at: u32, id: u32, side: usize) {
            let next = self.nodes[at as usize].next[side];
            self.nodes[id as usize].prev[side] = at;
            self.nodes[id as usize].next[side] = next;
            self.nodes[at as usize].next[side] = id;
            self.nodes[next as usize].prev[side] = id;
        }

        fn len(&self, w: u32) -> u32 {
            if w <= 256 {
                u32::from(w != 0)
            } else {
                self.rules[(w - 257) as usize].len
            }
        }

        fn join(&mut self, left: u32, right: u32) -> u32 {
            if left == 0 {
                return right;
            }
            if right == 0 {
                return left;
            }
            let id = self.rules.len() as u32 + 257;
            self.rules.push(Rule {
                left,
                right,
                len: self.len(left) + self.len(right),
                first: u32::MAX,
            });
            id
        }

        fn gaps(&mut self, side: usize, start: u32, end: u32) {
            if start < end {
                let id = self.node(end - start, 0);
                self.order[side].push(id);
                self.terminals[1 - side].push(id);
            }
        }

        fn lists(&mut self) {
            for side in 0..2 {
                let mut prev = 0;
                for &id in self.terminals[side].iter().chain(&self.order[side]) {
                    self.nodes[prev as usize].next[side] = id;
                    self.nodes[id as usize].prev[side] = prev;
                    prev = id;
                }
                self.nodes[prev as usize].next[side] = 0;
                self.nodes[0].prev[side] = prev;
            }
        }

        fn eliminate(&mut self, mut remaining: u32) -> usize {
            let mut steps = 0;
            while remaining > 0 {
                let [x, y] = self.nodes[0].prev;
                let a = self.nodes[x as usize];
                let b = self.nodes[y as usize];
                let w = a.width.min(b.width);
                assert!(w > 0 && w <= remaining);
                remaining -= w;
                steps += 1;
                if x == y {
                    self.cut(x, 0);
                    self.cut(x, 1);
                    continue;
                }
                let word = self.join(a.word, b.word);
                if a.width <= b.width {
                    self.cut(x, 0);
                    self.insert(y, x, 0);
                    self.nodes[x as usize].word = word;
                    if a.width == b.width {
                        self.cut(y, 0);
                        self.cut(y, 1);
                    } else {
                        self.nodes[y as usize].width -= w;
                    }
                } else {
                    self.cut(y, 1);
                    self.insert(x, y, 1);
                    self.nodes[y as usize].word = word;
                    self.nodes[x as usize].width -= w;
                }
            }
            steps
        }

        fn append(&mut self, word: u32, dest: &mut Vec<u8>) {
            self.stack.clear();
            self.stack.push(word);
            while let Some(id) = self.stack.pop() {
                if id <= 256 {
                    dest.push((id - 1) as u8);
                } else {
                    let rule = &mut self.rules[(id - 257) as usize];
                    if rule.first == u32::MAX {
                        rule.first = dest.len() as u32;
                        self.stack.push(rule.right);
                        self.stack.push(rule.left);
                    } else {
                        let start = rule.first as usize;
                        dest.extend_from_within(start..start + rule.len as usize);
                    }
                }
            }
        }
    }

    #[derive(Default)]
    pub struct Kernel {
        endpoint: Endpoint,
        queues: Vec<Vec<u32>>,
        strips: Vec<Strip>,
        image_order: Vec<u32>,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        offsets: Vec<u32>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
    }

    #[derive(Clone, Copy)]
    struct First {
        all: [u32; 256],
        thin: [u32; 256],
        thick: [u32; 256],
        m: u32,
        mark: u32,
        runs: usize,
    }

    impl Default for First {
        fn default() -> Self {
            First {
                all: [0; 256],
                thin: [0; 256],
                thick: [0; 256],
                m: 0,
                mark: 0,
                runs: 0,
            }
        }
    }

    #[inline(always)]
    fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
        let mut pos = 0;
        let mut i = 0;
        while i < records.len() {
            let rec = records[i];
            let c = (rec >> 8) as u8;
            let mut width = u32::from(rec & 255);
            i += 1;
            while i < records.len() && (records[i] >> 8) as u8 == c {
                width += u32::from(records[i] & 255);
                i += 1;
            }
            visit(c, pos, width);
            pos += width;
        }
    }

    #[inline(always)]
    fn split(
        c: u8,
        pos: u32,
        width: u32,
        p: u32,
        limit: u32,
        mut visit: impl FnMut(u8, u32, u32, bool),
    ) {
        let end = pos + width;
        if width > limit && (pos..end).contains(&p) {
            if pos < p {
                visit(c, pos, p - pos, false);
            }
            visit(c, p, 1, true);
            if p + 1 < end {
                visit(c, p + 1, end - p - 1, false);
            }
        } else {
            visit(c, pos, width, width <= limit);
        }
    }

    #[inline(always)]
    fn push_rows(queue: &mut Vec<u32>, len: u32, m: u32, w: u32) {
        const BLOCK: usize = 16;
        let at = len as usize;
        let need = at + BLOCK.max(w as usize);
        if queue.len() < need {
            grow(queue, need);
        }
        if w as usize <= BLOCK {
            let iota: [u32; BLOCK] = std::array::from_fn(|i| m.wrapping_add(i as u32));
            queue[at..at + BLOCK].copy_from_slice(&iota);
        } else {
            for (i, q) in queue[at..at + w as usize].iter_mut().enumerate() {
                *q = m + i as u32;
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(queue: &mut Vec<u32>, need: usize) {
        queue.resize(need.max(2 * queue.len()).max(1024), 0);
    }

    fn starts(counts: [u32; 256]) -> [u32; 256] {
        let mut total = 0;
        counts.map(|w| {
            let start = total;
            total += w;
            start
        })
    }

    struct Pass<'a> {
        queues: &'a mut [Vec<u32>; 256],
        strips: &'a mut Vec<Strip>,
        f: First,
    }

    impl Pass<'_> {
        #[inline(always)]
        fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
            self.f.runs += 1;
            if width <= limit {
                self.piece(c, pos, width, true, p);
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.piece(c, pos, width, false, p);
            } else {
                self.take_marked(c, pos, width, p, limit);
            }
        }

        #[cold]
        #[inline(never)]
        fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
            split(c, pos, width, p, limit, |c, x, w, keep| {
                self.piece(c, x, w, keep, p)
            });
        }

        #[inline(always)]
        fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
            let a = usize::from(c);
            let f = &mut self.f;
            if keep {
                if p.wrapping_sub(x) < w {
                    f.mark = f.m + (p - x);
                }
                push_rows(&mut self.queues[a], f.thin[a], f.m, w);
                f.thin[a] += w;
                f.m += w;
            } else {
                self.strips.push(Strip {
                    from: x,
                    to: f.all[a],
                    width: w,
                    byte: c,
                });
                f.thick[a] += 1;
            }
            f.all[a] += w;
        }
    }

    impl Kernel {
        pub fn queue_bytes(&self) -> usize {
            size_of::<Vec<u32>>() * self.queues.capacity()
                + 4 * self.queues.iter().map(Vec::capacity).sum::<usize>()
        }

        pub fn dict_bytes(&self) -> usize {
            size_of::<[u8; INLINE]>() * self.inline16.len() + self.lens.len()
        }

        pub fn total_ws_bytes(&self) -> usize {
            self.endpoint.ws_bytes()
                + self.queue_bytes()
                + size_of::<Strip>() * self.strips.capacity()
                + 4 * self.image_order.capacity()
                + size_of::<Band>() * self.bands.capacity()
                + self.spelling.capacity()
                + 4 * self.offsets.capacity()
                + 4 * self.edges.capacity()
                + 4 * self.extra.capacity()
                + size_of::<[u8; INLINE]>() * self.inline16.capacity()
                + self.lens.capacity()
        }

        fn induced(&mut self, n: u32, thin: &[u32; 256], stats: &mut Counts) -> [usize; 256] {
            let e = &mut self.endpoint;
            e.nodes.clear();
            e.nodes.push(Node::default());
            e.rules.clear();
            for order in e.order.iter_mut().chain(&mut e.terminals) {
                order.clear();
            }
            for s in &self.strips {
                e.node(s.width, u32::from(s.byte) + 1);
            }
            let mut x = 0;
            for (i, s) in self.strips.iter().enumerate() {
                e.gaps(0, x, s.from);
                e.order[0].push(i as u32 + 1);
                x = s.from + s.width;
            }
            e.gaps(0, x, n);
            x = 0;
            for &i in &self.image_order {
                let s = self.strips[i as usize];
                e.gaps(1, x, s.to);
                e.order[1].push(i + 1);
                x = s.to + s.width;
            }
            e.gaps(1, x, n);
            stats.open_nodes = e.nodes.len() - 1;
            e.lists();
            stats.steps = e.eliminate(n);
            stats.rules = e.rules.len();

            e.bottom.resize(e.nodes.len(), 0);
            let mut id = e.nodes[0].next[1];
            x = 0;
            while id != 0 {
                e.bottom[id as usize] = x;
                let node = e.nodes[id as usize];
                x += node.width;
                id = node.next[1];
            }
            let mut cursors = [0; 256];
            id = e.nodes[0].next[0];
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                while k < end {
                    let node = e.nodes[id as usize];
                    let width = (end - k).min(node.width - in_node);
                    let dst = e.bottom[id as usize] + in_node;
                    let code = if node.word == 0 {
                        a as u32
                    } else {
                        stats.words += 1;
                        e.append(node.word, &mut self.spelling);
                        self.spelling.push(a as u8);
                        self.offsets.push(self.spelling.len() as u32);
                        254 + self.offsets.len() as u32
                    };
                    self.bands.push(Band {
                        end: k + width,
                        delta: dst.wrapping_sub(k),
                        code,
                    });
                    in_node += width;
                    k += width;
                    if in_node == node.width {
                        id = node.next[0];
                        in_node = 0;
                    }
                }
            }
            cursors
        }

        pub fn construct(&mut self, records: &[u16], n: u32, p: u32, limit: u32) -> (Counts, u32, u32) {
            self.strips.clear();
            self.queues.resize_with(256, Vec::new);
            let mut pass = Pass {
                queues: (&mut self.queues[..]).try_into().expect("256 queues"),
                strips: &mut self.strips,
                f: First::default(),
            };
            maximal(records, |c, pos, width| pass.take(c, pos, width, p, limit));
            let First { all, thin, thick, m, mark, runs } = pass.f;
            let mut stats = Counts {
                n: n as usize,
                runs,
                retained: m as usize,
                thick: self.strips.len(),
                ..Counts::default()
            };
            let first = starts(all);
            let mut write = starts(thick);
            self.image_order.resize(self.strips.len(), 0);
            for (i, s) in self.strips.iter_mut().enumerate() {
                let a = usize::from(s.byte);
                s.to += first[a];
                self.image_order[write[a] as usize] = i as u32;
                write[a] += 1;
            }
            self.bands.clear();
            self.spelling.clear();
            self.offsets.clear();
            self.offsets.push(0);
            let cursors = self.induced(n, &thin, &mut stats);
            stats.bands = self.bands.len();
            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
            self.edges.resize(m as usize, 0);
            self.extra.clear();
            if wide {
                self.extra.resize(m as usize, 0);
            }
            stats.table_bytes = 4 * (self.edges.len() + self.extra.len());
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut from = 0;
                for b in &self.bands[cursors[a]..end] {
                    let span = (b.end - k) as usize;
                    let dst = k.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(b.code);
                    }
                    let src = &self.queues[a][from..from + span];
                    for (target, &j) in self.edges[dst..dst + span].iter_mut().zip(src) {
                        *target = pack | j;
                    }
                    from += span;
                    k = b.end;
                }
            }
            (stats, mark, bits)
        }

        pub fn walk(&mut self, n: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
            inline_words(&self.spelling, &self.offsets, &mut self.inline16, &mut self.lens);
            emit(
                &self.edges,
                &self.extra,
                &self.spelling,
                &self.offsets,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
            );
        }

        pub fn run(&mut self, records: &[u16], n: usize, p: u32, limit: u32, out: &mut Vec<u8>) -> Counts {
            let (stats, mark, bits) = self.construct(records, n as u32, p, limit);
            self.walk(n, mark, bits, out);
            stats
        }
    }

    pub fn inline_words(
        spelling: &[u8],
        offsets: &[u32],
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
    ) {
        if inline16.len() < 256 {
            inline16.clear();
            inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            lens.clear();
            lens.resize(256, 1);
        }
        inline16.truncate(256);
        lens.truncate(256);
        for w in offsets.windows(2) {
            let word = &spelling[w[0] as usize..w[1] as usize];
            let mut v = [0; INLINE];
            let head = word.len().min(INLINE);
            v[..head].copy_from_slice(&word[..head]);
            inline16.push(v);
            lens.push(if word.len() <= INLINE {
                word.len() as u8
            } else {
                LONG
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit(
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        offsets: &[u32],
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        out.resize(n + INLINE, 0);
        let mut at = 0;
        let mask = (1 << bits) - 1;
        let mut j = mark;
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = if extra.is_empty() {
                edge >> bits
            } else {
                extra[j as usize]
            } as usize;
            j = edge & mask;
            let len = lens[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                let w = code - 256;
                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if j == mark || at >= n {
                break;
            }
        }
        out.truncate(at.min(n));
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }
}

// ============================================================================
// POOLED `Slab256+16` QUEUE + EPOCH-STAMPED MYHILL-NERODE QUOTIENT + R4@16
// ============================================================================
mod open_variants {
    use super::open_058::{emit, inline_words};
    use super::{INLINE, LONG};
    use std::mem::size_of;

    /// `Slab256+16`: Pooled 256-entry (`+ 16` SIMD guard = `272` u32s) page arena!
    /// - Fast path (`99.3%` of runs): 1 branch (`in_page + w <= 256`) + 16-wide SIMD iota store!
    /// - Zero scalar split loops EVER!
    /// - Eliminates `Vec<Vec<u32>>`'s 256 separate heap buffers & multi-block l1-linf blowup!
    const PAGE_CAP: usize = 256;
    const PAGE_STRIDE: usize = 272; // 256 + 16 SIMD overrun guard inside each page!
    const NO_PAGE: u16 = u16::MAX;

    pub struct SlabQueue {
        slab: Vec<u32>,
        next_page: Vec<u16>,
        page_len: Vec<u16>,
        head: [u16; 256],
        tail: [u16; 256],
        in_page: [u16; 256],
        used_pages: usize,
    }

    impl Default for SlabQueue {
        fn default() -> Self {
            Self {
                slab: Vec::new(),
                next_page: Vec::new(),
                page_len: Vec::new(),
                head: [NO_PAGE; 256],
                tail: [NO_PAGE; 256],
                in_page: [PAGE_CAP as u16 + 1; 256],
                used_pages: 0,
            }
        }
    }

    impl SlabQueue {
        pub fn ws_bytes(&self) -> usize {
            4 * self.slab.capacity()
                + 2 * self.next_page.capacity()
                + 2 * self.page_len.capacity()
                + 256 * 6
        }

        pub fn reset(&mut self) {
            self.head = [NO_PAGE; 256];
            self.tail = [NO_PAGE; 256];
            self.in_page = [PAGE_CAP as u16 + 1; 256];
            self.used_pages = 0;
        }

        #[inline(always)]
        pub fn push_rows(&mut self, a: usize, m: u32, w: u32) {
            let ip = self.in_page[a] as usize;
            if ip + (w as usize) <= PAGE_CAP {
                let base = (self.tail[a] as usize) * PAGE_STRIDE + ip;
                let iota: [u32; 16] = std::array::from_fn(|i| m.wrapping_add(i as u32));
                self.slab[base..base + 16].copy_from_slice(&iota);
                self.in_page[a] = (ip + w as usize) as u16;
            } else {
                self.push_new_page(a, m, w);
            }
        }

        #[cold]
        #[inline(never)]
        fn push_new_page(&mut self, a: usize, m: u32, w: u32) {
            let prev = self.tail[a];
            if prev != NO_PAGE {
                self.page_len[prev as usize] = self.in_page[a];
            }
            let pid = self.used_pages;
            self.used_pages += 1;
            if pid >= self.next_page.len() {
                let new_cap = ((pid + 256) & !255).max(256);
                self.slab.resize(new_cap * PAGE_STRIDE, 0);
                self.next_page.resize(new_cap, NO_PAGE);
                self.page_len.resize(new_cap, 0);
            } else {
                self.next_page[pid] = NO_PAGE;
            }
            let pid16 = pid as u16;
            if prev == NO_PAGE {
                self.head[a] = pid16;
            } else {
                self.next_page[prev as usize] = pid16;
            }
            self.tail[a] = pid16;
            let base = pid * PAGE_STRIDE;
            if w <= 16 {
                let iota: [u32; 16] = std::array::from_fn(|i| m.wrapping_add(i as u32));
                self.slab[base..base + 16].copy_from_slice(&iota);
            } else {
                for (i, slot) in self.slab[base..base + w as usize].iter_mut().enumerate() {
                    *slot = m + i as u32;
                }
            }
            self.in_page[a] = w as u16;
        }

        pub fn finalize_tails(&mut self) {
            for a in 0..256 {
                let t = self.tail[a];
                if t != NO_PAGE {
                    self.page_len[t as usize] = self.in_page[a];
                }
            }
        }
    }

    /// Epoch-stamped Zero-Memset L1D Myhill-Nerode Memo Tables (`16 KiB` total!)
    const JOIN_MASK: usize = 1023;
    const BAND_MASK: usize = 1023;

    #[derive(Clone, Copy, Default)]
    struct JoinEntry {
        epoch: u16,
        out_rule: u32,
        left: u32,
        right: u32,
    }

    pub struct MnCache {
        epoch: u16,
        // Packed band entry: `(epoch_u16 << 48) | (word_id_24 << 24) | (byte_u8 << 16) | code_u16`
        band_memo: Vec<u64>,
        join_memo: Vec<JoinEntry>,
    }

    impl Default for MnCache {
        fn default() -> Self {
            Self {
                epoch: 0,
                band_memo: Vec::new(),
                join_memo: Vec::new(),
            }
        }
    }

    impl MnCache {
        pub fn ws_bytes(&self) -> usize {
            8 * self.band_memo.capacity() + size_of::<JoinEntry>() * self.join_memo.capacity()
        }

        pub fn next_epoch(&mut self) {
            if self.band_memo.is_empty() {
                self.band_memo.resize(BAND_MASK + 1, 0);
                self.join_memo.resize(JOIN_MASK + 1, JoinEntry::default());
            }
            self.epoch = self.epoch.wrapping_add(1);
            if self.epoch == 0 {
                self.band_memo.fill(0);
                self.join_memo.fill(JoinEntry::default());
                self.epoch = 1;
            }
        }
    }

    #[derive(Clone, Copy)]
    struct Strip {
        from: u32,
        to: u32,
        width: u32,
        byte: u8,
    }

    #[derive(Clone, Copy)]
    struct Band {
        end: u32,
        delta: u32,
        code: u32,
    }

    // Compact `Node`: `u16` links when `open_nodes < 65535`? Keep `u32` or `R4Top`/`R4Bot`!
    // Let's implement BOTH:
    //  - `EndpointMn`: 0.5.8 Endpoint + optional `MN` hash-consing
    //  - `R4Engine`: Safe-Rust `Rule-R4` (`16 B` `R4Top` + `8 B` `R4Bot`, zero `order/terminals/bottom/next`)!

    #[derive(Default, Clone, Copy)]
    struct Node {
        prev: [u32; 2],
        next: [u32; 2],
        width: u32,
        word: u32,
    }

    #[derive(Clone, Copy)]
    struct Rule {
        left: u32,
        right: u32,
        len: u32,
        first: u32,
    }

    #[derive(Default)]
    struct EndpointMn {
        nodes: Vec<Node>,
        rules: Vec<Rule>,
        order: [Vec<u32>; 2],
        terminals: [Vec<u32>; 2],
        bottom: Vec<u32>,
        stack: Vec<u32>,
    }

    impl EndpointMn {
        fn ws_bytes(&self) -> usize {
            size_of::<Node>() * self.nodes.capacity()
                + size_of::<Rule>() * self.rules.capacity()
                + 4 * (self.order[0].capacity()
                    + self.order[1].capacity()
                    + self.terminals[0].capacity()
                    + self.terminals[1].capacity()
                    + self.bottom.capacity()
                    + self.stack.capacity())
        }

        fn node(&mut self, width: u32, word: u32) -> u32 {
            let id = self.nodes.len() as u32;
            self.nodes.push(Node { width, word, ..Node::default() });
            id
        }

        fn cut(&mut self, id: u32, side: usize) {
            let h = self.nodes[id as usize];
            self.nodes[h.prev[side] as usize].next[side] = h.next[side];
            self.nodes[h.next[side] as usize].prev[side] = h.prev[side];
        }

        fn insert(&mut self, at: u32, id: u32, side: usize) {
            let next = self.nodes[at as usize].next[side];
            self.nodes[id as usize].prev[side] = at;
            self.nodes[id as usize].next[side] = next;
            self.nodes[at as usize].next[side] = id;
            self.nodes[next as usize].prev[side] = id;
        }

        fn len(&self, w: u32) -> u32 {
            if w <= 256 {
                u32::from(w != 0)
            } else {
                self.rules[(w - 257) as usize].len
            }
        }

        #[inline(always)]
        fn join<const USE_MN: bool>(&mut self, left: u32, right: u32, mn: &mut MnCache) -> u32 {
            if left == 0 {
                return right;
            }
            if right == 0 {
                return left;
            }
            if USE_MN {
                let slot = ((left.wrapping_mul(0x9E37_79B9)) ^ (right.wrapping_mul(0x85EB_CA6B))) as usize & JOIN_MASK;
                let e = mn.join_memo[slot];
                if e.epoch == mn.epoch && e.left == left && e.right == right {
                    return e.out_rule;
                }
                let id = self.rules.len() as u32 + 257;
                self.rules.push(Rule {
                    left,
                    right,
                    len: self.len(left) + self.len(right),
                    first: u32::MAX,
                });
                mn.join_memo[slot] = JoinEntry {
                    epoch: mn.epoch,
                    out_rule: id,
                    left,
                    right,
                };
                id
            } else {
                let id = self.rules.len() as u32 + 257;
                self.rules.push(Rule {
                    left,
                    right,
                    len: self.len(left) + self.len(right),
                    first: u32::MAX,
                });
                id
            }
        }

        fn gaps(&mut self, side: usize, start: u32, end: u32) {
            if start < end {
                let id = self.node(end - start, 0);
                self.order[side].push(id);
                self.terminals[1 - side].push(id);
            }
        }

        fn lists(&mut self) {
            for side in 0..2 {
                let mut prev = 0;
                for &id in self.terminals[side].iter().chain(&self.order[side]) {
                    self.nodes[prev as usize].next[side] = id;
                    self.nodes[id as usize].prev[side] = prev;
                    prev = id;
                }
                self.nodes[prev as usize].next[side] = 0;
                self.nodes[0].prev[side] = prev;
            }
        }

        fn eliminate<const USE_MN: bool>(&mut self, mut remaining: u32, mn: &mut MnCache) {
            while remaining > 0 {
                let [x, y] = self.nodes[0].prev;
                let a = self.nodes[x as usize];
                let b = self.nodes[y as usize];
                let w = a.width.min(b.width);
                remaining -= w;
                if x == y {
                    self.cut(x, 0);
                    self.cut(x, 1);
                    continue;
                }
                let word = self.join::<USE_MN>(a.word, b.word, mn);
                if a.width <= b.width {
                    self.cut(x, 0);
                    self.insert(y, x, 0);
                    self.nodes[x as usize].word = word;
                    if a.width == b.width {
                        self.cut(y, 0);
                        self.cut(y, 1);
                    } else {
                        self.nodes[y as usize].width -= w;
                    }
                } else {
                    self.cut(y, 1);
                    self.insert(x, y, 1);
                    self.nodes[y as usize].word = word;
                    self.nodes[x as usize].width -= w;
                }
            }
        }

        fn append(&mut self, word: u32, dest: &mut Vec<u8>) {
            self.stack.clear();
            self.stack.push(word);
            while let Some(id) = self.stack.pop() {
                if id <= 256 {
                    dest.push((id - 1) as u8);
                } else {
                    let rule = &mut self.rules[(id - 257) as usize];
                    if rule.first == u32::MAX {
                        rule.first = dest.len() as u32;
                        self.stack.push(rule.right);
                        self.stack.push(rule.left);
                    } else {
                        let start = rule.first as usize;
                        dest.extend_from_within(start..start + rule.len as usize);
                    }
                }
            }
        }
    }

    // Safe-Rust `Rule-R4` Engine (`16 B` `R4Top` + `8 B` `R4Bot`, prev-only tape!)
    #[derive(Default, Clone, Copy)]
    struct R4Top {
        prev: u32,
        width: u32,
        word: u32,
        other: u32, // bot index while active; reused as `bottom` coordinate after!
    }

    #[derive(Default, Clone, Copy)]
    struct R4Bot {
        prev: u32,
        top: u32,
    }

    #[derive(Default)]
    struct R4Engine {
        tops: Vec<R4Top>,
        bots: Vec<R4Bot>,
        strip_top: Vec<u32>,
        rules: Vec<Rule>,
        stack: Vec<u32>,
        surv_top: Vec<u32>,
    }

    impl R4Engine {
        fn ws_bytes(&self) -> usize {
            size_of::<R4Top>() * self.tops.capacity()
                + size_of::<R4Bot>() * self.bots.capacity()
                + 4 * self.strip_top.capacity()
                + size_of::<Rule>() * self.rules.capacity()
                + 4 * self.stack.capacity()
                + 4 * self.surv_top.capacity()
        }

        fn len(&self, w: u32) -> u32 {
            if w <= 256 {
                u32::from(w != 0)
            } else {
                self.rules[(w - 257) as usize].len
            }
        }

        #[inline(always)]
        fn join<const USE_MN: bool>(&mut self, left: u32, right: u32, mn: &mut MnCache) -> u32 {
            if left == 0 {
                return right;
            }
            if right == 0 {
                return left;
            }
            if USE_MN {
                let slot = ((left.wrapping_mul(0x9E37_79B9)) ^ (right.wrapping_mul(0x85EB_CA6B))) as usize & JOIN_MASK;
                let e = mn.join_memo[slot];
                if e.epoch == mn.epoch && e.left == left && e.right == right {
                    return e.out_rule;
                }
                let id = self.rules.len() as u32 + 257;
                self.rules.push(Rule {
                    left,
                    right,
                    len: self.len(left) + self.len(right),
                    first: u32::MAX,
                });
                mn.join_memo[slot] = JoinEntry {
                    epoch: mn.epoch,
                    out_rule: id,
                    left,
                    right,
                };
                id
            } else {
                let id = self.rules.len() as u32 + 257;
                self.rules.push(Rule {
                    left,
                    right,
                    len: self.len(left) + self.len(right),
                    first: u32::MAX,
                });
                id
            }
        }

        fn append(&mut self, word: u32, dest: &mut Vec<u8>) {
            self.stack.clear();
            self.stack.push(word);
            while let Some(id) = self.stack.pop() {
                if id <= 256 {
                    dest.push((id - 1) as u8);
                } else {
                    let rule = &mut self.rules[(id - 257) as usize];
                    if rule.first == u32::MAX {
                        rule.first = dest.len() as u32;
                        self.stack.push(rule.right);
                        self.stack.push(rule.left);
                    } else {
                        let start = rule.first as usize;
                        dest.extend_from_within(start..start + rule.len as usize);
                    }
                }
            }
        }
    }

    #[derive(Default)]
    pub struct VariantKernel<const USE_SLAB: bool, const USE_MN: bool, const USE_R4: bool> {
        ep: EndpointMn,
        r4: R4Engine,
        mn: MnCache,
        slab_q: SlabQueue,
        vec_q: Vec<Vec<u32>>,
        strips: Vec<Strip>,
        image_order: Vec<u32>,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        offsets: Vec<u32>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
    }

    #[inline(always)]
    fn push_rows_vec(queue: &mut Vec<u32>, len: u32, m: u32, w: u32) {
        const BLOCK: usize = 16;
        let at = len as usize;
        let need = at + BLOCK.max(w as usize);
        if queue.len() < need {
            grow_vec(queue, need);
        }
        let iota: [u32; BLOCK] = std::array::from_fn(|i| m.wrapping_add(i as u32));
        queue[at..at + BLOCK].copy_from_slice(&iota);
    }

    #[cold]
    #[inline(never)]
    fn grow_vec(queue: &mut Vec<u32>, need: usize) {
        queue.resize(need.max(2 * queue.len()).max(1024), 0);
    }

    fn starts(counts: [u32; 256]) -> [u32; 256] {
        let mut total = 0;
        counts.map(|w| {
            let start = total;
            total += w;
            start
        })
    }

    impl<const USE_SLAB: bool, const USE_MN: bool, const USE_R4: bool>
        VariantKernel<USE_SLAB, USE_MN, USE_R4>
    {
        pub fn queue_bytes(&self) -> usize {
            if USE_SLAB {
                self.slab_q.ws_bytes()
            } else {
                size_of::<Vec<u32>>() * self.vec_q.capacity()
                    + 4 * self.vec_q.iter().map(Vec::capacity).sum::<usize>()
            }
        }

        pub fn dict_bytes(&self) -> usize {
            size_of::<[u8; INLINE]>() * self.inline16.len() + self.lens.len()
        }

        pub fn total_ws_bytes(&self) -> usize {
            let elim_ws = if USE_R4 {
                self.r4.ws_bytes()
            } else {
                self.ep.ws_bytes()
            };
            let mn_ws = if USE_MN { self.mn.ws_bytes() } else { 0 };
            elim_ws
                + mn_ws
                + self.queue_bytes()
                + size_of::<Strip>() * self.strips.capacity()
                + 4 * self.image_order.capacity()
                + size_of::<Band>() * self.bands.capacity()
                + self.spelling.capacity()
                + 4 * self.offsets.capacity()
                + 4 * self.edges.capacity()
                + 4 * self.extra.capacity()
                + size_of::<[u8; INLINE]>() * self.inline16.capacity()
                + self.lens.capacity()
        }

        #[inline(always)]
        fn emit_band_code_ep(&mut self, w_id: u32, a: u8, words: &mut usize) -> u32 {
            if w_id == 0 {
                return u32::from(a);
            }
            if USE_MN && w_id < (1 << 24) {
                let slot = ((w_id.wrapping_mul(0x9E37_79B9)) ^ (u32::from(a).wrapping_mul(0x85EB_CA6B))) as usize & BAND_MASK;
                let tag = ((self.mn.epoch as u64) << 48) | ((w_id as u64) << 24) | ((a as u64) << 16);
                let cur = self.mn.band_memo[slot];
                if (cur & 0xFFFF_FFFF_FFFF_0000) == tag {
                    return (cur & 0xFFFF) as u32;
                }
                *words += 1;
                self.ep.append(w_id, &mut self.spelling);
                self.spelling.push(a);
                self.offsets.push(self.spelling.len() as u32);
                let code = 254 + self.offsets.len() as u32;
                if code <= u16::MAX as u32 {
                    self.mn.band_memo[slot] = tag | (code as u64);
                }
                code
            } else {
                *words += 1;
                self.ep.append(w_id, &mut self.spelling);
                self.spelling.push(a);
                self.offsets.push(self.spelling.len() as u32);
                254 + self.offsets.len() as u32
            }
        }

        #[inline(always)]
        fn emit_band_code_r4(&mut self, w_id: u32, a: u8, words: &mut usize) -> u32 {
            if w_id == 0 {
                return u32::from(a);
            }
            if USE_MN && w_id < (1 << 24) {
                let slot = ((w_id.wrapping_mul(0x9E37_79B9)) ^ (u32::from(a).wrapping_mul(0x85EB_CA6B))) as usize & BAND_MASK;
                let tag = ((self.mn.epoch as u64) << 48) | ((w_id as u64) << 24) | ((a as u64) << 16);
                let cur = self.mn.band_memo[slot];
                if (cur & 0xFFFF_FFFF_FFFF_0000) == tag {
                    return (cur & 0xFFFF) as u32;
                }
                *words += 1;
                self.r4.append(w_id, &mut self.spelling);
                self.spelling.push(a);
                self.offsets.push(self.spelling.len() as u32);
                let code = 254 + self.offsets.len() as u32;
                if code <= u16::MAX as u32 {
                    self.mn.band_memo[slot] = tag | (code as u64);
                }
                code
            } else {
                *words += 1;
                self.r4.append(w_id, &mut self.spelling);
                self.spelling.push(a);
                self.offsets.push(self.spelling.len() as u32);
                254 + self.offsets.len() as u32
            }
        }

        fn induced_ep(&mut self, n: u32, thin: &[u32; 256], words: &mut usize) -> [usize; 256] {
            self.ep.nodes.clear();
            self.ep.nodes.push(Node::default());
            self.ep.rules.clear();
            for order in self.ep.order.iter_mut().chain(&mut self.ep.terminals) {
                order.clear();
            }
            for s in &self.strips {
                self.ep.node(s.width, u32::from(s.byte) + 1);
            }
            let mut x = 0;
            for (i, s) in self.strips.iter().enumerate() {
                self.ep.gaps(0, x, s.from);
                self.ep.order[0].push(i as u32 + 1);
                x = s.from + s.width;
            }
            self.ep.gaps(0, x, n);
            x = 0;
            for &i in &self.image_order {
                let s = self.strips[i as usize];
                self.ep.gaps(1, x, s.to);
                self.ep.order[1].push(i + 1);
                x = s.to + s.width;
            }
            self.ep.gaps(1, x, n);
            self.ep.lists();
            self.ep.eliminate::<USE_MN>(n, &mut self.mn);

            self.ep.bottom.resize(self.ep.nodes.len(), 0);
            let mut id = self.ep.nodes[0].next[1];
            x = 0;
            while id != 0 {
                self.ep.bottom[id as usize] = x;
                let node = self.ep.nodes[id as usize];
                x += node.width;
                id = node.next[1];
            }
            let mut cursors = [0; 256];
            id = self.ep.nodes[0].next[0];
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                while k < end {
                    let node = self.ep.nodes[id as usize];
                    let width = (end - k).min(node.width - in_node);
                    let dst = self.ep.bottom[id as usize] + in_node;
                    let code = self.emit_band_code_ep(node.word, a as u8, words);
                    self.bands.push(Band {
                        end: k + width,
                        delta: dst.wrapping_sub(k),
                        code,
                    });
                    in_node += width;
                    k += width;
                    if in_node == node.width {
                        id = node.next[0];
                        in_node = 0;
                    }
                }
            }
            cursors
        }

        fn induced_r4(&mut self, n: u32, m: u32, thin: &[u32; 256], words: &mut usize) -> [usize; 256] {
            // Safe-Rust `Rule-R4` (`16 B` R4Top + `8 B` R4Bot, prev-only tape!)
            // Tape order:
            //  - `tops`: `0` sentinel, then `P_0` (codomain gaps = `terminals[0]`), then `A_0` (domain gaps + strips)
            //  - `bots`: `0` sentinel, then `P_1` (domain gaps = `terminals[1]`), then `A_1` (codomain gaps + strips)
            let h = self.strips.len();
            self.r4.tops.clear();
            self.r4.bots.clear();
            self.r4.rules.clear();
            self.r4.strip_top.resize(h, 0);

            self.r4.tops.push(R4Top::default());
            self.r4.bots.push(R4Bot::default());

            // 1. Create domain gaps (`in P_1` on bot, and we'll link them in `A_0` on top!)
            // Wait: to lay out `P_0` then `A_0` sequentially in `tops`, and `P_1` then `A_1` in `bots`:
            // First scan `image_order` to create codomain gaps (`P_0` in `tops`)!
            let mut prev_t = 0u32;
            let mut x = 0u32;
            for &si in &self.image_order {
                let s = self.strips[si as usize];
                if x < s.to {
                    let tid = self.r4.tops.len() as u32;
                    self.r4.tops.push(R4Top {
                        prev: prev_t,
                        width: s.to - x,
                        word: 0,
                        other: 0,
                    });
                    prev_t = tid;
                }
                x = s.to + s.width;
            }
            if x < n {
                let tid = self.r4.tops.len() as u32;
                self.r4.tops.push(R4Top {
                    prev: prev_t,
                    width: n - x,
                    word: 0,
                    other: 0,
                });
                prev_t = tid;
            }
            let p0_count = self.r4.tops.len() - 1;

            // Next scan domain `strips` to create `A_0` in `tops` AND `P_1` (domain gaps) in `bots`!
            let mut prev_b = 0u32;
            x = 0;
            for (i, s) in self.strips.iter().enumerate() {
                if x < s.from {
                    let tid = self.r4.tops.len() as u32;
                    let bid = self.r4.bots.len() as u32;
                    self.r4.tops.push(R4Top {
                        prev: prev_t,
                        width: s.from - x,
                        word: 0,
                        other: bid,
                    });
                    prev_t = tid;
                    self.r4.bots.push(R4Bot { prev: prev_b, top: tid });
                    prev_b = bid;
                }
                let tid = self.r4.tops.len() as u32;
                self.r4.tops.push(R4Top {
                    prev: prev_t,
                    width: s.width,
                    word: u32::from(s.byte) + 1,
                    other: 0, // filled in pass 3
                });
                self.r4.strip_top[i] = tid;
                prev_t = tid;
                x = s.from + s.width;
            }
            if x < n {
                let tid = self.r4.tops.len() as u32;
                let bid = self.r4.bots.len() as u32;
                self.r4.tops.push(R4Top {
                    prev: prev_t,
                    width: n - x,
                    word: 0,
                    other: bid,
                });
                prev_t = tid;
                self.r4.bots.push(R4Bot { prev: prev_b, top: tid });
                prev_b = bid;
            }

            // Finally scan `image_order` again to append `A_1` in `bots` (linking to `P_0` codomain gaps and `strip_top`!)
            let mut p0_idx = 1usize;
            x = 0;
            for &si in &self.image_order {
                let s = self.strips[si as usize];
                if x < s.to {
                    let tid = p0_idx as u32;
                    p0_idx += 1;
                    let bid = self.r4.bots.len() as u32;
                    self.r4.tops[tid as usize].other = bid;
                    self.r4.bots.push(R4Bot { prev: prev_b, top: tid });
                    prev_b = bid;
                }
                let tid = self.r4.strip_top[si as usize];
                let bid = self.r4.bots.len() as u32;
                self.r4.tops[tid as usize].other = bid;
                self.r4.bots.push(R4Bot { prev: prev_b, top: tid });
                prev_b = bid;
                x = s.to + s.width;
            }
            if x < n {
                let tid = p0_idx as u32;
                p0_idx += 1;
                let bid = self.r4.bots.len() as u32;
                self.r4.tops[tid as usize].other = bid;
                self.r4.bots.push(R4Bot { prev: prev_b, top: tid });
                prev_b = bid;
            }
            debug_assert_eq!(p0_idx - 1, p0_count);

            // Asymmetric `Rule R4` backward sweep (`prev`-only, zero `next` links!):
            let mut sa = prev_t;
            let mut sb = prev_b;
            let mut rem = n;
            while rem > 0 {
                let ta = self.r4.tops[sa as usize];
                let bb = self.r4.bots[sb as usize];
                let tb_id = bb.top;
                if sa == tb_id {
                    rem -= ta.width;
                    sa = ta.prev;
                    sb = bb.prev;
                    continue;
                }
                let tb = self.r4.tops[tb_id as usize];
                let w = ta.width.min(tb.width);
                rem -= w;
                let joined = self.r4.join::<USE_MN>(ta.word, tb.word, &mut self.mn);
                if ta.width < tb.width {
                    // `sa` (width `ta.width`) splices right after `tb_id` on side 0; in backward order,
                    // `sa` would be visited IMMEDIATELY BEFORE `tb_id`! Wait: `tb_id` is ALREADY earlier
                    // (`<= sa - 1`) in `prev` chain, so to visit `sa` right BEFORE `tb_id` when walking `prev`
                    // downward, we can't splice *before* `tb_id` without modifying `tb_id`'s successor's `prev` —
                    // INSTEAD `Rule R4` (`src/cells.rs`!) swaps roles of `sa` and `tb_id`:
                    // `tb_id` keeps the remaining `tb.width - w` at its current top position, while `sa`?
                    // NO! In `Rule R4`:
                    // On bottom (`sb`), the front `w` rows of `tb_id` WERE JUST CONSUMED (`sb` advances to `bb.prev`?!
                    // No, `tb.width > w`, so `sb` still has `tb.width - w` rows left at the right end? NO! `sb` is the RIGHT end!).
                    // Wait: at the RIGHT end of `tb_id`:
                    // The RIGHTMOST `w` rows of `tb_id` (on bottom) matched `sa` (on top)!
                    // On top, `tb_id`'s RIGHTMOST `w` rows come FIRST when walking `prev` from the right!
                    // So when walking `prev` on top and eventually reaching `tb_id`:
                    // - FIRST we hit the rightmost `w` rows (which now carry `joined` and exit to `ta.other`!),
                    // - THEN we hit the left `tb.width - w` rows of `tb_id` (which still partner with `sb` on bottom)!
                    // HOW DOES `Rule R4` DO THAT WITHOUT `next`?!
                    // Look! Currently `something.prev == tb_id`, and `tb_id.prev == old_prev`!
                    // If `tb_id` becomes the RIGHTMOST `w`-piece (`width = w, word = joined, other = ta.other, prev = sa`),
                    // AND `sa` becomes the LEFT `(tb.width - w)`-piece (`width = tb.width - w, word = tb.word, other = sb, prev = tb.prev`),
                    // AND `bots[ta.other].top = tb_id`, `bots[sb].top = sa`, while advancing top front `sa = ta.prev`!!
                    // CHECK THAT!!
                    // Whoever pointed `prev -> tb_id` STILL points `prev -> tb_id` (the rightmost `w`-piece)!
                    // Then `tb_id.prev` goes to `sa` (now holding the left `tb.width - w` piece)!
                    // Then `sa.prev` goes to `tb.prev`!!
                    // ZERO `next` POINTERS! PURE GENIUS OF MONODROMY'S RULE R4!!
                    let next_sa = ta.prev;
                    let ta_bot = ta.other;
                    self.r4.tops[tb_id as usize] = R4Top {
                        prev: sa,
                        width: w,
                        word: joined,
                        other: ta_bot,
                    };
                    self.r4.bots[ta_bot as usize].top = tb_id;
                    self.r4.tops[sa as usize] = R4Top {
                        prev: tb.prev,
                        width: tb.width - w,
                        word: tb.word,
                        other: sb,
                    };
                    self.r4.bots[sb as usize].top = sa;
                    sa = next_sa;
                } else if ta.width > tb.width {
                    // Symmetric on side 1 (`bots`)!
                    // On top (`sa`), the rightmost `w` rows matched `sb` (`tb_id`), leaving `ta.width - w` on the left of `sa`.
                    // On bottom, `ta.other` is earlier in `prev` chain: its rightmost `w` rows come FIRST when walking `prev`,
                    // followed by its left `ta.width - w` rows!
                    // Reuse `tb_id` to hold the left `ta.width - w` piece (`or reuse sb`!):
                    // Specifically: `ta.other` (`ab_id`) becomes the rightmost `w`-piece (`top = tb_id`, `prev = sb`),
                    // `tb_id` holds `(width = w, word = joined, other = ab_id)`,
                    // `sb` becomes the left `ta.width - w` bot piece (`top = sa`, `prev = ab.prev`),
                    // `sa` keeps `(width = ta.width - w, word = ta.word, other = sb, prev = ta.prev)`,
                    // and bottom front advances `sb = bb.prev`!
                    let next_sb = bb.prev;
                    let ab_id = ta.other;
                    let ab = self.r4.bots[ab_id as usize];
                    self.r4.tops[tb_id as usize].width = w;
                    self.r4.tops[tb_id as usize].word = joined;
                    self.r4.tops[tb_id as usize].other = ab_id;
                    self.r4.bots[ab_id as usize] = R4Bot {
                        prev: sb,
                        top: tb_id,
                    };
                    self.r4.bots[sb as usize] = R4Bot {
                        prev: ab.prev,
                        top: sa,
                    };
                    self.r4.tops[sa as usize].width = ta.width - w;
                    self.r4.tops[sa as usize].other = sb;
                    sb = next_sb;
                } else {
                    // Exact Keane saddle connection (`ta.width == tb.width`)!
                    // `sa` and `sb` (`tb_id`) cancel at the front, splicing `joined` directly onto `(tb_id on top, ta.other on bot)`!
                    // Wait: `tb_id` is in top chain (`pointed to by someone's prev`), and `ta.other` (`ab_id`) is in bot chain!
                    // Simply update `tops[tb_id]` to `(prev = tb.prev, width = w, word = joined, other = ab_id)` and
                    // `bots[ab_id].top = tb_id`, and advance BOTH `sa = ta.prev` and `sb = bb.prev`!!
                    // Wait: what if `tb_id == sa`? Handled at top of loop!
                    let next_sa = ta.prev;
                    let next_sb = bb.prev;
                    let ab_id = ta.other;
                    self.r4.tops[tb_id as usize].word = joined;
                    self.r4.tops[tb_id as usize].other = ab_id;
                    self.r4.bots[ab_id as usize].top = tb_id;
                    sa = next_sa;
                    sb = next_sb;
                }
            }

            // Assign `bottom` coordinates inside `tops[tid].other` by walking surviving `sb` backward (`from m down to 0`)!
            let mut x_bot = m;
            let mut b_cur = sb;
            while b_cur != 0 {
                let tid = self.r4.bots[b_cur as usize].top;
                x_bot -= self.r4.tops[tid as usize].width;
                self.r4.tops[tid as usize].other = x_bot;
                b_cur = self.r4.bots[b_cur as usize].prev;
            }
            debug_assert_eq!(x_bot, 0);

            // Reverse surviving `sa` chain into `surv_top` so we emit bands in forward `a = 0..256` order
            // (preserving exact cache-friendly forward `wire` order and identical MN codes!):
            self.r4.surv_top.clear();
            let mut t_cur = sa;
            while t_cur != 0 {
                self.r4.surv_top.push(t_cur);
                t_cur = self.r4.tops[t_cur as usize].prev;
            }
            self.r4.surv_top.reverse();

            let mut cursors = [0usize; 256];
            let mut si = 0usize;
            let mut in_node = 0u32;
            let mut k = 0u32;
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                while k < end {
                    let tid = self.r4.surv_top[si];
                    let top = self.r4.tops[tid as usize];
                    let width = (end - k).min(top.width - in_node);
                    let dst = top.other + in_node;
                    let code = self.emit_band_code_r4(top.word, a as u8, words);
                    self.bands.push(Band {
                        end: k + width,
                        delta: dst.wrapping_sub(k),
                        code,
                    });
                    in_node += width;
                    k += width;
                    if in_node == top.width {
                        si += 1;
                        in_node = 0;
                    }
                }
            }
            cursors
        }

        pub fn construct(&mut self, records: &[u16], n: u32, p: u32, limit: u32) -> (usize, u32, u32) {
            self.strips.clear();
            if USE_MN {
                self.mn.next_epoch();
            }
            if USE_SLAB {
                self.slab_q.reset();
            } else {
                self.vec_q.resize_with(256, Vec::new);
            }

            let mut all = [0u32; 256];
            let mut thin = [0u32; 256];
            let mut thick = [0u32; 256];
            let mut m = 0u32;
            let mut mark = 0u32;

            let mut pos = 0u32;
            let mut idx = 0usize;
            while idx < records.len() {
                let rec = records[idx];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                idx += 1;
                while idx < records.len() && (records[idx] >> 8) as u8 == c {
                    width += u32::from(records[idx] & 255);
                    idx += 1;
                }
                if width <= limit {
                    let a = usize::from(c);
                    if p.wrapping_sub(pos) < width {
                        mark = m + (p - pos);
                    }
                    if USE_SLAB {
                        self.slab_q.push_rows(a, m, width);
                    } else {
                        push_rows_vec(&mut self.vec_q[a], thin[a], m, width);
                    }
                    thin[a] += width;
                    m += width;
                    all[a] += width;
                } else if p.wrapping_sub(pos) >= width {
                    let a = usize::from(c);
                    self.strips.push(Strip {
                        from: pos,
                        to: all[a],
                        width,
                        byte: c,
                    });
                    thick[a] += 1;
                    all[a] += width;
                } else {
                    let end = pos + width;
                    let a = usize::from(c);
                    if pos < p {
                        let w0 = p - pos;
                        self.strips.push(Strip {
                            from: pos,
                            to: all[a],
                            width: w0,
                            byte: c,
                        });
                        thick[a] += 1;
                        all[a] += w0;
                    }
                    mark = m;
                    if USE_SLAB {
                        self.slab_q.push_rows(a, m, 1);
                    } else {
                        push_rows_vec(&mut self.vec_q[a], thin[a], m, 1);
                    }
                    thin[a] += 1;
                    m += 1;
                    all[a] += 1;
                    if p + 1 < end {
                        let w2 = end - p - 1;
                        self.strips.push(Strip {
                            from: p + 1,
                            to: all[a],
                            width: w2,
                            byte: c,
                        });
                        thick[a] += 1;
                        all[a] += w2;
                    }
                }
                pos += width;
            }
            if USE_SLAB {
                self.slab_q.finalize_tails();
            }

            let first = starts(all);
            let mut write = starts(thick);
            self.image_order.resize(self.strips.len(), 0);
            for (i, s) in self.strips.iter_mut().enumerate() {
                let a = usize::from(s.byte);
                s.to += first[a];
                self.image_order[write[a] as usize] = i as u32;
                write[a] += 1;
            }

            self.bands.clear();
            self.spelling.clear();
            self.offsets.clear();
            self.offsets.push(0);
            let mut words = 0usize;
            let cursors = if USE_R4 {
                self.induced_r4(n, m, &thin, &mut words)
            } else {
                self.induced_ep(n, &thin, &mut words)
            };

            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let wide = (255 + words as u32) > (u32::MAX >> bits);
            self.edges.resize(m as usize, 0);
            self.extra.clear();
            if wide {
                self.extra.resize(m as usize, 0);
            }

            let mut k = 0u32;
            if USE_SLAB {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut pid = self.slab_q.head[a];
                    let mut p_off = 0usize;
                    let mut p_rem = if pid != NO_PAGE {
                        self.slab_q.page_len[pid as usize] as usize
                    } else {
                        0
                    };
                    for b in &self.bands[cursors[a]..end] {
                        let mut span = (b.end - k) as usize;
                        let mut dst = k.wrapping_add(b.delta) as usize;
                        let pack = if wide { 0 } else { b.code << bits };
                        if wide {
                            self.extra[dst..dst + span].fill(b.code);
                        }
                        while span > 0 {
                            let take = span.min(p_rem);
                            let base = (pid as usize) * PAGE_STRIDE + p_off;
                            let src = &self.slab_q.slab[base..base + take];
                            for (target, &j) in self.edges[dst..dst + take].iter_mut().zip(src) {
                                *target = pack | j;
                            }
                            dst += take;
                            span -= take;
                            p_off += take;
                            p_rem -= take;
                            if p_rem == 0 && span > 0 {
                                pid = self.slab_q.next_page[pid as usize];
                                p_off = 0;
                                p_rem = self.slab_q.page_len[pid as usize] as usize;
                            }
                        }
                        if p_rem == 0 && pid != NO_PAGE {
                            pid = self.slab_q.next_page[pid as usize];
                            p_off = 0;
                            p_rem = if pid != NO_PAGE {
                                self.slab_q.page_len[pid as usize] as usize
                            } else {
                                0
                            };
                        }
                        k = b.end;
                    }
                }
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut from = 0;
                    for b in &self.bands[cursors[a]..end] {
                        let span = (b.end - k) as usize;
                        let dst = k.wrapping_add(b.delta) as usize;
                        let pack = if wide { 0 } else { b.code << bits };
                        if wide {
                            self.extra[dst..dst + span].fill(b.code);
                        }
                        let src = &self.vec_q[a][from..from + span];
                        for (target, &j) in self.edges[dst..dst + span].iter_mut().zip(src) {
                            *target = pack | j;
                        }
                        from += span;
                        k = b.end;
                    }
                }
            }
            (words, mark, bits)
        }

        pub fn walk(&mut self, n: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
            inline_words(&self.spelling, &self.offsets, &mut self.inline16, &mut self.lens);
            emit(
                &self.edges,
                &self.extra,
                &self.spelling,
                &self.offsets,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
            );
        }

        pub fn run(&mut self, records: &[u16], n: usize, p: u32, limit: u32, out: &mut Vec<u8>) -> usize {
            let (words, mark, bits) = self.construct(records, n as u32, p, limit);
            self.walk(n, mark, bits, out);
            words
        }
    }
}

struct BlockInput {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
}

fn load_blocks() -> Vec<BlockInput> {
    let dir = Path::new("/home/antigravity/admitted-bwt");
    let manifest = fs::read_to_string(dir.join("manifest.json")).expect("manifest");
    // Simple parser for the 6 blocks in manifest.json:
    let spec = [
        ("block-0.L", 516362u32),
        ("block-2.L", 408857u32),
        ("block-85.L", 220144u32),
        ("block-143.L", 856436u32),
        ("block-210.L", 460236u32),
        ("block-207.L", 502759u32),
    ];
    let _ = manifest;
    spec.iter()
        .map(|(f, p)| {
            let bytes = fs::read(dir.join(f)).expect(f);
            let recs = records_of(&bytes);
            BlockInput {
                name: (*f).to_string(),
                n: bytes.len(),
                p: *p,
                records: recs,
            }
        })
        .collect()
}

fn median_us(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let blocks = Arc::new(load_blocks());
    println!("Loaded {} admitted Wikidata blocks (size_of Node058={}B, R4Top=16B, R4Bot=8B)", blocks.len(), size_of::<[u32; 6]>());

    // 1. Verify byte-exact equality across all 5 arms on all 6 blocks + measure single-block WS!
    println!("\n=== 1. Single-Block Byte-Exact Verification & Working-Set Audit (KiB) ===");
    println!(
        "{:<12} | {:>9} {:>9} {:>9} {:>9} {:>9} | {:>8} {:>8}",
        "Block", "WS_058", "WS_MN16", "WS_R4@16", "WS_R4+MN", "WS_R4MNS", "Dict058", "DictMN"
    );
    for blk in blocks.iter() {
        let mut k0 = open_058::Kernel::default();
        let mut k_mn = open_variants::VariantKernel::<false, true, false>::default();
        let mut k_r4 = open_variants::VariantKernel::<false, false, true>::default();
        let mut k_r4mn = open_variants::VariantKernel::<false, true, true>::default();
        let mut k_all = open_variants::VariantKernel::<true, true, true>::default();

        let (mut o0, mut o1, mut o2, mut o3, mut o4) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        k0.run(&blk.records, blk.n, blk.p, LIMIT, &mut o0);
        k_mn.run(&blk.records, blk.n, blk.p, LIMIT, &mut o1);
        k_r4.run(&blk.records, blk.n, blk.p, LIMIT, &mut o2);
        k_r4mn.run(&blk.records, blk.n, blk.p, LIMIT, &mut o3);
        k_all.run(&blk.records, blk.n, blk.p, LIMIT, &mut o4);

        assert_eq!(o0, o1, "MN16 mismatch on {}", blk.name);
        assert_eq!(o0, o2, "R4@16 mismatch on {}", blk.name);
        assert_eq!(o0, o3, "R4+MN mismatch on {}", blk.name);
        assert_eq!(o0, o4, "R4+MN+Slab mismatch on {}", blk.name);

        println!(
            "{:<12} | {:>7.1} K {:>7.1} K {:>7.1} K {:>7.1} K {:>7.1} K | {:>6.1} K {:>6.1} K",
            blk.name,
            k0.total_ws_bytes() as f64 / 1024.0,
            k_mn.total_ws_bytes() as f64 / 1024.0,
            k_r4.total_ws_bytes() as f64 / 1024.0,
            k_r4mn.total_ws_bytes() as f64 / 1024.0,
            k_all.total_ws_bytes() as f64 / 1024.0,
            k0.dict_bytes() as f64 / 1024.0,
            k_all.dict_bytes() as f64 / 1024.0,
        );
    }

    // 2. Persistent Worker Steady-State Memory + 25-Round Interleaved 1-Thread Timing!
    let mut k0 = open_058::Kernel::default();
    let mut k_mn = open_variants::VariantKernel::<false, true, false>::default();
    let mut k_r4 = open_variants::VariantKernel::<false, false, true>::default();
    let mut k_r4mn = open_variants::VariantKernel::<false, true, true>::default();
    let mut k_all = open_variants::VariantKernel::<true, true, true>::default();
    let mut out = Vec::with_capacity(900_016);

    for _ in 0..2 {
        for blk in blocks.iter() {
            k0.run(&blk.records, blk.n, blk.p, LIMIT, &mut out);
            k_mn.run(&blk.records, blk.n, blk.p, LIMIT, &mut out);
            k_r4.run(&blk.records, blk.n, blk.p, LIMIT, &mut out);
            k_r4mn.run(&blk.records, blk.n, blk.p, LIMIT, &mut out);
            k_all.run(&blk.records, blk.n, blk.p, LIMIT, &mut out);
        }
    }

    println!("\n=== 2. Persistent Multi-Block Worker Steady-State Footprint (KiB) ===");
    println!(
        "  Native-058       : Total = {:>7.1} KiB (Queues = {:>7.1} KiB) -> 10 workers = {:>5.2} MiB",
        k0.total_ws_bytes() as f64 / 1024.0,
        k0.queue_bytes() as f64 / 1024.0,
        10.0 * k0.total_ws_bytes() as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  MN-Quotient16    : Total = {:>7.1} KiB (Queues = {:>7.1} KiB) -> 10 workers = {:>5.2} MiB",
        k_mn.total_ws_bytes() as f64 / 1024.0,
        k_mn.queue_bytes() as f64 / 1024.0,
        10.0 * k_mn.total_ws_bytes() as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  R4@16-Serial     : Total = {:>7.1} KiB (Queues = {:>7.1} KiB) -> 10 workers = {:>5.2} MiB",
        k_r4.total_ws_bytes() as f64 / 1024.0,
        k_r4.queue_bytes() as f64 / 1024.0,
        10.0 * k_r4.total_ws_bytes() as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  R4@16+MN16       : Total = {:>7.1} KiB (Queues = {:>7.1} KiB) -> 10 workers = {:>5.2} MiB",
        k_r4mn.total_ws_bytes() as f64 / 1024.0,
        k_r4mn.queue_bytes() as f64 / 1024.0,
        10.0 * k_r4mn.total_ws_bytes() as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  R4@16+MN+Slab256 : Total = {:>7.1} KiB (Queues = {:>7.1} KiB) -> 10 workers = {:>5.2} MiB",
        k_all.total_ws_bytes() as f64 / 1024.0,
        k_all.queue_bytes() as f64 / 1024.0,
        10.0 * k_all.total_ws_bytes() as f64 / (1024.0 * 1024.0)
    );

    let rounds = 25usize;
    println!("\n=== 3. Paired 25-Round Native Safe-Rust 1-Thread Timings vs `mibz2 0.5.8` (us) ===");
    println!(
        "{:<12} | {:>8} | {:>14} | {:>14} | {:>14} | {:>14}",
        "Block", "0.5.8", "MN-Quot16", "R4@16-Ser", "R4@16+MN16", "R4+MN+Slab"
    );
    let mut sum_t = [0.0f64; 5];
    let mut sum_c = [0.0f64; 5];
    let mut sum_w = [0.0f64; 5];

    for blk in blocks.iter() {
        let mut t = [const { Vec::new() }; 5];
        let mut tc = [const { Vec::new() }; 5];
        let mut tw = [const { Vec::new() }; 5];
        for r in 0..rounds {
            for step in 0..5 {
                let arm = (r + step) % 5;
                match arm {
                    0 => {
                        let s0 = Instant::now();
                        let (_, mark, bits) = k0.construct(&blk.records, blk.n as u32, blk.p, LIMIT);
                        let s1 = Instant::now();
                        k0.walk(blk.n, mark, bits, &mut out);
                        let s2 = Instant::now();
                        tc[0].push(s1.duration_since(s0).as_secs_f64() * 1e6);
                        tw[0].push(s2.duration_since(s1).as_secs_f64() * 1e6);
                        t[0].push(s2.duration_since(s0).as_secs_f64() * 1e6);
                    }
                    1 => {
                        let s0 = Instant::now();
                        let (_, mark, bits) = k_mn.construct(&blk.records, blk.n as u32, blk.p, LIMIT);
                        let s1 = Instant::now();
                        k_mn.walk(blk.n, mark, bits, &mut out);
                        let s2 = Instant::now();
                        tc[1].push(s1.duration_since(s0).as_secs_f64() * 1e6);
                        tw[1].push(s2.duration_since(s1).as_secs_f64() * 1e6);
                        t[1].push(s2.duration_since(s0).as_secs_f64() * 1e6);
                    }
                    2 => {
                        let s0 = Instant::now();
                        let (_, mark, bits) = k_r4.construct(&blk.records, blk.n as u32, blk.p, LIMIT);
                        let s1 = Instant::now();
                        k_r4.walk(blk.n, mark, bits, &mut out);
                        let s2 = Instant::now();
                        tc[2].push(s1.duration_since(s0).as_secs_f64() * 1e6);
                        tw[2].push(s2.duration_since(s1).as_secs_f64() * 1e6);
                        t[2].push(s2.duration_since(s0).as_secs_f64() * 1e6);
                    }
                    3 => {
                        let s0 = Instant::now();
                        let (_, mark, bits) = k_r4mn.construct(&blk.records, blk.n as u32, blk.p, LIMIT);
                        let s1 = Instant::now();
                        k_r4mn.walk(blk.n, mark, bits, &mut out);
                        let s2 = Instant::now();
                        tc[3].push(s1.duration_since(s0).as_secs_f64() * 1e6);
                        tw[3].push(s2.duration_since(s1).as_secs_f64() * 1e6);
                        t[3].push(s2.duration_since(s0).as_secs_f64() * 1e6);
                    }
                    _ => {
                        let s0 = Instant::now();
                        let (_, mark, bits) = k_all.construct(&blk.records, blk.n as u32, blk.p, LIMIT);
                        let s1 = Instant::now();
                        k_all.walk(blk.n, mark, bits, &mut out);
                        let s2 = Instant::now();
                        tc[4].push(s1.duration_since(s0).as_secs_f64() * 1e6);
                        tw[4].push(s2.duration_since(s1).as_secs_f64() * 1e6);
                        t[4].push(s2.duration_since(s0).as_secs_f64() * 1e6);
                    }
                }
                std::hint::black_box(out[0]);
            }
        }
        let m_tot: [f64; 5] = std::array::from_fn(|i| median_us(t[i].clone()));
        let m_con: [f64; 5] = std::array::from_fn(|i| median_us(tc[i].clone()));
        let m_wlk: [f64; 5] = std::array::from_fn(|i| median_us(tw[i].clone()));
        for i in 0..5 {
            sum_t[i] += m_tot[i];
            sum_c[i] += m_con[i];
            sum_w[i] += m_wlk[i];
        }
        println!(
            "{:<12} | {:>8.1} | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x)",
            blk.name,
            m_tot[0],
            m_tot[1], m_tot[1] / m_tot[0],
            m_tot[2], m_tot[2] / m_tot[0],
            m_tot[3], m_tot[3] / m_tot[0],
            m_tot[4], m_tot[4] / m_tot[0],
        );
    }
    println!(
        "{:<12} | {:>8.1} | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x)",
        "SUM (6 blks)",
        sum_t[0],
        sum_t[1], sum_t[1] / sum_t[0],
        sum_t[2], sum_t[2] / sum_t[0],
        sum_t[3], sum_t[3] / sum_t[0],
        sum_t[4], sum_t[4] / sum_t[0],
    );
    println!(
        "  construct  | {:>8.1} | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x)",
        sum_c[0],
        sum_c[1], sum_c[1] / sum_c[0],
        sum_c[2], sum_c[2] / sum_c[0],
        sum_c[3], sum_c[3] / sum_c[0],
        sum_c[4], sum_c[4] / sum_c[0],
    );
    println!(
        "  walk/emit  | {:>8.1} | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x) | {:>7.1} ({:.3}x)",
        sum_w[0],
        sum_w[1], sum_w[1] / sum_w[0],
        sum_w[2], sum_w[2] / sum_w[0],
        sum_w[3], sum_w[3] / sum_w[0],
        sum_w[4], sum_w[4] / sum_w[0],
    );

    // 4. 2-Thread Concurrent Worker Wall-Time Check (2 physical cores on this VM)
    let mut wall_2t = [const { Vec::new() }; 4];
    for r in 0..17 {
        for step in 0..4 {
            let arm = (r + step) % 4; // 0 = 0.5.8, 1 = MN16, 2 = R4+MN16, 3 = R4+MN+Slab256
            let b_ref = Arc::clone(&blocks);
            let s0 = Instant::now();
            thread::scope(|s| {
                for tid in 0..2 {
                    let b = Arc::clone(&b_ref);
                    s.spawn(move || {
                        let mut out_t = Vec::with_capacity(900_016);
                        match arm {
                            0 => {
                                let mut k = open_058::Kernel::default();
                                for it in 0..3 {
                                    for (bi, blk) in b.iter().enumerate() {
                                        if (bi + it) % 2 == tid {
                                            k.run(&blk.records, blk.n, blk.p, LIMIT, &mut out_t);
                                            std::hint::black_box(out_t[0]);
                                        }
                                    }
                                }
                            }
                            1 => {
                                let mut k = open_variants::VariantKernel::<false, true, false>::default();
                                for it in 0..3 {
                                    for (bi, blk) in b.iter().enumerate() {
                                        if (bi + it) % 2 == tid {
                                            k.run(&blk.records, blk.n, blk.p, LIMIT, &mut out_t);
                                            std::hint::black_box(out_t[0]);
                                        }
                                    }
                                }
                            }
                            2 => {
                                let mut k = open_variants::VariantKernel::<false, true, true>::default();
                                for it in 0..3 {
                                    for (bi, blk) in b.iter().enumerate() {
                                        if (bi + it) % 2 == tid {
                                            k.run(&blk.records, blk.n, blk.p, LIMIT, &mut out_t);
                                            std::hint::black_box(out_t[0]);
                                        }
                                    }
                                }
                            }
                            _ => {
                                let mut k = open_variants::VariantKernel::<true, true, true>::default();
                                for it in 0..3 {
                                    for (bi, blk) in b.iter().enumerate() {
                                        if (bi + it) % 2 == tid {
                                            k.run(&blk.records, blk.n, blk.p, LIMIT, &mut out_t);
                                            std::hint::black_box(out_t[0]);
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            });
            wall_2t[arm].push(s0.elapsed().as_secs_f64() * 1e3);
        }
    }
    let w0 = median_us(wall_2t[0].clone());
    let w1 = median_us(wall_2t[1].clone());
    let w2 = median_us(wall_2t[2].clone());
    let w3 = median_us(wall_2t[3].clone());
    println!(
        "\n=== 4. 2-Worker Concurrent Stream (18 blocks, ms): 0.5.8={:.2} | MN16={:.2} ({:.3}x) | R4+MN16={:.2} ({:.3}x) | R4+MN+Slab256={:.2} ({:.3}x) ===",
        w0, w1, w1 / w0, w2, w2 / w0, w3, w3 / w0
    );
}
