    #[cfg(target_arch="x86_64")]
    #[inline(always)]
    unsafe fn locate8(&self, lo: &[u32;8]) -> [usize;8] {
        use std::arch::x86_64::*;
        // The caller dispatches AVX2+POPCNT once per complete discovery. All
        // indices fit signed i32 after the logical shift by six.
        unsafe {
            let x=_mm256_loadu_si256(lo.as_ptr().cast());
            let pages=_mm256_srli_epi32::<6>(x);
            let common=self.starts.len().min(self.page_first.len());
            let max_page=i32::try_from(common).unwrap()-1;
            let outside=_mm256_cmpgt_epi32(pages,_mm256_set1_epi32(max_page));
            assert_eq!(_mm256_movemask_epi8(outside),0);
            // Both indexed arrays cover every gathered page. Loads use
            // natural page element widths and no out-of-range padding.
            let base_vec=_mm256_i32gather_epi32::<4>(self.page_first.as_ptr().cast(),pages);
            let low_mask=_mm256_i32gather_epi64::<8>(self.starts.as_ptr().cast(),_mm256_castsi256_si128(pages));
            let high_mask=_mm256_i32gather_epi64::<8>(self.starts.as_ptr().cast(),_mm256_extracti128_si256::<1>(pages));
            let shift=_mm256_andnot_si256(x,_mm256_set1_epi32(63));
            let low=_mm256_sllv_epi64(low_mask,_mm256_cvtepu32_epi64(_mm256_castsi256_si128(shift)));
            let high=_mm256_sllv_epi64(high_mask,_mm256_cvtepu32_epi64(_mm256_extracti128_si256::<1>(shift)));
            let bases:[u32;8]=std::mem::transmute(base_vec);
            let mut totals=[0u64;8];
            for (part,values) in [low,high].into_iter().enumerate() {
                let words:[u64;4]=if RANK==3 {
                    let lut=_mm256_broadcastsi128_si256(_mm_setr_epi8(0,1,1,2,1,2,2,3,1,2,2,3,2,3,3,4));
                    let nibble=_mm256_set1_epi8(15);
                    let a=_mm256_shuffle_epi8(lut,_mm256_and_si256(values,nibble));
                    let b=_mm256_shuffle_epi8(lut,_mm256_and_si256(_mm256_srli_epi16::<4>(values),nibble));
                    std::mem::transmute(_mm256_sad_epu8(_mm256_add_epi8(a,b),_mm256_setzero_si256()))
                } else {
                    let masked:[u64;4]=std::mem::transmute(values);
                    masked.map(|v|u64::from(v.count_ones()))
                };
                totals[part*4..part*4+4].copy_from_slice(&words);
            }
            std::array::from_fn(|i|bases[i] as usize + totals[i] as usize)
        }
    }
