from pathlib import Path
import os,re,subprocess,sys
d=Path(__file__).resolve().parent
ahead=d.parents[3]/"narrow-short"
subprocess.run([sys.executable,str(ahead/"prepare.py")],check=True)
s=(ahead/"constructor_ahead.rs").read_text()
s=re.sub(r'#\[path\s*=\s*"([^"]+)"\]',lambda m:'#[path="'+os.path.relpath((ahead/m[1]).resolve(),d)+'"]',s)
s=s.replace("pub struct Kernel {","pub struct Kernel {\n    reader:Nav,corrected_caps:Vec<u16>,original:Vec<u8>,outcome:(u8,usize,usize,usize,usize),")
(d/"open.rs").write_text(s+'\ninclude!("nav.rs");\ninclude!("order.rs");\ninclude!("finish.rs");\n')
(d/"adapter.inc.rs").write_bytes((d.parents[1]/"count-only/adapter.inc.rs").read_bytes()) if (d.parents[1]/"count-only/adapter.inc.rs").exists() else None
if not (d/"adapter.inc.rs").exists():
    subprocess.run([sys.executable,str(d.parents[1]/"count-only/prepare.py")],check=True)
    (d/"adapter.inc.rs").write_bytes((d.parents[1]/"count-only/adapter.inc.rs").read_bytes())
