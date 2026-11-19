#[cfg(census)]
fn finite() {
    let mut stock=control::kernel::Kernel::default();
    let mut changed=batched::kernel::Kernel::default();
    let mut raw=Vec::new();
    let mut tests=0usize;
    let mut check=|column:&[u8],p:usize,limit:u32|{
        let mut ids:Vec<_>=(0..column.len()).collect();
        ids.sort_by_key(|&x|column[x]);
        let mut row=p;
        let want:Vec<u8>=(0..column.len()).map(|_|{row=ids[row];column[row]}).collect();
        let mut records=Vec::<u16>::new();
        for &v in column {
            if let Some(last)=records.last_mut() {
                if *last>>8==u16::from(v) && (*last&255)<127 {*last+=1;continue;}
            }
            records.push((u16::from(v)<<8)|1);
        }
        let n=column.len();
        stock.run::<true,true,false>(&records,n,p as u32,limit,&mut raw);
        assert_eq!(raw,want);
        changed.run::<true,true,false>(&records,n,p as u32,limit,&mut raw);
        assert_eq!(raw,want);
        assert_eq!(stock.bench_state(&records,n,p as u32,limit),
                   changed.bench_state(&records,n,p as u32,limit));
        tests+=1;
    };
    for n in 1..=9 {
        for mask in 0..1u32<<n {
            let col:Vec<_>=(0..n).map(|i|((mask>>i)&1)as u8).collect();
            for p in 0..n {check(&col,p,1);check(&col,p,16);}
        }
    }
    for n in 1..=6 {
        for mut num in 0..3usize.pow(n as u32) {
            let col:Vec<_>=(0..n).map(|_|{let v=num%3;num/=3;[0,128,255][v]}).collect();
            for p in 0..n {check(&col,p,1);check(&col,p,16);}
        }
    }
    for seed in 0..40u32 {
        let mut z=seed+1;let mut col=Vec::new();
        for j in 0..96 {z=z.wrapping_mul(1664525).wrapping_add(1013904223);
            let w=[1,2,4,15,16,17,61,126,127,128,255][j%11];
            col.extend(std::iter::repeat_n(((z>>24)%11)as u8,w));}
        for p in [0,1,15,16,126,127,col.len()/2,col.len()-1] {check(&col,p,16);}
    }
    for h in [21844,21845,21846] {
        let mut col=vec![2];for i in 0..h{col.extend(std::iter::repeat_n((i&1)as u8,17));}
        check(&col,0,16);
    }
    println!("{tests} finite marked/limit cases, both complete states and independently sorted marked outputs matched");
}
