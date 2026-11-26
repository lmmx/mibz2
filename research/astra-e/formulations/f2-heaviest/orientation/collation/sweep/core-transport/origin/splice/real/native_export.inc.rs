fn main() {
    use std::io::Write;
    let args:Vec<_>=std::env::args().collect();
    let mut input=BufReader::new(std::fs::File::open(&args[1]).unwrap());
    let dir=std::path::Path::new(&args[2]);
    let mut calls=0;
    while let Some(x)=Input::read(&mut input) {
        let oracle=Truth::new(&x);
        let secs=oracle.sections(&x);
        let expected=oracle.output(x.p);
        for (arm,policy,name,h) in [(4,0,"frequent",8),(3,1,"quote",12)] {
            let mut sub=Subject::default();
            let r=sub.run(&x,arm,true);
            oracle.receipt(&x,secs[arm],&r);
            assert_eq!(sub.out,expected);
            let (mw,rs)=r.obj.unwrap();
            let (mark,skip)=r.mark.unwrap_or((u32::MAX,0));
            let file=std::fs::File::create(dir.join(format!("{:04}-{name}-h{h}.native",x.index))).unwrap();
            let mut f=std::io::BufWriter::new(file);
            f.write_all(b"JNATv1\0\0").unwrap();
            for v in [x.index,policy,h,x.n,r.sec.0,r.sec.1,mark,skip,mw.len() as u32,rs.len() as u32] {
                f.write_all(&v.to_le_bytes()).unwrap();
            }
            for (base,w) in mw {
                f.write_all(&base.to_le_bytes()).unwrap();
                f.write_all(&(w.len() as u32).to_le_bytes()).unwrap();
                f.write_all(&w).unwrap();
            }
            for (base,width) in rs {
                f.write_all(&base.to_le_bytes()).unwrap();f.write_all(&width.to_le_bytes()).unwrap();
            }
            f.write_all(&sub.out).unwrap();f.flush().unwrap();calls+=1;
        }
    }
    assert_eq!(calls,20);
    println!("{calls} first-object snapshots. All dense row/word/mark and complete marked-output checks in the frozen native generator passed.");
}
