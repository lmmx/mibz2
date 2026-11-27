//! Smooth-zone probe dissecting the `7.490 ms` residual `in_maint` measured on physical i9-10900K
//! hardware in `#1118` (`origin/evidence` `c0b2341c`, `scanner-seam-window/run-20261127T070349Z`).
//!
//! 100% safe Rust (`#![forbid(unsafe_code)]`), zero `src/` edits.
//!
//! Frozen Stop / Discovery Test (C2, frozen before timing):
//! - Exactness: every arm produces bit-identical block counts, bit offsets, CRCs, and symbol
//!   hashes against shipped `SliceSource` across `wikidata x4` (47.42 MiB, 1,724 blocks) and
//!   multi-chunk guard corpora (`text`, `random`, `records`, `runs`, `onebyte`).
//! - Discovery (admit `idea` claim):
//!   1. Accounting + component breakdown proves >= 50% (>= 3.75 ms i9-scaled) of `#1031`'s
//!      7.490 ms residual `in_maint` is avoidable copy/drain/lock/alloc work in `ChannelSource` +
//!      `Scanner::lookahead` (reproducing `base` 12 spans and `#1031` 879 spans on `wikidata x4`);
//!   2. A clean safe-Rust prototype cuts total `in_maint` by >= 70% vs shipped `base` AND cuts
//!      total end-to-end serial scan `S` (`scan + in_maint + overrun_retries`) on `wikidata x4`
//!      with zero disjoint regression on any guard corpus (`text`, `random`, `records`, `runs`,
//!      `onebyte`) and `<= 1.00x` peak window capacity vs `base`.
//! - Otherwise (STOP): record a scoped dead-end in `frontier/dead-ends.tsv`.
//!
//! Usage:
//!   cargo run --release --example in_maint_dissection -- [--repeats 7] /path/to/wikidata.bz2
#![forbid(unsafe_code)]

use bzip2::write::BzEncoder;
use bzip2::Compression;
use mibz2::bits::Bits;
use mibz2::block::{self, Item, ScanScratch};
use mibz2::stream::{Event, Scanner, SliceSource, Source};
use mibz2::{Error, ErrorKind};
use std::fs;
use std::hint::black_box;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;

const CHUNK: usize = 4 << 20;
const COMPACT: usize = 4 << 20;
const MAX_ITEM: usize = 64 << 20;
const INPUT_QUEUE: usize = 2;

#[derive(Default, Clone, Debug)]
struct Stats {
    spans: u64,
    recv_n: u64,
    seam_entries: u64,
    seam_resumes: u64,
    seam_swaps: u64,
    drains: u64,
    cancel_locks: u64,
    bytes_extend: u64,
    bytes_seam_init: u64,
    bytes_drain_move: u64,
    chunk_allocs: u64,
    chunk_drops: u64,
    retries: u64,
    wasted_bytes: u64,
    max_buf_cap: usize,
    max_seam_cap: usize,
    // Detailed component timers (nanoseconds, inside in_maint)
    ns_total_maint: u64,
    ns_seam_init: u64,
    ns_drain: u64,
    ns_extend: u64,
    ns_recycle_drop: u64,
    ns_cancel: u64,
    ns_reserve: u64,
}

impl Stats {
    fn total_bytes_moved(&self) -> u64 {
        self.bytes_extend + self.bytes_seam_init + self.bytes_drain_move
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    BaseShipped,
    Seam1031,
    Seam1031Atomic,
    Seam1031Pool4,
    SeamLazyOverran,
    SeamFixed128k,
    SeamAdaptive,
    SliceOracle,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::BaseShipped => "base_shipped",
            Arm::Seam1031 => "seam_1031",
            Arm::Seam1031Atomic => "seam_1031_atomic",
            Arm::Seam1031Pool4 => "seam_1031_pool4",
            Arm::SeamLazyOverran => "seam_lazy_overran",
            Arm::SeamFixed128k => "seam_fixed_128k",
            Arm::SeamAdaptive => "seam_adaptive",
            Arm::SliceOracle => "slice_oracle",
        }
    }

    fn all() -> &'static [Arm] {
        &[
            Arm::BaseShipped,
            Arm::Seam1031,
            Arm::Seam1031Atomic,
            Arm::Seam1031Pool4,
            Arm::SeamLazyOverran,
            Arm::SeamFixed128k,
            Arm::SeamAdaptive,
            Arm::SliceOracle,
        ]
    }
}

struct SharedMock {
    free_in: Mutex<Vec<Vec<u8>>>,
    cancelled_mutex: Mutex<bool>,
    cancelled_atomic: AtomicBool,
    free_cap: usize,
}

impl SharedMock {
    fn new(free_cap: usize) -> Self {
        Self {
            free_in: Mutex::new(Vec::new()),
            cancelled_mutex: Mutex::new(false),
            cancelled_atomic: AtomicBool::new(false),
            free_cap,
        }
    }

    fn spare(&self) -> Vec<u8> {
        self.free_in.lock().unwrap().pop().unwrap_or_default()
    }

    fn recycle(&self, chunk: Vec<u8>, stats: &mut Stats, time_parts: bool) {
        if chunk.capacity() == 0 {
            return;
        }
        let t0 = if time_parts {
            Some(Instant::now())
        } else {
            None
        };
        let dropped = {
            let mut st = self.free_in.lock().unwrap();
            if st.len() < self.free_cap {
                st.push(chunk);
                None
            } else {
                Some(chunk)
            }
        };
        if let Some(c) = dropped {
            stats.chunk_drops += 1;
            drop(c);
        }
        if let Some(t) = t0 {
            stats.ns_recycle_drop += t.elapsed().as_nanos() as u64;
        }
    }
}

