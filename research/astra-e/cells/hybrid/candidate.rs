//! Eager Cells with two packed slot words, a five-byte immediate grammar, and a
//! five-to-sixteen-byte literal table, and a running marked offset. This module is an experimental moonshot scorer entry.
use crate::bwt::{InverseBwt, Segmented};
use crate::runs::Runs;
use crate::Kernels;

const BITS: u32 = 20;
const MASK: u64 = (1 << BITS) - 1;
const MARK: u64 = 1 << BITS;
const TAG: u64 = (1 << 24) - 1;
const INLINE: usize = 5;
const LITERAL: usize = 16;

#[derive(Default)]
pub struct Candidate {
    fallback: Segmented,
    // A top slot holds its predecessor, width and opposite slot in the first
    // word. The second word describes the return word.
    tops: Vec<[u64; 2]>,
    rules: Vec<[u64; 2]>,
    literals: Vec<u128>,
    #[cfg(test)]
    cuts: usize,
}

fn word_len(w: u64) -> usize {
    (w & MASK) as usize
}

fn pack(previous: usize, rows: usize, opposite: usize) -> u64 {
    previous as u64 | (rows as u64) << BITS | (opposite as u64) << (2 * BITS)
}

fn pair(previous: usize, top: usize) -> u64 {
    previous as u64 | (top as u64) << 32
}

// Retain capacities between blocks, and request exactly the needed size when
// growing the fixed-size table.
fn fit<T: Copy>(vec: &mut Vec<T>, len: usize, fill: T) {
    if vec.capacity() < len {
        *vec = Vec::new();
        vec.reserve_exact(len);
    }
    vec.resize(len, fill);
}

