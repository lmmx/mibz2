    // On a mixed page even replies are twice the index of the containing run.
    // Odd replies carry either the full run response or a uniform page ID.
    #[inline]
    fn reply(&self, x:u32)->u64 {
        let p=(x>>6) as usize;
        let prefix=u64::MAX >> (63-(x&63));
        if PAGE_ID {
            let base=self.page_first[p];
            if base>>31 != 0 { return ((p as u64)<<1)|1; }
            return u64::from(base+(self.starts[p]&prefix).count_ones())<<1;
        }
        let word=self.starts[p];
        if word&1 != 0 { return word; }
        u64::from(self.page_first[p]+(word&prefix).count_ones())<<1
    }

    #[inline]
    fn read_reply(&self, answer:u64)->Run {
        if answer&1 == 0 { return self.runs[(answer>>1) as usize]; }
        let word=if PAGE_ID {self.starts[(answer>>1) as usize]} else {answer};
        Run { position_byte:(word as u32)>>1, delta:(word>>32) as u32 }
    }

    #[inline]
    fn recover(&self, answer:u64, x:u32)->usize {
        if answer&1 == 0 { return (answer>>1) as usize; }
        (self.page_first[(x>>6) as usize] & !(1<<31)) as usize
    }

    #[cfg(test)]
    fn locate(&self, x:u32)->usize {
        let answer=self.reply(x);
        self.recover(answer,x)
    }
