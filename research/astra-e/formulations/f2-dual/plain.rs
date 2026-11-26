// Added only to the stage comparison executable. The live source has its own cache.
impl Candidate {
    fn plain_bands(&mut self, section: u32, m: u32) -> bool {
        self.links.resize(m as usize, 0);
        self.offsets.clear();
        self.offsets.push(0);
        self.dictionary.clear();
        let bits = u32::BITS - (m - 1).leading_zeros();
        let limit = section + m;
        let mut root = section;
        while root < limit {
            let mut right = limit - root;
            let mut x = root;
            let start = self.dictionary.len();
            let dest = loop {
                let i = self.locate(x);
                let r = self.runs[i];
                right = right.min(self.runs[i + 1].start - x);
                self.dictionary.push(r.byte);
                let mapped = r.lf + (x - r.start);
                if (section..limit).contains(&mapped) {
                    right = right.min(limit - mapped);
                    break mapped - section;
                } else if mapped < section {
                    right = right.min(section - mapped);
                }
                x = mapped;
            };
            self.dictionary[start..].reverse();
            let code = self.offsets.len() as u32 - 1;
            if code > u32::MAX >> bits {
                return false;
            }
            for (i, e) in self.links[dest as usize..(dest + right) as usize]
                .iter_mut()
                .enumerate()
            {
                *e = (code << bits) | (root - section + i as u32);
            }
            self.offsets.push(self.dictionary.len() as u32);
            root += right;
        }
        true
    }

    pub fn measure(
        &mut self,
        cache: bool,
        r: &Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> [u64; 4] {
        let t0 = crate::thread_cpu_ns();
        let (section, m) = self.prepare(r.records(), n as u32, PATTERN).unwrap();
        let (q, skip) = self.transfer_mark(p, section, m).unwrap();
        let t1 = crate::thread_cpu_ns();
        let ok = if cache {
            self.bands(section, m)
        } else {
            self.plain_bands(section, m)
        };
        assert!(ok);
        let t2 = crate::thread_cpu_ns();
        self.expand(q, skip, m, n, out);
        std::hint::black_box(&out);
        let t3 = crate::thread_cpu_ns();
        [t1 - t0, t2 - t1, 0, t3 - t2]
    }
}
