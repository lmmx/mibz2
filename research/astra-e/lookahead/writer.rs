impl Kernel {
    pub fn lookahead(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        let (_, mark, bits) = self.construct::<true, true>(records, n as u32, p, 16);
        emit(&self.edges, &self.extra, &self.spelling, &self.offsets, n, mark, bits, out);
    }
}

fn emit(
    edges: &[u32], extra: &[u32], spelling: &[u8], offsets: &[u32],
    n: usize, mark: u32, bits: u32, out: &mut Vec<u8>,
) {
    out.clear();
    out.reserve(n);
    let mask = (1 << bits) - 1;
    let mut row = mark;
    let mut edge = edges[row as usize];
    loop {
        let code = if extra.is_empty() { edge >> bits } else { extra[row as usize] };
        row = edge & mask;
        // The marked cycle always has a successor, including its closing edge.
        // Fetch that successor while the current word is still available.
        edge = edges[row as usize];
        if code < 256 {
            out.push(code as u8);
        } else {
            let w = (code - 256) as usize;
            out.extend_from_slice(&spelling[offsets[w] as usize..offsets[w + 1] as usize]);
        }
        if row == mark {
            break;
        }
    }
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}
