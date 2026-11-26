from pathlib import Path
import re
import subprocess
import sys

here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here.parent/'harness.py'),str(out)],check=True)
control=(out/'slots.rs').read_text().replace('self.starts.capacity()*8+self.page_first.capacity()*4','0')
(out/'control.rs').write_text(control)
price=(here.parents[3]/'f2c/price_impl.rs').read_text().replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
impl='impl<const MODE:u8,const SCHEDULE:u8,const CACHE:u8> Dispatch<MODE,SCHEDULE,CACHE> {'
family=(here/'family.rs').read_text().replace('"../../../../f2c/second.rs"',f'"{here.parents[3]/"f2c/second.rs"}"')
family += '\n'+price.replace('impl Candidate {',impl)
family += '\n'+impl+ 'pub fn bytes(&self)->(usize,usize){(self.runs.capacity()*size_of::<Run>(),self.next_target.capacity()*4)} }'
family+='\n#[cfg(prediction_counts)]\n'+impl+'pub fn totals(&self)->[u64;3]{self.counters}}'
(out/'slots.rs').write_text(family)
s=(out/'focus.rs').read_text().replace('control::Candidate', 'control::Dispatch::<2,2>')
s=re.sub(r'slots::Dispatch::<2,([123])>', lambda m:'slots::Dispatch::<2,2,'+m[1]+'>',s)
s=s.replace('#[cfg(slot_counts)] p32.report(index);','')
s=s.replace('["mask","fixed","preload","bulk"]','["control","packed","side","embedded"]')
s=s.replace('control_directory,fixed_directory,preload_directory,bulk_directory','unused_control,unused_packed,side_bytes,unused_embedded')
s=s.replace('assert_eq!((b0,b0,b0),(b1,b2,b3));','assert_eq!((b0,b1,b3),(0,0,0)); assert!(b2>0);')
anchor='            let (r3,b3)=check!(p44);'
s=s.replace(anchor,anchor+'''
            #[cfg(prediction_counts)]
            for (name,t) in [("side",p40.totals()),("embedded",p44.totals())] {
                eprintln!("{index},{name},{},{},{}",t[0],t[1],t[2]);
            }''')
(out/'focus.rs').write_text(s)