impl Candidate {
    fn attempt<const FORCE: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: usize,
        out: &mut Vec<u8>,
    ) -> bool {
        if n == 0
            || n > MASK as usize
            || p >= n
            || (!FORCE && (records.len() * 20 >= n * 3 || records.len() * 64 < n))
        {
            return false;
        }
        let mut counts = [0usize; 256];
        let mut row = 0;
        let mut old = 256;
        for &record in records {
            let byte = (record >> 8) as usize;
            let len = (record & 255) as usize;
            if len == 0 || len > 127 {
                return false;
            }
            if byte != old || row == p || row == p + 1 {
                counts[byte] += 1;
            }
            // Count internal cuts as well as those exactly at record boundaries.
            counts[byte] += usize::from(row < p && p < row + len);
            counts[byte] += usize::from(row < p + 1 && p + 1 < row + len);
            row += len;
            old = byte;
        }
        if row != n {
            return false;
        }
        let mut letters = 0;
        for count in &mut counts {
            let width = *count;
            *count = letters;
            letters += width;
        }
        fit(&mut self.tops, letters, [0, 0]);
        self.rules.clear();
        self.literals.clear();
        #[cfg(test)]
        {
            self.cuts = 0;
        }
        // A separate bottom order occupies space in the future inverse output.
        out.resize(8 * letters + 7, 0);
        // SAFETY: all bit patterns represent u64. align_to_mut supplies aligned
        // complete words and borrows the bytes exclusively through this slice.
        let (_, bottom, _) = unsafe { out.align_to_mut::<u64>() };
        let mut i = 0;
        let mut start = 0;
        let mut top = 0;
        while i < records.len() {
            let byte = (records[i] >> 8) as usize;
            let mut end = start;
            while i < records.len() && (records[i] >> 8) as usize == byte {
                end += (records[i] & 255) as usize;
                i += 1;
            }
            while start < end {
                let stop = if start < p {
                    end.min(p)
                } else if start == p {
                    p + 1
                } else {
                    end
                };
                let b = counts[byte];
                counts[byte] += 1;
                let flags = if start == p { MARK } else { 0 };
                self.tops[top] = [
                    pack(top.saturating_sub(1), stop - start, b),
                    1 | flags | (byte as u64) << 24,
                ];
                bottom[b] = pair(b.saturating_sub(1), top);
                start = stop;
                top += 1;
            }
        }
        debug_assert_eq!(top, letters);
        let (cycle, mark) =
            match self.induce(bottom, n, letters, if FORCE { n } else { n * 8 / 25 }) {
                Some(result) => result,
                None => return false,
            };
        if word_len(cycle) != n {
            // The one-scan ownership certificate requires a full-period root.
            return false;
        }
        self.write(cycle, mark, out)
    }

    fn induce(
        &mut self,
        bottom: &mut [u64],
        _n: usize,
        letters: usize,
        mut budget: usize,
    ) -> Option<(u64, usize)> {
        let mut sa = letters - 1;
        let mut sb = letters - 1;
        let mut mark = 0usize;
        'fresh: loop {
            let [ta, mut a] = self.tops[sa];
            let mut prev_a = (ta & MASK) as usize;
            let mut ra = ((ta >> BITS) & MASK) as usize;
            let mut bs = (ta >> (2 * BITS)) as usize;
            if bs == sb {
                if a & MARK != 0 {
                    return Some((a, mark));
                }
                sa = prev_a;
                sb = bottom[sb] as u32 as usize;
                continue 'fresh;
            }
            let mut prev_b = bottom[sb] as u32 as usize;
            let mut ts = (bottom[sb] >> 32) as usize;
            let [tb, mut b] = self.tops[ts];
            let mut rb = ((tb >> BITS) & MASK) as usize;
            loop {
                if budget == 0 {
                    return None;
                }
                budget -= 1;
                #[cfg(test)]
                {
                    self.cuts += 1;
                }
                let alen = word_len(a);
                let tag = (a & TAG) + (b & TAG);
                let len = (tag & MASK) as usize;
                if b & MARK != 0 {
                    mark += alen;
                }
                let word = if len <= INLINE {
                    // Both operands are immediate and their unused high bytes
                    // are zero. b's bytes are shifted past a's byte prefix.
                    (a & !TAG) | ((b & !TAG) << (8 * alen)) | tag
                } else if len <= LITERAL {
                    let a_bytes = short_payload(a, &self.literals);
                    let b_bytes = short_payload(b, &self.literals);
                    let value = a_bytes | (b_bytes << (8 * alen));
                    let index = self.literals.len();
                    self.literals.push(value);
                    (index as u64) << 24 | tag
                } else {
                    let index = self.rules.len();
                    self.rules.push([a, b]);
                    (index as u64) << 24 | tag
                };
                let pred = (self.tops[ts][0] & MASK) as usize;
                self.tops[ts] = [pack(pred, ra.min(rb), bs), word];
                if ra == rb {
                    bottom[bs] = pair(bottom[bs] as u32 as usize, ts);
                    (sa, sb) = (prev_a, prev_b);
                    continue 'fresh;
                } else if ra > rb {
                    let next = bottom[bs] as u32 as usize;
                    bottom[sb] = pair(next, 0);
                    bottom[bs] = pair(sb, ts);
                    bs = sb;
                    ra -= rb;
                    sb = prev_b;
                    let link = bottom[sb];
                    prev_b = link as u32 as usize;
                    ts = (link >> 32) as usize;
                    let [meta, word] = self.tops[ts];
                    rb = ((meta >> BITS) & MASK) as usize;
                    b = word;
                } else {
                    bottom[bs] = pair(bottom[bs] as u32 as usize, ts);
                    self.tops[sa][0] = pred as u64;
                    self.tops[ts][0] = pack(sa, ra, bs);
                    ts = sa;
                    rb -= ra;
                    sa = prev_a;
                    let [meta, word] = self.tops[sa];
                    prev_a = (meta & MASK) as usize;
                    bs = (meta >> (2 * BITS)) as usize;
                    ra = ((meta >> BITS) & MASK) as usize;
                    a = word;
                }
            }
        }
    }

    fn write(&mut self, root: u64, mark: usize, out: &mut Vec<u8>) -> bool {
        let n = word_len(root);
        out.resize(n, 0);
        let first = n - 1 - mark;
        if n <= LITERAL {
            write_literal(out, first, root, &self.literals);
            return true;
        }
        let count = self.rules.len();
        // Reuse the now-dead top slots for positions. Every word of a top slot
        // is initialized and can be viewed as u32 under an exclusive borrow.
        let (_, pos, _) = unsafe { self.tops.align_to_mut::<u32>() };
        if pos.len() < count || (root >> 24) as usize != count - 1 {
            return false;
        }
        pos[count - 1] = first as u32;
        let mut next = [u64::MAX; 2];
        let mut tail = count;
        for j in (0..count).rev() {
            let children = self.rules[j];
            let mut d = pos[j] as usize;
            for (child, later) in children.into_iter().zip(next) {
                let len = word_len(child);
                if len <= LITERAL {
                    write_literal(out, d, child, &self.literals);
                } else if child == later {
                    tail -= 1;
                    self.rules[tail] = [child, d as u64];
                } else {
                    pos[(child >> 24) as usize] = d as u32;
                }
                d += len;
                if d >= n {
                    d -= n;
                }
            }
            next = children;
        }
        for &[child, destination] in &self.rules[tail..count] {
            let source = pos[(child >> 24) as usize] as usize;
            out.copy_within(source..source + word_len(child), destination as usize);
        }
        true
    }
}

#[inline]
fn short_payload(word: u64, literals: &[u128]) -> u128 {
    if word_len(word) <= INLINE {
        (word >> 24) as u128
    } else {
        literals[(word >> 24) as usize]
    }
}

fn write_literal(out: &mut [u8], dest: usize, word: u64, literals: &[u128]) {
    let bytes = short_payload(word, literals).to_le_bytes();
    let len = word_len(word);
    let take = len.min(out.len() - dest);
    out[dest..dest + take].copy_from_slice(&bytes[..take]);
    if take != len {
        out[..len - take].copy_from_slice(&bytes[take..len]);
    }
}

impl InverseBwt for Candidate {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(ll, counts, p, out);
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(ll, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        self.attempt::<false>(r.records(), n, p as usize, out)
            || self.fallback.invert_runs(r, n, p, out, k)
    }
}

#[cfg(test)]
#[path = "checks.rs"]
mod checks;
