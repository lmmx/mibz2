"""Implement mask-first and page-ID uniform-run replies on the same compact source."""
from pathlib import Path
from hashlib import sha256
here=Path(__file__).resolve().parent
s=(here.parent/'construction/eager.rs').read_text()
assert sha256(s.encode()).hexdigest()=='82704548e38df71d53eb7eab1b12674d62b0987d7fc789c5d1bc00cdff6213dc'
s=s.replace('pub struct Dispatch<const MODE: u8>', 'pub struct Dispatch<const MODE:u8,const PAGE_ID:bool=false>')
s=s.replace('impl<const MODE:u8>', 'impl<const MODE:u8,const PAGE_ID:bool>')
s=s.replace('Dispatch<MODE>', 'Dispatch<MODE,PAGE_ID>')
a=s.index('    #[inline]\n    fn locate(')
b=s.index('    #[inline(always)]\n    fn prepare_body',a)
s=s[:a]+(here/'reply.inc.rs').read_text()+'\n'+s[b:]
s=s.replace('            *mask &= !1;', '''            *mask &= !1;
            if *mask==0 {
                let r=self.runs[*base as usize];
                *mask=(u64::from(r.delta)<<32) | (u64::from(r.position_byte)<<1) | 1;
                if PAGE_ID { *base |= 1u32<<31; }
            }''')
s=s.replace('let r = self.runs[self.locate(x)];','let r = self.read_reply(self.reply(x));')
s=s.replace('let mut index = [0usize; LANES];','let mut index = [0u64; LANES];')
s=s.replace('index[lane] = self.locate(lo[lane]);','index[lane] = self.reply(lo[lane]);')
s=s.replace('let k = index[lane];\n                let r = self.runs[k];','let r=self.read_reply(index[lane]);')
s=s.replace('let mut k = index[lane];','let mut k=self.recover(index[lane],lo);')
s=s.replace('                    let k=c.locate(x);\n                    let r=c.runs[k];','                    let k=c.locate(x);\n                    let r=c.read_reply(c.reply(x));')
s=s.replace('fn packed_lf_and_boundaries() {','fn rank_check<const PAGE_ID:bool>() {').replace('#[test]\nfn rank_check','fn rank_check')
a=s.index('fn rank_check')
s=s[:a]+s[a:].replace('let mut c=Candidate::default();','let mut c=Dispatch::<2,PAGE_ID>::default();')
a=s.rindex('}')
s=s[:a]+'''    #[test]
    fn all_reply_types() { rank_check::<false>(); rank_check::<true>(); }
'''+s[a:]
(here/'candidate.rs').write_text(s)
