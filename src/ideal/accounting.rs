//! Research only (`scan-oracle`, `supplied-codes`, `ideal-*`): zero-lock handover and floor
//! accounting (`IDEAL-PARTS-2.md` §3).
//!
//! **Off by default** ([`enabled`] is `false`): each instrumentation point is a single relaxed
//! atomic load, with no clock reads and no atomic writes.
//!
//! When enabled ([`set_enabled(true)`]), records per-phase nanosecond time, call count, and
//! copied/written byte count for every ideal-parts handover and for emission's word spelling:
//! - [`Phase::HandS`]: scanner oracle handover (concurrent scanner thread)
//! - [`Phase::HandM`]: MTF/RLE2 oracle handover (worker)
//! - [`Phase::HandC`]: construction oracle handover (worker)
//! - [`Phase::HandEDeliver`]: emission oracle delivery ONLY (worker, separated from word spelling)
//! - [`Phase::EConsume`]: emission word spelling ONLY (`emit_codes` / `emit_bidirectional_codes`;
//!   0 in `E_full` mode when R replays)
//! - [`Phase::HandR`]: RLE1/CRC/output oracle handover (worker)
//! - [`Phase::DeliveryCopy`] / [`Phase::DeliveryDiscard`]: in-order reader copy or discard
//!
//! Calling [`take`] returns a [`FloorAccounting`] snapshot whose [`FloorAccounting::floor_summary`]
//! breaks down wall time directly into concurrent `hand_S`, per-worker pure handover
//! (`hand_M + hand_C + hand_E_deliver + hand_R`), per-worker `E_consume` (Residual A's spelling
//! slice), and implied Residual B (pure plumbing).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Turn handover and floor accounting on or off (off by default).
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::SeqCst);
}

/// Whether handover and floor accounting is currently enabled.
#[inline]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// The phases tracked by [`FloorAccounting`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Phase {
    HandS = 0,
    HandM = 1,
    HandC = 2,
    HandEDeliver = 3,
    EConsume = 4,
    HandR = 5,
    DeliveryCopy = 6,
    DeliveryDiscard = 7,
}

const PHASE_COUNT: usize = 8;

#[repr(align(64))]
struct AtomicPhase {
    ns: AtomicU64,
    calls: AtomicU64,
    bytes: AtomicU64,
}

impl AtomicPhase {
    const fn new() -> Self {
        Self {
            ns: AtomicU64::new(0),
            calls: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
        }
    }

    fn take(&self) -> PhaseStats {
        PhaseStats {
            ns: self.ns.swap(0, Ordering::SeqCst),
            calls: self.calls.swap(0, Ordering::SeqCst),
            bytes: self.bytes.swap(0, Ordering::SeqCst),
        }
    }

    fn reset(&self) {
        self.ns.store(0, Ordering::SeqCst);
        self.calls.store(0, Ordering::SeqCst);
        self.bytes.store(0, Ordering::SeqCst);
    }
}

static PHASES: [AtomicPhase; PHASE_COUNT] = [
    AtomicPhase::new(),
    AtomicPhase::new(),
    AtomicPhase::new(),
    AtomicPhase::new(),
    AtomicPhase::new(),
    AtomicPhase::new(),
    AtomicPhase::new(),
    AtomicPhase::new(),
];

/// Accumulated wall time, call count, and bytes for one phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhaseStats {
    /// Total elapsed nanoseconds summed across threads that ran this phase.
    pub ns: u64,
    /// Number of block/read invocations recorded.
    pub calls: u64,
    /// Bytes copied (for handovers) or written/processed (for `EConsume` / delivery).
    pub bytes: u64,
}

impl PhaseStats {
    /// Elapsed milliseconds (summed across threads).
    pub fn ms(self) -> f64 {
        self.ns as f64 / 1_000_000.0
    }

    /// Elapsed milliseconds per worker thread (`workers.max(1)`).
    pub fn ms_per_thread(self, workers: usize) -> f64 {
        self.ms() / workers.max(1) as f64
    }
}

