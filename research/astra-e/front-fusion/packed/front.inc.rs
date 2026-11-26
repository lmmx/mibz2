    let mut row_count=0usize;
    let mut cap_difference=0usize;
    let litp=f.lit.as_ptr();
    let onep=f.one.as_ptr();

    // Each primitive write is within the input-size allocation by the per-span
    // contraction theorem. Only the already consumed source prefix can be overwritten.
    macro_rules! long {
        ($c:expr,$total:expr,$boundary:expr) => {{
            let (c,total):(u64,u64)=($c,$total);
            if total!=0 {
                if total+row_count as u64>max_len as u64{return None;}
                let start=*written;
                let (tag,words)=if total<=254{(total as u16,1)}
                    else if total<=65535{(0,2)}else{(255,3)};
                dst.add(start).write((c as u16)<<8|tag);
                if words>=2{dst.add(start+1).write(total as u16);}
                if words==3{dst.add(start+2).write((total>>16)as u16);}
                *written+=words;
                cap_difference+=total.div_ceil(127)as usize-words;
                row_count+=total as usize;
            }
        }};
    }
    macro_rules! emit {
        ($c:expr,$total:expr,$span:expr,$boundary:expr) => {{
            let (c,total):(u64,u64)=($c,$total);
            if total>127 {
                if $span>22{return None;}
                long!(c,total,$boundary);
            } else {
                dst.add(*written).write((c as u16)<<8|total as u16);
                *written+=1;row_count+=total as usize;
            }
        }};
    }
