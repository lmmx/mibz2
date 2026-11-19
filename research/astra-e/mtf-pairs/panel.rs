use mibz2::{block::Symbols,runs,stream::{Event,Scanner,SliceSource},Kernels};
mod stock;
mod single;
mod pair;
mod wide;
#[cfg(compact)] mod compact;

#[derive(Default)]
struct Runs {
    runs:Vec<u16>, counts:Counts, per:Counts, lit:Vec<u64>, one:Vec<u64>,
}
struct Counts([u32;256]);
impl Default for Counts {fn default()->Self{Self([0;256])}}
impl std::ops::Deref for Counts {type Target=[u32;256];fn deref(&self)->&Self::Target{&self.0}}
impl std::ops::DerefMut for Counts {fn deref_mut(&mut self)->&mut Self::Target{&mut self.0}}
type Front=unsafe fn(&Symbols,&mut Runs)->Option<usize>;
#[cfg(not(compact))]
const ARMS:[(&str,Front);4]=[("stock",stock::front_avx2),("single",single::front_avx2),("pair",pair::front_avx2),("wide",wide::front_avx2)];
#[cfg(compact)]
const ARMS:[(&str,Front);3]=[("stock",stock::front_avx2),("wide",wide::front_avx2),("compact",compact::front_avx2)];
#[repr(C)]
struct Timespec{i:i64,n:i64}
unsafe extern "C"{fn clock_gettime(c:i32,t:*mut Timespec)->i32;}
fn cpu()->u64{let mut t=Timespec{i:0,n:0};unsafe{assert_eq!(clock_gettime(3,&mut t),0);} (t.i*1_000_000_000+t.n) as u64}
fn check(s:&Symbols, scratch:&mut Runs, reference:&mut runs::Runs){
    let n=runs::front_scalar(&s,reference);
    let mut previous=None;
    for (_name,arm) in ARMS {
        let result=unsafe{arm(s,scratch)};
        if let Some(p)=previous{assert_eq!(p,result);}
        previous=Some(result);
        if let Some(z)=result {
            assert_eq!(&Ok(z),&n);
            assert_eq!(scratch.runs,reference.records());
            let mut per=[0;256];
            let mut counts=[0;256];
            for &r in reference.records(){
                per[(r>>8)as usize]+=1;
                counts[(r>>8)as usize]+=u32::from(r&255);
            }
            assert_eq!(counts,*scratch.counts);
            assert_eq!(per,*scratch.per);
        }
    }
}
fn fixtures(scratch:&mut Runs, reference:&mut runs::Runs)->usize{
    let mut total=0;
    let mut table=std::array::from_fn(|i| (i as u8).wrapping_mul(59));
    for i in 1..16 {for j in 1..16 {for gap in [0,1,2,6,7,8,20,21,22,64]{
        let mut syms=vec![i+1];syms.extend(std::iter::repeat_n(1,gap));syms.extend([j+1,17,32,33,256,257]);
        check(&Symbols{crc:0,orig_ptr:0,seq_to_unseq:table,eob:257,syms,max_len:900000},scratch,reference);total+=1;
    }}}
    let mut state=1u32;
    for len in 0..180 {for seq in 0..12 {
        let mut syms=vec![];
        for _ in 0..len {state=state.wrapping_mul(1664525).wrapping_add(1013904223);syms.push(match state>>28 {0..=5=>(state&1) as u16,6..=12=>2+(state%15) as u16,13..=14=>17+(state%17) as u16,_=>(state%512) as u16});}
        if seq%4!=0 {syms.push(257);}
        if seq%3==0 {syms.extend([0,0,1,2,257]);}
        table.rotate_left(1);
        check(&Symbols{crc:0,orig_ptr:0,seq_to_unseq:table,eob:257,syms,max_len:if seq%2==0{len/2}else{900000}},scratch,reference);total+=1;
    }}
    total
}
fn main(){
    for enabled in [is_x86_feature_detected!("avx2"),is_x86_feature_detected!("bmi1"),is_x86_feature_detected!("bmi2"),is_x86_feature_detected!("lzcnt"),is_x86_feature_detected!("popcnt")] {assert!(enabled);}
    let args=std::env::args().collect::<Vec<_>>();
    let data=std::fs::read(&args[1]).unwrap();let mut scanner=Scanner::new(SliceSource(&data));
    let mut corpus=Vec::new();let mut reference=runs::Runs::default();let mut scratch=Runs::default();
    if args[2]=="check"{let tests=fixtures(&mut scratch,&mut reference);eprintln!("Synthetic {tests}");}
    let mut blocks=0;let mut symbols=0;let mut rows=0;
    while let Event::Block{index,symbols:s,..}=scanner.next(Vec::new()).unwrap(){
        blocks+=1;if args[2]=="check"{check(&s,&mut scratch,&mut reference);}
        if runs::takes_runs(&s){symbols+=s.syms.len();rows+=runs::front(&s,&mut reference,Kernels::detect()).unwrap();corpus.push((index,s));}
    }
    eprintln!("Corpus blocks {blocks}, record routes {}, active symbols {symbols}, rows {rows}",corpus.len());
    if args[2]=="check"{return;}
    let round_count:usize=args[2].parse().unwrap();
    println!("round,position,arm,front_ns,blocks,records");
    let orders=[[0,1,2],[1,2,0],[2,0,1],[2,1,0],[1,0,2],[0,2,1]];
    let ids=if args.get(3).map(String::as_str)==Some("wide"){[0,2,3]}else{[0,1,2]};
    let mut buffers=[Runs::default(),Runs::default(),Runs::default(),Runs::default()];
    for round in 0..=round_count{
        let order=orders[round.saturating_sub(1)%orders.len()];
        for (position,id) in order.iter().enumerate(){
            let id=ids[*id];let (name,arm)=ARMS[id];let t=cpu();let mut records=0;
            for (_,s) in &corpus{
                let r=&mut buffers[id];unsafe {std::hint::black_box(arm(s,r).unwrap());}
                records+=r.runs.len();std::hint::black_box(&*r);
            }
            let elapsed=cpu()-t;
            if round>0 {println!("{round},{position},{name},{elapsed},{},{records}",corpus.len());}
        }
    }
}
