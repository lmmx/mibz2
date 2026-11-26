const ARMS:[&str;4]=["named_a","named_b","recognize","literal"];
#[derive(Default)]
struct Engine{fast:control::Dispatch<2,2>,out:Vec<u8>}
struct Row{pair:control::PairResult,first:[u64;3],second:[u64;4],info:[usize;7],total:u64,cap:usize}
impl Engine{
    #[inline(never)]
    fn run(&mut self,x:&Input,route:usize,depth:u8)->Row{
        let t=now();
        let (mark,m)=self.fast.first(&x.caps,&x.freq,x.n,x.p).unwrap();
        let pair=self.fast.literal_finish(mark,m,x.n,x.p,route,depth,&mut self.out);
        let total=now()-t;
        let (second,info)=if pair.stock {(self.fast.second_phases(),self.fast.information())}else{([0;4],[0;7])};
        Row{first:self.fast.phases,second,info,pair,total,cap:self.fast.whole_capacity()+self.out.capacity()}
    }
}

// Explicitly contract the dense source permutation under its original byte mark.
fn oracle(t:&Truth,x:&Input)->(Obj,(u32,u32)){
    let (old,runs)=t.object(x.quote(),true);
    let kb=old.iter().find(|e|e.1==b"item\"").unwrap().0 as usize;
    let jb=old.iter().find(|e|e.1==b",\"").unwrap().0 as usize;
    let m=x.freq[34] as usize;
    let mut seen=vec![0;m];let mut source=vec![];let mut next=vec![0;m];let mut prev=vec![0;m];
    let mut dead=vec![false;m];let mut original=vec![0;m];
    for (i,(base,_)) in old.iter().enumerate(){
        let end=old.get(i+1).map_or(m,|e|e.0 as usize);original[*base as usize..end].fill(*base as usize);
    }
    for (k,w) in runs{
        let k=k as usize;for _ in 0..w{
            let from=k+seen[k];seen[k]+=1;let to=source.len();source.push(k);
            next[from]=to;prev[to]=from;dead[to]=k==kb;
            if k==kb{assert_eq!(original[to],jb);}
        }
    }
    let keep=(0..m).filter(|&v|!dead[v]).collect::<Vec<_>>();
    let mut embed=vec![usize::MAX;m];let mut bases=embed.clone();
    for (i,&v) in keep.iter().enumerate(){embed[v]=i;bases[original[v]]=i.min(bases[original[v]]);}
    let mut weights=vec![];
    for (b,mut w) in old{
        let b=b as usize;if bases[b]==usize::MAX{continue;}
        if b==kb{w.extend_from_slice(b",\"");}
        weights.push((bases[b] as u32,w));
    }
    let mut incoming=vec![u32::MAX;keep.len()];
    for &v in &keep{
        let mut y=next[v];
        if dead[y]{y=next[y];}
        assert_eq!(incoming[embed[y]],u32::MAX);incoming[embed[y]]=bases[original[v]] as u32;
    }
    let mut rr=Vec::<(u32,u32)>::new();
    for k in incoming{if let Some(last)=rr.last_mut(){if last.0==k{last.1+=1;continue;}}rr.push((k,1));}
    let (old_row,old_skip)=t.mark(x.p,x.quote(),true).unwrap();let v=old_row as usize;
    let mark=if dead[v]{(embed[prev[v]] as u32,5+old_skip)}else{(embed[v] as u32,old_skip)};
    ((weights,rr),mark)
}
fn check(x:&Input,truth:&Truth,a:&mut Engine,b:&mut Engine,depth:u8)->(Row,Row){
    let r=a.run(x,0,1);let f=b.run(x,3,depth);let expected=truth.output(x.p);
    assert_eq!(a.out,expected);assert_eq!(b.out,expected);assert_eq!(r.pair.snapshot,f.pair.snapshot);
    if f.pair.applied && depth==2{assert_eq!(f.pair.first.as_ref().unwrap(),&oracle(truth,x));}
    (r,f)
}
fn artificial(parts:&[&[u8]],isolated:bool)->Input{
    let mut text=Vec::new();for part in parts{text.push(b'"');text.extend_from_slice(part);}
    let n=text.len();let mut ord=(0..n).collect::<Vec<_>>();
    ord.sort_by(|&a,&b|(0..n).map(|i|text[(a+i)%n]).cmp((0..n).map(|i|text[(b+i)%n])));
    let mut caps=ord.into_iter().map(|i|(u16::from(text[(i+n-1)%n])<<8)|1).collect::<Vec<_>>();
    if isolated{caps.extend(std::iter::repeat_n(0xfe01,3));}
    Input::new(0,0,caps)
}
fn disjoint_stock()->Input{
    let cycles:[&[u8];2]=[b"\"item\",\"x\"x\"x\"x",b"\"outside"];
    let mut rows=Vec::new();for (c,w) in cycles.iter().enumerate(){for i in 0..w.len(){rows.push((c,i));}}
    let horizon=cycles[0].len()*cycles[1].len();
    rows.sort_by(|&(a,i),&(b,j)|{
        (0..horizon).map(|t|cycles[a][(i+t)%cycles[a].len()]).cmp((0..horizon).map(|t|cycles[b][(j+t)%cycles[b].len()]))
    });
    let caps=rows.into_iter().map(|(c,i)|{
        let w=cycles[c];(u16::from(w[(i+w.len()-1)%w.len()])<<8)|1
    }).collect();
    Input::new(0,0,caps)
}
fn small(){
    let (mut a,mut b)=(Engine::default(),Engine::default());let mut reasons=[0u32;8];let mut moved=0;
    let mut cases=Vec::new();
    let alphabet=[b'"',b',',b'x'];
    for n in 1..=7{let radix=if n<=5{3usize}else{2};
        for mut i in 0..radix.pow(n){let caps=(0..n).map(|_|{let k=alphabet[i%radix];i/=radix;(u16::from(k)<<8)|1}).collect();
            let x=Input::new(0,0,caps);if x.freq[34]!=0{cases.push(x);}
        }
    }
    let tests:Vec<Vec<&[u8]>>=vec![
        vec![b"item",b",",b"x",b"x",b"x",b"void",b"x"],
        vec![b",",b"item",b",",b"zz",b"zz",b"zz",b"zz",b"item",b","],
        vec![b"xx",b"xx",b"xx",b"item",b",",b"item",b"other",b"xx"],
        vec![b"item",b"x",b"x",b"x"],vec![b"z",b",",b"z"],
        vec![b"item",b",",b"item",b","],
        vec![b"item",b",",b"more",b"item",b",",b"more"],
    ];
    for words in tests{
        for repeat in [1,2,3]{let mut v=Vec::new();for _ in 0..repeat{v.extend_from_slice(&words);}
            for isolated in [false,true]{cases.push(artificial(&v,isolated));}
        }
    }
    cases.push(disjoint_stock());
    for mut x in cases{let t=Truth::new(&x);
        for p in 0..x.n{x.p=p;let (_,r)=check(&x,&t,&mut a,&mut b,2);reasons[r.pair.reason as usize]+=1;moved+=u32::from(r.pair.moved);}
    }
    assert!(reasons[1..8].iter().all(|&n|n>0)&&moved>0);
    println!("case_total,absent_item,absent_comma,disjoint_quote,nondeterministic,missing_heavier,eligible,second_fallback,deleted_marks");
    println!("{},{},{},{},{},{},{},{},{}",reasons.iter().sum::<u32>(),reasons[1],reasons[2],reasons[3],reasons[4],reasons[5],reasons[6],reasons[7],moved);
}
fn read_all(path:&str)->Vec<Input>{
    let mut r=BufReader::new(std::fs::File::open(path).unwrap());let mut xs=Vec::new();while let Some(x)=Input::read(&mut r){xs.push(x);}xs
}
fn cases(path:&str,depth:u8){
    let (mut a,mut b)=(Engine::default(),Engine::default());
    println!("stream,rows,old_quote,reason,applied,removed,moved,old_runs,new_runs,old_q,new_q,old_dictionary,new_dictionary,old_links,new_links");
    let mut fd=BufReader::new(std::fs::File::open(path).unwrap());
    while let Some(x)=Input::read(&mut fd){let t=Truth::new(&x);let (r,f)=check(&x,&t,&mut a,&mut b,depth);let s=&f.pair;
        println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,x.n,x.freq[34],s.reason,s.applied,s.removed,s.moved,s.old_runs,s.new_runs,r.info[0],f.info[0],r.info[3],f.info[3],r.info[6],f.info[6]);}
}
fn price(path:&str){
    let blocks=read_all(path);assert_eq!(blocks.len(),12);
    let expected=blocks.iter().map(|x|Truth::new(x).output(x.p)).collect::<Vec<_>>();
    let mut w:Vec<_>=(0..4).map(|_|Engine::default()).collect();let routes=[0,0,2,3];
    for (i,x) in blocks.iter().enumerate(){for a in 0..4{w[a].run(x,routes[a],0);assert_eq!(w[a].out,expected[i]);}}
    println!("cluster,rep,block_position,stream,position,arm,setup_ns,discover_ns,merge_ns,eligible_ns,direct_index_ns,stock_index_mark_ns,second_discover_ns,second_merge_ns,second_emit_ns,total_ns");
    for rep in 0..48{let c=rep/4;
        for step in 0..12{
            let i=(5*rep+if c%2==0{step}else{12-step})%12;
            for (pos,j) in [0,1,3,2].into_iter().enumerate(){
                let a=(rep%4+i+c+if c%2==0{j}else{4-j})%4;
                let r=w[a].run(&blocks[i],std::hint::black_box(routes[a]),0);assert_eq!(w[a].out,expected[i]);
                println!("{c},{rep},{step},{},{pos},{},{},{},{},{},{},{},{},{},{},{}",blocks[i].index,ARMS[a],
                    r.first[0],r.first[1],r.first[2],r.pair.eligible_ns,r.pair.index_ns,r.second[0],r.second[1],r.second[2],r.second[3],r.total);
            }
        }
    }
}
fn rss(path:&str,a:usize,hashes:&str){
    let expect=std::fs::read_to_string(hashes).unwrap().lines().skip(1).map(|l|{let (k,v)=l.split_once(',').unwrap();
        (k.parse::<u32>().unwrap(),v.parse::<u64>().unwrap())}).collect::<HashMap<_,_>>();
    let mut rd=BufReader::new(std::fs::File::open(path).unwrap());let mut w=Engine::default();let mut n=0;
    println!("stream,arm,retained_bytes,hwm_kib");
    while let Some(x)=Input::read(&mut rd){
        let r=w.run(&x,[0,0,2,3][a],0);let h=w.out.iter().fold(0xcbf29ce484222325u64,|v,&b|(v^u64::from(b)).wrapping_mul(0x100000001b3));
        assert_eq!(h,expect[&x.index]);let s=std::fs::read_to_string("/proc/self/status").unwrap();let hwm=s.lines().find(|v|v.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap();
        println!("{},{},{},{}",x.index,ARMS[a],r.cap,hwm);n+=1;
    }
    assert_eq!(n,228);
}
fn main(){let a=std::env::args().collect::<Vec<_>>();if a.len()==1{small();return;}
    match a[1].as_str(){"price"=>price(&a[2]),"rss"=>rss(&a[2],a[3].parse().unwrap(),&a[4]),"check"=>cases(&a[2],2),_=>unreachable!()}}
