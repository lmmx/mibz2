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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Condvar, Mutex};
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
    /// the decode's terminal failure, if it failed as a whole (the service core's outer
    /// unwind, `Hub::fail`): set with `cancelled`, before either is announced, and the
    /// reader's error in place of `Cancelled`
    terminal: Option<Error>,
}

impl State {
    /// The reader's error once the decode is cancelled: the terminal failure, if the decode
    /// failed as a whole, else `Cancelled`
    fn stopped(&self) -> Error {
        self.terminal
            .clone()
            .unwrap_or_else(|| Error::new(ErrorKind::Cancelled))
    }
}

struct Shared {
    state: Mutex<State>,
    cv: Condvar,
    slots: u64,
    /// the pipelined teams' counts (`StreamDecoder::pipeline_counts`): jobs taken by a
    /// leader while its helpers inverted another block, and those of them that failed
    overlapped: AtomicU64,
    overlap_errors: AtomicU64,
    /// testing only: the pipelined team's injected faults, and what they left
    #[cfg(test)]
    faults: Mutex<Faults>,
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
    /// its workers are pipelined teams
    pipelined: bool,
    /// the service core's hub, if it is the service core (`service`, or the default at
    /// `SERVICE_PLAN`): for `is_service` and `service_counts`
    hub: Option<Arc<Hub>>,
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
    /// cores, teams sharing each block, with the window scaled to the teams (`window`).
    /// At the measured plan, three teams of three (`PIPELINED_PLAN`), the teams are
    /// pipelined, with park and small (`PIPELINED_LEVERS`); every other plan runs as
    /// `planned` runs it. `planned(Plan::Workers(threads), ..)` is the decoder before the
    /// policy, and `planned(PIPELINED_PLAN, ..)` the three plain teams of three.
    ///
    /// One narrower exception comes first: at exactly `SERVICE_THREADS` threads on the
    /// service core's supported target (`service_target`: the measured i9-10900K's CPU and
    /// shape, with its own kernels), the decoder is the service core at `SERVICE_PLAN`
    /// (two teams of three that only build and walk, two service workers), its window
    /// scaled to its two teams as a plan's is (`window`: 8 blocks at the 40 a caller of
    /// `BzReader::parallel` or `decode_parallel` gives at 10 threads), its lookahead one
    /// block a team (2). A plan forced with `with_plan` is never replaced by it.
    pub fn new(threads: usize, slots: usize, k: Kernels) -> StreamDecoder {
        let threads = threads.max(1);
        let forced = FORCED.with(|c| c.get()).is_some();
        // (the cheap conditions first: the machine is read only at the target's budget)
        let dec = if !forced
            && threads == SERVICE_THREADS
            && k.avx2
            && k.clmul
            && service_target(threads, k, &Machine::here())
        {
            let p = SERVICE_PLAN;
            let slots = window(p.as_plan(), threads, slots);
            Self::service(p.teams, p.k, p.services, slots, k)
        } else {
            let plan = Plan::for_threads(threads);
            Self::defaulted(plan, window(plan, threads, slots), k)
        };
        record_built(&dec);
        dec
    }

    /// Whether this decoder is the service core (`service`, or `new` at `SERVICE_PLAN`)
    #[doc(hidden)]
    pub fn is_service(&self) -> bool {
        self.hub.is_some()
    }

    /// The blocks this decoder keeps in flight (its window, after any scaling to its teams)
    #[doc(hidden)]
    pub fn window(&self) -> u64 {
        self.shared.slots
    }

    /// The service core's counts so far (None for any other decoder): blocks prepared,
    /// inverted on a team's pool, and finished. Research and tests only.
    #[doc(hidden)]
    pub fn service_counts(&self) -> Option<ServiceCounts> {
        self.hub.as_ref().map(|h| h.counts())
    }

    /// The default decoder for `plan` (`new`'s): the pipelined teams with
    /// `PIPELINED_LEVERS` at exactly `PIPELINED_PLAN`, else `planned`
    fn defaulted(plan: Plan, slots: usize, k: Kernels) -> StreamDecoder {
        match plan {
            Plan::Teams { teams, k: team_k } if plan == PIPELINED_PLAN => {
                Self::pipelined_levers(teams, team_k, slots, k, PIPELINED_LEVERS)
            }
            _ => Self::planned(plan, slots, k),
        }
    }

    /// Whether this decoder runs pipelined teams (`pipelined`, `pipelined_levers`, or the
    /// default at `PIPELINED_PLAN`)
    #[doc(hidden)]
    pub fn is_pipelined(&self) -> bool {
        self.pipelined
    }

