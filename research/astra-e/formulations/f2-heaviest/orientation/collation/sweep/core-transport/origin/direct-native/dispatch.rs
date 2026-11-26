impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE>{
    pub fn frequent_section(&mut self,caps:&[u16],freq:&[u32;256],n:u32,u:(u32,u32),levels:usize,
                            heap:&mut std::collections::BinaryHeap<(u32,std::cmp::Reverse<u32>)>)
                            ->Option<(u32,u32)>{
        self.section=u;self.symbol=34;
        let (lo,m)=self.prepare_counted::<true>(caps,n,freq)?;
        let mut u=(lo,lo+m);
        for _ in 0..levels {u=self.heavy_child(u,heap);}
        Some(u)
    }
    fn heavy_child(&self,u:(u32,u32),heap:&mut std::collections::BinaryHeap<(u32,std::cmp::Reverse<u32>)>)->(u32,u32){
        use std::cmp::Reverse;
        let (lo,m)=(u.0,u.1-u.0);
        let mut widths=[0u32;256];let mut bases=[0u32;256];
        let mut touched=[0usize;256];
        heap.clear();heap.push((m,Reverse(lo)));
        let mut best=(0u32,Reverse(u32::MAX));
        while let Some((w,Reverse(mut x)))=heap.pop(){
            if w<best.0{break;}
            let end=x+w;let mut k=self.locate(x);let mut nt=0;
            while x<end{
                let run=self.runs[k];let c=run.byte() as usize;
                let y=end.min(run.position());
                if widths[c]==0{bases[c]=run.map(x);touched[nt]=c;nt+=1;}
                widths[c]+=y-x;x=y;k+=1;
            }
            for &c in &touched[..nt]{
                let a=bases[c];let w=std::mem::take(&mut widths[c]);
                if w<best.0{continue;}
                if (u.0..u.1).contains(&a){
                    debug_assert!(a+w<=u.1);best=best.max((w,Reverse(a)));
                }else{
                    heap.push((w,Reverse(a)));
                }
            }
        }
        debug_assert!(best.0>0);
        (best.1.0,best.1.0+best.0)
    }
    pub fn raw_cycle(&mut self,caps:&[u16],freq:&[u32;256],n:u32,p:u32,out:&mut Vec<u8>){
        let c=freq.iter().position(|&w|w>0).unwrap();
        self.symbol=c as u8;self.section=(0,freq[c]);
        self.prepare_counted::<true>(caps,n,freq).unwrap();
        self.finish(None,0,n,p,out);
    }
}
