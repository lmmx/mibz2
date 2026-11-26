//! Derive the marker section from the original column, eliminate its open complement.
#[allow(dead_code)]
#[path = "kernel.rs"]
mod kernel;
use crate::bwt::{InverseBwt, Segmented};
use crate::{runs::Runs, Kernels};
const PATTERN: &[u8] = br#"":""#;

#[derive(Default)]
pub struct Candidate {
    inverse: kernel::Kernel,
    fallback: Segmented,
}
impl InverseBwt for Candidate {
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
        let caps = runs.records();
        if caps.len() * 20 < 3 * n && caps.len() * 64 >= n {
            if let Some(s) = kernel::Kernel::section(caps, PATTERN) {
                self.inverse.run::<true, true, false>(caps, n, p, s, out);
                return true;
            }
        }
        self.fallback.invert_runs(runs, n, p, out, k)
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(l, c, p, out)
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(l, c, p)
    }
}
#[cfg(test)]
#[path = "checks.rs"]
mod checks;
