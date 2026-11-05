//! The block-parallel streaming decoder: compressed bytes written in (`StreamDecoder::write`, from
//! any thread, blocking while the input queued is full), decoded bytes read out in order
//! (`StreamDecoder::read`). Like lbzip2's, its parallelism is across blocks:
//!
//! - a **scanner** thread reads the stream in order (stream.rs): each block's header and
//!   Huffman-coded symbols (the one phase that must be sequential: a block's end is
//!   known only once they are read), and each stream's end and combined CRC;
//! - `threads` **workers** finish a block each (move-to-front, the inverse BWT, RLE1 and
//!   the block's CRC: `decode_block`), taking them in order from a queue whose idle
//!   workers wait without holding its lock (`JobQueue`);
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
use std::collections::{BTreeMap, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;

/// Input chunks queued for the scanner (the split writes 8 MB at a time)
pub(crate) const INPUT_QUEUE: usize = 2;
/// The blocks in flight a thread where the caller gives no window (`BzReader::parallel`,
/// `decode_parallel`): how far the scanner may run ahead of the reader
pub const SLOTS_PER_THREAD: usize = 4;
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

/// The jobs' queue, from the scanner to the workers in the order sent, with a channel's
/// ends: the sender's drop (the scanner's return, or its unwinding) closes it, and a send
/// fails once every worker's end is dropped. A worker waits for a job on the condition
/// variable, whose wait gives the lock up, so the lock is held only to push or pop, never
/// by a worker asleep. (A `Mutex<Receiver>` held across `recv` queued the idle workers on
/// the lock: a job went to its holder, which had to be woken first, while a worker just
/// back from a block slept on the lock, and each holder's pass woke the next.)
struct JobQueue<T> {
    state: Mutex<Queued<T>>,
    cv: Condvar,
}

struct Queued<T> {
    jobs: VecDeque<T>,
    /// workers asleep in `recv`, so a send wakes one only if one waits
    waiting: usize,
    /// the workers' ends not yet dropped
    receivers: usize,
    /// the sender's end dropped: no job will come
    closed: bool,
}

impl<T> JobQueue<T> {
    fn lock(&self) -> MutexGuard<'_, Queued<T>> {
        // (nothing panics while the lock is held: a poisoned lock is still consistent)
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// The scanner's end of the jobs' queue
struct JobTx<T>(Arc<JobQueue<T>>);
/// A worker's end of the jobs' queue
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
    /// Queue a job, waking a waiting worker if there is one; the job back if no worker's
    /// end is left (as a channel's send fails once its receivers are gone)
    fn send(&self, job: T) -> Result<(), T> {
        let mut q = self.0.lock();
        if q.receivers == 0 {
            return Err(job);
        }
        q.jobs.push_back(job);
        let wake = q.waiting > 0;
        drop(q);
        // (a worker not waiting is decoding, or about to look at the queue under the lock:
        // it takes the job itself, with no wake-up)
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
    /// The next job in the order sent, waiting for one; None once the queue is closed
    /// and empty (the jobs queued before the close are taken first)
    fn recv(&self) -> Option<T> {
        mark!(JobLock);
        let mut q = self.0.lock();
        mark!(JobRecv);
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
        let threads = threads.max(1);
        let plan = Plan::for_threads(threads);
        Self::planned(plan, window(plan, threads, slots), k)
    }

    /// A decoder spending its threads as `plan` says: production's workers, or teams of
    /// k sharing each block (`bwt::Cooperative`), with `slots` blocks in flight (at least
    /// the plan's workers or teams). The team size is 2, 3 or 4; another runs as that many
    /// production workers.
    pub fn planned(plan: Plan, slots: usize, k: Kernels) -> StreamDecoder {
        use super::bwt::{Cooperative, Segmented};
        let mut dec = match plan {
            Plan::Workers(n) => Self::with_inverse::<Segmented>(n, slots, k),
            Plan::Teams { teams, k: 2 } => Self::with_inverse::<Cooperative<2>>(teams, slots, k),
            Plan::Teams { teams, k: 3 } => Self::with_inverse::<Cooperative<3>>(teams, slots, k),
            Plan::Teams { teams, k: 4 } => Self::with_inverse::<Cooperative<4>>(teams, slots, k),
            Plan::Teams { teams, k: t } => {
                Self::with_inverse::<Segmented>(teams.max(1) * t.max(1), slots, k)
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
        });
        let (in_tx, in_rx) = sync_channel::<Vec<u8>>(INPUT_QUEUE);
        let (job_tx, job_rx) = job_queue::<Job>();
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
        drop(job_rx); // (the workers hold the queue's receiving ends)
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

fn scanner(shared: &Shared, rx: Receiver<Vec<u8>>, jobs: JobTx<Job>) {
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

fn worker<I: super::bwt::InverseBwt>(shared: &Shared, jobs: &JobRx<Job>, k: Kernels) {
    let mut scratch = Scratch::<I>::default();
    loop {
        // (`recv` marks JobLock and JobRecv)
        let Some((place, bit, symbols)) = jobs.recv() else {
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
                scratch = Scratch::default();
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

/// The jobs' queue alone (`miri_` so CI's Miri job runs them under its race detector): the
/// order, a wake-up for every job sent to waiting workers, the close and the ends' drops.
/// A wake-up lost or a close not seen fails at `PATIENCE` rather than hanging.
#[cfg(test)]
mod queue_tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread::JoinHandle;
    use std::time::Duration;

    const PATIENCE: Duration = Duration::from_secs(120);

    fn settled<T>(q: &JobQueue<T>) -> (usize, usize, usize, bool) {
        let st = q.lock();
        (st.jobs.len(), st.waiting, st.receivers, st.closed)
    }

    /// `n` workers taking jobs from `rx` until the queue ends, each sending what it took
    fn workers(n: usize, rx: &JobRx<u32>) -> (mpsc::Receiver<Vec<u32>>, Vec<JoinHandle<()>>) {
        let (tx, got) = mpsc::channel();
        let hs = (0..n)
            .map(|_| {
                let (rx, tx) = (rx.clone(), tx.clone());
                std::thread::spawn(move || {
                    let _ = tx.send(std::iter::from_fn(|| rx.recv()).collect());
                })
            })
            .collect();
        (got, hs)
    }

    /// What each of the `n` workers took, once the queue has ended them all
    fn ended(got: mpsc::Receiver<Vec<u32>>, hs: Vec<JoinHandle<()>>) -> Vec<Vec<u32>> {
        let all = (0..hs.len())
            .map(|_| {
                got.recv_timeout(PATIENCE)
                    .expect("a worker the close never ended")
            })
            .collect();
        for h in hs {
            h.join().unwrap();
        }
        all
    }

    /// One worker takes the jobs in exactly the order sent
    #[test]
    fn miri_queue_order() {
        let n = if cfg!(miri) { 200 } else { 100_000 };
        let (tx, rx) = job_queue::<u32>();
        let q = tx.0.clone();
        let (got, hs) = workers(1, &rx);
        drop(rx);
        for j in 0..n {
            tx.send(j).unwrap();
        }
        drop(tx);
        assert_eq!(ended(got, hs), vec![(0..n).collect::<Vec<_>>()]);
        assert_eq!(settled(&q), (0, 0, 0, true));
    }

    /// Many workers: every job reaches exactly one, each worker's in the order sent, and
    /// the sender's drop ends them all once the queue is empty
    #[test]
    fn miri_queue_each_job_once() {
        let n = if cfg!(miri) { 300 } else { 200_000 };
        let (tx, rx) = job_queue::<u32>();
        let q = tx.0.clone();
        let (got, hs) = workers(4, &rx);
        drop(rx);
        for j in 0..n {
            tx.send(j).unwrap();
        }
        drop(tx);
        let got = ended(got, hs);
        for v in &got {
            assert!(
                v.windows(2).all(|w| w[0] < w[1]),
                "a worker's jobs out of order"
            );
        }
        let mut all: Vec<u32> = got.concat();
        all.sort_unstable();
        assert_eq!(all, (0..n).collect::<Vec<_>>());
        assert_eq!(settled(&q), (0, 0, 0, true));
    }

    /// Bursts of 1 to 7 jobs to 4 workers that are all waiting or on their way back: each
    /// burst is taken before the next is sent, so a job left queued with a worker asleep
    /// (a wake-up lost, or skipped by the `waiting` count) fails here
    #[test]
    fn miri_queue_wakes_waiting_workers() {
        let rounds = if cfg!(miri) { 30 } else { 20_000 };
        let (tx, rx) = job_queue::<u32>();
        let q = tx.0.clone();
        let (ack_tx, ack_rx) = mpsc::channel();
        let hs: Vec<_> = (0..4)
            .map(|_| {
                let (rx, ack) = (rx.clone(), ack_tx.clone());
                std::thread::spawn(move || {
                    while let Some(j) = rx.recv() {
                        ack.send(j).unwrap();
                    }
                    ack.send(u32::MAX).unwrap(); // (ended)
                })
            })
            .collect();
        drop((rx, ack_tx));
        for r in 0..rounds {
            let burst = 1 + r % 7;
            for j in 0..burst {
                tx.send(j).unwrap();
            }
            let mut seen = [false; 7];
            for _ in 0..burst {
                let j = ack_rx
                    .recv_timeout(PATIENCE)
                    .expect("a job queued while its workers slept");
                assert!(
                    !std::mem::replace(&mut seen[j as usize], true),
                    "job {j} twice"
                );
            }
        }
        drop(tx);
        for _ in 0..hs.len() {
            let end = ack_rx.recv_timeout(PATIENCE);
            assert_eq!(end, Ok(u32::MAX), "a worker the close never ended");
        }
        for h in hs {
            h.join().unwrap();
        }
        assert_eq!(settled(&q), (0, 0, 0, true));
    }

    /// The close wakes every waiting worker, after the jobs queued before it are taken
    #[test]
    fn miri_queue_close_wakes_waiting_workers() {
        for before in [0, 5] {
            let (tx, rx) = job_queue::<u32>();
            let q = tx.0.clone();
            let (got, hs) = workers(3, &rx);
            drop(rx);
            // (all three asleep in recv)
            while q.lock().waiting < 3 {
                std::thread::yield_now();
            }
            for j in 0..before {
                tx.send(j).unwrap();
            }
            drop(tx);
            let taken: usize = ended(got, hs).iter().map(Vec::len).sum();
            assert_eq!(taken, before as usize);
            assert_eq!(settled(&q), (0, 0, 0, true));
        }
    }

    /// A send fails, the job handed back, once every worker's end is gone, a worker's
    /// unwinding included (as the channel's did); and the sender's unwinding closes it
    #[test]
    fn miri_queue_ends_dropped() {
        let (tx, rx) = job_queue::<u32>();
        let other = rx.clone();
        tx.send(1).unwrap();
        drop(rx);
        tx.send(2).unwrap();
        let h = std::thread::spawn(move || {
            assert_eq!(other.recv(), Some(1));
            panic!("a worker's panic outside catch_unwind");
        });
        assert!(h.join().is_err());
        assert_eq!(tx.send(3), Err(3));

        let (tx, rx) = job_queue::<u32>();
        let h = std::thread::spawn(move || {
            tx.send(7).unwrap();
            panic!("the scanner's panic");
        });
        assert!(h.join().is_err());
        assert_eq!(rx.recv(), Some(7));
        assert_eq!(rx.recv(), None);
    }
}
