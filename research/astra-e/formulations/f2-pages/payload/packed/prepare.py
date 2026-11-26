"""Re-price packed pages using counted construction, POPCNT and eight-byte end runs."""
from pathlib import Path
from hashlib import sha256
here=Path(__file__).resolve().parent
s=(here.parent/'construction/eager.rs').read_text()
assert sha256(s.encode()).hexdigest()=='82704548e38df71d53eb7eab1b12674d62b0987d7fc789c5d1bc00cdff6213dc'
s=s.replace('struct Dispatch<const MODE: u8>', 'struct Dispatch<const MODE: u8, const WIDTH:u32=44>')
s=s.replace('impl<const MODE:u8>', 'impl<const MODE:u8,const WIDTH:u32>')
s=s.replace('Dispatch<MODE>', 'Dispatch<MODE,WIDTH>')
s=s.replace('    page_first: Vec<u32>,\n','')
a=s.index('        let page = (x >> 6) as usize;');b=s.index('\n    }',a)
s=s[:a]+'''        let entry=self.starts[(x/WIDTH) as usize];
        let prefix=(1u64 << (x%WIDTH))-1;
        ((entry>>44) as u32+(entry & prefix).count_ones()) as usize'''+s[b:]
s=s.replace('n.div_ceil(64)', 'n.div_ceil(WIDTH)').replace('        self.page_first.resize(pages, 0);\n','')
s=s.replace('self.starts[(row>>6) as usize]|=1u64 << (row&63);','self.starts[(row/WIDTH) as usize]|=1u64 << (row%WIDTH);')
a=s.index('        for (mask, base) in self.starts.iter_mut().zip(&mut self.page_first)')
b=s.index('        Some((quote, tokens))',a)
s=s[:a]+'''        for entry in &mut self.starts {
            let all=*entry;
            let base=preceding+(all as u32 & 1)-1;
            preceding+=all.count_ones();
            *entry=((base as u64)<<44) | (all>>1);
        }
'''+s[b:]
s=s.replace('fn packed_lf_and_boundaries() {','fn verify_rank<const WIDTH:u32>() {').replace('#[test]\nfn verify_rank', 'fn verify_rank')
a=s.index('fn verify_rank')
s=s[:a]+s[a:].replace('let mut c=Candidate::default();','let mut c=Dispatch::<2,WIDTH>::default();')
a=s.rindex('}')
s=s[:a]+'''    #[test]
    fn packed_lf_and_boundaries() {
        verify_rank::<32>(); verify_rank::<40>(); verify_rank::<44>();
    }
'''+s[a:]
(here/'candidate.rs').write_text(s)
