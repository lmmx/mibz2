pub use mibz2::{bwt,runs,Kernels};
use mibz2::stream::{Scanner,SliceSource,Event};
use bwt::{InverseBwt,Segmented};
fn main() {
    let compressed=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut s=Scanner::new(SliceSource(&compressed));
    let (mut r,mut reference,mut kernel)=(runs::Runs::default(),Segmented::default(),kernel::Kernel::default());
    let (mut out,mut want)=(vec![],vec![]);let k=Kernels::detect();
    let(mut accepted,mut rows)=(0,0);
    println!("block,rows,residual,bits,page,cycle_visits,uniform_pages,mixed_pages,mixed_rows,mixed_visits,bands,over_u8,band_page_touches,old_bytes,packed_bytes");
    while let Event::Block{index,symbols,..}=s.next(Vec::new()).unwrap(){
        let n=runs::front(&symbols,&mut r,k).unwrap();
        if !runs::takes_runs(&symbols) || n<65536 || r.records().len()*20>=3*n {continue}
        assert!(reference.invert_runs(&r,n,symbols.orig_ptr,&mut want,k));
        kernel.inspect(r.records(),n,symbols.orig_ptr,index,&mut out);
        assert_eq!(out,want);
        accepted+=1;rows+=n;
    }
    assert_eq!((accepted,rows),(228,205197794));
}
