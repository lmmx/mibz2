//! OpenGated with dictionary-word anchors on its residual edge table.
//! Run prepare.py first to expose its pinned constructor from the journal.
#[allow(dead_code)]
#[path = "constructor.rs"]
mod kernel;
use crate::bwt::{InverseBwt, LazyCells};
use crate::{runs::Runs, Kernels};

#[derive(Default)]
pub struct AnchorKernel<const PROBE: bool> {
    open: kernel::Kernel,
    lazy: LazyCells,
}
pub type Candidate = AnchorKernel<false>;

impl<const PROBE: bool> InverseBwt for AnchorKernel<PROBE> {
    fn invert(&mut self, l: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(l, counts, p, out)
    }
    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.lazy
            .invert_checkpointed(l, counts, checkpoints, p, out)
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(l, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(
        &mut self,
        runs: &Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> bool {
        if runs.records().len() * 20 < n * 3 {
            self.open.anchored::<PROBE>(runs.records(), n, p, out);
            true
        } else {
            self.lazy.invert_runs(runs, n, p, out, k)
        }
    }
}
