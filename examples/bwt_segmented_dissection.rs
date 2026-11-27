//! Smooth-zone probe (`explore/bwt-segmented-dissection`, zero `src/` edits):
//! Deep dissection and redesign probe of post-#1080 `Segmented` BWT (`src/bwt.rs` +
//! `src/runs.rs` + `src/rle.rs` + `src/crc.rs`) across all 7 corpora (`wikidata`,
//! `text`, `random`, `records`, `pattern`, `runs`, `onebyte`).
//!
//! Run with `--features moonshot` across baseline x86-64, `-C target-cpu=native`, and
//! JCC-32B-aligned builds:
//!     cargo run --release --features moonshot --example bwt_segmented_dissection -- \
//!         /home/antigravity/wt-a6b-probe/corpora/wikidata.bz2 /tmp/bwt_seg_corpora/*.bz2

#[cfg(feature = "moonshot")]
mod imp {
    use mibz2::bwt::{self, InverseBwt, OpenCells, Segmented, PF_L1, PF_L2};
    use mibz2::runs::{self, Runs};
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::{crc, mtf, rle, ErrorKind, Kernels};
    use std::path::Path;
    use std::time::Instant;

    const MIN_SEGMENTED: usize = bwt::MIN_SEGMENTED;
    const FLAGGED_SEGMENTS: usize = 512;
    const TILE_SHIPPED: usize = 256;
    const START32: u32 = 1 << 31;
    const LINK32: u32 = !START32;
    const RLE_TILE: usize = 64 << 10;

    #[cfg(target_os = "linux")]
    fn thread_cpu_ns() -> u64 {
        #[repr(C)]
        struct Timespec {
            tv_sec: i64,
            tv_nsec: i64,
        }
        extern "C" {
            fn clock_gettime(clk_id: i32, tp: *mut Timespec) -> i32;
        }
        const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
        let mut ts = Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: valid pointer to initialized Timespec.
        unsafe {
            clock_gettime(CLOCK_THREAD_CPUTIME_ID, &mut ts);
        }
        (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64)
    }

    #[cfg(not(target_os = "linux"))]
    fn thread_cpu_ns() -> u64 {
        0
    }

    /// Scrubs > 45 MiB L3 cache (64 MiB stride-64 read/modify/write) to establish an L3-cold state.
    #[inline(never)]
    fn flush_l3(buf: &mut [u8]) {
        let mut acc = 0u8;
        let mut i = 0usize;
        while i < buf.len() {
            acc = acc.wrapping_add(buf[i]);
            buf[i] = acc.wrapping_add(1);
            i += 64;
        }
        std::hint::black_box(acc);
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ProdRoute {
        Open,
        RunsFallback,
        Column,
    }

    struct BlockData {
        corpus: String,
        route: ProdRoute,
        n: usize,
        orig_ptr: u32,
        crc_expected: u32,
        ll: Vec<u8>,
        counts: [u32; 256],
        runs: Runs,
        runs_vec: Vec<u16>,
        want_final: Vec<u8>,
    }

    // ---- Exact shipped `unrle_in_place` from src/rle.rs (since it is pub(crate)) ----

    type RleRun = (u32, u8, u8);

    fn find_runs_scalar(rle: &[u8], runs: &mut Vec<RleRun>, mut i: usize) -> Result<(), ErrorKind> {
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

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,bmi1")]
    unsafe fn find_runs_avx2(rle: &[u8], runs: &mut Vec<RleRun>) -> Result<(), ErrorKind> {
        use std::arch::x86_64::*;
        let n = rle.len();
        let p = rle.as_ptr();
        let mut i = 0usize;
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
            let j = i + m.trailing_zeros() as usize;
            let Some(&extra) = rle.get(j + 4) else {
                return Err(ErrorKind::RunUnfinished);
            };
            runs.push((j as u32, rle[j], extra));
            i = j + 5;
        }
        find_runs_scalar(rle, runs, i)
    }

    fn shipped_find_runs(rle: &[u8], runs: &mut Vec<RleRun>, k: Kernels) -> Result<(), ErrorKind> {
        runs.clear();
        #[cfg(target_arch = "x86_64")]
        if k.avx2() {
            // SAFETY: k.avx2() guarantees AVX2+BMI1.
            return unsafe { find_runs_avx2(rle, runs) };
        }
        find_runs_scalar(rle, runs, 0)
    }

    #[inline(never)]
    fn shipped_unrle_in_place(
        buf: &mut Vec<u8>,
        runs: &mut Vec<RleRun>,
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
        shipped_find_runs(buf, runs, k)?;
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
            while end - *at >= RLE_TILE {
                *s = crc::update_with(*s, &buf[*at..*at + RLE_TILE], k);
                *at += RLE_TILE;
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

    // ---- Helpers: random_rows, prefetch, reverse_copy ----

    fn random_rows(n: usize, m: usize, rows: &mut Vec<u32>) {
        rows.clear();
        let mut x = 0x2545_f491_4f6c_dd1du64 ^ n as u64;
        rows.extend((0..m).map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x % n as u64) as u32
        }));
    }

    #[inline(always)]
    unsafe fn prefetch_hint<const H: u8>(t: *const u32, row: u32) {
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::*;
            let p = t.add(row as usize) as *const i8;
            match H {
                PF_L1 => _mm_prefetch::<_MM_HINT_T0>(p),
                PF_L2 => _mm_prefetch::<_MM_HINT_T1>(p),
                _ => {}
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (t, row);
        }
    }

    #[inline(always)]
    unsafe fn reverse_copy_scalar(src: *const u8, dst: *mut u8, len: usize) {
        let mut i = 0usize;
        while i + 8 <= len {
            let w = (src.add(i) as *const u64).read_unaligned().swap_bytes();
            (dst.add(len - i - 8) as *mut u64).write_unaligned(w);
            i += 8;
        }
        while i < len {
            *dst.add(len - 1 - i) = *src.add(i);
            i += 1;
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn reverse_copy_avx2(src: *const u8, dst: *mut u8, len: usize) {
        use std::arch::x86_64::*;
        let rev = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let mut i = 0usize;
        while i + 32 <= len {
            let v = _mm256_loadu_si256(src.add(i) as *const __m256i);
            let s = _mm256_shuffle_epi8(v, rev);
            let r = _mm256_permute2x128_si256::<0x01>(s, s);
            _mm256_storeu_si256(dst.add(len - i - 32) as *mut __m256i, r);
            i += 32;
        }
        reverse_copy_scalar(src.add(i), dst, len - i);
    }

    #[inline(always)]
    unsafe fn reverse_copy_fast(src: *const u8, dst: *mut u8, len: usize, avx2: bool) {
        #[cfg(target_arch = "x86_64")]
        if avx2 && len >= 32 {
            reverse_copy_avx2(src, dst, len);
            return;
        }
        let _ = avx2;
        reverse_copy_scalar(src, dst, len);
    }

    // =========================================================================
    // LEVER M2: LF T-vector builders (`Column` & `RunsFallback`)
    // =========================================================================

    /// Shipped `build_lf` copy for isolated build benchmarking.
    #[inline(never)]
    fn build_lf_shipped(ll: &[u8], counts: &[u32; 256], tt: &mut Vec<u32>) -> bool {
        let n = ll.len();
        tt.clear();
        if n >= 1 << 24 || counts.iter().map(|&c| c as usize).sum::<usize>() != n {
            return false;
        }
        let (mut ftab, mut lim) = ([0u32; 256], [0u32; 256]);
        let mut cum = 0u32;
        for b in 0..256 {
            ftab[b] = cum;
            cum += counts[b];
            lim[b] = cum;
        }
        tt.reserve(n);
        let src = ll.as_ptr();
        let dst = tt.as_mut_ptr();
        let mut i = 0usize;
        while i + 8 <= n {
            let w = unsafe { (src.add(i) as *const u64).read_unaligned() };
            if w == w.rotate_left(8) {
                let c = (w & 0xff) as usize;
                let s = ftab[c];
                ftab[c] = s.wrapping_add(8);
                let base = (c as u32) | (s << 8);
                unsafe {
                    let d = dst.add(i);
                    *d = base;
                    *d.add(1) = base.wrapping_add(1 << 8);
                    *d.add(2) = base.wrapping_add(2 << 8);
                    *d.add(3) = base.wrapping_add(3 << 8);
                    *d.add(4) = base.wrapping_add(4 << 8);
                    *d.add(5) = base.wrapping_add(5 << 8);
                    *d.add(6) = base.wrapping_add(6 << 8);
                    *d.add(7) = base.wrapping_add(7 << 8);
                }
                i += 8;
            } else {
                unsafe {
                    for k in 0..8 {
                        let c = *src.add(i + k) as usize;
                        let s = ftab[c];
                        ftab[c] = s.wrapping_add(1);
                        *dst.add(i + k) = (c as u32) | (s << 8);
                    }
                }
                i += 8;
            }
        }
        while i < n {
            unsafe {
                let c = *src.add(i) as usize;
                let s = ftab[c];
                ftab[c] = s.wrapping_add(1);
                *dst.add(i) = (c as u32) | (s << 8);
            }
            i += 1;
        }
        if ftab != lim {
            return false;
        }
        unsafe { tt.set_len(n) };
        true
    }

    /// M2a: Pre-shifted `ftab[c] = c | (cum << 8)` with 8-byte equal-run fast path.
    /// Eliminates `shl $8` and `or` on every single byte!
    #[inline(never)]
    fn build_lf_preshift_run8(ll: &[u8], counts: &[u32; 256], tt: &mut Vec<u32>) -> bool {
        let n = ll.len();
        tt.clear();
        if n >= 1 << 24 || counts.iter().map(|&c| c as usize).sum::<usize>() != n {
            return false;
        }
        let (mut ftab, mut lim) = ([0u32; 256], [0u32; 256]);
        let mut cum = 0u32;
        for b in 0..256 {
            ftab[b] = (b as u32) | (cum << 8);
            cum += counts[b];
            lim[b] = (b as u32) | (cum << 8);
        }
        tt.reserve(n);
        let src = ll.as_ptr();
        let dst = tt.as_mut_ptr();
        let mut i = 0usize;
        while i + 8 <= n {
            let w = unsafe { (src.add(i) as *const u64).read_unaligned() };
            if w == w.rotate_left(8) {
                let c = (w & 0xff) as usize;
                let base = unsafe { *ftab.get_unchecked(c) };
                unsafe { *ftab.get_unchecked_mut(c) = base.wrapping_add(8 << 8) };
                unsafe {
                    let d = dst.add(i);
                    *d = base;
                    *d.add(1) = base.wrapping_add(1 << 8);
                    *d.add(2) = base.wrapping_add(2 << 8);
                    *d.add(3) = base.wrapping_add(3 << 8);
                    *d.add(4) = base.wrapping_add(4 << 8);
                    *d.add(5) = base.wrapping_add(5 << 8);
                    *d.add(6) = base.wrapping_add(6 << 8);
                    *d.add(7) = base.wrapping_add(7 << 8);
                }
                i += 8;
            } else {
                unsafe {
                    for k in 0..8 {
                        let c = *src.add(i + k) as usize;
                        let e = *ftab.get_unchecked(c);
                        *ftab.get_unchecked_mut(c) = e.wrapping_add(256);
                        *dst.add(i + k) = e;
                    }
                }
                i += 8;
            }
        }
        while i < n {
            unsafe {
                let c = *src.add(i) as usize;
                let e = *ftab.get_unchecked(c);
                *ftab.get_unchecked_mut(c) = e.wrapping_add(256);
                *dst.add(i) = e;
            }
            i += 1;
        }
        if ftab != lim {
            return false;
        }
        unsafe { tt.set_len(n) };
        true
    }

    /// M2b: Pre-shifted `ftab` + `u64` paired store combining (`(e0 as u64) | ((e1 as u64) << 32)`)
    /// cutting total Port-4 stores per 8 bytes from 16 (8 ftab + 8 tt) to 12 (8 ftab + 4 tt)!
    #[inline(never)]
    fn build_lf_preshift_norun(ll: &[u8], counts: &[u32; 256], tt: &mut Vec<u32>) -> bool {
        let n = ll.len();
        tt.clear();
        if n >= 1 << 24 || counts.iter().map(|&c| c as usize).sum::<usize>() != n {
            return false;
        }
        let (mut ftab, mut lim) = ([0u32; 256], [0u32; 256]);
        let mut cum = 0u32;
        for b in 0..256 {
            ftab[b] = (b as u32) | (cum << 8);
            cum += counts[b];
            lim[b] = (b as u32) | (cum << 8);
        }
        tt.reserve(n);
        let src = ll.as_ptr();
        let dst = tt.as_mut_ptr();
        let mut i = 0usize;
        while i + 8 <= n {
            let w = unsafe { (src.add(i) as *const u64).read_unaligned() };
            if w == w.rotate_left(8) {
                let c = (w & 0xff) as usize;
                let base = unsafe { *ftab.get_unchecked(c) };
                unsafe { *ftab.get_unchecked_mut(c) = base.wrapping_add(8 << 8) };
                unsafe {
                    let d = dst.add(i) as *mut u64;
                    let p0 = (base as u64) | ((base.wrapping_add(1 << 8) as u64) << 32);
                    let p1 = (base.wrapping_add(2 << 8) as u64)
                        | ((base.wrapping_add(3 << 8) as u64) << 32);
                    let p2 = (base.wrapping_add(4 << 8) as u64)
                        | ((base.wrapping_add(5 << 8) as u64) << 32);
                    let p3 = (base.wrapping_add(6 << 8) as u64)
                        | ((base.wrapping_add(7 << 8) as u64) << 32);
                    d.write_unaligned(p0);
                    d.add(1).write_unaligned(p1);
                    d.add(2).write_unaligned(p2);
                    d.add(3).write_unaligned(p3);
                }
                i += 8;
            } else {
                unsafe {
                    let d = dst.add(i) as *mut u64;
                    macro_rules! pair {
                        ($idx:expr, $off:expr) => {{
                            let c0 = *src.add(i + $off) as usize;
                            let e0 = *ftab.get_unchecked(c0);
                            *ftab.get_unchecked_mut(c0) = e0.wrapping_add(256);
                            let c1 = *src.add(i + $off + 1) as usize;
                            let e1 = *ftab.get_unchecked(c1);
                            *ftab.get_unchecked_mut(c1) = e1.wrapping_add(256);
                            d.add($idx)
                                .write_unaligned((e0 as u64) | ((e1 as u64) << 32));
                        }};
                    }
                    pair!(0, 0);
                    pair!(1, 2);
                    pair!(2, 4);
                    pair!(3, 6);
                }
                i += 8;
            }
        }
        while i < n {
            unsafe {
                let c = *src.add(i) as usize;
                let e = *ftab.get_unchecked(c);
                *ftab.get_unchecked_mut(c) = e.wrapping_add(256);
                *dst.add(i) = e;
            }
            i += 1;
        }
        if ftab != lim {
            return false;
        }
        unsafe { tt.set_len(n) };
        true
    }

    /// Exact copy of shipped `src/runs.rs::fill_avx2_trusted` writing into `&mut Vec<u32>`.
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn fill_avx2_shipped_inner(
        runs: &[u16],
        n: u32,
        mut next: [u32; 256],
        tt: &mut Vec<u32>,
    ) {
        use std::arch::x86_64::*;
        tt.clear();
        tt.reserve(n as usize + 8);
        let out = tt.as_mut_ptr();
        let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
        let step = _mm256_set1_epi32(2048);
        let mut q = 0usize;
        for &x in runs {
            let c = (x >> 8) as usize;
            let len = (x & 0xff) as usize;
            let cur = next[c];
            next[c] = cur + len as u32;
            let e = (c as u32) | (cur << 8);
            let mut v = _mm256_add_epi32(_mm256_set1_epi32(e as i32), lanes);
            let end = q + len;
            let mut w = q;
            loop {
                _mm256_storeu_si256(out.add(w) as *mut __m256i, v);
                w += 8;
                if w >= end {
                    break;
                }
                v = _mm256_add_epi32(v, step);
            }
            q = end;
        }
        debug_assert_eq!(q, n as usize);
        tt.set_len(n as usize);
    }

    #[inline(never)]
    fn fill_shipped_raw(runs: &[u16], counts: &[u32; 256], n: usize, tt: &mut Vec<u32>) {
        let mut next = [0u32; 256];
        let mut cum = 0u32;
        for b in 0..256 {
            next[b] = cum;
            cum += counts[b];
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            fill_avx2_shipped_inner(runs, n as u32, next, tt);
        }
    }

    /// M2c (`RunsFallback`): Pre-shifted `next[c] = c | (cum << 8)` + AVX2 `vpbroadcastd` from
    /// memory + hoisted `len <= 8` single-store fast path (no inner loop / wasted vpaddd for
    /// 96-99% of records!).
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn fill_avx2_preshift_len8_inner(
        runs: &[u16],
        n: u32,
        mut next: [u32; 256],
        tt: &mut Vec<u32>,
    ) {
        use std::arch::x86_64::*;
        tt.clear();
        tt.reserve(n as usize + 8);
        let out = tt.as_mut_ptr();
        let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
        let step = _mm256_set1_epi32(2048);
        let mut q = 0usize;
        for &x in runs {
            let c = (x >> 8) as usize;
            let len = (x & 0xff) as usize;
            let e = *next.get_unchecked(c);
            *next.get_unchecked_mut(c) = e.wrapping_add((len as u32) << 8);
            let mut v = _mm256_add_epi32(_mm256_set1_epi32(e as i32), lanes);
            _mm256_storeu_si256(out.add(q) as *mut __m256i, v);
            let end = q + len;
            if len > 8 {
                let mut w = q + 8;
                loop {
                    v = _mm256_add_epi32(v, step);
                    _mm256_storeu_si256(out.add(w) as *mut __m256i, v);
                    w += 8;
                    if w >= end {
                        break;
                    }
                }
            }
            q = end;
        }
        debug_assert_eq!(q, n as usize);
        tt.set_len(n as usize);
    }

    #[inline(never)]
    fn fill_preshift_len8(runs: &[u16], counts: &[u32; 256], n: usize, tt: &mut Vec<u32>) {
        let mut next = [0u32; 256];
        let mut cum = 0u32;
        for b in 0..256 {
            next[b] = (b as u32) | (cum << 8);
            cum += counts[b];
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            fill_avx2_preshift_len8_inner(runs, n as u32, next, tt);
        }
    }

    /// M2d (`RunsFallback`): Pre-shifted `next[c] = c | (cum << 8)` keeping the exact shipped
    /// branchless-per-record loop (zero extra `len <= 8` branch misprediction!), only removing
    /// `shl $8` + `or` and allowing direct memory `vpbroadcastd`!
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn fill_avx2_preshift_same_inner(
        runs: &[u16],
        n: u32,
        mut next: [u32; 256],
        tt: &mut Vec<u32>,
    ) {
        use std::arch::x86_64::*;
        tt.clear();
        tt.reserve(n as usize + 8);
        let out = tt.as_mut_ptr();
        let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
        let step = _mm256_set1_epi32(2048);
        let mut q = 0usize;
        for &x in runs {
            let c = (x >> 8) as usize;
            let len = (x & 0xff) as usize;
            let e = *next.get_unchecked(c);
            *next.get_unchecked_mut(c) = e.wrapping_add((len as u32) << 8);
            let mut v = _mm256_add_epi32(_mm256_set1_epi32(e as i32), lanes);
            let end = q + len;
            let mut w = q;
            loop {
                _mm256_storeu_si256(out.add(w) as *mut __m256i, v);
                w += 8;
                if w >= end {
                    break;
                }
                v = _mm256_add_epi32(v, step);
            }
            q = end;
        }
        debug_assert_eq!(q, n as usize);
        tt.set_len(n as usize);
    }

    #[inline(never)]
    fn fill_preshift_unroll2(runs: &[u16], counts: &[u32; 256], n: usize, tt: &mut Vec<u32>) {
        let mut next = [0u32; 256];
        let mut cum = 0u32;
        for b in 0..256 {
            next[b] = (b as u32) | (cum << 8);
            cum += counts[b];
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            fill_avx2_preshift_same_inner(runs, n as u32, next, tt);
        }
    }

    // =========================================================================
    // LEVER M3: Segmented room walk variants + custom scratch
    // =========================================================================

    #[derive(Default)]
    struct WalkScratch {
        starts: Vec<u32>,
        bufs: Vec<Vec<u8>>,
        next: Vec<usize>,
        first: usize,
        n: usize,
        chain: Vec<usize>,
        cycle_tmp: Vec<u8>,
    }

    impl WalkScratch {
        fn heap_bytes(&self) -> usize {
            self.starts.capacity() * 4
                + self.bufs.capacity() * std::mem::size_of::<Vec<u8>>()
                + self.bufs.iter().map(Vec::capacity).sum::<usize>()
                + self.next.capacity() * std::mem::size_of::<usize>()
                + self.chain.capacity() * std::mem::size_of::<usize>()
                + self.cycle_tmp.capacity()
        }
    }

    #[derive(Clone, Copy)]
    struct Lane {
        row: u32,
        seg: usize,
        p: *mut u8,
        end: *mut u8,
    }

    /// Standard `room_segments` parameterized by `L`, `PF`, `CLEAR_START`, `TILE`.
    #[inline(never)]
    fn walk_room_std<const L: usize, const PF: u8, const CLEAR_START: bool>(
        tt: &mut [u32],
        orig_ptr: u32,
        s: &mut WalkScratch,
        segments: usize,
    ) {
        const { assert!(L <= 64) };
        let n = tt.len();
        let r0 = orig_ptr;
        random_rows(n, segments - 1, &mut s.starts);
        s.starts.push(r0);
        s.starts.sort_unstable();
        s.starts.dedup();
        let ns = s.starts.len();
        for &r in &s.starts {
            tt[r as usize] |= START32;
        }
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let starts = &s.starts;
        let seg_of = move |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let t = tt.as_ptr();
        let bufs = &mut s.bufs;
        let next = &mut s.next;

        let take = move |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
            let q = unsafe { *t.add(starts[seg] as usize) } & LINK32;
            unsafe { *base = q as u8 };
            Lane {
                row: q >> 8,
                seg,
                p: unsafe { base.add(1) },
                end: unsafe { base.add(b.capacity()) },
            }
        };
        let finish = move |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
            let b = &mut bufs[l.seg];
            unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
            next[l.seg] = seg_of(l.row);
        };
        let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
            let b = &mut bufs[l.seg];
            let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
            unsafe { b.set_len(len) };
            b.reserve(b.capacity());
            l.p = unsafe { b.as_mut_ptr().add(len) };
            l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        };