/// Exact replica of shipped `src/parallel.rs` `ChannelSource` (base).
struct BaseSource {
    rx: mpsc::Receiver<Vec<u8>>,
    buf: Vec<u8>,
    base: u64,
    released: u64,
    eof: bool,
    shared: Arc<SharedMock>,
    stats: Stats,
    time_parts: bool,
}

impl Source for BaseSource {
    fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
        while !self.eof && self.base + (self.buf.len() as u64) < from + want as u64 {
            match self.rx.recv() {
                Ok(mut chunk) => {
                    let span_t0 = Instant::now();
                    self.stats.spans += 1;
                    self.stats.recv_n += 1;
                    let drop_n = (self.released - self.base) as usize;
                    if drop_n == self.buf.len() {
                        std::mem::swap(&mut self.buf, &mut chunk);
                        self.base = self.released;
                    } else {
                        if drop_n >= COMPACT {
                            let rem = self.buf.len() - drop_n;
                            self.stats.drains += 1;
                            self.stats.bytes_drain_move += rem as u64;
                            let td = if self.time_parts {
                                Some(Instant::now())
                            } else {
                                None
                            };
                            self.buf.drain(..drop_n);
                            if let Some(t) = td {
                                self.stats.ns_drain += t.elapsed().as_nanos() as u64;
                            }
                            self.base = self.released;
                        }
                        self.stats.bytes_extend += chunk.len() as u64;
                        let te = if self.time_parts {
                            Some(Instant::now())
                        } else {
                            None
                        };
                        self.buf.extend_from_slice(&chunk);
                        if let Some(t) = te {
                            self.stats.ns_extend += t.elapsed().as_nanos() as u64;
                        }
                    }
                    self.shared.recycle(chunk, &mut self.stats, self.time_parts);
                    let tc = if self.time_parts {
                        Some(Instant::now())
                    } else {
                        None
                    };
                    self.stats.cancel_locks += 1;
                    let cancelled = *self.shared.cancelled_mutex.lock().unwrap();
                    if let Some(t) = tc {
                        self.stats.ns_cancel += t.elapsed().as_nanos() as u64;
                    }
                    self.stats.max_buf_cap = self.stats.max_buf_cap.max(self.buf.capacity());
                    self.stats.ns_total_maint += span_t0.elapsed().as_nanos() as u64;
                    if cancelled {
                        return Err(ErrorKind::Cancelled);
                    }
                }
                Err(_) => {
                    self.eof = true;
                }
            }
        }
        let i = (from - self.base) as usize;
        Ok((self.buf.get(i..).unwrap_or(&[]), self.eof))
    }

    fn release(&mut self, upto: u64) {
        self.released = self.released.max(upto);
    }
}

/// Exact replica of `#1031` (`3ae44390`) `ChannelSource`, parameterized by atomic cancel &
/// one-shot seam fill so we can ablate every single mechanism cleanly.
struct SeamSource {
    rx: mpsc::Receiver<Vec<u8>>,
    buf: Vec<u8>,
    base: u64,
    released: u64,
    eof: bool,
    next: Vec<u8>,
    next_base: u64,
    seam: Vec<u8>,
    in_seam: bool,
    shared: Arc<SharedMock>,
    use_atomic_cancel: bool,
    oneshot_extra: usize, // 0 => exact #1031 incremental copy; >0 => one-shot copy `need + oneshot_extra`
    stats: Stats,
    time_parts: bool,
}

impl SeamSource {
    fn retire(&mut self, old: Vec<u8>) {
        if self.in_seam {
            self.seam = old;
            self.in_seam = false;
        } else {
            self.shared.recycle(old, &mut self.stats, self.time_parts);
        }
    }

    fn check_cancelled(&mut self) -> bool {
        let tc = if self.time_parts {
            Some(Instant::now())
        } else {
            None
        };
        let c = if self.use_atomic_cancel {
            self.shared.cancelled_atomic.load(Ordering::Relaxed)
        } else {
            self.stats.cancel_locks += 1;
            *self.shared.cancelled_mutex.lock().unwrap()
        };
        if let Some(t) = tc {
            self.stats.ns_cancel += t.elapsed().as_nanos() as u64;
        }
        c
    }
}