    /// A decoder spending its threads as `plan` says: production's workers, or teams of
    /// k sharing each block (`bwt::Cooperative`), with `slots` blocks in flight (at least
    /// the plan's workers or teams). The team size is 2, 3 or 4; another runs as that many
    /// production workers.
    pub fn planned(plan: Plan, slots: usize, k: Kernels) -> StreamDecoder {
        use super::bwt::{Cells, Cooperative, Segmented};
        let mut dec = match plan {
            // a worker inverts whole blocks, as the calling thread does: Cells, with its
            // admission gates and Segmented fallback
            Plan::Workers(n) => Self::with_inverse::<Cells>(n, slots, k),
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
        Self::launch(threads, slots, move |shared, jobs| {
            worker::<I>(shared, jobs, k)
        })
    }

    /// Research, opt-in, not the default: `teams` pipelined teams of `k` threads (at least
    /// 2), each overlapping its leader's serial stages (a block's RLE1 and CRC, the next
    /// block's move-to-front) with its helpers' build and walk of the block between, on
    /// one table a team, with `slots` blocks in flight (at least `teams`). Byte for byte
    /// the default decoder's output and errors.
    #[doc(hidden)]
    pub fn pipelined(teams: usize, k: usize, slots: usize, kernels: Kernels) -> StreamDecoder {
        let (teams, team_k) = (teams.max(1), k.max(2));
        let mut dec = Self::launch(teams, slots, move |shared, jobs| {
            pipelined_worker::<false, false, { super::coop::BARRIER_SPINS }>(
                shared, jobs, kernels, team_k,
            )
        });
        dec.threads = teams * team_k;
        dec.pipelined = true;
        dec
    }

    /// Research, opt-in, not the default: `pipelined` with its levers (`Levers`), each
    /// separately. With no lever set, it is `pipelined` exactly (the registered prototype).
    /// Byte for byte the default decoder's output and errors, whatever the levers.
    #[doc(hidden)]
    pub fn pipelined_levers(
        teams: usize,
        k: usize,
        slots: usize,
        kernels: Kernels,
        levers: Levers,
    ) -> StreamDecoder {
        use super::coop::{BARRIER_SPINS as LONG, SHORT_SPINS as SHORT};
        let (teams, team_k) = (teams.max(1), k.max(2));
        macro_rules! launch {
            ($park:literal, $small:literal, $spin:ident) => {
                Self::launch(teams, slots, move |shared, jobs| {
                    pipelined_worker::<$park, $small, $spin>(shared, jobs, kernels, team_k)
                })
            };
        }
        let mut dec = match (levers.park, levers.small, levers.spin2k) {
            (false, false, false) => launch!(false, false, LONG),
            (true, false, false) => launch!(true, false, LONG),
            (false, true, false) => launch!(false, true, LONG),
            (true, true, false) => launch!(true, true, LONG),
            (false, false, true) => launch!(false, false, SHORT),
            (true, false, true) => launch!(true, false, SHORT),
            (false, true, true) => launch!(false, true, SHORT),
            (true, true, true) => launch!(true, true, SHORT),
        };
        dec.threads = teams * team_k;
        dec.pipelined = true;
        dec
    }

    /// The pipelined teams' counts so far: jobs a leader took while its helpers inverted
    /// another block, and those of them whose move-to-front failed (both 0 for the other
    /// decoders). Research only.
    #[doc(hidden)]
    pub fn pipeline_counts(&self) -> (u64, u64) {
        (
            self.shared.overlapped.load(Ordering::Relaxed),
            self.shared.overlap_errors.load(Ordering::Relaxed),
        )
    }

    /// The scanner, and `threads` workers each running `work`
    fn launch<W>(threads: usize, slots: usize, work: W) -> StreamDecoder
    where
        W: Fn(&Shared, &Mutex<Receiver<Job>>) + Clone + Send + 'static,
    {
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
                terminal: None,
            }),
            cv: Condvar::new(),
            slots,
            overlapped: AtomicU64::new(0),
            overlap_errors: AtomicU64::new(0),
            #[cfg(test)]
            faults: Mutex::default(),
        });
        let (in_tx, in_rx) = sync_channel::<Vec<u8>>(INPUT_QUEUE);
        let (job_tx, job_rx) = channel::<Job>();
        let job_rx = Arc::new(Mutex::new(job_rx));
        let mut handles = Vec::with_capacity(threads + 1);
        for i in 0..threads {
            let (shared, job_rx, work) = (shared.clone(), job_rx.clone(), work.clone());
            handles.push(
                std::thread::Builder::new()
                    .name(format!("mibz2-{i}"))
                    .spawn(move || work(&shared, &job_rx))
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
            pipelined: false,
            hub: None,
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
        if !r.done && r.failed.is_none() {
            let st = self.shared.lock();
            if st.cancelled {
                r.failed = Some(st.stopped());
            }
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
                    r.failed = Some(st.stopped());
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
    let mut scratch = Scratch::<I>::default();
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
                crate::trace::allocations(std::mem::take(&mut scratch.allocation_history));
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

// ---- The pipelined team (research, opt-in: `StreamDecoder::pipelined`) ----
//
// A team of k threads whose leader overlaps its serial stages with the helpers' parallel
// ones, on different blocks. In each run of the team's pool (`coop::invert_pipelined`):
// - helpers 1..k build and walk block i on the team's one table, and assemble it;
// - the leader, meanwhile: block i − 1's RLE1 and CRC (from its assembled buffer) and its
//   result published; the next job taken, if one is queued (never waiting for one); its
//   move-to-front into the second front set (a block below `MIN_COOP` is then decoded
//   whole by the leader, on a one-core kernel's buffers of its own, rather than draining
//   the pipeline: on the Wikidata sample, 198 of its 431 blocks are below it); then it
//   joins block i's walk if it is still going.
// So a team keeps two front sets (`ll`, `counts`, `checkpoints`; or, for a block of few
// symbols, its run records and their cuts: `front_stage`) and two assembled buffers, one
// table, pool and segment scratch, and the one-core kernel's small set.
//
// A block of few symbols (`runs::takes_runs`) goes through the same run by its records:
// the leader's front is `runs::front` and `runs::cuts`, the helpers' build is the fill of
// the LF table by shares (`coop::RunsBuild`), and their walk is the same walk from the
// primary index, its segments joined in reverse (`coop::Source`).
//
// The worker never waits for a job while it holds a block not yet published: a job is
// waited for (`recv`) only with the pipeline empty, so a block the reader needs can never
// sit behind a team waiting for the scanner, which may be waiting for the reader's slot.

/// The pipelined team's levers (research, opt-in: `StreamDecoder::pipelined_levers`),
/// each off unless set, so that each can be measured alone and together
/// (docs/journal/2026-11-07-pipeline-levers-registration.md). Neither changes any output
/// or error.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Levers {
    /// Lever A: a thread waiting for the build's end (a helper at the helpers' barrier,
    /// the leader waiting to join the walk) spins its budget, then parks until the build's
    /// last helper unparks it, as the pool's own waits park, rather than yielding in a loop
    /// (`coop::invert_pipelined_with`'s `PARK`)
    pub park: bool,
    /// The spin-budget control: that wait's budget is `coop::SHORT_SPINS` (2,000, the
    /// pool's) rather than the registered `coop::BARRIER_SPINS` (4,096), whether it then
    /// yields or parks (`coop::invert_pipelined_with`'s `SPIN`)
    pub spin2k: bool,
    /// Lever B: after the leader decodes a block below `MIN_COOP` inside a run, it takes
    /// the next queued job too (never waiting for one), and so on until it stages a block
    /// for the team or finds none queued, so a small block no longer leaves the next run
    /// with nothing staged (`pipelined_worker`'s `SMALL`)
    pub small: bool,
}

/// A block's move-to-front stage's buffers (`Scratch`'s first three), and, for a block of
/// few symbols (`runs::takes_runs`), its run records and their cuts in their place
/// (`front_stage`)
struct Front {
    ll: Vec<u8>,
    counts: [u32; 256],
    checkpoints: Vec<[u32; 256]>,
    runs: super::runs::Runs,
    cuts: Vec<super::runs::Cut>,
    /// whether the block staged here is in `runs` and `cuts` (else in the first three), and
    /// its rows
    by_runs: bool,
    rows: usize,
}

impl Default for Front {
    fn default() -> Self {
        Front {
            ll: Vec::new(),
            counts: [0; 256],
            checkpoints: Vec::new(),
            runs: super::runs::Runs::default(),
            cuts: Vec::new(),
            by_runs: false,
            rows: 0,
        }
    }
}

impl Front {
    /// The block staged here, as a team inverts it
    fn source(&self, k: Kernels) -> super::coop::Source<'_> {
        if self.by_runs {
            super::coop::Source::Runs {
                runs: &self.runs,
                cuts: &self.cuts,
                n: self.rows,
                k,
            }
        } else {
            super::coop::Source::Column {
                ll: &self.ll,
                counts: &self.counts,
                checkpoints: &self.checkpoints,
                stride: super::mtf::CHECKPOINT,
            }
        }
    }

    /// The block staged here inverted on this thread alone, on a one-core kernel's table:
    /// `bwt::walk_segments_room_parts`'s parts, or, for a block in its records,
    /// `bwt::walk_back_room_parts`'s (the same, joined by `assemble_reversed`). Records that
    /// don't tile the block, which `front_stage` refuses, panic here.
    fn invert_alone(
        &self,
        orig_ptr: u32,
        k: Kernels,
        tt: &mut super::bwt::TVector,
        seg: &mut super::bwt::SegScratch,
        direct: &mut Vec<u8>,
    ) -> bool {
        if self.by_runs {
            assert!(
                super::runs::fill(&self.runs, self.rows, tt, k),
                "the run records don't tile the block"
            );
            super::bwt::walk_back_room_parts(tt, orig_ptr, direct, seg)
        } else {
            super::bwt::build(&self.ll, &self.counts, tt);
            super::bwt::walk_segments_room_parts(tt, orig_ptr, direct, seg)
        }
    }
}

/// A pipelined team's buffers, kept from block to block
#[derive(Default)]
struct Team {
    fronts: [Front; 2],
    /// the inverse BWT's assembled output: one block's being written while the other's
    /// RLE1 is undone
    bufs: [Vec<u8>; 2],
    tt: super::bwt::TVector,
    seg: super::bwt::SegScratch,
    /// None until the first cooperative block; an Err if its threads couldn't be spawned
    pool: Option<std::io::Result<super::coop::Pool>>,
    /// the leader's own one-core kernel's buffers, for a block below `MIN_COOP` (a table of
    /// at most `MIN_COOP` rows, 1 MB, while no team is spawned a full one)
    small: Solo,
}

/// The one-core kernel's buffers (`bwt::Cooperative::invert`'s: a table, the segments,
/// the output)
#[derive(Default)]
struct Solo {
    tt: super::bwt::TVector,
    seg: super::bwt::SegScratch,
    buf: Vec<u8>,
}

impl Team {
    #[cfg(feature = "trace")]
    fn buffers(&self) -> Vec<crate::trace::Buffer> {
        use crate::trace::Buffer;
        let [f0, f1] = &self.fronts;
        let s = &self.seg;
        // (the records' buffers, `runs.runs`, `runs.lit` and `runs.one`, once for each front)
        let records = [f0, f1].into_iter().flat_map(|f| f.runs.buffers());
        let mut v: Vec<Buffer> = records.filter(|b| b.cap > 0).collect();
        v.extend([Buffer::of("cuts", &f0.cuts), Buffer::of("cuts.b", &f1.cuts)]);
        v.extend([
            Buffer::of("ll", &f0.ll),
            Buffer::of("ll.b", &f1.ll),
            Buffer::of("checkpoints", &f0.checkpoints),
            Buffer::of("checkpoints.b", &f1.checkpoints),
            Buffer::of("rle", &self.bufs[0]),
            Buffer::of("rle.b", &self.bufs[1]),
            Buffer::of("tt", &self.tt.0),
            Buffer::of("small.tt", &self.small.tt.0),
            Buffer::of("small.rle", &self.small.buf),
            Buffer {
                name: "seg.bufs",
                len: s.bufs.iter().map(Vec::len).sum(),
                cap: s.bufs.iter().map(Vec::capacity).sum(),
            },
        ]);
        v
    }
}

/// A block whose move-to-front is done, in `Team::fronts[front]`
struct Staged {
    place: u64,
    bit: u64,
    symbols: super::block::Symbols,
    front: usize,
}

/// A block inverted and assembled, in `Team::bufs[buf]`, its RLE1 and CRC to come
struct Inverted {
    place: u64,
    bit: u64,
    symbols: super::block::Symbols,
    buf: usize,
}

/// The move-to-front stage of a block into `f`, and its primary index checked
/// (`decode_block`'s first steps): its length. A block of few symbols (`runs::takes_runs`)
/// is staged as its run records and their cuts (`runs::front`, `runs::cuts`), with no last
/// column, as `decode_block` stages it for a kernel that reads runs. The cuts are the
/// tiling's whole check, made here, so that no later stage refuses the records; records
/// that fail it are a bug in `runs::front` (a debug build stops here), and the block is
/// then staged by its column, which still gives the right bytes.
fn front_stage(b: &super::block::Symbols, f: &mut Front, k: Kernels) -> Result<usize, ErrorKind> {
    f.by_runs = false;
    if super::runs::takes_runs(b) {
        let n = super::runs::front(b, &mut f.runs, k)?;
        if b.orig_ptr as usize >= n {
            return Err(ErrorKind::BadOrigPtr);
        }
        if super::runs::cuts(&f.runs, n, &mut f.cuts) {
            (f.by_runs, f.rows) = (true, n);
            return Ok(n);
        }
        debug_assert!(false, "the run records don't tile the block");
    }
    let n = super::mtf::mtf_checkpoints(b, &mut f.ll, &mut f.counts, &mut f.checkpoints, k)?;
    if b.orig_ptr as usize >= n {
        return Err(ErrorKind::BadOrigPtr);
    }
    f.rows = n;
    Ok(n)
}

/// RLE1 undone into `out` and the block's CRC checked (`decode_block`'s last steps)
fn finish_block(inverted: &[u8], crc: u32, k: Kernels, out: &mut Vec<u8>) -> Result<(), ErrorKind> {
    let start = out.len();
    if let Err(kind) = super::rle::unrle(inverted, out, k) {
        out.truncate(start);
        return Err(kind);
    }
    let computed = super::crc::crc32(&out[start..], k);
    if computed != crc {
        out.truncate(start);
        return Err(ErrorKind::BlockCrc {
            stored: crc,
            computed,
        });
    }
    Ok(())
}

/// A block's result (or error, with its place and bit) into `ready`, and its symbols'
/// buffer recycled: the worker's own last steps
fn publish(shared: &Shared, place: u64, bit: u64, res: Result<Vec<u8>, ErrorKind>, syms: Vec<u16>) {
    let res = res.map_err(|kind| Error {
        kind,
        block: Some(place),
        bit: Some(bit),
    });
    #[cfg(test)]
    shared
        .faults
        .lock()
        .unwrap()
        .published
        .push((place, res.as_ref().err().cloned()));
    let mut st = shared.lock();
    if !st.cancelled {
        st.ready.insert(place, res);
    }
    if st.free_syms.len() < shared.slots as usize {
        st.free_syms.push(syms);
    }
    drop(st);
    mark!(Ready, place);
    shared.cv.notify_all();
}

/// Testing only: where a pipelined team's run panics, outside every stage's own catch, so
/// that the panic unwinds through the pool's run into `pipelined_worker`'s outer `Err`
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fault {
    /// the leader's part, on entry: the prior block it holds not yet finished
    LeadStart,
    /// the leader's part, on its way out: the prior published, the next job staged if one
    /// was queued
    LeadEnd,
}

/// Testing only (`Shared::faults`)
#[cfg(test)]
#[derive(Default)]
struct Faults {
    /// the run (its block's place) and the point to panic at, once
    at: Option<(u64, Fault)>,
    /// the blocks the team held when it panicked: the prior inverted, the next staged
    held: Option<(Option<u64>, Option<u64>)>,
    /// every block published (`publish`), in order, with its error if any
    published: Vec<(u64, Option<Error>)>,
    /// the outer unwinds `pipelined_worker` caught
    recovered: u32,
    /// a team's threshold in place of `MIN_COOP`, so that small blocks run the team
    min_coop: Option<usize>,
}

/// Testing only: the injected panic, if `place`'s run is to panic at `at`
#[cfg(test)]
fn inject(shared: &Shared, place: u64, at: Fault, l: &Lead) {
    let mut f = shared.faults.lock().unwrap();
    if f.at == Some((place, at)) {
        f.at = None;
        f.held = Some((
            l.inverted.as_ref().map(|b| b.place),
            l.next.as_ref().map(|b| b.place),
        ));
        drop(f);
        panic!("injected at block {place}, {at:?}");
    }
}

/// The fewest rows a block runs on the team with (below, the leader decodes it alone)
fn min_coop(_shared: &Shared) -> usize {
    #[cfg(test)]
    if let Some(n) = _shared.faults.lock().unwrap().min_coop {
        return n;
    }
    super::bwt::MIN_COOP
}

/// An output buffer to reuse, or None once the decode is cancelled
fn take_out(shared: &Shared) -> Option<Vec<u8>> {
    let mut st = shared.lock();
    if st.cancelled {
        return None;
    }
    let mut out = st.free_out.pop().unwrap_or_default();
    out.clear();
    Some(out)
}

/// An inverted block's RLE1 and CRC, from `inverted` (its assembled bytes), and its result
/// published; a panic in it becomes the block's `Internal` error. Returns whether it
/// panicked (then `inverted` is to be reset).
fn finish_and_publish(shared: &Shared, b: Inverted, inverted: &[u8], k: Kernels) -> bool {
    let Some(mut out) = take_out(shared) else {
        return false; // (cancelled: nothing to publish, the symbols dropped as the worker drops them)
    };
    mark!(RleStart, b.place);
    let crc = b.symbols.crc;
    let r = catch_unwind(AssertUnwindSafe(|| {
        finish_block(inverted, crc, k, &mut out)
    }));
    mark!(RleEnd, b.place, out.len(), out.capacity());
    let (res, panicked) = match r {
        Ok(Ok(())) => (Ok(out), false),
        Ok(Err(kind)) => (Err(kind), false),
        Err(p) => (Err(ErrorKind::Internal(panic_message(p))), true),
    };
    publish(shared, b.place, b.bit, res, b.symbols.syms);
    panicked
}

/// A block staged in `f` inverted on this thread alone (`bwt::Cooperative`'s one-core
/// kernel, on `sc`: a block below `MIN_COOP`, or every block of a team whose threads
/// couldn't be spawned), then finished and published. Returns whether it panicked (then `f`
/// and `sc` are to be reset).
fn solo(shared: &Shared, f: &Front, sc: &mut Solo, s: Staged, k: Kernels) -> bool {
    let Some(mut out) = take_out(shared) else {
        return false;
    };
    let r = catch_unwind(AssertUnwindSafe(|| {
        mark!(RunStart, s.place);
        let orig_ptr = s.symbols.orig_ptr;
        if f.invert_alone(orig_ptr, k, &mut sc.tt, &mut sc.seg, &mut sc.buf) {
            if f.by_runs {
                sc.seg.assemble_reversed(&mut sc.buf);
            } else {
                sc.seg.assemble(&mut sc.buf);
            }
        }
        // (the `walk-stats` feature's record of a block the leader walked alone)
        #[cfg(feature = "walk-stats")]
        super::coop::walk_stats::push(format!(
            "{{\"kind\":\"solo\",\"team\":{},\"block\":{},\"rows\":{}}}",
            super::coop::walk_stats::team(),
            s.place,
            f.rows
        ));
        mark!(RunEnd, s.place);
        mark!(RleStart, s.place);
        let r = finish_block(&sc.buf, s.symbols.crc, k, &mut out);
        mark!(RleEnd, s.place, out.len(), out.capacity());
        r
    }));
    let (res, panicked) = match r {
        Ok(Ok(())) => (Ok(out), false),
        Ok(Err(kind)) => (Err(kind), false),
        Err(p) => (Err(ErrorKind::Internal(panic_message(p))), true),
    };
    publish(shared, s.place, s.bit, res, s.symbols.syms);
    panicked
}

/// `solo` on the team's own buffers, a panic resetting them
fn solo_on(shared: &Shared, t: &mut Team, s: Staged, k: Kernels) {
    let i = s.front;
    if solo(shared, &t.fronts[i], &mut t.small, s, k) {
        t.fronts[i] = Front::default();
        t.small = Solo::default();
    }
}

/// What the leader's part of a run did with the next job
#[derive(Default)]
struct Lead {
    /// the block to finish (block i − 1), taken by the leader
    inverted: Option<Inverted>,
    /// the next block, staged for the next run (a block below `MIN_COOP` is decoded whole
    /// by the leader in the run instead, on its own one-core buffers)
    next: Option<Staged>,
    /// the leader's RLE1 panicked: its assembled buffer is to be reset
    rle_panicked: bool,
}

/// The next job if one is queued, without waiting: None if none is (or the jobs' lock is
/// held, by a worker waiting in `recv`, which will take the next one)
fn try_job(jobs: &Mutex<Receiver<Job>>) -> Option<Job> {
    use std::sync::TryLockError;
    let rx = match jobs.try_lock() {
        Ok(rx) => rx,
        Err(TryLockError::Poisoned(p)) => p.into_inner(),
        Err(TryLockError::WouldBlock) => return None,
    };
    rx.try_recv().ok()
}

/// What a job's move-to-front gave: a block for the team, a block below `MIN_COOP` for the
/// leader alone, or none (its error published)
enum Staging {
    Team(Staged),
    Small(Staged),
    Failed,
}

/// A job's move-to-front into `f`. A panic resets `f`.
fn stage(shared: &Shared, job: Job, f: &mut Front, front: usize, k: Kernels) -> Staging {
    let (place, bit, symbols) = job;
    mark!(MtfStart, place);
    let r = catch_unwind(AssertUnwindSafe(|| front_stage(&symbols, f, k)));
    mark!(MtfEnd, place);
    let s = Staged {
        place,
        bit,
        symbols,
        front,
    };
    match r {
        Ok(Ok(n)) if n >= min_coop(shared) => Staging::Team(s),
        Ok(Ok(_)) => Staging::Small(s),
        Ok(Err(kind)) => {
            publish(shared, place, bit, Err(kind), s.symbols.syms);
            Staging::Failed
        }
        Err(p) => {
            *f = Front::default();
            let kind = ErrorKind::Internal(panic_message(p));
            publish(shared, place, bit, Err(kind), s.symbols.syms);
            Staging::Failed
        }
    }
}

/// A pipelined team's worker: its leader's loop. `PARK` and `SMALL` are `Levers`' park and
/// small, `SPIN` the waits' spin budget; both false at `coop::BARRIER_SPINS`, it is the
/// registered prototype's
fn pipelined_worker<const PARK: bool, const SMALL: bool, const SPIN: u32>(
    shared: &Shared,
    jobs: &Mutex<Receiver<Job>>,
    k: Kernels,
    team_k: usize,
) {
    use super::bwt::FLAGGED_LANES;
    let mut t = Team::default();
    let mut staged: Option<Staged> = None;
    let mut inverted: Option<Inverted> = None;
    loop {
        let Some(s) = staged.take() else {
            // the pipeline drained: a block inverted is finished alone
            if let Some(b) = inverted.take() {
                let i = b.buf;
                if finish_and_publish(shared, b, &t.bufs[i], k) {
                    t.bufs[i] = Vec::new();
                }
                continue;
            }
            // nothing in flight: wait for the next job
            mark!(JobLock);
            let job = {
                let rx = jobs.lock().unwrap_or_else(|e| e.into_inner());
                mark!(JobRecv);
                rx.recv()
            };
            let Ok(job) = job else {
                #[cfg(feature = "trace")]
                if crate::trace::level() > 0 {
                    crate::trace::buffers(&t.buffers());
                }
                return;
            };
            mark!(JobGot, job.0);
            if shared.lock().cancelled {
                continue; // (drain the queue, so the scanner never waits on it)
            }
            match stage(shared, job, &mut t.fronts[0], 0, k) {
                Staging::Team(s) => staged = Some(s),
                Staging::Small(small) => solo_on(shared, &mut t, small, k),
                Staging::Failed => {}
            }
            continue;
        };
        if t.pool
            .get_or_insert_with(|| super::coop::Pool::pipelined(team_k))
            .is_err()
        {
            // (no team: every block on this thread, as `bwt::Cooperative` falls back)
            if let Some(b) = inverted.take() {
                let i = b.buf;
                if finish_and_publish(shared, b, &t.bufs[i], k) {
                    t.bufs[i] = Vec::new();
                }
            }
            solo_on(shared, &mut t, s, k);
            continue;
        }
        // the run: the helpers invert `s` into bufs[a], while the leader finishes
        // `inverted` from bufs[1 - a] and stages the next job in fronts[1 - s.front]
        let a = inverted.as_ref().map_or(0, |b| 1 - b.buf);
        let other = 1 - s.front;
        let lead = Mutex::new(Lead {
            inverted: inverted.take(),
            ..Lead::default()
        });
        let Team {
            fronts,
            bufs,
            tt,
            seg,
            pool,
            small,
        } = &mut t;
        let Some(Ok(pool)) = pool.as_ref() else {
            unreachable!("the pool was spawned above")
        };
        let [f0, f1] = fronts;
        let (hf, lf) = if s.front == 0 { (&*f0, f1) } else { (&*f1, f0) };
        let [b0, b1] = bufs;
        let (hb, lb) = if a == 0 { (b0, &*b1) } else { (b1, &*b0) };
        let lf = Mutex::new(lf);
        let small = Mutex::new(small);
        #[cfg(test)]
        let place = s.place;
        let leader = || {
            let mut l = lock_lead(&lead);
            #[cfg(test)]
            inject(shared, place, Fault::LeadStart, &l);
            if let Some(b) = l.inverted.take() {
                l.rle_panicked = finish_and_publish(shared, b, lb, k);
            }
            // (one job, as registered; with `SMALL`, another after each block below
            // `MIN_COOP`, until one is staged for the team or none is queued: `try_job`
            // never waits, so the leader never holds the run waiting for the scanner)
            let mut next_job = try_job(jobs);
            while let Some(job) = next_job.take() {
                mark!(JobGot, job.0);
                shared.overlapped.fetch_add(1, Ordering::Relaxed);
                if shared.lock().cancelled {
                    return;
                }
                let mut f = lf.lock().unwrap_or_else(|e| e.into_inner());
                match stage(shared, job, &mut f, other, k) {
                    Staging::Team(next) => l.next = Some(next),
                    Staging::Small(b) => {
                        // a block below `MIN_COOP`: decoded whole here, overlapped too
                        let mut sc = small.lock().unwrap_or_else(|e| e.into_inner());
                        if solo(shared, &f, &mut sc, b, k) {
                            **f = Front::default();
                            **sc = Solo::default();
                        }
                        // (lever B: its front set is free again, published and done with)
                        if SMALL {
                            next_job = try_job(jobs);
                        }
                    }
                    Staging::Failed => {
                        shared.overlap_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            #[cfg(test)]
            inject(shared, place, Fault::LeadEnd, &l);
        };
        // (the run's own kinds, not `InvertStart`/`InvertEnd`: a worker's inversion inside
        // its `decode_block`, which a pipelined team never runs)
        mark!(RunStart, s.place);
        let r = catch_unwind(AssertUnwindSafe(|| {
            super::coop::invert_pipelined_from::<FLAGGED_LANES, PARK, SPIN>(
                pool,
                hf.source(k),
                s.symbols.orig_ptr,
                tt,
                seg,
                hb,
                &leader,
                s.place,
            )
        }));
        mark!(RunEnd, s.place);
        let mut l = lead.into_inner().unwrap_or_else(|e| e.into_inner());
        match r {
            Ok(Ok(())) => {
                inverted = Some(Inverted {
                    place: s.place,
                    bit: s.bit,
                    symbols: s.symbols,
                    buf: a,
                });
                if l.rle_panicked {
                    t.bufs[1 - a] = Vec::new();
                }
            }
            Ok(Err(p)) => {
                // a helper's stage panicked: block i fails, and the back set is reset
                let kind = ErrorKind::Internal(panic_message(p));
                publish(shared, s.place, s.bit, Err(kind), s.symbols.syms);
                t.tt = Default::default();
                t.seg = Default::default();
                t.bufs = Default::default();
            }
            Err(p) => {
                #[cfg(test)]
                {
                    shared.faults.lock().unwrap().recovered += 1;
                }
                // the run itself panicked (the leader's part catches its own stages', so a
                // bug outside them): every block the run held fails, and the team is reset
                let msg = panic_message(p);
                let held = [Some(s), l.next.take()];
                for b in held.into_iter().flatten() {
                    let kind = ErrorKind::Internal(msg.clone());
                    publish(shared, b.place, b.bit, Err(kind), b.symbols.syms);
                }
                if let Some(b) = l.inverted.take() {
                    let kind = ErrorKind::Internal(msg.clone());
                    publish(shared, b.place, b.bit, Err(kind), b.symbols.syms);
                }
                t = Team::default();
                continue;
            }
        }
        staged = l.next.take();
    }
}

fn lock_lead(m: &Mutex<Lead>) -> std::sync::MutexGuard<'_, Lead> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// ---- The service core (research, opt-in: `StreamDecoder::service`) ----
//
// The teams' serial work taken out of the teams. K service workers serve every team: for
// a block to come, its move-to-front and checkpoints ("preparing"); for a block walked,
// its assembly, RLE1, CRC and publish ("finishing"). Each team's k threads only build and
// walk (`coop::invert_team`: `bwt::Cooperative`'s build and walk, unassembled).
//
// Threads: the scanner (as every decoder's); a feeder (`mibz2-svc-feed`) moving the
// scanner's jobs into the hub, so a service worker waits on one condition for either kind
// of work (it never sits in the jobs' `recv` while a walked block waits for it, which could
// leave the reader's next block behind a scanner waiting for the reader's slot); K service
// workers (`mibz2-svc-N`); and the teams (`mibz2-N`, each with its pool's `mibz2-coop-*`).
//
// The hub's two queues, and the buffers that bound them:
// - **prepared** (service → teams): blocks whose move-to-front is done, each in a front
//   set (`Front`: `ll`, counts, checkpoints). Fronts: a team's each, the queue's lookahead
//   (`teams`), and a service worker's each; a service worker takes a free one to prepare
//   into, a team gives it back once its block is walked (or dropped, or failed).
// - **finishing** (teams → service): blocks walked, each in a back set (`Back`: the walk's
//   segments, or for a block below `MIN_SEGMENTED` its bytes). Backs: two a team; a team
//   takes one with each block, a service worker gives it back once it has finished it.
//
// **The service's priority (registered):** finishing first, except that the prepared queue
// is never left empty while a block can be prepared. In order, a service worker:
// 1. prepares, if the prepared queue is empty and a job and a front set are free (so no
//    team waits for a block while finishing is backed up);
// 2. finishes the walked block earliest in the stream, if any (the reader's next, and a
//    back set freed for the teams);
// 3. prepares, if the prepared queue is below its lookahead (a block a team) and a job and
//    a front set are free;
// 4. ends, once the jobs are closed and no block is left in flight; else waits.
//
// **Blocks of few symbols** (`runs::takes_runs`) are prepared as their run records and
// cuts (`front_stage`), filled and walked backward by the team (`coop::invert_team_from`
// on `coop::Source::Runs`), and their segments joined in reverse by the finisher
// (`Back::reversed`). Nothing else about a block's way through the hub differs.
//
// **Small blocks** (below `MIN_COOP`) are prepared and finished by the service, as every
// block, and inverted whole by a team's first thread on the one-core kernel (its helpers
// idle meanwhile), exactly as a `Cooperative` team inverts them today. Not by the service:
// the model makes the service the scarce resource (about 0.8 of a core at 1.4 GB/s on its
// own stages), and a block's inversion costs about three times its serial stages, so small
// blocks on the service would saturate it first (registration).
//
// **Ordered output, errors, cancellation, panics:** every block's result (or its error,
// with its place and bit) goes to the decoder's `ready` through `publish`, as a worker's
// does, so the reader takes them in order. A cancelled decode drops its blocks wherever
// they are (their buffers returned), and the feeder drains the jobs so the scanner never
// waits on them. A panic in any stage (a service worker's preparing or finishing, a team's
// build or walk, any of its pool's threads) is caught where it happens and becomes that
// block's `Internal` error, the buffers it held replaced with new ones (never lost, so the
// buffer bounds never shrink into a deadlock). Each thread ends once the jobs are closed
// (the scanner's end) and every block taken has been published or dropped: the end of
// stream's flush.

/// A block's inversion's buffers (a back set): the walk's segments, or, for a block too
/// short to segment, its bytes walked straight into `direct`
#[derive(Default)]
struct Back {
    seg: super::bwt::SegScratch,
    direct: Vec<u8>,
    segmented: bool,
    /// the segments are an LF table's, walked backward (a block staged as its run
    /// records): joined by `assemble_reversed`
    reversed: bool,
}

/// A block prepared (move-to-front done), in its front set
struct Prepared {
    place: u64,
    bit: u64,
    symbols: super::block::Symbols,
    front: Front,
    rows: usize,
}

/// A block walked, in its back set
struct Walked {
    place: u64,
    bit: u64,
    symbols: super::block::Symbols,
    back: Back,
}

/// Where the tests' fault hook makes a service-core stage panic (research, tests only):
/// a block's preparing or finishing on a service worker, or a team's pool thread `thread`
/// in that block's walk (0 the team's first thread, 1.. its helpers)
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceFault {
    Prepare { place: u64 },
    Finish { place: u64 },
    Team { place: u64, thread: usize },
}

struct HubState {
    /// the scanner's jobs, moved here by the feeder; closed at the scanner's end
    jobs: std::collections::VecDeque<Job>,
    closed: bool,
    prepared: std::collections::VecDeque<Prepared>,
    walked: Vec<Walked>,
    fronts: Vec<Front>,
    backs: Vec<Back>,
    /// blocks taken from `jobs`, not yet published or dropped
    live: usize,
    /// what the service core has done so far (`StreamDecoder::service_counts`): blocks
    /// prepared and queued, blocks a team inverted on its pool (not alone), blocks
    /// finished (published, any result); counted under this lock where it is already held
    counts: ServiceCounts,
    /// the decode failed as a whole (`Hub::fail`): every thread ends at its next look at
    /// the hub, whatever is left
    failed: bool,
    /// the hub's threads that have ended (`Outer`'s drop, the last thing each does), of
    /// `Hub::threads`: the completion fence for `counts` (each thread's last retirement
    /// comes before its end, both under this lock)
    exited: usize,
    /// (tests only: live tokens taken (a job taken to prepare) and retired (published or
    /// dropped, by a stage or by `Hub::fail`), so a test sees none lost and none twice)
    #[cfg(test)]
    tokens: (u64, u64),
    /// (tests only: the waits a service worker and a team's first thread have entered,
    /// each counted under this lock just before its `Condvar::wait`, so a test holding the
    /// lock and seeing the count knows that thread is parked, not merely slow)
    #[cfg(test)]
    svc_waits: usize,
    #[cfg(test)]
    team_waits: usize,
    /// (tests only: the threads in `Condvar::wait` now, counted in under this lock before
    /// the wait and out after it, for `OuterFault::parked`'s handshake)
    #[cfg(test)]
    parked: usize,
}

impl HubState {
    /// After the decode failed (`Hub::fail`), a block a stage has done with is not handed
    /// on (no thread would take it from the queue): it retires here, with `o`'s hold, its
    /// buffers dropped by the caller. Whether it did.
    fn retired_on_failure(&mut self, o: &Outer) -> bool {
        if !self.failed {
            return false;
        }
        self.live = self.live.saturating_sub(1);
        o.held.set(None);
        #[cfg(test)]
        {
            self.tokens.1 += 1;
        }
        true
    }

    /// The decode's end for the hub's threads: every job taken and every block taken
    /// retired after the scanner's end, or the decode failed as a whole
    fn done(&self) -> bool {
        self.failed || (self.closed && self.jobs.is_empty() && self.live == 0)
    }
}

struct Hub {
    st: Mutex<HubState>,
    cv: Condvar,
    lookahead: usize,
    fault: Option<ServiceFault>,
    /// its threads: the feeder, the service workers and the teams' first threads
    threads: usize,
    /// (tests only: a panic outside every stage's catch, `outer_fault`; fired once)
    #[cfg(test)]
    outer: Option<OuterFault>,
    #[cfg(test)]
    fired: std::sync::atomic::AtomicBool,
}

/// A service-core thread's outer guard (D067's guard-and-fail). Each stage's own catch
/// recovers its block alone (an `Internal` error at its place and bit, its buffers
/// replaced), and stays the recovery for everything a stage runs. The guard covers what
/// runs outside those catches: acquiring a block and a buffer set under the hub's lock,
/// the hand-offs, a team's pool set-up, and the gap between a block's publication and its
/// retirement. Should a thread unwind there (a bug: nothing there should panic), the
/// hub's state can't be trusted to be repaired in place, so the decode fails as a whole
/// (`Hub::fail`) rather than continuing.
///
/// `held` is the live block this thread is responsible for (its place): set when it takes
/// one (a job to prepare, a prepared block to walk, a walked block to finish) and cleared
/// when it hands it on (queued for the next stage) or retires it, each under the hub's
/// lock, in the same critical section as the queue's change or `live`'s. So the guard
/// retires exactly the blocks no queue and no other thread holds, once each.
///
/// The guard is the first thing each thread makes, so it drops last: every lock the thread
/// held is released before it runs, and a team's pool (whose drop joins its helpers) has
/// ended before it. It allocates only its error's message and takes the two locks one
/// after the other, never nested.
struct Outer<'a> {
    shared: &'a Shared,
    hub: &'a Hub,
    held: std::cell::Cell<Option<u64>>,
    what: &'static str,
}

impl<'a> Outer<'a> {
    fn new(shared: &'a Shared, hub: &'a Hub, what: &'static str) -> Self {
        Outer {
            shared,
            hub,
            held: std::cell::Cell::new(None),
            what,
        }
    }
}

impl Drop for Outer<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.hub.fail(self.shared, self.held.take(), self.what);
        }
        self.hub.lock().exited += 1;
    }
}

