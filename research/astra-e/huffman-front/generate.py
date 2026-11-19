"""Freeze stock header/LUT/Reg code, redirect the per-selector output to a bounded local tile."""
from pathlib import Path
import subprocess
import sys
d=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
pin="d0a42980e896211906234c7679b38760ae44a46e"
def git(path):
    return subprocess.check_output(["git","show",f"{pin}:src/{path}.rs"]).decode()
(out/"bits.rs").write_text(git("bits").split("#[cfg(test)]")[0])
s=git("block").split("#[cfg(test)]")[0]
a=s.index("/// A block header's magic");b=s.index("/// A block's header and symbols")
c=s.index("/// A group's Huffman code",b)
header=s[:b]+"use mibz2::block::{Symbols,Item};\n"+s[c:]
(out/"stock.rs").write_text(header)
a=header.index("pub fn read_item(");b=header.index("    let magic =",a)
proto='''#[target_feature(enable="avx2,bmi1,bmi2,lzcnt,popcnt")]
pub unsafe fn read_fused<const TILE:usize>(br:&mut Bits,level:u32,out:&mut Output,counts:&mut [u64;256])->Option<(u32,u32,usize)> {
    macro_rules! fail {($k:expr)=>{return None};}
'''
s=header[:a]+proto+header[b:]
s=s.replace("return Ok(Item::End(br.bits(32)));","return None;")
a=s.index("    syms.clear();");b=s.index("    let mut n = 0;",a)
s=s[:a]+'''    assert!(TILE==50 || TILE==800);
    out.reset(selectors.len()*50,max_len);
    let mut front=Front::new(&seq_to_unseq,max_len);
    // 64 zeroed guard words allow the vectorized masks' last load at every tile end.
    let mut chunk=[0u16;864];
'''+s[b:]
s=s.replace("out[n].write((e >> SYM1 & 0x1ff) as u16);","chunk[n]=(e >> SYM1 & 0x1ff) as u16;")
s=s.replace("out[n + 1].write((e >> SYM2) as u16);","chunk[n+1]=(e >> SYM2) as u16;")
s=s.replace("out[n].write(s);","chunk[n]=s;")
at=s.index("        if r.overran(data) {\n            break Err",s.index("let res = 'groups"))
s=s[:at]+'''        if n==TILE {
            if !front.tile(&chunk,n,out){return None;}
            n=0;
        }
'''+s[at:]
at=s.index("    if let Err(k) = res");s=s[:at]+'''    if res.is_err() || br.overran() || n==0 {return None;}
    // EOB consumes a place in admission's sampled symbols, and emits no MTF event.
    if !front.tile(&chunk,n-1,out) {return None;}
    out.mask(0,1);
    if !out.admitted(max_len) || !front.close(out) {return None;}
    let len=front.counts.iter().map(|x|(x&0xffff_ffff)as usize).sum::<usize>();
    if len>max_len || orig_ptr as usize>=len{return None;}
    *counts=front.counts;
    Some((crc,orig_ptr,len))
}
'''
r=git("runs");a=r.index("#[cfg(target_arch",r.index("/// `src/mtf.rs`'s first level"));b=r.index("/// The AVX2",a) if "/// The AVX2" in r[a:] else 0
b=r.index("    v\n}",r.index("unsafe fn second(",a))+len("    v\n}")
s+=r[a:b].replace("static STEPS:","static MTF_STEPS:")+"\n"+(d/"front.rs").read_text()
(out/"fused.rs").write_text(s)
