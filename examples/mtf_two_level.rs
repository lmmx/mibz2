//! MTF's list layouts timed (research, not a benchmark): `mtf::mtf` ("two-level": the
//! first 32 bytes in two registers, the rest in memory as 7 aligned rows moved in
//! registers) against master's kernel ("master": the whole list in memory, a 32-byte blend
//! below 32, a memmove call past it) and ablations, each with the same first level but
//! the last: "regs+memmove", master's tail (a blend below 64, else a memmove call);
//! "regs+bases", libbzip2's moving-base sub-lists of 16; "rows-v2" and "rows-flat", rows
//! moved by unaligned shifted loads (a loop on the rows below the top one, or every row
//! blended); "rows-al", the library's tail compiled here. "libbzip2" is libbzip2's whole
//! two-level list (`mtfa`, `mtfbase`) vectorised in memory. Every arm's `ll`, counts and
//! result are checked against the scalar `mtf::mtf` on every block it is timed on; the
//! skeleton (the loop with no list: its byte is seq_to_unseq[nn]) is a floor, not a
//! decoder, and is not checked.
//!
//! On a Skylake-SP VM the arms' order changed with code layout from one build to the next
//! (the JCC erratum); build with `RUSTFLAGS="-C llvm-args=-x86-branches-within-32B-boundaries"`
//! to compare them there.
//!
//! ```text
//! mtf_two_level synth [REPS]          (one-kind streams: the chain a symbol, TSC cycles)
//! mtf_two_level file FILE.bz2 ROUNDS  (every block, arms interleaved, every round)
//! ```

#[cfg(target_arch = "x86_64")]
mod arms {
    use std::arch::x86_64::*;

    use mibz2::block::Symbols;
    use mibz2::{mtf, ErrorKind, Kernels};

    const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;

    pub type Kernel =
        unsafe fn(&Symbols, &mut Vec<u8>, &mut [u32; 256]) -> Result<usize, ErrorKind>;

