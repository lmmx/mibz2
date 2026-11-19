    fn patch(&mut self, bits: u32, _budget: u32) -> bool {
        self.owners.clear();
        self.labels.clear();
        self.edit_start.clear();
        self.edits.sort_unstable_by_key(|e| (e.rank, e.offset));
        for (i,e) in self.edits.iter().enumerate() {
            if self.owners.last() != Some(&e.rank) {
                self.owners.push(e.rank);
                self.edit_start.push(i as u32);
            }
        }
        self.edit_start.push(self.edits.len() as u32);
        let first = 255 + self.offsets.len() as u32;
        if self.extra.is_empty() && !self.owners.is_empty()
            && first + self.owners.len() as u32 - 1 > u32::MAX >> bits {
            self.reason=3;
            return false;
        }
        for i in 0..self.owners.len() {
            let row=self.owners[i] as usize;
            self.labels.push(self.source_code(row,bits));
            self.set_code(row,bits,first+i as u32);
        }
        true
    }
    fn changed_word(&self, index: usize, out: &mut Vec<u8>) {
        let code = self.labels[index];
        let byte = [code as u8];
        let raw = if code < 256 {
            &byte[..]
        } else {
            let word = (code - 256) as usize;
            &self.spelling[self.offsets[word] as usize..self.offsets[word + 1] as usize]
        };
        let mut from=0;
        for e in &self.edits[self.edit_start[index] as usize..self.edit_start[index+1] as usize] {
            let off=e.offset as usize;
            out.extend_from_slice(&raw[from..off]);
            out.resize(out.len()+usize::from(e.count),e.byte);
            from=off+1;
        }
        out.extend_from_slice(&raw[from..]);
    }
    fn write_once(&self, mark: u32, bits: u32, out: &mut Vec<u8>) {
        let mask=(1 << bits)-1;
        let first=255+self.offsets.len() as u32;
        let mut row=mark;
        let mut edge=self.edges[mark as usize];
        loop {
            let code = if self.extra.is_empty() { edge >> bits } else { self.extra[row as usize] };
            row=edge & mask;
            edge=self.edges[row as usize];
            if code < 256 {
                out.push(code as u8);
            } else if code < first {
                let w=(code-256) as usize;
                out.extend_from_slice(&self.spelling[self.offsets[w] as usize..self.offsets[w+1] as usize]);
            } else {
                self.changed_word((code-first) as usize,out);
            }
            if row==mark { break; }
        }
    }
