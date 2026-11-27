//! Smooth-zone probe (`mtf-column-front-dissection`, zero `src/` edits):
//! dissects `src/mtf.rs` (`Column`-route MTF/RLE2 front) on off-wikidata corpora and
//! evaluates 100% byte-exact candidate kernels over 7 interleaved repeats on baseline x86-64
//! and `-C target-cpu=native`.
//!
//! Run:
//!   cargo run --release --example mtf_column_front_probe -- exact /tmp/mtf_col_corpora/*.bz2
//!   cargo run --release --example mtf_column_front_probe -- bench /tmp/mtf_col_corpora/*.bz2
//!   RUSTFLAGS="-C target-cpu=native" cargo run --release --target-dir target/native --example mtf_column_front_probe -- bench /tmp/mtf_col_corpora/*.bz2

use mibz2::block::Symbols;
use mibz2::runs;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, ErrorKind, Kernels};
use std::path::Path;
use std::time::Instant;

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

struct CorpusBlock {
    corpus: String,
    is_column: bool,
    syms: Symbols,
    want_ll: Vec<u8>,
    want_counts: [u32; 256],
    want_n: usize,
}

fn load_corpus(path: &Path) -> Vec<CorpusBlock> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    let data = std::fs::read(path).expect("read bz2");
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut blocks = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    while let Event::Block { symbols, .. } = scanner.next(Vec::new()).expect("scan") {
        let is_column = !runs::takes_runs(&symbols);
        let n = mtf::mtf_scalar(&symbols, &mut ll, &mut counts).expect("mtf_scalar");
        blocks.push(CorpusBlock {
            corpus: name.clone(),
            is_column,
            syms: symbols,
            want_ll: ll.clone(),
            want_counts: counts,
            want_n: n,
        });
    }
    blocks
}

#[cfg(target_arch = "x86_64")]
mod kernels {
    use super::MAX_RUN_WEIGHT;
    use mibz2::block::Symbols;
    use mibz2::ErrorKind;
    use std::arch::x86_64::*;

    // ---- 32-byte L1 step table (identical to src/mtf.rs, STEPS32[0] is exact identity) ----
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

    // ---- 64-byte L1 step table (4 XMM registers f0..f3, 8 KiB in L1d; STEPS64[0] = identity) ----
    #[repr(C, align(128))]
    #[derive(Clone, Copy)]
    struct Step64 {
        a0: [u8; 16],
        a1: [u8; 16],
        a2: [u8; 16],
        a3: [u8; 16],
        m1: [u8; 16],
        m2: [u8; 16],
        m3: [u8; 16],
        _pad: [u8; 16],
    }

    const fn make_steps64() -> [Step64; 64] {
        const Z: u8 = 0x80;
        let mut t = [Step64 {
            a0: [0; 16],
            a1: [Z; 16],
            a2: [Z; 16],
            a3: [Z; 16],
            m1: [0; 16],
            m2: [0; 16],
            m3: [0; 16],
            _pad: [0; 16],
        }; 64];
        let mut nn = 0;
        while nn < 64 {
            let mut i = 0;
            while i < 16 {
                if nn < 16 {
                    t[nn].a0[i] = if i == 0 {
                        nn as u8
                    } else if i <= nn {
                        (i - 1) as u8
                    } else {
                        i as u8
                    };
                } else {
                    t[nn].a0[i] = if i == 0 { Z } else { (i - 1) as u8 };
                    if nn < 32 {
                        if i == 0 {
                            t[nn].a1[i] = (nn - 16) as u8;
                        }
                        if i <= nn - 16 {
                            t[nn].m1[i] = 0xff;
                        }
                    } else if nn < 48 {
                        if i == 0 {
                            t[nn].a2[i] = (nn - 32) as u8;
                        }
                        t[nn].m1[i] = 0xff;
                        if i <= nn - 32 {
                            t[nn].m2[i] = 0xff;
                        }
                    } else {
                        if i == 0 {
                            t[nn].a3[i] = (nn - 48) as u8;
                        }
                        t[nn].m1[i] = 0xff;
                        t[nn].m2[i] = 0xff;
                        if i <= nn - 48 {
                            t[nn].m3[i] = 0xff;
                        }
                    }
                }
                i += 1;
            }
            nn += 1;
        }
        t
    }

    static STEPS64: [Step64; 64] = make_steps64();

    // Signed index thresholds for 1-instruction `vpcmpgtb` row masks (`^ 0x80` unsigned compare).
    // Row j, lane i covers position p = 32*j + i.
    // For full 8-row `y0..y7`: want shift iff (p == 0 && nn >= 1) || (1 <= p <= nn)
    //   <=> (p.saturating_sub(1) ^ 0x80) < (nn ^ 0x80) as signed i8!
    #[repr(C, align(32))]
    struct SignedRows([[i8; 32]; 8]);

    const fn make_idx_prev() -> SignedRows {
        let mut r = [[0i8; 32]; 8];
        let mut j = 0;
        while j < 8 {
            let mut i = 0;
            while i < 32 {
                let p = 32 * j + i;
                let thresh = if p == 0 { 0usize } else { p - 1 };
                r[j][i] = ((thresh as u8) ^ 0x80) as i8;
                i += 1;
            }
            j += 1;
        }
        SignedRows(r)
    }

    static IDX_PREV: SignedRows = make_idx_prev();

    // For tail rows `t = nn - BASE` (j = 0..7, pos q = 32*j + i): want shift iff q <= t
    //   <=> ((q ^ 0x80) as i8) < (((t + 1) ^ 0x80) as i8) -- valid for all t <= 223!
    const fn make_idx_tail() -> SignedRows {
        let mut r = [[0i8; 32]; 8];
        let mut j = 0;
        while j < 8 {
            let mut i = 0;
            while i < 32 {
                let q = 32 * j + i;
                r[j][i] = ((q as u8) ^ 0x80) as i8;
                i += 1;
            }
            j += 1;
        }
        SignedRows(r)
    }

    static IDX_TAIL: SignedRows = make_idx_tail();

    // SHUF_BYTE0[r] = [r, 0x80, 0x80, ..., 0x80]: puts byte r in lane byte 0 and zeroes 1..15.
    #[repr(C, align(16))]
    struct ShufByte0([[u8; 16]; 16]);

    const fn make_shuf_byte0() -> ShufByte0 {
        let mut s = [[0x80u8; 16]; 16];
        let mut r = 0;
        while r < 16 {
            s[r][0] = r as u8;
            r += 1;
        }
        ShufByte0(s)
    }

    static SHUF_BYTE0: ShufByte0 = make_shuf_byte0();

    #[inline(always)]
    unsafe fn shift_on(r: __m256i, below: __m256i) -> __m256i {
        _mm256_alignr_epi8::<15>(r, _mm256_permute2x128_si256::<0x03>(r, below))
    }

    #[inline(always)]
    unsafe fn shift_y0(y0: __m256i) -> __m256i {
        // below = [zero_128, y0.lo] via permute2x128::<0x08>(y0, y0)
        _mm256_alignr_epi8::<15>(y0, _mm256_permute2x128_si256::<0x08>(y0, y0))
    }

    #[inline(always)]
    unsafe fn fill_run_avx2(out: *mut u8, n: usize, r: usize, max: usize, c: u8, v: __m256i) {
        if r <= 64 && n + 64 <= max {
            _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
            _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
        } else {
            let mut p = n;
            let end = n + r;
            while p + 32 <= end {
                _mm256_storeu_si256(out.add(p) as *mut __m256i, v);
                p += 32;
            }
            if p < end {
                if r >= 32 {
                    _mm256_storeu_si256(out.add(end - 32) as *mut __m256i, v);
                } else {
                    while p < end {
                        *out.add(p) = c;
                        p += 1;
                    }
                }
            }
        }
    }

    #[inline(always)]
    unsafe fn step32(f0: &mut __m128i, f1: &mut __m128i, nn: usize) {
        let t = STEPS32.as_ptr().add(nn);
        let a = _mm_load_si128((*t).a.as_ptr() as *const __m128i);
        let bb = _mm_load_si128((*t).b.as_ptr() as *const __m128i);
        let m = _mm_load_si128((*t).m.as_ptr() as *const __m128i);
        let g0 = _mm_or_si128(_mm_shuffle_epi8(*f0, a), _mm_shuffle_epi8(*f1, bb));
        *f1 = _mm_blendv_epi8(*f1, _mm_alignr_epi8::<15>(*f1, *f0), m);
        *f0 = g0;
    }

    #[inline(always)]
    unsafe fn step64(
        f0: &mut __m128i,
        f1: &mut __m128i,
        f2: &mut __m128i,
        f3: &mut __m128i,
        nn: usize,
    ) {
        let t = STEPS64.as_ptr().add(nn);
        let a0 = _mm_load_si128((*t).a0.as_ptr() as *const __m128i);
        let a1 = _mm_load_si128((*t).a1.as_ptr() as *const __m128i);
        let a2 = _mm_load_si128((*t).a2.as_ptr() as *const __m128i);
        let a3 = _mm_load_si128((*t).a3.as_ptr() as *const __m128i);
        let m1 = _mm_load_si128((*t).m1.as_ptr() as *const __m128i);
        let m2 = _mm_load_si128((*t).m2.as_ptr() as *const __m128i);
        let m3 = _mm_load_si128((*t).m3.as_ptr() as *const __m128i);
        let g0 = _mm_or_si128(
            _mm_or_si128(_mm_shuffle_epi8(*f0, a0), _mm_shuffle_epi8(*f1, a1)),
            _mm_or_si128(_mm_shuffle_epi8(*f2, a2), _mm_shuffle_epi8(*f3, a3)),
        );
        let nf3 = _mm_blendv_epi8(*f3, _mm_alignr_epi8::<15>(*f3, *f2), m3);
        let nf2 = _mm_blendv_epi8(*f2, _mm_alignr_epi8::<15>(*f2, *f1), m2);
        let nf1 = _mm_blendv_epi8(*f1, _mm_alignr_epi8::<15>(*f1, *f0), m1);
        *f0 = g0;
        *f1 = nf1;
        *f2 = nf2;
        *f3 = nf3;
    }

