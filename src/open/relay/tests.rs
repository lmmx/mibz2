// The relay emit against the serial walk it replaces (`emit_reference`, the walk as shipped
// before the 16-byte copy): cut placement, the boundaries' restoration on tight cuts,
// several cycles and periodic walks, tiny blocks, degenerate cuts, and arbitrary maps (not
// permutations), where the relay must give the serial walk's bytes or leave the walk to it.
use super::super::{emit, emit_reference, inline_words, INLINE};
use super::*;

struct Lcg(u64);
impl Lcg {
    fn next(&mut self, below: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % below.max(1)
    }
}

/// A dictionary of `words` words: short ones (1 to 16 bytes, across the 16-byte copy) and
/// long ones (17 to 60, `LONG`), or with `short` only 1 to 3 bytes
fn dictionary(g: &mut Lcg, words: usize, short: bool) -> (Vec<u8>, Vec<u32>) {
    let mut spelling = Vec::new();
    let mut offsets = vec![0u32];
    for _ in 0..words {
        let len = if short {
            1 + g.next(3)
        } else {
            match g.next(4) {
                0 => 1 + g.next(4),
                1 => 13 + g.next(4),
                2 => 17 + g.next(44),
                _ => 1 + g.next(16),
            }
        };
        spelling.extend((0..len).map(|_| g.next(256) as u8));
        offsets.push(spelling.len() as u32);
    }
    (spelling, offsets)
}

fn bits_of(m: usize) -> u32 {
    (usize::BITS - (m - 1).leading_zeros()).max(1)
}

/// `edges` for the successor map `to` with random codes below `codes`
fn wire(g: &mut Lcg, to: &[u32], codes: usize, bits: u32) -> Vec<u32> {
    to.iter()
        .map(|&t| {
            let code = if g.next(3) == 0 {
                g.next(256)
            } else {
                g.next(codes)
            } as u32;
            (code << bits) | t
        })
        .collect()
}

/// One walk by the shipped `emit` (into a dirty buffer) and by the reference: equal, and
/// whether the relay took it
fn check(edges: &[u32], spelling: &[u8], offsets: &[u32], n: usize, mark: u32, what: &str) -> bool {
    let bits = bits_of(edges.len());
    let (mut inline16, mut lens) = (Vec::new(), Vec::new());
    inline_words(spelling, offsets, &mut inline16, &mut lens);
    let mut want = vec![0xEE; 3];
    emit_reference(edges, &[], spelling, offsets, n, mark, bits, &mut want);
    let mut got = vec![0xA5; n / 2 + 7];
    let before = RELAYED.with(|r| r.get());
    emit(
        edges,
        &[],
        spelling,
        offsets,
        &inline16,
        &lens,
        n,
        mark,
        bits,
        &mut got,
    );
    assert!(got == want, "{what}: m {}, n {n}, mark {mark}", edges.len());
    RELAYED.with(|r| r.get()) > before
}

/// A random permutation of `m` rows made of `cycles` cycles (the marked row's first)
fn cycles(g: &mut Lcg, m: usize, cycles: usize) -> Vec<u32> {
    let mut order: Vec<u32> = (0..m as u32).collect();
    for i in (1..m).rev() {
        order.swap(i, g.next(i + 1));
    }
    // cut the order into `cycles` pieces, each closed on itself
    let mut ends: Vec<usize> = (1..cycles).map(|_| 1 + g.next(m - 1)).collect();
    ends.push(m);
    ends.sort_unstable();
    ends.dedup();
    let mut to = vec![0u32; m];
    let mut start = 0;
    for &end in &ends {
        for i in start..end {
            let next = if i + 1 == end { start } else { i + 1 };
            to[order[i] as usize] = order[next];
        }
        start = end;
    }
    to
}

