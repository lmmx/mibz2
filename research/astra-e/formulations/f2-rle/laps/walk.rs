// Full laps need no bounds checks on the remaining *coded* byte count.
// The signed correction is nonzero only when the RLE1 parser consumes a count byte.
#[derive(Default)]
struct Decoder {
    last: u8,
    count: u8,
    delta: i64,
}
impl Decoder {
    fn protected(&mut self, data: &[u8], out: &mut Vec<u8>) {
        for &b in data {
            if self.count == 4 {
                out.resize(out.len() + b as usize, self.last);
                self.delta += i64::from(b) - 1;
                self.count = 0;
            } else {
                out.push(b);
                self.count = if self.last == b { self.count + 1 } else { 1 };
                self.last = b;
            }
        }
    }
}
impl Kernel {
    #[inline(always)]
    fn walk<const CLIPPED: bool>(
        &self,
        n: usize,
        mark: u32,
        bits: u32,
        first: u32,
        s: &mut Decoder,
        out: &mut Vec<u8>,
    ) {
        let mut row = mark;
        let mut left = n;
        let mask = (1u32 << bits) - 1;
        loop {
            let e = self.edges[row as usize];
            let tag = if self.extra.is_empty() {
                e >> bits
            } else {
                self.extra[row as usize]
            };
            row = e & mask;
            if tag < 256 {
                out.push(tag as u8);
                if CLIPPED {
                    left -= 1;
                }
            } else {
                let dirty = tag >= first;
                let tagged = if dirty {
                    self.labels[(tag - first) as usize]
                } else {
                    tag
                };
                let code = tagged & 0x7fff_ffff;
                let one = [code as u8];
                let mut w = if code < 256 {
                    &one[..]
                } else {
                    let i = (code - 256) as usize;
                    &self.spelling[self.offsets[i] as usize..self.offsets[i + 1] as usize]
                };
                if CLIPPED {
                    w = &w[..w.len().min(left)];
                    left -= w.len();
                }
                if dirty {
                    if tagged >> 31 != 0 {
                        s.count = 0;
                    }
                    s.protected(w, out);
                } else {
                    out.extend_from_slice(w);
                }
            }
            if (CLIPPED && left == 0) || (!CLIPPED && row == mark) {
                break;
            }
        }
    }

    fn finish_laps(
        &self,
        n: usize,
        mark: u32,
        bits: u32,
        first: u32,
        out: &mut Vec<u8>,
    ) -> Result<(), mibz2::ErrorKind> {
        out.reserve(n + 64);
        let before = out.len();
        let mut s = Decoder::default();
        self.walk::<false>(n, mark, bits, first, &mut s, out);
        let period = ((out.len() - before) as i64 - s.delta) as usize;
        assert!(period > 0 && period <= n);
        if period < n {
            if self.labels.is_empty() {
                while out.len() - before < n {
                    let take = (out.len() - before).min(n - (out.len() - before));
                    out.extend_from_within(before..before + take);
                }
            } else {
                self.walk::<true>(n - period, mark, bits, first, &mut s, out);
            }
        }
        if s.count == 4 {
            Err(mibz2::ErrorKind::RunUnfinished)
        } else {
            Ok(())
        }
    }
}
