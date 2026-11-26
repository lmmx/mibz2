#![forbid(unsafe_code)]
//! (research only, `single-pass-rle1-crc`) Census of RLE1 block structure, per-pass breakdown
//! of shipped `rle::unrle_in_place` + `crc::update_with`, and 7-repeat A/B benchmark of
//! single-write-pass and single-pass tiled RLE1+CRC32 designs.
//!
//!     cargo run --release --example rle1_crc_census -- corpora/wikidata.bz2 [EXTRA_BZ2...]
//!
//! Optional flags:
//!     --repeats N      interleaved repeats for timing (default 7)
//!     --skip-bench     run self-tests and Part 1 census only
//!
//! Zero `src/` edits, `#![forbid(unsafe_code)]`, 100% byte-identical output and CRC32 verified
//! against `mibz2::rle::unrle` + `mibz2::crc::crc32` on every synthetic edge case and every
//! block of every input corpus.

use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{bwt, crc, mtf, rle, ErrorKind, Kernels};

const SHIPPED_TILE: usize = 64 << 10;
const POOLED_FROM: usize = 64 << 10;
const X4_MULTIPLIER: usize = 4;
const QUEUE_SPILL_LIMIT: usize = 64 << 10;

type Run = (u32, u8, u8);

#[inline]
fn prod_pool_cap(n: usize) -> usize {
    if n >= POOLED_FROM {
        n + n / 16 + 64
    } else {
        n
    }
}

// ---------------------------------------------------------------------------
// GF(2) CRC-32/BZIP2 combination (for backward in-place expansion with tiled CRC)
// ---------------------------------------------------------------------------

struct CrcCombine {
    /// `pow_byte[k]` is the 32x32 GF(2) matrix multiplying a CRC register by
    /// `x^(8 * 2^k) mod P(x)` in CRC-32/BZIP2's non-reflected MSB-first basis.
    pow_byte: [[u32; 32]; 32],
}

impl CrcCombine {
    fn new() -> Self {
        let mut step1 = [0u32; 32];
        for (j, col) in step1.iter_mut().enumerate() {
            let v = 1u32 << j;
            *col = if v & 0x8000_0000 != 0 {
                (v << 1) ^ crc::POLY
            } else {
                v << 1
            };
        }
        // Square 3 times: 1 bit -> 2 -> 4 -> 8 bits (1 byte)
        let mut byte1 = Self::mat_sq(&step1);
        byte1 = Self::mat_sq(&byte1);
        byte1 = Self::mat_sq(&byte1);

        let mut pow_byte = [[0u32; 32]; 32];
        pow_byte[0] = byte1;
        for k in 1..32 {
            pow_byte[k] = Self::mat_sq(&pow_byte[k - 1]);
        }
        Self { pow_byte }
    }

    #[inline]
    fn mat_vec(mat: &[u32; 32], mut v: u32) -> u32 {
        let mut sum = 0u32;
        let mut i = 0;
        while v != 0 {
            if v & 1 != 0 {
                sum ^= mat[i];
            }
            v >>= 1;
            i += 1;
        }
        sum
    }

    fn mat_sq(a: &[u32; 32]) -> [u32; 32] {
        let mut out = [0u32; 32];
        for (j, col) in out.iter_mut().enumerate() {
            *col = Self::mat_vec(a, a[j]);
        }
        out
    }

    /// Combine `c0 = update_with(init, a)` and `c1 = update_with(0, b)` where `len1 = b.len()`.
    #[inline]
    fn combine(&self, mut c0: u32, c1: u32, mut len1: usize) -> u32 {
        let mut k = 0;
        while len1 != 0 {
            if len1 & 1 != 0 {
                c0 = Self::mat_vec(&self.pow_byte[k], c0);
            }
            len1 >>= 1;
            k += 1;
        }
        c0 ^ c1
    }
}

// ---------------------------------------------------------------------------
// Safe 4-equal-byte scanners (bounds-check-free auto-vec 32B, SWAR u64, scalar)
// ---------------------------------------------------------------------------

/// Bounds-check-free safe 32-byte chunked scanner for 4 consecutive equal bytes.
/// Under `-C target-cpu=native` LLVM compiles the inner loop to the exact AVX2 sequence
/// (`vmovdqu` + 3x `vpcmpeqb` + 2x `vpand` + `vpmovmskb`) of `src/rle.rs::find_runs_avx2`;
/// under baseline x86-64 it compiles to 16-byte SSE2 (`pcmpeqb` + `pand` + `pmovmskb`).
#[inline]
fn find_next_run_autovec32(s: &[u8]) -> Option<usize> {
    let n = s.len();
    if n >= 35 {
        let m = (n - 3) & !31;
        let c0 = s[..m].chunks_exact(32);
        let c1 = s[1..m + 1].chunks_exact(32);
        let c2 = s[2..m + 2].chunks_exact(32);
        let c3 = s[3..m + 3].chunks_exact(32);
        for (chunk_idx, (((a, b), c), d)) in c0.zip(c1).zip(c2).zip(c3).enumerate() {
            let mut any = 0u8;
            for idx in 0..32 {
                any |= ((a[idx] == b[idx]) & (a[idx] == c[idx]) & (a[idx] == d[idx])) as u8;
            }
            if any != 0 {
                for idx in 0..32 {
                    if a[idx] == b[idx] && a[idx] == c[idx] && a[idx] == d[idx] {
                        return Some(chunk_idx * 32 + idx);
                    }
                }
            }
        }
        return find_next_run_scalar(s, m);
    }
    find_next_run_scalar(s, 0)
}

#[inline]
fn find_next_run_swar64(s: &[u8]) -> Option<usize> {
    const LO: u64 = 0x0101_0101_0101_0101;
    const HI: u64 = 0x8080_8080_8080_8080;
    let n = s.len();
    let mut i = 0usize;
    while i + 11 <= n {
        let w0 = u64::from_le_bytes(s[i..i + 8].try_into().unwrap());
        let w1 = u64::from_le_bytes(s[i + 1..i + 9].try_into().unwrap());
        let w2 = u64::from_le_bytes(s[i + 2..i + 10].try_into().unwrap());
        let w3 = u64::from_le_bytes(s[i + 3..i + 11].try_into().unwrap());
        let diff = (w0 ^ w1) | (w0 ^ w2) | (w0 ^ w3);
        let z = diff.wrapping_sub(LO) & !diff & HI;
        if z != 0 {
            return Some(i + (z.trailing_zeros() >> 3) as usize);
        }
        i += 8;
    }
    find_next_run_scalar(s, i)
}

