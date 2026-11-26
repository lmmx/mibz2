fn check_size(parts:&[(usize,u32,u32)],words:&[(u32,Vec<u8>)],m:u32,skip:&[bool])->usize{
    let mut out=0;let mut previous=None;
    let mut at=0;let mut j=0;
    for &(w,n,lf) in parts {
        let end=at+n;let mut pos=at;
        while pos<end{
            let next=if j+1<words.len(){words[j+1].0}else{m};
            let stop=end.min(next);
            if !skip[j]{
                let x=lf+pos-at;
                if previous!=Some((w,x)){out+=1;}
                previous=Some((w,x+stop-pos));
            }
            pos=stop;if pos==next{j+=1;}
        }
        at=end;
    }
    assert_eq!(at,m);out
}
fn held(path:&str){
    let mut reader=BufReader::new(std::fs::File::open(path).unwrap());let mut count=0;
    let mut ws:Vec<_>=(0..2).map(|_|Worker::default()).collect();
    println!("stream,h,root_only_records,literal_records,all_branch_records,actual_records,branch_sources,literal_present,literal_in_root,root_base,literal_base");
    while let Some(x)=Input::read(&mut reader){
        let oracle=Truth::new(&x).output(x.p);
        for (arm,route) in [15,17].into_iter().enumerate(){
            let w=&mut ws[arm];let r=w.run(&x,route,2);assert!(r.second && w.out==oracle);
            let (words,raw)=r.obj.unwrap();
            let index:HashMap<u32,usize>=words.iter().enumerate().map(|(i,(x,_))|(*x,i)).collect();
            let base=r.winner;let root=index[&base];let m=r.tokens;let k=words.len();
            let mut min=vec![m;k];let mut max=vec![0;k];let mut freq=vec![0;k];let mut starts=vec![0;k];let mut parts=Vec::new();
            let mut at=0;let mut j=0;
            for &(key,n) in &raw{
                let c=index[&key];
                min[c]=min[c].min(at);max[c]=at+n;
                parts.push((c,n,key+freq[c]));freq[c]+=n;
                while j+1<k && words[j+1].0<=at{j+=1;}
                if at!=words[j].0{starts[j]+=1;}
                at+=n;
            }
            let mut branch=(0..k).map(|c| c!=root && min[c]>=base && max[c]<=base+freq[root] && starts[c]!=0).collect::<Vec<_>>();
            let opts=branch.iter().filter(|&&take|take).count();
            branch[root]=true;
            let candidates=words.iter().enumerate().filter(|(_,(_,b))|b.as_slice()==b"}").map(|(i,_)|i).collect::<Vec<_>>();
            assert!(candidates.len()<=1);
            let child=candidates.first().copied();
            let contained=child.is_some_and(|i|min[i]>=base && max[i]<=base+freq[root]);
            let mut rootmask=vec![false;k];rootmask[root]=true;
            let mut fixed=rootmask.clone();
            if r.counts[0]==2 && r.counts[5]==base {
                if let Some(i)=child {assert!(contained);fixed[i]=true;}
            }
            let a=check_size(&parts,&words,m,&rootmask);
            let b=check_size(&parts,&words,m,&fixed);
            let c=check_size(&parts,&words,m,&branch);
            let actual=w.fast.forest_stats()[1];assert_eq!(actual,b);
            println!("{},{h},{a},{b},{c},{actual},{opts},{},{contained},{base},{}",x.index,child.is_some(),child.map_or(u32::MAX,|v|words[v].0),h=if arm==0{4}else{12});
        }
        count+=1;
    }
    eprintln!("{count} intrinsic held-out admissions, both depths matched dense bytes and independent rank-record counts");
}
