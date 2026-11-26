impl Candidate {
    pub fn measure(
        &mut self,
        input: &Runs,
        n: usize,
        mark: u32,
        out: &mut Vec<u8>,
        choose: bool,
    ) -> [u64; 5] {
        let t0 = crate::thread_cpu_ns();
        let seed = if choose { &b"\""[..] } else { &b"\":\""[..] };
        let (mut section, mut count) = self.prepare(input.records(), n as u32, seed).unwrap();
        let t1 = crate::thread_cpu_ns();
        if choose {
            for _ in 0..2 {
                (section, count) = self.heaviest(section, count);
            }
        }
        let (row, skip) = self.transfer_mark(mark, section, count).unwrap();
        let t2 = crate::thread_cpu_ns();
        self.build(section, count);
        let t3 = crate::thread_cpu_ns();
        self.inverse_merge();
        let t4 = crate::thread_cpu_ns();
        assert!(self.expand(row, skip, count, n as u32, out));
        std::hint::black_box(&out);
        let t5 = crate::thread_cpu_ns();
        [t1 - t0, t2 - t1, t3 - t2, t4 - t3, t5 - t4]
    }
}
