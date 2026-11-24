//! Research only (`character`): the decoder's character across worker counts, as a network
//! of activities, buffers and resources (docs/journal/2026-11-24-algorithm-level/briefs/
//! character.md; bench-character, examples/character.rs). One path only: the lazy workers
//! (`BzReader::parallel_workers`, lazy: `OpenCells`, a window of 4 blocks a worker).
//!
//! **Off** (the default): each hook is one relaxed atomic load.
//!
//! **On** ([`set_on`]), each thread keeps its own ledger of events (no lock shared between
//! threads while decoding), handed to a shared list when the thread ends and collected by
//! [`take`]:
//! - **an activity's begin and end** ([`begin`], [`end`]): the brief's tags A0 to A10 and the
//!   waits. A thread's activities nest as a stack, so its timeline is partitioned: at every
//!   instant it is in the activity on top of its stack, or in none (its residual);
//! - **a point** ([`point`]): a block issued, enqueued, taken by a worker, routed, published
//!   ready and taken by the reader, with its place in the input;
//! - **the clocks:** each event's time (`Instant`, from the epoch set when turned on). The
//!   thread's CPU time is read only at the waits' begins and ends and at the thread's start
//!   and end (a system call, too dear for every activity), so each stretch between waits has
//!   its wall time and its CPU time. At the thread's start and end, its scheduler counters
//!   (`/proc/thread-self/schedstat`: time on a CPU, runnable but waiting, timeslices) and its
//!   migrations (`/proc/thread-self/sched`, where the kernel gives them).
//!
//! **The buffers' occupancy** is also kept live ([`occ_point`]), apart from the ledgers: each
//! buffer's count, its integral over time and its high-water mark, each change timed under
//! one lock of its own. The integral is the left side of Little's law, checked against the
//! blocks' residence intervals from the ledgers.
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// An event's tag: an activity (begun and ended) or a point
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tag {
    // the feeder ("mibz2-read")
    /// A0: a chunk's buffer taken and the input read into it
    A0 = 0,
    /// waiting to hand a chunk to the scanner (2 queued)
    FeedWait = 1,
    // the scanner ("mibz2-scan")
    /// A1: waiting for compressed input
    InWait = 2,
    /// A1: the window's maintenance (swap, compaction, append, the chunk returned)
    InMaint = 3,
    /// A2: header, bitmap, selectors, code lengths and tables
    A2 = 4,
    /// A3: the symbols' buffer prepared and the Huffman decode
    A3 = 5,
    /// waiting for admission (`issued − taken` at the window, or the ready bytes at the cap)
    SlotWait = 6,
    /// A4: the block given its place and sent to the job queue
    Issue = 7,
    // a worker ("mibz2-<i>")
    /// waiting for a job (the queue empty)
    JobWait = 8,
    /// A5: MTF/RLE2 and the run records (`runs::front`, the run route's test), or the column
    /// route's MTF
    A5 = 9,
    /// A4: the output buffer from the pool (and the symbols' buffer returned)
    Pool = 10,
    /// A6a: Open's first pass, up to its admission
    A6a = 11,
    /// A6b: Open's set-up, induction and dictionary
    A6b = 12,
    /// A6c: Open's retained-edge wiring
    A6c = 13,
    /// A7: Open's emission (the walk)
    A7 = 14,
    /// A8: inversion apart from Open: the fallback (LazyCells' gate, then Segmented's build
    /// and walk), or the column route's
    A8 = 15,
    /// A9: RLE1 undone and the CRC (the tiled implementation, whole), or the column route's
    A9 = 16,
    /// A4: the block published to the ready set
    Publish = 17,
    // the reader (the caller's thread)
    /// A10: a read call, whole (the copy out, the pool return, the next block taken)
    Read = 18,
    /// waiting for the next block in order
    ReadWait = 19,
    // points
    /// the scanner: a block given its place (`x`: its symbols' bytes)
    Issued = 32,
    /// the scanner: in the job queue
    Enqueued = 33,
    /// a worker: taken from the job queue
    Got = 34,
    /// a worker: the block's route (`x`: [`route`]'s packing)
    Route = 35,
    /// a worker: published ready (`x`: its decoded bytes)
    Ready = 36,
    /// the reader: taken in order
    Taken = 37,
    /// the scanner, at a wait for admission: 1 if the ready bytes' cap holds it, 0 the window
    Bound = 38,
    /// the thread's first event and its last
    Start = 48,
    End = 49,
}

