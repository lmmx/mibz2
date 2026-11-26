//! Research only (`ideal-rle1`): zero-copy lock-free factor R (target T6) of the ideal-parts
//! matrix (`IDEAL-PARTS-2.md` §3).
//!
//! Records each block's final decoded bytes (after RLE1 expansion and CRC verification) by block
//! index during a `Record` pass, deduplicating repeated blocks across streams. Switching to
//! `Replay` freezes the store into a lock-free `OnceLock` snapshot.
//!
//! In `Replay`, RLE1 expansion and the block CRC are skipped completely, and the recorded final
//! bytes are handed to the in-order reader zero-copy as [`BlockBuf::Shared`] (`Arc<[u8]>`),
//! eliminating the ~1.68 GB cold DRAM copy on `wikidata x4`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

use super::accounting::{Phase, Scope};
use super::frozen::FrozenSlot;
use super::{Fnv1a, Mode};

/// Ready-queue block buffer under `ideal-rle1`: either an owned `Vec<u8>` (the shipped path) or
/// a zero-copy shared slice (`Arc<[u8]>`) straight from the frozen oracle store.
#[derive(Clone, Debug)]
pub(crate) enum BlockBuf {
    Owned(Vec<u8>),
    Shared(Arc<[u8]>),
}

impl PartialEq for BlockBuf {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for BlockBuf {}

impl Default for BlockBuf {
    #[inline]
    fn default() -> Self {
        Self::Owned(Vec::new())
    }
}

impl Deref for BlockBuf {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        match self {
            Self::Owned(v) => v,
            Self::Shared(a) => a,
        }
    }
}

impl From<Vec<u8>> for BlockBuf {
    #[inline]
    fn from(v: Vec<u8>) -> Self {
        Self::Owned(v)
    }
}

impl BlockBuf {
    /// Convert into an owned `Vec<u8>` (zero-copy for `Owned`; copies only if a caller asks for
    /// a whole owned `Vec<u8>` via `read_block` on a `Shared` replayed block).
    #[inline]
    pub(crate) fn into_vec(self) -> Vec<u8> {
        match self {
            Self::Owned(v) => v,
            Self::Shared(a) => a.to_vec(),
        }
    }

    /// Return the underlying `Vec<u8>` for pooling (`free_out`), or `None` for `Shared`.
    #[inline]
    pub(crate) fn into_recyclable(self) -> Option<Vec<u8>> {
        match self {
            Self::Owned(v) => Some(v),
            Self::Shared(_) => None,
        }
    }

    /// Allocated capacity for trace marks.
    #[cfg(feature = "trace")]
    #[inline]
    pub(crate) fn capacity(&self) -> usize {
        match self {
            Self::Owned(v) => v.capacity(),
            Self::Shared(a) => a.len(),
        }
    }
}

static MODE: AtomicU8 = AtomicU8::new(Mode::Real as u8);
static EXECUTED: AtomicU64 = AtomicU64::new(0);
static REPLAYED: AtomicU64 = AtomicU64::new(0);

/// Switch the RLE1/CRC/output oracle's mode for decodes started from now on.
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

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h = Fnv1a::new();
    h.bytes(&(bytes.len() as u64).to_le_bytes());
    h.bytes(bytes);
    h.finish()
}

#[derive(Default)]
struct Store {
    blocks: Vec<Option<Arc<[u8]>>>,
    distinct: HashMap<u64, Vec<Arc<[u8]>>>,
    bytes: u64,
    conflicts: u64,
}

type Snapshot = Box<[Option<Arc<[u8]>>]>;

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

/// Empty the store and invalidate any frozen snapshot.
pub fn clear_store() {
    *STORE.write().unwrap_or_else(|e| e.into_inner()) = Some(Store::default());
    FROZEN.invalidate();
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

/// Real RLE1 + CRC executions since the last call (and reset).
pub fn take_executed() -> u64 {
    EXECUTED.swap(0, Ordering::SeqCst)
}

/// Blocks whose final bytes were replayed since the last call (and reset).
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

pub(crate) fn count_executed() {
    EXECUTED.fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn record(index: u64, bytes: &[u8]) {
    let h = hash_bytes(bytes);
    let mut g = STORE.write().unwrap_or_else(|e| e.into_inner());
    let st = g.get_or_insert_with(Store::default);
    let found = st
        .distinct
        .get(&h)
        .and_then(|v| v.iter().find(|e| e.as_ref() == bytes).cloned());
    let entry = found.unwrap_or_else(|| {
        let e: Arc<[u8]> = Arc::from(bytes);
        st.bytes += std::mem::size_of::<Arc<[u8]>>() as u64 + e.len() as u64;
        st.distinct.entry(h).or_default().push(e.clone());
        e
    });
    let i = index as usize;
    if st.blocks.len() <= i {
        st.blocks.resize(i + 1, None);
    }
    match &st.blocks[i] {
        Some(prev) if prev.as_ref() != bytes => st.conflicts += 1,
        Some(_) => {}
        None => st.blocks[i] = Some(entry),
    }
    FROZEN.mark_dirty();
}

/// Lock-free check whether block `index` is present in the frozen store (used by `supplied-codes`
/// `Mode::SuppliedFull` / `E_full` to verify `R` will supply the block's final bytes before
/// skipping `E_consume`).
#[cfg(feature = "supplied-codes")]
#[inline]
pub(crate) fn has_block(index: u64) -> bool {
    FROZEN
        .with(&TLS, |blocks| {
            blocks.get(index as usize).is_some_and(Option::is_some)
        })
        .unwrap_or(false)
}

/// Zero-copy hand-over of block `index`'s recorded final bytes as [`BlockBuf::Shared`].
/// Keeps upstream emission's writes in `upstream` observable via `black_box`.
pub(crate) fn replay_shared(index: u64, upstream: &[u8]) -> Option<BlockBuf> {
    let mut scope = Scope::begin(Phase::HandR);
    let shared = FROZEN
        .with(&TLS, |blocks| {
            blocks
                .get(index as usize)
                .and_then(|b| b.as_ref().map(Arc::clone))
        })
        .flatten();
    let Some(shared) = shared else {
        scope.cancel();
        return None;
    };
    std::hint::black_box(upstream);
    scope.set_bytes(0);
    REPLAYED.fetch_add(1, Ordering::Relaxed);
    Some(BlockBuf::Shared(shared))
}

/// Appending fallback for single-threaded `decode_block` (`finish`), which appends into the
/// caller's `&mut Vec<u8>`. Parallel `decode_owned` uses [`replay_shared`] (zero copy) on both
/// run-records and column routes.
pub(crate) fn replay_append(index: u64, inverted: &[u8], out: &mut Vec<u8>) -> bool {
    let mut scope = Scope::begin(Phase::HandR);
    let hit = FROZEN
        .with(&TLS, |blocks| {
            let bytes = blocks.get(index as usize)?.as_ref()?;
            std::hint::black_box(inverted);
            out.extend_from_slice(bytes);
            Some(bytes.len() as u64)
        })
        .flatten();
    match hit {
        Some(copied) => {
            scope.set_bytes(copied);
            REPLAYED.fetch_add(1, Ordering::Relaxed);
            true
        }
        None => {
            scope.cancel();
            false
        }
    }
}
