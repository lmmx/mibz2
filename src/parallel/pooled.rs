//! Research only (`pooled-scan`): the pooled scan's chain thread and workers
//! (`crate::pooled_scan`; docs/journal/2026-11-24-algorithm-level/pooled-scan/registration.md).
//!
//! - **The chain thread** (in the scanner's place) holds the input (`ChannelSource`, as the
//!   scanner does), finds the magics in it as it arrives, and hands each block candidate's bytes
//!   (its first byte to the next magic's, and 16 more) to the workers in bit order, while the
//!   blocks given a place but not read, and the candidates not yet resolved, number under the
//!   window (`slots`). It walks the stream with `Scanner::next_hinted`: at a block position of
//!   a level-9 stream it waits for the worker's read of the candidate at exactly that bit, and
//!   takes it if the read gave a block and the candidate's ordinal is the block's place.
//!   Anywhere else it reads the item itself and hands the block on as the shipped scanner does.
//! - **A worker** takes the next job: a candidate, which it reads (the symbols, at level 9's
//!   bound) and reports, then decodes and publishes by its ordinal; or a block the chain read,
//!   decoded as the shipped worker decodes it.
//! - **The reader** is the shipped reader: it takes the blocks by place.
//!
//! A worker's block reaches the reader only once the chain has taken its read at that place;
//! a candidate stepped over, read badly or out of order is dropped. A decode whose candidates
//! stop matching places (a false or missed magic) or that meets a stream below level 9 leaves
//! the pooled path for the rest of its input, and runs as the shipped scanner runs.

use super::{
    catch_unwind, panic_message, Arc, AssertUnwindSafe, ChannelSource, Condvar, Error, ErrorKind,
    Event, Job, Kernels, Mutex, Ordering, Receiver, Scanner, Scratch, Shared, Source,
    StreamDecoder, VecDeque, INPUT_QUEUE, POOLED_FROM,
};
use crate::bits::Bits;
use crate::block::{self, Item, ScanScratch};
use crate::pooled_scan::{self as ps, Cand, Finder};
use crate::stream::Hinted;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::mpsc::sync_channel;

/// A candidate's job: its ordinal among the block candidates, its first bit, and its bytes
/// (from its first byte)
pub(super) struct ReadJob {
    ordinal: u64,
    bit: u64,
    bytes: Arc<[u8]>,
}

#[allow(clippy::large_enum_variant)] // (a job moves once, as the shipped `Job` does)
pub(super) enum Work {
    /// a block the chain read itself (the shipped scanner's job)
    Plain(Job),
    Read(ReadJob),
}

/// The workers' queue: a candidate or a block, in order
#[derive(Default)]
pub(super) struct Queue {
    q: Mutex<(VecDeque<Work>, bool)>,
    cv: Condvar,
}

impl Queue {
    fn push(&self, w: Work) {
        self.q
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .0
            .push_back(w);
        self.cv.notify_one();
    }

    fn recv(&self) -> Option<Work> {
        let mut g = self.q.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            if let Some(w) = g.0.pop_front() {
                return Some(w);
            }
            if g.1 {
                return None;
            }
            g = self.cv.wait(g).unwrap_or_else(|e| e.into_inner());
        }
    }

    fn close(&self) {
        self.q.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.cv.notify_all();
    }
}

/// A worker's read of a candidate: whether it gave a block within the bytes held, its end bit
/// and the CRC its header stores
struct Meta {
    ok: bool,
    end: u64,
    crc: u32,
}

/// The pooled scan's part of the decoder's state (under its lock)
#[derive(Default)]
pub(crate) struct PoolState {
    metas: HashMap<u64, Meta>,
    /// decoded candidates the chain has not reached yet, by ordinal, with their first bits
    done: HashMap<u64, (Result<crate::DecodedBlock, ErrorKind>, u64)>,
    /// candidates the chain took (each ordinal its place), and those it dropped
    accepted: HashSet<u64>,
    rejected: HashSet<u64>,
    /// candidates handed out and neither taken nor dropped
    spec: u64,
    /// the chain has ended: any candidate not taken is dropped
    closed: bool,
}

