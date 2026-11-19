//! A second left-extension phrase BWT over the integer alphabet of the first.
use super::{Node, Piece, NONE};

#[derive(Clone, Copy, Default)]
struct TokenRun {
    start: u32,
    lf: u32,
    word: u32,
}

#[derive(Default)]
pub(super) struct Second {
    hot_dictionary: Vec<u8>,
    #[cfg(test)]
    pub(super) hot_reads: u64,
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
    #[cfg(test)]
    pub(super) owners: [u64; 5],
    merged: Vec<Piece>,
    links: Vec<u32>,
    #[cfg(test)]
    pub(super) events: [u64; 7],
}

impl Second {
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

    fn discover<const HOT_AT: u32>(&mut self, d: u32, first_words: &[Piece]) {
        self.nodes.clear();
        self.edges.clear();
        self.selectors.clear();
        self.words.clear();
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
            let inherited = if n.parent == NONE {
                0
            } else {
                self.nodes[n.parent as usize].cursor
            };
            let added: u32 = self.edges[self.nodes[i].edge as usize..]
                .iter()
                .map(|&t| first_words[t as usize].width)
                .sum();
            self.nodes[i].cursor = inherited + added;
            let mut frequency = 0u32;
            let begin = self.selectors.len();
            self.nodes[i].sel = begin as u32;
            while self.runs[k].start < b {
                let r = self.runs[k];
                let lo = a.max(r.start);
                let width = b.min(self.runs[k + 1].start) - lo;
                let mapped = r.lf + lo - r.start;
                let id = if r.word == d {
                    frequency += width;
                    if self.nodes[i].word == NONE {
                        self.nodes[i].base = mapped - self.base[z];
                        self.nodes[i].word = self.words.len() as u32;
                        self.words.push(Piece {
                            id: i as u32,
                            width: self.nodes[i].cursor,
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
            if frequency != 0 && frequency >= HOT_AT {
                self.words[self.nodes[i].word as usize].id |= HOT;
            }
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
    pub(super) fn invert<const HOT_AT: u32>(
        &mut self,
        root: &[Piece],
        first_nodes: &[Node],
        first_words: &mut [Piece],
        edges: &[u8],
        p: u32,
        skip: u32,
        m: u32,
        n: u32,
        out: &mut Vec<u8>,
    ) -> bool {
        #[cfg(test)]
        {
            self.events = [0; 7];
            self.owners = [0; 5];
            self.hot_reads = 0;
        }
        let d = self.column(root, first_nodes, first_words.len(), m);
        let token_count = self.count[d as usize];
        if token_count == m {
            return false;
        }
        let Some((mut row, mut begin)) = self.shift(p, skip, d, first_words) else {
            return false;
        };
        self.discover::<HOT_AT>(d, first_words);
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
        self.hot_dictionary.clear();
        out.clear();
        out.reserve(n as usize);
        while out.len() < n as usize {
            let link = self.links[row as usize];
            let word = (link >> bits) as usize;
            let w = self.words[word];
            let take = (w.width - begin).min(n - out.len() as u32);
            let pos = out.len();
            if w.id & CACHED != 0 {
                let first = (w.id & !(CACHED | HOT)) + begin;
                let span = first as usize..(first + take) as usize;
                if w.id & HOT != 0 {
                    out.extend_from_slice(&self.hot_dictionary[span]);
                    #[cfg(test)]
                    {
                        self.hot_reads += take as u64;
                    }
                } else {
                    out.extend_from_within(span);
                }
                #[cfg(test)]
                {
                    self.owners[0] += take as u64;
                }
            } else {
                let mut skip = begin;
                let mut remain = take;
                let mut j = w.id & !HOT;
                while j != NONE && remain > 0 {
                    let node = self.nodes[j as usize];
                    for &sym in self.edges[node.edge as usize..node.end as usize]
                        .iter()
                        .rev()
                    {
                        let token = &mut first_words[sym as usize];
                        if skip >= token.width {
                            skip -= token.width;
                            continue;
                        }
                        let count = remain.min(token.width - skip);
                        let init = out.len();
                        if token.id & CACHED != 0 {
                            let first = (token.id & !CACHED) + skip;
                            out.extend_from_within(first as usize..(first + count) as usize);
                            #[cfg(test)]
                            {
                                self.owners[1] += count as u64;
                            }
                        } else if count == token.width {
                            super::spell(token.id, first_nodes, edges, out);
                            token.id = CACHED | init as u32;
                            #[cfg(test)]
                            {
                                self.owners[2] += count as u64;
                            }
                        } else {
                            spell_range(token.id, first_nodes, edges, skip, count, out);
                            #[cfg(test)]
                            {
                                self.owners[3] += count as u64;
                            }
                        }
                        #[cfg(test)]
                        {
                            self.events[1] += 1;
                        }
                        remain -= count;
                        skip = 0;
                        if remain == 0 {
                            break;
                        }
                    }
                    j = node.parent;
                }
                debug_assert_eq!(remain, 0);
                if take == w.width {
                    if w.id & HOT != 0 {
                        self.words[word].id = CACHED | HOT | self.hot_dictionary.len() as u32;
                        self.hot_dictionary.extend_from_slice(&out[pos..]);
                    } else {
                        self.words[word].id = CACHED | pos as u32;
                    }
                    #[cfg(test)]
                    {
                        self.owners[4] += 1;
                    }
                }
            }
            debug_assert_eq!(out.len() - pos, take as usize);
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
            self.events[4] = self.hot_dictionary.len() as u64;
            self.events[5] = self.nodes.len() as u64;
            self.events[6] = self.merged.len() as u64;
        }
        true
    }
}

const CACHED: u32 = 1 << 31;
const HOT: u32 = 1 << 30;

fn spell_range(
    mut j: u32,
    nodes: &[Node],
    edges: &[u8],
    mut skip: u32,
    mut take: u32,
    out: &mut Vec<u8>,
) {
    while j != NONE && take != 0 {
        let node = nodes[j as usize];
        let size = node.end - node.edge;
        if skip >= size {
            skip -= size;
        } else {
            let count = (size - skip).min(take);
            let end = node.end - skip;
            out.extend(edges[(end - count) as usize..end as usize].iter().rev());
            take -= count;
            skip = 0;
        }
        j = node.parent;
    }
    debug_assert_eq!(take, 0);
}
