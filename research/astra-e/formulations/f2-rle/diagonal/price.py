"""Pin the same verification and phase driver while substituting the sparse index."""
import sys,subprocess
from pathlib import Path
d=Path(__file__).resolve().parent
work=Path(sys.argv[1]);work.mkdir(parents=True,exist_ok=True)
subprocess.run([sys.executable,str(d/"prepare.py")],check=True)
main=(d.parent/"measure.rs").read_text().replace('"section.rs"',f'"{d/"section.rs"}"').replace('"open.rs"',f'"{d/"open.rs"}"').replace('"../../../../docs',f'"{d.parents[4]}/docs')
(work/"main.rs").write_text(main)
