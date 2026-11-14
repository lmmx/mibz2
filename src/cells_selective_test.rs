// Included inside cells::tests: snapshots and eager comparator are test-only.
fn compare_materialization(
    original: &mut Kernel,
    candidate: &mut Kernel,
    column: &[u8],
    origin: u32,
    budget: usize,
) -> Vec<(u32, bool, std::cmp::Ordering)> {
    let recs = Runs::of_column(column);
    // Different initialized unused bytes, also across retained large/small blocks.
    original.buf.fill(0x39);
    candidate.buf.fill(0xC6);
    original.materialization = Some(Vec::new());
    candidate.materialization = Some(Vec::new());
    let a = original.slots(column.len(), recs.records(), origin, budget, true);
    let b = candidate.slots(column.len(), recs.records(), origin, budget, true);
    assert_eq!(a, b);
    if a.is_err() {
        return Vec::new();
    }
    // Only padding/dead fields differ, never the initialized initial letter slots.
    for at in [original.lay.tops, original.lay.bots] {
        let end = at + original.letters * SLOT;
        assert_eq!(original.buf[at..end], candidate.buf[at..end]);
    }
    let a = original.induce_cells_original(budget);
    let b = candidate.induce_cells(budget);
    assert_eq!(a, b, "column {column:?}, origin {origin}, budget {budget}");
    assert_eq!(original.materialization, candidate.materialization);
    original.check_canaries();
    candidate.check_canaries();
    if let Ok((word, len, _)) = a {
        assert_eq!(original.rule_end, candidate.rule_end);
        let want = oracle(column, origin);
        let mut old_out: Vec<_> = want.iter().map(|x| !x).collect();
        let mut new_out = old_out.clone();
        original.write_cells(word, len, column.len(), &mut old_out);
        candidate.write_cells(word, len, column.len(), &mut new_out);
        assert_eq!(old_out, want);
        assert_eq!(new_out, want);
        assert_eq!(original.pos, candidate.pos);
    }
    candidate
        .materialization
        .as_ref()
        .unwrap()
        .iter()
        .map(|x| {
            let len = x.state[4] as u32;
            (len & !MARK, len & MARK != 0, x.state[0].cmp(&x.state[1]))
        })
        .collect()
}

#[test]
fn cells_selective_exhaustive_state() {
    let (mut old, mut new) = (Kernel::default(), Kernel::default());
    for (alphabet, max_n) in [(2usize, 8u32), (3, 6)] {
        for n in 1..=max_n {
            for mut digits in 0..alphabet.pow(n) {
                let column: Vec<_> = (0..n)
                    .map(|_| {
                        let byte = (digits % alphabet) as u8;
                        digits /= alphabet;
                        byte
                    })
                    .collect();
                for origin in 0..n {
                    compare_materialization(&mut old, &mut new, &column, origin, n as usize);
                }
            }
        }
    }
}

#[test]
fn miri_cells_selective_boundaries_reuse() {
    let (mut old, mut new) = (Kernel::default(), Kernel::default());
    let mut reached = Vec::new();
    // n=18/17/16 reaches unmarked word lengths17/16/15 before the marked join.
    let cases = [
        [vec![1], vec![0; 17]].concat(),
        [vec![1], vec![0; 16]].concat(),
        [vec![1], vec![0; 15]].concat(),
        vec![1, 0],
        b"bbaaa".to_vec(),
        long_writer_column(),
        long_writer_column()
            .into_iter()
            .flat_map(|b| [b; 10])
            .collect(),
        vec![0; 19],
        b"abababa".to_vec(),
        long_writer_column(),
    ];
    for column in cases {
        for origin in [0, 1, column.len() / 2, column.len() - 1] {
            reached.extend(compare_materialization(
                &mut old,
                &mut new,
                &column,
                origin as u32,
                column.len(),
            ));
        }
    }
    for len in [15, 16, 17] {
        assert!(
            reached.iter().any(|&(n, marked, _)| n == len && !marked),
            "unmarked {len} missing"
        );
    }
    assert!(reached.iter().any(|&(n, marked, _)| n <= 16 && marked));
    assert!(reached.iter().any(|&(n, marked, _)| n > 16 && marked));
    for order in [
        std::cmp::Ordering::Less,
        std::cmp::Ordering::Equal,
        std::cmp::Ordering::Greater,
    ] {
        assert!(reached.iter().any(|&(_, _, o)| o == order));
    }
    let column = long_writer_column();
    compare_materialization(&mut old, &mut new, &column, 3, column.len());
    let cells = new.materialization.as_ref().unwrap().len();
    for budget in [0, cells - 1, cells] {
        compare_materialization(&mut old, &mut new, &column, 3, budget);
    }
}
