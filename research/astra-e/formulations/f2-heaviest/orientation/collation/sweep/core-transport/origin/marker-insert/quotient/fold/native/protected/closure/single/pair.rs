#[derive(Default,Clone,Copy)]
struct Pair {
    k:u32,j:u32,k_base:u32,j_base:u32,k_mass:u32,j_mass:u32,
    row:u32,skip:u32,moved:bool,reason:u8,
}
pub struct PairResult {
    pub reason:u8,pub moved:bool,pub old_runs:u32,pub new_runs:u32,pub removed:u32,
    pub eligible_ns:u64,pub index_ns:u64,pub applied:bool,pub stock:bool,
    pub first:Option<(crate::Obj,(u32,u32))>,pub snapshot:Option<(crate::Obj,(u32,u32))>,
}
impl<const M:u8,const S:u8> Dispatch<M,S> {
    fn literal_ids(&self)->(u32,u32){
        let (mut k,mut j)=(NONE,NONE);
        for (i,w) in self.words.iter().enumerate(){
            let at=w.id as usize;
            match w.width{
                2 if &self.dict[at..at+2]==b",\""=>j=i as u32,
                5 if &self.dict[at..at+5]==b"item\""=>k=i as u32,
                _=>continue,
            }
            if k!=NONE && j!=NONE{break;}
        }
        (k,j)
    }
    pub fn literal_finish(&mut self,mark:Option<(u32,u32)>,m:u32,n:u32,p:u32,route:usize,inspect:u8,out:&mut Vec<u8>)->PairResult{
        let root_start=self.nodes[0].out as usize;
        let mut r=PairResult{reason:0,moved:false,old_runs:(self.merged.len()-root_start) as u32,new_runs:0,
            removed:0,eligible_ns:0,index_ns:0,applied:false,stock:false,first:None,snapshot:None};
        if route>=2 {
            let t=crate::now();let mut pair=Pair::default();
            if let Some((row,skip))=mark{
                let (k,j)=self.literal_ids();
                if k==NONE{pair.reason=1;}
                else if j==NONE{pair.reason=2;}
                else{pair=self.second.check_pair(&self.merged[root_start..],&self.nodes,self.words.len(),k,j,row,skip);}
            }else{pair.reason=3;}
            r.reason=pair.reason;r.eligible_ns=crate::now()-t;
            if r.reason==6 && route==3{
                let original=self.words[pair.k as usize];let old_len=self.dict.len();
                self.dict.extend_from_slice(b"item\",\"");
                self.words[pair.k as usize]=Piece{id:old_len as u32,width:7};
                let start=crate::now();
                let d=self.second.index_pair(&self.merged[root_start..],&self.nodes,self.words.len(),m,pair);
                r.index_ns=crate::now()-start;
                r.new_runs=self.second.pair_runs();
                r.stock=self.second.finish_pair(d,pair.row,pair.skip,m-pair.k_mass,n,&self.words,&self.dict,out);
                if r.stock && inspect==2{r.first=Some((self.second.pair_input(&self.words,&self.dict),(pair.row,pair.skip)));}
                if r.stock && inspect>0{r.snapshot=Some(self.second.canonical((pair.row,pair.skip),&self.words));}
                self.words[pair.k as usize]=original;
                self.dict.truncate(old_len);
                if r.stock{
                    r.removed=pair.k_mass;r.moved=pair.moved;r.applied=true;
                    return r;
                }
                r.reason=7;
            }
        }
        r.new_runs=r.old_runs;
        r.stock=self.finish(mark,m,n,p,out);
        if inspect>0 && r.stock{r.snapshot=Some(self.second.canonical(mark.unwrap(),&self.words));}
        r
    }
}
