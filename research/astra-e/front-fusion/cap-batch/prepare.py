"""Freeze the edge baseline and vary only the constructor's maximal-run iterator."""
from pathlib import Path
from hashlib import sha256
import re
import subprocess
import sys

here = Path(__file__).resolve().parent
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True,exist_ok=True)
ahead = here.parents[1] / "narrow-short"
subprocess.run([sys.executable, str(ahead/"prepare.py")], check=True)
baseline = (ahead/"constructor_ahead.rs").read_text()
assert sha256(baseline.encode()).hexdigest() == "c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f"
kernel = re.sub(r'#\[path\s*=\s*"([^"]+)"\]',lambda m:'#[path="'+str((ahead/m[1]).resolve())+'"]',baseline)
a = kernel.index("fn each_run(")
b = kernel.index("fn starts(",a)
old = kernel[a:b]
addition = "        if width == 127 {\n"
inner = old.index("        while i < records.len() &&")
stop = old.index("        let end", inner)
scalar = old[:inner] + addition + old[inner:stop] + "        }\n" + old[stop:]
(out/"control.rs").write_text(kernel)
(out/"scalar.rs").write_text(kernel[:a]+scalar+kernel[b:])
(out/"grouped.rs").write_text(kernel[:a]+scalar.replace("fn each_run(", "fn scalar(")+(here/"grouped.rs").read_text()+kernel[b:])
gate=subprocess.check_output(["git","show","badb2aec3624bd65606c429b2e6f79e9d9d0050f:research/astra-e/edge-budget/gate.rs"])
(out/"gate.rs").write_bytes(gate)
(out/"main.rs").write_bytes((here/"main.rs").read_bytes())
arm_path=here.parents[1]/"edge-budget"
subprocess.run([sys.executable,str(arm_path/"prepare.py")],check=True)
(out/"adapter.inc.rs").write_bytes((arm_path/"adapter.inc.rs").read_bytes())
for file in ["control.rs","scalar.rs","grouped.rs"]:
    with (out/file).open("a") as s:
        s.write("""
pub(super) fn observe(records:&[u16],p:u32)->Vec<(u8,u32,u32,bool)>{
    let mut answer=Vec::new();
    each_run(records,p,16,|c,x,w,k|answer.push((c,x,w,k)));
    answer
}
""")
