//! Research only (`ideal-mtf`): zero-copy lock-free factor M of the ideal-parts oracles.
//!
//! Records each block's BWT last column L as produced by the worker's MTF/RLE2 stage:
//! - on the run-records route (`runs::front`), the run records (`Arc<[u16]>`), byte row counts
//!   and per-byte record counts (`Runs`), which construction consumes directly;
//! - on the column route (`mtf::mtf` or `mtf::mtf_checkpoints`), the unpacked column `ll`,
//!   its byte counts, and any count checkpoints.
//!
//! Repeated blocks across streams share one entry. Entering `Replay` freezes the store into a
//! lock-free `OnceLock` snapshot; on the run-records route, `Replay` hands over `Arc<[u16]>`
//! without copying the run records.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

use super::frozen::FrozenSlot;
use super::{Fnv1a, Mode};
use crate::runs::Runs;

static MODE: AtomicU8 = AtomicU8::new(Mode::Real as u8);
static EXECUTED: AtomicU64 = AtomicU64::new(0);
static REPLAYED: AtomicU64 = AtomicU64::new(0);

/// Switch the MTF/RLE2 oracle's mode for decodes started from now on.
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

enum Entry {
    Runs {
        n: usize,
        runs: Arc<[u16]>,
        counts: Box<[u32; 256]>,
        per: Box<[u32; 256]>,
    },
    Column {
        ll: Box<[u8]>,
        counts: Box<[u32; 256]>,
        checkpoints: Box<[[u32; 256]]>,
    },
}

impl Entry {
    fn same_runs(&self, n: usize, runs: &[u16], counts: &[u32; 256], per: &[u32; 256]) -> bool {
        matches!(
            self,
            Entry::Runs {
                n: sn,
                runs: sr,
                counts: sc,
                per: sp,
            } if *sn == n && **sr == *runs && **sc == *counts && **sp == *per
        )
    }

    fn same_column(&self, ll: &[u8], counts: &[u32; 256], checkpoints: &[[u32; 256]]) -> bool {
        matches!(
            self,
            Entry::Column {
                ll: sl,
                counts: sc,
                checkpoints: scp,
            } if **sl == *ll && **sc == *counts && **scp == *checkpoints
        )
    }

    fn byte_size(&self) -> u64 {
        let base = std::mem::size_of::<Entry>() as u64 + std::mem::size_of::<[u32; 256]>() as u64;
        match self {
            Entry::Runs { runs, .. } => {
                base + std::mem::size_of::<[u32; 256]>() as u64
                    + (runs.len() * std::mem::size_of::<u16>()) as u64
            }
            Entry::Column {
                ll, checkpoints, ..
            } => {
                base + ll.len() as u64
                    + (checkpoints.len() * std::mem::size_of::<[u32; 256]>()) as u64
            }
        }
    }
}

fn hash_runs(n: usize, runs: &[u16], counts: &[u32; 256], per: &[u32; 256]) -> u64 {
    let mut h = Fnv1a::new();
    h.bytes(&[0]);
    h.bytes(&(n as u64).to_le_bytes());
    h.bytes(&(runs.len() as u64).to_le_bytes());
    for &r in runs {
        h.bytes(&r.to_le_bytes());
    }
    for &c in counts {
        h.bytes(&c.to_le_bytes());
    }
    for &p in per {
        h.bytes(&p.to_le_bytes());
    }
    h.finish()
}

fn hash_column(ll: &[u8], counts: &[u32; 256], checkpoints: &[[u32; 256]]) -> u64 {
    let mut h = Fnv1a::new();
    h.bytes(&[1]);
    h.bytes(&(ll.len() as u64).to_le_bytes());
    h.bytes(ll);
    for &c in counts {
        h.bytes(&c.to_le_bytes());
    }
    h.bytes(&(checkpoints.len() as u64).to_le_bytes());
    for cp in checkpoints {
        for &c in cp {
            h.bytes(&c.to_le_bytes());
        }
    }
    h.finish()
}

