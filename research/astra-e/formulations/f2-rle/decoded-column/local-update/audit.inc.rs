fn audit_update(raw:&[u8],rows:&[usize],edits:&[usize],new_rank:&[usize],index:u64) {
    let n=raw.len();let mut order=vec![0;n];for (i,&r) in rows.iter().enumerate(){order[r]=i;}
    let mut c=Column::new(raw,order);
    for &i in edits {
        let k=raw[i] as usize;let b=raw[i-1];let j=c.succ[i];
        if k==0{assert!(c.delete(i));}else{
            assert!(c.replace(i,b));for _ in 1..k{assert!(c.insert_before(j,b));}
        }
    }
    let mut decoded_id=vec![usize::MAX;c.byte.len()];let mut p=0;
    for i in 0..c.n{decoded_id[p]=i;p=c.succ[p];}assert_eq!(p,0);
    let mut out=0;for page in &c.pages{for &id in page{assert_eq!(new_rank[decoded_id[id]],out);out+=1;}}
    assert_eq!(out,new_rank.len());
    println!("{index},{n},{},{},{},{},{}",edits.len(),c.operations,c.moves,c.longest,c.displacement);
}
