use super::*;
#[test]
fn arbitrary_marks_and_overlaps() {
    let mut candidate = Candidate::default();
    let mut out = Vec::new();
    let (mut covered, mut absent) = (0, 0);
    for n in 1..=9 {
        for column in 0..1u32 << n {
            let ll: Vec<_> = (0..n).map(|i| ((column >> i) & 1) as u8).collect();
            let caps: Vec<_> = ll.iter().map(|&b| ((b as u16) << 8) | 1).collect();
            let mut psi: Vec<_> = (0..n).collect();
            psi.sort_by_key(|&i| ll[i]);
            for p in 0..n {
                let mut x = p;
                let future: Vec<_> = (0..n + 4)
                    .map(|_| {
                        x = psi[x];
                        ll[x]
                    })
                    .collect();
                for width in 1..=4 {
                    for bits in 0..1u32 << width {
                        let pattern: Vec<_> = (0..width).map(|i| ((bits >> i) & 1) as u8).collect();
                        let expected = (0..n).any(|i| future[i..].starts_with(&pattern));
                        let took =
                            candidate.for_pattern::<true>(&caps, n, p as u32, &mut out, &pattern);
                        assert_eq!(took, expected, "{ll:?}, {p}, {pattern:?}");
                        if took {
                            assert_eq!(out, future[..n]);
                            covered += 1;
                        } else {
                            absent += 1;
                        }
                    }
                }
            }
        }
    }
    println!("{covered} exact covered marks, {absent} exact section-free refusals");
}
#[test]
#[ignore]
fn corpus() {
    use mibz2::stream::{Event, Scanner, SliceSource};
    use std::io::Write;
    let data = std::fs::read(std::env::var("F2_CORPUS").unwrap()).unwrap();
    let mut results =
        std::io::BufWriter::new(std::fs::File::create(std::env::var("F2_CSV").unwrap()).unwrap());
    writeln!(results,"stream,rows,pattern,took,run_lookups,hits,certificate_writes,bands,dict_bytes,token_rows,workspace_capacity").unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let (mut c, mut r, mut oracle) = (Candidate::default(), Runs::default(), Segmented::default());
    let (mut out, mut want) = (Vec::new(), Vec::new());
    let kernels = Kernels::detect();
    let mut totals = [(0, 0); 2];
    let mut blocks = 0;
    while let Ok(Event::Block { index, symbols, .. }) = scanner.next(Vec::new()) {
        let n = crate::runs::front(&symbols, &mut r, kernels).unwrap();
        if n < 65536 {
            continue;
        }
        assert!(oracle.invert_runs(&r, n, symbols.orig_ptr, &mut want, kernels));
        for (i, (name, pattern)) in [("quote", &b"\""[..]), ("compound", PATTERN)]
            .into_iter()
            .enumerate()
        {
            let took = c.for_pattern::<false>(r.records(), n, symbols.orig_ptr, &mut out, pattern);
            if took {
                assert_eq!(out, want);
                totals[i].0 += 1;
                totals[i].1 += n;
            }
            let capacity = c.cache.capacity() * std::mem::size_of::<Cert>()
                + c.path.capacity() * std::mem::size_of::<Step>()
                + c.runs.capacity() * std::mem::size_of::<Run>()
                + 4 * (c.page_first.capacity() + c.links.capacity() + c.offsets.capacity())
                + 8 * c.starts.capacity()
                + c.dictionary.capacity();
            writeln!(
                results,
                "{index},{n},{name},{},{},{},{},{},{},{},{capacity}",
                u8::from(took),
                if took { c.stats[0] } else { 0 },
                if took { c.stats[1] } else { 0 },
                if took { c.stats[2] } else { 0 },
                if took { c.offsets.len() - 1 } else { 0 },
                if took { c.dictionary.len() } else { 0 },
                if took { c.links.len() } else { 0 }
            )
            .unwrap();
        }
        blocks += 1;
    }
    assert_eq!(blocks, 233);
    println!("{blocks} large blocks; verified (blocks, inverse bytes) for quote and compound: {totals:?}");
}

#[test]
fn every_band_and_arrow() {
    let mut candidate = Candidate::default();
    let (mut patterns, mut arrows, mut bands) = (0, 0, 0);
    for n in 1..=9 {
        for seed in 0..1u32 << n {
            let col: Vec<_> = (0..n).map(|i| ((seed >> i) & 1) as u8).collect();
            let caps: Vec<_> = col.iter().map(|&b| u16::from(b) << 8 | 1).collect();
            let mut psi: Vec<_> = (0..n).collect();
            psi.sort_by_key(|&i| col[i]);
            let mut lf = vec![0; n];
            for (y, &s) in psi.iter().enumerate() {
                lf[s] = y;
            }
            for width in 1..=4 {
                for word in 0..1u32 << width {
                    let pat: Vec<_> = (0..width).map(|i| ((word >> i) & 1) as u8).collect();
                    let rows: Vec<_> = (0..n)
                        .filter(|&r| {
                            let mut x = r;
                            pat.iter().all(|&b| {
                                x = psi[x];
                                col[x] == b
                            })
                        })
                        .collect();
                    let section = candidate.prepare(&caps, n as u32, &pat);
                    if rows.is_empty() {
                        assert!(section.is_none());
                        continue;
                    }
                    let (s, m) = section.unwrap();
                    assert_eq!(rows, (s as usize..(s + m) as usize).collect::<Vec<_>>());
                    assert!(candidate.bands(s, m));
                    let bits = u32::BITS - (m - 1).leading_zeros();
                    let mask = (1u32 << bits) - 1;
                    let mut prior = Vec::new();
                    let mut count = 0;
                    for (k, &start) in rows.iter().enumerate() {
                        let mut x = start;
                        let mut spelling = Vec::new();
                        loop {
                            spelling.push(col[x]);
                            x = lf[x];
                            if rows.contains(&x) {
                                break;
                            }
                        }
                        spelling.reverse();
                        if spelling != prior {
                            count += 1;
                            prior = spelling.clone();
                        }
                        let link = candidate.links[x - s as usize];
                        assert_eq!((link & mask) as usize, k);
                        let code = (link >> bits) as usize;
                        assert_eq!(code, count - 1);
                        assert_eq!(
                            candidate.dictionary[candidate.offsets[code] as usize
                                ..candidate.offsets[code + 1] as usize],
                            spelling
                        );
                    }
                    assert_eq!(candidate.offsets.len(), count + 1);
                    patterns += 1;
                    arrows += m;
                    bands += count;
                }
            }
        }
    }
    println!("verified all {arrows} induced arrows and all {bands} maximal bands of {patterns} nonempty word sections");
}
