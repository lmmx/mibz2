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