    #[inline(always)]
    unsafe fn extract_byte_ymm(yk: __m256i, r: usize) -> __m128i {
        let lo = _mm256_castsi256_si128(yk);
        let hi = _mm256_extracti128_si256::<1>(yk);
        let ml = _mm_set1_epi32(-(((r >> 4) & 1) as i32));
        let lane = _mm_blendv_epi8(lo, hi, ml);
        let shuf = _mm_load_si128(SHUF_BYTE0.0.as_ptr().add(r & 15) as *const __m128i);
        _mm_shuffle_epi8(lane, shuf)
    }

    // Extract byte `nn` from 8 YMM registers (y0..y7) and shift all 8 in parallel.
    // Returns byte `v` in lane 0 byte 0 of XMM (other bytes zero).
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    unsafe fn step_reg8_all(
        y0: &mut __m256i,
        y1: &mut __m256i,
        y2: &mut __m256i,
        y3: &mut __m256i,
        y4: &mut __m256i,
        y5: &mut __m256i,
        y6: &mut __m256i,
        y7: &mut __m256i,
        nn: usize,
    ) -> __m128i {
        // 1. Extract y_k via 3-level binary mux
        let b0 = _mm256_set1_epi32(-(((nn >> 5) & 1) as i32));
        let b1 = _mm256_set1_epi32(-(((nn >> 6) & 1) as i32));
        let b2 = _mm256_set1_epi32(-(((nn >> 7) & 1) as i32));
        let a0 = _mm256_blendv_epi8(*y0, *y1, b0);
        let a1 = _mm256_blendv_epi8(*y2, *y3, b0);
        let a2 = _mm256_blendv_epi8(*y4, *y5, b0);
        let a3 = _mm256_blendv_epi8(*y6, *y7, b0);
        let c0 = _mm256_blendv_epi8(a0, a1, b1);
        let c1 = _mm256_blendv_epi8(a2, a3, b1);
        let yk = _mm256_blendv_epi8(c0, c1, b2);
        let v_xmm = extract_byte_ymm(yk, nn & 31);

        // 2. Shift all 8 rows right by 1 byte where p <= nn (1 vpcmpgtb per row!)
        let nv = _mm256_set1_epi8(((nn as u8) ^ 0x80) as i8);
        let ip = IDX_PREV.0.as_ptr();
        let m7 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(7) as *const __m256i));
        let m6 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(6) as *const __m256i));
        let m5 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(5) as *const __m256i));
        let m4 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(4) as *const __m256i));
        let m3 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(3) as *const __m256i));
        let m2 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(2) as *const __m256i));
        let m1 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(1) as *const __m256i));
        let m0 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip as *const __m256i));

        *y7 = _mm256_blendv_epi8(*y7, shift_on(*y7, *y6), m7);
        *y6 = _mm256_blendv_epi8(*y6, shift_on(*y6, *y5), m6);
        *y5 = _mm256_blendv_epi8(*y5, shift_on(*y5, *y4), m5);
        *y4 = _mm256_blendv_epi8(*y4, shift_on(*y4, *y3), m4);
        *y3 = _mm256_blendv_epi8(*y3, shift_on(*y3, *y2), m3);
        *y2 = _mm256_blendv_epi8(*y2, shift_on(*y2, *y1), m2);
        *y1 = _mm256_blendv_epi8(*y1, shift_on(*y1, *y0), m1);
        *y0 = _mm256_or_si256(
            _mm256_blendv_epi8(*y0, shift_y0(*y0), m0),
            _mm256_zextsi128_si256(v_xmm),
        );
        v_xmm
    }

    // Shift 7 tail YMM registers y1..y7 (positions 32..255, t = nn - 32 in 0..224) and return v_xmm
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    unsafe fn step_tail7_reg(
        f1: __m128i,
        y1: &mut __m256i,
        y2: &mut __m256i,
        y3: &mut __m256i,
        y4: &mut __m256i,
        y5: &mut __m256i,
        y6: &mut __m256i,
        y7: &mut __m256i,
        t: usize,
    ) -> __m128i {
        let b0 = _mm256_set1_epi32(-(((t >> 5) & 1) as i32));
        let b1 = _mm256_set1_epi32(-(((t >> 6) & 1) as i32));
        let b2 = _mm256_set1_epi32(-(((t >> 7) & 1) as i32));
        let a0 = _mm256_blendv_epi8(*y1, *y2, b0);
        let a1 = _mm256_blendv_epi8(*y3, *y4, b0);
        let a2 = _mm256_blendv_epi8(*y5, *y6, b0);
        let c0 = _mm256_blendv_epi8(a0, a1, b1);
        let c1 = _mm256_blendv_epi8(a2, *y7, b1);
        let yk = _mm256_blendv_epi8(c0, c1, b2);
        let v_xmm = extract_byte_ymm(yk, t & 31);

        let tv = _mm256_set1_epi8((((t + 1) as u8) ^ 0x80) as i8);
        let it = IDX_TAIL.0.as_ptr();
        let below1 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(f1), f1);
        let m6 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(6) as *const __m256i));
        let m5 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(5) as *const __m256i));
        let m4 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(4) as *const __m256i));
        let m3 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(3) as *const __m256i));
        let m2 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(2) as *const __m256i));
        let m1 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(1) as *const __m256i));
        let m0 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it as *const __m256i));

        *y7 = _mm256_blendv_epi8(*y7, shift_on(*y7, *y6), m6);
        *y6 = _mm256_blendv_epi8(*y6, shift_on(*y6, *y5), m5);
        *y5 = _mm256_blendv_epi8(*y5, shift_on(*y5, *y4), m4);
        *y4 = _mm256_blendv_epi8(*y4, shift_on(*y4, *y3), m3);
        *y3 = _mm256_blendv_epi8(*y3, shift_on(*y3, *y2), m2);
        *y2 = _mm256_blendv_epi8(*y2, shift_on(*y2, *y1), m1);
        *y1 = _mm256_blendv_epi8(*y1, shift_on(*y1, below1), m0);
        v_xmm
    }

    // Shift 6 tail YMM registers y2..y7 (positions 64..255, t = nn - 64 in 0..192) and return v_xmm
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    unsafe fn step_tail6_reg(
        f3: __m128i,
        y2: &mut __m256i,
        y3: &mut __m256i,
        y4: &mut __m256i,
        y5: &mut __m256i,
        y6: &mut __m256i,
        y7: &mut __m256i,
        t: usize,
    ) -> __m128i {
        let b0 = _mm256_set1_epi32(-(((t >> 5) & 1) as i32));
        let b1 = _mm256_set1_epi32(-(((t >> 6) & 1) as i32));
        let b2 = _mm256_set1_epi32(-(((t >> 7) & 1) as i32));
        let a0 = _mm256_blendv_epi8(*y2, *y3, b0);
        let a1 = _mm256_blendv_epi8(*y4, *y5, b0);
        let a2 = _mm256_blendv_epi8(*y6, *y7, b0);
        let c0 = _mm256_blendv_epi8(a0, a1, b1);
        let yk = _mm256_blendv_epi8(c0, a2, b2);
        let v_xmm = extract_byte_ymm(yk, t & 31);

        let tv = _mm256_set1_epi8((((t + 1) as u8) ^ 0x80) as i8);
        let it = IDX_TAIL.0.as_ptr();
        let below2 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(f3), f3);
        let m5 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(5) as *const __m256i));
        let m4 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(4) as *const __m256i));
        let m3 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(3) as *const __m256i));
        let m2 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(2) as *const __m256i));
        let m1 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(1) as *const __m256i));
        let m0 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it as *const __m256i));

        *y7 = _mm256_blendv_epi8(*y7, shift_on(*y7, *y6), m5);
        *y6 = _mm256_blendv_epi8(*y6, shift_on(*y6, *y5), m4);
        *y5 = _mm256_blendv_epi8(*y5, shift_on(*y5, *y4), m3);
        *y4 = _mm256_blendv_epi8(*y4, shift_on(*y4, *y3), m2);
        *y3 = _mm256_blendv_epi8(*y3, shift_on(*y3, *y2), m1);
        *y2 = _mm256_blendv_epi8(*y2, shift_on(*y2, below2), m0);
        v_xmm
    }

    // Stack second() with signed vpcmpgtb masks (used only in Arm A)
    #[inline(always)]
    unsafe fn second_stack_signed(tail: *mut u8, t: usize, f1: __m128i) -> u8 {
        let v = *tail.add(t);
        let below0 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(f1), f1);
        let tv = _mm256_set1_epi8((((t + 1) as u8) ^ 0x80) as i8);
        let it = IDX_TAIL.0.as_ptr();
        if t < 32 {
            let r0 = _mm256_load_si256(tail as *const __m256i);
            let m = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it as *const __m256i));
            _mm256_store_si256(
                tail as *mut __m256i,
                _mm256_blendv_epi8(r0, shift_on(r0, below0), m),
            );
        } else {
            let r0 = _mm256_load_si256(tail as *const __m256i);
            let r1 = _mm256_load_si256(tail.add(32) as *const __m256i);
            let r2 = _mm256_load_si256(tail.add(64) as *const __m256i);
            let r3 = _mm256_load_si256(tail.add(96) as *const __m256i);
            let r4 = _mm256_load_si256(tail.add(128) as *const __m256i);
            let r5 = _mm256_load_si256(tail.add(160) as *const __m256i);
            let r6 = _mm256_load_si256(tail.add(192) as *const __m256i);
            let m0 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it as *const __m256i));
            let m1 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(1) as *const __m256i));
            let m2 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(2) as *const __m256i));
            let m3 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(3) as *const __m256i));
            let m4 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(4) as *const __m256i));
            let m5 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(5) as *const __m256i));
            let m6 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(6) as *const __m256i));
            _mm256_store_si256(
                tail as *mut __m256i,
                _mm256_blendv_epi8(r0, shift_on(r0, below0), m0),
            );
            _mm256_store_si256(
                tail.add(32) as *mut __m256i,
                _mm256_blendv_epi8(r1, shift_on(r1, r0), m1),
            );
            _mm256_store_si256(
                tail.add(64) as *mut __m256i,
                _mm256_blendv_epi8(r2, shift_on(r2, r1), m2),
            );
            _mm256_store_si256(
                tail.add(96) as *mut __m256i,
                _mm256_blendv_epi8(r3, shift_on(r3, r2), m3),
            );
            _mm256_store_si256(
                tail.add(128) as *mut __m256i,
                _mm256_blendv_epi8(r4, shift_on(r4, r3), m4),
            );
            _mm256_store_si256(
                tail.add(160) as *mut __m256i,
                _mm256_blendv_epi8(r5, shift_on(r5, r4), m5),
            );
            _mm256_store_si256(
                tail.add(192) as *mut __m256i,
                _mm256_blendv_epi8(r6, shift_on(r6, r5), m6),
            );
        }
        v
    }

    // ---- Arm A: Branchless RLE2 + 32B L1 + signed-vpcmpgtb stack second() + inline AVX2 fill ----
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_a_brl_rle_tier32(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        #[repr(align(64))]
        struct Rows([u8; 224]);
        let mut rows = Rows([0u8; 224]);
        rows.0.copy_from_slice(&s2u[32..]);
        let tail = rows.0.as_mut_ptr();
        let out = ll.as_mut_ptr();
        let eob = b.eob;
        let fast_lim = 33u16.min(eob);
        let safe_max = max.saturating_sub(64);
        let (mut run, mut weight) = (0u32, 1u32);
        let mut n = 0usize;
        let mut result = Ok(());
        for &s in &b.syms {
            if s < fast_lim && run <= 32 && n < safe_max {
                let is_run = s <= 1;
                let emit_run = if is_run { 0 } else { run };
                let c0 = _mm_cvtsi128_si32(f0) as u8;
                let v = _mm256_broadcastb_epi8(f0);
                _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                *counts.get_unchecked_mut(c0 as usize) += emit_run;
                n += emit_run as usize;

                let add_run = (s as u32 + 1) * weight;
                run = if is_run { run + add_run } else { 0 };
                weight = if is_run { weight << 1 } else { 1 };

                let nn = if is_run { 0usize } else { (s - 1) as usize };
                step32(&mut f0, &mut f1, nn);

                let emit_lit = (!is_run) as u32;
                let c1 = _mm_cvtsi128_si32(f0) as u8;
                *counts.get_unchecked_mut(c1 as usize) += emit_lit;
                *out.add(n) = c1;
                n += emit_lit as usize;
                continue;
            }
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let v = second_stack_signed(tail, nn - 32, f1);
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

    // ---- Arm B: All 256B in SIMD regs (f0, f1 + y1..y7), branched RLE2, zero stack/memset ----
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_b_reg8_tier32(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let p = b.seq_to_unseq.as_ptr();
        let mut f0 = _mm_loadu_si128(p as *const __m128i);
        let mut f1 = _mm_loadu_si128(p.add(16) as *const __m128i);
        let mut y1 = _mm256_loadu_si256(p.add(32) as *const __m256i);
        let mut y2 = _mm256_loadu_si256(p.add(64) as *const __m256i);
        let mut y3 = _mm256_loadu_si256(p.add(96) as *const __m256i);
        let mut y4 = _mm256_loadu_si256(p.add(128) as *const __m256i);
        let mut y5 = _mm256_loadu_si256(p.add(160) as *const __m256i);
        let mut y6 = _mm256_loadu_si256(p.add(192) as *const __m256i);
        let mut y7 = _mm256_loadu_si256(p.add(224) as *const __m256i);
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let v_xmm = step_tail7_reg(
                    f1,
                    &mut y1,
                    &mut y2,
                    &mut y3,
                    &mut y4,
                    &mut y5,
                    &mut y6,
                    &mut y7,
                    nn - 32,
                );
                f1 = _mm_alignr_epi8::<15>(f1, f0);
                f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), v_xmm);
            }
            let c = _mm_cvtsi128_si32(f0) as u8;
            *counts.get_unchecked_mut(c as usize) += 1;
            *out.add(n) = c;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    // ---- Arm C: Full 256B branchless 8xYMM (ZERO branches on nn), branched RLE2 ----
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_c_reg8_branchless(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let p = b.seq_to_unseq.as_ptr();
        let mut y0 = _mm256_loadu_si256(p as *const __m256i);
        let mut y1 = _mm256_loadu_si256(p.add(32) as *const __m256i);
        let mut y2 = _mm256_loadu_si256(p.add(64) as *const __m256i);
        let mut y3 = _mm256_loadu_si256(p.add(96) as *const __m256i);
        let mut y4 = _mm256_loadu_si256(p.add(128) as *const __m256i);
        let mut y5 = _mm256_loadu_si256(p.add(160) as *const __m256i);
        let mut y6 = _mm256_loadu_si256(p.add(192) as *const __m256i);
        let mut y7 = _mm256_loadu_si256(p.add(224) as *const __m256i);
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
                let f0 = _mm256_castsi256_si128(y0);
                let c = _mm_cvtsi128_si32(f0) as u8;
                *counts.get_unchecked_mut(c as usize) += run;
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
            let v_xmm = step_reg8_all(
                &mut y0, &mut y1, &mut y2, &mut y3, &mut y4, &mut y5, &mut y6, &mut y7, nn,
            );
            let c = _mm_cvtsi128_si32(v_xmm) as u8;
            *counts.get_unchecked_mut(c as usize) += 1;
            *out.add(n) = c;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    // ---- Arm D: Branchless RLE2 + 64B L1 (4 XMM f0..f3) + 6 YMM (y2..y7), zero stack/memset ----
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_d_brl_rle_reg8_tier64(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let p = b.seq_to_unseq.as_ptr();
        let mut f0 = _mm_loadu_si128(p as *const __m128i);
        let mut f1 = _mm_loadu_si128(p.add(16) as *const __m128i);
        let mut f2 = _mm_loadu_si128(p.add(32) as *const __m128i);
        let mut f3 = _mm_loadu_si128(p.add(48) as *const __m128i);
        let mut y2 = _mm256_loadu_si256(p.add(64) as *const __m256i);
        let mut y3 = _mm256_loadu_si256(p.add(96) as *const __m256i);
        let mut y4 = _mm256_loadu_si256(p.add(128) as *const __m256i);
        let mut y5 = _mm256_loadu_si256(p.add(160) as *const __m256i);
        let mut y6 = _mm256_loadu_si256(p.add(192) as *const __m256i);
        let mut y7 = _mm256_loadu_si256(p.add(224) as *const __m256i);
        let out = ll.as_mut_ptr();
        let eob = b.eob;
        let fast_lim = 65u16.min(eob);
        let safe_max = max.saturating_sub(64);
        let (mut run, mut weight) = (0u32, 1u32);
        let mut n = 0usize;
        let mut result = Ok(());
        for &s in &b.syms {
            if s < fast_lim && run <= 32 && n < safe_max {
                let is_run = s <= 1;
                let emit_run = if is_run { 0 } else { run };
                let c0 = _mm_cvtsi128_si32(f0) as u8;
                let v = _mm256_broadcastb_epi8(f0);
                _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                *counts.get_unchecked_mut(c0 as usize) += emit_run;
                n += emit_run as usize;

                let add_run = (s as u32 + 1) * weight;
                run = if is_run { run + add_run } else { 0 };
                weight = if is_run { weight << 1 } else { 1 };

                let nn = if is_run { 0usize } else { (s - 1) as usize };
                step64(&mut f0, &mut f1, &mut f2, &mut f3, nn);

                let emit_lit = (!is_run) as u32;
                let c1 = _mm_cvtsi128_si32(f0) as u8;
                *counts.get_unchecked_mut(c1 as usize) += emit_lit;
                *out.add(n) = c1;
                n += emit_lit as usize;
                continue;
            }
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
            if nn < 64 {
                step64(&mut f0, &mut f1, &mut f2, &mut f3, nn);
            } else {
                let v_xmm = step_tail6_reg(
                    f3,
                    &mut y2,
                    &mut y3,
                    &mut y4,
                    &mut y5,
                    &mut y6,
                    &mut y7,
                    nn - 64,
                );
                f3 = _mm_alignr_epi8::<15>(f3, f2);
                f2 = _mm_alignr_epi8::<15>(f2, f1);
                f1 = _mm_alignr_epi8::<15>(f1, f0);
                f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), v_xmm);
            }
            let c = _mm_cvtsi128_si32(f0) as u8;
            *counts.get_unchecked_mut(c as usize) += 1;
            *out.add(n) = c;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    // ---- Arm E: Branchless RLE2 + Full 256B branchless 8xYMM (ZERO branches on s<=1, run, nn!) ----
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_e_brl_rle_reg8_branchless(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let p = b.seq_to_unseq.as_ptr();
        let mut y0 = _mm256_loadu_si256(p as *const __m256i);
        let mut y1 = _mm256_loadu_si256(p.add(32) as *const __m256i);
        let mut y2 = _mm256_loadu_si256(p.add(64) as *const __m256i);
        let mut y3 = _mm256_loadu_si256(p.add(96) as *const __m256i);
        let mut y4 = _mm256_loadu_si256(p.add(128) as *const __m256i);
        let mut y5 = _mm256_loadu_si256(p.add(160) as *const __m256i);
        let mut y6 = _mm256_loadu_si256(p.add(192) as *const __m256i);
        let mut y7 = _mm256_loadu_si256(p.add(224) as *const __m256i);
        let out = ll.as_mut_ptr();
        let eob = b.eob;
        let safe_max = max.saturating_sub(64);
        let (mut run, mut weight) = (0u32, 1u32);
        let mut n = 0usize;
        let mut result = Ok(());
        for &s in &b.syms {
            if s != eob && run <= 32 && n < safe_max {
                let is_run = s <= 1;
                let emit_run = if is_run { 0 } else { run };
                let f0 = _mm256_castsi256_si128(y0);
                let c0 = _mm_cvtsi128_si32(f0) as u8;
                let v = _mm256_broadcastb_epi8(f0);
                _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                *counts.get_unchecked_mut(c0 as usize) += emit_run;
                n += emit_run as usize;

                let add_run = (s as u32 + 1) * weight;
                run = if is_run { run + add_run } else { 0 };
                weight = if is_run { weight << 1 } else { 1 };

                let nn = if is_run {
                    0usize
                } else {
                    ((s - 1) as usize).min(255)
                };
                let v_xmm = step_reg8_all(
                    &mut y0, &mut y1, &mut y2, &mut y3, &mut y4, &mut y5, &mut y6, &mut y7, nn,
                );

                let emit_lit = (!is_run) as u32;
                let c1 = _mm_cvtsi128_si32(v_xmm) as u8;
                *counts.get_unchecked_mut(c1 as usize) += emit_lit;
                *out.add(n) = c1;
                n += emit_lit as usize;
                continue;
            }
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
                let f0 = _mm256_castsi256_si128(y0);
                let c = _mm_cvtsi128_si32(f0) as u8;
                *counts.get_unchecked_mut(c as usize) += run;
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
            let v_xmm = step_reg8_all(
                &mut y0, &mut y1, &mut y2, &mut y3, &mut y4, &mut y5, &mut y6, &mut y7, nn,
            );
            let c = _mm_cvtsi128_si32(v_xmm) as u8;
            *counts.get_unchecked_mut(c as usize) += 1;
            *out.add(n) = c;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    // ---- Arm F: O(1) 128-symbol per-block adaptive dispatch across in-register kernels ----
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_f_adaptive_reg8(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let syms = &b.syms;
        let len = syms.len();
        if len < 512 {
            return arm_d_brl_rle_reg8_tier64(b, ll, counts);
        }
        let stride = (len >> 7).max(1);
        let mut n_run = 0u32;
        let mut n_high64 = 0u32;
        let mut n_lit = 0u32;
        let mut i = 0usize;
        let mut samples = 0u32;
        while i < len && samples < 128 {
            let s = *syms.get_unchecked(i);
            if s <= 1 {
                n_run += 1;
            } else if s != b.eob {
                n_lit += 1;
                if s >= 65 {
                    n_high64 += 1;
                }
            }
            i += stride;
            samples += 1;
        }
        // High-entropy tail (nn >= 64 exceeds ~10% of literals): use full-branchless 8xYMM.
        if n_high64 * 10 > n_lit {
            if n_run * 16 <= samples {
                // Low-run (<6%): branched RLE2 avoids the unconditional 32B store on every literal.
                arm_c_reg8_branchless(b, ll, counts)
            } else {
                // High-run + high-entropy (e.g. ELF): branchless RLE2 + branchless 8xYMM.
                arm_e_brl_rle_reg8_branchless(b, ll, counts)
            }
        } else {
            // Low-entropy (<64 dominant, e.g. prose/pysrc/text/wikidata): branchless RLE2 + 64B L1.
            arm_d_brl_rle_reg8_tier64(b, ll, counts)
        }
    }

    // ---- Round 2 mechanisms targeting Port-5 saturation & branch mispredictions ----

    struct Moving16<const N: usize, const K: usize> {
        a: [u8; N],
        bases: [usize; K],
    }

    impl<const N: usize, const K: usize> Moving16<N, K> {
        #[inline(always)]
        fn new(bytes: &[u8]) -> Self {
            let mut m = Self {
                a: [0u8; N],
                bases: [0; K],
            };
            let start = N - 16 * K;
            m.a[start..N].copy_from_slice(&bytes[..16 * K]);
            for k in 0..K {
                m.bases[k] = start + 16 * k;
            }
            m
        }

        #[inline(always)]
        unsafe fn take(&mut self, lno: usize, off: usize, x: u8) -> u8 {
            let a = self.a.as_mut_ptr();
            let pp = *self.bases.get_unchecked(lno);
            let row = a.add(pp);
            let v = *row.add(off);
            // Shift bytes 0..off right by 1 inside chunk `lno` with 1 single pshufb!
            let cur = _mm_loadu_si128(row as *const __m128i);
            let shuf =
                _mm_load_si128(STEPS32.as_ptr().add(off).read().a.as_ptr() as *const __m128i);
            _mm_storeu_si128(row as *mut __m128i, _mm_shuffle_epi8(cur, shuf));
            let mut j = lno;
            while j > 0 {
                let below = *self.bases.get_unchecked(j - 1);
                *a.add(*self.bases.get_unchecked(j)) = *a.add(below + 15);
                *self.bases.get_unchecked_mut(j - 1) = below - 1;
                j -= 1;
            }
            *a.add(*self.bases.get_unchecked(0)) = x;
            if *self.bases.get_unchecked(0) == 0 {
                self.repack();
            }
            v
        }

        #[cold]
        #[inline(never)]
        fn repack(&mut self) {
            let mut kk = N;
            for k in (0..K).rev() {
                let b = self.bases[k];
                kk -= 16;
                self.a.copy_within(b..b + 16, kk);
                self.bases[k] = kk;
            }
        }
    }

    // 32-byte chunk moving-base list (only 7 chunks for 32..255 or 8 chunks for 0..255!)
    #[repr(align(32))]
    struct Moving32<const N: usize, const K: usize> {
        a: [u8; N],
        bases: [usize; K],
    }

    impl<const N: usize, const K: usize> Moving32<N, K> {
        #[inline(always)]
        fn new(bytes: &[u8]) -> Self {
            let mut m = Self {
                a: [0u8; N],
                bases: [0; K],
            };
            let start = N - 32 * K;
            m.a[start..N].copy_from_slice(&bytes[..32 * K]);
            for k in 0..K {
                m.bases[k] = start + 32 * k;
            }
            m
        }

        #[inline(always)]
        unsafe fn take(&mut self, lno: usize, off: usize, x: u8, idx32: __m256i) -> u8 {
            let a = self.a.as_mut_ptr();
            let pp = *self.bases.get_unchecked(lno);
            let row = a.add(pp);
            let v = *row.add(off);
            let keep = _mm256_loadu_si256(row as *const __m256i);
            let on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(off as i8 + 1), idx32);
            _mm256_storeu_si256(row as *mut __m256i, _mm256_blendv_epi8(keep, on, m));
            let mut j = lno;
            while j > 0 {
                let below = *self.bases.get_unchecked(j - 1);
                *a.add(*self.bases.get_unchecked(j)) = *a.add(below + 31);
                *self.bases.get_unchecked_mut(j - 1) = below - 1;
                j -= 1;
            }
            *a.add(*self.bases.get_unchecked(0)) = x;
            if *self.bases.get_unchecked(0) <= 1 {
                self.repack();
            }
            v
        }

        #[cold]
        #[inline(never)]
        fn repack(&mut self) {
            let mut kk = N;
            for k in (0..K).rev() {
                let b = self.bases[k];
                kk -= 32;
                self.a.copy_within(b..b + 32, kk);
                self.bases[k] = kk;
            }
        }
    }

    // ---- Arm G: f0,f1 XMM + 14 moving-base 16-byte chunks (pshufb per chunk!) + inline AVX2 fill ----
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_g_regs_bases16(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        let mut tail = Moving16::<2048, 14>::new(&s2u[32..]);
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let t = nn - 32;
                let x = _mm_extract_epi8::<15>(f1) as u8;
                let v = tail.take(t >> 4, t & 15, x);
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

    // ---- Arm H: f0,f1 XMM + 7 moving-base 32-byte chunks + inline AVX2 fill ----
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_h_regs_bases32(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        let mut tail = Moving32::<2048, 7>::new(&s2u[32..]);
        let idx32 = _mm256_setr_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let t = nn - 32;
                let x = _mm_extract_epi8::<15>(f1) as u8;
                let v = tail.take(t >> 5, t & 31, x, idx32);
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

    // ---- Arm I: 3-tier stack second() (32 / 64 / 128 / 256) with signed vpcmpgtb + inline fill ----
    #[inline(always)]
    unsafe fn second_3tier(tail: *mut u8, t: usize, f1: __m128i) -> u8 {
        let v = *tail.add(t);
        let below0 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(f1), f1);
        let tv = _mm256_set1_epi8((((t + 1) as u8) ^ 0x80) as i8);
        let it = IDX_TAIL.0.as_ptr();
        if t < 32 {
            let r0 = _mm256_load_si256(tail as *const __m256i);
            let m0 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it as *const __m256i));
            _mm256_store_si256(
                tail as *mut __m256i,
                _mm256_blendv_epi8(r0, shift_on(r0, below0), m0),
            );
        } else if t < 96 {
            // Only 3 rows (positions 32..127): covers 96.3% of random, 89.3% of elf, 100% of prose!
            let r0 = _mm256_load_si256(tail as *const __m256i);
            let r1 = _mm256_load_si256(tail.add(32) as *const __m256i);
            let r2 = _mm256_load_si256(tail.add(64) as *const __m256i);
            let m1 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(1) as *const __m256i));
            let m2 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(2) as *const __m256i));
            // Row 0 shifts unconditionally since t >= 32!
            _mm256_store_si256(tail as *mut __m256i, shift_on(r0, below0));
            _mm256_store_si256(
                tail.add(32) as *mut __m256i,
                _mm256_blendv_epi8(r1, shift_on(r1, r0), m1),
            );
            _mm256_store_si256(
                tail.add(64) as *mut __m256i,
                _mm256_blendv_epi8(r2, shift_on(r2, r1), m2),
            );
        } else {
            let r0 = _mm256_load_si256(tail as *const __m256i);
            let r1 = _mm256_load_si256(tail.add(32) as *const __m256i);
            let r2 = _mm256_load_si256(tail.add(64) as *const __m256i);
            let r3 = _mm256_load_si256(tail.add(96) as *const __m256i);
            let r4 = _mm256_load_si256(tail.add(128) as *const __m256i);
            let r5 = _mm256_load_si256(tail.add(160) as *const __m256i);
            let r6 = _mm256_load_si256(tail.add(192) as *const __m256i);
            let m3 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(3) as *const __m256i));
            let m4 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(4) as *const __m256i));
            let m5 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(5) as *const __m256i));
            let m6 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(6) as *const __m256i));
            // Rows 0, 1, 2 shift unconditionally since t >= 96!
            _mm256_store_si256(tail as *mut __m256i, shift_on(r0, below0));
            _mm256_store_si256(tail.add(32) as *mut __m256i, shift_on(r1, r0));
            _mm256_store_si256(tail.add(64) as *mut __m256i, shift_on(r2, r1));
            _mm256_store_si256(
                tail.add(96) as *mut __m256i,
                _mm256_blendv_epi8(r3, shift_on(r3, r2), m3),
            );
            _mm256_store_si256(
                tail.add(128) as *mut __m256i,
                _mm256_blendv_epi8(r4, shift_on(r4, r3), m4),
            );
            _mm256_store_si256(
                tail.add(160) as *mut __m256i,
                _mm256_blendv_epi8(r5, shift_on(r5, r4), m5),
            );
            _mm256_store_si256(
                tail.add(192) as *mut __m256i,
                _mm256_blendv_epi8(r6, shift_on(r6, r5), m6),
            );
        }
        v
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_i_tier32_64_128(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        #[repr(align(64))]
        struct Rows([u8; 224]);
        let mut rows = Rows([0u8; 224]);
        rows.0.copy_from_slice(&s2u[32..]);
        let tail = rows.0.as_mut_ptr();
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let v = second_3tier(tail, nn - 32, f1);
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

    // ---- Arm J: Full 16 moving-base 16-byte chunks (0..255), pshufb per chunk, ZERO nn<32 branch! ----
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_j_bases16_full(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let mut list = Moving16::<4096, 16>::new(&b.seq_to_unseq);
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
                let c = *list.a.get_unchecked(*list.bases.get_unchecked(0));
                *counts.get_unchecked_mut(c as usize) += run;
                fill_run_avx2(out, n, r, max, c, _mm256_set1_epi8(c as i8));
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
            let v = list.take(nn >> 4, nn & 15, 0);
            *list.a.get_unchecked_mut(*list.bases.get_unchecked(0)) = v;
            *counts.get_unchecked_mut(v as usize) += 1;
            *out.add(n) = v;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    // ---- Arm K: Arm G (f0,f1 + 14 moving-base 16B chunks) + unconditional run flush on literals (no `if run != 0` branch!) ----
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_k_bases16_uncond_flush(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        let mut tail = Moving16::<2048, 14>::new(&s2u[32..]);
        let out = ll.as_mut_ptr();
        let eob = b.eob;
        let safe_max = max.saturating_sub(64);
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
            if run <= 32 && n < safe_max {
                let c0 = _mm_cvtsi128_si32(f0) as u8;
                *counts.get_unchecked_mut(c0 as usize) += run;
                _mm256_storeu_si256(out.add(n) as *mut __m256i, _mm256_broadcastb_epi8(f0));
                n += run as usize;
                (run, weight) = (0, 1);
            } else if run != 0 {
                let r = run as usize;
                if n + r > max {
                    result = Err(ErrorKind::BlockTooLong);
                    break;
                }
                let c = _mm_cvtsi128_si32(f0) as u8;
                *counts.get_unchecked_mut(c as usize) += run;
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let t = nn - 32;
                let x = _mm_extract_epi8::<15>(f1) as u8;
                let v = tail.take(t >> 4, t & 15, x);
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

    // ---- Round 3: Exact tail shift architectures (unaligned-32B, aligned-XMM14, aligned-YMM7-by-k, rep-movsb/ptr::copy) ----

    // Arm L: f0,f1 XMM + 7 YMM rows shifted top-down via unaligned vmovdqu(row - 1) (ZERO vperm2i128/vpalignr!)
    #[inline(always)]
    unsafe fn second_unaligned_7(tail: *mut u8, t: usize, f1: __m128i, idx32: __m256i) -> u8 {
        let v = *tail.add(t);
        let x = _mm_extract_epi8::<15>(f1) as u8;
        let k = t >> 5;
        let top = tail.add(32 * k);
        let m = _mm256_cmpgt_epi8(_mm256_set1_epi8((t & 31) as i8 + 1), idx32);
        let keep = _mm256_load_si256(top as *const __m256i);
        let on = _mm256_loadu_si256(top.sub(1) as *const __m256i);
        if k == 0 {
            let on = _mm256_insert_epi8::<0>(on, x as i8);
            _mm256_store_si256(top as *mut __m256i, _mm256_blendv_epi8(keep, on, m));
            return v;
        }
        _mm256_store_si256(top as *mut __m256i, _mm256_blendv_epi8(keep, on, m));
        let mut j = k - 1;
        while j > 0 {
            let row = tail.add(32 * j);
            let on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
            _mm256_store_si256(row as *mut __m256i, on);
            j -= 1;
        }
        let on =
            _mm256_insert_epi8::<0>(_mm256_loadu_si256(tail.sub(1) as *const __m256i), x as i8);
        _mm256_store_si256(tail as *mut __m256i, on);
        v
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_l_unaligned_7(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        #[repr(align(512))]
        struct Paged([u8; 288]);
        let mut store = Paged([0u8; 288]);
        store.0[32..256].copy_from_slice(&s2u[32..]);
        let tail = store.0.as_mut_ptr().add(32);
        let idx32 = _mm256_setr_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let v = second_unaligned_7(tail, nn - 32, f1, idx32);
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

    // Arm M: f0,f1 XMM + 14 ALIGNED 16-byte XMM rows (ZERO vperm2i128, ZERO cross-store forwarding!)
    #[inline(always)]
    unsafe fn second_aligned_xmm14(tail: *mut u8, t: usize, f1: __m128i) -> u8 {
        let v = *tail.add(t);
        let k = t >> 4; // 0..14
        let off = t & 15;
        // Shift top row k partially using STEPS32[16 + off].m (which has 0xFF at 0..=off!)
        let m = _mm_load_si128(STEPS32.as_ptr().add(16 + off).read().m.as_ptr() as *const __m128i);
        let top = tail.add(16 * k);
        let r_top = _mm_load_si128(top as *const __m128i);
        let below_top = if k == 0 {
            f1
        } else {
            _mm_load_si128(top.sub(16) as *const __m128i)
        };
        _mm_store_si128(
            top as *mut __m128i,
            _mm_blendv_epi8(r_top, _mm_alignr_epi8::<15>(r_top, below_top), m),
        );
        if k > 0 {
            let mut cur = below_top;
            let mut j = k - 1;
            while j > 0 {
                let prev = _mm_load_si128(tail.add(16 * (j - 1)) as *const __m128i);
                _mm_store_si128(
                    tail.add(16 * j) as *mut __m128i,
                    _mm_alignr_epi8::<15>(cur, prev),
                );
                cur = prev;
                j -= 1;
            }
            _mm_store_si128(tail as *mut __m128i, _mm_alignr_epi8::<15>(cur, f1));
        }
        v
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_m_aligned_xmm14(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        #[repr(align(64))]
        struct Rows([u8; 224]);
        let mut rows = Rows([0u8; 224]);
        rows.0.copy_from_slice(&s2u[32..]);
        let tail = rows.0.as_mut_ptr();
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let v = second_aligned_xmm14(tail, nn - 32, f1);
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

    // Arm N: f0,f1 XMM + 7 ALIGNED 32-byte YMM rows, looping only 0..=k (ZERO vpblendvb on rows 0..k!)
    #[inline(always)]
    unsafe fn second_aligned_ymm7_k(tail: *mut u8, t: usize, f1: __m128i, idx32: __m256i) -> u8 {
        let v = *tail.add(t);
        let below0 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(f1), f1);
        let k = t >> 5; // 0..7
        let off = t & 31;
        let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(off as i8 + 1), idx32);
        let top = tail.add(32 * k);
        let r_top = _mm256_load_si256(top as *const __m256i);
        let below_top = if k == 0 {
            below0
        } else {
            _mm256_load_si256(top.sub(32) as *const __m256i)
        };
        _mm256_store_si256(
            top as *mut __m256i,
            _mm256_blendv_epi8(r_top, shift_on(r_top, below_top), m),
        );
        if k > 0 {
            let mut cur = below_top;
            let mut j = k - 1;
            while j > 0 {
                let prev = _mm256_load_si256(tail.add(32 * (j - 1)) as *const __m256i);
                _mm256_store_si256(tail.add(32 * j) as *mut __m256i, shift_on(cur, prev));
                cur = prev;
                j -= 1;
            }
            _mm256_store_si256(tail as *mut __m256i, shift_on(cur, below0));
        }
        v
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_n_aligned_ymm7_k(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        #[repr(align(64))]
        struct Rows([u8; 224]);
        let mut rows = Rows([0u8; 224]);
        rows.0.copy_from_slice(&s2u[32..]);
        let tail = rows.0.as_mut_ptr();
        let idx32 = _mm256_setr_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let v = second_aligned_ymm7_k(tail, nn - 32, f1, idx32);
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

    // Arm O: Pure unaligned-YMM across ALL 256 bytes (ZERO nn<32 branch, ZERO vperm2i128/vpalignr, PagedList!)
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_o_unaligned_full(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        #[repr(align(512))]
        struct Paged([u8; 320]);
        let mut store = Paged([0u8; 320]);
        store.0[32..288].copy_from_slice(&b.seq_to_unseq);
        let base = store.0.as_mut_ptr().add(32);
        let idx32 = _mm256_setr_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
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
                fill_run_avx2(out, n, r, max, c, _mm256_set1_epi8(c as i8));
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
            let v = *base.add(nn);
            let k = nn >> 5;
            let top = base.add(32 * k);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8((nn & 31) as i8 + 1), idx32);
            let keep = _mm256_load_si256(top as *const __m256i);
            let on = _mm256_loadu_si256(top.sub(1) as *const __m256i);
            _mm256_store_si256(top as *mut __m256i, _mm256_blendv_epi8(keep, on, m));
            let mut j = k;
            while j > 0 {
                j -= 1;
                let row = base.add(32 * j);
                let on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
                _mm256_store_si256(row as *mut __m256i, on);
            }
            *base = v;
            *counts.get_unchecked_mut(v as usize) += 1;
            *out.add(n) = v;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    // Arm P: Flat branchless 8-row aligned-store + unaligned-load (ZERO branches on nn, ZERO vperm2i128/vpalignr!)
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_p_flat_unaligned_8(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        #[repr(align(512))]
        struct Paged([u8; 320]);
        let mut store = Paged([0u8; 320]);
        store.0[32..288].copy_from_slice(&b.seq_to_unseq);
        let base = store.0.as_mut_ptr().add(32);
        let ip = IDX_PREV.0.as_ptr();
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
                fill_run_avx2(out, n, r, max, c, _mm256_set1_epi8(c as i8));
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
            let v = *base.add(nn);
            let nv = _mm256_set1_epi8(((nn as u8) ^ 0x80) as i8);
            // Load top-down so shifted unaligned reads see pre-update bytes!
            for j in (0..8).rev() {
                let row = base.add(32 * j);
                let keep = _mm256_load_si256(row as *const __m256i);
                let on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
                let m = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(j) as *const __m256i));
                _mm256_store_si256(row as *mut __m256i, _mm256_blendv_epi8(keep, on, m));
            }
            *base = v;
            *counts.get_unchecked_mut(v as usize) += 1;
            *out.add(n) = v;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    // Arm Q: Signed-vpcmpgtb flat 7-row (NO t<32 branch!) + inline AVX2 fill + f0,f1 XMM
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_q_signed_flat7(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        #[repr(align(64))]
        struct Rows([u8; 224]);
        let mut rows = Rows([0u8; 224]);
        rows.0.copy_from_slice(&s2u[32..]);
        let tail = rows.0.as_mut_ptr();
        let it = IDX_TAIL.0.as_ptr();
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let t = nn - 32;
                let v = *tail.add(t);
                let below0 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(f1), f1);
                let tv = _mm256_set1_epi8((((t + 1) as u8) ^ 0x80) as i8);
                let r0 = _mm256_load_si256(tail as *const __m256i);
                let r1 = _mm256_load_si256(tail.add(32) as *const __m256i);
                let r2 = _mm256_load_si256(tail.add(64) as *const __m256i);
                let r3 = _mm256_load_si256(tail.add(96) as *const __m256i);
                let r4 = _mm256_load_si256(tail.add(128) as *const __m256i);
                let r5 = _mm256_load_si256(tail.add(160) as *const __m256i);
                let r6 = _mm256_load_si256(tail.add(192) as *const __m256i);
                let m0 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it as *const __m256i));
                let m1 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(1) as *const __m256i));
                let m2 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(2) as *const __m256i));
                let m3 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(3) as *const __m256i));
                let m4 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(4) as *const __m256i));
                let m5 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(5) as *const __m256i));
                let m6 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(6) as *const __m256i));
                _mm256_store_si256(
                    tail as *mut __m256i,
                    _mm256_blendv_epi8(r0, shift_on(r0, below0), m0),
                );
                _mm256_store_si256(
                    tail.add(32) as *mut __m256i,
                    _mm256_blendv_epi8(r1, shift_on(r1, r0), m1),
                );
                _mm256_store_si256(
                    tail.add(64) as *mut __m256i,
                    _mm256_blendv_epi8(r2, shift_on(r2, r1), m2),
                );
                _mm256_store_si256(
                    tail.add(96) as *mut __m256i,
                    _mm256_blendv_epi8(r3, shift_on(r3, r2), m3),
                );
                _mm256_store_si256(
                    tail.add(128) as *mut __m256i,
                    _mm256_blendv_epi8(r4, shift_on(r4, r3), m4),
                );
                _mm256_store_si256(
                    tail.add(160) as *mut __m256i,
                    _mm256_blendv_epi8(r5, shift_on(r5, r4), m5),
                );
                _mm256_store_si256(
                    tail.add(192) as *mut __m256i,
                    _mm256_blendv_epi8(r6, shift_on(r6, r5), m6),
                );
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

    // ---- Round 4: JCC-clean showdown of top mechanisms + partial-store elimination + adaptive ----

    // Arm P2: Flat 8-row unaligned-load + aligned-store with ZERO partial `*base = v` store!
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_p2_flat_unal8_nopartial(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        #[repr(align(512))]
        struct Paged([u8; 320]);
        let mut store = Paged([0u8; 320]);
        store.0[32..288].copy_from_slice(&b.seq_to_unseq);
        let base = store.0.as_mut_ptr().add(32);
        let ip = IDX_PREV.0.as_ptr();
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
                fill_run_avx2(out, n, r, max, c, _mm256_set1_epi8(c as i8));
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
            let v = *base.add(nn);
            let nv = _mm256_set1_epi8(((nn as u8) ^ 0x80) as i8);
            for j in (1..8).rev() {
                let row = base.add(32 * j);
                let keep = _mm256_load_si256(row as *const __m256i);
                let on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
                let m = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(j) as *const __m256i));
                _mm256_store_si256(row as *mut __m256i, _mm256_blendv_epi8(keep, on, m));
            }
            // Row 0: insert v in register so base[0..32] gets ONE clean 32B aligned store!
            let keep0 = _mm256_load_si256(base as *const __m256i);
            let on0 = _mm256_loadu_si256(base.sub(1) as *const __m256i);
            let m0 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip as *const __m256i));
            let new0 = _mm256_blendv_epi8(keep0, on0, m0);
            let new0 = _mm256_or_si256(
                _mm256_and_si256(new0, clear_b0),
                _mm256_zextsi128_si256(_mm_cvtsi32_si128(v as i32)),
            );
            _mm256_store_si256(base as *mut __m256i, new0);
            *counts.get_unchecked_mut(v as usize) += 1;
            *out.add(n) = v;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    // Arm Q2: Flat 8-row ALIGNED load/store + shift_on (ZERO branches on nn, ZERO unaligned loads, ZERO partial store!)
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_q2_flat_aligned8(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        #[repr(align(64))]
        struct Rows([u8; 256]);
        let mut rows = Rows([0u8; 256]);
        rows.0.copy_from_slice(&b.seq_to_unseq);
        let base = rows.0.as_mut_ptr();
        let ip = IDX_PREV.0.as_ptr();
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
                fill_run_avx2(out, n, r, max, c, _mm256_set1_epi8(c as i8));
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
            let v = *base.add(nn);
            let nv = _mm256_set1_epi8(((nn as u8) ^ 0x80) as i8);
            let r0 = _mm256_load_si256(base as *const __m256i);
            let r1 = _mm256_load_si256(base.add(32) as *const __m256i);
            let r2 = _mm256_load_si256(base.add(64) as *const __m256i);
            let r3 = _mm256_load_si256(base.add(96) as *const __m256i);
            let r4 = _mm256_load_si256(base.add(128) as *const __m256i);
            let r5 = _mm256_load_si256(base.add(160) as *const __m256i);
            let r6 = _mm256_load_si256(base.add(192) as *const __m256i);
            let r7 = _mm256_load_si256(base.add(224) as *const __m256i);
            let m0 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip as *const __m256i));
            let m1 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(1) as *const __m256i));
            let m2 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(2) as *const __m256i));
            let m3 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(3) as *const __m256i));
            let m4 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(4) as *const __m256i));
            let m5 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(5) as *const __m256i));
            let m6 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(6) as *const __m256i));
            let m7 = _mm256_cmpgt_epi8(nv, _mm256_load_si256(ip.add(7) as *const __m256i));
            let new0 = _mm256_or_si256(
                _mm256_blendv_epi8(r0, shift_y0(r0), m0),
                _mm256_zextsi128_si256(_mm_cvtsi32_si128(v as i32)),
            );
            _mm256_store_si256(base as *mut __m256i, new0);
            _mm256_store_si256(
                base.add(32) as *mut __m256i,
                _mm256_blendv_epi8(r1, shift_on(r1, r0), m1),
            );
            _mm256_store_si256(
                base.add(64) as *mut __m256i,
                _mm256_blendv_epi8(r2, shift_on(r2, r1), m2),
            );
            _mm256_store_si256(
                base.add(96) as *mut __m256i,
                _mm256_blendv_epi8(r3, shift_on(r3, r2), m3),
            );
            _mm256_store_si256(
                base.add(128) as *mut __m256i,
                _mm256_blendv_epi8(r4, shift_on(r4, r3), m4),
            );
            _mm256_store_si256(
                base.add(160) as *mut __m256i,
                _mm256_blendv_epi8(r5, shift_on(r5, r4), m5),
            );
            _mm256_store_si256(
                base.add(192) as *mut __m256i,
                _mm256_blendv_epi8(r6, shift_on(r6, r5), m6),
            );
            _mm256_store_si256(
                base.add(224) as *mut __m256i,
                _mm256_blendv_epi8(r7, shift_on(r7, r6), m7),
            );
            *counts.get_unchecked_mut(v as usize) += 1;
            *out.add(n) = v;
            n += 1;
        }
        ll.set_len(n);
        result.map(|()| n)
    }

    // Arm R: f0,f1 XMM (nn < 32) + flat 7-row unaligned load / aligned store for nn >= 32 (NO t<32 branch, NO vperm2i128/vpalignr!)
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_r_tier32_flat_unal7(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        #[repr(align(512))]
        struct Paged([u8; 288]);
        let mut store = Paged([0u8; 288]);
        store.0[32..256].copy_from_slice(&s2u[32..]);
        let tail = store.0.as_mut_ptr().add(32);
        let it = IDX_TAIL.0.as_ptr();
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
                fill_run_avx2(out, n, r, max, c, _mm256_broadcastb_epi8(f0));
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
                step32(&mut f0, &mut f1, nn);
            } else {
                let t = nn - 32;
                let v = *tail.add(t);
                let x = _mm_extract_epi8::<15>(f1) as i32;
                let tv = _mm256_set1_epi8((((t + 1) as u8) ^ 0x80) as i8);
                for j in (1..7).rev() {
                    let row = tail.add(32 * j);
                    let keep = _mm256_load_si256(row as *const __m256i);
                    let on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
                    let m = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it.add(j) as *const __m256i));
                    _mm256_store_si256(row as *mut __m256i, _mm256_blendv_epi8(keep, on, m));
                }
                let keep0 = _mm256_load_si256(tail as *const __m256i);
                let on0 = _mm256_loadu_si256(tail.sub(1) as *const __m256i);
                let m0 = _mm256_cmpgt_epi8(tv, _mm256_load_si256(it as *const __m256i));
                let new0 = _mm256_blendv_epi8(keep0, on0, m0);
                let new0 = _mm256_or_si256(
                    _mm256_and_si256(new0, clear_b0),
                    _mm256_zextsi128_si256(_mm_cvtsi32_si128(x)),
                );
                _mm256_store_si256(tail as *mut __m256i, new0);
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

    // Arm S: O(1) 128-sym per-block adaptive dispatch between Q_signed_flat7 (<=40% nn>=32) and P2_flat_unal8 (>40% nn>=32)
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_s_adaptive_q_p2(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let syms = &b.syms;
        let len = syms.len();
        if len < 512 {
            return arm_q_signed_flat7(b, ll, counts);
        }
        let stride = (len >> 7).max(1);
        let mut n_high32 = 0u32;
        let mut n_lit = 0u32;
        let mut i = 0usize;
        let mut samples = 0u32;
        while i < len && samples < 128 {
            let s = *syms.get_unchecked(i);
            if s >= 2 && s != b.eob {
                n_lit += 1;
                if s >= 33 {
                    n_high32 += 1;
                }
            }
            i += stride;
            samples += 1;
        }
        if n_high32 * 5 > n_lit * 2 {
            // >40% literals >= 32 (random, records, high-entropy binary): zero-branch flat 8-row!
            arm_p2_flat_unal8_nopartial(b, ll, counts)
        } else {
            // <=40% literals >= 32 (prose, source, elf, text, wikidata): f0/f1 XMM + signed flat7!
            arm_q_signed_flat7(b, ll, counts)
        }
    }

    // Arm T: Same adaptive dispatch using Q2_flat_aligned8 (zero unaligned loads!) vs Q_signed_flat7
    #[target_feature(enable = "avx2")]
    pub unsafe fn arm_t_adaptive_q_q2(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let syms = &b.syms;
        let len = syms.len();
        if len < 512 {
            return arm_q_signed_flat7(b, ll, counts);
        }
        let stride = (len >> 7).max(1);
        let mut n_high32 = 0u32;
        let mut n_lit = 0u32;
        let mut i = 0usize;
        let mut samples = 0u32;
        while i < len && samples < 128 {
            let s = *syms.get_unchecked(i);
            if s >= 2 && s != b.eob {
                n_lit += 1;
                if s >= 33 {
                    n_high32 += 1;
                }
            }
            i += stride;
            samples += 1;
        }
        if n_high32 * 5 > n_lit * 2 {
            arm_q2_flat_aligned8(b, ll, counts)
        } else {
            arm_q_signed_flat7(b, ll, counts)
        }
    }
}

