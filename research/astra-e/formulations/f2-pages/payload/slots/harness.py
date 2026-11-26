from pathlib import Path
import re,subprocess,sys
h=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(h.parent/'packed/harness.py'),str(out)],check=True)
price=(h.parents[2]/'f2c/price_impl.rs').read_text().replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
s=(h/'candidate.rs').read_text()
def resolve(m):return m[0].replace(m[1],str((h/m[1]).resolve()))
s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
s+='\n'+price.replace('impl Candidate {','impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {')
s+='\nimpl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {pub fn bytes(&self)->(usize,usize){(self.runs.capacity()*size_of::<Run>(),self.starts.capacity()*8+self.page_first.capacity()*4)} }'
(out/'slots.rs').write_text(s)
s=(out/'focus.rs').read_text().replace('mod packed;', 'mod slots;').replace('packed::Dispatch::<2,32>','slots::Dispatch::<2,1>').replace('packed::Dispatch::<2,40>','slots::Dispatch::<2,2>').replace('packed::Dispatch::<2,44>','slots::Dispatch::<2,3>')
s=s.replace('            assert_eq!((r0,r0,r0),(r1,r2,r3));','''            assert_eq!((r0,r0,r0),(r1,r2,r3));
            assert_eq!((b0,b0,b0),(b1,b2,b3));
            #[cfg(slot_counts)] p32.report(index);''')
s=s.replace('["split64","packed32","packed40","packed44"]','["mask","fixed","preload","bulk"]')
s=s.replace('packed32_directory,packed40_directory,packed44_directory','fixed_directory,preload_directory,bulk_directory')
(out/'focus.rs').write_text(s)
