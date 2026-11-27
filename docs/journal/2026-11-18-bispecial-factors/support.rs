//! Decode a marked cyclic byte word using only unweighted two-sided extension supports.
//! This file compiles on its own. It has no rank index, byte histograms or input BWT.
use std::{collections::VecDeque, env, fs};

#[derive(Default)]
struct Node {
    // Each byte points to the class reached by prefixing it to this context.
    incoming: Vec<(u8, usize)>,
    // Future subclasses in lexicographic order; an empty list denotes an unsplit class.
    children: Vec<usize>,
}

fn split(nodes: &mut Vec<Node>, id: usize, count: usize, n: usize) -> Vec<usize> {
    assert!(nodes[id].children.is_empty() && count >= 2);
    let start = nodes.len();
    assert!(start + count < 2 * n);
    nodes.resize_with(start + count, Node::default);
    let children: Vec<_> = (start..start + count).collect();
    nodes[id].children.clone_from(&children);
    children
}

struct Bits<'a> {
    data: &'a [u8],
    used: usize,
    limit: usize,
}

impl Bits<'_> {
    fn next(&mut self) -> bool {
        assert!(self.used < self.limit);
        let value = self.data[self.used / 8] & (1 << (self.used % 8)) != 0;
        self.used += 1;
        value
    }
}

pub struct Futures {
    // One row per distinct infinite future, in lexicographic order.
    left: Vec<(u8, usize)>,
    n: usize,
    mark: usize,
}

impl Futures {
    pub fn classes(&self) -> usize {
        self.left.len()
    }

    pub fn marked_output(&self) -> (Vec<u8>, usize) {
        let mut v = self.mark;
        let mut period = Vec::new();
        loop {
            let (a, target) = self.left[v];
            period.push(a);
            v = target;
            if v == self.mark {
                break;
            }
        }
        period.reverse();
        let bytes = period.iter().copied().cycle().take(self.n).collect();
        (bytes, period.len())
    }

    pub fn ordinary_column(&self, marked_period: usize) -> Option<Vec<u8>> {
        if self.classes() != marked_period || self.n % marked_period != 0 {
            return None;
        }
        let copies = self.n / marked_period;
        Some(
            self.left
                .iter()
                .flat_map(|&(byte, _)| std::iter::repeat_n(byte, copies))
                .collect(),
        )
    }
}

pub fn read(tape: &[u8]) -> Futures {
    assert!(tape.len() >= 52 && &tape[..4] == b"SUP1");
    let n = u32::from_le_bytes(tape[4..8].try_into().unwrap()) as usize;
    let marked_rank = u32::from_le_bytes(tape[8..12].try_into().unwrap()) as usize;
    let alphabet: Vec<u8> = (0..=255u8)
        .filter(|&a| tape[12 + (a / 8) as usize] & (1 << (a % 8)) != 0)
        .collect();
    let bit_count = u64::from_le_bytes(tape[44..52].try_into().unwrap()) as usize;
    assert!(n > 0 && n <= 900_000 && !alphabet.is_empty());
    assert_eq!(tape.len() - 52, bit_count.div_ceil(8));
    let mut bits = Bits {
        data: &tape[52..],
        used: 0,
        limit: bit_count,
    };
    let mut nodes = vec![Node::default()];
    let mut queue = VecDeque::new();
    if alphabet.len() == 1 {
        nodes[0].incoming.push((alphabet[0], 0));
    } else {
        let letters = split(&mut nodes, 0, alphabet.len(), n);
        nodes[0].incoming.extend(alphabet.into_iter().zip(letters));
        queue.push_back(0);
    }
    while let Some(source) = queue.pop_front() {
        let children = nodes[source].children.clone();
        let incoming = std::mem::take(&mut nodes[source].incoming);
        assert!(!incoming.is_empty());
        let mixed = incoming.len() > 1;
        for (a, target) in incoming {
            // Only a bispecial source needs bits. With one predecessor every
            // right extension follows it, whatever the unknown multiplicities.
            let present: Vec<_> = children
                .iter()
                .copied()
                .filter(|_| !mixed || bits.next())
                .collect();
            assert!(!present.is_empty());
            if present.len() == 1 {
                nodes[present[0]].incoming.push((a, target));
            } else {
                let images = split(&mut nodes, target, present.len(), n);
                queue.push_back(target);
                for (child, image) in present.into_iter().zip(images) {
                    nodes[child].incoming.push((a, image));
                }
            }
        }
        assert!(children.iter().all(|&c| !nodes[c].incoming.is_empty()));
    }
    assert_eq!(bits.used, bits.limit);

    // In the final graph every future has a unique successor and predecessor.
    // The ordered leaves give the distinct-future rank of the original mark.
    let mut stack = vec![0];
    let mut leaves = Vec::new();
    while let Some(v) = stack.pop() {
        if nodes[v].children.is_empty() {
            assert_eq!(nodes[v].incoming.len(), 1);
            leaves.push(v);
        } else {
            stack.extend(nodes[v].children.iter().rev());
        }
    }
    let mut rank = vec![usize::MAX; nodes.len()];
    for (index, &v) in leaves.iter().enumerate() {
        rank[v] = index;
    }
    let mut seen = vec![false; leaves.len()];
    let mut left = Vec::with_capacity(leaves.len());
    for &v in &leaves {
        let (a, target) = nodes[v].incoming[0];
        let dest = rank[target];
        assert!(dest < leaves.len() && !seen[dest]);
        seen[dest] = true;
        left.push((a, dest));
    }
    assert!(marked_rank < left.len());
    Futures {
        left,
        n,
        mark: marked_rank,
    }
}

#[allow(dead_code)] // Also included as a module by the round-trip check.
fn main() {
    let args: Vec<_> = env::args().collect();
    assert!(
        matches!(args.len(), 3 | 4),
        "support TAPE PRE_RLE_OUT [ORDINARY_COLUMN_OUT]"
    );
    let futures = read(&fs::read(&args[1]).unwrap());
    let (word, period) = futures.marked_output();
    fs::write(&args[2], word).unwrap();
    if args.len() == 4 {
        let last = futures
            .ordinary_column(period)
            .expect("the support has multiple orbits");
        fs::write(&args[3], last).unwrap();
    }
    println!(
        "{} distinct terminal futures, marked period {period}",
        futures.classes()
    );
}
