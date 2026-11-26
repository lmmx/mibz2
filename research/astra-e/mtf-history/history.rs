// The prefix is ordered by descending live-bit age. The tail is ordinary global rank order.
struct History {
    live:u64,
    recent:usize,
    next:usize,
    bytes:[u8;64],
    tail:[u8;256],
}
impl History{
    fn new(alphabet:&[u8;256])->Self{
        Self{live:0,recent:0,next:0,bytes:[0;64],tail:*alphabet}
    }
    #[inline(always)]
    unsafe fn select(&mut self,rank:usize)->u8{
        use std::arch::x86_64::_pdep_u64;
        let wanted=1u64.checked_shl(rank as u32).unwrap_or(0);
        let selected=_pdep_u64(wanted,self.live);
        let byte;
        if selected!=0{
            let age=selected.trailing_zeros() as usize;
            byte=self.bytes[self.next.wrapping_sub(1+age)&63];
            self.live^=selected;
        }else{
            // Invariant: the recent occurrences precede the tail, which ends at slot 255.
            byte=self.tail[rank];
            self.tail.copy_within(self.recent..rank,self.recent+1);
            self.recent+=1;
        }
        if self.live>>63!=0{
            self.recent-=1;
            self.tail[self.recent]=self.bytes[self.next];
        }
        self.live=(self.live<<1)|1;
        self.bytes[self.next]=byte;
        self.next=(self.next+1)&63;
        byte
    }
}
