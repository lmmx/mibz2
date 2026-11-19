from pathlib import Path
import subprocess,sys
from hashlib import sha256
here=Path(__file__).resolve().parent
root=here.parents[2]
out=Path(sys.argv[1]).resolve()
frozen=out/'frozen';rev='badb2aec3624bd65606c429b2e6f79e9d9d0050f'
j='docs/journal/2026-11-18-open-return/';r='research/astra-e/'
files=[j+f for f in ['open_return.rs','endpoint.rs','segments.rs']]
files += [r+f for f in ['edge-budget/gate.rs','edge-budget/prepare.py','narrow-short/prepare.py','narrow-short/ahead.rs','narrow-endpoint/prepare.py','narrow-endpoint/candidate.rs','short-words/endpoint.rs','lookahead/writer.rs']]
for rel in files:
    target=frozen/rel;target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(subprocess.check_output(['git','-C',root,'show',rev+':'+rel]))
subprocess.run([sys.executable,frozen/r/'edge-budget/prepare.py'],check=True)
key=frozen/r/'narrow-short/constructor_ahead.rs'
source=key.read_text()
assert sha256(source.encode()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
import re
source=re.sub(r'(#\[path = ")([^"]+)"',lambda m:m[1]+str((key.parent/m[2]).resolve())+'"',source)
source=source.replace('    edges: Vec<u32>,','    last_mark:[u32;2],\n    edges: Vec<u32>,')
source=source.replace('        if SEGMENTS {','        self.last_mark=[mark,bits];\n        if SEGMENTS {',1)
source+='''
impl Kernel {
    pub fn walk_counts(&self)->[usize;4] {
        let [mark,bits]=self.last_mark;
        let mask=(1<<bits)-1;
        let mut row=mark;
        let mut counts=[0;4];
        loop {
            let edge=self.edges[row as usize];
            let code=if self.extra.is_empty(){edge>>bits}else{self.extra[row as usize]};
            let next=edge&mask;
            counts[0]+=1;counts[1]+=usize::from(code>=256);
            counts[2]+=usize::from(row/16==next/16);counts[3]+=usize::from(row/1024==next/1024);
            row=next;if row==mark {break}
        }
        counts
    }
}
'''
stock=source+'''
impl Kernel {
    pub fn capacities(&self)->[usize;2] {[4*(self.edges.capacity()+self.extra.capacity()),0]}
}
'''
(out/'stock.rs').write_text(stock)
s=source.replace('    edges: Vec<u32>,','    select:Selector,\n    edges: Vec<u32>,')
s=s.replace('limit: u32, mut visit: impl FnMut(u8, u32, u32, bool)) {','limit: u32, moved:&[u32], mut visit: impl FnMut(u8, u32, u32, bool)) {\n    let mut moves=moved.iter().copied();let mut next=moves.next().unwrap_or(u32::MAX);')
s=s.replace('if width > limit && (pos..end).contains(&p) {','let change=pos==next;\n        if change {next=moves.next().unwrap_or(u32::MAX);}\n        let keep=(width<=limit)^change;\n        if !keep && (pos..end).contains(&p) {')
s=s.replace('visit(c, pos, width, width <= limit);','visit(c,pos,width,keep);')
s=s.replace('each_run(records, p, limit, |','each_run(records, p, limit, &self.select.moved, |')
s+='\n'+(here/'select.inc').read_text()+'''
impl Kernel {
    pub fn run_selected<const POLICY:u8>(&mut self,recs:&[u16],n:usize,p:u32,out:&mut Vec<u8>)->(Counts,[u64;2],usize){
        let t0=crate::clock();
        let (m,h,q)=self.select.plan::<POLICY>(recs,n,p);
        let t1=crate::clock();
        let c=self.run::<true,true,false>(recs,n,p,16,out);
        let t2=crate::clock();
        assert_eq!((c.retained,c.thick),(m as usize+q,h as usize));
        (c,[t1-t0,t2-t1],q)
    }
    pub fn capacities(&self)->[usize;2] {
        [4*(self.edges.capacity()+self.extra.capacity()),self.select.near.capacity()*size_of::<Near>()+self.select.moved.capacity()*4]
    }
}
'''
(out/'changed.rs').write_text(s)
(out/'main.rs').write_text('#![allow(dead_code)]\nmod stock;mod changed;\n'+(frozen/r/'edge-budget/gate.rs').read_text()+(here/'main.rs').read_text())
