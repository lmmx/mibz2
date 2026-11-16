// Included inside cells::tests: the lazy kernel (`cells_lazy.rs`) against the original
// induction and the oracle. With the `memory-lever` feature (research only), the
// experiment's staged kernel (`cells_lever.rs`) too, and its store-fence model.

/// The kernels these tests run: the lazy one, and (research only) the staged one
fn lever_variants() -> Vec<u8> {
    #[cfg(feature = "memory-lever")]
    return vec![LAZY, LAZY_PLAIN, STAGED_RULES];
    #[cfg(not(feature = "memory-lever"))]
    vec![LAZY, LAZY_PLAIN]
}

/// Whether the kernel is a lazy one (with its fetch-ahead, or without)
fn lazy_variant(variant: u8) -> bool {
    matches!(variant, LAZY | LAZY_PLAIN)
}

fn lever_cases() -> Vec<Vec<u8>> {
    vec![
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
    ]
}

/// Both kernels make the original induction's cells, in its order, with its words at its
/// offsets, on every short column: `compare_materialization`, the candidate one of them.
#[test]
fn cells_lever_exhaustive_state() {
    for variant in lever_variants() {
        let (mut old, mut new) = (Kernel::default(), Kernel::default());
        new.variant = variant;
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
}

/// The same on the boundary columns, with reuse of both kernels' buffers from block to
/// block, under Miri: every unsafe region of both kernels runs, canaries after each region.
/// The staged kernel sends its rules on several times in one induction; the lazy one takes
/// slots from its pools, never more than a slot a letter.
#[test]
fn miri_cells_lever_boundaries_reuse() {
    for variant in lever_variants() {
        let (mut old, mut new) = (Kernel::default(), Kernel::default());
        new.variant = variant;
        let mut reached = Vec::new();
        #[cfg(feature = "memory-lever")]
        let mut sends = 0;
        let mut pool = 0;
        for column in lever_cases() {
            for origin in [0, 1, column.len() / 2, column.len() - 1] {
                reached.extend(compare_materialization(
                    &mut old,
                    &mut new,
                    &column,
                    origin as u32,
                    column.len(),
                ));
                #[cfg(feature = "memory-lever")]
                {
                    sends = sends.max(new.rule_sends);
                }
                if lazy_variant(variant) {
                    assert!(new.lz.peak <= new.letters, "a slot a letter at most");
                    pool = pool.max(new.lz.peak);
                }
            }
        }
        for order in [
            std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal,
            std::cmp::Ordering::Greater,
        ] {
            assert!(reached.iter().any(|&(_, _, o)| o == order));
        }
        assert!(reached.iter().any(|&(n, marked, _)| n > 16 && marked));
        #[cfg(feature = "memory-lever")]
        if variant == STAGED_RULES {
            assert!(
                !STAGED || sends >= 3,
                "one induction sends its rules on several times: {sends}"
            );
        }
        if lazy_variant(variant) {
            assert!(pool > 1, "the lazy induction takes slots: {pool}");
        }
        // a refusal on the budget, at the same cell as the original's
        let column = long_writer_column();
        compare_materialization(&mut old, &mut new, &column, 3, column.len());
        let cells = new.materialization.as_ref().unwrap().len();
        for budget in [0, cells - 1, cells] {
            compare_materialization(&mut old, &mut new, &column, 3, budget);
        }
    }
}

/// Both kernels through `try_invert`, their own route, against the plain walk: every binary
/// column to length 10 at every row, and two long Euclid-like ones.
#[test]
fn cells_lever_exhaustive_arbitrary_columns() {
    for variant in lever_variants() {
        let mut kernel = Kernel {
            variant,
            ..Default::default()
        };
        for n in 1..=10 {
            for bits in 0..1usize << n {
                let ll: Vec<_> = (0..n).map(|i| ((bits >> i) & 1) as u8).collect();
                for p in 0..n {
                    check(&mut kernel, &ll, p as u32);
                }
            }
        }
        for ll in [
            [vec![1], vec![0; 100]].concat(),
            [vec![1; 3], vec![0; 200]].concat(),
        ] {
            for p in [0, ll.len() / 2, ll.len() - 1] {
                check(&mut kernel, &ll, p as u32);
            }
        }
    }
}

/// Both kernels through `try_invert` under Miri, over stale bytes in both buffers: exact
/// output, a budget of exactly the cells made accepted and one less refused, and for the
/// lazy kernel every slot a cell is written to inside its pool.
#[test]
fn miri_cells_lever_exact_fit_and_reuse() {
    for variant in lever_variants() {
        let mut kernel = Kernel {
            variant,
            ..Default::default()
        };
        for ll in [
            vec![1, 0],
            b"bbaaa".to_vec(),
            b"abababa".to_vec(),
            vec![0; 17],
            (0..65).map(|i| (i % 3) as u8).collect(),
            [vec![1; 31], vec![0; 34]].concat(),
            long_writer_column(),
        ] {
            for p in [0, ll.len() / 2, ll.len() - 1] {
                kernel.buf.fill(0x5A);
                kernel.materialization = Some(Vec::new());
                let cells = check(&mut kernel, &ll, p as u32);
                let lay = kernel.lay;
                // slots are given back and taken again: a block of many letters never has
                // half of them in slots at once (these columns peak near a quarter)
                if lazy_variant(variant) && kernel.letters >= 60 {
                    assert!(
                        kernel.lz.peak * 2 < kernel.letters,
                        "{} slots of {} letters",
                        kernel.lz.peak,
                        kernel.letters
                    );
                }
                for m in kernel.materialization.take().unwrap() {
                    let (fa, fb) = (m.state[7] as usize, m.state[8] as usize);
                    let (bs, ts) = (m.state[11] as usize, m.state[12] as usize);
                    assert!(fa >= lay.tops && fa + SLOT <= lay.ends[0]);
                    assert!(ts >= lay.tops && ts + SLOT <= lay.ends[0]);
                    if lazy_variant(variant) {
                        // the bottom slots' pool, in the output buffer after the streams
                        let (lo, hi) = (lay.bots, kernel.lz.bend);
                        assert_eq!(lo, lay.ends[1] + GAP);
                        assert!(fb >= lo && fb + BSLOT <= hi);
                        assert!(bs >= lo && bs + BSLOT <= hi);
                    }
                }
                let r = Runs::of_column(&ll);
                let mut out = vec![0xCC; ll.len()];
                if kernel.letters - 1 <= cells {
                    assert!(kernel
                        .try_invert(r.records(), ll.len(), p as u32, cells, true, &mut out)
                        .is_ok());
                    assert_eq!(out, oracle(&ll, p as u32));
                    if cells > 0 {
                        assert!(kernel
                            .try_invert(r.records(), ll.len(), p as u32, cells - 1, true, &mut out)
                            .is_err());
                    }
                }
            }
        }
    }
}

/// The store fence (Guardian D108). A model stands for the stores that go past the cache:
/// `send_rules` marks them pending wherever it sends whole lines, and the fence clears the
/// mark where it runs `sfence`. The writer asserts nothing is pending before it reads a
/// rule. Here every way out of the staged induction after a send is taken: a word returned,
/// a refusal on the budget, and an unwinding panic. After each, nothing is pending and
/// exactly one fence has run.
#[cfg(feature = "memory-lever")]
#[test]
fn miri_cells_staged_fence_covers_every_exit() {
    let column: Vec<u8> = long_writer_column()
        .into_iter()
        .flat_map(|b| [b; 10])
        .collect();
    let r = Runs::of_column(&column);
    let n = column.len();
    let mut kernel = Kernel {
        variant: STAGED_RULES,
        ..Default::default()
    };
    let fences = || FENCES.with(|f| f.get());
    let pending = || PENDING.with(|p| p.get());
    // how many fences found the mark set: where rules are staged, each exit below comes
    // after a send, so its fence must have found it
    let cleared = || CLEARED.with(|c| c.get());
    let found = usize::from(STAGED);
    // a word returned, after several sends (the writer's own assertion ran too)
    let (before, marked) = (fences(), cleared());
    let cells = check(&mut kernel, &column, 3);
    assert!(!STAGED || kernel.rule_sends >= 3, "{}", kernel.rule_sends);
    assert!(!pending());
    assert_eq!((fences(), cleared()), (before + 1, marked + found));
    // a refusal on the budget, after the sends
    let (before, marked) = (fences(), cleared());
    let mut out = vec![0xCC; n];
    assert_eq!(
        kernel.try_invert(r.records(), n, 3, cells - 1, true, &mut out),
        Err(Refusal::Budget)
    );
    assert!(!pending());
    assert_eq!((fences(), cleared()), (before + 1, marked + found));
    // an unwinding panic, right after the first send of whole lines
    if STAGED {
        let (before, marked) = (fences(), cleared());
        kernel.panic_after_send = true;
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = kernel.try_invert(r.records(), n, 3, n, true, &mut out);
        }));
        kernel.panic_after_send = false;
        assert!(caught.is_err(), "the induction panicked after a send of whole lines");
        assert!(!pending());
        assert_eq!((fences(), cleared()), (before + 1, marked + 1));
    }
    // and the kernel is whole afterwards
    check(&mut kernel, &column, 3);
}

