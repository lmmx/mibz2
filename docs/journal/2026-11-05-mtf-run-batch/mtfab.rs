//! Scratch harness (not in the repo): master's mtf_avx2 (copied verbatim as `old`) against
//! the branch's mtf::mtf (AVX2), each block of a file timed with rdtsc, both kernels on
//! every block, the order alternating by block and round; outputs checked equal.
//! Usage: mtfab FILE ROUNDS
use mibz2::block::Symbols;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, ErrorKind, Kernels};

const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn old(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    use std::arch::x86_64::*;
    let max = b.max_len;
    ll.clear();
    ll.reserve(max);
    *counts = [0; 256];
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    // the list at store[32..288], with slack before it for the shifted load, held as the
    // bytes themselves (each symbol's seq_to_unseq), not their indices: a move permutes
    // both alike, so the front is the byte with no lookup
    for i in 0..256 {
        store.0[32 + i] = b.seq_to_unseq[i];
    }
    let base = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let (mut run, mut weight) = (0u32, 1u32);
    // `n` bytes written at `out`; every write below is checked against `max`, which
    // `ll`'s capacity holds
    let out = ll.as_mut_ptr();
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
            if n + run as usize > max {
                result = Err(ErrorKind::BlockTooLong);
                break;
            }
            let c = *base;
            counts[c as usize] += run;
            if run <= 64 && n + 64 <= max {
                // a short run as two 32-byte stores of its byte: the bytes past the run are
                // still inside `ll`'s capacity (n + 64 <= max), and are written again by
                // what follows or left past the length set at the end
                let v = _mm256_set1_epi8(c as i8);
                _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
            } else {
                std::ptr::write_bytes(out.add(n), c, run as usize);
            }
            n += run as usize;
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        if n >= max {
            result = Err(ErrorKind::BlockTooLong);
            break;
        }
        // (s - 1 < 256: a symbol is below the alphabet, at most 257)
        let nn = (s - 1) as usize;
        let v = *base.add(nn);
        if nn < 32 {
            let old = _mm256_load_si256(base as *const __m256i);
            let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(nn as i8 + 1), idx); // i <= nn
            _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
        } else {
            std::ptr::copy(base, base.add(1), nn);
        }
        *base = v;
        let c = v;
        counts[c as usize] += 1;
        *out.add(n) = c;
        n += 1;
    }
    // SAFETY: the first n bytes were written above, n <= max <= capacity
    ll.set_len(n);
    result.map(|()| n)
}

/// The run symbols in a row that libbzip2 refuses: the 22nd's weight is 2^21
const RUN_SYMBOLS_REFUSED: u32 = 22;

/// A window of 64 symbols from `w`, as bit masks (bit j for `w[j]`): the byte symbols
/// (s >= 2), the symbols other than RUNA (s != 0), and whether the window holds the
/// end-of-block symbol or `RUN_SYMBOLS_REFUSED` run symbols in a row (the window is then
/// left to the symbol loop)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn window64(w: *const u16, eob: u16) -> (u64, u64, bool) {
    use std::arch::x86_64::*;
    let z = _mm256_setzero_si256();
    let e = _mm256_set1_epi16(eob as i16);
    let s0 = _mm256_loadu_si256(w as *const __m256i);
    let s1 = _mm256_loadu_si256(w.add(16) as *const __m256i);
    let s2 = _mm256_loadu_si256(w.add(32) as *const __m256i);
    let s3 = _mm256_loadu_si256(w.add(48) as *const __m256i);
    // run symbols (s >> 1 == 0) and RUNA (s == 0), each 16-bit lane 0 or -1
    macro_rules! run {
        ($x:expr) => {
            _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>($x), z)
        };
    }
    let ends = _mm256_or_si256(
        _mm256_or_si256(_mm256_cmpeq_epi16(s0, e), _mm256_cmpeq_epi16(s1, e)),
        _mm256_or_si256(_mm256_cmpeq_epi16(s2, e), _mm256_cmpeq_epi16(s3, e)),
    );
    // 32 lanes' masks in order: the packs interleave the 128-bit halves, the permute
    // puts them back
    macro_rules! bits {
        ($x:expr, $y:expr) => {
            _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0b11_01_10_00>(
                _mm256_packs_epi16($x, $y),
            )) as u32 as u64
        };
    }
    let runs = bits!(run!(s0), run!(s1)) | bits!(run!(s2), run!(s3)) << 32;
    let runa = bits!(_mm256_cmpeq_epi16(s0, z), _mm256_cmpeq_epi16(s1, z))
        | bits!(_mm256_cmpeq_epi16(s2, z), _mm256_cmpeq_epi16(s3, z)) << 32;
    // bit j of x set where symbols j.. j + 21 are all run symbols
    let mut x = runs;
    x &= x >> 1;
    x &= x >> 2;
    x &= x >> 4;
    x &= x >> 8;
    x &= x >> (RUN_SYMBOLS_REFUSED - 16);
    (!runs, !runa, x != 0 || _mm256_testz_si256(ends, ends) == 0)
}

