//! Research only (`phase-counters`): the phase counters' paths, driven for real on a machine
//! with no PMU by the software pass `SW` (task clock, page faults). Through the Workers plan
//! the instrument opens its groups on each worker, counts each block's decode_block, the
//! induction's and the writer's windows and their empty scopes, and the output stays
//! byte-identical. Where even software events cannot be opened (a locked-down
//! perf_event_paranoid), the failure is recorded and nothing is counted, and the output is
//! still byte-identical.
//!
//!     cargo test --features phase-counters --test phase
#![cfg(feature = "phase-counters")]
mod common;
use common::{cells, compress, corpora, oracle};
use mibz2::parallel::{with_plan, Plan};
use mibz2::phase;

#[test]
fn the_counters_count_each_scope_and_change_no_byte() {
    // (before any decode: the pass is read once a process)
    std::env::set_var("MIBZ2_PHASE_PASS", "SW");
    assert_eq!(phase::pass().map(|p| p.0), Some("SW"));
    let corp = corpora(1);
    let mut inputs = vec![
        compress(&corp[0].1[..2_000_000], 9),
        compress(&b"abacaba".repeat(200_000), 9),
    ];
    inputs.extend(cells::witnesses());
    for z in &inputs {
        let want = oracle(z).unwrap();
        for t in [1, 2, 4] {
            let got = with_plan(Plan::Workers(t), || mibz2::decode_parallel(z, t)).unwrap();
            assert!(got == want, "Workers({t}): the bytes");
        }
        // (the calling thread's decode: no worker scope, so nothing counted, nothing broken)
        assert!(mibz2::decode(z).unwrap() == want);
    }
    let s = phase::totals();
    let report = phase::report();
    assert!(report.starts_with("{\"pass\":\"SW\""), "{report}");
    assert!(report.contains("\"modes\":\"user+kernel\""), "{report}");
    if !s.errors.is_empty() {
        // (no software events here either: recorded, and nothing counted)
        assert!(
            s.errors.iter().all(|e| e.contains("perf_event_open")),
            "{:?}",
            s.errors
        );
        eprintln!(
            "phase counters not opened here ({:?}): the failure path was taken",
            s.errors
        );
        return;
    }
    assert!(s.threads > 0 && s.blocks > 0, "{report}");
    assert_eq!(s.whole.windows, s.blocks, "a whole window a block");
    let (ind, wr) = (s.phase[0], s.phase[1]);
    assert!(
        ind.windows > 0 && wr.windows > 0,
        "Cells' scopes were entered: {report}"
    );
    // (an induction the budget refuses has no writer after it: Segmented inverts that block,
    // inside the whole window, so in rest)
    assert!(
        wr.windows <= ind.windows,
        "a writer's scope only after an induction"
    );
    assert_eq!(s.whole_in[0].windows, ind.windows);
    assert_eq!(s.whole_in[1].windows, wr.windows);
    assert_eq!(
        s.empty.windows,
        ind.windows + wr.windows,
        "an empty scope for each scope"
    );
    // (software events never multiplex; the task clock is nested: a phase window inside
    // the whole group's reading of it, inside the blocks' whole windows)
    for w in [s.whole, ind, wr, s.whole_in[0], s.whole_in[1], s.empty] {
        assert_eq!(w.mux, 0, "{report}");
        assert_eq!(w.enabled, w.running);
    }
    for p in 0..2 {
        assert!(s.phase[p].v[0] > 0, "the task clock ran in phase {p}");
        assert!(
            s.phase[p].v[0] <= s.whole_in[p].v[0],
            "phase {p} inside the whole's window"
        );
    }
    assert!(s.whole_in[0].v[0] + s.whole_in[1].v[0] <= s.whole.v[0]);
    // (an empty scope is short: on average under 1 ms of task clock)
    assert!(s.empty.v[0] < 1_000_000 * s.empty.windows, "{report}");
}
