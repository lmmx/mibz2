//! Benchmarking only: the threads this process's decoders ran, by name, each recorded by
//! the thread itself. A worker records itself when it starts; a team's helper records itself
//! when it has first run a task, so a helper that never ran one (its pool dropped first, or
//! never spawned) is not recorded. Each is recorded under its own name (`std::thread::
//! Builder`'s), never the name it inherits from its spawner at birth, which an outside
//! observer polling `/proc` can catch alone (Guardian D059: a helper read under its leader's
//! name, or a short-lived pool not seen at all). `examples/ab.rs` reads it.
//!
//! **What it records, and its scope** (Guardian D061, D062): a worker's start and a
//! helper's first task *attempt* (a task that panicked, the panic caught by the pool, counts
//! as attempted). It records neither that the threads ran at the same time nor that their
//! work succeeded. Every entry is a thread of a decode that ran while recording was on.
//!
//! **Complete for every thread that contributed to a complete output, by its completion
//! order, not by any join** (D062):
//! - a helper records before it counts its task's completion (`done`, in `coop::helper`),
//!   and `Pool::run` returns only once every helper of the run has counted (`Wait`'s
//!   acquire), so a run that ended had every helper's record first;
//! - a worker records before its first job, so every block it publishes comes after its
//!   record;
//! - a block reaches the reader only through `publish` and the reader's own lock, so every
//!   byte read comes after the record of every worker that published it and of every helper
//!   of every team run that built it;
//! - so a harness that reads the decode's whole output and then `take`s the record holds
//!   every such thread's entry, in every arm: the shipped workers and the run-fed ones are
//!   spawned by `parallel::launch`, and every team runs on `Pool::run`.
//!
//! A thread that contributed nothing (a worker that published no block, a pool that ran no
//! task) may or may not be recorded. So the record can fall short of a plan that was set up
//! but not run, never exceed the threads that ran. The record says nothing about teardown:
//! the decoder's threads may still be ending, its join may come later (its last reference
//! can be a detached feeder's, `BzReader::drop`), and a thread that records after `take` is
//! simply not in it.
//!
//! **What it costs, and where:** off (unless a harness turns it on, `record`), one relaxed
//! load a thread, once. On, each recorded thread copies its name into a new `String` and
//! pushes it, under one process-wide mutex, once: an allocation, the vector's growth
//! (another allocation now and then), and a lock that threads starting together can
//! contend for. The calls sit inside every run's timing: a worker records before its first
//! job; a helper records after its first task and **before it publishes that task's
//! completion** (`done`), so a contended push can extend that run's completion. Equal counts
//! of recorded threads across arms do not make the cost equal: when threads start, and so
//! what they contend with, differs by arm, so this instrumentation can be differential.
//! Its cost is not measured here, and no claim that it is small, uncontended or equal across
//! arms is made. (D059 recommended observation outside the timed region; this design
//! replaces that recommendation with a record taken inside it, and does not meet it.)
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
    use std::sync::Mutex;

    /// (the record's switch is the process's: its tests take turns)
    static TURN: Mutex<()> = Mutex::new(());

    /// The record of the threads a pool's helpers ran: each under its own name once it ran
    /// a task, none before (a pool dropped first), none while recording is off; a helper
    /// whose task panicked (the panic caught, the run resuming it) is recorded, once, and its
    /// pool reused records nothing more. One test, as the switch is the process's: pools of
    /// other tests may record too while it records, so only its own prefixes count. Under
    /// Miri too (its race detector on the record's lock and the pool's hand-offs)
    #[test]
    fn miri_helpers_record_their_own_names_once_they_ran() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
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

    /// The record complete by completion, not by a join (D062): a pipelined team's decode
    /// read to its end, the record taken while the decoder still lives (nothing joined),
    /// holds its worker and both helpers
    #[cfg(not(miri))]
    #[test]
    fn the_record_is_whole_once_the_output_is_read_before_any_join() {
        use std::io::Write;
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        let mut x = 7u64;
        let mut chunk = |n: usize| -> Vec<u8> {
            (0..n)
                .map(|_| {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    b"abcdefgh"[(x % 8) as usize]
                })
                .collect()
        };
        let mut bz = Vec::new();
        for _ in 0..3 {
            let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(9));
            e.write_all(&chunk(crate::bwt::MIN_COOP + 20_000)).unwrap();
            bz.extend(e.finish().unwrap());
        }
        super::record(true);
        let dec = crate::StreamDecoder::pipelined(1, 3, 2, crate::Kernels::detect());
        dec.write(bz).unwrap();
        dec.close();
        let mut buf = vec![0u8; 1 << 16];
        while dec.read(&mut buf).unwrap() > 0 {}
        // (the whole output read; the decoder not dropped, nothing joined)
        super::record(false);
        let got = super::take();
        drop(dec);
        for name in ["mibz2-0", "mibz2-coop-p1", "mibz2-coop-p2"] {
            assert!(
                got.iter().any(|n| n == name),
                "{name} not recorded: {got:?}"
            );
        }
    }
}
