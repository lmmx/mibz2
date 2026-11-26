pub use mibz2::{bwt,runs,Kernels};
#[allow(dead_code,unused_imports)] mod shared;
#[allow(dead_code,unused_imports)] mod control;
use std::io::{Read,BufReader};
use std::collections::HashMap;
fn digest(bytes:&[u8])->u64{bytes.iter().fold(0xcbf29ce484222325,|h,&b|(h^u64::from(b)).wrapping_mul(0x100000001b3))}
fn hwm()->usize{
    let status=std::fs::read_to_string("/proc/self/status").unwrap();
    status.lines().find(|s|s.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap().parse().unwrap()
}
fn main(){
    let args=std::env::args().collect::<Vec<_>>();
    let arm=&*args[1];
    let data=&args[2];
    let tables=&args[3];
    let validate=arm=="truth";
    let mut intervals=HashMap::new();
    for line in std::fs::read_to_string(tables).unwrap().lines().skip(1){
        let c=line.split(',').collect::<Vec<_>>();
        intervals.insert((c[0].parse::<u32>().unwrap(),c[1].to_string()),(c[2].parse::<u32>().unwrap(),c[3].parse::<u32>().unwrap()));
    }
    let mut hashes=HashMap::new();
    if !validate{
        for line in std::fs::read_to_string(&args[4]).unwrap().lines().skip(1){
            let (a,b)=line.split_once(',').unwrap();hashes.insert(a.parse::<u32>().unwrap(),b.parse::<u64>().unwrap());
        }
    }
    if validate{println!("stream,fnv64");}else{println!("stream,arm,rotated,high_water_kib,retained_bytes_including_output");}
    let mut input=BufReader::new(std::fs::File::open(data).unwrap());
    let mut original=control::Dispatch::<2,2>::default();
    let mut subject=shared::Dispatch::<2,2>::default();
    let mut output=Vec::new();let mut checked=0;
    loop{
        let mut head=[0u8;16];if input.read(&mut head[..1]).unwrap()==0{break;}
        input.read_exact(&mut head[1..]).unwrap();
        let mut it=head.chunks_exact(4).map(|c|u32::from_le_bytes(c.try_into().unwrap()));
        let idx=it.next().unwrap();let n=it.next().unwrap();let p=it.next().unwrap();let m=it.next().unwrap();
        let mut buf=vec![0;2*m as usize];input.read_exact(&mut buf).unwrap();
        let caps=buf.chunks_exact(2).map(|c|u16::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>();
        drop(buf);
        let mut freq=[0u32;256];for &v in &caps{freq[(v>>8) as usize]+=u32::from(v&255);}
        if validate{
            let mut l=Vec::new();for v in &caps{l.extend(std::iter::repeat_n((v>>8) as u8,(v&255) as usize));}
            let mut psi=(0..n as usize).collect::<Vec<_>>();psi.sort_by_key(|&i|l[i]);
            let mut row=p as usize;output.clear();
            for _ in 0..n{row=psi[row];output.push(l[row]);}
            println!("{idx},{}",digest(&output));
        }else{
            let label=if arm=="original"{"quote"}else{arm};
            let &(lo,hi)=intervals.get(&(idx,label.to_owned())).unwrap();
            let mut a=0;let (byte,one)=freq.iter().enumerate().find_map(|(b,&w)|{a+=w;(a>lo).then_some((b as u8,hi<=a))}).unwrap();
            let single=one && arm!="raw_quote";
            let bytes=if arm=="original"{
                let (mark,tok)=original.first(&caps,&freq,n,p).unwrap();
                original.finish(mark,tok,n,p,&mut output);
                original.whole_capacity()
            }else{
                let (mark,tok)=if single{subject.first::<true>(&caps,&freq,n,p,(lo,hi),byte)}
                            else{subject.first::<false>(&caps,&freq,n,p,(lo,hi),byte)}.unwrap();
                subject.finish(mark,tok,n,p,&mut output);
                subject.whole_capacity()
            };
            assert_eq!(digest(&output),hashes[&idx]);
            println!("{idx},{arm},{single},{},{}",hwm(),bytes+output.capacity());
        }
        checked+=1;
    }
    assert_eq!(checked,228);
}