/// Both kernels through the public route (`CellsVariant`), against Segmented: admitted
/// blocks inverted by the kernel, refused ones by the fallback, the output equal either way.
#[test]
fn cells_lever_route_and_fallback() {
    fn run<const V: u8>() -> (Counts, usize) {
        let mut cells = CellsVariant::<V>::default();
        let mut seg = Segmented::default();
        let mut pool = 0;
        for ll in [
            vec![0; 10],
            vec![0; 11],
            [vec![1; 51], vec![0; 50]].concat(),
            vec![0; 25],
            // 200 runs of 16 rows over five bytes: inside both of admission's gates
            (0..200usize).flat_map(|i| [(i * 7 % 5) as u8; 16]).collect(),
        ] {
            let r = Runs::of_column(&ll);
            let mut want = Vec::new();
            let mut got = vec![42; ll.len()];
            assert!(seg.invert_runs(&r, ll.len(), 1, &mut want, crate::Kernels::scalar()));
            assert!(cells.invert_runs(&r, ll.len(), 1, &mut got, crate::Kernels::scalar()));
            assert_eq!(got, want);
            if cells.took_cells() {
                pool = pool.max(cells.pool().0);
            }
        }
        (cells.counts(), pool)
    }
    let (lazy, pool) = run::<LAZY>();
    assert!(lazy.admitted > 1 && lazy.density > 0, "{lazy:?}");
    let (plain, pool_plain) = run::<LAZY_PLAIN>();
    assert_eq!((plain.admitted, plain.cells), (lazy.admitted, lazy.cells));
    assert_eq!(pool_plain, pool);
    #[cfg(feature = "memory-lever")]
    {
        let (staged, _) = run::<STAGED_RULES>();
        assert!(staged.admitted > 1 && staged.density > 0, "{staged:?}");
        assert_eq!(lazy.cells, staged.cells);
    }
    assert!(pool > 0, "an admitted block's lazy induction took slots");
}

