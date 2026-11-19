
impl Kernel {
    // Sizes refer to initialized, walk-readable storage, independently of allocator capacity.
    pub fn inspect(&mut self, records: &[u16], n: usize, p: u32, limit: u32, out: &mut Vec<u8>)
        -> (Counts, [usize; 7])
    {
        let (c, mark, bits) = self.construct::<true, true>(records, n as u32, p, limit);
        emit(&self.edges, &self.extra, &self.spelling, &self.offsets, n, mark, bits, out);
        let mask = (1 << bits) - 1;
        let mut row = mark;
        let mut visits = 0;
        let mut dictionary = vec![false; c.words];
        loop {
            let edge = self.edges[row as usize];
            let code = if self.extra.is_empty() { edge >> bits } else { self.extra[row as usize] };
            if code >= 256 { dictionary[(code - 256) as usize] = true; }
            visits += 1;
            row = edge & mask;
            if row == mark {break;}
            assert!(visits < c.retained);
        }
        let mut offsets = vec![false; self.offsets.len()];
        let mut spell = 0;
        for (i, &touched) in dictionary.iter().enumerate() {
            if touched {
                offsets[i] = true;
                offsets[i+1] = true;
                spell += (self.offsets[i+1] - self.offsets[i]) as usize;
            }
        }
        assert_eq!(self.spelling.len(), c.traced + c.words);
        assert_eq!(self.offsets.len(), c.words + 1);
        let live = [self.edges.len()*4+self.extra.len()*4, self.spelling.len(),
                    self.offsets.len()*4, visits, spell, offsets.iter().filter(|x| **x).count()*4,
                    self.spelling.capacity() + 4*(self.offsets.capacity()+self.edges.capacity()+self.extra.capacity())];
        (c, live)
    }
}
