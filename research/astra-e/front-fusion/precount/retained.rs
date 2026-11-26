// Replay the exact reserve requests in corpus order with the pinned standard library.
// No custom allocator is involved. Values represent the run-front vectors only.
#[derive(Default)]struct Buffers{caps:Vec<u16>,lit:Vec<u64>,one:Vec<u64>}
impl Buffers {
    fn bits(&mut self,m:usize){let words=m/64+3;self.lit.reserve(words);self.one.reserve(words);}
    fn stock(&mut self,m:usize,bound:usize){self.bits(m);self.caps.reserve(m+bound/127+3);}
    fn report(&self)->(usize,usize){(self.caps.capacity()*2,(self.lit.capacity()+self.one.capacity())*8)}
}
fn main(){
    let csv=std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let(mut stock,mut fallback,mut custom)=(Buffers::default(),Buffers::default(),Buffers::default());
    println!("stream,control_cap_bytes,control_bitmap_bytes,fallback_cap_bytes,fallback_bitmap_bytes,custom_bitmap_bytes");
    for line in csv.lines().skip(1){
        let v:Vec<_>=line.split(',').collect();let m=v[1].parse().unwrap();let bound=v[2].parse().unwrap();let n:usize=v[3].parse().unwrap();let count:usize=v[6].parse().unwrap();
        stock.stock(m,bound);
        if m>=256{custom.bits(m);}
        if m<256||count*20>=3*n{fallback.stock(m,bound);}
        let(a,b)=stock.report();let(c,d)=fallback.report();let(_,e)=custom.report();println!("{},{a},{b},{c},{d},{e}",v[0]);
    }
}
