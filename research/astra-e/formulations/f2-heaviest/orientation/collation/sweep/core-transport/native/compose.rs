#[derive(Default,Clone,Copy)]
struct Slice {
    v: u32,
    source: u32,
    owner: u32,
    core: u32,
    word: u32,
}

#[derive(Default)]
struct Transport {
    slices: Vec<Slice>,
    source_order: Vec<u32>,
    core_order: Vec<u32>,
    next_slice: Vec<u32>,
    words: Vec<Piece>,
    dict: Vec<u8>,
}
impl Transport {
    fn bytes(&self)->usize {
        bytes(&self.slices)+bytes(&self.source_order)+bytes(&self.core_order)
            +bytes(&self.next_slice)+bytes(&self.words)+bytes(&self.dict)
    }
}

impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    fn original_mark(&self,p:u32,u:(u32,u32))->(Option<(u32,u32)>,u32) {
        if (u.0..u.1).contains(&p) {return (Some((p-u.0,0)),0);}
        if let Some(mark)=self.plan.hidden_mark {return (Some(mark),0);}
        let start=self.plan.forbid.complement(p);
        let mut x=start;
        let mut steps=0;
        loop {
            let r=self.runs[self.locate(x)];
            x=r.map(x);
            steps+=1;
            if r.terminal() {
                let i=self.plan.owners.partition_point(|o|o.v<=x)-1;
                let o=self.plan.owners[i];
                return (Some((o.base+x-o.v,steps+u32::from(o.depth))),0);
            }
            if x==start {return (None,start);}
        }
    }

    fn compose(&mut self,m:u32) {
        let root_start=self.nodes[0].out as usize;
        let root_end=self.merged.len();
        let owners=&self.plan.owners;
        let t=&mut self.transport;
        // Stable core buckets have known starts in compressed physical V order.
        t.core_order.clear();
        t.core_order.extend(0..self.bases.len() as u32);
        t.core_order.sort_unstable_by_key(|&i|self.bases[i as usize]);
        t.next_slice.clear();
        t.next_slice.resize(self.bases.len(),0);
        t.slices.clear();
        let mut j=0;
        for (k,&core) in t.core_order.iter().enumerate() {
            let i=core as usize;
            let mut v=self.bases[i];
            let end=t.core_order.get(k+1).map_or(m,|&next|self.bases[next as usize]);
            debug_assert!(v<end);
            t.next_slice[i]=t.slices.len() as u32;
            while v<end {
                let owner=owners[j];
                let edge=owners.get(j+1).map_or(m,|o|o.v);
                let stop=end.min(edge);
                debug_assert!(owner.v<=v && v<stop);
                t.slices.push(Slice{v,source:owner.base+v-owner.v,owner:j as u32,core,word:NONE});
                v=stop;
                if v==edge {j+=1;}
            }
        }
        debug_assert_eq!(j,owners.len());
        // Sorting the small core-bucket/atlas overlay by original source rank
        // makes equal full words adjacent. No full-word hashing is required.
        t.source_order.clear();
        t.source_order.extend(0..t.slices.len() as u32);
        t.source_order.sort_unstable_by_key(|&i|t.slices[i as usize].source);
        t.dict.clear();
        t.words.clear();
        self.nodes.clear();
        let mut previous=None;
        for &i in &t.source_order {
            let s=&mut t.slices[i as usize];
            let owner=owners[s.owner as usize];
            let key=(s.core,owner.depth,owner.prefix);
            if previous!=Some(key) {
                let offset=t.dict.len() as u32;
                let core=self.words[s.core as usize];
                t.dict.extend_from_slice(&owner.prefix[..owner.depth as usize]);
                t.dict.extend_from_slice(&self.dict[core.id as usize..(core.id+core.width) as usize]);
                t.words.push(Piece{id:offset,width:core.width+u32::from(owner.depth)});
                let mut node=Node::new(0,NONE,0);
                node.word=(t.words.len()-1) as u32;
                node.base=s.source;
                self.nodes.push(node);
                previous=Some(key);
            }
            s.word=(t.words.len()-1) as u32;
        }
        self.nodes[0].out=root_end as u32;
        let mut segments=0u32;
        for at in root_start..root_end {
            let p=self.merged[at];
            let core=p.id as usize;
            let mut v=self.bases[core];
            let mut j=t.next_slice[core] as usize;
            let end=v+p.width;
            while v<end {
                let s=t.slices[j];
                let edge=t.slices.get(j+1).map_or(m,|s|s.v);
                let stop=end.min(edge);
                debug_assert!(s.v<=v && s.core==p.id && v<stop);
                Self::emit(&mut self.merged,root_end,s.word,stop-v);
                segments+=1;
                v=stop;
                if v==edge {j+=1;}
            }
            t.next_slice[core]=j as u32;
            self.bases[core]=v;
        }
        std::mem::swap(&mut self.words,&mut t.words);
        std::mem::swap(&mut self.dict,&mut t.dict);
        self.measures[5]=t.slices.len() as u32;
        self.measures[6]=segments;
        self.measures[7]=(root_end-root_start) as u32;
        debug_assert_eq!(segments as usize,self.merged.len()-root_end);
    }

    pub fn first(&mut self,caps:&[u16],freq:&[u32;256],n:u32,p:u32,u:(u32,u32),h:usize)
        ->Option<(Option<(u32,u32)>,u32,u32)> {
        if u.0>=u.1 || u.1>n || h>8 {return None;}
        let t0=crate::now();
        let prior_atlas=self.plan.bytes();
        self.plan=Atlas::prepare(caps,freq,u,h,p);
        self.setup_scratch=prior_atlas+self.plan.workspace_peak;
        self.measures=[0;8];
        self.projected_index(caps,freq,n);
        let m=u.1-u.0;
        let (mark,missing)=self.original_mark(p,u);
        let t1=crate::now();
        self.build::<false>(self.plan.root.0,m);
        let t2=crate::now();
        self.inverse_merge::<false>();
        let t3=crate::now();
        self.compose(m);
        let end=crate::now();
        self.phases=[t1-t0,t2-t1,end-t2];
        self.compose_ns=end-t3;
        self.measures[1]=self.runs.len() as u32;
        self.measures[2]=self.plan.intersections;
        self.measures[3]=self.plan.owners.len() as u32;
        self.measures[4]=self.plan.forbid.mass;
        Some((mark,m,missing))
    }
}
