        let mut row=0u32;
        let mut previous=None;
        for &rec in caps {
            let byte=(rec>>8) as u8;
            let len=u32::from(rec&255);
            let end=row+len;
            if len==0 || len>127 || end>n {return None;}
            if previous==Some(byte) {
                // A cap boundary inside the same run extends only the exclusive end.
                self.runs.last_mut().unwrap().position_byte += len << 8;
            } else {
                let delta=next[byte as usize].wrapping_sub(row);
                self.runs.push(Run::new(end,byte,delta));
                self.starts[(row>>6) as usize]|=1u64 << (row&63);
                previous=Some(byte);
            }
            next[byte as usize]+=len;
            row=end;
        }
        if row!=n {return None;}