/// The activities and waits, in `Tag` order (their names in the ledgers)
pub const ACTIVITIES: [&str; 20] = [
    "A0",
    "feed_wait",
    "in_wait",
    "in_maint",
    "A2",
    "A3",
    "slot_wait",
    "issue",
    "job_wait",
    "A5",
    "pool",
    "A6a",
    "A6b",
    "A6c",
    "A7",
    "A8",
    "A9",
    "publish",
    "A10",
    "read_wait",
];

/// Whether an activity is a wait (its begin and end read the thread's CPU time)
pub fn is_wait(t: u8) -> bool {
    matches!(t, 1 | 2 | 6 | 8 | 19)
}

/// A block's routes (`Tag::Route`'s low byte)
pub mod route {
    /// run route, Open admitted outright (1/64 ≤ r/n < 0.15)
    pub const OPEN: u8 = 1;
    /// run route, Open attempted under its dense cap and admitted
    pub const OPEN_DENSE: u8 = 2;
    /// run route, Open attempted under its dense cap, refused after its first pass: the
    /// fallback
    pub const REFUSED: u8 = 3;
    /// run route, Open skipped (r/n < 1/64 or ≥ 0.28): the fallback
    pub const SKIPPED: u8 = 4;
    /// the column route
    pub const COLUMN: u8 = 5;
    /// The packing: the route, the block's rows (n) and its run records (r)
    pub fn pack(route: u8, n: usize, r: usize) -> u64 {
        u64::from(route) | (n as u64 & 0xFFF_FFFF) << 8 | (r as u64 & 0xFFF_FFFF) << 36
    }
    pub fn unpack(x: u64) -> (u8, u64, u64) {
        (x as u8, (x >> 8) & 0xFFF_FFFF, x >> 36)
    }
}

/// An event; `cpu` is the thread's CPU time (ns) where read, else `UNREAD`
#[derive(Clone, Copy, Debug)]
pub struct Ev {
    pub ns: u64,
    pub cpu: u64,
    pub x: u64,
    pub place: u32,
    pub tag: u8,
    /// 0 a point, 1 a begin, 2 an end
    pub edge: u8,
}

pub const UNREAD: u64 = u64::MAX;
/// A point's place where the thread's current block is meant (a worker's, from its `Got`)
pub const CURRENT: u32 = u32::MAX;

/// A thread's ledger, at its end
#[derive(Debug, Default)]
pub struct Ledger {
    pub name: String,
    pub events: Vec<Ev>,
    /// at its start and end: time on a CPU, runnable but waiting (ns), timeslices
    pub sched0: [u64; 3],
    pub sched1: [u64; 3],
    /// migrations at its start and end (`UNREAD` where the kernel does not give them)
    pub migrations0: u64,
    pub migrations1: u64,
}

static ON: AtomicBool = AtomicBool::new(false);
static EPOCH: OnceLock<Mutex<Instant>> = OnceLock::new();
static DONE: Mutex<Vec<Ledger>> = Mutex::new(Vec::new());

