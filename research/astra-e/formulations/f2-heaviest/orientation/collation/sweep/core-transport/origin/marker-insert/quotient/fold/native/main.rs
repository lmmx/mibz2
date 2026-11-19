const ARMS:[&str;4]=["named_a","named_b","recognize","fold"];
#[derive(Default)]
struct Engine{fast:control::Dispatch<2,2>,out:Vec<u8>}
struct Row{
    mark:Option<(u32,u32)>,tokens:u32,object:Option<Obj>,selection:Vec<(u32,u32,u32,bool)>,
    first:[u64;3],construction:[u64;3],pub_ns:u64,second:[u64;4],total:u64,
    stats:[u32;10],info:[usize;7],ran:bool,cap:usize
}

impl Engine {
    #[inline(never)]
    fn run(&mut self,x:&Input,mode:usize,inspect:bool)->Row{
        let t0=now();
        let (mut mark,mut m)=self.fast.first(&x.caps,&x.freq,x.n,x.p).unwrap();
        let mut construction=[0;3];let mut pub_ns=0;let mut stats=[0;10];
        if mode>=2{
            let start=now();
            (mark,m,stats,construction)=self.fast.fold_one(mark,m,mode==3);
            pub_ns=now()-start;
        }
        let object=inspect.then(||self.fast.object());
        let selection=if inspect && mode>=2 {self.fast.fold_selection()}else{vec![]};
        let ran=self.fast.finish(mark,m,x.n,x.p,&mut self.out);
        let total=now()-t0;
        Row{mark,tokens:m,object,selection,first:self.fast.phases,construction,pub_ns,second:if ran{self.fast.second_phases()}else{[0;4]},
            total,stats,info:if ran{self.fast.information()}else{[0;7]},ran,cap:self.fast.whole_capacity()+self.out.capacity()}
    }
}

fn dense(truth:&Truth,x:&Input,r:&Row,brute:bool){
    let (words,runs)=truth.object(x.quote(),true);
    let m=x.freq[34] as usize;
    let id=words.iter().enumerate().map(|(k,(b,_))|(*b,k)).collect::<HashMap<_,_>>();
    let mut owners=vec![0;m];
    for (k,(b,_)) in words.iter().enumerate(){
        let end=words.get(k+1).map_or(m,|v|v.0 as usize);
        owners[*b as usize..end].fill(k);
    }
    let mut seen=vec![0;words.len()];
    let mut next=vec![usize::MAX;m];let mut prev=next.clone();let mut incoming=vec![];
    for (base,w) in runs {
        let k=id[&base];
        for _ in 0..w{
            let source=base as usize+seen[k];seen[k]+=1;
            let dest=incoming.len();next[source]=dest;prev[dest]=source;incoming.push(k);
        }
    }
    let mut selected=vec![false;words.len()];
    let mut succs=vec![None;words.len()];
    for (k,&(b,f,g,take)) in r.selection.iter().enumerate(){
        assert_eq!((b,f),(words[k].0,seen[k] as u32));
        let rows=&next[b as usize..(b+f)as usize];
        let y=owners[rows[0]];
        let eligible=y!=k && rows.iter().all(|&x|owners[x]==y);
        assert_eq!(g,if eligible{words[y].0}else{u32::MAX});
        assert!(!take||eligible);
        selected[k]=take;succs[k]=eligible.then_some(y);
    }
    assert_eq!(r.selection.len(),words.len());
    assert!(selected.iter().enumerate().all(|(i,&on)|!on||!selected[succs[i].unwrap()]));
    if brute{
        let optimum=(0..(1usize<<words.len())).filter_map(|bits|{
            let mut w=0;
            for (k,g) in succs.iter().enumerate(){
                if bits & (1<<k)==0{continue;}
                let target=(*g)?;
                if bits & (1<<target)!=0{return None;}
                w+=seen[k];
            }
            Some(w)
        }).max().unwrap();
        assert_eq!(optimum,r.stats[3] as usize);
    }
    let keep=(0..m).filter(|&i|!selected[incoming[i]]).collect::<Vec<_>>();
    let mut embed=vec![usize::MAX;m];let mut bases=vec![usize::MAX;words.len()];
    for (v,&i) in keep.iter().enumerate(){
        embed[i]=v;bases[owners[i]]=bases[owners[i]].min(v);
    }
    let mut dictionary=vec![];let mut labels=vec![u32::MAX;keep.len()];
    for (k,(_,w)) in words.iter().enumerate(){
        if bases[k]==usize::MAX{continue;}
        let mut word=w.clone();
        if selected[k]{word.extend_from_slice(&words[succs[k].unwrap()].1);}
        dictionary.push((bases[k] as u32,word));
    }
    for &x in &keep{
        let k=owners[x];let y=next[x];let dest=if selected[k] {next[y]}else{y};
        let i=embed[dest];assert_eq!(labels[i],u32::MAX);labels[i]=bases[k] as u32;
    }
    let mut compressed=Vec::<(u32,u32)>::new();
    for k in labels{
        if let Some((last,w))=compressed.last_mut(){if *last==k{*w+=1;continue;}}
        compressed.push((k,1));
    }
    let mark=truth.mark(x.p,x.quote(),true).map(|(at,skip)|{
        let i=at as usize;
        if embed[i]!=usize::MAX{(embed[i] as u32,skip)}else{
            let p=prev[i];(embed[p] as u32,skip+words[owners[p]].1.len() as u32)
        }
    });
    assert_eq!(r.mark,mark);
    assert_eq!(r.tokens as usize,keep.len());
    assert_eq!(r.object.as_ref().unwrap(),&(dictionary,compressed));
}

