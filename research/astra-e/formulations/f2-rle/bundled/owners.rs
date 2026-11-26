impl Kernel {
    // The current owner changes only when the forward marked row reaches S.
    // Hence every owner of a five-position event window is either its first
    // owner's anchor or one of the retained rows in that same window.
    fn owners_of_windows(&mut self, n: u32) -> bool {
        self.owners.clear();
        self.steps = 0;
        let mut strip = 0;
        for &x in &self.danger {
            while strip < self.strips.len()
                && self.strips[strip].from + self.strips[strip].width <= x
            {
                strip += 1;
            }
            if strip == self.strips.len() || self.strips[strip].from > x {
                self.owners.push(x);
            }
        }
        let mut i = 0;
        while i < self.events.len() {
            let mut a = self.events[i];
            let mut end = a + 1;
            i += 1;
            while i < self.events.len() && self.events[i] == end {
                end += 1;
                i += 1;
            }
            while a < end {
                let mut x = a;
                let mut width = end - a;
                loop {
                    let j = self.strips.partition_point(|s| s.from <= x);
                    if j == 0 || x - self.strips[j - 1].from >= self.strips[j - 1].width {
                        let until = self.strips.get(j).map_or(n, |s| s.from);
                        width = width.min(until - x);
                        self.owners.extend(x..x + width);
                        break;
                    }
                    let t = &self.strips[j - 1];
                    width = width.min(t.from + t.width - x);
                    x = t.to + (x - t.from);
                    self.steps += 1;
                    if self.steps > 4096 {
                        return false;
                    }
                    if x == a {
                        break;
                    }
                }
                a += width;
            }
        }
        true
    }
}
