//! Compressed list contraction at rank-local minima of an interval exchange.
use crate::bwt::{InverseBwt, Segmented};
use crate::{runs::Runs, Kernels};

#[derive(Clone, Copy, Debug)]
struct Atom {
    raw: u32,
    seam: bool,
    x: u32,
    y: u32,
    width: u32,
    word: u32,
}

#[derive(Clone, Copy)]
struct Rule {
    left: u32,
    right: u32,
    len: u32,
}

#[derive(Default)]
struct Words(Vec<Rule>);

impl Words {
    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            self.0[(w - 257) as usize].len
        }
    }
    fn join(&mut self, a: u32, b: u32) -> u32 {
        if a == 0 {
            return b;
        }
        if b == 0 {
            return a;
        }
        let len = self.len(a) + self.len(b);
        let id = 257 + self.0.len() as u32;
        self.0.push(Rule {
            left: a,
            right: b,
            len,
        });
        id
    }
    // A repeat has a single child. Its output length fixes the exponent.
    fn power(&mut self, w: u32, h: u32) -> u32 {
        if h == 1 {
            return w;
        }
        let len = self.len(w) * h;
        let id = 257 + self.0.len() as u32;
        self.0.push(Rule {
            left: w,
            right: 0,
            len,
        });
        id
    }
    fn write(&self, root: u32, n: usize, out: &mut Vec<u8>) -> usize {
        enum Task {
            Word(u32),
            Repeat { start: usize, end: usize },
        }
        let mut seen = vec![u32::MAX; self.0.len()];
        let mut stack = vec![Task::Word(root)];
        out.clear();
        out.reserve(n);
        let mut used = 0;
        while let Some(task) = stack.pop() {
            let w = match task {
                Task::Word(w) => w,
                Task::Repeat { start, end } => {
                    while out.len() < end {
                        let add = (out.len() - start).min(end - out.len());
                        out.extend_from_within(start..start + add);
                    }
                    continue;
                }
            };
            if w <= 256 {
                out.push((w - 1) as u8);
            } else {
                let i = (w - 257) as usize;
                let r = self.0[i];
                if seen[i] == u32::MAX {
                    let start = out.len();
                    seen[i] = start as u32;
                    used += 1;
                    if r.right != 0 {
                        stack.push(Task::Word(r.right));
                    } else {
                        stack.push(Task::Repeat {
                            start,
                            end: start + r.len as usize,
                        });
                    }
                    stack.push(Task::Word(r.left));
                } else {
                    let pos = seen[i] as usize;
                    out.extend_from_within(pos..pos + r.len as usize);
                }
            }
        }
        while out.len() < n {
            out.extend_from_within(..out.len().min(n - out.len()));
        }
        used
    }
}

// AVL ropes. Each leaf carries consecutive base ranks and their consecutive
// original ranks. Joins and splits touch logarithmically many nodes.
#[derive(Clone, Copy, Default)]
struct Node {
    left: usize,
    right: usize,
    height: u32,
    total: u32,
    origin: u32,
    raw: u32,
    tag: u32,
    seam: bool,
}

struct Ropes {
    nodes: Vec<Node>,
    free: Vec<usize>,
}