/// Testing only: where a service-core thread panics outside every stage's catch
/// (`Outer`), once
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OuterAt {
    /// a service worker, at its loop's top, holding nothing
    SvcIdle,
    /// a service worker, a job and a front set just taken, before preparing it
    AfterJob,
    /// a service worker, a block prepared, before queueing it for the teams
    BeforePrepared,
    /// a service worker, a block just queued for the teams (held by the queue)
    AfterPrepared,
    /// a team, a prepared block and a back set just taken, before its pool is chosen
    AfterTake,
    /// a team, a block just queued for finishing (held by the queue)
    AfterWalked,
    /// a service worker, a walked block just taken, before finishing it
    AfterFinishTake,
    /// a service worker, a block's result published, before it is retired
    AfterPublish,
    /// the hub's feeder, a job just moved into the hub
    Feeder,
}

/// Testing only: an outer fault: where, at which block (None: the first time `at` is
/// reached), and how many of the hub's threads must be parked in its wait now
/// (`HubState::parked`), with the jobs closed, before it fires (a parked-state handshake, so a waiter is known to be
/// parked in `Condvar::wait` when the decode fails; 0 for none)
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
struct OuterFault {
    at: OuterAt,
    place: Option<u64>,
    parked: usize,
}

/// Testing only: panic here if the hub's outer fault is `at` this block (once)
#[cfg(test)]
fn outer_fault(hub: &Hub, at: OuterAt, place: Option<u64>) {
    let Some(f) = hub.outer else { return };
    if f.at != at || (f.place.is_some() && f.place != place) {
        return;
    }
    if hub.fired.swap(true, Ordering::AcqRel) {
        return;
    }
    if f.parked > 0 {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        loop {
            {
                let st = hub.lock();
                if st.closed && st.parked >= f.parked {
                    break;
                }
            }
            assert!(
                std::time::Instant::now() < deadline,
                "the handshake: never parked"
            );
            std::thread::yield_now();
        }
    }
    panic!("an injected outer panic at {at:?}, block {place:?}");
}

impl Hub {
    fn lock(&self) -> std::sync::MutexGuard<'_, HubState> {
        // (no stage runs under this lock; it is consistent at every unlock)
        self.st.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn wait<'a>(
        &self,
        g: std::sync::MutexGuard<'a, HubState>,
    ) -> std::sync::MutexGuard<'a, HubState> {
        self.cv.wait(g).unwrap_or_else(|e| e.into_inner())
    }

    /// A block left the decode (published or dropped): its buffers back, if any, and the
    /// thread's hold on it (`Outer::held`) released under the same lock
    fn retire(&self, o: &Outer, front: Option<Front>, back: Option<Back>) {
        self.retire_counted(o, front, back, false);
    }

    /// `retire`, counting a finished block (`ServiceCounts::finished`) under the same lock
    fn retire_counted(&self, o: &Outer, front: Option<Front>, back: Option<Back>, finished: bool) {
        let mut st = self.lock();
        st.live -= 1;
        o.held.set(None);
        #[cfg(test)]
        {
            st.tokens.1 += 1;
        }
        st.counts.finished += u64::from(finished);
        st.fronts.extend(front);
        st.backs.extend(back);
        drop(st);
        self.cv.notify_all();
    }
}

impl Hub {
    /// Its counts, and whether they are final (`ServiceCounts::ended`)
    fn counts(&self) -> ServiceCounts {
        let st = self.lock();
        ServiceCounts {
            ended: st.exited >= self.threads,
            ..st.counts
        }
    }

    /// The decode fails as a whole (`Outer`): a thread unwound outside every stage's
    /// catch. In this order:
    /// 1. the terminal error (`Internal`, naming the thread) is recorded, with `cancelled`,
    ///    before anything is woken, so the reader's next look sees it, not `Cancelled`; the
    ///    blocks ready but not yet read are dropped, as a cancel drops them, so **the bytes
    ///    before the failure are not all delivered**: only those the reader already had;
    /// 2. the hub fails: `held` (the block the unwinding thread was responsible for, if
    ///    any) is retired, and so is every block a queue holds (prepared, walked), their
    ///    buffers dropped, never reused; the jobs not yet taken are dropped. A block another
    ///    thread holds in a stage is that thread's to retire as its stage ends (a team's
    ///    pool waits for its helpers first);
    /// 3. both domains are woken: the reader, scanner and writers (`Shared::cv`; the
    ///    scanner then ends, closing the jobs), and every hub waiter (`Hub::cv`), each of
    ///    which sees `done` and ends.
    ///
    /// The first failure's error stands; a second (another thread unwinding meanwhile)
    /// changes nothing but its own block's retirement. Counts saturate rather than panic:
    /// this runs during an unwind, on state a bug may have left inconsistent.
    fn fail(&self, shared: &Shared, held: Option<u64>, what: &str) {
        {
            let mut st = shared.lock();
            if st.terminal.is_none() {
                st.terminal = Some(Error::new(ErrorKind::Internal(format!(
                    "the service core's {what} thread panicked outside its stages; the \
                     decode is stopped"
                ))));
            }
            st.cancelled = true;
            st.ready.clear();
        }
        {
            let mut st = self.lock();
            st.failed = true;
            let queued = st.prepared.len() + st.walked.len();
            let retired = queued + usize::from(held.is_some());
            st.prepared.clear();
            st.walked.clear();
            st.jobs.clear();
            st.live = st.live.saturating_sub(retired);
            #[cfg(test)]
            {
                st.tokens.1 += retired as u64;
            }
        }
        shared.cv.notify_all();
        self.cv.notify_all();
    }
}

impl StreamDecoder {
    /// Research, opt-in, not the default: the service core. `teams` teams of `k` threads
    /// (at least 1 team, and `k` of 1 to 4) only build and walk each block, while `services`
    /// service workers (at least 1) do every block's move-to-front and checkpoints, and its
    /// assembly, RLE1, CRC and publish, with `slots` blocks in flight (at least `teams`).
    /// Byte for byte the default decoder's output and errors.
    #[doc(hidden)]
    pub fn service(
        teams: usize,
        k: usize,
        services: usize,
        slots: usize,
        kernels: Kernels,
    ) -> StreamDecoder {
        Self::service_with_fault(teams, k, services, slots, kernels, None)
    }

    /// `service` with the tests' fault hook (research: a stage made to panic, for the tests
    /// of the panic path; `None` is `service`)
    #[doc(hidden)]
    pub fn service_with_fault(
        teams: usize,
        k: usize,
        services: usize,
        slots: usize,
        kernels: Kernels,
        fault: Option<ServiceFault>,
    ) -> StreamDecoder {
        Self::service_built(
            teams,
            k,
            services,
            slots,
            kernels,
            fault,
            #[cfg(test)]
            None,
        )
    }

    /// `service_with_fault`, with (tests only) an outer fault (`OuterFault`)
    fn service_built(
        teams: usize,
        k: usize,
        services: usize,
        slots: usize,
        kernels: Kernels,
        fault: Option<ServiceFault>,
        #[cfg(test)] outer: Option<OuterFault>,
    ) -> StreamDecoder {
        let (teams, team_k, services) = (teams.max(1), k.clamp(1, 4), services.max(1));
        let slots = slots.max(teams) as u64;
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                ready: BTreeMap::new(),
                issued: 0,
                taken: 0,
                end: None,
                cancelled: false,
                free_out: Vec::new(),
                free_syms: Vec::new(),
                terminal: None,
            }),
            cv: Condvar::new(),
            slots,
            overlapped: AtomicU64::new(0),
            overlap_errors: AtomicU64::new(0),
            #[cfg(test)]
            faults: Mutex::default(),
        });
        let lookahead = teams;
        let hub = Arc::new(Hub {
            st: Mutex::new(HubState {
                jobs: Default::default(),
                closed: false,
                prepared: Default::default(),
                walked: Vec::new(),
                fronts: (0..teams + lookahead + services)
                    .map(|_| Front::default())
                    .collect(),
                backs: (0..2 * teams).map(|_| Back::default()).collect(),
                live: 0,
                counts: ServiceCounts::default(),
                failed: false,
                exited: 0,
                #[cfg(test)]
                tokens: (0, 0),
                #[cfg(test)]
                svc_waits: 0,
                #[cfg(test)]
                team_waits: 0,
                #[cfg(test)]
                parked: 0,
            }),
            cv: Condvar::new(),
            lookahead,
            fault,
            threads: 1 + services + teams,
            #[cfg(test)]
            outer,
            #[cfg(test)]
            fired: Default::default(),
        });
        let (in_tx, in_rx) = sync_channel::<Vec<u8>>(INPUT_QUEUE);
        let (job_tx, job_rx) = channel::<Job>();
        let mut handles = Vec::with_capacity(teams + services + 2);
        let spawn = |name: String, f: Box<dyn FnOnce() + Send>| {
            std::thread::Builder::new()
                .name(name)
                .spawn(f)
                .expect("spawn a bz2 service-core thread")
        };
        let (s, h) = (shared.clone(), hub.clone());
        handles.push(spawn(
            "mibz2-svc-feed".into(),
            Box::new(move || feed(&s, &h, job_rx)),
        ));
        for i in 0..services {
            let (s, h) = (shared.clone(), hub.clone());
            handles.push(spawn(
                format!("mibz2-svc-{i}"),
                Box::new(move || service_worker(&s, &h, kernels)),
            ));
        }
        for i in 0..teams {
            let (s, h) = (shared.clone(), hub.clone());
            handles.push(spawn(
                format!("mibz2-{i}"),
                Box::new(move || team_worker(&s, &h, team_k, kernels)),
            ));
        }
        let sh = shared.clone();
        handles.push(spawn(
            "mibz2-scan".into(),
            Box::new(move || scanner(&sh, in_rx, job_tx)),
        ));
        StreamDecoder {
            input: Mutex::new(Some(in_tx)),
            shared,
            handles: Mutex::new(handles),
            reader: Mutex::default(),
            threads: teams * team_k + services,
            pipelined: false,
            hub: Some(hub),
        }
    }
}

/// The scanner's jobs into the hub, then its end (the jobs' channel closed). It holds no
/// block (a job is no one's until a service worker takes it), but its loss would leave the
/// jobs never closed, so it is guarded too. Once the decode has failed it drops the jobs
/// it is given.
fn feed(shared: &Shared, hub: &Hub, rx: Receiver<Job>) {
    let _o = Outer::new(shared, hub, "feeder");
    while let Ok(job) = rx.recv() {
        #[cfg(test)]
        let place = job.0;
        {
            let mut st = hub.lock();
            if !st.failed {
                st.jobs.push_back(job);
            }
        }
        hub.cv.notify_all();
        #[cfg(test)]
        outer_fault(hub, OuterAt::Feeder, Some(place));
    }
    hub.lock().closed = true;
    hub.cv.notify_all();
}

