"""Alternate correctness arm using the library's entire column-route worker with CRC."""
from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent;p=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d/"prepare.py"),str(p)],check=True)
s=(p/"main.rs").read_text().replace("column:Vec<u8>,checkpoints:Vec<[u32;256]>,", "plain:mibz2::Scratch<bwt::LazyCells>,")
a=s.index("            let mut counts=[0u32;256];");b=s.index("        }\n",s.index("return Ok((n,0,false));",a))
s=s[:a]+'''            self.raw.clear();self.decoded.clear();
            mibz2::decode_block(b,k,&mut self.plain,&mut self.decoded)?;
            return Ok((0,0,false));
'''+s[b:]
(p/"main.rs").write_text(s)
