const ARMS:[&str;4]=["named","indexed","dense","owner"];
#[derive(Default)]
struct Engine{named:control::Dispatch<2,2>,grammar:control::Grammar,event:control::Event,
    fallback:bwt::Segmented,ll:Vec<u8>,out:Vec<u8>}
#[derive(Default)]
struct Row{all:u64,first:u64,index:u64,contract:u64,emit:u64,old:u32,cap:usize,e:control::EventMeter}
impl Engine{
    #[inline(never)]
    fn run<const AUDIT:bool>(&mut self,x:&Input,arm:u8)->Row{
        use bwt::InverseBwt;
        let t=now();let mut r=Row::default();
        if let Some((mark,m))=self.named.first(&x.caps,&x.freq,x.n,x.p){
            r.first=now()-t;r.old=m;
            match arm {
                0=>{let t=now();self.named.finish(mark,m,x.n,x.p,&mut self.out);r.emit=now()-t;},
                1|2|3=>{let e=match arm {
                    1=>self.event.finish::<false,AUDIT,false,2,2>(&mut self.named,mark,m,x.n,x.p,&mut self.out),
                    2=>self.event.finish::<true,AUDIT,false,2,2>(&mut self.named,mark,m,x.n,x.p,&mut self.out),
                    3=>self.event.finish::<true,AUDIT,true,2,2>(&mut self.named,mark,m,x.n,x.p,&mut self.out),
                    _=>unreachable!(),
                };r.index=e.index_ns;r.contract=e.event_ns;r.emit=e.output_ns;r.e=e;},
                _=>unreachable!(),
            }
        }else{
            self.ll.clear();for &r in &x.caps{self.ll.extend(std::iter::repeat_n((r>>8)as u8,(r&255)as usize));}
            self.fallback.invert(&self.ll,&x.freq,x.p,&mut self.out);r.e.m.cold=3;
        }
        r.all=now()-t;r.cap=self.named.whole_capacity()+self.grammar.capacity()+self.event.capacity()
                             +self.out.capacity()+self.ll.capacity();r
    }
}
fn small(){
    eprintln!("synthetic cases hits partial_hits partial_misses copied_bytes: {:?}", control::ownership_fixtures());
    let mut w=Engine::default();let mut counts=[0u64;9];
    for n in 1..=9 {let alphabet=[b'"',b',',b'x'];let k=if n<=6{3usize}else{2};
        for mut seed in 0..k.pow(n){
            let mut x=Input::new(0,0,(0..n).map(|_|{let b=alphabet[seed%k];seed/=k;(u16::from(b)<<8)|1}).collect());
            let truth=Truth::new(&x);
            for p in 0..n{x.p=p;let y=truth.output(p);
                for arm in [0,1,2,3]{let r=w.run::<true>(&x,arm);assert_eq!(w.out,y);
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
    println!("stream,arm,n,old,live,events,edits,queue_pushes,pops,peak_queue,nodes,moved,terminal,cold,period,leaf_copies,stack,aux_capacity,total_capacity,hwm,hash,owner_slots,owner_bytes,published,hits,partial_hits,partial_misses,copied_bytes,leaf_bytes,visits");
    while let Some(x)=Input::read(&mut f){let r=w.run::<true>(&x,arm);let e=r.e;let o=e.ownership;
        let hash=w.out.iter().fold(0xcbf29ce484222325,|h,&b|(h^u64::from(b)).wrapping_mul(0x100000001b3));
        if hashes=="-"{let y=Truth::new(&x).output(x.p);assert_eq!(w.out,y);w.run::<false>(&x,arm);assert_eq!(w.out,y);}
        else{assert_eq!(hash,reference[&x.index]);}
        println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,ARMS[arm as usize],x.n,r.old,e.rows,
            e.events,e.edits,e.pushes,e.pops,e.peak_queue,e.m.nodes,e.m.moved,e.m.terminal,e.m.cold,e.m.period,
            e.m.leaf_visits,e.m.peak_stack,e.packed,r.cap,hwm(),hash,o.slots,o.bytes,o.published,o.hits,o.partial_hits,o.partial_misses,o.copied,o.leaf_bytes,o.visits);
    }
}
fn paid(path:&str){
    let mut f=BufReader::new(std::fs::File::open(path).unwrap());let mut blocks=vec![];
    while let Some(x)=Input::read(&mut f){let y=Truth::new(&x).output(x.p);blocks.push((x,y));}
    assert_eq!(blocks.len(),10);
    let mut ws:Vec<_>=(0..4).map(|_|Engine::default()).collect();
    for(x,y)in &blocks{for(a,w)in ws.iter_mut().enumerate(){w.run::<false>(x,a as u8);assert_eq!(&w.out,y);}}
    println!("cluster,rep,block_position,stream,position,arm,first_ns,index_ns,contract_ns,emit_ns,total_ns");
    for rep in 0..32{let c=rep/4;
        for step in 0..10{let ix=(3*rep+if c%2==0{step}else{10-step})%10;let(x,y)=&blocks[ix];
            for(pos,j)in [0,1,3,2].into_iter().enumerate(){
                let arm=(rep%4+ix+c+if c%2==0{j}else{4-j})%4;let w=&mut ws[arm];
                let r=w.run::<false>(x,std::hint::black_box(arm as u8));assert_eq!(&w.out,y);
                println!("{c},{rep},{step},{},{pos},{},{},{},{},{},{}",x.index,ARMS[arm],r.first,r.index,r.contract,r.emit,r.all);
            }
        }
    }
}
fn resident(path:&str){
    let mut f=BufReader::new(std::fs::File::open(path).unwrap());
    println!("cluster,rep,stream,position,arm,emit_ns");
    let mut block=0;
    while let Some(x)=Input::read(&mut f){
        let y=Truth::new(&x).output(x.p);let mut w=Engine::default();let g=w.run::<false>(&x,2);
        assert!(g.e.m.terminal);assert_eq!(w.out,y);
        let mut outs:[Vec<u8>;4]=std::array::from_fn(|_|Vec::default());
        let names=["plain_a","owner_a","plain_b","owner_b"];
        for rep in 0..33{
            let measured=rep>0;let lap=if measured{rep-1}else{0};let c=lap/4;
            for(p,j)in [0,1,3,2].into_iter().enumerate(){
                let a=(lap%4+block+c+if c%2==0{j}else{4-j})%4;
                let out=&mut outs[a];let t=now();
                if std::hint::black_box(a%2)==1 {w.event.again::<true,false,2,2>(&w.named,out,&mut control::Meter::default());}
                else {w.event.again::<false,false,2,2>(&w.named,out,&mut control::Meter::default());}
                let ns=now()-t;assert_eq!(out,&y);
                if measured{println!("{c},{lap},{},{p},{},{ns}",x.index,names[a]);}
            }
        }
        block+=1;
    }
    assert_eq!(block,10);
}
fn main(){let a=std::env::args().collect::<Vec<_>>();
    match a.get(1).map(String::as_str){None=>small(),Some("price")=>paid(&a[2]),Some("resident")=>resident(&a[2]),
        Some("census")=>census(&a[2],&a[3],a[4].parse().unwrap()),_=>unreachable!()}
}
