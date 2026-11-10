//! The cooperative inverse BWT's thread pool: k − 1 helper threads kept for a worker's
//! life, so a block run on k cores pays no spawn (research: the coop journal,
//! docs/journal/2026-10-28-cooperative-inversion.md; the integration plan's step 2).
//! `bwt::Cooperative` runs on it: each of the decoder's teams (`Plan::Teams`, which
//! `Plan::for_threads` picks where the threads and the physical cores both reach
//! `COOP_MIN_CORES`) keeps one pool for its worker's life.
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
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle, Thread};

use crate::bwt::{prefetch, random_rows, read_u64, SegScratch, TVector, LINK, PF_L1, START, TILE};

/// A task for a pool, called once on each of its k threads with the thread's index (the
/// caller's is 0)
pub(crate) type Task<'a> = &'a (dyn Fn(usize) + Sync);

/// The epoch that tells the helpers to leave
const SHUTDOWN: usize = usize::MAX;

/// Spins before a wait parks: a pool's next run is usually microseconds away, a block's
/// MTF stage (about 0.4 ms) apart. Miri interprets each spin, so it parks almost at once.
const SPINS: u32 = if cfg!(miri) { 4 } else { 2_000 };

/// The waits' accounting (feature `wait-stats`, off by default; research only): per wait
/// site, the waits, the spins, the parks and the yields, as counts, never times, for
/// `examples/wait_audit.rs`. Without the feature a `Tally` is empty and every call on it
/// compiles to nothing, so the decoder is unchanged.
pub mod wait_stats {
    /// A wait site in this module
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Site {
        /// a pool's helper, between runs, waiting for the next (`helper`)
        HelperIdle = 0,
        /// a pool's caller waiting for its helpers to end the run (`Wait`)
        CallerJoin = 1,
        /// a pipelined team's helper waiting for the build's end (`wait_walk_ready`)
        WalkHelper = 2,
        /// a pipelined team's leader waiting for the build's end (`Relay::join`)
        WalkLeader = 3,
    }

    /// The sites, in their index order
    pub const SITES: [Site; 4] = [
        Site::HelperIdle,
        Site::CallerJoin,
        Site::WalkHelper,
        Site::WalkLeader,
    ];

    impl Site {
        pub fn name(self) -> &'static str {
            match self {
                Site::HelperIdle => "pool_helper_idle",
                Site::CallerJoin => "pool_caller_join",
                Site::WalkHelper => "walk_ready_helper",
                Site::WalkLeader => "walk_ready_leader",
            }
        }
    }

    /// One site's counts since the last `reset`
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Counts {
        /// the waits entered
        pub waits: u64,
        /// those whose condition held at the first check (no spin, no park)
        pub immediate: u64,
        /// those that ended inside the spin budget, after at least one spin
        pub ended_spinning: u64,
        /// those that parked at least once
        pub parked: u64,
        /// those that yielded at least once
        pub yielded: u64,
        /// the `spin_loop` iterations, all waits
        pub spins: u64,
        /// the spins of the waits that went on to park or yield (budget spent for nothing)
        pub spins_wasted: u64,
        /// the `park` calls (a parked wait can park more than once)
        pub parks: u64,
        /// the `yield_now` calls
        pub yields: u64,
    }

    /// One wait's tally, kept on the waiter's stack and added to its site's counts once,
    /// when the wait ends (`end`)
    #[cfg(feature = "wait-stats")]
    #[derive(Default)]
    pub(crate) struct Tally {
        spins: u64,
        parks: u64,
        yields: u64,
    }

    #[cfg(feature = "wait-stats")]
    mod imp {
        use super::{Counts, Site, Tally};
        use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

        const FIELDS: usize = 9;
        #[allow(clippy::declare_interior_mutable_const)]
        const Z: AtomicU64 = AtomicU64::new(0);
        #[allow(clippy::declare_interior_mutable_const)]
        const ROW: [AtomicU64; FIELDS] = [Z; FIELDS];
        static COUNTS: [[AtomicU64; FIELDS]; 4] = [ROW; 4];

        impl Tally {
            #[inline]
            pub(crate) fn new() -> Tally {
                Tally::default()
            }
            #[inline]
            pub(crate) fn spin(&mut self) {
                self.spins += 1;
            }
            #[inline]
            pub(crate) fn park(&mut self) {
                self.parks += 1;
            }
            #[inline]
            pub(crate) fn yielded(&mut self) {
                self.yields += 1;
            }
            pub(crate) fn end(self, site: Site) {
                let c = &COUNTS[site as usize];
                let slept = self.parks > 0 || self.yields > 0;
                let add = [
                    1,
                    (self.spins == 0 && !slept) as u64,
                    (self.spins > 0 && !slept) as u64,
                    (self.parks > 0) as u64,
                    (self.yields > 0) as u64,
                    self.spins,
                    if slept { self.spins } else { 0 },
                    self.parks,
                    self.yields,
                ];
                for (a, v) in c.iter().zip(add) {
                    if v != 0 {
                        a.fetch_add(v, Relaxed);
                    }
                }
            }
        }

        pub(super) fn get(site: Site) -> Counts {
            let c = &COUNTS[site as usize];
            let v = |i: usize| c[i].load(Relaxed);
            Counts {
                waits: v(0),
                immediate: v(1),
                ended_spinning: v(2),
                parked: v(3),
                yielded: v(4),
                spins: v(5),
                spins_wasted: v(6),
                parks: v(7),
                yields: v(8),
            }
        }

        pub(super) fn reset() {
            for c in &COUNTS {
                for a in c {
                    a.store(0, Relaxed);
                }
            }
        }
    }

    /// Without the feature: nothing kept, nothing counted
    #[cfg(not(feature = "wait-stats"))]
    pub(crate) struct Tally;

    #[cfg(not(feature = "wait-stats"))]
    impl Tally {
        #[inline(always)]
        pub(crate) fn new() -> Tally {
            Tally
        }
        #[inline(always)]
        pub(crate) fn spin(&mut self) {}
        #[inline(always)]
        pub(crate) fn park(&mut self) {}
        #[inline(always)]
        pub(crate) fn yielded(&mut self) {}
        #[inline(always)]
        pub(crate) fn end(self, _site: Site) {}
    }

    /// Whether the counts are kept (the feature `wait-stats`)
    pub const ENABLED: bool = cfg!(feature = "wait-stats");

    /// A site's counts since the last `reset` (all zero without the feature)
    pub fn get(site: Site) -> Counts {
        #[cfg(feature = "wait-stats")]
        return imp::get(site);
        #[cfg(not(feature = "wait-stats"))]
        {
            let _ = site;
            Counts::default()
        }
    }

    /// Every site's counts back to zero
    pub fn reset() {
        #[cfg(feature = "wait-stats")]
        imp::reset();
    }
}
use wait_stats::{Site, Tally};

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

/// A plain team's helper threads' names: this and a number (`mibz2-coop-1`)
pub(crate) const HELPER: &str = "mibz2-coop-";
/// A pipelined team's helper threads' names: this and a number (`mibz2-coop-p1`)
pub(crate) const PIPELINED_HELPER: &str = "mibz2-coop-p";

impl Pool {
    /// A pool of `k` threads (`k` − 1 spawned; `k` of 0 is taken as 1). A helper that
    /// can't be spawned makes this an error, with those spawned before it shut down.
    pub(crate) fn new(k: usize) -> std::io::Result<Pool> {
        Self::named(k, HELPER)
    }

    /// `new`, for a pipelined team (`parallel::StreamDecoder::pipelined`): its helpers are
    /// named `PIPELINED_HELPER` and a number, so an observer of the threads' names (`ab`'s
    /// census and `thread_cpu`) can tell a pipelined team from a plain one. Both names start
    /// `mibz2-coop`, so whatever counts helpers by that prefix counts both.
    pub(crate) fn pipelined(k: usize) -> std::io::Result<Pool> {
        Self::named(k, PIPELINED_HELPER)
    }

    fn named(k: usize, prefix: &str) -> std::io::Result<Pool> {
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
                .name(format!("{prefix}{idx}"))
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
        let mut tally = Tally::new();
        while self.s.done.load(Ordering::Acquire) != self.helpers {
            spins += 1;
            if spins < SPINS {
                tally.spin();
                std::hint::spin_loop();
            } else {
                tally.park();
                thread::park();
            }
        }
        tally.end(Site::CallerJoin);
    }
}

fn helper(s: &Shared, idx: usize) {
    let mut seen = 0;
    loop {
        let mut spins = 0u32;
        let mut tally = Tally::new();
        let e = loop {
            let e = s.epoch.load(Ordering::Acquire);
            if e != seen {
                break e;
            }
            spins += 1;
            if spins < SPINS {
                tally.spin();
                std::hint::spin_loop();
            } else {
                tally.park();
                thread::park();
            }
        };
        tally.end(Site::HelperIdle);
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
    let b =
        FusedBuild::new(ll, counts, checkpoints, stride, pool.threads(), tt).map_err(|(e, _)| e)?;
    pool.run(&|j| b.share(j));
    b.finish().0
}

/// The most shares a `FusedBuild` is cut into (a bit each in its masks)
pub(crate) const MAX_SHARES: usize = 64;

/// `build_fused`'s checks before the scatter (its doc): O(256) a checkpoint
fn check_inputs(
    n: usize,
    counts: &[u32; 256],
    checkpoints: &[[u32; 256]],
    stride: usize,
    shares: usize,
) -> Result<(), BuildError> {
    if shares == 0 || shares > MAX_SHARES {
        return Err(BuildError("no shares, or more than MAX_SHARES"));
    }
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
    Ok(())
}

/// `build_fused` cut into its parts, for a caller that runs the shares on threads of its
/// own choosing (the pipelined team's helpers, `invert_pipelined`): `new` makes the checks
/// and reserves the table, `share(j)` is range j's scatter (on any thread, each j once),
/// and `finish` makes the end check and sets the table's length. `build_fused` is `new`,
/// a share on each of the pool's threads, then `finish`.
///
/// It holds the table for its life, so nothing else touches it between the parts. Each
/// share claims its index before it writes, so a share run twice writes nothing the
/// second time, and `finish` sets the table's length only if every share passed its end
/// check: the table is never left uninitialised, whichever shares ran.
pub(crate) struct FusedBuild<'a> {
    ll: &'a [u8],
    counts: &'a [u32; 256],
    checkpoints: &'a [[u32; 256]],
    stride: usize,
    shares: usize,
    starts: [u32; 256],
    tt: &'a mut TVector,
    slots: Slots,
    /// the shares that began (each claims its bit before it writes), and those that
    /// passed their end check
    claimed: AtomicU64,
    passed: AtomicU64,
}

impl<'a> FusedBuild<'a> {
    /// `build_fused`'s checks before the scatter, the table cleared and reserved, the rows
    /// cut into `shares` ranges (1 to `MAX_SHARES`). A refusal gives the table back, empty.
    pub(crate) fn new(
        ll: &'a [u8],
        counts: &'a [u32; 256],
        checkpoints: &'a [[u32; 256]],
        stride: usize,
        shares: usize,
        tt: &'a mut TVector,
    ) -> Result<FusedBuild<'a>, (BuildError, &'a mut TVector)> {
        let n = ll.len();
        tt.0.clear();
        if let Err(e) = check_inputs(n, counts, checkpoints, stride, shares) {
            return Err((e, tt));
        }
        let mut starts = [0u32; 256];
        let mut cum = 0u32;
        for b in 0..256 {
            starts[b] = cum;
            cum += counts[b];
        }
        tt.0.reserve(n);
        let slots = Slots(tt.0.as_mut_ptr());
        Ok(FusedBuild {
            ll,
            counts,
            checkpoints,
            stride,
            shares,
            starts,
            tt,
            slots,
            claimed: AtomicU64::new(0),
            passed: AtomicU64::new(0),
        })
    }

    /// Range j's scatter (j below the shares), on any thread; a j already claimed, or out
    /// of range, writes nothing
    pub(crate) fn share(&self, j: usize) {
        if j >= self.shares {
            return;
        }
        let bit = 1u64 << j;
        if self.claimed.fetch_or(bit, Ordering::Relaxed) & bit != 0 {
            return;
        }
        let (ll, counts, checkpoints, stride) =
            (self.ll, self.counts, self.checkpoints, self.stride);
        let (n, k, starts, slots) = (ll.len(), self.shares, &self.starts, &self.slots);
        // the counts before row e (a range's edge: a multiple of stride, or n)
        let at = |e: usize| {
            if e == n {
                counts
            } else {
                &checkpoints[e / stride]
            }
        };
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
            // (checked in `new`) and which no other range's window overlaps; range j is
            // written only by the share that claimed it, and the table's buffer is held
            // (`self.tt`), not moved, until `finish`
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
        // the end check: every window filled exactly (the share's writes released with it)
        if ok && ftab == lim {
            self.passed.fetch_or(bit, Ordering::Release);
        }
    }

    /// The end check over every share, the table's length set only if all passed, and the
    /// table given back. The shares' writes must happen before this call (the shares run
    /// on this thread, or in a pool's run that has returned, or on threads that released
    /// once their shares ended, as this thread acquired); `passed`'s acquire adds its own.
    pub(crate) fn finish(self) -> (Result<(), BuildError>, &'a mut TVector) {
        let all = if self.shares == MAX_SHARES {
            u64::MAX
        } else {
            (1u64 << self.shares) - 1
        };
        let n = self.ll.len();
        let tt = self.tt;
        if self.passed.load(Ordering::Acquire) != all {
            return (Err(BuildError("a range's bytes didn't fill its windows: the checkpoints or counts are not the block's")), tt);
        }
        // SAFETY: every slot below n was written exactly once (the end check, by every range,
        // whose windows tile [0, n)), and every share's writes are visible here
        unsafe { tt.0.set_len(n) };
        (Ok(()), tt)
    }
}

