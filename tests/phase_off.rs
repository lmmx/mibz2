//! Research only (`phase-counters`), with no pass set: nothing is opened or counted, the
//! child record's field is null, and the output is byte-identical.
//!
//!     cargo test --features phase-counters --test phase_off
#![cfg(feature = "phase-counters")]
mod common;
use common::{compress, corpora, oracle};
use mibz2::parallel::{with_plan, Plan};

#[test]
fn no_pass_counts_nothing() {
    std::env::remove_var("MIBZ2_PHASE_PASS");
    let corp = corpora(1);
    let z = compress(&corp[0].1[..1_000_000], 9);
    let want = oracle(&z).unwrap();
    let got = with_plan(Plan::Workers(2), || mibz2::decode_parallel(&z, 2)).unwrap();
    assert!(got == want);
    assert!(mibz2::phase::pass().is_none());
    assert_eq!(mibz2::phase::report(), "null");
    let s = mibz2::phase::totals();
    assert_eq!((s.threads, s.blocks, s.errors.len()), (0, 0, 0));
}
