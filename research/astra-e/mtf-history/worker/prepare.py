"""Use the exact scalar inverse and matched external-frontend hookup of the prior price."""
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile
d=Path(__file__).resolve().parent
old=d.parents[1]/"mtf-pairs/worker"
assert hashlib.sha256((old/"common.rs").read_bytes()).hexdigest()=="dcd86eada370d126fc9970a54e9dc7650c59ee2d22d0905c524cc6b37cb87029"
subprocess.run([sys.executable,old/"prepare.py"],check=True)
with tempfile.TemporaryDirectory() as tmp:
    subprocess.run([sys.executable,d.parent/"generate.py",tmp],check=True)
    (d/"history.front.rs").write_bytes((Path(tmp)/"history.rs").read_bytes())
