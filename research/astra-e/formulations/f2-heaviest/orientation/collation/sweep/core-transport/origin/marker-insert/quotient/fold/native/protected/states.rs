// Independent census in the original token coordinates. This diagnostic is
// outside every paid call and sees no folded object while predicting its Q.
fn state_census(obj:&Obj, doomed:&std::collections::HashSet<u32>)->[u32;5]{
    let mut incoming=Vec::new();
    let mut lf=Vec::new();
    let mut freq=HashMap::<u32,u32>::new();
    for &(k,w) in &obj.1{
        let count=freq.entry(k).or_default();
        for i in 0..w{incoming.push(k);lf.push(k+*count+i);}
        *count+=w;
    }
    let (&d,&mass)=freq.iter().max_by_key(|(k,n)|(**n,u32::MAX-**k)).unwrap();
    let mut queue=vec![(d,d+mass)];
    let mut covered=vec![false;incoming.len()];
    let mut i=0;let mut removed_states=0;let mut deleted_rows=0;let mut mixed_rows=0;
    while i<queue.len(){
        let (lo,hi)=queue[i];i+=1;
        let mut next=HashMap::<u32,(u32,u32)>::new();let mut dead=0;
        for x in lo as usize..hi as usize{
            assert!(!covered[x]);covered[x]=true;
            let k=incoming[x];dead+=u32::from(doomed.contains(&k));
            if k==d{continue;}
            let v=next.entry(k).or_insert((lf[x],lf[x]));
            assert_eq!(v.1,lf[x]);
            v.1+=1;
        }
        deleted_rows+=dead;
        if dead==hi-lo{removed_states+=1;} else{mixed_rows+=dead;}
        queue.extend(next.into_values());
    }
    [queue.len() as u32,removed_states,deleted_rows,mixed_rows,covered.iter().filter(|&&x|x).count() as u32]
}
