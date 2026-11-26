"""Create a separately instrumented table-build attribution binary from the original experiment."""
from pathlib import Path
import hashlib
import shutil
import subprocess
import sys

old,new=map(lambda s:Path(s).resolve(),sys.argv[1:])
new.mkdir(exist_ok=True,parents=True)
for name in ("bits","errors","stream"):
    shutil.copyfile(old/(name+".rs"),new/(name+".rs"))
driver_source=(old/"main.rs").read_text()
arms=("stock","twin","scatter","interval") if "pub mod interval" in driver_source else ("stock","twin","triple","quad")
for name in arms:
    source=(old/(name+".rs")).read_text()
    needle="        tables.push(Huff::new(&lens[..alpha]));"
    assert source.count(needle)==1
    priced="        let stamp=crate::thread_ns();\n"+needle+"\n        let delta=crate::thread_ns()-stamp;\n        crate::TABLE_TIME.with(|c|c.set(c.get()+delta));"
    (new/(name+".rs")).write_text(source.replace(needle,priced))
driver=(old/"main.rs").read_text().replace(str(old),str(new))
driver=driver.replace("fn thread_ns()->u64 {", "thread_local!{static TABLE_TIME:std::cell::Cell<u64>=const{std::cell::Cell::new(0)};}\nfn thread_ns()->u64 {")
driver=driver.replace("            let start=Instant::now();","            TABLE_TIME.with(|c|c.set(0));\n            let start=Instant::now();")
driver=driver.replace("metadata_fold\");", "metadata_fold,table_ns\");")
driver=driver.replace('let ticks=thread_ns()-cpu;','let ticks=thread_ns()-cpu;let table_ns=TABLE_TIME.with(|c|c.get());')
driver=driver.replace('{:016x}\",result.1', '{:016x},{table_ns}\",result.1')
(new/"main.rs").write_text(driver)
subprocess.run(["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1",str(new/"main.rs"),"-o",str(new/"native")],check=True)
print(hashlib.sha256((new/"native").read_bytes()).hexdigest())
