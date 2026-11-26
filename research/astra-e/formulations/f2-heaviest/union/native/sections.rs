    fn select(&mut self, section: u32, count: u32, k: usize) -> u32 {
        use std::cmp::Reverse;
        let mut widths = [0u32; 256];
        let mut starts = [0u32; 256];
        let mut touched = [0u8; 256];
        let mut best = Vec::with_capacity(k + 1);
        let mut cutoff = 0;
        self.heap.clear();
        self.heap.push((count, Reverse(section)));
        while let Some((width, Reverse(left))) = self.heap.pop() {
            if width < cutoff {
                break;
            }
            let right = left + width;
            let mut nbytes = 0;
            let mut index = self.locate(left);
            while self.runs[index].start < right {
                let run = self.runs[index];
                let lo = left.max(run.start);
                let byte = usize::from(run.byte);
                if widths[byte] == 0 {
                    starts[byte] = run.lf + (lo - run.start);
                    touched[nbytes] = run.byte;
                    nbytes += 1;
                }
                widths[byte] += right.min(self.runs[index + 1].start) - lo;
                index += 1;
            }
            for &byte in &touched[..nbytes] {
                let slot = usize::from(byte);
                let start = starts[slot];
                let frequency = std::mem::take(&mut widths[slot]);
                if frequency < cutoff {
                    continue;
                }
                if (section..section + count).contains(&start) {
                    best.push((frequency, Reverse(start)));
                    best.sort_unstable_by(|a,b| b.cmp(a));
                    best.truncate(k);
                    if best.len() == k {
                        cutoff = best[k-1].0;
                    }
                } else {
                    self.heap.push((frequency, Reverse(start)));
                }
            }
        }
        self.parts.clear();
        best.sort_unstable_by_key(|x| x.1.0);
        let mut total = 0;
        for (width, Reverse(start)) in best {
            if let Some(last) = self.parts.last_mut().filter(|p| p.1 == start) {
                last.1 += width;
            } else {
                self.parts.push((start, start + width, total));
            }
            total += width;
        }
        total
    }

    #[inline]
    fn union_rank(&self, row: u32) -> Option<u32> {
        self.parts.iter().find(|p| p.0 <= row && row < p.1).map(|p| p.2 + (row - p.0))
    }

    #[inline]
    fn hit(&self, byte: u8, row: u32) -> Option<u32> {
        if byte == b'"' { self.union_rank(row) } else { None }
    }

    fn transfer(&self, primary: u32) -> Option<(u32, u32)> {
        let (mut row, mut skip) = (primary, 0);
        loop {
            if let Some(rank) = self.union_rank(row) {
                return Some((rank,skip));
            }
            let r = self.runs[self.locate(row)];
            row = r.lf + (row - r.start);
            skip += 1;
            if row == primary { return None; }
        }
    }
