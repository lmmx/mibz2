use super::*;
use crate::bwt::{InverseBwt, Plain};
use crate::switch_elimination::Switches;

// Re-expand the independent dense LF from every surviving row, including all
// unmarked cycles. Also check local coordinate coverage and the exact word.
fn audit(g: &Network, grammar: &Grammar, l: &[u8], lf: &[usize]) {
    let mut live = vec![false; l.len()];
    for v in &g.vertices {
        let start = v.base as usize;
        for b in &mut live[start..start + v.width as usize] {
            assert!(!*b);
            *b = true;
        }
    }
    let mut targets = vec![NONE; g.bands.len()];
    for v in &g.vertices {
        for side in [IN, OUT] {
            let (mut id, mut prev, mut at, mut count) = (v.first[side], NONE, 0, 0);
            while id != NONE {
                let b = g.bands[id as usize];
                assert_eq!(b.prev[side], prev);
                assert_eq!(g.vertices[b.vertex[side] as usize].base, v.base);
                if side == IN {
                    targets[id as usize] = v.base + at;
                }
                at += b.width;
                count += 1;
                prev = id;
                id = b.next[side];
            }
            assert_eq!((count, at), (v.degree[side], v.width));
        }
    }
    for v in &g.vertices {
        let (mut start, mut id) = (v.base, v.first[OUT]);
        while id != NONE {
            let b = g.bands[id as usize];
            let mut word = Vec::new();
            let mut stack = vec![b.word];
            while let Some(k) = stack.pop() {
                if k < 256 {
                    word.push(k as u8);
                } else {
                    let r = &grammar.rules[(k - 256) as usize];
                    stack.extend([r.right, r.left]);
                }
            }
            assert_eq!(word.len(), grammar.len(b.word) as usize);
            for k in 0..b.width {
                let mut row = (start + k) as usize;
                let mut backward = Vec::new();
                loop {
                    backward.push(l[row]);
                    row = lf[row];
                    if live[row] {
                        break;
                    }
                }
                backward.reverse();
                assert_eq!(backward, word);
                assert_eq!(row as u32, targets[id as usize] + k);
            }
            start += b.width;
            id = b.next[OUT];
        }
    }
}

fn check(l: &[u8], p: u32) {
    let mut count = [0u32; 256];
    let mut implm = Switches::default();
    for &c in l {
        count[c as usize] += 1;
        implm.add_run(c, 1);
    }
    let mut sum = 0;
    let mut cursor = count.map(|n| {
        let a = sum;
        sum += n;
        a as usize
    });
    let lf: Vec<_> = l
        .iter()
        .map(|&c| {
            let j = &mut cursor[c as usize];
            let row = *j;
            *j += 1;
            row
        })
        .collect();
    let (mut out, mut expected) = (Vec::new(), Vec::new());
    Plain::default().invert(l, &count, p, &mut expected);
    let mut g = Network::default();
    let mut mark = g.from_runs(&implm.runs, p);
    let mut words = Grammar::default();
    for _ in 0..4 {
        audit(&g, &words, l, &lf);
        assert_eq!(g.vertices[mark.0 as usize].base + mark.1, p);
        let s = g.sweep(mark.0, &mut words, usize::MAX);
        audit(&g, &words, l, &lf);
        assert!(s.after_e - s.after_v <= s.before_e - s.before_v);
        if s.after_e == s.after_v {
            break;
        }
        let mut next = Network::default();
        mark = g.recode(&mut next, mark);
        assert!(next.bands.len() - next.vertices.len() <= s.after_e - s.after_v);
        g = next;
    }
    g.emit(mark, l.len(), &mut words, &mut out);
    assert_eq!(out, expected);
    implm.invert(l, &count, p, &mut out);
    assert_eq!(out, expected);
}

#[test]
fn all_coordinates_and_words() {
    let mut total = 0;
    for (base, last) in [(2usize, 9), (3, 6)] {
        for n in 1..=last {
            for mut k in 0..base.pow(n) {
                let l: Vec<_> = (0..n)
                    .map(|_| {
                        let v = k % base;
                        k /= base;
                        v as u8
                    })
                    .collect();
                for p in 0..n {
                    check(&l, p);
                    total += 1;
                }
            }
        }
    }
    println!("{total} exhaustive marked columns");
}

#[test]
fn marked_powers_rotations_and_wide_ports() {
    let mut mixed = Vec::new();
    for (c, w) in [(7, 1), (2, 127), (1, 128), (8, 256), (1, 11), (2, 259)] {
        mixed.resize(mixed.len() + w, c);
    }
    for l in [
        mixed,
        vec![0; 701],
        [&[1][..], &vec![0; 799]].concat(),
        (0..768).map(|i| (i % 4) as u8).collect(),
    ] {
        for p in [0, 1, 126, 127, 128, l.len() / 2, l.len() - 1] {
            check(&l, p as u32);
        }
    }
}