/// The AVX2 kernel. Its symbol loop is `mtf_scalar`'s, the list held as bytes and moved
/// by a blend (an index below 32) or a copy. After the block's first byte symbol it runs
/// the window loop, one step a byte symbol with the run before it, until a window it
/// leaves to the symbol loop: the block's last symbols (the end-of-block symbol among
/// them), a run of 22 symbols (refused), or a step that would end within 64 bytes of
/// the block's limit. The symbol loop then finishes the block from that run's start.
///
/// The window loop: 64 symbols from a run's start as bit masks, their byte symbols taken
/// in order (tzcnt, blsr). A byte symbol's run is the bijective base-2 number of the run
/// symbols before it, sum (s + 1) 2^k, which is 2^m - 1 plus their RUNB bits. With the
/// byte symbol's own bit (bit m, set in the not-RUNA mask) that is the run plus one: the
/// bits from the last byte symbol to this one, read with one shift and a bzhi. No branch
/// on a symbol's kind, so none mispredicted from one symbol to the next: the loop's one
/// unpredictable exit is once a window.
///
/// Each step writes a segment: the byte the step before moved to the front, and the run
/// of it before this step's byte symbol, `seg` bytes as two 32-byte stores (or a fill
/// above 64), counted at once. Its byte is then moved, and written by the next step (or
/// at the loop's exit). Every segment ends at most at `max - 64`, so the stores past it
/// stay inside `ll`'s capacity, the checks that `mtf_scalar` makes on the run and the
/// byte both pass, and a step that would fail it is left to the symbol loop, which makes
/// them.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1,bmi2")]
unsafe fn cur(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    use std::arch::x86_64::*;
    let max = b.max_len;
    ll.clear();
    ll.reserve(max);
    *counts = [0; 256];
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    // the list at store[32..288], with slack before it for the shifted load, held as the
    // bytes themselves (each symbol's seq_to_unseq), not their indices: a move permutes
    // both alike, so the front is the byte with no lookup
    for i in 0..256 {
        store.0[32 + i] = b.seq_to_unseq[i];
    }
    let base = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    // symbol s's byte (s >= 2: index s - 1 < 256, a symbol below the alphabet, at most
    // 257) moved to the front
    macro_rules! move_front {
        ($s:expr) => {{
            let s = $s as usize;
            let v = *base.add(s - 1);
            if s <= 32 {
                let old = _mm256_load_si256(base as *const __m256i);
                let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
                let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(s as i8), idx); // i <= s - 1
                _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
            } else {
                std::ptr::copy(base, base.add(1), s - 1);
            }
            *base = v;
            v
        }};
    }
    let syms = &b.syms[..];
    let (p, len, eob) = (syms.as_ptr(), syms.len(), b.eob);
    let (mut run, mut weight) = (0u32, 1u32);
    // `n` bytes written at `out`; every write below is checked against `max`, which
    // `ll`'s capacity holds
    let out = ll.as_mut_ptr();
    let mut n = 0usize;
    let mut result = Ok(());
    // the next symbol; whether the window loop is still to run
    let mut i = 0usize;
    let mut windows = true;
    while i < len {
        let s = *p.add(i);
        i += 1;
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
            let c = *base;
            counts[c as usize] += run;
            if run <= 64 && n + 64 <= max {
                // a short run as two 32-byte stores of its byte: the bytes past the run are
                // still inside `ll`'s capacity (n + 64 <= max), and are written again by
                // what follows or left past the length set at the end
                let v = _mm256_set1_epi8(c as i8);
                _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
            } else {
                std::ptr::write_bytes(out.add(n), c, run as usize);
            }
            n += run as usize;
            (run, weight) = (0, 1);
        }
        if s == eob {
            break;
        }
        if n >= max {
            result = Err(ErrorKind::BlockTooLong);
            break;
        }
        let v = move_front!(s);
        if !windows {
            counts[v as usize] += 1;
            *out.add(n) = v;
            n += 1;
            continue;
        }
        // the window loop, from symbol i (a run's start), with v's byte at n to write
        windows = false;
        let mut v = v;
        let mut q = i;
        let lim = max.saturating_sub(64);
        'windows: while q + 64 <= len {
            let w = p.add(q);
            let (mut bytes, notruna, leave) = window64(w, eob);
            if leave {
                break;
            }
            // (bytes != 0: 64 symbols with no byte symbol would be 22 run symbols in a row)
            let mut prev = 0u32;
            loop {
                let j = bytes.trailing_zeros();
                bytes &= bytes - 1;
                // v's byte and the run of it: the run symbols' RUNB bits, then a 1
                let seg = _bzhi_u64(notruna >> prev, j + 1 - prev) as usize;
                if n + seg > lim {
                    q += prev as usize;
                    break 'windows;
                }
                if seg <= 64 {
                    let x = _mm256_set1_epi8(v as i8);
                    _mm256_storeu_si256(out.add(n) as *mut __m256i, x);
                    _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, x);
                } else {
                    std::ptr::write_bytes(out.add(n), v, seg);
                }
                counts[v as usize] += seg as u32;
                n += seg;
                v = move_front!(*w.add(j as usize));
                prev = j + 1;
                if bytes == 0 {
                    break;
                }
            }
            q += prev as usize;
        }
        // v's byte written; the symbol loop goes on from the run after it (q)
        counts[v as usize] += 1;
        *out.add(n) = v;
        n += 1;
        i = q;
    }
    // SAFETY: the first n bytes were written above, n <= max <= capacity
    ll.set_len(n);
    result.map(|()| n)
}


