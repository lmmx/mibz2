#[derive(Default,Clone,Copy)]
struct Alias {
    id:u32,
    used:u32,
}
#[derive(Default)]
struct Transport {
    source_order:Vec<usize>,
    aliases:Vec<Alias>,
    core_counts:Vec<u32>,
}
impl Transport {
    fn bytes(&self)->usize {
        bytes(&self.source_order)+bytes(&self.aliases)+bytes(&self.core_counts)
    }
}
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    fn publish_lift(&mut self,m:u32,force:bool) {
        let old_start=self.nodes[0].out as usize;
        let old_end=self.merged.len();
        let core_types=self.words.len();
        let core_runs=(old_end-old_start) as u32;
        let t=&mut self.transport;
        let cap=if force {
            t.core_counts.clear();
            t.core_counts.resize(core_types,0);
            for p in &self.merged[old_start..old_end] {t.core_counts[p.id as usize]+=p.width;}
            *t.core_counts.iter().max().unwrap()
        }else{m};
        self.nodes.clear();
        self.nodes.extend(self.bases.iter().enumerate().map(|(i,&base)|{
            let mut node=Node::new(0,NONE,0);
            node.base=base;node.word=i as u32;node
        }));
        let owners=&self.plan.owners;
        t.source_order.clear();
        t.source_order.extend(0..owners.len());
        t.source_order.sort_unstable_by_key(|&i|owners[i].base);
        t.aliases.clear();
        t.aliases.resize(owners.len(),Alias::default());
        let mut begin=0;
        let mut bins=0;
        while begin<t.source_order.len() {
            let first=owners[t.source_order[begin]];
            let mut end=begin+1;
            while end<t.source_order.len() {
                let next=owners[t.source_order[end]];
                if next.depth!=first.depth || next.prefix!=first.prefix {break;}
                end+=1;
            }
            let stop=t.source_order.get(end).map_or(m,|&i|owners[i].base);
            let base_id=self.words.len() as u32;
            // All aliases of this prefix bin reuse a single positive or empty byte slice.
            let spelling=Piece{id:self.dict.len() as u32,width:u32::from(first.depth)};
            self.dict.extend_from_slice(&first.prefix[..first.depth as usize]);
            let mut rank=first.base;
            while rank<stop {
                let id=self.words.len() as u32;
                self.words.push(spelling);
                let mut node=Node::new(0,NONE,0);
                node.base=m+rank;node.word=id;self.nodes.push(node);
                rank+=cap;
            }
            for &lane in &t.source_order[begin..end] {
                let offset=owners[lane].base-first.base;
                t.aliases[lane]=Alias{id:base_id+offset/cap,used:offset%cap};
            }
            bins+=1;
            begin=end;
        }
        // V-destination half receives prefix IDs. Original incoming core RLE is
        // already in the required U-destination order and is copied as one slice.
        self.nodes[0].out=old_end as u32;
        for (i,owner) in owners.iter().enumerate(){
            let mut width=owners.get(i+1).map_or(m,|o|o.v)-owner.v;
            let mut a=t.aliases[i];
            while width>0{
                let take=width.min(cap-a.used);
                Self::emit(&mut self.merged,old_end,a.id,take);
                width-=take;a.id+=1;a.used=0;
            }
        }
        let prefix_runs=(self.merged.len()-old_end) as u32;
        self.merged.extend_from_within(old_start..old_end);
        self.measures[5]=bins;
        self.measures[6]=prefix_runs;
        self.measures[7]=core_runs;
        self.alias_count=self.words.len() as u32-core_types as u32-bins;
        self.core_type_count=core_types as u32;
        #[cfg(shift_counts)] {self.prefix_bytes=self.words[core_types..].iter().map(|v|v.width).sum();}
    }
}
