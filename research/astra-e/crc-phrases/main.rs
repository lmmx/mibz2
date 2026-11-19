pub use mibz2::{bwt,runs,Kernels};
use bwt::{InverseBwt,Segmented};
use mibz2::stream::{Event,Scanner,SliceSource};
use std::{io::Write,path::Path};
use std::hint::black_box;

const CUTS:[usize;8]=[16,32,64,128,256,512,1024,2048];
#[derive(Clone,Copy)]
struct Span { id:usize, start:usize, end:usize }
struct Input { index:u64, output:Vec<u8>, crc:u32, spans:Vec<Span>, words:usize }
#[repr(C)] struct Timespec { sec:i64, ns:i64 }
unsafe extern "C" {fn clock_gettime(id:i32,t:*mut Timespec)->i32;}
fn clock()->u64 {let mut t=Timespec{sec:0,ns:0}; assert_eq!(unsafe{clock_gettime(3,&mut t)},0);t.sec as u64*1_000_000_000+t.ns as u64}

fn count_positions(raw:&[u8])->Vec<(usize,i64)> {
    let (mut i, mut adjustment)=(0usize,0i64);
    let mut edits=vec![];
    while i<raw.len() {
        if i+3<raw.len() && raw[i..i+4]==[raw[i];4] {
            let count=raw[i+4];
            adjustment+=i64::from(count)-1;
            edits.push((i+4,adjustment)); i+=5;
        } else {i+=1;}
    }
    edits
}

fn capture(kernel:&mut kernel::Kernel,rs:&runs::Runs,n:usize,p:u32,raw:&[u8],output:&[u8])->(Vec<Span>,usize,[usize;5]) {
    let (codes,offsets,spelling)=kernel.capture(rs.records(),n,p);
    let edits=count_positions(raw);
    let (mut pos,mut e)=(0,0);
    let mut spans=Vec::new();
    let mut charged=[0usize;5]; // visits, coded-word bytes, clipped, unsafe words, unsafe coded bytes
    for code in codes.iter().cycle() {
        if pos==n {break}
        charged[0]+=1;
        let literal=[*code as u8];
        let id=code.saturating_sub(256) as usize;
        let word=if *code<256 {&literal[..]} else {&spelling[offsets[id] as usize..offsets[id+1] as usize]};
        let take=word.len().min(n-pos);
        assert!(take>0 && raw[pos..pos+take]==word[..take]);
        let adjusted=(pos as i64+if e==0 {0} else {edits[e-1].1}) as usize;
        let before=e;
        while e<edits.len() && edits[e].0<pos+take {e+=1;}
        if *code>=256 {
            charged[1]+=take;
            if take!=word.len() {charged[2]+=1;}
            else if e==before {
                assert_eq!(&output[adjusted..adjusted+take],word);
                spans.push(Span{id,start:adjusted,end:adjusted+take});
            } else {charged[3]+=1; charged[4]+=take;}
        }
        pos+=take;
    }
    assert_eq!((e,output.len() as i64),(edits.len(),n as i64+edits.last().map_or(0,|q|q.1)));
    (spans,offsets.len()-1,charged)
}

// Leave a phrase's first safe visit in its ordinary gap. A digest is needed
// only if a later safe visit of the exact same dictionary ID really occurs.
fn spans_for(input:&Input,cut:usize)->Vec<Span> {
    let mut seen=vec![false;input.words];
    input.spans.iter().copied().filter(|s| {
        s.end-s.start>=cut && std::mem::replace(&mut seen[s.id],true)
    }).collect()
}

