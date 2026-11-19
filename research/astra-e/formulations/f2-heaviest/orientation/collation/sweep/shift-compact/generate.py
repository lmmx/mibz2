"""Generate the projected-row Compact variant from the checked clipped generator."""
from pathlib import Path
import hashlib
import subprocess
import sys
D=Path(__file__).resolve().parent
CLIP=D.parents[1]/'clipped/native'
def once(s,a,b):
    assert s.count(a)==1,(a,s.count(a))
    return s.replace(a,b)
def main(out):
    subprocess.run([sys.executable,'-B',CLIP/'generate.py',out],check=True)
    s=(out/'shared.rs').read_text()
    # The previous frozen generator byte-checks original Compact and second sources.
    pinned='''b0377bd9d5fdfaa9c8a8e3c17a84fae24dab9b8f45ddb02bbf922e12c1e2a1b2'''
    assert hashlib.sha256(s.encode()).hexdigest()==pinned
    s=once(s,'const LIVE:u32=1<<31;', 'const LIVE:u32=1<<31;\n'+(D/'projection.rs').read_text())
    s=once(s,'    section:(u32,u32),','    plan:Plan,\n    pub measures:[u32;8],\n    section:(u32,u32),')
    s=once(s,'    fn prepare<const ONLY:bool>(', (D/'index.rs').read_text()+'\n    fn prepare<const ONLY:bool>(')
    s=once(s,'        self.bases.clear();','''        self.bases.clear();
        for &(from,base) in &self.plan.types{
            self.bases.push(base);
            let pos=self.dict.len() as u32;
            self.dict.extend_from_slice(&self.plan.prefix[from..]);
            self.words.push(Piece{id:pos,width:self.dict.len() as u32-pos});
        }''')
    s=once(s,'        self.nodes.push(root);','''        self.nodes.push(root);
        if tokens==0{return;}''')
    s=once(s,'if ONLY || job[lane]!=0{paths[lane].push(n.first);}','''if job[lane]!=0{paths[lane].push(n.first);}
                else{paths[lane].extend(self.plan.prefix.iter().rev().copied());}''')
    s=once(s,'            live ^= done;', '''            #[cfg(shift_counts)] {self.measures[0]+=live.count_ones();}
            live ^= done;''')
    s=s.replace('mapped-quote','mapped')
    s=once(s,'if !ONLY{self.dict.push(r.byte());}','self.dict.push(r.byte());')
    a=s.index('    pub fn first<const ONLY:bool>(');b=s.index('    pub fn object(',a)
    s=s[:a]+(D/'first.rs').read_text()+s[b:]
    a=s.index('                // Valid marked inputs may exceed');b=s.index('\n            }',a)
    s=s[:a]+'''                let root=&self.merged[self.nodes[0].out as usize..];
                let mut links=vec![(0u32,0u32);m as usize];let mut dest=0;
                for p in root{
                    let node=&mut self.nodes[p.id as usize];
                    for l in &mut links[node.base as usize..(node.base+p.width) as usize]{
                        *l=(node.word,dest);dest+=1;
                    }
                    node.base+=p.width;
                }
                let(mut row,mut skip)=(row,skip);out.clear();out.reserve(n as usize);
                while out.len()<n as usize{
                    let (id,y)=links[row as usize];let w=self.words[id as usize];
                    let take=(w.width-skip).min(n-out.len() as u32);let pos=w.id+skip;
                    out.extend_from_slice(&self.dict[pos as usize..(pos+take) as usize]);skip=0;row=y;
                }'''+s[b:]
    s=once(s,'+bytes(&self.bases)','''+bytes(&self.bases)+bytes(&self.plan.levels)
            +self.plan.levels.iter().map(|x|bytes(&x.parts)).sum::<usize>()+bytes(&self.plan.forbid.parts)
            +bytes(&self.plan.prefix)+bytes(&self.plan.early)+bytes(&self.plan.types)''')
    (out/'projected.rs').write_text(s)
    common=(CLIP/'main.rs').read_text()
    helpers=common[common.index('use std::collections::HashMap;'):common.index('#[derive(Default)] struct Library')]
    (out/'main.rs').write_text((D/'main.rs').read_text().replace('// INPUT_AND_DENSE_HELPERS',helpers))
if __name__=='__main__':main(Path(sys.argv[1]))
