/// The window loop with a choice a window: at most T run symbols (sparse) take steps whose
/// segment is a byte store when it is one byte (a branch that sparse windows predict), and
/// (SPARSE_MEMMOVE) whose move from index 32 is a memmove; denser windows take v2's
/// branch-free steps (BIG_MEMSET: a segment above 4096 a memset)
#[target_feature(enable = "avx2,bmi1,bmi2")]
unsafe fn hybrid<const T: u32, const SPARSE_MEMMOVE: bool, const BIG_MEMSET: bool>(
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
    for i in 0..256 {
        store.0[32 + i] = b.seq_to_unseq[i];
    }
    let base = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    macro_rules! blend0 {
        ($nn:expr) => {{
            let old = _mm256_load_si256(base as *const __m256i);
            let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8($nn as i8 + 1), idx);
            _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
        }};
    }
    // master's move: a blend below 32, else memmove
    macro_rules! move_mm {
        ($s:expr) => {{
            let nn = $s as usize - 1;
            let v = *base.add(nn);
            if nn < 32 {
                blend0!(nn);
            } else {
                std::ptr::copy(base, base.add(1), nn);
            }
            *base = v;
            v
        }};
    }
    // v2's move: the chunk loop, no call
    macro_rules! move_ck {
        ($s:expr) => {{
            let nn = $s as usize - 1;
            let v = *base.add(nn);
            let mut k = nn & !31;
            let p = base.add(k);
            let old = _mm256_load_si256(p as *const __m256i);
            let shifted = _mm256_loadu_si256(p.sub(1) as *const __m256i);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8((nn - k + 1) as i8), idx);
            _mm256_store_si256(p as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
            while k != 0 {
                k -= 32;
                let p = base.add(k);
                let shifted = _mm256_loadu_si256(p.sub(1) as *const __m256i);
                _mm256_store_si256(p as *mut __m256i, shifted);
            }
            *base = v;
            v
        }};
    }
    let syms = &b.syms[..];
    let (p, len, eob) = (syms.as_ptr(), syms.len(), b.eob);
    let (mut run, mut weight) = (0u32, 1u32);
    let out = ll.as_mut_ptr();
    let mut n = 0usize;
    let mut result = Ok(());
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
        let v = move_mm!(s);
        if !windows {
            counts[v as usize] += 1;
            *out.add(n) = v;
            n += 1;
            continue;
        }
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
            let mut prev = 0u32;
            if (!bytes).count_ones() <= T {
                loop {
                    let j = bytes.trailing_zeros();
                    bytes &= bytes - 1;
                    let seg = _bzhi_u64(notruna >> prev, j + 1 - prev) as usize;
                    if n + seg > lim {
                        q += prev as usize;
                        break 'windows;
                    }
                    if seg == 1 {
                        *out.add(n) = v;
                    } else if seg <= 64 {
                        let x = _mm256_set1_epi8(v as i8);
                        _mm256_storeu_si256(out.add(n) as *mut __m256i, x);
                        _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, x);
                    } else {
                        std::ptr::write_bytes(out.add(n), v, seg);
                    }
                    counts[v as usize] += seg as u32;
                    n += seg;
                    v = if SPARSE_MEMMOVE { move_mm!(*w.add(j as usize)) } else { move_ck!(*w.add(j as usize)) };
                    prev = j + 1;
                    if bytes == 0 {
                        break;
                    }
                }
            } else {
                loop {
                    let j = bytes.trailing_zeros();
                    bytes &= bytes - 1;
                    let seg = _bzhi_u64(notruna >> prev, j + 1 - prev) as usize;
                    if n + seg > lim {
                        q += prev as usize;
                        break 'windows;
                    }
                    let x = _mm256_set1_epi8(v as i8);
                    if BIG_MEMSET && seg > 4096 {
                        std::ptr::write_bytes(out.add(n), v, seg);
                    } else {
                        _mm256_storeu_si256(out.add(n) as *mut __m256i, x);
                        _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, x);
                        let mut k = 64;
                        while k < seg {
                            _mm256_storeu_si256(out.add(n + k) as *mut __m256i, x);
                            k += 32;
                        }
                    }
                    counts[v as usize] += seg as u32;
                    n += seg;
                    v = move_ck!(*w.add(j as usize));
                    prev = j + 1;
                    if bytes == 0 {
                        break;
                    }
                }
            }
            q += prev as usize;
        }
        counts[v as usize] += 1;
        *out.add(n) = v;
        n += 1;
        i = q;
    }
    ll.set_len(n);
    result.map(|()| n)
}
