"""Compare singleton recognition in original records with split hot and cold payloads."""
from pathlib import Path
from hashlib import sha256
import re
h=Path(__file__).resolve().parent
source=(h.parent/'candidate.rs').read_text()
assert sha256(source.encode()).hexdigest()=='84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06'
for split,file in [(False,'interleaved.rs'),(True,'candidate.rs')]:
    s=source.replace('path = "../../../f2c/','path = "../../../../f2c/')
    s=s.replace('pub type Candidate = Dispatch<2>;','pub type Candidate = Dispatch<2,2,true>;')
    s=s.replace('pub struct Dispatch<const MODE:u8,const SCHEDULE:u8=1>', 'pub struct Dispatch<const MODE:u8,const SCHEDULE:u8=2,const SINGLE:bool=true>')
    s=s.replace('impl<const MODE:u8,const SCHEDULE:u8>', 'impl<const MODE:u8,const SCHEDULE:u8,const SINGLE:bool>')
    s=s.replace('Dispatch<MODE,SCHEDULE>', 'Dispatch<MODE,SCHEDULE,SINGLE>')
    if split:
        a=s.index('#[repr(C, align(8))]');b=s.index('#[derive(Clone, Copy, Default)]\nstruct Piece',a)
        s=s[:a]+'''#[repr(transparent)]
#[derive(Clone,Copy,Default)]
struct Run(u32);
impl Run {
    #[inline]
    fn new(byte:u8,delta:u32)->Self { Self(((delta & ((1<<20)-1)) << 8) | u32::from(byte)) }
    #[inline]
    fn byte(self)->u8 { self.0 as u8 }
    #[inline]
    fn map(self,x:u32)->u32 { (x+(self.0>>8)) & ((1<<20)-1) }
}
'''+s[b:]
        s=s.replace('    runs: Vec<Run>,','    runs: Vec<Run>,\n    ends: Vec<u32>,').replace('        self.runs.clear();','        self.runs.clear();\n        self.ends.clear();')
        s=s.replace('self.runs.last_mut().unwrap().position_byte += len << 8;', '*self.ends.last_mut().unwrap() += len;')
        s=s.replace('self.runs.push(Run::new(end,byte,delta));','self.runs.push(Run::new(byte,delta));\n                self.ends.push(end);')
        end='self.ends[k]'
        cond='self.ends[k]<lo+w'
        s=s.replace('assert_eq!(size_of::<Run>(),8);','assert_eq!(size_of::<Run>(),4);').replace('assert_eq!(align_of::<Run>(),8);','assert_eq!(align_of::<Run>(),4);')
        s=s.replace('x>=c.runs[k-1].position()', 'x>=c.ends[k-1]').replace('x<r.position()', 'x<c.ends[k]')
    else:
        end='r.position()'
        cond='r.position_byte < ((lo+w)<<8)'
    a=s.index('    fn transfer_mark(')
    s=s[:a]+'''    #[inline]
    fn ends_path(&self, r:Run, k:usize, lo:u32, w:u32)->bool {
        let _=k;
        r.byte()==QUOTE || ((!SINGLE || w!=1) && (COND))
    }

'''.replace('COND',cond)+s[a:]
    s=re.sub(r'(r|replies\[lane\])\.retires\(lo\[lane\]\+width\[lane\]\)',r'self.ends_path(\1,index[lane],lo[lane],width[lane])',s)
    s=s.replace('                    let end=hi.min(r.position());',f'                    let end=if SINGLE && hi-a==1 {{hi}} else {{hi.min({end})}};')
    (h/file).write_text(s)