#[derive(Default)]
struct Store {
    blocks: Vec<Option<Arc<Entry>>>,
    distinct: HashMap<u64, Vec<Arc<Entry>>>,
    bytes: u64,
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

/// Real MTF/RLE2 executions (`runs::front`, `mtf::mtf`, `mtf::mtf_checkpoints`) since the last
/// call (and reset).
pub fn take_executed() -> u64 {
    EXECUTED.swap(0, Ordering::SeqCst)
}

/// Blocks whose MTF/RLE2 output was replayed since the last call (and reset).
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

pub(crate) fn record_runs(index: u64, n: usize, r: &Runs) {
    let (runs, counts, per) = r.snapshot();
    let h = hash_runs(n, runs, counts, per);
    let mut g = STORE.write().unwrap_or_else(|e| e.into_inner());
    let st = g.get_or_insert_with(Store::default);
    let found = st.distinct.get(&h).and_then(|v| {
        v.iter()
            .find(|e| e.same_runs(n, runs, counts, per))
            .cloned()
    });
    let entry = found.unwrap_or_else(|| {
        let e = Arc::new(Entry::Runs {
            n,
            runs: Arc::from(runs),
            counts: Box::new(*counts),
            per: Box::new(*per),
        });
        st.bytes += e.byte_size();
        st.distinct.entry(h).or_default().push(e.clone());
        e
    });
    let i = index as usize;
    if st.blocks.len() <= i {
        st.blocks.resize(i + 1, None);
    }
    match &st.blocks[i] {
        Some(prev) if !prev.same_runs(n, runs, counts, per) => st.conflicts += 1,
        Some(_) => {}
        None => st.blocks[i] = Some(entry),
    }
    FROZEN.mark_dirty();
}

pub(crate) fn record_column(
    index: u64,
    ll: &[u8],
    counts: &[u32; 256],
    checkpoints: &[[u32; 256]],
) {
    let h = hash_column(ll, counts, checkpoints);
    let mut g = STORE.write().unwrap_or_else(|e| e.into_inner());
    let st = g.get_or_insert_with(Store::default);
    let found = st.distinct.get(&h).and_then(|v| {
        v.iter()
            .find(|e| e.same_column(ll, counts, checkpoints))
            .cloned()
    });
    let entry = found.unwrap_or_else(|| {
        let e = Arc::new(Entry::Column {
            ll: ll.into(),
            counts: Box::new(*counts),
            checkpoints: checkpoints.into(),
        });
        st.bytes += e.byte_size();
        st.distinct.entry(h).or_default().push(e.clone());
        e
    });
    let i = index as usize;
    if st.blocks.len() <= i {
        st.blocks.resize(i + 1, None);
    }
    match &st.blocks[i] {
        Some(prev) if !prev.same_column(ll, counts, checkpoints) => st.conflicts += 1,
        Some(_) => {}
        None => st.blocks[i] = Some(entry),
    }
    FROZEN.mark_dirty();
}

pub(crate) enum Replay {
    Runs(usize),
    Column(usize),
    Miss,
}

/// Zero-copy handover of block `index`'s recorded L from the lock-free frozen store (`Arc<[u16]>`
/// on the run-records route).
pub(crate) fn replay(
    index: u64,
    reads_runs: bool,
    runs: &mut Runs,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    checkpoints: &mut Vec<[u32; 256]>,
) -> Replay {
    let res = FROZEN.with(&TLS, |blocks| {
        let Some(entry) = blocks.get(index as usize).and_then(|b| b.as_ref()) else {
            return Replay::Miss;
        };
        match entry.as_ref() {
            Entry::Runs {
                n,
                runs: r,
                counts: c,
                per,
            } => {
                REPLAYED.fetch_add(1, Ordering::Relaxed);
                if reads_runs {
                    runs.load_shared(Arc::clone(r), c, per);
                    Replay::Runs(*n)
                } else {
                    ll.clear();
                    ll.reserve(*n);
                    for &rec in r.iter() {
                        let byte = (rec >> 8) as u8;
                        let len = (rec & 255) as usize;
                        ll.resize(ll.len() + len, byte);
                    }
                    *counts = **c;
                    Replay::Column(*n)
                }
            }
            Entry::Column {
                ll: col,
                counts: c,
                checkpoints: cp,
            } => {
                ll.clear();
                ll.extend_from_slice(col);
                *counts = **c;
                if !cp.is_empty() {
                    checkpoints.clear();
                    checkpoints.extend_from_slice(cp);
                }
                REPLAYED.fetch_add(1, Ordering::Relaxed);
                Replay::Column(col.len())
            }
        }
    });
    res.unwrap_or(Replay::Miss)
}
