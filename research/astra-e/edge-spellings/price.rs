pub use mibz2::{bwt,runs,Kernels};
use bwt::{InverseBwt,Segmented};
use mibz2::stream::{Event,Scanner,SliceSource};

#[repr(C)] struct Timespec {sec:i64,ns:i64}
unsafe extern "C" {fn clock_gettime(id:i32,t:*mut Timespec)->i32;}
fn cpu()->u64 {
    let mut t=Timespec{sec:0,ns:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);
    t.sec as u64*1_000_000_000+t.ns as u64
}
#[derive(Default)] struct Worker {stock:stock::Kernel,handle:handle::Kernel,byte:byte::Kernel,out:Vec<u8>}
impl Worker {
    fn decode(&mut self,arm:usize,rec:&[u16],n:usize,p:u32,t:u32) {
        match arm {
            0|1=>{self.stock.run::<true,true,false>(rec,n,p,t,&mut self.out);}
            2=>{self.handle.run::<true,true,false>(rec,n,p,t,&mut self.out);}
            3=>{self.byte.run::<true,true,false>(rec,n,p,t,&mut self.out);}
            _=>unreachable!()
        }
    }
    fn storage(&self,arm:usize)->[usize;4] {
        match arm {
            0|1=>self.stock.measured_storage(),2=>self.handle.measured_storage(),_=>self.byte.measured_storage()
        }
    }
}
struct Block {index:u64,stream:u64,n:usize,r:runs::Runs,p:u32,dense:bool,want:Vec<u8>}

fn adversarial() {
    let mut w=Worker::default();
    let mut cases=0;
    let mut check=|l:&[u8],p:usize,t:u32|{
        let r=records(l);let want=reference(l,p);
        for a in [0,2,3]{
            w.decode(a,&r,l.len(),p as u32,t);
            assert_eq!(w.out,want);
        }
        cases+=1;
    };
    for n in 1..=9 {
        for mask in 0..1<<n {
            let l:Vec<_>=(0..n).map(|i|((mask>>i)&1) as u8).collect();
            for p in 0..n {check(&l,p,16);check(&l,p,1);}
        }
    }
    for seed in 0..96u32 {
        let mut z=seed+1;
        let mut l=Vec::new();
        for j in 0..120 {
            z=z.wrapping_mul(1664525).wrapping_add(1013904223);
            let a=((z>>24)%11) as u8;
            let width=[1,2,3,15,16,17,30,64,127,128,256][j%11];
            l.extend(std::iter::repeat_n(a,width));
        }
        for p in [0,1,16,17,127,l.len()/3,l.len()-1]{check(&l,p,16);}
    }
    for h in [21844,21845,100,21846] {
        let mut l=vec![2];
        for i in 0..h {l.extend(std::iter::repeat_n((i&1) as u8,17));}
        check(&l,0,16);
    }
    println!("{cases} adversarial marked comparisons each for frozen, handle and byte kernels, including periodic columns and 16-bit endpoint guard transitions");
}

fn main(){
    let args:Vec<_>=std::env::args().collect();
    if args[1]=="adversarial" {adversarial();return;}
    let timed=args[1]=="price";
    let raw=std::fs::read(&args[2]).unwrap();
    let oracle=std::fs::read(&args[3]).unwrap();
    let mut scan=Scanner::new(SliceSource(&raw));
    let k=Kernels::detect();
    let mut stock=mibz2::Scratch::<Segmented>::default();
    let (mut seg,mut plain,mut expanded)=(Segmented::default(),Vec::new(),Vec::new());
    let mut workers:[Worker;4]=std::array::from_fn(|_|Worker::default());
    let mut first=[false;12];
    let mut selected=Vec::new();
    let (mut consumed,mut marked,mut original)=(0,0,0);
    if !timed {println!("block,stream,n,arm,spelling,offset,edge,walk_vec_capacity");}
    while let Event::Block{index,stream,symbols,..}=scan.next(Vec::new()).unwrap() {
        original+=1;
        plain.clear();
        mibz2::decode_block(&symbols,k,&mut stock,&mut plain).unwrap();
        let expect=&oracle[consumed..consumed+plain.len()];
        assert_eq!(plain,expect);consumed+=plain.len();
        if !runs::takes_runs(&symbols){continue;}
        let mut r=runs::Runs::default();
        let n=runs::front(&symbols,&mut r,k).unwrap();
        let p=symbols.orig_ptr;
        let dense=r.records().len()*20>=3*n;
        if dense && (n<65536 || !fits(r.records(),p)){continue;}
        marked+=1;
        let mut want=Vec::new();
        assert!(seg.invert_runs(&r,n,p,&mut want,k));
        if !timed {
            for a in [0,2,3]{
                workers[a].decode(a,r.records(),n,p,16);
                assert_eq!(workers[a].out,want);
                expanded.clear();
                mibz2::rle::unrle(&workers[a].out,&mut expanded,k).unwrap();
                assert_eq!(mibz2::crc::crc32(&expanded,k),symbols.crc);
                assert_eq!(expanded,expect);
                let [s,o,e,c]=workers[a].storage(a);
                println!("{index},{stream},{n},{a},{s},{o},{e},{c}");
            }
        } else if n>=65536 && (!first[stream as usize] || dense){
            first[stream as usize]=true;
            selected.push(Block{index,stream,n,p,r,dense,want});
        }
    }
    assert_eq!((consumed,marked,original),(209700196,233,431));
    if !timed {eprintln!("431 full libbzip2 blocks and all 12 stream CRCs; 233 coded outputs, reconstructed block CRCs and libbzip2 outputs each for frozen, handle and byte kernels");return;}
    println!("round,block,stream,position,arm,dense,cpu_ns");
    assert_eq!(selected.iter().filter(|x|x.dense).count(),5);
    let rounds:usize=args[4].parse().unwrap();
    for round in 0..=rounds {
        for j in 0..selected.len() {
            let i=(if round%2==0 {j}else{selected.len()-j-1}+round*7)%selected.len();
            let b=&selected[i];let lap=round.saturating_sub(1);
            for pos in 0..4 {
                let a=(lap+i+if (lap/4+i)%2==0{pos}else{4-pos})%4;
                let t=cpu();
                assert!(b.r.records().len()*20<3*b.n || (b.n>=65536 && fits(b.r.records(),b.p)));
                workers[a].decode(a,b.r.records(),b.n,b.p,16);
                let elapsed=cpu()-t;
                assert_eq!(workers[a].out,b.want);
                if round>0 {println!("{round},{},{},{pos},{a},{},{elapsed}",b.index,b.stream,usize::from(b.dense));}
            }
        }
    }
}
