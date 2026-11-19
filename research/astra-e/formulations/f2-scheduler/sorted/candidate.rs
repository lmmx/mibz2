//! Quote-ended phrase BWT, with unary suffix paths contracted before materializing nodes.
use crate::bwt::{InverseBwt, Segmented};
use crate::runs::Runs;
use crate::Kernels;

#[path = "../../f2c/second.rs"]
mod second;

const QUOTE: u8 = b'"';
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

#[derive(Clone, Copy)]
struct Job {
    lo: u32,
    hi: u32,
    id: u32,
    label: u32,
}
#[derive(Default)]
pub struct Candidate {
    current: Vec<Vec<Job>>,
    next: Vec<Vec<Job>>,
    labels: Vec<u32>,
    fallback: Segmented,
    second: second::Second,
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
    fn prepare(&mut self, caps: &[u16], n: u32) -> Option<(u32, u32)> {
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
        if row != n || counts[QUOTE as usize] == 0 {
            return None;
        }
        let tokens = counts[QUOTE as usize];
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
        Some((counts[QUOTE as usize], tokens))
    }
    #[inline]
    fn locate(&self, x: u32) -> usize {
        let page = (x >> 6) as usize;
        let mask = u64::MAX >> (63 - (x & 63));
        (self.page_first[page] + (self.starts[page] & mask).count_ones()) as usize
    }
    fn transfer_mark(&self, p: u32, quote: u32) -> Option<(u32, u32)> {
        let (mut x, mut skip) = (p, 0);
        loop {
            let r = self.runs[self.locate(x)];
            x = r.lf + (x - r.start);
            if r.byte == QUOTE {
                return Some((x - quote, skip));
            }
            skip += 1;
            if x == p {
                return None;
            }
        }
    }

    fn build(&mut self, quote: u32, tokens: u32) {
        self.nodes.clear();
        self.selectors.clear();
        self.words.clear();
        self.dict.clear();
        self.labels.clear();
        // Index zero terminates a spelling chain. Labels encode one edge byte
        // and the index of the suffix; there are at most n distinct intervals.
        self.labels.push(0);
        self.labels.push(u32::from(QUOTE));
        let mut root = Node::new(quote, NONE, QUOTE);
        root.hi += tokens;
        self.nodes.push(root);
        let mut current = std::mem::take(&mut self.current);
        let mut next = std::mem::take(&mut self.next);
        current.resize_with(256, Vec::new);
        next.resize_with(256, Vec::new);
        let mut mask = [0u64; 4];
        mask[(QUOTE / 64) as usize] |= 1 << (QUOTE % 64);
        current[QUOTE as usize].push(Job {
            lo: quote,
            hi: quote + tokens,
            id: 0,
            label: 1,
        });
        let mut epoch = [NONE; 256];
        let mut active = [0usize; 256];
        while mask != [0; 4] {
            let mut new_mask = [0u64; 4];
            let mut cursor = 0;
            for group in 0..4 {
                let mut used = mask[group];
                while used != 0 {
                    let ch = group * 64 + used.trailing_zeros() as usize;
                    used &= used - 1;
                    for job in &current[ch] {
                        let (lo, hi) = (job.lo, job.hi);
                        // Across a depth the query order is increasing. Keep
                        // the previous original run whenever it covers lo.
                        if self.runs[cursor + 1].start <= lo {
                            cursor = self.locate(lo);
                        }
                        let mut k = cursor;
                        let r = self.runs[k];
                        if self.runs[k + 1].start >= hi && r.byte != QUOTE {
                            let c = r.byte as usize;
                            let mapped = r.lf + (lo - r.start);
                            let label = self.labels.len() as u32;
                            self.labels.push(job.label << 8 | u32::from(r.byte));
                            next[c].push(Job {
                                lo: mapped,
                                hi: mapped + (hi - lo),
                                id: job.id,
                                label,
                            });
                            new_mask[c / 64] |= 1u64 << (c % 64);
                            continue;
                        }
                        let i = job.id as usize;
                        self.nodes[i].sel = self.selectors.len() as u32;
                        let begin = self.selectors.len();
                        while self.runs[k].start < hi {
                            let r = self.runs[k];
                            let a = lo.max(r.start);
                            let width = hi.min(self.runs[k + 1].start) - a;
                            let mapped = r.lf + (a - r.start);
                            let c = r.byte as usize;
                            let id = if r.byte == QUOTE {
                                if self.nodes[i].word == NONE {
                                    self.nodes[i].base = mapped - quote;
                                    self.nodes[i].word = self.words.len() as u32;
                                    let off = self.dict.len() as u32;
                                    let mut idx = job.label;
                                    while idx != 0 {
                                        let v = self.labels[idx as usize];
                                        self.dict.push(v as u8);
                                        idx = v >> 8;
                                    }
                                    self.words.push(Piece {
                                        id: off,
                                        width: self.dict.len() as u32 - off,
                                    });
                                }
                                NONE
                            } else {
                                if epoch[c] != job.id {
                                    epoch[c] = job.id;
                                    active[c] = self.nodes.len();
                                    self.nodes.push(Node::new(mapped, job.id, r.byte));
                                    let label = self.labels.len() as u32;
                                    self.labels.push(job.label << 8 | u32::from(r.byte));
                                    next[c].push(Job {
                                        lo: mapped,
                                        hi: mapped,
                                        id: active[c] as u32,
                                        label,
                                    });
                                    new_mask[c / 64] |= 1u64 << (c % 64);
                                }
                                let child = next[c].last_mut().unwrap();
                                debug_assert_eq!(child.hi, mapped);
                                child.hi += width;
                                self.nodes[active[c]].hi += width;
                                child.id
                            };
                            if self.selectors.len() > begin
                                && self.selectors.last().unwrap().id == id
                            {
                                self.selectors.last_mut().unwrap().width += width;
                            } else {
                                self.selectors.push(Piece { id, width });
                            }
                            k += 1;
                        }
                        self.nodes[i].sels = self.selectors.len() as u32 - self.nodes[i].sel;
                    }
                    current[ch].clear();
                }
            }
            mask = new_mask;
            std::mem::swap(&mut current, &mut next);
        }
        self.current = current;
        self.next = next;
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
        for p in &self.merged[self.nodes[0].out as usize..] {
            let node = &mut self.nodes[p.id as usize];
            for j in node.base..node.base + p.width {
                self.links[j as usize] = (node.word << bits) | input;
                input += 1;
            }
            node.base += p.width;
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
        let Some((quote, tokens)) = self.prepare(caps, n as u32) else {
            return false;
        };
        let Some((row, skip)) = self.transfer_mark(p, quote) else {
            return false;
        };
        self.build(quote, tokens);
        self.inverse_merge();
        #[cfg(test)]
        {
            self.stats[3] = skip as u64;
            self.stats[4] = tokens as u64;
        }
        let root = &self.merged[self.nodes[0].out as usize..];
        self.second.invert(
            root,
            &self.nodes,
            &self.words,
            &self.dict,
            row,
            skip,
            tokens,
            n as u32,
            out,
        ) || self.expand(row, skip, tokens, n as u32, out)
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
#[path = "../../f2c/checks.rs"]
mod checks;
