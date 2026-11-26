// Derive history and old-tail costs using an independent conventional MTF list of ordinal IDs.
use mibz2::{runs,stream::{Event,Scanner,SliceSource}};
use std::io::{BufWriter,Write};
#[derive(Default)]
struct Geometry{
    live:u64,hits:u64,misses:u64,ageouts:u64,shift_sum:u64,shift_max:usize,
    weighted_hit_distance:u64,first_touches:u64,over_63:u64,hits_rank_sum:u64,
}
fn main(){
    let args=std::env::args().collect::<Vec<_>>();let data=std::fs::read(&args[1]).unwrap();
    let mut scan=Scanner::new(SliceSource(&data));let mut hist=[[0u64;256];3];
    let mut out=BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    writeln!(out,"block,route,window,literals,hits,misses,ageouts,shift_sum,shift_max,hit_age_sum,first_touches,rank_ge64,hit_rank_sum").unwrap();
    while let Event::Block{index,symbols:b,..}=scan.next(Vec::new()).unwrap(){
        let mut list=std::array::from_fn::<_,256,_>(|i|i as u8);let mut last=[usize::MAX;256];
        let mut stats=[Geometry::default(),Geometry::default(),Geometry::default()];let mut t=0usize;
        let active=runs::takes_runs(&b);
        for &sym in &b.syms {
            if sym<2{continue;}if sym==b.eob{break;}
            let r=usize::from(sym-1).min(255);let id=list[r];list.copy_within(..r,1);list[0]=id;
            for (k,g)in stats.iter_mut().enumerate(){
                let w=16<<k;let mask=u64::MAX>>(64-w);
                let distance=if last[id as usize]==usize::MAX{usize::MAX}else{t-1-last[id as usize]};
                let recent=g.live.count_ones() as usize;
                assert_eq!(r<recent,distance<w);
                if distance<w{
                    g.hits+=1;g.weighted_hit_distance+=distance as u64;g.hits_rank_sum+=r as u64;
                    let lower=(1u64<<distance).wrapping_sub(1);
                    assert_eq!((g.live&lower).count_ones()as usize,r);
                    assert_ne!(g.live&(1u64<<distance),0);g.live^=1u64<<distance;
                }else{
                    g.misses+=1;let residual=r-recent;g.shift_sum+=residual as u64;g.shift_max=g.shift_max.max(residual);
                    if distance==usize::MAX{g.first_touches+=1;}
                    if active{hist[k][residual]+=1;}
                }
                g.over_63+=u64::from(r>=64);
                g.ageouts+=g.live>>(w-1);g.live=((g.live<<1)|1)&mask;
            }
            last[id as usize]=t;t+=1;
        }
        for (k,g)in stats.iter().enumerate(){
            writeln!(out,"{index},{},{},{t},{},{},{},{},{},{},{},{},{}",u8::from(active),16<<k,g.hits,g.misses,g.ageouts,g.shift_sum,g.shift_max,g.weighted_hit_distance,g.first_touches,g.over_63,g.hits_rank_sum).unwrap();
        }
    }
    let mut f=BufWriter::new(std::fs::File::create(&args[3]).unwrap());writeln!(f,"window,tail_rank,count").unwrap();
    for (k,h) in hist.iter().enumerate(){for (rank,v) in h.iter().enumerate(){writeln!(f,"{},{rank},{v}",16<<k).unwrap();}}
}
