// Rotate the same generated buffer across arms. Interleave the stock library front.
use mibz2::{block::Symbols,runs,stream::{Event,Scanner,SliceSource},Kernels};
#[path="stock.rs"] mod original;
#[path="wide.rs"] mod composed;
#[cfg(compact)] #[path="compact.rs"] mod narrow;
struct Runs{runs:Vec<u16>,counts:[u32;256],per:[u32;256],lit:Vec<u64>,one:Vec<u64>}
impl Default for Runs{
    fn default()->Self{Self{runs:Vec::new(),counts:[0;256],per:[0;256],lit:Vec::new(),one:Vec::new()}}
}
#[repr(C)] struct Time{sec:i64,nano:i64}
unsafe extern "C"{fn clock_gettime(kind:i32,t:*mut Time)->i32;}
fn cpu()->i64{
    let mut t=Time{sec:0,nano:0};
    assert_eq!(unsafe {clock_gettime(3,&mut t)},0);
    t.sec*1_000_000_000+t.nano
}
#[inline(never)]
fn measured(kind:usize,corpus:&[Symbols],shared:&mut Runs,real:&mut runs::Runs,k:Kernels)->(i64,usize){
    let mut count=0;let a=cpu();
    match kind {
        #[cfg(not(compact))]
        0=>for b in corpus{std::hint::black_box(runs::front(b,real,k).unwrap());count+=real.records().len();std::hint::black_box(&*real);},
        #[cfg(compact)]
        0=>for b in corpus{std::hint::black_box(unsafe{narrow::front_avx2(b,shared)}.unwrap());count+=shared.runs.len();std::hint::black_box(&*shared);},
        1=>for b in corpus{std::hint::black_box(unsafe{original::front_avx2(b,shared)}.unwrap());count+=shared.runs.len();std::hint::black_box(&*shared);},
        2=>for b in corpus{std::hint::black_box(unsafe{composed::front_avx2(b,shared)}.unwrap());count+=shared.runs.len();std::hint::black_box(&*shared);},
        _=>unreachable!()
    }
    (cpu()-a,count)
}
#[inline(never)]
fn offset(depth:usize,kind:usize,corpus:&[Symbols],shared:&mut Runs,real:&mut runs::Runs,k:Kernels)->(i64,usize){
    if depth==0{return measured(kind,corpus,shared,real,k);}
    let padding=[depth as u8;320];std::hint::black_box(&padding);
    let res=offset(depth-1,kind,corpus,shared,real,k);
    std::hint::black_box(&padding);res
}
fn main(){
    let data=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scanner=Scanner::new(SliceSource(&data));let mut corpus=vec![];
    while let Event::Block{symbols:b,..}=scanner.next(Vec::new()).unwrap(){if runs::takes_runs(&b){corpus.push(b);}}
    assert_eq!(corpus.len(),383);
    let mut shared=Runs::default();let mut real=runs::Runs::default();let k=Kernels::detect();
    let depth:usize=std::env::args().nth(2).unwrap().parse().unwrap();
    let orders=[[0,1,2],[1,2,0],[2,0,1],[2,1,0],[1,0,2],[0,2,1]];
    for round in 0usize..=12{
        for (pos,&kind) in orders[round.saturating_sub(1usize)%6].iter().enumerate(){
            let (ns,count)=offset(depth,kind,&corpus,&mut shared,&mut real,k);
            assert_eq!(count,16326954);
            if round!=0{println!("{depth},{round},{pos},{},{ns},{count}",[if cfg!(compact){"compact"}else{"library"},"stock","wide"][kind]);}
        }
    }
}
