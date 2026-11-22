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
    assert_eq!((pass.queues.thin_of(7), pass.f.thick_rows[7]), (3, 9));
    assert_eq!((pass.queues.thin_of(9), pass.f.thick_rows[9]), (1, 6));
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
        (band.width(), band.dst(), band.code()),
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
    queues.push_thin(1, 0, 1);
    queues.push_thin(2, 1, 496);
    queues.push_thin(2, 497, 16);
    assert_eq!(queues.chunks(), 2);
    queues.push_thin(2, 513, 2);
    assert_eq!(queues.chunks(), 3);
    let heads = queues.heads();
    let mut decoded = [0; 514];
    outq::Reader::new(&out, &spill, &heads, 2).wire(&mut decoded, 0);
    assert!(decoded.into_iter().eq(1..=514));
}

/// Records that cut a run short of `CAP` (127) rows, which no decoded block's records do
/// (`runs.rs`'s `front`): the first pass's `Cap127` takes a short record as a whole thin run
/// without looking at the next, so a run is split. Both passes split it alike (`runs_of`), and
/// every instance decodes the column exactly, at limits on both sides of `CAP`. The columns'
/// runs are cut at random lengths from 1 to 127 rows.
#[test]
fn records_cut_short_of_cap() {
    use super::b5::{plain, runs_column, Lcg};
    let mut g = Lcg(0x6375_7473);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    let mut split = 0;
    for &(n, alphabet, mean) in &[
        (1usize, 1u32, 1u32),
        (40, 2, 8),
        (300, 3, 30),
        (3000, 5, 100),
        (5000, 2, 400),
        (70_000, 17, 16),
        (200_000, 30, 12),
    ] {
        let l = runs_column(&mut g, n, alphabet, mean);
        let mut recs = Vec::new();
        let mut i = 0;
        while i < n {
            let mut j = i;
            while j < n && l[j] == l[i] {
                j += 1;
            }
            let mut left = j - i;
            while left > 0 {
                let len = left.min(1 + g.below(127) as usize);
                split += usize::from(len < left && len < 127);
                recs.push((u16::from(l[i]) << 8) | len as u16);
                left -= len;
            }
            i = j;
        }
        for p in [0, g.below(n as u32), n as u32 - 1] {
            let want = plain(&l, p);
            for limit in [0, 1, 16, 40, 126, 127, 128, 1025] {
                k.run::<true, true>(&recs, n, p, limit, &mut out);
                assert!(out == want, "n {n}, row {p}, limit {limit}");
                if n <= 5000 {
                    for (induce, queue) in [(true, false), (false, true), (false, false)] {
                        match (induce, queue) {
                            (true, false) => k.run::<true, false>(&recs, n, p, limit, &mut out),
                            (false, true) => k.run::<false, true>(&recs, n, p, limit, &mut out),
                            _ => k.run::<false, false>(&recs, n, p, limit, &mut out),
                        };
                        assert!(
                            out == want,
                            "{induce} {queue}: n {n}, row {p}, limit {limit}"
                        );
                    }
                }
            }
        }
    }
    assert!(split > 1000, "{split} runs cut short");
}

/// The queues' fast push (`Pt64`, `OneBr`, `S1Trim`) and its slow one, read back in order:
/// pushes of 1 to 16 ranks (and of 17 to 40, which only the slow path takes) on three bytes,
/// across every chunk's end, across the epochs' ends (the fast push stops at a low half of
/// 65,520 and up), in `out` and in overflow chunks (`out` of one, of 120 and of 4,000 chunks
/// and a part)
#[test]
fn queue_pushes_read_back_across_chunks_and_epochs() {
    use super::b5::Lcg;
    let mut g = Lcg(0x7075_7368);
    for bytes in [1_040usize, 120 * 1024 + 7, 4_000 * 1024 + 500] {
        let mut out = vec![0u8; bytes];
        let mut spill = outq::Spill::default();
        let mut queues = outq::Queues::new(&mut out, &mut spill, u32::MAX);
        let mut want: [Vec<u32>; 3] = Default::default();
        let mut m = 0u32;
        while m < 3 * 65_536 + 700 {
            let a = g.below(3) as usize;
            let w = if g.below(20) == 0 {
                17 + g.below(24)
            } else {
                1 + g.below(16)
            };
            queues.push_thin(a, m, w);
            want[a].extend(m..m + w);
            m += w;
            assert_eq!(queues.thin_of(a) as usize, want[a].len());
        }
        assert!(!queues.over());
        let thin = queues.extract_thin();
        let heads = queues.heads();
        for a in 0..3 {
            assert_eq!(thin[a] as usize, want[a].len());
            let mut got = vec![0u32; want[a].len()];
            let mut reader = outq::Reader::new(&out, &spill, &heads, a);
            // read in spans of 1 to 40, as the bands read them
            let mut at = 0;
            while at < got.len() {
                let span = (1 + g.below(40) as usize).min(got.len() - at);
                reader.wire(&mut got[at..at + span], 0);
                at += span;
            }
            assert!(got == want[a], "{bytes} bytes, byte {a}");
        }
        assert!(bytes > 1 << 20 || spill.overflow() > 0);
    }
}

/// `wire_xor_nomark` (`Wire16`: 16, 8, 4 and 1 at a time) writes what `wire_xor` writes,
/// on spans of 1 to 40 ranks read across chunks and epochs, in `out` and in overflow chunks,
/// with rows that are both a span's and a rank's (the XOR's order)
#[test]
fn wire_without_the_mark_matches_wire() {
    use super::b5::Lcg;
    let mut g = Lcg(0x6e6f_6d6b);
    for bytes in [1_040usize, 300 * 1024] {
        let mut out = vec![0u8; bytes];
        let mut spill = outq::Spill::default();
        let mut queues = outq::Queues::new(&mut out, &mut spill, u32::MAX);
        let mut m = 0u32;
        let mut count = [0usize; 2];
        while m < 2 * 65_536 + 300 {
            let a = g.below(2) as usize;
            let w = 1 + g.below(16);
            queues.push_thin(a, m, w);
            count[a] += w as usize;
            m += w;
        }
        let heads = queues.heads();
        let rows = m as usize;
        let (mut plain, mut fast) = (vec![0u32; rows], vec![0u32; rows]);
        let mut dst = 0;
        for (a, &queued) in count.iter().enumerate() {
            let mut r1 = outq::Reader::new(&out, &spill, &heads, a);
            let mut r2 = outq::Reader::new(&out, &spill, &heads, a);
            let mut left = queued;
            while left > 0 {
                let span = (1 + g.below(40) as usize).min(left);
                let pack = g.below(64) << 20;
                let mut before = u32::MAX - 1;
                r1.wire_xor(&mut plain, dst, span, pack, u32::MAX, &mut before);
                assert_eq!(before, u32::MAX - 1);
                r2.wire_xor_nomark(&mut fast, dst, span, pack);
                dst += span;
                left -= span;
            }
        }
        assert_eq!(dst, rows);
        assert!(plain == fast, "{bytes} bytes");
    }
}
