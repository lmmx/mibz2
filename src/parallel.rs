//! The block-parallel streaming decoder: compressed bytes written in (`StreamDecoder::write`, from
//! any thread, blocking while the input queued is full), decoded bytes read out in order
//! (`StreamDecoder::read`). Like lbzip2's, its parallelism is across blocks:
//!
//! - a **scanner** thread reads the stream in order (stream.rs): each block's header and
//!   Huffman-coded symbols (the one phase that must be sequential: a block's end is
//!   known only once they are read), and each stream's end and combined CRC;
//! - `threads` **workers** finish a block each (move-to-front, the inverse BWT, RLE1 and
//!   the block's CRC: `decode_block`);
//! - the **reader** takes the blocks in order.
//!
//! **Memory is bounded:** at most `slots` blocks are between the scanner and the reader
//! (read, being decoded, or decoded and not yet read), the scanner waiting for a slot
//! before it reads the next. A slot holds a block's symbols (1.8 MB at most) and its
//! decoded bytes (about 1 MB of JSON; at most ~46 MB for a block of runs only); each
//! worker keeps ~7 MB of buffers; the input held is the chunks queued (`INPUT_QUEUE`) and
//! the scanner's window (its lookahead, 2 MB, from the block being read on). Buffers are
//! reused from block to block, not allocated each time.
//!
//! **Errors come in order:** a block's error is the reader's once every block before it
//! is read, so the bytes read before an error are exactly the stream's up to that block.
//! A worker's panic (a bug, never the input's doing by design) is caught and becomes an
//! error, so a bad input can only ever end the decode with an error.

use super::stream::{Event, Scanner, Source};
use super::{decode_block, Error, ErrorKind, Kernels, Scratch};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

/// Input chunks queued for the scanner (the split writes 8 MB at a time)
pub(crate) const INPUT_QUEUE: usize = 2;
/// Bytes the scanner's window keeps behind it before it drops them
pub(crate) const COMPACT: usize = 4 << 20;

/// A mark on the decoder's timeline (src/trace.rs), with the `trace` feature only
macro_rules! mark {
    ($kind:ident) => {
        #[cfg(feature = "trace")]
        crate::trace::mark(crate::trace::Kind::$kind, crate::trace::NONE);
    };
    ($kind:ident, $place:expr) => {
        #[cfg(feature = "trace")]
        crate::trace::mark(crate::trace::Kind::$kind, $place);
    };
    // (with two sizes, in bytes: in use and reserved)
    ($kind:ident, $place:expr, $len:expr, $cap:expr) => {
        #[cfg(feature = "trace")]
        crate::trace::mark_sized(crate::trace::Kind::$kind, $place, $len as u64, $cap as u64);
    };
}

/// A block to finish: its place in the stream, its first bit, its symbols
type Job = (u64, u64, super::block::Symbols);

struct State {
    /// decoded blocks (or errors) by their place in the stream, not yet read
    ready: BTreeMap<u64, Result<Vec<u8>, Error>>,
    /// blocks given a place (the scanner's), and taken by the reader
    issued: u64,
    taken: u64,
    /// the number of places once the scanner has ended (its last perhaps an error)
    end: Option<u64>,
    cancelled: bool,
    /// buffers to reuse: decoded blocks' and symbols'
    free_out: Vec<Vec<u8>>,
    free_syms: Vec<Vec<u16>>,
}

struct Shared {
    state: Mutex<State>,
    cv: Condvar,
    slots: u64,
    /// the level policy's bound on large blocks inverted at once (`Plan::for_stream`)
    gate: Option<Arc<Gate>>,
}

/// At most `permits` blocks of more than `rows` rows in their inverse BWT at once: the
/// level policy's bound when it plans more teams than a large block's table allows
/// (`plan_for_stream`), so a later stream of large blocks in a concatenation keeps today's
/// live tables. A worker takes a pass after its block's MTF stage, when the block's length
/// is known, and before its table is built; smaller blocks pass freely.
pub(crate) struct Gate {
    rows: usize,
    permits: usize,
    inside: Mutex<usize>,
    cv: Condvar,
}

impl Gate {
    pub(crate) fn new(rows: usize, permits: usize) -> Gate {
        Gate {
            rows,
            permits: permits.max(1),
            inside: Mutex::new(0),
            cv: Condvar::new(),
        }
    }

    /// For a block of `n` rows: none if it is small, else a pass once one is free, given
    /// back when dropped (on unwinding too)
    pub(crate) fn enter(&self, n: usize) -> Option<GatePass<'_>> {
        if n <= self.rows {
            return None;
        }
        let mut inside = self.inside.lock().unwrap_or_else(|e| e.into_inner());
        while *inside >= self.permits {
            inside = self.cv.wait(inside).unwrap_or_else(|e| e.into_inner());
        }
        *inside += 1;
        Some(GatePass(self))
    }
}

