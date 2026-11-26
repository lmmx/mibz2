from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
p=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d.parent/"prepare.py"),str(p)],check=True)
front=(p/"front.rs").read_text()
a=front.index("#[derive(Default)] pub(super) struct Bits");b=front.index("\n",a)
front=front[:a]+'use crate::front::Bits;\ninclude!("cap_count.rs");'+front[b:]
front=front.replace("-> Option<usize> {\n    use std::arch", "-> Option<(usize,usize)> {\n    use std::arch")
a=front.index("    let syms") if "    let syms" in front else front.index("    let m = b.syms.len();")
front=front[:a]+'''    if !(2..=257).contains(&b.eob){return None;}
    let mut alphabet=[0u64;4];
    for &c in &b.seq_to_unseq[..usize::from(b.eob-1)] {
        let bit=1u64<<(c%64);let word=&mut alphabet[usize::from(c/64)];
        if *word&bit!=0{return None;}*word|=bit;
    }
'''+front[a:]
front=front.replace("    let mut end = usize::MAX;", "    let mut max_literal=_mm256_setzero_si256();\n    let mut end = usize::MAX;")
front=front.replace("        let run = bits!(", "        max_literal=_mm256_max_epu16(max_literal,_mm256_max_epu16(x,y));\n        let run = bits!(")
front=front.replace("            o |= ((s != 0)", "            if s>b.eob{return None;}\n            o |= ((s != 0)")
a=front.index("    // ---- the literals ----")
front=front[:a]+'''    if _mm256_movemask_epi8(_mm256_cmpeq_epi16(_mm256_max_epu16(ve,max_literal),ve)) != -1 {return None;}
    let (caps,_)=capped_count(&f.lit,&f.one,end)?;
    if caps*20>=3*b.max_len{return None;}
'''+front[a:]
front=front.replace("if total>b.max_len as u64 ||", "if total+row_count as u64>b.max_len as u64 ||")
front=front.replace("                match next {","                if row_count>b.max_len{return None;}\n                match next {")
front=front.replace("(row_count<=b.max_len).then_some(row_count)", "(row_count<=b.max_len).then_some((row_count,caps))")
(p/"pre.rs").write_text(front);(p/"cap_count.rs").write_bytes((d.parent/"cap_count.rs").read_bytes())
kernel=(p/"kernel.rs").read_text();a=kernel.index("impl Kernel {\n    pub(super) fn fuse<")
fuse=kernel[a:].replace("fn fuse<const TILE:usize>","fn fuse_pre<const TILE:usize>")
fuse=fuse.replace("let n=unsafe{crate::front::front_avx2","let (n,caps)=unsafe{crate::pre::front_avx2")
a=fuse.index("            if c as u16==");b=fuse.index("            if TILE==",a)
fuse=fuse[:a]+fuse[b:]
fuse=fuse.replace("        if n==0||", "        p.caps=caps as u32;\n        if n==0||")
(p/"kernel.rs").write_text(kernel+fuse)
main=(p/"main.rs").read_text().replace("mod front;","mod front;\nmod pre;")
main=main.replace("fused=self.kernel.fuse::<TILE>(b,&mut self.bits,&mut self.raw);", "fused=if TILE==64{self.kernel.fuse_pre::<0>(b,&mut self.bits,&mut self.raw)}else{self.kernel.fuse::<TILE>(b,&mut self.bits,&mut self.raw)};")
(p/"main.rs").write_text(main)