/// Complete handover and emission-spelling breakdown collected since the last [`take`] or [`reset`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FloorAccounting {
    pub hand_s: PhaseStats,
    pub hand_m: PhaseStats,
    pub hand_c: PhaseStats,
    /// Emission oracle delivery ONLY (strictly separated from [`Self::e_consume`]).
    pub hand_e_deliver: PhaseStats,
    /// Emission word spelling ONLY (`emit_codes` / `emit_bidirectional_codes`; 0 in `E_full`).
    pub e_consume: PhaseStats,
    pub hand_r: PhaseStats,
    pub delivery_copy: PhaseStats,
    pub delivery_discard: PhaseStats,
}

/// Direct floor decomposition against measured wall time (`IDEAL-PARTS-2.md` §3.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloorSummary {
    pub wall_ms: f64,
    pub workers: usize,
    /// Concurrent scanner-thread handover (`hand_S`).
    pub hand_s_ms: f64,
    /// Per-worker pure oracle handover (`(hand_M + hand_C + hand_E_deliver + hand_R) / workers`).
    pub worker_handover_ms_per_thread: f64,
    /// Per-worker emission word spelling (`e_consume / workers`); 0 in `E_full` mode.
    pub e_consume_ms_per_thread: f64,
    /// Reader copy-out time (`D = Real`).
    pub delivery_copy_ms: f64,
    /// Reader discard/check time (`D = Discard` / `Check`).
    pub delivery_discard_ms: f64,
    /// Implied Residual B (pure non-handover, non-spelling plumbing):
    /// `max(0, wall_ms - worker_handover_ms_per_thread - e_consume_ms_per_thread)`.
    pub implied_residual_b_ms: f64,
    /// Total bytes copied across all five oracle handovers (`S, M, C, E_deliver, R`).
    pub total_handover_copied_bytes: u64,
}

impl FloorAccounting {
    /// Total worker-thread pure handover nanoseconds (`hand_M + hand_C + hand_E_deliver + hand_R`).
    /// Excludes concurrent `hand_S` and excludes `e_consume`.
    pub fn worker_handover_ns(&self) -> u64 {
        self.hand_m
            .ns
            .saturating_add(self.hand_c.ns)
            .saturating_add(self.hand_e_deliver.ns)
            .saturating_add(self.hand_r.ns)
    }

    /// Total bytes copied by worker-thread oracle handovers (`M + C + E_deliver + R`).
    pub fn worker_handover_copied_bytes(&self) -> u64 {
        self.hand_m
            .bytes
            .saturating_add(self.hand_c.bytes)
            .saturating_add(self.hand_e_deliver.bytes)
            .saturating_add(self.hand_r.bytes)
    }

    /// Total bytes copied across all five oracle handovers (`S + M + C + E_deliver + R`).
    pub fn total_handover_copied_bytes(&self) -> u64 {
        self.hand_s
            .bytes
            .saturating_add(self.worker_handover_copied_bytes())
    }

    /// Combined `hand_E_deliver + e_consume` nanoseconds (what the old conflated `HandE` reported).
    pub fn e_total_ns(&self) -> u64 {
        self.hand_e_deliver.ns.saturating_add(self.e_consume.ns)
    }

    /// Compute the floor decomposition directly against a measured decode wall time.
    pub fn floor_summary(&self, wall_ns: u64, workers: usize) -> FloorSummary {
        let w = workers.max(1);
        let wall_ms = wall_ns as f64 / 1_000_000.0;
        let hand_s_ms = self.hand_s.ms();
        let worker_handover_ms_per_thread =
            self.worker_handover_ns() as f64 / 1_000_000.0 / w as f64;
        let e_consume_ms_per_thread = self.e_consume.ms_per_thread(w);
        let implied_residual_b_ms =
            (wall_ms - worker_handover_ms_per_thread - e_consume_ms_per_thread).max(0.0);
        FloorSummary {
            wall_ms,
            workers: w,
            hand_s_ms,
            worker_handover_ms_per_thread,
            e_consume_ms_per_thread,
            delivery_copy_ms: self.delivery_copy.ms(),
            delivery_discard_ms: self.delivery_discard.ms(),
            implied_residual_b_ms,
            total_handover_copied_bytes: self.total_handover_copied_bytes(),
        }
    }
}

