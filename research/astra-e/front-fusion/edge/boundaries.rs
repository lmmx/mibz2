fn edge_boundaries() {
    let mut comparisons=0;let mut admitted=0;let mut rejected=0;let mut arms:[Arm;3]=std::array::from_fn(|_|Arm::default());
    for (width,heavy,count) in [(4u32,1usize,262144u32),(2,12000,131072)]{
        for delta in [-1i32,0,1]{
            let target=(count as i32+delta)as u32;let mut widths=vec![17u32;heavy];
            widths.extend(std::iter::repeat_n(width,(target/width)as usize));
            if target%width!=0{widths.push(target%width);}
            let mut syms=Vec::new();
            for (i,&w) in widths.iter().enumerate(){
                syms.push(2+(i%3)as u16);let mut extra=w-1;
                while extra>0{extra-=1;syms.push((extra&1)as u16);extra>>=1;}
            }
            syms.push(5);let n:u32=widths.iter().sum();
            for p in [0,1,16,17,n-1]{
                let s=Symbols{crc:0,orig_ptr:p,seq_to_unseq:std::array::from_fn(|i|i as u8),max_len:900000,eob:5,syms:syms.clone()};
                assert!(runs::takes_runs(&s));let mut reference=runs::Runs::default();let k=Kernels::detect();
                assert_eq!(runs::front(&s,&mut reference,k).unwrap(),n as usize);
                let dense=reference.records().len()*20>=3*n as usize;assert!(dense);
                let eligible=fits(reference.records(),p);
                for (i,a) in arms.iter_mut().enumerate(){
                    let z=if i==2{a.block::<64>(&s,k,true)}else{a.block::<0>(&s,k,i==1)};
                    if let Ok((length,_,fused))=z{assert_eq!(length,n as usize);if i>0 {assert_eq!(fused,eligible);}}
                    else{assert_eq!(z.unwrap_err(),ErrorKind::RunUnfinished);}
                }
                assert_eq!(arms[0].raw,arms[1].raw);assert_eq!(arms[0].raw,arms[2].raw);
                assert_eq!(arms[0].decoded,arms[1].decoded);assert_eq!(arms[0].decoded,arms[2].decoded);
                comparisons+=1;if eligible {admitted+=1}else{rejected+=1;}
            }
        }
    }
    println!("{comparisons} dense 1-MiB edge-cap crossings agreed ({admitted} eligible; {rejected} delegated), including 4/8-byte width and marked strip boundaries");
}
