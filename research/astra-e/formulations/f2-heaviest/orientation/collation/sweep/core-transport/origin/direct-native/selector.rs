// Fixed byte cylinders. The final quote is the landing byte in every pattern.
const COMMA:&[u8]=br#",""#;
const FREQUENT:&[u8]=&[34,58,34];
const PROPERTY:&[u8]=b"\",\"property\":\"";
const CHARS:&[u8;9]=b"\",:proety";
const STRIDE:usize=128;
#[derive(Clone,Copy)]
struct Page {row:u32,rank:[u32;9]}
#[derive(Default)]
struct LiteralSelector {pages:Vec<Page>}
impl LiteralSelector {
    fn base(freq:&[u32;256],c:u8)->u32 {freq[..c as usize].iter().sum()}
    fn short(caps:&[u16],freq:&[u32;256],pattern:&[u8])->(u32,u32) {
        let (&last,prefix)=pattern.split_last().unwrap();
        let mut lo=Self::base(freq,last);
        let mut hi=lo+freq[last as usize];
        for &c in prefix.iter().rev(){
            if lo==hi{break;}
            let mut pos=0;
            let mut lower=Self::base(freq,c);let mut upper=lower;
            for &r in caps{
                let width=u32::from(r&255);
                if (r>>8) as u8==c {
                    lower+=lo.saturating_sub(pos).min(width);
                    upper+=hi.saturating_sub(pos).min(width);
                }
                pos+=width;if pos>=hi{break;}
            }
            (lo,hi)=(lower,upper);
        }
        (lo,hi)
    }
    fn sampled(&mut self,caps:&[u16],freq:&[u32;256],pattern:&[u8])->(u32,u32){
        let mut next=*freq;
        let mut total=0;for x in &mut next{let start=total;total+=*x;*x=start;}
        let first=next;
        self.pages.clear();
        let mut row=0;
        for chunk in caps.chunks(STRIDE){
            self.pages.push(Page{row,rank:CHARS.map(|c|next[c as usize])});
            for &r in chunk{
                let w=u32::from(r&255);
                next[(r>>8) as usize]+=w;
                row+=w;
            }
        }
        let (&last,prefix)=pattern.split_last().unwrap();
        let mut lo=first[last as usize];let mut hi=lo+freq[last as usize];
        for &c in prefix.iter().rev(){
            if lo==hi {break;}
            let slot=CHARS.iter().position(|&x|x==c).unwrap();
            lo=self.endpoint(caps,lo,c,slot);
            hi=self.endpoint(caps,hi,c,slot);
        }
        (lo,hi)
    }
    fn endpoint(&self,caps:&[u16],x:u32,c:u8,slot:usize)->u32 {
        let i=self.pages.partition_point(|p|p.row<=x)-1;
        let mut pos=self.pages[i].row;let mut rank=self.pages[i].rank[slot];
        for &r in caps[i*STRIDE..].iter().take(STRIDE) {
            if pos>=x{break;}
            let w=u32::from(r&255);
            if (r>>8) as u8==c{rank+=(x-pos).min(w);}
            pos+=w;
        }
        rank
    }
}
