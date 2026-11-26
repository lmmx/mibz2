
impl Kernel {
    pub fn tape(&mut self,r:&Runs,n:usize,p:u32,index:u32,stream:u32,out:&mut Vec<u8>,file:&mut impl std::io::Write) {
        let (c,mark,bits)=self.construct::<true,true>(r.records(),n as u32,p,16);
        emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,out);
        assert!(self.extra.is_empty());
        let mut write=|v:u32|file.write_all(&v.to_le_bytes()).unwrap();
        for v in [index,stream,n as u32,c.retained as u32,mark,bits,c.words as u32,self.bands.len() as u32] {write(v);}
        for &e in &self.edges {write(e);}
        let mut k=0u32;
        let mask=(1<<bits)-1;
        let mut last_byte=0;
        for band in &self.bands {
            let start=k.wrapping_add(band.delta);
            let len=band.end-k;
            let a=if band.code<256 {band.code} else {self.spelling[self.offsets[band.code as usize-255] as usize-1] as u32};
            assert!(a>=last_byte);
            let mut previous=None;
            for &e in &self.edges[start as usize..(start+len) as usize] {
                assert_eq!(e>>bits,band.code);
                assert!(previous.map_or(true,|q| q<(e&mask)));
                previous=Some(e&mask);
            }
            for v in [start,len,band.code,a] {write(v);}
            last_byte=a;k=band.end;
        }
        assert_eq!(k,c.retained as u32);
    }
}
