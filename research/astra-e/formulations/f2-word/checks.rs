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
    let file = std::fs::read(std::env::var("F2_CORPUS").unwrap()).unwrap();
    let mut csv =
        std::io::BufWriter::new(std::fs::File::create(std::env::var("F2_CSV").unwrap()).unwrap());
    writeln!(csv,"stream,rows,primary,caps,took,run_pages,run_count,nodes,node_bytes,edges,selectors,words,dict_bytes,merged,root_runs,token_rows,lookups,merge_steps,token_visits,rewind,scratch_capacity_bytes").unwrap();
    let mut scan = Scanner::new(SliceSource(&file));
    let mut c = Candidate::default();
    let mut r = Runs::default();
    let mut reference = Segmented::default();
    let (mut out, mut want) = (Vec::new(), Vec::new());
    let (mut count, mut own, mut checked) = (0, 0, 0);
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let k = Kernels::detect();
        let n = crate::runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 {
            continue;
        }
        let took = c.attempt::<false>(r.records(), n, symbols.orig_ptr, &mut out);
        if took {
            assert!(reference.invert_runs(&r, n, symbols.orig_ptr, &mut want, k));
            assert_eq!(out, want);
            own += 1;
            checked += n;
        }
        let cap = c.runs.capacity() * std::mem::size_of::<Run>()
            + c.page_first.capacity() * 4
            + c.starts.capacity() * 8
            + c.nodes.capacity() * std::mem::size_of::<Node>()
            + c.edges.capacity()
            + c.selectors.capacity() * 8
            + c.words.capacity() * 8
            + c.dict.capacity()
            + c.merged.capacity() * 8
            + c.links.capacity() * 4;
        if took {
            writeln!(
                csv,
                "{index},{n},{},{},1,{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{cap}",
                symbols.orig_ptr,
                r.records().len(),
                c.page_first.len(),
                c.runs.len() - 1,
                c.nodes.len(),
                std::mem::size_of::<Node>(),
                c.edges.len(),
                c.selectors.len(),
                c.words.len(),
                c.dict.len(),
                c.merged.len(),
                c.merged.len() - c.nodes[0].out as usize,
                c.stats[4],
                c.stats[0],
                c.stats[1],
                c.stats[2],
                c.stats[3]
            )
            .unwrap();
        } else {
            writeln!(
                csv,
                "{index},{n},{},{},0,,,,,,,,,,,,,,,,{cap}",
                symbols.orig_ptr,
                r.records().len()
            )
            .unwrap();
        }
        count += 1;
    }
    println!("{own}/{count} large Wikidata blocks, {checked} checked inverse bytes");
    assert_eq!(count, 233);
}
