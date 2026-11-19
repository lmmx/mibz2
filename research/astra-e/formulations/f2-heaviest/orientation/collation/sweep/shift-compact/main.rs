pub use mibz2::{bwt,runs,Kernels};
#[allow(dead_code,unused_imports)] mod projected;
#[allow(dead_code,unused_imports)] mod shared;
#[allow(dead_code,unused_imports)] mod control;
type Obj=(Vec<(u32,Vec<u8>)>,Vec<(u32,u32)>);
// INPUT_AND_DENSE_HELPERS
impl Truth{
    fn shifted(&self,u:(u32,u32),h:usize)->Option<Vec<u32>>{
        let mut ranks:Vec<_>=(u.0..u.1).collect();
        for _ in 0..h{
            let b=self.l[self.psi[ranks[0] as usize]];
            if ranks.iter().any(|&x|self.l[self.psi[x as usize]]!=b){return None;}
            for x in &mut ranks{*x=self.psi[*x as usize] as u32;}
            assert!(ranks.windows(2).all(|p|p[0]<p[1]));
        }
        Some(ranks)
    }
    fn object_on(&self,ranks:&[u32])->Obj{
        let mut rank=vec![u32::MAX;self.l.len()];for (i,&x) in ranks.iter().enumerate(){rank[x as usize]=i as u32;}
        let mut labels=HashMap::<Vec<u8>,(u32,u32)>::new();let mut root:Vec<(u32,u32)>=Vec::new();
        for &dest in ranks{
            let mut w=Vec::new();let mut x=dest as usize;
            loop{w.push(self.l[x]);x=self.lf[x];if rank[x]!=u32::MAX{break;}}
            w.reverse();
            let a=rank[x];let slot=labels.entry(w).or_insert((a,a));assert_eq!(slot.1,a);slot.1+=1;
            if let Some(p)=root.last_mut(){if p.0==slot.0{p.1+=1;continue;}}
            root.push((slot.0,1));
        }
        let mut words=labels.into_iter().map(|(w,(a,_))|(a,w)).collect::<Vec<_>>();words.sort_by_key(|p|p.0);
        (words,root)
    }
    fn shifted_mark(&self,ranks:&[u32],p:u32)->Option<(u32,u32)>{
        let mut x=p;let mut d=0;
        loop{if let Ok(id)=ranks.binary_search(&x){return Some((id as u32,d));}
            x=self.lf[x as usize] as u32;d+=1;if x==p{return None;}}
    }
}
#[derive(Default)] struct Subject{
    projected:projected::Dispatch<2,2>,
    original:control::Dispatch<2,2>,
    clipped:shared::Dispatch<2,2>,
    out:Vec<u8>,
}
struct Receipt{obj:Option<Obj>,mark:Option<(u32,u32)>,second:bool,metrics:[u32;8],phases:[u64;3],cap:usize,first:u64,total:u64}
impl Subject{
    fn run(&mut self,x:&Input,u:(u32,u32),arm:i32,snapshot:bool)->Option<Receipt>{
        let start=now();
        macro_rules! complete{($c:expr,$mark:expr,$m:expr,$fallback:expr,$metrics:expr)=>{{
            let end=now();let obj=snapshot.then(||$c.object());
            let second=$c.finish($mark,$m,x.n,$fallback,&mut self.out);
            let total=now()-start;
            Receipt{obj,mark:$mark,second,metrics:$metrics,phases:$c.phases,cap:$c.whole_capacity()+self.out.capacity(),first:end-start,total}
        }}}
        if arm== -2{
            let (a,m)=self.original.first(&x.caps,&x.freq,x.n,x.p)?;
            Some(complete!(self.original,a,m,x.p,[0;8]))
        }else if arm== -1{
            let (b,mono)=x.bucket(u);
            let (a,m)=if mono{self.clipped.first::<true>(&x.caps,&x.freq,x.n,x.p,u,b)}
                      else{self.clipped.first::<false>(&x.caps,&x.freq,x.n,x.p,u,b)}?;
            Some(complete!(self.clipped,a,m,x.p,[0;8]))
        }else{
            let (a,m,missing)=self.projected.first(&x.caps,&x.freq,x.n,x.p,u,arm as usize)?;
            Some(complete!(self.projected,a,m,missing,self.projected.measures))
        }
    }
}
fn name(arm:i32)->String{match arm{-2=>"compact_quote".into(),-1=>"compact_frequent".into(),h=>format!("projected_h{h}")}}
fn highest(obj:&Obj,lo:u32)->(u32,u32){
    let mut count=HashMap::<u32,u32>::new();
    for &(base,w) in &obj.1{*count.entry(base).or_default()+=w;}
    let (&a,&w)=count.iter().max_by_key(|(a,w)|(**w,u32::MAX-**a)).unwrap();(lo+a,lo+a+w)
}
// The full sorted rank and byte oracle does not use the compressed rank table or Plan.
fn check(x:&Input,truth:&Truth,p:(u32,u32),h:i32,sub:&mut Subject,expected:&[u8])->Option<Receipt>{
    let possible=truth.shifted(p,h as usize);
    let result=sub.run(x,p,h,true);
    assert_eq!(result.is_some(),possible.is_some());
    let r=result?;let ranks=possible.unwrap();
    assert_eq!(r.obj.as_ref().unwrap(),&truth.object_on(&ranks));
    assert_eq!(r.mark,truth.shifted_mark(&ranks,x.p));
    assert_eq!(sub.out,expected);
    Some(r)
}
fn small(){
    let mut stats=[0u64;6];let mut sub=Subject::default();
    for (alphabet,max_n) in [(2,7),(3,5)]{
        for n in 1..=max_n{for seed in 0..u32::pow(alphabet,n){
            let mut z=seed;let caps=(0..n).map(|_|{let c=z%alphabet;z/=alphabet;((c<<8)|1) as u16}).collect();
            let mut x=Input::new(0,0,caps);let dense=Truth::new(&x);
            for lo in 0..n{for hi in lo+1..=n{for h in 0..=3{
                for p in 0..n{x.p=p;
                    let Some(r)=check(&x,&dense,(lo,hi),h,&mut sub,&dense.output(p)) else {stats[1]+=1;break;};
                    stats[0]+=1;stats[2]+=u64::from(r.metrics[5]>0);stats[3]+=u64::from(r.metrics[5]==hi-lo);
                    stats[4]+=u64::from(r.mark.is_none());stats[5]+=u64::from(!r.second);
                }
            }}}
        }}
    }
    println!("marked_exact={} unavailable_sections={} early_marked={} all_early_marked={} disjoint_marks={} including_first_token_fallback={}",
            stats[0],stats[1],stats[2],stats[3],stats[4],stats[5]);
}
fn blocks(mode:&str,path:&str,reps:u32){
    let mut reader=BufReader::new(std::fs::File::open(path).unwrap());let mut totals=[0usize;4];
    if mode=="price"{println!("stream,rep,arm,setup_mark_ns,discover_ns,merge_ns,first_ns,total_ns,reserved_bytes");}
    else{println!("stream,h,lo,hi,live_nodes,lf_pieces,components,forbidden_components,deleted_rows,early_rows,early_bands,mono,words,dict_bytes,root_runs,mark,skip,second,reserved_bytes");}
    while let Some(x)=Input::read(&mut reader){
        let dense=Truth::new(&x);let expected=dense.output(x.p);
        let mut control=Subject::default();let q=control.run(&x,x.quote(),-2,true).unwrap();assert_eq!(control.out,expected);
        let u=highest(q.obj.as_ref().unwrap(),x.quote().0);
        let original_object=dense.object(u,false);
        let mut subjects:Vec<Subject>=(0..6).map(|_|Subject::default()).collect();
        let mut available=Vec::new();
        for h in 0..=3{if let Some(r)=check(&x,&dense,u,h,&mut subjects[h as usize+2],&expected){
            let object=r.obj.as_ref().unwrap();
            assert_eq!(object.1,original_object.1);
            assert_eq!(object.0.len(),original_object.0.len());
            for ((a,shifted),(b,word)) in object.0.iter().zip(&original_object.0){
                let mut rotated=word.clone();let turn=h as usize % rotated.len();rotated.rotate_left(turn);
                assert_eq!(a,b);assert_eq!(*shifted,rotated);
            }
            totals[h as usize]+=1;available.push(h);
            if mode!="price"{let (mk,skip)=r.mark.unwrap_or((u32::MAX,0));let (w,root)=r.obj.unwrap();let s=r.metrics;
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,h,u.0,u.1,s[0],s[1],s[2],s[3],s[4],s[5],s[6],s[7],
                         w.len(),w.iter().map(|p|p.1.len()).sum::<usize>(),root.len(),mk,skip,r.second,r.cap);}
        }}
        if mode=="price"{
            for rep in 0..reps{for pos in [0,1,5,2,4,3]{
                let j=(rep+pos)%6;let arm=j as i32-2;if arm>=0 && !available.contains(&arm){continue;}
                let sub=&mut subjects[j as usize];let r=sub.run(&x,u,arm,false).unwrap();assert_eq!(sub.out,expected);
                println!("{},{},{},{},{},{},{},{},{}",x.index,rep,name(arm),r.phases[0],r.phases[1],r.phases[2],r.first,r.total,r.cap);
            }}
        }
    }
    eprintln!("exact source intervals at h0..3 {totals:?}");
}
// RSS_ADAPTER
fn main(){let args=std::env::args().collect::<Vec<_>>();if args.len()==1{small();}
      else if args[1]=="rss"{rss(&args[2],args[3].parse().unwrap(),&args[4],&args[5]);}
      else{blocks(&args[1],&args[2],args.get(3).map_or(36,|s|s.parse().unwrap()));}}
