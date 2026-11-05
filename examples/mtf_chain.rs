//! MTF's serial chain timed (research, not a benchmark): `mtf::mtf` (the AVX2 kernel)
//! against prototype kernels that hold the list's front in registers, on synthetic symbol
//! streams of one kind and on a file's real blocks. Every prototype's `ll`, counts and
//! result are checked against `mtf::mtf`'s on every block it is timed on; the skeleton
//! arm (the same loop with the list's update taken out) is a floor, not a decoder, and is
//! not checked.
//!
//! ```text
//! mtf_chain synth [REPS]           (one-kind streams, best of REPS, TSC cycles a symbol)
//! mtf_chain file FILE.bz2 ROUNDS   (all blocks, arms interleaved, every round printed)
//! ```

use std::arch::x86_64::*;

use mibz2::block::Symbols;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, ErrorKind, Kernels};

const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;

/// pshufb controls moving index nn (< 16) to the front: [nn, 0, 1, .., nn - 1, nn + 1, ..]
const fn ctl16() -> [[u8; 16]; 16] {
    let mut t = [[0u8; 16]; 16];
    let mut nn = 0;
    while nn < 16 {
        let mut i = 0;
        while i < 16 {
            t[nn][i] = if i == 0 {
                nn as u8
            } else if i <= nn {
                (i - 1) as u8
            } else {
                i as u8
            };
            i += 1;
        }
        nn += 1;
    }
    t
}

#[repr(align(16))]
struct Ctl([[u8; 16]; 16]);
static CTL: Ctl = Ctl(ctl16());

type Kernel = unsafe fn(&Symbols, &mut Vec<u8>, &mut [u32; 256]) -> Result<usize, ErrorKind>;

/// The list's tail in memory, moved as `mtf_avx2` moves its list: index j's byte to the
/// front of the tail, `x` (the byte pushed out of the registers) in its place at 0. Returns
/// the byte that was at j.
#[inline(always)]
unsafe fn tail_move(tail: *mut u8, j: usize, x: u8, idx: __m256i) -> u8 {
    let v = *tail.add(j);
    if j < 32 {
        let old = _mm256_load_si256(tail as *const __m256i);
        let shifted = _mm256_loadu_si256(tail.sub(1) as *const __m256i);
        let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(j as i8 + 1), idx);
        _mm256_store_si256(tail as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
    } else {
        std::ptr::copy(tail, tail.add(1), j);
    }
    *tail = x;
    v
}

/// `mtf_avx2`'s run handling and checks, with the list's step `$step` (taking nn, giving
/// the new front byte) and its front `$front` (the front byte, for a run)
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

