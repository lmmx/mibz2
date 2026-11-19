"""Embed the history selector in the complete original record frontend."""
from pathlib import Path
import hashlib
import subprocess
import sys
d=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
out.mkdir(exist_ok=True,parents=True)
s=subprocess.check_output(["git","show","d0a42980e896211906234c7679b38760ae44a46e:src/runs.rs"]).decode()
assert hashlib.sha256(s.encode()).hexdigest()=="bb8d4fa30854986e5d7d5304a40ee8809588b1bbb259edfb9fe3e11cd9cfdca9"
s=s[s.index("/// `src/mtf.rs`'s first level"):s.index("/// The block's LF table from its records")]
s=s.replace("unsafe fn front_avx2(", "pub(super) unsafe fn front_avx2(")
header="use super::Runs;\nuse mibz2::block::Symbols;\nconst CAP:u32=127;\n"
(out/"stock.rs").write_text(header+s)
a=s.index("    let mut f0 =");b=s.index("    // a record for each literal",a)
s=s[:a]+"    let mut history=History::new(s2u);\n"+s[b:]
a=s.index("            if s < 33 {");b=s.index("            if w != 0 {",a)
s=s[:a]+"            let c=history.select((s-1).min(255)) as u64;\n"+s[b:]
a=s.index("/// `src/mtf.rs`'s first level");b=s.index("/// What it reads:",a)
s=s[:a]+s[b:]
(out/"history.rs").write_text(header+(d/"history.rs").read_text()+s)
# The fixture generator checks each accepted fast path against independent scalar cap records.
base=subprocess.check_output(["git","show","dd0f8a1:research/astra-e/mtf-pairs/panel.rs"]).decode()
a=base.index("mod stock;");b=base.index("#[derive(Default)]",a)
base=base[:a]+"mod stock; mod history;\n"+base[b:]
a=base.index("#[cfg(not(compact))]");b=base.index("#[repr(C)]",a)
base=base[:a]+'''const ARMS:[(&str,Front);2]=[("stock",stock::front_avx2),("history",history::front_avx2)];
'''+base[b:]
base=base.replace("let limit=if cfg!(compact){32}else{16};","let limit=32;")
base=base[:base.index("    let round_count:usize=")]+"}\n"
(out/"check.rs").write_text(base)
p=subprocess.check_output(["git","show","efddae8:research/astra-e/mtf-pairs/placements.rs"]).decode()
p=p.replace('#[path="wide.rs"] mod composed;', 'mod history;').replace("composed::front_avx2","history::front_avx2").replace('"wide"','"history"')
(out/"placements.rs").write_text(p)
