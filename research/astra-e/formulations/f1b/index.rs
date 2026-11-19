// Boundary rank in a 64-row page and the next interval end.
#[derive(Default, Clone, Copy)]
struct Page {
    bits: u64,
    before: u32,
    next: u32,
}
#[derive(Default)]
struct Index {
    pages: Vec<Page>,
}
impl Index {
    fn size(&mut self, n: u32) {
        self.pages.clear();
        self.pages.resize(n.div_ceil(64) as usize, Page::default());
    }
    fn mark(&mut self, x: u32) {
        self.pages[x as usize / 64].bits |= 1u64 << (x % 64);
    }
    fn seal(&mut self, n: u32) -> u32 {
        let mut total = 0;
        for p in &mut self.pages {
            p.before = total;
            total += p.bits.count_ones();
        }
        let mut next = n;
        for (i, p) in self.pages.iter_mut().enumerate().rev() {
            p.next = next;
            if p.bits != 0 {
                next = i as u32 * 64 + p.bits.trailing_zeros();
            }
        }
        total
    }
    fn at(&self, x: u32) -> u32 {
        let p = &self.pages[x as usize / 64];
        let low = u64::MAX >> (63 - (x % 64));
        p.before + (p.bits & low).count_ones() - 1
    }
    fn get(&self, x: u32) -> (u32, u32) {
        let p = &self.pages[x as usize / 64];
        let low = u64::MAX >> (63 - (x % 64));
        let id = p.before + (p.bits & low).count_ones() - 1;
        let after = p.bits & !low;
        (
            id,
            if after == 0 {
                p.next
            } else {
                (x & !63) + after.trailing_zeros()
            },
        )
    }
}

fn chunks(
    records: &[u16],
    p: u32,
    limit: u32,
    bands: &[Band],
    mut pos: [u32; 256],
    mut visit: impl FnMut(u32, u32, u32, u32),
) {
    let mut cursor = pos.map(|k| bands.partition_point(|b| b.end <= k));
    let mut j = 0;
    runs_of(records, p, limit, |c, _x, w, keep| {
        if !keep {
            return;
        }
        let c = c as usize;
        let end = pos[c] + w;
        while pos[c] < end {
            let i = cursor[c];
            let b = bands[i];
            let first = if i == 0 { 0 } else { bands[i - 1].end };
            let width = end.min(b.end) - pos[c];
            visit(b.source + pos[c] - first, width, j, b.code);
            pos[c] += width;
            j += width;
            if pos[c] == b.end {
                cursor[c] += 1;
            }
        }
    });
}
impl ResultMap {
    fn reset(&mut self) {
        self.starts.clear();
        self.tail.clear();
        self.macros.clear();
        self.work = Work::default();
        self.anchors = 0;
    }
    fn anchor(&mut self, start: u32, end: u32) {
        if let Some(last) = self.starts.last_mut().filter(|last| last.end == start) {
            last.end = end;
        } else {
            self.starts.push(AnchorRun {
                start,
                end,
                base: self.anchors,
            });
        }
        self.anchors += end - start;
    }
}
