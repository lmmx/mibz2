"""Project the frozen run-only variable selector, omitting its dense input oracle."""
from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
p=d.parent/"variable/states.cpp"
rel=subprocess.check_output(["git","ls-files","--full-name",p],text=True).strip()
s=subprocess.check_output(["git","show","9e232479:"+rel],text=True)
a=s.index("using U=");b=s.index("U read32",a)
c=s.index("// This constructor reads only");e=s.index("std::vector<Key> oracle",c)
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
(out/"selector.inc").write_text("namespace selection {\n"+s[a:b]+"struct Input{U n;std::vector<Run> runs;};\n"+s[c:e]+"}\n")
