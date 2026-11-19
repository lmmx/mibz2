from pathlib import Path
from tempfile import TemporaryDirectory
import subprocess,sys

here=Path(__file__).resolve().parent
with TemporaryDirectory() as temp:
    out=Path(temp)
    subprocess.run([sys.executable,str(here.parent/"prepare.py"),temp],check=True)
    for source,dest in [("scalar.rs","kernel.rs"),("adapter.inc.rs","adapter.inc.rs"),("gate.rs","gate.rs")]:
        (here/dest).write_bytes((out/source).read_bytes())
