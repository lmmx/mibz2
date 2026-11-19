impl Second {
    // Observer only: production can accumulate the same terminal multiplicities
    // while reading terminal selector widths during dictionary discovery.
    pub(super) fn census(&self, p: u32, skip: u32, n: u32) -> Vec<(u32, u32, u32, u32, u32)> {
        let m = self.links.len() as u32;
        let bits = 32 - (m - 1).leading_zeros();
        let mask = (1u32 << bits) - 1;
        let mut rows: Vec<_> = self
            .words
            .iter()
            .map(|w| (w.width, 0u32, 0u32, 0u32, 0u32))
            .collect();
        for &link in &self.links {
            rows[(link >> bits) as usize].1 += 1;
        }
        let mut emitted = 0;
        let (mut at, mut discard) = (p, skip);
        while emitted < n {
            let link = self.links[at as usize];
            let w = &mut rows[(link >> bits) as usize];
            let count = (w.0 - discard).min(n - emitted);
            w.2 += 1;
            w.3 += count;
            if discard == 0 && count == w.0 {
                w.4 += 1;
            }
            at = link & mask;
            discard = 0;
            emitted += count;
        }
        rows
    }
}
