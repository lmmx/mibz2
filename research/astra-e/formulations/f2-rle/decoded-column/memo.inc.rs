#[derive(Default)]
struct Visits {
    calls:u64, bytes:u64, anchor_exits:u64, maximum:usize,
    memo:std::collections::HashMap<(u32,u32),std::cmp::Ordering>,
    stack:Vec<(u32,u32,bool)>, hits:u64,
}
fn comparison(mut x:usize,mut y:usize,labels:&[u8],anchor:&[usize],log:&mut Visits)->std::cmp::Ordering {
    log.calls+=1;log.stack.clear();
    let result=loop {
        if x==y{break std::cmp::Ordering::Equal;}
        if anchor[x]!=usize::MAX&&anchor[y]!=usize::MAX {log.anchor_exits+=1;break anchor[x].cmp(&anchor[y]);}
        let rev=x>y;
        let key=(x.min(y) as u32,x.max(y) as u32);
        if let Some(&r)=log.memo.get(&key){log.hits+=1;break if rev {r.reverse()} else {r};}
        log.stack.push((key.0,key.1,rev));log.bytes+=1;
        let r=labels[x].cmp(&labels[y]);if r!=std::cmp::Ordering::Equal{break r;}
        x=if x+1==labels.len(){0}else{x+1};
        y=if y+1==labels.len(){0}else{y+1};
        assert!(log.stack.len()<=labels.len());
    };
    log.maximum=log.maximum.max(log.stack.len());
    for &(a,b,rev) in &log.stack {log.memo.insert((a,b),if rev {result.reverse()} else {result});}
    result
}