fn epoch() -> Instant {
    *EPOCH
        .get_or_init(|| Mutex::new(Instant::now()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Turn the ledgers on or off for the decodes started from now on; on restarts the clock,
/// and drops what was not taken (the occupancy too)
pub fn set_on(on: bool) {
    if on {
        *EPOCH
            .get_or_init(|| Mutex::new(Instant::now()))
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Instant::now();
        DONE.lock().unwrap_or_else(|e| e.into_inner()).clear();
        BUF.with(|b| b.borrow_mut().take());
        *OCC.lock().unwrap_or_else(|e| e.into_inner()) = [Occ::default(); OCCS];
    }
    ON.store(on, Ordering::SeqCst);
}

#[inline]
pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Every thread's ledger since turned on: call once the decoder is dropped and its threads
/// have ended ([`finished`]), from the thread that read it (its own ledger too)
pub fn take() -> Vec<Ledger> {
    BUF.with(|b| drop(b.borrow_mut().take()));
    std::mem::take(&mut *DONE.lock().unwrap_or_else(|e| e.into_inner()))
}

/// The ledgers collected so far (each thread's when it ends)
pub fn finished() -> usize {
    DONE.lock().unwrap_or_else(|e| e.into_inner()).len()
}

/// The calling thread's CPU time, ns
pub fn thread_cpu_ns() -> u64 {
    #[cfg(target_os = "linux")]
    {
        #[repr(C)]
        struct Timespec {
            sec: i64,
            nsec: i64,
        }
        extern "C" {
            fn clock_gettime(clock: i32, tp: *mut Timespec) -> i32;
        }
        let mut t = Timespec { sec: 0, nsec: 0 };
        // SAFETY: a plain out-parameter (CLOCK_THREAD_CPUTIME_ID = 3)
        if unsafe { clock_gettime(3, &mut t) } == 0 {
            return t.sec as u64 * 1_000_000_000 + t.nsec as u64;
        }
    }
    UNREAD
}

fn sched() -> [u64; 3] {
    let s = std::fs::read_to_string("/proc/thread-self/schedstat").unwrap_or_default();
    let mut it = s
        .split_ascii_whitespace()
        .map(|x| x.parse::<u64>().unwrap_or(UNREAD));
    [
        it.next().unwrap_or(UNREAD),
        it.next().unwrap_or(UNREAD),
        it.next().unwrap_or(UNREAD),
    ]
}

fn migrations() -> u64 {
    std::fs::read_to_string("/proc/thread-self/sched")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.trim_start().starts_with("se.nr_migrations"))
                .and_then(|l| l.rsplit(':').next())
                .and_then(|v| v.trim().parse().ok())
        })
        .unwrap_or(UNREAD)
}

struct Buf {
    led: Ledger,
    start: Instant,
    /// a worker's current block (from its `Got`)
    cur: u32,
}

impl Buf {
    fn new() -> Buf {
        let mut b = Buf {
            led: Ledger {
                name: std::thread::current().name().unwrap_or("?").to_string(),
                events: Vec::with_capacity(1 << 14),
                sched0: sched(),
                migrations0: migrations(),
                ..Ledger::default()
            },
            start: epoch(),
            cur: CURRENT,
        };
        b.push(Tag::Start as u8, 0, CURRENT, 0, true);
        b
    }

    #[inline]
    fn push(&mut self, tag: u8, edge: u8, place: u32, x: u64, clock: bool) {
        let ns = self.start.elapsed().as_nanos() as u64;
        let cpu = if clock { thread_cpu_ns() } else { UNREAD };
        self.led.events.push(Ev {
            ns,
            cpu,
            x,
            place,
            tag,
            edge,
        });
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        self.push(Tag::End as u8, 0, CURRENT, 0, true);
        self.led.sched1 = sched();
        self.led.migrations1 = migrations();
        DONE.lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(std::mem::take(&mut self.led));
    }
}

thread_local! {
    static BUF: RefCell<Option<Buf>> = const { RefCell::new(None) };
}

#[inline]
fn with(f: impl FnOnce(&mut Buf)) {
    // (a thread ending while its ledger is handed over records nothing more)
    let _ = BUF.try_with(|b| f(b.borrow_mut().get_or_insert_with(Buf::new)));
}

/// An activity begun on this thread
#[inline]
pub fn begin(t: Tag) {
    if !on() {
        return;
    }
    let t = t as u8;
    with(|b| b.push(t, 1, CURRENT, 0, is_wait(t)));
}

