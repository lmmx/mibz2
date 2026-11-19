from pathlib import Path
import subprocess,sys,re
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,here/'prepare.py'],check=True)
subprocess.run([sys.executable,here.parent/'crc-phrases/prepare.py',out],check=True)
for tag,base in [('old',out/'frozen/research/astra-e/narrow-short/constructor_ahead.rs'),('new',here/'constructor.rs')]:
    s=base.read_text()
    s=re.sub(r'(#\[path = ")([^"]+)"',lambda m:m[1]+str((base.parent/m[2]).resolve())+'"',s)
    if tag=='old':
        s+='''
impl Kernel {
    pub fn native_price(&mut self,rs:&[u16],n:usize,p:u32,out:&mut Vec<u8>)->[u64;2] {
        let t0=crate::clock();let (_,mark,bits)=self.construct::<true,true>(rs,n as u32,p,16);let t1=crate::clock();
        emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,out);
        let t2=crate::clock();[t1-t0,t2-t1]
    }
    pub fn edges(&self)->(usize,Vec<(u32,u32)>) {
        let bits=(32-(self.edges.len() as u32-1).leading_zeros()).max(1);
        let m=(1<<bits)-1;
        (4*(self.edges.capacity()+self.extra.capacity()),self.edges.iter().enumerate().map(|(i,&e)|(e&m,if self.extra.is_empty(){e>>bits}else{self.extra[i]})).collect())
    }
}
'''
    else:
        s+='''
impl Kernel {
    pub fn native_price<const P:usize>(&mut self,rs:&[u16],n:usize,p:u32,out:&mut Vec<u8>)->[u64;2] {
        let t0=crate::clock();let (_,mark,bits)=self.construct::<true,true,P>(rs,n as u32,p,16);let t1=crate::clock();
        assert!(self.packed);
        self.emit_packed::<P>(n,mark,bits,out);
        let t2=crate::clock();[t1-t0,t2-t1]
    }
    pub fn check<const P:usize>(&self,edges:&[(u32,u32)])->[usize;4] {
        assert!(self.packed);
        let bits=(32-(edges.len() as u32-1).leading_zeros()).max(1);
        let mask=(1<<bits)-1;
        assert_eq!(self.pack.len(),edges.len()+1);
        for (i,&x) in edges.iter().enumerate() {
            let e=self.load24(i as u32);
            assert_eq!((e&mask,self.label24::<P>(i as u32,e,bits)),x);
        }
        [self.pack.capacity()*3,self.headers.capacity()*2,self.high.capacity(),4*(self.edges.capacity()+self.extra.capacity())]
    }
}
'''
    (out/f'{tag}.rs').write_text(s)
(out/'main.rs').write_text('#![allow(dead_code)]\nmod old;mod new;\n'+(here/'price-main.rs').read_text())
