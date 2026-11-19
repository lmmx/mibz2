fn adversarial(){
    let identity=std::array::from_fn(|i|i as u8);
    let cases=[
        vec![0;35], vec![1;25], vec![0,0,2,1,0,0,3,4], vec![2,2,3,3,4],
        vec![1,1,2,2,1,5,0,0,3,5,0,1], vec![4], vec![0,1,257,256,255,4]
    ];
    let mut arms:[Arm;2]=std::array::from_fn(|_|Arm::default());let mut num=0;
    let mut errors=std::collections::BTreeMap::new();let mut successful=0;
    for case in 0..350 {
        let mut state=0x1200041u64+case as u64;
        let len=128+case%500;
        let mut syms=if case<cases.len(){cases[case].clone()}else{
            (0..len).map(|_|{state=state.wrapping_mul(6364136223846793005).wrapping_add(1);match (state>>60)%9 {0..=2=>0,3..=5=>1,6=>2,7=>3,_=>4}}).collect::<Vec<u16>>()
        };
        if case%6!=0{syms.push(5);}
        if case%7==0{syms.extend_from_slice(&[0,0,1,5]);}
        if case%19==0{syms.splice(0..0,std::iter::repeat_n(0,22));}
        for table in [identity,std::array::from_fn(|i|if i<3{0}else{i as u8})]{
          for ptr in [0,1,4,126,127,128,899999,u32::MAX]{
            let b=Symbols{syms:syms.clone(),crc:0,orig_ptr:ptr,seq_to_unseq:table,eob:5,max_len:if case%4==0{300}else{900_000}};
            for k in [Kernels::detect(),Kernels::scalar()]{
                let a=arms[0].block::<0>(&b,k,false);let z=arms[1].block::<64>(&b,k,true);
                let a0=a.map(|(n,c,_)|(n,c));let z0=z.map(|(n,c,_)|(n,c));
                assert_eq!(a0,z0);if a0.is_ok(){assert_eq!(arms[0].raw,arms[1].raw);assert_eq!(arms[0].decoded,arms[1].decoded);successful+=1;}
                else {*errors.entry(format!("{:?}",a0.unwrap_err())).or_insert(0)+=1;}
                num+=1;
            }
          }
        }
    }
    println!("{num} exact scalar/detected comparisons; {successful} valid paths; error counts {errors:?}");
}
