#[derive(Clone,Copy)]
struct Production { a:u32, b:u32, size:u32 }
// A leaf's a is an offset into the unchanged first dictionary, with b=NONE.
#[derive(Default)]
pub struct Grammar {
    dag:Vec<Production>, names:Vec<Piece>, next_names:Vec<Piece>,
    root:Vec<Piece>, rewritten:Vec<Piece>, order:Vec<(u32,u32)>, remap:Vec<u32>,
    stack:Vec<(u32,u32,u32)>,
}
#[derive(Default,Debug)]
pub struct Meter {
    pub rounds:u32, pub initial:u32, pub nodes:u32, pub rows:u32, pub types:u32, pub final_runs:u32,
    pub paid_runs:u64, pub paid_classes:u64, pub removed:u32, pub moved:u32,
    pub expansions:u32, pub leaf_visits:u32, pub peak_stack:usize, pub period:u32,
    pub transfer:u32, pub init_ns:u64, pub folds_ns:u64, pub emit_ns:u64, pub extra:usize,
    pub cold:u32, pub terminal:bool,
}
impl Grammar {
    pub fn capacity(&self)->usize{
        bytes(&self.dag)+bytes(&self.names)+bytes(&self.next_names)+bytes(&self.root)
        +bytes(&self.rewritten)+bytes(&self.order)+bytes(&self.remap)+bytes(&self.stack)
    }
    fn start<const M:u8,const S:u8>(&mut self,w:&Dispatch<M,S>){
        self.dag.clear(); self.names.clear(); self.order.clear(); self.root.clear();
        for p in &w.words { self.dag.push(Production{a:p.id,b:NONE,size:p.width}); }
        for node in &w.nodes { if node.word!=NONE {self.order.push((node.base,node.word));} }
        self.order.sort_unstable();
        let k=self.order.len(); assert_eq!(k,w.words.len()); self.remap.resize(k,0);
        for (i,&(_,id)) in self.order.iter().enumerate(){
            self.remap[id as usize]=i as u32; self.names.push(Piece{id,width:0});
        }
        for p in &w.merged[w.nodes[0].out as usize..]{
            let id=self.remap[w.nodes[p.id as usize].word as usize];
            self.names[id as usize].width+=p.width; self.root.push(Piece{id,width:p.width});
        }
    }

    // Locate the marked vertex's exact successor in the incoming run stream while
    // recording one deterministic target per source and target-column cursors.
    fn inspect(&self,f:&mut FoldScratch,x:u32)->Option<u32>{
        const NEW:u32=NONE-1;
        f.classes.clear(); f.order.clear();
        let (mut base,mut owner)=(0,0);
        for (k,p) in self.names.iter().enumerate(){
            if base<=x {owner=k;}
            f.classes.push(FoldType{mass:p.width,base,succ:NEW,..FoldType::default()});
            f.order.push(k); base+=p.width;
        }
        let mut ordinal=x-f.classes[owner].base;
        let mut seen_mark=false;
        let mut j=0;let mut at=0;
        for (ri,r) in self.root.iter().enumerate(){
            let k=r.id as usize;let stop=at+r.width;
            if k==owner && !seen_mark{
                if ordinal<r.width{
                    seen_mark=true;
                    if at+ordinal==x{return Some(self.names[k].id);}
                }else{ordinal-=r.width;}
            }
            let first=j;
            let mut last=j;
            while j<f.classes.len(){
                let c=&mut f.classes[j];
                if c.base>=stop{break;}
                last=j;
                if c.base>=at{c.source_run=ri as u32; c.used=c.base-at;}
                if c.base+c.mass>stop{break;}
                j+=1;
            }
            let old=f.classes[k].succ;
            f.classes[k].succ=if first==last && first!=k && (old==NEW||old==first as u32){
                first as u32
            }else{NONE};
            at=stop;
        }
        assert!(seen_mark); assert_eq!(j,f.classes.len());
        f.choose(); None
    }

