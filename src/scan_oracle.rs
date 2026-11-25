//! Research only (`scan-oracle`): the bench-four-arm diagnostic's scanner side
//! (docs/journal/2026-11-24-four-arm-registration.md), for examples/four_arm.rs. The question
//! the four arms ask together with `supplied-codes` (the emission side): does removing the
//! scanner's work and Open's code-ordering together free whole-decoder time that removing either
//! alone does not?
//!
//! **The modes** (a process-global switch the harness sets between decodes; the parallel
//! decoder's scanner thread reads it once, at its start):
//! - [`Mode::Scan`]: the scanner as shipped (arms A and B);
//! - [`Mode::Record`], the oracle pass outside every clock: the scanner as shipped, each block's
//!   symbols (its header fields and its MTF/RLE2 symbols, `block::Symbols`) recorded into the
//!   store by the block's index in the input (counted across streams);
//! - [`Mode::Replay`] (arms C and D): no bit is read and no Huffman code decoded. For each slot
//!   the production scanner would fill, in the same order and under the same slot limit, the
//!   block's recorded symbols are copied into the reused symbols buffer (one `memcpy` of its
//!   u16 symbols, charged to the delivery) and sent through the production bounded job queue,
//!   exactly as the scanner sends a block it has read. The compressed input is still fed
//!   (`BzReader`'s reader thread) and drained, its chunks recycled as the scanner's window
//!   recycles them. Everything downstream is unchanged and timed: the workers' MTF and RLE2
//!   inversion, the runs, the BWT inversion (Open's construction and emission), RLE1, each
//!   block's CRC, the ordered output. What the replay removes: the bit reading, the Huffman
//!   tables and selectors, the symbol decoding, the stream headers and the stream's combined
//!   CRC check.
//!
//! **The store** holds each distinct block's symbols once (the corpus's repeated copies share
//! them, found by an FNV-1a hash of the symbols and header, confirmed equal), and an index from
//! the block's position to its entry. Its bytes are reported ([`store_size`]).
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

use crate::block::Symbols;

/// What the parallel decoder's scanner does
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    /// the scanner as shipped
    Scan = 0,
    /// the scanner as shipped, each block's symbols recorded into the store
    Record = 1,
    /// the recorded symbols delivered in place of the scanner's reading
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
static REPLAYED: AtomicU64 = AtomicU64::new(0);

/// Switch the mode for the decodes started from now on
pub fn set_mode(m: Mode) {
    MODE.store(m as u8, Ordering::SeqCst);
}

/// The mode now set
pub fn mode() -> Mode {
    Mode::of(MODE.load(Ordering::Relaxed))
}

/// A recorded block: `Symbols`' fields, the symbols boxed
pub struct Entry {
    pub crc: u32,
    pub orig_ptr: u32,
    pub seq_to_unseq: [u8; 256],
    pub syms: Box<[u16]>,
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
    /// by the block's index in the input
    blocks: Vec<Option<Arc<Entry>>>,
    /// the distinct entries, by hash
    distinct: HashMap<u64, Vec<Arc<Entry>>>,
    bytes: u64,
    /// a block index recorded twice with different symbols
    conflicts: u64,
}

static STORE: RwLock<Option<Store>> = RwLock::new(None);

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

/// Empty the store
pub fn clear_store() {
    *STORE.write().unwrap_or_else(|e| e.into_inner()) = Some(Store::default());
}

/// Record block `index`'s symbols (the oracle pass)
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
            syms: s.syms.clone().into_boxed_slice(),
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

/// The blocks a replaying scanner has delivered since the last call
pub fn take_replayed() -> u64 {
    REPLAYED.swap(0, Ordering::SeqCst)
}

/// Block `index`'s recorded symbols, copied into `syms` (cleared, reused), as the scanner
/// would hand them over: None past the last recorded block
pub(crate) fn replay(index: u64, mut syms: Vec<u16>) -> Option<Symbols> {
    let entry = {
        let g = STORE.read().unwrap_or_else(|e| e.into_inner());
        g.as_ref()?.blocks.get(index as usize)?.clone()?
    };
    syms.clear();
    syms.extend_from_slice(&entry.syms);
    REPLAYED.fetch_add(1, Ordering::Relaxed);
    Some(Symbols {
        crc: entry.crc,
        orig_ptr: entry.orig_ptr,
        seq_to_unseq: entry.seq_to_unseq,
        syms,
        eob: entry.eob,
        max_len: entry.max_len,
    })
}
