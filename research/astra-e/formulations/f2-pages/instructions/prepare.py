"""Add once-per-phase ISA dispatch to the frozen counted F2 kernel."""
from hashlib import sha256
from pathlib import Path
d=Path(__file__).resolve().parent
s=(d.parents[1]/'f2-front-counts/candidate.rs').read_text()
assert sha256(s.encode()).hexdigest()=='0f788874ed1149327bb9c2693cb15d2b2ef3d405cfb892fd777380f6af0fd986'
s=s.replace('path = "../f2c/', 'path = "../../f2c/')
s=s.replace('#[derive(Default)]\npub struct Candidate {','pub type Candidate = Dispatch<2>;\n\n#[derive(Default)]\npub struct Dispatch<const MODE: u8> {')
s=s.replace('impl Candidate {','impl<const MODE:u8> Dispatch<MODE> {')
s=s.replace('impl InverseBwt for Candidate {', 'impl<const MODE:u8> InverseBwt for Dispatch<MODE> {')
s=s.replace('    fn build(&mut self, quote: u32, tokens: u32) {', (d/'dispatch.inc.rs').read_text()+'\n    #[inline(always)]\n    fn build_body(&mut self, quote: u32, tokens: u32) {')
s=s.replace('    fn prepare_counted(&mut self, caps: &[u16], n: u32, counts: &[u32; 256])', '    #[inline(always)]\n    fn prepare_body(&mut self, caps: &[u16], n: u32, counts: &[u32; 256])')
(d/'candidate.rs').write_text(s)