#[test]
fn cut_placement() {
    for m in [
        1024usize, 1025, 1500, 2047, 2048, 2049, 4097, 65_536, 65_537, 262_143, 900_000,
    ] {
        let bits = bits_of(m);
        let mut g = Lcg(m as u64);
        for mark in [0u32, 1, (m - 1) as u32, (m / 2) as u32, g.next(m) as u32] {
            let (shift, low, n) = cuts(m, bits, mark);
            let cm = (1u32 << shift) - 1;
            let want: Vec<u32> = (0..m as u32).filter(|j| (j ^ mark) & cm == 0).collect();
            let got: Vec<u32> = (0..n as u32).map(|q| (q << shift) | low).collect();
            assert_eq!(got, want, "m {m}, mark {mark}");
            assert!(got.contains(&mark));
            // 2^(k-1) to 2^k cuts, k = 8 (cycle 8's 2^(k-1) + 1 holds only for low bits 0:
            // at m = 1,025 and a mark with low bits 7, the cuts are 7, 15, ..., 1,023)
            assert!((128..=256).contains(&n), "m {m}: {n} cuts");
        }
    }
}

/// Single cycles, several cycles, marks anywhere, `n` below, at and above the cycle's bytes
/// (the clamp, and `repeat` of a periodic walk), dictionaries across the 16-byte copy and the
/// long path
#[test]
fn relay_matches_serial_on_permutations() {
    let mut g = Lcg(0x7265_6c61);
    let mut relayed = 0;
    let mut cases = 0;
    for &m in &[
        1024usize, 1025, 1031, 2047, 2048, 3000, 4097, 9000, 70_000, 262_145,
    ] {
        for &k in &[1usize, 2, 3, 15] {
            for &short in &[false, true] {
                let to = cycles(&mut g, m, k);
                let words = 1 + g.next(3000);
                let (spelling, offsets) = dictionary(&mut g, words, short);
                let edges = wire(&mut g, &to, 256 + words, bits_of(m));
                let mark = g.next(m) as u32;
                // the bytes of the marked row's cycle, by the reference unclamped
                let mut full = Vec::new();
                emit_reference(
                    &edges,
                    &[],
                    &spelling,
                    &offsets,
                    64 * m,
                    mark,
                    bits_of(m),
                    &mut full,
                );
                let cyc = full.len();
                for n in [
                    1,
                    2,
                    15,
                    16,
                    17,
                    1000,
                    cyc / 3,
                    cyc.saturating_sub(1),
                    cyc,
                    cyc + 1,
                    3 * cyc + 5,
                ] {
                    let n = n.clamp(1, 900_000);
                    relayed += usize::from(check(&edges, &spelling, &offsets, n, mark, "perm"));
                    cases += 1;
                }
            }
        }
    }
    // the relay takes nearly every walk (a mark whose cycle meets no other cut leaves lane 0
    // alone, still the relay's)
    assert!(relayed * 10 > cases * 9, "{relayed} of {cases} relayed");
}

/// Tight cuts: one cycle in row order, so a cut every 2^shift steps (4 rows at m = 1,024),
/// with words of 1 to 3 bytes, of exactly 16, and long ones: segments shorter than the
/// 16-byte overrun, each piece's head overrun by the lane before
#[test]
fn boundary_restoration_on_tight_cuts() {
    let mut g = Lcg(0x7469_6768);
    for &m in &[1024usize, 1100, 2048, 2049, 8192] {
        let to: Vec<u32> = (0..m as u32).map(|j| (j + 1) % m as u32).collect();
        for case in 0..6 {
            let (spelling, offsets) = match case {
                0 | 1 => dictionary(&mut g, 50, true),
                2 => {
                    // every word exactly 16 bytes
                    let sp: Vec<u8> = (0..50 * 16).map(|_| g.next(256) as u8).collect();
                    (sp, (0..=50).map(|i| 16 * i).collect())
                }
                _ => dictionary(&mut g, 200, false),
            };
            let codes = 256 + offsets.len() - 1;
            let edges: Vec<u32> = to
                .iter()
                .map(|&t| {
                    let code = if case == 1 {
                        g.next(256)
                    } else {
                        256 + g.next(codes - 256)
                    };
                    ((code as u32) << bits_of(m)) | t
                })
                .collect();
            for mark in [0u32, 3, (m - 1) as u32, g.next(m) as u32] {
                for n in [m / 2, m, 5 * m, 900_000] {
                    assert!(check(&edges, &spelling, &offsets, n, mark, "tight"));
                }
            }
        }
    }
}

