from pathlib import Path
import hashlib,subprocess,sys
here=Path(__file__).resolve().parent;out=Path(sys.argv[1]).resolve();deps=Path(sys.argv[2]).resolve()
out.mkdir(parents=True,exist_ok=True)
subprocess.run([sys.executable,'-B',here.parents[3]/'native/generate.py',out],check=True)
p=out/'control.rs'
assert hashlib.sha256(p.read_bytes()).hexdigest()=='01663fb4fa08e977bf775b925565e7f71a1faf3f822f02d9122d1278fc7529ab'
root=Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
def frozen(rev,path):return subprocess.check_output(['git','-C',root,'show',f'{rev}:{path.relative_to(root)}'],text=True)
g=frozen('a34c81b',here.parents[1]/'grammar.rs')
s=frozen('df54544',here.parent/'event.rs')
def one(old,new):
    global s
    assert s.count(old)==1,old;s=s.replace(old,new)
one('pending:std::collections::VecDeque<u32>, grammar:Grammar,','pending:std::collections::VecDeque<u32>, grammar:Grammar, owners:Owners, emit_root:Option<(u32,u32,u32,u32)>,')
one('pub m:Meter, pub edits:u32,','pub m:Meter,pub ownership:OwnerCounts, pub edits:u32,')
one('+self.pending.capacity()*4+self.grammar.capacity()','+self.pending.capacity()*4+self.grammar.capacity()+self.owner_capacity()')
one('pub fn finish<const FOLD:bool,const AUDIT:bool,const M:u8,const S:u8>','pub fn finish<const FOLD:bool,const AUDIT:bool,const USE:bool,const M:u8,const S:u8>')
one('let mut g=EventMeter{rows:len,..Default::default()};','self.emit_root=None;let mut g=EventMeter{rows:len,..Default::default()};')
i=s.index('            let size=self.grammar.dag[id as usize].size;assert!(skip<size')
j=s.index('        }else{w.finish(mark,len,n,p,out);g.m.cold=2;}',i)
s=s[:i]+'''            self.emit_root=Some((id,skip,n,w.words.len() as u32));
            g.ownership=self.replay::<USE,AUDIT>(&w.dict,out,&mut g.m);
'''+s[j:]
p.write_text(p.read_text()+'\n'+g+'\n'+s+'\n'+(here/'owner.rs').read_text())
p=out/'main.rs';p.write_text(p.read_text().split('struct Origins {')[0]+(here/'main.rs').read_text())
if '--sources-only' not in sys.argv[3:]:
    subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports','-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',p,'--extern',f'mibz2={deps}/libmibz2.rlib','-L',f'dependency={deps}','-o',out/'owner'],check=True)
