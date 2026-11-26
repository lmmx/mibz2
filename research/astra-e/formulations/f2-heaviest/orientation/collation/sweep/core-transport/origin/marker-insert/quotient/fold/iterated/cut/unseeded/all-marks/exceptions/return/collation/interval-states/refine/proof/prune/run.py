"""Run the frozen structural panel and check its actual k8 membership-wire inputs."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
exe, wires = map(Path, sys.argv[1:])
header = ('block,charge,pass,states,atoms,original_cuts,extreme_cuts,active_cuts,skipped_states,'
          'original_fragments,extreme_fragments,active_fragments,optimal_score,original_added,pruned_added')
print(header, flush=True)
for arm in json.loads((here/'inputs.json').read_text()):
    wire = wires/arm['name']
    assert hashlib.sha256(wire.read_bytes()).hexdigest() == arm['sha256']
    subprocess.run([str(exe.resolve()),str(wire),str(arm['block'])],check=True)
