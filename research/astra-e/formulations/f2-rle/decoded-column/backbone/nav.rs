#[derive(Clone, Copy)]
struct Span { start:u32, rank:u32, len:u32 }
#[derive(Default)]
struct Nav {
    letter:Vec<(u32,u8)>,
    per:Vec<Vec<Span>>,
    base:Vec<u32>,
    n:u32,
}
impl Nav {
    fn build(&mut self,caps:&[u16]) {
        self.letter.clear();self.per.resize_with(256,Vec::new);for v in &mut self.per{v.clear();}
        let mut counts=[0;256];let mut pos=0;
        let mut j=0;
        while j<caps.len(){
            let c=(caps[j]>>8) as u8;let mut len=0;
            while j<caps.len()&&(caps[j]>>8)as u8==c {len+=u32::from(caps[j]&255);j+=1;}
            let b=c as usize;self.letter.push((pos,c));self.per[b].push(Span{start:pos,rank:counts[b],len});counts[b]+=len;pos+=len;
        }
        self.n=pos;self.base.clear();let mut t=0;for v in counts{self.base.push(t);t+=v;}self.base.push(t);
    }
    fn l(&self,x:u32)->u8{self.letter[self.letter.partition_point(|s|s.0<=x)-1].1}
    fn f(&self,x:u32)->u8{(self.base.partition_point(|&v|v<=x)-1)as u8}
    fn rank(&self,c:u8,x:u32)->u32{
        let a=&self.per[c as usize];let i=a.partition_point(|s|s.start<x);
        if i==0 {0}else{let s=a[i-1];s.rank+(x-s.start).min(s.len)}
    }
    fn lf(&self,x:u32)->u32{let c=self.l(x);self.base[c as usize]+self.rank(c,x)}
    fn psi(&self,x:u32)->u32{let c=self.f(x)as usize;let k=x-self.base[c];let a=&self.per[c];let s=a[a.partition_point(|s|s.rank<=k)-1];s.start+k-s.rank}
    fn windows(&self,p:u32,limit:usize)->Option<Vec<(u32,u8,u8)>>{
        let mut events=Vec::new();let mut dirty=Vec::new();
        for c in 0..256{
            let (mut lo,mut hi)=(self.base[c],self.base[c+1]);
            for _ in 1..4 {if lo==hi {break;}lo=self.base[c]+self.rank(c as u8,lo);hi=self.base[c]+self.rank(c as u8,hi);}
            if events.len()+(hi-lo)as usize>limit {return None;}
            for start in lo..hi {let mut x=start;for _ in 0..4{dirty.push(x);x=self.psi(x);}dirty.push(x);events.push((x,c as u8,self.f(x)));}
        }
        dirty.sort_unstable();if dirty.binary_search(&p).is_ok()||dirty.windows(2).any(|a|a[0]==a[1]){return None;}
        Some(events)
    }
    fn bytes(&self)->usize{
        self.letter.capacity()*std::mem::size_of::<(u32,u8)>()+self.per.capacity()*std::mem::size_of::<Vec<Span>>()+
        self.per.iter().map(|v|v.capacity()*std::mem::size_of::<Span>()).sum::<usize>()+4*self.base.capacity()
    }
}
