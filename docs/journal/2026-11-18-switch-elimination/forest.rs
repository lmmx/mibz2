//! Recover a labelled open transport from its boundary by T-joins on a tree.
use super::{NONE, OutputStats};
use crate::{Kernels, bwt::InverseBwt, runs::Runs};
use std::{cmp::Reverse, collections::BTreeSet, time::Instant};

#[derive(Clone, Copy, Debug)]
struct Arc {
    u: usize,
    v: usize,
    x: u32,
    y: u32,
    w: u32,
    word: u32,
}

struct Graph {
    vertices: usize,
    arcs: Vec<Arc>,
    mark: (usize, u32),
    rows: usize,
    #[cfg(test)]
    base: Vec<u32>,
}

struct Sets {
    parent: Vec<usize>,
    size: Vec<usize>,
}
impl Sets {
    fn root(&mut self, mut u: usize) -> usize {
        while self.parent[u] != u {
            self.parent[u] = self.parent[self.parent[u]];
            u = self.parent[u];
        }
        u
    }

    fn join(&mut self, u: usize, v: usize) -> bool {
        let (mut a, mut b) = (self.root(u), self.root(v));
        if a == b {
            return false;
        }
        if self.size[a] < self.size[b] {
            std::mem::swap(&mut a, &mut b);
        }
        self.parent[b] = a;
        self.size[a] += self.size[b];
        true
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Layer {
    pub vertices: usize,
    pub arcs: usize,
    pub rows: usize,
    pub chords: usize,
    pub retained: usize,
    pub zero_gain_mass: usize,
    pub slabs: usize,
    pub pair_tests: usize,
    pub peak_pairs: usize,
    pub bands: usize,
    pub rules: usize,
}

struct Branches {
    parent: Vec<usize>,
    depth: Vec<usize>,
    edge_word: Vec<u32>,
}

struct Recipe {
    layer: usize,
    entry: usize,
    exit: usize,
    chord: u32,
    len: u32,
    first: u32,
}

#[derive(Default)]
struct Recipes {
    trees: Vec<Branches>,
    rules: Vec<Recipe>,
    stack: Vec<u32>,
    stats: OutputStats,
    children_read: usize,
}
impl Recipes {
    fn len(&self, w: u32) -> u32 {
        if w < 256 {
            1
        } else {
            self.rules[(w - 256) as usize].len
        }
    }
    fn clear(&mut self) {
        self.trees.clear();
        self.rules.clear();
        self.stack.clear();
        self.stats = OutputStats::default();
        self.children_read = 0;
    }
    fn record(&mut self, t: &Tree, entry: usize, exit: usize, chord: u32) -> u32 {
        if entry == exit {
            return chord;
        }
        let len = i64::from(self.len(chord)) + t.time[exit] - t.time[entry];
        debug_assert!(len > 1);
        let id = 256 + self.rules.len() as u32;
        self.rules.push(Recipe {
            layer: self.trees.len(),
            entry,
            exit,
            chord,
            len: len as u32,
            first: NONE,
        });
        id
    }
    // Children are requested only at the first marked occurrence of this word.
    // Push in LF order: the interpreter pops in forward output order.
    fn write(&mut self, root: u32, n: usize, out: &mut Vec<u8>) {
        self.stack.clear();
        self.stack.push(root);
        while let Some(w) = self.stack.pop() {
            if out.len() == n {
                break;
            }
            if w < 256 {
                out.push(w as u8);
                self.stats.literals += 1;
                continue;
            }
            let r = &mut self.rules[(w - 256) as usize];
            if r.first != NONE {
                let k = (n - out.len()).min(r.len as usize);
                let a = r.first as usize;
                out.extend_from_within(a..a + k);
                self.stats.copied += k;
                self.stats.hits += 1;
                continue;
            }
            self.stats.opened += 1;
            r.first = out.len() as u32;
            let t = &self.trees[r.layer];
            let (mut a, mut b) = (r.entry, r.exit);
            let base = self.stack.len();
            self.stack.push(r.chord);
            while t.depth[a] > t.depth[b] {
                self.stack.push(t.edge_word[a]);
                a = t.parent[a];
            }
            // The exit half is encountered backwards. A separate stack is not
            // needed: find the LCA, then reverse just the exit suffix.
            let mut meet_a = a;
            let mut meet_b = b;
            while t.depth[meet_b] > t.depth[meet_a] {
                meet_b = t.parent[meet_b];
            }
            while meet_a != meet_b {
                meet_a = t.parent[meet_a];
                meet_b = t.parent[meet_b];
            }
            while a != meet_a {
                self.stack.push(t.edge_word[a]);
                a = t.parent[a];
            }
            let turn = self.stack.len();
            while b != meet_a {
                self.stack.push(t.edge_word[b]);
                b = t.parent[b];
            }
            self.stack[turn..].reverse();
            self.children_read += self.stack.len() - base;
        }
    }
}

struct Tree {
    comp: Vec<usize>,
    depth: Vec<usize>,
    potential: Vec<i64>,
    time: Vec<i64>,
    parent: Vec<usize>,
    parent_word: Vec<u32>,
    // A noncrossing topological order. All ports of a site are placed between
    // the children's incoming and outgoing subtrees, with openings first.
    position: Vec<usize>,
}
impl Tree {
    fn new(adj: &[Vec<(usize, i64, i64, u32)>]) -> Self {
        let n = adj.len();
        let mut t = Self {
            comp: vec![usize::MAX; n],
            depth: vec![0; n],
            potential: vec![0; n],
            time: vec![0; n],
            parent: (0..n).collect(),
            parent_word: vec![NONE; n],
            position: vec![0; n],
        };
        let mut timer = 0;
        for root in 0..n {
            if t.comp[root] != usize::MAX {
                continue;
            }
            t.comp[root] = root;
            let mut stack = vec![(root, false)];
            while let Some((u, visit)) = stack.pop() {
                if visit {
                    t.position[u] = timer;
                    timer += 1;
                    continue;
                }
                // The stack pops incoming subtrees, then the site, then outgoing
                // subtrees. The positive time direction is the arrow direction.
                for incoming in [false, true] {
                    if incoming {
                        stack.push((u, true));
                    }
                    for &(v, shift, dt, w) in &adj[u] {
                        if v == t.parent[u] || (dt < 0) != incoming {
                            continue;
                        }
                        t.comp[v] = root;
                        t.parent[v] = u;
                        t.parent_word[v] = w;
                        t.depth[v] = t.depth[u] + 1;
                        t.potential[v] = t.potential[u] + shift;
                        t.time[v] = t.time[u] + dt;
                        stack.push((v, false));
                    }
                }
            }
        }
        t
    }
}

impl Graph {
    fn initial(runs: &[(u8, u32)], p: u32) -> Self {
        let mut starts = vec![0];
        let mut totals = [0; 256];
        for &(c, w) in runs {
            starts.push(starts.last().unwrap() + w);
            totals[c as usize] += w;
        }
        let mut sum = 0;
        let mut next = totals.map(|w| {
            let y = sum;
            sum += w;
            y
        });
        let mut arcs = Vec::with_capacity(2 * runs.len());
        for (u, &(c, w)) in runs.iter().enumerate() {
            let y = next[c as usize];
            next[c as usize] += w;
            let mut v = starts.partition_point(|&x| x <= y) - 1;
            let mut x = 0;
            while x < w {
                let width = (w - x).min(starts[v + 1] - (y + x));
                arcs.push(Arc {
                    u,
                    v,
                    x,
                    y: y + x - starts[v],
                    w: width,
                    word: c as u32,
                });
                x += width;
                v += 1;
            }
        }
        let u = starts.partition_point(|&x| x <= p) - 1;
        Self {
            vertices: runs.len(),
            arcs,
            mark: (u, p - starts[u]),
            rows: sum as usize,
            #[cfg(test)]
            base: starts[..runs.len()].to_vec(),
        }
    }

    fn reduce(&self, grammar: &mut Recipes, limit: usize, s: &mut Layer) -> Option<Self> {
        let (n, e) = (self.vertices, &self.arcs);
        let mut sets = Sets {
            parent: (0..n).collect(),
            size: vec![1; n],
        };
        let mut order: Vec<_> = (0..e.len()).collect();
        order.sort_unstable_by_key(|&i| (Reverse(e[i].w), i));
        let mut adj = vec![Vec::new(); n];
        let mut chords = Vec::new();
        let mut marked = usize::MAX;
        for i in order {
            let b = e[i];
            let cut = b.u == self.mark.0 && b.x <= self.mark.1 && self.mark.1 < b.x + b.w;
            if cut {
                marked = i;
            }
            if cut || !sets.join(b.u, b.v) {
                chords.push(i);
            } else {
                let d = i64::from(b.x) - i64::from(b.y);
                let dt = i64::from(grammar.len(b.word));
                adj[b.u].push((b.v, d, dt, b.word));
                adj[b.v].push((b.u, -d, -dt, b.word));
            }
        }
        if chords.len() == e.len() {
            return None;
        }
        chords.sort_unstable();
        let new_mark = (
            chords.binary_search(&marked).unwrap(),
            self.mark.1 - e[marked].x,
        );
        let t = Tree::new(&adj);
        drop(adj);
        let mut events = Vec::with_capacity(4 * chords.len());
        let (mut host, mut lower) = (Vec::new(), Vec::new());
        for (j, &i) in chords.iter().enumerate() {
            let b = e[i];
            s.retained += b.w as usize;
            if t.comp[b.u] == t.comp[b.v]
                && t.potential[b.u] + i64::from(b.x) == t.potential[b.v] + i64::from(b.y)
            {
                s.zero_gain_mass += b.w as usize;
            }
            for (side, (u, k)) in [(b.u, b.x), (b.v, b.y)].into_iter().enumerate() {
                let z = t.potential[u] + i64::from(k);
                host.push(u);
                lower.push(z);
                events.push((t.comp[u], z, 2 * j + side));
                events.push((t.comp[u], z + i64::from(b.w), 2 * j + side));
            }
        }
        s.vertices = n;
        s.arcs = e.len();
        s.rows = self.rows;
        s.chords = chords.len();
        events.sort_unstable();
        let mut by: Vec<Vec<Arc>> = vec![Vec::new(); chords.len()];
        let mut active = BTreeSet::new();
        let mut stack = Vec::new();
        let mut i = 0;
        while i < events.len() {
            let (comp, z, _) = events[i];
            while i < events.len() && (events[i].0, events[i].1) == (comp, z) {
                let port = events[i].2;
                let key = (t.position[host[port]], port % 2 == 0, port);
                if !active.remove(&key) {
                    active.insert(key);
                }
                i += 1;
            }
            if active.is_empty() {
                continue;
            }
            assert!(i < events.len() && events[i].0 == comp);
            let w = (events[i].1 - z) as u32;
            let k = active.len() / 2;
            s.slabs += 1;
            s.pair_tests += k;
            s.peak_pairs = s.peak_pairs.max(k);
            if s.pair_tests > limit {
                return None;
            }
            stack.clear();
            for &(_, closing, b) in &active {
                if !closing {
                    stack.push(b);
                    continue;
                }
                let a = stack.pop().expect("a projected fibre is a Dyck word");
                let (u, v) = (a / 2, b / 2);
                let (x, y) = ((z - lower[a]) as u32, (z - lower[b]) as u32);
                if let Some(last) = by[u].last_mut() {
                    if last.v == v && last.x + last.w == x && last.y + last.w == y {
                        last.w += w;
                        continue;
                    }
                }
                by[u].push(Arc {
                    u,
                    v,
                    x,
                    y,
                    w,
                    word: NONE,
                });
            }
            assert!(stack.is_empty());
        }
        let before = grammar.rules.len();
        let mut arcs: Vec<Arc> = by.into_iter().flatten().collect();
        for a in &mut arcs {
            let first = e[chords[a.u]];
            let last = e[chords[a.v]];
            a.word = grammar.record(&t, first.v, last.u, first.word);
        }
        s.bands = arcs.len();
        s.rules = grammar.rules.len() - before;
        grammar.trees.push(Branches {
            parent: t.parent,
            depth: t.depth,
            edge_word: t.parent_word,
        });
        Some(Self {
            vertices: chords.len(),
            arcs,
            mark: new_mark,
            rows: s.retained,
            #[cfg(test)]
            base: chords.iter().map(|&i| self.base[e[i].u] + e[i].x).collect(),
        })
    }

    fn emit(&self, n: usize, words: &mut Recipes, out: &mut Vec<u8>) {
        let mut incoming = vec![Vec::new(); self.vertices];
        for &b in &self.arcs {
            incoming[b.v].push(b);
        }
        for list in &mut incoming {
            list.sort_unstable_by_key(|b| b.y);
        }
        out.clear();
        out.reserve(n);
        let mut q = self.mark;
        while out.len() < n {
            let list = &incoming[q.0];
            let a = list[list.partition_point(|a| a.y <= q.1) - 1];
            words.stats.roots += 1;
            words.write(a.word, n, out);
            q = (a.u, a.x + q.1 - a.y);
            if q == self.mark {
                while out.len() < n {
                    let take = out.len().min(n - out.len());
                    out.extend_from_within(..take);
                    words.stats.copied += take;
                }
                break;
            }
        }
    }
}

#[derive(Default)]
pub struct Forest {
    runs: Vec<(u8, u32)>,
    grammar: Recipes,
    pub layers: Vec<Layer>,
    pub budget_stop: bool,
    pub build_seconds: f64,
    pub output_seconds: f64,
}

impl Forest {
    pub fn stats(&self) -> &OutputStats {
        &self.grammar.stats
    }
    pub fn children_read(&self) -> usize {
        self.grammar.children_read
    }
    fn add(&mut self, c: u8, w: u32) {
        if let Some(last) = self.runs.last_mut() {
            if last.0 == c {
                last.1 += w;
                return;
            }
        }
        self.runs.push((c, w));
    }
    fn decode(&mut self, n: usize, p: u32, out: &mut Vec<u8>, begin: Instant) {
        self.layers.clear();
        self.grammar.clear();
        self.budget_stop = false;
        let mut g = Graph::initial(&self.runs, p);
        let mut allowance = 4 * n;
        while g.rows > 1 {
            let mut s = Layer::default();
            match g.reduce(&mut self.grammar, allowance, &mut s) {
                Some(next) => {
                    allowance -= s.pair_tests;
                    self.layers.push(s);
                    g = next;
                }
                None => {
                    self.budget_stop = s.pair_tests > allowance;
                    break;
                }
            }
        }
        let built = Instant::now();
        g.emit(n, &mut self.grammar, out);
        self.build_seconds = (built - begin).as_secs_f64();
        self.output_seconds = built.elapsed().as_secs_f64();
    }
}
impl InverseBwt for Forest {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, _: Kernels) -> bool {
        let start = Instant::now();
        self.runs.clear();
        let mut size = 0;
        for &rec in r.records() {
            let w = u32::from(rec as u8);
            if w == 0 {
                return false;
            }
            self.add((rec >> 8) as u8, w);
            size += w as usize;
        }
        if size != n || n == 0 {
            return false;
        }
        self.decode(n, p, out, start);
        true
    }
    fn invert(&mut self, l: &[u8], _: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        let start = Instant::now();
        self.runs.clear();
        for &c in l {
            self.add(c, 1);
        }
        if l.is_empty() {
            out.clear();
            return;
        }
        self.decode(l.len(), p, out, start);
    }
}
#[cfg(test)]
#[path = "forest_audit.rs"]
mod audit;
