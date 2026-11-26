// Full lookup reconstruction of an already captured edge geometry, without timing.
use std::{env, fs};

fn word(tape: &mut &[u8]) -> u32 {
    let (v,rest)=tape.split_at(4);
    *tape=rest;
    u32::from_le_bytes(v.try_into().unwrap())
}

#[derive(Clone,Copy)]
struct Band {start:usize, length:usize, code:u32, byte:u32}
enum Directory { Small(Vec<u16>), Large(Vec<u32>) }
impl Directory {
    fn base(&self,i:usize)->u32 {match self {Self::Small(v)=>v[i].into(), Self::Large(v)=>v[i]}}
    fn bytes(&self)->usize {match self {Self::Small(v)=>v.len()*2, Self::Large(v)=>v.len()*4}}
}
struct Geometry {
    mask:Vec<u64>, prefix:Directory, desc:Vec<u32>, q:Vec<u32>,
    bits:u32
}
impl Geometry {
    fn new(edges:&[u32], bands:&[Band], bits:u32)->Self {
        let m=edges.len();
        let mut mask=vec![0u64;m.div_ceil(64)];
        for b in bands {
            assert!(b.length>0 && b.start+b.length<=m);
            let one=1u64<<(b.start%64);
            assert_eq!(mask[b.start/64]&one,0);
            mask[b.start/64]|=one;
        }
        let mut n=0;
        let counts=mask.iter().map(|v| {let old=n; n+=v.count_ones(); old});
        let prefix=if bands.len()<=usize::from(u16::MAX) {
            Directory::Small(counts.map(|x|x as u16).collect())
        } else {Directory::Large(counts.collect())};
        assert_eq!(n as usize,bands.len());
        let mut g=Self{mask,prefix,desc:vec![0;bands.len()],q:Vec::with_capacity(m),bits};
        let low=(1u32<<bits)-1;
        let mut previous=None;
        for b in bands {
            let slot=g.band(b.start);
            let delta=(g.q.len() as u32).wrapping_sub(b.start as u32)&low;
            assert!(b.code<=u32::MAX>>bits);
            g.desc[slot]=(b.code<<bits)|delta;
            for &e in &edges[b.start..b.start+b.length] {
                assert_eq!(e>>bits,b.code);
                let rank=e&low;
                if let Some((label,last))=previous {
                    assert!(b.byte>label || (b.byte==label && rank>last));
                }
                g.q.push(rank);
                previous=Some((b.byte,rank));
            }
        }
        assert_eq!(g.q.len(),m);
        g
    }
    fn band(&self,row:usize)->usize {
        let page=row/64;
        let through=u64::MAX>>(63-(row%64));
        (self.prefix.base(page)+(self.mask[page]&through).count_ones()-1) as usize
    }
    fn descriptor(&self,row:usize)->(usize,u32) {
        let v=self.desc[self.band(row)];
        let q=(v.wrapping_add(row as u32))&((1u32<<self.bits)-1);
        (q as usize,v>>self.bits)
    }
}

#[derive(Default)]
struct Packed {buf:Vec<u64>,count:usize}
impl Packed {
    fn push(&mut self, v:u32, width:u32) {
        if width==0 {assert_eq!(v,0);return;}
        let i=self.count/64;
        let shift=self.count%64;
        if i==self.buf.len(){self.buf.push(0);}
        self.buf[i]|=u64::from(v)<<shift;
        if width as usize+shift>64 {self.buf.push(u64::from(v)>>(64-shift));}
        self.count+=width as usize;
    }
    fn get(&self, pos:usize, width:u32)->u32 {
        if width==0{return 0;}
        let (i,shift)=(pos/64,pos%64);
        let mut v=self.buf[i]>>shift;
        if width as usize+shift>64 {v|=self.buf[i+1]<<(64-shift);}
        (v as u32)&((1u32<<width)-1)
    }
}

