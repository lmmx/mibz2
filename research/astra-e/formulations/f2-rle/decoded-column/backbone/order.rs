use std::collections::{HashMap,HashSet};
#[derive(Clone,Copy,Debug)]
struct Node { id:u32,anchor:u32,symbol:u8 }
struct Overlay {
    n:u32, holes:Vec<u32>, by_hole:Vec<Vec<u32>>, nodes:Vec<Node>, by_symbol:Vec<Vec<u32>>, by_id:Vec<(u32,u32)>,
}
impl Overlay {
    fn new(n:u32)->Self{Self{n,holes:Vec::new(),by_hole:(0..256).map(|_|Vec::new()).collect(),nodes:Vec::new(),by_symbol:(0..256).map(|_|Vec::new()).collect(),by_id:Vec::new()}}
    fn reindex(&mut self){
        self.by_id.clear();for v in &mut self.by_symbol{v.clear();}
        for (i,e) in self.nodes.iter().enumerate(){self.by_id.push((e.id,i as u32));self.by_symbol[e.symbol as usize].push(i as u32);}
        self.by_id.sort_unstable();
    }
    fn hole_index(&mut self,nav:&Nav){
        self.holes.sort_unstable();self.holes.dedup();for v in &mut self.by_hole{v.clear();}
        for &id in &self.holes{self.by_hole[nav.l(id)as usize].push(id);}
    }
    fn boundary(&self,id:u32)->(u32,usize){
        if let Ok(k)=self.by_id.binary_search_by_key(&id,|pair|pair.0){let i=self.by_id[k].1 as usize;(self.nodes[i].anchor,i)}
        else{assert!(id<self.n);(id,self.nodes.partition_point(|e|e.anchor<=id))}
    }
    fn position(&self,id:u32)->u32{let (t,i)=self.boundary(id);t-self.holes.partition_point(|&x|x<t)as u32+i as u32}
    fn rank(&self,c:u8,id:u32,nav:&Nav)->u32{let (t,i)=self.boundary(id);nav.rank(c,t)-self.by_hole[c as usize].partition_point(|&x|x<t)as u32+self.by_symbol[c as usize].partition_point(|&k|(k as usize)<i)as u32}
    fn before(&self,pos:u32)->(u32,usize){
        let mut lo=0;let mut hi=self.n;
        while lo<hi {
            let b=lo+(hi-lo)/2;
            let end=b+1-self.holes.partition_point(|&x|x<=b)as u32+self.nodes.partition_point(|e|e.anchor<=b)as u32;
            if end<=pos{lo=b+1;}else{hi=b;}
        }
        let first=self.nodes.partition_point(|e|e.anchor<lo);
        let start=lo-self.holes.partition_point(|&x|x<lo)as u32+first as u32;
        (lo,first+(pos-start)as usize)
    }
    fn relocate(&mut self,changes:&mut [(u32,Node)],nav:&Nav) {
        let removed:HashSet<_>=changes.iter().map(|(_,e)|e.id).collect();
        self.nodes.retain(|e|!removed.contains(&e.id));
        self.holes.extend(removed.iter().copied().filter(|&id|id<self.n));
        self.hole_index(nav);
        changes.sort_unstable_by_key(|v|v.0);
        let mut next=Vec::with_capacity(self.nodes.len()+changes.len());let mut previous=0;
        for (i,&(target,mut node)) in changes.iter().enumerate(){
            let (anchor,at)=self.before(target-i as u32);
            next.extend_from_slice(&self.nodes[previous..at]);previous=at;node.anchor=anchor;next.push(node);
        }
        next.extend_from_slice(&self.nodes[previous..]);self.nodes=next;self.reindex();
    }
}
#[derive(Default)]
struct Links {pred:HashMap<u32,u32>,succ:HashMap<u32,u32>,new:Vec<u8>,base:Vec<u32>}
impl Links {
    fn f(&self,id:u32,nav:&Nav)->u8{if id<nav.n{nav.f(id)}else{self.new[(id-nav.n)as usize]}}
    fn pred(&self,id:u32,nav:&Nav)->u32{self.pred.get(&id).copied().unwrap_or_else(||nav.lf(id))}
    fn succ(&self,id:u32,nav:&Nav)->u32{self.succ.get(&id).copied().unwrap_or_else(||nav.psi(id))}
    fn join(&mut self,a:u32,b:u32){self.succ.insert(a,b);self.pred.insert(b,a);}
}
fn setup(nav:&Nav,events:&[(u32,u8,u8)],active:&mut Vec<u32>)->(Links,Overlay) {
    let mut link=Links::default();let mut overlay=Overlay::new(nav.n);
    let mut totals:[u32;256]=std::array::from_fn(|c|nav.base[c+1]-nav.base[c]);
    for &(y,b,k) in events{
        let prev=nav.lf(y);let next=nav.psi(y);totals[k as usize]-=1;totals[b as usize]+=u32::from(k);
        overlay.holes.extend([y,next]);active.push(prev);
        let mut tail=prev;
        for _ in 0..k {
            let id=nav.n+link.new.len()as u32;link.new.push(b);overlay.nodes.push(Node{id,anchor:y,symbol:b});link.join(tail,id);active.push(id);tail=id;
        }
        link.join(tail,next);overlay.nodes.push(Node{id:next,anchor:next,symbol:b});
    }
    let mut total=0;for z in totals{link.base.push(total);total+=z;}link.base.push(total);
    overlay.nodes.sort_by_key(|e|e.anchor);overlay.hole_index(nav);overlay.reindex();
    (link,overlay)
}
fn necessary(rows:&[(u32,u32,u32)])->Vec<bool>{
    let mut targets:Vec<_>=rows.iter().map(|z|z.2).collect();targets.sort_unstable();
    let mut keep=vec![false;rows.len()];let mut previous=vec![usize::MAX;rows.len()];let mut i=0;
    while i<rows.len(){
        let gap=rows[i].1-i as u32;let mut end=i+1;while end<rows.len()&&rows[end].1-end as u32==gap{end+=1;}
        let mut tails:Vec<usize>=Vec::new();
        for j in i..end{
            let t=rows[j].2;if t-targets.partition_point(|&v|v<t) as u32!=gap{continue;}
            let k=tails.partition_point(|&a|rows[a].2<t);
            if k>0{previous[j]=tails[k-1];}if k==tails.len(){tails.push(j)}else{tails[k]=j;}
        }
        if let Some(&last)=tails.last(){let mut j=last;while j!=usize::MAX{keep[j]=true;j=previous[j];}}
        i=end;
    }keep
}
fn batch(nav:&Nav,events:&[(u32,u8,u8)],limit:usize)->Option<(Overlay,Links,usize)> {
    let mut active=Vec::new();let (link,mut order)=setup(nav,events,&mut active);
    let mut calls=0;
    while !active.is_empty(){
        calls+=active.len();if calls>limit{return None;}
        let mut rows:Vec<_>=active.iter().map(|&x|{let c=link.f(x,nav);(x,order.position(x),link.base[c as usize]+order.rank(c,link.succ(x,nav),nav))}).collect();
        rows.sort_unstable_by_key(|v|v.1);let keep=necessary(&rows);let mut changes=Vec::new();active.clear();
        for (i,(x,_,t)) in rows.into_iter().enumerate(){if !keep[i]{let prev=link.pred(x,nav);active.push(prev);changes.push((t,Node{id:x,anchor:0,symbol:link.f(prev,nav)}));}}
        if !changes.is_empty(){order.relocate(&mut changes,nav);}
    }
    Some((order,link,calls))
}
