from pathlib import Path
from hashlib import sha256
import subprocess,sys
HERE=Path(__file__).resolve().parent
def generate(out):
    prev=HERE.parents[1]
    subprocess.run([sys.executable,'-B',prev/'generate.py',out],check=True)
    c=out/'control.rs';s=c.read_text()
    assert sha256(s.encode()).hexdigest()==(HERE/'control.sha').read_text().strip()
    c.write_text(s+(HERE/'pair.rs').read_text())
    p=out/'second.rs';s=p.read_text()
    tail=s[s.index('        let pages = m.div_ceil(64)'):s.index('    #[inline]\n    fn find')]
    tail=tail.replace('(0..alphabet)','(0..kinds)')
    fmt=s[s.index('        let token_count = self.count[d as usize];'):s.index('impl Second{\n    pub(super) fn capacity')]
    # The last two braces close invert and its impl.
    assert fmt.endswith('    }\n}\n')
    spec='''
impl Second {
    fn end_pair_index(&mut self,m:u32,kinds:usize)->u32{
'''+tail+'''
    pub(super) fn finish_pair(&mut self,d:u32,p:u32,skip:u32,m:u32,n:u32,first_words:&[Piece],bytes:&[u8],out:&mut Vec<u8>)->bool{
        self.phases=[0;4];
        let t0=crate::now();
'''+fmt
    p.write_text(s+(HERE/'index.rs').read_text()+spec)
    p=out/'main.rs';s=p.read_text().split('const ARMS:[&str;4]')[0]
    p.write_text(s+(HERE/'native.rs').read_text())

if __name__=='__main__':generate(Path(sys.argv[1]))
