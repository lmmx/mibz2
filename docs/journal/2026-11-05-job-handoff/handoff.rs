//! The job handoff alone, both ways, in the decoder's shape (no timing: counts only).
//! A scanner makes `jobs` jobs of `scan` work units each, waiting for a slot (at most
//! `slots` between it and the reader); `threads` workers do `work` units a job and put it
//! in `ready` (notify_all on the shared condvar, as parallel.rs does); a reader takes them
//! in order (notify_all). `old`: the jobs through Arc<Mutex<mpsc::Receiver>>, the lock held
//! across recv (master d1fd4405). `new`: the JobQueue of perf/job-handoff (copied).
//! Prints the process's voluntary and involuntary context switches per job (getrusage),
//! and, with `--ir`, nothing else (for callgrind: inclusive Ir of take/give per job).
use std::collections::{BTreeMap, VecDeque};
use std::hint::black_box;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

#[repr(C)]
struct Rusage([i64; 18]);
extern "C" {
    fn getrusage(who: i32, usage: *mut Rusage) -> i32;
}
fn csw() -> (i64, i64) {
    let mut r = Rusage([0; 18]);
    unsafe { getrusage(0, &mut r) };
    (r.0[16], r.0[17])
}
/// this thread's (RUSAGE_THREAD)
fn tcsw() -> (i64, i64) {
    let mut r = Rusage([0; 18]);
    unsafe { getrusage(1, &mut r) };
    (r.0[16], r.0[17])
}
static WV: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
static WI: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
static SV: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
fn add_worker() {
    let (v, i) = tcsw();
    WV.fetch_add(v, std::sync::atomic::Ordering::Relaxed);
    WI.fetch_add(i, std::sync::atomic::Ordering::Relaxed);
}

#[inline(never)]
fn spin(units: u64) -> u64 {
    let mut x = 0x9e37_79b9_7f4a_7c15u64 ^ units;
    for _ in 0..units {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
    }
    black_box(x)
}

type Job = (u64, [u64; 38]); // ~ the decoder's Job: 8 + 8 + Symbols (304 bytes)

