            let mut done=0;
            if live == ALL {
                if SCHEDULE==1 {
                    for lane in 0..LANES {
                        let r=self.runs[index[lane]];
                        if r.retires(lo[lane]+width[lane]) {
                            done |= 1<<lane;
                        } else {
                            paths[lane].push(r.byte());
                            lo[lane]=r.map(lo[lane]);
                        }
                    }
                } else {
                    let replies:[Run;LANES]=std::array::from_fn(|lane| self.runs[index[lane]]);
                    if SCHEDULE==2 {
                        for lane in 0..LANES {
                            let r=replies[lane];
                            if r.retires(lo[lane]+width[lane]) {
                                done |= 1<<lane;
                            } else {
                                paths[lane].push(r.byte());
                                lo[lane]=r.map(lo[lane]);
                            }
                        }
                    } else {
                        for lane in 0..LANES {
                            done |= u32::from(replies[lane].retires(lo[lane]+width[lane]))<<lane;
                        }
                        if done==0 {
                            for lane in 0..LANES {
                                let r=replies[lane];
                                paths[lane].push(r.byte());
                                lo[lane]=r.map(lo[lane]);
                            }
                        } else {
                            let mut advancing=ALL^done;
                            while advancing!=0 {
                                let lane=advancing.trailing_zeros() as usize;
                                advancing&=advancing-1;
                                let r=replies[lane];
                                paths[lane].push(r.byte());
                                lo[lane]=r.map(lo[lane]);
                            }
                        }
                    }
                }
            } else {
                let mut mask=live;
                while mask!=0 {
                    let lane=mask.trailing_zeros() as usize;
                    mask&=mask-1;
                    let r=self.runs[index[lane]];
                    if r.retires(lo[lane]+width[lane]) {
                        done |= 1<<lane;
                    } else {
                        paths[lane].push(r.byte());
                        lo[lane]=r.map(lo[lane]);
                    }
                }
            }
            #[cfg(slot_counts)]
            {
                self.wave_counts[live.count_ones() as usize][done.count_ones() as usize]+=1;
            }
