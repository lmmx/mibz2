//! The cooperative inverse BWT's thread pool: k − 1 helper threads kept for a worker's
//! life, so a block run on k cores pays no spawn (research: the coop journal,
//! docs/journal/2026-10-28-cooperative-inversion.md; the integration plan's step 2).
//! Nothing uses it yet.
//!
//! `Pool::run(f)` calls `f(0)` on the caller and `f(j)` on helper j, for j in 1..k, and
//! returns once all k calls have ended. Its guarantees, each tested below:
//! - **No helper touches `f` after `run` returns or unwinds.** A guard waits for every
//!   helper on every exit from `run`, the caller's own panic included. That is the one
//!   invariant the lifetime erasure in `run` rests on.
//! - **A panic never hangs the pool.** Each helper runs its call under `catch_unwind` and
//!   always reports that it ended. `run` then resumes the first panic caught, the caller's
//!   own first, so the decoder's worker turns it into `ErrorKind::Internal` as it does any
//!   other panic. The pool stays usable.
//! - **No wait depends on a thread that may not exist.** A helper that fails to spawn makes
//!   `Pool::new` an error, with the helpers already spawned shut down. Once spawned, a
//!   helper leaves its loop only when the pool is dropped. Its loop calls nothing that can
//!   panic outside `catch_unwind`, and it takes its locks poison-tolerantly.
//! - **Waiting is a short spin, then `park`.** Helpers between runs, and the caller waiting
//!   for them, spin briefly and then park, and are unparked by the event they wait for. An
//!   unpark that comes before the park makes the park return at once, so no wake-up is
//!   lost.
//!
//! The memory ordering: the task is published under a mutex and the epoch advanced with
//! `Release`; helpers load the epoch with `Acquire`. Each helper's writes are released by
//! its `done.fetch_add(AcqRel)`, and the caller acquires them by loading `done` with
//! `Acquire`. So everything the k calls wrote is visible to the caller when `run` returns.

use std::any::Any;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle, Thread};

/// A task for a pool, called once on each of its k threads with the thread's index (the
/// caller's is 0)
pub(crate) type Task<'a> = &'a (dyn Fn(usize) + Sync);

/// The epoch that tells the helpers to leave
const SHUTDOWN: usize = usize::MAX;

/// Spins before a wait parks: a pool's next run is usually microseconds away, a block's
/// MTF stage (about 0.4 ms) apart. Miri interprets each spin, so it parks almost at once.
const SPINS: u32 = if cfg!(miri) { 4 } else { 2_000 };

