impl Kernel {
    #[inline]
    fn prefix_s(&self, x: u32) -> u32 {
        let i = self.strips.partition_point(|s| s.from < x);
        if i == 0 {
            0
        } else {
            let t = &self.strips[i - 1];
            self.source_width[i - 1] + (x - t.from).min(t.width)
        }
    }
    #[inline]
    fn rank_b(&self, c: usize, x: u32) -> u32 {
        let thin = x - self.prefix_s(x);
        let a = self.thick_base[c] as usize;
        let b = self
            .thick_base
            .get(c + 1)
            .map(|&v| v as usize)
            .unwrap_or(self.strips.len());
        let bucket = &self.image_order[a..b];
        let i = bucket.partition_point(|&j| self.strips[j as usize].from < x);
        let thick = if i == 0 {
            0
        } else {
            let v = &self.strips[bucket[i - 1] as usize];
            self.image_width[a + i - 1] - self.image_width[a] + (x - v.from).min(v.width)
        };
        thick + self.queues[c].partition_point(|&j| j < thin) as u32
    }
    #[inline]
    fn psi(&self, y: u32) -> u32 {
        let i = self
            .image_order
            .partition_point(|&j| self.strips[j as usize].to <= y);
        if i != 0 {
            let v = &self.strips[self.image_order[i - 1] as usize];
            if y - v.to < v.width {
                return v.from + (y - v.to);
            }
        }
        let c = self.fbase.partition_point(|&v| v <= y) - 1;
        let k = y - self.image_width[i] - self.thin_base[c];
        let thin = self.queues[c][k as usize];
        let mut lo = 0usize;
        let mut hi = self.strips.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if self.strips[mid].from - self.source_width[mid] <= thin {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        thin + self.source_width[lo]
    }
    fn hazards(&mut self, n: u32) -> bool {
        self.events.clear();
        self.danger.clear();
        for c in 0..256 {
            let base = self.fbase[c];
            let mut lo = base;
            let mut hi = self.fbase.get(c + 1).copied().unwrap_or(n);
            for _ in 1..4 {
                if lo == hi {
                    break;
                }
                lo = base + self.rank_b(c, lo);
                hi = base + self.rank_b(c, hi);
            }
            if self.events.len() + (hi - lo) as usize > 1024 {
                return false;
            }
            self.events.extend(lo..hi);
        }
        for i in 0..self.events.len() {
            let mut y = self.events[i];
            for j in 0..5 {
                self.danger.push(y);
                if j < 4 {
                    y = self.psi(y);
                }
            }
        }
        self.danger.sort_unstable();
        self.danger.dedup();
        true
    }
    pub fn capacities(&self) -> usize {
        std::mem::size_of_val(&self.fbase) * 3
            + 4 * (self.source_width.capacity()
                + self.image_width.capacity()
                + self.events.capacity()
                + self.danger.capacity()
                + self.owners.capacity()
                + self.labels.capacity())
    }
}