type ArmFn = fn(&Symbols, &mut Vec<u8>, &mut [u32; 256], Kernels) -> Result<usize, ErrorKind>;

fn arm_shipped(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    k: Kernels,
) -> Result<usize, ErrorKind> {
    mtf::mtf(b, ll, counts, k)
}

#[cfg(target_arch = "x86_64")]
fn arms() -> Vec<(&'static str, ArmFn)> {
    vec![
        ("shipped", arm_shipped),
        ("A_brl_rle_tier32", |b, ll, c, _| unsafe {
            kernels::arm_a_brl_rle_tier32(b, ll, c)
        }),
        ("B_reg8_tier32", |b, ll, c, _| unsafe {
            kernels::arm_b_reg8_tier32(b, ll, c)
        }),
        ("C_reg8_branchless", |b, ll, c, _| unsafe {
            kernels::arm_c_reg8_branchless(b, ll, c)
        }),
        ("D_brl_rle_reg8_t64", |b, ll, c, _| unsafe {
            kernels::arm_d_brl_rle_reg8_tier64(b, ll, c)
        }),
        ("E_brl_rle_reg8_bl", |b, ll, c, _| unsafe {
            kernels::arm_e_brl_rle_reg8_branchless(b, ll, c)
        }),
        ("F_adaptive_reg8", |b, ll, c, _| unsafe {
            kernels::arm_f_adaptive_reg8(b, ll, c)
        }),
    ]
}

