//! Eliminate the open complement with an auxiliary interval exchange.
//! Its prefix contains terminal coordinates only. All narrow-run wiring is absent.
use super::{Band, Counts, Kernel};

#[derive(Default, Clone, Copy)]
struct Node {
    prev: [u16; 2],
    next: [u16; 2],
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
pub(super) struct Endpoint {
    nodes: Vec<Node>,
    rules: Vec<Rule>,
    order: [Vec<u16>; 2],
    terminals: [Vec<u16>; 2],
    bottom: Vec<u32>,
    stack: Vec<u32>,
}

impl Endpoint {
    fn node(&mut self, width: u32, word: u32) -> u16 {
        let id = self.nodes.len() as u16;
        self.nodes.push(Node {
            width,
            word,
            ..Node::default()
        });
        id
    }

    fn cut(&mut self, id: u16, side: usize) {
        let h = self.nodes[id as usize];
        self.nodes[h.prev[side] as usize].next[side] = h.next[side];
        self.nodes[h.next[side] as usize].prev[side] = h.prev[side];
    }

    fn insert(&mut self, at: u16, id: u16, side: usize) {
        let next = self.nodes[at as usize].next[side];
        self.nodes[id as usize].prev[side] = at;
        self.nodes[id as usize].next[side] = next;
        self.nodes[at as usize].next[side] = id;
        self.nodes[next as usize].prev[side] = id;
    }

    fn len(&self, w: u32) -> u32 {
        match w >> 30 {
            0 => u32::from(w != 0),
            1 => 2,
            2 => 3,
            _ => self.rules[(w & 0x3fff_ffff) as usize].len,
        }
    }

    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 { return right; }
        if right == 0 { return left; }
        // Leave the original single-letter representation available to node()
        // and distinguish actual rules from two- and three-byte immediates.
        let (lt, rt) = (left >> 30, right >> 30);
        match (lt, rt) {
            (0, 0) => (1 << 30) | (left - 1) | ((right - 1) << 8),
            (0, 1) => (2 << 30) | (left - 1) | ((right & 0xffff) << 8),
            (1, 0) => (2 << 30) | (left & 0xffff) | ((right - 1) << 16),
            _ => {
                let id = (3 << 30) | self.rules.len() as u32;
                self.rules.push(Rule {
                    left,
                    right,
                    len: self.len(left) + self.len(right),
                    first: u32::MAX,
                });
                id
            }
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
            let tag = id >> 30;
            if tag != 3 {
                if tag == 0 {
                    dest.push((id - 1) as u8);
                } else {
                    dest.extend_from_slice(&id.to_le_bytes()[..tag as usize + 1]);
                }
            } else {
                let rule = &mut self.rules[(id & 0x3fff_ffff) as usize];
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

impl Kernel {
    pub(super) fn induced_narrow(
        &mut self,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        let e = &mut self.narrow;
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
            e.order[0].push(i as u16 + 1);
            x = s.from + s.width;
        }
        e.gaps(0, x, n);
        x = 0;
        for &i in &self.image_order {
            let s = self.strips[i as usize];
            e.gaps(1, x, s.to);
            e.order[1].push(i as u16 + 1);
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
        assert_eq!(x as usize, stats.retained);
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
                    let height = e.len(node.word) as usize;
                    stats.words += 1;
                    stats.copies += width as usize;
                    stats.traced += height;
                    stats.thick_area += width as usize * height;
                    stats.unit_work += usize::from(width == 1) * height;
                    stats.longest = stats.longest.max(height);
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
        assert_eq!(id, 0);
        cursors
    }
}

const _: () = assert!(std::mem::size_of::<Node>() == 16);
