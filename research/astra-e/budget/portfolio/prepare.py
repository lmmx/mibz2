"""Freeze both experiment sources, regenerate the original budget instrumentation."""
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
budget = here.parent
root = budget.parents[2]
dest = Path(sys.argv[1]).resolve()
dest.mkdir(parents=True, exist_ok=True)
ahead = "80d8409359214b4e5abfd4430213921e82e9cd30"
pop = "3651470cb0ef5458e75cf97d3de363a1c73f2f76"
original = "00efe9b602edd272951fc746528c38d415365996"

def fetch(rev, name, into):
    dst = into / name
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(subprocess.check_output(["git","-C",str(root),"show",f"{rev}:{name}"]))
    return dst

local = dest / "frozen"
front = local / "research/astra-e"
j = "docs/journal/2026-11-18-open-return"
for f in ["open_return.rs","endpoint.rs","segments.rs"]:
    fetch(ahead, j+"/"+f, local)
for f in ["narrow-short/prepare.py","narrow-short/ahead.rs","narrow-endpoint/prepare.py","narrow-endpoint/candidate.rs","short-words/endpoint.rs","lookahead/writer.rs"]:
    fetch(ahead, "research/astra-e/"+f, local)
subprocess.run([sys.executable, front/"narrow-short/prepare.py"],check=True)
pp = "research/astra-e/formulations/f2-pages/instructions/candidate.rs"
popfile = fetch(pop, pp, local)
second = fetch(pop, "research/astra-e/formulations/f2c/second.rs", local)

# The capacity functions add read-only methods; timing uses unmodified inverse methods.
def append(p, body):
    with p.open("a") as sink: sink.write("\n"+body+"\n")
def vecs(fields):
    return " + ".join("crate::cap(&self."+f+")" for f in fields.split())
def array(field):
    return "self."+field+".iter().map(crate::cap).sum::<usize>()"
def impl(name, body):
    return "impl "+name+" {pub(crate) fn retained(&self)->usize { "+body+" } }"
for p in [front/"short-words/endpoint.rs",front/"narrow-short/narrow.rs"]:
    append(p,impl("Endpoint",vecs("nodes rules bottom stack")+" + "+array("order")+" + "+array("terminals")))
append(local/j/"segments.rs",impl("Segments",vecs("segments")+" + "+array("codes")))
append(front/"narrow-short/constructor_ahead.rs",impl("Kernel",vecs("queues strips image_order tiles directory bands spelling offsets edges extra")+" + "+array("queues")+" + self.endpoint.retained() + self.narrow.retained() + self.segments.retained()"))
append(front/"narrow-short/ahead.rs",impl("Candidate","self.open.retained()"))
append(second,impl("Second",vecs("runs count base epoch child page masks nodes edges selectors words dictionary merged links")))
append(popfile,"impl<const MODE:u8> Dispatch<MODE> {pub(crate) fn retained(&self)->usize { "+vecs("runs page_first starts nodes selectors edges words dict merged links")+" + "+array("lane_paths")+" + self.second.retained() } }")

# Reuse the original instrument verbatim through its library build, verified at the frozen revision.
subprocess.run(["git","-C",str(root),"diff","--quiet",original,"--","src","research/astra-e/budget/clock.rs","research/astra-e/budget/prepare.py","research/astra-e/budget/main.rs"],check=True)
prior = (budget/"prepare.py").read_text().split("modules = {")[0]
exec(compile(prior,str(budget/"prepare.py"),"exec"),{"__file__":str(budget/"prepare.py")})

main = dest / "portfolio.rs"
preamble = (budget/"main.rs").read_text().split("fn scan_only(")[0]
main.write_text('#![allow(dead_code)]\n' + f'#[path="{front}/narrow-short/ahead.rs"] mod writer;\n#[path="{popfile}"] mod counted;\n' + preamble + (here/"main.rs").read_text())
subprocess.run(["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1",main,"--extern",f"mibz2={dest}/libmibz2.rlib","-o",dest/"portfolio"],check=True)
