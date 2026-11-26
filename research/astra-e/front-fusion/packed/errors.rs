fn disputes(file:&str){
    let k=Kernels::detect();
    let mut arms:[Arm;3]=std::array::from_fn(|_|Arm::default());
    let(mut checks,mut rollback,mut direct,mut bytes)=(0,0,0,0);
    let mut check=|b:&mut Symbols,supported:Kernels|{
        let gold=b.syms.clone();
        let (wanted,_,_)=arms[0].decode(b,supported,0);
        let expected=arms[0].out.clone();
        for (i,kind) in [(1,2),(2,3)]{
            b.syms.copy_from_slice(&gold);
            let (got,words,done)=arms[i].decode(b,supported,kind);
            assert_eq!(got,wanted);assert_eq!(arms[i].out,expected);
            if !done{assert_eq!(gold,b.syms);}
            if kind==2{rollback+=usize::from(!done&&words>0);direct+=usize::from(done);if !done{bytes+=words*2;}}
            checks+=1;
        }
        b.syms=gold;
    };
    let data=std::fs::read(file).unwrap();let mut scan=Scanner::new(SliceSource(&data));
    while let Ok(Event::Block{index,symbols:mut b,..})=scan.next(Vec::new()){
        if ![1,41,165,278].contains(&index){continue;}
        let original=b.syms.clone();let original_p=b.orig_ptr;let n=b.max_len;let original_crc=b.crc;
        for mutation in 0..12 {
            b.syms.clone_from(&original);b.orig_ptr=original_p;b.max_len=n;b.crc=original_crc;
            let end=b.syms.len()-1;let alphabet=b.seq_to_unseq;
            match mutation{
                0=>b.orig_ptr=u32::MAX,1=>b.crc^=617,
                2=>b.max_len-=16533,3=>b.syms[end-4]=b.eob+1,
                4=>{b.syms.splice(end-2..end,[0;22]);}
                5=>{b.syms.splice(end-2..end,[1;21]);}
                6=>{b.syms.pop();}7=>b.seq_to_unseq[1]=b.seq_to_unseq[0],
                8=>{b.syms.extend([0,1,b.eob+8]);}
                9=>{b.syms[end-10]=b.eob;}
                10=>{b.syms.splice(0..0,[1;22]);}
                _=>{}
            }
            check(&mut b,k);check(&mut b,Kernels::scalar());
            b.seq_to_unseq=alphabet;
        }
    }
    for (width,heavy,count) in [(4u32,1usize,262144u32),(2,12000,131072)] {
        for delta in [-1i32,0,1] {
            let target=(count as i32+delta) as u32;let mut widths=vec![17;heavy];
            widths.extend(std::iter::repeat_n(width,(target/width)as usize));
            if target%width!=0{widths.push(target%width);}
            let mut syms=Vec::new();
            for (i,&w) in widths.iter().enumerate(){
                syms.push(2+(i%3)as u16);let v=w;for k in 0..v.ilog2(){syms.push(((v>>k)&1)as u16);}
            }
            syms.push(5);
            for p in [0,1,16,17,widths.iter().sum::<u32>()-1]{
                let mut b=Symbols{crc:0,orig_ptr:p,seq_to_unseq:std::array::from_fn(|i|i as u8),syms:syms.clone(),eob:5,max_len:900000};
                check(&mut b,k);
            }
        }
    }
    assert!(rollback>=17 && direct>=13);
    println!("{checks} complete-worker return and output comparisons; {rollback} mutated prefixes rolled back ({bytes} packed bytes); {direct} directly completed before CRC or RLE verdict");
}
