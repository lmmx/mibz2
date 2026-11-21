// The elimination's spirals (`endpoint.rs`, `eliminate::<true>`) and the width certificate
// that leaves them out (`CERTIFIED_WIDTH`): the contracted loop's lists, widths and words are
// the plain loop's, step for step, and its output the plain inverse's.
use super::b5::{plain, runs_column, Lcg};
use super::endpoint::CERTIFIED_WIDTH;
use super::*;

/// A construction's output, counts, bands (end, delta, code), spelling and word offsets
type Built = (Vec<u8>, Counts, Vec<(u32, u32, u32)>, Vec<u8>, Vec<u32>);

/// The construction of column `l` at row `p` with the spirals contracted (`Some(true)`), the
/// plain loop (`Some(false)`), or as the certificate decides (`None`): its output, counts,
/// bands and spelled words
fn construct(k: &mut Kernel, l: &[u8], p: u32, spirals: Option<bool>) -> Built {
    let r = Runs::of_column(l);
    k.endpoint.spirals = spirals;
    let mut out = vec![0xC3; l.len() / 2];
    let s = k.run::<true, true>(r.records(), l.len(), p, LIMIT, &mut out);
    k.endpoint.spirals = None;
    let bands = k.bands.iter().map(|b| (b.end, b.delta, b.code)).collect();
    (out, s, bands, k.spelling.clone(), k.offsets.clone())
}

/// Both loops on `l` at `p`: the same bands, words and steps, and the plain inverse's output;
/// the contracted loop's rules, which it returns with the plain loop's
fn both(k: &mut Kernel, l: &[u8], p: u32) -> (usize, usize) {
    let want = plain(l, p);
    let (o1, s1, b1, w1, f1) = construct(k, l, p, Some(false));
    let (o2, s2, b2, w2, f2) = construct(k, l, p, Some(true));
    assert!(o1 == want && o2 == want, "n {}, row {p}", l.len());
    assert!(!s1.spirals && s2.spirals);
    assert!(b1 == b2, "the bands, n {}, row {p}", l.len());
    assert!(w1 == w2 && f1 == f2, "the words, n {}, row {p}", l.len());
    assert_eq!(s1.steps, s2.steps);
    assert_eq!(
        (s1.words, s1.traced, s1.longest),
        (s2.words, s2.traced, s2.longest)
    );
    assert!(s2.rules <= s1.rules);
    (s2.rules, s1.rules)
}

/// The cycle's two spirals, a run of one row against a run of the rest (`1 0^(n-1)` and
/// `1^(n-1) 0`), and two- and three-run columns of every shape around them: the contracted
/// loop's steps are the plain loop's, in a handful of rules where the plain loop makes one a
/// step
#[test]
fn spiral_columns_contract_exactly() {
    let mut k = Kernel::default();
    let n = 300_000;
    let mut one_zeros = vec![0u8; n];
    one_zeros[0] = 1;
    let mut ones_zero = vec![1u8; n];
    ones_zero[n - 1] = 0;
    for l in [&one_zeros, &ones_zero] {
        for p in [0, 1, n as u32 / 2, n as u32 - 1] {
            let (rules, plain_rules) = both(&mut k, l, p);
            assert!(
                rules < 200 && plain_rules > 100_000,
                "{rules} against {plain_rules}"
            );
        }
    }
    for (a, b, c) in [
        (1usize, 1usize, 0usize),
        (17, 1, 0),
        (1, 17, 0),
        (34, 17, 0),
        (89, 55, 0),
        (1000, 3, 0),
        (3, 1000, 0),
        (17, 1000, 17),
        (1000, 1, 1000),
        (5, 99_999, 3),
        (99_999, 18, 7),
        (65_537, 2, 65_535),
    ] {
        let mut l = vec![0u8; a];
        l.extend(std::iter::repeat_n(1, b));
        l.extend(std::iter::repeat_n(2, c));
        let n = l.len() as u32;
        for p in [0, a as u32 - 1, n / 2, n - 1] {
            both(&mut k, &l, p);
        }
    }
}

/// Arbitrary columns, as `b5`'s, through one kernel alternating the loops: the same bands
/// and words
#[test]
fn spirals_on_arbitrary_columns() {
    let mut g = Lcg(0x7370_6972);
    let mut k = Kernel::default();
    let mut fewer = 0;
    for &n in &[64usize, 129, 1000, 4097, 70_000] {
        for &alphabet in &[1u32, 2, 3, 17, 256] {
            for &mean in &[1u32, 3, 16, 17, 64, 400, 5000] {
                let l = runs_column(&mut g, n, alphabet, mean);
                for p in [0, n as u32 - 1, g.below(n as u32)] {
                    let (rules, plain_rules) = both(&mut k, &l, p);
                    fewer += usize::from(rules < plain_rules);
                }
            }
        }
    }
    assert!(fewer > 20, "{fewer} columns with a spiral contracted");
}

/// Every binary column through 11 rows and every ternary one through 6, at every row: the
/// contracted loop is the plain loop
#[test]
fn spirals_on_every_small_column() {
    let mut k = Kernel::default();
    for (base, len) in [(2usize, 11usize), (3, 6)] {
        for n in 1..=len {
            for mut z in 0..base.pow(n as u32) {
                let l: Vec<u8> = (0..n)
                    .map(|_| {
                        let c = z % base;
                        z /= base;
                        c as u8
                    })
                    .collect();
                for p in 0..n as u32 {
                    both(&mut k, &l, p);
                }
            }
        }
    }
}

/// The certificate: a block whose widest node is `CERTIFIED_WIDTH` rows runs the plain loop,
/// one a row wider the contracted one (a thick run of that many rows and one thin row, the
/// marked row; its gaps are a row wide)
#[test]
fn width_certificate_edge() {
    let mut k = Kernel::default();
    for (w, spirals) in [(CERTIFIED_WIDTH, false), (CERTIFIED_WIDTH + 1, true)] {
        let mut l = vec![0u8; w as usize];
        l.push(1);
        for p in [0, w / 2, w] {
            let (out, s, ..) = construct(&mut k, &l, p, None);
            assert!(out == plain(&l, p));
            // (the marked row inside the run cuts it: a row narrower on either side)
            assert_eq!(s.spirals, spirals && p == w, "widest {w}, row {p}");
        }
    }
    // the gaps count too: one thin row between two thick runs of a byte each, and a gap of
    // retained rows wider than the line on the image side
    let mut l: Vec<u8> = (0..CERTIFIED_WIDTH + 1).map(|i| (i % 2) as u8).collect();
    l.extend(std::iter::repeat_n(2, 40));
    let (out, s, ..) = construct(&mut k, &l, 3, None);
    assert!(out == plain(&l, 3));
    assert!(s.spirals, "a gap of {} rows", CERTIFIED_WIDTH + 1);
}
