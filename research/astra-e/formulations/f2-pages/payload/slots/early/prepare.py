"""Start known survivor work before publishing unrelated retiring lanes."""
from pathlib import Path
from hashlib import sha256
h=Path(__file__).resolve().parent
s=(h.parent/'candidate.rs').read_text()
assert sha256(s.encode()).hexdigest()=='84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06'
s=s.replace('path = "../../../f2c/','path = "../../../../f2c/')
s=s.replace('pub type Candidate = Dispatch<2>;', '''#[inline]
fn prefetch<T>(value:&T) {
    #[cfg(target_arch="x86_64")]
    // SAFETY: SSE is part of x86-64. The shared reference points to a live array element.
    unsafe {
        use std::arch::x86_64::{_mm_prefetch,_MM_HINT_T0};
        _mm_prefetch::<_MM_HINT_T0>(std::ptr::from_ref(value).cast());
    }
    #[cfg(not(target_arch="x86_64"))]
    let _=value;
}

pub type Candidate = Dispatch<2,2,2>;''')
s=s.replace('pub struct Dispatch<const MODE:u8,const SCHEDULE:u8=1>','pub struct Dispatch<const MODE:u8,const SCHEDULE:u8=2,const EARLY:u8=2>')
s=s.replace('impl<const MODE:u8,const SCHEDULE:u8>','impl<const MODE:u8,const SCHEDULE:u8,const EARLY:u8>')
s=s.replace('Dispatch<MODE,SCHEDULE>','Dispatch<MODE,SCHEDULE,EARLY>')
s=s.replace('        let mut live = 0;', '        let mut live = 0;\n        let mut ready=0u32;')
s=s.replace('            if live == ALL {\n                for lane in 0..LANES {\n                    index[lane] = self.locate(lo[lane]);',
'''            if live == ALL && (EARLY<2 || ready==0) {
                for lane in 0..LANES {
                    index[lane] = self.locate(lo[lane]);''')
s=s.replace('''                let mut mask = live;
                while mask != 0 {
                    let lane = mask.trailing_zeros() as usize;
                    mask &= mask - 1;
                    index[lane] = self.locate(lo[lane]);''','''                let mut mask = if EARLY>=2 {live & !ready} else {live};
                while mask != 0 {
                    let lane = mask.trailing_zeros() as usize;
                    mask &= mask - 1;
                    index[lane] = self.locate(lo[lane]);''')
a=s.index('            live ^= done;')+len('            live ^= done;')
s=s[:a]+'''
            ready=0;
            if done!=0 {
                let mut survivors=live;
                if EARLY>=2 {ready=live;}
                while survivors!=0 {
                    let lane=survivors.trailing_zeros() as usize;
                    survivors&=survivors-1;
                    if EARLY==1 {
                        let page=(lo[lane]>>6) as usize;
                        prefetch(&self.starts[page]);
                        prefetch(&self.page_first[page]);
                    } else {
                        let k=self.locate(lo[lane]);
                        index[lane]=k;
                        if EARLY==3 {prefetch(&self.runs[k]);}
                    }
                }
            }'''+s[a:]
s=s.replace('fn verify_marks<const SCHEDULE:u8>()','fn verify_marks<const EARLY:u8>()').replace('let mut c = Dispatch::<2,SCHEDULE>::default();','let mut c = Dispatch::<2,2,EARLY>::default();')
(h/'candidate.rs').write_text(s)
