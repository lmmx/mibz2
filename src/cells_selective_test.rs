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
    // The original takes fresh slots after the letters'; the candidate takes none, but has
    // the same room, so that every offset outside the slots' regions is the same in both.
    original.fresh_room = true;
    candidate.fresh_room = true;
    // The original reads every field from both orders' slots; the candidate's bottom slots
    // are 8 bytes, the slot before and the letter's top slot.
    original.wide_bottom = true;
    candidate.wide_bottom = false;
    // The candidate may be the lazy kernel, which has no letters' slots to compare, or
    // (research only, `memory-lever`) the staged one, whose rules start at a multiple of 16,
    // so the original is laid out the same way.
    let lazy = matches!(candidate.variant, LAZY | LAZY_BUF);
    #[cfg(feature = "memory-lever")]
    {
        original.variant = if candidate.variant == STAGED_RULES {
            STAGED_RULES
        } else {
            0
        };
    }
    // the candidate's output buffer: the lazy kernel keeps its entries and bottom slots there
    let mut cout = Vec::new();
    let a = original.slots(column.len(), recs.records(), origin, budget, true, &mut Vec::new());
    let b = candidate.first_stage(column.len(), recs.records(), origin, budget, true, &mut cout);
    assert_eq!(a, b);
    if a.is_err() {
        return Vec::new();
    }
    // Only padding/dead fields differ, never the initialized initial letter slots: the top
    // slots are equal but for the partner (a bottom slot, whose size differs), and each
    // bottom slot names the same top slot, which names it back, and the slot before it; the
    // original's wide bottom slot repeats its top slot's fields.
    let (tops, bots) = (original.lay.tops, original.lay.bots);
    assert_eq!(candidate.lay.tops, tops);
    assert!(lazy || candidate.lay.bots == bots);
    assert_eq!(candidate.letters, original.letters);
    let read = |k: &Kernel, at: usize| u32::from_ne_bytes(k.buf[at..at + 4].try_into().unwrap());
    for i in 0..if lazy { 0 } else { original.letters } {
        let t = tops + i * SLOT;
        assert_eq!(original.buf[t..t + 16], candidate.buf[t..t + 16]);
        let (wide, narrow) = (bots + i * SLOT, bots + i * BSLOT);
        let top = read(original, wide + 16) as usize;
        assert_eq!(read(candidate, narrow + 4) as usize, top);
        assert_eq!(read(original, top + 16) as usize, wide);
        assert_eq!(read(candidate, top + 16) as usize, narrow);
        assert_eq!(original.buf[wide + 4..wide + 16], original.buf[top + 4..top + 16]);
        if i > 0 {
            assert_eq!(read(original, wide) as usize, wide - SLOT);
            assert_eq!(read(candidate, narrow) as usize, narrow - BSLOT);
        }
    }
    let a = original.induce_cells_original(budget);
    let b = candidate.induce_variant(budget, &mut cout);
    assert_eq!(a, b, "column {column:?}, origin {origin}, budget {budget}");
    // The slots a cell is written to differ (a promised slot is the one let go at the
    // front, not a fresh one), so the slots' offsets are left out: the state's 7..15 and a
    // slot's link and partner. The rest, words and their offsets included, is equal: the
    // selected slots are recorded as the cell's top slot's (length, word), once checked
    // that the two slots name each other (and, for the original, hold the same fields).
    let placed = |k: &Kernel| {
        k.materialization
            .as_ref()
            .unwrap()
            .iter()
            .map(|m| {
                let mut state = m.state;
                state[7..15].fill(0);
                (state, m.payload.clone(), m.slots.clone())
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(placed(original), placed(candidate));
    original.check_canaries(None);
    candidate.check_canaries(if lazy { Some(&cout) } else { None });
    if let Ok((word, len, _)) = a {
        assert_eq!(original.rule_end, candidate.rule_end);
        // once the induction is over, the candidate's rules' region is the original's, byte
        // for byte (in these layouts the rules lie in the kernel's buffer; a staged kernel's
        // rules were sent on a stage at a time)
        let rules = original.lay.rules..original.rule_end;
        assert_eq!(candidate.lay.rules, rules.start);
        assert_eq!(original.buf[rules.clone()], candidate.buf[rules]);
        let want = oracle(column, origin);
        let mut old_out: Vec<_> = want.iter().map(|x| !x).collect();
        let mut new_out = old_out.clone();
        original.write_cells(word, len, column.len(), &mut old_out, &Vec::new());
        candidate.write_cells(word, len, column.len(), &mut new_out, &Vec::new());
        assert_eq!(old_out, want);
        assert_eq!(new_out, want);
        assert_eq!(original.positions(), candidate.positions());
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
