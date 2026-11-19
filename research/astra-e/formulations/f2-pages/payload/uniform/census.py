"""Instrument the frozen eager-end refill to count actual directory and payload opportunities."""
from pathlib import Path
import re,sys
here=Path(__file__).resolve().parent
source=here.parent/'construction/eager.rs'
out=Path(sys.argv[1]).resolve()
out.mkdir(parents=True,exist_ok=True)
s=source.read_text()
def resolve(m):return m[0].replace(m[1],str((source.parent/m[1]).resolve()))
s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
s=s.replace('    fallback: Segmented,','    fallback: Segmented,\n    census: Census,')
a=s.index('        const LANES: usize = 8;')
s=s[:a]+'''        self.census=Census::default();
        let n=self.runs.last().unwrap().position();
        for (p,&word) in self.starts.iter().enumerate() {
            let cuts=word.count_ones() as usize;
            self.census.pages[cuts.min(4)]+=1;
            let first=p*64;
            let width=(n as usize-first).min(64);
            let (lo,hi)=longest(word,width);
            self.census.longest_rows[cuts.min(4)]+=hi-lo;
        }
'''+s[a:]
s=s.replace('            let mut done = 0;','            let mut done = 0;\n            let mut uniform_in_wave=0;')
old='''                let r = self.runs[k];
                if r.position_byte < ((lo[lane] + width[lane]) << 8) || r.byte() == QUOTE {'''
assert s.count(old)==1
s=s.replace(old,'''                let r = self.runs[k];
                let row=lo[lane] as usize;
                let page=row>>6;
                let mask=self.starts[page];
                let cuts=mask.count_ones() as usize;
                let class=cuts.min(4);
                let width_here=(n as usize-page*64).min(64);
                let (from,to)=longest(mask,width_here);
                let hit=(from..to).contains(&(row&63));
                let unary=r.position_byte >= ((lo[lane] + width[lane]) << 8) && r.byte()!=QUOTE;
                self.census.lookups[class]+=1;
                self.census.unary[class]+=usize::from(unary);
                self.census.longest_hits[class]+=usize::from(hit);
                self.census.longest_unary[class]+=usize::from(hit&&unary);
                uniform_in_wave+=usize::from(cuts==0);
                if r.position_byte < ((lo[lane] + width[lane]) << 8) || r.byte() == QUOTE {''')
s=s.replace('            live ^= done;', '            self.census.wave_hits[uniform_in_wave]+=1;\n            live ^= done;')
s+='''
#[derive(Default)]
struct Census{
    pages:[usize;5], longest_rows:[usize;5],lookups:[usize;5],unary:[usize;5],
    longest_hits:[usize;5],longest_unary:[usize;5],wave_hits:[usize;9],
}
fn longest(mut mask:u64,width:usize)->(usize,usize) {
    let (mut start,mut best)=(0,(0,0));
    while mask!=0 {
        let end=mask.trailing_zeros() as usize;
        if end-start > best.1-best.0 { best=(start,end); }
        start=end;
        mask&=mask-1;
    }
    if width-start>best.1-best.0 {(start,width)} else {best}
}
impl Dispatch<2>{
    pub fn report(&self,block:u64,rows:usize){
        let c=&self.census;
        for i in 0..5 {
            println!("page,{block},{rows},{i},{},{},{},{},{},{}",
                c.pages[i],c.longest_rows[i],c.lookups[i],c.unary[i],c.longest_hits[i],c.longest_unary[i]);
        }
        for (hits,count) in c.wave_hits.iter().enumerate() {
            println!("wave,{block},{rows},{hits},{count},0,0,0,0,0");
        }
    }
}
'''
(out/'candidate.rs').write_text(s)
(out/'census.rs').write_text((here/'census.rs').read_text())
