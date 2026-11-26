impl Candidate {
    pub fn measure(
        &mut self,
        input: &Runs,
        n: usize,
        mark: u32,
        out: &mut Vec<u8>,
        k: usize,
    ) -> [u64; 5] {
        let t0 = crate::thread_cpu_ns();
        let (section, count) = self.prepare(input.records(), n as u32, b"\"").unwrap();
        let t1 = crate::thread_cpu_ns();
        let count = self.select(section, count, k);
        let (row, skip) = self.transfer(mark).unwrap();
        let t2 = crate::thread_cpu_ns();
        self.build();
        let t3 = crate::thread_cpu_ns();
        self.inverse_merge();
        let t4 = crate::thread_cpu_ns();
        assert!(self.expand(row, skip, count, n as u32, out));
        std::hint::black_box(&out);
        let t5 = crate::thread_cpu_ns();
        [t1 - t0, t2 - t1, t3 - t2, t4 - t3, t5 - t4]
    }
}
