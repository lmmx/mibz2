#[derive(Default)]
struct Forest {
    order:Vec<u32>, first:Vec<u32>, last:Vec<u32>, head:Vec<u32>, next:Vec<u32>,
    slot:Vec<u32>, project:Vec<u32>, bfs:Vec<u32>,
}
impl Forest {
    fn capacity(&self)->usize{
        use super::bytes;
        bytes(&self.order)+bytes(&self.first)+bytes(&self.last)+bytes(&self.head)+bytes(&self.next)
            +bytes(&self.slot)+bytes(&self.project)+bytes(&self.bfs)
    }
}
impl Second {
    fn forest_piece(&mut self,slot:usize,w:u32,width:u32,lf:u32,d:u32,words:&[Piece],buf:&[u8],begin:usize){
        let id=if w==d {
            if self.nodes[slot].word==NONE {
                self.nodes[slot].base=lf;
                self.nodes[slot].word=self.words.len() as u32;
                let start=self.dictionary.len() as u32;
                let mut j=slot as u32;
                while j!=NONE {
                    let node=self.nodes[j as usize];
                    let z=words[self.edges[node.edge as usize] as usize];
                    self.dictionary.extend_from_slice(&buf[z.id as usize..(z.id+z.width) as usize]);
                    j=node.parent;
                }
                self.words.push(Piece{id:start,width:self.dictionary.len() as u32-start});
            }
            NONE
        } else {
            let k=w as usize;
            let target=if self.forest.slot[k]!=NONE {self.forest.slot[k] as usize} else {
                if self.epoch[k]!=slot as u32 {
                    self.epoch[k]=slot as u32;
                    self.child[k]=self.nodes.len();
                    self.nodes.push(Self::fresh_node(lf,slot as u32,w));
                }
                self.child[k]
            };
            debug_assert_eq!(self.nodes[target].hi,lf);
            self.nodes[target].hi+=width;
            target as u32
        };
        if self.selectors.len()>begin && self.selectors.last().unwrap().id==id {
            self.selectors.last_mut().unwrap().width+=width;
        } else {self.selectors.push(Piece{id,width});}
    }

