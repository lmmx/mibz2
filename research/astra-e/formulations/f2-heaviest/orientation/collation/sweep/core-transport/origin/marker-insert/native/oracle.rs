struct Origins {
    a:u32, end:u32, words:Vec<Vec<u8>>, names:Vec<usize>, dest:Vec<usize>, base:Vec<usize>, mass:Vec<usize>,
}
impl Origins{
    fn new(truth:&Truth,u:(u32,u32))->Self{
        let (a,end)=u;let m=(end-a) as usize;
        let mut names=Vec::with_capacity(m);let mut dest=Vec::with_capacity(m);
        let mut named=HashMap::<Vec<u8>,usize>::new();let mut base=Vec::new();let mut mass=Vec::new();
        for s in a..end{
            let mut y=s as usize;let mut v=Vec::new();
            loop{
                y=truth.psi[y];v.push(truth.l[y]);
                if (a as usize..end as usize).contains(&y){break;}
            }
            assert_eq!(v[0],b'"');v.rotate_left(1);
            let next=base.len();let k=*named.entry(v).or_insert_with(||{base.push((s-a) as usize);mass.push(0);next});
            assert_eq!(base[k]+mass[k],(s-a) as usize);
            mass[k]+=1;names.push(k);dest.push(y-a as usize);
        }
        let mut words=vec![Vec::new();base.len()];for (w,k) in named{words[k]=w;}
        Self{a,end,words,names,dest,base,mass}
    }
    fn expected(&self,truth:&Truth,p:u32,h:u32)->(Obj,Option<(u32,u32)>,u32){
        let m=self.dest.len();let top=(0..self.mass.len()).max_by_key(|&k|(self.mass[k],usize::MAX-self.base[k])).unwrap();
        let widths=self.words.iter().enumerate().map(|(k,b)|if h==0{
            if k==top{b.len()}else{0}
        }else if b.len()>=2 && b.len()<=h as usize+1 && b[b.len()-2..]==[b',',b'"'] {2}else{0}).collect::<Vec<_>>();
        let s=self.names.iter().filter(|&&k|widths[k]>0).count();
        let source_mark=truth.mark(p,(self.a,self.end),true);
        if s==0{return (truth.object((self.a,self.end),true),source_mark,0);}
        let mut pairs=Vec::with_capacity(m);for i in 0..m{pairs.push((self.dest[i],i));}
        pairs.sort_unstable();let mut marker_at=vec![usize::MAX;m];
        let mut selected=0;
        let mut marker_word=Vec::new();
        let mut incoming=vec![u32::MAX;m+s];
        for (y,i) in pairs{
            let k=self.names[i];let old=(s+self.base[k]) as u32;
            if widths[k]==0{
                incoming[s+y]=old;
            }else{
                incoming[selected]=old;incoming[s+y]=0;
                marker_at[i]=selected;selected+=1;
                if marker_word.is_empty(){
                    marker_word.extend_from_slice(&self.words[k][self.words[k].len()-widths[k]..]);
                }
            }
        }
        assert_eq!(selected,s);
        let mut words=vec![(0,marker_word)];
        for (i,b) in self.words.iter().enumerate(){words.push(((s+self.base[i]) as u32,b[..b.len()-widths[i]].to_vec()));}
        let mut root=Vec::<(u32,u32)>::new();
        for node in incoming{
            assert_ne!(node,u32::MAX);
            if let Some(last)=root.last_mut(){if last.0==node{last.1+=1;continue;}}
            root.push((node,1));
        }
        let mark=source_mark.map(|(i,b)|{
            let k=self.names[i as usize];let prefix=(self.words[k].len()-widths[k]) as u32;
            if b>=prefix{
                assert_ne!(marker_at[i as usize],usize::MAX);
                (marker_at[i as usize] as u32,b-prefix)
            }else{(s as u32+i,b)}
        });
        ((words,root),mark,s as u32)
    }
}
