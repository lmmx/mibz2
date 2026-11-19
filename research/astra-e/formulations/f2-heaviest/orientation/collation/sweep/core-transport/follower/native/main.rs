#[derive(Default)] struct Subject {
    physical:follower::Dispatch<2,2>,
    original:projected::Dispatch<2,2>,
    named:shared::Dispatch<2,2>,
    quote:control::Dispatch<2,2>,
    out:Vec<u8>,
}
struct Receipt {
    obj:Option<Obj>,mark:Option<(u32,u32)>,second:bool,metrics:[u32;8],phases:[u64;3],inner:[u64;4],
    cap:usize,first:u64,total:u64,publish:u64,
}
impl Subject {
    fn run(&mut self,x:&Input,u:(u32,u32),arm:i32,snapshot:bool)->Option<Receipt>{
        let t0=now();
        macro_rules! finish {($c:expr,$mark:expr,$m:expr,$missing:expr,$metrics:expr,$publish:expr)=>{{
            let t1=now();
            let obj=snapshot.then(||$c.object());
            let second=$c.finish($mark,$m,x.n,$missing,&mut self.out);
            let total=now()-t0;
            Receipt{obj,mark:$mark,second,metrics:$metrics,phases:$c.phases,inner:$c.second_phases(),
                    cap:$c.whole_capacity()+self.out.capacity(),first:t1-t0,total,publish:$publish}
        }}}
        if arm== -2 {
            let (a,m)=self.quote.first(&x.caps,&x.freq,x.n,x.p)?;
            return Some(finish!(self.quote,a,m,x.p,[0;8],0));
        }
        if arm== -1 {
            let (b,mono)=x.bucket(u);
            let (a,m)=if mono{self.named.first::<true>(&x.caps,&x.freq,x.n,x.p,u,b)}
                      else{self.named.first::<false>(&x.caps,&x.freq,x.n,x.p,u,b)}?;
            return Some(finish!(self.named,a,m,x.p,[0;8],0));
        }
        if arm<16 {
            let (a,m,q)=self.original.first(&x.caps,&x.freq,x.n,x.p,u,arm as usize)?;
            return Some(finish!(self.original,a,m,q,self.original.measures,self.original.compose_ns));
        }
        let (a,m,q)=self.physical.first(&x.caps,&x.freq,x.n,x.p,u,(arm-16) as usize)?;
        Some(finish!(self.physical,a,m,q,self.physical.measures,self.physical.compose_ns))
    }
}
fn name(arm:i32)->String {
    match arm {-2=>"compact_quote".into(),-1=>"compact_frequent".into(),0..16=>format!("original_h{arm}"),_=>format!("physical_h{}",arm-16)}
}
fn small(){
    let mut cases=0;let mut disjoint=0;let mut fallback=0;let mut sub=Subject::default();
    for (alphabet,max_n) in [(2,7),(3,5)] {
        for n in 1..=max_n {for seed in 0..u32::pow(alphabet,n) {
            let mut z=seed;let caps=(0..n).map(|_|{let c=z%alphabet;z/=alphabet;((c<<8)|1) as u16}).collect();
            let mut x=Input::new(0,0,caps);let dense=Truth::new(&x);
            for lo in 0..n {for hi in lo+1..=n {for h in [0,1,4,8] {for p in 0..n {
                let u=(lo,hi);x.p=p;
                let r=sub.run(&x,u,h+16,true).unwrap();
                dense.check_physical(u,h as usize,p,r.obj.as_ref().unwrap(),r.mark);
                assert_eq!(sub.out,dense.output(p));
                cases+=1;disjoint+=u32::from(r.mark.is_none());fallback+=u32::from(!r.second);
            }}}}
        }}
    }
    println!("exact_marked_cases={cases} disjoint_cycles={disjoint} including_first_level_fallback={fallback}");
}
fn blocks(mode:&str,path:&str,reps:u32){
    let mut reader=BufReader::new(std::fs::File::open(path).unwrap());
    let arms=[-1,0,4,8,16,20,24];let mut blocks=0;
    if mode=="price"{println!("stream,rep,arm,setup_mark_ns,discover_ns,merge_ns,publish_ns,first_ns,second_index_shift_ns,second_discover_ns,second_merge_ns,second_emit_ns,total_ns,reserved_bytes");}
    else{println!("stream,h,lo,hi,live_states,lf_pieces,forward_intersections,origin_bands,deleted_rows,overlay_fragments,physical_runs,core_runs,words,dict_bytes,max_mass,mark,skip,second,reserved_bytes");}
    while let Some(x)=Input::read(&mut reader) {
        let dense=Truth::new(&x);
        let expected=dense.output(x.p);
        let mut control=Subject::default();
        let q=control.run(&x,x.quote(),-2,true).unwrap();
        assert_eq!(control.out,expected);
        let u=highest(q.obj.as_ref().unwrap(),x.quote().0);
        let mut subjects:Vec<_>=(0..arms.len()).map(|_|Subject::default()).collect();
        for (&arm,sub) in arms.iter().zip(&mut subjects) {
            if arm<16 && mode!="price"{continue;}
            let r=sub.run(&x,u,arm,arm>=16).unwrap();
            assert_eq!(sub.out,expected);
            if arm>=16{
                let h=arm-16;
                let obj=r.obj.as_ref().unwrap();
                dense.check_physical(u,h as usize,x.p,obj,r.mark);
                if mode!="price"{
                    let (mark,skip)=r.mark.unwrap_or((u32::MAX,0));
                    let mut counts=HashMap::<u32,u32>::new();
                    for &(base,width) in &obj.1{*counts.entry(base).or_default()+=width;}
                    let s=r.metrics;
                    println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,h,u.0,u.1,
                        s[0],s[1],s[2],s[3],s[4],s[5],s[6],s[7],obj.0.len(),
                        obj.0.iter().map(|w|w.1.len()).sum::<usize>(),counts.values().max().unwrap(),mark,skip,r.second,r.cap);
                }
            }
        }
        if mode=="price" {
            for rep in 0..reps {for pos in [0,1,6,2,5,3,4] {
                let j=(rep as usize+pos)%arms.len();
                let r=subjects[j].run(&x,u,arms[j],false).unwrap();
                assert!(r.second);
                assert_eq!(subjects[j].out,expected);
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,rep,name(arms[j]),r.phases[0],r.phases[1],r.phases[2],
                         r.publish,r.first,r.inner[0],r.inner[1],r.inner[2],r.inner[3],r.total,r.cap);
            }}
        }
        blocks+=1;
    }
    eprintln!("verified {blocks} input blocks, physical h=0,4,8");
}
// RSS
fn main(){
    let args=std::env::args().collect::<Vec<_>>();
    if args.len()==1 {small();}
    else if args[1]=="rss"{rss(&args[2],args[3].parse().unwrap(),&args[4],&args[5]);}
    else{blocks(&args[1],&args[2],args.get(3).map_or(35,|s|s.parse().unwrap()));}
}
