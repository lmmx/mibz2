from pathlib import Path
import re,subprocess,sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here.parent/'harness.py'),str(out)],check=True)
price=(here.parents[2]/'f2c/price_impl.rs').read_text().replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
for mod,file,impl in [('control',here.parent/'construction/eager.rs','impl<const MODE:u8> Dispatch<MODE> {'),('uniform',here/'candidate.rs','impl<const MODE:u8,const PAGE_ID:bool> Dispatch<MODE,PAGE_ID> {')]:
    s=file.read_text()
    def resolve(m):return m[0].replace(m[1],str((file.parent/m[1]).resolve()))
    s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
    s+='\n'+price.replace('impl Candidate {',impl)
    s+='\n'+impl+' pub fn storage(&self)->(usize,usize){(self.runs.capacity()*size_of::<Run>(),self.starts.capacity()*8+self.page_first.capacity()*4)} }'
    (out/(mod+'.rs')).write_text(s)
s=(out/'focus.rs').read_text()
s=s.replace('mod starts;\n#[allow(dead_code)]\nmod ends;', 'mod uniform;')
s=s.replace('starts::Candidate::default()', 'uniform::Dispatch::<2,false>::default()').replace('ends::Candidate::default()', 'uniform::Dispatch::<2,true>::default()')
s=s.replace('control_runs,start_runs,end_runs,control_alloc,start_alloc,end_alloc','run_bytes,directory_bytes')
s=s.replace('assert_eq!(c0,c2+1);','assert_eq!((c0,b0,b0),(c2,b1,b2));')
s=s.replace('{c0},{c1},{c2},{b0},{b1},{b2}', '{c0},{b0}')
s=s.replace('["control12","starts8","ends8"]','["control","mask_first","base_first"]')
(out/'focus.rs').write_text(s)
