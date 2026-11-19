//! Exact reconstruction through maximal-ascent phrase BWTs. The shipping
//! interface is only used for run access and the fallback paths.
use crate::{
    bwt::{InverseBwt, LazyCells},
    runs::Runs,
    Kernels,
};

#[derive(Clone, Copy)]
struct Run {
    token: u32,
    width: u32,
}
#[derive(Default)]
struct Layer {
    runs: Vec<Run>,
    bytes: Vec<u8>,
    // End of the expansion of each token id, preceded by zero.
    offsets: Vec<u32>,
}
impl Layer {
    fn word(&self, token: u32) -> &[u8] {
        &self.bytes
            [self.offsets[token as usize] as usize..self.offsets[token as usize + 1] as usize]
    }
    fn push(&mut self, token: u32, width: u32) {
        if let Some(last) = self.runs.last_mut() {
            if last.token == token {
                last.width += width;
                return;
            }
        }
        self.runs.push(Run { token, width });
    }
}

#[derive(Clone, Copy)]
struct Span {
    end: u32,
    dest: u32,
    first: u32,
    last: u32,
}
#[derive(Default, Clone, Copy)]
struct Node {
    parent: u32,
    byte: u32,
    child: u32,
    sibling: u32,
    code: u32,
}
#[derive(Clone, Copy)]
struct Prefix {
    end: u32,
    name: u32,
}

#[derive(Default)]
struct Builder {
    bounds: Vec<u32>,
    cursor: Vec<usize>,
    prefix_start: Vec<usize>,
    spans: Vec<Span>,
    prefix: Vec<Prefix>,
    trie: Vec<Node>,
    stack: Vec<u32>,
    digits: Vec<u32>,
}

impl Builder {
    fn append(&mut self, parent: u32, byte: u32, end: u32) {
        // All suffix-byte requests for a given byte arrive in one contiguous
        // F bucket. A previously published child of this byte must be the head.
        let head = self.trie[parent as usize].child;
        let name = if head != 0 && self.trie[head as usize].byte == byte {
            head
        } else {
            let id = self.trie.len() as u32;
            self.trie[parent as usize].child = id;
            self.trie.push(Node {
                parent,
                byte,
                sibling: head,
                ..Node::default()
            });
            id
        };
        if let Some(last) = self.prefix.last_mut() {
            if last.name == name {
                last.end = end;
                return;
            }
        }
        self.prefix.push(Prefix { end, name });
    }

    fn cumulative(&mut self, src: &Layer) {
        let alphabet = src.offsets.len() - 1;
        self.bounds.clear();
        self.bounds.resize(alphabet + 1, 0);
        for r in &src.runs {
            self.bounds[r.token as usize + 1] += r.width;
        }
        for c in 0..alphabet {
            self.bounds[c + 1] += self.bounds[c];
        }
    }

