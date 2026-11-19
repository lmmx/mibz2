impl Second{
    pub(super) fn support_index(&mut self,root:&[Piece],nodes:&[Node],alphabet:usize,m:u32)->u32{
        self.column(root,nodes,alphabet,m)
    }
    pub(super) fn support(&self,d:u32,c:&[super::FoldType])->(Vec<Vec<u32>>,[usize;5]){
        let mut ranges=vec![(self.base[d as usize],self.base[d as usize]+self.count[d as usize])];
        let mut stamp=vec![usize::MAX;c.len()];let mut child=vec![0;c.len()];
        let mut support=Vec::new();let mut seen=Vec::new();let mut stats=[0usize;5];let mut i=0;
        while i<ranges.len(){
            let (a,b)=ranges[i];let mut run=self.find(a);let mut qualifies=true;
            seen.clear();stats[4]+=(b-a) as usize;
            while self.runs[run].start<b{
                let r=self.runs[run];let k=r.word as usize;let lo=a.max(r.start);let hi=b.min(self.runs[run+1].start);
                let x=r.lf+lo-r.start;let w=hi-lo;stats[1]+=1;
                if stamp[k]!=i{
                    stamp[k]=i;seen.push(c[k].base);qualifies&=c[k].succ!=NONE;
                    if r.word!=d{child[k]=ranges.len();ranges.push((x,x+w));}
                }else if r.word!=d{
                    let range=&mut ranges[child[k]];assert_eq!(range.1,x);range.1+=w;
                }
                run+=1;
            }
            stats[2]+=seen.len();
            if qualifies{seen.sort_unstable();stats[3]+=seen.len();support.push(seen.clone());}
            i+=1;
        }
        stats[0]=ranges.len();
        assert!(stats[2]<=stats[1] && stats[1]<=self.runs.len()-1+ranges.len()-1);
        (support,stats)
    }
}
