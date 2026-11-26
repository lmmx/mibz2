from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent;out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d/"prepare.py"),str(out)],check=True)
p=out/"main.rs";s=p.read_text();a=s.index("fn main(){")
s=s[:a]+'''
fn main(){
    let a:Vec<_>=std::env::args().collect();let which:usize=a[2].parse().unwrap();
    let data=std::fs::read(&a[1]).unwrap();let k=Kernels::detect();let mut arm=Arm::default();
    let mut scanner=Scanner::new(SliceSource(&data));let mut recycle=Vec::new();
    let (mut count,mut total,mut caps,mut bits)=(0,0,0usize,0usize);
    while let Ok(Event::Block{symbols:mut b,..})=scanner.next(recycle){
        let stock_runs=runs::takes_runs(&b);let (v,_,done)=arm.decode(&mut b,k,which);
        assert_eq!(v.unwrap(),b.crc);total+=arm.out.len();count+=1;
        if stock_runs&&!done{
            let m=b.syms.len();let need=m+b.max_len/127+3;if caps<need{caps=need.max(2*caps).max(4);}
            let need=m/64+3;if bits<need{bits=need.max(2*bits).max(4);}
        }
        recycle=b.syms;
    }
    let front=8*(arm.bits.lit.capacity()+arm.bits.one.capacity());
    let sep=2*arm.sep.capacity();let stock=2*caps+16*bits;
    let status=std::fs::read_to_string("/proc/self/status").unwrap();
    let peak=status.lines().find(|l|l.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap();
    assert_eq!((count,total),(431,209700196));
    println!("{which},{count},{total},{stock},{front},{sep},{peak}");
}'''
p.write_text(s)
