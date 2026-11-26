"""Four-arm complete comparison against the exact eager compact control."""
from pathlib import Path
import re,subprocess,sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here.parent/'harness.py'),str(out)],check=True)
price=(here.parents[2]/'f2c/price_impl.rs').read_text().replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
for mod,folder,file in [('control',here.parent/'construction','eager.rs'),('packed',here,'candidate.rs')]:
    s=(folder/file).read_text()
    def resolve(m):return m[0].replace(m[1],str((folder/m[1]).resolve()))
    s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
    impl='impl<const MODE:u8> Dispatch<MODE> {' if mod=='control' else 'impl<const MODE:u8,const WIDTH:u32> Dispatch<MODE,WIDTH> {'
    s+='\n'+price.replace('impl Candidate {',impl)
    s+='\n'+impl+' pub fn bytes(&self)->(usize,usize){(self.runs.capacity()*size_of::<Run>(),self.starts.capacity()*8'+('+self.page_first.capacity()*4' if mod=='control' else '')+')} }'
    (out/(mod+'.rs')).write_text(s)
s=(out/'focus.rs').read_text()
a=s.index('mod control;');b=s.index('#[repr(C)]',a)
s=s[:a]+'''mod control;
#[allow(dead_code)]
mod packed;

'''+s[b:]
s=s.replace('    let mut a=starts::Candidate::default();\n    let mut z=ends::Candidate::default();', '''    let mut p32=packed::Dispatch::<2,32>::default();
    let mut p40=packed::Dispatch::<2,40>::default();
    let mut p44=packed::Dispatch::<2,44>::default();''')
s=s.replace('control_runs,start_runs,end_runs,control_alloc,start_alloc,end_alloc','run_bytes,control_directory,packed32_directory,packed40_directory,packed44_directory')
s=s.replace('$arm.storage()', '$arm.bytes()')
a=s.index('            let (c0,b0)=check!(old);');b=s.index('            continue;',a)
s=s[:a]+'''            let (r0,b0)=check!(old);
            let (r1,b1)=check!(p32);
            let (r2,b2)=check!(p40);
            let (r3,b3)=check!(p44);
            assert_eq!((r0,r0,r0),(r1,r2,r3));
            println!("{index},{n},{},{r0},{b0},{b1},{b2},{b3}",r.records().len());
'''+s[b:]
s=s.replace('let mut outputs: [Vec<u8>;3]', 'let mut outputs: [Vec<u8>;4]')
s=s.replace('    let perms=[[0,1,2],[1,2,0],[0,2,1],[2,0,1],[1,0,2],[2,1,0]];', '''    let perms: [[usize;4];8]=std::array::from_fn(|r| std::array::from_fn(|j| {
        let base=[0usize,1,3,2]; (base[if r<4 {j} else {3-j}] + r%4)%4
    }));''')
s=s.replace('pass in 0..=60','pass in 0..=64').replace('perms[(pass+bi)%6]', 'perms[(pass+bi)%8]')
s=s.replace('1=>a.measure','1=>p32.measure').replace('_=>z.measure', '2=>p40.measure(&b.r,b.n,b.mark,out,true),\n                    _=>p44.measure')
s=s.replace('["control12","starts8","ends8"]','["split64","packed32","packed40","packed44"]')
(out/'focus.rs').write_text(s)