    fn forest_index(&mut self,root:&[Piece],nodes:&[Node],words:&[Piece],buf:&[u8],p:u32,mut skip:u32,m:u32)
        ->Option<(u32,u32,u32)>{
        let k=words.len();
        Self::reset(&mut self.base,k,0);Self::reset(&mut self.count,k,0);
        Self::reset(&mut self.forest.first,k,m);Self::reset(&mut self.forest.last,k,0);
        let mut at=0;
        for r in root {
            let n=nodes[r.id as usize];let c=n.word as usize;
            self.base[c]=n.base;
            self.count[c]+=r.width;
            self.forest.first[c]=self.forest.first[c].min(at);
            at+=r.width;self.forest.last[c]=at;
        }
        let d=(0..k).max_by_key(|&i|(self.count[i],u32::MAX-self.base[i])).unwrap();
        if self.count[d]==m {return None;}
        let d=d as u32;
        let f=&mut self.forest;
        f.order.clear();f.order.extend((0..k as u32).filter(|&c|self.count[c as usize]>0));
        f.order.sort_unstable_by_key(|&c|self.base[c as usize]);
        Self::reset(&mut f.head,k,NONE);Self::reset(&mut f.next,k,NONE);Self::reset(&mut f.slot,k,NONE);
        // The whole forward destination image of c belongs to one source
        // bucket exactly when its two extreme destinations have this owner.
        for &w in &f.order {
            if w==d {continue;}
            let c=w as usize;
            let j=f.order.partition_point(|&t|self.base[t as usize]<=f.first[c])-1;
            let owner=f.order[j] as usize;
            if f.last[c]<=self.base[owner]+self.count[owner] {
                f.next[c]=f.head[owner];f.head[owner]=w;
            }
        }
        self.nodes.clear();self.edges.clear();self.selectors.clear();self.words.clear();self.dictionary.clear();
        Self::reset(&mut self.epoch,k,NONE);self.child.resize(k,0);
        f.bfs.clear();f.bfs.push(d);f.slot[d as usize]=0;
        let mut first=Self::fresh_node(0,NONE,d);first.edge=0;first.end=1;
        self.edges.push(d);self.nodes.push(first);
        let mut j=0;
        while j<f.bfs.len() {
            let w=f.bfs[j];let mut child=f.head[w as usize];
            while child!=NONE {
                let c=child as usize;
                let i=f.bfs.len() as u32;f.bfs.push(child);f.slot[c]=i;
                let mut n=Self::fresh_node(0,j as u32,child);n.edge=i;n.end=i+1;
                self.edges.push(child);self.nodes.push(n);
                child=f.next[c];
            }
            j+=1;
        }
        self.ready=f.bfs.len();
        self.runs.clear();Self::reset(&mut f.project,k,0);
        let mut outer=0;let mut owner=0;
        for &c in &f.order {
            let i=c as usize;
            if self.base[i]<=p && p<self.base[i]+self.count[i] {owner=c;}
            if f.slot[i]==NONE {f.project[i]=outer;outer+=self.count[i];}
        }
        self.count.fill(0);
        let mut at=0;let mut i=0;
        for r in root {
            let n=nodes[r.id as usize];let c=n.word as usize;
            let lf=self.forest.project[c]+self.count[c];self.count[c]+=r.width;
            let mut pos=at;
            let end=at+r.width;
            while pos<end {
                let src=self.forest.order[i] as usize;
                let b=self.base[src];
                let hi=if i+1<self.forest.order.len(){self.base[self.forest.order[i+1] as usize]}else{m};
                let stop=end.min(hi);
                let from=self.forest.slot[src];
                if from==NONE {
                    let start=self.forest.project[src]+pos-b;
                    let mapped=lf+pos-at;
                    if !self.runs.last().is_some_and(|previous|previous.word==n.word && previous.lf+start-previous.start==mapped) {
                        self.runs.push(TokenRun{word:n.word,lf:mapped,start});
                    }
                } else {
                    let slot=from as usize;
                    if pos==b {self.nodes[slot].sel=self.selectors.len() as u32;}
                    let begin=self.nodes[slot].sel as usize;
                    self.forest_piece(slot,n.word,stop-pos,lf+pos-at,d,words,buf,begin);
                    if stop==hi {self.nodes[slot].sels=self.selectors.len() as u32-begin as u32;}
                }
                if stop==hi {i+=1;}
                pos=stop;
            }
            at=end;
        }
        let pages=outer.div_ceil(64) as usize;
        Self::reset(&mut self.page,pages,0);Self::reset(&mut self.masks,pages,0);
        for r in &self.runs {let bit=r.start&63;if bit!=0 {self.masks[(r.start>>6) as usize]|=1u64<<bit;}}
        self.runs.push(TokenRun{start:outer,lf:0,word:0});
        let mut run=0;
        for (page,slot) in self.page.iter_mut().enumerate() {
            while self.runs[run+1].start<=page as u32*64 {run+=1;}
            *slot=run as u32;
        }
        let mut label=owner as usize;
        let mut ordinal=p-self.base[label];
        let started_outside=self.forest.slot[label]==NONE;
        while self.forest.slot[label]!=NONE {
            let from=self.nodes[self.forest.slot[label] as usize];
            let pieces=&self.selectors[from.sel as usize..(from.sel+from.sels) as usize];
            let mut offset=ordinal;let mut j=0;
            while offset>=pieces[j].width {offset-=pieces[j].width;j+=1;}
            let id=pieces[j].id;
            for earlier in &pieces[..j] {if earlier.id==id {offset+=earlier.width;}}
            if id==NONE {return Some((d,from.base+offset,skip));}
            let child=self.nodes[id as usize];
            if (id as usize)<self.ready {
                label=self.forest.bfs[id as usize] as usize;
                ordinal=offset;
            } else {
                label=child.edge as usize;
                ordinal=child.lo+offset-self.forest.project[label];
            }
            skip+=words[label].width;
        }
        let origin=self.forest.project[label]+ordinal;
        let mut x=origin;
        loop {
            let r=self.runs[self.find(x)];x=r.lf+x-r.start;
            if r.word==d{return Some((d,x,skip));}
            debug_assert_eq!(self.forest.slot[r.word as usize],NONE);
            skip+=words[r.word as usize].width;
            if started_outside && x==origin{return None;}
        }
    }
}
