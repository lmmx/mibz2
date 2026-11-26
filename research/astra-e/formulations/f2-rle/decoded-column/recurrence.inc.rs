fn recurrence(raw:&[u8],new:&[u8],rows:&[usize],map:&[usize],vuln:&[bool],counts:&[usize],true_rank:&[usize],index:u64) {
    let n=raw.len();
    let m=new.len();
    let mut rank=vec![usize::MAX;m];
    let mut anchors=Vec::new();
    let mut old_safe=vec![false;n];
    for i in 0..n {
        if map[i]!=usize::MAX && !vuln[i] {
            rank[map[i]]=rows[i];anchors.push(map[i]);old_safe[i]=true;
        }
    }
    anchors.sort_unstable_by_key(|&j|rank[j]);
    let mut gap=vec![usize::MAX;m];let mut base=[0;256];
    for (i,&j) in anchors.iter().enumerate(){gap[j]=i;base[new[j] as usize]+=1;}
    let mut sum=0;for b in &mut base {let t=*b;*b=sum;sum+=t;}
    let mut original:[Vec<usize>;256]=std::array::from_fn(|_|Vec::new());
    let mut exclude:[Vec<usize>;256]=std::array::from_fn(|_|Vec::new());
    for i in 0..n {
        let prev=(i+n-1)%n;let c=raw[prev] as usize;
        original[c].push(rows[i]);if !old_safe[i]||!old_safe[prev] {exclude[c].push(rows[i]);}
        else{assert_eq!((map[prev]+1)%m,map[i]);}
    }
    for c in 0..256 {original[c].sort_unstable();exclude[c].sort_unstable();}
    let mut exceptional:Vec<_>=(0..m).filter(|&j|rank[j]==usize::MAX).collect();
    let mut boundary:[Vec<usize>;256]=std::array::from_fn(|_|Vec::new());
    let mut endpoints=Vec::new();
    for &a in &anchors {
        let next=(a+1)%m;
        if rank[next]==usize::MAX {boundary[new[a] as usize].push(next);}
        if rank[(a+m-1)%m]==usize::MAX {endpoints.push(a);}
    }
    assert!(!anchors.is_empty());
    let mut visits=Visits::default();
    for group in &mut boundary {group.sort_unstable_by(|&a,&b|comparison(a,b,new,&rank,&mut visits));}
    for &end in &endpoints {
        let mut succ=end;let mut j=(succ+m-1)%m;
        while rank[j]==usize::MAX {
            let c=new[j] as usize;
            let cut=anchors.get(gap[succ]).map_or(n,|&a|rank[a]);
            let normal=original[c].partition_point(|&r|r<cut)-exclude[c].partition_point(|&r|r<cut);
            let other=boundary[c].partition_point(|&z|comparison(z,succ,new,&rank,&mut visits).is_lt());
            gap[j]=base[c]+normal+other;
            succ=j;j=(j+m-1)%m;
        }
    }
    let v=exceptional.len();
    exceptional.extend_from_slice(&endpoints);
    let size=exceptional.len();
    let mut local=vec![usize::MAX;m];
    for (i,&j) in exceptional.iter().enumerate(){local[j]=i;}
    let mut jump=vec![0;size];
    let mut keys=vec![(0,0);size];
    for (i,&j) in exceptional.iter().enumerate(){
        assert_ne!(gap[j],usize::MAX);
        let safe=rank[j]!=usize::MAX;
        jump[i]=if safe{i}else{local[(j+1)%m]};
        keys[i]=(2*gap[j]+usize::from(safe),usize::from(new[j]));
    }
    let mut order:Vec<_>=(0..size).collect();
    let mut cls=vec![0;size];let mut rounds=0;
    while !order.is_empty() {
        order.sort_unstable_by_key(|&i|keys[i]);
        let mut classes=1;cls[order[0]]=0;
        for k in 1..size{classes+=usize::from(keys[order[k-1]]!=keys[order[k]]);cls[order[k]]=classes-1;}
        if classes==size {break;}
        assert!(rounds<32);
        for i in 0..size {keys[i]=(cls[i],cls[jump[i]]);}
        jump=(0..size).map(|i|jump[jump[i]]).collect();rounds+=1;
    }
    let mut ordered=order.iter().filter_map(|&i|(i<v).then_some(exceptional[i])).peekable();let mut output=0;
    for i in 0..=anchors.len(){
        while ordered.peek().is_some_and(|&j|gap[j]==i){let j=ordered.next().unwrap();assert_eq!(true_rank[j],output);output+=1;}
        if i<anchors.len(){assert_eq!(true_rank[anchors[i]],output);output+=1;}
    }
    assert_eq!(output,m);
    println!("{index},{n},{},{v},{},{},{},{},{},{rounds}",counts.len(),endpoints.len(),visits.calls,visits.bytes,visits.anchor_exits,visits.maximum);
}