        let mut queued = 0;
        let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
            row: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        });
        let mut active = 0;
        while active < L && queued < ns {
            lanes[active] = take(queued, bufs);
            unsafe { prefetch_hint::<PF>(t, lanes[active].row) };
            queued += 1;
            active += 1;
        }
        if active == L {
            let mut rows = [0u32; L];
            for (k, l) in lanes.iter().enumerate() {
                rows[k] = l.row;
            }
            let mut tile = [[0u8; TILE_SHIPPED]; L];
            let flush = |l: &mut Lane, row: &[u8; TILE_SHIPPED], from: usize, to: usize| {
                let len = to - from;
                unsafe {
                    std::ptr::copy_nonoverlapping(row.as_ptr().add(from), l.p, len);
                    l.p = l.p.add(len);
                }
            };
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l, bufs);
                    }
                    room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
                }
                let mut done = 0;
                while done < room {
                    let w = (room - done).min(TILE_SHIPPED);
                    let mut st = [0usize; L];
                    for j in 0..w {
                        let rs = &mut rows;
                        let mut marked: u64 = 0;
                        let tp = tile.as_mut_ptr().cast::<u8>();
                        macro_rules! sstep {
                            ($k:expr) => {{
                                let k = $k;
                                unsafe {
                                    let r = rs.get_unchecked_mut(k);
                                    let q = *t.add(*r as usize);
                                    if q & START32 != 0 {
                                        marked |= 1u64 << k;
                                    } else {
                                        *tp.add(k * TILE_SHIPPED + j) = q as u8;
                                        *r = q >> 8;
                                        prefetch_hint::<PF>(t, q >> 8);
                                    }
                                }
                            }};
                        }
                        let mut c = 0;
                        while c + 8 <= L {
                            sstep!(c);
                            sstep!(c + 1);
                            sstep!(c + 2);
                            sstep!(c + 3);
                            sstep!(c + 4);
                            sstep!(c + 5);
                            sstep!(c + 6);
                            sstep!(c + 7);
                            c += 8;
                        }
                        while c < L {
                            sstep!(c);
                            c += 1;
                        }
                        while marked != 0 {
                            let k = marked.trailing_zeros() as usize;
                            marked &= marked - 1;
                            let l = &mut lanes[k];
                            l.row = rs[k];
                            flush(l, &tile[k], st[k], j);
                            finish(l, bufs, next);
                            if queued == ns {
                                for (i, li) in lanes.iter_mut().enumerate() {
                                    li.row = rs[i];
                                    if i != k {
                                        let waiting = (marked >> i) & 1 != 0;
                                        flush(li, &tile[i], st[i], j + usize::from(!waiting));
                                    }
                                }
                                break 'full;
                            }
                            *l = take(queued, bufs);
                            rs[k] = l.row;
                            unsafe { prefetch_hint::<PF>(t, l.row) };
                            queued += 1;
                            st[k] = j + 1;
                        }
                    }
                    for (k, l) in lanes.iter_mut().enumerate() {
                        flush(l, &tile[k], st[k], w);
                    }
                    done += w;
                }
            }
        }
        let mut live: Vec<Lane> = lanes[..active]
            .iter()
            .copied()
            .filter(|l| next[l.seg] == usize::MAX)
            .collect();
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l, bufs);
            }
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let q = unsafe { *t.add(l.row as usize) };
                if q & START32 != 0 {
                    finish(l, bufs, next);
                    live.swap_remove(i);
                    continue;
                }
                l.row = q >> 8;
                unsafe {
                    prefetch_hint::<PF>(t, l.row);
                    *l.p = q as u8;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    grow(l, bufs);
                }
                i += 1;
            }
        }
        if CLEAR_START {
            for &r in starts {
                tt[r as usize] &= LINK32;
            }
        }
        s.first = seg_of(r0);
        s.n = n;
    }

    /// Branchless `cmov` + 8× unrolled `sstep!` room walk (zero per-lane conditional branch;
    /// single `if marked != 0` check per `L` lanes!).
    #[inline(never)]
    fn walk_room_cmov<const L: usize, const PF: u8, const CLEAR_START: bool, const TILE: usize>(
        tt: &mut [u32],
        orig_ptr: u32,
        s: &mut WalkScratch,
        segments: usize,
    ) {
        const { assert!(L <= 64) };
        let n = tt.len();
        let r0 = orig_ptr;
        random_rows(n, segments - 1, &mut s.starts);
        s.starts.push(r0);
        s.starts.sort_unstable();
        s.starts.dedup();
        let ns = s.starts.len();
        for &r in &s.starts {
            tt[r as usize] |= START32;
        }
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let starts = &s.starts;
        let seg_of = move |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let t = tt.as_ptr();
        let bufs = &mut s.bufs;
        let next = &mut s.next;

        let take = move |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
            let q = unsafe { *t.add(starts[seg] as usize) } & LINK32;
            unsafe { *base = q as u8 };
            Lane {
                row: q >> 8,
                seg,
                p: unsafe { base.add(1) },
                end: unsafe { base.add(b.capacity()) },
            }
        };
        let finish = move |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
            let b = &mut bufs[l.seg];
            unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
            next[l.seg] = seg_of(l.row);
        };
        let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
            let b = &mut bufs[l.seg];
            let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
            unsafe { b.set_len(len) };
            b.reserve(b.capacity());
            l.p = unsafe { b.as_mut_ptr().add(len) };
            l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        };

        let mut queued = 0;
        let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
            row: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        });
        let mut active = 0;
        while active < L && queued < ns {
            lanes[active] = take(queued, bufs);
            unsafe { prefetch_hint::<PF>(t, lanes[active].row) };
            queued += 1;
            active += 1;
        }
        if active == L {
            let mut rows = [0u32; L];
            for (k, l) in lanes.iter().enumerate() {
                rows[k] = l.row;
            }
            let mut tile = [[0u8; TILE]; L];
            let flush = |l: &mut Lane, row: &[u8; TILE], from: usize, to: usize| {
                let len = to - from;
                unsafe {
                    std::ptr::copy_nonoverlapping(row.as_ptr().add(from), l.p, len);
                    l.p = l.p.add(len);
                }
            };
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l, bufs);
                    }
                    room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
                }
                let mut done = 0;
                while done < room {
                    let w = (room - done).min(TILE);
                    let mut st = [0usize; L];
                    for j in 0..w {
                        let rs = &mut rows;
                        let mut marked: u64 = 0;
                        let tp = tile.as_mut_ptr().cast::<u8>();
                        macro_rules! cstep {
                            ($k:expr) => {{
                                let k = $k;
                                unsafe {
                                    let r = *rs.get_unchecked(k);
                                    let q = *t.add(r as usize);
                                    let is_start = q >> 31;
                                    marked |= (is_start as u64) << k;
                                    *tp.add(k * TILE + j) = q as u8;
                                    let next_r = if is_start != 0 { r } else { q >> 8 };
                                    *rs.get_unchecked_mut(k) = next_r;
                                    prefetch_hint::<PF>(t, next_r);
                                }
                            }};
                        }
                        let mut c = 0;
                        while c + 8 <= L {
                            cstep!(c);
                            cstep!(c + 1);
                            cstep!(c + 2);
                            cstep!(c + 3);
                            cstep!(c + 4);
                            cstep!(c + 5);
                            cstep!(c + 6);
                            cstep!(c + 7);
                            c += 8;
                        }
                        while c < L {
                            cstep!(c);
                            c += 1;
                        }
                        while marked != 0 {
                            let k = marked.trailing_zeros() as usize;
                            marked &= marked - 1;
                            let l = &mut lanes[k];
                            l.row = rs[k];
                            flush(l, &tile[k], st[k], j);
                            finish(l, bufs, next);
                            if queued == ns {
                                for (i, li) in lanes.iter_mut().enumerate() {
                                    li.row = rs[i];
                                    if i != k {
                                        let waiting = (marked >> i) & 1 != 0;
                                        flush(li, &tile[i], st[i], j + usize::from(!waiting));
                                    }
                                }
                                break 'full;
                            }
                            *l = take(queued, bufs);
                            rs[k] = l.row;
                            unsafe { prefetch_hint::<PF>(t, l.row) };
                            queued += 1;
                            st[k] = j + 1;
                        }
                    }
                    for (k, l) in lanes.iter_mut().enumerate() {
                        flush(l, &tile[k], st[k], w);
                    }
                    done += w;
                }
            }
        }
        let mut live: Vec<Lane> = lanes[..active]
            .iter()
            .copied()
            .filter(|l| next[l.seg] == usize::MAX)
            .collect();
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l, bufs);
            }
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let q = unsafe { *t.add(l.row as usize) };
                if q & START32 != 0 {
                    finish(l, bufs, next);
                    live.swap_remove(i);
                    continue;
                }
                l.row = q >> 8;
                unsafe {
                    prefetch_hint::<PF>(t, l.row);
                    *l.p = q as u8;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    grow(l, bufs);
                }
                i += 1;
            }
        }
        if CLEAR_START {
            for &r in starts {
                tt[r as usize] &= LINK32;
            }
        }
        s.first = seg_of(r0);
        s.n = n;
    }

    /// AVX2 transpose of 16 rows × 32 lanes (`src: *const [u8; 32]`) directly into
    /// `lanes[0..32].p` (each lane receives 16 contiguous bytes!).
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn transpose_16x32_flush_avx2(src: *const [u8; 32], lanes: &mut [Lane; 32]) {
        use std::arch::x86_64::*;
        macro_rules! ld {
            ($i:expr) => {
                _mm256_loadu_si256(src.add($i) as *const __m256i)
            };
        }
        // Stage 1: 8-bit interleave
        let r0 = ld!(0);
        let r1 = ld!(1);
        let t0 = _mm256_unpacklo_epi8(r0, r1);
        let t1 = _mm256_unpackhi_epi8(r0, r1);
        let r2 = ld!(2);
        let r3 = ld!(3);
        let t2 = _mm256_unpacklo_epi8(r2, r3);
        let t3 = _mm256_unpackhi_epi8(r2, r3);
        let r4 = ld!(4);
        let r5 = ld!(5);
        let t4 = _mm256_unpacklo_epi8(r4, r5);
        let t5 = _mm256_unpackhi_epi8(r4, r5);
        let r6 = ld!(6);
        let r7 = ld!(7);
        let t6 = _mm256_unpacklo_epi8(r6, r7);
        let t7 = _mm256_unpackhi_epi8(r6, r7);
        let r8 = ld!(8);
        let r9 = ld!(9);
        let t8 = _mm256_unpacklo_epi8(r8, r9);
        let t9 = _mm256_unpackhi_epi8(r8, r9);
        let r10 = ld!(10);
        let r11 = ld!(11);
        let t10 = _mm256_unpacklo_epi8(r10, r11);
        let t11 = _mm256_unpackhi_epi8(r10, r11);
        let r12 = ld!(12);
        let r13 = ld!(13);
        let t12 = _mm256_unpacklo_epi8(r12, r13);
        let t13 = _mm256_unpackhi_epi8(r12, r13);
        let r14 = ld!(14);
        let r15 = ld!(15);
        let t14 = _mm256_unpacklo_epi8(r14, r15);
        let t15 = _mm256_unpackhi_epi8(r14, r15);

        // Stage 2: 16-bit interleave
        let u0 = _mm256_unpacklo_epi16(t0, t2);
        let u1 = _mm256_unpackhi_epi16(t0, t2);
        let u2 = _mm256_unpacklo_epi16(t1, t3);
        let u3 = _mm256_unpackhi_epi16(t1, t3);
        let u4 = _mm256_unpacklo_epi16(t4, t6);
        let u5 = _mm256_unpackhi_epi16(t4, t6);
        let u6 = _mm256_unpacklo_epi16(t5, t7);
        let u7 = _mm256_unpackhi_epi16(t5, t7);
        let u8 = _mm256_unpacklo_epi16(t8, t10);
        let u9 = _mm256_unpackhi_epi16(t8, t10);
        let u10 = _mm256_unpacklo_epi16(t9, t11);
        let u11 = _mm256_unpackhi_epi16(t9, t11);
        let u12 = _mm256_unpacklo_epi16(t12, t14);
        let u13 = _mm256_unpackhi_epi16(t12, t14);
        let u14 = _mm256_unpacklo_epi16(t13, t15);
        let u15 = _mm256_unpackhi_epi16(t13, t15);

        // Stage 3: 32-bit interleave
        let v0 = _mm256_unpacklo_epi32(u0, u4);
        let v1 = _mm256_unpackhi_epi32(u0, u4);
        let v2 = _mm256_unpacklo_epi32(u1, u5);
        let v3 = _mm256_unpackhi_epi32(u1, u5);
        let v4 = _mm256_unpacklo_epi32(u2, u6);
        let v5 = _mm256_unpackhi_epi32(u2, u6);
        let v6 = _mm256_unpacklo_epi32(u3, u7);
        let v7 = _mm256_unpackhi_epi32(u3, u7);
        let v8 = _mm256_unpacklo_epi32(u8, u12);
        let v9 = _mm256_unpackhi_epi32(u8, u12);
        let v10 = _mm256_unpacklo_epi32(u9, u13);
        let v11 = _mm256_unpackhi_epi32(u9, u13);
        let v12 = _mm256_unpacklo_epi32(u10, u14);
        let v13 = _mm256_unpackhi_epi32(u10, u14);
        let v14 = _mm256_unpacklo_epi32(u11, u15);
        let v15 = _mm256_unpackhi_epi32(u11, u15);

        // Stage 4: 64-bit interleave + direct 16-byte stores to lanes[k] and lanes[k+16]
        macro_rules! store_pair {
            ($a:expr, $b:expr, $lo_k:expr, $hi_k:expr) => {{
                let w_lo = _mm256_unpacklo_epi64($a, $b);
                let w_hi = _mm256_unpackhi_epi64($a, $b);
                let l0 = lanes.get_unchecked_mut($lo_k);
                _mm_storeu_si128(l0.p as *mut __m128i, _mm256_castsi256_si128(w_lo));
                l0.p = l0.p.add(16);
                let l16 = lanes.get_unchecked_mut($lo_k + 16);
                _mm_storeu_si128(l16.p as *mut __m128i, _mm256_extracti128_si256::<1>(w_lo));
                l16.p = l16.p.add(16);

                let l1 = lanes.get_unchecked_mut($hi_k);
                _mm_storeu_si128(l1.p as *mut __m128i, _mm256_castsi256_si128(w_hi));
                l1.p = l1.p.add(16);
                let l17 = lanes.get_unchecked_mut($hi_k + 16);
                _mm_storeu_si128(l17.p as *mut __m128i, _mm256_extracti128_si256::<1>(w_hi));
                l17.p = l17.p.add(16);
            }};
        }
        store_pair!(v0, v8, 0, 1);
        store_pair!(v1, v9, 2, 3);
        store_pair!(v2, v10, 4, 5);
        store_pair!(v3, v11, 6, 7);
        store_pair!(v4, v12, 8, 9);
        store_pair!(v5, v13, 10, 11);
        store_pair!(v6, v14, 12, 13);
        store_pair!(v7, v15, 14, 15);
    }

    /// M3g: `L = 32` with **transposed `tile[j][k]` (`[[u8; 32]; 64]` = 2 KiB!)** —
    /// step `j` writes sequentially to ONE single 32-byte half-cacheline (zero 4 KiB L1d set
    /// aliasing, 1 active dirty cache line instead of 32!), flushed via AVX2 16×32 transpose!
    #[inline(never)]
    fn walk_room_transposed_32<const PF: u8, const CLEAR_START: bool>(
        tt: &mut [u32],
        orig_ptr: u32,
        s: &mut WalkScratch,
        segments: usize,
    ) {
        const L: usize = 32;
        const TTILE: usize = 64;
        let n = tt.len();
        let r0 = orig_ptr;
        random_rows(n, segments - 1, &mut s.starts);
        s.starts.push(r0);
        s.starts.sort_unstable();
        s.starts.dedup();
        let ns = s.starts.len();
        for &r in &s.starts {
            tt[r as usize] |= START32;
        }
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let starts = &s.starts;
        let seg_of = move |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let t = tt.as_ptr();
        let bufs = &mut s.bufs;
        let next = &mut s.next;

        let take = move |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
            let q = unsafe { *t.add(starts[seg] as usize) } & LINK32;
            unsafe { *base = q as u8 };
            Lane {
                row: q >> 8,
                seg,
                p: unsafe { base.add(1) },
                end: unsafe { base.add(b.capacity()) },
            }
        };
        let finish = move |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
            let b = &mut bufs[l.seg];
            unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
            next[l.seg] = seg_of(l.row);
        };
        let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
            let b = &mut bufs[l.seg];
            let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
            unsafe { b.set_len(len) };
            b.reserve(b.capacity());
            l.p = unsafe { b.as_mut_ptr().add(len) };
            l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        };

        let mut queued = 0;
        let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
            row: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        });
        let mut active = 0;
        while active < L && queued < ns {
            lanes[active] = take(queued, bufs);
            unsafe { prefetch_hint::<PF>(t, lanes[active].row) };
            queued += 1;
            active += 1;
        }
        if active == L {
            let mut rows = [0u32; L];
            for (k, l) in lanes.iter().enumerate() {
                rows[k] = l.row;
            }
            let mut tile = [[0u8; L]; TTILE];
            let flush_lane_scalar =
                |l: &mut Lane, tile: &[[u8; L]; TTILE], k: usize, from: usize, to: usize| {
                    let mut p = l.p;
                    for row in &tile[from..to] {
                        unsafe {
                            *p = *row.get_unchecked(k);
                            p = p.add(1);
                        }
                    }
                    l.p = p;
                };
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l, bufs);
                    }
                    room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
                }
                let mut done = 0;
                while done < room {
                    let w = (room - done).min(TTILE);
                    let mut st = [0usize; L];
                    let mut any_take = false;
                    for j in 0..w {
                        let rs = &mut rows;
                        let mut marked: u64 = 0;
                        let row_dst = unsafe { tile.as_mut_ptr().add(j) as *mut u8 };
                        macro_rules! sstep {
                            ($k:expr) => {{
                                let k = $k;
                                unsafe {
                                    let r = rs.get_unchecked_mut(k);
                                    let q = *t.add(*r as usize);
                                    if q & START32 != 0 {
                                        marked |= 1u64 << k;
                                    } else {
                                        *row_dst.add(k) = q as u8;
                                        *r = q >> 8;
                                        prefetch_hint::<PF>(t, q >> 8);
                                    }
                                }
                            }};
                        }
                        let mut c = 0;
                        while c < L {
                            sstep!(c);
                            sstep!(c + 1);
                            sstep!(c + 2);
                            sstep!(c + 3);
                            sstep!(c + 4);
                            sstep!(c + 5);
                            sstep!(c + 6);
                            sstep!(c + 7);
                            c += 8;
                        }
                        while marked != 0 {
                            any_take = true;
                            let k = marked.trailing_zeros() as usize;
                            marked &= marked - 1;
                            let l = &mut lanes[k];
                            l.row = rs[k];
                            flush_lane_scalar(l, &tile, k, st[k], j);
                            finish(l, bufs, next);
                            if queued == ns {
                                for (i, li) in lanes.iter_mut().enumerate() {
                                    li.row = rs[i];
                                    if i != k {
                                        let waiting = (marked >> i) & 1 != 0;
                                        flush_lane_scalar(
                                            li,
                                            &tile,
                                            i,
                                            st[i],
                                            j + usize::from(!waiting),
                                        );
                                    }
                                }
                                break 'full;
                            }
                            *l = take(queued, bufs);
                            rs[k] = l.row;
                            unsafe { prefetch_hint::<PF>(t, l.row) };
                            queued += 1;
                            st[k] = j + 1;
                        }
                    }
                    if !any_take && w == TTILE {
                        #[cfg(target_arch = "x86_64")]
                        unsafe {
                            let mut off = 0usize;
                            while off < TTILE {
                                transpose_16x32_flush_avx2(tile.as_ptr().add(off), &mut lanes);
                                off += 16;
                            }
                        }
                    } else {
                        for (k, l) in lanes.iter_mut().enumerate() {
                            flush_lane_scalar(l, &tile, k, st[k], w);
                        }
                    }
                    done += w;
                }
            }
        }
        let mut live: Vec<Lane> = lanes[..active]
            .iter()
            .copied()
            .filter(|l| next[l.seg] == usize::MAX)
            .collect();
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l, bufs);
            }
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let q = unsafe { *t.add(l.row as usize) };
                if q & START32 != 0 {
                    finish(l, bufs, next);
                    live.swap_remove(i);
                    continue;
                }
                l.row = q >> 8;
                unsafe {
                    prefetch_hint::<PF>(t, l.row);
                    *l.p = q as u8;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    grow(l, bufs);
                }
                i += 1;
            }
        }
        if CLEAR_START {
            for &r in starts {
                tt[r as usize] &= LINK32;
            }
        }
        s.first = seg_of(r0);
        s.n = n;
    }

    /// M3i: AVX2 `vpgatherdd` 32-lane walk (4 YMM registers hold all 32 `rows` in registers,
    /// zero per-lane branch, single 32B store into transposed `tile[j][0..32]` + AVX2 transpose).
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn walk_gather32_step(t: *const i32, rows: &mut [u32; 32], row_dst: *mut u8) -> u32 {
        use std::arch::x86_64::*;
        let rp = rows.as_ptr() as *const __m256i;
        let r0 = _mm256_loadu_si256(rp);
        let r1 = _mm256_loadu_si256(rp.add(1));
        let r2 = _mm256_loadu_si256(rp.add(2));
        let r3 = _mm256_loadu_si256(rp.add(3));

        let q0 = _mm256_i32gather_epi32::<4>(t, r0);
        let q1 = _mm256_i32gather_epi32::<4>(t, r1);
        let q2 = _mm256_i32gather_epi32::<4>(t, r2);
        let q3 = _mm256_i32gather_epi32::<4>(t, r3);

        let m0 = _mm256_movemask_ps(_mm256_castsi256_ps(q0)) as u32;
        let m1 = _mm256_movemask_ps(_mm256_castsi256_ps(q1)) as u32;
        let m2 = _mm256_movemask_ps(_mm256_castsi256_ps(q2)) as u32;
        let m3 = _mm256_movemask_ps(_mm256_castsi256_ps(q3)) as u32;
        let marked = m0 | (m1 << 8) | (m2 << 16) | (m3 << 24);

        // Pack low byte of each i32 lane in q0..q3 into 32 bytes.
        let mask_ff = _mm256_set1_epi32(0xff);
        let p01 = _mm256_packus_epi32(_mm256_and_si256(q0, mask_ff), _mm256_and_si256(q1, mask_ff));
        let p23 = _mm256_packus_epi32(_mm256_and_si256(q2, mask_ff), _mm256_and_si256(q3, mask_ff));
        // Fix lane ordering after in-lane packus_epi32 + packus_epi16:
        // p01 lo128 = [q0[0..4], q1[0..4]], hi128 = [q0[4..8], q1[4..8]]
        // permute4x64(0xD8): [q0[0..4], q0[4..8], q1[0..4], q1[4..8]]
        let p01 = _mm256_permute4x64_epi64::<0xD8>(p01);
        let p23 = _mm256_permute4x64_epi64::<0xD8>(p23);
        let packed = _mm256_permute4x64_epi64::<0xD8>(_mm256_packus_epi16(p01, p23));
        _mm256_storeu_si256(row_dst as *mut __m256i, packed);

        // Advance rows (for unmarked lanes; marked lanes will have rows[k] restored from caller
        // if needed, wait: preserve original rows[k] when marked!).
        let n0 = _mm256_srli_epi32::<8>(q0);
        let n1 = _mm256_srli_epi32::<8>(q1);
        let n2 = _mm256_srli_epi32::<8>(q2);
        let n3 = _mm256_srli_epi32::<8>(q3);
        let wp = rows.as_mut_ptr() as *mut __m256i;
        if marked == 0 {
            _mm256_storeu_si256(wp, n0);
            _mm256_storeu_si256(wp.add(1), n1);
            _mm256_storeu_si256(wp.add(2), n2);
            _mm256_storeu_si256(wp.add(3), n3);
        } else {
            // Keep old row where bit 31 of q was set (blendv uses sign bit!).
            let b0 = _mm256_castps_si256(_mm256_blendv_ps(
                _mm256_castsi256_ps(n0),
                _mm256_castsi256_ps(r0),
                _mm256_castsi256_ps(q0),
            ));
            let b1 = _mm256_castps_si256(_mm256_blendv_ps(
                _mm256_castsi256_ps(n1),
                _mm256_castsi256_ps(r1),
                _mm256_castsi256_ps(q1),
            ));
            let b2 = _mm256_castps_si256(_mm256_blendv_ps(
                _mm256_castsi256_ps(n2),
                _mm256_castsi256_ps(r2),
                _mm256_castsi256_ps(q2),
            ));
            let b3 = _mm256_castps_si256(_mm256_blendv_ps(
                _mm256_castsi256_ps(n3),
                _mm256_castsi256_ps(r3),
                _mm256_castsi256_ps(q3),
            ));
            _mm256_storeu_si256(wp, b0);
            _mm256_storeu_si256(wp.add(1), b1);
            _mm256_storeu_si256(wp.add(2), b2);
            _mm256_storeu_si256(wp.add(3), b3);
        }
        marked
    }

    #[inline(never)]
    fn walk_room_gather32(tt: &mut [u32], orig_ptr: u32, s: &mut WalkScratch, segments: usize) {
        const L: usize = 32;
        const TTILE: usize = 64;
        let n = tt.len();
        let r0 = orig_ptr;
        random_rows(n, segments - 1, &mut s.starts);
        s.starts.push(r0);
        s.starts.sort_unstable();
        s.starts.dedup();
        let ns = s.starts.len();
        for &r in &s.starts {
            tt[r as usize] |= START32;
        }
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let starts = &s.starts;
        let seg_of = move |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let t = tt.as_ptr();
        let bufs = &mut s.bufs;
        let next = &mut s.next;

        let take = move |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
            let q = unsafe { *t.add(starts[seg] as usize) } & LINK32;
            unsafe { *base = q as u8 };
            Lane {
                row: q >> 8,
                seg,
                p: unsafe { base.add(1) },
                end: unsafe { base.add(b.capacity()) },
            }
        };
        let finish = move |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
            let b = &mut bufs[l.seg];
            unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
            next[l.seg] = seg_of(l.row);
        };
        let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
            let b = &mut bufs[l.seg];
            let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
            unsafe { b.set_len(len) };
            b.reserve(b.capacity());
            l.p = unsafe { b.as_mut_ptr().add(len) };
            l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        };

        let mut queued = 0;
        let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
            row: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        });
        let mut active = 0;
        while active < L && queued < ns {
            lanes[active] = take(queued, bufs);
            queued += 1;
            active += 1;
        }
        if active == L {
            let mut rows = [0u32; L];
            for (k, l) in lanes.iter().enumerate() {
                rows[k] = l.row;
            }
            let mut tile = [[0u8; L]; TTILE];
            let flush_lane_scalar =
                |l: &mut Lane, tile: &[[u8; L]; TTILE], k: usize, from: usize, to: usize| {
                    let mut p = l.p;
                    for row in &tile[from..to] {
                        unsafe {
                            *p = *row.get_unchecked(k);
                            p = p.add(1);
                        }
                    }
                    l.p = p;
                };
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l, bufs);
                    }
                    room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
                }
                let mut done = 0;
                while done < room {
                    let w = (room - done).min(TTILE);
                    let mut st = [0usize; L];
                    let mut any_take = false;
                    for j in 0..w {
                        #[cfg(target_arch = "x86_64")]
                        let mut marked = unsafe {
                            walk_gather32_step(
                                t as *const i32,
                                &mut rows,
                                tile.as_mut_ptr().add(j) as *mut u8,
                            )
                        };
                        #[cfg(not(target_arch = "x86_64"))]
                        let mut marked = 0u32;
                        while marked != 0 {
                            any_take = true;
                            let k = marked.trailing_zeros() as usize;
                            marked &= marked - 1;
                            let l = &mut lanes[k];
                            l.row = rows[k];
                            flush_lane_scalar(l, &tile, k, st[k], j);
                            finish(l, bufs, next);
                            if queued == ns {
                                for (i, li) in lanes.iter_mut().enumerate() {
                                    li.row = rows[i];
                                    if i != k {
                                        let waiting = (marked >> i) & 1 != 0;
                                        flush_lane_scalar(
                                            li,
                                            &tile,
                                            i,
                                            st[i],
                                            j + usize::from(!waiting),
                                        );
                                    }
                                }
                                break 'full;
                            }
                            *l = take(queued, bufs);
                            rows[k] = l.row;
                            queued += 1;
                            st[k] = j + 1;
                        }
                    }
                    if !any_take && w == TTILE {
                        #[cfg(target_arch = "x86_64")]
                        unsafe {
                            let mut off = 0usize;
                            while off < TTILE {
                                transpose_16x32_flush_avx2(tile.as_ptr().add(off), &mut lanes);
                                off += 16;
                            }
                        }
                    } else {
                        for (k, l) in lanes.iter_mut().enumerate() {
                            flush_lane_scalar(l, &tile, k, st[k], w);
                        }
                    }
                    done += w;
                }
            }
        }
        let mut live: Vec<Lane> = lanes[..active]
            .iter()
            .copied()
            .filter(|l| next[l.seg] == usize::MAX)
            .collect();
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l, bufs);
            }
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let q = unsafe { *t.add(l.row as usize) };
                if q & START32 != 0 {
                    finish(l, bufs, next);
                    live.swap_remove(i);
                    continue;
                }
                l.row = q >> 8;
                unsafe {
                    *l.p = q as u8;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    grow(l, bufs);
                }
                i += 1;
            }
        }
        s.first = seg_of(r0);
        s.n = n;
    }

    // =========================================================================
    // LEVER M1: Single-Pass Fused Reverse-Assembly + AVX2 RLE1 + L1/L2-Hot CRC32
    // =========================================================================

    /// Streams `src[..avail]` (with `is_eof` indicating whether more bytes follow) through
    /// AVX2 RLE1 decoding directly into `out`, returning the number of unconsumed carry bytes
    /// (`0..=4`) which are left at `src[avail - rem .. avail]` (caller copies to `src[0..rem]`).
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,bmi1")]
    unsafe fn rle_decode_chunk_avx2(
        src: &[u8],
        avail: usize,
        is_eof: bool,
        out: &mut Vec<u8>,
    ) -> Result<usize, ErrorKind> {
        use std::arch::x86_64::*;
        let p = src.as_ptr();
        let mut i = 0usize;
        // Reserve conservatively for literal copy; runs re-check capacity before fill.
        out.reserve(avail + 264);
        let mut out_len = out.len();
        let mut out_cap = out.capacity();
        let mut out_p = out.as_mut_ptr();

        // Fast AVX2 loop: needs `i + 36 <= avail` so any 4-byte run at `j <= i + 31`
        // has its 5th count byte at `j + 4 <= i + 35 < avail` and `j + 5 <= avail`!
        while i + 36 <= avail {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let v2 = _mm256_loadu_si256(p.add(i + 2) as *const __m256i);
            let v3 = _mm256_loadu_si256(p.add(i + 3) as *const __m256i);
            let eq = _mm256_and_si256(
                _mm256_cmpeq_epi8(v0, v1),
                _mm256_and_si256(_mm256_cmpeq_epi8(v0, v2), _mm256_cmpeq_epi8(v0, v3)),
            );
            let m = _mm256_movemask_epi8(eq) as u32;
            if out_len + 64 > out_cap {
                out.set_len(out_len);
                out.reserve((avail - i) + 320);
                out_cap = out.capacity();
                out_p = out.as_mut_ptr();
            }
            // Always store all 32 bytes of `v0` straight from ymm0 (zero memcpy, zero vzeroupper!):
            // if m == 0, all 32 bytes are final; if m != 0, the leading `tz` literal bytes are
            // already in place and the trailing `32 - tz` slack bytes are immediately overwritten
            // by the run fill!
            _mm256_storeu_si256(out_p.add(out_len) as *mut __m256i, v0);
            if m == 0 {
                out_len += 32;
                i += 32;
                continue;
            }
            let tz = m.trailing_zeros() as usize;
            let j = i + tz;
            let c = *p.add(j);
            let extra = *p.add(j + 4) as usize;
            let run = 4 + extra;
            out_len += tz;
            if run <= 32 {
                // `out_len + 32 <= old_out_len + 64 <= out_cap`: 1-instruction run fill!
                let fill_v = _mm256_set1_epi8(c as i8);
                _mm256_storeu_si256(out_p.add(out_len) as *mut __m256i, fill_v);
                out_len += run;
            } else {
                if out_len + run + 64 > out_cap {
                    out.set_len(out_len);
                    out.reserve(run + (avail - j) + 320);
                    out_cap = out.capacity();
                    out_p = out.as_mut_ptr();
                }
                std::ptr::write_bytes(out_p.add(out_len), c, run);
                out_len += run;
            }
            i = j + 5;
        }

        // Scalar tail before EOF: process while at least 5 bytes remain so we never guess
        // across a chunk boundary!
        let limit = if is_eof {
            avail
        } else {
            avail.saturating_sub(4)
        };
        while i < limit {
            let c = *p.add(i);
            if i + 3 < avail && *p.add(i + 1) == c && *p.add(i + 2) == c && *p.add(i + 3) == c {
                if i + 4 >= avail {
                    out.set_len(out_len);
                    return Err(ErrorKind::RunUnfinished);
                }
                let run = 4 + (*p.add(i + 4) as usize);
                if out_len + run > out_cap {
                    out.set_len(out_len);
                    out.reserve(run + (avail - i) + 64);
                    out_cap = out.capacity();
                    out_p = out.as_mut_ptr();
                }
                std::ptr::write_bytes(out_p.add(out_len), c, run);
                out_len += run;
                i += 5;
            } else {
                if out_len + 1 > out_cap {
                    out.set_len(out_len);
                    out.reserve((avail - i) + 64);
                    out_cap = out.capacity();
                    out_p = out.as_mut_ptr();
                }
                *out_p.add(out_len) = c;
                out_len += 1;
                i += 1;
            }
        }
        out.set_len(out_len);
        Ok(avail - i)
    }

    fn rle_decode_chunk_scalar(
        src: &[u8],
        avail: usize,
        is_eof: bool,
        out: &mut Vec<u8>,
    ) -> Result<usize, ErrorKind> {
        let mut i = 0usize;
        let limit = if is_eof {
            avail
        } else {
            avail.saturating_sub(4)
        };
        while i < limit {
            let c = src[i];
            if i + 3 < avail && src[i + 1] == c && src[i + 2] == c && src[i + 3] == c {
                if i + 4 >= avail {
                    return Err(ErrorKind::RunUnfinished);
                }
                let run = 4 + src[i + 4] as usize;
                out.resize(out.len() + run, c);
                i += 5;
            } else {
                out.push(c);
                i += 1;
            }
        }
        Ok(avail - i)
    }

    #[inline(always)]
    fn rle_decode_chunk(
        src: &[u8],
        avail: usize,
        is_eof: bool,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<usize, ErrorKind> {
        #[cfg(target_arch = "x86_64")]
        if k.avx2() {
            return unsafe { rle_decode_chunk_avx2(src, avail, is_eof, out) };
        }
        rle_decode_chunk_scalar(src, avail, is_eof, out)
    }

    #[cold]
    #[inline(never)]
    fn grow_out_exact(out: &mut Vec<u8>, cur_len: usize, extra_need: usize) -> (*mut u8, usize) {
        // SAFETY: `cur_len` bytes have been initialized by caller.
        unsafe { out.set_len(cur_len) };
        // Grow by bounded chunks (`min 128 KiB`, capped so `text` never doubles 1 -> 2 MiB,
        // while `onebyte` 20 MiB still grows amortized fast via max(extra_need, cur_len / 2)).
        let step = extra_need.max((cur_len / 4).max(128 << 10));
        out.reserve_exact(step);
        (out.as_mut_ptr(), out.capacity())
    }

    /// Single-`#[target_feature]` AVX2 fused reverse-into-L1d + RLE1 + L2-hot CRC32!
    /// Eliminates all per-segment and per-chunk `vzeroupper` transitions, reverses every byte
    /// ONCE in 32B strides into a 4 KiB L1d stack buffer, hoists capacity checks outside the
    /// 32B literal scan, and folds PCLMULQDQ CRC32 every `CRC_TILE` bytes while L2-hot.
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,bmi1")]
    unsafe fn fused_rev_rle_crc_buf_avx2<const BUF_CAP: usize, const CRC_TILE: usize>(
        bufs: &[Vec<u8>],
        chain: &[usize],
        n: usize,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        use std::arch::x86_64::*;
        let rev = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        out.clear();
        if out.capacity() < n + 320 {
            out.reserve_exact((n + 320) - out.capacity());
        }
        let mut out_len = 0usize;
        let mut out_cap = out.capacity();
        let mut out_p = out.as_mut_ptr();

        let mut rev_buf = [0u8; BUF_CAP];
        let mut buf_len = 0usize;
        let mut crc_state = !0u32;
        let mut crc_at = 0usize;
        let mut rem_n = n;

        macro_rules! decode_buf {
            ($is_eof:expr) => {{
                let avail = buf_len;
                let is_eof = $is_eof;
                let p = rev_buf.as_ptr();
                let mut i = 0usize;
                // Hoist capacity check once for all non-expanding literal bytes in this chunk!
                if out_len + avail + 320 > out_cap {
                    let (np, nc) = grow_out_exact(out, out_len, avail + 320);
                    out_p = np;
                    out_cap = nc;
                }
                while i + 36 <= avail {
                    let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
                    let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
                    let v2 = _mm256_loadu_si256(p.add(i + 2) as *const __m256i);
                    let v3 = _mm256_loadu_si256(p.add(i + 3) as *const __m256i);
                    let eq = _mm256_and_si256(
                        _mm256_cmpeq_epi8(v0, v1),
                        _mm256_and_si256(_mm256_cmpeq_epi8(v0, v2), _mm256_cmpeq_epi8(v0, v3)),
                    );
                    let m = _mm256_movemask_epi8(eq) as u32;
                    _mm256_storeu_si256(out_p.add(out_len) as *mut __m256i, v0);
                    if m == 0 {
                        out_len += 32;
                        i += 32;
                        continue;
                    }
                    let tz = m.trailing_zeros() as usize;
                    let j = i + tz;
                    let c = *p.add(j);
                    let extra = *p.add(j + 4) as usize;
                    let run = 4 + extra;
                    out_len += tz;
                    if out_len + run + (avail - j) + 64 > out_cap {
                        let (np, nc) = grow_out_exact(out, out_len, run + (avail - j) + 320);
                        out_p = np;
                        out_cap = nc;
                    }
                    if run <= 32 {
                        let fill_v = _mm256_set1_epi8(c as i8);
                        _mm256_storeu_si256(out_p.add(out_len) as *mut __m256i, fill_v);
                        out_len += run;
                    } else {
                        std::ptr::write_bytes(out_p.add(out_len), c, run);
                        out_len += run;
                    }
                    i = j + 5;
                }
                let limit = if is_eof {
                    avail
                } else {
                    avail.saturating_sub(4)
                };
                while i < limit {
                    let c = *p.add(i);
                    if i + 3 < avail
                        && *p.add(i + 1) == c
                        && *p.add(i + 2) == c
                        && *p.add(i + 3) == c
                    {
                        if i + 4 >= avail {
                            out.set_len(out_len);
                            return Err(ErrorKind::RunUnfinished);
                        }
                        let run = 4 + (*p.add(i + 4) as usize);
                        if out_len + run + 64 > out_cap {
                            let (np, nc) = grow_out_exact(out, out_len, run + (avail - i) + 320);
                            out_p = np;
                            out_cap = nc;
                        }
                        std::ptr::write_bytes(out_p.add(out_len), c, run);
                        out_len += run;
                        i += 5;
                    } else {
                        if out_len + 1 > out_cap {
                            let (np, nc) = grow_out_exact(out, out_len, (avail - i) + 320);
                            out_p = np;
                            out_cap = nc;
                        }
                        *out_p.add(out_len) = c;
                        out_len += 1;
                        i += 1;
                    }
                }
                let rem = avail - i;
                if rem > 0 {
                    rev_buf.copy_within(avail - rem..avail, 0);
                }
                buf_len = rem;
                let avail_out = out_len - crc_at;
                if avail_out >= CRC_TILE {
                    let take = avail_out & !63;
                    out.set_len(out_len);
                    crc_state = crc::update_with(crc_state, &out[crc_at..crc_at + take], k);
                    crc_at += take;
                }
            }};
        }

        for &sg in chain {
            let b = bufs.get_unchecked(sg);
            let seg_take = b.len().min(rem_n);
            rem_n -= seg_take;
            let mut pos = seg_take;
            while pos > 0 {
                let space = BUF_CAP - buf_len;
                let chunk = pos.min(space);
                pos -= chunk;
                // Inline AVX2 reverse copy (zero vzeroupper!).
                let src = b.as_ptr().add(pos);
                let dst = rev_buf.as_mut_ptr().add(buf_len);
                let mut ri = 0usize;
                while ri + 32 <= chunk {
                    let v = _mm256_loadu_si256(src.add(ri) as *const __m256i);
                    let s = _mm256_shuffle_epi8(v, rev);
                    let r = _mm256_permute2x128_si256::<0x01>(s, s);
                    _mm256_storeu_si256(dst.add(chunk - ri - 32) as *mut __m256i, r);
                    ri += 32;
                }
                if ri < chunk {
                    reverse_copy_scalar(src.add(ri), dst, chunk - ri);
                }
                buf_len += chunk;
                if buf_len >= BUF_CAP - 32 {
                    let is_eof = rem_n == 0 && pos == 0;
                    decode_buf!(is_eof);
                }
            }
        }
        if buf_len > 0 {
            decode_buf!(true);
            debug_assert_eq!(buf_len, 0);
        }
        out.set_len(out_len);
        if crc_at < out_len {
            crc_state = crc::update_with(crc_state, &out[crc_at..], k);
        }
        Ok(!crc_state)
    }

    /// Single-pass fused reverse-assembly + AVX2 RLE1 + L1/L2-hot PCLMULQDQ CRC32!
    #[inline(never)]
    fn fused_rev_rle_crc<const BUF_CAP: usize, const CRC_TILE: usize>(
        s: &mut WalkScratch,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        let n = s.n;
        out.clear();
        if n == 0 {
            return Ok(crc::crc32(&[], k));
        }

        // 1. Collect segment chain in backward walk order, then reverse for forward order.
        s.chain.clear();
        let mut total = 0usize;
        let mut seg = s.first;
        while total < n {
            let blen = s.bufs[seg].len().min(n - total);
            s.chain.push(seg);
            total += blen;
            seg = s.next[seg];
            if seg == s.first {
                break;
            }
        }
        s.chain.reverse();

        if total == n {
            #[cfg(target_arch = "x86_64")]
            if k.avx2() {
                return unsafe {
                    fused_rev_rle_crc_buf_avx2::<BUF_CAP, CRC_TILE>(&s.bufs, &s.chain, n, out, k)
                };
            }
        }

        if out.capacity() < n + 64 {
            out.reserve_exact((n + 64) - out.capacity());
        }
        let mut rev_buf = [0u8; BUF_CAP];
        let mut carry = 0usize;
        let mut crc_state = !0u32;
        let mut crc_at = 0usize;
        let avx2 = k.avx2();

        let flush_crc = |crc_state: &mut u32, crc_at: &mut usize, out: &[u8]| {
            let avail_out = out.len() - *crc_at;
            if avail_out >= CRC_TILE {
                let take = avail_out & !63;
                *crc_state = crc::update_with(*crc_state, &out[*crc_at..*crc_at + take], k);
                *crc_at += take;
            }
        };

        if total == n {
            let mut rem_n = n;
            let mut buf_len = 0usize;
            for &sg in &s.chain {
                let b = &s.bufs[sg];
                let seg_take = b.len().min(rem_n);
                rem_n -= seg_take;
                let mut pos = seg_take;
                while pos > 0 {
                    let space = BUF_CAP - buf_len;
                    let chunk = pos.min(space);
                    pos -= chunk;
                    unsafe {
                        reverse_copy_fast(
                            b.as_ptr().add(pos),
                            rev_buf.as_mut_ptr().add(buf_len),
                            chunk,
                            avx2,
                        );
                    }
                    buf_len += chunk;
                    if buf_len >= BUF_CAP - 32 {
                        let is_eof = rem_n == 0 && pos == 0;
                        let rem = rle_decode_chunk(&rev_buf, buf_len, is_eof, out, k)?;
                        if rem > 0 {
                            rev_buf.copy_within(buf_len - rem..buf_len, 0);
                        }
                        buf_len = rem;
                        flush_crc(&mut crc_state, &mut crc_at, out);
                    }
                }
            }
            if buf_len > 0 {
                let rem = rle_decode_chunk(&rev_buf, buf_len, true, out, k)?;
                debug_assert_eq!(rem, 0);
            }
            let _ = carry;
        } else {
            let m = total;
            s.cycle_tmp.clear();
            s.cycle_tmp.reserve(m);
            let dst = s.cycle_tmp.as_mut_ptr();
            let mut off = 0usize;
            for &sg in &s.chain {
                let b = &s.bufs[sg];
                let len = b.len();
                unsafe {
                    reverse_copy_fast(b.as_ptr(), dst.add(off), len, avx2);
                }
                off += len;
            }
            unsafe { s.cycle_tmp.set_len(m) };

            let mut rem_n = n;
            let mut cyc_pos = 0usize;
            while rem_n > 0 {
                let mut fill = carry;
                while fill < BUF_CAP && rem_n > 0 {
                    let take = (m - cyc_pos).min(BUF_CAP - fill).min(rem_n);
                    rev_buf[fill..fill + take]
                        .copy_from_slice(&s.cycle_tmp[cyc_pos..cyc_pos + take]);
                    fill += take;
                    rem_n -= take;
                    cyc_pos += take;
                    if cyc_pos == m {
                        cyc_pos = 0;
                    }
                }
                let is_eof = rem_n == 0;
                let rem = rle_decode_chunk(&rev_buf, fill, is_eof, out, k)?;
                if rem > 0 {
                    rev_buf.copy_within(fill - rem..fill, 0);
                }
                carry = rem;
                flush_crc(&mut crc_state, &mut crc_at, out);
            }
        }

        if crc_at < out.len() {
            crc_state = crc::update_with(crc_state, &out[crc_at..], k);
        }
        Ok(!crc_state)
    }

    /// Zero-copy in-register AVX2 fused reverse + RLE1 + L2-hot CRC32!
    /// Entire segment-chain traversal runs inside ONE `#[target_feature(enable = "avx2,bmi1")]`
    /// function: ZERO `rev_buf` staging, ZERO per-segment `vzeroupper`, ONE 32B load + ONE 32B
    /// store per 32 bytes via `vpshufb + vperm2i128 + vpalignr`!
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,bmi1")]
    unsafe fn fused_rev_rle_crc_reg_avx2<const CRC_TILE: usize>(
        bufs: &[Vec<u8>],
        chain: &[usize],
        n: usize,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        use std::arch::x86_64::*;
        let rev = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        out.clear();
        out.reserve(n + 320);
        let mut out_len = 0usize;
        let mut out_cap = out.capacity();
        let mut out_p = out.as_mut_ptr();

        let mut carry_buf = [0u8; 96];
        let mut carry_len = 0usize;
        let mut crc_state = !0u32;
        let mut crc_at = 0usize;
        let mut rem_n = n;

        for &sg in chain {
            let b = bufs.get_unchecked(sg);
            let seg_take = b.len().min(rem_n);
            rem_n -= seg_take;
            let bp = b.as_ptr();
            let mut pos = seg_take;

            // Step 1: Resolve any <=4-byte carry from the previous segment!
            if carry_len > 0 {
                let take = pos.min(36);
                pos -= take;
                reverse_copy_scalar(bp.add(pos), carry_buf.as_mut_ptr().add(carry_len), take);
                let avail = carry_len + take;
                let mut i = 0usize;
                while i < carry_len && i + 5 <= avail {
                    let c = *carry_buf.get_unchecked(i);
                    if *carry_buf.get_unchecked(i + 1) == c
                        && *carry_buf.get_unchecked(i + 2) == c
                        && *carry_buf.get_unchecked(i + 3) == c
                    {
                        let run = 4 + (*carry_buf.get_unchecked(i + 4) as usize);
                        if out_len + run + 64 > out_cap {
                            out.set_len(out_len);
                            out.reserve(run + rem_n + pos + 320);
                            out_cap = out.capacity();
                            out_p = out.as_mut_ptr();
                        }
                        std::ptr::write_bytes(out_p.add(out_len), c, run);
                        out_len += run;
                        i += 5;
                    } else {
                        if out_len + 64 > out_cap {
                            out.set_len(out_len);
                            out.reserve(rem_n + pos + 320);
                            out_cap = out.capacity();
                            out_p = out.as_mut_ptr();
                        }
                        *out_p.add(out_len) = c;
                        out_len += 1;
                        i += 1;
                    }
                }
                if i >= carry_len {
                    // All old carry bytes resolved! Return any unconsumed bytes from `b` back to `pos`.
                    let unconsumed = avail - i;
                    pos += unconsumed;
                    carry_len = 0;
                } else {
                    // Very short segment (`avail < carry_len + 5`): keep remaining bytes in carry_buf.
                    let rem = avail - i;
                    carry_buf.copy_within(i..avail, 0);
                    carry_len = rem;
                    continue;
                }
            }

            // Step 2: Zero-copy in-register AVX2 reverse + RLE1 loop (`pos >= 36`)!
            // Key symmetry: a 4-byte equal run `[c,c,c,c]` is palindromic! So we test 4 equal
            // bytes BEFORE reversing (`raw0..raw3` at `pos-32..pos-35`), find the earliest
            // forward run via `tz = m.leading_zeros()`, and reverse ONLY `raw0` ONCE!
            // Zero wasted `nxt` work on `m != 0`, zero `bridge`/`vpalignr`, zero `rev_buf`,
            // zero `vzeroupper` per segment!
            while pos >= 36 {
                let raw0 = _mm256_loadu_si256(bp.add(pos - 32) as *const __m256i);
                let raw1 = _mm256_loadu_si256(bp.add(pos - 33) as *const __m256i);
                let raw2 = _mm256_loadu_si256(bp.add(pos - 34) as *const __m256i);
                let raw3 = _mm256_loadu_si256(bp.add(pos - 35) as *const __m256i);

                let eq = _mm256_and_si256(
                    _mm256_cmpeq_epi8(raw0, raw1),
                    _mm256_and_si256(_mm256_cmpeq_epi8(raw0, raw2), _mm256_cmpeq_epi8(raw0, raw3)),
                );
                let m = _mm256_movemask_epi8(eq) as u32;

                let s0 = _mm256_shuffle_epi8(raw0, rev);
                let cur = _mm256_permute2x128_si256::<0x01>(s0, s0);

                if out_len + 64 > out_cap {
                    out.set_len(out_len);
                    out.reserve(rem_n + pos + 320);
                    out_cap = out.capacity();
                    out_p = out.as_mut_ptr();
                }
                _mm256_storeu_si256(out_p.add(out_len) as *mut __m256i, cur);

                if m == 0 {
                    out_len += 32;
                    pos -= 32;
                    continue;
                }

                // Highest set bit in `m` is `k = 31 - tz` -> earliest forward run offset `tz`!
                let tz = m.leading_zeros() as usize;
                let c = *bp.add(pos - 1 - tz);
                let extra = *bp.add(pos - 5 - tz) as usize;
                let run = 4 + extra;
                out_len += tz;
                if run <= 32 {
                    let fill_v = _mm256_set1_epi8(c as i8);
                    _mm256_storeu_si256(out_p.add(out_len) as *mut __m256i, fill_v);
                    out_len += run;
                } else {
                    if out_len + run + 64 > out_cap {
                        out.set_len(out_len);
                        out.reserve(run + rem_n + pos + 320);
                        out_cap = out.capacity();
                        out_p = out.as_mut_ptr();
                    }
                    std::ptr::write_bytes(out_p.add(out_len), c, run);
                    out_len += run;
                }
                pos -= tz + 5;
            }

            // Step 3: Segment tail (`pos < 64`, `carry_len == 0`): reverse into carry_buf,
            // decode down to `<= 4` bytes (leaving carry for next segment).
            if pos > 0 {
                reverse_copy_scalar(bp, carry_buf.as_mut_ptr(), pos);
                let limit = pos.saturating_sub(4);
                let mut i = 0usize;
                while i < limit {
                    let c = *carry_buf.get_unchecked(i);
                    if *carry_buf.get_unchecked(i + 1) == c
                        && *carry_buf.get_unchecked(i + 2) == c
                        && *carry_buf.get_unchecked(i + 3) == c
                    {
                        let run = 4 + (*carry_buf.get_unchecked(i + 4) as usize);
                        if out_len + run + 64 > out_cap {
                            out.set_len(out_len);
                            out.reserve(run + rem_n + 320);
                            out_cap = out.capacity();
                            out_p = out.as_mut_ptr();
                        }
                        std::ptr::write_bytes(out_p.add(out_len), c, run);
                        out_len += run;
                        i += 5;
                    } else {
                        if out_len + 64 > out_cap {
                            out.set_len(out_len);
                            out.reserve(rem_n + 320);
                            out_cap = out.capacity();
                            out_p = out.as_mut_ptr();
                        }
                        *out_p.add(out_len) = c;
                        out_len += 1;
                        i += 1;
                    }
                }
                let rem = pos - i;
                if rem > 0 {
                    carry_buf.copy_within(i..pos, 0);
                }
                carry_len = rem;
            }

            // Step 4: Fold L2-hot CRC32 tile while output bytes are warm in L1/L2!
            let avail_out = out_len - crc_at;
            if avail_out >= CRC_TILE {
                let take = avail_out & !63;
                out.set_len(out_len);
                crc_state = crc::update_with(crc_state, &out[crc_at..crc_at + take], k);
                crc_at += take;
            }
        }

        // Step 5: Block EOF — drain final `<= 4` carry bytes!
        let mut i = 0usize;
        while i < carry_len {
            let c = *carry_buf.get_unchecked(i);
            if i + 3 < carry_len
                && *carry_buf.get_unchecked(i + 1) == c
                && *carry_buf.get_unchecked(i + 2) == c
                && *carry_buf.get_unchecked(i + 3) == c
            {
                if i + 4 >= carry_len {
                    out.set_len(out_len);
                    return Err(ErrorKind::RunUnfinished);
                }
                let run = 4 + (*carry_buf.get_unchecked(i + 4) as usize);
                if out_len + run > out_cap {
                    out.set_len(out_len);
                    out.reserve(run);
                    out_cap = out.capacity();
                    out_p = out.as_mut_ptr();
                }
                std::ptr::write_bytes(out_p.add(out_len), c, run);
                out_len += run;
                i += 5;
            } else {
                if out_len + 1 > out_cap {
                    out.set_len(out_len);
                    out.reserve(16);
                    out_cap = out.capacity();
                    out_p = out.as_mut_ptr();
                }
                *out_p.add(out_len) = c;
                out_len += 1;
                i += 1;
            }
        }
        out.set_len(out_len);
        if crc_at < out_len {
            crc_state = crc::update_with(crc_state, &out[crc_at..], k);
        }
        Ok(!crc_state)
    }

    #[inline(never)]
    fn fused_rev_rle_crc_reg<const CRC_TILE: usize>(
        s: &mut WalkScratch,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        let n = s.n;
        if n == 0 {
            out.clear();
            return Ok(crc::crc32(&[], k));
        }
        s.chain.clear();
        let mut total = 0usize;
        let mut seg = s.first;
        while total < n {
            let blen = s.bufs[seg].len().min(n - total);
            s.chain.push(seg);
            total += blen;
            seg = s.next[seg];
            if seg == s.first {
                break;
            }
        }
        s.chain.reverse();
        if total == n {
            #[cfg(target_arch = "x86_64")]
            if k.avx2() {
                return unsafe {
                    fused_rev_rle_crc_reg_avx2::<CRC_TILE>(&s.bufs, &s.chain, n, out, k)
                };
            }
        }
        // Short-cycle or scalar fallback: reuse buffered streaming fused decoder.
        fused_rev_rle_crc::<4096, CRC_TILE>(s, out, k)
    }

    /// Short-block (`n < MIN_SEGMENTED`) path: plain `walk_back` + RLE1 + CRC32.
    #[inline(never)]
    fn short_walk_rle_crc(
        tt: &[u32],
        orig_ptr: u32,
        tmp: &mut Vec<u8>,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        let n = tt.len();
        tmp.clear();
        tmp.resize(n, 0);
        let mut i = orig_ptr as usize;
        let mut m = n;
        for step in 0..n {
            let e = tt[i];
            tmp[n - 1 - step] = e as u8;
            i = (e >> 8) as usize;
            if i == orig_ptr as usize {
                m = step + 1;
                break;
            }
        }
        if m > 0 && m < n {
            tmp.copy_within(n - m.., 0);
            let mut have = m;
            while have < n {
                let take = have.min(n - have);
                tmp.copy_within(..take, have);
                have += take;
            }
        }
        out.clear();
        rle::unrle(tmp, out, k)?;
        Ok(crc::crc32(out, k))
    }

    /// Reference `assemble_reversed` on `WalkScratch` (for isolated walk & post-walk timing).
    #[inline(never)]
    fn assemble_reversed_walk_scratch(s: &WalkScratch, out: &mut Vec<u8>, k: Kernels) {
        let n = s.n;
        out.clear();
        out.reserve(n);
        let dst = out.as_mut_ptr();
        let mut at = n;
        let mut seg = s.first;
        let avx2 = k.avx2();
        while at > 0 {
            let b = &s.bufs[seg];
            let len = b.len().min(at);
            at -= len;
            unsafe {
                reverse_copy_fast(b.as_ptr(), dst.add(at), len, avx2);
            }
            seg = s.next[seg];
            if seg == s.first {
                break;
            }
        }
        let m = n - at;
        if m > 0 && m < n {
            unsafe {
                std::ptr::copy(dst.add(n - m), dst, m);
                let mut have = m;
                while have < n {
                    let take = have.min(n - have);
                    std::ptr::copy_nonoverlapping(dst, dst.add(have), take);
                    have += take;
                }
            }
        }
        unsafe { out.set_len(n) };
    }

    // ---- Corpus Loading & Route Classification ----

    fn load_corpus(path: &Path, k: Kernels) -> (String, Vec<BlockData>) {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("corpus")
            .to_string();
        let raw = std::fs::read(path).expect("read bz2");
        let mut scanner = Scanner::new(SliceSource(&raw));
        let mut syms_buf = Vec::new();
        let mut blocks = Vec::new();
        let mut open = OpenCells::default();
        let mut seg = Segmented::default();
        let mut tmp_out = Vec::new();
        let mut tmp_ll = Vec::new();
        let mut tmp_counts = [0u32; 256];

        loop {
            match scanner.next(syms_buf).expect("scan") {
                Event::Done => break,
                _ if false => unreachable!(),
                Event::Block { symbols: b, .. } => {
                    let crc_expected = b.crc;
                    let orig_ptr = b.orig_ptr;
                    let mut runs = Runs::default();
                    let route;
                    let n;
                    let ll;
                    let counts;
                    let runs_vec;

                    if runs::takes_runs(&b) {
                        n = runs::front(&b, &mut runs, k).expect("runs::front");
                        let rec = runs.records();
                        let open_admits = if n <= 900_000
                            && (orig_ptr as usize) < n
                            && rec.len() * 64 >= n
                            && rec.len() * 25 < n * 7
                        {
                            if rec.len() * 20 < n * 3 {
                                true
                            } else {
                                let (mut m, mut cur, mut len) = (0usize, u16::MAX, 0usize);
                                for &x in rec {
                                    let (byte, l) = (x >> 8, (x & 0xff) as usize);
                                    if byte == cur {
                                        len += l;
                                    } else {
                                        if len > 0 && len <= 16 {
                                            m += len;
                                        }
                                        cur = byte;
                                        len = l;
                                    }
                                }
                                if len > 0 && len <= 16 {
                                    m += len;
                                }
                                m < 256_000usize.min(n * 3 / 10)
                            }
                        } else {
                            false
                        };
                        route = if open_admits {
                            tmp_out.clear();
                            assert!(open.invert_runs(&runs, n, orig_ptr, &mut tmp_out, k));
                            ProdRoute::Open
                        } else {
                            ProdRoute::RunsFallback
                        };
                        runs_vec = rec.to_vec();
                        let mut cnt = [0u32; 256];
                        for &r in &runs_vec {
                            cnt[(r >> 8) as usize] += (r & 0xff) as u32;
                        }
                        counts = cnt;
                        ll = Vec::new();
                    } else {
                        route = ProdRoute::Column;
                        n = mtf::mtf(&b, &mut tmp_ll, &mut tmp_counts, k).expect("mtf");
                        ll = tmp_ll.clone();
                        counts = tmp_counts;
                        runs_vec = Vec::new();
                    }

                    // Compute exact ground truth output via shipped Segmented + rle + crc.
                    let mut inv = Vec::new();
                    if route == ProdRoute::Column {
                        seg.invert(&ll, &counts, orig_ptr, &mut inv);
                    } else {
                        assert!(seg.invert_runs(&runs, n, orig_ptr, &mut inv, k));
                    }
                    let mut want_final = Vec::new();
                    rle::unrle(&inv, &mut want_final, k).expect("rle::unrle");
                    let got_crc = crc::crc32(&want_final, k);
                    assert_eq!(got_crc, crc_expected, "ground truth CRC mismatch in {name}");

                    blocks.push(BlockData {
                        corpus: name.clone(),
                        route,
                        n,
                        orig_ptr,
                        crc_expected,
                        ll,
                        counts,
                        runs,
                        runs_vec,
                        want_final,
                    });
                    syms_buf = b.syms;
                }
            }
        }
        (name, blocks)
    }

    fn median(mut v: Vec<f64>) -> f64 {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    }

    fn stats(v: &[f64]) -> (f64, f64, f64) {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (s[s.len() / 2], s[0], s[s.len() - 1])
    }

    // ---- Full Pipeline Arms ----

    #[derive(Clone, Copy, Debug)]
    enum FullArm {
        Shipped,
        ShippedLocal,
        M1FusedBuf4k32k,
        M1FusedReg32k,
        M1FusedReg64k,
        M2BuildOnly,
        M3WalkL48,
        M3WalkCmov32,
        M4BestL32,
        M4BestL48,
    }

    impl FullArm {
        fn name(self) -> &'static str {
            match self {
                FullArm::Shipped => "shipped",
                FullArm::ShippedLocal => "shipped_local",
                FullArm::M1FusedBuf4k32k => "M1_fused_buf4k",
                FullArm::M1FusedReg32k => "M1_fused_reg32k",
                FullArm::M1FusedReg64k => "M1_fused_reg64k",
                FullArm::M2BuildOnly => "M2_preshift_build",
                FullArm::M3WalkL48 => "M3_walk_L48",
                FullArm::M3WalkCmov32 => "M3_walk_cmov32",
                FullArm::M4BestL32 => "M4_buf4k_u64",
                FullArm::M4BestL48 => "M4_reg_u64",
            }
        }
    }

    #[derive(Default)]
    struct ArmScratch {
        seg_shipped: Segmented,
        tt: Vec<u32>,
        ws: WalkScratch,
        rle_runs: Vec<RleRun>,
        stage: Vec<u8>,
        out: Vec<u8>,
    }

    impl ArmScratch {
        fn reset_allocs(&mut self) {
            *self = Self::default();
        }
        fn custom_heap_bytes(&self) -> usize {
            self.tt.capacity() * 4
                + self.ws.heap_bytes()
                + self.rle_runs.capacity() * std::mem::size_of::<RleRun>()
                + self.stage.capacity()
                + self.out.capacity()
        }
    }

    fn run_block_arm(
        arm: FullArm,
        b: &BlockData,
        sc: &mut ArmScratch,
        k: Kernels,
        verify: bool,
    ) -> u32 {
        let crc = match arm {
            FullArm::Shipped => {
                if b.route == ProdRoute::Column {
                    sc.seg_shipped
                        .invert(&b.ll, &b.counts, b.orig_ptr, &mut sc.stage);
                    sc.out.clear();
                    rle::unrle(&sc.stage, &mut sc.out, k).expect("unrle");
                    crc::crc32(&sc.out, k)
                } else {
                    assert!(sc
                        .seg_shipped
                        .invert_runs(&b.runs, b.n, b.orig_ptr, &mut sc.out, k));
                    shipped_unrle_in_place(&mut sc.out, &mut sc.rle_runs, &mut sc.stage, k)
                        .expect("unrle_in_place")
                }
            }
            FullArm::ShippedLocal => {
                if b.route == ProdRoute::Column {
                    assert!(build_lf_shipped(&b.ll, &b.counts, &mut sc.tt));
                } else {
                    fill_shipped_raw(&b.runs_vec, &b.counts, b.n, &mut sc.tt);
                }
                if b.n < MIN_SEGMENTED {
                    short_walk_rle_crc(&sc.tt, b.orig_ptr, &mut sc.stage, &mut sc.out, k)
                        .expect("short")
                } else {
                    walk_room_std::<32, PF_L1, true>(
                        &mut sc.tt,
                        b.orig_ptr,
                        &mut sc.ws,
                        FLAGGED_SEGMENTS,
                    );
                    if b.route == ProdRoute::Column {
                        assemble_reversed_walk_scratch(&sc.ws, &mut sc.stage, k);
                        sc.out.clear();
                        rle::unrle(&sc.stage, &mut sc.out, k).expect("unrle");
                        crc::crc32(&sc.out, k)
                    } else {
                        assemble_reversed_walk_scratch(&sc.ws, &mut sc.out, k);
                        shipped_unrle_in_place(&mut sc.out, &mut sc.rle_runs, &mut sc.stage, k)
                            .expect("unrle_in_place")
                    }
                }
            }
            FullArm::M1FusedBuf4k32k | FullArm::M1FusedReg32k | FullArm::M1FusedReg64k => {
                if b.route == ProdRoute::Column {
                    assert!(build_lf_shipped(&b.ll, &b.counts, &mut sc.tt));
                } else {
                    fill_shipped_raw(&b.runs_vec, &b.counts, b.n, &mut sc.tt);
                }
                if b.n < MIN_SEGMENTED {
                    short_walk_rle_crc(&sc.tt, b.orig_ptr, &mut sc.stage, &mut sc.out, k)
                        .expect("short")
                } else {
                    walk_room_std::<32, PF_L1, true>(
                        &mut sc.tt,
                        b.orig_ptr,
                        &mut sc.ws,
                        FLAGGED_SEGMENTS,
                    );
                    match arm {
                        FullArm::M1FusedBuf4k32k => {
                            fused_rev_rle_crc::<4096, 32768>(&mut sc.ws, &mut sc.out, k)
                                .expect("fused")
                        }
                        FullArm::M1FusedReg32k => {
                            fused_rev_rle_crc_reg::<32768>(&mut sc.ws, &mut sc.out, k)
                                .expect("fused_reg")
                        }
                        FullArm::M1FusedReg64k => {
                            fused_rev_rle_crc_reg::<65536>(&mut sc.ws, &mut sc.out, k)
                                .expect("fused_reg")
                        }
                        _ => unreachable!(),
                    }
                }
            }
            FullArm::M2BuildOnly => {
                if b.route == ProdRoute::Column {
                    assert!(build_lf_preshift_norun(&b.ll, &b.counts, &mut sc.tt));
                } else {
                    fill_preshift_unroll2(&b.runs_vec, &b.counts, b.n, &mut sc.tt);
                }
                if b.n < MIN_SEGMENTED {
                    short_walk_rle_crc(&sc.tt, b.orig_ptr, &mut sc.stage, &mut sc.out, k)
                        .expect("short")
                } else {
                    walk_room_std::<32, PF_L1, true>(
                        &mut sc.tt,
                        b.orig_ptr,
                        &mut sc.ws,
                        FLAGGED_SEGMENTS,
                    );
                    if b.route == ProdRoute::Column {
                        assemble_reversed_walk_scratch(&sc.ws, &mut sc.stage, k);
                        sc.out.clear();
                        rle::unrle(&sc.stage, &mut sc.out, k).expect("unrle");
                        crc::crc32(&sc.out, k)
                    } else {
                        assemble_reversed_walk_scratch(&sc.ws, &mut sc.out, k);
                        shipped_unrle_in_place(&mut sc.out, &mut sc.rle_runs, &mut sc.stage, k)
                            .expect("unrle_in_place")
                    }
                }
            }
            FullArm::M3WalkL48 | FullArm::M3WalkCmov32 => {
                if b.route == ProdRoute::Column {
                    assert!(build_lf_shipped(&b.ll, &b.counts, &mut sc.tt));
                } else {
                    fill_shipped_raw(&b.runs_vec, &b.counts, b.n, &mut sc.tt);
                }
                if b.n < MIN_SEGMENTED {
                    short_walk_rle_crc(&sc.tt, b.orig_ptr, &mut sc.stage, &mut sc.out, k)
                        .expect("short")
                } else {
                    match arm {
                        FullArm::M3WalkL48 => walk_room_std::<48, PF_L1, false>(
                            &mut sc.tt,
                            b.orig_ptr,
                            &mut sc.ws,
                            FLAGGED_SEGMENTS,
                        ),
                        FullArm::M3WalkCmov32 => walk_room_cmov::<32, PF_L1, false, 256>(
                            &mut sc.tt,
                            b.orig_ptr,
                            &mut sc.ws,
                            FLAGGED_SEGMENTS,
                        ),
                        _ => unreachable!(),
                    }
                    if b.route == ProdRoute::Column {
                        assemble_reversed_walk_scratch(&sc.ws, &mut sc.stage, k);
                        sc.out.clear();
                        rle::unrle(&sc.stage, &mut sc.out, k).expect("unrle");
                        crc::crc32(&sc.out, k)
                    } else {
                        assemble_reversed_walk_scratch(&sc.ws, &mut sc.out, k);
                        shipped_unrle_in_place(&mut sc.out, &mut sc.rle_runs, &mut sc.stage, k)
                            .expect("unrle_in_place")
                    }
                }
            }
            FullArm::M4BestL32 | FullArm::M4BestL48 => {
                if b.route == ProdRoute::Column {
                    assert!(build_lf_preshift_norun(&b.ll, &b.counts, &mut sc.tt));
                } else {
                    fill_shipped_raw(&b.runs_vec, &b.counts, b.n, &mut sc.tt);
                }
                if b.n < MIN_SEGMENTED {
                    short_walk_rle_crc(&sc.tt, b.orig_ptr, &mut sc.stage, &mut sc.out, k)
                        .expect("short")
                } else {
                    walk_room_std::<32, PF_L1, false>(
                        &mut sc.tt,
                        b.orig_ptr,
                        &mut sc.ws,
                        FLAGGED_SEGMENTS,
                    );
                    if matches!(arm, FullArm::M4BestL32) {
                        fused_rev_rle_crc::<4096, 65536>(&mut sc.ws, &mut sc.out, k)
                            .expect("fused_buf")
                    } else {
                        fused_rev_rle_crc_reg::<65536>(&mut sc.ws, &mut sc.out, k)
                            .expect("fused_reg")
                    }
                }
            }
        };
        if verify {
            assert_eq!(
                crc,
                b.crc_expected,
                "CRC mismatch on {} ({}) n={}",
                arm.name(),
                b.corpus,
                b.n
            );
            assert_eq!(
                sc.out.len(),
                b.want_final.len(),
                "len mismatch on {} ({}) n={}",
                arm.name(),
                b.corpus,
                b.n
            );
            assert!(
                sc.out == b.want_final,
                "byte mismatch on {} ({}) n={}",
                arm.name(),
                b.corpus,
                b.n
            );
        }
        crc
    }

    pub fn main() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        if args.is_empty() {
            eprintln!("usage: bwt_segmented_dissection <file.bz2>...");
            std::process::exit(1);
        }
        let k = Kernels::detect();
        println!(
            "Kernels: {k:?}, target_avx2={}",
            cfg!(target_feature = "avx2")
        );

        let mut corpora: Vec<(String, Vec<BlockData>)> = Vec::new();
        for a in &args {
            corpora.push(load_corpus(Path::new(a), k));
        }

        // =====================================================================
        // STAGE 0: Route & Structural Census + Shipped Sub-Stage Breakdown
        // =====================================================================
        println!("\n=== STAGE 0: ROUTE & STRUCTURAL CENSUS ===");
        println!(
            "{:<10} {:>5} {:>5} {:>5} {:>5} {:>5} {:>10} {:>8} {:>8} {:>8}",
            "corpus",
            "blks",
            "open",
            "runs",
            "col",
            "runt",
            "seg_rows",
            "r<=8%",
            "rle_runs",
            "shift>0"
        );
        let mut rle_runs_tmp = Vec::new();
        for (name, blks) in &corpora {
            let mut n_open = 0usize;
            let mut n_runs = 0usize;
            let mut n_col = 0usize;
            let mut n_runt = 0usize;
            let mut seg_rows = 0usize;
            let mut rec_tot = 0usize;
            let mut rec_le8 = 0usize;
            let mut rle_tot = 0usize;
            let mut shift_pos = 0usize;
            let mut seg_tmp = Segmented::default();
            let mut inv_tmp = Vec::new();
            for b in blks {
                match b.route {
                    ProdRoute::Open => n_open += 1,
                    ProdRoute::RunsFallback => n_runs += 1,
                    ProdRoute::Column => n_col += 1,
                }
                let is_seg = b.route != ProdRoute::Open || name == "runs";
                if is_seg {
                    if b.n < MIN_SEGMENTED {
                        n_runt += 1;
                    }
                    seg_rows += b.n;
                    for &r in &b.runs_vec {
                        rec_tot += 1;
                        if (r & 0xff) <= 8 {
                            rec_le8 += 1;
                        }
                    }
                    inv_tmp.clear();
                    if b.route == ProdRoute::Column {
                        seg_tmp.invert(&b.ll, &b.counts, b.orig_ptr, &mut inv_tmp);
                    } else {
                        seg_tmp.invert_runs(&b.runs, b.n, b.orig_ptr, &mut inv_tmp, k);
                    }
                    shipped_find_runs(&inv_tmp, &mut rle_runs_tmp, k).unwrap();
                    rle_tot += rle_runs_tmp.len();
                    let (mut grown, mut shift) = (0isize, 0isize);
                    for &(_, _, extra) in &rle_runs_tmp {
                        grown += extra as isize - 1;
                        shift = shift.max(grown);
                    }
                    if shift > 0 {
                        shift_pos += 1;
                    }
                }
            }
            let rle8_pct = if rec_tot > 0 {
                100.0 * rec_le8 as f64 / rec_tot as f64
            } else {
                0.0
            };
            println!(
                "{:<10} {:>5} {:>5} {:>5} {:>5} {:>5} {:>10} {:>7.2}% {:>8} {:>8}",
                name,
                blks.len(),
                n_open,
                n_runs,
                n_col,
                n_runt,
                seg_rows,
                rle8_pct,
                rle_tot,
                shift_pos
            );
        }

        // Build per-corpus Segmented block lists (note: for `runs`, force all blocks through Segmented
        // as guard; for `wikidata`, take non-Open blocks!).
        let seg_corpora: Vec<(String, Vec<&BlockData>)> = corpora
            .iter()
            .map(|(name, blks)| {
                let list: Vec<&BlockData> = blks
                    .iter()
                    .filter(|b| b.route != ProdRoute::Open || name == "runs")
                    .collect();
                (name.clone(), list)
            })
            .collect();

        // Shipped sub-stage breakdown (best of 3 warm thread CPU ms).
        println!("\n=== STAGE 0B: SHIPPED SUB-STAGE BREAKDOWN (best-of-3 warm thread CPU ms) ===");
        println!(
            "{:<10} {:>5} {:>8} {:>8} {:>8} {:>8} {:>8} | {:>5} {:>5} {:>5}",
            "corpus", "blks", "build", "walk", "asm", "rle+crc", "total", "bld%", "wlk%", "post%"
        );
        for (name, blks) in &seg_corpora {
            let mut bld_best = f64::INFINITY;
            let mut wlk_best = f64::INFINITY;
            let mut asm_best = f64::INFINITY;
            let mut rle_best = f64::INFINITY;
            let mut tt = Vec::new();
            let mut ws = WalkScratch::default();
            let mut stage = Vec::new();
            let mut out = Vec::new();
            let mut rruns = Vec::new();
            for _ in 0..3 {
                let (mut tb, mut tw, mut ta, mut tr) = (0u64, 0u64, 0u64, 0u64);
                for b in blks {
                    let t0 = thread_cpu_ns();
                    if b.route == ProdRoute::Column {
                        assert!(build_lf_shipped(&b.ll, &b.counts, &mut tt));
                    } else {
                        fill_shipped_raw(&b.runs_vec, &b.counts, b.n, &mut tt);
                    }
                    let t1 = thread_cpu_ns();
                    tb += t1 - t0;
                    if b.n < MIN_SEGMENTED {
                        let c =
                            short_walk_rle_crc(&tt, b.orig_ptr, &mut stage, &mut out, k).unwrap();
                        assert_eq!(c, b.crc_expected);
                        let t2 = thread_cpu_ns();
                        tw += t2 - t1;
                    } else {
                        walk_room_std::<32, PF_L1, true>(
                            &mut tt,
                            b.orig_ptr,
                            &mut ws,
                            FLAGGED_SEGMENTS,
                        );
                        let t2 = thread_cpu_ns();
                        tw += t2 - t1;
                        if b.route == ProdRoute::Column {
                            assemble_reversed_walk_scratch(&ws, &mut stage, k);
                            let t3 = thread_cpu_ns();
                            ta += t3 - t2;
                            out.clear();
                            rle::unrle(&stage, &mut out, k).unwrap();
                            let c = crc::crc32(&out, k);
                            assert_eq!(c, b.crc_expected);
                            let t4 = thread_cpu_ns();
                            tr += t4 - t3;
                        } else {
                            assemble_reversed_walk_scratch(&ws, &mut out, k);
                            let t3 = thread_cpu_ns();
                            ta += t3 - t2;
                            let c = shipped_unrle_in_place(&mut out, &mut rruns, &mut stage, k)
                                .unwrap();
                            assert_eq!(c, b.crc_expected);
                            let t4 = thread_cpu_ns();
                            tr += t4 - t3;
                        }
                    }
                }
                bld_best = bld_best.min(tb as f64 / 1e6);
                wlk_best = wlk_best.min(tw as f64 / 1e6);
                asm_best = asm_best.min(ta as f64 / 1e6);
                rle_best = rle_best.min(tr as f64 / 1e6);
            }
            let tot = bld_best + wlk_best + asm_best + rle_best;
            let post = asm_best + rle_best;
            println!(
                "{:<10} {:>5} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>8.2} | {:>4.1}% {:>4.1}% {:>4.1}%",
                name,
                blks.len(),
                bld_best,
                wlk_best,
                asm_best,
                rle_best,
                tot,
                100.0 * bld_best / tot.max(1e-9),
                100.0 * wlk_best / tot.max(1e-9),
                100.0 * post / tot.max(1e-9),
            );
        }

        // =====================================================================
        // STAGE 1: Isolated Lever Micro-Screens (exactness + warm CPU ms)
        // =====================================================================
        println!("\n=== STAGE 1A: ISOLATED BUILD SCREEN (M2, median of 5 warm CPU ms) ===");
        for (name, blks) in &seg_corpora {
            if !matches!(name.as_str(), "text" | "random" | "records" | "pattern") {
                continue;
            }
            let is_col = blks.first().is_some_and(|b| b.route == ProdRoute::Column);
            let mut tt_ref = Vec::new();
            let mut tt = Vec::new();
            // Verify bit-exact TVector on every block first!
            for b in blks {
                if is_col {
                    assert!(build_lf_shipped(&b.ll, &b.counts, &mut tt_ref));
                    assert!(build_lf_preshift_run8(&b.ll, &b.counts, &mut tt));
                    assert_eq!(tt, tt_ref, "M2a mismatch on {name}");
                    assert!(build_lf_preshift_norun(&b.ll, &b.counts, &mut tt));
                    assert_eq!(tt, tt_ref, "M2b mismatch on {name}");
                } else {
                    fill_shipped_raw(&b.runs_vec, &b.counts, b.n, &mut tt_ref);
                    fill_preshift_len8(&b.runs_vec, &b.counts, b.n, &mut tt);
                    assert_eq!(tt, tt_ref, "M2c mismatch on {name}");
                    fill_preshift_unroll2(&b.runs_vec, &b.counts, b.n, &mut tt);
                    assert_eq!(tt, tt_ref, "M2d mismatch on {name}");
                }
            }
            let mut t0_v = Vec::new();
            let mut t1_v = Vec::new();
            let mut t2_v = Vec::new();
            for _ in 0..5 {
                let s0 = thread_cpu_ns();
                for b in blks {
                    if is_col {
                        build_lf_shipped(&b.ll, &b.counts, &mut tt);
                    } else {
                        fill_shipped_raw(&b.runs_vec, &b.counts, b.n, &mut tt);
                    }
                }
                t0_v.push((thread_cpu_ns() - s0) as f64 / 1e6);

                let s1 = thread_cpu_ns();
                for b in blks {
                    if is_col {
                        build_lf_preshift_run8(&b.ll, &b.counts, &mut tt);
                    } else {
                        fill_preshift_len8(&b.runs_vec, &b.counts, b.n, &mut tt);
                    }
                }
                t1_v.push((thread_cpu_ns() - s1) as f64 / 1e6);

                let s2 = thread_cpu_ns();
                for b in blks {
                    if is_col {
                        build_lf_preshift_norun(&b.ll, &b.counts, &mut tt);
                    } else {
                        fill_preshift_unroll2(&b.runs_vec, &b.counts, b.n, &mut tt);
                    }
                }
                t2_v.push((thread_cpu_ns() - s2) as f64 / 1e6);
            }
            let m0 = median(t0_v);
            let m1 = median(t1_v);
            let m2 = median(t2_v);
            if is_col {
                println!(
                    "  {:<8} (Col): shipped={:6.2} ms | preshift_run8={:6.2} ({:+5.1}%) | preshift_norun={:6.2} ({:+5.1}%)",
                    name,
                    m0,
                    m1,
                    100.0 * (m1 - m0) / m0,
                    m2,
                    100.0 * (m2 - m0) / m0
                );
            } else {
                println!(
                    "  {:<8} (Run): shipped={:6.2} ms | preshift_len8={:6.2} ({:+5.1}%) | preshift_unr2={:6.2} ({:+5.1}%)",
                    name,
                    m0,
                    m1,
                    100.0 * (m1 - m0) / m0,
                    m2,
                    100.0 * (m2 - m0) / m0
                );
            }
        }

        println!("\n=== STAGE 1B: ISOLATED WALK SCREEN (M3, median of 5 warm CPU ms) ===");
        let walk_names = [
            "L32_pfL1_clr(ship)",
            "L32_pfL1_noclr",
            "L40_pfL1_noclr",
            "L48_pfL1_noclr",
            "L64_pfL1_noclr",
            "L32_SEG256_noclr",
            "L32_cmov_T256",
            "L32_cmov_T128",
            "L48_cmov_T128",
            "L32_trans_pfL1",
            "L32_gather_avx2",
        ];
        for (name, blks) in &seg_corpora {
            if !matches!(name.as_str(), "text" | "random" | "records") {
                continue;
            }
            let mut tvs: Vec<Vec<u32>> = Vec::with_capacity(blks.len());
            for b in blks {
                let mut tt = Vec::new();
                if b.route == ProdRoute::Column {
                    assert!(build_lf_shipped(&b.ll, &b.counts, &mut tt));
                } else {
                    fill_shipped_raw(&b.runs_vec, &b.counts, b.n, &mut tt);
                }
                tvs.push(tt);
            }
            let mut times = vec![Vec::new(); walk_names.len()];
            let mut ws = WalkScratch::default();
            let mut out = Vec::new();
            let mut tmp = Vec::new();
            for (wi, wname) in walk_names.iter().enumerate() {
                for (bi, b) in blks.iter().enumerate() {
                    let mut tt = tvs[bi].clone();
                    match wi {
                        0 => walk_room_std::<32, PF_L1, true>(
                            &mut tt,
                            b.orig_ptr,
                            &mut ws,
                            FLAGGED_SEGMENTS,
                        ),
                        1 => walk_room_std::<32, PF_L1, false>(
                            &mut tt,
                            b.orig_ptr,
                            &mut ws,
                            FLAGGED_SEGMENTS,
                        ),
                        2 => walk_room_std::<40, PF_L1, false>(
                            &mut tt,
                            b.orig_ptr,
                            &mut ws,
                            FLAGGED_SEGMENTS,
                        ),
                        3 => walk_room_std::<48, PF_L1, false>(
                            &mut tt,
                            b.orig_ptr,
                            &mut ws,
                            FLAGGED_SEGMENTS,
                        ),
                        4 => walk_room_std::<64, PF_L1, false>(
                            &mut tt,
                            b.orig_ptr,
                            &mut ws,
                            FLAGGED_SEGMENTS,
                        ),
                        5 => walk_room_std::<32, PF_L1, false>(&mut tt, b.orig_ptr, &mut ws, 256),
                        6 => walk_room_cmov::<32, PF_L1, false, 256>(
                            &mut tt,
                            b.orig_ptr,
                            &mut ws,
                            FLAGGED_SEGMENTS,
                        ),
                        7 => walk_room_cmov::<32, PF_L1, false, 128>(
                            &mut tt,
                            b.orig_ptr,
                            &mut ws,
                            FLAGGED_SEGMENTS,
                        ),
                        8 => walk_room_cmov::<48, PF_L1, false, 128>(
                            &mut tt,
                            b.orig_ptr,
                            &mut ws,
                            FLAGGED_SEGMENTS,
                        ),
                        9 => walk_room_transposed_32::<PF_L1, false>(
                            &mut tt,
                            b.orig_ptr,
                            &mut ws,
                            FLAGGED_SEGMENTS,
                        ),
                        10 => walk_room_gather32(&mut tt, b.orig_ptr, &mut ws, FLAGGED_SEGMENTS),
                        _ => unreachable!(),
                    }
                    assemble_reversed_walk_scratch(&ws, &mut tmp, k);
                    out.clear();
                    rle::unrle(&tmp, &mut out, k).unwrap();
                    assert_eq!(out, b.want_final, "walk {wname} mismatch on {name}");
                }
            }
            let mut work_tvs = tvs.clone();
            for r in 0..5 {
                for step in 0..walk_names.len() {
                    let wi = (r + step) % walk_names.len();
                    for (bi, tv) in work_tvs.iter_mut().enumerate() {
                        tv.copy_from_slice(&tvs[bi]);
                    }
                    let s0 = thread_cpu_ns();
                    for (bi, b) in blks.iter().enumerate() {
                        let tt = &mut work_tvs[bi];
                        match wi {
                            0 => walk_room_std::<32, PF_L1, true>(
                                tt,
                                b.orig_ptr,
                                &mut ws,
                                FLAGGED_SEGMENTS,
                            ),
                            1 => walk_room_std::<32, PF_L1, false>(
                                tt,
                                b.orig_ptr,
                                &mut ws,
                                FLAGGED_SEGMENTS,
                            ),
                            2 => walk_room_std::<40, PF_L1, false>(
                                tt,
                                b.orig_ptr,
                                &mut ws,
                                FLAGGED_SEGMENTS,
                            ),
                            3 => walk_room_std::<48, PF_L1, false>(
                                tt,
                                b.orig_ptr,
                                &mut ws,
                                FLAGGED_SEGMENTS,
                            ),
                            4 => walk_room_std::<64, PF_L1, false>(
                                tt,
                                b.orig_ptr,
                                &mut ws,
                                FLAGGED_SEGMENTS,
                            ),
                            5 => walk_room_std::<32, PF_L1, false>(tt, b.orig_ptr, &mut ws, 256),
                            6 => walk_room_cmov::<32, PF_L1, false, 256>(
                                tt,
                                b.orig_ptr,
                                &mut ws,
                                FLAGGED_SEGMENTS,
                            ),
                            7 => walk_room_cmov::<32, PF_L1, false, 128>(
                                tt,
                                b.orig_ptr,
                                &mut ws,
                                FLAGGED_SEGMENTS,
                            ),
                            8 => walk_room_cmov::<48, PF_L1, false, 128>(
                                tt,
                                b.orig_ptr,
                                &mut ws,
                                FLAGGED_SEGMENTS,
                            ),
                            9 => walk_room_transposed_32::<PF_L1, false>(
                                tt,
                                b.orig_ptr,
                                &mut ws,
                                FLAGGED_SEGMENTS,
                            ),
                            10 => walk_room_gather32(tt, b.orig_ptr, &mut ws, FLAGGED_SEGMENTS),
                            _ => unreachable!(),
                        }
                    }
                    times[wi].push((thread_cpu_ns() - s0) as f64 / 1e6);
                }
            }
            let base = median(times[0].clone());
            println!("  corpus={name} (base L32_pfL1_clr = {base:.2} ms):");
            for (wi, wname) in walk_names.iter().enumerate() {
                let m = median(times[wi].clone());
                println!(
                    "    {:<20} {:6.2} ms ({:+6.2}%)",
                    wname,
                    m,
                    100.0 * (m - base) / base
                );
            }
        }

        // Stage 1C: Isolated Post-Walk Screen (`asm + rle + crc` vs `M1_buf` vs `M1_reg`)!
        println!("\n=== STAGE 1C: ISOLATED POST-WALK SCREEN (M1, median of 5 warm CPU ms) ===");
        for (name, blks) in &seg_corpora {
            if !matches!(
                name.as_str(),
                "text" | "random" | "records" | "pattern" | "runs" | "onebyte"
            ) {
                continue;
            }
            // Pre-walk every block into its own WalkScratch
            let mut walked: Vec<WalkScratch> = Vec::with_capacity(blks.len());
            let mut tt = Vec::new();
            for b in blks {
                if b.route == ProdRoute::Column {
                    assert!(build_lf_shipped(&b.ll, &b.counts, &mut tt));
                } else {
                    fill_shipped_raw(&b.runs_vec, &b.counts, b.n, &mut tt);
                }
                let mut ws = WalkScratch::default();
                walk_room_std::<32, PF_L1, false>(&mut tt, b.orig_ptr, &mut ws, FLAGGED_SEGMENTS);
                walked.push(ws);
            }
            let mut t_ship = Vec::new();
            let mut t_buf = Vec::new();
            let mut t_reg32 = Vec::new();
            let mut t_reg64 = Vec::new();
            let mut stage = Vec::new();
            let mut out = Vec::new();
            let mut rruns = Vec::new();
            for _ in 0..5 {
                let s0 = thread_cpu_ns();
                for (bi, b) in blks.iter().enumerate() {
                    if b.route == ProdRoute::Column {
                        assemble_reversed_walk_scratch(&walked[bi], &mut stage, k);
                        out.clear();
                        rle::unrle(&stage, &mut out, k).unwrap();
                        assert_eq!(crc::crc32(&out, k), b.crc_expected);
                    } else {
                        assemble_reversed_walk_scratch(&walked[bi], &mut out, k);
                        let c =
                            shipped_unrle_in_place(&mut out, &mut rruns, &mut stage, k).unwrap();
                        assert_eq!(c, b.crc_expected);
                    }
                }
                t_ship.push((thread_cpu_ns() - s0) as f64 / 1e6);

                let s1 = thread_cpu_ns();
                for (bi, b) in blks.iter().enumerate() {
                    let c = fused_rev_rle_crc::<4096, 32768>(&mut walked[bi], &mut out, k).unwrap();
                    assert_eq!(c, b.crc_expected);
                }
                t_buf.push((thread_cpu_ns() - s1) as f64 / 1e6);

                let s2 = thread_cpu_ns();
                for (bi, b) in blks.iter().enumerate() {
                    let c = fused_rev_rle_crc_reg::<32768>(&mut walked[bi], &mut out, k).unwrap();
                    assert_eq!(c, b.crc_expected);
                }
                t_reg32.push((thread_cpu_ns() - s2) as f64 / 1e6);

                let s3 = thread_cpu_ns();
                for (bi, b) in blks.iter().enumerate() {
                    let c = fused_rev_rle_crc_reg::<65536>(&mut walked[bi], &mut out, k).unwrap();
                    assert_eq!(c, b.crc_expected);
                }
                t_reg64.push((thread_cpu_ns() - s3) as f64 / 1e6);
            }
            let ms = median(t_ship);
            let mb = median(t_buf);
            let mr32 = median(t_reg32);
            let mr64 = median(t_reg64);
            println!(
                "  {:<8}: ship={:6.2} ms | buf4k={:6.2} ({:+5.1}%) | reg32k={:6.2} ({:+5.1}%) | reg64k={:6.2} ({:+5.1}%)",
                name,
                ms,
                mb,
                100.0 * (mb - ms) / ms,
                mr32,
                100.0 * (mr32 - ms) / ms,
                mr64,
                100.0 * (mr64 - ms) / ms
            );
        }

        // =====================================================================
        // STAGE 2: Full-Stack 7-Repeat Interleaved Benchmark (Warm & L3-Cold)
        // =====================================================================
        let arms = [
            FullArm::Shipped,
            FullArm::ShippedLocal,
            FullArm::M1FusedBuf4k32k,
            FullArm::M1FusedReg32k,
            FullArm::M1FusedReg64k,
            FullArm::M2BuildOnly,
            FullArm::M3WalkL48,
            FullArm::M3WalkCmov32,
            FullArm::M4BestL32,
            FullArm::M4BestL48,
        ];

        // Exactness verification across ALL 7 corpora & ALL blocks BEFORE timing!
        println!("\n=== STAGE 2A: 100% BIT-EXACT + CRC-EXACT VERIFICATION (ALL 7 CORPORA) ===");
        let mut checks = 0usize;
        for &arm in &arms {
            let mut sc = ArmScratch::default();
            for (_, blks) in &seg_corpora {
                for b in blks {
                    run_block_arm(arm, b, &mut sc, k, true);
                    checks += 1;
                }
            }
        }
        println!("Verified {checks}/{checks} block decodes bit-exact & CRC-exact!");

        // Peak scratch + output memory audit on `text`, `random`, and `onebyte`.
        println!("\n=== STAGE 2B: PEAK BUFFER FOOTPRINT (text, random, onebyte) ===");
        for target_corpus in ["text", "random", "onebyte"] {
            if let Some((_, blks)) = seg_corpora.iter().find(|(n, _)| n == target_corpus) {
                for &arm in &[
                    FullArm::ShippedLocal,
                    FullArm::M1FusedBuf4k32k,
                    FullArm::M1FusedReg64k,
                    FullArm::M4BestL32,
                ] {
                    let mut sc = ArmScratch::default();
                    sc.reset_allocs();
                    for b in blks {
                        run_block_arm(arm, b, &mut sc, k, false);
                    }
                    let bytes = sc.custom_heap_bytes();
                    println!(
                        "  {:<8} {:<18} heap_cap = {:6.2} MiB",
                        target_corpus,
                        arm.name(),
                        bytes as f64 / (1024.0 * 1024.0)
                    );
                }
            }
        }

        // Timed 7-repeat interleaved benchmark (warm AND L3-cold).
        let mut l3_scrub = vec![0u8; 64 << 20];
        const REPS: usize = 7;

        for cold in [false, true] {
            let mode = if cold { "L3-COLD" } else { "WARM" };
            println!(
                "\n=== STAGE 2C: FULL SEGMENTED STACK ({mode}, {REPS} interleaved repeats) ==="
            );
            let mut scs: Vec<ArmScratch> = (0..arms.len()).map(|_| ArmScratch::default()).collect();
            // Warm up allocator capacities once per arm.
            for (ai, &arm) in arms.iter().enumerate() {
                for (_, blks) in &seg_corpora {
                    for b in blks {
                        run_block_arm(arm, b, &mut scs[ai], k, false);
                    }
                }
            }

            for (cname, blks) in &seg_corpora {
                if blks.is_empty() {
                    continue;
                }
                let mut cpu_ms = vec![vec![0.0f64; REPS]; arms.len()];
                let mut wall_ms = vec![vec![0.0f64; REPS]; arms.len()];
                for rep in 0..REPS {
                    for step in 0..arms.len() {
                        let ai = (rep + step) % arms.len();
                        let arm = arms[ai];
                        if cold {
                            flush_l3(&mut l3_scrub);
                        }
                        let w0 = Instant::now();
                        let c0 = thread_cpu_ns();
                        let mut acc = 0u32;
                        for b in blks {
                            acc ^= run_block_arm(arm, b, &mut scs[ai], k, false);
                        }
                        let c1 = thread_cpu_ns();
                        let w1 = w0.elapsed();
                        std::hint::black_box(acc);
                        cpu_ms[ai][rep] = (c1 - c0) as f64 / 1e6;
                        wall_ms[ai][rep] = w1.as_secs_f64() * 1e3;
                    }
                }
                let (base_med, base_min, base_max) = stats(&cpu_ms[0]);
                let (base_wmed, _, _) = stats(&wall_ms[0]);
                println!(
                    "\n  corpus={:<10} ({} blks) | shipped CPU={:.2} [{:.2}..{:.2}] ms | Wall={:.2} ms",
                    cname,
                    blks.len(),
                    base_med,
                    base_min,
                    base_max,
                    base_wmed
                );
                for (ai, &arm) in arms.iter().enumerate().skip(1) {
                    let (med, mn, mx) = stats(&cpu_ms[ai]);
                    let (wmed, _, _) = stats(&wall_ms[ai]);
                    let d_cpu = 100.0 * (med - base_med) / base_med.max(1e-9);
                    let d_wall = 100.0 * (wmed - base_wmed) / base_wmed.max(1e-9);
                    let disj = if mx < base_min {
                        "DISJ_FASTER"
                    } else if mn > base_max {
                        "DISJ_SLOWER"
                    } else {
                        "overlap"
                    };
                    println!(
                        "    {:<18} CPU={:6.2} [{:6.2}..{:6.2}] ({:+6.2}%) {:<11} | Wall={:6.2} ({:+6.2}%)",
                        arm.name(),
                        med,
                        mn,
                        mx,
                        d_cpu,
                        disj,
                        wmed,
                        d_wall
                    );
                }
            }
        }
    }
}

#[cfg(feature = "moonshot")]
fn main() {
    imp::main();
}

#[cfg(not(feature = "moonshot"))]
fn main() {
    eprintln!("bwt_segmented_dissection: build with --features moonshot");
}
