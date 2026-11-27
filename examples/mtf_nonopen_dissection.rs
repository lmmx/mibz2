//! Smooth-zone dissection & redesign probe for non-Open `src/mtf.rs`
//! (`mtf_avx2` and `mtf_checkpoints_avx2`, zero production `src/` changes).
//!
//! Usage:
//!   cargo run --release --example mtf_nonopen_dissection -- exact       /tmp/mtf-nonopen-corpora
//!   cargo run --release --example mtf_nonopen_dissection -- bench       /tmp/mtf-nonopen-corpora
//!   cargo run --release --example mtf_nonopen_dissection -- bench_final /tmp/mtf-nonopen-corpora

use mibz2::block::Symbols;
use mibz2::mtf::{self, CHECKPOINT};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{ErrorKind, Kernels};
use std::path::{Path, PathBuf};

const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;

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

struct LoadedBlock {
    syms: Symbols,
    want_ll: Vec<u8>,
    want_counts: [u32; 256],
    want_cps: Vec<[u32; 256]>,
    want_n: usize,
}

fn load_blocks(path: &Path) -> Vec<LoadedBlock> {
    let data = std::fs::read(path).expect("read bz2");
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut blocks = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut cps = Vec::new();
    while let Event::Block { symbols, .. } = scanner.next(Vec::new()).expect("scan") {
        let n = mtf::mtf_checkpoints_scalar(&symbols, &mut ll, &mut counts, &mut cps)
            .expect("mtf_checkpoints_scalar");
        blocks.push(LoadedBlock {
            syms: symbols,
            want_ll: ll.clone(),
            want_counts: counts,
            want_cps: cps.clone(),
            want_n: n,
        });
    }
    blocks
}

#[cfg(target_arch = "x86_64")]
mod kernels {
    use super::{CHECKPOINT, MAX_RUN_WEIGHT};
    use mibz2::block::Symbols;
    use mibz2::ErrorKind;
    use std::arch::x86_64::*;
    use std::mem::MaybeUninit;

    #[repr(C, align(64))]
    #[derive(Clone, Copy)]
    struct Step32 {
        a: [u8; 16],
        b: [u8; 16],
        m: [u8; 16],
        _pad: [u8; 16],
    }

    const fn make_steps32() -> [Step32; 32] {
        const Z: u8 = 0x80;
        let mut t = [Step32 {
            a: [0; 16],
            b: [Z; 16],
            m: [0; 16],
            _pad: [0; 16],
        }; 32];
        let mut nn = 0;
        while nn < 32 {
            let mut i = 0;
            while i < 16 {
                if nn < 16 {
                    t[nn].a[i] = if i == 0 {
                        nn as u8
                    } else if i <= nn {
                        (i - 1) as u8
                    } else {
                        i as u8
                    };
                } else {
                    t[nn].a[i] = if i == 0 { Z } else { (i - 1) as u8 };
                    if i == 0 {
                        t[nn].b[i] = (nn - 16) as u8;
                    }
                    if i <= nn - 16 {
                        t[nn].m[i] = 0xff;
                    }
                }
                i += 1;
            }
            nn += 1;
        }
        t
    }

    static STEPS32: [Step32; 32] = make_steps32();

    #[repr(C, align(64))]
    struct IdxSigned([[i8; 32]; 8]);

    const fn make_idx_signed() -> IdxSigned {
        let mut a = [[0i8; 32]; 8];
        let mut j = 0;
        while j < 8 {
            let mut i = 0;
            while i < 32 {
                a[j][i] = (((32 * j + i) as u8) ^ 0x80) as i8;
                i += 1;
            }
            j += 1;
        }
        IdxSigned(a)
    }

    static IDX_SIGNED: IdxSigned = make_idx_signed();

    /// 512-byte aligned backing buffers so `base.sub(1)` / `tail.sub(1)` unaligned loads
    /// never straddle a 4 KiB page boundary (eliminates 4K-split penalties on `random`/`records`).
    #[repr(align(512))]
    struct Base320([MaybeUninit<u8>; 320]);

    #[repr(align(512))]
    struct Tail256([MaybeUninit<u8>; 256]);

    #[cold]
    #[inline(never)]
    unsafe fn write_bytes_cold(dst: *mut u8, val: u8, count: usize) {
        std::ptr::write_bytes(dst, val, count);
    }

