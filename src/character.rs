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
    // any thread
    /// waiting to acquire the decoder's state lock (`Shared::lock`), apart from waiting on its
    /// condition variables
    LockState = 20,
    /// waiting to acquire the job queue's lock (`Jobs::lock`)
    LockJobs = 21,
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
    /// the scanner, at a wait for admission: both admission predicates and their operands
    /// (`x`: [`bound`]'s packing)
    Bound = 38,
    /// the scanner: admitted, its next read begun (the block it scans, outside issued − taken)
    ScanStart = 39,
    /// the scanner: its last read gave no block (the input's end, an error, a cancel)
    ScanAbandon = 40,
    /// the reader: the block it took read to its end (the block it holds, outside issued −
    /// taken, from its `Taken` to here)
    Consumed = 41,
    /// the thread's first event and its last
    Start = 48,
    End = 49,
}

/// The activities and waits, in `Tag` order (their names in the ledgers)
pub const ACTIVITIES: [&str; 22] = [
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
    "lock_state",
    "lock_jobs",
];

/// Whether an activity is a wait (its begin and end read the thread's CPU time)
pub fn is_wait(t: u8) -> bool {
    matches!(t, 1 | 2 | 6 | 8 | 19)
}

/// Whether an activity is a lock's acquisition (a wait, but too short and frequent for the CPU
/// time's system call at its ends: about 7 a block)
pub fn is_lock(t: u8) -> bool {
    matches!(t, 20 | 21)
}

/// Research treatments switched at run time, so that one binary serves stage 1 (all off) and
/// stage 2 (the brief's pacing, gates, placement and memory dose). Off, `end` pays one relaxed
/// load more.
pub mod treat {
    use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, AtomicUsize, Ordering};

    pub(super) static ANY: AtomicBool = AtomicBool::new(false);
    /// each activity's pacing: dependent-ALU iterations at its end (0: none)
    pub(super) static PACE: [AtomicU64; super::ACTIVITIES.len()] =
        [const { AtomicU64::new(0) }; super::ACTIVITIES.len()];
    /// the memory dose: its activity (`u64::MAX`: none), its bytes, and whether a chase
    pub(super) static DOSE_AT: AtomicU64 = AtomicU64::new(u64::MAX);
    pub(super) static DOSE_BYTES: AtomicUsize = AtomicUsize::new(0);
    pub(super) static DOSE_CHASE: AtomicBool = AtomicBool::new(false);
    /// the ready bytes' cap (0: the decoder's own)
    pub(super) static READY_CAP: AtomicUsize = AtomicUsize::new(0);
    /// the scanner's CPU (-1: unpinned)
    pub(super) static SCANNER_CPU: AtomicI32 = AtomicI32::new(-1);

    /// The pacing of activity `t`: `iters` iterations of a dependent multiply-add chain at each of
    /// its ends (no memory touched; a fixed count of core cycles, not a wall-clock deadline,
    /// which droop and preemption would shorten)
    pub fn set_pace(t: usize, iters: u64) {
        PACE[t].store(iters, Ordering::SeqCst);
        refresh();
    }
    /// The memory dose at each end of activity `t`: `bytes` of this thread's own scratch,
    /// streamed (one read and one write a 64-byte line) or chased (one dependent load a line,
    /// a random single cycle); `None` for none
    pub fn set_dose(at: Option<usize>, bytes: usize, chase: bool) {
        DOSE_AT.store(at.map_or(u64::MAX, |t| t as u64), Ordering::SeqCst);
        DOSE_BYTES.store(bytes, Ordering::SeqCst);
        DOSE_CHASE.store(chase, Ordering::SeqCst);
        refresh();
    }
    /// The ready bytes' cap for decoders built from now on (`None`: the decoder's own)
    pub fn set_ready_cap(bytes: Option<usize>) {
        READY_CAP.store(bytes.unwrap_or(0), Ordering::SeqCst);
    }
    /// The scanner pinned to `cpu` for decoders built from now on (`None`: unpinned)
    pub fn set_scanner_cpu(cpu: Option<usize>) {
        SCANNER_CPU.store(cpu.map_or(-1, |c| c as i32), Ordering::SeqCst);
    }
    fn refresh() {
        let any = PACE.iter().any(|p| p.load(Ordering::Relaxed) > 0)
            || DOSE_AT.load(Ordering::Relaxed) != u64::MAX;
        ANY.store(any, Ordering::SeqCst);
    }
    /// What is switched on, for the record
    pub fn describe() -> String {
        let pace: Vec<String> = PACE
            .iter()
            .enumerate()
            .filter(|(_, p)| p.load(Ordering::Relaxed) > 0)
            .map(|(i, p)| format!("{}={}", super::ACTIVITIES[i], p.load(Ordering::Relaxed)))
            .collect();
        let dose = DOSE_AT.load(Ordering::Relaxed);
        format!(
            "pace[{}] dose[{}] ready_cap[{}] scanner_cpu[{}]",
            pace.join(","),
            if dose == u64::MAX {
                String::new()
            } else {
                format!(
                    "{}:{}:{}",
                    super::ACTIVITIES[dose as usize],
                    DOSE_BYTES.load(Ordering::Relaxed),
                    if DOSE_CHASE.load(Ordering::Relaxed) {
                        "chase"
                    } else {
                        "stream"
                    }
                )
            },
            READY_CAP.load(Ordering::Relaxed),
            SCANNER_CPU.load(Ordering::Relaxed)
        )
    }
}

