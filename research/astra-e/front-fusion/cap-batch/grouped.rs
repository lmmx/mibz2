fn each_run(records: &[u16], p: u32, limit: u32, mut visit: impl FnMut(u8, u32, u32, bool)) {
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        unsafe { batch_avx2(records, p, limit, &mut visit); }
        return;
    }
    scalar(records, p, limit, visit)
}

#[inline(always)]
fn marked(c: u8, pos: u32, w: u32, p: u32, limit: u32, visit: &mut impl FnMut(u8,u32,u32,bool)) {
    if w > limit && (pos..pos + w).contains(&p) {
        if pos < p { visit(c, pos, p - pos, false); }
        visit(c, p, 1, true);
        if p + 1 < pos+w { visit(c, p+1, pos+w-p-1, false); }
    } else {
        visit(c, pos, w, w <= limit);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn batch_avx2(records: &[u16], p: u32, limit: u32, visit: &mut impl FnMut(u8, u32, u32, bool)) {
    use std::arch::x86_64::*;
    let mut i = 0;
    let mut pos = 0;
    while i < records.len() {
        let batch = (records.len()-i).min(16);
        let prefix = if batch == 16 {
            let v = _mm256_loadu_si256(records.as_ptr().add(i).cast());
            let width = _mm256_and_si256(v,_mm256_set1_epi16(255));
            let full = _mm256_cmpeq_epi16(width, _mm256_set1_epi16(127));
            (_mm256_movemask_epi8(full) as u32).trailing_zeros() as usize / 2
        } else { records[i..].iter().take_while(|v| (*v & 255) < 127).count() };
        let short = &records[i..i+prefix];
        if p.wrapping_sub(pos) >= 126*prefix as u32 {
            for &r in short {
                let w = u32::from(r & 255);
                visit((r >> 8) as u8, pos, w, w <= limit);
                pos += w;
            }
        } else {
            for &r in short {
                let w = u32::from(r & 255);
                marked((r >> 8) as u8, pos, w, p, limit, visit);
                pos += w;
            }
        }
        i += prefix;
        if prefix < batch {
            let byte = records[i] >> 8;
            let mut w = 0;
            while i < records.len() && records[i] >> 8 == byte {
                w += u32::from(records[i] & 255);
                i += 1;
            }
            marked(byte as u8, pos, w, p, limit, visit);
            pos += w;
        }
    }
}