    #[cold]
    #[inline(never)]
    fn push_checkpoint_cold(checkpoints: &mut Vec<[u32; 256]>, counts: &[u32; 256]) {
        checkpoints.push(*counts);
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    unsafe fn flush_crossing_run_cold(
        out: *mut u8,
        n: &mut usize,
        run: u32,
        c: u8,
        max: usize,
        next: &mut usize,
        counts: &mut [u32; 256],
        checkpoints: &mut Vec<[u32; 256]>,
    ) -> Result<(), ErrorKind> {
        let mut left = run as usize;
        if *n + left > max {
            return Err(ErrorKind::BlockTooLong);
        }
        while *n + left >= *next {
            let part = *next - *n;
            counts[c as usize] += part as u32;
            std::ptr::write_bytes(out.add(*n), c, part);
            *n = *next;
            left -= part;
            checkpoints.push(*counts);
            *next += CHECKPOINT;
        }
        if left != 0 {
            counts[c as usize] += left as u32;
            std::ptr::write_bytes(out.add(*n), c, left);
            *n += left;
        }
        Ok(())
    }

    #[inline(always)]
    unsafe fn step32_xmm(f0: &mut __m128i, f1: &mut __m128i, nn: usize) {
        let t = STEPS32.as_ptr().add(nn);
        let a = _mm_load_si128((*t).a.as_ptr() as *const __m128i);
        let bb = _mm_load_si128((*t).b.as_ptr() as *const __m128i);
        let m = _mm_load_si128((*t).m.as_ptr() as *const __m128i);
        let g0 = _mm_or_si128(_mm_shuffle_epi8(*f0, a), _mm_shuffle_epi8(*f1, bb));
        *f1 = _mm_blendv_epi8(*f1, _mm_alignr_epi8::<15>(*f1, *f0), m);
        *f0 = g0;
    }

    /// Flat unaligned 7-row tail shift (`t` in `0..224`), zero `vperm2i128`/`vpalignr`/`memmove`.
    #[inline(always)]
    unsafe fn tail_flat_unal7_inline(
        tail: *mut u8,
        t: usize,
        f1_last: u8,
        clear_b0: __m256i,
    ) -> u8 {
        let v = *tail.add(t);
        let it = IDX_SIGNED.0.as_ptr();
        let tv = _mm256_set1_epi8(((t as u8) ^ 0x80) as i8);
        for j in (1..7).rev() {
            let row = tail.add(32 * j);
            let keep = _mm256_load_si256(row as *const __m256i);
            let on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
            let gt = _mm256_cmpgt_epi8(_mm256_load_si256(it.add(j) as *const __m256i), tv);
            _mm256_store_si256(row as *mut __m256i, _mm256_blendv_epi8(on, keep, gt));
        }
        let keep0 = _mm256_load_si256(tail as *const __m256i);
        let on0 = _mm256_loadu_si256(tail.sub(1) as *const __m256i);
        let gt0 = _mm256_cmpgt_epi8(_mm256_load_si256(it as *const __m256i), tv);
        let new0 = _mm256_blendv_epi8(on0, keep0, gt0);
        let new0 = _mm256_or_si256(
            _mm256_and_si256(new0, clear_b0),
            _mm256_zextsi128_si256(_mm_cvtsi32_si128(f1_last as i32)),
        );
        _mm256_store_si256(tail as *mut __m256i, new0);
        v
    }

    #[cold]
    #[inline(never)]
    #[target_feature(enable = "avx2")]
    unsafe fn tail_flat_unal7_cold(tail: *mut u8, t: usize, f1_last: u8) -> u8 {
        let clear_b0 = _mm256_setr_epi8(
            0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        );
        tail_flat_unal7_inline(tail, t, f1_last, clear_b0)
    }

    #[inline(always)]
    unsafe fn fill_run_xmm64(out: *mut u8, n: usize, r: usize, max: usize, c: u8, f0: __m128i) {
        if r <= 32 && n + 32 <= max {
            let v = _mm_shuffle_epi8(f0, _mm_setzero_si128());
            _mm_storeu_si128(out.add(n) as *mut __m128i, v);
            _mm_storeu_si128(out.add(n + 16) as *mut __m128i, v);
        } else if r <= 64 && n + 64 <= max {
            let v = _mm_shuffle_epi8(f0, _mm_setzero_si128());
            _mm_storeu_si128(out.add(n) as *mut __m128i, v);
            _mm_storeu_si128(out.add(n + 16) as *mut __m128i, v);
            _mm_storeu_si128(out.add(n + 32) as *mut __m128i, v);
            _mm_storeu_si128(out.add(n + 48) as *mut __m128i, v);
        } else {
            write_bytes_cold(out.add(n), c, r);
        }
    }

    /// Compact inline AVX2 run fill with ZERO `memset` call and ZERO `vzeroupper`.
    #[inline(always)]
    unsafe fn fill_run_inline_nocall(out: *mut u8, n: usize, r: usize, max: usize, v: __m256i) {
        if r <= 64 && n + 64 <= max {
            _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
            _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
            return;
        }
        let mut p = out.add(n);
        let end = p.add(r);
        if r >= 32 {
            let mut rem = r;
            while rem >= 32 {
                _mm256_storeu_si256(p as *mut __m256i, v);
                p = p.add(32);
                rem -= 32;
            }
            _mm256_storeu_si256(end.sub(32) as *mut __m256i, v);
        } else {
            let v128 = _mm256_castsi256_si128(v);
            let w64 = _mm_cvtsi128_si64(v128) as u64;
            if r >= 16 {
                _mm_storeu_si128(p as *mut __m128i, v128);
                _mm_storeu_si128(end.sub(16) as *mut __m128i, v128);
            } else if r >= 8 {
                (p as *mut u64).write_unaligned(w64);
                (end.sub(8) as *mut u64).write_unaligned(w64);
            } else if r >= 4 {
                (p as *mut u32).write_unaligned(w64 as u32);
                (end.sub(4) as *mut u32).write_unaligned(w64 as u32);
            } else if r >= 2 {
                (p as *mut u16).write_unaligned(w64 as u16);
                (end.sub(2) as *mut u16).write_unaligned(w64 as u16);
            } else if r == 1 {
                *p = w64 as u8;
            }
        }
    }

    /// High-bandwidth 32B-aligned 4x32B (128B/iter) AVX2 run fill for long runs (plain `mtf`).
    #[inline(always)]
    unsafe fn fill_run_long_nocall(out: *mut u8, n: usize, r: usize, v: __m256i) {
        let mut p = out.add(n);
        let end = p.add(r);
        if r >= 128 {
            _mm256_storeu_si256(p as *mut __m256i, v);
            let align_off = 32 - ((p as usize) & 31);
            p = p.add(align_off);
            let mut rem = r - align_off;
            while rem >= 128 {
                _mm256_store_si256(p as *mut __m256i, v);
                _mm256_store_si256(p.add(32) as *mut __m256i, v);
                _mm256_store_si256(p.add(64) as *mut __m256i, v);
                _mm256_store_si256(p.add(96) as *mut __m256i, v);
                p = p.add(128);
                rem -= 128;
                std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::AcqRel);
            }
            if rem >= 64 {
                _mm256_store_si256(p as *mut __m256i, v);
                _mm256_store_si256(p.add(32) as *mut __m256i, v);
                p = p.add(64);
                rem -= 64;
            }
            if rem >= 32 {
                _mm256_store_si256(p as *mut __m256i, v);
            }
            _mm256_storeu_si256(end.sub(32) as *mut __m256i, v);
        } else {
            fill_run_inline_nocall(out, n, r, n + r, v);
        }
    }

    /// Compact 1024-byte AVX2 checkpoint copy (never unrolled, never turned into `call memcpy`).
    #[inline(always)]
    unsafe fn copy_ckpt_compact_nocall(dst: *mut [u32; 256], src: *const [u32; 256]) {
        let mut d = dst.cast::<u8>();
        let mut s = src.cast::<u8>();
        let end = d.add(1024);
        while d < end {
            let v0 = _mm256_loadu_si256(s as *const __m256i);
            let v1 = _mm256_loadu_si256(s.add(32) as *const __m256i);
            let v2 = _mm256_loadu_si256(s.add(64) as *const __m256i);
            let v3 = _mm256_loadu_si256(s.add(96) as *const __m256i);
            _mm256_storeu_si256(d as *mut __m256i, v0);
            _mm256_storeu_si256(d.add(32) as *mut __m256i, v1);
            _mm256_storeu_si256(d.add(64) as *mut __m256i, v2);
            _mm256_storeu_si256(d.add(96) as *mut __m256i, v3);
            d = std::hint::black_box(d.add(128));
            s = s.add(128);
        }
    }

