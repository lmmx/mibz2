#![allow(dead_code)]
pub use mibz2::{bwt,runs,Kernels};
use bwt::{InverseBwt,Segmented};
use mibz2::stream::{Event,Scanner,SliceSource};
use std::fs::File;
use std::io::{BufWriter,Write};
#[path="frozen/research/astra-e/narrow-short/constructor_ahead.rs"] mod stock;
include!("gate.rs");

const CUTS:[usize;7]=[0,1,2,3,4,8,16];
const SLOTS:[usize;6]=[0,1,7,8,9,10];
type Writer=BufWriter<File>;
fn file(dir:&std::path::Path,name:&str,header:&str)->Writer{
    let mut f=BufWriter::new(File::create(dir.join(name)).unwrap());
    writeln!(f,"{header}").unwrap();f
}

#[derive(Default)]
struct PieceList { ends:Vec<u32>, ranks:Vec<u32>, freq:[u32;17], forced:u32, forced_at:Option<usize> }
impl PieceList {
    fn add(&mut self,rank:u32,width:u32,forced:bool) {
        if forced {assert!(self.forced_at.is_none());self.forced_at=Some(self.ends.len());}
        self.ranks.push(rank);
        self.ends.push(self.ends.last().copied().unwrap_or(0)+width);
        self.freq[width as usize]+=1;
        self.forced+=u32::from(forced);
    }
    fn width(&self,i:usize)->u32 {self.ends[i]-if i==0{0}else{self.ends[i-1]}}
}

#[derive(Default,Clone,Copy,PartialEq,Eq,Debug)]
struct Growth { batches:u64, first:u64, realloc:u64, exposed:u64 }
impl Growth {
    fn bump(&mut self,cap:usize,len:usize){
        if cap==0 {self.first+=1;}else{self.realloc+=1;self.exposed+=len as u64;}
    }
}
fn growth_cap(old:usize,need:usize)->usize {4.max(2*old).max(need)}
fn model(p:&PieceList,limit:usize,mut cap:usize,cut:usize,stats:&mut [Growth;17])->usize {
    let mut scanned=0;
    while scanned<limit{
        let i=scanned+p.ends[scanned..limit].partition_point(|&n|n as usize<=cap);
        if i==limit{break;}
        let width=p.width(i) as usize;
        let len=p.ends[i] as usize-width;
        let end=len+width;
        let g=&mut stats[width];
        g.batches+=1;
        if width<=cut {
            // The suffix starting at old capacity triggers geometric push growth,
            // possibly several times when a large width arrives at an empty queue.
            while end>cap {let full=cap;g.bump(cap,full);cap=growth_cap(cap,full+1);}
        }else{
            g.bump(cap,len);cap=growth_cap(cap,end);
        }
        scanned=i+1;
    }
    cap
}
fn native(p:&PieceList,v:&mut Vec<u32>,cut:usize,obs:&mut [Growth;17]) {
    v.clear();
    for (i,&rank) in p.ranks.iter().enumerate(){
        let w=p.width(i) as usize;
        let old=v.capacity();let g=&mut obs[w];
        if w<=cut {
            for k in 0..w{
                let b=v.capacity();let l=v.len();v.push(rank+k as u32);
                if v.capacity()!=b{g.bump(b,l);}
            }
        }else{
            let l=v.len();v.extend(rank..rank+w as u32);
            if v.capacity()!=old{g.bump(old,l);}
        }
        g.batches+=u64::from(v.capacity()!=old);
    }
}

struct Track { slots:usize, cut:usize, caps:Vec<[usize;256]>, g:[Growth;17] }
impl Track {
    fn new(slots:usize,cut:usize)->Self {Self{slots,cut,caps:vec![[0;256];slots.max(1)],g:[Growth::default();17]}}
}

fn marked_width(records:&[u16],p:u32)->u32{
    let(mut i,mut start)=(0,0);
    while i<records.len(){
        let c=records[i]>>8;let mut end=start;
        while i<records.len() && records[i]>>8==c{
            end+=u32::from(records[i]&255);i+=1;
        }
        if p<end{return end-start;}start=end;
    }
    unreachable!()
}

