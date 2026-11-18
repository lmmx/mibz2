"""Reproduce the experimental OpenGated constructor with immediate short words."""
from pathlib import Path
from hashlib import sha256

here = Path(__file__).resolve().parent
journal = here.parents[2] / 'docs/journal/2026-11-18-open-return'
old = (journal/'open_return.rs').read_text()
assert sha256(old.encode()).hexdigest() == 'b02a1d2ed7490a0dbef8f5ecf464153d75d2cd71b42993807a3a713bc18bbfc1'
old = (old[:old.index('#[path = "switches.rs"]')] +
       '#[path = "endpoint.rs"]\nmod endpoint;\n' +
       '#[path = "../../../docs/journal/2026-11-18-open-return/segments.rs"]\nmod segments;\n')
(here/'constructor.rs').write_text(old)
ep = (journal/'endpoint.rs').read_text()
start = ep.index('    fn len(&self')
end = ep.index('    fn gaps(', start)
ep = ep[:start] + (here/'join.inc.rs').read_text() + ep[end:]
a = ep.index('            if id <= 256 {')
b = ep.index('                if rule.first', a)
ep = ep[:a] + '''            let tag = id >> 30;
            if tag != 3 {
                if tag == 0 {
                    dest.push((id - 1) as u8);
                } else {
                    dest.extend_from_slice(&id.to_le_bytes()[..tag as usize + 1]);
                }
            } else {
                let rule = &mut self.rules[(id & 0x3fff_ffff) as usize];
''' + ep[b:]
(here/'endpoint.rs').write_text(ep)
