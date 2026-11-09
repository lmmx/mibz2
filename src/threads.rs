//! Benchmarking only: the threads this process's decoders ran, by name, each recorded by
//! the thread itself. A worker records itself when it starts; a team's helper records itself
//! when it has first run a task, so a helper that never ran one (its pool dropped first, or
//! never spawned) is not recorded. Each is recorded under its own name (`std::thread::
//! Builder`'s), never the name it inherits from its spawner at birth, which an outside
//! observer polling `/proc` can catch alone (Guardian D059: a helper read under its leader's
//! name, or a short-lived pool not seen at all). `examples/ab.rs` reads it.
//!
//! **What it records, and its scope** (Guardian D061): a worker's start and a helper's first
//! task *attempt* (a task that panicked, the panic caught by the pool, counts as attempted).
//! It records neither that the threads ran at the same time nor that their work succeeded.
//! It is complete for a decode that starts with recording on, before the decoder spawns any
//! thread, in a process whose record is otherwise empty (ab's fresh child), and that is read
//! (`take`) only once every recorded thread has been joined (the decoder dropped): a
//! thread's push happens before its join returns, and nothing pushes after it. Read earlier,
//! a thread not yet started is missing.
//!
//! **What it costs:** off (unless a harness turns it on, `record`), one relaxed load a
//! thread, once. On, each recorded thread copies its name into a new `String` and pushes it,
//! under one process-wide mutex, once: an allocation, the vector's growth (another
//! allocation now and then), and a lock that threads starting together can contend for. So
//! equal counts of recorded threads across arms do not make the cost equal: the threads'
//! timing decides any contention. It is small beside a decode, but not zero, and not
//! claimed equal.
//!
//! **It never unwinds** (the premise the pool's helper loop relies on: `ran` is called
//! between a task and the helper's `done` count, and a panic there would end the helper
//! without counting it, leaving its run waiting for ever): a poisoned lock is taken as it
//! is, and the one other failure, an allocation's, aborts the process rather than unwinding.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static ON: AtomicBool = AtomicBool::new(false);
static RAN: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Record (or stop recording) the threads the decoders run from here on
#[doc(hidden)]
pub fn record(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

/// The threads recorded so far, by name, in the order they recorded themselves (and the
/// record emptied)
#[doc(hidden)]
pub fn take() -> Vec<String> {
    std::mem::take(&mut *RAN.lock().unwrap_or_else(|e| e.into_inner()))
}

/// This thread ran (a worker, at its start; a helper, after its first task attempt), if
/// recording. Never unwinds (the module's premise): the lock's poisoning is ignored, and
/// allocation failure aborts.
pub(crate) fn ran() {
    if ON.load(Ordering::Relaxed) {
        let name = std::thread::current().name().unwrap_or("").to_string();
        RAN.lock().unwrap_or_else(|e| e.into_inner()).push(name);
    }
}

#[cfg(test)]
mod tests {
    use crate::coop::Pool;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    /// The record of the threads a pool's helpers ran: each under its own name once it ran
    /// a task, none before (a pool dropped first), none while recording is off; a helper
    /// whose task panicked (the panic caught, the run resuming it) is recorded, once, and its
    /// pool reused records nothing more. One test, as the switch is the process's: pools of
    /// other tests may record too while it records, so only its own prefixes count. Under
    /// Miri too (its race detector on the record's lock and the pool's hand-offs)
    #[test]
    fn miri_helpers_record_their_own_names_once_they_ran() {
        let mine = |v: &[String], p: &str| {
            let mut n: Vec<String> = v.iter().filter(|x| x.starts_with(p)).cloned().collect();
            n.sort();
            n
        };
        super::record(false);
        let p = Pool::named(3, "mibz2-test-off").unwrap();
        p.run(&|_| {});
        drop(p);
        assert!(
            mine(&super::take(), "mibz2-test-off").is_empty(),
            "recorded while off"
        );
        super::record(true);
        // a pool dropped before any task: its helpers never ran one
        drop(Pool::named(4, "mibz2-test-idle").unwrap());
        // a pool that ran twice: each helper once, under its own name
        let p = Pool::named(3, "mibz2-test-ran").unwrap();
        p.run(&|_| {});
        p.run(&|_| {});
        drop(p);
        // a helper's task panicking: the run resumes the panic, the helper is recorded (an
        // attempt), and the pool, reused, records nothing more
        let p = Pool::named(3, "mibz2-test-panic").unwrap();
        let r = catch_unwind(AssertUnwindSafe(|| {
            p.run(&|j| {
                if j == 1 {
                    panic!("a helper's task");
                }
            })
        }));
        assert!(r.is_err(), "the helper's panic resumed");
        p.run(&|_| {});
        drop(p);
        super::record(false);
        let got = super::take();
        assert!(
            mine(&got, "mibz2-test-idle").is_empty(),
            "an idle helper recorded"
        );
        assert_eq!(
            mine(&got, "mibz2-test-ran"),
            ["mibz2-test-ran1", "mibz2-test-ran2"]
        );
        assert_eq!(
            mine(&got, "mibz2-test-panic"),
            ["mibz2-test-panic1", "mibz2-test-panic2"]
        );
    }
}
