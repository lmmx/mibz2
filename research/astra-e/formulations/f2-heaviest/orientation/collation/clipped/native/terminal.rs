                    let id = if r.terminal() {
                        let fresh=if ONLY{self.nodes[i].word==NONE}else{terminal_epoch[c]!=i as u32};
                        if fresh{
                            #[cfg(clipped_counts)] {self.mixed+=u32::from(epoch[c]==i as u32);}
                            let word=self.words.len() as u32;
                            if ONLY{
                                self.nodes[i].base=mapped-quote;
                                self.nodes[i].word=word;
                            }else{
                                self.bases.push(mapped-quote);
                                terminal_word[c]=word;
                            }
                            terminal_epoch[c]=i as u32;
                            let pos=self.dict.len() as u32;
                            if !ONLY{self.dict.push(r.byte());}
                            let mut j=i as u32;
                            while j!=NONE{
                                let n=self.nodes[j as usize];
                                self.dict.extend(self.edges[n.edge as usize..n.end as usize].iter().rev());
                                j=n.parent;
                            }
                            self.words.push(Piece{id:pos,width:self.dict.len() as u32-pos});
                        }
                        if ONLY{NONE}else{!terminal_word[c]}
                    }else{
