const LABELS:[&str;6]=["named_a","named_b","sparse4","quotient4","quotient8","quotient12"];
#[derive(Default)]
struct Worker{fast:control::Dispatch<2,2>,out:Vec<u8>}
struct ResultRow{
    counts:[u32;10],tokens:u32,mark:Option<(u32,u32)>,obj:Option<Obj>,
    second:bool,winner:u32,first:[u64;3],inner:[u64;4],publish:u64,total:u64,info:[usize;7],cap:usize,
}
impl Worker{
    #[inline(never)]
    fn run(&mut self,x:&Input,route:usize,inspect:u8)->ResultRow{
        let t=now();
        let (mut mark,mut m)=self.fast.first(&x.caps,&x.freq,x.n,x.p).unwrap();
        let mut counts=[0;10];let mut publish=0;
        if route==2{
            let t1=now();let r=self.fast.insert_marker(mark,m,4);mark=r.0;m=r.1;publish=now()-t1;
        }else if route>=3{
            let h=[4,8,12][route-3];let t1=now();
            (mark,m,counts)=self.fast.publish_quotient(mark,m,h);publish=now()-t1;
        }
        let obj=(inspect==2).then(||self.fast.object());
        let second=self.fast.finish(mark,m,x.n,x.p,&mut self.out);
        let total=now()-t;
        ResultRow{counts,tokens:m,mark,obj,second,winner:if second && inspect!=0{self.fast.chosen_base()}else{u32::MAX},
                  first:self.fast.phases,inner:if second{self.fast.second_phases()}else{[0;4]},publish,total,
                  info:if second{self.fast.information()}else{[0;7]},cap:self.fast.whole_capacity()+self.out.capacity()}
    }
}
fn check(r:&ResultRow,o:&Origins,truth:&Truth,p:u32,h:u32)->(usize,usize){
    let (obj,mark,selected,z)=o.dense_quotient(truth,p,h);
    assert_eq!(r.mark,mark);assert_eq!(r.obj.as_ref(),Some(&obj));assert_eq!(r.tokens as usize,o.names.len()+selected-z);
    if r.counts[0]==2{assert_eq!((r.counts[1] as usize,r.counts[2] as usize),(selected,z));}
    (selected,z)
}
fn small(){
    let mut w=Worker::default();let mut cases=0;let mut modes=[0;3];let mut marks=[0;3];let mut disjoint=0;let mut tie=0;
    for n in 1..=8{
        let alphabet=[b'"',b',',b'x'];let k=if n<=6{3usize}else{2};
        for mask in 0..k.pow(n){
            let mut z=mask;let caps=(0..n).map(|_|{let b=alphabet[z%k];z/=k;(u16::from(b)<<8)|1}).collect();
            let mut x=Input::new(0,0,caps);if x.freq[34]==0{continue;}
            let truth=Truth::new(&x);let o=Origins::new(&truth,x.quote());
            for p in 0..n{x.p=p;let expected=truth.output(p);
                for (route,h) in [(3,4),(4,8),(5,12)]{
                    let r=w.run(&x,route,2);let _=check(&r,&o,&truth,p,h);assert_eq!(w.out,expected);
                    modes[r.counts[0] as usize]+=1;marks[r.counts[8] as usize]+=1;disjoint+=u32::from(r.mark.is_none());
                    if r.counts[0]==2 && r.second && r.info[6]==r.counts[1] as usize && r.winner!=r.counts[5]{tie+=1;}
                    cases+=1;
                }
            }
        }
    }
    println!("{cases} dense quotient objects, ranks/skips and marked outputs; fallback/no-op/mixed {modes:?}; none-or-old/Z/non-Z-suffix marks {marks:?}; {disjoint} disjoint old marks; {tie} tied nonmarker second winners on a mixed quotient.");
}
fn geometry(mode:&str,path:&str){
    let mut input=BufReader::new(std::fs::File::open(path).unwrap());
    let mut ws:Vec<_>=(0..6).map(|_|Worker::default()).collect();let mut total=0;
    println!("stream,arm,rows,original_mass,mode,selected,z,hazard_mass,hazard_runs,z_base,old_runs,new_runs,marked_kind,hazard_types,tokens,mark,skip,second,second_base,lookups,second_words,second_dictionary_bytes,second_links,reserved_bytes");
    while let Some(x)=Input::read(&mut input){
        let truth=Truth::new(&x);let expected=truth.output(x.p);let o=(mode=="geometry").then(||Origins::new(&truth,x.quote()));
        for arm in 0..6{
            let r=ws[arm].run(&x,if arm==1{0}else{arm},if arm>=3 && o.is_some(){2}else{1});assert_eq!(ws[arm].out,expected);
            let mut c=r.counts;
            if arm>=3{if let Some(ref oo)=o{
                let (s,z)=check(&r,oo,&truth,x.p,[4,8,12][arm-3]);c[1]=s as u32;c[2]=z as u32;
            }}
            let (m,skip)=r.mark.unwrap_or((u32::MAX,0));let i=r.info;
            println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,LABELS[arm],x.n,x.freq[34],
                      c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9],r.tokens,m,skip,r.second,r.winner,i[0],i[2],i[3],i[6],r.cap);
        }
        total+=1;
    }
    eprintln!("{total} blocks, all six arms matched all dense marked bytes");
}
fn price(path:&str){
    let mut input=BufReader::new(std::fs::File::open(path).unwrap());let mut blocks=Vec::new();
    while let Some(x)=Input::read(&mut input){let y=Truth::new(&x).output(x.p);blocks.push((x,y));}
    assert_eq!(blocks.len(),10);let mut ws:Vec<_>=(0..6).map(|_|Worker::default()).collect();
    let codes=[0,0,2,3,4,5];
    for (x,y) in &blocks{for (arm,w) in ws.iter_mut().enumerate(){w.run(x,codes[arm],0);assert_eq!(&w.out,y);}}
    println!("cluster,rep,block_position,stream,position,arm,setup_ns,discover_ns,merge_ns,publication_ns,second_index_ns,second_discover_ns,second_merge_ns,second_emit_ns,total_ns");
    for rep in 0..60 {let cluster=rep/6;let turn=rep%6;
        for step in 0..10{
            let ix=(3*rep+if cluster%2==0{step}else{10-step})%10;
            let (x,y)=&blocks[ix];
            for (pos,j) in [0,1,5,2,4,3].into_iter().enumerate(){
                let arm=(turn+ix+cluster+if cluster%2==0{j}else{6-j})%6;
                let r=ws[arm].run(x,std::hint::black_box(codes[arm]),0);assert_eq!(&ws[arm].out,y);
                println!("{cluster},{rep},{step},{},{pos},{},{},{},{},{},{},{},{},{},{}",x.index,LABELS[arm],r.first[0],r.first[1],r.first[2],
                         r.publish,r.inner[0],r.inner[1],r.inner[2],r.inner[3],r.total);
            }
        }
    }
    eprintln!("3600 exact marked measured calls, 60 ten-block rounds, 10 balanced temporal clusters");
}
fn rss(path:&str,arm:usize,hashes:&str){
    let mut expected=HashMap::new();
    for line in std::fs::read_to_string(hashes).unwrap().lines().skip(1){
        let (i,h)=line.split_once(',').unwrap();expected.insert(i.parse::<u32>().unwrap(),h.parse::<u64>().unwrap());
    }
    let mut rd=BufReader::new(std::fs::File::open(path).unwrap());let mut w=Worker::default();let mut n=0;
    println!("stream,arm,retained_bytes_with_output,process_hwm_kib");
    while let Some(x)=Input::read(&mut rd){
        let r=w.run(&x,if arm==1{0}else{arm},0);
        let digest=w.out.iter().fold(0xcbf29ce484222325u64,|v,&b|(v^u64::from(b)).wrapping_mul(0x100000001b3));
        assert_eq!(digest,expected[&x.index]);n+=1;
        let state=std::fs::read_to_string("/proc/self/status").unwrap();
        let hwm=state.lines().find(|s|s.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap();
        println!("{},{},{},{}",x.index,LABELS[arm],r.cap,hwm);
    }
    assert_eq!(n,228);
}
fn main(){let args=std::env::args().collect::<Vec<_>>();
    if args.len()==1{small();return;}
    match args[1].as_str(){
        "price"=>price(&args[2]),"rss"=>rss(&args[2],args[3].parse().unwrap(),&args[4]),
        v @ ("geometry"|"admissions")=>geometry(v,&args[2]),_=>unreachable!(),
    }
}
