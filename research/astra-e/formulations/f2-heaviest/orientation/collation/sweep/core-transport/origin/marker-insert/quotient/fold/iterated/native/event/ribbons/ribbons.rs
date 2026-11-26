#[derive(Clone,Copy,Default,Debug)]
pub struct Ribbon {s:u32,d:u32,w:u32}
impl Ribbon{
    fn part(self,s:u32,w:u32)->Self{Self{s,d:self.d+(s-self.s),w}}
}
#[derive(Default)]
pub struct BandMeter{
    pub e:EventMeter,pub initial:u32,pub live:u32,pub high:u32,pub incl_tmp:u32,
    pub source:u64,pub overlaps:u64,pub left:u64,pub right:u64,pub generated:u64,pub coalesced:u64,
    pub shifted:u64,pub table_peak:usize,pub first_runs:u32,pub tests:u64,pub owner_searches:u64,pub fixed_queries:u64,
}
fn append_band(v:&mut Vec<Ribbon>,b:Ribbon){
    if let Some(a)=v.last_mut(){if a.s+a.w==b.s && a.d+a.w==b.d{a.w+=b.w;return;}}
    v.push(b);
}
pub trait Bands:Default{
    fn clear(&mut self);
    fn len(&self)->usize;
    fn known_bytes(&self)->usize;
    fn add(&mut self,b:Ribbon);
    fn first_last(&self)->(Ribbon,Ribbon);
    fn dest(&self,x:u32)->u32;
    fn visit(&self,f:impl FnMut(Ribbon));
    fn cut<const AUDIT:bool>(&mut self,lo:u32,w:u32,other:usize,g:&mut BandMeter,f:impl FnMut(Ribbon)->usize);
}
#[derive(Default)]
pub struct Flat(Vec<Ribbon>);
impl Bands for Flat{
    fn clear(&mut self){self.0.clear();}
    fn len(&self)->usize{self.0.len()}
    fn known_bytes(&self)->usize{bytes(&self.0)}
    fn add(&mut self,b:Ribbon){append_band(&mut self.0,b);}
    fn first_last(&self)->(Ribbon,Ribbon){(self.0[0],*self.0.last().unwrap())}
    fn dest(&self,x:u32)->u32{
        let r=self.0[self.0.partition_point(|b|b.s<=x)-1];assert!(x<r.s+r.w);r.d+(x-r.s)
    }
    fn visit(&self,mut f:impl FnMut(Ribbon)){for &r in &self.0{f(r);}}
    fn cut<const AUDIT:bool>(&mut self,lo:u32,w:u32,other:usize,g:&mut BandMeter,mut f:impl FnMut(Ribbon)->usize){
        let v=&mut self.0;let end=lo+w;
        let start=v.partition_point(|b|b.s<=lo)-1;
        let mut j=start;let mut x=lo;
        while x<end{
            let r=v[j];let y=end.min(r.s+r.w);
            assert!(r.s<=x && x<y);
            let temp=f(r.part(x,y-x));
            if AUDIT{g.incl_tmp=g.incl_tmp.max((other+v.len()+temp) as u32);}
            x=y;j+=1;
        }
        let a=v[start];let b=v[j-1];
        let mut repl=[Ribbon::default();2];let mut keep=0;
        if a.s<lo{repl[keep]=a.part(a.s,lo-a.s);keep+=1;if AUDIT{g.left+=1;}}
        if end<b.s+b.w{repl[keep]=b.part(end,b.s+b.w-end);keep+=1;if AUDIT{g.right+=1;}}
        if AUDIT{
            g.overlaps+=(j-start)as u64;
            if keep!=j-start{g.shifted+=(v.len()-j)as u64;}
        }
        v.splice(start..j,repl[..keep].iter().copied());
    }
}
#[derive(Default)]
pub struct Tree(std::collections::BTreeMap<u32,(u32,u32)>);
impl Bands for Tree{
    fn clear(&mut self){self.0.clear();}
    fn len(&self)->usize{self.0.len()}
    fn known_bytes(&self)->usize{self.0.len()*std::mem::size_of::<Ribbon>()}
    fn add(&mut self,b:Ribbon){
        if let Some(mut a)=self.0.last_entry(){
            let s=*a.key();let(d,w)=a.get_mut();
            if s+*w==b.s && *d+*w==b.d{*w+=b.w;return;}
        }
        self.0.insert(b.s,(b.d,b.w));
    }
    fn first_last(&self)->(Ribbon,Ribbon){
        let row=|(&s,&(d,w)):(&u32,&(u32,u32))|Ribbon{s,d,w};
        (row(self.0.first_key_value().unwrap()),row(self.0.last_key_value().unwrap()))
    }
    fn dest(&self,x:u32)->u32{
        let(&s,&(d,w))=self.0.range(..=x).next_back().unwrap();assert!(x<s+w);d+(x-s)
    }
    fn visit(&self,mut f:impl FnMut(Ribbon)){for(&s,&(d,w))in &self.0{f(Ribbon{s,d,w});}}
    fn cut<const AUDIT:bool>(&mut self,lo:u32,w:u32,other:usize,g:&mut BandMeter,mut f:impl FnMut(Ribbon)->usize){
        let end=lo+w;let mut x=lo;
        // All sources in this physical interval are live. After the first
        // predecessor search each crossing consumes exactly the next source key.
        let mut s=*self.0.range(..=x).next_back().unwrap().0;
        while x<end{
            let(d,width)=self.0.remove(&s).unwrap();let y=end.min(s+width);assert!(x<y);
            if s<x{self.0.insert(s,(d,x-s));if AUDIT{g.left+=1;}}
            let temp=f(Ribbon{s:x,d:d+(x-s),w:y-x});
            if y<s+width{self.0.insert(y,(d+(y-s),s+width-y));if AUDIT{g.right+=1;}}
            if AUDIT{g.overlaps+=1;g.incl_tmp=g.incl_tmp.max((other+self.0.len()+temp)as u32);}x=y;s=y;
        }
    }
}
#[derive(Default)]
struct RClass<B:Bands>{map:B,mass:u32,word:u32,base:u32,queued:bool}
#[derive(Default)]
pub struct RibbonFold<B:Bands>{
    c:Vec<RClass<B>>,active:usize,owners:Vec<(u32,u32)>,pending:std::collections::VecDeque<u32>,
    new_k:Vec<Ribbon>,grammar:Grammar,
}
impl<B:Bands> RibbonFold<B>{
    pub fn capacity(&self)->usize{
        bytes(&self.c)+bytes(&self.owners)+bytes(&self.new_k)+self.pending.capacity()*4+
        self.grammar.capacity()+self.c.iter().map(|c|c.map.known_bytes()).sum::<usize>()
    }
    fn start<const M:u8,const S:u8>(&mut self,w:&Dispatch<M,S>)->u32{
        self.active=w.words.len();
        if self.active>self.c.len(){self.c.resize_with(self.active,Default::default);}
        for c in &mut self.c{c.map.clear();c.mass=0;c.queued=false;}
        self.owners.clear();self.pending.clear();self.new_k.clear();self.grammar.dag.clear();
        self.grammar.dag.extend(w.words.iter().map(|p|Production{a:p.id,b:NONE,size:p.width}));
        for n in &w.nodes{if n.word!=NONE{
            let c=&mut self.c[n.word as usize];c.base=n.base;c.word=n.word;
            self.owners.push((n.base,n.word));
        }}
        self.owners.sort_unstable();
        let mut dest=0;
        for r in &w.merged[w.nodes[0].out as usize..]{
            let c=&mut self.c[w.nodes[r.id as usize].word as usize];
            c.map.add(Ribbon{s:c.base+c.mass,d:dest,w:r.width});
            c.mass+=r.width;dest+=r.width;
        }
        self.c[..self.active].iter().map(|c|c.map.len()as u32).sum()
    }
    #[inline]fn owner(&self,x:u32)->usize{
        self.owners[self.owners.partition_point(|p|p.0<=x)-1].1 as usize
    }
    fn target(&self,k:usize)->(Option<usize>,u32){
        let c=&self.c[k];if c.mass==0{return (None,0);}
        let(a,b)=c.map.first_last();let j=self.owner(a.d);
        if j==k{return (None,1);}
        ((self.owner(b.d+b.w-1)==j).then_some(j),2)
    }
    fn eligible<const AUDIT:bool>(&self,k:usize,g:&mut BandMeter)->Option<usize>{
        let(t,q)=self.target(k);
        if AUDIT{g.tests+=1;g.owner_searches+=u64::from(q);}
        t
    }
    fn schedule<const AUDIT:bool>(&mut self,k:usize,g:&mut BandMeter){
        if !self.c[k].queued && self.eligible::<AUDIT>(k,g).is_some(){
            self.pending.push_back(k as u32);self.c[k].queued=true;g.e.pushes+=1;
        }
    }
    fn dense_image(&self,n:u32)->Vec<u32>{
        let mut result=vec![NONE;n as usize];
        for c in &self.c[..self.active]{
            c.map.visit(|b|{for i in 0..b.w{result[(b.s+i)as usize]=b.d+i;}});
        }
        result
    }
    fn audit(&self,n:u32){
        let mut live=vec![false;n as usize];let mut images=live.clone();
        for k in 0..self.active{
            let c=&self.c[k];let mut num=0;let mut one=None;let mut mixed=false;
            let mut prev=None;
            c.map.visit(|b|{
                if let Some(Ribbon{s,d,w})=prev{
                    assert!(s+w<=b.s && d+w<=b.d);
                    assert!(s+w!=b.s || d+w!=b.d);
                }prev=Some(b);
                for i in 0..b.w{let s=b.s+i;let y=b.d+i;
                    assert!(!live[s as usize] && !images[y as usize]);live[s as usize]=true;images[y as usize]=true;
                    assert_eq!(self.owner(s),k);assert_eq!(c.map.dest(s),y);
                    let g=self.owner(y);if let Some(old)=one{mixed|=old!=g;}else{one=Some(g);}num+=1;}
            });
            assert_eq!(num,c.mass);
            assert_eq!(self.target(k).0,one.filter(|&j|!mixed && j!=k));
        }
        assert_eq!(live,images);
    }
    pub fn finish<const AUDIT:bool,const M:u8,const S:u8>(&mut self,w:&mut Dispatch<M,S>,
        mark:Option<(u32,u32)>,len:u32,n:u32,p:u32,out:&mut Vec<u8>)->BandMeter{
        let mut g=BandMeter::default();g.e.rows=len;
        let Some((mut x,mut skip))=mark else{w.finish(mark,len,n,p,out);g.e.m.cold=1;return g};
        let t=crate::now();
        let initial=self.start(w);g.first_runs=(w.merged.len()-w.nodes[0].out as usize)as u32;
        g.initial=initial;g.high=initial;g.incl_tmp=initial;g.live=initial;
        g.e.index_ns=crate::now()-t;let t=crate::now();
        let mut marked=self.owner(x); let mut fixed=self.c[marked].map.dest(x)==x;
        if AUDIT{g.owner_searches+=1;g.fixed_queries+=1;}
        for k in 0..self.active{self.schedule::<AUDIT>(k,&mut g);}
        g.e.peak_queue=self.pending.len()as u32;
        let mut tab=if AUDIT {self.c.iter().map(|c|c.map.known_bytes()).sum()}else{0};
        g.table_peak=tab;
        while !fixed{
            let Some(k)=self.pending.pop_front()else{break};let k=k as usize;
            self.c[k].queued=false;g.e.pops+=1;
            let Some(j)=self.eligible::<AUDIT>(k,&mut g)else{continue};
            let expected=if AUDIT && len<=16{
                let old=self.dense_image(len);let mut want=old.clone();
                for s in 0..len as usize{if old[s]!=NONE && self.owner(s as u32)==k{
                    let y=old[s]as usize;want[s]=old[y];want[y]=NONE;
                }}
                want
            }else{Vec::new()};
            self.new_k.clear();
            let [a,b]=self.c.get_disjoint_mut([k,j]).unwrap();
            let left=self.grammar.dag[a.word as usize].size;
            self.grammar.dag.push(Production{a:a.word,b:b.word,size:left+self.grammar.dag[b.word as usize].size});
            a.word=self.grammar.dag.len()as u32-1;
            let lhs=a.map.len();let rhs=b.map.len();let mut before=0;
            if AUDIT{before=a.map.known_bytes()+b.map.known_bytes();g.source+=lhs as u64;}
            let mut moved=None;
            a.map.visit(|r|{
                if marked==j && r.d<=x && x<r.d+r.w{moved=Some(r.s+(x-r.d));}
                let others=if AUDIT{g.live as usize-rhs}else{0};
                b.map.cut::<AUDIT>(r.d,r.w,others,&mut g,|t|{
                    append_band(&mut self.new_k,Ribbon{s:r.s+(t.s-r.d),d:t.d,w:t.w});self.new_k.len()
                });
                if AUDIT{g.incl_tmp=g.incl_tmp.max((g.live as usize-rhs+b.map.len()+self.new_k.len())as u32);}
            });
            if let Some(y)=moved{x=y;skip+=left;marked=k;g.e.m.moved+=1;}
            a.map.clear();for &r in &self.new_k{a.map.add(r);}
            b.mass-=a.mass;
            g.e.edits+=a.mass;g.e.rows-=a.mass;g.e.events+=1;
            if AUDIT{
                let count=a.map.len();
                g.live=(g.live as usize-lhs-rhs+count+b.map.len())as u32;
                g.high=g.high.max(g.live);g.generated+=count as u64;
                tab=tab-before+a.map.known_bytes()+b.map.known_bytes();g.table_peak=g.table_peak.max(tab);
                g.incl_tmp=g.incl_tmp.max(g.live+count as u32);
            }
            if k==marked{fixed=a.map.dest(x)==x;if AUDIT{g.fixed_queries+=1;}}
            if AUDIT && len<=16{self.audit(len);assert_eq!(self.dense_image(len),expected);}
            self.schedule::<AUDIT>(k,&mut g);self.schedule::<AUDIT>(j,&mut g);
            g.e.peak_queue=g.e.peak_queue.max(self.pending.len()as u32);
        }
        if AUDIT{
            g.coalesced=g.overlaps-g.generated;
            assert_eq!(u64::from(g.live)+g.source+g.coalesced,u64::from(g.initial)+g.left+g.right);
            assert!(2*g.high<=len+g.initial);
        }
        assert!(g.e.edits<len && g.e.pushes<=self.active as u32+2*g.e.events);
        g.e.event_ns=crate::now()-t;let t=crate::now();
        if fixed{
            let id=self.c[marked].word;let size=self.grammar.dag[id as usize].size;
            assert!(skip<size && size<=n);out.clear();out.reserve(n as usize);
            self.grammar.range::<AUDIT>(id,skip,size,&w.dict,out,&mut g.e.m);
            self.grammar.range::<AUDIT>(id,0,skip,&w.dict,out,&mut g.e.m);
            assert_eq!(out.len(),size as usize);g.e.m.period=size;g.e.m.terminal=true;
            while out.len()<n as usize{let take=out.len().min(n as usize-out.len());out.extend_from_within(..take);}
        }else{w.finish(mark,len,n,p,out);g.e.m.cold=2;}
        g.e.output_ns=crate::now()-t;g.e.m.nodes=self.grammar.dag.len()as u32;
        if AUDIT{g.e.packed=self.capacity();}g
    }
}
