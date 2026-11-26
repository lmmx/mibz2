use mibz2::Kernels;
mod log;mod front;
#[derive(Default)]struct Bits{lit:Vec<u64>,one:Vec<u64>}

fn run<const DEFER:bool>(gold:&[u16],alphabet:&[u8;256],max_len:usize,valid:bool){
    let mut ideal=gold.to_vec();
    let p=log::compact(&mut ideal,&alphabet[..3],usize::MAX,max_len as u64);
    let mut guarded=vec![0xfe11;32];guarded.extend_from_slice(gold);guarded.extend([0xfa0b;32]);
    let s=&mut guarded[32..32+gold.len()];let ptr=s.as_mut_ptr();let mut words=0;
    let mut bits=Bits::default();
    let result=unsafe{front::pack::<DEFER>(ptr,s.len(),ptr,4,alphabet,max_len,&mut bits,&mut words)};
    if valid{
        let(n,c)=result.unwrap();assert!(!p.failed);
        assert_eq!(words,p.words);assert_eq!(&s[..words],&ideal[..words]);
        if !DEFER{
            assert_eq!(n as u64,p.rows);let(mut i,mut caps)=(0,0usize);
            while i<words{caps+=log::take(s,&mut i).1.div_ceil(127)as usize;}
            assert_eq!(c,caps);
        }
    }
    log::restore(s,words,&alphabet[..3]);assert_eq!(s,gold);
    assert!(guarded[..32].iter().all(|&v|v==0xfe11));
    assert!(guarded[32+gold.len()..].iter().all(|&v|v==0xfa0b));
}

fn main(){
    assert!(Kernels::detect()!=Kernels::scalar());
    let mut table=[0u8;256];for(i,v)in table.iter_mut().enumerate(){*v=i as u8;}
    let widths=[1,2,15,16,17,126,127,128,253,254,255,256,65534,65535,65536,65537,899000];
    let(mut exact,mut rejected)=(0,0);
    for offset in 0..64{
        for w in widths{
            for open in [false,true]{
                if open&&offset!=0{continue;}
                for tail in [0,1,31,32,63]{
                    let mut s=vec![2;offset];
                    if !open{s.push(3);}
                    let v:u32=w+u32::from(open);
                    for k in 0..v.ilog2(){s.push(((v>>k)&1)as u16);}
                    s.extend(std::iter::repeat_n(2,tail));s.push(4);s.extend([0,1,2,3,4]);
                    run::<false>(&s,&table,900000,true);run::<true>(&s,&table,900000,true);exact+=2;
                    if w>255{
                        run::<false>(&s,&table,200,false);run::<true>(&s,&table,200,false);rejected+=2;
                    }
                }
            }
        }
    }
    println!("{exact} in-place AVX2 direct, first EOB, tag and bit-boundary checks; {rejected} reduced-bound rollback checks; both guard regions preserved");
}