fn small(){
    let graphs=control::small_optimizer();
    let mut w=Engine::default();let mut cnt=[0;5];let alphabet=[b'"',b',',b'x'];
    for n in 1..=9{
        let k=if n<=6{3usize}else{2};
        for mut mask in 0..k.pow(n){
            let caps=(0..n).map(|_|{let b=alphabet[mask%k];mask/=k;(u16::from(b)<<8)|1}).collect();
            let mut x=Input::new(0,0,caps);if x.freq[34]==0{continue;}
            let truth=Truth::new(&x);
            for p in 0..n{x.p=p;let r=w.run(&x,3,true);dense(&truth,&x,&r,true);
                assert_eq!(w.out,truth.output(p));cnt[0]+=1;cnt[1]+=u32::from(r.stats[3]!=0);
                cnt[2]+=r.stats[8];cnt[3]+=u32::from(r.mark.is_none());cnt[4]+=u32::from(r.stats[5]!=0);
            }
        }
    }
    println!("cases,folded,deleted_marks,disjoint_marks,empty_bucket_cases,brute_weighted_graphs");
    println!("{},{},{},{},{},{graphs}",cnt[0],cnt[1],cnt[2],cnt[3],cnt[4]);
}

fn cases(mode:&str,path:&str){
    let mut fd=BufReader::new(std::fs::File::open(path).unwrap());let mut n=0;
    let mut ws:Vec<_>=(0..4).map(|_|Engine::default()).collect();
    if mode=="selections"{println!("stream,base,mass,succ,canonical");}else{
        println!("stream,arm,rows,quote,tokens,eligible,eligible_mass,chosen,removed,old_runs,empty,new_runs,appended_dictionary,moved,lazy_chunks,second,lookups,words,second_dictionary,links,capacity");
    }
    while let Some(x)=Input::read(&mut fd){
        let truth=Truth::new(&x);let expected=truth.output(x.p);
        for (a,w) in ws.iter_mut().enumerate(){
            if mode=="selections" && a!=3{continue;}
            let r=w.run(&x,if a==1{0}else{a},a==3 && mode!="admissions");
            assert_eq!(w.out,expected);
            if a==3 && mode!="admissions"{dense(&truth,&x,&r,false);}
            if mode=="selections"{
                for (b,f,g,t) in r.selection{println!("{},{b},{f},{g},{}",x.index,u8::from(t));}
            }else{
                let s=r.stats;
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,ARMS[a],x.n,x.freq[34],r.tokens,
                    s[0],s[1],s[2],s[3],s[4],s[5],s[6],s[7],s[8],s[9],r.ran,r.info[0],r.info[2],r.info[3],r.info[6],r.cap);
            }
        }
        n+=1;
    }
    eprintln!("{n} blocks passed dense marked checks");
}