/// Control: `mtf_avx2` as it is in src/mtf.rs, compiled here
#[target_feature(enable = "avx2")]
unsafe fn cur(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    for i in 0..256 {
        store.0[32 + i] = b.seq_to_unseq[i];
    }
    let base = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
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

/// The list's first 16 bytes in an xmm register, moved by one pshufb whose control is a
/// table row by nn (off the chain); the other 240 in memory, moved as `mtf_avx2` moves
/// its list, only for an index of 16 or more
#[target_feature(enable = "avx2")]
unsafe fn reg16(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    for i in 16..256 {
        store.0[32 + i - 16] = b.seq_to_unseq[i];
    }
    let tail = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let mut f = _mm_loadu_si128(b.seq_to_unseq.as_ptr() as *const __m128i);
    let ctl = CTL.0.as_ptr();
    kernel_body!(
        b,
        ll,
        counts,
        |nn| {
            if nn < 16 {
                f = _mm_shuffle_epi8(f, _mm_load_si128(ctl.add(nn) as *const __m128i));
            } else {
                let x = _mm_extract_epi8::<15>(f) as u8;
                let v = tail_move(tail, nn - 16, x, idx);
                f = _mm_or_si128(_mm_slli_si128::<1>(f), _mm_cvtsi32_si128(v as i32));
            }
            _mm_cvtsi128_si32(f) as u8
        },
        || { _mm_cvtsi128_si32(f) as u8 }
    )
}

/// The list's first 32 bytes in two xmm registers (f0: 0..16, f1: 16..32): an index below
/// 16 one pshufb of f0; 16..32 a broadcast of f1's byte and an alignr into f0, alignr and
/// blend in f1; the other 224 in memory as `reg16`'s tail
#[target_feature(enable = "avx2")]
unsafe fn reg32(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    for i in 32..256 {
        store.0[32 + i - 32] = b.seq_to_unseq[i];
    }
    let tail = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let iota = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    let mut f0 = _mm_loadu_si128(b.seq_to_unseq.as_ptr() as *const __m128i);
    let mut f1 = _mm_loadu_si128(b.seq_to_unseq.as_ptr().add(16) as *const __m128i);
    let ctl = CTL.0.as_ptr();
    kernel_body!(
        b,
        ll,
        counts,
        |nn| {
            if nn < 16 {
                f0 = _mm_shuffle_epi8(f0, _mm_load_si128(ctl.add(nn) as *const __m128i));
            } else if nn < 32 {
                let j = _mm_set1_epi8((nn - 16) as i8);
                let vb = _mm_shuffle_epi8(f1, j);
                let u = _mm_alignr_epi8::<15>(f1, f0);
                f1 = _mm_blendv_epi8(u, f1, _mm_cmpgt_epi8(iota, j));
                f0 = _mm_alignr_epi8::<15>(f0, vb);
            } else {
                let x = _mm_extract_epi8::<15>(f1) as u8;
                let v = tail_move(tail, nn - 32, x, idx);
                f1 = _mm_alignr_epi8::<15>(f1, f0);
                f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
            }
            _mm_cvtsi128_si32(f0) as u8
        },
        || { _mm_cvtsi128_si32(f0) as u8 }
    )
}

/// A floor, not a decoder: the same loop with no list (the byte is seq_to_unseq[nn]), so
/// no chain from one symbol's move to the next; its output is wrong and not checked
#[target_feature(enable = "avx2")]
unsafe fn skeleton(
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

/// pshufb controls for a 32-byte front in two xmm registers, by nn < 32:
/// f0' = f0.shuffle(a) | f1.shuffle(b), f1' = f0.shuffle(c) | f1.shuffle(d)
#[repr(C, align(64))]
#[derive(Clone, Copy)]
struct Step {
    a: [u8; 16],
    b: [u8; 16],
    c: [u8; 16],
    d: [u8; 16],
}

const fn steps() -> [Step; 32] {
    const Z: u8 = 0x80;
    let mut t = [Step {
        a: [0; 16],
        b: [0; 16],
        c: [0; 16],
        d: [0; 16],
    }; 32];
    let ctl = ctl16();
    let mut nn = 0;
    while nn < 32 {
        let mut i = 0;
        while i < 16 {
            if nn < 16 {
                t[nn].a[i] = ctl[nn][i];
                t[nn].b[i] = Z;
                t[nn].c[i] = Z;
                t[nn].d[i] = i as u8;
            } else {
                let j = nn - 16;
                t[nn].a[i] = if i == 0 { Z } else { (i - 1) as u8 };
                t[nn].b[i] = if i == 0 { j as u8 } else { Z };
                t[nn].c[i] = if i == 0 { 15 } else { Z };
                t[nn].d[i] = if i == 0 {
                    Z
                } else if i <= j {
                    (i - 1) as u8
                } else {
                    i as u8
                };
            }
            i += 1;
        }
        nn += 1;
    }
    t
}

static STEPS: [Step; 32] = steps();

thread_local! {
    static REC: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Pass 1 of the two-pass kernels: every symbol, no branch on its kind; one record a byte
/// symbol, its index and the run before it ((run << 8) | nn), and after them the
/// end-of-block's record (the last run) if the block has one. None if a run's weight
/// would be refused (the caller then runs the one-pass kernel, for its exact error).
#[inline(always)]
unsafe fn classify(b: &Symbols, rec: *mut u32) -> Option<(usize, Option<u32>)> {
    let eob = b.eob;
    let (mut run, mut weight) = (0u32, 1u32);
    let mut k = 0usize;
    let mut bad = false;
    for &s in &b.syms {
        if s == eob {
            return if bad { None } else { Some((k, Some(run))) };
        }
        let is_run = s < 2;
        bad |= is_run & (weight >= MAX_RUN_WEIGHT);
        *rec.add(k) = ((s as u32).wrapping_sub(1) & 0xff) | (run << 8);
        k += (!is_run) as usize;
        let t = weight << (s & 1);
        run = if is_run { run.wrapping_add(t) } else { 0 };
        weight = if is_run { weight << 1 } else { 1 };
    }
    if bad {
        None
    } else {
        Some((k, None))
    }
}

/// Pass 2's list: a step taking nn (>= 1) to the new front byte, and the front byte
trait List {
    unsafe fn new(s2u: &[u8; 256]) -> Self;
    unsafe fn front(&self) -> u8;
    unsafe fn front32(&self) -> __m256i;
    unsafe fn step(&mut self, nn: usize) -> u8;
}

/// `mtf_avx2`'s list, in memory
struct MemList {
    store: [__m256i; 10],
    idx: __m256i,
}

impl List for MemList {
    #[inline(always)]
    unsafe fn new(s2u: &[u8; 256]) -> Self {
        let mut l = MemList {
            store: [_mm256_setzero_si256(); 10],
            idx: _mm256_setr_epi8(
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
            ),
        };
        let base = (l.store.as_mut_ptr() as *mut u8).add(32);
        std::ptr::copy_nonoverlapping(s2u.as_ptr(), base, 256);
        l
    }
    #[inline(always)]
    unsafe fn front(&self) -> u8 {
        *(self.store.as_ptr() as *const u8).add(32)
    }
    #[inline(always)]
    unsafe fn front32(&self) -> __m256i {
        _mm256_set1_epi8(self.front() as i8)
    }
    #[inline(always)]
    unsafe fn step(&mut self, nn: usize) -> u8 {
        let base = (self.store.as_mut_ptr() as *mut u8).add(32);
        let v = *base.add(nn);
        if nn < 32 {
            let old = _mm256_load_si256(base as *const __m256i);
            let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(nn as i8 + 1), self.idx);
            _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
        } else {
            std::ptr::copy(base, base.add(1), nn);
        }
        *base = v;
        v
    }
}

/// The front 32 in two xmm registers, moved with no branch below 32 (four pshufb by a
/// table row), the tail in memory
struct Reg32 {
    f0: __m128i,
    f1: __m128i,
    store: [__m256i; 10],
    idx: __m256i,
}

impl List for Reg32 {
    #[inline(always)]
    unsafe fn new(s2u: &[u8; 256]) -> Self {
        let mut l = Reg32 {
            f0: _mm_loadu_si128(s2u.as_ptr() as *const __m128i),
            f1: _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i),
            store: [_mm256_setzero_si256(); 10],
            idx: _mm256_setr_epi8(
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
            ),
        };
        let tail = (l.store.as_mut_ptr() as *mut u8).add(32);
        std::ptr::copy_nonoverlapping(s2u.as_ptr().add(32), tail, 224);
        l
    }
    #[inline(always)]
    unsafe fn front(&self) -> u8 {
        _mm_cvtsi128_si32(self.f0) as u8
    }
    #[inline(always)]
    unsafe fn front32(&self) -> __m256i {
        _mm256_broadcastb_epi8(self.f0)
    }
    #[inline(always)]
    unsafe fn step(&mut self, nn: usize) -> u8 {
        if nn < 32 {
            let t = STEPS.as_ptr().add(nn) as *const __m128i;
            let (f0, f1) = (self.f0, self.f1);
            self.f0 = _mm_or_si128(
                _mm_shuffle_epi8(f0, _mm_load_si128(t)),
                _mm_shuffle_epi8(f1, _mm_load_si128(t.add(1))),
            );
            self.f1 = _mm_or_si128(
                _mm_shuffle_epi8(f0, _mm_load_si128(t.add(2))),
                _mm_shuffle_epi8(f1, _mm_load_si128(t.add(3))),
            );
        } else {
            let tail = (self.store.as_mut_ptr() as *mut u8).add(32);
            let x = _mm_extract_epi8::<15>(self.f1) as u8;
            let v = tail_move(tail, nn - 32, x, self.idx);
            self.f1 = _mm_alignr_epi8::<15>(self.f1, self.f0);
            self.f0 = _mm_or_si128(_mm_slli_si128::<1>(self.f0), _mm_cvtsi32_si128(v as i32));
        }
        _mm_cvtsi128_si32(self.f0) as u8
    }
}

/// Pass 2: the byte records only, the run before each written as RW bytes of the front
/// (longer runs, and the block's last RW bytes, take the exact path), the count of a byte
/// deferred to the next record (with the run of it that record writes), so that no two
/// consecutive records add to the same count
#[inline(always)]
unsafe fn emit<L: List, const RW: usize>(
    b: &Symbols,
    recs: &[u32],
    last: Option<u32>,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    let max = b.max_len;
    let out = ll.as_mut_ptr();
    let mut n = 0usize;
    let mut list = L::new(&b.seq_to_unseq);
    let mut prev = list.front() as usize;
    let mut pend = 0u32;
    let mut result = Ok(());
    for &r in recs {
        let nn = (r & 0xff) as usize;
        let run = r >> 8;
        if run as usize <= RW && n + RW < max {
            let v = list.front32();
            _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
            if RW == 64 {
                _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
            }
            n += run as usize;
        } else {
            if run != 0 {
                if n + run as usize > max {
                    result = Err(ErrorKind::BlockTooLong);
                    break;
                }
                std::ptr::write_bytes(out.add(n), list.front(), run as usize);
                n += run as usize;
            }
            if n >= max {
                counts[prev] += run;
                result = Err(ErrorKind::BlockTooLong);
                break;
            }
        }
        let c = list.step(nn);
        counts[prev] += pend + run;
        (prev, pend) = (c as usize, 1);
        *out.add(n) = c;
        n += 1;
    }
    if result.is_ok() {
        if let Some(run) = last {
            if run != 0 {
                if n + run as usize > max {
                    result = Err(ErrorKind::BlockTooLong);
                } else {
                    std::ptr::write_bytes(out.add(n), list.front(), run as usize);
                    n += run as usize;
                    pend += run;
                }
            }
        }
    }
    counts[prev] += pend;
    ll.set_len(n);
    result.map(|()| n)
}

#[inline(always)]
unsafe fn two_pass<L: List, const RW: usize>(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    ll.clear();
    ll.reserve(b.max_len);
    *counts = [0; 256];
    REC.with(|cell| {
        let mut rec = cell.borrow_mut();
        rec.clear();
        rec.reserve(b.syms.len() + 1);
        match classify(b, rec.as_mut_ptr()) {
            None => cur(b, ll, counts),
            Some((k, last)) => emit::<L, RW>(b, std::slice::from_raw_parts(rec.as_ptr(), k), last, ll, counts),
        }
    })
}

#[target_feature(enable = "avx2")]
unsafe fn tp_mem(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    two_pass::<MemList, 64>(b, ll, counts)
}

#[target_feature(enable = "avx2")]
unsafe fn tp32_64(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    two_pass::<Reg32, 64>(b, ll, counts)
}

#[target_feature(enable = "avx2")]
unsafe fn tp32_32(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    two_pass::<Reg32, 32>(b, ll, counts)
}

/// Pass 1 alone (a floor for the two-pass kernels' first pass; no output)
#[target_feature(enable = "avx2")]
unsafe fn pass1_only(b: &Symbols, ll: &mut Vec<u8>, _counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    REC.with(|cell| {
        let mut rec = cell.borrow_mut();
        rec.clear();
        rec.reserve(b.syms.len() + 1);
        let k = classify(b, rec.as_mut_ptr()).map_or(0, |x| x.0);
        ll.clear();
        Ok(std::hint::black_box(k))
    })
}

/// 64 symbols' masks: run symbols (s < 2), RUNB (s == 1), the end-of-block symbol
#[inline(always)]
unsafe fn masks64(p: *const u16, eob: u16) -> (u64, u64, bool) {
    let z = _mm256_setzero_si256();
    let one = _mm256_set1_epi16(1);
    let e = _mm256_set1_epi16(eob as i16);
    let v: [__m256i; 4] = std::array::from_fn(|i| _mm256_loadu_si256(p.add(16 * i) as *const __m256i));
    let r: [__m256i; 4] = std::array::from_fn(|i| _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(v[i]), z));
    let d: [__m256i; 4] = std::array::from_fn(|i| _mm256_cmpeq_epi16(v[i], one));
    let any_e = _mm256_or_si256(
        _mm256_or_si256(_mm256_cmpeq_epi16(v[0], e), _mm256_cmpeq_epi16(v[1], e)),
        _mm256_or_si256(_mm256_cmpeq_epi16(v[2], e), _mm256_cmpeq_epi16(v[3], e)),
    );
    let pack = |a: __m256i, b: __m256i| -> u64 {
        _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0b11_01_10_00>(_mm256_packs_epi16(a, b)))
            as u32 as u64
    };
    let rm = pack(r[0], r[1]) | (pack(r[2], r[3]) << 32);
    let dm = pack(d[0], d[1]) | (pack(d[2], d[3]) << 32);
    (rm, dm, _mm256_testz_si256(any_e, any_e) == 0)
}

