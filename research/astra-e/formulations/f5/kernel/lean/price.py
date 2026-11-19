"""Compare full output+CRC for the one-byte hint, full actions and stock opend."""
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
base = here.parent
repo = here.parents[5]
work = Path(sys.argv[1]).resolve()
work.mkdir(parents=True, exist_ok=True)
subprocess.run([sys.executable, str(here / "prepare.py")], check=True)
subprocess.run([sys.executable, str(base / "phase.py"), str(work)], check=True)
lean = (here / "open.rs").read_text().replace(
    "../../../../../../docs/journal", str(repo / "docs/journal")
).replace('include!("book.rs");', (here / "book.rs").read_text())
(work / "lean.rs").write_text(lean)
source = (base / "phase.rs").read_text()
source = source.replace("mod direct;", "mod direct;\n#[allow(dead_code)] mod lean;")
source = source.replace("let mut direct = direct::Kernel::default();",
    "let mut direct=direct::Kernel::default(); let mut lean=lean::Kernel::default();let mut hints=lean::Book::default();")
source = source.replace("verified += n;", '''out.clear();
        lean.expanded(r.records(),n,p,&mut hints,&mut out).unwrap();
        assert_eq!(out,old_out);
        verified+=n;''')
source=source.replace("0..=20", "0..=21").replace("for offset in 0..2","for offset in 0..3").replace("(round + offset) % 2","(round + offset) % 3")
i=source.index("                if arm == 0 {")
j=source.index("                let before_crc",i)
source=source[:i]+'''                if arm==0 {
                    direct.expanded(r.records(),n,p,&mut book,&mut out).unwrap();
                } else if arm==1 {
                    old.run::<true,true,false>(r.records(),n,p,16,&mut raw);
                    mibz2::rle::unrle(&raw,&mut out,k).unwrap();
                } else {
                    lean.expanded(r.records(),n,p,&mut hints,&mut out).unwrap();
                }
'''+source[j:]
source=source.replace('["direct", "opend"][arm]', '["direct", "opend", "lean"][arm]')
(work / "main.rs").write_text(source)
