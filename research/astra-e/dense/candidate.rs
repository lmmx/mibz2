//! Straight first returns to the width-16 section. Store a resolved descriptor in
//! each homogeneous page and wire the output edge as soon as a band is traced.
use crate::bwt::{InverseBwt, LazyCells};
use crate::{runs::Runs, Kernels};

const PAGE: usize = 32;
const MIXED: u64 = 1 << 63;
const TERMINAL: u32 = 256;

fn tile(end: u32, delta: u32, symbol: u32) -> u64 {
    (u64::from(end) << 41) | (u64::from(symbol) << 32) | u64::from(delta)
}
fn tile_end(t: u64) -> u32 {
    (t >> 41) as u32
}

#[derive(Default)]
pub struct Candidate {
    fallback: LazyCells,
    queues: Vec<Vec<u32>>,
    // Input order and LF order need only the wide intervals. Each per-symbol list
    // stores their starts within that symbol and their widths.
    images: Vec<Vec<(u32, u32)>>,
    strips: Vec<(u32, u32, u32, u8)>,
    tiles: Vec<u64>,
    pages: Vec<u64>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
}

impl Candidate {
    pub(super) fn trace<const WIDTH: u32>(
        &mut self,
        records: &[u16],
        n: u32,
        primary: u32,
        out: &mut Vec<u8>,
    ) {
        self.queues.resize_with(256, Vec::new);
        self.images.resize_with(256, Vec::new);
        self.queues.iter_mut().for_each(Vec::clear);
        self.images.iter_mut().for_each(Vec::clear);
        self.strips.clear();
        let mut count = [0u32; 256];
        let mut start = 0;
        let mut retained = 0;
        let mut mark = 0;
        let mut i = 0;
        while i < records.len() {
            let byte = (records[i] >> 8) as u8;
            let a = usize::from(byte);
            let mut width = u32::from(records[i] & 255);
            i += 1;
            while i < records.len() && (records[i] >> 8) as u8 == byte {
                width += u32::from(records[i] & 255);
                i += 1;
            }
            let end = start + width;
            let in_run = (start..end).contains(&primary);
            if width <= WIDTH {
                if in_run {
                    mark = retained + primary - start;
                }
                self.queues[a].extend(retained..retained + width);
                retained += width;
            } else {
                let mut wide = |x: u32, w: u32| {
                    if w != 0 {
                        let ordinal = count[a] + x - start;
                        self.strips.push((x, w, ordinal, byte));
                        self.images[a].push((ordinal, w));
                    }
                };
                if in_run {
                    wide(start, primary - start);
                    wide(primary + 1, end - primary - 1);
                    mark = retained;
                    self.queues[a].push(retained);
                    retained += 1;
                } else {
                    wide(start, width);
                }
            }
            count[a] += width;
            start = end;
        }
        assert_eq!(start, n);
        let mut sum = 0;
        let base = count.map(|c| {
            sum += c;
            sum - c
        });
        self.tiles.clear();
        let mut next = 0;
        let mut narrow = 0u32;
        for &(x, w, ordinal, c) in &self.strips {
            if next != x {
                self.tiles
                    .push(tile(x, narrow.wrapping_sub(next), TERMINAL));
                narrow += x - next;
            }
            next = x + w;
            self.tiles.push(tile(
                next,
                (base[usize::from(c)] + ordinal).wrapping_sub(x),
                c.into(),
            ));
        }
        if next < n {
            self.tiles
                .push(tile(n, narrow.wrapping_sub(next), TERMINAL));
        }
        self.pages.clear();
        let mut ti = 0;
        for x in (0..n as usize).step_by(PAGE) {
            while tile_end(self.tiles[ti]) as usize <= x {
                ti += 1;
            }
            let first = self.tiles[ti];
            self.pages
                .push(if tile_end(first) as usize >= (x + PAGE).min(n as usize) {
                    first
                } else {
                    MIXED | ti as u64
                });
        }

        let bits = (32 - (retained - 1).leading_zeros()).max(1);
        let mask = (1 << bits) - 1;
        self.edges.resize(retained as usize, 0);
        self.extra.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        for a in 0..256 {
            let mut left = base[a];
            let mut qi = 0;
            for &(first, w) in self.images[a].iter().chain(std::iter::once(&(count[a], 0))) {
                let end = base[a] + first;
                while left < end {
                    let mut row = left;
                    let mut width = end - left;
                    let spell_start = self.spelling.len();
                    loop {
                        let mut t = self.pages[row as usize / PAGE];
                        if t & MIXED != 0 {
                            let mut j = t as u32 as usize;
                            while row >= tile_end(self.tiles[j]) {
                                j += 1;
                            }
                            t = self.tiles[j];
                        }
                        width = width.min(tile_end(t) - row);
                        row = row.wrapping_add(t as u32);
                        let symbol = (t >> 32) as u32 & 511;
                        if symbol == TERMINAL {
                            break;
                        }
                        self.spelling.push(symbol as u8);
                    }
                    let code = if spell_start == self.spelling.len() {
                        a as u32
                    } else {
                        self.spelling[spell_start..].reverse();
                        self.spelling.push(a as u8);
                        self.offsets.push(self.spelling.len() as u32);
                        self.offsets.len() as u32 + 254
                    };
                    // Past the packing limit, save the codes on a second tape. Unwritten
                    // destinations are overwritten when their unique band is processed.
                    if code > (u32::MAX >> bits) && self.extra.is_empty() {
                        self.extra
                            .extend(self.edges.iter().map(|&edge| edge >> bits));
                    }
                    let dst = row as usize;
                    let span = width as usize;
                    if !self.extra.is_empty() {
                        self.extra[dst..dst + span].fill(code);
                    }
                    for (entry, &src) in self.edges[dst..dst + span]
                        .iter_mut()
                        .zip(&self.queues[a][qi..qi + span])
                    {
                        *entry = (code << bits) | src;
                    }
                    qi += span;
                    left += width;
                }
                left += w;
            }
            debug_assert_eq!(qi, self.queues[a].len());
        }
        out.clear();
        out.reserve(n as usize);
        let mut j = mark;
        loop {
            let edge = self.edges[j as usize];
            let c = if self.extra.is_empty() {
                edge >> bits
            } else {
                self.extra[j as usize]
            };
            if c < 256 {
                out.push(c as u8);
            } else {
                let w = (c - 256) as usize;
                out.extend_from_slice(
                    &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize],
                );
            }
            j = edge & mask;
            if j == mark {
                break;
            }
        }
        assert!(!out.is_empty() && out.len() <= n as usize);
        while out.len() < n as usize {
            let take = (n as usize - out.len()).min(out.len());
            out.extend_from_within(..take);
        }
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
        if r.records().len() * 20 >= n * 3 {
            return self.fallback.invert_runs(r, n, p, out, k);
        }
        self.trace::<16>(r.records(), n as u32, p, out);
        true
    }
}
