pub use mibz2::{bwt,runs,Kernels};
use bwt::InverseBwt;
use mibz2::stream::{Scanner,SliceSource,Event};
use std::{cell::RefCell,io::{Write,Read},path::Path};
const IDS:[usize;11]=[1,19,61,108,150,194,240,282,329,371,407];
const NAMES:[&[u8];9]=[b"start\0",b"scan_queues\0",b"strip_images\0",b"endpoint_lists\0",b"endpoint_eliminate\0",b"bottom_positions\0",b"bands_spellings\0",b"edge_table\0",b"residual_emit\0"];
thread_local!{
    static STAMPS: RefCell<[u64;9]> = const {RefCell::new([0;9])};
    static COUNT: std::cell::Cell<bool> = const {std::cell::Cell::new(false)};
}
fn cap<T>(v:&Vec<T>)->usize{v.capacity()*std::mem::size_of::<T>()}
#[repr(C)] struct Timespec{s:i64,ns:i64}
extern "C"{fn clock_gettime(id:i32,x:*mut Timespec)->i32;}
#[inline(never)]
fn cpu()->u64{let mut t=Timespec{s:0,ns:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);t.s as u64*1_000_000_000+t.ns as u64}
#[inline(never)]
fn callgrind(op:usize,arg:usize){
    let p=[0x43540000+op,arg,0,0,0,0];
    unsafe{std::arch::asm!("rol rdi,3","rol rdi,13","rol rdi,61","rol rdi,51","xchg rbx,rbx",in("rax") p.as_ptr(),inout("rdx") 0usize=>_,inout("rdi") 0usize=>_,options(nostack));}
}
#[inline(never)]
fn cut(index:usize){
    STAMPS.with(|s|s.borrow_mut()[index]=cpu());
    if COUNT.get(){
        if index==0 {callgrind(4,0);callgrind(1,0);}
        else {callgrind(3,NAMES[index].as_ptr() as usize);if index==8{callgrind(5,0);}}
    }
}
struct Block{idx:usize,n:usize,p:u32,r:runs::Runs,reference:Vec<u8>}
fn main(){
    let args:Vec<_>=std::env::args().collect();
    let k=Kernels::detect();
    let data=std::fs::read(&args[1]).unwrap();
    let dir=Path::new(&args[2]);
    let mut scanner=Scanner::new(SliceSource(&data));
    let mut original=original::kernel::Kernel::default();
    let mut metered=meter::kernel::Kernel::default();
    let mut lazy=bwt::LazyCells::default();
    let(mut out,mut refout)=(Vec::new(),Vec::new());
    let mut blocks=Vec::new();
    let only_callgrind=args[3]=="callgrind";
    let mut geometry=std::io::BufWriter::new(std::fs::File::create(dir.join(if only_callgrind{"ir-geometry.csv"}else{"geometry.csv"})).unwrap());
    writeln!(geometry,"block,rows,caps,maxruns,retained,strips,nodes,steps,rules,bands,words,spelling,edgetable,queue_cap,strip_cap,link_cap,rule_cap,band_cap,dictionary_cap,edge_cap,other_cap").unwrap();
    while let Event::Block{index,symbols,..}=scanner.next(Vec::new()).unwrap(){
        let idx=index as usize;
        if only_callgrind && idx>1{break}
        if !runs::takes_runs(&symbols){continue}
        let mut r=runs::Runs::default();
        let n=runs::front(&symbols,&mut r,k).unwrap();
        if n<65536 || r.records().len()*20>=n*3{continue}
        let p=symbols.orig_ptr;
        assert!(lazy.invert_runs(&r,n,p,&mut refout,k));
        let c=metered.run::<true,true,false>(r.records(),n,p,16,&mut out);
        assert!(out==refout);
        let unmodified=original.run::<true,true,false>(r.records(),n,p,16,&mut out);
        assert!(out==refout && c.steps==unmodified.steps && c.table_bytes==unmodified.table_bytes);
        write!(geometry,"{idx},{n},{},{},{},{},{},{},{},{},{},{},{}",r.records().len(),c.runs,c.retained,c.thick,c.open_nodes,c.steps,c.rules,c.bands,c.words,c.traced,c.table_bytes).unwrap();
        for v in metered.capacities(){write!(geometry,",{v}").unwrap()}
        writeln!(geometry).unwrap();
        if IDS.contains(&idx){blocks.push(Block{idx,n,p,r,reference:refout.clone()});}
    }
    if only_callgrind {
        assert_eq!(blocks.len(),1); let b=&blocks[0];
        COUNT.set(true);cut(0);metered.run::<true,true,false>(b.r.records(),b.n,b.p,16,&mut out);cut(8);COUNT.set(false);
        assert!(out==b.reference);
        return;
    }
    assert_eq!(blocks.len(),11);
    let mut f=std::io::BufWriter::new(std::fs::File::create(dir.join("phases.csv")).unwrap());
    writeln!(f,"round,block,first,scan_queues,strip_images,endpoint_lists,endpoint_eliminate,bottom_positions,bands_spellings,edge_table,residual_emit,meter_total,original_total").unwrap();
    for r in 1..=24{
        for (i,b) in blocks.iter().enumerate(){
            let mut times=[0;9];let mut untouched=0;
            for j in 0..2{
                if (r+i+j)%2==0{
                    cut(0);metered.run::<true,true,false>(b.r.records(),b.n,b.p,16,&mut out);cut(8);
                    times=STAMPS.with(|s|*s.borrow())
                }else{
                    let t=cpu();original.run::<true,true,false>(b.r.records(),b.n,b.p,16,&mut out);untouched=cpu()-t;
                }
                assert!(out==b.reference);
            }
            write!(f,"{r},{},{},",b.idx,if (r+i)%2==0{"meter"}else{"original"}).unwrap();
            for pair in times.windows(2){write!(f,"{},",pair[1]-pair[0]).unwrap()}
            writeln!(f,"{},{}",times[8]-times[0],untouched).unwrap();
        }
    }
}
