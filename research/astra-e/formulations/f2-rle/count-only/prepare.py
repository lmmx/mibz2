from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
base=d.parent/"combined"
subprocess.run([sys.executable,str(base/"prepare.py")],check=True)
s=(base/"open.rs").read_text()
a=s.index('include!("../compiled/nav.rs");')
s=s[:a]+'''include!("nav.inc.rs");\ninclude!("patches.rs");\n'''
s=s.replace("    fbase:Bases","    edits: Vec<Edit>,added:usize, fbase:Bases")
nav=(d.parent/"compiled/nav.rs").read_text()
nav=nav.replace("fn hazards(&mut self, n: u32)", "fn event_ranges(&mut self, n: u32)")
a=nav.index("        for i in 0..self.events.len()")
b=nav.index("        true",a)
(d/"nav.inc.rs").write_text(nav[:a]+nav[b:])
(d/"open.rs").write_text(s)
(d/"adapter.inc.rs").write_bytes((base/"adapter.inc.rs").read_bytes())