/// A lazy entry holds a letter's length in `LEN_BITS` bits: a block of more rows is refused
/// before anything is laid out, and Cells' fallback takes it (no bzip2 block is that long).
#[test]
fn cells_lazy_refuses_a_block_past_its_entries() {
    let n = 1usize << LEN_BITS;
    // one run of `n` rows, as records of at most 127
    let mut recs = vec![127u16; n / 127];
    recs.push((n % 127) as u16);
    let mut out = vec![7, 7];
    let mut kernel = Kernel {
        variant: LAZY,
        ..Default::default()
    };
    assert_eq!(
        kernel.try_invert(&recs, n, 0, n, true, &mut out),
        Err(Refusal::Invalid)
    );
    assert!(kernel.buf.is_empty(), "nothing was allocated");
    assert_eq!(out, [7, 7]);
    // one row fewer is taken
    recs[0] -= 1;
    assert!(kernel.try_invert(&recs, n - 1, 0, n - 1, true, &mut out).is_ok());
    assert!(out.iter().all(|&b| b == 0) && out.len() == n - 1);
}

/// A closed letter that is not the walk's row's gives its two slots back. A column of
/// swapped pairs is a hundred cycles of two rows: each is cut into one cell, which closes.
/// Whatever the walk's row, the lazy induction never holds more than a few pairs of slots.
#[test]
fn miri_cells_lazy_closed_letters_give_their_slots_back() {
    let ll: Vec<u8> = (0..200u8).map(|i| i ^ 1).collect();
    let mut kernel = Kernel {
        variant: LAZY,
        ..Default::default()
    };
    for p in [0, 77, 199] {
        check(&mut kernel, &ll, p);
        assert_eq!(kernel.letters, 200);
        assert!(kernel.lz.peak <= 4, "{} pairs of slots", kernel.lz.peak);
    }
}

