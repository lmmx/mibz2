"""Regenerate the immutable combined kernel and insert costs at original phase boundaries."""
from pathlib import Path
import subprocess,sys,shutil

here = Path(__file__).resolve().parent
root = here.parents[3]
tmp = Path(sys.argv[1]).resolve()
tmp.mkdir(parents=True,exist_ok=True)
frozen = tmp/"frozen"
rev = "80d8409359214b4e5abfd4430213921e82e9cd30"
journal = "docs/journal/2026-11-18-open-return/"
research = "research/astra-e/"
def get(rel):
    p=frozen/rel
    p.parent.mkdir(parents=True,exist_ok=True)
    p.write_bytes(subprocess.check_output(["git","-C",root,"show",rev+":"+rel]))
for f in ["open_return.rs","endpoint.rs","segments.rs"]: get(journal+f)
for f in ["narrow-short/prepare.py","narrow-short/ahead.rs","narrow-endpoint/prepare.py","narrow-endpoint/candidate.rs","short-words/endpoint.rs","lookahead/writer.rs"]: get(research+f)
subprocess.run([sys.executable,frozen/research/"narrow-short/prepare.py"],check=True)
instrument=tmp/"instrument"
shutil.copytree(frozen,instrument,dirs_exist_ok=True)
key=instrument/research/"narrow-short/constructor_ahead.rs"
def once(src, needle, replacement):
    assert src.count(needle)==1,needle
    return src.replace(needle,replacement)

s=key.read_text()
for needle,step in [("        let first = starts(all);",1),("        let mut cursors = if INDUCE {",2),("        stats.bands = self.bands.len();",6),("        (stats, mark, bits)",7)]:
    s=once(s,needle,f'        crate::cut({step});\n'+needle)
s+='''
impl Kernel {
    pub fn capacities(&self)->[usize;8] {
        [crate::cap(&self.queues)+self.queues.iter().map(crate::cap).sum::<usize>(),
         crate::cap(&self.strips)+crate::cap(&self.image_order),
         self.endpoint.link_cap()+self.narrow.link_cap(),
         self.endpoint.rule_cap()+self.narrow.rule_cap(),
         crate::cap(&self.bands),
         crate::cap(&self.spelling)+crate::cap(&self.offsets),
         crate::cap(&self.edges)+crate::cap(&self.extra),
         crate::cap(&self.tiles)+crate::cap(&self.directory)]
    }
}
'''
key.write_text(s)
for sub in ["narrow-short/narrow.rs","short-words/endpoint.rs"]:
    p=instrument/research/sub
    s=p.read_text()
    for needle,step in [("        stats.steps = e.eliminate(n);",3),("        stats.rules = e.rules.len();",4),("        let mut cursors = [0; 256];",5)]:
        s=once(s,needle,f'        crate::cut({step});\n'+needle)
    s+='''
impl Endpoint {
    pub(super) fn link_cap(&self)->usize {
        crate::cap(&self.nodes)+self.order.iter().chain(self.terminals.iter()).map(crate::cap).sum::<usize>()+crate::cap(&self.bottom)
    }
    pub(super) fn rule_cap(&self)->usize { crate::cap(&self.rules)+crate::cap(&self.stack) }
}
'''
    p.write_text(s)

main=tmp/"main.rs"
mods="".join(f'#[path="{dir/research}/narrow-short/ahead.rs"] mod {name};\n' for dir,name in [(frozen,'original'),(instrument,'meter')])
main.write_text('#![allow(dead_code)]\n'+mods+(here/"main.rs").read_text())
deps=Path(sys.argv[2]).resolve()
flags=["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1","-C","debuginfo=1","--extern",f"mibz2={deps}/libmibz2.rlib","-L",f"dependency={deps}"]
subprocess.run([*flags,main,"-o",tmp/"phase"],check=True)
