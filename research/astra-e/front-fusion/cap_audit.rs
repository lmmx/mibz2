use mibz2::{Kernels,runs,stream::{Scanner,SliceSource,Event}};
include!("cap_count.rs");
fn masks(s:&[u16],eob:u16)->(Vec<u64>,Vec<u64>,usize){
    let end=s.iter().position(|&v|v==eob && v>1).unwrap();
    let mut lit=vec![0;end/64+3];let mut one=lit.clone();
    for (i,&v) in s[..end].iter().enumerate(){if v>1{lit[i/64]|=1<<(i%64);}if v!=0{one[i/64]|=1<<(i%64);}}
    (lit,one,end)
}
fn main(){
    let arg=std::env::args().nth(1).unwrap();
    if arg=="small"{
        let mut tested=0;for offset in 0..64{for t in 0..=21{for style in 0..4{
            let mut s=vec![2u16;offset];s.extend((0..t).map(|i|match style{0=>0,1=>1,_=>((i+style)%2)as u16}));s.extend([3,2,4]);
            let(l,o,end)=masks(&s,4);let n=(0..t).map(|i|(u64::from(s[offset+i])+1)<<i).sum::<u64>();let prefix=if offset==0{n.div_ceil(127)}else{offset as u64-1+(1+n).div_ceil(127)};
            assert_eq!(capped_count(&l,&o,end).unwrap().0 as u64,prefix+2);tested+=1;
        }}}
        println!("{tested} cap-count checks covering every literal bit offset and all valid RUNA/B lengths including opening runs");
        return;
    }
    let data=std::fs::read(arg).unwrap();let mut scan=Scanner::new(SliceSource(&data));let mut scratch=runs::Runs::default();
    println!("stream,syms,bound,n,literals,qualifying_long_runs,true_caps,necessary_reject,potential_long_search");
    while let Ok(Event::Block{index,symbols,..})=scan.next(Vec::new()){
        if !runs::takes_runs(&symbols){continue;}
        let n=runs::front(&symbols,&mut scratch,Kernels::detect()).unwrap();let(l,o,end)=masks(&symbols.syms,symbols.eob);
        let (caps,events)=capped_count(&l,&o,end).unwrap();assert_eq!(caps,scratch.records().len());
        let lits:usize=l.iter().map(|s|s.count_ones()as usize).sum();
        let potential=(lits+1+symbols.max_len/127)*20>=3*symbols.max_len && lits*20<3*symbols.max_len;
        println!("{index},{},{},{n},{lits},{events},{caps},{},{potential}",symbols.syms.len(),symbols.max_len,caps*20>=3*symbols.max_len);
    }
}