    /// `mtf_avx2`'s run handling and checks, around a list's step `$step` (taking nn,
    /// giving the new front byte) and its front `$front` (for a run)
    macro_rules! kernel_body {
        ($b:expr, $ll:expr, $counts:expr, |$nn:ident| $step:block, || $front:block) => {{
            let max = $b.max_len;
            $ll.clear();
            $ll.reserve(max);
            *$counts = [0; 256];
            let (mut run, mut weight) = (0u32, 1u32);
            let out = $ll.as_mut_ptr();
            let mut n = 0usize;
            let mut result = Ok(());
            for &s in &$b.syms {
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
                    if n + run as usize > max {
                        result = Err(ErrorKind::BlockTooLong);
                        break;
                    }
                    let c: u8 = $front;
                    $counts[c as usize] += run;
                    if run <= 64 && n + 64 <= max {
                        let v = _mm256_set1_epi8(c as i8);
                        _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                        _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
                    } else {
                        std::ptr::write_bytes(out.add(n), c, run as usize);
                    }
                    n += run as usize;
                    (run, weight) = (0, 1);
                }
                if s == $b.eob {
                    break;
                }
                if n >= max {
                    result = Err(ErrorKind::BlockTooLong);
                    break;
                }
                let $nn = (s - 1) as usize;
                let c: u8 = $step;
                $counts[c as usize] += 1;
                *out.add(n) = c;
                n += 1;
            }
            $ll.set_len(n);
            result.map(|()| n)
        }};
    }

    #[inline(always)]
    unsafe fn iota() -> __m256i {
        _mm256_setr_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        )
    }

    /// The bytes of a 16-byte row up to `off` (i <= off)
    #[inline(always)]
    unsafe fn le16(off: usize) -> __m128i {
        _mm_cmpgt_epi8(_mm_set1_epi8(off as i8 + 1), _mm256_castsi256_si128(iota()))
    }

    /// master's `mtf_avx2` (0769b52f), compiled here
    #[target_feature(enable = "avx2")]
    pub unsafe fn master(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        #[repr(align(64))]
        struct L([u8; 320]);
        let mut store = L([0u8; 320]);
        for i in 0..256 {
            store.0[32 + i] = b.seq_to_unseq[i];
        }
        let base = store.0.as_mut_ptr().add(32);
        let idx = iota();
        kernel_body!(
            b,
            ll,
            counts,
            |nn| {
                let v = *base.add(nn);
                if nn < 32 {
                    let old = _mm256_load_si256(base as *const __m256i);
                    let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
                    let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(nn as i8 + 1), idx);
                    _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
                } else {
                    std::ptr::copy(base, base.add(1), nn);
                }
                *base = v;
                v
            },
            || { *base }
        )
    }

    /// The first level's shuffle controls (as src/mtf.rs's `Step`)
    #[repr(C, align(64))]
    #[derive(Clone, Copy)]
    struct Step {
        a: [u8; 16],
        b: [u8; 16],
        m: [u8; 16],
        _pad: [u8; 16],
    }

    const fn steps() -> [Step; 32] {
        const Z: u8 = 0x80;
        let mut t = [Step {
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

    static STEPS: [Step; 32] = steps();

    /// The first level's move of nn < 32
    #[inline(always)]
    unsafe fn first(f0: &mut __m128i, f1: &mut __m128i, nn: usize) {
        let t = STEPS.as_ptr().add(nn);
        let a = _mm_load_si128((*t).a.as_ptr() as *const __m128i);
        let bb = _mm_load_si128((*t).b.as_ptr() as *const __m128i);
        let m = _mm_load_si128((*t).m.as_ptr() as *const __m128i);
        let g0 = _mm_or_si128(_mm_shuffle_epi8(*f0, a), _mm_shuffle_epi8(*f1, bb));
        *f1 = _mm_blendv_epi8(*f1, _mm_alignr_epi8::<15>(*f1, *f0), m);
        *f0 = g0;
    }

    /// The registers after a move from the second level: `v` in front, each byte on
    #[inline(always)]
    unsafe fn take_in(f0: &mut __m128i, f1: &mut __m128i, v: u8) {
        *f1 = _mm_alignr_epi8::<15>(*f1, *f0);
        *f0 = _mm_or_si128(_mm_slli_si128::<1>(*f0), _mm_cvtsi32_si128(v as i32));
    }

    /// The two-level first level, master's tail: index t of the tail moved as master moves
    /// its list (a blend below 32, else a memmove call), `f1`'s last in at 0
    #[target_feature(enable = "avx2")]
    pub unsafe fn regs_memmove(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        #[repr(align(64))]
        struct L([u8; 288]);
        let mut store = L([0u8; 288]);
        store.0[32..256].copy_from_slice(&s2u[32..]);
        let tail = store.0.as_mut_ptr().add(32);
        let idx = iota();
        kernel_body!(
            b,
            ll,
            counts,
            |nn| {
                if nn < 32 {
                    first(&mut f0, &mut f1, nn);
                } else {
                    let t = nn - 32;
                    let v = *tail.add(t);
                    if t < 32 {
                        let old = _mm256_load_si256(tail as *const __m256i);
                        let shifted = _mm256_loadu_si256(tail.sub(1) as *const __m256i);
                        let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(t as i8 + 1), idx);
                        _mm256_store_si256(
                            tail as *mut __m256i,
                            _mm256_blendv_epi8(old, shifted, m),
                        );
                    } else {
                        std::ptr::copy(tail, tail.add(1), t);
                    }
                    *tail = _mm_extract_epi8::<15>(f1) as u8;
                    take_in(&mut f0, &mut f1, v);
                }
                _mm_cvtsi128_si32(f0) as u8
            },
            || { _mm_cvtsi128_si32(f0) as u8 }
        )
    }

    /// libbzip2's moving bases: `bases[k]` the start in `a` of sub-list k (16 bytes),
    /// bases moving down a byte as sub-lists before a far move take a byte on, repacked at
    /// the top of `a` when the first reaches 0
    struct Moving<const N: usize, const K: usize> {
        a: [u8; N],
        bases: [usize; K],
    }

    impl<const N: usize, const K: usize> Moving<N, K> {
        fn new(bytes: &[u8]) -> Self {
            let mut m = Moving {
                a: [0u8; N],
                bases: [0; K],
            };
            for (i, &c) in bytes.iter().enumerate() {
                m.a[N - 16 * K + i] = c;
            }
            for (k, base) in m.bases.iter_mut().enumerate() {
                *base = N - 16 * K + 16 * k;
            }
            m
        }
        /// Sub-list `lno`'s byte `off` out, the bytes before it in the sub-lists a byte
        /// on (by sub-list: its own a blend, the ones before it a moving base), `x` in at
        /// the front of sub-list 0; returns the byte taken out
        #[inline(always)]
        unsafe fn take(&mut self, lno: usize, off: usize, x: u8) -> u8 {
            let a = self.a.as_mut_ptr();
            let pp = *self.bases.get_unchecked(lno);
            let v = *a.add(pp + off);
            let row = a.add(pp);
            let keep = _mm_loadu_si128(row as *const __m128i);
            let on = _mm_loadu_si128(row.sub(1) as *const __m128i);
            _mm_storeu_si128(row as *mut __m128i, _mm_blendv_epi8(keep, on, le16(off)));
            let mut j = lno;
            while j > 0 {
                let below = *self.bases.get_unchecked(j - 1);
                *a.add(*self.bases.get_unchecked(j)) = *a.add(below + 15);
                *self.bases.get_unchecked_mut(j - 1) = below - 1;
                j -= 1;
            }
            *a.add(self.bases[0]) = x;
            if self.bases[0] == 0 {
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
                for j in (0..16).rev() {
                    kk -= 1;
                    self.a[kk] = self.a[b + j];
                }
                self.bases[k] = kk;
            }
        }
    }

    /// The two-level first level, libbzip2's moving-base sub-lists for the tail (14 of
    /// 16, in 1 KiB)
    #[target_feature(enable = "avx2")]
    pub unsafe fn regs_bases(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        let mut tail = Moving::<1024, 14>::new(&s2u[32..]);
        kernel_body!(
            b,
            ll,
            counts,
            |nn| {
                if nn < 32 {
                    first(&mut f0, &mut f1, nn);
                } else {
                    let t = nn - 32;
                    let x = _mm_extract_epi8::<15>(f1) as u8;
                    let v = tail.take(t >> 4, t & 15, x);
                    take_in(&mut f0, &mut f1, v);
                }
                _mm_cvtsi128_si32(f0) as u8
            },
            || { _mm_cvtsi128_si32(f0) as u8 }
        )
    }

    /// libbzip2's whole list (16 sub-lists of 16 at moving bases in 4 KiB, as
    /// `decompress.c`), vectorised: an index below 16 a 16-byte blend in sub-list 0 and its
    /// front written, past it `Moving::take`
    #[target_feature(enable = "avx2")]
    pub unsafe fn libbzip2(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let mut list = Moving::<4096, 16>::new(&b.seq_to_unseq);
        kernel_body!(
            b,
            ll,
            counts,
            |nn| {
                if nn < 16 {
                    let a = list.a.as_mut_ptr();
                    let row = a.add(list.bases[0]);
                    let v = *row.add(nn);
                    let keep = _mm_loadu_si128(row as *const __m128i);
                    let on = _mm_loadu_si128(row.sub(1) as *const __m128i);
                    _mm_storeu_si128(row as *mut __m128i, _mm_blendv_epi8(keep, on, le16(nn)));
                    *row = v;
                    v
                } else {
                    let v = list.take(nn >> 4, nn & 15, 0);
                    // `take` put x at the front; the byte taken out goes there
                    *list.a.as_mut_ptr().add(list.bases[0]) = v;
                    v
                }
            },
            || { list.a[list.bases[0]] }
        )
    }

    /// The second level as 7 rows of 32 at `tail` (slack before it): its byte t out, the
    /// bytes before it a byte on, `x` in at 0. The top row by a blend; the rows below it
    /// whole, top first, each shifted load taking the last byte of the row below it before
    /// that row is stored; row 0's taking `x`. One branch on the top row being row 0.
    #[inline(always)]
    unsafe fn rows_take(tail: *mut u8, t: usize, x: u8) -> u8 {
        let v = *tail.add(t);
        let k = t >> 5;
        let top = tail.add(32 * k);
        let m = _mm256_cmpgt_epi8(_mm256_set1_epi8((t & 31) as i8 + 1), iota());
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

    /// As `rows_take` with no branch on the row count: all 7 rows, each blended by its own
    /// mask (whole below the top row, up to t in it, none above it)
    #[inline(always)]
    unsafe fn rows_take_flat(tail: *mut u8, t: usize, x: u8) -> u8 {
        let v = *tail.add(t);
        let idx = iota();
        let mut j = 7;
        while j > 0 {
            j -= 1;
            let row = tail.add(32 * j);
            let keep = _mm256_load_si256(row as *const __m256i);
            let mut on = _mm256_loadu_si256(row.sub(1) as *const __m256i);
            if j == 0 {
                on = _mm256_insert_epi8::<0>(on, x as i8);
            }
            let lim = (t as isize - 32 * j as isize + 1).clamp(0, 32) as i8;
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(lim), idx);
            _mm256_store_si256(row as *mut __m256i, _mm256_blendv_epi8(keep, on, m));
        }
        v
    }

    /// The second level as 7 aligned rows of 32 at `tail`, each moved in registers: row j
    /// a byte on with the last byte of row j - 1 (of `f1`, for row 0) taken in at 0, by a
    /// lane permute and a byte align, blended up to t. Every load is an aligned row or a
    /// byte inside one, as the stores before it wrote them, so each forwards from one
    /// store. Row 0 alone for t < 32, else every row, with no branch on how many.
    #[inline(always)]
    unsafe fn rows_take_al(tail: *mut u8, t: usize, f1: __m128i) -> u8 {
        let v = *tail.add(t);
        let idx = iota();
        let row = |j: usize| _mm256_load_si256(tail.add(32 * j) as *const __m256i);
        // row -1 as the shift reads it: f1 in the high lane
        let below0 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(f1), f1);
        // [below's last byte, r's bytes 0..31]
        let on = |r: __m256i, below: __m256i| {
            _mm256_alignr_epi8::<15>(r, _mm256_permute2x128_si256::<0x03>(r, below))
        };
        if t < 32 {
            let r0 = row(0);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(t as i8 + 1), idx);
            _mm256_store_si256(
                tail as *mut __m256i,
                _mm256_blendv_epi8(r0, on(r0, below0), m),
            );
        } else {
            let tv = _mm256_set1_epi8(t as u8 as i8);
            let r: [__m256i; 7] = std::array::from_fn(row);
            for j in 0..7 {
                let p = _mm256_add_epi8(idx, _mm256_set1_epi8((32 * j) as u8 as i8));
                let m = _mm256_cmpeq_epi8(_mm256_min_epu8(p, tv), p); // 32j + i <= t
                let below = if j == 0 { below0 } else { r[j - 1] };
                let new = _mm256_blendv_epi8(r[j], on(r[j], below), m);
                _mm256_store_si256(tail.add(32 * j) as *mut __m256i, new);
            }
        }
        v
    }

    /// The first level with `rows_take_al`'s second
    #[target_feature(enable = "avx2")]
    pub unsafe fn rows_al(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        #[repr(align(64))]
        struct L([u8; 224]);
        let mut store = L([0u8; 224]);
        store.0.copy_from_slice(&s2u[32..]);
        let tail = store.0.as_mut_ptr();
        kernel_body!(
            b,
            ll,
            counts,
            |nn| {
                if nn < 32 {
                    first(&mut f0, &mut f1, nn);
                } else {
                    let v = rows_take_al(tail, nn - 32, f1);
                    take_in(&mut f0, &mut f1, v);
                }
                _mm_cvtsi128_si32(f0) as u8
            },
            || { _mm_cvtsi128_si32(f0) as u8 }
        )
    }

    macro_rules! rows_arm {
        ($name:ident, $take:ident) => {
            #[target_feature(enable = "avx2")]
            pub unsafe fn $name(
                b: &Symbols,
                ll: &mut Vec<u8>,
                counts: &mut [u32; 256],
            ) -> Result<usize, ErrorKind> {
                let s2u = &b.seq_to_unseq;
                let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
                let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
                #[repr(align(64))]
                struct L([u8; 256]);
                let mut store = L([0u8; 256]);
                store.0[32..].copy_from_slice(&s2u[32..]);
                let tail = store.0.as_mut_ptr().add(32);
                kernel_body!(
                    b,
                    ll,
                    counts,
                    |nn| {
                        if nn < 32 {
                            first(&mut f0, &mut f1, nn);
                        } else {
                            let x = _mm_extract_epi8::<15>(f1) as u8;
                            let v = $take(tail, nn - 32, x);
                            take_in(&mut f0, &mut f1, v);
                        }
                        _mm_cvtsi128_si32(f0) as u8
                    },
                    || { _mm_cvtsi128_si32(f0) as u8 }
                )
            }
        };
    }
    rows_arm!(rows_v2, rows_take);
    rows_arm!(rows_flat, rows_take_flat);

    /// A floor, not a decoder: the same loop with no list (its byte is seq_to_unseq[nn]),
    /// so no chain from one symbol's move to the next; its output is not checked
    #[target_feature(enable = "avx2")]
    pub unsafe fn skeleton(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let s2u = &b.seq_to_unseq;
        let mut last = s2u[0];
        kernel_body!(
            b,
            ll,
            counts,
            |nn| {
                last = s2u[nn];
                last
            },
            || { last }
        )
    }

    /// The library's kernel (the two-level one on this branch)
    pub unsafe fn library(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        mtf::mtf(b, ll, counts, Kernels::detect())
    }

    pub const ARMS: &[(&str, Kernel, bool)] = &[
        ("master", master, true),
        ("two-level", library, true),
        ("regs+memmove", regs_memmove, true),
        ("regs+bases", regs_bases, true),
        ("rows-v2", rows_v2, true),
        ("rows-flat", rows_flat, true),
        ("rows-al", rows_al, true),
        ("libbzip2", libbzip2, true),
        ("skeleton", skeleton, false),
    ];
}