    fn weave(&mut self,f:&mut FoldScratch,x:u32,skip:u32,m:&mut Meter)->Option<(u32,u32)>{
        let c=&mut f.classes;
        for t in c.iter_mut(){t.after=t.mass;}
        let old_nodes=self.dag.len();
        for k in 0..c.len(){
            if c[k].selected{
                let g=c[k].succ as usize;
                c[g].after-=c[k].mass; c[g].target=true;
                let a=self.names[k].id;let b=self.names[g].id;
                self.dag.push(Production{a,b,size:self.dag[a as usize].size+self.dag[b as usize].size});
            }
        }
        if old_nodes==self.dag.len(){return None;}
        self.next_names.clear();
        let mut prod=old_nodes as u32;let mut base=0;
        for (k,t) in c.iter_mut().enumerate(){
            t.new_base=base; base+=t.after;
            if t.after==0{continue;}
            let id=if t.selected {let id=prod;prod+=1;id}else{self.names[k].id};
            t.node=self.next_names.len() as u32;
            self.next_names.push(Piece{id,width:t.after});
        }
        assert_eq!(prod,self.dag.len() as u32);
        self.rewritten.clear();
        let (mut at,mut removed)=(0,0);
        let mut marked=None;
        for r in &self.root{
            let k=r.id as usize;
            if at<=x && x<at+r.width{
                marked=Some(if c[k].selected{
                    m.moved+=1;
                    (c[k].new_base+c[k].before+x-at,skip+self.dag[self.names[k].id as usize].size)
                }else{(x-removed,skip)});
            }
            if c[k].selected{
                c[k].before+=r.width;removed+=r.width;
            }else if c[k].target{
                let mut left=r.width;
                while left!=0{
                    let src=self.root[c[k].source_run as usize];let old=src.id as usize;
                    let w=left.min(src.width-c[k].used);
                    let from=if c[old].selected{old}else{k};
                    assert!(c[from].after!=0);
                    sparse_append(&mut self.rewritten,Piece{id:c[from].node,width:w});
                    left-=w;c[k].used+=w;
                    if c[k].used==src.width{c[k].used=0;c[k].source_run+=1;}
                }
            }else{sparse_append(&mut self.rewritten,Piece{id:c[k].node,width:r.width});}
            at+=r.width;
        }
        assert_eq!(base,at-removed);m.removed+=removed;m.rounds+=1;
        std::mem::swap(&mut self.rewritten,&mut self.root);
        std::mem::swap(&mut self.next_names,&mut self.names);
        marked
    }

    fn range<const AUDIT:bool>(&mut self,word:u32,lo:u32,hi:u32,dict:&[u8],out:&mut Vec<u8>,m:&mut Meter){
        if lo==hi{return;}
        self.stack.clear();self.stack.push((word,lo,hi));
        while let Some((k,a,b))=self.stack.pop(){
            let node=self.dag[k as usize];
            if node.b==NONE{
                out.extend_from_slice(&dict[(node.a+a)as usize..(node.a+b)as usize]);
                if AUDIT{m.leaf_visits+=1;}
            }else{
                let left=self.dag[node.a as usize].size;
                if b>left{self.stack.push((node.b,a.saturating_sub(left),b-left));}
                if a<left{self.stack.push((node.a,a,b.min(left)));}
                if AUDIT{m.expansions+=1;m.peak_stack=m.peak_stack.max(self.stack.len());}
            }
        }
    }

    pub fn finish<const M:u8,const S:u8,const AUDIT:bool>(&mut self,w:&mut Dispatch<M,S>,
        saved:Option<(u32,u32)>,rows:u32,n:u32,p:u32,out:&mut Vec<u8>)->Meter{
        let mut m=Meter::default();
        let Some((mut x,mut skip))=saved else{
            w.finish(saved,rows,n,p,out); m.cold=1; return m;
        };
        let start=crate::now();self.start(w);let ready=crate::now();m.init_ns=ready-start;
        m.initial=self.names.len() as u32;m.transfer=skip;
        loop{
            m.paid_runs+=self.root.len() as u64;m.paid_classes+=self.names.len() as u64;
            // A one-row column makes its sole named word a complete marked period.
            let word=if self.names.len()==1 && self.names[0].width==1{Some(self.names[0].id)}
                     else{self.inspect(&mut w.fold_scratch,x)};
            if let Some(id)=word{
                m.folds_ns=crate::now()-ready;
                let emit=crate::now();
                let length=self.dag[id as usize].size; assert!(skip<length && length<=n);
                out.clear(); out.reserve(n as usize);
                self.range::<AUDIT>(id,skip,length,&w.dict,out,&mut m);
                self.range::<AUDIT>(id,0,skip,&w.dict,out,&mut m);
                assert_eq!(out.len(),length as usize);
                while out.len()<n as usize{
                    let take=out.len().min(n as usize-out.len());out.extend_from_within(..take);
                }
                m.period=length;m.terminal=true;m.emit_ns=crate::now()-emit;break;
            }
            if let Some((y,b))=self.weave(&mut w.fold_scratch,x,skip,&mut m){x=y;skip=b;}
            else{
                m.folds_ns=crate::now()-ready;
                let emit=crate::now();
                w.finish(saved,rows,n,p,out);m.cold=2;m.emit_ns=crate::now()-emit;break;
            }
        }
        m.nodes=self.dag.len() as u32;m.rows=rows-m.removed;
        m.types=self.names.len() as u32;m.final_runs=self.root.len() as u32;
        m.extra=self.capacity()+w.fold_scratch.capacity();m
    }
}
