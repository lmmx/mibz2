//! The reference arms beside the candidate, each an `InverseBwt` the shipped workers run:
//!
//! - `lazy`: LazyCells, 0.5.6's kernel on its own admission (the baseline);
//! - `cells`: eager Cells (the i9's `oce` arms: `parallel_workers(.., false)`);
//! - `open`: Astra's open decoder at width 16 on every run-route block (the i9's `open` arms);
//! - `opend`: that decoder under LazyCells' density line, LazyCells over it (the i9's `opend`).
//!
//! `open` and `opend` are the open-return comparison's arms (bench/open-return,
//! src/open_gate.rs there), compiled here from the journal unchanged.
use crate::bwt::{InverseBwt, LazyCells};
use crate::runs::Runs;
use crate::Kernels;

/// Astra's queued open endpoint at width 16, Segmented on the column's route
pub type Open = crate::open_return::Open<16, true, true>;

/// Whether a block of `records` capped run records in `n` rows is under LazyCells' density
/// line (`cells_lazy.rs`' `LAZY_DENSITY`, 3 records in 20 rows)
pub fn under_density(records: usize, n: usize) -> bool {
    records * 20 < n * 3
}

/// `open` under LazyCells' density line, LazyCells as shipped over it and on the column's
/// route: bench/open-return's `open_gate::OpenGated`, line for line
#[derive(Default)]
pub struct OpenGated {
    open: Open,
    lazy: LazyCells,
}

impl InverseBwt for OpenGated {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(ll, counts, p, out)
    }
    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.lazy
            .invert_checkpointed(ll, counts, checkpoints, p, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(ll, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if under_density(r.records().len(), n) {
            self.open.invert_runs(r, n, p, out, k)
        } else {
            self.lazy.invert_runs(r, n, p, out, k)
        }
    }
}

/// The arms by name, the candidate's included
pub const ARMS: [&str; 5] = ["lazy", "cells", "open", "opend", "candidate"];

/// `$f::<K>($args)` for the arm named `$arm` (one of `ARMS`), else `$else`
#[macro_export]
macro_rules! by_arm {
    ($arm:expr, $f:ident ( $($a:expr),* ), $else:expr) => {
        match $arm {
            "lazy" => $f::<mibz2::bwt::LazyCells>($($a),*),
            "cells" => $f::<mibz2::bwt::Cells>($($a),*),
            "open" => $f::<$crate::arms::Open>($($a),*),
            "opend" => $f::<$crate::arms::OpenGated>($($a),*),
            "candidate" => $f::<$crate::candidate::Candidate>($($a),*),
            _ => $else,
        }
    };
}
