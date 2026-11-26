impl Kernel {
    pub fn expanded(
        &mut self,
        caps: &[u16],
        n: usize,
        p: u32,
        h: &super::section::Hazards,
        out: &mut Vec<u8>,
    ) -> Result<(), mibz2::ErrorKind> {
        let (stats, mut row, bits) = self.construct::<true, true>(caps, n as u32, p, 16, h);
        let first_tag = 256 + stats.words as u32;
        for &rank in &self.protected {
            let i = rank as usize;
            if self.extra.is_empty() {
                let code = self.edges[i] >> bits;
                assert!(code < 256);
                self.edges[i] += first_tag << bits;
            } else {
                assert!(self.extra[i] < 256);
                self.extra[i] += first_tag;
            }
        }
        let mask = (1u32 << bits) - 1;
        let mut left = n;
        let (mut last, mut count) = (0u8, 0u8);
        out.reserve(n + 64);
        while left != 0 {
            let e = self.edges[row as usize];
            let code = if self.extra.is_empty() {
                e >> bits
            } else {
                self.extra[row as usize]
            };
            row = e & mask;
            if code < 256 {
                out.push(code as u8);
                left -= 1;
                count = 0;
            } else if code < first_tag {
                let i = (code - 256) as usize;
                let start = self.offsets[i] as usize;
                let len = (self.offsets[i + 1] as usize - start).min(left);
                out.extend_from_slice(&self.spelling[start..start + len]);
                left -= len;
                count = 0;
            } else {
                let b = (code - first_tag) as u8;
                if count == 4 {
                    out.resize(out.len() + b as usize, last);
                    count = 0;
                } else {
                    out.push(b);
                    count = if b == last { count + 1 } else { 1 };
                    last = b;
                }
                left -= 1;
            }
        }
        if count == 4 {
            Err(mibz2::ErrorKind::RunUnfinished)
        } else {
            Ok(())
        }
    }
}