/// What a service worker does next (the priority above)
// (one at a time, on the worker's stack: a front set's counts are 1 KB)
#[allow(clippy::large_enum_variant)]
enum SvcTask {
    Prepare(Job, Front),
    Finish(Walked),
}

fn service_worker(shared: &Shared, hub: &Hub, k: Kernels) {
    let o = Outer::new(shared, hub, "service");
    // the assembly's buffer, this worker's own
    let mut asm: Vec<u8> = Vec::new();
    loop {
        #[cfg(test)]
        outer_fault(hub, OuterAt::SvcIdle, None);
        let task = {
            let mut st = hub.lock();
            let mut idle = false;
            let task = loop {
                if st.failed {
                    break None;
                }
                let can_prepare = !st.jobs.is_empty() && !st.fronts.is_empty();
                if can_prepare && st.prepared.is_empty() {
                    break Some(take_job(&mut st, &o));
                }
                if !st.walked.is_empty() {
                    // (the earliest place: the reader's next)
                    let i = (0..st.walked.len())
                        .min_by_key(|&i| st.walked[i].place)
                        .unwrap_or(0);
                    let w = st.walked.swap_remove(i);
                    o.held.set(Some(w.place));
                    mark!(SvcGot, w.place, st.walked.len(), 0);
                    break Some(SvcTask::Finish(w));
                }
                if can_prepare && st.prepared.len() < hub.lookahead {
                    break Some(take_job(&mut st, &o));
                }
                if st.done() {
                    break None;
                }
                if !idle {
                    idle = true;
                    mark!(SvcIdleStart);
                }
                #[cfg(test)]
                {
                    st.svc_waits += 1;
                    st.parked += 1;
                }
                st = hub.wait(st);
                #[cfg(test)]
                {
                    st.parked -= 1;
                }
            };
            if idle {
                mark!(SvcIdleEnd);
            }
            task
        };
        match task {
            None => return,
            Some(SvcTask::Prepare(job, front)) => {
                #[cfg(test)]
                outer_fault(hub, OuterAt::AfterJob, Some(job.0));
                prepare(shared, hub, &o, job, front, k)
            }
            Some(SvcTask::Finish(w)) => {
                #[cfg(test)]
                outer_fault(hub, OuterAt::AfterFinishTake, Some(w.place));
                finish(shared, hub, &o, w, &mut asm, k)
            }
        }
    }
}

/// The next job and a front set, taken (the block live from here, and `o`'s)
fn take_job(st: &mut HubState, o: &Outer) -> SvcTask {
    let job = st.jobs.pop_front().expect("a job");
    let front = st.fronts.pop().expect("a front set");
    st.live += 1;
    o.held.set(Some(job.0));
    #[cfg(test)]
    {
        st.tokens.0 += 1;
    }
    SvcTask::Prepare(job, front)
}

/// A block's move-to-front (and its primary index's check) into `front`, then queued for
/// the teams; its error published, or its panic made its `Internal` error
fn prepare(shared: &Shared, hub: &Hub, o: &Outer, job: Job, mut front: Front, k: Kernels) {
    let (place, bit, symbols) = job;
    mark!(JobGot, place);
    if shared.lock().cancelled {
        return hub.retire(o, Some(front), None);
    }
    mark!(MtfStart, place);
    let r = catch_unwind(AssertUnwindSafe(|| {
        if hub.fault == Some(ServiceFault::Prepare { place }) {
            panic!("an injected panic in block {place}'s preparing");
        }
        front_stage(&symbols, &mut front, k)
    }));
    mark!(MtfEnd, place);
    match r {
        Ok(Ok(rows)) => {
            #[cfg(test)]
            outer_fault(hub, OuterAt::BeforePrepared, Some(place));
            let mut st = hub.lock();
            if st.retired_on_failure(o) {
                return;
            }
            st.prepared.push_back(Prepared {
                place,
                bit,
                symbols,
                front,
                rows,
            });
            o.held.set(None);
            st.counts.prepared += 1;
            mark!(Prepared, place, st.prepared.len(), 0);
            drop(st);
            hub.cv.notify_all();
            #[cfg(test)]
            outer_fault(hub, OuterAt::AfterPrepared, Some(place));
        }
        Ok(Err(kind)) => {
            publish(shared, place, bit, Err(kind), symbols.syms);
            hub.retire(o, Some(front), None);
        }
        Err(p) => {
            let kind = ErrorKind::Internal(panic_message(p));
            publish(shared, place, bit, Err(kind), symbols.syms);
            hub.retire(o, Some(Front::default()), None);
        }
    }
}

/// A walked block's assembly, RLE1 and CRC, and its result published; a panic made its
/// `Internal` error, its back set and the assembly's buffer replaced
fn finish(shared: &Shared, hub: &Hub, o: &Outer, w: Walked, asm: &mut Vec<u8>, k: Kernels) {
    let Walked {
        place,
        bit,
        symbols,
        back,
    } = w;
    let Some(mut out) = take_out(shared) else {
        return hub.retire(o, None, Some(back)); // (cancelled)
    };
    let r = catch_unwind(AssertUnwindSafe(|| {
        let inverted: &[u8] = if back.segmented {
            mark!(AssembleStart, place);
            if back.reversed {
                back.seg.assemble_reversed(asm);
            } else {
                back.seg.assemble(asm);
            }
            mark!(AssembleEnd, place);
            asm
        } else {
            &back.direct
        };
        mark!(RleStart, place);
        if hub.fault == Some(ServiceFault::Finish { place }) {
            panic!("an injected panic in block {place}'s finishing");
        }
        finish_block(inverted, symbols.crc, k, &mut out)
    }));
    mark!(RleEnd, place, out.len(), out.capacity());
    let (res, back) = match r {
        Ok(Ok(())) => (Ok(out), back),
        Ok(Err(kind)) => (Err(kind), back),
        Err(p) => {
            *asm = Vec::new();
            (Err(ErrorKind::Internal(panic_message(p))), Back::default())
        }
    };
    // (published, not yet retired: still `o`'s, so an unwind here retires it once, and
    // `Hub::fail` never publishes it again)
    publish(shared, place, bit, res, symbols.syms);
    #[cfg(test)]
    outer_fault(hub, OuterAt::AfterPublish, Some(place));
    hub.retire_counted(o, None, Some(back), true);
}

/// A team's next prepared block and a back set to walk it into, waiting until both are
/// there; None once the decode has ended. Taking a prepared block wakes the hub's waiters:
/// a service worker waiting with the prepared queue at its lookahead (its rule 3) may now
/// prepare again. (`notify_all`: teams and service workers share the one condition, so a
/// single wake-up could go to a team that can't use it.)
fn take_prepared(hub: &Hub, o: &Outer) -> Option<(Prepared, Back)> {
    let mut st = hub.lock();
    let mut idle = false;
    let got = loop {
        if st.failed {
            break None;
        }
        if !st.prepared.is_empty() && !st.backs.is_empty() {
            let p = st.prepared.pop_front().expect("a prepared block");
            let b = st.backs.pop().expect("a back set");
            o.held.set(Some(p.place));
            mark!(TeamGot, p.place, st.prepared.len(), 0);
            break Some((p, b));
        }
        if st.done() {
            break None;
        }
        if !idle {
            idle = true;
            mark!(
                TeamIdleStart,
                crate::trace::NONE,
                u8::from(!st.prepared.is_empty()),
                0
            );
        }
        #[cfg(test)]
        {
            st.team_waits += 1;
            st.parked += 1;
        }
        st = hub.wait(st);
        #[cfg(test)]
        {
            st.parked -= 1;
        }
    };
    drop(st);
    if idle {
        mark!(TeamIdleEnd);
    }
    if got.is_some() {
        hub.cv.notify_all();
    }
    got
}

/// A team: its first thread takes prepared blocks and inverts each on the team's pool
/// (`coop::invert_team`), or alone for a block below `MIN_COOP` (or with no pool), into a
/// back set, then queues it for finishing
fn team_worker(shared: &Shared, hub: &Hub, team_k: usize, k: Kernels) {
    use super::bwt::FLAGGED_LANES;
    // (made first, so dropped last: after the pool, whose drop joins its helpers)
    let o = Outer::new(shared, hub, "team");
    let mut tt = super::bwt::TVector::default();
    let mut pool: Option<std::io::Result<super::coop::Pool>> = None;
    loop {
        let Some((p, mut back)) = take_prepared(hub, &o) else {
            return;
        };
        #[cfg(test)]
        outer_fault(hub, OuterAt::AfterTake, Some(p.place));
        let Prepared {
            place,
            bit,
            symbols,
            front,
            rows,
        } = p;
        if shared.lock().cancelled {
            hub.retire(&o, Some(front), Some(back));
            continue;
        }
        let team = if team_k >= 2 && rows >= min_coop(shared) {
            pool.get_or_insert_with(|| super::coop::Pool::new(team_k))
                .as_ref()
                .ok()
        } else {
            None
        };
        let thread_fault = match hub.fault {
            Some(ServiceFault::Team { place: p, thread }) if p == place => Some(thread),
            _ => None,
        };
        mark!(RunStart, place);
        let r = catch_unwind(AssertUnwindSafe(|| match team {
            Some(pool) => {
                super::coop::invert_team_from::<FLAGGED_LANES>(
                    pool,
                    front.source(k),
                    symbols.orig_ptr,
                    &mut tt,
                    &mut back.seg,
                    place,
                    thread_fault,
                );
                back.segmented = true;
                back.reversed = front.by_runs;
            }
            None => {
                // (the one-core kernel, as `bwt::Cooperative` inverts a small block)
                if thread_fault == Some(0) {
                    panic!("an injected panic in block {place}'s one-core inversion");
                }
                back.segmented = front.invert_alone(
                    symbols.orig_ptr,
                    k,
                    &mut tt,
                    &mut back.seg,
                    &mut back.direct,
                );
                back.reversed = front.by_runs;
            }
        }));
        mark!(RunEnd, place);
        match r {
            Ok(()) => {
                let mut st = hub.lock();
                if st.retired_on_failure(&o) {
                    continue;
                }
                st.counts.pooled += u64::from(team.is_some());
                st.fronts.push(front);
                st.walked.push(Walked {
                    place,
                    bit,
                    symbols,
                    back,
                });
                o.held.set(None);
                mark!(Walked, place, st.walked.len(), 0);
                drop(st);
                hub.cv.notify_all();
                #[cfg(test)]
                outer_fault(hub, OuterAt::AfterWalked, Some(place));
            }
            Err(p) => {
                // a team thread's stage panicked: the block fails, the table and the back
                // set are replaced (the pool is reusable: `Pool::run` resumed the panic
                // once all its threads had ended)
                tt = Default::default();
                let kind = ErrorKind::Internal(panic_message(p));
                publish(shared, place, bit, Err(kind), symbols.syms);
                hub.retire(&o, Some(front), Some(Back::default()));
            }
        }
    }
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

/// The topology three teams of three were measured on (physical cores, L3 bytes as
/// `l3_bytes` reads them), and the fewest threads asked for that select them: 3 × 3 runs 9
/// decoding threads, inside any budget of 10 or more
const TEN_CORES: (usize, u64) = (10, 20 << 20);
const TEN_CORES_MIN_THREADS: usize = 10;

/// The one plan the default decoder runs pipelined (`StreamDecoder::new`): three teams of
/// three, the plan the pipelined team with both levers was measured at against the plain
/// teams, on the i9-10900K (docs/journal/2026-11-07-pipeline-levers-result.md: 1.072
/// [1.053, 1.091] faster, for about 5% more CPU and 8 MB more peak RSS). Every other plan
/// keeps the plain path.
pub const PIPELINED_PLAN: Plan = Plan::Teams { teams: 3, k: 3 };

/// The default pipelined teams' levers: park and small, not the spin-budget control
pub const PIPELINED_LEVERS: Levers = Levers {
    park: true,
    small: true,
    spin2k: false,
};

thread_local! {
    /// `Plan::for_threads`'s answer on this thread while `with_plan` runs
    static FORCED: std::cell::Cell<Option<Plan>> = const { std::cell::Cell::new(None) };
}

/// Testing: `f` run with `Plan::for_threads` answering `plan` on this thread, whatever the
/// machine and the threads asked for, so the default constructors (`StreamDecoder::new`,
/// `BzReader::parallel`, `decode_parallel`) build `plan`'s decoder. Restored on return or
/// unwind; other threads are unaffected.
#[doc(hidden)]
pub fn with_plan<T>(plan: Plan, f: impl FnOnce() -> T) -> T {
    struct Restore(Option<Plan>);
    impl Drop for Restore {
        fn drop(&mut self) {
            FORCED.with(|c| c.set(self.0));
        }
    }
    let _restore = Restore(FORCED.with(|c| c.replace(Some(plan))));
    f()
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
    ///   two block sizes (workstation journal §9), 4 teams there;
    /// - except on the measured ten-core topology (10 physical cores, a 20 MiB L3) at 10 or
    ///   more threads: three teams of three, which two registered runs qualified against
    ///   four teams of two on the i9-10900K (docs/journal/2026-11-06-three-tables.md). Any
    ///   other CPU with that topology gets it too, unmeasured. At that plan alone
    ///   (`PIPELINED_PLAN`) the default decoder runs its teams pipelined, with park and
    ///   small (`PIPELINED_LEVERS`; docs/journal/2026-11-07-pipeline-levers-result.md).
    pub fn for_threads(threads: usize) -> Plan {
        if let Some(plan) = FORCED.with(|c| c.get()) {
            return plan;
        }
        // (the plan policy's two facts only, read as master reads them: the predicate's
        // topology snapshot and quota are read by `StreamDecoder::new` at the service
        // core's one budget alone, so no other budget pays for their discovery)
        let (phys, l3) = MACHINE
            .with(|c| c.borrow().as_ref().map(|m| (m.phys, m.l3)))
            .unwrap_or_else(|| (physical_cores(), l3_bytes()));
        plan_with(threads.max(1), phys, l3)
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

/// The service core's counts (`StreamDecoder::service_counts`)
#[doc(hidden)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServiceCounts {
    /// blocks whose move-to-front a service worker did, queued for the teams
    pub prepared: u64,
    /// blocks a team inverted on its pool (`coop::invert_team`), not its first thread alone
    pub pooled: u64,
    /// blocks a service worker finished (assembly, RLE1, CRC) and published, any result
    /// (blocks failing in preparing or a team's run retire without finishing)
    pub finished: u64,
    /// whether every one of the hub's threads (feeder, service workers, teams) had ended
    /// when these were read: only then are they final. A reader's EOF is not that fence
    /// (the last block is published, and read, before its finisher retires it), nor is
    /// dropping a `BzReader` (its feeder may hold the decoder's last reference); joining
    /// the decoder (`StreamDecoder::join`) is.
    pub ended: bool,
}

/// The service core's shape: `teams` teams of `k` threads that only build and walk, and
/// `services` service workers doing every block's serial stages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServicePlan {
    pub teams: usize,
    pub k: usize,
    pub services: usize,
}

impl ServicePlan {
    /// The teams as a `Plan` (for `window`: a team counts as one worker)
    pub fn as_plan(&self) -> Plan {
        Plan::Teams {
            teams: self.teams,
            k: self.k,
        }
    }

    /// Its decoding threads: the teams' and the service workers' (8 at `SERVICE_PLAN`);
    /// the scanner, the hub's feeder and the caller's threads are not among them
    pub fn threads(&self) -> usize {
        self.teams * self.k + self.services
    }
}

/// The one shape the default decoder runs the service core at (`StreamDecoder::new`):
/// svc2, two teams of three and two service workers, the shape #258's registered run
/// measured at 10 threads on the i9-10900K (docs/journal/2026-11-09-service-2x3-result.md;
/// a runtime without this integration's later changes, so the guarded candidate is not
/// thereby qualified: docs/journal/2026-11-09-service-guarded-scope.md)
pub const SERVICE_PLAN: ServicePlan = ServicePlan {
    teams: 2,
    k: 3,
    services: 2,
};

/// The one thread budget the service core is the default at: exactly 10, as measured.
/// 9, 11 and every other budget keep the existing plan.
pub const SERVICE_THREADS: usize = 10;

/// The supported target's CPU: Intel family 6, model 165 (Comet Lake-S: the i9-10900K's
/// CPUID; other Comet Lake-S parts share it), with 10 physical cores, 20 logical CPUs (two
/// a core) and a 20 MiB L3, every one of them this process's to use
const SERVICE_CPU: (&str, u32, u32) = ("GenuineIntel", 6, 165);
const SERVICE_LOGICAL: usize = 20;
/// The supported target's processor name, as `/proc/cpuinfo`'s `model name` gives it on
/// the one measured machine: the i9-10900K alone. CPUID's family and model admit the whole
/// 10-core Comet Lake-S class (the i9-10900, 10900F, 10900T, 10850K, 10910, 10900KF, the
/// Xeon W-1290 parts), whose clocks, power limits and SKU differ from the measured part's;
/// the name narrows the predicate to the SKU measured
/// (docs/journal/2026-11-10-service-guarded-target-gated-registration.md, coverage)
const SERVICE_NAME: &str = "Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz";

/// What the plan policy and the service core's target predicate read of the machine
/// (`Machine::here`). The plan policy's two facts are as it has always read them (`phys`,
/// `l3`: None where the OS doesn't say). The predicate reads only `target` and `quota`,
/// each read to fail closed: None, or `Quota::Unknown`, wherever a fact it needs can't be
/// established.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Machine {
    /// the plan policy's: physical cores of the CPUs this process may use
    /// (`physical_cores`)
    pub phys: Option<usize>,
    /// the plan policy's: the L3's bytes (`l3_bytes`)
    pub l3: Option<u64>,
    /// the predicate's: the CPUs this process may use, as one coherent snapshot
    /// (`target_topology`), None if any of it can't be read or the CPUs differ
    pub target: Option<Topology>,
    /// the predicate's: the CPU quota of this process's cgroup and every ancestor
    /// (`cpu_quota`)
    pub quota: Quota,
}

