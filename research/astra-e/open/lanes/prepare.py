"""Pin the queued open constructor and substitute only its residual writer."""
from hashlib import sha256
from pathlib import Path

here = Path(__file__).resolve().parent
root = here.parents[3]
source = (root / 'docs/journal/2026-11-18-open-return/open_return.rs').read_bytes()
assert sha256(source).hexdigest() == 'b02a1d2ed7490a0dbef8f5ecf464153d75d2cd71b42993807a3a713bc18bbfc1'
s = source.decode()
ctor = s[:s.index('    pub fn run<const INDUCE:')]
ctor = ctor.replace('use crate::bwt::{InverseBwt, Segmented};', 'use crate::bwt::{InverseBwt, LazyCells};')
ctor = ctor.replace('segments: segments::Segments,', 'tapes: writer::Tapes,')
emit = s[s.index('fn emit('):s.index('fn records_of(')]
candidate = (here.parent / 'candidate.rs').read_text()
gate = candidate[candidate.index('#[derive(Default)]\npub struct Candidate'):candidate.index('#[cfg(test)]')]
gate = gate.replace('self.open.run::<16>(r.records(), n as u32, p, out);',
'''let (_, mark, bits) = self.open.construct::<true,true>(r.records(),n as u32,p,16);
            self.open.tapes.write(&self.open.edges,&self.open.extra,&self.open.spelling,
                                  &self.open.offsets,n,mark,bits,out);''')
s = ctor+'}\n'+emit+'\nmod writer;\n#[path = "../../../../docs/journal/2026-11-18-open-return/endpoint.rs"]\nmod endpoint;\n'+gate
(here/'candidate.rs').write_text(s)
