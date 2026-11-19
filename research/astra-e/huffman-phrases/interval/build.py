"""Instantiate two identically emitting BMI2 scanners with separate table constructors."""
from pathlib import Path
import hashlib
import shutil
import subprocess
import sys

HERE=Path(__file__).resolve().parent
old,new=map(lambda s:Path(s).resolve(),sys.argv[1:])
new.mkdir(exist_ok=True,parents=True)
for n in ["stock","twin","errors","bits","stream","scatter"]:
    shutil.copyfile(old/(n+".rs"),new/(n+".rs"))
scatter=(old/"scatter.rs").read_text()
a=scatter.index("        // Each entry, including an already")
b=scatter.index("        h\n",a)
interval=scatter[:a]+(HERE/"extend.inc.rs").read_text()+scatter[b:]
(new/"interval.rs").write_text(interval)
driver=(old/"main.rs").read_text().replace(str(old),str(new)).replace("quad","interval")
flags=["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1"]
(new/"main.rs").write_text(driver)
subprocess.run([*flags,str(new/"main.rs"),"-o",str(new/"native")],check=True)
print("native",hashlib.sha256((new/"native").read_bytes()).hexdigest())
# A separate binary checks all 1,024 packed entries including unused prefixes,
# and records the number of dwelled dyadic intervals only on the canonical corpus.
test=new/"audit"
shutil.copytree(new,test,ignore=shutil.ignore_patterns("audit","target","native"),dirs_exist_ok=True)
driver=driver.replace(str(new),str(test))
driver=driver.replace("fn main(){",'''static TABLES:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static LEAVES:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
fn audit_table(lens:&[u8]) {
    assert!(scatter::block::full_table(lens)==interval::block::full_table(lens));
    TABLES.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
}
fn main(){''')
driver=driver.replace('if args[1]=="synthetic"', '''if args[1]=="tables" {
    let data=fs::read(&args[2]).unwrap();let r=stock_scan(&data,false);
    println!("blocks {} tables {} exact LUT entries {} decision intervals {}",r.1,TABLES.load(std::sync::atomic::Ordering::Relaxed),1024*TABLES.load(std::sync::atomic::Ordering::Relaxed),LEAVES.load(std::sync::atomic::Ordering::Relaxed));return;
}
if args[1]=="synthetic"''')
(test/"main.rs").write_text(driver)
for name in ["scatter","interval"]:
    f=test/(name+".rs")
    s=f.read_text()+'\npub fn full_table(lens:&[u8])->[u32;1024] { Huff::new(lens).lut }\n'
    if name=="interval":
        s=s.replace("        let mut w=0u32;","        let mut visits=0usize;\n        let mut w=0u32;")
        s=s.replace("        while w<1024 {","        while w<1024 {\n            visits+=1;")
        s=s.replace("        h\n","        crate::LEAVES.fetch_add(visits,std::sync::atomic::Ordering::Relaxed);\n        h\n",1)
    f.write_text(s)
f=test/"stock.rs"
f.write_text(f.read_text().replace("        tables.push(Huff::new(&lens[..alpha]));","        crate::audit_table(&lens[..alpha]);\n        tables.push(Huff::new(&lens[..alpha]));",1))
subprocess.run([*flags,str(test/"main.rs"),"-o",str(test/"native")],check=True)
print("audit",hashlib.sha256((test/"native").read_bytes()).hexdigest())
