impl Kernel {
    pub fn inspect(&mut self, records:&[u16], n:usize, p:u32, block:u64, out:&mut Vec<u8>) {
        let (_,mark,bits)=self.construct::<true,true>(records,n as u32,p,16);
        emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,out);
        let m=self.edges.len();
        let low=24-bits;
        let code=|i:usize| if self.extra.is_empty() {self.edges[i]>>bits} else {self.extra[i]};
        let mut cycle=Vec::new(); let mut row=mark;
        loop {
            cycle.push(row as usize);
            row=self.edges[row as usize]&((1<<bits)-1);
            if row==mark {break}
        }
        for size in [4usize,8,16,32,64,128] {
            let mut exception=Vec::new();
            let mut uniform=0;let mut mixed=0;let mut mixed_rows=0;
            let mut over_u8=0;
            for start in (0..m).step_by(size) {
                let end=(start+size).min(m);
                let hi=code(start)>>low;
                let mut varied=false;
                for j in start..end {
                    let top=code(j)>>low;
                    varied|=top!=hi;
                    over_u8+=usize::from(top>255);
                }
                exception.push(varied);
                if varied {mixed+=1;mixed_rows+=end-start;} else {uniform+=1;}
            }
            let mut k=0u32;
            let mut touches=0;
            for b in &self.bands {
                let width=b.end-k;
                let first=k.wrapping_add(b.delta) as usize;
                assert!(width>0 && first+width as usize<=m);
                touches+=(first+width as usize-1)/size-first/size+1;
                k=b.end;
            }
            assert_eq!(k as usize,m);
            let visits=cycle.iter().filter(|&&v|exception[v/size]).count();
            let header_bytes=if mixed<32768 {2}else{4};
            let packed=3*(m+1)+header_bytes*exception.len()+size*mixed;
            println!("{block},{n},{m},{bits},{size},{},{uniform},{mixed},{mixed_rows},{visits},{},{over_u8},{touches},{},{packed}",cycle.len(),self.bands.len(),4*(m+self.extra.len()));
        }
    }
}