/// Reset all accounting counters to zero.
pub fn reset() {
    for p in &PHASES {
        p.reset();
    }
}

/// Take all accounting counters and reset them to zero.
pub fn take() -> FloorAccounting {
    FloorAccounting {
        hand_s: PHASES[Phase::HandS as usize].take(),
        hand_m: PHASES[Phase::HandM as usize].take(),
        hand_c: PHASES[Phase::HandC as usize].take(),
        hand_e_deliver: PHASES[Phase::HandEDeliver as usize].take(),
        e_consume: PHASES[Phase::EConsume as usize].take(),
        hand_r: PHASES[Phase::HandR as usize].take(),
        delivery_copy: PHASES[Phase::DeliveryCopy as usize].take(),
        delivery_discard: PHASES[Phase::DeliveryDiscard as usize].take(),
    }
}

#[inline]
pub(crate) fn record(phase: Phase, elapsed_ns: u64, bytes: u64) {
    let slot = &PHASES[phase as usize];
    slot.ns.fetch_add(elapsed_ns, Ordering::Relaxed);
    slot.calls.fetch_add(1, Ordering::Relaxed);
    if bytes > 0 {
        slot.bytes.fetch_add(bytes, Ordering::Relaxed);
    }
}

/// RAII scope that times a single phase only when [`enabled`] is true.
#[allow(dead_code)]
pub(crate) struct Scope {
    phase: Phase,
    start: Option<Instant>,
    bytes: u64,
    active: bool,
}

#[allow(dead_code)]
impl Scope {
    #[inline]
    pub(crate) fn begin(phase: Phase) -> Self {
        let on = enabled();
        Self {
            phase,
            start: on.then(Instant::now),
            bytes: 0,
            active: on,
        }
    }

    #[inline]
    pub(crate) fn set_bytes(&mut self, bytes: u64) {
        self.bytes = bytes;
    }

    /// Cancel recording (e.g. on an oracle cache miss that falls back to real execution).
    #[inline]
    pub(crate) fn cancel(&mut self) {
        self.active = false;
    }
}

impl Drop for Scope {
    #[inline]
    fn drop(&mut self) {
        if self.active {
            if let Some(t0) = self.start {
                let ns = t0.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
                record(self.phase, ns, self.bytes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_summary_arithmetic() {
        let a = FloorAccounting {
            hand_s: PhaseStats {
                ns: 2_000_000,
                calls: 10,
                bytes: 0,
            },
            hand_m: PhaseStats {
                ns: 1_000_000,
                calls: 10,
                bytes: 0,
            },
            hand_c: PhaseStats {
                ns: 1_000_000,
                calls: 10,
                bytes: 0,
            },
            hand_e_deliver: PhaseStats {
                ns: 2_000_000,
                calls: 10,
                bytes: 0,
            },
            e_consume: PhaseStats {
                ns: 40_000_000,
                calls: 10,
                bytes: 1_000_000,
            },
            hand_r: PhaseStats {
                ns: 4_000_000,
                calls: 10,
                bytes: 0,
            },
            delivery_copy: PhaseStats::default(),
            delivery_discard: PhaseStats::default(),
        };
        assert_eq!(a.worker_handover_ns(), 8_000_000);
        assert_eq!(a.total_handover_copied_bytes(), 0);
        assert_eq!(a.e_total_ns(), 42_000_000);
        let s = a.floor_summary(20_000_000, 4);
        assert!((s.wall_ms - 20.0).abs() < 1e-9);
        assert!((s.worker_handover_ms_per_thread - 2.0).abs() < 1e-9);
        assert!((s.e_consume_ms_per_thread - 10.0).abs() < 1e-9);
        assert!((s.implied_residual_b_ms - 8.0).abs() < 1e-9);
    }
}
