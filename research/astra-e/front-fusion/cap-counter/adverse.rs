#![allow(dead_code)]
pub use mibz2::{bwt,runs,Kernels};
include!("arms.inc.rs");
type Strip = (u32,u32,u32,u8);
#[derive(Debug)]
struct State {
    all: [u32;256],
    thin: [u32;256],
    m: u32,
    mark: u32,
    strips: Vec<Strip>,
}
thread_local! { static EXPECTED:std::cell::RefCell<Option<State>>=const{std::cell::RefCell::new(None)}; }
fn take_scan(all:&[u32;256],thin:&[u32;256],m:u32,mark:u32,strips:Vec<Strip>){
    EXPECTED.with(|cell|{
        let borrowed=cell.borrow();
        let d=borrowed.as_ref().unwrap();
        assert_eq!((all,thin,m,mark),(&d.all,&d.thin,d.m,d.mark));
        assert_eq!(strips,d.strips);
    });
}
fn expected(column:&[u8],mark:usize,limit:usize)->(State,Vec<u16>,Vec<u8>){
    let mut o=State{all:[0;256],thin:[0;256],m:0,mark:0,strips:Vec::new()};
    let mut recs=Vec::new();
    let mut row=0;
    while row<column.len(){
        let c=column[row];
        let mut end=row+1;
        while end<column.len()&&column[end]==c {end+=1;}
        let thick=end-row>limit;
        let mut left=end-row;
        while left>0 {let w=left.min(127);recs.push((u16::from(c)<<8)|w as u16);left-=w;}
        let mut x=row;
        while x<end{
            let keep=!thick||x==mark;
            let mut z=x+1;
            while z<end && (!thick || z==mark)==keep {z+=1;}
            let w=(z-x) as u32;
            let a=usize::from(c);
            if keep {
                if (x..z).contains(&mark){o.mark=o.m+(mark-x) as u32;}
                o.thin[a]+=w;
                o.m+=w;
            } else {
                o.strips.push((x as u32,o.all[a],w,c));
            }
            o.all[a]+=w;
            x=z;
        }
        row=end;
    }
    let mut cursor=[0;256];
    let mut start=0;
    for (base,count) in cursor.iter_mut().zip(o.all){*base=start;start+=count as usize;}
    let mut psi=vec![0;column.len()];
    for (r,&c) in column.iter().enumerate(){
        psi[cursor[c as usize]]=r;
        cursor[c as usize]+=1;
    }
    let mut x=mark;
    let out=(0..column.len()).map(|_|{x=psi[x];column[x]}).collect();
    (o,recs,out)
}
fn stock_rejections()->usize{
    use bwt::InverseBwt;
    use mibz2::block::Symbols;
    let k=Kernels::detect();
    let mut checked=0;
    for n in [4096usize,300000] {
        let mut syms=vec![2;n];syms.push(3);
        let mut r=runs::Runs::default();
        let b=Symbols{syms,crc:0,orig_ptr:0,seq_to_unseq:std::array::from_fn(|i|i as u8),eob:3,max_len:900000};
        assert_eq!(runs::front(&b,&mut r,k).unwrap(),n);
        assert_eq!(r.records().len(),n);
        for p in [0,n/2,n-1]{
            let mut stock=bwt::LazyCells::default();
            let mut reference=vec![41,0,17];
            let state=stock.invert_runs(&r,n,p as u32,&mut reference,k);
            macro_rules! check_stock {($arm:ty)=>{{
                let mut out=vec![41,0,17];
                let mut arm=<$arm>::default();
                assert_eq!(arm.invert_runs(&r,n,p as u32,&mut out,k),state);
                assert_eq!(out,reference);
            }}}
            check_stock!(edge::Candidate);check_stock!(counter_edge::Candidate);
            check_stock!(scalar_edge::Candidate);check_stock!(both::Candidate);
            checked+=1;
        }
    }
    checked
}
fn main(){
    let(mut a,mut b,mut c,mut d)=(edge::kernel::Kernel::default(),counter_edge::kernel::Kernel::default(),
                                 scalar_edge::kernel::Kernel::default(),both::kernel::Kernel::default());
    let mut out=Vec::new();
    let(mut cases,mut wide_endpoint,mut wide_edge,mut nonqueue)=(0,0,0,0);
    let mut verify=|column:&[u8],p:usize,w:usize,extra:bool|{
        let (state,recs,truth)=expected(column,p,w);
        EXPECTED.with(|v|v.replace(Some(state)));
        let n=column.len();
        let original=a.run::<true,true,false>(&recs,n,p as u32,w as u32,&mut out);
        assert_eq!(truth,out);
        let original_text=format!("{original:?}");
        macro_rules! verify_arm {($k:expr,$queue:literal)=>{
            {
                let got=$k.run::<true,$queue,false>(&recs,n,p as u32,w as u32,&mut out);
                assert_eq!(truth,out);assert_eq!(original_text,format!("{got:?}"));
            }
        }}
        verify_arm!(b,true);verify_arm!(c,true);verify_arm!(d,true);
        if extra{verify_arm!(a,false);verify_arm!(b,false);verify_arm!(c,false);verify_arm!(d,false);nonqueue+=1;}
        cases+=1;
        wide_endpoint+=usize::from(original.thick>(u16::MAX as usize-3)/3);
        wide_edge+=usize::from(original.table_bytes==8*original.retained);
    };
    for n in 1..=8 {
        for bits in 0..1<<n {
            let l:Vec<u8>=(0..n).map(|i|((bits>>i)&1) as u8).collect();
            for p in 0..n {for w in [1,2,16]{verify(&l,p,w,w==2);}}
        }
    }
    for length in [15,16,17,126,127,128,253,254,255,501]{
        let mut l=vec![71;length];l.extend_from_slice(&[10,10,10,71,71,71,12]);
        for p in [0,1,length/2,length-1,length,l.len()-1]{verify(&l,p,16,true);}
    }
    for h in [21_844,21_845,100,21_846]{
        let mut l=vec![2];
        for j in 0..h{l.extend(std::iter::repeat_n((j&1) as u8,17));}
        verify(&l,0,16,true);
    }
    // Many long thin runs and irregular thick strips force a larger packed label space.
    let mut state=29u32;
    let mut l=Vec::new();
    for x in 0..27_000{
        state^=state<<13;state^=state>>17;state^=state<<5;
        l.extend(std::iter::repeat_n((state%126) as u8,16));
        l.extend(std::iter::repeat_n((128+(state>>8)%125) as u8,17));
        if x==600 {l.push(255);}
    }
    verify(&l,1249,16,true);
    assert!(wide_endpoint>=2 && wide_edge>=1);
    let stock=stock_rejections();
    println!("{stock} dense stock rejections matched all four original return statuses and output buffers.");
    println!("{cases} independent marked constructions; four queued arms; {nonqueue} also used all four nonqueue specializations; {wide_endpoint} wide endpoint; {wide_edge} 8-byte packed edges.");
}