impl Source for SeamSource {
    fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
        let need = from + want as u64;
        while !self.eof && self.base + (self.buf.len() as u64) < need {
            let drop_n = (self.released - self.base) as usize;
            if !self.next.is_empty() {
                let span_t0 = Instant::now();
                self.stats.spans += 1;
                self.stats.seam_resumes += 1;
                if self.released >= self.next_base {
                    self.stats.seam_swaps += 1;
                    std::mem::swap(&mut self.buf, &mut self.next);
                    self.base = self.next_base;
                    let old = std::mem::take(&mut self.next);
                    self.retire(old);
                } else {
                    let at = (self.base + self.buf.len() as u64 - self.next_base) as usize;
                    let target_need = if self.oneshot_extra > 0 {
                        need.saturating_add(self.oneshot_extra as u64)
                    } else {
                        need
                    };
                    let upto = (target_need - self.next_base).min(self.next.len() as u64) as usize;
                    let add = upto - at;
                    if !self.in_seam {
                        self.stats.seam_entries += 1;
                        let mut seam = std::mem::take(&mut self.seam);
                        seam.clear();
                        let want_cap = if self.oneshot_extra > 0 {
                            self.buf.len() - drop_n + add
                        } else {
                            (self.buf.len() - drop_n + add).max(2 * want)
                        };
                        let tr = if self.time_parts {
                            Some(Instant::now())
                        } else {
                            None
                        };
                        seam.reserve(want_cap);
                        if let Some(t) = tr {
                            self.stats.ns_reserve += t.elapsed().as_nanos() as u64;
                        }
                        let tail_len = self.buf.len() - drop_n;
                        self.stats.bytes_seam_init += tail_len as u64;
                        let ts = if self.time_parts {
                            Some(Instant::now())
                        } else {
                            None
                        };
                        seam.extend_from_slice(&self.buf[drop_n..]);
                        if let Some(t) = ts {
                            self.stats.ns_seam_init += t.elapsed().as_nanos() as u64;
                        }
                        let chunk = std::mem::replace(&mut self.buf, seam);
                        self.base = self.released;
                        self.in_seam = true;
                        self.shared.recycle(chunk, &mut self.stats, self.time_parts);
                    } else {
                        let (len, cap) = (self.buf.len(), self.buf.capacity());
                        if drop_n >= COMPACT || (len + add > cap && len - drop_n + add <= cap) {
                            let rem = len - drop_n;
                            self.stats.drains += 1;
                            self.stats.bytes_drain_move += rem as u64;
                            let td = if self.time_parts {
                                Some(Instant::now())
                            } else {
                                None
                            };
                            self.buf.drain(..drop_n);
                            if let Some(t) = td {
                                self.stats.ns_drain += t.elapsed().as_nanos() as u64;
                            }
                            self.base = self.released;
                        }
                    }
                    self.stats.bytes_extend += add as u64;
                    let te = if self.time_parts {
                        Some(Instant::now())
                    } else {
                        None
                    };
                    self.buf.extend_from_slice(&self.next[at..upto]);
                    if let Some(t) = te {
                        self.stats.ns_extend += t.elapsed().as_nanos() as u64;
                    }
                    if upto == self.next.len() {
                        let next_buf = std::mem::take(&mut self.next);
                        self.shared
                            .recycle(next_buf, &mut self.stats, self.time_parts);
                    }
                }
                self.stats.max_buf_cap = self.stats.max_buf_cap.max(self.buf.capacity());
                self.stats.max_seam_cap = self.stats.max_seam_cap.max(if self.in_seam {
                    self.buf.capacity()
                } else {
                    self.seam.capacity()
                });
                let cancelled = if !self.use_atomic_cancel {
                    self.check_cancelled()
                } else {
                    false
                };
                self.stats.ns_total_maint += span_t0.elapsed().as_nanos() as u64;
                if cancelled {
                    return Err(ErrorKind::Cancelled);
                }
            } else {
                match self.rx.recv() {
                    Ok(mut chunk) => {
                        let span_t0 = Instant::now();
                        self.stats.spans += 1;
                        self.stats.recv_n += 1;
                        if drop_n == self.buf.len() {
                            std::mem::swap(&mut self.buf, &mut chunk);
                            self.base = self.released;
                            self.retire(chunk);
                        } else if chunk.is_empty() {
                            self.shared.recycle(chunk, &mut self.stats, self.time_parts);
                        } else {
                            self.next_base = self.base + self.buf.len() as u64;
                            self.next = chunk;
                        }
                        let cancelled = self.check_cancelled();
                        self.stats.max_buf_cap = self.stats.max_buf_cap.max(self.buf.capacity());
                        self.stats.ns_total_maint += span_t0.elapsed().as_nanos() as u64;
                        if cancelled {
                            return Err(ErrorKind::Cancelled);
                        }
                    }
                    Err(_) => {
                        self.eof = true;
                    }
                }
            }
        }
        let i = (from - self.base) as usize;
        Ok((self.buf.get(i..).unwrap_or(&[]), self.eof))
    }

    fn release(&mut self, upto: u64) {
        self.released = self.released.max(upto);
    }
}

#[derive(Clone, Copy)]
enum WantPolicy {
    /// Ask `want = 1` first; only copy into seam on actual `br.overran()` with `retry_step = 128 KiB`.
    LazyOverran,
    /// Fixed per-block `want = 128 KiB` (reset every block, not sticky).
    Fixed128k,
    /// Adaptive sliding-window max over last 16 blocks: `(max16 * 5 / 4).clamp(64 KiB, 1 MiB)`.
    Adaptive,
}

enum ScanState {
    Header { first: bool },
    Stream { level: u32, combined: u32 },
    Done,
}

/// Mirror of `src/stream.rs::Scanner` calling the exact shipped `block::read_item_with_scratch`,
/// differing only in per-block `want` policy (no sticky `2 MiB` lookahead across blocks).
struct PolicyScanner {
    src: SeamSource,
    policy: WantPolicy,
    bit: u64,
    state: ScanState,
    blocks: u64,
    streams: u64,
    scratch: ScanScratch,
    recent_lens: [usize; 16],
    recent_idx: usize,
}

impl PolicyScanner {
    fn new(src: SeamSource, policy: WantPolicy) -> Self {
        Self {
            src,
            policy,
            bit: 0,
            state: ScanState::Header { first: true },
            blocks: 0,
            streams: 0,
            scratch: ScanScratch::default(),
            recent_lens: [0; 16],
            recent_idx: 0,
        }
    }

