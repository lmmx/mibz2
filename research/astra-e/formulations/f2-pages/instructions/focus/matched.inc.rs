impl Dispatch<2> {
    // Every observation of this pair calls this same function. Hardware and
    // software share the state type, output, merge and recursive write code.
    #[inline(never)]
    pub fn matched(&mut self,r:&Runs,n:usize,p:u32,out:&mut Vec<u8>,hw:bool)->[u64;4] {
        let start=crate::thread_cpu_ns();
        let (quote,tokens)=if hw {
            assert!(std::is_x86_feature_detected!("popcnt"));
            // SAFETY: the preceding condition requires this feature.
            unsafe { self.prepare_pop(r.records(),n as u32,r.byte_counts()) }
        } else {
            self.prepare_body(r.records(),n as u32,r.byte_counts())
        }.unwrap();
        let (q,skip)=self.transfer_mark(p,quote).unwrap();
        let prepared=crate::thread_cpu_ns();
        if hw {
            // SAFETY: the same guarded policy was checked above.
            unsafe { self.build_pop(quote,tokens) }
        } else {
            self.build_body(quote,tokens);
        }
        let discovered=crate::thread_cpu_ns();
        self.inverse_merge();
        let merged=crate::thread_cpu_ns();
        let root=&self.merged[self.nodes[0].out as usize..];
        assert!(self.second.invert(root,&self.nodes,&self.words,&self.dict,q,skip,tokens,n as u32,out));
        std::hint::black_box(&out);
        let finish=crate::thread_cpu_ns();
        [prepared-start,discovered-prepared,merged-discovered,finish-merged]
    }
}
