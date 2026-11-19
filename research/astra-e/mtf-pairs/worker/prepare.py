"""Generate the isolated front and the verified predecessor's paid kernel in place."""
import subprocess
import sys
import tempfile
from pathlib import Path
here=Path(__file__).resolve().parent
ref="d0a42980e896211906234c7679b38760ae44a46e"
scalar=here.parents[1]/"front-fusion/cap-batch"
# These prepare programs are decoder inputs. Abort on drift.
for relative in ("front-fusion/cap-batch/prepare.py","narrow-short/prepare.py","edge-budget/prepare.py"):
    rel="research/astra-e/"+relative
    assert (here.parents[3]/rel).read_bytes()==subprocess.check_output(["git","show",ref+":"+rel])
with tempfile.TemporaryDirectory() as tmp:
    dest=Path(tmp)
    subprocess.run([sys.executable,here.parent/"generate.py",tmp],check=True)
    for arm in ("stock","wide"):
        (here/(arm+".front.rs")).write_bytes((dest/(arm+".rs")).read_bytes())
    subprocess.run([sys.executable,scalar/"prepare.py",tmp],check=True)
    (here/"kernel.rs").write_bytes((dest/"scalar.rs").read_bytes())
    (here/"gate.rs").write_bytes((dest/"gate.rs").read_bytes())
