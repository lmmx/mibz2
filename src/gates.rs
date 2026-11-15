//! Research only, behind the `phase-gates` feature (off by default): the four-worker
//! phase-concurrency screen's gates (#145, registration LOG 6893071988, priced by the
//! Optimiser's REVIEW 6893060396). Without the feature this module is not compiled and
//! nothing in the decoder refers to it. It is removed when the screen's verdict is recorded.
//!
//! **What it is.** Three sleeping counting gates, each a FIFO counting semaphore of a
//! `Mutex` and a `Condvar` (std only, no spinning of its own):
//! - **induction**, entered immediately before Cells' `induce_cells` and left as soon as it
//!   returns;
//! - **writer**, entered immediately before `write_cells` and left as soon as it returns;
//! - **blocks**, a whole-block lease, entered by a worker of the parallel decoder's Workers
//!   path once it holds a job, before the block's `decode_block`, and left once it returns:
//!   the blocks in flight inside the workers (arms E2 and E3), with the scanner's window and
//!   the four worker threads unchanged.
//!
//! Each permit is RAII: it is left when dropped, on every path, a refusal's, an unwind's and
//! a cancellation's included. Both phase gates are entered *outside* the `phase-counters`
//! scopes (the permit made before the scope and dropped after it), so a wait is never inside
//! a phase window. The induction's permit is dropped before the writer's gate is entered
//! (asserted per thread), so a worker never holds an induction permit while it waits for a
//! writer one; the lease is the only permit held across another gate's wait, always in the
//! one order blocks, then induction or writer.
//!
//! **Capacities** are read once a process from `MIBZ2_GATE_INDUCTION`, `MIBZ2_GATE_WRITER`
//! and `MIBZ2_GATE_BLOCKS`: unset, empty or 0 is no gate (entering it is free and counts
//! nothing). Tests set them with `configure`.
//!
//! **Counts, per gate:** the entries, the entries that waited (found the gate full, or
//! another thread before them in line), the total wait in ns (from the first finding to the
//! permit), and the most permits ever held at once. `report` gives them as one JSON object
//! for `examples/ab.rs`'s child record, or `null` where no gate is set.

use std::cell::Cell;
use std::sync::{Condvar, Mutex, MutexGuard, Once};
use std::time::Instant;

/// The gates
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    /// before `induce_cells`
    Induction = 0,
    /// before `write_cells`
    Writer = 1,
    /// a worker's whole-block lease, before `decode_block`
    Blocks = 2,
}

/// The gates' names, as the report gives them
pub const NAMES: [&str; 3] = ["induction", "writer", "blocks"];
/// The environment variables their capacities are read from, once a process
pub const VARS: [&str; 3] = [
    "MIBZ2_GATE_INDUCTION",
    "MIBZ2_GATE_WRITER",
    "MIBZ2_GATE_BLOCKS",
];

/// A gate's counts
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// 0: no gate
    pub capacity: usize,
    pub entries: u64,
    /// the entries that found the gate full (or a thread before them in line) and slept
    pub waited: u64,
    /// their total wait, ns
    pub wait_ns: u64,
    /// the most permits held at once
    pub max_held: usize,
}

struct State {
    held: usize,
    /// the next ticket to give, and the ticket whose turn it is: first come, first served
    next: u64,
    serving: u64,
    counts: Counts,
}

struct Sem {
    st: Mutex<State>,
    cv: Condvar,
}

impl Sem {
    const fn new() -> Sem {
        Sem {
            st: Mutex::new(State {
                held: 0,
                next: 0,
                serving: 0,
                counts: Counts {
                    capacity: 0,
                    entries: 0,
                    waited: 0,
                    wait_ns: 0,
                    max_held: 0,
                },
            }),
            cv: Condvar::new(),
        }
    }
    fn lock(&self) -> MutexGuard<'_, State> {
        self.st.lock().unwrap_or_else(|p| p.into_inner())
    }
}

static SEMS: [Sem; 3] = [Sem::new(), Sem::new(), Sem::new()];
static INIT: Once = Once::new();