fn timing(path:&str){
    let mut fd=BufReader::new(std::fs::File::open(path).unwrap());let mut blocks=Vec::new();
    while let Some(x)=Input::read(&mut fd){let y=Truth::new(&x).output(x.p);blocks.push((x,y));}
    assert_eq!(blocks.len(),10);let mut ws:Vec<_>=(0..4).map(|_|Engine::default()).collect();let routes=[0,0,2,3];
    for (x,y) in &blocks{for (w,route) in ws.iter_mut().zip(routes){w.run(x,route,false);assert_eq!(&w.out,y);}}
    println!("cluster,rep,block_position,stream,position,arm,setup_ns,discover_ns,merge_ns,publication_ns,recognize_ns,dictionary_ns,weave_ns,second_index_ns,second_discover_ns,second_merge_ns,second_emit_ns,total_ns");
    for rep in 0..48{
        let c=rep/4;
        for step in 0..10{
            let ix=(3*rep+if c%2==0{step}else{10-step})%10;
            let (x,y)=&blocks[ix];
            for (p,j) in [0,1,3,2].into_iter().enumerate(){
                let a=(rep%4+ix+c+if c%2==0{j}else{4-j})%4;
                let w=&mut ws[a];let r=w.run(x,std::hint::black_box(routes[a]),false);assert_eq!(&w.out,y);
                let s=r.second;let t=r.construction;
                println!("{c},{rep},{step},{},{p},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,ARMS[a],r.first[0],r.first[1],r.first[2],
                    r.pub_ns,t[0],t[1],t[2],s[0],s[1],s[2],s[3],r.total);
            }
        }
    }
    eprintln!("1920 measured exact outputs, twelve balanced chronological clusters");
}

fn rss(path:&str,a:usize,hashes:&str){
    let expected=std::fs::read_to_string(hashes).unwrap().lines().skip(1).map(|s|{
        let (a,b)=s.split_once(',').unwrap();(a.parse::<u32>().unwrap(),b.parse::<u64>().unwrap())
    }).collect::<HashMap<_,_>>();
    let mut fd=BufReader::new(std::fs::File::open(path).unwrap());let mut w=Engine::default();let mut n=0;
    println!("stream,arm,retained_bytes,hwm_kib");
    while let Some(x)=Input::read(&mut fd){
        let r=w.run(&x,if a==1{0}else{a},false);
        let h=w.out.iter().fold(0xcbf29ce484222325u64,|v,&b|(v^u64::from(b)).wrapping_mul(0x100000001b3));
        assert_eq!(h,expected[&x.index]);
        let mem=std::fs::read_to_string("/proc/self/status").unwrap();
        let hwm=mem.lines().find(|s|s.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap();
        println!("{},{},{},{}",x.index,ARMS[a],r.cap,hwm);n+=1;
    }
    assert_eq!(n,228);
}
fn main(){let args=std::env::args().collect::<Vec<_>>();
    if args.len()==1{small();return;}
    match args[1].as_str(){
        "price"=>timing(&args[2]),"rss"=>rss(&args[2],args[3].parse().unwrap(),&args[4]),
        mode @ ("geometry"|"admissions"|"selections")=>cases(mode,&args[2]),_=>unreachable!(),
    }
}
