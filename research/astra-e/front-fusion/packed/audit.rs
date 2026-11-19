use mibz2::{Kernels,runs,stream::{Event,Scanner,SliceSource}};
include!("log.rs");

fn small() {
    let alpha=[0x81,0x23,0x45];
    let mut cases=0;
    for len in 0..=8 {
        for word in 0..4usize.pow(len) {
            let mut src:Vec<u16>=(0..len).map(|i|((word>>(2*i))&3)as u16).collect();
            src.extend([4,61,0,255]);
            for stop in 0..=len as usize+1 {
                let mut s=src.clone();let v=compact(&mut s,&alpha,stop,900_000);
                assert!(!v.failed);
                assert_eq!(restore(&mut s,v.words,&alpha),v.consumed);
                assert_eq!(s,src);
                cases+=1;
            }
        }
    }
    let mut border=0;
    for w in [1,2,15,16,17,126,127,128,253,254,255,256,65534,65535,65536,65537,900000] {
        for opening in [false,true] {
            let v:u32=w+u32::from(opening);let mut s=Vec::new();
            if !opening{s.push(2);}
            for k in 0..v.ilog2(){s.push(((v>>k)&1)as u16);}
            s.extend([3,1,0,4,12345]);
            let gold=s.clone();let p=compact(&mut s,&alpha,1,900_000);
            assert_eq!(p.rows,u64::from(w));assert!(!p.failed);
            assert_eq!(restore(&mut s,p.words,&alpha),p.consumed);assert_eq!(gold,s);
            border+=1;
        }
    }
    let mut rejected=0;
    for final_suffix in [vec![9,4],vec![2,2],vec![2,0,0,0,0,4],{let mut s=vec![2];s.extend([1;22]);s.push(4);s}]{
        for cap in [5,15,900000]{
            let mut s=vec![2,0,3,1];s.extend(&final_suffix);let gold=s.clone();
            let p=compact(&mut s,&alpha,usize::MAX,cap);
            restore(&mut s,p.words,&alpha);assert_eq!(s,gold);
            rejected+=usize::from(p.failed);
        }
    }
    println!("{cases} exhaustive marked-prefix rollback checks, {border} width boundaries, {rejected} failed partially written prefixes restored");
}

fn main(){
    let arg=std::env::args().nth(1).unwrap();if arg=="small"{small();return;}
    let bytes=std::fs::read(arg).unwrap();let mut scan=Scanner::new(SliceSource(&bytes));
    let mut stock=runs::Runs::default();let k=Kernels::detect();
    println!("stream,n,mtf_symbols,stock_caps,run_heads,packed_words,extra16,extra32,rows_restored");
    while let Ok(Event::Block{index,symbols:mut b,..})=scan.next(Vec::new()){
        let n=runs::front(&b,&mut stock,k).unwrap();
        let alpha=&b.seq_to_unseq[..usize::from(b.eob-1)];
        let gold=b.syms.clone();let p=compact(&mut b.syms,alpha,usize::MAX,b.max_len as u64);
        assert!(!p.failed);assert_eq!(p.rows,n as u64);
        assert_eq!(p.consumed,gold.iter().position(|&s|s==b.eob).unwrap());
        let cap=stock.records();let(mut at,mut i,mut x16,mut x32)=(0,0,0,0);
        while at<p.words{
            let(c,w)=take(&b.syms,&mut at);
            let mut got=0;
            while i<cap.len() && cap[i]>>8==u16::from(c){got+=u32::from(cap[i]&255);i+=1;}
            assert_eq!(w,got);
            x16+=usize::from((255..=65535).contains(&w));x32+=usize::from(w>65535);
        }
        assert_eq!(i,cap.len());assert!(p.words<=cap.len()&&p.words<=p.consumed);
        let u=restore(&mut b.syms,p.words,alpha);assert_eq!(u,p.consumed);
        assert_eq!(b.syms,gold);
        println!("{index},{n},{},{},{},{},{x16},{x32},{u}",p.consumed,cap.len(),p.runs,p.words);
    }
}