/// The shared walk's handoffs, counted in the tests (each asserts it reached them)
#[cfg(test)]
pub(crate) mod handoffs {
    use std::sync::atomic::AtomicUsize;
    pub(crate) static GROWS: AtomicUsize = AtomicUsize::new(0);
    pub(crate) static PASS_ENDS_FULL: AtomicUsize = AtomicUsize::new(0);
    pub(crate) static MID_PASS_TAKES: AtomicUsize = AtomicUsize::new(0);
    pub(crate) static FULL_PHASE_ENDS_MID_PASS: AtomicUsize = AtomicUsize::new(0);
    pub(crate) static TAIL_ENTERED_FULL: AtomicUsize = AtomicUsize::new(0);
}
#[cfg(test)]
macro_rules! handoff {
    ($c:ident) => {
        handoffs::$c.fetch_add(1, Ordering::Relaxed)
    };
}
#[cfg(not(test))]
macro_rules! handoff {
    ($c:ident) => {};
}

/// The staged walk's transitions (`traverse_staged`), counted per thread in the tests, so
/// that a test running its walkers on its own thread counts only its own
#[cfg(test)]
pub(crate) mod staged_seen {
    use std::cell::Cell;
    pub(crate) const GROWS: usize = 0;
    pub(crate) const PASS_ENDS_FULL: usize = 1;
    pub(crate) const ROOM_OVER_TILE: usize = 2;
    pub(crate) const MID_PASS_TAKES: usize = 3;
    pub(crate) const MARKED_SEVERAL: usize = 4;
    pub(crate) const DONE_IN_TAKING_CHUNK: usize = 5;
    pub(crate) const TAKE_AT_CHUNK_END: usize = 6;
    pub(crate) const TAKE_IN_LATER_CHUNK: usize = 7;
    pub(crate) const MARK_AT_LATER_CHUNK_START: usize = 8;
    pub(crate) const EXHAUSTED_MID_PASS: usize = 9;
    pub(crate) const EXHAUSTED_AT_CHUNK_START: usize = 10;
    pub(crate) const EXHAUSTED_AT_CHUNK_END: usize = 11;
    pub(crate) const EXHAUSTED_AT_PASS_END: usize = 12;
    pub(crate) const ENDS_WITH_LANES_WAITING: usize = 13;
    pub(crate) const ENDS_AFTER_A_TAKE: usize = 14;
    pub(crate) const LATER_STEPPED: usize = 15;
    pub(crate) const LATER_STEPPED_ON_START: usize = 16;
    pub(crate) const LATER_STEPPED_FULL: usize = 17;
    pub(crate) const TAIL_ENTERED_FULL: usize = 18;
    pub(crate) const NEVER_FULL: usize = 19;
    pub(crate) const NO_SEGMENT: usize = 20;
    pub(crate) const N: usize = 21;
    pub(crate) const NAMES: [&str; N] = [
        "GROWS",
        "PASS_ENDS_FULL",
        "ROOM_OVER_TILE",
        "MID_PASS_TAKES",
        "MARKED_SEVERAL",
        "DONE_IN_TAKING_CHUNK",
        "TAKE_AT_CHUNK_END",
        "TAKE_IN_LATER_CHUNK",
        "MARK_AT_LATER_CHUNK_START",
        "EXHAUSTED_MID_PASS",
        "EXHAUSTED_AT_CHUNK_START",
        "EXHAUSTED_AT_CHUNK_END",
        "EXHAUSTED_AT_PASS_END",
        "ENDS_WITH_LANES_WAITING",
        "ENDS_AFTER_A_TAKE",
        "LATER_STEPPED",
        "LATER_STEPPED_ON_START",
        "LATER_STEPPED_FULL",
        "TAIL_ENTERED_FULL",
        "NEVER_FULL",
        "NO_SEGMENT",
    ];
    thread_local! {
        static SEEN: [Cell<usize>; N] = Default::default();
    }
    pub(crate) fn hit(i: usize) {
        SEEN.with(|s| s[i].set(s[i].get() + 1));
    }
    /// This thread's counts since the last `take`
    pub(crate) fn take() -> [usize; N] {
        SEEN.with(|s| std::array::from_fn(|i| s[i].replace(0)))
    }
}
macro_rules! seen {
    ($c:ident) => {
        #[cfg(test)]
        staged_seen::hit(staged_seen::$c)
    };
}

/// What a walk counts as it goes (`traverse`, `traverse_staged`): every method a no-op
/// for `()`, which the decoder passes unless the `walk-stats` feature is on and a caller
/// asks for counts, so that without them the walks' machine code is as before
pub(crate) trait Stat {
    /// a claim of the shared counter: a segment taken, or the queue found exhausted
    #[inline(always)]
    fn claimed(&self, _took: bool) {}
    /// a segment finished, `len` bytes long (its first byte, the take's, included)
    #[inline(always)]
    fn seg(&self, _len: usize) {}
    /// a byte written by the unstaged tail
    #[inline(always)]
    fn tail(&self) {}
    /// a staged fragment copied, `len` bytes (empty ones included)
    #[inline(always)]
    fn fragment(&self, _len: usize) {}
}

impl Stat for () {}

/// The walks' statistics, kept only with the `walk-stats` feature (research: the team
/// screen's T2 probes and the native engagement diagnostic,
/// docs/journal/2026-11-10-staged-engagement-diagnostic-draft.md). Each record is one JSON
/// line, pushed whole under one lock by the thread that did the work, once a call (never
/// in a walk's loop); the harness takes them all at its end (`take`).
#[cfg(feature = "walk-stats")]
pub mod walk_stats {
    use std::cell::Cell;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    static RECORDS: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static TEAMS: AtomicUsize = AtomicUsize::new(0);
    thread_local! {
        static TEAM: Cell<Option<usize>> = const { Cell::new(None) };
    }

    /// One walker call's counts (`Stat`), on the calling thread's stack
    #[derive(Default)]
    pub(crate) struct Counts {
        pub(crate) takes: Cell<u64>,
        pub(crate) exhausts: Cell<u64>,
        pub(crate) bytes: Cell<u64>,
        pub(crate) tail: Cell<u64>,
        pub(crate) fragments: Cell<u64>,
        pub(crate) copied: Cell<u64>,
    }

    impl super::Stat for Counts {
        #[inline(always)]
        fn claimed(&self, took: bool) {
            let c = if took { &self.takes } else { &self.exhausts };
            c.set(c.get() + 1);
        }
        #[inline(always)]
        fn seg(&self, len: usize) {
            self.bytes.set(self.bytes.get() + len as u64);
        }
        #[inline(always)]
        fn tail(&self) {
            self.tail.set(self.tail.get() + 1);
        }
        #[inline(always)]
        fn fragment(&self, len: usize) {
            if len > 0 {
                self.fragments.set(self.fragments.get() + 1);
                self.copied.set(self.copied.get() + len as u64);
            }
        }
    }

    /// This thread's team: a pipelined team's leader takes a new index the first time it
    /// asks (`pipelined_worker`'s thread), and passes it to the run's helpers in `Relay`
    pub(crate) fn team() -> usize {
        TEAM.with(|t| {
            t.get().unwrap_or_else(|| {
                let id = TEAMS.fetch_add(1, Ordering::Relaxed);
                t.set(Some(id));
                id
            })
        })
    }

    /// A pool index's bit in a summary's masks (a team has at most `MAX_SHARES` + 1 = 65
    /// threads; an index of 64 or more has no bit, and the checker finds it missing)
    pub(crate) fn bit(who: usize) -> u64 {
        1u64.checked_shl(who as u32).unwrap_or(0)
    }

    pub(crate) fn push(line: String) {
        RECORDS.lock().unwrap_or_else(|e| e.into_inner()).push(line);
    }

    /// Every record so far, taken, then a terminal line with their count
    pub fn take() -> Vec<String> {
        let mut v = std::mem::take(&mut *RECORDS.lock().unwrap_or_else(|e| e.into_inner()));
        v.push(format!("{{\"kind\":\"end\",\"records\":{}}}", v.len()));
        v
    }

    /// The records of `team` so far (the tests', which share the process with others)
    #[cfg(test)]
    pub(crate) fn of_team(team: usize) -> Vec<String> {
        let tag = format!("\"team\":{team},");
        RECORDS
            .lock()
            .unwrap()
            .iter()
            .filter(|l| l.contains(&tag))
            .cloned()
            .collect()
    }

    /// A walker call's record
    pub(crate) fn call(team: usize, block: u64, thread: usize, walker: &str, c: &Counts) {
        push(format!(
            "{{\"kind\":\"call\",\"team\":{team},\"block\":{block},\"thread\":{thread},\"role\":\"{}\",\"walker\":\"{}\",\"takes\":{},\"exhausts\":{},\"bytes\":{},\"tail\":{},\"fragments\":{},\"copied\":{}}}",
            if thread == 0 { "leader" } else { "helper" },
            walker,
            c.takes.get(),
            c.exhausts.get(),
            c.bytes.get(),
            c.tail.get(),
            c.fragments.get(),
            c.copied.get(),
        ));
    }
}

/// One lane's state in the shared walk: the row it reads next, and where its segment's
/// bytes go (`p` up to `end`, the segment's buffer's spare capacity)
#[derive(Clone, Copy)]
struct Lane {
    row: u32,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

/// The segments' buffers and successors, for the pool's threads to reach through: each
/// segment's are touched only by the lane that took it
#[derive(Clone, Copy)]
struct Segs {
    bufs: *mut Vec<u8>,
    next: *mut usize,
}
// SAFETY: a segment index is taken once (`taken.fetch_add`), so each buffer and successor
// slot has one writer; the outer vectors are not resized while the threads run
unsafe impl Sync for Segs {}

/// The room walk on k threads sharing one T-vector: `bwt::walk_segments_room`'s walk (its
/// `room_segments`), every thread's L lanes taking segments from one counter, so a block's
/// `segments` (k times the one-core walk's, so the tail's few long chains don't leave the
/// team waiting) are walked by k·L lanes at once. Each segment's bytes go to its buffer
/// in `s` and its successor to `s.next`, as the one-core walk leaves them, for
/// `SegScratch::assemble` to join: exactly `bwt::walk`'s output.
///
/// The one-core walk's contract (its Rows, Ownership, Spare, Handoffs, Initialised and
/// Lifetimes; `bwt::walk_segments_room_with_prefetch`) holds for each thread's lanes, with:
/// - **Sharing:** the table is read-only while the threads run. The starts are flagged
///   before the run and unflagged after it, on the caller.
/// - **Taking:** each segment index is taken once, by `fetch_add`, so its buffer and its
///   successor slot have one writer, the lane that took it.
/// - **Outer vectors:** `s.bufs` and `s.next` are sized before the run and not resized in
///   it; a lane grows only the buffer of the segment it holds.
/// - **Visibility:** the pool's run publishes every thread's writes to the caller before
///   `assemble` reads them (`Pool::run`'s ordering).
pub(crate) fn walk_room<const L: usize>(
    tt: &mut TVector,
    orig_ptr: u32,
    s: &mut SegScratch,
    pool: &Pool,
    segments: usize,
    staged: bool,
) {
    let w = RoomWalk::new(tt, orig_ptr, s, segments, staged);
    pool.run(&|_| w.share::<L>());
    w.finish();
}

/// `walk_room` cut into its parts, for a caller that runs the walk on threads of its own
/// choosing (the pipelined team, `invert_pipelined`): `new` picks and flags the starts and
/// sizes the buffers, `share` is one thread's lanes (on any number of threads, each
/// taking segments from the one counter until none is left), and `finish` unflags the
/// starts and records where the output starts, for `SegScratch::assemble`. It holds the
/// table and the scratch for its life, so the contract's Sharing and Outer vectors hold
/// between the parts.
///
/// The walk is whole only once every segment taken has been walked to its end: every
/// `share` returned, none unwound. `finish` must be called after every `share` has
/// returned, with their writes visible to it (as `FusedBuild::finish`).
pub(crate) struct RoomWalk<'a> {
    tt: &'a mut TVector,
    s: &'a mut SegScratch,
    r0: u32,
    t: Slots,
    starts: (*const u32, usize),
    segs: Segs,
    taken: AtomicUsize,
    cap: usize,
    /// each share's walk: the staged one (`traverse_staged`, research) or not (`traverse`)
    staged: bool,
}
// SAFETY: the raw pointers are into the table and the scratch it holds: the table and the
// starts are read-only while shares run, and each segment's buffer and successor slot has
// one writer, the lane that took it (`walk_room`'s contract)
unsafe impl Sync for RoomWalk<'_> {}

