pub use mibz2::{bwt,runs,Kernels};
use mibz2::stream::{Scanner,SliceSource,Event};
use bwt::{InverseBwt,Segmented};
use std::{io::Write,path::Path};
#[repr(C)] struct Timespec {sec:i64,ns:i64}
unsafe extern "C" {fn clock_gettime(id:i32,ts:*mut Timespec)->i32;}
fn clock()->u64{let mut t=Timespec{sec:0,ns:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);t.sec as u64*1_000_000_000+t.ns as u64}
const SYSTEMATIC:[u64;11]=[1,19,61,108,150,194,240,282,329,371,407];

fn small() {
    let(mut old,mut a,mut b,mut c)=(old::Kernel::default(),new::Kernel::default(),new::Kernel::default(),new::Kernel::default());
    let mut ll=Vec::new();
    for n in 1..=9 {for seed in 0..1usize<<n {ll.push((0..n).map(|j|((seed>>j)&1) as u8).collect::<Vec<_>>());}}
    for n in [63usize,127,256,511] {
        ll.push((0..n).map(|i| if i%43<36 {255}else{(i%7) as u8}).collect());
        ll.push((0..n).map(|i| ((i/19)%4) as u8).collect());
        ll.push(vec![42;n]);
    }
    let mut checked=0;
    for l in ll {
        let caps:Vec<_>=l.iter().map(|&c|u16::from(c)<<8 |1).collect();
        let mut psi:Vec<_>=(0..l.len()).collect();psi.sort_by_key(|&i|l[i]);
        for mark in 0..l.len() {
            let mut p=mark;let expected:Vec<_>=(0..l.len()).map(|_|{p=psi[p];l[p]}).collect();
            let mut out=vec![];
            old.run::<true,true,false>(&caps,l.len(),mark as u32,16,&mut out);assert_eq!(out,expected);
            let (_,refedges)=old.edges();
            a.packed_run::<8>(&caps,l.len(),mark as u32,&mut out);assert_eq!(out,expected);a.check::<8>(&refedges);
            b.packed_run::<16>(&caps,l.len(),mark as u32,&mut out);assert_eq!(out,expected);b.check::<16>(&refedges);
            c.packed_run::<32>(&caps,l.len(),mark as u32,&mut out);assert_eq!(out,expected);c.check::<32>(&refedges);
            checked+=1;
        }
    }
    println!("{checked} independently sorted arbitrary marked columns, including single- and multiple-cycle examples");
}
fn main() {
    let args:Vec<_>=std::env::args().collect();
    if args[1]=="small" {small();return}
    let dir=Path::new(&args[2]);
    let full=args[3]=="geometry";
    let compressed=std::fs::read(&args[1]).unwrap();
    let mut s=Scanner::new(SliceSource(&compressed));
    let (mut reference,mut old,mut a,mut b,mut c)=(Segmented::default(),old::Kernel::default(),new::Kernel::default(),new::Kernel::default(),new::Kernel::default());
    let(mut out,mut want)=(vec![],vec![]);
    let mut blocks=vec![];
    let (mut accepted,mut rows)=(0,0);
    let mut geo=if full {Some(std::io::BufWriter::new(std::fs::File::create(dir.join("capacity.csv")).unwrap()))} else {None};
    if let Some(f)=&mut geo {writeln!(f,"block,n,old_edges,arm,packed,headers,high,stock_retained").unwrap();}
    let k=Kernels::detect();
    while let Event::Block{index,symbols,..}=s.next(Vec::new()).unwrap(){
        if !full && !SYSTEMATIC.contains(&index) {continue}
        let mut r=runs::Runs::default();
        let n=runs::front(&symbols,&mut r,k).unwrap();
        if !runs::takes_runs(&symbols) || n<65536 ||r.records().len()*20>=3*n {continue}
        assert!(reference.invert_runs(&r,n,symbols.orig_ptr,&mut want,k));
        if full {
            let p=symbols.orig_ptr;
            old.run::<true,true,false>(r.records(),n,p,16,&mut out);assert_eq!(out,want);
            let (bytes,refedges)=old.edges();
            macro_rules! verify {($v:ident,$page:literal)=>{{
                $v.packed_run::<$page>(r.records(),n,p,&mut out);assert_eq!(out,want);
                let z=$v.check::<$page>(&refedges);
                writeln!(geo.as_mut().unwrap(),"{index},{n},{bytes},{},{},{},{},{}",$page,z[0],z[1],z[2],z[3]).unwrap();
            }};}
            verify!(a,8);verify!(b,16);verify!(c,32);accepted+=1;rows+=n;
        } else {blocks.push((index,r,n,symbols.orig_ptr,want.clone()));}
    }
    if full {assert_eq!((accepted,rows),(228,205197794));return}
    assert_eq!(blocks.len(),11);
    let mut output:[Vec<u8>;4]=Default::default();
    let perms:[[usize;4];8]=std::array::from_fn(|r|std::array::from_fn(|j|([0,1,3,2][if r<4 {j}else{3-j}]+r%4)%4));
    println!("block,round,position,arm,prepare,emit,complete");
    for round in 0..=64usize {for off in 0..11 {
        let i=(if round%2==0 {off}else{10-off}+round*5)%11;
        let (id,r,n,p,expected)=&blocks[i];
        for (pos,&arm) in perms[(i+round)%8].iter().enumerate() {
            let o=&mut output[arm];let v=match arm {
                0=>old.native_price(r.records(),*n,*p,o),1=>a.native_price::<8>(r.records(),*n,*p,o),
                2=>b.native_price::<16>(r.records(),*n,*p,o),_=>c.native_price::<32>(r.records(),*n,*p,o)
            };
            assert_eq!(o,expected);
            if round>0 {println!("{id},{round},{pos},{},{},{},{}",[0,8,16,32][arm],v[0],v[1],v[0]+v[1]);}
        }
    }}
}
