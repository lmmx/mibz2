//! Kitchen Sink cycle 10, theorem 10.2. The Open edges already spell forward.
//! Each seed owns the half-open path to the next seed. Different permutation
//! cycles are independent, including cycles with no coset seed at all.

use super::{inline_words, repeat, Cap, Kernel, INLINE, LIMIT, LONG};
use std::sync::atomic::{AtomicUsize, Ordering};

const STRIDE: usize = 512;

struct Table<'a> {
    edges: &'a [u32],
    extra: &'a [u32],
    inline: &'a [[u8; INLINE]],
    lens: &'a [u8],
    spelling: &'a [u8],
    offsets: &'a [u32],
    bits: u32,
    mark: usize,
}

struct Segment {
    bytes: Vec<u8>,
    next: usize,
}

impl Table<'_> {
    // The primary takes the last slot if it isn't already a regular seed.
    fn slot(&self, row: usize) -> usize {
        if row & (STRIDE - 1) == 0 {
            row / STRIDE
        } else {
            debug_assert_eq!(row, self.mark);
            self.edges.len().div_ceil(STRIDE)
        }
    }

    fn segment(&self, mut row: usize) -> Segment {
        let mask = (1 << self.bits) - 1;
        let mut bytes = Vec::new();
        // The table is a permutation. Even a one-seed cycle returns within m hops.
        for _ in 0..self.edges.len() {
            let edge = self.edges[row];
            let code = if self.extra.is_empty() {
                edge >> self.bits
            } else {
                self.extra[row]
            } as usize;
            let len = self.lens[code];
            let word = if len == LONG {
                let w = code - 256;
                &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize]
            } else {
                &self.inline[code][..usize::from(len)]
            };
            bytes.extend_from_slice(word);
            row = (edge & mask) as usize;
            if row & (STRIDE - 1) == 0 || row == self.mark {
                return Segment {
                    bytes,
                    next: self.slot(row),
                };
            }
        }
        unreachable!("Open edge permutation lost its starting seed")
    }

    fn emit(&self, lanes: usize, n: usize, out: &mut Vec<u8>) {
        let regular = self.edges.len().div_ceil(STRIDE);
        let count = regular + usize::from(self.mark & (STRIDE - 1) != 0);
        let cursor = AtomicUsize::new(0);
        let walk = || {
            let mut pieces = Vec::new();
            loop {
                let slot = cursor.fetch_add(1, Ordering::Relaxed);
                if slot >= count {
                    return pieces;
                }
                let seed = if slot == regular {
                    self.mark
                } else {
                    slot * STRIDE
                };
                pieces.push((slot, self.segment(seed)));
            }
        };
        let mut segments: Vec<_> = (0..count).map(|_| None).collect();
        std::thread::scope(|scope| {
            let helpers: Vec<_> = (1..lanes.min(count)).map(|_| scope.spawn(walk)).collect();
            for group in
                std::iter::once(walk()).chain(helpers.into_iter().map(|h| h.join().unwrap()))
            {
                for (slot, piece) in group {
                    segments[slot] = Some(piece);
                }
            }
        });
        out.clear();
        let first = self.slot(self.mark);
        let mut slot = first;
        loop {
            let piece = segments[slot].take().expect("revisited a non-primary seed");
            let take = piece.bytes.len().min(n - out.len());
            out.extend_from_slice(&piece.bytes[..take]);
            slot = piece.next;
            if slot == first || out.len() == n {
                break;
            }
        }
        repeat(n, out);
    }
}

impl Kernel {
    pub(crate) fn spod(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap: Cap,
        lanes: usize,
        out: &mut Vec<u8>,
    ) -> bool {
        let Some((stats, mark, bits)) =
            self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out)
        else {
            return false;
        };
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
        );
        self.seed_table(stats.retained, mark, bits)
            .emit(lanes, n, out);
        true
    }

    fn seed_table(&self, m: usize, mark: u32, bits: u32) -> Table<'_> {
        Table {
            edges: &self.edges[..m],
            extra: &self.extra,
            spelling: &self.spelling,
            offsets: &self.offsets,
            inline: &self.inline16,
            lens: &self.lens,
            bits,
            mark: mark as usize,
        }
    }
}

