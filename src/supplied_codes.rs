//! Research only (`supplied-codes`): zero-copy lock-free factor E (target T1) of the
//! ideal-parts matrix and the bench-supplied-codes diagnostic (`IDEAL-PARTS-2.md` §3;
//! `docs/journal/2026-11-24-supplied-codes-registration.md`).
//!
//! **The modes** (a process-global switch set between decodes; `src/open/supplied.rs` is the
//! kernel's side):
//! - [`Mode::Open`]: the kernel as shipped (`Kernel::walk`: pointer chase + word spelling);
//! - [`Mode::Supplied`] (`E_chase`): skips the pointer chase, borrows `&[u32]` straight from the
//!   lock-free `OnceLock` frozen store without copying into `edges` (`HandEDeliver`, 0 bytes
//!   copied), and spells words into `out` (`EConsume`, timed separately);
//! - [`Mode::SuppliedFull`] (`E_full`): when `R` (`ideal-rle1`) is also replaying the block,
//!   skips BOTH the pointer chase AND word spelling (`EConsume`), leaving `out` untouched so the
//!   floor's pure plumbing residual (Residual B) can be read directly. Falls back to
//!   [`Mode::Supplied`] if `R` is not replaying the block so output stays byte-identical in every
//!   mode;
//! - [`Mode::Record`]: records each Open block's output-order word codes into the store, then
//!   runs `Kernel::walk`;
//! - [`Mode::Check`]: runs `Kernel::walk` and borrowed `Supplied` consumption side by side,
//!   counting any mismatch.
//!
//! **Lock-free timed runs:** [`set_record_recs(false)`] gates off per-block `RECS.lock()` pushes
//! and `Instant::now()` reads during timed floor runs while keeping lock-free atomic [`Counts`]
//! (`take_counts`) and `character` / `ideal::accounting` phase ledgers active.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use crate::ideal::frozen::FrozenSlot;

/// What the open kernel's emit stage does with a block
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    /// arm A: the kernel as shipped
    Open = 0,
    /// arm B (`E_chase`): the block's codes borrowed in output order, consumed sequentially
    Supplied = 1,
    /// the oracle pass: the codes recorded into the store, then arm A's emit
    Record = 2,
    /// the check: arm A's bytes and arm B's compared on every block
    Check = 3,
    /// `E_full`: skips both pointer chase and word spelling (`E_consume`) when R replays
    SuppliedFull = 4,
}

impl Mode {
    /// The mode's name in the harness's arguments and records
    pub fn name(self) -> &'static str {
        match self {
            Mode::Open => "open",
            Mode::Supplied => "supplied",
            Mode::Record => "record",
            Mode::Check => "check",
            Mode::SuppliedFull => "supplied-full",
        }
    }

    /// The mode named `s`
    pub fn parse(s: &str) -> Option<Mode> {
        match s {
            "open" => Some(Mode::Open),
            "supplied" | "chase" => Some(Mode::Supplied),
            "record" => Some(Mode::Record),
            "check" => Some(Mode::Check),
            "supplied-full" | "full" => Some(Mode::SuppliedFull),
            _ => None,
        }
    }

    pub(crate) fn of(v: u8) -> Mode {
        match v {
            1 => Mode::Supplied,
            2 => Mode::Record,
            3 => Mode::Check,
            4 => Mode::SuppliedFull,
            _ => Mode::Open,
        }
    }
}

static MODE: AtomicU8 = AtomicU8::new(Mode::Open as u8);
static CONDITION: AtomicBool = AtomicBool::new(false);
static RECORD_RECS: AtomicBool = AtomicBool::new(true);
static COLLISIONS: AtomicU64 = AtomicU64::new(0);
static MISMATCHES: AtomicU64 = AtomicU64::new(0);

// Lock-free atomic block counts (always maintained even when `RECS` is gated off).
static CNT_OPEN: AtomicU64 = AtomicU64::new(0);
static CNT_WALKED: AtomicU64 = AtomicU64::new(0);
static CNT_SUPPLIED_CHASE: AtomicU64 = AtomicU64::new(0);
static CNT_SUPPLIED_FULL: AtomicU64 = AtomicU64::new(0);
static CNT_CONSUMED: AtomicU64 = AtomicU64::new(0);
static CNT_REFUSED: AtomicU64 = AtomicU64::new(0);

/// Lock-free emission counters collected since the last [`take_counts`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub open: u64,
    pub walked: u64,
    pub supplied_chase: u64,
    pub supplied_full: u64,
    pub consumed: u64,
    pub refused: u64,
}

/// Switch the mode for the blocks the kernel starts from now on.
///
/// Entering `Supplied`, `SuppliedFull`, or `Check` freezes the recorded store into the
/// lock-free `OnceLock` snapshot.
pub fn set_mode(m: Mode) {
    if matches!(m, Mode::Supplied | Mode::SuppliedFull | Mode::Check) {
        freeze_store();
    }
    MODE.store(m as u8, Ordering::SeqCst);
}

/// The mode now set
#[inline]
pub fn mode() -> Mode {
    Mode::of(MODE.load(Ordering::Relaxed))
}

/// Condition the consumer's inputs before its clock (both arms), or not
pub fn set_condition(on: bool) {
    CONDITION.store(on, Ordering::SeqCst);
}

/// Whether the consumer's inputs are conditioned
#[inline]
pub fn condition() -> bool {
    CONDITION.load(Ordering::Relaxed)
}

