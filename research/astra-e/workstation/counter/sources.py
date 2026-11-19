"""Record the actual checkout-local Rust dependencies named by rustc, including generated modules."""
import hashlib
import json
from pathlib import Path
import shlex
import sys

tree, depfile, output = map(Path, sys.argv[1:])
line = depfile.read_text().splitlines()[0]
files = [Path(name).resolve() for name in shlex.split(line.split(": ", 1)[1])]
digests = {str(p.relative_to(tree)):hashlib.sha256(p.read_bytes()).hexdigest()
           for p in files if p.is_relative_to(tree) and p.is_file()}
assert any(name.startswith("research/astra-e/") and name.endswith(".rs") for name in digests)
assert "bench/moonshot/src/main.rs" in digests
output.write_text(json.dumps(digests, sort_keys=True, indent=2)+"\n")
