"""Instrument only the initial queued geometry scan, rotating the real large-block sample."""
from pathlib import Path
import subprocess,sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here/"prepare.py"),str(out)],check=True)
for name in ["control.rs","scalar.rs","grouped.rs"]:
    p=out/name
    s=p.read_text().replace("        let mut last = None;", "        let phase_start=crate::cpu();\n        let mut last = None;")
    s=s.replace("        assert!(m > 0 && all.iter()", "        crate::PHASE.with(|v|v.set(crate::cpu()-phase_start));\n        assert!(m > 0 && all.iter()")
    assert s.count("phase_start")==2
    p.write_text(s)
p=out/"main.rs"
s=p.read_text().replace("fn main(){", "thread_local!{ static PHASE:std::cell::Cell<u64> = const {std::cell::Cell::new(0)}; }\nfn main(){")
s=s.replace("class,cpu_ns", "class,cpu_ns,phase_ns")
s=s.replace("for round in 0..=if selected{rounds}else{0} {", """for round in 0..=rounds {
            if round>0 && !edge && (!large || (ordinal+round)%24>=3) {continue;}""")
s=s.replace("let start=cpu();let output", "PHASE.with(|v|v.set(0));let start=cpu();let output")
s=s.replace('{class},{elapsed}");','{class},{elapsed},{}",PHASE.with(std::cell::Cell::get));')
p.write_text(s)
