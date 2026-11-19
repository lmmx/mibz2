"""Replace just the counted page constructor in the frozen counts-reuse candidate."""
from hashlib import sha256
from pathlib import Path
here=Path(__file__).resolve().parent
source=(here.parents[1]/'f2-front-counts/candidate.rs').read_text()
assert sha256(source.encode()).hexdigest()=='0f788874ed1149327bb9c2693cb15d2b2ef3d405cfb892fd777380f6af0fd986'
start=source.index('    fn prepare_counted(')
end=source.index('    fn transfer_mark(', start)
source=source[:start]+(here/'stream.inc.rs').read_text()+source[end:]
source=source.replace('path = "../f2c/', 'path = "../../f2c/')
source+='\ninclude!("check.inc.rs");\n'
(here/'candidate.rs').write_text(source)
