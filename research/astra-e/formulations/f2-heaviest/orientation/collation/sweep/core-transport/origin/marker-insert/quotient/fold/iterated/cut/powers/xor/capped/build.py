"""Build the joint same-caps paid stage harness against the frozen scalar worktree."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess as sp

here=Path(__file__).resolve().parent
p=argparse.ArgumentParser()
p.add_argument("donor",type=Path,help="prepared scalar d0a4298 including v1 scorer refs")
p.add_argument("target",type=Path,help="release build dir containing deps/libmibz2.rlib")
p.add_argument("out",type=Path)
a=p.parse_args()
donor,target,out=[v.resolve() for v in (a.donor,a.target,a.out)]
out.mkdir(parents=True,exist_ok=True)
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def run(*cmd):
    return sp.check_output(cmd,text=True).strip()
files={"open_return":donor/"docs/journal/2026-11-18-open-return/open_return.rs",
       "arms":donor/"bench/moonshot/src/arms.rs",
       "candidate":donor/"research/astra-e/front-fusion/cap-batch/worker/candidate.rs"}
generated="".join(f'#[path={json.dumps(str(v))}] pub mod {k};\n' for k,v in files.items())
main=out/"main.rs"
main.write_text(generated+(here/"panel.rs").read_text())
so=out/"libgrammar.so"
sp.run(["g++","-O3","-std=c++20","-fPIC","-shared",here/"bridge.cpp","-o",so],check=True)
rlib=target/"deps/libmibz2.rlib"
flags=["--edition=2021","-C","opt-level=3","-C","codegen-units=1","-C","debuginfo=line-tables-only","-C","lto=fat"]
sp.run(["rustc",*flags,main,"-L",f"dependency={target/'deps'}","--extern",f"mibz2={rlib}","-L",f"native={out}","-l","dylib=grammar","-C",f"link-arg=-Wl,-rpath,{out}","-o",out/"panel"],check=True)
for n in ("kernel.rs","gate.rs","adapter.inc.rs"):
    files[n]=files["candidate"].parent/n
manifest={"rustc":run("rustc","-Vv"),"g++":run("g++","--version").splitlines()[0],
          "donor":run("git","-C",str(donor),"rev-parse","HEAD"),"src_tree":run("git","-C",str(donor),"rev-parse","HEAD:src"),
          "source_sha256":{k:sha(v) for k,v in files.items()},"rlib_sha256":sha(rlib),
          "dso_sha256":sha(so),"exe_sha256":sha(out/"panel"),"rust_flags":flags}
(out/"build.json").write_text(json.dumps(manifest,indent=2)+"\n")
