// The input alphabet has one positive contiguous source bucket per ID. Runs are in
// destination order. Under the stable occurrence rule the j-th occurrence of ID a
// has predecessor bases[a]+j. Original dictionary ranges can alias or overlap.
#[repr(C)]
#[derive(Clone,Copy,Default,Debug,PartialEq,Eq)]
pub struct Piece {pub id:u32,pub width:u32}
#[derive(Clone,Default)]
pub struct Obj {pub runs:Vec<Piece>,pub words:Vec<Piece>,pub bases:Vec<u32>,pub m:u32}
pub enum Mark {New(u32,u32),Old(u32,u32)}
pub struct Patch {
    pub obj:Option<Obj>,pub holes:Vec<u32>,
    // For each deleted vertex in a live chain, its old live owner and byte offset.
    pub owner:Vec<Option<(u32,u32)>>,pub changes:u32,pub compound_bytes:u32,
    pub visits:usize,pub segments:usize,
}

impl Patch {
    pub fn mark(&self,r:u32,skip:u32)->Mark {
        let (x,k)=match self.holes.binary_search(&r) {
            Err(_)=>(r,skip),
            Ok(i)=>match self.owner[i] {
                Some((s,offset))=>(s,offset.checked_add(skip).unwrap()),
                None=>return Mark::Old(r,skip)
            }
        };
        Mark::New(x-self.holes.partition_point(|&z|z<x) as u32,k)
    }
}
fn push_run(r:&mut Vec<Piece>,id:u32,width:u32) {
    if width==0{return;}
    if let Some(last)=r.last_mut() {
        if last.id==id {last.width+=width;return;}
    }
    r.push(Piece{id,width});
}
fn copy_old(arena:&mut Vec<u8>,w:Piece) {
    let l=w.id as usize;
    arena.extend_from_within(l..l+w.width as usize);
}
#[derive(Clone,Copy)]
struct Segment {end:u32,id:Option<u32>}

