//! Combined narrow/short/ahead endpoint with sparse RLE1 repair in an empty output slot.
#[allow(dead_code)]
#[path = "../narrow-short/ahead.rs"]
mod base;
#[path = "repair.rs"]
mod repair;
use crate::bwt::InverseBwt;
use crate::{runs::Runs, Kernels};
use mibz2::ErrorKind;

#[derive(Default)]
pub struct FinalSlot<const PROBE: bool> {
    base: base::Candidate,
    edits: repair::Directory,
    spare: Vec<u8>,
}
pub type Candidate = FinalSlot<false>;

impl<const PROBE: bool> InverseBwt for FinalSlot<PROBE> {
    fn invert(&mut self, l: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.base.invert(l, counts, p, out)
    }
    fn wants_checkpoints(&self) -> bool {
        self.base.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        counts: &[u32; 256],
        cp: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.base.invert_checkpointed(l, counts, cp, p, out)
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.base.invert_in_place(l, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        self.base.invert_runs(r, n, p, out, k)
    }
    fn expand_runs(
        &mut self,
        r: &Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> Option<Result<(), ErrorKind>> {
        if !out.is_empty()
            || !k.avx2()
            || !cfg!(target_arch = "x86_64")
            || n >= 1 << 20
            || !crate::arms::under_density(r.records().len(), n)
        {
            return None;
        }
        // Every admitted constructor returns the full marked RLE1 text in out.
        // A small initial allowance covers a block whose encoded size is just
        // below its final size. Larger expansions use the ordinary RLE kernel.
        out.reserve(n + 256);
        let done = self.base.invert_runs(r, n, p, out, k);
        debug_assert!(done && out.len() == n);
        let mut costs = repair::Costs::default();
        let outcome = self.edits.apply::<PROBE>(out, k, &mut costs);
        let strategy = match outcome {
            Ok(strategy) => strategy,
            Err(err) => return Some(Err(err)),
        };
        let result = if strategy == repair::Strategy::Applied {
            Ok(())
        } else {
            // Planning does not alter the encoded bytes on fallback. Transfer
            // ownership to retained scratch without copying or a second inverse.
            std::mem::swap(&mut self.spare, out);
            out.clear();
            mibz2::rle::unrle(&self.spare, out, k)
        };
        if PROBE {
            eprintln!(
                "slot,{n},{p},{strategy:?},{},{},{},{},{},{},{},{}",
                self.edits.len(),
                costs.scanned,
                costs.left,
                costs.right,
                costs.filled,
                out.len(),
                self.edits.capacity_bytes(),
                self.spare.capacity()
            );
        }
        Some(result)
    }
}
