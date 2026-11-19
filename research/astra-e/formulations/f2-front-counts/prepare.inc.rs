    fn prepare_counted(
        &mut self,
        caps: &[u16],
        n: u32,
        counts: &[u32; 256],
    ) -> Option<(u32, u32)> {
        let tokens = counts[QUOTE as usize];
        if n == 0 || tokens == 0 { return None; }
        let mut total=0;
        let mut next=counts.map(|width| { let base=total; total+=width; base });
        if total!=n {return None;}
        let quote=next[QUOTE as usize];
        let pages=n.div_ceil(64) as usize;
        self.starts.clear();
        self.starts.resize(pages,0);
        self.page_first.resize(pages,0);
        self.runs.clear();
        let mut prev=None;
        let mut row=0;
        for &rec in caps {
            let byte=(rec>>8) as u8;
            let len=u32::from(rec&255);
            if len==0 || len>127 || row+len>n {return None;}
            if Some(byte)!=prev {
                self.runs.push(Run{start:row,lf:next[byte as usize],byte});
                self.starts[(row >> 6) as usize] |= 1u64 << (row & 63);
                prev=Some(byte);
            }
            next[byte as usize]+=len;
            row+=len;
        }
        if row!=n {return None;}
        self.runs.push(Run{start:n,lf:0,byte:0});
        let mut preceding=0u32;
        for (mask, base) in self.starts.iter_mut().zip(&mut self.page_first) {
            *base=preceding+(*mask as u32 & 1)-1;
            preceding += mask.count_ones();
            *mask &= !1;
        }
        Some((quote,tokens))
    }
