use mibz2::{Kernels,runs,stream::{Event,Scanner,SliceSource}};
fn main(){
    let data=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();let mut sc=Scanner::new(SliceSource(&data));let mut records=runs::Runs::default();
    println!("stream,syms,n,caps,true_runs,queue_rows,thick_pieces,capped_excess,early_literal_bound,reject,small");
    while let Ok(Event::Block{index,symbols,..})=sc.next(Vec::new()){
        if !runs::takes_runs(&symbols){continue;}
        let n=runs::front(&symbols,&mut records,Kernels::detect()).unwrap();let caps=records.records();
        let (mut i,mut pos,mut count,mut thin,mut thick)=(0,0,0,0,0);
        while i<caps.len(){
            let byte=caps[i]>>8;let mut width=0;while i<caps.len()&&caps[i]>>8==byte{width+=u32::from(caps[i]&255);i+=1;}
            let p=symbols.orig_ptr;
            if width<=16{thin+=width;}
            else if pos<=p&&p<pos+width {thin+=1;thick+=usize::from(pos<p)+usize::from(p+1<pos+width);}
            else{thick+=1;}
            pos+=width;count+=1;
        }
        assert_eq!(pos as usize,n);
        let literals=symbols.syms.iter().take_while(|&&s|s!=symbols.eob).filter(|&&s|s>1).count();
        let bound=literals*20>=3*symbols.max_len;
        if bound{assert!(caps.len()*20>=3*n);}
        println!("{index},{},{n},{},{count},{thin},{thick},{},{bound},{},{}",symbols.syms.len(),caps.len(),caps.len()-count,caps.len()*20>=3*n,symbols.syms.len()<256);
    }
}