#[cfg(not(target_arch = "x86_64"))]
fn arms() -> Vec<(&'static str, ArmFn)> {
    vec![("shipped", arm_shipped)]
}

#[cfg(target_arch = "x86_64")]
fn arms_r2() -> Vec<(&'static str, ArmFn)> {
    vec![
        ("shipped", arm_shipped),
        ("G_regs_bases16", |b, ll, c, _| unsafe {
            kernels::arm_g_regs_bases16(b, ll, c)
        }),
        ("H_regs_bases32", |b, ll, c, _| unsafe {
            kernels::arm_h_regs_bases32(b, ll, c)
        }),
        ("I_tier32_64_128", |b, ll, c, _| unsafe {
            kernels::arm_i_tier32_64_128(b, ll, c)
        }),
        ("J_bases16_full", |b, ll, c, _| unsafe {
            kernels::arm_j_bases16_full(b, ll, c)
        }),
        ("K_b16_uncond_fl", |b, ll, c, _| unsafe {
            kernels::arm_k_bases16_uncond_flush(b, ll, c)
        }),
    ]
}

#[cfg(not(target_arch = "x86_64"))]
fn arms_r2() -> Vec<(&'static str, ArmFn)> {
    vec![("shipped", arm_shipped)]
}

#[cfg(target_arch = "x86_64")]
fn arms_r3() -> Vec<(&'static str, ArmFn)> {
    vec![
        ("shipped", arm_shipped),
        ("L_unaligned_7", |b, ll, c, _| unsafe {
            kernels::arm_l_unaligned_7(b, ll, c)
        }),
        ("M_aligned_xmm14", |b, ll, c, _| unsafe {
            kernels::arm_m_aligned_xmm14(b, ll, c)
        }),
        ("N_aligned_ymm7_k", |b, ll, c, _| unsafe {
            kernels::arm_n_aligned_ymm7_k(b, ll, c)
        }),
        ("O_unaligned_full", |b, ll, c, _| unsafe {
            kernels::arm_o_unaligned_full(b, ll, c)
        }),
        ("P_flat_unal_8", |b, ll, c, _| unsafe {
            kernels::arm_p_flat_unaligned_8(b, ll, c)
        }),
        ("Q_signed_flat7", |b, ll, c, _| unsafe {
            kernels::arm_q_signed_flat7(b, ll, c)
        }),
    ]
}

