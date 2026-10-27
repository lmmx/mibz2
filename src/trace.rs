//! The parallel decoder's own timeline (feature `trace`), for the occupancy instrument
//! (examples/occupancy.rs, #39): marks taken inside parallel.rs's scanner, workers and
//! reader as they run, never a model of them.
//!
//! **Levels**, set at run time (`set_level`), so one build measures what each costs:
//! - 0, off: each mark is one relaxed atomic load;
//! - 1, the timeline: each mark a timestamp, its kind and its block's place, pushed to a
//!   buffer of the thread's own (no lock shared between threads while decoding);
//! - 2, full: also, at `decode_block`'s start and end only, the thread's CPU time and (on
//!   Linux) its scheduler counters (`/proc/thread-self/schedstat`: time on a CPU, time
//!   runnable but waiting for one, timeslices), so a block's elapsed time, CPU time and
//!   run-queue wait cover the same interval.
//!
//! A thread's buffer goes to a shared list when the thread ends, and `take` collects them
//! (the calling thread's too) once the decoder is dropped.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// A mark's kind: where on its thread's path the decoder was
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// the thread's first mark of a run, and its last (`take`, or the thread's end)
    ThreadStart,
    ThreadEnd,
    /// the scanner: waiting for compressed input (its channel's `recv`), and once it has
    /// a chunk, or the input's end (the feeder done, the channel closed)
    InputWait,
    InputGot,
    InputEnd,
    /// the scanner: before it looks for a slot, and once it has one
    SlotWait,
    SlotGot,
    /// the scanner: a block read (its Huffman-coded symbols) and given its place (counted
    /// in the window from here), then sent to the workers' channel
    Issued,
    Enqueued,
    /// a worker: before it takes the jobs' lock, once it holds it (waiting in `recv`), and
    /// once it has a job
    JobLock,
    JobRecv,
    JobGot,
    /// a worker: `decode_block`'s start (its output buffer taken) and end
    DecodeStart,
    DecodeEnd,
    /// a worker: its block (or error) in `ready`
    Ready,
    /// the reader: before it looks for the next block in order, and once it has taken it
    ReadWait,
    Taken,
}

/// No block: the place of a mark that has none
pub const NONE: u64 = u64::MAX;
/// A clock not read (below level 2, or not on this platform)
pub const UNREAD: u64 = u64::MAX;

#[derive(Clone, Copy, Debug)]
pub struct Mark {
    /// nanoseconds since `set_level` last turned the trace on
    pub ns: u64,
    pub kind: Kind,
    pub place: u64,
    /// the thread's CPU time, and its schedstat fields, at the mark: level 2 only, at
    /// `ThreadStart`, `ThreadEnd`, `DecodeStart` and `DecodeEnd` (else `UNREAD`)
    pub cpu_ns: u64,
    pub run_ns: u64,
    pub wait_ns: u64,
    pub slices: u64,
}

/// One thread's marks, in its order
pub struct ThreadTrace {
    pub name: String,
    pub marks: Vec<Mark>,
}

static LEVEL: AtomicU8 = AtomicU8::new(0);
static EPOCH: OnceLock<Mutex<Instant>> = OnceLock::new();
static DONE: Mutex<Vec<ThreadTrace>> = Mutex::new(Vec::new());

/// Set the level (0, 1 or 2); turning it on restarts the clock and drops marks not taken
pub fn set_level(level: u8) {
    if level > 0 {
        *epoch().lock().unwrap_or_else(|e| e.into_inner()) = Instant::now();
        DONE.lock().unwrap_or_else(|e| e.into_inner()).clear();
        BUF.with(|b| b.borrow_mut().take());
    }
    LEVEL.store(level.min(2), Ordering::Release);
}

pub fn level() -> u8 {
    LEVEL.load(Ordering::Relaxed)
}

/// Every thread's marks since the trace was turned on: call once the decoder is dropped
/// (its threads joined), from the thread that read it
pub fn take() -> Vec<ThreadTrace> {
    BUF.with(|b| {
        if let Some(buf) = b.borrow_mut().take() {
            drop(buf);
        }
    });
    std::mem::take(&mut *DONE.lock().unwrap_or_else(|e| e.into_inner()))
}

/// The threads whose marks are collected so far (each when it ends): the decoder's
/// threads end once it is dropped, not necessarily before `drop` returns (its last
/// reference can be another thread's), so a caller waits for the count it expects
pub fn finished() -> usize {
    DONE.lock().unwrap_or_else(|e| e.into_inner()).len()
}

