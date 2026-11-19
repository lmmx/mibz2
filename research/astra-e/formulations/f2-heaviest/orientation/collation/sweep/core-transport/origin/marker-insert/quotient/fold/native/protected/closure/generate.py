from pathlib import Path
import subprocess,sys
from hashlib import sha256
HERE=Path(__file__).resolve().parent

def sub(s,a,b):assert s.count(a)==1,a;return s.replace(a,b)

def make(out):
    subprocess.run([sys.executable,'-B',HERE.parent/'generate.py',out],check=True)
    p=out/'control.rs';s=p.read_text();assert sha256(s.encode()).hexdigest()==(HERE/'control.sha').read_text().strip()
    s=sub(s,'struct FoldScratch {','struct FoldScratch {\n    skip_choose:bool,\n    forced:Option<Vec<u32>>,')
    s=sub(s,'        self.choose();','''        if let Some(bases)=&self.forced {
            let mut pending=bases.iter().copied().peekable();
            for &k in &self.order{
                let c=&mut self.classes[k];
                if pending.peek()==Some(&c.base){assert_ne!(c.succ,NONE);c.selected=true;pending.next();}
            }
            assert!(pending.next().is_none());
        }else if !self.skip_choose{self.choose();}''')
    p.write_text(s+ (HERE/'census.rs').read_text())
    p=out/'second.rs';p.write_text(p.read_text()+(HERE/'second.rs').read_text())
    p=out/'main.rs';s=p.read_text();i=s.index('fn main(){let args=');p.write_text(s[:i]+(HERE/'fresh.rs').read_text()+(HERE/'driver.rs').read_text())

if __name__=='__main__':make(Path(sys.argv[1]))
