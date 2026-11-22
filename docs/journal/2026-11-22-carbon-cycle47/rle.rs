//! Phase 5's RLE1 undone: 4 equal bytes, then a byte counting 0-255 more of them. Scalar,
//! or (AVX2) the test for 4 equal bytes made at 32 positions at once, the stretches
//! between runs copied whole.

use super::{ErrorKind, Kernels};

/// `rle` with its runs undone, appended to `out`. Err if it ends with a run's 4 bytes and
/// no count: libbzip2 then reads a count past the block's end and refuses the block
pub fn unrle(rle: &[u8], out: &mut Vec<u8>, k: Kernels) -> Result<(), ErrorKind> {
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected (a `Kernels` invariant)
            return unsafe { unrle_avx2(rle, out) };
        }
    }
    let _ = k;
    unrle_scalar(rle, out, 0)
}

/// From `rle[i..]` (i at a run's start or a literal, never inside a run)
pub fn unrle_scalar(rle: &[u8], out: &mut Vec<u8>, mut i: usize) -> Result<(), ErrorKind> {
    let n = rle.len();
    out.reserve(n - i.min(n));
    while i < n {
        let c = rle[i];
        if i + 3 < n && rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
            let Some(&extra) = rle.get(i + 4) else {
                return Err(ErrorKind::RunUnfinished);
            };
            out.extend_from_slice(&[c, c, c, c]);
            out.resize(out.len() + extra as usize, c);
            i += 5;
        } else {
            out.push(c);
            i += 1;
        }
    }
    Ok(())
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn unrle_avx2(rle: &[u8], out: &mut Vec<u8>) -> Result<(), ErrorKind> {
    use std::arch::x86_64::*;
    let n = rle.len();
    out.reserve(n + 64);
    let p = rle.as_ptr();
    let mut i = 0;
    // whole 32-position windows while 35 bytes remain (SAFETY: every load is below i + 35)
    while i + 35 <= n {
        let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
        let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
        let v2 = _mm256_loadu_si256(p.add(i + 2) as *const __m256i);
        let v3 = _mm256_loadu_si256(p.add(i + 3) as *const __m256i);
        let eq = _mm256_and_si256(
            _mm256_cmpeq_epi8(v0, v1),
            _mm256_and_si256(_mm256_cmpeq_epi8(v0, v2), _mm256_cmpeq_epi8(v0, v3)),
        );
        let m = _mm256_movemask_epi8(eq) as u32;
        if m == 0 {
            out.extend_from_slice(&rle[i..i + 32]);
            i += 32;
            continue;
        }
        let j = i + m.trailing_zeros() as usize; // the first run's start
        out.extend_from_slice(&rle[i..j]);
        let c = rle[j];
        let Some(&extra) = rle.get(j + 4) else {
            return Err(ErrorKind::RunUnfinished);
        };
        out.extend_from_slice(&[c, c, c, c]);
        out.resize(out.len() + extra as usize, c);
        i = j + 5;
    }
    unrle_scalar(rle, out, i)
}

/// A run in a block's inverted bytes, as `unrle` reads them: its first byte's offset, the
/// byte, and its count (0-255 more of it)
pub(crate) type Run = (u32, u8, u8);

/// The bytes the in-place RLE1 takes its CRC in as it writes them (`unrle_in_place`): a
/// tile written is read again while it is still in the core's L2
const TILE: usize = 64 << 10;

