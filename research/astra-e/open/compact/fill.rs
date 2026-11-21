// Sequential compact thin runs crossed with bottom-order translation bands.
impl Kernel {
    fn fill<const WIDE: bool>(&mut self, mut buckets: [u32; 256], bits: u32) {
        let mut row = 0;
        let mut bands = self.bands.iter();
        let mut band = *bands.next().expect("the marked singleton is retained");
        let mut bias = band.source | if WIDE { 0 } else { band.code << bits };
        for &record in &self.thin_runs {
            let byte = (record >> 8) as usize;
            let width = u32::from(record & 255);
            let mut dest = buckets[byte];
            buckets[byte] += width;
            let end = row + width;
            loop {
                if row == band.end {
                    band = *bands.next().expect("thin row beyond last band");
                    let first = band.source | if WIDE { 0 } else { band.code << bits };
                    bias = first.wrapping_sub(row);
                }
                let stop = end.min(band.end);
                let span = stop - row;
                let source = bias.wrapping_add(row);
                if span == 1 {
                    self.edges[dest as usize] = source;
                    if WIDE {
                        self.extra[dest as usize] = band.code;
                    }
                } else {
                    let slice = dest as usize..(dest + span) as usize;
                    for (i, entry) in self.edges[slice.clone()].iter_mut().enumerate() {
                        *entry = source.wrapping_add(i as u32);
                    }
                    if WIDE {
                        self.extra[slice].fill(band.code);
                    }
                }
                dest += span;
                row = stop;
                if row == end {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod fill_tests {
    use super::{starts, Band, Kernel};

    #[test]
    fn packed_and_wide_band_intersections() {
        let mut k = Kernel::default();
        // A power-of-two m leaves the actual largest rank equal to the mask.
        let mut l = Vec::new();
        let mut counts = [0; 256];
        for i in 0..128 {
            let width = if i % 2 == 0 { 1 } else { 15 };
            let c = i * 17 % 113;
            k.thin_runs.push((c as u16) << 8 | width);
            counts[c] += u32::from(width);
            l.extend(std::iter::repeat_n(c, width as usize));
        }
        assert_eq!(l.len(), 1024);
        let bits = 10;
        let mut source = 1024;
        let mut end = 0;
        let mut codes = Vec::new();
        let mut ranks = Vec::new();
        while end < 1024 {
            let span = (1 + end % 13).min(1024 - end);
            source -= span;
            end += span;
            let code = if codes.is_empty() {
                u32::MAX >> bits
            } else {
                32 + end
            };
            k.bands.push(Band { end, source, code });
            ranks.extend(source..source + span);
            codes.extend(std::iter::repeat_n(code, span as usize));
        }
        let mut inverse: Vec<_> = (0..l.len()).collect();
        inverse.sort_by_key(|&j| l[j]);
        k.edges.resize(l.len(), 0);
        k.fill::<false>(starts(counts), bits);
        for (i, &j) in inverse.iter().enumerate() {
            assert_eq!(k.edges[i], codes[j] << bits | ranks[j]);
        }
        k.extra.resize(l.len(), 0);
        for (band, code) in k.bands.iter_mut().zip(0u32..) {
            band.code = code | 0xf0000000;
        }
        k.fill::<true>(starts(counts), bits);
        for (i, &j) in inverse.iter().enumerate() {
            assert_eq!(k.edges[i], ranks[j]);
            let band = k.bands.iter().find(|band| j < band.end as usize).unwrap();
            assert_eq!(k.extra[i], band.code);
        }
    }
}
