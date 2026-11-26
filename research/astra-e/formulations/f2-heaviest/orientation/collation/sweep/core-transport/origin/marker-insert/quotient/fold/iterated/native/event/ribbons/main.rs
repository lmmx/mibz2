const ARMS:[&str;6]=["named","repeated","indexed","dense","flat","tree"];
#[derive(Default)]
struct Engine{named:control::Dispatch<2,2>,grammar:control::Grammar,event:control::Event, flat:control::RibbonFold<control::Flat>,tree:control::RibbonFold<control::Tree>,
    fallback:bwt::Segmented,ll:Vec<u8>,out:Vec<u8>}
#[derive(Default)]
struct Row{all:u64,first:u64,index:u64,contract:u64,emit:u64,old:u32,cap:usize,e:control::EventMeter,b:control::BandMeter}
impl Engine{
    #[inline(never)]
    fn run<const AUDIT:bool>(&mut self,x:&Input,arm:u8)->Row{
        use bwt::InverseBwt;
        let t=now();let mut r=Row::default();
        if let Some((mark,m))=self.named.first(&x.caps,&x.freq,x.n,x.p){
            r.first=now()-t;r.old=m;
            match arm {
                0=>{let t=now();self.named.finish(mark,m,x.n,x.p,&mut self.out);r.emit=now()-t;},
                1=>{let g=self.grammar.finish::<2,2,AUDIT>(&mut self.named,mark,m,x.n,x.p,&mut self.out);
                    r.index=g.init_ns;r.contract=g.folds_ns;r.emit=g.emit_ns;r.e.m=g;},
                2|3=>{let e=if arm==2{
                    self.event.finish::<false,AUDIT,2,2>(&mut self.named,mark,m,x.n,x.p,&mut self.out)
                }else{
                    self.event.finish::<true,AUDIT,2,2>(&mut self.named,mark,m,x.n,x.p,&mut self.out)
                };r.index=e.index_ns;r.contract=e.event_ns;r.emit=e.output_ns;r.e=e;},
                4|5=>{let mut g=if arm==4{
                    self.flat.finish::<AUDIT,2,2>(&mut self.named,mark,m,x.n,x.p,&mut self.out)
                }else{
                    self.tree.finish::<AUDIT,2,2>(&mut self.named,mark,m,x.n,x.p,&mut self.out)
                };r.index=g.e.index_ns;r.contract=g.e.event_ns;r.emit=g.e.output_ns;
                r.e=std::mem::take(&mut g.e);r.b=g;},
                _=>unreachable!(),
            }
        }else{
            self.ll.clear();for &r in &x.caps{self.ll.extend(std::iter::repeat_n((r>>8)as u8,(r&255)as usize));}
            self.fallback.invert(&self.ll,&x.freq,x.p,&mut self.out);r.e.m.cold=3;
        }
        r.all=now()-t;if AUDIT{r.cap=self.named.whole_capacity()+self.grammar.capacity()+self.event.capacity()+self.tree.capacity()+self.flat.capacity()
                             +self.out.capacity()+self.ll.capacity();}r
    }
}
fn small(){
    let mut w=Engine::default();let mut counts=[0u64;9];
    for n in 1..=9 {let alphabet=[b'"',b',',b'x'];let k=if n<=6{3usize}else{2};
        for mut seed in 0..k.pow(n){
            let mut x=Input::new(0,0,(0..n).map(|_|{let b=alphabet[seed%k];seed/=k;(u16::from(b)<<8)|1}).collect());
            let truth=Truth::new(&x);
            for p in 0..n{x.p=p;let y=truth.output(p);
                let mut reference=(0,0,0,0,0,0,0);for arm in [3,4,5]{let r=w.run::<true>(&x,arm);assert_eq!(w.out,y);
                    let check=(r.e.events,r.e.edits,r.e.m.moved,r.e.m.nodes,r.e.m.cold,r.e.pushes,r.e.pops);
                    if arm==3{reference=check;}else{assert_eq!(check,reference);}
                    if arm==3{counts[0]+=1;counts[1]+=u64::from(r.e.m.terminal);counts[2]+=u64::from(r.e.m.moved);
                        counts[3]+=u64::from(r.e.m.terminal && r.e.rows>1);
                        counts[4]+=u64::from(r.e.events);
                        if r.e.m.cold>0{counts[4+r.e.m.cold as usize]+=1;}
                        counts[8]+=u64::from(r.e.m.period!=0 && x.n%r.e.m.period!=0);}
                }
            }
        }
    }
    println!("marks,terminal,mark_transfers,multiple_live,brute_events,uncovered,stalled,no_quote,partial_period");
    println!("{}",counts.map(|x|x.to_string()).join(","));
}
fn hwm()->usize{
    std::fs::read_to_string("/proc/self/status").unwrap().lines().find(|s|s.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap().parse().unwrap()
}
fn census(path:&str,hashes:&str,arm:u8){
    let reference=if hashes=="-"{HashMap::new()}else{
        std::fs::read_to_string(hashes).unwrap().lines().skip(1).map(|s|{let(k,h)=s.split_once(',').unwrap();(k.parse::<u32>().unwrap(),h.parse::<u64>().unwrap())}).collect()
    };
    let mut f=BufReader::new(std::fs::File::open(path).unwrap());
    let mut w=Engine::default();
    println!("stream,arm,n,old,live,events,edits,queue_pushes,pops,peak_queue,nodes,moved,terminal,cold,period,leaf_copies,stack,aux_capacity,total_capacity,hwm,hash,initial_ribbons,final_ribbons,high,incl_tmp,source_visited,intersections,left_splits,right_splits,generated,coalesced,flat_suffix_moved,table_memory,first_runs,eligible_tests,owner_queries,fixed_queries");
    while let Some(x)=Input::read(&mut f){let r=w.run::<true>(&x,arm);let e=r.e;let b=r.b;
        let hash=w.out.iter().fold(0xcbf29ce484222325,|h,&b|(h^u64::from(b)).wrapping_mul(0x100000001b3));
        if hashes=="-"{let y=Truth::new(&x).output(x.p);assert_eq!(w.out,y);w.run::<false>(&x,arm);assert_eq!(w.out,y);}
        else{assert_eq!(hash,reference[&x.index]);}
        println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,ARMS[arm as usize],x.n,r.old,e.rows,
            e.events,e.edits,e.pushes,e.pops,e.peak_queue,e.m.nodes,e.m.moved,e.m.terminal,e.m.cold,e.m.period,
            e.m.leaf_visits,e.m.peak_stack,e.packed,r.cap,hwm(),hash,b.initial,b.live,b.high,b.incl_tmp,b.source,b.overlaps,b.left,b.right,b.generated,b.coalesced,b.shifted,b.table_peak,b.first_runs,b.tests,b.owner_searches,b.fixed_queries);
    }
}
fn paid(path:&str){
    let mut f=BufReader::new(std::fs::File::open(path).unwrap());let mut blocks=vec![];
    while let Some(x)=Input::read(&mut f){let y=Truth::new(&x).output(x.p);blocks.push((x,y));}
    assert_eq!(blocks.len(),10);
    let mut ws:Vec<_>=(0..6).map(|_|Engine::default()).collect();
    for(x,y)in &blocks{for(a,w)in ws.iter_mut().enumerate(){w.run::<false>(x,a as u8);assert_eq!(&w.out,y);}}
    println!("cluster,rep,block_position,stream,position,arm,first_ns,index_ns,contract_ns,emit_ns,total_ns");
    for rep in 0..36{let c=rep/6;
        for step in 0..10{let ix=(3*rep+if c%2==0{step}else{10-step})%10;let(x,y)=&blocks[ix];
            for(pos,j)in [0,1,5,2,4,3].into_iter().enumerate(){
                let arm=(rep%6+ix+c+if c%2==0{j}else{6-j})%6;let w=&mut ws[arm];
                let r=w.run::<false>(x,std::hint::black_box(arm as u8));assert_eq!(&w.out,y);
                println!("{c},{rep},{step},{},{pos},{},{},{},{},{},{}",x.index,ARMS[arm],r.first,r.index,r.contract,r.emit,r.all);
            }
        }
    }
}
fn main(){let a=std::env::args().collect::<Vec<_>>();
    match a.get(1).map(String::as_str){None=>small(),Some("price")=>paid(&a[2]),
        Some("census")=>census(&a[2],&a[3],a[4].parse().unwrap()),_=>unreachable!()}
}
