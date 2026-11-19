mod splice;
use splice::*;
fn dense(obj:&Obj)->(Vec<u32>,Vec<u32>){
    let mut next=vec![u32::MAX;obj.m as usize];
    let mut label=next.clone();
    let mut pos=obj.bases.clone();
    let mut dst=0;
    for r in &obj.runs {
        let p=&mut pos[r.id as usize];
        for _ in 0..r.width {
            assert_eq!(next[*p as usize],u32::MAX);
            next[*p as usize]=dst;label[*p as usize]=r.id;*p+=1;dst+=1;
        }
    }
    (next,label)
}
fn word<'a>(obj:&Obj,d:&'a [u8],id:u32)->&'a [u8] {
    let w=obj.words[id as usize];
    &d[w.id as usize..(w.id+w.width)as usize]
}
fn output(obj:&Obj,d:&[u8],next:&[u32],ids:&[u32],mut x:u32,mut skip:u32,n:usize)->Vec<u8>{
    let mut raw=Vec::new();
    while raw.len()<n {
        let w=word(obj,d,ids[x as usize]);
        raw.extend_from_slice(&w[skip as usize..]);skip=0;x=next[x as usize];
    }
    raw.truncate(n);raw
}
fn main(){
    let max:usize=std::env::args().nth(1).map_or(7,|v|v.parse().unwrap());
    let (mut cases,mut marks,mut old_marks,mut moved,mut whole_deleted,mut changed,mut covered,mut inside)=(0,0,0,0,0,0,0,0);
    for k in 1..=3usize {
        for m in k..=max.min(if k==3{5}else{7}) {
            for seq in 0..k.pow(m as u32){
                let mut seq=seq;let mut counts=vec![0u32;k];let mut runs=Vec::<Piece>::new();
                for _ in 0..m {
                    let a=(seq%k)as u32;seq/=k;counts[a as usize]+=1;
                    if runs.last().is_some_and(|z|z.id==a){runs.last_mut().unwrap().width+=1;}else{runs.push(Piece{id:a,width:1});}
                }
                if counts.contains(&0){continue;}
                let mut bases=vec![0;k];let mut base=0;
                for a in (0..k).rev() {bases[a]=base;base+=counts[a];}
                let j=Obj{runs,words:vec![Piece{id:0,width:3},Piece{id:3,width:2},Piece{id:0,width:3}][..k].to_vec(),bases,m:m as u32};
                let (succ,labels)=dense(&j);
                for mask in 0..1<<m {
                    let mut data=b"abcXY".to_vec();
                    let holes=(0..m as u32).filter(|&i|mask&(1<<i)!=0).collect::<Vec<_>>();
                    let patch=splice(&j,&mut data,&holes);
                    let h=patch.obj.as_ref().unwrap_or(&j);
                    let (ns,nl)=dense(h);
                    let alive:Vec<_>=(0..m as u32).filter(|&i|mask&(1<<i)==0).collect();
                    for (local,&s) in alive.iter().enumerate() {
                        let mut x=s;let mut expected=Vec::new();
                        loop {
                            expected.extend_from_slice(word(&j,&data,labels[x as usize]));
                            x=succ[x as usize];if mask&(1<<x)==0{break;}
                        }
                        assert_eq!(ns[local],alive.binary_search(&x).unwrap() as u32);
                        assert_eq!(word(h,&data,nl[local]),expected);covered+=1;
                    }
                    for x in 0..m as u32 {
                        for skip in 0..word(&j,&data,labels[x as usize]).len() as u32 {
                            let bytes=3*m+1+(x as usize%3);
                            let expected=output(&j,&data,&succ,&labels,x,skip,bytes);
                            match patch.mark(x,skip) {
                                Mark::New(r,p)=>{
                                    assert_eq!(output(h,&data,&ns,&nl,r,p,bytes),expected);
                                    if mask&(1<<x)!=0{moved+=1;}if p!=0{inside+=1;}
                                }
                                Mark::Old(r,p)=>{
                                    assert_eq!((r,p),(x,skip));
                                    let mut at=x;
                                    loop{assert!(mask&(1<<at)!=0);at=succ[at as usize];if at==x{break;}}
                                    old_marks+=1;
                                }
                            }
                            marks+=1;
                        }
                    }
                    if patch.changes>0{changed+=1;}
                    if h.m==0{whole_deleted+=1;}
                    cases+=1;
                }
            }
        }
    }
    println!("{cases} sparse patches, {covered} independent direct induced partners and words, {marks} marked byte skips, {moved} transported deleted marks, {old_marks} certified deleted-only marked cycles, {inside} nonzero transported byte offsets, {changed} changed-live cases, {whole_deleted} empty retained sections");
}
