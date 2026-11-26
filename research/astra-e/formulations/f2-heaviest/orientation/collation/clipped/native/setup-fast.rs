        let (lo,hi)=self.section;
        if lo>=hi || hi>n{return None;}
        let mut total=0;
        let mut next=counts.map(|w|{let x=total;total+=w;x});
        if total!=n{return None;}
        let pages=n.div_ceil(64) as usize;
        self.starts.clear();self.starts.resize(pages,0);self.page_first.resize(pages,0);
        self.runs.clear();
        #[cfg(clipped_counts)] {self.interior_splits=0;}
        let mut row=0u32;
        if ONLY && counts[self.symbol as usize]==hi-lo {
            // U is the whole F byte bucket. No cap or run can have a partial image.
            let mut previous=None;
            for &rec in caps{
                let byte=(rec>>8) as u8;
                let len=u32::from(rec&255);
                if len==0 || len>127 || len>n-row{return None;}
                let end=row+len;
                if previous==Some(byte){self.runs.last_mut().unwrap().position_byte+=len<<8;}
                else {
                    self.runs.push(Run::new(end,byte,next[byte as usize].wrapping_sub(row),byte==self.symbol));
                    self.starts[(row>>6) as usize]|=1u64<<(row&63);previous=Some(byte);
                }
                row=end;next[byte as usize]+=len;
            }
        } else {
            let mut input=caps.iter().copied();
            let first=input.next()?;
            let mut byte=(first>>8) as u8;
            let mut pending=u32::from(first&255);
            if pending==0 || pending>127 || pending>n{return None;}
            for rec in input{
                let c=(rec>>8) as u8;
                let w=u32::from(rec&255);
                if w==0 || w>127 || w>n.saturating_sub(row+pending){return None;}
                if c==byte{pending+=w;}
                else{
                    self.flush_interval_run(row,pending,byte,&mut next);
                    row+=pending;byte=c;pending=w;
                }
            }
            self.flush_interval_run(row,pending,byte,&mut next);row+=pending;
        }
        if row!=n{return None;}
