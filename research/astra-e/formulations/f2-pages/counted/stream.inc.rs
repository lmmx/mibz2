    fn prepare_counted(&mut self, caps: &[u16], n: u32, counts: &[u32; 256]) -> Option<(u32, u32)> {
        let tokens = counts[QUOTE as usize];
        if n == 0 || tokens == 0 { return None; }
        let mut total = 0;
        let mut next = counts.map(|width| { let base = total; total += width; base });
        if total != n { return None; }
        let quote = next[QUOTE as usize];
        let pages = n.div_ceil(64) as usize;
        self.starts.clear();
        self.starts.reserve(pages);
        self.page_first.clear();
        self.page_first.reserve(pages);
        self.runs.clear();

        let (mut page, mut base, mut mask) = (0usize, 0, 0u64);
        let mut previous = None;
        let mut row = 0;
        for &record in caps {
            let byte = (record >> 8) as u8;
            let len = u32::from(record & 255);
            if len == 0 || len > 127 || row + len > n { return None; }
            if previous != Some(byte) {
                let run_id = self.runs.len() as u32;
                let next_page = (row >> 6) as usize;
                let offset = row & 63;
                if next_page != page {
                    self.starts.push(mask & !1);
                    self.page_first.push(base);
                    // Pages strictly between the two run starts contain only the previous run.
                    self.starts.resize(next_page, 0);
                    self.page_first.resize(next_page, run_id-1);
                    base = run_id - u32::from(offset != 0);
                    page = next_page;
                    mask = 0;
                }
                mask |= 1u64 << offset;
                self.runs.push(Run { start: row, lf: next[byte as usize], byte });
                previous = Some(byte);
            }
            next[byte as usize] += len;
            row += len;
        }
        if row != n { return None; }
        self.starts.push(mask & !1);
        self.page_first.push(base);
        self.starts.resize(pages, 0);
        self.page_first.resize(pages, self.runs.len() as u32 - 1);
        self.runs.push(Run { start: n, lf: 0, byte: 0 });
        Some((quote,tokens))
    }
