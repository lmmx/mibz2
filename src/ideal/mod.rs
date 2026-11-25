//! Research only (`ideal-construction`, `ideal-mtf`, `ideal-rle1`, `ideal-delivery`):
//! the ideal-parts matrix's oracles (`docs/journal/2026-11-24-algorithm-level/ideal-parts/`).
//!
//! Each oracle follows `scan_oracle`'s shape:
//! - a process-global mode set between decodes (`Real`, `Record` or `Replay`; for delivery,
//!   `Real`, `Discard` or `Check`);
//! - a `Record` pass that runs the part as shipped and stores its output by block index,
//!   sharing repeated blocks;
//! - a `Replay` mode that skips the part's own work and copies its recorded output once
//!   (charged to the handover) into the next stage's buffers;
//! - store byte counts and execution/replay counters.

#[cfg(feature = "ideal-construction")]
pub mod construction;
#[cfg(feature = "ideal-delivery")]
pub mod delivery;
#[cfg(feature = "ideal-mtf")]
pub mod mtf;
#[cfg(feature = "ideal-rle1")]
pub mod rle1;

#[cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
))]
use std::cell::Cell;

/// What an ideal-parts oracle does for a decode
#[cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    /// the part as shipped
    Real = 0,
    /// the part as shipped, each block's output recorded into the store
    Record = 1,
    /// the recorded output delivered in place of the part's own work
    Replay = 2,
}

#[cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
))]
impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Real => "real",
            Mode::Record => "record",
            Mode::Replay => "replay",
        }
    }

    pub fn parse(s: &str) -> Option<Mode> {
        [Mode::Real, Mode::Record, Mode::Replay]
            .into_iter()
            .find(|m| m.name() == s)
    }

    pub(crate) fn of(v: u8) -> Mode {
        match v {
            1 => Mode::Record,
            2 => Mode::Replay,
            _ => Mode::Real,
        }
    }
}

#[cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
))]
thread_local! {
    /// The parallel worker's current block index (counted across streams, as `scan_oracle` counts)
    static BLOCK: Cell<Option<u64>> = const { Cell::new(None) };
}

/// Restores the worker thread's previous block index on drop (including on panic)
#[cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
))]
pub(crate) struct BlockGuard(Option<u64>);

#[cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
))]
impl Drop for BlockGuard {
    fn drop(&mut self) {
        BLOCK.with(|c| c.set(self.0));
    }
}

/// Bind `index` as the block this worker thread is currently decoding
#[cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
))]
pub(crate) fn enter_block(index: u64) -> BlockGuard {
    let prev = BLOCK.with(|c| c.replace(Some(index)));
    BlockGuard(prev)
}

/// The block index bound to this worker thread, if any
#[cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
))]
pub(crate) fn current_block() -> Option<u64> {
    BLOCK.with(Cell::get)
}

/// 64-bit FNV-1a hasher for deduplicating identical recorded blocks across streams
#[cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
))]
pub(crate) struct Fnv1a(u64);

#[cfg(any(
    feature = "ideal-construction",
    feature = "ideal-mtf",
    feature = "ideal-rle1",
))]
impl Fnv1a {
    #[inline]
    pub(crate) fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    #[inline]
    pub(crate) fn bytes(&mut self, data: &[u8]) {
        let mut h = self.0;
        for &b in data {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        self.0 = h;
    }

    #[inline]
    pub(crate) fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(all(
    test,
    any(
        feature = "ideal-construction",
        feature = "ideal-mtf",
        feature = "ideal-rle1",
    )
))]
mod tests {
    use super::*;

    #[test]
    fn mode_names_and_roundtrip() {
        for m in [Mode::Real, Mode::Record, Mode::Replay] {
            assert_eq!(Mode::parse(m.name()), Some(m));
            assert_eq!(Mode::of(m as u8), m);
        }
        assert_eq!(Mode::parse("other"), None);
    }

    #[test]
    fn block_guard_restores_on_drop() {
        assert_eq!(current_block(), None);
        {
            let _g1 = enter_block(7);
            assert_eq!(current_block(), Some(7));
            {
                let _g2 = enter_block(42);
                assert_eq!(current_block(), Some(42));
            }
            assert_eq!(current_block(), Some(7));
        }
        assert_eq!(current_block(), None);
    }
}
