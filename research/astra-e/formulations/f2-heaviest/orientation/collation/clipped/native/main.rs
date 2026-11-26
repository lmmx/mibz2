pub use mibz2::{bwt,runs,Kernels};
#[allow(dead_code)] mod control;
#[allow(dead_code,unused_imports)] mod clip;
#[allow(dead_code,unused_imports)] mod fast;
type Obj=(Vec<(u32,Vec<u8>)>,Vec<(u32,u32)>);
type Native<const S:bool>=clip::Dispatch<2,2,S>;
type Original=control::Dispatch<2,2>;
use std::collections::HashMap;
use std::io::{Read,BufReader};

#[repr(C)] struct Ts {sec:i64,ns:i64}
unsafe extern "C"{fn clock_gettime(id:i32,t:*mut Ts)->i32;}
fn now()->u64{let mut t=Ts{sec:0,ns:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);(t.sec*1_000_000_000+t.ns) as u64}
struct Input{index:u32,n:u32,p:u32,caps:Vec<u16>,freq:[u32;256]}
impl Input {
    fn new(index:u32,p:u32,caps:Vec<u16>)->Self{
        let mut freq=[0;256];for &r in &caps{freq[(r>>8) as usize]+=u32::from(r&255);}
        Self{index,n:freq.iter().sum(),p,caps,freq}
    }
    fn read(r:&mut impl Read)->Option<Self>{
        let mut header=[0u8;16];
        if r.read(&mut header[..1]).unwrap()==0{return None;}
        r.read_exact(&mut header[1..]).unwrap();
        let mut h=header.chunks_exact(4).map(|b|u32::from_le_bytes(b.try_into().unwrap()));
        let (idx,n,p,m)=(h.next().unwrap(),h.next().unwrap(),h.next().unwrap(),h.next().unwrap());
        let mut buf=vec![0u8;2*m as usize];r.read_exact(&mut buf).unwrap();
        let x=Self::new(idx,p,buf.chunks_exact(2).map(|c|u16::from_le_bytes(c.try_into().unwrap())).collect());
        assert_eq!(x.n,n);Some(x)
    }
    fn bucket(&self,sec:(u32,u32))->(u8,bool){
        let mut a=0;
        for (b,&w) in self.freq.iter().enumerate(){
            let end=a+w;
            if sec.0<end{return (b as u8,sec.1<=end);}
            a=end;
        }
        unreachable!()
    }
    fn quote(&self)->(u32,u32){
        let a=self.freq[..34].iter().sum::<u32>();(a,a+self.freq[34])
    }
}
struct Truth{l:Vec<u8>,psi:Vec<usize>,lf:Vec<usize>}
impl Truth{
    fn new(x:&Input)->Self{
        let mut l=Vec::new();
        for &r in &x.caps{l.extend(std::iter::repeat_n((r>>8) as u8,(r&255) as usize));}
        let mut psi=(0..l.len()).collect::<Vec<_>>();
        psi.sort_by_key(|&i|l[i]);
        let mut lf=vec![0;l.len()];for (i,&p) in psi.iter().enumerate(){lf[p]=i;}
        Self{l,psi,lf}
    }
    fn output(&self,p:u32)->Vec<u8>{
        let mut row=p as usize;
        (0..self.l.len()).map(|_|{row=self.psi[row];self.l[row]}).collect()
    }
    fn mark(&self,p:u32,(a,b):(u32,u32),rotate:bool)->Option<(u32,u32)>{
        let mut x=p;let mut d=0;
        if !rotate && (a..b).contains(&p){return Some((p-a,0));}
        loop{x=self.lf[x as usize] as u32;d+=1;
            if (a..b).contains(&x){return Some((x-a,d-u32::from(rotate)));}
            if x==p{return None;}
        }
    }
    fn object(&self,(a,b):(u32,u32),rotate:bool)->Obj{
        let mut labels=HashMap::<Vec<u8>,(u32,u32)>::new();
        let mut incoming=Vec::new();
        for dest in a..b{
            let mut word=Vec::new();let mut x=dest;
            loop{
                word.push(self.l[x as usize]);x=self.lf[x as usize] as u32;
                if (a..b).contains(&x){break;}
            }
            word.reverse();if rotate{word.rotate_left(1);}
            let from=x-a;
            let e=labels.entry(word.clone()).or_insert((from,from));
            assert_eq!(from,e.1);e.1+=1;
            incoming.push(e.0);
        }
        let mut words=labels.into_iter().map(|(w,(p,_))|(p,w)).collect::<Vec<_>>();
        words.sort_by_key(|w|w.0);
        let mut runs:Vec<(u32,u32)>=Vec::new();
        for base in incoming{
            if let Some(last)=runs.last_mut(){if last.0==base{last.1+=1;continue;}}
            runs.push((base,1));
        }
        (words,runs)
    }
}
#[derive(Default)] struct Library{mono:Native<true>,multi:Native<false>,control:Original,quick:fast::Dispatch<2,2,true>,general:fast::Dispatch<2,2,false>,bytes:Vec<u8>}
struct ResultRow{object:Option<Obj>,mark:Option<(u32,u32)>,second:bool,first_ns:u64,total_ns:u64,mixed:u32,splits:u32,caps:usize,phases:[u64;3]}
fn execute(lib:&mut Library,input:&Input,sec:(u32,u32),arm:&str,snapshot:bool)->ResultRow{
    let t=now();
    let (first,single)=input.bucket(sec);
    let caps=&input.caps;let freq=&input.freq;let n=input.n;let p=input.p;
    let native=arm!="original";
    let mono=single && !arm.contains("raw_quote");
    macro_rules! run {($s:expr,$st:expr,$mixed:expr,$splits:expr)=>{{
        let (mark,m)=$st.unwrap();
        let end=now();
        let object=if snapshot{Some($s.object())}else{None};
        let second=$s.finish(mark,m,n,p,&mut lib.bytes);
        let total_ns=now()-t;
        #[cfg(clipped_profile)] let phases=$s.phases;
        #[cfg(not(clipped_profile))] let phases=[0;3];
        ResultRow{object,mark,second,first_ns:end-t,total_ns,mixed:$mixed,splits:$splits,caps:$s.whole_capacity()+lib.bytes.capacity(),phases}
    }}}
    if !native{return run!(lib.control,lib.control.first(caps,freq,n,p),0,0);}
    macro_rules! clipped {($s:expr)=>{{
        let res=$s.first(caps,freq,n,p,sec,first);
        #[cfg(clipped_counts)] let (mixed,splits)=($s.mixed,$s.interior_splits);
        #[cfg(not(clipped_counts))] let (mixed,splits)=(0,0);
        run!($s,res,mixed,splits)
    }}}
    if arm.starts_with("fast"){
        if mono{clipped!(lib.quick)}else{clipped!(lib.general)}
    }else if mono{clipped!(lib.mono)}else{clipped!(lib.multi)}
}
fn highest(obj:&Obj,lo:u32)->(u32,u32){
    let mut count=HashMap::<u32,u32>::new();
    for &(base,w) in &obj.1{*count.entry(base).or_default()+=w;}
    let (&a,&w)=count.iter().max_by_key(|(a,w)|(**w,u32::MAX-**a)).unwrap();
    (lo+a,lo+a+w)
}
fn small(){
    let mut counters=[0u64;7];
    for (alphabet,max_n) in [(2,8),(3,5)]{
        for n in 1..=max_n{for seed in 0..u32::pow(alphabet,n){
            let mut code=seed;
            let caps=(0..n).map(|_|{let c=33+code%alphabet;code/=alphabet;((c<<8)|1) as u16}).collect();
            let mut x=Input::new(0,0,caps);let check=Truth::new(&x);
            let mut lib=Library::default();
            for a in 0..n{for b in a+1..=n{
                let s=(a,b);let mono=x.bucket(s).1;let object=check.object(s,mono);
                for p in 0..n{
                    x.p=p;let result=execute(&mut lib,&x,s,"auto",p==0);
                    if p==0{assert_eq!(result.object.as_ref().unwrap(),&object);counters[0]+=1;
                       assert!(result.mixed<=2 && result.splits<=2);
                       counters[1+result.mixed as usize]+=1;}
                    assert_eq!(result.mark,check.mark(p,s,mono));assert_eq!(lib.bytes,check.output(p));
                    let quick=execute(&mut lib,&x,s,"fast",p==0);
                    assert_eq!(quick.object,result.object);assert_eq!(quick.mark,result.mark);
                    assert_eq!((quick.mixed,quick.splits),(result.mixed,result.splits));assert_eq!(lib.bytes,check.output(p));
                    counters[4]+=1;counters[5]+=u64::from(result.mark.is_none());counters[6]+=u64::from(!result.second);
                }
            }}
        }}
    }
    println!("sections={} mixed0={} mixed1={} mixed2={} marked={} disjoint={} including_token_fallback={}",
             counters[0],counters[1],counters[2],counters[3],counters[4],counters[5],counters[6]);
}
fn blocks(path:&str,mode:&str,reps:u32){
    let mut reader=BufReader::new(std::fs::File::open(path).unwrap());
    if mode=="time"{println!("stream,rep,arm,first_ns,total_ns");}
    else{println!("stream,section,lo,hi,rotated,words,root_runs,dict_bytes,mixed,run_splits,mark,skip,second,reserved_with_output");}
    let mut count=0;
    while let Some(x)=Input::read(&mut reader){
        let truth=Truth::new(&x);let expected=truth.output(x.p);
        let mut orig=Library::default();let q=x.quote();
        let ref_result=execute(&mut orig,&x,q,"original",true);assert_eq!(orig.bytes,expected);
        let obj=ref_result.object.as_ref().unwrap();let f=highest(obj,q.0);
        let sections=[q,q,q,f,(x.n/5,x.n/5+x.n/11),(x.n/2,x.n/2+x.n/13)];
        let names=["original","quote","raw_quote","frequent","fifth","half"];
        let mut libs:Vec<_>=(0..6).map(|_|Library::default()).collect();
        if mode=="time"{
            let order=[0,1,5,2,4,3];
            for rep in 0..reps{for pos in order{
                let arm=(rep as usize+pos)%6;
                let r=execute(&mut libs[arm],&x,sections[arm],names[arm],false);
                assert_eq!(libs[arm].bytes,expected);
                println!("{},{},{},{},{}",x.index,rep,names[arm],r.first_ns,r.total_ns);
            }}
        }else{
            for arm in 1..6{
                let sec=sections[arm];let rotate=x.bucket(sec).1 && arm!=2;
                let which=if mode=="fast_verify"{format!("fast_{}",names[arm])}else{names[arm].to_owned()};
                let r=execute(&mut libs[arm],&x,sec,&which,true);
                assert_eq!(libs[arm].bytes,expected);assert_eq!(r.mark,truth.mark(x.p,sec,rotate));
                let got=r.object.unwrap();assert_eq!(got,truth.object(sec,rotate));
                if arm==1{assert_eq!(got,*obj);assert_eq!(ref_result.mark,r.mark);}
                assert!(r.mixed<=2 && r.splits<=2);
                let (mark,skip)=r.mark.unwrap_or((u32::MAX,0));
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,names[arm],sec.0,sec.1,rotate,
                         got.0.len(),got.1.len(),got.0.iter().map(|w|w.1.len()).sum::<usize>(),r.mixed,r.splits,mark,skip,r.second,r.caps);
            }
        }
        count+=1;
    }
    eprintln!("{count} inputs verified");
}
fn price(path:&str,reps:u32,custom:Option<&str>){
    let mut reader=BufReader::new(std::fs::File::open(path).unwrap());
    let mut intervals=HashMap::new();
    if let Some(source)=custom{
        let content=std::fs::read_to_string(source).unwrap();
        for row in content.lines().skip(1){
            let c=row.split(',').collect::<Vec<_>>();
            let idx=c[0].parse::<u32>().unwrap();let lo=c[1].parse::<u32>().unwrap();let hi=c[2].parse::<u32>().unwrap();
            assert!(intervals.insert(idx,(lo,hi)).is_none());
        }
    }
    println!("stream,rep,arm,lo,hi,first_ns,total_ns,prepare_mark_ns,discover_ns,merge_ns,reserved_with_output");
    while let Some(x)=Input::read(&mut reader){
        let truth=Truth::new(&x);let expected=truth.output(x.p);
        let q=x.quote();let mut reference=Library::default();
        let r=execute(&mut reference,&x,q,"original",true);
        let f=highest(&r.object.unwrap(),q.0);
        let other=if custom.is_some(){*intervals.get(&x.index).unwrap()}else{(x.n/5,x.n/5+x.n/11)};
        let names=["original","quote","fast_quote","frequent","fast_frequent","fifth_or_custom","fast_fifth_or_custom","fast_raw_quote"];
        let sections=[q,q,q,f,f,other,other,q];
        let mut libs:Vec<_>=(0..8).map(|_|Library::default()).collect();
        for (j,name) in names.iter().enumerate(){
            let s=sections[j];let rotate=x.bucket(s).1 && !name.contains("raw_quote");
            let result=execute(&mut libs[j],&x,s,name,true);
            assert_eq!(result.object.unwrap(),truth.object(s,rotate));assert_eq!(result.mark,truth.mark(x.p,s,rotate));assert_eq!(libs[j].bytes,expected);
        }
        let order=[0,1,7,2,6,3,5,4];
        for rep in 0..reps{for k in order{
            let j=(rep as usize+k)%8;
            let name=names[j];let s=sections[j];let r=execute(&mut libs[j],&x,s,name,false);
            assert_eq!(libs[j].bytes,expected);
            println!("{},{},{},{},{},{},{},{},{},{},{}",x.index,rep,name,s.0,s.1,r.first_ns,r.total_ns,r.phases[0],r.phases[1],r.phases[2],r.caps);
        }}
    }
}
fn main(){
    let args=std::env::args().collect::<Vec<_>>();
    if args.len()==1{small();}else if args[1]=="price"{price(&args[2],args[3].parse().unwrap(),args.get(4).map(String::as_str));}else{blocks(&args[2],&args[1],args.get(3).map_or(24,|s|s.parse().unwrap()));}
}