/// A large block's pass through the `Gate`
pub(crate) struct GatePass<'a>(&'a Gate);

impl Drop for GatePass<'_> {
    fn drop(&mut self) {
        *self.0.inside.lock().unwrap_or_else(|e| e.into_inner()) -= 1;
        self.0.cv.notify_one();
    }
}

impl Shared {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        // (a panic while holding the lock is caught where it happens; the state is
        // consistent at every unlock, so a poisoned lock is still usable)
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// A write refused: the input is closed, or the decode has stopped
#[derive(Debug)]
pub struct Stopped;

pub struct StreamDecoder {
    input: Mutex<Option<SyncSender<Vec<u8>>>>,
    shared: Arc<Shared>,
    handles: Mutex<Vec<JoinHandle<()>>>,
    reader: Mutex<Reader>,
    threads: usize,
}

#[derive(Default)]
struct Reader {
    cur: Vec<u8>,
    at: usize,
    failed: Option<Error>,
    done: bool,
}

impl StreamDecoder {
    /// A decoder on `threads` threads (at least 1) with `slots` blocks in flight (at least
    /// `threads`), on kernels `k`, spending them as the cooperative policy plans
    /// (`Plan::for_threads`): `threads` workers of a block each, or, on 6 or more physical
    /// cores, teams of 2 sharing each block, with the window scaled to the teams (`window`).
    /// `planned(Plan::Workers(threads), ..)` is the decoder before the policy.
    pub fn new(threads: usize, slots: usize, k: Kernels) -> StreamDecoder {
        Self::for_stream(threads, slots, k, None)
    }

    /// `new`, knowing the input's first stream's level (its header's `BZh1`-`BZh9`, None if
    /// not known), as `Plan::for_stream` plans for it; `BzReader::parallel` and
    /// `decode_parallel` read it from the input. With no level it is `new`.
    pub fn for_stream(
        threads: usize,
        slots: usize,
        k: Kernels,
        level: Option<u32>,
    ) -> StreamDecoder {
        let threads = threads.max(1);
        let (plan, gate) = plan_for_stream(threads, physical_cores(), l3_bytes(), level);
        Self::planned_gated(plan, window(plan, threads, slots), k, gate)
    }

    /// A decoder spending its threads as `plan` says: production's workers, or teams of
    /// k sharing each block (`bwt::Cooperative`), with `slots` blocks in flight (at least
    /// the plan's workers or teams). The team size is 2, 3 or 4; another runs as that many
    /// production workers.
    pub fn planned(plan: Plan, slots: usize, k: Kernels) -> StreamDecoder {
        Self::planned_gated(plan, slots, k, None)
    }

    /// `planned`, with the large blocks inverted at once bounded by `gate` if one is given
    fn planned_gated(plan: Plan, slots: usize, k: Kernels, gate: Option<Gate>) -> StreamDecoder {
        use super::bwt::{Cooperative, Segmented};
        let g = gate.map(Arc::new);
        let mut dec = match plan {
            Plan::Workers(n) => Self::with_inverse_gated::<Segmented>(n, slots, k, g),
            Plan::Teams { teams, k: 2 } => {
                Self::with_inverse_gated::<Cooperative<2>>(teams, slots, k, g)
            }
            Plan::Teams { teams, k: 3 } => {
                Self::with_inverse_gated::<Cooperative<3>>(teams, slots, k, g)
            }
            Plan::Teams { teams, k: 4 } => {
                Self::with_inverse_gated::<Cooperative<4>>(teams, slots, k, g)
            }
            Plan::Teams { teams, k: t } => {
                Self::with_inverse_gated::<Segmented>(teams.max(1) * t.max(1), slots, k, g)
            }
        };
        dec.threads = plan.threads();
        dec
    }

    /// `new` with the inverse-BWT kernel `I` (bwt::InverseBwt), so the whole decoder can
    /// be timed with each
    pub fn with_inverse<I: super::bwt::InverseBwt + 'static>(
        threads: usize,
        slots: usize,
        k: Kernels,
    ) -> StreamDecoder {
        Self::with_inverse_gated::<I>(threads, slots, k, None)
    }

