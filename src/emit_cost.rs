//! Research only (`emit-cost`): the open kernel's stages timed block by block in the decoder's
//! own workers, for examples/emit_cost_sq.rs and the bench-emit-cost-sq diagnostic
//! (docs/journal/2026-11-23-emit-cost-sq-registration.md; the recorder is bench-emit-cost's,
//! docs/journal/2026-11-22-emit-cost-registration.md, ported onto master). Each block the open
//! kernel takes (`OpenCells`' `Kernel::try_run`) records its rows `n`, its retained rows `m`
//! (at `open::LIMIT`, 16), the construction's wall time, the square pass's (`squared-trace`'s
//! `sq` arm; 0 otherwise) and the emit's (`walk`); a block whose first pass refuses it (past
//! `Cap::dense`) records the refused pass's time with `m` 0. Four clock reads a block, in every
//! arm. Without the feature none of this is compiled and the kernel is as shipped.
//!
//! **The arm** (`squared-trace`, MONODROMY record 93): a process-global switch the harness sets
//! between decodes, read once a block at its start. `Base` is the kernel as shipped; `Sq` and
//! `SqShadow` change only the blocks the kernel already emits from both ends
//! (`QUEUE && !wide && m >= 65,536`), and write the same bytes (`open/squared.rs`).
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::Duration;

/// Which emitter the open kernel's two-ended blocks use
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Arm {
    /// the kernel as shipped: `emit_bidirectional`
    Base = 0,
    /// record 93 §2: `succ` recorded in construction, the serial square pass (its own stamp),
    /// then `emit_bidirectional_sq`
    Sq = 1,
    /// record 93 §5 and its addendum: `succ` recorded in construction, the square pass
    /// interleaved into the base two-ended walk (4 iterations a step), then
    /// `emit_bidirectional_sq` from the walk's exact state once it completes
    SqShadow = 2,
}

impl Arm {
    /// The arm's name in the harness's arguments and records
    pub fn name(self) -> &'static str {
        match self {
            Arm::Base => "base",
            Arm::Sq => "sq",
            Arm::SqShadow => "sq-shadow",
        }
    }

    /// The arm named `s` (`base`, `sq` or `sq-shadow`)
    pub fn parse(s: &str) -> Option<Arm> {
        [Arm::Base, Arm::Sq, Arm::SqShadow]
            .into_iter()
            .find(|a| a.name() == s)
    }

    pub(crate) fn of(v: u8) -> Arm {
        match v {
            1 => Arm::Sq,
            2 => Arm::SqShadow,
            _ => Arm::Base,
        }
    }
}

static ARM: AtomicU8 = AtomicU8::new(Arm::Base as u8);

/// Switch the arm for the blocks the kernel starts from now on (between decodes: a decode's
/// workers are spawned after it, so they see it)
pub fn set_arm(a: Arm) {
    ARM.store(a as u8, Ordering::SeqCst);
}

/// The arm now set (always `Base` without `squared-trace`, whose emitters the others need)
pub fn arm() -> Arm {
    if cfg!(feature = "squared-trace") {
        Arm::of(ARM.load(Ordering::Relaxed))
    } else {
        Arm::Base
    }
}

/// One block's record
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rec {
    /// the block's rows
    pub n: u32,
    /// its retained rows (the walk's steps); 0 for a block the first pass refused
    pub m: u32,
    /// the construction (`construct_capped`, with `succ`'s stores in the sq arms), ns
    pub construct_ns: u64,
    /// the serial square pass (`Sq` on a two-ended block), ns; otherwise the two adjacent clock
    /// reads' gap
    pub square_ns: u64,
    /// the emit (`walk`), ns; 0 for a refused block
    pub emit_ns: u64,
    /// the two-ended emitter's block (`QUEUE && !wide && m >= 65,536`), else the forward one
    pub bidi: bool,
    /// the first pass refused the block (it went to `LazyCells`)
    pub refused: bool,
    /// the arm the block ran under
    pub arm: Arm,
    /// the T² walk ran on this block (`emit_bidirectional_sq` took at least one round)
    pub sq_rounds: bool,
    /// the arm's own scratch (`succ` and `x2`) held by this worker's kernel after the block,
    /// bytes (their capacity); 0 in `Base`
    pub scratch: u64,
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
    use crate::bwt::{ForcedOpenCells, InverseBwt, Plain};
    use crate::runs::Runs;

    /// A block the open kernel takes records one entry, with its `n`, a retained count
    /// within it, and the output the plain inverse's
    #[test]
    fn records_each_open_block() {
        let mut l = Vec::new();
        for i in 0..4000u32 {
            l.extend(std::iter::repeat_n(
                b'a' + (i % 7) as u8,
                40 + (i % 13) as usize,
            ));
        }
        let n = l.len();
        let p = (n / 3) as u32;
        let mut counts = [0u32; 256];
        for &c in &l {
            counts[c as usize] += 1;
        }
        let mut want = Vec::new();
        Plain::default().invert(&l, &counts, p, &mut want);
        let mut out = Vec::new();
        let k = crate::Kernels::detect();
        assert!(ForcedOpenCells::default().invert_runs(&Runs::of_column(&l), n, p, &mut out, k));
        // (other tests' blocks may record alongside: this one's by its n)
        let recs: Vec<Rec> = take().into_iter().filter(|r| r.n as usize == n).collect();
        assert_eq!(recs.len(), 1, "{recs:?}");
        let rec = recs[0];
        assert_eq!(rec.n as usize, n);
        assert!(!rec.refused && rec.m > 0 && (rec.m as usize) < n, "{rec:?}");
        assert_eq!(out, want);
    }

    #[test]
    fn arm_names() {
        for a in [Arm::Base, Arm::Sq, Arm::SqShadow] {
            assert_eq!(Arm::parse(a.name()), Some(a));
            assert_eq!(Arm::of(a as u8), a);
        }
        assert_eq!(Arm::parse("t2"), None);
    }
}
