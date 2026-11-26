//! Induce a first return directly on an FM word interval in the original byte BWT.
use crate::bwt::{InverseBwt, Segmented};
use crate::runs::Runs;
use crate::Kernels;

const NONE: u32 = u32::MAX;

#[derive(Clone, Copy, Default)]
struct Run {
    start: u32,
    lf: u32,
    byte: u8,
}
#[derive(Clone, Copy, Default)]
struct Piece {
    id: u32,
    width: u32,
}
#[derive(Clone, Copy, Default)]
struct Node {
    lo: u32,
    hi: u32,
    parent: u32,
    first: u8,
    edge: u32,
    end: u32,
    sel: u32,
    sels: u32,
    base: u32,
    word: u32,
    out: u32,
    cursor: u32,
    used: u32,
}
impl Node {
    fn new(lo: u32, parent: u32, byte: u8) -> Self {
        Self {
            lo,
            hi: lo,
            parent,
            first: byte,
            word: NONE,
            ..Self::default()
        }
    }
}

#[derive(Default)]
pub struct Candidate {
    fallback: Segmented,
    heap: std::collections::BinaryHeap<(u32, std::cmp::Reverse<u32>)>,
    parts: Vec<(u32, u32, u32)>,
    runs: Vec<Run>,
    page_first: Vec<u32>,
    starts: Vec<u64>,
    nodes: Vec<Node>,
    selectors: Vec<Piece>,
    edges: Vec<u8>,
    words: Vec<Piece>,
    dict: Vec<u8>,
    merged: Vec<Piece>,
    links: Vec<u32>,
    #[cfg(test)]
    stats: [u64; 8],
}

impl Candidate {
    fn prepare(&mut self, caps: &[u16], n: u32, pattern: &[u8]) -> Option<(u32, u32)> {
        self.runs.clear();
        let mut counts = [0u32; 256];
        let mut row = 0u32;
        let mut prev = None;
        for &rec in caps {
            let c = (rec >> 8) as u8;
            let len = (rec & 255) as u32;
            if len == 0 || len > 127 {
                return None;
            }
            if Some(c) != prev {
                self.runs.push(Run {
                    start: row,
                    lf: counts[c as usize],
                    byte: c,
                });
                prev = Some(c);
            }
            counts[c as usize] += len;
            row += len;
        }
        let last = *pattern.last()?;
        if row != n || counts[last as usize] == 0 {
            return None;
        }
        let width = counts[last as usize];
        let mut total = 0;
        for c in &mut counts {
            let width = *c;
            *c = total;
            total += width;
        }
        let pages = n.div_ceil(64) as usize;
        self.starts.clear();
        self.starts.resize(pages, 0);
        self.page_first.clear();
        self.page_first.resize(pages, 0);
        for r in &mut self.runs {
            r.lf += counts[r.byte as usize];
            let offset = r.start & 63;
            if offset != 0 {
                self.starts[(r.start >> 6) as usize] |= 1u64 << offset;
            }
        }
        self.runs.push(Run {
            start: n,
            lf: 0,
            byte: 0,
        });
        let mut k = 0;
        for (page, base) in self.page_first.iter_mut().enumerate() {
            while self.runs[k + 1].start <= 64 * page as u32 {
                k += 1;
            }
            *base = k as u32;
        }
        let mut start = counts[last as usize];
        let mut end = start + width;
        for &symbol in pattern[..pattern.len() - 1].iter().rev() {
            let mut first = None;
            let mut total = 0;
            let mut idx = self.locate(start);
            while self.runs[idx].start < end {
                let r = self.runs[idx];
                if r.byte == symbol {
                    let lo = start.max(r.start);
                    first.get_or_insert(r.lf + (lo - r.start));
                    total += end.min(self.runs[idx + 1].start) - lo;
                }
                idx += 1;
            }
            start = first?;
            end = start + total;
        }
        Some((start, end - start))
    }
    #[inline]
    fn locate(&self, x: u32) -> usize {
        let page = (x >> 6) as usize;
        let mask = u64::MAX >> (63 - (x & 63));
        (self.page_first[page] + (self.starts[page] & mask).count_ones()) as usize
    }
    fn select(&mut self, section: u32, count: u32, k: usize) -> u32 {
        use std::cmp::Reverse;
        let mut widths = [0u32; 256];
        let mut starts = [0u32; 256];
        let mut touched = [0u8; 256];
        let mut best = Vec::with_capacity(k + 1);
        let mut cutoff = 0;
        self.heap.clear();
        self.heap.push((count, Reverse(section)));
        while let Some((width, Reverse(left))) = self.heap.pop() {
            if width < cutoff {
                break;
            }
            let right = left + width;
            let mut nbytes = 0;
            let mut index = self.locate(left);
            while self.runs[index].start < right {
                let run = self.runs[index];
                let lo = left.max(run.start);
                let byte = usize::from(run.byte);
                if widths[byte] == 0 {
                    starts[byte] = run.lf + (lo - run.start);
                    touched[nbytes] = run.byte;
                    nbytes += 1;
                }
                widths[byte] += right.min(self.runs[index + 1].start) - lo;
                index += 1;
            }
            for &byte in &touched[..nbytes] {
                let slot = usize::from(byte);
                let start = starts[slot];
                let frequency = std::mem::take(&mut widths[slot]);
                if frequency < cutoff {
                    continue;
                }
                if (section..section + count).contains(&start) {
                    best.push((frequency, Reverse(start)));
                    best.sort_unstable_by(|a,b| b.cmp(a));
                    best.truncate(k);
                    if best.len() == k {
                        cutoff = best[k-1].0;
                    }
                } else {
                    self.heap.push((frequency, Reverse(start)));
                }
            }
        }
        self.parts.clear();
        best.sort_unstable_by_key(|x| x.1.0);
        let mut total = 0;
        for (width, Reverse(start)) in best {
            if let Some(last) = self.parts.last_mut().filter(|p| p.1 == start) {
                last.1 += width;
            } else {
                self.parts.push((start, start + width, total));
            }
            total += width;
        }
        total
    }

