"""Build two diagnostic fronts with the same supplied gold labels plus the true paid history front."""
from pathlib import Path
import subprocess
import sys
d=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,d/"generate.py",out],check=True)
s=(out/"history.rs").read_text()
a=s.index("struct History");b=s.index("/// What it reads:",a)
t=s[:a]+s[b:]
t=t.replace("f: &mut Runs) -> Option<usize>","f: &mut Runs, gold: &[u8]) -> Option<usize>")
pos="            let c=history.select((s-1).min(255)) as u64;"
assert t.count(pos)==1
for arm in ["replay","topology"]:
    update="            let c=u64::from(*gold.get_unchecked(wi*64+tz));"
    state=""
    use=""
    if arm=="topology":
        state="    let mut live=0u64;"
        update="""            let rank=(s-1).min(255);
            let wanted=1u64.checked_shl(rank as u32).unwrap_or(0);
            let selected=_pdep_u64(wanted,live);
            live=((live^selected)<<1)|1;
"""+update
        use="    std::hint::black_box(live);\n"
    code=t.replace("    let mut history=History::new(s2u);",state).replace(pos,update).replace("    Some(n as usize)",use+"    Some(n as usize)")
    (out/(arm+".rs")).write_text(code)
s=s.replace("f: &mut Runs) -> Option<usize>","f: &mut Runs, _gold: &[u8]) -> Option<usize>")
(out/"paid.rs").write_text(s)
