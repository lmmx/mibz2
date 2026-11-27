//! Invert an ordered permutation by weighted first returns to its local minima.
use crate::bwt::{InverseBwt, Segmented};
use crate::{runs::Runs, Kernels};

#[derive(Clone, Copy)]
struct Band {
    end: u32,
    delta: i32,
    byte: u8,
}

fn bands(records: &[u16], n: usize) -> Option<Vec<Band>> {
    let mut counts = [0u32; 256];
    for &v in records {
        counts[usize::from(v >> 8)] += u32::from(v & 255);
    }
    let mut sum = 0;
    for v in &mut counts {
        let width = *v;
        *v = sum;
        sum += width;
    }
    if sum as usize != n || n == 0 {
        return None;
    }
    let mut bands: Vec<Band> = Vec::with_capacity(records.len());
    let mut row = 0;
    for &v in records {
        let byte = (v >> 8) as u8;
        let width = u32::from(v & 255);
        let dest = &mut counts[usize::from(byte)];
        if bands.last().is_some_and(|b| b.byte == byte) {
            bands.last_mut().unwrap().end = row + width;
        } else {
            bands.push(Band {
                end: row + width,
                delta: *dest as i32 - row as i32,
                byte,
            });
        }
        row += width;
        *dest += width;
    }
    Some(bands)
}

const STEP: u64 = 1u64 << 32;
const MARKED: u64 = 1u64 << 63;

#[derive(Debug, Default)]
pub struct Stats {
    pub n: usize,
    pub runs: usize,
    pub fixed: usize,
    pub down: usize,
    pub up: usize,
    pub minima: usize,
    pub period: usize,
    pub marked_minima: usize,
    pub max_excursion: u32,
    pub levels: Vec<usize>,
}

// The directional order is needed within a band only when source and image
// overlap. In the common disjoint case the compiler can vectorize the bulk add.
fn inherit(data: &mut [u64], lo: usize, hi: usize, delta: i32) {
    let width = hi - lo;
    if delta < 0 {
        let step = (-delta) as usize;
        if step >= width {
            let (earlier, later) = data.split_at_mut(lo);
            for (dst, src) in later[..width]
                .iter_mut()
                .zip(&earlier[lo - step..hi - step])
            {
                *dst = *src + STEP;
            }
        } else {
            for i in lo..hi {
                data[i] = data[i - step] + STEP;
            }
        }
    } else {
        let step = delta as usize;
        if step >= width {
            let (earlier, later) = data.split_at_mut(hi);
            for (dst, src) in earlier[lo..hi].iter_mut().zip(&later[lo + step - hi..step]) {
                *dst = *src + STEP;
            }
        } else {
            for i in (lo..hi).rev() {
                data[i] = data[i + step] + STEP;
            }
        }
    }
}

fn first_returns(bands: &[Band], n: usize, jumps: &mut Vec<u64>) {
    jumps.clear();
    jumps.extend((0..n as u32).map(u64::from));
    let mut lo = 0;
    for b in bands {
        let hi = b.end as usize;
        if b.delta < 0 {
            inherit(jumps, lo, hi, b.delta);
        }
        lo = hi;
    }
    for i in (0..bands.len()).rev() {
        let b = bands[i];
        if b.delta > 0 {
            let lo = if i == 0 { 0 } else { bands[i - 1].end as usize };
            inherit(jumps, lo, b.end as usize, b.delta);
        }
    }
}

fn mark_cycle(jumps: &mut [u64], anchor: usize) -> (usize, usize) {
    let (mut x, mut elapsed, mut visits) = (anchor, 0, 0);
    loop {
        let link = jumps[x];
        jumps[x] = MARKED | elapsed as u64;
        elapsed += (link >> 32) as usize;
        visits += 1;
        x = link as u32 as usize;
        if x == anchor {
            return (elapsed, visits);
        }
    }
}

fn expand_period(out: &mut [u8], period: usize) {
    let mut written = period;
    while written < out.len() {
        let add = written.min(out.len() - written);
        out.copy_within(..add, written);
        written += add;
    }
}

// In the ordinary case every row lies on the marked orbit. 'phase' is the
// forward distance from anchor to p. L[x] precedes x's starting position.
fn scatter<const ALL: bool>(
    bands: &[Band],
    jumps: &[u64],
    period: usize,
    phase: usize,
    out: &mut [u8],
) {
    let d = period as i32;
    let mut lo = 0;
    for b in bands {
        let hi = b.end as usize;
        for &link in &jumps[lo..hi] {
            let back;
            let distance;
            if link & MARKED != 0 {
                back = link as u32;
                distance = 0;
            } else {
                let dest = jumps[link as u32 as usize];
                if !ALL && dest & MARKED == 0 {
                    continue;
                }
                back = dest as u32;
                distance = (link >> 32) as u32;
            }
            let mut pos = distance as i32 - back as i32 - phase as i32 - 1;
            if pos < 0 {
                pos += d;
            }
            if pos < 0 {
                pos += d;
            }
            // distance <= period, phase >= 1, back >= 0.
            out[pos as usize] = b.byte;
        }
        lo = hi;
    }
}

