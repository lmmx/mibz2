impl Origins{
    fn dense_quotient(&self,truth:&Truth,p:u32,h:u32)->(Obj,Option<(u32,u32)>,usize,usize){
        let Some(zid)=self.words.iter().position(|w|w.as_slice()==b",\"") else {
            let (o,m,s)=self.expected(truth,p,h);return (o,m,s as usize,0);
        };
        let sel=self.words.iter().map(|w|w.len()>=2 && w.len()<=h as usize+1 && w.ends_with(b",\"")).collect::<Vec<_>>();
        let m=self.names.len();let s=self.names.iter().filter(|&&k|sel[k]).count();
        let b=self.base[zid];let z=self.mass[zid];
        let mut which=(0..m).filter(|&i|sel[self.names[i]]).collect::<Vec<_>>();which.sort_unstable_by_key(|&i|self.dest[i]);
        let mut marker=vec![usize::MAX;m];
        for (j,&i) in which.iter().enumerate(){marker[i]=b+j;}
        let old_at=|i:usize|if i<b{i}else if i>=b+z{i+s-z}else{marker[i]};
        let mut words=Vec::new();
        for (k,original) in self.words.iter().enumerate(){
            let base=if self.base[k]>=b+z{self.base[k]+s-z}else{self.base[k]};
            let length=if sel[k]&&k!=zid{original.len()-2}else{original.len()};
            words.push((base as u32,original[..length].to_vec()));
        }
        let mut incoming=vec![None;m+s-z];
        let mut assign=|x:usize,k:usize|assert!(incoming[x].replace(words[k].0).is_none());
        for i in 0..m{
            let k=self.names[i];let landing=old_at(self.dest[i]);
            if sel[k]{
                if k!=zid{assign(marker[i],k);}
                assign(landing,zid);
            }else{assign(landing,k);}
        }
        let mut root=Vec::<(u32,u32)>::new();
        for entry in incoming{
            let k=entry.unwrap();
            if let Some((last,width))=root.last_mut(){if *last==k{*width+=1;continue;}}
            root.push((k,1));
        }
        let mark=truth.mark(p,(self.a,self.end),true).map(|(r,skip)|{
            let i=r as usize;let k=self.names[i];
            if sel[k]&&k!=zid && skip as usize>=self.words[k].len()-2{
                (marker[i] as u32,skip-(self.words[k].len()-2) as u32)
            }else{(old_at(i) as u32,skip)}
        });
        ((words,root),mark,s,z)
    }
}