// ---- new: the patch's JobQueue, marks removed ----
struct JobQueue<T> {
    state: Mutex<Queued<T>>,
    cv: Condvar,
}
struct Queued<T> {
    jobs: VecDeque<T>,
    waiting: usize,
    receivers: usize,
    closed: bool,
}
impl<T> JobQueue<T> {
    fn lock(&self) -> MutexGuard<'_, Queued<T>> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}
struct JobTx<T>(Arc<JobQueue<T>>);
struct JobRx<T>(Arc<JobQueue<T>>);
fn job_queue<T>() -> (JobTx<T>, JobRx<T>) {
    let q = Arc::new(JobQueue {
        state: Mutex::new(Queued {
            jobs: VecDeque::new(),
            waiting: 0,
            receivers: 1,
            closed: false,
        }),
        cv: Condvar::new(),
    });
    (JobTx(q.clone()), JobRx(q))
}
impl<T> JobTx<T> {
    fn send(&self, job: T) -> Result<(), T> {
        let mut q = self.0.lock();
        if q.receivers == 0 {
            return Err(job);
        }
        q.jobs.push_back(job);
        let wake = q.waiting > 0;
        drop(q);
        if wake {
            self.0.cv.notify_one();
        }
        Ok(())
    }
}
impl<T> Drop for JobTx<T> {
    fn drop(&mut self) {
        self.0.lock().closed = true;
        self.0.cv.notify_all();
    }
}
impl<T> JobRx<T> {
    fn recv(&self) -> Option<T> {
        let mut q = self.0.lock();
        loop {
            if let Some(job) = q.jobs.pop_front() {
                return Some(job);
            }
            if q.closed {
                return None;
            }
            q.waiting += 1;
            q = self.0.cv.wait(q).unwrap_or_else(|e| e.into_inner());
            q.waiting -= 1;
        }
    }
}
impl<T> JobRx<T> {
    fn recv_spin(&self) -> Option<T> {
        let mut q = self.0.lock();
        let mut step = 0u32;
        loop {
            if let Some(job) = q.jobs.pop_front() {
                return Some(job);
            }
            if q.closed {
                return None;
            }
            if step <= 10 {
                drop(q);
                if step <= 6 {
                    for _ in 0..1u32 << step {
                        std::hint::spin_loop();
                    }
                } else {
                    std::thread::yield_now();
                }
                step += 1;
                q = self.0.lock();
                continue;
            }
            q.waiting += 1;
            q = self.0.cv.wait(q).unwrap_or_else(|e| e.into_inner());
            q.waiting -= 1;
        }
    }
}
impl<T> Clone for JobRx<T> {
    fn clone(&self) -> Self {
        self.0.lock().receivers += 1;
        JobRx(self.0.clone())
    }
}
impl<T> Drop for JobRx<T> {
    fn drop(&mut self) {
        self.0.lock().receivers -= 1;
    }
}

// ---- the two handoffs, kept out of line for callgrind ----
#[cfg(not(count))]
#[inline(never)]
fn old_take(rx: &Mutex<mpsc::Receiver<Job>>) -> Option<Job> {
    let rx = rx.lock().unwrap_or_else(|e| e.into_inner());
    rx.recv().ok()
}
#[cfg(not(count))]
#[inline(never)]
fn old_give(tx: &mpsc::Sender<Job>, j: Job) -> bool {
    tx.send(j).is_ok()
}
#[cfg(not(count))]
#[inline(never)]
fn new_take(rx: &JobRx<Job>) -> Option<Job> {
    rx.recv()
}
#[cfg(not(count))]
#[inline(never)]
fn new_give(tx: &JobTx<Job>, j: Job) -> bool {
    tx.send(j).is_ok()
}

// ---- with --cfg count: the same handoffs, counting events (not timing) ----
use std::sync::atomic::{AtomicI64, Ordering::SeqCst};
/// jobs sent and not yet taken; takes; takes begun with a job queued; of those, the lock
/// not free (held: old, by a worker asleep in recv); takes that slept for their job
static Q: AtomicI64 = AtomicI64::new(0);
static ARR: AtomicI64 = AtomicI64::new(0);
static ARRQ: AtomicI64 = AtomicI64::new(0);
static BLK: AtomicI64 = AtomicI64::new(0);
static SLEPT: AtomicI64 = AtomicI64::new(0);
#[cfg(count)]
fn old_take(rx: &Mutex<mpsc::Receiver<Job>>) -> Option<Job> {
    ARR.fetch_add(1, SeqCst);
    let q = Q.load(SeqCst) > 0;
    if q {
        ARRQ.fetch_add(1, SeqCst);
    }
    let g = match rx.try_lock() {
        Ok(g) => g,
        Err(_) => {
            if q {
                BLK.fetch_add(1, SeqCst);
            }
            rx.lock().unwrap()
        }
    };
    let r = match g.try_recv() {
        Ok(j) => Some(j),
        Err(mpsc::TryRecvError::Empty) => {
            SLEPT.fetch_add(1, SeqCst);
            g.recv().ok()
        }
        Err(_) => None,
    };
    if r.is_some() {
        Q.fetch_sub(1, SeqCst);
    }
    r
}
#[cfg(count)]
fn old_give(tx: &mpsc::Sender<Job>, j: Job) -> bool {
    Q.fetch_add(1, SeqCst);
    tx.send(j).is_ok()
}
static SPIN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(not(count))]
#[inline(never)]
fn spin_take(rx: &JobRx<Job>) -> Option<Job> {
    rx.recv_spin()
}
#[cfg(count)]
fn spin_take(rx: &JobRx<Job>) -> Option<Job> {
    ARR.fetch_add(1, SeqCst);
    let qd = Q.load(SeqCst) > 0;
    if qd {
        ARRQ.fetch_add(1, SeqCst);
    }
    if rx.0.state.try_lock().is_err() && qd {
        BLK.fetch_add(1, SeqCst);
    }
    let r = rx.recv_spin();
    if r.is_some() {
        Q.fetch_sub(1, SeqCst);
    }
    r
}
#[cfg(count)]
fn new_take(rx: &JobRx<Job>) -> Option<Job> {
    ARR.fetch_add(1, SeqCst);
    let qd = Q.load(SeqCst) > 0;
    if qd {
        ARRQ.fetch_add(1, SeqCst);
    }
    let mut q = match rx.0.state.try_lock() {
        Ok(g) => g,
        Err(_) => {
            if qd {
                BLK.fetch_add(1, SeqCst);
            }
            rx.0.lock()
        }
    };
    let mut slept = false;
    let r = loop {
        if let Some(job) = q.jobs.pop_front() {
            break Some(job);
        }
        if q.closed {
            break None;
        }
        if !slept {
            slept = true;
            SLEPT.fetch_add(1, SeqCst);
        }
        q.waiting += 1;
        q = rx.0.cv.wait(q).unwrap_or_else(|e| e.into_inner());
        q.waiting -= 1;
    };
    if r.is_some() {
        Q.fetch_sub(1, SeqCst);
    }
    r
}
#[cfg(count)]
fn new_give(tx: &JobTx<Job>, j: Job) -> bool {
    Q.fetch_add(1, SeqCst);
    tx.send(j).is_ok()
}

