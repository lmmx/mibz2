#[derive(Clone, Copy)]
#[repr(C)]
struct BucketGroup {
    base_width: u32,
    bit_start: u32,
    high_positions: u64,
}

#[derive(Default)]
struct BucketResidual {
    allowed: bool,
    used: bool,
    starts: Vec<u64>,
    prefix: Vec<u16>,
    labels: Vec<u32>,
    groups: Vec<BucketGroup>,
    lows: Vec<u64>,
}

// Even if every slot uses full-width raw ranks, the live final layout fits in
// the incumbent edge budget. Shared queues and retained Vec capacities are separate.
fn bucket_bound(m: usize, bits: u32, bands: usize) -> usize {
    10 * m.div_ceil(64) + 4 * bands + 16 * m.div_ceil(32) + 8 * (1+(m*bits as usize).div_ceil(64))
}
fn bucket_features() -> bool {
    #[cfg(target_arch="x86_64")]
    { std::is_x86_feature_detected!("bmi2") && std::is_x86_feature_detected!("popcnt") }
    #[cfg(not(target_arch="x86_64"))]
    { false }
}

struct BucketBits {
    carry: u64,
    used: u32,
    written: u32,
}
impl BucketBits {
    #[inline]
    fn push(&mut self, value: u32, width: u32, words: &mut Vec<u64>) {
        self.carry |= u64::from(value) << self.used;
        let next = self.used + width;
        if next >= 64 {
            words.push(self.carry);
            // Reaching 64 requires used>0 since width<=20.
            self.carry = u64::from(value) >> (64 - self.used);
        }
        self.used = next & 63;
        self.written += width;
    }
}

impl BucketResidual {
    fn reset(&mut self) {
        self.starts.clear();
        self.prefix.clear();
        self.labels.clear();
        self.groups.clear();
        self.lows.clear();
    }

    fn band(&self, row: usize) -> usize {
        let at = row >> 6;
        let through = u64::MAX >> (63 - (row & 63));
        (u32::from(self.prefix[at]) + (self.starts[at] & through).count_ones() - 1) as usize
    }

    #[inline]
    fn put_group(&mut self, q: &[u32], global_bits: u32, sorted: bool, sink: &mut BucketBits) {
        let base = if sorted { q[0] } else { 0 };
        let width = if sorted {
            // Smallest width w with floor(span/2^w)+group_length<=64.
            let span = q[q.len()-1] - base;
            let high_bound = if q.len()==32 { span / 33 } else { span / (65-q.len()) as u32 };
            u32::BITS - high_bound.leading_zeros()
        } else { global_bits };
        let mask = (1u32 << width)-1;
        let at = sink.written;
        let mut high = 0;
        if sorted {
            for (j,&rank) in q.iter().enumerate() {
                let delta = rank-base;
                high |= 1u64 << (j + (delta >> width) as usize);
                sink.push(delta & mask, width, &mut self.lows);
            }
        } else {
            for &rank in q {sink.push(rank, width, &mut self.lows);}
        }
        self.groups.push(BucketGroup {base_width: base | (width << global_bits), bit_start: at, high_positions: high});
    }

    fn build(&mut self, bands: &[Band], queues: &[Vec<u32>], m: usize, bits: u32) {
        self.reset();
        self.starts.resize(m.div_ceil(64),0);
        let mut k = 0u32;
        for band in bands {
            let dst = k.wrapping_add(band.delta) as usize;
            self.starts[dst >> 6] |= 1u64 << (dst & 63);
            k=band.end;
        }
        self.prefix.reserve(self.starts.len());
        let mut count = 0u32;
        for &mask in &self.starts {
            self.prefix.push(count as u16);
            count += mask.count_ones();
        }
        debug_assert_eq!(count as usize,bands.len());
        self.labels.resize(bands.len(),0);
        let low = (1u32 << bits)-1;
        k=0;
        for band in bands {
            let dst = k.wrapping_add(band.delta) as usize;
            let slot = self.band(dst);
            self.labels[slot] = (band.code << bits) | (k.wrapping_sub(dst as u32) & low);
            k=band.end;
        }

        self.groups.reserve(m.div_ceil(32));
        self.lows.reserve(m.div_ceil(16));
        let mut sink = BucketBits {carry:0, used:0, written:0};
        let mut crossing = [0u32;32];
        let mut filled = 0;
        for queue in queues {
            let mut q=queue.as_slice();
            if filled>0 {
                let take=(32-filled).min(q.len());
                crossing[filled..filled+take].copy_from_slice(&q[..take]);
                q=&q[take..]; filled+=take;
                if filled<32 {continue;}
                self.put_group(&crossing,bits,crossing.windows(2).all(|w|w[0]<w[1]),&mut sink);
                filled=0;
            }
            let mut chunks=q.chunks_exact(32);
            for group in &mut chunks {self.put_group(group,bits,true,&mut sink);}
            let tail=chunks.remainder();
            if !tail.is_empty() {
                crossing[..tail.len()].copy_from_slice(tail);
                filled=tail.len();
            }
        }
        if filled>0 {
            let q=&crossing[..filled];
            self.put_group(q,bits,q.windows(2).all(|w|w[0]<w[1]),&mut sink);
        }
        if sink.used>0 {self.lows.push(sink.carry);}
        // Allows one unaligned u32 read from the last bit offset, including a zero-width group.
        self.lows.push(0);
        debug_assert_eq!(self.groups.len(),m.div_ceil(32));
    }

