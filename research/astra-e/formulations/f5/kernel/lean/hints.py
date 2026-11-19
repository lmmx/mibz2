"""Repeat the corpus geometry pass with both worker-local action capacities recorded."""
from pathlib import Path
import subprocess
import sys
here=Path(__file__).resolve().parent
work=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here/"price.py"),str(work)],check=True)
with (work/"lean.rs").open("a") as f:
    f.write("\nimpl Book { pub(super) fn capacities(&self)->(usize,usize) {(self.hints.len(),self.hints.capacity())} }\n")
p=work/"main.rs"
s=p.read_text()
s=s[:s.index("        let z = book.sizes();")]+'''        let (words, hint_cap)=hints.capacities();
        let z=book.sizes();
        println!("{index},{words},{hint_cap},{},{}",z[4],z[3]);
    }
    assert_eq!(large,233);
    assert_eq!(verified,205197794);
}
'''
a=s.index("    let mut phase_file =")
b=s.index("    while let Ok",a)
s=s[:a]+'''    println!("stream,words,lean_bytes,full_action_bytes,full_owned_capacity");
'''+s[b:]
p.write_text(s)
