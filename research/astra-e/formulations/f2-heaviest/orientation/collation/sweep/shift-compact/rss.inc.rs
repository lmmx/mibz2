fn rss(path:&str,arm:i32,sections:&str,hashes:&str){
    let mut endpoints=HashMap::new();
    for s in std::fs::read_to_string(sections).unwrap().lines().skip(1){
        let c=s.split(',').collect::<Vec<_>>();
        if c[1]=="frequent"{endpoints.insert(c[0].parse::<u32>().unwrap(),(c[2].parse::<u32>().unwrap(),c[3].parse::<u32>().unwrap()));}
    }
    let mut digests=HashMap::new();
    for s in std::fs::read_to_string(hashes).unwrap().lines().skip(1){
        let (idx,h)=s.split_once(',').unwrap();digests.insert(idx.parse::<u32>().unwrap(),h.parse::<u64>().unwrap());
    }
    let mut rd=BufReader::new(std::fs::File::open(path).unwrap());
    let mut sub=Subject::default();let mut cases=0;
    println!("stream,arm,retained_bytes_with_output,process_hwm_kib");
    while let Some(x)=Input::read(&mut rd){
        let r=sub.run(&x,endpoints[&x.index],arm,false).unwrap();
        let h=sub.out.iter().fold(0xcbf29ce484222325u64,|h,&b|(h^u64::from(b)).wrapping_mul(0x100000001b3));
        assert_eq!(h,digests[&x.index]);
        let state=std::fs::read_to_string("/proc/self/status").unwrap();
        let hwm=state.lines().find(|s|s.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap();
        println!("{},{},{},{}",x.index,name(arm),r.cap,hwm);cases+=1;
    }
    assert_eq!(cases,228);
}