/// Pass 1 by bit masks: a window of 64 symbols' run and RUNB masks made with AVX2, then
/// one step a byte symbol (tzcnt of the byte mask): the run before it is the run symbols
/// since the byte symbol before, (2^m - 1) + their RUNB bits (bijective base 2). The same
/// records as `classify`; None where the one-pass kernel must run (a refused weight, or
/// an end-of-block symbol before the last)
#[inline(always)]
unsafe fn classify_mask(b: &Symbols, rec: *mut u32) -> Option<(usize, Option<u32>)> {
    let syms = &b.syms;
    let eob = b.eob;
    let len = syms.len();
    let ended = len > 0 && syms[len - 1] == eob;
    let body = if ended { len - 1 } else { len };
    let p = syms.as_ptr();
    let mut k = 0usize;
    // the open run: its digits so far and their RUNB bits; acc collects every run's
    // 2^m - 1, so bit 21 is set if any run had 22 digits (a weight of 2^21 refused)
    let (mut cm, mut cbits) = (0u32, 0u64);
    let mut acc = 0u64;
    let mut w = 0usize;
    while w < body {
        let n = (body - w).min(64);
        let (rm, dm, e) = if n == 64 {
            masks64(p.add(w), eob)
        } else {
            let (mut rm, mut dm, mut e) = (0u64, 0u64, false);
            for i in 0..n {
                let s = *p.add(w + i);
                rm |= ((s < 2) as u64) << i;
                dm |= ((s == 1) as u64) << i;
                e |= s == eob;
            }
            (rm, dm, e)
        };
        if e {
            return None;
        }
        let valid = if n == 64 { !0u64 } else { (1u64 << n) - 1 };
        let mut bm = !rm & valid;
        if bm != 0 {
            // the first byte symbol of the window closes the run carried in
            let j = bm.trailing_zeros();
            let m = cm + j;
            let mask = if m >= 64 { !0 } else { (1u64 << m) - 1 };
            let bits = cbits | ((dm & ((1u64 << j) - 1)) << cm.min(63));
            acc |= mask;
            let run = mask.wrapping_add(bits) as u32;
            let s = *p.add(w + j as usize);
            *rec.add(k) = ((s as u32).wrapping_sub(1) & 0xff) | (run << 8);
            k += 1;
            let mut prev = j + 1;
            bm &= bm - 1;
            while bm != 0 {
                let j = bm.trailing_zeros();
                let m = j - prev;
                let mask = (1u64 << m) - 1;
                let bits = (dm >> prev) & mask;
                acc |= mask;
                let run = (mask + bits) as u32;
                let s = *p.add(w + j as usize);
                *rec.add(k) = ((s as u32).wrapping_sub(1) & 0xff) | (run << 8);
                k += 1;
                prev = j + 1;
                bm &= bm - 1;
            }
            // the run symbols after the window's last byte symbol open the next run
            let m = n as u32 - prev;
            cm = m;
            cbits = if m == 0 { 0 } else { (dm >> prev) & ((1u64 << m) - 1) };
        } else {
            // no byte symbol: the whole window extends the open run
            if cm < 64 {
                cbits |= dm.checked_shl(cm).unwrap_or(0) & valid.checked_shl(cm).unwrap_or(0);
            }
            cm += n as u32;
        }
        w += 64;
    }
    // the last run, flushed by the end-of-block symbol
    let mask = if cm >= 64 { !0 } else { (1u64 << cm) - 1 };
    acc |= mask;
    if acc >> 21 & 1 != 0 {
        return None;
    }
    Some((k, ended.then(|| mask.wrapping_add(cbits) as u32)))
}

