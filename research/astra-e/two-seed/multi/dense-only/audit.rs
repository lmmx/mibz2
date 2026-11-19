pub use mibz2::{bwt,runs,Kernels};
use std::io::Read;
use std::sync::atomic::{AtomicU64,Ordering};
mod base;mod audit;
static COUNTS:[AtomicU64;5]=[const {AtomicU64::new(0)};5];
#[repr(C)] struct Timespec{sec:i64,nsec:i64}
unsafe extern "C" {fn clock_gettime(c:i32,t:*mut Timespec)->i32;}
fn audit_clock(c:i32)->u64 {
    let mut t=Timespec{sec:0,nsec:0};
    assert_eq!(unsafe{clock_gettime(c,&mut t)},0);
    t.sec as u64*1_000_000_000+t.nsec as u64
}
fn dense_charge(n:usize,m:usize,build:u64,emit:u64){
    for (dst,val) in COUNTS.iter().zip([1,n as u64,m as u64,build,emit]){dst.fetch_add(val,Ordering::Relaxed);}
}
fn run<I:bwt::InverseBwt+'static>(input:Vec<u8>,oracle:Option<Vec<u8>>){
    let begin=audit_clock(2);let wall=std::time::Instant::now();
    let mut r=mibz2::BzReader::parallel_workers_with::<I>(std::io::Cursor::new(input),1,4,Kernels::detect());
    let mut buf=vec![0u8;4<<20];let mut bytes=0;
    loop {
        let n=r.read(&mut buf).unwrap();if n==0{break;}
        if let Some(expected)=&oracle{assert_eq!(&buf[..n],&expected[bytes..bytes+n]);}
        bytes+=n;
    }
    drop(r);let elapsed=wall.elapsed().as_nanos();let process=audit_clock(2)-begin;
    assert_eq!(bytes,209700196);
    let v=std::array::from_fn::<_,5,_>(|j|COUNTS[j].load(Ordering::Relaxed));
    println!("{{\"count\":{},\"coded\":{},\"rows\":{},\"construct_ns\":{},\"emit_ns\":{},\"process_ns\":{process},\"wall_ns\":{elapsed},\"bytes\":{bytes}}}",v[0],v[1],v[2],v[3],v[4]);
}
fn main(){
    let a=std::env::args().collect::<Vec<_>>();let oracle=a.get(3).map(|s|std::fs::read(s).unwrap());
    let input=std::fs::read(&a[2]).unwrap();
    match a[1].as_str(){"base"=>run::<base::Candidate>(input,oracle),"audit"=>run::<audit::Candidate>(input,oracle),_=>panic!("bad worker")}
}
