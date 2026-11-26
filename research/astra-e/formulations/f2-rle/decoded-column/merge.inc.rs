#[derive(Default)]
struct Visits { calls:u64, bytes:u64, anchor_exits:u64, maximum:usize }
fn comparison(mut x:usize,mut y:usize, labels:&[u8],anchor:&[usize], log:&mut Visits)->std::cmp::Ordering {
    log.calls+=1;
    for k in 0..=labels.len() {
        if x==y{return std::cmp::Ordering::Equal;}
        if anchor[x]!=usize::MAX && anchor[y]!=usize::MAX {
            log.anchor_exits+=1;log.maximum=log.maximum.max(k);
            return anchor[x].cmp(&anchor[y]);
        }
        log.bytes+=1;
        let result=labels[x].cmp(&labels[y]);
        if result!=std::cmp::Ordering::Equal {log.maximum=log.maximum.max(k+1);return result;}
        x=if x+1==labels.len(){0}else{x+1};
        y=if y+1==labels.len(){0}else{y+1};
    }
    panic!("periodic decoded rotations require explicit tie policy");
}
// Search monotonically out from the projected old rank, then binary-search that bracket.
fn gallop(n:usize,predict:usize,mut lower:impl FnMut(usize)->bool)->usize {
    let mut lo=predict;let mut hi=predict;let mut step=1;
    if predict<n && lower(predict) {
        loop {
            lo=hi+1;hi=hi.saturating_add(step).min(n);
            if hi==n || !lower(hi) {break;}
            step*=2;
        }
    } else {
        while lo>0 {
            let next=lo.saturating_sub(step);
            if lower(next){lo=next+1;break;}
            hi=lo;lo=next;step*=2;
        }
    }
    while lo<hi {let mid=lo+(hi-lo)/2;if lower(mid){lo=mid+1;}else{hi=mid;}}
    lo
}
fn merge(raw:&[u8],new:&[u8],raw_rows:&[usize],map:&[usize],vuln:&[bool],counts:&[usize],new_rank:&[usize]) -> (usize,u64,u64,u64,usize,usize,usize) {
    let mut anchors=Vec::new();
    let mut old_rank=vec![usize::MAX;new.len()];
    let mut anchor_rank=vec![usize::MAX;new.len()];
    for (i,&j) in map.iter().enumerate(){
        if j==usize::MAX {continue;}
        old_rank[j]=raw_rows[i];
        if !vuln[i] {anchors.push(j);anchor_rank[j]=raw_rows[i];}
    }
    anchors.sort_unstable_by_key(|&j|anchor_rank[j]); // dense oracle; sparse implementation selects from old ranks excluding removed rows
    for &i in counts {
        let first=map[i-1]+1;
        for j in first..first+usize::from(raw[i]) {old_rank[j]=raw_rows[i-1];}
    }
    let mut visits=Visits::default();
    let mut assigned=Vec::new();
    for j in 0..new.len(){
        if anchor_rank[j]!=usize::MAX{continue;}
        let start=anchors.partition_point(|&x|anchor_rank[x]<old_rank[j]);
        let at=gallop(anchors.len(),start,|x| comparison(anchors[x],j,new,&anchor_rank,&mut visits).is_lt());
        assigned.push((at,j));
    }
    assigned.sort_unstable_by(|&(a,x),&(b,y)|a.cmp(&b).then_with(||comparison(x,y,new,&anchor_rank,&mut visits)));
    let mut i=0;let mut emitted=0;let mut chunks=0;let mut previous=None;let mut bwt_runs=0;
    for at in 0..=anchors.len() {
        let mut append=|j:usize| {
            assert_eq!(new_rank[j],emitted);
            let b=new[(j+new.len()-1)%new.len()];
            bwt_runs+=usize::from(previous!=Some(b));previous=Some(b);emitted+=1;
        };
        if assigned.get(i).is_some_and(|z|z.0==at){chunks+=1;}
        while assigned.get(i).is_some_and(|z|z.0==at) {append(assigned[i].1);i+=1;}
        if at<anchors.len(){append(anchors[at]);}
    }
    assert_eq!(emitted,new.len());
    (assigned.len(),visits.calls,visits.bytes,visits.anchor_exits,visits.maximum,chunks,bwt_runs)
}
