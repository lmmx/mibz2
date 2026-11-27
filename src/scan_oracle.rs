//! Research only (`scan-oracle`): zero-copy lock-free factor S of the ideal-parts oracles.
//!
//! Three modes, switched between decodes via [`set_mode`]:
//! - [`Mode::Scan`]: the scanner runs as shipped.
//! - [`Mode::Record`]: the scanner runs as shipped and records each block's [`Symbols`] into the
//!   store by block index, deduplicating repeated blocks across streams behind `Arc<Entry>`.
//! - [`Mode::Replay`]: freezes the store into a lock-free `OnceLock` snapshot. The scanner drains
//!   and recycles input chunks as shipped, enforces the same slot and ready-byte bounds, and sends
//!   each block's lightweight header (`syms: Vec::new()`, zero copy) through the job queue;
//!   workers borrow `&[u16]` directly from the frozen store via [`with_syms`].
//!
//! Also provides optional scanner-thread CPU pinning ([`set_scanner_cpu`], [`place_scanner`],
//! [`scanner_pin_ok`]) for pinned workstation kill tests (`bench-mtf-fold`).

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

use crate::block::Symbols;
use crate::ideal::frozen::FrozenSlot;

/// What the parallel decoder's scanner does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    /// The scanner as shipped.
    Scan = 0,
    /// The scanner as shipped, each block's symbols recorded into the store.
    Record = 1,
    /// The recorded symbols delivered in place of the scanner's reading.
    Replay = 2,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Scan => "scan",
            Mode::Record => "record",
            Mode::Replay => "replay",
        }
    }

    fn of(v: u8) -> Mode {
        match v {
            1 => Mode::Record,
            2 => Mode::Replay,
            _ => Mode::Scan,
        }
    }
}

static MODE: AtomicU8 = AtomicU8::new(Mode::Scan as u8);
static SCANNED: AtomicU64 = AtomicU64::new(0);
static REPLAYED: AtomicU64 = AtomicU64::new(0);
static SCANNER_CPU: AtomicI32 = AtomicI32::new(-1);
/// 0 = unpinned / not attempted, 1 = pinned and verified, -1 = pin failed.
static SCANNER_PIN_STATUS: AtomicI32 = AtomicI32::new(0);

/// Switch the mode for decodes started from now on.
///
/// Entering [`Mode::Replay`] freezes the recorded store into the lock-free `OnceLock` snapshot.
pub fn set_mode(m: Mode) {
    if m == Mode::Replay {
        freeze_store();
    }
    MODE.store(m as u8, Ordering::SeqCst);
}

/// The mode now set.
#[inline]
pub fn mode() -> Mode {
    Mode::of(MODE.load(Ordering::Relaxed))
}

/// Pin the scanner thread to `cpu` for decodes started from now on (`None`: unpinned).
pub fn set_scanner_cpu(cpu: Option<usize>) {
    SCANNER_CPU.store(cpu.map_or(-1, |c| c as i32), Ordering::SeqCst);
    SCANNER_PIN_STATUS.store(0, Ordering::SeqCst);
}

/// Whether the most recent decode's scanner pin succeeded (`None` when unpinned).
pub fn scanner_pin_ok() -> Option<bool> {
    if SCANNER_CPU.load(Ordering::Relaxed) < 0 {
        return None;
    }
    Some(SCANNER_PIN_STATUS.load(Ordering::SeqCst) == 1)
}

/// Pin the current (scanner) thread to [`SCANNER_CPU`] if configured, and verify the resulting
/// affinity mask.
pub(crate) fn place_scanner() {
    let cpu = SCANNER_CPU.load(Ordering::Relaxed);
    if cpu < 0 {
        return;
    }
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
            fn sched_getaffinity(pid: i32, size: usize, mask: *mut u64) -> i32;
        }
        let c = cpu as usize;
        let mut mask = [0u64; 16];
        if c < mask.len() * 64 {
            mask[c / 64] |= 1u64 << (c % 64);
            // SAFETY: pid 0 targets this thread; `mask` is valid for `size_of_val(&mask)` bytes.
            let rc_set =
                unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) };
            let mut got = [0u64; 16];
            // SAFETY: pid 0 targets this thread; `got` is valid for `size_of_val(&got)` bytes.
            let rc_get =
                unsafe { sched_getaffinity(0, std::mem::size_of_val(&got), got.as_mut_ptr()) };
            let ok = rc_set == 0 && rc_get == 0 && got == mask;
            SCANNER_PIN_STATUS.store(if ok { 1 } else { -1 }, Ordering::SeqCst);
            return;
        }
    }
    SCANNER_PIN_STATUS.store(-1, Ordering::SeqCst);
}

/// A recorded block: `Symbols`' fields, the symbols shared behind `Arc<[u16]>`.
pub struct Entry {
    pub crc: u32,
    pub orig_ptr: u32,
    pub seq_to_unseq: [u8; 256],
    pub syms: Arc<[u16]>,
    pub eob: u16,
    pub max_len: usize,
}

impl Entry {
    fn same(&self, s: &Symbols) -> bool {
        self.crc == s.crc
            && self.orig_ptr == s.orig_ptr
            && self.seq_to_unseq == s.seq_to_unseq
            && self.eob == s.eob
            && self.max_len == s.max_len
            && *self.syms == s.syms[..]
    }
}

#[derive(Default)]
struct Store {
    /// By the block's index in the input.
    blocks: Vec<Option<Arc<Entry>>>,
    /// The distinct entries, by hash.
    distinct: HashMap<u64, Vec<Arc<Entry>>>,
    bytes: u64,
    /// A block index recorded twice with different symbols.
    conflicts: u64,
}

type Snapshot = Box<[Option<Arc<Entry>>]>;