/// Launch the pooled scan's decoder: `threads` workers and the chain thread
pub(super) fn launch<I: crate::bwt::InverseBwt + 'static>(
    threads: usize,
    slots: usize,
    cap: Option<super::Cap>,
    k: Kernels,
) -> StreamDecoder {
    let threads = threads.max(1);
    let slots = slots.max(threads) as u64;
    let shared = Arc::new(Shared::new(slots, cap));
    let (in_tx, in_rx) = sync_channel::<Vec<u8>>(INPUT_QUEUE);
    let q = Arc::new(Queue::default());
    let mut handles = Vec::with_capacity(threads + 1);
    for i in 0..threads {
        let (shared, q) = (shared.clone(), q.clone());
        handles.push(
            std::thread::Builder::new()
                .name(format!("mibz2-{i}"))
                .spawn(move || worker::<I>(&shared, &q, k))
                .expect("spawn a bz2 worker"),
        );
    }
    let sh = shared.clone();
    handles.push(
        std::thread::Builder::new()
            .name("mibz2-scan".into())
            .spawn(move || chain(&sh, in_rx, &q))
            .expect("spawn the bz2 chain"),
    );
    StreamDecoder {
        input: Mutex::new(Some(in_tx)),
        shared,
        handles: Mutex::new(handles),
        reader: Mutex::default(),
        threads,
        pipelined: false,
        kernel: std::any::type_name::<I>(),
        hub: None,
    }
}

/// The chain's view of the candidates: those found and not handed out (in bit order), those
/// handed out and unresolved (by bit), how far the input is searched, and whether it ended
struct Cands {
    finder: Finder,
    found: VecDeque<Cand>,
    out: BTreeMap<u64, u64>,
    scanned: u64,
    seen: u64,
    eof: bool,
    ordinal: u64,
    degraded: bool,
    /// block candidates found (the test noise counts them)
    blocks_seen: u64,
}

/// The bytes a block's read is given past the next magic's first byte (the reader's refill
/// never needs more)
const MARGIN: u64 = 16;
/// The input the chain asks for past what it has searched, when it needs more
const PULL: usize = 1 << 20;
/// The most a candidate is given (the scanner's bound on an item, `stream::MAX_ITEM`)
const MAX_ITEM: u64 = 64 << 20;

impl Cands {
    /// Search more of the input from `floor` (a byte the chain has reached) on: false once it
    /// has ended and is all searched
    fn search(&mut self, src: &mut ChannelSource, floor: u64) -> Result<bool, ErrorKind> {
        if self.scanned < floor {
            self.scanned = floor;
            self.found.retain(|c| c.bit >= floor * 8);
        }
        if self.eof && self.scanned >= self.seen {
            return Ok(false);
        }
        let from = self.scanned;
        let want = (self.seen.saturating_sub(from)) as usize + PULL;
        let (data, eof) = src.view(from, want)?;
        let to = if eof {
            data.len()
        } else {
            // (a magic starting in the last 7 bytes held may run past them: found next time)
            data.len().saturating_sub(7)
        };
        let mut v = Vec::new();
        self.finder.find(data, from, 0, to, &mut v);
        v.retain(|c| c.bit >= from * 8);
        ps::noise(&mut v, &mut self.blocks_seen);
        self.found.extend(v);
        self.scanned = from + to as u64;
        self.seen = from + data.len() as u64;
        self.eof = eof;
        Ok(true)
    }
}

