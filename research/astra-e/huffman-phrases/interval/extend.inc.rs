        let eob=(alpha-1)as u32;
        let mut w=0u32;
        while w<1024 {
            let first=h.lut[w as usize];
            let mut used=first & 15;
            if used==0 {
                w+=1;
                continue;
            }
            let mut depth=used;
            let mut symbols=[(first>>FIRST)&511,0,0,0];
            let mut count=1usize;
            let mut entry=first;
            while count<4 && symbols[count-1]!=eob && used+h.min_len<=10 {
                if (count==2 && symbols[1]>=64) || (count==3 && (symbols[1]|symbols[2])>=16) {break;}
                let next=h.lut[((w<<used)&1023) as usize];
                let len=(next>>4)&15;
                if len==0 || used+len>10 {
                    depth=10;
                    break;
                }
                let id=(next>>FIRST)&511;
                depth=used+len;
                if (count==2 && id>=64) || (count==3 && id>=16) {break;}
                used=depth;
                symbols[count]=id;
                count+=1;
                let payload=match count {
                    2=>symbols[1]<<20,
                    3=>symbols[1]<<20|symbols[2]<<26,
                    _=>symbols[1]<<20|symbols[2]<<24|symbols[3]<<28,
                };
                entry=first&!15 | ((count as u32-1)<<8) | u32::from(id==eob)*TERMINAL | used | payload;
            }
            let span=1u32<<(10-depth);
            let end=(w|span.wrapping_sub(1))+1;
            h.lut[w as usize..end as usize].fill(entry);
            w=end;
        }
