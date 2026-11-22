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
        } else if k.lens[c as usize] != LONG {
            assert_eq!(w, k.inline16[c as usize][..usize::from(k.lens[c as usize])]);
        } else {
            assert_eq!(w, long_word(&k.inline16[c as usize], &k.spelling));
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
                &edges, &extra, &spelling, &inline16, &lens, n, mark, bits, &mut out,
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

#[test]
fn thin_and_thick_sum_to_each_symbols_prefix() {
    let mut kernel = Kernel::default();
    let mut out = [0; 1024];
    let mut pass = kernel.begin(&mut out, u32::MAX, usize::MAX);
    // The marked run contributes two thick fragments and one thin singleton.
    for (byte, start, width) in [(7, 0, 2), (9, 2, 6), (7, 8, 10), (9, 18, 1)] {
        pass.take::<true>(byte, start, width, 12, 3);
    }
    assert_eq!((pass.f.thin[7], pass.f.thick_rows[7]), (3, 9));
    assert_eq!((pass.f.thin[9], pass.f.thick_rows[9]), (1, 6));
    assert_eq!(
        pass.edges[..3 * pass.ns]
            .chunks_exact(3)
            .map(|s| s[1])
            .collect::<Vec<_>>(),
        [0, 2, 7]
    );
}

#[test]
fn strip_prefix_stops_at_dense_cap() {
    let mut kernel = Kernel::default();
    let mut out = [0; 16];
    let mut pass = kernel.begin(&mut out, u32::MAX, 1);
    pass.take::<false>(255, 0, 511, 1000, 16);
    pass.take::<false>(3, 511, 512, 1000, 16);
    assert!(pass.ns > pass.cap_strips);
    assert_eq!(&pass.edges[..], &[0, 0, (511 << 8) | 255]);
}

#[test]
fn maximum_band_fields_decode_independently() {
    let band = Band::new(900_000, 899_999, 0x00ff_abcd);
    assert_eq!(
        (band.end(), band.dst(), band.code()),
        (900_000, 899_999, 0x00ff_abcd)
    );
}

#[test]
fn long_inline_slot_keeps_both_offsets() {
    let bytes: Vec<_> = (0..80_000).map(|i| (i % 251) as u8).collect();
    let slot = long_slot(65_535, 65_567);
    assert_eq!(long_word(&slot, &bytes), &bytes[65_535..65_567]);
}

#[test]
fn dictionary_reuse_matches_each_blocks_plain_inverse() {
    let mut kernel = Kernel::default();
    let mut out = Vec::new();
    let mut g = super::b5::Lcg(43);
    let large = super::b5::runs_column(&mut g, 120_000, 6, 18);
    for l in [&large[..], &large[..97], &large[..]] {
        let r = Runs::of_column(l);
        let stats = kernel.run::<true, true>(r.records(), l.len(), 53, LIMIT, &mut out);
        assert!(out == super::b5::plain(l, 53));
        if l.len() > 100 {
            assert!(kernel.lens.len() - 256 < stats.words);
        }
    }
}

#[test]
fn full_block_reserves_spelling_once() {
    let mut kernel = Kernel::default();
    let mut out = Vec::new();
    for n in [65_535, 65_536, 65_537] {
        let r = Runs::of_column(&(0..n).map(|i| (i % 2) as u8).collect::<Vec<_>>());
        kernel.run::<true, true>(r.records(), n, 0, LIMIT, &mut out);
        // Alternating thin rows need no long words.
        if n >= 65_536 {
            assert_eq!(kernel.spelling.capacity(), 48 * 1024);
        }
    }
}

#[test]
fn two_ended_emission_meets_on_odd_even_and_singleton_cycles() {
    let mut inline = vec![];
    let mut lens = vec![];
    begin_inline(0, &mut inline, &mut lens);
    for cycle in 1..=7usize {
        let n = 73;
        let bits = 4;
        let edges: Vec<_> = (0..cycle)
            .map(|i| {
                ((65 + i as u32) << bits) | (((i + 1) % cycle) ^ ((i + cycle - 1) % cycle)) as u32
            })
            .collect();
        let mut out = vec![];
        emit_bidirectional(
            &edges,
            &[],
            &inline,
            &lens,
            n,
            0,
            (cycle - 1) as u32,
            bits,
            &mut out,
        );
        assert!(out.into_iter().eq((0..n).map(|i| 65 + (i % cycle) as u8)));
    }
}

/// A full nonzero tail still belongs to its original symbol. Its next ranks
/// spill into a new chunk, even when the two in-output chunks are adjacent.
#[test]
fn full_nonzero_tail_links_to_its_overflow_chunk() {
    let mut out = [0; 2048];
    let mut spill = outq::Spill::default();
    let mut queues = outq::Queues::new(&mut out, &mut spill, u32::MAX);
    let thin = [0; 256];
    queues.push(1, 0, 1, &thin);
    queues.push(2, 1, 496, &thin);
    queues.push(2, 497, 16, &thin);
    assert_eq!(queues.chunks(), 2);
    queues.push(2, 513, 2, &thin);
    assert_eq!(queues.chunks(), 3);
    let heads = queues.heads();
    let mut decoded = [0; 514];
    outq::Reader::new(&out, &spill, &heads, 2).wire(&mut decoded, 0);
    assert!(decoded.into_iter().eq(1..=514));
}

#[test]
fn burst_threshold_switches_the_link_representation() {
    let mut kernel = Kernel::default();
    let mut actual = Vec::new();
    for m in [burst::MIN_ROWS - 1, burst::MIN_ROWS, burst::MIN_ROWS + 1] {
        let column: Vec<u8> = (0..m).map(|i| (i % 251) as u8).collect();
        let mark = (m / 3) as u32;
        let counts = kernel.run::<true, true>(&records_of(&column), m, mark, LIMIT, &mut actual);
        assert_eq!(counts.retained, m);
        assert_eq!(kernel.predecessor.is_some(), m < burst::MIN_ROWS);
        assert_eq!(kernel.word_len.is_empty(), m < burst::MIN_ROWS);
        assert_eq!(actual, b5::plain(&column, mark));
    }
}

#[test]
fn burst_repairs_cut_starts_and_repeats_only_the_marked_cycle() {
    let m = burst::MIN_ROWS;
    let bits = usize::BITS - (m - 1).leading_zeros();
    let mark = 311;
    let mut kernel = Kernel {
        spelling: (180..203).collect(),
        ..Kernel::default()
    };
    inline_words(
        &kernel.spelling,
        &[0, 23],
        &mut kernel.inline16,
        &mut kernel.lens,
    );
    // Two disjoint cycles. Long words force the mixed-lane path. Unequal
    // distances between cuts let later lanes overrun earlier finished tasks.
    let mut rows: Vec<usize> = (1..m).step_by(2).collect();
    let mut seed = 51u32;
    for i in (1..rows.len()).rev() {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        rows.swap(i, seed as usize % (i + 1));
    }
    let start = rows.iter().position(|&row| row == mark).unwrap();
    rows.rotate_left(start);
    kernel.edges = (0..m as u32).collect();
    let mut period = Vec::new();
    for (i, &row) in rows.iter().enumerate() {
        let code = if i % 37 == 0 {
            period.extend_from_slice(&kernel.spelling);
            256
        } else {
            let byte = (row % 251) as u8;
            period.push(byte);
            u32::from(byte)
        };
        kernel.edges[row] = (code << bits) | rows[(i + 1) % rows.len()] as u32;
    }
    let mut actual = vec![0xa5; 3 * m];
    for n in [period.len() - 7, period.len(), 2 * period.len() + 31] {
        assert!(kernel.emit_burst(m, n, mark as u32, bits, &mut actual));
        assert!(actual
            .iter()
            .copied()
            .eq(period.iter().copied().cycle().take(n)));
    }
    // An isolated marked row supplies too few tasks. A dictionary length
    // outside the scout's range also goes through the ordinary forward walk.
    assert!(!kernel.emit_burst(m, m, 0, bits, &mut actual));
    kernel.spelling.resize(70_000, 99);
    kernel.inline16[256] = long_slot(0, 70_000);
    assert!(!kernel.emit_burst(m, m, mark as u32, bits, &mut actual));
}