/// A letter longer than 2^21 rows, cut by a short one: its length is read whole from its
/// entry (`LEN_BITS` bits), on the lazy route.
#[test]
fn cells_lazy_long_letter() {
    let ll = [vec![1u8; 3], vec![0u8; (1 << (LEN_BITS - 1)) + 5]].concat();
    let mut kernel = Kernel {
        variant: LAZY,
        ..Default::default()
    };
    for p in [1, ll.len() as u32 - 2] {
        check(&mut kernel, &ll, p);
    }
}

/// The lazy induction's fetch-ahead reads entries `FAR_PLACES` on in each stream, and a slot
/// through one. Here the output buffer comes in full of bytes that would read as tagged
/// entries naming far-off slots, on columns of fewer letters than the look-ahead and of
/// more: the first stage sets the bytes before the entries, every read stays inside the
/// buffer (Miri sees one that does not), and the output is the oracle's. Every slot hinted
/// is the slot then read at its place. The kernel without the fetch-ahead gives the same
/// cells and hints nothing.
#[test]
fn miri_cells_lazy_fetch_ahead_stays_inside() {
    for ll in [
        b"bbaaa".to_vec(),
        (0..2 * FAR_PLACES as u8).map(|i| i ^ 1).collect(),
        (0..65).map(|i| (i % 3) as u8).collect(),
        long_writer_column(),
    ] {
        let r = Runs::of_column(&ll);
        let n = ll.len();
        let mut cells = Vec::new();
        let mut met = 0;
        for variant in [LAZY, LAZY_PLAIN] {
            let mut kernel = Kernel {
                variant,
                ..Default::default()
            };
            for p in [0, n as u32 / 2, n as u32 - 1] {
                // (longer than the block needs, so that none of it is new and zeroed)
                let mut out = vec![0xFFu8; 64 * n + 4096];
                let (made, _) = kernel
                    .try_invert(r.records(), n, p, n, true, &mut out)
                    .unwrap();
                assert_eq!(out, oracle(&ll, p), "column {ll:?}, row {p}");
                cells.push(made);
                // a hinted slot is the one its place is then read from (the induction
                // asserts each), and on a column of enough letters some are hinted
                if variant == LAZY_PLAIN {
                    assert_eq!(kernel.hints_met, 0);
                } else if kernel.letters > 2 * FAR_PLACES {
                    assert!(kernel.hints_met > 0, "column {ll:?}, row {p}");
                    met += kernel.hints_met;
                }
                // the first stage alone: the bytes before the entries are set
                let mut out = vec![0xFFu8; 64 * n + 4096];
                kernel
                    .first_stage(n, r.records(), p, n, true, &mut out)
                    .unwrap();
                assert!(out[..BPAD].iter().all(|&b| b == 0));
            }
        }
        assert_eq!(cells[..3], cells[3..]);
        let _ = met;
    }
}