/// Degenerate cuts: the cuts on a cycle of their own, each segment one step (every cut's
/// successor the next cut); and the marked row alone on its cycle, every other cut on a
/// cycle the walk never meets (all scouted, none chained)
#[test]
fn degenerate_cuts() {
    let mut g = Lcg(0x6465_6765);
    for &m in &[1024usize, 1500, 4096, 70_001] {
        let bits = bits_of(m);
        let (spelling, offsets) = dictionary(&mut g, 300, false);
        let codes = 256 + 300;
        for mark in [0u32, 5, (m - 1) as u32] {
            let (shift, low, n_cuts) = cuts(m, bits, mark);
            let row = |q: usize| ((q as u32) << shift) | low;
            let is_cut = |j: u32| (j ^ mark) & ((1 << shift) - 1) == 0;
            // the cuts in one cycle of one-step segments; the other rows in another
            let mut to = vec![0u32; m];
            for q in 0..n_cuts {
                to[row(q) as usize] = row((q + 1) % n_cuts);
            }
            let rest: Vec<u32> = (0..m as u32).filter(|&j| !is_cut(j)).collect();
            for i in 0..rest.len() {
                to[rest[i] as usize] = rest[(i + 1) % rest.len()];
            }
            let edges = wire(&mut g, &to, codes, bits);
            for n in [1, 7, 100, 900_000] {
                assert!(check(&edges, &spelling, &offsets, n, mark, "one-step cuts"));
            }
            // the mark alone on a cycle through non-cut rows; every other cut elsewhere
            let mut to: Vec<u32> = (0..m as u32).collect();
            let mut others: Vec<u32> = rest.clone();
            let lap: Vec<u32> = std::iter::once(mark)
                .chain(others.drain(..rest.len() / 2))
                .collect();
            let far: Vec<u32> = (0..n_cuts)
                .map(row)
                .filter(|&c| c != mark)
                .chain(others)
                .collect();
            for v in [&lap, &far] {
                for i in 0..v.len() {
                    to[v[i] as usize] = v[(i + 1) % v.len()];
                }
            }
            let edges = wire(&mut g, &to, codes, bits);
            for n in [1, 50, 900_000] {
                check(&edges, &spelling, &offsets, n, mark, "mark alone");
            }
        }
    }
}

/// Tiny blocks: fewer than `RELAY_MIN` rows walk serially; at it, the relay; and `n` of a
/// byte or a few on a full table
#[test]
fn tiny_blocks() {
    let mut g = Lcg(0x7469_6e79);
    let (spelling, offsets) = dictionary(&mut g, 40, false);
    for m in [1usize, 2, 3, 255, 1023, 1024] {
        let to = cycles(&mut g, m, 1);
        let edges = wire(&mut g, &to, 256 + 40, bits_of(m));
        for n in [1, 2, 3, 16, 17, 2000] {
            let r = check(&edges, &spelling, &offsets, n, (m / 2) as u32, "tiny");
            assert_eq!(r, m >= RELAY_MIN, "m {m}");
        }
    }
}

/// Arbitrary maps, not permutations (several rows to one, cycles without cuts, a walk that
/// meets a cut twice): the serial walk's bytes, whoever walks
#[test]
fn arbitrary_maps_match_serial() {
    let mut g = Lcg(0x6172_6269);
    for case in 0..300 {
        let m = 1024 + g.next(5000);
        let bits = bits_of(m);
        let k0 = 1 + g.next(4);
        let mut to = cycles(&mut g, m, k0);
        // some rows redirected anywhere
        let k = [1, 5, m / 10, m][case % 4];
        for _ in 0..k {
            to[g.next(m)] = g.next(m) as u32;
        }
        let words = 1 + g.next(500);
        let (spelling, offsets) = dictionary(&mut g, words, case % 2 == 0);
        let edges = wire(&mut g, &to, 256 + words, bits);
        let mark = g.next(m) as u32;
        for n in [1 + g.next(100), 1 + g.next(20_000), 900_000] {
            check(&edges, &spelling, &offsets, n, mark, "arbitrary");
        }
    }
}