    fn initial_want(&self) -> usize {
        match self.policy {
            WantPolicy::LazyOverran => 1,
            WantPolicy::Fixed128k => 128 << 10,
            WantPolicy::Adaptive => {
                let m = self.recent_lens.iter().copied().max().unwrap_or(0);
                if m == 0 {
                    128 << 10
                } else {
                    (m + (m >> 2) + 4096).clamp(64 << 10, 1 << 20)
                }
            }
        }
    }

    fn next(&mut self, mut syms: Vec<u16>) -> Result<Event, Error> {
        loop {
            match self.state {
                ScanState::Done => return Ok(Event::Done),
                ScanState::Header { first } => {
                    let at = self.bit;
                    let err = |kind| Error {
                        kind,
                        block: None,
                        bit: Some(at),
                    };
                    let (data, eof) = self.src.view(at / 8, 4).map_err(err)?;
                    if data.is_empty() && eof {
                        if first {
                            return Err(err(ErrorKind::Empty));
                        }
                        self.state = ScanState::Done;
                        continue;
                    }
                    match data {
                        [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                            self.state = ScanState::Stream {
                                level: (l - b'0') as u32,
                                combined: 0,
                            };
                            self.bit += 32;
                        }
                        _ => {
                            return Err(err(if first {
                                ErrorKind::NotBz2
                            } else {
                                ErrorKind::TrailingData
                            }))
                        }
                    }
                }
                ScanState::Stream { level, combined } => {
                    let at = self.bit;
                    let index = self.blocks;
                    let err = |kind| Error {
                        kind,
                        block: Some(index),
                        bit: Some(at),
                    };
                    let base = at / 8;
                    let mut want = self.initial_want();
                    let mut retry_step = 128usize << 10;
                    loop {
                        let (data, eof) = self.src.view(base, want).map_err(err)?;
                        let data_len = data.len();
                        let mut br = Bits::at(data, at % 8);
                        let item =
                            block::read_item_with_scratch(&mut br, level, syms, &mut self.scratch);
                        let used = br.bit_pos() - at % 8;
                        if br.overran() {
                            if eof {
                                return Err(err(ErrorKind::Truncated));
                            }
                            if want >= MAX_ITEM {
                                return Err(err(ErrorKind::BlockTooLarge));
                            }
                            self.src.stats.retries += 1;
                            self.src.stats.wasted_bytes += data_len as u64;
                            want = (data_len + retry_step).max(want * 2).min(MAX_ITEM);
                            retry_step = (retry_step * 2).min(MAX_ITEM);
                            syms = match item {
                                Ok(Item::Block(b)) => b.syms,
                                Ok(Item::End(_)) => Vec::new(),
                                Err((_, s)) => s,
                            };
                            continue;
                        }
                        self.bit += used;
                        self.src.release(self.bit / 8);
                        let used_bytes = used.div_ceil(8) as usize;
                        self.recent_lens[self.recent_idx] = used_bytes;
                        self.recent_idx = (self.recent_idx + 1) & 15;
                        match item.map_err(|(kind, _)| err(kind))? {
                            Item::End(stored) => {
                                if stored != combined {
                                    return Err(err(ErrorKind::StreamCrc {
                                        stored,
                                        computed: combined,
                                    }));
                                }
                                self.bit = self.bit.div_ceil(8) * 8;
                                self.state = ScanState::Header { first: false };
                                self.streams += 1;
                                syms = Vec::new();
                                break;
                            }
                            Item::Block(symbols) => {
                                self.state = ScanState::Stream {
                                    level,
                                    combined: combined.rotate_left(1) ^ symbols.crc,
                                };
                                self.blocks += 1;
                                let (end, stream) = (self.bit, self.streams);
                                return Ok(Event::Block {
                                    index,
                                    bit: at,
                                    end,
                                    stream,
                                    level,
                                    symbols,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

fn thread_cpu_ns() -> u64 {
    let s = fs::read_to_string("/proc/thread-self/schedstat").unwrap_or_default();
    s.split_whitespace()
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
}

#[derive(Clone, Debug)]
struct RunResult {
    blocks: u64,
    digest: u64,
    wall_ns: u64,
    cpu_ns: u64,
    stats: Stats,
}

fn mix(mut h: u64, v: u64) -> u64 {
    h ^= v;
    h = h.wrapping_mul(0x1000_0000_01b3);
    h
}

fn spawn_feeder(
    data: Arc<Vec<u8>>,
    shared: Arc<SharedMock>,
    tx: mpsc::SyncSender<Vec<u8>>,
) -> thread::JoinHandle<u64> {
    thread::spawn(move || {
        let mut pos = 0usize;
        let mut allocs = 0u64;
        while pos < data.len() {
            let mut buf = shared.spare();
            if buf.capacity() < CHUNK {
                buf = vec![0u8; CHUNK];
                allocs += 1;
            }
            let n = (data.len() - pos).min(CHUNK);
            buf.resize(n, 0);
            buf.copy_from_slice(&data[pos..pos + n]);
            pos += n;
            if tx.send(buf).is_err() {
                break;
            }
        }
        allocs
    })
}

/// Wrapper sharing `Stats` out of `mibz2::stream::Scanner<S>` without touching `src/`.
struct TrackedSource<S: HasStats> {
    inner: S,
    out: Arc<Mutex<Stats>>,
}

trait HasStats: Source {
    fn take_stats(&mut self) -> Stats;
}

impl HasStats for BaseSource {
    fn take_stats(&mut self) -> Stats {
        std::mem::take(&mut self.stats)
    }
}

impl HasStats for SeamSource {
    fn take_stats(&mut self) -> Stats {
        std::mem::take(&mut self.stats)
    }
}

impl<S: HasStats> Source for TrackedSource<S> {
    fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
        self.inner.view(from, want)
    }

    fn release(&mut self, upto: u64) {
        self.inner.release(upto);
    }
}

impl<S: HasStats> Drop for TrackedSource<S> {
    fn drop(&mut self) {
        *self.out.lock().unwrap() = self.inner.take_stats();
    }
}

fn run_arm_real(arm: Arm, data: Arc<Vec<u8>>, time_parts: bool, contend_mutex: bool) -> RunResult {
    let free_cap = match arm {
        Arm::BaseShipped | Arm::Seam1031 | Arm::Seam1031Atomic => 1,
        _ => INPUT_QUEUE + 2,
    };
    let shared = Arc::new(SharedMock::new(free_cap));

    let stop_contend = Arc::new(AtomicBool::new(false));
    let contender = if contend_mutex {
        let s = shared.clone();
        let stop = stop_contend.clone();
        Some(thread::spawn(move || {
            let mut spins = 0u64;
            while !stop.load(Ordering::Relaxed) {
                {
                    let g = s.cancelled_mutex.lock().unwrap();
                    black_box(*g);
                }
                spins += 1;
                for _ in 0..40 {
                    std::hint::spin_loop();
                }
            }
            spins
        }))
    } else {
        None
    };

    let stats_slot = Arc::new(Mutex::new(Stats::default()));
    let mut blocks = 0u64;
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut syms = Vec::with_capacity(900_000);
    let wall_ns: u64;
    let cpu_ns: u64;
    let feeder_allocs: u64;

    macro_rules! drain_scanner {
        ($sc:expr) => {{
            loop {
                match $sc.next(syms).unwrap() {
                    Event::Block {
                        index,
                        bit,
                        end,
                        symbols,
                        ..
                    } => {
                        blocks += 1;
                        h = mix(h, index);
                        h = mix(h, bit);
                        h = mix(h, end);
                        h = mix(h, symbols.crc as u64);
                        h = mix(h, symbols.syms.len() as u64);
                        if let Some(&first) = symbols.syms.first() {
                            h = mix(h, first as u64);
                        }
                        if let Some(&last) = symbols.syms.last() {
                            h = mix(h, last as u64);
                        }
                        syms = symbols.syms;
                    }
                    Event::Done => break,
                }
            }
        }};
    }

    match arm {
        Arm::SliceOracle => {
            let cpu0 = thread_cpu_ns();
            let t0 = Instant::now();
            let mut sc = Scanner::new(SliceSource(&data));
            drain_scanner!(sc);
            wall_ns = t0.elapsed().as_nanos() as u64;
            cpu_ns = thread_cpu_ns().saturating_sub(cpu0);
            feeder_allocs = 0;
        }
        Arm::BaseShipped => {
            let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(INPUT_QUEUE);
            let feeder = spawn_feeder(data, shared.clone(), tx);
            let cpu0 = thread_cpu_ns();
            let t0 = Instant::now();
            let src = TrackedSource {
                inner: BaseSource {
                    rx,
                    buf: Vec::new(),
                    base: 0,
                    released: 0,
                    eof: false,
                    shared,
                    stats: Stats::default(),
                    time_parts,
                },
                out: stats_slot.clone(),
            };
            {
                let mut sc = Scanner::new(src);
                drain_scanner!(sc);
            }
            wall_ns = t0.elapsed().as_nanos() as u64;
            cpu_ns = thread_cpu_ns().saturating_sub(cpu0);
            feeder_allocs = feeder.join().unwrap();
        }
        Arm::Seam1031 | Arm::Seam1031Atomic | Arm::Seam1031Pool4 => {
            let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(INPUT_QUEUE);
            let feeder = spawn_feeder(data, shared.clone(), tx);
            let cpu0 = thread_cpu_ns();
            let t0 = Instant::now();
            let src = TrackedSource {
                inner: SeamSource {
                    rx,
                    buf: Vec::new(),
                    base: 0,
                    released: 0,
                    eof: false,
                    next: Vec::new(),
                    next_base: 0,
                    seam: Vec::new(),
                    in_seam: false,
                    shared,
                    use_atomic_cancel: arm != Arm::Seam1031,
                    oneshot_extra: 0,
                    stats: Stats::default(),
                    time_parts,
                },
                out: stats_slot.clone(),
            };
            {
                let mut sc = Scanner::new(src);
                drain_scanner!(sc);
            }
            wall_ns = t0.elapsed().as_nanos() as u64;
            cpu_ns = thread_cpu_ns().saturating_sub(cpu0);
            feeder_allocs = feeder.join().unwrap();
        }
        Arm::SeamLazyOverran | Arm::SeamFixed128k | Arm::SeamAdaptive => {
            let (policy, oneshot_extra) = match arm {
                Arm::SeamLazyOverran => (WantPolicy::LazyOverran, 0),
                Arm::SeamFixed128k => (WantPolicy::Fixed128k, 128 << 10),
                Arm::SeamAdaptive => (WantPolicy::Adaptive, 128 << 10),
                _ => unreachable!(),
            };
            let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(INPUT_QUEUE);
            let feeder = spawn_feeder(data, shared.clone(), tx);
            let cpu0 = thread_cpu_ns();
            let t0 = Instant::now();
            let src = SeamSource {
                rx,
                buf: Vec::new(),
                base: 0,
                released: 0,
                eof: false,
                next: Vec::new(),
                next_base: 0,
                seam: Vec::new(),
                in_seam: false,
                shared,
                use_atomic_cancel: true,
                oneshot_extra,
                stats: Stats::default(),
                time_parts,
            };
            let mut sc = PolicyScanner::new(src, policy);
            drain_scanner!(sc);
            wall_ns = t0.elapsed().as_nanos() as u64;
            cpu_ns = thread_cpu_ns().saturating_sub(cpu0);
            *stats_slot.lock().unwrap() = sc.src.take_stats();
            feeder_allocs = feeder.join().unwrap();
        }
    }

    if let Some(c) = contender {
        stop_contend.store(true, Ordering::Relaxed);
        let _ = c.join();
    }

    let mut stats = stats_slot.lock().unwrap().clone();
    stats.chunk_allocs = feeder_allocs;
    RunResult {
        blocks,
        digest: h,
        wall_ns,
        cpu_ns,
        stats,
    }
}

fn bz2_compress(raw: &[u8]) -> Vec<u8> {
    let mut enc = BzEncoder::new(Vec::new(), Compression::best());
    enc.write_all(raw).unwrap();
    enc.finish().unwrap()
}

fn repeat_stream(bz: &[u8], min_bytes: usize) -> Vec<u8> {
    let reps = min_bytes.div_ceil(bz.len()).max(1);
    let mut out = Vec::with_capacity(bz.len() * reps);
    for _ in 0..reps {
        out.extend_from_slice(bz);
    }
    out
}

fn build_guard_corpora() -> Vec<(&'static str, Vec<u8>)> {
    // Small deterministic generators matching tests/common/mod.rs shapes, repeated to >= 16 MiB
    // compressed so every guard corpus crosses multiple 4 MiB chunks (except tiny onebyte).
    let mut rng = 0xfeed_beef_dead_cafeu64;
    let mut next_u64 = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };

    // text: repo sources repeated to ~12 MiB raw -> ~1.8 MiB bz2 -> repeated to ~16 MiB bz2
    let mut text_raw = Vec::new();
    for dir in ["src", "examples"] {
        if let Ok(rd) = fs::read_dir(dir) {
            let mut files: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            files.sort();
            for f in files.into_iter().filter(|f| f.is_file()) {
                if let Ok(bytes) = fs::read(f) {
                    text_raw.extend_from_slice(&bytes);
                }
            }
        }
    }
    if text_raw.len() < 4_000_000 {
        let base = text_raw.clone();
        while text_raw.len() < 4_000_000 {
            text_raw.extend_from_slice(&base);
        }
    }
    let text_bz = repeat_stream(&bz2_compress(&text_raw[..4_000_000]), 16 << 20);

    // random: 4.5 MiB incompressible -> ~4.5 MiB bz2 -> repeated to ~18 MiB bz2 (full ~900 KiB blocks!)
    let mut rand_raw = vec![0u8; 4_500_000];
    for chunk in rand_raw.chunks_mut(8) {
        let v = next_u64().to_le_bytes();
        chunk.copy_from_slice(&v[..chunk.len()]);
    }
    let rand_bz = repeat_stream(&bz2_compress(&rand_raw), 18 << 20);

    // records: 4.5 MiB binary records -> ~4.5 MiB bz2 -> repeated to ~18 MiB bz2
    let rec_raw: Vec<u8> = (0..1_125_000u32)
        .flat_map(|i| (i.wrapping_mul(2_654_435_761) >> 12 | i << 20).to_le_bytes())
        .collect();
    let rec_bz = repeat_stream(&bz2_compress(&rec_raw), 18 << 20);

    // runs: 10 MiB random runs -> ~38 KiB bz2 -> repeated to ~8 MiB bz2 (2+ chunks)
    let mut runs_raw = Vec::with_capacity(2_500_000);
    while runs_raw.len() < 2_500_000 {
        let n = 1 + (next_u64() % 600) as usize;
        let b = b"xyz"[(next_u64() % 3) as usize];
        runs_raw.extend(std::iter::repeat(b).take(n));
    }
    let runs_bz = repeat_stream(&bz2_compress(&runs_raw), 8 << 20);

    // onebyte: tiny single-chunk stream (verifies small-input fast path)
    let one_bz = bz2_compress(&vec![b'a'; 2_000_000]);

    vec![
        ("text_16m", text_bz),
        ("random_18m", rand_bz),
        ("records_18m", rec_bz),
        ("runs_8m", runs_bz),
        ("onebyte", one_bz),
    ]
}

fn median(xs: &[f64]) -> f64 {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

fn minmax(xs: &[f64]) -> (f64, f64) {
    let lo = xs.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (lo, hi)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repeats = 7usize;
    let mut wiki_path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--repeats" && i + 1 < args.len() {
            repeats = args[i + 1].parse().expect("valid --repeats");
            i += 2;
        } else {
            wiki_path = Some(args[i].clone());
            i += 1;
        }
    }
    let wiki_path = wiki_path.unwrap_or_else(|| "corpora/wikidata.bz2".to_string());
    let wiki_1x = fs::read(Path::new(&wiki_path))
        .unwrap_or_else(|e| panic!("failed to read {wiki_path}: {e}"));
    // Exact wikidata-x4 stream (4x concatenation, matching i9 bundle c0b2341c: 47,419,732 B, 1,724 blks)
    let mut wiki_4x = Vec::with_capacity(wiki_1x.len() * 4);
    for _ in 0..4 {
        wiki_4x.extend_from_slice(&wiki_1x);
    }
    let wiki_4x = Arc::new(wiki_4x);
    println!(
        "=== in_maint Residual Dissection Probe (repeats={repeats}) ===\nInput: wikidata-x4 = {} bytes ({:.2} MiB, {} full 4 MiB chunks + {} B tail)",
        wiki_4x.len(),
        wiki_4x.len() as f64 / (1024.0 * 1024.0),
        wiki_4x.len() / CHUNK,
        wiki_4x.len() % CHUNK
    );

    // -------------------------------------------------------------------------
    // PART 1: Exact Deterministic Accounting + Parity Check on wikidata-x4
    // -------------------------------------------------------------------------
    println!(
        "\n--- PART 1: Exact Deterministic Accounting on wikidata-x4 (45.22 MiB, 1,724 blocks) ---"
    );
    let oracle = run_arm_real(Arm::SliceOracle, wiki_4x.clone(), false, false);
    println!(
        "Oracle: blocks={} digest=0x{:016x}",
        oracle.blocks, oracle.digest
    );
    println!(
        "{:<18} {:>6} {:>5} {:>6} {:>6} {:>6} {:>6} {:>8} {:>8} {:>8} {:>8} {:>5} {:>5} {:>6} {:>7} {:>7}",
        "arm",
        "spans",
        "recv",
        "s_ent",
        "s_res",
        "drain",
        "locks",
        "ext_MB",
        "sini_MB",
        "drn_MB",
        "tot_MB",
        "aloc",
        "drop",
        "retry",
        "buf_MB",
        "seam_MB"
    );
    for &arm in Arm::all() {
        let r = run_arm_real(arm, wiki_4x.clone(), true, false);
        assert_eq!(
            r.blocks,
            oracle.blocks,
            "{} block count mismatch",
            arm.name()
        );
        assert_eq!(r.digest, oracle.digest, "{} digest mismatch", arm.name());
        if arm == Arm::BaseShipped {
            assert_eq!(
                r.stats.spans, 12,
                "expected exact i9 trace-base.jsonl spans == 12"
            );
        }
        if arm == Arm::Seam1031 {
            assert_eq!(
                r.stats.spans, 879,
                "expected exact i9 trace-head.jsonl spans == 879"
            );
        }
        let s = &r.stats;
        let mb = 1024.0 * 1024.0;
        println!(
            "{:<18} {:>6} {:>5} {:>6} {:>6} {:>6} {:>6} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>5} {:>5} {:>6} {:>7.2} {:>7.2}",
            arm.name(),
            s.spans,
            s.recv_n,
            s.seam_entries,
            s.seam_resumes,
            s.drains,
            s.cancel_locks,
            s.bytes_extend as f64 / mb,
            s.bytes_seam_init as f64 / mb,
            s.bytes_drain_move as f64 / mb,
            s.total_bytes_moved() as f64 / mb,
            s.chunk_allocs,
            s.chunk_drops,
            s.retries,
            s.max_buf_cap as f64 / mb,
            s.max_seam_cap as f64 / mb
        );
    }

    // -------------------------------------------------------------------------
    // PART 2: Component-Level in_maint Microsecond Breakdown (median of 5)
    // -------------------------------------------------------------------------
    for contend in [false, true] {
        println!(
            "\n--- PART 2: Component-Level in_maint Breakdown on wikidata-x4 (contended_mutex={contend}, median of 5) ---"
        );
        println!(
            "{:<18} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}",
            "arm", "tot_ms", "s_init", "extend", "drain", "reserve", "recy_drp", "cancel", "other"
        );
        for &arm in Arm::all() {
            if arm == Arm::SliceOracle {
                continue;
            }
            let mut tot = Vec::new();
            let mut sini = Vec::new();
            let mut ext = Vec::new();
            let mut drn = Vec::new();
            let mut res = Vec::new();
            let mut rec = Vec::new();
            let mut can = Vec::new();
            let mut oth = Vec::new();
            for _ in 0..5 {
                let r = run_arm_real(arm, wiki_4x.clone(), true, contend);
                let s = &r.stats;
                let parts = s.ns_seam_init
                    + s.ns_extend
                    + s.ns_drain
                    + s.ns_reserve
                    + s.ns_recycle_drop
                    + s.ns_cancel;
                tot.push(s.ns_total_maint as f64 / 1e6);
                sini.push(s.ns_seam_init as f64 / 1e6);
                ext.push(s.ns_extend as f64 / 1e6);
                drn.push(s.ns_drain as f64 / 1e6);
                res.push(s.ns_reserve as f64 / 1e6);
                rec.push(s.ns_recycle_drop as f64 / 1e6);
                can.push(s.ns_cancel as f64 / 1e6);
                oth.push(s.ns_total_maint.saturating_sub(parts) as f64 / 1e6);
            }
            println!(
                "{:<18} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3}",
                arm.name(),
                median(&tot),
                median(&sini),
                median(&ext),
                median(&drn),
                median(&res),
                median(&rec),
                median(&can),
                median(&oth)
            );
        }
    }

    // -------------------------------------------------------------------------
    // PART 3: Interleaved End-to-End S + in_maint Timing on wikidata-x4
    // -------------------------------------------------------------------------
    let arms = Arm::all();
    let mut cpu_ms: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    let mut wall_ms: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    let mut maint_ms: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];

    // Warmup
    for &arm in arms {
        let _ = run_arm_real(arm, wiki_4x.clone(), false, false);
    }

    for rep in 0..repeats {
        for k in 0..arms.len() {
            let idx = (k + rep) % arms.len();
            let r = run_arm_real(arms[idx], wiki_4x.clone(), false, false);
            assert_eq!(r.blocks, oracle.blocks);
            assert_eq!(r.digest, oracle.digest);
            cpu_ms[idx].push(r.cpu_ns as f64 / 1e6);
            wall_ms[idx].push(r.wall_ns as f64 / 1e6);
            maint_ms[idx].push(r.stats.ns_total_maint as f64 / 1e6);
        }
    }

    let base_cpu_med = median(&cpu_ms[0]);
    let base_wall_med = median(&wall_ms[0]);
    let base_maint_med = median(&maint_ms[0]);

    println!(
        "\n--- PART 3: Interleaved End-to-End wikidata-x4 (n={repeats}, time_parts=false) ---"
    );
    println!(
        "{:<18} {:>10} {:>19} {:>8} {:>10} {:>19} {:>8} {:>9} {:>8}",
        "arm",
        "cpu_med",
        "[cpu_min..max]",
        "vs_base",
        "wall_med",
        "[wall_min..max]",
        "vs_base",
        "maint_ms",
        "m_cut"
    );
    for (idx, &arm) in arms.iter().enumerate() {
        let cm = median(&cpu_ms[idx]);
        let (clo, chi) = minmax(&cpu_ms[idx]);
        let wm = median(&wall_ms[idx]);
        let (wlo, whi) = minmax(&wall_ms[idx]);
        let mm = median(&maint_ms[idx]);
        let cdiff = (cm / base_cpu_med - 1.0) * 100.0;
        let wdiff = (wm / base_wall_med - 1.0) * 100.0;
        let mcut = (mm / base_maint_med - 1.0) * 100.0;
        println!(
            "{:<18} {:>10.2} [{:>7.2}..{:>7.2}] {:>+7.2}% {:>10.2} [{:>7.2}..{:>7.2}] {:>+7.2}% {:>9.3} {:>+7.1}%",
            arm.name(),
            cm,
            clo,
            chi,
            cdiff,
            wm,
            wlo,
            whi,
            wdiff,
            mm,
            mcut
        );
    }

    // -------------------------------------------------------------------------
    // PART 4: Off-Wikidata Guard Corpora (exactness + accounting + interleaved S)
    // -------------------------------------------------------------------------
    println!(
        "\n--- PART 4: Off-Wikidata Multi-Chunk Guard Corpora (exactness + accounting + n=5 S) ---"
    );
    let guard_arms = [
        Arm::BaseShipped,
        Arm::Seam1031,
        Arm::SeamLazyOverran,
        Arm::SeamFixed128k,
        Arm::SeamAdaptive,
    ];
    for (cname, cdata) in build_guard_corpora() {
        let cdata = Arc::new(cdata);
        let cor = run_arm_real(Arm::SliceOracle, cdata.clone(), false, false);
        println!(
            "\nCorpus: {cname} ({:.2} MiB bz2, {} blocks, digest=0x{:016x})",
            cdata.len() as f64 / (1024.0 * 1024.0),
            cor.blocks,
            cor.digest
        );
        let mut g_cpu: Vec<Vec<f64>> = vec![Vec::new(); guard_arms.len()];
        let mut g_maint: Vec<Vec<f64>> = vec![Vec::new(); guard_arms.len()];
        let mut g_stats: Vec<Stats> = Vec::new();
        for &arm in &guard_arms {
            let r = run_arm_real(arm, cdata.clone(), false, false);
            assert_eq!(
                r.blocks,
                cor.blocks,
                "{cname}/{} block mismatch",
                arm.name()
            );
            assert_eq!(
                r.digest,
                cor.digest,
                "{cname}/{} digest mismatch",
                arm.name()
            );
            g_stats.push(r.stats);
        }
        for rep in 0..5 {
            for k in 0..guard_arms.len() {
                let idx = (k + rep) % guard_arms.len();
                let r = run_arm_real(guard_arms[idx], cdata.clone(), false, false);
                g_cpu[idx].push(r.cpu_ns as f64 / 1e6);
                g_maint[idx].push(r.stats.ns_total_maint as f64 / 1e6);
            }
        }
        let bcpu = median(&g_cpu[0]);
        let bmaint = median(&g_maint[0]).max(1e-6);
        println!(
            "  {:<18} {:>6} {:>6} {:>8} {:>8} {:>10} {:>19} {:>8} {:>9} {:>8}",
            "arm",
            "spans",
            "retry",
            "tot_MB",
            "wast_KB",
            "cpu_med",
            "[cpu_min..max]",
            "vs_base",
            "maint_ms",
            "m_cut"
        );
        for (idx, &arm) in guard_arms.iter().enumerate() {
            let cm = median(&g_cpu[idx]);
            let (clo, chi) = minmax(&g_cpu[idx]);
            let mm = median(&g_maint[idx]);
            let s = &g_stats[idx];
            println!(
                "  {:<18} {:>6} {:>6} {:>8.2} {:>8.1} {:>10.2} [{:>7.2}..{:>7.2}] {:>+7.2}% {:>9.3} {:>+7.1}%",
                arm.name(),
                s.spans,
                s.retries,
                s.total_bytes_moved() as f64 / (1024.0 * 1024.0),
                s.wasted_bytes as f64 / 1024.0,
                cm,
                clo,
                chi,
                (cm / bcpu - 1.0) * 100.0,
                mm,
                (mm / bmaint - 1.0) * 100.0
            );
        }
    }
}
