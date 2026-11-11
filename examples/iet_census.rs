//! The interval-exchange census (docs/journal/2026-11-10-interval-exchange.md): how much work an
//! inverse BWT needs when it moves *intervals of rows* instead of rows. It counts; it times
//! nothing, and nothing in the decoder calls it.
//!
//! ```text
//! iet_census FILE.bz2 [EVERY]     one JSON line a full block (stdout)
//! ```
//!
//! **The object.** LF is a translation on each run of L, and the runs' images tile [0, n): LF is
//! an interval exchange on r intervals. An interval of rows that lies inside one run moves
//! rigidly, and its rows all emit the run's byte. So a *piece* (x, w), the rows [x, x + w), is
//! advanced by one LF evaluation whatever its width, for as long as no run boundary falls
//! inside it.
//!
//! **The induction.** Fix J = [a, a + W). Start one piece, J itself, and advance pieces until
//! they return to J, splitting a piece where a run boundary (or an edge of J) falls inside it.
//! The leaves are the pieces of the first-return map R_J, each with its landing place and its
//! *return word*, the bytes emitted on the way. The block is then the orbit of R_J from the
//! walk's first row: one step and one copied word a return (Kac: the return times, weighted by
//! the pieces' widths, sum to n).
//!
//! **What is counted,** for each horizon h = n / W:
//! - `steps`: piece-steps in the induction, each one LF evaluation (the row walk takes n);
//! - `leaves`, `splits`, `scanned`: the pieces of R_J, the children made, the rows read to find
//!   the run boundaries inside a piece that is not rigid;
//! - `words`: the leaves' return words' bytes in all (what an arena of whole words holds);
//! - `returns`, `pre`: R_J's steps on the walk's orbit, and the row steps before it first
//!   enters J.
//!
//! Every block's bytes are rebuilt from the leaves and checked against `bwt::walk`.

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

/// An interval exchange with a word on each piece: piece k is the points [starts[k],
/// starts[k + 1]) (the last ends at `size`), sent to lands[k] + (x - starts[k]), emitting
/// bytes[from[k]..from[k] + len[k]] (in the order the walk emits them: the text reversed)
struct Iet {
    size: u32,
    starts: Vec<u32>,
    lands: Vec<u32>,
    from: Vec<u32>,
    len: Vec<u32>,
    bytes: Vec<u8>,
}

impl Iet {
    fn piece(&self, x: u32) -> usize {
        self.starts.partition_point(|&s| s <= x) - 1
    }
    fn end(&self, k: usize) -> u32 {
        self.starts.get(k + 1).copied().unwrap_or(self.size)
    }
    fn word(&self, k: usize) -> &[u8] {
        &self.bytes[self.from[k] as usize..(self.from[k] + self.len[k]) as usize]
    }
    fn step(&self, k: usize, x: u32) -> u32 {
        self.lands[k] + (x - self.starts[k])
    }
}

struct Node {
    parent: u32,
    /// its own bytes, in the order emitted: a range of the arena
    from: u32,
    len: u32,
}

#[derive(Default, Clone)]
struct Count {
    steps: u64,
    leaves: u64,
    splits: u64,
    words: u64,
    longest: u64,
}

struct Piece {
    x: u32,
    w: u32,
    origin: u32,
    node: u32,
    age: u32,
}

