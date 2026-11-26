
impl Kernel {
    // Sizes refer to initialized, walk-readable storage, independently of allocator capacity.
    pub fn inspect(&mut self, records: &[u16], n: usize, p: u32, limit: u32, out: &mut Vec<u8>)
        -> (Counts, [usize; 11])
    {
        let (c, mark, bits) = self.construct::<true, true>(records, n as u32, p, limit);
        emit(&self.edges, &self.extra, &self.spelling, &self.offsets, n, mark, bits, out);
        let mask = (1 << bits) - 1;
        let mut row = mark;
        let mut visits = 0;
        let mut dictionary = vec![0u8; c.words];
        loop {
            let edge = self.edges[row as usize];
            let code = if self.extra.is_empty() { edge >> bits } else { self.extra[row as usize] };
            if code >= 256 {
                let v = &mut dictionary[(code - 256) as usize];
                *v = (*v+1).min(2);
            }
            visits += 1;
            row = edge & mask;
            if row == mark {break;}
            assert!(visits < c.retained);
        }
        let mut offsets = vec![false; self.offsets.len()];
        let mut spell = 0;
        for (i, &touched) in dictionary.iter().enumerate() {
            if touched != 0 {
                offsets[i] = true;
                offsets[i+1] = true;
                spell += (self.offsets[i+1] - self.offsets[i]) as usize;
            }
        }
        assert_eq!(self.spelling.len(), c.traced + c.words);
        assert_eq!(self.offsets.len(), c.words + 1);
        // Count distinct expansion events that touch each modeled cache line. The
        // frequencies saturate at two. Both offset endpoints can lie on the same line.
        let recurrent = |alignment: usize| {
            let mut spell_lines = vec![0u8; (alignment+self.spelling.len()).div_ceil(64)];
            let mut offset_lines = vec![0u8; (alignment+4*self.offsets.len()).div_ceil(64)];
            for (i, &v) in dictionary.iter().enumerate() {
                if v==0 {continue;}
                let start = (alignment + self.offsets[i] as usize) / 64;
                let end = (alignment + self.offsets[i+1] as usize - 1) / 64;
                for line in &mut spell_lines[start..=end] {*line=(*line+v).min(2);}
                let start = (alignment + 4*i) / 64;
                let end = (alignment + 4*(i+1)) / 64;
                for line in &mut offset_lines[start..=end] {*line=(*line+v).min(2);}
            }
            (64*spell_lines.iter().filter(|v| **v==2).count(),
             64*offset_lines.iter().filter(|v| **v==2).count())
        };
        let aligned=recurrent(0);
        let shifted=recurrent(32);
        let live = [self.edges.len()*4+self.extra.len()*4, self.spelling.len(),
                    self.offsets.len()*4, visits, spell, offsets.iter().filter(|x| **x).count()*4,
                    self.spelling.capacity() + 4*(self.offsets.capacity()+self.edges.capacity()+self.extra.capacity()),
                    aligned.0, aligned.1, shifted.0, shifted.1];
        (c, live)
    }
}
