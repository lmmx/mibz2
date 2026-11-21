use std::mem::MaybeUninit;

// Both slices have checked lengths. Spare bytes may be written ahead of the
// logical cursor, and each later word overwrites that padding as it advances.
#[inline]
fn write_bytes<const FIXED: bool>(to: &mut [MaybeUninit<u8>], from: &[u8]) {
    let n = if FIXED { 16 } else { from.len() };
    let to = &mut to[..n];
    let from = &from[..n];
    // SAFETY: slices have n entries; decoder output and dictionary are distinct.
    unsafe { std::ptr::copy_nonoverlapping(from.as_ptr(), to.as_mut_ptr().cast(), n) }
}

impl Kernel {
    pub fn dictionary<const PROBE: bool>(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        let (_, mark, bits) = self.construct::<true, true>(records, n as u32, p, 16);
        const SHIFT: usize = 20;
        const LOW: u32 = (1 << SHIFT) - 1;
        assert!(self.spelling.len() <= LOW as usize);
        // Leave the sentinel's low bits equal to the dictionary end for long words.
        for i in 0..self.offsets.len() - 1 {
            let begin = self.offsets[i];
            let length = self.offsets[i + 1] - begin;
            self.offsets[i] = begin | (length.min(4095) << SHIFT);
        }
        self.spelling.resize(self.spelling.len() + 15, 0);
        out.clear();
        out.reserve(n + 15);
        let dest = &mut out.spare_capacity_mut()[..n + 15];
        let mut pos = 0;
        let mask = (1 << bits) - 1;
        let mut row = mark;
        let mut counts = [0usize; 4];
        loop {
            let edge = self.edges[row as usize];
            let code = if self.extra.is_empty() { edge >> bits } else { self.extra[row as usize] };
            row = edge & mask;
            if code < 256 {
                dest[pos].write(code as u8);
                pos += 1;
                if PROBE { counts[0] += 1; }
            } else {
                let i = (code - 256) as usize;
                let descriptor = self.offsets[i];
                let begin = (descriptor & LOW) as usize;
                let length = (descriptor >> SHIFT) as usize;
                if length <= 16 {
                    write_bytes::<true>(&mut dest[pos..], &self.spelling[begin..]);
                    pos += length;
                    if PROBE { counts[1] += 1; }
                } else {
                    let end = if length < 4095 { begin + length } else { (self.offsets[i+1] & LOW) as usize };
                    write_bytes::<false>(&mut dest[pos..], &self.spelling[begin..end]);
                    pos += end - begin;
                    if PROBE { counts[2 + usize::from(length == 4095)] += 1; }
                }
            }
            if row == mark { break; }
        }
        assert!(pos > 0 && pos <= n);
        // SAFETY: the logical prefix has been filled contiguously by literal,
        // short, or long copies, each writing at least its logical length.
        unsafe { out.set_len(pos) };
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
        if PROBE { eprintln!("dictionary,{n},{p},{},{},{},{}", counts[0],counts[1],counts[2],counts[3]); }
    }
}
