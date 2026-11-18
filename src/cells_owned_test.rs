// Included inside cells::tests: the one-scan writer (`cells_owned.rs`), the writer of every
// full-period block, against the dense LF oracle, on the eager and the lazy kernel, with its
// own schedule counted taken.

/// The length of the LF cycle through row `p` of `column` (the block's period)
fn cycle_len(column: &[u8], p: u32) -> usize {
    let mut order: Vec<usize> = (0..column.len()).collect();
    order.sort_by_key(|&i| column[i]);
    // LF: row i's byte is the `rank`-th of its value, so it maps to that sorted position
    let mut lf = vec![0; column.len()];
    for (sorted, &i) in order.iter().enumerate() {
        lf[i] = sorted;
    }
    let (mut r, mut k) = (lf[p as usize], 1);
    while r != p as usize {
        r = lf[r];
        k += 1;
    }
    k
}

/// Every column of `alphabet` letters up to `max_n` rows, at every row: each block's bytes
/// by the one-scan writer, eager and lazy; (blocks it wrote by its own schedule, blocks of a
/// full period of 2 rows or more, which it must take every one of: the word is then the
/// last stored rule)
fn owned_exhaustive(alphabet: usize, max_n: u32) -> (usize, usize) {
    let mut eager = Kernel::default();
    let mut lazy = Kernel { variant: LAZY, ..Default::default() };
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
                check(&mut eager, &column, p);
                check(&mut lazy, &column, p);
                if n >= 2 && cycle_len(&column, p) == n as usize {
                    blocks += 2;
                }
            }
        }
    }
    (eager.owned_taken + lazy.owned_taken, blocks)
}

#[test]
fn owned_writer_exhaustive_binary() {
    let (taken, full) = owned_exhaustive(2, 10);
    // every full-period block takes the one-scan schedule; the others fall back
    assert_eq!(taken, full);
}

#[test]
fn owned_writer_exhaustive_ternary() {
    let (taken, full) = owned_exhaustive(3, 7);
    assert_eq!(taken, full);
}

/// Study 5's two witnesses at the 16-byte cutoff: L = 11 0^33 (one cycle; a 17-byte word
/// copied from a later address) and L = 11 0^34 (period 18 in 36 rows: the fallback), every row
#[test]
fn owned_writer_periodic_witnesses() {
    for zeros in [33usize, 34] {
        let column: Vec<u8> = [1u8, 1].iter().copied().chain(std::iter::repeat(0).take(zeros)).collect();
        let mut eager = Kernel::default();
        let mut lazy = Kernel { variant: LAZY, ..Default::default() };
        for p in 0..column.len() as u32 {
            check(&mut eager, &column, p);
            check(&mut lazy, &column, p);
        }
        assert!(eager.owned_taken > 0 || zeros == 34, "{zeros}: the one-scan schedule taken");
    }
}

/// Longer columns with long rules, copies from later addresses and a shared kernel across
/// blocks of different sizes: text-like (few letters, long runs, repeated phrases) and random
#[test]
fn owned_writer_long_columns() {
    let mut eager = Kernel::default();
    let mut lazy = Kernel { variant: LAZY, ..Default::default() };
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut rnd = |k: u64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed % k
    };
    let phrase = b"the cells kernel writes each rule once ";
    for (round, n) in [64usize, 300, 1_000, 4_096, 20_000, 70_000].into_iter().enumerate() {
        // a bwt column of a text: the text's rotations sorted, each one's last byte
        let text: Vec<u8> = (0..n)
            .map(|i| if rnd(10) < 8 { phrase[(i + round) % phrase.len()] } else { b'a' + rnd(4) as u8 })
            .collect();
        let mut rot: Vec<usize> = (0..n).collect();
        rot.sort_by(|&a, &b| {
            let (x, y) = (text[a..].iter().chain(&text[..a]), text[b..].iter().chain(&text[..b]));
            x.cmp(y)
        });
        let column: Vec<u8> = rot.iter().map(|&r| text[(r + n - 1) % n]).collect();
        for p in [0u32, 1, (n / 3) as u32, (n - 1) as u32] {
            check(&mut eager, &column, p);
            check(&mut lazy, &column, p);
        }
        let random: Vec<u8> = (0..n).map(|_| rnd(3) as u8).collect();
        for p in [0u32, (n / 2) as u32] {
            check(&mut eager, &random, p);
            check(&mut lazy, &random, p);
        }
    }
    assert!(eager.owned_taken > 0 && lazy.owned_taken > 0);
}


/// Under Miri: the one-scan writer on full-period blocks with long rules, copies from later
/// addresses and spine wraps, eager and lazy, then a smaller block on the same allocations
#[test]
fn miri_cells_owned_writer() {
    let column = long_writer_column();
    for variant in [0, LAZY] {
        let mut kernel = Kernel { variant, ..Default::default() };
        let mut out = Vec::new();
        let study = [[1u8, 1].as_slice(), &[0; 33]].concat();
        for (ll, p) in [(column.clone(), 0), (column.clone(), 3), (study, 2)] {
            let want = oracle(&ll, p);
            out.clear();
            out.extend(want.iter().map(|b| !b));
            let r = Runs::of_column(&ll);
            let before = kernel.owned_taken;
            assert!(kernel.try_invert(r.records(), ll.len(), p, ll.len(), true, &mut out).is_ok());
            assert_eq!(out, want, "variant {variant}, row {p}");
            if cycle_len(&ll, p) == ll.len() {
                assert_eq!(kernel.owned_taken, before + 1, "the one-scan schedule taken");
            }
        }
        let ll = b"bbaaa";
        let want = oracle(ll, 2);
        out.clear();
        out.extend(want.iter().map(|b| !b));
        assert!(kernel.try_invert(Runs::of_column(ll).records(), ll.len(), 2, ll.len(), true, &mut out).is_ok());
        assert_eq!(out, want);
    }
}

/// The BWT column of `text` (its rotations sorted, each one's last byte), and the row of the
/// rotation that starts the text
fn bwt_of(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut rot: Vec<usize> = (0..n).collect();
    rot.sort_by(|&a, &b| text[a..].iter().chain(&text[..a]).cmp(text[b..].iter().chain(&text[..b])));
    let column = rot.iter().map(|&r| text[(r + n - 1) % n]).collect();
    (column, rot.iter().position(|&r| r == 0).unwrap() as u32)
}

/// Copies that depend on copies: real text with long repeated lines (this file's own sources)
/// makes long words that survive inside the owned occurrence of other long words that
/// survive, so a command's source holds bytes only an earlier parent's command fills. Run in
/// any other order than increasing parent, those copies would read unfilled bytes (the
/// output starts as the complement of the block)
#[test]
fn owned_writer_nested_copies() {
    let sources = [include_str!("cells.rs"), include_str!("cells_lazy.rs"), include_str!("cells_owned.rs")];
    for (i, src) in sources.iter().enumerate() {
        for &(from, len) in &[(0usize, 12_000usize), (20_000, 30_000)] {
            let text = &src.as_bytes()[from.min(src.len())..(from + len).min(src.len())];
            if text.len() < 1_000 {
                continue;
            }
            let (column, p) = bwt_of(text);
            for variant in [0, LAZY] {
                let mut kernel = Kernel { variant, ..Default::default() };
                check(&mut kernel, &column, p);
                if cycle_len(&column, p) == column.len() {
                    assert_eq!(kernel.owned_taken, 1, "source {i} at {from}");
                }
            }
        }
    }
}