/// The CPUs this process may use and that are online, read together (`target_topology`):
/// every one of them with its core, package and L3 read, and all of one vendor, family,
/// model and L3 size
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topology {
    /// distinct (package, core) pairs among them
    pub phys: usize,
    /// how many CPUs
    pub logical: usize,
    /// each one's L3, in bytes (all the same)
    pub l3: u64,
    /// each one's vendor, family and model (all the same)
    pub cpu: (String, u32, u32),
    /// each one's processor name, `/proc/cpuinfo`'s `model name`, trimmed (all the same)
    pub name: String,
}

/// A cgroup's CPU quota, as `cpu_quota` establishes it
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Quota {
    /// established: no `cpu.max` limit on this process's cgroup or any ancestor, up to and
    /// including the hierarchy's true root
    Unlimited,
    /// established: the tightest limit on the way up is this many CPUs' time
    Cpus(f64),
    /// not established (unreadable, unparsable, a hidden ancestor, cgroup v1 or no cgroup
    /// v2 mount): the predicate is false
    #[default]
    Unknown,
}

/// `observe_builds`'s record: each decoder built, with the service core's hub to read its
/// counts from
type Builds = Vec<(Built, Option<Arc<Hub>>)>;

thread_local! {
    /// `Machine::here`'s answer on this thread while `with_machine` runs
    static MACHINE: std::cell::RefCell<Option<Machine>> = const { std::cell::RefCell::new(None) };
    /// `observe_builds`'s record, while it runs on this thread
    static BUILT: std::cell::RefCell<Option<Builds>> =
        const { std::cell::RefCell::new(None) };
}

impl Machine {
    /// This machine, as this process sees it now (or `with_machine`'s, on this thread)
    pub fn here() -> Machine {
        if let Some(m) = MACHINE.with(|c| c.borrow().clone()) {
            return m;
        }
        Machine {
            phys: physical_cores(),
            l3: l3_bytes(),
            target: target_topology(&RealFs),
            quota: cpu_quota(&RealFs),
        }
    }

    /// The i9-10900K as the service core's registered run saw it: every fact
    /// `service_target` asks for, as it asks
    pub fn service_target() -> Machine {
        Machine {
            phys: Some(TEN_CORES.0),
            l3: Some(TEN_CORES.1),
            target: Some(Topology {
                phys: TEN_CORES.0,
                logical: SERVICE_LOGICAL,
                l3: TEN_CORES.1,
                cpu: (SERVICE_CPU.0.into(), SERVICE_CPU.1, SERVICE_CPU.2),
                name: SERVICE_NAME.into(),
            }),
            quota: Quota::Unlimited,
        }
    }
}

/// Testing: `f` run with `Machine::here` answering `m` on this thread, so that the plan
/// policy (`Plan::for_threads`) and the service core's predicate (`service_target`) see
/// that machine and the default constructors (`StreamDecoder::new`, `BzReader::parallel`,
/// `decode_parallel`) choose as they would on it, by their own dispatch. Restored on return
/// or unwind; other threads are unaffected.
#[doc(hidden)]
pub fn with_machine<T>(m: Machine, f: impl FnOnce() -> T) -> T {
    struct Restore(Option<Machine>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let m = self.0.take();
            MACHINE.with(|c| *c.borrow_mut() = m);
        }
    }
    let _restore = Restore(MACHINE.with(|c| c.borrow_mut().replace(m)));
    f()
}

/// What `StreamDecoder::new` built, as `observe_builds` records it
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Built {
    pub service: bool,
    pub pipelined: bool,
    pub threads: usize,
    pub window: u64,
    /// the service core's counts once `observe_builds`'s `f` had returned (None otherwise)
    pub counts: Option<ServiceCounts>,
}

/// Testing: `f`'s result, and every decoder `StreamDecoder::new` built on this thread while
/// it ran (so through `BzReader::parallel`, `parallel_with` and `decode_parallel` too), with
/// the service core's counts read once `f` returned: what the default dispatch chose, and
/// what that decoder then did
#[doc(hidden)]
pub fn observe_builds<T>(f: impl FnOnce() -> T) -> (T, Vec<Built>) {
    struct Restore(Option<Builds>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let v = self.0.take();
            BUILT.with(|c| *c.borrow_mut() = v);
        }
    }
    let restore = Restore(BUILT.with(|c| c.borrow_mut().replace(Vec::new())));
    let t = f();
    let got = BUILT.with(|c| c.borrow_mut().take()).unwrap_or_default();
    drop(restore);
    let got = got
        .into_iter()
        .map(|(mut b, hub)| {
            b.counts = hub.map(|h| h.counts());
            b
        })
        .collect();
    (t, got)
}

/// `observe_builds`'s record of `dec`, if it is running on this thread
fn record_built(dec: &StreamDecoder) {
    BUILT.with(|c| {
        if let Some(v) = c.borrow_mut().as_mut() {
            v.push((
                Built {
                    service: dec.is_service(),
                    pipelined: dec.pipelined,
                    threads: dec.threads,
                    window: dec.shared.slots,
                    counts: None,
                },
                dec.hub.clone(),
            ));
        }
    });
}

