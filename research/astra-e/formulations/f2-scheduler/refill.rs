        const LANES: usize = 8;
        const ALL: u32 = (1 << LANES)-1;
        let mut paths: [Vec<u8>; LANES] = std::mem::take(&mut self.lane_paths);
        let mut job=[0usize;LANES];
        let mut lo=[0u32;LANES];
        let mut width=[0u32;LANES];
        let mut index=[0usize;LANES];
        let mut live=0;
        // Every allocated child has a fully published parent. Allocation order
        // remains a topological ordering even when earlier jobs retire later.
        let mut next=0;
        while live!=0 || next<self.nodes.len() {
            let mut empty=ALL^live;
            while empty!=0 && next<self.nodes.len() {
                let lane=empty.trailing_zeros() as usize;
                empty&=empty-1;
                live |= 1<<lane;
                job[lane]=next;
                let n=self.nodes[next];
                next+=1;
                lo[lane]=n.lo;
                width[lane]=n.hi-n.lo;
                paths[lane].clear();
                paths[lane].push(n.first);
            }
            if live==ALL {
                for lane in 0..LANES { index[lane]=self.locate(lo[lane]); }
            } else {
                let mut mask=live;
                while mask!=0 {
                    let lane=mask.trailing_zeros() as usize;
                    mask&=mask-1;
                    index[lane]=self.locate(lo[lane]);
                }
            }
            let mut done=0;
            let mut mask=live;
            while mask!=0 {
                let lane=mask.trailing_zeros() as usize;
                mask&=mask-1;
                let k=index[lane];
                let r=self.runs[k];
                if self.runs[k+1].start < lo[lane]+width[lane] || r.byte == QUOTE {
                    done |= 1<<lane;
                } else {
                    paths[lane].push(r.byte);
                    lo[lane]=r.lf+(lo[lane]-r.start);
                }
            }
            live ^= done;
            while done!=0 {
                let lane=done.trailing_zeros() as usize;
                done&=done-1;
                let i=job[lane];
                let lo=lo[lane];
                let hi=lo+width[lane];
                let mut k=index[lane];
                self.nodes[i].edge=self.edges.len() as u32;
                self.edges.extend_from_slice(&paths[lane]);
