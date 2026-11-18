//! Decode four return walks into byte tapes and join finished fragments.
use super::{append_code, emit, repeat};

const LANES: usize = 4;
const TARGET_SEEDS: usize = 32;

#[derive(Default, Copy, Clone)]
struct Fragment {
    lane: usize,
    begin: usize,
    end: usize,
    next: usize,
}

#[derive(Default)]
pub(super) struct Tapes {
    bytes: [Vec<u8>; LANES],
    fragments: Vec<Fragment>,
}

struct Dictionary<'a> {
    payload: &'a [u8],
    offsets: &'a [u32],
    extra: &'a [u32],
    bits: u32,
}

impl Dictionary<'_> {
    #[inline(always)]
    fn append<const WIDE: bool>(&self, row: usize, edge: u32, tape: &mut Vec<u8>) {
        let code = if WIDE {
            self.extra[row]
        } else {
            edge >> self.bits
        };
        append_code(code, self.payload, self.offsets, tape);
    }
}

impl Tapes {
    pub(super) fn write(
        &mut self,
        edges: &[u32],
        extra: &[u32],
        payload: &[u8],
        offsets: &[u32],
        n: usize,
        mark: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        if edges.len() < 512 {
            emit(edges, extra, payload, offsets, n, mark, bits, out);
            return;
        }
        let dict = Dictionary {
            payload,
            offsets,
            extra,
            bits,
        };
        if extra.is_empty() {
            self.walk::<false>(edges, &dict, n, mark as usize, out);
        } else {
            self.walk::<true>(edges, &dict, n, mark as usize, out);
        }
    }

    fn walk<const WIDE: bool>(
        &mut self,
        edges: &[u32],
        dict: &Dictionary<'_>,
        n: usize,
        mark: usize,
        out: &mut Vec<u8>,
    ) {
        let stride = edges.len().div_ceil(TARGET_SEEDS).next_power_of_two();
        let test = stride - 1;
        let low = mark & test;
        let count = (edges.len() - 1 - low) / stride + 1;
        debug_assert!(count >= LANES && count <= TARGET_SEEDS);
        self.fragments.resize(count, Fragment::default());
        let mask = (1u32 << dict.bits) - 1;
        let mut active = [true; LANES];
        let mut id = [0; LANES];
        let mut row = [0; LANES];
        let mut next_seed = LANES;
        for lane in 0..LANES {
            self.bytes[lane].clear();
            self.bytes[lane].reserve(n / LANES);
            id[lane] = lane;
            row[lane] = low + lane * stride;
            self.fragments[lane] = Fragment {
                lane,
                begin: 0,
                ..Fragment::default()
            };
        }
        // Issue the independent edge reads before any of their spellings.
        loop {
            let edge = row.map(|r| edges[r]);
            for lane in 0..LANES {
                dict.append::<WIDE>(row[lane], edge[lane], &mut self.bytes[lane]);
                row[lane] = (edge[lane] & mask) as usize;
            }
            if row.iter().all(|&r| r & test != low) {
                continue;
            }
            let mut exhausted = false;
            for lane in 0..LANES {
                if row[lane] & test == low {
                    let end = self.bytes[lane].len();
                    self.fragments[id[lane]].end = end;
                    self.fragments[id[lane]].next = row[lane] / stride;
                    if next_seed < count {
                        id[lane] = next_seed;
                        row[lane] = low + next_seed * stride;
                        self.fragments[next_seed] = Fragment {
                            lane,
                            begin: end,
                            ..Fragment::default()
                        };
                        next_seed += 1;
                    } else {
                        active[lane] = false;
                        exhausted = true;
                    }
                }
            }
            if exhausted {
                break;
            }
        }
        // At most three in-flight intervals remain, with no unassigned seed.
        for lane in 0..LANES {
            if !active[lane] {
                continue;
            }
            loop {
                let edge = edges[row[lane]];
                dict.append::<WIDE>(row[lane], edge, &mut self.bytes[lane]);
                row[lane] = (edge & mask) as usize;
                if row[lane] & test == low {
                    break;
                }
            }
            self.fragments[id[lane]].end = self.bytes[lane].len();
            self.fragments[id[lane]].next = row[lane] / stride;
        }
        out.clear();
        out.reserve(n);
        let first = mark / stride;
        let mut selected = first;
        loop {
            let f = self.fragments[selected];
            out.extend_from_slice(&self.bytes[f.lane][f.begin..f.end]);
            selected = f.next;
            if selected == first {
                break;
            }
        }
        repeat(n, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragment_assembly() {
        let mut tapes = Tapes::default();
        let mut expected = Vec::new();
        let mut actual = Vec::new();
        let dict = [9, 3, 6, 1, 2, 3, 5];
        let offsets = [0, 3, 7];
        let mut seed = 0x798d1234u32;
        let mut cases = 0;
        for m in [1usize, 511, 512, 513, 1023, 1024, 1025, 8195] {
            let bits = (usize::BITS - (m - 1).leading_zeros()).max(1);
            for kind in 0..4 {
                let mut perm: Vec<u32> = (0..m as u32).collect();
                match kind {
                    0 => perm.rotate_left(1 % m),
                    1 => perm.reverse(),
                    2 => {
                        for end in (1..m).rev() {
                            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                            perm.swap(end, (seed as usize) % (end + 1));
                        }
                    }
                    _ => {} // singleton cycles, including seeds off the marked cycle
                }
                let codes: Vec<u32> = (0..m)
                    .map(|i| match i % 5 {
                        0 => 256,
                        1 => 257,
                        _ => (i % 247) as u32,
                    })
                    .collect();
                let n: usize = codes
                    .iter()
                    .map(|&c| match c {
                        256 => 3,
                        257 => 4,
                        _ => 1,
                    })
                    .sum();
                let packed: Vec<u32> = perm
                    .iter()
                    .zip(&codes)
                    .map(|(&r, &v)| r | (v << bits))
                    .collect();
                for (edges, extra) in [(packed.as_slice(), &[][..]), (&perm[..], &codes[..])] {
                    for mark in (0..m).step_by((m / 13).max(1)).chain([m - 1]) {
                        emit(
                            edges,
                            extra,
                            &dict,
                            &offsets,
                            n,
                            mark as u32,
                            bits,
                            &mut expected,
                        );
                        tapes.write(
                            edges,
                            extra,
                            &dict,
                            &offsets,
                            n,
                            mark as u32,
                            bits,
                            &mut actual,
                        );
                        assert_eq!(actual, expected, "m={m}, shape={kind}, mark={mark}");
                        cases += 1;
                    }
                }
            }
        }
        eprintln!("matched the marked single-chain writer on {cases} residual fixtures");
    }
}
