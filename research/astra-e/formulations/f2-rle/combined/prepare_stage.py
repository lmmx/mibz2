"""Freeze the four compiler input closures and add exact retained-capacity observers."""
from hashlib import sha256
from pathlib import Path
import re, subprocess,sys
d=Path(__file__).resolve().parent
scratch=Path(sys.argv[1]).resolve();scratch.mkdir(parents=True,exist_ok=True)
subprocess.run([sys.executable,str(d/"prepare.py")],check=True)
repo=d.parents[4];journal=repo/"docs/journal/2026-11-18-open-return";ahead=d.parents[2]/"narrow-short"
paths=[journal/"open_return.rs",ahead/"constructor_ahead.rs",d.parent/"bundled/open.rs",d/"open.rs"]
sources={}
def expand(p):
    p=p.resolve(); s=p.read_text()
    sources[p]=sha256(s.encode()).hexdigest()
    return re.sub(r'include!\("([^"]+)"\);',lambda m:expand(p.parent/m[1]),s)
heap='''impl Endpoint {
    pub(super) fn heap_bytes(&self)->usize{
        crate::heap(&self.nodes)+crate::heap(&self.rules)+crate::heap(&self.bottom)+crate::heap(&self.stack)+
        self.order.iter().chain(&self.terminals).map(crate::heap).sum::<usize>()
    }
}'''
for i,(name,p) in enumerate(zip(("old","ahead","bundle","combo"),paths)):
    s=expand(p)
    def module(m):
        path=(p.parent/m[1]).resolve();sub=expand(path)
        if path.name=="endpoint.rs" or path.name=="narrow.rs":
            sub+="\n"+heap+"\n"
            dest=scratch/(name+"-"+path.name);dest.write_text(sub)
        else:
            dest=path
        return '#[path="'+str(dest)+'"]'
    s=re.sub(r'#\[path\s*=\s*"([^"]+)"\]',module,s)
    fields="queues strips image_order tiles directory bands spelling offsets edges extra".split()
    if i>=2:fields+="source_width image_width events danger owners labels raw".split()
    extras="+self.narrow.heap_bytes()" if i in (1,3) else ""
    formula="std::mem::size_of::<Self>()+self.endpoint.heap_bytes()"+extras+"+self.queues.iter().map(crate::heap).sum::<usize>()+"+"+".join("crate::heap(&self."+f+")" for f in fields)
    s+='\nimpl Kernel{pub(super) fn bytes(&self)->usize{'+formula+'}}\n'
    (scratch/(name+".rs")).write_text(s)
(scratch/"main.rs").write_bytes((d/"stage.rs").read_bytes())
sources[d/"stage.rs"]=sha256((d/"stage.rs").read_bytes()).hexdigest()
for path,digest in sorted(sources.items()):
    print(digest,path.relative_to(repo))
