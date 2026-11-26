//! Recover the exact section bands on demand, caching terminal intervals and dictionary prefixes.
use crate::bwt::{InverseBwt, Segmented};
use crate::runs::Runs;
use crate::Kernels;

const PATTERN: &[u8] = br#"":""#;

#[derive(Clone, Copy, Default)]
struct Run {
    start: u32,
    lf: u32,
    byte: u8,
}
#[derive(Clone, Copy, Default)]
struct Cert {
    lo: u32,
    hi: u32,
    dest: u32,
    offset: u32,
    len: u32,
}
#[derive(Clone, Copy)]
struct Step {
    index: u32,
    row: u32,
    before: u32,
    after: u32,
}

#[derive(Default)]
pub struct Candidate {
    fallback: Segmented,
    runs: Vec<Run>,
    page_first: Vec<u32>,
    starts: Vec<u64>,
    cache: Vec<Cert>,
    path: Vec<Step>,
    dictionary: Vec<u8>,
    offsets: Vec<u32>,
    links: Vec<u32>,
    #[cfg(test)]
    stats: [u64; 3],
}
impl Candidate {
    fn prepare(&mut self, caps: &[u16], n: u32, pattern: &[u8]) -> Option<(u32, u32)> {
        self.runs.clear();
        let mut counts = [0u32; 256];
        let mut row = 0u32;
        let mut prev = None;
        for &rec in caps {
            let c = (rec >> 8) as u8;
            let len = (rec & 255) as u32;
            if len == 0 || len > 127 {
                return None;
            }
            if Some(c) != prev {
                self.runs.push(Run {
                    start: row,
                    lf: counts[c as usize],
                    byte: c,
                });
                prev = Some(c);
            }
            counts[c as usize] += len;
            row += len;
        }
        let last = *pattern.last()?;
        if row != n || counts[last as usize] == 0 {
            return None;
        }
        let width = counts[last as usize];
        let mut total = 0;
        for c in &mut counts {
            let width = *c;
            *c = total;
            total += width;
        }
        let pages = n.div_ceil(64) as usize;
        self.starts.clear();
        self.starts.resize(pages, 0);
        self.page_first.clear();
        self.page_first.resize(pages, 0);
        for r in &mut self.runs {
            r.lf += counts[r.byte as usize];
            let offset = r.start & 63;
            if offset != 0 {
                self.starts[(r.start >> 6) as usize] |= 1u64 << offset;
            }
        }
        self.runs.push(Run {
            start: n,
            lf: 0,
            byte: 0,
        });
        let mut k = 0;
        for (page, base) in self.page_first.iter_mut().enumerate() {
            while self.runs[k + 1].start <= 64 * page as u32 {
                k += 1;
            }
            *base = k as u32;
        }
        let mut start = counts[last as usize];
        let mut end = start + width;
        for &symbol in pattern[..pattern.len() - 1].iter().rev() {
            let mut first = None;
            let mut total = 0;
            let mut idx = self.locate(start);
            while self.runs[idx].start < end {
                let r = self.runs[idx];
                if r.byte == symbol {
                    let lo = start.max(r.start);
                    first.get_or_insert(r.lf + (lo - r.start));
                    total += end.min(self.runs[idx + 1].start) - lo;
                }
                idx += 1;
            }
            start = first?;
            end = start + total;
        }
        Some((start, end - start))
    }
    #[inline]
    fn locate(&self, x: u32) -> usize {
        let page = (x >> 6) as usize;
        let mask = u64::MAX >> (63 - (x & 63));
        (self.page_first[page] + (self.starts[page] & mask).count_ones()) as usize
    }
    fn transfer_mark(&self, p: u32, start: u32, width: u32) -> Option<(u32, u32)> {
        let (mut x, mut skip) = (p, 0);
        while !(start..start + width).contains(&x) {
            let r = self.runs[self.locate(x)];
            x = r.lf + (x - r.start);
            skip += 1;
            if x == p {
                return None;
            }
        }
        Some((x - start, skip))
    }

