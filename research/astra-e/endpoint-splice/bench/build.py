"""Build distinct metered and native binaries against the frozen scalar library."""
import argparse
from pathlib import Path
import hashlib
import json
import shutil
import subprocess as sp
import sys
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
PIN="d0a42980e896211906234c7679b38760ae44a46e"

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("out",type=Path)
    ap.add_argument("kind",choices=["census","price"])
    z=ap.parse_args()
    d=z.out.resolve()/z.kind
    d.mkdir(parents=True,exist_ok=True)
    sp.run([sys.executable,"-B",HERE.parent/"prepare.py",d],check=True)
    source=d/"donor"
    source.mkdir(exist_ok=True)
    p=sp.Popen(["git","archive",PIN,"src"],cwd=ROOT,stdout=sp.PIPE)
    sp.run(["tar","-x","-C",source],stdin=p.stdout,check=True)
    assert p.wait()==0
    flags=["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1","--remap-path-prefix",f"{d}=/research"]
    lib=d/"libmibz2.rlib"
    sp.run([*flags,"--crate-type","rlib","--crate-name","mibz2","--cfg",'feature="moonshot"',source/"src/lib.rs","-o",lib],check=True)
    if z.kind=="census":
        for name in ["control","batched","count"]:
            kernel=d/name/"kernel.rs"
            kernel.write_text(kernel.read_text()+"\n"+(HERE/"inspect.inc.rs").read_text())
    source=d/"main.rs"
    source.write_text((HERE/"main.rs").read_text())
    shutil.copyfile(HERE/"finite.inc.rs",d/"finite.inc.rs")
    cfg=["--cfg","census"] if z.kind=="census" else []
    exe=d/"panel"
    sp.run([*flags,*cfg,source,"--extern",f"mibz2={lib}","-o",exe],check=True)
    sources=[p for p in d.rglob("*.rs") if "donor" not in p.relative_to(d).parts]
    record={"source_pin":PIN,"src_tree":sp.check_output(["git","rev-parse",f"{PIN}:src"],cwd=ROOT,text=True).strip(),
       "exe_sha256":sha(exe),"rlib_sha256":sha(lib),"rustc":sp.check_output(["rustc","-vV"],text=True),
       "flags":[*flags[1:],*cfg],"sources":{str(p.relative_to(d)):sha(p) for p in sorted(sources)},
       "corpus_sha256":sha(ROOT/"corpora/wikidata.bz2")}
    (d/"build.json").write_text(json.dumps(record,indent=2)+"\n")

if __name__=="__main__":
    main()