/// The kernel itself on columns whose retained rows pass `RELAY_MIN`: periodic texts (the
/// walk's cycle shorter than the block, `repeat`), random runs, and the relay's use
#[test]
fn kernel_columns_through_the_relay() {
    use super::super::{Kernel, LIMIT};
    use crate::bwt::{InverseBwt, Plain};
    use crate::runs::Runs;
    let plain = |l: &[u8], p: u32| {
        let mut counts = [0u32; 256];
        for &c in l {
            counts[c as usize] += 1;
        }
        let mut out = Vec::new();
        Plain::default().invert(l, &counts, p, &mut out);
        out
    };
    let mut g = Lcg(0x6b65_726e);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    let before = RELAYED.with(|r| r.get());
    let mut columns: Vec<Vec<u8>> = Vec::new();
    // random runs
    for &(n, a, mean) in &[
        (70_000usize, 4usize, 3usize),
        (200_000, 60, 2),
        (900_000, 30, 4),
    ] {
        let mut l = Vec::with_capacity(n);
        while l.len() < n {
            let c = g.next(a) as u8;
            let w = 1 + g.next(2 * mean);
            l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
        }
        columns.push(l);
    }
    // periodic texts: the column of a primitive word repeated, each byte of the word's own
    // column repeated (its rotations sorted plainly: random bytes, short common prefixes)
    for &(unit, reps) in &[(5000usize, 3usize), (1999, 7), (30_000, 2)] {
        let u: Vec<u8> = (0..unit).map(|_| b'a' + g.next(20) as u8).collect();
        let mut rot: Vec<usize> = (0..unit).collect();
        rot.sort_by(|&a, &b| {
            (0..unit)
                .map(|i| u[(a + i) % unit])
                .cmp((0..unit).map(|i| u[(b + i) % unit]))
        });
        columns.push(
            rot.iter()
                .flat_map(|&r| std::iter::repeat_n(u[(r + unit - 1) % unit], reps))
                .collect(),
        );
    }
    for l in &columns {
        let r = Runs::of_column(l);
        for p in [0, l.len() as u32 / 3, l.len() as u32 - 1] {
            out.clear();
            out.extend(plain(l, p).iter().map(|b| !b));
            k.run::<true, true>(r.records(), l.len(), p, LIMIT, &mut out);
            assert!(out == plain(l, p), "n {}, row {p}", l.len());
        }
    }
    let used = RELAYED.with(|r| r.get()) - before;
    assert!(used >= 3 * columns.len() / 2, "the relay took {used} walks");
    let _ = INLINE;
}

/// A walk from the mark into a cycle that meets no cut (a map, not a permutation): the
/// serial walk stops after `m` steps short of `n`, and so must the relay's bytes
#[test]
fn walk_into_a_cycle_without_cuts() {
    let mut g = Lcg(0x6e6f_6375);
    for &m in &[1024usize, 4096, 20_000] {
        let bits = bits_of(m);
        let (spelling, offsets) = dictionary(&mut g, 50, true);
        for mark in [0u32, 9, (m - 1) as u32] {
            let (shift, _, _) = cuts(m, bits, mark);
            let is_cut = |j: u32| (j ^ mark) & ((1 << shift) - 1) == 0;
            let free: Vec<u32> = (0..m as u32).filter(|&j| !is_cut(j)).collect();
            // the mark, a tail of 40 free rows, then a loop of 30 free rows; every other row
            // (the other cuts among them) a cycle of its own
            let mut to: Vec<u32> = (0..m as u32).map(|j| (j + 1) % m as u32).collect();
            let path: Vec<u32> = std::iter::once(mark)
                .chain(free[..70].iter().copied())
                .collect();
            for i in 0..path.len() - 1 {
                to[path[i] as usize] = path[i + 1];
            }
            to[path[70] as usize] = path[41];
            let edges = wire(&mut g, &to, 256 + 50, bits);
            for n in [100, 5 * m, 900_000] {
                check(&edges, &spelling, &offsets, n, mark, "no-cut loop");
            }
        }
    }
}

/// (Miri) the relay on one small table: tight cuts, a periodic walk, the clamp to `n`
#[test]
fn miri_relay_small() {
    let mut g = Lcg(0x6d69_7269);
    let m = RELAY_MIN;
    let (spelling, offsets) = dictionary(&mut g, 20, false);
    let to: Vec<u32> = (0..m as u32).map(|j| (j + 1) % m as u32).collect();
    let edges = wire(&mut g, &to, 256 + 20, bits_of(m));
    for n in [37, 3 * m] {
        assert!(check(&edges, &spelling, &offsets, n, 5, "miri"));
    }
}
