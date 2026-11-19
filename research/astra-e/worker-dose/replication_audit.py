"""Match the actual Rust dependency closure for the primary and automatically triggered CI heads."""
import hashlib
import json
from pathlib import Path
import shlex
import subprocess
import sys

# Run against the paid mixed build's cargo depfile and its source checkout.
tree=Path(sys.argv[1]).resolve()
depfile=Path(sys.argv[2])
a,b="c783b7820a2d89af8d2ddfb77e63f9c017e5247b","ecd26dd331c74bdfba1384f8fad6088d1b6dfd13"
def git(*args):
    return subprocess.check_output(["git",*args])
src=[]
generated={}
for item in shlex.split(depfile.read_text().split(":",1)[1]):
    p=Path(item).resolve()
    if not p.is_relative_to(tree) or not p.is_file():continue
    rel=str(p.relative_to(tree))
    tracked=subprocess.run(["git","cat-file","-e",f"{a}:{rel}"],capture_output=True).returncode==0
    if tracked:
        if rel not in src: src.append(rel)
    else:
        generated[rel]=hashlib.sha256(p.read_bytes()).hexdigest()
assert {"research/astra-e/worker-dose/family.rs","research/astra-e/worker-dose/candidate.rs","research/astra-e/edge-budget/candidate.rs"}<=set(src)
gates=[".github/workflows/astra-e.yml",".just","Justfile","Cargo.lock","Cargo.toml","bench/moonshot","bench/wikidata.json",
       "research/astra-e/ci","research/astra-e/prepare.sh","research/astra-e/edge-budget","research/astra-e/narrow-short",
       "research/astra-e/narrow-endpoint","research/astra-e/short-words","docs/journal/2026-11-18-open-return","src"]
assert subprocess.run(["git","diff","--quiet",a,b,"--",*src,*gates]).returncode==0
print(json.dumps(dict(primary=a,replication=b,tracked=sorted(src),generated=generated,other_paths=gates),indent=2))
