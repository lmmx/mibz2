"""Generate the frozen small-block selector policies and an independent use census."""
from pathlib import Path
import hashlib
import shutil
import subprocess
import sys

old,new=map(lambda s:Path(s).resolve(),sys.argv[1:])
new.mkdir(exist_ok=True,parents=True)
for n in ["stock","twin","errors","bits","stream","scatter"]:
    shutil.copyfile(old/(n+".rs"),new/(n+".rs"))
scatter=(old/"scatter.rs").read_text()
for name in ["pairtiny","singletiny"]:
    s=scatter.replace("fn new(lens: &[u8]) -> Huff", "fn new<const SMALL:bool>(lens: &[u8]) -> Huff")
    needle="        tables.push(Huff::new(&lens[..alpha]));"
    assert s.count(needle)==1
    s=s.replace(needle,'''        tables.push(if n_sel<=64 {Huff::new::<true>(&lens[..alpha])} else {Huff::new::<false>(&lens[..alpha])});''')
    if name=="pairtiny":
        s=s.replace("if b!=eob && b<64", "if !SMALL && b!=eob && b<64")
    else:
        s=s.replace("        let eob = (alpha-1)", "        if SMALL {return h;}\n        let eob = (alpha-1)",1)
    (new/(name+".rs")).write_text(s)
driver=(old/"main.rs").read_text().replace(str(old),str(new))
decl=next(l for l in driver.splitlines() if l.startswith('pub mod quad'))
driver=driver.replace(decl,decl.replace('quad','pairtiny')+'\n'+decl.replace('quad','singletiny'))
driver=driver.replace("    assert_eq!(stock,one_item!(quad,data,start));","    assert_eq!(stock,one_item!(pairtiny,data,start));\n    assert_eq!(stock,one_item!(singletiny,data,start));")
driver=driver.replace("scanner!(quad_scan,quad);","scanner!(pairtiny_scan,pairtiny);scanner!(singletiny_scan,singletiny);")
driver=driver.replace('const ARMS:[(&str,Scan);4]','const ARMS:[(&str,Scan);5]').replace('("quad",quad_scan)','("pairtiny",pairtiny_scan),("singletiny",singletiny_scan)')
driver=driver.replace("pos in 0..4","pos in 0..5").replace("if(r/4)%2==0{pos}else{3-pos}","if(r/5)%2==0{pos}else{4-pos}").replace("ARMS[(r+j)%4]","ARMS[(r+j)%5]")
(new/"main.rs").write_text(driver)
flags=["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1"]
subprocess.run([*flags,str(new/"main.rs"),"-o",str(new/"native")],check=True)
print("native",hashlib.sha256((new/"native").read_bytes()).hexdigest())
audit=new/"audit"
shutil.copytree(new,audit,ignore=shutil.ignore_patterns("audit","target","native"),dirs_exist_ok=True)
driver=driver.replace(str(new),str(audit))
driver=driver.replace('if args[1]=="synthetic"', '''if args[1]=="usage" {
    println!("block,group,declared_selectors,tables,symbols,listed_uses,consumed_uses,emitted_symbols");
    let data=fs::read(&args[2]).unwrap();let r=stock_scan(&data,false);assert_eq!(r.1,431);return;
}
if args[1]=="synthetic"''')
(audit/"main.rs").write_text(driver)
f=audit/"stock.rs";s=f.read_text()
needle="    unsafe { syms.set_len(n) };"
assert s.count(needle)==1
s=s.replace(needle,needle+'''
    static ORDINAL:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
    let block=ORDINAL.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
    let mut used=[0;MAX_GROUPS];let mut listed=[0;MAX_GROUPS];let mut emitted=[0;MAX_GROUPS];
    for &g in &selectors {listed[g as usize]+=1;}
    for (&g,word) in selectors.iter().zip(syms.chunks(GROUP)) {used[g as usize]+=1;emitted[g as usize]+=word.len();}
    for g in 0..n_groups {println!("{block},{g},{n_sel},{n_groups},{n},{},{},{}",listed[g],used[g],emitted[g]);}
''')
f.write_text(s)
subprocess.run([*flags,str(audit/"main.rs"),"-o",str(audit/"native")],check=True)
print("audit",hashlib.sha256((audit/"native").read_bytes()).hexdigest())
