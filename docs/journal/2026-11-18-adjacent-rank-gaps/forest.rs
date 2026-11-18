//! Experimental first returns through a forest on short BWT contexts.
use crate::bwt::{InverseBwt, Segmented};
use crate::{runs::Runs, Kernels};

#[derive(Clone, Copy)]
struct Run {
    end: u32,
    byte: u8,
}

fn read_runs(records: &[u16]) -> (Vec<Run>, [u32; 256], u32) {
    let mut runs: Vec<Run> = Vec::with_capacity(records.len());
    let mut counts = [0; 256];
    let mut n = 0;
    for &record in records {
        let byte = (record >> 8) as u8;
        let width = u32::from(record & 255);
        n += width;
        counts[usize::from(byte)] += width;
        if runs.last().is_some_and(|run| run.byte == byte) {
            runs.last_mut().unwrap().end = n;
        } else {
            runs.push(Run { end: n, byte });
        }
    }
    (runs, counts, n)
}

// Consecutive intervals with the same first byte are emitted in the old context order.
fn partition(runs: &[Run], counts: &[u32; 256], k: usize) -> Vec<u32> {
    assert!(k > 0);
    let mut ends = Vec::new();
    let mut n = 0;
    for &width in counts.iter().filter(|&&w| w > 0) {
        n += width;
        ends.push(n);
    }
    let mut buckets: [Vec<u32>; 256] = std::array::from_fn(|_| Vec::new());
    let mut histogram = [0; 256];
    let mut letters = Vec::new();
    for _ in 1..k {
        let (mut pos, mut r) = (0, 0);
        for &end in &ends {
            while pos < end {
                let run = runs[r];
                let a = usize::from(run.byte);
                let right = end.min(run.end);
                if histogram[a] == 0 {
                    letters.push(a);
                }
                histogram[a] += right - pos;
                pos = right;
                r += usize::from(right == run.end);
            }
            for a in letters.drain(..) {
                buckets[a].push(std::mem::take(&mut histogram[a]));
            }
        }
        let old_len = ends.len();
        ends.clear();
        let mut end = 0;
        for bucket in &mut buckets {
            for width in bucket.drain(..) {
                end += width;
                ends.push(end);
            }
        }
        if ends.len() == old_len {
            break;
        }
    }
    ends
}

#[derive(Clone, Copy)]
struct Slice {
    end: u32,
    image: u32,
    byte: u8,
}

#[derive(Clone, Copy, Default)]
struct DefaultEdge {
    parent: usize,
    byte: u8,
    mass: u32,
}

struct Model {
    ends: Vec<u32>,
    offsets: Vec<usize>,
    slices: Vec<Slice>,
    edges: Vec<Option<DefaultEdge>>,
}

impl Model {
    fn new(runs: &[Run], counts: &[u32; 256], k: usize) -> Self {
        let ends = partition(runs, counts, k);
        let mut cursor = [0; 256];
        let mut count = 0;
        for (slot, &width) in cursor.iter_mut().zip(counts) {
            *slot = count;
            count += width;
        }
        let mut offsets = vec![0];
        let mut edges = Vec::with_capacity(ends.len());
        let mut slices = Vec::with_capacity(runs.len() + ends.len());
        let mut mass = [0u32; 256];
        let mut touched = Vec::new();
        let (mut pos, mut r) = (0, 0);
        for &end in &ends {
            while pos < end {
                let run = runs[r];
                let a = usize::from(run.byte);
                let right = end.min(run.end);
                let width = right - pos;
                if mass[a] == 0 {
                    touched.push(a);
                }
                mass[a] += width;
                slices.push(Slice {
                    end: right,
                    image: cursor[a],
                    byte: run.byte,
                });
                cursor[a] += width;
                pos = right;
                r += usize::from(right == run.end);
            }
            let a = *touched
                .iter()
                .max_by_key(|&&a| (mass[a], std::cmp::Reverse(a)))
                .unwrap();
            let first_image = cursor[a] - mass[a];
            let parent = ends.partition_point(|&end| end <= first_image);
            debug_assert!(cursor[a] <= ends[parent]);
            edges.push(Some(DefaultEdge {
                parent,
                byte: a as u8,
                mass: mass[a],
            }));
            for a in touched.drain(..) {
                mass[a] = 0;
            }
            offsets.push(slices.len());
        }
        Self {
            ends,
            offsets,
            slices,
            edges,
        }
    }

    fn state(&self, row: u32) -> usize {
        self.ends.partition_point(|&end| end <= row)
    }

    fn start(&self, state: usize) -> u32 {
        if state == 0 {
            0
        } else {
            self.ends[state - 1]
        }
    }

