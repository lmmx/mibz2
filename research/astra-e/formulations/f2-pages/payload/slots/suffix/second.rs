//! A second left-extension phrase BWT over the integer alphabet of the first.
use super::{Node, Piece, NONE};

#[derive(Clone, Copy, Default)]
struct TokenRun {
    start: u32,
    lf: u32,
    word: u32,
}

#[derive(Default)]
pub(super) struct Second<const SPELL:u8> {
    #[cfg(spell_counts)]
    pub(super) copied: [u64;4],
    runs: Vec<TokenRun>,
    count: Vec<u32>,
    base: Vec<u32>,
    epoch: Vec<u32>,
    child: Vec<usize>,
    page: Vec<u32>,
    masks: Vec<u64>,
    nodes: Vec<Node>,
    edges: Vec<u32>,
    selectors: Vec<Piece>,
    words: Vec<Piece>,
    dictionary: Vec<u8>,
    merged: Vec<Piece>,
    links: Vec<u32>,
    #[cfg(test)]
    pub(super) events: [u64; 7],
}

impl<const SPELL:u8> Second<SPELL> {
    fn reset<T: Clone>(v: &mut Vec<T>, n: usize, initial: T) {
        v.clear();
        v.resize(n, initial);
    }

    fn column(&mut self, root: &[Piece], first_nodes: &[Node], alphabet: usize, m: u32) -> u32 {
        Self::reset(&mut self.count, alphabet, 0);
        Self::reset(&mut self.base, alphabet, 0);
        self.runs.clear();
        let mut row = 0;
        for p in root {
            let node = first_nodes[p.id as usize];
            let word = node.word as usize;
            self.base[word] = node.base;
            self.runs.push(TokenRun {
                start: row,
                lf: node.base + self.count[word],
                word: node.word,
            });
            self.count[word] += p.width;
            row += p.width;
        }
        debug_assert_eq!(row, m);
        let pages = m.div_ceil(64) as usize;
        Self::reset(&mut self.page, pages, 0);
        Self::reset(&mut self.masks, pages, 0);
        for r in &self.runs {
            let offset = r.start & 63;
            if offset != 0 {
                self.masks[(r.start >> 6) as usize] |= 1u64 << offset;
            }
        }
        self.runs.push(TokenRun {
            start: m,
            lf: 0,
            word: 0,
        });
        let mut k = 0;
        for (page, idx) in self.page.iter_mut().enumerate() {
            while self.runs[k + 1].start <= page as u32 * 64 {
                k += 1;
            }
            *idx = k as u32;
        }
        // The most frequent token; in a tie choose its first-column order.
        (0..alphabet)
            .max_by_key(|&i| (self.count[i], u32::MAX - self.base[i]))
            .unwrap() as u32
    }

    #[inline]
    fn find(&self, x: u32) -> usize {
        let page = (x >> 6) as usize;
        (self.page[page] + (self.masks[page] & (u64::MAX >> (63 - (x & 63)))).count_ones()) as usize
    }

    fn shift(&self, p: u32, mut skip: u32, d: u32, first_words: &[Piece]) -> Option<(u32, u32)> {
        let mut x = p;
        loop {
            let r = self.runs[self.find(x)];
            x = r.lf + x - r.start;
            if r.word == d {
                return Some((x - self.base[d as usize], skip));
            }
            skip += first_words[r.word as usize].width;
            if x == p {
                return None;
            }
        }
    }

    fn fresh_node(lo: u32, parent: u32, initial: u32) -> Node {
        let mut node = Node::new(lo, parent, 0);
        // Before discovery edge holds the first level-one token on this edge;
        // afterwards it holds the beginning of its full contracted spelling.
        node.edge = initial;
        node
    }