impl Ropes {
    fn new() -> Self {
        Self {
            nodes: vec![Node::default()],
            free: Vec::new(),
        }
    }
    fn alloc(&mut self, node: Node) -> usize {
        if let Some(id) = self.free.pop() {
            self.nodes[id] = node;
            return id;
        }
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }
    fn leaf(&mut self, origin: u32, raw: u32, width: u32, tag: u32, seam: bool) -> usize {
        self.alloc(Node {
            height: 1,
            total: width,
            origin,
            raw,
            tag,
            seam,
            ..Node::default()
        })
    }
    fn fork(&mut self, a: usize, b: usize) -> usize {
        let (left, right) = (self.nodes[a], self.nodes[b]);
        self.alloc(Node {
            left: a,
            right: b,
            height: 1 + left.height.max(right.height),
            total: left.total + right.total,
            ..Node::default()
        })
    }
    fn apply(&mut self, t: usize, w: u32, words: &mut Words) {
        self.nodes[t].tag = words.join(w, self.nodes[t].tag);
    }
    fn parts(&mut self, t: usize, words: &mut Words) -> (usize, usize) {
        let a = self.nodes[t];
        debug_assert_ne!(a.left, 0);
        if a.tag != 0 {
            self.apply(a.left, a.tag, words);
            self.apply(a.right, a.tag, words);
        }
        self.free.push(t);
        (a.left, a.right)
    }
    fn rebalance(&mut self, l: usize, r: usize, words: &mut Words) -> usize {
        let (hl, hr) = (self.nodes[l].height, self.nodes[r].height);
        if hl > hr + 1 {
            let (a, b) = self.parts(l, words);
            if self.nodes[b].height > self.nodes[a].height {
                let (c, d) = self.parts(b, words);
                let left = self.fork(a, c);
                let right = self.fork(d, r);
                self.fork(left, right)
            } else {
                let right = self.fork(b, r);
                self.fork(a, right)
            }
        } else if hr > hl + 1 {
            let (a, b) = self.parts(r, words);
            if self.nodes[a].height > self.nodes[b].height {
                let (c, d) = self.parts(a, words);
                let left = self.fork(l, c);
                let right = self.fork(d, b);
                self.fork(left, right)
            } else {
                let left = self.fork(l, a);
                self.fork(left, b)
            }
        } else {
            self.fork(l, r)
        }
    }
    fn merge(&mut self, l: usize, r: usize, words: &mut Words) -> usize {
        if l == 0 || r == 0 {
            return l | r;
        }
        if self.nodes[l].height > self.nodes[r].height + 1 {
            let (a, b) = self.parts(l, words);
            let right = self.merge(b, r, words);
            self.rebalance(a, right, words)
        } else if self.nodes[r].height > self.nodes[l].height + 1 {
            let (a, b) = self.parts(r, words);
            let left = self.merge(l, a, words);
            self.rebalance(left, b, words)
        } else {
            self.fork(l, r)
        }
    }
    fn split(&mut self, t: usize, count: u32, words: &mut Words) -> (usize, usize) {
        let a = self.nodes[t];
        if count == 0 {
            return (0, t);
        }
        if count == a.total {
            return (t, 0);
        }
        if a.left == 0 {
            self.nodes[t].total = count;
            let right = self.leaf(
                a.origin + count,
                a.raw + count,
                a.total - count,
                a.tag,
                false,
            );
            return (t, right);
        }
        let (l, r) = self.parts(t, words);
        let k = self.nodes[l].total;
        if count == k {
            return (l, r);
        }
        if count < k {
            let (prefix, rest) = self.split(l, count, words);
            (prefix, self.merge(rest, r, words))
        } else {
            let (rest, suffix) = self.split(r, count - k, words);
            (self.merge(l, rest, words), suffix)
        }
    }
    fn mark_first(&mut self, mut t: usize) {
        while self.nodes[t].left != 0 {
            t = self.nodes[t].left;
        }
        self.nodes[t].seam = true;
    }
    fn finish(&mut self, t: usize, y: &mut u32, words: &mut Words, result: &mut Vec<Atom>) {
        let a = self.nodes[t];
        if a.left != 0 {
            let (l, r) = self.parts(t, words);
            self.finish(l, y, words, result);
            self.finish(r, y, words, result);
        } else {
            result.push(Atom {
                x: a.origin,
                raw: a.raw,
                seam: a.seam,
                y: *y,
                width: a.total,
                word: a.tag,
            });
            *y += a.total;
            self.free.push(t);
        }
    }
}

#[derive(Clone, Copy)]
struct Port {
    seam: bool,
    y: u32,
    width: u32,
    word: u32,
    dest: usize,
    cut: bool,
    base: u32,
    rope: usize,
}

fn col(records: impl Iterator<Item = (u8, u32)> + Clone, n: u32, p: u32) -> Option<Vec<Atom>> {
    let mut counts = [0u32; 256];
    let mut runs = 0;
    let mut previous = None;
    for (byte, width) in records.clone() {
        counts[usize::from(byte)] += width;
        runs += usize::from(Some(byte) != previous);
        previous = Some(byte);
    }
    let mut total = 0;
    for slot in &mut counts {
        let w = *slot;
        *slot = total;
        total += w;
    }
    if total != n || p >= n {
        return None;
    }
    // Conjugate by x -> (x-p) mod n. The primary is now the least rank
    // in every reduced exchange. Split both tilings where the order wraps.
    let rotate = |t: u32| if t >= p { t - p } else { n - p + t };
    let mut out: Vec<Atom> = Vec::with_capacity(runs + 2);
    let mut pos = 0;
    let mut records = records.peekable();
    while let Some((a, mut width)) = records.next() {
        while let Some((_, more)) = records.next_if(|&(byte, _)| byte == a) {
            width += more;
        }
        let dest = &mut counts[a as usize];
        let (mut x, mut y) = (rotate(pos), rotate(*dest));
        pos += width;
        *dest += width;
        while width != 0 {
            let w = width.min(n - x).min(n - y);
            out.push(Atom {
                x,
                y,
                raw: x,
                seam: true,
                width: w,
                word: u32::from(a) + 1,
            });
            x += w;
            y += w;
            width -= w;
            if x == n {
                x = 0;
            }
            if y == n {
                y = 0;
            }
        }
    }
    let pivot = out.iter().position(|a| a.x == 0).unwrap();
    out.rotate_left(pivot);
    Some(out)
}