#[inline]
fn find_next_run_scalar(s: &[u8], mut i: usize) -> Option<usize> {
    let n = s.len();
    while i + 3 < n {
        let c = s[i];
        if s[i + 1] == c && s[i + 2] == c && s[i + 3] == c {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_runs_with(
    rle: &[u8],
    runs: &mut Vec<Run>,
    scanner: fn(&[u8]) -> Option<usize>,
) -> Result<(), ErrorKind> {
    runs.clear();
    let n = rle.len();
    let mut i = 0usize;
    while i + 3 < n {
        match scanner(&rle[i..]) {
            None => break,
            Some(rel) => {
                let j = i + rel;
                let Some(&extra) = rle.get(j + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                runs.push((j as u32, rle[j], extra));
                i = j + 5;
            }
        }
    }
    Ok(())
}

#[inline]
fn find_runs_fast(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
    find_runs_with(rle, runs, find_next_run_autovec32)
}

/// Fused 32-byte scan + copy from `src[..win_len]` into `dst[..win_len]` (disjoint slices),
/// using `src[..scan_len]` (where `scan_len = (win_len + 3).min(remaining_src)`) for lookahead.
/// Returns `(copied_bytes, run_found_at_offset_if_any)`.
#[inline]
fn fused_scan_copy_oop(
    src_with_lookahead: &[u8],
    win_len: usize,
    dst: &mut [u8],
) -> (usize, Option<usize>) {
    let scan_len = src_with_lookahead.len();
    let mut i = 0usize;
    if win_len >= 32 && scan_len >= 35 {
        let m = win_len.min(scan_len - 3) & !31;
        let c0 = src_with_lookahead[..m].chunks_exact(32);
        let c1 = src_with_lookahead[1..m + 1].chunks_exact(32);
        let c2 = src_with_lookahead[2..m + 2].chunks_exact(32);
        let c3 = src_with_lookahead[3..m + 3].chunks_exact(32);
        let out_chunks = dst[..m].chunks_exact_mut(32);
        for (chunk_idx, ((((a, b), c), d), out)) in
            c0.zip(c1).zip(c2).zip(c3).zip(out_chunks).enumerate()
        {
            let mut any = 0u8;
            for idx in 0..32 {
                any |= ((a[idx] == b[idx]) & (a[idx] == c[idx]) & (a[idx] == d[idx])) as u8;
            }
            if any != 0 {
                for idx in 0..32 {
                    if a[idx] == b[idx] && a[idx] == c[idx] && a[idx] == d[idx] {
                        let base = chunk_idx * 32;
                        out[..idx].copy_from_slice(&a[..idx]);
                        return (base + idx, Some(base + idx));
                    }
                }
            }
            out.copy_from_slice(a);
        }
        i = m;
    }
    while i < win_len {
        if i + 3 < scan_len {
            let c = src_with_lookahead[i];
            if src_with_lookahead[i + 1] == c
                && src_with_lookahead[i + 2] == c
                && src_with_lookahead[i + 3] == c
            {
                return (i, Some(i));
            }
        }
        dst[i] = src_with_lookahead[i];
        i += 1;
    }
    (win_len, None)
}

// (Phase A in single_pass_inplace uses find_next_run_autovec32 + copy_within directly,
// which leverages glibc AVX2/ERMS memmove at ~28.7 GB/s instead of a manual byte-slice loop.)

// ---------------------------------------------------------------------------
// Arm 1: Faithful mirror of shipped `src/rle.rs::unrle_in_place`
// ---------------------------------------------------------------------------

fn shipped_unrle_in_place(
    buf: &mut Vec<u8>,
    runs: &mut Vec<Run>,
    stage: &mut Vec<u8>,
    k: Kernels,
) -> Result<u32, ErrorKind> {
    let n = buf.len();
    if n > u32::MAX as usize {
        std::mem::swap(buf, stage);
        buf.clear();
        rle::unrle(stage, buf, k)?;
        return Ok(crc::crc32(buf, k));
    }
    find_runs_fast(buf, runs)?;
    let Some(&(first, _, _)) = runs.first() else {
        return Ok(crc::crc32(buf, k));
    };
    let (mut grown, mut shift) = (0isize, 0isize);
    for &(_, _, extra) in runs.iter() {
        grown += extra as isize - 1;
        shift = shift.max(grown);
    }
    let (shift, len) = (shift as usize, (n as isize + grown) as usize);
    if shift > n {
        std::mem::swap(buf, stage);
        buf.clear();
        rle::unrle(stage, buf, k)?;
        return Ok(crc::crc32(buf, k));
    }
    let first = first as usize;
    if shift > 0 {
        buf.reserve_exact(shift);
        buf.resize(n + shift, 0);
        buf.copy_within(first..n, first + shift);
    }
    let (mut s, mut at) = (!0u32, 0usize);
    let tiles = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
        while end - *at >= SHIPPED_TILE {
            *s = crc::update_with(*s, &buf[*at..*at + SHIPPED_TILE], k);
            *at += SHIPPED_TILE;
        }
    };
    tiles(&mut s, &mut at, buf, first);
    let (mut src, mut dst) = (first + shift, first);
    for &(j, c, extra) in runs.iter() {
        let from = j as usize + shift;
        if from > src {
            if dst != src {
                buf.copy_within(src..from, dst);
            }
            dst += from - src;
        }
        let run = 4 + extra as usize;
        buf[dst..dst + run].fill(c);
        dst += run;
        src = from + 5;
        tiles(&mut s, &mut at, buf, dst);
    }
    if n + shift > src && dst != src {
        buf.copy_within(src..n + shift, dst);
    }
    buf.truncate(len);
    s = crc::update_with(s, &buf[at..], k);
    Ok(!s)
}

/// Separate expand pass (without CRC) for per-pass breakdown of shipped A9.
fn shipped_expand_no_crc(
    buf: &mut Vec<u8>,
    runs: &[Run],
    stage: &mut Vec<u8>,
    k: Kernels,
) -> Result<(), ErrorKind> {
    let n = buf.len();
    let Some(&(first, _, _)) = runs.first() else {
        return Ok(());
    };
    let (mut grown, mut shift) = (0isize, 0isize);
    for &(_, _, extra) in runs.iter() {
        grown += extra as isize - 1;
        shift = shift.max(grown);
    }
    let (shift, len) = (shift as usize, (n as isize + grown) as usize);
    if shift > n {
        std::mem::swap(buf, stage);
        buf.clear();
        rle::unrle(stage, buf, k)?;
        return Ok(());
    }
    let first = first as usize;
    if shift > 0 {
        buf.reserve_exact(shift);
        buf.resize(n + shift, 0);
        buf.copy_within(first..n, first + shift);
    }
    let (mut src, mut dst) = (first + shift, first);
    for &(j, c, extra) in runs.iter() {
        let from = j as usize + shift;
        if from > src {
            if dst != src {
                buf.copy_within(src..from, dst);
            }
            dst += from - src;
        }
        let run = 4 + extra as usize;
        buf[dst..dst + run].fill(c);
        dst += run;
        src = from + 5;
    }
    if n + shift > src && dst != src {
        buf.copy_within(src..n + shift, dst);
    }
    buf.truncate(len);
    Ok(())
}

// ---------------------------------------------------------------------------
// Arm 2 & 3: Backward in-place expansion when `shift > 0 && min_grown >= 0`
// ---------------------------------------------------------------------------

fn backward_unrle_in_place(
    buf: &mut Vec<u8>,
    runs: &mut Vec<Run>,
    stage: &mut Vec<u8>,
    combiner: Option<&CrcCombine>,
    tile_crcs: &mut Vec<(u32, usize)>,
    tile: usize,
    k: Kernels,
) -> Result<u32, ErrorKind> {
    let n = buf.len();
    find_runs_fast(buf, runs)?;
    let Some(&(first, _, _)) = runs.first() else {
        return Ok(crc::crc32(buf, k));
    };
    let (mut grown, mut shift, mut min_grown) = (0isize, 0isize, 0isize);
    for &(_, _, extra) in runs.iter() {
        grown += extra as isize - 1;
        shift = shift.max(grown);
        min_grown = min_grown.min(grown);
    }
    let (shift, len) = (shift as usize, (n as isize + grown) as usize);
    if shift > n {
        std::mem::swap(buf, stage);
        buf.clear();
        rle::unrle(stage, buf, k)?;
        return Ok(crc::crc32(buf, k));
    }
    let first = first as usize;
    if shift == 0 {
        let (mut s, mut at) = (!0u32, 0usize);
        let tiles = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
            while end - *at >= tile {
                *s = crc::update_with(*s, &buf[*at..*at + tile], k);
                *at += tile;
            }
        };
        tiles(&mut s, &mut at, buf, first);
        let (mut src, mut dst) = (first, first);
        for &(j, c, extra) in runs.iter() {
            let from = j as usize;
            if from > src {
                if dst != src {
                    buf.copy_within(src..from, dst);
                }
                dst += from - src;
            }
            let run = 4 + extra as usize;
            buf[dst..dst + run].fill(c);
            dst += run;
            src = from + 5;
            tiles(&mut s, &mut at, buf, dst);
        }
        if n > src && dst != src {
            buf.copy_within(src..n, dst);
        }
        buf.truncate(len);
        s = crc::update_with(s, &buf[at..], k);
        return Ok(!s);
    }
    if min_grown >= 0 {
        buf.reserve_exact(len - n);
        buf.resize(len, 0);
        if let Some(comb) = combiner {
            tile_crcs.clear();
            let mut src_end = n;
            let mut dst_end = len;
            let mut crc_end = len;
            for &(j, c, extra) in runs.iter().rev() {
                let run_src_end = j as usize + 5;
                let lit_len = src_end - run_src_end;
                if lit_len > 0 {
                    let dst_start = dst_end - lit_len;
                    if dst_start != run_src_end {
                        buf.copy_within(run_src_end..src_end, dst_start);
                    }
                    dst_end = dst_start;
                }
                let run_len = 4 + extra as usize;
                let dst_start = dst_end - run_len;
                buf[dst_start..dst_end].fill(c);
                dst_end = dst_start;
                src_end = j as usize;
                while crc_end - dst_end >= tile {
                    let t_start = crc_end - tile;
                    let tc = crc::update_with(0, &buf[t_start..crc_end], k);
                    tile_crcs.push((tc, tile));
                    crc_end = t_start;
                }
            }
            debug_assert_eq!(dst_end, first);
            if crc_end > first {
                let tc = crc::update_with(0, &buf[first..crc_end], k);
                tile_crcs.push((tc, crc_end - first));
            }
            let mut s = crc::update_with(!0, &buf[..first], k);
            for &(tc, tlen) in tile_crcs.iter().rev() {
                s = comb.combine(s, tc, tlen);
            }
            return Ok(!s);
        } else {
            let mut src_end = n;
            let mut dst_end = len;
            for &(j, c, extra) in runs.iter().rev() {
                let run_src_end = j as usize + 5;
                let lit_len = src_end - run_src_end;
                if lit_len > 0 {
                    let dst_start = dst_end - lit_len;
                    if dst_start != run_src_end {
                        buf.copy_within(run_src_end..src_end, dst_start);
                    }
                    dst_end = dst_start;
                }
                let run_len = 4 + extra as usize;
                let dst_start = dst_end - run_len;
                buf[dst_start..dst_end].fill(c);
                dst_end = dst_start;
                src_end = j as usize;
            }
            debug_assert_eq!(dst_end, first);
            return Ok(crc::crc32(buf, k));
        }
    }
    // Fallback for mixed `min_grown < 0 && shift > 0`
    buf.reserve_exact(shift);
    buf.resize(n + shift, 0);
    buf.copy_within(first..n, first + shift);
    let (mut s, mut at) = (!0u32, 0usize);
    let tiles = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
        while end - *at >= tile {
            *s = crc::update_with(*s, &buf[*at..*at + tile], k);
            *at += tile;
        }
    };
    tiles(&mut s, &mut at, buf, first);
    let (mut src, mut dst) = (first + shift, first);
    for &(j, c, extra) in runs.iter() {
        let from = j as usize + shift;
        if from > src {
            if dst != src {
                buf.copy_within(src..from, dst);
            }
            dst += from - src;
        }
        let run = 4 + extra as usize;
        buf[dst..dst + run].fill(c);
        dst += run;
        src = from + 5;
        tiles(&mut s, &mut at, buf, dst);
    }
    if n + shift > src && dst != src {
        buf.copy_within(src..n + shift, dst);
    }
    buf.truncate(len);
    s = crc::update_with(s, &buf[at..], k);
    Ok(!s)
}

// ---------------------------------------------------------------------------
// Arm 4..7: One-pass out-of-place RLE1 + tiled CRC into `stage`
// ---------------------------------------------------------------------------

/// Fused one-pass out-of-place RLE1 + tiled CRC into pre-sized `stage`, then swap `buf`/`stage`.
/// Uses `fused_scan_copy_oop` so scan and copy run in ONE 32-byte SIMD loop directly into a
/// pre-sized slice (zero per-chunk `Vec::extend_from_slice` checks), followed by L1d-hot CRC.
fn single_pass_oop_fused(
    buf: &mut Vec<u8>,
    stage: &mut Vec<u8>,
    tile: usize,
    k: Kernels,
) -> Result<u32, ErrorKind> {
    let src = buf.as_slice();
    let n = src.len();
    let init_len = prod_pool_cap(n).max(n + 264);
    if stage.len() < init_len {
        stage.resize(init_len, 0);
    }
    let mut i = 0usize;
    let mut dst = 0usize;
    let mut crc_s = !0u32;
    let mut crc_at = 0usize;
    let chunk_cap = if tile > 0 { tile } else { 32 << 10 };

    while i < n {
        let win_len = chunk_cap.min(n - i);
        if dst + win_len + 264 > stage.len() {
            let grow = (stage.len() * 2).max(dst + win_len + 4096);
            stage.resize(grow, 0);
        }
        let scan_end = (i + win_len + 3).min(n);
        let (copied, run_rel) =
            fused_scan_copy_oop(&src[i..scan_end], win_len, &mut stage[dst..dst + win_len]);
        dst += copied;
        if let Some(rel) = run_rel {
            let j = i + rel;
            let Some(&extra) = src.get(j + 4) else {
                return Err(ErrorKind::RunUnfinished);
            };
            let c = src[j];
            let run_len = 4 + extra as usize;
            stage[dst..dst + run_len].fill(c);
            dst += run_len;
            i = j + 5;
        } else {
            i += copied;
        }
        if tile > 0 && dst - crc_at >= tile {
            crc_s = crc::update_with(crc_s, &stage[crc_at..dst], k);
            crc_at = dst;
        }
    }
    stage.truncate(dst);
    let crc = if tile > 0 {
        crc_s = crc::update_with(crc_s, &stage[crc_at..dst], k);
        !crc_s
    } else {
        crc::crc32(stage, k)
    };
    std::mem::swap(buf, stage);
    Ok(crc)
}

// ---------------------------------------------------------------------------
// Arm 8..12: Single-pass tiled IN-PLACE RLE1 + CRC32 (L1d lookahead queue)
// ---------------------------------------------------------------------------

/// Single-pass tiled IN-PLACE RLE1 + CRC32:
/// - `single_write_shift = false`: Phase B uses a 16 KiB L1d lookahead queue (`buf -> q -> buf`).
/// - `single_write_shift = true`: Phase B uses single-write tiled upward `copy_within` (saving
///   only `lag <= 512 B` at tile edges) on literal stretches so even `shift > 0` does 1 write pass.
fn single_pass_inplace(
    buf: &mut Vec<u8>,
    q: &mut Vec<u8>,
    stage: &mut Vec<u8>,
    tile: usize,
    single_write_shift: bool,
    k: Kernels,
) -> Result<u32, ErrorKind> {
    let n = buf.len();
    let mut src = 0usize;
    let mut dst = 0usize;
    let mut crc_s = !0u32;
    let mut crc_at = 0usize;
    let chunk_cap = if tile > 0 { tile } else { 32 << 10 };

    // Phase A: dst <= src (zero-copy prefix + in-place compaction)
    let mut pending_run: Option<(u8, usize)> = None;
    while src < n {
        let win_end = src.saturating_add(chunk_cap).min(n);
        let scan_end = if win_end < n { (win_end + 3).min(n) } else { n };
        match find_next_run_autovec32(&buf[src..scan_end]) {
            None => {
                let lit_len = win_end - src;
                if dst != src {
                    buf.copy_within(src..win_end, dst);
                }
                dst += lit_len;
                src = win_end;
            }
            Some(rel) => {
                let j = src + rel;
                let Some(&extra) = buf.get(j + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                let c = buf[j];
                let lit_len = j - src;
                if lit_len > 0 {
                    if dst != src {
                        buf.copy_within(src..j, dst);
                    }
                    dst += lit_len;
                }
                let next_src = j + 5;
                let run_len = 4 + extra as usize;
                if dst + run_len <= next_src {
                    if dst == j {
                        if extra == 1 {
                            buf[j + 4] = c;
                        }
                    } else {
                        buf[dst..dst + run_len].fill(c);
                    }
                    dst += run_len;
                    src = next_src;
                } else {
                    src = next_src;
                    pending_run = Some((c, run_len));
                    break;
                }
            }
        }
        if tile > 0 && dst - crc_at >= tile {
            crc_s = crc::update_with(crc_s, &buf[crc_at..dst], k);
            crc_at = dst;
        }
    }

    // Phase B: L1d lookahead queue (entered only on `shift > 0` blocks)
    if let Some((first_c, first_run_len)) = pending_run {
        q.clear();
        let mut q_head = 0usize;
        let mut read_pos = src;
        let mut appending = false;
        let fused = single_write_shift;

        let ensure_and_emit_fill = |buf: &mut Vec<u8>,
                                    q: &mut Vec<u8>,
                                    q_head: &mut usize,
                                    read_pos: &mut usize,
                                    dst: &mut usize,
                                    appending: &mut bool,
                                    c: u8,
                                    run_len: usize| {
            if *appending {
                buf.resize(buf.len() + run_len, c);
                *dst += run_len;
                return;
            }
            if *dst + run_len > *read_pos {
                if *read_pos < n {
                    let need_end = (*dst + run_len).min(n);
                    // In fused mode, only evacuate the exact `need_end` overlap so `q` stays tiny
                    // (<= 193 B on wikidata), allowing single-write tiled upward copy_within!
                    let take_end = if fused {
                        need_end
                    } else {
                        (*read_pos + chunk_cap).max(need_end).min(n)
                    };
                    if *q_head > 0 {
                        let rem = q.len() - *q_head;
                        q.copy_within(*q_head.., 0);
                        q.truncate(rem);
                        *q_head = 0;
                    }
                    q.extend_from_slice(&buf[*read_pos..take_end]);
                    *read_pos = take_end;
                }
                if *dst + run_len > *read_pos {
                    debug_assert_eq!(*read_pos, n);
                    buf.truncate(*dst);
                    *appending = true;
                    buf.resize(*dst + run_len, c);
                    *dst += run_len;
                    return;
                }
            }
            buf[*dst..*dst + run_len].fill(c);
            *dst += run_len;
        };

        let ensure_and_emit_slice = |buf: &mut Vec<u8>,
                                     q: &mut Vec<u8>,
                                     read_pos: &mut usize,
                                     dst: &mut usize,
                                     appending: &mut bool,
                                     slice_start: usize,
                                     slice_len: usize| {
            if slice_len == 0 {
                return;
            }
            if *appending {
                buf.extend_from_slice(&q[slice_start..slice_start + slice_len]);
                *dst += slice_len;
                return;
            }
            if *dst + slice_len > *read_pos {
                if *read_pos < n {
                    let need_end = (*dst + slice_len).min(n);
                    let take_end = if fused && q.len() <= 512 {
                        need_end
                    } else {
                        (*read_pos + chunk_cap).max(need_end).min(n)
                    };
                    q.extend_from_slice(&buf[*read_pos..take_end]);
                    *read_pos = take_end;
                }
                if *dst + slice_len > *read_pos {
                    debug_assert_eq!(*read_pos, n);
                    buf.truncate(*dst);
                    *appending = true;
                    buf.extend_from_slice(&q[slice_start..slice_start + slice_len]);
                    *dst += slice_len;
                    return;
                }
            }
            buf[*dst..*dst + slice_len].copy_from_slice(&q[slice_start..slice_start + slice_len]);
            *dst += slice_len;
        };

        ensure_and_emit_fill(
            buf,
            q,
            &mut q_head,
            &mut read_pos,
            &mut dst,
            &mut appending,
            first_c,
            first_run_len,
        );
        if tile > 0 && dst - crc_at >= tile {
            crc_s = crc::update_with(crc_s, &buf[crc_at..dst], k);
            crc_at = dst;
        }

        // Tiny scratch for single-write tiled upward copy_within when `fused == true`
        let mut next_saved = [0u8; 512];

        loop {
            let avail = q.len() - q_head;

            // Fast single-write path for `fused == true` when `dst == read_pos` and `0 < lag <= 512`:
            // Scan `buf[read_pos..win_end]` directly in place, save only `lag` boundary bytes,
            // and do ONE `buf.copy_within` (no `buf -> q -> buf` double copy!).
            if fused
                && !appending
                && dst == read_pos
                && avail > 0
                && avail <= 512
                && read_pos + avail + 8 <= n
            {
                let lag = avail;
                let mut q_has_run = false;
                for off in 0..lag {
                    let b0 = q[q_head + off];
                    let get_b = |pos: usize| -> u8 {
                        if pos < lag {
                            q[q_head + pos]
                        } else {
                            buf[read_pos + (pos - lag)]
                        }
                    };
                    if get_b(off + 1) == b0 && get_b(off + 2) == b0 && get_b(off + 3) == b0 {
                        q_has_run = true;
                        break;
                    }
                }
                if !q_has_run {
                    let max_body = (n - read_pos - lag).min(chunk_cap);
                    let scan_end = (read_pos + max_body + 3).min(n);
                    let body_len = match find_next_run_autovec32(&buf[read_pos..scan_end]) {
                        None => max_body,
                        Some(rel) => rel.min(max_body),
                    };
                    if body_len >= lag.max(64) {
                        next_saved[..lag]
                            .copy_from_slice(&buf[read_pos + body_len..read_pos + body_len + lag]);
                        buf.copy_within(read_pos..read_pos + body_len, read_pos + lag);
                        buf[dst..dst + lag].copy_from_slice(&q[q_head..q_head + lag]);
                        q.clear();
                        q.extend_from_slice(&next_saved[..lag]);
                        q_head = 0;
                        dst += lag + body_len;
                        read_pos = dst;
                        if tile > 0 && dst - crc_at >= tile {
                            crc_s = crc::update_with(crc_s, &buf[crc_at..dst], k);
                            crc_at = dst;
                        }
                        continue;
                    }
                }
            }

            if read_pos < n && avail < 5 {
                if q_head > 0 {
                    q.copy_within(q_head.., 0);
                    q.truncate(avail);
                    q_head = 0;
                }
                let take_end = if fused && avail <= 512 {
                    (read_pos + 64).min(n)
                } else {
                    (read_pos + chunk_cap).min(n)
                };
                q.extend_from_slice(&buf[read_pos..take_end]);
                read_pos = take_end;
            }
            let avail = q.len() - q_head;
            if avail == 0 {
                if read_pos == n {
                    break;
                }
                continue;
            }

            if avail > QUEUE_SPILL_LIMIT {
                stage.clear();
                stage.reserve(avail + (n - read_pos));
                stage.extend_from_slice(&q[q_head..]);
                stage.extend_from_slice(&buf[read_pos..n]);
                buf.truncate(dst);
                let mut si = 0usize;
                let sn = stage.len();
                while si < sn {
                    let win_end = si.saturating_add(chunk_cap).min(sn);
                    let scan_end = if win_end < sn {
                        (win_end + 3).min(sn)
                    } else {
                        sn
                    };
                    match find_next_run_autovec32(&stage[si..scan_end]) {
                        None => {
                            buf.extend_from_slice(&stage[si..win_end]);
                            dst += win_end - si;
                            si = win_end;
                        }
                        Some(rel) => {
                            let j = si + rel;
                            let Some(&extra) = stage.get(j + 4) else {
                                return Err(ErrorKind::RunUnfinished);
                            };
                            if j > si {
                                buf.extend_from_slice(&stage[si..j]);
                                dst += j - si;
                            }
                            let c = stage[j];
                            let run_len = 4 + extra as usize;
                            buf.resize(buf.len() + run_len, c);
                            dst += run_len;
                            si = j + 5;
                        }
                    }
                    if tile > 0 && dst - crc_at >= tile {
                        crc_s = crc::update_with(crc_s, &buf[crc_at..dst], k);
                        crc_at = dst;
                    }
                }
                appending = true;
                break;
            }

            let scan_limit = if read_pos < n { q.len() - 4 } else { q.len() };
            let win_end = q_head.saturating_add(chunk_cap).min(scan_limit);
            let scan_end = if win_end < q.len() {
                (win_end + 3).min(q.len())
            } else {
                q.len()
            };

            match find_next_run_autovec32(&q[q_head..scan_end]) {
                None => {
                    let lit_len = win_end - q_head;
                    let lit_start = q_head;
                    q_head = win_end;
                    ensure_and_emit_slice(
                        buf,
                        q,
                        &mut read_pos,
                        &mut dst,
                        &mut appending,
                        lit_start,
                        lit_len,
                    );
                    if read_pos < n && q_head == scan_limit {
                        let rem = q.len() - q_head;
                        q.copy_within(q_head.., 0);
                        q.truncate(rem);
                        q_head = 0;
                        let take_end = if fused && rem <= 512 {
                            (read_pos + 64).min(n)
                        } else {
                            (read_pos + chunk_cap).min(n)
                        };
                        q.extend_from_slice(&buf[read_pos..take_end]);
                        read_pos = take_end;
                    }
                }
                Some(rel) => {
                    let p = q_head + rel;
                    if read_pos < n && p >= scan_limit {
                        let lit_len = p - q_head;
                        let lit_start = q_head;
                        q_head = p;
                        ensure_and_emit_slice(
                            buf,
                            q,
                            &mut read_pos,
                            &mut dst,
                            &mut appending,
                            lit_start,
                            lit_len,
                        );
                        let rem = q.len() - q_head;
                        q.copy_within(q_head.., 0);
                        q.truncate(rem);
                        q_head = 0;
                        let take_end = if fused && rem <= 512 {
                            (read_pos + 64).min(n)
                        } else {
                            (read_pos + chunk_cap).min(n)
                        };
                        q.extend_from_slice(&buf[read_pos..take_end]);
                        read_pos = take_end;
                    } else {
                        let Some(&extra) = q.get(p + 4) else {
                            return Err(ErrorKind::RunUnfinished);
                        };
                        let c = q[p];
                        let lit_len = p - q_head;
                        let lit_start = q_head;
                        q_head = p + 5;
                        ensure_and_emit_slice(
                            buf,
                            q,
                            &mut read_pos,
                            &mut dst,
                            &mut appending,
                            lit_start,
                            lit_len,
                        );
                        ensure_and_emit_fill(
                            buf,
                            q,
                            &mut q_head,
                            &mut read_pos,
                            &mut dst,
                            &mut appending,
                            c,
                            4 + extra as usize,
                        );
                    }
                }
            }
            if tile > 0 && dst - crc_at >= tile {
                crc_s = crc::update_with(crc_s, &buf[crc_at..dst], k);
                crc_at = dst;
            }
        }
        if !appending {
            buf.truncate(dst);
        }
    } else {
        buf.truncate(dst);
    }

    let final_crc = if tile > 0 {
        crc_s = crc::update_with(crc_s, &buf[crc_at..], k);
        !crc_s
    } else {
        crc::crc32(buf, k)
    };
    Ok(final_crc)
}

// ---------------------------------------------------------------------------
// Synthetic self-test suite (exactness + edge cases)
// ---------------------------------------------------------------------------

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

fn check_all_arms(rle_in: &[u8], comb: &CrcCombine, k: Kernels, label: &str) {
    let mut want = Vec::new();
    let want_res = rle::unrle(rle_in, &mut want, k).map(|()| crc::crc32(&want, k));

    let mut runs = Vec::new();
    let mut stage = Vec::new();
    let mut q = Vec::new();
    let mut tile_crcs = Vec::new();

    {
        let mut buf = rle_in.to_vec();
        let got = shipped_unrle_in_place(&mut buf, &mut runs, &mut stage, k);
        assert_eq!(got, want_res, "shipped_inplace crc/err on {label}");
        if want_res.is_ok() {
            assert_eq!(buf, want, "shipped_inplace bytes on {label}");
        }
    }
    {
        let mut buf = rle_in.to_vec();
        let got = backward_unrle_in_place(
            &mut buf,
            &mut runs,
            &mut stage,
            None,
            &mut tile_crcs,
            SHIPPED_TILE,
            k,
        );
        assert_eq!(got, want_res, "backward_sep crc/err on {label}");
        if want_res.is_ok() {
            assert_eq!(buf, want, "backward_sep bytes on {label}");
        }
    }
    for &tile in &[4096usize, 16 << 10, 32 << 10] {
        let mut buf = rle_in.to_vec();
        let got = backward_unrle_in_place(
            &mut buf,
            &mut runs,
            &mut stage,
            Some(comb),
            &mut tile_crcs,
            tile,
            k,
        );
        assert_eq!(got, want_res, "backward_tiled({tile}) crc/err on {label}");
        if want_res.is_ok() {
            assert_eq!(buf, want, "backward_tiled({tile}) bytes on {label}");
        }
    }
    for &tile in &[0usize, 16 << 10, 32 << 10, 64 << 10] {
        let mut buf = rle_in.to_vec();
        let got = single_pass_oop_fused(&mut buf, &mut stage, tile, k);
        assert_eq!(got, want_res, "sp_oop_fused({tile}) crc/err on {label}");
        if want_res.is_ok() {
            assert_eq!(buf, want, "sp_oop_fused({tile}) bytes on {label}");
        }
    }
    for &fused in &[false, true] {
        for &tile in &[0usize, 4096, 16 << 10, 32 << 10] {
            for &pooled in &[false, true] {
                let mut buf = if pooled {
                    let mut v = Vec::with_capacity(prod_pool_cap(rle_in.len()));
                    v.extend_from_slice(rle_in);
                    v
                } else {
                    rle_in.to_vec()
                };
                let got = single_pass_inplace(&mut buf, &mut q, &mut stage, tile, fused, k);
                assert_eq!(
                    got, want_res,
                    "sp_inplace(tile={tile}, fused={fused}, pooled={pooled}) crc/err on {label}"
                );
                if want_res.is_ok() {
                    assert_eq!(
                        buf, want,
                        "sp_inplace(tile={tile}, fused={fused}, pooled={pooled}) bytes on {label}"
                    );
                }
            }
        }
    }
}

fn run_self_tests(comb: &CrcCombine, k: Kernels) {
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    let data: Vec<u8> = (0..200_000).map(|_| rng.next() as u8).collect();
    let whole = crc::crc32(&data, k);
    for &cut in &[
        0usize, 1, 7, 127, 128, 4096, 16384, 65535, 65536, 199_999, 200_000,
    ] {
        let c0 = crc::update_with(!0, &data[..cut], k);
        let c1 = crc::update_with(0, &data[cut..], k);
        assert_eq!(!comb.combine(c0, c1, data.len() - cut), whole);
    }

    let edge_cases: Vec<Vec<u8>> = vec![
        vec![],
        b"a".to_vec(),
        b"aaa".to_vec(),
        b"aaaa".to_vec(),
        b"aaaa\0".to_vec(),
        b"aaaa\x01".to_vec(),
        b"aaaa\x02".to_vec(),
        b"aaaa\xff".to_vec(),
        b"xaaaa".to_vec(),
        b"aaaa\0bbbb\0cccc\0".to_vec(),
        b"aaaa\xffbbbb\0cccc\x07z".to_vec(),
        b"aaaa\0bbbb\0cccc\xff".to_vec(),
        b"aaaaa".to_vec(),
        b"aaaaaaaaa".to_vec(),
        b"aaaaaaaaaa".to_vec(),
        [b"q".repeat(31), b"aaaa\x05".to_vec(), b"q".repeat(40)].concat(),
        [b"q".repeat(30), b"rrrr\0".to_vec(), b"s".repeat(3)].concat(),
        [b"0123".repeat(100), b"zzzz".to_vec()].concat(),
        [b"0123".repeat(100), b"zzzz\0".to_vec()].concat(),
        // Mixed min_grown < 0 && shift > 0 with long literal stretches between runs
        [
            b"aaaa\0".repeat(200),
            b"0123456789".repeat(2000),
            b"bbbb\xff".repeat(2),
            b"9876543210".repeat(3000),
            b"cccc\0".repeat(500),
            b"abcdefghij".repeat(2000),
            b"dddd\x80".repeat(5),
            b"klmnopqrst".repeat(2000),
        ]
        .concat(),
        // Pathological massive expansion (`shift > n` and `q > QUEUE_SPILL_LIMIT`)
        b"zzzz\xff".repeat(400),
    ];
    for (idx, c) in edge_cases.iter().enumerate() {
        let mut r1 = Vec::new();
        let mut r2 = Vec::new();
        let mut r3 = Vec::new();
        let e1 = find_runs_with(c, &mut r1, find_next_run_autovec32);
        let e2 = find_runs_with(c, &mut r2, find_next_run_swar64);
        let e3 = find_runs_with(c, &mut r3, |s| find_next_run_scalar(s, 0));
        assert_eq!(e1, e3, "autovec32 err mismatch edge {idx}");
        assert_eq!(e2, e3, "swar64 err mismatch edge {idx}");
        if e3.is_ok() {
            assert_eq!(r1, r3, "autovec32 runs mismatch edge {idx}");
            assert_eq!(r2, r3, "swar64 runs mismatch edge {idx}");
        }
        check_all_arms(c, comb, k, &format!("edge#{idx}"));
    }

    for len in 4..=40 {
        for at in 0..64 {
            let mut v: Vec<u8> = (0..at).map(|i| (i % 7) as u8 + b'a').collect();
            v.extend(std::iter::repeat(b'#').take(len));
            v.extend((0..50).map(|i| (i % 5) as u8 + b'A'));
            check_all_arms(&v, comb, k, &format!("offset len={len} at={at}"));
        }
    }

    for &t in &[4096usize, 16 << 10, 32 << 10] {
        for off in -4isize..=4 {
            let prefix = (t as isize + off).max(0) as usize;
            for &extra in &[0u8, 1, 2, 20, 255] {
                let mut v: Vec<u8> = (0..prefix).map(|i| (i % 251) as u8).collect();
                v.extend_from_slice(&[b'Z', b'Z', b'Z', b'Z', extra]);
                v.extend((0..500).map(|i| (i % 241) as u8));
                check_all_arms(&v, comb, k, &format!("tile={t} off={off} extra={extra}"));
            }
        }
    }

    let mut r = Rng(0x9e37_79b9_7f4a_7c15);
    for round in 0..120 {
        let target = match round % 4 {
            0 => r.below(300) as usize,
            1 => r.below(40_000) as usize,
            2 => (16 << 10) * (1 + r.below(3) as usize) + r.below(32) as usize - 16,
            _ => r.below(150_000) as usize,
        };
        let density = 1 + r.below(200);
        let max_extra = [0u64, 1, 2, 12, 255][r.below(5) as usize];
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
        check_all_arms(&v, comb, k, &format!("random#{round}"));
    }
}

// ---------------------------------------------------------------------------
// Corpus loading & Part 1 Census
// ---------------------------------------------------------------------------

struct BlockData {
    inv: Vec<u8>,
    crc: u32,
    dec_len: usize,
    run_count: usize,
    extra_eq_0: usize,
    extra_eq_1: usize,
    extra_ge_2: usize,
    first_run: usize,
    first_mut_run: usize,
    shift: usize,
    total_grown: isize,
    min_grown: isize,
}

struct Corpus {
    name: String,
    blocks: Vec<BlockData>,
}

fn load_corpus(path: &Path, k: Kernels) -> Corpus {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let compressed = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut sc = Scanner::new(SliceSource(&compressed));
    let mut syms = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut tt = bwt::TVector::default();
    let mut runs = Vec::new();
    let mut dec = Vec::new();
    let mut blocks = Vec::new();

    while let Event::Block { symbols, .. } = sc.next(syms).expect("scan block") {
        let n = mtf::mtf(&symbols, &mut ll, &mut counts, k).expect("mtf");
        bwt::build(&ll[..n], &counts, &mut tt);
        let mut inv = Vec::with_capacity(n);
        bwt::walk(&tt, symbols.orig_ptr, &mut inv);

        find_runs_fast(&inv, &mut runs).expect("find_runs");
        let (mut grown, mut shift, mut min_grown) = (0isize, 0isize, 0isize);
        let (mut e0, mut e1, mut e2) = (0usize, 0usize, 0usize);
        let mut first_mut_run = n;
        for &(j, _, extra) in &runs {
            match extra {
                0 => {
                    e0 += 1;
                    if first_mut_run == n {
                        first_mut_run = j as usize;
                    }
                }
                1 => e1 += 1,
                _ => {
                    e2 += 1;
                    if first_mut_run == n {
                        first_mut_run = j as usize;
                    }
                }
            }
            grown += extra as isize - 1;
            shift = shift.max(grown);
            min_grown = min_grown.min(grown);
        }
        let first_run = runs.first().map_or(n, |r| r.0 as usize);

        dec.clear();
        rle::unrle(&inv, &mut dec, k).expect("unrle");
        assert_eq!(dec.len(), (n as isize + grown) as usize);
        let got_crc = crc::crc32(&dec, k);
        assert_eq!(got_crc, symbols.crc, "CRC mismatch in {name}");

        blocks.push(BlockData {
            inv,
            crc: symbols.crc,
            dec_len: dec.len(),
            run_count: runs.len(),
            extra_eq_0: e0,
            extra_eq_1: e1,
            extra_ge_2: e2,
            first_run,
            first_mut_run,
            shift: shift as usize,
            total_grown: grown,
            min_grown,
        });
        syms = symbols.syms;
    }

    Corpus { name, blocks }
}

fn pct(num: usize, den: usize) -> f64 {
    if den == 0 {
        0.0
    } else {
        100.0 * (num as f64) / (den as f64)
    }
}

fn quantile_usize(vals: &[usize], q: f64) -> usize {
    if vals.is_empty() {
        return 0;
    }
    let mut s = vals.to_vec();
    s.sort_unstable();
    let idx = ((s.len() - 1) as f64 * q).round() as usize;
    s[idx.min(s.len() - 1)]
}

fn quantile_isize(vals: &[isize], q: f64) -> isize {
    if vals.is_empty() {
        return 0;
    }
    let mut s = vals.to_vec();
    s.sort_unstable();
    let idx = ((s.len() - 1) as f64 * q).round() as usize;
    s[idx.min(s.len() - 1)]
}

fn quantile_f64(vals: &[f64], q: f64) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    let mut s = vals.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    let idx = ((s.len() - 1) as f64 * q).round() as usize;
    s[idx.min(s.len() - 1)]
}

fn print_census(c: &Corpus) {
    let nb = c.blocks.len();
    let full_nb = c
        .blocks
        .iter()
        .filter(|b| b.inv.len() >= POOLED_FROM)
        .count();
    let total_n: usize = c.blocks.iter().map(|b| b.inv.len()).sum();
    let total_len: usize = c.blocks.iter().map(|b| b.dec_len).sum();

    let mut runs0_b = 0usize;
    let mut runs0_bytes = 0usize;
    let mut shift0_b = 0usize;
    let mut shift0_bytes = 0usize;
    let mut shift0_all_e1_b = 0usize;
    let mut shift0_all_e1_bytes = 0usize;
    let mut shift_gt0_b = 0usize;
    let mut shift_gt0_bytes = 0usize;
    let mut shift_gt_n_b = 0usize;
    let mut shift_gt_n_bytes = 0usize;
    let mut min_ge0_b = 0usize;
    let mut min_ge0_bytes = 0usize;
    let mut shift_gt0_min_ge0_b = 0usize;
    let mut shift_gt0_min_ge0_bytes = 0usize;
    let mut shift_gt0_min_lt0_b = 0usize;
    let mut shift_gt0_min_lt0_bytes = 0usize;

    let mut realloc_exact = 0usize;
    let mut realloc_prod_fresh = 0usize;
    let mut realloc_prod_recycled = 0usize;
    let mut recycled_cap = 0usize;

    let mut tot_runs = 0usize;
    let mut tot_e0 = 0usize;
    let mut tot_e1 = 0usize;
    let mut tot_e2 = 0usize;

    let mut v_runs = Vec::with_capacity(nb);
    let mut v_shift = Vec::with_capacity(nb);
    let mut v_grown = Vec::with_capacity(nb);
    let mut v_min_grown = Vec::with_capacity(nb);
    let mut v_ratio = Vec::with_capacity(nb);
    let mut v_first_frac = Vec::new();
    let mut v_first_mut_frac = Vec::new();

    for b in &c.blocks {
        let n = b.inv.len();
        tot_runs += b.run_count;
        tot_e0 += b.extra_eq_0;
        tot_e1 += b.extra_eq_1;
        tot_e2 += b.extra_ge_2;

        if b.run_count == 0 {
            runs0_b += 1;
            runs0_bytes += n;
        } else if b.shift == 0 {
            shift0_b += 1;
            shift0_bytes += n;
            if b.extra_eq_0 == 0 && b.extra_ge_2 == 0 {
                shift0_all_e1_b += 1;
                shift0_all_e1_bytes += n;
            }
        } else {
            shift_gt0_b += 1;
            shift_gt0_bytes += n;
            if b.shift > n {
                shift_gt_n_b += 1;
                shift_gt_n_bytes += n;
            }
            if b.min_grown >= 0 {
                shift_gt0_min_ge0_b += 1;
                shift_gt0_min_ge0_bytes += n;
            } else {
                shift_gt0_min_lt0_b += 1;
                shift_gt0_min_lt0_bytes += n;
            }
        }
        if b.min_grown >= 0 {
            min_ge0_b += 1;
            min_ge0_bytes += n;
        }
        if b.shift > 0 && b.shift <= n {
            if n + b.shift > n {
                realloc_exact += 1;
            }
            let fresh_cap = prod_pool_cap(n);
            if n + b.shift > fresh_cap {
                realloc_prod_fresh += 1;
            }
            let eff_cap = if n >= POOLED_FROM {
                recycled_cap.max(fresh_cap)
            } else {
                n
            };
            if n + b.shift > eff_cap {
                realloc_prod_recycled += 1;
            }
        }
        if n >= POOLED_FROM {
            recycled_cap = recycled_cap.max(prod_pool_cap(n));
        }

        v_runs.push(b.run_count);
        v_shift.push(b.shift);
        v_grown.push(b.total_grown);
        v_min_grown.push(b.min_grown);
        v_ratio.push(if n == 0 {
            1.0
        } else {
            b.dec_len as f64 / n as f64
        });
        if b.run_count > 0 && n > 0 {
            v_first_frac.push(b.first_run as f64 / n as f64);
            v_first_mut_frac.push(b.first_mut_run as f64 / n as f64);
        }
    }

    println!("\n=== CENSUS: {} ===", c.name);
    println!(
        "blocks: {} (full >=64KiB: {}, runts: {}) | pre-RLE1: {} B ({:.2} MiB) | post-RLE1: {} B ({:.2} MiB) | ratio: {:.6}x",
        nb,
        full_nb,
        nb - full_nb,
        total_n,
        total_n as f64 / (1024.0 * 1024.0),
        total_len,
        total_len as f64 / (1024.0 * 1024.0),
        total_len as f64 / total_n.max(1) as f64
    );
    println!(
        "  runs == 0:                 {:4}/{:4} ({:5.1}% blocks, {:5.1}% bytes)",
        runs0_b,
        nb,
        pct(runs0_b, nb),
        pct(runs0_bytes, total_n)
    );
    println!(
        "  runs > 0, shift == 0:      {:4}/{:4} ({:5.1}% blocks, {:5.1}% bytes)  [all extra==1: {}/{} ({:.1}% B)]",
        shift0_b,
        nb,
        pct(shift0_b, nb),
        pct(shift0_bytes, total_n),
        shift0_all_e1_b,
        nb,
        pct(shift0_all_e1_bytes, total_n)
    );
    println!(
        "  shift > 0 (4-pass):        {:4}/{:4} ({:5.1}% blocks, {:5.1}% bytes)  [shift > n stage-swap: {}/{} ({:.1}% B)]",
        shift_gt0_b,
        nb,
        pct(shift_gt0_b, nb),
        pct(shift_gt0_bytes, total_n),
        shift_gt_n_b,
        nb,
        pct(shift_gt_n_bytes, total_n)
    );
    println!(
        "    of shift > 0, min>=0:    {:4}/{:4} ({:5.1}% blocks, {:5.1}% bytes)  [backward-inplace eligible]",
        shift_gt0_min_ge0_b,
        nb,
        pct(shift_gt0_min_ge0_b, nb),
        pct(shift_gt0_min_ge0_bytes, total_n)
    );
    println!(
        "    of shift > 0, min<0:     {:4}/{:4} ({:5.1}% blocks, {:5.1}% bytes)  [mixed shrink+grow]",
        shift_gt0_min_lt0_b,
        nb,
        pct(shift_gt0_min_lt0_b, nb),
        pct(shift_gt0_min_lt0_bytes, total_n)
    );
    println!(
        "  min_grown >= 0 (all):      {:4}/{:4} ({:5.1}% blocks, {:5.1}% bytes)",
        min_ge0_b,
        nb,
        pct(min_ge0_b, nb),
        pct(min_ge0_bytes, total_n)
    );
    println!(
        "  reserve_exact reallocs (among 0<shift<=n): fresh exact cap={}/{} ({:.1}%) | prod pool cap={}/{} ({:.1}%) | recycled pool cap={}/{} ({:.1}%)",
        realloc_exact, nb, pct(realloc_exact, nb),
        realloc_prod_fresh, nb, pct(realloc_prod_fresh, nb),
        realloc_prod_recycled, nb, pct(realloc_prod_recycled, nb),
    );
    println!(
        "  runs total: {} (mean {:.1}/blk) | extra==0: {} ({:.1}%) | extra==1: {} ({:.1}%) | extra>=2: {} ({:.1}%)",
        tot_runs,
        tot_runs as f64 / nb.max(1) as f64,
        tot_e0, pct(tot_e0, tot_runs),
        tot_e1, pct(tot_e1, tot_runs),
        tot_e2, pct(tot_e2, tot_runs),
    );
    println!(
        "  dist run_count:  min={} p50={} p90={} p95={} p99={} max={}",
        quantile_usize(&v_runs, 0.0),
        quantile_usize(&v_runs, 0.50),
        quantile_usize(&v_runs, 0.90),
        quantile_usize(&v_runs, 0.95),
        quantile_usize(&v_runs, 0.99),
        quantile_usize(&v_runs, 1.0),
    );
    println!(
        "  dist shift (B):  min={} p50={} p90={} p95={} p99={} max={}",
        quantile_usize(&v_shift, 0.0),
        quantile_usize(&v_shift, 0.50),
        quantile_usize(&v_shift, 0.90),
        quantile_usize(&v_shift, 0.95),
        quantile_usize(&v_shift, 0.99),
        quantile_usize(&v_shift, 1.0),
    );
    println!(
        "  dist grown (B):  min={} p50={} p90={} p95={} p99={} max={}",
        quantile_isize(&v_grown, 0.0),
        quantile_isize(&v_grown, 0.50),
        quantile_isize(&v_grown, 0.90),
        quantile_isize(&v_grown, 0.95),
        quantile_isize(&v_grown, 0.99),
        quantile_isize(&v_grown, 1.0),
    );
    println!(
        "  dist min_grown:  min={} p10={} p50={} p90={} max={}",
        quantile_isize(&v_min_grown, 0.0),
        quantile_isize(&v_min_grown, 0.10),
        quantile_isize(&v_min_grown, 0.50),
        quantile_isize(&v_min_grown, 0.90),
        quantile_isize(&v_min_grown, 1.0),
    );
    println!(
        "  dist len/n:      min={:.5} p50={:.5} p90={:.5} p99={:.5} max={:.5}",
        quantile_f64(&v_ratio, 0.0),
        quantile_f64(&v_ratio, 0.50),
        quantile_f64(&v_ratio, 0.90),
        quantile_f64(&v_ratio, 0.99),
        quantile_f64(&v_ratio, 1.0),
    );
    if !v_first_frac.is_empty() {
        println!(
            "  first_run/n (runs>0): p10={:.3} p50={:.3} p90={:.3} | first_mut_run/n: p10={:.3} p50={:.3} p90={:.3}",
            quantile_f64(&v_first_frac, 0.10),
            quantile_f64(&v_first_frac, 0.50),
            quantile_f64(&v_first_frac, 0.90),
            quantile_f64(&v_first_mut_frac, 0.10),
            quantile_f64(&v_first_mut_frac, 0.50),
            quantile_f64(&v_first_mut_frac, 0.90),
        );
    }

    let mut ship_rd = 0usize;
    let mut ship_wr = 0usize;
    let mut bwd_rd = 0usize;
    let mut bwd_wr = 0usize;
    let mut oop_rd = 0usize;
    let mut oop_wr = 0usize;
    let mut sp_in_rd = 0usize;
    let mut sp_in_wr = 0usize;

    for b in &c.blocks {
        let n = b.inv.len();
        let len = b.dec_len;
        if b.run_count == 0 {
            ship_rd += n + len;
            bwd_rd += n + len;
            sp_in_rd += n;
        } else if b.shift > n {
            ship_rd += n + n + len;
            ship_wr += len;
            bwd_rd += n + n + len;
            bwd_wr += len;
            sp_in_rd += n;
            sp_in_wr += len;
        } else if b.shift == 0 {
            let tail = n - b.first_mut_run;
            ship_rd += n + tail + len;
            ship_wr += tail;
            bwd_rd += n + tail + len;
            bwd_wr += tail;
            sp_in_rd += n;
            sp_in_wr += tail;
        } else {
            let tail = n - b.first_run;
            ship_rd += n + tail + tail + len;
            ship_wr += tail + tail;
            if b.min_grown >= 0 {
                bwd_rd += n + tail + len;
                bwd_wr += tail;
            } else {
                bwd_rd += n + tail + tail + len;
                bwd_wr += tail + tail;
            }
            sp_in_rd += n;
            sp_in_wr += tail;
        }
        oop_rd += n;
        oop_wr += len;
    }

    let mb = 1024.0 * 1024.0;
    println!("  Analytic logical memory traffic (1x | x4, MiB; CRC L1d-hot counted as 0 L3 traffic for tiled single-pass):");
    println!(
        "    shipped_inplace:        rd={:6.1} wr={:6.1} tot={:6.1} MiB | x4: rd={:7.1} wr={:7.1} tot={:7.1} MiB",
        ship_rd as f64 / mb, ship_wr as f64 / mb, (ship_rd + ship_wr) as f64 / mb,
        (ship_rd * 4) as f64 / mb, (ship_wr * 4) as f64 / mb, ((ship_rd + ship_wr) * 4) as f64 / mb
    );
    println!(
        "    backward_inplace:       rd={:6.1} wr={:6.1} tot={:6.1} MiB | x4: rd={:7.1} wr={:7.1} tot={:7.1} MiB (wr {:.1}%)",
        bwd_rd as f64 / mb, bwd_wr as f64 / mb, (bwd_rd + bwd_wr) as f64 / mb,
        (bwd_rd * 4) as f64 / mb, (bwd_wr * 4) as f64 / mb, ((bwd_rd + bwd_wr) * 4) as f64 / mb,
        if ship_wr == 0 { 0.0 } else { 100.0 * (bwd_wr as f64) / (ship_wr as f64) }
    );
    println!(
        "    single_pass_oop_tiled:  rd={:6.1} wr={:6.1} tot={:6.1} MiB | x4: rd={:7.1} wr={:7.1} tot={:7.1} MiB (wr {})",
        oop_rd as f64 / mb, oop_wr as f64 / mb, (oop_rd + oop_wr) as f64 / mb,
        (oop_rd * 4) as f64 / mb, (oop_wr * 4) as f64 / mb, ((oop_rd + oop_wr) * 4) as f64 / mb,
        if ship_wr == 0 { "inf".to_string() } else { format!("{:.1}%", 100.0 * (oop_wr as f64) / (ship_wr as f64)) }
    );
    println!(
        "    single_pass_inplace:    rd={:6.1} wr={:6.1} tot={:6.1} MiB | x4: rd={:7.1} wr={:7.1} tot={:7.1} MiB (wr {:.1}%)",
        sp_in_rd as f64 / mb, sp_in_wr as f64 / mb, (sp_in_rd + sp_in_wr) as f64 / mb,
        (sp_in_rd * 4) as f64 / mb, (sp_in_wr * 4) as f64 / mb, ((sp_in_rd + sp_in_wr) * 4) as f64 / mb,
        if ship_wr == 0 { 0.0 } else { 100.0 * (sp_in_wr as f64) / (ship_wr as f64) }
    );
}

// ---------------------------------------------------------------------------
// Corpus-wide verification of every candidate arm
// ---------------------------------------------------------------------------

fn verify_corpus(c: &Corpus, comb: &CrcCombine, k: Kernels) {
    let mut runs = Vec::new();
    let mut stage = Vec::new();
    let mut q = Vec::new();
    let mut tile_crcs = Vec::new();
    let mut want = Vec::new();

    for (bi, b) in c.blocks.iter().enumerate() {
        want.clear();
        rle::unrle(&b.inv, &mut want, k).unwrap();

        let mut buf = Vec::with_capacity(prod_pool_cap(b.inv.len()));

        buf.clear();
        buf.extend_from_slice(&b.inv);
        let got = shipped_unrle_in_place(&mut buf, &mut runs, &mut stage, k).unwrap();
        assert_eq!(got, b.crc, "{} blk {bi} shipped_inplace crc", c.name);
        assert_eq!(buf, want, "{} blk {bi} shipped_inplace bytes", c.name);

        buf.clear();
        buf.extend_from_slice(&b.inv);
        let got = backward_unrle_in_place(
            &mut buf,
            &mut runs,
            &mut stage,
            None,
            &mut tile_crcs,
            SHIPPED_TILE,
            k,
        )
        .unwrap();
        assert_eq!(got, b.crc, "{} blk {bi} backward_sep crc", c.name);
        assert_eq!(buf, want, "{} blk {bi} backward_sep bytes", c.name);

        buf.clear();
        buf.extend_from_slice(&b.inv);
        let got = backward_unrle_in_place(
            &mut buf,
            &mut runs,
            &mut stage,
            Some(comb),
            &mut tile_crcs,
            32 << 10,
            k,
        )
        .unwrap();
        assert_eq!(got, b.crc, "{} blk {bi} backward_tiled32k crc", c.name);
        assert_eq!(buf, want, "{} blk {bi} backward_tiled32k bytes", c.name);

        for &tile in &[0usize, 16 << 10, 32 << 10, 64 << 10] {
            buf.clear();
            buf.extend_from_slice(&b.inv);
            let got = single_pass_oop_fused(&mut buf, &mut stage, tile, k).unwrap();
            assert_eq!(got, b.crc, "{} blk {bi} oop_fused({tile}) crc", c.name);
            assert_eq!(buf, want, "{} blk {bi} oop_fused({tile}) bytes", c.name);
        }

        for &fused in &[false, true] {
            for &tile in &[0usize, 16 << 10, 32 << 10, 64 << 10] {
                buf.clear();
                buf.extend_from_slice(&b.inv);
                let got =
                    single_pass_inplace(&mut buf, &mut q, &mut stage, tile, fused, k).unwrap();
                assert_eq!(
                    got, b.crc,
                    "{} blk {bi} sp_inplace(tile={tile}, fused={fused}) crc",
                    c.name
                );
                assert_eq!(
                    buf, want,
                    "{} blk {bi} sp_inplace(tile={tile}, fused={fused}) bytes",
                    c.name
                );
            }
        }
    }
    println!(
        "VERIFIED 100% byte-identical + CRC32-identical across all {} blocks of {} (15 arms)",
        c.blocks.len(),
        c.name
    );
}

// ---------------------------------------------------------------------------
// Timing helpers & Benchmarks (Per-block L3-warm AND Cold streaming)
// ---------------------------------------------------------------------------

struct Stats {
    min: f64,
    med: f64,
    max: f64,
}

fn summarize(samples: &[f64]) -> Stats {
    let mut s = samples.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    let n = s.len();
    let med = if n % 2 == 1 {
        s[n / 2]
    } else {
        0.5 * (s[n / 2 - 1] + s[n / 2])
    };
    Stats {
        min: s[0],
        med,
        max: s[n - 1],
    }
}

fn restore_all(work: &mut [Vec<u8>], blocks: &[BlockData]) {
    for (w, b) in work.iter_mut().zip(blocks.iter()) {
        w.clear();
        let cap = prod_pool_cap(b.inv.len());
        if w.capacity() < cap {
            w.reserve(cap);
        }
        w.extend_from_slice(&b.inv);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Regime {
    /// Single recycled hot buffer (~0.9 MiB) restored immediately before each block:
    /// models 1W production right after `invert_runs` (hot TLB + hot L3, L2-spilled full block).
    PerBlockL3Warm,
    /// Full unbatched 431-block sweep across distinct allocations (200 MiB >> 45 MiB L3):
    /// models LLC-evicted DRAM streaming under multi-worker contention.
    ColdStream,
}

fn run_timed_arm<F>(
    blocks: &[BlockData],
    hot_buf: &mut Vec<u8>,
    work: &mut [Vec<u8>],
    regime: Regime,
    mut body: F,
) -> (f64, u32)
where
    F: FnMut(usize, &mut Vec<u8>, &BlockData) -> u32,
{
    let mut xor_crc = 0u32;
    match regime {
        Regime::PerBlockL3Warm => {
            let mut total_ns = 0u128;
            for (i, b) in blocks.iter().enumerate() {
                hot_buf.clear();
                let cap = prod_pool_cap(b.inv.len());
                if hot_buf.capacity() < cap {
                    hot_buf.reserve(cap);
                }
                hot_buf.extend_from_slice(&b.inv);
                let t0 = Instant::now();
                xor_crc ^= body(i, hot_buf, b);
                total_ns += t0.elapsed().as_nanos();
            }
            (total_ns as f64 / 1e6, xor_crc)
        }
        Regime::ColdStream => {
            restore_all(work, blocks);
            let t0 = Instant::now();
            for (i, b) in blocks.iter().enumerate() {
                xor_crc ^= body(i, &mut work[i], b);
            }
            (t0.elapsed().as_nanos() as f64 / 1e6, xor_crc)
        }
    }
}

fn bench_scanners_and_passes(c: &Corpus, k: Kernels, repeats: usize, scale: usize) {
    let nb = c.blocks.len();
    let max_cap = c
        .blocks
        .iter()
        .map(|b| prod_pool_cap(b.inv.len().max(b.dec_len)))
        .max()
        .unwrap_or(1 << 20);
    let mut hot_buf = Vec::with_capacity(max_cap);
    let mut work: Vec<Vec<u8>> = (0..nb)
        .map(|i| Vec::with_capacity(prod_pool_cap(c.blocks[i].inv.len())))
        .collect();
    let mut runs = Vec::new();
    let mut stage = Vec::with_capacity(max_cap);
    let mut dec_blocks: Vec<Vec<u8>> = Vec::with_capacity(nb);
    for b in &c.blocks {
        let mut d = Vec::with_capacity(b.dec_len);
        rle::unrle(&b.inv, &mut d, k).unwrap();
        dec_blocks.push(d);
    }
    let mut pre_runs: Vec<Vec<Run>> = Vec::with_capacity(nb);
    for b in &c.blocks {
        let mut r = Vec::new();
        find_runs_fast(&b.inv, &mut r).unwrap();
        pre_runs.push(r);
    }

    // Also measure L1d-hot CRC32 & scan ceilings on a single run-free 16 KiB slice repeated to 838.8 MB
    let tile16: Vec<u8> = (0..(16 << 10)).map(|i| (i % 251) as u8).collect();
    let tot_bytes = c.blocks.iter().map(|b| b.dec_len).sum::<usize>() * scale;
    let l1_iters = tot_bytes / tile16.len();
    let mut l1_crc_samples = Vec::with_capacity(repeats);
    let mut l1_scan_samples = Vec::with_capacity(repeats);
    for rep in 0..=repeats {
        let t0 = Instant::now();
        let mut s = !0u32;
        for _ in 0..l1_iters {
            s = crc::update_with(s, black_box(&tile16), k);
        }
        let ms_crc = t0.elapsed().as_nanos() as f64 / 1e6;
        black_box(s);

        let t1 = Instant::now();
        let mut acc = 0usize;
        for _ in 0..l1_iters {
            acc += find_next_run_autovec32(black_box(&tile16)).unwrap_or(0);
        }
        let ms_scan = t1.elapsed().as_nanos() as f64 / 1e6;
        black_box(acc);

        if rep > 0 {
            l1_crc_samples.push(ms_crc);
            l1_scan_samples.push(ms_scan);
        }
    }
    let st_l1_crc = summarize(&l1_crc_samples);
    let st_l1_scan = summarize(&l1_scan_samples);
    let bytes_gb = tot_bytes as f64 / 1e9;
    println!(
        "\n--- L1d-hot (16 KiB resident) kernel ceilings for {:.2} MB ({} repeats) ---",
        bytes_gb * 1e3,
        repeats
    );
    println!(
        "  L1d-hot crc32 (PCLMULQDQ):  {:6.2} [{:6.2} .. {:6.2}] ms  ({:5.2} GB/s)",
        st_l1_crc.med,
        st_l1_crc.min,
        st_l1_crc.max,
        bytes_gb / (st_l1_crc.med / 1e3)
    );
    println!(
        "  L1d-hot scan_autovec32:     {:6.2} [{:6.2} .. {:6.2}] ms  ({:5.2} GB/s)",
        st_l1_scan.med,
        st_l1_scan.min,
        st_l1_scan.max,
        bytes_gb / (st_l1_scan.med / 1e3)
    );

    let names = [
        "scan_scalar",
        "scan_swar64",
        "scan_autovec32",
        "pass2_shift_up_only",
        "pass3_expand_no_crc",
        "pass4_crc32_only",
        "pass4_crc32_tiled16k",
        "shipped_unrle_avx2_only",
        "shipped_inplace_total",
    ];

    for regime in [Regime::PerBlockL3Warm, Regime::ColdStream] {
        let reg_name = match regime {
            Regime::PerBlockL3Warm => "Per-block L3-warm (1 hot recycled buf)",
            Regime::ColdStream => "Cold-stream (200 MiB sweep >> L3)",
        };
        let mut samples: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); names.len()];

        for rep in 0..=repeats {
            let mut order: Vec<usize> = (0..names.len()).collect();
            if rep % 2 == 1 {
                order.reverse();
            }
            for &arm in &order {
                let (ms, chk) = match arm {
                    0 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        find_runs_with(w, &mut runs, |s| find_next_run_scalar(s, 0)).unwrap();
                        runs.len() as u32
                    }),
                    1 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        find_runs_with(w, &mut runs, find_next_run_swar64).unwrap();
                        runs.len() as u32
                    }),
                    2 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        find_runs_fast(w, &mut runs).unwrap();
                        runs.len() as u32
                    }),
                    3 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, b| {
                        if b.shift > 0 && b.shift <= b.inv.len() {
                            let n = w.len();
                            w.reserve_exact(b.shift);
                            w.resize(n + b.shift, 0);
                            w.copy_within(b.first_run..n, b.first_run + b.shift);
                        }
                        w.len() as u32
                    }),
                    4 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |i, w, _| {
                        shipped_expand_no_crc(w, &pre_runs[i], &mut stage, k).unwrap();
                        w.len() as u32
                    }),
                    5 => {
                        let mut total_ns = 0u128;
                        let mut xor = 0u32;
                        match regime {
                            Regime::PerBlockL3Warm => {
                                for d in &dec_blocks {
                                    hot_buf.clear();
                                    hot_buf.extend_from_slice(d);
                                    let t0 = Instant::now();
                                    xor ^= crc::crc32(&hot_buf, k);
                                    total_ns += t0.elapsed().as_nanos();
                                }
                            }
                            Regime::ColdStream => {
                                restore_all(&mut work, &c.blocks);
                                let t0 = Instant::now();
                                for d in &dec_blocks {
                                    xor ^= crc::crc32(d, k);
                                }
                                total_ns = t0.elapsed().as_nanos();
                            }
                        }
                        (total_ns as f64 / 1e6, xor)
                    }
                    6 => {
                        let mut total_ns = 0u128;
                        let mut xor = 0u32;
                        match regime {
                            Regime::PerBlockL3Warm => {
                                for d in &dec_blocks {
                                    hot_buf.clear();
                                    hot_buf.extend_from_slice(d);
                                    let t0 = Instant::now();
                                    let mut s = !0u32;
                                    for ch in hot_buf.chunks(16 << 10) {
                                        s = crc::update_with(s, ch, k);
                                    }
                                    xor ^= !s;
                                    total_ns += t0.elapsed().as_nanos();
                                }
                            }
                            Regime::ColdStream => {
                                restore_all(&mut work, &c.blocks);
                                let t0 = Instant::now();
                                for d in &dec_blocks {
                                    let mut s = !0u32;
                                    for ch in d.chunks(16 << 10) {
                                        s = crc::update_with(s, ch, k);
                                    }
                                    xor ^= !s;
                                }
                                total_ns = t0.elapsed().as_nanos();
                            }
                        }
                        (total_ns as f64 / 1e6, xor)
                    }
                    7 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        stage.clear();
                        rle::unrle(w, &mut stage, k).unwrap();
                        stage.len() as u32
                    }),
                    8 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        shipped_unrle_in_place(w, &mut runs, &mut stage, k).unwrap()
                    }),
                    _ => unreachable!(),
                };
                black_box(chk);
                if rep > 0 {
                    samples[arm].push(ms * scale as f64);
                }
            }
        }

        let tot_med = summarize(&samples[8]).med;
        println!(
            "\n--- PART 1B: Per-pass breakdown on {} (x{}, {:.2} MB) [{}] ({} repeats) ---",
            c.name,
            scale,
            bytes_gb * 1e3,
            reg_name,
            repeats
        );
        println!(
            "{:<26} {:>9} {:>20} {:>9} {:>9}",
            "pass", "med ms", "[min .. max] ms", "GB/s", "% shipped"
        );
        for (i, name) in names.iter().enumerate() {
            let st = summarize(&samples[i]);
            println!(
                "{:<26} {:>9.2} [{:>7.2} .. {:>7.2}] {:>9.2} {:>8.1}%",
                name,
                st.med,
                st.min,
                st.max,
                bytes_gb / (st.med / 1e3),
                100.0 * st.med / tot_med
            );
        }
    }
}

