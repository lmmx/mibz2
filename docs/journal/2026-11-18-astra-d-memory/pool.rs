// Deterministic allocation census. One scanner, W reusable worker states and 4W slots.
use mibz2::{bwt::LazyCells,decode_block,stream::{Event,Scanner,SliceSource},Kernels,Scratch};
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicUsize,Ordering::Relaxed};
struct Count;
static LIVE:AtomicUsize=AtomicUsize::new(0);
static PEAK:AtomicUsize=AtomicUsize::new(0);
fn grow(n:usize) { let now=LIVE.fetch_add(n,Relaxed)+n; PEAK.fetch_max(now,Relaxed); }
unsafe impl GlobalAlloc for Count {
    unsafe fn alloc(&self,l:Layout)->*mut u8 {
        let p=System.alloc(l); if !p.is_null() {grow(l.size());} p
    }
    unsafe fn alloc_zeroed(&self,l:Layout)->*mut u8 {
        let p=System.alloc_zeroed(l); if !p.is_null() {grow(l.size());} p
    }
    unsafe fn dealloc(&self,p:*mut u8,l:Layout) {LIVE.fetch_sub(l.size(),Relaxed);System.dealloc(p,l);}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8 {
        let q=System.realloc(p,l,n);
        if !q.is_null() {if n>=l.size() {grow(n-l.size());} else {LIVE.fetch_sub(l.size()-n,Relaxed);}}
        q
    }
}
#[global_allocator] static ALLOC:Count=Count;
fn main() {
    let a:Vec<_>=std::env::args().collect();
    let data=std::fs::read(&a[1]).unwrap();
    let k=Kernels::detect();
    println!("workers,slots,blocks,output_bytes,peak_heap,retained_heap,final_capacity,symbol_capacity");
    for w in [7,8,10] {
        let baseline=LIVE.load(Relaxed);
        PEAK.store(baseline,Relaxed);
        let mut workers:Vec<_>=(0..w).map(|_|Scratch::<LazyCells>::default()).collect();
        let mut outputs=vec![Vec::new();4*w];
        let mut symbols=vec![Vec::new();4*w];
        let mut scanner=Scanner::new(SliceSource(&data));
        let mut total=0;
        let mut i=0;
        loop {
            let s=i%(4*w);
            let Event::Block{symbols:b,..}=scanner.next(std::mem::take(&mut symbols[s])).unwrap() else {break};
            outputs[s].clear();
            decode_block(&b,k,&mut workers[i%w],&mut outputs[s]).unwrap();
            symbols[s]=b.syms;
            total+=outputs[s].len();
            i+=1;
        }
        println!("{},{},{},{},{},{},{},{}",w,4*w,i,total,PEAK.load(Relaxed)-baseline,
            LIVE.load(Relaxed)-baseline,outputs.iter().map(Vec::capacity).sum::<usize>(),
            2*symbols.iter().map(Vec::capacity).sum::<usize>());
    }
}
