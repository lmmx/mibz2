from hashlib import sha256
from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
bundle=d.parent/"bundled"
ahead=d.parents[2]/"narrow-short"
for p in (bundle,ahead):
    subprocess.run([sys.executable,str(p/"prepare.py")],check=True)
normal=(d.parents[4]/"docs/journal/2026-11-18-open-return/open_return.rs").read_text()
best=(ahead/"constructor_ahead.rs").read_text()
assert 'self.induced_narrow(n, &thin, &mut stats)' in best
s=(bundle/"open.rs").read_text().replace('"owners.rs"','"../bundled/owners.rs"')
s=s.replace("    endpoint: endpoint::Endpoint,","    endpoint: endpoint::Endpoint,\n    narrow: narrow::Endpoint,")
lo=best.index("            // H strips plus at most")
hi=best.index("\n        } else {",lo)
s=s.replace("            self.induced(n, &thin, &mut stats)",best[lo:hi])
s=s.replace('#[path="../../../../../docs/journal/2026-11-18-open-return/endpoint.rs"] mod endpoint;',
            '#[path="../../../short-words/endpoint.rs"] mod endpoint;\n#[path="../../../narrow-short/narrow.rs"] mod narrow;')
i=s.index('fn emit(');j=s.index('#[inline]\nfn append_code',i)
s=s[:i]+best[best.index('fn emit('):best.index('#[inline]\nfn append_code')]+s[j:]
walk=(d.parent/"laps/walk.rs").read_text()
walk=walk.replace("        loop {\n            let e = self.edges[row as usize];","        let mut e=self.edges[row as usize];\n        loop {")
walk=walk.replace("            row = e & mask;","            row = e & mask;\n            e=self.edges[row as usize];")
assert "let mut e=self.edges[row as usize]" in walk and "e=self.edges[row as usize];\n            if tag" in walk
s=s.replace('include!("../laps/walk.rs");',walk)
(d/"open.rs").write_text(s)
(d/"adapter.inc.rs").write_bytes((bundle/"adapter.inc.rs").read_bytes())
