"""Compile the declared scatter follow-up from the exact generated four-phrase module."""
from pathlib import Path
import hashlib
import shutil
import subprocess
import sys

HERE=Path(__file__).resolve().parent
old,new=map(lambda s:Path(s).resolve(),sys.argv[1:])
new.mkdir(exist_ok=True,parents=True)
for n in ["stock","twin","quad","errors","bits","stream"]:
    shutil.copyfile(old/(n+".rs"),new/(n+".rs"))
quad=(old/"quad.rs").read_text()
assert quad.count("pub fn read_item(")==1
a=quad.index("pub fn read_item(")
b=quad.index("    macro_rules! fail",a)
signature=quad[a:b]
wrapped=signature+'''    // This standalone harness checks BMI2 at entry.
    unsafe { read_bmi2(br,level,syms) }
}
#[target_feature(enable="bmi2")]
'''+signature.replace("pub fn read_item(","unsafe fn read_bmi2(")
quad=quad[:a]+wrapped+quad[b:]
(new/"quad.rs").write_text(quad)
start=quad.index("                    out[n].write(((e>>FIRST)")
end=quad.index("                    n+=count;",start)
store=(HERE/"store.inc.rs").read_text()
(new/"scatter.rs").write_text(quad[:start]+store+quad[end:])
driver=(old/"main.rs").read_text().replace(str(old),str(new)).replace("triple","scatter")
driver=driver.replace("fn main(){",'fn main(){\n    assert!(std::is_x86_feature_detected!("bmi2"));')
(new/"main.rs").write_text(driver)
subprocess.run(["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1",str(new/"main.rs"),"-o",str(new/"native")],check=True)
print(hashlib.sha256((new/"native").read_bytes()).hexdigest())
