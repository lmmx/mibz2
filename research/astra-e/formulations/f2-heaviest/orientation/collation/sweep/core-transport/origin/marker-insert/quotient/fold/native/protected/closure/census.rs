
pub struct Feature {
    pub types:Vec<(u32,u32,u32,u32)>, // source base, mass, eligible successor base, native copy cost
    pub support:Vec<Vec<u32>>,        // original source-base IDs, one entry per feasible state
    pub sizes:[usize;5],              // Q, pieces examined, all incidence, qualifying incidence, visited rows
    pub cpu:[u64;4],                  // eligibility, index, interval discovery, type table
}
impl<const M:u8,const S:u8> Dispatch<M,S>{
    pub fn set_policy(&mut self,v:Vec<u32>){self.fold_scratch.forced=Some(v);}
    pub fn census(&mut self,m:u32)->Feature{
        let t=crate::now();
        let root=&self.merged[self.nodes[0].out as usize..];
        let f=&mut self.fold_scratch;
        f.forced=None;f.skip_choose=true;f.recognize(root,&self.nodes,self.words.len(),m);f.skip_choose=false;
        let t1=crate::now();
        let d=self.second.support_index(root,&self.nodes,self.words.len(),m);
        let t2=crate::now();
        let (support,sizes)=self.second.support(d,&f.classes);
        let t3=crate::now();
        let types=f.order.iter().map(|&k|{
            let c=f.classes[k];
            let (g,b)=if c.succ==NONE{(NONE,0)}else{(f.classes[c.succ as usize].base,
                self.words[k].width+self.words[c.succ as usize].width)};
            (c.base,c.mass,g,b)
        }).collect();
        Feature{types,support,sizes,cpu:[t1-t,t2-t1,t3-t2,crate::now()-t3]}
    }
}
