// EdgesMux (the Mechanisms line's cycle 19, Theorem 19.1): `edges` holds the strips' image
// order and then the elimination's arena until the wiring writes it. Each region is poisoned
// after its last read, which the output must not see, and (the controls) before it, which it
// must: the regions' lifetimes are disjoint from the edges', and the poison reaches what it
// claims to.
use super::b5::{plain, runs_column, Lcg};
use super::endpoint::Clobber;
use super::*;

fn columns() -> Vec<(Vec<u8>, u32)> {
    let mut g = Lcg(0x6d75_7831);
    let mut v = vec![];
    for (n, alphabet, mean) in [
        (5_000usize, 3u32, 20u32),
        (70_000, 17, 16),
        (200_000, 60, 12),
        (900_000, 60, 12),
    ] {
        let l = runs_column(&mut g, n, alphabet, mean);
        let p = g.below(n as u32);
        v.push((l, p));
    }
    // a spiral (the contracted loop) and a block of `Wide` links
    let mut l = vec![0u8; 300_000];
    l[0] = 1;
    v.push((l, 5));
    v.push(((0..900_000).map(|i| (i / 17 % 3) as u8).collect(), 11));
    v
}

/// What the kernel gives on column `l` at `p` with `edges` poisoned at `at`, through a
/// kernel that has decoded another block first (its `edges` longer than this block's): the
/// output, or `None` if the construction's checks panicked
fn poisoned(l: &[u8], p: u32, at: Clobber) -> Option<Vec<u8>> {
    let r = Runs::of_column(l);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    let other = runs_column(&mut Lcg(7), 400_000, 40, 3);
    k.run::<true, true>(
        Runs::of_column(&other).records(),
        other.len(),
        3,
        LIMIT,
        &mut out,
    );
    k.endpoint.clobber = at;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        k.run::<true, true>(r.records(), l.len(), p, LIMIT, &mut out);
        out
    }))
    .ok()
}

/// Poisoned after their last reads, the image order and the arena leave the output exact
#[test]
fn regions_dead_after_their_last_read() {
    for (l, p) in columns() {
        let want = plain(&l, p);
        for at in [Clobber::OrderAfterNodes, Clobber::AllBeforeWire] {
            assert!(
                poisoned(&l, p, at) == Some(want.clone()),
                "n {}, {at:?}",
                l.len()
            );
        }
    }
}

/// The controls: poisoned before their last reads, the same regions break the block (a
/// wrong output or a failed check), so the poison above reaches a region in use
#[test]
fn regions_live_before_their_last_read() {
    for (l, p) in columns() {
        let want = plain(&l, p);
        for at in [Clobber::OrderBeforeNodes, Clobber::ArenaBeforeBands] {
            assert!(
                poisoned(&l, p, at) != Some(want.clone()),
                "n {}, {at:?}",
                l.len()
            );
        }
    }
}

/// The image order costs no buffer of its own: on a full block `edges` is its `m` words, as
/// before (the order and the arena fit below `m`), and the kernel keeps no other `u32`
/// buffer the size of the strips
#[test]
fn order_in_edges_costs_nothing_on_full_blocks() {
    let mut g = Lcg(0x6d75_7832);
    // Wikidata's kind (about 3,400 strips and 196,000 retained rows)
    let l = super::dense::dense_column(&mut g, 900_000, 40);
    let r = Runs::of_column(&l);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    let s = k.run::<true, true>(r.records(), l.len(), 99, LIMIT, &mut out);
    assert!(out == plain(&l, 99));
    assert!(
        k.arena_words(s.thick, s.retained) == s.retained,
        "{} strips",
        s.thick
    );
    assert_eq!(k.edges.capacity(), s.retained);
}
