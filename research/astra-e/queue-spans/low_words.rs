// A bucket crosses at most thirteen global u16 page boundaries on a legal block.
#[derive(Default)]
struct LowCursor {
    from: usize,
    page: usize,
    end: usize,
}

impl LowCursor {
    fn new(byte: usize, cuts: &[[u32; 256]], rows: usize) -> Self {
        let end = cuts.first().map_or(rows, |cut| cut[byte] as usize);
        Self { from: 0, page: 0, end }
    }

    #[inline]
    fn write(
        &mut self, byte: usize, cuts: &[[u32; 256]], rows: &[u16],
        mut edges: &mut [u32], pack: u32, _stats: &mut Counts,
    ) {
        while !edges.is_empty() {
            while self.from == self.end {
                self.page += 1;
                self.end = cuts.get(self.page).map_or(rows.len(), |cut| cut[byte] as usize);
            }
            let take = edges.len().min(self.end - self.from);
            let (here, rest) = edges.split_at_mut(take);
            let high = pack | ((self.page as u32) << 16);
            for (link, &low) in here.iter_mut().zip(&rows[self.from..self.from + take]) {
                *link = high | u32::from(low);
            }
            #[cfg(astra_queue_stats)]
            {
                _stats.intersections += 1;
                _stats.singletons += usize::from(take == 1);
            }
            self.from += take;
            edges = rest;
        }
    }
}
