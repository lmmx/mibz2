use mibz2::{ErrorKind,Kernels};
const WINDOW:usize=32;

#[derive(Default)]
pub struct Audit {
    pub accepted:bool,
    pub direct:usize,
    pub joined:usize,
    pub bridge_bytes:usize,
    pub avoided:usize,
    pub fallback:&'static str,
}

#[derive(Clone,Copy,Default)]
pub struct Span {pub start:usize,pub len:usize}

// These addresses are logical marked positions. Locate at most four physical fragments.
#[inline]
fn byte<const K:usize>(x:usize,src:&[u8],parts:&[Span;K],end:&[usize;K])->u8 {
    let mut i=0;
    while x>=end[i] {i+=1;}
    let first=if i==0 {0} else {end[i-1]};
    src[parts[i].start + x-first]
}
#[inline]
fn safe<const K:usize>(x:usize,src:&[u8],parts:&[Span;K],end:&[usize;K])->bool {
    if x==0 || x==src.len() {return true;}
    let a=byte(x-1,src,parts,end);
    if a==byte(x,src,parts,end) {return false;}
    x<4 || (2..=4).any(|k|byte(x-k,src,parts,end)!=a)
}

// None is returned before any change to dst. Once a plan is certified, wrong data
// reaches stock unrle and its exact error reaches the ordinary caller.
pub fn parse<const K:usize,const AUDIT:bool>(
    src:&[u8],parts:&[Span;K],dst:&mut Vec<u8>,kernels:Kernels,audit:&mut Audit,
)->Option<Result<(),ErrorKind>> {
    let mut n=0;
    let ends:[usize;K]=std::array::from_fn(|i|{n+=parts[i].len;n});
    assert_eq!(n,src.len());
    let mut prefix=[0;K];
    let mut suffix=[0;K];
    for i in 0..K-1 {
        let seam=ends[i];
        if safe(seam,src,parts,&ends) {
            if AUDIT {audit.direct+=1;}
            continue;
        }
        let before=(1..=WINDOW.min(parts[i].len)).find(|&h|safe(seam-h,src,parts,&ends));
        let after=(1..=WINDOW.min(parts[i+1].len)).find(|&h|safe(seam+h,src,parts,&ends));
        let (Some(x),Some(y))=(before,after) else {
            if AUDIT {audit.fallback="radius";}
            return None;
        };
        suffix[i]=x;prefix[i+1]=y;
        if AUDIT {audit.joined+=1;audit.bridge_bytes+=x+y;}
    }
    // The nearest certified cut from the left cannot lie strictly after the
    // nearest one from the right: that right cut would be a nearer left candidate.
    debug_assert!((0..K).all(|i|prefix[i]+suffix[i]<=parts[i].len));
    if AUDIT {audit.accepted=true;}
    dst.reserve(n+if kernels.avx2(){64}else{0});
    Some((||{
        let mut bridge=[0u8;WINDOW*2];
        for i in 0..K {
            let p=parts[i];
            let body=&src[p.start+prefix[i]..p.start+p.len-suffix[i]];
            if !body.is_empty() {mibz2::rle::unrle(body,dst,kernels)?;}
            let tail=suffix[i];
            if tail!=0 {
                let head=prefix[i+1];
                bridge[..tail].copy_from_slice(&src[p.start+p.len-tail..p.start+p.len]);
                bridge[tail..tail+head].copy_from_slice(&src[parts[i+1].start..parts[i+1].start+head]);
                mibz2::rle::unrle(&bridge[..tail+head],dst,kernels)?;
            }
        }
        Ok(())
    })())
}
