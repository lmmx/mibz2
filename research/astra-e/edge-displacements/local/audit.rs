// Standalone native-format and marked-cycle oracle. Build beside native.inc.rs.
#![allow(dead_code)]
struct Kernel {
    packed_pages:DisplacementPages, edges:Vec<u32>,extra:Vec<u32>,
    spelling:Vec<u8>,offsets:Vec<u32>
}
fn append_code(code:u32,spelling:&[u8],offsets:&[u32],out:&mut Vec<u8>){
    if code<256 {out.push(code as u8);}else{
        let i=(code-256)as usize;
        out.extend_from_slice(&spelling[offsets[i]as usize..offsets[i+1]as usize]);
    }
}
fn repeat(n:usize,out:&mut Vec<u8>){
    assert!(!out.is_empty() && out.len()<=n);
    while out.len()<n{let len=out.len().min(n-out.len());out.extend_from_within(..len);}
}
include!("native.inc.rs");

fn cases() {
    let mut rng=0x183b7a1u32;
    let mut next_random=||{rng^=rng<<13;rng^=rng>>17;rng^=rng<<5;rng};
    let mut p=DisplacementPages::default();
    let mut checks=0;
    for m in 1..=99 {
        for layout in 0..12 {
            let rb=width32(m as u32-1).max(1);
            let mut next=(0..m as u32).collect::<Vec<_>>();
            match layout%4 {
                0 => {},
                1 => next.reverse(),
                2 => {for i in 0..m{next.swap(i,next_random()as usize%m);}},
                _ => {for i in (0..m).step_by(7){next.swap(i,next_random()as usize%m);}}
            }
            let codes=(0..m).map(|i|match layout/4{
                0=>0,1=>(i/8%3)as u32,_=>next_random()%256
            }).collect::<Vec<_>>();
            let packed=next.iter().zip(&codes).map(|(y,c)|y|(c<<rb)).collect::<Vec<_>>();
            for wide in [false,true] {
                let arg=if wide{&next}else{&packed};
                let extra=if wide{codes.as_slice()}else{&[]};
                p.encode(arg,extra,rb,m,0);
                for i in 0..m {assert_eq!((p.next(i),p.code(i)),(next[i],codes[i]));}
                let mut want=Vec::new();let mut row=0;
                loop {want.push(codes[row]as u8);row=next[row]as usize;if row==0{break;}}
                repeat(m,&mut want);
                let mut out=Vec::new();p.emit(&[],&[],&mut out);
                assert_eq!(out,want);
                checks+=1;
            }
        }
    }
    println!("{checks} synthetic packed/wide cases, all rows, marks, partial pages, periodic expansion and retained-buffer reuse checked");
}

fn main(){
    cases();
    let args:Vec<String>=std::env::args().collect();
    if args.len()==1{return;}
    let raw=std::fs::read(&args[1]).unwrap();
    let expected=std::fs::read_to_string(&args[2]).unwrap();
    let mut sizes=std::collections::BTreeMap::new();
    for line in expected.lines().skip(1) {
        let cc:Vec<_>=line.split(',').collect();
        if cc[6]=="32" {sizes.insert(cc[0].parse::<u32>().unwrap(),cc[16].parse::<usize>().unwrap());}
    }
    let mut pos=0;let mut calls=0;let mut allrows=0;
    let mut p=DisplacementPages::default();
    let word=|at:usize|u32::from_le_bytes(raw[at..at+4].try_into().unwrap());
    while pos<raw.len() {
        let h:[u32;8]=std::array::from_fn(|i|word(pos+i*4));
        let m=h[3] as usize;pos+=32;
        let e=(0..m).map(|i|word(pos+4*i)).collect::<Vec<_>>();
        pos+=4*m+16*h[7]as usize;
        p.encode(&e,&[],h[5],h[2]as usize,h[4]);
        for (i,&v) in e.iter().enumerate() {
            assert_eq!(p.next(i)|(p.code(i)<<h[5]),v);
        }
        let n=40+4*(p.row_dir.len()+p.code_dir.len())+p.row_pool.len()+p.code_pool.len();
        assert_eq!(n,sizes[&h[0]]+14);
        calls+=1;allrows+=m;
    }
    assert_eq!((pos,calls,allrows),(raw.len(),233,26567581));
    println!("{calls} real blocks and {allrows} packed original edges reproduced with all per-block native sizes exactly 14 bytes above the P32 oracle");
}
