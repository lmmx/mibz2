"""Two alternative end-record builders, each emitting identical directories and payloads."""
from hashlib import sha256
from pathlib import Path
here=Path(__file__).resolve().parent
s=(here.parent/'candidate.rs').read_text()
assert sha256(s.encode()).hexdigest()=='e93c1fa1482e7b654279e40e2dc1897da3676e9fe87336cb5497c273f9c5eace'
s=s.replace('path = "../../f2c/', 'path = "../../../f2c/')
a=s.index('        let mut prev = None;')
b=s.index('        let mut preceding',a)
for method,filename in [('coalesce','candidate.rs'),('eager','eager.rs')]:
    (here/filename).write_text(s[:a]+(here/(method+'.inc.rs')).read_text()+s[b:])
