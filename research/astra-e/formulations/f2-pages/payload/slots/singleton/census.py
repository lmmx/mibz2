"""Tabulate real lane interval widths in the exact compact preloaded scheduler."""
from pathlib import Path
import re,sys
here=Path(__file__).resolve().parent
file=here.parent/'candidate.rs'
s=file.read_text()
def resolve(m):return m[0].replace(m[1],str((file.parent/m[1]).resolve()))
s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
s=s.replace('pub type Candidate = Dispatch<2>;','pub type Candidate = Dispatch<2,2>;')
s=s.replace('    fallback: Segmented,','    counts:[[u64;4];9],\n    singles:[u64;9],\n    fallback: Segmented,')
s=s.replace('        const LANES: usize = 8;','        self.counts=[[0;4];9]; self.singles=[0;9];\n        const LANES: usize = 8;')
a=s.index('            let mut done=0;')
s=s[:a]+'''            let mut ones=0;
            for lane in 0..LANES {
                if live&(1<<lane)==0 { continue }
                let r=self.runs[index[lane]];
                let w=width[lane];
                let width_bin=(31-w.leading_zeros()).min(8) as usize;
                let hi=lo[lane]+w;
                self.counts[width_bin][0]+=1;
                self.counts[width_bin][1]+=u64::from(w);
                self.counts[width_bin][2]+=u64::from(r.byte()==QUOTE);
                self.counts[width_bin][3]+=u64::from(hi>r.position());
                ones+=usize::from(w==1);
            }
            self.singles[ones]+=1;
'''+s[a:]
s+='''
impl Dispatch<2,2> {
    pub fn report(&self,block:u64,rows:usize){
        for i in 0..9 {
            let r=self.counts[i];
            println!("{block},{rows},{i},{},{},{},{},{}",r[0],r[1],r[2],r[3],self.singles[i]);
        }
    }
}
'''
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
(out/'candidate.rs').write_text(s)
s=(here.parents[1]/'uniform/census.rs').read_text().replace('kind,block,rows,cuts_or_wave_hits,pages_or_waves,longest_rows,lookups,unary,longest_hits,longest_unary','block,rows,bin_or_singleton_lanes,queries,width_sum,delimiter,over_end,waves')
(out/'census.rs').write_text(s)