impl<'a> RoomWalk<'a> {
    pub(crate) fn new(
        tt: &'a mut TVector,
        orig_ptr: u32,
        s: &'a mut SegScratch,
        segments: usize,
        staged: bool,
    ) -> RoomWalk<'a> {
        let n = tt.0.len();
        let r0 = tt.0[orig_ptr as usize] >> 8;
        random_rows(n, segments.max(1) - 1, &mut s.starts);
        s.starts.push(r0);
        s.starts.sort_unstable();
        s.starts.dedup();
        let ns = s.starts.len();
        for &r in &s.starts {
            tt.0[r as usize] |= START;
        }
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let segs = Segs {
            bufs: s.bufs.as_mut_ptr(),
            next: s.next.as_mut_ptr(),
        };
        let starts = (s.starts.as_ptr(), ns);
        let t = Slots(tt.0.as_mut_ptr());
        RoomWalk {
            tt,
            s,
            r0,
            t,
            starts,
            segs,
            taken: AtomicUsize::new(0),
            cap,
            staged,
        }
    }

    /// One thread's L lanes, taking segments until none is left
    pub(crate) fn share<const L: usize>(&self) {
        self.share_with::<L, ()>(&())
    }

    /// `share`, its walk counted into `st` (`walk_stats::Counts` with the `walk-stats`
    /// feature; `()`, counting nothing, otherwise)
    pub(crate) fn share_with<const L: usize, S: Stat>(&self, st: &S) {
        // SAFETY: the starts are the scratch's, held and not resized until `finish`
        let starts = unsafe { std::slice::from_raw_parts(self.starts.0, self.starts.1) };
        // SAFETY: as `walk_room`'s contract: rows below n, starts flagged, each segment
        // taken once, every buffer with `cap` spare bytes
        unsafe {
            if self.staged {
                let claim = Counter {
                    taken: &self.taken,
                    ns: starts.len(),
                };
                traverse_staged::<L, 0, _, S>(
                    self.t.0 as *const u32,
                    starts,
                    self.segs,
                    &claim,
                    self.cap,
                    st,
                )
            } else {
                traverse::<L, S>(
                    self.t.0 as *const u32,
                    starts,
                    self.segs,
                    &self.taken,
                    self.cap,
                    st,
                )
            }
        }
    }

    /// One walker of the transition tests: the staged walk with mutation `MUT`, its claims
    /// from the shared counter until `budget` of them have been made, after which it finds
    /// the queue exhausted (as if another walker had taken the rest); `None` for no budget
    #[cfg(test)]
    pub(crate) fn share_staged_budget<const L: usize, const MUT: u8>(&self, budget: Option<usize>) {
        let starts = unsafe { std::slice::from_raw_parts(self.starts.0, self.starts.1) };
        let claim = tests_claim::Budget {
            inner: Counter {
                taken: &self.taken,
                ns: starts.len(),
            },
            left: std::cell::Cell::new(budget.unwrap_or(usize::MAX)),
        };
        // SAFETY: as `share`
        unsafe {
            traverse_staged::<L, MUT, _, ()>(
                self.t.0 as *const u32,
                starts,
                self.segs,
                &claim,
                self.cap,
                &(),
            )
        }
    }

    /// The unstaged walk as one walker of the transition tests (the rest of a queue a staged
    /// walker left)
    #[cfg(test)]
    pub(crate) fn share_unstaged<const L: usize>(&self) {
        let starts = unsafe { std::slice::from_raw_parts(self.starts.0, self.starts.1) };
        // SAFETY: as `share`
        unsafe {
            traverse::<L, ()>(
                self.t.0 as *const u32,
                starts,
                self.segs,
                &self.taken,
                self.cap,
                &(),
            )
        }
    }

    /// The starts unflagged and the output's first segment recorded; the table and the
    /// scratch given back
    pub(crate) fn finish(self) -> (&'a mut TVector, &'a mut SegScratch) {
        let (tt, s) = (self.tt, self.s);
        for &r in &s.starts {
            tt.0[r as usize] &= LINK;
        }
        s.first = s.starts.binary_search(&self.r0).unwrap_or(usize::MAX);
        s.n = tt.0.len();
        (tt, s)
    }
}

/// One thread's lanes of `walk_room`: `room_segments`'s loop, its next segment taken from
/// `taken` instead of a local count
///
/// SAFETY: as `walk_room`'s contract
// (never inlined: with `share`'s choice of walker beside it, it would be inlined there,
// changing the default walk's machine code; kept a function of its own, as it was)
#[inline(never)]
unsafe fn traverse<const L: usize, S: Stat>(
    t: *const u32,
    starts: &[u32],
    segs: Segs,
    taken: &AtomicUsize,
    cap: usize,
    st: &S,
) {
    let ns = starts.len();
    let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let take = |seg: usize| -> Lane {
        // SAFETY: seg < ns, taken once, so this lane is its buffer's one user
        let b = unsafe { &mut *segs.bufs.add(seg) };
        let base = b.as_mut_ptr();
        // SAFETY: the starts are rows below n; the buffer has `cap` > 0 spare bytes
        let q = unsafe { *t.add(starts[seg] as usize) } & LINK;
        unsafe { *base = q as u8 };
        prefetch::<PF_L1>(t, q >> 8);
        Lane {
            row: q >> 8,
            seg,
            p: unsafe { base.add(1) },
            end: unsafe { base.add(b.capacity()) },
        }
    };
    let finish = |l: &Lane| {
        // SAFETY: the lane holds segment l.seg, its buffer's one user and its successor's
        // one writer; p is within the buffer, every byte below it written
        let b = unsafe { &mut *segs.bufs.add(l.seg) };
        let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
        unsafe { b.set_len(len) };
        unsafe { *segs.next.add(l.seg) = seg_of(l.row) };
        st.seg(len);
    };
    let grow = |l: &mut Lane| {
        handoff!(GROWS);
        // SAFETY: as `finish`
        let b = unsafe { &mut *segs.bufs.add(l.seg) };
        let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
        unsafe { b.set_len(len) };
        b.reserve(b.capacity());
        l.p = unsafe { b.as_mut_ptr().add(len) };
        l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
    };
    let next_seg = || {
        let s = taken.fetch_add(1, Ordering::Relaxed);
        st.claimed(s < ns);
        (s < ns).then_some(s)
    };

    let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
        row: 0,
        seg: usize::MAX,
        p: std::ptr::null_mut(),
        end: std::ptr::null_mut(),
    });
    let mut active = 0;
    while active < L {
        let Some(seg) = next_seg() else { break };
        lanes[active] = take(seg);
        active += 1;
    }
    if active == L {
        'full: loop {
            // the pass's room: what every lane has spare, and no more than a buffer taken
            // mid-pass has after its first byte (`cap` - 1)
            let mut room = cap - 1;
            for l in lanes.iter_mut() {
                if l.p == l.end {
                    grow(l);
                }
                room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
            }
            debug_assert!(room > 0);
            if lanes
                .iter()
                .any(|l| unsafe { l.end.offset_from(l.p) } as usize == room)
            {
                handoff!(PASS_ENDS_FULL);
            }
            let mut base: [*mut u8; L] = std::array::from_fn(|k| lanes[k].p);
            for off in 0..room {
                for k in 0..L {
                    let l = &mut lanes[k];
                    // SAFETY: rows are below n (TVector), with the flag masked off below
                    let q = unsafe { *t.add(l.row as usize) };
                    if q & START != 0 {
                        l.p = base[k].wrapping_add(off);
                        finish(l);
                        let Some(seg) = next_seg() else {
                            // the lanes before this one wrote at `off`, those after it not yet
                            for (j, lj) in lanes.iter_mut().enumerate() {
                                if j < k {
                                    lj.p = base[j].wrapping_add(off + 1);
                                } else if j > k {
                                    lj.p = base[j].wrapping_add(off);
                                }
                            }
                            handoff!(FULL_PHASE_ENDS_MID_PASS);
                            break 'full;
                        };
                        handoff!(MID_PASS_TAKES);
                        *l = take(seg);
                        // its next byte, at offset off + 1, goes at its p
                        base[k] = l.p.wrapping_sub(off + 1);
                        continue;
                    }
                    l.row = q >> 8;
                    prefetch::<PF_L1>(t, l.row);
                    // SAFETY: base + off is below the lane's end: room is at most its spare
                    // at the pass's start (or, for a buffer taken mid-pass, its capacity
                    // after its first byte)
                    debug_assert!(base[k].wrapping_add(off) < l.end);
                    unsafe { *base[k].wrapping_add(off) = q as u8 };
                }
            }
            for (l, b) in lanes.iter_mut().zip(base) {
                l.p = b.wrapping_add(room);
            }
        }
    }
    // the lanes still walking: a finished lane's successor is set, and this thread holds
    // every segment its lanes took, so it alone reads those slots
    let unfinished =
        |l: &Lane| l.seg != usize::MAX && unsafe { *segs.next.add(l.seg) } == usize::MAX;
    let mut live: Vec<Lane> = lanes[..active].iter().copied().filter(unfinished).collect();
    // a pass can end with a lane's buffer exactly full (its room was all its spare): the
    // tail writes before it tests, so each lane comes into it with a byte to spare
    for l in live.iter_mut() {
        if l.p == l.end {
            handoff!(TAIL_ENTERED_FULL);
            grow(l);
        }
        debug_assert!(l.p < l.end);
    }
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            let q = unsafe { *t.add(l.row as usize) };
            if q & START != 0 {
                finish(l);
                live.swap_remove(i);
                continue;
            }
            l.row = q >> 8;
            prefetch::<PF_L1>(t, l.row);
            unsafe {
                *l.p = q as u8;
                l.p = l.p.add(1);
            }
            st.tail();
            if l.p == l.end {
                grow(l);
            }
            i += 1;
        }
    }
}

/// The research selector of the cooperative walk (`set_staged_traverse`), and the walk's
/// one-caller form for counting it (`walk_one_caller`). Off by default: the decoder's teams
/// then walk unstaged (`traverse`), exactly as before. Research only: no part of the API.
#[doc(hidden)]
pub mod research {
    use super::*;

    static STAGED: AtomicBool = AtomicBool::new(false);

    /// The teams' walks from now on staged (`traverse_staged`) or not (`traverse`, the
    /// default). Read when a walk is set up (`RoomWalk::new`'s callers), so a block whose
    /// walk is already set up keeps the walk it has; the output is the same either way.
    pub fn set_staged_traverse(on: bool) {
        STAGED.store(on, Ordering::Relaxed);
    }

    /// Whether the teams' walks are staged
    #[inline]
    pub fn staged_traverse() -> bool {
        STAGED.load(Ordering::Relaxed)
    }

    /// The cooperative walk of a built table on the calling thread alone, as one share of a
    /// team's (`RoomWalk::new` with `segments`, one `share`, `finish`), staged or not: the
    /// same-segmentation control of the staged walk's count. Its segments are left in `s`
    /// for `SegScratch::assemble`, as a team's are.
    #[inline(never)]
    pub fn walk_one_caller(
        tt: &mut TVector,
        orig_ptr: u32,
        s: &mut SegScratch,
        segments: usize,
        staged: bool,
    ) {
        let w = RoomWalk::new(tt, orig_ptr, s, segments, staged);
        w.share::<{ crate::bwt::FLAGGED_LANES }>();
        w.finish();
    }
}

/// Where a staged walker's next segment comes from: the shared counter (`Counter`) in the
/// decoder; in the transition tests, a counter that finds the queue exhausted early
trait Claim {
    fn claim(&self) -> Option<usize>;
}

/// The team's one segment counter: each index below `ns` taken once (`fetch_add`)
struct Counter<'a> {
    taken: &'a AtomicUsize,
    ns: usize,
}

impl Claim for Counter<'_> {
    #[inline(always)]
    fn claim(&self) -> Option<usize> {
        let s = self.taken.fetch_add(1, Ordering::Relaxed);
        (s < self.ns).then_some(s)
    }
}

#[cfg(test)]
mod tests_claim {
    /// A walker that, after `left` claims, finds the queue exhausted: another walker took
    /// the rest of it (the test runs that one next)
    pub(crate) struct Budget<'a> {
        pub(crate) inner: super::Counter<'a>,
        pub(crate) left: std::cell::Cell<usize>,
    }
    impl super::Claim for Budget<'_> {
        fn claim(&self) -> Option<usize> {
            let l = self.left.get();
            if l == 0 {
                return None;
            }
            self.left.set(l - 1);
            self.inner.claim()
        }
    }
}

/// The transition tests' deliberately broken staged walks (`traverse_staged`'s `MUT`, 0 in
/// the decoder): each a construction error the tests must catch
#[allow(dead_code)]
pub(crate) mod mutation {
    /// On exhaustion, the unstaged walk's rule: the lanes after the exhausting lane "have
    /// not stepped", so are flushed through j, not j + 1
    pub(crate) const OLD_RULE: u8 = 1;
    /// After a take, the new segment's staged start at j: the tile's stale byte at j copied
    pub(crate) const ST_AT_J: u8 = 2;
    /// A segment's length published before its last fragment is flushed
    pub(crate) const FINISH_BEFORE_FLUSH: u8 = 3;
    /// On exhaustion, a still-marked (waiting) lane flushed through j + 1, as if it stepped
    pub(crate) const WAITING_AS_STEPPED: u8 = 4;
    /// On exhaustion, the rows not copied back into the lanes before the tail
    pub(crate) const NO_ROWS_BACK: u8 = 5;
    /// On exhaustion, the other lanes' staged bytes not flushed (the share returns with
    /// owned data uncommitted)
    pub(crate) const NO_EXHAUST_FLUSH: u8 = 6;
    /// A lane entering the tail exactly full not grown first (a stale end pointer)
    pub(crate) const NO_TAIL_GROW: u8 = 7;
    pub(crate) const ALL: [u8; 7] = [1, 2, 3, 4, 5, 6, 7];
}

