fn forest_metrics(path:&str){
    let mut input=BufReader::new(std::fs::File::open(path).unwrap());
    let mut ws:Vec<_>=(0..6).map(|_|Worker::default()).collect();
    let codes=[0,12,3,15,5,17];
    let names=["named","named_forest","quot4","quot4_forest","quot12","quot12_forest"];
    println!("stream,arm,certified,rank_records,rank_pages,physical_nodes,selectors,merged,actual_discovery_finds,second_words,spelling_bytes");
    while let Some(x)=Input::read(&mut input){
        let expected=Truth::new(&x).output(x.p);
        for i in 0..6{
            let result=ws[i].run(&x,codes[i],0);
            assert!(result.second && ws[i].out==expected);
            let s=ws[i].fast.forest_stats();
            println!("{},{},{},{},{},{},{},{},{},{},{}",x.index,names[i],s[0],s[1],s[2],s[3],s[4],s[5],s[6],result.info[2],result.info[3]);
        }
    }
}
