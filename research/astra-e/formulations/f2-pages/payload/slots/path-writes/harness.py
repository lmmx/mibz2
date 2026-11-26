from pathlib import Path
import re
import subprocess
import sys

here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here.parent/'harness.py'),str(out)],check=True)
control=(out/'slots.rs').read_text()
control=control.replace('self.starts.capacity()*8+self.page_first.capacity()*4','self.lane_paths.iter().map(|p|p.capacity()).sum()')
control+='\n'+'impl<const A:u8,const B:u8> Dispatch<A,B> {pub fn capacities(&self)->[usize;8]{std::array::from_fn(|i|self.lane_paths[i].capacity())}}'
(out/'control.rs').write_text(control)
price=(here.parents[3]/'f2c/price_impl.rs').read_text().replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
impl='impl<const MODE:u8,const SCHEDULE:u8,const PATH:u8> Dispatch<MODE,SCHEDULE,PATH> {'
family=(here/'family.rs').read_text().replace('"../../../../f2c/second.rs"',f'"{here.parents[3]/"f2c/second.rs"}"')
family += '\n'+price.replace('impl Candidate {',impl)
family += '\n'+impl+ 'pub fn bytes(&self)->(usize,usize){(self.runs.capacity()*size_of::<Run>(),self.lane_paths.iter().map(|p|p.capacity()).sum())} }'
family += '\n'+impl+'pub fn capacities(&self)->[usize;8]{std::array::from_fn(|i|self.lane_paths[i].capacity())}}'
(out/'slots.rs').write_text(family)
s=(out/'focus.rs').read_text().replace('control::Candidate', 'control::Dispatch::<2,2>')
s=re.sub(r'slots::Dispatch::<2,([123])>', lambda m:'slots::Dispatch::<2,2,'+m[1]+'>',s)
s=s.replace('#[cfg(slot_counts)] p32.report(index);','')
s=s.replace('["mask","fixed","preload","bulk"]','["control","frame","slack","cursor"]')
s=s.replace('control_directory,fixed_directory,preload_directory,bulk_directory','control_paths,frame_paths,slack_paths,cursor_paths')
anchor='            let (r3,b3)=check!(p44);'
assert anchor in s
s=s.replace(anchor,anchor+'''
            let cap=old.capacities();
            assert_eq!([cap;3],[p32.capacities(),p40.capacities(),p44.capacities()]);''')
(out/'focus.rs').write_text(s)
