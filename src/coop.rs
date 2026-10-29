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

use crate::bwt::{read_u64, TVector};

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

/// Why `build_fused` refused its inputs or found its table wrong: a bug in the caller (the
/// MTF stage's checkpoints, or its counts), never an input error, since the last column
/// and its counts come from the decoder itself
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BuildError(pub(crate) &'static str);

/// The rows `j` of `k` ranges starts at: n·j/k rounded to a multiple of `stride` (so a
/// checkpoint gives its counts), the last range ending at n
fn edge(n: usize, k: usize, stride: usize, j: usize) -> usize {
    if j >= k {
        n
    } else {
        ((n * j / k + stride / 2) / stride * stride).min(n)
    }
}

/// A shared table's base pointer, for the pool's threads to write disjoint slots through
struct Slots(*mut u32);
// SAFETY: each thread writes only the slots of its own windows (checked in `build_fused`)
unsafe impl Sync for Slots {}

/// The T-vector of `ll` (whose bytes `counts` counts) built by the pool's k threads in one
/// pass, `bwt::build`'s table word for word. The rows are cut into k ranges at multiples
/// of `stride` (`mtf::CHECKPOINT` in the decoder), and `checkpoints[i]` is the counts of
/// `ll[..i * stride]`, so range j's first slot in bucket b is b's start plus
/// `checkpoints[start / stride][b]`, and its last is below b's start plus the next range's.
/// Each range writes each of its entries whole, `ll[s] | i << 8` at slot s, with
/// `bwt::build`'s 16-row run stores; there is no count, copy or barrier before the scatter
/// (the coop journal §16).
///
/// It is sound whatever `checkpoints` and `counts` hold, and the table is right only if
/// they are. Before the run (O(256) a checkpoint): the counts sum to n, the checkpoints
/// are one a whole stride, start at zero, sum to their offsets, rise and stay within the
/// counts. So each range's window in each bucket lies inside [0, n), and the ranges'
/// windows are disjoint. Each write is checked against its window's end, so no thread
/// writes outside its own windows. After the run, every range's every cursor must have
/// reached its window's end. That proves every slot written once. It also proves the
/// table right: the windows' sizes are then the bytes each range holds, so, summing from
/// the first range (whose windows start at the buckets' starts), every window starts
/// where `bwt::build` would put that range's first byte of its bucket. Only then is the
/// table's length set, so a refused or failed build leaves it empty, never stale or
/// uninitialised.
pub(crate) fn build_fused(
    ll: &[u8],
    counts: &[u32; 256],
    checkpoints: &[[u32; 256]],
    stride: usize,
    pool: &Pool,
    tt: &mut TVector,
) -> Result<(), BuildError> {
    let n = ll.len();
    let tt = &mut tt.0;
    tt.clear();
    if stride == 0 || n >= 1 << 24 {
        return Err(BuildError("a zero stride or a block of 2^24 rows or more"));
    }
    if counts.iter().map(|&c| c as usize).sum::<usize>() != n {
        return Err(BuildError("the counts don't sum to the block's length"));
    }
    if checkpoints.len() != n / stride + 1 {
        return Err(BuildError("not one checkpoint a whole stride"));
    }
    let mut last = &[0u32; 256];
    for (i, cp) in checkpoints.iter().enumerate() {
        let rises = cp.iter().zip(last).all(|(&c, &l)| c >= l);
        let within = cp.iter().zip(counts).all(|(&c, &t)| c <= t);
        let sums = cp.iter().map(|&c| c as usize).sum::<usize>() == i * stride;
        if !(rises && within && sums) {
            return Err(BuildError(
                "a checkpoint that falls, passes the counts or misses its offset",
            ));
        }
        last = cp;
    }
    let mut starts = [0u32; 256];
    let mut cum = 0u32;
    for b in 0..256 {
        starts[b] = cum;
        cum += counts[b];
    }
    tt.reserve(n);
    let k = pool.threads();
    let slots = Slots(tt.as_mut_ptr());
    let slots = &slots;
    let bad = AtomicUsize::new(0);
    // the counts before row e (a range's edge: a multiple of stride, or n)
    let at = |e: usize| {
        if e == n {
            counts
        } else {
            &checkpoints[e / stride]
        }
    };
    pool.run(&|j| {
        let (lo, hi) = (edge(n, k, stride, j), edge(n, k, stride, j + 1));
        let (before, after) = (at(lo), at(hi));
        let mut ftab = [0u32; 256];
        let mut lim = [0u32; 256];
        for b in 0..256 {
            ftab[b] = starts[b] + before[b];
            lim[b] = starts[b] + after[b];
        }
        // the one-row step: row i's link at its byte's next slot, inside the window
        let one = |i: usize, ftab: &mut [u32; 256]| -> bool {
            let c = ll[i] as usize;
            let s = ftab[c];
            if s >= lim[c] {
                return false;
            }
            // SAFETY: s is inside this range's window of bucket c, which lies in [0, n)
            // (checked before the run) and which no other range's window overlaps
            unsafe { *slots.0.add(s as usize) = ll[s as usize] as u32 | (i as u32) << 8 };
            ftab[c] = s + 1;
            true
        };
        let mut i = lo;
        let mut ok = true;
        'rows: while i + 16 <= hi {
            let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
            if a == b && a == a.rotate_left(8) {
                // 16 equal bytes: 16 consecutive slots of their bucket, in one store each
                let c = ll[i] as usize;
                let s = ftab[c] as usize;
                if s + 16 > lim[c] as usize {
                    ok = false;
                    break 'rows;
                }
                let base = (i as u32) << 8;
                for (jj, &l) in ll[s..s + 16].iter().enumerate() {
                    // SAFETY: s..s + 16 is inside this range's window of bucket c (as above)
                    unsafe { *slots.0.add(s + jj) = l as u32 | (base + ((jj as u32) << 8)) };
                }
                ftab[c] += 16;
            } else {
                for ii in i..i + 16 {
                    if !one(ii, &mut ftab) {
                        ok = false;
                        break 'rows;
                    }
                }
            }
            i += 16;
        }
        while ok && i < hi {
            ok = one(i, &mut ftab);
            i += 1;
        }
        // the end check: every window filled exactly
        if !ok || ftab != lim {
            bad.fetch_add(1, Ordering::Relaxed);
        }
    });
    if bad.load(Ordering::Relaxed) != 0 {
        return Err(BuildError("a range's bytes didn't fill its windows: the checkpoints or counts are not the block's"));
    }
    // SAFETY: every slot below n was written exactly once (the end check, by every range,
    // whose windows tile [0, n)), and the pool's run has ended, its writes visible here
    unsafe { tt.set_len(n) };
    Ok(())
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

    fn xorshift(seed: u64) -> impl FnMut() -> u64 {
        let mut x = seed | 1;
        move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        }
    }

    /// Last columns of several shapes and lengths
    fn columns(scale: usize) -> Vec<(String, Vec<u8>)> {
        let mut r = xorshift(0x9e37_79b9_7f4a_7c15);
        let mut runs = Vec::new();
        while runs.len() < 3 * scale {
            let c = (r() % 5) as u8;
            let len = [1, 2, 15, 16, 17, 31, 33, 200][(r() % 8) as usize];
            runs.extend(std::iter::repeat(c).take(len));
        }
        let mut v: Vec<(String, Vec<u8>)> = vec![
            ("empty".into(), vec![]),
            ("one byte".into(), vec![9]),
            ("one byte value".into(), vec![42; 2 * scale + 7]),
            ("period 3".into(), b"abc".repeat(scale)),
            (
                "random".into(),
                (0..3 * scale + 5).map(|_| r() as u8).collect(),
            ),
            (
                "few bytes".into(),
                (0..2 * scale).map(|_| b"ab"[(r() & 1) as usize]).collect(),
            ),
            ("runs".into(), runs),
        ];
        for n in [2, 15, 16, 17, 33, 63, 64, 65] {
            v.push((format!("random {n}"), (0..n).map(|_| r() as u8).collect()));
        }
        v
    }

    fn counts_of(ll: &[u8]) -> [u32; 256] {
        let mut c = [0u32; 256];
        ll.iter().for_each(|&b| c[b as usize] += 1);
        c
    }

    fn checkpoints_of(ll: &[u8], stride: usize) -> Vec<[u32; 256]> {
        let mut cum = [0u32; 256];
        let mut v = vec![cum];
        for chunk in ll.chunks_exact(stride) {
            chunk.iter().for_each(|&b| cum[b as usize] += 1);
            v.push(cum);
        }
        v
    }

    fn fused_matches_build(p: &Pool, stride: usize, scale: usize) {
        // one table kept across every column, growing and shrinking, as a worker keeps it
        let mut tt = TVector::new();
        for (name, ll) in columns(scale) {
            let counts = counts_of(&ll);
            let mut want = TVector::new();
            crate::bwt::build(&ll, &counts, &mut want);
            let cps = checkpoints_of(&ll, stride);
            build_fused(&ll, &counts, &cps, stride, p, &mut tt).unwrap();
            assert!(
                tt.0 == want.0,
                "{name}: k {}, stride {stride}: the fused table differs from bwt::build's",
                p.threads()
            );
        }
    }

    #[test]
    fn build_fused_is_build() {
        for k in 1..=4 {
            let p = Pool::new(k).unwrap();
            for stride in [16, 64, 1000, crate::mtf::CHECKPOINT] {
                fused_matches_build(&p, stride, 20_000);
            }
        }
    }

    /// Wrong checkpoints or counts are refused, or caught by the end check: an error, the
    /// table empty, and no write outside a window (Miri, in the miri_ test below)
    fn refuses(p: &Pool, stride: usize, scale: usize) {
        let mut r = xorshift(7);
        let ll: Vec<u8> = (0..4 * stride * scale + 9)
            .map(|_| (r() % 7) as u8)
            .collect();
        let counts = counts_of(&ll);
        let cps = checkpoints_of(&ll, stride);
        let mut tt = TVector::new();
        build_fused(&ll, &counts, &cps, stride, p, &mut tt).unwrap();
        let mut want = TVector::new();
        crate::bwt::build(&ll, &counts, &mut want);
        let check = |what: &str, counts: &[u32; 256], cps: &[[u32; 256]], tt: &mut TVector| {
            assert!(
                build_fused(&ll, counts, cps, stride, p, tt).is_err(),
                "{what}: k {}, accepted",
                p.threads()
            );
            assert!(tt.0.is_empty(), "{what}: the table was left set");
        };
        // a count moved from one byte to another at every checkpoint in turn, the sums kept:
        // refused, or caught by the end check, where a range starts there; accepted only
        // where no range reads it, and then the table is still bwt::build's
        let k = p.threads();
        let edges: Vec<usize> = (1..k).map(|j| edge(ll.len(), k, stride, j)).collect();
        let (from, to) = (ll[0] as usize, (ll[0] as usize + 1) % 7);
        let mut refused = 0;
        for m in 1..cps.len() {
            let mut moved = cps.clone();
            if moved[m][from] == 0 {
                continue;
            }
            moved[m][from] -= 1;
            moved[m][to] += 1;
            let r = build_fused(&ll, &counts, &moved, stride, p, &mut tt);
            if edges.contains(&(m * stride)) {
                assert!(
                    r.is_err(),
                    "a moved count at checkpoint {m}, a range's edge: k {k}, accepted"
                );
            }
            match r {
                Ok(()) => assert!(
                    tt.0 == want.0,
                    "a moved count at {m}: a wrong table accepted"
                ),
                Err(_) => {
                    assert!(tt.0.is_empty());
                    refused += 1;
                }
            }
        }
        assert!(
            k == 1 || refused > 0,
            "k {k}: no moved count reached an edge"
        );
        let mut falls = cps.clone();
        falls[1] = falls[2];
        check(
            "a checkpoint repeated out of place",
            &counts,
            &falls,
            &mut tt,
        );
        check("a checkpoint missing", &counts, &cps[1..], &mut tt);
        let mut more = counts;
        more[3] += 1;
        check("a count too many", &more, &cps, &mut tt);
        let mut swapped = counts;
        swapped.swap(1, 2);
        if swapped != counts {
            check("two counts swapped", &swapped, &cps, &mut tt);
        }
        assert!(
            build_fused(&ll, &counts, &cps, 0, p, &mut tt).is_err(),
            "a zero stride"
        );
        assert!(tt.0.is_empty());
        // and the same pool and table still build right afterwards
        build_fused(&ll, &counts, &cps, stride, p, &mut tt).unwrap();
        assert!(tt.0 == want.0);
    }

    #[test]
    fn build_fused_refuses_wrong_checkpoints() {
        for k in 1..=4 {
            refuses(&Pool::new(k).unwrap(), 256, 8);
        }
    }

    /// Small blocks under Miri: the disjoint slots (its race detector), the windows' bounds
    /// and the refusals, at k = 2 and 3
    #[test]
    fn miri_build_fused() {
        for k in [2, 3] {
            let p = Pool::new(k).unwrap();
            fused_matches_build(&p, 64, 80);
            refuses(&p, 64, 2);
        }
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
