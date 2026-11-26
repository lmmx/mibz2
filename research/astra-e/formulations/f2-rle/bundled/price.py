from pathlib import Path
import sys,subprocess
d=Path(__file__).resolve().parent
work=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d/"prepare.py")],check=True)
base=d.parent/"laps"
subprocess.run([sys.executable,str(base/"price.py"),str(work)],check=True)
s=(work/"main.rs").read_text().replace('["laps", "opend","compiled"]','["bundle", "opend","laps"]')
s=s.replace('#[path="plain.rs"]mod plain;','#[path="parent.rs"]mod plain;')
(work/"parent.rs").write_bytes((work/"lap.rs").read_bytes())
sparse=(d/"open.rs").read_text().replace("../../../../../docs",str(d.parents[4]/"docs")).replace('"../compiled/nav.rs"',f'"{d.parent/"compiled/nav.rs"}"').replace('"../laps/walk.rs"',f'"{base/"walk.rs"}"').replace('"owners.rs"',f'"{d/"owners.rs"}"')
(work/"lap.rs").write_text(sparse)
(work/"main.rs").write_text(s)
