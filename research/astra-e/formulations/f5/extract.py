"""Build the pinned queued-open constructor and record its actual marked word stream."""
import argparse
from pathlib import Path
import hashlib, json, os, subprocess, tempfile
p=argparse.ArgumentParser()
p.add_argument("corpus",type=Path); p.add_argument("out",type=Path)
p.add_argument("--mibz2-rlib",type=Path,help="reuse a moonshot-feature build of this crate")
a=p.parse_args()
here=Path(__file__).resolve().parent
root=here.parents[3]
journal=root/"docs/journal/2026-11-18-open-return"
source=(journal/"open_return.rs").read_bytes()
assert hashlib.sha256(source).hexdigest()=="b02a1d2ed7490a0dbef8f5ecf464153d75d2cd71b42993807a3a713bc18bbfc1"
assert hashlib.sha256(a.corpus.read_bytes()).hexdigest()=="82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
s=source.decode().split("    pub fn run<const INDUCE:")[0]
s=s.replace("    segments: segments::Segments,\n","")
s=s[s.index("#[derive(Default, Clone, Copy, Debug)]"):]
s+='}\n#[path='+json.dumps(str(journal/"endpoint.rs"))+']\nmod endpoint;\n'
s+=(here/"extract.rs").read_text()
with tempfile.TemporaryDirectory() as tmp:
    d=Path(tmp); (d/"main.rs").write_text(s)
    if a.mibz2_rlib:
        lib=a.mibz2_rlib.resolve()
        subprocess.run(["rustc","-A","dead_code","--edition=2021","-O",str(d/"main.rs"),"-L",f"dependency={lib.parent}","--extern",f"mibz2={lib}","-o",str(d/"extract")],check=True)
        cmd=[str(d/"extract")]
    else:
        (d/"Cargo.toml").write_text('[package]\nname="f5-extract"\nversion="0.0.0"\nedition="2021"\n[[bin]]\nname="f5-extract"\npath="main.rs"\n[dependencies]\nmibz2={path='+json.dumps(str(root))+',features=["moonshot"]}\n')
        cmd=["cargo","run","-q","--release","--manifest-path",str(d/"Cargo.toml"),"--"]
    a.out.mkdir(exist_ok=True,parents=True)
    subprocess.run([*cmd,str(a.corpus.resolve()),str(a.out.resolve())],check=True)