/// The task's fat pointer, its lifetime erased; it is called only inside the run that
/// posted it (the guard in `run`)
#[derive(Clone, Copy)]
struct Erased(*const (dyn Fn(usize) + Sync + 'static));
// SAFETY: it points at a `Sync` closure, and is dereferenced only while that closure lives
unsafe impl Send for Erased {}

struct Shared {
    /// advanced once a run (SHUTDOWN to leave)
    epoch: AtomicUsize,
    /// the helpers whose call in this run has ended, panicked or not
    done: AtomicUsize,
    /// the run's task, set before its epoch is published
    task: Mutex<Option<Erased>>,
    /// the first panic a helper caught in this run
    panic: Mutex<Option<Box<dyn Any + Send>>>,
    /// the thread waiting in `run`, for the last helper to unpark
    caller: Mutex<Option<Thread>>,
}

/// A lock that ignores poisoning: nothing is held across a call that could panic, but a
/// helper must never fail to report, whatever happened before it
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// k − 1 helper threads, the caller of `run` the k-th
pub(crate) struct Pool {
    k: usize,
    shared: Arc<Shared>,
    helpers: Vec<JoinHandle<()>>,
}

impl Pool {
    /// A pool of `k` threads (`k` − 1 spawned; `k` of 0 is taken as 1). A helper that
    /// can't be spawned makes this an error, with those spawned before it shut down.
    pub(crate) fn new(k: usize) -> std::io::Result<Pool> {
        let k = k.max(1);
        let shared = Arc::new(Shared {
            epoch: AtomicUsize::new(0),
            done: AtomicUsize::new(0),
            task: Mutex::new(None),
            panic: Mutex::new(None),
            caller: Mutex::new(None),
        });
        let mut pool = Pool {
            k: 1,
            shared,
            helpers: Vec::with_capacity(k - 1),
        };
        for idx in 1..k {
            let shared = Arc::clone(&pool.shared);
            let h = thread::Builder::new()
                .name(format!("mibz2-coop-{idx}"))
                .spawn(move || helper(&shared, idx))?;
            // (on an error above, `pool` drops with the helpers spawned so far, its k
            // counting only them)
            pool.helpers.push(h);
            pool.k = idx + 1;
        }
        Ok(pool)
    }

    /// The pool's threads, the caller included
    pub(crate) fn threads(&self) -> usize {
        self.k
    }

    /// `f(0)` on the caller and `f(j)` on helper j, returning when all k have ended. A
    /// panic in any of them is resumed here once all have ended: the caller's own if it
    /// panicked, else the first a helper caught.
    pub(crate) fn run(&self, f: Task) {
        if self.k == 1 {
            return f(0);
        }
        let s = &*self.shared;
        // SAFETY: only the lifetime is erased. The guard below keeps `run` from returning
        // or unwinding until every helper's call has ended, and a helper reads the task
        // only after this run's epoch is published, so no helper uses it after `f` ends
        let raw: *const (dyn Fn(usize) + Sync + '_) = f;
        let erased = Erased(unsafe {
            std::mem::transmute::<
                *const (dyn Fn(usize) + Sync + '_),
                *const (dyn Fn(usize) + Sync + 'static),
            >(raw)
        });
        *lock(&s.caller) = Some(thread::current());
        *lock(&s.task) = Some(erased);
        s.done.store(0, Ordering::Relaxed);
        s.epoch.fetch_add(1, Ordering::Release);
        for h in &self.helpers {
            h.thread().unpark();
        }
        let guard = Wait {
            s,
            helpers: self.k - 1,
        };
        let own = catch_unwind(AssertUnwindSafe(|| f(0)));
        drop(guard);
        *lock(&s.task) = None;
        let caught = lock(&s.panic).take();
        if let Err(p) = own {
            resume_unwind(p);
        }
        if let Some(p) = caught {
            resume_unwind(p);
        }
    }
}

/// Waits, when dropped, until every helper's call in the run has ended
struct Wait<'a> {
    s: &'a Shared,
    helpers: usize,
}

impl Drop for Wait<'_> {
    fn drop(&mut self) {
        let mut spins = 0u32;
        while self.s.done.load(Ordering::Acquire) != self.helpers {
            spins += 1;
            if spins < SPINS {
                std::hint::spin_loop();
            } else {
                thread::park();
            }
        }
    }
}

fn helper(s: &Shared, idx: usize) {
    let mut seen = 0;
    loop {
        let mut spins = 0u32;
        let e = loop {
            let e = s.epoch.load(Ordering::Acquire);
            if e != seen {
                break e;
            }
            spins += 1;
            if spins < SPINS {
                std::hint::spin_loop();
            } else {
                thread::park();
            }
        };
        if e == SHUTDOWN {
            return;
        }
        seen = e;
        let task = *lock(&s.task);
        if let Some(Erased(f)) = task {
            // SAFETY: the run that published epoch `e` is waiting for this call (`Wait`)
            let r = catch_unwind(AssertUnwindSafe(|| unsafe { (*f)(idx) }));
            if let Err(p) = r {
                let mut slot = lock(&s.panic);
                if slot.is_none() {
                    *slot = Some(p);
                }
            }
        }
        let caller = lock(&s.caller).clone();
        // (the last helper to end wakes the caller; an earlier unpark is harmless)
        s.done.fetch_add(1, Ordering::AcqRel);
        if let Some(c) = caller {
            c.unpark();
        }
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        self.shared.epoch.store(SHUTDOWN, Ordering::Release);
        for h in &self.helpers {
            h.thread().unpark();
        }
        for h in self.helpers.drain(..) {
            // (a helper's calls run under catch_unwind, so it ends without panicking)
            let _ = h.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    fn each_index_once(p: &Pool) {
        let k = p.threads();
        let hits: Vec<AtomicUsize> = (0..k).map(|_| AtomicUsize::new(0)).collect();
        p.run(&|j| {
            hits[j].fetch_add(1, Ordering::Relaxed);
        });
        assert!(hits.iter().all(|h| h.load(Ordering::Relaxed) == 1));
    }

    #[test]
    fn miri_pool_runs_every_index_once() {
        for k in [0, 1, 2, 3, 4] {
            let p = Pool::new(k).unwrap();
            assert_eq!(p.threads(), k.max(1));
            for _ in 0..3 {
                each_index_once(&p);
            }
        }
    }

    /// The helpers' plain writes into disjoint slots are the caller's to read when `run`
    /// returns (Miri's race detector checks the ordering)
    #[test]
    fn miri_pool_writes_visible_after_run() {
        struct Slots(*mut u64);
        // SAFETY: each thread writes only its own slots through it
        unsafe impl Sync for Slots {}
        let p = Pool::new(3).unwrap();
        let mut v = vec![0u64; 3 * 64];
        for round in 1..=3u64 {
            let slots = Slots(v.as_mut_ptr());
            let slots = &slots;
            p.run(&|j| {
                for i in j * 64..(j + 1) * 64 {
                    // SAFETY: thread j writes only its own 64 slots, inside v
                    unsafe { *slots.0.add(i) = round * 1000 + i as u64 };
                }
            });
            assert!(v
                .iter()
                .enumerate()
                .all(|(i, &x)| x == round * 1000 + i as u64));
        }
    }

    #[test]
    fn miri_pool_helper_panic_is_resumed_and_the_pool_reusable() {
        let p = Pool::new(3).unwrap();
        for _ in 0..2 {
            let r = catch_unwind(AssertUnwindSafe(|| {
                p.run(&|j| {
                    if j == 2 {
                        panic!("helper {j}");
                    }
                })
            }));
            let msg = r.unwrap_err();
            assert_eq!(
                msg.downcast_ref::<String>().map(String::as_str),
                Some("helper 2")
            );
            each_index_once(&p);
        }
    }

    /// The caller's panic waits for the helpers first: they borrow the caller's frame, so
    /// unwinding early would let them use it after it is gone (Miri checks the borrow)
    #[test]
    fn miri_pool_caller_panic_waits_for_the_helpers() {
        let p = Pool::new(3).unwrap();
        let ended = AtomicUsize::new(0);
        let r = catch_unwind(AssertUnwindSafe(|| {
            let local = vec![7u8; 16];
            p.run(&|j| {
                if j == 0 {
                    panic!("caller");
                }
                for _ in 0..50 {
                    std::hint::black_box(&local);
                    thread::yield_now();
                }
                assert_eq!(local[j], 7);
                ended.fetch_add(1, Ordering::Relaxed);
            })
        }));
        assert_eq!(*r.unwrap_err().downcast_ref::<&str>().unwrap(), "caller");
        assert_eq!(
            ended.load(Ordering::Relaxed),
            2,
            "the helpers ended before run returned"
        );
        each_index_once(&p);
    }

    /// The caller's panic is the one resumed when helpers panic too
    #[test]
    fn miri_pool_caller_panic_wins() {
        let p = Pool::new(2).unwrap();
        let r = catch_unwind(AssertUnwindSafe(|| {
            p.run(&|j| panic!("{}", if j == 0 { "caller" } else { "helper" }))
        }));
        assert_eq!(r.unwrap_err().downcast_ref::<String>().unwrap(), "caller");
        each_index_once(&p);
    }

    #[test]
    fn miri_pool_dropped_while_parked() {
        let p = Pool::new(4).unwrap();
        each_index_once(&p);
        // long enough for every helper to have parked
        thread::sleep(std::time::Duration::from_millis(if cfg!(miri) {
            1
        } else {
            20
        }));
        drop(p);
        drop(Pool::new(3).unwrap()); // never run
    }

    /// Many short runs back to back, the race between a helper's park and the next run's
    /// unpark, and the caller's park and the last helper's unpark
    #[test]
    fn pool_many_short_runs() {
        let p = Pool::new(4).unwrap();
        let sum = AtomicU64::new(0);
        let n = 20_000u64;
        for i in 0..n {
            p.run(&|j| {
                sum.fetch_add(i * 4 + j as u64, Ordering::Relaxed);
            });
            if i % 1000 == 0 {
                // let the helpers' spins run out and park between some runs
                thread::sleep(std::time::Duration::from_micros(200));
            }
        }
        let want: u64 = (0..n).map(|i| 4 * (i * 4) + 6).sum();
        assert_eq!(sum.load(Ordering::Relaxed), want);
    }

    /// Pools kept by several threads at once, as the decoder's workers would keep them
    #[test]
    fn pool_per_worker() {
        thread::scope(|sc| {
            for _ in 0..3 {
                sc.spawn(|| {
                    let p = Pool::new(3).unwrap();
                    for _ in 0..2_000 {
                        each_index_once(&p);
                    }
                });
            }
        });
    }
}
