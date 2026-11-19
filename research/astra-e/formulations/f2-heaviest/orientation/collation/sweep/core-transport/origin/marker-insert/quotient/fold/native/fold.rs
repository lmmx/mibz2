#[derive(Clone, Copy, Default)]
struct FoldType {
    node: u32,
    base: u32,
    mass: u32,
    succ: u32,
    indegree: u32,
    take: u32,
    skip: u32,
    selected: bool,
    target: bool,
    after: u32,
    new_base: u32,
    source_run: u32,
    used: u32,
    before: u32,
}

#[derive(Default)]
struct FoldScratch {
    classes: Vec<FoldType>,
    order: Vec<usize>,
    peeled: Vec<usize>,
    cycle: Vec<usize>,
    back: Vec<bool>,
}

impl FoldScratch {
    // Enforce the first and last circular adjacency when closing the path.
    fn cycle_boundary(&mut self, first: bool, store: bool) -> i64 {
        let root = self.classes[self.cycle[0]];
        let impossible = i64::MIN / 4;
        let mut take = if first { i64::from(root.take) } else { impossible };
        let mut skip = if first { impossible } else { i64::from(root.skip) };
        self.back.clear();
        for &k in &self.cycle[1..] {
            let c = self.classes[k];
            self.back.push(take > skip);
            (take, skip) = (skip + i64::from(c.take), take.max(skip) + i64::from(c.skip));
        }
        let mut used = !first && take > skip;
        let score = if used { take } else { skip };
        if store {
            for i in (1..self.cycle.len()).rev() {
                self.classes[self.cycle[i]].selected = used;
                used = !used && self.back[i - 1];
            }
            assert_eq!(used, first);
            self.classes[self.cycle[0]].selected = first;
        }
        score
    }

    fn choose(&mut self) {
        let c = &mut self.classes;
        self.peeled.clear();
        for k in 0..c.len() {
            c[k].take = c[k].mass;
            let g = c[k].succ as usize;
            if g < c.len() && c[g].succ != NONE {
                c[g].indegree += 1;
            }
        }
        for &k in &self.order {
            if c[k].succ != NONE && c[k].indegree == 0 {
                self.peeled.push(k);
            }
        }
        let mut i = 0;
        while i < self.peeled.len() {
            let k = self.peeled[i];
            let g = c[k].succ as usize;
            if c[g].succ != NONE {
                c[g].take += c[k].skip;
                c[g].skip += c[k].take.max(c[k].skip);
                c[g].indegree -= 1;
                if c[g].indegree == 0 {
                    self.peeled.push(g);
                }
            }
            i += 1;
        }
        for j in 0..self.order.len() {
            let start = self.order[j];
            if self.classes[start].indegree == 0 { continue; }
            self.cycle.clear();
            let mut k = start;
            while self.classes[k].indegree != 0 {
                self.cycle.push(k);
                self.classes[k].indegree = 0;
                k = self.classes[k].succ as usize;
            }
            assert_eq!(k,start);
            assert!(self.cycle.len() >= 2);
            let a = self.cycle_boundary(false,false);
            let b = self.cycle_boundary(true,false);
            self.cycle_boundary(b>a,true);
        }
        for &k in self.peeled.iter().rev() {
            let t = self.classes[k];
            self.classes[k].selected = !self.classes[t.succ as usize].selected && t.take > t.skip;
        }
    }

    fn recognize(&mut self, root: &[Piece], nodes: &[Node], alphabet: usize, m: u32) {
        const UNSEEN:u32 = u32::MAX-1;
        self.classes.clear();
        self.classes.resize(alphabet,FoldType{succ:UNSEEN,..FoldType::default()});
        self.order.clear();
        for r in root {
            let n=nodes[r.id as usize];
            let k=n.word as usize;
            let c=&mut self.classes[k];
            if c.mass==0 {
                c.node=r.id;
                c.base=n.base;
                self.order.push(k);
            }
            c.mass+=r.width;
        }
        assert_eq!(self.order.len(),alphabet);
        self.order.sort_unstable_by_key(|&k|self.classes[k].base);
        let mut at=0;
        let mut j=0;
        for (ri,r) in root.iter().enumerate() {
            let k=nodes[r.id as usize].word as usize;
            let end=at+r.width;
            let owner=self.order[j];
            let mut right=owner;
            while j<self.order.len() {
                let g=self.order[j];
                let c=&mut self.classes[g];
                if c.base>=end {break;}
                right=g;
                if c.base>=at {
                    c.source_run=ri as u32;
                    c.used=c.base-at;
                }
                if c.base+c.mass>end {break;}
                j+=1;
            }
            let g=self.classes[k].succ;
            self.classes[k].succ=if owner!=right || owner==k || (g!=UNSEEN && g!=owner as u32) {
                NONE
            } else {owner as u32};
            at=end;
        }
        assert_eq!(at,m);
        assert_eq!(j,self.order.len());
        self.choose();
    }

    fn capacity(&self)->usize{
        bytes(&self.classes)+bytes(&self.order)+bytes(&self.peeled)+bytes(&self.cycle)+bytes(&self.back)
    }
}

impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE>{
    pub fn fold_one(&mut self,mark:Option<(u32,u32)>,m:u32,publish:bool)
        ->(Option<(u32,u32)>,u32,[u32;10],[u64;3]){
        let t=crate::now();
        let root=&self.merged[self.nodes[0].out as usize..];
        let f=&mut self.fold_scratch;
        f.recognize(root,&self.nodes,self.words.len(),m);
        let t1=crate::now();
        let mut stats=[0;10];
        let c=&mut f.classes;
        for k in 0..c.len(){c[k].after=c[k].mass;}
        for k in 0..c.len(){
            if c[k].succ!=NONE {stats[0]+=1;stats[1]+=c[k].mass;}
            if c[k].selected{
                let g=c[k].succ as usize;
                assert!(!c[g].selected);
                c[g].after-=c[k].mass;
                c[g].target=true;
                stats[2]+=1;stats[3]+=c[k].mass;
            }
        }
        stats[4]=root.len() as u32;
        if !publish || stats[3]==0 {return (mark,m,stats,[t1-t,0,0]);}
        let mut base=0;
        for &k in &f.order{
            c[k].new_base=base;
            base+=c[k].after;
            stats[5]+=u32::from(c[k].after==0);
            if c[k].selected{
                let a=self.words[k];
                let b=self.words[c[k].succ as usize];
                let begin=self.dict.len() as u32;
                self.dict.extend_from_within(a.id as usize..(a.id+a.width) as usize);
                self.dict.extend_from_within(b.id as usize..(b.id+b.width) as usize);
                self.words[k]=Piece{id:begin,width:a.width+b.width};
                stats[7]+=a.width+b.width;
            }
        }
        let t2=crate::now();
        self.selectors.clear();
        let (mut at,mut removed)=(0,0);
        let mut mapped=mark;
        for r in root {
            let k=self.nodes[r.id as usize].word as usize;
            let end=at+r.width;
            if let Some((x,skip))=mark {
                if at<=x && x<end {
                    if c[k].selected {
                        let suffix=self.words[c[k].succ as usize].width;
                        let prefix=self.words[k].width-suffix;
                        mapped=Some((c[k].new_base+c[k].before+(x-at),prefix+skip));
                        stats[8]=1;
                    }else{mapped=Some((x-removed,skip));}
                }
            }
            if c[k].selected {
                removed+=r.width;
                c[k].before+=r.width;
            }else if c[k].target{
                let mut left=r.width;
                while left>0{
                    let source=root[c[k].source_run as usize];
                    let old=self.nodes[source.id as usize].word as usize;
                    let width=left.min(source.width-c[k].used);
                    let id=if c[old].selected{source.id}else{r.id};
                    sparse_append(&mut self.selectors,Piece{id,width});
                    c[k].used+=width;
                    if c[k].used==source.width{
                        c[k].source_run+=1;c[k].used=0;
                    }
                    left-=width;
                    stats[9]+=1;
                }
            }else{sparse_append(&mut self.selectors,*r);}
            at=end;
        }
        assert_eq!(base,m-removed);
        for k in c.iter(){
            let n=&mut self.nodes[k.node as usize];
            if k.after==0{n.word=NONE;} else{n.base=k.new_base;}
        }
        stats[6]=self.selectors.len() as u32;
        std::mem::swap(&mut self.merged,&mut self.selectors);
        self.nodes[0].out=0;
        (mapped,base,stats,[t1-t,t2-t1,crate::now()-t2])
    }
    pub fn fold_selection(&self)->Vec<(u32,u32,u32,bool)>{
        let f=&self.fold_scratch;
        f.order.iter().map(|&k|{
            let c=f.classes[k];
            (c.base,c.mass,if c.succ==NONE{NONE}else{f.classes[c.succ as usize].base},c.selected)
        }).collect()
    }
}

// Independent exhaustive graph family, including cycles with ties and attached paths.
pub fn small_optimizer()->u32{
    let mut f=FoldScratch::default();
    let mut cases=0;
    for n in 1usize..=6{
        for code in 0..n.pow(n as u32){
            for pattern in 0..3{
                f.classes.clear();f.order.clear();let mut v=code;
                for i in 0..n{
                    let g=v%n;v/=n;
                    let mass=match pattern{0=>1,1=>1+i%3,_=>1+(5*i+code)%7};
                    f.classes.push(FoldType{mass:mass as u32,succ:if i==g{NONE}else{g as u32},..FoldType::default()});
                    f.order.push(i);
                }
                f.choose();
                let best=(0..(1usize<<n)).filter_map(|bits|{
                    let mut w=0;
                    for (i,c) in f.classes.iter().enumerate(){
                        if bits>>i&1==0{continue;}
                        if c.succ==NONE || (bits>>c.succ)&1!=0{return None;}
                        w+=c.mass;
                    }
                    Some(w)
                }).max().unwrap();
                assert_eq!(best,f.classes.iter().filter(|c|c.selected).map(|c|c.mass).sum::<u32>());
                assert!(f.classes.iter().all(|c|!c.selected || !f.classes[c.succ as usize].selected));
                cases+=1;
            }
        }
    }
    cases
}
