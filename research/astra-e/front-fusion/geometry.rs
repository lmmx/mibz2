
#[derive(Clone)]
struct Parts {
    all:[u32;256], thin:[u32;256], thick:[u32;256],
    m:u32, mark:u32, pos:u32, caps:u32, primary:u32, last:u16,
}
impl Parts {
    fn start(primary:u32)->Self{Self{all:[0;256],thin:[0;256],thick:[0;256],m:0,mark:0,pos:0,caps:0,primary,last:256}}
    #[inline(always)]
    fn piece(&mut self,k:&mut Kernel,c:u8,x:u32,w:u32,keep:bool) {
        let a=c as usize;
        if keep {
            if x<=self.primary && self.primary-x<w {self.mark=self.m+self.primary-x;}
            k.queues[a].extend(self.m..self.m+w);self.thin[a]+=w;self.m+=w;
        } else {
            k.strips.push(Strip{from:x,to:self.all[a],width:w,byte:c});self.thick[a]+=1;
        }
        self.all[a]+=w;
    }
    #[inline(always)]
    fn run(&mut self,k:&mut Kernel,c:u8,w:u32) {
        let x=self.pos;let end=x+w;let p=self.primary;
        if w>16 && x<=p && p<end {
            if x<p {self.piece(k,c,x,p-x,false);}
            self.piece(k,c,p,1,true);
            if p+1<end {self.piece(k,c,p+1,end-p-1,false);}
        } else {self.piece(k,c,x,w,w<=16);}
        self.pos=end;
    }
}
impl Kernel {
    pub(super) fn fuse<const TILE:usize>(&mut self,b:&mibz2::block::Symbols,f:&mut crate::front::Bits,out:&mut Vec<u8>)->Option<(usize,u32)>{
        let mut p=Parts::start(b.orig_ptr);self.strips.clear();self.queues.resize_with(256,Vec::new);for q in &mut self.queues {q.clear();}
        let mut tile=[0u32;TILE];let mut used=0;let mut seen=0u32;
        let n=unsafe{crate::front::front_avx2(b,f,|c,w|{
            if c as u16==p.last || w>b.max_len as u32-seen{return false;}
            seen+=w;p.last=c as u16;
            p.caps+=1+(w-1)/127;
            if TILE==0 {p.run(self,c,w);}else{
                tile[used]=(w<<8)|u32::from(c);used+=1;
                if used==TILE{for &word in &tile{p.run(self,word as u8,word>>8);}used=0;}
            }
            true
        })}?;
        if n==0||b.orig_ptr as usize>=n||p.caps as usize*20>=3*n{return None;}
        for &word in &tile[..used]{p.run(self,word as u8,word>>8);}
        let cap_count=p.caps;let (_,mark,bits)=self.complete(p);
        emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,out);
        Some((n,cap_count))
    }
}
