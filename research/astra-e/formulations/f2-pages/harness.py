"""Write an instrumented scratch module for each source without altering the measured decoder."""
from pathlib import Path
import re
import sys

here = Path(__file__).resolve().parent
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=True)
price = (here.parent / 'f2c/price_impl.rs').read_text()
for module, folder in [('baseline', here.parent/'f2-scheduler'), ('layout', here)]:
    s = (folder/'candidate.rs').read_text()
    def resolve(m):
        return m[0].replace(m[1], str((folder / m[1]).resolve()))
    s = re.sub(r'(?:#\[path = "|include!\(")([^"]+)"', resolve, s)
    s += '\n' + (price if module == 'baseline' else price.replace('impl Candidate {', 'impl<const WIDTH:u32> PageCandidate<WIDTH> {'))
    if module == 'layout':
        s += (here/'geometry.inc.rs').read_text()
    (out/(module+'.rs')).write_text(s)
(out/'price.rs').write_text((here/'price.rs').read_text())