    fn with_inverse_gated<I: super::bwt::InverseBwt + 'static>(
        threads: usize,
        slots: usize,
        k: Kernels,
        gate: Option<Arc<Gate>>,
    ) -> StreamDecoder {
        let threads = threads.max(1);
        let slots = slots.max(threads) as u64;
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                ready: BTreeMap::new(),
                issued: 0,
                taken: 0,
                end: None,
                cancelled: false,
                free_out: Vec::new(),
                free_syms: Vec::new(),
            }),
            cv: Condvar::new(),
            slots,
            gate,
        });
        let (in_tx, in_rx) = sync_channel::<Vec<u8>>(INPUT_QUEUE);
        let (job_tx, job_rx) = channel::<Job>();
        let job_rx = Arc::new(Mutex::new(job_rx));
        let mut handles = Vec::with_capacity(threads + 1);
        for i in 0..threads {
            let (shared, job_rx) = (shared.clone(), job_rx.clone());
            handles.push(
                std::thread::Builder::new()
                    .name(format!("mibz2-{i}"))
                    .spawn(move || worker::<I>(&shared, &job_rx, k))
                    .expect("spawn a bz2 worker"),
            );
        }
        let sh = shared.clone();
        handles.push(
            std::thread::Builder::new()
                .name("mibz2-scan".into())
                .spawn(move || scanner(&sh, in_rx, job_tx))
                .expect("spawn the bz2 scanner"),
        );
        StreamDecoder {
            input: Mutex::new(Some(in_tx)),
            shared,
            handles: Mutex::new(handles),
            reader: Mutex::default(),
            threads,
        }
    }

    /// The threads decoding blocks: the workers, or the teams' threads in all
    pub fn threads(&self) -> usize {
        self.threads
    }

    /// Queue compressed bytes, blocking while the queue is full. Err if the input is
    /// closed or the decode has stopped (ended in an error, or cancelled): its writer
    /// should stop, as a pipe's writer does when its reader is gone.
    pub fn write(&self, data: Vec<u8>) -> Result<(), Stopped> {
        let tx = self
            .input
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or(Stopped)?;
        if data.is_empty() {
            return Ok(());
        }
        tx.send(data).map_err(|_| Stopped)
    }

    /// The input's end
    pub fn close(&self) {
        self.input.lock().unwrap_or_else(|e| e.into_inner()).take();
    }

    /// Stop: the threads end, a writer's next write fails, and the reader's next read
    /// fails (unless it has read to the end)
    pub fn cancel(&self) {
        self.close();
        let mut st = self.shared.lock();
        st.cancelled = true;
        st.ready.clear();
        drop(st);
        self.shared.cv.notify_all();
    }

    /// Decoded bytes into `buf`, in order: how many (0 only at the end), blocking until
    /// some are decoded. Err once the stream is found bad (the same error every time
    /// after), or after `cancel`.
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut r = self.reader.lock().unwrap_or_else(|e| e.into_inner());
        if !r.done && r.failed.is_none() && self.shared.lock().cancelled {
            r.failed = Some(Error::new(ErrorKind::Cancelled));
        }
        loop {
            if let Some(e) = &r.failed {
                return Err(e.clone());
            }
            if r.at < r.cur.len() {
                let n = buf.len().min(r.cur.len() - r.at);
                buf[..n].copy_from_slice(&r.cur[r.at..r.at + n]);
                r.at += n;
                return Ok(n);
            }
            if r.done || buf.is_empty() {
                return Ok(0);
            }
            let mut st = self.shared.lock();
            let spent = std::mem::take(&mut r.cur);
            if spent.capacity() != 0 && st.free_out.len() < self.shared.slots as usize {
                st.free_out.push(spent);
            }
            r.at = 0;
            mark!(ReadWait);
            loop {
                if st.cancelled {
                    r.failed = Some(Error::new(ErrorKind::Cancelled));
                    break;
                }
                let taken = st.taken;
                if let Some(res) = st.ready.remove(&taken) {
                    st.taken += 1;
                    self.shared.cv.notify_all();
                    mark!(Taken, taken);
                    match res {
                        Ok(v) => r.cur = v,
                        Err(e) => r.failed = Some(e),
                    }
                    break;
                }
                if st.end == Some(st.taken) {
                    r.done = true;
                    break;
                }
                st = self.shared.cv.wait(st).unwrap_or_else(|e| e.into_inner());
            }
        }
    }

    /// Cancel and wait for the threads
    pub fn join(&self) {
        self.cancel();
        let handles = std::mem::take(&mut *self.handles.lock().unwrap_or_else(|e| e.into_inner()));
        for h in handles {
            let _ = h.join();
        }
    }
}

impl Drop for StreamDecoder {
    fn drop(&mut self) {
        self.join();
    }
}

/// The scanner's input: chunks from the channel, held from the block being read on
struct ChannelSource<'a> {
    rx: Receiver<Vec<u8>>,
    buf: Vec<u8>,
    /// the absolute offset of buf[0]
    base: u64,
    released: u64,
    eof: bool,
    shared: &'a Shared,
}

