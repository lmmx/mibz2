from pathlib import Path
import subprocess
import sys
HERE=Path(__file__).resolve().parent
out=Path(sys.argv[1]);deps=sys.argv[2]
subprocess.run([sys.executable,'-B',str(HERE.parent/'native'/'build.py'),str(out),deps],check=True)
p=out/'second.rs';s=p.read_text()
start=s.index('        let f=&mut self.forest;')
end=s.index('        self.count.fill(0);',start)
s=s[:start]+(HERE/'select.rs').read_text()+s[end:]
start=s.index('            while pos<end {',start)
end=s.index('            at=end;',start)
s=s[:start]+(HERE/'emit.rs').read_text()+s[end:]
p.write_text(s)
subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports','-A','unused_mut','-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',str(out/'main.rs'),'--extern','mibz2='+deps+'/libmibz2.rlib','-L','dependency='+deps,'-o',str(out/'sparse')],check=True)
