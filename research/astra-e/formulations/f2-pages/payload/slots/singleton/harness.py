from pathlib import Path
import re,subprocess,sys
h=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(h.parent/'harness.py'),str(out)],check=True)
price=(h.parents[3]/'f2c/price_impl.rs').read_text().replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
for mod,file in [('control',h.parent/'candidate.rs'),('aos',h/'interleaved.rs'),('soa',h/'candidate.rs')]:
    s=file.read_text()
    def resolve(m):return m[0].replace(m[1],str((file.parent/m[1]).resolve()))
    s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
    impl='impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {' if mod=='control' else 'impl<const MODE:u8,const SCHEDULE:u8,const SINGLE:bool> Dispatch<MODE,SCHEDULE,SINGLE> {'
    s+='\n'+price.replace('impl Candidate {',impl)
    s+='\n'+impl+' pub fn bytes(&self)->(usize,usize){(self.runs.capacity()*size_of::<Run>(),'+('self.ends.capacity()*4' if mod=='soa' else '0')+')} }'
    (out/(mod+'.rs')).write_text(s)
s=(out/'focus.rs').read_text().replace('mod slots;', 'mod aos;\n#[allow(dead_code)]\nmod soa;')
s=s.replace('let mut old=control::Candidate::default();','let mut old=control::Dispatch::<2,2>::default();')
s=s.replace('slots::Dispatch::<2,1>','aos::Dispatch::<2,2,true>').replace('slots::Dispatch::<2,2>','soa::Dispatch::<2,2,false>').replace('slots::Dispatch::<2,3>','soa::Dispatch::<2,2,true>')
s=s.replace('assert_eq!((r0,r0,r0),(r1,r2,r3));','assert_eq!(r0,r1); assert_eq!((r0,r0),(r2+b2,r3+b3));')
s=s.replace('            assert_eq!((b0,b0,b0),(b1,b2,b3));\n            #[cfg(slot_counts)] p32.report(index);','')
s=s.replace('run_bytes,control_directory,fixed_directory,preload_directory,bulk_directory','control_run_bytes,control_cold,aos_cold,soa_cold,soa_singleton_cold')
s=s.replace('["mask","fixed","preload","bulk"]','["control","aos_single","soa_always","soa_single"]')
(out/'focus.rs').write_text(s)
