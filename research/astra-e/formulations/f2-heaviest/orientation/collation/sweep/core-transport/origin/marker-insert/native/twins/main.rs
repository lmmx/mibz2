#[inline(never)]
fn invoke(w:&mut Worker,x:&Input,arm:usize)->Row{
    w.run(x,std::hint::black_box(arm),false)
}
fn main(){
    let path=std::env::args().nth(1).unwrap();let mut rd=BufReader::new(std::fs::File::open(path).unwrap());
    let mut blocks=Vec::new();
    while let Some(x)=Input::read(&mut rd){
        let expected=Truth::new(&x).output(x.p);blocks.push((x,expected));
    }
    assert_eq!(blocks.len(),10);
    let mut workers=[Worker::default(),Worker::default(),Worker::default()];
    let codes=[0,0,2];let names=["named_a","named_b","sparse4"];
    let orders=[[0,1,2],[0,2,1],[1,0,2],[1,2,0],[2,0,1],[2,1,0]];
    for (x,expected) in &blocks{for _ in 0..2{for j in 0..3{
        invoke(&mut workers[j],x,codes[j]);assert_eq!(&workers[j].out,expected);
    }}}
    println!("cluster,rep,block_position,stream,position,arm,setup_ns,discover_ns,merge_ns,publication_ns,second_index_ns,second_discover_ns,second_merge_ns,second_emit_ns,total_ns");
    let mut cluster_means=Vec::new();
    for cluster in 0..24{
        let mut log_sum=0f64;
        for turn in 0..6{
            let rep=6*cluster+turn;let mut totals=[0u64;3];
            for j in 0..10{
                let at=(rep*3+if cluster%2==0{j}else{10-j})%10;
                let (x,expected)=&blocks[at];
                for (pos,&arm) in orders[(turn+at+cluster)%6].iter().enumerate(){
                    let r=invoke(&mut workers[arm],x,codes[arm]);
                    assert_eq!(&workers[arm].out,expected);
                    totals[arm]+=r.total;
                    println!("{cluster},{rep},{j},{},{pos},{},{},{},{},{},{},{},{},{},{}",x.index,names[arm],r.first[0],r.first[1],r.first[2],r.publish,r.inner[0],r.inner[1],r.inner[2],r.inner[3],r.total);
                }
            }
            log_sum+=(totals[1] as f64/totals[0] as f64).ln();
        }
        cluster_means.push(log_sum/6.);
        if cluster==7{
            let mean=cluster_means.iter().sum::<f64>()/8.;
            let sd=(cluster_means.iter().map(|x|(x-mean)*(x-mean)).sum::<f64>()/7.).sqrt();
            let projected=2.0686576*sd/24f64.sqrt();
            eprintln!("48-round control-only projected half-width {projected:.8}, threshold {:.8}",1.08f64.ln());
            if projected>=1.08f64.ln(){eprintln!("stopped at 48 under frozen control-only rule");return;}
        }
    }
    eprintln!("completed all 144 rounds and 4320 byte-checked observations");
}
