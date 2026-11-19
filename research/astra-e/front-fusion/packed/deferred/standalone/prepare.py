"""Build four standalone executables to isolate representation and binary-layout changes."""
from pathlib import Path
import subprocess,sys

here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
deps=Path(sys.argv[2]).resolve()
out.mkdir(parents=True,exist_ok=True)
for name,generator,mode in [("edge",here.parent.parent/"prepare.py",0),
                            ("inplace_separate_build",here.parent.parent/"prepare.py",2),
                            ("inplace_only_build",here.parent/"prepare.py",2),
                            ("deferred",here.parent/"prepare.py",3)]:
    src=out/name
    subprocess.run([sys.executable,str(generator),str(src)],check=True)
    p=src/"main.rs";s=p.read_text();s=s[:s.index("fn main(){")]
    s+="""
fn main(){
    let data=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let k=Kernels::detect();let mut arm=Arm::default();let mut recycle=Vec::new();
    let(mut blocks,mut total,mut direct)=(0,0,0);
    let wall=std::time::Instant::now();let now=cpu();
    for _ in 0..2 {
        let mut scan=Scanner::new(SliceSource(&data));
        loop {
            let event=scan.next(std::mem::take(&mut recycle)).unwrap();
            if let Event::Block{symbols:mut b,..}=event{
                let (answer,_,done)=arm.decode(&mut b,k,MODE);
                assert_eq!(answer.unwrap(),b.crc);direct+=usize::from(done);
                total+=arm.out.len();blocks+=1;recycle=b.syms;
            }else{break;}
        }
    }
    let elapsed=cpu()-now;let wall_ns=wall.elapsed().as_nanos();
    assert_eq!((total,blocks),(419400392,862));
    assert_eq!(direct,if MODE==0{0}else{466});
    let status=std::fs::read_to_string("/proc/self/status").unwrap();
    let rss=status.lines().find(|l|l.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap();
    println!("{elapsed},{wall_ns},{rss},{blocks},{total},{direct}");
}
"""
    p.write_text(s.replace("MODE",str(mode)))
    subprocess.run(["rustc","-O","--edition","2021","-A","unused","-C","codegen-units=1",
                    str(p),"--extern",f"mibz2={deps}/libmibz2.rlib","-L",f"dependency={deps}","-o",str(out/(name+".bin"))],check=True)