    // A functional graph is a collection of in-trees attached to circuits.
    // Cut the least-used default on each circuit. Parents then precede children.
    fn order(&mut self) -> (Vec<usize>, usize) {
        let mut mark = vec![0u8; self.ends.len()];
        let mut order = Vec::with_capacity(mark.len());
        let mut path = Vec::new();
        let mut cycles = 0;
        for start in 0..mark.len() {
            if mark[start] == 2 {
                continue;
            }
            let mut v = start;
            while mark[v] == 0 {
                mark[v] = 1;
                path.push(v);
                v = self.edges[v].unwrap().parent;
            }
            if mark[v] == 1 {
                let i = path.iter().rposition(|&w| w == v).unwrap();
                let cut = *path[i..]
                    .iter()
                    .min_by_key(|&&w| (self.edges[w].unwrap().mass, w))
                    .unwrap();
                self.edges[cut] = None;
                cycles += 1;
                let mut j = path.iter().rposition(|&w| w == cut).unwrap();
                // Enumerate the circuit from the cut towards its children.
                for _ in i..path.len() {
                    order.push(path[j]);
                    if j == i {
                        j = path.len() - 1;
                    } else {
                        j -= 1;
                    }
                }
                order.extend(path[..i].iter().rev());
            } else {
                order.extend(path.iter().rev());
            }
            for w in path.drain(..) {
                mark[w] = 2;
            }
        }
        (order, cycles)
    }
}

// jump(x) = base + (x-left), up to end. Destination IDs name individual
// exceptional transitions. A band's ID range stays contiguous.
#[derive(Clone, Copy)]
struct Piece {
    end: u32,
    image_end: u32,
}

struct Answers {
    ranges: Vec<std::ops::Range<usize>>,
    pieces: Vec<Piece>,
}

impl Answers {
    fn append(&mut self, begin: usize, end: u32, image_end: u32, width: u32) {
        if self.pieces.len() > begin {
            let last = self.pieces.last_mut().unwrap();
            if last.image_end == image_end - width {
                *last = Piece { end, image_end };
                return;
            }
        }
        self.pieces.push(Piece { end, image_end });
    }

    fn at(&self, state: usize, x: u32) -> u32 {
        let p = &self.pieces[self.ranges[state].clone()];
        let part = p[p.partition_point(|v| v.end <= x)];
        part.image_end - (part.end - x)
    }
}

#[derive(Clone, Copy)]
struct Cut {
    base: u32,
    image: u32,
    width: u32,
    byte: u8,
    exit: usize,
}

#[derive(Clone, Copy)]
struct Path {
    end: u32,
    depth: u32,
}

#[derive(Default, Debug, Clone)]
pub struct Stats {
    pub n: usize,
    pub runs: usize,
    pub states: usize,
    pub slices: usize,
    pub cycles: usize,
    pub capped: usize,
    pub max_depth: u32,
    pub path_bytes: usize,
    pub cut_rows: usize,
    pub cut_slices: usize,
    pub pieces: usize,
    pub inherited: usize,
    pub towers: usize,
    pub period: usize,
    pub returns: usize,
}

struct Compiled {
    paths: Vec<Path>,
    spellings: Vec<u8>,
    exits: Vec<Cut>,
    answers: Answers,
    stats: Stats,
}

const MAX_DEPTH: u32 = 64;

fn compile(model: &mut Model, r: usize) -> Compiled {
    let (order, cycles) = model.order();
    let mut stats = Stats {
        n: *model.ends.last().unwrap() as usize,
        runs: r,
        states: order.len(),
        slices: model.slices.len(),
        cycles,
        ..Stats::default()
    };
    let mut paths = vec![Path { end: 0, depth: 0 }; order.len()];
    let mut spellings = Vec::new();
    let mut answers = Answers {
        ranges: vec![0..0; order.len()],
        pieces: Vec::new(),
    };
    let mut exits = Vec::new();
    // Computing a state uses only its parent's already compiled ranges.
    for state in order {
        if let Some(edge) = model.edges[state] {
            let prior = paths[edge.parent];
            if prior.depth == MAX_DEPTH {
                model.edges[state] = None;
                stats.capped += 1;
            } else {
                spellings
                    .extend_from_within((prior.end - prior.depth) as usize..prior.end as usize);
                spellings.push(edge.byte);
                paths[state] = Path {
                    depth: prior.depth + 1,
                    end: spellings.len() as u32,
                };
                stats.max_depth = stats.max_depth.max(prior.depth + 1);
            }
        }
        let default = model.edges[state];
        let begin = answers.pieces.len();
        let mut x = model.start(state);
        // All default slices consume consecutive ranks of one parent, even when
        // interrupted by cuts. Locate the first parent piece just once.
        let mut parent_cursor = None;
        for seg in &model.slices[model.offsets[state]..model.offsets[state + 1]] {
            let width = seg.end - x;
            if let Some(edge) = default.filter(|edge| edge.byte == seg.byte) {
                let mut y = seg.image;
                let until = y + width;
                let range = answers.ranges[edge.parent].clone();
                let j = parent_cursor.get_or_insert_with(|| {
                    range.start + answers.pieces[range.clone()].partition_point(|p| p.end <= y)
                });
                while y < until {
                    let inherited = answers.pieces[*j];
                    let t = inherited.end.min(until);
                    let h = t - y;
                    x += h;
                    answers.append(begin, x, inherited.image_end - (inherited.end - t), h);
                    stats.inherited += 1;
                    *j += usize::from(t == inherited.end);
                    y = t;
                }
            } else {
                let base = stats.cut_rows as u32;
                exits.push(Cut {
                    base,
                    image: seg.image,
                    width,
                    byte: seg.byte,
                    exit: state,
                });
                stats.cut_rows += width as usize;
                answers.append(begin, seg.end, base + width, width);
                x = seg.end;
            }
        }
        answers.ranges[state] = begin..answers.pieces.len();
    }
    stats.cut_slices = exits.len();
    stats.path_bytes = spellings.len();
    stats.pieces = answers.pieces.len();
    Compiled {
        paths,
        spellings,
        exits,
        answers,
        stats,
    }
}

