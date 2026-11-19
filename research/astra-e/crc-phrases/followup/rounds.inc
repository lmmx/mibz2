    assert_eq!((accepted,total,pos,all.len()),(228,431,209700196,228));
    let cuts=[512,1024,2048];
    let mut timings=std::io::BufWriter::new(std::fs::File::create(dir.join("times.csv")).unwrap());
    writeln!(timings,"block,round,position,arm,cpu_ns").unwrap();
    let names=["stock","empty","paid_empty","warm512","warm1024","warm2048","paid512","paid1024","paid2048"];
    let plans:Vec<Vec<Vec<Span>>>=all.iter().map(|b|cuts.iter().map(|&c|spans_for(b,c)).collect()).collect();
    let mut precomputed:Vec<Vec<Vec<Digest>>>=vec![vec![vec![];3];all.len()];
    let mut scratch=vec![];
    for (bi,b) in all.iter().enumerate() {
        for i in 0..3 {assert_eq!(checksum(b,&plans[bi][i],&mut precomputed[bi][i],true),b.crc);}
    }
    for round in 0..=36usize {
        for off in 0..all.len() {
            let bi=(if round%2==0 {off} else {all.len()-1-off}+round*37)%all.len();
            let b=&all[bi];
            for position in 0..9 {
                let arm=((if round%2==0 {position} else {8-position})+round+bi)%9;
                let start=clock();
                let got=if arm==0 {mibz2::crc::crc32(black_box(&b.output),k)}
                    else if arm==1 {checksum(black_box(b),black_box(&[]),&mut scratch,false)}
                    else if arm==2 {checksum(black_box(b),black_box(&[]),&mut scratch,true)}
                    else {
                        let i=(arm-3)%3;
                        let dst=if arm<6 {&mut precomputed[bi][i]} else {&mut scratch};
                        checksum(black_box(b),&plans[bi][i],dst,arm>=6)
                    };
                let elapsed=clock()-start;
                assert_eq!(got,b.crc);
                if round>0 {writeln!(timings,"{},{round},{position},{},{elapsed}",b.index,names[arm]).unwrap();}
            }
        }
    }
}
