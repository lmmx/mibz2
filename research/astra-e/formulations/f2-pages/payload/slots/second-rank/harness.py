from pathlib import Path
import re
import subprocess
import sys

here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here.parent/'harness.py'),str(out)],check=True)
control=(out/'slots.rs').read_text()
(out/'control.rs').write_text(control)
price=(here.parents[3]/'f2c/price_impl.rs').read_text().replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
impl='impl<const MODE:u8,const SCHEDULE:u8,const SEC:u8> Dispatch<MODE,SCHEDULE,SEC> {'
family=(here/'family.rs').read_text().replace('"second.rs"',f'"{here/"second.rs"}"')
family += '\n'+price.replace('impl Candidate {',impl)
family += '\n'+impl+ 'pub fn bytes(&self)->(usize,usize){(self.runs.capacity()*size_of::<Run>(),self.starts.capacity()*8 + self.page_first.capacity()*4)} }'
(out/'slots.rs').write_text(family)
s=(out/'focus.rs').read_text().replace('control::Candidate', 'control::Dispatch::<2,2>')
s=re.sub(r'slots::Dispatch::<2,([123])>', lambda m:'slots::Dispatch::<2,2,'+str(int(m[1])-1)+'>',s)
s=s.replace('#[cfg(slot_counts)] p32.report(index);','')
s=s.replace('["mask","fixed","preload","bulk"]','["control","generic","guarded","fused"]')
s=s.replace('control_directory,fixed_directory,preload_directory,bulk_directory','control_directory,generic_directory,guarded_directory,fused_directory')
(out/'focus.rs').write_text(s)
