//! Research only (`phase-gates`): the gates' capacities read once a process from the
//! environment, an unset gate reported `null`, and the bytes unchanged.
//!
//!     cargo test --features phase-gates --test phase_gates_env
#![cfg(feature = "phase-gates")]
mod common;
use common::{cells, compress, corpora, oracle};
use mibz2::parallel::{with_plan, Plan};

#[test]
fn the_capacities_come_from_the_environment() {
    // (before any gate is entered: they are read once a process)
    std::env::set_var("MIBZ2_GATE_INDUCTION", "1");
    std::env::remove_var("MIBZ2_GATE_WRITER");
    std::env::set_var("MIBZ2_GATE_BLOCKS", "2");
    let corp = corpora(1);
    // (with Cells' witnesses, so that its induction is entered)
    let mut z = compress(&corp[0].1[..2_000_000], 9);
    z.extend(cells::witnesses().concat());
    let want = oracle(&z).unwrap();
    let got = with_plan(Plan::Workers(4), || mibz2::decode_parallel(&z, 4)).unwrap();
    assert!(got == want);
    let c = mibz2::gates::counts();
    assert_eq!((c[0].capacity, c[1].capacity, c[2].capacity), (1, 0, 2));
    assert!(c[0].entries > 0 && c[2].entries > 0 && c[1].entries == 0);
    let r = mibz2::gates::report();
    assert!(
        r.starts_with("{\"induction\":{\"capacity\":1,") && r.contains("\"writer\":null"),
        "{r}"
    );
    assert!(r.contains("\"blocks\":{\"capacity\":2,"), "{r}");
    // (a later change to the environment is not read)
    std::env::set_var("MIBZ2_GATE_WRITER", "3");
    assert_eq!(mibz2::gates::counts()[1].capacity, 0);
}