    #[inline]
    fn union_rank(&self, row: u32) -> Option<u32> {
        self.parts.iter().find(|p| p.0 <= row && row < p.1).map(|p| p.2 + (row - p.0))
    }

    #[inline]
    fn hit(&self, byte: u8, row: u32) -> Option<u32> {
        if byte == b'"' { self.union_rank(row) } else { None }
    }

    fn transfer(&self, primary: u32) -> Option<(u32, u32)> {
        let (mut row, mut skip) = (primary, 0);
        loop {
            if let Some(rank) = self.union_rank(row) {
                return Some((rank,skip));
            }
            let r = self.runs[self.locate(row)];
            row = r.lf + (row - r.start);
            skip += 1;
            if row == primary { return None; }
        }
    }

    fn build(&mut self) {
        self.nodes.clear();
        self.selectors.clear();
        self.edges.clear();
        self.words.clear();
        self.dict.clear();
        for &(left, right, _) in &self.parts {
            let mut root = Node::new(left, NONE, 0);
            root.hi = right;
            self.nodes.push(root);
        }
        let mut epoch = [NONE; 256];
        let mut active = [0usize; 256];
        let mut i = 0;
        while i < self.nodes.len() {
            let node = self.nodes[i];
            let (mut lo, mut hi) = (node.lo, node.hi);
            self.nodes[i].edge = self.edges.len() as u32;
            if node.parent != NONE {
                self.edges.push(node.first);
            }
            let mut k;
            loop {
                k = self.locate(lo);
                #[cfg(test)]
                {
                    self.stats[0] += 1;
                }
                let r = self.runs[k];
                let mapped = r.lf + (lo - r.start);
                if self.runs[k + 1].start < hi || self.hit(r.byte, mapped).is_some() {
                    break;
                }
                self.edges.push(r.byte);
                let width = hi - lo;
                lo = r.lf + (lo - r.start);
                hi = lo + width;
            }
            self.nodes[i].end = self.edges.len() as u32;
            self.nodes[i].sel = self.selectors.len() as u32;
            let begin = self.selectors.len();
            while self.runs[k].start < hi {
                let r = self.runs[k];
                let a = lo.max(r.start);
                let width = hi.min(self.runs[k + 1].start) - a;
                let mapped = r.lf + (a - r.start);
                let c = r.byte as usize;
                let id = if let Some(rank) = self.hit(r.byte, mapped) {
                    if self.nodes[i].word == NONE {
                        self.nodes[i].base = rank;
                        self.nodes[i].word = self.words.len() as u32;
                        let pos = self.dict.len() as u32;
                        self.dict.push(r.byte);
                        let mut j = i as u32;
                        while j != NONE {
                            let n = self.nodes[j as usize];
                            self.dict
                                .extend(self.edges[n.edge as usize..n.end as usize].iter().rev());
                            j = n.parent;
                        }
                        self.words.push(Piece {
                            id: pos,
                            width: self.dict.len() as u32 - pos,
                        });
                    }
                    NONE
                } else {
                    if epoch[c] != i as u32 {
                        epoch[c] = i as u32;
                        active[c] = self.nodes.len();
                        self.nodes.push(Node::new(mapped, i as u32, r.byte));
                    }
                    let target = active[c];
                    debug_assert_eq!(self.nodes[target].hi, mapped);
                    self.nodes[target].hi += width;
                    target as u32
                };
                if self.selectors.len() > begin && self.selectors.last().unwrap().id == id {
                    self.selectors.last_mut().unwrap().width += width;
                } else {
                    self.selectors.push(Piece { id, width });
                }
                k += 1;
            }
            self.nodes[i].sels = self.selectors.len() as u32 - self.nodes[i].sel;
            i += 1;
        }
    }

