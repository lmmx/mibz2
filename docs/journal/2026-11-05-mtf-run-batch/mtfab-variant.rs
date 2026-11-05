/// Window-loop variants (research): DENSE: a window with no run symbol takes byte steps
/// (a byte store each, no segment); MEMMOVE: an index from 32 moves by memmove (master's),
/// not the chunk loop; HALF: a segment of at most 32 is one 32-byte store (a second only
/// above 32); MEMSET: a segment above 1024 is a memset call, not the 32-byte loop
#[target_feature(enable = "avx2,bmi1,bmi2")]
unsafe fn variant<const DENSE: bool, const MV: u8, const HALF: bool, const MEMSET: bool>(
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
    macro_rules! move_front {
        ($s:expr) => {{
            let nn = $s as usize - 1;
            let v = *base.add(nn);
            if MV == 2 {
                if nn < 32 {
                    let old = _mm256_load_si256(base as *const __m256i);
                    let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
                    let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(nn as i8 + 1), idx);
                    _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
                } else {
                    // all eight chunks, each blended by i <= nn - 32k (saturated), no branch:
                    // loads first, then stores, so each shifted load reads the old list
                    let lim8 = _mm256_set1_epi8(0);
                    let _ = lim8;
                    let mut o = [_mm256_setzero_si256(); 8];
                    let mut sh = [_mm256_setzero_si256(); 8];
                    for c in 0..8 {
                        o[c] = _mm256_load_si256(base.add(32 * c) as *const __m256i);
                        sh[c] = _mm256_loadu_si256(base.add(32 * c).sub(1) as *const __m256i);
                    }
                    for c in 0..8 {
                        // i <= nn - 32c, as a signed byte compare on the clamped bound
                        let t = (nn as i32 - 32 * c as i32 + 1).clamp(0, 32) as i8;
                        let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(t), idx);
                        _mm256_store_si256(base.add(32 * c) as *mut __m256i, _mm256_blendv_epi8(o[c], sh[c], m));
                    }
                }
            } else if MV == 1 {
                if nn < 32 {
                    let old = _mm256_load_si256(base as *const __m256i);
                    let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
                    let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(nn as i8 + 1), idx);
                    _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
                } else {
                    std::ptr::copy(base, base.add(1), nn);
                }
            } else {
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
        let v = move_front!(s);
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
            if DENSE && bytes == !0 {
                // 64 byte symbols: 64 bytes after the pending one, all before lim
                if n + 65 > lim {
                    break;
                }
                for j in 0..64 {
                    counts[v as usize] += 1;
                    *out.add(n) = v;
                    n += 1;
                    v = move_front!(*w.add(j));
                }
                q += 64;
                continue;
            }
            let mut prev = 0u32;
            loop {
                let j = bytes.trailing_zeros();
                bytes &= bytes - 1;
                let seg = _bzhi_u64(notruna >> prev, j + 1 - prev) as usize;
                if n + seg > lim {
                    q += prev as usize;
                    break 'windows;
                }
                let x = _mm256_set1_epi8(v as i8);
                if MEMSET && seg > 1024 {
                    std::ptr::write_bytes(out.add(n), v, seg);
                } else {
                    _mm256_storeu_si256(out.add(n) as *mut __m256i, x);
                    if !HALF || seg > 32 {
                        _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, x);
                        let mut k = 64;
                        while k < seg {
                            _mm256_storeu_si256(out.add(n + k) as *mut __m256i, x);
                            k += 32;
                        }
                    }
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
        counts[v as usize] += 1;
        *out.add(n) = v;
        n += 1;
        i = q;
    }
    ll.set_len(n);
    result.map(|()| n)
}
