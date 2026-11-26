fn emit_column(caps:&[u16],order:&Overlay,out:&mut Vec<u16>){
    out.clear();let mut source=0u32;let mut pos=0u32;let mut j=0;let mut h=0;let mut e=0;
    while pos<=order.n{
        while e<order.nodes.len()&&order.nodes[e].anchor==pos{
            out.push((u16::from(order.nodes[e].symbol)<<8)|1);e+=1;
        }
        if pos==order.n{break;}
        let mut end=source+u32::from(caps[j]&255);
        if h<order.holes.len()&&order.holes[h]==pos{pos+=1;h+=1;}
        else {
            if h<order.holes.len(){end=end.min(order.holes[h]);}
            if e<order.nodes.len(){end=end.min(order.nodes[e].anchor);}
            assert!(end>pos);out.push((caps[j]&0xff00)|(end-pos)as u16);pos=end;
        }
        if pos==source+u32::from(caps[j]&255){j+=1;source=pos;}
    }
    assert_eq!((e,h,j),(order.nodes.len(),order.holes.len(),caps.len()));
}
impl Kernel {
    fn first_lap(&self,mark:u32,bits:u32,out:&mut Vec<u8>){
        let mask=(1<<bits)-1;let mut row=mark;let mut edge=self.edges[row as usize];
        loop {
            let code=if self.extra.is_empty(){edge>>bits}else{self.extra[row as usize]};
            row=edge&mask;edge=self.edges[row as usize];append_code(code,&self.spelling,&self.offsets,out);
            if row==mark{break;}
        }
    }
    pub fn expanded(&mut self,caps:&[u16],n:usize,p:u32,out:&mut Vec<u8>,k:super::Kernels)->Result<(),mibz2::ErrorKind>{
        self.reader.build(caps);
        self.outcome=(1,0,0,0,0);
        if let Some(events)=self.reader.windows(p,1024){
            let fills:usize=events.iter().map(|e|usize::from(e.2)).sum();
            let size=n+fills-events.len();
            self.outcome=(2,events.len(),0,0,0);
            if fills<=4096&&size<=900_000{
                if let Some((order,_,calls))=batch(&self.reader,&events,8192){
                    let mark=order.position(p);
                    self.outcome=(3,events.len(),calls,order.nodes.len(),order.holes.len());
                    let mut corrected=std::mem::take(&mut self.corrected_caps);
                    emit_column(caps,&order,&mut corrected);
                    drop(order);
                    let (_,root,bits)=self.construct::<true,true>(&corrected,size as u32,mark,16);
                    self.corrected_caps=corrected;
                    let start=out.len();out.reserve(size);self.first_lap(root,bits,out);
                    if out.len()-start==size{self.outcome.0=0;return Ok(());}
                    out.truncate(start);
                }
            }
        }
        let mut raw=std::mem::take(&mut self.original);
        self.run::<true,true,false>(caps,n,p,16,&mut raw);
        let result=mibz2::rle::unrle(&raw,out,k);self.original=raw;result
    }
    pub fn observations(&self)->(u8,usize,usize,usize,usize){self.outcome}
}