/// RLE1 undone in place: `buf` holds a block's inverted bytes (`unrle`'s `rle`) and is left
/// holding exactly `unrle`'s output for them. Returns that output's CRC (`crc::crc32`'s),
/// taken a 64 KiB tile at a time as the tiles are written. Err as `unrle` errs (then `buf`'s
/// contents are unspecified).
///
/// `runs` is scratch (the runs found, `Run`); `stage` takes the inverted bytes where the runs
/// grow the block past twice its length (a block of runs only, such as one of a single
/// byte), which `unrle` then reads into `buf` from there: shifting them in place would write
/// the growth twice. The two buffers are swapped for it, so nothing is copied, and `buf`
/// ends with `stage`'s old allocation.
///
/// In place via zero-shift bipolar decomposition: let `d_r = dst_r - src_r` (`d_0 = 0`,
/// `d_{r+1} = d_r + extra_r - 1 >= d_r - 1`). Because `d` steps down by at most `1` per run,
/// `d` never crosses from `> 0` to `< 0` without hitting `d_v == 0` (`dst_v == src_v`) in
/// between, and `dst` is strictly monotone below `len`. Stretches with `d == 0` stay in place
/// (`0` copies), stretches with `d < 0` copy forward (`dst < src`), and positive zones
/// (`d > 0`) copy backward (`dst > src`) in a single pass without an upfront shift or `stage`
/// swap, streaming `64 KiB` CRC tiles as zones finalize.
pub(crate) fn unrle_in_place(
    buf: &mut Vec<u8>,
    runs: &mut Vec<Run>,
    stage: &mut Vec<u8>,
    k: Kernels,
) -> Result<u32, ErrorKind> {
    let n = buf.len();
    if n > u32::MAX as usize {
        // (no block is this long: `Run`'s offsets are 32 bits)
        std::mem::swap(buf, stage);
        buf.clear();
        unrle(stage, buf, k)?;
        return Ok(super::crc::crc32(buf, k));
    }
    find_runs(buf, runs, k)?;
    let m = runs.len();
    if m == 0 {
        return Ok(super::crc::crc32(buf, k));
    }
    let mut grown = 0isize;
    for &(_, _, extra) in runs.iter() {
        grown += extra as isize - 1;
    }
    let len = (n as isize + grown) as usize;
    if len > n {
        buf.reserve_exact(len - n);
        buf.resize(len, 0);
    }
    // the CRC register over buf[..at], whole tiles of the final bytes
    let (mut s, mut at) = (!0u32, 0usize);
    let tiles = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
        while end - *at >= TILE {
            *s = super::crc::update_with(*s, &buf[*at..*at + TILE], k);
            *at += TILE;
        }
    };
    tiles(&mut s, &mut at, buf, runs[0].0 as usize);

    let mut r = 0usize;
    let mut d = 0isize;
    while r <= m {
        let src_r = if r == 0 {
            0
        } else {
            runs[r - 1].0 as usize + 5
        };
        let end_r = if r < m { runs[r].0 as usize } else { n };
        if d < 0 && end_r > src_r {
            let dst_r = (src_r as isize + d) as usize;
            buf.copy_within(src_r..end_r, dst_r);
        }
        if r == m {
            break;
        }
        let (j_r, c_r, extra_r) = runs[r];
        let d_next = d + (extra_r as isize - 1);
        if d_next <= 0 {
            let r_start = (j_r as isize + d) as usize;
            let run_len = 4 + extra_r as usize;
            buf[r_start..r_start + run_len].fill(c_r);
            tiles(&mut s, &mut at, buf, r_start + run_len);
            d = d_next;
            r += 1;
            continue;
        }

        // Positive zone `r + 1 ..= v`: `d_q > 0` up to the first `v <= m` with `d_v == 0`
        // (or `v == m` with `d_m > 0`).
        let mut v = r + 1;
        let mut d_v = d_next;
        while v < m && d_v > 0 {
            d_v += runs[v].2 as isize - 1;
            v += 1;
        }
        if v == m && d_v > 0 {
            let src_m = runs[m - 1].0 as usize + 5;
            if n > src_m {
                buf.copy_within(src_m..n, (src_m as isize + d_v) as usize);
            }
        }
        let mut d_after = d_v;
        let mut q = v;
        while q > r + 1 {
            q -= 1;
            let (j_q, c_q, extra_q) = runs[q];
            let d_q = d_after - (extra_q as isize - 1);
            let src_q = runs[q - 1].0 as usize + 5;
            let end_q = j_q as usize;
            let r_start = (end_q as isize + d_q) as usize;
            buf[r_start..r_start + 4 + extra_q as usize].fill(c_q);
            if end_q > src_q {
                buf.copy_within(src_q..end_q, (src_q as isize + d_q) as usize);
            }
            d_after = d_q;
        }
        let r_start = (j_r as isize + d) as usize;
        buf[r_start..r_start + 4 + extra_r as usize].fill(c_r);

        let zone_end = if v < m {
            runs[v - 1].0 as usize + 5
        } else {
            len
        };
        tiles(&mut s, &mut at, buf, zone_end);
        if v == m {
            break;
        }
        r = v;
        d = 0;
    }

    buf.truncate(len);
    s = super::crc::update_with(s, &buf[at..], k);
    Ok(!s)
}

