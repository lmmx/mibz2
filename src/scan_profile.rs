//! Research only (`scan-profile`): the bench-scan-profile diagnostic's timers
//! (docs/journal/2026-11-24-scan-profile-registration.md), for examples/scan_profile.rs. The
//! question: what is the parallel decoder's serial scanner's time made of, work against
//! waiting, by component, at 1, 6 and 10 workers?
//!
//! **Off** ([`set_on`] false, the default): each hook is one relaxed atomic load.
//!
//! **On**, a clock read at each boundary, nothing else:
//! - **the scanner thread, per block:** its wait for a free slot (the window full: the queue
//!   ahead of the workers), its read of the block (`Scanner::next`), and the hand-off to the job
//!   queue (`send_job`);
//! - **inside that read:** the waits for compressed input (the reader's channel), and the
//!   phases of `block::read_item_with_scratch`:
//!   - the header (magic, CRC, `orig_ptr`);
//!   - the bitmap of bytes in use and the group and selector counts;
//!   - the selectors (their unary, move-to-front-coded values);
//!   - the code lengths and the Huffman tables' builds;
//!   - the symbols buffer's preparation (its reserve and zero fill);
//!   - the symbol decode (the Huffman decode of every group, its table switched per selector).
//!
//!   The rest of the read (the stream headers and ends, the window's lookahead) is the read's
//!   remainder. Also counted: the selectors, the table switches (a selector naming another
//!   table than the one before it), the tables built, the symbols, the block's bits;
//! - **each worker:** its time blocked waiting for a job (the queue empty), its jobs, and its
//!   whole life.
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

static ON: AtomicBool = AtomicBool::new(false);

/// Turn the timers on or off, for the decodes started from now on
pub fn set_on(on: bool) {
    ON.store(on, Ordering::SeqCst);
}

/// Whether the timers are on
#[inline]
pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// The phases of a block's read (`read_item_with_scratch`), in order
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Phase {
    Header = 0,
    Bitmap = 1,
    Selectors = 2,
    Tables = 3,
    Prep = 4,
    Decode = 5,
}

pub const PHASES: usize = 6;
pub const PHASE_NAMES: [&str; PHASES] =
    ["header", "bitmap", "selectors", "tables", "prep", "decode"];

/// One block's scanner record
#[derive(Clone, Debug, Default)]
pub struct BlockRec {
    /// its index in the input (counted across streams)
    pub index: u64,
    /// its compressed bits (from its first bit to the next item's)
    pub bits: u64,
    /// its MTF/RLE2 symbols, the end-of-block one included
    pub syms: u64,
    pub selectors: u64,
    pub switches: u64,
    pub tables: u64,
    /// the scanner's wait for a free slot before the block's read
    pub slot_wait_ns: u64,
    /// the read (`Scanner::next`), whole: the input waits and phases inside it, and the rest
    pub read_ns: u64,
    /// the waits for compressed input inside the read
    pub input_wait_ns: u64,
    /// the read's phases (`Phase`)
    pub phase_ns: [u64; PHASES],
    /// the hand-off to the job queue
    pub send_ns: u64,
}

/// One worker's record, at its end
#[derive(Clone, Debug, Default)]
pub struct WorkerRec {
    pub jobs: u64,
    /// blocked waiting for a job
    pub idle_ns: u64,
    /// from its start to its end
    pub life_ns: u64,
}

static BLOCKS: Mutex<Vec<BlockRec>> = Mutex::new(Vec::new());
static WORKERS: Mutex<Vec<WorkerRec>> = Mutex::new(Vec::new());

thread_local! {
    /// the block being read on this thread: its phases so far and the read's input waits
    static CUR: RefCell<BlockRec> = RefCell::new(BlockRec::default());
    static LAST: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// The records since the last take: the scanner's blocks, and the workers that ended
pub fn take() -> (Vec<BlockRec>, Vec<WorkerRec>) {
    (
        std::mem::take(&mut *BLOCKS.lock().unwrap_or_else(|e| e.into_inner())),
        std::mem::take(&mut *WORKERS.lock().unwrap_or_else(|e| e.into_inner())),
    )
}

/// (the read) the phases' clock started
#[inline]
pub(crate) fn phase_start() {
    LAST.with(|l| l.set(Some(Instant::now())));
}

/// (the read) the phase `p` ended now: its time since the last boundary added
#[inline]
pub(crate) fn phase_end(p: Phase) {
    let now = Instant::now();
    let d = LAST
        .with(|l| l.replace(Some(now)))
        .map_or(0, |t| (now - t).as_nanos() as u64);
    CUR.with(|c| c.borrow_mut().phase_ns[p as usize] += d);
}

/// (the read) the selectors' counts
#[inline]
pub(crate) fn counts(selectors: u64, switches: u64, tables: u64) {
    CUR.with(|c| {
        let mut c = c.borrow_mut();
        c.selectors += selectors;
        c.switches += switches;
        c.tables += tables;
    });
}

/// (the read) a wait for input of `ns`
#[inline]
pub(crate) fn input_wait(ns: u64) {
    CUR.with(|c| c.borrow_mut().input_wait_ns += ns);
}

/// (the scanner) a block's record finished: the read's record so far and the scanner's own
/// times, pushed; the thread's record reset
pub(crate) fn block_done(
    index: u64,
    bits: u64,
    syms: u64,
    slot_wait_ns: u64,
    read_ns: u64,
    send_ns: u64,
) {
    let mut r = CUR.with(|c| std::mem::take(&mut *c.borrow_mut()));
    r.index = index;
    r.bits = bits;
    r.syms = syms;
    r.slot_wait_ns = slot_wait_ns;
    r.read_ns = read_ns;
    r.send_ns = send_ns;
    BLOCKS.lock().unwrap_or_else(|e| e.into_inner()).push(r);
}

/// (the scanner) the record of a read that gave no block (a stream's end), dropped
pub(crate) fn block_none() {
    CUR.with(|c| *c.borrow_mut() = BlockRec::default());
}

/// (a worker) its record at its end
pub(crate) fn worker_done(r: WorkerRec) {
    WORKERS.lock().unwrap_or_else(|e| e.into_inner()).push(r);
}