#[cfg(not(target_arch = "x86_64"))]
fn arms_r3() -> Vec<(&'static str, ArmFn)> {
    vec![("shipped", arm_shipped)]
}

#[cfg(target_arch = "x86_64")]
fn arms_r4() -> Vec<(&'static str, ArmFn)> {
    vec![
        ("shipped", arm_shipped),
        ("I_tier32_64_128", |b, ll, c, _| unsafe {
            kernels::arm_i_tier32_64_128(b, ll, c)
        }),
        ("Q_signed_flat7", |b, ll, c, _| unsafe {
            kernels::arm_q_signed_flat7(b, ll, c)
        }),
        ("P2_flat_unal8_np", |b, ll, c, _| unsafe {
            kernels::arm_p2_flat_unal8_nopartial(b, ll, c)
        }),
        ("Q2_flat_algn8", |b, ll, c, _| unsafe {
            kernels::arm_q2_flat_aligned8(b, ll, c)
        }),
        ("R_t32_flat_unal7", |b, ll, c, _| unsafe {
            kernels::arm_r_tier32_flat_unal7(b, ll, c)
        }),
        ("S_adapt_Q_P2", |b, ll, c, _| unsafe {
            kernels::arm_s_adaptive_q_p2(b, ll, c)
        }),
        ("T_adapt_Q_Q2", |b, ll, c, _| unsafe {
            kernels::arm_t_adaptive_q_q2(b, ll, c)
        }),
    ]
}

