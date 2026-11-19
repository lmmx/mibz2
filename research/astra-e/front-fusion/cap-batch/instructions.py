"""Measure the frozen block's initial queue/geometry scan after one untimed decode."""
from pathlib import Path
import subprocess,sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here/"prepare.py"),str(out)],check=True)
for name in ["control.rs","scalar.rs","grouped.rs"]:
    p=out/name
    s=p.read_text().replace("        let mut last = None;", "        crate::cut(true);\n        let mut last = None;")
    s=s.replace("        assert!(m > 0 && all.iter()", "        crate::cut(false);\n        assert!(m > 0 && all.iter()")
    p.write_text(s)
p=out/"main.rs"
s=p.read_text().split("fn main(){")[0]
# The opcodes and client request ABI are shared with the independent cost ledger.
prior=(here.parents[1]/"narrow-short/cost/main.rs").read_text()
a=prior.index("#[inline(never)]\nfn callgrind(")
b=prior.index("#[inline(never)]\nfn cut(",a)
s+=prior[a:b]+"""
thread_local!{static MEASURE: std::cell::Cell<bool> = const{std::cell::Cell::new(false)};}
#[inline(never)]
fn cut(start:bool){
    if !MEASURE.with(std::cell::Cell::get){return;}
    if start{callgrind(4,0);callgrind(1,0);}
    else{callgrind(3,b"geometry\\0".as_ptr() as usize);callgrind(5,0);}
}
fn main(){
    let a:Vec<_>=std::env::args().collect();
    let which:usize=a[2].parse().unwrap();let panel=a[3]=="panel";let target:u64=a[3].parse().unwrap_or(u64::MAX);
    let input=std::fs::read(&a[1]).unwrap();
    let mut scan=Scanner::new(SliceSource(&input));
    let k=Kernels::detect();assert!(is_x86_feature_detected!("avx2"));
    let mut worker:Box<dyn Decoder>=match which{
        0|1=>Box::new(Worker::<arm0::Candidate>::default()),
        2=>Box::new(Worker::<arm2::Candidate>::default()),
        3=>Box::new(Worker::<arm3::Candidate>::default()),
        _=>unreachable!()
    };
    while let Ok(Event::Block{index,symbols:s,..})=scan.next(Vec::new()){
        let ids=[1,19,41,56,61,108,150,165,194,240,270,278,282,329,371,407];
        if (panel && !ids.contains(&index)) || (!panel && index!=target) {continue;}
        MEASURE.with(|x|x.set(false));
        let expected=worker.decode(&s,k).to_owned();
        MEASURE.with(|x|x.set(true));
        let output=worker.decode(&s,k);
        assert_eq!(expected,output);
        println!("arm {which}, block {index}, {} checked decoded bytes",output.len());
        if !panel{return;}
    }assert!(panel)
}
"""
p.write_text(s)
