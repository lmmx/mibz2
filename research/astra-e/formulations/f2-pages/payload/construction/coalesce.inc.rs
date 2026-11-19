        let mut row=0u32;
        let mut i=0;
        while i<caps.len() {
            let byte=(caps[i]>>8) as u8;
            let start=row;
            loop {
                let len=u32::from(caps[i]&255);
                if len==0 || len>127 {return None;}
                row+=len;
                i+=1;
                if i==caps.len() || (caps[i]>>8) as u8 != byte {break;}
            }
            if row>n {return None;}
            let delta=next[byte as usize].wrapping_sub(start);
            self.runs.push(Run::new(row,byte,delta));
            self.starts[(start>>6) as usize]|=1u64 << (start&63);
            next[byte as usize]+=row-start;
        }
        if row!=n {return None;}
