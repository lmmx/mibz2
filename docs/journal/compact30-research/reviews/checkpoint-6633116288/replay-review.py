import json, re, hashlib
from pathlib import Path
d=json.loads(Path('audit-input.json').read_text()); e=d['evidence']
def get(suffix):
    found=[v for k,v in e.items() if k.endswith(suffix)]
    assert len(found)==1, suffix
    return found[0]
def classes(text):
    rows=[]
    for n,a,b,delta in re.findall(r'count\s+([\d,]+):\s+([\d,]+) ->\s+([\d,]+) instructions\s+(-?[\d,]+)',text):
        n,a,b,delta=[int(x.replace(',','')) for x in (n,a,b,delta)]
        assert n*(b-a)==delta
        rows.append((n,a,b,delta))
    return rows
dr=classes(d['descriptorClasses']); sr=classes(get('scanner-u1u2-trial/callgrind/classes-run1.txt'))
assert sum(x[3] for x in dr)==-58209045
assert sum(x[3] for x in sr)==-84060421
dlow=2*38294883-20487195-1173383-378882
slow=88402421-4359203
assert dlow==54550306 and slow==84043218
ops=[(int(a,16),v) for a,v in re.findall(r'^\s+([0-9a-f]+):\s+(.+)$',d['descriptorRefill'],re.M)]
trace=[(a,v) for a,v in ops if a<=0x94442 and a not in (0x943f4,0x943f9,0x943fd)]
assert len(trace)==69
lib=lambda s: re.sub(r'^0x[0-9a-f]+\s+', '',s,flags=re.M)
assert lib(get('scanner-u1u2-trial/callgrind/only-su-run1-library-sites.txt'))==lib(get('scanner-u1u2-trial/callgrind/only-v2-run1-library-sites.txt'))
for t in d['trees']:
    by={x['path']:x['sha'] for x in t['tree']['tree']}
    assert by['examples/c30walk/ctor2.rs']=='168c2bac9ad0d8da4d9f82788f009d32f419608e'
    assert by['examples/c30walk/ctor3.rs']=='01f91419904a605b721dc1742ba8087fe79f74cf'
out={'descriptor_own_saving':58209045,'descriptor_interval':[dlow,62277795],'u2_interval':[slow,84060421],'descriptor_own_writes_delta':-193756+5197912,'u2_own_writes_delta':3391432,'descriptor_wrapper_trace':[(hex(a),v) for a,v in trace],'C6_incremental_conditional_before_H':(69-40)*1084913-5*(1795482-1084913),'candidate_shared_negative_classes':sorted(set((n,a) for n,a,b,v in dr if v<0)&set((n,a) for n,a,b,v in sr if v<0)),'limitations':['Raw compressed profiles not replayed in this run; tables and listings replayed.','Equal execution-count classes do not prove identical eliminated instructions.','Missing structural counters remain a charter 5.3b method finding.']}
Path('replay-results.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({k:v for k,v in out.items() if k!='descriptor_wrapper_trace'},indent=2))
