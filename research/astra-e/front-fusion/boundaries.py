from pathlib import Path
import sys,subprocess
d=Path(__file__).resolve().parent;p=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d/"prepare.py"),str(p)],check=True)
s=(p/"main.rs").read_text().replace("fn main(){",'include!("boundaries.rs");\nfn main(){boundaries();return;')
(p/"main.rs").write_text(s);(p/"boundaries.rs").write_bytes((d/"boundaries.rs").read_bytes())
