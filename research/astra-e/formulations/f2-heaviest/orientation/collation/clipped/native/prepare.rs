        let (lo,hi)=self.section;
        if lo>=hi || hi>n{return None;}
        let mut total=0;
        let mut next=counts.map(|w|{let x=total;total+=w;x});
        if total!=n{return None;}
        // class 0 = entirely outside; 1 = entirely inside; 2 = spans an endpoint.
        let class:[u8;256]=std::array::from_fn(|c|{
            let a=next[c];let b=a+counts[c];
            if b<=lo || a>=hi{0}else if lo<=a && b<=hi{1}else{2}
        });
        let pages=n.div_ceil(64) as usize;
        self.starts.clear();self.starts.resize(pages,0);self.page_first.resize(pages,0);
        self.runs.clear();
        #[cfg(clipped_counts)] {self.interior_splits=0;}
        let mut row=0u32;
        let mut previous=NONE;
        for &rec in caps{
            let byte=(rec>>8) as u8;
            let len=u32::from(rec&255);
            if len==0 || len>127 || len>n-row{return None;}
            let dest=next[byte as usize];
            let end=dest+len;
            let delta=dest.wrapping_sub(row);
            let class=class[byte as usize];
            let mut x=dest;
            loop{
                let (stop,terminal)=match class{
                    0=>(end,false),1=>(end,true),
                    _ if x<lo=>(end.min(lo),false),
                    _ if x<hi=>(end.min(hi),true),
                    _=>(end,false),
                };
                let key=u32::from(byte)|(u32::from(terminal)<<8);
                let final_row=row+stop-x;
                if previous==key{
                    self.runs.last_mut().unwrap().position_byte+=(stop-x)<<8;
                }else{
                    #[cfg(clipped_counts)] {self.interior_splits+=u32::from(previous!=NONE && (previous&255)==u32::from(byte));}
                    self.runs.push(Run::new(final_row,byte,delta,terminal));
                    self.starts[(row>>6) as usize]|=1u64<<(row&63);
                    previous=key;
                }
                row=final_row;
                if stop==end{break;}
                x=stop;
            }
            next[byte as usize]=end;
        }
        if row!=n{return None;}