/// Hand out candidates in bit order while there is room in the window, searching the input
/// as needed; `floor` is the byte the chain has reached
fn pump(
    shared: &Shared,
    q: &Queue,
    c: &mut Cands,
    src: &mut ChannelSource,
    floor: u64,
) -> Result<(), ErrorKind> {
    loop {
        {
            let st = shared.lock();
            if st.cancelled || st.pool.closed {
                return Ok(());
            }
            if (st.issued - st.taken) + st.pool.spec >= shared.slots {
                return Ok(());
            }
        }
        // the first block candidate, and the magic after it (its bytes' end)
        let i = c.found.iter().position(|x| !x.end);
        let next = i.and_then(|i| c.found.get(i + 1).copied());
        let (i, cand) = match i {
            Some(i) if next.is_some() || c.eof => (i, c.found[i]),
            _ => {
                if !c.search(src, floor)? {
                    return Ok(());
                }
                continue;
            }
        };
        if cand.bit < floor * 8 {
            c.found.drain(..=i);
            continue;
        }
        let first = cand.bit / 8;
        let last = next.map_or(u64::MAX, |n| n.bit / 8 + MARGIN);
        let want = last.saturating_sub(first).min(MAX_ITEM) as usize;
        let (data, _) = src.view(first, want)?;
        let bytes: Arc<[u8]> = Arc::from(&data[..want.min(data.len())]);
        c.found.drain(..=i);
        ps::count(&ps::DISPATCHED, 1);
        ps::count(&ps::COPIED, bytes.len() as u64);
        let ordinal = c.ordinal;
        c.ordinal += 1;
        c.out.insert(cand.bit, ordinal);
        shared.lock().pool.spec += 1;
        q.push(Work::Read(ReadJob {
            ordinal,
            bit: cand.bit,
            bytes,
        }));
    }
}

/// Drop candidate `o` (its read and decode unused)
fn reject(shared: &Shared, o: u64) {
    let mut st = shared.lock();
    st.pool.rejected.insert(o);
    st.pool.done.remove(&o);
    st.pool.metas.remove(&o);
    st.pool.spec -= 1;
    ps::count(&ps::REJECTED, 1);
}

/// Leave the pooled path: every candidate handed out and unresolved dropped
fn degrade(shared: &Shared, c: &mut Cands) {
    if !c.degraded {
        c.degraded = true;
        ps::count(&ps::DEGRADED, 1);
    }
    for (_, o) in std::mem::take(&mut c.out) {
        reject(shared, o);
    }
    c.found.clear();
}

/// The hint at a block position: the worker's read of the candidate at exactly `at`, where it
/// is the chain's next block in a level-9 stream and gave a block; else None (the chain reads
/// the item itself)
fn hint(
    shared: &Shared,
    q: &Queue,
    c: &mut Cands,
    src: &mut ChannelSource,
    at: u64,
    index: u64,
    level: u32,
) -> Option<(u64, u32)> {
    if c.degraded {
        return None;
    }
    if level != 9 {
        degrade(shared, c);
        return None;
    }
    // candidates the chain has stepped over (false magics, or blocks it read itself): dropped
    let before: Vec<u64> = c.out.range(..at).map(|(_, &o)| o).collect();
    c.out.retain(|&b, _| b >= at);
    for o in before {
        reject(shared, o);
    }
    let o = loop {
        if pump(shared, q, c, src, at / 8).is_err() {
            return None;
        }
        if let Some(&o) = c.out.get(&at) {
            break o;
        }
        match c.found.iter().find(|x| x.bit >= at) {
            // a block candidate here not handed out: the window is full; wait for room
            Some(x) if x.bit == at && !x.end => {
                let mut st = shared.lock();
                while !st.cancelled && (st.issued - st.taken) + st.pool.spec >= shared.slots {
                    st.scanner_waiting = true;
                    st = shared.scan_cv.wait(st).unwrap_or_else(|e| e.into_inner());
                }
                st.scanner_waiting = false;
                if st.cancelled {
                    return None;
                }
            }
            // a stream's end here, or no magic here: the chain reads it
            Some(_) => return None,
            None if c.scanned * 8 > at + 48 || c.eof => return None,
            None => {
                if !matches!(c.search(src, at / 8), Ok(true)) {
                    return None;
                }
            }
        }
    };
    c.out.remove(&at);
    if o != index {
        // the candidates no longer match the places: the rest as the shipped scanner runs
        reject(shared, o);
        degrade(shared, c);
        return None;
    }
    let mut st = shared.lock();
    loop {
        if st.cancelled {
            return None;
        }
        if let Some(m) = st.pool.metas.remove(&o) {
            if m.ok {
                return Some((m.end, m.crc));
            }
            drop(st);
            reject(shared, o);
            return None;
        }
        st.scanner_waiting = true;
        st = shared.scan_cv.wait(st).unwrap_or_else(|e| e.into_inner());
        st.scanner_waiting = false;
    }
}

