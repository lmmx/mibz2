// Included inside cells::tests: study 6's first-occurrence writer (`cells_first.rs`, research
// only) against the dense LF oracle, on the eager and the lazy kernel, short periods included.

fn first_kernels() -> [Kernel; 2] {
    [
        Kernel { first_writer: true, ..Default::default() },
        Kernel { first_writer: true, variant: LAZY, ..Default::default() },
    ]
}

/// Every column of `alphabet` letters up to `max_n` rows, at every row, eager and lazy: the
/// walk takes every block whose word is a rule (a cycle of two rows or more; a one-row cycle's
/// word is the marked byte itself), whatever its period
fn first_exhaustive(alphabet: usize, max_n: u32) {
    let mut ks = first_kernels();
    let mut blocks = 0;
    for n in 1..=max_n {
        for mut digits in 0..alphabet.pow(n) {
            let column: Vec<u8> = (0..n)
                .map(|_| {
                    let byte = (digits % alphabet) as u8;
                    digits /= alphabet;
                    byte
                })
                .collect();
            for p in 0..n {
                for k in ks.iter_mut() {
                    check(k, &column, p);
                }
                if cycle_len(&column, p) >= 2 {
                    blocks += 1;
                }
            }
        }
    }
    for k in &ks {
        assert_eq!(k.first_taken, blocks, "every block with a cycle of two rows or more taken");
    }
}

#[test]
fn first_writer_exhaustive_binary() {
    first_exhaustive(2, 10);
}

#[test]
fn first_writer_exhaustive_ternary() {
    first_exhaustive(3, 7);
}

/// Study 6's comb: L = 1^(n-1) 0 at row n - 2 makes a left comb of n - 1 marked rules, so the
/// walk's stack peaks at n - 2, its bound R - 1
#[test]
fn first_writer_comb_stack() {
    for n in [4usize, 5, 17, 100, 1_000, 5_000] {
        let column: Vec<u8> = std::iter::repeat(1).take(n - 1).chain([0]).collect();
        for mut k in first_kernels() {
            check(&mut k, &column, (n - 2) as u32);
            let rules = (k.rule_end - k.lay.rules) / 16;
            assert_eq!(rules, n - 1, "{n}: a comb of n - 1 rules");
            assert_eq!(k.first_peak, n - 2, "{n}: the stack's peak");
        }
    }
}

/// Copies that depend on copies (as `owned_writer_nested_copies`): this file's sources' BWTs
#[test]
fn first_writer_nested_copies() {
    let sources = [include_str!("cells.rs"), include_str!("cells_lazy.rs"), include_str!("cells_first.rs")];
    for src in sources {
        for &(from, len) in &[(0usize, 12_000usize), (20_000, 30_000)] {
            let text = &src.as_bytes()[from.min(src.len())..(from + len).min(src.len())];
            if text.len() < 1_000 {
                continue;
            }
            let (column, p) = bwt_of(text);
            for mut k in first_kernels() {
                check(&mut k, &column, p);
                assert_eq!(k.first_taken, 1);
            }
        }
    }
}

/// Under Miri: the walk on long rules, copies, the wrap, a short period and reused allocations
#[test]
fn miri_cells_first_writer() {
    let column = long_writer_column();
    for mut k in first_kernels() {
        let study = [[1u8, 1].as_slice(), &[0; 34]].concat();
        for (ll, p) in [(column.clone(), 0), (column.clone(), 3), (study, 3), (b"bbaaa".to_vec(), 2)] {
            let before = k.first_taken;
            check(&mut k, &ll, p);
            assert_eq!(k.first_taken, before + 1);
        }
    }
}
