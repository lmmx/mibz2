"""Observe last-target prediction at the time each full wave obtains its run."""
from pathlib import Path
import re,sys
from hashlib import sha256
here=Path(__file__).resolve().parent
file=here.parent/'candidate.rs'
s=file.read_text()
assert sha256(s.encode()).hexdigest()=='84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06'
def resolve(m):return m[0].replace(m[1],str((file.parent/m[1]).resolve()))
s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
s=s.replace('pub type Candidate = Dispatch<2>;','pub type Candidate = Dispatch<2,2>;')
s=s.replace('    fallback: Segmented,','    observations:[u64;22],\n    fallback: Segmented,')
s=s.replace('        let mut next = 0;','''        self.observations=[0;22];
        let mut next_target=vec![usize::MAX;self.runs.len()];
        let mut from=[usize::MAX;LANES];
        let mut predicted=[usize::MAX;LANES];
        let mut next = 0;''')
s=s.replace('                job[lane] = next;','                from[lane]=usize::MAX;\n                job[lane] = next;')
a=s.index('            let mut done=0;')
s=s[:a]+'''            let mut wave_hits=0;
            for lane in 0..LANES {
                if live & (1<<lane)==0 {continue}
                self.observations[0]+=1;
                let src=from[lane];
                if src==usize::MAX {continue}
                self.observations[1]+=1;
                let j=index[lane];
                let guess=predicted[lane];
                if guess==usize::MAX {
                    self.observations[2]+=1;
                } else {
                    let lo_guess=if guess==0 {0} else {self.runs[guess-1].position()};
                    let hi_guess=self.runs[guess].position();
                    assert_eq!(guess==j,lo_guess<=lo[lane] && lo[lane]<hi_guess);
                    if guess==j {
                        wave_hits+=1;
                        self.observations[3]+=1;
                        let r=self.runs[src];
                        let source_lo=if src==0 {0} else {self.runs[src-1].position()};
                        let left=r.map(source_lo)>=lo_guess;
                        let right=r.map(r.position())<=hi_guess;
                        self.observations[7+usize::from(left)+2*usize::from(right)]+=1;
                        self.observations[11]+=u64::from(j<65535);
                        self.observations[12]+=u64::from(j<4095);
                    } else {
                        self.observations[4]+=1;
                        self.observations[5]+=u64::from(guess.abs_diff(j)==1);
                        self.observations[6]+=u64::from(guess.abs_diff(j)<=4);
                    }
                }
                if guess!=j && next_target[src]!=j {
                    self.observations[13]+=1;
                    next_target[src]=j;
                }
            }
            for lane in 0..LANES {
                if live & (1<<lane)!=0 {
                    from[lane]=index[lane];
                    predicted[lane]=next_target[index[lane]];
                }
            }
            if wave_hits!=0 {self.observations[13+wave_hits]+=1;}
'''+s[a:]
s+='''
impl Dispatch<2,2> {
    pub fn report(&self,block:u64,rows:usize){
        print!("{block},{rows},{}",self.runs.len());
        for x in self.observations {print!(",{x}");}
        println!();
    }
}
'''
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
(out/'candidate.rs').write_text(s)
main=(here.parents[1]/'uniform/census.rs').read_text()
main=main.replace('kind,block,rows,cuts_or_wave_hits,pages_or_waves,longest_rows,lookups,unary,longest_hits,longest_unary','block,rows,runs,queries,continued,cold,hit,wrong,wrong_neighbor,wrong_within_four,hit_interior,hit_left,hit_right,hit_whole,hit_low16,hit_low12,writes,'+','.join(f'waves_{i}_hits' for i in range(1,9)))
(out/'census.rs').write_text(main)
