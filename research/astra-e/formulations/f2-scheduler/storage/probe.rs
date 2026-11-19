pub use mibz2::{bwt,runs,Kernels};
use bwt::{InverseBwt,Segmented};
use mibz2::stream::{Event,Scanner,SliceSource};
#[path="scalar/candidate.rs"] mod scalar;
#[path="fixed/candidate.rs"] mod fixed;
#[path="refill/candidate.rs"] mod refill;
#[path="double/candidate.rs"] mod double;
fn main() {
    let file=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut io=Scanner::new(SliceSource(&file));
    let mut kernels=(scalar::Candidate::default(),fixed::Candidate::default(),refill::Candidate::default(),double::Candidate::default());
    let (mut r,mut reference,mut have,mut want)=(runs::Runs::default(),Segmented::default(),Vec::new(),Vec::new());
    let k=Kernels::detect();let mut count=0;
    println!("stream,rows,arm,admitted,first_vectors,first_lane_paths,second_vectors,second_lane_paths,total_capacity_bytes");
    while let Ok(Event::Block{index,symbols,..})=io.next(Vec::new()) {
        let n=runs::front(&symbols,&mut r,k).unwrap();
        if n<65_536 {continue}
        let p=symbols.orig_ptr;
        assert!(reference.invert_runs(&r,n,p,&mut want,k));
        for arm in 0..4 {
            let (yes,c)=match arm {0=>kernels.0.audit(&r,n,p,&mut have),1=>kernels.1.audit(&r,n,p,&mut have),2=>kernels.2.audit(&r,n,p,&mut have),_=>kernels.3.audit(&r,n,p,&mut have)};
            if yes {assert_eq!(have,want);}
            println!("{index},{n},{},{},{},{},{},{},{}",["scalar","fixed","refill","double"][arm],u8::from(yes),c[0],c[1],c[2],c[3],c.iter().sum::<usize>());
        }
        count+=1;
    }
    assert_eq!(count,233);
}