impl Source for ChannelSource<'_> {
    fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
        while !self.eof && self.base + (self.buf.len() as u64) < from + want as u64 {
            mark!(InputWait);
            match self.rx.recv() {
                Ok(chunk) => {
                    mark!(InputGot);
                    let drop_n = (self.released - self.base) as usize;
                    if drop_n >= COMPACT {
                        self.buf.drain(..drop_n);
                        self.base = self.released;
                    }
                    self.buf.extend_from_slice(&chunk);
                    mark!(
                        Window,
                        crate::trace::NONE,
                        self.base + self.buf.len() as u64 - self.released,
                        self.buf.capacity()
                    );
                }
                Err(_) => {
                    mark!(InputEnd);
                    self.eof = true;
                }
            }
            if self.shared.lock().cancelled {
                return Err(ErrorKind::Cancelled);
            }
        }
        let i = (from - self.base) as usize;
        Ok((self.buf.get(i..).unwrap_or(&[]), self.eof))
    }

    fn release(&mut self, upto: u64) {
        self.released = self.released.max(upto);
    }
}

fn scanner(shared: &Shared, rx: Receiver<Vec<u8>>, jobs: std::sync::mpsc::Sender<Job>) {
    let src = ChannelSource {
        rx,
        buf: Vec::new(),
        base: 0,
        released: 0,
        eof: false,
        shared,
    };
    let mut sc = Scanner::new(src);
    let finish = |res: Option<Error>| {
        let mut st = shared.lock();
        let at = st.issued;
        match res {
            Some(e) if e.kind != ErrorKind::Cancelled => {
                st.ready.insert(at, Err(e));
                st.issued += 1;
                st.end = Some(at + 1);
            }
            _ => st.end = Some(at),
        }
        drop(st);
        shared.cv.notify_all();
    };
    loop {
        // a slot first, and a symbols buffer to reuse
        mark!(SlotWait);
        let syms = {
            let mut st = shared.lock();
            while !st.cancelled && st.issued - st.taken >= shared.slots {
                st = shared.cv.wait(st).unwrap_or_else(|e| e.into_inner());
            }
            if st.cancelled {
                drop(st);
                return finish(None);
            }
            st.free_syms.pop().unwrap_or_default()
        };
        mark!(SlotGot);
        let next = catch_unwind(AssertUnwindSafe(|| sc.next(syms)))
            .unwrap_or_else(|p| Err(Error::new(ErrorKind::Internal(panic_message(p)))));
        match next {
            Ok(Event::Block {
                index,
                bit,
                symbols,
                ..
            }) => {
                let mut st = shared.lock();
                let place = st.issued;
                st.issued += 1;
                drop(st);
                mark!(
                    Issued,
                    place,
                    std::mem::size_of_val(&symbols.syms[..]),
                    symbols.syms.capacity() * std::mem::size_of::<u16>()
                );
                debug_assert_eq!(place, index);
                if jobs.send((place, bit, symbols)).is_err() {
                    return finish(None);
                }
                mark!(Enqueued, place);
            }
            Ok(Event::Done) => return finish(None),
            Err(e) => return finish(Some(e)),
        }
    }
    // (the input's receiver and the jobs' sender are dropped here: a writer's next write
    // fails, and the workers end once the jobs queued are done)
}

fn worker<I: super::bwt::InverseBwt>(shared: &Shared, jobs: &Mutex<Receiver<Job>>, k: Kernels) {
    let mut scratch = Scratch::<I>::default().gated(shared.gate.clone());
    loop {
        mark!(JobLock);
        let job = {
            let rx = jobs.lock().unwrap_or_else(|e| e.into_inner());
            mark!(JobRecv);
            rx.recv()
        };
        let Ok((place, bit, symbols)) = job else {
            // (its buffers' high-water marks, for the footprint ledger)
            #[cfg(feature = "trace")]
            if crate::trace::level() > 0 {
                crate::trace::buffers(&scratch.buffers());
            }
            return;
        };
        mark!(JobGot, place);
        let mut out = {
            let mut st = shared.lock();
            if st.cancelled {
                continue; // (drain the queue, so the scanner never waits on it)
            }
            st.free_out.pop().unwrap_or_default()
        };
        out.clear();
        mark!(DecodeStart, place);
        let res = catch_unwind(AssertUnwindSafe(|| {
            decode_block(&symbols, k, &mut scratch, &mut out)
        }));
        mark!(DecodeEnd, place, out.len(), out.capacity());
        let res = match res {
            Ok(Ok(_)) => Ok(out),
            Ok(Err(kind)) => Err(kind),
            Err(p) => {
                scratch = Scratch::default().gated(shared.gate.clone());
                Err(ErrorKind::Internal(panic_message(p)))
            }
        }
        .map_err(|kind| Error {
            kind,
            block: Some(place),
            bit: Some(bit),
        });
        let mut st = shared.lock();
        if !st.cancelled {
            st.ready.insert(place, res);
        }
        if st.free_syms.len() < shared.slots as usize {
            st.free_syms.push(symbols.syms);
        }
        drop(st);
        mark!(Ready, place);
        shared.cv.notify_all();
    }
}

