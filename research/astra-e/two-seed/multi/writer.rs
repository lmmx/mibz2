use crate::{original,Table,writer};
use std::mem::MaybeUninit;

const OPEN:usize = usize::MAX;

#[derive(Clone,Copy,Default)]
struct Lane {
    id: usize,
    start: usize,
    limit: usize,
    len: usize,
    row: u32,
    edge: u32,
    landing: usize,
    blocked: bool,
}

#[derive(Default)]
pub struct Work {
    pub occupancy: [usize;5],
    pub passes: usize,
    pub hops: usize,
    pub relocated: usize,
    pub compacted: usize,
    pub rotated: usize,
}

fn repack<const K:usize,const AUDIT:bool>(
    storage: &mut [MaybeUninit<u8>], lanes:&mut [Lane;K], work:&mut Work,
) {
    let mut starts=[0;K];
    let open=lanes.iter().filter(|l|l.landing==OPEN).count();
    let free=storage.len()-lanes.iter().map(|l|l.len).sum::<usize>();
    let mut cursor=0;
    let mut rank=0;
    for (i,l) in lanes.iter_mut().enumerate() {
        starts[i]=cursor;
        let slack=if l.landing==OPEN {
            let s=free/open+usize::from(rank<free%open);
            rank+=1;
            s
        } else {0};
        cursor+=l.len+slack;
        l.limit=cursor;
        l.blocked=false;
    }
    assert_eq!(cursor,storage.len());
    for i in 0..K {
        let l=&lanes[i];
        if starts[i]<l.start && l.len>0 {
            storage.copy_within(l.start..l.start+l.len,starts[i]);
            if AUDIT {work.relocated+=l.len;}
        }
    }
    for i in (0..K).rev() {
        let l=&lanes[i];
        if starts[i]>l.start && l.len>0 {
            storage.copy_within(l.start..l.start+l.len,starts[i]);
            if AUDIT {work.relocated+=l.len;}
        }
    }
    for i in 0..K {lanes[i].start=starts[i];}
}

// The active lanes stay compact until one reaches a seed or its byte boundary.
// J is the exact live count, so the inner loop has no per-lane active mask.
#[inline(never)]
fn advance<const K:usize,const J:usize,const AUDIT:bool>(
    t:&Table, seeds:&[u32;K], active:&mut [Lane], store:&mut [MaybeUninit<u8>], work:&mut Work,
) {
    let lanes:&mut [Lane;J]=active.try_into().unwrap();
    let mask=(1<<t.bits)-1;
    let mut loaded:[u32;J]=std::array::from_fn(|i|lanes[i].edge);
    loop {
        let previous=loaded;
        let dest:[u32;J]=std::array::from_fn(|i|previous[i]&mask);
        loaded=std::array::from_fn(|i|t.edges[dest[i] as usize]);
        if AUDIT {work.occupancy[J]+=1;}
        let mut stop=false;
        for i in 0..J {
            let l=&mut lanes[i];
            let pos=l.start+l.len;
            let end=writer::place(previous[i]>>t.bits,t,store,pos,l.limit);
            if let Some(end)=end {
                l.len=end-l.start;
                l.row=dest[i];
                if AUDIT {work.hops+=1;}
                let mut hit=false;
                for &s in seeds {hit|=s==dest[i];}
                if hit {
                    l.landing=seeds.iter().position(|&s|s==dest[i]).unwrap();
                    stop=true;
                }
            } else {
                l.blocked=true;
                loaded[i]=previous[i];
                stop=true;
            }
        }
        if stop {
            for i in 0..J {lanes[i].edge=loaded[i];}
            return;
        }
    }
}

pub fn emit<const K:usize,const AUDIT:bool>(t:&Table,out:&mut Vec<u8>)->Work {
    assert!(K==3 || K==4);
    let mut work=Work::default();
    let m=t.edges.len();
    if m<K || !t.extra.is_empty() {
        original::Kernel::stock(t,out);
        return work;
    }
    let seeds:[u32;K]=std::array::from_fn(|i|((t.mark as usize+i*m/K)%m)as u32);
    let mut lanes:[Lane;K]=std::array::from_fn(|i|
        Lane{id:i,row:seeds[i],edge:t.edges[seeds[i] as usize],landing:OPEN,..Lane::default()});
    out.clear();
    out.reserve(t.n);
    let storage=&mut out.spare_capacity_mut()[..t.n];
    let mut pending=K;
    while pending>0 {
        if AUDIT {work.passes+=1;}
        repack::<K,AUDIT>(storage,&mut lanes,&mut work);
        let mut working=[Lane::default();K];
        let mut count=0;
        for &l in &lanes {if l.landing==OPEN {working[count]=l;count+=1;}}
        let before=pending;
        while count>0 {
            macro_rules! advance {($j:literal)=>{advance::<K,$j,AUDIT>(t,&seeds,&mut working[..count],storage,&mut work)}}
            match count {1=>advance!(1),2=>advance!(2),3=>advance!(3),4=>advance!(4),_=>unreachable!()}
            let mut i=0;
            while i<count {
                let l=working[i];
                if l.blocked || l.landing!=OPEN {
                    if l.landing!=OPEN {pending-=1;}
                    lanes[l.id]=l;
                    count-=1;
                    working[i]=working[count];
                } else {i+=1;}
            }
        }
        assert!(pending<before,"one complete remaining arc must fit per pass");
    }
    let mut pos=0;
    for l in &lanes {
        if l.start!=pos {
            storage.copy_within(l.start..l.start+l.len,pos);
            if AUDIT {work.compacted+=l.len;}
        }
        pos+=l.len;
    }
    let mut order:[usize;K]=std::array::from_fn(|i|i);
    let mut pos=0;
    let mut selected=0;
    for i in 0..K {
        let j=i+order[i..].iter().position(|&v|v==selected).unwrap();
        if i!=j {
            let extent=order[i..=j].iter().map(|&v|lanes[v].len).sum::<usize>();
            storage[pos..pos+extent].rotate_right(lanes[selected].len);
            order[i..=j].rotate_right(1);
            if AUDIT {work.rotated+=extent;}
        }
        pos+=lanes[selected].len;
        selected=lanes[selected].landing;
        if selected==0 {break;}
    }
    assert_eq!(selected,0);
    writer::finish(t,out,pos);
    work
}