#[cfg(not(target_arch = "x86_64"))]
fn arms_r4() -> Vec<(&'static str, ArmFn)> {
    vec![("shipped", arm_shipped)]
}

fn synthetic_edge_cases() -> Vec<(&'static str, Symbols)> {
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
    // 1. Empty block (immediate EOB)
    cases.push(("empty_eob", mk(vec![2], 2, 100)));
    // 2. Tiny alphabet (eob < 33) with runs and literals
    cases.push((
        "tiny_eob_5",
        mk(vec![0, 1, 2, 3, 4, 0, 0, 0, 2, 1, 5], 5, 100),
    ));
    // 3. Long run > 64 + near max_len boundary (last 64 bytes)
    let mut syms_long = vec![0, 0, 0, 0, 0, 0, 0, 2]; // run = 127, then literal 2
    for i in 0..70 {
        syms_long.push(((i % 250) + 2) as u16);
    }
    syms_long.push(257);
    cases.push(("long_run_and_tail", mk(syms_long, 257, 200)));
    // 4. Exact max_len hit
    cases.push(("exact_max_len", mk(vec![0, 0, 2, 3, 4, 257], 257, 6)));
    // 5. BlockTooLong via run
    cases.push(("err_block_too_long_run", mk(vec![0, 0, 0, 2, 257], 257, 5)));
    // 6. BlockTooLong via literal
    cases.push((
        "err_block_too_long_lit",
        mk(vec![2, 3, 4, 5, 6, 7, 257], 257, 5),
    ));
    // 7. RunTooLong (22 consecutive RUNA -> weight reaches 2^21)
    let mut syms_rtl = vec![0u16; 22];
    syms_rtl.push(257);
    cases.push(("err_run_too_long", mk(syms_rtl, 257, 900_000)));
    // 8. Trailing run with NO EOB (malformed public Symbols)
    cases.push(("trailing_run_no_eob", mk(vec![2, 3, 0, 1, 0], 257, 100)));
    // 9. Boundary nn = 0..255 sweep (s = 2..=256)
    let mut syms_sweep = Vec::new();
    for s in 2u16..=256 {
        syms_sweep.push(s);
    }
    for s in (2u16..=256).rev() {
        syms_sweep.push(s);
    }
    syms_sweep.push(0);
    syms_sweep.push(257);
    cases.push(("full_nn_sweep_255", mk(syms_sweep, 257, 1000)));
    // 10. Symbol > 256 clamping (public Symbols guard)
    cases.push(("clamp_over_256", mk(vec![2, 256, 258, 300, 257], 257, 100)));
    cases
}