/// One thread's lanes of `walk_room`, staged as the one-core walk's busy phase is
/// (`bwt::room_segments`, #171), its next segment taken from `claim` (the team's one
/// counter) instead of a local count. Research, behind `research::set_staged_traverse`.
///
/// While all L lanes are busy, the rows are held together in `rows`, and offset j of a
/// chunk of at most `TILE` offsets puts lane k's byte at `tile[k][j]`; each lane's bytes
/// from `st[k]` are copied to its segment's buffer when the chunk ends or the segment does.
/// The tile, rows and staged starts are this walker's own: nothing staged is shared, and a
/// segment, once claimed, is walked to its end by the lane that claimed it. A marked lane
/// (its entry a start) is handled after the offset, in lane order:
/// - its staged bytes up to j flushed, then its segment finished (length, successor);
/// - the next segment claimed; its first byte written to its buffer directly (`take`),
///   and its staging starting at j + 1.
///
/// When a claim finds the queue exhausted (other walkers may still hold segments), every
/// other lane's staged bytes are flushed: an unmarked lane stepped at j, so through j + 1;
/// a marked lane not yet handled did not, so through j; a lane that took a segment at j
/// has nothing staged (st = j + 1). Its rows are copied back, and the unstaged tail walks
/// what is left. So every owned byte is in its buffer, and every owned segment finished,
/// before this returns: before `Relay::walk_and_leave`'s decrement, or the pool's run end.
///
/// SAFETY: as `walk_room`'s contract (`traverse`'s), and `claim` gives each segment index
/// below `starts.len()` once, across every walker
#[inline(never)]
unsafe fn traverse_staged<const L: usize, const MUT: u8, C: Claim, S: Stat>(
    t: *const u32,
    starts: &[u32],
    segs: Segs,
    claim: &C,
    cap: usize,
    st: &S,
) {
    // a pass marks its lanes in a u64
    const { assert!(L <= 64) };
    let next_seg = || {
        let r = claim.claim();
        st.claimed(r.is_some());
        r
    };
    let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let take = |seg: usize| -> Lane {
        // SAFETY: seg < ns, taken once, so this lane is its buffer's one user
        let b = unsafe { &mut *segs.bufs.add(seg) };
        let base = b.as_mut_ptr();
        // SAFETY: the starts are rows below n; the buffer has `cap` > 0 spare bytes
        let q = unsafe { *t.add(starts[seg] as usize) } & LINK;
        unsafe { *base = q as u8 };
        prefetch::<PF_L1>(t, q >> 8);
        Lane {
            row: q >> 8,
            seg,
            p: unsafe { base.add(1) },
            end: unsafe { base.add(b.capacity()) },
        }
    };
    let finish = |l: &Lane| {
        // SAFETY: the lane holds segment l.seg, its buffer's one user and its successor's
        // one writer; p is within the buffer, every byte below it written
        let b = unsafe { &mut *segs.bufs.add(l.seg) };
        let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
        unsafe { b.set_len(len) };
        unsafe { *segs.next.add(l.seg) = seg_of(l.row) };
        st.seg(len);
    };
    let grow = |l: &mut Lane| {
        seen!(GROWS);
        // SAFETY: as `finish`; every byte below p is committed (no staged byte is pending
        // when a lane grows: at a pass's start, and in the tail, which stages nothing)
        let b = unsafe { &mut *segs.bufs.add(l.seg) };
        let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
        unsafe { b.set_len(len) };
        b.reserve(b.capacity());
        l.p = unsafe { b.as_mut_ptr().add(len) };
        l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
    };
    // copy lane k's tile row from `from` to `to` into its buffer at p
    let flush = |l: &mut Lane, row: &[u8; TILE], from: usize, to: usize| {
        let len = to - from;
        // SAFETY: from..to is inside the row; the lane's buffer has room for the chunk's
        // bytes (the pass's room, as for the unstaged writes), so p + len is at most its end
        debug_assert!(from <= to && to <= TILE);
        if MUT != 0 {
            // (a mutated walk, the tests' only: checked in every build, so that a
            // mutation that would overrun a buffer panics here instead)
            assert!(
                len <= unsafe { l.end.offset_from(l.p) } as usize,
                "a flush past its buffer"
            );
        }
        debug_assert!(len <= unsafe { l.end.offset_from(l.p) } as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(row.as_ptr().add(from), l.p, len);
            st.fragment(len);
            l.p = l.p.add(len);
        }
    };

    let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
        row: 0,
        seg: usize::MAX,
        p: std::ptr::null_mut(),
        end: std::ptr::null_mut(),
    });
    let mut active = 0;
    while active < L {
        let Some(seg) = next_seg() else { break };
        lanes[active] = take(seg);
        active += 1;
    }
    if active == 0 {
        seen!(NO_SEGMENT);
    } else if active < L {
        seen!(NEVER_FULL);
    }
    if active == L {
        let mut rows = [0u32; L];
        for (k, l) in lanes.iter().enumerate() {
            rows[k] = l.row;
        }
        let mut tile = [[0u8; TILE]; L];
        'full: loop {
            // the pass's room: what every lane has spare, and no more than a buffer taken
            // mid-pass has after its first byte (`cap` - 1)
            let mut room = cap - 1;
            for l in lanes.iter_mut() {
                if l.p == l.end {
                    grow(l);
                }
                room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
            }
            debug_assert!(room > 0);
            if lanes
                .iter()
                .any(|l| unsafe { l.end.offset_from(l.p) } as usize == room)
            {
                seen!(PASS_ENDS_FULL);
            }
            if room > TILE {
                seen!(ROOM_OVER_TILE);
            }
            let mut done = 0;
            while done < room {
                let w = (room - done).min(TILE);
                let mut st = [0usize; L];
                for j in 0..w {
                    // the offset: every lane whose entry doesn't start a segment steps; a
                    // lane whose entry does is only marked, and handled below, in lane order
                    let rs = &mut rows;
                    let mut marked: u64 = 0;
                    let tp = tile.as_mut_ptr().cast::<u8>();
                    macro_rules! sstep {
                        ($k:expr) => {{
                            let k = $k;
                            // SAFETY: k < L; rows are below n (TVector); k * TILE + j is
                            // inside the tile (j < w <= TILE)
                            unsafe {
                                let r = rs.get_unchecked_mut(k);
                                let q = *t.add(*r as usize);
                                if q & START != 0 {
                                    marked |= 1 << k;
                                } else {
                                    *tp.add(k * TILE + j) = q as u8;
                                    *r = q >> 8;
                                    prefetch::<PF_L1>(t, q >> 8);
                                }
                            }
                        }};
                    }
                    let mut c = 0;
                    while c + 8 <= L {
                        sstep!(c);
                        sstep!(c + 1);
                        sstep!(c + 2);
                        sstep!(c + 3);
                        sstep!(c + 4);
                        sstep!(c + 5);
                        sstep!(c + 6);
                        sstep!(c + 7);
                        c += 8;
                    }
                    while c < L {
                        sstep!(c);
                        c += 1;
                    }
                    if marked != 0 && j == 0 && done > 0 {
                        seen!(MARK_AT_LATER_CHUNK_START);
                    }
                    #[cfg(test)]
                    if marked.count_ones() > 1 {
                        seen!(MARKED_SEVERAL);
                    }
                    #[cfg(test)]
                    let mut took = false;
                    while marked != 0 {
                        let k = marked.trailing_zeros() as usize;
                        marked &= marked - 1;
                        let l = &mut lanes[k];
                        l.row = rs[k];
                        if st[k] > 0 {
                            seen!(DONE_IN_TAKING_CHUNK);
                        }
                        // its segment's bytes end before offset j: committed, then its end
                        if MUT == mutation::FINISH_BEFORE_FLUSH {
                            finish(l);
                            flush(l, &tile[k], st[k], j);
                        } else {
                            flush(l, &tile[k], st[k], j);
                            finish(l);
                        }
                        let Some(seg) = next_seg() else {
                            // No segment is left unclaimed (other walkers may still hold
                            // theirs). Every lane's staged bytes are committed: an unmarked
                            // lane stepped at j (its byte is at j), a marked lane not yet
                            // handled did not (it sits on its segment's end), and a lane
                            // that took a segment at j has none staged (st = j + 1)
                            for (i, li) in lanes.iter_mut().enumerate() {
                                if MUT != mutation::NO_ROWS_BACK {
                                    li.row = rs[i];
                                }
                                if i == k || MUT == mutation::NO_EXHAUST_FLUSH {
                                    continue;
                                }
                                let waiting = marked >> i & 1 != 0;
                                let to = if MUT == mutation::OLD_RULE {
                                    if i < k {
                                        j + 1
                                    } else {
                                        j
                                    }
                                } else if MUT == mutation::WAITING_AS_STEPPED {
                                    j + 1
                                } else {
                                    j + usize::from(!waiting)
                                };
                                flush(li, &tile[i], st[i], to);
                                #[cfg(test)]
                                if i > k && !waiting {
                                    seen!(LATER_STEPPED);
                                    if li.p == li.end {
                                        seen!(LATER_STEPPED_FULL);
                                    }
                                    // SAFETY: the lane's row is below n
                                    if unsafe { *t.add(rs[i] as usize) } & START != 0 {
                                        seen!(LATER_STEPPED_ON_START);
                                    }
                                }
                            }
                            seen!(EXHAUSTED_MID_PASS);
                            #[cfg(test)]
                            {
                                if j == 0 && done > 0 {
                                    seen!(EXHAUSTED_AT_CHUNK_START);
                                }
                                if j + 1 == w && done + w < room {
                                    seen!(EXHAUSTED_AT_CHUNK_END);
                                }
                                if done + j + 1 == room {
                                    seen!(EXHAUSTED_AT_PASS_END);
                                }
                                if marked != 0 {
                                    seen!(ENDS_WITH_LANES_WAITING);
                                }
                                if took {
                                    seen!(ENDS_AFTER_A_TAKE);
                                }
                            }
                            break 'full;
                        };
                        seen!(MID_PASS_TAKES);
                        *l = take(seg);
                        rs[k] = l.row;
                        // its first byte is in its buffer; its next, at offset j + 1, is
                        // staged
                        st[k] = if MUT == mutation::ST_AT_J { j } else { j + 1 };
                        #[cfg(test)]
                        {
                            took = true;
                            if j + 1 == w {
                                seen!(TAKE_AT_CHUNK_END);
                            }
                            if done > 0 {
                                seen!(TAKE_IN_LATER_CHUNK);
                            }
                        }
                    }
                }
                for (k, l) in lanes.iter_mut().enumerate() {
                    flush(l, &tile[k], st[k], w);
                }
                done += w;
            }
        }
    }
    // the lanes still walking: a finished lane's successor is set, and this thread holds
    // every segment its lanes took, so it alone reads those slots
    let unfinished =
        |l: &Lane| l.seg != usize::MAX && unsafe { *segs.next.add(l.seg) } == usize::MAX;
    let mut live: Vec<Lane> = lanes[..active].iter().copied().filter(unfinished).collect();
    // a pass can end with a lane's buffer exactly full (its room was all its spare): the
    // tail writes before it tests, so each lane comes into it with a byte to spare
    for l in live.iter_mut() {
        if l.p == l.end {
            seen!(TAIL_ENTERED_FULL);
            if MUT != mutation::NO_TAIL_GROW {
                grow(l);
            }
        }
        // (checked before any tail write, in every build: the mutation above must panic
        // here, never write past the buffer)
        assert!(l.p < l.end, "a lane entered the tail full");
    }
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            let q = unsafe { *t.add(l.row as usize) };
            if q & START != 0 {
                finish(l);
                live.swap_remove(i);
                continue;
            }
            l.row = q >> 8;
            prefetch::<PF_L1>(t, l.row);
            unsafe {
                *l.p = q as u8;
                l.p = l.p.add(1);
            }
            st.tail();
            if l.p == l.end {
                grow(l);
            }
            i += 1;
        }
    }
}

/// A mark on the decoder's timeline (src/trace.rs), with the `trace` feature only
macro_rules! tmark {
    ($kind:ident, $place:expr) => {
        #[cfg(feature = "trace")]
        crate::trace::mark(crate::trace::Kind::$kind, $place);
    };
}

/// Spins before a pipelined team's thread yields while it waits for the build to end (the
/// registered wait's budget, and lever A's, so that park and yield differ in that alone)
pub(crate) const BARRIER_SPINS: u32 = if cfg!(miri) { 4 } else { 1 << 12 };

/// The spin-budget control's budget (`parallel::Levers::spin2k`): the pool's own `SPINS`
pub(crate) const SHORT_SPINS: u32 = SPINS;

/// Wait, spinning `SPIN` times then yielding, until `f` holds (the registered pipelined
/// team's wait at `BARRIER_SPINS`)
fn wait_until<const SPIN: u32>(f: impl Fn() -> bool, tally: &mut Tally) {
    let mut spins = 0u32;
    while !f() {
        if spins < SPIN {
            spins += 1;
            tally.spin();
            std::hint::spin_loop();
        } else {
            tally.yielded();
            thread::yield_now();
        }
    }
}

