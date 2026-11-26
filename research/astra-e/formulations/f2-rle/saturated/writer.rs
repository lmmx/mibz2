impl Kernel {
    pub fn expanded(
        &mut self,
        caps: &[u16],
        n: usize,
        p: u32,
        h: &super::section::Hazards,
        out: &mut Vec<u8>,
        k: super::Kernels,
    ) -> Result<(), mibz2::ErrorKind> {
        let (s, mark, bits) =
            self.construct::<true, true>(caps, n as u32, p, 16, h.danger.len() as u32);
        self.owners.clear();
        self.labels.clear();
        self.steps = 0;
        self.worked = false;
        for &start in &h.danger {
            let mut x = start;
            loop {
                let j = self.strips.partition_point(|strip| strip.from <= x);
                if j == 0 {
                    self.owners.push(x);
                    break;
                }
                let strip = &self.strips[j - 1];
                if x - strip.from >= strip.width {
                    self.owners.push(x);
                    break;
                }
                self.steps += 1;
                if self.steps > 4096 {
                    // The ordinary graph is complete and still unmodified. Use it directly.
                    emit(
                        &self.edges,
                        &self.extra,
                        &self.spelling,
                        &self.offsets,
                        n,
                        mark,
                        bits,
                        &mut self.raw,
                    );
                    return mibz2::rle::unrle(&self.raw, out, k);
                }
                x = strip.to + (x - strip.from);
                if x == start {
                    break;
                } // a closed component disjoint from the retained section
            }
        }
        self.worked = true;
        self.owners.sort_unstable();
        self.owners.dedup();
        let (mut strip, mut skipped) = (0, 0u32);
        let first = 256 + s.words as u32;
        let mask = (1 << bits) - 1;
        for &x in &self.owners {
            while strip < self.strips.len() && self.strips[strip].from <= x {
                let v = &self.strips[strip];
                assert!(v.from + v.width <= x);
                skipped += v.width;
                strip += 1;
            }
            let pos = (x - skipped) as usize;
            let tag = first + self.labels.len() as u32;
            if self.extra.is_empty() {
                self.labels.push(self.edges[pos] >> bits);
                self.edges[pos] = (self.edges[pos] & mask) | (tag << bits);
            } else {
                self.labels.push(self.extra[pos]);
                self.extra[pos] = tag;
            }
        }
        let mut row = mark;
        let mut left = n;
        let (mut count, mut last) = (0u8, 0u8);
        out.reserve(n + 64);
        while left != 0 {
            let e = self.edges[row as usize];
            let tag = if self.extra.is_empty() {
                e >> bits
            } else {
                self.extra[row as usize]
            };
            row = e & mask;
            if tag < 256 {
                out.push(tag as u8);
                left -= 1;
                count = 0;
                continue;
            }
            let protected = tag >= first;
            let code = if protected {
                self.labels[(tag - first) as usize]
            } else {
                tag
            };
            let lit = [code as u8];
            let word = if code < 256 {
                &lit[..]
            } else {
                let i = (code - 256) as usize;
                &self.spelling[self.offsets[i] as usize..self.offsets[i + 1] as usize]
            };
            let take = left.min(word.len());
            if protected {
                for &b in &word[..take] {
                    if count == 4 {
                        out.resize(out.len() + b as usize, last);
                        count = 0;
                    } else {
                        out.push(b);
                        count = if b == last { count + 1 } else { 1 };
                        last = b;
                    }
                }
            } else {
                out.extend_from_slice(&word[..take]);
                count = 0;
            }
            left -= take;
        }
        if count == 4 {
            Err(mibz2::ErrorKind::RunUnfinished)
        } else {
            Ok(())
        }
    }
    pub fn pricing(&self) -> (usize, usize, bool) {
        (self.steps, self.labels.len(), self.worked)
    }
}
