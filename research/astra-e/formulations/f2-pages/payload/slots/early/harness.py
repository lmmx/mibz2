from pathlib import Path
import re,subprocess,sys
h=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(h.parent/'harness.py'),str(out)],check=True)
price=(h.parents[3]/'f2c/price_impl.rs').read_text().replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
for mod,file,early in [('control',h.parent/'candidate.rs',False),('early',h/'candidate.rs',True)]:
    s=file.read_text()
    def resolve(m):return m[0].replace(m[1],str((file.parent/m[1]).resolve()))
    s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
    impl='impl<const MODE:u8,const SCHEDULE:u8,const EARLY:u8> Dispatch<MODE,SCHEDULE,EARLY> {' if early else 'impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {'
    s+='\n'+price.replace('impl Candidate {',impl)
    s+='\n'+impl+' pub fn bytes(&self)->(usize,usize){(self.runs.capacity()*size_of::<Run>(),self.starts.capacity()*8+self.page_first.capacity()*4)} }'
    (out/(mod+'.rs')).write_text(s)
s=(out/'focus.rs').read_text().replace('mod slots;', 'mod early;')
s=s.replace('let mut old=control::Candidate::default();','let mut old=control::Dispatch::<2,2>::default();')
for i in [1,2,3]:s=s.replace(f'slots::Dispatch::<2,{i}>',f'early::Dispatch::<2,2,{i}>')
s=s.replace('["mask","fixed","preload","bulk"]','["control","directory_hint","advance","advance_hint"]')
s=s.replace('fixed_directory,preload_directory,bulk_directory','hint_directory,early_directory,payload_hint_directory')
(out/'focus.rs').write_text(s)