/// Where a pipelined run's inversion is: building (or built on one core, the fused build
/// having refused its inputs), walking, or over
// (one a run, on the leader's stack: the build's variant is large for its 256 starts)
#[allow(clippy::large_enum_variant)]
enum Stage<'a> {
    Build(Result<FusedBuild<'a>, &'a mut TVector>, &'a mut SegScratch),
    Walk(RoomWalk<'a>),
    Over,
}

/// A pipelined run's shared state (`invert_pipelined`)
struct Relay<'a> {
    stage: std::cell::UnsafeCell<Stage<'a>>,
    ll: &'a [u8],
    counts: &'a [u32; 256],
    orig_ptr: u32,
    segments: usize,
    /// the walk's shares staged (`RoomWalk`'s `staged`)
    staged: bool,
    out: std::cell::UnsafeCell<&'a mut Vec<u8>>,
    /// the block's place, for the trace's marks only
    #[cfg_attr(not(feature = "trace"), allow(dead_code))]
    place: u64,
    /// the helpers whose build share has ended
    built: AtomicUsize,
    /// set (release) once the stage is the walk or over: the build's last helper made it so
    walk_ready: std::sync::atomic::AtomicBool,
    /// lever A's waiters (`PARK`): the threads parked, or about to park, until `walk_ready`
    /// is set; whoever sets it unparks them. Always empty without `PARK`
    sleepers: Mutex<Vec<Thread>>,
    /// the threads in the walk, plus the helpers not yet in it; whoever leaves it at 0
    /// ends the walk. Once 0 it stays 0: a thread joins only while it is above 0
    walkers: AtomicUsize,
    /// a stage panicked: the output is not to be trusted
    failed: std::sync::atomic::AtomicBool,
    panic: Mutex<Option<Box<dyn Any + Send>>>,
    /// the `walk-stats` feature's per-block counts (empty without it)
    #[cfg_attr(not(feature = "walk-stats"), allow(dead_code))]
    stats: RelayStats,
}

/// A pipelined run's block summary for the `walk-stats` feature, counted apart from the
/// walker calls' records so that each can check the other: the team, the pool indices that
/// entered the walk (each sets its bit on entering, before its share), whether the
/// leader joined, and the helpers counted into the build
#[derive(Default)]
struct RelayStats {
    #[cfg(feature = "walk-stats")]
    team: usize,
    #[cfg(feature = "walk-stats")]
    entered: AtomicU64,
    #[cfg(feature = "walk-stats")]
    joined: AtomicBool,
    #[cfg(feature = "walk-stats")]
    built: AtomicU64,
}

#[cfg(feature = "walk-stats")]
impl RelayStats {
    fn new() -> Self {
        RelayStats {
            team: walk_stats::team(),
            ..Default::default()
        }
    }

    fn share(&self, block: u64, thread: usize) {
        walk_stats::push(format!(
            "{{\"kind\":\"share\",\"team\":{},\"block\":{block},\"thread\":{thread}}}",
            self.team
        ));
    }

    /// The block's summary, by the thread ending its walk: its starts, the segments' bytes
    /// and rows from the scratch, and the bits above
    fn summary(&self, block: u64, s: &SegScratch) {
        let ns = s.starts.len();
        let bytes: usize = s.bufs[..ns].iter().map(Vec::len).sum();
        walk_stats::push(format!(
            "{{\"kind\":\"summary\",\"team\":{},\"block\":{block},\"rows\":{},\"starts\":{ns},\"seg_bytes\":{bytes},\"entered\":{},\"joined\":{},\"built\":{}}}",
            self.team,
            s.n,
            self.entered.load(Ordering::Relaxed),
            self.joined.load(Ordering::Relaxed),
            self.built.load(Ordering::Relaxed),
        ));
    }
}
// SAFETY: `stage` and `out` are touched by one thread at a time, as the protocol in
// `invert_pipelined` says: the stage shared read-only while its shares run, and changed
// only by the one thread that ends the build (the last helper to count itself in
// `built`) or the walk (the thread that takes `walkers` to 0), each change released to
// the next readers by `walk_ready` or by the pool's run ending
unsafe impl Sync for Relay<'_> {}

impl Relay<'_> {
    /// Wait until the build has ended (`walk_ready`): `SPIN` spins, then, without `PARK`,
    /// `yield_now` in a loop (the registered wait, at `BARRIER_SPINS`), or with `PARK`
    /// (lever A), this thread is listed in `sleepers` and parks until `set_walk_ready`
    /// unparks it, as the pool's own waits park.
    ///
    /// No wake-up is lost: the waiter lists itself under the lock, then loads the flag;
    /// the setter stores the flag (release), then takes the lock and unparks everyone
    /// listed. If the waiter's lock came first, the setter finds it listed and unparks it
    /// (an unpark before the park makes the park return at once); if the setter's came
    /// first, its store happened before the waiter's lock, so the waiter's load sees the
    /// flag and it never parks. A spurious wake-up, or a token left from an earlier
    /// unpark, only makes it check the flag again.
    fn wait_walk_ready<const PARK: bool, const SPIN: u32>(&self, site: Site) {
        let mut tally = Tally::new();
        self.wait_walk_ready_tallied::<PARK, SPIN>(&mut tally);
        tally.end(site);
    }

    fn wait_walk_ready_tallied<const PARK: bool, const SPIN: u32>(&self, tally: &mut Tally) {
        let ready = || self.walk_ready.load(Ordering::Acquire);
        if !PARK {
            return wait_until::<SPIN>(ready, tally);
        }
        for _ in 0..SPIN {
            if ready() {
                return;
            }
            tally.spin();
            std::hint::spin_loop();
        }
        lock(&self.sleepers).push(thread::current());
        while !ready() {
            tally.park();
            thread::park();
        }
    }

    /// The build's end published to the walk's threads (`wait_walk_ready`): with `PARK`,
    /// every thread listed as waiting is unparked
    fn set_walk_ready<const PARK: bool>(&self) {
        self.walk_ready.store(true, Ordering::Release);
        if PARK {
            for t in lock(&self.sleepers).drain(..) {
                t.unpark();
            }
        }
    }

    fn caught(&self, r: std::thread::Result<()>) {
        if let Err(p) = r {
            self.failed.store(true, Ordering::Relaxed);
            let mut slot = lock(&self.panic);
            if slot.is_none() {
                *slot = Some(p);
            }
        }
    }

    /// The build's end, on the last helper: the end check (or the one-core build, if the
    /// fused one refused or failed: a bug, as `bwt::Cooperative`), then the walk's set-up
    ///
    /// SAFETY: the caller is the one thread that counted the last helper into `built`,
    /// so no other thread reads the stage until `walk_ready` is set
    unsafe fn end_build(&self) {
        let stage = unsafe { &mut *self.stage.get() };
        let Stage::Build(..) = stage else {
            return;
        };
        let Stage::Build(build, seg) = std::mem::replace(stage, Stage::Over) else {
            unreachable!()
        };
        if self.failed.load(Ordering::Relaxed) {
            return; // (a share panicked: no walk, the stage over)
        }
        tmark!(WalkSetupStart, self.place);
        let r = catch_unwind(AssertUnwindSafe(|| {
            let tt = match build {
                Ok(b) => match b.finish() {
                    (Ok(()), tt) => tt,
                    (Err(e), tt) => {
                        debug_assert!(
                            false,
                            "the cooperative build refused its checkpoints: {}",
                            e.0
                        );
                        crate::bwt::build(self.ll, self.counts, tt);
                        tt
                    }
                },
                Err(tt) => tt,
            };
            RoomWalk::new(tt, self.orig_ptr, seg, self.segments, self.staged)
        }));
        tmark!(WalkSetupEnd, self.place);
        match r {
            Ok(w) => *stage = Stage::Walk(w),
            Err(p) => self.caught(Err(p)),
        }
    }

    /// This thread's part in the walk (it is counted in `walkers`), then its leave: the
    /// last to leave ends the walk and assembles the output
    ///
    /// `who` is this thread's index in the team's pool (0 the leader), for the `walk-stats`
    /// feature's records only
    #[cfg_attr(not(feature = "walk-stats"), allow(unused_variables))]
    fn walk_and_leave<const L: usize>(&self, who: usize) {
        #[cfg(feature = "walk-stats")]
        self.stats
            .entered
            .fetch_or(walk_stats::bit(who), Ordering::Relaxed);
        // SAFETY: `walk_ready` was acquired, and the stage changes again only when the
        // last walker leaves, which this thread has not
        if let Stage::Walk(w) = unsafe { &*self.stage.get() } {
            tmark!(WalkStart, self.place);
            #[cfg(not(feature = "walk-stats"))]
            self.caught(catch_unwind(AssertUnwindSafe(|| w.share::<L>())));
            #[cfg(feature = "walk-stats")]
            {
                let c = walk_stats::Counts::default();
                self.caught(catch_unwind(AssertUnwindSafe(|| w.share_with::<L, _>(&c))));
                let walker = if w.staged {
                    "traverse_staged"
                } else {
                    "traverse"
                };
                walk_stats::call(self.stats.team, self.place, who, walker, &c);
            }
            tmark!(WalkEnd, self.place);
        }
        if self.walkers.fetch_sub(1, Ordering::AcqRel) == 1 {
            // SAFETY: every other walker has left, and none can join (walkers is 0)
            let stage = unsafe { &mut *self.stage.get() };
            if let Stage::Walk(w) = std::mem::replace(stage, Stage::Over) {
                tmark!(AssembleStart, self.place);
                let r = catch_unwind(AssertUnwindSafe(|| {
                    let (_, s) = w.finish();
                    #[cfg(feature = "walk-stats")]
                    self.stats.summary(self.place, s);
                    if !self.failed.load(Ordering::Relaxed) {
                        // SAFETY: only the thread ending the walk touches `out`
                        s.assemble(unsafe { &mut *self.out.get() });
                    }
                }));
                tmark!(AssembleEnd, self.place);
                self.caught(r);
            }
        }
    }

    /// The leader's join, once its own work is done: into the walk if it has not ended
    fn join<const L: usize, const PARK: bool, const SPIN: u32>(&self) {
        tmark!(BuildWaitStart, self.place);
        self.wait_walk_ready::<PARK, SPIN>(Site::WalkLeader);
        tmark!(BuildWaitEnd, self.place);
        let mut v = self.walkers.load(Ordering::Acquire);
        loop {
            if v == 0 {
                return;
            }
            match self
                .walkers
                .compare_exchange_weak(v, v + 1, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(now) => v = now,
            }
        }
        #[cfg(feature = "walk-stats")]
        self.stats.joined.store(true, Ordering::Relaxed);
        self.walk_and_leave::<L>(0);
    }
}

/// One run of a pipelined team (`parallel`'s pipelined worker): the pool's helpers
/// (threads 1..k) invert a block, `bwt::Cooperative`'s table and walk with no thread of its
/// own, while the caller (thread 0) runs `lead`, its own serial work on other blocks
/// (another block's RLE1 and CRC, the next block's move-to-front), then joins the walk if
/// it is still going. Exactly `bwt::walk`'s output, in `out`.
///
/// The helpers' run: each its share of the fused build (`FusedBuild`, k − 1 shares); the
/// last to end it makes the end check and sets up the walk (`RoomWalk::new`), and sets
/// `walk_ready`; the others wait for that (spinning, then yielding); then all walk,
/// taking segments from the one counter, and the last to leave the walk unflags the
/// starts and assembles the output into `out`. No helper waits on the caller, so `lead`
/// can take as long as it likes.
///
/// A panic in a helper's stage is caught there and counted (no thread is left waiting for
/// it: the build's helpers all count in, panicked or not, and the walk's last to leave
/// ends it). The first is returned as an Err; then `out` holds nothing to trust, and the
/// table and scratch are to be reset. A panic in `lead` is resumed (`Pool::run`) once the
/// helpers have ended.
// (the tests' name for the registered waits; the decoder calls `invert_pipelined_with`)
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn invert_pipelined<const L: usize>(
    pool: &Pool,
    ll: &[u8],
    counts: &[u32; 256],
    checkpoints: &[[u32; 256]],
    stride: usize,
    orig_ptr: u32,
    tt: &mut TVector,
    seg: &mut SegScratch,
    out: &mut Vec<u8>,
    lead: &(dyn Fn() + Sync),
    place: u64,
) -> Result<(), Box<dyn Any + Send>> {
    invert_pipelined_with::<L, false, BARRIER_SPINS>(
        pool,
        ll,
        counts,
        checkpoints,
        stride,
        orig_ptr,
        tt,
        seg,
        out,
        lead,
        place,
        false,
    )
}

