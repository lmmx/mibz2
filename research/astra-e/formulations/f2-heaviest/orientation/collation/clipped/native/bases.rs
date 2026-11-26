        if !ONLY{
            let root=self.nodes[0].out;
            self.nodes.clear();
            self.nodes.extend(self.bases.iter().enumerate().map(|(i,&base)|{
                let mut node=Node::new(0,NONE,0);node.base=base;node.word=i as u32;node
            }));
            self.nodes[0].out=root;
        }
