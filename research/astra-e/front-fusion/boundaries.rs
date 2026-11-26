fn boundaries() {
    let mut cases=0;let mut fused=0;let mut overwide=0;
    let mut arms:[Arm;3]=std::array::from_fn(|_|Arm::default());
    let mut families=vec![vec![6u32,7,7].repeat(108),vec![6,7,7].repeat(108),
        vec![1,16,17,126,127,128,254,255,2048,1,3,8].repeat(22),
        vec![17;22_000]];
    families[1][0]+=1;
    for (fi,widths) in families.iter().enumerate(){
        let n:u32=widths.iter().sum();let mut syms=Vec::new();
        for (j,&w) in widths.iter().enumerate(){
            syms.push(2+(j%3)as u16);
            let mut extra=w-1;
            while extra>0{extra-=1;syms.push((extra&1)as u16);extra>>=1;}
        }
        syms.push(5);
        let mut cuts=vec![0,n-1,n/2];let mut pos=0;
        for &w in widths.iter().take(12){cuts.extend([pos,pos+w/2,pos+w-1]);pos+=w;}
        cuts.sort_unstable();cuts.dedup();
        for p in cuts{
            let b=Symbols{crc:0,syms:syms.clone(),seq_to_unseq:std::array::from_fn(|i|i as u8),eob:5,max_len:900000,orig_ptr:p};
            let k=Kernels::detect();let mut accepted=None;
            for (i,a) in arms.iter_mut().enumerate(){
                let result=if i==2{a.block::<64>(&b,k,true)}else{a.block::<0>(&b,k,i==1)};
                // Even an unfinished final RLE1 group must leave the exact same inverse prefix.
                assert!(result.is_ok()||matches!(result,Err(ErrorKind::RunUnfinished)));
                if i>0&&result.is_ok(){let direct=result.unwrap().2;assert_eq!(direct,fi!=0);accepted=Some(direct);}
            }
            assert_eq!(arms[0].raw,arms[1].raw);assert_eq!(arms[0].raw,arms[2].raw);
            assert_eq!(arms[0].decoded,arms[1].decoded);assert_eq!(arms[0].decoded,arms[2].decoded);
            cases+=1;fused+=usize::from(accepted==Some(true));overwide+=usize::from(fi==3);
        }
    }
    println!("{cases} exact maximal-width/wrap/density cases, {fused} valid fused completions, {overwide} marked wide-endpoint cases");
}
