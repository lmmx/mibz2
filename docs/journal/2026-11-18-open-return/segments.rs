//! List-rank the residual permutation, staging one word code per retained rank.
use super::{append_code, repeat};

const LANES: usize = 8;
const STRIDE: usize = 128;

#[derive(Default, Clone, Copy)]
struct Segment {
    lane: usize,
    begin: usize,
    end: usize,
    next: usize,
}

#[derive(Default)]
pub(super) struct Segments {
    codes: [Vec<u32>; LANES],
    segments: Vec<Segment>,
}

impl Segments {
    pub(super) fn emit(
        &mut self,
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        offsets: &[u32],
        n: usize,
        mark: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        let low = mark as usize % STRIDE;
        let starts = (edges.len() - 1 - low) / STRIDE + 1;
        self.segments.resize(starts, Segment::default());
        let mask = (1 << bits) - 1;
        let mut next = 0;
        let mut row = [0; LANES];
        let mut current = [usize::MAX; LANES];
        let mut completed = 0;
        for tape in &mut self.codes {
            tape.clear();
        }
        while completed < starts {
            for lane in 0..LANES {
                let tape = &mut self.codes[lane];
                if current[lane] == usize::MAX {
                    if next == starts {
                        continue;
                    }
                    current[lane] = next;
                    self.segments[next].begin = tape.len();
                    self.segments[next].lane = lane;
                    row[lane] = low + next * STRIDE;
                    next += 1;
                }
                let edge = edges[row[lane]];
                let code = if extra.is_empty() {
                    edge >> bits
                } else {
                    extra[row[lane]]
                };
                tape.push(code);
                row[lane] = (edge & mask) as usize;
                if row[lane] % STRIDE == low {
                    let seg = &mut self.segments[current[lane]];
                    seg.next = row[lane] / STRIDE;
                    seg.end = tape.len();
                    current[lane] = usize::MAX;
                    completed += 1;
                }
            }
        }
        out.clear();
        out.reserve(n);
        let first = mark as usize / STRIDE;
        let mut id = first;
        loop {
            let seg = self.segments[id];
            for &code in &self.codes[seg.lane][seg.begin..seg.end] {
                append_code(code, spelling, offsets, out);
            }
            id = seg.next;
            if id == first {
                break;
            }
        }
        repeat(n, out);
    }
}
