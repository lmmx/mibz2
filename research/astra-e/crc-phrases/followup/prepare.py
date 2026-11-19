from pathlib import Path
import subprocess,sys
from hashlib import sha256
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,here.parent/'prepare.py',out],check=True)
main=out/'main.rs'
s=main.read_text()
assert sha256((here.parent/'main.rs').read_bytes()).hexdigest()=='f4e0ca97769083d0e229b70e20bc574ea13ca864bcf01c85af93812878314fe2'
s=s.replace('fn x_bytes(mut n:usize)', 'fn reference_x_bytes(mut n:usize)')
start=s.index('#[derive(Clone,Copy,Default)] struct Digest')
s=s[:start]+'''const fn const_mul(mut a:u32,mut b:u32)->u32 {
    let mut v=0;
    while b!=0 {
        if b&1!=0 {v^=a;}
        b>>=1;
        let top=a>>31;
        a=(a<<1)^((P as u32)&0u32.wrapping_sub(top));
    }
    v
}
const fn powers()->[[u32;256];3] {
    let mut tab=[[0;256];3]; let mut step=256;
    let mut k=0;
    while k<3 {
        tab[k][0]=1;
        let mut i=1;
        while i<256 {tab[k][i]=const_mul(tab[k][i-1],step);i+=1;}
        step=const_mul(tab[k][255],step);k+=1;
    }
    tab
}
static POWERS:[[u32;256];3]=powers();
fn x_bytes(n:usize)->u32 {
    assert!(n<1<<24);
    let mut x=POWERS[0][n&255];
    if n>=256 {x=unsafe{multiply(x,POWERS[1][(n>>8)&255])};}
    if n>=65536 {x=unsafe{multiply(x,POWERS[2][(n>>16)&255])};}
    x
}
'''+s[start:]
s=s.replace('struct Digest {x:u32,r:u32,ready:bool}', 'struct Digest {x:u32,r:u32}').replace('if !d.ready {','if d.x==0 {').replace('            d.ready=true;','')
s=s.replace('    if fresh {digests.clear();', '''    if spans.is_empty() {return !crc_copy::crc::advance(!0,&b.output,crc_copy::Kernels{clmul:true});}
    if fresh {digests.clear();''')
# Retain every admitted block and use measured work proportional to a real corpus.
s=s.replace('    let mut nine=Vec::new();', '''    let mut all=Vec::new();
    for n in (0..900001).step_by(149).chain([0,1,255,256,65535,65536,900000]) {
        assert_eq!(x_bytes(n),reference_x_bytes(n));
    }
    let mut seed=0xb3d61ac5u32;
    for i in 0..32768u32 {
        seed^=seed<<13;seed^=seed>>17;seed^=seed<<5;
        let b=seed.rotate_left(i%32) ^ 0x5d440151;
        assert_eq!(unsafe{multiply(seed,b)},plain_mul(seed,b));
    }''')
s=s.replace('        if [1,85,134,176,225,269,313,359,397].contains(&index) {nine.push(b);}','        all.push(b);')
a=s.index('    assert_eq!((accepted,total,pos,nine.len())')
s=s[:a]+(here/'rounds.inc').read_text()
main.write_text(s)
