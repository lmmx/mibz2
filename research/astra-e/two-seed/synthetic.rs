use crate::{Table,writer};
fn next_perm(p:&mut [usize])->bool {
    let Some(i)=(0..p.len()-1).rev().find(|&i|p[i]<p[i+1]) else {return false;};
    let j=(i+1..p.len()).rev().find(|&j|p[j]>p[i]).unwrap();
    p.swap(i,j);p[i+1..].reverse();true
}
pub fn run() {
    let mut cases=0;
    let mut matched=false;
    let mut different=false;
    let mut clipped=false;
    let mut boundary=false;
    for m in 1usize..=5 {
        let mut p=(0..m).collect::<Vec<_>>();
        loop {
            for layout in 0..4 {
                let words:Vec<Vec<u8>>=(0..m).map(|i|match (i+layout)%4 {
                    0=>vec![(19*i+1) as u8],
                    1=>vec![1+i as u8,0,255],
                    2=>vec![3+i as u8,2],
                    _=>vec![(231-i) as u8],
                }).collect();
                let mut t=Table{edges:vec![],extra:vec![],offsets:vec![0],spelling:vec![],
                    n:words.iter().map(Vec::len).sum(),mark:0,bits:3};
                for (i,w) in words.iter().enumerate() {
                    let c=if w.len()==1 {w[0] as u32} else {
                        t.spelling.extend(w);
                        t.offsets.push(t.spelling.len() as u32);
                        255+t.offsets.len() as u32-1
                    };
                    t.edges.push(p[i] as u32 | (c << t.bits));
                }
                for mark in 0..m {
                    t.mark=mark as u32;
                    let mut x=mark;
                    let mut want=Vec::new();
                    let mut rows=Vec::new();
                    loop {rows.push(x);want.extend(&words[x]);x=p[x];if x==mark {break;}}
                    let period=want.len();
                    clipped|=t.n%period!=0;
                    for i in period..t.n {want.push(want[i%period]);}
                    for seed in 0..m {
                        matched|=seed!=mark && rows.contains(&seed);
                        different|=!rows.contains(&seed);
                        for split in 0..=t.n {
                            let mut out=vec![133;2*t.n+17];
                            let w=writer::split_at::<true,false>(&t,&mut out,seed as u32,split);
                            assert_eq!(out,want);
                            assert!(w.moved<=2*w.carried);
                            let w=writer::split_at::<true,true>(&t,&mut out,seed as u32,split);
                            assert_eq!(out,want);
                            assert!(w.moved<=2*w.carried);
                            boundary|=w.boundary;
                            cases+=2;
                        }
                    }
                }
            }
            if !next_perm(&mut p){break;}
        }
    }
    assert!(matched && different && clipped && boundary);
    println!("{cases} exact packed-output comparisons, all permutations through five, every mark, second seed and byte split, four unequal label models, sealed gaps, short cycles and bounded repeats");
}
