"""Reuse the parent's frozen paired-sum and interval calculation with the four published-point arm names."""
from pathlib import Path
here=Path(__file__).resolve().parent
s=(here.parent/'report.py').read_text()
s=s.replace("arms=('singleton','split-ordinal','unseeded','dense')","arms=('histogram','prefix','delayed','dense')")
start=s.index("for a,b in [('split-ordinal'")
end=s.index("    entry={}",start)
s=s[:start]+"""for a,b in [('prefix','histogram'),('delayed','histogram'),('delayed','prefix'),
            ('histogram','dense'),('prefix','dense'),('delayed','dense')]:
"""+s[end:]
exec(compile(s,str(here.parent/'report.py'),'exec'))