/// The service core's supported target, the one place the default decoder runs it
/// (`StreamDecoder::new`), every condition explicit and each one the measured run's:
/// - exactly `SERVICE_THREADS` (10) threads asked for: not 9, not 11, not "10 or more";
/// - the kernels the run used: AVX2 (with BMI1) and the CLMUL CRC, as `Kernels::detect`
///   finds them there (the scalar kernels, or any other mix, keep the existing plan);
/// - an x86-64 Linux machine (the only OS whose topology this reads);
/// - the CPUs this process may use, online, read as one coherent snapshot
///   (`target_topology`): every one `GenuineIntel`, family 6, model 165, with a 20 MiB L3,
///   20 of them on 10 physical cores, and every one named `SERVICE_NAME`
///   (`Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`): the measured SKU, not the 10-core
///   Comet Lake-S class that family and model alone admit (its other members are
///   unmeasured, and the one result does not qualify them). The existing 10-core / 20 MiB
///   rule's wider reach (any CPU of that shape) is not inherited;
/// - an established cgroup CPU quota (`cpu_quota`) of none: unlimited, as measured. Any
///   limit, however large, keeps the existing plan.
///
/// Discovery fails closed: any fact that can't be established (`Machine::target` None,
/// `Quota::Unknown`) makes the predicate false. On any other machine or budget the decoder
/// is exactly what it was.
pub fn service_target(threads: usize, k: Kernels, m: &Machine) -> bool {
    threads == SERVICE_THREADS
        && k.avx2
        && k.clmul
        && cfg!(all(target_os = "linux", target_arch = "x86_64"))
        && m.target.as_ref().is_some_and(|t| {
            (t.cpu.0.as_str(), t.cpu.1, t.cpu.2) == SERVICE_CPU
                && t.name == SERVICE_NAME
                && t.phys == TEN_CORES.0
                && t.logical == SERVICE_LOGICAL
                && t.l3 == TEN_CORES.1
        })
        && m.quota == Quota::Unlimited
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
    if phys == TEN_CORES.0 && l3 == Some(TEN_CORES.1) && threads >= TEN_CORES_MIN_THREADS {
        return Plan::Teams { teams: 3, k: 3 };
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

/// The service core's hub under Miri's race detector: its threads (feeder, service
/// workers, teams), queues, buffer bounds and wake-ups, on a stream small enough to
/// interpret (four streams of a small block each, as Python's bz2 writes them)
#[cfg(test)]
mod service_tests {
    use super::*;

    /// `bz2.compress(b"service core %d " % i * (3 + i), 1)` for i in 0..4, concatenated
    pub(super) const Z: &[u8] = &[
        66, 90, 104, 49, 49, 65, 89, 38, 83, 89, 8, 54, 204, 213, 0, 0, 20, 153, 128, 64, 0, 64, 0,
        10, 32, 153, 0, 32, 0, 34, 191, 213, 79, 81, 147, 66, 12, 152, 142, 105, 17, 214, 214, 167,
        152, 140, 98, 214, 248, 187, 146, 41, 194, 132, 128, 65, 182, 102, 168, 66, 90, 104, 49,
        49, 65, 89, 38, 83, 89, 209, 238, 231, 126, 0, 0, 27, 153, 128, 64, 0, 32, 0, 10, 32, 153,
        0, 32, 0, 49, 6, 76, 64, 85, 71, 168, 201, 167, 4, 33, 211, 101, 148, 120, 194, 24, 97,
        101, 159, 23, 114, 69, 56, 80, 144, 209, 238, 231, 126, 66, 90, 104, 49, 49, 65, 89, 38,
        83, 89, 96, 89, 60, 120, 0, 0, 34, 153, 128, 64, 0, 16, 0, 10, 32, 153, 0, 32, 0, 49, 6,
        76, 64, 138, 158, 161, 234, 105, 177, 34, 68, 226, 105, 42, 117, 60, 152, 145, 49, 49, 42,
        84, 248, 187, 146, 41, 194, 132, 131, 2, 201, 227, 192, 66, 90, 104, 49, 49, 65, 89, 38,
        83, 89, 195, 71, 89, 100, 0, 0, 41, 153, 128, 64, 0, 8, 0, 10, 32, 153, 0, 32, 0, 49, 6,
        76, 64, 138, 158, 161, 234, 105, 176, 96, 193, 224, 232, 104, 244, 124, 56, 48, 112, 112,
        104, 209, 248, 187, 146, 41, 194, 132, 134, 26, 58, 203, 32,
    ];

    pub(super) fn want() -> Vec<u8> {
        (0..4)
            .flat_map(|i| format!("service core {i} ").repeat(3 + i).into_bytes())
            .collect()
    }

    fn run(dec: &StreamDecoder, z: &[u8]) -> (Vec<u8>, Result<(), Error>) {
        dec.write(z.to_vec()).unwrap();
        dec.close();
        let (mut out, mut buf) = (Vec::new(), [0u8; 64]);
        loop {
            match dec.read(&mut buf) {
                Ok(0) => return (out, Ok(())),
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => return (out, Err(e)),
            }
        }
    }

    /// Each stream's block's stored CRC flipped in turn, and stream `s`'s first block's bit
    fn flipped(s: usize) -> (Vec<u8>, u64) {
        let at: Vec<usize> = (0..Z.len() - 3)
            .filter(|&i| &Z[i..i + 4] == b"BZh1")
            .collect();
        let mut z = Z.to_vec();
        z[at[s] + 4 + 6] ^= 0x10;
        (z, (at[s] as u64 + 4) * 8)
    }

    /// A hub as `service_with_fault` makes it, with no threads: `lookahead` blocks of
    /// lookahead and `fronts` front sets
    fn hub(lookahead: usize, fronts: usize) -> Hub {
        hub_with(lookahead, fronts, 1, None)
    }

    /// `hub`, with `backs` back sets and the fault hook given
    fn hub_with(lookahead: usize, fronts: usize, backs: usize, fault: Option<ServiceFault>) -> Hub {
        Hub {
            st: Mutex::new(HubState {
                jobs: Default::default(),
                closed: false,
                prepared: Default::default(),
                walked: Vec::new(),
                fronts: (0..fronts).map(|_| Front::default()).collect(),
                backs: (0..backs).map(|_| Back::default()).collect(),
                live: 0,
                counts: ServiceCounts::default(),
                failed: false,
                exited: 0,
                tokens: (0, 0),
                svc_waits: 0,
                team_waits: 0,
                parked: 0,
            }),
            cv: Condvar::new(),
            lookahead,
            fault,
            threads: 0,
            outer: None,
            fired: Default::default(),
        }
    }

    /// A decoder's shared state, as `service_with_fault` makes it (cancelled or not)
    fn shared(cancelled: bool) -> Shared {
        Shared {
            state: Mutex::new(State {
                ready: BTreeMap::new(),
                issued: 0,
                taken: 0,
                end: None,
                cancelled,
                free_out: Vec::new(),
                free_syms: Vec::new(),
                terminal: None,
            }),
            cv: Condvar::new(),
            slots: 4,
            overlapped: AtomicU64::new(0),
            overlap_errors: AtomicU64::new(0),
            faults: Mutex::default(),
        }
    }

    /// Wait (yielding, at most `secs`) until `ok` holds of the hub's state, checked under
    /// its lock; whether it did
    fn until(h: &Hub, secs: u64, ok: impl Fn(&HubState) -> bool) -> bool {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
        loop {
            if ok(&h.lock()) {
                return true;
            }
            if std::time::Instant::now() > deadline {
                return false;
            }
            std::thread::yield_now();
        }
    }

    /// The wake-up of rule 3 under a controlled schedule (D045's handshake): the service
    /// worker is shown to be parked in its `Condvar::wait` before the pop it must wake on,
    /// not merely given time. Each waiter counts its wait under the hub's lock just before
    /// it waits, and `Condvar::wait` releases that lock only once the thread is waiting, so
    /// a test that then holds the lock and sees the count knows the thread is parked (or,
    /// after a spurious wake-up, blocked on the lock to recheck: the only schedule in which
    /// a missing wake-up could go unseen here). A team's first thread is parked before it
    /// too (waiting for a back set), so the one wake-up a `notify_one` would send can go to
    /// the team, which can't use it, rather than to the service worker.
    ///
    /// Then a team takes the prepared block (`take_prepared`, the rule-3 wake-up) with the
    /// lock never released between that pop and the handshake but by the pop itself, and
    /// nothing else happens until the service worker has taken its job or 10 s (Miri: 120)
    /// have passed: no job fed, no front set returned, no block walked.
    #[test]
    fn miri_a_parked_service_worker_wakes_on_the_pop() {
        let shared = shared(false);
        // lookahead 1, two front sets (one holding block 0), no back set yet
        let h = hub_with(1, 2, 0, None);
        {
            let mut st = h.lock();
            st.live = 1;
            let front = st.fronts.pop().unwrap();
            st.prepared.push_back(Prepared {
                place: 0,
                bit: 0,
                symbols: job(0).2,
                front,
                rows: 0,
            });
            st.jobs.push_back(job(1));
        }
        let secs = if cfg!(miri) { 120 } else { 10 };
        let o = Outer::new(&shared, &h, "test");
        std::thread::scope(|s| {
            // a team's first thread, parked: a prepared block, but no back set to walk into
            let team = s.spawn(|| take_prepared(&h, &Outer::new(&shared, &h, "test")));
            assert!(
                until(&h, secs, |st| st.team_waits == 1),
                "the team never parked"
            );
            // the service worker, parked: the prepared queue at its lookahead
            let svc = s.spawn(|| service_worker(&shared, &h, Kernels::scalar()));
            assert!(
                until(&h, secs, |st| st.svc_waits == 1),
                "the service never parked"
            );
            let (p, back) = {
                let mut st = h.lock();
                // both parked, and nothing done: the handshake
                assert_eq!((st.team_waits, st.svc_waits), (1, 1));
                assert_eq!((st.jobs.len(), st.prepared.len()), (1, 1));
                // a back set for this thread's pop, given without a wake-up (the team
                // parked above is not told of it)
                st.backs.push(Back::default());
                drop(st);
                take_prepared(&h, &o).expect("block 0")
            };
            assert_eq!(p.place, 0);
            let woke = until(&h, secs, |st| st.jobs.is_empty());
            // (end it either way: block 1's empty symbols fail their move-to-front, so it
            // retires; block 0 retires here, then the hub is closed: the close's own
            // wake-up ends both threads even where the pop's was missing)
            h.retire(&o, Some(p.front), Some(back));
            h.lock().closed = true;
            h.cv.notify_all();
            svc.join().unwrap();
            assert!(team.join().unwrap().is_none());
            assert!(
                woke,
                "the parked service worker never woke on the team's pop"
            );
        });
        let st = h.lock();
        assert!(st.done() && st.prepared.is_empty());
        assert_eq!((st.fronts.len(), st.backs.len()), (2, 1));
        assert!(shared.lock().ready.get(&1).is_some_and(|r| r.is_err()));
    }

    /// Every route by which a block leaves the decode returns the buffers it held (or new
    /// ones, after a panic) and retires it once: preparing (its error, its panic, a
    /// cancelled decode), a team's run (its panic, a cancelled decode) and finishing (a CRC
    /// error, its panic, success, a cancelled decode). Each route is driven alone on a hub
    /// holding exactly one live block, and the hub then checked: `live` back to zero, and
    /// every front and back set home, so no route can shrink a buffer bound into a deadlock
    /// or leave the end waiting (D045: retirement and buffer return).
    #[test]
    fn miri_every_retirement_returns_its_buffers() {
        const FRONTS: usize = 3;
        const BACKS: usize = 2;
        let k = Kernels::scalar();
        let home = |h: &Hub, what: &str| {
            let st = h.lock();
            assert_eq!(st.live, 0, "{what}: retired");
            assert_eq!(st.fronts.len(), FRONTS, "{what}: the front sets");
            assert_eq!(st.backs.len(), BACKS, "{what}: the back sets");
            assert!(st.prepared.is_empty() && st.walked.is_empty(), "{what}");
        };
        let published = |sh: &Shared, place: u64| sh.lock().ready.get(&place).cloned();
        // preparing: a move-to-front error, a panic, a cancelled decode
        for (what, fault, cancelled) in [
            ("preparing's error", None, false),
            (
                "preparing's panic",
                Some(ServiceFault::Prepare { place: 3 }),
                false,
            ),
            ("preparing cancelled", None, true),
        ] {
            let (sh, h) = (shared(cancelled), hub_with(1, FRONTS, BACKS, fault));
            let o = Outer::new(&sh, &h, "test");
            let task = {
                let mut st = h.lock();
                st.jobs.push_back(job(3));
                take_job(&mut st, &o)
            };
            let SvcTask::Prepare(j, front) = task else {
                unreachable!()
            };
            prepare(&sh, &h, &o, j, front, k);
            assert_eq!(o.held.get(), None, "{what}: released");
            home(&h, what);
            let r = published(&sh, 3);
            if cancelled {
                assert!(r.is_none(), "{what}");
            } else {
                let e = r.expect(what).unwrap_err();
                assert_eq!((e.block, e.bit), (Some(3), Some(0)), "{what}");
                assert_eq!(fault.is_some(), matches!(e.kind, ErrorKind::Internal(_)));
            }
        }
        // a team's run: a panic (on the one-core path, as a block below MIN_COOP), and a
        // cancelled decode; the team ends once the hub is done
        for (what, fault, cancelled) in [
            (
                "a team's panic",
                Some(ServiceFault::Team {
                    place: 4,
                    thread: 0,
                }),
                false,
            ),
            ("a team cancelled", None, true),
        ] {
            let (sh, h) = (shared(cancelled), hub_with(1, FRONTS, BACKS, fault));
            {
                let mut st = h.lock();
                let front = st.fronts.pop().unwrap();
                st.prepared.push_back(Prepared {
                    place: 4,
                    bit: 32,
                    symbols: job(4).2,
                    front,
                    rows: 0,
                });
                st.live = 1;
                st.closed = true;
            }
            team_worker(&sh, &h, 2, Kernels::detect());
            home(&h, what);
            let r = published(&sh, 4);
            if cancelled {
                assert!(r.is_none(), "{what}");
            } else {
                let e = r.expect(what).unwrap_err();
                assert!(matches!(e.kind, ErrorKind::Internal(_)), "{what}: {e}");
                assert_eq!((e.block, e.bit), (Some(4), Some(32)), "{what}");
            }
        }
        // finishing: a CRC error, a panic, success, a cancelled decode (a block too short
        // to segment, its bytes walked straight into `direct`)
        let crc_x = super::super::crc::crc32(b"x", k);
        for (what, fault, cancelled, crc, want) in [
            ("finishing's CRC error", None, false, crc_x ^ 1, None),
            (
                "finishing's panic",
                Some(ServiceFault::Finish { place: 5 }),
                false,
                crc_x,
                None,
            ),
            ("finishing", None, false, crc_x, Some(b"x".to_vec())),
            ("finishing cancelled", None, true, crc_x, None),
        ] {
            let (sh, h) = (shared(cancelled), hub_with(1, FRONTS, BACKS, fault));
            let back = {
                let mut st = h.lock();
                st.live = 1;
                st.backs.pop().unwrap()
            };
            let mut symbols = job(5).2;
            symbols.crc = crc;
            let w = Walked {
                place: 5,
                bit: 40,
                symbols,
                back: Back {
                    direct: b"x".to_vec(),
                    ..back
                },
            };
            let o = Outer::new(&sh, &h, "test");
            o.held.set(Some(5));
            finish(&sh, &h, &o, w, &mut Vec::new(), k);
            assert_eq!(o.held.get(), None, "{what}: released");
            home(&h, what);
            let r = published(&sh, 5);
            match (cancelled, want) {
                (true, _) => assert!(r.is_none(), "{what}"),
                (false, Some(bytes)) => assert_eq!(r, Some(Ok(bytes)), "{what}"),
                (false, None) => {
                    let e = r.expect(what).unwrap_err();
                    assert_eq!((e.block, e.bit), (Some(5), Some(40)), "{what}");
                    assert_eq!(fault.is_some(), matches!(e.kind, ErrorKind::Internal(_)));
                }
            }
        }
    }

    fn job(place: u64) -> Job {
        let symbols = super::super::block::Symbols {
            crc: 0,
            orig_ptr: 0,
            seq_to_unseq: [0; 256],
            syms: vec![],
            eob: 1,
            max_len: 100_000,
        };
        (place, 0, symbols)
    }

    /// A service worker waiting with the prepared queue at its lookahead (rule 3) wakes when
    /// a team takes a prepared block, with nothing else to wake it (no walked block, no
    /// front set returned, no new job), and prepares the job waiting
    #[test]
    fn miri_a_team_taking_a_prepared_block_wakes_the_service() {
        let shared = Shared {
            state: Mutex::new(State {
                ready: BTreeMap::new(),
                issued: 0,
                taken: 0,
                end: None,
                cancelled: false,
                free_out: Vec::new(),
                free_syms: Vec::new(),
                terminal: None,
            }),
            cv: Condvar::new(),
            slots: 4,
            overlapped: AtomicU64::new(0),
            overlap_errors: AtomicU64::new(0),
            faults: Mutex::default(),
        };
        let h = hub(1, 2);
        {
            // block 0 prepared (the queue at its lookahead of 1), block 1 waiting as a job
            let mut st = h.lock();
            st.live = 1;
            let front = st.fronts.pop().unwrap();
            st.prepared.push_back(Prepared {
                place: 0,
                bit: 0,
                symbols: job(0).2,
                front,
                rows: 0,
            });
            st.jobs.push_back(job(1));
        }
        let o = Outer::new(&shared, &h, "test");
        std::thread::scope(|s| {
            let svc = s.spawn(|| service_worker(&shared, &h, Kernels::scalar()));
            // the service worker can't prepare yet: wait until it is parked on the hub (it
            // holds the lock only while it decides), then give it nothing else to see
            for _ in 0..if cfg!(miri) { 20 } else { 2000 } {
                std::thread::yield_now();
            }
            assert_eq!(h.lock().jobs.len(), 1, "prepared beyond the lookahead");
            let (p, back) = take_prepared(&h, &o).expect("block 0");
            assert_eq!(p.place, 0);
            // the service takes the job only if the pop woke it
            let wait = std::time::Duration::from_secs(if cfg!(miri) { 120 } else { 10 });
            let deadline = std::time::Instant::now() + wait;
            let mut woke = true;
            while !h.lock().jobs.is_empty() {
                if std::time::Instant::now() > deadline {
                    woke = false;
                    break;
                }
                std::thread::yield_now();
            }
            // (block 1's empty symbols fail their move-to-front: its error published, and
            // it retires; block 0 retires here, then the hub is closed and both end, the
            // close's own wake-up ending the service worker even if the pop's was missing)
            h.retire(&o, Some(p.front), Some(back));
            h.lock().closed = true;
            h.cv.notify_all();
            svc.join().unwrap();
            assert!(woke, "the service worker never woke on the team's pop");
        });
        let st = h.lock();
        assert!(st.done() && st.prepared.is_empty());
        assert!(shared.lock().ready.get(&1).is_some_and(|r| r.is_err()));
    }

    #[test]
    fn miri_service_core_hub() {
        let k = Kernels::scalar();
        let shapes: &[(usize, usize, usize, usize)] = if cfg!(miri) {
            &[(1, 2, 1, 1), (2, 1, 2, 3)]
        } else {
            &[(1, 2, 1, 1), (2, 1, 2, 3), (3, 3, 1, 12), (1, 4, 2, 2)]
        };
        for &(teams, tk, services, slots) in shapes {
            let shape = format!("{teams} x {tk} + {services} in {slots}");
            let (got, res) = run(&StreamDecoder::service(teams, tk, services, slots, k), Z);
            res.unwrap();
            assert!(got == want(), "{shape}");
            // a bad block CRC: the error at its place and bit, as the default's
            let (z, bit) = flipped(2);
            let (got, res) = run(&StreamDecoder::service(teams, tk, services, slots, k), &z);
            let (want_got, want_res) = run(
                &StreamDecoder::with_inverse::<super::super::bwt::Segmented>(1, 2, k),
                &z,
            );
            let e = res.clone().unwrap_err();
            assert!(matches!(e.kind, ErrorKind::BlockCrc { .. }), "{shape}: {e}");
            assert_eq!((e.block, e.bit), (Some(2), Some(bit)), "{shape}");
            assert_eq!(res, want_res, "{shape}");
            assert!(got == want_got, "{shape}");
            // a panic in each kind of stage: its block's Internal error, the bytes before
            for fault in [
                ServiceFault::Prepare { place: 1 },
                ServiceFault::Finish { place: 2 },
                ServiceFault::Team {
                    place: 1,
                    thread: 0,
                },
            ] {
                let dec =
                    StreamDecoder::service_with_fault(teams, tk, services, slots, k, Some(fault));
                let (got, res) = run(&dec, Z);
                let e = res.unwrap_err();
                assert!(matches!(&e.kind, ErrorKind::Internal(m) if m.contains("injected")));
                let place = match fault {
                    ServiceFault::Prepare { place }
                    | ServiceFault::Finish { place }
                    | ServiceFault::Team { place, .. } => place,
                };
                assert_eq!(e.block, Some(place), "{shape} {fault:?}");
                let before: usize = (0..place as usize)
                    .map(|i| format!("service core {i} ").len() * (3 + i))
                    .sum();
                assert!(got == want()[..before], "{shape} {fault:?}");
            }
            // cancelled before its input ends, and dropped
            let dec = StreamDecoder::service(teams, tk, services, slots, k);
            dec.write(Z[..100].to_vec()).unwrap();
            dec.cancel();
            assert_eq!(
                dec.read(&mut [0; 8]).unwrap_err().kind,
                ErrorKind::Cancelled
            );
            drop(dec);
        }
    }
}

/// The files discovery reads (`RealFs`, or a test's fixture): a file's text, or why not
trait Fs {
    fn read(&self, path: &str) -> std::io::Result<String>;
    fn is_dir(&self, path: &str) -> bool;
}

struct RealFs;

impl Fs for RealFs {
    fn read(&self, path: &str) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }
    fn is_dir(&self, path: &str) -> bool {
        std::path::Path::new(path).is_dir()
    }
}

/// A CPU list (`0-3,8,10-11`), strictly: None for anything else
fn cpu_list(s: &str) -> Option<std::collections::BTreeSet<usize>> {
    let s = s.trim();
    let mut v = std::collections::BTreeSet::new();
    if s.is_empty() {
        return Some(v);
    }
    for part in s.split(',') {
        let (a, b) = part.split_once('-').unwrap_or((part, part));
        let (a, b): (usize, usize) = (a.parse().ok()?, b.parse().ok()?);
        if a > b || b > 1 << 16 {
            return None;
        }
        v.extend(a..=b);
    }
    Some(v)
}

/// The predicate's topology (`Topology`), read to fail closed (Linux): the CPUs in this
/// process's affinity (`Cpus_allowed_list`) that sysfs lists online; CPUs it lists absent
/// (not `present`) or offline are positively excluded, not guessed. Each counted CPU's core
/// and package ids, its L3 (`cache/index3`, of level 3) and its `/proc/cpuinfo` record (by
/// `processor`) must all be read; one unreadable or unparsable is None, and so is any
/// difference among them (a mixed vendor, model or L3 size).
fn target_topology(fs: &dyn Fs) -> Option<Topology> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let status = fs.read("/proc/self/status").ok()?;
    let allowed = cpu_list(
        status
            .lines()
            .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))?,
    )?;
    let sys = "/sys/devices/system/cpu";
    let online = cpu_list(&fs.read(&format!("{sys}/online")).ok()?)?;
    let present = cpu_list(&fs.read(&format!("{sys}/present")).ok()?)?;
    let cpus: Vec<usize> = allowed
        .iter()
        .copied()
        .filter(|c| present.contains(c) && online.contains(c))
        .collect();
    if cpus.is_empty() {
        return None;
    }
    // (cpuinfo's records by processor)
    let info = fs.read("/proc/cpuinfo").ok()?;
    let mut ids = std::collections::BTreeMap::new();
    for rec in info.split("\n\n").filter(|r| !r.trim().is_empty()) {
        let field = |name: &str| {
            rec.lines().find_map(|l| {
                let (k, v) = l.split_once(':')?;
                (k.trim() == name).then(|| v.trim().to_string())
            })
        };
        let n: usize = field("processor")?.parse().ok()?;
        let id = (
            field("vendor_id")?,
            field("cpu family")?.parse::<u32>().ok()?,
            field("model")?.parse::<u32>().ok()?,
            field("model name")?,
        );
        if ids.insert(n, id).is_some() {
            return None;
        }
    }
    let num = |p: String| p.trim().parse::<u64>().ok();
    let (mut cores, mut l3, mut cpu) = (std::collections::BTreeSet::new(), None, None);
    for c in &cpus {
        let t = format!("{sys}/cpu{c}");
        let core = num(fs.read(&format!("{t}/topology/core_id")).ok()?)?;
        let pkg = num(fs.read(&format!("{t}/topology/physical_package_id")).ok()?)?;
        cores.insert((pkg, core));
        if num(fs.read(&format!("{t}/cache/index3/level")).ok()?)? != 3 {
            return None;
        }
        let size = cache_size(&fs.read(&format!("{t}/cache/index3/size")).ok()?)?;
        if *l3.get_or_insert(size) != size {
            return None;
        }
        let id = ids.get(c)?;
        if cpu.get_or_insert_with(|| id.clone()) != id {
            return None;
        }
    }
    let (vendor, family, model, name) = cpu?;
    Some(Topology {
        phys: cores.len(),
        logical: cpus.len(),
        l3: l3?,
        cpu: (vendor, family, model),
        name,
    })
}

/// A sysfs cache size (`20480K`), strictly
fn cache_size(s: &str) -> Option<u64> {
    let s = s.trim();
    let (num, mult) = match s.as_bytes().last()? {
        b'K' => (&s[..s.len() - 1], 1u64 << 10),
        b'M' => (&s[..s.len() - 1], 1 << 20),
        _ => (s, 1),
    };
    num.parse::<u64>().ok()?.checked_mul(mult)
}

/// This process's CPU quota, established or `Unknown` (Linux, cgroup v2 only). From
/// `/proc/self/cgroup`'s one v2 entry (`0::path`) and the one `cgroup2` mount in
/// `/proc/self/mountinfo` (its root and mount point), the process's cgroup directory, then
/// every directory up to the mount point, each `cpu.max` parsed strictly and the tightest
/// kept. A level's `cpu.max` may be absent (the cpu controller not enabled there: no limit
/// at that level) only if its directory is there. The mount point must be the hierarchy's
/// true root (no `cgroup.type`, which every non-root cgroup has): a mount of a cgroup below
/// the root (a cgroup namespace, a container) hides ancestors whose limits can't be read,
/// so it is `Unknown`. Any other read failure, any unparsable or ambiguous file (no v2
/// entry, cgroup v1 alone, several entries or mounts, an escaped path), is `Unknown`.
fn cpu_quota(fs: &dyn Fs) -> Quota {
    if !cfg!(target_os = "linux") {
        return Quota::Unknown;
    }
    cpu_quota_of(fs).unwrap_or(Quota::Unknown)
}

fn cpu_quota_of(fs: &dyn Fs) -> Option<Quota> {
    use std::io::ErrorKind::NotFound;
    let cg = fs.read("/proc/self/cgroup").ok()?;
    let mut v2 = cg.lines().filter_map(|l| l.strip_prefix("0::"));
    let path = v2.next()?.trim().to_string();
    if v2.next().is_some() || !path.starts_with('/') || path.contains("..") || path.contains(' ') {
        return None;
    }
    let info = fs.read("/proc/self/mountinfo").ok()?;
    let mut mounts = info.lines().filter_map(|l| {
        let (pre, post) = l.split_once(" - ")?;
        (post.split_whitespace().next()? == "cgroup2").then_some(pre)
    });
    let pre = mounts.next()?;
    if mounts.next().is_some() {
        return None;
    }
    let f: Vec<&str> = pre.split_whitespace().collect();
    let (root, point) = (*f.get(3)?, *f.get(4)?);
    if root.contains('\\') || point.contains('\\') || !point.starts_with('/') {
        return None;
    }
    // the process's cgroup under the mount's root, as a path under the mount point
    let rel = if root == "/" {
        path.as_str()
    } else {
        let r = path.strip_prefix(root)?;
        if !(r.is_empty() || r.starts_with('/')) {
            return None;
        }
        r
    };
    let point = point.trim_end_matches('/');
    let mut levels = vec![point.to_string()];
    let mut at = String::from(point);
    for part in rel.split('/').filter(|p| !p.is_empty()) {
        at = format!("{at}/{part}");
        levels.push(at.clone());
    }
    let mut q = Quota::Unlimited;
    for dir in &levels {
        if !fs.is_dir(dir) {
            return None;
        }
        match fs.read(&format!("{dir}/cpu.max")) {
            Ok(text) => {
                q = match (q, parse_cpu_max(&text)) {
                    (_, Quota::Unknown) => return None,
                    (Quota::Cpus(a), Quota::Cpus(b)) => Quota::Cpus(a.min(b)),
                    (Quota::Unlimited, b) => b,
                    (a, Quota::Unlimited) => a,
                    (Quota::Unknown, _) => return None,
                };
            }
            Err(e) if e.kind() == NotFound => {}
            Err(_) => return None,
        }
    }
    // the top must be the true root: `cgroup.type` absent there, positively
    match fs.read(&format!("{point}/cgroup.type")) {
        Err(e) if e.kind() == NotFound => Some(q),
        _ => None,
    }
}

/// A `cpu.max` file's quota, strictly: one line, `max PERIOD` or `QUOTA PERIOD`, each a
/// positive integer (in µs); anything else is `Unknown`
fn parse_cpu_max(s: &str) -> Quota {
    let line = s.strip_suffix('\n').unwrap_or(s);
    let f: Vec<&str> = line.split(' ').collect();
    let pos = |t: &str| {
        (!t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()))
            .then(|| t.parse::<u64>().ok())
            .flatten()
            .filter(|&n| n > 0)
    };
    match f.as_slice() {
        [q, p] => match (pos(p), *q) {
            (Some(_), "max") => Quota::Unlimited,
            (Some(p), q) => pos(q).map_or(Quota::Unknown, |q| Quota::Cpus(q as f64 / p as f64)),
            (None, _) => Quota::Unknown,
        },
        _ => Quota::Unknown,
    }
}

