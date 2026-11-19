from pathlib import Path
from hashlib import sha256
import tempfile,subprocess,sys
here=Path(__file__).resolve().parent
with tempfile.TemporaryDirectory() as tmp:
    out=Path(tmp)
    subprocess.run([sys.executable,here.parent/'crc-phrases/prepare.py',out],check=True)
    pin=out/'frozen/research/astra-e/narrow-short'
    # CRC research capture is an appended separate impl; copy only the frozen implementation.
    s=(pin/'constructor_ahead.rs').read_text().split('\nimpl Kernel {\n    pub fn capture(')[0]
    assert sha256(s.encode()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
    s=s.replace('    extra: Vec<u32>,','    extra: Vec<u32>,\n    pack: Vec<[u8;3]>,\n    headers: Vec<u16>,\n    high: Vec<u8>,\n    packed:bool,')
    s=s.replace('fn construct<const INDUCE: bool, const QUEUE: bool>(', 'fn construct<const INDUCE: bool, const QUEUE: bool, const PACK_PAGE:usize>(')
    s=s.replace('self.construct::<INDUCE, QUEUE>(', 'self.construct::<INDUCE, QUEUE,0>(')
    anchor='        self.edges.resize(m as usize, 0);'
    assert s.count(anchor)==1
    s=s.replace(anchor,'''        self.packed=false;
        if PACK_PAGE>0 && QUEUE && !wide && self.build_packed::<PACK_PAGE>(m as usize,bits,&cursors) {
            self.packed=true;
            stats.table_bytes=self.pack.len()*3+self.headers.len()*2+self.high.len();
            return (stats,mark,bits)
        }
'''+anchor)
    s+='\n'+(here/'packed.inc').read_text()
    (here/'constructor.rs').write_text(s)
    (here/'narrow.rs').write_bytes((pin/'narrow.rs').read_bytes())
    adapter=(pin/'adapter.inc.rs').read_text().replace('struct Candidate {','struct Packed<const P:usize> {').replace('impl InverseBwt for Candidate {','impl<const P:usize> InverseBwt for Packed<P> {').replace('self.open.run::<true, true, false>(r.records(), n, p, 16, out);','self.open.packed_run::<P>(r.records(),n,p,out);')
    (here/'adapter.inc.rs').write_text(adapter)
for p in [8,16,32]:
    (here/f'p{p}.rs').write_text('#[allow(dead_code)]\n#[path="constructor.rs"]\nmod kernel;\ninclude!("adapter.inc.rs");\npub type Candidate = Packed<'+str(p)+'>;\n')
