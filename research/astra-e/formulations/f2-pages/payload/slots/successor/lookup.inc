            if CACHE>=2 {
                let mut need=live;
                let mut have=live;
                while have!=0 {
                    let lane=have.trailing_zeros() as usize;
                    have &= have-1;
                    let code=prediction[lane];
                    if code==0 {continue}
                    let x=lo[lane];
                    let mut j=code as usize-1;
                    if CACHE==3 && x>=upper_range {j+=65535;}
                    if let Some(r)=self.runs.get(j) {
                        if x<r.position::<CACHE>() && (j==0 || x>=self.runs[j-1].position::<CACHE>()) {
                            index[lane]=j;
                            need &= !(1<<lane);
                        }
                    }
                }
                #[cfg(prediction_counts)]
                {
                    self.counters[0]+=u64::from(live.count_ones());
                    self.counters[1]+=u64::from((live^need).count_ones());
                }
                if need==ALL {
                    for lane in 0..LANES {index[lane]=self.locate(lo[lane]);}
                } else {
                    let mut rest=need;
                    while rest!=0 {
                        let lane=rest.trailing_zeros() as usize;
                        rest&=rest-1;
                        index[lane]=self.locate(lo[lane]);
                    }
                }
                while need!=0 {
                    let lane=need.trailing_zeros() as usize;
                    need &= need-1;
                    let source=from[lane];
                    if source==usize::MAX {continue}
                    let j=index[lane];
                    if CACHE==2 {
                        let code=j as u32+1;
                        if self.next_target[source]!=code {
                            self.next_target[source]=code;
                            #[cfg(prediction_counts)] {self.counters[2]+=1;}
                        }
                    } else {
                        let code=(j%65535) as u32+1;
                        if self.runs[source].prediction()!=code {
                            self.runs[source].set_prediction(code);
                            #[cfg(prediction_counts)] {self.counters[2]+=1;}
                        }
                    }
                }
                let mut have=live;
                while have!=0 {
                    let lane=have.trailing_zeros() as usize;
                    have &= have-1;
                    let j=index[lane];
                    from[lane]=j;
                    prediction[lane]=if CACHE==2 {self.next_target[j]} else {self.runs[j].prediction()};
                }
            } else
