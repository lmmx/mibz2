                if e & 15 != 0 {
                    r.skip_short(e & 15);
                    let count=1+((e>>8)&3) as usize;
                    out[n].write(((e>>FIRST)&511) as u16);
                    if count<=2 {
                        out[n+1].write(((e>>20)&511) as u16);
                    } else if MAX_PHRASE==3 || count==3 {
                        out[n+1].write(((e>>20)&63) as u16);
                        out[n+2].write(((e>>26)&63) as u16);
                    } else {
                        out[n+1].write(((e>>20)&15) as u16);
                        out[n+2].write(((e>>24)&15) as u16);
                        out[n+3].write(((e>>28)&15) as u16);
                    }
                    n+=count;
                    left-=count;
                    if e & TERMINAL != 0 {
                        break 'groups Ok(());
                    }
