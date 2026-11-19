
impl Kernel {
    pub fn record(&mut self, r:&Runs, n:usize, p:u32, index:u32, stream:u32,
                  out:&mut Vec<u8>, dump:&mut impl std::io::Write) {
        let (c,mark,bits)=self.construct::<true,true>(r.records(),n as u32,p,16);
        emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,out);
        let mut times=vec![0u32;self.handles.len()];
        let mut row=mark;
        let mask=(1<<bits)-1;
        loop {
            let e=self.edges[row as usize];
            let code=if self.extra.is_empty(){e>>bits}else{self.extra[row as usize]};
            if code>=256 {times[(code-256) as usize]+=1;}
            row=e&mask;
            if row==mark {break;}
        }
        let mut u32s=|words:&[u32]| {
            for v in words { dump.write_all(&v.to_le_bytes()).unwrap(); }
        };
        u32s(&[index,stream,n as u32,c.retained as u32,c.table_bytes as u32,self.handles.len() as u32]);
        assert_eq!(self.handles.len(),c.words);
        for (i,(&(h,width),q)) in self.handles.iter().zip(times).enumerate() {
            assert_eq!(width,q);
            let s=&self.spelling[self.offsets[i] as usize..self.offsets[i+1] as usize];
            for v in [h,q,s.len() as u32] {dump.write_all(&v.to_le_bytes()).unwrap();}
            dump.write_all(s).unwrap();
        }
    }
}