#[target_feature(enable = "avx2,bmi1,bmi2")]
unsafe fn tpm32_32(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    ll.clear();
    ll.reserve(b.max_len);
    *counts = [0; 256];
    REC.with(|cell| {
        let mut rec = cell.borrow_mut();
        rec.clear();
        rec.reserve(b.syms.len() + 1);
        match classify_mask(b, rec.as_mut_ptr()) {
            None => cur(b, ll, counts),
            Some((k, last)) => {
                emit::<Reg32, 32>(b, std::slice::from_raw_parts(rec.as_ptr(), k), last, ll, counts)
            }
        }
    })
}

#[target_feature(enable = "avx2,bmi1,bmi2")]
unsafe fn tpm32_64(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    ll.clear();
    ll.reserve(b.max_len);
    *counts = [0; 256];
    REC.with(|cell| {
        let mut rec = cell.borrow_mut();
        rec.clear();
        rec.reserve(b.syms.len() + 1);
        match classify_mask(b, rec.as_mut_ptr()) {
            None => cur(b, ll, counts),
            Some((k, last)) => {
                emit::<Reg32, 64>(b, std::slice::from_raw_parts(rec.as_ptr(), k), last, ll, counts)
            }
        }
    })
}

/// Pass 1 by masks alone (a floor; no output)
#[target_feature(enable = "avx2,bmi1,bmi2")]
unsafe fn pass1m_only(b: &Symbols, ll: &mut Vec<u8>, _counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    REC.with(|cell| {
        let mut rec = cell.borrow_mut();
        rec.clear();
        rec.reserve(b.syms.len() + 1);
        let k = classify_mask(b, rec.as_mut_ptr()).map_or(0, |x| x.0);
        ll.clear();
        Ok(std::hint::black_box(k))
    })
}

