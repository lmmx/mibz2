#[derive(Clone,Copy)]
struct QEvent { before_z:u32, node:u32, width:u32 }
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    fn quotient(&mut self, old:Option<(u32,u32)>, m:u32, h:u32)
        ->(Option<(u32,u32)>,u32,[u32;10]){
        let from=self.nodes[0].out as usize;
        let len=(self.merged.len()-from) as u32;
        self.links.clear(); self.links.resize(self.words.len(),0);
        let (mut zid,mut types)=(NONE,0);
        for (i,w) in self.words.iter().enumerate(){
            if w.width>=2 && w.width<=h+1 && self.dict[(w.id+w.width-2) as usize]==b','{
                debug_assert_eq!(self.dict[(w.id+w.width-1) as usize],b'"');
                if w.width==2 {zid=i as u32;} else {self.links[i]=1;types+=1;}
            }
        }
        if zid==NONE{
            let (mark,new_m,c)=self.insert_marker(old,m,h);
            return (mark,new_m,[0,c[0],0,new_m-m,0,0,len,c[3],c[4],types]);
        }
        if types==0{return (old,m,[1,0,0,0,0,0,len,len,0,0]);}
        let root=&self.merged[from..];
        let (mut z,mut extra,mut zb,mut znode)=(0,0,0,NONE);
        let (mut owner,mut base)=(NONE,0);
        let old_row=old.map_or(0,|(row,_)|row);
        self.quot_events.clear();
        for r in root {
            let n=self.nodes[r.id as usize];
            if n.base<=old_row && (owner==NONE || n.base>base) {owner=n.word;base=n.base;}
            if n.word==zid {zb=n.base;znode=r.id;z+=r.width;}
            else if self.links[n.word as usize]!=0{
                self.quot_events.push(QEvent{before_z:z,node:r.id,width:r.width});
                extra+=r.width;
            }
        }
        assert!(z>0 && extra>0);
        for (i,w) in self.words.iter_mut().enumerate(){if self.links[i]!=0{w.width-=2;}}
        let mut kind=0;
        let marked=old.map(|(r,skip)|{
            if owner==zid{
                kind=1;
                let i=r-zb;
                let mut shift=0;
                for e in &self.quot_events {
                    if e.before_z>i{break;}
                    shift+=e.width;
                }
                return (r+shift,skip);
            }
            let length=self.words[owner as usize].width;
            if self.links[owner as usize]!=0 && skip>=length {
                kind=2;
                let mut i=r-base;
                let mut preceding=0;
                for e in &self.quot_events{
                    if self.nodes[e.node as usize].word==owner{
                        if i<e.width{return (zb+e.before_z+preceding+i,skip-length);}
                        i-=e.width;
                    }
                    preceding+=e.width;
                }
                unreachable!();
            }
            (r+if r>=zb+z{extra}else{0},skip)
        });
        // Scan A, weaving the sorted hazardous records at b + preceding-Z.
        // A's original offsets never change while its physical result is emitted.
        self.selectors.clear();
        let mut at=0;
        let mut j=0;
        for r in root{
            let n=&mut self.nodes[r.id as usize];
            let flags=&mut self.links[n.word as usize];
            let label=if *flags&1!=0 {znode}else{r.id};
            if *flags&2==0{
                if n.base>=zb+z{n.base+=extra;}
                *flags|=2;
            }
            let end=at+r.width;
            while j<self.quot_events.len(){
                let e=self.quot_events[j];let anchor=zb+e.before_z;
                if anchor>end{break;}
                assert!(at<=anchor);
                if anchor>at{sparse_append(&mut self.selectors,Piece{id:label,width:anchor-at});}
                sparse_append(&mut self.selectors,Piece{id:e.node,width:e.width});
                at=anchor;j+=1;
            }
            if at<end{sparse_append(&mut self.selectors,Piece{id:label,width:end-at});}
            at=end;
        }
        let q_runs=self.selectors.len() as u32;
        assert!(j==self.quot_events.len() && at==m && q_runs.abs_diff(len)<=2*j as u32);
        std::mem::swap(&mut self.merged,&mut self.selectors);
        self.nodes[0].out=0;
        (marked,m+extra,[2,z+extra,z,extra,j as u32,zb,len,q_runs,kind,types])
    }
    pub fn publish_quotient(&mut self, mark:Option<(u32,u32)>,m:u32,h:u32)->(Option<(u32,u32)>,u32,[u32;10]){
        self.quotient(mark,m,h)
    }
}