#[cfg(target_arch = "x86_64")]
fn main() {
    use arms::*;
    use mibz2::block::Symbols;
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::{mtf, Kernels};

    assert!(is_x86_feature_detected!("avx2"), "AVX2 needed");
    let tsc = || unsafe { std::arch::x86_64::_rdtsc() };
    let check = |name: &str, k: Kernel, b: &Symbols| {
        let (mut l0, mut c0, mut l1, mut c1) = (Vec::new(), [0; 256], Vec::new(), [0; 256]);
        let r0 = mtf::mtf(b, &mut l0, &mut c0, Kernels::scalar());
        let r1 = unsafe { k(b, &mut l1, &mut c1) };
        assert_eq!(r0, r1, "{name}: result");
        if r0.is_ok() {
            assert!(l0 == l1, "{name}: ll");
            assert!(c0 == c1, "{name}: counts");
        }
    };
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(|s| s.as_str()) {
        Some("synth") => {
            let reps: usize = a.get(2).map_or(5, |s| s.parse().unwrap());
            let block = |syms: Vec<u16>| Symbols {
                crc: 0,
                orig_ptr: 0,
                seq_to_unseq: std::array::from_fn(|i| i as u8),
                syms,
                eob: 257,
                max_len: 900_000,
            };
            let mut cases: Vec<(String, Symbols)> = Vec::new();
            for nn in [1u16, 5, 15, 16, 31, 32, 40, 63, 64, 100, 200, 255] {
                let mut v = vec![nn + 1; 899_999];
                v.push(257);
                cases.push((format!("byte, index {nn}"), block(v)));
            }
            let mut x = 0x2545_f491_4f6c_dd1du64;
            let mut rnd = move || {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x
            };
            for hi in [16u64, 32, 64, 256] {
                let mut v: Vec<u16> = (0..899_999)
                    .map(|_| 2 + (rnd() % (hi - 1)) as u16)
                    .collect();
                v.push(257);
                cases.push((format!("byte, index uniform 1..{}", hi - 1), block(v)));
            }
            // Wikidata's byte-symbol mix (84.6% below 16, 13.7% 16-31, 1.6% 32-63, 0.3%
            // 64-127) with no run symbols
            let mut v: Vec<u16> = (0..899_999)
                .map(|_| {
                    let r = rnd() % 1000;
                    1 + if r < 846 {
                        1 + rnd() % 15
                    } else if r < 983 {
                        16 + rnd() % 16
                    } else if r < 997 {
                        32 + rnd() % 32
                    } else {
                        64 + rnd() % 64
                    } as u16
                })
                .collect();
            v.push(257);
            cases.push(("byte, Wikidata's index mix".into(), block(v)));
            println!("TSC cycles a symbol, best of {reps} (predictable branches: the chain)");
            print!("{:34}", "case");
            for (name, _, _) in ARMS {
                print!(" {name:>12}");
            }
            println!();
            for (name, b) in &cases {
                for &(an, k, checked) in ARMS {
                    if checked {
                        check(an, k, b);
                    }
                }
                print!("{name:34}");
                let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
                for &(_, k, _) in ARMS {
                    let mut best = u64::MAX;
                    for _ in 0..reps {
                        let t = tsc();
                        unsafe { k(b, &mut ll, &mut counts) }.unwrap();
                        best = best.min(tsc() - t);
                    }
                    print!(" {:>12.2}", best as f64 / b.syms.len() as f64);
                }
                println!();
            }
        }
        Some("file") => {
            let data = std::fs::read(&a[2]).unwrap();
            let rounds: usize = a.get(3).map_or(15, |s| s.parse().unwrap());
            let mut sc = Scanner::new(SliceSource(&data));
            let mut blocks = Vec::new();
            while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
                blocks.push(symbols);
            }
            let nsyms: usize = blocks.iter().map(|b| b.syms.len()).sum();
            let mut nbytes = 0;
            for b in &blocks {
                for &(an, k, checked) in ARMS {
                    if checked {
                        check(an, k, b);
                    }
                }
                let mut ll = Vec::new();
                nbytes += mtf::mtf(b, &mut ll, &mut [0; 256], Kernels::scalar()).unwrap();
            }
            println!(
                "{}: {} blocks, {nsyms} symbols, {nbytes} ll bytes; every checked arm's ll, \
                 counts and result equal the scalar mtf's on every block",
                a[2],
                blocks.len()
            );
            println!("round, then each arm's TSC cycles a symbol over the file; arms' order reversed every other round");
            let mut all: Vec<Vec<f64>> = vec![Vec::new(); ARMS.len()];
            let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
            for r in 0..rounds {
                let mut order: Vec<usize> = (0..ARMS.len()).collect();
                if r % 2 == 1 {
                    order.reverse();
                }
                let mut row = vec![0f64; ARMS.len()];
                for &i in &order {
                    let k = ARMS[i].1;
                    let t = tsc();
                    for b in &blocks {
                        let _ = unsafe { k(b, &mut ll, &mut counts) };
                    }
                    row[i] = (tsc() - t) as f64 / nsyms as f64;
                }
                print!("{r:3}");
                for (i, (name, _, _)) in ARMS.iter().enumerate() {
                    print!("  {name}: {:.3}", row[i]);
                    all[i].push(row[i]);
                }
                println!();
            }
            println!("summary: median [min, max] TSC cycles a symbol; paired ratio to master, median [min, max]");
            let med = |v: &mut Vec<f64>| {
                v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                v[v.len() / 2]
            };
            for (i, (name, _, _)) in ARMS.iter().enumerate() {
                let mut v = all[i].clone();
                let mut ratio: Vec<f64> = (0..rounds).map(|r| all[i][r] / all[0][r]).collect();
                let m = med(&mut v);
                let rm = med(&mut ratio);
                println!(
                    "  {name:14} {m:.3} [{:.3}, {:.3}]   ratio {rm:.4} [{:.4}, {:.4}]",
                    v[0],
                    v[v.len() - 1],
                    ratio[0],
                    ratio[ratio.len() - 1]
                );
            }
        }
        _ => panic!("usage: mtf_two_level synth [REPS] | mtf_two_level file FILE.bz2 ROUNDS"),
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn main() {}
