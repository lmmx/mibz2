"""Original scalar constructor, with sparse accounting only on actual dense admissions."""
import subprocess
import sys
from pathlib import Path

d=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
lib=Path(sys.argv[2]).resolve()
subprocess.run([sys.executable,d.parent/'build_worker.py',out],check=True)
g=out/'generated'
s=(g/'scalar.rs').read_text()
a=s.index('    pub fn run<const INDUCE: bool, const QUEUE: bool, const SEGMENTS: bool>(')
b=s.index('    pub fn price<',a)
fn=s[a:b]
cut='        let (stats, mark, bits) = self.construct::<INDUCE, QUEUE>(records, n as u32, p, limit);'
assert fn.count(cut)==1
fn=fn.replace(cut,'''        let selected=records.len()*20>=n*3;
        let start=if selected {crate::audit_clock(3)} else {0};
'''+cut+'''
        let built=if selected {crate::audit_clock(3)} else {0};
''')
tail='        stats\n'
assert fn.count(tail)==1
fn=fn.replace(tail,'''        if selected {
            let end=crate::audit_clock(3);
            crate::dense_charge(n,stats.retained,built-start,end-built);
        }
'''+tail)
(g/'meter.rs').write_text(s[:a]+fn+s[b:])
template='''#[allow(dead_code)]
#[path="generated/FILE.rs"] mod kernel;
include!("generated/gate.rs");
include!("generated/adapter.inc.rs");
'''
for slot,file in [('base','scalar'),('audit','meter')]:
    (out/(slot+'.rs')).write_text(template.replace('FILE',file))
(out/'main.rs').write_bytes((d/'audit.rs').read_bytes())
subprocess.run(['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1','-C','debuginfo=1',out/'main.rs','--extern',f'mibz2={lib}','-o',out/'audit'],check=True)
