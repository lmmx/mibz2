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
    pub fn new(threads: usize, slots: usize, k: Kernels) -> StreamDecoder {
        let threads = threads.max(1);
        let plan = Plan::for_threads(threads);
        Self::defaulted(plan, window(plan, threads, slots), k)
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
                    .spawn(move || {
                        super::threads::ran();
                        work(&shared, &job_rx)
                    })
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

// The pipelined team with a run-fed leader (research, opt-in: the `run-fed` feature,
// `StreamDecoder::pipelined_run_fed`; docs/journal/2026-11-09-run-fed-leader.md)
#[cfg(any(test, feature = "run-fed"))]
mod run_team;
#[cfg(any(test, feature = "run-fed"))]
#[doc(hidden)]
pub use run_team::{run_fed_phases, RunFedSort};

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
// So a team keeps two front sets (`ll`, `counts`, `checkpoints`) and two assembled
// buffers, one table, pool and segment scratch, and the one-core kernel's small set.
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

/// A block's move-to-front stage's buffers (`Scratch`'s first three)
struct Front {
    ll: Vec<u8>,
    counts: [u32; 256],
    checkpoints: Vec<[u32; 256]>,
}

impl Default for Front {
    fn default() -> Self {
        Front {
            ll: Vec::new(),
            counts: [0; 256],
            checkpoints: Vec::new(),
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
        vec![
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
        ]
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
/// (`decode_block`'s first steps): its length
fn front_stage(b: &super::block::Symbols, f: &mut Front, k: Kernels) -> Result<usize, ErrorKind> {
    let n = super::mtf::mtf_checkpoints(b, &mut f.ll, &mut f.counts, &mut f.checkpoints, k)?;
    if b.orig_ptr as usize >= n {
        return Err(ErrorKind::BadOrigPtr);
    }
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
        super::bwt::build(&f.ll, &f.counts, &mut sc.tt);
        super::bwt::walk_segments_room(&mut sc.tt, s.symbols.orig_ptr, &mut sc.buf, &mut sc.seg);
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
            super::coop::invert_pipelined_with::<FLAGGED_LANES, PARK, SPIN>(
                pool,
                &hf.ll,
                &hf.counts,
                &hf.checkpoints,
                super::mtf::CHECKPOINT,
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

#[cfg(test)]
mod unwind_tests;

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