fn bench_designs(c: &Corpus, comb: &CrcCombine, k: Kernels, repeats: usize, scale: usize) {
    let nb = c.blocks.len();
    let max_cap = c
        .blocks
        .iter()
        .map(|b| prod_pool_cap(b.inv.len().max(b.dec_len)))
        .max()
        .unwrap_or(1 << 20);
    let mut hot_buf = Vec::with_capacity(max_cap);
    let mut work: Vec<Vec<u8>> = (0..nb)
        .map(|i| Vec::with_capacity(prod_pool_cap(c.blocks[i].inv.len())))
        .collect();
    let mut runs = Vec::new();
    let mut stage = Vec::with_capacity(max_cap);
    let mut q = Vec::with_capacity(64 << 10);
    let mut tile_crcs = Vec::with_capacity(64);
    let mut dec_blocks: Vec<Vec<u8>> = Vec::with_capacity(nb);
    let mut expected_xor = 0u32;
    for b in &c.blocks {
        let mut d = Vec::with_capacity(b.dec_len);
        rle::unrle(&b.inv, &mut d, k).unwrap();
        dec_blocks.push(d);
        expected_xor ^= b.crc;
    }

    let mut cold_dsts: Vec<Vec<u8>> = (0..nb)
        .map(|i| Vec::with_capacity(prod_pool_cap(c.blocks[i].dec_len)))
        .collect();

    let arm_names = [
        "shipped_inplace",
        "backward_inplace_sep_crc",
        "backward_inplace_tiled32k",
        "shipped_oop_avx2_sep_crc",
        "sp_oop_fused_notile",
        "sp_oop_fused_tiled16k",
        "sp_oop_fused_tiled32k",
        "sp_oop_fused_tiled64k",
        "sp_oop_fused_16k_cold_dst",
        "sp_inplace_q_tiled16k",
        "sp_inplace_1w_notile",
        "sp_inplace_1w_tiled16k",
        "sp_inplace_1w_tiled32k",
        "sp_inplace_1w_tiled64k",
        "crc32_only_floor",
    ];

    for regime in [Regime::PerBlockL3Warm, Regime::ColdStream] {
        let reg_name = match regime {
            Regime::PerBlockL3Warm => "Per-block L3-warm (1 hot recycled buf)",
            Regime::ColdStream => "Cold-stream (200 MiB sweep >> L3)",
        };
        let mut samples: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arm_names.len()];

        for rep in 0..=repeats {
            let mut order: Vec<usize> = (0..arm_names.len()).collect();
            if rep % 2 == 1 {
                order.reverse();
            }
            for &arm in &order {
                let (ms, chk) = match arm {
                    0 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        shipped_unrle_in_place(w, &mut runs, &mut stage, k).unwrap()
                    }),
                    1 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        backward_unrle_in_place(
                            w,
                            &mut runs,
                            &mut stage,
                            None,
                            &mut tile_crcs,
                            SHIPPED_TILE,
                            k,
                        )
                        .unwrap()
                    }),
                    2 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        backward_unrle_in_place(
                            w,
                            &mut runs,
                            &mut stage,
                            Some(comb),
                            &mut tile_crcs,
                            32 << 10,
                            k,
                        )
                        .unwrap()
                    }),
                    3 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        stage.clear();
                        rle::unrle(w, &mut stage, k).unwrap();
                        let c = crc::crc32(&stage, k);
                        std::mem::swap(w, &mut stage);
                        c
                    }),
                    4 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        single_pass_oop_fused(w, &mut stage, 0, k).unwrap()
                    }),
                    5 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        single_pass_oop_fused(w, &mut stage, 16 << 10, k).unwrap()
                    }),
                    6 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        single_pass_oop_fused(w, &mut stage, 32 << 10, k).unwrap()
                    }),
                    7 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        single_pass_oop_fused(w, &mut stage, 64 << 10, k).unwrap()
                    }),
                    8 => {
                        let mut total_ns = 0u128;
                        let mut xor = 0u32;
                        match regime {
                            Regime::PerBlockL3Warm => {
                                for (i, b) in c.blocks.iter().enumerate() {
                                    hot_buf.clear();
                                    hot_buf.extend_from_slice(&b.inv);
                                    let t0 = Instant::now();
                                    xor ^= single_pass_oop_fused(
                                        &mut hot_buf,
                                        &mut cold_dsts[i],
                                        16 << 10,
                                        k,
                                    )
                                    .unwrap();
                                    total_ns += t0.elapsed().as_nanos();
                                }
                            }
                            Regime::ColdStream => {
                                restore_all(&mut work, &c.blocks);
                                let t0 = Instant::now();
                                for i in 0..nb {
                                    xor ^= single_pass_oop_fused(
                                        &mut work[i],
                                        &mut cold_dsts[i],
                                        16 << 10,
                                        k,
                                    )
                                    .unwrap();
                                }
                                total_ns = t0.elapsed().as_nanos();
                            }
                        }
                        (total_ns as f64 / 1e6, xor)
                    }
                    9 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        single_pass_inplace(w, &mut q, &mut stage, 16 << 10, false, k).unwrap()
                    }),
                    10 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        single_pass_inplace(w, &mut q, &mut stage, 0, true, k).unwrap()
                    }),
                    11 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        single_pass_inplace(w, &mut q, &mut stage, 16 << 10, true, k).unwrap()
                    }),
                    12 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        single_pass_inplace(w, &mut q, &mut stage, 32 << 10, true, k).unwrap()
                    }),
                    13 => run_timed_arm(&c.blocks, &mut hot_buf, &mut work, regime, |_, w, _| {
                        single_pass_inplace(w, &mut q, &mut stage, 64 << 10, true, k).unwrap()
                    }),
                    14 => {
                        let mut total_ns = 0u128;
                        let mut xor = 0u32;
                        match regime {
                            Regime::PerBlockL3Warm => {
                                for d in &dec_blocks {
                                    hot_buf.clear();
                                    hot_buf.extend_from_slice(d);
                                    let t0 = Instant::now();
                                    xor ^= crc::crc32(&hot_buf, k);
                                    total_ns += t0.elapsed().as_nanos();
                                }
                            }
                            Regime::ColdStream => {
                                restore_all(&mut work, &c.blocks);
                                let t0 = Instant::now();
                                for d in &dec_blocks {
                                    xor ^= crc::crc32(d, k);
                                }
                                total_ns = t0.elapsed().as_nanos();
                            }
                        }
                        (total_ns as f64 / 1e6, xor)
                    }
                    _ => unreachable!(),
                };
                assert_eq!(
                    chk, expected_xor,
                    "CRC xor mismatch on arm {}",
                    arm_names[arm]
                );
                if rep > 0 {
                    samples[arm].push(ms * scale as f64);
                }
            }
        }

        let base = summarize(&samples[0]);
        let bytes_gb = (c.blocks.iter().map(|b| b.dec_len).sum::<usize>() * scale) as f64 / 1e9;
        println!(
            "\n--- PART 2: Designs A/B on {} (x{}, {:.2} MB) [{}] ({} repeats) ---",
            c.name,
            scale,
            bytes_gb * 1e3,
            reg_name,
            repeats
        );
        println!(
            "{:<28} {:>9} {:>20} {:>9} {:>10} {:>12}",
            "arm", "med ms", "[min .. max] ms", "GB/s", "vs shipped", "non-overlap"
        );
        for (i, name) in arm_names.iter().enumerate() {
            let st = summarize(&samples[i]);
            let delta_pct = 100.0 * (st.med - base.med) / base.med;
            let non_ov = if st.max < base.min {
                "FASTER"
            } else if st.min > base.max {
                "SLOWER"
            } else {
                "overlap"
            };
            println!(
                "{:<28} {:>9.2} [{:>7.2} .. {:>7.2}] {:>9.2} {:>+9.2}% {:>12}",
                name,
                st.med,
                st.min,
                st.max,
                bytes_gb / (st.med / 1e3),
                delta_pct,
                non_ov
            );
        }
    }
}

