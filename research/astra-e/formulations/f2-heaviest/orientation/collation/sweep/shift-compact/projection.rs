#[derive(Clone,Copy,Default,Debug)]
struct Part{lo:u32,hi:u32,base:u32}
#[derive(Default,Clone)]
struct Section{parts:Vec<Part>,mass:u32}
impl Section{
    fn push(&mut self,lo:u32,hi:u32){
        if lo>=hi{return;}
        if let Some(p)=self.parts.last_mut(){
            assert!(p.hi<=lo);
            if p.hi==lo{p.hi=hi;self.mass+=hi-lo;return;}
        }
        self.parts.push(Part{lo,hi,base:self.mass});self.mass+=hi-lo;
    }
    fn after(&self,x:u32)->usize{self.parts.partition_point(|p|p.hi<=x)}
    fn rank(&self,x:u32)->Option<u32>{
        let s=self.parts.get(self.after(x))?;
        (s.lo<=x).then(||s.base+x-s.lo)
    }
    fn complement(&self,x:u32)->u32{
        let removed=self.parts.get(self.after(x)).map_or(self.mass,|p|p.base+x.saturating_sub(p.lo));
        x-removed
    }
    fn row(&self,t:u32)->u32{
        let k=self.parts.partition_point(|p|p.base<=t)-1;
        self.parts[k].lo+t-self.parts[k].base
    }
}
#[derive(Clone,Copy)]
struct Early{lo:u32,hi:u32,word:u32}
#[derive(Default)]
struct Plan{
    levels:Vec<Section>,forbid:Section,prefix:Vec<u8>,
    early:Vec<Early>,types:Vec<(usize,u32)>,root:(u32,u32),
}
impl Plan{
    fn target(&self)->&Section{self.levels.last().unwrap()}
    fn prepare(caps:&[u16],freq:&[u32;256],u:(u32,u32),depth:usize)->Option<Self>{
        let mut p=Self::default();
        let mut initial=Section::default();initial.push(u.0,u.1);
        p.levels.push(initial);
        let mut total=0;let start=freq.map(|w|{let a=total;total+=w;a});
        if u.0>=u.1 || u.1>total{return None;}
        for _ in 0..depth{
            let previous=p.target();
            let a=previous.parts[0].lo;let b=previous.parts.last().unwrap().hi;
            let sym=(0..256).find(|&c|start[c]<=a && a<start[c]+freq[c]).unwrap();
            if b>start[sym]+freq[sym]{return None;}
            // Stable select for just this prefix byte on a monotone list of ranges.
            let mut f=start[sym];let mut l=0;let mut i=0;let mut next=Section::default();
            for &rec in caps{
                let w=u32::from(rec&255);
                if usize::from(rec>>8)==sym{
                    while i<previous.parts.len() && previous.parts[i].lo<f+w{
                        let seg=previous.parts[i];let lo=f.max(seg.lo);let hi=(f+w).min(seg.hi);
                        next.push(l+lo-f,l+hi-f);
                        if hi<seg.hi{break;} i+=1;
                    }
                    f+=w;
                }
                l+=w;if i==previous.parts.len(){break;}
            }
            assert_eq!(next.mass,previous.mass);p.levels.push(next);p.prefix.push(sym as u8);
        }
        let mut all=p.levels.iter().skip(1).flat_map(|l|l.parts.iter().map(|s|(s.lo,s.hi))).collect::<Vec<_>>();
        all.sort_unstable();
        for (lo,hi) in all{
            let begin=p.forbid.parts.last().map_or(lo,|s|s.hi.max(lo));
            if begin<hi{p.forbid.push(begin,hi);}
        }
        // Find exactly the early positive returns skipped by the common prefix.
        // Intersect each earlier level with target, restricted to surviving destination ordinals.
        let target=p.target();let mut early=Vec::new();let mut types=Vec::new();
        let mut alive=Section::default();alive.push(0,u.1-u.0);
        for h in (0..depth).rev(){
            let level=&p.levels[h];let mut new=Section::default();let mut ty=None;
            let(mut l,mut v)=(0,0);
            for q in &alive.parts{
                let mut t=q.lo;
                while t<q.hi{
                    while level.parts[l].base+level.parts[l].hi-level.parts[l].lo<=t{l+=1;}
                    let part=level.parts[l];let x=part.lo+t-part.base;
                    let mut end=q.hi.min(t+part.hi-x);
                    while v<target.parts.len() && target.parts[v].hi<=x{v+=1;}
                    if let Some(z)=target.parts.get(v){
                        if x>=z.lo{
                            end=end.min(t+z.hi-x);let base=z.base+x-z.lo;
                            let id=*ty.get_or_insert_with(||{
                                let id=types.len() as u32;types.push((h,base));id
                            });
                            early.push(Early{lo:t,hi:end,word:id});t=end;continue;
                        }
                        end=end.min(t+z.lo-x);
                    }
                    new.push(t,end);t=end;
                }
            }
            alive=new;
        }
        p.root=(p.forbid.complement(u.0),p.forbid.complement(u.1));
        assert_eq!(alive.mass,p.root.1-p.root.0);
        early.sort_unstable_by_key(|x|x.lo);
        p.early=early;p.types=types;
        Some(p)
    }
    fn mono(&self,freq:&[u32;256])->bool{
        if !self.early.is_empty(){return false;}
        let t=self.target();let lo=t.parts[0].lo;let hi=t.parts.last().unwrap().hi;
        let mut end=0;
        for &w in freq{end+=w;if lo<end{return hi<=end;}}
        unreachable!()
    }
}
