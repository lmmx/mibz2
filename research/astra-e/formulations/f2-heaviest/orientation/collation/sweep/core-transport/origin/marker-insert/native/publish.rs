// The original public token represents B = R + quote when W = quote + R.
// A selected trailing comma in W is B's penultimate byte. Its physical
// outgoing marker spells comma + quote; old rotated intervals retain their ranks.
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    pub fn insert_marker(&mut self, old_mark:Option<(u32,u32)>, m:u32, h:u32)
        ->(Option<(u32,u32)>,u32,[u32;5]) {
        let root_start=self.nodes[0].out as usize;
        let old_root=&self.merged[root_start..];
        let k=self.words.len();
        self.links.clear();
        self.links.resize(k,0);
        // h=0 is the matched full-old-word control. Count its masses on the paid path.
        let mut full_winner=NONE;
        if h==0 {
            let mut best=(0,0);
            for r in old_root {
                let n=self.nodes[r.id as usize];
                let count=&mut self.links[n.word as usize];
                *count+=r.width;
                let key=(*count,u32::MAX-n.base);
                if key>best{best=key;full_winner=n.word;}
            }
        }
        let mut marker=Piece::default();
        let mut selected_types=0;
        for (id,w) in self.words.iter_mut().enumerate(){
            let width=if h==0 {
                if id as u32==full_winner {w.width} else {0}
            }else if w.width>=2 && w.width<=h+1 && self.dict[(w.id+w.width-2) as usize]==b',' {
                debug_assert_eq!(self.dict[(w.id+w.width-1) as usize],b'"');
                2
            }else{0};
            self.links[id]=u32::from(width!=0);
            if width!=0{
                marker=Piece{id:w.id+w.width-width,width};
                w.width-=width;
                selected_types+=1;
            }
        }
        if selected_types==0{return (old_mark,m,[0,0,0,old_root.len() as u32,0]);}
        let mut s=0;
        self.selectors.clear();
        let old_row=old_mark.map_or(0,|(r,_)|r);
        let mut owner_base=0;
        let mut owner=NONE;
        for &r in old_root{
            let n=self.nodes[r.id as usize];
            if n.base<=old_row && (owner==NONE || n.base>owner_base){
                owner=n.word;owner_base=n.base;
            }
            if self.links[n.word as usize]!=0{
                sparse_append(&mut self.selectors,r);
                s+=r.width;
            }
        }
        let filtered=self.selectors.len();
        let mut new_mark=old_mark.map(|(r,b)|(s+r,b));
        let mut remap=None;
        if let Some((r,b))=old_mark{
            let prefix=self.words[owner as usize].width;
            if self.links[owner as usize]!=0 && b>=prefix{
                remap=Some((r-owner_base,b-prefix));
            }
        }
        let marker_node=self.nodes.len() as u32;
        let mut marker_n=Node::new(0,NONE,0);
        marker_n.base=0;
        marker_n.word=k as u32;
        self.nodes.push(marker_n);
        let mut before=0;
        let transferred=remap.is_some() as u32;
        for r in old_root{
            let n=&mut self.nodes[r.id as usize];
            let flags=&mut self.links[n.word as usize];
            let selected=*flags&1!=0;
            if *flags&2==0 {n.base+=s;*flags|=2;}
            if selected{
                if n.word==owner{
                    if let Some((ord,skip))=remap{
                        if ord<r.width {new_mark=Some((before+ord,skip));remap=None;}
                        else{remap=Some((ord-r.width,skip));}
                    }
                }
                before+=r.width;
            }
            sparse_append(&mut self.selectors,Piece{id:if selected{marker_node}else{r.id},width:r.width});
        }
        assert!(remap.is_none() && before==s);
        self.words.push(marker);
        let new_runs=self.selectors.len() as u32;
        std::mem::swap(&mut self.merged,&mut self.selectors);
        self.nodes[0].out=0;
        (new_mark,m+s,[s,selected_types,filtered as u32,new_runs,transferred])
    }
}
fn sparse_append(runs:&mut Vec<Piece>,p:Piece){
    if let Some(last)=runs.last_mut(){
        if last.id==p.id{last.width+=p.width;return;}
    }
    runs.push(p);
}
