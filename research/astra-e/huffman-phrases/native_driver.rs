use std::{fs,hint::black_box,time::Instant};
#[derive(Debug,PartialEq)]
struct Observed { symbols:Vec<u16>,crc:u32,p:u32,level:usize,alphabet:[u8;256],eob:u16 }
#[derive(Debug,PartialEq)]
enum ItemResult { Block(Observed), End(u32), Failure(ErrorKind,Vec<u16>) }
macro_rules! observe {
    ($s:expr)=>{Observed{symbols:$s.syms,crc:$s.crc,p:$s.orig_ptr,level:$s.max_len,alphabet:$s.seq_to_unseq,eob:$s.eob}};
}
macro_rules! one_item {
    ($m:ident,$data:expr,$start:expr)=>{{
        let mut br=bits::Bits::at($data,$start);
        let item=match $m::block::read_item(&mut br,9,Vec::new()) {
            Ok($m::block::Item::End(c))=>ItemResult::End(c),
            Ok($m::block::Item::Block(s))=>ItemResult::Block(observe!(s)),
            Err((k,v))=>ItemResult::Failure(k,v),
        };
        (br.bit_pos(),br.overran(),item)
    }}
}
fn check_item(data:&[u8],start:u64)->String {
    let stock=one_item!(stock,data,start);
    assert_eq!(stock,one_item!(triple,data,start));
    assert_eq!(stock,one_item!(quad,data,start));
    match stock.2 {ItemResult::Block(_) => format!("block overran={}",stock.1),
        ItemResult::End(_)=>"end".into(),ItemResult::Failure(e,_)=>format!("{:?} overran={}",e,stock.1)}
}
#[derive(Debug,PartialEq)]
struct Block { index:u64, stream:u64, bit:u64, end:u64,level:u32, item:Observed }

macro_rules! scanner {
    ($name:ident,$m:ident)=>{
        #[inline(never)]
        fn $name(input:&[u8], record:bool)->(u64,usize,usize,Vec<Block>) {
            let mut scanner=$m::stream::Scanner::new($m::stream::SliceSource(input));
            let mut reuse=Vec::new();
            let mut rows=Vec::new();
            let(mut check,mut count,mut total)=(0u64,0,0);
            loop {
                match scanner.next(reuse).unwrap() {
                    $m::stream::Event::Block{symbols,bit,end,index,stream,level}=>{
                        count+=1;total+=symbols.syms.len();
                        check=check.rotate_left(5)^end^(symbols.crc as u64).rotate_left(13)^symbols.syms.len() as u64;
                        black_box(symbols.syms.as_slice());
                        if record {
                            rows.push(Block{index,stream,bit,end,level,item:observe!(symbols)});
                            reuse=Vec::new();
                        } else {
                            reuse=symbols.syms;
                        }
                    }
                    $m::stream::Event::Done=>break
                }
            }
            (check,count,total,rows)
        }
    }
}
scanner!(stock_scan,stock);scanner!(twin_scan,twin);scanner!(triple_scan,triple);scanner!(quad_scan,quad);
type Scan=fn(&[u8],bool)->(u64,usize,usize,Vec<Block>);
const ARMS:[(&str,Scan);4]=[("stock",stock_scan),("twin",twin_scan),("triple",triple_scan),("quad",quad_scan)];

#[repr(C)] struct Timespec {s:i64,ns:i64}
unsafe extern "C" {fn clock_gettime(id:i32,t:*mut Timespec)->i32;}
fn thread_ns()->u64 {
    let mut t=Timespec{s:0,ns:0};
    assert_eq!(unsafe{clock_gettime(3,&mut t)},0);
    t.s as u64*1_000_000_000+t.ns as u64
}
fn main(){
    let args:Vec<String>=std::env::args().collect();
    if args[1]=="synthetic" {return stock::block::fixtures(args[2].parse().unwrap());}
    let data=fs::read(&args[2]).unwrap();
    if args[1]=="exact" {
        let expected=stock_scan(&data,true);
        for (name,scan) in &ARMS[1..] {
            let got=scan(&data,true);
            assert_eq!((&expected.0,&expected.1,&expected.2),(&got.0,&got.1,&got.2));
            assert!(expected.3==got.3);
            println!("{name} blocks {} symbols {} bit/metadata fold {:016x}; every symbol and complete block metadata checked",got.1,got.2,got.0);
        }
        return;
    }
    assert_eq!(args[1],"time");
    let rounds=args[3].parse::<usize>().unwrap();
    let expected=stock_scan(&data,false);
    assert_eq!((expected.1,expected.2),(431,24_979_299));
    for (_,scan) in ARMS {
        assert_eq!(scan(&data,false),expected);
    }
    println!("round,position,arm,thread_ns,wall_ns,blocks,symbols,metadata_fold");
    for r in 0..rounds {
        for pos in 0..4 {
            let j=if(r/4)%2==0{pos}else{3-pos};
            let (label,scan)=ARMS[(r+j)%4];
            let start=Instant::now();let cpu=thread_ns();
            let result=scan(black_box(&data),false);
            let ticks=thread_ns()-cpu;let elapsed=start.elapsed().as_nanos();
            assert_eq!(result,expected);
            println!("{r},{pos},{label},{ticks},{elapsed},{},{},{:016x}",result.1,result.2,result.0);
        }
    }
}
