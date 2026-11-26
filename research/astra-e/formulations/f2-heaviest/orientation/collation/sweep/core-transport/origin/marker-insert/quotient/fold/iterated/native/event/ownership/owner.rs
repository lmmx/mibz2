#[derive(Default)]
pub struct Owners { offsets:Vec<u32> }
#[derive(Clone,Copy,Default)]
pub struct OwnerCounts {
    pub visits:u32,pub leaf_bytes:u32,pub copied:u32,pub hits:u32,pub partial_hits:u32,
    pub partial_misses:u32,pub published:u32,pub slots:u32,pub bytes:usize,
}
impl Owners{
    fn range<const AUDIT:bool>(&mut self,g:&mut Grammar,leaves:u32,id:u32,lo:u32,hi:u32,dict:&[u8],
                               out:&mut Vec<u8>,meter:&mut Meter,m:&mut OwnerCounts){
        if lo==hi{return;}
        g.stack.clear();g.stack.push((id,lo,hi));
        while let Some((k,a,b))=g.stack.pop(){
            let node=g.dag[k as usize];if AUDIT{m.visits+=1;}
            if node.b==NONE{
                out.extend_from_slice(&dict[(node.a+a)as usize..(node.a+b)as usize]);
                if AUDIT{m.leaf_bytes+=b-a;meter.leaf_visits+=1;}
            }else{
                let slot=&mut self.offsets[(k-leaves)as usize];
                let full=a==0 && b==node.size;
                if *slot!=NONE{
                    let start=*slot;
                    // The cached source is a preceding *complete* occurrence.
                    if AUDIT{assert!(start+node.size<=out.len() as u32);}
                    out.extend_from_within((start+a)as usize..(start+b)as usize);
                    if AUDIT{m.copied+=b-a;m.hits+=1;m.partial_hits+=u32::from(!full);}
                }else{
                    if full{
                        // Both children are strictly older, so no lookup for this
                        // node can recur before this complete occurrence is emitted.
                        *slot=out.len()as u32;if AUDIT{m.published+=1;}
                    }else if AUDIT{m.partial_misses+=1;}
                    let left=g.dag[node.a as usize].size;
                    if b>left{g.stack.push((node.b,a.saturating_sub(left),b-left));}
                    if a<left{g.stack.push((node.a,a,b.min(left)));}
                    if AUDIT{meter.expansions+=1;meter.peak_stack=meter.peak_stack.max(g.stack.len());}
                }
            }
        }
    }
}
impl Event{
    pub fn owner_capacity(&self)->usize{bytes(&self.owners.offsets)}
    pub fn replay<const USE:bool,const AUDIT:bool>(&mut self,dict:&[u8],out:&mut Vec<u8>,m:&mut Meter)->OwnerCounts{
        let(id,skip,n,leaves)=self.emit_root.unwrap();let size=self.grammar.dag[id as usize].size;
        assert!(skip<size && size<=n);
        let mut counts=OwnerCounts::default();out.clear();out.reserve(n as usize);
        if USE{
            self.owners.offsets.clear();self.owners.offsets.resize(self.grammar.dag.len()-leaves as usize,NONE);
            self.owners.range::<AUDIT>(&mut self.grammar,leaves,id,skip,size,dict,out,m,&mut counts);
            self.owners.range::<AUDIT>(&mut self.grammar,leaves,id,0,skip,dict,out,m,&mut counts);
            if AUDIT{assert_eq!(counts.copied+counts.leaf_bytes,size);}
            counts.slots=self.owners.offsets.len() as u32;counts.bytes=self.owner_capacity();
        }else{
            self.grammar.range::<AUDIT>(id,skip,size,dict,out,m);
            self.grammar.range::<AUDIT>(id,0,skip,dict,out,m);
        }
        assert_eq!(out.len(),size as usize);m.period=size;m.terminal=true;
        while out.len()<n as usize{let take=out.len().min(n as usize-out.len());out.extend_from_within(..take);}
        counts
    }
    pub fn again<const USE:bool,const AUDIT:bool,const M:u8,const S:u8>(&mut self,w:&Dispatch<M,S>,
            out:&mut Vec<u8>,m:&mut Meter)->OwnerCounts{self.replay::<USE,AUDIT>(&w.dict,out,m)}
}

pub fn ownership_fixtures()->[u64;5]{
    let mut e=Event::default();let g=&mut e.grammar.dag;
    // Nonpalindromic multi-byte leaves, duplicate children, overlapping subgrammars.
    let bytes=b"abcXYZ!";
    for (a,size) in [(0,3),(3,3),(6,1)]{g.push(Production{a,b:NONE,size});}
    let mut truth=vec![b"abc".to_vec(),b"XYZ".to_vec(),b"!".to_vec()];
    for (a,b) in [(0,1),(3,3),(4,2),(5,4),(6,3),(6,7),(8,8)]{
        let mut text=truth[a].clone();text.extend_from_slice(&truth[b]);
        g.push(Production{a:a as u32,b:b as u32,size:text.len()as u32});truth.push(text);
    }
    let mut counts=[0u64;5];let mut out=vec![];
    for (k,w) in truth.iter().enumerate(){
        for skip in 0..w.len(){for extra in [0,1,w.len(),w.len()*2+1]{
            let n=w.len()+extra;let want=(0..n).map(|p|w[(p+skip)%w.len()]).collect::<Vec<_>>();
            e.emit_root=Some((k as u32,skip as u32,n as u32,3));
            let m=e.replay::<true,true>(bytes,&mut out,&mut Meter::default());
            assert_eq!(out,want);
            e.replay::<false,true>(bytes,&mut out,&mut Meter::default());assert_eq!(out,want);
            counts[0]+=1;counts[1]+=u64::from(m.hits);counts[2]+=u64::from(m.partial_hits);
            counts[3]+=u64::from(m.partial_misses);counts[4]+=u64::from(m.copied);
        }}
    }
    assert!(counts[2]>0 && counts[3]>0);counts
}
