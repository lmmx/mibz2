    pub fn first(&mut self,caps:&[u16],freq:&[u32;256],n:u32,p:u32,u:(u32,u32),h:usize)
        ->Option<(Option<(u32,u32)>,u32,u32)>{
        let t0=crate::now();
        self.plan=Plan::prepare(caps,freq,u,h)?;
        self.measures=[0;8];
        self.projected_index(caps,freq,n);
        let m=u.1-u.0;
        let (mark,missing)=self.mark_shifted(p);
        let t1=crate::now();
        let lo=self.plan.root.0;let mass=self.plan.root.1-lo;
        self.symbol=0;
        let mono=self.plan.mono(freq);
        if mono{self.build::<true>(lo,mass);}else{self.build::<false>(lo,mass);}
        let t2=crate::now();
        if mono{self.inverse_merge::<true>();}else{
            self.inverse_merge::<false>();
            self.splice_early(m);
            let root=self.nodes[0].out;
            self.nodes.clear();
            self.nodes.extend(self.bases.iter().enumerate().map(|(i,&base)|{
                let mut node=Node::new(0,NONE,0);node.base=base;node.word=i as u32;node
            }));
            self.nodes[0].out=root;
        }
        self.phases=[t1-t0,t2-t1,crate::now()-t2];
        self.measures[1]=self.runs.len() as u32;self.measures[2]=self.plan.target().parts.len() as u32;
        self.measures[3]=self.plan.forbid.parts.len() as u32;self.measures[4]=self.plan.forbid.mass;
        self.measures[5]=m-mass;self.measures[6]=self.plan.early.len() as u32;self.measures[7]=u32::from(mono);
        Some((mark,m,missing))
    }
