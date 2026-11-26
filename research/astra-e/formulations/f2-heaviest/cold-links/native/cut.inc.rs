                let small = self.nodes[next];
                if small.hi - small.lo <= self.threshold {
                    self.nodes[next].edge = self.edges.len() as u32;
                    self.edges.push(small.first);
                    self.nodes[next].end = self.edges.len() as u32;
                    self.nodes[next].sel = self.selectors.len() as u32;
                    self.nodes[next].sels = 1;
                    self.selectors.push(Piece { id: NONE, width: small.hi - small.lo });
                    next += 1;
                    continue;
                }
