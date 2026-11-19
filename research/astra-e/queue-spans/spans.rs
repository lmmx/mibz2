// Each entry stores a retained L-rank and up to 4096 successive ranks.
const INDEX_BITS: u32 = 20;
const INDEX_MASK: u32 = (1 << INDEX_BITS) - 1;
const MAX_SPAN: u32 = 1 << (32 - INDEX_BITS);

#[inline]
fn push_span<const QUEUE: u8>(queue: &mut Vec<u32>, mut row: u32, mut width: u32, _stats: &mut Counts) {
    if QUEUE == 3 {
        if let Some(last) = queue.last_mut() {
            let old_width = (*last >> INDEX_BITS) + 1;
            if (*last & INDEX_MASK) + old_width == row {
                let joined = width.min(MAX_SPAN - old_width);
                *last += joined << INDEX_BITS;
                row += joined;
                width -= joined;
                #[cfg(astra_queue_stats)]
                {
                    _stats.queue_joined_rows += joined as usize;
                    _stats.queue_cap += usize::from(width > 0);
                }
            }
        }
    }
    while width > 0 {
        let take = width.min(MAX_SPAN);
        queue.push(row | ((take - 1) << INDEX_BITS));
        row += take;
        width -= take;
    }
}

#[derive(Default)]
struct SpanCursor {
    index: usize,
    row: u32,
    left: usize,
}

impl SpanCursor {
    #[inline]
    fn write(&mut self, spans: &[u32], mut edges: &mut [u32], pack: u32, _stats: &mut Counts) {
        while !edges.is_empty() {
            if self.left == 0 {
                let entry = spans[self.index];
                self.index += 1;
                self.row = entry & INDEX_MASK;
                self.left = ((entry >> INDEX_BITS) + 1) as usize;
            }
            let take = self.left.min(edges.len());
            let (here, rest) = edges.split_at_mut(take);
            #[cfg(astra_queue_stats)]
            {
                _stats.intersections += 1;
                _stats.singletons += usize::from(take == 1);
            }
            let start = pack | self.row;
            for (i, link) in here.iter_mut().enumerate() {
                *link = start + i as u32;
            }
            self.row += take as u32;
            self.left -= take;
            edges = rest;
        }
    }
}
