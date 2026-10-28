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
//!
//! **Sizes** (level 1 and up): a few marks carry two byte counts, `len` (bytes holding data
//! still to be read) and `cap` (bytes reserved): the symbols a block's read wrote
//! (`Issued`), its decoded bytes (`DecodeEnd`), the scanner's window (`Window`); and
//! `InvertStart` carries the block's length in rows. A worker also records its buffers'
//! sizes once, as it ends (`buffers`). Reading a `Vec`'s length and capacity is two loads;
//! nothing here allocates on the decoder's path beyond the marks' own buffer.
//!
//! **The footprint ledger** (`ledger`, below) is the arithmetic of what the parallel
//! decoder allocates, by category, from the stream's level, the threads, the window and
//! the input chunk: a pure function, checked against the buffers' real capacities by this
//! module's tests. docs/ledger.md says what its figures mean and what they can't tell.

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
    /// a worker, inside `decode_block`: the inverse BWT's start and end (the T-vector
    /// built, then walked: the phase when it is live); the start carries the block's
    /// length in rows (`len`)
    InvertStart,
    InvertEnd,
    /// the scanner: its window once a chunk is appended (`len`: the bytes not yet
    /// released, which it will still read; `cap`: the window's capacity)
    Window,
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
    /// the sizes a mark carries (`Issued`, `DecodeEnd`, `Window`: bytes in use and
    /// reserved; `InvertStart`: the block's rows in `len`), else `UNREAD`
    pub len: u64,
    pub cap: u64,
}

/// One thread's marks, in its order, and the buffers it recorded (`buffers`)
pub struct ThreadTrace {
    pub name: String,
    pub marks: Vec<Mark>,
    pub buffers: Vec<Buffer>,
}

/// A buffer's size: bytes in use (`len`) and reserved (`cap`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Buffer {
    pub name: &'static str,
    pub len: usize,
    pub cap: usize,
}

impl Buffer {
    /// A `Vec`'s, in bytes
    pub fn of<T>(name: &'static str, v: &Vec<T>) -> Buffer {
        let size = std::mem::size_of::<T>();
        Buffer {
            name,
            len: v.len() * size,
            cap: v.capacity() * size,
        }
    }
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
    buffers: Vec<Buffer>,
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
            buffers: Vec::new(),
            start: *epoch().lock().unwrap_or_else(|e| e.into_inner()),
            level,
            #[cfg(target_os = "linux")]
            sched: if level >= 2 {
                std::fs::File::open("/proc/thread-self/schedstat").ok()
            } else {
                None
            },
        };
        b.push(Kind::ThreadStart, NONE, true, UNREAD, UNREAD);
        b
    }

    #[inline]
    fn push(&mut self, kind: Kind, place: u64, clocks: bool, len: u64, cap: u64) {
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
            len,
            cap,
        });
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        self.push(Kind::ThreadEnd, NONE, true, UNREAD, UNREAD);
        let t = ThreadTrace {
            name: std::mem::take(&mut self.name),
            marks: std::mem::take(&mut self.marks),
            buffers: std::mem::take(&mut self.buffers),
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
    mark_sized(kind, place, UNREAD, UNREAD);
}

/// `mark`, carrying two sizes (`Mark::len`, `Mark::cap`)
#[inline]
pub fn mark_sized(kind: Kind, place: u64, len: u64, cap: u64) {
    let level = LEVEL.load(Ordering::Relaxed);
    if level == 0 {
        return;
    }
    let clocks = matches!(kind, Kind::DecodeStart | Kind::DecodeEnd);
    // (a thread ending while its buffer is dropped marks nothing more)
    let _ = BUF.try_with(|b| {
        b.borrow_mut()
            .get_or_insert_with(|| Buf::new(level))
            .push(kind, place, clocks, len, cap)
    });
}

