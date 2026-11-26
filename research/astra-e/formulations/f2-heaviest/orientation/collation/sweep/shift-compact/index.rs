    fn projected_index(&mut self,caps:&[u16],freq:&[u32;256],n:u32){
        let f=&self.plan.forbid;let target=self.plan.target();let mut total=0;
        let mut next=freq.map(|w|{let a=total;total+=w;a});assert_eq!(total,n);
        let mut fi=next.map(|x|f.after(x));
        let mut ti=next.map(|x|target.after(x));
        let kept=n-f.mass;let pages=kept.div_ceil(64) as usize;
        self.starts.clear();self.starts.resize(pages,0);self.page_first.resize(pages,0);
        self.runs.clear();
        let(mut left,mut row,mut src_idx)=(0u32,0u32,0usize);
        for &record in caps{
            let c=(record>>8) as usize;let width=u32::from(record&255);let end=left+width;
            assert!(width!=0 && width<=127 && end<=n);
            let mut y=next[c];next[c]+=width;
            while left<end{
                while src_idx<f.parts.len() && f.parts[src_idx].hi<=left{src_idx+=1;}
                let mut w=end-left;
                if let Some(s)=f.parts.get(src_idx){
                    if s.lo<=left{w=w.min(s.hi-left);left+=w;y+=w;continue;}
                    w=w.min(s.lo-left);
                }
                let v=&mut ti[c];
                while *v<target.parts.len() && target.parts[*v].hi<=y{*v+=1;}
                let dest=target.parts.get(*v);
                let terminal=dest.is_some_and(|s|s.lo<=y);
                let mapped=if terminal{
                    let s=dest.unwrap();w=w.min(s.hi-y);s.base+y-s.lo
                }else{
                    if let Some(s)=dest{w=w.min(s.lo-y);}
                    let k=&mut fi[c];
                    while *k<f.parts.len() && f.parts[*k].hi<=y{*k+=1;}
                    let removed=f.parts.get(*k).map_or(f.mass,|s|{
                        debug_assert!(s.lo>=y+w);s.base
                    });
                    y-removed
                };
                let delta=mapped.wrapping_sub(row);
                if self.runs.last().is_some_and(|r|r.byte()==c as u8 && r.delta==delta && r.terminal()==terminal){
                    self.runs.last_mut().unwrap().position_byte+=w<<8;
                }else{
                    self.starts[(row>>6) as usize]|=1u64<<(row&63);
                    self.runs.push(Run::new(row+w,c as u8,delta,terminal));
                }
                left+=w;row+=w;y+=w;
            }
        }
        assert_eq!(left,n);assert_eq!(row,kept);
        let mut preceding=0u32;
        for (mask,base) in self.starts.iter_mut().zip(&mut self.page_first){
            *base=preceding+(*mask as u32&1)-1;preceding+=mask.count_ones();*mask&=!1;
        }
    }
    fn mark_shifted(&self,mut p:u32)->(Option<(u32,u32)>,u32){
        let mut skip=0;
        loop{
            if let Some(t)=self.plan.target().rank(p){return (Some((t,skip)),0);}
            if let Some((j,t))=self.plan.levels.iter().enumerate().skip(1).find_map(|(j,l)|l.rank(p).map(|t|(j,t))){
                p=self.plan.levels[j-1].row(t);skip+=1;continue;
            }
            break;
        }
        let start=self.plan.forbid.complement(p);
        let mut x=start;
        loop{
            let r=self.runs[self.locate(x)];x=r.map(x);skip+=1;
            if r.terminal(){return (Some((x,skip)),0);}
            if x==start{return (None,start);}
        }
    }
    fn splice_early(&mut self,m:u32){
        if self.plan.early.is_empty(){return;}
        let start=self.merged.len();let mut pos=0;let mut src=self.nodes[0].out as usize;let mut used=0;
        for k in 0..=self.plan.early.len(){
            let stop=self.plan.early.get(k).map_or(m,|e|e.lo);
            while pos<stop{
                let part=self.merged[src];let take=(stop-pos).min(part.width-used);
                Self::emit(&mut self.merged,start,part.id,take);used+=take;pos+=take;
                if used==part.width{used=0;src+=1;}
            }
            if let Some(e)=self.plan.early.get(k){
                Self::emit(&mut self.merged,start,e.word,e.hi-e.lo);pos=e.hi;
            }
        }
        self.nodes[0].out=start as u32;
    }
