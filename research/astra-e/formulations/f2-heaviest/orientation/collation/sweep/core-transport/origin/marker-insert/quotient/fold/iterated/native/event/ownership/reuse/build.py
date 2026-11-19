from pathlib import Path
from hashlib import sha256
import json,subprocess,sys
here=Path(__file__).resolve().parent;out=Path(sys.argv[1]).resolve();deps=Path(sys.argv[2]).resolve()
subprocess.run([sys.executable,'-B',here.parent/'build.py',out,deps,'--sources-only'],check=True)
checks=json.loads((here.parent/'manifest.json').read_text())['sha256']
p=out/'control.rs';s=p.read_text()
assert sha256(s.encode()).hexdigest()==next(v for k,v in checks.items() if k.endswith('/control.rs'))
def replace(a,b):
    global s
    assert s.count(a)==1,a;s=s.replace(a,b)
replace('pub bytes:usize,','pub bytes:usize,pub borrowed:usize,')
replace('const AUDIT:bool,const USE:bool,const M:u8','const AUDIT:bool,const USE:bool,const REUSE:bool,const M:u8')
replace('self.replay::<USE,AUDIT>(&w.dict,out,&mut g.m)','self.replay::<USE,AUDIT,REUSE>(&w.dict,out,&mut g.m)')
replace('pub fn replay<const USE:bool,const AUDIT:bool>','pub fn replay<const USE:bool,const AUDIT:bool,const REUSE:bool>')
replace('self.owners.offsets.clear();self.owners.offsets.resize(self.grammar.dag.len()-leaves as usize,NONE);','''let slots=self.grammar.dag.len()-leaves as usize;
            if REUSE {
                assert!(self.prev.capacity()>=slots);
                std::mem::swap(&mut self.prev,&mut self.owners.offsets);
            }
            self.owners.offsets.clear();self.owners.offsets.resize(slots,NONE);''')
replace('counts.slots=self.owners.offsets.len() as u32;counts.bytes=self.owner_capacity();','''counts.slots=self.owners.offsets.len() as u32;
            if REUSE{
                counts.borrowed=self.owner_capacity();
                std::mem::swap(&mut self.prev,&mut self.owners.offsets);
            }else{counts.bytes=self.owner_capacity();}''')
replace('self.replay::<USE,AUDIT>(&w.dict,out,m)','self.replay::<USE,AUDIT,false>(&w.dict,out,m)')
replace('e.replay::<true,true>(bytes,','e.replay::<true,true,false>(bytes,')
replace('e.replay::<false,true>(bytes,','e.replay::<false,true,false>(bytes,')
p.write_text(s)
p=out/'main.rs';s=p.read_text()
assert sha256(s.encode()).hexdigest()==next(v for k,v in checks.items() if k.endswith('/main.rs'))
s=s.replace('["named","indexed","dense","owner"]','["dense","side_a","side_b","reuse"]')
a=s.index('                0=>{let t=now();');b=s.index('                _=>unreachable!(),\n            }',a)
s=s[:a]+'''                0..=3=>{let e=match arm {
                    0=>self.event.finish::<true,AUDIT,false,false,2,2>(&mut self.named,mark,m,x.n,x.p,&mut self.out),
                    1|2=>self.event.finish::<true,AUDIT,true,false,2,2>(&mut self.named,mark,m,x.n,x.p,&mut self.out),
                    3=>self.event.finish::<true,AUDIT,true,true,2,2>(&mut self.named,mark,m,x.n,x.p,&mut self.out),
                    _=>unreachable!(),
                };r.index=e.index_ns;r.contract=e.event_ns;r.emit=e.output_ns;r.e=e;},
'''+s[b:]
s=s.replace('leaf_bytes,visits");','leaf_bytes,visits,borrowed_bytes");').replace('o.leaf_bytes,o.visits);','o.leaf_bytes,o.visits,o.borrowed);')
a=s.index('println!("{},{},',s.index('fn census'));b=s.index('",x.index',a);s=s[:b]+',{}'+s[b:]
p.write_text(s)
subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports','-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',p,'--extern',f'mibz2={deps}/libmibz2.rlib','-L',f'dependency={deps}','-o',out/'reuse'],check=True)