fn panic_message(p: Box<dyn std::any::Any + Send>) -> String {
    p.downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| p.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic".into())
}

/// How a decoder spends its threads: production's one block a worker, or teams of `k`
/// threads each inverting one block together (`bwt::Cooperative<k>`), so `teams` tables
/// are live on `teams × k` threads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    Workers(usize),
    Teams { teams: usize, k: usize },
}

/// The fewest physical cores the cooperative plan engages on: at 4 production is at its
/// peak on the i9 (docs/journal/2026-10-28-workstation-scaling.md), and CI's 4 vCPUs are 2
/// cores
pub const COOP_MIN_CORES: usize = 6;

/// A full block's T-vector: 900,000 rows × 4 bytes
const FULL_TABLE: f64 = 3.6e6;

/// The highest level whose streams the policy plans teams on every physical core for (the
/// level policy's rule R1, held out at levels 1-3 on the i9:
/// docs/journal/2026-10-29-level-policy.md). Higher levels keep `Plan::for_threads`.
pub const SMALL_LEVEL: u32 = 3;

/// The rows a block of a stream at `SMALL_LEVEL` can hold (a level's blocks are at most
/// level × 100 000 bytes before the BWT): larger blocks go through the level policy's gate
const SMALL_ROWS: usize = SMALL_LEVEL as usize * 100_000;

/// A stream header's level, from the input's first bytes (`BZh1`-`BZh9`), if they are one
pub(crate) fn header_level(head: &[u8]) -> Option<u32> {
    match head {
        [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => Some((l - b'0') as u32),
        _ => None,
    }
}

impl Plan {
    /// The cooperative decoder's policy for a caller's `threads`, on this machine, and the
    /// default (`StreamDecoder::new`, so `BzReader::parallel` and `decode_parallel`), as
    /// validated by the decoder A/B (bench/coop_decoder.py; on the i9,
    /// docs/journal/2026-10-29-cooperative-decoder.md):
    /// - where the machine's topology can't be read (only Linux's is, so far), or fewer
    ///   than `COOP_MIN_CORES` physical cores serve the threads, production's `threads`
    ///   workers, as today;
    /// - else teams of 2 on the physical cores the threads can use (never SMT siblings), as
    ///   many as keep the live tables near 0.7 of the L3: the knee measured on the i9 at
    ///   two block sizes (workstation journal §9), 4 teams there.
    pub fn for_threads(threads: usize) -> Plan {
        plan_with(threads.max(1), physical_cores(), l3_bytes())
    }

    /// `for_threads`, knowing the input's first stream's level (`BZh1`-`BZh9`; None: not
    /// known, and `for_threads`' plan). Where that plans teams and the level is at most
    /// `SMALL_LEVEL`, teams of 2 on all the physical cores the threads can use, with
    /// `for_threads`' team count kept as the bound on blocks larger than `SMALL_LEVEL`'s
    /// inverted at once, for a later stream of a concatenation with larger blocks
    /// (`StreamDecoder::for_stream`). Levels above `SMALL_LEVEL` plan as `for_threads`.
    ///
    /// At levels 1 and 2 the blocks are below `bwt::MIN_COOP` rows, so each team inverts
    /// its block on one core and its helper never starts: those teams are one-core
    /// workers, more of them than `for_threads` plans. The helpers start only at a larger
    /// block, which the bound then admits as `for_threads` would.
    pub fn for_stream(threads: usize, level: Option<u32>) -> Plan {
        plan_for_stream(threads.max(1), physical_cores(), l3_bytes(), level).0
    }

    /// The decoder's worker threads (teams count as one each) and its threads in all
    pub fn workers(&self) -> usize {
        match *self {
            Plan::Workers(n) => n.max(1),
            Plan::Teams { teams, .. } => teams.max(1),
        }
    }

    pub fn threads(&self) -> usize {
        match *self {
            Plan::Workers(n) => n.max(1),
            Plan::Teams { teams, k } => teams.max(1) * k.max(1),
        }
    }
}

/// The blocks in flight for `plan` when a caller asked for `slots` on `threads` threads:
/// as many a worker as asked (a team counting as one worker), at least one each
fn window(plan: Plan, threads: usize, slots: usize) -> usize {
    match plan {
        Plan::Workers(_) => slots,
        Plan::Teams { teams, .. } => (slots.saturating_mul(teams) / threads.max(1)).max(teams),
    }
}

/// `Plan::for_threads` on a machine of `phys` physical cores and an L3 of `l3` bytes
pub(crate) fn plan_with(threads: usize, phys: Option<usize>, l3: Option<u64>) -> Plan {
    let Some(phys) = phys else {
        return Plan::Workers(threads);
    };
    let cores = threads.min(phys);
    if cores < COOP_MIN_CORES {
        return Plan::Workers(threads);
    }
    let cap = l3.map_or(4, |b| {
        ((0.7 * b as f64) / FULL_TABLE).round().max(1.0) as usize
    });
    Plan::Teams {
        teams: (cores / 2).min(cap).max(1),
        k: 2,
    }
}

/// `Plan::for_stream` on a machine of `phys` physical cores and an L3 of `l3` bytes, with the
/// gate bounding its large blocks where it plans more teams than `plan_with` does
fn plan_for_stream(
    threads: usize,
    phys: Option<usize>,
    l3: Option<u64>,
    level: Option<u32>,
) -> (Plan, Option<Gate>) {
    let plan = plan_with(threads, phys, l3);
    if let (Plan::Teams { teams, k: 2 }, Some(level), Some(phys)) = (plan, level, phys) {
        let wide = threads.min(phys) / 2;
        if level <= SMALL_LEVEL && wide > teams {
            return (
                Plan::Teams { teams: wide, k: 2 },
                Some(Gate::new(SMALL_ROWS, teams)),
            );
        }
    }
    (plan, None)
}

/// The physical cores of the CPUs this process may use, where the OS says (Linux: the CPUs
/// in this process's affinity, `Cpus_allowed_list`, grouped by their sysfs package and
/// core ids), so a process limited to some CPUs counts only their cores
fn physical_cores() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let allowed = status
            .lines()
            .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))?
            .trim()
            .to_string();
        let mut cores = std::collections::BTreeSet::new();
        for part in allowed.split(',') {
            let (a, b) = part.split_once('-').unwrap_or((part, part));
            let (a, b): (usize, usize) = (a.parse().ok()?, b.parse().ok()?);
            for c in a..=b {
                let t = format!("/sys/devices/system/cpu/cpu{c}/topology");
                let Ok(core) = std::fs::read_to_string(format!("{t}/core_id")) else {
                    continue; // (an allowed CPU that is offline)
                };
                let pkg = std::fs::read_to_string(format!("{t}/physical_package_id")).ok()?;
                cores.insert((pkg.trim().to_string(), core.trim().to_string()));
            }
        }
        (!cores.is_empty()).then_some(cores.len())
    }
    #[cfg(not(target_os = "linux"))]
    None
}

