"""Replay contracted-node rank answers into the immutable dynamic scheduler's publication code."""
from pathlib import Path
from hashlib import sha256
import sys
here=Path(__file__).resolve().parent
s=(here.parent/'candidate.rs').read_text()
assert sha256(s.encode()).hexdigest()=="1a659b1c65663db8901146d58623261c238c87dcd1a5ce6cb234c372cd59d4bf"
a=s.index('    fn build(&mut self, quote:')
b=s.index('    #[inline]\n    fn emit',a)
original=s[a:b]
capture=original.replace('fn build(&mut self, quote: u32, tokens: u32)', 'fn capture(&mut self, quote:u32, tokens:u32, tape:&mut Tape)')
loc='                self.nodes[i].edge = self.edges.len() as u32;'
assert capture.count(loc)==1
capture=capture.replace(loc, '''                let start=tape.labels.len() as u32;
                tape.labels.extend_from_slice(&paths[lane]);
                tape.nodes.push((i as u32,lo,hi,k as u32,start,tape.labels.len() as u32));
'''+loc)
beg=original[:original.index('        const LANES:')]
beg=beg.replace('fn build(&mut self, quote: u32, tokens: u32)', 'fn publish(&mut self, quote:u32,tokens:u32,tape:&Tape)')
walk=original[original.index(loc):original.index('            }\n        }\n        self.lane_paths = paths;')]
walk=walk.replace('self.edges.extend_from_slice(&paths[lane]);','self.edges.extend_from_slice(&tape.labels[begin as usize..end as usize]);')
publish=beg+'''        for &(node,lo,hi,first,begin,end) in &tape.nodes {
            let i=node as usize;
            let mut k=first as usize;
'''+walk+'        }\n    }\n'
s=s[:b]+capture+publish+s[b:]
s=s.replace('../f2c/second.rs','second.rs')
s+='''
#[derive(Default)] pub struct Tape {nodes:Vec<(u32,u32,u32,u32,u32,u32)>,labels:Vec<u8>}
impl Candidate {
    pub fn record(&mut self,r:&Runs,n:usize)->Tape {
        let mut t=Tape::default();
        let (section,m)=self.prepare(r.records(),n as u32).unwrap();
        self.capture(section,m,&mut t);t
    }
    pub fn timing(&mut self,r:&Runs,n:usize,p:u32,out:&mut Vec<u8>,oracle:Option<&Tape>)->[u64;4] {
        let t0=crate::thread_cpu_ns();
        let (section,m)=self.prepare(r.records(),n as u32).unwrap();
        let (row,skip)=self.transfer_mark(p,section).unwrap();
        let t1=crate::thread_cpu_ns();
        if let Some(t)=oracle {self.publish(section,m,t);} else {self.build(section,m);}
        let t2=crate::thread_cpu_ns();
        self.inverse_merge();
        let t3=crate::thread_cpu_ns();
        let root=&self.merged[self.nodes[0].out as usize..];
        assert!(self.second.invert(root,&self.nodes,&self.words,&self.dict,row,skip,m,n as u32,out));
        std::hint::black_box(&out);let t4=crate::thread_cpu_ns();
        [t1-t0,t2-t1,t3-t2,t4-t3]
    }
}
'''
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
(out/'candidate.rs').write_text(s)
