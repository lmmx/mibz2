            while pos<end {
                let next=self.forest.order.get(i).copied();
                let b=next.map_or(m,|c|self.base[c as usize]);
                if pos<b || next.is_none() {
                    let stop=if next.is_none(){end}else{end.min(b)};
                    let start=pos-self.forest.head[i];let mapped=lf+pos-at;
                    if !self.runs.last().is_some_and(|r|r.word==n.word && r.lf+start-r.start==mapped) {
                        self.runs.push(TokenRun{word:n.word,lf:mapped,start});
                    }
                    pos=stop;
                } else {
                    let slot=self.forest.slot[next.unwrap() as usize] as usize;let hi=self.forest.next[i];
                    if pos==b {self.nodes[slot].sel=self.selectors.len() as u32;}
                    let begin=self.nodes[slot].sel as usize;let stop=end.min(hi);
                    self.forest_piece(slot,n.word,stop-pos,lf+pos-at,d,words,buf,begin);
                    if stop==hi{self.nodes[slot].sels=self.selectors.len() as u32-begin as u32;i+=1;}
                    pos=stop;
                }
            }
