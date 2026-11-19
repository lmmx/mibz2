#[test]
#[ignore]
fn observed_waves() {
    use mibz2::stream::{Event,Scanner,SliceSource};
    use std::io::Write;
    let data=std::fs::read(std::env::var("F2_CORPUS").unwrap()).unwrap();
    let mut f=std::io::BufWriter::new(std::fs::File::create(std::env::var("F2_WAVES").unwrap()).unwrap());
    let mut scan=Scanner::new(SliceSource(&data));
    let (mut kernel,mut r,mut reference)=(Candidate::default(),Runs::default(),Segmented::default());
    let (mut want,mut out)=(Vec::new(),Vec::new());
    let k=Kernels::detect();
    writeln!(f,"stream,rows,lookups,waves,one,two,three,four,five,six,seven,eight").unwrap();
    let (mut accepted,mut checked)=(0,0);
    while let Ok(Event::Block{index,symbols,..})=scan.next(Vec::new()) {
        let n=crate::runs::front(&symbols,&mut r,k).unwrap();
        if n<65536 || !kernel.attempt::<false>(r.records(),n,symbols.orig_ptr,&mut out){continue}
        assert!(reference.invert_runs(&r,n,symbols.orig_ptr,&mut want,k));
        assert_eq!(want,out);
        assert!(kernel.second.events[3]>0);
        let h=kernel.histogram;
        let lookups:u64=h.iter().enumerate().map(|(i,&w)|i as u64*w).sum();
        let waves:u64=h.iter().sum();
        writeln!(f,"{index},{n},{lookups},{waves},{},{},{},{},{},{},{},{}",h[1],h[2],h[3],h[4],h[5],h[6],h[7],h[8]).unwrap();
        accepted+=1;checked+=n;
    }
    println!("{accepted}/233 accepted, {checked} inverse bytes matched");
    assert_eq!((accepted,checked),(228,205197794));
}