fn verify_exact_list(all_blocks: &[CorpusBlock], k: Kernels, arm_list: &[(&'static str, ArmFn)]) {
    let mut ll = Vec::with_capacity(900_000);
    let mut counts = [0u32; 256];
    let mut total_checks = 0usize;

    // 1. All corpus blocks (both Column and Runs/Open blocks forced through mtf!)
    for (name, f) in arm_list {
        for (bi, blk) in all_blocks.iter().enumerate() {
            let res = f(&blk.syms, &mut ll, &mut counts, k);
            let n = res.unwrap_or_else(|e| {
                panic!("arm {name} failed on {} blk {bi}: {e:?}", blk.corpus);
            });
            assert_eq!(
                n, blk.want_n,
                "arm {name} len mismatch on {} blk {bi}",
                blk.corpus
            );
            assert_eq!(
                ll, blk.want_ll,
                "arm {name} ll mismatch on {} blk {bi}",
                blk.corpus
            );
            assert_eq!(
                counts, blk.want_counts,
                "arm {name} counts mismatch on {} blk {bi}",
                blk.corpus
            );
            total_checks += 1;
        }
    }

    // 2. Synthetic edge & error cases (compared against shipped mtf::mtf / mtf_scalar)
    let synth = synthetic_edge_cases();
    let mut want_ll = Vec::new();
    let mut want_counts = [0u32; 256];
    for (case_name, syms) in &synth {
        let want_res = mtf::mtf(syms, &mut want_ll, &mut want_counts, k);
        for (arm_name, f) in arm_list {
            let got_res = f(syms, &mut ll, &mut counts, k);
            assert_eq!(
                got_res, want_res,
                "arm {arm_name} result mismatch on synthetic {case_name}"
            );
            if want_res.is_ok() {
                assert_eq!(
                    ll, want_ll,
                    "arm {arm_name} ll mismatch on synthetic {case_name}"
                );
                assert_eq!(
                    counts, want_counts,
                    "arm {arm_name} counts mismatch on synthetic {case_name}"
                );
            }
            total_checks += 1;
        }
    }

    println!(
        "EXACTNESS PASS: {total_checks} / {total_checks} checks ({} blocks + {} synthetic cases x {} arms)",
        all_blocks.len(),
        synth.len(),
        arm_list.len()
    );
}

fn median_min_max(mut v: Vec<f64>) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.total_cmp(b));
    let med = v[v.len() / 2];
    (med, v[0], v[v.len() - 1])
}