// A deliberately forced column route for the moonshot gate. Its scanner checks
// every Wikidata block and corrupt stream independently of production admission.
#[cfg(all(feature = "spod", feature = "moonshot"))]
impl super::OpenCells {
    #[doc(hidden)]
    pub fn check_spod_column(&mut self, column: &[u8], p: u32, out: &mut Vec<u8>) {
        assert!(self.open.spod(
            &super::records_of(column),
            column.len(),
            p,
            Cap::NONE,
            4,
            out
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open::{emit_reference, records_of};

    fn compare(next: &[usize], mark: usize, n: usize, wide: bool) {
        let m = next.len();
        let bits = (usize::BITS - (m - 1).leading_zeros()).max(1);
        let spelling: Vec<_> = (0..35).collect();
        let offsets = [0, 3, 35];
        let codes: Vec<_> = (0..m)
            .map(|i| match i % 7 {
                0 => 256,
                1 => 257,
                _ => (i % 251) as u32,
            })
            .collect();
        let edges: Vec<_> = next
            .iter()
            .zip(&codes)
            .map(|(&j, &code)| j as u32 | if wide { 0 } else { code << bits })
            .collect();
        let extra = if wide { &codes[..] } else { &[] };
        let (mut inline, mut lens) = (Vec::new(), Vec::new());
        inline_words(&spelling, &offsets, &mut inline, &mut lens);
        let tab = Table {
            edges: &edges,
            extra,
            spelling: &spelling,
            offsets: &offsets,
            inline: &inline,
            lens: &lens,
            bits,
            mark,
        };
        let mut want = Vec::new();
        emit_reference(
            &edges,
            extra,
            &spelling,
            &offsets,
            n,
            mark as u32,
            bits,
            &mut want,
        );
        let mut got = vec![0xab; n + 91];
        for lanes in [1, 2, 4, 10] {
            tab.emit(lanes, n, &mut got);
            assert_eq!(got, want, "mark {mark}, m {m}, lanes {lanes}");
        }
    }

    #[test]
    fn splice_permuted_segments_and_primary_seed() {
        let m = 2053;
        let next: Vec<_> = (0..m).map(|i| (i + 997) % m).collect();
        for mark in [0, 511, 512, m - 1] {
            for wide in [false, true] {
                compare(&next, mark, 12000, wide);
            }
        }
    }

    #[test]
    fn sub_stride_periodic_and_disjoint_cycles() {
        compare(&[0], 0, 150, false);
        compare(&[2, 0, 1], 1, 512, true);
        // Even and odd cycles. Only even rows have regular seeds. Marking an
        // odd row must still recover the entire odd orbit and repeat it.
        let m = 2048;
        let next: Vec<_> = (0..m).map(|i| (i + 2) % m).collect();
        for mark in [0, 1, 513] {
            compare(&next, mark, 20000, false);
        }
    }

    /// bench/spod/bench.py verifies the manifest before passing each column and mark.
    #[test]
    #[ignore]
    fn authenticated_column() {
        use crate::bwt::{InverseBwt, Plain};
        let column = std::fs::read(std::env::var("SPOD_COLUMN").unwrap()).unwrap();
        let mark = std::env::var("SPOD_MARK").unwrap().parse().unwrap();
        let mut counts = [0; 256];
        for &b in &column {
            counts[b as usize] += 1;
        }
        let n = column.len();
        let (mut want, mut out) = (Vec::new(), Vec::new());
        Plain::default().invert(&column, &counts, mark, &mut want);
        let mut k = Kernel::default();
        assert!(k.spod(&records_of(&column), n, mark, Cap::NONE, 4, &mut out));
        assert_eq!(out, want);
        if std::env::var_os("SPOD_TIME_EMIT").is_some() {
            time_emit(&mut k, &column, mark, &want);
        }
    }

    // Pair the existing emit and the complete scoped walk + splice on the same
    // warm table, excluding construction. Validate outside every timed region.
    fn time_emit(k: &mut Kernel, column: &[u8], p: u32, want: &[u8]) {
        let n = column.len();
        let mut out = Vec::new();
        let (stats, mark, bits) = k
            .construct_capped::<true, true>(
                &records_of(column),
                n as u32,
                p,
                LIMIT,
                Cap::NONE,
                &mut out,
            )
            .unwrap();
        inline_words(&k.spelling, &k.offsets, &mut k.inline16, &mut k.lens);
        let t = k.seed_table(stats.retained, mark, bits);
        for round in 0..8 {
            for lanes in [2, 4, 8, 10] {
                for i in 0..3 {
                    let arm = (if round % 2 == 0 { i } else { 2 - i } + round / 2) % 3;
                    let start = std::time::Instant::now();
                    if arm == 2 {
                        t.emit(lanes, n, &mut out);
                    } else {
                        super::super::emit(
                            t.edges, t.extra, t.spelling, t.offsets, t.inline, t.lens, n, mark,
                            bits, &mut out,
                        );
                    }
                    let ns = start.elapsed().as_nanos();
                    assert_eq!(out, want);
                    println!("EMIT,{round},{lanes},{arm},{ns}");
                }
            }
        }
    }
}
