const ARMS:[&str;4]=["named_a","named_b","dag","joint"];
#[derive(Default)]
struct Engine { named:control::Dispatch<2,2>,grammar:control::Grammar,
    joint:joint::kernel::Kernel, fallback:bwt::Segmented, ll:Vec<u8>, out:Vec<u8> }
#[derive(Default)]
struct Counts{t:u64,stage:[u64;3],m:control::Meter,capacity:usize,initial:u32}
impl Engine{
    #[inline(never)]
    fn run<const AUDIT:bool>(&mut self,x:&Input,mode:u8)->Counts{
        use bwt::InverseBwt;
        let start=now();let mut g=Counts::default();
        if mode==3{
            self.joint.run::<true,true,false>(&x.caps,x.n as usize,x.p,16,&mut self.out);
        }else if let Some((mark,m))=self.named.first(&x.caps,&x.freq,x.n,x.p){
            g.stage=self.named.phases;g.initial=m;
            if mode==2{
                g.m=self.grammar.finish::<2,2,AUDIT>(&mut self.named,mark,m,x.n,x.p,&mut self.out);
            }else{self.named.finish(mark,m,x.n,x.p,&mut self.out);}
        }else{
            self.ll.clear();
            for &r in &x.caps { self.ll.extend(std::iter::repeat_n((r>>8)as u8,(r&255)as usize)); }
            self.fallback.invert(&self.ll,&x.freq,x.p,&mut self.out);g.m.cold=3;
        }
        g.t=now()-start;
        g.capacity=if mode==3{0}else{self.named.whole_capacity()+self.grammar.capacity()+
            self.out.capacity()+self.ll.capacity()};
        g
    }
}
fn small(){
    let mut w=Engine::default();let mut cnt=[0u64;8]; let alphabet=[b'"',b',',b'x'];
    for n in 1..=9 {
        let k=if n<=6{3usize}else{2};
        for mut word in 0..k.pow(n){
            let mut x=Input::new(0,0,(0..n).map(|_|{let v=alphabet[word%k];word/=k;(u16::from(v)<<8)|1}).collect());
            let truth=Truth::new(&x);
            for p in 0..n{x.p=p;let g=w.run::<true>(&x,2);assert_eq!(w.out,truth.output(p));
                cnt[0]+=1;cnt[1]+=u64::from(g.m.terminal);cnt[2]+=u64::from(g.m.moved);
                cnt[3]+=u64::from(g.m.terminal&&g.m.rows>1);cnt[4]+=u64::from(g.m.period>0&&x.n%g.m.period!=0);
                if g.m.cold!=0{cnt[4+g.m.cold as usize]+=1;}
            }
        }
    }
    println!("marked_cases,terminal,moved_across_rounds,self_loop_inside_larger_graph,partial_period,uncovered,uncollapsed,quote_absent");
    println!("{}",cnt.map(|x|x.to_string()).join(","));
}
fn hwm()->usize{
    std::fs::read_to_string("/proc/self/status").unwrap().lines().find(|s|s.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap().parse().unwrap()
}
fn hash(s:&[u8])->u64{s.iter().fold(0xcbf29ce484222325,|h,b|(h^u64::from(*b)).wrapping_mul(0x100000001b3))}
fn census(path:&str,expected:&str,a:usize){
    let reference=if expected=="-" {HashMap::new()} else {std::fs::read_to_string(expected).unwrap().lines().skip(1).map(|s|{let (k,h)=s.split_once(',').unwrap();(k.parse::<u32>().unwrap(),h.parse::<u64>().unwrap())}).collect::<HashMap<_,_>>()};
    let mut f=BufReader::new(std::fs::File::open(path).unwrap());
    let mut w=Engine::default();
    println!("stream,arm,n,old_rows,old_classes,rounds,final_rows,final_runs,final_classes,dag_nodes,paid_runs,paid_classes,removed,moved,initial_skip,terminal,cold,period,expanded_internal,leaf_copies,stack,grammar_and_fold_bytes,total_capacity,hwm_kib,hash");
    while let Some(x)=Input::read(&mut f){let g=w.run::<true>(&x,if a==1{0}else{a as u8});
        let h=hash(&w.out);if expected=="-" {let truth=Truth::new(&x).output(x.p); assert_eq!(w.out,truth);w.run::<false>(&x,if a==1{0}else{a as u8});assert_eq!(w.out,truth);}else{assert_eq!(h,reference[&x.index]);}let m=g.m;
        println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{h}",
            x.index,ARMS[a],x.n,g.initial,m.initial,m.rounds,m.rows,m.final_runs,m.types,m.nodes,
            m.paid_runs,m.paid_classes,m.removed,m.moved,m.transfer,m.terminal,m.cold,m.period,
            m.expansions,m.leaf_visits,m.peak_stack,m.extra,g.capacity,hwm());
    }
}
fn paid(path:&str){
    let mut f=BufReader::new(std::fs::File::open(path).unwrap());
    let mut blocks=vec![];while let Some(x)=Input::read(&mut f){
        let y=Truth::new(&x).output(x.p); blocks.push((x,y));
    }
    assert_eq!(blocks.len(),10);
    let mut ws:Vec<_>=(0..4).map(|_|Engine::default()).collect();let modes=[0,0,2,3];
    for(x,y)in &blocks{for(w,&mode)in ws.iter_mut().zip(&modes){w.run::<false>(x,mode);assert_eq!(&w.out,y);}}
    println!("cluster,rep,block_position,stream,position,arm,first_setup_mark,first_discover,first_merge,dag_init,folds,emit,total_ns");
    for rep in 0..32{
        let c=rep/4;
        for step in 0..10{
            let ix=(3*rep+if c%2==0{step}else{10-step})%10;
            let(x,y)=&blocks[ix];
            for(pos,j)in [0,1,3,2].into_iter().enumerate(){
                let arm=(rep%4+ix+c+if c%2==0{j}else{4-j})%4;
                let w=&mut ws[arm];let g=w.run::<false>(x,std::hint::black_box(modes[arm]));assert_eq!(&w.out,y);
                let t=g.stage;let m=g.m;
                println!("{c},{rep},{step},{},{pos},{},{},{},{},{},{},{},{}",x.index,ARMS[arm],t[0],t[1],t[2],m.init_ns,m.folds_ns,m.emit_ns,g.t);
            }
        }
    }
}
fn main(){let a=std::env::args().collect::<Vec<_>>();
    match a.get(1).map(String::as_str){None=>small(),Some("paid")=>paid(&a[2]),
        Some("census")=>census(&a[2],&a[3],a[4].parse().unwrap()),_=>unreachable!()}
}