    #[inline]
    fn emit(merged: &mut Vec<Piece>, start: usize, id: u32, width: u32) {
        if merged.len() > start && merged.last().unwrap().id == id {
            merged.last_mut().unwrap().width += width;
        } else {
            merged.push(Piece { id, width });
        }
    }

    fn inverse_merge(&mut self) {
        self.merged.clear();
        for i in (0..self.nodes.len()).rev() {
            let node = self.nodes[i];
            let start = self.merged.len();
            self.nodes[i].out = start as u32;
            self.nodes[i].cursor = start as u32;
            for k in node.sel..node.sel + node.sels {
                let Piece { id, mut width } = self.selectors[k as usize];
                if id == NONE {
                    Self::emit(&mut self.merged, start, i as u32, width);
                    #[cfg(test)]
                    {
                        self.stats[1] += 1;
                    }
                } else {
                    let child = &mut self.nodes[id as usize];
                    while width > 0 {
                        let piece = self.merged[child.cursor as usize];
                        let count = width.min(piece.width - child.used);
                        Self::emit(&mut self.merged, start, piece.id, count);
                        #[cfg(test)]
                        {
                            self.stats[1] += 1;
                        }
                        width -= count;
                        child.used += count;
                        if child.used == piece.width {
                            child.cursor += 1;
                            child.used = 0;
                        }
                    }
                }
            }
        }
    }

    fn expand(
        &mut self,
        mut row: u32,
        mut skip: u32,
        tokens: u32,
        n: u32,
        out: &mut Vec<u8>,
    ) -> bool {
        let bits = 32 - (tokens - 1).leading_zeros();
        if self.words.len() - 1 > (u32::MAX >> bits) as usize {
            return false;
        }
        let mask = (1u32 << bits) - 1;
        self.links.resize(tokens as usize, 0);
        let mut input = 0;
        for root in 0..self.parts.len() {
            let end = if root == 0 { self.merged.len() } else { self.nodes[root-1].out as usize };
            for p in &self.merged[self.nodes[root].out as usize..end] {
            let node = &mut self.nodes[p.id as usize];
            for j in node.base..node.base + p.width {
                self.links[j as usize] = (node.word << bits) | input;
                input += 1;
            }
            node.base += p.width;
        }
        }
        debug_assert_eq!(input, tokens);
        out.clear();
        out.reserve(n as usize);
        while out.len() < n as usize {
            let link = self.links[row as usize];
            let p = self.words[(link >> bits) as usize];
            let count = (p.width - skip).min(n - out.len() as u32);
            let a = (p.id + skip) as usize;
            out.extend_from_slice(&self.dict[a..a + count as usize]);
            #[cfg(test)]
            {
                self.stats[2] += 1;
            }
            row = link & mask;
            skip = 0;
        }
        true
    }

    fn attempt<const FORCE: bool>(
        &mut self,
        caps: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> bool {
        self.for_k::<FORCE>(caps, n, p, out, 8)
    }

    fn for_k<const FORCE: bool>(
        &mut self,
        caps: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: usize,
    ) -> bool {
        if n == 0 || n > 900000 || p as usize >= n || caps.len() > n {
            return false;
        }
        if !FORCE && (caps.len() * 20 >= 3 * n || caps.len() * 64 < n) {
            return false;
        }
        #[cfg(test)]
        {
            self.stats = [0; 8];
        }
        let Some((quote, tokens)) = self.prepare(caps, n as u32, b"\"") else {
            return false;
        };
        let tokens = self.select(quote, tokens, k);
        let Some((row, skip)) = self.transfer(p) else {
            return false;
        };
        self.build();
        self.inverse_merge();
        #[cfg(test)]
        {
            self.stats[3] = skip as u64;
            self.stats[4] = tokens as u64;
        }
        self.expand(row, skip, tokens, n as u32, out)
    }
}

impl InverseBwt for Candidate {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(ll, counts, p, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(ll, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        self.attempt::<false>(r.records(), n, p, out) || self.fallback.invert_runs(r, n, p, out, k)
    }
}

#[cfg(test)]
#[path = "checks.rs"]
mod checks;
