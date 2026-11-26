use mibz2::{Kernels,runs,stream::{Event,Scanner,SliceSource}};
include!("../../edge-budget/gate.rs");

fn main() {
    let data = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scan = Scanner::new(SliceSource(&data));
    let mut runs = runs::Runs::default();
    println!("stream,n,records,maximal,short_prefix,full16,partial,cap_groups,tail,mark_possible");
    while let Ok(Event::Block{index,symbols:s,..}) = scan.next(Vec::new()) {
        if !runs::takes_runs(&s) { continue; }
        let n = runs::front(&s,&mut runs,Kernels::detect()).unwrap();
        let r = runs.records();
        if r.len()*20 >= 3*n && (n < 65_536 || !fits(r,s.orig_ptr)) {continue;}
        let (mut i, mut pos, mut maximal, mut short, mut full, mut partial, mut cap, mut tail, mut marked) = (0,0,0,0,0,0,0,0,0);
        while i < r.len() {
            let batch = (r.len()-i).min(16);
            let prefix = r[i..i+batch].iter().take_while(|v| (*v & 255) < 127).count();
            if batch < 16 {tail += prefix;} else if prefix == 16 {full += 1;} else {partial += 1;}
            if (pos..pos+126*prefix).contains(&(s.orig_ptr as usize)){marked += prefix;}
            for &v in &r[i..i+prefix] {pos += usize::from(v & 255);}
            short += prefix; maximal += prefix; i += prefix;
            if prefix < batch {
                let byte = r[i] >> 8;
                assert_eq!(r[i]&255,127); cap += 1; maximal += 1;
                while i < r.len() && r[i]>>8 == byte {pos += usize::from(r[i] & 255); i += 1;}
            }
        }
        assert_eq!(pos,n);
        println!("{index},{n},{},{maximal},{short},{full},{partial},{cap},{tail},{marked}",r.len());
    }
}