/// The runs of `rle`, in order, as `unrle` reads them (from the start, a run's 5 bytes taken
/// whole), into `runs`; Err where `unrle` errs (a run's 4 bytes at the end, with no count)
fn find_runs(rle: &[u8], runs: &mut Vec<Run>, k: Kernels) -> Result<(), ErrorKind> {
    runs.clear();
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected (a `Kernels` invariant)
            return unsafe { find_runs_avx2(rle, runs) };
        }
    }
    let _ = k;
    find_runs_scalar(rle, runs, 0)
}

/// `find_runs` from `rle[i..]` (i at a run's start or a literal, never inside a run)
fn find_runs_scalar(rle: &[u8], runs: &mut Vec<Run>, mut i: usize) -> Result<(), ErrorKind> {
    let n = rle.len();
    while i + 3 < n {
        let c = rle[i];
        if rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
            let Some(&extra) = rle.get(i + 4) else {
                return Err(ErrorKind::RunUnfinished);
            };
            runs.push((i as u32, c, extra));
            i += 5;
        } else {
            i += 1;
        }
    }
    Ok(())
}

/// `find_runs` with `unrle_avx2`'s test for 4 equal bytes at 32 positions at once
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn find_runs_avx2(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
    use std::arch::x86_64::*;
    let n = rle.len();
    let p = rle.as_ptr();
    let mut i = 0;
    // whole 32-position windows while 35 bytes remain (SAFETY: every load is below i + 35)
    while i + 35 <= n {
        let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
        let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
        let v2 = _mm256_loadu_si256(p.add(i + 2) as *const __m256i);
        let v3 = _mm256_loadu_si256(p.add(i + 3) as *const __m256i);
        let eq = _mm256_and_si256(
            _mm256_cmpeq_epi8(v0, v1),
            _mm256_and_si256(_mm256_cmpeq_epi8(v0, v2), _mm256_cmpeq_epi8(v0, v3)),
        );
        let m = _mm256_movemask_epi8(eq) as u32;
        if m == 0 {
            i += 32;
            continue;
        }
        let j = i + m.trailing_zeros() as usize; // the first run's start
        let Some(&extra) = rle.get(j + 4) else {
            return Err(ErrorKind::RunUnfinished);
        };
        runs.push((j as u32, rle[j], extra));
        i = j + 5;
    }
    find_runs_scalar(rle, runs, i)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    fn kernels() -> Vec<Kernels> {
        let mut v = vec![Kernels::scalar()];
        if Kernels::detect() != Kernels::scalar() {
            v.push(Kernels::detect());
        }
        v
    }

    /// `unrle_in_place` against `unrle` and `crc32` on `rle`, by every kernel: the same bytes
    /// and CRC, or the same error
    fn same(rle: &[u8]) {
        for k in kernels() {
            let mut want = Vec::new();
            let w = unrle(rle, &mut want, k);
            let (mut buf, mut runs, mut stage) = (rle.to_vec(), Vec::new(), Vec::new());
            let got = unrle_in_place(&mut buf, &mut runs, &mut stage, k);
            match w {
                Ok(()) => {
                    assert_eq!(
                        got,
                        Ok(crate::crc::crc32(&want, k)),
                        "len {} {k:?}",
                        rle.len()
                    );
                    assert!(buf == want, "len {} {k:?}", rle.len());
                }
                Err(e) => assert_eq!(got, Err(e), "len {} {k:?}", rle.len()),
            }
        }
    }

    /// Runs at the start, at the end, back to back, of counts 0 (the block shrinks), 1 (the
    /// same length) and 255; 4 equal bytes at the very end with no count (the error); runs
    /// across the AVX2 scan's 32-position windows; and 5 or more equal bytes, of which the
    /// fifth is the count
    #[test]
    fn miri_edge_runs() {
        let cases: Vec<Vec<u8>> = vec![
            vec![],
            b"a".to_vec(),
            b"aaa".to_vec(),
            b"aaaa".to_vec(),
            b"aaaa\0".to_vec(),
            b"aaaa\x01".to_vec(),
            b"aaaa\xff".to_vec(),
            b"xaaaa".to_vec(),
            b"aaaa\0bbbb\0cccc\0".to_vec(),
            b"aaaa\xffbbbb\0cccc\x07z".to_vec(),
            b"aaaaa".to_vec(),
            b"aaaaaaaaa".to_vec(),
            b"aaaaaaaaaa".to_vec(),
            [b"q".repeat(31), b"aaaa\x05".to_vec(), b"q".repeat(40)].concat(),
            [b"q".repeat(30), b"rrrr\0".to_vec(), b"s".repeat(3)].concat(),
            [b"0123".repeat(100), b"zzzz".to_vec()].concat(),
            [b"0123".repeat(100), b"zzzz\0".to_vec()].concat(),
        ];
        for c in &cases {
            same(c);
        }
    }

    /// `miri_edge_runs`'s runs at every offset: every run length 4..=40 of one byte, at
    /// every offset in a 64-byte window (across the AVX2 scan's windows)
    #[test]
    fn edge_runs_at_every_offset() {
        for len in 4..=40 {
            for at in 0..64 {
                let mut v: Vec<u8> = (0..at).map(|i| (i % 7) as u8 + b'a').collect();
                v.extend(std::iter::repeat(b'#').take(len));
                v.extend((0..50).map(|i| (i % 5) as u8 + b'A'));
                same(&v);
            }
        }
    }

    /// Random blocks of literals and runs, shrinking, growing and growing past twice their
    /// length (the staged path), up to 3 tiles long and across the tiles' edges
    #[test]
    fn random_blocks() {
        let mut r = Rng(0x9e37_79b9_7f4a_7c15);
        for round in 0..400 {
            let target = match round % 4 {
                0 => r.below(300) as usize,
                1 => r.below(70_000) as usize,
                2 => (1 << 16) * (1 + r.below(3) as usize) + r.below(64) as usize - 32,
                _ => r.below(3 * TILE as u64) as usize,
            };
            let density = 1 + r.below(200);
            let max_extra = [0u64, 1, 2, 8, 255][r.below(5) as usize];
            let mut v = Vec::with_capacity(target + 8);
            while v.len() < target {
                if r.below(density) == 0 {
                    let c = r.below(4) as u8;
                    v.extend([c; 4]);
                    v.push(r.below(max_extra + 1) as u8);
                } else {
                    v.push(r.below(256) as u8);
                }
            }
            same(&v);
        }
    }

    /// The tiles' edges exactly: a final length of each 64 KiB multiple and its neighbours,
    /// reached by runs that grow and runs that shrink
    #[test]
    fn tile_boundaries() {
        for tiles in 1..=3usize {
            for d in [-2isize, -1, 0, 1, 2] {
                let want = (tiles * TILE) as isize + d;
                for extra in [0u8, 1, 3] {
                    // literals, then runs of `extra` from 10 KiB on, each 4 + extra out
                    let mut v: Vec<u8> = (0..10_240).map(|i| (i % 251) as u8).collect();
                    let mut out_len = v.len() as isize;
                    while out_len + 4 + (extra as isize) < want {
                        v.extend([b'r'; 4]);
                        v.push(extra);
                        out_len += 4 + extra as isize;
                        v.push(b'.');
                        out_len += 1;
                    }
                    while out_len < want {
                        v.push(b'.');
                        out_len += 1;
                    }
                    same(&v);
                }
            }
        }
    }
}
