// Check the actual release-standard Vec<u32> operations against the stated capacity laws.
fn grow(c: usize, required: usize) -> usize { (2*c).max(required).max(4) }
fn action(v:&mut Vec<u32>,w:usize,repeat:bool)->(usize,usize,usize) {
    let old=v.capacity();let len=v.len();
    let (mut events,mut initialized)=(0,0);
    if repeat {
        for x in 0..w{
            let cap=v.capacity();let before=v.len();
            v.push(x as u32);
            if v.capacity()!=cap {events+=1;initialized+=usize::from(cap>0)*before;}
        }
    }else{
        v.extend(0..w as u32);
        if v.capacity()!=old {events=1;initialized=usize::from(old>0)*len;}
    }
    (v.capacity(),events,initialized)
}
fn main(){
    let mut checked=0;
    for cap in 0..=64 {
        for len in 0..=cap {
            for width in 0..=32 {
                let mut b=Vec::<u32>::with_capacity(cap);
                let mut p=Vec::<u32>::with_capacity(cap);
                assert_eq!(b.capacity(),cap);assert_eq!(p.capacity(),cap);
                b.resize(len,42);p.resize(len,42);
                let bulk=action(&mut b,width,false);let pushes=action(&mut p,width,true);
                let end=len+width;
                assert_eq!(bulk.0,if end<=cap{cap}else{grow(cap,end)});
                let mut simulated=cap;
                while simulated<end {simulated=grow(simulated,simulated+1);}
                assert_eq!(pushes.0,simulated);assert_eq!(b,p);
                if (cap==0 || cap>=4) && width<=4 {
                    assert_eq!((bulk.0,bulk.1),(pushes.0,pushes.1));
                    assert!(pushes.2>=bulk.2 && pushes.2-bulk.2<=width.saturating_sub(1));
                }
                checked+=1;
            }
        }
    }
    println!("{checked} (initial capacity, initialized length, append width) states checked against actual range extend and push");
    for (c,l,widths) in [(0,0,&[5][..]),(0,0,&[8]),(0,0,&[5,6,1]),(2,2,&[4]),(4,3,&[2])] {
        let mut b=Vec::with_capacity(c);let mut p=Vec::with_capacity(c);
        b.resize(l,42);p.resize(l,42);
        let mut prev=Vec::new();
        for &w in widths {prev.push((action(&mut b,w,false),action(&mut p,w,true)));}
        println!("start cap={c} len={l} widths={widths:?} per-action ((bulk cap,growths,initialized at reallocation),(push cap,growths,initialized at reallocation)) {prev:?}");
    }
}
