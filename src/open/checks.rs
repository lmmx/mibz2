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
    let mut out = Vec::new();
    let (s, mark, bits) =
        k.construct::<INDUCE, QUEUE>(&records, n as u32, p as u32, threshold as u32, &mut out);
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
    // the elimination with `Wide` links, which real blocks take only past 21,844 thick strips
    let mut wide = Kernel::default();
    wide.endpoint.force_wide = true;
    wide.endpoint.tight = true;
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
                        audit::<true, false>(&mut wide, &l, p, t);
                        audit::<true, true>(&mut wide, &l, p, t);
                        cases += 1;
                    }
                }
            }
        }
    }
    println!("{cases} column/mark/threshold triples, six complete arrow audits each");
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
            k.endpoint.force_wide = true;
            audit::<true, true>(&mut k, &l, p, t);
            k.endpoint.force_wide = false;
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

/// The 16-byte emit against the walk as shipped before it (`emit_reference`), on arbitrary
/// walks over arbitrary dictionaries: literals and words of 0 to 40 bytes (across the
/// 16-byte line and the `LONG` path), narrow and wide (`extra`) codes, several cycles, marks
/// on and off a cycle, `n` above and below the cycle's length (the clamp and `repeat`), and
/// an output buffer left longer or shorter than `n` by an earlier block. Either both panic
/// (an empty walk, `repeat`'s assert) or both give the same bytes.
#[test]
fn emit_matches_reference() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let mut seed = 0x7531_6e65u64;
    let mut next = |below: usize| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 33) as usize) % below.max(1)
    };
    let (mut inline16, mut lens) = (Vec::new(), Vec::new());
    let mut out = Vec::new();
    let (mut cases, mut panics) = (0usize, 0usize);
    for _ in 0..200_000 {
        let words = next(12);
        let empty = next(50) == 0;
        let mut spelling = Vec::new();
        let mut offsets = vec![0u32];
        for _ in 0..words {
            let len = match next(4) {
                0 => 2 + next(15),
                1 => 15 + next(4),
                2 => 17 + next(24),
                _ => usize::from(!empty) * (1 + next(40)),
            };
            spelling.extend((0..len).map(|_| next(256) as u8));
            offsets.push(spelling.len() as u32);
        }
        let m = 1 + next(40);
        let bits = (usize::BITS - (m - 1).leading_zeros()).max(1);
        let wide = next(4) == 0;
        let codes = 256 + words;
        let mut edges = vec![0u32; m];
        let mut extra = Vec::new();
        // a permutation of the rows (cycles), or with some chance an arbitrary map
        let mut perm: Vec<u32> = (0..m as u32).collect();
        for i in (1..m).rev() {
            perm.swap(i, next(i + 1));
        }
        let arbitrary = next(5) == 0;
        for (r, e) in edges.iter_mut().enumerate() {
            let to = if arbitrary { next(m) as u32 } else { perm[r] };
            let code = if next(3) == 0 { next(256) } else { next(codes) } as u32;
            if wide {
                extra.push(code);
                *e = to;
            } else {
                *e = (code << bits) | to;
            }
        }
        let mark = next(m) as u32;
        let span = if next(2) == 0 { 64 } else { 700 };
        let n = 1 + next(span);
        let dirty = next(3);
        let reference = catch_unwind(AssertUnwindSafe(|| {
            let mut r = vec![0xEE; dirty];
            emit_reference(&edges, &extra, &spelling, &offsets, n, mark, bits, &mut r);
            r
        }))
        .ok();
        // an earlier block's bytes, longer or shorter than this one
        out.clear();
        out.resize(next(2 * n + 40), 0xA5);
        let got = catch_unwind(AssertUnwindSafe(|| {
            inline_words(&spelling, &offsets, &mut inline16, &mut lens);
            emit(
                &edges, &extra, &spelling, &offsets, &inline16, &lens, n, mark, bits, &mut out,
            );
        }))
        .ok()
        .map(|()| out.clone());
        panics += usize::from(reference.is_none());
        assert_eq!(got, reference, "case {cases}");
        cases += 1;
    }
    println!("{cases} walks, {panics} empty (both panic)");
}
