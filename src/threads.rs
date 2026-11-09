//! Benchmarking only: the threads this process's decoders ran, by name, each recorded by
//! the thread itself. A worker records itself when it starts; a team's helper records itself
//! when it first runs a task, so a helper that never ran one (its pool dropped first, or
//! never spawned) is not recorded. Each is recorded under its own name (`std::thread::
//! Builder`'s), never the name it inherits from its spawner at birth, which an outside
//! observer polling `/proc` can catch alone (Guardian D059: a helper read under its leader's
//! name). Off unless a harness turns it on (`record`): off, each thread's one check is a
//! relaxed load; on, one lock and push a thread, once. `examples/ab.rs` reads it.

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

/// This thread ran (a worker, at its start; a helper, at its first task), if recording
pub(crate) fn ran() {
    if ON.load(Ordering::Relaxed) {
        let name = std::thread::current().name().unwrap_or("").to_string();
        RAN.lock().unwrap_or_else(|e| e.into_inner()).push(name);
    }
}

#[cfg(test)]
mod tests {
    use crate::coop::Pool;

    /// The record of the threads a pool's helpers ran: each under its own name once it ran
    /// a task, none before (a pool dropped first), none while recording is off. Pools of
    /// other tests may record too while this one records: only this test's prefix counts.
    #[test]
    fn helpers_record_their_own_names_once_they_ran() {
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
    }
}
