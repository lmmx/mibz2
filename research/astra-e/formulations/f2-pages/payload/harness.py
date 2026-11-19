"""Compare the original 12-byte runs with the two compact layouts."""
from pathlib import Path
import re, subprocess, sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
out.mkdir(parents=True,exist_ok=True)
price=(here.parents[1]/'f2c/price_impl.rs').read_text()
price=price.replace('impl Candidate {','impl<const MODE:u8> Dispatch<MODE> {').replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
for name,file in [('control',here.parent/'instructions/candidate.rs'),('starts',here/'starts.rs'),('ends',here/'candidate.rs')]:
    s=file.read_text()
    def resolve(m):return m[0].replace(m[1],str((file.parent/m[1]).resolve()))
    s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
    (out/(name+'.rs')).write_text(s+'\n'+price+'''
impl<const MODE:u8> Dispatch<MODE> {
    pub fn storage(&self)->(usize,usize) {
        (self.runs.len(), self.runs.capacity()*size_of::<Run>())
    }
}
''')
s=(here.parent/'instructions/focus/focus.rs').read_text()
a=s.index('#[allow(dead_code)]\nmod counted;');b=s.index('#[repr(C)]',a)
s=s[:a]+'''#[allow(dead_code)]
mod control;
#[allow(dead_code)]
mod starts;
#[allow(dead_code)]
mod ends;

'''+s[b:]
s=s.replace('    let compressed=', '''    let full=std::env::args().nth(2).unwrap()=="geometry";
    let mut old=control::Candidate::default();
    let mut a=starts::Candidate::default();
    let mut z=ends::Candidate::default();
    let mut tmp=Vec::new();
    let mut accepted=0;
    if full {println!("block,rows,records,control_runs,start_runs,end_runs,control_alloc,start_alloc,end_alloc");}
    let compressed=''')
s=s.replace('if !select || r.records()', 'if (!select && !full) || r.records()')
s=s.replace('        blocks.push(Block', '''        if full {
            accepted+=1;
            macro_rules! check { ($arm:ident) => {{
                assert!($arm.invert_runs(&r,n,symbols.orig_ptr,&mut tmp,kn));
                assert_eq!(tmp,want);
                $arm.storage()
            }}; }
            let (c0,b0)=check!(old);
            let (c1,b1)=check!(a);
            let (c2,b2)=check!(z);
            assert_eq!(c0,c1);
            assert_eq!(c0,c2+1);
            println!("{index},{n},{},{c0},{c1},{c2},{b0},{b1},{b2}",r.records().len());
            continue;
        }
        blocks.push(Block''')
s=s.replace('    assert_eq!(blocks.len(),9);','    if full {assert_eq!(accepted,228);return;}\n    assert_eq!(blocks.len(),9);')
a=s.index('    let mut frozen=');b=s.index('    let mut outputs:',a)
s=s[:a]+s[b:]
s=s.replace('0=>frozen.measure','0=>old.measure').replace('1=>generic.matched','1=>a.measure').replace('_=>hardware.matched','_=>z.measure').replace('b.mark,out,false)','b.mark,out,true)')
s=s.replace('["counted","generic","hardware"]','["control12","starts8","ends8"]')
(out/'focus.rs').write_text(s)
