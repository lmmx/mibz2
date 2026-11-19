"""Generate two 8-byte F2 encodings from the frozen guarded POPCNT candidate."""
from hashlib import sha256
from pathlib import Path
import re
here=Path(__file__).resolve().parent
source=(here.parent/'instructions/candidate.rs').read_text()
assert sha256(source.encode()).hexdigest() == '9eedac9e1f7c54a4eaf966c5c1f34e8b919862995ee272085b4cac436d9bfd8b'
small=(here.parents[1]/'f2b/checks.rs').read_text().split('\n#[test]\n#[ignore]')[0]

def variant(ends):
    s=source
    a=s.index('#[derive(Clone, Copy, Default)]\nstruct Run {')
    b=s.index('#[derive(Clone, Copy, Default)]\nstruct Piece',a)
    s=s[:a]+(here/'record.inc.rs').read_text()+s[b:]
    a=s.index('        self.runs.clear();',s.index('fn prepare('));b=s.index('    #[inline]\n    fn locate',a)
    s=s[:a]+'''        let mut counts=[0u32;256];
        for &record in caps {
            counts[(record>>8) as usize]+=u32::from(record&255);
        }
        self.prepare_body(caps,n,&counts)
    }
'''+s[b:]
    a=s.index('                self.runs.push(Run {')
    b=s.index('                self.starts[',a)
    add='''                let delta=next[byte as usize].wrapping_sub(row);
                self.runs.push(Run::new(row,byte,delta));
'''
    if ends:
        s=s[:a]+'''                if let Some(previous)=prev {
                    self.runs.push(Run::new(row,previous,delta));
                }
                delta=next[byte as usize].wrapping_sub(row);
'''+s[b:]
        s=s.replace('        let mut prev = None;', '        let mut prev = None;\n        let mut delta=0;')
    else:s=s[:a]+add+s[b:]
    a=s.index('        self.runs.push(Run {\n            start: n,')
    b=s.index('        let mut preceding',a)
    close='        self.runs.push(Run::new(n,prev.unwrap(),delta));\n' if ends else '        self.runs.push(Run::new(n,0,0));\n'
    s=s[:a]+close+s[b:]
    s=s.replace('x = r.lf + (x - r.start);','x = r.map(x);')
    s=s.replace('lo[lane] = r.lf + (lo[lane] - r.start);','lo[lane] = r.map(lo[lane]);')
    s=s.replace('let mapped = r.lf + (a - r.start);','let mapped = r.map(a);')
    s=re.sub(r'\br\.byte\b', 'r.byte()', s)
    if ends:
        s=s.replace('self.runs[k + 1].start < lo[lane] + width[lane]', 'r.position_byte < ((lo[lane] + width[lane]) << 8)')
        a=s.index('                while self.runs[k].start < hi {')
        b=s.index('                    let mapped =',a)
        s=s[:a]+'''                let mut a=lo;
                while a<hi {
                    let r=self.runs[k];
                    let end=hi.min(r.position());
                    let width=end-a;
'''+s[b:]
        s=s.replace('                    k += 1;', '                    a=end;\n                    k += 1;')
    else:
        s=s.replace('self.runs[k + 1].start','self.runs[k+1].position()')
        s=s.replace('self.runs[k].start','self.runs[k].position()')
        s=s.replace('r.start','r.position()')
    s=s[:s.index('#[cfg(test)]\n#[path = "../../f2c/checks.rs"]')]
    bound='(k==0 || x>=c.runs[k-1].position()) && x<r.position()' if ends else 'x>=r.position() && x<c.runs[k+1].position()'
    s+='\n#[cfg(test)]\nmod checks {\n'+small+'\n'+(here/'check.inc.rs').read_text().replace('BOUNDARY_CONDITION',bound)+'\n}\n'
    return s

(here/'candidate.rs').write_text(variant(True))
(here/'starts.rs').write_text(variant(False))