/// An activity ended on this thread (the one on top of its stack)
#[inline]
pub fn end(t: Tag) {
    if !on() {
        return;
    }
    let t = t as u8;
    with(|b| b.push(t, 2, CURRENT, 0, is_wait(t)));
}

/// A point on this thread: `place` the block's (or [`CURRENT`]: a worker's block), `x` the
/// tag's figure. `Got` sets the worker's current block.
#[inline]
pub fn point(t: Tag, place: u64, x: u64) {
    if !on() {
        return;
    }
    with(|b| {
        let mut place = if place == u64::MAX {
            CURRENT
        } else {
            place as u32
        };
        if t == Tag::Got {
            b.cur = place;
        } else if place == CURRENT {
            place = b.cur;
        }
        b.push(t as u8, 0, place, x, false)
    });
}

/// The buffers whose occupancy is kept live
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Buffer {
    /// issued and not yet taken by the reader: the admission credit held
    Admitted = 0,
    /// in the job queue
    Queued = 1,
    /// taken by a worker and not yet published
    Executing = 2,
    /// published and not yet taken by the reader (blocks)
    Ready = 3,
    /// the same, in decoded bytes
    ReadyBytes = 4,
}

pub const OCCS: usize = 5;
pub const BUFFERS: [&str; OCCS] = ["admitted", "queued", "executing", "ready", "ready_bytes"];

/// A buffer's live occupancy
#[derive(Clone, Copy, Debug, Default)]
pub struct Occ {
    pub n: i64,
    /// the time of its last change (ns from the epoch)
    pub last: u64,
    /// the integral of `n` over time to `last` (count × ns)
    pub integral: u128,
    pub hwm: i64,
    /// arrivals (positive changes, summed)
    pub arrivals: u64,
}

static OCC: Mutex<[Occ; OCCS]> = Mutex::new(
    [Occ {
        n: 0,
        last: 0,
        integral: 0,
        hwm: 0,
        arrivals: 0,
    }; OCCS],
);

/// Buffers' occupancies changed now, each `(buffer, change)`, under one lock, and the point
/// that changed them on this thread's ledger at the same instant (one clock read for both, so a
/// thread preempted between them cannot part them)
#[inline]
pub fn occ_point(changes: &[(Buffer, i64)], t: Tag, place: u64, x: u64) {
    if !on() {
        return;
    }
    // (the thread's ledger first, so that its start precedes this point)
    with(|_| {});
    let e = epoch();
    let mut o = OCC.lock().unwrap_or_else(|e| e.into_inner());
    let now = e.elapsed().as_nanos() as u64;
    occ_at(&mut o, changes, now);
    drop(o);
    with(|b| {
        let mut place = if place == u64::MAX {
            CURRENT
        } else {
            place as u32
        };
        if t == Tag::Got {
            b.cur = place;
        } else if place == CURRENT {
            place = b.cur;
        }
        b.led.events.push(Ev {
            ns: now,
            cpu: UNREAD,
            x,
            place,
            tag: t as u8,
            edge: 0,
        });
    });
}

fn occ_at(o: &mut [Occ; OCCS], changes: &[(Buffer, i64)], now: u64) {
    for &(b, d) in changes {
        let s = &mut o[b as usize];
        if s.n > 0 {
            s.integral += s.n as u128 * u128::from(now.saturating_sub(s.last));
        }
        s.last = now;
        s.n += d;
        s.hwm = s.hwm.max(s.n);
        if d > 0 {
            s.arrivals += d as u64;
        }
    }
}

/// The buffers' occupancies, each integrated to `to` (ns from the epoch)
pub fn take_occ(to: u64) -> [Occ; OCCS] {
    let mut o = *OCC.lock().unwrap_or_else(|e| e.into_inner());
    for s in o.iter_mut() {
        if s.n > 0 && to > s.last {
            s.integral += s.n as u128 * u128::from(to - s.last);
            s.last = to;
        }
    }
    o
}

/// The time now, ns from the epoch
pub fn now_ns() -> u64 {
    epoch().elapsed().as_nanos() as u64
}