fn ports(atoms: &[Atom], words: &mut Words) -> (Vec<Port>, Vec<usize>) {
    let mut ports = Vec::with_capacity(2 * atoms.len());
    let mut first = Vec::with_capacity(atoms.len() + 1);
    for a in atoms {
        first.push(ports.len());
        if a.x == a.y {
            continue;
        }
        let up = a.y > a.x;
        let step = a.y.abs_diff(a.x);
        let size = step.min(a.width);
        let end = a.x + a.width;
        let mut x = if up { a.x } else { end - size };
        let finish = x + size;
        while x < finish {
            // Entering a band, at most two different multiplicities occur.
            let (h, y, w) = if up {
                let h = (end - x).div_ceil(step);
                let y = (u64::from(x) + u64::from(h) * u64::from(step)) as u32;
                (h, y, (finish - x).min(end + step - y))
            } else {
                let h = (x - a.x) / step + 1;
                let y = x - h * step;
                (h, y, (finish - x).min(a.x - y))
            };
            let tag = words.power(a.word, h);
            let mut y = y;
            let mut source = x;
            let mut dest = atoms.partition_point(|b| b.x + b.width <= y);
            let until = y + w;
            while y < until {
                let b = atoms[dest];
                let width = (b.x + b.width).min(until) - y;
                let least_source = if up { source } else { source - (h - 1) * step };
                ports.push(Port {
                    seam: a.seam && least_source == a.x,
                    y,
                    width,
                    word: tag,
                    dest,
                    cut: !up && b.y > b.x,
                    base: 0,
                    rope: 0,
                });
                y += width;
                source += width;
                dest += 1;
            }
            x += w;
        }
    }
    first.push(ports.len());
    (ports, first)
}

#[derive(Debug, Default)]
pub struct Round {
    pub rows: u32,
    pub atoms: usize,
    pub ports: usize,
    pub seeds: usize,
    pub minima: u32,
    pub leaves: usize,
    pub rules: usize,
}

#[derive(Debug, Default)]
pub struct Stats {
    pub rounds: Vec<Round>,
    pub period: u32,
    pub rules: usize,
    pub used: usize,
}

fn contract(atoms: &[Atom], words: &mut Words) -> (Vec<Atom>, Round) {
    let old_rules = words.0.len();
    let (mut ports, first) = ports(atoms, words);
    let mut by_image: Vec<usize> = (0..ports.len()).collect();
    by_image.sort_unstable_by_key(|&i| ports[i].y);
    let mut incoming = vec![0usize; atoms.len() + 1];
    let mut ropes = Ropes::new();
    let mut minima = 0;
    let mut seeds = 0;
    for &i in &by_image {
        let arc = &mut ports[i];
        incoming[arc.dest + 1] += 1;
        if arc.cut {
            arc.base = minima;
            let target = atoms[arc.dest];
            arc.rope = ropes.leaf(minima, target.raw + (arc.y - target.x), arc.width, 0, false);
            minima += arc.width;
            seeds += 1;
        }
    }
    for i in 0..atoms.len() {
        incoming[i + 1] += incoming[i];
    }

    let mut out = Vec::with_capacity(ports.len());
    let plus = (0..atoms.len()).filter(|&i| atoms[i].y > atoms[i].x);
    let minus = (0..atoms.len()).rev().filter(|&i| atoms[i].y < atoms[i].x);
    for i in plus.chain(minus) {
        let mut all = 0;
        for &j in &by_image[incoming[i]..incoming[i + 1]] {
            all = ropes.merge(all, ports[j].rope, words);
        }
        for arc in &mut ports[first[i]..first[i + 1]] {
            let (part, rest) = ropes.split(all, arc.width, words);
            all = rest;
            ropes.apply(part, arc.word, words);
            if arc.seam {
                ropes.mark_first(part);
            }
            if arc.cut {
                let mut y = arc.base;
                ropes.finish(part, &mut y, words, &mut out);
                debug_assert_eq!(y, arc.base + arc.width);
            } else {
                arc.rope = part;
            }
        }
        debug_assert_eq!(all, 0);
    }
    let leaves = out.len();
    out.sort_unstable_by_key(|a| a.x);
    let mut m = 0;
    for i in 0..out.len() {
        let a = out[i];
        if m != 0
            && !a.seam
            && out[m - 1].raw + out[m - 1].width == a.raw
            && out[m - 1].y + out[m - 1].width == a.y
            && words.len(out[m - 1].word) == words.len(a.word)
        {
            out[m - 1].width += a.width;
        } else {
            out[m] = a;
            m += 1;
        }
    }
    out.truncate(m);
    let last = atoms.last().unwrap();
    let stat = Round {
        rows: last.x + last.width,
        atoms: atoms.len(),
        ports: ports.len(),
        seeds,
        minima,
        leaves,
        rules: words.0.len() - old_rules,
    };
    (out, stat)
}