    fn bands(&mut self, section: u32, m: u32) -> bool {
        self.cache.clear();
        self.cache.resize(self.runs.len() - 1, Cert::default());
        self.links.resize(m as usize, 0);
        self.offsets.clear();
        self.offsets.push(0);
        self.dictionary.clear();
        let bits = u32::BITS - (m - 1).leading_zeros();
        let limit = section + m;
        let mut root = section;
        while root < limit {
            self.path.clear();
            let mut x = root;
            let begin = self.dictionary.len() as u32;
            let (mut left, mut right, dest);
            loop {
                let index = self.locate(x);
                let r = self.runs[index];
                let end = self.runs[index + 1].start;
                #[cfg(test)]
                {
                    self.stats[0] += 1;
                }
                let cert = self.cache[index];
                if cert.lo <= x && x < cert.hi {
                    left = x - cert.lo;
                    right = cert.hi - x;
                    dest = cert.dest + left;
                    self.dictionary.extend_from_within(
                        cert.offset as usize..(cert.offset + cert.len) as usize,
                    );
                    #[cfg(test)]
                    {
                        self.stats[1] += 1;
                    }
                    break;
                }
                let mapped = r.lf + (x - r.start);
                let before = x - r.start;
                let after = end - x;
                if (section..limit).contains(&mapped) {
                    left = before.min(mapped - section);
                    right = after.min(limit - mapped);
                    dest = mapped - section;
                    self.dictionary.push(r.byte);
                    break;
                }
                // Crossing a section endpoint would change this path's stopping time.
                let (before, after) = if mapped < section {
                    (before, after.min(section - mapped))
                } else {
                    (before.min(mapped - limit), after)
                };
                self.path.push(Step {
                    index: index as u32,
                    row: x,
                    before,
                    after,
                });
                x = mapped;
            }
            for step in self.path.iter().rev() {
                left = left.min(step.before);
                right = right.min(step.after);
                let i = step.index as usize;
                self.dictionary.push(self.runs[i].byte);
                // Distinct section representatives use disjoint row orbits. A
                // one-row certificate cannot be queried by a later representative.
                if left + right > 1 {
                    self.cache[i] = Cert {
                        lo: step.row - left,
                        hi: step.row + right,
                        dest: dest - left,
                        offset: begin,
                        len: self.dictionary.len() as u32 - begin,
                    };
                    #[cfg(test)]
                    {
                        self.stats[2] += 1;
                    }
                }
            }
            let width = right.min(limit - root);
            let code = self.offsets.len() as u32 - 1;
            if code > u32::MAX >> bits {
                return false;
            }
            let source = root - section;
            for (offset, e) in self.links[dest as usize..(dest + width) as usize]
                .iter_mut()
                .enumerate()
            {
                *e = (code << bits) | (source + offset as u32);
            }
            self.offsets.push(self.dictionary.len() as u32);
            root += width;
        }
        true
    }

    fn expand(&self, mut row: u32, mut skip: u32, m: u32, n: usize, out: &mut Vec<u8>) {
        let bits = u32::BITS - (m - 1).leading_zeros();
        let mask = (1u32 << bits) - 1;
        out.clear();
        out.reserve(n);
        while out.len() < n {
            let link = self.links[row as usize];
            let word = (link >> bits) as usize;
            let start = (self.offsets[word] + skip) as usize;
            let len = (self.offsets[word + 1] as usize - start).min(n - out.len());
            out.extend_from_slice(&self.dictionary[start..start + len]);
            row = link & mask;
            skip = 0;
        }
    }

    fn for_pattern<const FORCE: bool>(
        &mut self,
        caps: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        pattern: &[u8],
    ) -> bool {
        if n == 0 || n > 900000 || p as usize >= n || caps.len() > n {
            return false;
        }
        if !FORCE && (caps.len() * 20 >= 3 * n || caps.len() * 64 < n) {
            return false;
        }
        #[cfg(test)]
        {
            self.stats = [0; 3];
        }
        let Some((s, m)) = self.prepare(caps, n as u32, pattern) else {
            return false;
        };
        let Some((q, skip)) = self.transfer_mark(p, s, m) else {
            return false;
        };
        if !self.bands(s, m) {
            return false;
        }
        self.expand(q, skip, m, n, out);
        true
    }
}
impl InverseBwt for Candidate {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(ll, counts, p, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(ll, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        self.for_pattern::<false>(r.records(), n, p, out, PATTERN)
            || self.fallback.invert_runs(r, n, p, out, k)
    }
}

#[cfg(test)]
#[path = "checks.rs"]
mod checks;
