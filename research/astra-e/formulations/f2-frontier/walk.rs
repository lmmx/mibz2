        // Nodes within the current frontier depend only on earlier parents.
        // Do all page-rank lookups in a wave before consuming their run bounds.
        const LANES: usize = 8;
        let mut paths: [Vec<u8>; LANES] = std::mem::take(&mut self.lane_paths);
        while i < self.nodes.len() {
            let batch=(self.nodes.len()-i).min(LANES);
            let mut lo=[0;LANES];
            let mut width=[0;LANES];
            let mut index=[0usize;LANES];
            let mut live=(1u32<<batch)-1;
            for lane in 0..batch {
                let node=self.nodes[i+lane];
                lo[lane]=node.lo;
                width[lane]=node.hi-node.lo;
                paths[lane].clear();
                paths[lane].push(node.first);
            }
            while live!=0 {
                let mut mask=live;
                while mask!=0 {
                    let lane=mask.trailing_zeros() as usize;
                    mask&=mask-1;
                    index[lane]=self.locate(lo[lane]);
                }
                let mut mask=live;
                while mask!=0 {
                    let lane=mask.trailing_zeros() as usize;
                    mask&=mask-1;
                    let k=index[lane];
                    let r=self.runs[k];
                    if self.runs[k+1].start < lo[lane]+width[lane] || r.byte==QUOTE {
                        live &= !(1<<lane);
                    } else {
                        paths[lane].push(r.byte);
                        lo[lane]=r.lf+(lo[lane]-r.start);
                    }
                }
            }
            for lane in 0..batch {
                let lo=lo[lane];
                let hi=lo+width[lane];
                let mut k=index[lane];
                self.nodes[i].edge=self.edges.len() as u32;
                self.edges.extend_from_slice(&paths[lane]);
