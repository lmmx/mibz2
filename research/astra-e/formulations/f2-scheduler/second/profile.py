"""Time the unchanged second-level stages and export the completed recursive lookup DAG."""
from pathlib import Path
import sys
here=Path(__file__).resolve().parent
root=here.parents[1]
out=Path(sys.argv[1])
out.mkdir(parents=True,exist_ok=True)
s=(root/"f2c/second.rs").read_text().replace("    runs: Vec<TokenRun>,","    runs: Vec<TokenRun>,\n    pub(super) laps:[u64;5],")
cuts=[("        let d = self.column(", "let t0=crate::thread_cpu_ns();"),
      ("        self.discover(d, first_words, bytes);","let t1=crate::thread_cpu_ns();"),
      ("        self.merge();","let t2=crate::thread_cpu_ns();"),
      ("        let bits = 32 - (token_count - 1).leading_zeros();","let t3=crate::thread_cpu_ns();"),
      ("        out.clear();","let t4=crate::thread_cpu_ns();"),
      ("        true\n    }\n}","let t5=crate::thread_cpu_ns(); self.laps=[t1-t0,t2-t1,t3-t2,t4-t3,t5-t4];")]
for anchor,inject in cuts:
    assert s.count(anchor)==1,anchor
    s=s.replace(anchor,inject+"\n"+anchor)
s+='''
impl Second {
    pub(super) fn export(&self, stream:usize, out:&mut dyn std::io::Write) {
        for (id,node) in self.nodes.iter().enumerate() {
            let bytes=if node.word==NONE {0} else {self.words[node.word as usize].width};
            writeln!(out,"{stream},{id},{},{},{},{bytes}",node.parent,node.end-node.edge,node.sels).unwrap();
        }
    }
}
'''
(out/"second.rs").write_text(s)
s=(here.parent/'candidate.rs').read_text().replace('../f2c/second.rs','second.rs')
s+='''
impl Candidate {
    pub fn profile(&mut self,r:&Runs,n:usize,p:u32,out:&mut Vec<u8>,round:usize,stream:usize,f:&mut dyn std::io::Write)->[u64;6] {
        let t0=crate::thread_cpu_ns();
        assert!(self.attempt::<false>(r.records(),n,p,out));
        let t1=crate::thread_cpu_ns();
        if round==0 {self.second.export(stream,f);}
        let [col,discover,merge,table,emit]=self.second.laps;
        [col,discover,merge,table,emit,t1-t0]
    }
}
'''
(out/"candidate.rs").write_text(s)