/// Reg32's step for an index of 32 or more, out of line so that the loop keeps f0 and f1
/// in registers
#[inline(never)]
#[cold]
#[target_feature(enable = "avx2")]
unsafe fn far32(f0: __m128i, f1: __m128i, tail: *mut u8, nn: usize) -> (__m128i, __m128i) {
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let x = _mm_extract_epi8::<15>(f1) as u8;
    let v = tail_move(tail, nn - 32, x, idx);
    (
        _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32)),
        _mm_alignr_epi8::<15>(f1, f0),
    )
}

/// `emit::<Reg32, RW>` written out: f0 and f1 locals, the tail its own array, the far step
/// out of line
#[inline(always)]
unsafe fn emit_r32<const RW: usize>(
    b: &Symbols,
    recs: &[u32],
    last: Option<u32>,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    let max = b.max_len;
    let out = ll.as_mut_ptr();
    let mut n = 0usize;
    let s2u = &b.seq_to_unseq;
    let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
    let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
    #[repr(align(64))]
    struct T([u8; 320]);
    let mut store = T([0u8; 320]);
    std::ptr::copy_nonoverlapping(s2u.as_ptr().add(32), store.0.as_mut_ptr().add(32), 224);
    let tail = store.0.as_mut_ptr().add(32);
    let steps = STEPS.as_ptr() as *const __m128i;
    let mut prev = (_mm_cvtsi128_si32(f0) as u8) as usize;
    let mut pend = 0u32;
    let mut result = Ok(());
    for &r in recs {
        let nn = (r & 0xff) as usize;
        let run = r >> 8;
        if run as usize <= RW && n + RW < max {
            let v = _mm256_broadcastb_epi8(f0);
            _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
            if RW == 64 {
                _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
            }
            n += run as usize;
        } else {
            if run != 0 {
                if n + run as usize > max {
                    result = Err(ErrorKind::BlockTooLong);
                    break;
                }
                std::ptr::write_bytes(out.add(n), _mm_cvtsi128_si32(f0) as u8, run as usize);
                n += run as usize;
            }
            if n >= max {
                counts[prev] += run;
                result = Err(ErrorKind::BlockTooLong);
                break;
            }
        }
        if nn < 32 {
            let t = steps.add(4 * nn);
            let (a, c) = (f0, f1);
            f0 = _mm_or_si128(
                _mm_shuffle_epi8(a, _mm_load_si128(t)),
                _mm_shuffle_epi8(c, _mm_load_si128(t.add(1))),
            );
            f1 = _mm_or_si128(
                _mm_shuffle_epi8(a, _mm_load_si128(t.add(2))),
                _mm_shuffle_epi8(c, _mm_load_si128(t.add(3))),
            );
        } else {
            (f0, f1) = far32(f0, f1, tail, nn);
        }
        let c = _mm_cvtsi128_si32(f0) as u8;
        counts[prev] += pend + run;
        (prev, pend) = (c as usize, 1);
        *out.add(n) = c;
        n += 1;
    }
    if result.is_ok() {
        if let Some(run) = last {
            if run != 0 {
                if n + run as usize > max {
                    result = Err(ErrorKind::BlockTooLong);
                } else {
                    std::ptr::write_bytes(out.add(n), _mm_cvtsi128_si32(f0) as u8, run as usize);
                    n += run as usize;
                    pend += run;
                }
            }
        }
    }
    counts[prev] += pend;
    ll.set_len(n);
    result.map(|()| n)
}