/// The last-level cache's size, where the OS says (Linux: cpu0's level-3 cache)
fn l3_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let s = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index3/size").ok()?;
        let s = s.trim();
        let (num, mult) = match s.as_bytes().last()? {
            b'K' => (&s[..s.len() - 1], 1u64 << 10),
            b'M' => (&s[..s.len() - 1], 1 << 20),
            _ => (s, 1),
        };
        num.parse::<u64>().ok().map(|n| n * mult)
    }
    #[cfg(not(target_os = "linux"))]
    None
}

#[cfg(test)]
mod plan_tests {
    use super::*;

    #[test]
    fn the_policy() {
        let i9 = (Some(10), Some(20 << 20));
        // below 6 cores, or no topology: production's workers, as today
        assert_eq!(plan_with(4, i9.0, i9.1), Plan::Workers(4));
        assert_eq!(plan_with(5, i9.0, i9.1), Plan::Workers(5));
        assert_eq!(plan_with(8, None, i9.1), Plan::Workers(8));
        assert_eq!(plan_with(20, Some(4), i9.1), Plan::Workers(20));
        // the i9: teams of 2, at most 4 (0.7 × 20 MiB over a full table)
        assert_eq!(plan_with(6, i9.0, i9.1), Plan::Teams { teams: 3, k: 2 });
        assert_eq!(plan_with(8, i9.0, i9.1), Plan::Teams { teams: 4, k: 2 });
        assert_eq!(plan_with(10, i9.0, i9.1), Plan::Teams { teams: 4, k: 2 });
        assert_eq!(plan_with(20, i9.0, i9.1), Plan::Teams { teams: 4, k: 2 });
        // a bigger cache, more teams; no cache size read, 4
        assert_eq!(
            plan_with(16, Some(16), Some(32 << 20)),
            Plan::Teams { teams: 7, k: 2 }
        );
        assert_eq!(
            plan_with(12, Some(12), None),
            Plan::Teams { teams: 4, k: 2 }
        );
        let p = Plan::Teams { teams: 4, k: 2 };
        assert_eq!((p.workers(), p.threads()), (4, 8));
    }