// A reduced permutation has arbitrary positive edge weights. Fixed edges
// represent whole cycles and are removed from subsequent levels.
fn solve_weighted(edges: &[u64], p: usize, sizes: &mut Vec<usize>) -> (Vec<u32>, u32) {
    const ABSENT: u32 = u32::MAX;
    let n = edges.len();
    sizes.push(n);
    if edges[p] as u32 as usize == p {
        let mut phase = vec![ABSENT; n];
        phase[p] = 0;
        return (phase, (edges[p] >> 32) as u32);
    }
    let mut next: Vec<u64> = (0..n as u32).map(u64::from).collect();
    let mut low = vec![false; n];
    for (x, &edge) in edges.iter().enumerate() {
        let y = edge as u32 as usize;
        if y < x {
            next[x] = next[y] + (edge & !(u32::MAX as u64));
            if (edges[y] as u32 as usize) > y {
                low[y] = true;
            }
        }
    }
    for x in (0..n).rev() {
        let edge = edges[x];
        let y = edge as u32 as usize;
        if y > x {
            next[x] = next[y] + (edge & !(u32::MAX as u64));
        }
    }
    let mut id = vec![ABSENT; n];
    let mut reduced = Vec::new();
    for (x, &v) in low.iter().enumerate() {
        if v {
            id[x] = reduced.len() as u32;
            reduced.push(next[x]);
        }
    }
    for e in &mut reduced {
        *e = (*e & !(u32::MAX as u64)) | u64::from(id[*e as u32 as usize]);
    }
    let first = next[p];
    let q = id[first as u32 as usize] as usize;
    let (smaller, period) = solve_weighted(&reduced, q, sizes);
    let advance = (first >> 32) as i32;
    let mut phase = vec![ABSENT; n];
    for (x, &v) in next.iter().enumerate() {
        let t = id[v as u32 as usize];
        if t == ABSENT || smaller[t as usize] == ABSENT {
            continue;
        }
        let mut a = smaller[t as usize] as i32 + (v >> 32) as i32 - advance;
        if a < 0 {
            a += period as i32;
        }
        if a >= period as i32 {
            a -= period as i32;
        }
        phase[x] = a as u32;
    }
    (phase, period)
}

fn recursive(jumps: &mut [u64], p: usize, low: &[bool], stats: &mut Stats) {
    let mut id = vec![u32::MAX; jumps.len()];
    let mut reduced = Vec::with_capacity(low.iter().filter(|&&yes| yes).count());
    for (x, &yes) in low.iter().enumerate() {
        if yes {
            id[x] = reduced.len() as u32;
            reduced.push(jumps[x]);
        }
    }
    for e in &mut reduced {
        *e = (*e & !(u32::MAX as u64)) | u64::from(id[*e as u32 as usize]);
    }
    let anchor = jumps[p] as u32 as usize;
    let (phases, period) = solve_weighted(&reduced, id[anchor] as usize, &mut stats.levels);
    stats.period = period as usize;
    for (i, &j) in id.iter().enumerate() {
        if j != u32::MAX {
            let a = phases[j as usize];
            if a != u32::MAX {
                let back = if a == 0 { 0 } else { period - a };
                jumps[i] = MARKED | u64::from(back);
                stats.marked_minima += 1;
            }
        }
    }
}

#[derive(Default)]
pub struct Valleys<const RECURSE: bool> {
    jump: Vec<u64>,
    fallback: Segmented,
}

impl<const RECURSE: bool> Valleys<RECURSE> {
    pub fn records<const CENSUS: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> Option<Stats> {
        let bands = bands(records, n)?;
        if p as usize >= n {
            return None;
        }
        let mut s = Stats {
            n,
            runs: bands.len(),
            ..Stats::default()
        };
        out.resize(n, 0);
        let primary = bands[bands.partition_point(|b| b.end <= p)];
        if primary.delta == 0 && !CENSUS {
            out.fill(primary.byte);
            s.period = 1;
            return Some(s);
        }
        first_returns(&bands, n, &mut self.jump);
        let mut low = Vec::new();
        if CENSUS || RECURSE {
            low.resize(n, false);
            for &link in &self.jump {
                if link >> 32 > 0 {
                    low[link as u32 as usize] = true;
                    s.max_excursion = s.max_excursion.max((link >> 32) as u32);
                }
            }
            s.minima = low.iter().filter(|&&yes| yes).count();
            let mut lo = 0;
            for b in &bands {
                let width = b.end as usize - lo;
                match b.delta.cmp(&0) {
                    std::cmp::Ordering::Greater => s.up += width,
                    std::cmp::Ordering::Less => s.down += width,
                    std::cmp::Ordering::Equal => s.fixed += width,
                }
                lo = b.end as usize;
            }
        }
        if primary.delta == 0 {
            out.fill(primary.byte);
            s.period = 1;
            return Some(s);
        }
        let first = self.jump[p as usize];
        let phase = (first >> 32) as usize;
        if RECURSE {
            recursive(&mut self.jump, p as usize, &low, &mut s);
        } else {
            (s.period, s.marked_minima) = mark_cycle(&mut self.jump, first as u32 as usize);
        }
        if s.period == n {
            scatter::<true>(&bands, &self.jump, s.period, phase, out);
        } else {
            scatter::<false>(&bands, &self.jump, s.period, phase, out);
            expand_period(out, s.period);
        }
        Some(s)
    }

    pub fn column(&mut self, ll: &[u8], p: u32, out: &mut Vec<u8>) -> Stats {
        let mut records = Vec::new();
        let mut i = 0;
        while i < ll.len() {
            let a = ll[i];
            let mut end = i + 1;
            while end < ll.len() && end - i < 127 && ll[end] == a {
                end += 1;
            }
            records.push((u16::from(a) << 8) | (end - i) as u16);
            i = end;
        }
        self.records::<true>(&records, ll.len(), p, out).unwrap()
    }
}

impl<const RECURSE: bool> InverseBwt for Valleys<RECURSE> {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(ll, counts, p, out);
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(ll, counts, p)
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, _: Kernels) -> bool {
        self.records::<false>(r.records(), n, p, out).is_some()
    }
}