/// The first-return map of `m` to J = [a, a + big_w), as an interval exchange on [0, big_w)
/// whose words are the return words
fn induce(m: &Iet, a: u32, big_w: u32) -> (Iet, Count) {
    let (ja, jb) = (a, a + big_w);
    let mut c = Count::default();
    let mut nodes: Vec<Node> = vec![Node {
        parent: u32::MAX,
        from: 0,
        len: 0,
    }];
    let mut arena: Vec<u8> = Vec::new();
    // (origin, width, land, node)
    let mut leaves: Vec<(u32, u32, u32, u32)> = Vec::new();
    let mut stack = vec![Piece {
        x: a,
        w: big_w,
        origin: a,
        node: 0,
        age: 0,
    }];
    let child = |nodes: &mut Vec<Node>, arena: &Vec<u8>, parent: u32| -> u32 {
        nodes.push(Node {
            parent,
            from: arena.len() as u32,
            len: 0,
        });
        nodes.len() as u32 - 1
    };
    while let Some(mut p) = stack.pop() {
        // (a node's bytes are contiguous in the arena: a piece is walked to its end, or to a
        // split, before another is taken up, and a split starts new nodes)
        nodes[p.node as usize].from = arena.len() as u32;
        nodes[p.node as usize].len = 0;
        loop {
            let k = m.piece(p.x);
            let e = p.x + p.w;
            if e > m.end(k) {
                // not rigid: cut at the pieces' starts inside it
                let mut s = p.x;
                let mut kk = k;
                while s < e {
                    let t = m.end(kk).min(e);
                    let node = child(&mut nodes, &arena, p.node);
                    stack.push(Piece {
                        x: s,
                        w: t - s,
                        origin: p.origin + (s - p.x),
                        node,
                        age: p.age,
                    });
                    c.splits += 1;
                    s = t;
                    kk += 1;
                }
                break;
            }
            let w = m.word(k);
            arena.extend_from_slice(w);
            nodes[p.node as usize].len += w.len() as u32;
            c.steps += 1;
            p.x = m.step(k, p.x);
            p.age += 1;
            let (x, e) = (p.x, p.x + p.w);
            if x >= ja && e <= jb {
                leaves.push((p.origin, p.w, x, p.node));
                c.longest = c.longest.max(p.age as u64);
                break;
            }
            if e <= ja || x >= jb {
                continue;
            }
            // it straddles an edge of J: cut there; the part inside has returned
            let mut s = x;
            for cut in [ja, jb, e] {
                let t = cut.clamp(s, e);
                if t > s {
                    let node = child(&mut nodes, &arena, p.node);
                    let origin = p.origin + (s - x);
                    if s >= ja && t <= jb {
                        leaves.push((origin, t - s, s, node));
                    } else {
                        stack.push(Piece {
                            x: s,
                            w: t - s,
                            origin,
                            node,
                            age: p.age,
                        });
                    }
                    c.splits += 1;
                    s = t;
                }
            }
            break;
        }
    }
    leaves.sort_unstable();
    c.leaves = leaves.len() as u64;
    let mut out = Iet {
        size: big_w,
        starts: vec![],
        lands: vec![],
        from: vec![],
        len: vec![],
        bytes: vec![],
    };
    let mut path = Vec::new();
    let mut at = ja;
    for &(origin, width, land, node) in &leaves {
        assert_eq!(origin, at, "the leaves don't tile J");
        at += width;
        path.clear();
        let mut k = node;
        while k != u32::MAX {
            path.push(k);
            k = nodes[k as usize].parent;
        }
        let from = out.bytes.len() as u32;
        for &k in path.iter().rev() {
            let nd = &nodes[k as usize];
            out.bytes
                .extend_from_slice(&arena[nd.from as usize..(nd.from + nd.len) as usize]);
        }
        out.starts.push(origin - ja);
        out.lands.push(land - ja);
        out.from.push(from);
        out.len.push(out.bytes.len() as u32 - from);
    }
    assert_eq!(at, jb, "the leaves don't cover J");
    c.words = out.bytes.len() as u64;
    (out, c)
}