    /// The default's window: what the A/B ran (2 a team at 2 a thread asked), and at least
    /// a block a team
    #[test]
    fn the_window() {
        let t4 = Plan::Teams { teams: 4, k: 2 };
        assert_eq!(window(Plan::Workers(4), 4, 8), 8);
        assert_eq!(window(t4, 20, 40), 8);
        assert_eq!(window(t4, 8, 16), 8);
        assert_eq!(window(t4, 20, 1), 4);
        assert_eq!(window(t4, 20, usize::MAX), usize::MAX / 20);
    }

    /// The default decoder's threads are the plan's for this machine
    #[test]
    fn the_default_is_the_plan() {
        for t in [1, 4, 8] {
            let dec = StreamDecoder::new(t, 2 * t, Kernels::detect());
            assert_eq!(dec.threads(), Plan::for_threads(t).threads());
            dec.close();
            let e = dec.read(&mut [0; 16]).unwrap_err();
            assert_eq!(e.kind, ErrorKind::Empty);
        }
    }

    /// The level policy (R1) on the i9: teams of 2 on every core at levels 1-3, with
    /// `for_threads`' 4 teams the bound on larger blocks; levels 4-9, no level, fewer cores
    /// or no topology plan as `for_threads`, with no bound
    #[test]
    fn the_level_policy() {
        let i9 = (Some(10), Some(20 << 20));
        let at = |t, l| {
            let (p, g) = plan_for_stream(t, i9.0, i9.1, l);
            (p, g.map(|g| (g.rows, g.permits)))
        };
        let wide = (Plan::Teams { teams: 5, k: 2 }, Some((300_000, 4)));
        for l in 1..=3 {
            assert_eq!(at(20, Some(l)), wide);
            assert_eq!(at(10, Some(l)), wide);
            // fewer threads: no more teams than for_threads' plan, so no bound
            assert_eq!(at(8, Some(l)), (Plan::Teams { teams: 4, k: 2 }, None));
            assert_eq!(at(6, Some(l)), (Plan::Teams { teams: 3, k: 2 }, None));
            assert_eq!(at(4, Some(l)), (Plan::Workers(4), None));
        }
        for l in [None, Some(4), Some(5), Some(6), Some(9)] {
            for t in [1, 4, 6, 8, 10, 20] {
                assert_eq!(
                    at(t, l),
                    (plan_with(t, i9.0, i9.1), None),
                    "{t} threads, {l:?}"
                );
            }
        }
        // no topology, or below 6 physical cores: workers at any level, as for_threads
        assert_eq!(
            plan_for_stream(20, None, i9.1, Some(1)).0,
            Plan::Workers(20)
        );
        assert_eq!(
            plan_for_stream(20, Some(4), i9.1, Some(1)).0,
            Plan::Workers(20)
        );
        // an L3 whose cap already plans a team a core pair: no change
        let big = plan_for_stream(16, Some(16), Some(64 << 20), Some(1));
        assert_eq!(
            (big.0, big.1.is_none()),
            (Plan::Teams { teams: 8, k: 2 }, true)
        );
        assert_eq!(header_level(b"BZh9"), Some(9));
        assert_eq!(header_level(b"BZh1\x31AY"), Some(1));
        for h in [&b"BZh0"[..], b"BZh", b"", b"BZx9", b"bzh9"] {
            assert_eq!(header_level(h), None, "{h:?}");
        }
    }

