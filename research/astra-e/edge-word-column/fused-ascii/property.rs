// Finite functional stress test of the actual queue publisher and emitter. Unit bands
// suffice for every permutation, including permutations outside the BWT constraints.
#[derive(Clone,Copy)]
struct Band {end:u32,delta:u32,code:u32}
include!("queue.rs");
include!("emit.rs");
fn repeat(n:usize,out:&mut Vec<u8>){
    assert!(!out.is_empty()&&out.len()<=n);
    while out.len()<n{let w=out.len().min(n-out.len());out.extend_from_within(..w);}
}
fn next(a:&mut [usize])->bool{
    let Some(i)=(1..a.len()).rev().find(|&j|a[j-1]<a[j]) else{return false};
    let k=(i..a.len()).rev().find(|&j|a[j]>a[i-1]).unwrap();
    a.swap(i-1,k);a[i..].reverse();true
}
fn main(){
    let mut cases=0u64;let mut short=0u64;let mut nondivisor=0u64;let mut alias=0u64;
    let mut dir=PairDirectory::default();
    let dict=&[0x21,0,0x7f];let first=257;
    for m in 1..=5usize{
        let mut target=(0..m).collect::<Vec<_>>();
        loop{
            for bits in 0..1usize<<(2*m){
                let mut names=Vec::new();let mut incoming=vec![0;m];let mut predecessor=vec![0;m];
                let mut total=0;
                for (x,&y) in target.iter().enumerate(){
                    let c=[0,127,128,256][(bits>>(2*x))&3];names.push(c);
                    predecessor[y]=x;incoming[y]=if c==256{127}else{c};total+=if c==256{3}else{1};
                }
                let mut queues=vec![Vec::new();256];let mut cursors=[0usize;256];let mut bands=Vec::<Band>::new();let mut k=0u32;
                for c in 0..256{
                    cursors[c]=bands.len();for y in 0..m{if incoming[y]==c as u32{
                        let x=predecessor[y];let delta=(x as u32).wrapping_sub(k);
                        let code=names[x];
                        if bands.len()>cursors[c]&&bands.last().is_some_and(|b|b.code==code&&b.delta==delta){
                            bands.last_mut().unwrap().end+=1;
                        }else{bands.push(Band{end:k+1,delta,code});}
                        queues[c].push(y as u32);k+=1;
                    }}
                }
                let w=(usize::BITS-(m-1).leading_zeros()).max(1);
                for mark in 0..m{
                    let mut edges=vec![0;m];
                    dir.write(&bands,&cursors,&queues,m as u32,mark as u32,w,first,&mut edges);
                    for x in 0..m{
                        let y=target[x];let pair=names[x]<128&&names[y]<128&&y!=mark;
                        let (end,code)=if pair{(target[y],first+128*names[x]+names[y])}else{(y,names[x])};
                        assert_eq!(edges[x],end as u32|(code<<w));
                        alias+=u64::from(pair&&names[target[y]]<128&&target[y]!=mark);
                    }
                    let mut lap=Vec::new();let mut x=mark;
                    loop{if names[x]==256{lap.extend_from_slice(dict);}else{lap.push(names[x] as u8);}
                         x=target[x];if x==mark{break;}}
                    let mut out=Vec::new();emit_pair(&edges,dict,&[0,3],total,mark as u32,w,first,&mut out);
                    assert_eq!(out.len(),total);for (i,c) in out.iter().enumerate(){assert_eq!(*c,lap[i%lap.len()]);}
                    short+=u64::from(lap.len()<total);nondivisor+=u64::from(total%lap.len()!=0);
                    cases+=1;
                }
            }
            if !next(&mut target){break;}
        }
    }
    println!("{cases} arbitrary marked labelled permutations through five rows, {short} short marked byte laps, {nondivisor} nondivisor horizons, {alias} overlapping eligible starts checked");
}
