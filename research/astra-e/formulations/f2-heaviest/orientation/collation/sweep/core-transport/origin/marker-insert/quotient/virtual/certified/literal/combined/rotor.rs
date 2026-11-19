const ROSTER:[usize;8]=[9,15,27,39,11,17,29,41];
const POLICY:[&str;8]=["h4_explicit","h4_child","h4_direct","h4_both","h12_explicit","h12_child","h12_direct","h12_both"];
fn price(path:&str){
    let mut input=BufReader::new(std::fs::File::open(path).unwrap());let mut blocks=Vec::new();
    while let Some(x)=Input::read(&mut input){let out=Truth::new(&x).output(x.p);blocks.push((x,out));}
    assert_eq!(blocks.len(),10);
    let mut ws:Vec<_>=(0..8).map(|_|Worker::default()).collect();
    for (x,y) in &blocks {for (arm,w) in ws.iter_mut().enumerate(){w.run(x,ROSTER[arm],0);assert_eq!(&w.out,y);}}
    println!("cluster,round,slot,stream,arm,publish_ns,index_ns,discovery_ns,second_merge_ns,second_output_ns,whole_ns,post_first_ns");
    for rep in 0..64{let cluster=rep/8;
        for step in 0..10 {
            let block=(3*rep+if cluster%2==0{step}else{10-step})%10;
            let (x,y)=&blocks[block];
            for (slot,j) in [0,1,7,2,6,3,5,4].into_iter().enumerate(){
                let i=(rep%8+block+cluster+if cluster%2==0{j}else{8-j})%8;
                let r=ws[i].run(x,std::hint::black_box(ROSTER[i]),0);assert_eq!(&ws[i].out,y);
                let first=r.first.iter().sum::<u64>();
                println!("{cluster},{rep},{slot},{},{},{},{},{},{},{},{},{}",x.index,POLICY[i],r.publish,
                    r.inner[0],r.inner[1],r.inner[2],r.inner[3],r.total,r.total-first);
            }
        }
    }
    eprintln!("5120 complete output checks in 64 matched eight-arm rotations");
}