    fn arrays(&self) -> [[usize;2];5] {
        [
            [8*self.starts.len(),8*self.starts.capacity()],
            [2*self.prefix.len(),2*self.prefix.capacity()],
            [4*self.labels.len(),4*self.labels.capacity()],
            [std::mem::size_of::<BucketGroup>()*self.groups.len(),std::mem::size_of::<BucketGroup>()*self.groups.capacity()],
            [8*self.lows.len(),8*self.lows.capacity()],
        ]
    }

    fn live(&self)->usize {self.arrays().iter().map(|x|x[0]).sum()}

    #[cfg(target_arch="x86_64")]
    #[inline(always)]
    unsafe fn edge(&self,row:u32,bits:u32,mask:u32)->(u32,u32) {
        let ordinal=self.band(row as usize);
        let label=self.labels[ordinal];
        let k=row.wrapping_add(label)&mask;
        let j=k&31;
        let h=self.groups[(k >> 5) as usize];
        let w=h.base_width>>bits;
        let p=h.bit_start+j*w;
        // x86_64 is little endian. Every group has 1..32 entries and the u64 sentinel
        // covers even a zero-width final group. At most 20+7 significant bits are read.
        let value=unsafe{self.lows.as_ptr().cast::<u8>().add((p>>3) as usize).cast::<u32>().read_unaligned()};
        let low=(value>>(p&7))&((1u32<<w)-1);
        let hi=if h.high_positions!=0 {
            (std::arch::x86_64::_pdep_u64(1u64<<j,h.high_positions).trailing_zeros()-j)<<w
        } else {0};
        ((h.base_width&mask)+hi+low,label>>bits)
    }
}

impl Kernel {
    pub fn displacement_entry(&self, row:usize, bits:u32) -> (u32,u32) {
        let mask=(1u32<<bits)-1;
        if self.bucket.used {
            #[cfg(target_arch="x86_64")]
            {return unsafe {self.bucket.edge(row as u32,bits,mask)};}
            #[cfg(not(target_arch="x86_64"))]
            unreachable!();
        }
        let e=self.edges[row];
        (e&mask, if self.extra.is_empty() {e>>bits} else {self.extra[row]})
    }

    pub fn displacement_arrays(&self) -> [[usize;2];7] {
        let [mask,prefix,labels,groups,lows]=self.bucket.arrays();
        [mask,prefix,labels,groups,lows,
         [4*self.edges.len(),4*self.edges.capacity()],
         [4*self.extra.len(),4*self.extra.capacity()]]
    }
    pub fn displacement_storage(&self) -> [usize;4] {
        let arrays=self.displacement_arrays();
        [arrays.iter().map(|a|a[0]).sum(),arrays.iter().map(|a|a[1]).sum(),
         self.spelling.len()+4*self.offsets.len(),
         self.spelling.capacity()+4*self.offsets.capacity()]
    }

    fn emit_bucket(&self,n:usize,mark:u32,bits:u32,out:&mut Vec<u8>) {
        #[cfg(target_arch="x86_64")]
        unsafe {self.emit_bucket_bmi2(n,mark,bits,out);}
        #[cfg(not(target_arch="x86_64"))]
        unreachable!();
    }

    #[cfg(target_arch="x86_64")]
    #[target_feature(enable="bmi2,popcnt")]
    unsafe fn emit_bucket_bmi2(&self,n:usize,mark:u32,bits:u32,out:&mut Vec<u8>) {
        out.clear();
        out.reserve(n);
        let mask=(1u32<<bits)-1;
        let mut edge=unsafe{self.bucket.edge(mark,bits,mask)};
        loop {
            let (row,code)=edge;
            edge=unsafe{self.bucket.edge(row,bits,mask)};
            append_code(code,&self.spelling,&self.offsets,out);
            if row==mark {break;}
        }
        repeat(n,out);
    }
}
