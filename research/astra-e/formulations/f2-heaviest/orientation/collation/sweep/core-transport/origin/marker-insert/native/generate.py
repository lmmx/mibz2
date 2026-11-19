"""Generate a private sparse publisher from both already validated native backends."""
from hashlib import sha256
from pathlib import Path
import subprocess
import sys

HERE=Path(__file__).resolve().parent
TWO=HERE.parents[2]/'two-section/native'

def main(out):
    subprocess.run([sys.executable,'-B',TWO/'generate.py',out],check=True)
    original=(out/'lifted.rs').read_text()
    # Keep h4/h8 bit-for-bit identical, and use a separate 12-byte packet only for h12.
    wide=original.replace('prefix: [u8; 8]', 'prefix: [u8; 12]').replace('prefix: [0; 8]', 'prefix: [0; 12]')
    wide=wide.replace('h <= 8 && n <=', 'h <= 12 && n <=').replace('|| h>8 {return None;}', '|| h>12 {return None;}')
    assert original.count('prefix: [u8; 8]')==2 and 'h>8' not in wide
    (out/'lifted12.rs').write_text(wide)
    p=out/'control.rs'
    s=p.read_text()
    assert 'pub fn first(&mut self,caps:' in s and 'pub fn second_phases(&self)' in s
    p.write_text(s+'\n'+(HERE/'publish.rs').read_text())
    main=(out/'main.rs').read_text()
    # Input, original dense map and the original-run archive reader from the pinned harness.
    header=main
    # Some generations begin the extra oracle with another comment. Keep the common first Truth impl only.
    first=header.index('impl Truth{')
    end=header.index('\n}\n',first)+3
    header=header[:end]
    for name in ['follower','projected','shared']:
        header=header.replace('#[allow(dead_code,unused_imports)] mod '+name+';\n','')
    output=header+'\n'+(HERE/'oracle.rs').read_text()+'\n'+(HERE/'main.rs').read_text()
    (out/'main.rs').write_text(output)

if __name__=='__main__':main(Path(sys.argv[1]).resolve())
