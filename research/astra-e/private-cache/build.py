"""Build a standalone observation of the frozen i9 winner without changing decoder sources."""
import hashlib
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
REV = "badb2aec3624bd65606c429b2e6f79e9d9d0050f"
out = Path(sys.argv[1]).resolve()
library = Path(sys.argv[2]).resolve() if len(sys.argv) > 2 else out / "libmibz2.rlib"
out.mkdir(parents=True, exist_ok=True)
frozen = out / "source"
archive = subprocess.Popen(
    ["git", "-C", ROOT, "archive", REV, "src", "docs/journal/2026-11-18-open-return",
     "research/astra-e/edge-budget", "research/astra-e/narrow-short",
     "research/astra-e/narrow-endpoint", "research/astra-e/short-words",
     "research/astra-e/lookahead"], stdout=subprocess.PIPE)
frozen.mkdir(exist_ok=True)
subprocess.run(["tar", "-x", "-C", frozen], stdin=archive.stdout, check=True)
assert archive.wait() == 0
if len(sys.argv) == 2:
    subprocess.run(["rustc", "--edition=2021", "-C", "opt-level=3", "-C", "codegen-units=1",
                    "--crate-name", "mibz2", "--crate-type", "rlib", "--cfg", 'feature="moonshot"',
                    frozen/"src/lib.rs", "-o", library], check=True)
d = frozen / "research/astra-e"
subprocess.run([sys.executable, d / "edge-budget/prepare.py"], check=True)
source = d / "narrow-short/constructor_ahead.rs"
assert hashlib.sha256(source.read_bytes()).hexdigest() == "c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f"
with source.open("a") as f:
    f.write((HERE / "observe.inc.rs").read_text())
for name in ["gate.rs"]:
    shutil.copy(d / "edge-budget" / name, out / name)
shutil.copy(HERE / "geometry.rs", out / "geometry.rs")
for name, program in [("main.rs", "replay"), ("panel.rs", "panel"), ("libcheck.rs", "libcheck")]:
    shutil.copy(HERE / name, out / name)
    subprocess.run(["rustc", "--edition=2021", "-C", "opt-level=3", "-C", "codegen-units=1",
                    out/name, "--extern", f"mibz2={library}",
                    "-L", f"dependency={library.parent}", "-o", out/program], check=True)