/// Gate per-block `RECS.lock()` recording on or off (`true` by default for compatibility with
/// `examples/supplied_codes.rs`, `examples/four_arm.rs` and `examples/character.rs`; set `false`
/// during timed zero-copy floor decodes to eliminate mutex contention and per-block clock reads).
pub fn set_record_recs(on: bool) {
    RECORD_RECS.store(on, Ordering::SeqCst);
}

/// Whether per-block `Rec` recording under `RECS.lock()` is currently enabled
#[inline]
pub fn record_recs() -> bool {
    RECORD_RECS.load(Ordering::Relaxed)
}

/// Lock-free emission counts since the last call (and reset)
pub fn take_counts() -> Counts {
    Counts {
        open: CNT_OPEN.swap(0, Ordering::SeqCst),
        walked: CNT_WALKED.swap(0, Ordering::SeqCst),
        supplied_chase: CNT_SUPPLIED_CHASE.swap(0, Ordering::SeqCst),
        supplied_full: CNT_SUPPLIED_FULL.swap(0, Ordering::SeqCst),
        consumed: CNT_CONSUMED.swap(0, Ordering::SeqCst),
        refused: CNT_REFUSED.swap(0, Ordering::SeqCst),
    }
}

#[inline]
pub(crate) fn count_open() {
    CNT_OPEN.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub(crate) fn count_walked() {
    CNT_WALKED.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub(crate) fn count_supplied_chase() {
    CNT_SUPPLIED_CHASE.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub(crate) fn count_supplied_full() {
    CNT_SUPPLIED_FULL.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub(crate) fn count_consumed() {
    CNT_CONSUMED.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub(crate) fn count_refused() {
    CNT_REFUSED.fetch_add(1, Ordering::Relaxed);
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

type Store = HashMap<Key, Arc<[u32]>>;

static STORE: RwLock<Option<Store>> = RwLock::new(None);
static FROZEN: FrozenSlot<Store> = FrozenSlot::new();

thread_local! {
    static TLS: RefCell<(u64, Option<Arc<Store>>)> = const { RefCell::new((0, None)) };
}

fn freeze_store() {
    FROZEN.freeze(|| {
        let g = STORE.read().unwrap_or_else(|e| e.into_inner());
        let st = g.as_ref()?;
        (!st.is_empty()).then(|| st.clone())
    });
}

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
            s.insert(key, Arc::from(codes));
            FROZEN.mark_dirty();
        }
    }
}

/// Zero-copy borrowed lookup of `key`'s codes straight from the lock-free frozen store.
/// Falls back lazily to freezing if called directly (e.g. unit tests calling `staged` without
/// `set_mode`).
#[inline]
pub(crate) fn with_codes<R>(key: &Key, f: impl FnOnce(&[u32]) -> R) -> Option<R> {
    let mut f = Some(f);
    if let Some(res) = FROZEN.with(&TLS, |map| {
        map.get(key)
            .filter(|c| !c.is_empty())
            .map(|c| (f.take().unwrap())(c))
    }) {
        return res;
    }
    // Cold path: direct unit test called `staged(Mode::Supplied, ...)` without `set_mode`.
    freeze_store();
    FROZEN
        .with(&TLS, |map| {
            map.get(key)
                .filter(|c| !c.is_empty())
                .map(|c| (f.take().unwrap())(c))
        })
        .flatten()
}

/// The codes held for `key`, copied (the check's comparison and unit tests)
#[cfg(test)]
pub(crate) fn held(key: &Key) -> Option<Vec<u32>> {
    with_codes(key, |c| c.to_vec())
}

/// Empty the store and its collision count, and invalidate any frozen snapshot
pub fn clear_store() {
    *STORE.write().unwrap_or_else(|e| e.into_inner()) = None;
    FROZEN.invalidate();
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

/// Fallback `RwLock` acquisitions during `Supplied` / `SuppliedFull` since the last call
/// (0 on the `OnceLock` fast path)
pub fn take_lock_acquires() -> u64 {
    FROZEN.take_lock_acquires()
}

/// Whether `Supplied` / `SuppliedFull` is currently serving from the lock-free `OnceLock` path
pub fn is_frozen_lock_free() -> bool {
    FROZEN.is_once_active()
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
    /// the delivery (zero-copy lookup in the frozen store), ns; else two adjacent clock reads' gap
    pub deliver_ns: u64,
    /// the conditioning read, ns; else two adjacent clock reads' gap
    pub cond_ns: u64,
    /// the consumer (word spelling `E_consume`, or `0` in `E_full`), ns
    pub consume_ns: u64,
    /// the bytes the worker's kernel holds after the block, every buffer's capacity
    pub scratch: u64,
    /// the block's key id ([`Key::id`]); 0 for a refused block
    pub id: u64,
}

static RECS: Mutex<Vec<Rec>> = Mutex::new(Vec::new());

pub(crate) fn record(r: Rec) {
    if record_recs() {
        RECS.lock().unwrap_or_else(|e| e.into_inner()).push(r);
    }
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
        for m in [
            Mode::Open,
            Mode::Supplied,
            Mode::Record,
            Mode::Check,
            Mode::SuppliedFull,
        ] {
            assert_eq!(Mode::parse(m.name()), Some(m));
            assert_eq!(Mode::of(m as u8), m);
        }
        assert_eq!(Mode::parse("chase"), Some(Mode::Supplied));
        assert_eq!(Mode::parse("full"), Some(Mode::SuppliedFull));
        assert_eq!(Mode::parse("base"), None);
    }
}
