use crate::{bwt::{InverseBwt, LazyCells}, runs::Runs, Kernels};

#[derive(Default)]
pub struct Candidate {
    open: kernel::Kernel,
    lazy: LazyCells,
}

impl InverseBwt for Candidate {
    fn invert(&mut self, l: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(l, counts, p, out)
    }
    fn wants_checkpoints(&self) -> bool { self.lazy.wants_checkpoints() }
    fn invert_checkpointed(&mut self, l: &[u8], counts: &[u32; 256], cp: &[[u32; 256]], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert_checkpointed(l, counts, cp, p, out)
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(l, counts, p)
    }
    fn reads_runs(&self) -> bool { true }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if r.records().len() * 20 >= n * 3 && (n < 65_536 || !fits(r.records(), p)) {
            return self.lazy.invert_runs(r, n, p, out, k);
        }
        self.open.run::<true, true, false>(r.records(), n, p, 16, out);
        true
    }
}
