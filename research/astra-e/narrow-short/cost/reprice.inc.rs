    let mut f=std::io::BufWriter::new(std::fs::File::create(dir.join("paired.csv")).unwrap());
    writeln!(f,"round,block,position,arm,scan_ns,total_ns").unwrap();
    for r in 0..=32{
        for (i,b) in blocks.iter().enumerate(){
            for j in 0..4{
                let arm=(r+i+j)%4;
                let t=cpu();
                match arm {
                    0=>{original.run::<true,true,false>(b.r.records(),b.n,b.p,16,&mut out);},
                    1=>{queue.run::<true,true,false>(b.r.records(),b.n,b.p,16,&mut out);},
                    2=>{cut(0);metered.run::<true,true,false>(b.r.records(),b.n,b.p,16,&mut out);cut(8);},
                    3=>{cut(0);all_meter.run::<true,true,false>(b.r.records(),b.n,b.p,16,&mut out);cut(8);},
                    _=>unreachable!()
                }
                let total=cpu()-t;
                let scan=if arm<2 {0} else {STAMPS.with(|st|{let q=st.borrow();q[1]-q[0]})};
                assert!(out==b.reference);
                if r>0{writeln!(f,"{r},{},{j},{},{scan},{total}",b.idx,["original","queues","meter_original","meter_queues"][arm]).unwrap();}
            }
        }
    }
}
