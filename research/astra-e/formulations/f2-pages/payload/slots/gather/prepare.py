from pathlib import Path
from hashlib import sha256

here=Path(__file__).resolve().parent
s=(here.parent/'candidate.rs').read_text()
assert sha256(s.encode()).hexdigest() == '84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06'
s=s.replace('../../../f2c/second.rs','../../../../f2c/second.rs').replace('pub type Candidate = Dispatch<2>;', 'pub type Candidate = Dispatch<2,2,3>;')
s=s.replace('const SCHEDULE:u8=1>', 'const SCHEDULE:u8=2,const RANK:u8=3>').replace('impl<const MODE:u8,const SCHEDULE:u8>', 'impl<const MODE:u8,const SCHEDULE:u8,const RANK:u8>').replace('Dispatch<MODE,SCHEDULE>','Dispatch<MODE,SCHEDULE,RANK>')
a=s.index('        let mask = u64::MAX', s.index('    fn locate(')); b=s.index('\n    }',a)
s=s[:a]+'''        let payload=self.starts[page];
        let count=if RANK!=0 {
            payload.wrapping_shl(!x).count_ones()
        } else {
            (payload & (u64::MAX >> (63 - (x & 63)))).count_ones()
        };
        (self.page_first[page]+count) as usize'''+s[b:]
s=s.replace('    fn build(&mut self, quote: u32, tokens: u32) {','''    fn build(&mut self, quote: u32, tokens: u32) {
        #[cfg(target_arch="x86_64")]
        if RANK>=2 && std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("popcnt") {
            // SAFETY: both features required by this call have been detected.
            return unsafe { self.build_avx(quote,tokens) };
        }''')
s=s.replace('self.build_body(', 'self.build_body::<false>(')
s=s.replace('    #[inline(always)]\n    fn build_body(', '''    #[cfg(target_arch="x86_64")]
    #[target_feature(enable="avx2,popcnt")]
    unsafe fn build_avx(&mut self, quote:u32, tokens:u32) {
        self.build_body::<true>(quote,tokens)
    }

'''+(here/'lookup.inc').read_text()+'''
    #[inline(always)]
    fn build_body<const VECTOR:bool>(''')
a=s.index('                for lane in 0..LANES {\n                    index[lane] = self.locate(lo[lane]);')
s=s[:a]+s[a:].replace('''                for lane in 0..LANES {
                    index[lane] = self.locate(lo[lane]);
                }''','''                #[cfg(target_arch="x86_64")]
                if VECTOR {
                    // SAFETY: only build_avx instantiates VECTOR=true.
                    index=unsafe { self.locate8(&lo) };
                } else {
                    for lane in 0..LANES {index[lane]=self.locate(lo[lane]);}
                }
                #[cfg(not(target_arch="x86_64"))]
                for lane in 0..LANES {index[lane]=self.locate(lo[lane]);}''',1)
(here/'family.rs').write_text(s)
