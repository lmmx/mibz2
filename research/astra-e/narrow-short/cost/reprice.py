"""Build four matched call paths: metered and unmetered counter and baseline variants."""
from pathlib import Path
import subprocess,sys
from queues import build
here=Path(__file__).resolve().parent
tmp=Path(sys.argv[1]).resolve()
deps=Path(sys.argv[2]).resolve()
new=tmp/"queue"; new_meter=tmp/"queue_meter"
build(tmp/"frozen",new)
build(tmp/"instrument",new_meter)
rel="research/astra-e/narrow-short/ahead.rs"
s=(tmp/"main.rs").read_text()
start=s.index('    let mut f=std::io::BufWriter')
s=s[:start]+(here/"reprice.inc.rs").read_text()
s=s.replace('fn main(){','fn main(){\n    let mut queue=queues::kernel::Kernel::default();\n    let mut all_meter=queue_meter::kernel::Kernel::default();')
s=s.replace('        write!(geometry,','        let proposed=queue.run::<true,true,false>(r.records(),n,p,16,&mut out);\n        assert!(out==refout && c.steps==proposed.steps && c.table_bytes==proposed.table_bytes && c.bands==proposed.bands && c.words==proposed.words);\n        write!(geometry,',1)
s=s.replace('mod original;',f'mod original;\n#[path="{new/rel}"] mod queues;\n#[path="{new_meter/rel}"] mod queue_meter;')
s=s.replace('let b=&blocks[0];\n        COUNT','let b=&blocks[0];\n        all_meter.run::<true,true,false>(b.r.records(),b.n,b.p,16,&mut out);\n        COUNT').replace('COUNT.set(true);cut(0);metered.run','COUNT.set(true);cut(0);all_meter.run')
p=tmp/"reprice.rs";p.write_text(s)
subprocess.run(["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1","-C","debuginfo=1",p,"--extern",f"mibz2={deps}/libmibz2.rlib","-L",f"dependency={deps}","-o",tmp/"reprice"],check=True)
