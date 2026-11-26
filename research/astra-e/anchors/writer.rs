include!("anchor.rs");

impl Kernel {
    pub fn anchored<const PROBE: bool>(
        &mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>
    ) {
        let (counts, mark, bits) = self.construct::<true, true>(records, n as u32, p, 16);
        let compact_mark = self.anchors.build(&mut self.edges, &self.extra, mark, bits);
        out.clear();
        out.reserve(n);
        let mut writes = 0;
        self.anchors.emit(compact_mark, |code, tail| {
            if code < 256 {
                out.push(code as u8);
            } else {
                let w = (code - 256) as usize;
                out.extend_from_slice(
                    &self.spelling[self.offsets[w] as usize .. self.offsets[w + 1] as usize]
                );
            }
            out.extend_from_slice(tail);
            if PROBE {
                writes += 1;
            }
        });
        if PROBE {
            let words = self.anchors.entries.len() - LANES;
            let literals: usize = self.anchors.tails.iter().map(Vec::len).sum();
            let scratch = self.anchors.entries.capacity() * std::mem::size_of::<Anchor>()
                        + self.anchors.tails.iter().map(Vec::capacity).sum::<usize>();
            eprintln!("anchors,{n},{p},{mark},{},{words},{literals},{writes},{scratch}", counts.retained);
        }
        assert!(!out.is_empty() && out.len() <= n);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }
}
