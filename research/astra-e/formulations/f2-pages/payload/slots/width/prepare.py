"""Parameterize live width in the frozen compact preloaded wave."""
from pathlib import Path
from hashlib import sha256
import re
h=Path(__file__).resolve().parent
s=(h.parent/'candidate.rs').read_text()
assert sha256(s.encode()).hexdigest()=='84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06'
s=s.replace('path = "../../../f2c/','path = "../../../../f2c/')
s=s.replace('pub type Candidate = Dispatch<2>;','''struct Paths<const WIDTH:usize>([Vec<u8>;WIDTH]);
impl<const WIDTH:usize> Default for Paths<WIDTH> {
    fn default()->Self { Self(std::array::from_fn(|_|Vec::new())) }
}

pub type Candidate = Dispatch<2,2,16>;''')
s=s.replace('pub struct Dispatch<const MODE:u8,const SCHEDULE:u8=1>', 'pub struct Dispatch<const MODE:u8,const SCHEDULE:u8=2,const LANES:usize=8>')
s=s.replace('impl<const MODE:u8,const SCHEDULE:u8>', 'impl<const MODE:u8,const SCHEDULE:u8,const LANES:usize>')
s=s.replace('Dispatch<MODE,SCHEDULE>', 'Dispatch<MODE,SCHEDULE,LANES>')
s=s.replace('lane_paths: [Vec<u8>; 8],','lane_paths: Paths<LANES>,').replace('std::mem::take(&mut self.lane_paths)','std::mem::take(&mut self.lane_paths).0').replace('self.lane_paths = paths;', 'self.lane_paths=Paths(paths);')
s=s.replace('        const LANES: usize = 8;\n','')
s=s.replace('const ALL: u32 = (1 << LANES) - 1;', 'let all: u32 = (1 << LANES) - 1;')
s=re.sub(r'\bALL\b','all',s)
s=s.replace('[[u64;9];9]','[[u64;25];25]').replace('[[0;9];9]','[[0;25];25]').replace('1..=8 {', '1..=LANES {').replace('[u64; 9]', '[u64; 25]').replace('self.histogram = [0; 9]', 'self.histogram = [0; 25]')
s=s.replace('fn verify_marks<const SCHEDULE:u8>()','fn verify_marks<const LANES:usize>()').replace('let mut c = Dispatch::<2,SCHEDULE>::default();','let mut c = Dispatch::<2,2,LANES>::default();').replace('verify_marks::<1>(); verify_marks::<2>(); verify_marks::<3>();','verify_marks::<4>(); verify_marks::<8>(); verify_marks::<12>(); verify_marks::<16>(); verify_marks::<24>();')
(h/'candidate.rs').write_text(s)
