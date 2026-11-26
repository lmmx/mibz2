//! Exact byte oracle over arbitrary residual permutations, independently of BWT construction.
mod sparse;
use sparse::{oracle_write, paid, Plan, Table};
fn perms(next: &mut [u32], at: usize, count: &mut usize) {
    if at < next.len() {
        for j in at..next.len() {
            next.swap(at, j);
            perms(next, at + 1, count);
            next.swap(at, j);
        }
        return;
    }
    let m = next.len();
    let bits = (usize::BITS - (m - 1).leading_zeros()).max(1);
    let words = [vec![85, 0], vec![255, 128, 1, 5, 16], vec![172]];
    for label_bits in 0..1usize << m {
        let labels: Vec<u32> = (0..m)
            .map(|i| {
                if label_bits >> i & 1 == 0 {
                    ((i * 63 + 7) & 255) as u32
                } else {
                    256 + (i % 3) as u32
                }
            })
            .collect();
        for mark in 0..m {
            let mut want = Vec::new();
            let mut row = mark;
            loop {
                let code = labels[row];
                if code < 256 {
                    want.push(code as u8);
                } else {
                    want.extend_from_slice(&words[(code - 256) as usize]);
                }
                row = next[row] as usize;
                if row == mark {
                    break;
                }
            }
            let p = want.len();
            let total = 2 * p + p / 2;
            let expected: Vec<u8> = want.iter().copied().cycle().take(total).collect();
            for wide in [false, true] {
                let mut t = Table {
                    n: total,
                    mark: mark as u32,
                    bits,
                    edges: next.to_vec(),
                    extra: Vec::new(),
                    offsets: vec![0, 2, 7, 8],
                    spelling: words.iter().flatten().copied().collect(),
                };
                if wide {
                    t.extra = labels.clone();
                } else {
                    for (e, &c) in t.edges.iter_mut().zip(&labels) {
                        *e |= c << bits;
                    }
                }
                let mut z = Plan::default();
                let mut actual = Vec::new();
                macro_rules! check {
                    ($lanes:literal,$shift:literal) => {{
                        let oracle = t.oracle($shift);
                        paid::<$lanes, $shift>(&mut z, &t, &mut actual);
                        assert!(
                            actual == expected && z.period as usize == p && z.jobs == oracle.jobs
                        );
                        oracle_write::<$lanes>(&oracle, &t, &mut actual);
                        assert!(actual == expected);
                        *count += 1;
                    }};
                }
                check!(4, 1);
                check!(8, 2);
                check!(8, 7);
            }
        }
    }
}
fn main() {
    let mut count = 0;
    for m in 1..=5 {
        perms(&mut (0..m).collect::<Vec<_>>(), 0, &mut count);
    }
    println!(
        "{count} permutation/label/mark/width/landmark checks agree with independent marked bytes"
    );
}
