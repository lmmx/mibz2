"""Instrument a separate build, keeping the complete uninstrumented four-arm timings distinct."""
from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent;scratch=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d/"price.py"),str(scratch)],check=True,stdout=subprocess.DEVNULL)
p=scratch/"combo.rs"
s=p.read_text().replace("reader:Nav,corrected_caps:", "pub(super) phases:[u64;6],reader:Nav,corrected_caps:")
s=s.replace("        self.reader.build(caps);", "        self.phases=[0;6];let mut stamp=crate::cpu();self.reader.build(caps);self.phases[0]=crate::cpu()-stamp;stamp=crate::cpu();")
s=s.replace("            let fills:usize", "            self.phases[1]=crate::cpu()-stamp;stamp=crate::cpu();let fills:usize")
s=s.replace("                    let mark=order.position(p);","                    self.phases[2]=crate::cpu()-stamp;stamp=crate::cpu();let mark=order.position(p);")
s=s.replace("                    let (_,root,bits)=", "                    self.phases[3]=crate::cpu()-stamp;stamp=crate::cpu();let (_,root,bits)=")
s=s.replace("                    let start=out.len();", "                    self.phases[4]=crate::cpu()-stamp;stamp=crate::cpu();let start=out.len();")
s=s.replace("                    if out.len()-start==size", "                    self.phases[5]=crate::cpu()-stamp;if out.len()-start==size")
p.write_text(s)
s=(scratch/"main.rs").read_text()
s=s[:s.index("fn main()")]+'''fn main(){
    let data=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut sc=Scanner::new(SliceSource(&data));let mut r=runs::Runs::default();let mut ordinal=0;
    let mut old=ahead::Kernel::default();let mut edited=combo::Kernel::default();let mut raw=Vec::new();let mut out=Vec::new();
    let k=Kernels::detect();
    println!("stream,round,position,arm,complete_cpu,nav,hazards,reorder,scatter,geometry,copy,rle,reason");
    while let Ok(Event::Block{index,symbols,..})=sc.next(Vec::new()){
        let n=runs::front(&symbols,&mut r,k).unwrap();if n<65536{continue;}
        let sample=ordinal%24==0;ordinal+=1;if !sample||r.records().len()*20>=3*n {continue;}
        let caps=r.records();let p=symbols.orig_ptr;
        for round in 0..=24 {
            for pos in 0..2 {
                let arm=(round+pos)%2;out.clear();let began=cpu();let rle;let phases;let reason;
                if arm==0{
                    old.run::<true,true,false>(caps,n,p,16,&mut raw);
                    let t=cpu();mibz2::rle::unrle(&raw,&mut out,k).unwrap();rle=cpu()-t;phases=[0;6];reason=0;
                } else {
                    edited.expanded(caps,n,p,&mut out,k).unwrap();rle=0;phases=edited.phases;reason=edited.observations().0;
                }
                let elapsed=cpu()-began;assert_eq!(mibz2::crc::crc32(&out,k),symbols.crc);
                if round!=0{println!("{index},{round},{pos},{arm},{elapsed},{},{},{},{},{},{},{rle},{reason}",phases[0],phases[1],phases[2],phases[3],phases[4],phases[5]);}
            }
        }
    }
}'''
(scratch/"attribution.rs").write_text(s)