// Each source ID has positive mass. Physical source rank order, including arbitrary
// numeric ID permutations, is established once without materializing a row array.
pub fn splice(j:&Obj,arena:&mut Vec<u8>,holes:&[u32])->Patch {
    splice_with_reserve::<false>(j,arena,holes)
}
pub fn splice_with_reserve<const EXACT:bool>(j:&Obj,arena:&mut Vec<u8>,holes:&[u32])->Patch {
    let e=holes.len();
    let mut result=Patch{obj:None,holes:holes.to_vec(),owner:vec![None;e],changes:0,
                         compound_bytes:0,visits:0,segments:0};
    if e==0 {return result;} // unchanged views into J
    assert!(holes.windows(2).all(|w|w[0]<w[1]) && holes[e-1]<j.m);
    let k=j.bases.len();
    assert_eq!(k,j.words.len());
    let mut order:Vec<_>=(0..k).collect();
    // The ordinary physical alphabet uses sorted bases. The same API supports a
    // numerical ID permutation, in which case this metadata sort is charged.
    if !j.bases.windows(2).all(|w|w[0]<w[1]) {order.sort_unstable_by_key(|&a|j.bases[a]);}
    let mut owner_id=vec![0usize;e];
    let mut next_hole=vec![0usize;k];
    let mut end_hole=vec![0usize;k];
    let mut ends=vec![0u32;k];
    let mut first=0;
    for (rank,&a) in order.iter().enumerate(){
        let end=order.get(rank+1).map_or(j.m,|&b|j.bases[b]);
        ends[a]=end;next_hole[a]=first;
        while first<e && holes[first]<end {owner_id[first]=a;first+=1;}
        end_hole[a]=first;
    }
    let (mut pred,mut succ)=(vec![0;e],vec![0;e]);
    let mut cur=j.bases.clone();
    let (mut y,mut idx)=(0u32,0usize);
    for run in &j.runs {
        let a=run.id as usize;
        let x=cur[a];
        while idx<e && holes[idx]<y+run.width {
            pred[idx]=x+(holes[idx]-y);idx+=1;
        }
        while next_hole[a]<end_hole[a] && holes[next_hole[a]]<x+run.width {
            let h=next_hole[a];succ[h]=y+(holes[h]-x);next_hole[a]+=1;
        }
        cur[a]+=run.width;y+=run.width;
    }
    assert_eq!(y,j.m);
    let mut changes=Vec::new();
    for (i,&s) in pred.iter().enumerate() {
        if holes.binary_search(&s).is_err(){changes.push((s,i));}
    }
    changes.sort_unstable_by_key(|&(s,_)|s);
    result.changes=changes.len() as u32;
    let mut out=Obj{m:j.m-e as u32,..Obj::default()};
    let mut cuts=Vec::<Segment>::with_capacity(k+2*changes.len());
    let mut piece_index=vec![0usize;k];
    let mut events:Vec<(u32,Option<u32>)>=holes.iter().map(|&r|(r,None)).collect();
    let (mut ci,mut hi,mut new_rank)=(0,0,0);
    let old_bytes=arena.len();
    let mut suffix=0usize;
    let mut copy_plan=Vec::<Piece>::new();
    let mut append=|arena:&mut Vec<u8>,w:Piece|{
        if EXACT {copy_plan.push(w);}else{copy_old(arena,w);}
    };
    for &a in &order {
        piece_index[a]=cuts.len();
        let mut x=j.bases[a];
        let end=ends[a];
        while x<end {
            let stop=changes.get(ci).map_or(end,|(s,_)|(*s).min(end));
            if stop>x {
                let h0=hi;
                while hi<e && holes[hi]<stop {assert!(holes[hi]>=x);hi+=1;}
                let width=stop-x-(hi-h0) as u32;
                let label=if width>0 {
                    let id=out.words.len() as u32;out.words.push(j.words[a]);
                    out.bases.push(new_rank);new_rank+=width;Some(id)
                } else {None};
                cuts.push(Segment{end:stop,id:label});
                x=stop;
            }
            if x==end {break;}
            let (s,mut hole)=changes[ci];assert_eq!(x,s);
            let new_id=out.words.len() as u32;
            let start=old_bytes+suffix;
            append(arena,j.words[a]);suffix+=j.words[a].width as usize;
            let landing=loop {
                assert!(result.owner[hole].is_none());
                result.owner[hole]=Some((s,(old_bytes+suffix-start) as u32));
                let w=j.words[owner_id[hole]];
                append(arena,w);suffix+=w.width as usize;
                match holes.binary_search(&succ[hole]) {
                    Ok(h)=>hole=h,Err(_)=>break succ[hole]
                }
            };
            events.push((landing,Some(new_id)));
            out.words.push(Piece{id:start.try_into().unwrap(),width:(old_bytes+suffix-start).try_into().unwrap()});
            out.bases.push(new_rank);new_rank+=1;
            cuts.push(Segment{end:x+1,id:Some(new_id)});
            ci+=1;x+=1;
        }
    }
    assert_eq!(new_rank,out.m);
    if EXACT {
        arena.reserve_exact(suffix);
        for w in copy_plan {copy_old(arena,w);}
    }
    assert_eq!(arena.len(),old_bytes+suffix);
    result.compound_bytes=suffix.try_into().unwrap();
    result.segments=cuts.len();
    events.sort_unstable_by_key(|v|v.0);
    assert!(events.windows(2).all(|w|w[0].0!=w[1].0));
    let mut event=0;
    cur.clone_from(&j.bases);
    y=0;
    for run in &j.runs {
        let a=run.id as usize;
        let stop=y+run.width;
        while y<stop {
            result.visits+=1;
            let site=cuts[piece_index[a]];
            let width=(stop-y).min(site.end-cur[a]);
            assert!(width>0);
            let end=y+width;
            while event<events.len() && events[event].0<end {
                let (pos,value)=events[event];assert!(pos>=y);
                if y<pos {push_run(&mut out.runs,site.id.unwrap(),pos-y);}
                if let Some(id)=value {push_run(&mut out.runs,id,1);}
                y=pos+1;event+=1;
            }
            if y<end {push_run(&mut out.runs,site.id.unwrap(),end-y);}
            y=end;cur[a]+=width;
            if cur[a]==site.end {piece_index[a]+=1;}
        }
    }
    assert_eq!(event,events.len());
    assert!(out.words.len()<=k+2*changes.len() && out.runs.len()<=j.runs.len()+3*changes.len());
    result.obj=Some(out);
    result
}
