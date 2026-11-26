"""Add read-only capacity observers to the exact frozen one- and two-level implementations."""
from pathlib import Path
import sys
here=Path(__file__).resolve().parent
forms=here.parents[1]
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
for arm,source in (("scalar",forms/"f2c"),("fixed",forms/"f2-frontier"),("refill",here.parent),("double",here.parent/"second")):
    dest=out/arm;dest.mkdir(exist_ok=True)
    first=(source/"candidate.rs").read_text().replace("../f2c/second.rs","second.rs")
    second=(source/"second.rs" if arm=="double" else forms/"f2c/second.rs").read_text()
    first_path="0" if arm=="scalar" else "self.lane_paths.iter().map(|v|v.capacity()).sum::<usize>()"
    second_path="0" if arm!="double" else "self.paths.iter().map(|v|v.capacity()*4).sum::<usize>()"
    second+='''
impl Second {
    pub(super) fn capacities(&self)->(usize,usize) {
        let paths=LANE;
        let arrays=(self.count.capacity()+self.base.capacity()+self.epoch.capacity()+self.page.capacity()+self.edges.capacity()+self.links.capacity())*4
            +self.child.capacity()*std::mem::size_of::<usize>()+self.masks.capacity()*8
            +self.nodes.capacity()*std::mem::size_of::<Node>()+self.runs.capacity()*std::mem::size_of::<TokenRun>()
            +(self.selectors.capacity()+self.words.capacity()+self.merged.capacity())*std::mem::size_of::<Piece>()
            +self.dictionary.capacity();
        (arrays,paths)
    }
}
'''.replace("LANE",second_path)
    first+='''
impl Candidate {
    pub fn audit(&mut self, r:&Runs,n:usize,p:u32,out:&mut Vec<u8>)->(bool,[usize;4]) {
        let took=self.attempt::<false>(r.records(),n,p,out);
        let paths=LANE;
        let first=self.runs.capacity()*std::mem::size_of::<Run>()+self.nodes.capacity()*std::mem::size_of::<Node>()
          +self.page_first.capacity()*4+self.starts.capacity()*8+self.edges.capacity()+self.dict.capacity()
          +(self.selectors.capacity()+self.merged.capacity()+self.words.capacity())*std::mem::size_of::<Piece>()
          +self.links.capacity()*4;
        let (second,sp)=self.second.capacities();
        (took,[first,paths,second,sp])
    }
}
'''.replace("LANE",first_path)
    (dest/"candidate.rs").write_text(first)
    (dest/"second.rs").write_text(second)