/// (the decoder) the ready bytes' cap switched at run time, if it is
pub(crate) fn ready_cap() -> Option<usize> {
    Some(treat::READY_CAP.load(Ordering::Relaxed)).filter(|&c| c > 0)
}

/// (the scanner, at its start) pinned to its CPU, if one is switched on
pub(crate) fn place_scanner() {
    let cpu = treat::SCANNER_CPU.load(Ordering::Relaxed);
    if cpu < 0 {
        return;
    }
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
        }
        let mut mask = [0u64; 16];
        let c = cpu as usize;
        if c < 1024 {
            mask[c / 64] |= 1 << (c % 64);
            // SAFETY: this thread (0), a mask of the size given
            unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) };
        }
    }
}

/// A dependent multiply-add chain of `n` iterations (the pacing): no memory touched
#[inline(never)]
fn spin(n: u64) {
    let mut x = std::hint::black_box(0x9E37_79B9_7F4A_7C15u64);
    for _ in 0..n {
        x = x
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
    }
    std::hint::black_box(x);
}

thread_local! {
    /// the memory dose's scratch, this thread's own: its bytes, and the chase's cycle
    static SCRATCH: RefCell<(Vec<u8>, Vec<u32>)> = const { RefCell::new((Vec::new(), Vec::new())) };
}

/// The memory dose: `bytes` of this thread's scratch streamed or chased
#[inline(never)]
fn dose(bytes: usize, chase: bool) {
    SCRATCH.with(|s| {
        let mut s = s.borrow_mut();
        let lines = bytes / 64;
        if lines == 0 {
            return;
        }
        if !chase {
            if s.0.len() < lines * 64 {
                s.0 = vec![1u8; lines * 64];
            }
            for l in s.0[..lines * 64].chunks_exact_mut(64) {
                l[0] = l[0].wrapping_add(l[32]);
            }
        } else {
            if s.1.len() != lines * 16 {
                // a random single cycle through the lines (Sattolo's, a fixed seed), each
                // line's first word the next line's index
                let mut perm: Vec<u32> = (0..lines as u32).collect();
                let mut r = 0x2545_F491_4F6C_DD1Du64;
                for i in (1..lines).rev() {
                    r ^= r << 13;
                    r ^= r >> 7;
                    r ^= r << 17;
                    let j = (r % i as u64) as usize;
                    perm.swap(i, j);
                }
                s.1 = vec![0u32; lines * 16];
                for (i, &next) in perm.iter().enumerate() {
                    s.1[i * 16] = next;
                }
            }
            let mut at = 0u32;
            for _ in 0..lines {
                at = s.1[at as usize * 16];
            }
            std::hint::black_box(at);
        }
    });
}

