impl Second {
    // The trie root has a single incoming query. Reduce its pieces directly
    // into the ordinary first child nodes, terminal word, and selectors.
    fn root_piece(&mut self, word:u32, width:u32, mapped:u32, d:u32, words:&[Piece], bytes:&[u8]) {
        if width==0 {return;}
        let id=if word==d {
            if self.nodes[0].word==NONE {
                self.nodes[0].base=mapped;
                self.nodes[0].word=0;
                let spelling=words[d as usize];
                self.dictionary.extend_from_slice(&bytes[spelling.id as usize..(spelling.id+spelling.width) as usize]);
                self.words.push(Piece{id:0,width:spelling.width});
            }
            NONE
        } else {
            let c=word as usize;
            if self.epoch[c]!=0 {
                self.epoch[c]=0;
                self.child[c]=self.nodes.len();
                self.nodes.push(Self::fresh_node(mapped,0,word));
            }
            let id=self.child[c];
            debug_assert_eq!(self.nodes[id].hi,mapped);
            self.nodes[id].hi+=width;
            id as u32
        };
        if let Some(last)=self.selectors.last_mut() {
            if last.id==id {last.width+=width;return;}
        }
        self.selectors.push(Piece{id,width});
    }

    fn root_index(&mut self, root:&[Piece], nodes:&[Node], words:&[Piece], bytes:&[u8],
                  p:u32, mut skip:u32, m:u32)->Option<(u32,u32,u32)> {
        let alphabet=words.len();
        Self::reset(&mut self.count,alphabet,0);
        Self::reset(&mut self.base,alphabet,0);
        for piece in root {
            let node=&nodes[piece.id as usize];
            self.base[node.word as usize]=node.base;
            self.count[node.word as usize]+=piece.width;
        }
        let pivot=(0..alphabet).max_by_key(|&i|(self.count[i],u32::MAX-self.base[i])).unwrap();
        let (b,s)=(self.base[pivot],self.count[pivot]);
        if s==m {return None;}
        let d=pivot as u32;
        let end=b+s;
        self.count.fill(0);
        Self::reset(&mut self.epoch,alphabet,NONE);
        self.child.resize(alphabet,0);
        self.nodes.clear(); self.edges.clear(); self.selectors.clear();
        self.words.clear(); self.dictionary.clear(); self.runs.clear();
        let mut first=Self::fresh_node(0,NONE,d);
        first.edge=0;
        first.end=1;
        self.nodes.push(first);
        self.edges.push(d);
        let mut at=0;
        let mut seed=None;
        for piece in root {
            let node=&nodes[piece.id as usize];
            let k=node.word as usize;
            let next=at+piece.width;
            let offset=if k==pivot {0} else if node.base<b {node.base} else {node.base-s};
            let lf=offset+self.count[k];
            self.count[k]+=piece.width;
            if at<b {self.runs.push(TokenRun{start:at,lf,word:node.word});}
            if at<end && b<next {
                let lo=at.max(b);
                self.root_piece(node.word,next.min(end)-lo,lf+lo-at,d,words,bytes);
                if lo<=p && p<next.min(end) {seed=Some((node.word,lf+p-at));}
            }
            if next>end {
                let lo=at.max(end);
                self.runs.push(TokenRun{start:lo-s,lf:lf+lo-at,word:node.word});
            }
            at=next;
        }
        debug_assert_eq!(at,m);
        self.nodes[0].sels=self.selectors.len() as u32;
        let pages=(m-s).div_ceil(64) as usize;
        Self::reset(&mut self.page,pages,0);
        Self::reset(&mut self.masks,pages,0);
        for r in &self.runs {
            let bit=r.start&63;
            if bit!=0 {self.masks[(r.start>>6) as usize]|=1u64<<bit;}
        }
        self.runs.push(TokenRun{start:m-s,lf:0,word:0});
        let mut run=0;
        for (page,slot) in self.page.iter_mut().enumerate() {
            while self.runs[run+1].start<=page as u32*64 {run+=1;}
            *slot=run as u32;
        }
        // LF carries a source ordinal for d and a complementary position for
        // any other label. Only a mark initially in B needs its streamed seed.
        let origin=if p<b {p} else {p.saturating_sub(s)};
        let mut x=origin;
        if let Some((word,lf))=seed {
            if word==d {return Some((d,lf,skip));}
            x=lf;
            skip+=words[word as usize].width;
        }
        loop {
            let r=self.runs[self.find(x)];
            x=r.lf+x-r.start;
            if r.word==d {return Some((d,x,skip));}
            skip+=words[r.word as usize].width;
            if seed.is_none() && x==origin {return None;}
        }
    }
}