fn main(){
    let argv:Vec<_>=std::env::args().collect();
    let input=std::fs::read(&argv[1]).unwrap();
    let dir=std::path::Path::new(&argv[2]);
    std::fs::create_dir_all(dir).unwrap();
    let histhead=(1..=16).map(|i|format!("width{i}")).collect::<Vec<_>>().join(",");
    let mut freq=file(dir,"widths.csv",&format!("block,stream,label,rows,forced_singleton,{histhead}"));
    let mut caps=file(dir,"capacities.csv","block,label,slots,range,upto8,allpush");
    let mut history=file(dir,"blocks.csv","block,stream,slots,cut,slot,label_capacity_rows,all_slot_capacity_rows");
    let mut marks=file(dir,"marks.csv","block,stream,label,marked_width,marked_row,retained_source_rank,queue_len_before,slots,cut,capacity_before,capacity_after");
    let mut growth=file(dir,"growth.csv","slots,cut,width,growing_appends,first_allocations,reallocations,existing_u32_at_reallocation");
    let mut scan=Scanner::new(SliceSource(&input));let kernels=Kernels::detect();
    let mut worker=stock::Kernel::default();let mut refseg=Segmented::default();
    let(mut out,mut want)=(Vec::new(),Vec::new());
    let mut tracks:Vec<Track>=SLOTS.iter().flat_map(|&w|CUTS.iter().map(move|&c|Track::new(w,c))).collect();
    let mut serial:Vec<Vec<Vec<u32>>>=(0..3).map(|_|(0..256).map(|_|Vec::new()).collect()).collect();
    let(mut counted,mut coded,mut rows,mut pushes,mut splits)=(0,0,0,0,0);
    let mut checked=0u64;
    while let Event::Block{index,stream,symbols,..}=scan.next(Vec::new()).unwrap(){
        if !runs::takes_runs(&symbols){continue;}
        let mut r=runs::Runs::default();let n=runs::front(&symbols,&mut r,kernels).unwrap();let p=symbols.orig_ptr;
        if r.records().len()*20>=3*n && (n<65536 || !fits(r.records(),p)){continue;}
        assert!(refseg.invert_runs(&r,n,p,&mut want,kernels));
        let stockc=worker.run::<true,true,false>(r.records(),n,p,16,&mut out);
        assert_eq!(out,want);
        let mut list:Vec<PieceList>=(0..256).map(|_|PieceList::default()).collect();
        let marked=marked_width(r.records(),p);
        let long_mark=marked>16;
        let mut m=0;
        stock::kept(r.records(),p,|a,x,w|{
            list[a as usize].add(m,w,x==p && long_mark);
            m+=w;
        });
        assert_eq!(m as usize,stockc.retained);
        assert_eq!(list.iter().map(|b| b.forced).sum::<u32>(),u32::from(long_mark));
        for (a,b) in list.iter().enumerate(){
            if b.ends.is_empty(){continue;}
            write!(freq,"{index},{stream},{a},{},{}",b.ends.last().unwrap(),b.forced).unwrap();
            for f in &b.freq[1..]{write!(freq,",{f}").unwrap();}writeln!(freq).unwrap();
            pushes+=b.ranks.len();
        }
        for group in tracks.chunks_mut(CUTS.len()){
            let w=group[0].slots;
            let slot=if w==0{0}else{index as usize%w};
            for b in group.iter_mut(){if w==0 {b.caps[0]=[0;256];}}
            for (a,pp) in list.iter().enumerate(){
                for t in group.iter_mut(){
                    let old=t.caps[slot][a];
                    let mut g=[Growth::default();17];
                    let new=model(pp,pp.ends.len(),old,t.cut,&mut g);
                    if let Some(i)=pp.forced_at {
                        let mut scratch=[Growth::default();17];
                        let before=model(pp,i,old,t.cut,&mut scratch);
                        let len=pp.ends[i] as usize-1;
                        let after=if before==len {growth_cap(before,len+1)} else {before};
                        writeln!(marks,"{index},{stream},{a},{marked},{p},{},{len},{w},{},{before},{after}",pp.ranks[i],t.cut).unwrap();
                    }
                    for j in 1..=16 {
                        t.g[j].batches+=g[j].batches;t.g[j].first+=g[j].first;
                        t.g[j].realloc+=g[j].realloc;t.g[j].exposed+=g[j].exposed;
                    }
                    // Native Rust controls on every actual piece, fresh and serial. Verify
                    // all final queue bytes, so reordering equal spans is detected too.
                    if w<=1 && [0,8,16].contains(&t.cut) {
                        let mut fresh=Vec::new();
                        let v=if w==0{&mut fresh}else{&mut serial[if t.cut==0{0}else if t.cut==8{1}else{2}][a]};
                        assert_eq!(v.capacity(),old);
                        let mut obs=[Growth::default();17];native(pp,v,t.cut,&mut obs);
                        assert_eq!(obs,g);assert_eq!(v.capacity(),new);
                        assert_eq!(v.as_slice(),worker.queue(a).as_slice());
                        checked+=v.len() as u64;
                    }
                    if w==1 && t.cut==0 {assert_eq!(new,worker.queue(a).capacity());}
                    t.caps[slot][a]=new;
                }
                for i in 1..=4 {assert_eq!(group[i].caps[slot][a],group[0].caps[slot][a]);}
                if !pp.ends.is_empty(){
                    writeln!(caps,"{index},{a},{w},{},{},{}",group[0].caps[slot][a],group[5].caps[slot][a],group[6].caps[slot][a]).unwrap();
                }
            }
            for t in group.iter(){
                let bucket:usize=t.caps[slot].iter().sum();
                let all:usize=t.caps.iter().flat_map(|v|v.iter()).sum();
                writeln!(history,"{index},{stream},{w},{},{slot},{bucket},{all}",t.cut).unwrap();
            }
        }
        counted+=1;coded+=n;rows+=m as usize;splits+=usize::from(long_mark);
    }
    for t in &tracks{for w in 1..=16{
        let g=t.g[w];writeln!(growth,"{},{},{w},{},{},{},{}",t.slots,t.cut,g.batches,g.first,g.realloc,g.exposed).unwrap();
    }}
    assert_eq!((counted,coded,rows),(233usize,209697662usize,26567581usize));
    println!("{counted} admissions, {coded} marked output bytes, {rows} kept u32 entries, {pushes} kept append calls and {splits} forced primary singletons");
    println!("{checked} native entries compared directly against original queues (fresh and reused, thresholds 0,8,16), all {counted} marked outputs matched Segmented");
}
