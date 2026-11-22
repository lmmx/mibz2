//! Research only (`emit-cost`): the open kernel's two stages timed block by block in the
//! decoder's own workers, for examples/emit_cost.rs and the bench-emit-cost diagnostic
//! (docs/journal/2026-11-22-emit-cost-registration.md). Each block the open kernel takes
//! (`OpenCells`' `Kernel::try_run`) records its rows `n`, its retained rows `m` (at
//! `open::LIMIT`, 16), the construction's wall time and the emit's (`walk`); a block whose
//! first pass refuses it (past `Cap::dense`) records the refused pass's time with `m` 0.
//! Without the feature none of this is compiled and the kernel is as shipped.
use std::sync::Mutex;
use std::time::Duration;

/// One block's record
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rec {
    /// the block's rows
    pub n: u32,
    /// its retained rows (the walk's steps); 0 for a block the first pass refused
    pub m: u32,
    /// the construction (`construct_capped`), ns
    pub construct_ns: u64,
    /// the emit (`walk`), ns; 0 for a refused block
    pub emit_ns: u64,
    /// the two-ended emitter ran (`emit_bidirectional`), else the forward one
    pub bidi: bool,
    /// the first pass refused the block (it went to `LazyCells`)
    pub refused: bool,
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
        // (other tests' blocks may record alongside, under `--all-features`: this one's by its n)
        let recs: Vec<Rec> = take().into_iter().filter(|r| r.n as usize == n).collect();
        assert_eq!(recs.len(), 1, "{recs:?}");
        let rec = recs[0];
        assert_eq!(rec.n as usize, n);
        assert!(!rec.refused && rec.m > 0 && (rec.m as usize) < n, "{rec:?}");
        assert_eq!(out, want);
    }
}
