"""Freeze controls and insert only the early-stop publication into compact/preload."""
from hashlib import sha256
from pathlib import Path
import re,sys

HERE=Path(__file__).resolve().parent
FORM=HERE.parents[2]
DST=Path(sys.argv[1]);DST.mkdir(parents=True,exist_ok=True)
inputs={"compact":FORM/"f2-pages/payload/slots/candidate.rs","known":FORM/"f2d/frontier/candidate.rs",
        "refill":FORM/"f2-scheduler/candidate.rs"}
expected = {
    "compact":"84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06",
    "known":"73397b7fabee01d3245794126bd210362460c32dd3747aad75d2be9f5961aa60",
    "refill":"1a659b1c65663db8901146d58623261c238c87dcd1a5ce6cb234c372cd59d4bf",
}
for name,path in inputs.items():
    assert sha256(path.read_bytes()).hexdigest()==expected[name],name
    s=path.read_text()
    s=re.sub(r'#\[path = "(\.\./)+f2c/second.rs"\]','#[path = "second.rs"]',s)
    if name=="compact":
        sig='impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE>'
        prep='self.prepare_counted(r.records(),n,r.byte_counts())'
    else:
        sig='impl Candidate';prep='self.prepare(r.records(),n)'
    extra=(HERE/"control.inc.rs").read_text().replace("IMPL_CONTROL",sig).replace("PREPARE",prep)
    (DST/(name+".rs")).write_text(s+'\n'+extra)
    if name=="compact":
        assert "threshold: u32" not in s
        s=s.replace('    fallback: Segmented,','    threshold: u32,\n    fallback: Segmented,',1)
        site='            while empty != 0 && next < self.nodes.len() {'
        assert s.count(site)==1
        s=s.replace(site,site+'\n'+(HERE/"cut.inc.rs").read_text())
        (DST/"cut.rs").write_text(s+'\n'+extra+'\n'+(HERE/"direction.rs").read_text())
for name in ("main.rs",):
    (DST/name).write_bytes((HERE/name).read_bytes())
(DST/"second.rs").write_bytes((FORM/"f2c/second.rs").read_bytes())
print(*(f"{name} {sha256(path.read_bytes()).hexdigest()}" for name,path in inputs.items()),sep="\n")
