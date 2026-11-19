// Compiled after candidate.rs in a scratch module without test counters.
impl<const HOT_AT: u32> Mixed<HOT_AT> {
    pub fn measure(
        &mut self,
        r: &Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        two: bool,
    ) -> [u64; 4] {
        let t0 = crate::thread_cpu_ns();
        let (quote, m) = self.prepare(r.records(), n as u32).unwrap();
        let (q, skip) = self.transfer_mark(p, quote).unwrap();
        let t1 = crate::thread_cpu_ns();
        self.build(quote, m);
        let t2 = crate::thread_cpu_ns();
        self.inverse_merge();
        let t3 = crate::thread_cpu_ns();
        if two {
            let root = &self.merged[self.nodes[0].out as usize..];
            assert!(self.second.invert::<HOT_AT>(
                root,
                &self.nodes,
                &mut self.words,
                &self.edges,
                q,
                skip,
                m,
                n as u32,
                out
            ));
        } else {
            self.materialize_first();
            assert!(self.expand(q, skip, m, n as u32, out));
        }
        std::hint::black_box(&out);
        let t4 = crate::thread_cpu_ns();
        [t1 - t0, t2 - t1, t3 - t2, t4 - t3]
    }
}
