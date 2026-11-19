use std::arch::x86_64::*;

#[derive(Default)]
pub struct Output{
    pub records:Vec<u16>,
    run_bits:Vec<u64>,
    pub syms:usize,
}
impl Output{
    fn reset(&mut self,upper:usize,limit:usize){
        self.records.clear();self.records.reserve(upper.min(limit/2)+limit/127+2);
        self.run_bits.clear();self.run_bits.reserve(upper.min(limit/2).div_ceil(64));self.syms=0;
    }
    fn mask(&mut self,bits:u64,n:usize){
        if n==0{return;}
        let off=self.syms%64;
        if off==0{self.run_bits.push(bits);}else{
            *self.run_bits.last_mut().unwrap()|=bits<<off;
            if n>64-off {self.run_bits.push(bits>>(64-off));}
        }
        self.syms+=n;
    }
    fn count(&self,start:usize,n:usize)->usize{
        let k=start/64;let shift=start%64;let mut b=self.run_bits[k]>>shift;
        if shift!=0 && n>64-shift{b|=self.run_bits[k+1]<<(64-shift);}
        if n<64{b&=(1u64<<n)-1;} b.count_ones()as usize
    }
    fn admitted(&self,limit:usize)->bool{
        if limit>=1<<24 || self.syms*2>limit{return false;}
        if self.syms<=4096{
            let count=self.run_bits.iter().map(|x|x.count_ones() as usize).sum::<usize>();
            32*count>=self.syms
        }else{
            let step=self.syms/64;let count=(0..64).map(|i|self.count(i*step,64)).sum::<usize>();count*32>=4096
        }
    }
    pub fn scratch_bytes(&self)->usize{self.records.capacity()*2+self.run_bits.capacity()*8}
}
#[repr(align(64))]struct Tail([u8;224]);
struct Front{
    f0:__m128i,f1:__m128i,tail:Tail,idx:__m256i,
    counts:[u64;256],max_len:usize,big:usize,
    byte:u8,has_literal:bool,digits:u32,ones:u64,
}
impl Front{
    #[target_feature(enable="avx2,bmi1,bmi2,lzcnt,popcnt")]
    unsafe fn new(alphabet:&[u8;256],max_len:usize)->Self{
        let mut tail=Tail([0;224]);tail.0.copy_from_slice(&alphabet[32..]);
        Self{f0:_mm_loadu_si128(alphabet.as_ptr().cast()),f1:_mm_loadu_si128(alphabet.as_ptr().add(16).cast()),tail,
            idx:_mm256_setr_epi8(0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31),
            counts:[0;256],max_len,big:0,byte:alphabet[0],has_literal:false,digits:0,ones:0}
    }
    #[inline(always)]
    unsafe fn select(&mut self,s:usize)->u8{
        // A set literal bit is at least 2, and EOB was removed.
        if s<33{
            let t=STEPS.as_ptr().add(s-1);
            let a=_mm_load_si128((*t).a.as_ptr().cast());let b=_mm_load_si128((*t).b.as_ptr().cast());
            let c=_mm_load_si128((*t).c.as_ptr().cast());let d=_mm_load_si128((*t).d.as_ptr().cast());
            let first=_mm_or_si128(_mm_shuffle_epi8(self.f0,a),_mm_shuffle_epi8(self.f1,b));
            self.f1=_mm_or_si128(_mm_shuffle_epi8(self.f1,c),_mm_shuffle_epi8(self.f0,d));
            self.f0=first;
        }else{
            let v=second(self.tail.0.as_mut_ptr(),(s-1).min(255)-32,self.f1,self.idx);
            self.f1=_mm_alignr_epi8::<15>(self.f1,self.f0);
            self.f0=_mm_or_si128(_mm_slli_si128::<1>(self.f0),_mm_cvtsi32_si128(v as i32));
        }
        _mm_cvtsi128_si32(self.f0) as u8
    }
    #[inline(always)]
    fn emit(&mut self,byte:u8,w:usize,out:&mut Output)->bool{
        if w<=127{
            if w!=0{
                out.records.push((u16::from(byte)<<8)|w as u16);
                self.counts[usize::from(byte)]+=(1u64<<32)|w as u64;
            }
        }else{
            self.big+=w;if self.big>self.max_len{return false;}
            self.counts[usize::from(byte)]+=(w.div_ceil(127) as u64)<<32|w as u64;
            let mut left=w;
            while left>127{out.records.push((u16::from(byte)<<8)|127);left-=127;}
            out.records.push((u16::from(byte)<<8)|left as u16);
        }
        true
    }
    #[inline(always)]
    fn pending(&mut self,zeros:u32,ones:u64)->bool{
        if zeros+self.digits>21{return false;}
        self.ones|=ones<<self.digits;self.digits+=zeros;true
    }
    #[inline(always)]
    fn close(&mut self,out:&mut Output)->bool{
        let w=(1usize<<self.digits)+self.ones as usize-usize::from(!self.has_literal);
        self.ones=0;self.digits=0;self.emit(self.byte,w,out)
    }
    #[target_feature(enable="avx2,bmi1,bmi2,lzcnt,popcnt")]
    unsafe fn tile(&mut self,words:&[u16;864],used:usize,out:&mut Output)->bool{
        let mut base=0;
        while base<used{
            let count=(used-base).min(64);let p=words.as_ptr().add(base);
            let zero=_mm256_setzero_si256();let one=_mm256_set1_epi16(1);
            let pack=|x,y|_mm256_movemask_epi8(_mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16(x,y))) as u32 as u64;
            let classify=|i|{let x=_mm256_loadu_si256(p.add(i).cast());let y=_mm256_loadu_si256(p.add(i+16).cast());
                let runs=pack(_mm256_cmpeq_epi16(_mm256_min_epu16(x,one),x),_mm256_cmpeq_epi16(_mm256_min_epu16(y,one),y));
                let a=pack(_mm256_cmpeq_epi16(x,zero),_mm256_cmpeq_epi16(y,zero));(runs,runs^a)};
            let (r0,b0)=classify(0);let(r1,b1)=classify(32);
            let run=_bzhi_u64(r0|(r1<<32),count as u32);
            let bits=b0|(b1<<32);
            out.mask(run,count);
            let mut lit=_bzhi_u64(!run,count as u32);
            let first=(lit.trailing_zeros() as usize).min(count);
            if !self.pending(first as u32,_bzhi_u64(bits,first as u32)){return false;}
            if lit!=0{
                if !self.close(out){return false;}
                self.has_literal=true;
                loop{
                    let t=lit.trailing_zeros() as usize;lit&=lit-1;
                    let c=self.select(usize::from(*p.add(t)));
                    let after=(lit.trailing_zeros() as usize).min(count);
                    let gap=after-1-t;if gap>21{return false;}
                    let ones=_bzhi_u64(bits.wrapping_shr((t+1)as u32),gap as u32);
                    if lit==0{
                        self.byte=c;self.ones=ones; self.digits=gap as u32;break;
                    }
                    if !self.emit(c,(1usize<<gap)+ones as usize,out){return false;}
                }
            }
            base+=count;
        }
        out.syms*2<=self.max_len
    }
}
