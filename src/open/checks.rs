use super::*;

// Independent expansion of every induced arrow, including arrows on cycles
// other than the marked one. A final string comparison alone can miss an
// incorrect arrow between two equally spelled excursions.
fn audit<const INDUCE: bool, const QUEUE: bool>(
    k: &mut Kernel,
    l: &[u8],
    p: usize,
    threshold: usize,
) {
    let n = l.len();
    let records = records_of(l);
    let (s, mark, bits) =
        k.construct::<INDUCE, QUEUE>(&records, n as u32, p as u32, threshold as u32);
    let mut counts = [0usize; 256];
    let mut keep = vec![false; n];
    let mut i = 0;
    while i < n {
        let mut end = i + 1;
        while end < n && l[end] == l[i] {
            end += 1;
        }
        keep[i..end].fill(end - i <= threshold);
        i = end;
    }
    keep[p] = true;
    for &c in l {
        counts[c as usize] += 1;
    }
    let sigma = counts.iter().filter(|&&w| w > 0).count();
    assert!(s.bands <= 3 * s.thick + sigma);
    let mut total = 0;
    let mut pos = counts.map(|width| {
        let a = total;
        total += width;
        a
    });
    let lf: Vec<_> = l
        .iter()
        .map(|&c| {
            let at = &mut pos[c as usize];
            let old = *at;
            *at += 1;
            old
        })
        .collect();
    let mut m = 0;
    let rank: Vec<_> = keep
        .iter()
        .map(|&v| {
            let j = m;
            m += usize::from(v);
            j
        })
        .collect();
    assert_eq!(m, s.retained);
    assert_eq!(rank[p], mark as usize);
    let mask = (1 << bits) - 1;
    for x in (0..n).filter(|&x| keep[x]) {
        let mut w = vec![l[x]];
        let mut y = lf[x];
        while !keep[y] {
            w.push(l[y]);
            y = lf[y];
        }
        w.reverse();
        let edge = k.edges[rank[y]];
        assert_eq!(rank[x], (edge & mask) as usize);
        let c = if k.extra.is_empty() {
            edge >> bits
        } else {
            k.extra[rank[y]]
        };
        if c < 256 {
            assert_eq!(w, [c as u8]);
        } else {
            let id = (c - 256) as usize;
            assert_eq!(
                w,
                k.spelling[k.offsets[id] as usize..k.offsets[id + 1] as usize]
            );
        }
    }
}

#[test]
fn all_port_arrows() {
    let mut k = Kernel::default();
    let mut cases = 0;
    for (base, len) in [(2usize, 8usize), (3, 5)] {
        for n in 1..=len {
            for mut z in 0..base.pow(n as u32) {
                let l: Vec<_> = (0..n)
                    .map(|_| {
                        let c = z % base;
                        z /= base;
                        c as u8
                    })
                    .collect();
                for p in 0..n {
                    for t in [0, 1, 2, 4, 16] {
                        audit::<false, false>(&mut k, &l, p, t);
                        audit::<true, false>(&mut k, &l, p, t);
                        audit::<false, true>(&mut k, &l, p, t);
                        audit::<true, true>(&mut k, &l, p, t);
                        cases += 1;
                    }
                }
            }
        }
    }
    println!("{cases} column/mark/threshold triples, four complete arrow audits each");
}

#[test]
fn capped_runs_and_shifted_marks() {
    let mut k = Kernel::default();
    let mut l = Vec::new();
    for (b, w) in [
        (3, 1),
        (1, 127),
        (2, 128),
        (0, 508),
        (1, 19),
        (3, 260),
        (2, 1),
    ] {
        l.resize(l.len() + w, b);
    }
    for p in [0, 1, 2, 127, 128, 129, 255, 256, 757, l.len() - 1] {
        for t in [0, 1, 16, 126, 127, 128, 259, 260, 800] {
            audit::<false, true>(&mut k, &l, p, t);
            audit::<true, false>(&mut k, &l, p, t);
        }
    }
}

#[test]
fn residual_segments_on_short_cycles() {
    use crate::bwt::{InverseBwt, Plain};
    let mut kernel = Kernel::default();
    let mut plain = Plain::default();
    let (mut actual, mut reference) = (Vec::new(), Vec::new());
    let mut seed = 12383217u32;
    for n in [1, 31, 127, 128, 129, 385, 1025] {
        for alphabet in [1, 2, 11, 256] {
            let l: Vec<_> = (0..n)
                .map(|_| {
                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    ((seed >> 8) % alphabet) as u8
                })
                .collect();
            let mut counts = [0; 256];
            for &c in &l {
                counts[c as usize] += 1;
            }
            let recs = records_of(&l);
            for p in [0, (n / 2) as u32, (n - 1) as u32] {
                plain.invert(&l, &counts, p, &mut reference);
                for limit in [0, 1, 4, 16, 1025] {
                    kernel.run::<true, true>(&recs, n, p, limit, &mut actual);
                    assert_eq!(actual, reference);
                }
            }
        }
    }
}