#[target_feature(enable = "avx2,bmi1,bmi2")]
unsafe fn tpx32_32(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    ll.clear();
    ll.reserve(b.max_len);
    *counts = [0; 256];
    REC.with(|cell| {
        let mut rec = cell.borrow_mut();
        rec.clear();
        rec.reserve(b.syms.len() + 1);
        match classify_mask(b, rec.as_mut_ptr()) {
            None => cur(b, ll, counts),
            Some((k, last)) => {
                emit_r32::<32>(b, std::slice::from_raw_parts(rec.as_ptr(), k), last, ll, counts)
            }
        }
    })
}

#[target_feature(enable = "avx2,bmi1,bmi2")]
unsafe fn tpx32_64(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    ll.clear();
    ll.reserve(b.max_len);
    *counts = [0; 256];
    REC.with(|cell| {
        let mut rec = cell.borrow_mut();
        rec.clear();
        rec.reserve(b.syms.len() + 1);
        match classify_mask(b, rec.as_mut_ptr()) {
            None => cur(b, ll, counts),
            Some((k, last)) => {
                emit_r32::<64>(b, std::slice::from_raw_parts(rec.as_ptr(), k), last, ll, counts)
            }
        }
    })
}

/// `mtf_avx2`'s loop as it is (its run/byte branch, its run writes, its counts), the list
/// replaced by emit_r32's register front: an index below 32 four pshufb with no branch,
/// 32 or more out of line
#[target_feature(enable = "avx2")]
unsafe fn br32(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    let s2u = &b.seq_to_unseq;
    let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
    let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
    #[repr(align(64))]
    struct T([u8; 320]);
    let mut store = T([0u8; 320]);
    std::ptr::copy_nonoverlapping(s2u.as_ptr().add(32), store.0.as_mut_ptr().add(32), 224);
    let tail = store.0.as_mut_ptr().add(32);
    let steps = STEPS.as_ptr() as *const __m128i;
    kernel_body!(
        b,
        ll,
        counts,
        |nn| {
            if nn < 32 {
                let t = steps.add(4 * nn);
                let (a, c) = (f0, f1);
                f0 = _mm_or_si128(
                    _mm_shuffle_epi8(a, _mm_load_si128(t)),
                    _mm_shuffle_epi8(c, _mm_load_si128(t.add(1))),
                );
                f1 = _mm_or_si128(
                    _mm_shuffle_epi8(a, _mm_load_si128(t.add(2))),
                    _mm_shuffle_epi8(c, _mm_load_si128(t.add(3))),
                );
            } else {
                (f0, f1) = far32(f0, f1, tail, nn);
            }
            _mm_cvtsi128_si32(f0) as u8
        },
        || { _mm_cvtsi128_si32(f0) as u8 }
    )
}

