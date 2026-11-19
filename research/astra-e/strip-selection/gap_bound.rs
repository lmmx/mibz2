use mibz2::{runs,Kernels};
use mibz2::stream::{Event,Scanner,SliceSource};
use std::collections::HashMap;

#[derive(Clone,Copy)]
struct Piece { byte:u8, kept:bool, kind:usize }
const FIXED:usize=2;

fn ledger(pieces:&[Piece],q:usize)->(i64,i64,i64,i64,usize,usize) {
    let count=pieces.len();
    let mut buckets:Vec<Vec<usize>>=(0..256).map(|_|vec![]).collect();
    for (i,p) in pieces.iter().enumerate(){buckets[p.byte as usize].push(i);}
    let top:Vec<_>=(0..count).collect();
    let bottom:Vec<_>=buckets.into_iter().flatten().collect();
    let mut single=vec![0i64;count];
    let(mut crossings,mut same,mut edges)=(0i64,0i64,0usize);
    for order in [top,bottom] {
        let mut previous=count;
        for next in order.into_iter().chain([count]) {
            let a=previous<count && pieces[previous].kept;
            let b=next<count && pieces[next].kept;
            let unlike=a!=b;crossings+=i64::from(unlike);
            let gain=if unlike {1}else{-1};
            let ca=previous<count && pieces[previous].kind<FIXED;
            let cb=next<count && pieces[next].kind<FIXED;
            if ca {single[previous]+=gain;}
            if cb {single[next]+=gain;}
            if ca && cb {edges+=1;same+=i64::from(!unlike);}
            previous=next;
        }
    }
    let mut sums=[0i64;2];
    let mut available=0;
    for (kind,answer) in sums.iter_mut().enumerate() {
        let mut choices:Vec<_>=(0..count).filter(|&i|pieces[i].kind==kind).map(|i|single[i]/2).collect();
        choices.sort_unstable_by(|a,b|b.cmp(a));
        assert!(choices.len()>=q);
        *answer=choices[..q].iter().sum();
        available+=choices.len();
    }
    let linear=sums[0]+sums[1];
    (crossings/2,linear,same,(linear+same).min((4*q) as i64),edges,available)
}
fn pieces(records:&[u16],mark:u32)->Vec<Piece> {
    let(mut i,mut row)=(0,0);
    let mut ans=vec![];
    while i<records.len(){
        let c=(records[i]>>8) as u8;let mut width=0;
        while i<records.len() && (records[i]>>8) as u8==c {width+=u32::from(records[i]&255);i+=1;}
        let end=row+width;
        let at=(row..end).contains(&mark);
        if width>16 && at {
            if row<mark {ans.push(Piece{byte:c,kept:false,kind:FIXED});}
            ans.push(Piece{byte:c,kept:true,kind:FIXED});
            if mark+1<end {ans.push(Piece{byte:c,kept:false,kind:FIXED});}
        } else {
            let kind=if !at && width==16 {0}else if !at && width==17 {1}else{FIXED};
            ans.push(Piece{byte:c,kept:width<=16,kind});
        }
        row=end;
    }
    ans
}

fn small() {
    let mut cases=0;let mut compared=0;
    for n in 2usize..=7 {
        for mut assignment in 0..3usize.pow(n as u32) {
            let mut pieces=Vec::new();
            for j in 0..n {
                let k=assignment%3;assignment/=3;
                pieces.push(Piece{byte:((j*5+j/2)%3) as u8,kept:k==0 || k==2 && j%2==0,kind:k});
            }
            let n0=pieces.iter().filter(|p|p.kind==0).count();let n1=pieces.iter().filter(|p|p.kind==1).count();
            for q in 0..=n0.min(n1) {
                let(gaps,_,_,bound,_,_)=ledger(&pieces,q);
                for mask in 0..1usize<<n {
                    let mut counts=[0;3];
                    for (i,p) in pieces.iter().enumerate(){if mask>>i&1!=0{counts[p.kind]+=1;}}
                    if counts!=[q,q,0] {continue}
                    let changed:Vec<_>=pieces.iter().enumerate().map(|(i,p)|Piece{byte:p.byte,kept:p.kept^(mask>>i&1!=0),kind:FIXED}).collect();
                    let after=ledger(&changed,0).0;assert!(gaps-after<=bound);
                    compared+=1;
                }
                cases+=1;
            }
        }
    }
    println!("{cases} finite budget problems, {compared} feasible simultaneous exchanges");
}
fn main() {
    let args:Vec<_>=std::env::args().collect();if args[1]=="small" {small();return}
    let mut properties=HashMap::new();
    for path in &args[2..] {
        let data=std::fs::read_to_string(path).unwrap();
        for line in data.lines().skip(1) {
            let v:Vec<usize>=line.split(',').map(|x|x.parse().unwrap()).collect();
            if v[3]==1 {properties.insert(v[1] as u64,(v[4],v[5],v[6],v[7]));}
        }
    }
    assert_eq!(properties.len(),233);
    let data=std::fs::read(&args[1]).unwrap();
    let mut sc=Scanner::new(SliceSource(&data));
    println!("stream,block,q,old_gaps,linear_upper,same_candidate_edges,gap_gain_upper,all_candidate_edges,eligible");
    while let Event::Block{index,stream,symbols,..}=sc.next(Vec::new()).unwrap(){
        let Some(&(q,_,_,_))=properties.get(&index) else{continue};
        let mut r=runs::Runs::default();runs::front(&symbols,&mut r,Kernels::detect()).unwrap();
        let (g,l,a,b,e,eligible)=ledger(&pieces(r.records(),symbols.orig_ptr),q);
        println!("{stream},{index},{q},{g},{l},{a},{b},{e},{eligible}");
    }
}
