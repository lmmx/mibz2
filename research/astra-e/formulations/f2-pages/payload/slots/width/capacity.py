"""Reuse F2's existing storage observer on the five actual scheduling widths."""
import ast
from pathlib import Path
import subprocess,sys
h=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(h/'harness.py'),str(out)],check=True)
forms=h.parents[3]
account=forms/'f2-scheduler/storage/account.py'
templates=[n.value for n in ast.walk(ast.parse(account.read_text())) if isinstance(n,ast.Constant) and isinstance(n.value,str)]
second=next(v for v in templates if 'pub(super) fn capacities' in v).replace('LANE','0')
first=next(v for v in templates if 'pub fn audit' in v)
expression=first[first.index('        let first='):first.index('        let (second,sp)')]
(out/'second.rs').write_text((forms/'f2c/second.rs').read_text()+second)
s=(out/'widths.rs').read_text().replace(str(forms/'f2c/second.rs'),str(out/'second.rs'))
s+='''\nimpl<const MODE:u8,const SCHEDULE:u8,const LANES:usize> Dispatch<MODE,SCHEDULE,LANES> {
    pub fn capacity_total(&self)->(usize,usize,usize) {
'''+expression+'''
        (first,self.path_bytes(),self.second.capacities().0)
    }
}
'''
(out/'widths.rs').write_text(s)
s=(out/'focus.rs').read_text().replace('block,rows,arm,path_capacity_bytes,waves,queries,full_waves','block,rows,arm,first_vectors,private_path_bytes,second_vectors,total')
a=s.index('                #[cfg(slot_counts)]');b=s.index('\n            }};',a)
s=s[:a]+'''                let (first,paths,second)=$arm.capacity_total();
                println!("{index},{n},{},{first},{paths},{second},{}",$name,first+paths+second);'''+s[b:]
(out/'capacity.rs').write_text(s)