    /// O(1) 128-symbol high-entropy detector (>50% literals with `nn >= 32`: `random`, `records`).
    #[inline(always)]
    fn is_high_entropy(b: &Symbols) -> bool {
        let syms = &b.syms;
        let len = syms.len();
        if len < 512 {
            return false;
        }
        let stride = (len >> 7).max(1);
        let mut high32 = 0u32;
        let mut lits = 0u32;
        let mut i = 0usize;
        let mut samples = 0u32;
        while i < len && samples < 128 {
            let s = syms[i];
            if s >= 2 && s != b.eob {
                lits += 1;
                if s >= 33 {
                    high32 += 1;
                }
            }
            i += stride;
            samples += 1;
        }
        high32 * 2 > lits
    }

    /// Arm A: PagedList with pre-applied `s2u`, uninitialized 320B stack backing + direct `cp[0]`
    /// zeroing, inline 64B YMM splat + cold `memset`, zero-STLF register-shifted front for `nn < 32`,
    /// flat unaligned 8-row tail for `nn >= 32`, and cold checkpoint helpers.
    #[inline(never)]
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_a_ck_paged_nos2u_splat(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
        checkpoints: &mut Vec<[u32; 256]>,
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        checkpoints.clear();
        checkpoints.reserve(1);
        std::ptr::write_bytes(checkpoints.as_mut_ptr(), 0, 1);
        checkpoints.set_len(1);

        let mut store = Base320([MaybeUninit::uninit(); 320]);
        let base = store.0.as_mut_ptr().cast::<u8>().add(32);
        *base.sub(1) = 0;
        std::ptr::copy_nonoverlapping(b.seq_to_unseq.as_ptr(), base, 256);
        let it = IDX_SIGNED.0.as_ptr();
        let idx0 = _mm256_load_si256(it as *const __m256i);
        let clear_b0 = _mm256_setr_epi8(
            0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        );
        let out = ll.as_mut_ptr();
        let eob = b.eob;
        let (mut run, mut weight) = (0u32, 1u32);
        let mut n = 0usize;
        let mut next = CHECKPOINT;
        let mut result = Ok(());
        let mut syms = b.syms.iter();

        macro_rules! step_byte {
            ($s:expr) => {{
                let nn = (($s - 1) as usize).min(255);
                let c = *base.add(nn);
                let tv = _mm256_set1_epi8(((nn as u8) ^ 0x80) as i8);
                if nn < 32 {
                    let keep0 = _mm256_load_si256(base as *const __m256i);
                    let lo_in_hi = _mm256_permute2x128_si256::<0x08>(keep0, keep0);
                    let shifted = _mm256_alignr_epi8::<15>(keep0, lo_in_hi);
                    let on0 = _mm256_or_si256(
                        shifted,
                        _mm256_zextsi128_si256(_mm_cvtsi32_si128(c as i32)),
                    );
                    let gt0 = _mm256_cmpgt_epi8(idx0, tv);
                    _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(on0, keep0, gt0));
                } else {
                    for j in (1..8).rev() {
                        let row = base.add(32 * j);
                        let keep = _mm256_load_si256(row as *const __m256i);
                        let on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
                        let gt =
                            _mm256_cmpgt_epi8(_mm256_load_si256(it.add(j) as *const __m256i), tv);
                        _mm256_store_si256(row as *mut __m256i, _mm256_blendv_epi8(on, keep, gt));
                    }
                    let keep0 = _mm256_load_si256(base as *const __m256i);
                    let on0 = _mm256_loadu_si256(base.sub(1) as *const __m256i);
                    let gt0 = _mm256_cmpgt_epi8(idx0, tv);
                    let new0 = _mm256_blendv_epi8(on0, keep0, gt0);
                    let new0 = _mm256_or_si256(
                        _mm256_and_si256(new0, clear_b0),
                        _mm256_zextsi128_si256(_mm_cvtsi32_si128(c as i32)),
                    );
                    _mm256_store_si256(base as *mut __m256i, new0);
                }
                *counts.get_unchecked_mut(c as usize) += 1;
                *out.add(n) = c;
                n += 1;
            }};
        }

