use super::*;

#[test]
fn union_marks() {
    let mut decoder = Candidate::default();
    let mut out = Vec::new();
    let mut takes = 0;
    let mut misses = 0;
    for size in 1..=9 {
        for mask in 0..1usize << size {
            let last: Vec<u8> = (0..size).map(|i| 34 + ((mask >> i) & 1) as u8).collect();
            let caps: Vec<u16> = last.iter().map(|b| (u16::from(*b) << 8) | 1).collect();
            let mut psi: Vec<usize> = (0..size).collect();
            psi.sort_by_key(|i| last[*i]);
            for mark in 0..size {
                let mut row = mark;
                let want: Vec<u8> = (0..size)
                    .map(|_| {
                        row = psi[row];
                        last[row]
                    })
                    .collect();
                for k in [1, 2, 4, 8] {
                    if decoder.for_k::<true>(&caps, size, mark as u32, &mut out, k) {
                        assert_eq!(out, want);
                        takes += 1;
                    } else {
                        misses += 1;
                    }
                }
            }
        }
    }
    assert!(takes > 0 && misses > 0);
    eprintln!("{takes} exact nested marked outputs, {misses} selected-section refusals");
}

#[test]
#[ignore]
fn corpus() {
    use mibz2::stream::{Event, Scanner, SliceSource};
    use std::io::Write;
    let data = std::fs::read(std::env::var("F2_CORPUS").unwrap()).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut csv =
        std::io::BufWriter::new(std::fs::File::create(std::env::var("F2_CSV").unwrap()).unwrap());
    writeln!(csv,"stream,k,rows,pieces,token_rows,token_visits,distinct_tags,spelling_bytes,lookup_steps,components").unwrap();
    let mut input = Runs::default();
    let mut exact = Segmented::default();
    let mut decoder = Candidate::default();
    let (mut result, mut expected) = (Vec::new(), Vec::new());
    let mut accepted = [0; 3];
    let kernels = Kernels::detect();
    while let Ok(Event::Block { index, symbols, .. }) = scanner.next(Vec::new()) {
        let n = crate::runs::front(&symbols, &mut input, kernels).unwrap();
        if n < 65536 || input.records().len() * 20 >= 3 * n || input.records().len() * 64 < n {
            continue;
        }
        assert!(exact.invert_runs(&input, n, symbols.orig_ptr, &mut expected, kernels));
        for (i, k) in [1, 4, 8].into_iter().enumerate() {
            assert!(decoder.for_k::<false>(input.records(), n, symbols.orig_ptr, &mut result, k));
            assert_eq!(result, expected);
            let pieces = decoder.merged.len() - decoder.nodes[decoder.parts.len() - 1].out as usize;
            writeln!(
                csv,
                "{index},{k},{n},{pieces},{},{},{},{},{},{}",
                decoder.stats[4],
                decoder.stats[2],
                decoder.words.len(),
                decoder.dict.len(),
                decoder.stats[0],
                decoder.parts.len()
            )
            .unwrap();
            accepted[i] += 1;
        }
    }
    assert_eq!(accepted, [228; 3]);
    eprintln!("checked all 228 admissions at k=1,4,8");
}
