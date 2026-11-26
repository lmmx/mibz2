    fn build(&mut self, quote: u32, tokens:u32) {
        self.nodes.clear();
        self.selectors.clear();
        self.words.clear();
        self.dict.clear();
        self.labels.clear();
        // Index zero terminates a spelling chain. Labels encode one edge byte
        // and the index of the suffix; there are at most n distinct intervals.
        self.labels.push(0);
        self.labels.push(u32::from(QUOTE));
        let mut root=Node::new(quote,NONE,QUOTE);
        root.hi+=tokens;
        self.nodes.push(root);
        let mut current=std::mem::take(&mut self.current);
        let mut next=std::mem::take(&mut self.next);
        current.resize_with(256,Vec::new);
        next.resize_with(256,Vec::new);
        let mut mask=[0u64;4];
        mask[(QUOTE/64) as usize]|=1<<(QUOTE%64);
        current[QUOTE as usize].push(Job{lo:quote,hi:quote+tokens,id:0,label:1});
        let mut epoch=[NONE;256];
        let mut active=[0usize;256];
        while mask!=[0;4] {
            let mut new_mask=[0u64;4];
            let mut cursor=0;
            for group in 0..4 {
                let mut used=mask[group];
                while used!=0 {
                    let ch=group*64+used.trailing_zeros() as usize;
                    used&=used-1;
                    for job in &current[ch] {
                        let (lo,hi)=(job.lo,job.hi);
                        // Across a depth the query order is increasing. Keep
                        // the previous original run whenever it covers lo.
                        if self.runs[cursor+1].start<=lo {
                            cursor=self.locate(lo);
                        }
                        let mut k=cursor;
                        let r=self.runs[k];
                        if self.runs[k+1].start>=hi && r.byte!=QUOTE {
                            let c=r.byte as usize;
                            let mapped=r.lf+(lo-r.start);
                            let label=self.labels.len() as u32;
                            self.labels.push(job.label<<8|u32::from(r.byte));
                            next[c].push(Job{lo:mapped,hi:mapped+(hi-lo),id:job.id,label});
                            new_mask[c/64]|=1u64<<(c%64);
                            continue
                        }
                        let i=job.id as usize;
                        self.nodes[i].sel=self.selectors.len() as u32;
                        let begin=self.selectors.len();
                        while self.runs[k].start<hi {
                            let r=self.runs[k];
                            let a=lo.max(r.start);
                            let width=hi.min(self.runs[k+1].start)-a;
                            let mapped=r.lf+(a-r.start);
                            let c=r.byte as usize;
                            let id=if r.byte==QUOTE {
                                if self.nodes[i].word==NONE {
                                    self.nodes[i].base=mapped-quote;
                                    self.nodes[i].word=self.words.len() as u32;
                                    let off=self.dict.len() as u32;
                                    let mut idx=job.label;
                                    while idx!=0 {
                                        let v=self.labels[idx as usize];
                                        self.dict.push(v as u8);
                                        idx=v>>8;
                                    }
                                    self.words.push(Piece{id:off,width:self.dict.len() as u32-off});
                                }
                                NONE
                            } else {
                                if epoch[c]!=job.id {
                                    epoch[c]=job.id;
                                    active[c]=self.nodes.len();
                                    self.nodes.push(Node::new(mapped,job.id,r.byte));
                                    let label=self.labels.len() as u32;
                                    self.labels.push(job.label<<8|u32::from(r.byte));
                                    next[c].push(Job{lo:mapped,hi:mapped,id:active[c] as u32,label});
                                    new_mask[c/64]|=1u64<<(c%64);
                                }
                                let child=next[c].last_mut().unwrap();
                                debug_assert_eq!(child.hi,mapped);
                                child.hi+=width;
                                self.nodes[active[c]].hi+=width;
                                child.id
                            };
                            if self.selectors.len()>begin && self.selectors.last().unwrap().id==id {
                                self.selectors.last_mut().unwrap().width+=width;
                            } else {
                                self.selectors.push(Piece{id,width});
                            }
                            k+=1;
                        }
                        self.nodes[i].sels=self.selectors.len() as u32-self.nodes[i].sel;
                    }
                    current[ch].clear();
                }
            }
            mask=new_mask;
            std::mem::swap(&mut current,&mut next);
        }
        self.current=current;
        self.next=next;
    }