fn chain(shared: &Shared, rx: Receiver<Vec<u8>>, q: &Queue) {
    let src = ChannelSource {
        rx,
        buf: Vec::new(),
        base: 0,
        released: 0,
        eof: false,
        shared,
    };
    let mut sc = Scanner::new(src);
    let mut cands = Cands {
        finder: Finder::detect(),
        found: VecDeque::new(),
        out: BTreeMap::new(),
        scanned: 0,
        seen: 0,
        eof: false,
        ordinal: 0,
        degraded: false,
        blocks_seen: 0,
    };
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
        st.pool.closed = true;
        st.pool.done.clear();
        drop(st);
        shared.cv.notify_one();
        q.close();
    };
    loop {
        // a place first, as the shipped scanner waits for one
        let syms = {
            let mut st = shared.lock();
            while !st.cancelled && shared.full(&st) {
                st.scanner_waiting = true;
                st = shared.scan_cv.wait(st).unwrap_or_else(|e| e.into_inner());
            }
            st.scanner_waiting = false;
            if st.cancelled {
                drop(st);
                return finish(None);
            }
            st.free_syms.pop().unwrap_or_default()
        };
        let next = catch_unwind(AssertUnwindSafe(|| {
            sc.next_hinted(syms, &mut |src, at, index, level| {
                hint(shared, q, &mut cands, src, at, index, level)
            })
        }))
        .unwrap_or_else(|p| Err(Error::new(ErrorKind::Internal(panic_message(p)))));
        match next {
            Ok(Hinted::Pooled { index, .. }) => {
                let mut st = shared.lock();
                let place = st.issued;
                st.issued += 1;
                debug_assert_eq!(place, index);
                st.pool.accepted.insert(place);
                st.pool.spec -= 1;
                ps::count(&ps::ACCEPTED, 1);
                let mut wake = false;
                if let Some((res, bit)) = st.pool.done.remove(&place) {
                    wake = shared.insert_locked(
                        &mut st,
                        place,
                        res.map_err(|kind| err(place, bit, kind)),
                    );
                }
                drop(st);
                if wake {
                    shared.cv.notify_one();
                }
            }
            Ok(Hinted::Own(Event::Block {
                index,
                bit,
                symbols,
                ..
            })) => {
                let mut st = shared.lock();
                let place = st.issued;
                st.issued += 1;
                drop(st);
                debug_assert_eq!(place, index);
                ps::count(&ps::FALLBACKS, 1);
                q.push(Work::Plain((place, bit, symbols)));
            }
            Ok(Hinted::Own(Event::Done)) => return finish(None),
            Err(e) => return finish(Some(e)),
        }
    }
}

/// A block's error, as the shipped worker gives it: its place and its first bit
fn err(place: u64, bit: u64, kind: ErrorKind) -> Error {
    Error {
        kind,
        block: Some(place),
        bit: Some(bit),
    }
}

fn worker<I: crate::bwt::InverseBwt>(shared: &Shared, q: &Queue, k: Kernels) {
    let mut scratch = Scratch::<I>::default();
    let mut scan = ScanScratch::default();
    let slots = shared.slots as usize;
    while let Some(w) = q.recv() {
        if shared.stopped.load(Ordering::Acquire) {
            continue;
        }
        match w {
            Work::Plain((place, bit, mut symbols)) => {
                let res = decode::<I>(shared, &mut scratch, &mut symbols, place, k, slots);
                let res = res.map_err(|kind| Error {
                    kind,
                    block: Some(place),
                    bit: Some(bit),
                });
                let mut st = shared.lock();
                let wake = shared.insert_locked(&mut st, place, res);
                if symbols.syms.capacity() > 0 && st.free_syms.len() < slots {
                    st.free_syms.push(std::mem::take(&mut symbols.syms));
                }
                drop(st);
                if wake {
                    shared.cv.notify_one();
                }
            }
            Work::Read(job) => read_and_decode::<I>(shared, &mut scratch, &mut scan, job, k, slots),
        }
    }
}

