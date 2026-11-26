impl Second{
    // The stock emitter advances terminal bases to their exclusive ends.
    // Recover the normalized marked column after emission with clocks stopped.
    pub(super) fn canonical(&self,mark:(u32,u32),weights:&[Piece])->(crate::Obj,(u32,u32)){
        let root=&self.merged[self.nodes[0].out as usize..];
        let mut mass=vec![0u32;self.nodes.len()];
        for run in root { mass[run.id as usize]+=run.width; }
        let mut dictionary=Vec::new();
        for (i,n) in self.nodes.iter().enumerate(){
            if n.word==NONE{continue;}
            let w=self.words[n.word as usize];let base=n.base-mass[i];
            dictionary.push((base,self.dictionary[w.id as usize..(w.id+w.width) as usize].to_vec()));
        }
        dictionary.sort_unstable_by_key(|(b,_)|*b);
        let mut runs=Vec::<(u32,u32)>::new();
        for r in root {
            let b=self.nodes[r.id as usize].base-mass[r.id as usize];
            if let Some((last,width))=runs.last_mut(){if *last==b{*width+=r.width;continue;}}
            runs.push((b,r.width));
        }
        let d=(0..self.count.len()).max_by_key(|&i|(self.count[i],u32::MAX-self.base[i])).unwrap() as u32;
        let shifted=self.shift(mark.0,mark.1,d,weights).unwrap();
        ((dictionary,runs),shifted)
    }
}
