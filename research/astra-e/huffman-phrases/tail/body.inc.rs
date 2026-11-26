        while left > 0 {
            let entry=t.lut[r.peek(data,LUT_BITS) as usize];
            if entry & 15==0 {
                let (value,after)=t.decode_entry_slow(entry,r,data);
                r=after;
                let value=match value { Ok(s)=>s,Err(k)=>break 'groups Err(k) };
                out[n].write(value);
                n+=1;
                left-=1;
                if value==eob {break 'groups Ok(());}
                continue;
            }
            let count=1+((entry>>8)&3)as usize;
            out[n].write(((entry>>FIRST)&511)as u16);
            if count>left {
                r.skip_short((entry>>4)&15);
                n+=1;
                left-=1;
                continue;
            }
            r.skip_short(entry&15);
            // left <= 3. No speculative fourth store near MAX_SYMS.
            if count==2 {
                out[n+1].write(((entry>>20)&511)as u16);
            } else if count==3 {
                out[n+1].write(((entry>>20)&63)as u16);
                out[n+2].write((entry>>26)as u16);
            }
            n+=count;
            left-=count;
            if entry & TERMINAL !=0 {break 'groups Ok(());}
        }
