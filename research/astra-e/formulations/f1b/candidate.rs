//! F1b sparse residual construction and macro return, with the opend gate.
use crate::bwt::{InverseBwt, LazyCells};
use crate::{runs::Runs, Kernels};
mod endpoint;
include!("index.rs");
include!("kernel.rs");

#[derive(Default)]
pub struct Candidate {
    open: Kernel,
    map: ResultMap,
    lazy: LazyCells,
}

impl InverseBwt for Candidate {
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
        if r.records().len() * 20 < n * 3 {
            self.open
                .factor::<false>(r.records(), n as u32, p, 16, &mut self.map);
            self.map.emit::<false>(&self.open, n, out);
            true
        } else {
            self.lazy.invert_runs(r, n, p, out, k)
        }
    }
}
