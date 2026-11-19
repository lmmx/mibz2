from pathlib import Path
from hashlib import sha256
here=Path(__file__).resolve().parent
s=(here.parent/'candidate.rs').read_text()
assert sha256(s.encode()).hexdigest()=='84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06'
s=s.replace('../../../f2c/second.rs','../../../../f2c/second.rs').replace('pub type Candidate = Dispatch<2>;', 'pub type Candidate = Dispatch<2,2,3>;')
s=s.replace('const SCHEDULE:u8=1>', 'const SCHEDULE:u8=2,const CACHE:u8=3>').replace('impl<const MODE:u8,const SCHEDULE:u8>', 'impl<const MODE:u8,const SCHEDULE:u8,const CACHE:u8>').replace('Dispatch<MODE,SCHEDULE>','Dispatch<MODE,SCHEDULE,CACHE>')
s=s.replace('    fallback: Segmented,','    next_target:Vec<u32>,\n    #[cfg(prediction_counts)]\n    counters:[u64;3],\n    fallback: Segmented,')
a=s.index('    #[inline]\n    fn retires(');b=s.index('\n}\n',a)
s=s[:a]+'''    #[inline]
    fn retires<const CACHE:u8>(self,hi:u32)->bool {
        let field=if CACHE&1!=0 {self.position_byte & 0x0fff_ffff} else {self.position_byte};
        field<(hi<<8) || self.byte()==QUOTE
    }
    #[inline]
    fn new<const CACHE:u8>(pos:u32, byte:u8, delta:u32)->Self {
        let delta=if CACHE&1!=0 {delta & 0x000f_ffff} else {delta};
        Self {position_byte:(pos<<8) | u32::from(byte),delta}
    }
    #[inline]
    fn position<const CACHE:u8>(self)->u32 {
        let p=self.position_byte>>8;
        if CACHE&1!=0 {p & 0x000f_ffff} else {p}
    }
    #[inline]
    fn byte(self)->u8 {self.position_byte as u8}
    #[inline]
    fn map<const CACHE:u8>(self,row:u32)->u32 {
        let x=row.wrapping_add(self.delta);
        if CACHE&1!=0 {x & 0x000f_ffff} else {x}
    }
    #[inline]
    fn prediction(self)->u32 {(self.position_byte>>28)<<12 | (self.delta>>20)}
    #[inline]
    fn set_prediction(&mut self,code:u32) {
        self.position_byte=(self.position_byte & 0x0fff_ffff) | ((code>>12)<<28);
        self.delta=(self.delta & 0x000f_ffff) | (code<<20);
    }'''+s[b:]
start=s.index('impl<const MODE:u8,const SCHEDULE:u8,const CACHE:u8>')
end=s.index('#[cfg(test)]\nmod checks',start)
body=s[start:end]
body=body.replace('Run::new(','Run::new::<CACHE>(').replace('.position()','.position::<CACHE>()').replace('.retires(','.retires::<CACHE>(')
# The count array's map is independent of Run::map.
body=body.replace('r.map(', 'r.map::<CACHE>(')
body=body.replace('        let mut job = [0usize; LANES];','''        if CACHE==2 {
            self.next_target.clear();
            self.next_target.resize(self.runs.len(),0);
        }
        #[cfg(prediction_counts)] {self.counters=[0;3];}
        let upper_range=self.runs.get(65534).map(|r|r.position::<CACHE>()).unwrap_or(u32::MAX);
        let mut from=[usize::MAX;LANES];
        let mut prediction=[0u32;LANES];
        let mut job = [0usize; LANES];''')
body=body.replace('                job[lane] = next;','                from[lane]=usize::MAX;\n                prediction[lane]=0;\n                job[lane] = next;')
a=body.index('            if live == ALL {')
body=body[:a]+(here/'lookup.inc').read_text()+body[a:]
tests=s[end:].replace('r.map(', 'r.map::<3>(').replace('.position()', '.position::<3>()')
s=s[:start]+body+tests
(here/'family.rs').write_text(s)