/// (an activity's end) the treatments switched on for it
#[inline(never)]
fn treat_at(t: u8) {
    let i = usize::from(t);
    if let Some(p) = treat::PACE.get(i) {
        let n = p.load(Ordering::Relaxed);
        if n > 0 {
            spin(n);
        }
    }
    if treat::DOSE_AT.load(Ordering::Relaxed) == i as u64 {
        dose(
            treat::DOSE_BYTES.load(Ordering::Relaxed),
            treat::DOSE_CHASE.load(Ordering::Relaxed),
        );
    }
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

/// The scanner's admission predicates at a wait (`Tag::Bound`'s `x`): bit 0 the window's
/// (`issued − taken ≥ W`), bit 1 the ready bytes' cap's (`ready bytes ≥ cap && issued − taken ≥
/// floor`), then `issued − taken` (bits 8–31) and the ready bytes in KiB (bits 32–63)
pub mod bound {
    pub fn pack(window: bool, cap: bool, ahead: u64, ready_bytes: u64) -> u64 {
        u64::from(window)
            | u64::from(cap) << 1
            | (ahead & 0xFF_FFFF) << 8
            | (ready_bytes >> 10) << 32
    }
    /// (window, cap, ahead, ready KiB)
    pub fn unpack(x: u64) -> (bool, bool, u64, u64) {
        (x & 1 == 1, x & 2 == 2, (x >> 8) & 0xFF_FFFF, x >> 32)
    }
}

/// An event; `cpu` is the thread's CPU time (ns) where read, else `UNREAD`; `seq` a queue
/// transition's place in the buffers' one linear order (taken under their lock; 0 for the
/// rest); `on_cpu` the logical CPU the thread ran on at the event
#[derive(Clone, Copy, Debug)]
pub struct Ev {
    pub ns: u64,
    pub cpu: u64,
    pub x: u64,
    pub seq: u64,
    pub place: u32,
    pub on_cpu: u16,
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
    /// the events past its preallocated capacity (each a reallocation's risk while decoding)
    pub overflows: u64,
}

static ON: AtomicBool = AtomicBool::new(false);
/// Each ledger's preallocated events (the harness sets it from the work ahead)
static CAPACITY: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1 << 14);
static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Each thread's ledger's events preallocated from now on: `worker` for a worker's, `other` for
/// the scanner's, the feeder's and the reader's
pub fn set_capacity(worker: usize, other: usize) {
    CAPACITY.store(worker << 32 | other, Ordering::SeqCst);
}