fn run_bench(paths: &[String], repeats: usize, arm_list: Vec<(&'static str, ArmFn)>) {
    let k = Kernels::detect();
    let mut all_blocks = Vec::new();
    let mut corpus_order = Vec::new();
    for p in paths {
        let bs = load_corpus(Path::new(p));
        if let Some(first) = bs.first() {
            corpus_order.push(first.corpus.clone());
        }
        all_blocks.extend(bs);
    }

    verify_exact_list(&all_blocks, k, &arm_list);
    let n_arms = arm_list.len();
    let n_corp = corpus_order.len();

    // Index blocks by corpus: Column blocks for Column corpora, all blocks for guard corpora (text, wikidata, json, rustsrc)
    let mut corp_col_indices: Vec<Vec<usize>> = vec![Vec::new(); n_corp];
    let mut corp_all_indices: Vec<Vec<usize>> = vec![Vec::new(); n_corp];
    for (bi, blk) in all_blocks.iter().enumerate() {
        let ci = corpus_order.iter().position(|c| c == &blk.corpus).unwrap();
        corp_all_indices[ci].push(bi);
        if blk.is_column {
            corp_col_indices[ci].push(bi);
        }
    }

    // Store per-(arm, repeat, corpus) CPU and wall ns for Column blocks + guard corpora
    let mut col_cpu = vec![vec![vec![0.0f64; n_corp]; repeats]; n_arms];
    let mut col_wall = vec![vec![vec![0.0f64; n_corp]; repeats]; n_arms];
    let mut guard_cpu = vec![vec![vec![0.0f64; n_corp]; repeats]; n_arms];

    let mut ll = Vec::with_capacity(900_000);
    let mut counts = [0u32; 256];

    // Warm-up pass
    for &(_, f) in &arm_list {
        for blk in &all_blocks {
            if blk.is_column {
                let _ = f(&blk.syms, &mut ll, &mut counts, k).unwrap();
            }
        }
    }

    // 7 interleaved repeats with rotated arm order
    for r in 0..repeats {
        for step in 0..n_arms {
            let ai = (r + step) % n_arms;
            let f = arm_list[ai].1;
            for ci in 0..n_corp {
                if !corp_col_indices[ci].is_empty() && corpus_order[ci] != "wikidata" {
                    let w0 = Instant::now();
                    let c0 = thread_cpu_ns();
                    for &bi in &corp_col_indices[ci] {
                        let _ = f(&all_blocks[bi].syms, &mut ll, &mut counts, k).unwrap();
                    }
                    let dc = (thread_cpu_ns() - c0) as f64 / 1e6;
                    let dw = w0.elapsed().as_secs_f64() * 1e3;
                    col_cpu[ai][r][ci] = dc;
                    col_wall[ai][r][ci] = dw;
                }
                if corpus_order[ci] == "text"
                    || corpus_order[ci] == "wikidata"
                    || corpus_order[ci] == "json"
                    || corpus_order[ci] == "rustsrc"
                {
                    let c0 = thread_cpu_ns();
                    for &bi in &corp_all_indices[ci] {
                        let _ = f(&all_blocks[bi].syms, &mut ll, &mut counts, k).unwrap();
                    }
                    guard_cpu[ai][r][ci] = (thread_cpu_ns() - c0) as f64 / 1e6;
                }
            }
        }
    }

    let col_corpora: Vec<usize> = (0..n_corp)
        .filter(|&ci| !corp_col_indices[ci].is_empty() && corpus_order[ci] != "wikidata")
        .collect();

    println!(
        "\n=== PRIMARY COLUMN AGGREGATE ({} corpora, {} Column blocks, {} interleaved repeats) ===",
        col_corpora.len(),
        col_corpora
            .iter()
            .map(|&ci| corp_col_indices[ci].len())
            .sum::<usize>(),
        repeats
    );
    println!(
        "{:<22} {:>8} {:>19} {:>7} {:>6} {:>8} {:>19} {:>7}",
        "arm",
        "CPU_med",
        "CPU_[min..max]",
        "CPU_d%",
        "disj",
        "Wall_med",
        "Wall_[min..max]",
        "Wall_d%"
    );

    let mut ship_cpu_vec = Vec::new();
    let mut ship_wall_vec = Vec::new();
    for r in 0..repeats {
        ship_cpu_vec.push(col_corpora.iter().map(|&ci| col_cpu[0][r][ci]).sum::<f64>());
        ship_wall_vec.push(
            col_corpora
                .iter()
                .map(|&ci| col_wall[0][r][ci])
                .sum::<f64>(),
        );
    }
    let (ship_c_med, ship_c_min, ship_c_max) = median_min_max(ship_cpu_vec);
    let (ship_w_med, _, _) = median_min_max(ship_wall_vec);

    for ai in 0..n_arms {
        let mut cvec = Vec::new();
        let mut wvec = Vec::new();
        for r in 0..repeats {
            cvec.push(
                col_corpora
                    .iter()
                    .map(|&ci| col_cpu[ai][r][ci])
                    .sum::<f64>(),
            );
            wvec.push(
                col_corpora
                    .iter()
                    .map(|&ci| col_wall[ai][r][ci])
                    .sum::<f64>(),
            );
        }
        let (cm, cmin, cmax) = median_min_max(cvec);
        let (wm, wmin, wmax) = median_min_max(wvec);
        let cd = 100.0 * (cm - ship_c_med) / ship_c_med;
        let wd = 100.0 * (wm - ship_w_med) / ship_w_med;
        let disj = if ai == 0 {
            "—"
        } else if cmax < ship_c_min {
            "FASTER"
        } else if cmin > ship_c_max {
            "SLOWER"
        } else {
            "ovlp"
        };
        println!(
            "{:<22} {:>8.2} [{:>7.2}..{:>7.2}] {:>+6.2}% {:>6} {:>8.2} [{:>7.2}..{:>7.2}] {:>+6.2}%",
            arm_list[ai].0, cm, cmin, cmax, cd, disj, wm, wmin, wmax, wd
        );
    }

    println!("\n=== PER-CORPUS COLUMN BREAKDOWN (CPU median ms & delta% vs shipped) ===");
    print!("{:<22}", "arm");
    for &ci in &col_corpora {
        print!(
            " {:>14}",
            format!("{}({})", corpus_order[ci], corp_col_indices[ci].len())
        );
    }
    println!();
    let ship_per: Vec<(f64, f64, f64)> = col_corpora
        .iter()
        .map(|&ci| median_min_max((0..repeats).map(|r| col_cpu[0][r][ci]).collect()))
        .collect();
    for ai in 0..n_arms {
        print!("{:<22}", arm_list[ai].0);
        for (idx, &ci) in col_corpora.iter().enumerate() {
            let (m, lo, hi) = median_min_max((0..repeats).map(|r| col_cpu[ai][r][ci]).collect());
            if ai == 0 {
                print!(" {:>11.2} ms", m);
            } else {
                let d = 100.0 * (m - ship_per[idx].0) / ship_per[idx].0;
                let mark = if hi < ship_per[idx].1 {
                    "*"
                } else if lo > ship_per[idx].2 {
                    "!"
                } else {
                    " "
                };
                print!(" {:>+6.1}%({:>5.1}){}", d, m, mark);
            }
        }
        println!();
    }

    println!("\n=== GUARD CORPORA (forced through mtf, CPU median ms & delta% vs shipped) ===");
    let guards: Vec<usize> = (0..n_corp)
        .filter(|&ci| {
            corpus_order[ci] == "text"
                || corpus_order[ci] == "wikidata"
                || corpus_order[ci] == "json"
                || corpus_order[ci] == "rustsrc"
        })
        .collect();
    print!("{:<22}", "arm");
    for &ci in &guards {
        print!(
            " {:>16}",
            format!("{}({})", corpus_order[ci], corp_all_indices[ci].len())
        );
    }
    println!();
    let ship_g: Vec<(f64, f64, f64)> = guards
        .iter()
        .map(|&ci| median_min_max((0..repeats).map(|r| guard_cpu[0][r][ci]).collect()))
        .collect();
    for ai in 0..n_arms {
        print!("{:<22}", arm_list[ai].0);
        for (idx, &ci) in guards.iter().enumerate() {
            let (m, lo, hi) = median_min_max((0..repeats).map(|r| guard_cpu[ai][r][ci]).collect());
            if ai == 0 {
                print!(" {:>13.2} ms", m);
            } else {
                let d = 100.0 * (m - ship_g[idx].0) / ship_g[idx].0;
                let mark = if hi < ship_g[idx].1 {
                    "*"
                } else if lo > ship_g[idx].2 {
                    "!"
                } else {
                    " "
                };
                print!(" {:>+7.1}%({:>5.1}){}", d, m, mark);
            }
        }
        println!();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("Usage: mtf_column_front_probe <exact|bench> <bz2 files...>");
        std::process::exit(1);
    }
    let k = Kernels::detect();
    match args[0].as_str() {
        "exact" => {
            let mut all = Vec::new();
            for p in &args[1..] {
                all.extend(load_corpus(Path::new(p)));
            }
            verify_exact_list(&all, k, &arms());
            verify_exact_list(&all, k, &arms_r2());
            verify_exact_list(&all, k, &arms_r3());
            verify_exact_list(&all, k, &arms_r4());
        }
        "bench" => run_bench(&args[1..], 7, arms()),
        "bench2" => run_bench(&args[1..], 7, arms_r2()),
        "bench3" => run_bench(&args[1..], 7, arms_r3()),
        "bench4" => run_bench(&args[1..], 7, arms_r4()),
        other => {
            eprintln!("Unknown mode: {other}");
            std::process::exit(1);
        }
    }
}
