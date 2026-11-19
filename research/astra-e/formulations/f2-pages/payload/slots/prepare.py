"""Specialize response handling only while all eight dynamic discovery slots are filled."""
from pathlib import Path
from hashlib import sha256
h=Path(__file__).resolve().parent
s=(h.parent/'construction/eager.rs').read_text()
assert sha256(s.encode()).hexdigest()=='82704548e38df71d53eb7eab1b12674d62b0987d7fc789c5d1bc00cdff6213dc'
s=s.replace('pub struct Dispatch<const MODE: u8>', 'pub struct Dispatch<const MODE:u8,const SCHEDULE:u8=1>')
s=s.replace('impl<const MODE:u8>', 'impl<const MODE:u8,const SCHEDULE:u8>')
s=s.replace('Dispatch<MODE>', 'Dispatch<MODE,SCHEDULE>')
s=s.replace('impl Run {','''impl Run {
    #[inline]
    fn retires(self,hi:u32)->bool {self.position_byte<(hi<<8) || self.byte()==QUOTE}''')
a=s.index('            let mut done = 0;')
b=s.index('            live ^= done;',a)
s=s[:a]+(h/'respond.inc.rs').read_text()+s[b:]
s=s.replace('    fallback: Segmented,','    #[cfg(slot_counts)]\n    wave_counts:[[u64;9];9],\n    fallback: Segmented,')
s=s.replace('        const LANES: usize = 8;', '        #[cfg(slot_counts)] { self.wave_counts=[[0;9];9]; }\n        const LANES: usize = 8;')
s=s.replace('#[test]\nfn arbitrary_marks() {\n    let mut c = Candidate::default();',
            'fn verify_marks<const SCHEDULE:u8>() {\n    let mut c = Dispatch::<2,SCHEDULE>::default();')
s=s.replace('#[test]\nfn packed_lf_and_boundaries()', '#[test]\nfn arbitrary_marks() { verify_marks::<1>(); verify_marks::<2>(); verify_marks::<3>(); }\n\n#[test]\nfn packed_lf_and_boundaries()')
s+='''
#[cfg(slot_counts)]
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    pub fn report(&self,block:u64) {
        for active in 1..=8 {
            for retired in 0..=active {
                println!("wave,{block},{active},{retired},{}",self.wave_counts[active][retired]);
            }
        }
    }
}
'''
(h/'candidate.rs').write_text(s)