#[cfg(test)]
mod front_tests {
    //! The teams' front stage: which path it stages a block by, and that the block staged
    //! inverts to the column's bytes, alone and as a team's source.

    use super::super::block::Symbols;
    use super::super::bwt::{self, SegScratch, TVector};
    use super::super::coop::{invert_team_from, Pool, Source};
    use super::super::runs;
    use super::*;

    /// A column as a block's symbols: its bytes in use, move-to-front indices, and RUNA and
    /// RUNB for the zeros (as bzip2 codes them), under a bound of `max_len` rows
    fn symbols_of(ll: &[u8], max_len: usize) -> Symbols {
        let mut used = [false; 256];
        ll.iter().for_each(|&c| used[c as usize] = true);
        let mut seq_to_unseq = [0u8; 256];
        let mut list: Vec<u8> = (0..=255u8).filter(|&c| used[c as usize]).collect();
        seq_to_unseq[..list.len()].copy_from_slice(&list);
        let eob = list.len() as u16 + 1;
        let (mut syms, mut zeros) = (Vec::new(), 0u32);
        let flush = |zeros: &mut u32, syms: &mut Vec<u16>| {
            // (bijective base 2: RUNA 1, RUNB 2, weights doubling)
            while *zeros > 0 {
                syms.push(((*zeros - 1) & 1) as u16);
                *zeros = (*zeros - 1) >> 1;
            }
        };
        for &c in ll {
            let i = list.iter().position(|&x| x == c).unwrap();
            if i == 0 {
                zeros += 1;
                continue;
            }
            flush(&mut zeros, &mut syms);
            syms.push(i as u16 + 1);
            list.remove(i);
            list.insert(0, c);
        }
        flush(&mut zeros, &mut syms);
        syms.push(eob);
        Symbols {
            crc: 0,
            orig_ptr: 0,
            seq_to_unseq,
            syms,
            eob,
            max_len,
        }
    }

    /// A record `runs::front` never writes (its length is over a record's cap)
    const UNTOUCHED: u16 = 0xffff;

    /// `front_stage` stages a block by its records exactly where `runs::takes_runs` admits
    /// it (one front set reused across both kinds, as a team reuses it), with the block's
    /// rows; and the block staged gives `bwt::walk`'s bytes, inverted alone
    /// (`Front::invert_alone`) and as a team's source (`Front::source`), which is the
    /// records' only when the block was staged by them
    #[test]
    fn front_stage_takes_the_records_where_selected() {
        let mut x = 0x2545_f491_4f6c_dd1du64;
        let mut rnd = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let mut column = |n: usize, runny: bool| -> Vec<u8> {
            let mut v = Vec::with_capacity(n + 64);
            while v.len() < n {
                let (c, len) = if runny {
                    ((rnd() % 6) as u8, 1 + (rnd() % 60) as usize)
                } else {
                    (rnd() as u8, 1)
                };
                v.extend(std::iter::repeat(c).take(len));
            }
            v.truncate(n);
            v
        };
        // (no two in a row of the same length: the front set's rows are the block's own)
        let cases = [
            (bwt::MIN_SEGMENTED + 4321, true),
            (bwt::MIN_SEGMENTED + 5000, false),
            (1000, true),
            (1200, false),
            (bwt::MIN_SEGMENTED + 99, true),
        ];
        let pool = Pool::new(2).unwrap();
        let mut f = Front::default();
        let (mut tt, mut seg) = (TVector::new(), SegScratch::default());
        let (mut by_records, mut by_column) = (0, 0);
        for kern in [Kernels::scalar(), Kernels::detect()] {
            for (i, &(n, runny)) in cases.iter().enumerate() {
                let ll = column(n, runny);
                // (the bound is the length, so a block of a symbol a row keeps the column)
                let mut b = symbols_of(&ll, n);
                b.orig_ptr = (n / 3) as u32;
                let want_path = runs::takes_runs(&b);
                assert_eq!(want_path, runny, "case {i}: the selector");
                let mut want_tt = TVector::new();
                let mut counts = [0u32; 256];
                ll.iter().for_each(|&c| counts[c as usize] += 1);
                bwt::build(&ll, &counts, &mut want_tt);
                let mut want = Vec::new();
                bwt::walk(&want_tt, b.orig_ptr, &mut want);

                *f.runs.records_mut() = vec![UNTOUCHED];
                f.ll = vec![0xee; 3];
                assert_eq!(front_stage(&b, &mut f, kern), Ok(n), "case {i}");
                assert_eq!((f.by_runs, f.rows), (want_path, n), "case {i}: staged");
                let took_records = *f.runs.records_mut() != [UNTOUCHED];
                assert_eq!(took_records, want_path, "case {i}: the records written");
                assert_eq!(f.ll == ll, !want_path, "case {i}: the column written");
                assert_eq!(
                    matches!(f.source(kern), Source::Runs { .. }),
                    want_path,
                    "case {i}: the team's source"
                );

                let join = |seg: &SegScratch, out: &mut Vec<u8>| {
                    if want_path {
                        seg.assemble_reversed(out)
                    } else {
                        seg.assemble(out)
                    }
                };
                let mut out = vec![7; 2];
                let segmented = f.invert_alone(b.orig_ptr, kern, &mut tt, &mut seg, &mut out);
                assert_eq!(segmented, n >= bwt::MIN_SEGMENTED, "case {i}");
                if segmented {
                    join(&seg, &mut out);
                }
                assert!(out == want, "case {i}, {kern:?}: inverted alone");

                invert_team_from::<8>(
                    &pool,
                    f.source(kern),
                    b.orig_ptr,
                    &mut tt,
                    &mut seg,
                    0,
                    None,
                );
                let mut out = vec![7; 2];
                join(&seg, &mut out);
                assert!(out == want, "case {i}, {kern:?}: inverted by a team");
                if want_path {
                    by_records += 1;
                } else {
                    by_column += 1;
                }
            }
        }
        assert!(by_records >= 3 && by_column >= 2);
    }

    /// A bad primary index is refused on either path, with the same error
    #[test]
    fn front_stage_checks_the_primary_index_on_both_paths() {
        for runny in [true, false] {
            let ll: Vec<u8> = (0..4000u32)
                .map(|i| {
                    if runny {
                        (i / 90 % 3) as u8
                    } else {
                        (i.wrapping_mul(2_654_435_761) >> 13) as u8
                    }
                })
                .collect();
            let mut b = symbols_of(&ll, ll.len());
            assert_eq!(runs::takes_runs(&b), runny);
            let mut f = Front::default();
            b.orig_ptr = ll.len() as u32;
            assert_eq!(
                front_stage(&b, &mut f, Kernels::detect()),
                Err(ErrorKind::BadOrigPtr)
            );
            b.orig_ptr -= 1;
            assert_eq!(front_stage(&b, &mut f, Kernels::detect()), Ok(ll.len()));
        }
    }
}

#[cfg(test)]
mod unwind_tests;