    fn reduce(&mut self, src: &Layer, dst: &mut Layer, mut p: u32, skip: &mut usize) -> (u32, u32) {
        self.cumulative(src);
        self.spans.clear();
        self.cursor.clear();
        self.cursor.extend(self.bounds.iter().map(|&x| x as usize));
        let mut x = 0;
        let mut c = 0;
        for &Run { token: a, width } in &src.runs {
            let end = x + width;
            while x < end {
                while self.bounds[c + 1] <= x {
                    c += 1;
                }
                let hi = end.min(self.bounds[c + 1]);
                let y = &mut self.cursor[a as usize];
                self.spans.push(Span {
                    end: hi,
                    dest: *y as u32,
                    first: c as u32,
                    last: a,
                });
                *y += (hi - x) as usize;
                x = hi;
            }
        }
        loop {
            let i = self.spans.partition_point(|s| s.end <= p);
            let span = self.spans[i];
            if span.last >= span.first {
                break;
            }
            let lo = if i == 0 { 0 } else { self.spans[i - 1].end };
            *skip += src.word(span.last).len();
            p = span.dest + p - lo;
        }

        self.prefix.clear();
        self.trie.clear();
        self.trie.push(Node::default());
        self.prefix_start.resize(self.bounds.len(), 0);
        let mut prior = u32::MAX;
        x = 0;
        for i in 0..self.spans.len() {
            let span = self.spans[i];
            let c = span.first;
            let a = span.last as usize;
            if c != prior {
                self.prefix_start[c as usize] = self.prefix.len();
                self.cursor[c as usize] = self.prefix.len();
                prior = c;
            }
            if span.last >= c {
                self.append(0, c, span.end);
            } else {
                let mut j = self.cursor[a];
                let mut y = span.dest;
                while self.prefix[j].end <= y {
                    j += 1;
                }
                while x < span.end {
                    let pre = self.prefix[j];
                    let w = (span.end - x).min(pre.end - y);
                    x += w;
                    y += w;
                    self.append(pre.name, c, x);
                    j += usize::from(pre.end == y);
                }
                self.cursor[a] = j;
            }
            x = span.end;
        }

        self.cursor.clone_from(&self.prefix_start);
        dst.runs.clear();
        let mut rank = 0;
        let mut m = 0;
        x = 0;
        for &span in &self.spans {
            if span.last >= span.first {
                if x <= p && p < span.end {
                    rank = m + p - x;
                }
                let j = &mut self.cursor[span.last as usize];
                let mut y = span.dest;
                while self.prefix[*j].end <= y {
                    *j += 1;
                }
                let limit = y + span.end - x;
                while y < limit {
                    let pre = self.prefix[*j];
                    let end = limit.min(pre.end);
                    self.trie[pre.name as usize].code = u32::MAX;
                    dst.push(pre.name, end - y);
                    m += end - y;
                    y = end;
                    *j += usize::from(pre.end == y);
                }
            }
            x = span.end;
        }

        dst.bytes.clear();
        dst.offsets.clear();
        dst.offsets.push(0);
        self.stack.clear();
        self.stack.push(0);
        while let Some(id) = self.stack.pop() {
            let node = self.trie[id as usize];
            if node.code == u32::MAX {
                self.trie[id as usize].code = dst.offsets.len() as u32 - 1;
                self.digits.clear();
                let mut q = id;
                while q != 0 {
                    let part = self.trie[q as usize];
                    self.digits.push(part.byte);
                    q = part.parent;
                }
                for &b in self.digits.iter().rev() {
                    dst.bytes.extend_from_slice(src.word(b));
                }
                dst.offsets.push(dst.bytes.len() as u32);
            }
            let mut child = node.child;
            while child != 0 {
                self.stack.push(child);
                child = self.trie[child as usize].sibling;
            }
        }
        for run in &mut dst.runs {
            run.token = self.trie[run.token as usize].code;
        }
        (m, rank)
    }
}

#[derive(Default)]
pub(super) struct Solver {
    a: Layer,
    b: Layer,
    build: Builder,
    table: Vec<u32>,
    wide: Vec<u32>,
    backward: Vec<u32>,
}

impl Solver {
    pub(super) fn decode(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        stop: usize,
    ) {
        self.a.runs.clear();
        self.a.bytes.clear();
        self.a.bytes.extend(0u8..=255);
        self.a.offsets.clear();
        self.a.offsets.extend(0u32..=256);
        for &r in records {
            self.a.push((r >> 8).into(), (r & 255).into());
        }

        let (mut m, mut p) = (n as u32, p);
        let mut skip = 0;
        for _ in 0..8 {
            if m as usize <= stop {
                break;
            }
            let (next, q) = self.build.reduce(&self.a, &mut self.b, p, &mut skip);
            std::mem::swap(&mut self.a, &mut self.b);
            p = q;
            if next == m {
                break;
            }
            m = next;
        }

        self.build.cumulative(&self.a);
        let bits = (32 - (m - 1).leading_zeros()).max(1);
        let mask = (1 << bits) - 1;
        let wide = self.a.offsets.len() - 2 > (u32::MAX >> bits) as usize;
        self.table.clear();
        self.wide.clear();
        for r in &self.a.runs {
            let next = &mut self.build.bounds[r.token as usize];
            let base = r.token << bits;
            self.table
                .extend((*next..*next + r.width).map(|y| base | y));
            if wide {
                self.wide.resize(self.table.len(), r.token);
            }
            *next += r.width;
        }
        assert_eq!(self.table.len(), m as usize);
        self.backward.clear();
        let mut row = p;
        loop {
            let cell = self.table[row as usize];
            self.backward.push(if wide {
                self.wide[row as usize]
            } else {
                cell >> bits
            });
            row = cell & mask;
            if row == p {
                break;
            }
        }
        out.clear();
        out.reserve(n);
        let mut tokens = self.backward.iter().rev();
        let first = self.a.word(*tokens.next().unwrap());
        out.extend_from_slice(&first[skip..]);
        for &token in tokens {
            out.extend_from_slice(self.a.word(token));
        }
        out.extend_from_slice(&first[..skip]);
        assert!(!out.is_empty() && out.len() <= n);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }
}

#[derive(Default)]
pub struct Candidate {
    solver: Solver,
    fallback: LazyCells,
}
impl InverseBwt for Candidate {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(ll, counts, p, out);
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(ll, counts, p)
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if n < 65536 || r.records().len() * 20 >= n * 3 {
            return self.fallback.invert_runs(r, n, p, out, k);
        }
        self.solver.decode(r.records(), n, p, out, 32768);
        true
    }
}