/// The logical CPU this thread is on now (`sched_getcpu`, the vDSO's), or `u16::MAX`
#[inline]
pub fn this_cpu() -> u16 {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn sched_getcpu() -> i32;
        }
        // SAFETY: no arguments
        let c = unsafe { sched_getcpu() };
        if c >= 0 {
            return c as u16;
        }
    }
    u16::MAX
}
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
        SEQ.store(0, Ordering::SeqCst);
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
        let name = std::thread::current().name().unwrap_or("?").to_string();
        let cap = CAPACITY.load(Ordering::Relaxed);
        let worker = name
            .strip_prefix("mibz2-")
            .is_some_and(|d| d.parse::<u32>().is_ok());
        let cap = if worker { cap >> 32 } else { cap & 0xFFFF_FFFF };
        let mut b = Buf {
            led: Ledger {
                name,
                events: Vec::with_capacity(cap.max(64)),
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
        self.push_at(ns, 0, tag, edge, place, x, clock);
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn push_at(&mut self, ns: u64, seq: u64, tag: u8, edge: u8, place: u32, x: u64, clock: bool) {
        let cpu = if clock { thread_cpu_ns() } else { UNREAD };
        let ev = &mut self.led.events;
        if ev.len() == ev.capacity() {
            self.led.overflows += 1;
        }
        ev.push(Ev {
            ns,
            cpu,
            x,
            seq,
            place,
            on_cpu: this_cpu(),
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

/// An activity ended on this thread (the one on top of its stack); a treatment switched on
/// for it runs first, inside the activity
#[inline]
pub fn end(t: Tag) {
    if treat::ANY.load(Ordering::Relaxed) {
        treat_at(t as u8);
    }
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
    /// the scanner's block in its read: admitted and not yet issued (0 or 1)
    Scanning = 5,
    /// the reader's block, taken and not yet read to its end (0 or 1)
    Consuming = 6,
}

pub const OCCS: usize = 7;
pub const BUFFERS: [&str; OCCS] = [
    "admitted",
    "queued",
    "executing",
    "ready",
    "ready_bytes",
    "scanning",
    "consuming",
];

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
    // (the transitions' one linear order: each takes the next number under this lock)
    let seq = SEQ.fetch_add(1, Ordering::Relaxed) + 1;
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
        b.push_at(now, seq, t as u8, 0, place, x, false);
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

static FEATURES: AtomicBool = AtomicBool::new(false);
/// Each run-route block's structure (the untimed features pass only): (place, its runs'
/// widths in the census's bins 1, 2–3, 4–16, 17–127, 128+, and h₁ of its BWT column, bits a
/// byte)
type Features = (u32, [u64; 5], f64);
static FEATS: Mutex<Vec<Features>> = Mutex::new(Vec::new());

/// Turn the blocks' structure on or off (the harness's untimed `--features` pass alone: an
/// O(r) pass a block, never in a timed decode)
pub fn set_features(on: bool) {
    FEATS.lock().unwrap_or_else(|e| e.into_inner()).clear();
    FEATURES.store(on, Ordering::SeqCst);
}

#[inline]
pub(crate) fn features_on() -> bool {
    FEATURES.load(Ordering::Relaxed)
}

/// (a worker, its current block's) run records (`byte << 8 | length`, a long run cut into
/// several): its runs' widths' histogram and the column's first-order entropy
pub(crate) fn block_features(records: &[u16]) {
    let mut bins = [0u64; 5];
    let mut count = [0u64; 256];
    let bin = |w: u64| match w {
        0..=1 => 0,
        2..=3 => 1,
        4..=16 => 2,
        17..=127 => 3,
        _ => 4,
    };
    // (a long run is cut into several records: consecutive records of one byte are one run)
    let (mut cur, mut width) = (None, 0u64);
    for &x in records {
        let (byte, w) = (x >> 8, u64::from(x & 0xFF));
        count[usize::from(byte)] += w;
        if cur == Some(byte) {
            width += w;
        } else {
            if cur.is_some() {
                bins[bin(width)] += 1;
            }
            (cur, width) = (Some(byte), w);
        }
    }
    if cur.is_some() {
        bins[bin(width)] += 1;
    }
    let n: u64 = count.iter().sum();
    let h1 = if n == 0 {
        0.0
    } else {
        count
            .iter()
            .filter(|&&c| c > 0)
            .map(|&c| {
                let q = c as f64 / n as f64;
                -q * q.log2()
            })
            .sum()
    };
    let mut place = CURRENT;
    with(|b| place = b.cur);
    FEATS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((place, bins, h1));
}

/// The blocks' structure recorded since `set_features(true)`
pub fn take_features() -> Vec<Features> {
    std::mem::take(&mut *FEATS.lock().unwrap_or_else(|e| e.into_inner()))
}

/// The time now, ns from the epoch
pub fn now_ns() -> u64 {
    epoch().elapsed().as_nanos() as u64
}
