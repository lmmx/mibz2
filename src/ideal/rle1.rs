//! Research only (`ideal-rle1`): factor R (target T6) of the ideal-parts matrix.
//!
//! Records each block's final decoded bytes (after RLE1 expansion and CRC verification) by block
//! index during a `Record` pass, sharing repeated blocks across streams. In `Replay`, emission
//! still runs in full, RLE1 expansion and the block CRC are skipped completely, and the
//! recorded final bytes are copied once (charged to the handover) into the block's output
//! buffer handed to the in-order reader.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

use super::{Fnv1a, Mode};

static MODE: AtomicU8 = AtomicU8::new(Mode::Real as u8);
static EXECUTED: AtomicU64 = AtomicU64::new(0);
static REPLAYED: AtomicU64 = AtomicU64::new(0);

/// Switch the RLE1/CRC/output oracle's mode for decodes started from now on
pub fn set_mode(m: Mode) {
    MODE.store(m as u8, Ordering::SeqCst);
}

/// The mode now set
pub fn mode() -> Mode {
    Mode::of(MODE.load(Ordering::Relaxed))
}

struct Entry {
    bytes: Box<[u8]>,
}

impl Entry {
    fn same(&self, bytes: &[u8]) -> bool {
        *self.bytes == *bytes
    }

    fn byte_size(&self) -> u64 {
        std::mem::size_of::<Entry>() as u64 + self.bytes.len() as u64
    }
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h = Fnv1a::new();
    h.bytes(&(bytes.len() as u64).to_le_bytes());
    h.bytes(bytes);
    h.finish()
}

#[derive(Default)]
struct Store {
    blocks: Vec<Option<Arc<Entry>>>,
    distinct: HashMap<u64, Vec<Arc<Entry>>>,
    bytes: u64,
    conflicts: u64,
}

static STORE: RwLock<Option<Store>> = RwLock::new(None);

/// Empty the store
pub fn clear_store() {
    *STORE.write().unwrap_or_else(|e| e.into_inner()) = Some(Store::default());
}

/// The store: (blocks indexed, distinct entries, their bytes, conflicts, blocks missing below
/// the last indexed)
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

/// Real RLE1 + CRC executions since the last call (and reset)
pub fn take_executed() -> u64 {
    EXECUTED.swap(0, Ordering::SeqCst)
}

/// Blocks whose final bytes were replayed since the last call (and reset)
pub fn take_replayed() -> u64 {
    REPLAYED.swap(0, Ordering::SeqCst)
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
        .and_then(|v| v.iter().find(|e| e.same(bytes)).cloned());
    let entry = found.unwrap_or_else(|| {
        let e = Arc::new(Entry {
            bytes: bytes.into(),
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
        Some(prev) if !prev.same(bytes) => st.conflicts += 1,
        Some(_) => {}
        None => st.blocks[i] = Some(entry),
    }
}

fn lookup(index: u64) -> Option<Arc<Entry>> {
    let g = STORE.read().unwrap_or_else(|e| e.into_inner());
    g.as_ref()?.blocks.get(index as usize)?.clone()
}

/// Hand over block `index`'s recorded final bytes, replacing `out` (the run-records route,
/// where emission already wrote inverted bytes into `out`). Lock released before copying.
pub(crate) fn replay_replace(index: u64, out: &mut Vec<u8>) -> bool {
    let Some(entry) = lookup(index) else {
        return false;
    };
    // Keep upstream emission's writes to `out` observable before overwriting it.
    std::hint::black_box(out.as_slice());
    out.clear();
    out.extend_from_slice(&entry.bytes);
    REPLAYED.fetch_add(1, Ordering::Relaxed);
    true
}

/// Hand over block `index`'s recorded final bytes, appending to `out` (the column route,
/// where emission wrote into `ll` or `rle`). Lock released before copying.
pub(crate) fn replay_append(index: u64, inverted: &[u8], out: &mut Vec<u8>) -> bool {
    let Some(entry) = lookup(index) else {
        return false;
    };
    std::hint::black_box(inverted);
    out.extend_from_slice(&entry.bytes);
    REPLAYED.fetch_add(1, Ordering::Relaxed);
    true
}