/// This thread's buffers' sizes, kept with its marks (nothing at level 0): a worker's, as
/// it ends. Its capacities are then its high-water marks, as its buffers never shrink.
pub fn buffers(list: &[Buffer]) {
    let level = LEVEL.load(Ordering::Relaxed);
    if level == 0 {
        return;
    }
    let _ = BUF.try_with(|b| {
        b.borrow_mut()
            .get_or_insert_with(|| Buf::new(level))
            .buffers
            .extend_from_slice(list)
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

// ---------------------------------------------------------------------------------------
// The footprint ledger

/// The window the scanner reads a block's first bytes through (stream.rs's `LOOKAHEAD`; it
/// doubles only for a block whose compressed form is longer, which no real block is)
pub const LOOKAHEAD: usize = 2 << 20;
/// `BzReader::parallel`'s input chunk (read.rs's `CHUNK`)
pub const CHUNK: usize = 4 << 20;
/// The T-vector's bytes a row (bwt.rs's `TVector`: a `u32` a row)
pub const TT_ROW: usize = 4;

/// What the ledger is computed for
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// the stream's level, 1-9 (the stream header's): blocks of at most 100,000 × level rows
    pub level: u32,
    pub threads: usize,
    /// the window: blocks between the scanner and the reader (`StreamDecoder::new` raises
    /// it to `threads`, as `Settings::new` does)
    pub slots: usize,
    /// the writer's input chunk, in bytes (`BzReader::parallel`'s feeder: `CHUNK`)
    pub chunk: usize,
    /// the longest block, in rows (at most 100,000 × level, and that by default)
    pub rows: usize,
    /// the largest decoded block, in bytes (`rows` by default: text with few RLE1 runs; at
    /// most `max_decoded(rows)`, a block of runs only)
    pub decoded: usize,
}

impl Settings {
    /// A stream of `level` whose blocks are full and decode to about their length, decoded
    /// by `BzReader::parallel_with(_, threads, slots, _)`
    pub fn new(level: u32, threads: usize, slots: usize) -> Settings {
        let threads = threads.max(1);
        let rows = block_rows(level);
        Settings {
            level,
            threads,
            slots: slots.max(threads),
            chunk: CHUNK,
            rows,
            decoded: rows,
        }
    }
}

/// A level's block bound, in rows (block.rs's `max_len`)
pub fn block_rows(level: u32) -> usize {
    100_000 * level as usize
}

/// The most bytes a block of `rows` rows decodes to: every 5 bytes a run of 4 + 255
pub fn max_decoded(rows: usize) -> usize {
    rows / 5 * 259 + rows % 5
}

/// One line of the ledger: a kind of buffer, how many, and each one's bytes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// whose: "worker" (one set each), "slot" (one each in the window, the pool's), "input",
    /// "decoder"
    pub owner: &'static str,
    /// the buffer's name, as `Scratch::buffers` gives it where a worker has it
    pub name: &'static str,
    /// how many there are at most, and the rule ("threads", "slots", ...)
    pub count: usize,
    pub per: &'static str,
    /// bytes reserved each once a block of `rows` rows and `decoded` bytes has been through
    /// (`Vec` capacity; exact where `alloc_max` equals it)
    pub alloc: usize,
    /// bytes reserved each at most, whatever order the blocks come in (`Vec` growth
    /// doubles, and capacity never shrinks); `None` where no bound short of the absurd
    /// holds (it ratchets with the data: measure it)
    pub alloc_max: Option<usize>,
    /// bytes each holding data that will still be read, at most
    pub live: usize,
    /// bytes each randomly accessed during the T-vector phase (the build's scattered
    /// writes, the walk's dependent reads): a model of the walk's working set, not a
    /// measured cache residency
    pub hot: usize,
    /// when it is allocated: "once" (then reused, its capacity a high-water mark), "per
    /// block" or "per chunk" (allocated and freed each time)
    pub life: &'static str,
    /// when it is live, and anything else to know
    pub note: &'static str,
}

/// The parallel decoder's allocations by category (see docs/ledger.md): from the code, not
/// measured. Thread stacks are listed for completeness (virtual: a stack's pages are
/// resident only once touched), and so is the scanner's per-block table, approximately.
pub fn ledger(s: &Settings) -> Vec<Entry> {
    let (t, rows, max_rows) = (s.threads, s.rows, block_rows(s.level));
    let segmented = rows >= crate::bwt::MIN_SEGMENTED;
    let ns = crate::bwt::FLAGGED_SEGMENTS;
    let seg_cap = 2 * rows / ns + 64;
    let out_req = (rows + 64).max(s.decoded);
    let window_max = crate::parallel::COMPACT + LOOKAHEAD + s.chunk;
    let mut v = vec![
        Entry {
            owner: "worker",
            name: "ll",
            count: t,
            per: "threads",
            alloc: max_rows,
            alloc_max: Some(max_rows),
            live: rows,
            hot: 0,
            life: "once",
            note: "the BWT's last column: reserved at the level's bound; written by MTF, read in order by the T-vector's build, dead from the walk on",
        },
        Entry {
            owner: "worker",
            name: "tt",
            count: t,
            per: "threads",
            alloc: TT_ROW * rows,
            alloc_max: Some(TT_ROW * (2 * rows).saturating_sub(1)),
            live: TT_ROW * rows,
            hot: TT_ROW * rows,
            life: "once",
            note: "the T-vector: live from its build to the walk's end; doubles when a block is longer than any before it on that worker",
        },
        Entry {
            owner: "worker",
            name: "rle",
            count: t,
            per: "threads",
            alloc: rows,
            alloc_max: Some((2 * rows).saturating_sub(1)),
            live: rows,
            hot: 0,
            life: "once",
            note: "the walk's output (RLE1-coded): written by the join, read in order by RLE1; doubles as tt does",
        },
        Entry {
            owner: "worker",
            name: "counts",
            count: t,
            per: "threads",
            alloc: 1024,
            alloc_max: Some(1024),
            live: 1024,
            hot: 1024,
            life: "once",
            note: "the byte counts (inline, on the worker's stack)",
        },
    ];
    if segmented {
        v.extend([
            Entry {
                owner: "worker",
                name: "seg.bufs",
                count: t,
                per: "threads",
                alloc: ns * seg_cap,
                alloc_max: None,
                live: rows,
                hot: 0,
                life: "once",
                note: "the walk's segments, joined into rle: each reserved 2 x the mean segment, doubled when its segment fills it, so each ratchets with the longest it has held",
            },
            Entry {
                owner: "worker",
                name: "seg.bufs.outer",
                count: t,
                per: "threads",
                alloc: ns * std::mem::size_of::<Vec<u8>>(),
                alloc_max: Some(2 * ns * std::mem::size_of::<Vec<u8>>()),
                live: ns * std::mem::size_of::<Vec<u8>>(),
                hot: 0,
                life: "once",
                note: "the segments' Vec headers",
            },
            Entry {
                owner: "worker",
                name: "seg.starts",
                count: t,
                per: "threads",
                alloc: 2 * (ns - 1) * 4,
                alloc_max: Some(2 * (ns - 1) * 4),
                live: ns * 4,
                hot: ns * 4,
                life: "once",
                note: "the segments' start rows, binary-searched at each segment's end",
            },
            Entry {
                owner: "worker",
                name: "seg.next",
                count: t,
                per: "threads",
                alloc: ns * 8,
                alloc_max: Some(2 * ns * 8),
                live: ns * 8,
                hot: 0,
                life: "once",
                note: "each segment's successor",
            },
        ]);
    }
    v.extend([
        Entry {
            owner: "slot",
            name: "symbols",
            count: s.slots,
            per: "slots",
            alloc: 2 * crate::block::MAX_SYMS,
            alloc_max: Some(2 * crate::block::MAX_SYMS),
            live: 2 * (rows + 1),
            hot: 0,
            life: "once",
            note: "a block's Huffman-decoded symbols, reserved at the format's bound whatever the level: live from the scanner's read to the end of MTF, held until the worker publishes its block",
        },
        Entry {
            owner: "slot",
            name: "out",
            count: s.slots + 1,
            per: "slots + 1",
            alloc: out_req,
            alloc_max: Some(2 * out_req - 1),
            live: s.decoded,
            hot: 0,
            life: "once",
            note: "a block's decoded bytes: in a worker, in `ready`, or the reader's current block (the + 1); live until the reader has copied them out",
        },
        Entry {
            owner: "input",
            name: "chunks",
            count: crate::parallel::INPUT_QUEUE + 2,
            per: "INPUT_QUEUE + 2",
            alloc: s.chunk,
            alloc_max: Some(s.chunk),
            live: s.chunk,
            hot: 0,
            life: "per chunk",
            note: "compressed input: queued, one held by the writer (filling it, or blocked sending it), one being appended to the window",
        },
        Entry {
            owner: "input",
            name: "window",
            count: 1,
            per: "1",
            alloc: s.chunk,
            alloc_max: Some(2 * window_max - 1),
            live: LOOKAHEAD + s.chunk,
            hot: 0,
            life: "once",
            note: "the scanner's compressed bytes, from the block being read on (live: not yet released); up to COMPACT bytes already read are kept before it drops them, so it holds less than COMPACT + LOOKAHEAD + chunk, reserved by doubling from the first chunk",
        },
        Entry {
            owner: "decoder",
            name: "block tables",
            count: 1,
            per: "1",
            alloc: 18_002 + 6 * 4_804,
            alloc_max: Some(18_002 + 6 * 4_804),
            live: 18_002 + 6 * 4_804,
            hot: 0,
            life: "per block",
            note: "the scanner's selectors and Huffman tables for the block it reads (approximate: the tables' struct size)",
        },
        Entry {
            owner: "decoder",
            name: "stacks",
            count: t + 2,
            per: "threads + 2",
            alloc: 2 << 20,
            alloc_max: Some(2 << 20),
            live: 0,
            hot: 0,
            life: "once",
            note: "the workers', scanner's and feeder's stacks: std's default 2 MiB each (RUST_MIN_STACK), virtual, a few KiB touched",
        },
    ]);
    v
}

/// A ledger's totals: (reserved, reserved at most or `None` where an entry has no bound,
/// the sum of each entry's live maximum, the model's hot bytes), over heap entries
/// (stacks left out: virtual)
pub fn totals(entries: &[Entry]) -> (usize, Option<usize>, usize, usize) {
    let heap = entries.iter().filter(|e| e.name != "stacks");
    let (mut alloc, mut max, mut live, mut hot) = (0, Some(0usize), 0, 0);
    for e in heap {
        alloc += e.count * e.alloc;
        max = max.zip(e.alloc_max).map(|(a, b)| a + e.count * b);
        live += e.count * e.live;
        hot += e.count * e.hot;
    }
    (alloc, max, live, hot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::{Event, Scanner, SliceSource};
    use crate::{decode_block, Kernels, Scratch};

    fn compress(data: &[u8], level: u32) -> Vec<u8> {
        use std::io::Write;
        let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(level));
        e.write_all(data).unwrap();
        e.finish().unwrap()
    }

    /// Letters at random (few RLE1 runs, so a block's decoded bytes are about its rows)
    fn letters(n: usize, seed: u64) -> Vec<u8> {
        let mut x = seed | 1;
        (0..n)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                b'a' + (x % 26) as u8
            })
            .collect()
    }

    /// `Vec`'s growth: a reserve for `need` elements on a cleared buffer of `cap`
    fn grown(cap: usize, need: usize) -> usize {
        if need <= cap {
            cap
        } else {
            need.max(2 * cap)
        }
    }

    fn get(v: &[Buffer], name: &str) -> Buffer {
        *v.iter()
            .find(|b| b.name == name)
            .unwrap_or_else(|| panic!("no buffer {name}"))
    }

    fn entry<'a>(l: &'a [Entry], name: &str) -> &'a Entry {
        l.iter()
            .find(|e| e.name == name)
            .unwrap_or_else(|| panic!("no ledger entry {name}"))
    }

    /// A worker's buffers after two blocks, the second longer, through the decoder's own
    /// calls (the scanner, `decode_block`, one output buffer cleared and reused as the
    /// pool's are): each capacity is the ledger's figure, or within its bounds where it
    /// depends on the blocks' order
    #[test]
    fn ledger_matches_a_workers_buffers() {
        let level = 1;
        // two streams: one block of 70,000 rows, then one of 99,000 (a level-1 block holds
        // 100,000), so the second outgrows what the first reserved
        let mut data = compress(&letters(70_000, 7), level);
        data.extend(compress(&letters(99_000, 11), level));
        let k = Kernels::detect();
        let pad = if k.avx2 { 64 } else { 0 };
        let mut sc = Scanner::new(SliceSource(&data));
        let mut scratch = Scratch::<crate::bwt::Segmented>::default();
        let (mut out, mut syms) = (Vec::new(), Vec::new());
        let (mut rows, mut decoded) = (Vec::new(), Vec::new());
        let (mut tt, mut rle, mut out_cap) = (0usize, 0usize, 0usize);
        let mut syms_cap = 0;
        while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
            syms_cap = symbols.syms.capacity() * 2;
            out.clear();
            decode_block(&symbols, k, &mut scratch, &mut out).unwrap();
            let n = get(&scratch.buffers(), "ll").len;
            rows.push(n);
            decoded.push(out.len());
            tt = grown(tt, n);
            rle = grown(rle, n);
            out_cap = grown(out_cap, n + pad);
            syms = symbols.syms;
        }
        assert_eq!(rows.len(), 2);
        assert!(rows[0] < rows[1] && rows[1] < 2 * rows[0] && rows[0] >= 1 << 16);
        assert!(decoded.iter().zip(&rows).all(|(d, n)| d <= n));
        let b = scratch.buffers();
        let s = Settings {
            rows: rows[1],
            decoded: decoded[1],
            ..Settings::new(level, 1, 1)
        };
        let l = ledger(&s);
        let within = |name: &str, cap: usize| {
            let e = entry(&l, name);
            assert!(
                e.alloc <= cap && e.alloc_max.map_or(true, |m| cap <= m),
                "{name}: {cap} outside the ledger's {} ..= {:?}",
                e.alloc,
                e.alloc_max
            );
        };
        // the last column: reserved at the level's bound, exactly
        assert_eq!(get(&b, "ll").cap, block_rows(level));
        assert_eq!(get(&b, "ll").cap, entry(&l, "ll").alloc);
        // the T-vector and the walk's output: grown as Vec grows, doubled by the second
        // block (2 x 70,000 rows reserved, 99,000 used)
        assert_eq!(get(&b, "tt").cap, TT_ROW * tt);
        assert_eq!(tt, 2 * rows[0]);
        assert_eq!(get(&b, "tt").len, TT_ROW * rows[1]);
        within("tt", get(&b, "tt").cap);
        assert_eq!(get(&b, "rle").cap, rle);
        within("rle", get(&b, "rle").cap);
        assert_eq!(get(&b, "counts").cap, entry(&l, "counts").alloc);
        // the segments' bookkeeping
        assert_eq!(get(&b, "seg.starts").cap, entry(&l, "seg.starts").alloc);
        let next = get(&b, "seg.next");
        let ns = next.len / 8;
        assert!(ns > FLAGGED - 8 && ns <= FLAGGED, "{ns} segments");
        within("seg.next", next.cap);
        within("seg.bufs.outer", get(&b, "seg.bufs.outer").cap);
        // each segment's buffer reserved 2 x the mean segment at least: the ledger's
        // figure, less the starts that coincided
        let seg = get(&b, "seg.bufs");
        let cap0 = 2 * rows[1] / ns + 64;
        assert!(seg.cap >= ns * cap0 && seg.cap >= seg.len, "{seg:?}");
        assert!(seg.cap + (FLAGGED - ns) * cap0 >= entry(&l, "seg.bufs").alloc);
        assert!(seg.len >= rows[1] && seg.len <= rows[1] + FLAGGED);
        // a block's symbols: the format's bound, whatever the level
        assert_eq!(syms_cap, entry(&l, "symbols").alloc);
        // a block's output buffer: rows + 64 reserved (AVX2's RLE1, rows for the scalar
        // one), grown as Vec grows
        assert_eq!(out.capacity(), out_cap);
        within("out", out.capacity() + 64 - pad);
        // every buffer a worker has is in the ledger
        for x in &b {
            entry(&l, x.name);
        }
    }

    const FLAGGED: usize = crate::bwt::FLAGGED_SEGMENTS;

    /// The arithmetic at -9 (docs/ledger.md's table): exact figures for full blocks
    #[test]
    fn ledger_at_level_9() {
        let l = ledger(&Settings::new(9, 4, 8));
        assert_eq!(entry(&l, "ll").alloc, 900_000);
        assert_eq!(entry(&l, "tt").alloc, 3_600_000);
        assert_eq!(entry(&l, "tt").alloc_max, Some(7_199_996));
        assert_eq!(entry(&l, "tt").hot, 3_600_000);
        assert_eq!(entry(&l, "symbols").alloc, 1_800_200);
        assert_eq!(entry(&l, "symbols").count, 8);
        assert_eq!(entry(&l, "out").count, 9);
        assert_eq!(entry(&l, "chunks").count, 4);
        assert_eq!(entry(&l, "seg.bufs").alloc, 512 * (2 * 900_000 / 512 + 64));
        let (_, max, _, hot) = totals(&l);
        assert_eq!(max, None);
        assert_eq!(hot, 4 * (3_600_000 + 1024 + 2048));
        // the window raised to the threads, as StreamDecoder::new raises it
        assert_eq!(Settings::new(9, 4, 1).slots, 4);
        assert_eq!(max_decoded(900_000), 46_620_000);
    }
}
