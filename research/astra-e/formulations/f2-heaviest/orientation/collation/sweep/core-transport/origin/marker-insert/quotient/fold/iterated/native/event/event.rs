#[derive(Default,Clone,Copy)]
struct EventClass { head:u32, tail:u32, count:u32, word:u32, queued:bool }
#[derive(Default)]
pub struct Event {
    succ:Vec<u32>, owner:Vec<u32>, prev:Vec<u32>, next:Vec<u32>, c:Vec<EventClass>,
    pending:std::collections::VecDeque<u32>, grammar:Grammar,
}
#[derive(Default)]
pub struct EventMeter {
    pub m:Meter, pub edits:u32, pub events:u32, pub pushes:u32, pub pops:u32,
    pub peak_queue:u32,pub rows:u32,pub packed:usize,pub index_ns:u64,pub event_ns:u64,pub output_ns:u64,
}
impl Event {
    pub fn capacity(&self)->usize{
        bytes(&self.succ)+bytes(&self.owner)+bytes(&self.prev)+bytes(&self.next)+bytes(&self.c)
        +self.pending.capacity()*4+self.grammar.capacity()
    }
    fn build<const FOLD:bool,const M:u8,const S:u8>(&mut self,w:&Dispatch<M,S>,len:u32){
        self.succ.resize(len as usize,0);self.owner.resize(len as usize,0);
        self.c.clear();self.c.resize(w.words.len(),EventClass::default());
        for n in &w.nodes{
            if n.word!=NONE{self.c[n.word as usize]=EventClass{head:n.base,word:n.word,..Default::default()};}
        }
        let mut dest=0;
        for r in &w.merged[w.nodes[0].out as usize..]{
            let k=w.nodes[r.id as usize].word;let c=&mut self.c[k as usize];
            let src=(c.head+c.count)as usize;let end=src+r.width as usize;
            self.owner[src..end].fill(k);
            for (i,d) in self.succ[src..end].iter_mut().enumerate(){*d=dest+i as u32;}
            dest+=r.width;c.count+=r.width;
        }
        assert_eq!(dest,len);
        if FOLD{
            self.prev.clear();self.prev.extend((0..len).map(|i|i.wrapping_sub(1)));
            self.next.clear();self.next.extend(1..=len);
            for c in &mut self.c{
                c.tail=c.head+c.count-1;
                self.prev[c.head as usize]=NONE;self.next[c.tail as usize]=NONE;
            }
            self.grammar.dag.clear();
            self.grammar.dag.extend(w.words.iter().map(|p|Production{a:p.id,b:NONE,size:p.width}));
            self.pending.clear();
        }
    }
    fn target(&self,k:usize)->Option<usize>{
        let c=self.c[k];if c.count==0{return None;}
        let owners=|row:u32|self.owner[self.succ[row as usize] as usize]as usize;
        let j=owners(c.head);
        (j!=k && owners(c.tail)==j).then_some(j)
    }
    fn schedule(&mut self,k:usize,stats:&mut EventMeter){
        if !self.c[k].queued && self.target(k).is_some(){
            self.c[k].queued=true; self.pending.push_back(k as u32);stats.pushes+=1;
        }
    }
    fn audit(&self)->Vec<Option<usize>>{
        let mut live=vec![false;self.succ.len()];let mut image=live.clone();
        let mut eligible=vec![None;self.c.len()];
        for (k,c) in self.c.iter().enumerate(){
            if c.count==0{continue;}
            let mut rows=Vec::new();let mut x=c.head;let mut y=NONE;
            while x!=NONE{
                assert!(!live[x as usize]);live[x as usize]=true;
                let t=self.succ[x as usize];assert!(!image[t as usize]);image[t as usize]=true;
                assert_eq!(self.prev[x as usize],y);assert_eq!(self.owner[x as usize],k as u32);
                if y!=NONE{assert!(y<x && self.succ[y as usize]<t);}
                rows.push(t);y=x;x=self.next[x as usize];
            }
            assert_eq!(rows.len(),c.count as usize);assert_eq!(y,c.tail);
            let j=self.owner[rows[0]as usize]as usize;
            eligible[k]=(j!=k && rows.iter().all(|&r|self.owner[r as usize]==j as u32)).then_some(j);
            assert_eq!(eligible[k],self.target(k));
        }
        assert_eq!(live,image);eligible
    }
    fn walk<const M:u8,const S:u8>(&self,w:&Dispatch<M,S>,p:u32,skip:u32,n:u32,out:&mut Vec<u8>)->u32{
        let spelling=|x:u32|{let q=w.words[self.owner[x as usize]as usize];
            &w.dict[q.id as usize..(q.id+q.width)as usize]};
        out.clear();out.reserve(n as usize);
        let first=spelling(p);out.extend_from_slice(&first[skip as usize..]);
        let mut x=self.succ[p as usize];
        while x!=p{out.extend_from_slice(spelling(x));x=self.succ[x as usize];}
        out.extend_from_slice(&first[..skip as usize]);let period=out.len();
        assert!(period>0 && period<=n as usize);
        while out.len()<n as usize{let take=out.len().min(n as usize-out.len());out.extend_from_within(..take);}
        period as u32
    }
    pub fn finish<const FOLD:bool,const AUDIT:bool,const M:u8,const S:u8>(
        &mut self,w:&mut Dispatch<M,S>,mark:Option<(u32,u32)>,len:u32,n:u32,p:u32,out:&mut Vec<u8>
    )->EventMeter{
        let mut g=EventMeter{rows:len,..Default::default()};
        let Some((mut x,mut skip))=mark else{w.finish(mark,len,n,p,out);g.m.cold=1;return g};
        let t=crate::now();self.build::<FOLD,M,S>(w,len);g.index_ns=crate::now()-t;
        if !FOLD{
            let t=crate::now();g.m.period=self.walk(w,x,skip,n,out);g.output_ns=crate::now()-t;
            g.packed=self.capacity();return g;
        }
        let t=crate::now();
        for k in 0..self.c.len(){self.schedule(k,&mut g);}
        g.peak_queue=self.pending.len()as u32;
        let small=AUDIT && len<=16;
        while self.succ[x as usize]!=x{
            let Some(k)=self.pending.pop_front() else{break};
            let k=k as usize;self.c[k].queued=false;g.pops+=1;
            let Some(j)=self.target(k) else{continue};
            let before=if small{self.audit()}else{Vec::new()};
            let ck=self.c[k];let cj=self.c[j];
            let a=ck.word;let b=cj.word;let left=self.grammar.dag[a as usize].size;
            self.c[k].word=self.grammar.dag.len()as u32;
            self.grammar.dag.push(Production{a,b,size:left+self.grammar.dag[b as usize].size});
            let mut v=ck.head;
            while v!=NONE{
                let y=self.succ[v as usize];self.succ[v as usize]=self.succ[y as usize];
                if y==x{x=v;skip+=left;g.m.moved+=1;}
                let l=self.prev[y as usize];let r=self.next[y as usize];
                if l==NONE{self.c[j].head=r;}else{self.next[l as usize]=r;}
                if r==NONE{self.c[j].tail=l;}else{self.prev[r as usize]=l;}
                v=self.next[v as usize];
            }
            self.c[j].count-=ck.count;
            g.edits+=ck.count;g.rows-=ck.count;g.events+=1;
            if small{
                let after=self.audit();
                for i in 0..after.len(){if i!=j && i!=k{assert_eq!(before[i],after[i]);}}
            }
            self.schedule(k,&mut g);self.schedule(j,&mut g);
            g.peak_queue=g.peak_queue.max(self.pending.len() as u32);
        }
        assert!(g.edits<len && g.pushes<=self.c.len()as u32+2*g.events);
        g.event_ns=crate::now()-t;let t=crate::now();
        if self.succ[x as usize]==x{
            let id=self.c[self.owner[x as usize] as usize].word;
            let size=self.grammar.dag[id as usize].size;assert!(skip<size && size<=n);
            out.clear();out.reserve(n as usize);
            self.grammar.range::<AUDIT>(id,skip,size,&w.dict,out,&mut g.m);
            self.grammar.range::<AUDIT>(id,0,skip,&w.dict,out,&mut g.m);
            assert_eq!(out.len(),size as usize);g.m.period=size;g.m.terminal=true;
            while out.len()<n as usize{
                let take=out.len().min(n as usize-out.len());out.extend_from_within(..take);
            }
        }else{w.finish(mark,len,n,p,out);g.m.cold=2;}
        g.m.nodes=self.grammar.dag.len()as u32;
        g.output_ns=crate::now()-t;g.packed=self.capacity();g
    }
}
