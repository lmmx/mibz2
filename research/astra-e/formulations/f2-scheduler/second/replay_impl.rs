impl Candidate {
    pub fn preload(&mut self,r:&Runs,n:usize,p:u32)->(u32,u32,u32) {
        let (section,m)=self.prepare(r.records(),n as u32).unwrap();
        let (row,skip)=self.transfer_mark(p,section).unwrap();
        self.build(section,m);
        self.inverse_merge();
        (row,skip,m)
    }
    pub fn replay(&mut self, state:(u32,u32,u32), n:usize,out:&mut Vec<u8>)->[u64;2] {
        let (row,skip,m)=state;
        let root=&self.merged[self.nodes[0].out as usize..];
        let t0=crate::thread_cpu_ns();
        assert!(self.second.invert(root,&self.nodes,&self.words,&self.dict,row,skip,m,n as u32,out));
        std::hint::black_box(&out);
        [self.second.discovery_ns,crate::thread_cpu_ns()-t0]
    }
}
