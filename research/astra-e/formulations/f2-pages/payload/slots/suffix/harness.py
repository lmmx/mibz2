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
impl='impl<const MODE:u8,const SCHEDULE:u8,const SPELL:u8> Dispatch<MODE,SCHEDULE,SPELL> {'
family=(here/'family.rs').read_text().replace('"second.rs"',f'"{here/"second.rs"}"')
family += '\n'+price.replace('impl Candidate {',impl)
family += '\n'+impl+ 'pub fn bytes(&self)->(usize,usize){(self.runs.capacity()*size_of::<Run>(),self.starts.capacity()*8 + self.page_first.capacity()*4)} }'
family += '''
#[cfg(spell_counts)]
'''+impl+'''
pub fn take_counts(&mut self) -> [u64;8] {
    let [a,b,c,d]=std::mem::take(&mut self.copied);
    let [e,f,g,h]=std::mem::take(&mut self.second.copied);
    [a,b,c,d,e,f,g,h]
}}'''
(out/'slots.rs').write_text(family)
s=(out/'focus.rs').read_text().replace('control::Candidate', 'control::Dispatch::<2,2>')
s=re.sub(r'slots::Dispatch::<2,([123])>', lambda m:'slots::Dispatch::<2,2,'+str(int(m[1]))+'>',s)
s=s.replace('#[cfg(slot_counts)] p32.report(index);','')
s=s.replace('["mask","fixed","preload","bulk"]','["control","first","second","both"]')
s=s.replace('control_directory,fixed_directory,preload_directory,bulk_directory','control_directory,first_directory,second_directory,both_directory')
anchor='            let (r3,b3)=check!(p44);'
assert anchor in s
s=s.replace(anchor,anchor+'''
            #[cfg(spell_counts)]
            for (name,counts) in [("first",p32.take_counts()),("second",p40.take_counts()),("both",p44.take_counts())] {
                eprintln!("{index},{name},{},{},{},{},{},{},{},{}",counts[0],counts[1],counts[2],counts[3],counts[4],counts[5],counts[6],counts[7]);
            }''')
(out/'focus.rs').write_text(s)
