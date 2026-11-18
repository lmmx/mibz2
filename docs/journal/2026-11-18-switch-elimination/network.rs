//! An ordered band network. At each vertex both endpoint lists tile the same
//! integer interval. Relative list order and widths suffice for all splices.
use super::{Grammar, NONE, Pass};

const OUT: usize = 0;
const IN: usize = 1;

#[derive(Clone, Copy)]
struct Band {
    vertex: [u32; 2],
    prev: [u32; 2],
    next: [u32; 2],
    width: u32,
    word: u32,
}

#[derive(Clone, Copy)]
struct Vertex {
    first: [u32; 2],
    degree: [u32; 2],
    width: u32,
    looped: bool,
    #[cfg(test)]
    base: u32,
}

#[derive(Default)]
pub(super) struct Network {
    vertices: Vec<Vertex>,
    bands: Vec<Band>,
    free: Vec<u32>,
    // Only builders need the tails. Splices change links at known positions.
    tails: Vec<[u32; 2]>,
}

impl Network {
    pub(super) fn initial_band_count(&self) -> usize {
        self.bands.len()
    }

    fn clear(&mut self) {
        self.vertices.clear();
        self.bands.clear();
        self.free.clear();
        self.tails.clear();
    }

    fn vertex(&mut self, width: u32, _base: u32) -> u32 {
        let id = self.vertices.len() as u32;
        self.vertices.push(Vertex {
            first: [NONE; 2],
            degree: [0; 2],
            width,
            looped: false,
            #[cfg(test)]
            base: _base,
        });
        self.tails.push([NONE; 2]);
        id
    }

    fn allocate(&mut self, u: u32, v: u32, width: u32, word: u32) -> u32 {
        let band = Band {
            vertex: [u, v],
            prev: [NONE; 2],
            next: [NONE; 2],
            width,
            word,
        };
        if u == v {
            self.vertices[u as usize].looped = true;
        }
        if let Some(id) = self.free.pop() {
            self.bands[id as usize] = band;
            id
        } else {
            let id = self.bands.len() as u32;
            self.bands.push(band);
            id
        }
    }

    fn append(&mut self, u: u32, v: u32, width: u32, word: u32) {
        let id = self.allocate(u, v, width, word);
        for (side, node) in [u, v].into_iter().enumerate() {
            let k = node as usize;
            let tail = self.tails[k][side];
            if tail == NONE {
                self.vertices[k].first[side] = id;
            } else {
                self.bands[tail as usize].next[side] = id;
            }
            self.bands[id as usize].prev[side] = tail;
            self.tails[k][side] = id;
            self.vertices[k].degree[side] += 1;
        }
    }

    // Visit source runs in stable first-column order. Their images and target
    // vertices both arrive left to right; no LF or last-column array is built.
    pub(super) fn from_runs(&mut self, runs: &[(u8, u32)], p: u32) -> (u32, u32) {
        self.clear();
        let mut buckets = [0usize; 256];
        let (mut base, mut mark) = (0, (NONE, 0));
        for &(c, width) in runs {
            buckets[c as usize] += 1;
            let u = self.vertex(width, base);
            if base <= p && p < base + width {
                mark = (u, p - base);
            }
            base += width;
        }
        let mut sum = 0;
        let mut next = buckets.map(|count| {
            let at = sum;
            sum += count;
            at
        });
        let mut order = vec![0u32; runs.len()];
        for (u, &(c, _)) in runs.iter().enumerate() {
            order[next[c as usize]] = u as u32;
            next[c as usize] += 1;
        }
        let (mut target, mut used) = (0, 0);
        for u in order {
            let (c, mut left) = runs[u as usize];
            while left > 0 {
                let w = left.min(runs[target].1 - used);
                self.append(u, target as u32, w, c as u32);
                used += w;
                if used == runs[target].1 {
                    target += 1;
                    used = 0;
                }
                left -= w;
            }
        }
        assert_ne!(mark.0, NONE);
        mark
    }

    // Insert new_id immediately before old at its remote endpoint. If old is
    // exhausted, replace it there instead. No search or coordinate update.
    fn substitute(&mut self, side: usize, old: u32, new_id: u32, exhausted: bool) {
        let band = self.bands[old as usize];
        let node = &mut self.vertices[band.vertex[side] as usize];
        let prev = band.prev[side];
        if prev == NONE {
            node.first[side] = new_id;
        } else {
            self.bands[prev as usize].next[side] = new_id;
        }
        let next = if exhausted { band.next[side] } else { old };
        self.bands[new_id as usize].prev[side] = prev;
        self.bands[new_id as usize].next[side] = next;
        if next != NONE {
            self.bands[next as usize].prev[side] = new_id;
        }
        if !exhausted {
            node.degree[side] += 1;
        }
    }