#[cfg(test)]
mod service_unwind_tests;

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
        assert_eq!(plan_with(9, i9.0, i9.1), Plan::Teams { teams: 4, k: 2 });
        // the i9 at 10 threads or more: three teams of three (docs/journal/2026-11-06-three-tables.md)
        for t in [10, 11, 12, 16, 20, 64, usize::MAX] {
            assert_eq!(
                plan_with(t, i9.0, i9.1),
                Plan::Teams { teams: 3, k: 3 },
                "{t}"
            );
        }
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

    /// Three teams of three only on the measured topology, and never more decoding threads
    /// than asked for
    #[test]
    fn the_ten_core_exception() {
        let t33 = Plan::Teams { teams: 3, k: 3 };
        // (the plan the default runs pipelined, and the only one: `StreamDecoder::new`)
        assert_eq!(t33, PIPELINED_PLAN);
        let mib = 1u64 << 20;
        // a core more or fewer, or a cache a byte off or unread: the old rule
        for (phys, l3) in [
            (Some(9), Some(20 * mib)),
            (Some(11), Some(20 * mib)),
            (Some(10), Some(20 * mib - 1)),
            (Some(10), Some(20 * mib + 1)),
            (Some(10), Some(16 * mib)),
            (Some(10), None),
            (None, Some(20 * mib)),
        ] {
            for t in [6, 9, 10, 12, 20] {
                let p = plan_with(t, phys, l3);
                assert_ne!(p, t33, "{phys:?} {l3:?} at {t}");
            }
        }
        // the budget: whatever the topology, no plan runs more decoding threads than asked
        for phys in [
            None,
            Some(4),
            Some(6),
            Some(9),
            Some(10),
            Some(11),
            Some(16),
        ] {
            for l3 in [None, Some(16 * mib), Some(20 * mib), Some(32 * mib)] {
                for t in 1..=24 {
                    let p = plan_with(t, phys, l3);
                    assert!(p.threads() <= t, "{phys:?} {l3:?} at {t}: {p:?}");
                }
            }
        }
        assert_eq!((t33.workers(), t33.threads()), (3, 9));
        // its default window (`SLOTS_PER_THREAD` a thread asked for) is 12 blocks, as the
        // registered runs used; fewer slots still give a block a team; saturating at the top
        assert_eq!(window(t33, 10, 40), 12);
        assert_eq!(window(t33, 20, 80), 12);
        assert_eq!(window(t33, 10, 1), 3);
        assert_eq!(window(t33, 10, usize::MAX), usize::MAX / 10);
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

    /// The default decoder's threads are the plan's for this machine, pipelined at
    /// `PIPELINED_PLAN` alone; and with each plan forced, whatever the machine
    #[test]
    fn the_default_is_the_plan() {
        for t in [1, 4, 8] {
            let dec = StreamDecoder::new(t, 2 * t, Kernels::detect());
            assert_eq!(dec.threads(), Plan::for_threads(t).threads());
            assert_eq!(dec.is_pipelined(), Plan::for_threads(t) == PIPELINED_PLAN);
            dec.close();
            let e = dec.read(&mut [0; 16]).unwrap_err();
            assert_eq!(e.kind, ErrorKind::Empty);
        }
        for plan in [
            PIPELINED_PLAN,
            Plan::Workers(4),
            Plan::Teams { teams: 4, k: 2 },
            Plan::Teams { teams: 2, k: 3 },
            Plan::Teams { teams: 3, k: 4 },
        ] {
            let dec = with_plan(plan, || StreamDecoder::new(10, 40, Kernels::detect()));
            assert_eq!(dec.is_pipelined(), plan == PIPELINED_PLAN, "{plan:?}");
            assert_eq!(dec.threads(), plan.threads(), "{plan:?}");
            dec.close();
            let e = dec.read(&mut [0; 16]).unwrap_err();
            assert_eq!(e.kind, ErrorKind::Empty);
        }
    }

    /// The service core's target predicate: every condition alone flips it off
    #[test]
    fn the_service_target() {
        let both = Kernels {
            avx2: true,
            clmul: true,
        };
        let i9 = Machine::service_target();
        let here = cfg!(all(target_os = "linux", target_arch = "x86_64"));
        assert_eq!(service_target(10, both, &i9), here);
        // the budget: exactly 10
        for t in [0, 1, 4, 8, 9, 11, 12, 16, 20, 64, usize::MAX] {
            assert!(!service_target(t, both, &i9), "{t} threads");
        }
        // the kernels: the run's own, both
        for k in [
            Kernels::scalar(),
            Kernels {
                avx2: true,
                clmul: false,
            },
            Kernels {
                avx2: false,
                clmul: true,
            },
        ] {
            assert!(!service_target(10, k, &i9), "{k:?}");
        }
        // each fact of the topology off by a little, or the snapshot unread
        let mib = 1u64 << 20;
        let with = |f: &dyn Fn(&mut Topology)| {
            let mut m = i9.clone();
            f(m.target.as_mut().unwrap());
            m
        };
        let off: Vec<(&str, Machine)> = vec![
            ("9 cores", with(&|t| t.phys = 9)),
            ("11 cores", with(&|t| t.phys = 11)),
            ("19 CPUs", with(&|t| t.logical = 19)),
            ("10 CPUs (no SMT)", with(&|t| t.logical = 10)),
            ("L3 a byte less", with(&|t| t.l3 = 20 * mib - 1)),
            ("L3 16 MiB", with(&|t| t.l3 = 16 * mib)),
            ("model 166", with(&|t| t.cpu.2 = 166)),
            ("model 158", with(&|t| t.cpu.2 = 158)),
            ("family 15", with(&|t| t.cpu.1 = 15)),
            ("AMD", with(&|t| t.cpu.0 = "AuthenticAMD".into())),
            // the other 10-core Comet Lake-S parts: CPUID's family and model, another name
            (
                "i9-10900KF",
                with(&|t| t.name = "Intel(R) Core(TM) i9-10900KF CPU @ 3.70GHz".into()),
            ),
            (
                "i9-10900",
                with(&|t| t.name = "Intel(R) Core(TM) i9-10900 CPU @ 2.80GHz".into()),
            ),
            (
                "i9-10850K",
                with(&|t| t.name = "Intel(R) Core(TM) i9-10850K CPU @ 3.60GHz".into()),
            ),
            (
                "Xeon W-1290P",
                with(&|t| t.name = "Intel(R) Xeon(R) W-1290P CPU @ 3.70GHz".into()),
            ),
            ("no name", with(&|t| t.name = String::new())),
            (
                "the name padded",
                with(&|t| t.name = format!(" {SERVICE_NAME}")),
            ),
            (
                "no snapshot",
                Machine {
                    target: None,
                    ..i9.clone()
                },
            ),
            (
                "quota 9.5",
                Machine {
                    quota: Quota::Cpus(9.5),
                    ..i9.clone()
                },
            ),
            (
                "quota 2",
                Machine {
                    quota: Quota::Cpus(2.0),
                    ..i9.clone()
                },
            ),
            (
                "quota unknown",
                Machine {
                    quota: Quota::Unknown,
                    ..i9.clone()
                },
            ),
            ("the default machine", Machine::default()),
        ];
        for (what, m) in &off {
            assert!(!service_target(10, both, m), "{what}");
        }
        // the plan policy's own facts are not the predicate's
        let m = Machine {
            phys: None,
            l3: None,
            ..i9.clone()
        };
        assert_eq!(service_target(10, both, &m), here);
        // any established limit, however large, turns it off: unlimited alone was measured
        for q in [10.0, 12.5, 20.0, 64.0, 1e9] {
            let m = Machine {
                quota: Quota::Cpus(q),
                ..i9.clone()
            };
            assert!(!service_target(10, both, &m), "quota {q}");
        }
        assert_eq!(Quota::default(), Quota::Unknown);
        assert_eq!(SERVICE_PLAN.threads(), 8);
    }

    /// A machine's files for discovery's tests: each path's text, or the error reading it;
    /// a path not given is not found, and a directory is there if listed
    #[derive(Default, Clone)]
    struct FakeFs {
        files: std::collections::BTreeMap<String, Result<String, std::io::ErrorKind>>,
        dirs: std::collections::BTreeSet<String>,
    }

    impl FakeFs {
        fn file(mut self, p: &str, s: &str) -> Self {
            self.files.insert(p.into(), Ok(s.into()));
            self
        }
        fn fails(mut self, p: &str, e: std::io::ErrorKind) -> Self {
            self.files.insert(p.into(), Err(e));
            self
        }
        fn dir(mut self, p: &str) -> Self {
            self.dirs.insert(p.into());
            self
        }
        fn without(mut self, p: &str) -> Self {
            self.files.remove(p);
            self.dirs.remove(p);
            self
        }
    }

    impl Fs for FakeFs {
        fn read(&self, p: &str) -> std::io::Result<String> {
            match self.files.get(p) {
                Some(Ok(s)) => Ok(s.clone()),
                Some(Err(e)) => Err((*e).into()),
                None => Err(std::io::ErrorKind::NotFound.into()),
            }
        }
        fn is_dir(&self, p: &str) -> bool {
            self.dirs.contains(p)
        }
    }

    /// A desktop's cgroup v2 hierarchy: the process in /user.slice/u/app, the true root
    /// mounted at /sys/fs/cgroup, no `cpu.max` anywhere (the controller enabled nowhere)
    fn hierarchy() -> FakeFs {
        let root = "/sys/fs/cgroup";
        FakeFs::default()
            .file("/proc/self/cgroup", "0::/user.slice/u/app\n")
            .file(
                "/proc/self/mountinfo",
                "22 1 0:21 / / rw - ext4 /dev/vda rw\n\
                 30 22 0:26 / /sys/fs/cgroup rw,nosuid - cgroup2 cgroup2 rw\n",
            )
            .dir(root)
            .dir("/sys/fs/cgroup/user.slice")
            .dir("/sys/fs/cgroup/user.slice/u")
            .dir("/sys/fs/cgroup/user.slice/u/app")
            .file("/sys/fs/cgroup/user.slice/cgroup.type", "domain\n")
    }

    /// Discovery's quota fails closed: unreadable membership, an unreadable or restrictive
    /// parent, no or a relocated v2 mount, cgroup v1 alone, a hidden ancestor (a cgroup
    /// namespace) are `Unknown` or the limit; an established unlimited hierarchy, and
    /// inherited quotas below and at ten, are what they are
    #[test]
    fn discovery_fails_closed_quota() {
        let (leaf, parent) = (
            "/sys/fs/cgroup/user.slice/u/app/cpu.max",
            "/sys/fs/cgroup/user.slice/cpu.max",
        );
        let i9 = Machine::service_target();
        let both = Kernels {
            avx2: true,
            clmul: true,
        };
        let here = cfg!(all(target_os = "linux", target_arch = "x86_64"));
        let on = |fs: &FakeFs| {
            service_target(
                10,
                both,
                &Machine {
                    quota: cpu_quota(fs),
                    ..i9.clone()
                },
            )
        };
        let unlimited = if here {
            Quota::Unlimited
        } else {
            Quota::Unknown
        };
        let cases: Vec<(&str, FakeFs, Quota)> = vec![
            ("established unlimited", hierarchy(), unlimited),
            (
                "a leaf's max",
                hierarchy().file(leaf, "max 100000\n"),
                unlimited,
            ),
            (
                "membership unreadable",
                hierarchy().fails("/proc/self/cgroup", std::io::ErrorKind::PermissionDenied),
                Quota::Unknown,
            ),
            (
                "membership missing",
                hierarchy().without("/proc/self/cgroup"),
                Quota::Unknown,
            ),
            (
                "a parent's cpu.max unreadable",
                hierarchy().fails(parent, std::io::ErrorKind::PermissionDenied),
                Quota::Unknown,
            ),
            (
                "a parent's cpu.max an I/O error, beneath an unlimited leaf",
                hierarchy()
                    .file(leaf, "max 100000\n")
                    .fails(parent, std::io::ErrorKind::Other),
                Quota::Unknown,
            ),
            (
                "a restrictive parent beneath an unlimited leaf",
                hierarchy()
                    .file(leaf, "max 100000\n")
                    .file(parent, "800000 100000\n"),
                if here {
                    Quota::Cpus(8.0)
                } else {
                    Quota::Unknown
                },
            ),
            (
                "an inherited quota of ten",
                hierarchy().file(parent, "1000000 100000\n"),
                if here {
                    Quota::Cpus(10.0)
                } else {
                    Quota::Unknown
                },
            ),
            (
                "the tightest of two",
                hierarchy()
                    .file(leaf, "1500000 100000\n")
                    .file(parent, "1200000 100000\n"),
                if here {
                    Quota::Cpus(12.0)
                } else {
                    Quota::Unknown
                },
            ),
            (
                "a parent's cpu.max unparsable",
                hierarchy().file(parent, "max garbage\n"),
                Quota::Unknown,
            ),
            (
                "a level's directory missing",
                hierarchy().without("/sys/fs/cgroup/user.slice/u"),
                Quota::Unknown,
            ),
            (
                "no cgroup2 mount",
                hierarchy().file(
                    "/proc/self/mountinfo",
                    "22 1 0:21 / / rw - ext4 /dev/vda rw\n",
                ),
                Quota::Unknown,
            ),
            (
                "mountinfo unreadable",
                hierarchy().fails("/proc/self/mountinfo", std::io::ErrorKind::PermissionDenied),
                Quota::Unknown,
            ),
            (
                "a relocated mount (its root not the process's ancestor)",
                hierarchy().file(
                    "/proc/self/mountinfo",
                    "30 22 0:26 /system.slice /sys/fs/cgroup rw - cgroup2 cgroup2 rw\n",
                ),
                Quota::Unknown,
            ),
            (
                "two cgroup2 mounts",
                hierarchy().file(
                    "/proc/self/mountinfo",
                    "30 22 0:26 / /sys/fs/cgroup rw - cgroup2 cgroup2 rw\n\
                     31 22 0:26 / /mnt/cg rw - cgroup2 cgroup2 rw\n",
                ),
                Quota::Unknown,
            ),
            (
                "cgroup v1 alone",
                hierarchy().file(
                    "/proc/self/cgroup",
                    "4:cpu,cpuacct:/user.slice\n1:name=systemd:/x\n",
                ),
                Quota::Unknown,
            ),
            (
                "a hidden ancestor (the mount's top not the true root)",
                hierarchy().file("/sys/fs/cgroup/cgroup.type", "domain\n"),
                Quota::Unknown,
            ),
            (
                "the root's cgroup.type unreadable",
                hierarchy().fails(
                    "/sys/fs/cgroup/cgroup.type",
                    std::io::ErrorKind::PermissionDenied,
                ),
                Quota::Unknown,
            ),
        ];
        for (what, fs, want) in &cases {
            assert_eq!(cpu_quota(fs), *want, "{what}");
            let admits = matches!(want, Quota::Unlimited);
            assert_eq!(on(fs), here && admits, "{what}: the predicate");
        }
        // `cpu.max`, strictly
        assert_eq!(parse_cpu_max("max 100000\n"), Quota::Unlimited);
        assert_eq!(parse_cpu_max("200000 100000\n"), Quota::Cpus(2.0));
        assert_eq!(parse_cpu_max("1000000 100000"), Quota::Cpus(10.0));
        for bad in [
            "",
            "\n",
            "x 100000",
            "100000 0",
            "100000",
            "-5 100000",
            "max garbage",
            "max",
            "inf 100000",
            "1e6 100000",
            "max 100000 5",
            "max  100000",
            "+5 100000",
            "max 100000\n\n",
            "0 100000",
        ] {
            assert_eq!(parse_cpu_max(bad), Quota::Unknown, "{bad:?}");
        }
    }

    /// A ten-core, twenty-CPU Comet Lake-S as sysfs and cpuinfo show it, CPU n on core
    /// n % 10
    fn cpus() -> FakeFs {
        let sys = "/sys/devices/system/cpu";
        let mut fs = FakeFs::default()
            .file("/proc/self/status", "Name:\tx\nCpus_allowed_list:\t0-19\n")
            .file(&format!("{sys}/online"), "0-19\n")
            .file(&format!("{sys}/present"), "0-19\n");
        let mut info = String::new();
        for c in 0..20 {
            let t = format!("{sys}/cpu{c}");
            fs = fs
                .file(&format!("{t}/topology/core_id"), &format!("{}\n", c % 10))
                .file(&format!("{t}/topology/physical_package_id"), "0\n")
                .file(&format!("{t}/cache/index3/level"), "3\n")
                .file(&format!("{t}/cache/index3/size"), "20480K\n");
            info += &format!(
                "processor\t: {c}\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 165\n\
                 model name\t: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n\n"
            );
        }
        fs.file("/proc/cpuinfo", &info)
    }

    /// Discovery's topology fails closed: a read error is never an offline CPU, and the CPUs
    /// must agree; CPUs positively offline or absent are left out
    #[test]
    fn discovery_fails_closed_topology() {
        if !cfg!(target_os = "linux") {
            assert_eq!(target_topology(&cpus()), None);
            return;
        }
        let sys = "/sys/devices/system/cpu";
        let i9 = Machine::service_target().target;
        assert_eq!(target_topology(&cpus()), i9);
        let pd = std::io::ErrorKind::PermissionDenied;
        let none: Vec<(&str, FakeFs)> = vec![
            (
                "a core id unreadable",
                cpus().fails(&format!("{sys}/cpu7/topology/core_id"), pd),
            ),
            (
                "a core id missing",
                cpus().without(&format!("{sys}/cpu7/topology/core_id")),
            ),
            (
                "a package unreadable",
                cpus().fails(&format!("{sys}/cpu3/topology/physical_package_id"), pd),
            ),
            (
                "an L3 unreadable",
                cpus().fails(&format!("{sys}/cpu12/cache/index3/size"), pd),
            ),
            (
                "an L3 of another size",
                cpus().file(&format!("{sys}/cpu12/cache/index3/size"), "16384K\n"),
            ),
            (
                "index3 not level 3",
                cpus().file(&format!("{sys}/cpu0/cache/index3/level"), "2\n"),
            ),
            (
                "online unreadable",
                cpus().fails(&format!("{sys}/online"), pd),
            ),
            (
                "present unreadable",
                cpus().without(&format!("{sys}/present")),
            ),
            ("status unreadable", cpus().fails("/proc/self/status", pd)),
            (
                "affinity unparsable",
                cpus().file("/proc/self/status", "Cpus_allowed_list:\t0-x\n"),
            ),
            ("cpuinfo unreadable", cpus().fails("/proc/cpuinfo", pd)),
            ("a CPU of another model", {
                let fs = cpus();
                let info = fs.read("/proc/cpuinfo").unwrap().replacen(
                    "processor\t: 19\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 165",
                    "processor\t: 19\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 166",
                    1,
                );
                fs.file("/proc/cpuinfo", &info)
            }),
            ("a CPU with no cpuinfo record", {
                let fs = cpus();
                let info = fs.read("/proc/cpuinfo").unwrap().replacen(
                    "processor\t: 19\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 165\n\
                         model name\t: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n\n",
                    "",
                    1,
                );
                assert_ne!(info, fs.read("/proc/cpuinfo").unwrap());
                fs.file("/proc/cpuinfo", &info)
            }),
            ("a CPU of another name", {
                let fs = cpus();
                let info = fs.read("/proc/cpuinfo").unwrap().replacen(
                    "model\t\t: 165\nmodel name\t: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n\n",
                    "model\t\t: 165\nmodel name\t: Intel(R) Core(TM) i9-10900KF CPU @ 3.70GHz\n\n",
                    1,
                );
                assert_ne!(info, fs.read("/proc/cpuinfo").unwrap());
                fs.file("/proc/cpuinfo", &info)
            }),
            ("a CPU with no model name", {
                let fs = cpus();
                let info = fs.read("/proc/cpuinfo").unwrap().replacen(
                    "model name\t: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n",
                    "",
                    1,
                );
                assert_ne!(info, fs.read("/proc/cpuinfo").unwrap());
                fs.file("/proc/cpuinfo", &info)
            }),
        ];
        for (what, fs) in &none {
            assert_eq!(target_topology(fs), None, "{what}");
        }
        // positively offline (sysfs's online list) and absent CPUs are left out, not errors:
        // the snapshot then is a smaller machine, not the target
        let offline = cpus().file(&format!("{sys}/online"), "0-18\n");
        let t = target_topology(&offline).unwrap();
        assert_eq!((t.logical, t.phys), (19, 10));
        let absent = cpus()
            .file("/proc/self/status", "Cpus_allowed_list:\t0-63\n")
            .fails(&format!("{sys}/cpu40/topology/core_id"), pd);
        assert_eq!(target_topology(&absent), i9, "CPUs allowed but not present");
        let masked = cpus().file("/proc/self/status", "Cpus_allowed_list:\t0-9\n");
        let t = target_topology(&masked).unwrap();
        assert_eq!((t.logical, t.phys), (10, 10));
    }

    /// `StreamDecoder::new`'s own dispatch on the target machine (`with_machine`): the
    /// service core at exactly 10 threads, its window scaled to its two teams, and the
    /// existing plan at every other budget, on the scalar kernels, off the target, and with
    /// a plan forced
    #[test]
    fn the_service_default() {
        if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            return;
        }
        let both = Kernels {
            avx2: true,
            clmul: true,
        };
        let i9 = Machine::service_target();
        let build = |m: &Machine, t: usize, slots: usize, k: Kernels| {
            let (dec, b) =
                observe_builds(|| with_machine(m.clone(), || StreamDecoder::new(t, slots, k)));
            assert_eq!(b.len(), 1);
            assert_eq!(
                (b[0].service, b[0].window),
                (dec.is_service(), dec.window())
            );
            dec
        };
        // the target: the service core, 8 decoding threads, a window of 8 at 4 a thread
        let dec = build(&i9, 10, 40, both);
        assert!(dec.is_service() && !dec.is_pipelined());
        assert_eq!((dec.threads(), dec.window()), (8, 8));
        assert_eq!(dec.service_counts(), Some(ServiceCounts::default()));
        // custom windows scale as a plan's do (`window`), at least a block a team
        for (slots, want) in [
            (1, 2),
            (2, 2),
            (10, 2),
            (20, 4),
            (40, 8),
            (41, 8),
            (60, 12),
            (80, 16),
        ] {
            assert_eq!(build(&i9, 10, slots, both).window(), want, "{slots} slots");
        }
        assert_eq!(
            build(&i9, 10, usize::MAX, both).window(),
            (usize::MAX / 10) as u64
        );
        // every other budget: what the plan policy says on this machine, as before
        for t in [1, 4, 8, 9, 11, 12, 16, 20] {
            let dec = build(&i9, t, 4 * t, both);
            let plan = with_machine(i9.clone(), || Plan::for_threads(t));
            assert!(!dec.is_service(), "{t}");
            assert_eq!(dec.threads(), plan.threads(), "{t}");
            assert_eq!(dec.is_pipelined(), plan == PIPELINED_PLAN, "{t}");
            assert_eq!(dec.window(), window(plan, t, 4 * t) as u64, "{t}");
        }
        // 10 threads on the scalar kernels, or off the target: the three pipelined teams
        // of three (the existing 10-core rule), or the plan for that machine
        let dec = build(&i9, 10, 40, Kernels::scalar());
        assert!(!dec.is_service() && dec.is_pipelined() && dec.threads() == 9);
        let mut amd = i9.clone();
        amd.target.as_mut().unwrap().cpu = ("AuthenticAMD".into(), 25, 33);
        let dec = build(&amd, 10, 40, both);
        assert!(!dec.is_service() && dec.is_pipelined() && dec.window() == 12);
        let small = Machine {
            phys: Some(4),
            target: None,
            ..i9.clone()
        };
        let dec = build(&small, 10, 40, both);
        assert!(!dec.is_service() && !dec.is_pipelined() && dec.threads() == 10);
        // a forced plan wins, on the target too
        for plan in [
            PIPELINED_PLAN,
            Plan::Workers(10),
            Plan::Teams { teams: 4, k: 2 },
        ] {
            let dec = with_plan(plan, || build(&i9, 10, 40, both));
            assert!(!dec.is_service(), "{plan:?}");
            assert_eq!(dec.threads(), plan.threads(), "{plan:?}");
        }
        // and it decodes: four streams, through the service core's own threads
        let dec = build(&i9, 10, 40, both);
        dec.write(service_tests::Z.to_vec()).unwrap();
        dec.close();
        let mut out = Vec::new();
        let mut buf = [0u8; 64];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => panic!("{e}"),
            }
        }
        assert_eq!(out, service_tests::want());
        // (EOF is not the workers' end: the last block is read once published, before its
        // finisher retires it, D067; join is the fence, and `ended` its witness)
        dec.join();
        let c = dec.service_counts().unwrap();
        assert!(c.ended, "{c:?}");
        assert_eq!((c.prepared, c.finished), (4, 4));
        // (none of these blocks is big enough for a team's pool: tests/service_default.rs
        // reaches it, on full blocks)
        assert_eq!(c.pooled, 0);
    }

    /// The target-gated route's condition 1 (docs/STEWARDSHIP.md;
    /// docs/journal/2026-11-10-service-guarded-target-gated-registration.md): on the machine
    /// the tests run on, as discovery reads it now (no `with_machine`), the predicate is
    /// evaluated at every budget, and wherever it is false the public dispatch builds
    /// master's decoder: `Plan::for_threads`'s plan, pipelined exactly at `PIPELINED_PLAN`,
    /// its threads and its window (`window`), never the service core, through
    /// `StreamDecoder::new`, `BzReader::parallel` and `decode_parallel`. On a CI runner
    /// (`GITHUB_ACTIONS=true`) the predicate must be false at every budget: a runner where
    /// it held would be a runner the off-target bench does not describe. The facts read are
    /// printed, so each runner's log records them. This shows fallback selection only, not
    /// that discovery, layout or shared code cost nothing there: the CI bench bounds that
    #[test]
    fn off_target_here_builds_masters_plan() {
        let here = Machine::here();
        let k = Kernels::detect();
        let ci = std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true");
        eprintln!("off_target_here: CI {ci}, kernels {k:?}, machine {here:?}");
        let budgets: Vec<usize> = (1..=24).chain([32, 48, 64]).collect();
        for &t in &budgets {
            let on = service_target(t, k, &here);
            if ci {
                assert!(
                    !on,
                    "the predicate holds on this CI runner at {t} threads: {here:?}"
                );
            }
            if on {
                continue; // (the target itself: tests/service_default.rs and the_service_default)
            }
            let plan = Plan::for_threads(t);
            let want = (false, plan == PIPELINED_PLAN, plan.threads());
            let slots = SLOTS_PER_THREAD * t;
            let (dec, b) = observe_builds(|| StreamDecoder::new(t, slots, k));
            assert_eq!(b.len(), 1, "{t}");
            let got = (b[0].service, b[0].pipelined, b[0].threads);
            assert_eq!(got, want, "StreamDecoder::new at {t} threads: {plan:?}");
            assert_eq!(
                b[0].window,
                window(plan, t, slots) as u64,
                "{t}: the window"
            );
            assert!(dec.service_counts().is_none());
            dec.close();
            let (out, b) = observe_builds(|| {
                let mut r = crate::BzReader::parallel(service_tests::Z, t);
                let mut out = Vec::new();
                std::io::Read::read_to_end(&mut r, &mut out).unwrap();
                out
            });
            assert_eq!(out, service_tests::want(), "BzReader::parallel at {t}");
            assert_eq!(b.len(), 1, "{t}");
            let got = (b[0].service, b[0].pipelined, b[0].threads);
            assert_eq!(got, want, "BzReader::parallel at {t} threads: {plan:?}");
            assert_eq!(
                b[0].window,
                window(plan, t, slots) as u64,
                "{t}: the window"
            );
            let (out, b) = observe_builds(|| crate::decode_parallel(service_tests::Z, t).unwrap());
            assert_eq!(out, service_tests::want(), "decode_parallel at {t}");
            assert_eq!(b.len(), 1, "{t}");
            let got = (b[0].service, b[0].pipelined, b[0].threads);
            assert_eq!(got, want, "decode_parallel at {t} threads: {plan:?}");
        }
        // the plan itself is master's: the policy over the plan policy's two facts alone,
        // read as master reads them
        for &t in &budgets {
            assert_eq!(
                Plan::for_threads(t),
                plan_with(t, physical_cores(), l3_bytes()),
                "{t}"
            );
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
