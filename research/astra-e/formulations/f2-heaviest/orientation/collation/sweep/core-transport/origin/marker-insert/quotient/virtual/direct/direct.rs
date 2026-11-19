impl Second {
    pub(super) fn mixed(&mut self,root:&[Piece],nodes:&[Node],words:&mut [Piece],bytes:&[u8],
               flags:&[u32],z:u32,mark:Option<(u32,u32)>,m:u32,n:u32,out:&mut Vec<u8>,inspect:bool)
        ->(Option<(u32,u32)>,u32,[u32;10],bool,Option<crate::Obj>){
        let start=crate::now();
        let zi=z as usize;
        Self::reset(&mut self.count,words.len(),0);
        Self::reset(&mut self.base,words.len(),0);
        let old_row=mark.map_or(0,|(x,_)|x);
        let (mut owner,mut owner_base)=(NONE,0);
        let mut event_runs=0;
        let mut extra=0;
        for p in root {
            let node=&nodes[p.id as usize]; let k=node.word as usize;
            self.count[k]+=p.width;
            self.base[k]=node.base;
            if node.base<=old_row && (owner==NONE || node.base>owner_base) {owner=node.word;owner_base=node.base;}
            if flags[k]!=0 {event_runs+=1;extra+=p.width;}
        }
        let old_z=self.count[zi];let b=self.base[zi];let s=old_z+extra;
        let mut types=0;
        for i in 0..words.len(){
            if self.base[i]>=b+old_z {self.base[i]+=extra;}
            if flags[i]!=0 {words[i].width-=2;types+=1;}
        }
        self.count[zi]=s;
        let actual=(0..words.len()).max_by_key(|&i|(self.count[i],u32::MAX-self.base[i])).unwrap();
        let mut kind=0;
        let new_mark=mark.map(|(x,skip)|{
            let prefix=if owner==z{0}else{words[owner as usize].width};
            if (owner==z || flags[owner as usize]!=0) && skip>=prefix {
                kind=if owner==z{1}else{2};
                let mut ordinal=x-owner_base;
                let mut selected=0;
                for p in root {
                    let w=nodes[p.id as usize].word;
                    if w==owner {
                        if ordinal<p.width{return (b+selected+ordinal,skip-prefix);}
                        ordinal-=p.width;
                    }
                    if w==z || flags[w as usize]!=0 {selected+=p.width;}
                }
                unreachable!();
            }
            (x+if x>=b+old_z {extra}else{0},skip)
        });
        let make=||super::virtual_column::Virtual::new(root,nodes,flags,z,b);
        let mut length=0;
        let object=inspect.then(||{
            let mut ww=words.iter().enumerate().map(|(i,w)|(self.base[i],bytes[w.id as usize..(w.id+w.width) as usize].to_vec())).collect::<Vec<_>>();
            ww.sort_by_key(|w|w.0);
            let mut rr:Vec<(u32,u32)>=Vec::new();
            for (w,n) in make() {
                let a=self.base[w as usize];
                if let Some(last)=rr.last_mut(){if last.0==a {last.1+=n;continue;}}
                rr.push((a,n));
            }
            length=rr.len() as u32;
            (ww,rr)
        });
        let counts=[2,s,old_z,extra,event_runs,b,root.len() as u32,length,kind,types];
        let taken=if let Some((row,skip))=new_mark {
            if let Some((d,r,off))=self.direct_index(make(),actual,words,bytes,row,skip,m+extra) {
                self.phases=[0;4];self.phases[0]=crate::now()-start;
                self.virtual_finish(d,r,off,words,bytes,n,out)
            } else {false}
        } else {false};
        (new_mark,m+extra,counts,taken,object)
    }
}
