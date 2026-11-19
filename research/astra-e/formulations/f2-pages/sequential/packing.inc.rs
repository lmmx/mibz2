impl RankPages {
    // Runs are sorted by original row, ending at the n-sentinel. Scan them exactly
    // once. The containing run at each page start is the number to encode.
    fn build<const WIDTH: u32, const SPLIT: bool>(&mut self, n: u32, runs: &mut [Run], counts: &[u32;256]) {
        let pages = n.div_ceil(WIDTH) as usize;
        if WIDTH == 64 && !SPLIT {
            self.wide.clear();
            self.wide.reserve(pages);
        } else {
            self.packed.clear();
            self.packed.reserve(pages);
            if SPLIT {
                self.split.clear();
                self.split.reserve(pages);
            }
        }
        let mut k = 0;
        runs[0].lf += counts[runs[0].byte as usize];
        for page in 0..pages {
            let start = page as u32 * WIDTH;
            if runs[k+1].start == start {
                k += 1;
                runs[k].lf += counts[runs[k].byte as usize];
            }
            let base = k as u32;
            let mut cuts = 0;
            let end = (start + WIDTH).min(n);
            while runs[k+1].start < end {
                k += 1;
                let r = &mut runs[k];
                cuts |= 1u64 << (r.start - start - 1);
                r.lf += counts[r.byte as usize];
            }
            if SPLIT {
                self.split.push(base);
                self.packed.push(cuts << 1);
            } else if WIDTH == 64 {
                self.wide.push(WidePage { cuts, base });
            } else {
                self.packed.push(cuts | ((base as u64) << BASE_SHIFT));
            }
        }
    }

    #[inline]
    fn rank<const WIDTH: u32, const SPLIT: bool>(&self, x: u32) -> usize {
        if SPLIT {
            let page = (x >> 6) as usize;
            let prefix = u64::MAX >> (63 - (x & 63));
            (self.split[page] + (self.packed[page] & prefix).count_ones()) as usize
        } else {
            self.locate::<WIDTH>(x)
        }
    }
}
