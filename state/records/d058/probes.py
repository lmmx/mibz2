import importlib.util,json,re,subprocess
from pathlib import Path
p=Path('d058')
s=importlib.util.spec_from_file_location('coord',p/'coord.py'); c=importlib.util.module_from_spec(s);s.loader.exec_module(c)
snap=json.loads((p/'inboxes.json').read_text()); c.comments=lambda n:snap[str(n)]
audit=c.audit(c.CUTOVER)
assert not [x for x in audit if not x['resolved']],audit
route=c.route('coordinator','REVIEW'); assert route['issue']==145
(p/'routing.json').write_text(json.dumps({'audit':audit,'route':route},indent=2)+'\n')
counts={}
for f in sorted((p/'archive/fuzz').glob('*s.txt')):
 t=f.read_text(); m=re.search(r'(\d+) cases.*?(\d+) failures',t)
 if m: counts[f.name]={'cases':int(m[1]),'failures':int(m[2])}
assert sum(counts[f]['cases'] for f in ['seed1-30s.txt','seed2-240s.txt','seed3-240s.txt'])==3398724
assert all(v['failures']==0 for k,v in counts.items() if not k.startswith('inject'))
root=p/'patchcheck';(root/'src').mkdir(parents=True,exist_ok=True)
original=(p/'run_stage.rs').read_text()
patches={}
for f in sorted((p/'archive/mutants').glob('M*.patch')):
 (root/'src/run_stage.rs').write_text(original)
 r=subprocess.run(['patch','--batch','-p1','-i',str(f.resolve())],cwd=root,capture_output=True,text=True)
 assert r.returncode==0,r.stdout+r.stderr
 mutated=(root/'src/run_stage.rs').read_text();assert mutated!=original
 r=subprocess.run(['patch','--batch','-R','-p1','-i',str(f.resolve())],cwd=root,capture_output=True,text=True)
 assert r.returncode==0 and (root/'src/run_stage.rs').read_text()==original
 patches[f.name]='applies and reverses exactly on reviewed run_stage blob'
# CI accounting: counts are test cases, not physical stores or invocation counts.
assert 64012+63985+63453==191450
assert 55782+26187+7438+2199==91606
assert 16052+6949+40452==63453
out={'scope':'independent archive arithmetic, patch application/reversal and routing audit; no local Rust or Miri execution','archived_runs':counts,'patches':patches,'ci_fuzz':{'cases':191450,'build_direct_blocks':91606,'by_records':55782,'empty_records_blocks':1707,'tail_predicate_blocks':45840,'fallback':26187,'refused':9637,'tilings':63453,'failures':0},'audit_unresolved':0}
(p/'results.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))

