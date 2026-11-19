from pathlib import Path
import hashlib
import shutil
import subprocess
import sys

old,new=map(lambda s:Path(s).resolve(),sys.argv[1:])
here=Path(__file__).resolve().parent
new.mkdir(exist_ok=True,parents=True)
for n in ["stock","twin","errors","bits","stream","scatter"]:
    shutil.copyfile(old/(n+".rs"),new/(n+".rs"))
scatter=(old/"scatter.rs").read_text()
a=scatter.index("        while left > 0 {")
b=scatter.index("        if r.overran(data)",a)
(new/"tail.rs").write_text(scatter[:a]+(here/"body.inc.rs").read_text()+scatter[b:])
driver=(old/"main.rs").read_text().replace(str(old),str(new)).replace("quad","tail")
(new/"main.rs").write_text(driver)
flags=["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1"]
subprocess.run([*flags,str(new/"main.rs"),"-o",str(new/"native")],check=True)
print("native",hashlib.sha256((new/"native").read_bytes()).hexdigest())
audit=new/"audit"
shutil.copytree(new,audit,ignore=shutil.ignore_patterns("audit","target","native"),dirs_exist_ok=True)
driver=driver.replace(str(new),str(audit))
driver=driver.replace("fn main(){",'''static CALLS:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
static MULTI:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
fn main(){''')
driver=driver.replace('if args[1]=="synthetic"', '''if args[1]=="count" {
    let data=fs::read(&args[2]).unwrap();let r=tail_scan(&data,false);
    println!("blocks {} symbols {} tail lookups {} accepted multi-symbol tails {}",r.1,r.2,CALLS.load(std::sync::atomic::Ordering::Relaxed),MULTI.load(std::sync::atomic::Ordering::Relaxed));return;
}
if args[1]=="synthetic"''')
(audit/"main.rs").write_text(driver)
f=audit/"tail.rs";s=f.read_text()
s=s.replace("        while left > 0 {","        while left > 0 {\n            crate::CALLS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);")
s=s.replace("            r.skip_short(entry&15);","            if count>=2 {crate::MULTI.fetch_add(1,std::sync::atomic::Ordering::Relaxed);}\n            r.skip_short(entry&15);")
f.write_text(s)
subprocess.run([*flags,str(audit/"main.rs"),"-o",str(audit/"native")],check=True)
print("audit",hashlib.sha256((audit/"native").read_bytes()).hexdigest())
