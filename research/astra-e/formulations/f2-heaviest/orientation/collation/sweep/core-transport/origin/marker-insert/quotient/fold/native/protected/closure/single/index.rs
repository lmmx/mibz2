impl Second {
    pub(super) fn check_pair(&mut self,root:&[Piece],nodes:&[Node],kinds:usize,k:u32,j:u32,row:u32,skip:u32)->super::Pair{
        let mut pair=super::Pair{k,j,row,skip,..super::Pair::default()};
        Self::reset(&mut self.count,kinds,0);
        let (mut at,mut first,mut last,mut prior)=(0,u32::MAX,0,0);
        for run in root{
            let n=&nodes[run.id as usize];let word=n.word;
            let count=self.count[word as usize];
            let end=at+run.width;
            if word==k{
                first=first.min(at);last=end;pair.k_base=n.base;
            }else if word==j{pair.j_base=n.base;}
            if at<=row && row<end{
                pair.moved=word==k;
                prior=if pair.moved{count+row-at}else{self.count[k as usize]};
            }
            self.count[word as usize]+=run.width;
            at=end;
        }
        pair.k_mass=self.count[k as usize];pair.j_mass=self.count[j as usize];
        if first<pair.j_base || last>pair.j_base+pair.j_mass {pair.reason=4;return pair;}
        let cutoff=pair.k_mass.max(pair.j_mass);
        // A strict third-class witness is sufficient even with tied heaviest IDs.
        if !self.count.iter().any(|&v|v>cutoff){pair.reason=5;return pair;}
        if pair.moved{
            pair.row=pair.k_base+prior-if pair.k_base>pair.j_base{pair.k_mass}else{0};
            pair.skip+=5;
        }else{pair.row=row-prior;}
        pair.reason=6;pair
    }
    #[inline]
    fn add_pair_run(&mut self,word:u32,base:u32,width:u32,at:&mut u32){
        let k=word as usize;
        if self.runs.last().is_none_or(|r|r.word!=word){
            self.runs.push(TokenRun{start:*at,lf:base+self.count[k],word});
        }
        self.base[k]=base;
        self.count[k]+=width;
        *at+=width;
    }
    pub(super) fn index_pair(&mut self,root:&[Piece],nodes:&[Node],kinds:usize,m:u32,p:super::Pair)->u32{
        Self::reset(&mut self.count,kinds,0);Self::reset(&mut self.base,kinds,0);self.runs.clear();
        let mut cursor=0;let mut used=p.j_base;
        while used>=root[cursor].width{used-=root[cursor].width;cursor+=1;}
        let mut at=0;let kbase=p.k_base-if p.k_base>p.j_base{p.k_mass}else{0};
        for run in root{
            let n=&nodes[run.id as usize];
            if n.word==p.k{continue;}
            if n.word!=p.j{
                let base=n.base-if n.base>p.j_base{p.k_mass}else{0};
                self.add_pair_run(n.word,base,run.width,&mut at);continue;
            }
            let mut left=run.width;
            while left>0{
                let source=root[cursor];let w=left.min(source.width-used);
                let (word,base)=if nodes[source.id as usize].word==p.k{(p.k,kbase)}else{(p.j,p.j_base)};
                self.add_pair_run(word,base,w,&mut at);
                used+=w;if used==source.width{cursor+=1;used=0;}
                left-=w;
            }
        }
        assert_eq!(at,m-p.k_mass);
        self.end_pair_index(at,kinds)
    }
    pub(super) fn pair_runs(&self)->u32{(self.runs.len()-1) as u32}
}
impl Second{
    pub(super) fn pair_input(&self,words:&[Piece],dict:&[u8])->crate::Obj{
        let mut weights=Vec::new();
        for (k,&mass) in self.count.iter().enumerate(){
            if mass==0{continue;}
            let w=words[k];
            weights.push((self.base[k],dict[w.id as usize..(w.id+w.width) as usize].to_vec()));
        }
        weights.sort_unstable_by_key(|w|w.0);
        let root=self.runs.windows(2).map(|r|(self.base[r[0].word as usize],r[1].start-r[0].start)).collect();
        (weights,root)
    }
}
