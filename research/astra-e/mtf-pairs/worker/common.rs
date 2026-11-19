use crate::{bwt::{InverseBwt,LazyCells},runs::Runs as StockRuns,Kernels};
include!("gate.rs");

struct Runs{
    runs:Vec<u16>,
    counts:[u32;256],
    per:[u32;256],
    lit:Vec<u64>,
    one:Vec<u64>,
}
impl Default for Runs {
    fn default()->Self{
        Self{runs:Vec::new(),counts:[0;256],per:[0;256],lit:Vec::new(),one:Vec::new()}
    }
}
#[derive(Default)]
pub struct Candidate{
    open:kernel::Kernel,
    lazy:LazyCells,
    front:Runs,
}
impl InverseBwt for Candidate {
    fn invert(&mut self,l:&[u8],counts:&[u32;256],p:u32,out:&mut Vec<u8>){self.lazy.invert(l,counts,p,out);}
    fn wants_checkpoints(&self)->bool{self.lazy.wants_checkpoints()}
    fn invert_checkpointed(&mut self,l:&[u8],counts:&[u32;256],cp:&[[u32;256]],p:u32,out:&mut Vec<u8>){
        self.lazy.invert_checkpointed(l,counts,cp,p,out);
    }
    fn invert_in_place(&mut self,l:&mut Vec<u8>,counts:&[u32;256],p:u32)->bool{self.lazy.invert_in_place(l,counts,p)}
    fn reads_runs(&self)->bool{true}
    fn invert_symbol_runs(&mut self,b:&mibz2::block::Symbols,k:Kernels,out:&mut Vec<u8>)->bool{
        #[cfg(target_arch="x86_64")]
        if b.syms.len()>=64 && k.avx2 && is_x86_feature_detected!("bmi2") && is_x86_feature_detected!("lzcnt") && is_x86_feature_detected!("popcnt"){
            // This CPU has exactly the complete stock front's required features.
            if let Some(n)=unsafe{accelerated::front_avx2(b,&mut self.front)} {
                let r=&self.front.runs;
                if (b.orig_ptr as usize)<n && (r.len()*20<n*3 || (n>=65_536 && fits(r,b.orig_ptr))) {
                    self.open.run::<true,true,false>(r,n,b.orig_ptr,16,out);
                    return true;
                }
            }
        }
        let _=(b,k,out);
        false
    }
    fn invert_runs(&mut self,r:&StockRuns,n:usize,p:u32,out:&mut Vec<u8>,k:Kernels)->bool{
        if r.records().len()*20>=n*3 && (n<65_536 || !fits(r.records(),p)){return self.lazy.invert_runs(r,n,p,out,k);}
        self.open.run::<true,true,false>(r.records(),n,p,16,out);
        true
    }
}
