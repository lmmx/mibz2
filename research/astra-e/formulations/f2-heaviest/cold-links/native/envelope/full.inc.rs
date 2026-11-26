impl<const MODE: u8, const SCHEDULE: u8> Dispatch<MODE, SCHEDULE> {
    pub fn complete(&mut self, r: &Runs, n: u32, p: u32, out: &mut Vec<u8>) -> u64 {
        let t = crate::now();
        let (quote, m) = self
            .prepare_counted(r.records(), n, r.byte_counts())
            .unwrap();
        let (row, skip) = self.transfer_mark(p, quote).unwrap();
        self.build(quote, m);
        self.inverse_merge();
        let root = &self.merged[self.nodes[0].out as usize..];
        assert!(self.second.invert(
            root,
            &self.nodes,
            &self.words,
            &self.dict,
            row,
            skip,
            m,
            n,
            out
        ));
        std::hint::black_box(&out);
        crate::now() - t
    }
}
