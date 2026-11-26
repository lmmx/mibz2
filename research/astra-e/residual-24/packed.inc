
impl Kernel {
    fn build_packed<const P:usize>(&mut self,m:usize,bits:u32,cursors:&[usize;256])->bool {
        debug_assert!(P.is_power_of_two());
        const MIXED:u16=u16::MAX;
        const CLEAR:u16=u16::MAX-1;
        let low=24-bits;
        let largest=255+self.offsets.len().saturating_sub(1);
        if largest>>(low as usize)>255 {return false}
        self.headers.resize(m.div_ceil(P),CLEAR);
        self.headers.fill(CLEAR);
        let mut k=0;
        for b in &self.bands {
            let span=(b.end-k) as usize;
            let dst=k.wrapping_add(b.delta) as usize;
            let high=(b.code>>low) as u16;
            for h in &mut self.headers[dst/P..=(dst+span-1)/P] {
                if *h==CLEAR {*h=high;} else if *h!=high {*h=MIXED;}
            }
            k=b.end;
        }
        let mut mixed=0usize;
        for h in &mut self.headers {
            if *h==MIXED {
                if mixed>=32768 {return false}
                *h=0x8000 | mixed as u16;
                mixed+=1;
            } else {debug_assert!(*h<=255);}
        }
        self.pack.resize(m+1,[0;3]);
        self.high.resize(mixed*P,0);
        let code_mask=(1<<low)-1;
        let mut k=0;
        for a in 0..256 {
            let end=cursors.get(a+1).copied().unwrap_or(self.bands.len());
            let mut from=0;
            for b in &self.bands[cursors[a]..end] {
                let span=(b.end-k) as usize;
                let dst=k.wrapping_add(b.delta) as usize;
                let pack=(b.code&code_mask)<<bits;
                let top=(b.code>>low) as u8;
                let src=&self.queues[a][from..from+span];
                for (slot,&j) in self.pack[dst..dst+span].iter_mut().zip(src) {
                    let [x,y,z,_]=(pack|j).to_le_bytes();
                    *slot=[x,y,z];
                }
                let limit=dst+span;
                let mut next=dst;
                while next<limit {
                    let h=self.headers[next/P];
                    let chunk=(P-next%P).min(limit-next);
                    if h&0x8000!=0 {
                        let offset=(h as usize & 0x7fff)*P+next%P;
                        self.high[offset..offset+chunk].fill(top);
                    }
                    next+=chunk;
                }
                from+=span;
                k=b.end;
            }
        }
        debug_assert_eq!(k as usize,m);
        true
    }

    #[inline(always)]
    fn load24(&self,row:u32)->u32 {
        let i=row as usize;
        assert!(i+1<self.pack.len());
        // SAFETY: every live row has three initialized bytes and the padding
        // record provides a fourth byte even at the final actual row.
        let p=self.pack.as_ptr().cast::<u8>();
        unsafe {u32::from_le(p.add(3*i).cast::<u32>().read_unaligned()) & 0x00ff_ffff}
    }
    #[inline(always)]
    fn label24<const P:usize>(&self,row:u32,edge:u32,bits:u32)->u32 {
        let i=row as usize;
        let header=self.headers[i/P];
        let high=if header&0x8000==0 {u32::from(header)} else {
            u32::from(self.high[(usize::from(header)&0x7fff)*P+i%P])
        };
        (high<<(24-bits))|(edge>>bits)
    }
    fn emit_packed<const P:usize>(&self,n:usize,mark:u32,bits:u32,out:&mut Vec<u8>) {
        out.clear();out.reserve(n);
        let mask=(1<<bits)-1;
        let mut row=mark;
        let mut edge=self.load24(row);
        loop {
            let current=row;
            row=edge & mask;
            // The word metadata and next link are independent after row is known.
            let next=self.load24(row);
            let code=self.label24::<P>(current,edge,bits);
            append_code(code,&self.spelling,&self.offsets,out);
            edge=next;
            if row==mark {break}
        }
        repeat(n,out);
    }
    pub fn packed_run<const P:usize>(&mut self, records:&[u16], n:usize, p:u32, out:&mut Vec<u8>) {
        let (_,mark,bits)=self.construct::<true,true,P>(records,n as u32,p,16);
        if self.packed {
            self.emit_packed::<P>(n,mark,bits,out)
        } else {
            emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,out);
        }
    }
}