    fn eliminate(&mut self, node: usize, words: &mut Grammar) {
        let [mut b, mut a] = self.vertices[node].first;
        while a != NONE {
            let incoming = self.bands[a as usize];
            let outgoing = self.bands[b as usize];
            let w = incoming.width.min(outgoing.width);
            let word = words.join(outgoing.word, incoming.word);
            let id = self.allocate(incoming.vertex[OUT], outgoing.vertex[IN], w, word);
            self.substitute(OUT, a, id, incoming.width == w);
            self.substitute(IN, b, id, outgoing.width == w);
            if w == incoming.width {
                self.free.push(a);
                a = incoming.next[IN];
            } else {
                self.bands[a as usize].width -= w;
            }
            if w == outgoing.width {
                self.free.push(b);
                b = outgoing.next[OUT];
            } else {
                self.bands[b as usize].width -= w;
            }
        }
        debug_assert_eq!(b, NONE);
        self.vertices[node].width = 0;
        self.vertices[node].degree = [0; 2];
        self.vertices[node].first = [NONE; 2];
    }

    pub(super) fn sweep(&mut self, mark: u32, grammar: &mut Grammar, budget: usize) -> Pass {
        let start = grammar.rules.len();
        let mut pass = Pass::default();
        for v in &self.vertices {
            debug_assert_ne!(v.width, 0, "compact or recode before each sweep");
            pass.before_v += 1;
            pass.before_e += v.degree[OUT] as usize;
            pass.before_m += v.width as usize;
        }
        for u in 0..self.vertices.len() {
            let v = self.vertices[u];
            if u as u32 == mark || v.looped {
                continue;
            }
            if grammar.rules.len() + (v.degree[OUT] + v.degree[IN] - 1) as usize > budget {
                pass.budget = true;
                break;
            }
            self.eliminate(u, grammar);
        }
        for v in self.vertices.iter().filter(|v| v.width != 0) {
            pass.after_v += 1;
            pass.after_e += v.degree[OUT] as usize;
            pass.after_m += v.width as usize;
            pass.loop_vertices += usize::from(v.looped);
        }
        pass.splices = grammar.rules.len() - start;
        pass
    }

    // Make each outgoing band a vertex in source-rank order. At a switch,
    // overlap the incoming and outgoing lists to give consecutive band pairs.
    pub(super) fn recode(&self, next: &mut Network, mark: (u32, u32)) -> (u32, u32) {
        next.clear();
        let mut names = vec![NONE; self.bands.len()];
        let mut new_mark = (NONE, 0);
        for (u, v) in self.vertices.iter().enumerate() {
            let mut off = 0;
            let mut id = v.first[OUT];
            while id != NONE {
                let b = self.bands[id as usize];
                #[cfg(test)]
                let base = v.base + off;
                #[cfg(not(test))]
                let base = 0;
                let name = next.vertex(b.width, base);
                names[id as usize] = name;
                if u as u32 == mark.0 && off <= mark.1 && mark.1 < off + b.width {
                    new_mark = (name, mark.1 - off);
                }
                off += b.width;
                id = b.next[OUT];
            }
        }
        for v in &self.vertices {
            let [mut b, mut a] = v.first;
            let (mut used_a, mut used_b) = (0, 0);
            while a != NONE {
                let incoming = self.bands[a as usize];
                let outgoing = self.bands[b as usize];
                let w = (incoming.width - used_a).min(outgoing.width - used_b);
                next.append(names[a as usize], names[b as usize], w, incoming.word);
                used_a += w;
                used_b += w;
                if used_a == incoming.width {
                    a = incoming.next[IN];
                    used_a = 0;
                }
                if used_b == outgoing.width {
                    b = outgoing.next[OUT];
                    used_b = 0;
                }
            }
        }
        assert_ne!(new_mark.0, NONE);
        new_mark
    }

    // Index incoming intervals of the final small return system. Binary search
    // also covers a budget stop on a large or a badly ordered system.
    pub(super) fn emit(&self, mark: (u32, u32), n: usize, words: &mut Grammar, out: &mut Vec<u8>) {
        let mut offsets = vec![0u32; self.bands.len()];
        let mut index = Vec::new();
        let mut ranges = Vec::with_capacity(self.vertices.len() + 1);
        for v in &self.vertices {
            let mut at = 0;
            let mut id = v.first[OUT];
            while id != NONE {
                let b = self.bands[id as usize];
                offsets[id as usize] = at;
                at += b.width;
                id = b.next[OUT];
            }
            ranges.push(index.len());
            at = 0;
            id = v.first[IN];
            while id != NONE {
                let b = self.bands[id as usize];
                index.push((at, id));
                at += b.width;
                id = b.next[IN];
            }
        }
        ranges.push(index.len());
        out.clear();
        out.reserve(n);
        let mut here = mark;
        while out.len() < n {
            let v = here.0 as usize;
            let incoming = &index[ranges[v]..ranges[v + 1]];
            let j = incoming.partition_point(|&(start, _)| start <= here.1) - 1;
            let (start, id) = incoming[j];
            let b = self.bands[id as usize];
            words.stats.roots += 1;
            words.write(b.word, n, out);
            here = (b.vertex[OUT], offsets[id as usize] + here.1 - start);
            if here == mark {
                while out.len() < n {
                    let take = (n - out.len()).min(out.len());
                    out.extend_from_within(..take);
                    words.stats.copied += take;
                }
                break;
            }
        }
    }
}

#[cfg(test)]
#[path = "switch_audit.rs"]
mod audit;