#[derive(Clone, Copy)]
struct Return {
    next: u32,
    word: u32,
}

// An edge of the section has forward spelling: exceptional byte, then the
// suffix of the entry state's default path down to the exceptional state.
#[derive(Clone, Copy)]
struct Word {
    end: u32,
    len: u32,
    first: u8,
}

fn word(paths: &[Path], entry: usize, exit: &Cut) -> Word {
    let p = paths[entry];
    Word {
        end: p.end,
        len: p.depth - paths[exit.exit].depth,
        first: exit.byte,
    }
}

fn marked(model: &Model, compiled: &mut Compiled, p: u32, out: &mut Vec<u8>) {
    let Compiled {
        paths,
        spellings,
        exits,
        answers,
        stats,
    } = compiled;
    let mut links = vec![Return { next: 0, word: 0 }; stats.cut_rows];
    let mut words = Vec::new();
    for root in exits.iter() {
        let s = model.state(root.image);
        let covered = &answers.pieces[answers.ranges[s].clone()];
        let mut i = covered.partition_point(|v| v.end <= root.image);
        let mut pos = root.image;
        while pos < root.image + root.width {
            let piece = covered[i];
            let end = piece.end.min(root.image + root.width);
            let mut q = piece.image_end - (piece.end - pos);
            let mut dest = exits.partition_point(|e| e.base + e.width <= q);
            while pos < end {
                let e = &exits[dest];
                let width = (end - pos).min(e.base + e.width - q);
                let tag = words.len() as u32;
                words.push(word(paths, s, e));
                let start = (root.base + (pos - root.image)) as usize;
                for (off, link) in links[start..start + width as usize].iter_mut().enumerate() {
                    *link = Return {
                        next: q + off as u32,
                        word: tag,
                    };
                }
                q += width;
                pos += width;
                dest += 1;
            }
            i += 1;
        }
    }
    stats.towers = words.len();
    let s = model.state(p);
    let anchor = answers.at(s, p);
    let e = exits[exits.partition_point(|e| e.base + e.width <= anchor)];
    let phase = word(paths, s, &e).len as usize + 1;
    // The row reached from p by 'phase' LF steps is anchor. Form its forward
    // period right-aligned, advance by phase in forward time, then repeat.
    let n = stats.n;
    out.resize(n, 0);
    let mut pos = n;
    let mut j = anchor;
    loop {
        let link = links[j as usize];
        let w = words[link.word as usize];
        let end = pos;
        pos -= w.len as usize + 1;
        out[pos] = w.first;
        out[pos + 1..end].copy_from_slice(&spellings[(w.end - w.len) as usize..w.end as usize]);
        j = link.next;
        stats.returns += 1;
        if j == anchor {
            break;
        }
    }
    let period = n - pos;
    stats.period = period;
    out.copy_within(pos..n, 0);
    out[..period].rotate_left(phase % period);
    let mut written = period;
    while written < n {
        let add = written.min(n - written);
        out.copy_within(..add, written);
        written += add;
    }
}

#[derive(Default)]
pub struct Forest<const K: usize> {
    fallback: Segmented,
}

impl<const K: usize> Forest<K> {
    pub fn records(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> Option<Stats> {
        let (runs, counts, total) = read_runs(records);
        if total as usize != n || p >= total {
            return None;
        }
        let mut model = Model::new(&runs, &counts, K);
        let mut compiled = compile(&mut model, runs.len());
        marked(&model, &mut compiled, p, out);
        Some(compiled.stats)
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

impl<const K: usize> InverseBwt for Forest<K> {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(ll, counts, p, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(ll, counts, p)
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, _: Kernels) -> bool {
        Self::records(r.records(), n, p, out).is_some()
    }
}