    fn discover(&mut self, d: u32, first_words: &[Piece], bytes: &[u8]) {
        self.nodes.clear();
        self.edges.clear();
        self.selectors.clear();
        self.words.clear();
        self.dictionary.clear();
        Self::reset(&mut self.epoch, first_words.len(), NONE);
        self.child.resize(first_words.len(), 0);
        let z = d as usize;
        let mut root = Self::fresh_node(self.base[z], NONE, d);
        root.hi += self.count[z];
        self.nodes.push(root);
        let mut i = 0;
        while i < self.nodes.len() {
            let n = self.nodes[i];
            let (mut a, mut b) = (n.lo, n.hi);
            self.nodes[i].edge = self.edges.len() as u32;
            self.edges.push(n.edge);
            let mut k;
            loop {
                k = self.find(a);
                #[cfg(test)]
                {
                    self.events[0] += 1;
                }
                let r = self.runs[k];
                if r.word == d || self.runs[k + 1].start < b {
                    break;
                }
                self.edges.push(r.word);
                let width = b - a;
                a = r.lf + a - r.start;
                b = a + width;
            }
            self.nodes[i].end = self.edges.len() as u32;
            let begin = self.selectors.len();
            self.nodes[i].sel = begin as u32;
            while self.runs[k].start < b {
                let r = self.runs[k];
                let lo = a.max(r.start);
                let width = b.min(self.runs[k + 1].start) - lo;
                let mapped = r.lf + lo - r.start;
                let id = if r.word == d {
                    if self.nodes[i].word == NONE {
                        self.nodes[i].base = mapped - self.base[z];
                        self.nodes[i].word = self.words.len() as u32;
                        let pos = self.dictionary.len() as u32;
                        let mut j = i as u32;
                        while j != NONE {
                            let node = self.nodes[j as usize];
                            if SPELL & 2 != 0 && (node.word as usize) < self.words.len() {
                                let spelled = self.words[node.word as usize];
                                self.dictionary.extend_from_within(spelled.id as usize..(spelled.id + spelled.width) as usize);
                                #[cfg(spell_counts)]
                                {
                                    self.copied[2] += 1;
                                    self.copied[3] += u64::from(spelled.width);
                                }
                                break;
                            }
                            for &sym in self.edges[node.edge as usize..node.end as usize]
                                .iter()
                                .rev()
                            {
                                let w = first_words[sym as usize];
                                #[cfg(spell_counts)]
                                {
                                    self.copied[0] += 1;
                                    self.copied[1] += u64::from(w.width);
                                }
                                self.dictionary.extend_from_slice(
                                    &bytes[w.id as usize..(w.id + w.width) as usize],
                                );
                                #[cfg(test)]
                                {
                                    self.events[1] += 1;
                                }
                            }
                            j = node.parent;
                        }
                        self.words.push(Piece {
                            id: pos,
                            width: self.dictionary.len() as u32 - pos,
                        });
                    }
                    NONE
                } else {
                    let c = r.word as usize;
                    if self.epoch[c] != i as u32 {
                        self.epoch[c] = i as u32;
                        self.child[c] = self.nodes.len();
                        self.nodes.push(Self::fresh_node(mapped, i as u32, r.word));
                    }
                    let target = self.child[c];
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
            self.nodes[i].sels = self.selectors.len() as u32 - begin as u32;
            i += 1;
        }
    }

    fn merge(&mut self) {
        self.merged.clear();
        for i in (0..self.nodes.len()).rev() {
            let n = self.nodes[i];
            self.nodes[i].out = self.merged.len() as u32;
            self.nodes[i].cursor = self.merged.len() as u32;
            for &s in &self.selectors[n.sel as usize..(n.sel + n.sels) as usize] {
                if s.id == NONE {
                    self.merged.push(Piece {
                        id: i as u32,
                        width: s.width,
                    });
                } else {
                    let child = &mut self.nodes[s.id as usize];
                    let mut left = s.width;
                    while left > 0 {
                        let piece = self.merged[child.cursor as usize];
                        let take = left.min(piece.width - child.used);
                        self.merged.push(Piece {
                            id: piece.id,
                            width: take,
                        });
                        left -= take;
                        child.used += take;
                        if child.used == piece.width {
                            child.used = 0;
                            child.cursor += 1;
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn invert(
        &mut self,
        root: &[Piece],
        first_nodes: &[Node],
        first_words: &[Piece],
        bytes: &[u8],
        p: u32,
        skip: u32,
        m: u32,
        n: u32,
        out: &mut Vec<u8>,
    ) -> bool {
        #[cfg(test)]
        {
            self.events = [0; 7];
        }
        let d = self.column(root, first_nodes, first_words.len(), m);
        let token_count = self.count[d as usize];
        if token_count == m {
            return false;
        }
        let Some((mut row, mut begin)) = self.shift(p, skip, d, first_words) else {
            return false;
        };
        self.discover(d, first_words, bytes);
        self.merge();
        let bits = 32 - (token_count - 1).leading_zeros();
        if self.words.len() - 1 > (u32::MAX >> bits) as usize {
            return false;
        }
        self.links.resize(token_count as usize, 0);
        let mut at = 0;
        for piece in &self.merged[self.nodes[0].out as usize..] {
            let node = &mut self.nodes[piece.id as usize];
            let end = node.base + piece.width;
            for slot in &mut self.links[node.base as usize..end as usize] {
                *slot = (node.word << bits) | at;
                at += 1;
            }
            node.base = end;
        }
        debug_assert_eq!(at, token_count);
        let mask = (1u32 << bits) - 1;
        out.clear();
        out.reserve(n as usize);
        while out.len() < n as usize {
            let link = self.links[row as usize];
            let w = self.words[(link >> bits) as usize];
            let take = (w.width - begin).min(n - out.len() as u32);
            let a = (w.id + begin) as usize;
            out.extend_from_slice(&self.dictionary[a..a + take as usize]);
            begin = 0;
            row = link & mask;
            #[cfg(test)]
            {
                self.events[2] += 1;
            }
        }
        #[cfg(test)]
        {
            self.events[3] = token_count as u64;
            self.events[4] = self.dictionary.len() as u64;
            self.events[5] = self.nodes.len() as u64;
            self.events[6] = self.merged.len() as u64;
        }
        true
    }
}