/// Check classify_mask's records against classify's on a block
fn check_classify(b: &Symbols) {
    let mut r0 = vec![0u32; b.syms.len() + 1];
    let mut r1 = vec![0u32; b.syms.len() + 1];
    let a = unsafe { classify(b, r0.as_mut_ptr()) };
    let c = unsafe { classify_mask(b, r1.as_mut_ptr()) };
    assert_eq!(a, c, "classify_mask's result");
    if let Some((k, _)) = a {
        assert!(r0[..k] == r1[..k], "classify_mask's records");
    }
}

fn library(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    mtf::mtf(b, ll, counts, Kernels::detect())
}

unsafe fn library_k(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    library(b, ll, counts)
}

const ARMS: [(&str, Kernel, bool); 15] = [
    ("mtf::mtf", library_k, true),
    ("cur (copy)", cur, true),
    ("reg16", reg16, true),
    ("reg32", reg32, true),
    ("skeleton", skeleton, false),
    ("tp_mem", tp_mem, true),
    ("tp32_64", tp32_64, true),
    ("tp32_32", tp32_32, true),
    ("pass1", pass1_only, false),
    ("tpm32_32", tpm32_32, true),
    ("tpm32_64", tpm32_64, true),
    ("pass1m", pass1m_only, false),
    ("tpx32_32", tpx32_32, true),
    ("tpx32_64", tpx32_64, true),
    ("br32", br32, true),
];

fn tsc() -> u64 {
    unsafe { _rdtsc() }
}

fn synth_block(syms: Vec<u16>) -> Symbols {
    let mut s = syms;
    s.push(257);
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq: std::array::from_fn(|i| (255 - i) as u8),
        syms: s,
        eob: 257,
        max_len: 900_000,
    }
}

/// Check an arm against the library on a block: the same result, ll and counts
fn check(name: &str, k: Kernel, b: &Symbols) {
    let (mut ll0, mut c0) = (Vec::new(), [0u32; 256]);
    let (mut ll1, mut c1) = (Vec::new(), [0u32; 256]);
    let r0 = library(b, &mut ll0, &mut c0);
    let r1 = unsafe { k(b, &mut ll1, &mut c1) };
    assert_eq!(r0, r1, "{name}: result");
    if r0.is_ok() {
        assert!(ll0 == ll1, "{name}: ll");
        assert!(c0 == c1, "{name}: counts");
    }
}

