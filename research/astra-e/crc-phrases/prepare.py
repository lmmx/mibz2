"""Pin the target-measured combined constructor and exact CRC implementation."""
from pathlib import Path
import subprocess,sys
from hashlib import sha256
here=Path(__file__).resolve().parent
root=here.parents[2]
out=Path(sys.argv[1]).resolve()
frozen=out/'frozen'
rev='80d8409359214b4e5abfd4430213921e82e9cd30'
journal='docs/journal/2026-11-18-open-return/'
research='research/astra-e/'
files=[journal+f for f in ['open_return.rs','endpoint.rs','segments.rs']]
files += [research+f for f in ['narrow-short/prepare.py','narrow-short/ahead.rs','narrow-endpoint/prepare.py','narrow-endpoint/candidate.rs','short-words/endpoint.rs','lookahead/writer.rs']]
files += ['src/crc.rs']
for rel in files:
    p=frozen/rel
    p.parent.mkdir(parents=True,exist_ok=True)
    p.write_bytes(subprocess.check_output(['git','-C',root,'show',f'{rev}:{rel}']))
subprocess.run([sys.executable,frozen/research/'narrow-short/prepare.py'],check=True)
key=frozen/research/'narrow-short/constructor_ahead.rs'
src=key.read_text()
assert sha256(src.encode()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
src+='''
impl Kernel {
    pub fn capture(&mut self, records:&[u16], n:usize, p:u32)->(Vec<u32>,&[u32],&[u8]) {
        let (_,mark,bits)=self.construct::<true,true>(records,n as u32,p,16);
        let mask=(1u32<<bits)-1;
        let mut row=mark;
        let mut codes=Vec::new();
        loop {
            let edge=self.edges[row as usize];
            codes.push(if self.extra.is_empty() {edge>>bits} else {self.extra[row as usize]});
            row=edge&mask;
            if row==mark {break}
        }
        (codes,&self.offsets,&self.spelling)
    }
}
'''
key.write_text(src)
crc=frozen/'src/crc.rs'
crc.write_text(crc.read_text()+'''
/// Expose the production incremental dispatch to a stand-alone research binary.
pub fn advance(s:u32,data:&[u8],k:Kernels)->u32 {
    #[cfg(target_arch="x86_64")]
    if k.clmul && data.len()>=128 {
        // SAFETY: operational assumption established by detected() in main.
        return unsafe {clmul_update(s,data)};
    }
    update(s,data)
}
''')
(out/'main.rs').write_text(f'#![allow(dead_code)]\n#[path="{key}"] mod kernel;\n'+f'mod crc_copy {{pub struct Kernels {{pub clmul:bool}} #[path="{crc}"] pub mod crc;}}\n'+(here/'main.rs').read_text())
