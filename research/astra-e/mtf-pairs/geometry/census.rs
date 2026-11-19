use mibz2::{runs,stream::{Event, Scanner,SliceSource},Kernels};
use std::io::{BufWriter,Write};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let data=std::fs::read(&args[1]).unwrap();
    let mut scan=Scanner::new(SliceSource(&data));
    let mut runs=runs::Runs::default();
    let mut b=BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    let mut h=BufWriter::new(std::fs::File::create(&args[3]).unwrap());
    writeln!(b,"block,stream,route,n,syms,literals,threshold,low,greedy_word_pairs,adjacent_input_pairs,unrestricted_pairs,used_words").unwrap();
    let mut hist=[[[0u64;16];16];2];
    while let Event::Block{index,stream,symbols:s,..}=scan.next(Vec::new()).unwrap(){
        let n=runs::front(&s,&mut runs,Kernels::detect()).unwrap();
        let ranks=s.syms.iter().take_while(|&&s0| s0<=1 || s0!=s.eob).enumerate()
            .filter(|&(_, &s)| s>1).map(|(pos,&s)|(pos,(s-1) as usize)).collect::<Vec<_>>();
        let active=runs::takes_runs(&s);
        for threshold in [2,4,8,16] {
            let low=ranks.iter().filter(|&&(_,rank)|rank<threshold).count();
            let mut counts=[0;4];
            for mode in 0..3 {
                let mut i=0;
                let mut last_pair_word=usize::MAX;
                while i+1<ranks.len() {
                    let (pos,x)=ranks[i];
                    let (end,y)=ranks[i+1];
                    if x<threshold && y<threshold && (mode==2 || pos/64==end/64) && (mode!=1 || end==pos+1) {
                        counts[mode]+=1;
                        if mode==0 {
                            if last_pair_word!=pos/64 {counts[3]+=1; last_pair_word=pos/64;}
                            if threshold==16 && active {hist[usize::from(end!=pos+1)][x][y]+=1;}
                        }
                        i+=2;
                    }else{i+=1;}
                }
            }
            writeln!(b,"{index},{stream},{},{n},{},{},{threshold},{low},{},{},{},{}",u8::from(active),s.syms.len(),ranks.len(),counts[0],counts[1],counts[2],counts[3]).unwrap();
        }
    }
    writeln!(h,"first_rank,second_rank,contiguous,separated").unwrap();
    for i in 1..16 {for j in 1..16 {writeln!(h,"{i},{j},{},{}",hist[0][i][j],hist[1][i][j]).unwrap();}}
}
