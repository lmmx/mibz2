from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
p=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d/"prepare.py"),str(p)],check=True)
s=(p/"main.rs").read_text();s=s[:s.index("fn main(){")]+'''include!("edge_boundaries.rs");
fn main(){
    let args:Vec<_>=std::env::args().collect();
    if args[1]=="boundaries"{edge_boundaries();return;}
    let which:usize=args[2].parse().unwrap();let mut arm=Arm::default();
    let data=std::fs::read(&args[1]).unwrap();let mut scan=Scanner::new(SliceSource(&data));let k=Kernels::detect();
    let mut count=0;let mut direct=0;let mut bytes=0;
    let mut caps=Vec::<u16>::new();let mut one=Vec::<u64>::new();let mut lit=Vec::<u64>::new();
    while let Ok(Event::Block{index,symbols,..})=scan.next(Vec::new()){
        let (n,records,done)=if which==2{arm.block::<64>(&symbols,k,true)}else{arm.block::<0>(&symbols,k,which==1)}.unwrap();
        assert_eq!(mibz2::crc::crc32(&arm.decoded,k),symbols.crc);bytes+=arm.decoded.len();count+=1;direct+=usize::from(done);
        if runs::takes_runs(&symbols)&&!done{
            let m=symbols.syms.len();caps.reserve(m+symbols.max_len/127+3);one.reserve(m/64+3);lit.reserve(m/64+3);
        }
        let simulated=2*caps.capacity()+8*(one.capacity()+lit.capacity());
        let extra=8*(arm.bits.one.capacity()+arm.bits.lit.capacity());
        println!("{which},{index},{n},{records},{done},{simulated},{extra},{}",arm.kernel.queues_and_geometry());
    }
    let rss=std::fs::read_to_string("/proc/self/status").unwrap().lines().find(|s|s.starts_with("VmHWM:")).unwrap().to_owned();
    eprintln!("arm {which}: {count} real CRCs, {bytes} bytes, {direct} direct completions; {rss}");
}'''
# Vec length stays zero in the simulated reserve ledger. No payload is touched in those vectors.
(p/"main.rs").write_text(s)
(p/"edge_boundaries.rs").write_bytes((d/"boundaries.rs").read_bytes())
with (p/"kernel.rs").open("a") as f:f.write('''\nimpl Kernel{
 pub(super) fn queues_and_geometry(&self)->usize{
     self.queues.capacity()*std::mem::size_of::<Vec<u32>>() + self.queues.iter().map(|v|v.capacity()*4).sum::<usize>() + self.strips.capacity()*std::mem::size_of::<Strip>()
 }
}\n''')
