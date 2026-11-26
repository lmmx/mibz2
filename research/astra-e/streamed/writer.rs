include!("chunks.rs");

impl Kernel {
    pub fn expand(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        chunk: &mut Vec<u8>,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<(), ErrorKind> {
        let (_, mark, bits) = self.construct::<true, true>(records, n as u32, p, 16);
        let mask = (1 << bits) - 1;
        let mut row = mark;
        let mut remaining = n;
        chunk.clear();
        chunk.reserve(2 * CHUNK);
        // The common case has few RLE runs. Match the normal decoder's initial
        // output reservation and avoid geometrically growing it a chunk at a time.
        out.reserve(n + 64);
        while remaining != 0 {
            let edge = self.edges[row as usize];
            let code = if self.extra.is_empty() {
                edge >> bits
            } else {
                self.extra[row as usize]
            };
            row = edge & mask;
            if code < 256 {
                chunk.push(code as u8);
                if chunk.len() == CHUNK {
                    flush(chunk, out, k)?;
                }
                remaining -= 1;
            } else {
                let w = (code - 256) as usize;
                let bytes = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                let take = bytes.len().min(remaining);
                word(chunk, &bytes[..take], out, k)?;
                remaining -= take;
            }
        }
        rle::unrle(chunk, out, k)
    }
}
