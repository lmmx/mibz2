"""Build the short certificate-only panel and same-code whole-corpus gate attribution."""
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
output = Path(sys.argv[1]).resolve()
subprocess.run([sys.executable, str(here.parent / "prepare.py"), str(output)], check=True)
main = (output / "main.rs").read_text()
main = main[:main.index("fn main(){")].replace('include!("gate.rs");', 'include!("gate.rs");\ninclude!("certificate.rs");')
gate = (output / "gate.rs").read_text()
assert gate.count("fn fits(") == 1
(output / "gate.rs").write_text(gate.replace("fn fits(", "#[inline(never)]\nfn control("))
(output / "certificate.rs").write_text((here / "geometry.rs").read_text() + (here / "simd.rs").read_text())
(output / "main.rs").write_text(main + (here / "panel.rs").read_text())
