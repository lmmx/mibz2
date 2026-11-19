use mibz2::{block::Symbols,runs,stream::{Event,Scanner,SliceSource}};
mod paid; mod replay; mod topology;
struct Runs {runs:Vec<u16>,counts:[u32;256],per:[u32;256],lit:Vec<u64>,one:Vec<u64>}
impl Default for Runs{fn default()->Self{Self{runs:Vec::new(),counts:[0;256],per:[0;256],lit:Vec::new(),one:Vec::new()}}}
#[repr(C)]struct Time{i:i64,n:i64}
unsafe extern "C"{fn clock_gettime(c:i32,t:*mut Time)->i32;}
fn cpu()->i64{let mut t=Time{i:0,n:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);t.i*1_000_000_000+t.n}
fn main(){
    let data=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();let mut scan=Scanner::new(SliceSource(&data));
    let mut blocks=Vec::new();let mut scratch=Runs::default();let mut scalar=runs::Runs::default();
    type Front=unsafe fn(&Symbols,&mut Runs,&[u8])->Option<usize>;
    let arms:[(&str,Front);3]=[("replay",replay::front_avx2),("topology",topology::front_avx2),("paid",paid::front_avx2)];
    while let Event::Block{symbols:b,..}=scan.next(Vec::new()).unwrap(){
        if !runs::takes_runs(&b){continue;}
        let mut list=b.seq_to_unseq;let mut gold=vec![0;b.syms.len()];
        for(i,&s)in b.syms.iter().enumerate(){
            if s<=1{continue;}if s==b.eob{break;}
            let rank=usize::from(s-1).min(255);let c=list[rank];list.copy_within(..rank,1);list[0]=c;gold[i]=c;
        }
        let n=runs::front_scalar(&b,&mut scalar).unwrap();
        for (_,f) in arms{
            assert_eq!(unsafe{f(&b,&mut scratch,&gold)},Some(n));
            assert_eq!(scratch.runs,scalar.records());
            let mut counts=[0;256];let mut per=[0;256];
            for &v in scalar.records(){counts[(v>>8)as usize]+=u32::from(v&255);per[(v>>8)as usize]+=1;}
            assert_eq!(counts,scratch.counts);assert_eq!(per,scratch.per);
        }
        blocks.push((b,gold));
    }
    eprintln!("All {} record routes matched capped records, lengths and both counter arrays; supplied gold bytes {}",blocks.len(),blocks.iter().map(|q|q.1.len()).sum::<usize>());
    let ord=[[0,1,2],[1,2,0],[2,0,1],[2,1,0],[1,0,2],[0,2,1]];
    println!("round,position,arm,ns,records");
    for round in 0usize..=24{for (pos,&a)in ord[round.saturating_sub(1)%6].iter().enumerate(){
        let (name,f)=arms[a];let t=cpu();let mut seen=0;
        for(s,gold)in &blocks{std::hint::black_box(unsafe{f(s,&mut scratch,gold)}.unwrap());seen+=scratch.runs.len();std::hint::black_box(&scratch);}
        let ns=cpu()-t;assert_eq!(seen,16326954);
        if round>0{println!("{round},{pos},{name},{ns},{seen}");}
    }}
}
