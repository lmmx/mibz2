fn main(){
    let args=std::env::args().collect::<Vec<_>>();let mode=args[1].as_str();
    let mut input=BufReader::new(std::fs::File::open(&args[2]).unwrap());let mut inputs=Vec::new();
    while let Some(x)=Input::read(&mut input){inputs.push(x);}
    if mode=="fresh"{fresh(&inputs,&args[3]);return;}
    let mut named=Engine::default();let mut folded=Engine::default();let mut policy=HashMap::<u32,Vec<u32>>::new();
    if mode=="apply"{
        for l in std::fs::read_to_string(&args[3]).unwrap().lines().skip(1){
            let (i,b)=l.split_once(',').unwrap();policy.entry(i.parse().unwrap()).or_default().push(b.parse().unwrap());
        }
        println!("stream,selected,removed,old_runs,new_runs,appended,old_q,new_q,old_dictionary,new_dictionary,links,mark_moved");
    }else if mode=="price"{println!("rep,stream,root_ns,eligible_ns,index_ns,geometry_ns,table_ns,total_ns,states,pieces,incidence,eligible_incidence,visited_rows,eligible_states");}
    else{assert_eq!(mode,"export");println!("kind,stream,values");}
    let rounds=if mode=="price"{13}else{1};
    for rep in 0..rounds{
        for j in 0..inputs.len(){
            let x=&inputs[(j+3*rep)%inputs.len()];
            if mode=="apply"{
                let truth=Truth::new(x);let bytes=truth.output(x.p);
                let a=named.run(x,0,false);let mut chosen=policy.remove(&x.index).unwrap_or_default();
                chosen.sort_unstable();folded.fast.set_policy(chosen);
                let b=folded.run(x,3,true);dense(&truth,x,&b,false);
                assert_eq!(named.out,bytes);assert_eq!(folded.out,bytes);
                assert_eq!(named.fast.snapshot_second(a.mark,a.ran),folded.fast.snapshot_second(b.mark,b.ran));
                println!("{},{},{},{},{},{},{},{},{},{},{},{}",x.index,b.stats[2],b.stats[3],b.stats[4],b.stats[6],b.stats[7],
                         a.info[0],b.info[0],a.info[3],b.info[3],b.info[6],b.stats[8]);continue;
            }
            let t=now();let (_,m)=named.fast.first(&x.caps,&x.freq,x.n,x.p).unwrap();let start=now();
            let f=named.fast.census(m);let total=now()-t;let s=f.sizes;let ns=f.cpu;
            if mode=="export"{
                for (k,n,g,w) in f.types{println!("T,{},{k},{n},{g},{w}",x.index);}
                for e in f.support{
                    print!("S,{}",x.index);for k in e{print!(",{k}");}println!();
                }
            }else if rep!=0{
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{}",rep-1,x.index,start-t,ns[0],ns[1],ns[2],ns[3],total,s[0],s[1],s[2],s[3],s[4],f.support.len());
            }
        }
    }
}
