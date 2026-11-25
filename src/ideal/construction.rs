//! Research only (`ideal-construction`): factor C (target T2) of the ideal-parts matrix.
//!
//! Records Open's built structures (`Kernel::construct_capped`: `edges[..m]`, `extra`,
//! `spelling`, `inline16[256..]`, `lens[256..]`, `predecessor`, `m`, `mark`, `bits`) by block
//! index during a `Record` pass, sharing repeated blocks across streams. In `Replay`, skips
//! `construct_capped` completely, copies the recorded structures once into `Kernel`'s buffers
//! (charged to the handover), and runs `Kernel::walk` (emission) on them. Upstream MTF/RLE2
//! still runs in full, its output discarded where construction would consume it.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

use super::{Fnv1a, Mode};

const INLINE: usize = 16;

static MODE: AtomicU8 = AtomicU8::new(Mode::Real as u8);
static CONSTRUCTED: AtomicU64 = AtomicU64::new(0);
static REPLAYED: AtomicU64 = AtomicU64::new(0);
static REFUSED_REPLAYED: AtomicU64 = AtomicU64::new(0);

/// Switch the construction oracle's mode for decodes started from now on
pub fn set_mode(m: Mode) {
    MODE.store(m as u8, Ordering::SeqCst);
}

/// The mode now set
pub fn mode() -> Mode {
    Mode::of(MODE.load(Ordering::Relaxed))
}

struct Built {
    n: u32,
    m: u32,
    mark: u32,
    bits: u32,
    predecessor: Option<u32>,
    edges: Box<[u32]>,
    extra: Box<[u32]>,
    spelling: Box<[u8]>,
    inline16_words: Box<[[u8; INLINE]]>,
    lens_words: Box<[u8]>,
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
        inline16_words: &[[u8; INLINE]],
        lens_words: &[u8],
    ) -> bool {
        self.n == n
            && self.m == m
            && self.mark == mark
            && self.bits == bits
            && self.predecessor == predecessor
            && *self.edges == *edges
            && *self.extra == *extra
            && *self.spelling == *spelling
            && *self.inline16_words == *inline16_words
            && *self.lens_words == *lens_words
    }

    fn byte_size(&self) -> u64 {
        std::mem::size_of::<Entry>() as u64
            + (self.edges.len() * std::mem::size_of::<u32>()) as u64
            + (self.extra.len() * std::mem::size_of::<u32>()) as u64
            + self.spelling.len() as u64
            + (self.inline16_words.len() * INLINE) as u64
            + self.lens_words.len() as u64
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
    inline16_words: &[[u8; INLINE]],
    lens_words: &[u8],
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
    h.bytes(&(inline16_words.len() as u64).to_le_bytes());
    for w in inline16_words {
        h.bytes(w);
    }
    h.bytes(&(lens_words.len() as u64).to_le_bytes());
    h.bytes(lens_words);
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

/// Real `construct_capped` executions since the last call (and reset)
pub fn take_constructed() -> u64 {
    CONSTRUCTED.swap(0, Ordering::SeqCst)
}

/// Open blocks whose built structures were replayed since the last call (and reset)
pub fn take_replayed() -> u64 {
    REPLAYED.swap(0, Ordering::SeqCst)
}

/// Blocks whose recorded Open refusal was replayed since the last call (and reset)
pub fn take_refused_replayed() -> u64 {
    REFUSED_REPLAYED.swap(0, Ordering::SeqCst)
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
    inline16_words: &[[u8; INLINE]],
    lens_words: &[u8],
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
        inline16_words,
        lens_words,
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
                    inline16_words,
                    lens_words,
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
            inline16_words: inline16_words.into(),
            lens_words: lens_words.into(),
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
                    inline16_words,
                    lens_words,
                ),
                Entry::Refused => false,
            };
            if !same {
                st.conflicts += 1;
            }
        }
        None => st.blocks[i] = Some(entry),
    }
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
}

pub(crate) enum Replay {
    Built { m: usize, mark: u32, bits: u32 },
    Refused,
    Miss,
}

/// Hand over block `index`'s recorded built structures into `Kernel`'s buffers (one copy per
/// structure, charged to the handover), with the lock released before copying.
#[allow(clippy::too_many_arguments)]
pub(crate) fn replay(
    index: u64,
    n: u32,
    predecessor: &mut Option<u32>,
    edges: &mut Vec<u32>,
    extra: &mut Vec<u32>,
    spelling: &mut Vec<u8>,
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    init_literals: impl FnOnce(usize, &mut Vec<[u8; INLINE]>, &mut Vec<u8>),
) -> Replay {
    let entry = {
        let g = STORE.read().unwrap_or_else(|e| e.into_inner());
        match g
            .as_ref()
            .and_then(|s| s.blocks.get(index as usize))
            .and_then(|b| b.clone())
        {
            Some(e) => e,
            None => return Replay::Miss,
        }
    };
    match entry.as_ref() {
        Entry::Refused => {
            REFUSED_REPLAYED.fetch_add(1, Ordering::Relaxed);
            Replay::Refused
        }
        Entry::Built(b) => {
            debug_assert_eq!(b.n, n);
            *predecessor = b.predecessor;
            edges.clear();
            edges.extend_from_slice(&b.edges);
            extra.clear();
            extra.extend_from_slice(&b.extra);
            spelling.clear();
            spelling.extend_from_slice(&b.spelling);
            init_literals(b.lens_words.len(), inline16, lens);
            inline16.extend_from_slice(&b.inline16_words);
            lens.extend_from_slice(&b.lens_words);
            REPLAYED.fetch_add(1, Ordering::Relaxed);
            Replay::Built {
                m: b.m as usize,
                mark: b.mark,
                bits: b.bits,
            }
        }
    }
}