fn synth(reps: usize) {
    let mut rng = 0x9e3779b97f4a7c15u64;
    let mut rand = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    let mut cases: Vec<(String, Symbols)> = Vec::new();
    for nn in [1u16, 2, 8, 15, 16, 24, 31, 32, 40, 100] {
        cases.push((format!("byte, index {nn}"), synth_block(vec![nn + 1; 899_999])));
    }
    // uniform random indices below a bound
    for hi in [16u64, 32, 256] {
        let v: Vec<u16> = (0..899_999).map(|_| (1 + rand() % (hi - 1)) as u16 + 1).collect();
        cases.push((format!("byte, index uniform 1..{}", hi - 1), synth_block(v)));
    }
    // a run of one byte (RUNA) then a byte of index 1, repeated
    let v: Vec<u16> = (0..449_999).flat_map(|_| [0u16, 2]).collect();
    cases.push(("RUNA, index 1, repeated".into(), synth_block(v)));
    // RUNB RUNA (a run of 4), index 5
    let v: Vec<u16> = (0..179_999).flat_map(|_| [1u16, 0, 6]).collect();
    cases.push(("RUNB RUNA, index 5, repeated".into(), synth_block(v)));
    // random: each symbol a run symbol with probability 0.4, else index 1..15
    let mut v = Vec::new();
    let mut out = 0usize;
    let mut w = 1usize;
    let mut run = 0usize;
    while out + run + 64 < 899_000 {
        if rand() % 10 < 4 && w < 64 {
            let s = (rand() & 1) as u16;
            run += (s as usize + 1) * w;
            w <<= 1;
            v.push(s);
        } else {
            out += run + 1;
            run = 0;
            w = 1;
            v.push((1 + rand() % 15) as u16 + 1);
        }
    }
    cases.push(("random: 40% run symbols, index 1..15".into(), synth_block(v)));
    println!("TSC cycles a symbol (TSC at the nominal clock), best of {reps}");
    print!("{:42}", "case");
    for (name, _, _) in ARMS {
        print!(" {name:>11}");
    }
    println!();
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    for (name, b) in &cases {
        for (an, k, checked) in ARMS {
            if checked {
                check(an, k, b);
            }
        }
        check_classify(b);
        print!("{name:42}");
        let nsyms = b.syms.len() - 1;
        for (_, k, _) in ARMS {
            let mut best = u64::MAX;
            for _ in 0..reps {
                let t = tsc();
                unsafe { k(b, &mut ll, &mut counts) }.unwrap();
                best = best.min(tsc() - t);
            }
            print!(" {:>11.2}", best as f64 / nsyms as f64);
        }
        println!();
    }
}

fn file(path: &str, rounds: usize) {
    let data = std::fs::read(path).unwrap();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut blocks = Vec::new();
    loop {
        match sc.next(Vec::new()).unwrap() {
            Event::Block { symbols, .. } => blocks.push(symbols),
            Event::Done => break,
        }
    }
    let nsyms: usize = blocks.iter().map(|b| b.syms.len() - 1).sum();
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let mut nbytes = 0usize;
    for b in &blocks {
        nbytes += library(b, &mut ll, &mut counts).unwrap();
        check_classify(b);
        for (an, k, checked) in ARMS {
            if checked {
                check(an, k, b);
            }
        }
    }
    println!(
        "{path}: {} blocks, {nsyms} symbols, {nbytes} ll bytes; every checked arm's ll, counts and result equal mtf::mtf's on every block",
        blocks.len()
    );
    println!("round, then each arm's TSC cycles a symbol (and a ll byte) over the file; arms' order reversed every other round");
    let mut all: Vec<Vec<f64>> = vec![Vec::new(); ARMS.len()];
    for r in 0..rounds {
        let order: Vec<usize> = if r % 2 == 0 {
            (0..ARMS.len()).collect()
        } else {
            (0..ARMS.len()).rev().collect()
        };
        let mut row = vec![0f64; ARMS.len()];
        for &a in &order {
            let k = ARMS[a].1;
            let t = tsc();
            for b in &blocks {
                unsafe { k(b, &mut ll, &mut counts) }.unwrap();
            }
            row[a] = (tsc() - t) as f64;
        }
        print!("{r:3}");
        for (a, &c) in row.iter().enumerate() {
            print!(
                "  {}: {:.3} ({:.4})",
                ARMS[a].0,
                c / nsyms as f64,
                c / nbytes as f64
            );
            all[a].push(c / nsyms as f64);
        }
        println!();
    }
    println!("summary: median [min, max] TSC cycles a symbol; paired ratio to mtf::mtf, median [min, max]");
    for (a, v) in all.iter().enumerate() {
        let mut s = v.clone();
        s.sort_by(|x, y| x.partial_cmp(y).unwrap());
        let mut rat: Vec<f64> = v.iter().zip(&all[0]).map(|(x, y)| x / y).collect();
        rat.sort_by(|x, y| x.partial_cmp(y).unwrap());
        println!(
            "  {:12} {:.3} [{:.3}, {:.3}]   ratio {:.4} [{:.4}, {:.4}]",
            ARMS[a].0,
            s[s.len() / 2],
            s[0],
            s[s.len() - 1],
            rat[rat.len() / 2],
            rat[0],
            rat[rat.len() - 1]
        );
    }
}

fn main() {
    assert!(is_x86_feature_detected!("avx2"), "AVX2 needed");
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("synth") => synth(a.get(2).map_or(5, |s| s.parse().unwrap())),
        Some("file") => file(&a[2], a.get(3).map_or(15, |s| s.parse().unwrap())),
        _ => panic!("usage: mtf_chain synth [REPS] | mtf_chain file FILE.bz2 ROUNDS"),
    }
}
