    #[inline(always)]
    fn flush_interval_run(&mut self,row:u32,w:u32,byte:u8,next:&mut[u32;256]){
        let a=next[byte as usize];let b=a+w;
        next[byte as usize]=b;
        let (lo,hi)=self.section;
        let delta=a.wrapping_sub(row);
        let mut start=row;
        let stop=row+w;
        let left=lo.wrapping_sub(a);
        let right=hi.wrapping_sub(a);
        if left>0 && left<w{
            let cut=row+left;
            self.runs.push(Run::new(cut,byte,delta,false));
            self.starts[(start>>6) as usize]|=1u64<<(start&63);
            start=cut;
            #[cfg(clipped_counts)] {self.interior_splits+=1;}
        }
        if right>0 && right<w{
            let cut=row+right;
            self.runs.push(Run::new(cut,byte,delta,true));
            self.starts[(start>>6) as usize]|=1u64<<(start&63);
            start=cut;
            #[cfg(clipped_counts)] {self.interior_splits+=1;}
        }
        let terminal=b>lo && b<=hi;
        self.runs.push(Run::new(stop,byte,delta,terminal));
        self.starts[(start>>6) as usize]|=1u64<<(start&63);
    }
