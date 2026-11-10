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
            Plan::Workers(n) => Self::with_pairs(n, slots, k),
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

    /// Production's `threads` workers (`Segmented`), in SMT sibling pairs where more
    /// workers than physical cores share them (`pairs::place`): each pair pinned to its
    /// core, sharing a walk token and the core's two tables (`paired_worker`). Where no
    /// core is shared (1 or 2 threads on CI's 2 cores, no SMT, the topology unread), it
    /// is `with_inverse::<Segmented>`, today's decoder.
    pub fn with_pairs(threads: usize, slots: usize, k: Kernels) -> StreamDecoder {
        match super::pairs::allowed_cores() {
            Some(cores) => Self::with_pairs_on(threads, slots, k, &cores),
            None => Self::with_inverse::<super::bwt::Segmented>(threads.max(1), slots, k),
        }
    }

    /// `with_pairs` on the cores `cores` (each core's CPUs) as if they were this
    /// machine's: for tests, which impose CI's 2 cores x 2 siblings on any machine
    #[doc(hidden)]
    pub fn with_pairs_on(
        threads: usize,
        slots: usize,
        k: Kernels,
        cores: &[Vec<usize>],
    ) -> StreamDecoder {
        use super::pairs::{pin, place, Pair};
        let threads = threads.max(1);
        let Some(placement) = place(threads, cores) else {
            return Self::with_inverse::<super::bwt::Segmented>(threads, slots, k);
        };
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
        let (job_tx, job_rx) = channel::<Job>();
        let job_rx = Arc::new(Mutex::new(job_rx));
        let npairs = placement
            .iter()
            .filter_map(|p| p.pair)
            .map(|(p, _)| p + 1)
            .max();
        let pairs: Vec<Arc<Pair>> = (0..npairs.unwrap_or(0))
            .map(|_| Arc::new(Pair::new()))
            .collect();
        let mut handles = Vec::with_capacity(threads + 1);
        for (i, pl) in placement.into_iter().enumerate() {
            let (shared, job_rx) = (shared.clone(), job_rx.clone());
            let pair = pl.pair.map(|(p, m)| (pairs[p].clone(), m));
            handles.push(
                std::thread::Builder::new()
                    .name(format!("mibz2-{i}"))
                    .spawn(move || {
                        if !pl.cpus.is_empty() {
                            let _ = pin(&pl.cpus);
                        }
                        match pair {
                            Some((pair, me)) => paired_worker(&shared, &job_rx, k, &pair, me),
                            None => worker::<super::bwt::Segmented>(&shared, &job_rx, k),
                        }
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

/// A block a paired worker holds between its stages: its last column (the walk's output
/// is written over it) and counts, and once built, one of its core's two tables
struct Held {
    place: u64,
    bit: u64,
    orig_ptr: u32,
    crc: u32,
    ll: Vec<u8>,
    counts: [u32; 256],
    tt: Option<super::bwt::TVector>,
}

/// A worker of a sibling pair (`pairs::Pair`): it holds up to two blocks, and at each
/// stage's end does the first of these it can:
/// 1. walk its oldest built block, if the core's token is free (or was handed to it);
/// 2. build an unbuilt block, if one of the core's two tables is free;
/// 3. take a new block (fewer than two held) and undo its MTF: waiting for one only when
///    it holds none, so a worker holding a block never blocks on the queue (`try_lock`:
///    an idle worker asleep in `recv` holds the lock, and any job goes to it);
/// 4. sleep until its sibling's walk ends (which hands it the token if it is waiting to
///    walk, and frees a table either way).
///
/// No wait is unbounded while a block is held: the token is held only across a walk,
/// which waits on nothing, and a worker sleeps only while its sibling is walking (it
/// holds a built block and the token is taken, or an unbuilt one and both tables are, and
/// a table is only ever held by a built block or a walk). So every block held reaches the
/// reader, and the window and the ordered output are as `worker`'s.
fn paired_worker(
    shared: &Shared,
    jobs: &Mutex<Receiver<Job>>,
    k: Kernels,
    pair: &super::pairs::Pair,
    me: usize,
) {
    use super::bwt::{build, SegScratch};
    use super::pairs::Want;
    use std::sync::mpsc::TryRecvError;
    let mut held: Vec<Held> = Vec::with_capacity(2);
    let mut lls: Vec<Vec<u8>> = Vec::with_capacity(2);
    let mut seg = SegScratch::default();
    let mut closed = false;
    loop {
        // 1. walk
        if let Some(i) = held.iter().position(|h| h.tt.is_some()) {
            if pair.try_walk(me) {
                let h = held.remove(i);
                walk_and_finish(shared, pair, me, h, &mut seg, k, &mut lls);
                continue;
            }
        }
        // 2. build
        if let Some(i) = held.iter().position(|h| h.tt.is_none()) {
            if let Some(mut tt) = pair.take_table() {
                let h = &mut held[i];
                match catch_unwind(AssertUnwindSafe(|| build(&h.ll, &h.counts, &mut tt))) {
                    Ok(()) => {
                        h.tt = Some(tt);
                        pair.built(me);
                    }
                    Err(p) => {
                        pair.give_back(me, super::bwt::TVector::new(), false);
                        let h = held.remove(i);
                        publish(
                            shared,
                            h.place,
                            h.bit,
                            Err(ErrorKind::Internal(panic_message(p))),
                        );
                    }
                }
                continue;
            }
        }
        // 3. a new block
        if held.len() < 2 && !closed {
            let job = if held.is_empty() {
                mark!(JobLock);
                let rx = jobs.lock().unwrap_or_else(|e| e.into_inner());
                mark!(JobRecv);
                rx.recv().map_err(|_| TryRecvError::Disconnected)
            } else {
                match jobs.try_lock() {
                    Ok(rx) => rx.try_recv(),
                    Err(std::sync::TryLockError::Poisoned(e)) => e.into_inner().try_recv(),
                    Err(std::sync::TryLockError::WouldBlock) => Err(TryRecvError::Empty),
                }
            };
            match job {
                Ok(job) => {
                    if let Some(h) = front_mtf(shared, job, k, &mut lls) {
                        held.push(h);
                    }
                    continue;
                }
                Err(TryRecvError::Disconnected) => closed = true,
                Err(TryRecvError::Empty) => {}
            }
        }
        if held.is_empty() {
            debug_assert!(closed);
            return;
        }
        // 4. sleep until the sibling's walk ends
        pair.sleep(
            me,
            Want {
                walk: held.iter().any(|h| h.tt.is_some()),
                table: held.iter().any(|h| h.tt.is_none()),
            },
        );
    }
}

/// A new block's first stage (MTF, RLE2), its symbols back to the scanner as soon as it
/// has read them; None if it fails (its error published) or the decode is cancelled
fn front_mtf(shared: &Shared, job: Job, k: Kernels, lls: &mut Vec<Vec<u8>>) -> Option<Held> {
    let (place, bit, symbols) = job;
    mark!(JobGot, place);
    if shared.lock().cancelled {
        return None; // (drain the queue, so the scanner never waits on it)
    }
    mark!(DecodeStart, place);
    let mut ll = lls.pop().unwrap_or_default();
    let mut counts = [0u32; 256];
    let res = catch_unwind(AssertUnwindSafe(|| {
        super::mtf::mtf(&symbols, &mut ll, &mut counts, k)
    }));
    let (orig_ptr, crc) = (symbols.orig_ptr, symbols.crc);
    {
        let mut st = shared.lock();
        if st.free_syms.len() < shared.slots as usize {
            st.free_syms.push(symbols.syms);
        }
    }
    let res = match res {
        Ok(Ok(n)) if (orig_ptr as usize) < n => Ok(()),
        Ok(Ok(_)) => Err(ErrorKind::BadOrigPtr),
        Ok(Err(kind)) => Err(kind),
        Err(p) => Err(ErrorKind::Internal(panic_message(p))),
    };
    match res {
        Ok(()) => Some(Held {
            place,
            bit,
            orig_ptr,
            crc,
            ll,
            counts,
            tt: None,
        }),
        Err(kind) => {
            lls.push(ll);
            publish(shared, place, bit, Err(kind));
            None
        }
    }
}

/// A built block's walk, holding the core's token, then (the token and its table given
/// back first, so the sibling may walk) RLE1 and the CRC into an output buffer
fn walk_and_finish(
    shared: &Shared,
    pair: &super::pairs::Pair,
    me: usize,
    mut h: Held,
    seg: &mut super::bwt::SegScratch,
    k: Kernels,
    lls: &mut Vec<Vec<u8>>,
) {
    let mut tt = h.tt.take().expect("a built block");
    let walked = catch_unwind(AssertUnwindSafe(|| {
        super::bwt::walk_segments_room(&mut tt, h.orig_ptr, &mut h.ll, seg)
    }));
    if let Err(p) = walked {
        *seg = super::bwt::SegScratch::default();
        pair.end_walk(me, super::bwt::TVector::new());
        publish(
            shared,
            h.place,
            h.bit,
            Err(ErrorKind::Internal(panic_message(p))),
        );
        return;
    }
    pair.end_walk(me, tt);
    let out = {
        let mut st = shared.lock();
        if st.cancelled {
            drop(st);
            lls.push(h.ll);
            return;
        }
        st.free_out.pop().unwrap_or_default()
    };
    let mut out = out;
    out.clear();
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), ErrorKind> {
        super::rle::unrle(&h.ll, &mut out, k)?;
        let computed = super::crc::crc32(&out, k);
        if computed != h.crc {
            return Err(ErrorKind::BlockCrc {
                stored: h.crc,
                computed,
            });
        }
        Ok(())
    }));
    mark!(DecodeEnd, h.place, out.len(), out.capacity());
    let res = match res {
        Ok(Ok(())) => Ok(out),
        Ok(Err(kind)) => Err(kind),
        Err(p) => Err(ErrorKind::Internal(panic_message(p))),
    };
    lls.push(h.ll);
    publish(shared, h.place, h.bit, res);
}

/// A block's result to the reader (dropped if the decode is cancelled)
fn publish(shared: &Shared, place: u64, bit: u64, res: Result<Vec<u8>, ErrorKind>) {
    let res = res.map_err(|kind| Error {
        kind,
        block: Some(place),
        bit: Some(bit),
    });
    let mut st = shared.lock();
    if !st.cancelled {
        st.ready.insert(place, res);
    }
    drop(st);
    mark!(Ready, place);
    shared.cv.notify_all();
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