        'block: loop {
            let limit = max.min(next);
            let mut at = None;
            for &s in syms.by_ref() {
                if s <= 1 {
                    if weight >= MAX_RUN_WEIGHT {
                        result = Err(ErrorKind::RunTooLong);
                        break 'block;
                    }
                    run += (s as u32 + 1) * weight;
                    weight <<= 1;
                    continue;
                }
                if run != 0 {
                    let r = run as usize;
                    if n + r > limit {
                        at = Some(s);
                        break;
                    }
                    let c = *base;
                    *counts.get_unchecked_mut(c as usize) += run;
                    if r <= 64 && n + 64 <= max {
                        let v = _mm256_set1_epi8(c as i8);
                        _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                        _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
                    } else {
                        write_bytes_cold(out.add(n), c, r);
                    }
                    n += r;
                    (run, weight) = (0, 1);
                }
                if s == eob {
                    break 'block;
                }
                if n >= limit {
                    at = Some(s);
                    break;
                }
                step_byte!(s);
            }
            let Some(s) = at else { break 'block };
            if run != 0 {
                let c = *base;
                if let Err(e) = flush_crossing_run_cold(
                    out,
                    &mut n,
                    run,
                    c,
                    max,
                    &mut next,
                    counts,
                    checkpoints,
                ) {
                    result = Err(e);
                    break 'block;
                }
                (run, weight) = (0, 1);
                if s == eob {
                    break 'block;
                }
                if n >= max {
                    result = Err(ErrorKind::BlockTooLong);
                    break 'block;
                }
            } else {
                if n >= max {
                    result = Err(ErrorKind::BlockTooLong);
                    break 'block;
                }
                push_checkpoint_cold(checkpoints, counts);
                next += CHECKPOINT;
            }
            step_byte!(s);
        }
        if result.is_ok() && n == next {
            push_checkpoint_cold(checkpoints, counts);
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    /// Arm C (diagnostic): 100% XMM hot loop with `#[cold] #[inline(never)]` tail call —
    /// demonstrates System-V ABI XMM spills on `-C target-cpu=native` when a cold call lives
    /// inside the inner loop.
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_c_ck_reg_cold_tail(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
        checkpoints: &mut Vec<[u32; 256]>,
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        checkpoints.clear();
        checkpoints.push([0; 256]);
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        let mut store = Tail256([MaybeUninit::uninit(); 256]);
        let tail = store.0.as_mut_ptr().cast::<u8>().add(32);
        *tail.sub(1) = 0;
        std::ptr::copy_nonoverlapping(s2u.as_ptr().add(32), tail, 224);
        let out = ll.as_mut_ptr();
        let eob = b.eob;
        let (mut run, mut weight) = (0u32, 1u32);
        let mut n = 0usize;
        let mut next = CHECKPOINT;
        let mut result = Ok(());
        let mut syms = b.syms.iter();

        macro_rules! step_byte {
            ($s:expr) => {{
                let nn = (($s - 1) as usize).min(255);
                if nn < 32 {
                    step32_xmm(&mut f0, &mut f1, nn);
                } else {
                    let f1_last = _mm_extract_epi8::<15>(f1) as u8;
                    let v = tail_flat_unal7_cold(tail, nn - 32, f1_last);
                    f1 = _mm_alignr_epi8::<15>(f1, f0);
                    f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
                }
                let c = _mm_cvtsi128_si32(f0) as u8;
                *counts.get_unchecked_mut(c as usize) += 1;
                *out.add(n) = c;
                n += 1;
            }};
        }

        'block: loop {
            let limit = max.min(next);
            let mut at = None;
            for &s in syms.by_ref() {
                if s <= 1 {
                    if weight >= MAX_RUN_WEIGHT {
                        result = Err(ErrorKind::RunTooLong);
                        break 'block;
                    }
                    run += (s as u32 + 1) * weight;
                    weight <<= 1;
                    continue;
                }
                if run != 0 {
                    let r = run as usize;
                    if n + r > limit {
                        at = Some(s);
                        break;
                    }
                    let c = _mm_cvtsi128_si32(f0) as u8;
                    *counts.get_unchecked_mut(c as usize) += run;
                    fill_run_xmm64(out, n, r, max, c, f0);
                    n += r;
                    (run, weight) = (0, 1);
                }
                if s == eob {
                    break 'block;
                }
                if n >= limit {
                    at = Some(s);
                    break;
                }
                step_byte!(s);
            }
            let Some(s) = at else { break 'block };
            if run != 0 {
                let c = _mm_cvtsi128_si32(f0) as u8;
                if let Err(e) = flush_crossing_run_cold(
                    out,
                    &mut n,
                    run,
                    c,
                    max,
                    &mut next,
                    counts,
                    checkpoints,
                ) {
                    result = Err(e);
                    break 'block;
                }
                (run, weight) = (0, 1);
                if s == eob {
                    break 'block;
                }
                if n >= max {
                    result = Err(ErrorKind::BlockTooLong);
                    break 'block;
                }
            } else {
                if n >= max {
                    result = Err(ErrorKind::BlockTooLong);
                    break 'block;
                }
                push_checkpoint_cold(checkpoints, counts);
                next += CHECKPOINT;
            }
            step_byte!(s);
        }
        if result.is_ok() && n == next {
            push_checkpoint_cold(checkpoints, counts);
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    /// Zero-call-in-loop branchless 8-row flat unaligned P2 checkpointed decoder
    /// (for high-entropy Column blocks: `random`, `records`).
    #[inline(never)]
    #[target_feature(enable = "avx2")]
    unsafe fn ck_zerocall_p2_flat8(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
        checkpoints: &mut Vec<[u32; 256]>,
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        let max_ckpts = max / CHECKPOINT + 2;
        checkpoints.clear();
        checkpoints.reserve(max_ckpts);
        *counts = [0; 256];
        let cp_ptr = checkpoints.as_mut_ptr();
        std::ptr::write_bytes(cp_ptr, 0, 1);
        let mut cp_len = 1usize;

        let mut store = Base320([MaybeUninit::uninit(); 320]);
        let base = store.0.as_mut_ptr().cast::<u8>().add(32);
        *base.sub(1) = 0;
        std::ptr::copy_nonoverlapping(b.seq_to_unseq.as_ptr(), base, 256);
        let it = IDX_SIGNED.0.as_ptr();
        let clear_b0 = _mm256_setr_epi8(
            0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        );
        let out = ll.as_mut_ptr();
        let eob = b.eob;
        let (mut run, mut weight) = (0u32, 1u32);
        let mut n = 0usize;
        let mut next = CHECKPOINT;
        let mut result = Ok(());

        for &s in &b.syms {
            if s <= 1 {
                if weight >= MAX_RUN_WEIGHT {
                    result = Err(ErrorKind::RunTooLong);
                    break;
                }
                run += (s as u32 + 1) * weight;
                weight <<= 1;
                continue;
            }
            if run != 0 {
                let r = run as usize;
                if n + r > max {
                    result = Err(ErrorKind::BlockTooLong);
                    break;
                }
                let c = *base;
                let v = _mm256_set1_epi8(c as i8);
                if n + r < next {
                    *counts.get_unchecked_mut(c as usize) += run;
                    fill_run_inline_nocall(out, n, r, max, v);
                    n += r;
                } else {
                    let mut left = r;
                    while n + left >= next {
                        let part = next - n;
                        *counts.get_unchecked_mut(c as usize) += part as u32;
                        fill_run_inline_nocall(out, n, part, max, v);
                        n = next;
                        left -= part;
                        copy_ckpt_compact_nocall(cp_ptr.add(cp_len), counts);
                        cp_len += 1;
                        next += CHECKPOINT;
                    }
                    if left != 0 {
                        *counts.get_unchecked_mut(c as usize) += left as u32;
                        fill_run_inline_nocall(out, n, left, max, v);
                        n += left;
                    }
                }
                (run, weight) = (0, 1);
            }
            if s == eob {
                break;
            }
            if n >= max {
                result = Err(ErrorKind::BlockTooLong);
                break;
            }
            if n == next {
                copy_ckpt_compact_nocall(cp_ptr.add(cp_len), counts);
                cp_len += 1;
                next += CHECKPOINT;
            }
            let nn = ((s - 1) as usize).min(255);
            let c = *base.add(nn);
            let tv = _mm256_set1_epi8(((nn as u8) ^ 0x80) as i8);
            for j in (1..8).rev() {
                let row = base.add(32 * j);
                let keep = _mm256_load_si256(row as *const __m256i);
                let on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
                let gt = _mm256_cmpgt_epi8(_mm256_load_si256(it.add(j) as *const __m256i), tv);
                _mm256_store_si256(row as *mut __m256i, _mm256_blendv_epi8(on, keep, gt));
            }
            let keep0 = _mm256_load_si256(base as *const __m256i);
            let on0 = _mm256_loadu_si256(base.sub(1) as *const __m256i);
            let gt0 = _mm256_cmpgt_epi8(_mm256_load_si256(it as *const __m256i), tv);
            let new0 = _mm256_blendv_epi8(on0, keep0, gt0);
            let new0 = _mm256_or_si256(
                _mm256_and_si256(new0, clear_b0),
                _mm256_zextsi128_si256(_mm_cvtsi32_si128(c as i32)),
            );
            _mm256_store_si256(base as *mut __m256i, new0);
            *counts.get_unchecked_mut(c as usize) += 1;
            *out.add(n) = c;
            n += 1;
        }
        if result.is_ok() && n == next {
            copy_ckpt_compact_nocall(cp_ptr.add(cp_len), counts);
            cp_len += 1;
        }
        checkpoints.set_len(cp_len);
        ll.set_len(n);
        result.map(|()| n)
    }

    /// Zero-call-in-loop single-function XMM `(f0, f1)` + lazy flat-7 tail checkpointed decoder.
    /// ZERO calls inside the symbol loop, ZERO XMM spills (`xmm_rsp = 0`), `vzeroupper = 1`
    /// (epilogue only), pre-reserved `checkpoints` with compact inline AVX2 checkpoint copies.
    #[inline(never)]
    #[target_feature(enable = "avx2")]
    unsafe fn ck_zerocall_reg7(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
        checkpoints: &mut Vec<[u32; 256]>,
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        let max_ckpts = max / CHECKPOINT + 2;
        checkpoints.clear();
        checkpoints.reserve(max_ckpts);
        *counts = [0; 256];
        let cp_ptr = checkpoints.as_mut_ptr();
        std::ptr::write_bytes(cp_ptr, 0, 1);
        let mut cp_len = 1usize;

        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        let mut store = Tail256([MaybeUninit::uninit(); 256]);
        let tail = store.0.as_mut_ptr().cast::<u8>().add(32);
        let mut tail_ready = false;
        let clear_b0 = _mm256_setr_epi8(
            0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        );
        let out = ll.as_mut_ptr();
        let eob = b.eob;
        let (mut run, mut weight) = (0u32, 1u32);
        let mut n = 0usize;
        let mut next = CHECKPOINT;
        let mut result = Ok(());

        for &s in &b.syms {
            if s <= 1 {
                if weight >= MAX_RUN_WEIGHT {
                    result = Err(ErrorKind::RunTooLong);
                    break;
                }
                run += (s as u32 + 1) * weight;
                weight <<= 1;
                continue;
            }
            if run != 0 {
                let r = run as usize;
                if n + r > max {
                    result = Err(ErrorKind::BlockTooLong);
                    break;
                }
                let c = _mm_cvtsi128_si32(f0) as u8;
                let v = _mm256_broadcastb_epi8(f0);
                if n + r < next {
                    *counts.get_unchecked_mut(c as usize) += run;
                    fill_run_inline_nocall(out, n, r, max, v);
                    n += r;
                } else {
                    let mut left = r;
                    while n + left >= next {
                        let part = next - n;
                        *counts.get_unchecked_mut(c as usize) += part as u32;
                        fill_run_inline_nocall(out, n, part, max, v);
                        n = next;
                        left -= part;
                        copy_ckpt_compact_nocall(cp_ptr.add(cp_len), counts);
                        cp_len += 1;
                        next += CHECKPOINT;
                    }
                    if left != 0 {
                        *counts.get_unchecked_mut(c as usize) += left as u32;
                        fill_run_inline_nocall(out, n, left, max, v);
                        n += left;
                    }
                }
                (run, weight) = (0, 1);
            }
            if s == eob {
                break;
            }
            if n >= max {
                result = Err(ErrorKind::BlockTooLong);
                break;
            }
            if n == next {
                copy_ckpt_compact_nocall(cp_ptr.add(cp_len), counts);
                cp_len += 1;
                next += CHECKPOINT;
            }
            let nn = ((s - 1) as usize).min(255);
            if nn < 32 {
                step32_xmm(&mut f0, &mut f1, nn);
            } else {
                if !tail_ready {
                    *tail.sub(1) = 0;
                    let src = s2u.as_ptr().add(32);
                    for j in 0..7 {
                        let chunk = _mm256_loadu_si256(src.add(32 * j) as *const __m256i);
                        _mm256_store_si256(tail.add(32 * j) as *mut __m256i, chunk);
                    }
                    tail_ready = true;
                }
                let f1_last = _mm_extract_epi8::<15>(f1) as u8;
                let v = tail_flat_unal7_inline(tail, nn - 32, f1_last, clear_b0);
                f1 = _mm_alignr_epi8::<15>(f1, f0);
                f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
            }
            let c = _mm_cvtsi128_si32(f0) as u8;
            *counts.get_unchecked_mut(c as usize) += 1;
            *out.add(n) = c;
            n += 1;
        }
        if result.is_ok() && n == next {
            copy_ckpt_compact_nocall(cp_ptr.add(cp_len), counts);
            cp_len += 1;
        }
        checkpoints.set_len(cp_len);
        ll.set_len(n);
        result.map(|()| n)
    }

    /// Arm T (final candidate): 3-way O(1) adaptive zero-call checkpointed decoder:
    /// - Small/run-dominated blocks (`syms.len() <= 2048`: `wiki_nonopen`, `pattern`, `onebyte`)
    ///   -> `arm_a_ck_paged_nos2u_splat` (minimal setup + full glibc ERMS `memset` for long runs);
    /// - High-entropy Column blocks (`> 50%` `nn >= 32`: `random`, `records`)
    ///   -> `ck_zerocall_p2_flat8` (branchless 8-row flat unaligned P2, zero hot-loop calls);
    /// - General blocks (`text`, `wiki_all` Open, `runs`)
    ///   -> `ck_zerocall_reg7` (zero-call-in-loop XMM `(f0, f1)` + lazy flat-7 tail, 0 XMM spills).
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_t_ck_zerocall_adapt(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
        checkpoints: &mut Vec<[u32; 256]>,
    ) -> Result<usize, ErrorKind> {
        if b.syms.len() <= 2048 {
            arm_a_ck_paged_nos2u_splat(b, ll, counts, checkpoints)
        } else if is_high_entropy(b) {
            ck_zerocall_p2_flat8(b, ll, counts, checkpoints)
        } else {
            ck_zerocall_reg7(b, ll, counts, checkpoints)
        }
    }

    #[inline(never)]
    #[target_feature(enable = "avx2")]
    unsafe fn mtf_p2_flat_unal8_np(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let mut store = Base320([MaybeUninit::uninit(); 320]);
        let base = store.0.as_mut_ptr().cast::<u8>().add(32);
        *base.sub(1) = 0;
        std::ptr::copy_nonoverlapping(b.seq_to_unseq.as_ptr(), base, 256);
        let it = IDX_SIGNED.0.as_ptr();
        let clear_b0 = _mm256_setr_epi8(
            0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        );
        let out = ll.as_mut_ptr();
        let eob = b.eob;
        let (mut run, mut weight) = (0u32, 1u32);
        let mut n = 0usize;
        let mut result = Ok(());
        for &s in &b.syms {
            if s <= 1 {
                if weight >= MAX_RUN_WEIGHT {
                    result = Err(ErrorKind::RunTooLong);
                    break;
                }
                run += (s as u32 + 1) * weight;
                weight <<= 1;
                continue;
            }
            if run != 0 {
                let r = run as usize;
                if n + r > max {
                    result = Err(ErrorKind::BlockTooLong);
                    break;
                }
                let c = *base;
                *counts.get_unchecked_mut(c as usize) += run;
                if r <= 64 && n + 64 <= max {
                    let v = _mm256_set1_epi8(c as i8);
                    _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                    _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
                } else {
                    write_bytes_cold(out.add(n), c, r);
                }
                n += r;
                (run, weight) = (0, 1);
            }
            if s == eob {
                break;
            }
            if n >= max {
                result = Err(ErrorKind::BlockTooLong);
                break;
            }
            let nn = ((s - 1) as usize).min(255);
            let c = *base.add(nn);
            let tv = _mm256_set1_epi8(((nn as u8) ^ 0x80) as i8);
            for j in (1..8).rev() {
                let row = base.add(32 * j);
                let keep = _mm256_load_si256(row as *const __m256i);
                let on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
                let gt = _mm256_cmpgt_epi8(_mm256_load_si256(it.add(j) as *const __m256i), tv);
                _mm256_store_si256(row as *mut __m256i, _mm256_blendv_epi8(on, keep, gt));
            }
            let keep0 = _mm256_load_si256(base as *const __m256i);
            let on0 = _mm256_loadu_si256(base.sub(1) as *const __m256i);
            let gt0 = _mm256_cmpgt_epi8(_mm256_load_si256(it as *const __m256i), tv);
            let new0 = _mm256_blendv_epi8(on0, keep0, gt0);
            let new0 = _mm256_or_si256(
                _mm256_and_si256(new0, clear_b0),
                _mm256_zextsi128_si256(_mm_cvtsi32_si128(c as i32)),
            );
            _mm256_store_si256(base as *mut __m256i, new0);
            *counts.get_unchecked_mut(c as usize) += 1;
            *out.add(n) = c;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    /// Plain `mtf` (secondary diagnostic): zero-call XMM `(f0, f1)` + flat-7 tail / adaptive P2.
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_m_mtf_uni_adapt(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        if is_high_entropy(b) {
            return mtf_p2_flat_unal8_np(b, ll, counts);
        }
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        let mut store = Tail256([MaybeUninit::uninit(); 256]);
        let tail = store.0.as_mut_ptr().cast::<u8>().add(32);
        let mut tail_ready = false;
        let clear_b0 = _mm256_setr_epi8(
            0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        );
        let out = ll.as_mut_ptr();
        let eob = b.eob;
        let (mut run, mut weight) = (0u32, 1u32);
        let mut n = 0usize;
        let mut result = Ok(());
        for &s in &b.syms {
            if s <= 1 {
                if weight >= MAX_RUN_WEIGHT {
                    result = Err(ErrorKind::RunTooLong);
                    break;
                }
                run += (s as u32 + 1) * weight;
                weight <<= 1;
                continue;
            }
            if run != 0 {
                let r = run as usize;
                if n + r > max {
                    result = Err(ErrorKind::BlockTooLong);
                    break;
                }
                let c = _mm_cvtsi128_si32(f0) as u8;
                *counts.get_unchecked_mut(c as usize) += run;
                let v = _mm256_broadcastb_epi8(f0);
                if r <= 64 && n + 64 <= max {
                    _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                    _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
                } else {
                    fill_run_long_nocall(out, n, r, v);
                }
                n += r;
                (run, weight) = (0, 1);
            }
            if s == eob {
                break;
            }
            if n >= max {
                result = Err(ErrorKind::BlockTooLong);
                break;
            }
            let nn = ((s - 1) as usize).min(255);
            if nn < 32 {
                step32_xmm(&mut f0, &mut f1, nn);
            } else {
                if !tail_ready {
                    *tail.sub(1) = 0;
                    let src = s2u.as_ptr().add(32);
                    for j in 0..7 {
                        let chunk = _mm256_loadu_si256(src.add(32 * j) as *const __m256i);
                        _mm256_store_si256(tail.add(32 * j) as *mut __m256i, chunk);
                    }
                    tail_ready = true;
                }
                let f1_last = _mm_extract_epi8::<15>(f1) as u8;
                let v = tail_flat_unal7_inline(tail, nn - 32, f1_last, clear_b0);
                f1 = _mm_alignr_epi8::<15>(f1, f0);
                f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
            }
            let c = _mm_cvtsi128_si32(f0) as u8;
            *counts.get_unchecked_mut(c as usize) += 1;
            *out.add(n) = c;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }
}

type CkArmFn = fn(
    &Symbols,
    &mut Vec<u8>,
    &mut [u32; 256],
    &mut Vec<[u32; 256]>,
    Kernels,
) -> Result<usize, ErrorKind>;
type MtfArmFn = fn(&Symbols, &mut Vec<u8>, &mut [u32; 256], Kernels) -> Result<usize, ErrorKind>;

fn ck_arms() -> Vec<(&'static str, CkArmFn)> {
    #[cfg(target_arch = "x86_64")]
    {
        vec![
            ("ship_ck", |b, ll, c, cp, k| {
                mtf::mtf_checkpoints(b, ll, c, cp, k)
            }),
            ("A_paged_splat", |b, ll, c, cp, _| unsafe {
                kernels::arm_a_ck_paged_nos2u_splat(b, ll, c, cp)
            }),
            ("C_reg_cold7", |b, ll, c, cp, _| unsafe {
                kernels::arm_c_ck_reg_cold_tail(b, ll, c, cp)
            }),
            ("T_zerocall_adapt", |b, ll, c, cp, _| unsafe {
                kernels::arm_t_ck_zerocall_adapt(b, ll, c, cp)
            }),
        ]
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        vec![("ship_ck", |b, ll, c, cp, k| {
            mtf::mtf_checkpoints(b, ll, c, cp, k)
        })]
    }
}

fn mtf_arms() -> Vec<(&'static str, MtfArmFn)> {
    #[cfg(target_arch = "x86_64")]
    {
        vec![
            ("ship_mtf", |b, ll, c, k| mtf::mtf(b, ll, c, k)),
            ("M_uni_adapt", |b, ll, c, _| unsafe {
                kernels::arm_m_mtf_uni_adapt(b, ll, c)
            }),
        ]
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        vec![("ship_mtf", |b, ll, c, k| mtf::mtf(b, ll, c, k))]
    }
}

fn synthetic_cases() -> Vec<(&'static str, Symbols)> {
    let s2u: [u8; 256] = std::array::from_fn(|i| ((i * 73 + 19) & 255) as u8);
    let mk = |syms: Vec<u16>, eob: u16, max_len: usize| Symbols {
        syms,
        eob,
        max_len,
        seq_to_unseq: s2u,
        crc: 0,
        orig_ptr: 0,
    };
    let mut cases = Vec::new();
    cases.push(("empty_eob", mk(vec![2], 2, 100)));
    cases.push((
        "tiny_eob_5",
        mk(vec![0, 1, 2, 3, 4, 0, 0, 0, 2, 1, 5], 5, 100),
    ));
    let mut syms_exact_ck = vec![0u16; 14];
    syms_exact_ck.push(2);
    syms_exact_ck.push(10);
    cases.push(("exact_checkpoint_end", mk(syms_exact_ck, 10, 40_000)));
    let mut syms_cross1 = vec![2, 3, 4, 5, 6];
    syms_cross1.extend([0u16; 14]);
    syms_cross1.extend([7, 8, 9, 20]);
    cases.push(("run_crossing_one_ckpt", mk(syms_cross1, 20, 40_000)));
    let mut syms_cross4 = vec![3];
    syms_cross4.extend([0u16; 16]);
    syms_cross4.extend([4, 0, 1, 20]);
    cases.push(("run_crossing_multi_ckpts", mk(syms_cross4, 20, 100_000)));
    let mut syms_max = vec![0u16; 7];
    syms_max.push(2);
    syms_max.push(10);
    cases.push(("run_exact_max_len", mk(syms_max, 10, 128)));
    let mut syms_rtl = vec![0u16; 22];
    syms_rtl.push(2);
    cases.push(("err_run_too_long", mk(syms_rtl, 2, 900_000)));
    cases.push((
        "err_block_too_long_run",
        mk(vec![0, 0, 0, 0, 2, 10], 10, 10),
    ));
    cases.push(("err_block_too_long_lit", mk(vec![2, 3, 4, 5, 6, 10], 10, 4)));
    cases.push(("trailing_run_no_eob", mk(vec![2, 3, 0, 1], 10, 100)));
    let mut syms_255: Vec<u16> = (2..=256).collect();
    syms_255.extend((2..=256).rev());
    syms_255.push(257);
    cases.push(("full_nn_sweep_255", mk(syms_255, 257, 1000)));
    cases.push(("clamp_over_256", mk(vec![2, 300, 257, 400], 400, 100)));
    cases
}

fn load_all_corpora(dir: &Path) -> Vec<(String, Vec<LoadedBlock>, usize)> {
    let wiki_all = load_blocks(&dir.join("wikidata.bz2"));
    let wiki_nonopen: Vec<LoadedBlock> = wiki_all
        .iter()
        .filter(|b| b.want_n < 100_000)
        .map(|b| LoadedBlock {
            syms: Symbols {
                syms: b.syms.syms.clone(),
                eob: b.syms.eob,
                max_len: b.syms.max_len,
                seq_to_unseq: b.syms.seq_to_unseq,
                crc: b.syms.crc,
                orig_ptr: b.syms.orig_ptr,
            },
            want_ll: b.want_ll.clone(),
            want_counts: b.want_counts,
            want_cps: b.want_cps.clone(),
            want_n: b.want_n,
        })
        .collect();
    let mut sets = vec![
        ("wiki_nonopen".to_string(), wiki_nonopen, 400),
        ("wiki_all".to_string(), wiki_all, 1),
    ];
    for (name, passes) in [
        ("text", 1),
        ("random", 1),
        ("records", 1),
        ("runs", 10),
        ("pattern", 10),
        ("onebyte", 100),
    ] {
        sets.push((
            name.to_string(),
            load_blocks(&dir.join(format!("{name}.bz2"))),
            passes,
        ));
    }
    sets
}

fn run_exact(dir: &Path, k: Kernels) {
    let sets = load_all_corpora(dir);
    let cks = ck_arms();
    let mtfs = mtf_arms();
    let mut ll = Vec::with_capacity(900_000);
    let mut counts = [0u32; 256];
    let mut cps = Vec::with_capacity(64);
    let mut checks = 0usize;

    for (cname, blocks, _) in &sets {
        if cname == "wiki_nonopen" {
            continue;
        }
        for (bi, blk) in blocks.iter().enumerate() {
            for (aname, f) in &cks {
                let n = f(&blk.syms, &mut ll, &mut counts, &mut cps, k)
                    .unwrap_or_else(|e| panic!("ck {aname} failed on {cname}[{bi}]: {e:?}"));
                assert_eq!(n, blk.want_n, "n mismatch {aname} {cname}[{bi}]");
                assert_eq!(ll, blk.want_ll, "ll mismatch {aname} {cname}[{bi}]");
                assert_eq!(
                    counts, blk.want_counts,
                    "counts mismatch {aname} {cname}[{bi}]"
                );
                assert_eq!(cps, blk.want_cps, "cps mismatch {aname} {cname}[{bi}]");
                checks += 1;
            }
            for (aname, f) in &mtfs {
                let n = f(&blk.syms, &mut ll, &mut counts, k)
                    .unwrap_or_else(|e| panic!("mtf {aname} failed on {cname}[{bi}]: {e:?}"));
                assert_eq!(n, blk.want_n, "n mismatch {aname} {cname}[{bi}]");
                assert_eq!(ll, blk.want_ll, "ll mismatch {aname} {cname}[{bi}]");
                assert_eq!(
                    counts, blk.want_counts,
                    "counts mismatch {aname} {cname}[{bi}]"
                );
                checks += 1;
            }
        }
    }

    for (name, syms) in synthetic_cases() {
        let mut ref_ll = Vec::new();
        let mut ref_c = [0u32; 256];
        let mut ref_cp = Vec::new();
        if name == "clamp_over_256" {
            let ref_mtf = mtf::mtf(&syms, &mut ref_ll, &mut ref_c, k);
            for (aname, f) in &cks {
                if *aname == "ship_ck" {
                    continue;
                }
                let res = f(&syms, &mut ll, &mut counts, &mut cps, k);
                assert_eq!(res, ref_mtf, "synthetic {name} {aname} result mismatch");
                assert_eq!(ll, ref_ll, "synthetic {name} {aname} ll mismatch");
                assert_eq!(counts, ref_c, "synthetic {name} {aname} counts mismatch");
                checks += 1;
            }
            for (aname, f) in &mtfs {
                let res = f(&syms, &mut ll, &mut counts, k);
                assert_eq!(res, ref_mtf, "synthetic {name} {aname} result mismatch");
                assert_eq!(ll, ref_ll, "synthetic {name} {aname} ll mismatch");
                assert_eq!(counts, ref_c, "synthetic {name} {aname} counts mismatch");
                checks += 1;
            }
            continue;
        }
        let ref_res = mtf::mtf_checkpoints_scalar(&syms, &mut ref_ll, &mut ref_c, &mut ref_cp);
        for (aname, f) in &cks {
            let res = f(&syms, &mut ll, &mut counts, &mut cps, k);
            assert_eq!(res, ref_res, "synthetic {name} {aname} result mismatch");
            if res.is_ok() {
                assert_eq!(ll, ref_ll, "synthetic {name} {aname} ll mismatch");
                assert_eq!(counts, ref_c, "synthetic {name} {aname} counts mismatch");
                assert_eq!(cps, ref_cp, "synthetic {name} {aname} cps mismatch");
            }
            checks += 1;
        }
        let mut ref_ll2 = Vec::new();
        let mut ref_c2 = [0u32; 256];
        let ref_mtf = mtf::mtf_scalar(&syms, &mut ref_ll2, &mut ref_c2);
        for (aname, f) in &mtfs {
            let res = f(&syms, &mut ll, &mut counts, k);
            assert_eq!(res, ref_mtf, "synthetic {name} {aname} result mismatch");
            if res.is_ok() {
                assert_eq!(ll, ref_ll2, "synthetic {name} {aname} ll mismatch");
                assert_eq!(counts, ref_c2, "synthetic {name} {aname} counts mismatch");
            }
            checks += 1;
        }
    }
    println!("EXACT PASS: {checks}/{checks} checks identical (648 corpus blocks + 12 synthetic edge/error cases)");
}

fn summarize(samples: &[f64]) -> (f64, f64, f64) {
    let mut v = samples.to_vec();
    v.sort_by(f64::total_cmp);
    (v[v.len() / 2], v[0], v[v.len() - 1])
}

fn run_bench(dir: &Path, k: Kernels, final_only: bool) {
    const REPEATS: usize = 7;
    let sets = load_all_corpora(dir);
    let cks: Vec<_> = ck_arms()
        .into_iter()
        .filter(|(name, _)| {
            !final_only || matches!(*name, "ship_ck" | "A_paged_splat" | "T_zerocall_adapt")
        })
        .collect();
    let mtfs = mtf_arms();
    let mut ll = Vec::with_capacity(900_000);
    let mut counts = [0u32; 256];
    let mut cps = Vec::with_capacity(64);

    println!(
        "=== mtf_checkpoints (7 interleaved repeats, steal-free CLOCK_THREAD_CPUTIME_ID ms) ==="
    );
    for (cname, blocks, passes) in &sets {
        let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(REPEATS); cks.len()];
        for (_, f) in &cks {
            for blk in blocks {
                let n = f(&blk.syms, &mut ll, &mut counts, &mut cps, k).unwrap();
                std::hint::black_box(n);
            }
        }
        for r in 0..REPEATS {
            for step in 0..cks.len() {
                let ai = if r % 2 == 0 {
                    (r + step) % cks.len()
                } else {
                    (r + cks.len() - 1 - step) % cks.len()
                };
                let f = cks[ai].1;
                let t0 = thread_cpu_ns();
                for _ in 0..*passes {
                    for blk in blocks {
                        let n = f(&blk.syms, &mut ll, &mut counts, &mut cps, k).unwrap();
                        std::hint::black_box(n);
                    }
                }
                let ms = (thread_cpu_ns() - t0) as f64 / (1e6 * (*passes as f64));
                times[ai].push(ms);
            }
        }
        let (base_med, base_lo, base_hi) = summarize(&times[0]);
        println!("\n[ck] {cname} (blks={}, passes={passes}):", blocks.len());
        for (ai, (aname, _)) in cks.iter().enumerate() {
            let (med, lo, hi) = summarize(&times[ai]);
            let delta = (med / base_med - 1.0) * 100.0;
            let disj = if ai == 0 {
                "REF"
            } else if hi < base_lo {
                "FASTER*"
            } else if lo > base_hi {
                "SLOWER!"
            } else {
                "ovlp"
            };
            println!(
                "  {:<16} {:8.3} [{:8.3}..{:8.3}] ms  ({:+6.2}%)  {}",
                aname, med, lo, hi, delta, disj
            );
        }
    }

    println!("\n=== mtf (plain, no checkpoints; 7 interleaved repeats, thread CPU ms) ===");
    for (cname, blocks, passes) in &sets {
        let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(REPEATS); mtfs.len()];
        for (_, f) in &mtfs {
            for blk in blocks {
                let n = f(&blk.syms, &mut ll, &mut counts, k).unwrap();
                std::hint::black_box(n);
            }
        }
        for r in 0..REPEATS {
            for step in 0..mtfs.len() {
                let ai = if r % 2 == 0 {
                    (r + step) % mtfs.len()
                } else {
                    (r + mtfs.len() - 1 - step) % mtfs.len()
                };
                let f = mtfs[ai].1;
                let t0 = thread_cpu_ns();
                for _ in 0..*passes {
                    for blk in blocks {
                        let n = f(&blk.syms, &mut ll, &mut counts, k).unwrap();
                        std::hint::black_box(n);
                    }
                }
                let ms = (thread_cpu_ns() - t0) as f64 / (1e6 * (*passes as f64));
                times[ai].push(ms);
            }
        }
        let (base_med, base_lo, base_hi) = summarize(&times[0]);
        println!("\n[mtf] {cname} (blks={}, passes={passes}):", blocks.len());
        for (ai, (aname, _)) in mtfs.iter().enumerate() {
            let (med, lo, hi) = summarize(&times[ai]);
            let delta = (med / base_med - 1.0) * 100.0;
            let disj = if ai == 0 {
                "REF"
            } else if hi < base_lo {
                "FASTER*"
            } else if lo > base_hi {
                "SLOWER!"
            } else {
                "ovlp"
            };
            println!(
                "  {:<16} {:8.3} [{:8.3}..{:8.3}] ms  ({:+6.2}%)  {}",
                aname, med, lo, hi, delta, disj
            );
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("exact");
    let dir = PathBuf::from(
        args.get(2)
            .map(String::as_str)
            .unwrap_or("/tmp/mtf-nonopen-corpora"),
    );
    let k = Kernels::detect();
    assert!(k.avx2(), "AVX2 required");
    match mode {
        "exact" => run_exact(&dir, k),
        "bench" => run_bench(&dir, k, false),
        "bench_final" => run_bench(&dir, k, true),
        other => panic!("unknown mode {other} (use exact | bench | bench_final)"),
    }
}
