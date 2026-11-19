"""Restore the independently verified scalar and edge sources, replacing just the marked emitter."""
from pathlib import Path
import subprocess
import sys

here=Path(__file__).resolve().parent
root=here.parents[3]
out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else here/'target'
subprocess.run([sys.executable,here.parent/'build_worker.py',out],check=True)
scalar_commit='d0a42980e896211906234c7679b38760ae44a46e'
rel=Path('research/astra-e/front-fusion/cap-batch')
p=subprocess.Popen(['git','-C',root,'archive',scalar_commit,rel],stdout=subprocess.PIPE)
subprocess.run(['tar','-x','-C',out/'frozen'],stdin=p.stdout,check=True)
assert p.wait()==0
subprocess.run([sys.executable,out/'frozen'/rel/'prepare.py',out/'scalar-source'],cwd=root,check=True)
edge=(out/'generated/edge0.rs').read_text()
edge2=(out/'generated/two-seed.rs').read_text()
marker='\nmod twoseed {\n'
assert edge2.count(marker)==1
edge2base,wr=edge2.split(marker)
assert wr.rstrip().endswith('}')
wr=wr.rstrip()[:-1]
assert wr.count('split::<false,true>(&t,out);')==1
wr=wr.replace('split::<false,true>(&t,out);','many::emit::<4,false>(&t,out);')
module=(here/'writer.rs').read_text().replace('use crate::{original,Table,writer};','use super::Table;')
module=module.replace('original::Kernel::stock(t,out);','super::super::emit(t.edges,t.extra,t.spelling,t.offsets,t.n,t.mark,t.bits,out);')
module=module.replace('writer::place(', 'super::place(').replace('writer::finish(', 'super::finish(')
module=module.replace('pub fn emit<', 'pub(super) fn emit<')
section=marker+wr+'\nmod many {\n'+module+'\n}\n}\n'
edge4=edge2base+section
(out/'generated/edge-four.rs').write_text(edge4)
orig=(out/'scalar-source/scalar.rs').read_text()
for name in ['gate.rs','adapter.inc.rs']:
    assert (out/'scalar-source'/name).read_bytes()==(out/'generated'/name).read_bytes()
needle='        } else {\n            emit(\n                &self.edges,'
assert edge.count(needle)==orig.count(needle)==1
guarded=orig.replace(needle,needle.replace('emit(','twoseed::emit_split('))
(out/'generated/scalar.rs').write_text(orig)
(out/'generated/scalar-four.rs').write_text(guarded+section)