fn bench_by_category(c: &Corpus, comb: &CrcCombine, k: Kernels, repeats: usize, scale: usize) {
    type BlockPred = fn(&BlockData) -> bool;
    let cats: [(&str, BlockPred); 3] = [
        ("runs == 0", |b| b.run_count == 0),
        ("runs > 0, shift == 0", |b| b.run_count > 0 && b.shift == 0),
        ("shift > 0", |b| b.shift > 0),
    ];

    println!(
        "\n--- PART 2C: Category breakdown on {} (Per-block L3-warm, scaled x{}, {} repeats) ---",
        c.name, scale, repeats
    );
    let mut runs = Vec::new();
    let mut stage = Vec::with_capacity(1 << 20);
    let mut q = Vec::with_capacity(64 << 10);
    let mut tile_crcs = Vec::with_capacity(64);

    for (label, pred) in cats {
        let idxs: Vec<usize> = c
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, b)| pred(b))
            .map(|(i, _)| i)
            .collect();
        if idxs.is_empty() {
            continue;
        }
        let sub_bytes: usize = idxs.iter().map(|&i| c.blocks[i].dec_len).sum();
        let sub_blocks: Vec<BlockData> = idxs
            .iter()
            .map(|&i| {
                let b = &c.blocks[i];
                BlockData {
                    inv: b.inv.clone(),
                    crc: b.crc,
                    dec_len: b.dec_len,
                    run_count: b.run_count,
                    extra_eq_0: b.extra_eq_0,
                    extra_eq_1: b.extra_eq_1,
                    extra_ge_2: b.extra_ge_2,
                    first_run: b.first_run,
                    first_mut_run: b.first_mut_run,
                    shift: b.shift,
                    total_grown: b.total_grown,
                    min_grown: b.min_grown,
                }
            })
            .collect();
        let mut hot_buf = Vec::with_capacity(1 << 20);
        let mut work: Vec<Vec<u8>> = Vec::new();

        let mut s_ship = Vec::with_capacity(repeats);
        let mut s_bwd = Vec::with_capacity(repeats);
        let mut s_oop = Vec::with_capacity(repeats);
        let mut s_spin = Vec::with_capacity(repeats);
        for rep in 0..=repeats {
            let (m0, _) = run_timed_arm(
                &sub_blocks,
                &mut hot_buf,
                &mut work,
                Regime::PerBlockL3Warm,
                |_, w, _| shipped_unrle_in_place(w, &mut runs, &mut stage, k).unwrap(),
            );
            let (m1, _) = run_timed_arm(
                &sub_blocks,
                &mut hot_buf,
                &mut work,
                Regime::PerBlockL3Warm,
                |_, w, _| {
                    backward_unrle_in_place(
                        w,
                        &mut runs,
                        &mut stage,
                        Some(comb),
                        &mut tile_crcs,
                        32 << 10,
                        k,
                    )
                    .unwrap()
                },
            );
            let (m2, _) = run_timed_arm(
                &sub_blocks,
                &mut hot_buf,
                &mut work,
                Regime::PerBlockL3Warm,
                |_, w, _| single_pass_oop_fused(w, &mut stage, 16 << 10, k).unwrap(),
            );
            let (m3, _) = run_timed_arm(
                &sub_blocks,
                &mut hot_buf,
                &mut work,
                Regime::PerBlockL3Warm,
                |_, w, _| single_pass_inplace(w, &mut q, &mut stage, 16 << 10, true, k).unwrap(),
            );
            if rep > 0 {
                s_ship.push(m0 * scale as f64);
                s_bwd.push(m1 * scale as f64);
                s_oop.push(m2 * scale as f64);
                s_spin.push(m3 * scale as f64);
            }
        }
        let st0 = summarize(&s_ship);
        let st1 = summarize(&s_bwd);
        let st2 = summarize(&s_oop);
        let st3 = summarize(&s_spin);
        println!(
            "  {:<22} ({:3} blks, {:6.1} MiB x1): shipped={:6.2} | bwd_tiled={:6.2} ({:+5.1}%) | oop_fused16k={:6.2} ({:+5.1}%) | in_1w16k={:6.2} ({:+5.1}%)",
            label,
            idxs.len(),
            sub_bytes as f64 / (1024.0 * 1024.0),
            st0.med,
            st1.med, 100.0 * (st1.med - st0.med) / st0.med,
            st2.med, 100.0 * (st2.med - st0.med) / st0.med,
            st3.med, 100.0 * (st3.med - st0.med) / st0.med,
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut paths: Vec<String> = Vec::new();
    let mut repeats = 7usize;
    let mut skip_bench = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repeats" => {
                i += 1;
                repeats = args[i].parse().expect("--repeats N");
            }
            "--skip-bench" => skip_bench = true,
            p => paths.push(p.to_string()),
        }
        i += 1;
    }
    if paths.is_empty() {
        paths.push("corpora/wikidata.bz2".to_string());
    }

    let k = Kernels::detect();
    let comb = CrcCombine::new();
    println!("Kernels::detect() = {k:?} | repeats = {repeats}");
    run_self_tests(&comb, k);
    println!(
        "Self-tests passed (edge cases, offsets, tile boundaries, random blocks, GF(2) combine)."
    );

    let mut corpora = Vec::with_capacity(paths.len());
    for p in &paths {
        let c = load_corpus(Path::new(p), k);
        print_census(&c);
        verify_corpus(&c, &comb, k);
        corpora.push(c);
    }

    if skip_bench {
        return;
    }

    for (idx, c) in corpora.iter().enumerate() {
        let scale = if c.name.contains("wikidata") {
            X4_MULTIPLIER
        } else {
            1
        };
        if idx == 0 || c.name.contains("wikidata") {
            bench_scanners_and_passes(c, k, repeats, scale);
            bench_designs(c, &comb, k, repeats, scale);
            bench_by_category(c, &comb, k, repeats, scale);
        } else {
            bench_designs(c, &comb, k, repeats.min(5), 1);
        }
    }
}
