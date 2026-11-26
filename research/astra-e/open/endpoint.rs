//! Eliminate the open complement with an auxiliary interval exchange.
//! Its prefix contains terminal coordinates only. All narrow-run wiring is absent.
use super::{Band, Kernel};

#[derive(Default, Clone, Copy)]
struct Node {
    prev: [u32; 2],
    next: [u32; 2],
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
    order: [Vec<u32>; 2],
    terminals: [Vec<u32>; 2],
    first_band: Vec<u32>,
    stack: Vec<u32>,
}

impl Endpoint {
    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(Node {
            width,
            word,
            ..Node::default()
        });
        id
    }

    fn cut(&mut self, id: u32, side: usize) {
        let h = self.nodes[id as usize];
        self.nodes[h.prev[side] as usize].next[side] = h.next[side];
        self.nodes[h.next[side] as usize].prev[side] = h.prev[side];
    }

    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = self.nodes[at as usize].next[side];
        self.nodes[id as usize].prev[side] = at;
        self.nodes[id as usize].next[side] = next;
        self.nodes[at as usize].next[side] = id;
        self.nodes[next as usize].prev[side] = id;
    }

    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            self.rules[(w - 257) as usize].len
        }
    }

    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let id = self.rules.len() as u32 + 257;
        self.rules.push(Rule {
            left,
            right,
            len: self.len(left) + self.len(right),
            first: u32::MAX,
        });
        id
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
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let rule = &mut self.rules[(id - 257) as usize];
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
    pub(super) fn induced(&mut self, n: u32, thin: &[u32; 256]) {
        let e = &mut self.endpoint;
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
            e.order[0].push(i as u32 + 1);
            x = s.from + s.width;
        }
        e.gaps(0, x, n);
        x = 0;
        for &i in &self.image_order {
            let s = self.strips[i as usize];
            e.gaps(1, x, s.to);
            e.order[1].push(i + 1);
            x = s.to + s.width;
        }
        e.gaps(1, x, n);

        e.lists();
        e.eliminate(n);

        e.first_band.resize(e.nodes.len(), 0);
        let mut id = e.nodes[0].next[0];
        let mut in_node = 0;
        let mut k = 0;
        self.top_bands.clear();
        // Top-order word codes carry the initial thin byte. Only its 255 bucket
        // boundaries can cut an endpoint node further.
        for (a, &count) in thin.iter().enumerate() {
            let end = k + count;
            while k < end {
                if in_node == 0 {
                    e.first_band[id as usize] = self.top_bands.len() as u32;
                }
                let node = e.nodes[id as usize];
                let width = (end - k).min(node.width - in_node);
                let code = if node.word == 0 {
                    a as u32
                } else {
                    e.append(node.word, &mut self.spelling);
                    self.spelling.push(a as u8);
                    self.offsets.push(self.spelling.len() as u32);
                    254 + self.offsets.len() as u32
                };
                self.top_bands.push(Band {
                    end: k + width,
                    source: k,
                    code,
                });
                k += width;
                in_node += width;
                if in_node == node.width {
                    id = node.next[0];
                    in_node = 0;
                }
            }
        }
        // The same nodes in exit order spell consecutive retained L ranks.
        let mut bottom = 0;
        id = e.nodes[0].next[1];
        while id != 0 {
            let node = e.nodes[id as usize];
            let mut band_id = e.first_band[id as usize] as usize;
            let end = bottom + node.width;
            while bottom < end {
                let top = self.top_bands[band_id];
                bottom += top.end - top.source;
                self.bands.push(Band {
                    end: bottom,
                    source: top.source,
                    code: top.code,
                });
                band_id += 1;
            }
            id = node.next[1];
        }
        assert_eq!(bottom, k);
    }
}
