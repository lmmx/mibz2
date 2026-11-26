#[allow(dead_code,unused_imports)] mod lifted12;
#[derive(Default)] struct Worker {
    fast:control::Dispatch<2,2>, lift:lifted::Dispatch<2,2>, twelve:lifted12::Dispatch<2,2>, out:Vec<u8>,
}
const LABELS:[&str;8]=["named_quote","full_word_marker","sparse4","sparse8","sparse12","lift4","lift8","lift12"];
struct Row { obj:Option<Obj>, mark:Option<(u32,u32)>, second:bool, counts:[u32;5], tokens:u32,
             first:[u64;3],publish:u64,inner:[u64;4],total:u64,geom:[usize;7],capacity:usize }
impl Worker{
    fn run(&mut self,x:&Input,arm:usize,snapshot:bool)->Row{
        let t=now();let q=x.quote();
        let mut counts=[0;5];let mut publication=0;
        macro_rules! finish {($worker:expr,$mark:expr,$m:expr)=>{{
            let obj=if snapshot{Some($worker.object())}else{None};
            let sec=$worker.finish($mark,$m,x.n,x.p,&mut self.out);
            let total=now()-t;
            Row{obj,mark:$mark,second:sec,counts,tokens:$m,first:$worker.phases,
                publish:publication,inner:if sec{$worker.second_phases()}else{[0;4]},total,
                geom:if sec{$worker.information()}else{[0;7]},capacity:$worker.whole_capacity()+self.out.capacity()}
        }}}
        if arm<5{
            let (mut mark,mut m)=self.fast.first(&x.caps,&x.freq,x.n,x.p).unwrap();
            if arm>0{
                let h=[0,0,4,8,12][arm];
                let a=now();(mark,m,counts)=self.fast.insert_marker(mark,m,h);publication=now()-a;
            }
            return finish!(self.fast,mark,m);
        }
        if arm==7{
            let (mark,m,_)=self.twelve.first(&x.caps,&x.freq,x.n,x.p,q,12,true).unwrap();
            publication=self.twelve.compose_ns;
            return finish!(self.twelve,mark,m);
        }
        let (mark,m,_)=self.lift.first(&x.caps,&x.freq,x.n,x.p,q,if arm==5{4}else{8},true).unwrap();
        publication=self.lift.compose_ns;
        finish!(self.lift,mark,m)
    }
}
fn verify_sparse(row:&Row,origins:&Origins,truth:&Truth,p:u32,arm:usize){
    let (obj,mark,s)=origins.expected(truth,p,[0,0,4,8,12][arm]);
    assert_eq!(row.mark,mark);assert_eq!(row.obj.as_ref(),Some(&obj));assert_eq!(row.counts[0],s);
}
fn small(){
    let mut w=Worker::default();let mut checked=0;let mut absent=0;let mut transferred=0;let mut disjoint=0;
    let mut empty_prefix=0;
    for n in 1..=7{
        let alphabet=[b'"',b',',b'x'];
        let k=if n<=5{3usize}else{2};
        for mask in 0..k.pow(n){
            let mut z=mask;
            let caps=(0..n).map(|_|{let b=alphabet[z%k];z/=k;(u16::from(b)<<8)|1}).collect();
            let mut x=Input::new(0,0,caps);
            if x.freq[34]==0{continue;}
            let truth=Truth::new(&x);let origins=Origins::new(&truth,x.quote());
            for p in 0..n{
                x.p=p;let output=truth.output(p);
                for arm in 1..5{
                    let row=w.run(&x,arm,true);
                    verify_sparse(&row,&origins,&truth,p,arm);
                    assert_eq!(w.out,output);
                    absence_add(&mut absent,row.counts[0]==0);
                    absence_add(&mut disjoint,row.mark.is_none());
                    if row.obj.as_ref().unwrap().0.iter().any(|v|v.1.is_empty()){empty_prefix+=1;}
                    transferred+=row.counts[4];checked+=1;
                }
            }
        }
    }
    println!("exact augmented columns and marked bytes {checked}; absent fixed marker {absent}; mark in moved suffix {transferred}; disjoint old marked cycle {disjoint}; zero-length old prefix present {empty_prefix}");
}
fn absence_add(v:&mut u32,b:bool){*v+=u32::from(b);}
fn rss(path:&str,arm:usize,hashes:&str){
    let mut expected=HashMap::new();
    for line in std::fs::read_to_string(hashes).unwrap().lines().skip(1){
        let (i,h)=line.split_once(',').unwrap();
        expected.insert(i.parse::<u32>().unwrap(),h.parse::<u64>().unwrap());
    }
    let mut rd=BufReader::new(std::fs::File::open(path).unwrap());let mut w=Worker::default();let mut count=0;
    println!("stream,arm,retained_bytes_with_output,process_hwm_kib");
    while let Some(x)=Input::read(&mut rd){
        let r=w.run(&x,arm,false);
        let hash=w.out.iter().fold(0xcbf29ce484222325u64,|v,&b|(v^u64::from(b)).wrapping_mul(0x100000001b3));
        assert_eq!(hash,expected[&x.index]);
        let state=std::fs::read_to_string("/proc/self/status").unwrap();
        let hwm=state.lines().find(|s|s.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap();
        println!("{},{},{},{}",x.index,LABELS[arm],r.capacity,hwm);count+=1;
    }
    assert_eq!(count,228);
}
fn main(){
    let args=std::env::args().collect::<Vec<_>>();if args.len()==1{small();return;}
    let mode=&args[1];
    if mode=="rss"{rss(&args[2],args[3].parse().unwrap(),&args[4]);return;}
    let arm_filter=args.get(4).map(|s|s.parse::<usize>().unwrap());
    let reps=args.get(3).map_or(32,|s|s.parse().unwrap());
    let mut input=BufReader::new(std::fs::File::open(&args[2]).unwrap());
    let mut ws:Vec<_>=(0..8).map(|_|Worker::default()).collect();
    if mode=="price"{println!("stream,rep,position,arm,setup_ns,discover_ns,merge_ns,publication_ns,second_index_ns,second_discover_ns,second_merge_ns,second_emit_ns,total_ns");}
    else{println!("stream,arm,rows,old_mass,selected,old_types_selected,filtered_runs,augmented_runs,mark_in_suffix,tokens,mark,skip,second,lookups,nodes,words,dictionary_bytes,selectors,merged,links,reserved_bytes");}
    let mut blocks=0;
    while let Some(x)=Input::read(&mut input){
        let truth=Truth::new(&x);let expected=truth.output(x.p);
        let oracle=(mode=="geometry").then(||Origins::new(&truth,x.quote()));
        assert!(x.freq[34]>0);
        for arm in 0..8{
            if arm_filter.is_some_and(|i|i!=arm){continue;}
            let r=ws[arm].run(&x,arm,oracle.is_some()&&(1..5).contains(&arm));
            assert_eq!(ws[arm].out,expected);
            if (1..5).contains(&arm){if let Some(ref o)=oracle{verify_sparse(&r,o,&truth,x.p,arm);}}
            if mode!="price"{
                let (m,skip)=r.mark.unwrap_or((u32::MAX,0));
                let g=r.geom;let c=r.counts;
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,LABELS[arm],x.n,x.freq[34],
                         c[0],c[1],c[2],c[3],c[4],r.tokens,m,skip,r.second,g[0],g[1],g[2],g[3],g[4],g[5],g[6],r.capacity);
            }
        }
        if mode=="price"{
            for rep in 0..reps{for (pos,j) in [0,1,7,2,6,3,5,4].into_iter().enumerate(){
                let arm=(rep+j)%8;let r=ws[arm].run(&x,arm,false);assert_eq!(ws[arm].out,expected);
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,rep,pos,LABELS[arm],r.first[0],r.first[1],r.first[2],r.publish,r.inner[0],r.inner[1],r.inner[2],r.inner[3],r.total);
            }}
        }
        blocks+=1;
    }
    eprintln!("{blocks} blocks matched dense marked inverse");
}
