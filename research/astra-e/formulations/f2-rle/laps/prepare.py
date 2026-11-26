from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
base=d.parent/"compiled"
subprocess.run([sys.executable,str(base/"prepare.py")],check=True)
s=(base/"open.rs").read_text().replace('include!("nav.rs");','include!("../compiled/nav.rs");')
s=s.replace("for &x in &self.owners {","for owner in &mut self.owners {\n            let x=*owner;")
s=s.replace("let pos = (x - skipped) as usize;","let pos = (x - skipped) as usize;\n            *owner=pos as u32;")
s=s.replace("self.labels.push(self.edges[pos] >> bits);","self.labels.push((self.edges[pos]>>bits) | (1<<31));").replace("self.labels.push(self.extra[pos]);","self.labels.push(self.extra[pos] | (1<<31));")
a=s.index("        let mut row = mark;")
b=s.index("    pub fn pricing(",a)
s=s[:a]+'''        for &owner in &self.owners {
            let next=self.edges[owner as usize]&mask;
            let tag=if self.extra.is_empty(){self.edges[next as usize]>>bits}else{self.extra[next as usize]};
            if tag>=first {self.labels[(tag-first)as usize]&=0x7fff_ffff;}
        }
        self.finish_laps(n,mark,bits,first,out)
    }
'''+s[b:] + '\ninclude!("walk.rs");\n'
(d/"open.rs").write_text(s)
(d/"adapter.inc.rs").write_bytes((base/"adapter.inc.rs").read_bytes())
