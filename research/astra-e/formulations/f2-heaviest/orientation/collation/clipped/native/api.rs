#[allow(dead_code)]
impl<PARAMS> TYPE {
    pub fn first(&mut self,caps:&[u16],freq:&[u32;256],n:u32,p:u32 SECTION_SIGNATURE)->Option<(Option<(u32,u32)>,u32)>{
        let (lo,m)=START_MARK;
        let mark=self.transfer_mark(p,lo);
        self.build(lo,m);self.inverse_merge();
        AFTER_MERGE
        Some((mark,m))
    }
    pub fn object(&self)->(Vec<(u32,Vec<u8>)>,Vec<(u32,u32)>){
        let mut words=Vec::new();
        for node in &self.nodes{
            if node.word==NONE{continue;}
            let w=self.words[node.word as usize];
            words.push((node.base,self.dict[w.id as usize..(w.id+w.width) as usize].to_vec()));
        }
        words.sort_by_key(|w|w.0);
        let root=&self.merged[self.nodes[0].out as usize..];
        let runs=root.iter().map(|r|(self.nodes[r.id as usize].base,r.width)).collect();
        (words,runs)
    }
    pub fn finish(&mut self,mark:Option<(u32,u32)>,m:u32,n:u32,p:u32,out:&mut Vec<u8>)->bool{
        let Some((row,skip))=mark else{
            let mut x=p;out.clear();
            loop{
                let r=self.runs[self.locate(x)];out.push(r.byte());x=r.map(x);
                if x==p{break;}
            }
            out.reverse();
            while out.len()<n as usize{let take=out.len().min(n as usize-out.len());out.extend_from_within(..take);}
            return false;
        };
        let root=&self.merged[self.nodes[0].out as usize..];
        let second=self.second.invert(root,&self.nodes,&self.words,&self.dict,row,skip,m,n,out);
        if !second{
            if !self.expand(row,skip,m,n,out){
                // Valid marked inputs may exceed the compact first-level link bit budget.
                let mut x=p;out.clear();
                loop{
                    let r=self.runs[self.locate(x)];out.push(r.byte());x=r.map(x);
                    if x==p{break;}
                }
                out.reverse();
                while out.len()<n as usize{let take=out.len().min(n as usize-out.len());out.extend_from_within(..take);}
            }
        }
        second
    }
}
