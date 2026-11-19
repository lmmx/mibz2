//! A candidate kernel: copy this file, rename `Template`, and build with
//! `just moonshot-score path/to/your_file.rs`.
//!
//! The contract is the shipped workers' block-decode entry, `mibz2::bwt::InverseBwt`: a block's
//! RLE1-coded text from its last column (or from its capped run records), exactly as
//! LazyCells gives it. The decoder around it (Huffman, MTF, RLE1, CRC, the workers and the
//! window) is mibz2's own, so the candidate is timed in the very workers 0.5.6 runs
//! (`BzReader::parallel_workers_with::<Candidate>`).
//!
//! This file is compiled as the module `crate::candidate`. `crate::bwt`, `crate::runs` and
//! `crate::Kernels` are mibz2's, so code written inside mibz2 (`use crate::bwt::...`)
//! compiles here unchanged; a sibling file can be a submodule (`#[path = "x.rs"] mod x;`).
//! The run records are `r.records()` (capped runs, `byte << 8 | length`, in L order).
use crate::bwt::{InverseBwt, LazyCells};
use crate::runs::Runs;
use crate::Kernels;

/// The scorer reads this name
pub type Candidate = Template;

/// Here: LazyCells on every route, so this template scores as the null candidate
#[derive(Default)]
pub struct Template {
    shipped: LazyCells,
}

impl InverseBwt for Template {
    /// The column's route: `ll` the last column, `counts` its bytes' counts, `orig_ptr` the
    /// primary index; the RLE1-coded text into `out` (cleared first)
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        self.shipped.invert(ll, counts, orig_ptr, out)
    }

    /// Optional: the output written over `ll` (see the trait's documentation); false to decline
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], orig_ptr: u32) -> bool {
        self.shipped.invert_in_place(ll, counts, orig_ptr)
    }

    /// True to be offered the blocks of few symbols by their run records (`invert_runs`)
    fn reads_runs(&self) -> bool {
        true
    }

    /// The run records' route: `r.records()` tile `n` rows; false only for a broken tiling
    fn invert_runs(
        &mut self,
        r: &Runs,
        n: usize,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> bool {
        self.shipped.invert_runs(r, n, orig_ptr, out, k)
    }
}
