"""Compile a structural instrument against two immutable source blobs in the worktree."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

here=Path(__file__).resolve().parent
pins=json.loads((here/'sources.json').read_text())
out=Path(sys.argv[1]).resolve()
out.mkdir(parents=True,exist_ok=True)
original=out/'donor'
target=original/'refine'
target.mkdir(parents=True,exist_ok=True)
for path,pin in pins.items():
    sha=pin['sha256']
    raw=(here.parents[2]/path).read_bytes()
    if hashlib.sha256(raw).hexdigest()!=sha:
        raw=subprocess.check_output(['git','cat-file','blob',pin['git_blob']])
    if hashlib.sha256(raw).hexdigest()!=sha:
        raise ValueError(path)
    if path=='refine/refine.cpp':
        raw=raw[:raw.index(b'int main(int argc,char** argv)')] + (here/'panel.cpp').read_bytes()
    (original/path).write_bytes(raw)
subprocess.run(['g++','-O2','-std=c++20',str(target/'refine.cpp'),'-o',str(out/'prune')],check=True)