#[derive(Default)]
pub struct Extrema {
    fallback: Segmented,
}

impl Extrema {
    pub fn records(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> Option<Stats> {
        let runs = records
            .iter()
            .map(|&v| ((v >> 8) as u8, u32::from(v & 255)));
        let atoms = col(runs, n as u32, p)?;
        Some(Self::expand(atoms, n, out))
    }
    fn expand(mut atoms: Vec<Atom>, n: usize, out: &mut Vec<u8>) -> Stats {
        let mut words = Words::default();
        let mut stats = Stats::default();
        while atoms[0].y != 0 {
            let (next, round) = contract(&atoms, &mut words);
            stats.rounds.push(round);
            atoms = next;
        }
        let root = atoms[0].word;
        stats.period = words.len(root);
        stats.rules = words.0.len();
        stats.used = words.write(root, n, out);
        stats
    }
    pub fn column(ll: &[u8], p: u32, out: &mut Vec<u8>) -> Stats {
        let mut records = Vec::new();
        let mut i = 0;
        while i < ll.len() {
            let a = ll[i];
            let mut end = i + 1;
            while end < ll.len() && end - i < 127 && ll[end] == a {
                end += 1;
            }
            records.push((u16::from(a) << 8) | (end - i) as u16);
            i = end;
        }
        Self::records(&records, ll.len(), p, out).unwrap()
    }
}

impl InverseBwt for Extrema {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(l, c, p, out)
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(l, c, p)
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, _: Kernels) -> bool {
        Self::records(r.records(), n, p, out).is_some()
    }
}

// This path consumes the scanner's Huffman symbols directly. A complete
// RUNA/RUNB integer extends a single maximal run, regardless of its width.
fn maximal(b: &crate::block::Symbols) -> Result<(Vec<(u8, u32)>, usize), crate::ErrorKind> {
    use crate::ErrorKind;
    let mut runs: Vec<(u8, u32)> = Vec::new();
    let mut total = 0usize;
    let mut append = |a: u8, w: u32| -> Result<(), ErrorKind> {
        total += w as usize;
        if total > b.max_len {
            return Err(ErrorKind::BlockTooLong);
        }
        if let Some(last) = runs.last_mut().filter(|v| v.0 == a) {
            last.1 += w;
        } else {
            runs.push((a, w));
        }
        Ok(())
    };
    let mut list = b.seq_to_unseq;
    let (mut pending, mut weight) = (0, 1u32);
    for &sym in &b.syms {
        if sym <= 1 {
            if weight >= 1 << 21 {
                return Err(ErrorKind::RunTooLong);
            }
            pending += (u32::from(sym) + 1) * weight;
            weight *= 2;
        } else {
            if pending != 0 {
                append(list[0], pending)?;
                pending = 0;
                weight = 1;
            }
            if sym == b.eob {
                return Ok((runs, total));
            }
            let index = (usize::from(sym) - 1).min(255);
            let a = list[index];
            list.copy_within(..index, 1);
            list[0] = a;
            append(a, 1)?;
        }
    }
    Err(ErrorKind::Truncated)
}

