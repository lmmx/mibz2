impl Candidate {
    pub fn export(&mut self, caps:&[u16],n:u32,p:u32,out:&mut dyn std::io::Write, stream:usize) {
        let (section,m)=self.prepare(caps,n).unwrap();
        assert!(self.transfer_mark(p,section).is_some());
        self.build(section,m);
        for (id,node) in self.nodes.iter().enumerate() {
            let dict=if node.word==NONE {0} else {self.words[node.word as usize].width};
            writeln!(out,"{stream},{id},{},{},{},{dict}",node.parent,node.end-node.edge,node.sels).unwrap();
        }
    }
}