fn read_and_decode<I: crate::bwt::InverseBwt>(
    shared: &Shared,
    scratch: &mut Scratch<I>,
    scan: &mut ScanScratch,
    job: ReadJob,
    k: Kernels,
    slots: usize,
) {
    let o = job.ordinal;
    let syms = shared.lock().free_syms.pop().unwrap_or_default();
    let read = catch_unwind(AssertUnwindSafe(|| {
        let mut br = Bits::at(&job.bytes, job.bit % 8);
        let item = block::read_item_with_scratch(&mut br, 9, syms, scan);
        let end = job.bit + (br.bit_pos() - job.bit % 8);
        (item, br.overran(), end)
    }));
    let (symbols, meta) = match read {
        Ok((Ok(Item::Block(s)), false, end)) => {
            let crc = s.crc;
            (Some(s), Meta { ok: true, end, crc })
        }
        Ok((Ok(Item::Block(s)), true, end)) => {
            give_back(shared, s.syms, slots);
            (
                None,
                Meta {
                    ok: false,
                    end,
                    crc: 0,
                },
            )
        }
        Ok((Err((_, s)), _, end)) => {
            give_back(shared, s, slots);
            (
                None,
                Meta {
                    ok: false,
                    end,
                    crc: 0,
                },
            )
        }
        _ => (
            None,
            Meta {
                ok: false,
                end: 0,
                crc: 0,
            },
        ),
    };
    {
        let mut st = shared.lock();
        if !st.pool.rejected.contains(&o) && !st.pool.closed {
            st.pool.metas.insert(o, meta);
        }
        let wake = st.scanner_waiting;
        drop(st);
        if wake {
            shared.scan_cv.notify_all();
        }
    }
    let Some(mut symbols) = symbols else {
        return;
    };
    let res = decode::<I>(shared, scratch, &mut symbols, o, k, slots);
    let mut st = shared.lock();
    if symbols.syms.capacity() > 0 && st.free_syms.len() < slots {
        st.free_syms.push(std::mem::take(&mut symbols.syms));
    }
    let mut wake = false;
    if st.pool.accepted.contains(&o) {
        wake = shared.insert_locked(&mut st, o, res.map_err(|kind| err(o, job.bit, kind)));
    } else if !st.pool.rejected.contains(&o) && !st.pool.closed {
        st.pool.done.insert(o, (res, job.bit));
    }
    drop(st);
    if wake {
        shared.cv.notify_one();
    }
}

fn give_back(shared: &Shared, syms: Vec<u16>, slots: usize) {
    if syms.capacity() > 0 {
        let mut st = shared.lock();
        if st.free_syms.len() < slots {
            st.free_syms.push(syms);
        }
    }
}

/// A block decoded as the shipped worker decodes it, a panic caught as its error
fn decode<I: crate::bwt::InverseBwt>(
    shared: &Shared,
    scratch: &mut Scratch<I>,
    symbols: &mut block::Symbols,
    place: u64,
    k: Kernels,
    slots: usize,
) -> Result<crate::DecodedBlock, ErrorKind> {
    let res = catch_unwind(AssertUnwindSafe(|| {
        #[cfg(any(
            feature = "scan-oracle",
            feature = "supplied-codes",
            feature = "ideal-construction",
            feature = "ideal-mtf",
            feature = "ideal-rle1",
        ))]
        let _ideal_block = crate::ideal::enter_block(place);
        #[cfg(not(any(
            feature = "scan-oracle",
            feature = "supplied-codes",
            feature = "ideal-construction",
            feature = "ideal-mtf",
            feature = "ideal-rle1",
        )))]
        let _ = place;
        crate::decode_owned(symbols, k, scratch, |n, syms| {
            let mut st = shared.lock();
            if syms.capacity() > 0 && st.free_syms.len() < slots {
                st.free_syms.push(syms);
            }
            let out = if n >= POOLED_FROM {
                st.free_out
                    .pop()
                    .unwrap_or_else(|| Vec::with_capacity(n + n / 16 + 64))
            } else {
                Vec::new()
            };
            drop(st);
            out
        })
    }));
    match res {
        Ok(r) => r,
        Err(p) => {
            *scratch = Scratch::default();
            Err(ErrorKind::Internal(panic_message(p)))
        }
    }
}