pub fn direct(data: &[u8], k: Kernels) -> Result<Vec<u8>, crate::Error> {
    use crate::stream::{Event, Scanner, SliceSource};
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let (mut raw, mut out) = (Vec::new(), Vec::new());
    while let Event::Block {
        index,
        bit,
        symbols,
        ..
    } = sc.next(syms)?
    {
        let error = |kind| crate::Error {
            kind,
            block: Some(index),
            bit: Some(bit),
        };
        let (runs, n) = maximal(&symbols).map_err(error)?;
        let atoms = col(runs.iter().copied(), n as u32, symbols.orig_ptr)
            .ok_or_else(|| error(crate::ErrorKind::BadOrigPtr))?;
        Extrema::expand(atoms, n, &mut raw);
        crate::finish(&raw, symbols.crc, k, &mut out).map_err(error)?;
        syms = symbols.syms;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn audit(mut atoms: Vec<Atom>) {
        let last = *atoms.last().unwrap();
        let n = (last.x + last.width) as usize;
        let initial_atoms = atoms.len();
        let mut lf = vec![0; n];
        let mut byte = vec![0; n];
        let mut gap_boundary = vec![false; n];
        for a in &atoms {
            gap_boundary[a.x as usize] = true;
            for j in 0..a.width {
                lf[(a.x + j) as usize] = (a.y + j) as usize;
                byte[(a.x + j) as usize] = (a.word - 1) as u8;
            }
        }
        let mut words = Words::default();
        let mut spelling = Vec::new();
        loop {
            assert!(atoms.len() <= 1 + 3 * (initial_atoms - 1));
            let raw: Vec<_> = atoms.iter().flat_map(|a| a.raw..a.raw + a.width).collect();
            assert!(raw.windows(2).all(|p| p[0] < p[1]));
            let mut active = vec![false; n];
            for &x in &raw {
                active[x as usize] = true;
            }
            for a in &atoms {
                words.write(a.word, words.len(a.word) as usize, &mut spelling);
                for j in 0..a.width {
                    let mut x = (a.raw + j) as usize;
                    let mut bad = false;
                    let mut trace = Vec::new();
                    loop {
                        bad |= gap_boundary[x];
                        trace.push(byte[x]);
                        x = lf[x];
                        if active[x] {
                            break;
                        }
                    }
                    trace.reverse();
                    assert_eq!(trace, spelling);
                    assert_eq!(x as u32, raw[(a.y + j) as usize]);
                    assert_eq!(bad, j == 0 && a.seam);
                }
            }
            if atoms[0].y == 0 {
                break;
            }
            let (next, s) = contract(&atoms, &mut words);
            assert!(s.minima <= s.rows / 2);
            assert!(s.ports <= 3 * s.atoms);
            assert!(s.leaves <= s.seeds + s.ports);
            atoms = next;
        }
    }

    #[test]
    fn interval_count_can_grow() {
        let mut x = 0;
        let widths = [1, 6, 1, 4];
        let images = [4, 6, 5, 0];
        let mut atoms = Vec::new();
        for (i, (&w, &y)) in widths.iter().zip(&images).enumerate() {
            atoms.push(Atom {
                x,
                y,
                raw: x,
                width: w,
                word: i as u32 + 1,
                seam: true,
            });
            x += w;
        }
        audit(atoms.clone());
        let mut words = Words::default();
        let (next, _) = contract(&atoms, &mut words);
        assert_eq!(next.len(), 5);
    }

    #[test]
    fn direct_run_reader_agrees_with_scalar_mtf() {
        use crate::{block::Symbols, ErrorKind};
        let mut b = Symbols {
            crc: 0,
            orig_ptr: 0,
            seq_to_unseq: std::array::from_fn(|i| i as u8),
            syms: Vec::new(),
            eob: 6,
            max_len: 900_000,
        };
        let mut ll = Vec::new();
        let mut counts = [0; 256];
        for mut code in 0..5usize.pow(6) {
            b.syms.clear();
            for _ in 0..6 {
                b.syms.push((code % 5) as u16);
                code /= 5;
            }
            b.syms.push(b.eob);
            let (runs, n) = maximal(&b).unwrap();
            crate::mtf::mtf_scalar(&b, &mut ll, &mut counts).unwrap();
            assert_eq!(n, ll.len());
            assert!(runs.windows(2).all(|p| p[0].0 != p[1].0));
            let expanded: Vec<_> = runs
                .iter()
                .flat_map(|&(a, w)| std::iter::repeat(a).take(w as usize))
                .collect();
            assert_eq!(expanded, ll);
        }
        b.syms = vec![0; 22];
        b.syms.push(b.eob);
        assert_eq!(maximal(&b), Err(ErrorKind::RunTooLong));
        b.max_len = 2;
        b.syms = vec![1, 0, b.eob];
        assert_eq!(maximal(&b), Err(ErrorKind::BlockTooLong));
    }

    #[test]
    fn arbitrary_columns_every_primary() {
        let mut plain = crate::bwt::Plain::default();
        let (mut oracle, mut actual) = (Vec::new(), Vec::new());
        let (mut columns, mut marks) = (0, 0);
        for (sigma, limit) in [(2usize, 10), (3, 7)] {
            for n in 1..=limit {
                for mut code in 0..sigma.pow(n) {
                    let mut last = vec![0; n as usize];
                    let mut counts = [0; 256];
                    for a in &mut last {
                        *a = (code % sigma) as u8;
                        counts[*a as usize] += 1;
                        code /= sigma;
                    }
                    columns += 1;
                    for p in 0..n {
                        plain.invert(&last, &counts, p, &mut oracle);
                        Extrema::column(&last, p, &mut actual);
                        assert_eq!(actual, oracle, "{last:?} primary {p}");
                        marks += 1;
                    }
                }
            }
        }
        eprintln!("{columns} columns, {marks} primary choices");
    }

    #[test]
    fn arbitrary_permutations_and_wide_exchanges() {
        // Every permutation through seven ranks. The alphabet merges adjacent
        // elementary translations, independently of the contracted words.
        fn perm(data: &mut [u32], i: usize, checked: &mut usize) {
            if i == data.len() {
                let mut atoms: Vec<Atom> = Vec::new();
                for (x, &y) in data.iter().enumerate() {
                    let x = x as u32;
                    if let Some(a) = atoms.last_mut() {
                        if a.y + a.width == y {
                            a.width += 1;
                            continue;
                        }
                    }
                    atoms.push(Atom {
                        x,
                        y,
                        raw: x,
                        width: 1,
                        word: 1 + (x % 3),
                        seam: true,
                    });
                }
                audit(atoms);
                *checked += 1;
                return;
            }
            for j in i..data.len() {
                data.swap(i, j);
                perm(data, i + 1, checked);
                data.swap(i, j);
            }
        }
        let mut total = 0;
        for n in 1..=7 {
            perm(&mut (0..n).collect::<Vec<_>>(), 0, &mut total);
        }
        let mut z = 1729u32;
        let mut draw = || {
            z ^= z << 13;
            z ^= z >> 17;
            z ^= z << 5;
            z
        };
        for _ in 0..2000 {
            let k = 2 + (draw() % 19) as usize;
            let mut atoms = Vec::new();
            let mut x = 0;
            for _ in 0..k {
                let width = 1 + draw() % 43;
                atoms.push(Atom {
                    x,
                    y: 0,
                    raw: x,
                    width,
                    word: 1 + draw() % 5,
                    seam: true,
                });
                x += width;
            }
            let mut order: Vec<_> = (0..k).collect();
            for i in (1..k).rev() {
                order.swap(i, (draw() as usize) % (i + 1));
            }
            let mut y = 0;
            for i in order {
                atoms[i].y = y;
                y += atoms[i].width;
            }
            audit(atoms);
        }
        eprintln!(
            "{total} permutations and 2000 seeded interval exchanges, every surviving row audited"
        );
    }

    #[test]
    fn partial_final_period_and_front_contract() {
        let mut out = Vec::new();
        assert!(Extrema::records(&[0x101, 1, 1], 4, 0, &mut out).is_none());
        assert!(Extrema::records(&[0x101, 1, 1], 3, 3, &mut out).is_none());
        // Column "aba" fixes row 0 and exchanges rows 1 and 2.
        let mut plain = crate::bwt::Plain::default();
        let mut counts = [0; 256];
        counts[97] = 2;
        counts[98] = 1;
        let mut expected = Vec::new();
        plain.invert(b"aba", &counts, 1, &mut expected);
        Extrema::column(b"aba", 1, &mut out);
        assert_eq!(out, expected);
        assert_eq!(out, b"aba");
    }
}
