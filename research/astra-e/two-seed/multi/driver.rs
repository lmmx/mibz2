const ARMS:[&str;8]=["stock","stock twin","two","two twin","three","three twin","four","four twin"];
fn run(arm:usize,t:&Table,out:&mut Vec<u8>) {
    match arm/2 {
        0=>original::Kernel::stock(t,out),
        1=>{writer::split::<false,true>(t,out);},
        2=>{multi::emit::<3,false>(t,out);},
        3=>{multi::emit::<4,false>(t,out);},
        _=>unreachable!(),
    }
}
fn next_perm(p:&mut [usize])->bool {
    let Some(i)=(0..p.len()-1).rev().find(|&i|p[i]<p[i+1]) else {return false;};
    let j=(i+1..p.len()).rev().find(|&j|p[j]>p[i]).unwrap();
    p.swap(i,j);p[i+1..].reverse();true
}
fn fixtures() {
    let mut calls=0;
    let mut max_pass=[0;5];
    for m in 1..=7 {
        let mut p=(0..m).collect::<Vec<_>>();
        loop {
            for setting in 0..4 {
                let words=(0..m).map(|i|match (i+setting)%4 {
                    0=>vec![i as u8;1],1=>vec![14+i as u8,0,255],2=>vec![3,4+i as u8],
                    _=>vec![19+i as u8;31],
                }).collect::<Vec<_>>();
                let mut t=Table{edges:vec![],extra:vec![],offsets:vec![0],spelling:vec![],n:words.iter().map(Vec::len).sum(),mark:0,bits:3};
                for (i,w) in words.iter().enumerate() {
                    let c=if w.len()==1 {w[0] as u32} else {
                        t.spelling.extend(w);t.offsets.push(t.spelling.len() as u32);254+t.offsets.len() as u32
                    };
                    t.edges.push(p[i] as u32 | (c<<t.bits));
                }
                for mark in 0..m {
                    t.mark=mark as u32;
                    let mut row=mark;let mut want=Vec::new();
                    loop {want.extend(&words[row]);row=p[row];if row==mark{break;}}
                    let period=want.len();for i in period..t.n{want.push(want[i%period]);}
                    let mut out=vec![23;t.n+2];
                    let a=multi::emit::<3,true>(&t,&mut out);assert_eq!(out,want);
                    let b=multi::emit::<4,true>(&t,&mut out);assert_eq!(out,want);
                    assert!(a.passes<=3 && b.passes<=4);
                    max_pass[3]=max_pass[3].max(a.passes);max_pass[4]=max_pass[4].max(b.passes);
                    calls+=2;
                }
            }
            if !next_perm(&mut p){break;}
        }
    }
    // Single labelled edges can force one new closure per pass.
    let mut t=Table{edges:vec![],extra:vec![],spelling:vec![],offsets:vec![0],n:113,mark:0,bits:2};
    for (i,size) in [1,33,38,41].into_iter().enumerate() {
        t.spelling.extend(std::iter::repeat_n(i as u8,size));
        t.offsets.push(t.spelling.len() as u32);
        t.edges.push(((256+i as u32)<<2)|((i as u32+1)%4));
    }
    let mut out=vec![];
    let forced=multi::emit::<4,true>(&t,&mut out);
    assert_eq!(forced.passes,4);assert_eq!(out,t.spelling);
    max_pass[4]=max_pass[4].max(forced.passes);calls+=1;
    assert_eq!(max_pass[3],3);assert_eq!(max_pass[4],4);
    println!("{calls} exact outputs, all permutations through 7, variable word lengths up to 31, all marks, crossed all possible pass counts");
}
fn main() {
    let args=std::env::args().collect::<Vec<_>>();
    if args[1]=="synthetic" {return fixtures();}
    let gate=args[1]=="gate";
    assert!(gate || args[1]=="price");
    let data=std::fs::read(Path::new(&args[2])).unwrap();
    let k=Kernels::detect();
    let mut scanner=Scanner::new(SliceSource(&data));
    let mut builder=original::Kernel::default();let mut base=Segmented::default();
    let mut runs=runs::Runs::default();let mut outs:[Vec<u8>;8]=Default::default();
    let mut gold=Vec::new();let mut samples=Vec::new();let(mut admissions,mut bytes)=(0,0);
    if gate {println!("block,n,m,k,passes,hops,one,two,three,four,relocated,compacted,rotated,cap");}
    while let Event::Block{index,symbols,..}=scanner.next(Vec::new()).unwrap() {
        if (!gate && !IDS.contains(&index)) || !runs::takes_runs(&symbols) {continue;}
        let n=runs::front(&symbols,&mut runs,k).unwrap();
        if runs.records().len()*20>=3*n && (n<65536 || !fits(runs.records(),symbols.orig_ptr)) {continue;}
        let t=builder.make(runs.records(),n,symbols.orig_ptr);
        assert!(base.invert_runs(&runs,n,symbols.orig_ptr,&mut gold,k));
        if gate {
            admissions+=1;bytes+=n;
            for (i,out) in outs.iter_mut().enumerate(){run(i,&t,out);assert_eq!(out,&gold);}
            assert!(outs.iter().all(|o|o.capacity()==outs[0].capacity()));
            let a=multi::emit::<3,true>(&t,&mut outs[4]);assert_eq!(outs[4],gold);
            let b=multi::emit::<4,true>(&t,&mut outs[6]);assert_eq!(outs[6],gold);
            for (i,x) in [a,b].iter().enumerate() {
                assert_eq!(x.hops,t.edges.len());
                println!("{index},{n},{},{},{},{},{},{},{},{},{},{},{},{}",t.edges.len(),i+3,x.passes,x.hops,
                          x.occupancy[1],x.occupancy[2],x.occupancy[3],x.occupancy[4],x.relocated,x.compacted,x.rotated,outs[0].capacity());
            }
        } else {samples.push((index,t,std::mem::take(&mut gold)));}
    }
    if gate {
        assert_eq!((admissions,bytes),(233,209697662));
        eprintln!("{admissions} admitted tables, {bytes} pre-RLE bytes and equal capacity versus independent Segmented and stock edge");
        return;
    }
    assert_eq!(samples.len(),14);
    println!("round,block,position,arm,cpu_ns");
    for lap in 0usize..=40 {
        for off in 0..samples.len() {
            let j=(lap*5+if lap%2==0{off}else{samples.len()-off-1})%samples.len();
            let (id,t,want)=&samples[j];
            for pos in 0..8 {
                let shift=(lap.saturating_sub(1)+j)%8;
                let arm=(shift+if (lap.saturating_sub(1)/8+j)%2==0{pos}else{8-pos})%8;
                let start=cpu();run(arm,t,&mut outs[arm]);let elapsed=cpu()-start;
                assert_eq!(&outs[arm],want);
                if lap>0 {println!("{lap},{id},{pos},{},{elapsed}",ARMS[arm]);}
            }
        }
    }
}
