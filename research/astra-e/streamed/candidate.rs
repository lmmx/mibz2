//! Use bounded RLE1 input chunks when opend selects its queued endpoint constructor.
#[allow(dead_code)]
#[path = "constructor.rs"]
mod kernel;
use crate::bwt::InverseBwt;
use crate::{runs::Runs, Kernels};
use mibz2::ErrorKind;

#[derive(Default)]
pub struct Candidate {
    streamed: kernel::Kernel,
    chunk: Vec<u8>,
    fallback: crate::arms::OpenGated,
}

impl InverseBwt for Candidate {
    fn invert(&mut self, l: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(l, counts, p, out)
    }
    fn wants_checkpoints(&self) -> bool {
        self.fallback.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.fallback
            .invert_checkpointed(l, counts, checkpoints, p, out)
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(l, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        self.fallback.invert_runs(r, n, p, out, k)
    }
    fn expand_runs(
        &mut self,
        r: &Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> Option<Result<(), ErrorKind>> {
        if !crate::arms::under_density(r.records().len(), n) {
            return None;
        }
        Some(self.streamed.expand(r.records(), n, p, &mut self.chunk, out, k))
    }
}