struct State {
    issued: u64,
    taken: u64,
    ready: BTreeMap<u64, u64>,
    end: Option<u64>,
}
struct Shared {
    st: Mutex<State>,
    cv: Condvar,
    slots: u64,
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mode = a[1].clone();
    let threads: usize = a[2].parse().unwrap();
    let jobs: u64 = a[3].parse().unwrap();
    let scan: u64 = a[4].parse().unwrap();
    let work: u64 = a[5].parse().unwrap();
    let slots: u64 = a[6].parse().unwrap();
    let ir = a.iter().any(|s| s == "--ir");
    // the share of short jobs (the corpus: ~200 of 431 blocks short), a 1/1000th of the work
    // and a 1/100th of the scan each
    let short: u64 = a.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
    let is_short = move |i: u64| (i.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 33) % 100 < short;
    let sh = Arc::new(Shared {
        st: Mutex::new(State {
            issued: 0,
            taken: 0,
            ready: BTreeMap::new(),
            end: None,
        }),
        cv: Condvar::new(),
        slots,
    });
    // (job i's work varies 0.5x-1.5x, deterministic)
    let work_of = move |i: u64| {
        let w = work / 2 + (i.wrapping_mul(0x9e37_79b9) % (work.max(1)));
        if is_short(i) { w / 1000 } else { w }
    };
    SPIN.store(mode == "spin", std::sync::atomic::Ordering::Relaxed);
    let before = csw();
    let mut hs = Vec::new();
    let finish_worker = move |sh: &Shared, place: u64, v: u64| {
        let mut st = sh.st.lock().unwrap();
        st.ready.insert(place, v);
        drop(st);
        sh.cv.notify_all();
    };
    let scanner_loop = move |sh: &Shared, give: &mut dyn FnMut(Job) -> bool| {
        for i in 0..jobs {
            {
                let mut st = sh.st.lock().unwrap();
                while st.issued - st.taken >= sh.slots {
                    st = sh.cv.wait(st).unwrap();
                }
            }
            spin(if is_short(i) { scan / 100 } else { scan });
            {
                let mut st = sh.st.lock().unwrap();
                st.issued += 1;
            }
            if !give((i, [i; 38])) {
                break;
            }
        }
        let mut st = sh.st.lock().unwrap();
        st.end = Some(st.issued);
        drop(st);
        sh.cv.notify_all();
        SV.store(tcsw().0, std::sync::atomic::Ordering::Relaxed);
    };
    if mode == "old" {
        let (tx, rx) = mpsc::channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));
        for _ in 0..threads {
            let (sh, rx) = (sh.clone(), rx.clone());
            hs.push(std::thread::spawn(move || {
                while let Some((place, p)) = old_take(&rx) {
                    let v = spin(work_of(place)) ^ p[37];
                    finish_worker(&sh, place, v);
                }
                add_worker();
            }));
        }
        let sh2 = sh.clone();
        hs.push(std::thread::spawn(move || {
            scanner_loop(&sh2, &mut |j| old_give(&tx, j));
        }));
    } else {
        let (tx, rx) = job_queue::<Job>();
        for _ in 0..threads {
            let (sh, rx) = (sh.clone(), rx.clone());
            hs.push(std::thread::spawn(move || {
                let spinning = SPIN.load(std::sync::atomic::Ordering::Relaxed);
                while let Some((place, p)) = if spinning { spin_take(&rx) } else { new_take(&rx) } {
                    let v = spin(work_of(place)) ^ p[37];
                    finish_worker(&sh, place, v);
                }
                add_worker();
            }));
        }
        drop(rx);
        let sh2 = sh.clone();
        hs.push(std::thread::spawn(move || {
            scanner_loop(&sh2, &mut |j| new_give(&tx, j));
        }));
    }
    // the reader, in order
    let mut sum = 0u64;
    loop {
        let mut st = sh.st.lock().unwrap();
        loop {
            let t = st.taken;
            if let Some(v) = st.ready.remove(&t) {
                st.taken += 1;
                sh.cv.notify_all();
                sum ^= v;
                break;
            }
            if st.end == Some(st.taken) {
                break;
            }
            st = sh.cv.wait(st).unwrap();
        }
        if st.end == Some(st.taken) {
            break;
        }
    }
    for h in hs {
        h.join().unwrap();
    }
    let after = csw();
    let (v, iv) = (after.0 - before.0, after.1 - before.1);
    if !ir {
        println!(
            "{mode} t{threads} n{jobs} scan {scan} work {work} slots {slots} short {short}%: process vol {:.3} invol {:.3} | workers vol {:.3} invol {:.3} | scanner vol {:.3} | reader vol {:.3}  (per job; {sum:x})",
            v as f64 / jobs as f64,
            iv as f64 / jobs as f64,
            WV.load(std::sync::atomic::Ordering::Relaxed) as f64 / jobs as f64,
            WI.load(std::sync::atomic::Ordering::Relaxed) as f64 / jobs as f64,
            SV.load(std::sync::atomic::Ordering::Relaxed) as f64 / jobs as f64,
            tcsw().0 as f64 / jobs as f64,
        );
        if cfg!(count) {
            let n = jobs as f64;
            println!(
                "  per job: takes begun with a job queued {:.3}, of them blocked on the lock {:.3}; takes that slept {:.3}",
                ARRQ.load(SeqCst) as f64 / n,
                BLK.load(SeqCst) as f64 / n,
                SLEPT.load(SeqCst) as f64 / n
            );
        }
    } else {
        println!("{sum:x}");
    }
}
