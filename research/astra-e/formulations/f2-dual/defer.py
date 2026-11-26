"""Isolate the cost of turning exact affine bands into one packed word per token row."""
from pathlib import Path
import sys

here=Path(__file__).resolve().parent
text=(here/"candidate.rs").read_text().replace("    cache: Vec<Cert>,","    cache: Vec<Cert>,\n    descriptors:Vec<(u32,u32,u32)>,")
a=text.index("            let source = root - section;")
b=text.index("            self.offsets.push",a)
text=text[:a]+'''            self.descriptors.push((root-section,dest,width));
'''+text[b:]
text=text.replace("        self.links.resize(m as usize, 0);","        self.descriptors.clear();")
text+='''impl Candidate {
    pub fn measure(&mut self, r:&Runs,n:usize,p:u32,out:&mut Vec<u8>)->[u64;4] {
        let t0=crate::thread_cpu_ns();
        let (s,m)=self.prepare(r.records(),n as u32,PATTERN).unwrap();
        let (row,skip)=self.transfer_mark(p,s,m).unwrap();
        let t1=crate::thread_cpu_ns();
        assert!(self.bands(s,m));
        std::hint::black_box(&self.descriptors);
        let t2=crate::thread_cpu_ns();
        let bits=u32::BITS-(m-1).leading_zeros();
        self.links.resize(m as usize,0);
        for (code,&(source,dest,width)) in self.descriptors.iter().enumerate() {
            for (k,e) in self.links[dest as usize..(dest+width) as usize].iter_mut().enumerate() {
                *e=((code as u32)<<bits)|(source+k as u32);
            }
        }
        let t3=crate::thread_cpu_ns();
        self.expand(row,skip,m,n,out);
        std::hint::black_box(&out);
        let t4=crate::thread_cpu_ns();
        [t1-t0,t2-t1,t3-t2,t4-t3]
    }
}
'''
out=Path(sys.argv[1])
out.mkdir(parents=True,exist_ok=True)
(out/"dual.rs").write_text(text)
src=(here/"price.rs").read_text()
a=src.index("    let (mut cache,")
b=src.index("    let (mut reference",a)
src=src[:a]+'''    let(mut cache,mut lazy)=(dual::Candidate::default(),LazyCells::default());
'''+src[b:]
src=src.replace('#[path = "trie.rs"]\nmod trie;','').replace("0..4 {","0..2 {").replace("% 4;","% 2;")
a=src.index("                    0 =>")
b=src.index("                    _ =>",a)
src=src[:a]+'''                    0=>cache.measure(&r,n,p,&mut out),
'''+src[b:]
src=src.replace('["cache", "plain", "trie", "lazy"]','["deferred","lazy"]').replace("merge_ns","scatter_ns")
(out/"price.rs").write_text(src)