    /// The gate admits at most its permits of blocks above its rows at once and smaller
    /// ones freely, and a pass held through a panic is given back
    #[test]
    fn the_gate() {
        use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
        let g = Gate::new(100, 2);
        assert!(g.enter(100).is_none());
        let (now, most) = (AtomicUsize::new(0), AtomicUsize::new(0));
        std::thread::scope(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    for _ in 0..200 {
                        let _pass = g.enter(101).expect("a large block's pass");
                        let n = now.fetch_add(1, SeqCst) + 1;
                        most.fetch_max(n, SeqCst);
                        std::thread::yield_now();
                        now.fetch_sub(1, SeqCst);
                    }
                });
            }
        });
        assert!(
            most.load(SeqCst) <= 2,
            "{} inside at once",
            most.load(SeqCst)
        );
        assert_eq!(*g.inside.lock().unwrap(), 0);
        let r = catch_unwind(AssertUnwindSafe(|| {
            let _pass = g.enter(101);
            panic!("a panic holding a pass");
        }));
        assert!(r.is_err());
        assert_eq!(*g.inside.lock().unwrap(), 0);
    }

    fn compress(data: &[u8], level: u32) -> Vec<u8> {
        use std::io::Write;
        let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(level));
        e.write_all(data).unwrap();
        e.finish().unwrap()
    }

    /// Words of random letters (few RLE1 runs, so a block's rows are about its bytes)
    fn words(n: usize, seed: u64) -> Vec<u8> {
        let mut x = seed | 1;
        let mut v = Vec::with_capacity(n);
        while v.len() < n {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            v.push(if x % 7 == 0 {
                b' '
            } else {
                b'a' + (x % 26) as u8
            });
        }
        v
    }

    /// `data` written into `dec` from another thread and read out whole
    fn through(dec: StreamDecoder, data: &[u8]) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        std::thread::scope(|s| {
            s.spawn(|| {
                for c in data.chunks(64 << 10) {
                    if dec.write(c.to_vec()).is_err() {
                        return;
                    }
                }
                dec.close();
            });
            let mut buf = vec![0u8; 1 << 16];
            loop {
                match dec.read(&mut buf) {
                    Ok(0) => break Ok(out),
                    Ok(n) => out.extend_from_slice(&buf[..n]),
                    Err(e) => break Err(e),
                }
            }
        })
    }

    /// Concatenations of a -3 stream (blocks under 300 000 rows) and a -9 one (larger),
    /// in both orders, decode to the serial decoder's bytes under the level policy's
    /// bound (one permit: large blocks wait for each other) on teams and on workers, and
    /// through `BzReader::parallel`'s plan for this machine
    #[test]
    fn mixed_levels_decode_whole() {
        use std::io::Read;
        let small = compress(&words(700_000, 3), 3);
        let large = compress(&words(1_100_000, 9), 9);
        for (name, data) in [
            ("-3 then -9", [&small[..], &large[..]].concat()),
            ("-9 then -3", [&large[..], &small[..]].concat()),
        ] {
            let want = crate::decode(&data).unwrap();
            for plan in [Plan::Teams { teams: 3, k: 2 }, Plan::Workers(3)] {
                let dec = StreamDecoder::planned_gated(
                    plan,
                    6,
                    Kernels::detect(),
                    Some(Gate::new(SMALL_ROWS, 1)),
                );
                let got = through(dec, &data).unwrap();
                assert!(
                    got == want,
                    "{name}, {plan:?}: not the serial decoder's bytes"
                );
            }
            let mut got = Vec::new();
            crate::BzReader::parallel(std::io::Cursor::new(data.clone()), 20)
                .read_to_end(&mut got)
                .unwrap();
            assert!(
                got == want,
                "{name}, BzReader::parallel: not the serial decoder's bytes"
            );
            assert!(
                crate::decode_parallel(&data, 20).unwrap() == want,
                "{name}, decode_parallel"
            );
        }
    }

    /// Reading the header first changes no error: an empty input, a short one, one not a
    /// .bz2, and a source failing at its first read fail as the serial reader does
    #[test]
    fn the_header_read_keeps_the_errors() {
        use std::io::Read;
        fn kind(r: std::io::Result<usize>) -> Option<ErrorKind> {
            let e = r.expect_err("an error");
            e.get_ref()
                .and_then(|i| i.downcast_ref::<Error>())
                .map(|e| e.kind.clone())
        }
        for input in [&b""[..], b"BZ", b"BZh", b"hello, world", b"BZh9\x31AY&SY"] {
            let (mut a, mut b) = (Vec::new(), Vec::new());
            let serial = kind(crate::BzReader::new(input).read_to_end(&mut a));
            let par = kind(
                crate::BzReader::parallel(std::io::Cursor::new(input.to_vec()), 4)
                    .read_to_end(&mut b),
            );
            assert_eq!(serial, par, "{input:?}");
            assert!(a == b, "{input:?}: the bytes before the error");
        }
        struct Failing;
        impl Read for Failing {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("the source failed"))
            }
        }
        let e = crate::BzReader::parallel(Failing, 4)
            .read_to_end(&mut Vec::new())
            .unwrap_err();
        assert_eq!(e.to_string(), "the source failed");
    }

    /// Whatever this machine is, the plan is one of the two shapes and never more threads
    /// than asked
    #[test]
    fn the_policy_here() {
        if cfg!(target_os = "linux") {
            let p = physical_cores().expect("Linux's topology");
            let logical = std::thread::available_parallelism().unwrap().get();
            assert!(p >= 1 && p <= logical, "{p} cores of {logical} CPUs");
        }
        for t in [1, 2, 4, 6, 8, 64] {
            let p = Plan::for_threads(t);
            assert!(p.threads() <= t.max(1), "{t}: {p:?}");
        }
    }
}
