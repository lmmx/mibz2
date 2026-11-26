//! Copy ordinary return words and parse only cylinder-certified dangerous bytes.
#[allow(dead_code)]
#[path = "open.rs"]
mod kernel;
#[path = "section.rs"]
mod section;
use crate::bwt::InverseBwt;
use crate::{runs::Runs, Kernels};
use mibz2::ErrorKind;

pub struct Candidate {
    inverse: kernel::Kernel,
    hazards: section::Hazards,
    other: crate::arms::OpenGated,
}
impl Default for Candidate {
    fn default() -> Self {
        Self {
            inverse: kernel::Kernel::default(),
            hazards: section::Hazards::new(),
            other: Default::default(),
        }
    }
}
impl InverseBwt for Candidate {
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.other.invert(l, c, p, out)
    }
    fn wants_checkpoints(&self) -> bool {
        self.other.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        c: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.other.invert_checkpointed(l, c, checkpoints, p, out)
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.other.invert_in_place(l, c, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        self.other.invert_runs(r, n, p, out, k)
    }
    fn expand_runs(
        &mut self,
        r: &Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        _: Kernels,
    ) -> Option<Result<(), ErrorKind>> {
        let caps = r.records();
        if !crate::arms::under_density(caps.len(), n) || !self.hazards.build(caps, 1024) {
            return None;
        }
        Some(self.inverse.expanded(caps, n, p, &self.hazards, out))
    }
}