/// A tower of inductions, each on the middle 1 / ratio of the level below, and the block
/// rebuilt from it: (each level's count, the steps before the walk enters each J, the top
/// map's steps). Panics if the bytes differ from `back` (the block's bytes last to first)
fn tower(
    base: &Iet,
    ratio: u32,
    levels: usize,
    orig_ptr: u32,
    back: &[u8],
) -> (Vec<Count>, u64, u64) {
    let n = back.len();
    let mut maps: Vec<Iet> = Vec::new();
    let mut offs: Vec<u32> = Vec::new();
    let mut counts = Vec::new();
    for _ in 0..levels {
        let m = maps.last().unwrap_or(base);
        let big_w = (m.size / ratio).max(1);
        let a = (m.size - big_w) / 2;
        let (next, c) = induce(m, a, big_w);
        counts.push(c);
        offs.push(a);
        maps.push(next);
    }
    let mut out: Vec<u8> = Vec::with_capacity(n + (1 << 20));
    let mut x = orig_ptr;
    let mut pre = 0u64;
    // down the tower: at each level, steps until the walk is inside the next J
    for lv in 0..levels {
        let m = if lv == 0 { base } else { &maps[lv - 1] };
        let (ja, jb) = (offs[lv], offs[lv] + maps[lv].size);
        while !(x >= ja && x < jb) && out.len() < n {
            let k = m.piece(x);
            out.extend_from_slice(m.word(k));
            x = m.step(k, x);
            pre += 1;
        }
        x = x.wrapping_sub(ja);
    }
    let top = maps.last().unwrap_or(base);
    let mut returns = 0u64;
    while out.len() < n {
        let k = top.piece(x);
        out.extend_from_slice(top.word(k));
        x = top.step(k, x);
        returns += 1;
    }
    out.truncate(n);
    assert!(out == back, "the tower's bytes differ from the walk's");
    (counts, pre, returns)
}

fn census(l: &[u8], orig_ptr: u32, walked: &[u8]) -> String {
    let n = l.len();
    let mut cnt = [0u32; 256];
    for &c in l {
        cnt[c as usize] += 1;
    }
    let mut next = [0u32; 256];
    let mut cum = 0;
    for c in 0..256 {
        next[c] = cum;
        cum += cnt[c];
    }
    // the runs of L: LF's pieces, each emitting its byte
    let mut base = Iet {
        size: n as u32,
        starts: vec![],
        lands: vec![],
        from: vec![],
        len: vec![],
        bytes: vec![],
    };
    for i in 0..n {
        let c = l[i] as usize;
        if i == 0 || l[i] != l[i - 1] {
            base.starts.push(i as u32);
            base.lands.push(next[c]);
            base.from.push(base.bytes.len() as u32);
            base.len.push(1);
            base.bytes.push(l[i]);
        }
        next[c] += 1;
    }
    let runs = base.starts.len();
    let back: Vec<u8> = walked.iter().rev().copied().collect();
    let mut rows = Vec::new();
    for (ratio, levels) in [
        (4u32, 1usize),
        (8, 1),
        (16, 1),
        (32, 1),
        (4, 2),
        (4, 3),
        (4, 4),
        (4, 5),
        (2, 4),
        (2, 6),
        (2, 8),
        (8, 2),
        (8, 3),
        (16, 2),
    ] {
        if (n as u64) < (ratio as u64).pow(levels as u32) * 64 {
            continue;
        }
        let (counts, pre, returns) = tower(&base, ratio, levels, orig_ptr, &back);
        let per: Vec<String> = counts
            .iter()
            .map(|c| {
                format!(
                    "[{},{},{},{},{}]",
                    c.steps, c.leaves, c.splits, c.words, c.longest
                )
            })
            .collect();
        rows.push(format!(
            "{{\"ratio\":{ratio},\"levels\":{levels},\"per\":[{}],\"pre\":{pre},\"returns\":{returns}}}",
            per.join(",")
        ));
    }
    format!(
        "{{\"n\":{n},\"runs\":{runs},\"towers\":[{}]}}",
        rows.join(",")
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1]).expect("read");
    let every: usize = args.get(2).map_or(1, |s| s.parse().unwrap());
    let min: usize = args.get(3).map_or(65536, |s| s.parse().unwrap());
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut ll, mut counts, mut tt) = (Vec::new(), [0u32; 256], TVector::new());
    let mut syms = Vec::new();
    let mut walked = Vec::new();
    let mut i = 0;
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        if ll.len() >= min {
            if i % every == 0 {
                build(&ll, &counts, &mut tt);
                walked.clear();
                walk(&tt, b.orig_ptr, &mut walked);
                println!("{}", census(&ll, b.orig_ptr, &walked));
            }
            i += 1;
        }
        syms = b.syms;
    }
}
