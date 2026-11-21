//! Gated Open-Return with inline endpoint words of length two and three.
#[allow(dead_code)]
#[path = "constructor.rs"]
mod kernel;
use crate::{bwt::{InverseBwt, LazyCells}, runs::Runs, Kernels};

#[derive(Default)]
pub struct ShortWords<const PROBE: bool> {
    open: kernel::Kernel,
    lazy: LazyCells,
}

pub type Candidate = ShortWords<false>;

impl<const PROBE: bool> InverseBwt for ShortWords<PROBE> {
    fn invert(&mut self, ll: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(ll,c,p,out)
    }
    fn wants_checkpoints(&self) -> bool { self.lazy.wants_checkpoints() }
    fn invert_checkpointed(&mut self, ll: &[u8], c: &[u32; 256], cp: &[[u32; 256]], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert_checkpointed(ll,c,cp,p,out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(ll,c,p)
    }
    fn reads_runs(&self) -> bool { true }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if r.records().len() * 20 >= n * 3 { return self.lazy.invert_runs(r,n,p,out,k); }
        let s = self.open.run::<true,true,false>(r.records(),n,p,16,out);
        if PROBE { eprintln!("words,{n},{p},{},{},{},{}", s.steps,s.rules,s.traced,s.retained); }
        true
    }
}
