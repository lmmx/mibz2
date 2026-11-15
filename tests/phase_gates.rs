//! Research only (`phase-gates`; #145, registration LOG 6893071988): the phase-concurrency
//! screen's gates through the Workers plan at 4 workers. At every capacity the output is
//! byte-identical, each gate counts every entry of its phase (the same entries whatever the
//! capacity), never holds more permits than its capacity, never waits where it has room for
//! every worker, and caps of 1 on every gate at once never deadlock. And the semaphore alone:
//! one at a time, no wait uncontended, no writer's permit with an induction's held.
//! (Here, not in the library's unit tests: the gates are the process's, and these tests take
//! turns, which the library's other tests, decoding with Cells at once, would not.)
//!
//!     cargo test --features phase-gates --test phase_gates
#![cfg(feature = "phase-gates")]
mod common;
use common::{cells, compress, corpora, oracle};
use mibz2::gates::{self, Counts};
use mibz2::parallel::{with_plan, Plan};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// (the gates are the process's: these tests take turns)
static TURN: Mutex<()> = Mutex::new(());

const WORKERS: usize = 4;

fn inputs() -> Vec<Vec<u8>> {
    let corp = corpora(1);
    let mut v = vec![
        compress(&corp[0].1[..2_000_000], 9),
        compress(&corp[0].1, 1),
        compress(&b"abacaba".repeat(200_000), 9),
    ];
    v.extend(cells::witnesses());
    v
}

/// Every input decoded at 4 workers with the gates at `caps`: the bytes checked against the
/// oracle's, and the gates' counts
fn run(inputs: &[(Vec<u8>, Vec<u8>)], caps: [usize; 3]) -> [Counts; 3] {
    gates::configure(caps);
    for (z, want) in inputs {
        let got = with_plan(Plan::Workers(WORKERS), || {
            mibz2::decode_parallel(z, WORKERS)
        })
        .unwrap();
        assert!(got == *want, "gates {caps:?}: the bytes");
    }
    gates::counts()
}

#[test]
fn every_capacity_gives_the_same_bytes_and_sane_counts() {
    let _t = TURN.lock().unwrap_or_else(|p| p.into_inner());
    let inputs: Vec<_> = inputs()
        .into_iter()
        .map(|z| {
            let want = oracle(&z).unwrap();
            (z, want)
        })
        .collect();
    // (no gate: nothing counted)
    assert_eq!(run(&inputs, [0, 0, 0]), [Counts::default(); 3]);
    assert_eq!(gates::report(), "null");
    // (K, the calibration arm: every gate at 4, room for every worker, so no wait ever)
    let k = run(&inputs, [4, 4, 4]);
    assert!(k[0].entries > 0 && k[1].entries > 0, "Cells engaged: {k:?}");
    // (an induction the budget refuses has no writer after it)
    assert!(k[1].entries <= k[0].entries);
    assert!(k[2].entries >= inputs.len() as u64, "a lease a block");
    for g in k {
        assert_eq!((g.capacity, g.waited, g.wait_ns), (4, 0, 0), "{k:?}");
        assert!(g.max_held >= 1 && g.max_held <= WORKERS);
    }
    let report = gates::report();
    assert!(
        report.starts_with("{\"induction\":{\"capacity\":4,") && report.contains("\"blocks\":{"),
        "{report}"
    );
    // (the registered arms B, C, D, E2, E3, and the caps of 1 everywhere)
    for caps in [
        [1, 4, 4],
        [4, 1, 4],
        [1, 1, 4],
        [4, 4, 2],
        [4, 4, 3],
        [1, 4, 0],
        [4, 1, 0],
        [1, 1, 0],
        [1, 1, 1],
    ] {
        let c = run(&inputs, caps);
        for (i, g) in c.iter().enumerate() {
            if caps[i] == 0 {
                assert_eq!(*g, Counts::default(), "gate {i} unset: {caps:?}");
                continue;
            }
            // (the same entries whatever the capacity: every phase entry passes its gate)
            assert_eq!(g.entries, k[i].entries, "gate {i} at {caps:?}");
            assert!(g.waited <= g.entries);
            assert!(
                g.max_held >= 1 && g.max_held <= caps[i],
                "gate {i} at {caps:?}: {g:?}"
            );
            assert!(
                g.waited == 0 || g.wait_ns > 0,
                "gate {i} at {caps:?}: {g:?}"
            );
            if caps[i] >= WORKERS {
                assert_eq!(g.waited, 0, "gate {i} at {caps:?}: room for every worker");
            }
        }
    }
    gates::configure([0, 0, 0]);
}

