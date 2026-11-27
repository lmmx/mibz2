//! Research only (`ideal-construction`): zero-copy lock-free factor C of the ideal-parts oracles.
//!
//! Records Open's built structures (`Kernel::construct_capped`: `edges[..m]`, `extra`,
//! `spelling`, `inline16`, `lens` with the 256 literal slots pre-populated, `predecessor`, `m`,
//! `mark`, `bits`) by block index during a `Record` pass, sharing repeated blocks across streams.
//! Entering `Replay` freezes the store into a lock-free `OnceLock` snapshot; `with_built` borrows
//! `&Built` directly for emission without copying any structure.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

use super::frozen::FrozenSlot;
use super::{Fnv1a, Mode};

const INLINE: usize = 16;

static MODE: AtomicU8 = AtomicU8::new(Mode::Real as u8);
static CONSTRUCTED: AtomicU64 = AtomicU64::new(0);
static REPLAYED: AtomicU64 = AtomicU64::new(0);
static REFUSED_REPLAYED: AtomicU64 = AtomicU64::new(0);

/// Switch the construction oracle's mode for decodes started from now on.
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

pub(crate) struct Built {
    pub(crate) n: u32,
    pub(crate) m: u32,
    pub(crate) mark: u32,
    pub(crate) bits: u32,
    pub(crate) predecessor: Option<u32>,
    pub(crate) edges: Box<[u32]>,
    pub(crate) extra: Box<[u32]>,
    pub(crate) spelling: Box<[u8]>,
    pub(crate) inline16: Box<[[u8; INLINE]]>,
    pub(crate) lens: Box<[u8]>,
}

enum Entry {
    Built(Built),
    Refused,
}

impl Built {
    #[allow(clippy::too_many_arguments)]
    fn same(
        &self,
        n: u32,
        m: u32,
        mark: u32,
        bits: u32,
        predecessor: Option<u32>,
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        inline16: &[[u8; INLINE]],
        lens: &[u8],
    ) -> bool {
        self.n == n
            && self.m == m
            && self.mark == mark
            && self.bits == bits
            && self.predecessor == predecessor
            && *self.edges == *edges
            && *self.extra == *extra
            && *self.spelling == *spelling
            && *self.inline16 == *inline16
            && *self.lens == *lens
    }

    fn byte_size(&self) -> u64 {
        std::mem::size_of::<Entry>() as u64
            + (self.edges.len() * std::mem::size_of::<u32>()) as u64
            + (self.extra.len() * std::mem::size_of::<u32>()) as u64
            + self.spelling.len() as u64
            + (self.inline16.len() * INLINE) as u64
            + self.lens.len() as u64
    }
}

#[allow(clippy::too_many_arguments)]
fn hash_built(
    n: u32,
    m: u32,
    mark: u32,
    bits: u32,
    predecessor: Option<u32>,
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
) -> u64 {
    let mut h = Fnv1a::new();
    h.bytes(&[1]);
    h.bytes(&n.to_le_bytes());
    h.bytes(&m.to_le_bytes());
    h.bytes(&mark.to_le_bytes());
    h.bytes(&bits.to_le_bytes());
    match predecessor {
        Some(p) => {
            h.bytes(&[1]);
            h.bytes(&p.to_le_bytes());
        }
        None => h.bytes(&[0]),
    }
    h.bytes(&(edges.len() as u64).to_le_bytes());
    for &v in edges {
        h.bytes(&v.to_le_bytes());
    }
    h.bytes(&(extra.len() as u64).to_le_bytes());
    for &v in extra {
        h.bytes(&v.to_le_bytes());
    }
    h.bytes(&(spelling.len() as u64).to_le_bytes());
    h.bytes(spelling);
    h.bytes(&(inline16.len() as u64).to_le_bytes());
    for w in inline16 {
        h.bytes(w);
    }
    h.bytes(&(lens.len() as u64).to_le_bytes());
    h.bytes(lens);
    h.finish()
}

#[derive(Default)]
struct Store {
    blocks: Vec<Option<Arc<Entry>>>,
    distinct: HashMap<u64, Vec<Arc<Entry>>>,
    refused: Option<Arc<Entry>>,
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
        let distinct_built: u64 = s.distinct.values().map(|v| v.len() as u64).sum();
        let distinct_refused = u64::from(s.refused.is_some());
        (
            s.blocks.len() as u64,
            distinct_built + distinct_refused,
            s.bytes,
            s.conflicts,
            s.blocks.iter().filter(|b| b.is_none()).count() as u64,
        )
    })
}

/// Real `construct_capped` executions since the last call (and reset).
pub fn take_constructed() -> u64 {
    CONSTRUCTED.swap(0, Ordering::SeqCst)
}

