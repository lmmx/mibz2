use mibz2::{Kernels,runs,stream::{Event,Scanner,SliceSource}};
include!("../../edge-budget/gate.rs");

fn main() {
    let data = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scan = Scanner::new(SliceSource(&data));
    let mut runs = runs::Runs::default();
    println!("stream,records,baseline_neighbor_labels,scalar_neighbor_labels,full_predecessors,redundant_after_tail");
    while let Ok(Event::Block{index,symbols:s,..}) = scan.next(Vec::new()) {
        if !runs::takes_runs(&s) { continue; }
        let n = runs::front(&s,&mut runs,Kernels::detect()).unwrap();
        let r = runs.records();
        if r.len()*20 >= 3*n && (n < 65_536 || !fits(r,s.orig_ptr)) {continue;}
        let (mut i,mut touched,mut redundant)=(0,0,0);
        while i<r.len(){
            let start=r[i];i+=1;
            if start&255!=127 {continue;}
            let mut last=start;
            while i<r.len(){
                touched+=1;
                redundant+=usize::from(last&255!=127);
                if r[i]>>8!=start>>8 {break;}
                last=r[i];i+=1;
            }
        }
        let informative=r[..r.len()-1].iter().filter(|v|(*v&255)==127).count();
        assert_eq!(touched,informative+redundant);
        println!("{index},{},{},{touched},{informative},{redundant}",r.len(),r.len()-1);
    }
}
