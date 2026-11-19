from pathlib import Path
from hashlib import sha256
import subprocess,sys

HERE=Path(__file__).resolve().parent
QUOT=HERE.parents[1]/'native'
SPARSE=HERE.parents[2]/'native'

def generate(out):
    subprocess.run([sys.executable,'-B',SPARSE/'generate.py',out],check=True)
    p=out/'control.rs'; s=p.read_text()
    assert sha256(s.encode()).hexdigest()==(QUOT/'control.sha').read_text().strip()
    s=s.replace('    links: Vec<u32>,','    links: Vec<u32>,\n    fold_scratch: FoldScratch,')
    s=s.replace('+self.second.capacity()+0','+self.second.capacity()+self.fold_scratch.capacity()')
    p.write_text(s+'\n'+(HERE/'fold.rs').read_text())
    p=out/'main.rs';s=p.read_text().split('#[allow(dead_code,unused_imports)] mod lifted12;')[0]
    s=s.replace('#[allow(dead_code,unused_imports)] mod lifted;\n','')
    p.write_text(s+(HERE/'main.rs').read_text())

if __name__=='__main__':generate(Path(sys.argv[1]))