/// Open blocks whose built structures were replayed since the last call (and reset).
pub fn take_replayed() -> u64 {
    REPLAYED.swap(0, Ordering::SeqCst)
}

/// Blocks whose recorded Open refusal was replayed since the last call (and reset).
pub fn take_refused_replayed() -> u64 {
    REFUSED_REPLAYED.swap(0, Ordering::SeqCst)
}

/// Fallback `RwLock` acquisitions during `Replay` since the last call (0 on the `OnceLock` path).
pub fn take_lock_acquires() -> u64 {
    FROZEN.take_lock_acquires()
}

/// Whether `Replay` is currently serving from the lock-free `OnceLock` fast path.
pub fn is_frozen_lock_free() -> bool {
    FROZEN.is_once_active()
}

pub(crate) fn count_constructed() {
    CONSTRUCTED.fetch_add(1, Ordering::Relaxed);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn record_built(
    index: u64,
    n: u32,
    m: u32,
    mark: u32,
    bits: u32,
    predecessor: Option<u32>,
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
) {
    let h = hash_built(
        n,
        m,
        mark,
        bits,
        predecessor,
        edges,
        extra,
        spelling,
        inline16,
        lens,
    );
    let mut g = STORE.write().unwrap_or_else(|e| e.into_inner());
    let st = g.get_or_insert_with(Store::default);
    let found = st.distinct.get(&h).and_then(|v| {
        v.iter()
            .find(|e| match e.as_ref() {
                Entry::Built(b) => b.same(
                    n,
                    m,
                    mark,
                    bits,
                    predecessor,
                    edges,
                    extra,
                    spelling,
                    inline16,
                    lens,
                ),
                Entry::Refused => false,
            })
            .cloned()
    });
    let entry = found.unwrap_or_else(|| {
        let built = Built {
            n,
            m,
            mark,
            bits,
            predecessor,
            edges: edges.into(),
            extra: extra.into(),
            spelling: spelling.into(),
            inline16: inline16.into(),
            lens: lens.into(),
        };
        st.bytes += built.byte_size();
        let e = Arc::new(Entry::Built(built));
        st.distinct.entry(h).or_default().push(e.clone());
        e
    });
    let i = index as usize;
    if st.blocks.len() <= i {
        st.blocks.resize(i + 1, None);
    }
    match &st.blocks[i] {
        Some(prev) => {
            let same = match prev.as_ref() {
                Entry::Built(b) => b.same(
                    n,
                    m,
                    mark,
                    bits,
                    predecessor,
                    edges,
                    extra,
                    spelling,
                    inline16,
                    lens,
                ),
                Entry::Refused => false,
            };
            if !same {
                st.conflicts += 1;
            }
        }
        None => st.blocks[i] = Some(entry),
    }
    FROZEN.mark_dirty();
}

pub(crate) fn record_refused(index: u64) {
    let mut g = STORE.write().unwrap_or_else(|e| e.into_inner());
    let st = g.get_or_insert_with(Store::default);
    let entry = match &st.refused {
        Some(e) => e.clone(),
        None => {
            let e = Arc::new(Entry::Refused);
            st.bytes += std::mem::size_of::<Entry>() as u64;
            st.refused = Some(e.clone());
            e
        }
    };
    let i = index as usize;
    if st.blocks.len() <= i {
        st.blocks.resize(i + 1, None);
    }
    match &st.blocks[i] {
        Some(prev) if !matches!(prev.as_ref(), Entry::Refused) => st.conflicts += 1,
        Some(_) => {}
        None => st.blocks[i] = Some(entry),
    }
    FROZEN.mark_dirty();
}

pub(crate) enum Replay<R> {
    Built(R),
    Refused,
    Miss,
}

/// Zero-copy borrowed construction hand-over: borrows `&Built` straight from the lock-free
/// frozen store (0 bytes copied) and invokes `f(&Built)`.
pub(crate) fn with_built<R>(index: u64, n: u32, f: impl FnOnce(&Built) -> R) -> Replay<R> {
    let res = FROZEN.with(&TLS, |blocks| {
        let entry = blocks.get(index as usize).and_then(|b| b.as_ref());
        match entry {
            None => Replay::Miss,
            Some(e) => match e.as_ref() {
                Entry::Refused => {
                    REFUSED_REPLAYED.fetch_add(1, Ordering::Relaxed);
                    Replay::Refused
                }
                Entry::Built(b) => {
                    debug_assert_eq!(b.n, n);
                    REPLAYED.fetch_add(1, Ordering::Relaxed);
                    Replay::Built(f(b))
                }
            },
        }
    });
    res.unwrap_or(Replay::Miss)
}