thread_local! {
    /// the permits this thread holds, a bit a gate
    static HELD: Cell<u8> = const { Cell::new(0) };
}

fn init() {
    INIT.call_once(|| {
        for (sem, var) in SEMS.iter().zip(VARS) {
            let cap = std::env::var(var).unwrap_or_default();
            let cap = if cap.is_empty() {
                0
            } else {
                cap.parse::<usize>()
                    .unwrap_or_else(|_| panic!("{var} is a whole number (0 or unset: no gate)"))
            };
            sem.lock().counts.capacity = cap;
        }
    });
}

/// Set the capacities (`[induction, writer, blocks]`, 0 no gate) and clear the counts, in
/// place of the environment's (tests only: no permit may be held, nor any thread waiting)
pub fn configure(caps: [usize; 3]) {
    init();
    for (sem, cap) in SEMS.iter().zip(caps) {
        let mut st = sem.lock();
        assert!(
            st.held == 0 && st.next == st.serving,
            "configure with a gate in use"
        );
        st.counts = Counts {
            capacity: cap,
            ..Counts::default()
        };
    }
}

/// A permit: the gate is left when it is dropped. `None`: no gate was set
#[must_use]
pub struct Permit(Option<Gate>);

/// Enter gate `g`: at once if it has room and no thread is before this one in line, else
/// asleep until it is this thread's turn and there is room
#[inline]
pub fn enter(g: Gate) -> Permit {
    init();
    let bit = 1u8 << g as u8;
    HELD.with(|h| {
        let held = h.get();
        assert!(
            held & bit == 0,
            "the {} gate entered twice",
            NAMES[g as usize]
        );
        assert!(
            g != Gate::Writer || held & (1 << Gate::Induction as u8) == 0,
            "the writer's gate entered with an induction permit held"
        );
    });
    let sem = &SEMS[g as usize];
    let mut st = sem.lock();
    let cap = st.counts.capacity;
    if cap == 0 {
        return Permit(None);
    }
    st.counts.entries += 1;
    let ticket = st.next;
    st.next += 1;
    if ticket != st.serving || st.held >= cap {
        st.counts.waited += 1;
        let t0 = Instant::now();
        while ticket != st.serving || st.held >= st.counts.capacity {
            st = sem.cv.wait(st).unwrap_or_else(|p| p.into_inner());
        }
        st.counts.wait_ns += t0.elapsed().as_nanos() as u64;
    }
    st.serving += 1;
    st.held += 1;
    st.counts.max_held = st.counts.max_held.max(st.held);
    // (the next in line may go too, if there is room)
    let queued = st.next != st.serving;
    drop(st);
    if queued {
        sem.cv.notify_all();
    }
    HELD.with(|h| h.set(h.get() | bit));
    Permit(Some(g))
}

impl Drop for Permit {
    #[inline]
    fn drop(&mut self) {
        let Some(g) = self.0 else { return };
        HELD.with(|h| h.set(h.get() & !(1u8 << g as u8)));
        let sem = &SEMS[g as usize];
        let mut st = sem.lock();
        st.held -= 1;
        let queued = st.next != st.serving;
        drop(st);
        if queued {
            sem.cv.notify_all();
        }
    }
}

/// The gates' counts so far, `[induction, writer, blocks]`
pub fn counts() -> [Counts; 3] {
    init();
    [0, 1, 2].map(|i| SEMS[i].lock().counts)
}

/// The counts as one JSON object, a gate `null` where it is not set, or `null` where none is
pub fn report() -> String {
    let c = counts();
    if c.iter().all(|c| c.capacity == 0) {
        return "null".into();
    }
    let parts: Vec<String> = NAMES
        .iter()
        .zip(c)
        .map(|(name, c)| {
            if c.capacity == 0 {
                format!("\"{name}\":null")
            } else {
                format!(
                    "\"{name}\":{{\"capacity\":{},\"entries\":{},\"waited\":{},\"wait_ns\":{},\"max_held\":{}}}",
                    c.capacity, c.entries, c.waited, c.wait_ns, c.max_held
                )
            }
        })
        .collect();
    format!("{{{}}}", parts.join(","))
}
