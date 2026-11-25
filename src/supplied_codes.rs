//! Research only (`supplied-codes`): the bench-supplied-codes diagnostic's switch, store and
//! records (docs/journal/2026-11-24-supplied-codes-registration.md), for
//! examples/supplied_codes.rs. The question: if the open kernel's emitter were handed the
//! block's existing dictionary word codes already in marked-cycle output order, so that it no
//! longer walks the retained cycle to find that order, how much faster is the decoder?
//!
//! **The modes** (a process-global switch the harness sets between decodes, read once a block
//! at its start; `src/open/supplied.rs` is the kernel's side):
//! - [`Mode::Open`], arm A: the kernel as shipped, construction and the current emitter
//!   (`emit_bidirectional` on its two-ended blocks, `emit` on the others);
//! - [`Mode::Supplied`], arm B: the same construction, then the block's code sequence
//!   q_0..q_{L-1} (L the marked cycle's length) taken from the [store](store) by the block's
//!   [`Key`] and copied into the first L words of the packed-edge buffer (`edges`, 4m bytes),
//!   then consumed from the front (and, on a two-ended block, the back) with the shipped
//!   emitters' own short/long-word appends. Nothing else is supplied: no output, no offsets,
//!   no word boundaries, no dictionary;
//! - [`Mode::Record`], the oracle pass outside the clock: the construction, the cycle walked
//!   once to record its codes into the store, then arm A's emit;
//! - [`Mode::Check`], the per-round check outside the clock: arm A's bytes, then arm B's from
//!   the store, compared byte for byte on every block (a difference counts a
//!   [mismatch](mismatches)); arm B's bytes go to the output, where the decoder checks the
//!   block's and the stream's CRCs.
//!
//! **Conditioning** ([`set_condition`], the diagnostic receipt's decodes): in both arms, after
//! the delivery and before the consumer's clock, one sequential read of every buffer either
//! consumer may read (`edges[..m]`, `extra`, the inline words, their lengths and the long words'
//! spelling), one load a 64-byte line. Off, nothing is touched.
//!
//! **The records:** five clock reads a block in every mode (construct | deliver | condition |
//! consume), the block's rows, retained rows, cycle length (arm B and the oracle), whether it
//! was two-ended, refused or supplied, the kernel's scratch and the block's key id.
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{Mutex, RwLock};
use std::time::Duration;

/// What the open kernel's emit stage does with a block
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    /// arm A: the kernel as shipped
    Open = 0,
    /// arm B: the block's codes supplied in output order, consumed sequentially
    Supplied = 1,
    /// the oracle pass: the codes recorded into the store, then arm A's emit
    Record = 2,
    /// the check: arm A's bytes and arm B's compared on every block
    Check = 3,
}

impl Mode {
    /// The mode's name in the harness's arguments and records
    pub fn name(self) -> &'static str {
        match self {
            Mode::Open => "open",
            Mode::Supplied => "supplied",
            Mode::Record => "record",
            Mode::Check => "check",
        }
    }

    /// The mode named `s`
    pub fn parse(s: &str) -> Option<Mode> {
        [Mode::Open, Mode::Supplied, Mode::Record, Mode::Check]
            .into_iter()
            .find(|m| m.name() == s)
    }

    pub(crate) fn of(v: u8) -> Mode {
        match v {
            1 => Mode::Supplied,
            2 => Mode::Record,
            3 => Mode::Check,
            _ => Mode::Open,
        }
    }
}

static MODE: AtomicU8 = AtomicU8::new(Mode::Open as u8);
static CONDITION: AtomicBool = AtomicBool::new(false);
static COLLISIONS: AtomicU64 = AtomicU64::new(0);
static MISMATCHES: AtomicU64 = AtomicU64::new(0);

/// Switch the mode for the blocks the kernel starts from now on (between decodes: a decode's
/// workers are spawned after it, so they see it)
pub fn set_mode(m: Mode) {
    MODE.store(m as u8, Ordering::SeqCst);
}

/// The mode now set
pub fn mode() -> Mode {
    Mode::of(MODE.load(Ordering::Relaxed))
}

/// Condition the consumer's inputs before its clock (both arms), or not
pub fn set_condition(on: bool) {
    CONDITION.store(on, Ordering::SeqCst);
}

/// Whether the consumer's inputs are conditioned
pub fn condition() -> bool {
    CONDITION.load(Ordering::Relaxed)
}

/// A block's identity in the store, from what the construction knows in O(1): equal for equal
/// blocks (the corpus's repeated copies share their codes), and checked against collisions
/// when recorded ([`collisions`]) and again by every check decode's bytes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub n: u32,
    pub p: u32,
    /// retained rows
    pub m: u32,
    /// the marked row's retained rank
    pub mark: u32,
    /// the block's run records, their count, first and last
    pub records: u32,
    pub first: u16,
    pub last: u16,
}

