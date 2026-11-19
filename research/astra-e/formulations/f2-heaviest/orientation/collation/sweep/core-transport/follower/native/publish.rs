#[derive(Clone,Copy)]
struct Previous {
    owner: usize,
    word: u32,
    #[cfg(debug_assertions)]
    moved_end: u32,
}
#[derive(Default)]
struct Transport {
    // The old-source sweep produces consecutive fragment ranges for each physical owner.
    source_order: Vec<u32>,
    fragments: Vec<Piece>,
    ranges: Vec<(u32,u32)>,
    previous: Vec<Option<Previous>>,
    words: Vec<Piece>,
    dict: Vec<u8>,
}
impl Transport {
    fn bytes(&self)->usize {
        bytes(&self.source_order)+bytes(&self.fragments)+bytes(&self.ranges)
            +bytes(&self.previous)+bytes(&self.words)+bytes(&self.dict)
    }
}
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    fn publish(&mut self,m:u32) {
        let start=self.nodes[0].out as usize;
        let end=self.merged.len();
        self.measures[7]=(end-start) as u32;
        self.nodes.clear();
        let owners=&self.plan.owners;
        // At cap zero every physical coordinate agrees with the original coordinate.
        if owners.len()==1 && owners[0].depth==0 {
            self.nodes.extend(self.bases.iter().enumerate().map(|(i,&base)|{
                let mut node=Node::new(0,NONE,0);node.base=base;node.word=i as u32;node
            }));
            self.nodes[0].out=start as u32;
            self.measures[5]=(end-start) as u32;
            self.measures[6]=(end-start) as u32;
            return;
        }
        let t=&mut self.transport;
        t.source_order.clear();
        t.source_order.extend(0..owners.len() as u32);
        t.source_order.sort_unstable_by_key(|&i|owners[i as usize].base);
        t.ranges.clear();
        t.ranges.resize(owners.len(),(0,0));
        t.previous.clear();
        t.previous.resize(self.bases.len(),None);
        t.fragments.clear();
        t.words.clear();
        t.dict.clear();
        let mut j=0;
        let mut dest=0;
        let mut lane_start=0;
        let mut fragments=0;
        for k in start..end {
            let p=self.merged[k];
            let core=p.id as usize;
            let until=dest+p.width;
            while dest<until {
                let lane=t.source_order[j] as usize;
                let owner=owners[lane];
                let lane_end=t.source_order.get(j+1).map_or(m,|&i|owners[i as usize].base);
                let stop=until.min(lane_end);
                debug_assert!(owner.base<=dest && dest<stop);
                let previous=t.previous[core].filter(|prev|{
                    let old=owners[prev.owner];
                    old.depth==owner.depth && old.prefix==owner.prefix
                });
                let word=if let Some(prev)=previous {
                    #[cfg(debug_assertions)]
                    assert!(owner.v+dest-owner.base>=prev.moved_end);
                    prev.word
                } else {
                    let id=t.words.len() as u32;
                    let base=self.bases[core];
                    let w=self.words[core];
                    let offset=t.dict.len() as u32;
                    t.dict.extend_from_slice(&self.dict[w.id as usize..(w.id+w.width) as usize]);
                    t.dict.extend_from_slice(&owner.prefix[..owner.depth as usize]);
                    t.words.push(Piece{id:offset,width:w.width+u32::from(owner.depth)});
                    let mut node=Node::new(0,NONE,0);
                    node.word=id;
                    node.base=base;
                    self.nodes.push(node);
                    id
                };
                t.previous[core]=Some(Previous{owner:lane,word,
                    #[cfg(debug_assertions)] moved_end:owner.v+stop-owner.base});
                self.bases[core]+=stop-dest;
                Self::emit(&mut t.fragments,lane_start,word,stop-dest);
                fragments+=1;
                dest=stop;
                if dest==lane_end {
                    t.ranges[lane]=(lane_start as u32,t.fragments.len() as u32);
                    lane_start=t.fragments.len();
                    j+=1;
                }
            }
        }
        debug_assert_eq!(j,owners.len());
        debug_assert_eq!(dest,m);
        self.nodes[0].out=end as u32;
        for &(lo,hi) in &t.ranges {
            for p in &t.fragments[lo as usize..hi as usize] {
                Self::emit(&mut self.merged,end,p.id,p.width);
            }
        }
        std::mem::swap(&mut self.words,&mut t.words);
        std::mem::swap(&mut self.dict,&mut t.dict);
        self.measures[5]=fragments;
        self.measures[6]=(self.merged.len()-end) as u32;
        debug_assert!(self.measures[6]<=fragments && fragments<=self.measures[7]+owners.len() as u32-1);
    }
}
