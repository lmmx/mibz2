// Research worker path. A consumed run array supplies the edit directory.
#[derive(Clone, Copy)]
struct Edit { src: usize, dst: usize, byte: u8, extra: u8 }
impl Edit {
    fn put(self, slot: &mut [u16]) {
        let bits = self.src as u64 | ((self.dst as u64) << 20) |
            ((self.byte as u64) << 48) | ((self.extra as u64) << 56);
        for (i, v) in slot.iter_mut().enumerate() { *v = (bits >> (16*i)) as u16; }
    }
    fn get(slot: &[u16]) -> Self {
        let bits = u64::from(slot[0]) | (u64::from(slot[1]) << 16) |
            (u64::from(slot[2]) << 32) | (u64::from(slot[3]) << 48);
        Self {src: (bits & 0xfffff) as usize, dst: (bits >> 20 & 0xfffffff) as usize,
              byte: (bits >> 48) as u8, extra: (bits >> 56) as u8}
    }
}

// Same four unaligned input vectors and 32 candidate starts as the shipped AVX2 RLE1.
#[target_feature(enable="avx2,bmi1")]
unsafe fn next_quad(s: &[u8], mut i: usize) -> Option<usize> {
    use std::arch::x86_64::*;
    while i + 35 <= s.len() {
        let p = s.as_ptr().add(i);
        let v = _mm256_loadu_si256(p.cast());
        let eq = |j| _mm256_cmpeq_epi8(v, _mm256_loadu_si256(p.add(j).cast()));
        let mask = _mm256_movemask_epi8(_mm256_and_si256(eq(1), _mm256_and_si256(eq(2),eq(3)))) as u32;
        if mask != 0 { return Some(i + mask.trailing_zeros() as usize); }
        i += 32;
    }
    s[i..].windows(4).position(|w| w[0] == w[1] && w[0] == w[2] && w[0] == w[3]).map(|j| i+j)
}

fn repair(data: &mut Vec<u8>, directory: &mut [u16]) -> Result<bool, ErrorKind> {
    let n = data.len();
    if n >= 1<<20 { return Ok(false); }
    let (mut src, mut dst, mut count) = (0, 0, 0);
    astra_mark(12);
    // No change to any encoded byte until the directory and output capacity are known.
    while let Some(j) = unsafe { next_quad(data, src) } {
        let extra = *data.get(j+4).ok_or(ErrorKind::RunUnfinished)?;
        let Some(slot) = directory.get_mut(4*count..4*count+4) else { return Ok(false); };
        let q = dst + (j-src);
        // For < 2^20 encoded bytes, 259 decoded bytes per five inputs fit 28 bits.
        Edit {src:j, dst:q, byte:data[j], extra}.put(slot);
        src = j+5;
        dst = q + 4 + extra as usize;
        count += 1;
    }
    let m = dst + (n-src);
    if m > data.capacity() { return Ok(false); }
    if m > n { data.resize(m, 0); }
    let entries = directory[..4*count].chunks_exact(4);
    astra_mark(13);
    src = 0;
    dst = 0;
    for e in entries.clone().map(Edit::get) {
        if dst < src { data.copy_within(src..e.src,dst); }
        src = e.src+5;
        dst = e.dst+4+e.extra as usize;
    }
    if dst < src { data.copy_within(src..n,dst); }
    astra_mark(14);
    let mut end = n;
    for e in entries.clone().rev().map(Edit::get) {
        let src = e.src+5;
        let dst = e.dst+4+e.extra as usize;
        if dst > src { data.copy_within(src..end,dst); }
        end = e.src;
    }
    astra_mark(15);
    for e in entries.map(Edit::get) {
        data[e.dst..e.dst+4+e.extra as usize].fill(e.byte);
    }
    data.truncate(m);
    Ok(true)
}

fn finish_slot<I: bwt::InverseBwt>(s: &mut Scratch<I>, out: &mut Vec<u8>,
                                   stored: u32, k: Kernels) -> Result<u32,ErrorKind> {
    match repair(out, s.runs.astra_directory()) {
        Ok(true) => {
            astra_mark(10);
            let computed = crc::crc32(out,k);
            if computed == stored { Ok(computed) }
            else { out.clear(); Err(ErrorKind::BlockCrc{stored,computed}) }
        }
        Ok(false) => {
            s.ll.clear();
            s.ll.extend_from_slice(out);
            out.clear();
            finish(&s.ll,stored,k,out)
        }
        Err(err) => { out.clear(); Err(err) }
    }
}