impl Key {
    /// A 64-bit id of the key (FNV-1a over its fields), for the records' distinct blocks
    pub fn id(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for v in [
            self.n,
            self.p,
            self.m,
            self.mark,
            self.records,
            u32::from(self.first),
            u32::from(self.last),
        ] {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
        h
    }
}

type Store = HashMap<Key, Box<[u32]>>;
static STORE: RwLock<Option<Store>> = RwLock::new(None);

/// Record a block's code sequence (the oracle pass). A key already held with other codes is a
/// collision, counted, and the codes first held are kept
pub(crate) fn store(key: Key, codes: &[u32]) {
    let mut s = STORE.write().unwrap_or_else(|e| e.into_inner());
    let s = s.get_or_insert_with(HashMap::new);
    match s.get(&key) {
        Some(held) if **held != *codes => {
            COLLISIONS.fetch_add(1, Ordering::Relaxed);
        }
        Some(_) => {}
        None => {
            s.insert(key, codes.into());
        }
    }
}

/// Copy the codes held for `key` into the front of `dst` (arm B's delivery): their count, or
/// `None` if none are held or they don't fit
pub(crate) fn deliver(key: &Key, dst: &mut [u32]) -> Option<usize> {
    let s = STORE.read().unwrap_or_else(|e| e.into_inner());
    let codes = s.as_ref()?.get(key)?;
    let l = codes.len();
    if l == 0 || l > dst.len() {
        return None;
    }
    dst[..l].copy_from_slice(codes);
    Some(l)
}

/// The codes held for `key`, copied (the check's comparison)
pub(crate) fn held(key: &Key) -> Option<Vec<u32>> {
    let s = STORE.read().unwrap_or_else(|e| e.into_inner());
    s.as_ref()?.get(key).map(|c| c.to_vec())
}

/// Empty the store and its collision count
pub fn clear_store() {
    *STORE.write().unwrap_or_else(|e| e.into_inner()) = None;
    COLLISIONS.store(0, Ordering::SeqCst);
}

/// The store's (distinct keys, codes held, bytes of codes)
pub fn store_size() -> (usize, usize, usize) {
    let s = STORE.read().unwrap_or_else(|e| e.into_inner());
    s.as_ref().map_or((0, 0, 0), |s| {
        let codes: usize = s.values().map(|c| c.len()).sum();
        (s.len(), codes, 4 * codes)
    })
}

/// Keys recorded with codes other than those held for them, since the store was last cleared
pub fn collisions() -> u64 {
    COLLISIONS.load(Ordering::SeqCst)
}

pub(crate) fn mismatch() {
    MISMATCHES.fetch_add(1, Ordering::Relaxed);
}

/// The check's blocks whose arm-B bytes differed from arm A's, or whose held codes differed from
/// the block's own cycle, since the last call (and reset)
pub fn mismatches() -> u64 {
    MISMATCHES.swap(0, Ordering::SeqCst)
}

/// One block's record
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rec {
    /// the block's rows
    pub n: u32,
    /// its retained rows; 0 for a block the first pass refused
    pub m: u32,
    /// the marked cycle's length (the codes supplied, recorded or checked); 0 in `open`
    pub cycle: u32,
    /// the mode the block ran under
    pub mode: Mode,
    /// the consumer's inputs conditioned before its clock
    pub cond: bool,
    /// the two-ended emitter's block (`QUEUE && !wide && m >= 65,536`), else the forward one
    pub bidi: bool,
    /// the first pass refused the block (it went to `LazyCells`)
    pub refused: bool,
    /// arm B (or the check) found the block's codes in the store
    pub supplied: bool,
    /// the construction, ns
    pub construct_ns: u64,
    /// the delivery (arm B: the lookup and the copy into `edges`), ns; else two adjacent clock
    /// reads' gap
    pub deliver_ns: u64,
    /// the conditioning read, ns; else two adjacent clock reads' gap
    pub cond_ns: u64,
    /// the consumer (the emit), ns
    pub consume_ns: u64,
    /// the bytes the worker's kernel holds after the block, every buffer's capacity
    pub scratch: u64,
    /// the block's key id ([`Key::id`]); 0 for a refused block
    pub id: u64,
}

static RECS: Mutex<Vec<Rec>> = Mutex::new(Vec::new());

pub(crate) fn record(r: Rec) {
    RECS.lock().unwrap_or_else(|e| e.into_inner()).push(r);
}

/// The records since the last `take`, in the order the workers finished them
pub fn take() -> Vec<Rec> {
    std::mem::take(&mut *RECS.lock().unwrap_or_else(|e| e.into_inner()))
}

pub(crate) fn ns(d: Duration) -> u64 {
    d.as_nanos().min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_names() {
        for m in [Mode::Open, Mode::Supplied, Mode::Record, Mode::Check] {
            assert_eq!(Mode::parse(m.name()), Some(m));
            assert_eq!(Mode::of(m as u8), m);
        }
        assert_eq!(Mode::parse("base"), None);
    }
}