fn epoch() -> &'static Mutex<Instant> {
    EPOCH.get_or_init(|| Mutex::new(Instant::now()))
}

struct Buf {
    name: String,
    marks: Vec<Mark>,
    start: Instant,
    level: u8,
    #[cfg(target_os = "linux")]
    sched: Option<std::fs::File>,
}

impl Buf {
    fn new(level: u8) -> Buf {
        let mut b = Buf {
            name: std::thread::current().name().unwrap_or("?").to_string(),
            marks: Vec::with_capacity(4096),
            start: *epoch().lock().unwrap_or_else(|e| e.into_inner()),
            level,
            #[cfg(target_os = "linux")]
            sched: if level >= 2 {
                std::fs::File::open("/proc/thread-self/schedstat").ok()
            } else {
                None
            },
        };
        b.push(Kind::ThreadStart, NONE, true);
        b
    }

    #[inline]
    fn push(&mut self, kind: Kind, place: u64, clocks: bool) {
        let ns = self.start.elapsed().as_nanos() as u64;
        let (mut cpu_ns, mut run_ns, mut wait_ns, mut slices) = (UNREAD, UNREAD, UNREAD, UNREAD);
        if clocks && self.level >= 2 {
            cpu_ns = thread_cpu_ns();
            #[cfg(target_os = "linux")]
            if let Some(f) = &self.sched {
                (run_ns, wait_ns, slices) = schedstat(f);
            }
        }
        self.marks.push(Mark {
            ns,
            kind,
            place,
            cpu_ns,
            run_ns,
            wait_ns,
            slices,
        });
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        self.push(Kind::ThreadEnd, NONE, true);
        let t = ThreadTrace {
            name: std::mem::take(&mut self.name),
            marks: std::mem::take(&mut self.marks),
        };
        DONE.lock().unwrap_or_else(|e| e.into_inner()).push(t);
    }
}

thread_local! {
    static BUF: RefCell<Option<Buf>> = const { RefCell::new(None) };
}

/// A mark on this thread's timeline (nothing at level 0); `DecodeStart` and `DecodeEnd`
/// also read the clocks at level 2
#[inline]
pub fn mark(kind: Kind, place: u64) {
    let level = LEVEL.load(Ordering::Relaxed);
    if level == 0 {
        return;
    }
    let clocks = matches!(kind, Kind::DecodeStart | Kind::DecodeEnd);
    // (a thread ending while its buffer is dropped marks nothing more)
    let _ = BUF.try_with(|b| {
        b.borrow_mut()
            .get_or_insert_with(|| Buf::new(level))
            .push(kind, place, clocks)
    });
}

/// The calling thread's CPU time, in nanoseconds (`UNREAD` where there is no such clock)
pub fn thread_cpu_ns() -> u64 {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        #[repr(C)]
        struct Timespec {
            sec: i64,
            nsec: i64,
        }
        extern "C" {
            fn clock_gettime(clock: i32, tp: *mut Timespec) -> i32;
        }
        #[cfg(target_os = "linux")]
        const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
        #[cfg(target_os = "macos")]
        const CLOCK_THREAD_CPUTIME_ID: i32 = 16;
        let mut t = Timespec { sec: 0, nsec: 0 };
        // SAFETY: a plain out-parameter of the size the C library writes (struct timespec,
        // LP64, as on every 64-bit Linux and macOS)
        if unsafe { clock_gettime(CLOCK_THREAD_CPUTIME_ID, &mut t) } == 0 {
            return t.sec as u64 * 1_000_000_000 + t.nsec as u64;
        }
    }
    UNREAD
}

/// A thread's `schedstat`: nanoseconds on a CPU, nanoseconds runnable but waiting for one,
/// and timeslices (each `UNREAD` if it can't be read)
#[cfg(target_os = "linux")]
pub fn schedstat(f: &std::fs::File) -> (u64, u64, u64) {
    use std::os::unix::fs::FileExt;
    let mut buf = [0u8; 96];
    let Ok(n) = f.read_at(&mut buf, 0) else {
        return (UNREAD, UNREAD, UNREAD);
    };
    let mut it = std::str::from_utf8(&buf[..n])
        .unwrap_or("")
        .split_ascii_whitespace()
        .map(|x| x.parse::<u64>().unwrap_or(UNREAD));
    (
        it.next().unwrap_or(UNREAD),
        it.next().unwrap_or(UNREAD),
        it.next().unwrap_or(UNREAD),
    )
}
