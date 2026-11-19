const NAMES:[&str;6]=["compact_quote","adaptive_frequent","literal_frequent","comma_quote","property_landed","twice_heaviest"];
#[derive(Default)]
struct Subject {
    native:shared::Dispatch<2,2>,quote:control::Dispatch<2,2>,selector:LiteralSelector,
    heap:std::collections::BinaryHeap<(u32,std::cmp::Reverse<u32>)>,out:Vec<u8>,
}
struct Receipt {
    sec:(u32,u32),mark:Option<(u32,u32)>,obj:Option<Obj>,second:bool,info:[usize;7],
    select:u64,first:[u64;3],inner:[u64;4],total:u64,cap:usize,
}
impl Subject{
    fn run(&mut self,x:&Input,arm:usize,snapshot:bool)->Receipt{
        let start=now();
        let quote=x.quote();
        let sec=match arm{
            0=>quote,
            1|5=>if quote.0<quote.1{self.native.frequent_section(&x.caps,&x.freq,x.n,quote,if arm==5{2}else{1},&mut self.heap).unwrap()}else{quote},
            2=>LiteralSelector::short(&x.caps,&x.freq,FREQUENT),
            3=>LiteralSelector::short(&x.caps,&x.freq,COMMA),
            4=>self.selector.sampled(&x.caps,&x.freq,PROPERTY),
            _=>unreachable!(),
        };
        let select=now()-start;
        let extra=self.selector.pages.capacity()*std::mem::size_of::<Page>()
                 +self.heap.capacity()*std::mem::size_of::<(u32,std::cmp::Reverse<u32>)>();
        if sec.0==sec.1{
            self.native.raw_cycle(&x.caps,&x.freq,x.n,x.p,&mut self.out);
            return Receipt{sec,mark:None,obj:None,second:false,info:[0;7],select,first:[0;3],inner:[0;4],
                           total:now()-start,cap:self.native.whole_capacity()+self.out.capacity()+extra}
        }
        macro_rules! finish {($subject:expr,$mark:expr,$m:expr)=>{{
            let obj=snapshot.then(||$subject.object());
            let second=$subject.finish($mark,$m,x.n,x.p,&mut self.out);
            let total=now()-start;
            Receipt{sec,mark:$mark,obj,second,info:if second{$subject.info()}else{[0;7]},select,first:$subject.phases,
                    inner:if second{$subject.second_phases()}else{[0;4]},total,
                    cap:$subject.whole_capacity()+self.out.capacity()+extra}
        }}}
        if arm==0{
            let (a,m)=self.quote.first(&x.caps,&x.freq,x.n,x.p).unwrap();
            return finish!(self.quote,a,m);
        }
        let c=if arm==3{b','}else{b'"'};
        let (a,m)=self.native.first::<true>(&x.caps,&x.freq,x.n,x.p,sec,c).unwrap();
        finish!(self.native,a,m)
    }
}
impl Truth {
    fn cylinder(&self,freq:&[u32;256],w:&[u8])->(u32,u32){
        let c=w[0] as usize;
        let first=freq[..c].iter().sum::<u32>();
        let mut hits=None;
        for x in first..first+freq[c]{
            let mut y=x as usize;
            if w.iter().all(|&b|{y=self.psi[y];self.l[y]==b}){
                let e=hits.get_or_insert((x,x));
                assert_eq!(e.1,x);e.1+=1;
            }
        }
        hits.unwrap_or((0,0))
    }
    fn sections(&self,x:&Input)->[(u32,u32);6]{
        let q=self.cylinder(&x.freq,b"\"");
        let frequent=if q.0<q.1{highest(&self.object(q,false),q.0)}else{q};
        let twice=if frequent.0<frequent.1{highest(&self.object(frequent,false),frequent.0)}else{frequent};
        [q,frequent,self.cylinder(&x.freq,FREQUENT),self.cylinder(&x.freq,COMMA),self.cylinder(&x.freq,PROPERTY),twice]
    }
    fn receipt(&self,x:&Input,sec:(u32,u32),r:&Receipt){
        if sec.0==sec.1{
            assert_eq!(r.sec.0,r.sec.1);assert!(r.obj.is_none());assert!(r.mark.is_none());return;
        }
        assert_eq!(sec,r.sec);assert_eq!(r.obj.as_ref().unwrap(),&self.object(sec,true));
        assert_eq!(r.mark,self.mark(x.p,sec,true));
    }
}
fn check_small(x:&mut Input,subjects:&mut [Subject],stats:&mut[u64;4]){
    let dense=Truth::new(x);let sections=dense.sections(x);
    for p in 0..x.n{
        x.p=p;let expected=dense.output(p);
        for (arm,sub) in subjects.iter_mut().enumerate(){
            let r=sub.run(x,arm,true);let s=sections[arm];
            dense.receipt(x,s,&r);
            assert_eq!(sub.out,expected);
            stats[0]+=1;stats[1]+=u64::from(s.0==s.1);stats[2]+=u64::from(s.0<s.1 && r.mark.is_none());
            stats[3]+=u64::from(r.second);
        }
    }
}
fn last_from_text(t:&[u8])->Vec<u16>{
    let n=t.len();
    let mut order=(0..n).collect::<Vec<_>>();
    order.sort_by(|&a,&b|(0..n).map(|j|t[(a+j)%n]).cmp((0..n).map(|j|t[(b+j)%n])));
    order.iter().map(|&i|(u16::from(t[(i+n-1)%n])<<8)|1).collect()
}
fn small(){
    let mut subjects:Vec<_>=(0..NAMES.len()).map(|_|Subject::default()).collect();
    let mut stats=[0;4];
    for alphabet in [[34,44],[34,58],[34,112]]{
        for n in 1..=7{for seed in 0..(1<<n){
            let caps=(0..n).map(|i|((alphabet[(seed>>i)&1] as u16)<<8)|1).collect();
            check_small(&mut Input::new(0,0,caps),&mut subjects,&mut stats);
        }}
    }
    for text in [PROPERTY,COMMA,FREQUENT,b"missing",b"\",\"property\":\",\"property\":\"",b"\",\"propertyxxx\",\"property\":\""]{
        check_small(&mut Input::new(0,0,last_from_text(text)),&mut subjects,&mut stats);
    }
    println!("marked={} empty_policy={} disjoint_mark={} second={}",stats[0],stats[1],stats[2],stats[3]);
}
fn blocks(mode:&str,path:&str,reps:u32){
    let mut reader=BufReader::new(std::fs::File::open(path).unwrap());let mut cases=0;
    if mode=="price"{println!("stream,rep,arm,select_ns,setup_mark_ns,discover_ns,merge_ns,second_index_ns,second_discover_ns,second_merge_ns,second_emit_ns,total_ns,reserved_bytes,second");}
    else{println!("stream,arm,lo,hi,mark,skip,words,dict_bytes,token_runs,edge_bytes,selectors,lf_pieces,second_dict_bytes,second,reserved_bytes");}
    while let Some(x)=Input::read(&mut reader){
        let dense=Truth::new(&x);let expected=dense.output(x.p);let sections=dense.sections(&x);
        let mut subjects:Vec<_>=(0..NAMES.len()).map(|_|Subject::default()).collect();
        for (arm,sub) in subjects.iter_mut().enumerate(){
            let r=sub.run(&x,arm,true);dense.receipt(&x,sections[arm],&r);
            assert_eq!(sub.out,expected);
            if mode!="price"{
                let (mark,skip)=r.mark.unwrap_or((u32::MAX,0));let s=r.info;
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,NAMES[arm],r.sec.0,r.sec.1,mark,skip,
                         s[0],s[1],s[2],s[3],s[4],s[5],s[6],r.second,r.cap);
            }
        }
        if mode=="price"{
            for rep in 0..reps{for pos in [0,1,5,2,4,3]{
                let arm=(rep as usize+pos)%NAMES.len();let sub=&mut subjects[arm];let r=sub.run(&x,arm,false);assert_eq!(sub.out,expected);
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,rep,NAMES[arm],r.select,r.first[0],r.first[1],r.first[2],
                         r.inner[0],r.inner[1],r.inner[2],r.inner[3],r.total,r.cap,r.second);
            }}
        }
        cases+=1;
    }
    eprintln!("verified {cases} blocks under six policies");
}
fn rss(path:&str,arm:usize,hashes:&str){
    let mut digests=HashMap::new();
    for s in std::fs::read_to_string(hashes).unwrap().lines().skip(1){
        let (i,h)=s.split_once(',').unwrap();digests.insert(i.parse::<u32>().unwrap(),h.parse::<u64>().unwrap());
    }
    let mut rd=BufReader::new(std::fs::File::open(path).unwrap());
    let mut sub=Subject::default();let mut cases=0;
    println!("stream,arm,reserved_bytes,process_hwm_kib");
    while let Some(x)=Input::read(&mut rd){
        let r=sub.run(&x,arm,false);
        let h=sub.out.iter().fold(0xcbf29ce484222325u64,|v,&b|(v^u64::from(b)).wrapping_mul(0x100000001b3));
        assert_eq!(h,digests[&x.index]);
        let state=std::fs::read_to_string("/proc/self/status").unwrap();
        let peak=state.lines().find(|s|s.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap();
        println!("{},{},{},{}",x.index,NAMES[arm],r.cap,peak);cases+=1;
    }
    assert_eq!(cases,228);
}
fn main(){let args=std::env::args().collect::<Vec<_>>();
    if args.len()==1{small()}
    else if args[1]=="rss"{rss(&args[2],args[3].parse().unwrap(),&args[4]);}
    else{blocks(&args[1],&args[2],args.get(3).map_or(36,|s|s.parse().unwrap()));}
}