fn geometry(b:&Input,spans:&[Span])->[usize;8] {
    let mut seen=vec![false;b.words];
    let (mut pos,mut covered,mut unique,mut body,mut gaps,mut tiny,mut tiny_bytes)=(0,0,0,0,0,0,0);
    for s in spans {
        let len=s.end-s.start;
        let gap=s.start-pos;
        if gap>0 {gaps+=1;if gap<128 {tiny+=1;tiny_bytes+=gap;}}
        covered+=len;
        if !std::mem::replace(&mut seen[s.id],true) {unique+=1;body+=len;}
        pos=s.end;
    }
    let gap=b.output.len()-pos;
    if gap>0 {gaps+=1;if gap<128 {tiny+=1;tiny_bytes+=gap;}}
    [spans.len(),covered,unique,body,gaps,tiny,tiny_bytes,pos]
}

const P:u64=0x104c11db7;
fn reduce(mut v:u64)->u32 {
    while v>>32!=0 {v^=P<<(31-v.leading_zeros());}
    v as u32
}
fn plain_mul(a:u32,mut b:u32)->u32 {
    let mut v=0u64; let mut x=u64::from(a);
    while b!=0 {if b&1!=0 {v^=x;} b>>=1; x<<=1;}
    reduce(v)
}
const fn reciprocal()->u64 {
    let mut r=1u128<<64; let mut q=0u128;
    while r>>32!=0 {let k=95-r.leading_zeros(); r^=(P as u128)<<k; q|=1u128<<k;}
    q as u64
}
#[cfg(target_arch="x86_64")]
#[target_feature(enable="pclmulqdq,sse4.1")]
unsafe fn multiply(a:u32,b:u32)->u32 {
    use std::arch::x86_64::*;
    let low=|u:u64,v:u64| _mm_cvtsi128_si64(_mm_clmulepi64_si128::<0>(_mm_cvtsi64_si128(u as i64),_mm_cvtsi64_si128(v as i64))) as u64;
    let v=low(u64::from(a),u64::from(b));
    let q=low(v>>31,reciprocal())>>33;
    let r=(v^low(q,P)) & 0x1ffffffff;
    (r ^ (P & 0u64.wrapping_sub(r>>32))) as u32
}
fn x_bytes(mut n:usize)->u32 {
    let(mut result,mut step)=(1,256);
    while n>0 {if n&1!=0 {result=plain_mul(result,step);} step=plain_mul(step,step);n>>=1;}
    result
}
#[derive(Clone,Copy,Default)] struct Digest {x:u32,r:u32,ready:bool}
fn checksum(b:&Input,spans:&[Span],digests:&mut Vec<Digest>,fresh:bool)->u32 {
    if fresh {digests.clear();digests.resize(b.words,Digest::default());}
    let mut pos=0; let mut state=!0;
    for s in spans {
        let k=crc_copy::Kernels{clmul:true};
        if pos<s.start {state=crc_copy::crc::advance(state,&b.output[pos..s.start],k);}
        let d=&mut digests[s.id];
        if !d.ready {
            d.x=x_bytes(s.end-s.start);
            d.r=crc_copy::crc::advance(0,&b.output[s.start..s.end],crc_copy::Kernels{clmul:true});
            d.ready=true;
        }
        state=unsafe{multiply(state,d.x)}^d.r;
        pos=s.end;
    }
    !crc_copy::crc::advance(state,&b.output[pos..],crc_copy::Kernels{clmul:true})
}
fn main() {
    assert!(std::is_x86_feature_detected!("pclmulqdq") && std::is_x86_feature_detected!("sse4.1") && std::is_x86_feature_detected!("ssse3"));
    let args:Vec<_>=std::env::args().collect();
    let dir=Path::new(&args[3]);
    let compressed=std::fs::read(&args[1]).unwrap();
    let oracle=std::fs::read(&args[2]).unwrap();
    let mut witness=std::io::BufWriter::new(std::fs::File::create(dir.join("blocks.csv")).unwrap());
    let mut geo=std::io::BufWriter::new(std::fs::File::create(dir.join("geometry.csv")).unwrap());
    writeln!(witness,"block,coded,decoded,words,safe_visits,all_visits,word_bytes,clips,hazard_word_visits,hazard_word_bytes").unwrap();
    writeln!(geo,"block,cut,visits,covered,unique,digest_bytes,gaps,short_gaps,short_gap_bytes").unwrap();
    let mut s=Scanner::new(SliceSource(&compressed));
    let (mut r,mut reference,mut kernel)=(runs::Runs::default(),Segmented::default(),kernel::Kernel::default());
    let k=Kernels::detect(); let(mut pos,mut accepted,mut total)=(0,0,0);
    let mut nine=Vec::new();
    let mut raw=Vec::new();let mut output=Vec::new();
    while let Event::Block{index,symbols,..}=s.next(Vec::new()).unwrap() {
        let n=runs::front(&symbols,&mut r,k).unwrap();
        assert!(reference.invert_runs(&r,n,symbols.orig_ptr,&mut raw,k));
        output.clear(); mibz2::rle::unrle(&raw,&mut output,k).unwrap();
        assert_eq!(output,oracle[pos..pos+output.len()]);
        pos+=output.len();total+=1;
        assert_eq!(mibz2::crc::crc32(&output,k),symbols.crc);
        if !runs::takes_runs(&symbols) || n<65536 || r.records().len()*20>=3*n {continue}
        let(spans,words,c)=capture(&mut kernel,&r,n,symbols.orig_ptr,&raw,&output);
        let b=Input{index,output:output.clone(),crc:symbols.crc,spans,words};
        writeln!(witness,"{index},{n},{},{},{},{},{},{},{},{}",output.len(),b.words,b.spans.len(),c[0],c[1],c[2],c[3],c[4]).unwrap();
        for cut in CUTS {
            let sel=spans_for(&b,cut); let g=geometry(&b,&sel);
            let mut digest=vec![];
            assert_eq!(checksum(&b,&sel,&mut digest,true),b.crc);
            writeln!(geo,"{index},{cut},{},{},{},{},{},{},{}",g[0],g[1],g[2],g[3],g[4],g[5],g[6]).unwrap();
        }
        accepted+=1;
        if [1,85,134,176,225,269,313,359,397].contains(&index) {nine.push(b);}
    }
    assert_eq!((accepted,total,pos,nine.len()),(228,431,209700196,9));
    // Preregistered increasing thresholds 32,128,512,2048 cover small, fold
    // threshold, medium and long spells. All paid arms still read the gaps.
    let cuts=[32,128,512,2048];
    let mut timings=std::io::BufWriter::new(std::fs::File::create(dir.join("times.csv")).unwrap());
    writeln!(timings,"block,round,position,cut,fresh,cpu_ns").unwrap();
    let mut initialized:Vec<Vec<Vec<Digest>>>=vec![vec![vec![];4];9];
    let plans:Vec<Vec<Vec<Span>>>=nine.iter().map(|b|cuts.iter().map(|&c|spans_for(b,c)).collect()).collect();
    let mut scratch=vec![];
    for round in 0..=54usize {
        for off in 0..9 {
            let bi=(off+round*5)%9; let b=&nine[bi];
            for position in 0..9 {
                let arm=((if round%2==0 {position} else {8-position})+round+bi)%9;
                let start=clock();
                let got=if arm==0 {mibz2::crc::crc32(black_box(&b.output),k)} else {
                    let i=(arm-1)%4;
                    let dst=if arm<=4 {&mut scratch} else {&mut initialized[bi][i]};
                    let fresh=arm<=4 || dst.is_empty();
                    checksum(black_box(b),&plans[bi][i],dst,fresh)
                };
                let elapsed=clock()-start;
                assert_eq!(got,b.crc);
                if round>0 {writeln!(timings,"{},{round},{position},{},{},{elapsed}",b.index,if arm==0 {0}else{cuts[(arm-1)%4]},arm<=4 && arm!=0).unwrap();}
            }
        }
    }
}
