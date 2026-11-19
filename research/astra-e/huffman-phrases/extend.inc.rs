        // Each entry, including an already extended entry, retains its full first ID and length.
        let eob = (alpha-1) as u32;
        for w in 0..1024u32 {
            let first=h.lut[w as usize];
            if first & 15 == 0 || first & TERMINAL != 0 {continue;}
            let l1=first & 15;
            let next = |used:u32| {
                let e=h.lut[((w<<used)&1023) as usize];
                let len=e>>4&15;
                (e>>FIRST&511,len,len!=0 && used+len<=10)
            };
            let (b,l2,ok)=next(l1);
            if !ok {continue;}
            let l12=l1+l2;
            let base=first & !15;
            let mut phrase=base | b<<20 | 1<<8 | u32::from(b==eob)*TERMINAL | l12;
            if b!=eob && b<64 && l12<10 {
                let (c,l3,ok)=next(l12);
                if ok && c<64 {
                    let l123=l12+l3;
                    phrase=base | b<<20 | c<<26 | 2<<8 | u32::from(c==eob)*TERMINAL | l123;
                    if MAX_PHRASE==4 && (b|c)<16 && c!=eob && l123<10 {
                        let (d,l4,ok)=next(l123);
                        if ok && d<16 {
                            phrase=base | b<<20 | c<<24 | d<<28 | 3<<8 | u32::from(d==eob)*TERMINAL | (l123+l4);
                        }
                    }
                }
            }
            h.lut[w as usize]=phrase;
        }
