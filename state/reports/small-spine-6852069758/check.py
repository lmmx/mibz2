import importlib.util, json, re, hashlib, shutil
from pathlib import Path
p=Path(__file__).parent
spec=importlib.util.spec_from_file_location('coord',p/'coord.py')
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
data=json.loads((p/'audit-input.json').read_text())
c.comments=lambda n:data[str(n)]
r={'audit':c.audit(c.CUTOVER),'route':c.route('coordinator','REVIEW')}
(p/'routing.json').write_text(json.dumps(r,indent=2)+'\n')
b=(p/'base.rs').read_text(); h=(p/'head.rs').read_text()
def chunk(s,start,end): return s[s.index(start):s.index(end,s.index(start))]
def normalize(s): return re.sub(r'\s+','',s)
regions={
 'position_reset':('if self.pos.capacity() < nr {','let pos = self.pos.as_mut_ptr();'),
 'short_emit_seed_marked_byte':('// the marked rules, from the word down to the row','// where each rule first lies:'),
 'descending_and_ascending':('// where each rule first lies:','// the marked rules again:'),
 'repetition_truncation':('// a cycle shorter than the block:','#[cfg(test)]\nmod tests'),
}
result={}
for name,(start,end) in regions.items():
 x,y=chunk(b,start,end),chunk(h,start,end)
 if name=='position_reset': y=y.rsplit('}',1)[0]
 if name=='short_emit_seed_marked_byte': y=y[:y.index('// With no long spine seed')]
 result[name]=normalize(x)==normalize(y)
start='// the marked rules again:'; end='// a cycle shorter than the block:'
x,y=chunk(b,start,end),chunk(h,start,end)
# Strip only the extra conditional's final closing brace, retaining loop body.
result['F01_recopy']=normalize(x)==normalize(y)[:-1]
result['production_before_writer']=normalize(b[:b.index('fn write_cells(')].replace('    all_small: bool,\n','').replace('    largest_plain: u32,\n',''))==normalize(h[:h.index('fn write_cells(')].replace('    all_small: bool,\n','').replace('    largest_plain: u32,\n',''))
result['tools']={t:shutil.which(t) for t in ['rustc','cargo','just']}
result['sha256']={n:hashlib.sha256((p/n).read_bytes()).hexdigest() for n in ['base.rs','head.rs','coord.py']}
(p/'source-check.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'source':result,'audit_findings':len(r['audit']),'route':r['route']},indent=2))
assert all(v for k,v in result.items() if isinstance(v,bool))
