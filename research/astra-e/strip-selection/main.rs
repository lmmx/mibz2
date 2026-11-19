pub use mibz2::{bwt,runs,Kernels};
use mibz2::stream::{Scanner,SliceSource,Event};
use bwt::{InverseBwt,Segmented};
#[repr(C)] struct Timespec {sec:i64,ns:i64}
unsafe extern "C" {fn clock_gettime(id:i32,t:*mut Timespec)->i32;}
fn clock()->u64 {let mut t=Timespec{sec:0,ns:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);t.sec as u64*1_000_000_000+t.ns as u64}

struct Block { index:u64,stream:u64,n:usize,p:u32,r:runs::Runs,want:Vec<u8> }
#[derive(Default)]
struct Study {base:stock::Kernel,others:[changed::Kernel;4],out:[Vec<u8>;5]}
struct Row {q:usize,counts:[usize;10],cpu:[u64;2],storage:[usize;2]}

impl Study {
    fn call(&mut self,b:&Block,mode:usize,meter:bool)->Row {
        let o=&mut self.out[mode];let rec=b.r.records();
        let t=clock();
        // Keep the champion's admission including its dense certificate inside a paid call.
        assert!(rec.len()*20<b.n*3 || (b.n>=65536 && fits(rec,b.p)));
        let(q,cpu,storage,g,w)=if mode==0 {
            let c=self.base.run::<true,true,false>(rec,b.n,b.p,16,o);
            let end=clock();
            (0,[0,end-t],self.base.capacities(),
              [c.retained,c.thick,c.open_nodes,c.steps,c.rules,c.bands,c.traced,c.words,c.copies,c.table_bytes],
              if meter {self.base.walk_counts()}else{[0;4]})
        } else {
            let k=&mut self.others[mode-1];
            let (c,parts,q)=match mode {
                1=>k.run_selected::<1>(rec,b.n,b.p,o),2=>k.run_selected::<2>(rec,b.n,b.p,o),
                3=>k.run_selected::<3>(rec,b.n,b.p,o),_=>k.run_selected::<4>(rec,b.n,b.p,o)
            };
            let end=clock();
            (q,[parts[0],end-t],k.capacities(),[c.retained,c.thick,c.open_nodes,c.steps,c.rules,c.bands,c.traced,c.words,c.copies,c.table_bytes],
             if meter {k.walk_counts()}else{[0;4]})
        };
        assert_eq!(o,&b.want);
        if meter {
            assert_eq!(w[0],g[0]); assert_eq!(w[1],g[8]);
            println!("{},{},{},{mode},{q},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",b.stream,b.index,b.n,
                g[0],g[1],g[2],g[3],g[4],g[5],g[6],g[7],g[8],g[9],w[2],w[3],storage[0],storage[1]);
        }
        Row{q,counts:g,cpu,storage}
    }
}
fn main() {
    let args:Vec<_>=std::env::args().collect();
    let mode=args[1].as_str();
    let train=mode=="train";let timed=mode=="price";
    let winner=args.get(3).map_or(2,|x|x.parse::<usize>().unwrap());
    assert!((2..=4).contains(&winner));
    let compressed=std::fs::read(&args[2]).unwrap();
    let mut sc=Scanner::new(SliceSource(&compressed));
    let(k,mut seg)=(Kernels::detect(),Segmented::default());
    let mut state=Study::default();let mut blocks=Vec::new();
    let mut counts=[0usize;12];
    if !timed {println!("stream,block,n,arm,q,retained,strips,nodes,steps,rules,bands,dictionary_bytes,words,word_visits,edge_bytes,same_line,same_4k,edge_cap,selector_cap");}
    while let Event::Block{index,stream,symbols,..}=sc.next(Vec::new()).unwrap(){
        if (stream%3==0)!=train {continue}
        let mut r=runs::Runs::default();let n=runs::front(&symbols,&mut r,k).unwrap();
        if n<65536 || !runs::takes_runs(&symbols) || (r.records().len()*20>=3*n && !fits(r.records(),symbols.orig_ptr)) {continue}
        let ord=counts[stream as usize];counts[stream as usize]+=1;
        if timed && ord>=2 {continue}
        let mut want=vec![];assert!(seg.invert_runs(&r,n,symbols.orig_ptr,&mut want,k));
        let block=Block{index,stream,n,p:symbols.orig_ptr,r,want};
        if timed {blocks.push(block);continue}
        let mut m=0;let mut h=0; let mut intervention=None;
        let selected=[0,1,winner];
        for &arm in if train {&[0,1,2,3,4][..]}else{&selected[..]} {
            let row=state.call(&block,arm,true);
            if arm==0 {m=row.counts[0];h=row.counts[1];}
            else {
                assert_eq!((row.counts[0],row.counts[1]),(m+row.q,h));
                if let Some(q)=intervention{assert_eq!(row.q,q);}else{intervention=Some(row.q);}
            }
        }
    }
    if !timed {return}
    assert_eq!(blocks.len(),16);
    println!("round,block,stream,position,arm,classify_ns,complete_ns");
    let permutations=[[0,1,winner],[1,winner,0],[winner,0,1],[winner,1,0],[0,winner,1],[1,0,winner]];
    for round in 0..=24 {
        for off in 0..blocks.len() {
            let i=(if round%2==0 {off}else{blocks.len()-1-off}+round*3)%blocks.len();
            let b=&blocks[i];
            for(pos,&arm) in permutations[(round+i)%6].iter().enumerate(){
                let v=state.call(b,arm,false);
                if round>0 {println!("{round},{},{},{pos},{arm},{},{}",b.index,b.stream,v.cpu[0],v.cpu[1]);}
            }
        }
    }
}