static STORE: RwLock<Option<Store>> = RwLock::new(None);
static FROZEN: FrozenSlot<Snapshot> = FrozenSlot::new();

thread_local! {
    static TLS: RefCell<(u64, Option<Arc<Snapshot>>)> = const { RefCell::new((0, None)) };
}

fn freeze_store() {
    FROZEN.freeze(|| {
        let g = STORE.read().unwrap_or_else(|e| e.into_inner());
        let st = g.as_ref()?;
        (!st.blocks.is_empty()).then(|| st.blocks.clone().into_boxed_slice())
    });
}

fn hash(s: &Symbols) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |b: u8| {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    };
    for b in s.crc.to_le_bytes() {
        eat(b);
    }
    for b in s.orig_ptr.to_le_bytes() {
        eat(b);
    }
    for b in (s.syms.len() as u64).to_le_bytes() {
        eat(b);
    }
    for v in &s.syms {
        for b in v.to_le_bytes() {
            eat(b);
        }
    }
    h
}

/// Empty the store and invalidate any frozen snapshot.
pub fn clear_store() {
    *STORE.write().unwrap_or_else(|e| e.into_inner()) = Some(Store::default());
    FROZEN.invalidate();
}

/// Record block `index`'s symbols (the oracle pass).
pub(crate) fn record(index: u64, s: &Symbols) {
    let h = hash(s);
    let mut g = STORE.write().unwrap_or_else(|e| e.into_inner());
    let st = g.get_or_insert_with(Store::default);
    let found = st
        .distinct
        .get(&h)
        .and_then(|v| v.iter().find(|e| e.same(s)).cloned());
    let entry = found.unwrap_or_else(|| {
        let e = Arc::new(Entry {
            crc: s.crc,
            orig_ptr: s.orig_ptr,
            seq_to_unseq: s.seq_to_unseq,
            syms: Arc::from(s.syms.as_slice()),
            eob: s.eob,
            max_len: s.max_len,
        });
        st.bytes += 2 * e.syms.len() as u64 + std::mem::size_of::<Entry>() as u64;
        st.distinct.entry(h).or_default().push(e.clone());
        e
    });
    let i = index as usize;
    if st.blocks.len() <= i {
        st.blocks.resize(i + 1, None);
    }
    match &st.blocks[i] {
        Some(e) if !e.same(s) => st.conflicts += 1,
        Some(_) => {}
        None => st.blocks[i] = Some(entry),
    }
    FROZEN.mark_dirty();
}

/// The store: (blocks indexed, distinct entries, their bytes, conflicts, blocks missing below
/// the last indexed).
pub fn store_size() -> (u64, u64, u64, u64, u64) {
    let g = STORE.read().unwrap_or_else(|e| e.into_inner());
    g.as_ref().map_or((0, 0, 0, 0, 0), |s| {
        (
            s.blocks.len() as u64,
            s.distinct.values().map(|v| v.len() as u64).sum(),
            s.bytes,
            s.conflicts,
            s.blocks.iter().filter(|b| b.is_none()).count() as u64,
        )
    })
}

/// Real scanner block decodes since the last call (and reset).
pub fn take_scanned() -> u64 {
    SCANNED.swap(0, Ordering::SeqCst)
}

/// The blocks a replaying scanner has delivered since the last call (and reset).
pub fn take_replayed() -> u64 {
    REPLAYED.swap(0, Ordering::SeqCst)
}

/// Fallback `RwLock` acquisitions during `Replay` since the last call (0 on the `OnceLock` path).
pub fn take_lock_acquires() -> u64 {
    FROZEN.take_lock_acquires()
}

/// Whether `Replay` is currently serving from the lock-free `OnceLock` fast path.
pub fn is_frozen_lock_free() -> bool {
    FROZEN.is_once_active()
}

#[inline]
pub(crate) fn count_scanned() {
    SCANNED.fetch_add(1, Ordering::Relaxed);
}

/// Zero-copy scanner hand-over: returns block `index`'s header with an empty `Vec<u16>` (0
/// symbol bytes copied on the scanner thread) together with the recorded symbols' byte length
/// for occupancy accounting. Workers borrow `&[u16]` straight from the frozen store via
/// [`with_syms`]. Returns `None` past the last recorded block.
pub(crate) fn replay_header(index: u64) -> Option<(Symbols, usize)> {
    let out = FROZEN
        .with(&TLS, |blocks| {
            let entry = blocks.get(index as usize)?.as_ref()?;
            let syms_bytes = entry.syms.len() * std::mem::size_of::<u16>();
            Some((
                Symbols {
                    crc: entry.crc,
                    orig_ptr: entry.orig_ptr,
                    seq_to_unseq: entry.seq_to_unseq,
                    syms: Vec::new(),
                    eob: entry.eob,
                    max_len: entry.max_len,
                },
                syms_bytes,
            ))
        })
        .flatten();
    if out.is_some() {
        REPLAYED.fetch_add(1, Ordering::Relaxed);
    }
    out
}

/// Borrow block `b`'s symbols slice without copying: on the `Replay` path where the scanner
/// sent a lightweight header (`b.syms.is_empty()`), borrows `&[u16]` straight from the
/// lock-free frozen store.
#[inline]
pub(crate) fn with_syms<R>(b: &Symbols, f: impl FnOnce(&[u16]) -> R) -> R {
    if mode() == Mode::Replay && b.syms.is_empty() {
        if let Some(idx) = crate::ideal::current_block() {
            let mut f = Some(f);
            if let Some(Some(res)) = FROZEN.with(&TLS, |blocks| {
                let entry = blocks.get(idx as usize)?.as_ref()?;
                Some((f.take().unwrap())(&entry.syms))
            }) {
                return res;
            }
            return (f.take().unwrap())(&b.syms);
        }
    }
    f(&b.syms)
}