/// `invert_pipelined`, its waits for the build's end chosen: `SPIN` spins, then a yield in a
/// loop (`PARK` false: at `BARRIER_SPINS`, `invert_pipelined` itself; at `SHORT_SPINS`, the
/// spin-budget control) or a park (`PARK` true, lever A: research, `parallel::Levers`). The
/// same output, errors and panics whatever the wait.
#[allow(clippy::too_many_arguments)]
pub(crate) fn invert_pipelined_with<const L: usize, const PARK: bool, const SPIN: u32>(
    pool: &Pool,
    ll: &[u8],
    counts: &[u32; 256],
    checkpoints: &[[u32; 256]],
    stride: usize,
    orig_ptr: u32,
    tt: &mut TVector,
    seg: &mut SegScratch,
    out: &mut Vec<u8>,
    lead: &(dyn Fn() + Sync),
    place: u64,
    staged: bool,
) -> Result<(), Box<dyn Any + Send>> {
    // (a pool of one thread runs the helpers' part itself, after `lead`)
    let helpers = pool.threads().saturating_sub(1).max(1);
    let shares = helpers.min(MAX_SHARES);
    tmark!(BuildSetupStart, place);
    let build = match FusedBuild::new(ll, counts, checkpoints, stride, shares, tt) {
        Ok(b) => Ok(b),
        Err((e, tt)) => {
            debug_assert!(
                false,
                "the cooperative build refused its checkpoints: {}",
                e.0
            );
            // (built on this thread, before the run: a bug's path, as `bwt::Cooperative`)
            crate::bwt::build(ll, counts, tt);
            Err(tt)
        }
    };
    tmark!(BuildSetupEnd, place);
    let relay = Relay {
        stage: std::cell::UnsafeCell::new(Stage::Build(build, seg)),
        ll,
        counts,
        orig_ptr,
        segments: crate::bwt::FLAGGED_SEGMENTS * helpers,
        staged,
        out: std::cell::UnsafeCell::new(out),
        built: AtomicUsize::new(0),
        walk_ready: AtomicBool::new(false),
        sleepers: Mutex::new(if PARK {
            Vec::with_capacity(pool.threads())
        } else {
            Vec::new()
        }),
        walkers: AtomicUsize::new(helpers),
        failed: AtomicBool::new(false),
        panic: Mutex::new(None),
        place,
        #[cfg(feature = "walk-stats")]
        stats: RelayStats::new(),
        #[cfg(not(feature = "walk-stats"))]
        stats: RelayStats::default(),
    };
    let relay = &relay;
    let helper = |h: usize| {
        tmark!(HelperStart, place);
        // SAFETY: the stage is shared read-only until the last helper counts in below
        if let Stage::Build(Ok(b), _) = unsafe { &*relay.stage.get() } {
            tmark!(BuildStart, place);
            let r = catch_unwind(AssertUnwindSafe(|| b.share(h)));
            #[cfg(feature = "walk-stats")]
            if r.is_ok() {
                relay.stats.share(place, h + 1);
            }
            relay.caught(r);
            tmark!(BuildEnd, place);
        }
        #[cfg(feature = "walk-stats")]
        relay
            .stats
            .built
            .fetch_or(walk_stats::bit(h + 1), Ordering::Relaxed);
        if relay.built.fetch_add(1, Ordering::AcqRel) + 1 == helpers {
            // SAFETY: this is the last helper to count in: every share has ended
            unsafe { relay.end_build() };
            relay.set_walk_ready::<PARK>();
        } else {
            tmark!(BuildWaitStart, place);
            relay.wait_walk_ready::<PARK, SPIN>(Site::WalkHelper);
            tmark!(BuildWaitEnd, place);
        }
        relay.walk_and_leave::<L>(h + 1);
        tmark!(HelperEnd, place);
    };
    if pool.threads() < 2 {
        tmark!(LeadStart, place);
        lead();
        tmark!(LeadEnd, place);
        helper(0);
    } else {
        pool.run(&|j| {
            if j == 0 {
                tmark!(LeadStart, place);
                lead();
                tmark!(LeadEnd, place);
                relay.join::<L, PARK, SPIN>();
            } else {
                helper(j - 1);
            }
        });
    }
    let caught = lock(&relay.panic).take();
    caught.map_or(Ok(()), Err)
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

    /// A plain pool's helpers and a pipelined team's are told apart by name (`ab`'s census
    /// reads the names), and both counted as helpers by the `mibz2-coop` prefix
    #[test]
    fn helper_names() {
        for (p, want) in [
            (Pool::new(3).unwrap(), ["mibz2-coop-1", "mibz2-coop-2"]),
            (
                Pool::pipelined(3).unwrap(),
                ["mibz2-coop-p1", "mibz2-coop-p2"],
            ),
        ] {
            let names: Vec<_> = p
                .helpers
                .iter()
                .map(|h| h.thread().name().unwrap().to_string())
                .collect();
            assert_eq!(names, want);
            // (Linux keeps 15 bytes of a thread's name, which is what the census reads)
            assert!(names
                .iter()
                .all(|n| n.len() <= 15 && n.starts_with("mibz2-coop")));
            each_index_once(&p);
        }
    }

    /// The waits' counts (feature `wait-stats`): a pool's waits are counted at their sites,
    /// each wait ends one way, and a helper left idle long enough parks. The counts are
    /// global and other tests add to them meanwhile, so only what they can't take away is
    /// checked: lower bounds, and each site's own sums.
    #[cfg(feature = "wait-stats")]
    #[test]
    fn wait_stats_count_the_pools_waits() {
        use wait_stats::{get, SITES};
        let p = Pool::new(3).unwrap();
        let before = get(Site::CallerJoin).waits;
        let parks = get(Site::HelperIdle).parks;
        for _ in 0..20 {
            each_index_once(&p);
        }
        thread::sleep(std::time::Duration::from_millis(20));
        each_index_once(&p);
        assert!(get(Site::CallerJoin).waits >= before + 21);
        assert!(get(Site::HelperIdle).parks > parks, "an idle helper parked");
        drop(p);
        for s in SITES {
            let c = get(s);
            assert!(c.spins_wasted <= c.spins, "{s:?}");
            assert!(c.parked <= c.parks && c.yielded <= c.yields, "{s:?}");
        }
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

    /// The shared walk on `ll`'s table from `orig_ptr`, against `bwt::walk` and the
    /// segments' oracle, with `segments` a block
    fn walk_matches(
        p: &Pool,
        ll: &[u8],
        orig_ptr: u32,
        segments: usize,
        s: &mut SegScratch,
        what: &str,
        staged: bool,
    ) {
        let counts = counts_of(ll);
        let mut tt = TVector::new();
        crate::bwt::build(ll, &counts, &mut tt);
        let mut want = Vec::new();
        crate::bwt::walk(&tt, orig_ptr, &mut want);
        let before = tt.0.clone();
        walk_room::<8>(&mut tt, orig_ptr, s, p, segments, staged);
        assert!(tt.0 == before, "{what}: the flags were not all taken off");
        // the segments are the oracle's, on every cycle, not only those the output joins
        let oracle = crate::bwt::tests::segment_oracle(&tt, &s.starts);
        for (i, (bytes, next)) in oracle.iter().enumerate() {
            assert!(
                s.bufs[i] == *bytes,
                "{what}: segment {i}'s bytes, k {}",
                p.threads()
            );
            assert_eq!(s.next[i], *next, "{what}: segment {i}'s successor");
        }
        let mut out = vec![1, 2, 3];
        s.assemble(&mut out);
        assert!(
            out == want,
            "{what}: k {}, segments {segments}: not walk's output",
            p.threads()
        );
    }

    #[test]
    fn walk_room_is_walk() {
        let mut s = SegScratch::default();
        for k in 1..=4 {
            let p = Pool::new(k).unwrap();
            for (name, ll) in columns(30_000) {
                if ll.is_empty() {
                    continue;
                }
                let n = ll.len() as u32;
                for segments in [1, 2, 7, 64, 512 * k, 4096] {
                    for orig_ptr in [0, n / 3, n - 1] {
                        for staged in [false, true] {
                            walk_matches(&p, &ll, orig_ptr, segments, &mut s, &name, staged);
                        }
                    }
                }
            }
        }
    }

    /// Small blocks under Miri at k = 2 and 3: the shared buffers' and successors' one
    /// writer each (its race detector), the lanes' unchecked writes, and every handoff
    /// reached (lanes taken mid-pass, full passes, the full phase ending mid-pass, the
    /// tail entered full, buffers grown)
    #[test]
    fn miri_walk_room_handoffs() {
        use handoffs::*;
        let mut s = SegScratch::default();
        let mut r = xorshift(11);
        let cases: Vec<Vec<u8>> = vec![
            (0..1500).map(|_| (r() % 4) as u8).collect(),
            b"ab".repeat(600),
            (0..900).map(|i| ((i / 37) % 3) as u8).collect(),
        ];
        for k in [2, 3] {
            let p = Pool::new(k).unwrap();
            for (c, ll) in cases.iter().enumerate() {
                for segments in [24, 96] {
                    for staged in [false, true] {
                        walk_matches(
                            &p,
                            ll,
                            (ll.len() / 2) as u32,
                            segments,
                            &mut s,
                            &format!("case {c}"),
                            staged,
                        );
                    }
                }
            }
        }
        for (name, c) in [
            ("GROWS", &GROWS),
            ("PASS_ENDS_FULL", &PASS_ENDS_FULL),
            ("MID_PASS_TAKES", &MID_PASS_TAKES),
            ("FULL_PHASE_ENDS_MID_PASS", &FULL_PHASE_ENDS_MID_PASS),
        ] {
            assert!(c.load(Ordering::Relaxed) > 0, "never reached: {name}");
        }
    }

    /// `bwt::Cooperative` on a block past `MIN_COOP` really runs the team (its pool spawned,
    /// no fallback), and gives `walk`'s output; below it, the one-core kernel
    #[test]
    fn cooperative_kernel_engages() {
        use crate::bwt::InverseBwt;
        let mut r = xorshift(3);
        let big: Vec<u8> = (0..crate::bwt::MIN_COOP + 12_345)
            .map(|_| (r() % 9) as u8)
            .collect();
        let small = &big[..crate::bwt::MIN_COOP - 1];
        for ll in [&big[..], small] {
            let counts = counts_of(ll);
            let cps = checkpoints_of(ll, crate::mtf::CHECKPOINT);
            let mut tt = TVector::new();
            crate::bwt::build(ll, &counts, &mut tt);
            let mut want = Vec::new();
            crate::bwt::walk(&tt, 17, &mut want);
            let mut c = crate::bwt::Cooperative::<3>::default();
            assert!(c.wants_checkpoints());
            let mut out = Vec::new();
            c.invert_checkpointed(ll, &counts, &cps, 17, &mut out);
            assert!(out == want);
            assert_eq!(c.engaged(), ll.len() >= crate::bwt::MIN_COOP);
        }
    }

    /// The build and the walk cut into their parts (`FusedBuild`, `RoomWalk`), their shares
    /// run on `s` threads of the test's own, against `bwt::build` and `bwt::walk`; then the
    /// pipelined run (`invert_pipelined`) on a pool of s helpers and a leader, the leader's
    /// own work running meanwhile: byte for byte the same
    fn split_matches(s: usize, ll: &[u8], stride: usize, orig_ptrs: &[u32], what: &str) {
        let counts = counts_of(ll);
        let cps = checkpoints_of(ll, stride);
        let mut want_tt = TVector::new();
        crate::bwt::build(ll, &counts, &mut want_tt);
        // the build's shares, each on a thread of its own (and each run twice: the second
        // claim writes nothing)
        let mut tt = TVector::new();
        let b = FusedBuild::new(ll, &counts, &cps, stride, s, &mut tt)
            .map_err(|(e, _)| e)
            .unwrap();
        thread::scope(|sc| {
            for j in 0..s {
                let b = &b;
                sc.spawn(move || {
                    b.share(j);
                    b.share(j);
                });
            }
        });
        let (r, tt) = b.finish();
        r.unwrap();
        assert!(tt.0 == want_tt.0, "{what}: shares {s}: the split build");
        // a share left out: refused, the table left empty
        if s > 1 {
            let mut t2 = TVector::new();
            let b = FusedBuild::new(ll, &counts, &cps, stride, s, &mut t2)
                .map_err(|(e, _)| e)
                .unwrap();
            for j in 1..s {
                b.share(j);
            }
            let (r, t2) = b.finish();
            assert!(r.is_err() && t2.0.is_empty(), "{what}: a share missing");
        }
        let mut seg = SegScratch::default();
        for &op in orig_ptrs {
            let mut want = Vec::new();
            crate::bwt::walk(&want_tt, op, &mut want);
            // the walk's shares on s threads
            for staged in [false, true] {
                let before = tt.0.clone();
                let w = RoomWalk::new(&mut *tt, op, &mut seg, 64 * s, staged);
                thread::scope(|sc| {
                    for _ in 0..s {
                        let w = &w;
                        sc.spawn(move || w.share::<8>());
                    }
                });
                let (tt2, seg2) = w.finish();
                assert!(tt2.0 == before, "{what}: the flags were not all taken off");
                let mut out = vec![9];
                seg2.assemble(&mut out);
                assert!(
                    out == want,
                    "{what}: shares {s}, orig_ptr {op}, staged {staged}: the split walk"
                );
            }
            // the pipelined run: s helpers, and the leader's work meanwhile, its waits
            // spinning and yielding (as registered) or parking (lever A), its walk staged
            // or not
            for (wait, staged) in WAITS.into_iter().flat_map(|w| [(w, false), (w, true)]) {
                let p = Pool::new(s + 1).unwrap();
                let mut pt = TVector::new();
                let mut ps = SegScratch::default();
                let mut out = vec![7; 3];
                let led = AtomicUsize::new(0);
                pipelined::<8>(
                    wait,
                    &p,
                    ll,
                    &counts,
                    &cps,
                    stride,
                    op,
                    &mut pt,
                    &mut ps,
                    &mut out,
                    &|| {
                        led.fetch_add(1, Ordering::Relaxed);
                    },
                    staged,
                )
                .unwrap();
                assert_eq!(led.load(Ordering::Relaxed), 1);
                assert!(
                    out == want,
                    "{what}: {s} helpers, orig_ptr {op}, {wait:?}, staged {staged}: the pipelined run"
                );
                assert!(pt.0 == want_tt.0, "{what}: the pipelined run's table");
            }
        }
    }

    /// The waits the tests run: the registered yield, the spin-budget control, lever A,
    /// and lever A at the short budget
    #[derive(Clone, Copy, Debug)]
    enum Wait2 {
        Yield,
        Yield2k,
        Park,
        Park2k,
    }
    const WAITS: [Wait2; 4] = [Wait2::Yield, Wait2::Yield2k, Wait2::Park, Wait2::Park2k];

    /// `invert_pipelined_with`, at one of the waits
    #[allow(clippy::too_many_arguments)]
    fn pipelined<const L: usize>(
        wait: Wait2,
        pool: &Pool,
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        stride: usize,
        orig_ptr: u32,
        tt: &mut TVector,
        seg: &mut SegScratch,
        out: &mut Vec<u8>,
        lead: &(dyn Fn() + Sync),
        staged: bool,
    ) -> Result<(), Box<dyn Any + Send>> {
        let f = match wait {
            Wait2::Yield => invert_pipelined_with::<L, false, BARRIER_SPINS>,
            Wait2::Yield2k => invert_pipelined_with::<L, false, SHORT_SPINS>,
            Wait2::Park => invert_pipelined_with::<L, true, BARRIER_SPINS>,
            Wait2::Park2k => invert_pipelined_with::<L, true, SHORT_SPINS>,
        };
        f(
            pool,
            ll,
            counts,
            checkpoints,
            stride,
            orig_ptr,
            tt,
            seg,
            out,
            lead,
            0,
            staged,
        )
    }

    #[test]
    fn split_build_and_walk_are_build_and_walk() {
        let stride = crate::mtf::CHECKPOINT;
        for s in [1, 2, 3] {
            for (name, ll) in columns(30_000) {
                if ll.len() < 2 {
                    continue;
                }
                let n = ll.len() as u32;
                split_matches(s, &ll, 1000, &[0, n / 3, n - 1], &name);
            }
            // a block past MIN_COOP, at the decoder's stride
            let mut r = xorshift(17 + s as u64);
            let big: Vec<u8> = (0..crate::bwt::MIN_COOP + 4321)
                .map(|_| (r() % 11) as u8)
                .collect();
            split_matches(s, &big, stride, &[12_345], "big");
        }
    }

    /// The same under Miri, small blocks: the split parts' shared writes (its race
    /// detector), the pipelined run's hand-offs between helpers and the leader's join
    #[test]
    fn miri_split_build_and_walk() {
        let mut r = xorshift(23);
        let ll: Vec<u8> = (0..1300).map(|_| (r() % 5) as u8).collect();
        for s in [1, 2, 3] {
            split_matches(s, &ll, 64, &[650], "miri");
        }
    }

    /// A pipelined run whose leader takes longer than the helpers, and one whose leader
    /// ends at once: the leader joins the walk or finds it over, and the output is the same
    #[test]
    fn pipelined_leader_late_or_early() {
        let mut r = xorshift(29);
        let ll: Vec<u8> = (0..crate::bwt::MIN_COOP + 99)
            .map(|_| (r() % 7) as u8)
            .collect();
        let counts = counts_of(&ll);
        let cps = checkpoints_of(&ll, crate::mtf::CHECKPOINT);
        let mut tt = TVector::new();
        crate::bwt::build(&ll, &counts, &mut tt);
        let mut want = Vec::new();
        crate::bwt::walk(&tt, 77, &mut want);
        for k in [2, 3, 4] {
            let p = Pool::new(k).unwrap();
            for (late, wait, staged) in WAITS
                .into_iter()
                .flat_map(|w| [(false, w), (true, w)])
                .flat_map(|(l, w)| [(l, w, false), (l, w, true)])
            {
                let (mut tt, mut seg, mut out) =
                    (TVector::new(), SegScratch::default(), Vec::new());
                let lead = || {
                    if late {
                        thread::sleep(std::time::Duration::from_millis(30));
                    }
                };
                pipelined::<32>(
                    wait, &p, &ll, &counts, &cps, CP, 77, &mut tt, &mut seg, &mut out, &lead,
                    staged,
                )
                .unwrap();
                assert!(out == want, "k {k}, late {late}, {wait:?}, staged {staged}");
            }
        }
    }

    /// A panic in the leader's work is resumed once the helpers have ended, and the pool
    /// and buffers still give the right output afterwards
    #[test]
    fn pipelined_leader_panic() {
        let mut r = xorshift(31);
        let ll: Vec<u8> = (0..5000).map(|_| (r() % 7) as u8).collect();
        let counts = counts_of(&ll);
        let cps = checkpoints_of(&ll, crate::mtf::CHECKPOINT);
        let mut want_tt = TVector::new();
        crate::bwt::build(&ll, &counts, &mut want_tt);
        let mut want = Vec::new();
        crate::bwt::walk(&want_tt, 3, &mut want);
        let p = Pool::new(3).unwrap();
        let (mut tt, mut seg, mut out) = (TVector::new(), SegScratch::default(), Vec::new());
        let r = catch_unwind(AssertUnwindSafe(|| {
            invert_pipelined::<8>(
                &p,
                &ll,
                &counts,
                &cps,
                CP,
                3,
                &mut tt,
                &mut seg,
                &mut out,
                &|| panic!("lead"),
                0,
            )
        }));
        assert_eq!(*r.unwrap_err().downcast_ref::<&str>().unwrap(), "lead");
        let mut tt = TVector::new();
        let mut seg = SegScratch::default();
        invert_pipelined::<8>(
            &p,
            &ll,
            &counts,
            &cps,
            CP,
            3,
            &mut tt,
            &mut seg,
            &mut out,
            &|| {},
            0,
        )
        .unwrap();
        assert!(out == want);
    }

    const CP: usize = crate::mtf::CHECKPOINT;

    /// Lever A's wait alone, its race between a waiter listing itself and the setter's
    /// unpark: many waiters, the flag set after a spin, a yield or a sleep, so some find it
    /// set before they list themselves, some after, and some park first (no wake-up lost:
    /// a lost wake-up would hang the test)
    #[test]
    fn miri_parked_wait_loses_no_wakeup() {
        let rounds = if cfg!(miri) { 6 } else { 300 };
        for i in 0..rounds {
            let mut tt = TVector::new();
            let mut seg = SegScratch::default();
            let mut out = Vec::new();
            let relay = Relay {
                stage: std::cell::UnsafeCell::new(Stage::Build(Err(&mut tt), &mut seg)),
                ll: &[],
                counts: &[0; 256],
                orig_ptr: 0,
                segments: 1,
                staged: false,
                out: std::cell::UnsafeCell::new(&mut out),
                place: 0,
                built: AtomicUsize::new(0),
                walk_ready: AtomicBool::new(false),
                sleepers: Mutex::new(Vec::new()),
                walkers: AtomicUsize::new(0),
                failed: AtomicBool::new(false),
                panic: Mutex::new(None),
                stats: RelayStats::default(),
            };
            let woke = AtomicUsize::new(0);
            thread::scope(|sc| {
                for _ in 0..3 {
                    sc.spawn(|| {
                        relay.wait_walk_ready::<true, BARRIER_SPINS>(Site::WalkHelper);
                        woke.fetch_add(1, Ordering::Relaxed);
                    });
                }
                match i % 3 {
                    0 => {}
                    1 => thread::yield_now(),
                    _ => thread::sleep(std::time::Duration::from_micros(200)),
                }
                relay.set_walk_ready::<true>();
            });
            assert_eq!(woke.load(Ordering::Relaxed), 3);
            // (a waiter that listed itself after the setter's unparks found the flag set
            // and never parked: its entry is left in the list, dropped with the run's relay)
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

    // ---- The staged walk's transitions (`traverse_staged`), forced ----
    //
    // A walker's claims are cut off after a budget of b (`RoomWalk::share_staged_budget`): it
    // then finds the queue exhausted, exactly as when another walker has claimed the rest,
    // and a second walker (staged, or the unstaged `traverse`) walks what is left. Every b
    // from 0 to past the segment count is tried, so the exhaustion lands at every claim
    // the walk makes: in the first lanes' takes, mid-pass, after a take in the same
    // offset, at chunk starts and ends and pass ends. Each case checks every segment and
    // successor against the oracle, and the output against `bwt::walk` and against the
    // unstaged team walk's.

    /// One case: Ok, or what went wrong (a panic counts as caught, for the mutations)
    #[allow(clippy::too_many_arguments)]
    fn staged_case<const L: usize, const MUT: u8>(
        tt: &mut TVector,
        orig_ptr: u32,
        segments: usize,
        budget: Option<usize>,
        second_staged: bool,
        want: &[u8],
        old: &[u8],
        s: &mut SegScratch,
    ) -> Result<(), String> {
        // (fresh buffers, so each case's capacities are `cap`: a buffer grown in an earlier
        // case would hide the grows)
        *s = SegScratch::default();
        let before = tt.0.clone();
        let r = catch_unwind(AssertUnwindSafe(|| {
            let w = RoomWalk::new(&mut *tt, orig_ptr, &mut *s, segments, true);
            w.share_staged_budget::<L, MUT>(budget);
            if second_staged {
                w.share_staged_budget::<L, MUT>(None);
            } else {
                w.share_unstaged::<L>();
            }
            w.finish();
        }));
        if r.is_err() {
            tt.0 = before;
            *s = SegScratch::default();
            return Err("panicked".into());
        }
        if tt.0 != before {
            return Err("the flags were not all taken off".into());
        }
        let oracle = crate::bwt::tests::segment_oracle(tt, &s.starts);
        for (i, (bytes, next)) in oracle.iter().enumerate() {
            if s.bufs[i] != *bytes {
                return Err(format!("segment {i}'s bytes"));
            }
            if s.next[i] != *next {
                return Err(format!("segment {i}'s successor"));
            }
        }
        let mut out = vec![5];
        s.assemble(&mut out);
        if out != want {
            return Err("not walk's output".into());
        }
        if out != old {
            return Err("not the unstaged team walk's output".into());
        }
        Ok(())
    }

    /// The crafted tables of the transition tests: periodic and multicycle tables, runs,
    /// one byte value (every row its own cycle), random, and few symbols
    fn staged_tables(scale: usize) -> Vec<(String, Vec<u8>)> {
        let mut r = xorshift(41);
        let mut v = vec![
            (
                "random 4".to_string(),
                (0..4 * scale).map(|_| (r() % 4) as u8).collect::<Vec<u8>>(),
            ),
            ("period 2".into(), b"ab".repeat(scale)),
            (
                "blocks of 37".into(),
                (0..3 * scale).map(|i| ((i / 37) % 3) as u8).collect(),
            ),
            ("one value".into(), vec![7; scale / 2 + 3]),
            // a real BWT (one cycle: long segments, grows, a tail entered full)
            (
                "bwt of random 3".into(),
                bwt_of(&(0..2 * scale).map(|_| (r() % 3) as u8).collect::<Vec<u8>>()),
            ),
            (
                "bwt of text".into(),
                bwt_of(
                    &b"the cat sat on the mat; "
                        .repeat(scale / 12)
                        .iter()
                        .enumerate()
                        .map(|(i, &c)| if i % 97 == 0 { b'#' } else { c })
                        .collect::<Vec<u8>>(),
                ),
            ),
        ];
        for (name, ll) in columns(scale) {
            if ll.len() >= 64 {
                v.push((name, ll));
            }
        }
        v
    }

    /// The last column of `text`'s sorted rotations (its permutation one cycle when the
    /// text is primitive)
    fn bwt_of(text: &[u8]) -> Vec<u8> {
        let n = text.len();
        let mut rot: Vec<usize> = (0..n).collect();
        rot.sort_by(|&a, &b| {
            (0..n)
                .map(|i| text[(a + i) % n].cmp(&text[(b + i) % n]))
                .find(|o| o.is_ne())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        rot.iter().map(|&r| text[(r + n - 1) % n]).collect()
    }

    /// Runs every case of the sweep for mutation `MUT`: (cases, failures, first failure);
    /// with `seen`, this thread's transition counts are added to it
    fn staged_sweep<const L: usize, const MUT: u8>(
        scale: usize,
        segment_counts: &[usize],
        step: usize,
        seen: &mut [usize; staged_seen::N],
    ) -> (usize, usize, Option<String>) {
        let mut s = SegScratch::default();
        let (mut cases, mut failures, mut first) = (0, 0, None);
        staged_seen::take();
        // (under Miri, the crafted tables only: `columns`' add little there but time)
        let tables = staged_tables(scale);
        let keep = if cfg!(miri) { 6 } else { tables.len() };
        for (name, ll) in tables.into_iter().take(keep) {
            let counts = counts_of(&ll);
            let mut tt = TVector::new();
            crate::bwt::build(&ll, &counts, &mut tt);
            let n = ll.len() as u32;
            for orig_ptr in [0, n / 2] {
                let mut want = Vec::new();
                crate::bwt::walk(&tt, orig_ptr, &mut want);
                for &segments in segment_counts {
                    // the unstaged team walk's output, the same starts (one walker)
                    let mut old = Vec::new();
                    {
                        let w = RoomWalk::new(&mut tt, orig_ptr, &mut s, segments, false);
                        w.share::<L>();
                        let (_, s2) = w.finish();
                        s2.assemble(&mut old);
                    }
                    let ns = s.starts.len();
                    let mut budgets: Vec<Option<usize>> =
                        (0..=ns + 1).step_by(step).map(Some).collect();
                    budgets.extend([Some(ns), None]);
                    for (bi, &budget) in budgets.iter().enumerate() {
                        let second_staged = bi % 2 == 0;
                        cases += 1;
                        if let Err(e) = staged_case::<L, MUT>(
                            &mut tt,
                            orig_ptr,
                            segments,
                            budget,
                            second_staged,
                            &want,
                            &old,
                            &mut s,
                        ) {
                            failures += 1;
                            first.get_or_insert(format!(
                                "{name}, orig_ptr {orig_ptr}, segments {segments}, budget {budget:?}: {e}"
                            ));
                        }
                    }
                }
            }
        }
        for (a, b) in seen.iter_mut().zip(staged_seen::take()) {
            *a += b;
        }
        (cases, failures, first)
    }

    /// Every transition of the staged walk, forced and observed, each case exact
    #[test]
    fn staged_transitions() {
        let mut seen = [0; staged_seen::N];
        // short segments at 8 lanes (many takes, exhaustion everywhere), and long ones
        // (passes over several chunks, grows, a tail entered full)
        let (cases, failures, first) = staged_sweep::<8, 0>(3_000, &[3, 24, 96, 400], 1, &mut seen);
        assert_eq!(failures, 0, "{first:?}");
        let (c2, f2, first2) = staged_sweep::<32, 0>(3_000, &[40, 300], 1, &mut seen);
        assert_eq!(f2, 0, "{first2:?}");
        eprintln!("staged transitions: {} cases", cases + c2);
        for (i, name) in staged_seen::NAMES.iter().enumerate() {
            eprintln!("  {name}: {}", seen[i]);
        }
        for (i, name) in staged_seen::NAMES.iter().enumerate() {
            assert!(seen[i] > 0, "never reached: {name}");
        }
    }

    /// The negative controls: each deliberately broken transition (`mutation`) fails the
    /// same sweep, by a wrong segment, successor or output, or a panic before any write
    /// past a buffer. The unstaged walk's exhaustion rule ("the lanes after k have not
    /// stepped") among them.
    #[test]
    fn staged_mutations_are_caught() {
        fn caught<const MUT: u8>(what: &str) {
            let mut seen = [0; staged_seen::N];
            let (cases, f8, _) = staged_sweep::<8, MUT>(3_000, &[3, 24, 96, 400], 1, &mut seen);
            let (_, f32, _) = staged_sweep::<32, MUT>(3_000, &[40, 300], 1, &mut seen);
            eprintln!("mutation {what}: {} of {cases}+ cases failed", f8 + f32);
            assert!(f8 + f32 > 0, "mutation {what} was not caught");
        }
        caught::<{ mutation::OLD_RULE }>("OLD_RULE");
        caught::<{ mutation::ST_AT_J }>("ST_AT_J");
        caught::<{ mutation::FINISH_BEFORE_FLUSH }>("FINISH_BEFORE_FLUSH");
        caught::<{ mutation::WAITING_AS_STEPPED }>("WAITING_AS_STEPPED");
        caught::<{ mutation::NO_ROWS_BACK }>("NO_ROWS_BACK");
        caught::<{ mutation::NO_EXHAUST_FLUSH }>("NO_EXHAUST_FLUSH");
        caught::<{ mutation::NO_TAIL_GROW }>("NO_TAIL_GROW");
        assert_eq!(mutation::ALL.len(), 7);
    }

    /// The staged walk under Miri, small tables, a coarse budget sweep: its unchecked tile
    /// writes and flushes (and, with the threads of `miri_walk_room_handoffs` and
    /// `miri_split_build_and_walk`, its buffers' one writer each)
    #[test]
    fn miri_staged_transitions() {
        let mut seen = [0; staged_seen::N];
        let scale = if cfg!(miri) { 240 } else { 3_000 };
        let step = if cfg!(miri) { 9 } else { 1 };
        let (_, failures, first) = staged_sweep::<8, 0>(scale, &[3, 24, 96], step, &mut seen);
        assert_eq!(failures, 0, "{first:?}");
        for name in [
            "MID_PASS_TAKES",
            "EXHAUSTED_MID_PASS",
            "LATER_STEPPED",
            "GROWS",
        ] {
            let i = staged_seen::NAMES.iter().position(|n| *n == name).unwrap();
            assert!(seen[i] > 0, "never reached: {name}");
        }
    }

    /// The staged walk on real blocks: every block of each file in `MIBZ2_STAGED_CORPORA`
    /// (colon-separated .bz2 paths) at least `MIN_COOP` rows, at the decoder's lanes and the
    /// pipelined team's segment count, forced exhaustion at a spread of claims, then two
    /// and three walkers on threads; every segment, successor and output exact.
    /// `MIBZ2_STAGED_CORPORA=a.bz2:b.bz2 cargo test --release --lib staged_on_corpora -- --ignored`
    #[test]
    #[ignore]
    fn staged_on_corpora() {
        use crate::stream::{Event, Scanner, SliceSource};
        let Ok(paths) = std::env::var("MIBZ2_STAGED_CORPORA") else {
            eprintln!("MIBZ2_STAGED_CORPORA unset: skipped");
            return;
        };
        const L: usize = crate::bwt::FLAGGED_LANES;
        let k = crate::Kernels::detect();
        let mut seen = [0; staged_seen::N];
        let mut s = SegScratch::default();
        let (mut blocks, mut cases) = (0, 0);
        staged_seen::take();
        for path in paths.split(':') {
            let data = std::fs::read(path).unwrap();
            let mut sc = Scanner::new(SliceSource(&data));
            let mut syms = Vec::new();
            let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
            while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap()
            {
                let n = crate::mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
                let orig_ptr = b.orig_ptr;
                syms = b.syms;
                if n < crate::bwt::MIN_COOP {
                    continue;
                }
                blocks += 1;
                let mut tt = TVector::new();
                crate::bwt::build(&ll[..n], &counts, &mut tt);
                let mut want = Vec::new();
                crate::bwt::walk(&tt, orig_ptr, &mut want);
                for segments in [1024, 1536] {
                    let mut old = Vec::new();
                    {
                        let w = RoomWalk::new(&mut tt, orig_ptr, &mut s, segments, false);
                        w.share::<L>();
                        let (_, s2) = w.finish();
                        s2.assemble(&mut old);
                    }
                    assert!(old == want, "{path}: the unstaged walk");
                    let ns = s.starts.len();
                    for (bi, budget) in [0, 1, L - 1, L, L + 1, ns / 3, ns / 2, ns - 1, ns]
                        .into_iter()
                        .map(Some)
                        .chain([None])
                        .enumerate()
                    {
                        cases += 1;
                        staged_case::<L, 0>(
                            &mut tt,
                            orig_ptr,
                            segments,
                            budget,
                            bi % 2 == 0,
                            &want,
                            &old,
                            &mut s,
                        )
                        .unwrap_or_else(|e| {
                            panic!("{path}, block {blocks}, budget {budget:?}: {e}")
                        });
                    }
                    for walkers in [2, 3] {
                        let w = RoomWalk::new(&mut tt, orig_ptr, &mut s, segments, true);
                        thread::scope(|sc| {
                            for _ in 0..walkers {
                                let w = &w;
                                sc.spawn(move || w.share::<L>());
                            }
                        });
                        let (_, s2) = w.finish();
                        let mut out = Vec::new();
                        s2.assemble(&mut out);
                        assert!(out == want, "{path}: {walkers} staged walkers");
                        cases += 1;
                    }
                }
            }
        }
        for (a, b) in seen.iter_mut().zip(staged_seen::take()) {
            *a += b;
        }
        eprintln!("staged on corpora: {blocks} blocks, {cases} cases, every one exact");
        for (i, name) in staged_seen::NAMES.iter().enumerate() {
            eprintln!("  {name}: {}", seen[i]);
        }
        assert!(blocks > 0);
    }

    // ---- The `walk-stats` feature's records, reconciled per block ----

    /// A number field of a record line
    #[cfg(feature = "walk-stats")]
    fn num(line: &str, key: &str) -> Option<u64> {
        let at = line.find(&format!("\"{key}\":"))? + key.len() + 3;
        let rest = &line[at..];
        let end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        rest[..end].parse().ok()
    }

    #[cfg(feature = "walk-stats")]
    fn text<'l>(line: &'l str, key: &str) -> Option<&'l str> {
        let at = line.find(&format!("\"{key}\":\""))? + key.len() + 4;
        let rest = &line[at..];
        Some(&rest[..rest.find('"')?])
    }

    /// One block's records reconciled, as the native diagnostic's checker reconciles them
    /// (docs/journal/2026-11-10-staged-engagement-diagnostic-draft.md §3): the walker calls
    /// are exactly the summary's entered set (a lost record, even a zero-work one, is an
    /// error), the leader's iff it joined, the helpers' shares exactly the built set; the
    /// calls' takes are the starts and their bytes the segments' bytes, one exhausting claim
    /// each, every call by `walker`
    #[cfg(feature = "walk-stats")]
    fn reconcile(recs: &[&str], walker: &str) -> Result<(), String> {
        let summaries: Vec<&&str> = recs
            .iter()
            .filter(|l| text(l, "kind") == Some("summary"))
            .collect();
        let [sum] = summaries[..] else {
            return Err(format!("{} summaries", summaries.len()));
        };
        let entered = num(sum, "entered").unwrap();
        let built = num(sum, "built").unwrap();
        let joined = sum.contains("\"joined\":true");
        let (mut seen, mut shares) = (0u64, 0u64);
        let (mut takes, mut bytes) = (0, 0);
        for l in recs {
            let bit = 1u64 << num(l, "thread").unwrap_or(63);
            match text(l, "kind") {
                Some("call") => {
                    if seen & bit != 0 {
                        return Err("a duplicate call".into());
                    }
                    seen |= bit;
                    takes += num(l, "takes").unwrap();
                    bytes += num(l, "bytes").unwrap();
                    if num(l, "exhausts") != Some(1) {
                        return Err(format!("exhausts in {l}"));
                    }
                    if text(l, "walker") != Some(walker) {
                        return Err(format!("walker in {l}"));
                    }
                }
                Some("share") => shares |= bit,
                _ => {}
            }
        }
        if seen != entered {
            return Err(format!("calls {seen:b}, entered {entered:b}"));
        }
        if (seen & 1 != 0) != joined {
            return Err(format!("leader call {}, joined {joined}", seen & 1));
        }
        if shares != built {
            return Err(format!("shares {shares:b}, built {built:b}"));
        }
        if takes != num(sum, "starts").unwrap() || bytes != num(sum, "seg_bytes").unwrap() {
            return Err(format!("takes {takes}, bytes {bytes}: {sum}"));
        }
        Ok(())
    }

    /// Pipelined runs with the feature on: the output is `walk`'s (as without it), and
    /// every block's records reconcile; then the reconciliation's negative controls (its
    /// mutants): a lost zero-work helper record, a lost leader record, a duplicate, a byte
    /// miscounted, a share lost, must each be caught
    #[cfg(feature = "walk-stats")]
    #[test]
    fn walk_stats_reconcile_per_block() {
        let team = walk_stats::team();
        let mut r = xorshift(37);
        let mut place = 0u64;
        let mut zero_work = 0;
        for k in [2, 3, 4] {
            let p = Pool::new(k).unwrap();
            for n in [3_000, 20_000] {
                let ll: Vec<u8> = (0..n).map(|_| (r() % 6) as u8).collect();
                let counts = counts_of(&ll);
                let cps = checkpoints_of(&ll, CP);
                let mut want_tt = TVector::new();
                crate::bwt::build(&ll, &counts, &mut want_tt);
                let mut want = Vec::new();
                crate::bwt::walk(&want_tt, 11, &mut want);
                for staged in [false, true] {
                    for late in [false, true] {
                        let (mut tt, mut seg, mut out) =
                            (TVector::new(), SegScratch::default(), Vec::new());
                        let lead = || {
                            if late {
                                thread::sleep(std::time::Duration::from_millis(5));
                            }
                        };
                        invert_pipelined_with::<8, false, BARRIER_SPINS>(
                            &p, &ll, &counts, &cps, CP, 11, &mut tt, &mut seg, &mut out, &lead,
                            place, staged,
                        )
                        .unwrap();
                        assert!(out == want, "k {k}, n {n}, staged {staged}");
                        let all = walk_stats::of_team(team);
                        let tag = format!("\"block\":{place},");
                        let recs: Vec<&str> = all
                            .iter()
                            .filter(|l| l.contains(&tag))
                            .map(String::as_str)
                            .collect();
                        let walker = if staged {
                            "traverse_staged"
                        } else {
                            "traverse"
                        };
                        reconcile(&recs, walker)
                            .unwrap_or_else(|e| panic!("block {place}: {e}\n{recs:#?}"));
                        // the mutants, each a record set that must not reconcile
                        let calls: Vec<usize> = (0..recs.len())
                            .filter(|&i| text(recs[i], "kind") == Some("call"))
                            .collect();
                        for &i in &calls {
                            let mut m = recs.clone();
                            if num(m[i], "takes") == Some(0) {
                                zero_work += 1;
                            }
                            m.remove(i);
                            assert!(reconcile(&m, walker).is_err(), "a lost call record");
                            let mut m = recs.clone();
                            m.push(recs[i]);
                            assert!(reconcile(&m, walker).is_err(), "a duplicate");
                            let bumped = recs[i].replacen("\"bytes\":", "\"bytes\":1", 1);
                            let mut m = recs.clone();
                            m[i] = &bumped;
                            assert!(reconcile(&m, walker).is_err(), "a byte miscounted");
                        }
                        let other = if staged {
                            "traverse"
                        } else {
                            "traverse_staged"
                        };
                        assert!(reconcile(&recs, other).is_err(), "the wrong walker");
                        if let Some(i) = recs.iter().position(|l| text(l, "kind") == Some("share"))
                        {
                            let mut m = recs.clone();
                            m.remove(i);
                            assert!(reconcile(&m, walker).is_err(), "a lost share");
                        }
                        place += 1;
                    }
                }
            }
        }
        // a zero-work call (one claim, the queue already exhausted) occurred and its loss
        // was caught: the Researcher's counterexample (#145 6760210997)
        assert!(zero_work > 0, "no zero-work call was seen");
    }
}
