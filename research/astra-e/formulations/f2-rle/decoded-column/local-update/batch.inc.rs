fn audit_batch(raw:&[u8],rows:&[usize],map:&[usize],groups:&[usize],new:&[u8],expected:&[usize],index:u64) {
    let n=raw.len();let m=new.len();
    let mut original=vec![0;n];for (i,&r) in rows.iter().enumerate(){original[r]=i;}
    let mut active=Vec::new();let mut order=Vec::new();
    for x in original {
        if map[x]!=usize::MAX{order.push(map[x]);continue;}
        let j=map[x-1];active.push(j);
        for t in 1..=usize::from(raw[x]){order.push(j+t);active.push(j+t);}
    }
    let mut c=Column::new(new,order);
    let mut queries=0;let mut retained=0;let mut moves=0;let mut epochs=0;let mut displacement=0;
    while !active.is_empty(){
        assert!(epochs<=m);epochs+=1;queries+=active.len();
        let mut rows:Vec<_>=active.iter().map(|&x|{
            let ch=new[x];(x,c.position(x),c.base(ch)+c.rank(ch,c.succ[x]))
        }).collect();
        rows.sort_unstable_by_key(|&(_,a,_)|a);
        let mut targets:Vec<_>=rows.iter().map(|z|z.2).collect();targets.sort_unstable();
        let mut keep=vec![false;rows.len()];let mut preds=vec![usize::MAX;rows.len()];
        let mut i=0;
        while i<rows.len(){
            let old_gap=rows[i].1-i;
            let mut end=i+1;while end<rows.len()&&rows[end].1-end==old_gap{end+=1;}
            let mut tails:Vec<usize>=Vec::new();
            for j in i..end{
                let target=rows[j].2;
                if target-targets.partition_point(|&v|v<target)!=old_gap{continue;}
                let k=tails.partition_point(|&q|rows[q].2<target);
                if k>0{preds[j]=tails[k-1];}
                if k==tails.len(){tails.push(j)}else{tails[k]=j;}
            }
            if let Some(&last)=tails.last(){let mut j=last;while j!=usize::MAX{keep[j]=true;j=preds[j];}}
            i=end;
        }
        let mut change=Vec::new();active.clear();
        for (i,(x,old,target)) in rows.into_iter().enumerate(){
            if keep[i]{retained+=1;continue;}
            moves+=1;displacement+=old.abs_diff(target);change.push((target,x));active.push(c.pred[x]);c.remove(x);
        }
        change.sort_unstable();
        for (target,x) in change {c.place(x,target);}
    }
    let mut emitted=0;for p in &c.pages{for &x in p{assert_eq!(expected[x],emitted);emitted+=1;}}
    assert_eq!(emitted,m);
    println!("{index},{n},{},{epochs},{queries},{retained},{moves},{displacement}",groups.len());
}
