"""Count exact repeat-within-original-run opportunities in the preloaded F2 wave."""
from pathlib import Path
import re,sys
h=Path(__file__).resolve().parent
file=h.parent/'slots/candidate.rs'
s=file.read_text()
def resolve(m):return m[0].replace(m[1],str((file.parent/m[1]).resolve()))
s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
s=s.replace('pub type Candidate = Dispatch<2>;','pub type Candidate = Dispatch<2,2>;')
s=s.replace('    fallback: Segmented,','    repeat:[u64;8],\n    fallback: Segmented,')
s=s.replace('        const LANES: usize = 8;','        self.repeat=[0;8];\n        const LANES: usize = 8;')
s=s.replace('            let mut done=0;', '''            for lane in 0..LANES {
                if live & (1<<lane) == 0 {continue}
                let k=index[lane];
                let r=self.runs[k];
                self.repeat[0]+=1;
                let hi=lo[lane]+width[lane];
                if r.retires(hi) {continue}
                self.repeat[1]+=1;
                let start=if k==0 {0} else {self.runs[k-1].position()};
                let end=r.position();
                let jump=r.delta as i32;
                assert_ne!(jump,0,"a zero-displacement non-delimiter interval cannot reach the root");
                let future=if jump>0 {(end-hi)/jump as u32} else {(lo[lane]-start)/jump.unsigned_abs()};
                if future==0 {continue}
                self.repeat[2]+=1;
                self.repeat[3]+=u64::from(jump>0);
                self.repeat[4]+=u64::from(jump<0);
                self.repeat[5]=self.repeat[5].max(u64::from(future));
                self.repeat[6]+=u64::from(future>=2);
                self.repeat[7]+=u64::from(future>=8);
            }
            let mut done=0;''')
s+='''
impl Dispatch<2,2> {
    pub fn report(&self,block:u64,rows:usize){
        let r=self.repeat;
        println!("{block},{rows},{},{},{},{},{},{},{},{}",r[0],r[1],r[2],r[3],r[4],r[5],r[6],r[7]);
    }
}
'''
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
(out/'candidate.rs').write_text(s)
s=(h.parent/'uniform/census.rs').read_text().replace('kind,block,rows,cuts_or_wave_hits,pages_or_waves,longest_rows,lookups,unary,longest_hits,longest_unary','block,rows,queries,unary,internal_transitions,positive,negative,max_further,at_least_two,at_least_eight')
(out/'census.rs').write_text(s)
