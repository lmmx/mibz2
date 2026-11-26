fn name(arm:i32)->String {
    match arm {-2=>"compact_quote".into(),-1=>"compact_frequent".into(),h=>format!("core_h{h}")}
}
fn exact(x:&Input,dense:&Truth,u:(u32,u32),h:i32,sub:&mut Subject,expected:&[u8],obj:&Obj)->Receipt {
    let r=sub.run(x,u,h,true).unwrap();
    assert_eq!(r.obj.as_ref().unwrap(),obj);
    assert_eq!(r.mark,dense.mark(x.p,u,false));
    assert_eq!(sub.out,expected);
    assert_eq!(r.metrics[6] as usize,obj.1.len());
    r
}
fn small(){
    let mut cases=0;let mut disjoint=0;let mut fallback=0;let mut sub=Subject::default();
    for (alphabet,max_n) in [(2,7),(3,5)] {
        for n in 1..=max_n {for seed in 0..u32::pow(alphabet,n) {
            let mut z=seed;let caps=(0..n).map(|_|{let c=z%alphabet;z/=alphabet;((c<<8)|1) as u16}).collect();
            let mut x=Input::new(0,0,caps);let dense=Truth::new(&x);
            for lo in 0..n {for hi in lo+1..=n {
                let u=(lo,hi);
                let obj=dense.object(u,false);
                for h in [0,1,4,8] {for p in 0..n {
                    x.p=p;
                    let r=exact(&x,&dense,u,h,&mut sub,&dense.output(p),&obj);
                    cases+=1;disjoint+=u32::from(r.mark.is_none());fallback+=u32::from(!r.second);
                }}
            }}
        }}
    }
    println!("exact_marked_cases={cases} disjoint_cycles={disjoint} including_first_level_fallback={fallback}");
}
fn blocks(mode:&str,path:&str,reps:u32){
    let mut reader=BufReader::new(std::fs::File::open(path).unwrap());
    let arms=[-1,0,4,8];let mut blocks=0;
    if mode=="price"{println!("stream,rep,arm,setup_mark_ns,discover_ns,merge_ns,compose_ns,first_ns,total_ns,reserved_bytes,setup_extra_capacity_upper");}
    else{println!("stream,h,lo,hi,live_states,lf_pieces,forward_intersections,origin_bands,deleted_rows,overlay_slices,transport_segments,core_runs,words,dict_bytes,full_runs,mark,skip,second,reserved_bytes,setup_extra_capacity_upper");}
    while let Some(x)=Input::read(&mut reader) {
        let dense=Truth::new(&x);
        let expected=dense.output(x.p);
        let mut control=Subject::default();
        let q=control.run(&x,x.quote(),-2,true).unwrap();
        assert_eq!(control.out,expected);
        let u=highest(q.obj.as_ref().unwrap(),x.quote().0);
        let obj=dense.object(u,false);
        let mut subjects:Vec<_>=(0..arms.len()).map(|_|Subject::default()).collect();
        for (&arm,sub) in arms.iter().zip(&mut subjects) {
            if arm<0 {
                let _=sub.run(&x,u,arm,false).unwrap();assert_eq!(sub.out,expected);
                continue;
            }
            let r=exact(&x,&dense,u,arm,sub,&expected,&obj);
            if mode!="price" {
                let (mark,skip)=r.mark.unwrap_or((u32::MAX,0));
                let s=r.metrics;
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,arm,u.0,u.1,
                         s[0],s[1],s[2],s[3],s[4],s[5],s[6],s[7],obj.0.len(),obj.0.iter().map(|w|w.1.len()).sum::<usize>(),
                         obj.1.len(),mark,skip,r.second,r.cap,r.setup_scratch);
            }
        }
        if mode=="price" {
            for rep in 0..reps {for pos in [0,1,3,2] {
                let j=(rep as usize+pos)%arms.len();
                let r=subjects[j].run(&x,u,arms[j],false).unwrap();
                assert_eq!(subjects[j].out,expected);
                println!("{},{},{},{},{},{},{},{},{},{},{}",x.index,rep,name(arms[j]),r.phases[0],r.phases[1],r.phases[2],
                         r.compose_ns,r.first,r.total,r.cap,r.setup_scratch);
            }}
        }
        blocks+=1;
    }
    eprintln!("verified {blocks} input blocks, h=0,4,8");
}
// RSS
fn main(){
    let args=std::env::args().collect::<Vec<_>>();
    if args.len()==1 {small();}
    else if args[1]=="rss"{rss(&args[2],args[3].parse().unwrap(),&args[4],&args[5]);}
    else{blocks(&args[1],&args[2],args.get(3).map_or(32,|s|s.parse().unwrap()));}
}