include!("../variant.rs");
include!("../hybrid.rs");

fn tsc() -> u64 {
    unsafe {
        std::arch::x86_64::_mm_lfence();
        let t = std::arch::x86_64::_rdtsc();
        std::arch::x86_64::_mm_lfence();
        t
    }
}

const NAMES: [&str; 3] = ["old", "v2", "v4 (worktree)"];

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let rounds: usize = a[2].parse().unwrap();
    let k = Kernels::detect();
    assert!(k.avx2);
    let mut sc = Scanner::new(SliceSource(&data));
    let mut blocks: Vec<Symbols> = Vec::new();
    while let Event::Block { mut symbols, .. } = sc.next(Vec::new()).unwrap() {
        symbols.syms.shrink_to_fit();
        blocks.push(symbols);
    }
    let na = NAMES.len();
    let nsym: usize = blocks.iter().map(|b| b.syms.len()).sum();
    let mut ll: Vec<Vec<u8>> = vec![Vec::new(); na];
    let mut c = vec![[0u32; 256]; na];
    let mut nbytes = 0usize;
    let run = |i: usize, b: &Symbols, ll: &mut Vec<u8>, c: &mut [u32; 256]| -> Result<usize, ErrorKind> {
        unsafe {
            match i {
                0 => old(b, ll, c),
                1 => variant::<false, 0, false, false>(b, ll, c),
                _ => mtf::mtf(b, ll, c, k),
            }
        }
    };
    for b in &blocks {
        let r: Vec<_> = (0..na).map(|i| run(i, b, &mut ll[i], &mut c[i])).collect();
        for i in 1..na {
            assert!(r[i] == r[0] && ll[i] == ll[0] && c[i] == c[0], "{} differs", NAMES[i]);
        }
        nbytes += r[0].clone().unwrap();
    }
    println!("{}: {} blocks, {nsym} symbols, {nbytes} ll bytes; all {na} kernels agree on every block", a[1], blocks.len());
    let mut ratios: Vec<Vec<f64>> = vec![Vec::new(); na];
    for r in 0..rounds {
        let mut t = vec![0u64; na];
        for (bi, b) in blocks.iter().enumerate() {
            for j in 0..na {
                let i = (j + bi + r) % na;
                let t0 = tsc();
                let _ = std::hint::black_box(run(i, b, &mut ll[i], &mut c[i]));
                t[i] += tsc() - t0;
            }
        }
        let line: Vec<String> = (0..na).map(|i| format!("{:.3}", t[i] as f64 / nsym as f64)).collect();
        println!("r{r:2} c/sym: {}", line.join(" "));
        for i in 0..na { ratios[i].push(t[i] as f64 / t[0] as f64); }
    }
    for i in 1..na {
        let v = &mut ratios[i];
        let lr: Vec<f64> = v.iter().map(|x| x.ln()).collect();
        let m = lr.iter().sum::<f64>() / lr.len() as f64;
        let sd = (lr.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (lr.len() - 1) as f64).sqrt();
        let se = sd / (lr.len() as f64).sqrt();
        v.sort_by(|p, q| p.partial_cmp(q).unwrap());
        println!("{:>20}/old: mean {:+.1}% ({:+.1}, {:+.1}), min {:.3}, max {:.3}", NAMES[i], (m.exp() - 1.0) * 100.0, ((m - 2.2 * se).exp() - 1.0) * 100.0, ((m + 2.2 * se).exp() - 1.0) * 100.0, v[0], v[v.len() - 1]);
    }
}
