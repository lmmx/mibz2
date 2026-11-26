from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
base=d.parent/"laps"
subprocess.run([sys.executable,str(base/"prepare.py")],check=True)
s=(base/"open.rs").read_text().replace('"walk.rs"','"../laps/walk.rs"')
a=s.index("        for &start in &self.danger")
b=s.index("        self.worked = true;",a)
s=s[:a]+'''        if !self.owners_of_windows(n as u32) {
            self.reason=2;
            emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,&mut self.raw);
            return mibz2::rle::unrle(&self.raw,out,k);
        }
'''+s[b:]+ '\ninclude!("owners.rs");\n'
(d/"open.rs").write_text(s)
(d/"adapter.inc.rs").write_bytes((base/"adapter.inc.rs").read_bytes())