#[test]
fn caps_of_one_on_every_gate_never_deadlock() {
    let _t = TURN.lock().unwrap_or_else(|p| p.into_inner());
    // (the inputs in one decode each: Cells' blocks among many)
    let z: Vec<u8> = inputs().concat();
    let want = oracle(&z).unwrap();
    for caps in [[1, 1, 0], [1, 1, 1], [1, 1, 2]] {
        gates::configure(caps);
        let (tx, rx) = mpsc::channel();
        let zz = z.clone();
        std::thread::spawn(move || {
            for _ in 0..3 {
                let got = with_plan(Plan::Workers(WORKERS), || {
                    mibz2::decode_parallel(&zz, WORKERS)
                });
                tx.send(got).unwrap();
            }
        });
        for i in 0..3 {
            let got = rx
                .recv_timeout(Duration::from_secs(120))
                .unwrap_or_else(|_| panic!("gates {caps:?}: decode {i} did not finish"));
            assert!(got.unwrap() == want, "gates {caps:?}: the bytes");
        }
        let c = gates::counts();
        assert!(
            c[0].entries > 0 && c[0].max_held == 1 && c[1].max_held == 1,
            "{c:?}"
        );
    }
    gates::configure([0, 0, 0]);
}

// ---- the semaphore alone
#[test]
fn a_gate_of_one_admits_one_at_a_time() {
    let _t = TURN.lock().unwrap_or_else(|p| p.into_inner());
    gates::configure([1, 0, 0]);
    let inside = Arc::new(AtomicUsize::new(0));
    let most = Arc::new(AtomicUsize::new(0));
    let hs: Vec<_> = (0..4)
        .map(|_| {
            let (inside, most) = (inside.clone(), most.clone());
            std::thread::spawn(move || {
                for _ in 0..200 {
                    let _p = gates::enter(gates::Gate::Induction);
                    let n = inside.fetch_add(1, Ordering::SeqCst) + 1;
                    most.fetch_max(n, Ordering::SeqCst);
                    std::hint::spin_loop();
                    inside.fetch_sub(1, Ordering::SeqCst);
                }
            })
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    let c = gates::counts()[0];
    assert_eq!(most.load(Ordering::SeqCst), 1);
    assert_eq!((c.capacity, c.entries, c.max_held), (1, 800, 1));
    assert!(c.waited <= c.entries);
    assert!(c.waited == 0 || c.wait_ns > 0);
    // (the writer's gate unset: free, counted nowhere)
    drop(gates::enter(gates::Gate::Writer));
    assert_eq!(gates::counts()[1], Counts::default());
    assert!(
        gates::report().contains("\"writer\":null"),
        "{}",
        gates::report()
    );
    gates::configure([0, 0, 0]);
    assert_eq!(gates::report(), "null");
}

#[test]
fn an_uncontended_gate_never_waits() {
    let _t = TURN.lock().unwrap_or_else(|p| p.into_inner());
    gates::configure([4, 4, 4]);
    for _ in 0..10 {
        let _b = gates::enter(gates::Gate::Blocks);
        drop(gates::enter(gates::Gate::Induction));
        drop(gates::enter(gates::Gate::Writer));
    }
    let c = gates::counts();
    for g in c {
        assert_eq!(
            (g.capacity, g.entries, g.waited, g.wait_ns, g.max_held),
            (4, 10, 0, 0, 1)
        );
    }
    assert!(gates::report().starts_with("{\"induction\":{\"capacity\":4,\"entries\":10,"));
    gates::configure([0, 0, 0]);
}

#[test]
fn the_writer_is_never_entered_with_an_induction_permit() {
    let _t = TURN.lock().unwrap_or_else(|p| p.into_inner());
    gates::configure([1, 1, 0]);
    let r = std::thread::spawn(|| {
        let _i = gates::enter(gates::Gate::Induction);
        let _w = gates::enter(gates::Gate::Writer);
    })
    .join();
    assert!(
        r.is_err(),
        "entering the writer's gate holding induction panics"
    );
    // (the unwind left the induction's gate: it has room again)
    let c = gates::counts()[0];
    assert_eq!(c.entries, 1);
    drop(gates::enter(gates::Gate::Induction));
    assert_eq!(gates::counts()[0].waited, 0);
    gates::configure([0, 0, 0]);
}
