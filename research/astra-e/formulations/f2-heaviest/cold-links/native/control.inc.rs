IMPL_CONTROL {
    pub fn control(&mut self,r:&Runs,n:u32,p:u32)->Option<[u64;6]> {
        let t0=crate::now();
        let (quote,m)=PREPARE?;
        let mark=self.transfer_mark(p,quote)?;
        let t1=crate::now();
        self.build(quote,m);
        let t2=crate::now();
        self.inverse_merge();
        std::hint::black_box((&self.merged,&self.words,&self.dict,mark));
        let t3=crate::now();
        Some([t1-t0,t2-t1,t3-t2,0,0,0])
    }
    pub fn stage_memory(&self)->usize {
        crate::cap(&self.runs)+crate::cap(&self.starts)+crate::cap(&self.page_first)
         +crate::cap(&self.edges)+crate::cap(&self.words)+crate::cap(&self.dict)
         +crate::cap(&self.selectors)+crate::cap(&self.nodes)+crate::cap(&self.merged)
         +self.lane_paths.iter().map(crate::cap).sum::<usize>()
    }
}
