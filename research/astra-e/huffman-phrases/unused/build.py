"""Elide precisely the unused small-block canonical/group LUT constructors."""
from pathlib import Path
import hashlib
import shutil
import subprocess
import sys

old,new=map(lambda s:Path(s).resolve(),sys.argv[1:])
new.mkdir(exist_ok=True,parents=True)
for n in ["stock","twin","errors","bits","stream","singletiny"]:
    shutil.copyfile(old/(n+".rs"),new/(n+".rs"))

for source,arm in [("stock","stockskip"),("singletiny","skipboth")]:
    s=(old/(source+".rs")).read_text()
    a=s.index("    let mut tables = Vec::with_capacity(n_groups);")
    b=s.index("    let eob = (alpha - 1) as u16;",a)
    region=s[a:b]
    region=region.replace("    let mut tables = Vec::with_capacity(n_groups);",'''    let all=(1u8<<n_groups)-1;
    let used=if n_sel<=64 { selectors.iter().fold(0u8,|mask,&g|mask|(1u8<<g)) } else { all };
    let mut compact=[0u8;MAX_GROUPS];
    let mut tables=Vec::with_capacity(used.count_ones() as usize);''')
    region=region.replace("    for _ in 0..n_groups {","    for g in 0..n_groups {",1)
    region=region.replace("        tables.push(",'''        if used & (1u8<<g)==0 {continue;}
        compact[g]=tables.len() as u8;
        tables.push(''',1)
    region+='''    if used!=all {
        for g in &mut selectors {*g=compact[*g as usize];}
    }
'''
    (new/(arm+".rs")).write_text(s[:a]+region+s[b:])
driver=(old/"main.rs").read_text().replace(str(old),str(new)).replace("pairtiny","skipboth").replace("scatter","stockskip")
# Preserve the preregistered order of the two control/treatment pairs.
driver=driver.replace('("skipboth",skipboth_scan),("singletiny",singletiny_scan)', '("singletiny",singletiny_scan),("skipboth",skipboth_scan)')
(new/"main.rs").write_text(driver)
subprocess.run(["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1",str(new/"main.rs"),"-o",str(new/"native")],check=True)
print("native",hashlib.sha256((new/"native").read_bytes()).hexdigest())