fn bits(v:u32)->u32 {u32::BITS-v.leading_zeros()}
#[cfg(target_arch="x86_64")]
#[target_feature(enable="bmi2")]
unsafe fn pdep(mask:u64,j:usize)->u32 {
    std::arch::x86_64::_pdep_u64(1u64<<j,mask).trailing_zeros()
}
fn kth(mask:u64,j:usize,bmi:bool)->u32 {
    #[cfg(target_arch="x86_64")]
    if bmi {return unsafe{pdep(mask,j)};}
    let mut w=mask;
    for _ in 0..j {w&=w-1;}
    w.trailing_zeros()
}
#[derive(Clone,Copy)]
struct Header {stamp:u32, at:u32}
struct Groups {
    header:Vec<Header>, high:Vec<u64>, lows:Packed, exception_groups:usize,
    exception_entries:usize, widths:[usize;32], worst_delta:u32
}
impl Groups {
    fn new(queue:&[u32], global:u32, group:usize, ef:bool)->Self {
        let mut v=Self{header:Vec::new(),high:Vec::new(),lows:Packed::default(),exception_groups:0,
            exception_entries:0,widths:[0;32],worst_delta:0};
        let mask=(1u32<<global)-1;
        for q in queue.chunks(group) {
            let at=v.lows.count as u32;
            if ef {
                let sorted=q.windows(2).all(|x|x[0]<x[1]);
                let base=if sorted {q[0]} else {0};
                let mut w=if sorted {0} else {global};
                if sorted {
                    let span=q[q.len()-1]-base;
                    while (span>>w) as usize+q.len()>64 {w+=1;}
                    v.worst_delta=v.worst_delta.max(span);
                } else {v.exception_groups+=1;v.exception_entries+=q.len();}
                let low=(1u32<<w)-1;
                let mut high=0;
                for (j,&x) in q.iter().enumerate() {
                    let delta=x-base;
                    v.lows.push(delta&low,w);
                    if sorted {high|=1u64<<(j+(delta>>w) as usize);}
                }
                v.header.push(Header{stamp:base|w<<global,at});
                v.high.push(high);
                v.widths[w as usize]+=1;
            } else {
                let mut min=i32::MAX;
                let mut max=i32::MIN;
                for (j,&x) in q.iter().enumerate() {
                    let d=x as i32-j as i32;min=min.min(d);max=max.max(d);
                }
                let spread=(max-min) as u32;
                let w=bits(spread).min(global);
                let base=(min as u32)&mask;
                for (j,&x) in q.iter().enumerate() {
                    v.lows.push(x.wrapping_sub(base+j as u32)&mask,w);
                }
                v.worst_delta=v.worst_delta.max(spread);
                v.header.push(Header{stamp:base|w<<global,at});
                v.widths[w as usize]+=1;
            }
        }
        v
    }
    fn select(&self,k:usize,global:u32,group:usize,ef:bool,bmi:bool)->u32 {
        let g=k/group;
        let j=k%group;
        let h=self.header[g];
        let mask=(1u32<<global)-1;
        let base=h.stamp&mask;
        let w=h.stamp>>global;
        let low=self.lows.get(h.at as usize+j*w as usize,w);
        if !ef{return (base+j as u32+low)&mask;}
        let hi=self.high[g];
        if hi==0{return low;}
        base+((kth(hi,j,bmi)-j as u32)<<w)+low
    }
}

fn main() {
    let data=fs::read(env::args().nth(1).unwrap()).unwrap();
    let mut tape=data.as_slice();
    let bmi=cfg!(target_arch="x86_64") && std::is_x86_feature_detected!("bmi2");
    println!("block,stream,n,m,mark,bits,words,bands,mode,group,band_masks,band_prefix,band_descriptors,group_heads,group_unary,delta_bits,delta_bytes,boundary_groups,boundary_entries,worst_delta,checked");
    while !tape.is_empty() {
        let (block,stream,n,m,mark,b,words,nb)=(word(&mut tape),word(&mut tape),word(&mut tape),word(&mut tape),word(&mut tape),word(&mut tape),word(&mut tape),word(&mut tape));
        let edges:Vec<_>=(0..m).map(|_|word(&mut tape)).collect();
        let bands:Vec<_>=(0..nb).map(|_| Band{start:word(&mut tape) as usize,length:word(&mut tape) as usize,code:word(&mut tape),byte:word(&mut tape)}).collect();
        let g=Geometry::new(&edges,&bands,b);
        for ef in [false,true] {
            for group in [16,32,48,64] {
                let f=Groups::new(&g.q,b,group,ef);
                for (row,&want) in edges.iter().enumerate() {
                    let (i,code)=g.descriptor(row);
                    let next=f.select(i,b,group,ef,bmi);
                    assert_eq!((code<<b)|next,want);
                }
                let heads=f.header.len()*8;
                let unary=f.high.len()*8;
                println!("{block},{stream},{n},{m},{mark},{b},{words},{nb},{},{group},{},{},{},{heads},{unary},{},{},{},{},{},{}",
                    if ef {"ef"}else{"slope"},g.mask.len()*8,g.prefix.bytes(),g.desc.len()*4,
                    f.lows.count,f.lows.buf.len()*8,f.exception_groups,f.exception_entries,f.worst_delta,edges.len());
            }
        }
    }
}
