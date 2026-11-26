        let f=&mut self.forest;
        let b=self.base[d as usize];let z=b+self.count[d as usize];
        f.order.clear();
        f.order.extend((0..k as u32).filter(|&v|{
            let i=v as usize;
            v!=d && self.count[i]>1 && f.first[i]>=b && f.last[i]<=z
        }));
        f.order.sort_unstable_by_key(|&c|self.base[c as usize]);
        // Test all candidate intervals for any internal input seam using one
        // monotone old-root cursor, without indexing the other source classes.
        let mut at=0;let mut next=0;
        f.order.retain(|&label|{
            let c=label as usize;
            while at<=self.base[c] {at+=root[next].width;next+=1;}
            at<self.base[c]+self.count[c]
        });
        let root_pos=f.order.partition_point(|&v|self.base[v as usize]<b);
        f.order.insert(root_pos,d);
        Self::reset(&mut f.slot,k,NONE);
        self.nodes.clear();self.edges.clear();self.selectors.clear();self.words.clear();self.dictionary.clear();
        Self::reset(&mut self.epoch,k,NONE);self.child.resize(k,0);
        f.bfs.clear();f.bfs.push(d);
        f.bfs.extend(f.order.iter().copied().filter(|&c|c!=d));
        for (j,&w) in f.bfs.iter().enumerate(){
            let slot=j as u32;f.slot[w as usize]=slot;
            let mut n=Self::fresh_node(0,if j==0{NONE}else{0},w);
            n.edge=slot;n.end=slot+1;
            self.nodes.push(n);self.edges.push(w);
        }
        self.ready=f.bfs.len();
        self.runs.clear();
        f.head.clear();f.head.push(0);
        for &c in &f.order {f.head.push(f.head.last().copied().unwrap()+self.count[c as usize]);}
        let outer=m-*f.head.last().unwrap();
        Self::reset(&mut f.project,k,0);let mut owner=0;
        for c in 0..k{
            let from=self.base[c];
            if from<=p && p<from+self.count[c]{owner=c as u32;}
            if f.slot[c]==NONE {
                let i=f.order.partition_point(|&w|self.base[w as usize]<from);
                f.project[c]=from-f.head[i];
            }
        }
        // Save the selected ends in source order before restoring count as LF
        // occurrence cursors. Storage is proportional to the handful of sources.
        f.next.clear();
        for &c in &f.order{f.next.push(self.base[c as usize]+self.count[c as usize]);}
